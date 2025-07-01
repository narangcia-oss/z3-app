use askama::Template;
use axum::{
    Extension, Router,
    extract::{Form, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
};
use axum_login::{
    AuthManagerLayerBuilder, AuthnBackend,
    tower_sessions::{MemoryStore, SessionManagerLayer},
};
use password_auth::generate_hash;
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use z3_app::{
    db::{
        db_utils,
        models::{
            accounts::Account,
            posts::{NewPost, Post},
            users::{AuthSession, Backend, Credentials, User},
        },
    },
    templates::{
        ErrorMessageTemplate, LoginFormTemplate, MainTemplate, PostTemplate, RedirectTemplate,
        SignupFormTemplate, SignupSuccessTemplate, UserHeaderTemplate, WelcomeTemplate,
    },
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
        .layer(CompressionLayer::new())
        .layer(auth_layer.build())
        .with_state(backend);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{addr}");

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {e}");
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
async fn root(Extension(session): Extension<AuthSession>) -> Html<String> {
    if let Some(user) = &session.user {
        // User is authenticated - show the main app
        let template: MainTemplate = MainTemplate {
            posts: Post::get_published().await,
        };
        let user_header = UserHeaderTemplate {
            username: user.username.clone(),
        };
        let mut template_content = user_header.render().unwrap();
        template_content.push_str(&template.render().unwrap());
        Html(template_content)
    } else {
        // User is not authenticated - show authentication options
        let welcome_template = WelcomeTemplate {};
        Html(welcome_template.render().unwrap())
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
async fn post_get(Extension(session): Extension<AuthSession>) -> Result<Html<String>, StatusCode> {
    // Check if user is authenticated
    if session.user.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let posts: Vec<Post> = Post::get_published().await;
    let mut html = String::new();

    for post in posts {
        let post_template: PostTemplate = PostTemplate { post };
        html.push_str(&post_template.render().unwrap());
    }

    Ok(Html(html))
}

#[derive(Debug, Deserialize)]
pub struct PostForm {
    pub title: String,
    pub body: String,
}

/// Handles POST requests to the `/posts` route by creating a new post and returning the post as HTML.
///
/// # Examples
///
/// ```
/// // In an Axum application, this handler can be used as follows:
/// let app = axum::Router::new().route("/posts", post(post_post));
/// ```
async fn post_post(
    Extension(session): Extension<AuthSession>,
    Form(input): Form<PostForm>,
) -> Result<Html<String>, StatusCode> {
    println!("Received post input: {input:?}");

    if input.title.is_empty() || input.body.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get the current user
    let user = session.user;
    if user.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user = user.unwrap();

    let new_post: NewPost = NewPost {
        title: input.title,
        body: input.body,
        published: Some(true),
        author_id: Some(user.id),
        created_at: chrono::Utc::now().naive_utc(),
    };

    let pool = db_utils::establish_pool();
    let mut conn = pool.get().unwrap();
    match Post::create(
        &mut conn,
        &new_post.title,
        &new_post.body,
        &new_post.author_id,
        new_post.created_at,
    ) {
        Some(post) => {
            let post_template = PostTemplate { post };
            let html = post_template.render().unwrap();
            Ok(Html(html))
        }
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Renders the signup form
async fn signup_form() -> Html<String> {
    let template = SignupFormTemplate {};
    Html(template.render().unwrap())
}

#[derive(Debug, Deserialize)]
pub struct SignupForm {
    pub username: String,
    pub password: String,
    pub email: String,
}

/// Handles signup POST, creates a new user
#[axum::debug_handler]
async fn signup_post(Form(input): Form<SignupForm>) -> Result<Html<String>, StatusCode> {
    let pool = db_utils::establish_pool();
    let mut conn = pool.get().unwrap();
    let hashed: String = generate_hash(&input.password);

    // Create user first
    let user_result = User::create(&mut conn, input.username);

    match user_result {
        Ok(user) => {
            // Create associated email account
            let account_result =
                Account::create_email_account(&mut conn, user.id, input.email, hashed);

            match account_result {
                Ok(_) => {
                    let success_template = SignupSuccessTemplate {};
                    Ok(Html(success_template.render().unwrap()))
                }
                Err(e) => {
                    println!("Failed to create account: {e}");
                    let error_template = ErrorMessageTemplate {
                        message: "Failed to create account. Email might already be in use."
                            .to_string(),
                    };
                    Ok(Html(error_template.render().unwrap()))
                }
            }
        }
        Err(e) => {
            println!("Failed to create user: {e}");
            let error_template = ErrorMessageTemplate {
                message: "Failed to create user. Username might already be taken.".to_string(),
            };
            Ok(Html(error_template.render().unwrap()))
        }
    }
}

/// Renders the login form
async fn login_form() -> Html<String> {
    let template = LoginFormTemplate {};
    Html(template.render().unwrap())
}

/// Handles login POST, authenticates user and starts session
#[axum::debug_handler]
async fn login_post(
    Extension(mut session): Extension<AuthSession>,
    State(backend): State<Backend>,
    Form(input): Form<Credentials>,
) -> Result<Html<String>, StatusCode> {
    match backend.authenticate(input.clone()).await {
        Ok(Some(user)) => {
            session.login(&user).await.unwrap();
            let redirect_template = RedirectTemplate {
                redirect_url: "/".to_string(),
            };
            Ok(Html(redirect_template.render().unwrap()))
        }
        _ => {
            let error_template = ErrorMessageTemplate {
                message: "Invalid email or password. Please try again.".to_string(),
            };
            Ok(Html(error_template.render().unwrap()))
        }
    }
}

/// Handles signout POST, ends the session
async fn signout_post(Extension(mut session): Extension<AuthSession>) -> Redirect {
    let _ = session.logout().await;
    Redirect::to("/")
}
