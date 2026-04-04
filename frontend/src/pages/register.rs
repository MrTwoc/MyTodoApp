use crate::api::auth::{RegisterRequest, register};
use crate::components::button::{Button, ButtonVariant};
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::{Input, InputType};
use crate::store::{use_api_client, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

#[derive(Debug, Clone, PartialEq)]
enum PasswordStrength {
    Weak,
    Fair,
    Good,
    Strong,
}

impl PasswordStrength {
    fn from_password(password: &str) -> Self {
        let mut score = 0;
        if password.len() >= 6 {
            score += 1;
        }
        if password.len() >= 10 {
            score += 1;
        }
        if password.chars().any(|c| c.is_uppercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_lowercase()) {
            score += 1;
        }
        if password.chars().any(|c| c.is_numeric()) {
            score += 1;
        }
        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        }

        if score <= 2 {
            PasswordStrength::Weak
        } else if score <= 3 {
            PasswordStrength::Fair
        } else if score <= 4 {
            PasswordStrength::Good
        } else {
            PasswordStrength::Strong
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Fair => "Fair",
            PasswordStrength::Good => "Good",
            PasswordStrength::Strong => "Strong",
        }
    }

    fn as_class(&self) -> &'static str {
        match self {
            PasswordStrength::Weak => "weak",
            PasswordStrength::Fair => "fair",
            PasswordStrength::Good => "good",
            PasswordStrength::Strong => "strong",
        }
    }
}

#[component]
pub fn RegisterPage() -> impl IntoView {
    let user_store = use_user_store();
    let client = use_api_client();
    let navigate = use_navigate();

    let user_store_for_effect = user_store.clone();
    let navigate_for_effect = navigate.clone();
    Effect::new(move || {
        if user_store_for_effect.is_authenticated() {
            navigate_for_effect(
                "/dashboard",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    let (username, set_username) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let (show_confirm_password, set_show_confirm_password) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);

    let (username_error, set_username_error) = signal(Option::<String>::None);
    let (email_error, set_email_error) = signal(Option::<String>::None);
    let (password_error, set_password_error) = signal(Option::<String>::None);
    let (confirm_password_error, set_confirm_password_error) = signal(Option::<String>::None);

    let password_strength = move || {
        let pwd = password.get();
        if pwd.is_empty() {
            None
        } else {
            Some(PasswordStrength::from_password(&pwd))
        }
    };

    let passwords_match = move || {
        let pwd = password.get();
        let confirm = confirm_password.get();
        if confirm.is_empty() {
            true
        } else {
            pwd == confirm
        }
    };

    let validate_username = move |username_val: &str| {
        if username_val.is_empty() {
            set_username_error.set(Some("Username is required".to_string()));
            false
        } else if username_val.len() < 3 {
            set_username_error.set(Some("Username must be at least 3 characters".to_string()));
            false
        } else if !username_val
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_')
        {
            set_username_error.set(Some(
                "Username can only contain letters, numbers, and underscores".to_string(),
            ));
            false
        } else {
            set_username_error.set(None);
            true
        }
    };

    let validate_email = move |email_val: &str| {
        if email_val.is_empty() {
            set_email_error.set(Some("Email is required".to_string()));
            false
        } else if !email_val.contains('@') || !email_val.contains('.') {
            set_email_error.set(Some("Please enter a valid email".to_string()));
            false
        } else {
            set_email_error.set(None);
            true
        }
    };

    let validate_password = move |password_val: &str| {
        if password_val.is_empty() {
            set_password_error.set(Some("Password is required".to_string()));
            false
        } else if password_val.len() < 6 {
            set_password_error.set(Some("Password must be at least 6 characters".to_string()));
            false
        } else {
            set_password_error.set(None);
            true
        }
    };

    let validate_confirm_password = move |confirm_val: &str| {
        if confirm_val.is_empty() {
            set_confirm_password_error.set(Some("Please confirm your password".to_string()));
            false
        } else if confirm_val != password.get() {
            set_confirm_password_error.set(Some("Passwords do not match".to_string()));
            false
        } else {
            set_confirm_password_error.set(None);
            true
        }
    };

    let on_submit = move |_: ev::SubmitEvent| {
        let username_val = username.get();
        let email_val = email.get();
        let phone_val = phone.get();
        let password_val = password.get();
        let confirm_val = confirm_password.get();

        let username_valid = validate_username(&username_val);
        let email_valid = validate_email(&email_val);
        let password_valid = validate_password(&password_val);
        let confirm_valid = validate_confirm_password(&confirm_val);

        if !username_valid || !email_valid || !password_valid || !confirm_valid {
            return;
        }

        set_loading.set(true);
        set_error.set(None);

        let client_clone = client.clone();
        let user_store_clone = user_store.clone();
        let navigate_clone = navigate.clone();
        let set_loading_clone = set_loading;
        let set_error_clone = set_error;

        wasm_bindgen_futures::spawn_local(async move {
            let req = RegisterRequest {
                username: username_val,
                email: email_val,
                phone: phone_val,
                password: password_val,
            };
            match register(&client_clone, &req).await {
                Ok(resp) => {
                    user_store_clone.login(resp.access_token, resp.user);
                    navigate_clone(
                        "/dashboard",
                        NavigateOptions {
                            replace: true,
                            ..Default::default()
                        },
                    );
                }
                Err(e) => {
                    set_error_clone.set(Some(e.message));
                    set_loading_clone.set(false);
                }
            }
        });
    };

    let toggle_password = move |_| {
        set_show_password.update(|v| *v = !*v);
    };

    let toggle_confirm_password = move |_| {
        set_show_confirm_password.update(|v| *v = !*v);
    };

    view! {
        <div class="auth-page">
            <div class="auth-container">
                <h1 class="auth-title">"Create Account"</h1>
                <p class="auth-subtitle">"Sign up to get started"</p>

                <Form on_submit=Callback::from(on_submit)>
                    <FormGroup label="Username".to_string() required=true error=(move || username_error.get().unwrap_or_default())()>
                        <Input
                            input_type=InputType::Text
                            placeholder="Choose a username".to_string()
                            on_input=Callback::from(move |v: String| {
                                set_username.set(v.clone());
                                if !v.is_empty() {
                                    validate_username(&v);
                                } else {
                                    set_username_error.set(None);
                                }
                            })
                        />
                    </FormGroup>
                    <FormGroup label="Email".to_string() required=true error=(move || email_error.get().unwrap_or_default())()>
                        <Input
                            input_type=InputType::Email
                            placeholder="Enter your email".to_string()
                            on_input=Callback::from(move |v: String| {
                                set_email.set(v.clone());
                                if !v.is_empty() {
                                    validate_email(&v);
                                } else {
                                    set_email_error.set(None);
                                }
                            })
                        />
                    </FormGroup>
                    <FormGroup label="Phone".to_string()>
                        <Input
                            input_type=InputType::Tel
                            placeholder="Enter your phone number (optional)".to_string()
                            on_input=Callback::from(move |v: String| set_phone.set(v))
                        />
                    </FormGroup>
                    <FormGroup label="Password".to_string() required=true error=(move || password_error.get().unwrap_or_default())()>
                        <div class="password-input-wrapper">
                            <Input
                                input_type=((move || if show_password.get() { InputType::Text } else { InputType::Password }))()
                                placeholder="Create a password".to_string()
                                on_input=Callback::from(move |v: String| {
                                    set_password.set(v.clone());
                                    if !v.is_empty() {
                                        validate_password(&v);
                                    } else {
                                        set_password_error.set(None);
                                    }
                                    if !confirm_password.get().is_empty() {
                                        validate_confirm_password(&confirm_password.get());
                                    }
                                })
                            />
                            <button
                                type="button"
                                class="password-toggle"
                                on:click=toggle_password
                            >
                                {move || if show_password.get() { "Hide" } else { "Show" }}
                            </button>
                        </div>
                        {move || {
                            password_strength().map(|strength| {
                                view! {
                                    <div class="password-strength">
                                        <div class="strength-bar">
                                            <div class=("strength-fill", true) class=(strength.as_class(), true)></div>
                                        </div>
                                        <div class="strength-text">{strength.as_str()}</div>
                                    </div>
                                }.into_any()
                            }).unwrap_or_else(|| ().into_any())
                        }}
                    </FormGroup>
                    <FormGroup label="Confirm Password".to_string() required=true error=(move || confirm_password_error.get().unwrap_or_default())()>
                        <div class="password-input-wrapper">
                            <Input
                                input_type=((move || if show_confirm_password.get() { InputType::Text } else { InputType::Password }))()
                                placeholder="Confirm your password".to_string()
                                on_input=Callback::from(move |v: String| {
                                    set_confirm_password.set(v.clone());
                                    if !v.is_empty() {
                                        validate_confirm_password(&v);
                                    } else {
                                        set_confirm_password_error.set(None);
                                    }
                                })
                            />
                            <button
                                type="button"
                                class="password-toggle"
                                on:click=toggle_confirm_password
                            >
                                {move || if show_confirm_password.get() { "Hide" } else { "Show" }}
                            </button>
                        </div>
                        {move || {
                            if !confirm_password.get().is_empty() && !passwords_match() {
                                view! { <div class="password-match-error">"Passwords do not match"</div> }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                    </FormGroup>

                    {move || error.get().map(|msg| {
                        view! { <div class="auth-error">{msg}</div> }.into_any()
                    }).unwrap_or_else(|| ().into_any())}

                    <FormActions>
                        <Button
                            variant=ButtonVariant::Primary
                            full_width=true
                            disabled=((move || loading.get()))()
                        >
                            {move || if loading.get() { "Creating account..." } else { "Sign Up" }}
                        </Button>
                    </FormActions>
                </Form>

                <div class="auth-footer">
                    <span>"Already have an account? "</span>
                    <a href="/login" class="auth-link">"Sign in"</a>
                </div>
            </div>
        </div>
    }
}
