import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { HintSettingsButton } from "./hintSettings/HintSettingsButton";
import { NewGameButton } from "./newGame/NewGameButton";
import { ShareMenu } from "./share/ShareMenu";
import { WorldSettingsButton } from "./world/WorldSettingsButton";
import { ThemeErrorBoundary } from "../components/ErrorFallback";
import { FullScreenSpinner } from "../components/FullScreenSpinner";
import { Suspense } from "react";
import { useRecoilValue } from "recoil";
import { featureFlagsState } from "../state/featureFlags";

export default function SudokuAppBar() {
    const { experimentWorld } = useRecoilValue(featureFlagsState);

    return (
        <Box sx={{ flexGrow: 1 }} className="app-bar">
            <AppBar position="static" variant="outlined" color="default" elevation={0}>
                <Toolbar>
                    <ThemeErrorBoundary inline>
                        <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                            Sudoku
                        </Typography>
                        <Suspense fallback={<FullScreenSpinner />}>
                            {experimentWorld && <WorldSettingsButton />}
                            <ShareMenu />
                            <HintSettingsButton />
                            <NewGameButton />
                        </Suspense>
                    </ThemeErrorBoundary>
                </Toolbar>
            </AppBar>
        </Box>
    );
}
