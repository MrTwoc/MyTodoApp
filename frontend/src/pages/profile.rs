use crate::api::user::{ChangePasswordRequest, UpdateUserRequest, change_password, update_user};
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::card::Card;
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::{Input, InputType};
use crate::store::{use_api_client, use_user_store};
use leptos::ev;
use leptos::prelude::*;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let user_store = use_user_store();
    let client = use_api_client();

    let (editing, set_editing) = signal(false);
    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (save_loading, set_save_loading) = signal(false);
    let (save_error, set_save_error) = signal(Option::<String>::None);
    let (save_success, set_save_success) = signal(false);

    let (old_password, set_old_password) = signal(String::new());
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (show_password_section, set_show_password_section) = signal(false);
    let (password_loading, set_password_loading) = signal(false);
    let (password_error, set_password_error) = signal(Option::<String>::None);
    let (password_success, set_password_success) = signal(false);

    Effect::new({
        let user_store = user_store.clone();
        move || {
            if let Some(p) = user_store.profile() {
                set_username.set(p.username.clone());
                set_email.set(p.email.clone());
                set_phone.set(p.phone.clone());
                set_description.set(p.description.clone().unwrap_or_default());
            }
        }
    });

    let client_for_save = client.clone();
    let user_store_for_save = user_store.clone();
    let save_profile = move |_: ev::SubmitEvent| {
        let Some(p) = user_store_for_save.profile() else {
            return;
        };
        let user_id = p.user_id;

        set_save_loading.set(true);
        set_save_error.set(None);
        set_save_success.set(false);

        let client_clone = client_for_save.clone();
        let user_store_clone = user_store_for_save.clone();
        let set_save_loading_clone = set_save_loading;
        let set_save_error_clone = set_save_error;
        let set_save_success_clone = set_save_success;
        let set_editing_clone = set_editing;

        let username_val = username.get();
        let email_val = email.get();
        let phone_val = phone.get();
        let desc_val = description.get();

        wasm_bindgen_futures::spawn_local(async move {
            let req = UpdateUserRequest {
                username: Some(username_val),
                email: Some(email_val),
                phone: Some(phone_val),
                description: Some(desc_val),
                avatar: None,
            };
            match update_user(&client_clone, user_id, &req).await {
                Ok(updated_user) => {
                    user_store_clone.update_profile(updated_user);
                    set_save_loading_clone.set(false);
                    set_save_success_clone.set(true);
                    set_editing_clone.set(false);
                }
                Err(e) => {
                    set_save_error_clone.set(Some(e.message));
                    set_save_loading_clone.set(false);
                }
            }
        });
    };

    let user_store_for_start = user_store.clone();
    let start_edit = move |_| {
        set_editing.set(true);
        set_save_error.set(None);
        set_save_success.set(false);
        if let Some(p) = user_store_for_start.profile() {
            set_username.set(p.username.clone());
            set_email.set(p.email.clone());
            set_phone.set(p.phone.clone());
            set_description.set(p.description.clone().unwrap_or_default());
        }
    };

    let user_store_for_cancel = user_store.clone();
    let cancel_edit = move |_| {
        set_editing.set(false);
        set_save_error.set(None);
        set_save_success.set(false);
        if let Some(p) = user_store_for_cancel.profile() {
            set_username.set(p.username.clone());
            set_email.set(p.email.clone());
            set_phone.set(p.phone.clone());
            set_description.set(p.description.clone().unwrap_or_default());
        }
    };

    let toggle_password_section = move |_| {
        set_show_password_section.update(|v| *v = !*v);
        set_password_error.set(None);
        set_password_success.set(false);
        set_old_password.set(String::new());
        set_new_password.set(String::new());
        set_confirm_password.set(String::new());
    };

    let user_store_for_pwd = user_store.clone();
    let change_pwd = move |_: ev::SubmitEvent| {
        let Some(p) = user_store_for_pwd.profile() else {
            return;
        };
        let user_id = p.user_id;

        let old_val = old_password.get();
        let new_val = new_password.get();
        let confirm_val = confirm_password.get();

        if old_val.is_empty() || new_val.is_empty() || confirm_val.is_empty() {
            set_password_error.set(Some("Please fill in all password fields".to_string()));
            return;
        }
        if new_val != confirm_val {
            set_password_error.set(Some("New passwords do not match".to_string()));
            return;
        }
        if new_val.len() < 6 {
            set_password_error.set(Some(
                "New password must be at least 6 characters".to_string(),
            ));
            return;
        }

        set_password_loading.set(true);
        set_password_error.set(None);
        set_password_success.set(false);

        let client_clone = client.clone();
        let set_password_loading_clone = set_password_loading;
        let set_password_error_clone = set_password_error;
        let set_password_success_clone = set_password_success;

        wasm_bindgen_futures::spawn_local(async move {
            let req = ChangePasswordRequest {
                old_password: old_val,
                new_password: new_val,
            };
            match change_password(&client_clone, user_id, &req).await {
                Ok(_) => {
                    set_password_loading_clone.set(false);
                    set_password_success_clone.set(true);
                    set_old_password.set(String::new());
                    set_new_password.set(String::new());
                    set_confirm_password.set(String::new());
                }
                Err(e) => {
                    set_password_error_clone.set(Some(e.message));
                    set_password_loading_clone.set(false);
                }
            }
        });
    };

    let save_profile_cb = Callback::from(save_profile);
    let start_edit_cb = Callback::from(start_edit);
    let cancel_edit_cb = Callback::from(cancel_edit);
    let change_pwd_cb = Callback::from(change_pwd);
    let toggle_password_section_cb = Callback::from(toggle_password_section);

    view! {
        <div class="page">
            <header class="page-header">
                <div>
                    <a href="/settings" class="back-btn">"← Back"</a>
                    <h1 class="page-title">"Profile"</h1>
                </div>
            </header>

            <div class="settings-grid">
                <Card title="Personal Information".to_string() subtitle="Your basic account details".to_string()>
                    {move || {
                        if editing.get() {
                            view! {
                                <Form on_submit=save_profile_cb>
                                    <FormGroup label="Username".to_string() required=true>
                                        <Input
                                            input_type=InputType::Text
                                            value=username.get()
                                            on_input=Callback::from(move |v: String| set_username.set(v))
                                        />
                                    </FormGroup>
                                    <FormGroup label="Email".to_string() required=true>
                                        <Input
                                            input_type=InputType::Email
                                            value=email.get()
                                            on_input=Callback::from(move |v: String| set_email.set(v))
                                        />
                                    </FormGroup>
                                    <FormGroup label="Phone".to_string()>
                                        <Input
                                            input_type=InputType::Tel
                                            value=phone.get()
                                            on_input=Callback::from(move |v: String| set_phone.set(v))
                                        />
                                    </FormGroup>
                                    <FormGroup label="Bio".to_string()>
                                        <textarea
                                            class="input-field profile-textarea"
                                            rows="3"
                                            on:input=move |ev| set_description.set(event_target_value(&ev))
                                        >{description.get()}</textarea>
                                    </FormGroup>

                                    {move || save_error.get().map(|msg| {
                                        view! { <div class="auth-error">{msg}</div> }.into_any()
                                    }).unwrap_or_else(|| ().into_any())}

                                    {move || if save_success.get() {
                                        view! { <div class="auth-success">"Profile updated successfully"</div> }.into_any()
                                    } else {
                                        ().into_any()
                                    }}

                                    <FormActions>
                                        <Button
                                            variant=ButtonVariant::Secondary
                                            size=ButtonSize::Sm
                                            on_click=cancel_edit_cb
                                        >"Cancel"</Button>
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Sm
                                            disabled=save_loading.get()
                                        >
                                            {move || if save_loading.get() { "Saving..." } else { "Save" }}
                                        </Button>
                                    </FormActions>
                                </Form>
                            }.into_any()
                        } else {
                            user_store.profile().map(|p| {
                                view! {
                                    <div class="profile-info">
                                        <div class="profile-field">
                                            <span class="profile-label">"Username"</span>
                                            <span class="profile-value">{p.username.clone()}</span>
                                        </div>
                                        <div class="profile-field">
                                            <span class="profile-label">"Email"</span>
                                            <span class="profile-value">{p.email.clone()}</span>
                                        </div>
                                        <div class="profile-field">
                                            <span class="profile-label">"Phone"</span>
                                            <span class="profile-value">{if p.phone.is_empty() { "Not set".to_string() } else { p.phone.clone() }}</span>
                                        </div>
                                        {if let Some(desc) = &p.description {
                                            if !desc.is_empty() {
                                                view! {
                                                    <div class="profile-field">
                                                        <span class="profile-label">"Bio"</span>
                                                        <span class="profile-value">{desc.clone()}</span>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        } else {
                                            ().into_any()
                                        }}
                                        <div class="profile-field">
                                            <span class="profile-label">"Member since"</span>
                                            <span class="profile-value">{format_timestamp(p.reg_time)}</span>
                                        </div>
                                    </div>
                                }.into_any()
                            }).unwrap_or_else(|| view! { <p>"Not logged in"</p> }.into_any())
                        }
                    }}
                    {move || if !editing.get() {
                        view! {
                            <div class="profile-actions">
                                <Button
                                    variant=ButtonVariant::Secondary
                                    size=ButtonSize::Sm
                                    on_click=start_edit_cb
                                >"Edit Profile"</Button>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                </Card>

                <Card title="Security".to_string() subtitle="Manage your password".to_string()>
                    <Button
                        variant=ButtonVariant::Secondary
                        size=ButtonSize::Sm
                        on_click=toggle_password_section_cb
                    >
                        {move || if show_password_section.get() { "Hide" } else { "Change Password" }}
                    </Button>

                    {move || if show_password_section.get() {
                        view! {
                            <div class="profile-password-form">
                                <Form on_submit=change_pwd_cb>
                                    <FormGroup label="Current Password".to_string() required=true>
                                        <Input
                                            input_type=InputType::Password
                                            placeholder="Enter current password".to_string()
                                            on_input=Callback::from(move |v: String| set_old_password.set(v))
                                        />
                                    </FormGroup>
                                    <FormGroup label="New Password".to_string() required=true>
                                        <Input
                                            input_type=InputType::Password
                                            placeholder="Enter new password".to_string()
                                            on_input=Callback::from(move |v: String| {
                                                set_new_password.set(v.clone());
                                                set_password_success.set(false);
                                            })
                                        />
                                    </FormGroup>
                                    <FormGroup label="Confirm New Password".to_string() required=true>
                                        <Input
                                            input_type=InputType::Password
                                            placeholder="Confirm new password".to_string()
                                            on_input=Callback::from(move |v: String| {
                                                set_confirm_password.set(v.clone());
                                                set_password_success.set(false);
                                            })
                                        />
                                    </FormGroup>

                                    {move || password_error.get().map(|msg| {
                                        view! { <div class="auth-error">{msg}</div> }.into_any()
                                    }).unwrap_or_else(|| ().into_any())}

                                    {move || if password_success.get() {
                                        view! { <div class="auth-success">"Password changed successfully"</div> }.into_any()
                                    } else {
                                        ().into_any()
                                    }}

                                    <FormActions>
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Sm
                                            disabled=password_loading.get()
                                        >
                                            {move || if password_loading.get() { "Changing..." } else { "Update Password" }}
                                        </Button>
                                    </FormActions>
                                </Form>
                            </div>
                        }.into_any()
                    } else {
                        ().into_any()
                    }}
                </Card>
            </div>
        </div>
    }
}

fn format_timestamp(timestamp: i64) -> String {
    use chrono::DateTime;
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        dt.format("%B %d, %Y").to_string()
    } else {
        "Unknown".to_string()
    }
}
