# Todo List Backend API

A RESTful API for managing todos with SQLite database.

## Features

- CRUD operations for todos
- Automatic quadrant calculation (Eisenhower Matrix)
- Filter by quadrant and completion status
- Tag support
- Due date management
- CORS enabled

## Installation

npm install

## Running

npm start

Server runs on http://localhost:3001

## API Endpoints

- GET /api/v1/todos - Get all todos
- GET /api/v1/todos/:id - Get single todo
- POST /api/v1/todos - Create todo
- PUT /api/v1/todos/:id - Update todo
- DELETE /api/v1/todos/:id - Delete todo
- PATCH /api/v1/todos/:id/toggle - Toggle completion

## License

MIT