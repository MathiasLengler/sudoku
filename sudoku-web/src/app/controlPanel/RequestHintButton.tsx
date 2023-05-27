import LightbulbIcon from "@mui/icons-material/Lightbulb";
import MyIconButton from "../components/MyIconButton";
import * as React from "react";
import { useTryStrategies } from "../sudokuActions";
import { useRecoilValue, useSetRecoilState } from "recoil";
import { hintSettingsState } from "../state/forms/hintSettings";
import { hintState } from "../state/hint";

export function RequestHintButton() {
    // TODO: implement extended hintSettings
    const hintSettings = useRecoilValue(hintSettingsState);
    const tryStrategies = useTryStrategies();

    const setSolverHint = useSetRecoilState(hintState);

    const hintStrategies = async () => {
        const tryStrategiesResult = await tryStrategies(hintSettings.strategies);
        if (!tryStrategiesResult) {
            setSolverHint({ enabled: false });
            return;
        }
        const [strategy, { deductions }] = tryStrategiesResult;
        console.info(`Strategy ${strategy} made progress:`, deductions);

        setSolverHint({ enabled: true, strategy, deductions });
    };

    return (
        <MyIconButton
            tooltip="Request Hint [TODO]"
            icon={LightbulbIcon}
            size="large"
            onClick={async () => {
                await hintStrategies();
            }}
        />
    );
}
