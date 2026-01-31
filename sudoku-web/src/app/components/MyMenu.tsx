import { useState, type MouseEventHandler } from "react";

import { Menu } from "@mantine/core";

type MyMenuProps = {
    children: (params: { onMenuOpen: MouseEventHandler<HTMLButtonElement> }) => React.ReactNode;
    menuItems: {
        onClick: () => Promise<void> | void;
        label: string;
        icon?: React.ReactNode;
        disabled?: boolean;
    }[];
};

export function MyMenu({ children, menuItems }: MyMenuProps) {
    const [opened, setOpened] = useState(false);

    const makeHandleMenuClose = (action?: () => Promise<void> | void) => async () => {
        setOpened(false);
        if (action) {
            try {
                await action();
            } catch (err) {
                console.error("Error while executing menu action:", err);
            }
        }
    };

    const onMenuOpen: MouseEventHandler<HTMLButtonElement> = () => setOpened(true);

    return (
        <Menu opened={opened} onClose={() => setOpened(false)} shadow="md" width={200}>
            <Menu.Target>{children({ onMenuOpen })}</Menu.Target>
            <Menu.Dropdown>
                {menuItems.map((menuItem, i) => (
                    <Menu.Item
                        key={i}
                        onClick={makeHandleMenuClose(menuItem.onClick)}
                        disabled={menuItem.disabled}
                        leftSection={menuItem.icon}
                    >
                        {menuItem.label}
                    </Menu.Item>
                ))}
            </Menu.Dropdown>
        </Menu>
    );
}
