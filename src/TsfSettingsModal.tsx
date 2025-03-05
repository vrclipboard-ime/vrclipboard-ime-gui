import React, { useEffect, useState } from 'react';
import { open } from '@tauri-apps/plugin-shell';
import { AlertCircle, Settings, Check, X, ExternalLink } from 'lucide-react';
import { Config } from './SettingsComponent';
import { invoke } from '@tauri-apps/api/core';

interface TsfSettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSaveSettings: (config: Config) => Promise<void>;
  currentSettings: Config;
  onTsfEnabled?: () => void;
}

const TsfSettingsModal: React.FC<TsfSettingsModalProps> = ({
  isOpen,
  onClose,
  onSaveSettings,
  currentSettings,
  onTsfEnabled
}) => {
  const [checkingStatus, setCheckingStatus] = useState<'idle' | 'checking'>('idle');
  
  // TSFが利用可能かどうかを定期的にチェックする
  useEffect(() => {
    if (!isOpen) return; // モーダルが閉じている場合は処理しない
    
    let intervalId: number | null = null;
    
    const checkTsfAvailability = async () => {
      try {
        const available: boolean = await invoke('check_tsf_availability_command');
        
        if (available) {
          // TSFが利用可能になったら
          const newSettings = { ...currentSettings, use_tsf_reconvert: true };
          await onSaveSettings(newSettings);
          
          // 成功コールバックを呼び出し
          if (onTsfEnabled) {
            onTsfEnabled();
          }
          
          // モーダルを閉じる
          onClose();
          
          // インターバルをクリア
          if (intervalId !== null) {
            clearInterval(intervalId);
          }
        }
      } catch (error) {
        console.error('TSF利用可能性チェックに失敗しました:', error);
      }
    };
    
    // 初回チェック
    checkTsfAvailability();
    
    // 1秒ごとにチェック（標準的なsetIntervalを使用）
    intervalId = window.setInterval(checkTsfAvailability, 1000);
    
    // クリーンアップ関数
    return () => {
      if (intervalId !== null) {
        clearInterval(intervalId);
      }
    };
  }, [isOpen, currentSettings, onSaveSettings, onClose, onTsfEnabled]);

  if (!isOpen) return null;

  const openWindowsSettings = async () => {
    invoke('open_ms_settings_regionlanguage_jpnime');
  };

  const useLegacyConverter = async () => {
    try {
      const newSettings = { ...currentSettings, use_tsf_reconvert: false };
      await onSaveSettings(newSettings);
      onClose();
    } catch (error) {
      console.error('設定保存に失敗しました:', error);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto transition-colors">
        <div className="flex justify-between items-start mb-4">
          <h2 className="text-lg font-medium text-gray-800 dark:text-gray-200 flex items-center">
            <AlertCircle size={18} className="mr-2 text-yellow-500" />
            Windowsの設定が必要です
          </h2>
          <button onClick={onClose} className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200">
            <X size={20} />
          </button>
        </div>
        
        <div className="flex flex-col sm:flex-row gap-3 justify-start mb-4">
          <button
            onClick={openWindowsSettings}
            className="flex items-center justify-center bg-indigo-500 hover:bg-indigo-600 text-white px-4 py-2 rounded"
          >
            <Settings size={16} className="mr-2" />
            Windows設定を開く
          </button>
        </div>

        <div className="text-gray-700 dark:text-gray-300 mb-4">
          <div className="bg-gray-50 dark:bg-gray-700 p-4 rounded mb-4">
            <h3 className="text-sm font-medium mb-2 flex items-center">
              <Settings size={14} className="mr-1.5" />
              設定手順
            </h3>
            <ol className="list-decimal list-inside text-sm space-y-1.5 ml-1">
              <li>上のボタンからWindows設定を開く</li>
              <li>「全般」を開く</li>
              <li>一番下の「以前のバージョンの Microsoft IME を使う」をオンにする</li>
            </ol>
          </div>
        </div>
        
        <div className="bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg mb-6 overflow-hidden">
          <div className="p-2 bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-600 text-sm font-medium text-gray-700 dark:text-gray-300">
            Windows設定画面
          </div>
          <div className="p-4 flex justify-center">
            <img src="./windows_settings.png"/>
          </div>
        </div>
        
        <div className="flex flex-col sm:flex-row gap-3 justify-end">
          <button
            onClick={useLegacyConverter}
            className="flex items-center justify-center bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 px-4 py-2 rounded"
          >
            <Check size={16} className="mr-2" />
            レガシー変換を使用
            <span className="text-xs text-red-500 ml-1.5">(機能制限あり)</span>
          </button>
        </div>
        
        <div className="mt-4 text-xs text-gray-500 dark:text-gray-400 text-center">
          設定が完了すると、このウィンドウは自動的に閉じます
        </div>
      </div>
    </div>
  );
};

export default TsfSettingsModal;