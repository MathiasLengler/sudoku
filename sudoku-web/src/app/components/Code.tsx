import { Typography } from "@mui/material";
import React from "react";

type CodeProps = {
    children: string;
    wrap?: boolean;
};

export const Code: React.FunctionComponent<CodeProps> = ({ children, wrap = false }) => {
    return (
        <Typography
            sx={[
                {
                    minHeight: "1lh",
                },
                wrap
                    ? {
                          whiteSpace: "pre-wrap",
                      }
                    : {
                          whiteSpace: "pre",
                      },
            ]}
            variant="code"
        >
            {children}
        </Typography>
    );
};
