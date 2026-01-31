import "@mantine/core/styles.css";
import "@mantine/notifications/styles.css";

import { createTheme, MantineProvider } from "@mantine/core";
import { useColorScheme } from "@mantine/hooks";
import type { ReactNode } from "react";

type MyThemeProps = {
    children: ReactNode;
};

const fontFamily = ['"Roboto Flex Variable"', '"Roboto"', '"Helvetica"', '"Arial"', "sans-serif"].join(",");
const fontFamilyMonospace = ['"Inconsolata"', "monospace"].join(",");

const theme = createTheme({
    fontFamily,
    fontFamilyMonospace,
    primaryColor: "blue",
    defaultRadius: "sm",
    components: {
        Tooltip: {
            defaultProps: {
                openDelay: 700,
                closeDelay: 200,
            },
        },
    },
});

export function MyTheme({ children }: MyThemeProps) {
    const colorScheme = useColorScheme();

    return (
        <MantineProvider theme={theme} defaultColorScheme="auto" forceColorScheme={colorScheme}>
            {children}
        </MantineProvider>
    );
}
