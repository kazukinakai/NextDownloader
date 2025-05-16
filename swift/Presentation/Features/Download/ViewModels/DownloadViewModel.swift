import Foundation
import Combine
import SwiftUI

@MainActor
class DownloadViewModel: ObservableObject {
    @Published var downloadItems: [DownloadItem] = []
    @Published var isLoading: Bool = false
    @Published var errorMessage: String? = nil
    @Published var selectedDownloadOption: DownloadOptionPreset = .default
    @Published var contentTypeInfo: ContentType = .unknown
    
    private let downloadService = DownloadService.shared
    private let downloadManager = DownloadManager.shared
    private var cancellables = Set<AnyCancellable>()
    
    init() {
        setupBindings()
        checkDependencies()
    }
    
    private func setupBindings() {
        downloadService.$downloadItems
            .receive(on: DispatchQueue.main)
            .assign(to: \.downloadItems, on: self)
            .store(in: &cancellables)
        
        downloadService.$systemStatus
            .receive(on: DispatchQueue.main)
            .map { status -> String? in
                if case .missingDependencies = status {
                    return status.description
                }
                return nil
            }
            .assign(to: \.errorMessage, on: self)
            .store(in: &cancellables)
    }
    
    func checkDependencies() {
        Task {
            isLoading = true
            let available = await downloadService.checkDependencies()
            isLoading = false
        }
    }
    
    /// URLからコンテンツタイプを検出
    func detectContentType(url: String) async {
        guard !url.isEmpty else { return }
        
        isLoading = true
        do {
            contentTypeInfo = try await downloadManager.detectContentType(url: url)
            // コンテンツタイプに基づいて最適なダウンロードオプションを選択
            switch contentTypeInfo {
            case .mp4:
                selectedDownloadOption = .mp4Direct
            case .hls:
                selectedDownloadOption = .hls
            case .dash:
                selectedDownloadOption = .dash
            case .youtube:
                selectedDownloadOption = .youtube
            case .unknown:
                selectedDownloadOption = .default
            }
        } catch {
            errorMessage = "コンテンツタイプの検出に失敗しました: \(error.localizedDescription)"
        }
        isLoading = false
    }
    
    func addDownload(url: String, title: String? = nil, format: VideoFormat = .mp4) {
        guard !url.isEmpty else { return }
        
        Task {
            // 選択されたダウンロードオプションを取得
            let options = selectedDownloadOption.options
            
            // フォーマットを反映したオプションを作成
            var updatedOptions = options
            updatedOptions.format = format
            
            await downloadService.addDownload(url: url, title: title, format: format, options: updatedOptions)
        }
    }
    
    func startDownload(item: DownloadItem) {
        downloadService.startDownload(item: item)
    }
    
    func pauseDownload(item: DownloadItem) {
        downloadService.pauseDownload(item: item)
    }
    
    func cancelDownload(item: DownloadItem) {
        downloadService.cancelDownload(item: item)
    }
    
    func clearErrorMessage() {
        errorMessage = nil
    }
}

/// ダウンロードオプションのプリセット
/// UIで選択しやすいようにプリセットとして提供
 enum DownloadOptionPreset: String, CaseIterable, Identifiable {
    case `default` = "標準"
    case mp4Direct = "MP4直リンク最適化"
    case hls = "HLS最適化"
    case dash = "DASH最適化"
    case youtube = "YouTube最適化"
    case lowBandwidth = "低速回線向け"
    case highBandwidth = "高速回線向け"
    
    var id: String { self.rawValue }
    
    var options: DownloadOptions {
        switch self {
        case .default:
            return .default
        case .mp4Direct:
            return .mp4
        case .hls:
            return .hls
        case .dash:
            return .dash
        case .youtube:
            return .default
        case .lowBandwidth:
            return .lowBandwidth
        case .highBandwidth:
            return .highBandwidth
        }
    }
    
    var description: String {
        switch self {
        case .default:
            return "標準設定 (16並列)"
        case .mp4Direct:
            return "MP4直リンク最適化 (16並列, 4MBチャンク)"
        case .hls:
            return "HLS最適化 (16並列, 1MBチャンク)"
        case .dash:
            return "DASH最適化 (8並列, HTTP/2)"
        case .youtube:
            return "YouTube最適化 (16並列)"
        case .lowBandwidth:
            return "低速回線向け (4並列, 1MBチャンク)"
        case .highBandwidth:
            return "高速回線向け (32並列, 8MBチャンク, HTTP/3)"
        }
    }
}
