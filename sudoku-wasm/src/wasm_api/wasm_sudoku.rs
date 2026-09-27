use crate::error::Result;
use crate::generic_instances::WasmStrategySet;
use crate::serialized::{self, ISerializedDynamicSudoku};
use crate::typescript::*;
use sudoku::base::BaseEnum;
use sudoku::cell::dynamic::{DynamicCandidates, DynamicValue};
use sudoku::generator::DynamicGeneratorSettings;
use sudoku::generator::multi_shot::DynamicMultiShotGeneratorSettings;
use sudoku::grid::dynamic::DynamicGrid;
use sudoku::grid::format::GridFormatEnum;
use sudoku::position::DynamicPosition;
use sudoku::solver::strategic::DynamicSolveStep;
use sudoku::solver::strategic::deduction::transport::TransportDeductions;
use sudoku::transport::TransportSudoku;
use sudoku::{DynamicSudoku, DynamicSudokuActions};
use tsify::Ts;
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

    pub fn new(base: Ts<BaseEnum>) -> Result<Self> {
        Ok(DynamicSudoku::new(base.to_rust()?).into())
    }

    #[wasm_bindgen(js_name = fromDynamicGrid)]
    pub fn from_dynamic_grid(dynamic_grid: Ts<DynamicGrid>) -> Result<Self> {
        let dynamic_grid = dynamic_grid.to_rust()?;

        Ok(DynamicSudoku::try_from(dynamic_grid)?.into())
    }

    pub fn generate(
        generator_settings: Ts<DynamicGeneratorSettings>,
        on_progress: IGenerateOnProgress,
    ) -> Result<Self> {
        Ok(DynamicSudoku::generate(
            generator_settings.to_rust()?,
            import_generate_on_progress(on_progress)?,
        )?
        .into())
    }

    #[wasm_bindgen(js_name = generateMultiShot)]
    pub fn generate_multi_shot(
        multi_shot_generator_settings: Ts<DynamicMultiShotGeneratorSettings>,
        on_progress: IGenerateMultiShotOnProgress,
    ) -> Result<Self> {
        Ok(DynamicSudoku::generate_multi_shot(
            multi_shot_generator_settings.to_rust()?,
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
    pub fn get_transport_sudoku(&self) -> Result<Ts<TransportSudoku>> {
        let transport_sudoku = TransportSudoku::from(&self.sudoku);

        export_ts(&transport_sudoku)
    }

    #[wasm_bindgen(js_name = toDynamicGrid)]
    pub fn to_dynamic_grid(&self) -> Result<Ts<DynamicGrid>> {
        export_ts(&self.sudoku.to_dynamic_grid())
    }

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&mut self, pos: Ts<DynamicPosition>, value: Ts<DynamicValue>) -> Result<()> {
        self.sudoku.set_value(pos.to_rust()?, value.to_rust()?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setOrToggleValue)]
    pub fn set_or_toggle_value(
        &mut self,
        pos: Ts<DynamicPosition>,
        value: Ts<DynamicValue>,
    ) -> Result<()> {
        self.sudoku
            .set_or_toggle_value(pos.to_rust()?, value.to_rust()?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setCandidates)]
    pub fn set_candidates(
        &mut self,
        pos: Ts<DynamicPosition>,
        candidates: Ts<DynamicCandidates>,
    ) -> Result<()> {
        self.sudoku
            .set_candidates(pos.to_rust()?, candidates.to_rust()?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = toggleCandidate)]
    pub fn toggle_candidate(
        &mut self,
        pos: Ts<DynamicPosition>,
        candidate: Ts<DynamicValue>,
    ) -> Result<()> {
        self.sudoku
            .toggle_candidate(pos.to_rust()?, candidate.to_rust()?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setCandidate)]
    pub fn set_candidate(
        &mut self,
        pos: Ts<DynamicPosition>,
        candidate: Ts<DynamicValue>,
    ) -> Result<()> {
        self.sudoku
            .set_candidate(pos.to_rust()?, candidate.to_rust()?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteCandidate)]
    pub fn delete_candidate(
        &mut self,
        pos: Ts<DynamicPosition>,
        candidate: Ts<DynamicValue>,
    ) -> Result<()> {
        self.sudoku
            .delete_candidate(pos.to_rust()?, candidate.to_rust()?)?;
        Ok(())
    }

    pub fn delete(&mut self, pos: Ts<DynamicPosition>) -> Result<()> {
        self.sudoku.delete(pos.to_rust()?)?;
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

    pub fn export(&self, format: Ts<GridFormatEnum>) -> Result<String> {
        Ok(self.sudoku.export(format.to_rust()?))
    }

    #[wasm_bindgen(js_name = tryStrategies)]
    pub fn try_strategies(
        &mut self,
        strategies: Ts<WasmStrategySet>,
    ) -> Result<Option<Ts<DynamicSolveStep>>> {
        let opt_dyn_solve_step = self.sudoku.try_strategies(strategies.to_rust()?.0)?;

        opt_dyn_solve_step.as_ref().map(export_ts).transpose()
    }

    #[wasm_bindgen(js_name = applyDeductions)]
    pub fn apply_deductions(&mut self, deductions: Ts<TransportDeductions>) -> Result<()> {
        self.sudoku.apply_deductions(deductions.to_rust()?)?;
        Ok(())
    }
}

/// (De)serialization
#[wasm_bindgen]
impl WasmSudoku {
    pub fn serialize(&self) -> Result<ISerializedDynamicSudoku> {
        serialized::serialize(&self.sudoku)
    }

    pub fn deserialize(bytes: &ISerializedDynamicSudoku) -> Result<Self> {
        Ok(serialized::deserialize::<DynamicSudoku>(bytes)?.into())
    }
}
