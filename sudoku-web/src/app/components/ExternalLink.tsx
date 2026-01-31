import { Anchor } from "@mantine/core";
import type { ReactNode } from "react";

export function ExternalLink({ children, href }: { children: ReactNode; href: string }) {
    return (
        <Anchor href={href} target="_blank" rel="noopener" c="inherit" underline="hover">
            {children}
        </Anchor>
    );
}
