import type { SyncStorage } from "jotai/vanilla/utils/atomWithStorage";
import * as z from "zod";

// https://zod.dev/codecs?id=jsonschema
const jsonCodec = <T extends z.core.$ZodType>(schema: T) =>
    z.codec(z.string(), schema, {
        decode: (jsonString, ctx) => {
            try {
                // eslint-disable-next-line @typescript-eslint/no-unsafe-return
                return JSON.parse(jsonString);
            } catch (err: unknown) {
                ctx.issues.push({
                    code: "invalid_format",
                    format: "json",
                    input: jsonString,
                    message: err instanceof Error ? err.message : "Unknown error parsing JSON",
                });
                return z.NEVER;
            }
        },
        encode: (value) => JSON.stringify(value),
    });

export function getZodLocalStorage<Schema extends z.ZodTypeAny>(schema: Schema): SyncStorage<z.output<typeof schema>> {
    const jsonSchema = jsonCodec(schema);

    return {
        getItem(key, initialValue) {
            const storedValue = localStorage.getItem(key);
            try {
                return jsonSchema.decode(storedValue ?? "");
            } catch {
                return initialValue;
            }
        },
        setItem(key, value) {
            localStorage.setItem(key, jsonSchema.encode(value));
        },
        removeItem(key) {
            localStorage.removeItem(key);
        },
        subscribe(key, callback, initialValue) {
            const handler = (e: StorageEvent) => {
                if (e.storageArea === localStorage && e.key === key) {
                    let newValue;
                    try {
                        newValue = jsonSchema.decode(e.newValue ?? "");
                    } catch {
                        newValue = initialValue;
                    }
                    callback(newValue);
                }
            };
            window.addEventListener("storage", handler);
            return () => window.removeEventListener("storage", handler);
        },
    };
}
