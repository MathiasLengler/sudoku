import { atom } from "jotai";

export type FeatureFlags = {
    experimentWorld: boolean;
};

export const featureFlagsState = atom<FeatureFlags>(() => {
    const params = new URLSearchParams(window.location.search);
    return {
        experimentWorld: params.has("world"),
    };
});
