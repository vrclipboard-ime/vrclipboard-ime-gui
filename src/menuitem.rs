use leptos::{ev::MouseEvent, prelude::*};

#[component]
pub fn MenuItem(
    children: Children,
    label: impl Into<String>,
    path: impl Into<String>,
    is_active: impl Fn() -> bool + Send + Sync + 'static,
    on_click: impl FnMut(MouseEvent) + 'static,
) -> impl IntoView {
    let label = label.into();
    let path = path.into();

    view! {
        <a on:click={on_click} href={path} class={move || if is_active() { "flex items-center w-full px-3 py-2 rounded transition-colors bg-indigo-100 text-indigo-700 dark:bg-indigo-900/50 dark:text-indigo-300" } else { "flex items-center w-full px-3 py-2 rounded transition-colors hover:bg-gray-100 text-gray-600 dark:hover:bg-gray-700 dark:text-gray-300" }}>
            <div class="flex items-center justify-center w-5 h-5">
                {children()}
            </div>
            <span class="ml-2 text-sm">{label}</span>
        </a>
    }
}
