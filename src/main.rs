use askama::Template;
use axum::http::StatusCode;
use axum::{Extension, extract::State, response::Redirect};
use axum::{Router, extract::Form, response::Html, routing::get, routing::post};
use axum_login::tower_sessions::MemoryStore;
use axum_login::{AuthManagerLayer, AuthnBackend};
use diesel::prelude::*;
use password_auth::generate_hash;
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use z3_app::db::db_utils;
use z3_app::db::models::users::{AuthSession, Backend, Credentials};
use z3_app::db::models::{
    posts::{NewPost, Post},
    users::User,
};
use z3_app::templates::templates_defs::{MainTemplate, PostTemplate};

use axum_login::{
    AuthManagerLayerBuilder, login_required,
    tower_sessions::SessionManagerLayer,
};

/// Launches the Axum web server with HTML template rendering and static file serving.
///
/// Sets up application routes for the root path (`/`), a test page (`/test`), and static file serving at `/static`.
/// Binds to `127.0.0.1:3000` and serves requests asynchronously. Exits the process if the server encounters a runtime error.
///
/// # Examples
///
/// ```no_run
/// // Run the application by executing the binary.
/// // The server will be accessible at http://127.0.0.1:3000/
/// ```
#[tokio::main]
async fn main() {
    let backend = Backend::new();
    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store);
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer);
    let app = Router::new()
        .route("/", get(root))
        .route("/posts", get(post_get))
        .route("/posts", post(post_post))
        .route("/signup", get(signup_form).post(signup_post))
        .route("/login", get(login_form).post(login_post))
        .route("/signout", post(signout_post))
        .nest_service("/static", ServeDir::new("static"))
        .layer(auth_layer.build())
        .with_state(backend);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

/// Handles requests to the root path by rendering the main HTML template.
///
/// Returns the rendered `MainTemplate` as an HTML response.
///
/// # Examples
///
/// ```
/// // In an Axum application, this handler serves the "/" route:
/// let response = root().await;
/// assert!(response.0.contains("<html"));
/// ```
async fn root() -> Html<String> {
    let template: MainTemplate = MainTemplate {
        posts: Post::get_published().await,
    };
    Html(template.render().unwrap())
}

/// Handles POST requests to the `/posts` route by creating a new post and returning the post as HTML.
///
/// # Examples
///
/// ```
/// // In an Axum application, this handler can be used as follows:
/// let app = axum::Router::new().route("/posts", post(post_post));
/// ```
async fn post_post(Form(input): Form<NewPost>) -> Result<Html<String>, StatusCode> {
    println!("Received post input: {:?}", input);

    if input.title.is_empty() || input.body.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    match Post::create(
        &mut db_utils::establish_connection(),
        &input.title,
        &input.body,
        &input.author_id,
        input.created_at,
    ) {
        Some(post) => {
            let post_template = PostTemplate { post };
            let html = post_template.render().unwrap();
            Ok(Html(html))
        }
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Handles GET requests to the `/posts` route by rendering a list of posts.
///
/// Returns multiple rendered `PostTemplate` as one HTML response.
///
/// # Examples
/// /// ```
/// // In an Axum application, this handler can be used as follows:
/// let app = axum::Router::new().route("/posts", get(post_get));
/// ```
async fn post_get() -> Html<String> {
    let posts: Vec<Post> = Post::get_published().await;
    let mut html = String::new();

    for post in posts {
        let post_template: PostTemplate = PostTemplate { post };
        html.push_str(&post_template.render().unwrap());
    }

    Html(html)
}

/// Renders the signup form
async fn signup_form() -> Html<String> {
    Html(
        r#"<form method='post' action='/signup'>
        <input name='username' placeholder='Username'/><br/>
        <input name='email' placeholder='Email'/><br/>
        <input name='password' type='password' placeholder='Password'/><br/>
        <button type='submit'>Sign Up</button>
    </form>"#
            .to_string(),
    )
}

/// Handles signup POST, creates a new user
#[axum::debug_handler]
async fn signup_post(Form(input): Form<SignupForm>) -> Result<Redirect, StatusCode> {
    let mut conn = db_utils::establish_connection();
    let hashed = generate_hash(&input.password);
    let new_user = (
        z3_app::db::schema::users::username.eq(&input.username),
        z3_app::db::schema::users::password.eq(hashed),
        z3_app::db::schema::users::email.eq(input.email.clone()),
    );
    let res = diesel::insert_into(z3_app::db::schema::users::table)
        .values(&new_user)
        .execute(&mut conn);
    match res {
        Ok(_) => Ok(Redirect::to("/login")),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Renders the login form
async fn login_form() -> Html<String> {
    Html(
        r#"<form method='post' action='/login'>
        <input name='username' placeholder='Username'/><br/>
        <input name='password' type='password' placeholder='Password'/><br/>
        <button type='submit'>Login</button>
    </form>"#
            .to_string(),
    )
}

/// Handles login POST, authenticates user and starts session
#[axum::debug_handler]
async fn login_post(
    Extension(mut session): Extension<AuthSession>,
    State(backend): State<Backend>,
    Form(input): Form<Credentials>,
) -> Result<Redirect, StatusCode> {
    match backend.authenticate(input.clone()).await {
        Ok(Some(user)) => {
            session.login(&user).await.unwrap();
            Ok(Redirect::to("/"))
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Handles signout POST, ends the session
async fn signout_post(Extension(mut session): Extension<AuthSession>) -> Redirect {
    let _ = session.logout().await;
    Redirect::to("/")
}

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}
