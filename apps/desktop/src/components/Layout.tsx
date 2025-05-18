import React, { ReactNode } from 'react';
import { Box, Flex } from '@chakra-ui/react';
import Sidebar from './Sidebar';
import Header from './Header';

interface LayoutProps {
  children: ReactNode;
}

/**
 * アプリケーションのレイアウトコンポーネント
 */
const Layout: React.FC<LayoutProps> = ({ children }) => {
  return (
    <Flex h="100%" w="100%">
      {/* サイドバー */}
      <Sidebar />
      
      {/* メインコンテンツエリア */}
      <Flex flexDirection="column" flex="1" overflow="hidden">
        {/* ヘッダー */}
        <Header />
        
        {/* コンテンツ */}
        <Box
          flex="1"
          p={4}
          overflowY="auto"
          bg="gray.50"
          _dark={{ bg: "gray.800" }}
        >
          {children}
        </Box>
      </Flex>
    </Flex>
  );
};

export default Layout;