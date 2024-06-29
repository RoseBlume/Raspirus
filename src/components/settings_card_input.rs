use leptonic::components::button::Button;
use leptonic::components::icon::Icon;
use leptonic::components::prelude::ButtonColor;
use leptonic::prelude::icondata;
use leptos::*;
use leptos::logging::log;
use tauri_wasm::plugin::dialog::FileDialogBuilder;

#[component]
pub fn SettingsInputCard(
    title: String,
    short_description: String,
    short_description_2: String,
    icon: icondata::Icon,
    set_value: WriteSignal<String>,
) -> impl IntoView {

    let handle_button_click = move || {
        spawn_local(async move {
            let file = FileDialogBuilder::new().pick_file().await;
            log!("Selected file: {:?}", file);
            // Same as with the folder, if the file is ok, we parse the path, else we don't do anything
            match file {
                Ok(Some(path)) => {
                    let path_buffer = path.path;
                    let path_string = path_buffer.into_os_string().into_string().unwrap_or_default();
                    set_value.set(path_string);
                }
                _ => return,
            }
        });
    };

    view! {
        <div class="flex flex-col m-6 p-2 bg-white rounded-2xl shadow-md">
            <div class="flex items-center justify-between mx-4">
                <div class="flex items-center">
                    <Icon icon=icon
                        class="w-16 h-16 rounded-2xl p-3 border border-maingreen-light text-maingreen-light bg-green-50"
                    />
                    <div class="flex flex-col ml-3">
                        <div class="font-medium">{title}</div>
                        <p class="text-sm text-gray-600 leading-none mt-1">{short_description}</p>
                        <p class="text-sm text-gray-600 leading-none mt-1">{short_description_2}</p>
                    </div>
                </div>
                <Button on_press=move |_| handle_button_click() color=ButtonColor::Info> "Change" </Button>
            </div>
        </div>
    }
}
