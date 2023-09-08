use axum::{extract::State, routing, Json, Router};
use chrono::{DateTime, Utc};

pub fn routes() -> Router<crate::AppState> {
    Router::new().route("/students", routing::get(get_students))
}

async fn get_students(State(state): State<crate::AppState>) -> Json<Vec<Student>> {
    *state.should_refresh.lock().unwrap() = true;
    Json(state.students.lock().unwrap().clone())
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Student {
    pub name: String,
    pub belt: String,
    pub time_start_dt: DateTime<Utc>,
    pub time_start: String,
    pub time_end: String,
}
