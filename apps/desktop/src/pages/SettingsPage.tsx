import React, { useEffect, useState } from 'react';
import {
  Box,
  Button,
  Card,
  CardBody,
  CardHeader,
  Container,
  Divider,
  FormControl,
  FormLabel,
  Heading,
  Input,
  NumberInput,
  NumberInputField,
  NumberInputStepper,
  NumberIncrementStepper,
  NumberDecrementStepper,
  Select,
  Stack,
  Switch,
  Text,
  useToast,
  VStack,
  HStack,
  InputGroup,
  InputRightElement,
  IconButton,
  useColorMode,
} from '@chakra-ui/react';
import { FiSave, FiFolder, FiRefreshCw } from 'react-icons/fi';
import { DependencyStatus } from '../types/dependency';
import DependencyCheck from '../components/DependencyCheck';
import { getConfig, saveConfig } from '../api/config';
import { selectDirectory } from '../api/utils';
import { AppConfig } from '../types/config';

interface SettingsPageProps {
  dependencies: DependencyStatus | null;
  dependenciesChecked: boolean;
  onDependenciesCheck: () => Promise<void>;
}

/**
 * 設定ページコンポーネント
 */
const SettingsPage: React.FC<SettingsPageProps> = ({
  dependencies,
  dependenciesChecked,
  onDependenciesCheck,
}) => {
  const toast = useToast();
  const { colorMode, setColorMode } = useColorMode();
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [configPath, setConfigPath] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isCheckingDeps, setIsCheckingDeps] = useState(!dependenciesChecked);

  // 設定を取得
  const fetchConfig = async () => {
    try {
      const { config, config_path } = await getConfig();
      setConfig(config);
      setConfigPath(config_path);
    } catch (error) {
      console.error('設定の取得に失敗しました:', error);
      toast({
        title: 'エラー',
        description: '設定の取得に失敗しました',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setIsLoading(false);
    }
  };

  // 設定を保存
  const handleSaveConfig = async () => {
    if (!config) return;

    setIsSaving(true);

    try {
      await saveConfig(config);
      
      // UIのカラーモードを更新
      if (config.theme !== 'system') {
        setColorMode(config.theme);
      }
      
      toast({
        title: '保存完了',
        description: '設定を保存しました',
        status: 'success',
        duration: 3000,
        isClosable: true,
      });
    } catch (error) {
      console.error('設定の保存に失敗しました:', error);
      toast({
        title: 'エラー',
        description: '設定の保存に失敗しました',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    } finally {
      setIsSaving(false);
    }
  };

  // ディレクトリ選択
  const handleSelectDirectory = async () => {
    if (!config) return;
    
    try {
      const selectedDir = await selectDirectory(
        'デフォルトダウンロードディレクトリを選択',
        config.default_download_dir
      );
      
      if (selectedDir) {
        setConfig({
          ...config,
          default_download_dir: selectedDir,
        });
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

  // 依存関係ツールのパス選択
  const handleSelectToolPath = async (tool: 'ytdlp' | 'aria2c' | 'ffmpeg') => {
    if (!config) return;
    
    try {
      const selectedPath = await selectDirectory(
        `${tool}の実行ファイルを選択`,
        config.tool_paths[tool] || ''
      );
      
      if (selectedPath) {
        setConfig({
          ...config,
          tool_paths: {
            ...config.tool_paths,
            [tool]: selectedPath,
          },
        });
      }
    } catch (error) {
      console.error('ファイル選択に失敗しました:', error);
      toast({
        title: 'エラー',
        description: 'ファイルの選択に失敗しました',
        status: 'error',
        duration: 5000,
        isClosable: true,
      });
    }
  };

  // 初回読み込み
  useEffect(() => {
    fetchConfig();
  }, []);

  // 設定が読み込まれていない場合はローディング表示
  if (isLoading || !config) {
    return (
      <Container maxW="container.lg" py={4}>
        <VStack spacing={6} align="stretch">
          <Heading size="md">設定</Heading>
          <Text>設定を読み込んでいます...</Text>
        </VStack>
      </Container>
    );
  }

  return (
    <Container maxW="container.lg" py={4}>
      <VStack spacing={6} align="stretch">
        <HStack justify="space-between">
          <Heading size="md">設定</Heading>
          <Button
            leftIcon={<FiSave />}
            colorScheme="brand"
            onClick={handleSaveConfig}
            isLoading={isSaving}
            loadingText="保存中..."
          >
            保存
          </Button>
        </HStack>

        {/* 一般設定 */}
        <Card>
          <CardHeader>
            <Heading size="sm">一般設定</Heading>
          </CardHeader>
          <CardBody>
            <VStack spacing={4} align="stretch">
              {/* デフォルトダウンロードディレクトリ */}
              <FormControl>
                <FormLabel>デフォルトダウンロードディレクトリ</FormLabel>
                <InputGroup>
                  <Input
                    value={config.default_download_dir}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        default_download_dir: e.target.value,
                      })
                    }
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

              {/* テーマ */}
              <FormControl>
                <FormLabel>テーマ</FormLabel>
                <Select
                  value={config.theme}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      theme: e.target.value,
                    })
                  }
                >
                  <option value="light">ライト</option>
                  <option value="dark">ダーク</option>
                  <option value="system">システム設定に合わせる</option>
                </Select>
              </FormControl>

              {/* 言語 */}
              <FormControl>
                <FormLabel>言語</FormLabel>
                <Select
                  value={config.language}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      language: e.target.value,
                    })
                  }
                >
                  <option value="ja">日本語</option>
                  <option value="en">English</option>
                </Select>
              </FormControl>

              {/* 通知設定 */}
              <FormControl display="flex" alignItems="center">
                <FormLabel mb="0">ダウンロード完了時に通知する</FormLabel>
                <Switch
                  isChecked={config.notify_on_completion}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      notify_on_completion: e.target.checked,
                    })
                  }
                  colorScheme="brand"
                />
              </FormControl>

              {/* アーカイブ自動解凍 */}
              <FormControl display="flex" alignItems="center">
                <FormLabel mb="0">アーカイブを自動的に解凍する</FormLabel>
                <Switch
                  isChecked={config.auto_extract_archives}
                  onChange={(e) =>
                    setConfig({
                      ...config,
                      auto_extract_archives: e.target.checked,
                    })
                  }
                  colorScheme="brand"
                />
              </FormControl>
            </VStack>
          </CardBody>
        </Card>

        {/* ダウンロード設定 */}
        <Card>
          <CardHeader>
            <Heading size="sm">ダウンロード設定</Heading>
          </CardHeader>
          <CardBody>
            <VStack spacing={4} align="stretch">
              {/* 同時ダウンロード数 */}
              <FormControl>
                <FormLabel>同時ダウンロード数</FormLabel>
                <NumberInput
                  min={1}
                  max={10}
                  value={config.download_manager.concurrent_downloads}
                  onChange={(_, value) =>
                    setConfig({
                      ...config,
                      download_manager: {
                        ...config.download_manager,
                        concurrent_downloads: value,
                      },
                    })
                  }
                >
                  <NumberInputField />
                  <NumberInputStepper>
                    <NumberIncrementStepper />
                    <NumberDecrementStepper />
                  </NumberInputStepper>
                </NumberInput>
              </FormControl>

              {/* 再試行回数 */}
              <FormControl>
                <FormLabel>ダウンロード再試行回数</FormLabel>
                <NumberInput
                  min={0}
                  max={10}
                  value={config.download_manager.retry_count}
                  onChange={(_, value) =>
                    setConfig({
                      ...config,
                      download_manager: {
                        ...config.download_manager,
                        retry_count: value,
                      },
                    })
                  }
                >
                  <NumberInputField />
                  <NumberInputStepper>
                    <NumberIncrementStepper />
                    <NumberDecrementStepper />
                  </NumberInputStepper>
                </NumberInput>
              </FormControl>

              {/* タイムアウト */}
              <FormControl>
                <FormLabel>タイムアウト（秒）</FormLabel>
                <NumberInput
                  min={10}
                  max={300}
                  value={config.download_manager.timeout_seconds}
                  onChange={(_, value) =>
                    setConfig({
                      ...config,
                      download_manager: {
                        ...config.download_manager,
                        timeout_seconds: value,
                      },
                    })
                  }
                >
                  <NumberInputField />
                  <NumberInputStepper>
                    <NumberIncrementStepper />
                    <NumberDecrementStepper />
                  </NumberInputStepper>
                </NumberInput>
              </FormControl>

              {/* 速度制限 */}
              <FormControl>
                <FormLabel>ダウンロード速度制限（KB/s、0は無制限）</FormLabel>
                <NumberInput
                  min={0}
                  max={10000}
                  value={config.download_manager.speed_limit / 1024}
                  onChange={(_, value) =>
                    setConfig({
                      ...config,
                      download_manager: {
                        ...config.download_manager,
                        speed_limit: value * 1024,
                      },
                    })
                  }
                >
                  <NumberInputField />
                  <NumberInputStepper>
                    <NumberIncrementStepper />
                    <NumberDecrementStepper />
                  </NumberInputStepper>
                </NumberInput>
              </FormControl>
            </VStack>
          </CardBody>
        </Card>

        {/* 依存ツール設定 */}
        <Card>
          <CardHeader>
            <Heading size="sm">依存ツール設定</Heading>
          </CardHeader>
          <CardBody>
            <VStack spacing={4} align="stretch">
              {/* yt-dlpのパス */}
              <FormControl>
                <FormLabel>yt-dlpのパス（自動検出できない場合）</FormLabel>
                <InputGroup>
                  <Input
                    value={config.tool_paths.ytdlp || ''}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        tool_paths: {
                          ...config.tool_paths,
                          ytdlp: e.target.value,
                        },
                      })
                    }
                    placeholder="自動検出"
                  />
                  <InputRightElement>
                    <IconButton
                      aria-label="ファイルを選択"
                      icon={<FiFolder />}
                      size="sm"
                      onClick={() => handleSelectToolPath('ytdlp')}
                    />
                  </InputRightElement>
                </InputGroup>
              </FormControl>

              {/* aria2cのパス */}
              <FormControl>
                <FormLabel>aria2cのパス（自動検出できない場合）</FormLabel>
                <InputGroup>
                  <Input
                    value={config.tool_paths.aria2c || ''}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        tool_paths: {
                          ...config.tool_paths,
                          aria2c: e.target.value,
                        },
                      })
                    }
                    placeholder="自動検出"
                  />
                  <InputRightElement>
                    <IconButton
                      aria-label="ファイルを選択"
                      icon={<FiFolder />}
                      size="sm"
                      onClick={() => handleSelectToolPath('aria2c')}
                    />
                  </InputRightElement>
                </InputGroup>
              </FormControl>

              {/* ffmpegのパス */}
              <FormControl>
                <FormLabel>ffmpegのパス（自動検出できない場合）</FormLabel>
                <InputGroup>
                  <Input
                    value={config.tool_paths.ffmpeg || ''}
                    onChange={(e) =>
                      setConfig({
                        ...config,
                        tool_paths: {
                          ...config.tool_paths,
                          ffmpeg: e.target.value,
                        },
                      })
                    }
                    placeholder="自動検出"
                  />
                  <InputRightElement>
                    <IconButton
                      aria-label="ファイルを選択"
                      icon={<FiFolder />}
                      size="sm"
                      onClick={() => handleSelectToolPath('ffmpeg')}
                    />
                  </InputRightElement>
                </InputGroup>
              </FormControl>
            </VStack>
          </CardBody>
        </Card>

        {/* 依存関係チェック */}
        <DependencyCheck
          dependencies={dependencies}
          isChecking={isCheckingDeps}
          onCheck={onDependenciesCheck}
        />

        {/* 設定ファイルのパス */}
        <Text fontSize="sm" color="gray.500">
          設定ファイル: {configPath}
        </Text>
      </VStack>
    </Container>
  );
};

export default SettingsPage;