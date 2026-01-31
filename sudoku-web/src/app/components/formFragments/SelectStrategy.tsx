import { IconInfoCircle } from "@tabler/icons-react";
import { Group, Select, Text, Tooltip } from "@mantine/core";
import type { Control, FieldValues, Path } from "react-hook-form";
import { Controller } from "react-hook-form";
import { ALL_STRATEGIES, STRATEGY_OPTIONS } from "../../constants";
import { ExternalLink } from "../ExternalLink";

type SelectStrategyProps<T extends FieldValues> = {
    control: Control<T>;
    name: Path<T>;
};
function SelectStrategy<T extends FieldValues>({ control, name }: SelectStrategyProps<T>) {
    return (
        <Controller
            control={control}
            name={name}
            render={({ field }) => (
                <Select
                    {...field}
                    label="Strategy"
                    data={ALL_STRATEGIES.map((strategy) => {
                        const option = STRATEGY_OPTIONS[strategy];
                        return {
                            value: strategy,
                            label: option.label,
                        };
                    })}
                    renderOption={({ option }) => {
                        const strategyOption = STRATEGY_OPTIONS[option.value as keyof typeof STRATEGY_OPTIONS];
                        return (
                            <Group gap="xs">
                                <Text size="sm">{strategyOption.label}</Text>
                                <Tooltip label={<ExternalLink href={strategyOption.link}>{strategyOption.description}</ExternalLink>}>
                                    <IconInfoCircle size={16} style={{ opacity: 0.6 }} />
                                </Tooltip>
                            </Group>
                        );
                    }}
                    required
                />
            )}
        />
    );
}

export default SelectStrategy;
