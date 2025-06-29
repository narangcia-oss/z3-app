use askama::Template;
use axum::{Router, extract::Form, response::Html, routing::get};
use z3_app::templates::{main::MainTemplate, test::TestTemplate};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use diesel::prelude::*;

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
        .route("/test", get(test))
        .route("/test", axum::routing::post(test_post))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);

    let listener: tokio::net::TcpListener = tokio::net::TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {}", e);
        std::process::exit(1);
    }
}

/// Handles GET requests to the `/test` route by rendering the `TestTemplate` with a fixed message.
///
/// Returns an HTML response containing the rendered template.
///
/// # Examples
///
/// ```
// In an Axum application, this handler can be used as follows:
/// let app = Router::new().route("/test", axum::routing::get(test));
/// ```
async fn test() -> Html<String> {
    let template: TestTemplate<'_> = TestTemplate {
        test: "Hello, world!",
    };
    Html(template.render().expect("Failed to render test template"))
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
    let template: MainTemplate = MainTemplate {};
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

async fn get_posts() -> Vec<z3_app::db::models::Post> {
    use z3_app::db::schema::posts::dsl::*;

    let connection: &mut SqliteConnection = &mut z3_app::db::db_utils::establish_connection();
    let results: Vec<z3_app::db::models::Post> = posts
        .filter(diesel::ExpressionMethods::eq(published, true))
        .limit(5)
        .select(z3_app::db::models::Post::as_select())
        .load(connection)
        .expect("Error loading posts");

    println!("Displaying {} posts", results.len());

    results
}
