import Foundation

struct DownloadOptions {
    // 接続設定
    var connections: Int = 16        // 並列コネクション数 (-x)
    var splits: Int = 16             // 分割数 (-s)
    var chunkSize: Int = 4           // チャンクサイズ (MB) (-k)
    
    // リトライ設定
    var retryWait: Int = 2           // リトライ待機時間 (秒)
    var maxRetries: Int = 5          // 最大リトライ回数
    
    // プロトコル設定
    var useHttp2: Bool = true        // HTTP/2を使用
    var useQuic: Bool = false        // QUIC (HTTP/3) を使用
    var useKeepAlive: Bool = true    // Keep-Aliveを使用
    
    // 出力設定
    var format: VideoFormat = .mp4   // 出力フォーマット
    
    // プリセット
    static let `default` = DownloadOptions()
    
    // MP4直リンク用の最適設定
    static let mp4 = DownloadOptions(
        connections: 16,
        splits: 16,
        chunkSize: 4,
        retryWait: 2,
        maxRetries: 5,
        useHttp2: true,
        useKeepAlive: true
    )
    
    // HLS用の最適設定
    static let hls = DownloadOptions(
        connections: 16,
        splits: 16,
        chunkSize: 1,
        retryWait: 1,
        maxRetries: 10,
        useHttp2: true,
        useKeepAlive: true
    )
    
    // DASH用の最適設定
    static let dash = DownloadOptions(
        connections: 8,
        splits: 8,
        chunkSize: 1,
        retryWait: 2,
        maxRetries: 5,
        useHttp2: true,
        useQuic: true,
        useKeepAlive: true
    )
    
    // 低速接続用の設定
    static let lowBandwidth = DownloadOptions(
        connections: 4,
        splits: 4,
        chunkSize: 1,
        retryWait: 5,
        maxRetries: 10,
        useHttp2: true,
        useKeepAlive: true
    )
    
    // 高速接続用の設定
    static let highBandwidth = DownloadOptions(
        connections: 32,
        splits: 32,
        chunkSize: 8,
        retryWait: 1,
        maxRetries: 3,
        useHttp2: true,
        useQuic: true,
        useKeepAlive: true
    )
}
