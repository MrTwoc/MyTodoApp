use crate::api::team::{get_team, get_team_logs};
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::{use_api_client, use_team_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

fn format_timestamp(ts: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((ts * 1000) as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let seconds = date.get_seconds();
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn format_relative_time(ts: i64) -> String {
    let now = (js_sys::Date::now() / 1000.0) as i64;
    let diff = now - ts;
    if diff < 60 {
        format!("{}秒前", diff)
    } else if diff < 3600 {
        format!("{}分钟前", diff / 60)
    } else if diff < 86400 {
        format!("{}小时前", diff / 3600)
    } else {
        format!("{}天前", diff / 86400)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ActionCategory {
    Member,
    Team,
    Task,
    Other,
}

fn action_category(action: &str) -> ActionCategory {
    match action {
        "MemberJoined" | "MemberLeft" | "MemberRemoved" | "MemberRoleChanged" => ActionCategory::Member,
        "TeamCreated" | "TeamUpdated" | "TeamClosed" | "SubTeamCreated" | "SubTeamDeleted" => ActionCategory::Team,
        "TaskCreated" | "TaskCompleted" | "TaskDeleted" => ActionCategory::Task,
        _ => ActionCategory::Other,
    }
}

fn action_color(category: ActionCategory) -> &'static str {
    match category {
        ActionCategory::Member => "action-member",
        ActionCategory::Team => "action-team",
        ActionCategory::Task => "action-task",
        ActionCategory::Other => "action-other",
    }
}

fn format_action(action: &str) -> String {
    match action {
        "MemberJoined" => "成员加入".to_string(),
        "MemberLeft" => "成员离开".to_string(),
        "MemberRemoved" => "成员被移除".to_string(),
        "MemberRoleChanged" => "角色变更".to_string(),
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

fn action_icon(category: ActionCategory) -> &'static str {
    match category {
        ActionCategory::Member => "+",
        ActionCategory::Team => "*",
        ActionCategory::Task => "!",
        ActionCategory::Other => "~",
    }
}

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

#[component]
pub fn TeamHistoryPage() -> impl IntoView {
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

    let team_name = move || {
        current_team()
            .map(|team| team.team_name.clone())
            .unwrap_or_else(|| "团队历史".to_string())
    };

    let on_back = {
        let n = navigate.clone();
        let tid = team_id;
        move |_: ev::MouseEvent| {
            n(&format!("/teams/{}", tid), Default::default());
        }
    };

    Effect::new(move |_| {
        if team_id == 0 {
            set_page_error.set(Some("无效的团队 ID".to_string()));
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
                            let operator_id =
                                v.get("operator_id").and_then(|x| x.as_u64()).unwrap_or(0);
                            let action = v
                                .get("action")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let target_type = v
                                .get("target_type")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let target_id = v.get("target_id").and_then(|x| x.as_u64());
                            let details = v
                                .get("details")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string());
                            let created_at =
                                v.get("created_at").and_then(|x| x.as_i64()).unwrap_or(0);
                            let ip_address = v
                                .get("ip_address")
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string());
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

    // Precompute per-category counts for stats bar
    let member_count = move || {
        logs.get().iter().filter(|l| action_category(&l.action) == ActionCategory::Member).count()
    };
    let team_count = move || {
        logs.get().iter().filter(|l| action_category(&l.action) == ActionCategory::Team).count()
    };
    let task_count = move || {
        logs.get().iter().filter(|l| action_category(&l.action) == ActionCategory::Task).count()
    };
    let total_count = move || logs.get().len();

    view! {
        <div class="team-history-page">
            <TeamModuleNav team_id />

            <div class="page-container">
                <div class="th-back-row">
                    <button class="th-back-btn" on:click=on_back>
                        <span>"←"</span>
                        "返回团队"
                    </button>
                </div>

                <Show when=move || page_error.get().is_some()>
                    <p class="page-error">{page_error.get().unwrap_or_default()}</p>
                </Show>

                <Show when=move || !loading.get() && page_error.get().is_none()>
                    // Hero
                    <div class="th-hero">
                        <div class="th-hero-bg-glow"></div>
                        <div class="th-hero-icon-badge">
                            <span class="th-hero-letter">"H"</span>
                        </div>
                        <div class="th-hero-info">
                            <h1 class="th-hero-title">{team_name.clone()}</h1>
                            <p class="th-hero-subtitle">"团队修改历史"</p>
                            <div class="th-hero-team-chip">
                                <span class="th-chip-dot"></span>
                                <span>"{team_name.clone()} 变更记录"</span>
                            </div>
                        </div>
                        <div class="th-hero-decoration">
                            <span>"H"</span>
                        </div>
                    </div>

                    // Stats bar
                    <div class="th-stats-row">
                        <div class="th-stat-card">
                            <div class="th-stat-top">
                                <span class="th-stat-num">{total_count()}</span>
                                <span class="th-stat-icon">"~"</span>
                            </div>
                            <span class="th-stat-label">"总记录数"</span>
                            <div class="th-stat-bar">
                                <div class="th-stat-bar-fill" style="width: 100%"></div>
                            </div>
                        </div>
                        <div class="th-stat-card">
                            <div class="th-stat-top">
                                <span class="th-stat-num purple">{member_count()}</span>
                                <span class="th-stat-icon">"+"</span>
                            </div>
                            <span class="th-stat-label">"成员变更"</span>
                            <div class="th-stat-bar">
                                <div class="th-stat-bar-fill purple" style=format!("width: {}%", if total_count() > 0 { (member_count() * 100 / total_count()).max(1) } else { 0 })></div>
                            </div>
                        </div>
                        <div class="th-stat-card">
                            <div class="th-stat-top">
                                <span class="th-stat-num blue">{team_count()}</span>
                                <span class="th-stat-icon">"*"</span>
                            </div>
                            <span class="th-stat-label">"团队变更"</span>
                            <div class="th-stat-bar">
                                <div class="th-stat-bar-fill blue" style=format!("width: {}%", if total_count() > 0 { (team_count() * 100 / total_count()).max(1) } else { 0 })></div>
                            </div>
                        </div>
                        <div class="th-stat-card">
                            <div class="th-stat-top">
                                <span class="th-stat-num green">{task_count()}</span>
                                <span class="th-stat-icon">"!"</span>
                            </div>
                            <span class="th-stat-label">"任务变更"</span>
                            <div class="th-stat-bar">
                                <div class="th-stat-bar-fill green" style=format!("width: {}%", if total_count() > 0 { (task_count() * 100 / total_count()).max(1) } else { 0 })></div>
                            </div>
                        </div>
                    </div>

                    // Error state
                    <Show when=move || logs_error.get().is_some()>
                        <p class="page-error">{logs_error.get().unwrap_or_default()}</p>
                    </Show>

                    // Timeline
                    <Show
                        when=move || !logs.get().is_empty()
                        fallback=move || {
                            view! {
                                <div class="th-empty">
                                    <div class="th-empty-glyph">"~"</div>
                                    <p class="th-empty-title">"暂无修改记录"</p>
                                    <p class="th-empty-hint">"团队的所有操作和变更都将显示在这里"</p>
                                </div>
                            }
                        }
                    >
                        <div class="th-timeline-wrap">
                            <div class="th-timeline-line"></div>
                            {logs.get().into_iter().map(|log| {
                                let cat = action_category(&log.action);
                                let color_class = action_color(cat);
                                let icon = action_icon(cat);
                                let time_str = format_timestamp(log.created_at);
                                let rel_time = format_relative_time(log.created_at);
                                let action_text = format_action(&log.action);
                                let operator = log.operator_id.to_string();
                                let details = log.details.clone().unwrap_or_default();
                                let details_empty = details.is_empty();

                                view! {
                                    <div class="timeline-item">
                                        <div class="timeline-icon-wrap">
                                            <div class=format!("timeline-icon {}", color_class)>
                                                {icon}
                                            </div>
                                        </div>
                                        <div class="timeline-card">
                                            <div class="timeline-card-header">
                                                <div class="timeline-action-tags">
                                                    <span class=format!("timeline-action-tag {}", color_class)>
                                                        {action_text}
                                                    </span>
                                                </div>
                                                <div class="timeline-time-wrap">
                                                    <span class="timeline-time-rel">{rel_time}</span>
                                                    <span class="timeline-time-abs">{time_str}</span>
                                                </div>
                                            </div>
                                            <div class="timeline-card-body">
                                                <div class="timeline-operator">
                                                    <div class="timeline-avatar">{operator.chars().next().unwrap_or('U')}</div>
                                                    <span class="timeline-operator-label">"操作者 ID: "{operator}</span>
                                                </div>
                                                <Show when=move || !details_empty>
                                                    <div class="timeline-details">
                                                        <span class="timeline-details-text">{details.clone()}</span>
                                                    </div>
                                                </Show>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Show>
                </Show>

                <Show when=move || loading.get()>
                    <Loading variant=LoadingVariant::Spinner label="加载中...".to_string() />
                </Show>
            </div>
        </div>
    }
}
