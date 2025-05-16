// バックグラウンドスクリプト

// 検出されたメディアを保存
let detectedMedia = [];

// Native Messagingホスト名
const nativeHostName = 'com.nextdownloader.app';

// コンテンツスクリプトからのメッセージを受信
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'mediaDetected') {
    // 重複を避けるためにURLでフィルタリング
    const newMedia = message.media.filter(item => 
      !detectedMedia.some(existing => existing.url === item.url)
    );
    
    if (newMedia.length > 0) {
      // 新しいメディアを追加
      detectedMedia = [...detectedMedia, ...newMedia];
      
      // バッジを更新
      updateBadge();
      
      // ポップアップが開いている場合は通知
      chrome.runtime.sendMessage({
        action: 'updateMediaList',
        media: detectedMedia
      });
    }
  }
  
  return true; // 非同期レスポンスを有効化
});

// ポップアップからのメッセージを受信
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'getDetectedMedia') {
    sendResponse({ media: detectedMedia });
  } else if (message.action === 'clearDetectedMedia') {
    detectedMedia = [];
    updateBadge();
    sendResponse({ success: true });
  } else if (message.action === 'downloadMedia') {
    // Macネイティブアプリにメディア情報を送信
    sendToNativeApp(message.mediaItem, message.pageInfo);
    sendResponse({ success: true });
  }
  
  return true; // 非同期レスポンスを有効化
});

// バッジを更新
function updateBadge() {
  const count = detectedMedia.length;
  if (count > 0) {
    chrome.action.setBadgeText({ text: count.toString() });
    chrome.action.setBadgeBackgroundColor({ color: '#4caf50' });
  } else {
    chrome.action.setBadgeText({ text: '' });
  }
}

// Macネイティブアプリにメッセージを送信
function sendToNativeApp(mediaItem, pageInfo) {
  const message = {
    url: mediaItem.url,
    title: pageInfo.title,
    referrer: pageInfo.url
  };
  
  chrome.runtime.sendNativeMessage(nativeHostName, message, response => {
    if (chrome.runtime.lastError) {
      console.error('Native messaging error:', chrome.runtime.lastError);
      // エラーの場合はポップアップに通知
      chrome.runtime.sendMessage({
        action: 'nativeAppError',
        error: chrome.runtime.lastError.message
      });
    } else {
      console.log('Response from native app:', response);
      // 成功の場合はポップアップに通知
      chrome.runtime.sendMessage({
        action: 'nativeAppSuccess',
        response: response
      });
    }
  });
}

// タブが更新されたときにメディアリストをクリア
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'loading') {
    // 新しいページが読み込まれた場合、そのタブに関連するメディアをクリア
    detectedMedia = detectedMedia.filter(item => 
      !item.tabId || item.tabId !== tabId
    );
    updateBadge();
  }
});

// 拡張機能がインストールまたは更新されたときの処理
chrome.runtime.onInstalled.addListener(details => {
  if (details.reason === 'install') {
    // 初回インストール時の処理
    chrome.tabs.create({ url: 'welcome.html' });
  }
});
