use crate::models::user::{User, UserStatus};
use crate::models::user_settings::UserSettings;
use crate::utils::id_generator::generate_user_id;
use crate::utils::utils_passwd;
use anyhow::Result;
use sqlx::{PgPool, Row};

pub struct DbUser;

impl DbUser {
    /// 创建新用户
    pub async fn create_user(
        pool: &PgPool,
        username: &str,
        password: &str,
        email: &str,
        phone: &str,
    ) -> Result<User> {
        let user_id = generate_user_id();
        let user_reg_time = chrono::Utc::now().timestamp();
        let password_hash = utils_passwd::hash_password(password)?;

        let user_teams: Vec<i64> = vec![];
        let user_teams_json = serde_json::to_value(&user_teams).unwrap();
        let user_settings = UserSettings::default();

        let result = sqlx::query(
            r#"
            INSERT INTO users (user_id, user_username, user_password, user_email, user_reg_time, user_phone, user_teams, user_settings)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING user_id, user_username, user_password, user_email, user_reg_time, user_phone, user_teams, user_last_login_time, user_description, user_avatar, user_status, user_settings
            "#,
        )
        .bind(user_id as i64)
        .bind(username)
        .bind(&password_hash)
        .bind(email)
        .bind(user_reg_time)
        .bind(phone)
        .bind(user_teams_json)
        .bind(serde_json::to_value(&user_settings)?)
        .fetch_one(pool)
        .await?;

        tracing::info!("创建用户成功: user_id = {}", user_id);

        Ok(Self::row_to_user(result)?)
    }

    /// 根据用户ID查询用户
    pub async fn get_user_by_id(pool: &PgPool, user_id: u64) -> Result<Option<User>> {
        let result = sqlx::query(
            r#"
            SELECT user_id, user_username, user_password, user_email, user_reg_time, user_phone, user_teams, user_last_login_time, user_description, user_avatar, user_status, user_settings
            FROM users
            WHERE user_id = $1
            "#,
        )
        .bind(user_id as i64)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_user(row)?)),
            None => Ok(None),
        }
    }

    /// 根据邮箱查询用户
    pub async fn get_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>> {
        let result = sqlx::query(
            r#"
            SELECT user_id, user_username, user_password, user_email, user_reg_time, user_phone, user_teams, user_last_login_time, user_description, user_avatar, user_status, user_settings
            FROM users
            WHERE user_email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_user(row)?)),
            None => Ok(None),
        }
    }

    /// 根据用户名查询用户
    pub async fn get_user_by_username(pool: &PgPool, username: &str) -> Result<Option<User>> {
        let result = sqlx::query(
            r#"
            SELECT user_id, user_username, user_password, user_email, user_reg_time, user_phone, user_teams, user_last_login_time, user_description, user_avatar, user_status, user_settings
            FROM users
            WHERE user_username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        match result {
            Some(row) => Ok(Some(Self::row_to_user(row)?)),
            None => Ok(None),
        }
    }

    /// 更新用户信息（可更新用户名、邮箱、手机号、描述、头像）
    pub async fn update_user(
        pool: &PgPool,
        user_id: u64,
        username: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        description: Option<&str>,
        avatar: Option<&str>,
    ) -> Result<Option<User>> {
        let mut updates = Vec::new();
        let mut param_count = 1usize;

        if username.is_some() {
            updates.push(format!("user_username = ${}", param_count));
            param_count += 1;
        }
        if email.is_some() {
            updates.push(format!("user_email = ${}", param_count));
            param_count += 1;
        }
        if phone.is_some() {
            updates.push(format!("user_phone = ${}", param_count));
            param_count += 1;
        }
        if description.is_some() {
            updates.push(format!("user_description = ${}", param_count));
            param_count += 1;
        }
        if avatar.is_some() {
            updates.push(format!("user_avatar = ${}", param_count));
            param_count += 1;
        }

        if updates.is_empty() {
            return Self::get_user_by_id(pool, user_id).await;
        }

        let mut query = format!(
            "UPDATE users SET {} WHERE user_id = ${} RETURNING user_id, user_username, user_password, user_email, user_reg_time, user_phone, user_teams, user_last_login_time, user_description, user_avatar, user_status, user_settings",
            updates.join(", "),
            param_count
        );

        let mut row_result = sqlx::query(&query);

        if let Some(v) = username {
            row_result = row_result.bind(v);
        }
        if let Some(v) = email {
            row_result = row_result.bind(v);
        }
        if let Some(v) = phone {
            row_result = row_result.bind(v);
        }
        if let Some(v) = description {
            row_result = row_result.bind(v);
        }
        if let Some(v) = avatar {
            row_result = row_result.bind(v);
        }
        row_result = row_result.bind(user_id as i64);

        let result = row_result.fetch_optional(pool).await?;

        match result {
            Some(row) => {
                tracing::info!("更新用户成功: user_id = {}", user_id);
                Ok(Some(Self::row_to_user(row)?))
            }
            None => Ok(None),
        }
    }

    /// 删除用户
    pub async fn delete_user(pool: &PgPool, user_id: u64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE user_id = $1")
            .bind(user_id as i64)
            .execute(pool)
            .await?;

        let affected = result.rows_affected();
        tracing::info!("删除用户: user_id = {}, affected = {}", user_id, affected);
        Ok(affected > 0)
    }

    /// 更新用户头像（便捷函数）
    pub async fn update_user_avatar(
        pool: &PgPool,
        user_id: u64,
        avatar_url: &str,
    ) -> Result<Option<User>> {
        Self::update_user(pool, user_id, None, None, None, None, Some(avatar_url)).await
    }

    /// 将数据库行转换为 User 结构体
    fn row_to_user(row: sqlx::postgres::PgRow) -> Result<User> {
        let user_id: i64 = row.get("user_id");
        let user_username: String = row.get("user_username");
        let user_password: String = row.get("user_password");
        let user_email: String = row.get("user_email");
        let user_reg_time: i64 = row.get("user_reg_time");
        let user_phone: String = row.get("user_phone");
        let user_teams: serde_json::Value = row.get("user_teams");
        let user_teams: Vec<i64> = serde_json::from_value(user_teams).unwrap_or_default();
        let user_last_login_time: Option<i64> = row.get("user_last_login_time");
        let user_description: Option<String> = row.get("user_description");
        let user_avatar: Option<String> = row.get("user_avatar");
        let user_status: String = row.get("user_status");
        let user_settings: serde_json::Value = row.get("user_settings");

        let user_status = match user_status.as_str() {
            "Active" => UserStatus::Active,
            "Inactive" => UserStatus::Inactive,
            _ => UserStatus::Active,
        };

        let user_settings: UserSettings = serde_json::from_value(user_settings).unwrap_or_default();

        Ok(User {
            user_id: user_id as u64,
            user_username,
            user_password,
            user_email,
            user_reg_time,
            user_phone,
            user_teams: user_teams.into_iter().map(|v| v as u64).collect(),
            user_last_login_time,
            user_description,
            user_settings,
            user_avatar,
            user_status,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_user_crud() {
        // 写一个删除语句，提前删除接下来的测试用户

        // 创建数据库连接池
        let pool = crate::db::pool::create_pool().await.unwrap();

        // 创建用户
        let user = DbUser::create_user(
            &pool,
            "testuser_crud11",
            "TestPass123!",
            "test_crud11@example.com",
            "1380013810",
        )
        .await
        .unwrap();

        println!("创建用户: {:?}", user);
        assert_eq!(user.user_username, "testuser_crud11");
        assert_eq!(user.user_email, "test_crud11@example.com");
        assert_eq!(user.user_phone, "1380013810");

        // 查询用户（按ID）
        let found = DbUser::get_user_by_id(&pool, user.user_id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.user_id, user.user_id);
        assert_eq!(found.user_username, "testuser_crud11");

        // 查询用户（按邮箱）
        let found_by_email = DbUser::get_user_by_email(&pool, "test_crud11@example.com")
            .await
            .unwrap();
        assert!(found_by_email.is_some());
        assert_eq!(found_by_email.unwrap().user_id, user.user_id);

        // 查询用户（按用户名）
        let found_by_username = DbUser::get_user_by_username(&pool, "testuser_crud11")
            .await
            .unwrap();
        assert!(found_by_username.is_some());
        assert_eq!(found_by_username.unwrap().user_id, user.user_id);

        // 更新用户
        let updated = DbUser::update_user(
            &pool,
            user.user_id,
            Some("updated_username"),
            None,
            None,
            Some("这是一个测试用户"),
            Some("https://example.com/avatar.png"),
        )
        .await
        .unwrap();

        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.user_username, "updated_username");
        assert_eq!(
            updated.user_description,
            Some("这是一个测试用户".to_string())
        );
        assert_eq!(
            updated.user_avatar,
            Some("https://example.com/avatar.png".to_string())
        );

        // 测试更新头像便捷函数
        let avatar_updated =
            DbUser::update_user_avatar(&pool, user.user_id, "https://new.avatar.png")
                .await
                .unwrap();
        assert!(avatar_updated.is_some());
        assert_eq!(
            avatar_updated.unwrap().user_avatar,
            Some("https://new.avatar.png".to_string())
        );

        // 删除用户
        let deleted = DbUser::delete_user(&pool, user.user_id).await.unwrap();
        assert!(deleted);

        // 确认用户已删除
        let found = DbUser::get_user_by_id(&pool, user.user_id).await.unwrap();
        assert!(found.is_none());
    }
}
