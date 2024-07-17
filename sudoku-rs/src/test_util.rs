#[cfg(feature = "log")]
pub(crate) fn init_test_logger() {
    use env_logger::Env;

    let _ = env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .is_test(true)
        .try_init();
}

#[cfg(not(feature = "log"))]
pub(crate) fn init_logger() {}

macro_rules! for_all_bases {
    ($using_base:expr) => {
        use $crate::base::consts::*;

        {
            type Base = Base2;
            $using_base
        }
        {
            type Base = Base3;
            $using_base
        }
        {
            type Base = Base4;
            $using_base
        }
        {
            type Base = Base5;
            $using_base
        }
    };
}
pub(crate) use for_all_bases;
