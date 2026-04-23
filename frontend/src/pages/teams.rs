use crate::api::ApiClient;
use crate::api::team::{CreateTeamRequest, create_team, list_teams};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::Input;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::modal::Modal;
use crate::components::search::SearchInput;
use crate::store::team_store::TeamStore;
use crate::store::{use_api_client, use_team_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

fn load_team_list(client: ApiClient, team_store: TeamStore) {
    team_store.set_loading(true);
    wasm_bindgen_futures::spawn_local(async move {
        match list_teams(&client).await {
            Ok(teams) => team_store.set_teams(teams),
            Err(e) => team_store.set_error(e.message),
        }
    });
}

#[component]
pub fn TeamsPage() -> impl IntoView {
    let team_store = use_team_store();
    let client = use_api_client();
    let navigate = use_navigate();

    let (search_query, set_search_query) = signal(String::new());
    let (loaded, set_loaded) = signal(false);

    let (show_create_modal, set_show_create_modal) = signal(false);
    let (team_name, set_team_name) = signal(String::new());
    let (team_desc, set_team_desc) = signal(String::new());
    let (team_limit, set_team_limit) = signal(String::new());
    let (create_loading, set_create_loading) = signal(false);
    let (form_error, set_form_error) = signal(Option::<String>::None);

    let nav_back = {
        let n = navigate.clone();
        move |_| n("/dashboard", Default::default())
    };

    let open_create = Callback::from(move |_| {
        set_form_error.set(None);
        set_show_create_modal.set(true);
    });

    let close_create = Callback::from(move |_| {
        set_show_create_modal.set(false);
    });

    let do_create_client = client.clone();
    let do_create_store = team_store.clone();

    let do_create = Callback::from(move |_: ev::SubmitEvent| {
        let name = team_name.get().trim().to_string();
        let desc = team_desc.get().trim().to_string();
        let limit_text = team_limit.get().trim().to_string();

        if name.is_empty() {
            set_form_error.set(Some("Team name is required".to_string()));
            return;
        }

        let member_limit = if limit_text.is_empty() {
            None
        } else {
            match limit_text.parse::<u16>() {
                Ok(v) => Some(v),
                Err(_) => {
                    set_form_error.set(Some("Member limit must be a number".to_string()));
                    return;
                }
            }
        };

        let req = CreateTeamRequest {
            team_name: name,
            team_description: if desc.is_empty() { None } else { Some(desc) },
            team_visibility: None,
            team_member_limit: member_limit,
        };

        let client = do_create_client.clone();
        let team_store = do_create_store.clone();
        let set_loading = set_create_loading;
        let set_show = set_show_create_modal;
        let set_name = set_team_name;
        let set_desc = set_team_desc;
        let set_limit = set_team_limit;
        let clear_error = set_form_error;

        set_loading.set(true);
        clear_error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            match create_team(&client, &req).await {
                Ok(team) => {
                    team_store.add_team(team);
                    set_name.set(String::new());
                    set_desc.set(String::new());
                    set_limit.set(String::new());
                    set_show.set(false);
                    set_loading.set(false);
                }
                Err(e) => {
                    clear_error.set(Some(e.message));
                    set_loading.set(false);
                }
            }
        });
    });

    let retry_load = {
        let client = client.clone();
        let team_store = team_store.clone();
        Callback::from(move |_| {
            load_team_list(client.clone(), team_store.clone());
        })
    };

    let filtered_teams_store = team_store.clone();

    let filtered_teams = move || {
        let q = search_query.get().to_lowercase();
        let teams = filtered_teams_store.state.get().teams;

        if q.is_empty() {
            teams
        } else {
            teams
                .into_iter()
                .filter(|team| {
                    team.team_name.to_lowercase().contains(&q)
                        || team
                            .team_settings
                            .team_description
                            .as_ref()
                            .is_some_and(|desc| desc.to_lowercase().contains(&q))
                })
                .collect::<Vec<_>>()
        }
    };

    let effect_client = client.clone();
    let effect_store = team_store.clone();
    Effect::new(move |_| {
        if !loaded.get() {
            set_loaded.set(true);
            load_team_list(effect_client.clone(), effect_store.clone());
        }
    });

    view! {
        <div class="teams-page">
            <div class="teams-header">
                <div class="teams-header-left">
                    <button class="back-btn" on:click=nav_back>"← Back"</button>
                    <h1 class="teams-page-title">"Teams"</h1>
                    <p class="teams-page-subtitle">"Manage your " <span class="teams-accent">"collaborative workspaces"</span></p>
                </div>
                <div class="teams-header-right">
                    <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=open_create>
                        "+ New Team"
                    </Button>
                </div>
            </div>

            <div class="teams-toolbar">
                <SearchInput
                    placeholder="Search teams...".to_string()
                    instant=true
                    on_search=Callback::from(move |q: String| {
                        set_search_query.set(q);
                    })
                />
            </div>

            {move || {
                if team_store.state.get().is_loading {
                    view! {
                        <div class="teams-loading">
                            <Loading variant=LoadingVariant::Spinner label="Loading teams...".to_string() />
                        </div>
                    }.into_any()
                } else if let Some(err) = team_store.state.get().error.clone() {
                    view! {
                        <div class="teams-error">
                            <div class="teams-error-icon">
                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <circle cx="12" cy="12" r="10"/>
                                    <line x1="12" y1="8" x2="12" y2="12"/>
                                    <line x1="12" y1="16" x2="12.01" y2="16"/>
                                </svg>
                            </div>
                            <p class="teams-error-text">{err}</p>
                            <Button
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Sm
                                on_click=retry_load
                            >
                                "Retry"
                            </Button>
                        </div>
                    }.into_any()
                } else {
                    let teams = filtered_teams();
                    if teams.is_empty() {
                        view! {
                            <div class="teams-empty">
                                <div class="teams-empty-icon">
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                                        <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                        <circle cx="9" cy="7" r="4"/>
                                        <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                                        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                                    </svg>
                                </div>
                                <h3 class="teams-empty-title">"No Teams Yet"</h3>
                                <p class="teams-empty-desc">"Create your first team to start collaborating with others"</p>
                                <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=open_create>
                                    "Create Team"
                                </Button>
                            </div>
                        }.into_any()
                    } else {
                        let cards = teams
                            .into_iter()
                            .map(|team| {
                                let team_store_for_detail = team_store.clone();
                                let nav = navigate.clone();
                                let id = team.team_id;
                                view! {
                                    <div class="team-card" on:click=move |_| {
                                        team_store_for_detail.set_active_team(Some(id));
                                        let path = format!("/teams/{}", id);
                                        nav(&path, Default::default());
                                    }>
                                        <div class="team-card-header">
                                            <div class="team-card-icon">
                                                {team.team_name.split_whitespace().take(2).map(|w| w.chars().next().map(|c| c.to_ascii_uppercase()).unwrap_or(' ')).collect::<String>()}
                                            </div>
                                            <div class="team-card-info">
                                                <h3 class="team-card-name">{team.team_name.clone()}</h3>
                                                <span class="team-card-id">"#" {team.team_id}</span>
                                            </div>
                                        </div>
                                        <p class="team-card-desc">{team.team_settings.team_description.unwrap_or_else(|| "No description".to_string())}</p>
                                        <div class="team-card-stats">
                                            <div class="team-card-stat">
                                                <svg class="team-card-stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                                                    <circle cx="9" cy="7" r="4"/>
                                                    <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                                                    <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                                                </svg>
                                                <span class="team-card-stat-value">{team.team_members.len()}</span>
                                                <span class="team-card-stat-label">"members"</span>
                                            </div>
                                            {if team.team_settings.team_member_limit > 0 {
                                                view! {
                                                    <div class="team-card-stat">
                                                        <svg class="team-card-stat-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                            <path d="M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z"/>
                                                            <path d="M12 6v6l4 2"/>
                                                        </svg>
                                                        <span class="team-card-stat-value">{team.team_settings.team_member_limit}</span>
                                                        <span class="team-card-stat-label">"limit"</span>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }}
                                        </div>
                                        <div class="team-card-arrow">
                                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                <path d="M9 18l6-6-6-6"/>
                                            </svg>
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>();
                        view! { <div class="teams-grid">{cards}</div> }.into_any()
                    }
                }
            }}

            <Modal
                title="Create Team".to_string()
                open=show_create_modal.into()
                on_close=close_create
            >
                <Form on_submit=do_create>
                    <FormGroup label="Team Name".to_string() required=true error=form_error.get().unwrap_or_default()>
                        <Input
                            value=team_name.get()
                            on_input=Callback::from(move |v: String| {
                                set_team_name.set(v);
                            })
                        />
                    </FormGroup>
                    <FormGroup label="Team Description".to_string()>
                        <textarea
                            class="input-field profile-textarea"
                            rows="3"
                            prop:value=team_desc
                            on:input=move |ev| set_team_desc.set(event_target_value(&ev))
                        />
                    </FormGroup>
                    <FormGroup label="Member Limit".to_string()>
                        <Input
                            value=team_limit.get()
                            on_input=Callback::from(move |v: String| {
                                set_team_limit.set(v);
                            })
                        />
                    </FormGroup>

                    {move || {
                        form_error.get().map(|msg| {
                            view! { <p class="auth-error">{msg}</p> }.into_any()
                        })
                    }}

                    <FormActions>
                        <Button
                            variant=ButtonVariant::Ghost
                            size=ButtonSize::Sm
                            on_click=close_create
                        >
                            "Cancel"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Sm
                            disabled=create_loading.get()
                        >
                            {move || if create_loading.get() { "Creating..." } else { "Create" }}
                        </Button>
                    </FormActions>
                </Form>
            </Modal>
        </div>
    }
}
