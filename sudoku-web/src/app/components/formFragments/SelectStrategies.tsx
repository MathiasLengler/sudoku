import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import { Stack, Tooltip } from "@mui/material";
import type { Control, FieldPath, FieldValues } from "react-hook-form";
import { ALL_STRATEGIES, STRATEGY_OPTIONS } from "../../constants";
import { ExternalLink } from "../ExternalLink";
import { MyCheckboxGroup } from "./mui-rhf/MyCheckboxGroup";

type SelectStrategiesProps<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
> = {
    control: Control<TFieldValues, unknown, TTransformedValues>;
    name: TName;
};
function SelectStrategies<
    TFieldValues extends FieldValues = FieldValues,
    TName extends FieldPath<TFieldValues> = FieldPath<TFieldValues>,
    TTransformedValues = TFieldValues,
>({ control, name }: SelectStrategiesProps<TFieldValues, TName, TTransformedValues>) {
    return (
        <MyCheckboxGroup
            control={control}
            name={name}
            label="Strategies"
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

export default SelectStrategies;
