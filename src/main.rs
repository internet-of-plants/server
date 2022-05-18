use std::net::SocketAddr;

use server::router;

#[tokio::main]
async fn main() {
    //#[cfg(not(debug_assertions))]
    //let server = server.tls().cert_path("cert.pem").key_path("privkey.pem");

    //#[cfg(debug_assertions)]
    //if std::env::var("RUST_BACKTRACE").is_err() {
    //    std::env::set_var("RUST_BACKTRACE", "1");
    //}

    //if std::env::var("RUST_LOG").is_err() {
    //    #[cfg(not(debug_assertions))]
    //    let val = "server=debug,axum=info,event=info,now=info,timer=info";

    //    #[cfg(debug_assertions)]
    //    let val =
    //        "server=trace,axum=info,event=trace,now=trace,timer=trace";

    //    std::env::set_var("RUST_LOG", val);
    //}

    //pretty_env_logger::init();

    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop";
    let router = router(url).await;

    let addr = SocketAddr::from(([0, 0, 0, 0], 4001));
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
