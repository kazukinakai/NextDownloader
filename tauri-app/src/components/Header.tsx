import React from 'react';
import { useLocation } from 'react-router-dom';
import {
  Flex,
  Heading,
  IconButton,
  useColorMode,
  useColorModeValue,
} from '@chakra-ui/react';
import { FiMoon, FiSun } from 'react-icons/fi';

/**
 * ヘッダーコンポーネント
 * 現在のページタイトルとテーマ切り替えボタンを表示します
 */
const Header: React.FC = () => {
  const { colorMode, toggleColorMode } = useColorMode();
  const location = useLocation();
  const bgColor = useColorModeValue('white', 'gray.800');
  const borderColor = useColorModeValue('gray.200', 'gray.700');
  
  // 現在のパスに基づいてページタイトルを取得
  const getPageTitle = () => {
    switch (location.pathname) {
      case '/':
        return 'ホーム';
      case '/downloads':
        return 'ダウンロード管理';
      case '/settings':
        return '設定';
      default:
        return 'NextDownloader';
    }
  };
  
  return (
    <Flex
      as="header"
      align="center"
      justify="space-between"
      py={4}
      px={6}
      bg={bgColor}
      borderBottom="1px"
      borderColor={borderColor}
    >
      <Heading size="md" fontWeight="semibold">
        {getPageTitle()}
      </Heading>
      
      {/* テーマ切り替えボタン */}
      <IconButton
        aria-label="テーマ切り替え"
        icon={colorMode === 'light' ? <FiMoon /> : <FiSun />}
        onClick={toggleColorMode}
        variant="ghost"
        size="md"
      />
    </Flex>
  );
};

export default Header;