use askama::Template;

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate {
    pub title: String,
    pub description: String,
    pub content: String,
}
