"""调查员甲补丁那一份副本：只装探针要的符号（候选开关不起作用，判法照补丁）。"""
import sys
path = sys.argv[1]
s = open(path, encoding="utf-8").read()
anchor = "/// I-7.18（根记录的回收门槛等于现算的门槛） 与 I-7.19（钉着的槽不住本实例写的根）：只判最新根，只判建账的发布\n"
assert s.count(anchor) == 1
helpers = '''/// 攻方腿原型：探针要的符号；这一份副本的判法照调查员甲补丁，候选开关不起作用。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I718Candidate {
    CurrentOrBing,
    Jia,
    YiLiteral,
    YiAfterTheFirstLap,
    JiaAnyLapOfTheNextSlot,
    JiaAnyUnreadableSlot,
    DingJiaAnyLapWithTheNextRecord,
    DingYiWithTheNextRecord,
}
thread_local! {
    pub static I718_PROBE_COUNTERS: std::cell::RefCell<(u64, u64)> = const { std::cell::RefCell::new((0, 0)) };
}
pub fn set_i718_candidate_for_the_attack_probe(_candidate: I718Candidate) {}

'''
s = s.replace(anchor, helpers + anchor)
open(path, "w", encoding="utf-8").write(s)
print("helpers only")
