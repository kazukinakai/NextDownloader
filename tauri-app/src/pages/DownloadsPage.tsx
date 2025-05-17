import React, { useState, useEffect } from 'react';
import {
  Box,
  Button,
  Card,
  CardBody,
  Container,
  Flex,
  Heading,
  Progress,
  Table,
  Tbody,
  Td,
  Text,
  Th,
  Thead,
  Tr,
  IconButton,
  Badge,
  useToast,
  HStack,
} from '@chakra-ui/react';
import { FiTrash2, FiPause, FiPlay } from 'react-icons/fi';

// APIラッパーのインポート
import { getDownloadProgress, cancelDownload, setupDownloadListeners } from '../api/downloader';

// ダウンロードアイテムの型定義
interface DownloadItem {
  id: string;
  url: string;
  destination: string;
  progress: number;
  status: 'downloading' | 'paused' | 'completed' | 'error';
  createdAt: Date;
}

/**
 * ダウンロード管理ページコンポーネント
 * 進行中および完了したダウンロードを管理します
 */
const DownloadsPage: React.FC = () => {
  const toast = useToast();
  
  // 状態管理
  const [downloads, setDownloads] = useState<DownloadItem[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  
  // ダウンロードリスナーの設定
  useEffect(() => {
    const setupListeners = async () => {
      // ダウンロードの進捗と完了のリスナーを設定
      const unlisten = await setupDownloadListeners(
        // 進捗更新時のコールバック
        (id, progress) => {
          setDownloads((prevDownloads) => {
            return prevDownloads.map((download) => {
              if (download.id === id) {
                return {
                  ...download,
                  progress,
                  status: progress >= 1.0 ? 'completed' : 'downloading',
                };
              }
              return download;
            });
          });
        },
        // 完了時のコールバック
        (id) => {
          setDownloads((prevDownloads) => {
            return prevDownloads.map((download) => {
              if (download.id === id) {
                return {
                  ...download,
                  progress: 1.0,
                  status: 'completed',
                };
              }
              return download;
            });
          });
          
          toast({
            title: 'ダウンロード完了',
            description: `ダウンロードID: ${id} が完了しました`,
            status: 'success',
            duration: 3000,
            isClosable: true,
          });
        }
      );
      
      // コンポーネントのアンマウント時にリスナーを解除
      return () => {
        unlisten();
      };
    };
    
    setupListeners();
    
    // ダミーデータ（開発用）
    // 実際のアプリケーションでは、保存されたダウンロード履歴を読み込む
    setDownloads([
      {
        id: 'download-1',
        url: 'https://example.com/video1.mp4',
        destination: '/Users/username/Downloads/video1.mp4',
        progress: 0.75,
        status: 'downloading',
        createdAt: new Date(),
      },
      {
        id: 'download-2',
        url: 'https://example.com/video2.mp4',
        destination: '/Users/username/Downloads/video2.mp4',
        progress: 1.0,
        status: 'completed',
        createdAt: new Date(Date.now() - 3600000), // 1時間前
      },
    ]);
  }, [toast]);
  
  // ダウンロードの進捗を更新する関数
  const updateDownloadProgress = async (downloadId: string) => {
    try {
      const progress = await getDownloadProgress(downloadId);
      
      setDownloads((prevDownloads) => {
        return prevDownloads.map((download) => {
          if (download.id === downloadId) {
            return {
              ...download,
              progress,
              status: progress >= 1.0 ? 'completed' : 'downloading',
            };
          }
          return download;
        });
      });
    } catch (error) {
      console.error(`ダウンロード進捗の取得に失敗しました (ID: ${downloadId}):`, error);
    }
  };
  
  // ダウンロードをキャンセルする関数
  const handleCancelDownload = async (downloadId: string) => {
    setIsLoading(true);
    
    try {
      await cancelDownload(downloadId);
      
      // ダウンロードリストから削除
      setDownloads((prevDownloads) => {
        return prevDownloads.filter((download) => download.id !== downloadId);
      });
      
      toast({
        title: 'ダウンロードキャンセル',
        description: 'ダウンロードがキャンセルされました',
        status: 'info',
        duration: 3000,
        isClosable: true,
      });
    } catch (error) {
      console.error(`ダウンロードのキャンセルに失敗しました (ID: ${downloadId}):`, error);
      toast({
        title: 'エラー',
        description: 'ダウンロードのキャンセルに失敗しました',
        status: 'error',
        duration: 3000,
        isClosable: true,
      });
    } finally {
      setIsLoading(false);
    }
  };
  
  // ダウンロードの一時停止/再開（実装予定）
  const handleTogglePause = (downloadId: string) => {
    // この機能は将来的に実装予定
    toast({
      title: '機能準備中',
      description: 'この機能は現在開発中です',
      status: 'info',
      duration: 3000,
      isClosable: true,
    });
  };
  
  // ステータスに応じたバッジの表示
  const renderStatusBadge = (status: string) => {
    switch (status) {
      case 'downloading':
        return <Badge colorScheme="blue">ダウンロード中</Badge>;
      case 'paused':
        return <Badge colorScheme="yellow">一時停止</Badge>;
      case 'completed':
        return <Badge colorScheme="green">完了</Badge>;
      case 'error':
        return <Badge colorScheme="red">エラー</Badge>;
      default:
        return <Badge>不明</Badge>;
    }
  };
  
  // 日時のフォーマット
  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('ja-JP', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    }).format(date);
  };
  
  // 進行中のダウンロードと完了したダウンロードに分類
  const activeDownloads = downloads.filter((download) => download.status !== 'completed');
  const completedDownloads = downloads.filter((download) => download.status === 'completed');
  
  return (
    <Container maxW="container.lg" py={4}>
      <Box mb={6}>
        <Heading size="md" mb={4}>ダウンロード管理</Heading>
        
        {/* 進行中のダウンロード */}
        <Card variant="outline" mb={6}>
          <CardBody>
            <Heading size="sm" mb={3}>進行中のダウンロード ({activeDownloads.length})</Heading>
            
            {activeDownloads.length === 0 ? (
              <Text color="gray.500">現在進行中のダウンロードはありません</Text>
            ) : (
              <Table size="sm" variant="simple">
                <Thead>
                  <Tr>
                    <Th>URL</Th>
                    <Th>進捗</Th>
                    <Th>ステータス</Th>
                    <Th>操作</Th>
                  </Tr>
                </Thead>
                <Tbody>
                  {activeDownloads.map((download) => (
                    <Tr key={download.id}>
                      <Td maxW="300px" isTruncated title={download.url}>
                        {download.url}
                      </Td>
                      <Td>
                        <Flex align="center">
                          <Progress
                            value={download.progress * 100}
                            size="sm"
                            colorScheme="blue"
                            flex="1"
                            mr={2}
                          />
                          <Text fontSize="xs" width="40px" textAlign="right">
                            {Math.round(download.progress * 100)}%
                          </Text>
                        </Flex>
                      </Td>
                      <Td>{renderStatusBadge(download.status)}</Td>
                      <Td>
                        <HStack spacing={1}>
                          <IconButton
                            aria-label="一時停止/再開"
                            icon={download.status === 'paused' ? <FiPlay /> : <FiPause />}
                            size="sm"
                            variant="ghost"
                            onClick={() => handleTogglePause(download.id)}
                          />
                          <IconButton
                            aria-label="キャンセル"
                            icon={<FiTrash2 />}
                            size="sm"
                            variant="ghost"
                            colorScheme="red"
                            onClick={() => handleCancelDownload(download.id)}
                            isLoading={isLoading}
                          />
                        </HStack>
                      </Td>
                    </Tr>
                  ))}
                </Tbody>
              </Table>
            )}
          </CardBody>
        </Card>
        
        {/* 完了したダウンロード */}
        <Card variant="outline">
          <CardBody>
            <Heading size="sm" mb={3}>完了したダウンロード ({completedDownloads.length})</Heading>
            
            {completedDownloads.length === 0 ? (
              <Text color="gray.500">完了したダウンロードはありません</Text>
            ) : (
              <Table size="sm" variant="simple">
                <Thead>
                  <Tr>
                    <Th>URL</Th>
                    <Th>保存先</Th>
                    <Th>完了日時</Th>
                    <Th>操作</Th>
                  </Tr>
                </Thead>
                <Tbody>
                  {completedDownloads.map((download) => (
                    <Tr key={download.id}>
                      <Td maxW="250px" isTruncated title={download.url}>
                        {download.url}
                      </Td>
                      <Td maxW="250px" isTruncated title={download.destination}>
                        {download.destination}
                      </Td>
                      <Td>{formatDate(download.createdAt)}</Td>
                      <Td>
                        <IconButton
                          aria-label="削除"
                          icon={<FiTrash2 />}
                          size="sm"
                          variant="ghost"
                          colorScheme="red"
                          onClick={() => handleCancelDownload(download.id)}
                        />
                      </Td>
                    </Tr>
                  ))}
                </Tbody>
              </Table>
            )}
          </CardBody>
        </Card>
      </Box>
    </Container>
  );
};

export default DownloadsPage;