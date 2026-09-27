"""L1 闸那一侧（P1 剥包装、P2 设给里面那条命令的环境、P3 --list、P10 --config 按 TOML 读）：拿 JSON 喂仓里被判的重型测试闸
（子 agent 身份，只看退出码，被判的命令一条都不执行）。前两轮攻过的形状（L1–L14、R1–R8、T1–T4）只留几格当对照，不重复。
复跑：python3 probe_l1_hook.py
格：ATTACK 是该拒的写法，打中 = 放行；CONTROL 是对照。语义（libtest / cargo / systemd-run 真这样跑）由 verify_semantics.sh 在小 crate 上量。"""
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common as pc  # noqa: E402

cells = pc.Cells("L1 闸：P1 P2 P3 P10")
MC = f"bash {pc.REPOSITORY}/research/scripts/run-with-memory-cap.sh 8G"
C561 = "record_checker_judges_absence_by_the_persisted_set"
SIGMA = "every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present"
RUNNER = "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER"
X = os.path.join(pc.SCRATCH_ROOT, "x")
BIN = f"target/release/deps/{C561}-0123456789abcdef"
DETECTIONS = os.path.join(pc.scratch("l1-hook-"), "detections.jsonl")
PLAIN = f"cargo test --release -p singlefs-harness --test {C561}"
TOML_RUNNER = f"'target.x86_64-unknown-linux-gnu.runner=\"{X}/add-ignored.sh\"'"


def cell(kind, label, command, want):
    exit_code, first_lines = pc.hook(command, pc.REPOSITORY, detections=DETECTIONS)
    holds = (exit_code == 2) if want == "refuse" else (exit_code == 0)
    shown = command.replace(pc.REPOSITORY, "<仓>").replace(pc.SCRATCH_ROOT, "<草稿>")
    cells.expect(kind, f"{label} ⇒ 应当{'拒' if want == 'refuse' else '放行'}", holds,
                 f"退 {exit_code}；「{shown[:200]}」" + (f"；{first_lines[:120]}" if first_lines and exit_code else ""))


# ── P3：libtest 的 -- 之后 --list 是过滤词，不是选项 ──
cell("CONTROL", "T1 -- --ignored --list（只列，前两轮对照）", f"{MC} {PLAIN} -- --ignored --list", "allow")
cell("CONTROL", "T2 -- --include-ignored --skip --list（第二轮 T2，P3 修的那一格）", f"{MC} {PLAIN} -- --include-ignored --skip --list", "refuse")
cell("CONTROL", "T8 -- --include-ignored <全量用例>（不带 --list）", f"{MC} {PLAIN} -- --include-ignored {SIGMA}", "refuse")
cell("ATTACK", "T5 -- --include-ignored <全量用例> -- --list（第二个 -- 之后 --list 是过滤词，libtest 照跑全量）",
     f"{MC} {PLAIN} -- --include-ignored {SIGMA} -- --list", "refuse")
cell("ATTACK", "T6 -- --include-ignored --exact <全量用例> -- --list", f"{MC} {PLAIN} -- --include-ignored --exact {SIGMA} -- --list", "refuse")
cell("ATTACK", "T7 直接执行测试二进制 --include-ignored <全量用例> -- --list", f"{MC} ./{BIN} --include-ignored {SIGMA} -- --list", "refuse")
cell("CONTROL", "T9 -- --ignored -- --list（只剩过滤词 --list，一条都不跑：放行是对的）", f"{MC} {PLAIN} -- --ignored -- --list", "allow")
# ── P10：--config 写在子命令之后（cargo 的全局选项，test 后面照认）──
cell("CONTROL", "R6 cargo --config <runner> test …（第二轮对照）", f"{MC} cargo --config {TOML_RUNNER} test -p singlefs-harness --test {C561}", "refuse")
cell("ATTACK", "R9 cargo test --config <runner> -p … --test <登记目标>（--config 写在 test 之后）",
     f"{MC} cargo test --config {TOML_RUNNER} -p singlefs-harness --test {C561}", "refuse")
cell("ATTACK", "R10 cargo test -p … --test <登记目标> --config=<runner>",
     f"{MC} cargo test --release -p singlefs-harness --test {C561} --config={TOML_RUNNER}", "refuse")
cell("ATTACK", "R11 cargo test --config 'target.x86_64-unknown-linux-gnu = { runner = … }'（行内表写在 test 之后）",
     f"{MC} cargo test --config 'target.x86_64-unknown-linux-gnu = {{ runner = \"{X}/add-ignored.sh\" }}' -p singlefs-harness --test {C561}", "refuse")
# ── P2：systemd-run 给里面那条命令设环境的别的写法 ──
cell("CONTROL", "R2 systemd-run -E 设 runner（第二轮 R2，P2 修的那一格）", f"{MC} systemd-run --user --scope -E {RUNNER}={X}/add-ignored.sh {PLAIN}", "refuse")
cell("ATTACK", "S1 systemd-run --wait --pipe -p Environment=<runner>（service 单元的 Environment= 属性）",
     f"{MC} systemd-run --user --wait --pipe -p Environment={RUNNER}={X}/add-ignored.sh {PLAIN}", "refuse")
cell("ATTACK", "S2 systemd-run --wait --pipe --property=Environment=<runner>",
     f"{MC} systemd-run --user --wait --pipe --property=Environment={RUNNER}={X}/add-ignored.sh {PLAIN}", "refuse")
# ── P1：包装程序要值的长选项、值另起一个词（getopt 认），闸把值当成要起的命令 ──
cell("CONTROL", "L5 systemd-run --user --scope -q -u k1（前两轮对照）", f"{MC} systemd-run --user --scope -q -u k1 {PLAIN} -- --ignored", "refuse")
cell("ATTACK", "S3 systemd-run --user --scope --expand-environment no <命令>（值另起一个词）",
     f"{MC} systemd-run --user --scope --expand-environment no {PLAIN} -- --ignored", "refuse")
cell("ATTACK", "S4 strace --output <文件> <命令>（G2「看到但没做」3 记过的同一类，今天仍放行）",
     f"{MC} strace --output {X}/s.txt {PLAIN} -- --ignored", "refuse")
sys.exit(cells.finish())
