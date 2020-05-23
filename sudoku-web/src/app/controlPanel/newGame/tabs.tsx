import React from 'react';
import SwipeableViews from 'react-swipeable-views';
import AppBar from '@material-ui/core/AppBar';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import {GenerateForm} from "./generateForm";
import {WasmSudokuController} from "../../wasmSudokuController";
import {ImportForm} from "./importForm";

interface NewGameTabsProps {
  sudokuController: WasmSudokuController;
  onClose: () => void;
}

export const NewGameTabs: React.FunctionComponent<NewGameTabsProps> = (props) => {
  const {sudokuController, onClose} = props;

  const [tabIndex, setTabIndex] = React.useState(0);

  const handleChange = (event: React.ChangeEvent<unknown>, newValue: number) => {
    setTabIndex(newValue);
  };

  return (
    <div>
      <AppBar position="static" color="default">
        <Tabs
          value={tabIndex}
          onChange={handleChange}
          indicatorColor="primary"
          textColor="primary"
          variant="fullWidth"
          centered
        >
          <Tab label="Generate"/>
          <Tab label="Import"/>
        </Tabs>
      </AppBar>
      <SwipeableViews
        index={tabIndex}
        disabled
      >
        <GenerateForm onClose={onClose} sudokuController={sudokuController}/>
        <ImportForm onClose={onClose} sudokuController={sudokuController}/>
      </SwipeableViews>
    </div>
  );
};
