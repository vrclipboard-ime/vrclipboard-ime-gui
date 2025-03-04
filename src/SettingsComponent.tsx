import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ChevronDown, Settings, Save, AlertCircle, Check } from 'lucide-react';

interface Config {
  prefix: string;
  split: string;
  command: string;
  ignore_prefix: boolean;
  on_copy_mode: OnCopyMode;
  skip_url: boolean; 
  use_tsf_reconvert: boolean;
  skip_on_out_of_vrc: boolean;
}

enum OnCopyMode {
  ReturnToClipboard = 'ReturnToClipboard',
  ReturnToChatbox = 'ReturnToChatbox',
  SendDirectly = 'SendDirectly'
}

interface InputFieldProps {
  name: string;
  label: string;
  value: string;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
  disabled?: boolean;
  description?: string;
}

const InputField: React.FC<InputFieldProps> = ({ name, label, value, onChange, disabled, description }) => (
  <div className="mb-3">
    <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1 transition-colors">
      {label}
    </label>
    <input
      type="text"
      name={name}
      value={value}
      onChange={onChange}
      className={`w-full p-1.5 text-sm border rounded focus:border-indigo-400 outline-none transition-colors ${
        disabled 
          ? 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 cursor-not-allowed' 
          : 'bg-white dark:bg-gray-700 dark:text-gray-200 dark:border-gray-600'
      }`}
      disabled={disabled}
    />
    {description && (
      <p className="mt-0.5 text-xs text-gray-500 dark:text-gray-400 transition-colors">{description}</p>
    )}
  </div>
);

interface CheckboxFieldProps {
  id: string;
  name: string;
  label: React.ReactNode;
  checked: boolean;
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

const CheckboxField: React.FC<CheckboxFieldProps> = ({ id, name, label, checked, onChange }) => (
  <div className="mb-2">
    <div className="flex items-start">
      <div className="flex items-center h-4">
        <input
          type="checkbox"
          id={id}
          name={name}
          checked={checked}
          onChange={onChange}
          className="h-3.5 w-3.5 text-indigo-500 dark:text-indigo-400 border-gray-300 dark:border-gray-600 rounded dark:bg-gray-700 transition-colors"
        />
      </div>
      <div className="ml-2 text-xs">
        <label htmlFor={id} className="text-gray-700 dark:text-gray-300 transition-colors">
          {label}
        </label>
      </div>
    </div>
  </div>
);

const SettingsComponent: React.FC = () => {
  const [settings, setSettings] = useState<Config>({
    prefix: ';',
    split: '/',
    command: ';',
    ignore_prefix: false,
    on_copy_mode: OnCopyMode.ReturnToChatbox,
    skip_url: true,
    use_tsf_reconvert: false,
    skip_on_out_of_vrc: true,
  });
  const [isOpen, setIsOpen] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'success' | 'error'>('idle');
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadSettings();
    document.addEventListener('mousedown', handleClickOutside);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, []);

  const loadSettings = async () => {
    try {
      const loadedSettings: Config = await invoke('load_settings');
      setSettings(loadedSettings);
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  };

  const saveSettings = async (newSettings: Config) => {
    setSaveStatus('saving');
    try {
      await invoke('save_settings', { config: newSettings });
      setSaveStatus('success');
      setTimeout(() => setSaveStatus('idle'), 2000);
    } catch (error) {
      console.error('Failed to save settings:', error);
      setSaveStatus('error');
      setTimeout(() => setSaveStatus('idle'), 3000);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const { name, value, type, checked } = e.target;
    const newSettings = {
      ...settings,
      [name]: type === 'checkbox' ? checked : value
    };
    setSettings(newSettings);
    saveSettings(newSettings);
  };

  const handleSelectChange = (value: OnCopyMode) => {
    const newSettings = { ...settings, on_copy_mode: value };
    setSettings(newSettings);
    setIsOpen(false);
    saveSettings(newSettings);
  };

  const handleClickOutside = (event: MouseEvent) => {
    if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
      setIsOpen(false);
    }
  };

  const getOnCopyModeLabel = (mode: OnCopyMode) => {
    switch (mode) {
      case OnCopyMode.ReturnToClipboard:
        return 'クリップボードへ送信';
      case OnCopyMode.ReturnToChatbox:
        return 'チャットボックスへ送信';
      case OnCopyMode.SendDirectly:
        return '直接チャットへ送信';
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

  return (
    <div className="h-full">
      <div className="flex items-center justify-between mb-2">
        <h2 className="text-base font-medium text-gray-700 dark:text-gray-200 flex items-center transition-colors">
          <Settings size={16} className="mr-1.5" />
          設定
        </h2>
        <SaveStatusIndicator />
      </div>

      <div className="bg-white dark:bg-gray-800 rounded border border-gray-100 dark:border-gray-700 p-3 transition-colors">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <h3 className="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300 border-b dark:border-gray-700 pb-1 transition-colors">基本設定</h3>
            
            <InputField
              name="split"
              label="区切り文字"
              value={settings.split}
              onChange={handleChange}
              description="複数の変換モードを使いたい場合の区切り文字"
            />
            
            <InputField
              name="command"
              label="モード変更文字"
              value={settings.command}
              onChange={handleChange}
              description="変換モードを変更するための文字"
            />
            
            <CheckboxField
              id="ignore_prefix"
              name="ignore_prefix"
              label="無条件で変換"
              checked={settings.ignore_prefix}
              onChange={handleChange}
            />
            
            <InputField
              name="prefix"
              label="開始文字"
              value={settings.prefix}
              onChange={handleChange}
              disabled={settings.ignore_prefix}
              description="変換を開始する文字（無条件で変換がオンの場合は無効）"
            />
            
            <div className="relative mb-3" ref={dropdownRef}>
              <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1 transition-colors">
                コピー時の動作
              </label>
              <div
                className="w-full p-1.5 text-sm border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200 flex justify-between items-center cursor-pointer hover:border-indigo-300 dark:hover:border-indigo-500 transition-colors"
                onClick={() => setIsOpen(!isOpen)}
              >
                <span>{getOnCopyModeLabel(settings.on_copy_mode)}</span>
                <ChevronDown size={14} className={`transition-transform ${isOpen ? 'transform rotate-180' : ''}`} />
              </div>
              {isOpen && (
                <div className="absolute z-10 mt-0.5 w-full bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded shadow-sm overflow-hidden text-sm transition-colors">
                  {Object.values(OnCopyMode).map((mode) => (
                    <div
                      key={mode}
                      className="p-1.5 hover:bg-indigo-50 dark:hover:bg-indigo-900/50 cursor-pointer"
                      onClick={() => handleSelectChange(mode)}
                    >
                      <div className={`flex items-center ${settings.on_copy_mode === mode ? 'text-indigo-600 dark:text-indigo-400 font-medium' : 'dark:text-gray-300'}`}>
                        {settings.on_copy_mode === mode && <Check size={12} className="mr-1.5" />}
                        <span className={settings.on_copy_mode === mode ? 'ml-0' : 'ml-4'}>
                          {getOnCopyModeLabel(mode)}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
          
          <div>
            <h3 className="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300 border-b dark:border-gray-700 pb-1 transition-colors">詳細設定</h3>
            
            <CheckboxField
              id="skip_url"
              name="skip_url"
              label="URL が含まれている文章をスキップ"
              checked={settings.skip_url}
              onChange={handleChange}
            />
            
            <CheckboxField
              id="skip_on_out_of_vrc"
              name="skip_on_out_of_vrc"
              label="VRChat以外からのコピーをスキップ"
              checked={settings.skip_on_out_of_vrc}
              onChange={handleChange}
            />
            
            <div className="mb-3 p-2 bg-gray-50 dark:bg-gray-700 rounded border border-gray-200 dark:border-gray-600 text-xs transition-colors">
              <CheckboxField
                id="use_tsf_reconvert"
                name="use_tsf_reconvert"
                label={
                  <span>
                    <span className="text-indigo-600 dark:text-indigo-400 font-medium transition-colors">ベータ機能:</span> TSF再変換を使用
                  </span>
                }
                checked={settings.use_tsf_reconvert}
                onChange={handleChange}
              />
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 ml-5 transition-colors">
                Windows10/11では「以前のバージョンの Microsoft IME を使う」を有効化する必要があります。有効にすると区切り、モード変更、開始文字が無効化されます。
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default SettingsComponent;