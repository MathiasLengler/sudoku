import LightbulbIcon from "@mui/icons-material/Lightbulb";
import MyIconButton from "../components/MyIconButton";
import * as React from "react";
import { useApplyDeductions, useTryStrategies } from "../sudokuActions";
import { useRecoilCallback, useRecoilValue } from "recoil";
import { hintSettingsState } from "../state/forms/hintSettings";
import { type Hint, hintState, type OptionalHint } from "../state/hint";
import assertNever from "assert-never/index";
import _ from "lodash";
import { sudokuIsSolvedState } from "../state/sudoku";

export function RequestHintButton() {
    const tryStrategies = useTryStrategies();
    const applyDeductions = useApplyDeductions();
    const isSolved = useRecoilValue(sudokuIsSolvedState);

    const hideHint = useRecoilCallback(({ set, reset }) => () => {
        reset(hintState);
    });

    const getHint = useRecoilCallback(
        ({ snapshot }) =>
            async (): Promise<OptionalHint> => {
                const hintSettings = await snapshot.getPromise(hintSettingsState);
                const tryStrategiesResult = await tryStrategies(hintSettings.strategies);
                if (!tryStrategiesResult) {
                    console.info("No strategy made progress.");
                    return;
                }
                const [strategy, { deductions }] = tryStrategiesResult;
                console.info(`Strategy ${strategy} made progress:`, deductions);

                if (hintSettings.multipleDeductions) {
                    return { strategy, deductions };
                } else {
                    const deduction = _.head(deductions);
                    if (!deduction) {
                        throw new Error(
                            `Expected at least one deduction from strategy ${strategy}, instead got: ${deductions}`
                        );
                    }
                    console.info("Selected deduction:", deduction);
                    return { strategy, deductions: [deduction] };
                }
            },
        [tryStrategies]
    );

    const showHint = useRecoilCallback(
        ({ set }) =>
            async (): Promise<boolean> => {
                const hint = await getHint();
                if (hint) {
                    set(hintState, hint);
                    return true;
                } else {
                    return false;
                }
            },
        [getHint]
    );

    const applyHint = useRecoilCallback(
        ({ snapshot }) =>
            async (hint: Hint) => {
                const { strategy, deductions } = hint;

                console.info(`Applying deductions from strategy ${strategy}:`, deductions);

                await applyDeductions({ deductions });

                hideHint();
            },
        [applyDeductions, hideHint]
    );

    const requestHint = useRecoilCallback(
        ({ snapshot }) =>
            async () => {
                // TODO: implement loop

                const {
                    //
                    strategies,
                    mode,
                    doLoop,
                    loopDelayMs,
                    multipleDeductions,
                } = await snapshot.getPromise(hintSettingsState);
                const hint = await snapshot.getPromise(hintState);

                if (mode === "toggleHint") {
                    // Ignore doLoop

                    if (hint) {
                        hideHint();
                    } else {
                        await showHint();
                    }
                } else if (mode === "hintApply") {
                    if (hint) {
                        await applyHint(hint);
                    } else {
                        await showHint();
                    }
                } else if (mode === "apply") {
                    if (hint) {
                        hideHint();
                    }
                    const newHint = await getHint();
                    if (newHint) {
                        await applyHint(newHint);
                    }
                } else {
                    assertNever(mode);
                }
            },
        [applyHint, getHint, hideHint, showHint]
    );

    return (
        <MyIconButton
            tooltip="Request Hint [TODO]"
            icon={LightbulbIcon}
            color={isSolved ? "success" : "default"}
            size="large"
            onClick={async () => {
                await requestHint();
            }}
        />
    );
}
