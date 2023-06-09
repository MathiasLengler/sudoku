import React from "react";
import { Typography } from "@mui/material";

interface CodeProps {
    children: string;
    wrap?: boolean;
}

export const Code: React.FunctionComponent<CodeProps> = ({ children, wrap = false }) => {
    return (
        <Typography
            sx={{
                whiteSpace: wrap ? "pre-wrap" : "pre",
            }}
            variant="code"
        >
            {children}
        </Typography>
    );
};
