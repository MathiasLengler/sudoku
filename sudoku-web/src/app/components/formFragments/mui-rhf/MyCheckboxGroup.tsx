import {
    FormControl,
    FormControlLabel,
    FormLabel,
    Checkbox,
    FormGroup,
    type FormControlProps,
} from "@mui/material";
import {
    type FieldValues,
    type FieldPath,
    type Control,
    useController,
    useFormState,
} from "react-hook-form";

export type MyCheckboxOption = {
    id: string | number;
    label: React.ReactNode;
    disabled?: boolean;
};

export type MyCheckboxGroupProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    label?: React.ReactNode;
    options: MyCheckboxOption[];
    required?: boolean;
    row?: boolean;
} & Pick<FormControlProps, "fullWidth" | "disabled">;

export function MyCheckboxGroup<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MyCheckboxGroupProps<TFieldValues, TName, TTransformedValues>) {
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

    // The field value should be an array of selected option ids
    const selectedValues: (string | number)[] = Array.isArray(field.value) ? field.value : [];

    const handleChange = (optionId: string | number, checked: boolean) => {
        if (checked) {
            field.onChange([...selectedValues, optionId]);
        } else {
            field.onChange(selectedValues.filter((id) => id !== optionId));
        }
    };

    return (
        <FormControl fullWidth={fullWidth} disabled={isDisabled} required={required}>
            {label && <FormLabel>{label}</FormLabel>}
            <FormGroup row={row} onBlur={field.onBlur}>
                {options.map((option) => (
                    <FormControlLabel
                        key={option.id}
                        control={
                            <Checkbox
                                name={field.name}
                                checked={selectedValues.includes(option.id)}
                                onChange={(e) => handleChange(option.id, e.target.checked)}
                            />
                        }
                        label={option.label}
                        disabled={option.disabled}
                    />
                ))}
            </FormGroup>
        </FormControl>
    );
}
