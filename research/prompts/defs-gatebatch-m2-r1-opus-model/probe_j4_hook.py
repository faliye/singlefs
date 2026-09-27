"""J4 探针：拿 JSON 喂重型测试闸（只看退出码，不执行任何被判的命令），子 agent（implementation-writer）身份。
复跑：python3 probe_j4_hook.py [--hook <heavy-test-guard.sh>]
每条命令都经 run-with-memory-cap.sh 包一层（把「没经内存包装」那一道拒绝与「重型」分开）。
BYPASS 格：跑到登记的崩溃枚举用例（或整包测试）却该拒而没拒；OVER 格：只列用例、或只跑快用例却被拒；CONTROL 格：今天应当判对的。"""
import json
import os
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import probe_common  # noqa: E402

HOOK = probe_common.option("--hook", probe_common.DEFAULT_HOOK)
ADMISSION = probe_common.option("--admission", probe_common.DEFAULT_ADMISSION)
REPOSITORY = probe_common.REPOSITORY
cells = probe_common.Cells("J4 重型测试闸")
MC = "bash research/scripts/run-with-memory-cap.sh 8G"
C561 = "record_checker_judges_absence_by_the_persisted_set"
FLOOR = "second_transaction_crash_inside_the_floor_raise_pushed_by_the_session"
SIGMA = "every_crash_state_of_sigma_leaves_every_unit_of_the_claimed_publishes_present"
FAST = "the_false_red_and_the_misaligned_hole_have_identical_crash_images_and_only_the_hole_is_missing"
BIN = f"target/release/deps/{C561}-0123456789abcdef"
X = "/tmp/claude-1000/opus-j4-x"

scratch = tempfile.mkdtemp(prefix="opus-j4-")
detections = os.path.join(scratch, "detections.jsonl")


def hook(command, cwd=REPOSITORY, agent_type="implementation-writer"):
    payload = {"tool_name": "Bash", "cwd": cwd, "tool_input": {"command": command}}
    if agent_type:
        payload["agent_type"] = agent_type
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=REPOSITORY, AGENT_HOOK_DETECTIONS=detections)
    environment.pop("SINGLEFS_HEAVY_TESTS", None)
    completed = subprocess.run(["bash", HOOK], input=json.dumps(payload), capture_output=True, text=True, env=environment)
    return completed.returncode, completed.stderr.strip().split("\n")[0][:150]


def cell(kind, label, command, want, cwd=REPOSITORY):
    exit_code, first_line = hook(command, cwd)
    holds = (exit_code == 2) if want == "refuse" else (exit_code == 0)
    cells.expect(kind, f"{label} ⇒ 应当{'拒（退 2）' if want == 'refuse' else '放行（退 0）'}", holds,
                 f"退 {exit_code}；「{command[:150]}」{('；' + first_line) if first_line else ''}")


try:
    cell("CONTROL", "C1 点名 c561 目标、--include-ignored --exact 全量用例", f"{MC} cargo test --release -p singlefs-harness --test {C561} -- --include-ignored --exact {SIGMA}", "refuse")
    cell("CONTROL", "C2 --test=<名> 的等号写法、--ignored", f"{MC} cargo test -p singlefs-harness --test={C561} -- --ignored", "refuse")
    cell("CONTROL", "C3 cargo +stable t 与通配", f"{MC} cargo +stable t -p singlefs-harness --test 'record_checker_*' -- --ignored", "refuse")
    cell("CONTROL", "C4 直接执行 deps 下的测试二进制带 --ignored", f"{MC} ./{BIN} --ignored", "refuse")
    cell("CONTROL", "C5 在包目录里不带 -p", f"{MC} cargo test --test {FLOOR} -- --ignored", "refuse",
         cwd=os.path.join(REPOSITORY, "crates/singlefs-harness"))
    cell("CONTROL", "K1 点名 c561 目标、不带 --ignored（只跑快用例）", f"{MC} cargo test -p singlefs-harness --test {C561}", "allow")
    cell("CONTROL", "K2 别的测试目标带 --ignored", f"{MC} cargo test -p singlefs-harness --test second_transaction_step_one_overwrite -- --ignored", "allow")
    bypasses = [
        ("B1 cargo nextest（本机没装，装了就能跑）", f"{MC} cargo nextest run -p singlefs-harness --run-ignored all -E 'test(={SIGMA})'"),
        ("B2 cargo mutants（本机装了 27.1.0；尾参交给 cargo test）", f"cargo mutants -p singlefs-harness -- --test {C561} -- --include-ignored"),
        ("B3 cargo --config 定一个别名再用别名", f"{MC} cargo --config 'alias.xt=\"test\"' xt -p singlefs-harness --test {C561} -- --ignored"),
        ("B4 环境变量里的 runner 替测试二进制加 --include-ignored", f"CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER={X}/add-ignored.sh {MC} cargo test -p singlefs-harness --test {C561}"),
        ("B5 --config 里的 runner", f"{MC} cargo --config 'target.x86_64-unknown-linux-gnu.runner=[\"{X}/add-ignored.sh\"]' test -p singlefs-harness --test {C561}"),
        ("B6 把测试二进制拷到别处再执行", f"cp {BIN} {X}/rc && {MC} {X}/rc --ignored"),
        ("B7 find -exec 执行测试二进制", f"{MC} find target/release/deps -name '{C561}-*' -type f -executable -exec {{}} --ignored ';'"),
        ("B8 /usr/bin/time -v 包一层（量峰值内存的常见写法）", f"{MC} /usr/bin/time -v cargo test -p singlefs-harness --test {C561} -- --ignored"),
        ("B9 flock 包一层", f"{MC} flock {X}/lock cargo test -p singlefs-harness --test {C561} -- --ignored"),
        ("B10 rustup run stable 包一层", f"{MC} rustup run stable cargo test -p singlefs-harness --test {C561} -- --ignored"),
        ("B11 chrt 包一层", f"{MC} chrt -i 0 cargo test -p singlefs-harness --test {C561} -- --ignored"),
        ("B12 systemd-run --scope 自己设内存上限", f"systemd-run --user --scope -q -p MemoryMax=8G cargo test -p singlefs-harness --test {C561} -- --ignored"),
        ("B13 prlimit 包一层", f"{MC} prlimit --as=8000000000 cargo test -p singlefs-harness --test {C561} -- --ignored"),
    ]
    for label, command in bypasses:
        cell("ATTACK", label, command, "refuse")
    overs = [
        ("R1 --ignored --list 只列用例、一条都不跑", f"{MC} cargo test -p singlefs-harness --test {C561} -- --ignored --list"),
        ("R2 --include-ignored 加过滤、只跑一条快用例", f"{MC} cargo test -p singlefs-harness --test {C561} -- --include-ignored --exact {FAST}"),
    ]
    for label, command in overs:
        cell("ATTACK", label, command, "allow")

    # 同样的包装换成名字带 layer0 的目标：看缺口是这一批新加的，还是闸早就有的（INFO，不计入判定）
    for label, command in [("L1 /usr/bin/time -v 包层 0 目标", f"{MC} /usr/bin/time -v cargo test -p singlefs-harness --test first_transaction_step_seven_layer0"),
                           ("L2 cargo mutants 整包", "cargo mutants -p singlefs-harness"),
                           ("L3 --no-run 只编层 0 目标", f"{MC} cargo test --no-run -p singlefs-harness --test first_transaction_step_seven_layer0")]:
        exit_code, first_line = hook(command)
        cells.expect("INFO", label, True, f"退 {exit_code}；「{command[:120]}」")

    # I：登记的用例函数的 #[ignore] 被去掉之后（实现员调试时常这么干），「不带 --ignored」就跑到全量，闸与登记自查都看不出
    copy = os.path.join(scratch, "copy")
    for relative in ["Cargo.toml", ".claude/gate.d/stage-inputs.tsv"] + [f"crates/{name}/Cargo.toml" for name in
                                                                          ("singlefs-format", "singlefs-core", "singlefs-harness", "singlefs-checker")] \
            + [f"crates/singlefs-harness/tests/{target}.rs" for target in
               (C561, FLOOR, "first_transaction_step_seven_layer0", "second_transaction_step_zero_layer0")]:
        os.makedirs(os.path.dirname(os.path.join(copy, relative)), exist_ok=True)
        shutil.copy(os.path.join(REPOSITORY, relative), os.path.join(copy, relative))
    c561_copy = os.path.join(copy, f"crates/singlefs-harness/tests/{C561}.rs")
    with open(c561_copy, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    function_line = next(index for index, line in enumerate(lines) if f"fn {SIGMA}(" in line)
    removed = lines[function_line - 1] if lines[function_line - 1].lstrip().startswith("#[ignore") else None
    cells.expect("CONTROL", "I0 c561 的全量用例函数上一行是 #[ignore …]（今天的源码）", removed is not None, f"上一行「{lines[function_line - 1][:80]}」")
    if removed is not None:
        del lines[function_line - 1]
        with open(c561_copy, "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines))
        cell("ATTACK", "I1 副本里去掉那一行 #[ignore] 之后，点名 c561 目标、不带 --ignored（会跑 2^18 个状态）", f"{MC} cargo test -p singlefs-harness --test {C561}",
             "refuse", cwd=copy)
        completed = subprocess.run([sys.executable, ADMISSION, "crash-cases", copy],
                                   capture_output=True, text=True)
        cells.expect("ATTACK", "I2 同一个副本上 admission.py crash-cases ⇒ 应当退 2、点出登记的用例函数没标 #[ignore]", completed.returncode == 2,
                     f"退 {completed.returncode}；stdout 首行「{completed.stdout.strip().split(chr(10))[0][:120]}」")
finally:
    shutil.rmtree(scratch, ignore_errors=True)
sys.exit(cells.finish())
