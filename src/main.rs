#![feature(async_closure)]

use axum::{Router, Server};
use local_ip_address::local_ip;
use std::{
    net::{AddrParseError, SocketAddr},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;
use tokio::task::JoinError;

mod api;
mod frontend;
mod students;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    let app = AppState {
        students: Arc::new(Mutex::new(vec![])),
        should_refresh: Arc::new(Mutex::new(false)),
    };
    let router = Router::new()
        .nest("/api", api::routes())
        .merge(frontend::routes())
        .with_state(app.clone());

    let local_ip = local_ip().unwrap();
    println!("Web address: http://{local_ip}:12000");
    let addr = SocketAddr::from_str("0.0.0.0:12000")?;

    let server = tokio::spawn(async move {
        Server::try_bind(&addr)?
            .serve(router.into_make_service())
            .with_graceful_shutdown(shutdown())
            .await
    });

    'outer: loop {
        let loaded_students = students::load_students().await;
        if let Err(err) = loaded_students {
            println!("{:?}", err);
        } else {
            let mut students_lock = app.students.lock().unwrap();
            *students_lock = loaded_students.unwrap();
        }
        {
            let mut should_refresh = app.should_refresh.lock().unwrap();
            if !*should_refresh {
                continue;
            }
            *should_refresh = false;
        }
        for _ in 0..60 * 5 {
            if server.is_finished() {
                break 'outer;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    server.await??;

    Ok(())
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
    should_refresh: Arc<Mutex<bool>>,
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
