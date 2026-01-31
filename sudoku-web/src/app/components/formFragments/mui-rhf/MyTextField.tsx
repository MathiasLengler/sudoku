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
    const { control, name, disabled = false, ...restProps } = props;

    const {
        field,
        fieldState: { error },
    } = useController({
        name,
        control,
    });

    const { isSubmitting } = useFormState({
        control,
    });

    const isDisabled = disabled || isSubmitting;
    return (
        <TextField
            {...restProps}
            name={field.name}
            value={field.value}
            onChange={field.onChange}
            onBlur={field.onBlur}
            disabled={isDisabled}
            error={!!error}
            helperText={getFieldErrorMessage(error)}
        />
    );
}
