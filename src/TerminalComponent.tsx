import React, { useState, useEffect, useRef } from 'react';
import { Check, Copy, Trash, Download } from 'lucide-react';
import { useLogs } from './LogContext';

const TerminalComponent: React.FC = () => {
  const { logs, clearLogs } = useLogs();
  const [filter, setFilter] = useState<string>('');
  const [autoScroll, setAutoScroll] = useState<boolean>(true);
  const [copied, setCopied] = useState<boolean>(false);
  const terminalRef = useRef<HTMLDivElement>(null);

  // 自動スクロール効果
  useEffect(() => {
    // コンポーネントがマウントされたとき、または新しいログが追加されたときに自動スクロール
    if (autoScroll && terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [logs, filter, autoScroll]);
  
  // コンポーネントがマウントされた時に一度だけ実行
  useEffect(() => {
    // 初期スクロール位置を設定
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, []);

  const getLogColor = (level: string): string => {
    switch (level) {
      case 'ERROR': return 'text-red-500 dark:text-red-400';
      case 'WARN': return 'text-yellow-600 dark:text-yellow-400';
      case 'INFO': return 'text-blue-500 dark:text-blue-400';
      case 'DEBUG': return 'text-purple-500 dark:text-purple-400';
      case 'TRACE': return 'text-gray-500 dark:text-gray-400';
      default: return 'text-gray-700 dark:text-gray-300';
    }
  };

  const handleCopyLogs = async () => {
    const logText = filteredLogs
      .map(log => `[${log.timestamp} ${log.module_path}] [${log.level.toUpperCase()}] ${log.message}`)
      .join('\n');
    
    try {
      await navigator.clipboard.writeText(logText);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error('Failed to copy logs:', error);
    }
  };

  const handleSaveLogs = () => {
    const logText = filteredLogs
      .map(log => `[${log.timestamp} ${log.module_path}] [${log.level.toUpperCase()}] ${log.message}`)
      .join('\n');
    
    const blob = new Blob([logText], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'vrclipboard-ime-debug.log';
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  };

  const filteredLogs = logs.filter(
    log => filter === '' || 
    log.message.toLowerCase().includes(filter.toLowerCase()) || 
    log.level.toLowerCase().includes(filter.toLowerCase())
  );

  return (
    <div className="h-full">
      <h2 className="text-base font-medium mb-2 text-gray-700 dark:text-gray-200 flex items-center justify-between">
        <span>デバッグログ</span>
        <div className="flex items-center space-x-2">
          <button 
            onClick={handleCopyLogs}
            className="flex items-center text-xs text-gray-600 dark:text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded"
            title="ログをコピー"
          >
            {copied ? <Check size={12} className="mr-1" /> : <Copy size={12} className="mr-1" />}
            {copied ? 'コピー完了' : 'コピー'}
          </button>
          <button 
            onClick={clearLogs}
            className="flex items-center text-xs text-gray-600 dark:text-gray-400 hover:text-red-600 dark:hover:text-red-400 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded"
            title="ログをクリア"
          >
            <Trash size={12} className="mr-1" />
            クリア
          </button>
          <button 
            onClick={handleSaveLogs}
            className="flex items-center text-xs text-gray-600 dark:text-gray-400 hover:text-green-600 dark:hover:text-green-400 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded"
            title="ログを保存"
          >
            <Download size={12} className="mr-1" />
            保存
          </button>
        </div>
      </h2>

      <div className="flex items-center mb-2 space-x-2">
        <input
          type="text"
          placeholder="ログをフィルタ..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="text-xs p-1.5 w-full border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200"
        />
        <div className="flex items-center">
          <input
            type="checkbox"
            id="auto-scroll"
            checked={autoScroll}
            onChange={() => setAutoScroll(!autoScroll)}
            className="mr-1 h-3 w-3"
          />
          <label htmlFor="auto-scroll" className="text-xs text-gray-600 dark:text-gray-400">自動スクロール</label>
        </div>
      </div>

      <div 
        ref={terminalRef}
        className="font-mono text-xs bg-black dark:bg-gray-900 text-green-400 p-2 rounded border border-gray-700 h-[calc(100vh-150px)] overflow-y-auto"
      >
        {filteredLogs.length > 0 ? (
          filteredLogs.map((log, index) => (
            <div key={index} className={`mb-1 ${getLogColor(log.level)}`}>
              <span className="text-gray-500 dark:text-gray-400">[{log.timestamp} {log.module_path}]</span>{' '}
              <span className="font-semibold">[{log.level.toUpperCase()}]</span>{' '}
              <span>{log.message}</span>
            </div>
          ))
        ) : (
          <div className="text-gray-500 dark:text-gray-400 italic">ログはありません</div>
        )}
      </div>
    </div>
  );
};

export default TerminalComponent;