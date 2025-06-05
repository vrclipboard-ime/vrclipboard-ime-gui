use crate::menuitem::MenuItem;
use leptos::prelude::*;
use lucide_leptos::{Bug, Info, List, Settings};

#[component]
pub fn Sidebar() -> impl IntoView {
    let (active_menu_item, set_active_menu_item) = signal(String::from("log"));

    view! {
        <nav class="w-36 py-2 px-1 border-r border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-800 transition-colors flex flex-col h-full">
            // 上側のメニュー
            <div class="space-y-1 flex-grow">
                <MenuItem
                    label="ログ"
                    path="/log"
                    is_active=move || active_menu_item.get() == "log"
                    on_click={move |_| set_active_menu_item.set("log".to_string())}
                >
                    <List size=16 />
                </MenuItem>
                <MenuItem
                    label="設定"
                    path="/settings"
                    is_active=move || active_menu_item.get() == "settings"
                    on_click={move |_| set_active_menu_item.set("settings".to_string())}
                >
                    <Settings size=16 />
                </MenuItem>
            </div>

            // 下側のメニュー
            <div class="space-y-1 pt-2 border-t border-gray-100 dark:border-gray-700">
                <MenuItem
                    label="デバッグ"
                    path="/debug"
                    is_active=move || active_menu_item.get() == "debug"
                    on_click={move |_| set_active_menu_item.set("debug".to_string())}
                >
                    <Bug size=16 />
                </MenuItem>
                <MenuItem
                    label="情報"
                    path="/info"
                    is_active=move || active_menu_item.get() == "info"
                    on_click={move |_| set_active_menu_item.set("info".to_string())}
                >
                    <Info size=16 />
                </MenuItem>
            </div>
        </nav>
    }
}
