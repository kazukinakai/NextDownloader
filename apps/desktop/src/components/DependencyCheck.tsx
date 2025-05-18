import React from 'react';
import {
  Box,
  VStack,
  HStack,
  Text,
  Icon,
  Button,
  useColorModeValue,
  Spinner,
} from '@chakra-ui/react';
import { FiCheck, FiX, FiAlertTriangle, FiRefreshCw } from 'react-icons/fi';
import { DependencyStatus } from '../types/dependency';

interface DependencyCheckProps {
  dependencies: DependencyStatus | null;
  isChecking: boolean;
  onCheck: () => void;
}

/**
 * 依存関係チェックコンポーネント
 */
const DependencyCheck: React.FC<DependencyCheckProps> = ({
  dependencies,
  isChecking,
  onCheck,
}) => {
  const bgColor = useColorModeValue('white', 'gray.700');
  const borderColor = useColorModeValue('gray.200', 'gray.600');

  // 依存関係の状態を表示するアイテム
  const DependencyItem = ({ name, installed }: { name: string; installed: boolean }) => (
    <HStack spacing={4} w="100%">
      <Box
        w="24px"
        h="24px"
        borderRadius="full"
        bg={installed ? 'green.100' : 'red.100'}
        color={installed ? 'green.500' : 'red.500'}
        display="flex"
        alignItems="center"
        justifyContent="center"
        _dark={{
          bg: installed ? 'green.900' : 'red.900',
          color: installed ? 'green.300' : 'red.300',
        }}
      >
        <Icon as={installed ? FiCheck : FiX} boxSize={4} />
      </Box>
      <Text fontWeight="medium">{name}</Text>
    </HStack>
  );

  // 全ての依存関係がインストールされているか
  const allDependenciesInstalled = dependencies
    ? dependencies.ytdlp && dependencies.aria2c && dependencies.ffmpeg
    : false;

  return (
    <Box
      borderWidth="1px"
      borderRadius="lg"
      borderColor={borderColor}
      p={4}
      bg={bgColor}
      shadow="sm"
    >
      <VStack align="stretch" spacing={4}>
        <HStack justify="space-between">
          <Text fontSize="lg" fontWeight="bold">
            依存関係チェック
          </Text>
          <Button
            leftIcon={<FiRefreshCw />}
            size="sm"
            onClick={onCheck}
            isLoading={isChecking}
            loadingText="チェック中"
            variant="outline"
          >
            再チェック
          </Button>
        </HStack>

        {isChecking ? (
          <Box textAlign="center" py={4}>
            <Spinner size="md" color="brand.500" mb={2} />
            <Text>依存関係をチェックしています...</Text>
          </Box>
        ) : dependencies ? (
          <>
            <VStack align="stretch" spacing={3}>
              <DependencyItem name="yt-dlp" installed={dependencies.ytdlp} />
              <DependencyItem name="aria2c" installed={dependencies.aria2c} />
              <DependencyItem name="ffmpeg" installed={dependencies.ffmpeg} />
            </VStack>

            {!allDependenciesInstalled && (
              <Box
                mt={2}
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
                  <Icon as={FiAlertTriangle} />
                  <Text fontSize="sm">
                    一部の依存関係がインストールされていません。すべての機能を使用するには、不足している依存関係をインストールしてください。
                  </Text>
                </HStack>
              </Box>
            )}
          </>
        ) : (
          <Text>依存関係情報を取得できませんでした。</Text>
        )}
      </VStack>
    </Box>
  );
};

export default DependencyCheck;