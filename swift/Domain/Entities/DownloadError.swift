import Foundation

enum DownloadError: Error {
    case invalidUrl
    case processFailed(String)
    case fileNotFound
    case permissionDenied
    case networkError
    case unknownError
    case toolNotFound(String)
    
    var localizedDescription: String {
        switch self {
        case .invalidUrl:
            return "無効なURLです"
        case .processFailed(let message):
            return "処理に失敗しました: \(message)"
        case .fileNotFound:
            return "ファイルが見つかりません"
        case .permissionDenied:
            return "アクセス権限がありません"
        case .networkError:
            return "ネットワークエラーが発生しました"
        case .unknownError:
            return "不明なエラーが発生しました"
        case .toolNotFound(let toolName):
            return "必要なツール（\(toolName)）が見つかりません"
        }
    }
}
