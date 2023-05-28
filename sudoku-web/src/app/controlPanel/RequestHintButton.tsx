import LightbulbIcon from "@mui/icons-material/Lightbulb";
import MyIconButton from "../components/MyIconButton";
import * as React from "react";
import { useApplyDeductions, useTryStrategies } from "../actions/sudokuActions";
import { useRecoilCallback, useRecoilValue } from "recoil";
import { hintSettingsState, scaleLoopDelayIndex } from "../state/forms/hintSettings";
import { type Hint, hintState, type OptionalHint } from "../state/hint";
import assertNever from "assert-never/index";
import _ from "lodash";
import { sudokuIsSolvedState } from "../state/sudoku";
import { useSnackbar } from "notistack";
import { useState } from "react";
import type { IconButtonProps } from "@mui/material/IconButton/IconButton";
import CircularProgress from "@mui/material/CircularProgress";

export function RequestHintButton() {
    const [isRequestingHint, setIsRequestingHint] = useState(false);
    const [requestHintAbortController, setRequestHintAbortController] = useState(() => new AbortController());

    const tryStrategies = useTryStrategies();
    const applyDeductions = useApplyDeductions();
    const sudokuIsSolved = useRecoilValue(sudokuIsSolvedState);

    const { enqueueSnackbar } = useSnackbar();

    const hideHint = useRecoilCallback(({ reset }) => () => {
        reset(hintState);
    });

    const getHint = useRecoilCallback(
        ({ snapshot }) =>
            async (): Promise<OptionalHint> => {
                const sudokuIsSolved = await snapshot.getPromise(sudokuIsSolvedState);
                if (sudokuIsSolved) {
                    enqueueSnackbar({ variant: "success", message: "Sudoku solved" });
                    return;
                }

                const hintSettings = await snapshot.getPromise(hintSettingsState);
                let tryStrategiesResult;
                try {
                    tryStrategiesResult = await tryStrategies(hintSettings.strategies);
                } catch (err) {
                    if (!(err instanceof Error)) throw err;
                    console.error("Failed to execute strategies", hintSettings.strategies, ":", err);
                    enqueueSnackbar({ variant: "error", message: err.message });
                }
                if (!tryStrategiesResult) {
                    enqueueSnackbar({ variant: "warning", message: "No strategy made progress" });
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
        [enqueueSnackbar, tryStrategies]
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
        () =>
            async (hint: Hint): Promise<boolean> => {
                const { strategy, deductions } = hint;

                console.info(`Applying deductions from strategy ${strategy}:`, deductions);

                let madeProgress = true;
                try {
                    await applyDeductions({ deductions });
                } catch (err) {
                    if (!(err instanceof Error)) throw err;
                    console.error("Failed to apply deductions", deductions, ":", err);
                    enqueueSnackbar({ variant: "error", message: `Failed to apply hint: ${err.message}` });
                    madeProgress = false;
                }

                hideHint();
                return madeProgress;
            },
        [applyDeductions, enqueueSnackbar, hideHint]
    );

    const requestSingleHint = useRecoilCallback(
        ({ snapshot }) =>
            async (): Promise<boolean> => {
                const { mode } = await snapshot.getPromise(hintSettingsState);
                const hint = await snapshot.getPromise(hintState);

                if (mode === "toggleHint") {
                    if (hint) {
                        hideHint();
                    } else {
                        await showHint();
                    }
                    return false;
                }
                if (mode === "hintApply") {
                    if (hint) {
                        return await applyHint(hint);
                    } else {
                        return await showHint();
                    }
                }
                if (mode === "apply") {
                    if (hint) {
                        hideHint();
                    }
                    const newHint = await getHint();
                    if (newHint) {
                        return await applyHint(newHint);
                    } else {
                        return false;
                    }
                }
                assertNever(mode);
            },
        [applyHint, getHint, hideHint, showHint]
    );

    const requestHint = useRecoilCallback(
        ({ snapshot }) =>
            async () => {
                if (isRequestingHint) {
                    console.warn("Unexpected concurrent call to requestHint");
                    return;
                }

                setIsRequestingHint(true);

                try {
                    const { mode, doLoop, loopDelayIndex } = await snapshot.getPromise(hintSettingsState);

                    if (doLoop && mode !== "toggleHint") {
                        while (await requestSingleHint()) {
                            if (loopDelayIndex) {
                                const loopDelayMs = scaleLoopDelayIndex(loopDelayIndex);
                                console.info("Sleeping for", loopDelayMs);
                                await new Promise(resolve => setTimeout(resolve, loopDelayMs));

                                if (requestHintAbortController.signal.aborted) {
                                    console.info("requestHint aborted");
                                    setRequestHintAbortController(new AbortController());
                                    return;
                                }
                            }
                        }
                    } else {
                        await requestSingleHint();
                    }
                } finally {
                    setIsRequestingHint(false);
                }
            },
        [isRequestingHint, requestHintAbortController.signal, requestSingleHint]
    );

    let iconColor: IconButtonProps["color"];
    if (isRequestingHint) {
        iconColor = "warning";
    } else if (sudokuIsSolved) {
        iconColor = "success";
    } else {
        iconColor = "default";
    }
    return (
        <MyIconButton
            label="Request Hint [_]"
            icon={LightbulbIcon}
            color={iconColor}
            size="large"
            badge={isRequestingHint ? <CircularProgress size="1rem" color="warning" /> : null}
            onClick={async () => {
                if (isRequestingHint) {
                    requestHintAbortController.abort();
                } else {
                    await requestHint();
                }
            }}
        />
    );
}
