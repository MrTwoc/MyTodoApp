use crate::api::auth::{LoginRequest, login};
use crate::components::button::{Button, ButtonVariant};
use crate::components::form::{Form, FormActions, FormGroup};
use crate::components::input::{Input, InputType};
use crate::store::{use_api_client, use_user_store};
use leptos::ev;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LoginPage() -> impl IntoView {
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

    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (show_password, set_show_password) = signal(false);
    let (remember_me, set_remember_me) = signal(false);
    let (error, set_error) = signal(Option::<String>::None);
    let (loading, set_loading) = signal(false);
    let (email_error, set_email_error) = signal(Option::<String>::None);
    let (password_error, set_password_error) = signal(Option::<String>::None);

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

    let on_submit = move |_: ev::SubmitEvent| {
        let email_val = email.get();
        let password_val = password.get();

        let email_valid = validate_email(&email_val);
        let password_valid = validate_password(&password_val);

        if !email_valid || !password_valid {
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
            let req = LoginRequest {
                email: email_val,
                password: password_val,
            };
            match login(&client_clone, &req).await {
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

    view! {
        <div class="auth-page">
            <div class="auth-container">
                <h1 class="auth-title">"Welcome Back"</h1>
                <p class="auth-subtitle">"Sign in to your account"</p>

                <Form on_submit=Callback::from(on_submit)>
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
                    <FormGroup label="Password".to_string() required=true error=(move || password_error.get().unwrap_or_default())()>
                        <div class="password-input-wrapper">
                            {move || if show_password.get() {
                                view! {
                                    <Input
                                        input_type=InputType::Text
                                        placeholder="Enter your password".to_string()
                                        on_input=Callback::from(move |v: String| {
                                            set_password.set(v.clone());
                                            if !v.is_empty() {
                                                validate_password(&v);
                                            } else {
                                                set_password_error.set(None);
                                            }
                                        })
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <Input
                                        input_type=InputType::Password
                                        placeholder="Enter your password".to_string()
                                        on_input=Callback::from(move |v: String| {
                                            set_password.set(v.clone());
                                            if !v.is_empty() {
                                                validate_password(&v);
                                            } else {
                                                set_password_error.set(None);
                                            }
                                        })
                                    />
                                }.into_any()
                            }}
                            <button
                                type="button"
                                class="password-toggle"
                                on:click=toggle_password
                            >
                                {move || if show_password.get() { "Hide" } else { "Show" }}
                            </button>
                        </div>
                    </FormGroup>

                    <div class="form-options">
                        <label class="checkbox-label">
                            <input
                                type="checkbox"
                                checked=move || remember_me.get()
                                on:change=move |ev| {
                                    let checked = event_target_checked(&ev);
                                    set_remember_me.set(checked);
                                }
                            />
                            <span>"Remember me"</span>
                        </label>
                        <a href="/forgot-password" class="forgot-link">"Forgot password?"</a>
                    </div>

                    {move || error.get().map(|msg| {
                        view! { <div class="auth-error">{msg}</div> }.into_any()
                    }).unwrap_or_else(|| ().into_any())}

                    <FormActions>
                        <Button
                            variant=ButtonVariant::Primary
                            full_width=true
                            disabled=(move || loading.get())()
                        >
                            {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                        </Button>
                    </FormActions>
                </Form>

                <div class="auth-footer">
                    <span>"Don't have an account? "</span>
                    <a href="/register" class="auth-link">"Sign up"</a>
                </div>
            </div>
        </div>
    }
}
