import { useRecoilState } from "recoil";
import { RadioButtonGroup, SliderElement, SwitchElement, useForm } from "react-hook-form-mui";
import { zodResolver } from "@hookform/resolvers/zod";
import React from "react";
import { Box, Button, DialogActions, DialogContent, DialogTitle, Stack } from "@mui/material";
import {
    DEFAULT_HINT_SETTINGS,
    type HintSettings,
    hintSettingsSchema,
    hintSettingsState,
    MAX_LOOP_DELAY_MS,
} from "../../state/forms/hintSettings";
import SelectStrategies from "../../components/formFragments/SelectStrategies";
import { formatDurationMs } from "../../i18n";
import _ from "lodash";
import { LoadingButton } from "@mui/lab";
import SaveIcon from "@mui/icons-material/Save";
import { Fieldset } from "../../components/Fieldset";
import { ResetFormButton } from "../../components/ResetFormButton";

interface HintSettingsDialogProps {
    onClose: () => void;
}

function scaleDurationMs(n: number) {
    if (n === 0) {
        return 0;
    }
    if (n <= 10) {
        return 2 ** (n - 1);
    }
    return 1000 * 2 ** (n - 11);
}

console.log(
    _.range(0, 20).map(n => ({
        n,
        scaleDurationMs: scaleDurationMs(n),
        nRoundtrip: unscaleDurationMs(scaleDurationMs(n)),
    }))
);

function unscaleDurationMs(n: number) {
    if (n === 0) {
        return 0;
    }
    if (n < 1000) {
        return Math.log2(n) + 1;
    }
    return Math.log2(n / 1000) + 11;
}

export function HintSettingsDialog({ onClose }: HintSettingsDialogProps) {
    const [hintSettingsFormValues, setHintSettingsFormValues] = useRecoilState(hintSettingsState);

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

    const mode = watch("mode");
    const doLoop = watch("doLoop");
    return (
        <>
            <DialogTitle>Hint settings</DialogTitle>
            <DialogContent>
                <form
                    id="hint-settings-form"
                    noValidate
                    onSubmit={handleSubmit(async hintSettings => {
                        setHintSettingsFormValues(hintSettings);
                        onClose();
                    })}
                    style={{ display: "sticky" }}
                >
                    <Stack spacing={2}>
                        <SelectStrategies control={control} name="strategies" />
                        <RadioButtonGroup
                            control={control}
                            name="mode"
                            label="Mode"
                            options={
                                [
                                    {
                                        id: "toggleHint",
                                        label: "Toggle hint",
                                    },
                                    {
                                        id: "hintApply",
                                        label: "Show hint, then apply it",
                                    },
                                    {
                                        id: "apply",
                                        label: "Apply hint directly",
                                    },
                                ] satisfies { id: HintSettings["mode"]; label: string }[]
                            }
                            required
                        />
                        <Fieldset label="Loop" disabled={mode === "toggleHint"}>
                            <SwitchElement
                                control={control}
                                name="doLoop"
                                label="Loop until strategies make no further progress"
                            />
                            <Box sx={{ mx: 2 }}>
                                <SliderElement
                                    control={control}
                                    name="loopDelayMs"
                                    label="Loop delay"
                                    disabled={mode === "toggleHint" || !doLoop}
                                    step={1}
                                    min={0}
                                    max={unscaleDurationMs(MAX_LOOP_DELAY_MS)}
                                    marks={[0, unscaleDurationMs(MAX_LOOP_DELAY_MS)].map(loopDelayMs => ({
                                        value: loopDelayMs,
                                        label: formatDurationMs(scaleDurationMs(loopDelayMs)),
                                    }))}
                                    scale={scaleDurationMs}
                                    valueLabelDisplay="auto"
                                    valueLabelFormat={loopDelayMs => formatDurationMs(loopDelayMs)}
                                    getAriaLabel={() => "Delay"}
                                    getAriaValueText={loopDelayMs => formatDurationMs(loopDelayMs)}
                                />
                            </Box>
                        </Fieldset>
                        <Fieldset label="Deductions">
                            <SwitchElement
                                control={control}
                                name="multipleDeductions"
                                label="Apply multiple deductions"
                            />
                        </Fieldset>
                    </Stack>
                </form>
            </DialogContent>
            <DialogActions>
                <ResetFormButton disabled={isSubmitting} onClick={() => reset(DEFAULT_HINT_SETTINGS)} />
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <LoadingButton
                    type="submit"
                    form="hint-settings-form"
                    color="primary"
                    variant="contained"
                    endIcon={<SaveIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    <span>Save settings</span>
                </LoadingButton>
            </DialogActions>
        </>
        // </form>
    );
}
