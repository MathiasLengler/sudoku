import { zodResolver } from "@hookform/resolvers/zod";
import { IconDice, IconPlayerPlay } from "@tabler/icons-react";
import { Box, Button, Group, Progress, Select, Slider, Stack, Switch, Text, TextInput } from "@mantine/core";
import * as _ from "es-toolkit";
import { useAtom } from "jotai";
import { useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
import type { DynamicGeneratorSettings, GeneratorProgress } from "../../../types";
import { useGenerate, useGenerateMultiShot, type TrackedMultiShotGeneratorProgress } from "../../actions/sudokuActions";
import { Fieldset } from "../../components/Fieldset";
import SelectStrategies from "../../components/formFragments/SelectStrategies";
import SelectStrategy from "../../components/formFragments/SelectStrategy";
import MyIconButton from "../../components/MyIconButton";
import { ResetFormButton } from "../../components/ResetFormButton";
import {
    ALL_GOAL_OPTIMIZATIONS,
    ALL_GRID_METRIC_NAMES,
    GRID_METRIC_NAMES_WITH_STRATEGY,
    GRID_METRIC_OPTIONS,
} from "../../constants";
import {
    GENERATE_FORM_DEFAULT_VALUES,
    generateFormValuesSchema,
    generateFormValuesState,
    iterationsIndexToIterations,
    MAX_ITERATIONS_INDEX,
    MIN_ITERATIONS_INDEX,
    SEED_MAX,
    type GenerateFormValues,
} from "../../state/forms/generate";
import { BASE_MARKS, BASE_MAX, BASE_MIN, baseToLabel, parseBase } from "../../utils/base";
import { baseToCellCount } from "../../utils/sudoku";

function GenerateProgressLayout({
    linearProgress,
    description,
}: {
    linearProgress: React.ReactNode;
    description: string;
}) {
    return (
        <Box pt="md">
            <Box pb="xs">{linearProgress}</Box>
            <Text size="sm" c="dimmed" ta="center">
                {description}
            </Text>
        </Box>
    );
}

type GenerateProgressProps = {
    progress?: GeneratorProgress;
    cellCount: number;
};
function GenerateProgress({ progress, cellCount }: GenerateProgressProps) {
    if (!progress) {
        return <GenerateProgressLayout linearProgress={<Progress value={100} animated />} description={"Generating solution"} />;
    }

    const { pruningPositionCount, pruningPositionIndex, deletedCount } = progress;
    const value = (pruningPositionIndex / pruningPositionCount) * 100;

    return (
        <GenerateProgressLayout
            linearProgress={<Progress value={value} />}
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
                linearProgress={<Progress value={100} animated />}
                description={"Initializing multi-shot generator"}
            />
        );
    }

    const { totalIterations } = trackedMultiShotGeneratorProgress.latestProgress;

    const finishedIterationsCount = trackedMultiShotGeneratorProgress.finishedIterationsCount;
    const finishedPercentage = (finishedIterationsCount / totalIterations) * 100;

    const inProgressCount = trackedMultiShotGeneratorProgress.seenIterationsCount - finishedIterationsCount;

    return (
        <GenerateProgressLayout
            linearProgress={<Progress value={finishedPercentage} />}
            description={`Iteration ${finishedIterationsCount}/${totalIterations}, in progress: ${inProgressCount}`}
        />
    );
}

type GenerateFormProps = {
    onClose: () => void;
};
export function GenerateForm({ onClose }: GenerateFormProps) {
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

    // Cancel generation on unmount/modal close
    useEffect(() => {
        return () => {
            cancelGenerate();
            cancelGenerateMultiShot();
        };
    }, [cancelGenerate, cancelGenerateMultiShot]);

    return (
        <>
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
                <Stack gap="md">
                    <Box>
                        <Text size="sm" fw={500} mb="xs">Size</Text>
                        <Controller
                            name="base"
                            control={control}
                            render={({ field: { value, onChange } }) => (
                                <Slider
                                    value={value}
                                    onChange={onChange}
                                    label={baseToLabel}
                                    min={BASE_MIN}
                                    max={BASE_MAX}
                                    marks={BASE_MARKS.map(m => ({ value: m.value, label: m.label }))}
                                />
                            )}
                        />
                    </Box>
                    <Box>
                        <Text size="sm" fw={500} mb="xs">Minimum number of givens</Text>
                        <Controller
                            name="minGivens"
                            control={control}
                            render={({ field: { value, onChange } }) => (
                                <Slider
                                    value={value}
                                    onChange={onChange}
                                    step={1}
                                    min={0}
                                    max={cellCount}
                                    marks={[
                                        { value: 0, label: "0" },
                                        { value: cellCount, label: `${cellCount}` },
                                    ]}
                                />
                            )}
                        />
                    </Box>
                    <SelectStrategies control={control} name="strategies" />

                    <Fieldset label="Post generation">
                        <Controller
                            name="setAllDirectCandidates"
                            control={control}
                            render={({ field: { value, onChange, ...field } }) => (
                                <Switch
                                    {...field}
                                    checked={value}
                                    onChange={(e) => onChange(e.currentTarget.checked)}
                                    label="Fill candidates"
                                />
                            )}
                        />
                    </Fieldset>

                    <Fieldset label="Random seed">
                        <Group align="flex-end">
                            <Controller
                                name="useSeed"
                                control={control}
                                render={({ field: { value, onChange, ...field } }) => (
                                    <Switch
                                        {...field}
                                        checked={value}
                                        onChange={(e) => onChange(e.currentTarget.checked)}
                                        label="Use seed"
                                    />
                                )}
                            />
                            <Controller
                                name="seed"
                                control={control}
                                render={({ field }) => (
                                    <TextInput
                                        {...field}
                                        value={field.value ?? ""}
                                        label="Seed"
                                        disabled={!useSeed}
                                        style={{ flex: 1 }}
                                        leftSection={
                                            <MyIconButton
                                                label="Generate random seed"
                                                icon={IconDice}
                                                size="sm"
                                                disabled={!useSeed}
                                                onClick={() => {
                                                    setValue(
                                                        "seed",
                                                        Math.trunc(Math.random() * SEED_MAX).toFixed(0),
                                                    );
                                                }}
                                            />
                                        }
                                    />
                                )}
                            />
                        </Group>
                    </Fieldset>
                    <Fieldset label="Multi-shot settings">
                        <Stack gap="sm">
                            <Controller
                                name="multiShot"
                                control={control}
                                render={({ field: { value, onChange, ...field } }) => (
                                    <Switch
                                        {...field}
                                        checked={value}
                                        onChange={(e) => onChange(e.currentTarget.checked)}
                                        label="Multi-shot"
                                    />
                                )}
                            />
                            <Controller
                                name="parallel"
                                control={control}
                                render={({ field: { value, onChange, ...field } }) => (
                                    <Switch
                                        {...field}
                                        checked={value}
                                        onChange={(e) => onChange(e.currentTarget.checked)}
                                        label="Parallel"
                                        disabled={!multiShot}
                                    />
                                )}
                            />
                            <Box>
                                <Text size="sm" fw={500} mb="xs">Iterations</Text>
                                <Controller
                                    name="iterationsIndex"
                                    control={control}
                                    render={({ field: { value, onChange } }) => (
                                        <Slider
                                            value={value}
                                            onChange={onChange}
                                            label={(v) => `${iterationsIndexToIterations(v)}`}
                                            disabled={!multiShot}
                                            step={1}
                                            min={MIN_ITERATIONS_INDEX}
                                            max={MAX_ITERATIONS_INDEX}
                                        />
                                    )}
                                />
                            </Box>
                            <Controller
                                name="metric.kind"
                                control={control}
                                render={({ field }) => (
                                    <Select
                                        {...field}
                                        label="Metric"
                                        disabled={!multiShot}
                                        description={GRID_METRIC_OPTIONS[metric.kind]?.description}
                                        data={ALL_GRID_METRIC_NAMES.map((gridMetric) => {
                                            const option = GRID_METRIC_OPTIONS[gridMetric];
                                            return {
                                                value: gridMetric,
                                                label: option.label,
                                                disabled: option.disabled,
                                            };
                                        })}
                                    />
                                )}
                            />
                            {(GRID_METRIC_NAMES_WITH_STRATEGY as string[]).includes(metric.kind) && (
                                <SelectStrategy control={control} name="metric.strategy" />
                            )}
                            <Controller
                                name="optimize"
                                control={control}
                                render={({ field }) => (
                                    <Select
                                        {...field}
                                        label="Optimize"
                                        disabled={!multiShot}
                                        data={ALL_GOAL_OPTIMIZATIONS.map((goalOptimization) => ({
                                            value: goalOptimization,
                                            label: goalOptimization,
                                        }))}
                                    />
                                )}
                            />
                        </Stack>
                    </Fieldset>
                </Stack>
            </form>
            <Stack gap="sm" mt="md">
                {isSubmitting &&
                    (multiShot ? (
                        <GenerateMultiShotProgress
                            trackedMultiShotGeneratorProgress={trackedMultiShotGeneratorProgress}
                        />
                    ) : (
                        <GenerateProgress progress={generateProgress} cellCount={cellCount} />
                    ))}
                <Group justify="space-between">
                    <ResetFormButton disabled={isSubmitting} onClick={() => reset(GENERATE_FORM_DEFAULT_VALUES)} />
                    <Button
                        variant="subtle"
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
                        rightSection={<IconPlayerPlay size={18} />}
                        loading={isSubmitting}
                    >
                        Generate
                    </Button>
                </Group>
            </Stack>
        </>
    );
}
