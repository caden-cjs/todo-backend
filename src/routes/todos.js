import { Router } from "express";
import { getAllTodos, getTodoById, createTodo, updateTodo, deleteTodo, toggleTodo } from "../controllers/todoController.js";

const router = Router();
router.get("/", getAllTodos);
router.get("/:id", getTodoById);
router.post("/", createTodo);
router.put("/:id", updateTodo);
router.delete("/:id", deleteTodo);
router.patch("/:id/toggle", toggleTodo);
export default router;