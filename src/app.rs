use {
    crate::{services::posts, state::State},
    actix_files::Files,
    actix_web::{App, HttpServer, web::Data},
    std::io::Error,
};

pub async fn run(state: State, addr: String, port: u16) -> Result<(), Error> {
    let data = Data::new(state);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(Files::new("/static", "./static"))
            .configure(posts::configure)
    })
    .bind((addr, port))?
    .run()
    .await
}
