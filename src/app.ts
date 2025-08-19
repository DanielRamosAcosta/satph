import { Hono } from "hono";
import z, { ZodError } from "zod";
import { authelia, AutheliaError } from "./authelia.ts";
import { zValidator } from "./zValidator.ts";
import { logger } from "./logger.ts";

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
    logger.info({ err }, "Authentication error");
    return c.json(
      { status: RESPONSE_STATUS.FAILURE, message: err.message },
      401
    );
  }
  if (err instanceof ZodError) {
    logger.info({ err }, "Validation error");
    return c.json(
      {
        status: RESPONSE_STATUS.FAILURE,
        message: "Validation Error",
        details: z.treeifyError(err),
      },
      400
    );
  }

  logger.error({ err }, "Internal server error");
  
  return c.json(
    { status: RESPONSE_STATUS.FAILURE, message: "Internal Server Error" },
    500
  );
});

honoApp.post("/auth", zValidator("json", AuthRequestSchema), async (c) => {
  const result = await c.req.valid("json");
  logger.info({ username: result.username, ip: result.ip, protocol: result.protocol }, "Received auth request");

  if (isLocalNetwork(result.ip) || isLocalhost(result.ip)) {
    logger.info({ ip: result.ip }, "Internal request from IP");
    await authelia.firstFactor(result.username, result.password);
    return c.json({ status: RESPONSE_STATUS.SUCCESS });
  }

  const realPassword = result.password.slice(0, -6);
  const totp = result.password.slice(-6);

  const session = await authelia.firstFactor(result.username, realPassword);

  await authelia.secondFactor(session, totp);

  return c.json({ status: RESPONSE_STATUS.SUCCESS });
});

honoApp.get("/health", async (c) => {
  try {
    return c.json({ status: "ok", authelia: await authelia.ping() });
  } catch (err) {
    return c.json(
      {
        status: "fail",
        authelia: err instanceof Error ? err.message : String(err),
      },
      503
    );
  }
});

function isLocalNetwork(ip: string) {
  return ip.startsWith("192.168.");
}

function isLocalhost(ip: string) {
  return ip === "127.0.0.1" || ip === "::1";
}

export const app = honoApp;
