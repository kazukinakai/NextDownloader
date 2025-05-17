import React from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import { Box } from '@chakra-ui/react';

// ページコンポーネントのインポート
import HomePage from './pages/HomePage';
import DownloadsPage from './pages/DownloadsPage';
import SettingsPage from './pages/SettingsPage';

// レイアウトコンポーネントのインポート
import Layout from './components/Layout';

// アプリケーションのメインコンポーネント
const App: React.FC = () => {
  return (
    <Router>
      <Box height="100vh" width="100%" overflow="hidden">
        <Layout>
          <Routes>
            <Route path="/" element={<HomePage />} />
            <Route path="/downloads" element={<DownloadsPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </Layout>
      </Box>
    </Router>
  );
};

export default App;