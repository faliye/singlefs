"""P4 / P4b（按每一处同名 fn <名>( 判 #[ignore]）：用例函数的一份定义不在测试目标自己的源文件里（include! 进来的文件），闸与 crash-cases 自查怎么判。
小仓：包 pkg、测试目标 own_case、登记的用例函数 the_case；闸喂 JSON（不执行），自查跑仓里被判的 admission.py crash-cases。
语义（cargo test --release 不带 --ignored 照跑 include! 进来那一份）见 verify_semantics.sh 的 SEM-P4。
复跑：python3 probe_l1_attrs.py"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("L1 P4：include! 进来的同名定义")
MC = f"bash {pc.REPOSITORY}/research/scripts/run-with-memory-cap.sh 8G"
DETECTIONS = os.path.join(pc.scratch("l1-attrs-"), "detections.jsonl")
# (种类, 名字, own_case.rs, 另外的文件 {相对 tests/ 的路径: 正文}, 闸应当怎么判, crash-cases 应当退几)
VARIANTS = [
    ("CONTROL", "V0 #[test] #[ignore] fn the_case（一份，标了）", "#[test]\n#[ignore]\nfn the_case() {}\n", {}, "allow", 0),
    ("CONTROL", "V3 同名 cfg 二选一、release 那一份不标（第二轮 V3，P4 修的那一格）",
     "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_case() {}\n", {}, "refuse", 2),
    ("ATTACK", "I1 debug 下标 ignore；release 下 include!(\"common/release_case.rs\") 进来一份不标的同名用例",
     "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\ninclude!(\"common/release_case.rs\");\n",
     {"common/release_case.rs": "#[test]\nfn the_case() { /* 全量 */ }\n"}, "refuse", 2),
    ("ATTACK", "I2 同 I1，include! 的参数换成 concat!(\"common/\", \"release_case.rs\")",
     "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\ninclude!(concat!(\"common/\", \"release_case.rs\"));\n",
     {"common/release_case.rs": "#[test]\nfn the_case() { /* 全量 */ }\n"}, "refuse", 2),
    ("CONTROL", "I3 只有 include! 进来的一份（测试目标里没有字面的 fn the_case(）：判不出，按没标算（P5）",
     "include!(\"common/release_case.rs\");\n", {"common/release_case.rs": "#[test]\n#[ignore]\nfn the_case() {}\n"}, "refuse", 2),
]
for kind, label, source, extra, guard_want, check_want in VARIANTS:
    work = pc.scratch("l1-attrs-case-")
    pc.write(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    pc.write(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
    pc.write(os.path.join(work, "crates/pkg/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
    pc.write(os.path.join(work, "crates/pkg/tests/own_case.rs"), source)
    for relative, text in extra.items():
        pc.write(os.path.join(work, "crates/pkg/tests", relative), text)
    pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
             "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0\t# 样本\n")
    pc.run(["git", "init", "-q", "-b", "master", work])
    guard_exit, _lines = pc.hook(f"{MC} cargo test --release -p pkg --test own_case", work, detections=DETECTIONS)
    check_exit, check_out, _ = pc.run([sys.executable, pc.ADMISSION, "crash-cases", work])
    cells.expect(kind, f"{label}：闸（cargo test --release，不带 --ignored） ⇒ 应当{'拒' if guard_want == 'refuse' else '放行'}",
                 (guard_exit == 2) if guard_want == "refuse" else (guard_exit == 0), f"闸退 {guard_exit}")
    cells.expect(kind, f"{label}：crash-cases 自查 ⇒ 应当退 {check_want}", check_exit == check_want,
                 f"自查退 {check_exit}：{check_out.strip().splitlines()[-1][:140] if check_out.strip() else '（无输出）'}")
sys.exit(cells.finish())
