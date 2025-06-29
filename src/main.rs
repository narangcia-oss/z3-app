use askama::Template;
use axum::http::StatusCode;
use axum::{Router, extract::Form, response::Html, routing::get, routing::post};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use z3_app::db::db_utils;
use z3_app::db::models::{NewPost, Post};
use z3_app::templates::{main::MainTemplate, test::TestTemplate};

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
    let app = Router::new()
        .route("/", get(root))
        .route("/test", post(test_post))
        .route("/posts", post(post_post))
        .nest_service("/static", ServeDir::new("static"));

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

#[derive(Deserialize, Debug, Clone)]
struct TestInput {
    test_input: String,
}

/// Handles POST requests to the `/test` route by rendering the `TestTemplate` with a fixed message.
///
/// Returns an HTML response containing the rendered template.
/// # Examples
/// ```
/// // In an Axum application, this handler can be used as follows:
/// let app = axum::Router::new().route("/test", post(test_post));
/// ```
async fn test_post(Form(input): Form<TestInput>) -> Html<String> {
    let template = TestTemplate {
        test: &input.test_input,
    };
    Html(template.render().expect("Failed to render test template"))
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
    match Post::create(
        &mut db_utils::establish_connection(),
        &input.title,
        &input.body,
    ) {
        Some(post) => {
            let html = format!(
                r#"<li class="border-b pb-4">
                    <h2 class="text-xl font-semibold text-gray-700">{}</h2>
                    <p class="text-gray-500">{}</p>
                </li>"#,
                post.title, post.body
            );
            Ok(Html(html))
        }
        None => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
