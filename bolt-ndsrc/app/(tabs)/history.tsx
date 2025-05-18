import { StyleSheet, Text, View, ScrollView } from 'react-native';
import { History } from 'lucide-react-native';
import { useTheme } from '@/context/ThemeContext';
import { colors } from '@/constants/colors';

export default function HistoryScreen() {
  const { isDark } = useTheme();
  const theme = colors[isDark ? 'dark' : 'light'];

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      <ScrollView style={styles.scrollView} contentContainerStyle={styles.content}>
        <View style={styles.emptyState}>
          <View style={[styles.iconContainer, { backgroundColor: theme.surfaceVariant }]}>
            <History size={48} color={theme.textSecondary} />
          </View>
          <Text style={[styles.emptyTitle, { color: theme.text }]}>No Download History</Text>
          <Text style={[styles.emptyText, { color: theme.textSecondary }]}>
            Your completed downloads will appear here
          </Text>
        </View>
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  scrollView: {
    flex: 1,
  },
  content: {
    flexGrow: 1,
    padding: 20,
  },
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    marginTop: 40,
  },
  iconContainer: {
    width: 96,
    height: 96,
    borderRadius: 48,
    alignItems: 'center',
    justifyContent: 'center',
    marginBottom: 24,
  },
  emptyTitle: {
    fontSize: 24,
    fontWeight: '600',
    marginBottom: 8,
  },
  emptyText: {
    fontSize: 16,
    textAlign: 'center',
    maxWidth: 300,
  },
});