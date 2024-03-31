import { atom, selector } from "recoil";

export type FeatureFlags = {
    experimentWorld: boolean;
};

export const featureFlagsState = atom<FeatureFlags>({
    key: "FeatureFlags",
    default: selector({
        key: "DefaultFeatureFlags",
        get: () => {
            const params = new URLSearchParams(window.location.search);
            return {
                experimentWorld: params.has("world"),
            };
        },
    }),
});
