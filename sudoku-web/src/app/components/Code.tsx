import { Typography } from "@mui/material";

type CodeProps = {
    children: string;
    wrap?: boolean;
};

export function Code({ children, wrap = false }: CodeProps) {
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
}
