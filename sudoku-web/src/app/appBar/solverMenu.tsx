import { useTryStrategies } from "../sudokuActions";
import { useRecoilValue } from "recoil";
import { sudokuIsSolvedState } from "../state/sudoku";
import { CustomMenu } from "./customMenu";
import { ALL_STRATEGIES } from "../../constants";
import type { DynamicStrategy } from "../../../../sudoku-rs/bindings";
import { IconButton } from "@mui/material";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import React from "react";
import _ from "lodash";

const STRATEGIES_PYRAMID = _.initial(ALL_STRATEGIES).map((strategy, i) => ({
    untilStrategy: strategy,
    strategies: _.take(ALL_STRATEGIES, i + 1),
}));

export function SolverMenu() {
    const tryStrategies = useTryStrategies();
    const isSolved = useRecoilValue(sudokuIsSolvedState);

    const tryStrategiesInLoop = async (strategies: DynamicStrategy[]) => {
        while (true) {
            const tryStrategiesResult = await tryStrategies(strategies);
            if (!tryStrategiesResult) {
                break;
            }
            const [strategy, deductions] = tryStrategiesResult;
            console.info(`Made progress with strategy ${strategy}:`, deductions);
        }
        // TODO: show in Snackbar
        console.info("All strategies failed to make progress");
    };

    return (
        <CustomMenu
            menuItems={[
                ...ALL_STRATEGIES.map(strategy => ({
                    label: strategy as string,
                    onClick: async () => {
                        await tryStrategies([strategy]);
                    },
                })),
                ...STRATEGIES_PYRAMID.map(({ untilStrategy, strategies }) => ({
                    label: `Loop: ${untilStrategy}`,
                    onClick: async () => {
                        await tryStrategiesInLoop(strategies);
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
