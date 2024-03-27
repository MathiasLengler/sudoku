import React from "react";
import type { MouseEvent, ReactNode } from "react";
import { Badge, Box, SvgIcon } from "@mui/material";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";
import type { IconButtonProps } from "@mui/material/IconButton/IconButton";
import type { BadgeProps } from "@mui/material/Badge/Badge";

type MyIconButtonProps = {
    label: string;
    onClick: (event: MouseEvent<HTMLButtonElement>) => Promise<void> | void;
    disabled?: boolean;
    size?: "small" | "medium" | "large";
    icon: typeof SvgIcon;
    color?: IconButtonProps["color"];
    badge?: ReactNode;
    badgeColor?: BadgeProps["color"];
}

function MyIconButton({
    label,
    onClick,
    disabled = false,
    size,
    icon: Icon,
    color,
    badge,
    badgeColor,
}: MyIconButtonProps) {
    const icon = <Icon fontSize={size} />;
    return (
        <Tooltip title={label}>
            <Box className="icon-button-container">
                <IconButton
                    onClick={(ev) => {
                        (async () => await onClick(ev))().catch((err) =>
                            console.error("Error in IconButton onClick:", err),
                        );
                    }}
                    size={size}
                    disabled={disabled}
                    color={color}
                    aria-label={label}
                >
                    {badge ? (
                        <Badge badgeContent={badge} color={badgeColor}>
                            {icon}
                        </Badge>
                    ) : (
                        icon
                    )}
                </IconButton>
            </Box>
        </Tooltip>
    );
}

export default MyIconButton;
