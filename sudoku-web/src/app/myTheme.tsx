import React from "react";
import type { ReactNode } from "react";
import useMediaQuery from "@mui/material/useMediaQuery";
import { createTheme, StyledEngineProvider, ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";

interface MyThemeProps {
    children: ReactNode;
}

// TODO: switch to: https://mui.com/material-ui/experimental-api/css-theme-variables/overview/
export const MyTheme = ({ children }: MyThemeProps) => {
    const prefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)");

    const theme = React.useMemo(
        () =>
            createTheme({
                palette: {
                    primary: {
                        main: prefersDarkMode ? "#5FA1F2FF" : "#0D4FA0",
                    },
                    mode: prefersDarkMode ? "dark" : "light",
                    background: prefersDarkMode
                        ? {
                              default: "#121212",
                              paper: "#2C2C2C",
                          }
                        : {},
                },
                components: {
                    MuiTooltip: {
                        defaultProps: {
                            enterDelay: 700,
                            leaveDelay: 200,
                        },
                    },
                },
            }),
        [prefersDarkMode]
    );

    return (
        <StyledEngineProvider injectFirst>
            <ThemeProvider theme={theme}>
                <CssBaseline />
                {children}
            </ThemeProvider>
        </StyledEngineProvider>
    );
};
