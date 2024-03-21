import type { MouseEventHandler } from "react";
import React from "react";
import { ListItemIcon, ListItemText, Menu } from "@mui/material";
import MenuItem from "@mui/material/MenuItem";

interface MyMenuProps {
    children: (params: { onMenuOpen: MouseEventHandler<HTMLButtonElement> }) => React.ReactNode;
    menuItems: {
        onClick: () => Promise<void> | void;
        label: string;
        icon?: React.ReactNode;
        disabled?: boolean;
    }[];
}

export function MyMenu({ children, menuItems }: MyMenuProps) {
    const [menuAnchorEl, setMenuAnchorEl] = React.useState<null | HTMLElement>(null);

    const makeHandleMenuClose = (action?: () => Promise<void> | void) => async () => {
        setMenuAnchorEl(null);
        if (action) {
            try {
                await action();
            } catch (err) {
                console.error("Error while executing menu action:", err);
            }
        }
    };

    const onMenuOpen: MouseEventHandler<HTMLButtonElement> = (e) => setMenuAnchorEl(e.currentTarget);

    return (
        <>
            {children({ onMenuOpen })}
            <Menu keepMounted open={!!menuAnchorEl} anchorEl={menuAnchorEl} onClose={makeHandleMenuClose()}>
                {menuItems.map((menuItem, i) => (
                    <MenuItem key={i} onClick={makeHandleMenuClose(menuItem.onClick)} disabled={menuItem.disabled}>
                        {menuItem.icon && <ListItemIcon>{menuItem.icon}</ListItemIcon>}
                        <ListItemText>{menuItem.label}</ListItemText>
                    </MenuItem>
                ))}
            </Menu>
        </>
    );
}
