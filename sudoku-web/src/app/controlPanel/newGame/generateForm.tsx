import React, { useEffect } from "react";
import range from "lodash/range";
import type { WasmSudokuController } from "../../wasmSudokuController";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import CircularProgress from "@mui/material/CircularProgress";
import { CheckboxButtonGroup, SliderElement, SwitchElement, useForm } from "react-hook-form-mui";
import type { DynamicStrategy } from "../../../types";
import { Box, DialogContent, FormLabel } from "@mui/material";
import { baseToCellCount, baseToSideLength } from "../../utils";

const BASE_MIN = 2;
const BASE_MAX = 5;
const BASE_MARKS = range(BASE_MIN, BASE_MAX + 1).map(base => {
    const sideLength = baseToSideLength(base);
    return {
        value: base,
        label: `${sideLength}x${sideLength}`,
    };
});

interface GenerateFormProps {
    sudokuController: WasmSudokuController;
    onClose: () => void;
}
interface FormData {
    base: number;
    minGivens: number;
    strategies: DynamicStrategy[];
    setAllDirectCandidates: boolean;
}

const arrayOfAll =
    <T,>() =>
    <U extends T[]>(array: U & ([T] extends [U[number]] ? unknown : "Invalid")) =>
        array;

// Copy of sudokuController.allStrategies
const ALL_STRATEGIES = arrayOfAll<DynamicStrategy>()([
    "SingleCandidate",
    "HiddenSingles",
    "GroupReduction",
    "Backtracking",
]);

export const GenerateForm: React.FunctionComponent<GenerateFormProps> = props => {
    const { sudokuController, onClose } = props;

    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
        setValue,
    } = useForm<FormData>({
        defaultValues: {
            base: 3,
            minGivens: 0,
            strategies: ALL_STRATEGIES,
            setAllDirectCandidates: true,
        },
    });

    const { base, minGivens } = watch();

    const cellCount = baseToCellCount(base);

    useEffect(() => {
        if (cellCount < minGivens) {
            setValue("minGivens", cellCount);
        }
    }, [cellCount, minGivens, setValue]);

    return (
        <form
            noValidate
            onSubmit={handleSubmit(async formData => {
                const { base, minGivens, setAllDirectCandidates, strategies } = formData;

                const cellCount = baseToCellCount(base);

                await sudokuController.generate({
                    base,
                    target: {
                        fromFilled: {
                            distance: cellCount - minGivens,
                            set_all_direct_candidates: setAllDirectCandidates,
                        },
                    },
                    strategies,
                });
                onClose();
            })}
        >
            <DialogContent>
                <SliderElement
                    //
                    control={control}
                    name="base"
                    label="Size"
                    min={BASE_MIN}
                    max={BASE_MAX}
                    marks={BASE_MARKS}
                    valueLabelDisplay="auto"
                />
                <SliderElement
                    //
                    control={control}
                    name="minGivens"
                    label="Minimum number of givens"
                    step={1}
                    min={0}
                    max={cellCount}
                    marks={[
                        { value: 0, label: 0 },
                        { value: cellCount, label: cellCount },
                    ]}
                    valueLabelDisplay="auto"
                />
                <CheckboxButtonGroup
                    control={control}
                    name="strategies"
                    label="Strategies"
                    options={ALL_STRATEGIES.map(strategy => ({ id: strategy, label: strategy }))}
                    row
                />
                <FormLabel component="legend">Post generation</FormLabel>
                <SwitchElement control={control} name="setAllDirectCandidates" label="Set all direct candidates" />
            </DialogContent>
            <DialogActions>
                {isSubmitting && (
                    <Box>
                        <CircularProgress />
                    </Box>
                )}
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <Button type="submit" color="primary" disabled={isSubmitting}>
                    Generate
                </Button>
            </DialogActions>
        </form>
    );
};
