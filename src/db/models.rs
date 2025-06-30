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
}

#[derive(Insertable, Debug, Clone, Deserialize)]
#[diesel(table_name = crate::db::schema::posts)]
pub struct NewPost {
    pub title: String,
    pub body: String,
    #[serde(default = "default_published")]
    pub published: Option<bool>,
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

    pub fn create(conn: &mut diesel::PgConnection, title: &str, body: &str) -> Option<Post> {
        let new_post: NewPost = NewPost::new(title.to_string(), body.to_string(), Some(true));
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
    pub fn new(title: String, body: String, published: Option<bool>) -> Self {
        NewPost {
            title,
            body,
            published,
        }
    }
}

mod users {
    use super::*;
    use crate::db::schema::users as users_table;
    use async_trait::async_trait;
    use axum_login::{AuthUser, AuthnBackend, UserId};
    use password_auth::verify_password;
    use serde::{Deserialize, Serialize};
    use tokio::task;

    #[derive(Queryable, Selectable, Clone, Serialize, Deserialize)]
    #[diesel(table_name = users_table)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    pub struct User {
        pub id: i32,
        pub username: String,
        pub password: String,
        pub email: Option<String>,
    }

    impl std::fmt::Debug for User {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("User")
                .field("id", &self.id)
                .field("username", &self.username)
                .field("password", &"[redacted]")
                .finish()
        }
    }

    impl AuthUser for User {
        type Id = i32;
        fn id(&self) -> Self::Id {
            self.id
        }
        fn session_auth_hash(&self) -> &[u8] {
            self.password.as_bytes()
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct Credentials {
        pub username: String,
        pub password: String,
        pub next: Option<String>,
    }

    #[derive(Clone)]
    pub struct Backend {
        pub db: std::sync::Arc<tokio::sync::Mutex<diesel::PgConnection>>,
    }

    impl Backend {
        pub fn new(db: diesel::PgConnection) -> Self {
            Self {
                db: std::sync::Arc::new(tokio::sync::Mutex::new(db)),
            }
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
            let username = creds.username.clone();
            let user: Option<User> = task::spawn_blocking(move || {
                let mut conn = db.blocking_lock();
                users_table::table
                    .filter(users_table::username.eq(&username))
                    .select(User::as_select())
                    .first::<User>(&mut *conn)
                    .ok()
            })
            .await?;

            let password = creds.password;
            let valid = user
                .as_ref()
                .map(|u| verify_password(password.clone(), &u.password).is_ok())
                .unwrap_or(false);
            Ok(if valid { user } else { None })
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
