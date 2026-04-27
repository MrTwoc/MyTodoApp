use crate::api::group::{
    add_group_member, create_group, delete_group, get_group, leave_group, list_groups,
    remove_group_member, update_group, CreateGroupRequest, Group as ApiGroup,
};
use crate::api::task::{assign_task_to_group, list_tasks, unassign_task_from_group};
use crate::api::team::get_members;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::Input;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::modal::Modal;
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::task_store::Task;
use crate::store::team_store::TeamMember;
use crate::store::{use_api_client, use_team_store, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

fn format_timestamp(ts: i64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64((ts * 1000) as f64));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

/// 任务优先级标签
fn priority_label(p: u8) -> &'static str {
    match p {
        1 => "P1",
        2 => "P2",
        3 => "P3",
        _ => "P4",
    }
}

/// 任务状态颜色类名
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
// GroupDetailModal — 小组详情弹窗（成员列表 / 任务列表 / 管理操作）
// ---------------------------------------------------------------------------
#[component]
fn GroupDetailModal(
    group: ApiGroup,
    team_id: u64,
    current_user_id: u64,
    is_group_leader: bool,
    team_members: Vec<TeamMember>,
    team_tasks: Vec<Task>,
    on_close: Callback<(ev::MouseEvent,)>,
    on_action: Callback<((),)>,
) -> impl IntoView {
    let client = use_api_client();

    // Pre-compute derived values BEFORE any consumption of team_tasks
    let group_tasks_empty = team_tasks.is_empty();
    // 预计算 group_description 避免 move 闭包冲突
    let group_desc_check = group.group_description.is_some();
    let group_desc = group.group_description.clone();

    // 消费 team_tasks / team_members（在预计算之后）
    let group_tasks: Vec<Task> = team_tasks
        .into_iter()
        .filter(|t| t.task_group_id == Some(group.group_id))
        .collect();

    let inviteable_members: Vec<TeamMember> = team_members
        .iter()
        .filter(|m| {
            !group.group_members.iter().any(|gm| gm.user_id == m.user_id)
                && m.user_id != group.group_leader_id
        })
        .cloned()
        .collect();

    // Pre-compute For loop iterators (avoid move in view! closures)
    let members_for_iter = group.group_members.clone();
    let inviteable_members_empty = inviteable_members.is_empty();
    // Tab state
    let (active_tab, set_active_tab) = signal(0u8); // 0=成员 1=任务
    let is_team_leader = {
        let g = group.clone();
        move || g.group_leader_id == current_user_id
    };
    let (action_loading, set_action_loading) = signal(false);
    let (action_error, set_action_error) = signal(Option::<String>::None);
    let (confirm_modal, set_confirm_modal) = signal(Option::<String>::None); // Some(msg) = show confirm
    let (confirm_action, set_confirm_action) = signal(String::new());
    let (invite_modal, set_invite_modal) = signal(false);

    // ---- 解散小组 ----
    let on_delete_group = {
        let client = client.clone();
        let group_id = group.group_id;
        Callback::from(move |_: ev::MouseEvent| {
            set_confirm_modal.set(Some("确定要解散该小组吗？该操作不可恢复。".to_string()));
            set_confirm_action.set("delete_group".to_string());
        })
    };

    // ---- 踢出成员 ----
    let on_kick_member = {
        let client = client.clone();
        let group_id = group.group_id;
        Callback::from(move |user_id: u64| {
            let client = client.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match remove_group_member(&client, group_id, user_id).await {
                    Ok(()) => {
                        set_action_loading.set(false);
                        on_action.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // ---- 退出小组 ----
    let on_leave_group = {
        let client = client.clone();
        let group_id = group.group_id;
        Callback::from(move |_: ev::MouseEvent| {
            set_confirm_modal.set(Some("确定要退出该小组吗？".to_string()));
            set_confirm_action.set("leave_group".to_string());
        })
    };

    // ---- 确认操作 ----
    let on_confirm_yes: Callback<(ev::MouseEvent,)> = {
        let client = client.clone();
        let confirm_action = confirm_action.clone();
        let group_id = group.group_id;
        Callback::from(move |_: ev::MouseEvent| {
            set_confirm_modal.set(None);
            let action = confirm_action.get();
            set_action_loading.set(true);
            set_action_error.set(None);

            if action == "delete_group" {
                let client = client.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match delete_group(&client, group_id).await {
                        Ok(()) => {
                            set_action_loading.set(false);
                            on_action.run(((),));
                        }
                        Err(e) => {
                            set_action_error.set(Some(e.message));
                            set_action_loading.set(false);
                        }
                    }
                });
            } else if action == "leave_group" {
                let client = client.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match leave_group(&client, group_id, current_user_id).await {
                        Ok(()) => {
                            set_action_loading.set(false);
                            on_action.run(((),));
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

    // ---- 任务指派 ----
    let on_assign_task = {
        let client = client.clone();
        let group_id = group.group_id;
        Callback::from(move |task_id: u64| {
            let client = client.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match assign_task_to_group(&client, task_id, group_id).await {
                    Ok(_) => {
                        set_action_loading.set(false);
                        on_action.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // ---- 放弃任务（取消指派）----
    let on_unassign_task = {
        let client = client.clone();
        Callback::from(move |task_id: u64| {
            let client = client.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match unassign_task_from_group(&client, task_id).await {
                    Ok(_) => {
                        set_action_loading.set(false);
                        on_action.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // ---- 邀请成员 ----
    let on_invite = {
        let client = client.clone();
        let group_id = group.group_id;
        Callback::from(move |user_id: u64| {
            let client = client.clone();
            set_action_loading.set(true);
            set_action_error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let req = crate::api::group::AddMemberRequest {
                    user_id,
                    level: 1,
                };
                match add_group_member(&client, group_id, &req).await {
                    Ok(()) => {
                        set_action_loading.set(false);
                        set_invite_modal.set(false);
                        on_action.run(((),));
                    }
                    Err(e) => {
                        set_action_error.set(Some(e.message));
                        set_action_loading.set(false);
                    }
                }
            });
        })
    };

    // Tab and UI button handlers (FnMut closures for native buttons)
    let tab0_handler = move |_: ev::MouseEvent| set_active_tab.set(0);
    let tab1_handler = move |_: ev::MouseEvent| set_active_tab.set(1);
    let open_invite_handler = move |_: ev::MouseEvent| set_invite_modal.set(true);


// -----------------------------------------------------------------------------
// MemberListContent
// ---------------------------------------------------------------------------
#[component]
fn MemberListContent(
    members: Vec<TeamMember>,
    is_gl: bool,
    current_user_id: u64,
    group_leader_id: u64,
    loading: bool,
    on_kick: Callback<(u64,)>,
) -> impl IntoView {
    view! {
        <div class="group-detail-tab-content">
            <div class="member-list">
                <For each=move || members.clone() key=|m| m.user_id let:member>
                    <MemberItem
                        member=member.clone()
                        is_gl
                        current_user_id
                        group_leader_id
                        loading
                        on_kick=on_kick.clone()
                    />
                </For>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// TaskListContent — 任务列表（提取为独立组件，解耦 move 闭包）
// ---------------------------------------------------------------------------
#[component]
fn TaskListContent(
    tasks: Vec<Task>,
    is_gl: bool,
    current_user_id: u64,
    loading: bool,
    on_unassign: Callback<(u64,)>,
    on_assign: Callback<(u64,)>,
) -> impl IntoView {
    let is_empty = tasks.is_empty();
    let tasks_for_list = tasks.clone();
    let tasks_for_assign: Vec<Task> = tasks
        .iter()
        .filter(|t| t.task_group_id.is_none())
        .cloned()
        .collect();
    view! {
        <div class="group-detail-tab-content">
            <Show when=move || is_empty>
                <p class="empty-hint">"暂无指派给本小组的任务"</p>
            </Show>
            <div class="task-list">
                <For each=move || tasks_for_list.clone() key=|t| t.task_id let:task>
                    <TaskItem
                        task=task.clone()
                        is_gl
                        current_user_id
                        loading
                        on_unassign=on_unassign.clone()
                    />
                </For>
            </div>
            <Show when=move || is_gl>
                <div class="assign-task-section">
                    <p class="section-label">"指派新任务给本小组"</p>
                    <div class="task-select-list">
                        {tasks_for_assign.iter().map(|t| {
                            view! {
                                <div class="task-select-item">
                                    <span class="task-name">{t.task_name.clone()}</span>
                                    <Button
                                        variant=ButtonVariant::Primary
                                        size=ButtonSize::Sm
                                        disabled=loading
                                        on_click=Callback::from({
                                            let tid = t.task_id;
                                            move |_: ev::MouseEvent| on_assign.clone().run((tid,))
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
// MemberItem — 成员列表项
// -----------------------------------------------------------------------------
#[component]
fn MemberItem(
    member: TeamMember,
    is_gl: bool,
    current_user_id: u64,
    group_leader_id: u64,
    loading: bool,
    on_kick: Callback<(u64,)>,
) -> impl IntoView {
    let name = member.username.clone().unwrap_or_else(|| member.user_id.to_string());
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
// TaskItem — 任务列表项
// ---------------------------------------------------------------------------
#[component]
fn TaskItem(
    task: Task,
    is_gl: bool,
    current_user_id: u64,
    loading: bool,
    on_unassign: Callback<(u64,)>,
) -> impl IntoView {
    let is_leader = task.task_leader_id == current_user_id;
    let p = task.task_priority.min(3);
    let sc = status_class(&format!("{:?}", task.task_status));

    view! {
        <div class="task-item">
            <div class="task-item-main">
                <span class={move || format!("priority-badge priority-{}", p)}>
                    {priority_label(task.task_priority)}
                </span>
                <span class="task-name">{task.task_name.clone()}</span>
                <span class={move || format!("status-badge {}", sc)}>
                    {format!("{:?}", task.task_status)}
                </span>
            </div>
            <div class="task-item-actions">
                <Show when=move || is_leader>
                    <Button
                        variant=ButtonVariant::Danger
                        size=ButtonSize::Sm
                        disabled=loading
                        on_click=Callback::from({
                        let cb = on_unassign.clone();
                        let tid = task.task_id;
                        move |_: ev::MouseEvent| cb.run((tid,))
                    })
                    >
                        "放弃"
                    </Button>
                </Show>
                <Show when=move || is_gl>
                    <Button
                        variant=ButtonVariant::Danger
                        size=ButtonSize::Sm
                        disabled=loading
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
// InviteModalContent — 邀请成员弹窗内容（提取为独立组件，解耦 move 闭包）
// ---------------------------------------------------------------------------
#[component]
fn InviteModalContent(
    members: Vec<TeamMember>,
    loading: bool,
    on_invite: Callback<(u64,)>,
    on_close: Callback<(ev::MouseEvent,)>,
) -> impl IntoView {
    view! {
        <div class="confirm-overlay">
            <div class="confirm-box invite-box">
                <h3 class="confirm-title">"邀请成员"</h3>
                <div class="invite-member-list">
                    <For each=move || members.clone() key=|m| m.user_id let:member>
                        <InviteItem
                            member=member.clone()
                            loading
                            on_invite=on_invite.clone()
                        />
                    </For>
                </div>
                <div class="confirm-actions">
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=on_close
                    >
                        "关闭"
                    </Button>
                </div>
            </div>
        </div>
    }
}
#[component]
fn InviteItem(
    member: TeamMember,
    loading: bool,
    on_invite: Callback<(u64,)>,
) -> impl IntoView {
    let name = member.username.clone().unwrap_or_else(|| member.user_id.to_string());

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
    view! {
        <div class="group-detail-modal">
            <div class="group-detail-header">
                <div class="group-detail-title-row">
                    <h2 class="group-detail-title">{group.group_name.clone()}</h2>
                    <Show when=move || is_group_leader>
                        <span class="group-card-badge">"组长"</span>
                    </Show>
                </div>
                <Show when=move || group_desc_check>
                    <p class="group-detail-desc">{group_desc.clone().unwrap()}</p>
                </Show>
                <div class="group-detail-meta">
                    <span class="group-meta-item">
                        <span class="group-meta-label">"创建时间: "</span>
                        <span>{format_timestamp(group.group_create_time)}</span>
                    </span>
                </div>
            </div>

            // 标签页切换
            <div class="group-detail-tabs">
                <button
                    class=move || if active_tab.get() == 0 { "tab-btn tab-btn-active" } else { "tab-btn" }
                    on:click=tab0_handler
                >
                    {"成员 ("}{group.group_members.len()}{")"}
                </button>
                <button
                    class=move || if active_tab.get() == 1 { "tab-btn tab-btn-active" } else { "tab-btn" }
                    on:click=tab1_handler
                >
                    {"任务 ("}{group_tasks.len()}{")"}
                </button>
            </div>

            // 成员列表
            <Show when=move || active_tab.get() == 0>
                <MemberListContent
                    members=group.group_members.clone()
                    is_gl=is_group_leader
                    current_user_id
                    group_leader_id=group.group_leader_id
                    loading=action_loading.get()
                    on_kick=on_kick_member.clone()
                />
                // 组长可邀请未加入的成员
                <Show when=move || is_group_leader && !inviteable_members_empty>
                    <div class="invite-section">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Sm
                            on_click=Callback::from(move |_| { open_invite_handler(ev::MouseEvent::new("click").unwrap()); })
                        >
                            "邀请成员"
                        </Button>
                    </div>
                </Show>
            </Show>

            // 任务列表
            <Show when=move || active_tab.get() == 1>
                <TaskListContent
                    tasks=group_tasks.clone()
                    is_gl=is_group_leader
                    current_user_id
                    loading=action_loading.get()
                    on_unassign=on_unassign_task.clone()
                    on_assign=on_assign_task.clone()
                />
            </Show>

            // 错误提示
            <Show when=move || action_error.get().is_some()>
                <p class="group-card-error">{action_error.get().unwrap()}</p>
            </Show>

            // 底部操作栏
            <div class="group-detail-footer">
                <Show when=move || is_group_leader>
                    <Button
                        variant=ButtonVariant::Danger
                        size=ButtonSize::Sm
                        disabled=action_loading.get()
                        on_click=on_delete_group
                    >
                        "解散小组"
                    </Button>
                </Show>
                <Show when=move || !is_group_leader>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        disabled=action_loading.get()
                        on_click=on_leave_group
                    >
                        "退出小组"
                    </Button>
                </Show>
                <Button
                    variant=ButtonVariant::Secondary
                    size=ButtonSize::Sm
                    on_click=on_close
                >
                    "关闭"
                </Button>
            </div>

            // 确认弹窗
            <Show when=move || confirm_modal.get().is_some()>
                <div class="confirm-overlay">
                    <div class="confirm-box">
                        <p class="confirm-msg">{confirm_modal.get().unwrap()}</p>
                        <div class="confirm-actions">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=Callback::from(move |_| set_confirm_modal.set(None))
                            >
                                "取消"
                            </Button>
                            <Button
                                variant=ButtonVariant::Danger
                                size=ButtonSize::Sm
                                disabled=action_loading.get()
                                on_click=on_confirm_yes
                            >
                                "确定"
                            </Button>
                        </div>
                    </div>
                </div>
            </Show>

            // 邀请成员弹窗
            <Show when=move || invite_modal.get()>
                <InviteModalContent
                    members=inviteable_members.clone()
                    loading=action_loading.get()
                    on_invite=on_invite.clone()
                    on_close=Callback::from(move |_| set_invite_modal.set(false))
                />
            </Show>
        </div>
    }
}

// ---------------------------------------------------------------------------
// GroupCard — 小组卡片（显示小组信息 + 打开详情按钮）
// ---------------------------------------------------------------------------
#[component]
fn GroupCard(
    group: ApiGroup,
    team_id: u64,
    current_user_id: u64,
    team_tasks: Vec<Task>,
    on_assign_success: Callback<((),)>,
    on_join_success: Callback<((),)>,
) -> impl IntoView {
    let client = use_api_client();

    // Pre-compute all derived values BEFORE any move closures
    let is_member = group
        .group_members
        .iter()
        .any(|m| m.user_id == current_user_id);
    let is_group_leader = group.group_leader_id == current_user_id;
    let leader_name = group
        .group_members
        .iter()
        .find(|m| m.user_id == group.group_leader_id)
        .and_then(|m| m.username.clone())
        .unwrap_or_else(|| group.group_leader_id.to_string());
    let group_desc = group.group_description.clone();
    let group_desc_check = group_desc.is_some();
    let group_name = group.group_name.clone();
    let group_members_len = group.group_members.len();
    let member_names: Vec<String> = group
        .group_members
        .iter()
        .map(|m| m.username.clone().unwrap_or_else(|| m.user_id.to_string()))
        .collect();
    let tasks_not_empty = !team_tasks.is_empty();
    let group_id = group.group_id;

    let (show_assign_modal, set_show_assign_modal) = signal(false);
    let (show_detail_modal, set_show_detail_modal) = signal(false);
    let (selected_task_id, set_selected_task_id) = signal(0u64);
    let (assign_loading, set_assign_loading) = signal(false);
    let (assign_error, set_assign_error) = signal(Option::<String>::None);

    let (join_loading, set_join_loading) = signal(false);
    let (join_error, set_join_error) = signal(Option::<String>::None);

    let on_assign_confirm: Callback<(ev::MouseEvent,)> = {
        let client = client.clone();
        let on_assign_success = on_assign_success.clone();
        let group_id = group.group_id;
        Callback::from(move |_: ev::MouseEvent| {
            let task_id = selected_task_id.get();
            if task_id == 0 {
                set_assign_error.set(Some("Please select a task".to_string()));
                return;
            }
            let client = client.clone();
            set_assign_loading.set(true);
            set_assign_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match assign_task_to_group(&client, task_id, group_id).await {
                    Ok(_updated_task) => {
                        set_assign_loading.set(false);
                        set_show_assign_modal.set(false);
                        on_assign_success.run(((),));
                    }
                    Err(e) => {
                        set_assign_error.set(Some(e.message));
                        set_assign_loading.set(false);
                    }
                }
            });
        })
    };

    let on_join: Callback<(ev::MouseEvent,)> = {
        let client = client.clone();
        let on_join_success = on_join_success.clone();
        let group_id = group.group_id;
        Callback::from(move |_: ev::MouseEvent| {
            let client = client.clone();
            set_join_loading.set(true);
            set_join_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let req = crate::api::group::AddMemberRequest {
                    user_id: current_user_id,
                    level: 1,
                };
                match add_group_member(&client, group_id, &req).await {
                    Ok(()) => {
                        set_join_loading.set(false);
                        on_join_success.run(((),));
                    }
                    Err(e) => {
                        set_join_error.set(Some(e.message));
                        set_join_loading.set(false);
                    }
                }
            });
        })
    };

    let do_assign_close: Callback<(ev::MouseEvent,)> = Callback::from(move |_: ev::MouseEvent| {
        set_show_assign_modal.set(false);
    });

    let do_detail_close: Callback<(ev::MouseEvent,)> = Callback::from(move |_: ev::MouseEvent| {
        set_show_detail_modal.set(false);
    });

    let on_action_done: Callback<((),)> = {
        let on_join_success = on_join_success.clone();
        let on_assign_success = on_assign_success.clone();
        Callback::from(move |_: ()| {
            on_join_success.run(((),));
            on_assign_success.run(((),));
        })
    };

    // Use stored task list (not moved) for rendering
    let stored_tasks = team_tasks.clone();
    let stored_tasks_for_opts = stored_tasks.clone();
    let on_task_change = move |e: leptos::ev::Event| {
        let v = event_target_value(&e);
        set_selected_task_id.set(v.parse().unwrap_or(0));
    };

    view! {
        <div class="group-card">
            <div class="group-card-header">
                <h3 class="group-card-title">{group_name.clone()}</h3>
                <Show when=move || is_group_leader>
                    <span class="group-card-badge">"组长"</span>
                </Show>
            </div>

            <Show when=move || group_desc_check>
                <p class="group-card-desc">{group_desc.clone().unwrap()}</p>
            </Show>

            <div class="group-card-meta">
                <span class="group-meta-item">
                    <span class="group-meta-label">"组长: "</span>
                    <span>{leader_name}</span>
                </span>
                <span class="group-meta-item">
                    <span class="group-meta-label">"成员: "</span>
                    <span>{group_members_len}</span>
                </span>
                <span class="group-meta-item">
                    <span class="group-meta-label">"创建: "</span>
                    <span>{format_timestamp(group.group_create_time)}</span>
                </span>
            </div>

            <Show when=move || is_member>
                <div class="group-card-member-list">
                    <span class="group-meta-label">"成员: "</span>
                    {member_names.iter().map(|name| {
                        view! { <span class="group-member-chip">{name.clone()}</span> }
                    }).collect::<Vec<_>>()}
                </div>
            </Show>

            <div class="group-card-actions">
                // 打开详情/管理弹窗（成员可见）
                <Show when=move || is_member>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=Callback::from(move |_: ev::MouseEvent| set_show_detail_modal.set(true))
                    >
                        "管理"
                    </Button>
                </Show>

                // 指派任务（团队管理员，且有未指派任务时）
                <Show when=move || is_group_leader && tasks_not_empty>
                    <Button
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Sm
                        on_click=Callback::from(move |_: ev::MouseEvent| set_show_assign_modal.set(true))
                    >
                        "指派任务"
                    </Button>
                </Show>

                // 加入小组（非成员）
                <Show when=move || !is_member>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        disabled=join_loading.get()
                        on_click=on_join
                    >
                        "加入小组"
                    </Button>
                </Show>
            </div>

            <Show when=move || join_error.get().is_some()>
                <p class="group-card-error">{join_error.get().unwrap()}</p>
            </Show>

            // 指派任务弹窗
            <Modal
                title=format!("指派任务到 {}", group_name)
                open=MaybeSignal::derive(move || show_assign_modal.get())
                on_close=do_assign_close
            >
                <div class="assign-modal-body">
                    <div class="form-group">
                        <label class="form-label">"选择任务"</label>
                        <select
                            class="form-select"
                            on:change=on_task_change
                        >
                            <option value="0">"-- 选择任务 --"</option>
                            {let opts: Vec<_> = stored_tasks_for_opts.iter().map(|t| {
                                view! {
                                    <option value=t.task_id>{t.task_name.clone()}</option>
                                }
                            }).collect(); opts}
                        </select>
                    </div>

                    <Show when=move || assign_error.get().is_some()>
                        <p class="form-error">{assign_error.get().unwrap()}</p>
                    </Show>

                    <div class="assign-modal-actions">
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Sm
                            on_click=Callback::from(move |_: ev::MouseEvent| set_show_assign_modal.set(false))
                        >
                            "取消"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Sm
                            disabled=assign_loading.get()
                            on_click=on_assign_confirm
                        >
                            {if assign_loading.get() { "指派中..." } else { "确认指派" }}
                        </Button>
                    </div>
                </div>
            </Modal>

            // 小组详情管理弹窗
            <Modal
                title=format!("小组详情 - {}", group_name)
                open=MaybeSignal::derive(move || show_detail_modal.get())
                on_close=do_detail_close
            >
                <GroupDetailModal
                    group=group.clone()
                    team_id
                    current_user_id
                    is_group_leader
                    team_members=Vec::new()
                    team_tasks=stored_tasks.clone()
                    on_close=do_detail_close
                    on_action=on_action_done
                />
            </Modal>
        </div>
    }
}

// ---------------------------------------------------------------------------
// TeamGroupsPage — 小组管理主页
// ---------------------------------------------------------------------------
#[component]
pub fn TeamGroupsPage() -> impl IntoView {
    let params = use_params_map();
    let team_id = params
        .get()
        .get("team_id")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(0);

    let client = use_api_client();
    let team_store = use_team_store();
    let user_store = use_user_store();

    let (page_error, set_page_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(true);
    let (groups, set_groups) = signal(Vec::<ApiGroup>::new());
    let (groups_error, set_groups_error) = signal(Option::<String>::None);

    let (show_create_modal, set_show_create_modal) = signal(false);
    let (create_loading, set_create_loading) = signal(false);
    let (create_error, set_create_error) = signal(Option::<String>::None);
    let (new_group_name, set_new_group_name) = signal(String::new());
    let (new_group_desc, set_new_group_desc) = signal(String::new());

    let (team_tasks, set_team_tasks) = signal(Vec::<Task>::new());
    let (team_members, set_team_members) = signal(Vec::<TeamMember>::new());

    let current_user_id = user_store.user_id().unwrap_or(0);

    let current_team = {
        let team_store = team_store.clone();
        move || {
            team_store
                .state
                .get()
                .teams
                .iter()
                .find(|team| team.team_id == team_id)
                .cloned()
        }
    };

    let is_team_leader = move || {
        current_team()
            .map(|t| t.team_leader_id == current_user_id)
            .unwrap_or(false)
    };

    let do_load: Callback<((),), ()> = {
        let client = client.clone();
        let team_store = team_store.clone();
        let set_page_error = set_page_error.clone();
        let set_loading = set_loading.clone();
        let set_groups = set_groups.clone();
        let set_groups_error = set_groups_error.clone();
        let set_team_tasks = set_team_tasks.clone();
        let set_team_members = set_team_members.clone();

        Callback::from(move |_: ()| {
            let team_store = team_store.clone();
            if team_id == 0 {
                set_page_error.set(Some("Invalid team id".to_string()));
                set_loading.set(false);
                return;
            }

            set_loading.set(true);
            set_page_error.set(None);
            set_groups_error.set(None);

            let client = client.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::team::get_team(&client, team_id).await {
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
                    Ok(members) => {
                        set_team_members.set(members);
                    }
                    Err(_e) => {}
                }

                match list_tasks(&client, 1, 50, None, Some(team_id)).await {
                    Ok(resp) => {
                        set_team_tasks.set(resp.tasks);
                    }
                    Err(_e) => {}
                }

                match list_groups(&client, team_id).await {
                    Ok(group_list) => {
                        set_groups.set(group_list);
                    }
                    Err(e) => {
                        set_groups_error.set(Some(e.message));
                    }
                }

                set_loading.set(false);
            });
        })
    };

    Effect::new(move |_| {
        do_load.run(((),));
    });

    let on_assign_success: Callback<((),)> = {
        let do_load = do_load.clone();
        Callback::from(move |_: ()| {
            do_load.run(((),));
        })
    };

    let on_join_success: Callback<((),)> = {
        let do_load = do_load.clone();
        Callback::from(move |_: ()| {
            do_load.run(((),));
        })
    };

    let on_create_submit = {
        let client = client.clone();
        let do_load = do_load.clone();
        Callback::from(move |_: ev::SubmitEvent| {
            let name = new_group_name.get();
            if name.trim().is_empty() {
                set_create_error.set(Some("Group name is required".to_string()));
                return;
            }
            let req = CreateGroupRequest {
                group_name: name.clone(),
                group_leader_id: None,
                group_description: if new_group_desc.get().trim().is_empty() {
                    None
                } else {
                    Some(new_group_desc.get().trim().to_string())
                },
            };
            set_create_loading.set(true);
            set_create_error.set(None);

            let client = client.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match create_group(&client, team_id, &req).await {
                    Ok(_group) => {
                        set_create_loading.set(false);
                        set_show_create_modal.set(false);
                        set_new_group_name.set(String::new());
                        set_new_group_desc.set(String::new());
                        do_load.run(((),));
                    }
                    Err(e) => {
                        set_create_error.set(Some(e.message));
                        set_create_loading.set(false);
                    }
                }
            });
        })
    };

    let do_create_close: Callback<(ev::MouseEvent,)> = Callback::from(move |_: ev::MouseEvent| {
        set_show_create_modal.set(false);
    });

    // Input handlers need Callback<(String,), ()>
    let name_input: Callback<(String,), ()> = Callback::from(move |v: String| {
        set_new_group_name.set(v);
    });
    let desc_input: Callback<(String,), ()> = Callback::from(move |v: String| {
        set_new_group_desc.set(v);
    });

    view! {
        <div class="team-groups-page">
            <TeamModuleNav team_id />

            <div class="page-container">
                <div class="page-header">
                    <h1 class="page-title">"小组管理"</h1>
                    <Button
                        variant=ButtonVariant::Primary
                        on_click=Callback::from(move |_: ev::MouseEvent| set_show_create_modal.set(true))
                    >
                        "创建小组"
                    </Button>
                </div>

                <Show when=move || page_error.get().is_some()>
                    <p class="page-error">{page_error.get().unwrap()}</p>
                </Show>

                <Show when=move || loading.get()>
                    <Loading variant=LoadingVariant::Spinner />
                </Show>

                <Show when=move || groups_error.get().is_some()>
                    <p class="page-error">{groups_error.get().unwrap()}</p>
                </Show>

                <Show when=move || !loading.get() && groups.get().is_empty()>
                    <div class="empty-state">
                        <p>"暂无可用小组"</p>
                    </div>
                </Show>

                <div class="groups-grid">
                    <For each=move || groups.get() key=|g| g.group_id let:group>
                        <GroupCard
                            group
                            team_id
                            current_user_id
                            team_tasks=team_tasks.get()
                            on_assign_success=on_assign_success.clone()
                            on_join_success=on_join_success.clone()
                        />
                    </For>
                </div>
            </div>

            <Modal
                title="创建小组".to_string()
                open=MaybeSignal::Dynamic(Signal::derive(move || show_create_modal.get()))
                on_close=do_create_close
            >
                <Form on_submit=on_create_submit>
                    <FormGroup>
                        <label class="form-label">"小组名称"</label>
                        <Input
                            value=new_group_name.get()
                            on_input=name_input
                            placeholder="请输入小组名称".to_string()
                        />
                    </FormGroup>

                    <FormGroup>
                        <label class="form-label">"小组描述（可选）"</label>
                        <Input
                            value=new_group_desc.get()
                            on_input=desc_input
                            placeholder="请输入小组描述".to_string()
                        />
                    </FormGroup>

                    <Show when=move || create_error.get().is_some()>
                        <p class="form-error">{create_error.get().unwrap()}</p>
                    </Show>

                    <FormActions>
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Sm
                            on_click=do_create_close
                        >
                            "取消"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
                            size=ButtonSize::Sm
                            disabled=create_loading.get()
                            >
                            {if create_loading.get() { "创建中..." } else { "创建" }}
                        </Button>
                    </FormActions>
                </Form>
            </Modal>
        </div>
    }
}
