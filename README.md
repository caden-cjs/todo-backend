# Todo List Backend API (Rust)

A high-performance RESTful API for managing todos, built with Rust, Axum, and SQLite.

## Features

- High performance with Rust + Axum async web framework
- SQLite database with rusqlite (bundled)
- CRUD operations for todos
- Automatic quadrant calculation (Eisenhower Matrix)
- Filter by quadrant and completion status
- Tag support and due date management
- CORS enabled for frontend integration

## Tech Stack

- **Axum 0.7** - Modern async web framework
- **Tokio** - Async runtime
- **rusqlite** - SQLite bindings
- **Serde** - JSON serialization

## Installation

### Prerequisites
- Rust 1.70+ (install via https://rustup.rs)

### Build & Run

```bash
# Build release binary
cargo build --release

# Run server
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
GET /api/v1/todos?quadrant=2&completed=false
```

### Get Single Todo
```http
GET /api/v1/todos/:id
```

### Create Todo
```http
POST /api/v1/todos
Content-Type: application/json

{"text":"New task","important":true,"urgent":false}
```

### Update Todo
```http
PUT /api/v1/todos/:id
Content-Type: application/json

{"text":"Updated","completed":true}
```

### Toggle Completion
```http
PATCH /api/v1/todos/:id/toggle
```

### Delete Todo
```http
DELETE /api/v1/todos/:id
```

## Eisenhower Matrix Quadrants

| Quadrant | Important | Urgent | Action |
|----------|-----------|--------|--------|
| 1 | Yes | Yes | Do First |
| 2 | Yes | No | Schedule |
| 3 | No | Yes | Delegate |
| 4 | No | No | Eliminate |

## License

MIT