import { type SliderProps, Slider, FormControl, FormLabel, type FormControlProps } from "@mui/material";
import { type FieldValues, type FieldPath, type Control, useController, useFormState } from "react-hook-form";

export type MySliderProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    label?: React.ReactNode;
} & Omit<SliderProps, "name" | "value" | "onChange" | "onBlur"> &
    Pick<FormControlProps, "fullWidth">;

export function MySlider<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>(props: MySliderProps<TFieldValues, TName, TTransformedValues>) {
    const { control, name, label, disabled = false, fullWidth, ...restProps } = props;

    const { field } = useController({
        name,
        control,
    });

    const { isSubmitting } = useFormState({
        control,
    });

    const isDisabled = disabled || isSubmitting;

    return (
        <FormControl fullWidth={fullWidth}>
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
