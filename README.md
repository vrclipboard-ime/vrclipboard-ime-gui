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

## テスト

```powershell
cargo check --package vrclipboard-ime-gui
cargo test --package vrclipboard-ime-gui
```

## 配布用フォルダー

```powershell
.\package-gpui.ps1
```

`dist-gpui`に実行ファイル、AzooKey DLL、モデルがまとめられます。
