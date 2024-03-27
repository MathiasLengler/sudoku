// Reference: https://github.com/facebookexperimental/Recoil/issues/629#issuecomment-914273925
export type CreateSerializableParam<Type> = {
    [Property in keyof Type]: Type[Property] extends import("recoil").SerializableParam
        ? Type[Property]
        : // eslint-disable-next-line @typescript-eslint/no-explicit-any
          Type[Property] extends Record<string, any> | undefined | null
          ? CreateSerializableParam<Type[Property]>
          : Type[Property];
};

// eslint-disable-next-line @typescript-eslint/no-empty-function, @typescript-eslint/no-unused-vars
export function assert<T extends true>() {}
