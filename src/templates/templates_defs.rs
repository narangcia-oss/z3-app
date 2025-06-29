use askama::Template;

#[derive(Template)]
#[template(path = "html/main.html")]
pub struct MainTemplate {
  pub posts: Vec<crate::db::models::Post>,
}

#[derive(Template)]
#[template(path = "html/_components/post.html")]
pub struct PostTemplate {
  pub post: crate::db::models::Post,
}
