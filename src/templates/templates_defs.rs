use askama::Template;

#[derive(Template)]
#[template(path = "html/main.html")]
pub struct MainTemplate {
  pub posts: Vec<crate::db::models::Post>,
}
