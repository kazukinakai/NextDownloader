import Foundation
import Combine

class Aria2cService {
    static let shared = Aria2cService()
    
    private var executableURL: URL? {
        return ExternalToolsService.shared.getToolPath(type: .aria2c)
    }
    private var currentTasks: [UUID: Process] = [:]
    private var progressHandlers: [UUID: (Double, String, String) -> Void] = [:]
    
    private init() {}
    
    /// aria2cが利用可能かチェック
    func isAvailable() -> Bool {
        return executableURL != nil
    }
    
    /// aria2cのバージョンを取得
    func getVersion() async throws -> String {
        guard let execURL = executableURL else {
            throw DownloadError.toolNotFound("aria2c")
        }
        
        let process = Process()
        process.executableURL = execURL
        process.arguments = ["--version"]
        
        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        
        try process.run()
        process.waitUntilExit()
        
        let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
        guard let output = String(data: outputData, encoding: .utf8) else {
            throw DownloadError.processFailed("バージョン情報を取得できませんでした")
        }
        
        // 最初の行にバージョン情報が含まれている
        let lines = output.components(separatedBy: .newlines)
        if let firstLine = lines.first, firstLine.contains("aria2") {
            return firstLine
        }
        
        return "不明なバージョン"
    }
    
    /// 動画をダウンロード（基本設定）
    func downloadVideo(taskId: UUID, url: String, outputPath: String, filename: String) async throws -> URL {
        return try await downloadVideoAdvanced(
            taskId: taskId,
            url: url,
            outputPath: outputPath,
            filename: filename,
            options: .default
        )
    }
    
    /// 動画をダウンロード（高度な設定）
    func downloadVideoAdvanced(taskId: UUID, url: String, outputPath: String, filename: String, options: DownloadOptions) async throws -> URL {
        guard let execURL = executableURL else {
            throw DownloadError.toolNotFound("aria2c")
        }
        
        let process = Process()
        process.executableURL = execURL
        
        // 基本的な引数
        var arguments = [
            "-x\(options.connections)",      // 並列コネクション数
            "-s\(options.splits)",          // 分割数
            "-k\(options.chunkSize)M",      // チャンクサイズ
            "--retry-wait=\(options.retryWait)",  // リトライ間隔
            "--max-tries=\(options.maxRetries)",  // 最大リトライ回数
            "--dir=\(outputPath)",
            "--out=\(filename).\(options.format.rawValue)",
            "--summary-interval=1",          // 進捗サマリーの更新間隔
            "--download-result=full",        // 詳細な結果を表示
            "--file-allocation=none"         // 高速化のためにファイル事前割り当てを無効化
        ]
        
        // HTTP/2サポート
        if options.useHttp2 {
            arguments.append("--enable-http-pipelining=true")
            arguments.append("--http2=true")
        }
        
        // QUICサポート（HTTP/3）
        if options.useQuic {
            arguments.append("--enable-quic=true")
        }
        
        // Keep-Aliveサポート
        if options.useKeepAlive {
            arguments.append("--enable-http-keep-alive=true")
        }
        
        // URLを追加
        arguments.append(url)
        
        // プロセスの出力をキャプチャ
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
                // aria2cの出力から進捗情報を抽出
                self.parseProgress(output: output, taskId: taskId)
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
            throw DownloadError.processFailed("aria2cエラー: \(errorMessage)")
        }
        
        // ダウンロードしたファイルのURLを返す
        let downloadedFilePath = "\(outputPath)/\(filename).\(options.format.rawValue)"
        return URL(fileURLWithPath: downloadedFilePath)
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
    
    /// aria2cの出力から進捗情報をパース
    private func parseProgress(output: String, taskId: UUID) {
        // aria2cの出力例:
        // [#7a6e1a 16MiB/341MiB(4%) CN:8 DL:6.2MiB ETA:53s]
        
        // 進捗情報を抽出
        var progress: Double = 0.0
        var speed: String = ""
        var eta: String = ""
        
        // 進捗率を抽出
        if let progressMatch = output.range(of: "\\d+%", options: .regularExpression) {
            let progressStr = output[progressMatch].replacingOccurrences(of: "%", with: "")
            if let progressValue = Double(progressStr) {
                progress = progressValue / 100.0
            }
        }
        
        // ダウンロード速度を抽出
        if let speedMatch = output.range(of: "DL:\\S+", options: .regularExpression) {
            speed = String(output[speedMatch].dropFirst(3))
        }
        
        // 残り時間を抽出
        if let etaMatch = output.range(of: "ETA:\\S+", options: .regularExpression) {
            eta = String(output[etaMatch].dropFirst(4))
        }
        
        // コールバックを呼び出す
        if let handler = progressHandlers[taskId] {
            DispatchQueue.main.async {
                handler(progress, speed, eta)
            }
        }
    }
}
