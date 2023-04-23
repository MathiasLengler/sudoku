import { useApplyDeductions, useTryStrategies } from "../sudokuActions";
import { useRecoilValue, useSetRecoilState } from "recoil";
import { sudokuIsSolvedState } from "../state/sudoku";
import { CustomMenu } from "../components/CustomMenu";
import { ALL_STRATEGIES } from "../../constants";
import type { DynamicStrategy } from "../../../../sudoku-rs/bindings";
import { IconButton } from "@mui/material";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import React from "react";
import _ from "lodash";
import { solverHintState } from "../state/deductions";

const STRATEGIES_PYRAMID = _.initial(ALL_STRATEGIES).map((strategy, i) => ({
    untilStrategy: strategy,
    strategies: _.take(ALL_STRATEGIES, i + 1),
}));

export function SolverMenu() {
    const tryStrategies = useTryStrategies();
    const applyDeductions = useApplyDeductions();
    const isSolved = useRecoilValue(sudokuIsSolvedState);

    // TODO: implement reset logic
    const setSolverHint = useSetRecoilState(solverHintState);

    const tryStrategiesInLoop = async (strategies: DynamicStrategy[]) => {
        while (true) {
            const tryStrategiesResult = await tryStrategies(strategies);
            if (!tryStrategiesResult) {
                break;
            }
            const [strategy, deductions] = tryStrategiesResult;
            console.info(`Strategy ${strategy} made progress:`, deductions);

            await applyDeductions(deductions);
        }
        // TODO: show in Snackbar
        console.info("All strategies failed to make progress");
    };

    const hintStrategies = async (strategies: DynamicStrategy[]) => {
        const tryStrategiesResult = await tryStrategies(strategies);
        if (!tryStrategiesResult) {
            setSolverHint({ enabled: false });
            return;
        }
        const [strategy, deductions] = tryStrategiesResult;
        console.info(`Strategy ${strategy} made progress:`, deductions);

        setSolverHint({ enabled: true, strategy, ...deductions });
    };

    return (
        <CustomMenu
            menuItems={[
                ...ALL_STRATEGIES.map(strategy => ({
                    label: strategy as string,
                    onClick: async () => {
                        const tryStrategiesResult = await tryStrategies([strategy]);
                        if (!tryStrategiesResult) {
                            return;
                        }
                        const [, deductions] = tryStrategiesResult;
                        console.info(`Strategy ${strategy} made progress:`, deductions);

                        await applyDeductions(deductions);
                    },
                })),
                ...STRATEGIES_PYRAMID.map(({ untilStrategy, strategies }) => ({
                    label: `Loop: ${untilStrategy}`,
                    onClick: async () => {
                        await tryStrategiesInLoop(strategies);
                    },
                })),
                ...STRATEGIES_PYRAMID.map(({ untilStrategy, strategies }) => ({
                    label: `Hint (max): ${untilStrategy}`,
                    onClick: async () => {
                        await hintStrategies(strategies);
                    },
                })),
            ]}
        >
            {({ onMenuOpen }) => (
                <IconButton
                    color={isSolved ? "success" : "inherit"}
                    size="large"
                    aria-label="Solver"
                    onClick={onMenuOpen}
                >
                    <LightbulbIcon fontSize="large" />
                </IconButton>
            )}
        </CustomMenu>
    );
}
