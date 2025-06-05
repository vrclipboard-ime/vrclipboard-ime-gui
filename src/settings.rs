use crate::checkbox::CheckboxField;
use leptos::prelude::*;
use lucide_leptos::Settings as SettingsIcon;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <div class="h-full">
            // ヘッダー
            <div class="text-base font-medium mb-4 text-gray-700 dark:text-gray-200 flex items-center transition-colors">
                <div class="mr-1.5">
                    <SettingsIcon size=16 />
                </div>

                設定
            </div>

            // 設定本体
            <div class="bg-white dark:bg-gray-800 rounded border border-gray-100 dark:border-gray-700 p-3 transition-colors">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <h3 class="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300 border-b dark:border-gray-700 pb-1 transition-colors">基本設定</h3>

                        <CheckboxField label="URL が含まれている文章をスキップ"/>
                        <CheckboxField label="VRChat 以外からのコピーをスキップ"/>
                    </div>

                    <div>
                        <h3 class="text-sm font-medium mb-2 text-gray-700 dark:text-gray-300 border-b dark:border-gray-700 pb-1 transition-colors">詳細設定</h3>
                    </div>
                </div>
            </div>
        </div>
    }
}
