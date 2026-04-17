import request from "supertest";
import app from "../app";
import pool from "../db";

// Mock the db pool so tests don't need a real DB
jest.mock("../db", () => ({
  query: jest.fn(),
}));

const mockQuery = pool.query as jest.Mock;

describe("POST /api/users", () => {
  afterEach(() => jest.clearAllMocks());

  it("returns 400 when wallet_address is missing", async () => {
    const res = await request(app).post("/api/users").send({});
    expect(res.status).toBe(400);
  });

  it("upserts and returns user on valid wallet_address", async () => {
    const user = {
      id: "abc-123",
      wallet_address: "GABC123",
      phone: null,
      created_at: new Date().toISOString(),
    };
    mockQuery.mockResolvedValueOnce({ rows: [user] });

    const res = await request(app)
      .post("/api/users")
      .send({ wallet_address: "GABC123" });

    expect(res.status).toBe(200);
    expect(res.body.wallet_address).toBe("GABC123");
    expect(res.body.id).toBe("abc-123");
  });

  it("returns same user on second call with same wallet_address (idempotent)", async () => {
    const user = {
      id: "abc-123",
      wallet_address: "GABC123",
      phone: null,
      created_at: new Date().toISOString(),
    };
    mockQuery.mockResolvedValue({ rows: [user] });

    const res1 = await request(app)
      .post("/api/users")
      .send({ wallet_address: "GABC123" });
    const res2 = await request(app)
      .post("/api/users")
      .send({ wallet_address: "GABC123" });

    expect(res1.body.id).toBe(res2.body.id);
  });
});
