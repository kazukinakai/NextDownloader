import React, { useState, useEffect } from 'react';
import {
  Box,
  Button,
  Card,
  CardBody,
  Container,
  Divider,
  FormControl,
  FormLabel,
  Heading,
  Input,
  Switch,
  Text,
  VStack,
  HStack,
  useToast,
  Badge,
  Select,
} from '@chakra-ui/react';
import { FiSave, FiRefreshCw } from 'react-icons/fi';

// APIラッパーのインポート
import { checkDependencies } from '../api/downloader';

/**
 * 設定ページコンポーネント
 * アプリケーションの設定と依存関係の管理を行います
 */
const SettingsPage: React.FC = () => {
  const toast = useToast();
  
  // 状態管理
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [dependenciesStatus, setDependenciesStatus] = useState({
    ytdlp: false,
    aria2c: false,
    ffmpeg: false,
  });
  const [settings, setSettings] = useState({
    downloadPath: '/Users/username/Downloads',
    maxConcurrentDownloads: 3,
    notifyOnCompletion: true,
    autoExtractArchives: false,
    theme: 'system',
    language: 'ja',
  });
  
  // 依存関係のチェック
  useEffect(() => {
    checkDependenciesStatus();
  }, []);
  
  // 依存関係のステータスを確認する関数
  const checkDependenciesStatus = async () => {
    setIsLoading(true);
    
    try {
      const status = await checkDependencies();
      setDependenciesStatus(status);
      
      // すべての依存関係が揃っている場合は成功メッセージを表示
      if (status.ytdlp && status.aria2c && status.ffmpeg) {
        toast({
          title: '依存関係のチェック完了',
          description: 'すべての依存関係が正常に見つかりました',
          status: 'success',
          duration: 3000,
          isClosable: true,
        });
      } else {
        // 不足している依存関係がある場合は警告を表示
        toast({
          title: '依存関係の問題',
          description: '一部の依存関係が見つかりません',
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
    } finally {
      setIsLoading(false);
    }
  };
  
  // 設定を保存する関数
  const saveSettings = () => {
    // 実際のアプリケーションでは、設定をファイルやデータベースに保存する
    // ここではデモとして成功メッセージを表示するだけ
    toast({
      title: '設定を保存しました',
      status: 'success',
      duration: 3000,
      isClosable: true,
    });
  };
  
  // 依存関係のステータスバッジを表示する関数
  const renderStatusBadge = (isAvailable: boolean) => {
    return isAvailable ? (
      <Badge colorScheme="green">インストール済み</Badge>
    ) : (
      <Badge colorScheme="red">未インストール</Badge>
    );
  };
  
  // 依存関係のインストール手順を表示する関数
  const renderInstallInstructions = (dependency: string) => {
    switch (dependency) {
      case 'yt-dlp':
        return (
          <Text fontSize="sm" color="gray.600" mt={1}>
            インストール方法: <code>pip install yt-dlp</code> または <code>brew install yt-dlp</code>
          </Text>
        );
      case 'aria2c':
        return (
          <Text fontSize="sm" color="gray.600" mt={1}>
            インストール方法: <code>brew install aria2</code> または <code>apt-get install aria2</code>
          </Text>
        );
      case 'ffmpeg':
        return (
          <Text fontSize="sm" color="gray.600" mt={1}>
            インストール方法: <code>brew install ffmpeg</code> または <code>apt-get install ffmpeg</code>
          </Text>
        );
      default:
        return null;
    }
  };
  
  return (
    <Container maxW="container.md" py={4}>
      <VStack spacing={6} align="stretch">
        <Box>
          <Heading size="md" mb={4}>設定</Heading>
          
          {/* 一般設定 */}
          <Card variant="outline" mb={6}>
            <CardBody>
              <Heading size="sm" mb={4}>一般設定</Heading>
              
              <VStack spacing={4} align="stretch">
                {/* デフォルトのダウンロードパス */}
                <FormControl>
                  <FormLabel>デフォルトのダウンロードパス</FormLabel>
                  <Input
                    value={settings.downloadPath}
                    onChange={(e) => setSettings({ ...settings, downloadPath: e.target.value })}
                  />
                </FormControl>
                
                {/* 同時ダウンロード数 */}
                <FormControl>
                  <FormLabel>最大同時ダウンロード数</FormLabel>
                  <Select
                    value={settings.maxConcurrentDownloads}
                    onChange={(e) => setSettings({ ...settings, maxConcurrentDownloads: parseInt(e.target.value) })}
                  >
                    <option value={1}>1</option>
                    <option value={2}>2</option>
                    <option value={3}>3</option>
                    <option value={5}>5</option>
                    <option value={10}>10</option>
                  </Select>
                </FormControl>
                
                {/* テーマ設定 */}
                <FormControl>
                  <FormLabel>テーマ</FormLabel>
                  <Select
                    value={settings.theme}
                    onChange={(e) => setSettings({ ...settings, theme: e.target.value })}
                  >
                    <option value="light">ライト</option>
                    <option value="dark">ダーク</option>
                    <option value="system">システム設定に合わせる</option>
                  </Select>
                </FormControl>
                
                {/* 言語設定 */}
                <FormControl>
                  <FormLabel>言語</FormLabel>
                  <Select
                    value={settings.language}
                    onChange={(e) => setSettings({ ...settings, language: e.target.value })}
                  >
                    <option value="ja">日本語</option>
                    <option value="en">English</option>
                  </Select>
                </FormControl>
                
                <Divider />
                
                {/* 通知設定 */}
                <FormControl display="flex" alignItems="center">
                  <FormLabel mb={0}>ダウンロード完了時に通知する</FormLabel>
                  <Switch
                    isChecked={settings.notifyOnCompletion}
                    onChange={(e) => setSettings({ ...settings, notifyOnCompletion: e.target.checked })}
                  />
                </FormControl>
                
                {/* アーカイブ自動展開設定 */}
                <FormControl display="flex" alignItems="center">
                  <FormLabel mb={0}>アーカイブを自動的に展開する</FormLabel>
                  <Switch
                    isChecked={settings.autoExtractArchives}
                    onChange={(e) => setSettings({ ...settings, autoExtractArchives: e.target.checked })}
                  />
                </FormControl>
                
                {/* 保存ボタン */}
                <Box mt={2}>
                  <Button
                    leftIcon={<FiSave />}
                    colorScheme="blue"
                    onClick={saveSettings}
                  >
                    設定を保存
                  </Button>
                </Box>
              </VStack>
            </CardBody>
          </Card>
          
          {/* 依存関係の管理 */}
          <Card variant="outline">
            <CardBody>
              <HStack justify="space-between" mb={4}>
                <Heading size="sm">依存関係の管理</Heading>
                <Button
                  size="sm"
                  leftIcon={<FiRefreshCw />}
                  onClick={checkDependenciesStatus}
                  isLoading={isLoading}
                >
                  再チェック
                </Button>
              </HStack>
              
              <VStack spacing={4} align="stretch">
                {/* yt-dlp */}
                <Box>
                  <HStack>
                    <Text fontWeight="medium">yt-dlp</Text>
                    {renderStatusBadge(dependenciesStatus.ytdlp)}
                  </HStack>
                  <Text fontSize="sm" color="gray.600">
                    YouTubeなどの動画サイトからのダウンロードに必要です
                  </Text>
                  {!dependenciesStatus.ytdlp && renderInstallInstructions('yt-dlp')}
                </Box>
                
                {/* aria2c */}
                <Box>
                  <HStack>
                    <Text fontWeight="medium">aria2c</Text>
                    {renderStatusBadge(dependenciesStatus.aria2c)}
                  </HStack>
                  <Text fontSize="sm" color="gray.600">
                    高速なダウンロードとレジューム機能に必要です
                  </Text>
                  {!dependenciesStatus.aria2c && renderInstallInstructions('aria2c')}
                </Box>
                
                {/* ffmpeg */}
                <Box>
                  <HStack>
                    <Text fontWeight="medium">ffmpeg</Text>
                    {renderStatusBadge(dependenciesStatus.ffmpeg)}
                  </HStack>
                  <Text fontSize="sm" color="gray.600">
                    動画・音声の変換と処理に必要です
                  </Text>
                  {!dependenciesStatus.ffmpeg && renderInstallInstructions('ffmpeg')}
                </Box>
              </VStack>
            </CardBody>
          </Card>
        </Box>
      </VStack>
    </Container>
  );
};

export default SettingsPage;