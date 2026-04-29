use crate::api::group::{get_group, Group as ApiGroup};
use crate::api::task::{assign_task_to_group, list_tasks};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::team_module_nav::TeamModuleNav;
use crate::store::task_store::Task;
use crate::store::{use_api_client, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};

// ---------------------------------------------------------------------------
// GroupAssignPage — 任务指派页面
// ---------------------------------------------------------------------------
#[component]
pub fn GroupAssignPage() -> impl IntoView {
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

    let navigate = use_navigate();
    let client = use_api_client();
    let client_for_load = client.clone();
    let user_store = use_user_store();
    let current_user_id = user_store.user_id().unwrap_or(0);

    let (loading, set_loading) = signal(true);
    let (page_error, set_page_error) = signal(Option::<String>::None);
    let (group, set_group) = signal(Option::<ApiGroup>::None);
    let (all_tasks, set_all_tasks) = signal(Vec::<Task>::new());
    let (selected_task_id, set_selected_task_id) = signal(0u64);
    let (assign_loading, set_assign_loading) = signal(false);
    let (assign_error, set_assign_error) = signal(Option::<String>::None);
    let (assign_success, set_assign_success) = signal(false);

    // Pre-compute
    let is_group_leader = move || {
        group
            .get()
            .map(|g| g.group_leader_id == current_user_id)
            .unwrap_or(false)
    };
    let group_name = move || {
        group
            .get()
            .map(|g| g.group_name.clone())
            .unwrap_or_default()
    };
    let unassigned_tasks: Memo<Vec<Task>> = Memo::new(move |_| {
        all_tasks
            .get()
            .iter()
            .filter(|t| t.task_group_id.is_none())
            .cloned()
            .collect()
    });
    let tasks_empty = move || unassigned_tasks.get().is_empty();

    // Load data
    let do_load: Callback<((),), ()> = Callback::from(move |_: ()| {
        let client = client_for_load.clone();
        let set_loading = set_loading.clone();
        let set_page_error = set_page_error.clone();
        let set_group = set_group.clone();
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
    });

    Effect::new(move |_| {
        do_load.run(((),));
    });

    // Task select handler
    let on_task_change = move |e: leptos::ev::Event| {
        let v = event_target_value(&e);
        set_selected_task_id.set(v.parse().unwrap_or(0));
    };

    // Confirm assign
    let on_assign_confirm: Callback<(ev::MouseEvent,)> = {
        let client = client.clone();
        Callback::from(move |_: ev::MouseEvent| {
            let task_id = selected_task_id.get();
            if task_id == 0 {
                set_assign_error.set(Some("请选择一个任务".to_string()));
                return;
            }
            let client = client.clone();
            set_assign_loading.set(true);
            set_assign_error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                match assign_task_to_group(&client, task_id, group_id).await {
                    Ok(_updated_task) => {
                        set_assign_loading.set(false);
                        set_assign_success.set(true);
                    }
                    Err(e) => {
                        set_assign_error.set(Some(e.message));
                        set_assign_loading.set(false);
                    }
                }
            });
        })
    };

    view! {
        <div class="team-groups-page">
            <TeamModuleNav team_id />

            <div class="page-container">
                // Back button
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
                    <p class="page-error">{page_error.get().unwrap()}</p>
                </Show>

                <Show when=move || !loading.get() && group.get().is_some()>
                    <div class="group-detail-card">
                        <div class="group-detail-header">
                            <div class="group-detail-title-row">
                                <h2 class="group-detail-title">{group_name.clone()}</h2>
                                <Show when=move || is_group_leader()>
                                    <span class="group-card-badge">"组长"</span>
                                </Show>
                            </div>
                            <p class="group-detail-desc">"请从下方选择一个任务指派给本小组"</p>
                        </div>

                        // Task selector
                        <div class="assign-form-section">
                            <div class="form-group">
                                <label class="form-label">"选择任务"</label>
                                <select
                                    class="form-select"
                                    on:change=on_task_change
                                >
                                    <option value="0">"-- 选择任务 --"</option>
                                    {unassigned_tasks.get().iter().map(|t| {
                                        view! {
                                            <option value=t.task_id>{t.task_name.clone()}</option>
                                        }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </div>

                            <Show when=move || assign_error.get().is_some()>
                                <p class="form-error">{assign_error.get().unwrap()}</p>
                            </Show>

                            <Show when=move || assign_success.get()>
                                <p class="form-success">"任务指派成功！"</p>
                            </Show>

                            <div class="assign-modal-actions">
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
                                    "返回"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Primary
                                    size=ButtonSize::Sm
                                    disabled=assign_loading.get() || tasks_empty()
                                    on_click=on_assign_confirm
                                >
                                    {if assign_loading.get() { "指派中..." } else { "确认指派" }}
                                </Button>
                            </div>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
