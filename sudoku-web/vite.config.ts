import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import { pigment, type PigmentOptions } from "@pigment-css/vite-plugin";
import { createTheme } from "@mui/material";

// TODO: migrate remaining webpack config

const fontFamily = ['"Roboto Flex Variable"', '"Roboto"', '"Helvetica"', '"Arial"', "sans-serif"].join(",");
const fontFamilyMonospace = ['"Inconsolata"', "monospace"].join(",");

const pigmentConfig: PigmentOptions = {
    transformLibraries: ["@mui/material"],
    // TODO: migrate custom theme in here
    theme: createTheme({
        cssVariables: true,
        colorSchemes: {
            dark: {
                palette: {
                    primary: {
                        main: "#5FA1F2FF",
                    },
                    background: {
                        default: "#121212",
                        paper: "#2C2C2C",
                    },
                },
            },
            light: {
                palette: {
                    primary: {
                        main: "#0D4FA0",
                    },
                },
            },
        },
        typography: {
            fontFamily,
            fontFamilyMonospace,
            code: {
                fontFamily: fontFamilyMonospace,
                // TODO: evaluate if needed and migrate to new switching logic
                // color: prefersDarkMode
                //     ? undefined
                //     : // non-transparent black for cleaner overdraw of characters.
                //       "rgb(16 16 16)",
                overflowWrap: "break-word",
                overflowX: "auto",
            },
        },
    }),
};

// https://vite.dev/config/
export default defineConfig({
    plugins: [
        //
        react(),
        pigment(pigmentConfig),
    ],
});
