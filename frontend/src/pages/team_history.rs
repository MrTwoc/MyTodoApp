use crate::api::team::{get_team, get_team_logs};
use crate::components::card::Card;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::{use_api_client, use_team_store};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

fn format_timestamp(ts: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((ts * 1000) as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, minutes)
}

fn format_action(action: &str) -> String {
    match action {
        "MemberJoined" => "成员加入".to_string(),
        "MemberLeft" => "成员离开".to_string(),
        "MemberRemoved" => "成员被移除".to_string(),
        "MemberRoleChanged" => "成员角色变更".to_string(),
        "TeamCreated" => "团队创建".to_string(),
        "TeamUpdated" => "团队信息更新".to_string(),
        "TeamClosed" => "团队关闭".to_string(),
        "SubTeamCreated" => "子团队创建".to_string(),
        "SubTeamDeleted" => "子团队删除".to_string(),
        "RequestApproved" => "申请通过".to_string(),
        "RequestRejected" => "申请拒绝".to_string(),
        "TaskCreated" => "任务创建".to_string(),
        "TaskCompleted" => "任务完成".to_string(),
        "TaskDeleted" => "任务删除".to_string(),
        _ => action.to_string(),
    }
}

#[component]
pub fn TeamHistoryPage() -> impl IntoView {
    #[derive(Clone)]
    struct TeamLog {
        log_id: u64,
        team_id: u64,
        operator_id: u64,
        action: String,
        target_type: String,
        target_id: Option<u64>,
        details: Option<String>,
        created_at: i64,
        _ip_address: Option<String>,
    }

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
    let (logs, set_logs) = signal(Vec::<TeamLog>::new());
    let (loading, set_loading) = signal(true);
    let (logs_error, set_logs_error) = signal(Option::<String>::None);

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
        let set_logs = set_logs;
        let set_loading = set_loading;
        let set_logs_error = set_logs_error;

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

            match get_team_logs(&client, team_id).await {
                Ok(log_list) => {
                    let parsed_logs: Vec<TeamLog> = log_list
                        .into_iter()
                        .map(|v| {
                            let log_id = v.get("log_id").and_then(|x| x.as_u64()).unwrap_or(0);
                            let team_id = v.get("team_id").and_then(|x| x.as_u64()).unwrap_or(0);
                            let operator_id = v.get("operator_id").and_then(|x| x.as_u64()).unwrap_or(0);
                            let action = v.get("action").and_then(|x| x.as_str()).unwrap_or("").to_string();
                            let target_type = v.get("target_type").and_then(|x| x.as_str()).unwrap_or("").to_string();
                            let target_id = v.get("target_id").and_then(|x| x.as_u64());
                            let details = v.get("details").and_then(|x| x.as_str()).map(|s| s.to_string());
                            let created_at = v.get("created_at").and_then(|x| x.as_i64()).unwrap_or(0);
                            let ip_address = v.get("ip_address").and_then(|x| x.as_str()).map(|s| s.to_string());
                            TeamLog {
                                log_id,
                                team_id,
                                operator_id,
                                action,
                                target_type,
                                target_id,
                                details,
                                created_at,
                                _ip_address: ip_address,
                            }
                        })
                        .collect();
                    set_logs.set(parsed_logs);
                }
                Err(e) => {
                    set_logs_error.set(Some(e.message));
                }
            }

            set_loading.set(false);
        });
    });

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    <button class="back-btn" on:click=on_back>"← Back"</button>
                    <a href=move || format!("/teams/{}", team_id) class="page-title">
                        {move || {
                            current_team()
                                .map(|team| format!("{} - History", team.team_name))
                                .unwrap_or_else(|| "Team History".to_string())
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
                <Show
                    when=move || !loading.get()
                    fallback=move || {
                        view! {
                            <Loading variant=LoadingVariant::Spinner label="Loading...".to_string() />
                        }
                    }
                >
                    <Show
                        when=move || logs_error.get().is_none()
                        fallback=move || {
                            view! {
                                <Card title="Error".to_string()>
                                    <p class="auth-error">
                                        {logs_error.get().unwrap_or_else(|| "Unknown error".to_string())}
                                    </p>
                                </Card>
                            }
                        }
                    >
                        <Card title="Team History".to_string() subtitle="Modification history".to_string()>
                            {move || {
                                if logs.get().is_empty() {
                                    view! { <p class="empty-text">"No history records yet."</p> }.into_any()
                                } else {
                                    let log_items = logs
                                        .get()
                                        .into_iter()
                                        .map(|log| {
                                            let time = format_timestamp(log.created_at);
                                            let action_text = format_action(&log.action);
                                            let operator = log.operator_id.to_string();
                                            let details = log.details.clone().unwrap_or_default();
                                            
                                            view! {
                                                <div class="team-history-item">
                                                    <div class="team-history-time">{time}</div>
                                                    <div class="team-history-content">
                                                        <span class="team-history-operator">"User "{operator}</span>
                                                        <span class="team-history-action">{action_text}</span>
                                                        {if !details.is_empty() {
                                                            view! { <span class="team-history-details">{details}</span> }.into_any()
                                                        } else {
                                                            view! { () }.into_any()
                                                        }}
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    view! { <div class="team-history-list">{log_items}</div> }.into_any()
                                }
                            }}
                        </Card>
                    </Show>
                </Show>
            </Show>
        </div>
    }
}