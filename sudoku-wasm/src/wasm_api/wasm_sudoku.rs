use crate::error::Result;
use crate::typescript::*;
use sudoku::base::BaseEnum;
use sudoku::error::Error as SudokuError;
use sudoku::transport::TransportSudoku;
use sudoku::{DynamicSudoku, DynamicSudokuActions};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmSudoku {
    sudoku: DynamicSudoku,
}

impl Default for WasmSudoku {
    fn default() -> Self {
        DynamicSudoku::new(BaseEnum::Base3).into()
    }
}

impl From<DynamicSudoku> for WasmSudoku {
    fn from(sudoku: DynamicSudoku) -> Self {
        Self { sudoku }
    }
}

/// Constructors
#[wasm_bindgen]
impl WasmSudoku {
    #[wasm_bindgen(constructor)]
    pub fn default() -> Self {
        Default::default()
    }

    pub fn new(base: IBaseEnum) -> Result<Self> {
        Ok(DynamicSudoku::new(import_base_enum(base)?).into())
    }

    #[wasm_bindgen(js_name = fromDynamicGrid)]
    pub fn from_dynamic_grid(dynamic_grid: IDynamicGrid) -> Result<Self> {
        let dynamic_grid = import_dynamic_grid(dynamic_grid)?;

        Ok(DynamicSudoku::try_from(dynamic_grid)?.into())
    }

    pub fn generate(
        generator_settings: IDynamicGeneratorSettings,
        on_progress: IGenerateOnProgress,
    ) -> Result<Self> {
        Ok(DynamicSudoku::generate(
            import_dynamic_generator_settings(generator_settings)?,
            import_generate_on_progress(on_progress)?,
        )?
        .into())
    }

    #[wasm_bindgen(js_name = generateMultiShot)]
    pub fn generate_multi_shot(
        multi_shot_generator_settings: IDynamicMultiShotGeneratorSettings,
        on_progress: IGenerateMultiShotOnProgress,
    ) -> Result<Self> {
        Ok(DynamicSudoku::generate_multi_shot(
            import_dynamic_multi_shot_generator_settings(multi_shot_generator_settings)?,
            import_generate_multi_shot_on_progress(on_progress)?,
        )?
        .into())
    }

    pub fn import(input: &str) -> Result<Self> {
        Ok(DynamicSudoku::import(input)?.into())
    }
}

#[wasm_bindgen]
impl WasmSudoku {
    pub fn equals(&self, other: &Self) -> bool {
        self.sudoku == other.sudoku
    }

    #[wasm_bindgen(js_name = getTransportSudoku)]
    pub fn get_transport_sudoku(&self) -> Result<ITransportSudoku> {
        let transport_sudoku = TransportSudoku::from(&self.sudoku);

        export_transport_sudoku(transport_sudoku)
    }

    #[wasm_bindgen(js_name = toDynamicGrid)]
    pub fn to_dynamic_grid(&self) -> Result<IDynamicGrid> {
        export_dynamic_grid(self.sudoku.to_dynamic_grid())
    }

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&mut self, pos: IDynamicPosition, value: IDynamicValue) -> Result<()> {
        self.sudoku
            .set_value(import_dynamic_position(pos)?, import_dynamic_value(value)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setOrToggleValue)]
    pub fn set_or_toggle_value(
        &mut self,
        pos: IDynamicPosition,
        value: IDynamicValue,
    ) -> Result<()> {
        self.sudoku
            .set_or_toggle_value(import_dynamic_position(pos)?, import_dynamic_value(value)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setCandidates)]
    pub fn set_candidates(
        &mut self,
        pos: IDynamicPosition,
        candidates: IDynamicCandidates,
    ) -> Result<()> {
        self.sudoku.set_candidates(
            import_dynamic_position(pos)?,
            import_dynamic_candidates(candidates)?,
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = toggleCandidate)]
    pub fn toggle_candidate(
        &mut self,
        pos: IDynamicPosition,
        candidate: IDynamicValue,
    ) -> Result<()> {
        self.sudoku.toggle_candidate(
            import_dynamic_position(pos)?,
            import_dynamic_value(candidate)?,
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setCandidate)]
    pub fn set_candidate(&mut self, pos: IDynamicPosition, candidate: IDynamicValue) -> Result<()> {
        self.sudoku.set_candidate(
            import_dynamic_position(pos)?,
            import_dynamic_value(candidate)?,
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteCandidate)]
    pub fn delete_candidate(
        &mut self,
        pos: IDynamicPosition,
        candidate: IDynamicValue,
    ) -> Result<()> {
        self.sudoku.delete_candidate(
            import_dynamic_position(pos)?,
            import_dynamic_value(candidate)?,
        )?;
        Ok(())
    }

    pub fn delete(&mut self, pos: IDynamicPosition) -> Result<()> {
        self.sudoku.delete(import_dynamic_position(pos)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setAllDirectCandidates)]
    pub fn set_all_direct_candidates(&mut self) {
        self.sudoku.set_all_direct_candidates();
    }

    pub fn undo(&mut self) {
        self.sudoku.undo();
    }

    pub fn redo(&mut self) {
        self.sudoku.redo();
    }

    pub fn export(&self, format: IGridFormatEnum) -> Result<String> {
        Ok(self.sudoku.export(import_grid_format_enum(format)?))
    }

    #[wasm_bindgen(js_name = tryStrategies)]
    pub fn try_strategies(
        &mut self,
        strategies: IStrategySet,
    ) -> Result<Option<IDynamicSolveStep>> {
        let opt_dyn_solve_step = self
            .sudoku
            .try_strategies(import_strategy_set(strategies)?)?;

        opt_dyn_solve_step
            .map(export_dynamic_solve_step)
            .transpose()
    }

    #[wasm_bindgen(js_name = applyDeductions)]
    pub fn apply_deductions(&mut self, deductions: ITransportDeductions) -> Result<()> {
        self.sudoku
            .apply_deductions(import_transport_deductions(deductions)?)?;
        Ok(())
    }
}

/// (De)serialization
#[wasm_bindgen]
impl WasmSudoku {
    pub fn serialize(&self) -> Result<ISerializedDynamicSudoku> {
        let vec = postcard::to_stdvec(&self.sudoku).map_err(SudokuError::from)?;
        Ok(JsValue::from(vec).into())
    }

    pub fn deserialize(
        #[wasm_bindgen(unchecked_param_type = "SerializedDynamicSudoku")] bytes: &[u8],
    ) -> Result<Self> {
        let sudoku: DynamicSudoku = postcard::from_bytes(bytes).map_err(SudokuError::from)?;
        Ok(Self { sudoku })
    }
}
