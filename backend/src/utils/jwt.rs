use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};

const JWT_SECRET: &[u8] = b"your-secret-key-at-least-32-bytes-long-for-security";
const JWT_ALGORITHM: Algorithm = Algorithm::HS256;
const ACCESS_TOKEN_EXPIRE_HOURS: i64 = 24;
const REFRESH_TOKEN_EXPIRE_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub user_id: i64,
    pub username: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl JwtClaims {
    pub fn new(user_id: i64, username: String, role: String) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(ACCESS_TOKEN_EXPIRE_HOURS)).timestamp();
        let iat = now.timestamp();
        Self {
            sub: user_id.to_string(),
            user_id,
            username,
            role,
            exp,
            iat,
        }
    }

    pub fn with_expiration(user_id: i64, username: String, role: String, hours: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(hours)).timestamp();
        let iat = now.timestamp();
        Self {
            sub: user_id.to_string(),
            user_id,
            username,
            role,
            exp,
            iat,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub user_id: i64,
    pub token_type: String,
    pub exp: i64,
    pub iat: i64,
}

impl RefreshTokenClaims {
    pub fn new(user_id: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::days(REFRESH_TOKEN_EXPIRE_DAYS)).timestamp();
        let iat = now.timestamp();
        Self {
            sub: user_id.to_string(),
            user_id,
            token_type: "refresh".to_string(),
            exp,
            iat,
        }
    }
}

pub fn generate_access_token(user_id: i64, username: String, role: String) -> Result<String> {
    let claims = JwtClaims::new(user_id, username, role);
    encode_token(&claims)
}

pub fn generate_refresh_token(user_id: i64) -> Result<String> {
    let claims = RefreshTokenClaims::new(user_id);
    encode_refresh_token(&claims)
}

pub fn generate_token_pair(
    user_id: i64,
    username: String,
    role: String,
) -> Result<(String, String)> {
    let access_token = generate_access_token(user_id, username.clone(), role)?;
    let refresh_token = generate_refresh_token(user_id)?;
    Ok((access_token, refresh_token))
}

fn encode_token(claims: &JwtClaims) -> Result<String> {
    let header = Header::new(JWT_ALGORITHM);
    encode(&header, claims, &EncodingKey::from_secret(JWT_SECRET)).context("生成 Access Token 失败")
}

fn encode_refresh_token(claims: &RefreshTokenClaims) -> Result<String> {
    let header = Header::new(JWT_ALGORITHM);
    encode(&header, claims, &EncodingKey::from_secret(JWT_SECRET))
        .context("生成 Refresh Token 失败")
}

pub fn verify_access_token(token: &str) -> Result<TokenData<JwtClaims>> {
    decode_token(token)
}

pub fn verify_refresh_token(token: &str) -> Result<TokenData<RefreshTokenClaims>> {
    decode_refresh_token(token)
}

fn decode_token(token: &str) -> Result<TokenData<JwtClaims>> {
    decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::new(JWT_ALGORITHM),
    )
    .map_err(|e| anyhow::anyhow!("验证 Access Token 失败: {}", e))
}

fn decode_refresh_token(token: &str) -> Result<TokenData<RefreshTokenClaims>> {
    decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::new(JWT_ALGORITHM),
    )
    .map_err(|e| anyhow::anyhow!("验证 Refresh Token 失败: {}", e))
}

pub fn get_user_id_from_token(token: &str) -> Result<i64> {
    let token_data = verify_access_token(token)?;
    Ok(token_data.claims.user_id)
}

pub fn is_token_expired(token: &str) -> bool {
    match verify_access_token(token) {
        Ok(_) => false,
        Err(e) => e.to_string().contains("ExpiredSignature"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_USER_ID: i64 = 12345;
    const TEST_USERNAME: &str = "testuser";
    const TEST_ROLE: &str = "user";

    #[test]
    fn test_generate_and_verify_access_token() {
        let token = generate_access_token(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        )
        .unwrap();

        assert!(!token.is_empty());

        let claims = verify_access_token(&token).unwrap();
        assert_eq!(claims.claims.user_id, TEST_USER_ID);
        assert_eq!(claims.claims.username, TEST_USERNAME);
        assert_eq!(claims.claims.role, TEST_ROLE);
    }

    #[test]
    fn test_generate_refresh_token() {
        let token = generate_refresh_token(TEST_USER_ID).unwrap();

        assert!(!token.is_empty());

        let claims = verify_refresh_token(&token).unwrap();
        assert_eq!(claims.claims.user_id, TEST_USER_ID);
        assert_eq!(claims.claims.token_type, "refresh");
    }

    #[test]
    fn test_generate_token_pair() {
        let (access_token, refresh_token) = generate_token_pair(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        )
        .unwrap();

        assert!(!access_token.is_empty());
        assert!(!refresh_token.is_empty());

        let access_claims = verify_access_token(&access_token).unwrap();
        let refresh_claims = verify_refresh_token(&refresh_token).unwrap();

        assert_eq!(access_claims.claims.user_id, TEST_USER_ID);
        assert_eq!(refresh_claims.claims.user_id, TEST_USER_ID);
    }

    #[test]
    fn test_verify_invalid_token() {
        let result = verify_access_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_tampered_token() {
        let token = generate_access_token(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        )
        .unwrap();

        let tampered: String = token.chars().rev().collect();
        let result = verify_access_token(&tampered);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_user_id_from_token() {
        let token = generate_access_token(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        )
        .unwrap();

        let user_id = get_user_id_from_token(&token).unwrap();
        assert_eq!(user_id, TEST_USER_ID);
    }

    #[test]
    fn test_token_expiration_check() {
        let token = generate_access_token(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        )
        .unwrap();

        assert!(!is_token_expired(&token));
    }

    #[test]
    fn test_expired_token() {
        let claims = JwtClaims::with_expiration(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
            -1,
        );
        let expired_token = encode(
            &Header::new(JWT_ALGORITHM),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .unwrap();

        let result = verify_access_token(&expired_token);
        assert!(result.is_err());
        assert!(is_token_expired(&expired_token));
    }

    #[test]
    fn test_empty_token() {
        let result = verify_access_token("");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_secret_key() {
        let token = generate_access_token(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        )
        .unwrap();

        let wrong_decode = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(b"wrong_secret_key_not_the_real_one"),
            &Validation::new(JWT_ALGORITHM),
        );
        assert!(wrong_decode.is_err());
    }

    #[test]
    fn test_wrong_algorithm() {
        let claims = JwtClaims::new(
            TEST_USER_ID,
            TEST_USERNAME.to_string(),
            TEST_ROLE.to_string(),
        );
        let token = encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_secret(JWT_SECRET),
        );
        assert!(token.is_err());
    }

    #[test]
    fn test_missing_claims_fields() {
        #[derive(Serialize)]
        struct PartialClaims {
            sub: String,
        }

        let partial_claims = PartialClaims {
            sub: TEST_USER_ID.to_string(),
        };
        let token = encode(
            &Header::new(JWT_ALGORITHM),
            &partial_claims,
            &EncodingKey::from_secret(JWT_SECRET),
        )
        .unwrap();

        let result = verify_access_token(&token);
        assert!(result.is_err());
    }
}
