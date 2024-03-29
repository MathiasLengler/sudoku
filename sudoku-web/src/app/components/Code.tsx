import { Typography } from "@mui/material";
import React from "react";

type CodeProps = {
    children: string;
    wrap?: boolean;
};

export const Code: React.FunctionComponent<CodeProps> = ({ children, wrap = false }) => {
    return (
        <Typography
            sx={{
                whiteSpace: wrap ? "pre-wrap" : "pre",
                minHeight: "1lh",
            }}
            variant="code"
        >
            {children}
        </Typography>
    );
};
