import CssBaseline from "@mui/material/CssBaseline";
import DefaultPropsProvider from "@mui/material/DefaultPropsProvider";
import type { ReactNode } from "react";

type MyThemeProps = {
    children: ReactNode;
};

export const MyTheme = ({ children }: MyThemeProps) => {
    return (
        <DefaultPropsProvider
            value={{
                MuiTooltip: {
                    enterDelay: 700,
                    leaveDelay: 200,
                },
                MuiDialogContent: {
                    dividers: true,
                },
                MuiDialogActions: {
                    sx: {
                        justifyContent: "space-between",
                    },
                },
                MuiTypography: {
                    variantMapping: {
                        code: "pre",
                    },
                },
                MuiStack: {
                    useFlexGap: true,
                },
            }}
        >
            <CssBaseline />
            {children}
        </DefaultPropsProvider>
    );
};
