import { StyleSheet, Text, View, TouchableOpacity, ScrollView } from 'react-native';
import { Plus, Download, Link } from 'lucide-react-native';
import { TextInput, Alert } from 'react-native';
import { useTheme } from '@/context/ThemeContext';
import { colors } from '@/constants/colors';
import { useState } from 'react';
import { isValidUrl } from '@/utils/validation';

export default function DownloadsScreen() {
  const [url, setUrl] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const { isDark } = useTheme();
  const theme = colors[isDark ? 'dark' : 'light'];

  const handleDownload = async () => {
    if (!url) return;
    
    if (!isValidUrl(url)) {
      Alert.alert('Invalid URL', 'Please enter a valid video URL');
      return;
    }

    setIsLoading(true);
    try {
      // TODO: Implement actual download logic
      console.log('Downloading:', url);
      Alert.alert('Success', 'Download started successfully');
      setUrl('');
    } catch (error) {
      Alert.alert('Error', 'Failed to start download');
      console.error(error);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <View style={[styles.container, { backgroundColor: theme.background }]}>
      <ScrollView style={styles.scrollView} contentContainerStyle={styles.content}>
        <View style={styles.emptyState}>
          <View style={styles.inputContainer}>
            <View style={[styles.inputWrapper, { backgroundColor: theme.surface }]}>
              <Link size={20} color={theme.textSecondary} style={styles.inputIcon} />
              <TextInput
                style={[styles.input, { color: theme.text }]}
                placeholder="Enter video URL"
                returnKeyType="go"
                onSubmitEditing={handleDownload}
                placeholderTextColor={theme.placeholder}
                value={url}
                onChangeText={setUrl}
                autoCapitalize="none"
                autoCorrect={false}
                keyboardType="url"
                editable={!isLoading}
              />
            </View>
            <TouchableOpacity 
              style={[
                styles.downloadButton,
                (!url || isLoading) && styles.downloadButtonDisabled
              ]} 
              onPress={handleDownload}
              disabled={!url || isLoading}
            >
              {isLoading ? (
                <View style={styles.loadingIndicator} />
              ) : (
                <Download size={20} color="#ffffff" />
              )}
            </TouchableOpacity>
          </View>
          <View style={styles.iconContainer}>
            <Download size={48} color={theme.textSecondary} />
          </View>
          <Text style={[styles.emptyTitle, { color: theme.text }]}>No Active Downloads</Text>
          <Text style={[styles.emptyText, { color: theme.textSecondary }]}>
            Start by adding a new download using the button below
          </Text>
          <TouchableOpacity style={styles.addButton} onPress={() => {}}>
            <Plus size={24} color="#ffffff" />
            <Text style={styles.buttonText}>Add New Download</Text>
          </TouchableOpacity>
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
    paddingTop: 32,
  },
  emptyState: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'flex-start',
  },
  inputContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    width: '100%',
    maxWidth: 500,
    marginBottom: 48,
  },
  inputWrapper: {
    flex: 1,
    flexDirection: 'row',
    alignItems: 'center',
    borderRadius: 12,
    paddingHorizontal: 16,
    height: 48,
    marginRight: 12,
    shadowColor: '#000000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.05,
    shadowRadius: 3,
    elevation: 2,
  },
  inputIcon: {
    marginRight: 12,
  },
  input: {
    flex: 1,
    fontSize: 16,
    height: '100%',
  },
  downloadButton: {
    width: 48,
    height: 48,
    borderRadius: 12,
    backgroundColor: '#2563eb',
    alignItems: 'center',
    justifyContent: 'center',
    shadowColor: '#2563eb',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.2,
    shadowRadius: 8,
    elevation: 4,
  },
  downloadButtonDisabled: {
    backgroundColor: '#94a3b8',
    shadowOpacity: 0,
  },
  loadingIndicator: {
    width: 20,
    height: 20,
    borderRadius: 10,
    borderWidth: 2,
    borderColor: '#ffffff',
    borderTopColor: 'transparent',
    transform: [{ rotate: '45deg' }],
  },
  iconContainer: {
    width: 96,
    height: 96,
    borderRadius: 48,
    backgroundColor: '#f1f5f9',
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
    marginBottom: 32,
    maxWidth: 300,
  },
  addButton: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#2563eb',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 12,
    shadowColor: '#2563eb',
    shadowOffset: { width: 0, height: 4 },
    shadowOpacity: 0.2,
    shadowRadius: 8,
    elevation: 4,
  },
  buttonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
    marginLeft: 8,
  },
});