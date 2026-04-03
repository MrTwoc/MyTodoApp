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
                <AppHeader />
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
fn AppHeader() -> impl IntoView {
    use components::theme_switcher::ThemeSwitcher;

    view! {
        <header class="app-header">
            <a href="/dashboard" class="app-logo">"todoManager"</a>
            <div class="header-actions">
                <ThemeSwitcher />
            </div>
        </header>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> })
}
