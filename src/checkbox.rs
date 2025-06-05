use leptos::prelude::*;
use uuid::Uuid;

#[component]
pub fn CheckboxField(label: impl Into<String>) -> impl IntoView {
    let label = label.into();
    let checkbox_id = format!("checkbox-{}", Uuid::new_v4());

    view! {
        <div class="mb-2">
            <div class="flex items-start">
                <div class="flex items-center-h4">
                    <input
                        type="checkbox"
                        id={checkbox_id.clone()}
                        class="h-3.5 w-3.5 text-indigo-500 text-indigo-400 border-gray-300 dark:border-gray-600 rounded dark:bg-gray-700 transition-colors"
                    />
                </div>
                <div class="ml-2 text-xs">
                    <label
                        for={checkbox_id}
                        class="text-gray-700 dark:text-gray-300 transition-colors cursor-pointer"
                    >
                        {label}
                    </label>
                </div>
            </div>
        </div>
    }
}
