import { StyleSheet, Text, View, ScrollView, TouchableOpacity, Switch } from 'react-native';
import { ChevronRight, FolderOpen, Download, Moon, Bell, Sparkles } from 'lucide-react-native';
import { useTheme } from '@/context/ThemeContext';
import { colors } from '@/constants/colors';

export default function SettingsScreen() {
  const { mode, setMode, isDark } = useTheme();
  const theme = colors[isDark ? 'dark' : 'light'];

  return (
    <ScrollView style={[styles.container, { backgroundColor: theme.background }]}>
      <View style={styles.section}>
        <Text style={[styles.sectionTitle, { color: theme.primary }]}>Download Settings</Text>
        <TouchableOpacity style={[styles.settingItem, { backgroundColor: theme.surface }]}>
          <View style={[styles.settingIcon, { backgroundColor: theme.primaryLight }]}>
            <FolderOpen size={20} color="#2563eb" />
          </View>
          <View style={styles.settingContent}>
            <Text style={[styles.settingTitle, { color: theme.text }]}>Download Location</Text>
            <Text style={[styles.settingValue, { color: theme.textSecondary }]}>Downloads</Text>
          </View>
          <ChevronRight size={20} color={theme.textSecondary} />
        </TouchableOpacity>
        
        <TouchableOpacity style={[styles.settingItem, { backgroundColor: theme.surface }]}>
          <View style={[styles.settingIcon, { backgroundColor: theme.primaryLight }]}>
            <Sparkles size={20} color="#2563eb" />
          </View>
          <View style={styles.settingContent}>
            <Text style={[styles.settingTitle, { color: theme.text }]}>Maximum Quality</Text>
            <Text style={[styles.settingValue, { color: theme.textSecondary }]}>Auto-detect best quality</Text>
          </View>
          <ChevronRight size={20} color={theme.textSecondary} />
        </TouchableOpacity>
        
        <TouchableOpacity style={[styles.settingItem, { backgroundColor: theme.surface }]}>
          <View style={[styles.settingIcon, { backgroundColor: theme.primaryLight }]}>
            <Download size={20} color="#2563eb" />
          </View>
          <View style={styles.settingContent}>
            <Text style={[styles.settingTitle, { color: theme.text }]}>Download Mode</Text>
            <Text style={[styles.settingValue, { color: theme.textSecondary }]}>Smart quality selection</Text>
          </View>
          <ChevronRight size={20} color={theme.textSecondary} />
        </TouchableOpacity>
      </View>

      <View style={styles.section}>
        <Text style={[styles.sectionTitle, { color: theme.primary }]}>Appearance</Text>
        <View style={[styles.settingItem, { backgroundColor: theme.surface }]}>
          <View style={[styles.settingIcon, { backgroundColor: theme.primaryLight }]}>
            <Moon size={20} color="#2563eb" />
          </View>
          <View style={styles.settingContent}>
            <Text style={[styles.settingTitle, { color: theme.text }]}>Dark Mode</Text>
            <Text style={[styles.settingValue, { color: theme.textSecondary }]}>
              {mode === 'system' ? 'Follow system' : mode === 'dark' ? 'On' : 'Off'}
            </Text>
          </View>
          <Switch
            value={isDark}
            onValueChange={() => setMode(isDark ? 'light' : 'dark')}
            trackColor={{ false: '#cbd5e1', true: '#93c5fd' }}
            thumbColor={false ? '#2563eb' : '#f1f5f9'}
          />
        </View>
      </View>

      <View style={styles.section}>
        <Text style={[styles.sectionTitle, { color: theme.primary }]}>Notifications</Text>
        <View style={[styles.settingItem, { backgroundColor: theme.surface }]}>
          <View style={[styles.settingIcon, { backgroundColor: theme.primaryLight }]}>
            <Bell size={20} color="#2563eb" />
          </View>
          <View style={styles.settingContent}>
            <Text style={[styles.settingTitle, { color: theme.text }]}>Download Alerts</Text>
            <Text style={[styles.settingValue, { color: theme.textSecondary }]}>Enabled</Text>
          </View>
          <Switch 
            value={true}
            onValueChange={() => {}}
            trackColor={{ false: '#cbd5e1', true: '#93c5fd' }}
            thumbColor={true ? '#2563eb' : '#f1f5f9'}
          />
        </View>
      </View>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
  },
  section: {
    marginTop: 24,
    paddingHorizontal: 20,
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '600',
    marginBottom: 12,
  },
  settingItem: {
    flexDirection: 'row',
    alignItems: 'center',
    padding: 16,
    borderRadius: 12,
    marginBottom: 8,
    shadowColor: '#000000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 3,
    elevation: 2,
  },
  settingIcon: {
    width: 36,
    height: 36,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    marginRight: 12,
  },
  settingContent: {
    flex: 1,
  },
  settingTitle: {
    fontSize: 16,
    fontWeight: '500',
    marginBottom: 2,
  },
  settingValue: {
    fontSize: 14,
  },
});