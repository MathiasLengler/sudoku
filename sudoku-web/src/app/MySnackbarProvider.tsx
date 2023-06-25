import type { ReactNode } from "react";
import React from "react";
import { closeSnackbar, SnackbarProvider } from "notistack";
import IconButton from "@mui/material/IconButton";
import CloseIcon from "@mui/icons-material/Close";

interface MySnackbarProviderProps {
    children: ReactNode;
}
export function MySnackbarProvider({ children }: MySnackbarProviderProps) {
    return (
        <SnackbarProvider
            preventDuplicate
            anchorOrigin={{
                vertical: "top",
                horizontal: "left",
            }}
            action={key => (
                <IconButton
                    onClick={() => {
                        closeSnackbar(key);
                    }}
                    size="small"
                    aria-label="Close"
                >
                    <CloseIcon fontSize="small" />
                </IconButton>
            )}
        >
            {children}
        </SnackbarProvider>
    );
}
