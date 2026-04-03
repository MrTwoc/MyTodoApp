use crate::api::ApiClient;
use crate::api::team::{CreateTeamRequest, create_team, list_teams};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::Input;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::modal::Modal;
use crate::components::search::SearchInput;
use crate::components::team_card::TeamCard;
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

    let refresh = {
        let client = client.clone();
        let team_store = team_store.clone();
        Callback::from(move |_| {
            set_search_query.set(String::new());
            load_team_list(client.clone(), team_store.clone());
        })
    };

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
        <div class="page">
            <header class="page-header">
                <div>
                    <button class="back-btn" on:click=nav_back>{"← Back"}</button>
                    <h1 class="page-title">"Teams"</h1>
                </div>
                <Button variant=ButtonVariant::Primary size=ButtonSize::Sm on_click=open_create>
                    "New Team"
                </Button>
            </header>

            <div class="teams-toolbar">
                <SearchInput
                    placeholder="Search teams...".to_string()
                    instant=true
                    on_search=Callback::from(move |q: String| {
                        set_search_query.set(q);
                    })
                />
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    on_click=refresh
                >
                    "Refresh"
                </Button>
            </div>

            {move || {
                if team_store.state.get().is_loading {
                    view! { <Loading variant=LoadingVariant::Spinner label="Loading teams...".to_string() /> }.into_any()
                } else if let Some(err) = team_store.state.get().error.clone() {
                    view! {
                        <Card
                            title="Load failed".to_string()
                            subtitle="Team data could not be loaded".to_string()
                        >
                                        <p class="auth-error">{err}</p>
                                    <Button
                                        variant=ButtonVariant::Primary
                                        size=ButtonSize::Sm
                                    on_click=retry_load
                            >
                                "Retry"
                            </Button>
                        </Card>
                    }.into_any()
                } else {
                    let teams = filtered_teams();
                    if teams.is_empty() {
                        view! {
                            <Card
                                title="No Teams".to_string()
                                subtitle="Create your first team to get started".to_string()
                            >
                                <p class="empty-text">"No teams found for your account yet."</p>
                            </Card>
                        }.into_any()
                    } else {
                        let cards = teams
                            .into_iter()
                            .map(|team| {
                                let team_store_for_detail = team_store.clone();
                                let nav = navigate.clone();
                                let id = team.team_id;
                                let team_name = team.team_name.clone();
                                let team_members = team.team_members.len();
                                view! {
                                    <TeamCard
                                        team=team.clone()
                                        interactive=true
                                        on_click=Callback::from(move |_| {
                                            team_store_for_detail.set_active_team(Some(id));
                                            let path = format!("/teams/{}", id);
                                            nav(&path, Default::default());
                                        })
                                    >
                                        <div class="team-card-meta" style="margin-top: 10px; display: flex; justify-content: space-between; color: #6c757d; font-size: 12px;">
                                            <span>{team_name}</span>
                                            <span>{team_members} " members"</span>
                                        </div>
                                    </TeamCard>
                                }
                            })
                            .collect::<Vec<_>>();
                        view! { <div class="team-list">{cards}</div> }.into_any()
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
