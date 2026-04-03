#[cfg(feature = "log")]
pub(crate) fn init_test_logger() {
    use env_logger::Env;

    let _ = env_logger::Builder::from_env(Env::default().default_filter_or(
        "trace,sudoku::solver::strategic::strategies::impls::locked_sets::v2=info",
    ))
    .is_test(true)
    .try_init();
}

#[cfg(not(feature = "log"))]
pub(crate) fn init_logger() {}

macro_rules! test_max_base3 {
    ($using_base:block) => {
        #[test]
        fn test_base2() {
            type Base = $crate::base::consts::Base2;
            $using_base
        }
        #[test]
        fn test_base3() {
            type Base = $crate::base::consts::Base3;
            $using_base
        }
    };
}

macro_rules! test_max_base4 {
    ($using_base:block) => {
        $crate::test_util::test_max_base3!($using_base);
        #[test]
        fn test_base4() {
            type Base = $crate::base::consts::Base4;
            $using_base
        }
    };
}

macro_rules! test_max_base5 {
    ($using_base:block) => {
        $crate::test_util::test_max_base4!($using_base);
        #[test]
        fn test_base5() {
            type Base = $crate::base::consts::Base5;
            $using_base
        }
    };
}

macro_rules! for_base_grid_samples {
    (|$grid:ident, $name:ident| $block:block) => {
        #[allow(unused_mut)]
        for (i, mut $grid) in Base::grid_samples().enumerate() {
            let $name = format!("base_{}_sample_{i}", Base::BASE);
            $block
        }
    };
}

macro_rules! for_base_grid_samples_direct_candidates {
    (|$grid:ident, $name:ident| $block:block) => {
        #[allow(unused_mut)]
        for (i, mut $grid) in Base::grid_samples().enumerate() {
            let $name = format!("base_{}_sample_{i}_direct_candidates", Base::BASE);
            $grid.set_all_direct_candidates();
            $block
        }
    };
}

macro_rules! test_all_sample_grids {
    (|$grid:ident| $block:block) => {
        test_all_sample_grids!(|$grid, name| {
            println!("Testing {name}");
            $block
        });
    };
    (|$grid:ident, $name:ident| $block:block) => {
        mod direct_candidates {
            use super::*;

            $crate::test_util::test_max_base5!({
                $crate::test_util::for_base_grid_samples_direct_candidates!(|$grid, $name| {
                    $block
                })
            });
        }

        $crate::test_util::test_max_base5!({
            $crate::test_util::for_base_grid_samples!(|$grid, $name| { $block })
        });

        #[test]
        fn test_base_2_solved() {
            let $name = "base_2_solved".to_owned();
            #[allow(unused_mut)]
            let mut $grid = $crate::samples::base_2_solved();
            $block
        }
        #[test]
        fn test_base_2_candidates_coordinates() {
            let $name = "base_2_candidates_coordinates".to_owned();
            #[allow(unused_mut)]
            let mut $grid = $crate::samples::base_2_candidates_coordinates();
            $block
        }
    };
}

pub(crate) use for_base_grid_samples;
pub(crate) use for_base_grid_samples_direct_candidates;
pub(crate) use test_all_sample_grids;
pub(crate) use test_max_base3;
pub(crate) use test_max_base4;
pub(crate) use test_max_base5;
