import CssBaseline from "@mui/material/CssBaseline";
import { createTheme, StyledEngineProvider, ThemeProvider } from "@mui/material/styles";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useAtomValue } from "jotai";
import { useEffect, useMemo, type ReactNode } from "react";
import { appSettingsState } from "../state/forms/appSettings";

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

export function MyTheme({ children }: MyThemeProps) {
    const systemPrefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)");
    const appSettings = useAtomValue(appSettingsState);

    // Determine dark mode based on user setting
    const prefersDarkMode = useMemo(() => {
        if (appSettings.colorMode === "auto") {
            return systemPrefersDarkMode;
        }
        return appSettings.colorMode === "dark";
    }, [appSettings.colorMode, systemPrefersDarkMode]);

    // Apply theme hue to CSS variables
    useEffect(() => {
        document.documentElement.style.setProperty("--hue", String(appSettings.themeColorHue));
    }, [appSettings.themeColorHue]);

    const theme = useMemo(() => {
        return createTheme({
            palette: {
                primary: {
                    main: prefersDarkMode
                        ? `hsl(${appSettings.themeColorHue}, 70%, 66%)`
                        : `hsl(${appSettings.themeColorHue}, 85%, 34%)`,
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
    }, [prefersDarkMode, appSettings.themeColorHue]);

    return (
        <StyledEngineProvider injectFirst>
            <ThemeProvider theme={theme}>
                <CssBaseline />
                {children}
            </ThemeProvider>
        </StyledEngineProvider>
    );
}
