import * as Comlink from "comlink";

export type SaveComlinkRemote<T> = Comlink.Remote<T> & {
    [saveComlinkRemoteSymbol]: true;
};

const saveComlinkRemoteSymbol = Symbol("SaveComlinkProxy");

export function fixupComlinkRemote<T>(comlinkRemote: Comlink.Remote<T>): SaveComlinkRemote<T> {
    return new Proxy(
        // Target a plain object for `typeof proxy === "object"`
        // Reference: https://stackoverflow.com/a/42493645
        { [saveComlinkRemoteSymbol]: true },
        {
            get: (_target, property: string | symbol): unknown => {
                if (property === saveComlinkRemoteSymbol) {
                    return _target[saveComlinkRemoteSymbol];
                }
                // Not a thenable
                // Reference: https://stackoverflow.com/a/53890904
                if (property === "then") {
                    return undefined;
                }
                // Fix error when passing proxy to `JSON.stringify`
                if (property === "toJSON") {
                    return () => {
                        console.warn("JSON.stringify called on Comlink proxy");
                        return { COMLINK_PROXY_PLACEHOLDER: true };
                    };
                }
                return (comlinkRemote as Record<string | symbol, unknown>)[property];
            },
        },
    ) as SaveComlinkRemote<T>;
}
