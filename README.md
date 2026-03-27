# Todo List Backend API (Rust)

A high-performance RESTful API for managing todos, built with Rust, Axum, and SQLite.

## Features

- ⚡ High performance with Rust + Axum
- 📦 SQLite database with rusqlite
- ✅ CRUD operations for todos
- 📊 Automatic quadrant calculation (Eisenhower Matrix)
- 🔍 Filter by quadrant and completion status
- 🏷️ Tag support
- 📅 Due date management
- 🔓 CORS enabled

## Tech Stack

- **Axum** - Modern async web framework
- **Tokio** - Async runtime
- **rusqlite** - SQLite bindings
- **Serde** - JSON serialization

## Installation

### Prerequisites
- Rust 1.70+ (install via https://rustup.rs)

### Build & Run

```bash
# Build
cargo build --release

# Run
cargo run --release
```

Server runs on `http://localhost:3001`

## API Endpoints

### Base URL
```
http://localhost:3001/api/v1
```

### Get All Todos
```http
GET /api/v1/todos
```

Query parameters:
- `quadrant` (optional): Filter by quadrant (1-4)
- `completed` (optional): Filter by completion status (true/false)

### Get Single Todo
```http
GET /api/v1/todos/:id
```

### Create Todo
```http
POST /api/v1/todos
Content-Type: application/json

{
  "text": "New task",
  "important": true,
  "urgent": false,
  "dueDate": "2024-12-31",
  "tags": ["work"]
}
```

### Update Todo
```http
PUT /api/v1/todos/:id
Content-Type: application/json

{
  "text": "Updated task",
  "completed": true
}
```

### Toggle Completed Status
```http
PATCH /api/v1/todos/:id/toggle
```

### Delete Todo
```http
DELETE /api/v1/todos/:id
```

## Quadrant Calculation (Eisenhower Matrix)

| Quadrant | Important | Urgent | Description |
|----------|-----------|--------|-------------|
| 1 | ✅ | ✅ | Important & Urgent (Do First) |
| 2 | ✅ | ❌ | Important & Not Urgent (Schedule) |
| 3 | ❌ | ✅ | Not Important & Urgent (Delegate) |
| 4 | ❌ | ❌ | Not Important & Not Urgent (Eliminate) |

## Testing

```bash
# Create a todo
curl -X POST http://localhost:3001/api/v1/todos \
  -H "Content-Type: application/json" \
  -d '{"text":"Test task","important":true,"urgent":false}'

# Get all todos
curl http://localhost:3001/api/v1/todos

# Toggle completion
curl -X PATCH http://localhost:3001/api/v1/todos/{id}/toggle

# Delete a todo
curl -X DELETE http://localhost:3001/api/v1/todos/{id}
```

## Project Structure

```
todo-backend-rust/
├── Cargo.toml          # Dependencies
├── src/
│   └── main.rs         # Server implementation
├── data/
│   └── todos.db        # SQLite database
└── README.md
```

## Performance

Rust backend provides:
- ~10x faster response times compared to Node.js
- Lower memory footprint
- Thread-safe concurrent access via Mutex
- Zero-cost async with Tokio

## License

MIT