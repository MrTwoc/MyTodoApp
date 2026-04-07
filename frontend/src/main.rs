use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

mod api;
mod components;
mod pages;
mod store;

use components::theme_switcher::ThemeSwitcher;
use pages::dashboard::DashboardPage;
use pages::login::LoginPage;
use pages::not_found::NotFoundPage;
use pages::profile::ProfilePage;
use pages::protected_route::ProtectedRoute;
use pages::register::RegisterPage;
use pages::settings::SettingsPage;
use pages::task_detail::TaskDetailPage;
use pages::tasks::TasksPage;
use pages::team_detail::TeamDetailPage;
use pages::teams::TeamsPage;
use store::{create_stores, provide_stores};

#[component]
fn App() -> impl IntoView {
    let stores = create_stores();
    provide_stores(stores);

    view! {
        <Router>
            <div class="app-container">
                <AppSidebar />
                <main class="app-main">
                    <Routes fallback=|| view! { <NotFoundPage /> }>
                        <Route path=path!("/") view=LoginPage />
                        <Route path=path!("/login") view=LoginPage />
                        <Route path=path!("/register") view=RegisterPage />
                        <Route
                            path=path!("/dashboard")
                            view=|| view! {
                                <ProtectedRoute>
                                    <DashboardPage />
                                </ProtectedRoute>
                            }
                        />
                        <Route
                            path=path!("/tasks")
                            view=|| view! {
                                <ProtectedRoute>
                                    <TasksPage />
                                </ProtectedRoute>
                            }
                        />
                        <Route
                            path=path!("/tasks/:task_id")
                            view=|| view! {
                                <ProtectedRoute>
                                    <TaskDetailPage />
                                </ProtectedRoute>
                            }
                        />
                        <Route
                            path=path!("/teams")
                            view=|| view! {
                                <ProtectedRoute>
                                    <TeamsPage />
                                </ProtectedRoute>
                            }
                        />
                        <Route
                            path=path!("/teams/:team_id")
                            view=|| view! {
                                <ProtectedRoute>
                                    <TeamDetailPage />
                                </ProtectedRoute>
                            }
                        />
                        <Route
                            path=path!("/settings")
                            view=|| view! {
                                <ProtectedRoute>
                                    <SettingsPage />
                                </ProtectedRoute>
                            }
                        />
                        <Route
                            path=path!("/profile")
                            view=|| view! {
                                <ProtectedRoute>
                                    <ProfilePage />
                                </ProtectedRoute>
                            }
                        />
                        <Route path=path!("/*any") view=NotFoundPage />
                    </Routes>
                </main>
            </div>
        </Router>
    }
}

#[component]
fn AppSidebar() -> impl IntoView {
    use leptos_router::hooks::use_location;
    use store::use_user_store;

    let location = use_location();
    let user_store = use_user_store();

    let logo_text = move || {
        let state = user_store.state.get();
        if state.is_authenticated {
            state.profile.as_ref()
                .and_then(|p| {
                    let username = &p.username;
                    if username.is_empty() { None } else { Some(username.clone()) }
                })
                .unwrap_or_else(|| "todoManager".to_string())
        } else {
            "todoManager".to_string()
        }
    };

    let is_active = move |path: &str| -> bool {
        location.pathname.get().starts_with(path)
    };

    view! {
        <aside class="app-sidebar">
            <div class="app-sidebar-header">
                <a href="/profile" class="app-logo">{logo_text()}</a>
                <ThemeSwitcher />
            </div>
            <nav class="app-sidebar-nav">
                <a
                    href="/dashboard"
                    class="sidebar-link"
                    class:active=move || is_active("/dashboard")
                >
                    <svg class="sidebar-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="3" y="3" width="7" height="7" rx="1"/>
                        <rect x="14" y="3" width="7" height="7" rx="1"/>
                        <rect x="3" y="14" width="7" height="7" rx="1"/>
                        <rect x="14" y="14" width="7" height="7" rx="1"/>
                    </svg>
                    "Dashboard"
                </a>
                <a
                    href="/tasks"
                    class="sidebar-link"
                    class:active=move || is_active("/tasks")
                >
                    <svg class="sidebar-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M9 11l3 3L22 4"/>
                        <path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/>
                    </svg>
                    "Tasks"
                </a>
                <a
                    href="/teams"
                    class="sidebar-link"
                    class:active=move || is_active("/teams")
                >
                    <svg class="sidebar-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/>
                        <circle cx="9" cy="7" r="4"/>
                        <path d="M23 21v-2a4 4 0 00-3-3.87"/>
                        <path d="M16 3.13a4 4 0 010 7.75"/>
                    </svg>
                    "Teams"
                </a>
                <a
                    href="/settings"
                    class="sidebar-link"
                    class:active=move || is_active("/settings")
                >
                    <svg class="sidebar-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="3"/>
                        <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06a1.65 1.65 0 001.82.33H9a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/>
                    </svg>
                    "Settings"
                </a>
            </nav>
            <div class="app-sidebar-footer">
                <a
                    href="/profile"
                    class="sidebar-link"
                    class:active=move || is_active("/profile")
                >
                    <svg class="sidebar-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
                        <circle cx="12" cy="7" r="4"/>
                    </svg>
                    "Profile"
                </a>
            </div>
        </aside>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> })
}
