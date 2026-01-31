import type { MouseEvent, ReactNode } from "react";
import { ActionIcon, Indicator, Tooltip } from "@mantine/core";
import type { TablerIcon } from "@tabler/icons-react";

type MyIconButtonProps = {
    label: string;
    onClick: (event: MouseEvent<HTMLButtonElement>) => Promise<void> | void;
    disabled?: boolean;
    size?: "sm" | "md" | "lg";
    icon: TablerIcon;
    color?: string;
    badge?: ReactNode;
    badgeColor?: string;
};

function MyIconButton({
    label,
    onClick,
    disabled = false,
    size = "lg",
    icon: Icon,
    color,
    badge,
    badgeColor,
}: MyIconButtonProps) {
    const iconSize = size === "sm" ? 18 : size === "md" ? 22 : 26;
    const iconElement = <Icon size={iconSize} />;

    return (
        <Tooltip label={label}>
            <div className="icon-button-container">
                {badge ? (
                    <Indicator
                        label={badge}
                        color={badgeColor}
                        size={16}
                        position="top-end"
                        processing={false}
                    >
                        <ActionIcon
                            onClick={(ev) => {
                                (async () => await onClick(ev))().catch((err) =>
                                    console.error("Error in IconButton onClick:", err),
                                );
                            }}
                            size={size}
                            disabled={disabled}
                            color={color}
                            variant="subtle"
                            aria-label={label}
                        >
                            {iconElement}
                        </ActionIcon>
                    </Indicator>
                ) : (
                    <ActionIcon
                        onClick={(ev) => {
                            (async () => await onClick(ev))().catch((err) =>
                                console.error("Error in IconButton onClick:", err),
                            );
                        }}
                        size={size}
                        disabled={disabled}
                        color={color}
                        variant="subtle"
                        aria-label={label}
                    >
                        {iconElement}
                    </ActionIcon>
                )}
            </div>
        </Tooltip>
    );
}

export default MyIconButton;
