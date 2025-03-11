import React, { useState } from 'react';
import { Github, ExternalLink, Coffee, RefreshCw, Check, AlertCircle, Download, X } from 'lucide-react';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

const AboutComponent: React.FC = () => {
  const [updateStatus, setUpdateStatus] = useState<'idle' | 'checking' | 'available' | 'downloading' | 'notAvailable' | 'error'>('idle');
  const [downloadProgress, setDownloadProgress] = useState<number>(0);
  const [contentLength, setContentLength] = useState<number>(0);
  const [updateInfo, setUpdateInfo] = useState<{version: string, date: string | undefined, body: string | undefined} | null>(null);
  const [showUpdateDialog, setShowUpdateDialog] = useState(false);

  const openLink = (url: string) => {
    window.open(url, '_blank', 'noopener,noreferrer');
  };

  const checkForUpdates = async () => {
    try {
      setUpdateStatus('checking');
      const update = await check();
      
      if (update) {
        console.log(`更新が見つかりました: ${update.version} (${update.date}) - ${update.body}`);
        setUpdateStatus('available');
        setUpdateInfo({
          version: update.version,
          date: update.date,
          body: update.body
        });
        setShowUpdateDialog(true);
      } else {
        setUpdateStatus('notAvailable');
        setTimeout(() => setUpdateStatus('idle'), 3000);
      }
    } catch (error) {
      console.error('Update check error:', error);
      setUpdateStatus('error');
      setTimeout(() => setUpdateStatus('idle'), 3000);
    }
  };

  const handleInstallUpdate = async () => {
    try {
      setUpdateStatus('downloading');
      const update = await check();
      if (!update) {
        setUpdateStatus('error');
        setTimeout(() => setUpdateStatus('idle'), 3000);
        return;
      }
      
      let downloaded = 0;
      
      // 更新をダウンロードしてインストール
      await update.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            setContentLength(event.data.contentLength);
            console.log(`ダウンロード開始: ${event.data.contentLength} bytes`);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            const progress = (downloaded / contentLength) * 100;
            setDownloadProgress(progress);
            console.log(`ダウンロード中: ${downloaded} / ${contentLength} (${progress.toFixed(1)}%)`);
            break;
          case 'Finished':
            console.log('ダウンロード完了');
            break;
        }
      });
      
      console.log('更新がインストールされました');
      // アプリを再起動
      await relaunch();
    } catch (error) {
      console.error('Update installation error:', error);
      setUpdateStatus('error');
      setTimeout(() => setUpdateStatus('idle'), 3000);
    }
  };

  const getUpdateButtonContent = () => {
    switch (updateStatus) {
      case 'idle':
        return (
          <>
            <RefreshCw size={14} className="mr-1.5" />
            更新を確認
          </>
        );
      case 'checking':
        return (
          <>
            <RefreshCw size={14} className="mr-1.5 animate-spin" />
            確認中...
          </>
        );
      case 'available':
        return (
          <>
            <Download size={14} className="mr-1.5" />
            更新が見つかりました
          </>
        );
      case 'downloading':
        return (
          <>
            <Download size={14} className="mr-1.5 animate-pulse" />
            ダウンロード中... {Math.round(downloadProgress)}%
          </>
        );
      case 'notAvailable':
        return (
          <>
            <Check size={14} className="mr-1.5" />
            最新バージョンです
          </>
        );
      case 'error':
        return (
          <>
            <AlertCircle size={14} className="mr-1.5" />
            エラーが発生しました
          </>
        );
    }
  };

  // 更新ダイアログの表示
  const UpdateDialog = () => {
    if (!showUpdateDialog || !updateInfo) return null;

    return (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-5 max-w-md w-full">
          <div className="flex justify-between items-start mb-4">
            <h3 className="text-lg font-medium text-gray-800 dark:text-gray-200">
              アップデートが利用可能です
            </h3>
            <button 
              onClick={() => setShowUpdateDialog(false)} 
              className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              <X size={18} />
            </button>
          </div>
          
          <div className="mb-4">
            <div className="flex items-center mb-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300 w-20">バージョン:</span>
              <span className="text-sm text-gray-600 dark:text-gray-400">{updateInfo.version}</span>
            </div>
            <div className="flex items-center mb-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300 w-20">リリース日:</span>
              <span className="text-sm text-gray-600 dark:text-gray-400">{updateInfo.date}</span>
            </div>
            <div className="mb-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300 block mb-1">変更内容:</span>
              <div className="text-sm text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-700 p-2 rounded border border-gray-200 dark:border-gray-600">
                {updateInfo.body}
              </div>
            </div>
          </div>
          
          <div className="flex justify-end space-x-3">
            <button
              onClick={() => setShowUpdateDialog(false)}
              className="px-4 py-2 bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 rounded hover:bg-gray-300 dark:hover:bg-gray-600 text-sm"
            >
              後で
            </button>
            <button
              onClick={() => {
                setShowUpdateDialog(false);
                handleInstallUpdate();
              }}
              className="px-4 py-2 bg-indigo-500 text-white rounded hover:bg-indigo-600 text-sm flex items-center"
            >
              <Download size={14} className="mr-1.5" />
              今すぐ更新
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="h-full">
      <h2 className="text-base font-medium mb-4 text-gray-700 dark:text-gray-200 flex items-center transition-colors">
        <Coffee size={16} className="mr-1.5" />
        アプリケーション情報
      </h2>

      <div className="bg-white dark:bg-gray-800 rounded border border-gray-100 dark:border-gray-700 p-4 transition-colors">
        <div className="flex flex-col sm:flex-row items-start sm:items-center mb-4">
          <div className="font-semibold text-lg text-indigo-600 dark:text-indigo-400 mr-3">VRClipboard-IME</div>
          <div className="bg-indigo-100 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300 text-xs py-1 px-2 rounded">
            v1.11.0
          </div>
        </div>

        <div className="space-y-4">
          <div>
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 border-b dark:border-gray-700 pb-1">
              アプリケーション情報
            </h3>
            <div className="text-sm">
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">バージョン:</span>
                <span className="text-gray-600 dark:text-gray-400">1.11.1</span>
              </div>
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">ライセンス:</span>
                <span className="text-gray-600 dark:text-gray-400">MIT</span>
              </div>
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">最終更新:</span>
                <span className="text-gray-600 dark:text-gray-400">2025年3月11日</span>
              </div>
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">技術:</span>
                <span className="text-gray-600 dark:text-gray-400">Tauri, Rust, React, TypeScript</span>
              </div>
            </div>
          </div>

          <div>
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 border-b dark:border-gray-700 pb-1">
              開発者
            </h3>
            <div className="text-sm">
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">作者:</span>
                <span className="text-gray-600 dark:text-gray-400">mii443</span>
              </div>
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">VRChat:</span>
                <span className="text-gray-600 dark:text-gray-400">みー mii</span>
              </div>
              <div className="flex items-center mb-1">
                <span className="text-gray-700 dark:text-gray-300 w-20">GitHub:</span>
                <button 
                  onClick={() => openLink('https://github.com/mii443')}
                  className="text-indigo-600 dark:text-indigo-400 hover:underline flex items-center"
                >
                  mii443
                  <ExternalLink size={12} className="ml-1" />
                </button>
              </div>
            </div>
          </div>

          <div>
            <h3 className="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 border-b dark:border-gray-700 pb-1">
              リンク
            </h3>
            <div className="flex flex-wrap gap-2">
              <button
                onClick={() => openLink('https://github.com/mii443/vrclipboard-ime-gui')}
                className="flex items-center text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 px-3 py-1.5 rounded text-sm"
              >
                <Github size={14} className="mr-1.5" />
                GitHubリポジトリ
              </button>
              <button
                onClick={() => openLink('https://vrime.mii.dev')}
                className="flex items-center text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 px-3 py-1.5 rounded text-sm"
              >
                <ExternalLink size={14} className="mr-1.5" />
                ウェブサイト
              </button>
              <button
                onClick={checkForUpdates}
                disabled={updateStatus === 'checking' || updateStatus === 'downloading'}
                className={`flex items-center text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 px-3 py-1.5 rounded text-sm ${
                  (updateStatus === 'checking' || updateStatus === 'downloading') ? 'opacity-70 cursor-not-allowed' : ''
                }`}
              >
                {getUpdateButtonContent()}
              </button>
            </div>
          </div>
          
          {updateStatus === 'downloading' && (
            <div className="mt-2">
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded h-2 overflow-hidden">
                <div 
                  className="bg-indigo-500 h-full rounded transition-all duration-300" 
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
            </div>
          )}
        </div>
      </div>
      
      {/* 更新ダイアログ */}
      <UpdateDialog />
    </div>
  );
};

export default AboutComponent;