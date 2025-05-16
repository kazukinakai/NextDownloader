import Foundation
import Combine

class DownloadManager {
    static let shared = DownloadManager()
    
    private let ytDlpService = YtDlpService.shared
    private let aria2cService = Aria2cService.shared
    private let ffmpegService = FFmpegService.shared
    private let hlsDownloadService = HlsDownloadService.shared
    
    @Published var systemStatus: SystemStatus = .unknown
    
    private init() {
        checkSystemStatus()
    }
    
    /// システムの状態を確認
    func checkSystemStatus() {
        Task {
            let ytDlpAvailable = ytDlpService.isAvailable()
            let aria2cAvailable = aria2cService.isAvailable()
            let ffmpegAvailable = ffmpegService.isAvailable()
            
            await MainActor.run {
                if ytDlpAvailable && aria2cAvailable && ffmpegAvailable {
                    systemStatus = .ready
                } else {
                    systemStatus = .missingDependencies(
                        ytDlp: ytDlpAvailable,
                        aria2c: aria2cAvailable,
                        ffmpeg: ffmpegAvailable
                    )
                }
            }
        }
    }
    
    /// コンテンツタイプを検出
    func detectContentType(url: String) async throws -> ContentType {
        // URLの拡張子をチェック
        if url.lowercased().hasSuffix(".mp4") {
            return .mp4
        } else if url.lowercased().hasSuffix(".m3u8") {
            return .hls
        } else if url.lowercased().hasSuffix(".mpd") {
            return .dash
        }
        
        // YouTube URLをチェック
        if url.contains("youtube.com") || url.contains("youtu.be") {
            return .youtube
        }
        
        // yt-dlpを使用してコンテンツタイプを検出
        do {
            let info = try await ytDlpService.getVideoInfo(url: url)
            
            // プロトコルに基づいてコンテンツタイプを決定
            if let formats = info.formats {
                for format in formats {
                    if format.url.contains(".m3u8") {
                        return .hls
                    } else if format.url.contains(".mpd") {
                        return .dash
                    }
                }
            }
            
            // デフォルトはMP4として扱う
            return .mp4
        } catch {
            print("コンテンツタイプ検出エラー: \(error.localizedDescription)")
            return .unknown
        }
    }
    
    /// コンテンツタイプに基づいて最適なダウンロードオプションを返す
    func optimizeDownload(url: String, contentType: ContentType) -> DownloadOptions {
        switch contentType {
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
    
    /// URLからダウンロード
    func download(taskId: UUID, url: String, outputPath: String, filename: String, options: DownloadOptions? = nil) async throws -> URL {
        // コンテンツタイプを検出
        let contentType = try await detectContentType(url: url)
        
        // オプションが指定されていない場合は、コンテンツタイプに基づいて最適なオプションを使用
        let downloadOptions = options ?? optimizeDownload(url: url, contentType: contentType)
        
        // コンテンツタイプに応じたダウンロード方法を選択
        switch contentType {
        case .mp4:
            return try await aria2cService.downloadVideoAdvanced(
                taskId: taskId,
                url: url,
                outputPath: outputPath,
                filename: filename,
                options: downloadOptions
            )
        case .hls:
            return try await hlsDownloadService.downloadHlsAdvanced(
                taskId: taskId,
                url: url,
                outputPath: outputPath,
                filename: filename,
                options: downloadOptions
            )
        case .dash, .youtube:
            // YouTube/DASHの場合はyt-dlpを使用
            return try await downloadWithYtDlp(
                taskId: taskId,
                url: url,
                outputPath: outputPath,
                filename: filename,
                options: downloadOptions
            )
        case .unknown:
            // 不明な場合はyt-dlpを試す
            return try await downloadWithYtDlp(
                taskId: taskId,
                url: url,
                outputPath: outputPath,
                filename: filename,
                options: downloadOptions
            )
        }
    }
    
    /// yt-dlpを使用してダウンロード
    private func downloadWithYtDlp(taskId: UUID, url: String, outputPath: String, filename: String, options: DownloadOptions) async throws -> URL {
        // yt-dlpの引数を構築
        var ytDlpArgs = [
            "--no-warnings",
            "--downloader", "aria2c",
            "--downloader-args", "aria2c:-x\(options.connections) -s\(options.splits) -k\(options.chunkSize)M",
            "-o", "\(outputPath)/\(filename).%(ext)s"
        ]
        
        // フォーマットに応じたオプションを追加
        switch options.format {
        case .mp4:
            ytDlpArgs.append("--merge-output-format")
            ytDlpArgs.append("mp4")
        case .mkv:
            ytDlpArgs.append("--merge-output-format")
            ytDlpArgs.append("mkv")
        case .mp3:
            ytDlpArgs.append("--extract-audio")
            ytDlpArgs.append("--audio-format")
            ytDlpArgs.append("mp3")
        }
        
        // URLを追加
        ytDlpArgs.append(url)
        
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/local/bin/yt-dlp")
        process.arguments = ytDlpArgs
        
        let outputPipe = Pipe()
        let errorPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = errorPipe
        
        try process.run()
        process.waitUntilExit()
        
        // 終了コードをチェック
        if process.terminationStatus != 0 {
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let errorMessage = String(data: errorData, encoding: .utf8) ?? "不明なエラー"
            throw DownloadError.processFailed("yt-dlpエラー: \(errorMessage)")
        }
        
        // ダウンロードしたファイルのURLを返す
        let fileManager = FileManager.default
        let directoryContents = try fileManager.contentsOfDirectory(atPath: outputPath)
        
        // ファイル名の先頭部分が一致するファイルを探す
        if let matchingFile = directoryContents.first(where: { $0.hasPrefix(filename) }) {
            return URL(fileURLWithPath: "\(outputPath)/\(matchingFile)")
        }
        
        throw DownloadError.fileNotFound
    }
    
    /// 進捗状況のコールバックを設定
    func setProgressHandler(for taskId: UUID, handler: @escaping (Double, String, String) -> Void) {
        aria2cService.setProgressHandler(for: taskId, handler: handler)
        hlsDownloadService.setProgressHandler(for: taskId, handler: handler)
    }
}

/// システムの状態
enum SystemStatus {
    case ready
    case missingDependencies(ytDlp: Bool, aria2c: Bool, ffmpeg: Bool)
    case unknown
    
    var isReady: Bool {
        if case .ready = self {
            return true
        }
        return false
    }
    
    var description: String {
        switch self {
        case .ready:
            return "システム準備完了"
        case .missingDependencies(let ytDlp, let aria2c, let ffmpeg):
            var missing = [String]()
            if !ytDlp { missing.append("yt-dlp") }
            if !aria2c { missing.append("aria2c") }
            if !ffmpeg { missing.append("ffmpeg") }
            return "依存関係が不足しています: \(missing.joined(separator: ", "))"
        case .unknown:
            return "システム状態不明"
        }
    }
}
