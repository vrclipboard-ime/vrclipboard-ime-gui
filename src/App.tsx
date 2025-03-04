import { useState, useEffect } from "react";
import { List, Settings, Terminal, Bug, Info } from 'lucide-react';
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import TitleBar from "./TitleBar";
import SettingsComponent from "./SettingsComponent";
import { ThemeProvider, useTheme } from "./ThemeContext";
import TerminalComponent from "./TerminalComponent";
import AboutComponent from "./AboutComponent";

interface Log {
  time: string;
  original: string;
  converted: string;
}

const AppContent = () => {
  const { theme } = useTheme();
  const [logs, setLogs] = useState<Log[]>([]);
  const [activeMenuItem, setActiveMenuItem] = useState('home');

  useEffect(() => {
    const unlisten = listen<Log>('addLog', (event) => {
      setLogs(prevLogs => [{ time: event.payload.time, original: event.payload.original, converted: event.payload.converted }, ...prevLogs]);
    });

    return () => {
      unlisten.then(f => f());
    }
  }, []);

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
        return <SettingsComponent />;
      case 'terminal':
        return <TerminalComponent />;
      case 'about':
        return <AboutComponent />;
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
    </div>
  );
};

function App() {
  return (
    <ThemeProvider>
      <AppContent />
    </ThemeProvider>
  );
}

export default App;