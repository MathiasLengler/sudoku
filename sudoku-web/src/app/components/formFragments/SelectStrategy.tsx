import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import { Stack, Tooltip } from "@mui/material";
import type { Control, FieldValues, Path } from "react-hook-form";
import { ALL_STRATEGIES, STRATEGY_OPTIONS } from "../../constants";
import { SelectElement } from "react-hook-form-mui";
import { ExternalLink } from "../ExternalLink";

type SelectStrategyProps<T extends FieldValues> = {
    control: Control<T>;
    name: Path<T>;
};
function SelectStrategy<T extends FieldValues>({ control, name }: SelectStrategyProps<T>) {
    return (
        <SelectElement
            control={control}
            name={name}
            label="Strategy"
            options={ALL_STRATEGIES.map((strategy) => {
                const option = STRATEGY_OPTIONS[strategy];
                return {
                    id: strategy,
                    label: (
                        <Stack direction="row" alignItems="center" gap={1}>
                            {option.label}
                            <Tooltip title={<ExternalLink href={option.link}>{option.description}</ExternalLink>}>
                                <InfoOutlinedIcon fontSize="small" />
                            </Tooltip>
                        </Stack>
                    ),
                };
            })}
            required
        />
    );
}

export default SelectStrategy;
