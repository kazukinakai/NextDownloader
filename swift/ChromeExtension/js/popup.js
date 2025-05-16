// ポップアップスクリプト

// DOM要素
const statusContainer = document.getElementById('statusContainer');
const statusText = document.getElementById('statusText');
const mediaContainer = document.getElementById('mediaContainer');
const mediaList = document.getElementById('mediaList');
const scanButton = document.getElementById('scanButton');
const downloadButton = document.getElementById('downloadButton');

// 選択されたメディア
let selectedMediaItem = null;
let pageInfo = {
  title: '',
  url: ''
};

// 初期化
document.addEventListener('DOMContentLoaded', () => {
  // 現在のタブ情報を取得
  chrome.tabs.query({ active: true, currentWindow: true }, tabs => {
    const currentTab = tabs[0];
    pageInfo = {
      title: currentTab.title,
      url: currentTab.url
    };
    
    // 検出済みメディアを取得
    chrome.runtime.sendMessage({ action: 'getDetectedMedia' }, response => {
      if (response && response.media && response.media.length > 0) {
        updateMediaList(response.media);
      } else {
        statusContainer.className = 'status inactive';
        statusText.textContent = 'このページでメディアが検出されていません';
      }
    });
  });
  
  // スキャンボタンのイベントリスナー
  scanButton.addEventListener('click', () => {
    scanPage();
  });
  
  // ダウンロードボタンのイベントリスナー
  downloadButton.addEventListener('click', () => {
    if (selectedMediaItem) {
      downloadMedia(selectedMediaItem);
    }
  });
});

// ページをスキャン
function scanPage() {
  statusContainer.className = 'status inactive';
  statusText.textContent = 'スキャン中...';
  
  chrome.tabs.query({ active: true, currentWindow: true }, tabs => {
    const currentTab = tabs[0];
    
    chrome.tabs.sendMessage(currentTab.id, { action: 'scanPage' }, response => {
      if (chrome.runtime.lastError) {
        statusText.textContent = 'エラー: ' + chrome.runtime.lastError.message;
        return;
      }
      
      if (response && response.media && response.media.length > 0) {
        updateMediaList(response.media);
        
        // バックグラウンドスクリプトにも通知
        chrome.runtime.sendMessage({
          action: 'mediaDetected',
          media: response.media,
          pageInfo: {
            title: currentTab.title,
            url: currentTab.url
          }
        });
      } else {
        statusContainer.className = 'status inactive';
        statusText.textContent = 'メディアが検出されませんでした';
      }
    });
  });
}

// メディアリストを更新
function updateMediaList(media) {
  if (media.length > 0) {
    statusContainer.className = 'status active';
    statusText.textContent = `${media.length}個のメディアが検出されました`;
    
    mediaContainer.style.display = 'block';
    mediaList.innerHTML = '';
    
    media.forEach((item, index) => {
      const mediaItem = document.createElement('div');
      mediaItem.className = 'media-item';
      
      const mediaUrl = document.createElement('div');
      mediaUrl.className = 'media-url';
      mediaUrl.textContent = truncateUrl(item.url);
      mediaUrl.title = item.url;
      
      mediaUrl.addEventListener('click', () => {
        // 選択状態を更新
        document.querySelectorAll('.media-item').forEach(el => {
          el.style.backgroundColor = '';
        });
        mediaItem.style.backgroundColor = '#e3f2fd';
        
        selectedMediaItem = item;
        downloadButton.disabled = false;
      });
      
      mediaItem.appendChild(mediaUrl);
      mediaList.appendChild(mediaItem);
    });
  } else {
    statusContainer.className = 'status inactive';
    statusText.textContent = 'メディアが検出されませんでした';
    mediaContainer.style.display = 'none';
  }
}

// URLを短縮表示
function truncateUrl(url) {
  if (url.length > 50) {
    return url.substring(0, 47) + '...';
  }
  return url;
}

// メディアをダウンロード
function downloadMedia(mediaItem) {
  chrome.runtime.sendMessage({
    action: 'downloadMedia',
    mediaItem: mediaItem,
    pageInfo: pageInfo
  }, response => {
    if (response && response.success) {
      statusText.textContent = 'ダウンロードリクエストを送信しました';
      downloadButton.disabled = true;
    } else {
      statusText.textContent = 'ダウンロードリクエストの送信に失敗しました';
    }
  });
}

// バックグラウンドスクリプトからのメッセージを受信
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'updateMediaList') {
    updateMediaList(message.media);
  } else if (message.action === 'nativeAppError') {
    statusContainer.className = 'status inactive';
    statusText.textContent = 'エラー: ' + message.error;
  } else if (message.action === 'nativeAppSuccess') {
    statusContainer.className = 'status active';
    statusText.textContent = 'ダウンロードが開始されました';
  }
  
  return true; // 非同期レスポンスを有効化
});
