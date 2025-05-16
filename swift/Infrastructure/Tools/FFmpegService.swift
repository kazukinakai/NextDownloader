import Foundation

class FFmpegService {
    static let shared = FFmpegService()
    
    private var executableURL: URL? {
        return ExternalToolsService.shared.getToolPath(type: .ffmpeg)
    }
    private var currentTasks: [UUID: Process] = [:]
    
    private init() {}
    
    /// ffmpegが利用可能かチェック
    func isAvailable() -> Bool {
        return executableURL != nil
    }
    
    /// 動画を処理する
    func processVideo(taskId: UUID, inputUrl: URL, outputPath: String, filename: String, format: VideoFormat = .mp4) async throws -> URL {
        guard let execURL = executableURL else {
            throw DownloadError.toolNotFound("ffmpeg")
        }
        
        let outputFilename = "\(filename).\(format.rawValue)"
        let outputUrl = URL(fileURLWithPath: "\(outputPath)/\(outputFilename)")
        
        let process = Process()
        process.executableURL = execURL
        
        // ffmpegのコマンドライン引数を設定
        var arguments = [
            "-i", inputUrl.path,
            "-c:v", "copy",  // ビデオコーデックをコピー
            "-c:a", "copy",  // オーディオコーデックをコピー
            "-y"             // 既存ファイルを上書き
        ]
        
        // フォーマット固有の設定を追加
        switch format {
        case .mp4:
            arguments.append(contentsOf: ["-movflags", "faststart"])  // Web再生用に最適化
        case .mkv:
            break  // mkvの場合は特別な設定なし
        case .mp3:
            arguments.append(contentsOf: ["-vn"])  // ビデオストリームを除外
        }
        
        // 出力ファイルパスを追加
        arguments.append(outputUrl.path)
        
        process.arguments = arguments
        
        // プロセスの出力をキャプチャ
        let errorPipe = Pipe()
        process.standardError = errorPipe  // ffmpegはstandardErrorに進捗情報を出力
        
        // タスクを保存
        currentTasks[taskId] = process
        
        // プロセスを実行
        try process.run()
        
        // プロセスの終了を待機
        process.waitUntilExit()
        
        // タスクを削除
        currentTasks.removeValue(forKey: taskId)
        
        // 終了コードをチェック
        if process.terminationStatus != 0 {
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let errorMessage = String(data: errorData, encoding: .utf8) ?? "不明なエラー"
            throw DownloadError.processFailed("ffmpegエラー: \(errorMessage)")
        }
        
        return outputUrl
    }
    
    /// 動画から音声を抽出
    func extractAudio(taskId: UUID, inputUrl: URL, outputPath: String, filename: String) async throws -> URL {
        return try await processVideo(taskId: taskId, inputUrl: inputUrl, outputPath: outputPath, filename: filename, format: .mp3)
    }
    
    /// ダウンロードをキャンセル
    func cancelProcess(taskId: UUID) {
        if let process = currentTasks[taskId] {
            process.terminate()
            currentTasks.removeValue(forKey: taskId)
        }
    }
}
