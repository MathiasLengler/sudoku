import { zodResolver } from "@hookform/resolvers/zod";
import CasinoIcon from "@mui/icons-material/Casino";
import PlayArrowIcon from "@mui/icons-material/PlayArrow";
import TabPanel from "@mui/lab/TabPanel";
import { Box, FormGroup, LinearProgress, Stack, Typography } from "@mui/material";
import Button from "@mui/material/Button";
import * as _ from "es-toolkit";
import { useAtom } from "jotai";
import { useEffect } from "react";
import { SelectElement, SliderElement, SwitchElement, TextFieldElement, useForm } from "react-hook-form-mui";
import type { DynamicGeneratorSettings, GeneratorProgress } from "../../../types";
import {
    useGenerate,
    useGenerateMultiShot,
    type TrackedMultiShotGeneratorProgress,
} from "../../../app/actions/sudokuActions";
import { Fieldset } from "../../../app/components/Fieldset";
import SelectStrategies from "../../../app/components/formFragments/SelectStrategies";
import SelectStrategy from "../../../app/components/formFragments/SelectStrategy";
import MyIconButton from "../../../app/components/MyIconButton";
import { ResetFormButton } from "../../../app/components/ResetFormButton";
import {
    ALL_GOAL_OPTIMIZATIONS,
    ALL_GRID_METRIC_NAMES,
    GRID_METRIC_NAMES_WITH_STRATEGY,
    GRID_METRIC_OPTIONS,
} from "../../../app/constants";
import {
    GENERATE_FORM_DEFAULT_VALUES,
    generateFormValuesSchema,
    generateFormValuesState,
    iterationsIndexToIterations,
    MAX_ITERATIONS_INDEX,
    MIN_ITERATIONS_INDEX,
    SEED_MAX,
    type GenerateFormValues,
} from "../../../app/state/forms/generate";
import { BASE_MARKS, BASE_MAX, BASE_MIN, baseToLabel, parseBase } from "../../../app/utils/base";
import { baseToCellCount } from "../../../app/utils/sudoku";
import type { NewGameTabValue } from "../index";

function GenerateProgressLayout({
    linearProgress,
    description,
}: {
    linearProgress: React.ReactNode;
    description: string;
}) {
    return (
        <Box sx={{ display: "flex", alignItems: "center", pt: 2, flexDirection: "column" }}>
            <Box sx={{ width: 1, pb: 1 }}>{linearProgress}</Box>
            <Typography
                variant="body2"
                sx={{
                    color: "text.secondary",
                }}
            >
                {description}
            </Typography>
        </Box>
    );
}

type GenerateProgressProps = {
    progress?: GeneratorProgress;
    cellCount: number;
};
function GenerateProgress({ progress, cellCount }: GenerateProgressProps) {
    if (!progress) {
        return <GenerateProgressLayout linearProgress={<LinearProgress />} description={"Generating solution"} />;
    }

    const { pruningPositionCount, pruningPositionIndex, deletedCount } = progress;
    const value = (pruningPositionIndex / pruningPositionCount) * 100;

    return (
        <GenerateProgressLayout
            linearProgress={<LinearProgress variant="determinate" value={value} />}
            description={`Cell ${pruningPositionIndex}/${pruningPositionCount} - deleted ${deletedCount}, remaining ${
                cellCount - deletedCount
            }`}
        />
    );
}

type GenerateMultiShotProgressProps = {
    trackedMultiShotGeneratorProgress?: TrackedMultiShotGeneratorProgress;
};
function GenerateMultiShotProgress({ trackedMultiShotGeneratorProgress }: GenerateMultiShotProgressProps) {
    if (!trackedMultiShotGeneratorProgress) {
        return (
            <GenerateProgressLayout
                linearProgress={<LinearProgress />}
                description={"Initializing multi-shot generator"}
            />
        );
    }

    const { totalIterations } = trackedMultiShotGeneratorProgress.latestProgress;

    const seenIterationsCount = trackedMultiShotGeneratorProgress.seenIterationsCount;
    const processingPercentage = (seenIterationsCount / totalIterations) * 100;

    const finishedIterationsCount = trackedMultiShotGeneratorProgress.finishedIterationsCount;
    const finishedPercentage = (finishedIterationsCount / totalIterations) * 100;

    const inProgressCount = seenIterationsCount - finishedIterationsCount;

    return (
        <GenerateProgressLayout
            linearProgress={
                <LinearProgress variant="buffer" value={finishedPercentage} valueBuffer={processingPercentage} />
            }
            description={`Iteration ${finishedIterationsCount}/${totalIterations}, in progress: ${inProgressCount}`}
        />
    );
}

type GenerateFormPageProps = {
    onClose: () => void;
};
export function GenerateFormPage({ onClose }: GenerateFormPageProps) {
    const [generateFormValues, setGenerateFormValues] = useAtom(generateFormValuesState);

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

    const formValues = watch();

    const [base, minGivens, useSeed, multiShot, metric] = watch([
        "base",
        "minGivens",
        "useSeed",
        "multiShot",
        "metric",
    ]);
    const cellCount = baseToCellCount(base);

    useEffect(() => {
        if (cellCount < minGivens) {
            setValue("minGivens", cellCount);
        }
    }, [cellCount, minGivens, setValue]);

    const { generate, generateProgress, cancelGenerate } = useGenerate();

    const { generateMultiShot, trackedMultiShotGeneratorProgress, cancelGenerateMultiShot } = useGenerateMultiShot();

    // Cancel generation on unmount/page navigation
    useEffect(() => {
        return () => {
            cancelGenerate();
            cancelGenerateMultiShot();
        };
    }, [cancelGenerate, cancelGenerateMultiShot]);

    return (
        <TabPanel value={"generate" satisfies NewGameTabValue} sx={{ px: 0, py: 2 }}>
            <form
                id="generate-form"
                onSubmit={handleSubmit(async () => {
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
                        base: parseBase(base),
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
                        <SwitchElement control={control} name="setAllDirectCandidates" label="Fill candidates" />
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
                                slotProps={{
                                    htmlInput: { inputMode: "numeric" },
                                    input: {
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
                                    },
                                }}
                            />
                        </FormGroup>
                    </Fieldset>
                    <Fieldset label="Multi-shot settings">
                        <SwitchElement control={control} name="multiShot" label="Multi-shot" />
                        <SwitchElement control={control} name="parallel" label="Parallel" disabled={!multiShot} />
                        <Stack spacing={2}>
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
                            <SelectElement
                                control={control}
                                name="metric.kind"
                                label="Metric"
                                disabled={!multiShot}
                                helperText={GRID_METRIC_OPTIONS[metric.kind]?.description}
                                options={ALL_GRID_METRIC_NAMES.map((gridMetric) => {
                                    const option = GRID_METRIC_OPTIONS[gridMetric];
                                    return {
                                        id: gridMetric,
                                        label: option.label,
                                        disabled: option.disabled,
                                    };
                                })}
                            />
                            {(GRID_METRIC_NAMES_WITH_STRATEGY as string[]).includes(metric.kind) && (
                                <SelectStrategy control={control} name="metric.strategy" />
                            )}
                            <SelectElement
                                control={control}
                                name="optimize"
                                label="Optimize"
                                disabled={!multiShot}
                                options={ALL_GOAL_OPTIMIZATIONS.map((goalOptimizations) => ({
                                    id: goalOptimizations,
                                    label: goalOptimizations,
                                }))}
                            />
                        </Stack>
                    </Fieldset>
                </Stack>
            </form>
            <Stack direction="column" sx={{ width: 1, mt: 2 }}>
                {isSubmitting &&
                    (multiShot ? (
                        <GenerateMultiShotProgress trackedMultiShotGeneratorProgress={trackedMultiShotGeneratorProgress} />
                    ) : (
                        <GenerateProgress progress={generateProgress} cellCount={cellCount} />
                    ))}
                <Stack direction="row" sx={{ width: 1, flex: 1, alignItems: "center", justifyContent: "space-between" }}>
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
        </TabPanel>
    );
}
