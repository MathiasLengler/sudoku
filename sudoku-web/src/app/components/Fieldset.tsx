import type { ReactNode } from "react";

import { FormControl, FormLabel } from "@mui/material";

type FieldsetProps = {
    label: string;
    disabled?: boolean;
    children: ReactNode;
};

export function Fieldset({ label, disabled, children }: FieldsetProps) {
    return (
        <FormControl component="fieldset" disabled={disabled}>
            <FormLabel component="legend">{label}</FormLabel>
            {children}
        </FormControl>
    );
}
