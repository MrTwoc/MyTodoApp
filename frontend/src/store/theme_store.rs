use crate::store::{get_local_storage_item, set_local_storage_item};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::MediaQueryList;

const THEME_KEY: &str = "todo_theme";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => Theme::Dark,
            "system" => Theme::System,
            _ => Theme::Light,
        }
    }

    pub fn resolve(&self) -> Theme {
        match self {
            Theme::System => {
                if prefers_dark_mode() {
                    Theme::Dark
                } else {
                    Theme::Light
                }
            }
            other => *other,
        }
    }
}

fn prefers_dark_mode() -> bool {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
            return mql.matches();
        }
    }
    false
}

fn listen_system_theme_change(callback: impl Fn(Theme) + 'static) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(mql)) = window.match_media("(prefers-color-scheme: dark)") {
            let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
                let mql_ref = _event
                    .target()
                    .and_then(|t| t.dyn_into::<MediaQueryList>().ok());
                let is_dark = if let Some(mq) = mql_ref {
                    mq.matches()
                } else {
                    prefers_dark_mode()
                };

                if is_dark {
                    callback(Theme::Dark);
                } else {
                    callback(Theme::Light);
                }
            }) as Box<dyn FnMut(_)>);

            let _ =
                mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeStore {
    pub theme: ReadSignal<Theme>,
    pub set_theme: WriteSignal<Theme>,
}

impl ThemeStore {
    pub fn toggle(&self) {
        let current = self.theme.get();
        let new_theme = match current {
            Theme::Light => Theme::Dark,
            Theme::Dark | Theme::System => Theme::Light,
        };
        self.set_theme.set(new_theme);
    }

    pub fn apply_to_dom(&self) {
        let resolved = self.theme.get().resolve();
        if let Some(html) = document().document_element() {
            if let Some(el) = html.dyn_ref::<web_sys::HtmlElement>() {
                let _ = el.set_attribute("data-theme", resolved.as_str());
            }
        }
    }

    pub fn effective_theme(&self) -> Theme {
        self.theme.get().resolve()
    }

    pub fn is_dark(&self) -> bool {
        matches!(self.effective_theme(), Theme::Dark)
    }

    pub fn is_light(&self) -> bool {
        matches!(self.effective_theme(), Theme::Light)
    }
}

pub fn create_theme_store() -> ThemeStore {
    let stored = get_local_storage_item(THEME_KEY);
    let initial = stored.map(|s| Theme::from_str(&s)).unwrap_or(Theme::System);

    let resolved = initial.resolve();

    if let Some(html) = document().document_element() {
        if let Some(el) = html.dyn_ref::<web_sys::HtmlElement>() {
            let _ = el.set_attribute("data-theme", resolved.as_str());
        }
    }

    let (theme, set_theme) = signal(initial);

    let _ = Effect::new(move || {
        let t = theme.get();
        set_local_storage_item(THEME_KEY, t.as_str());

        let resolved = t.resolve();
        if let Some(html) = document().document_element() {
            if let Some(el) = html.dyn_ref::<web_sys::HtmlElement>() {
                let _ = el.set_attribute("data-theme", resolved.as_str());
            }
        }
    });

    listen_system_theme_change(move |system_theme| {
        let current = theme.get();
        if current == Theme::System {
            if let Some(html) = document().document_element() {
                if let Some(el) = html.dyn_ref::<web_sys::HtmlElement>() {
                    let _ = el.set_attribute("data-theme", system_theme.as_str());
                }
            }
        }
    });

    ThemeStore { theme, set_theme }
}
