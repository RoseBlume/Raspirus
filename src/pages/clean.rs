use leptos::*;
use leptos_i18n::t;
use crate::components::home_button::HomeButton;
use crate::i18n::use_i18n;

// TODO: Styling

#[component]
pub fn Clean() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div class="h-screen">
            <div class="flex h-full justify-center p-12 text-center">
                <div class="flex justify-center items-center h-full">
                    <div class="w-full">
                        <h1 class="inline-block align-middle p-2 font-medium leading-tight text-5xl mt-0 mb-2 text-maingreen">
                            {t!(i18n, clean_title)}
                        </h1>
                        <div class="flex justify-center">
                            <img
                                src="/images/success_image.png"
                                alt="Success"
                                class="h-auto max-w-[60%]"
                            />
                        </div>
                        <HomeButton />
                    </div>
                </div>
            </div>
        </div>
    }
}