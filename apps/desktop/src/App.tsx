import React, { useEffect, useState } from 'react';
import { Routes, Route, useLocation } from 'react-router-dom';
import { Box } from '@chakra-ui/react';

// コンポーネント
import Layout from './components/Layout';

// ページ
import HomePage from './pages/HomePage';
import DownloadsPage from './pages/DownloadsPage';
import SettingsPage from './pages/SettingsPage';

// APIクライアント
import { checkDependencies } from './api/dependency';
import { DependencyStatus } from './types/dependency';

const App: React.FC = () => {
  const location = useLocation();
  const [dependencies, setDependencies] = useState<DependencyStatus | null>(null);
  const [dependenciesChecked, setDependenciesChecked] = useState(false);

  // 依存関係のチェック
  useEffect(() => {
    const checkDeps = async () => {
      try {
        const status = await checkDependencies();
        setDependencies(status);
      } catch (error) {
        console.error('依存関係のチェックに失敗しました:', error);
      } finally {
        setDependenciesChecked(true);
      }
    };

    checkDeps();
  }, []);

  return (
    <Box height="100vh" display="flex" flexDirection="column">
      <Layout>
        <Routes>
          <Route 
            path="/" 
            element={
              <HomePage 
                dependencies={dependencies} 
                dependenciesChecked={dependenciesChecked} 
              />
            } 
          />
          <Route path="/downloads" element={<DownloadsPage />} />
          <Route 
            path="/settings" 
            element={
              <SettingsPage 
                dependencies={dependencies}
                dependenciesChecked={dependenciesChecked}
                onDependenciesCheck={async () => {
                  try {
                    const status = await checkDependencies();
                    setDependencies(status);
                  } catch (error) {
                    console.error('依存関係のチェックに失敗しました:', error);
                  }
                }}
              />
            } 
          />
        </Routes>
      </Layout>
    </Box>
  );
};

export default App;