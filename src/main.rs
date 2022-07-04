use std::net::SocketAddr;

#[cfg(not(debug_assertions))]
use std::path::PathBuf;

#[cfg(not(debug_assertions))]
use axum_server::tls_rustls::RustlsConfig;

use server::router;

#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if std::env::var("RUST_LOG").is_err() {
        #[cfg(not(debug_assertions))]
        let val = "server=debug,axum=info,event=info,now=info,timer=info";

        #[cfg(debug_assertions)]
        let val = "server=trace,axum=info,event=trace,now=trace,timer=trace";

        std::env::set_var("RUST_LOG", val);
    }

    //pretty_env_logger::init();

    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop";
    let router = router(url).await;

    #[cfg(debug_assertions)]
    let addr = SocketAddr::from(([0, 0, 0, 0], 4001));

    #[cfg(not(debug_assertions))]
    let addr = SocketAddr::from(([0, 0, 0, 0], 4001));

    tracing::info!("Listening on {}", addr);

    #[cfg(debug_assertions)]
    {
        axum::Server::bind(&addr)
            .serve(router.into_make_service())
            .await
            .expect("unable to bind address");
    }

    #[cfg(not(debug_assertions))]
    {
        let tls_config =
            RustlsConfig::from_pem_file(PathBuf::from("cert.pem"), PathBuf::from("privkey.pem"))
                .await
                .expect("unable to open certificate files");
        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await
            .expect("unable to bind https server");
    }
}
