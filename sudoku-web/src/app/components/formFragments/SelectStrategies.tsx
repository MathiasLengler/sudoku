import { IconInfoCircle } from "@tabler/icons-react";
import { Checkbox, Group, Stack, Text, Tooltip } from "@mantine/core";
import type { Control, FieldValues, Path } from "react-hook-form";
import { Controller } from "react-hook-form";
import { ALL_STRATEGIES, STRATEGY_OPTIONS } from "../../constants";
import { ExternalLink } from "../ExternalLink";

type SelectStrategiesProps<T extends FieldValues> = {
    control: Control<T>;
    name: Path<T>;
};
function SelectStrategies<T extends FieldValues>({ control, name }: SelectStrategiesProps<T>) {
    return (
        <Controller
            control={control}
            name={name}
            render={({ field: { value, onChange } }) => (
                <Checkbox.Group
                    value={value as string[]}
                    onChange={onChange}
                    label="Strategies"
                >
                    <Stack gap="xs" mt="xs">
                        {ALL_STRATEGIES.map((strategy) => {
                            const option = STRATEGY_OPTIONS[strategy];
                            return (
                                <Checkbox
                                    key={strategy}
                                    value={strategy}
                                    label={
                                        <Group gap="xs">
                                            <Text size="sm">{option.label}</Text>
                                            <Tooltip label={<ExternalLink href={option.link}>{option.description}</ExternalLink>}>
                                                <IconInfoCircle size={16} style={{ opacity: 0.6 }} />
                                            </Tooltip>
                                        </Group>
                                    }
                                />
                            );
                        })}
                    </Stack>
                </Checkbox.Group>
            )}
        />
    );
}

export default SelectStrategies;
