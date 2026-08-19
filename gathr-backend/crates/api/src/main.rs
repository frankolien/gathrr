use actix_web::middleware::from_fn;
use actix_web::{web, App, HttpServer};
use anyhow::Context;
use gathr_api::{ratelimit, routes, AppState};
use gathr_common::{telemetry, Config};

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    telemetry::init();

    let config = Config::from_env().context("configuration is incomplete")?;
    let bind_addr = config.bind_addr.clone();

    let db = gathr_infra_db::connect(&config.database_url, 10)
        .await
        .context("could not connect to postgres")?;
    gathr_infra_db::run_migrations(&db)
        .await
        .context("could not apply migrations")?;

    let state = web::Data::new(AppState::new(db, config));

    tracing::info!(%bind_addr, "gathr api listening");

    HttpServer::new(move || {
        App::new()
            .wrap(from_fn(ratelimit::enforce))
            .app_data(state.clone())
            .app_data(web::JsonConfig::default().limit(64 * 1024))
            .configure(routes::configure)
    })
    .bind(&bind_addr)
    .with_context(|| format!("could not bind {bind_addr}"))?
    .run()
    .await
    .context("server stopped unexpectedly")
}
