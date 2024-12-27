import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import wasm from "vite-plugin-wasm";

// TODO: migrate remaining webpack config

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
    esbuild: {
        supported: {
            "top-level-await": true,
        },
        ...(mode === "profile" && {
            minifyIdentifiers: false, // makes Chrome DevTools easier to use
        }),
    },
    server: {
        fs: {
            // Allow serving files from one level up to the project root
            allow: [".."],
        },
        headers: {
            "Cross-Origin-Embedder-Policy": "require-corp",
            "Cross-Origin-Opener-Policy": "same-origin",
        },
    },
    worker: {
        format: "es",
    },
    plugins: [react(), wasm()],
    ...(mode === "profile" && {
        resolve: {
            alias: {
                "react-dom/client": "react-dom/profiling",
            },
        },
    }),
}));
