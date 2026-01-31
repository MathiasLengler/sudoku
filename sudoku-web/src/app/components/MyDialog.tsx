import { Modal } from "@mantine/core";
import { useMediaQuery } from "@mantine/hooks";
import type { ReactNode } from "react";

type MyDialogProps = {
    open: boolean;
    onClose: () => void;
    children: (onClose: () => void) => ReactNode;
};

export function MyDialog({ open, onClose, children }: MyDialogProps) {
    const isMobile = useMediaQuery("(max-width: 768px)");

    return (
        <div
            onKeyDown={(e) => {
                // Disable global game shortcuts in dialog boxes.
                e.stopPropagation();
            }}
        >
            <Modal
                opened={open}
                onClose={onClose}
                fullScreen={isMobile}
                size="lg"
                overlayProps={{ backgroundOpacity: 0.55, blur: 3 }}
            >
                {children(onClose)}
            </Modal>
        </div>
    );
}
