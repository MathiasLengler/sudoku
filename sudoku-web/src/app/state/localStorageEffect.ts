import { DefaultValue, type AtomEffect } from "recoil";
import { z } from "zod";

type SimpleAtomEffect<T> = (param: {
    node: { key: string };
    setSelf: (param: T) => void;

    // Subscribe callbacks to events.
    // Atom effect observers are called before global transaction observers
    onSet: (param: (newValue: T, oldValue: T | DefaultValue, isReset: boolean) => void) => void;
}) => void | (() => void);

// FIXME: is this useful?
//  effects can only read other atoms, not write to them
//  this limits what `innerToOuter` can de
export function transformEffect<TOuter, TInner>(
    innerEffect: SimpleAtomEffect<TInner>,
    outerToInner: (value: TOuter) => TInner,
    innerToOuter: (value: TInner, effectParams: Parameters<AtomEffect<TOuter>>[0]) => TOuter,
) {
    const outerEffect = ((params) => {
        const {
            onSet,
            setSelf,
            node: { key },
        } = params;
        innerEffect({
            node: { key },
            setSelf: (newValue) => {
                setSelf(innerToOuter(newValue, params));
            },
            onSet: (innerOnSet) => {
                onSet((newValue, _oldValue, isReset) => {
                    innerOnSet(outerToInner(newValue), new DefaultValue(), isReset);
                });
            },
        });
    }) satisfies AtomEffect<TOuter>;
    return outerEffect;
}

// TODO: switch to NPM lib `idb-keyval` for performance/more space
//  API is async, but recoil effects should support that.

export function localStorageEffect<Schema extends z.ZodTypeAny>(schema: Schema) {
    type SchemaType = z.infer<typeof schema>;
    const effect: SimpleAtomEffect<SchemaType> = ({ setSelf, onSet, node: { key: nodeKey } }) => {
        const key = `recoil_v1_${nodeKey}`;
        const savedValue = localStorage.getItem(key);
        if (savedValue != null) {
            try {
                setSelf(schema.parse(JSON.parse(savedValue)) as SchemaType);
            } catch (err) {
                console.error(`Failed to restore recoil atom ${nodeKey} from local storage key ${key}:`, err);
                localStorage.removeItem(key);
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
