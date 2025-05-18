import React, { useState } from 'react';
import {
  Box,
  Button,
  Card,
  CardBody,
  CardHeader,
  Container,
  FormControl,
  FormLabel,
  Heading,
  Input,
  Select,
  Stack,
  Text,
  useToast,
  VStack,
  HStack,
  Flex,
  Divider,
  useColorModeValue,
  Icon,
  InputGroup,
  InputRightElement,
  IconButton,
} from '@chakra-ui/react';
import { FiDownload, FiFolder, FiInfo } from 'react-icons/fi';
import { DependencyStatus } from '../types/dependency';
import DependencyCheck from '../components/DependencyCheck';
import { startDownload } from '../api/download';
import { selectDirectory } from '../api/utils';
import { getConfig } from '../api/config';

interface HomePageProps {
  dependencies: DependencyStatus | null;
  dependenciesChecked: boolean;
}

/**
 * ホームページコンポーネント
 */
const HomePage: React.FC<HomePageProps> = ({ dependencies, dependenciesChecked }) => {
  const toast = useToast();
  const [url, setUrl] = useState('');
  const [destinationDir, setDestinationDir] = useState('');
  const [filename, setFilename] = useState('');
  const [format, setFormat] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isCheckingDeps, setIsCheckingDeps] = useState(!dependenciesChecked);

  // 依存関係のチェック
  const handleCheckDependencies = async () => {
    // 親コンポーネントでチェックするので何もしない
  };

  // ディレクトリ選択
  const handleSelectDirectory = async () => {
    try {
      const selectedDir = await selectDirectory('保存先ディレクトリを選択', destinationDir);
      if (selectedDir) {
        setDestinationDir(selectedDir);
      }
    } catch (error) {
      console.error('ディレクトリ選択に失敗しました:', error);
      toast({
        title: 'エラー',
        description: 'ディレクトリの選択に失敗しました',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    }
  };

  // 設定からデフォルトディレクトリを取得
  const loadDefaultDirectory = async () => {
    try {
      const { config } = await getConfig();
      if (config.default_download_dir && !destinationDir) {
        setDestinationDir(config.default_download_dir);
      }
    } catch (error) {
      console.error('設定の取得に失敗しました:', error);
    }
  };

  // コンポーネントマウント時にデフォルトディレクトリを読み込む
  React.useEffect(() => {
    loadDefaultDirectory();
  }, []);

  // ダウンロード開始
  const handleStartDownload = async () => {
    if (!url) {
      toast({
        title: '入力エラー',
        description: 'URLを入力してください',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
      return;
    }

    if (!destinationDir) {
      toast({
        title: '入力エラー',
        description: '保存先ディレクトリを選択してください',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
      return;
    }

    setIsLoading(true);

    try {
      const response = await startDownload({
        url,
        destination_dir: destinationDir,
        filename: filename || undefined,
        format: format || undefined,
      });

      toast({
        title: 'ダウンロード開始',
        description: 'ダウンロードを開始しました',
        status: 'success',
        duration: 5000,
        isClosable: true,
      });

      // フォームをリセット
      setUrl('');
      setFilename('');
      setFormat('');
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

  // 依存関係のチェック結果に基づいて、ダウンロードボタンを無効化するかどうかを決定
  const isDownloadDisabled = !dependencies || !(dependencies.ytdlp || dependencies.aria2c);

  return (
    <Container maxW="container.lg" py={4}>
      <VStack spacing={6} align="stretch">
        {/* ダウンロードフォーム */}
        <Card>
          <CardHeader>
            <Heading size="md">ダウンロード</Heading>
          </CardHeader>
          <CardBody>
            <VStack spacing={4} align="stretch">
              {/* URL入力 */}
              <FormControl isRequired>
                <FormLabel>URL</FormLabel>
                <Input
                  placeholder="https://example.com/video.mp4"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                />
              </FormControl>

              {/* 保存先ディレクトリ */}
              <FormControl isRequired>
                <FormLabel>保存先ディレクトリ</FormLabel>
                <InputGroup>
                  <Input
                    placeholder="保存先ディレクトリを選択"
                    value={destinationDir}
                    onChange={(e) => setDestinationDir(e.target.value)}
                    readOnly
                  />
                  <InputRightElement>
                    <IconButton
                      aria-label="ディレクトリを選択"
                      icon={<FiFolder />}
                      size="sm"
                      onClick={handleSelectDirectory}
                    />
                  </InputRightElement>
                </InputGroup>
              </FormControl>

              {/* ファイル名（オプション） */}
              <FormControl>
                <FormLabel>ファイル名（オプション）</FormLabel>
                <Input
                  placeholder="ファイル名"
                  value={filename}
                  onChange={(e) => setFilename(e.target.value)}
                />
              </FormControl>

              {/* フォーマット（オプション） */}
              <FormControl>
                <FormLabel>フォーマット（オプション）</FormLabel>
                <Select
                  placeholder="フォーマットを選択"
                  value={format}
                  onChange={(e) => setFormat(e.target.value)}
                >
                  <option value="mp4">MP4</option>
                  <option value="webm">WebM</option>
                  <option value="mp3">MP3 (音声のみ)</option>
                  <option value="best">最高品質</option>
                </Select>
              </FormControl>

              {/* ダウンロードボタン */}
              <Button
                leftIcon={<FiDownload />}
                colorScheme="brand"
                onClick={handleStartDownload}
                isLoading={isLoading}
                loadingText="開始中..."
                isDisabled={isDownloadDisabled}
              >
                ダウンロード開始
              </Button>

              {/* 依存関係の警告 */}
              {isDownloadDisabled && (
                <Box
                  p={3}
                  borderRadius="md"
                  bg="yellow.50"
                  color="yellow.800"
                  _dark={{
                    bg: 'yellow.900',
                    color: 'yellow.200',
                  }}
                >
                  <HStack>
                    <Icon as={FiInfo} />
                    <Text fontSize="sm">
                      ダウンロードを開始するには、必要な依存関係（yt-dlpまたはaria2c）をインストールしてください。
                    </Text>
                  </HStack>
                </Box>
              )}
            </VStack>
          </CardBody>
        </Card>

        {/* 依存関係チェック */}
        <DependencyCheck
          dependencies={dependencies}
          isChecking={isCheckingDeps}
          onCheck={handleCheckDependencies}
        />
      </VStack>
    </Container>
  );
};

export default HomePage;