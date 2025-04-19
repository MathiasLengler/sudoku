export type MyMeasurement = {
    name: string;
    detail?: object;
};

export async function measure<R>({ name, detail }: MyMeasurement, f: () => Promise<R>): Promise<R> {
    performance.mark(`${name}-start`);
    let isError = false;
    try {
        return await f();
    } catch (error) {
        isError = true;
        throw error;
    } finally {
        performance.measure(name, { start: `${name}-start`, detail: { ...detail, isError } });
    }
}
export function withMeasure<Args extends unknown[], R>(
    { name, detail }: MyMeasurement,
    f: (...args: Args) => Promise<R>,
): (...args: Args) => Promise<R> {
    return (...args: Args): Promise<R> => {
        return measure({ name, detail }, () => f(...args));
    };
}
