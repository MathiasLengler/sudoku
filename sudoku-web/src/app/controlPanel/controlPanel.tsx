import type * as React from "react";
import { ValueSelector } from "./valueSelector";
import { Toolbar } from "./toolbar";

export const ControlPanel = () => {
    return (
        <>
            <Toolbar />
            <ValueSelector />
        </>
    );
};
