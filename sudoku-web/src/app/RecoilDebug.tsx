import { useEffect } from "react";
import { useRecoilSnapshot } from "recoil";

const isEnabled = true;

export function RecoilDebug() {
    const snapshot = useRecoilSnapshot();
    useEffect(() => {
        if (!isEnabled) return;
        console.debug("RecoilDebug: new snapshot, modified atoms:");
        for (const node of snapshot.getNodes_UNSTABLE({ isModified: true })) {
            console.debug("RecoilDebug:", node.key, snapshot.getLoadable(node).contents);
        }
    }, [snapshot]);

    return null;
}
