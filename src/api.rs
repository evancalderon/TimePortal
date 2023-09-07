use axum::{extract::State, routing, Json, Router};
use chrono::{DateTime, Utc};

pub fn routes() -> Router<crate::AppState> {
    Router::new()
        .route("/students", routing::get(get_students))
        .route("/forcerefresh", routing::post(post_force_refresh))
}

async fn get_students(State(state): State<crate::AppState>) -> Json<Vec<Student>> {
    Json(state.students.lock().unwrap().clone())
}

async fn post_force_refresh(State(state): State<crate::AppState>) {
    *state.refresh_now.lock().unwrap() = true;
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Student {
    pub name: String,
    pub belt: String,
    pub time_start_dt: DateTime<Utc>,
    pub time_start: String,
    pub time_end: String,
}
