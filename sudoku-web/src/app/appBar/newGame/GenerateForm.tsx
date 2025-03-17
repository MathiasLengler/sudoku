import { Box, Checkbox, DialogContent, FormGroup, LinearProgress, Stack, Typography } from "@mui/material";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import { useEffect } from "react";
import { SliderElement, SwitchElement, TextFieldElement, useForm } from "react-hook-form-mui";

import { zodResolver } from "@hookform/resolvers/zod";
import CasinoIcon from "@mui/icons-material/Casino";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import LoopIcon from "@mui/icons-material/Loop";
import TabPanel from "@mui/lab/TabPanel";
import * as _ from "lodash-es";
import { useRecoilState } from "recoil";
import type { DynamicGeneratorSettings, GeneratorProgress, MultiShotGeneratorProgress } from "../../../types";
import { useGenerate, useGenerateMultiShot, type IterationToProgress } from "../../actions/sudokuActions";
import { Fieldset } from "../../components/Fieldset";
import SelectStrategies from "../../components/formFragments/SelectStrategies";
import MyIconButton from "../../components/MyIconButton";
import { ResetFormButton } from "../../components/ResetFormButton";
import {
    BASE_MARKS,
    BASE_MAX,
    BASE_MIN,
    baseToLabel,
    GENERATE_FORM_DEFAULT_VALUES,
    generateFormValuesSchema,
    generateFormValuesState,
    iterationsIndexToIterations,
    MAX_ITERATIONS_INDEX,
    MIN_ITERATIONS_INDEX,
    SEED_MAX,
    type GenerateFormValues,
} from "../../state/forms/generate";
import { baseToCellCount } from "../../utils/sudoku";
import type { NewGameTabValue } from "./NewGameDialog";
import assertNever from "assert-never";

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

type GenerateMultiShotProgressProps = {
    progress?: MultiShotGeneratorProgress;
    iterationToProgress: IterationToProgress;
};
function GenerateMultiShotProgress({ progress, iterationToProgress: seenIterations }: GenerateMultiShotProgressProps) {
    if (!progress) {
        return null;
    }

    const { totalIterations } = progress;

    const seenIterationsCount = seenIterations.size;
    const processingPercentage = (seenIterationsCount / totalIterations) * 100;

    const finishedIterationsCount = seenIterations
        .values()
        .filter((progress) => progress.kind === "finished")
        .reduce((sum) => sum + 1, 0);
    const finishedPercentage = (finishedIterationsCount / totalIterations) * 100;

    const gridTemplateColumns = `repeat(${Math.ceil(Math.sqrt(totalIterations))}, 1fr)`;

    return (
        <Box sx={{ display: "flex", alignItems: "center", pt: 2, flexDirection: "column" }}>
            <Box
                sx={{
                    display: "grid",
                    gridTemplateColumns: gridTemplateColumns,
                }}
            >
                {_.range(0, totalIterations).map((iteration) => {
                    const progress = seenIterations.get(iteration);

                    return (
                        <Checkbox
                            key={iteration}
                            checked={progress?.kind === "finished"}
                            indeterminate={progress?.kind === "started"}
                            indeterminateIcon={<LoopIcon />}
                            sx={{ p: 0, m: 0 }}
                            size="small"
                            readOnly
                        />
                    );
                })}
            </Box>
            <Box sx={{ width: 1, pb: 1 }}>
                <LinearProgress variant="buffer" value={finishedPercentage} valueBuffer={processingPercentage} />
            </Box>
            <Box sx={{ minWidth: 35 }}>
                <Typography
                    variant="body2"
                    sx={{
                        color: "text.secondary",
                    }}
                >{`Iteration ${seenIterationsCount}/${totalIterations}`}</Typography>
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

    const { base, minGivens, useSeed, multiShot } = watch();
    const cellCount = baseToCellCount(base);

    useEffect(() => {
        if (cellCount < minGivens) {
            setValue("minGivens", cellCount);
        }
    }, [cellCount, minGivens, setValue]);

    const { generate, generateProgress, cancelGenerate } = useGenerate();

    const { generateMultiShot, generateMultiShotProgress, cancelGenerateMultiShot, iterationToProgress } =
        useGenerateMultiShot();

    // Cancel generation on unmount/modal close
    useEffect(() => {
        return () => {
            cancelGenerate();
            cancelGenerateMultiShot();
        };
    }, [cancelGenerate, cancelGenerateMultiShot]);

    return (
        <>
            <DialogContent>
                <TabPanel value={"generate-form" satisfies NewGameTabValue} sx={{ p: 0 }}>
                    <form
                        id="generate-form"
                        onSubmit={handleSubmit(async (formValues) => {
                            const {
                                base,
                                minGivens,
                                setAllDirectCandidates,
                                strategies,
                                seed,
                                useSeed,
                                multiShot,
                                iterationsIndex,
                                metric,
                                optimize,
                                parallel,
                            } = formValues;

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
                            };

                            try {
                                if (multiShot) {
                                    await generateMultiShot({
                                        generatorSettings,
                                        iterations: iterationsIndexToIterations(iterationsIndex),
                                        metric,
                                        optimize,
                                        parallel,
                                    });
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
                            <Fieldset label="Multi-shot settings">
                                <SwitchElement control={control} name="multiShot" label="Multi-shot" />
                                <SwitchElement
                                    control={control}
                                    name="parallel"
                                    label="Parallel"
                                    disabled={!multiShot}
                                />
                                <SliderElement
                                    control={control}
                                    name="iterationsIndex"
                                    label="Iterations"
                                    disabled={!multiShot}
                                    step={1}
                                    min={MIN_ITERATIONS_INDEX}
                                    max={MAX_ITERATIONS_INDEX}
                                    scale={iterationsIndexToIterations}
                                    valueLabelDisplay="auto"
                                    getAriaLabel={() => "Iterations"}
                                    getAriaValueText={(iterations) => `${iterations}`}
                                />
                                {/* TODO: metric, optimize, parallel */}
                            </Fieldset>
                        </Stack>
                    </form>
                </TabPanel>
            </DialogContent>
            <DialogActions>
                <Stack direction="column" sx={{ width: 1 }}>
                    {isSubmitting &&
                        (multiShot ? (
                            <GenerateMultiShotProgress
                                progress={generateMultiShotProgress}
                                iterationToProgress={iterationToProgress}
                            />
                        ) : (
                            <GenerateProgress progress={generateProgress} cellCount={cellCount} />
                        ))}
                    <Stack
                        direction="row"
                        sx={{ width: 1, flex: 1, alignItems: "center", justifyContent: "space-between" }}
                    >
                        <ResetFormButton disabled={isSubmitting} onClick={() => reset(GENERATE_FORM_DEFAULT_VALUES)} />
                        <Button
                            type="button"
                            onClick={() => {
                                if (isSubmitting) {
                                    if (multiShot) {
                                        cancelGenerateMultiShot();
                                    } else {
                                        cancelGenerate();
                                    }
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
                    </Stack>
                </Stack>
            </DialogActions>
        </>
    );
};
