import React, { useCallback, useEffect, useState } from "react";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import { CheckboxButtonGroup, SliderElement, SwitchElement, TextFieldElement, useForm } from "react-hook-form-mui";
import { Box, DialogContent, FormGroup, FormLabel, IconButton, LinearProgress, Typography } from "@mui/material";
import { baseToCellCount } from "../../utils";
import { ALL_STRATEGIES } from "../../../constants";
import { useGenerate } from "../../sudokuActions";
import CasinoIcon from "@mui/icons-material/Casino";
import ReplayIcon from "@mui/icons-material/Replay";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import { LoadingButton } from "@mui/lab";
import { useRecoilState } from "recoil";
import { zodResolver } from "@hookform/resolvers/zod";
import _ from "lodash";
import {
    BASE_MARKS,
    BASE_MAX,
    BASE_MIN,
    GENERATE_FORM_DEFAULT_VALUES,
    type GenerateFormValues,
    generateFormValuesSchema,
    generateFormValuesState,
    SEED_MAX,
} from "../../state/generateForm";
import type { GeneratorProgress } from "../../../types";

interface GenerateProgressProps {
    progress?: GeneratorProgress;
}
function GenerateProgress({ progress }: GenerateProgressProps) {
    if (!progress) return null;

    const value = (progress.positionIndex / progress.positionsCount) * 100;

    return (
        <Box sx={{ display: "flex", alignItems: "center", pt: 2 }}>
            <Box sx={{ width: "100%", mr: 1 }}>
                <LinearProgress variant="determinate" value={value} />
            </Box>
            <Box sx={{ minWidth: 35 }}>
                <Typography
                    variant="body2"
                    color="text.secondary"
                >{`${progress.positionIndex}/${progress.positionsCount}`}</Typography>
            </Box>
        </Box>
    );
}

interface GenerateFormProps {
    onClose: () => void;
}
export const GenerateForm = ({ onClose }: GenerateFormProps) => {
    const [generateFormValues, setGenerateFormValues] = useRecoilState(generateFormValuesState);
    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
        setValue,
        reset,
    } = useForm<GenerateFormValues>({
        values: generateFormValues,
        resolver: zodResolver(generateFormValuesSchema),
    });

    const { base, minGivens, useSeed, seed } = watch();
    const cellCount = baseToCellCount(base);

    useEffect(() => {
        if (cellCount < minGivens) {
            setValue("minGivens", cellCount);
        }
    }, [cellCount, minGivens, setValue]);

    const [progress, setProgress] = useState<GeneratorProgress>();

    const [progressAbortController, setProgressAbortController] = useState(() => new AbortController());

    const onProgress = useCallback(
        (progress: GeneratorProgress) => {
            if (progressAbortController.signal.aborted) {
                setProgressAbortController(new AbortController());
                throw progressAbortController.signal.reason;
            }

            setProgress(progress);
        },
        [progressAbortController.signal]
    );

    const generate = useGenerate(onProgress);

    return (
        <form
            onSubmit={handleSubmit(async formValues => {
                const { base, minGivens, setAllDirectCandidates, strategies, seed, useSeed } = formValues;

                const cellCount = baseToCellCount(base);

                await generate({
                    base,
                    target: {
                        fromFilled: {
                            distanceFromFilled: cellCount - minGivens,
                            setAllDirectCandidates,
                        },
                    },
                    seed: useSeed && !_.isUndefined(seed) ? BigInt(seed) : undefined,
                    strategies,
                });

                setGenerateFormValues(formValues);

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
                        inputProps={{ inputMode: "numeric" }}
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
                    />
                </FormGroup>
                <GenerateProgress progress={progress} />
            </DialogContent>
            <DialogActions sx={{ justifyContent: "space-between" }}>
                <IconButton
                    type="button"
                    disabled={isSubmitting}
                    onClick={() => {
                        reset(GENERATE_FORM_DEFAULT_VALUES);
                    }}
                >
                    <ReplayIcon />
                </IconButton>
                <Button
                    type="button"
                    onClick={() => {
                        if (isSubmitting) {
                            progressAbortController.abort();
                        } else {
                            onClose();
                        }
                    }}
                >
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
