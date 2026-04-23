use crate::api::task::{TaskListResponse, list_tasks};
use crate::api::team::get_team;
use crate::components::card::Card;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::task_store::Task;
use crate::store::task_store::TaskStatus;
use crate::store::{use_api_client, use_team_store};
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

#[component]
pub fn TeamTasksPage() -> impl IntoView {
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
    let (tasks, set_tasks) = signal(Vec::<Task>::new());
    let (loading, set_loading) = signal(true);
    let (tasks_error, set_tasks_error) = signal(Option::<String>::None);

    // let on_back = {
    //     let n = navigate.clone();
    //     let tid = team_id;
    //     move |_| n(&format!("/teams/{}", tid), Default::default())
    // };

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
        let set_tasks = set_tasks;
        let set_loading = set_loading;
        let set_tasks_error = set_tasks_error;

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

            match list_tasks(&client, 1, 100, None, Some(team_id)).await {
                Ok(TaskListResponse {
                    tasks: task_list, ..
                }) => {
                    set_tasks.set(task_list);
                }
                Err(e) => {
                    set_tasks_error.set(Some(e.message));
                }
            }

            set_loading.set(false);
        });
    });

    let tasks_total = move || tasks.get().len();
    let tasks_active = move || {
        tasks
            .get()
            .iter()
            .filter(|task| matches!(task.task_status, TaskStatus::Active))
            .count()
    };
    let tasks_done = move || {
        tasks
            .get()
            .iter()
            .filter(|task| matches!(task.task_status, TaskStatus::Completed))
            .count()
    };
    let tasks_paused = move || {
        tasks
            .get()
            .iter()
            .filter(|task| matches!(task.task_status, TaskStatus::Paused))
            .count()
    };

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    {/*
                    <button class="back-btn" on:click=on_back>"← Back"</button>
                    */}
                    <a href=move || format!("/teams/{}", team_id) class="page-title">
                        {move || {
                            current_team()
                                .map(|team| format!("{} - Tasks", team.team_name))
                                .unwrap_or_else(|| "Team Tasks".to_string())
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

                <Show
                    when=move || !loading.get()
                    fallback=move || {
                        view! {
                            <Loading variant=LoadingVariant::Spinner label="Loading tasks...".to_string() />
                        }
                    }
                >
                    <Show
                        when=move || tasks_error.get().is_none()
                        fallback=move || {
                            view! {
                                <Card title="Error".to_string()>
                                    <p class="auth-error">
                                        {tasks_error.get().unwrap_or_else(|| "Unknown error".to_string())}
                                    </p>
                                </Card>
                            }
                        }
                    >
                        <Card title="Tasks".to_string() subtitle="Team task list".to_string()>
                            {move || {
                                if tasks.get().is_empty() {
                                    view! { <p class="empty-text">"No tasks yet for this team."</p> }.into_any()
                                } else {
                                    let task_items = tasks
                                        .get()
                                        .into_iter()
                                        .map(|task| {
                                            let _task_id = task.task_id;
                                            let name = task.task_name.clone();
                                            let desc = task.task_description.clone().unwrap_or_else(|| "No description".to_string());
                                            let status = status_text(&task.task_status).to_string();
                                            let status_class = status_color(&task.task_status);
                                            let created = format_timestamp(task.task_create_time);

                                            view! {
                                                <div class="team-task-card">
                                                    <div class="team-task-card-header">
                                                        <h4 class="team-task-card-title">{name}</h4>
                                                        <span class=format!("task-status-badge team-task-status {}", status_class)>
                                                            {status}
                                                        </span>
                                                    </div>
                                                    <p class="team-task-card-desc">{desc}</p>
                                                    <div class="team-task-card-footer">
                                                        <span class="team-task-meta-item">
                                                            <span class="team-task-meta-label">"Created: "</span>
                                                            <span>{created}</span>
                                                        </span>
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    view! { <div class="team-task-list">{task_items}</div> }.into_any()
                                }
                            }}
                        </Card>
                    </Show>
                </Show>
            </Show>
        </div>
    }
}
