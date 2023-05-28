import React, { type MouseEvent } from "react";
import { Box, SvgIcon } from "@mui/material";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";
import type { IconButtonProps } from "@mui/material/IconButton/IconButton";

interface MyIconButtonProps {
    label: string;
    onClick: (event: MouseEvent<HTMLButtonElement>) => Promise<void> | void;
    disabled?: boolean;
    size?: "small" | "medium" | "large";
    icon: typeof SvgIcon;
    color?: IconButtonProps["color"];
}

function MyIconButton({ label, onClick, disabled = false, size, icon: Icon, color }: MyIconButtonProps) {
    return (
        <Tooltip title={label}>
            <Box className="icon-button-container">
                <IconButton
                    onClick={ev => {
                        (async () => await onClick(ev))().catch(err =>
                            console.error("Error in IconButton onClick:", err)
                        );
                    }}
                    size={size}
                    disabled={disabled}
                    color={color}
                    aria-label={label}
                >
                    <Icon fontSize={size} />
                </IconButton>
            </Box>
        </Tooltip>
    );
}

export default MyIconButton;
