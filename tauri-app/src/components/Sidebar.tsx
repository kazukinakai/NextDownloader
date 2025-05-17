import React from 'react';
import { NavLink as RouterLink, useLocation } from 'react-router-dom';
import {
  Box,
  Flex,
  VStack,
  Icon,
  Text,
  Link,
  useColorModeValue,
} from '@chakra-ui/react';
import { FiHome, FiDownload, FiSettings } from 'react-icons/fi';

/**
 * サイドバーナビゲーションコンポーネント
 * アプリケーションの主要なナビゲーションリンクを提供します
 */
const Sidebar: React.FC = () => {
  const location = useLocation();
  const bgColor = useColorModeValue('gray.50', 'gray.900');
  const activeColor = useColorModeValue('brand.500', 'brand.300');
  const hoverColor = useColorModeValue('gray.100', 'gray.700');
  
  // ナビゲーションアイテムの定義
  const navItems = [
    { name: 'ホーム', icon: FiHome, path: '/' },
    { name: 'ダウンロード', icon: FiDownload, path: '/downloads' },
    { name: '設定', icon: FiSettings, path: '/settings' },
  ];
  
  return (
    <Box
      w="240px"
      h="100%"
      bg={bgColor}
      py={5}
      px={3}
    >
      {/* アプリケーションロゴ */}
      <Flex
        alignItems="center"
        justifyContent="center"
        mb={8}
        px={4}
      >
        <Text
          fontSize="xl"
          fontWeight="bold"
          color="brand.500"
        >
          NextDownloader
        </Text>
      </Flex>
      
      {/* ナビゲーションリンク */}
      <VStack spacing={1} align="stretch">
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          
          return (
            <Link
              key={item.path}
              as={RouterLink}
              to={item.path}
              textDecoration="none"
              _focus={{ boxShadow: 'none' }}
            >
              <Flex
                align="center"
                p={3}
                mx={1}
                borderRadius="md"
                role="group"
                cursor="pointer"
                bg={isActive ? activeColor : 'transparent'}
                color={isActive ? 'white' : 'inherit'}
                _hover={{
                  bg: isActive ? activeColor : hoverColor,
                }}
                transition="all 0.2s"
              >
                <Icon
                  mr={4}
                  fontSize="16"
                  as={item.icon}
                />
                <Text fontSize="sm" fontWeight={isActive ? 'medium' : 'normal'}>
                  {item.name}
                </Text>
              </Flex>
            </Link>
          );
        })}
      </VStack>
      
      {/* フッター情報 */}
      <Box mt="auto" px={4} py={3}>
        <Text fontSize="xs" color="gray.500" textAlign="center">
          NextDownloader v0.1.0
        </Text>
      </Box>
    </Box>
  );
};

export default Sidebar;