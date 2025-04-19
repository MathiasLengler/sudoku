import { useRegisterSW } from "virtual:pwa-register/react";

export function SwManager() {
    useRegisterSW({
        // Enable automatic reload
        immediate: true,
    });

    return null;
}
