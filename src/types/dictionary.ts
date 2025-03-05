export enum ConversionMethod {
    Replace = 'Replace',
    None = 'None',
    Converter = 'Converter'
}

export interface DictionaryEntry {
    input: string;
    method: ConversionMethod;
    output?: string;
    use_regex: boolean;
    priority: number;
    converter_char?: string;
}

export interface RustDictionaryEntry {
    input: string;
    method: string | { Converter: string };
    output?: string;
    use_regex: boolean;
    priority: number;
}

export interface RustDictionary {
    entries: RustDictionaryEntry[];
}

export interface Dictionary {
    entries: DictionaryEntry[];
  }

export interface ConverterInfo {
  id: string;
  name: string;
  description: string;
}

export const availableConverters: ConverterInfo[] = [
  { id: 'r', name: 'ローマ字→漢字', description: 'ローマ字を漢字に変換します' },
  { id: 'h', name: 'ひらがな変換', description: '入力をひらがなに変換します' },
  { id: 'k', name: 'カタカナ変換', description: '入力をカタカナに変換します' },
  { id: 'c', name: '計算', description: '数式を計算します' },
  { id: 'n', name: '無変換', description: '入力をそのまま出力します' },
];

export function getConverterInfo(id: string): ConverterInfo | undefined {
  return availableConverters.find(converter => converter.id === id);
}

export function getDefaultDictionaryEntry(): DictionaryEntry {
  return {
    input: '',
    method: ConversionMethod.Replace,
    output: '',
    use_regex: false,
    priority: 0
  };
}

export function convertToRustEntry(entry: DictionaryEntry): RustDictionaryEntry {
    let method: string | { Converter: string };

    if (entry.method === ConversionMethod.Converter && entry.converter_char) {
        method = { Converter: entry.converter_char };
    } else {
        method = entry.method;
    }

    return {
        input: entry.input,
        method: method,
        output: entry.method === ConversionMethod.Replace ? entry.output : undefined,
        use_regex: entry.use_regex,
        priority: entry.priority
    };
}

export function convertFromRustEntry(entry: any): DictionaryEntry {
    let method: ConversionMethod;
    let converter_char: string | undefined;

    if (typeof entry.method === 'object' && entry.method.Converter) {
        method = ConversionMethod.Converter;
        converter_char = entry.method.Converter;
    } else {
        method = entry.method as ConversionMethod;
    }

    return {
        input: entry.input,
        method: method,
        output: entry.output,
        use_regex: entry.use_regex,
        priority: entry.priority,
        converter_char: converter_char
    };
}