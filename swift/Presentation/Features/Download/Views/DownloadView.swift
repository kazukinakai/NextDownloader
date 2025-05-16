import SwiftUI

struct DownloadView: View {
    @StateObject private var viewModel = DownloadViewModel()
    @State private var videoUrl: String = ""
    @State private var selectedFormat: VideoFormat = .mp4
    @State private var showingAlert: Bool = false
    @State private var alertMessage: String = ""
    
    var body: some View {
        VStack(spacing: 20) {
            // タイトル
            Text("NextDownloader - HLS動画高速ダウンローダー")
                .font(.largeTitle)
                .fontWeight(.bold)
            
            // URL入力フィールド
            HStack {
                TextField("動画URLを入力してください", text: $videoUrl)
                    .textFieldStyle(RoundedBorderTextFieldStyle())
                
                Button(action: {
                    // クリップボードからURLを貼り付け
                    if let clipboardString = NSPasteboard.general.string(forType: .string) {
                        videoUrl = clipboardString
                    }
                }) {
                    Text("貼り付け")
                }
            }
            .padding(.horizontal)
            
            // フォーマット選択
            HStack {
                Text("フォーマット:")
                Picker("フォーマット", selection: $selectedFormat) {
                    Text("MP4 動画").tag(VideoFormat.mp4)
                    Text("MKV 動画").tag(VideoFormat.mkv)
                    Text("MP3 音声のみ").tag(VideoFormat.mp3)
                }
                .pickerStyle(SegmentedPickerStyle())
                .frame(width: 300)
            }
            .padding(.horizontal)
            
            // ダウンロードボタン
            Button(action: {
                startDownload()
            }) {
                Text("ダウンロード開始")
                    .frame(width: 200)
                    .padding()
                    .background(viewModel.isLoading ? Color.gray : Color.blue)
                    .foregroundColor(.white)
                    .cornerRadius(10)
            }
            .disabled(viewModel.isLoading || videoUrl.isEmpty)
            
            // 依存関係エラー表示
            if let errorMessage = viewModel.errorMessage {
                VStack {
                    Text(errorMessage)
                        .foregroundColor(.red)
                        .padding()
                    
                    Button("依存関係をインストール") {
                        // 依存関係のインストール処理（実際の実装では、ここでインストールスクリプトを実行）
                        alertMessage = "インストール機能は現在実装中です。手動でインストールしてください。"
                        showingAlert = true
                    }
                    .buttonStyle(.borderedProminent)
                }
                .padding()
                .background(Color.red.opacity(0.1))
                .cornerRadius(10)
            }
            
            // ダウンロード履歴
            VStack(alignment: .leading) {
                Text("ダウンロード履歴")
                    .font(.headline)
                
                List {
                    ForEach(viewModel.downloadItems) { item in
                        HStack {
                            VStack(alignment: .leading) {
                                Text(item.title)
                                    .fontWeight(.semibold)
                                Text(item.url)
                                    .font(.caption)
                                    .foregroundColor(.gray)
                            }
                            
                            Spacer()
                            
                            // 進捗バー
                            if item.status.isActive {
                                ProgressView(value: item.progress, total: 1.0)
                                    .frame(width: 100)
                            }
                            
                            // ステータス
                            Text(item.status.rawValue)
                                .foregroundColor(statusColor(for: item.status))
                                .frame(width: 100)
                            
                            // 操作ボタン
                            HStack(spacing: 8) {
                                if item.status.canPause {
                                    Button(action: {
                                        viewModel.pauseDownload(item: item)
                                    }) {
                                        Image(systemName: "pause.circle")
                                    }
                                    .buttonStyle(BorderlessButtonStyle())
                                }
                                
                                if item.status.canResume {
                                    Button(action: {
                                        viewModel.startDownload(item: item)
                                    }) {
                                        Image(systemName: "play.circle")
                                    }
                                    .buttonStyle(BorderlessButtonStyle())
                                }
                                
                                if item.status.canCancel {
                                    Button(action: {
                                        viewModel.cancelDownload(item: item)
                                    }) {
                                        Image(systemName: "xmark.circle")
                                            .foregroundColor(.red)
                                    }
                                    .buttonStyle(BorderlessButtonStyle())
                                }
                                
                                if item.status == .completed {
                                    Button(action: {
                                        // ファイルを開く
                                        NSWorkspace.shared.open(URL(fileURLWithPath: item.outputPath))
                                    }) {
                                        Image(systemName: "folder")
                                    }
                                    .buttonStyle(BorderlessButtonStyle())
                                }
                            }
                        }
                        .padding(.vertical, 4)
                    }
                }
                .frame(minHeight: 200)
            }
            .padding()
        }
        .padding()
        .alert(isPresented: $showingAlert) {
            Alert(title: Text("通知"), message: Text(alertMessage), dismissButton: .default(Text("OK")))
        }
        .onAppear {
            // 依存関係のチェック
            viewModel.checkDependencies()
        }
    }
    
    private func startDownload() {
        guard !videoUrl.isEmpty else { return }
        
        // ViewModel経由でダウンロードを追加
        viewModel.addDownload(url: videoUrl, format: selectedFormat)
        
        // URLをクリア
        videoUrl = ""
    }
    
    private func statusColor(for status: DownloadStatus) -> Color {
        switch status {
        case .completed:
            return .green
        case .failed, .cancelled:
            return .red
        case .downloading, .processing:
            return .blue
        case .paused:
            return .orange
        case .pending:
            return .gray
        }
    }
}

struct DownloadView_Previews: PreviewProvider {
    static var previews: some View {
        DownloadView()
    }
}
