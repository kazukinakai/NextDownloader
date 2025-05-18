import React from 'react';
import { Box, VStack, Icon, Tooltip, Flex, Text, useColorModeValue } from '@chakra-ui/react';
import { NavLink, useLocation } from 'react-router-dom';
import { FiHome, FiDownload, FiSettings } from 'react-icons/fi';

/**
 * サイドバーコンポーネント
 */
const Sidebar: React.FC = () => {
  const location = useLocation();
  const bgColor = useColorModeValue('white', 'gray.900');
  const activeColor = useColorModeValue('brand.500', 'brand.300');
  const hoverColor = useColorModeValue('gray.100', 'gray.700');
  
  // ナビゲーションアイテム
  const navItems = [
    { path: '/', label: 'ホーム', icon: FiHome },
    { path: '/downloads', label: 'ダウンロード', icon: FiDownload },
    { path: '/settings', label: '設定', icon: FiSettings },
  ];
  
  return (
    <Box
      w="64px"
      h="100%"
      bg={bgColor}
      borderRight="1px"
      borderColor={useColorModeValue('gray.200', 'gray.700')}
      py={4}
    >
      <VStack spacing={4} align="center">
        {/* アプリロゴ */}
        <Box
          w="40px"
          h="40px"
          borderRadius="md"
          bg="brand.500"
          display="flex"
          alignItems="center"
          justifyContent="center"
          mb={4}
        >
          <Text
            fontSize="xl"
            fontWeight="bold"
            color="white"
          >
            ND
          </Text>
        </Box>
        
        {/* ナビゲーションアイテム */}
        {navItems.map((item) => {
          const isActive = location.pathname === item.path;
          
          return (
            <Tooltip
              key={item.path}
              label={item.label}
              placement="right"
              hasArrow
            >
              <Flex
                as={NavLink}
                to={item.path}
                w="48px"
                h="48px"
                borderRadius="md"
                align="center"
                justify="center"
                color={isActive ? activeColor : 'gray.500'}
                bg={isActive ? useColorModeValue('gray.100', 'gray.700') : 'transparent'}
                _hover={{
                  bg: hoverColor,
                  color: isActive ? activeColor : 'gray.600',
                }}
                transition="all 0.2s"
              >
                <Icon as={item.icon} boxSize={6} />
              </Flex>
            </Tooltip>
          );
        })}
      </VStack>
    </Box>
  );
};

export default Sidebar;