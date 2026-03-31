use crate::utils::jwt::verify_access_token;
use salvo::http::StatusCode;
use salvo::prelude::*;

const AUTH_HEADER: &str = "Authorization";
const BEARER_PREFIX: &str = "Bearer ";

#[handler]
pub async fn auth_check(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let token = extract_token(req);

    match token {
        Some(token) => match verify_access_token(&token) {
            Ok(token_data) => {
                let claims = token_data.claims;
                depot.insert("user_id", claims.user_id);
                depot.insert("username", claims.username);
                depot.insert("role", claims.role);
                ctrl.call_next(req, depot, res).await;
            }
            Err(e) => {
                tracing::warn!("Token verification failed: {:?}", e);
                unauthorized_response(res);
                ctrl.skip_rest();
            }
        },
        None => {
            tracing::warn!("Missing Authorization header");
            unauthorized_response(res);
            ctrl.skip_rest();
        }
    }
}

fn extract_token(req: &Request) -> Option<String> {
    let auth_header = req.header::<String>(AUTH_HEADER)?;
    if auth_header.starts_with(BEARER_PREFIX) {
        Some(auth_header[BEARER_PREFIX.len()..].to_string())
    } else {
        None
    }
}

fn unauthorized_response(res: &mut Response) {
    res.status_code(StatusCode::UNAUTHORIZED);
    res.render(Json(serde_json::json!({
        "error": "Unauthorized",
        "message": "Missing or invalid authentication token"
    })));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::jwt::{generate_access_token, generate_token_pair};
    use salvo::test::{ResponseExt, TestClient};

    #[handler]
    async fn protected_handler(depot: &mut Depot) -> String {
        let user_id = depot.get::<i64>("user_id").copied().unwrap_or(0);
        let username = depot.get::<String>("username").cloned().unwrap_or_default();
        let role = depot.get::<String>("role").cloned().unwrap_or_default();
        format!("user_id:{}, username:{}, role:{}", user_id, username, role)
    }

    #[tokio::test]
    async fn test_auth_with_valid_token() {
        let token = generate_access_token(1, "testuser".to_string(), "user".to_string()).unwrap();

        let router = Router::new().hoop(auth_check).get(protected_handler);
        let service = Service::new(router);

        let content = TestClient::get("http://127.0.0.1:8080/")
            .add_header("Authorization", format!("Bearer {}", token), true)
            .send(&service)
            .await
            .take_string()
            .await
            .unwrap();

        assert!(content.contains("user_id:1"));
        assert!(content.contains("username:testuser"));
        assert!(content.contains("role:user"));
    }

    #[tokio::test]
    async fn test_auth_without_token() {
        let router = Router::new().hoop(auth_check).get(protected_handler);
        let service = Service::new(router);

        let mut res = TestClient::get("http://127.0.0.1:8080/")
            .send(&service)
            .await;

        let status = res.status_code.map(|c| c.as_u16()).unwrap_or(200);
        assert_eq!(status, StatusCode::UNAUTHORIZED.as_u16());

        let json = res.take_json::<serde_json::Value>().await.unwrap();
        assert_eq!(json["error"], "Unauthorized");
    }

    #[tokio::test]
    async fn test_auth_with_invalid_token() {
        let router = Router::new().hoop(auth_check).get(protected_handler);
        let service = Service::new(router);

        let mut res = TestClient::get("http://127.0.0.1:8080/")
            .add_header("Authorization", "Bearer invalid_token_12345", true)
            .send(&service)
            .await;

        let status = res.status_code.map(|c| c.as_u16()).unwrap_or(200);
        assert_eq!(status, StatusCode::UNAUTHORIZED.as_u16());
    }

    #[tokio::test]
    async fn test_auth_without_bearer_prefix() {
        let token = generate_access_token(1, "testuser".to_string(), "user".to_string()).unwrap();

        let router = Router::new().hoop(auth_check).get(protected_handler);
        let service = Service::new(router);

        let mut res = TestClient::get("http://127.0.0.1:8080/")
            .add_header("Authorization", token, true)
            .send(&service)
            .await;

        let status = res.status_code.map(|c| c.as_u16()).unwrap_or(200);
        assert_eq!(status, StatusCode::UNAUTHORIZED.as_u16());
    }
}
