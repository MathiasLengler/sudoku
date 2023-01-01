import { atom, useRecoilValue, useSetRecoilState } from "recoil";
import * as React from "react";
import { Suspense, useEffect } from "react";

const noDefaultState = atom<string>({
    key: "NoDefault",
});
const ShowNoDefault = () => {
    const noDefault = useRecoilValue(noDefaultState);
    return <p>{noDefault}</p>;
};
export const NoDefaultSandbox = () => {
    const setNoDefault = useSetRecoilState(noDefaultState);

    useEffect(() => {
        const setDelayed = async () => {
            console.log("Waiting");
            await new Promise(resolve => setTimeout(resolve, 1000));
            console.log("Done waiting");
            setNoDefault("foo");
        };

        setDelayed().catch(console.error);
    }, [setNoDefault]);

    return (
        <>
            <h1>NoDefaultSandbox</h1>
            <Suspense fallback={"ShowNoDefault fallback"}>
                <ShowNoDefault />
            </Suspense>
        </>
    );
};
