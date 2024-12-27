import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import wasm from "vite-plugin-wasm";
import { optimizeLodashImports } from "@optimize-lodash/rollup-plugin";

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
        watch: {
            ignored: [
                // Defer watching of wasm-pack build output.
                // Background:
                // wasm-bindgen-rayon imports the wasm package itself from "snippets/".
                // "wasm-pack build" first updates the package.json, then emits those snippets, then finishes the containing wasm package.
                // If the vite watcher reloads between those points, the vite reload fails with:
                //   Failed to resolve import "../../.."
                // As a workaround, we ignore everything inside the wasm package,
                // except the top-level entry points (sudoku_wasm.js and similar).
                "**/sudoku-wasm/pkg/**",
                "!**/sudoku-wasm/pkg/sudoku_wasm.*",
            ],
        },
    },
    worker: {
        format: "es",
    },
    plugins: [react(), wasm(), optimizeLodashImports()],
    resolve: {
        alias: {
            lodash: "lodash-es",
            ...(mode === "profile" && {
                "react-dom/client": "react-dom/profiling",
            }),
        },
    },
}));
