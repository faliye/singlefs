"""K1（Y2 属性解析、Y3 剥包装、Y4 --list）探针：拿 JSON 喂重型测试闸（子 agent 身份，只看退出码，不执行任何被判的命令）；
Y2 另在小仓上跑 admission.py crash-cases 看自查怎么判。
复跑：python3 probe_k1_hook.py
格：ATTACK 是「该拒（或该判红）」的写法，打中 = 放行；OVER 是「不该拒」而拒了（误拒）；CONTROL 是对照。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("K1 闸与属性解析")
MC = f"bash {pc.REPOSITORY}/research/scripts/run-with-memory-cap.sh 8G"
C561 = "record_checker_judges_absence_by_the_persisted_set"
SIGMA = "every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present"
RUNNER = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"
X = os.path.join(pc.SCRATCH_ROOT, "x")
BIN = f"target/release/deps/{C561}-0123456789abcdef"
DETECTIONS = os.path.join(pc.scratch("k1-hook-"), "detections.jsonl")


def cell(kind, label, command, want, cwd=pc.REPOSITORY):
    exit_code, first_line = pc.hook(command, cwd, detections=DETECTIONS)
    holds = (exit_code == 2) if want == "refuse" else (exit_code == 0)
    cells.expect(kind, f"{label} ⇒ 应当{'拒' if want == 'refuse' else '放行'}", holds,
                 f"退 {exit_code}；「{command.replace(pc.REPOSITORY, '<仓>').replace(pc.SCRATCH_ROOT, '<草稿>')[:170]}」"
                 + (f"；{first_line}" if first_line and exit_code else ""))


TARGET = f"cargo test --release -p singlefs-harness --test {C561} -- --ignored"
# ── Y3：剥包装 ──
cell("CONTROL", "L1 strace -f -o <文件>", f"{MC} strace -f -o {X}/s.txt {TARGET}", "refuse")
cell("ATTACK", "L2 strace -fo <文件>（短选项合写，strace 常见写法）", f"{MC} strace -fo {X}/s.txt {TARGET}", "refuse")
cell("CONTROL", "L3 flock -x -w 10 <锁>", f"{MC} flock -x -w 10 {X}/lock {TARGET}", "refuse")
cell("ATTACK", "L4 flock -xw 10 <锁>（短选项合写）", f"{MC} flock -xw 10 {X}/lock {TARGET}", "refuse")
cell("CONTROL", "L5 systemd-run --user --scope -q -u k1", f"systemd-run --user --scope -q -u k1 {TARGET}", "refuse")
cell("ATTACK", "L6 systemd-run --user --scope -qu k1（短选项合写）", f"systemd-run --user --scope -qu k1 {TARGET}", "refuse")
cell("ATTACK", "L7 /usr/bin/time -ao <文件>（短选项合写）", f"{MC} /usr/bin/time -ao {X}/t.txt {TARGET}", "refuse")
cell("CONTROL", "L8 flock <锁> bash -c '…'（派发点名的形状）", f"{MC} flock {X}/lock bash -c '{TARGET}'", "refuse")
cell("CONTROL", "L9 flock <锁> -c '…'", f"{MC} flock {X}/lock -c '{TARGET}'", "refuse")
cell("CONTROL", "L10 env -i PATH=… 包一层", f"{MC} env -i PATH=/usr/bin:/bin {TARGET}", "refuse")
cell("CONTROL", "L11 nice / timeout / strace -f 套三层", f"{MC} nice -n 19 timeout 10h strace -f -o {X}/s.txt {TARGET}", "refuse")
cell("ATTACK", "L12 env -S '<整条命令>'（lib_shell_words 的前缀，不在这一批改动里）", f"{MC} env -S '{TARGET}'", "refuse")
cell("ATTACK", "L13 不套内存包装的 strace -fo：重型与「没经内存包装」两道都漏", f"strace -fo {X}/s.txt {TARGET}", "refuse")
cell("CONTROL", "L14 不套内存包装的 strace -f -o、非重型目标：「没经内存包装」那一道拒",
     f"strace -f -o {X}/s.txt cargo test --release -p singlefs-harness --test second_transaction_step_one_overwrite", "refuse")
# ── Y3：runner / 别名 ──
PLAIN = f"cargo test --release -p singlefs-harness --test {C561}"
cell("CONTROL", "R1 命令前缀里设 runner、不带 --ignored", f"{RUNNER}={X}/add-ignored.sh {MC} {PLAIN}", "refuse")
cell("ATTACK", "R2 systemd-run -E 设 runner、不带 --ignored", f"{MC} systemd-run --user --scope -E {RUNNER}={X}/add-ignored.sh {PLAIN}", "refuse")
cell("ATTACK", "R3 systemd-run --setenv= 设 runner、不带 --ignored", f"{MC} systemd-run --user --scope --setenv={RUNNER}={X}/add-ignored.sh {PLAIN}", "refuse")
cell("CONTROL", "R4 --config alias.xt=… 定别名", f"{MC} cargo --config 'alias.xt=\"test --release -p singlefs-harness --test {C561} -- --ignored\"' xt", "refuse")
cell("ATTACK", "R5 --config alias.\"xt\"=…（TOML 带引号的键，同一个别名）", f"{MC} cargo --config 'alias.\"xt\"=\"test --release -p singlefs-harness --test {C561} -- --ignored\"' xt", "refuse")
cell("CONTROL", "R6 --config target.<三元组>.runner=…", f"{MC} cargo --config 'target.x86_64-unknown-linux-gnu.runner=\"{X}/add-ignored.sh\"' test -p singlefs-harness --test {C561}", "refuse")
cell("ATTACK", "R7 --config target.<三元组>.\"runner\"=…（带引号的键）", f"{MC} cargo --config 'target.x86_64-unknown-linux-gnu.\"runner\"=\"{X}/add-ignored.sh\"' test -p singlefs-harness --test {C561}", "refuse")
cell("ATTACK", "R8 export 设 runner，同一条命令里再跑", f"export {RUNNER}={X}/add-ignored.sh; {MC} {PLAIN}", "refuse")
# ── Y4：--list ──
cell("CONTROL", "T1 -- --ignored --list（只列）", f"{MC} {PLAIN} -- --ignored --list", "allow")
cell("ATTACK", "T2 -- --include-ignored --skip --list（--list 是 --skip 的值，照样全跑）", f"{MC} {PLAIN} -- --include-ignored --skip --list", "refuse")
cell("ATTACK", "T3 直接执行测试二进制 --include-ignored --skip --list", f"{MC} ./{BIN} --include-ignored --skip --list", "refuse")
cell("ATTACK", "T4 -- --include-ignored --exact <全量用例> --logfile --list", f"{MC} {PLAIN} -- --include-ignored --exact {SIGMA} --logfile --list", "refuse")

# ── Y2：属性解析（小仓：包 pkg、测试目标 own_case、登记的用例函数 the_case）──
Y2_VARIANTS = [
    # (种类, 名字, own_case.rs 的正文, 闸应当怎么判不带 --ignored 的 cargo test --release, crash-cases 应当退几)
    ("CONTROL", "V0 #[test] #[ignore] fn the_case", "#[test]\n#[ignore]\nfn the_case() {}\n", "allow", 0),
    ("CONTROL", "V1 没标 ignore", "#[test]\nfn the_case() {}\n", "refuse", 2),
    ("CONTROL", "V2 #[ignore = \"…]…\\\"…\"] 与 #[rustfmt::skip] 夹在中间",
     "#[test]\n#[ignore = \"有 ] 方括号与 \\\" 引号\"]\n#[rustfmt::skip]\npub(crate) fn the_case() {}\n", "allow", 0),
    ("ATTACK", "V3 debug 下一份标 ignore、release 下一份不标（同名、cfg 二选一）",
     "#[cfg(debug_assertions)]\n#[test]\n#[ignore]\nfn the_case() {}\n#[cfg(not(debug_assertions))]\n#[test]\nfn the_case() { /* 全量 */ }\n", "refuse", 2),
    ("ATTACK", "V4 子模块里一份标 ignore 的同名函数写在前面，顶层那一份（--exact the_case 跑的）不标",
     "mod slow {\n    #[test]\n    #[ignore]\n    fn the_case() {}\n}\n#[test]\nfn the_case() { /* 全量 */ }\n", "refuse", 2),
    ("ATTACK", "V5 宏生成的用例（没有字面的 fn the_case(），不标 ignore",
     "macro_rules! crash_case {\n    ($name:ident) => {\n        #[test]\n        fn $name() { /* 全量 */ }\n    };\n}\ncrash_case!(the_case);\n", "refuse", 2),
    ("CONTROL", "V6 #[cfg_attr(not(debug_assertions), ignore)]（设计上不算 ignore：两边都判重型 / 判红）",
     "#[test]\n#[cfg_attr(not(debug_assertions), ignore)]\nfn the_case() {}\n", "refuse", 2),
    ("OVER", "V7 # [ignore]（# 与 [ 之间有空格，rustc 认）", "#[test]\n# [ignore]\nfn the_case() {}\n", "allow", 0),
    ("CONTROL", "V8 /// 文档注释、/* 块注释 */ 夹在属性之间",
     "/// 文档\n#[test]\n/* 块注释 */\n#[ignore]\n// 行注释\nfn the_case() {}\n", "allow", 0),
]
admission_path = pc.ADMISSION
for kind, label, source, guard_want, crash_cases_want in Y2_VARIANTS:
    work = pc.scratch("k1-y2-")
    pc.write(os.path.join(work, "Cargo.toml"), '[workspace]\nmembers = ["crates/pkg"]\n')
    pc.write(os.path.join(work, "crates/pkg/Cargo.toml"), '[package]\nname = "pkg"\nversion = "0.0.0"\nedition = "2021"\n')
    pc.write(os.path.join(work, "crates/pkg/src/lib.rs"), "pub fn one() -> u32 { 1 }\n")
    pc.write(os.path.join(work, "crates/pkg/tests/own_case.rs"), source)
    pc.write(os.path.join(work, ".claude/gate.d/stage-inputs.tsv"),
             "crash-case:own\tcrates/ Cargo.toml\ttest=pkg:own_case:the_case count-line=LAYER0\t# 样本\n")
    pc.run(["git", "init", "-q", "-b", "master", work])
    guard_exit, first_line = pc.hook(f"{MC} cargo test --release -p pkg --test own_case", work, detections=DETECTIONS)
    guard_ok = (guard_exit == 2) if guard_want == "refuse" else (guard_exit == 0)
    check_exit, check_out, _check_err = pc.run([sys.executable, admission_path, "crash-cases", work])
    check_ok = check_exit == crash_cases_want
    cells.expect(kind, f"{label}：闸 ⇒ 应当{'拒' if guard_want == 'refuse' else '放行'}", guard_ok, f"闸退 {guard_exit}")
    cells.expect(kind, f"{label}：crash-cases 自查 ⇒ 应当退 {crash_cases_want}", check_ok,
                 f"自查退 {check_exit}：{check_out.strip().splitlines()[-1][:150] if check_out.strip() else '（无输出）'}")

sys.exit(cells.finish())
