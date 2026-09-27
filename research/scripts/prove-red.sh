#!/usr/bin/env bash
# admission: always 实现员每写一条新测试都要现证它会红，判的是这一刻副本里的源码与变异行
# run-condition: command python3 cargo
# 实现员的证红：在仓副本里逐条施加 crates/mutations.tsv 里点名的变异，经内存包装跑那一行指定的测试目标，看点名的测试红没红，再还原。
#
#   prove-red.sh --copy <仓副本> [--memory <上限>] <crate> <变异名…>
#   prove-red.sh --selftest            # 用假的 cargo 走一遍；PROVE_RED_BREAK=<项> 时必须判红
#
# 判法与次序：
#   ① --copy 必须是一份副本：目录在、里面有 crates/mutations.tsv、不是这份脚本所在的主工作区（在主工作区上改坏源码会波及别的会话）；
#   ② 逐个找变异名对应的行（六段：名、文件、原文、替换文、cargo test 的参数、必须红的测试名）；找不到、或参数里没有 -p <crate>、
#      没有 --lib / --test / --bin 之一的，整次拒绝（一条都不跑，退 2）——不挑目标的 cargo test 会跑到名字带 layer0 的测试二进制；
#   ③ --test 的目标名字里带 layer0 的跳过、逐条列出（层 0 归提交时的崩溃验证员）；
#   ④ 每组不同的参数先跑一次基线（不改源码），基线红就停（退 2）；
#   ⑤ 每条变异：原文在文件里恰好命中一次才改，经 research/scripts/run-with-memory-cap.sh <上限> 跑 cargo test <参数>，
#      输出里有「test …<必须红的测试名> ... FAILED」算抓到，编译失败算无效，其余算没红；跑完把原文件写回并 touch。
# 每条一行判定打到 stdout（名、结局、日志路径）；日志放 ${PROVE_RED_LOG_DIRECTORY:-<副本>/prove-red-logs}。
# 退出码：0 点名的每条都抓到（跳过的不算失败，照列）；1 有没红或无效；2 用法错、被拒或基线红；250–254 原样转内存包装的结局。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 /dev/fd/3 "$SCRIPT_DIRECTORY" "$@" 3<<'PY'
import os, re, shutil, subprocess, sys, tempfile

SCRIPT_DIRECTORY = sys.argv[1]
MAIN_ROOT = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))
MEMORY_WRAPPER = os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh")
BROKEN = os.environ.get("PROVE_RED_BREAK", "")
TARGET_OPTIONS = {"--lib", "--test", "--bin"}


def default_memory_cap():
    try:
        match = re.search(r'REPLAY_MEMORY_CAP="\$\{REPLAY_MEMORY_CAP:-([0-9]+[KMGT])\}"', open(os.path.join(SCRIPT_DIRECTORY, "replay.sh"), encoding="utf-8").read())
        return match.group(1) if match else "8G"
    except OSError:
        return "8G"


def load_rows(copy):
    rows = {}
    for line in open(os.path.join(copy, "crates", "mutations.tsv"), encoding="utf-8"):
        if line.startswith("#") or not line.strip():
            continue
        fields = line.rstrip("\n").split("\t")
        if len(fields) == 6:
            rows[fields[0]] = fields
    return rows


def refusal_for(row, crate):
    arguments = row[4].split()
    before_separator = arguments[: arguments.index("--")] if "--" in arguments else arguments
    if BROKEN != "any-target" and not TARGET_OPTIONS & set(before_separator):
        return f"参数「{row[4]}」没挑目标（--lib / --test / --bin 都没有），会跑到名字带 layer0 的测试二进制"
    if "-p" not in before_separator or before_separator[before_separator.index("-p") + 1:before_separator.index("-p") + 2] != [crate]:
        return f"参数「{row[4]}」不是 -p {crate}"
    return None


def layer0_target(row):
    arguments = row[4].split()
    return any(argument == "--test" and index + 1 < len(arguments) and "layer0" in arguments[index + 1]
               for index, argument in enumerate(arguments))


def run_cargo(copy, arguments, memory, log_path, cargo):
    with open(log_path, "w", encoding="utf-8") as log:
        result = subprocess.run(["bash", MEMORY_WRAPPER, memory, cargo, "test", *arguments], cwd=copy, stdout=log, stderr=subprocess.STDOUT)
    return result.returncode, open(log_path, encoding="utf-8", errors="replace").read()


def prove(copy, memory, crate, names, cargo="cargo"):
    """→ (退出码, [输出行])。"""
    copy = os.path.realpath(copy)
    if not os.path.isfile(os.path.join(copy, "crates", "mutations.tsv")):
        return 2, [f"✗ {copy} 里没有 crates/mutations.tsv，不像仓副本", "→ 怎么办：rsync -a --exclude target --exclude .git <仓根>/ <草稿目录>/copy/ 拷一份再给 --copy"]
    if copy == MAIN_ROOT and BROKEN != "main-allowed":
        return 2, [f"✗ --copy 指的是主工作区 {copy}", "→ 怎么办：在草稿目录里拷一份仓副本（rsync -a --exclude target --exclude .git），证红只在副本上改坏源码"]
    rows = load_rows(copy)
    refusals, chosen, skipped = [], [], []
    for name in names:
        row = rows.get(name)
        if row is None:
            refusals.append(f"{name}：crates/mutations.tsv 里没有这一行")
            continue
        why = refusal_for(row, crate)
        if why:
            refusals.append(f"{name}：{why}")
        elif layer0_target(row) and BROKEN != "layer0-run":
            skipped.append(f"{name}：目标名字带 layer0，留给提交时的崩溃验证员")
        else:
            chosen.append(row)
    if refusals:
        return 2, [f"✗ {line}" for line in refusals] + ["→ 怎么办：给那一行的 cargo test 参数加上 -p <crate> 与 --lib / --test <目标> / --bin <名> 之一再证；一条都没跑"]
    logs = os.environ.get("PROVE_RED_LOG_DIRECTORY") or os.path.join(copy, "prove-red-logs")
    os.makedirs(logs, exist_ok=True)
    output = [f"- 跳过 {line}" for line in skipped]
    for arguments in sorted({row[4] for row in chosen}):
        code, text = run_cargo(copy, arguments.split(), memory, os.path.join(logs, "baseline.log"), cargo)
        if 250 <= code <= 254:
            return code, output + [f"✗ 基线那一次撞了内存包装的结局（退出码 {code}，含义见 run-with-memory-cap.sh 文件头）", "→ 怎么办：照那一行的含义处理，不绕开包装重跑"]
        if code != 0 and BROKEN != "baseline-ignored":
            return 2, output + [f"✗ 基线（没改源码）就红：cargo test {arguments}（日志 {logs}/baseline.log）", "→ 怎么办：先让基线绿；基线里本来就红的测试不能拿来证红"]
    failed = False
    for index, row in enumerate(chosen, 1):
        name, relative, original, replacement, arguments, must_red = row
        path = os.path.join(copy, relative)
        original_text = open(path, encoding="utf-8").read()
        anchor = original.replace("\\n", "\n")
        if original_text.count(anchor) != 1:
            output.append(f"✗ {name}：原文在 {relative} 里命中 {original_text.count(anchor)} 次，要恰好 1 次")
            failed = True
            continue
        log_path = os.path.join(logs, f"{index:03d}.log")
        try:
            open(path, "w", encoding="utf-8").write(original_text.replace(anchor, replacement.replace("\\n", "\n"), 1))
            code, text = run_cargo(copy, arguments.split(), memory, log_path, cargo)
        finally:
            open(path, "w", encoding="utf-8").write(original_text)
            os.utime(path)
        if 250 <= code <= 254:
            output.append(f"✗ {name}：内存包装的结局，退出码 {code}（日志 {log_path}）")
            failed = True
        elif re.search(r"^test (?:\S*::)?" + re.escape(must_red) + r" \.\.\. FAILED", text, re.M) and BROKEN != "never-caught":
            output.append(f"{name}\t抓到\t{must_red} 红了（日志 {log_path}）")
        elif re.search(r"^error(?:\[E\d+\])?:", text, re.M) or "could not compile" in text:
            output.append(f"✗ {name}\t无效\t替换之后编不过（日志 {log_path}）")
            failed = True
        else:
            output.append(f"✗ {name}\t没红\t{must_red} 没判红（日志 {log_path}）")
            failed = True
    summary = f"点名 {len(names)} 条：跑了 {len(chosen)} 条，跳过 {len(skipped)} 条"
    if failed:
        return 1, output + [f"✗ {summary}，有没红或无效的（逐条列在上面）", "→ 怎么办：没红的换一条真会改行为的变异或补断言；无效的改替换文；照 .claude/rules/mutation-sampling.md 分类"]
    return 0, output + [f"✓ {summary}，跑的都抓到了"]


def selftest():
    work = tempfile.mkdtemp(prefix="prove-red-selftest-")
    try:
        copy = os.path.join(work, "copy")
        os.makedirs(os.path.join(copy, "crates", "demo", "src"))
        open(os.path.join(copy, "crates", "demo", "src", "lib.rs"), "w").write("fn answer() -> u32 { 42 }\n")
        table = ("# 样本\n"
                 "caught\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                 "untargeted\tcrates/demo/src/lib.rs\t{ 42 }\t{ 43 }\t-p demo -- answer\tanswer_is_42\n"
                 "layer\tcrates/demo/src/lib.rs\t{ 42 }\t{ 44 }\t-p demo --test demo_layer0 -- x\tx\n")
        open(os.path.join(copy, "crates", "mutations.tsv"), "w").write(table)
        fake_bin = os.path.join(work, "bin")
        os.makedirs(fake_bin)
        fake_cargo = os.path.join(fake_bin, "cargo")
        open(fake_cargo, "w").write("#!/usr/bin/env bash\nif grep -q '{ 42 }' crates/demo/src/lib.rs; then echo 'test tests::answer_is_42 ... ok'; exit 0; fi\n"
                                    "echo 'test tests::answer_is_42 ... FAILED'; exit 101\n")
        os.chmod(fake_cargo, 0o755)
        fake_wrapper = os.path.join(work, "wrapper.sh")
        open(fake_wrapper, "w").write("#!/usr/bin/env bash\nshift\nexec \"$@\"\n")
        global MEMORY_WRAPPER
        MEMORY_WRAPPER = fake_wrapper
        os.environ["PROVE_RED_LOG_DIRECTORY"] = os.path.join(work, "logs")
        failures = []
        checked_cases = 0
        code, lines = prove(copy, "1G", "demo", ["caught"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any("\t抓到\t" in line for line in lines):
            failures.append(f"会红的变异应当抓到、退 0，实际 {code} {lines}")
        checked_cases += 1
        if open(os.path.join(copy, "crates", "demo", "src", "lib.rs")).read() != "fn answer() -> u32 { 42 }\n":
            failures.append("跑完没把源码写回原样")
        code, lines = prove(copy, "1G", "demo", ["caught", "untargeted"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("没挑目标" in line for line in lines):
            failures.append(f"不挑目标的变异行应当整次拒绝、退 2，实际 {code} {lines}")
        code, lines = prove(copy, "1G", "demo", ["layer"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any("跳过" in line and "layer0" in line for line in lines):
            failures.append(f"目标带 layer0 的应当跳过并列出，实际 {code} {lines}")
        code, lines = prove(MAIN_ROOT, "1G", "demo", ["caught"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("主工作区" in line for line in lines):
            failures.append(f"--copy 指主工作区应当以「主工作区」为由拒绝，实际 {code} {lines}")
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 prove() / refusal_for() / layer0_target() 的判法；PROVE_RED_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ prove-red 自检通过：会红的抓到并写回源码，不挑目标的整次拒绝，layer0 目标跳过并列出，主工作区拒绝（查了 {checked_cases} 种）")
        return 0
    finally:
        shutil.rmtree(work, ignore_errors=True)


def main():
    arguments = sys.argv[2:]
    if arguments == ["--selftest"]:
        return selftest()
    copy, memory = None, default_memory_cap()
    while arguments and arguments[0] in ("--copy", "--memory"):
        if len(arguments) < 2:
            print("✗ 选项后面没给值\n→ 怎么办：prove-red.sh --copy <仓副本> [--memory <上限>] <crate> <变异名…>")
            return 2
        if arguments[0] == "--copy":
            copy = arguments[1]
        else:
            memory = arguments[1]
        arguments = arguments[2:]
    if copy is None or len(arguments) < 2:
        print("✗ 用法不对\n→ 怎么办：prove-red.sh --copy <仓副本> [--memory <上限>] <crate> <变异名…>")
        return 2
    code, lines = prove(copy, memory, arguments[0], arguments[1:])
    for line in lines:
        print(line)
    return code


sys.exit(main())
PY
