import { useRecoilState } from "recoil";
import { useForm } from "react-hook-form-mui";
import { zodResolver } from "@hookform/resolvers/zod";
import React from "react";
import { Box, Button, DialogActions, DialogContent, DialogTitle } from "@mui/material";
import CircularProgress from "@mui/material/CircularProgress";
import { type HintSettings, hintSettingsSchema, hintSettingsState } from "../../state/forms/hintSettings";
import SelectStrategies from "../../components/formFragments/SelectStrategies";

interface HintSettingsDialogProps {
    onClose: () => void;
}

export function HintSettingsDialog({ onClose }: HintSettingsDialogProps) {
    const [hintSettingsFormValues, setHintSettingsFormValues] = useRecoilState(hintSettingsState);
    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
    } = useForm<HintSettings>({
        values: hintSettingsFormValues,
        resolver: zodResolver(hintSettingsSchema),
    });

    return (
        <>
            <DialogTitle>Hint settings</DialogTitle>
            <DialogContent dividers>
                <form
                    id="hint-settings-form"
                    noValidate
                    onSubmit={handleSubmit(async hintSettings => {
                        setHintSettingsFormValues(hintSettings);
                        onClose();
                    })}
                    style={{ display: "sticky" }}
                >
                    <SelectStrategies control={control} name="strategies" />
                </form>
            </DialogContent>
            <DialogActions>
                {isSubmitting && (
                    <Box>
                        <CircularProgress />
                    </Box>
                )}
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="hint-settings-form"
                    color="primary"
                    variant="contained"
                    disabled={isSubmitting}
                >
                    Save settings
                </Button>
            </DialogActions>
        </>
        // </form>
    );
}
