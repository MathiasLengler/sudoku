import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import { Stack, Tooltip } from "@mui/material";
import type { Control, FieldPath, FieldValues } from "react-hook-form";
import { ALL_STRATEGIES, STRATEGY_OPTIONS, type SelectedStrategies } from "../../constants";
import { ExternalLink } from "../ExternalLink";
import { MySelect } from "./mui-rhf/MySelect";
import type { StrategyEnum } from "../../../types";

type SelectStrategyProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
    /** Optional list of strategies to show. Defaults to ALL_STRATEGIES. */
    strategies?: readonly StrategyEnum[];
};
function SelectStrategy<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>({ control, name, strategies = ALL_STRATEGIES }: SelectStrategyProps<TFieldValues, TName, TTransformedValues>) {
    return (
        <MySelect
            control={control}
            name={name}
            label="Strategy"
            options={strategies.map((strategy) => {
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
