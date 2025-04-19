import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import { Link, Stack, Tooltip } from "@mui/material";
import type { ReactNode } from "react";
import type { Control, FieldValues, Path } from "react-hook-form";
import { CheckboxButtonGroup } from "react-hook-form-mui";
import { ALL_STRATEGIES, STRATEGY_OPTIONS } from "../../constants";

function ExternalLink({ children, href }: { children: ReactNode; href: string }) {
    return (
        <Link rel="noopener" target="_blank" href={href} color="inherit" underline="hover">
            {children}
        </Link>
    );
}

type SelectStrategiesProps<T extends FieldValues> = {
    control: Control<T>;
    name: Path<T>;
};
function SelectStrategies<T extends FieldValues>({ control, name }: SelectStrategiesProps<T>) {
    return (
        <>
            <CheckboxButtonGroup
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
                                <Tooltip
                                    title={
                                        <>
                                            <ExternalLink href={option.link}>{option.description}</ExternalLink>
                                        </>
                                    }
                                >
                                    <InfoOutlinedIcon fontSize="small" />
                                </Tooltip>
                            </Stack>
                        ),
                    };
                })}
                required
            />
        </>
    );
}

export default SelectStrategies;
