import {
    FormControl,
    FormControlLabel,
    FormLabel,
    Radio,
    RadioGroup,
    type FormControlProps,
} from "@mui/material";
import {
    type FieldValues,
    type FieldPath,
    type Control,
    useController,
    useFormState,
} from "react-hook-form";

export type MyRadioOption = {
    id: string | number;
    label: React.ReactNode;
    disabled?: boolean;
};

export type MyRadioGroupProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    label?: React.ReactNode;
    options: MyRadioOption[];
    required?: boolean;
    row?: boolean;
} & Pick<FormControlProps, "fullWidth" | "disabled">;

export function MyRadioGroup<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MyRadioGroupProps<TFieldValues, TName, TTransformedValues>) {
    const { control, name, label, options, required, row, disabled, fullWidth } = props;

    const { field } = useController({
        name,
        control,
    });

    const { isSubmitting } = useFormState({
        control,
    });

    // eslint-disable-next-line @typescript-eslint/prefer-nullish-coalescing
    const isDisabled = disabled || isSubmitting;

    return (
        <FormControl fullWidth={fullWidth} disabled={isDisabled} required={required}>
            {label && <FormLabel>{label}</FormLabel>}
            <RadioGroup
                name={field.name}
                value={field.value ?? ""}
                onChange={field.onChange}
                onBlur={field.onBlur}
                row={row}
            >
                {options.map((option) => (
                    <FormControlLabel
                        key={option.id}
                        value={option.id}
                        control={<Radio />}
                        label={option.label}
                        disabled={option.disabled}
                    />
                ))}
            </RadioGroup>
        </FormControl>
    );
}
