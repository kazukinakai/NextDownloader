import Foundation

struct DownloadItem: Identifiable, Equatable {
    let id: UUID
    let url: String
    let title: String
    var status: DownloadStatus
    let outputPath: String
    let timestamp: Date = Date()
    let format: VideoFormat
    
    // 進捗情報
    var progress: Double = 0.0
    var speed: String = ""
    var remainingTime: String = ""
    var fileSize: Int64 = 0
    var downloadedSize: Int64 = 0
    
    // メタデータ
    var thumbnail: URL?
    var duration: TimeInterval?
    var author: String?
    
    static func == (lhs: DownloadItem, rhs: DownloadItem) -> Bool {
        return lhs.id == rhs.id
    }
}

enum DownloadStatus: String, Codable {
    case pending = "保留中"
    case downloading = "ダウンロード中"
    case processing = "処理中"
    case paused = "一時停止"
    case completed = "完了"
    case failed = "失敗"
    case cancelled = "キャンセル"
    
    var isActive: Bool {
        return self == .downloading || self == .processing
    }
    
    var canPause: Bool {
        return self == .downloading
    }
    
    var canResume: Bool {
        return self == .paused
    }
    
    var canCancel: Bool {
        return self == .downloading || self == .processing || self == .paused || self == .pending
    }
}
