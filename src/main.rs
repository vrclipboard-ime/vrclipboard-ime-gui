mod checkbox;
mod info;
mod menuitem;
mod settings;
mod sidebar;

use info::Info;
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;
use settings::Settings;
use sidebar::Sidebar;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! {
            <div class="h-screen flex flex-col bg-white dark:bg-gray-900 transition-colors select-none">
                <div class="flex flex-1 h-[calc(100vh-32px)] overflow-hidden">
                    <Router>
                        <Sidebar />
                        <main class="container flex-1 p-3 overflow-y-auto dark:text-gray-200 transition-colors">
                            <Routes fallback=|| view! { <h1>""</h1> }>
                                <Route path=path!("/") view=|| view! { <h1>"Home"</h1> }/>
                                <Route path=path!("/settings") view=Settings />
                                <Route path=path!("/info") view=Info />
                                <Route path=path!("/*any") view=|| view! { <h1>"Not Found"</h1> }/>
                            </Routes>
                        </main>
                    </Router>
                </div>
            </div>
        }
    })
}
