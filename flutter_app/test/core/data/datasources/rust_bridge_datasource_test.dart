import 'dart:convert';
import 'package:flutter_app/core/data/datasources/rust_bridge_datasource.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';

// モックFFI APIを生成
@GenerateMocks([FlutterRustBridgeApi])
import 'rust_bridge_datasource_test.mocks.dart';

void main() {
  group('RustBridgeDataSource Tests', () {
    late MockFlutterRustBridgeApi mockApi;
    late RustBridgeDataSource dataSource;

    setUp(() {
      mockApi = MockFlutterRustBridgeApi();
      dataSource = RustBridgeDataSource();
      // 内部APIフィールドをモックで上書き
      dataSource.setApiForTesting(mockApi);
    });

    test('initialize should call initializeDownloader on API', () async {
      // Arrange
      when(mockApi.initializeDownloader()).thenAnswer((_) async => true);

      // Act
      final result = await dataSource.initialize();

      // Assert
      expect(result, true);
      verify(mockApi.initializeDownloader()).called(1);
    });

    test('startDownload should call API and parse JSON response', () async {
      // Arrange
      const url = 'https://example.com/test.mp4';
      const destination = '/path/to/test.mp4';
      final options = {'timeout': 30, 'retryCount': 3};
      
      final expectedResponse = {
        'downloadId': 'test-id',
        'url': url,
        'destination': destination,
        'status': 'downloading',
        'progress': 0.0,
        'downloadedSize': 0,
        'totalSize': 1024,
        'createdAt': '2025-05-22T10:00:00Z',
        'updatedAt': '2025-05-22T10:00:00Z',
      };
      
      when(mockApi.startDownload(url, destination, any))
          .thenAnswer((_) async => jsonEncode(expectedResponse));

      // Act
      final result = await dataSource.startDownload(url, destination, options);

      // Assert
      expect(result, equals(expectedResponse));
      verify(mockApi.startDownload(url, destination, any)).called(1);
    });

    test('getDownloadProgress should call API and parse JSON response', () async {
      // Arrange
      const downloadId = 'test-id';
      
      final expectedResponse = {
        'downloadId': downloadId,
        'progress': 0.5,
        'downloadedSize': 512,
        'totalSize': 1024,
        'status': 'downloading',
      };
      
      when(mockApi.getDownloadProgress(downloadId))
          .thenAnswer((_) async => jsonEncode(expectedResponse));

      // Act
      final result = await dataSource.getDownloadProgress(downloadId);

      // Assert
      expect(result, equals(expectedResponse));
      verify(mockApi.getDownloadProgress(downloadId)).called(1);
    });

    test('cancelDownload should call API and return result', () async {
      // Arrange
      const downloadId = 'test-id';
      
      when(mockApi.cancelDownload(downloadId))
          .thenAnswer((_) async => true);

      // Act
      final result = await dataSource.cancelDownload(downloadId);

      // Assert
      expect(result, true);
      verify(mockApi.cancelDownload(downloadId)).called(1);
    });

    test('getDownloadList should call API and parse JSON response', () async {
      // Arrange
      final expectedResponse = [
        {
          'downloadId': 'test-id-1',
          'url': 'https://example.com/test1.mp4',
          'destination': '/path/to/test1.mp4',
          'status': 'completed',
          'progress': 1.0,
          'downloadedSize': 1024,
          'totalSize': 1024,
          'createdAt': '2025-05-21T10:00:00Z',
          'updatedAt': '2025-05-21T10:05:00Z',
        },
        {
          'downloadId': 'test-id-2',
          'url': 'https://example.com/test2.mp4',
          'destination': '/path/to/test2.mp4',
          'status': 'downloading',
          'progress': 0.5,
          'downloadedSize': 512,
          'totalSize': 1024,
          'createdAt': '2025-05-22T10:00:00Z',
          'updatedAt': '2025-05-22T10:02:00Z',
        }
      ];
      
      when(mockApi.getDownloadList())
          .thenAnswer((_) async => jsonEncode(expectedResponse));

      // Act
      final result = await dataSource.getDownloadList();

      // Assert
      expect(result, equals(expectedResponse));
      expect(result.length, 2);
      verify(mockApi.getDownloadList()).called(1);
    });
  });
}

// RustBridgeDataSourceのテスト用拡張
extension RustBridgeDataSourceTestExt on RustBridgeDataSource {
  void setApiForTesting(FlutterRustBridgeApi api) {
    // テスト用にAPIを設定
    _api = api;
    _isInitialized = true;
  }
}

// モックAPI用の基本クラス
abstract class FlutterRustBridgeApi {
  Future<bool> initializeDownloader();
  Future<String> startDownload(String url, String destination, String optionsJson);
  Future<String> startAuthenticatedDownload(String url, String destination, String cookiesPath, String optionsJson);
  Future<String> getDownloadProgress(String downloadId);
  Future<bool> pauseDownload(String downloadId);
  Future<bool> resumeDownload(String downloadId);
  Future<bool> cancelDownload(String downloadId);
  Future<String> getDownloadList();
  Future<String> detectContentType(String url);
  Future<String> getYoutubeVideoInfo(String url);
  Future<String> getSettings();
} 