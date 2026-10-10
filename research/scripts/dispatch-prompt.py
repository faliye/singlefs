#!/usr/bin/env python3
# admission: always 每一次调都按此刻的定义打骨架或核一份提示，上一次的结论不替这一次作保
# run-condition: command bash
"""派发提示的骨架：按定义的 required-inputs 与共用约束要的固定行打出来，主 agent 只填内容；核一份提示还剩不剩占位。

用法：
    dispatch-prompt.py <类型> [--set 名=值 …] [--batch <调度记录>#<批号>] [--out 文件]   # 打骨架；--set 直接填某一行
    dispatch-prompt.py --check <提示文件> [--type <类型>] [--model <模型>] [--root <仓根>]  # 派之前一次列全派发闸会拒的地方、退 3
    dispatch-prompt.py --selftest                                                      # 下面的弄坏开关设了哪一个都必须判红

--check 的判法不在这里另写：知道类型（--type，或骨架第一行「# 派发：<类型>」）就起派发闸自己的
`bash .claude/hooks/runner-dispatch-guard.sh --check-prompt <提示文件> --type <类型> --root <仓根>`，同一份判法各条都判完，
不成立的一次列全（占位、重型测试一行、定义要的输入、写范围、快照、长活声明、书记员规格……）；⑤ 与在跑的实现员撞文件、⑦ 并发与限额窗口
要会话状态，写成「派的那一刻另判」。不知道类型、或派发闸跑不起来的，只查占位（<填 …>），并写明别几条没核。

为什么：2026-09-20 起 8 天里派发闸拒了 187 次，前几种原因全是缺一行（岔路行、要改的文件、出口、还差）；主 agent 派完再给子 agent
发了 573 条纠正或追加。缺项该在写的时候补，不该在派的时候拒：骨架把定义要的每一样都列成一行占位，占位没填完就不派。
骨架里的固定行与派发闸（.claude/hooks/runner-dispatch-guard.sh）判的是同一套：重型测试一行、开工先读（没有定义的类型）、
定义的 required-inputs 每组一行、实现员的「要动的 crates 文件」、写回员的「规格检查已判」、执行员的「这一段回答的岔路」、
设计员与执行员的「预计挂钟」与超过 2 小时时的「分片」「续跑」「多机」三行（派发闸的判法在 .claude/hooks/lib_long_run.py）。
退出码：0 打出骨架，或 --check 判下来派发闸放行（不知道类型时：没有占位）；2 用法错、定义不在、提示文件读不了；3 --check 判下来派发闸会拒（或有占位）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句的开头，括号里是那条「✗ 自检：」的开头）：
  DISPATCH_PROMPT_BREAK=ignore-placeholders  placeholders() 一处占位都不认（不知道类型、只查占位的那一路）
      --check 带占位、不知道类型的骨架（「✗ 自检：--check 对带占位、不知道类型的骨架应当退 3 并逐行列出」）
  DISPATCH_PROMPT_BREAK=guard-skipped  check_prompt() 知道类型也不起派发闸的 --check-prompt，只查占位
      --check 带类型一次列全（「✗ 自检：--check 带类型应当一次列全派发闸会拒的三处」）
  DISPATCH_PROMPT_BREAK=long-run-lines-dropped  skeleton() 不打预计挂钟与分片、续跑、多机那四行
      设计员与执行员的骨架带预计挂钟四行（「✗ 自检：设计员与执行员的骨架应当带预计挂钟与分片、续跑、多机四行」）
"""
import os
import re
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
BROKEN = os.environ.get("DISPATCH_PROMPT_BREAK", "")
PLACEHOLDER = re.compile(r"<填[^>]*>")
# --check 起的派发闸（判法只在它那里）与骨架第一行写的类型
DISPATCH_GUARD = os.path.join(REPOSITORY_ROOT, ".claude", "hooks", "runner-dispatch-guard.sh")
DISPATCH_HEADER = re.compile(r"\A# 派发：(\S+)")
GUARD_TIMEOUT_SECONDS = 120
HEAVY_TEST_OWNERS = set()   # 子 agent 一个都不跑重型测试，骨架一律带「重型测试：不跑」
HEAVY_WORK_TYPES = {"implementation-writer", "experiment-runner", "three-way-attack", "investigator", "tooling-writer", "clerk", "mutation-triage"}
# 定义之外、派发闸另判的行：类型 → 那一行的骨架
# 派设计员、执行员要的预计挂钟一行与超过 2 小时时的三行（派发闸 ⑭；只写 2 小时内的，后三行删掉）
LONG_RUN_LINES = ["预计挂钟：<填 2 小时内 / 超过 2 小时，后面接推的依据>",
                  "分片：<填 超过 2 小时才写：分片单元，几片怎么定；2 小时内删掉这一行>",
                  "续跑：<填 超过 2 小时才写：同一条命令重起只补没做完的，怎么认做完的；2 小时内删掉这一行>",
                  "多机：<填 超过 2 小时才写：台数从 layer0-shard-configuration-check.sh --emit-hosts 读、每台的线程与内存从 host-resources.py --json 现算，"
                  "不写常量；分不了的这一行改写成「分不了：」点名缺的分片、续跑、多机与为什么；2 小时内删掉这一行>"]
EXTRA_LINES = {
    "implementation-writer": ["要动的 crates 文件：<填 从仓根起逐个列全，不与在跑的实现员撞>"],
    "kb-scribe": ["规格检查已判：写回员自己起草并跑 kb-spec-check.py"],
    "experiment-runner": ["这一段回答的岔路：<填 岔路单里的哪几行>", "上一段岔路表里还差：<填 续做时写还差的行；第一段删掉这一行>", *LONG_RUN_LINES],
    "experiment-designer": [*LONG_RUN_LINES],
}


def frontmatter_fields(definition_path):
    fields = {}
    try:
        lines = open(definition_path, encoding="utf-8").read().split("\n")
    except OSError:
        return None
    if not lines or lines[0].strip() != "---":
        return None
    for line in lines[1:]:
        if line.strip() == "---":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


def skeleton(agent_type, agents_dir, settings, batch):
    fields = frontmatter_fields(os.path.join(agents_dir, f"{agent_type}.md"))
    lines = [f"# 派发：{agent_type}", ""]
    if fields is None:
        lines.append("开工先读 `.claude/agent-common.md`")
    if agent_type not in HEAVY_TEST_OWNERS:
        lines.append("重型测试：不跑")
    lines.append(f"批次：{batch or '<填 调度记录#批号>'}")
    if agent_type in HEAVY_WORK_TYPES:
        lines.append(f"线程上限：{settings.get('线程上限', '<填 整机核数 ÷ 同时在跑的重活数>')}")
    required = (fields or {}).get("required-inputs", "")
    emitted = set()
    for group in [piece.strip() for piece in required.split(",") if piece.strip()]:
        name = group.split("|", 1)[0].strip()
        emitted.add(name)
        lines.append(f"{name}：{settings.get(name, '<填 ' + group + '>')}")
    for extra in EXTRA_LINES.get(agent_type, []):
        if BROKEN == "long-run-lines-dropped" and extra in LONG_RUN_LINES:
            continue
        key = extra.split("：", 1)[0]
        if key in emitted:
            continue   # 定义的 required-inputs 已经要了这一行，不重复
        lines.append(f"{key}：{settings[key]}" if key in settings else extra)
    lines += ["", "## 关的是哪几条", "", settings.get("关的是哪几条", "<填 逐条列已经抓到的问题，每条各自的验收标准>"), "",
              "## 出口", "", settings.get("出口", "<填 做到什么算完、交什么到哪>"), ""]
    return "\n".join(lines) + "\n"


def placeholders(text):
    if BROKEN == "ignore-placeholders":
        return []
    return [(number, line) for number, line in enumerate(text.split("\n"), 1) if PLACEHOLDER.search(line)]


def guard_check(path, agent_type, root, model):
    """起派发闸的 --check-prompt：→ (退出码 0 / 3, None) 判成了（输出已经原样打出）；(None, 没判成的原因) 起不来或输出认不出。"""
    command = ["bash", DISPATCH_GUARD, "--check-prompt", path, "--type", agent_type, "--root", root or REPOSITORY_ROOT]
    if model:
        command += ["--model", model]
    try:
        completed = subprocess.run(command, capture_output=True, text=True, timeout=GUARD_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return None, repr(error)
    if completed.returncode not in (0, 2) or not completed.stdout.strip() or "用法：runner-dispatch-guard.sh" in completed.stdout:
        return None, f"退出码 {completed.returncode}：{(completed.stdout + completed.stderr).strip()[-300:]}"
    sys.stdout.write(completed.stdout)
    if completed.stderr.strip():
        sys.stdout.write(completed.stderr)
    return (3 if completed.returncode == 2 else 0), None


def check_prompt(path, agent_type=None, root=None, model=None):
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as error:
        print(f"  ✗ 读不了提示文件 {path}：{error}\n  → 怎么办：给一个存在的提示文件路径")
        return 2
    header = DISPATCH_HEADER.match(text)
    agent_type = agent_type or (header.group(1) if header else None)
    if agent_type and BROKEN != "guard-skipped":
        code, failure = guard_check(path, agent_type, root, model)
        if code is not None:
            return code
        print(f"  ! 派发闸的 --check-prompt 没跑成（{failure}），这一次只查了占位，别几条派的那一刻才判")
    elif not agent_type:
        print("  ! 没给类型（--type <类型>，或骨架第一行「# 派发：<类型>」），派发闸的别几条没核，这一次只查了占位")
    found = placeholders(text)
    for number, line in found:
        print(f"  ✗ 第 {number} 行还是占位：{line.strip()}")   # gate-lint:detail
    if found:
        print(f"  → 怎么办：把这 {len(found)} 处 <填 …> 换成实际内容再派；确实不适用的整行删掉")
        return 3
    print(f"  ✓ 提示里没有占位（查了 {len(text.split(chr(10)))} 行）")
    return 0


def selftest():
    work = tempfile.mkdtemp(prefix="dispatch-prompt-selftest-")
    failures, checked = [], 0
    try:
        agents = os.path.join(work, ".claude", "agents")
        os.makedirs(agents)
        open(os.path.join(agents, "sample-writer.md"), "w", encoding="utf-8").write(
            "---\nname: sample-writer\nmodel: sonnet\nrequired-inputs: 草稿目录, 报告|报告路径, 条款\n---\n")
        text = skeleton("sample-writer", agents, {"草稿目录": "~/.local/share/singlefs/work/x/"}, None)
        checked += 1
        wanted = ["重型测试：不跑", "批次：<填 调度记录#批号>", "草稿目录：~/.local/share/singlefs/work/x/", "报告：<填 报告|报告路径>", "条款：<填 条款>", "## 出口"]
        missing = [piece for piece in wanted if piece not in text]
        if missing or "开工先读" in text:
            failures.append(f"有定义的类型：骨架缺 {missing}，或多了没定义类型才有的开工先读行：\n{text}")
        text_builtin = skeleton("Explore", agents, {}, "records/x.md#1")
        checked += 1
        if "开工先读 `.claude/agent-common.md`" not in text_builtin or "批次：records/x.md#1" not in text_builtin:
            failures.append(f"没有定义的类型应当带开工先读一行、--batch 填进批次行：\n{text_builtin}")
        open(os.path.join(agents, "implementation-writer.md"), "w", encoding="utf-8").write(
            "---\nname: implementation-writer\nmodel: opus\nrequired-inputs: 草稿目录, 报告, 条款, 要动的 crates 文件\n---\n")
        text_writer = skeleton("implementation-writer", agents, {}, None)
        checked += 1
        if text_writer.count("要动的 crates 文件：") != 1 or "线程上限：<填" not in text_writer:
            failures.append(f"实现员骨架应当带线程上限，「要动的 crates 文件」只一行（定义已经要了就不再加）：\n{text_writer}")
        long_run_keys = ["预计挂钟：<填", "分片：<填", "续跑：<填", "多机：<填"]
        for long_run_type in ("experiment-designer", "experiment-runner"):
            text_long_run = skeleton(long_run_type, agents, {}, None)
            checked += 1
            if any(text_long_run.count(key) != 1 for key in long_run_keys):
                failures.append(f"设计员与执行员的骨架应当带预计挂钟与分片、续跑、多机四行（{long_run_type}）：\n{text_long_run}")
        text_gate = skeleton("gate-triage", agents, {}, None)
        checked += 1
        if "重型测试：不跑" not in text_gate:
            failures.append("原来跑重型测试的类型名，骨架也要带「重型测试：不跑」")
        draft = os.path.join(work, "draft.md")
        open(draft, "w", encoding="utf-8").write(text.split("\n", 1)[1])   # 去掉「# 派发：」那一行：不知道类型，只查占位
        import io, contextlib
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_draft = check_prompt(draft)
        checked += 1
        if code_draft != 3 or "报告：<填" not in buffer.getvalue() or "没给类型" not in buffer.getvalue():
            failures.append(f"--check 对带占位、不知道类型的骨架应当退 3 并逐行列出、写明没给类型，实际 {code_draft}：{buffer.getvalue()[-300:]}")
        filled = os.path.join(work, "filled.md")
        open(filled, "w", encoding="utf-8").write(PLACEHOLDER.sub("已填", text))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_filled = check_prompt(filled, root=work)
        checked += 1
        if code_filled != 0:
            failures.append(f"--check 对填完的提示应当退 0（骨架第一行认出类型、派发闸放行），实际 {code_filled}：{buffer.getvalue()[-300:]}")
        # 带类型时起派发闸的 --check-prompt，同一份判法一次列全：缺重型测试一行、有占位、缺禁读清单（写的是「禁读：」）三处
        open(os.path.join(agents, "sample-attack.md"), "w", encoding="utf-8").write(
            "---\nname: sample-attack\nmodel: sonnet\nrequired-inputs: 禁读清单, 草稿目录, 攻击面\n---\n")
        attack = os.path.join(work, "attack.md")
        open(attack, "w", encoding="utf-8").write("草稿目录 ~/.local/share/singlefs/work/t/\n禁读：别的腿的产出\n攻击面：<填 攻击面>\n")
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code_attack = check_prompt(attack, "sample-attack", work)
        checked += 1
        wanted_attack = ["有 3 处不成立", "没有「重型测试：不跑」一行", "占位没填", "缺它定义要的输入：禁读清单", "禁读清单：<填 禁读清单>"]
        if code_attack != 3 or any(piece not in buffer.getvalue() for piece in wanted_attack):
            failures.append(f"--check 带类型应当一次列全派发闸会拒的三处（{wanted_attack}），实际 {code_attack}：{buffer.getvalue()[-600:]}")
    finally:
        import shutil
        shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 skeleton()、check_prompt() 与派发闸的 --check-prompt；DISPATCH_PROMPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项）：有定义的类型按 required-inputs 逐组一行、--set 填进去；没有定义的带开工先读；实现员带要动的文件与线程上限；"
          "每种类型都带重型测试行；设计员与执行员带预计挂钟与分片、续跑、多机四行；--check 不知道类型时只查占位（有占位退 3、写明没给类型），"
          "知道类型时起派发闸的 --check-prompt、会拒的几处一次列全退 3，填完的退 0")
    return 0


def main(argv):
    if argv[:1] == ["--selftest"]:
        return selftest()
    if argv[:1] == ["--check"]:
        options, rest = {}, argv[2:]
        while len(rest) >= 2 and rest[0] in ("--type", "--model", "--root"):
            options[rest[0]] = rest[1]
            rest = rest[2:]
        if len(argv) < 2 or argv[1].startswith("--") or rest:
            print("  ✗ --check 要一个提示文件，后面只认 --type <类型>、--model <模型>、--root <仓根>\n"
                  "  → 怎么办：dispatch-prompt.py --check <提示文件> --type <类型>（骨架第一行写了「# 派发：<类型>」的可以不给 --type）")
            return 2
        return check_prompt(argv[1], options.get("--type"), options.get("--root"), options.get("--model"))
    if not argv or argv[0].startswith("--"):
        print("  ✗ 用法：dispatch-prompt.py <类型> [--set 名=值 …] [--batch <调度记录>#<批号>] [--out 文件] | --check <提示文件> | --selftest\n"
              "  → 怎么办：第一个参数写 .claude/agents/ 里的类型名（clerk、implementation-writer …）")
        return 2
    agent_type, settings, batch, out = argv[0], {}, None, None
    rest = argv[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--set" and rest and "=" in rest[0]:
            name, value = rest.pop(0).split("=", 1)
            settings[name.strip()] = value.strip()
        elif flag == "--batch" and rest:
            batch = rest.pop(0)
        elif flag == "--out" and rest:
            out = rest.pop(0)
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --set 名=值、--batch <记录>#<号>、--out 文件")
            return 2
    agents_dir = os.path.join(REPOSITORY_ROOT, ".claude", "agents")
    if agent_type not in ("Explore",) and not os.path.isfile(os.path.join(agents_dir, f"{agent_type}.md")):
        print(f"  ✗ .claude/agents/{agent_type}.md 不在：这个类型没有项目定义\n  → 怎么办：一次性的活派 clerk；只读搜索派 Explore；别的类型先写定义")
        return 2
    text = skeleton(agent_type, agents_dir, settings, batch)
    if out:
        with open(out, "x", encoding="utf-8") as handle:
            handle.write(text)
        print(f"  ✓ 骨架写进 {out}（{len(placeholders(text))} 处占位待填）")
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
