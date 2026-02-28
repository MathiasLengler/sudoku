import { zodResolver } from "@hookform/resolvers/zod";
import SaveIcon from "@mui/icons-material/Save";
import { Box, Button, DialogActions, DialogContent, DialogTitle, Typography } from "@mui/material";
import { Stack } from "@mui/material";
import { RadioButtonGroup, SliderElement, SwitchElement, useForm } from "react-hook-form-mui";
import { useAtom } from "jotai";
import { Fieldset } from "../../components/Fieldset";
import { ResetFormButton } from "../../components/ResetFormButton";
import {
    DEFAULT_APP_SETTINGS,
    appSettingsSchema,
    appSettingsState,
    type AppSettings,
    type ColorMode,
} from "../../state/forms/appSettings";

type SettingsDialogProps = {
    onClose: () => void;
};

export function SettingsDialog({ onClose }: SettingsDialogProps) {
    const [appSettings, setAppSettings] = useAtom(appSettingsState);

    const {
        control,
        handleSubmit,
        formState: { isSubmitting },
        reset,
    } = useForm<AppSettings>({
        values: appSettings,
        resolver: zodResolver(appSettingsSchema),
    });

    return (
        <>
            <DialogTitle>Settings</DialogTitle>
            <DialogContent>
                <form
                    id="settings-form"
                    noValidate
                    onSubmit={handleSubmit((settings) => {
                        setAppSettings(settings);
                        onClose();
                    })}
                    style={{ display: "sticky" }}
                >
                    <Stack spacing={2}>
                        <Fieldset label="Theme">
                            <Box sx={{ mx: 2 }}>
                                <SliderElement
                                    control={control}
                                    name="themeColorHue"
                                    label="Theme color hue"
                                    min={0}
                                    max={360}
                                    step={1}
                                    valueLabelDisplay="auto"
                                    sx={{
                                        "& .MuiSlider-track": {
                                            background:
                                                "linear-gradient(to right, hsl(0, 70%, 50%), hsl(60, 70%, 50%), hsl(120, 70%, 50%), hsl(180, 70%, 50%), hsl(240, 70%, 50%), hsl(300, 70%, 50%), hsl(360, 70%, 50%))",
                                        },
                                        "& .MuiSlider-rail": {
                                            background:
                                                "linear-gradient(to right, hsl(0, 70%, 50%), hsl(60, 70%, 50%), hsl(120, 70%, 50%), hsl(180, 70%, 50%), hsl(240, 70%, 50%), hsl(300, 70%, 50%), hsl(360, 70%, 50%))",
                                            opacity: 1,
                                        },
                                    }}
                                />
                            </Box>
                            <RadioButtonGroup
                                control={control}
                                name="colorMode"
                                label="Color mode"
                                options={
                                    [
                                        { id: "auto", label: "Auto (system)" },
                                        { id: "light", label: "Light" },
                                        { id: "dark", label: "Dark" },
                                    ] satisfies { id: ColorMode; label: string }[]
                                }
                                required
                            />
                        </Fieldset>

                        <Fieldset label="Game behavior">
                            <SwitchElement
                                control={control}
                                name="valueHintingInStickyMode"
                                label="Value hinting in sticky mode"
                            />
                            <SwitchElement
                                control={control}
                                name="removeCandidatesOnSetValue"
                                label="Remove candidates on set value"
                            />
                            <SwitchElement
                                control={control}
                                name="highlightConflictIncorrectValue"
                                label="Highlight conflict/incorrect value"
                            />
                            <SwitchElement
                                control={control}
                                name="highlightMissingNote"
                                label="Highlight missing note"
                            />
                            <SwitchElement
                                control={control}
                                name="highlightStickyCandidates"
                                label="Highlight sticky value candidates"
                            />
                            <SwitchElement
                                control={control}
                                name="switchStickyValueOnTapGiven"
                                label="Switch sticky value when tapping given value"
                            />
                        </Fieldset>

                        <Fieldset label="User interface">
                            <SwitchElement control={control} name="showTimer" label="Show timer" />
                            <SwitchElement control={control} name="inputNumberBlock" label="Input number block" />
                            <Typography variant="caption" color="text.secondary" sx={{ pl: 4 }}>
                                Takes more space, but useful for larger bases
                            </Typography>
                        </Fieldset>
                    </Stack>
                </form>
            </DialogContent>
            <DialogActions>
                <ResetFormButton disabled={isSubmitting} onClick={() => reset(DEFAULT_APP_SETTINGS)} />
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="settings-form"
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
