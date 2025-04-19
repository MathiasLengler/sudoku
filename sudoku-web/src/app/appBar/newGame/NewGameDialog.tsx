import React from "react";
import { GenerateForm } from "./GenerateForm";
import { DialogTitle } from "@mui/material";
import Tab from "@mui/material/Tab";
import TabContext from "@mui/lab/TabContext";
import TabList from "@mui/lab/TabList";
import { ImportForm } from "./ImportForm";

type NewGameTabsProps = {
    onClose: () => void;
};

export type NewGameTabValue = "generate-form" | "import-form";
export const NewGameDialog = ({ onClose }: NewGameTabsProps) => {
    const [tabValue, setTabValue] = React.useState<NewGameTabValue>("generate-form");

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
};
