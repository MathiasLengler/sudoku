import { type SliderProps, FormControl, FormLabel, Slider } from "@mui/material";
import { type Control, type FieldPath, type FieldValues, useController, useFormState } from "react-hook-form";

export type MySliderProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    label?: React.ReactNode;
} & Omit<SliderProps, "name" | "value" | "onChange" | "onBlur">;

export function MySlider<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MySliderProps<TFieldValues, TName, TTransformedValues>) {
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
        <FormControl fullWidth>
            {label && <FormLabel>{label}</FormLabel>}
            <Slider
                {...restProps}
                name={field.name}
                value={field.value}
                onChange={(_, value) => {
                    field.onChange(value);
                }}
                onBlur={field.onBlur}
                disabled={isDisabled}
            />
        </FormControl>
    );
}
