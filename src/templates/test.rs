use askama::Template;

#[derive(Template)]
#[template(path = "test.html")]
pub struct TestTemplate<'a> {
    pub test: &'a str,
}
