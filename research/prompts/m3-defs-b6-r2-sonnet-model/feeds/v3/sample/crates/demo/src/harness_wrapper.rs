use singlefs_harness::numeric_environment_variable::process_environment_variable;

pub fn threads() -> usize {
    process_environment_variable("SAMPLE_HARNESS_WRAPPER_THREADS").expect("set it").parse().unwrap()
}
