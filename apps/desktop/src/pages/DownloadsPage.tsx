import React, { useEffect, useState } from 'react';
import {
  Box,
  Container,
  Heading,
  Text,
  SimpleGrid,
  VStack,
  HStack,
  Button,
  useToast,
  Spinner,
  Center,
  Tabs,
  TabList,
  TabPanels,
  Tab,
  TabPanel,
  Badge,
  useColorModeValue,
  Icon,
} from '@chakra-ui/react';
import { FiRefreshCw, FiInfo } from 'react-icons/fi';
import DownloadCard from '../components/DownloadCard';
import { getDownloads, getDownloadProgress, cancelDownload } from '../api/download';
import { DownloadInfo } from '../types/download';

/**
 * ダウンロードページコンポーネント
 */
const DownloadsPage: React.FC = () => {
  const toast = useToast();
  const [downloads, setDownloads] = useState<DownloadInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // ダウンロード一覧を取得
  const fetchDownloads = async () => {
    try {
      const downloadsList = await getDownloads();
      setDownloads(downloadsList);
    } catch (error) {
      console.error('ダウンロード一覧の取得に失敗しました:', error);
      toast({
        title: 'エラー',
        description: 'ダウンロード一覧の取得に失敗しました',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setIsLoading(false);
      setIsRefreshing(false);
    }
  };

  // ダウンロード進捗の更新
  const updateDownloadProgress = async () => {
    const activeDownloads = downloads.filter(
      (download) => download.status.toLowerCase() === 'downloading' || download.status.toLowerCase() === 'initializing'
    );

    if (activeDownloads.length === 0) return;

    try {
      const updatedDownloads = await Promise.all(
        activeDownloads.map(async (download) => {
          try {
            return await getDownloadProgress(download.id);
          } catch (error) {
            console.error(`ダウンロード進捗の取得に失敗しました (ID: ${download.id}):`, error);
            return download;
          }
        })
      );

      // ダウンロード一覧を更新
      setDownloads((prevDownloads) => {
        const newDownloads = [...prevDownloads];
        updatedDownloads.forEach((updatedDownload) => {
          const index = newDownloads.findIndex((d) => d.id === updatedDownload.id);
          if (index !== -1) {
            newDownloads[index] = updatedDownload;
          }
        });
        return newDownloads;
      });
    } catch (error) {
      console.error('ダウンロード進捗の更新に失敗しました:', error);
    }
  };

  // ダウンロードをキャンセル
  const handleCancelDownload = async (id: string) => {
    try {
      await cancelDownload(id);
      
      // ダウンロード一覧を更新
      setDownloads((prevDownloads) => 
        prevDownloads.filter((download) => download.id !== id)
      );
      
      toast({
        title: 'キャンセル完了',
        description: 'ダウンロードをキャンセルしました',
        status: 'info',
        duration: 3000,
        isClosable: true,
      });
    } catch (error) {
      console.error('ダウンロードのキャンセルに失敗しました:', error);
      toast({
        title: 'エラー',
        description: 'ダウンロードのキャンセルに失敗しました',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    }
  };

  // 手動更新
  const handleRefresh = () => {
    setIsRefreshing(true);
    fetchDownloads();
  };

  // 初回読み込み
  useEffect(() => {
    fetchDownloads();
  }, []);

  // アクティブなダウンロードの進捗を定期的に更新
  useEffect(() => {
    const hasActiveDownloads = downloads.some(
      (download) => download.status.toLowerCase() === 'downloading' || download.status.toLowerCase() === 'initializing'
    );

    if (!hasActiveDownloads) return;

    const intervalId = setInterval(updateDownloadProgress, 1000);
    return () => clearInterval(intervalId);
  }, [downloads]);

  // ダウンロードをステータスでフィルタリング
  const activeDownloads = downloads.filter(
    (download) => download.status.toLowerCase() === 'downloading' || download.status.toLowerCase() === 'initializing'
  );
  
  const completedDownloads = downloads.filter(
    (download) => download.status.toLowerCase() === 'completed'
  );
  
  const otherDownloads = downloads.filter(
    (download) => 
      download.status.toLowerCase() !== 'downloading' && 
      download.status.toLowerCase() !== 'initializing' &&
      download.status.toLowerCase() !== 'completed'
  );

  return (
    <Container maxW="container.lg" py={4}>
      <VStack spacing={6} align="stretch">
        <HStack justify="space-between">
          <Heading size="md">ダウンロード一覧</Heading>
          <Button
            leftIcon={<FiRefreshCw />}
            size="sm"
            onClick={handleRefresh}
            isLoading={isRefreshing}
            variant="outline"
          >
            更新
          </Button>
        </HStack>

        {isLoading ? (
          <Center py={10}>
            <Spinner size="xl" color="brand.500" />
          </Center>
        ) : (
          <Tabs variant="enclosed" colorScheme="brand">
            <TabList>
              <Tab>
                アクティブ
                {activeDownloads.length > 0 && (
                  <Badge ml={2} colorScheme="blue" borderRadius="full">
                    {activeDownloads.length}
                  </Badge>
                )}
              </Tab>
              <Tab>
                完了済み
                {completedDownloads.length > 0 && (
                  <Badge ml={2} colorScheme="green" borderRadius="full">
                    {completedDownloads.length}
                  </Badge>
                )}
              </Tab>
              <Tab>
                その他
                {otherDownloads.length > 0 && (
                  <Badge ml={2} colorScheme="gray" borderRadius="full">
                    {otherDownloads.length}
                  </Badge>
                )}
              </Tab>
            </TabList>

            <TabPanels>
              {/* アクティブなダウンロード */}
              <TabPanel>
                {activeDownloads.length > 0 ? (
                  <SimpleGrid columns={{ base: 1, md: 2 }} spacing={4}>
                    {activeDownloads.map((download) => (
                      <DownloadCard
                        key={download.id}
                        download={download}
                        onCancel={handleCancelDownload}
                      />
                    ))}
                  </SimpleGrid>
                ) : (
                  <Center py={10}>
                    <VStack spacing={2}>
                      <Icon as={FiInfo} boxSize={10} color="gray.400" />
                      <Text color="gray.500">アクティブなダウンロードはありません</Text>
                    </VStack>
                  </Center>
                )}
              </TabPanel>

              {/* 完了済みのダウンロード */}
              <TabPanel>
                {completedDownloads.length > 0 ? (
                  <SimpleGrid columns={{ base: 1, md: 2 }} spacing={4}>
                    {completedDownloads.map((download) => (
                      <DownloadCard
                        key={download.id}
                        download={download}
                        onCancel={handleCancelDownload}
                      />
                    ))}
                  </SimpleGrid>
                ) : (
                  <Center py={10}>
                    <VStack spacing={2}>
                      <Icon as={FiInfo} boxSize={10} color="gray.400" />
                      <Text color="gray.500">完了したダウンロードはありません</Text>
                    </VStack>
                  </Center>
                )}
              </TabPanel>

              {/* その他のダウンロード */}
              <TabPanel>
                {otherDownloads.length > 0 ? (
                  <SimpleGrid columns={{ base: 1, md: 2 }} spacing={4}>
                    {otherDownloads.map((download) => (
                      <DownloadCard
                        key={download.id}
                        download={download}
                        onCancel={handleCancelDownload}
                      />
                    ))}
                  </SimpleGrid>
                ) : (
                  <Center py={10}>
                    <VStack spacing={2}>
                      <Icon as={FiInfo} boxSize={10} color="gray.400" />
                      <Text color="gray.500">その他のダウンロードはありません</Text>
                    </VStack>
                  </Center>
                )}
              </TabPanel>
            </TabPanels>
          </Tabs>
        )}
      </VStack>
    </Container>
  );
};

export default DownloadsPage;