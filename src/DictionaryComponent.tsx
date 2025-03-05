import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Book, Save, Plus, Trash, ChevronUp, ChevronDown, AlertCircle, Check, Edit, AlignLeft, Info, X } from 'lucide-react';
import { 
  Dictionary, 
  DictionaryEntry, 
  ConversionMethod, 
  availableConverters, 
  getDefaultDictionaryEntry, 
  getConverterInfo,
  convertToRustEntry,
  convertFromRustEntry
} from './types/dictionary';

const DictionaryComponent: React.FC = () => {
  const [dictionary, setDictionary] = useState<Dictionary>({ entries: [] });
  const [selectedEntry, setSelectedEntry] = useState<DictionaryEntry | null>(null);
  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [isEditing, setIsEditing] = useState(false);
  const [showDialog, setShowDialog] = useState(false);
  const [isMethodDropdownOpen, setIsMethodDropdownOpen] = useState(false);
  const [isConverterDropdownOpen, setIsConverterDropdownOpen] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');

  const methodDropdownRef = useRef<HTMLDivElement>(null);
  const converterDropdownRef = useRef<HTMLDivElement>(null);

  // 辞書データの読み込み
  useEffect(() => {
    loadDictionary();
  }, []);

  // ドロップダウン外のクリックを検知して閉じる
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (methodDropdownRef.current && !methodDropdownRef.current.contains(event.target as Node)) {
        setIsMethodDropdownOpen(false);
      }
      if (converterDropdownRef.current && !converterDropdownRef.current.contains(event.target as Node)) {
        setIsConverterDropdownOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const loadDictionary = async () => {
    try {
      const loadedDictionary: any = await invoke('load_dictionary');
      
      // Rust形式からTypeScript形式に変換
      const entriesWithPriority = loadedDictionary.entries.map((entry: any, index: number) => {
        const convertedEntry = convertFromRustEntry(entry);
        // priorityが設定されていない場合はデフォルト値を設定
        if (convertedEntry.priority === undefined) {
          convertedEntry.priority = index;
        }
        return convertedEntry;
      });
      
      setDictionary({ entries: entriesWithPriority });
    } catch (error) {
      console.error('Failed to load dictionary:', error);
    }
  };

  const saveDictionary = async (dict: Dictionary) => {
    setSaveStatus('saving');
    try {
      // TypeScript形式からRust形式に変換
      const rustDict = {
        entries: dict.entries.map(entry => convertToRustEntry(entry))
      };

      await invoke('save_dictionary', { dictionary: rustDict });
      setSaveStatus('success');
      setTimeout(() => setSaveStatus('idle'), 2000);
    } catch (error) {
      console.error('Failed to save dictionary:', error);
      setSaveStatus('error');
      setTimeout(() => setSaveStatus('idle'), 3000);
    }
  };

  const handleAddEntry = () => {
    setSelectedEntry(getDefaultDictionaryEntry());
    setEditIndex(null);
    setIsEditing(true);
    setShowDialog(true);
  };

  const handleEditEntry = (entry: DictionaryEntry, index: number) => {
    setSelectedEntry({...entry});
    setEditIndex(index);
    setIsEditing(true);
    setShowDialog(true);
  };

  const handleDeleteEntry = (index: number) => {
    const newEntries = [...dictionary.entries];
    newEntries.splice(index, 1);
    const newDict = { ...dictionary, entries: newEntries };
    setDictionary(newDict);
    saveDictionary(newDict);
  };

  const handleSaveEntry = () => {
    if (!selectedEntry) return;
    
    const newEntries = [...dictionary.entries];
    
    // 新規追加の場合
    if (editIndex === null) {
      newEntries.push(selectedEntry);
    } else {
      // 編集の場合
      newEntries[editIndex] = selectedEntry;
    }
    
    const newDict = { ...dictionary, entries: newEntries };
    setDictionary(newDict);
    saveDictionary(newDict);
    setShowDialog(false);
    setIsEditing(false);
    setSelectedEntry(null);
  };

  const handleChangePriority = (index: number, direction: 'up' | 'down') => {
    if (dictionary.entries.length <= 1) return;
    
    const newEntries = [...dictionary.entries];
    const entry = newEntries[index];
    
    if (direction === 'up' && index > 0) {
      const prevEntry = newEntries[index - 1];
      const tempPriority = prevEntry.priority;
      prevEntry.priority = entry.priority;
      entry.priority = tempPriority;
      
      // 実際の配列の順序も変更
      newEntries[index] = prevEntry;
      newEntries[index - 1] = entry;
    } else if (direction === 'down' && index < newEntries.length - 1) {
      const nextEntry = newEntries[index + 1];
      const tempPriority = nextEntry.priority;
      nextEntry.priority = entry.priority;
      entry.priority = tempPriority;
      
      // 実際の配列の順序も変更
      newEntries[index] = nextEntry;
      newEntries[index + 1] = entry;
    }
    
    const newDict = { ...dictionary, entries: newEntries };
    setDictionary(newDict);
    saveDictionary(newDict);
  };

  const handleChangeEntryField = (field: keyof DictionaryEntry, value: any) => {
    if (!selectedEntry) return;
    
    const updatedEntry = { ...selectedEntry, [field]: value };
    
    // 変換方法がReplace以外の場合はoutputをnullに
    if (field === 'method' && value !== ConversionMethod.Replace) {
      updatedEntry.output = undefined;
    }
    
    // 変換方法がConverterの場合はconverter_charをデフォルト値に
    if (field === 'method' && value === ConversionMethod.Converter && !updatedEntry.converter_char) {
      updatedEntry.converter_char = 'r';
    }
    
    setSelectedEntry(updatedEntry);
  };

  const handleSelectMethod = (method: ConversionMethod) => {
    handleChangeEntryField('method', method);
    setIsMethodDropdownOpen(false);
  };

  const handleSelectConverter = (converterId: string) => {
    handleChangeEntryField('converter_char', converterId);
    setIsConverterDropdownOpen(false);
  };

  const getMethodLabel = (method: ConversionMethod, converterChar?: string) => {
    switch(method) {
      case ConversionMethod.Replace:
        return '置き換え';
      case ConversionMethod.None:
        return '無変換';
      case ConversionMethod.Converter:
        if (converterChar) {
          const converter = getConverterInfo(converterChar);
          return converter ? `変換: ${converter.name}` : '変換';
        }
        return '変換';
      default:
        return method;
    }
  };

  const SaveStatusIndicator = () => {
    switch (saveStatus) {
      case 'saving':
        return <span className="flex items-center text-indigo-500 text-xs animate-pulse"><Save size={10} className="mr-0.5" /> 保存中</span>;
      case 'success':
        return <span className="flex items-center text-green-500 text-xs"><Check size={10} className="mr-0.5" /> 保存完了</span>;
      case 'error':
        return <span className="flex items-center text-red-500 text-xs"><AlertCircle size={10} className="mr-0.5" /> 保存失敗</span>;
      default:
        return null;
    }
  };

  // 辞書エントリダイアログ
  const EntryDialog = () => {
    if (!showDialog || !selectedEntry) return null;

    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-4 max-w-xl w-full max-h-[90vh] overflow-y-auto transition-colors">
          <div className="flex justify-between items-start mb-4">
            <h3 className="text-lg font-medium text-gray-800 dark:text-gray-200">
              {editIndex !== null ? '辞書エントリの編集' : '新しい辞書エントリ'}
            </h3>
            <button 
              onClick={() => {
                setShowDialog(false);
                setIsEditing(false);
                setSelectedEntry(null);
                setIsMethodDropdownOpen(false);
                setIsConverterDropdownOpen(false);
              }} 
              className="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
              <X size={20} />
            </button>
          </div>
          
          <div className="space-y-4">
            <div>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                変換対象文字列
              </label>
              <input
                type="text"
                value={selectedEntry.input}
                onChange={(e) => handleChangeEntryField('input', e.target.value)}
                className="w-full p-1.5 text-sm border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200 focus:border-indigo-400 outline-none"
                placeholder="例: こんにちは"
              />
              <div className="flex items-center mt-1.5">
                <input
                  type="checkbox"
                  id="use_regex"
                  checked={selectedEntry.use_regex}
                  onChange={(e) => handleChangeEntryField('use_regex', e.target.checked)}
                  className="h-3.5 w-3.5 text-indigo-500 border-gray-300 dark:border-gray-600 rounded dark:bg-gray-700"
                />
                <label htmlFor="use_regex" className="ml-2 text-xs text-gray-700 dark:text-gray-300">
                  正規表現を使用
                </label>
              </div>
            </div>
            
            <div className="relative mb-3">
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                変換方法
              </label>
              <div
                ref={methodDropdownRef}
                className="w-full p-1.5 text-sm border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200 flex justify-between items-center cursor-pointer hover:border-indigo-300 dark:hover:border-indigo-500 transition-colors"
                onClick={() => setIsMethodDropdownOpen(!isMethodDropdownOpen)}
              >
                <span>{getMethodLabel(selectedEntry.method as ConversionMethod, selectedEntry.converter_char)}</span>
                <ChevronDown size={14} className={`transition-transform ${isMethodDropdownOpen ? 'transform rotate-180' : ''}`} />
              </div>
              {isMethodDropdownOpen && (
                <div className="absolute z-10 mt-0.5 w-full bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded shadow-sm overflow-hidden text-sm transition-colors">
                  <div
                    className="p-1.5 hover:bg-indigo-50 dark:hover:bg-indigo-900/50 cursor-pointer"
                    onClick={() => handleSelectMethod(ConversionMethod.Replace)}
                  >
                    <div className={`flex items-center ${selectedEntry.method === ConversionMethod.Replace ? 'text-indigo-600 dark:text-indigo-400 font-medium' : 'dark:text-gray-300'}`}>
                      {selectedEntry.method === ConversionMethod.Replace && <Check size={12} className="mr-1.5" />}
                      <span className={selectedEntry.method === ConversionMethod.Replace ? 'ml-0' : 'ml-4'}>
                        置き換え
                      </span>
                    </div>
                  </div>
                  <div
                    className="p-1.5 hover:bg-indigo-50 dark:hover:bg-indigo-900/50 cursor-pointer"
                    onClick={() => handleSelectMethod(ConversionMethod.None)}
                  >
                    <div className={`flex items-center ${selectedEntry.method === ConversionMethod.None ? 'text-indigo-600 dark:text-indigo-400 font-medium' : 'dark:text-gray-300'}`}>
                      {selectedEntry.method === ConversionMethod.None && <Check size={12} className="mr-1.5" />}
                      <span className={selectedEntry.method === ConversionMethod.None ? 'ml-0' : 'ml-4'}>
                        無変換
                      </span>
                    </div>
                  </div>
                  <div
                    className="p-1.5 hover:bg-indigo-50 dark:hover:bg-indigo-900/50 cursor-pointer"
                    onClick={() => handleSelectMethod(ConversionMethod.Converter)}
                  >
                    <div className={`flex items-center ${selectedEntry.method === ConversionMethod.Converter ? 'text-indigo-600 dark:text-indigo-400 font-medium' : 'dark:text-gray-300'}`}>
                      {selectedEntry.method === ConversionMethod.Converter && <Check size={12} className="mr-1.5" />}
                      <span className={selectedEntry.method === ConversionMethod.Converter ? 'ml-0' : 'ml-4'}>
                        変換
                      </span>
                    </div>
                  </div>
                </div>
              )}
            </div>
            
            {selectedEntry.method === ConversionMethod.Converter && (
              <div className="relative mb-3">
                <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                  変換器
                </label>
                <div
                  ref={converterDropdownRef}
                  className="w-full p-1.5 text-sm border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200 flex justify-between items-center cursor-pointer hover:border-indigo-300 dark:hover:border-indigo-500 transition-colors"
                  onClick={() => setIsConverterDropdownOpen(!isConverterDropdownOpen)}
                >
                  <span>
                    {selectedEntry.converter_char 
                      ? getConverterInfo(selectedEntry.converter_char)?.name || 'ローマ字→漢字'
                      : 'ローマ字→漢字'}
                  </span>
                  <ChevronDown size={14} className={`transition-transform ${isConverterDropdownOpen ? 'transform rotate-180' : ''}`} />
                </div>
                {isConverterDropdownOpen && (
                  <div className="absolute z-10 mt-0.5 w-full bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded shadow-sm overflow-hidden text-sm transition-colors">
                    {availableConverters.map(converter => (
                      <div
                        key={converter.id}
                        className="p-1.5 hover:bg-indigo-50 dark:hover:bg-indigo-900/50 cursor-pointer"
                        onClick={() => handleSelectConverter(converter.id)}
                      >
                        <div className={`flex items-center ${selectedEntry.converter_char === converter.id ? 'text-indigo-600 dark:text-indigo-400 font-medium' : 'dark:text-gray-300'}`}>
                          {selectedEntry.converter_char === converter.id && <Check size={12} className="mr-1.5" />}
                          <span className={selectedEntry.converter_char === converter.id ? 'ml-0' : 'ml-4'}>
                            {converter.name} - {converter.description}
                          </span>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
            
            {selectedEntry.method === ConversionMethod.Replace && (
              <div>
                <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                  置き換え後の文字列
                </label>
                <input
                  type="text"
                  value={selectedEntry.output || ''}
                  onChange={(e) => handleChangeEntryField('output', e.target.value)}
                  className="w-full p-1.5 text-sm border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200 focus:border-indigo-400 outline-none"
                  placeholder="例: Hello"
                />
              </div>
            )}
            
            <div>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                優先順位
              </label>
              <input
                type="number"
                value={selectedEntry.priority}
                onChange={(e) => handleChangeEntryField('priority', parseInt(e.target.value) || 0)}
                className="w-full p-1.5 text-sm border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200 focus:border-indigo-400 outline-none"
                min="0"
              />
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                数値が大きいほど優先度が高くなります。
              </p>
            </div>
          </div>
          
          <div className="mt-6 flex justify-end">
            <button
              onClick={() => {
                setShowDialog(false);
                setIsEditing(false);
                setSelectedEntry(null);
                setIsMethodDropdownOpen(false);
                setIsConverterDropdownOpen(false);
              }}
              className="mr-2 px-3 py-1.5 rounded text-sm bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200"
            >
              キャンセル
            </button>
            <button
              onClick={handleSaveEntry}
              className="px-3 py-1.5 rounded text-sm bg-indigo-500 hover:bg-indigo-600 text-white dark:bg-indigo-600 dark:hover:bg-indigo-700"
            >
              保存
            </button>
          </div>
        </div>
      </div>
    );
  };

  return (
    <div className="h-full">
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-base font-medium text-gray-700 dark:text-gray-200 flex items-center transition-colors">
          <Book size={16} className="mr-1.5" />
          辞書
        </h2>
        <div className="flex items-center space-x-2">
          <SaveStatusIndicator />
          <button
            onClick={handleAddEntry}
            className="flex items-center text-xs text-white bg-indigo-500 hover:bg-indigo-600 dark:bg-indigo-600 dark:hover:bg-indigo-700 px-2 py-1 rounded"
          >
            <Plus size={12} className="mr-1" />
            新規追加
          </button>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-800 rounded border border-gray-100 dark:border-gray-700 p-3 transition-colors">
        {dictionary.entries.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full text-sm">
              <thead>
                <tr className="bg-gray-50 dark:bg-gray-700 transition-colors">
                  <th className="px-3 py-2 text-left text-xs text-gray-500 dark:text-gray-400 font-medium">優先度</th>
                  <th className="px-3 py-2 text-left text-xs text-gray-500 dark:text-gray-400 font-medium">変換対象</th>
                  <th className="px-3 py-2 text-left text-xs text-gray-500 dark:text-gray-400 font-medium">変換方法</th>
                  <th className="px-3 py-2 text-left text-xs text-gray-500 dark:text-gray-400 font-medium">変換後</th>
                  <th className="px-3 py-2 text-left text-xs text-gray-500 dark:text-gray-400 font-medium">操作</th>
                </tr>
              </thead>
              <tbody>
                {dictionary.entries.map((entry, index) => (
                  <tr key={index} className="border-t border-gray-100 dark:border-gray-700 transition-colors">
                    <td className="px-3 py-2 text-gray-700 dark:text-gray-300 whitespace-nowrap">
                      <div className="flex items-center">
                        <span className="mr-2">{entry.priority}</span>
                        <div className="flex flex-col">
                          <button
                            onClick={() => handleChangePriority(index, 'up')}
                            className="text-gray-500 hover:text-indigo-500 mb-0.5 disabled:opacity-30"
                            disabled={index === 0}
                          >
                            <ChevronUp size={14} />
                          </button>
                          <button
                            onClick={() => handleChangePriority(index, 'down')}
                            className="text-gray-500 hover:text-indigo-500 disabled:opacity-30"
                            disabled={index === dictionary.entries.length - 1}
                          >
                            <ChevronDown size={14} />
                          </button>
                        </div>
                      </div>
                    </td>
                    <td className="px-3 py-2 text-gray-700 dark:text-gray-300">
                      <div className="flex items-center">
                        {entry.use_regex && (
                          <span className="mr-1 text-xs bg-purple-100 dark:bg-purple-900/30 text-purple-600 dark:text-purple-400 px-1 py-0.5 rounded">正規表現</span>
                        )}
                        <span className="font-mono">{entry.input}</span>
                      </div>
                    </td>
                    <td className="px-3 py-2 text-gray-700 dark:text-gray-300">
                      {getMethodLabel(entry.method as ConversionMethod, entry.converter_char)}
                    </td>
                    <td className="px-3 py-2 text-gray-700 dark:text-gray-300 font-mono">
                      {entry.method === ConversionMethod.Replace ? entry.output : '-'}
                    </td>
                    <td className="px-3 py-2">
                      <div className="flex items-center space-x-1">
                        <button
                          onClick={() => handleEditEntry(entry, index)}
                          className="text-indigo-500 hover:text-indigo-600 dark:text-indigo-400 dark:hover:text-indigo-300 p-1"
                          title="編集"
                        >
                          <Edit size={14} />
                        </button>
                        <button
                          onClick={() => handleDeleteEntry(index)}
                          className="text-red-500 hover:text-red-600 dark:text-red-400 dark:hover:text-red-300 p-1"
                          title="削除"
                        >
                          <Trash size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="flex flex-col items-center justify-center py-8 text-gray-500 dark:text-gray-400">
            <AlignLeft size={32} strokeWidth={1.5} className="mb-2" />
            <p className="mb-1">辞書エントリがありません</p>
            <p className="text-xs mb-4">「新規追加」ボタンから辞書エントリを追加してください</p>
            <button
              onClick={handleAddEntry}
              className="flex items-center text-xs bg-indigo-500 hover:bg-indigo-600 text-white px-3 py-1.5 rounded"
            >
              <Plus size={12} className="mr-1.5" />
              新規エントリを追加
            </button>
          </div>
        )}
      </div>

      {/* エントリ編集ダイアログ */}
      <EntryDialog />
    </div>
  );
};

export default DictionaryComponent;