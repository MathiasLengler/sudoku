import React from "react";
import AppBar from "@mui/material/AppBar";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import { GenerateForm } from "./generateForm";
import { WasmSudokuController } from "../../wasmSudokuController";
import { ImportForm } from "./importForm";

interface TabPanelProps {
    children: React.ReactNode;
    index: number;
    tabIndex: number;
}

function TabPanel(props: TabPanelProps) {
    const { tabIndex, index, children } = props;

    return (
        <div role="tabpanel" hidden={tabIndex !== index}>
            {tabIndex === index && children}
        </div>
    );
}

interface NewGameTabsProps {
    sudokuController: WasmSudokuController;
    onClose: () => void;
}

export const NewGameTabs: React.FunctionComponent<NewGameTabsProps> = props => {
    const { sudokuController, onClose } = props;

    const [tabIndex, setTabIndex] = React.useState(0);

    const handleChange = (event: React.ChangeEvent<unknown>, newValue: number) => {
        setTabIndex(newValue);
    };

    return (
        <div>
            <AppBar position="static" color="default">
                <Tabs
                    value={tabIndex}
                    onChange={handleChange}
                    indicatorColor="primary"
                    textColor="primary"
                    variant="fullWidth"
                    centered
                >
                    <Tab label="Generate" />
                    <Tab label="Import" />
                </Tabs>
            </AppBar>
            <TabPanel index={0} tabIndex={tabIndex}>
                <GenerateForm onClose={onClose} sudokuController={sudokuController} />
            </TabPanel>
            <TabPanel index={1} tabIndex={tabIndex}>
                <ImportForm onClose={onClose} sudokuController={sudokuController} />
            </TabPanel>
        </div>
    );
};
