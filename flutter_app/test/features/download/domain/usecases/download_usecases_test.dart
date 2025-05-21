import 'package:flutter_app/features/download/domain/entities/download_info.dart';
import 'package:flutter_app/features/download/domain/repositories/download_repository.dart';
import 'package:flutter_app/features/download/domain/usecases/download_usecases.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';

// モックの生成
@GenerateMocks([DownloadRepository])
import 'download_usecases_test.mocks.dart';

void main() {
  late MockDownloadRepository mockRepository;
  late StartDownloadUseCase startDownloadUseCase;
  late GetDownloadProgressUseCase getDownloadProgressUseCase;
  late CancelDownloadUseCase cancelDownloadUseCase;
  late GetDownloadListUseCase getDownloadListUseCase;

  setUp(() {
    mockRepository = MockDownloadRepository();
    startDownloadUseCase = StartDownloadUseCase(mockRepository);
    getDownloadProgressUseCase = GetDownloadProgressUseCase(mockRepository);
    cancelDownloadUseCase = CancelDownloadUseCase(mockRepository);
    getDownloadListUseCase = GetDownloadListUseCase(mockRepository);
  });

  group('StartDownloadUseCase', () {
    final testUrl = 'https://example.com/test.mp4';
    final testDestination = '/path/to/test.mp4';
    final testOptions = DownloadOptions();
    
    final testDownloadInfo = DownloadInfo(
      downloadId: 'test-download-id',
      url: testUrl,
      destination: testDestination,
      status: 'downloading',
      progress: 0.0,
      downloadedSize: 0,
      totalSize: 1024,
      createdAt: '2025-05-22T10:00:00Z',
      updatedAt: '2025-05-22T10:00:00Z',
    );

    test('should start download using repository', () async {
      // Arrange
      when(mockRepository.startDownload(testUrl, testDestination, testOptions))
          .thenAnswer((_) async => testDownloadInfo);

      // Act
      final result = await startDownloadUseCase(testUrl, testDestination, testOptions);

      // Assert
      expect(result, equals(testDownloadInfo));
      verify(mockRepository.startDownload(testUrl, testDestination, testOptions)).called(1);
    });
  });

  group('GetDownloadProgressUseCase', () {
    final testDownloadId = 'test-download-id';
    final testProgress = DownloadProgress(
      downloadId: testDownloadId,
      progress: 0.5,
      downloadedSize: 512,
      totalSize: 1024,
      status: 'downloading',
    );

    test('should get download progress using repository', () async {
      // Arrange
      when(mockRepository.getDownloadProgress(testDownloadId))
          .thenAnswer((_) async => testProgress);

      // Act
      final result = await getDownloadProgressUseCase(testDownloadId);

      // Assert
      expect(result, equals(testProgress));
      verify(mockRepository.getDownloadProgress(testDownloadId)).called(1);
    });
  });

  group('CancelDownloadUseCase', () {
    final testDownloadId = 'test-download-id';

    test('should cancel download using repository', () async {
      // Arrange
      when(mockRepository.cancelDownload(testDownloadId))
          .thenAnswer((_) async => true);

      // Act
      final result = await cancelDownloadUseCase(testDownloadId);

      // Assert
      expect(result, equals(true));
      verify(mockRepository.cancelDownload(testDownloadId)).called(1);
    });

    test('should return false if cancel fails', () async {
      // Arrange
      when(mockRepository.cancelDownload(testDownloadId))
          .thenAnswer((_) async => false);

      // Act
      final result = await cancelDownloadUseCase(testDownloadId);

      // Assert
      expect(result, equals(false));
      verify(mockRepository.cancelDownload(testDownloadId)).called(1);
    });
  });

  group('GetDownloadListUseCase', () {
    final testDownloadList = [
      DownloadInfo(
        downloadId: 'download-1',
        url: 'https://example.com/test1.mp4',
        destination: '/path/to/test1.mp4',
        status: 'completed',
        progress: 1.0,
        downloadedSize: 1024,
        totalSize: 1024,
        createdAt: '2025-05-21T10:00:00Z',
        updatedAt: '2025-05-21T10:05:00Z',
      ),
      DownloadInfo(
        downloadId: 'download-2',
        url: 'https://example.com/test2.mp4',
        destination: '/path/to/test2.mp4',
        status: 'downloading',
        progress: 0.5,
        downloadedSize: 512,
        totalSize: 1024,
        createdAt: '2025-05-22T10:00:00Z',
        updatedAt: '2025-05-22T10:02:00Z',
      ),
    ];

    test('should get download list using repository', () async {
      // Arrange
      when(mockRepository.getDownloadList())
          .thenAnswer((_) async => testDownloadList);

      // Act
      final result = await getDownloadListUseCase();

      // Assert
      expect(result, equals(testDownloadList));
      expect(result.length, 2);
      verify(mockRepository.getDownloadList()).called(1);
    });
  });

  group('Providers', () {
    test('StartDownloadUseCase provider should create an instance', () {
      final container = ProviderContainer(
        overrides: [
          downloadRepositoryProvider.overrideWithValue(MockDownloadRepository()),
        ],
      );

      final useCase = container.read(startDownloadUseCaseProvider);
      expect(useCase, isA<StartDownloadUseCase>());
    });

    test('GetDownloadProgressUseCase provider should create an instance', () {
      final container = ProviderContainer(
        overrides: [
          downloadRepositoryProvider.overrideWithValue(MockDownloadRepository()),
        ],
      );

      final useCase = container.read(getDownloadProgressUseCaseProvider);
      expect(useCase, isA<GetDownloadProgressUseCase>());
    });
  });
} 