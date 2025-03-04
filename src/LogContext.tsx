import React, { createContext, useState, useContext, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

export interface LogMessage {
  level: 'info' | 'warn' | 'error' | 'debug' | 'trace';
  message: string;
  module_path: string;
  timestamp: string;
}

export interface LogContextType {
  logs: LogMessage[];
  clearLogs: () => void;
}

export const LogContext = createContext<LogContextType | undefined>(undefined);

export const LogProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [logs, setLogs] = useState<LogMessage[]>([]);

  useEffect(() => {
    // グローバルにログイベントを監視
    const setupLogListener = async () => {
      try {
        // log-eventイベントのリスナーを設定
        const unlisten = await listen<LogMessage>('log-event', (event) => {
          setLogs(prev => [...prev, event.payload]);
        });
        
        // クリーンアップ関数を返す
        return () => {
          unlisten();
        };
      } catch (error) {
        console.error('Failed to set up log listener:', error);
        return () => {};
      }
    };

    // リスナーをセットアップし、その結果をPromiseとして保持
    const unlistenPromise = setupLogListener();
    
    // コンポーネントのアンマウント時にリスナーを解除
    return () => {
      unlistenPromise.then(cleanupFn => cleanupFn()).catch(err => {
        console.error('Error cleaning up event listener:', err);
      });
    };
  }, []);

  const clearLogs = () => {
    setLogs([]);
  };

  return (
    <LogContext.Provider value={{ logs, clearLogs }}>
      {children}
    </LogContext.Provider>
  );
};

// カスタムフック
export const useLogs = (): LogContextType => {
  const context = useContext(LogContext);
  if (context === undefined) {
    throw new Error('useLogs must be used within a LogProvider');
  }
  return context;
};