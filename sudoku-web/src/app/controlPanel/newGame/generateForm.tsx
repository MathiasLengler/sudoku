import React, { useState } from "react";
import Typography from "@mui/material/Typography";
import Slider from "@mui/material/Slider";
import range from "lodash/range";
import { WasmSudokuController } from "../../wasmSudokuController";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";

const BASE_MIN = 2;
const BASE_MAX = 5;
const BASE_MARKS = range(BASE_MIN, BASE_MAX + 1).map(base => {
    const sideLength = Math.pow(base, 2);
    return {
        value: base,
        label: `${sideLength}x${sideLength}`,
    };
});

interface GenerateFormProps {
    sudokuController: WasmSudokuController;
    onClose: () => void;
}

export const GenerateForm: React.FunctionComponent<GenerateFormProps> = props => {
    const { sudokuController, onClose } = props;

    const [loading, setLoading] = useState(false);
    const [base, setBase] = useState(3);
    const [minGivens, setMinGivens] = useState(0);
    const cellCount = Math.pow(base, 4);

    if (cellCount < minGivens) {
        setMinGivens(cellCount);
    }

    return (
        <>
            <Box p={3}>
                <Typography gutterBottom>Size</Typography>
                <Slider
                    value={base}
                    onChange={(e, base) => setBase(base as number)}
                    valueLabelDisplay="auto"
                    step={null}
                    min={BASE_MIN}
                    max={BASE_MAX}
                    marks={BASE_MARKS}
                    disabled={loading}
                />
                <Typography gutterBottom>Minimum number of givens</Typography>
                <Slider
                    value={minGivens}
                    onChange={(e, minGivens) => setMinGivens(minGivens as number)}
                    valueLabelDisplay="auto"
                    step={1}
                    min={0}
                    max={cellCount}
                    marks={[
                        { value: 0, label: 0 },
                        { value: cellCount, label: cellCount },
                    ]}
                    disabled={loading}
                />
            </Box>
            <DialogActions>
                {loading && <CircularProgress />}
                <Button onClick={onClose} disabled={loading}>
                    Cancel
                </Button>
                <Button
                    color="primary"
                    disabled={loading}
                    onClick={async () => {
                        setLoading(true);

                        try {
                            await sudokuController.generate({
                                base,
                                target: {
                                    fromFilled: {
                                        distance: cellCount - minGivens,
                                    },
                                },
                            });
                            onClose();
                        } finally {
                            setLoading(false);
                        }
                    }}
                >
                    Generate
                </Button>
            </DialogActions>
        </>
    );
};
