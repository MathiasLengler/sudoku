import type { ReactNode } from "react";

import { Fieldset as MantineFieldset } from "@mantine/core";

type FieldsetProps = {
    label: string;
    disabled?: boolean;
    children: ReactNode;
};

export function Fieldset({ label, disabled, children }: FieldsetProps) {
    return (
        <MantineFieldset legend={label} disabled={disabled}>
            {children}
        </MantineFieldset>
    );
}
