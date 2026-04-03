import { Link } from "@mui/material";
import type { ReactNode } from "react";

export function ExternalLink({ children, href }: { children: ReactNode; href: string }) {
    return (
        <Link rel="noopener" target="_blank" href={href} color="inherit" underline="hover">
            {children}
        </Link>
    );
}
