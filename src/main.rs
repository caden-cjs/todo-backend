use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, patch, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub text: String,
    pub completed: bool,
    pub important: bool,
    pub urgent: bool,
    pub quadrant: i32,
    pub due_date: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub text: String,
    #[serde(default)]
    pub important: bool,
    #[serde(default)]
    pub urgent: bool,
    pub due_date: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub text: Option<String>,
    pub completed: Option<bool>,
    pub important: Option<bool>,
    pub urgent: Option<bool>,
    pub due_date: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct TodoQuery {
    pub quadrant: Option<i32>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

pub struct Db { conn: Mutex<Connection> }

impl Db {
    pub fn new() -> Self {
        let conn = Connection::open("data/todos.db").unwrap_or_else(|_| {
            std::fs::create_dir_all("data").ok();
            Connection::open("data/todos.db").unwrap()
        });
        conn.execute("CREATE TABLE IF NOT EXISTS todos (id TEXT PRIMARY KEY, text TEXT NOT NULL, completed INTEGER DEFAULT 0, important INTEGER DEFAULT 0, urgent INTEGER DEFAULT 0, quadrant INTEGER DEFAULT 4, due_date TEXT, tags TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)", []).unwrap();
        Self { conn: Mutex::new(conn) }
    }

    fn calc_q(i: bool, u: bool) -> i32 { if i && u { 1 } else if i { 2 } else if u { 3 } else { 4 } }

    fn parse(row: &rusqlite::Row) -> Result<Todo, rusqlite::Error> {
        let tags: Vec<String> = serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_default();
        Ok(Todo {
            id: row.get(0)?, text: row.get(1)?,
            completed: row.get::<_, i32>(2)? != 0,
            important: row.get::<_, i32>(3)? != 0,
            urgent: row.get::<_, i32>(4)? != 0,
            quadrant: row.get(5)?, due_date: row.get(6)?, tags,
            created_at: row.get::<_, String>(8)?.parse().unwrap(),
            updated_at: row.get::<_, String>(9)?.parse().unwrap(),
        })
    }
}

async fn get_all(State(db): State<Db>, Query(q): Query<TodoQuery>) -> Json<ApiResponse<Vec<Todo>>> {
    let c = db.conn.lock().unwrap();
    let mut sql = "SELECT * FROM todos WHERE 1=1".to_string();
    let mut p: Vec<Box<dyn rusqlite::ToSql>> = vec![];
    if let Some(v) = q.quadrant { sql.push_str(" AND quadrant=?"); p.push(Box::new(v)); }
    if let Some(v) = q.completed { sql.push_str(" AND completed=?"); p.push(Box::new(if v {1}else{0})); }
    sql.push_str(" ORDER BY created_at DESC");
    let params: Vec<&dyn rusqlite::ToSql> = p.iter().map(|x| x.as_ref()).collect();
    let todos: Vec<Todo> = c.prepare(&sql).unwrap().query_map(params.as_slice(), Db::parse).unwrap().map(|r| r.unwrap()).collect();
    Json(ApiResponse { success: true, data: Some(todos), error: None, message: None })
}

async fn get_one(State(db): State<Db>, Path(id): Path<String>) -> Result<Json<ApiResponse<Todo>>, StatusCode> {
    let c = db.conn.lock().unwrap();
    match c.prepare("SELECT * FROM todos WHERE id=?").unwrap().query_row(params![&id], Db::parse) {
        Ok(t) => Ok(Json(ApiResponse { success: true, data: Some(t), error: None, message: None })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn create(State(db): State<Db>, Json(b): Json<CreateTodo>) -> Result<Json<ApiResponse<Todo>>, (StatusCode, Json<ApiResponse<()>>)> {
    if b.text.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(ApiResponse { success: false, data: None, error: Some("Text required"), message: None })));
    }
    let id = Uuid::new_v4().to_string();
    let q = Db::calc_q(b.important, b.urgent);
    let now = Utc::now();
    let c = db.conn.lock().unwrap();
    c.execute("INSERT INTO todos VALUES (?,?,0,?,?,?,?,?,?,?)", params![&id, b.text.trim(), if b.important{1}else{0}, if b.urgent{1}else{0}, q, b.due_date, serde_json::to_string(&b.tags).unwrap(), now.to_rfc3339(), now.to_rfc3339()]).unwrap();
    Ok(Json(ApiResponse { success: true, data: Some(Todo { id, text: b.text.trim().to_string(), completed: false, important: b.important, urgent: b.urgent, quadrant: q, due_date: b.due_date, tags: b.tags, created_at: now, updated_at: now }), error: None, message: None }))
}

async fn update(State(db): State<Db>, Path(id): Path<String>, Json(b): Json<UpdateTodo>) -> Result<Json<ApiResponse<Todo>>, (StatusCode, Json<ApiResponse<()>>)> {
    let c = db.conn.lock().unwrap();
    let ex = match c.prepare("SELECT * FROM todos WHERE id=?").unwrap().query_row(params![&id], Db::parse) {
        Ok(t) => t, Err(_) => return Err((StatusCode::NOT_FOUND, Json(ApiResponse { success: false, data: None, error: Some("Not found"), message: None }))),
    };
    let text = b.text.map(|t| t.trim().to_string()).unwrap_or(ex.text);
    let comp = b.completed.unwrap_or(ex.completed);
    let imp = b.important.unwrap_or(ex.important);
    let urg = b.urgent.unwrap_or(ex.urgent);
    let q = Db::calc_q(imp, urg);
    let due = b.due_date.or(ex.due_date);
    let tags = b.tags.unwrap_or(ex.tags);
    let now = Utc::now();
    c.execute("UPDATE todos SET text=?,completed=?,important=?,urgent=?,quadrant=?,due_date=?,tags=?,updated_at=? WHERE id=?", params![text, if comp{1}else{0}, if imp{1}else{0}, if urg{1}else{0}, q, due, serde_json::to_string(&tags).unwrap(), now.to_rfc3339(), &id]).unwrap();
    Ok(Json(ApiResponse { success: true, data: Some(Todo { id: ex.id, text, completed: comp, important: imp, urgent: urg, quadrant: q, due_date: due, tags, created_at: ex.created_at, updated_at: now }), error: None, message: None }))
}

async fn delete(State(db): State<Db>, Path(id): Path<String>) -> Json<ApiResponse<()>> {
    db.conn.lock().unwrap().execute("DELETE FROM todos WHERE id=?", params![&id]).unwrap();
    Json(ApiResponse { success: true, data: None, error: None, message: Some("Deleted") })
}

async fn toggle(State(db): State<Db>, Path(id): Path<String>) -> Result<Json<ApiResponse<Todo>>, (StatusCode, Json<ApiResponse<()>>)> {
    let c = db.conn.lock().unwrap();
    let ex = match c.prepare("SELECT * FROM todos WHERE id=?").unwrap().query_row(params![&id], Db::parse) {
        Ok(t) => t, Err(_) => return Err((StatusCode::NOT_FOUND, Json(ApiResponse { success: false, data: None, error: Some("Not found"), message: None }))),
    };
    let nc = !ex.completed;
    let now = Utc::now();
    c.execute("UPDATE todos SET completed=?,updated_at=? WHERE id=?", params![if nc{1}else{0}, now.to_rfc3339(), &id]).unwrap();
    Ok(Json(ApiResponse { success: true, data: Some(Todo { id: ex.id, text: ex.text, completed: nc, important: ex.important, urgent: ex.urgent, quadrant: ex.quadrant, due_date: ex.due_date, tags: ex.tags, created_at: ex.created_at, updated_at: now }), error: None, message: None }))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "timestamp": Utc::now().to_rfc3339() }))
}

#[tokio::main]
async fn main() {
    let db = Db::new();
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/todos", get(get_all).post(create))
        .route("/api/v1/todos/:id", get(get_one).put(update).delete(delete))
        .route("/api/v1/todos/:id/toggle", patch(toggle))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(db);
    let addr = "0.0.0.0:3001";
    println!("🚀 Todo API running on http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app).await.unwrap();
}