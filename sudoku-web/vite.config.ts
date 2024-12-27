import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import { pigment, type PigmentOptions } from "@pigment-css/vite-plugin";
import { createTheme } from "@mui/material";

// TODO: migrate remaining webpack config

const pigmentConfig: PigmentOptions = {
    transformLibraries: ["@mui/material"],
    // TODO: migrate custom theme in here
    theme: createTheme({
        cssVariables: true,
        /* other parameters, if any */
    }),
};

// https://vite.dev/config/
export default defineConfig({
    plugins: [react(), pigment(pigmentConfig)],
});
