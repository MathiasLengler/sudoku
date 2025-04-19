#[cfg(feature = "log")]
pub(crate) fn init_test_logger() {
    use env_logger::Env;

    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
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
        #[test]
        fn test_base4() {
            type Base = $crate::base::consts::Base4;
            $using_base
        }
    };
}

macro_rules! test_all_bases {
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
        #[test]
        fn test_base4() {
            type Base = $crate::base::consts::Base4;
            $using_base
        }
        #[test]
        fn test_base5() {
            type Base = $crate::base::consts::Base5;
            $using_base
        }
    };
}
pub(crate) use test_all_bases;
pub(crate) use test_max_base3;
pub(crate) use test_max_base4;
