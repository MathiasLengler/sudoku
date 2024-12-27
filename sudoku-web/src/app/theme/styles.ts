// MUI Pigment CSS
import "@pigment-css/react/styles.css";
// Fonts
import "@fontsource-variable/roboto-flex/index.css";
import "../../../res/fonts/inconsolata.css";
// Custom styles
import "../../../res/styles.css";

import type { Theme } from "@mui/material/styles";

declare module "@mui/material-pigment-css" {
    interface ThemeArgs {
        theme: Theme;
    }
}
