import React, { ReactNode } from 'react';
import { Box, Flex, useColorModeValue } from '@chakra-ui/react';
import Sidebar from './Sidebar';
import Header from './Header';

interface LayoutProps {
  children: ReactNode;
}

/**
 * アプリケーションの基本レイアウト
 * サイドバーとヘッダーを含む全体のレイアウトを提供します
 */
const Layout: React.FC<LayoutProps> = ({ children }) => {
  const bgColor = useColorModeValue('white', 'gray.800');
  const borderColor = useColorModeValue('gray.200', 'gray.700');
  
  return (
    <Flex h="100%" w="100%">
      {/* サイドバー */}
      <Sidebar />
      
      {/* メインコンテンツエリア */}
      <Box
        flex="1"
        display="flex"
        flexDirection="column"
        bg={bgColor}
        borderLeft="1px"
        borderColor={borderColor}
        overflow="hidden"
      >
        {/* ヘッダー */}
        <Header />
        
        {/* コンテンツエリア */}
        <Box
          flex="1"
          p={4}
          overflowY="auto"
        >
          {children}
        </Box>
      </Box>
    </Flex>
  );
};

export default Layout;