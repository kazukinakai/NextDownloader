import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_app/features/download/domain/entities/download_info.dart';

void main() {
  group('DownloadInfo Entity Tests', () {
    test('should create DownloadInfo instance with given values', () {
      final downloadInfo = DownloadInfo(
        downloadId: 'test-id-123',
        url: 'https://example.com/test.mp4',
        destination: '/path/to/destination.mp4',
        status: 'downloading',
        progress: 0.5,
        downloadedSize: 1024,
        totalSize: 2048,
        createdAt: '2025-05-22T10:00:00Z',
        updatedAt: '2025-05-22T10:01:00Z',
      );

      expect(downloadInfo.downloadId, 'test-id-123');
      expect(downloadInfo.url, 'https://example.com/test.mp4');
      expect(downloadInfo.destination, '/path/to/destination.mp4');
      expect(downloadInfo.status, 'downloading');
      expect(downloadInfo.progress, 0.5);
      expect(downloadInfo.downloadedSize, 1024);
      expect(downloadInfo.totalSize, 2048);
      expect(downloadInfo.createdAt, '2025-05-22T10:00:00Z');
      expect(downloadInfo.updatedAt, '2025-05-22T10:01:00Z');
      expect(downloadInfo.errorMessage, null);
    });

    test('should create DownloadInfo from JSON', () {
      final json = {
        'downloadId': 'json-test-id',
        'url': 'https://example.com/test2.mp4',
        'destination': '/path/to/dest2.mp4',
        'status': 'completed',
        'progress': 1.0,
        'downloadedSize': 2048,
        'totalSize': 2048,
        'createdAt': '2025-05-22T09:00:00Z',
        'updatedAt': '2025-05-22T09:05:00Z',
        'errorMessage': null,
      };

      final downloadInfo = DownloadInfo.fromJson(json);

      expect(downloadInfo.downloadId, 'json-test-id');
      expect(downloadInfo.url, 'https://example.com/test2.mp4');
      expect(downloadInfo.destination, '/path/to/dest2.mp4');
      expect(downloadInfo.status, 'completed');
      expect(downloadInfo.progress, 1.0);
      expect(downloadInfo.downloadedSize, 2048);
      expect(downloadInfo.totalSize, 2048);
      expect(downloadInfo.createdAt, '2025-05-22T09:00:00Z');
      expect(downloadInfo.updatedAt, '2025-05-22T09:05:00Z');
      expect(downloadInfo.errorMessage, null);
    });

    test('should convert DownloadInfo to JSON', () {
      final downloadInfo = DownloadInfo(
        downloadId: 'convert-test-id',
        url: 'https://example.com/test3.mp4',
        destination: '/path/to/dest3.mp4',
        status: 'error',
        progress: 0.7,
        downloadedSize: 1433,
        totalSize: 2048,
        createdAt: '2025-05-22T08:00:00Z',
        updatedAt: '2025-05-22T08:05:00Z',
        errorMessage: 'Connection error',
      );

      final json = downloadInfo.toJson();

      expect(json['downloadId'], 'convert-test-id');
      expect(json['url'], 'https://example.com/test3.mp4');
      expect(json['destination'], '/path/to/dest3.mp4');
      expect(json['status'], 'error');
      expect(json['progress'], 0.7);
      expect(json['downloadedSize'], 1433);
      expect(json['totalSize'], 2048);
      expect(json['createdAt'], '2025-05-22T08:00:00Z');
      expect(json['updatedAt'], '2025-05-22T08:05:00Z');
      expect(json['errorMessage'], 'Connection error');
    });
  });

  group('DownloadProgress Entity Tests', () {
    test('should create DownloadProgress instance with given values', () {
      final progress = DownloadProgress(
        downloadId: 'progress-id',
        progress: 0.3,
        downloadedSize: 614,
        totalSize: 2048,
        status: 'downloading',
        errorMessage: null,
      );

      expect(progress.downloadId, 'progress-id');
      expect(progress.progress, 0.3);
      expect(progress.downloadedSize, 614);
      expect(progress.totalSize, 2048);
      expect(progress.status, 'downloading');
      expect(progress.errorMessage, null);
    });

    test('should create DownloadProgress from JSON', () {
      final json = {
        'downloadId': 'progress-json-id',
        'progress': 0.6,
        'downloadedSize': 1228,
        'totalSize': 2048,
        'status': 'downloading',
        'errorMessage': null,
      };

      final progress = DownloadProgress.fromJson(json);

      expect(progress.downloadId, 'progress-json-id');
      expect(progress.progress, 0.6);
      expect(progress.downloadedSize, 1228);
      expect(progress.totalSize, 2048);
      expect(progress.status, 'downloading');
      expect(progress.errorMessage, null);
    });
  });

  group('YoutubeVideoInfo Entity Tests', () {
    test('should create YoutubeVideoInfo instance with given values', () {
      final formats = [
        YoutubeVideoFormat(
          quality: '720p',
          format: 'mp4',
          size: 10240,
        ),
        YoutubeVideoFormat(
          quality: '1080p',
          format: 'mp4',
          size: 20480,
        ),
      ];

      final videoInfo = YoutubeVideoInfo(
        title: 'Test Video',
        author: 'Test Author',
        duration: 120,
        formats: formats,
      );

      expect(videoInfo.title, 'Test Video');
      expect(videoInfo.author, 'Test Author');
      expect(videoInfo.duration, 120);
      expect(videoInfo.formats.length, 2);
      expect(videoInfo.formats[0].quality, '720p');
      expect(videoInfo.formats[1].quality, '1080p');
    });
  });
} 