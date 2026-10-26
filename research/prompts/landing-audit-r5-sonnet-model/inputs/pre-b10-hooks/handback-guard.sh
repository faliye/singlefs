#!/usr/bin/env bash
# PreToolUse hook（SubagentHandback）：子 agent 交回之前拦六样——自己起的后台任务还在跑（全部子 agent），交回正文太长、三方腿报告里的引文对不上、
# 书记员与执行员的绿行不报数、实现员执行员工具员改了 .rs 没跑命名检查、写回员规格的目标不在又没写进「找不到的目标」（这五样只判项目子 agent）。
# hook-events: PreToolUse:SubagentHandback
# gate-similar: handback-scratch-check.sh 同挂 PreToolUse[SubagentHandback]，但它判的是临时目录里自己建的编译目录与仓副本删没删（上游副本，本仓不许就地改），判据与这里四样没有一条相同（④ 判的是自己起的后台任务还在不在跑，不是临时目录），并进去要改上游
# gate-similar: continuation-guard.sh 也按 agent 号去主会话目录下找子 agent 的会话记录，但它挂 SendMessage、判收件人交回过没有；这里只取子 agent 第一条记录的时刻当「开工时刻」，找法两行，不值得抽共用
# admission: always Claude Code 在子 agent 每次调交回工具之前调它，判的是这一次交回的正文与它点名的报告
# run-condition: command python3
#
# 判法。任一条成立就拒（退出 2，stderr 写原因与出路）：
#   ④ 全部子 agent（输入里有 agent_id、不是 main；general-purpose 也判，主 agent 不判）：它自己起的后台任务（它会话记录
#      <主会话记录去掉 .jsonl>/subagents/agent-<agent_id>.jsonl 里 run_in_background 的工具结果、或前台跑满超时被挪进后台的，带输出文件路径的那些）
#      此刻还有进程开着输出文件，就是还在跑：拒，按任务列出最上层的进程号、已跑多久、命令与整棵子树从最底层往上的停止次序，
#      出路是结束本轮等完成通知再交回，或按进程号逐个停掉再交回。认法与看门狗「交回之后后台还在跑」同一套，从 scripts/agents/agent-watch.py 导入
#      （background_start_in_result、own_background_tasks_still_running），不另抄一份。命令里自己用 `&`、`nohup` 放出去又改了输出去向的进程认不出（与看门狗同一处射程）。
#      找不到它的会话记录、导入不了 agent-watch.py 就不判这一条，stderr 写一句。
# 下面三条只判项目子 agent（`.claude/agents/<agent_type>.md` 在的）；主 agent、general-purpose 这类不判：
#   ① 交回正文超过 HANDBACK_CHARACTER_LIMIT 个字符（按 python 的 len 数，默认 4000；原 2000 时 8 天里拒了 67 次，每次都在最大的上下文上重写整份交回）：全文写进报告文件，交回只写结论、报告路径与 sha256；
#      拒绝说明写明文件名：三方腿写派发提示给的 research/prompts/<轮>-*-output.md，别的类型写派发提示给的报告路径，没给就写草稿目录的 result.md；
#   ② 三方腿（类型表只登记在定义 frontmatter 的 leg: three-way 一行，按项目根经同目录 lib_three_way_legs.py 读，citation_checked_agents；
#      派发闸、骨架与账表读同一处，拆分提案第九十节 ⑯；读不到那份库这一条不判，stderr 写一句）：交回里点名的 research/prompts/…-output.md 逐份交给
#      scripts/agents/cite-check.py（--root 项目根，--unchanged-since 取这个子 agent 会话记录第一条的时刻），有对不上的拒；交回里一份报告都没点名也拒；
#      找不到它的会话记录（认不出开工时刻）就不判这一条，stderr 写一句；
#   ③ kb-scribe / experiment-runner：交回正文与它点名的报告文件（research/prompts/、工作根或过渡期旧根下的 .md；两个根怎么认只在同目录 lib_work_root.py）里，以 ✓ 起头的行没有数字、
#      或写「查了 0 项」这类零项的，拒，逐行列出——绿行要连它报的查了多少项一起贴，0 项或没有数的按没判写。
#   ⑤ implementation-writer / experiment-runner / tooling-writer（拆分提案第九十节 ⑥，共用约束「门禁」一节「命名检查交回前跑」）：它自己的会话记录里
#      用 Edit / Write / MultiEdit 改成了 crates/ 或 scripts/ 下的 .rs（工具结果不是错误），改动所在的树是主工作区（CLAUDE_PROJECT_DIR）或带 .git 的副本（git worktree），
#      而在那棵树最后一次改 .rs 之后，没有真跑过的那两条命令：门禁 code-source-discipline.sh 的 --check 点名 vague-names、test-file-names、ordinal-names、
#      data-names 四格（不带 --check 的整道也算），与上游 naming-lint.sh；两条的根都要解到那棵树（根里的 ~、$HOME、$PWD、$(pwd) 按命令当时的目录展开；不给根时按它们自己的默认：前者取脚本往上两级，后者从当前目录往上找 .git）。
#      「真跑过」：Bash 调用有工具结果，不是错误、或错误是「Exit code N」（命名检查红了也算跑过）；被钩子拒的、超时的、没有结果的不算。只看命令，不看报告里写了什么。
#      命令认法：同目录 lib_shell_words.py 切命令位置（前缀、bash、capped.sh、run-with-memory-cap.sh 剥掉），lib_heavy_tests.py 的 command_under_launcher
#      剥 scripts/hosts/two-logs.sh 这类包装、checked_cells 读 --check。改动所在的树没有 .git 的（rsync 拷的副本，共用约束写明两条命令交主 agent 在补丁打进主工作区后跑）不判。
#      拒了就逐棵树列出最后改的 .rs、缺哪一条，给出两条命令原样（<根> 填那棵树）。认不出的：Bash 里写 .rs（mv、cp、重定向）、改完又被删掉的副本（看不到 .git，按 rsync 副本不判）。
#   ⑥ kb-scribe（拆分提案第九十节 ⑦，.claude/agents/kb-scribe.md「规格里的目标文件不存在」那一句）：规格里每一条的目标文件（kb-spec-check.py 的 load_spec /
#      parse_markdown_spec 认的两种写法，旧串、新串都在的条目才算）在主工作区（CLAUDE_PROJECT_DIR）不在，交回正文、交回点名的报告与派发提示「报告：」那一份里
#      都没有一节标题含「找不到的目标」、节里写着这个文件（从仓根起的路径或绝对路径）的，拒，逐个列出。规格从派发提示（它会话记录的第一条）里找：
#      提示点名的工作根或旧根下的 .json / .md / .txt（「报告：」那一份除外），「草稿目录：」下写回员自己起草的 spec.json / spec.md，提示正文本身（markdown 写法）。
#      一份规格都认不出的不判（写回员停在起草之前交回）；导入不了 kb-spec-check.py、读不了会话记录的不判，stderr 写一句。
# 这一道自己出错（读不了报告、cite-check 跑不起来、JSON 解析不了）不拒：放行，错误写进 stderr。
#
#   handback-guard.sh             # 从 stdin 读 hook 的 JSON
#   handback-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝
# 弄坏开关（只给自证判红用；自证里经 stdin 起的那一次入口照样带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里 cases 的标签，括号里是那条「✗ 自检：<标签>」失败行在标签之后的开头）：
#   HANDBACK_GUARD_BREAK=no-limit  decide() 不判 ① 交回正文的长度
#       超过上限的项目子 agent（「应当返回 2，实际 0」）
#       走真实 stdin 入口：超长交回（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=limit-hint-unnamed  ① 超长的拒绝说明退回改前那一句（只写「没给就写进草稿目录」，不写 result.md 与三方腿的文件名）
#       超长交回的拒绝说明写明报告文件名（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=no-cite  decide() 不判 ② 三方腿与核查员的引文
#       攻方腿引文行号错（「应当返回 2，实际 0」）
#       核查员没点名报告（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=cite-any-agent  ② 引文这一条对每个项目子 agent 都判，不只三方腿与核查员：交回里没点名 -output.md 的一律拒
#       正好在上限的项目子 agent（「应当返回 0，实际 2」）
#       回扫员不核引文（「应当返回 0，实际 2」）
#       书记员绿行带数（「应当返回 0，实际 2」）
#       回扫员的绿行不判（「应当返回 0，实际 2」）
#       起过的后台任务都跑完了（「应当返回 0，实际 2」）
#       那个后台任务结束之后再交回放行（「应当返回 0，实际 2」）
#       ⑤ ⑥ 那几格里应当放行的项目子 agent（改了 .rs 之后两条命名检查都跑了放行、写回员规格目标都在放行……）（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=no-green-count  green_count_problems() 不判 ③ 绿行没有数
#       书记员绿行没数（「应当返回 2，实际 0」）
#       执行员报告文件里的绿行没数（「应当返回 2，实际 0」）
#       执行员工作根下的报告里绿行没数（~ 写法）（「应当返回 2，实际 0」）
#       执行员过渡期旧根下的报告里绿行没数（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=zero-count-ok  green_count_problems() 不判 ③ 绿行报 0 项
#       书记员绿行 0 项（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=judge-general-purpose  is_project_agent() 把带 agent_type 的一律当项目子 agent（general-purpose 也判 ①②③）
#       不是项目定义的 agent 不判（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=no-background  background_problems() 一律交空表，不判 ④ 自己起的后台任务还在跑
#       交回时自己起的后台任务还在跑（项目子 agent）（「应当返回 2，实际 0」）
#       交回时自己起的后台任务还在跑（通用 agent 也判）（「应当返回 2，实际 0」）
#       后台还在跑的拒绝说明列出进程号、命令与两条出路（等完成通知、按进程号停）（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=no-naming-check  decide() 不判 ⑤ 改了 .rs 没跑命名检查
#       改了 .rs 没跑命名检查拒（「应当返回 2，实际 0」）
#       执行员改了 scripts/ 下 .rs 没跑命名检查拒（「应当返回 2，实际 0」）
#       命名检查跑在最后一次改 .rs 之前拒（「应当返回 2，实际 0」）
#       只跑了一条命名检查拒（「应当返回 2，实际 0」）
#       四格点名少一格不算跑过（「应当返回 2，实际 0」）
#       改了 .rs 没跑的拒绝说明给出两条命令（根填改动所在的树）（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=naming-root-unmatched  naming_check_runs() 认出的根一律当成别的树（跑在哪都不算跑在改动所在的树上）
#       改了 .rs 之后两条命名检查都跑了放行（「应当返回 0，实际 2」）
#       git worktree 副本里改的、命名检查根写成副本根放行（「应当返回 0，实际 2」）
#       经 two-logs.sh 包着跑命名检查放行（「应当返回 0，实际 2」）
#       cd 之后根写成 $(pwd)、接管道跑命名检查放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-any-file  naming_source_tree() 不看后缀，crates/ 或 scripts/ 下改什么文件都要命名检查
#       只改 .md 放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-copy-judged  naming_source_tree() 没有 .git 的副本（rsync 拷的）也判
#       rsync 副本里改的 .rs 交主 agent 放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=naming-refused-counts  tool_calls() 把被钩子拒的、没有结果的 Bash 也当成跑过
#       命名检查被钩子拒了不算跑过（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=no-scribe-targets  decide() 不判 ⑥ 写回员规格的目标不在
#       写回员规格目标不在、报告没写找不到的目标拒（「应当返回 2，实际 0」）
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#       写回员规格写在提示正文里、目标不在拒（「应当返回 2，实际 0」）
#       写回员自己起草的 spec.json 目标不在拒（「应当返回 2，实际 0」）
#       写回员目标不在的拒绝说明列出文件与出路（「应当返回 True，实际 False」）
#   HANDBACK_GUARD_BREAK=scribe-section-anywhere  missing_targets_listed() 报告里哪一节提到都算写进了「找不到的目标」
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#   HANDBACK_GUARD_BREAK=scribe-section-unread  missing_targets_listed() 不读「找不到的目标」一节，目标不在一律拒
#       写回员规格目标不在、报告「找不到的目标」一节写了它放行（「应当返回 0，实际 2」）
#   HANDBACK_GUARD_BREAK=leg-types-hardcoded  citation_checked_agents() 不读定义 frontmatter 的 leg: three-way，退回改前写死的四种（三种腿加没有定义的 three-way-verifier）
#       三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同（「应当返回 ['sample-leg', 'three-way-defense', 'three-way-forward']，实际 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier']」）
#       三方腿类型表:样本仓登记的 sample-leg 没点名报告拒（「应当返回 2，实际 0」）
#       三方腿类型表:样本仓没登记的攻方不核引文（「应当返回 0，实际 2」）
#   SINGLEFS_WORK_ROOT_BREAK=new-root-unknown（同目录 lib_work_root.py：工作根退回旧根）  ③ 认不出工作根下的报告文件，⑥ 认不出派发提示点名的工作根下的规格与报告
#       执行员工作根下的报告里绿行没数（~ 写法）（「应当返回 2，实际 0」）
#       写回员规格目标不在、报告没写找不到的目标拒（「应当返回 2，实际 0」）
#       写回员规格目标不在、只在报告别的节提到拒（「应当返回 2，实际 0」）
#       写回员目标不在的拒绝说明列出文件与出路（「应当返回 True，实际 False」）
#   SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown（同上：不认过渡期旧根）  ③ 认不出旧根下的报告文件
#       执行员过渡期旧根下的报告里绿行没数（「应当返回 2，实际 0」）
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$HOOK_DIR/../singlefs-ai-sop/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shlex, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone

HANDBACK_CHARACTER_LIMIT = int(os.environ.get("HANDBACK_CHARACTER_LIMIT", "4000"))
CITATION_CHECKED_AGENTS_BEFORE_REGISTRY = {"three-way-forward", "three-way-attack", "three-way-defense", "three-way-verifier"}   # 改前写死的那一份：只有弄坏开关 leg-types-hardcoded 下还在用
GREEN_COUNT_CHECKED_AGENTS = {"kb-scribe", "experiment-runner"}
LEG_REPORT_MENTION = re.compile(r"research/prompts/[^\s`'\"，。；：、（）()「」]+-output\.md")
try:   # 工作根与过渡期旧根的唯一定义：同目录的 lib_work_root.py（读不到时那两处下的报告认不出、不判，stderr 写一句）
    sys.dont_write_bytecode = True
    sys.path.insert(0, sys.argv[1]); import lib_work_root as work_roots
    SCRATCH_MENTION_PREFIX, work_roots_error = work_roots.mention_prefix_pattern(), None
except Exception as error:  # 文件不在、语法错、弄坏开关写错都算读不到
    work_roots, SCRATCH_MENTION_PREFIX, work_roots_error = None, "(?!)", error
try:   # 三方腿类型表的读法（登记处是定义 frontmatter 的 leg: three-way 一行；与派发闸、骨架、账表共用）：同目录的 lib_three_way_legs.py（读不到时 ② 不判，stderr 写一句）
    sys.path.insert(0, sys.argv[1]); import lib_three_way_legs as three_way_legs
    three_way_legs_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    three_way_legs, three_way_legs_error = None, error
REPORT_MENTION = re.compile(r"(?:research/prompts/|" + SCRATCH_MENTION_PREFIX + r")[^\s`'\"，。；：、（）()「」]+\.md")
GREEN_LINE = re.compile(r"^\s*(?:[-*>|]\s*)*✓")
# 编号不是数：阶段号、实验 / 决策 / 欠账 / 不变量号、阶段文件名、时刻
NOT_A_COUNT = re.compile(r"\d+\s*号|\b\d{2}-[\w.-]+|\b[EDC]\d+|\bI-[\d.]+|\d{1,2}:\d{2}")
ZERO_COUNT = re.compile(r"(?:查了|检查了?|扫了|判了|核了|共|跑了)\s*0\s*(?:项|个|条|份|处|种|行)")
BROKEN = os.environ.get("HANDBACK_GUARD_BREAK", "")
# ⑤ 交回前要跑命名检查的类型、要跑的四格与两条命令（共用约束 .claude/agent-common.md「门禁」一节「命名检查交回前跑」，<根> 填改动所在的树）
NAMING_CHECKED_AGENTS = {"implementation-writer", "experiment-runner", "tooling-writer"}
NAMING_CHECK_CELLS = frozenset({"vague-names", "test-file-names", "ordinal-names", "data-names"})
NAMING_SOURCE_DIRECTORIES = ("crates", "scripts")
NAMING_STAGE_TAIL = "/.claude/gate.d/code-source-discipline.sh"
NAMING_LINT_TAIL = "/.claude/singlefs-ai-sop/scripts/naming-lint.sh"
NAMING_STAGE_COMMAND = ("SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 vague-names、test-file-names、ordinal-names、data-names 四格; "
                        "范围=4 格; 为什么=交回前核自己改动里的命名' bash .claude/gate.d/code-source-discipline.sh "
                        "--check vague-names,test-file-names,ordinal-names,data-names {root}")
NAMING_LINT_COMMAND = "bash .claude/singlefs-ai-sop/scripts/naming-lint.sh {root}"
FILE_EDITING_TOOLS = {"Edit", "Write", "MultiEdit"}
# ⑥ 写回员：规格目标不在时报告里要有的那一节（.claude/agents/kb-scribe.md），写回员自己起草的规格放草稿目录的这两个名字
SCRIBE_TARGET_CHECKED_AGENTS = {"kb-scribe"}
MISSING_TARGETS_SECTION = "找不到的目标"
SCRIBE_DRAFTED_SPECIFICATIONS = ("spec.json", "spec.md")
MARKDOWN_HEADING = re.compile(r"^ {0,3}(#{1,6})\s+(.*)$")
PROMPT_LABEL_LINE_START = r"^[ \t>*_\-]*"
SPECIFICATION_SUFFIXES = (".json", ".md", ".txt")
# ① 拒绝说明改前的写法（没写报告文件名，辩方 landing-audit-r2 正推 8）：只有弄坏开关 limit-hint-unnamed 下还在用
LIMIT_HINT_BEFORE = "→ 怎么办：全文写进报告文件（派发提示给的报告路径；没给就写进草稿目录），交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。"


def is_project_agent(agent_type, project_root):
    if BROKEN == "judge-general-purpose" and agent_type:
        return True
    return bool(agent_type) and os.path.isfile(os.path.join(project_root, ".claude", "agents", f"{agent_type}.md"))


def citation_checked_agents(project_root):
    """② 要核引文的三方腿类型：项目根下定义 frontmatter 写了 leg: three-way 的（lib_three_way_legs.py 的 three_way_leg_types）。
    读不到那份库交空集、stderr 写一句；弄坏开关 leg-types-hardcoded 下退回改前写死的四种。"""
    if BROKEN == "leg-types-hardcoded":
        return frozenset(CITATION_CHECKED_AGENTS_BEFORE_REGISTRY)
    if three_way_legs is None:
        print(f"! handback-guard 读不到共用模块 lib_three_way_legs.py（{three_way_legs_error!r}），三方腿引文这一条没判（放行）", file=sys.stderr)
        return frozenset()
    return three_way_legs.three_way_leg_types(project_root)


def started_at(hook_input):
    """这个子 agent 会话记录第一条的时刻（epoch 秒）；找不到返回 None。"""
    transcript_path = hook_input.get("transcript_path") or ""
    agent_id = hook_input.get("agent_id") or ""
    if not transcript_path.endswith(".jsonl") or not agent_id:
        return None
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl")
    try:
        with open(own, encoding="utf-8", errors="replace") as handle:
            for line in handle:
                stamp = json.loads(line).get("timestamp")
                if stamp:
                    return datetime.fromisoformat(stamp.replace("Z", "+00:00")).timestamp()
    except (OSError, ValueError):
        return None
    return None


def resolved(path, project_root):
    path = work_roots.expand_written_path(path) if work_roots else path   # ~/…、$HOME/…、$SINGLEFS_WORK_ROOT/… 写法
    return path if os.path.isabs(path) else os.path.join(project_root, path)


def citation_problems(message, hook_input, project_root, cite_check):
    reports = sorted(set(LEG_REPORT_MENTION.findall(message)))
    if not reports:
        return ["交回里没点名报告文件（research/prompts/<轮>-…-output.md）"]
    since = started_at(hook_input)
    if since is None:
        print("! handback-guard：找不到这个子 agent 的会话记录，认不出开工时刻，引文这一条没判（放行）", file=sys.stderr)
        return []
    command = [sys.executable, cite_check, *[resolved(path, project_root) for path in reports], "--root", project_root, "--unchanged-since", str(since)]
    try:
        result = subprocess.run(command, capture_output=True, text=True, timeout=110)
    except (OSError, subprocess.TimeoutExpired) as error:
        print(f"! handback-guard：cite-check 跑不起来，引文这一条没判（放行）：{error}", file=sys.stderr)
        return []
    if result.returncode == 1:
        return [line.strip() for line in result.stdout.splitlines() if line.strip().startswith("✗")]
    if result.returncode != 0:
        print(f"! handback-guard：cite-check 退出码 {result.returncode}，引文这一条没判（放行）：{result.stdout[-300:]}{result.stderr[-300:]}", file=sys.stderr)
    return []


def green_count_problems(message, project_root):
    texts = [("交回正文", message)]
    for path in sorted(set(REPORT_MENTION.findall(message))):
        absolute = resolved(path, project_root)
        if os.path.isfile(absolute):
            try:
                texts.append((path, open(absolute, encoding="utf-8", errors="replace").read()))
            except OSError as error:
                print(f"! handback-guard：读不了 {absolute}（{error}），这一份的绿行没判", file=sys.stderr)
    problems = []
    for name, text in texts:
        for number, line in enumerate(text.splitlines(), 1):
            if not GREEN_LINE.match(line):
                continue
            if BROKEN != "no-green-count" and not re.search(r"\d", NOT_A_COUNT.sub("", line)):
                problems.append(f"{name}:{number} 绿行没有数：{line.strip()[:120]}")
            elif BROKEN != "zero-count-ok" and ZERO_COUNT.search(line):
                problems.append(f"{name}:{number} 绿行报的是 0 项：{line.strip()[:120]}")
    return problems


def background_problems(hook_input, agent_watch_path):
    """④ 这个子 agent 自己起的后台任务此刻还在跑的：交 own_background_tasks_still_running 的结果；主 agent、判不了的交空表。"""
    agent_id = hook_input.get("agent_id") or ""
    transcript_path = hook_input.get("transcript_path") or ""
    if BROKEN == "no-background" or agent_id in ("", "main"):
        return []
    own = os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl") if transcript_path.endswith(".jsonl") else ""
    if not os.path.isfile(own):
        print(f"! handback-guard：找不到这个子 agent 自己的会话记录（{own or '输入里没有主会话记录'}），后台任务这一条没判（放行）", file=sys.stderr)
        return []
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("agent_watch", agent_watch_path)
        agent_watch = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(agent_watch)
        return agent_watch.own_background_tasks_still_running(own)
    except Exception as error:   # 文件不在、导入失败、读不了会话记录：这一条没判，放行
        print(f"! handback-guard：后台任务这一条没判（放行）：{type(error).__name__}：{error}", file=sys.stderr)
        return []


def background_refusal(running):
    lines = []
    for task_id, output, tops in running:
        for pid, seconds, command, stop_order in tops:
            lines.append(f"  任务 {task_id}（输出 {output}）：进程 {pid} 已跑 {int(seconds // 60)} 分 {int(seconds % 60)} 秒：{command[:160]}；"
                         f"停的次序（从最底层往上）：{' '.join(map(str, stop_order[:24]))}{' …' if len(stop_order) > 24 else ''}")
    return (f"✗ 交回时你自己起的后台任务还在跑（{len(running)} 个任务）：\n" + "\n".join(lines[:20]) +
            "\n→ 怎么办：要它的结果就结束本轮、不再调工具，等它的完成通知到了、把结果写进报告再交回；不要了就按上面的次序逐个跑 "
            "`python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>`，停完 `ps -o pid,ppid,args --ppid <进程号>` 再列一遍确认一个不剩，"
            "在报告「没做什么」里写明停了哪条、为什么，再交回（.claude/agent-common.md「不做」一节停自己起的一条链那一条）。")


def own_transcript_records(hook_input):
    """这个子 agent 自己的会话记录逐条（dict）；输入里没有主会话记录或 agent_id、文件读不了交 None。"""
    transcript_path = hook_input.get("transcript_path") or ""
    agent_id = hook_input.get("agent_id") or ""
    if not transcript_path.endswith(".jsonl") or not agent_id:
        return None
    records = []
    try:
        with open(os.path.join(transcript_path[: -len(".jsonl")], "subagents", f"agent-{agent_id}.jsonl"), encoding="utf-8", errors="replace") as handle:
            for line in handle:
                try:
                    record = json.loads(line)
                except ValueError:
                    continue
                if isinstance(record, dict):
                    records.append(record)
    except OSError:
        return None
    return records


def tool_calls(records):
    """会话记录里的工具调用，按次序：[(工具名, 参数, 当前目录, 跑过没有)]。跑过：有工具结果，不是错误，或错误是「Exit code N」（跑了、退非 0）；
    被钩子拒的、超时的、没有结果的不算。弄坏开关 naming-refused-counts 下一律算跑过。"""
    results, calls = {}, []
    for record in records:
        message = record.get("message")
        content = message.get("content") if isinstance(message, dict) else None
        for block in content if isinstance(content, list) else []:
            if not isinstance(block, dict):
                continue
            if record.get("type") == "assistant" and block.get("type") == "tool_use":
                arguments = block.get("input") if isinstance(block.get("input"), dict) else {}
                calls.append((block.get("id"), block.get("name"), arguments, record.get("cwd")))
            elif record.get("type") == "user" and block.get("type") == "tool_result":
                results[block.get("tool_use_id")] = block

    def ran(result):
        if BROKEN == "naming-refused-counts":
            return True
        if result is None:
            return False
        if result.get("is_error") is not True:
            return True
        text = result.get("content")
        if isinstance(text, list):
            text = "".join(str(part.get("text") or "") for part in text if isinstance(part, dict))
        return isinstance(text, str) and text.lstrip().startswith("Exit code")
    return [(name, arguments, cwd, ran(results.get(call_id))) for call_id, name, arguments, cwd in calls]


def naming_source_tree(path, project_root):
    """改的文件是某棵树里 crates/ 或 scripts/ 下的 .rs 时交 (那棵树的根, 判不判)；不是交 None。
    主工作区（project_root）与带 .git 的副本（git worktree）判；没有 .git 的副本（rsync 拷的，命令交主 agent 跑）不判。
    弄坏开关 naming-any-file 下不看后缀，naming-copy-judged 下没有 .git 的副本也判。"""
    if not isinstance(path, str) or not path or (not path.endswith(".rs") and BROKEN != "naming-any-file"):
        return None
    absolute = os.path.normpath(path if os.path.isabs(path) else os.path.join(project_root, path))
    parts = absolute.split(os.sep)
    trees = [os.sep.join(parts[:index]) or os.sep for index in range(1, len(parts) - 1) if parts[index] in NAMING_SOURCE_DIRECTORIES]
    if not trees:
        return None
    for project in dict.fromkeys((os.path.normpath(project_root), os.path.realpath(project_root))):
        if project in trees:
            return project, True
    for tree in reversed(trees):
        if os.path.exists(os.path.join(tree, ".git")):
            return tree, True
    return trees[-1], BROKEN == "naming-copy-judged"


def naming_check_runs(command, directory, heavy):
    """一条 Bash 命令里起的命名检查：{(哪一条, 根的真实路径)}。哪一条是 "stage"（门禁 code-source-discipline.sh 的 --check 点全四格，或不带 --check 的整道）
    或 "lint"（上游 naming-lint.sh）。命令位置照 lib_shell_words 切，two-logs.sh 这类包装照 lib_heavy_tests 的 command_under_launcher 剥。
    弄坏开关 naming-root-unmatched 下认出的根一律接一段，当成别的树。"""
    found, pending = set(), [(command, directory, 0)]
    while pending:
        text, current, depth = pending.pop()
        try:
            commands = heavy.shell_words.commands_at_command_position(text, current).commands
        except Exception as error:   # 切不开的命令（引号不配对这类）：这一条不算跑过
            print(f"! handback-guard：切不开一条 Bash 命令，命名检查按没在这一条里跑算：{type(error).__name__}：{error}", file=sys.stderr)
            continue
        for item in commands:
            words = list(item.words)
            if not words:
                continue
            unwrapped = heavy.command_under_launcher(words, item.directory) if depth < 4 else None
            if unwrapped:
                inner_directory = heavy.shell_words.resolve_path(item.directory, unwrapped[1]) if unwrapped[1] else item.directory
                pending.append((shlex.join(unwrapped[0]), inner_directory, depth + 1))
                continue
            script = os.path.normpath(heavy.shell_words.resolve_path(item.directory, os.path.expanduser(words[0])) or words[0])
            if script.endswith(NAMING_STAGE_TAIL):
                kind, arguments = "stage", words[1:]
                cells = heavy.checked_cells(arguments)
                if "--list" in arguments or (cells is not None and not NAMING_CHECK_CELLS <= cells):
                    continue
                default = script[: -len(NAMING_STAGE_TAIL)] if os.path.isabs(script) else None
            elif script.endswith(NAMING_LINT_TAIL):
                kind, arguments, default, candidate = "lint", words[1:], None, item.directory
                while candidate and candidate != os.sep:   # 不给根时它从当前目录往上找（lib.sh 的 project_root）
                    if any(os.path.exists(os.path.join(candidate, mark)) for mark in (".git", ".singlefs-ai-sop-version")):
                        default = candidate
                        break
                    candidate = os.path.dirname(candidate)
            else:
                continue
            positional, position = [], 0
            while position < len(arguments):
                if arguments[position] == "--check":
                    position += 2
                    continue
                if not arguments[position].startswith("-"):
                    positional.append(arguments[position])
                position += 1
            root = default
            if positional:
                variables = dict(item.variables or {})
                if item.directory:
                    variables["PWD"] = item.directory
                written = item.directory if positional[0] in ("$(pwd)", "$(pwd -P)", "`pwd`") and item.directory else positional[0]
                expanded = heavy.shell_words.expand_known_variables(os.path.expanduser(written), variables)
                root = heavy.shell_words.resolve_path(item.directory, expanded) if expanded is not None else None
            if root:
                found.add((kind, os.path.realpath(root) + ("/别的树" if BROKEN == "naming-root-unmatched" else "")))
    return found


def naming_problems(hook_input, project_root, heavy):
    """⑤：改了 .rs、改完之后没在那棵树上把两条命名检查都跑过的 → [(树, 最后改的文件, [缺的那几条])]。读不了会话记录交 []（stderr 写一句）。"""
    records = own_transcript_records(hook_input)
    if records is None:
        print("! handback-guard：找不到这个子 agent 自己的会话记录，命名检查这一条没判（放行）", file=sys.stderr)
        return []
    last_edit, runs_after = {}, {}
    for name, arguments, cwd, ran in tool_calls(records):
        if name in FILE_EDITING_TOOLS and ran:
            tree = naming_source_tree(arguments.get("file_path"), project_root)
            if tree is not None and tree[1]:
                last_edit[tree[0]] = arguments.get("file_path")
                runs_after[tree[0]] = set()
        elif name == "Bash" and ran and runs_after:
            for kind, root in naming_check_runs(str(arguments.get("command") or ""), cwd, heavy):
                for tree, kinds in runs_after.items():
                    if os.path.realpath(tree) == root:
                        kinds.add(kind)
    return [(tree, last_edit[tree], [kind for kind in ("stage", "lint") if kind not in runs_after[tree]])
            for tree in last_edit if len(runs_after[tree]) < 2]


def naming_refusal(agent_type, problems):
    lines = [f"  {tree}：最后改的是 {edited}；改完之后没跑：{'、'.join('code-source-discipline.sh 那四格' if kind == 'stage' else 'naming-lint.sh' for kind in missing)}"
             for tree, edited, missing in problems]
    commands = [f"    {command.format(root=tree)}" for tree, _, _ in problems for command in (NAMING_STAGE_COMMAND, NAMING_LINT_COMMAND)]
    return (f"✗ {agent_type} 改了 crates/ 或 scripts/ 下的 .rs，会话记录里改完之后没跑过共用约束点名的两条命名检查（只认真跑过的命令，报告里写跑了不算）：\n" +
            "\n".join(lines[:10]) +
            "\n→ 怎么办：在改动所在的树上跑下面两条（<根> 已填好），命中在自己改动的文件里的修掉、在别的文件里的照写，两条的命令与末尾原样输出贴进报告，再交回"
            "（.claude/agent-common.md「门禁」一节「命名检查交回前跑」）：\n" + "\n".join(commands[:20]) +
            "\n  在 rsync 拷的副本（没有 .git）里改的这里不判，照那一条把两条命令写进报告「要主 agent 跑的命令」。")


def first_prompt(records):
    """派发提示：会话记录里第一条 user 记录的文字（字符串，或 text 块连起来）。"""
    for record in records:
        if record.get("type") != "user" or not isinstance(record.get("message"), dict):
            continue
        content = record["message"].get("content")
        if isinstance(content, str):
            return content
        if isinstance(content, list):
            return "\n".join(str(block.get("text") or "") for block in content if isinstance(block, dict) and block.get("type") == "text")
        return ""
    return ""


def prompt_label_paths(prompt, label):
    """提示里「<label>：<路径>」那几行的路径（去掉反引号与句末标点，取第一个词）。"""
    paths = []
    for match in re.finditer(PROMPT_LABEL_LINE_START + re.escape(label) + r"[*\s]*[：:][*\s]*(\S+)", prompt, re.M):
        value = match.group(1).strip("`'\"").rstrip("。，；：、.,;:")
        if value:
            paths.append(value)
    return paths


def scribe_spec_entries(prompt, project_root, kb_spec):
    """写回员这一次的规格条目：[(规格出处, 条目)]，只收旧串、新串都在的。出处：提示点名的工作根或旧根下的 .json / .md / .txt（「报告：」那一份除外）、
    「草稿目录：」下的 spec.json / spec.md、提示正文本身。"""
    reports = {os.path.realpath(resolved(path, project_root)) for path in prompt_label_paths(prompt, "报告")}
    candidates = []
    for mention in re.findall(SCRATCH_MENTION_PREFIX + r"[^\s`'\"，。；：、（）()「」]+", prompt):
        path = resolved(mention.rstrip(".:"), project_root)
        if path.endswith(SPECIFICATION_SUFFIXES) and os.path.isfile(path) and os.path.realpath(path) not in reports:
            candidates.append(path)
    for draft in prompt_label_paths(prompt, "草稿目录"):
        candidates += [path for path in (os.path.join(resolved(draft, project_root), name) for name in SCRIBE_DRAFTED_SPECIFICATIONS) if os.path.isfile(path)]
    loaded = []
    for path in dict.fromkeys(candidates):
        try:
            loaded += [(path, entry) for entry in kb_spec.load_spec(path)]
        except Exception as error:   # 读不了、写法意外：这一份不算规格
            print(f"! handback-guard：读不了规格 {path}（{type(error).__name__}：{error}），这一份没判", file=sys.stderr)
    loaded += [("派发提示正文", entry) for entry in kb_spec.parse_markdown_spec(prompt)]
    return [(source, entry) for source, entry in loaded
            if str(entry.get("file") or "").strip() and entry.get("old") is not None and entry.get("new") is not None]


def missing_targets_sections(text):
    """文字里标题含「找不到的目标」的那几节的正文（到同级或更高一级的下一个标题为止）。"""
    kept, level = [], None
    for line in text.splitlines():
        heading = MARKDOWN_HEADING.match(line)
        if heading:
            if level is not None and len(heading.group(1)) <= level:
                level = None
            if level is None and MISSING_TARGETS_SECTION in heading.group(2):
                level = len(heading.group(1))
                continue
        if level is not None:
            kept.append(line)
    return "\n".join(kept)


def missing_targets_listed(texts, forms):
    """forms（规格里的写法、从仓根起的路径、绝对路径）之一写在了哪一份文字的「找不到的目标」一节里。
    弄坏开关 scribe-section-anywhere 下整份文字哪里提到都算，scribe-section-unread 下一节都不读。"""
    for _, text in texts:
        section = text if BROKEN == "scribe-section-anywhere" else "" if BROKEN == "scribe-section-unread" else missing_targets_sections(text)
        for form in forms:
            if form and re.search(r"(?<![A-Za-z0-9_\-./])" + re.escape(form) + r"(?![A-Za-z0-9_\-/]|\.[A-Za-z0-9_])", section):
                return True
    return False


def scribe_target_problems(hook_input, message, project_root, scripts_directory):
    """⑥：规格里在主工作区不在、又没写进「找不到的目标」一节的目标 → [「路径（规格 出处）」]。认不出规格、读不了会话记录、导入不了 kb-spec-check.py 交 []。"""
    records = own_transcript_records(hook_input)
    if records is None:
        print("! handback-guard：找不到这个写回员自己的会话记录，规格目标这一条没判（放行）", file=sys.stderr)
        return []
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("kb_spec_check", os.path.join(scripts_directory, "kb", "kb-spec-check.py"))
        kb_spec = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(kb_spec)
    except Exception as error:   # 文件不在、导入失败
        print(f"! handback-guard：导入不了 kb-spec-check.py，规格目标这一条没判（放行）：{type(error).__name__}：{error}", file=sys.stderr)
        return []
    prompt = first_prompt(records)
    entries = scribe_spec_entries(prompt, project_root, kb_spec)
    if not entries:
        return []
    texts = [("交回正文", message)]
    for path in dict.fromkeys(REPORT_MENTION.findall(message) + prompt_label_paths(prompt, "报告")):
        absolute = resolved(path, project_root)
        if os.path.isfile(absolute):
            try:
                texts.append((path, open(absolute, encoding="utf-8", errors="replace").read()))
            except OSError as error:
                print(f"! handback-guard：读不了报告 {absolute}（{error}），这一份的「找不到的目标」没看", file=sys.stderr)
    problems, root = [], os.path.normpath(project_root)
    for source, entry in entries:
        written = str(entry["file"]).strip()
        absolute = os.path.normpath(written if os.path.isabs(written) else os.path.join(root, written))
        if os.path.exists(absolute):
            continue
        relative = os.path.relpath(absolute, root) if absolute.startswith(root + os.sep) else absolute
        if not missing_targets_listed(texts, (written, relative, absolute)):
            problems.append(f"{relative}（规格 {source}）")
    return list(dict.fromkeys(problems))


def decide(hook_input, project_root, cite_check):
    """返回 (0, None) 放行；(2, 说明) 拒绝。"""
    running = background_problems(hook_input, os.path.join(os.path.dirname(cite_check), "agent-watch.py"))
    if running:
        return 2, background_refusal(running)
    agent_type = hook_input.get("agent_type")
    if not is_project_agent(agent_type, project_root):
        return 0, None
    message = str((hook_input.get("tool_input") or {}).get("message") or "")
    if BROKEN != "no-limit" and len(message) > HANDBACK_CHARACTER_LIMIT:
        hint = (LIMIT_HINT_BEFORE if BROKEN == "limit-hint-unnamed" else
                "→ 怎么办：全文写进报告文件：三方腿写派发提示给的 research/prompts/<轮>-*-output.md；别的类型写派发提示给的报告路径，"
                "没给报告路径就写进草稿目录的 result.md（文件名不带 report）；交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。")
        return 2, f"✗ {agent_type} 的交回正文 {len(message)} 字，超过 {HANDBACK_CHARACTER_LIMIT} 字。\n" + hint
    if (BROKEN == "cite-any-agent" or agent_type in citation_checked_agents(project_root)) and BROKEN != "no-cite":
        problems = citation_problems(message, hook_input, project_root, cite_check)
        if problems:
            return 2, ("✗ 报告里的引文对不上（scripts/agents/cite-check.py）：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：按每处的提示改报告里的行号或引文——行号去被引文件里现查（grep -nF），不从背景材料里数；改完再跑一次 "
                       "python3 scripts/agents/cite-check.py <报告> 看它转绿，再交回。")
    if agent_type in GREEN_COUNT_CHECKED_AGENTS:
        problems = green_count_problems(message, project_root)
        if problems:
            return 2, ("✗ 绿行没报查了多少项，或报的是 0 项：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：贴绿行时连它报的「查了多少项」一起贴；那一道报 0 项或根本没报数的，按没判写（「本次未判：…」），不写成 ✓（.claude/agent-common.md「门禁」一节）。")
    if agent_type in NAMING_CHECKED_AGENTS and BROKEN != "no-naming-check":
        heavy = heavy_tests_library()
        problems = naming_problems(hook_input, project_root, heavy) if heavy is not None else []
        if problems:
            return 2, naming_refusal(agent_type, problems)
    if agent_type in SCRIBE_TARGET_CHECKED_AGENTS and BROKEN != "no-scribe-targets":
        problems = scribe_target_problems(hook_input, message, project_root, os.path.dirname(os.path.dirname(cite_check)))
        if problems:
            return 2, ("✗ 写回员规格点名的目标文件在主工作区不在，交回与报告里「找不到的目标」一节也没有它：\n" + "\n".join(f"  {line}" for line in problems[:20]) +
                       "\n→ 怎么办：照 .claude/agents/kb-scribe.md 那一句办：整份不写，报告里加一节「## 找不到的目标」，逐条写规格里的文件（从仓根起的路径）与旧串，"
                       "以及 `git log --diff-filter=DR --name-only -- <文件名>` 和 `grep -rnF <旧串的一段>` 查到的新位置（查不到写查不到），再交回；"
                       "目标其实在的，核规格里的路径是不是写错了（从仓根起写）。")
    return 0, None


HEAVY_TESTS_LIBRARY = []

def heavy_tests_library():
    """同目录的 lib_heavy_tests.py（它带着 lib_shell_words）；导入不了交 None，stderr 写一句（命名检查这一条不判）。"""
    if not HEAVY_TESTS_LIBRARY:
        try:
            import importlib.util
            spec = importlib.util.spec_from_file_location("lib_heavy_tests", os.path.join(sys.argv[1], "lib_heavy_tests.py"))
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            HEAVY_TESTS_LIBRARY.append(module)
        except Exception as error:   # 文件不在、语法错、它导入 lib_shell_words 失败
            print(f"! handback-guard：导入不了 lib_heavy_tests.py，命名检查这一条没判（放行）：{type(error).__name__}：{error}", file=sys.stderr)
            HEAVY_TESTS_LIBRARY.append(None)
    return HEAVY_TESTS_LIBRARY[0]


def selftest(hook_dir):
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与过渡期旧根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    if three_way_legs is None:
        print(f"  ✗ 自检读不到三方腿类型表的读法 {os.path.join(hook_dir, 'lib_three_way_legs.py')}：{three_way_legs_error!r}")
        print("    → 怎么办：恢复那份库（三方腿类型表只登记在定义 frontmatter 的 leg: three-way，读法只在那份库里），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="handback-guard-selftest-")
    sleeper = None   # ④ 的样本：开着一个后台任务输出文件在睡的进程，自检自己起、自己停
    scratch_reports = []   # ③ 放在工作根与旧根下的报告目录，自检的 TMPDIR 之外
    try:
        cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "agents", "cite-check.py")
        for agent in ("three-way-attack", "three-way-verifier", "kb-scribe", "experiment-runner", "sweep", "implementation-writer", "tooling-writer"):
            os.makedirs(os.path.join(work, ".claude", "agents"), exist_ok=True)   # 三方腿类型表登记在 frontmatter：样本仓里攻方与核查员登记成三方腿
            open(os.path.join(work, ".claude", "agents", f"{agent}.md"), "w").write(
                f"---\nname: {agent}\nleg: three-way\n---\n# {agent}\n" if agent.startswith("three-way-") else f"# {agent}\n")
        os.makedirs(os.path.join(work, "kb"))
        open(os.path.join(work, "kb", "rule.md"), "w", encoding="utf-8").write("# 规则\n第一条：先查再写。\n")
        os.makedirs(os.path.join(work, "research", "prompts"))
        open(os.path.join(work, "research", "prompts", "r1-opus-output.md"), "w", encoding="utf-8").write("「第一条：先查再写」（`kb/rule.md:2`）\n")
        open(os.path.join(work, "research", "prompts", "r2-opus-output.md"), "w", encoding="utf-8").write("「第一条：先查再写」（`kb/rule.md:1`）\n")
        session = os.path.join(work, "session")
        os.makedirs(os.path.join(session, "subagents"))
        # 开工时刻取样本文件都建好之后：被引文件不算「开工之后改过」
        started = datetime.fromtimestamp(os.path.getmtime(os.path.join(work, "research", "prompts", "r2-opus-output.md")) + 5, timezone.utc)
        open(os.path.join(session, "subagents", "agent-a1.jsonl"), "w").write(json.dumps({"timestamp": started.isoformat()}) + "\n")
        # ④ a2 起的后台任务 blive 的输出文件有一个在睡的进程开着（交回时还在跑）；a3 起的 bdone 输出文件在、没有进程开着（跑完了）
        live_output, done_output = os.path.join(work, "tasks", "blive.output"), os.path.join(work, "tasks", "bdone.output")
        os.makedirs(os.path.dirname(live_output))
        open(done_output, "w").close()

        def background_started_record(task_id, output):
            return json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [
                {"type": "tool_result", "tool_use_id": f"t-{task_id}",
                 "content": f"Command running in background with ID: {task_id}. Output is being written to: {output}. You will be notified when it completes."}]}})
        for agent_id, task_id, output in (("a2", "blive", live_output), ("a3", "bdone", done_output)):
            open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w").write(
                json.dumps({"timestamp": started.isoformat()}) + "\n" + background_started_record(task_id, output) + "\n")
        with open(live_output, "w") as output_handle:
            sleeper = subprocess.Popen(["sleep", "120"], stdout=output_handle, stderr=subprocess.STDOUT)
        scribe_report = os.path.join(work, "research", "prompts", "scribe-result.md")
        open(scribe_report, "w", encoding="utf-8").write("✓ 文档铁律检查通过（检查 486，跳过 0）\n✓ 通过\n")
        # ③ 的报告放工作根（不看弄坏开关）与过渡期旧根下各一份，交回里工作根那一份写成 ~ 起头；自己建、finally 里删
        home = os.path.expanduser("~")
        for scratch_root in (work_roots.configured_work_root(), work_roots.legacy_root()):
            os.makedirs(scratch_root, exist_ok=True)
            scratch_reports.append(tempfile.mkdtemp(prefix="handback-guard-selftest-", dir=scratch_root))
            open(os.path.join(scratch_reports[-1], "result.md"), "w", encoding="utf-8").write("✓ 通过\n")
        new_root_report = os.path.join(scratch_reports[0], "result.md")
        new_root_report = "~" + new_root_report[len(home):] if new_root_report.startswith(home + "/") else new_root_report

        def case(label, agent_type, message, want, agent_id="a1"):
            hook_input = {"tool_name": "SubagentHandback", "tool_input": {"message": message}, "agent_type": agent_type,
                          "agent_id": agent_id, "transcript_path": session + ".jsonl"}
            return (label, want, decide(hook_input, work, cite_check)[0])
        cases = [
            case("超过上限的项目子 agent", "sweep", "字" * (HANDBACK_CHARACTER_LIMIT + 1), 2),
            case("正好在上限的项目子 agent", "sweep", "字" * HANDBACK_CHARACTER_LIMIT, 0),
            case("不是项目定义的 agent 不判", "general-purpose", "字" * (HANDBACK_CHARACTER_LIMIT + 1), 0),
            case("主 agent 不判", None, "字" * (HANDBACK_CHARACTER_LIMIT + 1), 0),
            case("攻方腿引文对上", "three-way-attack", "报告 research/prompts/r1-opus-output.md sha256 x", 0),
            case("攻方腿引文行号错", "three-way-attack", "报告 research/prompts/r2-opus-output.md sha256 x", 2),
            case("核查员没点名报告", "three-way-verifier", "核完了", 2),
            case("回扫员不核引文", "sweep", "报告 research/prompts/r2-opus-output.md", 0),
            case("书记员绿行带数", "kb-scribe", "✓ doc-decisions 检查通过（查了 12 项）", 0),
            case("书记员绿行没数", "kb-scribe", "✓ doc-registries 转绿", 2),
            case("书记员绿行 0 项", "kb-scribe", "✓ doc-registries 通过（查了 0 项）", 2),
            case("执行员报告文件里的绿行没数", "experiment-runner", f"报告 {os.path.relpath(scribe_report, work)}", 2),
            case("执行员工作根下的报告里绿行没数（~ 写法）", "experiment-runner", f"报告 {new_root_report}", 2),
            case("执行员过渡期旧根下的报告里绿行没数", "experiment-runner", f"报告 {os.path.join(scratch_reports[1], 'result.md')}", 2),
            case("回扫员的绿行不判", "sweep", "✓ 通过", 0),
            case("交回时自己起的后台任务还在跑（项目子 agent）", "sweep", "报告 /tmp/x", 2, "a2"),
            case("交回时自己起的后台任务还在跑（通用 agent 也判）", "general-purpose", "交回", 2, "a2"),
            case("起过的后台任务都跑完了", "sweep", "交回", 0, "a3"),
            case("主 agent 不判后台任务", None, "交回", 0, None),
        ]
        # 三方腿类型表（拆分提案第九十节 ⑯）：② 读到的类型集合与登记处（定义 frontmatter 的 leg: three-way 一行）逐个相同。样本仓里的登记故意与改前写死的四种不同：
        # 攻方没登记、辩方的值带行尾注释、另登记一种不以 three-way 起头的 sample-leg、核查员写 leg: other、材料员没登记、正文里写的不算；本仓按 frontmatter 现扫，至少要有一种。
        # 再经 decide 判一遍：样本仓里 sample-leg 没点名报告拒、没登记的攻方不核引文（开关 leg-types-hardcoded 下样本仓那三格红）
        legs_root = os.path.join(work, "legs-sample")
        os.makedirs(os.path.join(legs_root, ".claude", "agents"))
        for leg_name, leg_line in (("three-way-forward", "leg: three-way\n"), ("three-way-attack", ""), ("three-way-defense", "leg: three-way  # 注释\n"),
                                   ("three-way-verifier", "leg: other\n"), ("three-way-materials", ""), ("sample-leg", "leg: three-way\n")):
            open(os.path.join(legs_root, ".claude", "agents", f"{leg_name}.md"), "w", encoding="utf-8").write(
                f"---\nname: {leg_name}\n{leg_line}---\n正文里写的\nleg: three-way\n不算\n")
        frontmatter_scanned_legs = three_way_legs.three_way_leg_types_scanned   # 「frontmatter 里写着什么」：同一份库里不拆键值的另一种读法
        repository_root = os.path.dirname(os.path.dirname(hook_dir))
        repository_legs = frontmatter_scanned_legs(repository_root)

        def legs_case(label, agent_type, want):
            return (label, want, decide({"tool_name": "SubagentHandback", "tool_input": {"message": "核完了"}, "agent_type": agent_type}, legs_root, cite_check)[0])
        cases += [
            ("三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同", frontmatter_scanned_legs(legs_root), sorted(citation_checked_agents(legs_root))),
            ("三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种", (repository_legs, True), (sorted(citation_checked_agents(repository_root)), bool(repository_legs))),
            legs_case("三方腿类型表:样本仓登记的 sample-leg 没点名报告拒", "sample-leg", 2),
            legs_case("三方腿类型表:样本仓没登记的攻方不核引文", "three-way-attack", 0),
        ]

        # ⑤ 命名检查：样本树就是 work（项目根）；git worktree 副本带 .git 文件，rsync 副本不带
        def transcript(agent_id, records):
            with open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w", encoding="utf-8") as handle:
                handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": "# 派发"}}, ensure_ascii=False) + "\n")
                for number, (tool, arguments, outcome, cwd) in enumerate(records):
                    handle.write(json.dumps({"type": "assistant", "timestamp": started.isoformat(), "cwd": cwd, "message": {"role": "assistant", "content": [
                        {"type": "tool_use", "id": f"call-{number}", "name": tool, "input": arguments}]}}, ensure_ascii=False) + "\n")
                    result = {"type": "tool_result", "tool_use_id": f"call-{number}", "content": outcome[1]}
                    if outcome[0]:
                        result["is_error"] = True
                    handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": [result]}}, ensure_ascii=False) + "\n")
        worktree, copy = os.path.join(work, "worktree"), os.path.join(work, "rsync-copy")
        os.makedirs(worktree)
        open(os.path.join(worktree, ".git"), "w").write("gitdir: /nonexistent\n")
        done, exited_red, refused = (False, "ok"), (True, "Exit code 1\n  ✗ 命中 1 处"), (True, "PreToolUse:Bash hook error: ✗ 拒")
        def edited(path):
            return ("Edit", {"file_path": path, "old_string": "a", "new_string": "b"}, done, work)
        def naming_stage(root, cells="vague-names,test-file-names,ordinal-names,data-names"):
            return ("Bash", {"command": NAMING_STAGE_COMMAND.replace("vague-names,test-file-names,ordinal-names,data-names", cells).format(root=root)}, done, work)
        def naming_lint(root, outcome=exited_red):
            return ("Bash", {"command": NAMING_LINT_COMMAND.format(root=root)}, outcome, work)
        core_source = os.path.join(work, "crates", "core", "src", "lib.rs")
        transcript("naming-unrun", [edited(core_source)])
        transcript("naming-run", [edited(core_source), naming_stage(work), naming_lint(".")])
        transcript("naming-markdown", [edited(os.path.join(work, "crates", "core", "README.md"))])
        transcript("naming-rsync-copy", [edited(os.path.join(copy, "crates", "core", "src", "lib.rs"))])
        transcript("naming-scripts", [edited(os.path.join(work, "scripts", "experiment-apparatus", "e1_probe.rs"))])
        transcript("naming-before-edit", [edited(core_source), naming_stage(work), naming_lint(work), edited(core_source)])
        transcript("naming-one-command", [edited(core_source), naming_stage(work)])
        transcript("naming-refused", [edited(core_source), naming_stage(work), naming_lint(work, refused)])
        transcript("naming-three-cells", [edited(core_source), naming_stage(work, "vague-names,test-file-names,ordinal-names"), naming_lint(work)])
        transcript("naming-worktree", [edited(os.path.join(worktree, "crates", "core", "src", "lib.rs")), naming_stage(worktree), naming_lint(worktree)])
        transcript("naming-two-logs", [edited(core_source), ("Bash", {"command": "bash scripts/hosts/two-logs.sh naming -- " + NAMING_STAGE_COMMAND.format(root=work)}, done, work),
                                       ("Bash", {"command": f"cd {work} && bash scripts/hosts/two-logs.sh lint -- " + NAMING_LINT_COMMAND.format(root="$PWD")}, done, "/")])
        transcript("naming-pwd", [edited(core_source), ("Bash", {"command": f"cd {work} && " + NAMING_STAGE_COMMAND.format(root='"$(pwd)"') + " && "
                                                                 + NAMING_LINT_COMMAND.format(root='"$(pwd)"') + " | tail -n 20"}, done, "/")])
        naming_refusal_text = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "命名检查两条都跑了"}, "agent_type": "implementation-writer",
                                      "agent_id": "naming-unrun", "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases += [
            case("改了 .rs 没跑命名检查拒", "implementation-writer", "命名检查两条都跑了、都绿", 2, "naming-unrun"),
            case("改了 .rs 之后两条命名检查都跑了放行", "implementation-writer", "交回", 0, "naming-run"),
            case("只改 .md 放行", "implementation-writer", "交回", 0, "naming-markdown"),
            case("rsync 副本里改的 .rs 交主 agent 放行", "implementation-writer", "交回", 0, "naming-rsync-copy"),
            case("执行员改了 scripts/ 下 .rs 没跑命名检查拒", "experiment-runner", "交回", 2, "naming-scripts"),
            case("命名检查跑在最后一次改 .rs 之前拒", "tooling-writer", "交回", 2, "naming-before-edit"),
            case("只跑了一条命名检查拒", "implementation-writer", "交回", 2, "naming-one-command"),
            case("命名检查被钩子拒了不算跑过", "implementation-writer", "交回", 2, "naming-refused"),
            case("四格点名少一格不算跑过", "implementation-writer", "交回", 2, "naming-three-cells"),
            case("git worktree 副本里改的、命名检查根写成副本根放行", "implementation-writer", "交回", 0, "naming-worktree"),
            case("经 two-logs.sh 包着跑命名检查放行", "implementation-writer", "交回", 0, "naming-two-logs"),
            case("cd 之后根写成 $(pwd)、接管道跑命名检查放行", "implementation-writer", "交回", 0, "naming-pwd"),
            case("回扫员改 .rs 不判命名检查", "sweep", "交回", 0, "naming-unrun"),
            ("改了 .rs 没跑的拒绝说明给出两条命令（根填改动所在的树）", True,
             all(part in naming_refusal_text for part in (f"ordinal-names,data-names {work}", f"naming-lint.sh {work}", "→ 怎么办"))),
        ]

        # ⑥ 写回员规格的目标：规格与报告放工作根下（派发提示认工作根下的规格文件），目标一份在 work 里、一份不在
        scribe_root = tempfile.mkdtemp(prefix="handback-guard-selftest-", dir=work_roots.configured_work_root())
        scratch_reports.append(scribe_root)
        os.makedirs(os.path.join(work, ".claude", "kb"), exist_ok=True)
        open(os.path.join(work, ".claude", "kb", "present.md"), "w", encoding="utf-8").write("旧的一行\n")
        def scribe_file(name, text):
            path = os.path.join(scribe_root, name)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            open(path, "w", encoding="utf-8").write(text)
            return path
        present_spec = scribe_file("present-spec.json", json.dumps([{"file": ".claude/kb/present.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"}], ensure_ascii=False))
        missing_spec = scribe_file("missing-spec.json", json.dumps([{"file": ".claude/kb/present.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"},
                                                                    {"file": ".claude/kb/missing.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"}], ensure_ascii=False))
        drafted = scribe_file("drafted/spec.json", json.dumps([{"file": ".claude/kb/missing.md", "old": "旧的一行", "new": "新的一行", "basis": "判决"}], ensure_ascii=False))
        plain_report = scribe_file("plain-result.md", "# 写回员报告\n\n## 改过的文件\n\n0 份。\n")
        listed_report = scribe_file("listed-result.md", "# 写回员报告\n\n## 找不到的目标\n\n- `.claude/kb/missing.md`，旧串「旧的一行」；git log 与 grep 都查不到\n\n## 改过的文件\n\n0 份。\n")
        elsewhere_report = scribe_file("elsewhere-result.md", "# 写回员报告\n\n## 找不到的目标\n\n0 条。\n\n## 改过的文件\n\n没改 `.claude/kb/missing.md`。\n")
        inline_spec = "文件：.claude/kb/missing.md\n旧串：\n```\n旧的一行\n```\n新串：\n```\n新的一行\n```\n依据：判决\n"
        for agent_id, prompt in (("scribe-present", f"规格：{present_spec}\n报告：{plain_report}\n"),
                                 ("scribe-missing", f"规格：{missing_spec}\n报告：{plain_report}\n"),
                                 ("scribe-inline", f"照下面的规格写回。\n\n{inline_spec}\n报告：{plain_report}\n"),
                                 ("scribe-drafted", f"判决 research/prompts/x-main-verification.md\n草稿目录：{os.path.dirname(drafted)}\n报告：{plain_report}\n"),
                                 ("scribe-verdict-only", f"判决 research/prompts/x-main-verification.md\n草稿目录：{os.path.join(scribe_root, 'empty-draft')}\n")):
            with open(os.path.join(session, "subagents", f"agent-{agent_id}.jsonl"), "w", encoding="utf-8") as handle:
                handle.write(json.dumps({"type": "user", "timestamp": started.isoformat(), "message": {"role": "user", "content": prompt}}, ensure_ascii=False) + "\n")
        scribe_refusal_text = decide({"tool_name": "SubagentHandback", "tool_input": {"message": f"报告 {plain_report}"}, "agent_type": "kb-scribe",
                                      "agent_id": "scribe-missing", "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases += [
            case("写回员规格目标都在放行", "kb-scribe", f"报告 {plain_report}", 0, "scribe-present"),
            case("写回员规格目标不在、报告没写找不到的目标拒", "kb-scribe", f"报告 {plain_report}", 2, "scribe-missing"),
            case("写回员规格目标不在、报告「找不到的目标」一节写了它放行", "kb-scribe", f"报告 {listed_report}", 0, "scribe-missing"),
            case("写回员规格目标不在、只在报告别的节提到拒", "kb-scribe", f"报告 {elsewhere_report}", 2, "scribe-missing"),
            case("写回员规格写在提示正文里、目标不在拒", "kb-scribe", f"报告 {plain_report}", 2, "scribe-inline"),
            case("写回员自己起草的 spec.json 目标不在拒", "kb-scribe", f"报告 {plain_report}", 2, "scribe-drafted"),
            case("写回员没有规格不判", "kb-scribe", "停在起草之前，要主 agent 判", 0, "scribe-verdict-only"),
            ("写回员目标不在的拒绝说明列出文件与出路", True,
             all(part in scribe_refusal_text for part in (".claude/kb/missing.md", "找不到的目标", "git log --diff-filter=DR", "→ 怎么办"))),
        ]
        live_refusal = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "交回"}, "agent_type": "sweep", "agent_id": "a2",
                               "transcript_path": session + ".jsonl"}, work, cite_check)[1] or ""
        cases.append(("后台还在跑的拒绝说明列出进程号、命令与两条出路（等完成通知、按进程号停）", True,
                      all(part in live_refusal for part in (f"进程 {sleeper.pid}", "sleep 120", "blive", "完成通知", "proc.py stop"))))
        sleeper.kill()
        sleeper.wait()
        cases.append(case("那个后台任务结束之后再交回放行", "sweep", "交回", 0, "a2"))
        stdin_input = json.dumps({"tool_name": "SubagentHandback", "tool_input": {"message": "字" * (HANDBACK_CHARACTER_LIMIT + 1)}, "agent_type": "sweep"})
        entry = subprocess.run(["bash", os.path.join(hook_dir, "handback-guard.sh")], input=stdin_input, capture_output=True, text=True,
                               env={**os.environ, "CLAUDE_PROJECT_DIR": work}, timeout=60)
        cases.append(("走真实 stdin 入口：超长交回", 2, entry.returncode))
        limit_refusal = decide({"tool_name": "SubagentHandback", "tool_input": {"message": "字" * (HANDBACK_CHARACTER_LIMIT + 1)}, "agent_type": "sweep"},
                               work, cite_check)[1] or ""
        cases.append(("超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）", True,
                      all(part in limit_refusal for part in ("草稿目录的 result.md", "research/prompts/<轮>-*-output.md", "→ 怎么办"))))
        failures = [item for item in cases if item[1] != item[2]]
        for label, want, got in failures:
            print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
        if failures:
            print("    → 看 decide()、citation_problems()、green_count_problems()、naming_problems()、naming_check_runs()、scribe_target_problems() 的判法；"
                  "HANDBACK_GUARD_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ handback-guard 自检通过（查了 {len(cases)} 种）：子 agent 交回时自己起的后台任务还在跑拒（通用 agent 也判、主 agent 不判、说明里列出进程号与两条出路），"
              "跑完了或停掉之后放行；项目子 agent 超长交回拒（拒绝说明写明 result.md 与三方腿的 -output.md）、正好在上限放行、非项目 agent 与主 agent 不判；"
              "三方腿（类型读定义 frontmatter 的 leg: three-way，样本仓与本仓读到的都与 frontmatter 逐个相同）引文对不上或没点名报告拒、对上放行；书记员与执行员绿行没数或报 0 项拒（交回正文与点名的报告文件都查，报告在 research/prompts/、工作根（含 ~ 写法）与过渡期旧根下都认）；"
              "实现员、执行员、工具员改了 .rs 没在改完之后跑两条命名检查拒（跑在改之前、只跑一条、被钩子拒、少一格都拒，拒绝说明给出两条命令），跑了、只改 .md、rsync 副本、worktree 副本根对上、two-logs.sh 包着、根写成 $(pwd) 放行；"
              "写回员规格目标不在而「找不到的目标」一节没写拒（规格文件、提示正文、草稿目录里起草的都认），目标都在、那一节写了、没有规格放行；走真实 stdin 入口拒得出")
        return 0
    finally:
        if sleeper is not None and sleeper.poll() is None:
            sleeper.kill()
            sleeper.wait()
        shutil.rmtree(work, ignore_errors=True)
        for directory in scratch_reports:
            shutil.rmtree(directory, ignore_errors=True)


def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError as error:
        print(f"! handback-guard 读不懂 hook 的 JSON，这一次放行：{error}", file=sys.stderr)
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    cite_check = os.path.join(os.path.dirname(os.path.dirname(hook_dir)), "scripts", "agents", "cite-check.py")
    if work_roots is None:
        print(f"! handback-guard 读不到 {os.path.join(hook_dir, 'lib_work_root.py')}（{work_roots_error!r}）：工作根与旧根下的报告这一次没判", file=sys.stderr)
    code, message = decide(hook_input, project_root, cite_check)
    if message:
        print(message, file=sys.stderr)
    return code

sys.exit(main())
PY
