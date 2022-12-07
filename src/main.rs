use std::{net::SocketAddr, time::Duration, panic::AssertUnwindSafe};
use futures::future::{Future, FutureExt};

#[cfg(not(debug_assertions))]
use std::path::PathBuf;

#[cfg(not(debug_assertions))]
use axum_server::tls_rustls::RustlsConfig;

use server::{router, logger::*, Compilation, Result, Pool};
use tracing_subscriber::{prelude::*, EnvFilter};

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

    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| {
                "server=trace,tracing=trace,hyper=info,axum=trace,event=trace,now=trace,timer=trace"
                    .into()
            },
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop";
    let pool = Pool::connect(url)
        .await
        .expect("Unable to connect to database");
    let pool: &'static Pool = Box::leak(pool.into());

    tokio::task::spawn(update_compilations(pool));

    let router = router(pool).await;

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

async fn update_compilations(pool: &'static Pool) {
    loop {
        wrap_panic("update compilations".to_owned(), update_compilations_tick(pool)).await;
        tokio::time::sleep(Duration::from_secs(7200)).await;
    }
}

async fn update_compilations_tick(pool: &'static Pool) -> Result<()> {
    let mut txn = pool.begin().await?;
    let all_compilations = Compilation::all_active(&mut txn).await?;
    txn.commit().await?;

    for compilation in all_compilations {
        wrap_panic(format!("update compilation ({:?})", compilation.id()), update_compilations_each(pool, &compilation)).await;
    }
    Ok(())
}

async fn update_compilations_each(pool: &'static Pool, compilation: &Compilation) -> Result<()> {
    let mut txn = pool.begin().await?;
    compilation.compile_if_outdated(&mut txn).await?;
    txn.commit().await?;
    Ok(())
}

async fn wrap_panic<F: Future<Output = Result<()>>>(label: String, future: F) {
    match AssertUnwindSafe(future).catch_unwind().await {
        Ok(Ok(())) => {},
        Ok(Err(err)) => error!("{label}: {err}"),
        Err(any) => {
            // Note: Technically panics can be of any form, but most should be &str or String
            match any.downcast::<String>() {
                Ok(msg) => error!("Panic at {label}: {msg}"),
                Err(any) => match any.downcast::<&str>() {
                    Ok(msg) => error!("Panic at {label}: {msg}"),
                    Err(any) => {
                        let type_id = any.type_id();
                        error!("{label}: Unable to downcast panic message {type_id:?}",);
                    }
                },
            }
        }
    }
}