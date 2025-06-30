/// After changing schema.rs, you need to run the migration command to apply changes.
///
/// You can use the `diesel` CLI to manage migrations in your Rust project.
///
/// To generate a new migration using the `diesel` CLI, run the following commands:
/// ```bash
/// diesel migration generate --diff-schema <migration_name>
/// diesel migration run
/// ```
/// Then i recommend properly making your models here
pub mod posts {
    use diesel::prelude::*;
    use serde::Deserialize;

    #[derive(Queryable, Selectable, Debug, Clone, Deserialize)]
    #[diesel(table_name = crate::db::schema::posts)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Post {
        pub id: i32,
        pub title: String,
        pub body: String,
        pub published: bool,
        pub author_id: Option<i32>,
        pub created_at: chrono::NaiveDateTime,
    }

    #[derive(Insertable, Debug, Clone, Deserialize)]
    #[diesel(table_name = crate::db::schema::posts)]
    pub struct NewPost {
        pub title: String,
        pub body: String,
        #[serde(default = "default_published")]
        pub published: Option<bool>,
        pub author_id: Option<i32>,
        pub created_at: chrono::NaiveDateTime,
    }

    fn default_published() -> Option<bool> {
        Some(true)
    }

    impl Post {
        pub async fn get_published() -> Vec<Post> {
            use crate::db::schema::posts::dsl::*;
            let connection: &mut diesel::PgConnection =
                &mut crate::db::db_utils::establish_connection();
            let results: Vec<Post> = posts
                .filter(diesel::ExpressionMethods::eq(published, true))
                .limit(5)
                .select(Post::as_select())
                .load(connection)
                .expect("Error loading posts");
            println!("Displaying {} posts", results.len());
            results
        }

        pub fn create(
            conn: &mut diesel::PgConnection,
            title: &str,
            body: &str,
            author_id: &Option<i32>,
            created_at_value: chrono::NaiveDateTime,
        ) -> Option<Post> {
            let new_post: NewPost = NewPost::new(
                title.to_string(),
                body.to_string(),
                Some(true),
                *author_id,
                created_at_value,
            );
            println!("Creating post: {:?}", new_post);
            let result = diesel::insert_into(crate::db::schema::posts::table)
                .values(&new_post)
                .returning(Post::as_returning())
                .get_result(conn);
            match result {
                Ok(post) => {
                    println!("Inserted post: {:?}", post);
                    Some(post)
                }
                Err(e) => {
                    println!("Diesel insert error: {}", e);
                    None
                }
            }
        }
    }

    impl NewPost {
        pub fn new(
            title: String,
            body: String,
            published: Option<bool>,
            author_id: Option<i32>,
            created_at_value: chrono::NaiveDateTime,
        ) -> Self {
            NewPost {
                title,
                body,
                published,
                author_id,
                created_at: created_at_value,
            }
        }
    }
}

pub mod accounts {
    use crate::db::schema::accounts;
    use diesel::prelude::*;
    use serde::{Deserialize, Serialize};

    /// Account model for handling authentication.
    ///
    /// The `type_` field determines the authentication method:
    /// - "email" for email/password authentication
    /// - "oauth" for OAuth providers (TODO: implement OAuth support)
    ///
    /// For email authentication: email and password fields are used
    /// For OAuth authentication: provider, provider_account_id, and token fields are used
    #[derive(Queryable, Selectable, Clone, Serialize, Deserialize, Debug)]
    #[diesel(table_name = accounts)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct Account {
        pub id: i32,
        pub user_id: i32,
        /// Authentication type: "email" for email/password, "oauth" for OAuth (TODO)
        pub type_: String,

        // Email/password authentication fields
        pub email: Option<String>,
        pub password: Option<String>,

        // OAuth authentication fields (TODO: implement OAuth support)
        pub provider: Option<String>,
        pub provider_account_id: Option<String>,
        pub refresh_token: Option<String>,
        pub access_token: Option<String>,
        pub expires_at: Option<i32>,
        pub token_type: Option<String>,
        pub scope: Option<String>,
        pub id_token: Option<String>,
        pub session_state: Option<String>,
        pub refresh_token_expires_in: Option<i32>,
    }

    #[derive(Insertable, Debug, Clone)]
    #[diesel(table_name = accounts)]
    pub struct NewAccount {
        pub user_id: i32,
        pub type_: String,
        pub email: Option<String>,
        pub password: Option<String>,
        pub provider: Option<String>,
        pub provider_account_id: Option<String>,
    }

    impl Account {
        /// Create a new email/password account
        pub fn create_email_account(
            conn: &mut diesel::PgConnection,
            user_id: i32,
            email: String,
            password_hash: String,
        ) -> Result<Account, diesel::result::Error> {
            let new_account = NewAccount {
                user_id,
                type_: "email".to_string(),
                email: Some(email),
                password: Some(password_hash),
                provider: None,
                provider_account_id: None,
            };

            diesel::insert_into(accounts::table)
                .values(&new_account)
                .returning(Account::as_returning())
                .get_result(conn)
        }

        /// Find account by email for authentication
        pub fn find_by_email(
            conn: &mut diesel::PgConnection,
            email: &str,
        ) -> Result<Account, diesel::result::Error> {
            accounts::table
                .filter(accounts::email.eq(email))
                .filter(accounts::type_.eq("email"))
                .select(Account::as_select())
                .first(conn)
        }
    }
}

pub mod users {
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
        pub db: std::sync::Arc<tokio::sync::Mutex<diesel::PgConnection>>,
    }

    impl Backend {
        pub fn new() -> Self {
            let db = crate::db::db_utils::establish_connection();
            Self {
                db: std::sync::Arc::new(tokio::sync::Mutex::new(db)),
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
                let mut conn = db.blocking_lock();

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
                    .first::<(User, Account)>(&mut *conn)
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

        async fn get_user(
            &self,
            user_id: &UserId<Self>,
        ) -> Result<Option<Self::User>, Self::Error> {
            let db = self.db.clone();
            let id = *user_id;
            let user = task::spawn_blocking(move || {
                let mut conn = db.blocking_lock();
                users_table::table
                    .filter(users_table::id.eq(id))
                    .select(User::as_select())
                    .first::<User>(&mut *conn)
                    .ok()
            })
            .await?;
            Ok(user)
        }
    }

    pub type AuthSession = axum_login::AuthSession<Backend>;
}
