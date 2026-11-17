"""scripts/mutation/crates-mutation-rows.py 的自证夹具：只被它的自证与弄坏开关的自证读的模块级常量（样本 crate 的源码、样本变异表的行）。
自证起子进程用 -c 跑的代码串（POOL_ERROR_CHILD_PROGRAM、STAGE_CHILD_PROGRAM）不放这里：算键库折不出经 globals() 放进来的名字。crates-mutation-rows.py 的 load_selftest_fixtures 只在自证开跑时按路径载入这一份，照 FIXTURE_VALUES 的名字放进那份模块；
判法单元的闭包里没有这一份，改这里的样本文本判法单元指纹不变（crates-mutation-rows.py 文件头「自证夹具」）。
运行期也读的常量（FAKE_CARGO、FAKE_LLVM_TOOL 这一类）不放这里。只被载入，没有命令行入口。
"""
import collections


SELFTEST_TESTS = '''
#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn negates() { assert_eq!(super::negate(3), -3); }
    #[test]
    fn halves() { assert_eq!(super::halve(6), 3); }
    #[test]
    fn reads_answer() {}
    #[test]
    fn gpu_parity() {}
}
'''

# 样本 crate：假 cargo 照「// TEST <名字> NEEDS <原文>」判红不红；下面的 #[test] 函数只给单元键认点名的测试是哪一个单元（文件头「行键」），跑不跑、红不红都照假 cargo
SELFTEST_SOURCE = '''// 本模块自证的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
''' + SELFTEST_TESTS

SELFTEST_TABLE_ROWS = [
    "add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- adds\tadds",
    "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib -- doubles\tdoubles",
    "negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --lib -- negates\tnegates",
    "halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --lib -- halves\thalves",
]

SELFTEST_READS_OUTSIDE_TEST = "// TEST reads_answer READS ../../fixture-data/answer.txt\n"

SELFTEST_READS_OUTSIDE_ROW = "reads-outside\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ 1 - value }\t-p tiny --lib -- reads_answer\treads_answer"

# 点名的测试照 GPU 对拍用例在没卡的机器上那样打一行 NOT_RUN_NO_GPU_ADAPTER 就返回成功：改坏的那一处它碰不到（文件头「本次未跑」）
SELFTEST_NOT_RUN_TEST = "// TEST gpu_parity NOTRUN 样本：本机没有 GPU 适配器\n"

SELFTEST_NOT_RUN_ROW = "card-path-changed\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 4 }\t-p tiny --lib -- gpu_parity\tgpu_parity"

# 点名的测试要编 gpu 特性的一行（真表里 GPU 判器那几批）：假 cargo 要读得到 .cargo/config.toml 的 [env] BINDGEN_EXTRA_CLANG_ARGS（FAKE_CARGO 头上那一段）
SELFTEST_GPU_FEATURE_ROW = "gpu-double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --features gpu --lib -- doubles\tdoubles"

# 产品 GPU 包的真显卡对拍开的是 tests-on-a-real-gpu（不是 gpu）：同样当上卡的特性认（GPU_TEST_FEATURES）
SELFTEST_PRODUCT_GPU_FEATURE_ROW = ("product-gpu-double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t"
                                    "-p tiny --features tests-on-a-real-gpu --lib -- doubles\tdoubles")

# 同一个特性、点名的测试在没卡的机器上打 NOT_RUN_NO_GPU_ADAPTER 后返回成功（SELFTEST_NOT_RUN_TEST 那一条用例）
SELFTEST_PRODUCT_GPU_NOT_RUN_ROW = ("product-card-path-changed\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 4 }\t"
                                    "-p tiny --features tests-on-a-real-gpu --lib -- gpu_parity\tgpu_parity")

SELFTEST_CARGO_CONFIGURATION = '[env]\nBINDGEN_EXTRA_CLANG_ARGS = "-I/crates-mutation-rows-selftest/include"\n'

# 替换文空着的删行变异（真表里删掉一句调用、删掉 litmus 里一道屏障的那一类）
SELFTEST_DELETION_ROW = "halve-deleted\tcrates/tiny/src/lib.rs\t{ value / 2 }\t\t-p tiny --lib -- halves\thalves"

# 派活之前那几样各踩一样的行（接在 4 行正常变异与删行变异之后：变异表第 7–11 行），ROW 那一行改的 docs/outside.rs 另写进样本仓
SELFTEST_STATIC_PROBLEM_ROWS = [
    ("MALFORMED", "empty-expected\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\t"),
    ("ANCHOR", "anchor-missing\tcrates/tiny/src/lib.rs\t{ left / right }\t{ left }\t-p tiny --lib -- adds\tadds"),
    ("NAME", "name-anchored\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left + 1 }\t-p tiny --lib -- adds\tadds$"),
    ("ROW", "outside-copy\tdocs/outside.rs\tpub fn outside() {}\tpub fn outside() { }\t-p tiny --lib -- adds\tadds"),
    ("STRAY", "stray-escape\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ \\&value * 2 }\t-p tiny --lib -- doubles\tdoubles"),
]

# 插桩编译那一格的样本 crate：helps 只执行 add（RUNS），两行同点 adds（同一条测试共用一份覆盖），一共 3 条测试记 3 份覆盖
SELFTEST_COVERAGE_SOURCE = '''// 覆盖那一格的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST helps NEEDS value + 7 RUNS helps,add
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn helper(value: u32) -> u32 { value + 7 }

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn helps() { assert_eq!(super::add(2, 3), 5); }
}
'''

SELFTEST_COVERAGE_ROWS = [
    "add-becomes-mul\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- adds\tadds",
    "add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\tadds",
    "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib -- doubles\tdoubles",
    "helper-becomes-eight\tcrates/tiny/src/lib.rs\t{ value + 7 }\t{ value + 8 }\t-p tiny --lib -- helps\thelps",
]

# 单元键那几格（controls 路第 5 节 crates 变异那一行的 S34、S30、N33、N34、N45，加上被改坏的单元、点名测试的闭包、常量、覆盖复用基线几格）的样本 crate：
#   check_root 里有两条变异（root-457、root-double），点名 checks_root；sum 的变异点名 sums（它还调 pad）；helper 的变异点名 helps（它只调 add，够不着 helper）；
#   taken 的变异点名 branches（它静态上还调 untaken，执行的只有 taken）；OFFSET 是 check_root 用的常量（V5 那一类）
SELFTEST_UNIT_SOURCE = '''// 单元键那几格的样本 crate（假 cargo 按下面几行判）
// TEST checks_root NEEDS slot + 457
// TEST checks_root NEEDS base * 2
// TEST sums NEEDS left + right
// TEST helps NEEDS value + 7 RUNS helps,add
// TEST branches NEEDS value + 9 RUNS branches,taken
pub const OFFSET: u32 = 95;
pub fn check_root(slot: u32) -> u32 {
    let base = slot + 457;
    let floor = base.max(OFFSET);
    floor * 0 + base * 2
}
pub fn pad(value: u32) -> u32 { value + 0 }
pub fn sum(left: u32, right: u32) -> u32 { left + right }
pub fn add(first: u32, second: u32) -> u32 { first + second }
pub fn helper(value: u32) -> u32 { value + 7 }
pub fn taken(value: u32) -> u32 { value + 9 }
pub fn untaken(value: u32) -> u32 { value + 10 }

#[cfg(test)]
mod tests {
    #[test]
    fn checks_root() { assert_eq!(super::check_root(1), 916); }
    #[test]
    fn sums() { assert_eq!(super::sum(super::pad(2), 3), 5); }
    #[test]
    fn helps() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn branches() { let value = super::taken(1); if value == 0 { super::untaken(value); } }
}
'''

SELFTEST_UNIT_ROWS = [
    "root-457\tcrates/tiny/src/lib.rs\t= slot + 457;\t= slot + 458;\t-p tiny --lib -- checks_root\tchecks_root",
    "root-double\tcrates/tiny/src/lib.rs\t0 + base * 2\t0 + base * 3\t-p tiny --lib -- checks_root\tchecks_root",
    "sum-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- sums\tsums",
    "helper-eight\tcrates/tiny/src/lib.rs\t{ value + 7 }\t{ value + 8 }\t-p tiny --lib -- helps\thelps",
    "taken-eleven\tcrates/tiny/src/lib.rs\t{ value + 9 }\t{ value + 11 }\t-p tiny --lib -- branches\tbranches",
]

SELFTEST_SPLIT_LIFECYCLE_LINE = ('GPU_LIFECYCLE unix_us=1793843585212043 elapsed_us=12792699 pid=67478 thread=ThreadId(7) test=Some("{name}") '
                                 'phase=queue_submit_end card=0 pci="01:00.0" states=0..1')

# 链接失败那一格（selftest_link_failure_cell）的 cargo 输出：编过了、失败在链接，有 error: linking with 与 rust-lld 的报错，没有 error[E<码>]
SELFTEST_LINK_FAILURE_OUTPUT = ("   Compiling tiny v0.1.0 (/work/copy/crates/tiny)\n"
                                "error: linking with `cc` failed: exit status: 1\n"
                                "  |\n"
                                "  = note: rust-lld: error: undefined hidden symbol: anon.0123456789abcdef0123456789abcdef.0.llvm.1234567890\n"
                                "          >>> referenced by tiny.d7t865e39y9bc2k43l33k59a7.1udd635.rcgu.o:(tiny::answer)\n"
                                "\n"
                                "error: could not compile `tiny` (lib test) due to 1 previous error\n")

SELFTEST_OTHER_BATCH_ROW = "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib --quiet -- doubles\tdoubles"

SELFTEST_REACHABILITY_SOURCE = '''// static-check 可达性那几格的样本 crate：选路函数 judge_not_run_without_the_gpu_feature；CPU 判器入口 cpu_judge_entry 只在 run_enumeration 的 CPU 那一臂里调，
// 宽松的恢复 lax_base_recovery 在 GPU 路上也调（按单元停）；never_called 谁都不调
pub fn judge_not_run_without_the_gpu_feature(configured: bool, compiled: bool) -> bool { configured && !compiled }

pub fn cpu_only_check(value: u32) -> bool { value + 1 > 0 }

pub fn cpu_judge_entry(value: u32) -> bool { cpu_only_check(value) }

pub fn gpu_judge(value: u32) -> bool { value < 1000 }

pub fn recover_value(value: u32) -> u32 { value + 3 }

pub fn lax_base_recovery(value: u32) -> bool { recover_value(value) > 0 }

pub fn run_enumeration(value: u32) -> bool {
    let _ = lax_base_recovery(value);
    if judge_not_run_without_the_gpu_feature(true, true) { gpu_judge(value) } else { cpu_judge_entry(value) }
}

pub fn plain() -> u32 { 2 + 2 }

pub fn never_called(value: u32) -> u32 { value * 7 }

#[cfg(test)]
mod tests {
    fn configuration_on_the_cpu() -> &'static str { "ENABLE_GPU=0\\n" }

    fn hand_out_letting_the_gpu_refuse() -> bool { true }

    #[test]
    fn enumeration_judges() { assert!(super::run_enumeration(1)); }
    #[test]
    fn enumeration_with_an_observer() { assert!(super::run_enumeration(1)); assert!(super::cpu_judge_entry(1)); }
    #[test]
    fn enumeration_on_the_cpu() { assert!(!configuration_on_the_cpu().is_empty()); assert!(super::run_enumeration(1)); }
    #[test]
    fn enumeration_on_refused_tables() { assert!(hand_out_letting_the_gpu_refuse()); assert!(super::run_enumeration(1)); }
    #[test]
    fn plain_sum() { assert_eq!(super::plain(), 4); }
    #[test]
    fn source_names_every_function() { assert!(include_str!("lib.rs").contains("fn never_called")); }
}
'''

SELFTEST_REACHABILITY_ROWS = [
    "cpu-check-blind\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
    "cpu-check-observed\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_with_an_observer\tenumeration_with_an_observer",
    "cpu-check-on-the-cpu-configuration\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_on_the_cpu\tenumeration_on_the_cpu",
    "cpu-check-refused-tables\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_on_refused_tables\tenumeration_on_refused_tables",
    "never-called-seven\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "recovery-three\tcrates/tiny/src/lib.rs\t{ value + 3 }\t{ value + 4 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
    "plain-four\tcrates/tiny/src/lib.rs\t{ 2 + 2 }\t{ 2 + 3 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "never-called-read-as-text\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 9 }\t-p tiny --lib -- source_names_every_function\tsource_names_every_function",
]

SELFTEST_REACHABILITY_FILE = "crates/tiny/src/lib.rs"

# 样本 crate 的截断表：run_enumeration 连到 cpu_judge_entry 的那一条边（按边停）与 lax_base_recovery（按单元停）
SELFTEST_REACHABILITY_STOPS = (((SELFTEST_REACHABILITY_FILE, "run_enumeration"), (SELFTEST_REACHABILITY_FILE, "cpu_judge_entry")),
                               (None, (SELFTEST_REACHABILITY_FILE, "lax_base_recovery")))

# 闭包拒算的样本测试（读 CARGO_MANIFEST_DIR 的函数拒算 runtime-path，调它的测试闭包算不出键、已收的成员照有）插进 SELFTEST_REACHABILITY_SOURCE 的 mod tests 开头
SELFTEST_REACHABILITY_REFUSED_TESTS = '''    fn reads_the_manifest_directory() -> bool { std::env::var("CARGO_MANIFEST_DIR").is_ok() }

    #[test]
    fn refused_calls_never_called() { assert!(reads_the_manifest_directory()); assert_eq!(super::never_called(1), 7); }
    #[test]
    fn refused_calls_plain_only() { assert!(reads_the_manifest_directory()); assert_eq!(super::plain(), 4); }
    #[test]
    fn refused_enumeration_on_the_gpu_route() { assert!(reads_the_manifest_directory()); assert!(super::run_enumeration(1)); }

'''

SELFTEST_REACHABILITY_REFUSED_ROWS = [
    "refused-reaches-never-called\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- refused_calls_never_called\trefused_calls_never_called",
    "refused-misses-never-called\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- refused_calls_plain_only\trefused_calls_plain_only",
    "refused-behind-the-cpu-judge\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- refused_enumeration_on_the_gpu_route\trefused_enumeration_on_the_gpu_route",
    "whole-misses-never-called\tcrates/tiny/src/lib.rs\t{ value * 7 }\t{ value * 8 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "whole-behind-the-cpu-judge\tcrates/tiny/src/lib.rs\t{ value + 1 > 0 }\t{ value + 1 > 5 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
]

SELFTEST_NOT_RUN_ITEM_LINE = ("NOT_RUN_NO_GPU_ADAPTER test=crash_state_enumeration::fixed_script::stale_tail "
                              "reason=配置开了 GPU（ENABLE_GPU=1）而这份二进制没编进 gpu 特性：判段不跑\n")

SELFTEST_NOT_RUN_ITEM_OUTPUT = ("running 1 test\ntest tests::stale_tail_recovers_every_crash_state ... ok\n\nsuccesses:\n\n"
                                "---- tests::stale_tail_recovers_every_crash_state stdout ----\n" + SELFTEST_NOT_RUN_ITEM_LINE + "\n"
                                "successes:\n    tests::stale_tail_recovers_every_crash_state\n\ntest result: ok. 1 passed; 0 failed\n")

SELFTEST_DEVICE_LOST_FIELDS = "card=NVIDIA GeForce RTX 5090 reason=Xid 31：样本里设备丢失"

SELFTEST_DEVICE_LOST_PANIC = "thread 'tests::gpu_parity' panicked at src/lib.rs:1:5:\nassertion `left == right` failed\n"

SELFTEST_GPU_ROUTE_SOURCE = '''// 开 GPU 补 gpu 特性那一格的样本 crate：enumeration_judges 经 run_enumeration 够到选路函数 judge_not_run_without_the_gpu_feature，plain_sum 够不着
// TEST enumeration_judges NEEDS 1 + 1 RUNS enumeration_judges,run_enumeration,judge_not_run_without_the_gpu_feature
// TEST plain_sum NEEDS 2 + 2
// TEST without_the_gpu_feature_routes_to_not_run NEEDS configured && !compiled
pub fn judge_not_run_without_the_gpu_feature(configured: bool, compiled: bool) -> bool { configured && !compiled }

pub fn run_enumeration() -> u32 { if judge_not_run_without_the_gpu_feature(false, true) { 0 } else { 1 + 1 } }

pub fn plain() -> u32 { 2 + 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn enumeration_judges() { assert_eq!(super::run_enumeration(), 2); }
    #[test]
    fn plain_sum() { assert_eq!(super::plain(), 4); }
    #[cfg(not(feature = "gpu"))]
    #[test]
    fn without_the_gpu_feature_routes_to_not_run() { assert!(super::judge_not_run_without_the_gpu_feature(true, false)); }
}
'''

SELFTEST_GPU_ROUTE_ROWS = [
    "enumeration-two-to-three\tcrates/tiny/src/lib.rs\t{ 1 + 1 }\t{ 1 + 2 }\t-p tiny --lib -- enumeration_judges\tenumeration_judges",
    "plain-four-to-five\tcrates/tiny/src/lib.rs\t{ 2 + 2 }\t{ 2 + 3 }\t-p tiny --lib -- plain_sum\tplain_sum",
    "route-and-becomes-or\tcrates/tiny/src/lib.rs\t{ configured && !compiled }\t{ configured || !compiled }\t"
    "-p tiny --lib -- without_the_gpu_feature_routes_to_not_run\twithout_the_gpu_feature_routes_to_not_run",
]

SELFTEST_GPU_THREADS_SOURCE = '''
// TEST released_agrees NEEDS 1 + 1 RUNS released_agrees,shared
pub fn shared() -> u32 { 1 + 1 }
'''

SELFTEST_GPU_THREADS_ROWS = [
    # 放开名单里的测试目标（目标名取本模块的 GPU_TEST_THREADS_RELEASED_TARGETS，格里 format 进去）
    "released-one-plus-one\tcrates/tiny/src/lib.rs\t{ 1 + 1 }\t{ 1 + 2 }\t-p tiny --features gpu --test {target} -- released_agrees\treleased_agrees",
    SELFTEST_GPU_FEATURE_ROW,
    SELFTEST_TABLE_ROWS[0],
    "gpu-halve-exact\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --features gpu --lib -- --exact tests::halves\thalves",
    "gpu-negate-own-threads\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --features gpu --lib -- negates --test-threads=3\tnegates",
]

# 只经卡池取卡的测试目标放开那一格（crates-mutation-rows.py 文件头「带 gpu 特性的格测试参数收尾带 --test-threads」）：tiny 另有两个集成测试目标，
#   pool_only_sample 登记在样本仓验证包那份名单的 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL 里，still_bypassing_sample 只登记在还没挪上卡池的那一张里。
#   两条用例各有自己的函数与自己的 NEEDS 原文：假 cargo 判 NEEDS 是看「去掉这一条自己那一行之后，原文还在不在整份源码里」，
#   两条共用同一段原文时，一条的变异被另一条那一行里的原文兜住，两条都判不红。
SELFTEST_GPU_POOL_TARGETS_SOURCE = '''
// TEST pool_agrees NEEDS 5 + 5 RUNS pool_agrees,pooled
// TEST bypass_agrees NEEDS 7 + 7 RUNS bypass_agrees,bypassing
pub fn pooled() -> u32 { 5 + 5 }
pub fn bypassing() -> u32 { 7 + 7 }
'''

SELFTEST_GPU_POOL_TARGETS_ROWS = [
    "pool-only-five-plus-five\tcrates/tiny/src/lib.rs\t{ 5 + 5 }\t{ 5 + 6 }\t-p tiny --features gpu --test pool_only_sample -- pool_agrees\tpool_agrees",
    "still-bypassing-seven-plus-seven\tcrates/tiny/src/lib.rs\t{ 7 + 7 }\t{ 7 + 8 }\t-p tiny --features gpu --test still_bypassing_sample -- bypass_agrees\tbypass_agrees",
]

# 样本仓里验证包那份名单（crates/singlefs-verification/tests/gpu_unit_checks_every_gpu_test_takes_its_cards_through_gpu_test_start.rs）的两张表，写法照真文件
SELFTEST_GPU_POOL_LIST = '''//! 样本：只经卡池取卡的测试文件名单
const FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL: [(&str, &str); 1] = [
    (
        "tests/pool_only_sample.rs",
        include_str!("pool_only_sample.rs"),
    ),
];

const TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL: [(&str, &str); 1] = [(
    "tests/still_bypassing_sample.rs",
    "样本：还绕开卡池",
)];
'''

# 同一份名单去掉 FILES_THAT_TAKE_CARDS_ONLY_FROM_THE_POOL 那张表：认不出要报一句、一个都不放开
SELFTEST_GPU_POOL_LIST_WITHOUT_TABLE = '''//! 样本：名单改了写法
const TEST_FILES_NOT_YET_MOVED_ONTO_THE_CARD_POOL: [(&str, &str); 1] = [(
    "tests/still_bypassing_sample.rs",
    "样本：还绕开卡池",
)];
'''

# 重测试目标只许 release 那两格（文件头同名一段）：tiny 另有一个集成测试目标 tiny_crash_stream，自证自己的登记表只登记它一条崩溃枚举用例
SELFTEST_RELEASE_SOURCE = '''
// TEST streams_agree NEEDS 3 + 4 RUNS streams_agree,stream_total
pub fn stream_total() -> u32 { 3 + 4 }
'''

SELFTEST_RELEASE_REGISTRY = ("# 自证用的登记表：一条崩溃枚举用例点 tiny 的集成测试目标 tiny_crash_stream\n"
                             "crash-case:tiny-stream\t-\ttest=tiny:tiny_crash_stream:streams_agree count-line=TINY\t# 自证样本\n")

SELFTEST_RELEASE_CRASH_ROW = "stream-seven-becomes-eight\tcrates/tiny/src/lib.rs\t{ 3 + 4 }\t{ 3 + 5 }\t-p tiny --test tiny_crash_stream -- streams_agree\tstreams_agree"

SELFTEST_RELEASE_WRITTEN_ROW = ("halve-released-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t"
                                "-p tiny --release --features gpu --lib -- halves\thalves")

SELFTEST_EXTERNAL_LOCK = ('version = 4\n\n[[package]]\nname = "fakedep"\nversion = "1.2.3"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n'
                          'checksum = "0000000000000000000000000000000000000000000000000000000000000000"\n\n'
                          '[[package]]\nname = "tiny"\nversion = "0.1.0"\ndependencies = [\n "fakedep",\n]\n')

SelftestSentinelRow = collections.namedtuple("SelftestSentinelRow", "line_number name")

SELFTEST_WGSL_KERNEL = '''// WGSL 按 fn 取键那一格的样本内核
const SCALE: u32 = 3u;
fn scaled(value: u32) -> u32 {
    let base = value + 1u;
    return base * SCALE;
}
fn untouched(value: u32) -> u32 {
    return value + 5u;
}
// 只在注释里的一句：wgsl 样本的注释行
'''

SELFTEST_WGSL_ROWS = [
    "wgsl-scaled-doubles\tcrates/tiny/src/kernel.wgsl\treturn base * SCALE;\treturn base * 2u;\t-p tiny --lib -- doubles\tdoubles",
    "wgsl-scale-four\tcrates/tiny/src/kernel.wgsl\tSCALE: u32 = 3u\tSCALE: u32 = 4u\t-p tiny --lib -- doubles\tdoubles",
    "wgsl-comment-only\tcrates/tiny/src/kernel.wgsl\twgsl 样本的注释行\twgsl 样本的另一行\t-p tiny --lib -- doubles\tdoubles",
]

SELFTEST_TWIN_UNIT_SOURCE = '''// 撞键拒算那一格的样本 crate：两份内容相同的函数各在一个文件里，点名的测试两个都走；另一条测试只走 three
// TEST values_are_one FILES src/left.rs,src/right.rs { 1 } RUNS values_are_one,value
// TEST three_is_three NEEDS 1 + 2
pub mod left;
pub mod right;

pub fn three() -> u32 { 1 + 2 }

#[cfg(test)]
mod tests {
    #[test]
    fn values_are_one() { assert_eq!(super::left::value() + super::right::value(), 2); }
    #[test]
    fn three_is_three() { assert_eq!(super::three(), 3); }
}
'''

SELFTEST_TWIN_UNIT_ROWS = [
    "left-one-to-two\tcrates/tiny/src/left.rs\t{ 1 }\t{ 2 }\t-p tiny --lib -- values_are_one\tvalues_are_one",
    "right-one-to-two\tcrates/tiny/src/right.rs\t{ 1 }\t{ 2 }\t-p tiny --lib -- values_are_one\tvalues_are_one",
    "three-becomes-four\tcrates/tiny/src/lib.rs\t{ 1 + 2 }\t{ 1 + 3 }\t-p tiny --lib -- three_is_three\tthree_is_three",
]

SELFTEST_TARGET_TEST = "fn run() {{ assert_eq!(tiny::shared(), {0}); }}\n\n#[test]\nfn smoke() {{ run(); }}\n"

SELFTEST_TARGET_ROWS = [
    "alpha-one-to-three\tcrates/tiny/tests/alpha.rs\t(), 1)\t(), 3)\t-p tiny --test alpha -- smoke\tsmoke",
    "beta-two-to-three\tcrates/tiny/tests/beta.rs\t(), 2)\t(), 3)\t-p tiny --test beta -- smoke\tsmoke",
]

SELFTEST_ANCHOR_MISSING_ROW = "anchor-missing\tcrates/tiny/src/lib.rs\t{ left / right }\t{ left }\t-p tiny --lib -- adds\tadds"

# 改的是数据文件（不是 .rs）而原文命中 0 次的一行：单元键不核锚点（数据文件按整份字节算），不先剔掉就算得出键、会被双机分片的驱动分给第二台
SELFTEST_DATA_FILE_ANCHOR_ROW = "data-file-anchor-missing\tfixture-data/answer.txt\t43\t44\t-p tiny --lib -- adds\tadds"

# 牵连面那一格：单元键样本 crate 的 5 行再加一行改 OFFSET（check_root 用的常量），点名 checks_root：checks_root 的闭包进了三行
SELFTEST_IMPACT_ROW = "offset-plus-one\tcrates/tiny/src/lib.rs\tpub const OFFSET: u32 = 95;\tpub const OFFSET: u32 = 96;\t-p tiny --lib -- checks_root\tchecks_root"

SELFTEST_IMPACT_TEST = "// TEST checks_root NEEDS OFFSET: u32 = 95 RUNS checks_root,check_root\n"

# 照闭包手数：6 行的闭包都有 mod tests 那个单元；checks_root 的闭包（tests::checks_root、check_root、OFFSET）进了三行；
# 计数 1 的按名字排、add 最前（helps 的闭包里有它，helper 是那一行被改坏的单元）
SELFTEST_IMPACT_EXPECTED = ("牵连最大的函数单元：crates/tiny/src/lib.rs::tests 进了 6 行的闭包（前 5：crates/tiny/src/lib.rs::tests 6、crates/tiny/src/lib.rs::OFFSET 3、"
                            "crates/tiny/src/lib.rs::check_root 3、crates/tiny/src/lib.rs::tests::checks_root 3、crates/tiny/src/lib.rs::add 1）；一次改动最坏牵连 6 行"
                            "（算了 6 行的闭包；拒算与锚点失效的 0 行没有闭包、没算进）")


FIXTURE_VALUES = {   # load_selftest_fixtures 照这张表把名字放进 crates-mutation-rows.py 那份模块
    "SELFTEST_TESTS": SELFTEST_TESTS,
    "SELFTEST_SOURCE": SELFTEST_SOURCE,
    "SELFTEST_TABLE_ROWS": SELFTEST_TABLE_ROWS,
    "SELFTEST_READS_OUTSIDE_TEST": SELFTEST_READS_OUTSIDE_TEST,
    "SELFTEST_READS_OUTSIDE_ROW": SELFTEST_READS_OUTSIDE_ROW,
    "SELFTEST_NOT_RUN_TEST": SELFTEST_NOT_RUN_TEST,
    "SELFTEST_NOT_RUN_ROW": SELFTEST_NOT_RUN_ROW,
    "SELFTEST_GPU_FEATURE_ROW": SELFTEST_GPU_FEATURE_ROW,
    "SELFTEST_PRODUCT_GPU_FEATURE_ROW": SELFTEST_PRODUCT_GPU_FEATURE_ROW,
    "SELFTEST_PRODUCT_GPU_NOT_RUN_ROW": SELFTEST_PRODUCT_GPU_NOT_RUN_ROW,
    "SELFTEST_CARGO_CONFIGURATION": SELFTEST_CARGO_CONFIGURATION,
    "SELFTEST_DELETION_ROW": SELFTEST_DELETION_ROW,
    "SELFTEST_STATIC_PROBLEM_ROWS": SELFTEST_STATIC_PROBLEM_ROWS,
    "SELFTEST_COVERAGE_SOURCE": SELFTEST_COVERAGE_SOURCE,
    "SELFTEST_COVERAGE_ROWS": SELFTEST_COVERAGE_ROWS,
    "SELFTEST_UNIT_SOURCE": SELFTEST_UNIT_SOURCE,
    "SELFTEST_UNIT_ROWS": SELFTEST_UNIT_ROWS,
    "SELFTEST_SPLIT_LIFECYCLE_LINE": SELFTEST_SPLIT_LIFECYCLE_LINE,
    "SELFTEST_LINK_FAILURE_OUTPUT": SELFTEST_LINK_FAILURE_OUTPUT,
    "SELFTEST_OTHER_BATCH_ROW": SELFTEST_OTHER_BATCH_ROW,
    "SELFTEST_REACHABILITY_SOURCE": SELFTEST_REACHABILITY_SOURCE,
    "SELFTEST_REACHABILITY_ROWS": SELFTEST_REACHABILITY_ROWS,
    "SELFTEST_REACHABILITY_FILE": SELFTEST_REACHABILITY_FILE,
    "SELFTEST_REACHABILITY_STOPS": SELFTEST_REACHABILITY_STOPS,
    "SELFTEST_REACHABILITY_REFUSED_TESTS": SELFTEST_REACHABILITY_REFUSED_TESTS,
    "SELFTEST_REACHABILITY_REFUSED_ROWS": SELFTEST_REACHABILITY_REFUSED_ROWS,
    "SELFTEST_NOT_RUN_ITEM_LINE": SELFTEST_NOT_RUN_ITEM_LINE,
    "SELFTEST_NOT_RUN_ITEM_OUTPUT": SELFTEST_NOT_RUN_ITEM_OUTPUT,
    "SELFTEST_DEVICE_LOST_FIELDS": SELFTEST_DEVICE_LOST_FIELDS,
    "SELFTEST_DEVICE_LOST_PANIC": SELFTEST_DEVICE_LOST_PANIC,
    "SELFTEST_GPU_ROUTE_SOURCE": SELFTEST_GPU_ROUTE_SOURCE,
    "SELFTEST_GPU_ROUTE_ROWS": SELFTEST_GPU_ROUTE_ROWS,
    "SELFTEST_GPU_THREADS_SOURCE": SELFTEST_GPU_THREADS_SOURCE,
    "SELFTEST_GPU_THREADS_ROWS": SELFTEST_GPU_THREADS_ROWS,
    "SELFTEST_GPU_POOL_TARGETS_SOURCE": SELFTEST_GPU_POOL_TARGETS_SOURCE,
    "SELFTEST_GPU_POOL_TARGETS_ROWS": SELFTEST_GPU_POOL_TARGETS_ROWS,
    "SELFTEST_GPU_POOL_LIST": SELFTEST_GPU_POOL_LIST,
    "SELFTEST_GPU_POOL_LIST_WITHOUT_TABLE": SELFTEST_GPU_POOL_LIST_WITHOUT_TABLE,
    "SELFTEST_RELEASE_SOURCE": SELFTEST_RELEASE_SOURCE,
    "SELFTEST_RELEASE_REGISTRY": SELFTEST_RELEASE_REGISTRY,
    "SELFTEST_RELEASE_CRASH_ROW": SELFTEST_RELEASE_CRASH_ROW,
    "SELFTEST_RELEASE_WRITTEN_ROW": SELFTEST_RELEASE_WRITTEN_ROW,
    "SELFTEST_EXTERNAL_LOCK": SELFTEST_EXTERNAL_LOCK,
    "SelftestSentinelRow": SelftestSentinelRow,
    "SELFTEST_WGSL_KERNEL": SELFTEST_WGSL_KERNEL,
    "SELFTEST_WGSL_ROWS": SELFTEST_WGSL_ROWS,
    "SELFTEST_TWIN_UNIT_SOURCE": SELFTEST_TWIN_UNIT_SOURCE,
    "SELFTEST_TWIN_UNIT_ROWS": SELFTEST_TWIN_UNIT_ROWS,
    "SELFTEST_TARGET_TEST": SELFTEST_TARGET_TEST,
    "SELFTEST_TARGET_ROWS": SELFTEST_TARGET_ROWS,
    "SELFTEST_ANCHOR_MISSING_ROW": SELFTEST_ANCHOR_MISSING_ROW,
    "SELFTEST_DATA_FILE_ANCHOR_ROW": SELFTEST_DATA_FILE_ANCHOR_ROW,
    "SELFTEST_IMPACT_ROW": SELFTEST_IMPACT_ROW,
    "SELFTEST_IMPACT_TEST": SELFTEST_IMPACT_TEST,
    "SELFTEST_IMPACT_EXPECTED": SELFTEST_IMPACT_EXPECTED,
}
