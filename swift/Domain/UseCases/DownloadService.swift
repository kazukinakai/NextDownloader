import Foundation
import Combine

class DownloadService: ObservableObject {
    static let shared = DownloadService()
    
    @Published var downloadItems: [DownloadItem] = []
    @Published var systemStatus: SystemStatus = .unknown
    
    private let downloadManager = DownloadManager.shared
    
    private var cancellables = Set<AnyCancellable>()
    private var progressHandlers: [UUID: (Double, String, String) -> Void] = [:]
    
    private init() {
        // システムステータスを監視
        downloadManager.$systemStatus
            .receive(on: DispatchQueue.main)
            .assign(to: \.systemStatus, on: self)
            .store(in: &cancellables)
    }
    
    // 依存関係のチェック
    func checkDependencies() async -> Bool {
        downloadManager.checkSystemStatus()
        return systemStatus.isReady
    }
    
    // 新しいダウンロードを追加
    func addDownload(url: String, title: String? = nil, format: VideoFormat = .mp4, options: DownloadOptions? = nil) async {
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
        
        // 進捗ハンドラを設定
        downloadManager.setProgressHandler(for: id) { [weak self] progress, speed, eta in
            guard let self = self else { return }
            
            Task {
                await self.updateDownloadItem(id: id) { item in
                    var updatedItem = item
                    updatedItem.progress = progress
                    updatedItem.speed = speed
                    updatedItem.remainingTime = eta
                    return updatedItem
                }
            }
        }
        
        // タイトルが指定されていない場合は取得を試みる
        if title == nil {
            Task {
                do {
                    let videoTitle = try await YtDlpService.shared.getVideoTitle(url: url)
                    await updateDownloadItem(id: id) { item in
                        var updatedItem = item
                        updatedItem.title = videoTitle
                        return updatedItem
                    }
                } catch {
                    print("タイトル取得エラー: \(error.localizedDescription)")
                }
            }
        }
    }
    
    // ダウンロードを開始
    func startDownload(item: DownloadItem) {
        Task {
            await updateDownloadItem(id: item.id) { item in
                var updatedItem = item
                updatedItem.status = .downloading
                return updatedItem
            }
            
            do {
                // コンテンツタイプを検出
                let contentType = try await downloadManager.detectContentType(url: item.url)
                
                // 最適なダウンロードオプションを取得
                let options = downloadManager.optimizeDownload(url: item.url, contentType: contentType)
                
                // ダウンロードを実行
                let tempFilename = "download_\(item.id.uuidString)"
                let downloadedFileUrl = try await downloadManager.download(
                    taskId: item.id,
                    url: item.url,
                    outputPath: item.outputPath,
                    filename: tempFilename,
                    options: options
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
    
    // ダウンロードを一時停止
    func pauseDownload(item: DownloadItem) {
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
    
    // ダウンロードをキャンセル
    func cancelDownload(item: DownloadItem) {
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
    
    // 特定のダウンロードアイテムを更新
    private func updateDownloadItem(id: UUID, updateHandler: (DownloadItem) -> DownloadItem) async {
        await MainActor.run {
            if let index = downloadItems.firstIndex(where: { $0.id == id }) {
                downloadItems[index] = updateHandler(downloadItems[index])
            }
        }
    }
}
