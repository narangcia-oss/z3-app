use askama::Template;

#[derive(Template)]
#[template(path = "html/main.html")]
pub struct MainTemplate {
    pub posts: Vec<crate::db::models::posts::Post>,
}

#[derive(Template)]
#[template(path = "html/_components/post.html")]
pub struct PostTemplate {
    pub post: crate::db::models::posts::Post,
}

#[derive(Template)]
#[template(path = "html/_components/login_form.html")]
pub struct LoginFormTemplate {}

#[derive(Template)]
#[template(path = "html/_components/signup_form.html")]
pub struct SignupFormTemplate {}

#[derive(Template)]
#[template(path = "html/_components/welcome.html")]
pub struct WelcomeTemplate {}

#[derive(Template)]
#[template(path = "html/_components/user_header.html")]
pub struct UserHeaderTemplate {
    pub username: String,
}

#[derive(Template)]
#[template(path = "html/_components/signup_success.html")]
pub struct SignupSuccessTemplate {}

#[derive(Template)]
#[template(path = "html/_components/error_message.html")]
pub struct ErrorMessageTemplate {
    pub message: String,
}

#[derive(Template)]
#[template(path = "html/_components/redirect.html")]
pub struct RedirectTemplate {
    pub redirect_url: String,
}
