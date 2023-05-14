import type { AtomEffect } from "recoil";
import { z } from "zod";

export function localStorageEffect<Schema extends z.ZodTypeAny>(schema: Schema) {
    type SchemaType = z.infer<typeof schema>;
    const effect: AtomEffect<SchemaType> = ({ setSelf, onSet, node: { key: nodeKey } }) => {
        const key = `recoil_v1_${nodeKey}`;
        const savedValue = localStorage.getItem(key);
        if (savedValue != null) {
            try {
                setSelf(schema.parse(JSON.parse(savedValue)));
            } catch (err) {
                console.error(`Failed to restore recoil atom ${nodeKey} from local storage key ${key}:`, err);
            }
        }

        onSet((newValue, _, isReset) => {
            if (isReset) {
                localStorage.removeItem(key);
            } else {
                localStorage.setItem(key, JSON.stringify(newValue));
            }
        });
    };
    return effect;
}
