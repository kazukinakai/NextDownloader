import Foundation
import Combine

class SettingsViewModel: ObservableObject {
    @Published var downloadPath: String = ""
    @Published var useYtDlp: Bool = true
    @Published var useAria2c: Bool = true
    @Published var useFfmpeg: Bool = true
    
    private var cancellables = Set<AnyCancellable>()
    
    init() {
        loadSettings()
    }
    
    private func loadSettings() {
        // UserDefaultsから設定を読み込む
        let defaults = UserDefaults.standard
        downloadPath = defaults.string(forKey: "downloadPath") ?? NSHomeDirectory() + "/Downloads"
        useYtDlp = defaults.bool(forKey: "useYtDlp")
        useAria2c = defaults.bool(forKey: "useAria2c")
        useFfmpeg = defaults.bool(forKey: "useFfmpeg")
    }
    
    func saveSettings() {
        // UserDefaultsに設定を保存
        let defaults = UserDefaults.standard
        defaults.set(downloadPath, forKey: "downloadPath")
        defaults.set(useYtDlp, forKey: "useYtDlp")
        defaults.set(useAria2c, forKey: "useAria2c")
        defaults.set(useFfmpeg, forKey: "useFfmpeg")
    }
    
    func selectDownloadPath() {
        // ファイル選択ダイアログを表示する処理
        // 実際の実装ではNSOpenPanelなどを使用
    }
}
