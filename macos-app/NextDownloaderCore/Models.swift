import Foundation

/// ダウンロードするコンテンツのタイプ
public enum ContentType: String, Codable {
    /// 通常のMP4ファイル
    case mp4
    /// HLSストリーミング (m3u8)
    case hls
    /// MPEG-DASHストリーミング (mpd)
    case dash
    /// YouTube動画
    case youtube
    /// 不明なタイプ
    case unknown
    
    /// コンテンツタイプの説明
    public var description: String {
        switch self {
        case .mp4:
            return "MP4 動画"
        case .hls:
            return "HLS ストリーミング"
        case .dash:
            return "MPEG-DASH ストリーミング"
        case .youtube:
            return "YouTube 動画"
        case .unknown:
            return "不明なフォーマット"
        }
    }
    
    /// デフォルトのダウンロードオプション
    public var defaultOptions: DownloadOptions {
        switch self {
        case .mp4:
            return DownloadOptions.mp4
        case .hls:
            return DownloadOptions.hls
        case .dash:
            return DownloadOptions.dash
        case .youtube, .unknown:
            return DownloadOptions.default
        }
    }
}

/// 動画フォーマット
public enum VideoFormat: String, Codable {
    /// MP4フォーマット
    case mp4
    /// MKVフォーマット
    case mkv
    /// MP3フォーマット (音声のみ)
    case mp3
}

/// ダウンロードオプション
public struct DownloadOptions: Codable {
    /// 並列コネクション数
    public var connections: Int
    /// ファイル分割数
    public var splits: Int
    /// チャンクサイズ (MB)
    public var chunkSize: Int
    /// リトライ待機時間 (秒)
    public var retryWait: Int
    /// 最大リトライ回数
    public var maxRetries: Int
    /// HTTP/2を使用
    public var useHttp2: Bool
    /// QUIC (HTTP/3)を使用
    public var useQuic: Bool
    /// Keep-Aliveを使用
    public var useKeepAlive: Bool
    /// 出力フォーマット
    public var format: VideoFormat
    
    /// 初期化
    public init(
        connections: Int = 16,
        splits: Int = 16,
        chunkSize: Int = 4,
        retryWait: Int = 2,
        maxRetries: Int = 5,
        useHttp2: Bool = true,
        useQuic: Bool = false,
        useKeepAlive: Bool = true,
        format: VideoFormat = .mp4
    ) {
        self.connections = connections
        self.splits = splits
        self.chunkSize = chunkSize
        self.retryWait = retryWait
        self.maxRetries = maxRetries
        self.useHttp2 = useHttp2
        self.useQuic = useQuic
        self.useKeepAlive = useKeepAlive
        self.format = format
    }
    
    // CodingKeys for JSON conversion
    enum CodingKeys: String, CodingKey {
        case connections
        case splits
        case chunkSize = "chunk_size"
        case retryWait = "retry_wait"
        case maxRetries = "max_retries"
        case useHttp2 = "use_http2"
        case useQuic = "use_quic"
        case useKeepAlive = "use_keep_alive"
        case format
    }
    
    /// デフォルト設定
    public static let `default` = DownloadOptions()
    
    /// MP4直リンク用の最適設定
    public static let mp4 = DownloadOptions(
        connections: 16,
        splits: 16,
        chunkSize: 4,
        retryWait: 2,
        maxRetries: 5,
        useHttp2: true,
        useKeepAlive: true
    )
    
    /// HLS用の最適設定
    public static let hls = DownloadOptions(
        connections: 16,
        splits: 16,
        chunkSize: 1,
        retryWait: 1,
        maxRetries: 10,
        useHttp2: true,
        useKeepAlive: true
    )
    
    /// DASH用の最適設定
    public static let dash = DownloadOptions(
        connections: 8,
        splits: 8,
        chunkSize: 1,
        retryWait: 2,
        maxRetries: 5,
        useHttp2: true,
        useQuic: true,
        useKeepAlive: true
    )
    
    /// 低速接続用の設定
    public static let lowBandwidth = DownloadOptions(
        connections: 4,
        splits: 4,
        chunkSize: 1,
        retryWait: 5,
        maxRetries: 10,
        useHttp2: true,
        useKeepAlive: true
    )
    
    /// 高速接続用の設定
    public static let highBandwidth = DownloadOptions(
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

/// ダウンロード関連のエラー
public enum DownloadError: Error {
    /// ファイルが見つからない
    case fileNotFound
    /// コンテンツタイプが不明
    case unknownContentType
    /// プロセス実行失敗
    case processFailed
    /// I/Oエラー
    case ioError
    /// JSON解析エラー
    case jsonParsingFailed
    /// ビデオ情報取得失敗
    case videoInfoFetchFailed
    /// 不明なエラー
    case unknown
    
    /// エラーの説明
    public var description: String {
        switch self {
        case .fileNotFound:
            return "ファイルが見つかりません"
        case .unknownContentType:
            return "コンテンツタイプが不明です"
        case .processFailed:
            return "プロセスの実行に失敗しました"
        case .ioError:
            return "I/Oエラーが発生しました"
        case .jsonParsingFailed:
            return "JSONの解析に失敗しました"
        case .videoInfoFetchFailed:
            return "ビデオ情報の取得に失敗しました"
        case .unknown:
            return "不明なエラーが発生しました"
        }
    }
}

/// 動画情報
public struct VideoInfo: Codable {
    /// タイトル
    public var title: String?
    /// 利用可能なフォーマット
    public var formats: [FormatInfo]?
    /// 説明
    public var description: String?
    /// 長さ（秒）
    public var duration: Double?
    /// URL
    public var url: String?
    
    /// 初期化
    public init(title: String? = nil, formats: [FormatInfo]? = nil, description: String? = nil, duration: Double? = nil, url: String? = nil) {
        self.title = title
        self.formats = formats
        self.description = description
        self.duration = duration
        self.url = url
    }
}

/// フォーマット情報
public struct FormatInfo: Codable {
    /// フォーマットID
    public var formatId: String?
    /// URL
    public var url: String?
    /// マニフェストURL
    public var manifestUrl: String?
    /// 幅
    public var width: Int?
    /// 高さ
    public var height: Int?
    /// 拡張子
    public var ext: String?
    
    /// 初期化
    public init(formatId: String? = nil, url: String? = nil, manifestUrl: String? = nil, width: Int? = nil, height: Int? = nil, ext: String? = nil) {
        self.formatId = formatId
        self.url = url
        self.manifestUrl = manifestUrl
        self.width = width
        self.height = height
        self.ext = ext
    }
    
    enum CodingKeys: String, CodingKey {
        case formatId = "format_id"
        case url
        case manifestUrl = "manifest_url"
        case width
        case height
        case ext
    }
}

/// 進捗情報
public struct ProgressInfo: Codable {
    /// 進捗（0.0〜1.0）
    public var progress: Double
    /// ダウンロード速度
    public var speed: String
    /// 残り時間
    public var eta: String
    
    /// 初期化
    public init(progress: Double, speed: String, eta: String) {
        self.progress = progress
        self.speed = speed
        self.eta = eta
    }
}

/// ダウンロードステータス
public enum DownloadStatus {
    /// 待機中
    case pending
    /// ダウンロード中
    case downloading
    /// 一時停止
    case paused
    /// 完了
    case completed
    /// 失敗
    case failed
    /// キャンセル
    case cancelled
    
    /// ステータスの説明
    public var description: String {
        switch self {
        case .pending:
            return "待機中"
        case .downloading:
            return "ダウンロード中"
        case .paused:
            return "一時停止"
        case .completed:
            return "完了"
        case .failed:
            return "失敗"
        case .cancelled:
            return "キャンセル"
        }
    }
}

/// ダウンロードアイテム
public struct DownloadItem: Identifiable {
    /// 一意のID
    public let id: UUID
    /// ダウンロードURL
    public let url: String
    /// タイトル
    public var title: String
    /// ステータス
    public var status: DownloadStatus
    /// 進捗（0.0〜1.0）
    public var progress: Double
    /// ダウンロード速度
    public var speed: String
    /// 残り時間
    public var remainingTime: String
    /// 出力パス
    public let outputPath: String
    /// 出力フォーマット
    public let format: VideoFormat
    
    /// 初期化
    public init(
        id: UUID = UUID(),
        url: String,
        title: String,
        status: DownloadStatus = .pending,
        progress: Double = 0.0,
        speed: String = "",
        remainingTime: String = "",
        outputPath: String,
        format: VideoFormat = .mp4
    ) {
        self.id = id
        self.url = url
        self.title = title
        self.status = status
        self.progress = progress
        self.speed = speed
        self.remainingTime = remainingTime
        self.outputPath = outputPath
        self.format = format
    }
}

/// システムの状態
public enum SystemStatus {
    /// 準備完了
    case ready
    /// 依存関係が足りない
    case missingDependencies(ytdlp: Bool, aria2c: Bool, ffmpeg: Bool)
    /// 不明な状態
    case unknown
    
    /// 準備完了かどうか
    public var isReady: Bool {
        if case .ready = self {
            return true
        }
        return false
    }
    
    /// 状態の説明
    public var description: String {
        switch self {
        case .ready:
            return "システム準備完了"
        case .missingDependencies(let ytdlp, let aria2c, let ffmpeg):
            var missing: [String] = []
            if !ytdlp { missing.append("yt-dlp") }
            if !aria2c { missing.append("aria2c") }
            if !ffmpeg { missing.append("ffmpeg") }
            return "依存関係が不足しています: \(missing.joined(separator: ", "))"
        case .unknown:
            return "システム状態不明"
        }
    }
}
