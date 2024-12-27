import type { ReactNode } from "react";
import React from "react";
import IconButton from "@mui/material/IconButton";
import CloseIcon from "@mui/icons-material/Close";
import { NotificationsProvider } from "@toolpad/core/useNotifications";

type MySnackbarProviderProps = {
    children: ReactNode;
};
export function MyNotificationsProvider({ children }: MySnackbarProviderProps) {
    return (
        <NotificationsProvider
        // preventDuplicate
        // anchorOrigin={{
        //     vertical: "top",
        //     horizontal: "left",
        // }}
        // action={(key) => (
        //     <IconButton
        //         onClick={() => {
        //             closeSnackbar(key);
        //         }}
        //         size="small"
        //         aria-label="Close"
        //     >
        //         <CloseIcon fontSize="small" />
        //     </IconButton>
        // )}
        >
            {children}
        </NotificationsProvider>
    );
}
