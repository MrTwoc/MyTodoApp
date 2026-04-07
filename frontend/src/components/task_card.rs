use crate::store::task_store::{Task, TaskStatus};
use leptos::ev;
use leptos::prelude::*;

#[derive(Clone, PartialEq)]
pub enum TaskCardVariant {
    Default,
    Compact,
    Elevated,
}

#[component]
pub fn TaskCard(
    task: Task,
    #[prop(default = false)] interactive: bool,
    #[prop(default = TaskCardVariant::Default)] variant: TaskCardVariant,
    #[prop(optional)] on_click: Option<Callback<(ev::MouseEvent,)>>,
    #[prop(optional)] on_toggle_favorite: Option<Callback<(u64,)>>,
    #[prop(optional)] extra_actions: Option<Children>,
) -> impl IntoView {
    let variant_class = match variant {
        TaskCardVariant::Default => "task-card",
        TaskCardVariant::Compact => "task-card-compact",
        TaskCardVariant::Elevated => "task-card-elevated",
    };

    let status_class = match task.task_status {
        TaskStatus::Active => "status-active",
        TaskStatus::Completed => "status-completed",
        TaskStatus::Paused => "status-paused",
    };

    let status_label = match task.task_status {
        TaskStatus::Active => "Active",
        TaskStatus::Completed => "Completed",
        TaskStatus::Paused => "Paused",
    };

    let priority_label = match task.task_priority {
        0..=2 => "Low",
        3..=5 => "Medium",
        6..=8 => "High",
        _ => "Urgent",
    };

    let priority_class = match task.task_priority {
        0..=2 => "priority-low",
        3..=5 => "priority-medium",
        6..=8 => "priority-high",
        _ => "priority-urgent",
    };

    // Removed: rendering of task type badge (Personal/Team)

    let handle_click = move |ev: ev::MouseEvent| {
        if let Some(cb) = on_click.as_ref() {
            cb.run((ev,));
        }
    };

    let handle_favorite_click = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        if let Some(cb) = on_toggle_favorite.as_ref() {
            cb.run((task.task_id,));
        }
    };

    view! {
        <div
            class=("task-card-wrapper", true)
            class=(variant_class, true)
            class=("task-card-interactive", interactive)
            on:click=handle_click
        >
            <div class="task-card-header">
                <span class=format!("task-status-badge {}", status_class)>
                    {status_label}
                </span>
                <span class=format!("task-priority-badge {}", priority_class)>
                    {priority_label}
                </span>
                <button
                    class=format!("favorite-btn {}", if task.is_favorite { "favorited" } else { "" })
                    on:click=handle_favorite_click
                    title=if task.is_favorite { "Remove from favorites" } else { "Add to favorites" }
                >
                    <svg viewBox="0 0 24 24" fill=if task.is_favorite { "currentColor" } else { "none" } stroke="currentColor" stroke-width="2">
                        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/>
                    </svg>
                </button>
            </div>

            <h3 class="task-card-title">{task.task_name}</h3>

            {if let Some(desc) = &task.task_description {
                view! { <p class="task-card-desc">{desc.as_str()}</p> }.into_any()
            } else {
                view! { <div class="task-card-desc"></div> }.into_any()
            }}

            {if let Some(deadline) = task.task_deadline {
                let deadline_str = format_timestamp(deadline);
                view! {
                    <div class="task-card-deadline">
                        <svg class="task-card-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="12" cy="12" r="10"/>
                            <path d="M12 6v6l4 2"/>
                        </svg>
                        <span>{deadline_str}</span>
                    </div>
                }.into_any()
            } else {
                ().into_any()
            }}

            <div class="task-card-footer">
                {if let Some(actions) = extra_actions {
                    view! { <div class="task-card-extra-actions">{actions()}</div> }.into_any()
                } else {
                    ().into_any()
                }}
            </div>
        </div>
    }
}

fn format_timestamp(ts: i64) -> String {
    let datetime = chrono::DateTime::<chrono::Utc>::from_timestamp(ts, 0).unwrap();
    datetime.format("%Y/%m/%d").to_string()
}

#[component]
pub fn TaskCardSkeleton() -> impl IntoView {
    view! {
        <div class="task-card-wrapper task-card-skeleton">
            <div class="skeleton" style="gap: 12px">
                <div class="skeleton-row">
                    <div class="skeleton-item skeleton-rect" style="width: 60px; height: 20px"></div>
                    <div class="skeleton-item skeleton-rect" style="width: 50px; height: 20px"></div>
                </div>
                <div class="skeleton-item skeleton-rect" style="width: 80%; height: 24px"></div>
                <div class="skeleton-item skeleton-rect" style="width: 100%; height: 40px"></div>
                <div class="skeleton-item skeleton-rect" style="width: 120px; height: 16px"></div>
            </div>
        </div>
    }
}
