import z from "zod";

const ConfigSchema = z.object({
  AUTHELIA_BASE_URL: z.url(),
});

export const config = ConfigSchema.parse(process.env);
