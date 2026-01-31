import { type TextFieldProps, TextField } from "@mui/material";
import { type FieldValues, type FieldPath, type Control, useController, useFormState } from "react-hook-form";
import { getFieldErrorMessage } from "./util";

export type MyTextFieldProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
} & Omit<TextFieldProps, "name" | "value" | "onChange" | "onBlur" | "error" | "helperText">;

export function MyTextField<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MyTextFieldProps<TFieldValues, TName, TTransformedValues>) {
    const {
        field,
        fieldState: { error },
    } = useController({
        name: props.name,
        control: props.control,
    });

    const { isSubmitting } = useFormState({
        control: props.control,
    });

    return (
        <TextField
            {...props}
            name={field.name}
            value={field.value}
            onChange={field.onChange}
            onBlur={field.onBlur}
            // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
            disabled={props.disabled || isSubmitting}
            error={!!error}
            helperText={getFieldErrorMessage(error)}
        />
    );
}
