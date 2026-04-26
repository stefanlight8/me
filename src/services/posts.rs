use {
    crate::{post, state::State, templates::post::PostTemplate},
    actix_web::{HttpResponse, Responder, web},
    askama::Template,
    comrak::{Options, markdown_to_html},
};

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/posts").route("/{slug}", web::get().to(get_post)));

    tracing::debug!("configured posts service");
}

pub fn md_to_html(md: &str) -> String {
    let options = Options::default();

    markdown_to_html(md, &options)
}

pub async fn get_post(state: web::Data<State>, slug: web::Path<String>) -> impl Responder {
    let slug = slug.into_inner();

    match post::get_post(state.posts_path.clone(), &slug).await {
        Ok(post) => {
            let template = PostTemplate {
                title: post.header.title,
                description: post.header.description,
                content: md_to_html(&post.content),
            };

            match template.render() {
                Ok(html) => HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(html),
                Err(err) => {
                    tracing::error!("template render error: {:?}", err);

                    HttpResponse::InternalServerError().body("Failed to render")
                }
            }
        }
        Err(post::PostParsingError::NotFound) => HttpResponse::NotFound().finish(),
        Err(err) => {
            tracing::error!("failed to parse post {}: {:?}", slug, err);

            HttpResponse::InternalServerError().body("Failed to parse post")
        }
    }
}
