import Foundation

class NativeMessagingService {
    static let shared = NativeMessagingService()
    
    // メッセージ受信時のコールバック
    var onMessageReceived: ((String, String, String) -> Void)?
    
    private init() {
        // 標準入力からのメッセージを監視
        setupStandardInputMonitoring()
    }
    
    private func setupStandardInputMonitoring() {
        // 標準入力からのメッセージを非同期で監視
        DispatchQueue.global(qos: .background).async {
            while true {
                // 最初の4バイトはメッセージの長さを表す
                var length: UInt32 = 0
                let bytesRead = FileHandle.standardInput.readData(ofLength: 4)
                
                if bytesRead.count == 4 {
                    (bytesRead as NSData).getBytes(&length, length: 4)
                    
                    // リトルエンディアンからホストのエンディアンに変換
                    #if os(OSX)
                    length = CFSwapInt32LittleToHost(length)
                    #endif
                    
                    // メッセージ本体を読み込む
                    if length > 0 {
                        let messageData = FileHandle.standardInput.readData(ofLength: Int(length))
                        if let messageString = String(data: messageData, encoding: .utf8),
                           let jsonData = messageString.data(using: .utf8) {
                            
                            do {
                                if let json = try JSONSerialization.jsonObject(with: jsonData, options: []) as? [String: Any],
                                   let url = json["url"] as? String,
                                   let title = json["title"] as? String,
                                   let referrer = json["referrer"] as? String {
                                    
                                    // メインスレッドでコールバックを呼び出す
                                    DispatchQueue.main.async {
                                        self.onMessageReceived?(url, title, referrer)
                                    }
                                }
                            } catch {
                                print("JSONパースエラー: \(error.localizedDescription)")
                            }
                        }
                    }
                } else {
                    // 標準入力が閉じられた場合は終了
                    break
                }
                
                // 少し待機してCPU使用率を抑える
                Thread.sleep(forTimeInterval: 0.1)
            }
        }
    }
    
    // Chrome拡張機能にメッセージを送信
    func sendMessage(message: [String: Any]) {
        do {
            let jsonData = try JSONSerialization.data(withJSONObject: message, options: [])
            
            // メッセージの長さを最初の4バイトとして送信（リトルエンディアン）
            var length = UInt32(jsonData.count)
            #if os(OSX)
            length = CFSwapInt32HostToLittle(length)
            #endif
            
            let lengthData = Data(bytes: &length, count: 4)
            
            // 標準出力に書き込む
            FileHandle.standardOutput.write(lengthData)
            FileHandle.standardOutput.write(jsonData)
            FileHandle.standardOutput.synchronizeFile()
        } catch {
            print("メッセージ送信エラー: \(error.localizedDescription)")
        }
    }
    
    // Native Messaging用のマニフェストファイルを生成
    func generateManifestFile(appId: String, appName: String) -> Bool {
        // 実行可能ファイルのパスを取得
        guard let executablePath = Bundle.main.executablePath else {
            return false
        }
        
        // マニフェストファイルの内容
        let manifest: [String: Any] = [
            "name": appName,
            "description": "HLS動画高速ダウンローダー",
            "path": executablePath,
            "type": "stdio",
            "allowed_origins": [
                "chrome-extension://\(appId)/"
            ]
        ]
        
        do {
            let jsonData = try JSONSerialization.data(withJSONObject: manifest, options: .prettyPrinted)
            
            // マニフェストファイルの保存先
            let fileManager = FileManager.default
            let manifestDir = fileManager.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support/Google/Chrome/NativeMessagingHosts")
            
            // ディレクトリが存在しない場合は作成
            if !fileManager.fileExists(atPath: manifestDir.path) {
                try fileManager.createDirectory(at: manifestDir, withIntermediateDirectories: true)
            }
            
            let manifestPath = manifestDir.appendingPathComponent("\(appName).json")
            
            // ファイルに書き込み
            try jsonData.write(to: manifestPath)
            return true
        } catch {
            print("マニフェスト生成エラー: \(error.localizedDescription)")
            return false
        }
    }
}
