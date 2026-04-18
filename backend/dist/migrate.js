"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
require("dotenv/config");
const fs_1 = __importDefault(require("fs"));
const path_1 = __importDefault(require("path"));
const db_1 = __importDefault(require("./db"));
async function runMigrations() {
    const migrationsDir = path_1.default.join(__dirname, "../migrations");
    const files = fs_1.default.readdirSync(migrationsDir).sort();
    await db_1.default.query(`
    CREATE TABLE IF NOT EXISTS schema_migrations (
      filename VARCHAR(255) PRIMARY KEY,
      applied_at TIMESTAMPTZ DEFAULT NOW()
    )
  `);
    for (const file of files) {
        if (!file.endsWith(".sql"))
            continue;
        const { rows } = await db_1.default.query("SELECT filename FROM schema_migrations WHERE filename = $1", [file]);
        if (rows.length > 0) {
            console.log(`Skipping already applied migration: ${file}`);
            continue;
        }
        const sql = fs_1.default.readFileSync(path_1.default.join(migrationsDir, file), "utf8");
        console.log(`Applying migration: ${file}`);
        await db_1.default.query(sql);
        await db_1.default.query("INSERT INTO schema_migrations (filename) VALUES ($1)", [file]);
        console.log(`Applied: ${file}`);
    }
    await db_1.default.end();
    console.log("All migrations complete.");
}
runMigrations().catch((err) => {
    console.error("Migration failed:", err);
    process.exit(1);
});
//# sourceMappingURL=migrate.js.map