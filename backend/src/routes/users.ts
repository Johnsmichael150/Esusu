import { Router, Request, Response } from "express";
import { z } from "zod";
import pool from "../db";

const router = Router();

const RegisterSchema = z.object({
  wallet_address: z.string().min(1).max(64),
});

// POST /api/users — register or login by wallet address (upsert)
router.post("/", async (req: Request, res: Response) => {
  const parsed = RegisterSchema.safeParse(req.body);
  if (!parsed.success) {
    return res.status(400).json({ error: "Invalid request", details: parsed.error.flatten() });
  }

  const { wallet_address } = parsed.data;

  try {
    const result = await pool.query<{
      id: string;
      wallet_address: string;
      phone: string | null;
      created_at: string;
    }>(
      `INSERT INTO users (wallet_address)
       VALUES ($1)
       ON CONFLICT (wallet_address) DO UPDATE SET wallet_address = EXCLUDED.wallet_address
       RETURNING id, wallet_address, phone, created_at`,
      [wallet_address]
    );

    return res.status(200).json(result.rows[0]);
  } catch (err) {
    console.error("POST /api/users error:", err);
    return res.status(500).json({ error: "Internal server error" });
  }
});

export default router;
