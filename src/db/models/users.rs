use super::accounts::Account;
use crate::db::schema::{sessions, users as users_table, verification_tokens};
use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use diesel::prelude::*;
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use tokio::task;

/// Core user entity containing basic user information.
/// Authentication is handled separately through the accounts system.
#[derive(Queryable, Selectable, Clone, Serialize, Deserialize)]
#[diesel(table_name = users_table)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub created_at: chrono::NaiveDateTime,
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("created_at", &self.created_at)
            .finish()
    }
}

#[derive(Queryable, Selectable, Clone, Serialize, Deserialize, Debug)]
#[diesel(table_name = sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Session {
    pub id: String,
    pub session_token: String,
    pub user_id: i32,
    pub expires: chrono::NaiveDateTime,
}

#[derive(Queryable, Selectable, Clone, Serialize, Deserialize, Debug)]
#[diesel(table_name = verification_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VerificationToken {
    pub identifier: String,
    pub token: String,
    pub expires: chrono::NaiveDateTime,
}

impl User {
    /// Create a new user with the given username
    pub fn create(
        conn: &mut diesel::PgConnection,
        username: String,
    ) -> Result<User, diesel::result::Error> {
        use crate::db::schema::users as users_table;

        let new_user = (
            users_table::username.eq(username),
            users_table::created_at.eq(chrono::Utc::now().naive_utc()),
        );

        diesel::insert_into(users_table::table)
            .values(&new_user)
            .returning(User::as_returning())
            .get_result(conn)
    }
}

impl AuthUser for User {
    type Id = i32;
    fn id(&self) -> Self::Id {
        self.id
    }
    fn session_auth_hash(&self) -> &[u8] {
        self.username.as_bytes()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    pub next: Option<String>,
}

#[derive(Clone)]
pub struct Backend {
    pub db:
        std::sync::Arc<diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>,
}

impl Backend {
    pub fn new() -> Self {
        let db = crate::db::db_utils::establish_pool();
        Self {
            db: std::sync::Arc::new(db),
        }
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
    #[error(transparent)]
    TaskJoin(#[from] tokio::task::JoinError),
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let db = self.db.clone();
        let email = creds.email.clone();
        let password = creds.password;

        let user_and_account: Option<(User, Account)> = task::spawn_blocking(move || {
            let pool = db;
            let mut conn = pool.get().ok()?;

            // Join users with their email/password accounts
            users_table::table
                .inner_join(
                    crate::db::schema::accounts::table
                        .on(crate::db::schema::accounts::user_id.eq(users_table::id)),
                )
                .filter(crate::db::schema::accounts::email.eq(&email))
                .filter(crate::db::schema::accounts::type_.eq("email"))
                .filter(crate::db::schema::accounts::password.is_not_null())
                .select((User::as_select(), Account::as_select()))
                .first::<(User, Account)>(&mut conn)
                .ok()
        })
        .await?;

        if let Some((user, account)) = user_and_account {
            if let Some(stored_password) = account.password {
                let valid = verify_password(password, &stored_password).is_ok();
                Ok(if valid { Some(user) } else { None })
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let db = self.db.clone();
        let user_id = *user_id; // Copy the value so it can be moved into the closure
        let user = task::spawn_blocking(move || {
            let pool = db;
            let mut conn = pool.get().ok()?;
            users_table::table
                .filter(users_table::id.eq(user_id))
                .select(User::as_select())
                .first::<User>(&mut conn)
                .ok()
        })
        .await?;
        Ok(user)
    }
}

pub type AuthSession = axum_login::AuthSession<Backend>;
