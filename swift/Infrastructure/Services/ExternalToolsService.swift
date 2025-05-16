import Foundation
import Combine

/// 外部ツールの管理を担当するサービスクラス
class ExternalToolsService {
    static let shared = ExternalToolsService()
    
    // 外部ツールの種類
    enum ToolType: String, CaseIterable {
        case ytDlp = "yt-dlp"
        case aria2c = "aria2c"
        case ffmpeg = "ffmpeg"
    }
    
    // 外部ツールの情報
    struct ToolInfo {
        let type: ToolType
        let version: String
        let path: URL
        let isInstalled: Bool
    }
    
    // アプリケーションサポートディレクトリのパス
    private let appSupportDir: URL
    
    // 外部ツールのインストールディレクトリ
    private let toolsDir: URL
    
    // 現在のツール情報
    private var toolsInfo: [ToolType: ToolInfo] = [:]
    
    // ツール更新の通知用サブジェクト
    private let toolsUpdateSubject = PassthroughSubject<ToolType, Never>()
    
    // ツール更新の通知用パブリッシャー
    var toolsUpdatePublisher: AnyPublisher<ToolType, Never> {
        return toolsUpdateSubject.eraseToAnyPublisher()
    }
    
    private init() {
        // アプリケーションサポートディレクトリを取得
        let fileManager = FileManager.default
        
        // アプリケーション識別子を取得
        let bundleId = Bundle.main.bundleIdentifier ?? "com.nextdownloader.app"
        
        // アプリケーションサポートディレクトリのパスを構築
        appSupportDir = fileManager.urls(for: .applicationSupportDirectory, in: .userDomainMask).first!
            .appendingPathComponent(bundleId)
        
        // 外部ツールのインストールディレクトリを構築
        toolsDir = appSupportDir.appendingPathComponent("tools")
        
        // ディレクトリが存在しない場合は作成
        try? fileManager.createDirectory(at: toolsDir, withIntermediateDirectories: true)
        
        // 初期化時に外部ツールをセットアップ
        setupTools()
    }
    
    /// 外部ツールをセットアップ
    private func setupTools() {
        // 各ツールをセットアップ
        for toolType in ToolType.allCases {
            setupTool(type: toolType)
        }
    }
    
    /// 特定のツールをセットアップ
    private func setupTool(type: ToolType) {
        let fileManager = FileManager.default
        
        // ツールのインストール先パス
        let toolPath = toolsDir.appendingPathComponent(type.rawValue)
        
        // ツールがすでにインストールされているか確認
        let isInstalled = fileManager.fileExists(atPath: toolPath.path)
        
        // バージョン情報を初期化
        var version = "不明"
        
        if isInstalled {
            // インストール済みの場合はバージョンを取得
            version = getToolVersion(type: type, path: toolPath)
        } else {
            // インストールされていない場合はバンドルからコピー
            if copyToolFromBundle(type: type) {
                // コピー成功後にバージョンを取得
                version = getToolVersion(type: type, path: toolPath)
            }
        }
        
        // ツール情報を保存
        toolsInfo[type] = ToolInfo(
            type: type,
            version: version,
            path: toolPath,
            isInstalled: fileManager.fileExists(atPath: toolPath.path)
        )
    }
    
    /// バンドルから外部ツールをコピー
    private func copyToolFromBundle(type: ToolType) -> Bool {
        let fileManager = FileManager.default
        
        // バンドル内のツールのパス
        guard let bundlePath = Bundle.main.url(forResource: type.rawValue, withExtension: nil) else {
            print("バンドル内に\(type.rawValue)が見つかりません")
            return false
        }
        
        // コピー先のパス
        let destinationPath = toolsDir.appendingPathComponent(type.rawValue)
        
        do {
            // 既存のファイルがある場合は削除
            if fileManager.fileExists(atPath: destinationPath.path) {
                try fileManager.removeItem(at: destinationPath)
            }
            
            // ファイルをコピー
            try fileManager.copyItem(at: bundlePath, to: destinationPath)
            
            // 実行権限を付与
            try fileManager.setAttributes([.posixPermissions: 0o755], ofItemAtPath: destinationPath.path)
            
            print("\(type.rawValue)を正常にインストールしました")
            return true
        } catch {
            print("\(type.rawValue)のインストールに失敗しました: \(error.localizedDescription)")
            return false
        }
    }
    
    /// ツールのバージョンを取得
    private func getToolVersion(type: ToolType, path: URL) -> String {
        let process = Process()
        process.executableURL = path
        
        // ツールタイプに応じたバージョン取得コマンドを設定
        switch type {
        case .ytDlp:
            process.arguments = ["--version"]
        case .aria2c:
            process.arguments = ["--version"]
        case .ffmpeg:
            process.arguments = ["-version"]
        }
        
        let outputPipe = Pipe()
        process.standardOutput = outputPipe
        
        do {
            try process.run()
            process.waitUntilExit()
            
            let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: outputData, encoding: .utf8) {
                // 最初の行を取得
                let firstLine = output.components(separatedBy: .newlines).first ?? ""
                return firstLine.trimmingCharacters(in: .whitespacesAndNewlines)
            }
        } catch {
            print("\(type.rawValue)のバージョン取得に失敗しました: \(error.localizedDescription)")
        }
        
        return "不明"
    }
    
    /// 特定のツールの実行パスを取得
    func getToolPath(type: ToolType) -> URL? {
        guard let toolInfo = toolsInfo[type], toolInfo.isInstalled else {
            return nil
        }
        return toolInfo.path
    }
    
    /// 特定のツールがインストールされているか確認
    func isToolInstalled(type: ToolType) -> Bool {
        return toolsInfo[type]?.isInstalled ?? false
    }
    
    /// 特定のツールのバージョンを取得
    func getToolVersion(type: ToolType) -> String {
        return toolsInfo[type]?.version ?? "不明"
    }
    
    /// 全てのツール情報を取得
    func getAllToolsInfo() -> [ToolInfo] {
        return ToolType.allCases.compactMap { toolsInfo[$0] }
    }
    
    /// 特定のツールを更新
    func updateTool(type: ToolType) -> Bool {
        // ここでは単純にバンドルからのコピーで更新
        // 実際のアプリケーションでは、最新バージョンをダウンロードする処理を実装
        if copyToolFromBundle(type: type) {
            // ツール情報を更新
            setupTool(type: type)
            
            // 更新通知を発行
            toolsUpdateSubject.send(type)
            
            return true
        }
        return false
    }
    
    /// 全てのツールを更新
    func updateAllTools() -> [ToolType: Bool] {
        var results: [ToolType: Bool] = [:]
        
        for type in ToolType.allCases {
            results[type] = updateTool(type: type)
        }
        
        return results
    }
    
    /// 外部ツールを実行するためのProcessを作成
    func createProcess(type: ToolType, arguments: [String]) -> Process? {
        guard let toolPath = getToolPath(type: type) else {
            return nil
        }
        
        let process = Process()
        process.executableURL = toolPath
        process.arguments = arguments
        
        return process
    }
}
