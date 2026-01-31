import { Box, Group, Title } from "@mantine/core";
import { Suspense } from "react";
import { useAtomValue } from "jotai";
import { ThemeErrorBoundary } from "../components/ErrorFallback";
import { FullScreenSpinner } from "../components/FullScreenSpinner";
import { featureFlagsState } from "../state/featureFlags";
import { HintSettingsButton } from "./hintSettings/HintSettingsButton";
import { NewGameButton } from "./newGame/NewGameButton";
import { ShareMenu } from "./share/ShareMenu";
import { WorldSettingsButton } from "./world/WorldSettingsButton";

export default function SudokuAppBar() {
    const { experimentWorld } = useAtomValue(featureFlagsState);

    return (
        <Box className="app-bar" p="xs" style={{ borderBottom: "1px solid var(--mantine-color-default-border)" }}>
            <Group justify="space-between" align="center">
                <ThemeErrorBoundary inline>
                    <Title order={4}>Sudoku</Title>
                    <Group gap="xs">
                        <Suspense fallback={<FullScreenSpinner />}>
                            {experimentWorld && <WorldSettingsButton />}
                            <ShareMenu />
                            <HintSettingsButton />
                            <NewGameButton />
                        </Suspense>
                    </Group>
                </ThemeErrorBoundary>
            </Group>
        </Box>
    );
}
