import type { ReactNode } from "react";

import { Link } from "@mui/material";

export function ExternalLink({ children, href }: { children: ReactNode; href: string }) {
    return (
        <Link rel="noopener" target="_blank" href={href} color="inherit" underline="hover">
            {children}
        </Link>
    );
}
