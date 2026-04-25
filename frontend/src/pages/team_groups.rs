use crate::api::group::{
    add_group_member, create_group, list_groups, CreateGroupRequest, Group as ApiGroup,
};
use crate::api::task::{assign_task_to_group, list_tasks};
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

#[component]
fn GroupCard(
    group: ApiGroup,
    team_id: u64,
    current_user_id: u64,
    is_team_leader: bool,
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
    let is_leader = group.group_leader_id == current_user_id;
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

    let (show_assign_modal, set_show_assign_modal) = signal(false);
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

    // Use stored task list (not moved) for rendering
    let stored_tasks = team_tasks.clone();
    let on_task_change = move |e: leptos::ev::Event| {
        let v = event_target_value(&e);
        set_selected_task_id.set(v.parse().unwrap_or(0));
    };

    view! {
        <div class="group-card">
            <div class="group-card-header">
                <h3 class="group-card-title">{group_name.clone()}</h3>
                <Show when=move || is_leader>
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
                <Show when=move || is_team_leader && tasks_not_empty>
                    <Button
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Sm
                        on_click=Callback::from(move |_: ev::MouseEvent| set_show_assign_modal.set(true))
                    >
                        "指派任务"
                    </Button>
                </Show>

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
                            {stored_tasks.iter().map(|t| {
                                view! {
                                    <option value=t.task_id>{t.task_name.clone()}</option>
                                }
                            }).collect::<Vec<_>>()}
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
        </div>
    }
}

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
                            is_team_leader=is_team_leader()
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
                            on_click=Callback::from(move |_: ev::MouseEvent| set_show_create_modal.set(false))
                        >
                            "取消"
                        </Button>
                        <Button
                            variant=ButtonVariant::Primary
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
