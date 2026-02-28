import TabContext from "@mui/lab/TabContext";
import TabList from "@mui/lab/TabList";
import { Box, Paper, Typography } from "@mui/material";
import Tab from "@mui/material/Tab";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { GenerateFormPage } from "./-components/GenerateFormPage";
import { ImportFormPage } from "./-components/ImportFormPage";

export const Route = createFileRoute("/new-game/")({
    component: NewGamePage,
});

export type NewGameTabValue = "generate" | "import";

function NewGamePage() {
    const navigate = useNavigate();
    const [tabValue, setTabValue] = useState<NewGameTabValue>("generate");

    const onClose = () => {
        void navigate({ to: "/" });
    };

    return (
        <Box
            sx={{
                display: "flex",
                flexDirection: "column",
                alignItems: "center",
                justifyContent: "flex-start",
                p: 2,
                height: "100%",
                overflow: "auto",
            }}
        >
            <Paper
                sx={{
                    width: "100%",
                    maxWidth: 600,
                    p: 2,
                }}
                elevation={2}
            >
                <Typography variant="h5" sx={{ mb: 2, textAlign: "center" }}>
                    New Game
                </Typography>
                <TabContext value={tabValue}>
                    <TabList
                        onChange={(_ev, newTabIndex: NewGameTabValue) => {
                            setTabValue(newTabIndex);
                        }}
                        aria-label="New game forms"
                        variant="fullWidth"
                    >
                        <Tab label="Generate" value="generate" />
                        <Tab label="Import" value="import" />
                    </TabList>
                    {tabValue === "generate" ? (
                        <GenerateFormPage onClose={onClose} />
                    ) : (
                        <ImportFormPage onClose={onClose} />
                    )}
                </TabContext>
            </Paper>
        </Box>
    );
}
