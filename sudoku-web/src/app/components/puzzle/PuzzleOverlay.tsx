import CheckCircleIcon from "@mui/icons-material/CheckCircle";
import ErrorIcon from "@mui/icons-material/Error";
import ExtensionIcon from "@mui/icons-material/Extension";
import ReplayIcon from "@mui/icons-material/Replay";
import { Box, Button, Chip, Paper, Stack, Typography } from "@mui/material";
import { useAtomValue } from "jotai";
import { useState } from "react";
import type { StrategyEnum } from "../../../types";
import { useStartPuzzle, useExitPuzzleMode } from "../../actions/puzzleActions";
import { STRATEGY_OPTIONS } from "../../constants";
import {
    getStrategyStats,
    isPuzzleModeState,
    puzzleStatsState,
    puzzleStatusState,
    puzzleTargetStrategyState,
} from "../../state/puzzle";

function StrategyStatsChip({ strategy }: { strategy: StrategyEnum }) {
    const stats = useAtomValue(puzzleStatsState);
    const strategyStats = getStrategyStats(stats, strategy);

    return (
        <Chip
            size="small"
            label={`${strategyStats.solved}W / ${strategyStats.failed}L`}
            color="default"
            variant="outlined"
        />
    );
}

export function PuzzleOverlay() {
    const isPuzzleMode = useAtomValue(isPuzzleModeState);
    const targetStrategy = useAtomValue(puzzleTargetStrategyState);
    const status = useAtomValue(puzzleStatusState);
    const exitPuzzleMode = useExitPuzzleMode();
    const { startPuzzle } = useStartPuzzle();
    const [isGenerating, setIsGenerating] = useState(false);

    if (!isPuzzleMode || !targetStrategy || !status) {
        return null;
    }

    const strategyLabel = STRATEGY_OPTIONS[targetStrategy]?.label ?? targetStrategy;

    const handleContinue = async () => {
        setIsGenerating(true);
        try {
            await startPuzzle(targetStrategy);
        } catch (err) {
            console.error("Failed to generate new puzzle:", err);
        } finally {
            setIsGenerating(false);
        }
    };

    if (status === "active") {
        return (
            <Paper
                elevation={3}
                sx={{
                    position: "absolute",
                    top: 8,
                    left: "50%",
                    transform: "translateX(-50%)",
                    px: 2,
                    py: 1,
                    bgcolor: "info.light",
                    zIndex: 10,
                }}
            >
                <Stack direction="row" spacing={1} alignItems="center">
                    <ExtensionIcon color="info" fontSize="small" />
                    <Typography variant="body2" fontWeight="bold">
                        Find: {strategyLabel}
                    </Typography>
                    <StrategyStatsChip strategy={targetStrategy} />
                </Stack>
            </Paper>
        );
    }

    // Solved or Failed state
    const isSolved = status === "solved";

    return (
        <Box
            sx={{
                position: "absolute",
                top: 0,
                left: 0,
                right: 0,
                bottom: 0,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                bgcolor: "rgba(0, 0, 0, 0.5)",
                zIndex: 100,
            }}
        >
            <Paper
                elevation={6}
                sx={{
                    p: 4,
                    maxWidth: 400,
                    textAlign: "center",
                    bgcolor: isSolved ? "success.light" : "error.light",
                }}
            >
                <Stack spacing={2} alignItems="center">
                    {isSolved ? (
                        <CheckCircleIcon sx={{ fontSize: 64 }} color="success" />
                    ) : (
                        <ErrorIcon sx={{ fontSize: 64 }} color="error" />
                    )}

                    <Typography variant="h5" fontWeight="bold">
                        {isSolved ? "Correct!" : "Incorrect"}
                    </Typography>

                    <Typography variant="body1">
                        {isSolved
                            ? `You correctly identified the ${strategyLabel} technique.`
                            : `The deduction didn't match the ${strategyLabel} technique.`}
                    </Typography>

                    <StrategyStatsChip strategy={targetStrategy} />

                    <Stack direction="row" spacing={2}>
                        <Button
                            variant="contained"
                            color={isSolved ? "success" : "primary"}
                            startIcon={<ReplayIcon />}
                            onClick={handleContinue}
                            disabled={isGenerating}
                            loading={isGenerating}
                        >
                            {isGenerating ? "Generating..." : "Next Puzzle"}
                        </Button>
                        <Button variant="outlined" onClick={() => exitPuzzleMode()}>
                            Exit
                        </Button>
                    </Stack>
                </Stack>
            </Paper>
        </Box>
    );
}
