#!/usr/bin/env bash
# PreToolUse hook（Agent）：派 experiment-runner 时，派发提示必须说清这一段为哪几行岔路而跑。
#
# 为什么：2026-09-17 两个计数实验派了十一次执行员，每次交回主 agent 只问「还缺什么」就接着派；
# 岔路 1、3、4、7 能判的数在开跑四个多小时时已经齐了，之后两个多小时的数不改变任何选择
# （records/2026-09-17-已分配口径三方与两个实验.md 第六节）。规则写在 .claude/rules/three-way-inference.md
# 「交岔路时写岔路单，派实验时带上它」；这道闸让「续派之前先对着岔路单判够不够」跳不过去。
#
# 判法：不是派 experiment-runner 的放行；派发提示写着「只修锚点」的放行（那种活没有登记与岔路）。
#   ① 派发提示里没有「这一段回答的岔路：<非空>」一行，拒绝；
#   ② 登记对应的实验已经有实验页（.claude/kb/experiments/<号>-*.md，第一段交回才会有）算续做：
#      没有「上一段岔路表里还差：<非空>」一行、或写的是「无 / 没有」，拒绝——一行都不差就该交岔路表，不续派。
# 实验号从派发提示里的 research/prompts/e<号>-preregistration.md 或 e<号>-r<n>-prereg.md 取。
#
#   runner-dispatch-guard.sh             # 从 stdin 读 hook 的 JSON
#   runner-dispatch-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝；RUNNER_DISPATCH_GUARD_DISABLE_CHECK=1 时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 agent-write-scope.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import glob, json, os, re, shutil, subprocess, sys, tempfile

NONE_WORDS = {"", "无", "没有", "无。", "-", "—", "空", "none"}

def labelled_value(prompt, label):
    match = re.search(label + r"[：:]\s*(.*)", prompt)
    return None if match is None else match.group(1).strip()

def decide(hook_input, project_root):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    if os.environ.get("RUNNER_DISPATCH_GUARD_DISABLE_CHECK") == "1":
        return 0, None
    tool_input = hook_input.get("tool_input") or {}
    if tool_input.get("subagent_type") != "experiment-runner":
        return 0, None
    prompt = tool_input.get("prompt") or ""
    if "只修锚点" in prompt:
        return 0, None
    answered = labelled_value(prompt, "这一段回答的岔路")
    if answered is None or answered in NONE_WORDS:
        return 2, ("✗ 派 experiment-runner 的提示里没写「这一段回答的岔路：…」。\n"
                   "→ 怎么办：对着岔路单（research/prompts/<轮>-forks.md）写明这一段的量让哪几行够判；"
                   "说不出是哪一行，这一段就不该派（.claude/rules/three-way-inference.md「交岔路时写岔路单，派实验时带上它」）。")
    number_match = re.search(r"research/prompts/e(\d+)-(?:preregistration|r\d+-prereg)\.md", prompt)
    if number_match is None:
        return 0, None
    pages = glob.glob(os.path.join(project_root, ".claude", "kb", "experiments", f"{int(number_match.group(1))}-*.md"))
    if not pages:
        return 0, None
    remaining = labelled_value(prompt, "上一段岔路表里还差")
    if remaining is None:
        return 2, (f"✗ E{number_match.group(1)} 已经有实验页，这是续做，派发提示里却没写「上一段岔路表里还差：…」。\n"
                   "→ 怎么办：先读上一段交回的岔路表，把还开着的行点名写进提示；一行都不差就停下交岔路表，不续派。")
    if remaining in NONE_WORDS:
        return 2, (f"✗ E{number_match.group(1)} 上一段岔路表里一行都不差了，不续派。\n"
                   "→ 怎么办：按岔路单交岔路表给用户；剩下的量在实验页标「够判后未跑」，状态写「已跑（够判）」。")
    return 0, None

def selftest(hook_dir):
    work = tempfile.mkdtemp()
    try:
        os.makedirs(os.path.join(work, ".claude", "kb", "experiments"))
        open(os.path.join(work, ".claude", "kb", "experiments", "153-样本.md"), "w").write("## E153 样本 —— 部分已跑\n")
        first = "跑前登记：research/prompts/e160-preregistration.md\n"
        again = "跑前登记：research/prompts/e153-preregistration.md\n"
        def case(label, subagent_type, prompt, want):
            hook_input = {"tool_name": "Agent", "tool_input": {"subagent_type": subagent_type, "prompt": prompt}}
            return (label, want, decide(hook_input, work)[0])
        cases = [
            case("不是执行员", "experiment-designer", "随便", 0),
            case("只修锚点", "experiment-runner", "只修锚点：表 e153", 0),
            case("第一段点名了岔路", "experiment-runner", first + "这一段回答的岔路：岔路 1、岔路 4\n", 0),
            case("第一段没点名岔路", "experiment-runner", first, 2),
            case("第一段岔路写无", "experiment-runner", first + "这一段回答的岔路：无\n", 2),
            case("续做点名了还差的行", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n", 0),
            case("续做没写还差", "experiment-runner", again + "这一段回答的岔路：岔路 1\n", 2),
            case("续做还差写无", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：无\n", 2),
            case("重跑登记也算续做", "experiment-runner", "research/prompts/e153-r2-prereg.md\n这一段回答的岔路：岔路 3\n", 2),
        ]
        script = os.path.join(hook_dir, "runner-dispatch-guard.sh")
        for label, prompt, want in (("stdin:续做没写还差", again + "这一段回答的岔路：岔路 1\n", 2),
                                    ("stdin:第一段点名了岔路", first + "这一段回答的岔路：岔路 1\n", 0)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-runner", "prompt": prompt}}),
                                       capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
            cases.append((label, want, completed.returncode))
    finally:
        shutil.rmtree(work)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() 的判法；RUNNER_DISPATCH_GUARD_DISABLE_CHECK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过：非执行员与只修锚点放行，没点名岔路、续做没写还差或还差写无的拒绝（查了 {len(cases)} 种）")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError:
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    code, message = decide(hook_input, project_root)
    if code:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
