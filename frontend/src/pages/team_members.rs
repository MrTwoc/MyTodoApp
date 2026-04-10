use crate::api::team::{get_members, get_team};
use crate::components::card::Card;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::team_store::TeamMember;
use crate::store::{use_api_client, use_team_store};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

#[component]
pub fn TeamMembersPage() -> impl IntoView {
    let params = use_params_map();
    let team_id = params
        .get()
        .get("team_id")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);

    let navigate = use_navigate();
    let client = use_api_client();
    let team_store = use_team_store();

    let (page_error, set_page_error) = signal(Option::<String>::None);
    let (members, set_members) = signal(Vec::<TeamMember>::new());
    let (loading, set_loading) = signal(true);
    let (members_error, set_members_error) = signal(Option::<String>::None);

    let on_back = {
        let n = navigate.clone();
        let tid = team_id;
        move |_| n(&format!("/teams/{}", tid), Default::default())
    };

    let current_team = {
        let team_store = team_store.clone();
        move || {
            team_store
                .state
                .get()
                .teams
                .into_iter()
                .find(|team| team.team_id == team_id)
        }
    };

    Effect::new(move |_| {
        if team_id == 0 {
            set_page_error.set(Some("Invalid team id".to_string()));
            set_loading.set(false);
            return;
        }

        let client = client.clone();
        let team_store = team_store.clone();
        let set_page_error = set_page_error;
        let set_members = set_members;
        let set_loading = set_loading;
        let set_members_error = set_members_error;

        wasm_bindgen_futures::spawn_local(async move {
            match get_team(&client, team_id).await {
                Ok(team) => {
                    team_store.upsert_team(team);
                }
                Err(e) => {
                    set_page_error.set(Some(e.message));
                    set_loading.set(false);
                    return;
                }
            }

            match get_members(&client, team_id).await {
                Ok(member_list) => {
                    set_members.set(member_list);
                }
                Err(e) => {
                    set_members_error.set(Some(e.message));
                }
            }

            set_loading.set(false);
        });
    });

    let total_members = move || members.get().len();

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    <button class="back-btn" on:click=on_back>"← Back"</button>
                    <a href=move || format!("/teams/{}", team_id) class="page-title">
                        {move || {
                            current_team()
                                .map(|team| format!("{} - Members", team.team_name))
                                .unwrap_or_else(|| "Team Members".to_string())
                        }}
                    </a>
                    <TeamModuleNav team_id=team_id />
                </div>
            </header>

            <Show
                when=move || page_error.get().is_none()
                fallback=move || {
                    view! {
                        <Card title="Error".to_string()>
                            <p class="auth-error">
                                {page_error.get().unwrap_or_else(|| "Unknown error".to_string())}
                            </p>
                        </Card>
                    }
                }
            >
                <Card title="Members".to_string() subtitle="Team members".to_string()>
                    <div class="team-detail-stats">
                        <div class="team-stat">
                            <span class="team-stat-number">{total_members}</span>
                            <span class="team-stat-label">"Total Members"</span>
                        </div>
                    </div>
                </Card>

                <Show
                    when=move || !loading.get()
                    fallback=move || {
                        view! {
                            <Loading variant=LoadingVariant::Spinner label="Loading members...".to_string() />
                        }
                    }
                >
                    <Show
                        when=move || members_error.get().is_none()
                        fallback=move || {
                            view! {
                                <Card title="Error".to_string()>
                                    <p class="auth-error">
                                        {members_error.get().unwrap_or_else(|| "Unknown error".to_string())}
                                    </p>
                                </Card>
                            }
                        }
                    >
                        <Card title="Member List".to_string() subtitle="Team member list".to_string()>
                            {move || {
                                if members.get().is_empty() {
                                    view! { <p class="empty-text">"No members in this team."</p> }.into_any()
                                } else {
                                    let member_items = members
                                        .get()
                                        .into_iter()
                                        .map(|member| {
                                            let _user_id = member.user_id;
                                            let level = member.level;
                                            let username = member.username.clone().unwrap_or_else(|| "Unknown".to_string());
                                            
                                            view! {
                                                <div class="team-member-item">
                                                    <div class="team-member-meta">
                                                        <span class="team-member-id">
                                                            <svg class="member-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                                                                <circle cx="12" cy="7" r="4"/>
                                                            </svg>
                                                            {username}
                                                        </span>
                                                        <span class="team-member-role">{format!("Level {}", level)}</span>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    view! { <div class="team-member-list">{member_items}</div> }.into_any()
                                }
                            }}
                        </Card>
                    </Show>
                </Show>
            </Show>
        </div>
    }
}