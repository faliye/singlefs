use super::*;

#[test]
fn linear_find_counts_the_step_that_finds_the_answer() {
    assert_eq!(
        linear_find_steps(Some(0), 50),
        1,
        "找到第一个也走一步（变异 m2）"
    );
    assert_eq!(linear_find_steps(Some(3), 50), 4);
    assert_eq!(linear_find_steps(None, 50), 50);
}
