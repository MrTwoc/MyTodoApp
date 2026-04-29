use crate::api::group::{
    add_group_member, delete_group, get_group, leave_group, remove_group_member,
    AddMemberRequest, Group as ApiGroup,
};
use crate::api::task::{assign_task_to_group, list_tasks, unassign_task_from_group};
use crate::api::team::get_members;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::task_store::Task;
use crate::store::team_store::TeamMember;
use crate::store::{use_api_client, use_team_store, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

fn format_timestamp(ts: i64) -> String {
    let date =
        js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((ts * 1000) as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn priority_label(p: u8) -> &'static str {
    match p {
        1 => "P1",
        2 => "P2",
        3 => "P3",
        _ => "P4",
    }
}

fn status_class(s: &str) -> &'static str {
    match s {
        "todo" => "status-todo",
        "in_progress" => "status-in-progress",
        "review" => "status-review",
        "done" => "status-done",
        _ => "status-todo",
    }
}

// ---------------------------------------------------------------------------
// MemberItem
// ---------------------------------------------------------------------------
#[component]
fn MemberItem(
    member: TeamMember,
    is_gl: bool,
    current_user_id: u64,
    group_leader_id: u64,
    loading: bool,
    on_kick: Callback<(u64,)>,
) -> impl IntoView {
    let name = member
        .username
        .clone()
        .unwrap_or_else(|| member.user_id.to_string());
    let is_leader = member.user_id == group_leader_id;
    let can_kick = is_gl && member.user_id != current_user_id && !is_leader;

    view! {
        <div class="member-item">
            <div class="member-info">
                <span class="member-name">{name}</span>
                <Show when=move || is_leader>
                    <span class="member-role-badge">"组长"</span>
                </Show>
            </div>
            <Show when=move || can_kick>
                <Button
                    variant=ButtonVariant::Danger
                    size=ButtonSize::Sm
                    disabled=loading
                    on_click=Callback::from({
                        let cb = on_kick.clone();
                        let uid = member.user_id;
                        move |_: ev::MouseEvent| cb.run((uid,))
                    })
                >
                    "踢出"
                </Button>
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// InviteItem
// ---------------------------------------------------------------------------
#[component]
fn InviteItem(
    member: TeamMember,
    loading: bool,
    on_invite: Callback<(u64,)>,
) -> impl IntoView {
    let name = member
        .username
        .clone()
        .unwrap_or_else(|| member.user_id.to_string());

    view! {
        <div class="invite-member-item">
            <span class="member-name">{name}</span>
            <Button
                variant=ButtonVariant::Primary
                size=ButtonSize::Sm
                disabled=loading
                on_click=Callback::from({
                    let cb = on_invite.clone();
                    let uid = member.user_id;
                    move |_: ev::MouseEvent| cb.run((uid,))
                })
            >
                "邀请"
            </Button>
        </div>
    }
}

// ---------------------------------------------------------------------------
// MembersTab — 成员 Tab 内容（独立组件，无 FnOnce 问题）
// ---------------------------------------------------------------------------
#[component]
fn MembersTab(
    members: Vec<TeamMember>,
    inviteable: Vec<TeamMember>,
    current_user_id: u64,
    group_leader_id: u64,
    is_group_leader: bool,
    action_loading: bool,
    on_kick: Callback<(u64,)>,
    on_invite: Callback<(u64,)>,
    on_open_invite: Callback<(ev::MouseEvent,)>,
) -> impl IntoView {
    let inviteable_empty = inviteable.is_empty();

    view! {
        <div class="group-detail-tab-content">
            <div class="member-list">
                {members.into_iter().map(|member| {
                    view! {
                        <MemberItem
                            member=member
                            is_gl=is_group_leader
                            current_user_id=current_user_id
                            group_leader_id=group_leader_id
                            loading=action_loading
                            on_kick=on_kick.clone()
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>

            <Show when=move || is_group_leader && !inviteable_empty>
                <div class="invite-section">
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=on_open_invite
                    >
                        "邀请成员"
                    </Button>
                </div>
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// TaskItem
// ---------------------------------------------------------------------------
#[component]
fn TaskItem(
    task: Task,
    current_user_id: u64,
    is_group_leader: bool,
    action_loading: bool,
    on_unassign: Callback<(u64,)>,
) -> impl IntoView {
    let priority_class = format!("priority-badge priority-{}", task.task_priority.min(3));
    let status_str = format!("{:?}", task.task_status);
    let status_class_str = format!("status-badge {}", status_class(&status_str));
    let is_leader = task.task_leader_id == current_user_id;

    view! {
        <div class="task-item">
            <div class="task-item-main">
                <span class={priority_class}>{priority_label(task.task_priority)}</span>
                <span class="task-name">{task.task_name.clone()}</span>
                <span class={status_class_str}>{status_str}</span>
            </div>
            <div class="task-item-actions">
                <Show when=move || is_leader || is_group_leader>
                    <Button
                        variant=ButtonVariant::Danger
                        size=ButtonSize::Sm
                        disabled=action_loading
                        on_click=Callback::from({
                            let cb = on_unassign.clone();
                            let tid = task.task_id;
                            move |_: ev::MouseEvent| cb.run((tid,))
                        })
                    >
                        "取消指派"
                    </Button>
                </Show>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// AssignTaskItem
// ---------------------------------------------------------------------------
#[component]
fn AssignTaskItem(
    task: Task,
    action_loading: bool,
    on_assign: Callback<(u64,)>,
) -> impl IntoView {
    view! {
        <div class="task-select-item">
            <span class="task-name">{task.task_name.clone()}</span>
            <Button
                variant=ButtonVariant::Primary
                size=ButtonSize::Sm
                disabled=action_loading
                on_click=Callback::from({
                    let cb = on_assign.clone();
                    let tid = task.task_id;
                    move |_: ev::MouseEvent| cb.run((tid,))
                })
            >
                "指派"
            </Button>
        </div>
    }
}

// ---------------------------------------------------------------------------
// TasksTab — 任务 Tab 内容（独立组件，无 FnOnce 问题）
// ---------------------------------------------------------------------------
#[component]
fn TasksTab(
    tasks: Vec<Task>,
    unassigned: Vec<Task>,
    current_user_id: u64,
    is_group_leader: bool,
    action_loading: bool,
    on_assign: Callback<(u64,)>,
    on_unassign: Callback<(u64,)>,
) -> impl IntoView {
// Pre-collect assign task items to avoid into_iter() inside view! closure
    let is_empty = tasks.is_empty();
    let tasks_for_list = tasks.clone();
    let tasks_for_assign_list: Vec<Task> = tasks
        .iter()
        .filter(|t| t.task_group_id.is_none())
        .cloned()
        .collect();
    let tasks_assign_len = tasks_for_assign_list.len();
    let can_show_assign_section = is_group_leader && tasks_assign_len > 0;

    view! {
        <div class="group-detail-tab-content">
            <Show when=move || is_empty>
                <p class="empty-hint">"暂无指派给本小组的任务"</p>
            </Show>
            <div class="task-list">
                <For each=move || tasks_for_list.clone() key=|t| t.task_id let:task>
                    <TaskItem
                        task=task.clone()
                        is_group_leader=is_group_leader
                        current_user_id=current_user_id
                        action_loading=action_loading
                        on_unassign=on_unassign.clone()
                    />
                </For>
            </div>

            <Show when=move || can_show_assign_section>
                <div class="assign-task-section">
                    <p class="section-label">"指派新任务给本小组"</p>
                    <div class="task-select-list">
                        {tasks_for_assign_list.iter().map(|t| {
                            let tid = t.task_id;
                            view! {
                                <div class="task-select-item">
                                    <span class="task-name">{t.task_name.clone()}</span>
                                    <Button
                                        variant=ButtonVariant::Primary
                                        size=ButtonSize::Sm
                                        disabled=action_loading
                                        on_click=Callback::from({
                                            let cb = on_assign.clone();
                                            move |_: ev::MouseEvent| cb.run((tid,))
                                        })
                                    >
                                        "指派"
                                    </Button>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                </div>
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// GroupManagePage — 小组详情管理页面
// ---------------------------------------------------------------------------
#[component]
pub fn GroupManagePage() -> impl IntoView {
    let params = use_params_map();
    let team_id = params
        .get()
        .get("team_id")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);
    let group_id = params
        .get()
        .get("group_id")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);

    let client = use_api_client();
    let user_store = use_user_store();
    let current_user_id = user_store.user_id().unwrap_or(0);

    let (loading, set_loading) = signal(true);
    let (page_error, set_page_error) = signal(Option::<String>::None);
    let (group, set_group) = signal(Option::<ApiGroup>::None);
    let (team_members, set_team_members) = signal(Vec::<TeamMember>::new());
    let (all_tasks, set_all_tasks) = signal(Vec::<Task>::new());

    let (active_tab, set_active_tab) = signal(0u8);
    let (action_loading, set_action_loading) = signal(false);
    let (action_error, set_action_error) = signal(Option::<String>::None);
    let (confirm_modal, set_confirm_modal) = signal(Option::<String>::None);
    let (confirm_action, set_confirm_action) = signal(String::new());
    let (invite_modal, set_invite_modal) = signal(false);

    let is_group_leader = move || {
        group.get().map(|g| g.group_leader_id == current_user_id).unwrap_or(false)
    };
    let is_gl = is_group_leader();
    let show_group_detail = move || !loading.get() && group.get().is_some();
    let show_gl_badge = is_gl;
    let show_delete_btn = is_gl;
    let show_leave_btn = !is_gl;
    let group_members = move || group.get().map(|g| g.group_members.clone()).unwrap_or_default();
    let inviteable_members = move || team_members
        .get()
        .iter()
        .filter(|m| {
            !group_members()
                .iter()
                .any(|gm| gm.user_id == m.user_id)
                && m.user_id != group.get().map(|g| g.group_leader_id).unwrap_or(0)
        })
        .cloned()
        .collect();
    let group_name = move || group.get().map(|g| g.group_name.clone()).unwrap_or_default();
    let group_desc_check = move || group.get().and_then(|g| g.group_description.clone()).is_some();
    let group_desc = move || group.get().and_then(|g| g.group_description.clone());
    let group_desc_value = move || group.get().and_then(|g| g.group_description.clone()).unwrap_or_default();
    let group_default_ts = 0i64;
    let group_ts = group.get().map(|g| g.group_create_time).unwrap_or(group_default_ts);
    let group_leader_id = move || group.get().map(|g| g.group_leader_id).unwrap_or(0);
    let group_tasks = move || all_tasks
        .get()
        .into_iter()
        .filter(|t| t.task_group_id == Some(group_id))
        .collect();
    let unassigned_tasks = move || all_tasks
        .get()
        .iter()
        .filter(|t| t.task_group_id.is_none())
        .cloned()
        .collect();

    // Load data
    let do_load: Callback<((),), ()> = {
        let client = client.clone();
        Callback::from(move |(): ()| {
            let client = client.clone();
            let set_loading = set_loading.clone();
            let set_page_error = set_page_error.clone();
            let set_group = set_group.clone();
            let set_team_members = set_team_members.clone();
            let set_all_tasks = set_all_tasks.clone();

            set_loading.set(true);
            set_page_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match get_group(&client, group_id).await {
                    Ok(g) => set_group.set(Some(g)),
                    Err(e) => {
                        set_page_error.set(Some(e.message));
                        set_loading.set(false);
                        return;
                    }
                }
                match get_members(&client, team_id).await {
                    Ok(members) => set_team_members.set(members),
                    Err(e) => {
                        set_page_error.set(Some(e.message));
                        set_loading.set(false);
                        return;
                    }
                }
                match list_tasks(&client, 1, 100, None, Some(team_id)).await {
                    Ok(crate::api::task::TaskListResponse { tasks, .. }) => set_all_tasks.set(tasks),
                    Err(e) => {
                        set_page_error.set(Some(e.message));
                        set_loading.set(false);
                        return;
                    }
                }
                set_loading.set(false);
            });
        })
    };

    Effect::new(move |_| {
        do_load.run(((),));
    });

    // Tab handlers
    let tab0_handler = move |_: ev::MouseEvent| set_active_tab.set(0);
    let tab1_handler = move |_: ev::MouseEvent| set_active_tab.set(1);
    let open_invite_handler = move |_: ev::MouseEvent| set_invite_modal.set(true);

    // Kick member
    let on_kick_member = {
        let client = client.clone();
        Callback::from(move |user_id: u64| {
            let client = client.clone();
            let do_load = do_load.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match remove_group_member(&client, group_id, user_id).await {
                    Ok(()) => {
                        set_action_loading.set(false);
                        do_load.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // Delete group
    let on_delete_group = {
        Callback::from(move |_: ev::MouseEvent| {
            set_confirm_modal.set(Some("确定要解散该小组吗？该操作不可恢复。".to_string()));
            set_confirm_action.set("delete_group".to_string());
        })
    };

    // Leave group
    let on_leave_group = {
        Callback::from(move |_: ev::MouseEvent| {
            set_confirm_modal.set(Some("确定要退出该小组吗？".to_string()));
            set_confirm_action.set("leave_group".to_string());
        })
    };

    // Confirm action
    let on_confirm_yes: Callback<(ev::MouseEvent,)> = {
        let client = client.clone();
        Callback::from(move |_: ev::MouseEvent| {
            set_confirm_modal.set(None);
            let action = confirm_action.get();
            set_action_loading.set(true);
            set_action_error.set(None);

            if action == "delete_group" {
                let client = client.clone();
                let navigate = use_navigate();
                wasm_bindgen_futures::spawn_local(async move {
                    match delete_group(&client, group_id).await {
                        Ok(()) => {
                            set_action_loading.set(false);
                            navigate(&format!("/teams/{}/groups", team_id), Default::default());
                        }
                        Err(e) => {
                            set_action_error.set(Some(e.message));
                            set_action_loading.set(false);
                        }
                    }
                });
            } else if action == "leave_group" {
                let client = client.clone();
                let navigate = use_navigate();
                wasm_bindgen_futures::spawn_local(async move {
                    match leave_group(&client, group_id, current_user_id).await {
                        Ok(()) => {
                            set_action_loading.set(false);
                            navigate(&format!("/teams/{}/groups", team_id), Default::default());
                        }
                        Err(e) => {
                            set_action_error.set(Some(e.message));
                            set_action_loading.set(false);
                        }
                    }
                });
            }
        })
    };

    // Assign task
    let on_assign_task = {
        let client = client.clone();
        Callback::from(move |task_id: u64| {
            let client = client.clone();
            let do_load = do_load.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match assign_task_to_group(&client, task_id, group_id).await {
                    Ok(_) => {
                        set_action_loading.set(false);
                        do_load.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // Unassign task
    let on_unassign_task = {
        let client = client.clone();
        Callback::from(move |task_id: u64| {
            let client = client.clone();
            let do_load = do_load.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match unassign_task_from_group(&client, task_id).await {
                    Ok(_) => {
                        set_action_loading.set(false);
                        do_load.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // Invite member
    let on_invite = {
        let client = client.clone();
        Callback::from(move |user_id: u64| {
            let client = client.clone();
            let do_load = do_load.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let req = AddMemberRequest { user_id, level: 1 };
                match add_group_member(&client, group_id, &req).await {
                    Ok(()) => {
                        set_action_loading.set(false);
                        set_invite_modal.set(false);
                        do_load.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    view! {
        <div class="team-groups-page">
            <TeamModuleNav team_id />

            <div class="page-container">
                <div class="page-back-row">
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=Callback::from({
                            let tid = team_id;
                            move |_: ev::MouseEvent| {
                                let navigate = use_navigate();
                                navigate(&format!("/teams/{}/groups", tid), Default::default());
                            }
                        })
                    >
                        "← 返回小组列表"
                    </Button>
                </div>

                <Show when=move || loading.get()>
                    <Loading variant=LoadingVariant::Spinner />
                </Show>

                <Show when=move || page_error.get().is_some()>
                    <p class="page-error">{page_error.get().unwrap_or_default()}</p>
                </Show>

                <Show when=move || !loading.get() && group.get().is_some()>
                    <div class="group-detail-card">
                        <div class="group-detail-header">
                            <div class="group-detail-title-row">
                                <h2 class="group-detail-title">{group_name.clone()}</h2>
                                <Show when=move || show_gl_badge>
                                    <span class="group-card-badge">"组长"</span>
                                </Show>
                            </div>
                            <Show when=move || group_desc_check()>
                                <p class="group-detail-desc">{group_desc_value().clone()}</p>
                            </Show>
                            <div class="group-detail-meta">
                                <span class="group-meta-item">
                                    <span class="group-meta-label">"创建时间: "</span>
                                    <span>{format_timestamp(group_ts)}</span>
                                </span>
                            </div>
                        </div>

                        <div class="group-detail-actions">
                            <Show when=move || show_delete_btn>
                                <Button variant=ButtonVariant::Danger size=ButtonSize::Sm on_click=on_delete_group>
                                    "解散小组"
                                </Button>
                            </Show>
                            <Show when=move || show_leave_btn>
                                <Button variant=ButtonVariant::Secondary size=ButtonSize::Sm on_click=on_leave_group>
                                    "退出小组"
                                </Button>
                            </Show>
                        </div>

                        <div class="group-detail-tabs">
                            <button
                                class=move || {
                                    if active_tab.get() == 0 {
                                        "tab-btn tab-btn-active"
                                    } else {
                                        "tab-btn"
                                    }
                                }
                                on:click=tab0_handler
                            >
                                {"成员 (".to_string() + &group_members().len().to_string() + ")"}
                            </button>
                            <button
                                class=move || {
                                    if active_tab.get() == 1 {
                                        "tab-btn tab-btn-active"
                                    } else {
                                        "tab-btn"
                                    }
                                }
                                on:click=tab1_handler
                            >
                                {"任务 ("}
                            </button>
                        </div>

                        <Show when=move || active_tab.get() == 0>
                            <MembersTab
                                members=group_members().clone()
                                inviteable=inviteable_members()
                                current_user_id
                                group_leader_id=group_leader_id()
                                is_group_leader=is_gl
                                action_loading=action_loading.get()
                                on_kick=on_kick_member.clone()
                                on_invite=on_invite.clone()
                                on_open_invite=Callback::from(open_invite_handler.clone())
                            />
                        </Show>

                        <Show when=move || active_tab.get() == 1>
                            <TasksTab
                                tasks=group_tasks()
                                unassigned=unassigned_tasks()
                                current_user_id
                                is_group_leader=is_gl
                                action_loading=action_loading.get()
                                on_assign=on_assign_task.clone()
                                on_unassign=on_unassign_task.clone()
                            />
                        </Show>

                        <Show when=move || action_error.get().is_some()>
                            <p class="group-card-error">{action_error.get().unwrap_or_default()}</p>
                        </Show>
                    </div>
                </Show>
            </div>

            // Confirm modal
            <Show when=move || confirm_modal.get().is_some()>
                <div class="confirm-overlay">
                    <div class="confirm-box">
                        <p class="confirm-msg">{confirm_modal.get().unwrap_or_default()}</p>
                        <div class="confirm-actions">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=Callback::from(move |_: ev::MouseEvent| {
                                    set_confirm_modal.set(None);
                                })
                            >
                                "取消"
                            </Button>
                            <Button
                                variant=ButtonVariant::Danger
                                size=ButtonSize::Sm
                                disabled=action_loading.get()
                                on_click=on_confirm_yes.clone()
                            >
                                "确认"
                            </Button>
                        </div>
                    </div>
                </div>
            </Show>

            // Invite modal
            <Show when=move || invite_modal.get()>
                <div class="confirm-overlay">
                    <div class="confirm-box invite-box">
                        <h3 class="confirm-title">"邀请成员"</h3>
                        <div class="invite-member-list">
                            {inviteable_members()
                                .iter()
                                .map(|m| {
                                    view! {
                                        <div class="invite-member-item">
                                            <span>{m.username.clone().unwrap_or_default()}</span>
                                            <Button
                                                variant=ButtonVariant::Primary
                                                size=ButtonSize::Sm
                                                on_click=Callback::from({
                                                    let uid = m.user_id;
                                                    let cb = on_invite.clone();
                                                    move |_: ev::MouseEvent| {
                                                        cb.run((uid,));
                                                    }
                                                })
                                            >
                                                "邀请"
                                            </Button>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()}
                        </div>
                        <div class="confirm-actions">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=Callback::from(move |_: ev::MouseEvent| {
                                    set_invite_modal.set(false);
                                })
                            >
                                "关闭"
                            </Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
