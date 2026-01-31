import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import {
    Box,
    Button,
    DialogActions,
    DialogContent,
    DialogTitle,
    FormControl,
    InputLabel,
    LinearProgress,
    MenuItem,
    Select,
    Stack,
    Typography,
} from "@mui/material";
import { useAtomValue } from "jotai";
import { useEffect, useState } from "react";
import type { StrategyEnum } from "../../../types";
import { useStartPuzzle, useExitPuzzleMode } from "../../actions/puzzleActions";
import { STRATEGY_OPTIONS } from "../../constants";
import {
    getStrategyStats,
    isPuzzleModeState,
    PUZZLE_STRATEGIES,
    puzzleStatsState,
    puzzleStatusState,
    puzzleTargetStrategyState,
} from "../../state/puzzle";

type PuzzleDialogProps = {
    onClose: () => void;
};

function StrategyStatsDisplay({ strategy }: { strategy: StrategyEnum }) {
    const stats = useAtomValue(puzzleStatsState);
    const strategyStats = getStrategyStats(stats, strategy);
    const total = strategyStats.solved + strategyStats.failed;
    const winRate = total > 0 ? Math.round((strategyStats.solved / total) * 100) : 0;

    return (
        <Typography variant="body2" color="text.secondary">
            Solved: {strategyStats.solved} | Failed: {strategyStats.failed}
            {total > 0 && ` | Win rate: ${winRate}%`}
        </Typography>
    );
}

function ActivePuzzleStatus() {
    const targetStrategy = useAtomValue(puzzleTargetStrategyState);
    const status = useAtomValue(puzzleStatusState);
    const exitPuzzleMode = useExitPuzzleMode();

    if (!targetStrategy || !status) {
        return null;
    }

    const strategyLabel = STRATEGY_OPTIONS[targetStrategy]?.label ?? targetStrategy;

    let statusMessage: string;
    let statusColor: "info" | "success" | "error";

    switch (status) {
        case "active":
            statusMessage = `Find and apply the ${strategyLabel} technique`;
            statusColor = "info";
            break;
        case "solved":
            statusMessage = "Correct! You found the right deduction.";
            statusColor = "success";
            break;
        case "failed":
            statusMessage = "Incorrect. The deduction didn't match.";
            statusColor = "error";
            break;
    }

    return (
        <Box sx={{ mb: 2, p: 2, bgcolor: `${statusColor}.light`, borderRadius: 1 }}>
            <Typography variant="h6" gutterBottom>
                Current Puzzle: {strategyLabel}
            </Typography>
            <Typography variant="body1" color={`${statusColor}.dark`}>
                {statusMessage}
            </Typography>
            <StrategyStatsDisplay strategy={targetStrategy} />
            {status !== "active" && (
                <Button variant="outlined" onClick={() => exitPuzzleMode()} sx={{ mt: 1 }}>
                    Exit Puzzle Mode
                </Button>
            )}
        </Box>
    );
}

export function PuzzleDialog({ onClose }: PuzzleDialogProps) {
    const [selectedStrategy, setSelectedStrategy] = useState<StrategyEnum>("HiddenSingles");
    const isPuzzleMode = useAtomValue(isPuzzleModeState);
    const { startPuzzle, generateProgress, cancelGenerate } = useStartPuzzle();
    const [isGenerating, setIsGenerating] = useState(false);

    // Cancel generation on unmount
    useEffect(() => {
        return () => {
            cancelGenerate();
        };
    }, [cancelGenerate]);

    const handleStartPuzzle = async () => {
        setIsGenerating(true);
        try {
            await startPuzzle(selectedStrategy);
            onClose();
        } catch (err) {
            if (!(err instanceof DOMException && err.name === "AbortError")) {
                console.error("Failed to start puzzle:", err);
            }
        } finally {
            setIsGenerating(false);
        }
    };

    return (
        <>
            <DialogTitle>Challenge Mode</DialogTitle>
            <DialogContent>
                <Stack spacing={3} sx={{ pt: 1 }}>
                    <Typography variant="body1">
                        Test your puzzle-solving skills! Select a strategy and we&apos;ll generate a puzzle that
                        requires that specific technique. Your goal is to spot and apply the correct deduction.
                    </Typography>

                    {isPuzzleMode && <ActivePuzzleStatus />}

                    <FormControl fullWidth>
                        <InputLabel id="strategy-select-label">Strategy to Practice</InputLabel>
                        <Select
                            labelId="strategy-select-label"
                            value={selectedStrategy}
                            label="Strategy to Practice"
                            onChange={(e) => setSelectedStrategy(e.target.value as StrategyEnum)}
                            disabled={isGenerating}
                        >
                            {PUZZLE_STRATEGIES.map((strategy) => (
                                <MenuItem key={strategy} value={strategy}>
                                    <Box>
                                        <Typography variant="body1">
                                            {STRATEGY_OPTIONS[strategy]?.label ?? strategy}
                                        </Typography>
                                        <Typography variant="caption" color="text.secondary">
                                            {STRATEGY_OPTIONS[strategy]?.description}
                                        </Typography>
                                    </Box>
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>

                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Your stats for {STRATEGY_OPTIONS[selectedStrategy]?.label ?? selectedStrategy}:
                        </Typography>
                        <StrategyStatsDisplay strategy={selectedStrategy} />
                    </Box>

                    {isGenerating && generateProgress && (
                        <Box>
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                                Generating puzzle...
                            </Typography>
                            <LinearProgress
                                variant="determinate"
                                value={
                                    (generateProgress.pruningPositionIndex / generateProgress.pruningPositionCount) *
                                    100
                                }
                            />
                        </Box>
                    )}
                    {isGenerating && !generateProgress && (
                        <Box>
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                                Generating solution...
                            </Typography>
                            <LinearProgress />
                        </Box>
                    )}
                </Stack>
            </DialogContent>
            <DialogActions>
                <Button
                    onClick={() => {
                        if (isGenerating) {
                            cancelGenerate();
                        } else {
                            onClose();
                        }
                    }}
                >
                    {isGenerating ? "Cancel" : "Close"}
                </Button>
                <Button
                    variant="contained"
                    color="primary"
                    endIcon={<PlayArrowIcon />}
                    onClick={handleStartPuzzle}
                    disabled={isGenerating}
                    loading={isGenerating}
                    loadingPosition="end"
                >
                    <span>Start Puzzle</span>
                </Button>
            </DialogActions>
        </>
    );
}
