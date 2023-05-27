import React from "react";
import { ALL_STRATEGIES } from "../../../constants";
import { CheckboxButtonGroup } from "react-hook-form-mui";
import type { Control, FieldValues, Path } from "react-hook-form";

interface SelectStrategiesProps<T extends FieldValues> {
    control: Control<T>;
    name: Path<T>;
}
function SelectStrategies<T extends FieldValues>({ control, name }: SelectStrategiesProps<T>) {
    return (
        <CheckboxButtonGroup
            control={control}
            name={name}
            label="Strategies"
            options={ALL_STRATEGIES.map(strategy => ({ id: strategy, label: strategy }))}
            row
            required
        />
    );
}

export default SelectStrategies;
