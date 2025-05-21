import Cocoa
import FlutterMacOS
import macos_window_utils

class MainFlutterWindow: NSWindow {
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    
    // ウィンドウの初期サイズを設定
    self.setFrame(NSRect(x: windowFrame.origin.x, y: windowFrame.origin.y, width: 1024, height: 768), display: true)
    
    // ウィンドウの最小サイズを設定
    self.minSize = NSSize(width: 800, height: 600)
    
    // ウィンドウのタイトルを設定
    self.title = "NextDownloader"
    
    // ウィンドウのスタイルを設定
    self.styleMask = [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView]
    
    // タイトルバーを透明にする
    self.titlebarAppearsTransparent = true
    
    // タイトルバーの高さを設定
    self.titleVisibility = .visible
    
    // ウィンドウの背景色を設定
    self.backgroundColor = NSColor.windowBackgroundColor
    
    // ウィンドウのドックアイコンを設定
    // NSApp.applicationIconImage = NSImage(named: "AppIcon")
    
    // macos_window_utilsの初期化
    MacOSWindowUtils.createMacOSWindowUtilsIfNeeded()

    RegisterGeneratedPlugins(registry: flutterViewController)

    super.awakeFromNib()
  }
  
  // ウィンドウが閉じられる前に呼び出される
  override func close() {
    // アプリ終了前の処理を行う場所
    super.close()
  }
}
