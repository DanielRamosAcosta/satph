import { Hono } from "hono";
import { zValidator } from "@hono/zod-validator";
import z from "zod";
import { authelia, AutheliaError } from "./authelia.ts";

const honoApp = new Hono();

const AuthRequestSchema = z.object({
  username: z.string().min(2).max(100),
  password: z.string().min(6).max(100),
  ip: z.string(),
  protocol: z.enum(["SSH", "FTP", "DAV"]),
});

const RESPONSE_STATUS = {
  SUCCESS: 1,
  FAILURE: 0,
} as const;

honoApp.onError((err, c) => {
  if (err instanceof AutheliaError) {
    console.log("Authentication error: ", err.message);
    return c.json(
      { status: RESPONSE_STATUS.FAILURE, message: err.message },
      401
    );
  }

  return c.json(
    { status: RESPONSE_STATUS.FAILURE, message: "Internal Server Error" },
    500
  );
});

honoApp.post("/auth", zValidator("json", AuthRequestSchema), async (c) => {
  const result = await c.req.valid("json");
  console.log(
    "Received auth request",
    result.username,
    result.ip,
    result.protocol
  );

  const realPassword = result.password.slice(0, -6);
  const totp = result.password.slice(-6);

  const session = await authelia.firstFactor(result.username, realPassword);

  await authelia.secondFactor(session, totp);

  return c.json({ status: RESPONSE_STATUS.SUCCESS });
});

honoApp.get("/health", async (c) => {
  try {
    await authelia.ping();
    return c.json({ status: "ok", authelia: "reachable" });
  } catch (err) {
    return c.json({ status: "fail", authelia: err instanceof Error ? err.message : String(err) }, 503);
  }
});

export const app = honoApp;
