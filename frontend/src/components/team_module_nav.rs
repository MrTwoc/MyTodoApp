use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn TeamModuleNav(team_id: u64) -> impl IntoView {
    let location = use_location();

    let is_active = move |path: &str| -> bool { location.pathname.get() == path };

    let base_path = move || format!("/teams/{}", team_id);

    let tasks_path = move || format!("{}/tasks", base_path());
    let members_path = move || format!("{}/members", base_path());
    let history_path = move || format!("{}/history", base_path());
    let groups_path = move || format!("{}/groups", base_path());

    view! {
        <nav class="team-module-nav">
            <div class="team-module-nav-inner">
                <a
                    href=tasks_path()
                    class="team-module-nav-link"
                    class:active=move || is_active(&tasks_path())
                >
                    <svg class="team-module-nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M9 11l3 3L22 4"/>
                        <path d="M21 12v7a2 2 0 01-2 2H5a2 2 0 01-2-2V5a2 2 0 012-2h11"/>
                    </svg>
                    "队伍任务"
                </a>
                <a
                    href=members_path()
                    class="team-module-nav-link"
                    class:active=move || is_active(&members_path())
                >
                    <svg class="team-module-nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/>
                        <circle cx="9" cy="7" r="4"/>
                        <path d="M23 21v-2a4 4 0 00-3-3.87"/>
                        <path d="M16 3.13a4 4 0 010 7.75"/>
                    </svg>
                    "队伍队员"
                </a>
                <a
                    href=history_path()
                    class="team-module-nav-link"
                    class:active=move || is_active(&history_path())
                >
                    <svg class="team-module-nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"/>
                        <polyline points="12 6 12 12 16 14"/>
                    </svg>
                    "队伍修改历史"
                </a>
                <a
                    href=groups_path()
                    class="team-module-nav-link"
                    class:active=move || is_active(&groups_path())
                >
                    <svg class="team-module-nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/>
                        <circle cx="9" cy="7" r="4"/>
                        <path d="M23 21v-2a4 4 0 00-3-3.87"/>
                        <path d="M16 3.13a4 4 0 010 7.75"/>
                    </svg>
                    "小组管理"
                </a>
            </div>
        </nav>
    }
}
