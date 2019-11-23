import React, {useState} from "react";
import Typography from "@material-ui/core/Typography";
import Slider from "@material-ui/core/Slider";
import range from "lodash/range";
import {WasmSudokuController} from "../../wasmSudokuController";
import Button from "@material-ui/core/Button";
import DialogActions from "@material-ui/core/DialogActions";
import Box from "@material-ui/core/Box";

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
      />
    </Box>
    <DialogActions>
      <Button onClick={onClose}>
        Cancel
      </Button>
      <Button onClick={async () => {
        // TODO: show spinner
        await sudokuController.generate({
          base,
          target: {
            fromMinimal: {
              distance
            }
          }
        });
        onClose();
      }} color="primary">
        Generate
      </Button>
    </DialogActions>
  </>;
};
