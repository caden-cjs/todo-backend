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
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

// ============ Models ============

#[derive(Debug, Serialize, Deserialize, Clone)]
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

// ============ Database ============

#[derive(Clone)]
pub struct Db {
    conn: Arc<std::sync::Mutex<Connection>>,
}

impl Db {
    pub fn new() -> Self {
        let conn = Connection::open("data/todos.db").unwrap_or_else(|_| {
            std::fs::create_dir_all("data").ok();
            Connection::open("data/todos.db").unwrap()
        });

        conn.execute(
            "CREATE TABLE IF NOT EXISTS todos (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                completed INTEGER DEFAULT 0,
                important INTEGER DEFAULT 0,
                urgent INTEGER DEFAULT 0,
                quadrant INTEGER DEFAULT 4,
                due_date TEXT,
                tags TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )
        .unwrap();

        Self { conn: Arc::new(std::sync::Mutex::new(conn)) }
    }

    fn calculate_quadrant(important: bool, urgent: bool) -> i32 {
        if important && urgent { 1 }
        else if important && !urgent { 2 }
        else if !important && urgent { 3 }
        else { 4 }
    }

    fn parse_todo(row: &rusqlite::Row) -> Result<Todo, rusqlite::Error> {
        let tags_str: String = row.get(7)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        
        Ok(Todo {
            id: row.get(0)?,
            text: row.get(1)?,
            completed: row.get::<_, i32>(2)? != 0,
            important: row.get::<_, i32>(3)? != 0,
            urgent: row.get::<_, i32>(4)? != 0,
            quadrant: row.get(5)?,
            due_date: row.get(6)?,
            tags,
            created_at: row.get::<_, String>(8)?.parse().unwrap(),
            updated_at: row.get::<_, String>(9)?.parse().unwrap(),
        })
    }
}

// ============ Handlers ============

async fn get_all_todos(
    State(db): State<Db>,
    Query(query): Query<TodoQuery>,
) -> Json<ApiResponse<Vec<Todo>>> {
    let conn = db.conn.lock().unwrap();
    
    let mut sql = "SELECT * FROM todos WHERE 1=1".to_string();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    
    if let Some(q) = query.quadrant {
        sql.push_str(" AND quadrant = ?");
        params_vec.push(Box::new(q));
    }
    
    if let Some(c) = query.completed {
        sql.push_str(" AND completed = ?");
        params_vec.push(Box::new(if c { 1 } else { 0 }));
    }
    
    sql.push_str(" ORDER BY created_at DESC");
    
    let params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    
    let todos: Vec<Todo> = conn
        .prepare(&sql)
        .unwrap()
        .query_map(params.as_slice(), Db::parse_todo)
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    
    Json(ApiResponse { success: true, data: Some(todos), error: None, message: None })
}

async fn get_todo_by_id(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Todo>>, StatusCode> {
    let conn = db.conn.lock().unwrap();
    
    let result = conn
        .prepare("SELECT * FROM todos WHERE id = ?")
        .unwrap()
        .query_row(params![&id], Db::parse_todo);
    
    match result {
        Ok(todo) => Ok(Json(ApiResponse { success: true, data: Some(todo), error: None, message: None })),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_todo(
    State(db): State<Db>,
    Json(body): Json<CreateTodo>,
) -> Result<Json<ApiResponse<Todo>>, (StatusCode, Json<ApiResponse<()>>)> {
    if body.text.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse { success: false, data: None, error: Some("Text is required".to_string()), message: None }),
        ));
    }
    
    let id = Uuid::new_v4().to_string();
    let quadrant = Db::calculate_quadrant(body.important, body.urgent);
    let now = Utc::now();
    let tags_json = serde_json::to_string(&body.tags).unwrap();
    
    let conn = db.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO todos (id, text, completed, important, urgent, quadrant, due_date, tags, created_at, updated_at)
         VALUES (?, ?, 0, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &id,
            body.text.trim(),
            if body.important { 1 } else { 0 },
            if body.urgent { 1 } else { 0 },
            quadrant,
            body.due_date,
            tags_json,
            now.to_rfc3339(),
            now.to_rfc3339()
        ],
    )
    .unwrap();
    
    let todo = Todo {
        id,
        text: body.text.trim().to_string(),
        completed: false,
        important: body.important,
        urgent: body.urgent,
        quadrant,
        due_date: body.due_date,
        tags: body.tags,
        created_at: now,
        updated_at: now,
    };
    
    Ok(Json(ApiResponse { success: true, data: Some(todo), error: None, message: None }))
}

async fn update_todo(
    State(db): State<Db>,
    Path(id): Path<String>,
    Json(body): Json<UpdateTodo>,
) -> Result<Json<ApiResponse<Todo>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = db.conn.lock().unwrap();
    
    let existing = conn
        .prepare("SELECT * FROM todos WHERE id = ?")
        .unwrap()
        .query_row(params![&id], Db::parse_todo);
    
    let existing = match existing {
        Ok(t) => t,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse { success: false, data: None, error: Some("Todo not found".to_string()), message: None }),
            ));
        }
    };
    
    let text = body.text.map(|t| t.trim().to_string()).unwrap_or(existing.text);
    let completed = body.completed.unwrap_or(existing.completed);
    let important = body.important.unwrap_or(existing.important);
    let urgent = body.urgent.unwrap_or(existing.urgent);
    let quadrant = Db::calculate_quadrant(important, urgent);
    let due_date = body.due_date.or(existing.due_date);
    let tags = body.tags.unwrap_or(existing.tags);
    let tags_json = serde_json::to_string(&tags).unwrap();
    let now = Utc::now();
    
    conn.execute(
        "UPDATE todos SET text=?, completed=?, important=?, urgent=?, quadrant=?, due_date=?, tags=?, updated_at=? WHERE id=?",
        params![
            text,
            if completed { 1 } else { 0 },
            if important { 1 } else { 0 },
            if urgent { 1 } else { 0 },
            quadrant,
            due_date,
            tags_json,
            now.to_rfc3339(),
            &id
        ],
    )
    .unwrap();
    
    let updated = Todo {
        id: existing.id,
        text,
        completed,
        important,
        urgent,
        quadrant,
        due_date,
        tags,
        created_at: existing.created_at,
        updated_at: now,
    };
    
    Ok(Json(ApiResponse { success: true, data: Some(updated), error: None, message: None }))
}

async fn delete_todo(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> Json<ApiResponse<()>> {
    let conn = db.conn.lock().unwrap();
    conn.execute("DELETE FROM todos WHERE id = ?", params![&id]).unwrap();
    Json(ApiResponse { success: true, data: None, error: None, message: Some("Todo deleted".to_string()) })
}

async fn toggle_todo(
    State(db): State<Db>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Todo>>, (StatusCode, Json<ApiResponse<()>>)> {
    let conn = db.conn.lock().unwrap();
    
    let existing = conn
        .prepare("SELECT * FROM todos WHERE id = ?")
        .unwrap()
        .query_row(params![&id], Db::parse_todo);
    
    let existing = match existing {
        Ok(t) => t,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse { success: false, data: None, error: Some("Todo not found".to_string()), message: None }),
            ));
        }
    };
    
    let new_completed = !existing.completed;
    let now = Utc::now();
    
    conn.execute(
        "UPDATE todos SET completed=?, updated_at=? WHERE id=?",
        params![if new_completed { 1 } else { 0 }, now.to_rfc3339(), &id],
    )
    .unwrap();
    
    let updated = Todo {
        id: existing.id,
        text: existing.text,
        completed: new_completed,
        important: existing.important,
        urgent: existing.urgent,
        quadrant: existing.quadrant,
        due_date: existing.due_date,
        tags: existing.tags,
        created_at: existing.created_at,
        updated_at: now,
    };
    
    Ok(Json(ApiResponse { success: true, data: Some(updated), error: None, message: None }))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339()
    }))
}

// ============ Main ============

#[tokio::main]
async fn main() {
    let db = Db::new();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/todos", get(get_all_todos))
        .route("/api/v1/todos", post(create_todo))
        .route("/api/v1/todos/:id", get(get_todo_by_id))
        .route("/api/v1/todos/:id", put(update_todo))
        .route("/api/v1/todos/:id", delete(delete_todo))
        .route("/api/v1/todos/:id/toggle", patch(toggle_todo))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(db);
    
    let addr = "0.0.0.0:3001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    println!("🚀 Todo API server running on http://{}", addr);
    println!("📡 API endpoints available at http://{}/api/v1/todos", addr);
    
    axum::serve(listener, app).await.unwrap();
}