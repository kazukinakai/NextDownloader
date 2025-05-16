import Foundation
import Combine

class HlsDownloadService {
    static let shared = HlsDownloadService()
    
    private let ytDlpService = YtDlpService.shared
    private let aria2cService = Aria2cService.shared
    private let ffmpegService = FFmpegService.shared
    
    private var currentTasks: [UUID: Process] = [:]
    private var progressHandlers: [UUID: (Double, String, String) -> Void] = [:]
    
    private init() {}
    
    /// HLSマニフェストを解析
    func parseManifest(url: String) async throws -> [String] {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/local/bin/yt-dlp")
        process.arguments = [
            "--dump-json",
            "--no-warnings",
            url
        ]
        
        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        
        try process.run()
        process.waitUntilExit()
        
        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: outputData, encoding: .utf8) else {
            throw DownloadError.processFailed("マニフェスト解析に失敗しました")
        }
        
        // JSONからセグメントURLを抽出
        // 実際の実装では、yt-dlpの出力からセグメントURLを抽出する処理が必要
        // この例では簡略化のため、yt-dlpの--get-urlオプションを使用
        
        return try await getSegmentUrls(url: url)
    }
    
    /// セグメントURLを取得
    private func getSegmentUrls(url: String) async throws -> [String] {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/local/bin/yt-dlp")
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
        guard let output = String(data: outputData, encoding: .utf8) else {
            throw DownloadError.processFailed("セグメントURL取得に失敗しました")
        }
        
        // 出力を行ごとに分割
        let urls = output.components(separatedBy: .newlines)
            .filter { !$0.isEmpty }
        
        return urls
    }
    
    /// HLS動画をダウンロード
    func downloadHls(taskId: UUID, url: String, outputPath: String, filename: String) async throws -> URL {
        // 進捗状況を更新
        updateProgress(taskId: taskId, progress: 0.0, speed: "解析中...", eta: "")
        
        // yt-dlpを使用してHLSをダウンロード（最も簡単な方法）
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/local/bin/yt-dlp")
        process.arguments = [
            "--no-warnings",
            "--downloader", "aria2c",
            "--downloader-args", "aria2c:-x16 -s16 -k1M --enable-http-pipelining=true --http2=true --enable-http-keep-alive=true",
            "-o", "\(outputPath)/\(filename).%(ext)s",
            url
        ]
        
        let outputPipe = Pipe()
        let errorPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = errorPipe
        
        // タスクを保存
        currentTasks[taskId] = process
        
        // プロセスを実行
        try process.run()
        
        // 進捗状況を監視
        let fileHandle = outputPipe.fileHandleForReading
        fileHandle.readabilityHandler = { [weak self] handle in
            guard let self = self else { return }
            
            let data = handle.availableData
            if data.isEmpty { return } // EOF
            
            if let output = String(data: data, encoding: .utf8) {
                // 進捗情報を抽出
                self.parseYtDlpProgress(output: output, taskId: taskId)
            }
        }
        
        // プロセスの終了を待機
        process.waitUntilExit()
        
        // タスクを削除
        currentTasks.removeValue(forKey: taskId)
        progressHandlers.removeValue(forKey: taskId)
        
        // ファイルハンドラをクリーンアップ
        fileHandle.readabilityHandler = nil
        
        // 終了コードをチェック
        if process.terminationStatus != 0 {
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let errorMessage = String(data: errorData, encoding: .utf8) ?? "不明なエラー"
            throw DownloadError.processFailed("HLSダウンロードエラー: \(errorMessage)")
        }
        
        // ダウンロードしたファイルのURLを返す
        // 注意: yt-dlpは拡張子を自動的に決定するため、実際のファイル名を確認する必要がある
        let fileManager = FileManager.default
        let directoryContents = try fileManager.contentsOfDirectory(atPath: outputPath)
        
        // ファイル名の先頭部分が一致するファイルを探す
        if let matchingFile = directoryContents.first(where: { $0.hasPrefix(filename) }) {
            return URL(fileURLWithPath: "\(outputPath)/\(matchingFile)")
        }
        
        throw DownloadError.fileNotFound
    }
    
    /// 高度なオプションでHLS動画をダウンロード
    func downloadHlsAdvanced(taskId: UUID, url: String, outputPath: String, filename: String, options: DownloadOptions) async throws -> URL {
        // 進捗状況を更新
        updateProgress(taskId: taskId, progress: 0.0, speed: "解析中...", eta: "")
        
        // yt-dlpを使用してHLSをダウンロード（高度なオプション）
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/local/bin/yt-dlp")
        
        // aria2cのオプションを構築
        var aria2cArgs = [
            "-x\(options.connections)",
            "-s\(options.splits)",
            "-k\(options.chunkSize)M",
            "--retry-wait=\(options.retryWait)",
            "--max-tries=\(options.maxRetries)"
        ]
        
        if options.useHttp2 {
            aria2cArgs.append("--enable-http-pipelining=true")
            aria2cArgs.append("--http2=true")
        }
        
        if options.useQuic {
            aria2cArgs.append("--enable-quic=true")
        }
        
        if options.useKeepAlive {
            aria2cArgs.append("--enable-http-keep-alive=true")
        }
        
        // yt-dlpの引数を構築
        var ytDlpArgs = [
            "--no-warnings",
            "--downloader", "aria2c",
            "--downloader-args", "aria2c:\(aria2cArgs.joined(separator: " "))",
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
        
        process.arguments = ytDlpArgs
        
        let outputPipe = Pipe()
        let errorPipe = Pipe()
        process.standardOutput = outputPipe
        process.standardError = errorPipe
        
        // タスクを保存
        currentTasks[taskId] = process
        
        // プロセスを実行
        try process.run()
        
        // 進捗状況を監視
        let fileHandle = outputPipe.fileHandleForReading
        fileHandle.readabilityHandler = { [weak self] handle in
            guard let self = self else { return }
            
            let data = handle.availableData
            if data.isEmpty { return } // EOF
            
            if let output = String(data: data, encoding: .utf8) {
                // 進捗情報を抽出
                self.parseYtDlpProgress(output: output, taskId: taskId)
            }
        }
        
        // プロセスの終了を待機
        process.waitUntilExit()
        
        // タスクを削除
        currentTasks.removeValue(forKey: taskId)
        progressHandlers.removeValue(forKey: taskId)
        
        // ファイルハンドラをクリーンアップ
        fileHandle.readabilityHandler = nil
        
        // 終了コードをチェック
        if process.terminationStatus != 0 {
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let errorMessage = String(data: errorData, encoding: .utf8) ?? "不明なエラー"
            throw DownloadError.processFailed("HLSダウンロードエラー: \(errorMessage)")
        }
        
        // ダウンロードしたファイルのURLを返す
        let fileManager = FileManager.default
        let directoryContents = try fileManager.contentsOfDirectory(atPath: outputPath)
        
        // ファイル名の先頭部分が一致するファイルを探す
        let expectedExtension = options.format.rawValue
        if let matchingFile = directoryContents.first(where: { 
            $0.hasPrefix(filename) && $0.hasSuffix(".\(expectedExtension)") 
        }) {
            return URL(fileURLWithPath: "\(outputPath)/\(matchingFile)")
        }
        
        throw DownloadError.fileNotFound
    }
    
    /// ダウンロードをキャンセル
    func cancelDownload(taskId: UUID) {
        if let process = currentTasks[taskId] {
            process.terminate()
            currentTasks.removeValue(forKey: taskId)
            progressHandlers.removeValue(forKey: taskId)
        }
    }
    
    /// 進捗状況のコールバックを設定
    func setProgressHandler(for taskId: UUID, handler: @escaping (Double, String, String) -> Void) {
        progressHandlers[taskId] = handler
    }
    
    /// 進捗状況を更新
    private func updateProgress(taskId: UUID, progress: Double, speed: String, eta: String) {
        if let handler = progressHandlers[taskId] {
            DispatchQueue.main.async {
                handler(progress, speed, eta)
            }
        }
    }
    
    /// yt-dlpの出力から進捗情報をパース
    private func parseYtDlpProgress(output: String, taskId: UUID) {
        // yt-dlpの出力例:
        // [download] 10.2% of 50.00MiB at 2.00MiB/s ETA 00:20
        
        // 進捗情報を抽出
        var progress: Double = 0.0
        var speed: String = ""
        var eta: String = ""
        
        // 進捗率を抽出
        if let progressMatch = output.range(of: "\\d+\\.\\d+%", options: .regularExpression) {
            let progressStr = output[progressMatch].replacingOccurrences(of: "%", with: "")
            if let progressValue = Double(progressStr) {
                progress = progressValue / 100.0
            }
        }
        
        // ダウンロード速度を抽出
        if let speedMatch = output.range(of: "at \\S+/s", options: .regularExpression) {
            speed = String(output[speedMatch].dropFirst(3))
        }
        
        // 残り時間を抽出
        if let etaMatch = output.range(of: "ETA \\S+", options: .regularExpression) {
            eta = String(output[etaMatch].dropFirst(4))
        }
        
        // 進捗状況を更新
        updateProgress(taskId: taskId, progress: progress, speed: speed, eta: eta)
    }
}
