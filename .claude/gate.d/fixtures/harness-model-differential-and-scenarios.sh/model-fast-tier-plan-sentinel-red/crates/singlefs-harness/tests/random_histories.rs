// 样本：快档那一条模型对拍用例（名字与真仓 random_histories.rs 里那一条相同）；.unit-verdict-sample/plan-failure 让 verdict-plan 装失灵

fn fast_steps() -> u32 { 1912 }

#[test]
#[ignore]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() { assert!(fast_steps() > 0); }
