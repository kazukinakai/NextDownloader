import { extendTheme, type ThemeConfig } from '@chakra-ui/react';

// カラーモード設定
const config: ThemeConfig = {
  initialColorMode: 'system',
  useSystemColorMode: true,
};

// カスタムカラー
const colors = {
  brand: {
    50: '#e6f7ff',
    100: '#b3e0ff',
    200: '#80caff',
    300: '#4db3ff',
    400: '#1a9dff',
    500: '#0086e6',
    600: '#006bb8',
    700: '#00508a',
    800: '#00365c',
    900: '#001b2e',
  },
};

// フォント設定
const fonts = {
  heading: `'Noto Sans JP', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`,
  body: `'Noto Sans JP', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`,
};

// コンポーネントスタイルのカスタマイズ
const components = {
  Button: {
    baseStyle: {
      fontWeight: 'medium',
      borderRadius: 'md',
    },
    variants: {
      solid: (props: { colorScheme: string }) => ({
        bg: props.colorScheme === 'brand' ? 'brand.500' : undefined,
        _hover: {
          bg: props.colorScheme === 'brand' ? 'brand.600' : undefined,
        },
      }),
      outline: (props: { colorScheme: string }) => ({
        borderColor: props.colorScheme === 'brand' ? 'brand.500' : undefined,
        color: props.colorScheme === 'brand' ? 'brand.500' : undefined,
        _hover: {
          bg: props.colorScheme === 'brand' ? 'brand.50' : undefined,
        },
      }),
    },
  },
  Card: {
    baseStyle: {
      container: {
        borderRadius: 'lg',
        overflow: 'hidden',
      },
    },
  },
};

// テーマの拡張
const theme = extendTheme({
  config,
  colors,
  fonts,
  components,
  styles: {
    global: (props: { colorMode: string }) => ({
      body: {
        bg: props.colorMode === 'dark' ? 'gray.800' : 'gray.50',
      },
    }),
  },
});

export default theme;