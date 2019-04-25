import {TypescriptTest} from "../crate/pkg";

import("../crate/pkg").then(module => {
  module.run();

  const ctrl = module.get_sudoku_controller();

  ctrl.say_hello();

  const sudoku = ctrl.get_sudoku();

  const test = ctrl.test_typescript() as TypescriptTest;

  switch (test.tag) {
    case "V1":
      console.log("V1");
      console.log(test.fields.Foo);
      break;
    case "V2":
      console.log("V2");
      break;
    case "V3":
      console.log("V3");
      break;
  }

  console.log(test);
});
