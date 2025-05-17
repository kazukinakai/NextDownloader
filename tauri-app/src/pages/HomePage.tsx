import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Box,
  Button,
  Card,
  CardBody,
  Container,
  Flex,
  FormControl,
  FormLabel,
  Heading,
  Input,
  Text,
  VStack,
  useToast,
  Alert,
  AlertIcon,
  AlertTitle,
  AlertDescription,
} from '@chakra-ui/react';
import { FiDownload, FiLink } from 'react-icons/fi';

// APIラッパーのインポート
import { checkDependencies, detectContentType, startDownload, ContentType } from '../api/downloader';

/**
 * ホームページコンポーネント
 * メインのダウンロード機能を提供します
 */
const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const toast = useToast();
  
  // 状態管理
  const [url, setUrl] = useState<string>('');
  const [destination, setDestination] = useState<string>('');
  const [format, setFormat] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [contentType, setContentType] = useState<ContentType | null>(null);
  const [dependenciesChecked, setDependenciesChecked] = useState<boolean>(false);
  const [dependenciesStatus, setDependenciesStatus] = useState({
    ytdlp: false,
    aria2c: false,
    ffmpeg: false,
  });
  
  // 依存関係のチェック
  useEffect(() => {
    const checkDeps = async () => {
      try {
        const status = await checkDependencies();
        setDependenciesStatus(status);
        setDependenciesChecked(true);
        
        // 依存関係に問題がある場合は警告を表示
        if (!status.ytdlp || !status.aria2c || !status.ffmpeg) {
          toast({
            title: '依存関係の問題',
            description: '一部の依存関係が見つかりません。設定ページで確認してください。',
            status: 'warning',
            duration: 5000,
            isClosable: true,
          });
        }
      } catch (error) {
        console.error('依存関係のチェックに失敗しました:', error);
        toast({
          title: 'エラー',
          description: '依存関係のチェックに失敗しました',
          status: 'error',
          duration: 5000,
          isClosable: true,
        });
      }
    };
    
    checkDeps();
  }, [toast]);
  
  // URLの検証とコンテンツタイプの検出
  const handleUrlChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const newUrl = e.target.value;
    setUrl(newUrl);
    
    // URLが空の場合はコンテンツタイプをリセット
    if (!newUrl) {
      setContentType(null);
      return;
    }
    
    // URLが十分に長い場合はコンテンツタイプを検出
    if (newUrl.length > 10 && (newUrl.startsWith('http://') || newUrl.startsWith('https://'))) {
      try {
        const type = await detectContentType(newUrl);
        setContentType(type);
      } catch (error) {
        console.error('コンテンツタイプの検出に失敗しました:', error);
        setContentType(null);
      }
    }
  };
  
  // ダウンロードの開始
  const handleDownload = async () => {
    if (!url) {
      toast({
        title: 'URLが必要です',
        description: 'ダウンロードするURLを入力してください',
        status: 'error',
        duration: 3000,
        isClosable: true,
      });
      return;
    }
    
    if (!destination) {
      toast({
        title: '保存先が必要です',
        description: 'ファイルの保存先を指定してください',
        status: 'error',
        duration: 3000,
        isClosable: true,
      });
      return;
    }
    
    setIsLoading(true);
    
    try {
      const downloadId = await startDownload(url, destination, format || undefined);
      
      toast({
        title: 'ダウンロードを開始しました',
        description: 'ダウンロード管理ページで進捗を確認できます',
        status: 'success',
        duration: 3000,
        isClosable: true,
      });
      
      // ダウンロード管理ページに移動
      navigate('/downloads');
    } catch (error) {
      console.error('ダウンロードの開始に失敗しました:', error);
      toast({
        title: 'エラー',
        description: `ダウンロードの開始に失敗しました: ${error}`,
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setIsLoading(false);
    }
  };
  
  // 依存関係の問題がある場合の警告表示
  const renderDependencyWarning = () => {
    if (!dependenciesChecked) return null;
    
    const missingDeps = [];
    if (!dependenciesStatus.ytdlp) missingDeps.push('yt-dlp');
    if (!dependenciesStatus.aria2c) missingDeps.push('aria2c');
    if (!dependenciesStatus.ffmpeg) missingDeps.push('ffmpeg');
    
    if (missingDeps.length === 0) return null;
    
    return (
      <Alert status="warning" mb={4} borderRadius="md">
        <AlertIcon />
        <Box>
          <AlertTitle>依存関係の問題</AlertTitle>
          <AlertDescription>
            以下の依存関係が見つかりません: {missingDeps.join(', ')}
            <br />
            設定ページでインストール方法を確認してください。
          </AlertDescription>
        </Box>
      </Alert>
    );
  };
  
  return (
    <Container maxW="container.md" py={4}>
      <VStack spacing={6} align="stretch">
        <Box textAlign="center" mb={4}>
          <Heading size="lg" mb={2}>NextDownloader</Heading>
          <Text color="gray.600">様々な形式のコンテンツを簡単にダウンロード</Text>
        </Box>
        
        {renderDependencyWarning()}
        
        <Card variant="outline">
          <CardBody>
            <VStack spacing={4} align="stretch">
              {/* URL入力 */}
              <FormControl isRequired>
                <FormLabel>ダウンロードURL</FormLabel>
                <Input
                  placeholder="https://example.com/video.mp4"
                  value={url}
                  onChange={handleUrlChange}
                  leftIcon={<FiLink />}
                />
                {contentType && (
                  <Text fontSize="sm" color="blue.500" mt={1}>
                    検出されたコンテンツタイプ: {contentType}
                  </Text>
                )}
              </FormControl>
              
              {/* 保存先の指定 */}
              <FormControl isRequired>
                <FormLabel>保存先</FormLabel>
                <Input
                  placeholder="/Users/username/Downloads/"
                  value={destination}
                  onChange={(e) => setDestination(e.target.value)}
                />
              </FormControl>
              
              {/* フォーマットの指定（オプション） */}
              <FormControl>
                <FormLabel>希望するフォーマット（オプション）</FormLabel>
                <Input
                  placeholder="mp4, webm, etc."
                  value={format}
                  onChange={(e) => setFormat(e.target.value)}
                />
              </FormControl>
              
              {/* ダウンロードボタン */}
              <Flex justify="center" mt={2}>
                <Button
                  colorScheme="blue"
                  size="lg"
                  leftIcon={<FiDownload />}
                  onClick={handleDownload}
                  isLoading={isLoading}
                  loadingText="処理中..."
                  isDisabled={!url || !destination}
                >
                  ダウンロード開始
                </Button>
              </Flex>
            </VStack>
          </CardBody>
        </Card>
        
        {/* 使用方法のヒント */}
        <Card variant="outline">
          <CardBody>
            <Heading size="sm" mb={2}>使用方法</Heading>
            <Text fontSize="sm">
              1. ダウンロードしたいコンテンツのURLを入力してください。<br />
              2. ファイルの保存先を指定してください。<br />
              3. 必要に応じて希望するフォーマットを指定してください。<br />
              4. 「ダウンロード開始」ボタンをクリックしてダウンロードを開始します。
            </Text>
          </CardBody>
        </Card>
      </VStack>
    </Container>
  );
};

export default HomePage;