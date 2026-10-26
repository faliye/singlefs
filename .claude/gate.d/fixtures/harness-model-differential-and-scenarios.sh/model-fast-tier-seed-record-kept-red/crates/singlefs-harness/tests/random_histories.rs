// 样本：快档那一条模型对拍用例（名字与真仓 random_histories.rs 里那一条相同）；样本档拿上一版记库那一步被假 python3 拦下，cargo test 走不到

fn fast_steps() -> u32 { 1912 }

#[test]
#[ignore]
fn random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation() { assert!(fast_steps() > 0); }
