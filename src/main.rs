#![feature(async_closure)]

use axum::{Router, Server};
use clap::Parser;
use local_ip_address::local_ip;
use std::{
    net::AddrParseError,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use thiserror::Error;
use tokio::task::JoinError;

mod api;
mod frontend;
mod students;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), ApplicationError> {
    let args = AppArgs::parse();
    let app = AppState {
        students: Arc::new(Mutex::new(vec![])),
        should_refresh: Arc::new(AtomicBool::new(true)),
    };
    let router = Router::new()
        .nest("/api", api::routes())
        .merge(frontend::routes())
        .with_state(app.clone());

    println!("Web address: http://{}:12000", local_ip().unwrap());
    let mut server = tokio::spawn(
        Server::try_bind(&([0, 0, 0, 0], 12000).into())?
            .serve(router.into_make_service())
            .with_graceful_shutdown(shutdown()),
    );

    Ok(loop {
        let should_refresh =
            app.should_refresh
                .compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed);
        if let Ok(true) = should_refresh {
            do_refresh(&app).await;
        }
        let sleep = tokio::time::sleep(Duration::from_secs(60 * args.refresh_every));
        tokio::pin!(sleep);
        tokio::select! {
            result = &mut server => break result??,
            _ = &mut sleep => drop(sleep),
        };
    })
}

async fn do_refresh(app: &AppState) {
    let loaded_students = students::load_students().await;
    if let Err(err) = loaded_students {
        println!("{:?}", err);
    } else {
        let mut students_lock = app.students.lock().unwrap();
        *students_lock = loaded_students.unwrap();
    }
}

async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    use tokio::signal::unix::{signal, SignalKind};
    #[cfg(unix)]
    let terminate = async {
        signal(SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, shutting down gracefully");
}

#[derive(Error, Debug)]
enum ApplicationError {
    #[error("Invalid socket address")]
    InvalidSocketAddr(#[from] AddrParseError),
    #[error("Failed to bind to address")]
    BindError(#[from] hyper::Error),
    #[error("Server thread has panicked, shutting down...")]
    PanickError(#[from] JoinError),
}

#[derive(Clone)]
pub struct AppState {
    students: Arc<Mutex<Vec<api::Student>>>,
    should_refresh: Arc<AtomicBool>,
}

#[derive(clap::Parser)]
#[command(author = "Evan Calderon", version = "1.0.0", long_about = None)]
struct AppArgs {
    // The number of minutes to wait between refreshes
    #[arg(short = 'e', long, default_value_t = 5)]
    refresh_every: u64,
}
