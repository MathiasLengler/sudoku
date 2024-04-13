import { atom, selector } from "recoil";

export type FeatureFlags = {
    experimentWorld: boolean;
};

export const featureFlagsState = selector<FeatureFlags>({
    key: "FeatureFlags",
    get: () => {
        const params = new URLSearchParams(window.location.search);
        return {
            experimentWorld: params.has("world"),
        };
    },
});
