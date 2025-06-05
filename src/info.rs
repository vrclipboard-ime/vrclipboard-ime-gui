use leptos::{prelude::*, task::spawn_local};
use lucide_leptos::{Coffee, ExternalLink, Github};
use serde::{Deserialize, Serialize};

use crate::invoke;

#[derive(Serialize, Deserialize)]
struct OpenArgs<'a> {
    url: &'a str,
}

fn open(url: &'static str) {
    spawn_local(async move {
        invoke(
            "open",
            serde_wasm_bindgen::to_value(&OpenArgs { url }).unwrap(),
        )
        .await;
    });
}

#[component]
pub fn Info() -> impl IntoView {
    view! {
        <div class="h-full">
            <h2 class="text-base font-medium mb-4 text-gray-700 dark:text-gray-200 flex items-center transition-colors">
                <div class="mr-1.5">
                    <Coffee size=16 />
                </div>
                アプリケーション情報
            </h2>

            <div class="bg-white dark:bg-gray-800 rounded border border-gray-100 dark:border-gray-700 p-4 transition-colors">
                <div class="flex flex-col sm:flex-row items-start sm:items-center mb-4">
                    <div class="font-semibold text-lg text-indigo-600 dark:text-indigo-400 mr-3">
                        VRClipboard-IME
                    </div>
                    <div class="bg-indigo-100 dark:bg-indigo-900/50 text-indigo-700 dark:text-indigo-300 text-xs py-1 px-2 rounded">
                        v2.0.0
                    </div>
                </div>

                <div class="space-y-4">
                    <div>
                        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 border-b dark:border-gray-700 pb-1">
                            アプリケーション情報
                        </h3>
                        <div class="text-sm">
                            <div class="flex items-center mb-1">
                                <span class="text-gray-700 dark:text-gray-300 w-20">バージョン:</span>
                                <span class="text-gray-600 dark:text-gray-400">2.0.0</span>
                            </div>
                            <div class="flex items-center mb-1">
                            <span class="text-gray-700 dark:text-gray-300 w-20">ライセンス:</span>
                            <span class="text-gray-600 dark:text-gray-400">MIT</span>
                        </div>
                        <div class="flex items-center mb-1">
                            <span class="text-gray-700 dark:text-gray-300 w-20">最終更新:</span>
                            <span class="text-gray-600 dark:text-gray-400">2025年6月5日</span>
                        </div>
                        <div class="flex items-center mb-1">
                            <span class="text-gray-700 dark:text-gray-300 w-20">技術:</span>
                                <span class="text-gray-600 dark:text-gray-400">Tauri, Leptos, Rust</span>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 border-b dark:border-gray-700 pb-1">
                            開発者
                        </h3>
                        <div class="text-sm">
                            <div class="flex items-center mb-1">
                                <span class="text-gray-700 dark:text-gray-300 w-20">作者:</span>
                                <span class="text-gray-600 dark:text-gray-400">mii443</span>
                            </div>
                            <div class="flex items-center mb-1">
                                <span class="text-gray-700 dark:text-gray-300 w-20">VRChat:</span>
                                <span class="text-gray-600 dark:text-gray-400">みー mii</span>
                            </div>
                            <div class="flex items-center mb-1">
                                <span class="text-gray-700 dark:text-gray-300 w-20">GitHub:</span>
                                <button
                                    on:click=move |_| {
                                        open("https://github.com/mii443");
                                    }
                                    class="text-indigo-600 dark:text-indigo-400 hover:underline flex items-center"
                                >
                                    mii443
                                    <div class="ml-1">
                                        <ExternalLink size={12} />
                                    </div>
                                </button>
                            </div>
                        </div>
                    </div>

                    <div>
                        <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 border-b dark:border-gray-700 pb-1">
                            リンク
                        </h3>
                        <div class="flex flex-wrap gap-2">
                            <button
                                on:click=move |_| {
                                    open("https://github.com/mii443/vrclipboard-ime-gui");
                                }
                                class="flex items-center text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 px-3 py-1.5 rounded text-sm"
                            >
                                <div class="mr-1.5">
                                    <Github size={12} />
                                </div>
                                GitHubリポジトリ
                            </button>
                            <button
                                on:click=move |_| {
                                    open("https://vrime.mii.dev");
                                }
                                class="flex items-center text-gray-600 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 px-3 py-1.5 rounded text-sm"
                            >
                                <div class="mr-1.5">
                                    <ExternalLink size={12} />
                                </div>
                                ウェブサイト
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
