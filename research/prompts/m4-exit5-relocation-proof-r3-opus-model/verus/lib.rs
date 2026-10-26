// 攻方腿 r3 的攻击 crate：把正推（Sonnet r2）那份改后协议的两个模块原样引进来
// （gc_ring_proofs = 根环 / 回退窗口版，relocation_interval_proofs = 区间版 P4），
// 另引冻结副本两份基线（gc_crash_proofs、compaction_plan_proofs）作对照，一个字都不改。
// attack 模块在这两份正推模块上复跑第二轮攻方两个反例的变体，看改后协议到底排除了哪一形。
#![allow(unused_imports)]
pub mod compaction_plan_proofs;
pub mod gc_crash_proofs;
pub mod gc_ring_proofs;
pub mod relocation_interval_proofs;
pub mod attack;
