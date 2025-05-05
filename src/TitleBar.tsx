import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { listen } from '@tauri-apps/api/event';
import { X, Minus, Square, Maximize2, Moon, Sun } from 'lucide-react';
import { useState, useEffect } from 'react';
import { useTheme } from './ThemeContext';

const appWindow = getCurrentWebviewWindow();

const TitleBar = () => {
  const [isMaximized, setIsMaximized] = useState(false);
  const { theme, toggleTheme } = useTheme();

  useEffect(() => {
    const checkMaximized = async () => {
      try {
        const maximized = await appWindow.isMaximized();
        setIsMaximized(maximized);
      } catch (error) {
        console.error('Failed to check if window is maximized:', error);
      }
    };

    checkMaximized();

    // Tauriのイベントリスナーを設定
    let unlistenMaximize: (() => void) | undefined;
    let unlistenRestore: (() => void) | undefined;

    const setupListeners = async () => {
      try {
        // ウィンドウ最大化イベントをリッスン
        unlistenMaximize = await listen('tauri://resize', () => {
          checkMaximized();
        });

        // ウィンドウ復元イベントもリッスン
        unlistenRestore = await listen('tauri://move', () => {
          checkMaximized();
        });
      } catch (error) {
        console.error('Failed to set up event listeners:', error);
      }
    };

    setupListeners();

    return () => {
      // クリーンアップ
      if (unlistenMaximize) unlistenMaximize();
      if (unlistenRestore) unlistenRestore();
    };
  }, []);

  const handleClose = () => appWindow.close();
  const handleMinimize = () => appWindow.minimize();
  const handleToggleMaximize = () => appWindow.toggleMaximize();

  const TitleButton = ({
    onClick,
    icon,
    hoverClass = "hover:bg-gray-700"
  }: {
    onClick: () => void,
    icon: React.ReactNode,
    hoverClass?: string
  }) => (
    <button
      onClick={onClick}
      className={`p-1 rounded focus:outline-none transition-colors ${hoverClass}`}
    >
      {icon}
    </button>
  );

  return (
    <div
      className="flex justify-between items-center bg-indigo-600 dark:bg-indigo-900 text-white h-8 px-2 transition-colors"
      data-tauri-drag-region
    >
      <div className="flex items-center" data-tauri-drag-region>
        <span className="text-xs font-medium" data-tauri-drag-region>VRClipboard-IME</span>
        <span className="text-xs opacity-80 ml-1" data-tauri-drag-region>v1.12.0</span>
      </div>
      <div className="flex">
        <TitleButton
          onClick={toggleTheme}
          icon={theme === 'dark' ?
            <Sun size={12} className="text-white/90" /> :
            <Moon size={12} className="text-white/90" />
          }
        />
        <TitleButton
          onClick={handleMinimize}
          icon={<Minus size={12} className="text-white/90" />}
        />
        <TitleButton
          onClick={handleToggleMaximize}
          icon={isMaximized ?
            <Square size={12} className="text-white/90" /> :
            <Maximize2 size={12} className="text-white/90" />
          }
        />
        <TitleButton
          onClick={handleClose}
          icon={<X size={14} className="text-white/90" />}
          hoverClass="hover:bg-red-600"
        />
      </div>
    </div>
  );
};

export default TitleBar;