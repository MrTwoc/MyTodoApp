use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::form::{Form, FormActions, FormGroup};
use crate::store::task_store::Task;
use leptos::ev;
use leptos::prelude::*;

// ── TaskFormData ──────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct TaskFormData {
    pub name: String,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub priority: u8,
    pub deadline: Option<i64>,
}

// ── TagInput ──────────────────────────────────────────────────────────────────

#[component]
pub fn TagInput(
    tags: ReadSignal<Vec<String>>,
    set_tags: WriteSignal<Vec<String>>,
    #[prop(optional)] placeholder: Option<String>,
) -> impl IntoView {
    let (input_value, set_input_value) = signal(String::new());

    let commit_tag = move || {
        let raw = input_value.get();
        let tag = raw.trim().trim_end_matches(',').to_string();
        if !tag.is_empty() {
            let mut current = tags.get();
            if !current.contains(&tag) {
                current.push(tag);
                set_tags.set(current);
            }
            set_input_value.set(String::new());
        }
    };

    let on_keydown = move |ev: ev::KeyboardEvent| {
        if ev.key() == "Enter" || ev.key() == "," {
            ev.prevent_default();
            commit_tag();
        }
    };

    let on_blur = move |_: ev::FocusEvent| {
        commit_tag();
    };

    view! {
        <div class="tag-input-wrapper">
            <div class="tag-chips">
                {move || {
                    tags.get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, tag)| {
                            let tag_clone = tag.clone();
                            let remove = move |_: ev::MouseEvent| {
                                let mut current = tags.get();
                                current.remove(i);
                                set_tags.set(current);
                            };
                            view! {
                                <span class="tag-chip">
                                    {tag_clone}
                                    <button
                                        type="button"
                                        class="tag-chip-remove"
                                        on:click=remove
                                        aria-label="Remove tag"
                                    >
                                        "×"
                                    </button>
                                </span>
                            }
                        })
                        .collect::<Vec<_>>()
                }}
            </div>
            <input
                type="text"
                class="input-field tag-input-field"
                placeholder=placeholder.unwrap_or_else(|| "Type and press Enter…".to_string())
                prop:value=input_value
                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                on:keydown=on_keydown
                on:blur=on_blur
            />
        </div>
    }
}

// ── DatePicker ────────────────────────────────────────────────────────────────

fn timestamp_to_date_str(ts: i64) -> String {
    let ms = (ts * 1000) as f64;
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn date_str_to_timestamp(s: &str) -> Option<i64> {
    let js_str = wasm_bindgen::JsValue::from_str(s);
    let date = js_sys::Date::new(&js_str);
    let ms = date.get_time();
    if ms.is_nan() {
        None
    } else {
        Some((ms / 1000.0) as i64)
    }
}

#[component]
pub fn DatePicker(
    value: ReadSignal<Option<i64>>,
    set_value: WriteSignal<Option<i64>>,
    #[prop(optional)] label: Option<String>,
) -> impl IntoView {
    let date_str = move || value.get().map(timestamp_to_date_str).unwrap_or_default();

    let on_change = move |ev: ev::Event| {
        let raw = event_target_value(&ev);
        if raw.is_empty() {
            set_value.set(None);
        } else {
            set_value.set(date_str_to_timestamp(&raw));
        }
    };

    view! {
        <div class="input-wrapper">
            {label.map(|text| view! {
                <label class="input-label">{text}</label>
            })}
            <input
                type="date"
                class="input-field"
                prop:value=date_str
                on:change=on_change
            />
        </div>
    }
}

// ── PrioritySelector ─────────────────────────────────────────────────────────

#[component]
pub fn PrioritySelector(value: ReadSignal<u8>, set_value: WriteSignal<u8>) -> impl IntoView {
    let levels: &'static [(u8, &'static str)] =
        &[(0, "Low"), (3, "Medium"), (6, "High"), (9, "Urgent")];

    view! {
        <div class="priority-selector">
            {levels
                .iter()
                .map(|(level, label)| {
                    let level = *level;
                    let label = *label;
                    view! {
                        <Button
                            variant=if value.get() == level { ButtonVariant::Primary } else { ButtonVariant::Secondary }
                            size=ButtonSize::Sm
                            on_click=Callback::from(move |_: ev::MouseEvent| {
                                set_value.set(level);
                            })
                        >
                            {label}
                        </Button>
                    }
                })
                .collect::<Vec<_>>()}
        </div>
    }
}

// ── TaskForm ──────────────────────────────────────────────────────────────────

#[component]
pub fn TaskForm(
    #[prop(optional)] task: Option<Task>,
    #[prop(optional)] on_submit: Option<Callback<(TaskFormData,)>>,
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let is_edit = task.is_some();

    let initial_name = task
        .as_ref()
        .map(|t| t.task_name.clone())
        .unwrap_or_default();
    let initial_desc = task
        .as_ref()
        .and_then(|t| t.task_description.clone())
        .unwrap_or_default();
    let initial_keywords: Vec<String> = task
        .as_ref()
        .map(|t| t.task_keywords.iter().cloned().collect())
        .unwrap_or_default();
    let initial_priority = task.as_ref().map(|t| t.task_priority).unwrap_or(0);
    let initial_deadline = task.as_ref().and_then(|t| t.task_deadline);

    let (name, set_name) = signal(initial_name);
    let (description, set_description) = signal(initial_desc);
    let (keywords, set_keywords) = signal(initial_keywords);
    let (priority, set_priority) = signal(initial_priority);
    let (deadline, set_deadline) = signal(initial_deadline);
    let (name_error, set_name_error) = signal(Option::<String>::None);

    let handle_submit = move |_ev: ev::SubmitEvent| {
        let n = name.get();
        if n.trim().is_empty() {
            set_name_error.set(Some("Task name is required.".to_string()));
            return;
        }
        set_name_error.set(None);

        let desc = {
            let d = description.get();
            if d.trim().is_empty() { None } else { Some(d) }
        };

        if let Some(cb) = on_submit {
            cb.run((TaskFormData {
                name: n,
                description: desc,
                keywords: keywords.get(),
                priority: priority.get(),
                deadline: deadline.get(),
            },));
        }
    };

    let handle_cancel = move |_ev: ev::MouseEvent| {
        if let Some(cb) = on_cancel {
            cb.run(());
        }
    };

    view! {
        <Form on_submit=Callback::from(handle_submit)>
            <FormGroup label="Task Name".to_string() required=true>
                <input
                    type="text"
                    class="input-field"
                    class=("input-error", move || name_error.get().is_some())
                    placeholder="Enter task name…"
                    prop:value=name
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                    required=true
                />
                {move || {
                    name_error.get().map(|e| view! {
                        <span class="form-error">{e}</span>
                    })
                }}
            </FormGroup>

            <FormGroup label="Description".to_string()>
                <textarea
                    class="input-field task-form-textarea"
                    placeholder="Enter description (optional)…"
                    prop:value=description
                    on:input=move |ev| set_description.set(event_target_value(&ev))
                    rows="3"
                />
            </FormGroup>

            <FormGroup label="Tags / Keywords".to_string()>
                <TagInput
                    tags=keywords
                    set_tags=set_keywords
                    placeholder="Type a tag and press Enter or comma…".to_string()
                />
            </FormGroup>

            <FormGroup label="Priority".to_string()>
                <PrioritySelector value=priority set_value=set_priority />
            </FormGroup>

            <FormGroup label="Deadline".to_string()>
                <DatePicker value=deadline set_value=set_deadline />
            </FormGroup>

            <FormActions>
                {if on_cancel.is_some() {
                    view! {
                        <Button
                            variant=ButtonVariant::Secondary
                            size=ButtonSize::Md
                            on_click=Callback::from(handle_cancel)
                        >
                            "Cancel"
                        </Button>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                <Button variant=ButtonVariant::Primary size=ButtonSize::Md>
                    {if is_edit { "Save Changes" } else { "Create Task" }}
                </Button>
            </FormActions>
        </Form>
    }
}
