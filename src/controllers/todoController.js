import { v4 as uuidv4 } from "uuid";
import db from "../db.js";

const calcQ = (i, u) => i && u ? 1 : i && !u ? 2 : !i && u ? 3 : 4;
const parse = (r) => ({ id: r.id, text: r.text, completed: !!r.completed, important: !!r.important, urgent: !!r.urgent, quadrant: r.quadrant, dueDate: r.due_date, tags: r.tags ? JSON.parse(r.tags) : [], createdAt: r.created_at, updatedAt: r.updated_at });

export const getAllTodos = (req, res, next) => {
  try {
    const { quadrant, completed } = req.query;
    let q = "SELECT * FROM todos WHERE 1=1", p = [];
    if (quadrant) { q += " AND quadrant = ?"; p.push(+quadrant); }
    if (completed !== undefined) { q += " AND completed = ?"; p.push(completed === "true" ? 1 : 0); }
    q += " ORDER BY created_at DESC";
    res.json({ success: true, data: db.prepare(q).all(...p).map(parse) });
  } catch (e) { next(e); }
};

export const getTodoById = (req, res, next) => {
  try {
    const r = db.prepare("SELECT * FROM todos WHERE id = ?").get(req.params.id);
    if (!r) return res.status(404).json({ success: false, error: "Todo not found" });
    res.json({ success: true, data: parse(r) });
  } catch (e) { next(e); }
};

export const createTodo = (req, res, next) => {
  try {
    const { text, important = false, urgent = false, dueDate = null, tags = [] } = req.body;
    if (!text) return res.status(400).json({ success: false, error: "Text is required" });
    const id = uuidv4(), q = calcQ(important, urgent), now = new Date().toISOString();
    db.prepare("INSERT INTO todos VALUES (?, ?, 0, ?, ?, ?, ?, ?, ?, ?)").run(id, text.trim(), important ? 1 : 0, urgent ? 1 : 0, q, dueDate, JSON.stringify(tags), now, now);
    res.status(201).json({ success: true, data: { id, text: text.trim(), completed: false, important, urgent, quadrant: q, dueDate, tags, createdAt: now, updatedAt: now } });
  } catch (e) { next(e); }
};

export const updateTodo = (req, res, next) => {
  try {
    const { id } = req.params, ex = db.prepare("SELECT * FROM todos WHERE id = ?").get(id);
    if (!ex) return res.status(404).json({ success: false, error: "Todo not found" });
    const { text, completed, important, urgent, dueDate, tags } = req.body;
    const t = text !== undefined ? text.trim() : ex.text;
    const c = completed !== undefined ? (completed ? 1 : 0) : ex.completed;
    const i = important !== undefined ? (important ? 1 : 0) : ex.important;
    const u = urgent !== undefined ? (urgent ? 1 : 0) : ex.urgent;
    const q = calcQ(!!i, !!u);
    const now = new Date().toISOString();
    db.prepare("UPDATE todos SET text=?, completed=?, important=?, urgent=?, quadrant=?, due_date=?, tags=?, updated_at=? WHERE id=?").run(t, c, i, u, q, dueDate !== undefined ? dueDate : ex.due_date, tags !== undefined ? JSON.stringify(tags) : ex.tags, now, id);
    res.json({ success: true, data: { id, text: t, completed: !!c, important: !!i, urgent: !!u, quadrant: q, dueDate: dueDate !== undefined ? dueDate : ex.due_date, tags: tags !== undefined ? tags : JSON.parse(ex.tags), createdAt: ex.created_at, updatedAt: now } });
  } catch (e) { next(e); }
};

export const deleteTodo = (req, res, next) => {
  try { db.prepare("DELETE FROM todos WHERE id = ?").run(req.params.id); res.json({ success: true, message: "Todo deleted" }); } catch (e) { next(e); }
};

export const toggleTodo = (req, res, next) => {
  try {
    const t = db.prepare("SELECT * FROM todos WHERE id = ?").get(req.params.id);
    if (!t) return res.status(404).json({ success: false, error: "Todo not found" });
    const nc = t.completed ? 0 : 1, now = new Date().toISOString();
    db.prepare("UPDATE todos SET completed=?, updated_at=? WHERE id=?").run(nc, now, req.params.id);
    res.json({ success: true, data: { ...parse(t), completed: !!nc, updatedAt: now } });
  } catch (e) { next(e); }
};