# VRClipboard-IME

VRChatのChatBoxで日本語入力を支援するWindowsアプリです。UIはRustのGPUIで実装されています。

## 開発

必要なもの:

- Windows 10/11
- 最新のstable Rust
- Visual Studio Build Tools（Desktop development with C++）

```powershell
cargo run --package vrclipboard-ime-gui
```

AzooKeyのネイティブリソースを更新する場合:

```powershell
.\prepare-azookey.ps1
```

ネイティブ更新にはCMakeも必要です。上記スクリプトは固定したllama.cppリビジョンに`native-patches/ggml-windows-utf8.patch`を適用し、`ggml.dll`を再ビルドします。外部のAzooKeyソースは変更しません。`-SkipBuild`でもこの修正DLLのビルドは行います。DLLだけを更新する場合:

```powershell
.\build-portable-ggml.ps1 -LlamaSourceRoot ..\azookey-kkc-rs\vendor\llama.cpp
```

## チャット入力

### チャットボックスへの貼り付け

「コピー時の動作」を「チャットボックスへ貼り付け」に設定し、標準チャット入力欄にローマ字を入力してコピーすると、変換結果で入力欄を置き換えます。Ctrl+AとCtrl+Vの押下時間と間隔はそれぞれ20msで、前面時の固定待機は合計60msです。背面時はCtrlの認識を待つ20msを加え、合計80msです。チャット送信は手動です。

VRChatが前面ならSendInputを使用します。VRモードなどで背面にある場合は、VRChat.exeの可視・非所有トップレベルウィンドウを検索し、左CtrlだけをSendInputで押下します。GetAsyncKeyStateで押下を確認してからA/VをPostMessageで対象ウィンドウへ送り、選択から貼り付けまでCtrlを保持します。エラー時にもCtrlの解除を試み、解除失敗時は再試行します。ウィンドウを前面に切り替える処理はありません。短いCtrl押下中は前面アプリにもCtrlの状態が見えるため、その間のユーザー操作と干渉する可能性があります。

Windowsではクリップボードの更新番号で通知の重複とアプリ自身の書き込みを除外します。変換結果をコピーし直すと、文字列が同じでも再変換できます。

コピー後もVRC内でチャット入力欄が選択されている必要があります。対象ウィンドウが複数ある、変換中にクリップボードや対象が変わる、前面方式で前面ウィンドウが変わる場合は貼り付けを中止します。VRC内の入力欄の選択状態は判定できません。クリップボード所有者が判明する場合はVRChat.exeからのコピーを許可し、それ以外は設定に従って除外します。所有者がないコピーも処理するため、コピー元を完全には識別できません。「直接チャットへ送信」はOSCを使用します。

## 自動チェック

```powershell
cargo check --package vrclipboard-ime-gui
cargo test --package vrclipboard-ime-gui
```

## 配布用フォルダー

```powershell
.\package-gpui.ps1
```

`dist-gpui`に実行ファイル、AzooKey DLL、モデルがまとめられます。

AzooKeyのSwiftリソースは実行ファイルと同じ階層に必要です。配布スクリプトは3つの`*.resources`フォルダーをその階層と`azookey-native`内に同梱します。初期化前にも配置を検査し、旧配置や`cargo run`では不足するリソースを`azookey-native`からコピーします。不完全な配布はネイティブ処理の実行前にエラーとして報告します。実行ファイルだけを取り出さず、配布フォルダー全体を使用してください。

同梱の`ggml.dll`にはWindowsのUTF-8パス修正を適用し、日本語・空白を含む配布先とユーザーデータ先でAzooKeyの変換・再変換を確認しています。必要なランタイムも同梱しています。

UIやクリップボードを操作せず、AzooKeyの変換・再変換を確認する場合:

```powershell
.\dist-gpui\vrclipboard-ime-gui.exe --check-azookey
```

配布ZIPを日本語・空白を含むフォルダーへ展開し、開発環境のPATHを使わずに変換・再変換を検証する場合:

```powershell
.\test-azookey-package.ps1 -ZipPath .\target\release\配布ファイル.zip
```

通常のCPU自動選択に加え、同じAzooKeyの汎用CPUバックエンドと旧辞書配置の自動修復も確認します。検証用コピーとログは`target/azookey-package-check`に残します。
