import type { ZodSchema } from "zod";
import z from "zod"
import { zValidator as zv } from '@hono/zod-validator'
import type { ValidationTargets } from 'hono'

export const zValidator = <T extends ZodSchema, Target extends keyof ValidationTargets>(
  target: Target,
  schema: T
) =>
  zv(target, schema, (result, c) => {
    if (!result.success) {
      throw result.error;
    }
  })
