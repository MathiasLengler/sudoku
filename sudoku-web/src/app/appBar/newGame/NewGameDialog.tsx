import { Tabs, Text } from "@mantine/core";
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
        <>
            <Text size="lg" fw={500} mb="md">
                New game
            </Text>
            <Tabs value={tabValue} onChange={(val) => setTabValue(val as NewGameTabValue)}>
                <Tabs.List grow>
                    <Tabs.Tab value="generate-form">Generate</Tabs.Tab>
                    <Tabs.Tab value="import-form">Import</Tabs.Tab>
                </Tabs.List>
                <Tabs.Panel value="generate-form" pt="md">
                    <GenerateForm onClose={onClose} />
                </Tabs.Panel>
                <Tabs.Panel value="import-form" pt="md">
                    <ImportForm onClose={onClose} />
                </Tabs.Panel>
            </Tabs>
        </>
    );
}
