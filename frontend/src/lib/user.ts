import { z } from "zod";

export const userSchema = z
    .object({
        id: z.string().uuid(),
        created_at: z.string().datetime(),
        updated_at: z.string().datetime(),
        permission: z.number().int().nonnegative(),
        username: z.string().min(1)
    })
    .transform((o) => ({
        id: o.id,
        createdAt: new Date(o.created_at),
        updatedAt: new Date(o.updated_at),
        permission: o.permission,
        username: o.username
    }));

export type User = z.infer<typeof userSchema>;
