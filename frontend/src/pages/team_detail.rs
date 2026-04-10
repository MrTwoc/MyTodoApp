use crate::api::ApiClient;
use crate::api::task::{CreateTaskRequest, TaskListResponse, create_task, list_tasks};
use crate::api::team::{
    AddMemberRequest, UpdateRoleRequest, UpdateTeamRequest, add_member, get_members, get_team,
    remove_member, update_member_role, update_team,
};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::{Card, CardFooter};
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::Input;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::modal::Modal;
use crate::components::search::SearchInput;
use crate::components::task_form::{TaskFormData, TaskFormMode, TaskFormModal};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::task_store::{Task, TaskStatus};
use crate::store::team_store::{TeamMember, TeamStore};
use crate::store::user_store::UserStore;
use crate::store::{use_api_client, use_team_store, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

fn format_timestamp(ts: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((ts * 1000) as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn status_text(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Active => "Active",
        TaskStatus::Completed => "Completed",
        TaskStatus::Paused => "Paused",
    }
}

fn status_color(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Active => "status-active",
        TaskStatus::Completed => "status-completed",
        TaskStatus::Paused => "status-paused",
    }
}

fn load_team_detail_data(
    team_id: u64,
    client: ApiClient,
    team_store: TeamStore,
    user_store: UserStore,
    set_page_error: WriteSignal<Option<String>>,
    set_member_action_error: WriteSignal<Option<String>>,
    set_tasks_error: WriteSignal<Option<String>>,
    set_tasks_loading: WriteSignal<bool>,
    set_team_tasks: WriteSignal<Vec<Task>>,
    set_current_user_id_val: WriteSignal<u64>,
    set_current_user_level_val: WriteSignal<u8>,
    set_team_leader_id_val: WriteSignal<u64>,
) {
    if team_id == 0 {
        set_page_error.set(Some("Invalid team id".to_string()));
        return;
    }

    set_page_error.set(None);
    set_member_action_error.set(None);
    set_tasks_loading.set(true);
    set_tasks_error.set(None);

    let client = client.clone();
    let team_store = team_store.clone();

    let set_page_error = set_page_error;
    let set_member_action_error = set_member_action_error;
    let set_tasks_error = set_tasks_error;
    let set_tasks_loading = set_tasks_loading;
    let set_team_tasks = set_team_tasks;

    wasm_bindgen_futures::spawn_local(async move {
        match get_team(&client, team_id).await {
            Ok(team) => {
                team_store.upsert_team(team);
                team_store.set_active_team(Some(team_id));
            }
            Err(e) => {
                set_page_error.set(Some(e.message));
                return;
            }
        }

        let user_id = user_store.user_id().unwrap_or(0);

        match get_members(&client, team_id).await {
            Ok(members) => {
                let members_clone = members.clone();
                team_store.set_team_members(team_id, members);
                let my_member = members_clone.iter().find(|m| m.user_id == user_id);
                if let Some(m) = my_member {
                    set_current_user_level_val.set(m.level);
                }
                if let Some(team) = team_store.state.get().teams.iter().find(|t| t.team_id == team_id) {
                    set_team_leader_id_val.set(team.team_leader_id);
                }
                set_current_user_id_val.set(user_id);
            }
            Err(e) => {
                set_member_action_error.set(Some(e.message));
            }
        }

        match list_tasks(&client, 1, 50, None, Some(team_id)).await {
            Ok(TaskListResponse { tasks, .. }) => {
                set_team_tasks.set(tasks);
            }
            Err(e) => {
                set_tasks_error.set(Some(e.message));
            }
        }

        set_tasks_loading.set(false);
    });
}

#[component]
fn TeamTaskRow(task: Task) -> impl IntoView {
    let navigate = use_navigate();
    let team_store = use_team_store();
    let task_id = task.task_id;
    let status = status_text(&task.task_status).to_string();
    let status_class = status_color(&task.task_status);
    let name = task.task_name.clone();
    let description = task
        .task_description
        .clone()
        .unwrap_or_else(|| "No description".to_string());

    let task_create_time = format_timestamp(task.task_create_time);

    let task_leader = {
        let team_store = team_store.clone();
        let task_team_id = task.task_team_id;
        let leader_id = task.task_leader_id;
        move || -> String {
            if let Some(team_id) = task_team_id {
                let teams = team_store.state.get().teams.clone();
                if let Some(team) = teams.iter().find(|t| t.team_id == team_id) {
                    if let Some(member) = team.team_members.iter().find(|m| m.user_id == leader_id) {
                        return member.username.clone().unwrap_or_else(|| leader_id.to_string());
                    }
                    return leader_id.to_string();
                }
            }
            leader_id.to_string()
        }
    };

    let on_open = Callback::new(move |_| {
        let path = format!("/tasks/{}", task_id);
        navigate(&path, Default::default());
    });

    view! {
        <div class="team-task-card">
            <div class="team-task-card-header">
                <h4 class="team-task-card-title">{name}</h4>
                <span class=format!("task-status-badge team-task-status {}", status_class)>
                    {status}
                </span>
            </div>
            <p class="team-task-card-desc">{description}</p>
            <div class="team-task-card-footer">
                <div class="team-task-card-meta">
                    <span class="team-task-meta-item">
                        <span class="team-task-meta-label">"Created: "</span>
                        <span>{task_create_time}</span>
                    </span>
                    <span class="team-task-meta-item">
                        <span class="team-task-meta-label">"Leader: "</span>
                        <span>{task_leader()}</span>
                    </span>
                </div>
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    on_click=on_open
                >
                    "View Details"
                </Button>
            </div>
        </div>
    }
}

#[component]
fn TeamMemberRow(
    member: TeamMember,
    team_id: u64,
    current_user_id: u64,
    current_user_level: u8,
    team_leader_id: u64,
    set_member_action_error: WriteSignal<Option<String>>,
    member_action_loading: ReadSignal<bool>,
    set_member_action_loading: WriteSignal<bool>,
) -> impl IntoView {
    let client = use_api_client();
    let team_store = use_team_store();
    let user_id = member.user_id;
    let role = member.level;
    let (draft_role, set_draft_role) = signal(role.to_string());

    let is_leader = user_id == team_leader_id;
    let higher_or_equal_level = role >= current_user_level;
    let can_edit = !is_leader && !higher_or_equal_level && current_user_id != user_id;
    let is_self = user_id == current_user_id;

    let on_save = {
        let client_for_update = client.clone();
        let team_store = team_store.clone();
        let set_member_action_error = set_member_action_error;
        let set_member_action_loading = set_member_action_loading;

        Callback::from(move |_| {
            let role_text = draft_role.get();
            let new_level = match role_text.parse::<u8>() {
                Ok(v) => v,
                Err(_) => {
                    set_member_action_error.set(Some("Invalid role level".to_string()));
                    return;
                }
            };
            let req = UpdateRoleRequest { level: new_level };
            let team_store = team_store.clone();
            let client = client_for_update.clone();
            let set_member_action_error = set_member_action_error;
            let set_member_action_loading = set_member_action_loading;

            set_member_action_loading.set(true);
            set_member_action_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match update_member_role(&client, team_id, user_id, &req).await {
                    Ok(()) => match get_members(&client, team_id).await {
                        Ok(members) => {
                            team_store.set_team_members(team_id, members);
                            set_member_action_loading.set(false);
                        }
                        Err(e) => {
                            set_member_action_error.set(Some(e.message));
                            set_member_action_loading.set(false);
                        }
                    },
                    Err(e) => {
                        set_member_action_error.set(Some(e.message));
                        set_member_action_loading.set(false);
                    }
                }
            });
        })
    };

    let on_remove = {
        let client_for_remove = client;
        let team_store = team_store;
        let set_member_action_error = set_member_action_error;
        let set_member_action_loading = set_member_action_loading;

        Callback::from(move |_| {
            let client = client_for_remove.clone();
            let team_store = team_store.clone();
            let set_member_action_error = set_member_action_error;
            let set_member_action_loading = set_member_action_loading;

            set_member_action_loading.set(true);
            set_member_action_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match remove_member(&client, team_id, user_id).await {
                    Ok(()) => match get_members(&client, team_id).await {
                        Ok(members) => {
                            team_store.set_team_members(team_id, members);
                            set_member_action_loading.set(false);
                        }
                        Err(e) => {
                            set_member_action_error.set(Some(e.message));
                            set_member_action_loading.set(false);
                        }
                    },
                    Err(e) => {
                        set_member_action_error.set(Some(e.message));
                        set_member_action_loading.set(false);
                    }
                }
            });
        })
    };

    view! {
        <div class="team-member-item">
            <div class="team-member-meta">
                <span class="team-member-id">
                    <svg class="member-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
                        <circle cx="12" cy="7" r="4"/>
                    </svg>
                    {user_id}
                </span>
                <span class="team-member-role">{format!("Level {}", role)}</span>
                <Show when=move || is_leader>
                    <span class="team-member-badge">"Leader"</span>
                </Show>
                <Show when=move || !can_edit && !is_self>
                    <span class="team-member-badge locked">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12">
                            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                            <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                        </svg>
                        "Locked"
                    </span>
                </Show>
            </div>
            <div class="team-member-actions">
                <Input
                    value=draft_role.get()
                    disabled=!can_edit
                    on_input=Callback::from(move |v: String| {
                        set_draft_role.set(v);
                    })
                />
                <Button
                    variant=ButtonVariant::Primary
                    size=ButtonSize::Sm
                    disabled=member_action_loading.get() || !can_edit
                    on_click=on_save
                >
                    "Save"
                </Button>
                <Button
                    variant=ButtonVariant::Danger
                    size=ButtonSize::Sm
                    disabled=member_action_loading.get() || is_leader
                    on_click=on_remove
                >
                    "Remove"
                </Button>
            </div>
        </div>
    }
}

#[component]
pub fn TeamDetailPage() -> impl IntoView {
    let params = use_params_map();
    let team_id = params
        .get()
        .get("team_id")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);

    let team_store = use_team_store();
    let client = use_api_client();
    let navigate = use_navigate();
    let user_store = use_user_store();

    let (loaded, set_loaded) = signal(false);
    let (page_error, set_page_error) = signal(Option::<String>::None);

    let (team_tasks, set_team_tasks) = signal(Vec::<Task>::new());
    let (tasks_loading, set_tasks_loading) = signal(false);
    let (tasks_error, set_tasks_error) = signal(Option::<String>::None);

    let (member_search_query, set_member_search_query) = signal(String::new());
    let (member_action_error, set_member_action_error) = signal(Option::<String>::None);
    let (member_action_loading, set_member_action_loading) = signal(false);
    let (new_member_user_id, set_new_member_user_id) = signal(String::new());
    let (new_member_level, set_new_member_level) = signal(String::new());

    let (show_edit_team_modal, set_show_edit_team_modal) = signal(false);
    let (edit_team_loading, set_edit_team_loading) = signal(false);
    let (edit_team_error, set_edit_team_error) = signal(Option::<String>::None);
    let (edit_member_limit, set_edit_member_limit) = signal(String::new());
    let (edit_visibility, set_edit_visibility) = signal(String::new());
    let (edit_description, set_edit_description) = signal(String::new());
    let (edit_team_name, set_edit_team_name) = signal(String::new());

    let (show_create_task_modal, set_show_create_task_modal) = signal(false);
    let (create_task_loading, set_create_task_loading) = signal(false);
    let (create_task_error, set_create_task_error) = signal(Option::<String>::None);

    let on_edit_team_submit = {
        let client = client.clone();
        let team_store = team_store.clone();
        Callback::from(move |_: ev::SubmitEvent| {
            set_edit_team_loading.set(true);
            set_edit_team_error.set(None);

            let member_limit: Option<u16> = if edit_member_limit.get().trim().is_empty() {
                Some(0)
            } else {
                edit_member_limit.get().trim().parse().ok()
            };
            // Validate team name input
            let team_name_input = edit_team_name.get();
            if team_name_input.trim().is_empty() {
                set_edit_team_error.set(Some("Team name is required".to_string()));
                set_edit_team_loading.set(false);
                return;
            }
            let visibility = if edit_visibility.get() == "Public" {
                Some("Public".to_string())
            } else {
                Some("Private".to_string())
            };
            let description = if edit_description.get().trim().is_empty() {
                None
            } else {
                Some(edit_description.get().trim().to_string())
            };

            let req = UpdateTeamRequest {
                team_name: Some(team_name_input.clone()),
                team_description: description,
                team_visibility: visibility,
                team_member_limit: member_limit,
            };

            let client = client.clone();
            let team_store = team_store.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match update_team(&client, team_id, &req).await {
                    Ok(updated_team) => {
                        team_store.upsert_team(updated_team);
                        set_edit_team_loading.set(false);
                        set_show_edit_team_modal.set(false);
                    }
                    Err(e) => {
                        set_edit_team_error.set(Some(e.message));
                        set_edit_team_loading.set(false);
                    }
                }
            });
        })
    };

    let on_back = {
        let n = navigate.clone();
        move |_| n("/teams", Default::default())
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

    let leader_username = {
        let team_store = team_store.clone();
        move || -> String {
            let teams = team_store.state.get().teams.clone();
            let team = teams.iter().find(|t| t.team_id == team_id);
            if let Some(team) = team {
                if let Some(member) = team.team_members.iter().find(|m| m.user_id == team.team_leader_id) {
                    return member.username.clone().unwrap_or_else(|| team.team_leader_id.to_string());
                }
                return team.team_leader_id.to_string();
            }
            "Unknown".to_string()
        }
    };

    let total_members = move || current_team().map_or(0, |team| team.team_members.len());

    let (current_user_id_val, set_current_user_id_val) = signal(0u64);
    let (current_user_level_val, set_current_user_level_val) = signal(0u8);
    let (team_leader_id_val, set_team_leader_id_val) = signal(0u64);

    let team_leader_id = move || team_leader_id_val.get();
    let current_user_id = move || current_user_id_val.get();
    let current_user_level = move || current_user_level_val.get();

    let current_members = move || {
        current_team()
            .map(|team| team.team_members)
            .unwrap_or_default()
    };

    let current_members = move || {
        current_team()
            .map(|team| team.team_members)
            .unwrap_or_default()
    };

    let filtered_members = move || {
        let q = member_search_query.get().to_lowercase();
        if q.is_empty() {
            current_members()
        } else {
            current_members()
                .into_iter()
                .filter(|member| member.user_id.to_string().contains(&q))
                .collect::<Vec<_>>()
        }
    };

    let tasks_total = move || team_tasks.get().len();
    let tasks_active = move || {
        team_tasks
            .get()
            .iter()
            .filter(|task| matches!(task.task_status, TaskStatus::Active))
            .count()
    };
    let tasks_done = move || {
        team_tasks
            .get()
            .iter()
            .filter(|task| matches!(task.task_status, TaskStatus::Completed))
            .count()
    };
    let tasks_paused = move || {
        team_tasks
            .get()
            .iter()
            .filter(|task| matches!(task.task_status, TaskStatus::Paused))
            .count()
    };

    let effect_client = client.clone();
    let effect_store = team_store.clone();
    let effect_page_error = set_page_error;
    let effect_member_error = set_member_action_error;
    let effect_tasks_error = set_tasks_error;
    let effect_tasks_loading = set_tasks_loading;
    let effect_team_tasks = set_team_tasks;
    let effect_user_store = user_store.clone();
    let effect_current_user_id = set_current_user_id_val;
    let effect_current_level = set_current_user_level_val;
    let effect_leader_id = set_team_leader_id_val;
    Effect::new(move |_| {
        if !loaded.get() {
            set_loaded.set(true);
            load_team_detail_data(
                team_id,
                effect_client.clone(),
                effect_store.clone(),
                effect_user_store.clone(),
                effect_page_error,
                effect_member_error,
                effect_tasks_error,
                effect_tasks_loading,
                effect_team_tasks,
                effect_current_user_id,
                effect_current_level,
                effect_leader_id,
            );
        }
    });

    let on_refresh = Callback::from({
        let client = client.clone();
        let team_store = team_store.clone();
        let user_store = user_store.clone();
        let set_page_error = set_page_error;
        let set_member_action_error = set_member_action_error;
        let set_tasks_error = set_tasks_error;
        let set_tasks_loading = set_tasks_loading;
        let set_team_tasks = set_team_tasks;
        let set_current_user_id_val = set_current_user_id_val;
        let set_current_user_level_val = set_current_user_level_val;
        let set_team_leader_id_val = set_team_leader_id_val;
        move |_| {
            load_team_detail_data(
                team_id,
                client.clone(),
                team_store.clone(),
                user_store.clone(),
                set_page_error,
                set_member_action_error,
                set_tasks_error,
                set_tasks_loading,
                set_team_tasks,
                set_current_user_id_val,
                set_current_user_level_val,
                set_team_leader_id_val,
            );
        }
    });

    let on_create_task = Callback::from(move |_| {
        set_show_create_task_modal.set(true);
    });

    let on_create_task_submit = {
        let client = client.clone();
        let set_show_create_task_modal = set_show_create_task_modal;
        let set_create_task_loading = set_create_task_loading;
        let set_create_task_error = set_create_task_error;
        let set_team_tasks = set_team_tasks;
        let set_tasks_loading = set_tasks_loading;
        let set_tasks_error = set_tasks_error;
        Callback::from(move |data: TaskFormData| {
            let user_id = user_store.user_id().unwrap_or(0);
            let task_name = data.task_name.clone();
            let task_description = data.task_description.clone();
            let task_keywords = data.task_keywords.clone();
            let task_priority = data.task_priority;
            let task_difficulty = data.task_difficulty;
            let task_deadline = data.task_deadline;
            let req = CreateTaskRequest {
                task_name,
                task_description,
                task_keywords,
                task_priority,
                task_difficulty,
                task_deadline,
                task_leader_id: user_id,
                task_team_id: Some(team_id),
            };

            set_create_task_loading.set(true);
            set_create_task_error.set(None);

            let client = client.clone();
            let set_show_create_task_modal = set_show_create_task_modal;
            let set_create_task_loading = set_create_task_loading;
            let set_create_task_error = set_create_task_error;
            let set_team_tasks = set_team_tasks;
            let set_tasks_loading = set_tasks_loading;
            let set_tasks_error = set_tasks_error;

            wasm_bindgen_futures::spawn_local(async move {
                match create_task(&client, &req).await {
                    Ok(_) => {
                        set_show_create_task_modal.set(false);
                        set_create_task_loading.set(false);
                        set_tasks_loading.set(true);
                        match list_tasks(&client, 1, 50, None, Some(team_id)).await {
                            Ok(TaskListResponse { tasks, .. }) => {
                                set_team_tasks.set(tasks);
                            }
                            Err(e) => {
                                set_tasks_error.set(Some(e.message));
                            }
                        }
                        set_tasks_loading.set(false);
                    }
                    Err(e) => {
                        set_create_task_error.set(Some(e.message));
                        set_create_task_loading.set(false);
                    }
                }
            });
        })
    };

    let on_create_task_close = Callback::from(move || {
        set_show_create_task_modal.set(false);
        set_create_task_error.set(None);
    });

    let on_add_member = {
        let client = client.clone();
        let set_member_action_error = set_member_action_error;
        let set_member_action_loading = set_member_action_loading;
        let set_new_member_user_id = set_new_member_user_id;
        let set_new_member_level = set_new_member_level;
        Callback::from(move |_: ev::SubmitEvent| {
            let user_id_text = new_member_user_id.get();
            let level_text = new_member_level.get();

            if user_id_text.trim().is_empty() {
                set_member_action_error.set(Some("Member user id is required".to_string()));
                return;
            }
            if level_text.trim().is_empty() {
                set_member_action_error.set(Some("Role level is required".to_string()));
                return;
            }

            let user_id = match user_id_text.parse::<u64>() {
                Ok(v) => v,
                Err(_) => {
                    set_member_action_error.set(Some("Invalid user id".to_string()));
                    return;
                }
            };

            let level = match level_text.parse::<u8>() {
                Ok(v) => v,
                Err(_) => {
                    set_member_action_error.set(Some("Role level should be 0-255".to_string()));
                    return;
                }
            };

            let req = AddMemberRequest { user_id, level };
            let team_id = team_id;
            let client = client.clone();
            let team_store = team_store.clone();
            let set_member_action_error = set_member_action_error;
            let set_member_action_loading = set_member_action_loading;
            let set_new_member_user_id = set_new_member_user_id;
            let set_new_member_level = set_new_member_level;

            set_member_action_loading.set(true);
            set_member_action_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match add_member(&client, team_id, &req).await {
                    Ok(()) => {
                        set_new_member_user_id.set(String::new());
                        set_new_member_level.set(String::new());
                        match get_members(&client, team_id).await {
                            Ok(members) => {
                                team_store.set_team_members(team_id, members);
                            }
                            Err(e) => {
                                set_member_action_error.set(Some(e.message));
                            }
                        }
                        set_member_action_loading.set(false);
                    }
                    Err(e) => {
                        set_member_action_error.set(Some(e.message));
                        set_member_action_loading.set(false);
                    }
                }
            });
        })
    };

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    <button class="back-btn" on:click=on_back>"← Back"</button>
                    <h1 class="page-title">
                        {move || {
                            current_team()
                                .map(|team| team.team_name)
                                .unwrap_or_else(|| "Team".to_string())
                        }}
                    </h1>
                    <TeamModuleNav team_id=team_id />
                </div>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    on_click=on_refresh
                >
                    "Refresh"
                </Button>
            </header>

            <Show
                when=move || page_error.get().is_none()
                fallback=move || {
                    view! {
                        <Card
                            title="Team detail failed".to_string()
                            subtitle="Unable to load team".to_string()
                        >
                            <p class="auth-error">
                                {page_error.get().unwrap_or_else(|| "Unknown error".to_string())}
                            </p>
                            <FormActions>
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Sm
                                    on_click=on_refresh
                                >
                                    "Retry"
                                </Button>
                            </FormActions>
                        </Card>
                    }
                }
            >
                <div class="team-detail-grid">
                    <div class="team-info-col">
                        {move || {
                            current_team().map(|team| {
                                let created = format_timestamp(team.team_create_time);
                                let visibility = format!("{:?}", team.team_settings.team_visibility);
                                let description = team
                                    .team_settings
                                    .team_description
                                    .unwrap_or_else(|| "No description".to_string());
                                let member_limit = team.team_settings.team_member_limit;

                                view! {
                                    <Card title="Team Information".to_string() subtitle="Basic team info".to_string()>
                                        <div class="team-detail-info">
                                            <p class="team-detail-field">
                                                <span class="team-detail-field-icon id">ID</span>
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Team ID"</span>
                                                    <span>{team.team_id}</span>
                                                </span>
                                            </p>
                                            <p class="team-detail-field">
                                                <span class="team-detail-field-icon leader">LD</span>
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Leader"</span>
                                                    <span>{leader_username()}</span>
                                                </span>
                                            </p>
                                            <p class="team-detail-field">
                                                <span class="team-detail-field-icon created">CR</span>
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Created"</span>
                                                    <span>{created}</span>
                                                </span>
                                            </p>
                                            <p class="team-detail-field">
                                                <span class="team-detail-field-icon members">MB</span>
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Members"</span>
                                                    <span>{total_members()}</span>
                                                </span>
                                            </p>
                                            <p class="team-detail-field">
                                                <span class="team-detail-field-icon limit">LM</span>
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Member limit"</span>
                                                    <span>{if member_limit == 0 { "Unlimited".to_string() } else { member_limit.to_string() }}</span>
                                                </span>
                                            </p>
                                            <p class="team-detail-field">
                                                <span class="team-detail-field-icon visibility">VS</span>
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Visibility"</span>
                                                    <span>{visibility}</span>
                                                </span>
                                            </p>
                                            <p class="team-detail-field full-width">
                                                <span class="team-detail-field-content">
                                                    <span class="team-detail-label">"Description"</span>
                                                    <span class="team-detail-desc">{description}</span>
                                                </span>
                                            </p>
                                        </div>
                                        <CardFooter>
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                size=ButtonSize::Sm
                                                on_click=Callback::from(move |_| {
                                                if let Some(t) = current_team() {
                                                        // Pre-fill existing team settings into edit modal fields
                                                        set_edit_member_limit.set(t.team_settings.team_member_limit.to_string());
                                                        set_edit_visibility.set(format!("{:?}", t.team_settings.team_visibility));
                                                        set_edit_description.set(t.team_settings.team_description.clone().unwrap_or_default());
                                                        // Prefill team name as well
                                                        set_edit_team_name.set(t.team_name.clone());
                                                    }
                                                    set_show_edit_team_modal.set(true);
                                                })
                                            >
                                                "Edit"
                                            </Button>
                                        </CardFooter>
                                    </Card>
                                }
                            })
                        }}
                    </div>
                    <div class="team-stats-col">
                        <Card title="Task Progress".to_string() subtitle="Team tasks".to_string()>
                            <div class="team-detail-stats">
                                <div class="team-stat">
                                    <span class="team-stat-number">{tasks_total}</span>
                                    <span class="team-stat-label">"Total"</span>
                                </div>
                                <div class="team-stat">
                                    <span class="team-stat-number">{tasks_active}</span>
                                    <span class="team-stat-label">"Active"</span>
                                </div>
                                <div class="team-stat">
                                    <span class="team-stat-number">{tasks_done}</span>
                                    <span class="team-stat-label">"Done"</span>
                                </div>
                                <div class="team-stat">
                                    <span class="team-stat-number">{tasks_paused}</span>
                                    <span class="team-stat-label">"Paused"</span>
                                </div>
                            </div>
                        </Card>
                    </div>
                </div>

                <Card title="Members".to_string() subtitle="Member management".to_string()>
                    <SearchInput
                        placeholder="Search members by user id".to_string()
                        instant=true
                        on_search=Callback::from(move |query: String| {
                            set_member_search_query.set(query);
                        })
                    />

                    <Form on_submit=on_add_member>
                        <div class="team-member-add">
                            <FormGroup label="User ID".to_string() required=true>
                                <Input
                                    value=new_member_user_id.get()
                                    on_input=Callback::from(move |v: String| {
                                        set_new_member_user_id.set(v);
                                    })
                                />
                            </FormGroup>
                            <FormGroup label="Role".to_string() required=true>
                                <Input
                                    value=new_member_level.get()
                                    on_input=Callback::from(move |v: String| {
                                        set_new_member_level.set(v);
                                    })
                                />
                            </FormGroup>
                            <Button
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Sm
                                disabled=member_action_loading.get()
                            >
                                {move || if member_action_loading.get() { "Adding..." } else { "Add Member" }}
                            </Button>
                        </div>
                    </Form>

                    {move || {
                        member_action_error.get().map(|msg| view! {
                            <p class="auth-error">{msg}</p>
                        })
                    }}

                    {move || {
                    let members = filtered_members();
                    if members.is_empty() {
                        view! { <p class="empty-text">"No members found."</p> }.into_any()
                    } else {
                        let uid = current_user_id();
                        let ulvl = current_user_level();
                        let lid = team_leader_id();
                        let items = members
                            .into_iter()
                            .map(move |member: TeamMember| {
                                view! {
                                    <TeamMemberRow
                                        member=member
                                        team_id=team_id
                                        current_user_id=uid
                                        current_user_level=ulvl
                                        team_leader_id=lid
                                        set_member_action_error=set_member_action_error
                                        member_action_loading=member_action_loading
                                        set_member_action_loading=set_member_action_loading
                                    />
                                }
                            })
                            .collect::<Vec<_>>();
                        view! { <div class="team-member-list">{items}</div> }.into_any()
                    }
                    }}
                </Card>

                <Card title="Team Tasks".to_string() subtitle="Task list".to_string()>
                    <div class="team-task-header">
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Sm
                            on_click=on_create_task
                        >
                            "Create Task"
                        </Button>
                    </div>
                    {move || {
                        if tasks_loading.get() {
                            view! {
                                <Loading variant=LoadingVariant::Spinner label="Loading tasks...".to_string() />
                            }.into_any()
                        } else if let Some(msg) = tasks_error.get() {
                            view! {
                                <div>
                                    <p class="auth-error">{msg}</p>
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        size=ButtonSize::Sm
                                        on_click=on_refresh
                                    >
                                        "Retry"
                                    </Button>
                                </div>
                            }.into_any()
                        } else if team_tasks.get().is_empty() {
                            view! {
                                <p class="empty-text">"No tasks yet for this team."</p>
                            }.into_any()
                        } else {
                            let tasks = team_tasks
                                .get()
                                .into_iter()
                                .map(|task| {
                                    view! {
                                        <TeamTaskRow task=task />
                                    }
                                })
                                .collect::<Vec<_>>();
                            view! { <div class="team-task-list">{tasks}</div> }.into_any()
                        }
                    }}
                </Card>



            <Modal
            open=show_edit_team_modal.into()
            title="Edit Team Information".to_string()
        >
            <Form on_submit=on_edit_team_submit>
                <FormGroup label="Team name".to_string() required=true>
                    <Input
                        value=edit_team_name.get()
                        placeholder="Enter team name".to_string()
                        on_input=Callback::from(move |v: String| {
                            set_edit_team_name.set(v);
                        })
                    />
                </FormGroup>
                <FormGroup label="Member limit".to_string() required=false>
                    <Input
                        value=edit_member_limit.get()
                        placeholder="0 for unlimited".to_string()
                        on_input=Callback::from(move |v: String| {
                            set_edit_member_limit.set(v);
                        })
                    />
                </FormGroup>
                        <FormGroup label="Visibility".to_string() required=true>
                            <select
                                class="input-field"
                                prop:value=edit_visibility.get()
                                on:change=move |ev| {
                                    set_edit_visibility.set(event_target_value(&ev));
                                }
                            >
                                <option value="Public">Public</option>
                                <option value="Private">Private</option>
                            </select>
                        </FormGroup>
                        <FormGroup label="Description".to_string() required=false>
                            <textarea
                                class="input-field task-description-input"
                                placeholder="Enter team description"
                                prop:value=edit_description.get()
                                on:input=move |ev| {
                                    set_edit_description.set(event_target_value(&ev));
                                }
                            />
                        </FormGroup>
                        {move || {
                            if let Some(msg) = edit_team_error.get() {
                                view! { <p class="auth-error">{msg}</p> }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                        <FormActions>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Md
                                on_click=Callback::from(move |_| {
                                    set_show_edit_team_modal.set(false);
                                })
                            >
                                "Cancel"
                            </Button>
                            <Button
                                variant=ButtonVariant::Primary
                                size=ButtonSize::Md
                                disabled=edit_team_loading.get()
                            >
                                "Save"
                            </Button>
                        </FormActions>
                    </Form>
                </Modal>

            <TaskFormModal
                open=show_create_task_modal.into()
                mode=TaskFormMode::Create
                force_team_task=true
                active_team_id=Some(team_id)
                on_submit=on_create_task_submit
                on_close=on_create_task_close
            />
            </Show>
        </div>
    }
}
