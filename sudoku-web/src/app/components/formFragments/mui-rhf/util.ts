import type { FieldError } from "react-hook-form";

export function getFieldErrorMessage(
    fieldError:
        | FieldError
        // zod array errors
        | (FieldError | undefined)[]
        | undefined,
): string | undefined {
    if (!fieldError) {
        return undefined;
    }
    if (Array.isArray(fieldError)) {
        return fieldError
            .filter((err) => !!err)
            .map((err) => err.message)
            .join(", ");
    }
    return fieldError.message;
}
