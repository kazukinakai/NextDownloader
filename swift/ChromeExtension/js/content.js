// HLS動画を検出するコンテンツスクリプト

// メディアソースを検出する関数
function detectMediaSources() {
  const mediaElements = document.querySelectorAll('video, audio');
  const mediaSources = [];
  
  // video/audioタグからソースを検出
  mediaElements.forEach(media => {
    // sourceタグからのソース
    const sources = Array.from(media.querySelectorAll('source'));
    sources.forEach(source => {
      if (source.src.includes('.m3u8') || source.src.includes('.mpd')) {
        mediaSources.push({
          url: source.src,
          type: source.type || 'unknown',
          element: 'source'
        });
      }
    });
    
    // video/audioタグ自体のsrc属性
    if (media.src && (media.src.includes('.m3u8') || media.src.includes('.mpd'))) {
      mediaSources.push({
        url: media.src,
        type: media.type || 'unknown',
        element: media.tagName.toLowerCase()
      });
    }
  });
  
  // ネットワークリクエストからHLSストリームを検出（可能であれば）
  // 注: これは完全には機能しない場合があります（ブラウザの制限による）
  
  return mediaSources;
}

// ページ内のすべてのリクエストを監視する（可能であれば）
function monitorNetworkRequests() {
  // 実装は複雑なため、ここでは簡略化
  // 実際の実装では、Performance API や DevTools Protocol を使用する必要があるかもしれません
}

// 検出されたメディアをバックグラウンドスクリプトに送信
function sendMediaToBackground(mediaSources) {
  chrome.runtime.sendMessage({
    action: 'mediaDetected',
    media: mediaSources,
    pageInfo: {
      title: document.title,
      url: document.location.href
    }
  });
}

// 初期検出
window.addEventListener('load', () => {
  setTimeout(() => {
    const mediaSources = detectMediaSources();
    if (mediaSources.length > 0) {
      sendMediaToBackground(mediaSources);
    }
  }, 2000); // ページ読み込み後少し待機
});

// 動的に追加されるメディア要素を監視
const observer = new MutationObserver(mutations => {
  mutations.forEach(mutation => {
    if (mutation.addedNodes.length > 0) {
      setTimeout(() => {
        const mediaSources = detectMediaSources();
        if (mediaSources.length > 0) {
          sendMediaToBackground(mediaSources);
        }
      }, 500);
    }
  });
});

// DOM変更の監視を開始
observer.observe(document.body, {
  childList: true,
  subtree: true
});

// バックグラウンドスクリプトからのメッセージを受信
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === 'scanPage') {
    const mediaSources = detectMediaSources();
    sendResponse({ media: mediaSources });
  }
  return true; // 非同期レスポンスを有効化
});

// XHRリクエストをフックして.m3u8や.mpdファイルを検出
const originalXhrOpen = XMLHttpRequest.prototype.open;
XMLHttpRequest.prototype.open = function(method, url) {
  if (typeof url === 'string' && (url.includes('.m3u8') || url.includes('.mpd'))) {
    setTimeout(() => {
      sendMediaToBackground([{
        url: url,
        type: 'xhr-detected',
        element: 'xhr'
      }]);
    }, 0);
  }
  return originalXhrOpen.apply(this, arguments);
};

// Fetch APIをフックして.m3u8や.mpdファイルを検出
const originalFetch = window.fetch;
window.fetch = function(input, init) {
  if (typeof input === 'string' && (input.includes('.m3u8') || input.includes('.mpd'))) {
    setTimeout(() => {
      sendMediaToBackground([{
        url: input,
        type: 'fetch-detected',
        element: 'fetch'
      }]);
    }, 0);
  }
  return originalFetch.apply(this, arguments);
};
