import React, { useEffect } from "react";
import range from "lodash/range";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import CircularProgress from "@mui/material/CircularProgress";
import { CheckboxButtonGroup, SliderElement, SwitchElement, TextFieldElement, useForm } from "react-hook-form-mui";
import type { DynamicStrategy } from "../../../types";
import { Box, DialogContent, FormGroup, FormLabel, IconButton } from "@mui/material";
import { baseToCellCount, baseToSideLength } from "../../utils";
import { ALL_STRATEGIES } from "../../../constants";
import { useGenerate } from "../../sudokuActions";
import CasinoIcon from "@mui/icons-material/Casino";
import ReplayIcon from "@mui/icons-material/Replay";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import { LoadingButton } from "@mui/lab";

const BASE_MIN = 2;
const BASE_MAX = 5;
const BASE_MARKS = range(BASE_MIN, BASE_MAX + 1).map(base => {
    const sideLength = baseToSideLength(base);
    return {
        value: base,
        label: `${sideLength}x${sideLength}`,
    };
});
const SEED_MAX = Number.MAX_SAFE_INTEGER;

interface GenerateFormProps {
    onClose: () => void;
}
type FormData = {
    base: number;
    minGivens: number;
    strategies: DynamicStrategy[];
    setAllDirectCandidates: boolean;
    useSeed: false;
    seed: string;
};

let previousFormData: FormData | undefined;

export const GenerateForm = ({ onClose }: GenerateFormProps) => {
    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
        setValue,
        reset,
    } = useForm<FormData>({
        defaultValues: previousFormData ?? {
            base: 3,
            minGivens: 0,
            strategies: ["Backtracking"],
            setAllDirectCandidates: true,
            useSeed: false,
            seed: "0",
        },
    });

    const { base, minGivens, useSeed, seed } = watch();

    console.log({ seed });

    const cellCount = baseToCellCount(base);

    useEffect(() => {
        if (cellCount < minGivens) {
            setValue("minGivens", cellCount);
        }
    }, [cellCount, minGivens, setValue]);

    const generate = useGenerate();

    return (
        <form
            noValidate
            onSubmit={handleSubmit(async formData => {
                const { base, minGivens, setAllDirectCandidates, strategies, seed, useSeed } = formData;

                const cellCount = baseToCellCount(base);

                await generate({
                    base,
                    target: {
                        fromFilled: {
                            distance: cellCount - minGivens,
                            set_all_direct_candidates: setAllDirectCandidates,
                        },
                    },
                    seed: useSeed ? BigInt(seed) : undefined,
                    strategies,
                });

                previousFormData = formData;

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
                    required
                />

                <FormLabel component="legend">Post generation</FormLabel>
                <SwitchElement control={control} name="setAllDirectCandidates" label="Fill candidates" />

                <FormLabel component="legend">Random seed</FormLabel>
                <FormGroup row>
                    <SwitchElement control={control} name="useSeed" label="Use seed" />
                    <TextFieldElement
                        sx={{ flex: 1 }}
                        control={control}
                        name="seed"
                        label="Seed"
                        disabled={!useSeed}
                        validation={{
                            min: { value: 0, message: "Seed must not be negative" },
                            max: { value: SEED_MAX, message: "Seed too big" },
                        }}
                        InputProps={{
                            startAdornment: (
                                <IconButton
                                    onClick={() => {
                                        setValue("seed", Math.trunc(Math.random() * SEED_MAX).toFixed(0));
                                    }}
                                    disabled={!useSeed}
                                >
                                    <CasinoIcon />
                                </IconButton>
                            ),
                        }}
                        type="number"
                    />
                </FormGroup>
            </DialogContent>
            <DialogActions sx={{ justifyContent: "space-between" }}>
                <IconButton
                    type="button"
                    disabled={isSubmitting}
                    onClick={() => {
                        reset();
                    }}
                >
                    <ReplayIcon />
                </IconButton>
                <Button type="button" onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <LoadingButton
                    type="submit"
                    color="primary"
                    variant="contained"
                    endIcon={<PlayArrowIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    <span>Generate</span>
                </LoadingButton>
            </DialogActions>
        </form>
    );
};
