import SwiftUI

@main
struct NextDownloaderApp: App {
    var body: some Scene {
        WindowGroup {
            DownloadView()
                .frame(minWidth: 900, minHeight: 700)
                .preferredColorScheme(.dark) // ダークモードをデフォルトに設定
        }
        .windowStyle(HiddenTitleBarWindowStyle())
        .windowToolbarStyle(UnifiedCompactWindowToolbarStyle())
        .commands {
            // 不要なメニュー項目を削除
            CommandGroup(replacing: .newItem) { }
            CommandGroup(replacing: .undoRedo) { }
            
            // アプリケーション固有のコマンド
            CommandMenu("ダウンロード") {
                Button("新規ダウンロード") {
                    NSPasteboard.general.setString("", forType: .string)
                }
                .keyboardShortcut("n", modifiers: [.command])
                
                Button("依存関係のチェック") {
                    // 依存関係のチェック処理
                }
                .keyboardShortcut("d", modifiers: [.command])
            }
        }
    }
}
