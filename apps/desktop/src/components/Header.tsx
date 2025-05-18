import React, { useEffect, useState } from 'react';
import { 
  Box, 
  Flex, 
  Heading, 
  IconButton, 
  useColorMode, 
  useColorModeValue,
  Text,
  HStack,
  Badge
} from '@chakra-ui/react';
import { FiMoon, FiSun } from 'react-icons/fi';
import { useLocation } from 'react-router-dom';
import { getVersion } from '../api/utils';

/**
 * ヘッダーコンポーネント
 */
const Header: React.FC = () => {
  const { colorMode, toggleColorMode } = useColorMode();
  const location = useLocation();
  const [version, setVersion] = useState<string>('');
  
  // パスに基づいてページタイトルを取得
  const getPageTitle = () => {
    switch (location.pathname) {
      case '/':
        return 'ホーム';
      case '/downloads':
        return 'ダウンロード';
      case '/settings':
        return '設定';
      default:
        return 'NextDownloader';
    }
  };
  
  // バージョン情報を取得
  useEffect(() => {
    const fetchVersion = async () => {
      try {
        const v = await getVersion();
        setVersion(v);
      } catch (error) {
        console.error('バージョン情報の取得に失敗しました:', error);
      }
    };
    
    fetchVersion();
  }, []);
  
  return (
    <Flex
      as="header"
      align="center"
      justify="space-between"
      py={3}
      px={4}
      bg={useColorModeValue('white', 'gray.900')}
      borderBottom="1px"
      borderColor={useColorModeValue('gray.200', 'gray.700')}
    >
      <Heading as="h1" size="md">
        {getPageTitle()}
      </Heading>
      
      <HStack spacing={4}>
        {/* バージョン情報 */}
        {version && (
          <Badge colorScheme="gray" variant="subtle">
            v{version}
          </Badge>
        )}
        
        {/* テーマ切り替えボタン */}
        <IconButton
          aria-label="テーマ切り替え"
          icon={colorMode === 'light' ? <FiMoon /> : <FiSun />}
          onClick={toggleColorMode}
          variant="ghost"
          size="sm"
        />
      </HStack>
    </Flex>
  );
};

export default Header;