import initSqlJs from "sql.js";
import { fileURLToPath } from "url";
import { dirname, join } from "path";
import fs from "fs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const dbPath = join(__dirname, "..", "data", "todos.db");
const dataDir = join(__dirname, "..", "data");

if (!fs.existsSync(dataDir)) fs.mkdirSync(dataDir, { recursive: true });

let db, SQL;

const initDb = async () => {
  SQL = await initSqlJs();
  if (fs.existsSync(dbPath)) {
    db = new SQL.Database(fs.readFileSync(dbPath));
  } else {
    db = new SQL.Database();
  }
  db.run("CREATE TABLE IF NOT EXISTS todos (id TEXT PRIMARY KEY, text TEXT NOT NULL, completed INTEGER DEFAULT 0, important INTEGER DEFAULT 0, urgent INTEGER DEFAULT 0, quadrant INTEGER DEFAULT 4, due_date TEXT, tags TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)");
  saveDb();
};

const saveDb = () => fs.writeFileSync(dbPath, Buffer.from(db.export()));

await initDb();

export default {
  prepare: (sql) => ({
    run: (...p) => { db.run(sql, p); saveDb(); },
    get: (...p) => { const s = db.prepare(sql); s.bind(p); if (s.step()) { const r = s.getAsObject(); s.free(); return r; } s.free(); },
    all: (...p) => { const s = db.prepare(sql); s.bind(p); const r = []; while (s.step()) r.push(s.getAsObject()); s.free(); return r; }
  })
};