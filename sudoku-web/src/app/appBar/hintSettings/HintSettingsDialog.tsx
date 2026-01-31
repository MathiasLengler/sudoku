import { zodResolver } from "@hookform/resolvers/zod";
import SaveIcon from "@mui/icons-material/Save";
import { Box, Button, DialogActions, DialogContent, DialogTitle } from "@mui/material";
import { Stack } from "@mui/material";
import { RadioButtonGroup, SliderElement, SwitchElement, useForm } from "react-hook-form-mui";
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
    } = useForm({
        values: hintSettingsSchema.encode(hintSettingsFormValues),
        resolver: zodResolver(hintSettingsSchema),
    });

    const [mode] = watch(["mode"]);
    return (
        <>
            <DialogTitle>Hint settings</DialogTitle>
            <DialogContent>
                <form
                    id="hint-settings-form"
                    noValidate
                    onSubmit={handleSubmit((hintSettings) => {
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
                                    name="loopDelayIndex"
                                    label="Loop delay"
                                    step={1}
                                    min={0}
                                    max={MAX_LOOP_DELAY_INDEX}
                                    marks={[0, MAX_LOOP_DELAY_INDEX].map((loopDelayMs) => ({
                                        value: loopDelayMs,
                                        label: formatDurationMs(scaleLoopDelayIndex(loopDelayMs)),
                                    }))}
                                    scale={scaleLoopDelayIndex}
                                    valueLabelDisplay="auto"
                                    valueLabelFormat={(loopDelayMs) => formatDurationMs(loopDelayMs)}
                                    getAriaLabel={() => "Delay"}
                                    getAriaValueText={(loopDelayMs) => formatDurationMs(loopDelayMs)}
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
                <Button
                    type="submit"
                    form="hint-settings-form"
                    color="primary"
                    variant="contained"
                    endIcon={<SaveIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    <span>Save settings</span>
                </Button>
            </DialogActions>
        </>
    );
}
