import { z } from "zod";

export class AppError extends Error {
    constructor(
        public error: string,
        public errorCode = 0
    ) {
        if (errorCode == 0) {
            super(error);
        } else {
            super(`${errorCode}: ${error}`);
        }
    }

    toString(): string {
        return this.message;
    }
}

export const appErrorSchema = z
    .object({
        error: z.string(),
        error_code: z.number().int().nonnegative()
    })
    .transform((o) => new AppError(o.error, o.error_code));
