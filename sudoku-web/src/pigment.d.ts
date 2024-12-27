// Required for module augmentation
/* eslint-disable @typescript-eslint/consistent-type-definitions */

import type { Theme, SxProps } from "@mui/material/styles";
import {} from "@mui/material/themeCssVarsAugmentation";

// Pigment CSS is themed using a @mui theme
declare module "@mui/material-pigment-css" {
    interface ThemeArgs {
        theme: Theme;
    }
}

// Any JSX element can have sx prop
declare global {
    namespace React {
        interface HTMLAttributes {
            sx?: SxProps<Theme>;
        }
        interface SVGProps {
            sx?: SxProps<Theme>;
        }
    }
}

// Custom typography
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
