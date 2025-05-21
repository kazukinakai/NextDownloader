import 'package:flutter/material.dart';
import 'package:flutter_app/features/download/domain/entities/download_info.dart';
import 'package:flutter_app/features/download/domain/repositories/download_repository.dart';
import 'package:flutter_app/features/download/presentation/providers/download_providers.dart';
import 'package:flutter_app/features/download/presentation/screens/download_screen.dart';
import 'package:flutter_app/features/download/presentation/widgets/download_form_widget.dart';
import 'package:flutter_app/features/download/presentation/widgets/download_item_widget.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';

@GenerateMocks([DownloadRepository])
import 'download_screen_test.mocks.dart';

void main() {
  group('DownloadScreen Widget Tests', () {
    late MockDownloadRepository mockRepository;

    setUp(() {
      mockRepository = MockDownloadRepository();
    });

    testWidgets('should display empty state when no downloads', (WidgetTester tester) async {
      // リポジトリが空のリストを返すように設定
      when(mockRepository.getDownloadList()).thenAnswer((_) async => []);

      // テスト用のウィジェットをビルド
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            downloadRepositoryProvider.overrideWithValue(mockRepository),
          ],
          child: const MaterialApp(
            home: DownloadScreen(),
          ),
        ),
      );

      // ウィジェットの描画を待機
      await tester.pumpAndSettle();

      // 空の状態のメッセージが表示されているか確認
      expect(find.text('ダウンロードがありません'), findsOneWidget);
      expect(find.text('URLを入力してダウンロードを開始してください'), findsOneWidget);

      // 入力フォームが表示されているか確認
      expect(find.byType(DownloadFormWidget), findsOneWidget);

      // リストアイテムが表示されていないことを確認
      expect(find.byType(DownloadItemWidget), findsNothing);
    });

    testWidgets('should display downloads when list is not empty', (WidgetTester tester) async {
      // テスト用のダウンロード情報
      final testDownloads = [
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

      // リポジトリがテスト用のダウンロードリストを返すように設定
      when(mockRepository.getDownloadList()).thenAnswer((_) async => testDownloads);

      // テスト用のウィジェットをビルド
      await tester.pumpWidget(
        ProviderScope(
          overrides: [
            downloadRepositoryProvider.overrideWithValue(mockRepository),
          ],
          child: const MaterialApp(
            home: DownloadScreen(),
          ),
        ),
      );

      // ウィジェットの描画を待機
      await tester.pumpAndSettle();

      // 空の状態のメッセージが表示されていないことを確認
      expect(find.text('ダウンロードがありません'), findsNothing);

      // ダウンロード数のテキストが表示されているか確認
      expect(find.text('2 件'), findsOneWidget);

      // ダウンロードアイテムが2つ表示されているか確認
      expect(find.byType(DownloadItemWidget), findsNWidgets(2));

      // リフレッシュボタンをタップした時の挙動をテスト
      await tester.tap(find.byIcon(Icons.refresh));
      await tester.pumpAndSettle();

      // getDownloadListが再度呼ばれたことを確認
      verify(mockRepository.getDownloadList()).called(2); // 初期表示 + リフレッシュ
    });
  });
} 