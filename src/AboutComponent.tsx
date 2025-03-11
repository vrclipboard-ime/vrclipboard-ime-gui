import React from 'react';
import { Github, ExternalLink, Coffee } from 'lucide-react';

const AboutComponent: React.FC = () => {
  const openLink = (url: string) => {
    // ブラウザの標準APIを使用してリンクを開く
    window.open(url, '_blank', 'noopener,noreferrer');
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
            v1.10.0
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
                <span className="text-gray-600 dark:text-gray-400">1.11.0</span>
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
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default AboutComponent;