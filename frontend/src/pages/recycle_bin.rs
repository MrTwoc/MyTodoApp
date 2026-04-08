use leptos::prelude::*;

use crate::api::client::ApiClient;
use crate::api::task;
use crate::store::task_store::Task;

#[component]
pub fn RecycleBinPage() -> impl IntoView {
    let (deleted_tasks, set_deleted_tasks) = signal(Vec::<Task>::new());
    let (is_loading, set_is_loading) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);

    let client = expect_context::<ApiClient>();

    let do_load: Callback<()> = {
        let client = client.clone();
        let set_deleted_tasks = set_deleted_tasks.clone();
        let set_is_loading = set_is_loading.clone();
        let set_error = set_error.clone();

        Callback::from(move || {
            let client = client.clone();
            let set_deleted_tasks = set_deleted_tasks.clone();
            let set_is_loading = set_is_loading.clone();
            let set_error = set_error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                set_is_loading.set(true);
                set_error.set(None);

                match task::get_recycle_bin(&client).await {
                    Ok(tasks) => {
                        set_deleted_tasks.set(tasks);
                    }
                    Err(e) => {
                        set_error.set(Some(e.to_string()));
                    }
                }

                set_is_loading.set(false);
            });
        })
    };

    do_load.run(());

    let format_timestamp = move |ts: Option<i64>| -> String {
        if let Some(t) = ts {
            chrono::DateTime::from_timestamp(t, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        }
    };

    let format_deadline = move |deadline: Option<i64>| -> String {
        if let Some(d) = deadline {
            chrono::DateTime::from_timestamp(d, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "No deadline".to_string())
        } else {
            "No deadline".to_string()
        }
    };

    view! {
        <div class="recycle-bin-page">
            <div class="page-header">
                <h1>"Recycle Bin"</h1>
                <p class="subtitle">"Manage deleted tasks - restore or permanently delete them"</p>
                <button class="refresh-btn" on:click=move |_| do_load.run(())>
                    "Refresh"
                </button>
            </div>

            {move || {
                if is_loading.get() {
                    view! { <div class="loading">"Loading..."</div> }.into_any()
                } else if let Some(err_msg) = error.get() {
                    view! {
                        <div class="error-message">
                            <p>{err_msg}</p>
                            <button on:click=move |_| do_load.run(())>"Retry"</button>
                        </div>
                    }.into_any()
                } else {
                    let tasks = deleted_tasks.get();

                    if tasks.is_empty() {
                        view! {
                            <div class="empty-state">
                                <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                    <polyline points="3 6 5 6 21 6"/>
                                    <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                                </svg>
                                <h3>"Recycle Bin is empty"</h3>
                                <p>"Deleted tasks will appear here"</p>
                            </div>
                        }.into_any()
                    } else {
                        let task_list = tasks
                            .into_iter()
                            .map(|task| {
                                let task_id = task.task_id;
                                let c = client.clone();
                                let c2 = client.clone();
                                let s = set_deleted_tasks.clone();
                                let s2 = set_deleted_tasks.clone();
                                let e = set_error.clone();
                                let e2 = set_error.clone();
                                view! {
                                    <div class="task-item">
                                        <div class="col-task">
                                            <div class="task-name">{task.task_name}</div>
                                            {task.task_description.map(|desc| {
                                                view! { <div class="task-desc">{desc}</div> }
                                            })}
                                        </div>
                                        <div class="col-priority">
                                            <span class=format!("priority-badge priority-{}", task.task_priority)>
                                                {task.task_priority}
                                            </span>
                                        </div>
                                        <div class="col-deadline">
                                            {format_deadline(task.task_deadline)}
                                        </div>
                                        <div class="col-deleted">
                                            {format_timestamp(task.deleted_at)}
                                        </div>
                                        <div class="col-actions">
                                            <button
                                                class="btn-restore"
                                                on:click=move |_| {
                                                    let c = c.clone();
                                                    let s = s.clone();
                                                    let e = e.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        match task::restore_task(&c, task_id).await {
                                                            Ok(_) => {
                                                                s.update(|tasks| {
                                                                    tasks.retain(|t| t.task_id != task_id);
                                                                });
                                                            }
                                                            Err(err) => {
                                                                e.set(Some(err.to_string()));
                                                            }
                                                        }
                                                    });
                                                }
                                            >
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/>
                                                    <path d="M3 3v5h5"/>
                                                </svg>
                                                "Restore"
                                            </button>
                                            <button
                                                class="btn-delete-permanent"
                                                on:click=move |_| {
                                                    let c = c2.clone();
                                                    let s = s2.clone();
                                                    let e = e2.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        match task::permanent_delete_task(&c, task_id).await {
                                                            Ok(_) => {
                                                                s.update(|tasks| {
                                                                    tasks.retain(|t| t.task_id != task_id);
                                                                });
                                                            }
                                                            Err(err) => {
                                                                e.set(Some(err.to_string()));
                                                            }
                                                        }
                                                    });
                                                }
                                            >
                                                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                    <polyline points="3 6 5 6 21 6"/>
                                                    <path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/>
                                                </svg>
                                                "Delete"
                                            </button>
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>();

                        view! {
                            <div class="task-list">
                                <div class="task-list-header">
                                    <span class="col-task">"Task"</span>
                                    <span class="col-priority">"Priority"</span>
                                    <span class="col-deadline">"Deadline"</span>
                                    <span class="col-deleted">"Deleted At"</span>
                                    <span class="col-actions">"Actions"</span>
                                </div>
                                {task_list}
                            </div>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
