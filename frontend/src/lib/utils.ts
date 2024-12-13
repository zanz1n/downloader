import { Err, Ok, type Result } from "ts-results-es";
import type { z, ZodType } from "zod";
import { AppError } from "./error";

export type ErrorElement = {
    name: string;
    obj?: { _errors: string[] };
};

export function joinErrorFields(
    fieldSeparator: string,
    errorSeparator: string,
    data: ErrorElement[]
): string {
    let error = "";
    for (const field of data) {
        if (!field.obj) {
            continue;
        }

        if (error.length == 0) {
            error +=
                field.name +
                ": " +
                field.obj._errors.join(errorSeparator + " ");
        } else {
            error +=
                fieldSeparator +
                " " +
                field.name +
                ": " +
                field.obj._errors.join(errorSeparator + " ");
        }
    }

    return error;
}

export async function safeParse<T extends ZodType>(
    schema: T,
    res: Response
): Promise<Result<z.infer<T>, AppError>> {
    try {
        const data = await res.json();
        return Ok(schema.parse(data));
    } catch (e) {
        if (e instanceof Error) {
            return Err(new AppError(e.message));
        }
        return Err(new AppError("Unknown"));
    }
}
