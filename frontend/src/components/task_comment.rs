use crate::api::task_comment::{
    CreateCommentRequest, TaskComment, UpdateCommentRequest, create_comment, delete_comment,
    get_task_comments, update_comment,
};
use crate::api::user::get_user as api_get_user;
use crate::store::{use_api_client, use_user_store};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn format_comment_timestamp(ts: i64) -> String {
    let ms = (ts * 1000) as f64;
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
    let year = date.get_full_year();
    let month = date.get_month() + 1;
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!(
        "{:04}/{:02}/{:02} {:02}:{:02}",
        year, month, day, hours, minutes
    )
}

fn time_ago(ts: i64) -> String {
    let now_ms = js_sys::Date::now();
    let then_ms = (ts * 1000) as f64;
    let diff_ms = now_ms - then_ms;
    let diff_secs = (diff_ms / 1000.0) as i64;

    if diff_secs < 60 {
        "just now".to_string()
    } else if diff_secs < 3600 {
        format!("{}m ago", diff_secs / 60)
    } else if diff_secs < 86400 {
        format!("{}h ago", diff_secs / 3600)
    } else if diff_secs < 604800 {
        format!("{}d ago", diff_secs / 86400)
    } else {
        format_comment_timestamp(ts)
    }
}

#[component]
pub fn TaskComments(task_id: u64) -> impl IntoView {
    let client = use_api_client();
    let user_store = use_user_store();

    let (comments, set_comments) = signal(Vec::<TaskComment>::new());
    let (user_names, set_user_names) = signal(std::collections::HashMap::<u64, String>::new());
    let (user_avatars, set_user_avatars) =
        signal(std::collections::HashMap::<u64, Option<String>>::new());
    let new_comment = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let editing_comment_id = RwSignal::new(Option::<u64>::None);
    let edit_content = RwSignal::new(String::new());

    let load_comments = {
        let client = client.clone();
        let task_id = task_id;
        let set_comments = set_comments.clone();
        let set_user_names = set_user_names.clone();
        let set_user_avatars = set_user_avatars.clone();
        move || {
            let client = client.clone();
            let task_id = task_id;
            let set_comments = set_comments.clone();
            let set_user_names = set_user_names.clone();
            let set_user_avatars = set_user_avatars.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match get_task_comments(&client, task_id).await {
                    Ok(comments_list) => {
                        let mut unique_users = std::collections::HashSet::new();
                        for c in &comments_list {
                            unique_users.insert(c.user_id);
                        }

                        let mut names = user_names.get();
                        let mut avatars = user_avatars.get();
                        for uid in unique_users {
                            if !names.contains_key(&uid) {
                                match api_get_user(&client, uid).await {
                                    Ok(user) => {
                                        names.insert(uid, user.username.clone());
                                        avatars.insert(uid, user.avatar.clone());
                                    }
                                    Err(_) => {
                                        names.insert(uid, format!("User {}", uid));
                                    }
                                }
                            }
                        }
                        set_user_names.set(names);
                        set_user_avatars.set(avatars);
                        set_comments.set(comments_list);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load comments: {}", e.message);
                    }
                }
            });
        }
    };

    Effect::new({
        let load_comments = load_comments.clone();
        move |_| {
            load_comments();
        }
    });

    let submit_comment = {
        let client = client.clone();
        let task_id = task_id;
        let new_comment = new_comment.clone();
        let is_submitting = is_submitting.clone();
        let load_comments = load_comments.clone();
        move || {
            let content = new_comment.get();
            if content.trim().is_empty() {
                return;
            }
            let client = client.clone();
            let task_id = task_id;
            let new_comment = new_comment.clone();
            let is_submitting = is_submitting.clone();
            let load_comments = load_comments.clone();

            is_submitting.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let req = CreateCommentRequest {
                    content: content.clone(),
                    parent_id: None,
                };
                match create_comment(&client, task_id, &req).await {
                    Ok(_) => {
                        new_comment.set(String::new());
                        load_comments();
                    }
                    Err(e) => {
                        tracing::error!("Failed to create comment: {}", e.message);
                    }
                }
                is_submitting.set(false);
            });
        }
    };

    let current_user_id = move || user_store.state.get().profile.map(|p| p.user_id);

    view! {
        <div class="task-detail-card">
            <div class="task-detail-card-header">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                </svg>
                <h3 class="task-detail-card-title">"Comments"</h3>
                <span class="task-detail-card-count">
                    {move || comments.get().len()}
                </span>
            </div>
            <div class="task-detail-card-body">
                // Comment input
                <div class="comment-input-area">
                    <textarea
                        class="comment-input"
                        placeholder="Write a comment..."
                        rows="3"
                        prop:value=move || new_comment.get()
                        on:input=move |ev| {
                            let val = ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                            new_comment.set(val);
                        }
                        on:keydown={
                            let submit_comment = submit_comment.clone();
                            move |ev| {
                                if ev.key() == "Enter" && ev.shift_key() == false {
                                    ev.prevent_default();
                                    let content = new_comment.get();
                                    if !content.trim().is_empty() && !is_submitting.get() {
                                        submit_comment();
                                    }
                                }
                            }
                        }
                    />
                    <div class="comment-input-actions">
                        <button
                            class="comment-submit-btn"
                            disabled=move || is_submitting.get() || new_comment.get().trim().is_empty()
                            on:click={
                                let submit_comment = submit_comment.clone();
                                move |_: web_sys::MouseEvent| { submit_comment(); }
                            }
                        >
                            {move || if is_submitting.get() { "Posting..." } else { "Post Comment" }}
                        </button>
                    </div>
                </div>

                // Comment list
                <div class="comment-list">
                    {move || {
                        let comments_list = comments.get();
                        if comments_list.is_empty() {
                            view! {
                                <div class="task-detail-empty-state">
                                    <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                                    </svg>
                                    <p>"No comments yet. Be the first to comment!"</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                {comments_list.into_iter().map(|comment| {
                                    let comment_id = comment.comment_id;
                                    let user_id = comment.user_id;
                                    let content = comment.content.clone();
                                    let created_at = comment.created_at;
                                    let updated_at = comment.updated_at;
                                    let is_editing = move || editing_comment_id.get() == Some(comment_id);
                                    let is_owner = move || current_user_id() == Some(user_id);
                                    let username = user_names.get().get(&user_id).cloned().unwrap_or_else(|| format!("User {}", user_id));
                                    let avatar = user_avatars.get().get(&user_id).cloned().flatten();

                                    // Clone client and load_comments for inner closures
                                    let client_for_save = client.clone();
                                    let load_comments_for_save = load_comments.clone();
                                    let editing_comment_id_for_save = editing_comment_id.clone();
                                    let edit_content_for_save = edit_content.clone();
                                    let client_for_delete = client.clone();
                                    let load_comments_for_delete = load_comments.clone();

                                    view! {
                                        <div class="comment-item">
                                            <div class="comment-avatar">
                                                {if let Some(_avatar_url) = avatar {
                                                    view! {
                                                        <div class="comment-avatar-img">{username.chars().next().unwrap_or('U').to_string()}</div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="comment-avatar-placeholder">
                                                            {username.chars().next().unwrap_or('U').to_string()}
                                                        </div>
                                                    }.into_any()
                                                }}
                                            </div>
                                            <div class="comment-body">
                                                <div class="comment-header">
                                                    <span class="comment-author">{username}</span>
                                                    <span class="comment-time" title=format_comment_timestamp(created_at)>
                                                        {time_ago(created_at)}
                                                    </span>
                                                    {if updated_at.is_some() {
                                                        view! {
                                                            <span class="comment-edited">"(edited)"</span>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }}
                                                </div>
                                                {{
                                                    let content = content.clone();
                                                    move || {
                                                    if is_editing() {
                                                        let client_save = client_for_save.clone();
                                                        let load_save = load_comments_for_save.clone();
                                                        let ecid_save = editing_comment_id_for_save.clone();
                                                        let econtent_save = edit_content_for_save.clone();
                                                        view! {
                                                            <div class="comment-edit-area">
                                                                <textarea
                                                                    class="comment-edit-input"
                                                                    rows="3"
                                                                    prop:value=move || edit_content.get()
                                                                    on:input=move |ev| {
                                                                        let val = ev.target().unwrap().unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                                                                        edit_content.set(val);
                                                                    }
                                                                />
                                                                <div class="comment-edit-actions">
                                                                    <button class="comment-edit-cancel" on:click=move |_: web_sys::MouseEvent| {
                                                                        editing_comment_id.set(None);
                                                                        edit_content.set(String::new());
                                                                    }>"Cancel"</button>
                                                                    <button class="comment-edit-save" on:click=move |_| {
                                                                        let new_content = econtent_save.get();
                                                                        if !new_content.trim().is_empty() {
                                                                            let client_save = client_save.clone();
                                                                            let load_save = load_save.clone();
                                                                            wasm_bindgen_futures::spawn_local(async move {
                                                                                let req = UpdateCommentRequest { content: new_content.clone() };
                                                                                match update_comment(&client_save, comment_id, &req).await {
                                                                                    Ok(_) => {
                                                                                        ecid_save.set(None);
                                                                                        econtent_save.set(String::new());
                                                                                        load_save();
                                                                                    }
                                                                                    Err(e) => tracing::error!("Failed to update comment: {}", e.message),
                                                                                }
                                                                            });
                                                                        }
                                                                    }>"Save"</button>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <p class="comment-content">{content.clone()}</p>
                                                        }.into_any()
                                                    }
                                                    }
                                                }}
                                                {move || {
                                                    if is_owner() && !is_editing() {
                                                        let client_del = client_for_delete.clone();
                                                        let load_del = load_comments_for_delete.clone();
                                                        let content_for_edit = content.clone();
                                                        view! {
                                                            <div class="comment-actions">
                                                                <button class="comment-action-btn" on:click=move |_| {
                                                                    editing_comment_id.set(Some(comment_id));
                                                                    edit_content.set(content_for_edit.clone());
                                                                }>
                                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
                                                                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
                                                                    </svg>
                                                                    "Edit"
                                                                </button>
                                                                <button class="comment-action-btn comment-delete-btn" on:click=move |_| {
                                                                    let client_del = client_del.clone();
                                                                    let load_del = load_del.clone();
                                                                    wasm_bindgen_futures::spawn_local(async move {
                                                                        match delete_comment(&client_del, comment_id).await {
                                                                            Ok(_) => load_del(),
                                                                            Err(e) => tracing::error!("Failed to delete comment: {}", e.message),
                                                                        }
                                                                    });
                                                                }>
                                                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                                        <polyline points="3 6 5 6 21 6"/>
                                                                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                                                                    </svg>
                                                                    "Delete"
                                                                </button>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
