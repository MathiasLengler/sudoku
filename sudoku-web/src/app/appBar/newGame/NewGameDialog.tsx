import TabContext from "@mui/lab/TabContext";
import TabList from "@mui/lab/TabList";
import { DialogTitle } from "@mui/material";
import Tab from "@mui/material/Tab";
import { useState } from "react";
import { GenerateForm } from "./GenerateForm";
import { ImportForm } from "./ImportForm";

type NewGameTabsProps = {
    onClose: () => void;
};

export type NewGameTabValue = "generate-form" | "import-form";
export function NewGameDialog({ onClose }: NewGameTabsProps) {
    const [tabValue, setTabValue] = useState<NewGameTabValue>("generate-form");

    return (
        <TabContext value={tabValue}>
            <DialogTitle>New game</DialogTitle>
            <TabList
                onChange={(_ev, newTabIndex: NewGameTabValue) => {
                    setTabValue(newTabIndex);
                }}
                aria-label="New game forms"
                variant="fullWidth"
            >
                <Tab label="Generate" value="generate-form" />
                <Tab label="Import" value="import-form" />
            </TabList>
            {tabValue === "generate-form" ? <GenerateForm onClose={onClose} /> : <ImportForm onClose={onClose} />}
        </TabContext>
    );
}
