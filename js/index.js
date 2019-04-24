// TODO: typescript

import("../crate/pkg").then(module => {
  module.run();

  const ctrl = module.get_sudoku_controller();

  ctrl.say_hello();

  const sudoku = ctrl.get_sudoku();

  ctrl.test_typescript();

  debugger;

  console.log(sudoku);
});
