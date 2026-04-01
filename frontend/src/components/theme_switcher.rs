use crate::store::{theme_store::Theme, use_theme_store};
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ThemeSwitcherVariant {
    Button,
    Segmented,
    Icon,
}

#[component]
pub fn ThemeSwitcher(
    #[prop(default = ThemeSwitcherVariant::Icon)] variant: ThemeSwitcherVariant,
    #[prop(default = false)] show_label: bool,
) -> impl IntoView {
    match variant {
        ThemeSwitcherVariant::Icon => {
            view! { <ThemeSwitcherIcon show_label=show_label /> }.into_any()
        }
        ThemeSwitcherVariant::Button => view! { <ThemeSwitcherButton /> }.into_any(),
        ThemeSwitcherVariant::Segmented => view! { <ThemeSwitcherSegmented /> }.into_any(),
    }
}

#[component]
fn ThemeSwitcherIcon(#[prop(default = false)] show_label: bool) -> impl IntoView {
    let theme_store = use_theme_store();

    let is_dark = move || theme_store.theme.get() == Theme::Dark;

    view! {
        <button
            class="theme-switcher theme-switcher-icon"
            on:click=move |_| theme_store.toggle()
            aria-label="Toggle theme"
        >
            <span class="theme-switcher-icon-inner">
                <Show
                    when=is_dark
                    fallback=move || view! {
                        <svg class="theme-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <circle cx="12" cy="12" r="5"/>
                            <line x1="12" y1="1" x2="12" y2="3"/>
                            <line x1="12" y1="21" x2="12" y2="23"/>
                            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                            <line x1="1" y1="12" x2="3" y2="12"/>
                            <line x1="21" y1="12" x2="23" y2="12"/>
                            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                        </svg>
                    }
                >
                    <svg class="theme-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                    </svg>
                </Show>
            </span>
            {if show_label {
                view! {
                    <span class="theme-switcher-label">
                        {move || match theme_store.theme.get() {
                            Theme::Light => "Light",
                            Theme::Dark => "Dark",
                            Theme::System => "Auto",
                        }}
                    </span>
                }.into_any()
            } else {
                ().into_any()
            }}
        </button>
    }
}

#[component]
fn ThemeSwitcherButton() -> impl IntoView {
    let theme_store = use_theme_store();

    let is_dark = move || theme_store.theme.get() == Theme::Dark;

    view! {
        <button
            class="theme-switcher theme-switcher-button"
            on:click=move |_| theme_store.toggle()
        >
            <Show
                when=is_dark
                fallback=move || view! {
                    <svg class="theme-icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <circle cx="12" cy="12" r="5"/>
                        <line x1="12" y1="1" x2="12" y2="3"/>
                        <line x1="12" y1="21" x2="12" y2="23"/>
                        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                        <line x1="1" y1="12" x2="3" y2="12"/>
                        <line x1="21" y1="12" x2="23" y2="12"/>
                        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                    </svg>
                }
            >
                <svg class="theme-icon-sm" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
            </Show>
            <span>{move || match theme_store.theme.get() {
                Theme::Light => "Light Mode",
                Theme::Dark => "Dark Mode",
                Theme::System => "Auto Mode",
            }}</span>
        </button>
    }
}

#[component]
fn ThemeSwitcherSegmented() -> impl IntoView {
    let theme_store = use_theme_store();

    let is_light = move || matches!(theme_store.theme.get(), Theme::Light);
    let is_dark = move || matches!(theme_store.theme.get(), Theme::Dark);

    view! {
        <div class="theme-switcher theme-switcher-segmented">
            <button
                class=("theme-segment", true)
                class=("theme-segment-active", is_light)
                on:click=move |_| theme_store.set_theme.set(Theme::Light)
            >
                <svg class="theme-icon-xs" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="5"/>
                    <line x1="12" y1="1" x2="12" y2="3"/>
                    <line x1="12" y1="21" x2="12" y2="23"/>
                    <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
                    <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
                    <line x1="1" y1="12" x2="3" y2="12"/>
                    <line x1="21" y1="12" x2="23" y2="12"/>
                    <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
                    <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
                </svg>
                <span>"Light"</span>
            </button>
            <button
                class=("theme-segment", true)
                class=("theme-segment-active", is_dark)
                on:click=move |_| theme_store.set_theme.set(Theme::Dark)
            >
                <svg class="theme-icon-xs" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
                </svg>
                <span>"Dark"</span>
            </button>
        </div>
    }
}
