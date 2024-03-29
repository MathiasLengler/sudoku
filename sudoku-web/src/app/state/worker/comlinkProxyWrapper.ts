import * as Comlink from "comlink";

export function fixupComlinkProxy<T>(comlinkProxy: Comlink.Remote<T>): Comlink.Remote<T> {
    return new Proxy(
        // Target a plain object for `typeof proxy === "object"`
        // Reference: https://stackoverflow.com/a/42493645
        {},
        {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            get: (_target, property: string): any => {
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
                return (comlinkProxy as unknown as Record<string, unknown>)[property];
            },
        },
    ) as unknown as Comlink.Remote<T>;
}
