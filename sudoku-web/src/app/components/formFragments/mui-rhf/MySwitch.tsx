import { type FormControlLabelProps, FormControlLabel, Switch, type SwitchProps } from "@mui/material";
import { type FieldValues, type FieldPath, type Control, useController, useFormState } from "react-hook-form";

export type MySwitchProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    label: React.ReactNode;
} & Omit<SwitchProps, "name" | "checked" | "onChange" | "onBlur"> &
    Omit<FormControlLabelProps, "control" | "label">;

export function MySwitch<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MySwitchProps<TFieldValues, TName, TTransformedValues>) {
    const { control, name, label, disabled = false, ...restProps } = props;

    const { field } = useController({
        name,
        control,
    });

    const { isSubmitting } = useFormState({
        control,
    });

    const isDisabled = disabled || isSubmitting;

    return (
        <FormControlLabel
            {...restProps}
            label={label}
            disabled={isDisabled}
            control={
                <Switch
                    name={field.name}
                    checked={!!field.value}
                    onChange={field.onChange}
                    onBlur={field.onBlur}
                    disabled={isDisabled}
                />
            }
        />
    );
}
