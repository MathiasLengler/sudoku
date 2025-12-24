import LightbulbIcon from "@mui/icons-material/Lightbulb";
import CircularProgress from "@mui/material/CircularProgress";
import type { IconButtonProps } from "@mui/material/IconButton";
import assertNever from "assert-never";
import * as _ from "es-toolkit";
import { useNotifications } from "@toolpad/core/useNotifications";
import { useCallback, useState } from "react";
import { useAtomValue } from "jotai";
import { useApplyDeductions, useTryStrategies } from "../actions/sudokuActions";
import MyIconButton from "../components/MyIconButton";
import { hintSettingsState, scaleLoopDelayIndex } from "../state/forms/hintSettings";
import { hintState, type Hint, type OptionalHint } from "../state/hint";
import { sudokuIsSolvedState } from "../state/sudoku";
import { RESET, useAtomCallback } from "jotai/utils";

export function RequestHintButton() {
    const [isRequestingHint, setIsRequestingHint] = useState(false);
    const [requestHintAbortController, setRequestHintAbortController] = useState(() => new AbortController());

    const tryStrategies = useTryStrategies();
    const applyDeductions = useApplyDeductions();
    const sudokuIsSolved = useAtomValue(sudokuIsSolvedState);

    const notifications = useNotifications();

    const hideHint = useAtomCallback(
        useCallback((_get, set) => {
            set(hintState, RESET);
        }, []),
    );

    const getHint = useAtomCallback(
        useCallback(
            async (get): Promise<OptionalHint> => {
                const sudokuIsSolved = await get(sudokuIsSolvedState);
                if (sudokuIsSolved) {
                    notifications.show("Sudoku is solved", {
                        key: "solved",
                        severity: "success",
                    });
                    return;
                }

                const hintSettings = get(hintSettingsState);
                let tryStrategiesResult;
                try {
                    tryStrategiesResult = await tryStrategies(hintSettings.strategies);
                } catch (err) {
                    if (!(err instanceof Error)) throw err;
                    console.error("Failed to execute strategies", hintSettings.strategies, ":", err);
                    notifications.show(err.message, { severity: "error" });
                    return;
                }
                if (!tryStrategiesResult) {
                    notifications.show("No strategy made progress", {
                        key: "no-progress",
                        severity: "warning",
                    });
                    return;
                }
                const {
                    strategy,
                    deductions: { deductions },
                } = tryStrategiesResult;
                console.info(`Strategy ${strategy} made progress:`, deductions);

                if (hintSettings.multipleDeductions) {
                    return { strategy, deductions };
                } else {
                    const deduction = _.head(deductions);
                    if (!deduction) {
                        throw new Error(
                            `Expected at least one deduction from strategy ${strategy}, instead got: "${JSON.stringify(deductions)}"`,
                        );
                    }
                    console.info("Selected deduction:", deduction);
                    return { strategy, deductions: [deduction] };
                }
            },
            [notifications, tryStrategies],
        ),
    );

    const showHint = useAtomCallback(
        useCallback(
            async (_get, set): Promise<boolean> => {
                const hint = await getHint();
                if (hint) {
                    set(hintState, hint);
                    return true;
                } else {
                    return false;
                }
            },
            [getHint],
        ),
    );

    const applyHint = useCallback(
        async (hint: Hint): Promise<boolean> => {
            const { strategy, deductions } = hint;

            console.info(`Applying deductions from strategy ${strategy}:`, deductions);

            let madeProgress = true;
            try {
                await applyDeductions({ deductions });
            } catch (err) {
                if (!(err instanceof Error)) throw err;
                console.error("Failed to apply deductions", deductions, ":", err);
                notifications.show(`Failed to apply hint: ${err.message}`, { severity: "error" });
                madeProgress = false;
            }

            hideHint();
            return madeProgress;
        },
        [applyDeductions, notifications, hideHint],
    );

    const requestSingleHint = useAtomCallback(
        useCallback(
            async (get): Promise<boolean> => {
                const { mode } = get(hintSettingsState);
                const hint = get(hintState);

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
            [applyHint, getHint, hideHint, showHint],
        ),
    );

    const requestHint = useAtomCallback(
        useCallback(
            async (get) => {
                if (isRequestingHint) {
                    console.warn("Unexpected concurrent call to requestHint");
                    return;
                }

                setIsRequestingHint(true);

                try {
                    const { mode, doLoop, loopDelayIndex } = get(hintSettingsState);

                    if (doLoop && mode !== "toggleHint") {
                        while (await requestSingleHint()) {
                            if (loopDelayIndex) {
                                const loopDelayMs = scaleLoopDelayIndex(loopDelayIndex);
                                console.info("Sleeping for", loopDelayMs);
                                await new Promise((resolve) => setTimeout(resolve, loopDelayMs));

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
            [isRequestingHint, requestHintAbortController.signal, requestSingleHint],
        ),
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
