/**
 * 依存関係のステータス
 */
export interface DependencyStatus {
  /** yt-dlpがインストールされているか */
  ytdlp: boolean;
  /** aria2cがインストールされているか */
  aria2c: boolean;
  /** ffmpegがインストールされているか */
  ffmpeg: boolean;
}