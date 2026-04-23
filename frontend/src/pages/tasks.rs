use chrono::Utc;
use std::collections::HashSet;

use crate::api::task::{
    CreateTaskRequest, create_task as api_create_task, list_tasks, toggle_task_favorite,
};
use crate::api::team::list_teams;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::kanban::KanbanBoard;
use crate::components::loading::{Loading, LoadingVariant};
use crate::components::modal::Modal;
use crate::components::search::{Pagination, SearchInput};
use crate::components::task_card::{TaskCard, TaskCardSkeleton};
use crate::components::task_form::{TaskForm, TaskFormData, TaskFormMode};
use crate::store::task_store::{Task, TaskStatus};
use crate::store::{use_api_client, use_task_store, use_team_store, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

// const OFFLINE_PAGE_SIZE: u32 = 20;

fn clamp_page(page: u32, total: u32, page_size: u32) -> u32 {
    let total_pages = if total == 0 {
        1
    } else {
        total.div_ceil(page_size)
    };
    page.clamp(1, total_pages)
}

fn take_page<T>(items: Vec<T>, page: u32, page_size: u32) -> Vec<T> {
    if items.is_empty() {
        return Vec::new();
    }
    let start = ((page.saturating_sub(1)) * page_size) as usize;
    if start >= items.len() {
        return Vec::new();
    }
    let end = start.saturating_add(page_size as usize).min(items.len());
    items
        .into_iter()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

#[component]
pub fn TasksPage() -> impl IntoView {
    let task_store = use_task_store();
    let store_for_filter = task_store.clone();
    let store_for_render = task_store.clone();
    // let offline_store = use_offline_task_store();
    let client = use_api_client();
    let navigate = use_navigate();

    // let nav_back = {
    //     let n = navigate.clone();
    //     move |_| n("/", Default::default())
    // };

    let (show_create_modal, set_show_create_modal) = signal(false);
    let (show_edit_modal, set_show_edit_modal) = signal(false);
    let (editing_task, set_editing_task) = signal(None::<Task>);
    // let (offline_page, set_offline_page) = signal(1_u32);
    use leptos::prelude::RwSignal;
    let view_mode: RwSignal<&str> = RwSignal::new("kanban");
    let team_store = use_team_store();

    let filter_all = {
        let store = task_store.clone();
        // let offline_page = set_offline_page.clone();
        Callback::from(move |_| {
            store.set_filter_status(None);
            // offline_page.set(1);
        })
    };
    let filter_active = {
        let store = task_store.clone();
        // let offline_page = set_offline_page.clone();
        Callback::from(move |_| {
            store.set_filter_status(Some(TaskStatus::Active));
            // offline_page.set(1);
        })
    };
    let filter_completed = {
        let store = task_store.clone();
        // let offline_page = set_offline_page.clone();
        Callback::from(move |_| {
            store.set_filter_status(Some(TaskStatus::Completed));
            // offline_page.set(1);
        })
    };
    let filter_paused = {
        let store = task_store.clone();
        // let offline_page = set_offline_page.clone();
        Callback::from(move |_| {
            store.set_filter_status(Some(TaskStatus::Paused));
            // offline_page.set(1);
        })
    };

    let handle_search = {
        let store = task_store.clone();
        // let offline_page = set_offline_page.clone();
        Callback::from(move |query: String| {
            store.set_search_query(if query.is_empty() { None } else { Some(query) });
            // offline_page.set(1);
        })
    };

    let handle_toggle_favorite = {
        let store = task_store.clone();
        let client = use_api_client();
        Callback::from(move |task_id: u64| {
            let store = store.clone();
            let client = client.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match toggle_task_favorite(&client, task_id).await {
                    Ok(is_favorite) => {
                        store.toggle_favorite(task_id);
                    }
                    Err(e) => {
                        store.toggle_favorite(task_id);
                    }
                }
            });
        })
    };

    // let is_offline_mode_store = offline_store.clone();
    // let is_offline_mode = move || is_offline_mode_store.state.get().enabled;

    let client_load = client.clone();
    let store_load = task_store.clone();
    let client_teams = client.clone();
    let team_store_load = team_store.clone();

    let teams_state = team_store.state;
    let need_load_teams = teams_state.get_untracked().teams.is_empty();

    // let is_offline_check = is_offline_mode.clone();
    Effect::new(move |_| {
        // if !is_offline_check() {
        let client = client_load.clone();
        let store = store_load.clone();
        let client_teams = client_teams.clone();
        let team_store = team_store_load.clone();

        wasm_bindgen_futures::spawn_local(async move {
            store.set_loading(true);

            if need_load_teams {
                let teams_result = list_teams(&client_teams).await;
                if let Ok(teams) = teams_result {
                    team_store.set_teams(teams);
                }
            }

            match list_tasks(&client, 1, 20, None, None).await {
                Ok(resp) => {
                    store.set_tasks(resp.tasks, resp.total.unwrap_or(0));
                }
                Err(e) => {
                    store.set_error(e.message);
                }
            }
        });
        // }
    });

    let current_page = {
        let task_store = task_store.clone();
        // let offline_store = offline_store.clone();
        move || {
            let page = task_store.state.get().pagination.page;
            let page_size = task_store.state.get().pagination.page_size;
            let total = task_store.filtered_tasks().len() as u32;
            clamp_page(page, total, page_size)
        }
    };

    let total_pages = {
        let task_store = task_store.clone();
        move || {
            let total = task_store.filtered_tasks().len() as u32;
            let page_size = task_store.state.get().pagination.page_size;
            if total == 0 {
                1
            } else {
                total.div_ceil(page_size)
            }
        }
    };

    let visible_tasks = {
        let task_store = task_store.clone();
        move || {
            let page_size = task_store.state.get().pagination.page_size;
            let page = task_store.state.get().pagination.page;
            let total = task_store.filtered_tasks().len() as u32;
            let all = task_store.filtered_tasks();
            take_page(all, clamp_page(page, total, page_size), page_size)
        }
    };

    let handle_page_change = {
        let online_store = task_store.clone();
        // let offline_store = offline_store.clone();
        // let offline_page = set_offline_page.clone();
        Callback::from(move |page: u32| {
            let page_size = online_store.state.get().pagination.page_size;
            let total = online_store.filtered_tasks().len() as u32;
            let page = clamp_page(page, total, page_size);
            // if is_offline_mode() {
            //     offline_page.set(page);
            // } else {
            online_store.set_page(page);
            // }
        })
    };

    let open_create = {
        let set_show_create_modal = set_show_create_modal;
        Callback::from(move |_| {
            set_show_create_modal.set(true);
        })
    };

    let do_create_submit = {
        let task_store = task_store.clone();
        let user_store = use_user_store();
        let client = use_api_client();
        let set_show_create_modal = set_show_create_modal;
        Callback::from(move |data: TaskFormData| {
            // if offline_store.is_enabled() {
            //     let store = offline_store.clone();
            //     let task = store.new_task(
            //         data.task_name,
            //         data.task_description,
            //         data.task_keywords,
            //         data.task_priority,
            //         data.task_deadline,
            //     );
            //     store.add_task(task);
            //     set_offline_page.set(1);
            // } else {
            let client = client.clone();
            let task_store = task_store.clone();
            let task_name = data.task_name.clone();
            let task_description = data.task_description.clone();
            let task_keywords = data.task_keywords.clone();
            let task_priority = data.task_priority;
            let task_difficulty = data.task_difficulty;
            let task_deadline = data.task_deadline;
            let task_team_id = data.task_team_id;
            let user_id = user_store.user_id().unwrap_or(0);
            let team_id = if data.task_team_id.is_some() {
                data.task_team_id
            } else {
                team_store.state.get().active_team_id
            };
            wasm_bindgen_futures::spawn_local(async move {
                let req = CreateTaskRequest {
                    task_name,
                    task_description,
                    task_keywords,
                    task_priority,
                    task_difficulty,
                    task_deadline,
                    task_leader_id: user_id,
                    task_team_id: team_id,
                };
                match api_create_task(&client, &req).await {
                    Ok(task) => {
                        task_store.add_task(task);
                    }
                    Err(e) => {
                        tracing::error!("Failed to create task: {}", e.message);
                    }
                }
            });
            // }
            set_show_create_modal.set(false);
        })
    };

    let do_create_close = Callback::from(move |_| {
        set_show_create_modal.set(false);
    });

    let do_create_cancel = Callback::from(move || {
        set_show_create_modal.set(false);
    });

    let do_edit_submit = {
        let task_store = task_store.clone();
        let editing_task = editing_task;
        let set_show_edit_modal = set_show_edit_modal;
        let set_editing_task = set_editing_task;
        Callback::from(move |data: TaskFormData| {
            if let Some(mut task) = editing_task.get() {
                task.task_name = data.task_name;
                task.task_description = data.task_description;
                task.task_keywords = data
                    .task_keywords
                    .into_iter()
                    .filter(|keyword| !keyword.trim().is_empty())
                    .collect::<HashSet<_>>();
                task.task_priority = data.task_priority;
                task.task_deadline = data.task_deadline;
                task.task_update_time = Some(Utc::now().timestamp());
                // TODO: update via API
                set_editing_task.set(None);
                set_show_edit_modal.set(false);
            }
        })
    };

    let do_edit_close = Callback::from({
        let set_show_edit_modal = set_show_edit_modal;
        let set_editing_task = set_editing_task;
        move |_| {
            set_editing_task.set(None);
            set_show_edit_modal.set(false);
        }
    });

    let do_edit_cancel = Callback::from({
        let set_show_edit_modal = set_show_edit_modal;
        let set_editing_task = set_editing_task;
        move || {
            set_editing_task.set(None);
            set_show_edit_modal.set(false);
        }
    });

    let toggle_view_mode: Callback<((),), ()> = {
        let vm = view_mode.clone();
        Callback::from(move |_| {
            let current = vm.get();
            match current {
                "card" => vm.set("kanban"),
                "kanban" => vm.set("card"),
                _ => vm.set("card"),
            }
        })
    };

    // let toggle_offline = {
    //     let store = offline_store.clone();
    //     move |ev: ev::Event| {
    //         store.set_enabled(event_target_checked(&ev));
    //         set_offline_page.set(1);
    //     }
    // };

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    {/*
                    <button class="back-btn" on:click=nav_back>"← Back"</button>
                    */}
                    <h1 class="page-title">"Tasks"</h1>
                </div>
                <div class="task-header-actions">
                    // <div class="toggle-switch">
                    //     <span class="toggle-label online-label" class:checked=move || !is_offline_mode()>"Online"</span>
                    //     <label class="toggle">
                    //         <input
                    //             type="checkbox"
                    //             checked=move || is_offline_mode()
                    //             on:change=toggle_offline
                    //         />
                    //         <span class="toggle-slider"></span>
                    //     </label>
                    //     <span class="toggle-label offline-label" class:checked=move || is_offline_mode()>"Offline"</span>
                    // </div>
                    <Button
                        variant=ButtonVariant::Primary
                        size=ButtonSize::Sm
                        on_click=open_create
                    >
                        "New Task"
                    </Button>
                </div>
            </header>

            <div class="task-toolbar">
                <SearchInput
                    placeholder="Search tasks...".to_string()
                    instant=true
                    on_search=handle_search
                />
                // <p class="task-mode-hint">
                //     {move || {
                //         if is_offline_mode() {
                //             "Offline mode: only local personal tasks are shown."
                //         } else {
                //             "Online mode: can view online task list. Enable offline mode for local task edit."
                //         }
                //     }}
                // </p>
            </div>

            {move || {
                if view_mode.get() != "kanban" {
                    let filter_fav = {
                        let store = store_for_filter.clone();
                        Callback::from(move |_| {
                            let current = store.state.get().filters.show_favorites_only.unwrap_or(false);
                            store.set_show_favorites_only(!current);
                        })
                    };
                    view! {
                        <div class="filter-bar">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=filter_all
                            >
                                "All"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=filter_active
                            >
                                "Active"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=filter_completed
                            >
                                "Completed"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=filter_paused
                            >
                                "Paused"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=filter_fav
                            >
                                "Favorites"
                            </Button>
                            <div class="filter-bar-spacer"></div>
                            <div class="view-switcher">
                                <button class="view-switcher-btn" on:click={let vm = toggle_view_mode.clone(); move |_| { vm.run(((),)); }} title="Switch to Kanban view">
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <rect x="3" y="3" width="5" height="18" rx="1"/>
                                        <rect x="10" y="3" width="5" height="12" rx="1"/>
                                        <rect x="17" y="3" width="5" height="8" rx="1"/>
                                    </svg>
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    let filter_fav = {
                        let store = task_store.clone();
                        Callback::from(move |_| {
                            let current = store.state.get().filters.show_favorites_only.unwrap_or(false);
                            store.set_show_favorites_only(!current);
                        })
                    };
                    view! {
                        <div class="filter-bar kanban-filter-bar">
                            <Button
                                variant=ButtonVariant::Secondary
                                size=ButtonSize::Sm
                                on_click=filter_fav
                            >
                                "Favorites"
                            </Button>
                            <div class="filter-bar-spacer"></div>
                            <div class="view-switcher">
                                <button class="view-switcher-btn active" on:click={let vm = toggle_view_mode.clone(); move |_| { vm.run(((),)); }} title="Switch to Card view">
                                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                                        <rect x="3" y="3" width="7" height="7" rx="1"/>
                                        <rect x="14" y="3" width="7" height="7" rx="1"/>
                                        <rect x="3" y="14" width="7" height="7" rx="1"/>
                                        <rect x="14" y="14" width="7" height="7" rx="1"/>
                                    </svg>
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }
            }}

            // {move || {
            //     offline_store
            //         .state
            //         .get()
            //         .error
            //         .clone()
            //         .map(|message| view! { <p class="auth-error">{message}</p> })
            // }}

            <div class="tasks-content">
                {{
                    let task_store_for_render = store_for_render.clone();
                    move || {
                        let task_store_for_inner = task_store_for_render.clone();
                        let state = task_store_for_inner.state.get();
                        if state.is_loading {
                            view! {
                                <div class="task-list">
                                    <Loading variant=LoadingVariant::Spinner label="Loading tasks...".to_string() />
                                    <TaskCardSkeleton />
                                    <TaskCardSkeleton />
                                </div>
                            }.into_any()
                        } else if view_mode.get() == "kanban" {
                            let all_tasks = task_store_for_inner.filtered_tasks();
                            if all_tasks.is_empty() {
                                view! {
                                    <div class="empty-state-container">
                                        <div class="empty-state-icon"></div>
                                        <Card
                                            title="No Tasks".to_string()
                                            subtitle="No tasks found matching your filters.".to_string()
                                        >
                                            <p class="empty-text">
                                                "Create a new task to get started."
                                            </p>
                                        </Card>
                                    </div>
                                }.into_any()
                            } else {
                                let navigate = navigate.clone();
                                view! {
                                    <KanbanBoard
                                        tasks=all_tasks
                                        on_task_click=Callback::from(move |task_id| {
                                            navigate(&format!("/tasks/{}", task_id), Default::default());
                                        })
                                        on_task_toggle_favorite=handle_toggle_favorite.clone()
                                    />
                                }.into_any()
                            }
                        } else {
                            let tasks = visible_tasks();
                            if tasks.is_empty() {
                                view! {
                                    <div class="empty-state-container">
                                        <div class="empty-state-icon"></div>
                                        <Card
                                            title="No Tasks".to_string()
                                            subtitle="No tasks found matching your filters.".to_string()
                                        >
                                            <p class="empty-text">
                                                "Create a new task to get started."
                                            </p>
                                        </Card>
                                    </div>
                                }.into_any()
                            } else {
                                let cards: Vec<_> = tasks
                                    .into_iter()
                                    .map(|task| {
                                        let task_id = task.task_id;
                                        let on_fav = handle_toggle_favorite.clone();
                                        view! {
                                            <TaskCard
                                                task=task
                                                interactive=true
                                                on_toggle_favorite=on_fav
                                                on_click=Callback::from({
                                                    let navigator = navigate.clone();
                                                    move |_| {
                                                        navigator(&format!("/tasks/{}", task_id), Default::default());
                                                    }
                                                })
                                            />
                                        }
                                    })
                                    .collect();
                                view! { <div class="task-grid">{cards}</div> }.into_any()
                            }
                        }
                    }
                }}
            </div>

            {move || {
                if total_pages() > 0 && view_mode.get() != "kanban" {
                    view! {
                        <Pagination
                            current_page=current_page()
                            total_pages=total_pages()
                            on_page_change=handle_page_change
                        />
                    }.into_any()
                } else {
                    ().into_any()
                }
            }}

            <Modal
                title="Create Task".to_string()
                open=MaybeSignal::derive(move || show_create_modal.get())
                on_close=do_create_close
            >
                <TaskForm
                    // offline_mode=is_offline_mode()
                    active_team_id=team_store.state.get().active_team_id
                    on_submit=do_create_submit
                    on_cancel=do_create_cancel
                />
            </Modal>

            <Modal
                title="Edit Task".to_string()
                open=MaybeSignal::derive(move || show_edit_modal.get())
                on_close=do_edit_close
            >
                <TaskForm
                    mode=TaskFormMode::Edit
                    // offline_mode=is_offline_mode()
                    initial_data=TaskFormData::from(editing_task.get().unwrap_or_default())
                    active_team_id=team_store.state.get().active_team_id
                    on_submit=do_edit_submit
                    on_cancel=do_edit_cancel
                />
            </Modal>
        </div>
    }
}
