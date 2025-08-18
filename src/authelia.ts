import { config } from "./config.ts";

export class AutheliaError extends Error {}

export const authelia = {
  firstFactor: async (username: string, password: string) => {
    console.log("Authenticating user", username, "with first factor");
    const response = await fetch(`${config.AUTHELIA_BASE_URL}/api/firstfactor`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ username, password }),
    });

    if (!response.ok) {
      throw new AutheliaError("Failed to authenticate with first factor");
    }

    const cookies = response.headers.get("set-cookie");

    if (!cookies) {
        throw new AutheliaError("No cookies received from Authelia");
    }

    const authCookie = cookies.split("; ")
        .map(cookie => cookie.trim().split("="))
        .find(([name]) => name === "authelia_session");

    if (!authCookie) {
        throw new AutheliaError("No session cookie received from Authelia");
    }

    const value = authCookie[1]

    if (!value) {
        throw new AutheliaError("No session cookie value received from Authelia");
    }

    return value;
  },

  secondFactor: async (session: string, totp: string) => {
    console.log("Authenticating user with second factor");
    const response = await fetch(`${config.AUTHELIA_BASE_URL}/api/secondfactor/totp`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Cookie": `authelia_session=${session}`,
      },
      body: JSON.stringify({ token: totp }),
    });

    if (!response.ok) {
      throw new AutheliaError("Failed to authenticate with second factor");
    }
  },

  ping: async (): Promise<string> => {
    try {
      const response = await fetch(`${config.AUTHELIA_BASE_URL}/api/health`);
      if (!response.ok) {
        throw new AutheliaError(`Authelia health check failed: ${response.status}`);
      }
      const data = await response.json();
      return data.status;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new AutheliaError(`Authelia connectivity error: ${message}`);
    }
  },
};
