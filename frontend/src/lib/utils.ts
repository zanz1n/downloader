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
            error += field.name + ": " + field.obj._errors.join(errorSeparator + " ");
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
