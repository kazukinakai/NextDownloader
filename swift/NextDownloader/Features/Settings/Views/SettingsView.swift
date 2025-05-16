import SwiftUI

struct SettingsView: View {
    @StateObject private var viewModel = SettingsViewModel()
    
    var body: some View {
        Form {
            Section(header: Text("ダウンロード設定")) {
                HStack {
                    Text("保存先")
                    Spacer()
                    Text(viewModel.downloadPath)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Button("選択") {
                        viewModel.selectDownloadPath()
                    }
                }
            }
            
            Section(header: Text("使用ツール")) {
                Toggle("yt-dlp", isOn: $viewModel.useYtDlp)
                Toggle("aria2c", isOn: $viewModel.useAria2c)
                Toggle("ffmpeg", isOn: $viewModel.useFfmpeg)
            }
            
            Section {
                Button("設定を保存") {
                    viewModel.saveSettings()
                }
                .frame(maxWidth: .infinity)
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .frame(minWidth: 400, minHeight: 300)
    }
}
