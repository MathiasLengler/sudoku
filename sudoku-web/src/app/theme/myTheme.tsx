import type { ReactNode } from "react";
import React from "react";
import useMediaQuery from "@mui/material/useMediaQuery";
import { createTheme, StyledEngineProvider, ThemeProvider } from "@mui/material/styles";
import CssBaseline from "@mui/material/CssBaseline";
import "./styles";

/* eslint-disable @typescript-eslint/consistent-type-definitions */
declare module "@mui/material/styles" {
    interface TypographyVariants {
        code: React.CSSProperties;
        fontFamilyMonospace: React.CSSProperties["fontFamily"];
    }

    // allow configuration using `createTheme`
    interface TypographyVariantsOptions {
        code?: React.CSSProperties;
        fontFamilyMonospace?: React.CSSProperties["fontFamily"];
    }
}

// Update the Typography's variant prop options
declare module "@mui/material/Typography" {
    interface TypographyPropsVariantOverrides {
        code: true;
    }
}
/* eslint-enable @typescript-eslint/consistent-type-definitions */

type MyThemeProps = {
    children: ReactNode;
};

const fontFamily = ['"Roboto Flex Variable"', '"Roboto"', '"Helvetica"', '"Arial"', "sans-serif"].join(",");
const fontFamilyMonospace = ['"Inconsolata"', "monospace"].join(",");

// TODO: switch to: https://mui.com/material-ui/experimental-api/css-theme-variables/overview/
export const MyTheme = ({ children }: MyThemeProps) => {
    const prefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)");

    const theme = React.useMemo(() => {
        return createTheme({
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
            typography: {
                fontFamily,
                fontFamilyMonospace,
                code: {
                    fontFamily: fontFamilyMonospace,
                    color: prefersDarkMode
                        ? undefined
                        : // non-transparent black for cleaner overdraw of characters.
                          "rgb(16 16 16)",
                    overflowWrap: "break-word",
                    overflowX: "auto",
                },
            },
            components: {
                MuiTooltip: {
                    defaultProps: {
                        enterDelay: 700,
                        leaveDelay: 200,
                    },
                },
                MuiDialogContent: {
                    defaultProps: {
                        dividers: true,
                    },
                },
                MuiDialogActions: {
                    defaultProps: {
                        sx: {
                            justifyContent: "space-between",
                        },
                    },
                },
                MuiTypography: {
                    defaultProps: {
                        variantMapping: {
                            code: "pre",
                        },
                    },
                },
                MuiStack: {
                    defaultProps: {
                        useFlexGap: true,
                    },
                },
            },
        });
    }, [prefersDarkMode]);

    return (
        <StyledEngineProvider injectFirst>
            <ThemeProvider theme={theme}>
                <CssBaseline />
                {children}
            </ThemeProvider>
        </StyledEngineProvider>
    );
};
