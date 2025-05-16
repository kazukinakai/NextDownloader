import Foundation

/// NextDownloaderのRustコアとのブリッジング
public class NextDownloaderCore {
    /// シングルトンインスタンス
    public static let shared = NextDownloaderCore()
    
    /// 初期化
    private init() {
        // ライブラリの初期化処理があれば実行
    }
    
    /// システムの依存関係をチェック
    /// - Returns: (yt-dlp利用可能, aria2c利用可能, ffmpeg利用可能)
    public func checkDependencies() -> (ytdlp: Bool, aria2c: Bool, ffmpeg: Bool) {
        var result: [UInt8] = [0, 0, 0]
        
        let status = result.withUnsafeMutableBufferPointer { bufferPtr in
            check_dependencies(bufferPtr.baseAddress)
        }
        
        if status == 0 {
            return (
                ytdlp: result[0] == 1,
                aria2c: result[1] == 1,
                ffmpeg: result[2] == 1
            )
        } else {
            // エラーが発生した場合はすべてfalseを返す
            return (ytdlp: false, aria2c: false, ffmpeg: false)
        }
    }
    
    /// URLからコンテンツタイプを検出
    /// - Parameter url: 検出するURL
    /// - Returns: コンテンツタイプ
    public func detectContentType(url: String) -> ContentType {
        var result: Int32 = 4 // デフォルトはunknown
        
        let status = url.withCString { urlPtr in
            detect_content_type(urlPtr, &result)
        }
        
        if status == 0 {
            switch result {
            case 0:
                return .mp4
            case 1:
                return .hls
            case 2:
                return .dash
            case 3:
                return .youtube
            default:
                return .unknown
            }
        } else {
            return .unknown
        }
    }
    
    /// URLからビデオ情報を取得
    /// - Parameter url: 取得するURL
    /// - Returns: ビデオ情報（取得できない場合はnil）
    public func getVideoInfo(url: String) async throws -> VideoInfo? {
        return try await withCheckedThrowingContinuation { continuation in
            var jsonResult: UnsafeMutablePointer<CChar>? = nil
            
            let status = url.withCString { urlPtr in
                get_video_info(urlPtr, &jsonResult)
            }
            
            if status == 0 && jsonResult != nil {
                defer {
                    // メモリリーク防止のためCの文字列を解放
                    free_string(jsonResult)
                }
                
                let jsonString = String(cString: jsonResult!)
                
                do {
                    let decoder = JSONDecoder()
                    let videoInfo = try decoder.decode(VideoInfo.self, from: jsonString.data(using: .utf8)!)
                    continuation.resume(returning: videoInfo)
                } catch {
                    continuation.resume(throwing: DownloadError.jsonParsingFailed)
                }
            } else {
                continuation.resume(throwing: DownloadError.videoInfoFetchFailed)
            }
        }
    }
    
    /// URLからビデオをダウンロード
    /// - Parameters:
    ///   - url: ダウンロードするURL
    ///   - outputPath: 出力先ディレクトリ
    ///   - filename: ファイル名
    ///   - options: ダウンロードオプション（nilの場合はデフォルト値を使用）
    ///   - progressCallback: 進捗コールバック
    /// - Returns: ダウンロードしたファイルのパス
    public func downloadVideo(
        url: String,
        outputPath: String,
        filename: String,
        options: DownloadOptions? = nil,
        progressCallback: ((Double, String, String) -> Void)? = nil
    ) async throws -> String {
        return try await withCheckedThrowingContinuation { continuation in
            var resultPath: UnsafeMutablePointer<CChar>? = nil
            
            // オプションをJSON文字列に変換
            let optionsJson: String?
            if let options = options {
                let encoder = JSONEncoder()
                if let data = try? encoder.encode(options),
                   let json = String(data: data, encoding: .utf8) {
                    optionsJson = json
                } else {
                    optionsJson = nil
                }
            } else {
                optionsJson = nil
            }
            
            let status = url.withCString { urlPtr in
                outputPath.withCString { outputPathPtr in
                    filename.withCString { filenamePtr in
                        if let optionsJson = optionsJson {
                            optionsJson.withCString { optionsJsonPtr in
                                download_video(urlPtr, outputPathPtr, filenamePtr, optionsJsonPtr, &resultPath)
                            }
                        } else {
                            download_video(urlPtr, outputPathPtr, filenamePtr, nil, &resultPath)
                        }
                    }
                }
            }
            
            if status == 0 && resultPath != nil {
                defer {
                    // メモリリーク防止のためCの文字列を解放
                    free_string(resultPath)
                }
                
                let path = String(cString: resultPath!)
                continuation.resume(returning: path)
            } else {
                let error: DownloadError
                switch status {
                case 1:
                    error = .fileNotFound
                case 2:
                    error = .processFailed
                case 3:
                    error = .ioError
                case 4:
                    error = .jsonParsingFailed
                default:
                    error = .unknown
                }
                continuation.resume(throwing: error)
            }
        }
    }
}

// MARK: - FFI関数の宣言

/// システムの依存関係をチェック
@_cdecl("check_dependencies")
private func check_dependencies(_ result: UnsafeMutablePointer<UInt8>?) -> Int32 {
    let libPath = Bundle.module.path(forResource: "libnextdownloader_core", ofType: "dylib")
    
    guard let lib = dlopen(libPath, RTLD_LAZY) else {
        return -1
    }
    defer { dlclose(lib) }
    
    guard let fn = dlsym(lib, "check_dependencies") else {
        return -1
    }
    
    typealias CheckDependenciesFn = @convention(c) (UnsafeMutablePointer<UInt8>?) -> Int32
    let function = unsafeBitCast(fn, to: CheckDependenciesFn.self)
    
    return function(result)
}

/// URLからコンテンツタイプを検出
@_cdecl("detect_content_type")
private func detect_content_type(_ url: UnsafePointer<CChar>?, _ result: UnsafeMutablePointer<Int32>?) -> Int32 {
    let libPath = Bundle.module.path(forResource: "libnextdownloader_core", ofType: "dylib")
    
    guard let lib = dlopen(libPath, RTLD_LAZY) else {
        return -1
    }
    defer { dlclose(lib) }
    
    guard let fn = dlsym(lib, "detect_content_type") else {
        return -1
    }
    
    typealias DetectContentTypeFn = @convention(c) (UnsafePointer<CChar>?, UnsafeMutablePointer<Int32>?) -> Int32
    let function = unsafeBitCast(fn, to: DetectContentTypeFn.self)
    
    return function(url, result)
}

/// URLからビデオ情報を取得
@_cdecl("get_video_info")
private func get_video_info(_ url: UnsafePointer<CChar>?, _ jsonResult: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?) -> Int32 {
    let libPath = Bundle.module.path(forResource: "libnextdownloader_core", ofType: "dylib")
    
    guard let lib = dlopen(libPath, RTLD_LAZY) else {
        return -1
    }
    defer { dlclose(lib) }
    
    guard let fn = dlsym(lib, "get_video_info") else {
        return -1
    }
    
    typealias GetVideoInfoFn = @convention(c) (UnsafePointer<CChar>?, UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?) -> Int32
    let function = unsafeBitCast(fn, to: GetVideoInfoFn.self)
    
    return function(url, jsonResult)
}

/// URLからビデオをダウンロード
@_cdecl("download_video")
private func download_video(
    _ url: UnsafePointer<CChar>?,
    _ outputPath: UnsafePointer<CChar>?,
    _ filename: UnsafePointer<CChar>?,
    _ optionsJson: UnsafePointer<CChar>?,
    _ resultPath: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
) -> Int32 {
    let libPath = Bundle.module.path(forResource: "libnextdownloader_core", ofType: "dylib")
    
    guard let lib = dlopen(libPath, RTLD_LAZY) else {
        return -1
    }
    defer { dlclose(lib) }
    
    guard let fn = dlsym(lib, "download_video") else {
        return -1
    }
    
    typealias DownloadVideoFn = @convention(c) (
        UnsafePointer<CChar>?,
        UnsafePointer<CChar>?,
        UnsafePointer<CChar>?,
        UnsafePointer<CChar>?,
        UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?
    ) -> Int32
    let function = unsafeBitCast(fn, to: DownloadVideoFn.self)
    
    return function(url, outputPath, filename, optionsJson, resultPath)
}

/// C文字列を解放
@_cdecl("free_string")
private func free_string(_ ptr: UnsafeMutablePointer<CChar>?) {
    let libPath = Bundle.module.path(forResource: "libnextdownloader_core", ofType: "dylib")
    
    guard let lib = dlopen(libPath, RTLD_LAZY) else {
        return
    }
    defer { dlclose(lib) }
    
    guard let fn = dlsym(lib, "free_string") else {
        return
    }
    
    typealias FreeStringFn = @convention(c) (UnsafeMutablePointer<CChar>?) -> Void
    let function = unsafeBitCast(fn, to: FreeStringFn.self)
    
    function(ptr)
}

// MARK: - dlopenのためのインポート
import Darwin.C
