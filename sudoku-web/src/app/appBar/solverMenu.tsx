import { useTryStrategy } from "../sudokuActions";
import { useRecoilValue } from "recoil";
import { sudokuIsSolvedState } from "../state/sudoku";
import { CustomMenu } from "./customMenu";
import { ALL_STRATEGIES } from "../../constants";
import type { DynamicStrategy } from "../../../../sudoku-rs/bindings";
import { IconButton } from "@mui/material";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import React from "react";

export function SolverMenu() {
    const tryStrategy = useTryStrategy();
    const isSolved = useRecoilValue(sudokuIsSolvedState);

    return (
        <CustomMenu
            menuItems={[
                ...ALL_STRATEGIES.map(strategy => ({
                    label: strategy as string,
                    onClick: async () => {
                        await tryStrategy(strategy);
                    },
                })),
                {
                    label: "Debug Iterative",
                    onClick: async () => {
                        const strategies: DynamicStrategy[] = [
                            "SingleCandidate",
                            // "HiddenSingles",
                            // "GroupReduction",
                            // "Backtracking",
                        ];
                        outer: while (true) {
                            for (const strategy of strategies) {
                                console.info("Trying strategy:", strategy);
                                if (await tryStrategy(strategy)) {
                                    console.info("Made progress with:", strategy);
                                    continue outer;
                                }
                            }
                            break;
                        }
                        // TODO: show in Snackbar
                        console.info("All strategies failed to make progress");
                    },
                },
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
