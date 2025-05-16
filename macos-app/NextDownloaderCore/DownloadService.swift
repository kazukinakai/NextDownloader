import Foundation
import Combine

/// ダウンロードサービス - Rustコアとの連携を担当
public class DownloadService: ObservableObject {
    /// シングルトンインスタンス
    public static let shared = DownloadService()
    
    /// ダウンロードアイテムリスト
    @Published public var downloadItems: [DownloadItem] = []
    
    /// システム状態
    @Published public var systemStatus: SystemStatus = .unknown
    
    /// Rustコアとのブリッジ
    private let core = NextDownloaderCore.shared
    
    /// キャンセルトークン
    private var cancellables = Set<AnyCancellable>()
    
    /// 進捗ハンドラ
    private var progressHandlers: [UUID: (Double, String, String) -> Void] = [:]
    
    /// 初期化
    private init() {
        // 初期化時にシステム状態をチェック
        Task {
            await checkDependencies()
        }
    }
    
    /// 依存関係のチェック
    public func checkDependencies() async -> Bool {
        let dependencies = core.checkDependencies()
        
        await MainActor.run {
            if dependencies.ytdlp && dependencies.aria2c && dependencies.ffmpeg {
                systemStatus = .ready
            } else {
                systemStatus = .missingDependencies(
                    ytdlp: dependencies.ytdlp,
                    aria2c: dependencies.aria2c,
                    ffmpeg: dependencies.ffmpeg
                )
            }
        }
        
        return systemStatus.isReady
    }
    
    /// 新しいダウンロードを追加
    public func addDownload(url: String, title: String? = nil, format: VideoFormat = .mp4, options: DownloadOptions? = nil) async {
        let id = UUID()
        let documentsPath = FileManager.default.urls(for: .downloadsDirectory, in: .userDomainMask).first!.path
        
        // 初期状態のダウンロードアイテムを作成
        let downloadItem = DownloadItem(
            id: id,
            url: url,
            title: title ?? "読み込み中...",
            status: .pending,
            outputPath: documentsPath,
            format: format
        )
        
        // UIを更新するためにメインスレッドで実行
        await MainActor.run {
            downloadItems.append(downloadItem)
        }
        
        // タイトルが指定されていない場合は取得を試みる
        if title == nil {
            Task {
                do {
                    if let videoInfo = try await core.getVideoInfo(url: url),
                       let videoTitle = videoInfo.title {
                        await updateDownloadItem(id: id) { item in
                            var updatedItem = item
                            updatedItem.title = videoTitle
                            return updatedItem
                        }
                    }
                } catch {
                    print("タイトル取得エラー: \(error.localizedDescription)")
                }
            }
        }
    }
    
    /// ダウンロードを開始
    public func startDownload(item: DownloadItem) {
        Task {
            await updateDownloadItem(id: item.id) { item in
                var updatedItem = item
                updatedItem.status = .downloading
                return updatedItem
            }
            
            do {
                // コンテンツタイプを検出
                let contentType = core.detectContentType(url: item.url)
                
                // 最適なダウンロードオプションを取得
                let options = contentType.defaultOptions
                
                // 進捗ハンドラを設定
                let progressHandler: ((Double, String, String) -> Void)? = { [weak self] progress, speed, eta in
                    guard let self = self else { return }
                    
                    Task {
                        await self.updateDownloadItem(id: item.id) { item in
                            var updatedItem = item
                            updatedItem.progress = progress
                            updatedItem.speed = speed
                            updatedItem.remainingTime = eta
                            return updatedItem
                        }
                    }
                }
                
                // ダウンロードを実行
                let tempFilename = "download_\(item.id.uuidString)"
                let downloadedFilePath = try await core.downloadVideo(
                    url: item.url,
                    outputPath: item.outputPath,
                    filename: tempFilename,
                    options: options,
                    progressCallback: progressHandler
                )
                
                // ダウンロード完了を通知
                await updateDownloadItem(id: item.id) { item in
                    var updatedItem = item
                    updatedItem.status = .completed
                    updatedItem.progress = 1.0
                    updatedItem.speed = ""
                    updatedItem.remainingTime = ""
                    return updatedItem
                }
                
                print("ダウンロード完了: \(downloadedFilePath)")
                
            } catch {
                print("ダウンロードエラー: \(error.localizedDescription)")
                
                // エラーを通知
                await updateDownloadItem(id: item.id) { item in
                    var updatedItem = item
                    updatedItem.status = .failed
                    updatedItem.speed = ""
                    updatedItem.remainingTime = ""
                    return updatedItem
                }
            }
        }
    }
    
    /// ダウンロードを一時停止
    public func pauseDownload(item: DownloadItem) {
        // 現在のバージョンでは一時停止機能は実装されていません
        // 将来的に実装する場合は、aria2cのセッション機能を利用する
        
        Task {
            await updateDownloadItem(id: item.id) { item in
                var updatedItem = item
                updatedItem.status = .paused
                return updatedItem
            }
        }
    }
    
    /// ダウンロードをキャンセル
    public func cancelDownload(item: DownloadItem) {
        Task {
            await updateDownloadItem(id: item.id) { item in
                var updatedItem = item
                updatedItem.status = .cancelled
                updatedItem.speed = ""
                updatedItem.remainingTime = ""
                return updatedItem
            }
        }
    }
    
    /// 特定のダウンロードアイテムを更新
    private func updateDownloadItem(id: UUID, updateHandler: (DownloadItem) -> DownloadItem) async {
        await MainActor.run {
            if let index = downloadItems.firstIndex(where: { $0.id == id }) {
                downloadItems[index] = updateHandler(downloadItems[index])
            }
        }
    }
}
