import { zodResolver } from "@hookform/resolvers/zod";
import { IconDeviceFloppy } from "@tabler/icons-react";
import { Box, Button, Group, Radio, Slider, Stack, Switch, Text } from "@mantine/core";
import { Controller, useForm } from "react-hook-form";
import { useAtom } from "jotai";
import { Fieldset } from "../../components/Fieldset";
import { ResetFormButton } from "../../components/ResetFormButton";
import SelectStrategies from "../../components/formFragments/SelectStrategies";
import { formatDurationMs } from "../../i18n";
import {
    DEFAULT_HINT_SETTINGS,
    MAX_LOOP_DELAY_INDEX,
    hintSettingsSchema,
    hintSettingsState,
    scaleLoopDelayIndex,
    type HintSettings,
} from "../../state/forms/hintSettings";

type HintSettingsDialogProps = {
    onClose: () => void;
};

export function HintSettingsDialog({ onClose }: HintSettingsDialogProps) {
    const [hintSettingsFormValues, setHintSettingsFormValues] = useAtom(hintSettingsState);

    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
        reset,
    } = useForm<HintSettings>({
        values: hintSettingsFormValues,
        resolver: zodResolver(hintSettingsSchema),
    });

    const [mode] = watch(["mode"]);
    return (
        <>
            <Text size="lg" fw={500} mb="md">
                Hint settings
            </Text>
            <form
                id="hint-settings-form"
                noValidate
                onSubmit={handleSubmit((hintSettings) => {
                    setHintSettingsFormValues(hintSettings);
                    onClose();
                })}
            >
                <Stack gap="md">
                    <SelectStrategies control={control} name="strategies" />
                    <Controller
                        name="mode"
                        control={control}
                        render={({ field }) => (
                            <Radio.Group
                                {...field}
                                label="Mode"
                                required
                            >
                                <Stack gap="xs" mt="xs">
                                    <Radio value="toggleHint" label="Toggle hint" />
                                    <Radio value="hintApply" label="Show hint, then apply it" />
                                    <Radio value="apply" label="Apply hint directly" />
                                </Stack>
                            </Radio.Group>
                        )}
                    />
                    <Fieldset label="Loop" disabled={mode === "toggleHint"}>
                        <Controller
                            name="doLoop"
                            control={control}
                            render={({ field: { value, onChange, ...field } }) => (
                                <Switch
                                    {...field}
                                    checked={value}
                                    onChange={(e) => onChange(e.currentTarget.checked)}
                                    label="Loop until strategies make no further progress"
                                />
                            )}
                        />
                        <Box px="md" mt="md">
                            <Controller
                                name="loopDelayIndex"
                                control={control}
                                render={({ field: { value, onChange } }) => (
                                    <Slider
                                        value={value}
                                        onChange={onChange}
                                        label={(v) => formatDurationMs(scaleLoopDelayIndex(v))}
                                        step={1}
                                        min={0}
                                        max={MAX_LOOP_DELAY_INDEX}
                                        marks={[
                                            { value: 0, label: formatDurationMs(scaleLoopDelayIndex(0)) },
                                            { value: MAX_LOOP_DELAY_INDEX, label: formatDurationMs(scaleLoopDelayIndex(MAX_LOOP_DELAY_INDEX)) },
                                        ]}
                                    />
                                )}
                            />
                        </Box>
                    </Fieldset>
                    <Fieldset label="Deductions">
                        <Controller
                            name="multipleDeductions"
                            control={control}
                            render={({ field: { value, onChange, ...field } }) => (
                                <Switch
                                    {...field}
                                    checked={value}
                                    onChange={(e) => onChange(e.currentTarget.checked)}
                                    label="Apply multiple deductions"
                                />
                            )}
                        />
                    </Fieldset>
                </Stack>
            </form>
            <Group justify="space-between" mt="md">
                <ResetFormButton disabled={isSubmitting} onClick={() => reset(DEFAULT_HINT_SETTINGS)} />
                <Button onClick={onClose} disabled={isSubmitting} variant="subtle">
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="hint-settings-form"
                    rightSection={<IconDeviceFloppy size={18} />}
                    loading={isSubmitting}
                >
                    Save settings
                </Button>
            </Group>
        </>
    );
}
