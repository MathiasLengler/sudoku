import React, {useState} from "react";
import Typography from "@mui/material/Typography";
import Slider from "@mui/material/Slider";
import range from "lodash/range";
import {WasmSudokuController} from "../../wasmSudokuController";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import Box from "@mui/material/Box";
import CircularProgress from "@mui/material/CircularProgress";

const baseMin = 2;
const baseMax = 5;
const baseMarks = range(baseMin, baseMax + 1).map(base => {
  const sideLength = Math.pow(base, 2);
  return {
    value: base,
    label: `${sideLength}x${sideLength}`,
  }
});

interface GenerateFormProps {
  sudokuController: WasmSudokuController;
  onClose: () => void;
}

export const GenerateForm: React.FunctionComponent<GenerateFormProps> = (props) => {
  const {sudokuController, onClose} = props;

  const [loading, setLoading] = useState(false);
  const [base, setBase] = useState(3);
  const [distance, setDistance] = useState(0);
  const maxDistance = Math.pow(base, 4);

  if (maxDistance < distance) {
    setDistance(maxDistance);
  }

  return <>
    <Box p={3}>
      <Typography gutterBottom>
        Grid Size
      </Typography>
      <Slider
        value={base}
        onChange={(e, base) => setBase(base as number)}
        valueLabelDisplay="auto"
        step={null}
        min={baseMin}
        max={baseMax}
        marks={baseMarks}
        disabled={loading}
      />
      <Typography gutterBottom>
        Additional clues (starting from minimal Sudoku)
      </Typography>
      <Slider
        value={distance}
        onChange={(e, distance) => setDistance(distance as number)}
        valueLabelDisplay="auto"
        step={1}
        min={0}
        max={maxDistance}
        marks={[{value: 0, label: 0}, {value: maxDistance, label: maxDistance}]}
        disabled={loading}
      />
    </Box>
    <DialogActions>
      {loading && <CircularProgress/>}
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
                fromMinimal: {
                  distance
                }
              }
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
  </>;
};
