import Foundation

enum VideoFormat: String, Codable {
    case mp4
    case mkv
    case mp3
    
    var description: String {
        switch self {
        case .mp4:
            return "MP4 動画"
        case .mkv:
            return "MKV 動画"
        case .mp3:
            return "MP3 音声"
        }
    }
}
