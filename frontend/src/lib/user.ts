import { z } from "zod";

export const userSchema = z
    .object({
        id: z.string().uuid(),
        created_at: z.date(),
        updated_at: z.date(),
        permission: z.number().nonnegative(),
        username: z.string().min(1)
    })
    .transform((o) => ({
        id: o.id,
        createdAt: o.created_at,
        updatedAt: o.updated_at,
        permission: o.permission,
        username: o.username
    }));

export type User = z.infer<typeof userSchema>;
