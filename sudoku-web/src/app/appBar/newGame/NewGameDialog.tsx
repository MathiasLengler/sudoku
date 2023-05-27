import React from "react";
import AppBar from "@mui/material/AppBar";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import { GenerateForm } from "./GenerateForm";
import { ImportForm } from "./ImportForm";

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
    onClose: () => void;
}

export const NewGameDialog = ({ onClose }: NewGameTabsProps) => {
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
                <GenerateForm onClose={onClose} />
            </TabPanel>
            <TabPanel index={1} tabIndex={tabIndex}>
                <ImportForm onClose={onClose} />
            </TabPanel>
        </div>
    );
};
