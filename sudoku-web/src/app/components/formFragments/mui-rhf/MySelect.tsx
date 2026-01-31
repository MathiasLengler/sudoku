import {
    type SelectProps,
    Select,
    FormControl,
    FormLabel,
    MenuItem,
    FormHelperText,
    type FormControlProps,
} from "@mui/material";
import {
    type FieldValues,
    type FieldPath,
    type Control,
    useController,
    useFormState,
} from "react-hook-form";
import { getFieldErrorMessage } from "./util";

export type MySelectOption = {
    id: string | number;
    label: React.ReactNode;
    disabled?: boolean;
};

export type MySelectProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    label?: React.ReactNode;
    options: MySelectOption[];
    helperText?: React.ReactNode;
    required?: boolean;
} & Omit<SelectProps, "name" | "value" | "onChange" | "onBlur" | "error"> &
    Pick<FormControlProps, "fullWidth">;

export function MySelect<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MySelectProps<TFieldValues, TName, TTransformedValues>) {
    const { control, name, label, options, helperText, required, disabled, fullWidth, ...restProps } =
        props;

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

    // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
    const isDisabled = disabled || isSubmitting;
    const hasError = !!error;
    const displayHelperText = getFieldErrorMessage(error) ?? helperText;

    return (
        <FormControl fullWidth={fullWidth} error={hasError} disabled={isDisabled} required={required}>
            {label && <FormLabel>{label}</FormLabel>}
            <Select
                {...restProps}
                name={field.name}
                value={field.value ?? ""}
                onChange={field.onChange}
                onBlur={field.onBlur}
                error={hasError}
            >
                {options.map((option) => (
                    <MenuItem key={option.id} value={option.id} disabled={option.disabled}>
                        {option.label}
                    </MenuItem>
                ))}
            </Select>
            {displayHelperText && <FormHelperText>{displayHelperText}</FormHelperText>}
        </FormControl>
    );
}
