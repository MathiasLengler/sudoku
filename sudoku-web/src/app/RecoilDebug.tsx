import React, { useEffect } from "react";
import { useRecoilSnapshot } from "recoil";

const isEnabled = true;

export function RecoilDebug() {
    const snapshot = useRecoilSnapshot();
    useEffect(() => {
        if (!isEnabled) return;
        console.debug("RecoilDebug: The following atoms were modified:");
        for (const node of snapshot.getNodes_UNSTABLE({ isModified: true })) {
            console.debug("RecoilDebug:", node.key, snapshot.getLoadable(node));
        }
    }, [snapshot]);

    return null;
}
