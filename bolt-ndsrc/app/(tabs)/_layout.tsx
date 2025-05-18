import { Tabs } from 'expo-router';
import { Download, History, Settings } from 'lucide-react-native';
import { StyleSheet } from 'react-native';
import { useTheme } from '@/context/ThemeContext';
import { colors } from '@/constants/colors';

export default function TabLayout() {
  const { isDark } = useTheme();
  const theme = colors[isDark ? 'dark' : 'light'];

  return (
    <Tabs
      screenOptions={{
        headerShown: true,
        tabBarStyle: [styles.tabBar, { backgroundColor: theme.surface, borderTopColor: theme.border }],
        tabBarActiveTintColor: theme.primary,
        tabBarInactiveTintColor: theme.textSecondary,
        headerStyle: [styles.header, { backgroundColor: theme.surface }],
        headerTitleStyle: [styles.headerTitle, { color: theme.text }],
      }}>
      <Tabs.Screen
        name="index"
        options={{
          title: 'Downloads',
          tabBarIcon: ({ color, size }) => <Download color={color} size={size} />,
        }}
      />
      <Tabs.Screen
        name="history"
        options={{
          title: 'History',
          tabBarIcon: ({ color, size }) => <History color={color} size={size} />,
        }}
      />
      <Tabs.Screen
        name="settings"
        options={{
          title: 'Settings',
          tabBarIcon: ({ color, size }) => <Settings color={color} size={size} />,
        }}
      />
    </Tabs>
  );
}

const styles = StyleSheet.create({
  tabBar: {
    borderTopWidth: 1,
    height: 60,
    paddingBottom: 8,
    paddingTop: 8,
  },
  header: {
    shadowColor: '#000000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 3,
    elevation: 3,
  },
  headerTitle: {
    fontSize: 18,
    fontWeight: '600',
  },
});