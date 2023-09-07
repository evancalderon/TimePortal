use axum::{Router, Server};
use local_ip_address::local_ip;
use std::{
    net::{AddrParseError, SocketAddr},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};
use thiserror::Error;

mod api;
mod frontend;
mod students;

#[tokio::main]
async fn main() -> Result<(), ApplicationError> {
    let app = AppState {
        students: Arc::new(Mutex::new(vec![])),
        refresh_now: Arc::new(Mutex::new(false)),
    };
    let router = Router::new()
        .nest("/api", api::routes())
        .merge(frontend::routes())
        .with_state(app.clone());

    let local_ip = local_ip().unwrap();
    println!("Web address: http://{local_ip}:12000");
    let addr = SocketAddr::from_str("0.0.0.0:12000")?;
    let server = async move {
        Server::try_bind(&addr)?
            .serve(router.into_make_service())
            .with_graceful_shutdown(shutdown())
            .await
    };

    tokio::spawn(async move {
        loop {
            let loaded = students::load_students().await;
            if let Err(err) = loaded {
                println!("{:?}", err);
            } else {
                let students = loaded.unwrap();
                let mut lock = app.students.lock().unwrap();
                *lock = students;
            }
            for i in 0..30 * 5 {
                if i % 2 == 1 {
                    continue;
                }
                {
                    let mut refresh_now = app.refresh_now.lock().unwrap();
                    if *refresh_now {
                        *refresh_now = false;
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    });

    tokio::join!(server).0.ok();

    Ok(())
}

#[derive(Error, Debug)]
enum ApplicationError {
    #[error("Invalid socket address")]
    InvalidSocketAddr(#[from] AddrParseError),
    #[error("Failed to bind to address")]
    BindError(#[from] hyper::Error),
}

#[derive(Clone)]
pub struct AppState {
    students: Arc<Mutex<Vec<api::Student>>>,
    refresh_now: Arc<Mutex<bool>>,
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
