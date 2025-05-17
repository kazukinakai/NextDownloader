import SwiftUI

@main
struct NextDownloaderApp: App {
    @StateObject private var downloadService = DownloadService.shared
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(downloadService)
                .frame(minWidth: 800, minHeight: 600)
        }
        .windowStyle(.hiddenTitleBar)
        .commands {
            CommandGroup(replacing: .newItem) {
                Button("新規ダウンロード") {
                    NotificationCenter.default.post(name: .newDownload, object: nil)
                }
                .keyboardShortcut("n", modifiers: .command)
            }
            
            CommandGroup(after: .newItem) {
                Divider()
                Button("依存関係のチェック") {
                    Task {
                        _ = await DownloadService.shared.checkDependencies()
                    }
                }
                .keyboardShortcut("r", modifiers: [.command, .shift])
            }
        }
    }
}

// 通知名の拡張
extension Notification.Name {
    static let newDownload = Notification.Name("newDownload")
}
