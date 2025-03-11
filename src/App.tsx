import { useState, useEffect } from "react";
import { List, Settings, Terminal, Bug, Info, Check, Download, X } from 'lucide-react';
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import "./App.css";
import TitleBar from "./TitleBar";
import SettingsComponent from "./SettingsComponent";
import { ThemeProvider } from "./ThemeContext";
import { LogProvider } from "./LogContext";
import TerminalComponent from "./TerminalComponent";
import AboutComponent from "./AboutComponent";
import TsfSettingsModal from "./TsfSettingsModal";
import { Config } from "./SettingsComponent";
import DictionaryComponent from "./DictionaryComponent";

interface Log {
  time: string;
  original: string;
  converted: string;
}

const AppContent = () => {
  const [logs, setLogs] = useState<Log[]>([]);
  const [activeMenuItem, setActiveMenuItem] = useState('home');
  const [showTsfModal, setShowTsfModal] = useState(false);
  const [currentSettings, setCurrentSettings] = useState<Config | null>(null);
  const [_isTsfAvailable, setIsTsfAvailable] = useState<boolean | null>(null);
  const [showTsfSuccessMessage, setShowTsfSuccessMessage] = useState(false);
  
  // 更新関連のstate
  const [updateAvailable, setUpdateAvailable] = useState(false);
  const [updateDownloading, setUpdateDownloading] = useState(false);
  const [updateProgress, setUpdateProgress] = useState(0);
  const [contentLength, setContentLength] = useState<number>(0);
  const [updateInfo, setUpdateInfo] = useState<{version: string, date: string | undefined, body: string | undefined} | null>(null);

  useEffect(() => {
    const unlisten = listen<Log>('addLog', (event) => {
      setLogs(prevLogs => [{ time: event.payload.time, original: event.payload.original, converted: event.payload.converted }, ...prevLogs]);
    });

    return () => {
      unlisten.then(f => f());
    }
  }, []);

  // TSF設定のチェック
  useEffect(() => {
    const checkTsfSettings = async () => {
      try {
        const settings: Config = await invoke('load_settings');
        setCurrentSettings(settings);
        
        if (settings.use_tsf_reconvert) {
          // TSF設定が有効な場合、TSFが利用可能か確認
          const available: boolean = await invoke('check_tsf_availability_command');
          setIsTsfAvailable(available);
          
          if (!available) {
            setShowTsfModal(true);
          }
        }
      } catch (error) {
        console.error('TSF設定確認エラー:', error);
      }
    };
    
    checkTsfSettings();
  }, []);
  
  // アプリ起動時に更新を自動チェック
  useEffect(() => {
    const checkForUpdates = async () => {
      try {
        const update = await check();
        if (update) {
          console.log(`更新が見つかりました: ${update.version} (${update.date}) - ${update.body}`);
          setUpdateAvailable(true);
          setUpdateInfo({
            version: update.version,
            date: update.date,
            body: update.body
          });
        }
      } catch (error) {
        console.error('更新チェックエラー:', error);
      }
    };
    
    // アプリ起動から少し遅らせて更新チェック
    const timer = setTimeout(() => {
      checkForUpdates();
    }, 3000);
    
    return () => clearTimeout(timer);
  }, []);

  // 設定保存関数
  const saveSettings = async (config: Config) => {
    try {
      await invoke('save_settings', { config });
      setCurrentSettings(config);
    } catch (error) {
      console.error('設定保存エラー:', error);
    }
  };
  
  // 更新処理
  const handleInstallUpdate = async () => {
    try {
      setUpdateDownloading(true);
      setUpdateAvailable(false);
      
      const update = await check();
      if (!update) {
        setUpdateDownloading(false);
        return;
      }
      
      let downloaded = 0;
      
      await update.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            setContentLength(event.data.contentLength);
            console.log(`ダウンロード開始: ${event.data.contentLength} bytes`);
            break;
          case 'Progress':
            downloaded += event.data.chunkLength;
            const progress = contentLength > 0 ? (downloaded / contentLength) * 100 : 0;
            setUpdateProgress(progress);
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
      console.error('更新インストールエラー:', error);
      setUpdateDownloading(false);
    }
  };

  const dismissUpdate = () => {
    setUpdateAvailable(false);
  };

  const renderLogEntry = (log: { time: string; original: string; converted: string }, index: number) => (
    <div key={index} className="mb-2 p-2 bg-white/90 dark:bg-gray-800 rounded border border-gray-100 dark:border-gray-700 text-sm transition-colors">
      <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">{log.time}</div>
      <div className="flex flex-col sm:flex-row sm:items-center gap-1">
        <div className="text-gray-800 dark:text-gray-200 px-1.5 py-0.5 bg-gray-50 dark:bg-gray-700 rounded flex-grow transition-colors">{log.original}</div>
        <div className="text-gray-400 hidden sm:block text-xs">→</div>
        <div className="text-emerald-600 dark:text-emerald-400 px-1.5 py-0.5 bg-emerald-50 dark:bg-emerald-900/30 rounded flex-grow transition-colors">{log.converted}</div>
      </div>
    </div>
  );

  const MenuItem = ({ icon, label, id }: { icon: React.ReactNode, label: string, id: string }) => (
    <button
      onClick={() => setActiveMenuItem(id)}
      className={`flex items-center w-full px-3 py-2 rounded transition-colors ${
        activeMenuItem === id 
          ? 'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300' 
          : 'hover:bg-gray-100 text-gray-600 dark:hover:bg-gray-700 dark:text-gray-300'
      }`}
    >
      <div className="flex items-center justify-center w-5 h-5">
        {icon}
      </div>
      <span className="ml-2 text-sm">{label}</span>
    </button>
  );

  const renderContent = () => {
    switch (activeMenuItem) {
      case 'home':
        return (
          <div className="h-full">
            <h2 className="text-base font-medium mb-2 text-gray-700 dark:text-gray-200 flex items-center transition-colors">
              <Terminal size={16} className="mr-1.5" />
              変換ログ
            </h2>
            <div className="bg-gray-50 dark:bg-gray-800 border border-gray-100 dark:border-gray-700 p-2 rounded h-[calc(100vh-110px)] overflow-y-auto transition-colors">
              {logs.length > 0 ? (
                logs.map((log, index) => renderLogEntry(log, index))
              ) : (
                <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-gray-500">
                  <Terminal size={24} strokeWidth={1.5} />
                  <p className="mt-2 text-sm">ログはまだありません</p>
                  <p className="text-xs">テキストを変換すると、ここに表示されます</p>
                </div>
              )}
            </div>
          </div>
        );
      case 'settings':
        return <SettingsComponent 
          setShowTsfModal={setShowTsfModal} 
          currentSettings={currentSettings}
          onSaveSettings={saveSettings}
        />;
      case 'terminal':
        return <TerminalComponent />;
      case 'about':
        return <AboutComponent />;
      case 'dictionary':
        return <DictionaryComponent />;
      default:
        return null;
    }
  };

  return (
    <div className="h-screen flex flex-col bg-white dark:bg-gray-900 transition-colors">
      <TitleBar />
      <div className="flex flex-1 h-[calc(100vh-32px)] overflow-hidden">
        {/* サイドメニュー */}
        <div className="w-36 py-2 px-1 border-r border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-800 transition-colors flex flex-col h-full">
          <div className="space-y-1 flex-grow">
            <MenuItem icon={<List size={16} />} label="ログ" id="home" />
            <MenuItem icon={<Settings size={16} />} label="設定" id="settings" />
            {/*<MenuItem icon={<Book size={16} />} label="辞書" id="dictionary" />*/}
          </div>
          {/* 下側にデバッグタブを配置 */}
          <div className="pt-2 border-t border-gray-100 dark:border-gray-700">
            <MenuItem icon={<Bug size={16} />} label="デバッグ" id="terminal" />
            <MenuItem icon={<Info size={16} />} label="情報" id="about" />
          </div>
        </div>

        {/* メインコンテンツ */}
        <div className="flex-1 p-3 overflow-y-auto dark:text-gray-200 transition-colors">
          {renderContent()}
        </div>
      </div>
      
      {/* TSF設定モーダル */}
      {currentSettings && (
        <TsfSettingsModal 
          isOpen={showTsfModal}
          onClose={() => setShowTsfModal(false)}
          onSaveSettings={saveSettings}
          currentSettings={currentSettings}
          onTsfEnabled={() => {
            setShowTsfSuccessMessage(true);
            setTimeout(() => setShowTsfSuccessMessage(false), 5000); // 5秒後に非表示
          }}
        />
      )}
      
      {/* TSF有効化成功メッセージ */}
      {showTsfSuccessMessage && (
        <div className="fixed bottom-4 right-4 bg-green-100 dark:bg-green-900 border border-green-200 dark:border-green-800 text-green-800 dark:text-green-200 px-4 py-3 rounded shadow-lg flex items-center">
          <div className="mr-3 text-green-500 dark:text-green-400">
            <Check size={20} />
          </div>
          <div>
            <p className="font-medium text-sm">TSF再変換が有効化されました</p>
            <p className="text-xs text-green-700 dark:text-green-300">文字変換機能が拡張されました</p>
          </div>
        </div>
      )}
      
      {/* 更新通知 */}
      {updateAvailable && !updateDownloading && updateInfo && (
        <div className="fixed bottom-4 right-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-4 rounded shadow-lg flex flex-col max-w-xs z-50">
          <div className="flex justify-between items-start mb-2">
            <h3 className="text-sm font-medium text-gray-800 dark:text-gray-200">アップデートが利用可能です</h3>
            <button onClick={dismissUpdate} className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
              <X size={16} />
            </button>
          </div>
          <div className="text-xs text-gray-600 dark:text-gray-400 mb-3">
            <div className="mb-1">
              <span className="font-medium">バージョン: </span>
              {updateInfo.version}
            </div>
            <div className="mb-2">
              <span className="font-medium">リリース日: </span>
              {updateInfo.date}
            </div>
            <div>
              <span className="font-medium">変更内容: </span>
              <p className="mt-1 bg-gray-50 dark:bg-gray-700 p-2 rounded border border-gray-200 dark:border-gray-600">
                {updateInfo.body}
              </p>
            </div>
          </div>
          <div className="flex space-x-2">
            <button
              onClick={dismissUpdate}
              className="flex-1 text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 py-1.5 rounded text-sm"
            >
              後で
            </button>
            <button
              onClick={handleInstallUpdate}
              className="flex-1 bg-indigo-500 hover:bg-indigo-600 text-white py-1.5 rounded text-sm flex items-center justify-center"
            >
              <Download size={14} className="mr-1.5" />
              更新する
            </button>
          </div>
        </div>
      )}
      
      {/* ダウンロード中ダイアログ */}
      {updateDownloading && (
        <div className="fixed bottom-4 right-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-4 rounded shadow-lg flex flex-col max-w-xs z-50">
          <h3 className="text-sm font-medium text-gray-800 dark:text-gray-200 mb-2">ダウンロード中...</h3>
          <div className="w-full bg-gray-200 dark:bg-gray-700 rounded h-2 overflow-hidden">
            <div 
              className="bg-indigo-500 h-full rounded transition-all duration-300" 
              style={{ width: `${updateProgress}%` }} 
            />
          </div>
          <p className="text-xs text-gray-600 dark:text-gray-400 mt-1 text-right">{Math.round(updateProgress)}%</p>
        </div>
      )}
    </div>
  );
};

function App() {
  return (
    <ThemeProvider>
      <LogProvider>
        <AppContent />
      </LogProvider>
    </ThemeProvider>
  );
}

export default App;