import React from 'react';
import {
  Box,
  Flex,
  Text,
  Progress,
  Badge,
  IconButton,
  HStack,
  VStack,
  useColorModeValue,
  Tooltip,
} from '@chakra-ui/react';
import { FiX, FiExternalLink, FiFolder } from 'react-icons/fi';
import { DownloadInfo } from '../types/download';
import { formatFileSize, formatTime } from '../api/utils';

interface DownloadCardProps {
  download: DownloadInfo;
  onCancel: (id: string) => void;
}

/**
 * ダウンロードカードコンポーネント
 */
const DownloadCard: React.FC<DownloadCardProps> = ({ download, onCancel }) => {
  // ステータスに基づいてバッジの色を決定
  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'downloading':
        return 'blue';
      case 'completed':
        return 'green';
      case 'error':
        return 'red';
      case 'cancelled':
        return 'orange';
      case 'paused':
        return 'yellow';
      default:
        return 'gray';
    }
  };

  // ステータスの日本語表示
  const getStatusText = (status: string) => {
    switch (status.toLowerCase()) {
      case 'initializing':
        return '初期化中';
      case 'downloading':
        return 'ダウンロード中';
      case 'completed':
        return '完了';
      case 'error':
        return 'エラー';
      case 'cancelled':
        return 'キャンセル';
      case 'paused':
        return '一時停止';
      default:
        return status;
    }
  };

  // 進捗情報の表示
  const progressText = () => {
    if (download.status.toLowerCase() === 'completed') {
      return `${formatFileSize(download.downloaded_size)}`;
    }

    if (download.total_size) {
      return `${formatFileSize(download.downloaded_size)} / ${formatFileSize(download.total_size)}`;
    }

    return `${formatFileSize(download.downloaded_size)}`;
  };

  // 速度情報の表示
  const speedText = () => {
    if (!download.speed || download.status.toLowerCase() !== 'downloading') {
      return null;
    }

    return `${formatFileSize(download.speed)}/s`;
  };

  // 残り時間の表示
  const etaText = () => {
    if (!download.eta || download.status.toLowerCase() !== 'downloading') {
      return null;
    }

    return `残り時間: ${formatTime(download.eta)}`;
  };

  return (
    <Box
      borderWidth="1px"
      borderRadius="lg"
      overflow="hidden"
      bg={useColorModeValue('white', 'gray.700')}
      shadow="sm"
      transition="all 0.2s"
      _hover={{ shadow: 'md' }}
    >
      <Box p={4}>
        <Flex justify="space-between" align="center" mb={2}>
          <HStack spacing={2}>
            <Badge colorScheme={getStatusColor(download.status)}>
              {getStatusText(download.status)}
            </Badge>
            <Badge variant="outline" colorScheme="gray">
              {download.content_type}
            </Badge>
          </HStack>

          {/* キャンセルボタン（ダウンロード中の場合のみ表示） */}
          {download.status.toLowerCase() === 'downloading' && (
            <IconButton
              aria-label="キャンセル"
              icon={<FiX />}
              size="sm"
              variant="ghost"
              colorScheme="red"
              onClick={() => onCancel(download.id)}
            />
          )}
        </Flex>

        {/* ファイル名 */}
        <Text fontWeight="bold" mb={2} noOfLines={1} title={download.filename}>
          {download.filename}
        </Text>

        {/* URL */}
        <Text fontSize="sm" color="gray.500" mb={2} noOfLines={1} title={download.url}>
          {download.url}
        </Text>

        {/* 進捗バー */}
        {download.status.toLowerCase() !== 'completed' && (
          <Progress
            value={download.progress * 100}
            size="sm"
            colorScheme={getStatusColor(download.status)}
            mb={2}
            borderRadius="full"
            isIndeterminate={download.status.toLowerCase() === 'initializing'}
          />
        )}

        {/* 進捗情報 */}
        <Flex justify="space-between" align="center">
          <Text fontSize="sm">{progressText()}</Text>
          <HStack spacing={2}>
            {speedText() && <Text fontSize="sm">{speedText()}</Text>}
            {etaText() && <Text fontSize="sm">{etaText()}</Text>}
          </HStack>
        </Flex>

        {/* ステータスメッセージ */}
        {download.status_message && (
          <Text fontSize="xs" color="gray.500" mt={1}>
            {download.status_message}
          </Text>
        )}
      </Box>
    </Box>
  );
};

export default DownloadCard;