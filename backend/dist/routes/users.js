"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = require("express");
const zod_1 = require("zod");
const db_1 = __importDefault(require("../db"));
const router = (0, express_1.Router)();
const RegisterSchema = zod_1.z.object({
    wallet_address: zod_1.z.string().min(1).max(64),
});
// POST /api/users — register or login by wallet address (upsert)
router.post("/", async (req, res) => {
    const parsed = RegisterSchema.safeParse(req.body);
    if (!parsed.success) {
        return res.status(400).json({ error: "Invalid request", details: parsed.error.flatten() });
    }
    const { wallet_address } = parsed.data;
    try {
        const result = await db_1.default.query(`INSERT INTO users (wallet_address)
       VALUES ($1)
       ON CONFLICT (wallet_address) DO UPDATE SET wallet_address = EXCLUDED.wallet_address
       RETURNING id, wallet_address, phone, created_at`, [wallet_address]);
        return res.status(200).json(result.rows[0]);
    }
    catch (err) {
        console.error("POST /api/users error:", err);
        return res.status(500).json({ error: "Internal server error" });
    }
});
exports.default = router;
//# sourceMappingURL=users.js.map