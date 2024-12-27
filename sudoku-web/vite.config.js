import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import { pigment } from "@pigment-css/vite-plugin";
const pigmentConfig = {
    transformLibraries: ["@mui/material"],
};
// https://vite.dev/config/
export default defineConfig({
    plugins: [react(), pigment(pigmentConfig)],
});
