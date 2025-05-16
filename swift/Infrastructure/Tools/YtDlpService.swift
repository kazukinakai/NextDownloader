import Foundation

class YtDlpService {
    static let shared = YtDlpService()
    
    private var executableURL: URL? {
        return ExternalToolsService.shared.getToolPath(type: .ytDlp)
    }
    
    private init() {}
    
    /// yt-dlpが利用可能かチェック
    func isAvailable() -> Bool {
        return executableURL != nil
    }
    
    /// 動画情報を取得
    func getVideoInfo(url: String) async throws -> VideoInfo {
        guard let execURL = executableURL else {
            throw DownloadError.toolNotFound("yt-dlp")
        }
        
        let process = Process()
        process.executableURL = execURL
        process.arguments = [
            "-J",  // JSON形式で出力
            "--no-warnings",
            url
        ]
        
        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        
        try process.run()
        process.waitUntilExit()
        
        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: outputData, encoding: .utf8) else {
            throw DownloadError.processFailed("出力を解析できませんでした")
        }
        
        // JSONデコード処理
        guard let data = output.data(using: .utf8) else {
            throw DownloadError.processFailed("JSONデータに変換できませんでした")
        }
        
        do {
            let decoder = JSONDecoder()
            let videoInfo = try decoder.decode(VideoInfo.self, from: data)
            return videoInfo
        } catch {
            throw DownloadError.processFailed("JSONデコードに失敗しました: \(error.localizedDescription)")
        }
    }
    
    /// 動画URLを取得
    func getVideoUrl(url: String) async throws -> String {
        guard let execURL = executableURL else {
            throw DownloadError.toolNotFound("yt-dlp")
        }
        
        let process = Process()
        process.executableURL = execURL
        process.arguments = [
            "--get-url",
            "--no-warnings",
            url
        ]
        
        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        
        try process.run()
        process.waitUntilExit()
        
        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        guard let videoUrl = String(data: outputData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines),
              !videoUrl.isEmpty else {
            throw DownloadError.processFailed("動画URLを取得できませんでした")
        }
        
        return videoUrl
    }
    
    /// 動画のタイトルを取得
    func getVideoTitle(url: String) async throws -> String {
        guard let execURL = executableURL else {
            throw DownloadError.toolNotFound("yt-dlp")
        }
        
        let process = Process()
        process.executableURL = execURL
        process.arguments = [
            "--get-title",
            "--no-warnings",
            url
        ]
        
        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        
        try process.run()
        process.waitUntilExit()
        
        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        guard let title = String(data: outputData, encoding: .utf8)?.trimmingCharacters(in: .whitespacesAndNewlines),
              !title.isEmpty else {
            throw DownloadError.processFailed("動画タイトルを取得できませんでした")
        }
        
        return title
    }
}
