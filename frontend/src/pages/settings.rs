use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::form::FormActions;
use crate::components::theme_switcher::ThemeSwitcher;
use crate::store::{use_theme_store, use_user_store};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn SettingsPage() -> impl IntoView {
    let user_store = use_user_store();
    let theme_store = use_theme_store();
    let navigate = use_navigate();

    let profile = {
        let us = user_store.clone();
        move || us.profile()
    };

    let nav_back = {
        let n = navigate.clone();
        move |_| n("/", Default::default())
    };

    let nav_login = {
        let n = navigate.clone();
        let us = user_store.clone();
        move |_| {
            us.logout();
            n("/login", Default::default());
        }
    };

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    <button class="back-btn" on:click=nav_back>"← Back"</button>
                    <h1 class="page-title">"Settings"</h1>
                </div>
            </header>

            <div class="settings-grid">
                <Card title="Profile".to_string() subtitle="Your account information".to_string()>
                    {move || profile().map(|p| {
                        view! {
                            <div class="profile-info">
                                <div class="profile-field">
                                    <span class="profile-label">"User ID"</span>
                                    <span class="profile-value profile-id">{p.user_id}</span>
                                </div>
                                <div class="profile-field">
                                    <span class="profile-label">"Username"</span>
                                    <span class="profile-value">{p.username.clone()}</span>
                                </div>
                                <div class="profile-field">
                                    <span class="profile-label">"Email"</span>
                                    <span class="profile-value">{p.email.clone()}</span>
                                </div>
                            </div>
                        }.into_any()
                    }).unwrap_or_else(|| view! { <p>"Not logged in"</p> }.into_any())}
                    <div class="profile-actions">
                        <a href="/profile" class="profile-link-btn">"View Full Profile →"</a>
                    </div>
                </Card>

                <Card title="Appearance".to_string() subtitle="Customize your experience".to_string()>
                    <ThemeSwitcher variant=crate::components::theme_switcher::ThemeSwitcherVariant::Segmented show_label=true />
                </Card>

                <Card title="Account".to_string() subtitle="Manage your account".to_string()>
                    <FormActions>
                        <Button
                            variant=ButtonVariant::Danger
                            size=ButtonSize::Sm
                            on_click=Callback::from(nav_login)
                        >"Sign Out"</Button>
                    </FormActions>
                </Card>
            </div>
        </div>
    }
}
