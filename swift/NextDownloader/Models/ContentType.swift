import Foundation

enum ContentType: String, Codable {
    case mp4 = "mp4"           // 直接MP4ファイル
    case hls = "hls"           // HLS (m3u8)
    case dash = "dash"         // MPEG-DASH (mpd)
    case youtube = "youtube"   // YouTube
    case unknown = "unknown"   // 不明なタイプ
    
    var description: String {
        switch self {
        case .mp4:
            return "MP4 動画"
        case .hls:
            return "HLS ストリーミング"
        case .dash:
            return "MPEG-DASH ストリーミング"
        case .youtube:
            return "YouTube 動画"
        case .unknown:
            return "不明なフォーマット"
        }
    }
    
    var defaultOptions: DownloadOptions {
        switch self {
        case .mp4:
            return .mp4
        case .hls:
            return .hls
        case .dash:
            return .dash
        case .youtube:
            return .default
        case .unknown:
            return .default
        }
    }
}
