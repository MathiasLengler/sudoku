import React, { useEffect } from "react";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import { SliderElement, SwitchElement, TextFieldElement, useForm } from "react-hook-form-mui";
import { Box, DialogContent, FormGroup, LinearProgress, Typography } from "@mui/material";
import { Stack } from "@mui/material";

import { baseToCellCount } from "../../utils/sudoku";
import { useGenerate, useMultiShotGenerate } from "../../actions/sudokuActions";
import CasinoIcon from "@mui/icons-material/Casino";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import { useRecoilState } from "recoil";
import { zodResolver } from "@hookform/resolvers/zod";
import * as _ from "lodash-es";
import {
    BASE_MARKS,
    BASE_MAX,
    BASE_MIN,
    baseToLabel,
    GENERATE_FORM_DEFAULT_VALUES,
    type GenerateFormValues,
    generateFormValuesSchema,
    generateFormValuesState,
    SEED_MAX,
} from "../../state/forms/generate";
import type { DynamicGeneratorSettings, GeneratorProgress } from "../../../types";
import MyIconButton from "../../components/MyIconButton";
import SelectStrategies from "../../components/formFragments/SelectStrategies";
import { ResetFormButton } from "../../components/ResetFormButton";
import { Fieldset } from "../../components/Fieldset";
import TabPanel from "@mui/lab/TabPanel";
import type { NewGameTabValue } from "./NewGameDialog";

type GenerateProgressProps = {
    progress?: GeneratorProgress;
    cellCount: number;
};
function GenerateProgress({ progress, cellCount }: GenerateProgressProps) {
    if (!progress) {
        return null;
    }

    const { pruningPositionCount, pruningPositionIndex, deletedCount } = progress;
    const value = (pruningPositionIndex / pruningPositionCount) * 100;

    return (
        <Box sx={{ display: "flex", alignItems: "center", pt: 2, flexDirection: "column" }}>
            <Box sx={{ width: 1, pb: 1 }}>
                <LinearProgress variant="determinate" value={value} />
            </Box>
            <Box sx={{ minWidth: 35 }}>
                <Typography
                    variant="body2"
                    sx={{
                        color: "text.secondary",
                    }}
                >{`Cell ${pruningPositionIndex}/${pruningPositionCount} - deleted ${deletedCount}, remaining ${
                    cellCount - deletedCount
                }`}</Typography>
            </Box>
        </Box>
    );
}

type GenerateFormProps = {
    onClose: () => void;
};
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

    const { base, minGivens, useSeed } = watch();
    const cellCount = baseToCellCount(base);

    useEffect(() => {
        if (cellCount < minGivens) {
            setValue("minGivens", cellCount);
        }
    }, [cellCount, minGivens, setValue]);

    const { generate, progress, cancelGenerate } = useGenerate();

    // Cancel generation on unmount/modal close
    useEffect(() => {
        return () => {
            cancelGenerate();
        };
    }, [cancelGenerate]);

    const multiShotGenerate = useMultiShotGenerate();

    return (
        <>
            <DialogContent>
                <TabPanel value={"generate-form" satisfies NewGameTabValue} sx={{ p: 0 }}>
                    <form
                        id="generate-form"
                        onSubmit={handleSubmit(async (formValues) => {
                            const { base, minGivens, setAllDirectCandidates, strategies, seed, useSeed, parallel } =
                                formValues;

                            const generatorSettings: DynamicGeneratorSettings = {
                                base,
                                prune: {
                                    target: {
                                        minClueCount: minGivens,
                                    },
                                    strategies,
                                    setAllDirectCandidates,
                                    // TODO: expose
                                    order: "random",
                                    startFromNearMinimalGrid: false,
                                },
                                solution: undefined,
                                seed: useSeed && !_.isUndefined(seed) ? BigInt(seed) : undefined,
                                // FIXME: multi/single shot mode call generate_multi_shot/generate
                                // parallel,
                            };

                            try {
                                if (parallel) {
                                    await multiShotGenerate(
                                        {
                                            generatorSettings,
                                            // TODO: expose other multiShot settings in form
                                            iterations: 1000,
                                            metric: "strategyTotalScore",
                                            optimize: "maximize",
                                            parallel: true,
                                        },
                                        (progress) => {
                                            console.log("MultiShot progress", progress);
                                        },
                                    );
                                } else {
                                    await generate(generatorSettings);
                                }
                            } catch (err) {
                                if (!(err instanceof DOMException && err.name === "AbortError")) {
                                    throw err;
                                }
                                console.info("Generate form submission aborted");
                                return;
                            }

                            setGenerateFormValues(formValues);

                            onClose();
                        })}
                    >
                        <Stack spacing={2}>
                            <SliderElement
                                control={control}
                                name="base"
                                label="Size"
                                min={BASE_MIN}
                                max={BASE_MAX}
                                marks={BASE_MARKS}
                                valueLabelDisplay="auto"
                                getAriaLabel={() => "Size"}
                                getAriaValueText={(base) => baseToLabel(base)}
                            />
                            <SliderElement
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
                                getAriaLabel={() => "Minimum number of givens"}
                                getAriaValueText={(minGivens) => `${minGivens}`}
                            />
                            <SelectStrategies control={control} name="strategies" />

                            <Fieldset label="Post generation">
                                <SwitchElement
                                    control={control}
                                    name="setAllDirectCandidates"
                                    label="Fill candidates"
                                />
                            </Fieldset>

                            <Fieldset label="Random seed">
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
                                                <MyIconButton
                                                    label="Generate random seed"
                                                    icon={CasinoIcon}
                                                    disabled={!useSeed}
                                                    onClick={() => {
                                                        setValue(
                                                            "seed",
                                                            Math.trunc(Math.random() * SEED_MAX).toFixed(0),
                                                        );
                                                    }}
                                                />
                                            ),
                                        }}
                                    />
                                </FormGroup>
                            </Fieldset>
                            <SwitchElement control={control} name="parallel" label="Parallel" />
                            <GenerateProgress progress={progress} cellCount={cellCount} />
                        </Stack>
                    </form>
                </TabPanel>
            </DialogContent>
            <DialogActions>
                <ResetFormButton disabled={isSubmitting} onClick={() => reset(GENERATE_FORM_DEFAULT_VALUES)} />
                <Button
                    type="button"
                    onClick={() => {
                        if (isSubmitting) {
                            cancelGenerate();
                        } else {
                            onClose();
                        }
                    }}
                >
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="generate-form"
                    color="primary"
                    variant="contained"
                    endIcon={<PlayArrowIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    <span>Generate</span>
                </Button>
            </DialogActions>
        </>
    );
};
