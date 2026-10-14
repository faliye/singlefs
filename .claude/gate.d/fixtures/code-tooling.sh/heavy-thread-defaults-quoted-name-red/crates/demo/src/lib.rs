#[cfg(test)]
mod tests;

pub mod use_alias;
pub mod vars_iter;

pub fn worker_threads() -> usize {
    std::env::var("SAMPLE_AFTER_TEST_DECL_THREADS").expect("set it").parse().unwrap()
}
