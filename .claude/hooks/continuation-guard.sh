#!/usr/bin/env bash
# admission: always Claude Code 每一次触发都要现判这一次调用，上一次的结论不替这一次作保
# run-condition: command python3
# PreToolUse hook（SendMessage）：给一个已经交回过、或被中断过（撞限额、被停、报错）的子 agent 发消息之前拦一道。
#
# 为什么：交回之后不续做，新活一律新派。用户 2026-09-24 问「启动subagent的时候 为什么会复用 id呢 很奇怪 这个能禁止复用吗」，
# 选的是「交回之后不许续做」；还没交回的（在干活、或结束本轮去等自己后台任务的）照旧放行，整点询问与纠正要送得到。
# 中断过的不续：2026-09-19 实测，E155 第二次跑的执行员撞限额之后被续做，上下文涨到 95 万，380 次调用平均每次 62.6 万、
# 累计读缓存 2.36 亿、整份重写两次共 123.5 万，真正的输出 2.9 万。用户定：「以后这样被中断的 subagent 都应该新开，
# 但是新开的时候要保证上下文和成果物的接续；如果接续会导致主 agent 的上下文膨胀，那么干脆就重开」。
#
# 判法：下面两条任一成立就拒绝。先看主会话记录（hook 输入的 transcript_path）里这个收件人最近一次任务通知与交回消息，
# 再看它在本会话 subagents 目录里的会话记录（`<会话目录>/subagents/agent-<id>.jsonl`，按 agent id 找）；子 agent 的会话记录不在（被挪走、删掉、
# 还没落盘）时，主会话里的 failed / killed 与交回消息照判，都没有才放行（别的会话、团队成员、名字而不是 id 的收件人照旧放行）。
# 子 agent 的会话记录不在、或记录里取不到上下文与续做次数时，拒绝说明写「取不到」，不写 0。
#   ① 交回过：主会话记录里已经有它的交回消息（`<agent-message from="<id>">` 后跟 `[Subagent hand-back]`，交回结果落进它自己的会话记录之前先到的那一侧），
#      或者它自己的会话记录里有一次送达的交回：assistant 调过 SubagentHandback，并且后面有 tool_use_id 与它相同的 tool_result，
#      不是错误（is_error 不为真，内容也不是 {"success":false,…} 那种没送出；被交回闸拒之后再没送达的不算，先被拒、后来送达的算）。
#      这一认法只在 scripts/agents/subagent_transcript.py 的 handback_delivered 一处（看门狗 agent-watch.py 与结局账 agent-cost.py 取的是同一份）；
#      读不到那份库时这一支不判、放行，stderr 写「读不到共用模块」（主会话记录里的交回消息与 ② 照判）。
#   ② 中断过：本会话记录里它最近一次任务通知是 failed 或 killed。中断过的不留「只差收尾」的口子（用户 2026-09-19：
#      「按照你现在的方法实际上也是在让上下文白白膨胀，没有意义」）。
# 两条的出路都是新派一个同类 agent：派发提示指到它的报告与产物；要它会话里的进度，用 scripts/agents/agent-handover.py
# 从会话记录里抽出来给新 agent 读。
# 判不了的照「放行但必记检出」：输入不是合法的 JSON 对象（解不开、不是 JSON、顶层不是对象）照常退 0，stderr 写一行可定位的诊断（钩子名、哪一处、输入的前 200 字节），
# 检出记录（AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path）追加一行，findings 以「续做闸没判全」起头、outcome 只记，看门狗据此报给主 agent；
# 输入里按字面找不到会话号的，session_id 写「读不到」（report_unparsable_input）。
# 读得懂而判不全的同样照「放行但必记检出」：主会话记录不在（transcript_path 没给或不是文件）、读不到 scripts/agents/subagent_transcript.py、判的时候抛异常，
#   放行时 stderr 写一句、检出记录追加一行（findings 以「续做闸没判全」起头、outcome 只记，record_unjudged）；拒的那一次照拒，不另记。
#
# 上下文多大不拦（用户 2026-09-19：「不应该限制 280K，这个不可理。检测的看门狗不应该有终止的权利，subagent 也不应该有超过了
# token 终止的逻辑，应该告诉主 agent 看看任务是不是异常，如果不是就继续跑」）：看门狗报「上下文过大」，主 agent 看任务有没有异常，
# 没有就接着跑、接着续。这里原先还按上下文 40 万拒绝续做，那天主 agent 据此停掉了一个 28 万、任务并无异常的 agent。
#
#   continuation-guard.sh             # 从 stdin 读 hook 的 JSON
#   continuation-guard.sh --selftest  # 在临时目录里走一遍放行与拒绝
# 弄坏开关只在自证里认（门禁 code-tooling 的 break-switch-outside-selftest 格）：平常跑（不是 --selftest，也不是自证起的子进程）设了 CONTINUATION_GUARD_BREAK 就退 2、
# stderr 点名变量只在自证里认；自证经 stdin 起的那几次入口带 CONTINUATION_GUARD_SELFTEST_MARK 指到自证建的标记文件，照样带着调用方设的值（refuse_break_outside_selftest）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里 case 的标签，括号里是那条「✗ 自检：<标签>」失败行在标签之后的开头）：
#   CONTINUATION_GUARD_BREAK=disable-check  decide() 一进来就放行，交回过的、中断过的都不判
#       主会话已有交回消息、子 agent 记录里还没有：拒（「应当返回 2，实际 0」）
#       子会话记录不在、主会话里最近一次通知是 failed：拒（「应当返回 2，实际 0」）
#       子会话记录不在、主会话里有交回消息：拒（「应当返回 2，实际 0」）
#       子会话记录不在时拒绝说明写上下文与续做次数取不到（「应当返回 True，实际 False」）
#       撞过限额（failed）的上下文小也不续（「应当返回 2，实际 0」）
#       被停（killed）的不续（「应当返回 2，实际 0」）
#       已交回（SubagentHandback 送达）的不续（「应当返回 2，实际 0」）
#       共用样本：先被拒、后来送达的不续（「应当返回 2，实际 0」）
#       stdin:被停过的续做（「应当返回 2，实际 0」）
#       stdin:已交回的续做（「应当返回 2，实际 0」）
#       stdin:子会话记录不在、主会话里被停过的续做（「应当返回 2，实际 0」）
#       「读不到共用模块时放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#       「主会话记录不在时放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   CONTINUATION_GUARD_BREAK=anyrecord  latest_task_status() 不只认任务通知：主 agent 自己命令里写着的「<task-id>…<status>completed</status>」也算，盖掉真通知的 killed
#       被停（killed）的不续（「应当返回 2，实际 0」）
#       stdin:被停过的续做（「应当返回 2，实际 0」）
#   CONTINUATION_GUARD_BREAK=ignore-main-handback  handback_in_session_transcript() 不看主会话记录里的交回消息，只看子 agent 自己的会话记录
#       主会话已有交回消息、子 agent 记录里还没有：拒（「应当返回 2，实际 0」）
#       子会话记录不在、主会话里有交回消息：拒（「应当返回 2，实际 0」）
#   CONTINUATION_GUARD_BREAK=missing-record-passes  decide() 找不到子 agent 的会话记录就放行，不看主会话里的任务通知与交回消息（改前的样子）
#       子会话记录不在、主会话里最近一次通知是 failed：拒（「应当返回 2，实际 0」）
#       子会话记录不在、主会话里有交回消息：拒（「应当返回 2，实际 0」）
#       子会话记录不在时拒绝说明写上下文与续做次数取不到（「应当返回 True，实际 False」）
#       stdin:子会话记录不在、主会话里被停过的续做（「应当返回 2，实际 0」）
#       「主会话记录不在时放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   CONTINUATION_GUARD_BREAK=library-unread  当作读不到 scripts/agents/subagent_transcript.py：子 agent 自己会话记录里的交回不判、放行，stderr 写「读不到共用模块」
#       已交回（SubagentHandback 送达）的不续（「应当返回 2，实际 0」）
#       共用样本：先被拒、后来送达的不续（「应当返回 2，实际 0」）
#       stdin:已交回的续做（「应当返回 2，实际 0」）
#   CONTINUATION_GUARD_BREAK=unparsable-input-silent  main() 读不出输入（不是合法的 JSON 对象）时静默放行，不打诊断、不记检出（改前的样子）
#       stdin:坏 JSON 的 stderr 有诊断（「应当返回 True，实际 False」）
#       stdin:坏 JSON 记一条检出（结局只记、会话号读不到）（「应当返回 (1, '只记', '读不到', True)，实际 (0, None, None, False)」）
#       stdin:JSON 数组照常放行并记一条检出（「应当返回 (0, 2)，实际 (0, 0)」）
#   CONTINUATION_GUARD_BREAK=library-unread-unrecorded  读不到 scripts/agents/subagent_transcript.py 时只打 stderr、不记检出（改前的样子）
#       「读不到共用模块时放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   CONTINUATION_GUARD_BREAK=transcript-missing-unrecorded  主会话记录不在时照常放行、不记检出（改前的样子）
#       「主会话记录不在时放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   CONTINUATION_GUARD_BREAK=exception-unrecorded  判的时候抛异常不接住、照原样抛出（改前的样子：进程退 1，放行、不记检出）
#       「判的时候抛异常时放行并记一条检出（结局只记） 应当返回 (0, 1, '只记', True)，实际 …」
#   CONTINUATION_GUARD_BREAK=honor-outside-selftest  平常跑设了弄坏开关也照认，不拒（改前的样子）；关守卫项只在单独一项时认，与别的项同写照拒（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）
#       平常跑设了弄坏开关：退 2 并点名只在自证里认（「应当返回 (2, True)，实际 (0, False)」）
#   CONTINUATION_GUARD_BREAK=honor-outside-selftest-with-others  带标记的自证子进程里，关守卫项与别的项同写也照认（改前的样子：honor-outside-selftest 写在哪一项都认；只在带标记的子进程里认，平常跑同写照拒）
#       「关守卫项与别的项同写：带标记的子进程里也退 2 并点名它 应当返回 (2, True)，实际 (0, False)」
# 送达的认法在 scripts/agents/subagent_transcript.py，它的开关（开关表在它的文件头）打开后这里红的项：
#   SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  被交回闸拒的交回也算送达
#       调过 SubagentHandback 但 tool_result 是错误的放行（「应当返回 0，实际 2」）
#       调过 SubagentHandback 但 tool_result 是 success:false（没送出）的放行（「应当返回 0，实际 2」）
#       共用样本：被拒之后再没送达的放行（「应当返回 0，实际 2」）
# hook-events: PreToolUse:SendMessage
# gate-similar: runner-dispatch-guard.sh 同样读主会话记录里的交回消息（<agent-message from=…> [Subagent hand-back]），但它挂 Agent|Task、判在跑的实现员与 opus 数；这里挂 SendMessage、判收件人交回过没有，只按一个 agent 号找一种记号，两行正则，不值得抽共用
# gate-overlap:copy-kept bash-command-detector.sh refuse_break_outside_selftest() 认标记文件、拆开关项、关守卫项只在单独一项时认那十来行两份各一：守卫要在读任何共用模块之前判（共用模块读不到时开关照样只在自证里认），抽进同目录的库，库读不到的那一刻守卫跟着没了
# gate-similar: handback-guard.sh 挂 SubagentHandback、在交回那一刻判交回正文；这里在交回之后判能不能再给它发消息
# 子 agent 会话记录里「交回送达没有」的认法不在这里写：调 scripts/agents/subagent_transcript.py（看门狗 agent-watch.py、结局账 agent-cost.py 调同一份）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 runner-dispatch-guard.sh 同一个写法）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import json, os, re, shutil, subprocess, sys, tempfile
from datetime import datetime, timezone

try:   # 「交回」（送达）的认法只在 scripts/agents/subagent_transcript.py 一处（看门狗与结局账同取它）；读不到时 ① 的子 agent 那一侧不判、放行，stderr 写一句
    sys.dont_write_bytecode = True
    if os.environ.get("CONTINUATION_GUARD_BREAK") == "library-unread":
        raise ImportError("CONTINUATION_GUARD_BREAK=library-unread")
    sys.path.insert(0, os.path.join(sys.argv[1], "..", "..", "scripts", "agents"))
    import subagent_transcript as transcripts
    transcripts_error = None
except Exception as error:   # 文件不在、语法错都算读不到
    transcripts, transcripts_error = None, error

SELFTEST_MARK_VARIABLE = "CONTINUATION_GUARD_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "continuation-guard-selftest"

# 关守卫项（records/2026-10-23-subagent拆分提案.md 第一百二十二节 ②）：只在它是唯一一项时认；与别的项同写照拒并点名它（带标记的自证子进程里也拒）。
#   GUARD_OFF_WITH_OTHERS_ITEM 是它的弄坏开关：带标记的自证子进程里同写着它时，同写的关守卫项照认（改前的样子）
GUARD_OFF_ITEM = "honor-outside-selftest"
GUARD_OFF_WITH_OTHERS_ITEM = "honor-outside-selftest-with-others"

def refuse_break_outside_selftest():
    """弄坏开关只在自证里认：平常跑（不是 --selftest，也不是自证起的子进程）设了 CONTINUATION_GUARD_BREAK 就退 2，点名变量只在自证里认；
    没设、--selftest、标记文件在而内容对得上的返回 0。关守卫项 honor-outside-selftest 单独一项时不拒（改前的样子）；与别的项同写一律拒、点名它
    （带标记的子进程里同写着 honor-outside-selftest-with-others 的除外）。"""
    value = os.environ.get("CONTINUATION_GUARD_BREAK", "")
    if not value or (len(sys.argv) > 2 and sys.argv[2] == "--selftest"):
        return 0
    items = [item for item in value.split(",") if item]
    mark = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    try:
        marked = bool(mark) and open(mark, encoding="utf-8").read().strip() == SELFTEST_MARK_CONTENT
    except OSError:
        marked = False
    others = [item for item in items if item != GUARD_OFF_ITEM]
    if GUARD_OFF_ITEM in items and others and not (marked and GUARD_OFF_WITH_OTHERS_ITEM in others):
        print(f"✗ 续做闸：CONTINUATION_GUARD_BREAK={value}：关守卫项 {GUARD_OFF_ITEM} 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；这一次没判", file=sys.stderr)
        print("→ 怎么办：unset CONTINUATION_GUARD_BREAK 再来；要看某一项红在哪一格，跑 CONTINUATION_GUARD_BREAK=<项> bash .claude/hooks/continuation-guard.sh --selftest，一次只设那一项", file=sys.stderr)
        return 2
    if marked or items == [GUARD_OFF_ITEM]:
        return 0
    print(f"✗ 续做闸：CONTINUATION_GUARD_BREAK={value} 是弄坏开关，只在自证（continuation-guard.sh --selftest）里认；平常跑设了就拒，这一次没判", file=sys.stderr)
    print("→ 怎么办：unset CONTINUATION_GUARD_BREAK 再来；要看开关红在哪一格，跑 bash .claude/hooks/continuation-guard.sh --selftest（开关写在文件头）", file=sys.stderr)
    return 2

def subagent_transcript_path(hook_input, recipient):
    transcript_path = hook_input.get("transcript_path") or ""
    if not transcript_path.endswith(".jsonl") or not re.fullmatch(r"[0-9a-z]{8,}", recipient or ""):
        return None
    candidate = os.path.join(transcript_path[:-len(".jsonl")], "subagents", f"agent-{recipient}.jsonl")
    return candidate if os.path.isfile(candidate) else None

def context_and_continuations(path):
    """最后一次调用的上下文（读缓存 + 写缓存 + 没进缓存的输入），与续做消息的条数。会话记录不在、读不了时两样都是 None；
    记录里没有一次带 usage 的调用时上下文是 None（取不到，不当 0）。"""
    if path is None:
        return None, None
    last_context, continuations = None, 0
    try:
        handle = open(path, encoding="utf-8", errors="replace")
    except OSError:
        return None, None
    for line in handle:
        try:
            record = json.loads(line)
        except ValueError:
            continue
        message = record.get("message")
        if not isinstance(message, dict):
            continue
        if message.get("role") == "assistant":
            usage = message.get("usage") or {}
            if usage:
                last_context = ((usage.get("cache_read_input_tokens") or 0) + (usage.get("cache_creation_input_tokens") or 0)
                                + (usage.get("input_tokens") or 0))
        elif message.get("role") == "user" and (record.get("origin") or {}).get("kind") == "coordinator":
            continuations += 1
    handle.close()
    return last_context, continuations

TASK_STATUS = re.compile(r"<task-id>([^<]+)</task-id>(?:(?!<task-id>).)*?<status>([a-z_]+)</status>")

def latest_task_status(session_transcript, recipient):
    """本会话记录里这个 agent 最近一次任务通知的状态（completed / failed / killed …），没有通知就是 None。"""
    if not session_transcript or not os.path.isfile(session_transcript):
        return None
    status = None
    for line in open(session_transcript, encoding="utf-8", errors="replace"):
        if recipient not in line or "<status>" not in line:
            continue
        try:
            record = json.loads(line)
        except ValueError:
            continue
        # 只认真正的任务通知：origin 是 task-notification 的消息、排队的附件、队列操作。主 agent 自己的命令与回复里也可能写着
        # 「<task-id>…</task-id>…<status>failed</status>」（2026-09-19 实测：一条 grep 命令的文本让被停的 agent 读成了 failed）。
        is_notification = ((record.get("origin") or {}).get("kind") == "task-notification"
                           or record.get("type") == "queue-operation"
                           or (record.get("attachment") or {}).get("type") == "queued_command")
        if not is_notification and os.environ.get("CONTINUATION_GUARD_BREAK") != "anyrecord":
            continue
        for task_id, found in TASK_STATUS.findall(line):
            if task_id == recipient:
                status = found
    return status

def handback_delivered(path, unjudged=None):
    """子 agent 的会话记录里有没有一次送达的交回（认法在 scripts/agents/subagent_transcript.py）；读不到那份库时交 False（不判，放行），stderr 写一句，
    给了 unjudged（列表）的往里记 (弄坏开关名, 一句)，放行时由入口照「放行但必记检出」记进检出记录。"""
    if transcripts is None:
        print(f"! 续做闸读不到共用模块 scripts/agents/subagent_transcript.py（{transcripts_error!r}），子 agent 会话记录里的交回没判", file=sys.stderr)
        if unjudged is not None:
            unjudged.append(("library-unread-unrecorded", f"读不到共用模块 scripts/agents/subagent_transcript.py（{transcripts_error!r}），{path} 里的交回送达没有没判"))
        return False
    return transcripts.handback_delivered(path)

def handback_in_session_transcript(session_transcript, recipient):
    """主会话记录里有没有它的交回消息（<agent-message from="<id>"> 后跟 [Subagent hand-back]）：交回的结果要过一会儿才落进子 agent 自己的
    会话记录，主会话那一侧先到——只看子 agent 那一份，交回之后几秒内发的消息会被放行。"""
    if not session_transcript or not os.path.isfile(session_transcript) or os.environ.get("CONTINUATION_GUARD_BREAK") == "ignore-main-handback":
        return False
    marker = re.compile(r'agent-message from=\\?"' + re.escape(recipient) + r'\\?">(?:\\n|\s)*\[Subagent hand-back\]')
    for line in open(session_transcript, encoding="utf-8", errors="replace"):
        if recipient in line and "[Subagent hand-back]" in line and marker.search(line):
            return True
    return False

def decide(hook_input, unjudged=None):
    """返回 (0, None) 放行；(2, 说明) 拒绝。判不了的几处（主会话记录不在、读不到共用模块）给了 unjudged（列表）的往里记 (弄坏开关名, 一句)。"""
    if os.environ.get("CONTINUATION_GUARD_BREAK") == "disable-check":
        return 0, None
    tool_input = hook_input.get("tool_input") or {}
    recipient = tool_input.get("to") or ""
    if not recipient:
        return 0, None
    path = subagent_transcript_path(hook_input, recipient)
    if path is None and os.environ.get("CONTINUATION_GUARD_BREAK") == "missing-record-passes":   # 弄坏开关：改前的样子，子会话记录不在就放行
        return 0, None
    session_transcript = hook_input.get("transcript_path")
    if (not session_transcript or not os.path.isfile(str(session_transcript))) and unjudged is not None:
        unjudged.append(("transcript-missing-unrecorded", f"主会话记录不在（transcript_path：{session_transcript!r}），收件人 {recipient} 的任务通知、交回消息与它自己的会话记录都没看"))
    status = latest_task_status(session_transcript, recipient)   # 先看主会话：子会话记录不在时这两样照判
    if status not in ("failed", "killed"):
        if handback_in_session_transcript(session_transcript, recipient):
            evidence = "主会话记录里有它的交回消息"
        elif path is not None and handback_delivered(path, unjudged):
            evidence = "它的会话记录里有一次送达的 SubagentHandback"
        else:
            return 0, None
        return 2, (f"✗ 子 agent {recipient} 已经交回过（{evidence}），交回之后不续做，新活一律新派。\n"
                   "→ 怎么办：不给它发消息。派一个新的同类 agent，派发提示指到它交回的报告与留下的产物；要它会话里的进度，跑 "
                   f"`python3 scripts/agents/agent-handover.py --agent {recipient} --out <交接摘要.md>` 抽出来给新 agent 读"
                   "（交接摘要只给新的读，主 agent 不读它）。")
    last_context, continuations = context_and_continuations(path)
    if path is None:
        sizes = "它的会话记录不在本会话的 subagents 目录里，上下文与续做次数取不到"
    else:
        sizes = (f"它最后一次调用的上下文 {'取不到' if last_context is None else f'{last_context / 10000:.1f} 万'}，"
                 f"已被续做 {'取不到' if continuations is None else f'{continuations} 次'}")
    reason = (f"✗ 子 agent {recipient} 最近一次任务通知是 {status}（撞限额、被停或报错），中断过的不续，新开一个接着做（{sizes}）。\n")
    return 2, (reason +
               "→ 怎么办：不续这一个。跑 `python3 scripts/agents/agent-handover.py --agent "
               f"{recipient} --out <交接摘要.md>` 从它的会话记录里抽出派发提示、续做消息、写过的文件、跑过的命令与它最后说的话；"
               "派一个新的同类 agent，派发提示指到交接摘要与它留下的产物，第一步先核现场（编不编得过、产物是不是这一轮的），再接着做。"
               "交接摘要只给新的读，主 agent 不读它；接续非得经过主 agent 的上下文，就干脆从头重派。")

def selftest(hook_dir):
    work = tempfile.mkdtemp()
    try:
        # 弄坏开关只在自证里认：标记文件写进环境，自证起的子进程带着它才认调用方设的开关值
        selftest_mark = os.path.join(work, "selftest-mark")
        with open(selftest_mark, "w", encoding="utf-8") as mark_handle:
            mark_handle.write(SELFTEST_MARK_CONTENT + "\n")
        os.environ[SELFTEST_MARK_VARIABLE] = selftest_mark
        session_transcript = os.path.join(work, "session.jsonl")
        open(session_transcript, "w").close()
        subagents = os.path.join(work, "session", "subagents")
        os.makedirs(subagents)
        def write_agent(agent_id, context_tokens, continuations, extra_records=()):
            records = [{"message": {"role": "assistant", "usage": {"input_tokens": 10, "cache_read_input_tokens": context_tokens - 10}}}]
            records += [{"origin": {"kind": "coordinator"}, "message": {"role": "user", "content": "续派"}}] * continuations
            records += list(extra_records)
            with open(os.path.join(subagents, f"agent-{agent_id}.jsonl"), "w") as handle:
                handle.write("\n".join(json.dumps(record, ensure_ascii=False) for record in records) + "\n")
        # 每份真会话记录开头都有：工具清单（带着 SubagentHandback 这个名字）与派发提示（常常也提到它）
        opening_records = [
            {"type": "attachment", "attachment": {"type": "prompt_snapshot", "tools": [
                {"name": "SubagentHandback", "description": "Deliver your final report to the agent that spawned you"}]}},
            {"type": "user", "message": {"role": "user", "content": "干完用 SubagentHandback 交回"}}]
        def tool_call(call_id, tool_name, tool_input):
            return {"type": "assistant", "message": {"role": "assistant", "content": [
                {"type": "tool_use", "id": call_id, "name": tool_name, "input": tool_input}]}}
        def tool_result(call_id, content, is_error=None):
            block = {"type": "tool_result", "tool_use_id": call_id, "content": content}
            if is_error is not None:
                block["is_error"] = is_error
            return {"type": "user", "message": {"role": "user", "content": [block]}}
        def result_text(payload):
            return [{"type": "text", "text": json.dumps(payload)}]
        write_agent("aaaa1111bbbb2222", 120_000, 1)
        write_agent("cccc3333dddd4444", 950_000, 3)
        write_agent("gggg7777hhhh8888", 80_000, 0)
        write_agent("iiii9999jjjj0000", 80_000, 1)
        write_agent("kkkk1212llll3434", 80_000, 0)
        write_agent("mmmm1313nnnn1414", 80_000, 0, opening_records + [
            tool_call("toolu_handback_delivered", "SubagentHandback", {"message": "报告"}),
            tool_result("toolu_handback_delivered", result_text({"success": True, "message": "Report delivered to your caller."}))])
        write_agent("oooo1515pppp1616", 80_000, 0, opening_records + [
            tool_call("toolu_handback_errored", "SubagentHandback", {"message": "报告"}),
            tool_result("toolu_handback_errored", "<tool_use_error>InputValidationError</tool_use_error>", is_error=True)])
        write_agent("qqqq1717rrrr1818", 80_000, 0, opening_records + [
            tool_call("toolu_handback_not_sent", "SubagentHandback", {"message": "报告"}),
            tool_result("toolu_handback_not_sent", result_text({"success": False, "message": "Nothing was sent"}))])
        write_agent("ssss1919tttt2020", 80_000, 0, opening_records + [
            tool_call("toolu_read_definition", "Read", {"file_path": "agents/x.md"}),
            tool_result("toolu_read_definition", "交回：最后一步调 SubagentHandback，把报告交给主 agent")])
        write_agent("uuuu2121vvvv2222", 80_000, 0, opening_records + [
            tool_call("toolu_background_build", "Bash", {"command": "cargo build", "run_in_background": True}),
            tool_result("toolu_background_build", "Command running in background with ID: b1")])
        write_agent("wwww2323xxxx2424", 80_000, 0, opening_records + [
            tool_call("toolu_handback_pending", "SubagentHandback", {"message": "报告"})])
        # scripts/agents/subagent_transcript.py 的两份共用样本（看门狗与 agent-cost.py 的自证判的是同两份）
        if transcripts is not None:
            write_agent("yyyy2525zzzz2626", 80_000, 0, transcripts.sample_refused_then_delivered())
            write_agent("yyyy2727zzzz2828", 80_000, 0, transcripts.sample_refused_only())
        def notification(agent_id, status):
            return json.dumps({"origin": {"kind": "task-notification"}, "message": {"role": "user", "content":
                               f"<task-notification>\n<task-id>{agent_id}</task-id>\n<output-file>x</output-file>\n<status>{status}</status>\n</task-notification>"}})
        with open(session_transcript, "w") as handle:
            fake_command = json.dumps({"message": {"role": "assistant", "content": [{"type": "tool_use", "name": "Bash", "input": {"command":
                                       "grep '<task-id>kkkk1212llll3434</task-id>.*<status>completed</status>' session.jsonl"}}]}})
            handle.write("\n".join([notification("gggg7777hhhh8888", "failed"),
                                     notification("iiii9999jjjj0000", "failed"), notification("iiii9999jjjj0000", "completed"),
                                     notification("kkkk1212llll3434", "killed"), notification("mmmm1313nnnn1414", "completed"),
                                     notification("uuuu2121vvvv2222", "completed"), fake_command]) + "\n")
        # 主会话那一侧已经收到交回消息、子 agent 自己的会话记录里还没落下交回结果（2026-09-26 那一次：交回之后 5 秒就续做）
        write_agent("wwww3131xxxx3232", 1000, 0)
        with open(session_transcript, "a") as handle:
            handle.write(json.dumps({"type": "user", "content": '<agent-message from="wwww3131xxxx3232">\n[Subagent hand-back] 报告'}) + "\n")
            # 子 agent 的会话记录不在（没有 write_agent），只有主会话里的任务通知或交回消息（开关 missing-record-passes 下这几格红）
            handle.write("\n".join([notification("abab4141cdcd5151", "failed"), notification("abab8181cdcd9191", "killed"),
                                     json.dumps({"type": "user", "content": '<agent-message from="efef6161abab7171">\n[Subagent hand-back] 报告'})]) + "\n")
        def case(label, recipient, message, want):
            hook_input = {"tool_name": "SendMessage", "transcript_path": session_transcript, "tool_input": {"to": recipient, "message": message}}
            return (label, want, decide(hook_input)[0])
        def missing_record_message_says_unavailable():
            message = decide({"tool_name": "SendMessage", "transcript_path": session_transcript,
                              "tool_input": {"to": "abab4141cdcd5151", "message": "接着做"}})[1] or ""
            return "上下文与续做次数取不到" in message and "0.0 万" not in message and "续做 0 次" not in message
        cases = [
            case("主会话已有交回消息、子 agent 记录里还没有：拒", "wwww3131xxxx3232", "再补一格", 2),
            case("上下文小的续做", "aaaa1111bbbb2222", "接着做第二段", 0),
            case("上下文大的续做放行：大小不是理由", "cccc3333dddd4444", "接着做第二段", 0),
            case("收件人没有会话记录、主会话里也没有它的通知与交回：放", "eeee5555ffff6666", "接着做", 0),
            case("子会话记录不在、主会话里最近一次通知是 failed：拒", "abab4141cdcd5151", "接着做", 2),
            case("子会话记录不在、主会话里有交回消息：拒", "efef6161abab7171", "再补一件", 2),
            ("子会话记录不在时拒绝说明写上下文与续做次数取不到", True, missing_record_message_says_unavailable()),
            case("收件人是名字不是 id", "reviewer", "接着做", 0),
            case("撞过限额（failed）的上下文小也不续", "gggg7777hhhh8888", "限额重置了，接着做", 2),
            case("failed 之后又 completed 的放行", "iiii9999jjjj0000", "接着做", 0),
            case("被停（killed）的不续", "kkkk1212llll3434", "接着做", 2),
            case("已交回（SubagentHandback 送达）的不续", "mmmm1313nnnn1414", "再补一件", 2),
            case("调过 SubagentHandback 但 tool_result 是错误的放行", "oooo1515pppp1616", "接着做", 0),
            case("调过 SubagentHandback 但 tool_result 是 success:false（没送出）的放行", "qqqq1717rrrr1818", "接着做", 0),
            case("只有工具清单与正文里带着 SubagentHandback 这个名字的放行", "ssss1919tttt2020", "整点询问", 0),
            case("结束本轮在等后台任务、还没交回的放行", "uuuu2121vvvv2222", "整点询问", 0),
            case("调了 SubagentHandback、还没有 tool_result 的放行", "wwww2323xxxx2424", "接着做", 0),
            case("共用样本：先被拒、后来送达的不续", "yyyy2525zzzz2626", "再补一件", 2),
            case("共用样本：被拒之后再没送达的放行", "yyyy2727zzzz2828", "接着做", 0),
        ]
        script = os.path.join(hook_dir, "continuation-guard.sh")
        for label, recipient, want in (("stdin:被停过的续做", "kkkk1212llll3434", 2), ("stdin:上下文大的续做", "cccc3333dddd4444", 0),
                                       ("stdin:已交回的续做", "mmmm1313nnnn1414", 2), ("stdin:在等后台任务的询问", "uuuu2121vvvv2222", 0),
                                       ("stdin:子会话记录不在、主会话里被停过的续做", "abab8181cdcd9191", 2)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "SendMessage", "transcript_path": session_transcript,
                                                         "tool_input": {"to": recipient, "message": "接着做"}}),
                                       capture_output=True, text=True, env=dict(os.environ, AGENT_HOOK_DETECTIONS=os.path.join(work, "stdin-detections.jsonl")))
            cases.append((label, want, completed.returncode))
        # 判不了的照「放行但必记检出」：在进程里照钩子入口调 judge（检出记在自证自己的临时目录），读不到共用模块、主会话记录不在、判的时候抛异常各一格
        in_process_detections = os.path.join(work, "in-process-detections.jsonl")
        def judged_in_process(hook_input, replaced=None):
            """→ (退出码或「抛出 …」, 检出多了几行, 最后一行的结局, 最后一行的 findings 都以「续做闸没判全」起头)。"""
            before = len(open(in_process_detections, encoding="utf-8").readlines()) if os.path.exists(in_process_detections) else 0
            saved = {name: globals()[name] for name in (replaced or {})}
            saved_detections = os.environ.get("AGENT_HOOK_DETECTIONS")
            globals().update(replaced or {})
            os.environ["AGENT_HOOK_DETECTIONS"] = in_process_detections
            try:
                code = judge(hook_input, hook_dir)
            except Exception as error:   # 弄坏开关 exception-unrecorded：照原样抛出
                code = f"抛出 {type(error).__name__}"
            finally:
                globals().update(saved)
                if saved_detections is None:
                    os.environ.pop("AGENT_HOOK_DETECTIONS", None)
                else:
                    os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
            lines = [json.loads(line) for line in open(in_process_detections, encoding="utf-8")][before:] if os.path.exists(in_process_detections) else []
            return (code, len(lines), lines[-1].get("outcome") if lines else None,
                    bool(lines) and all(str(finding).startswith("续做闸没判全：") for finding in lines[-1].get("findings") or [None]))
        def raising_decide(hook_input, unjudged=None):
            raise RuntimeError("自证造的异常")
        cases.append(("读不到共用模块时放行并记一条检出（结局只记）", (0, 1, "只记", True),
                      judged_in_process({"tool_name": "SendMessage", "session_id": "s-library", "transcript_path": session_transcript,
                                         "tool_input": {"to": "mmmm1313nnnn1414", "message": "再补一件"}}, {"transcripts": None})))
        cases.append(("主会话记录不在时放行并记一条检出（结局只记）", (0, 1, "只记", True),
                      judged_in_process({"tool_name": "SendMessage", "session_id": "s-missing", "transcript_path": os.path.join(work, "absent.jsonl"),
                                         "tool_input": {"to": "kkkk1212llll3434", "message": "接着做"}})))
        cases.append(("判的时候抛异常时放行并记一条检出（结局只记）", (0, 1, "只记", True),
                      judged_in_process({"tool_name": "SendMessage", "session_id": "s-raise", "transcript_path": session_transcript,
                                         "tool_input": {"to": "kkkk1212llll3434", "message": "接着做"}}, {"decide": raising_decide})))
        # 输入不是合法的 JSON 对象：照常放行，stderr 写诊断、检出记录记一条（结局只记）；开关 unparsable-input-silent 下诊断与检出那几格红
        unjudged_detections = os.path.join(work, "unjudged-detections.jsonl")
        def run_raw(raw_text):
            return subprocess.run(["bash", script], input=raw_text, capture_output=True, text=True,
                                  env=dict(os.environ, AGENT_HOOK_DETECTIONS=unjudged_detections))
        def unjudged_records():
            return [json.loads(line) for line in open(unjudged_detections, encoding="utf-8")] if os.path.exists(unjudged_detections) else []
        broken_entry = run_raw('{"tool_name": "SendMessage", "tool_input": {"to": "kkkk1212llll3434"')
        broken_records = unjudged_records()
        cases.append(("stdin:坏 JSON 放行", 0, broken_entry.returncode))
        cases.append(("stdin:坏 JSON 的 stderr 有诊断", True,
                      all(fragment in broken_entry.stderr for fragment in ("continuation-guard.sh", "不是合法的 JSON 对象", '{"tool_name": "SendMessage"', "检出记进"))))
        cases.append(("stdin:坏 JSON 记一条检出（结局只记、会话号读不到）", (1, "只记", "读不到", True),
                      (len(broken_records), broken_records[-1].get("outcome") if broken_records else None, broken_records[-1].get("session_id") if broken_records else None,
                       bool(broken_records) and all(str(finding).startswith("续做闸没判全：") for finding in broken_records[-1].get("findings") or [None]))))
        array_entry = run_raw("[1, 2]")
        cases.append(("stdin:JSON 数组照常放行并记一条检出", (0, 2), (array_entry.returncode, len(unjudged_records()))))
        # 平常跑（不带标记）设了弄坏开关：退 2 并点名变量；自证起的子进程（带标记）照认。honor-outside-selftest 设着时探针里也带上它，第一格红
        unmarked_environment = {key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE}
        probe_item = "honor-outside-selftest" if "honor-outside-selftest" in os.environ.get("CONTINUATION_GUARD_BREAK", "").split(",") else "disable-check"
        plain_run = subprocess.run(["bash", script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                   env=dict(unmarked_environment, CONTINUATION_GUARD_BREAK=probe_item, AGENT_HOOK_DETECTIONS=unjudged_detections))
        cases.append(("平常跑设了弄坏开关：退 2 并点名只在自证里认", (2, True),
                      (plain_run.returncode, "CONTINUATION_GUARD_BREAK" in plain_run.stderr and "自证" in plain_run.stderr)))
        marked_run = subprocess.run(["bash", script], input=json.dumps({"tool_name": "SendMessage", "transcript_path": session_transcript,
                                                                       "tool_input": {"to": "kkkk1212llll3434", "message": "接着做"}}),
                                    capture_output=True, text=True, env=dict(os.environ, CONTINUATION_GUARD_BREAK="disable-check"))
        cases.append(("自证起的子进程（带标记）照认弄坏开关", 0, marked_run.returncode))
        # 关守卫项与别的项同写：平常跑与带标记的子进程都退 2 并点名它只在单独一项时认；honor-outside-selftest-with-others 设着时带标记那一格的开关里也带上它，那一格红
        combined_item = GUARD_OFF_ITEM + ",disable-check"
        combined_plain = subprocess.run(["bash", script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                        env=dict(unmarked_environment, CONTINUATION_GUARD_BREAK=combined_item, AGENT_HOOK_DETECTIONS=unjudged_detections))
        cases.append(("关守卫项与别的项同写：平常跑退 2 并点名它只在单独一项时认", (2, True),
                      (combined_plain.returncode, GUARD_OFF_ITEM in combined_plain.stderr and "唯一一项" in combined_plain.stderr)))
        lenient = GUARD_OFF_WITH_OTHERS_ITEM in os.environ.get("CONTINUATION_GUARD_BREAK", "").split(",")
        combined_marked = subprocess.run(["bash", script, "--force", "--break-guard-probe-unknown-option"], input="", capture_output=True, text=True,
                                         env=dict(os.environ, CONTINUATION_GUARD_BREAK=combined_item + ("," + GUARD_OFF_WITH_OTHERS_ITEM if lenient else ""),
                                                  AGENT_HOOK_DETECTIONS=unjudged_detections))
        cases.append(("关守卫项与别的项同写：带标记的子进程里也退 2 并点名它", (2, True),
                      (combined_marked.returncode, GUARD_OFF_ITEM in combined_marked.stderr and "唯一一项" in combined_marked.stderr)))
    finally:
        shutil.rmtree(work)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() 与 scripts/agents/subagent_transcript.py 的 handback_delivered()；CONTINUATION_GUARD_BREAK 或 SUBAGENT_TRANSCRIPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {len(cases)} 种）：上下文大小不论、收件人没有会话记录而主会话里也没有它的通知与交回、中断之后又完成过、还没交回（在等后台任务、交回报错或没送出、只在工具清单里带着名字、"
          "被拒之后再没送达）的放行；交回送达过的（先被拒、后来送达的也算）、最近一次通知是 failed 或 killed 的一律拒绝；子 agent 会话记录不在时主会话里的 failed / killed 与交回消息照判，拒绝说明写上下文与续做次数取不到；"
          "输入不是合法的 JSON 对象（坏 JSON、JSON 数组）照常放行，stderr 写诊断并记一条只记的检出（会话号读不到写「读不到」）；"
          "读不到共用模块、主会话记录不在、判的时候抛异常照常放行并记一条只记的检出；"
          "弄坏开关平常跑设了就拒，关守卫项与别的项同写平常跑与带标记的子进程都拒并点名它")
    return 0

INPUT_PREFIX_BYTES = 200
INPUT_SESSION_FIELD = re.compile(r'"session_id"\s*:\s*"((?:[^"\\]|\\.)*)"')

def detections_path(hook_dir):
    """检出记录在哪：AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path；都取不到交 None。"""
    configured = os.environ.get("AGENT_HOOK_DETECTIONS")
    if configured:
        return configured
    try:
        sys.path.insert(0, hook_dir)
        import lib_work_root
        return lib_work_root.default_detections_path()
    except Exception:   # 文件不在、语法错、工作根的弄坏开关写错都算取不到
        return None

def report_unparsable_input(raw, problem, hook_dir):
    """输入不是合法的 JSON 对象：照「放行但必记检出」stderr 写一行可定位的诊断（钩子名、哪一处、输入的前 200 字节），检出记录追加一行
    （结局只记，findings 以「续做闸没判全」起头；会话号按字面找得到的照抄，找不到写「读不到」）。"""
    text = raw.decode("utf-8", "replace")
    found = INPUT_SESSION_FIELD.search(text)
    message = (f"输入不是合法的 JSON 对象（{problem}），这一次没判、照常放行；读到 {len(raw)} 字节，"
               f"前 {INPUT_PREFIX_BYTES} 字节：{raw[:INPUT_PREFIX_BYTES].decode('utf-8', 'replace')!r}")
    entry = {"time": datetime.now(timezone.utc).isoformat(), "session_id": found.group(1) if found else "读不到", "agent_id": None,
             "agent_type": "主 agent", "transcript_path": None, "command": "SendMessage（续做闸读不出输入）",
             "findings": [f"续做闸没判全：{message}"], "outcome": "只记"}
    path = detections_path(hook_dir)
    recorded = "检出没记成：取不到检出记录的位置（AGENT_HOOK_DETECTIONS 没设，同目录 lib_work_root.py 也读不到）"
    if path:
        try:
            os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
            with open(path, "a", encoding="utf-8") as handle:
                handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
            recorded = f"检出记进 {path}"
        except OSError as error:
            recorded = f"检出没记成：写不了 {path}（{error}）"
    print(f"! continuation-guard.sh：{message}；{recorded}", file=sys.stderr)

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest()
    if refused:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw.decode("utf-8"))
        if not isinstance(hook_input, dict):
            raise ValueError(f"顶层是 {type(hook_input).__name__}，不是对象")
    except ValueError as error:   # 判不了：照常放行，打诊断并记一条检出（弄坏开关 unparsable-input-silent 下静默放行，改前的样子）
        if os.environ.get("CONTINUATION_GUARD_BREAK") != "unparsable-input-silent":
            report_unparsable_input(raw, f"{type(error).__name__}: {error}", hook_dir)
        return 0
    return judge(hook_input, hook_dir)

def judge(hook_input, hook_dir):
    """读得懂的 hook 输入 → 退出码。放行而有判不了的几处、或判的时候抛异常，照「放行但必记检出」stderr 写一句、检出记录追加一行（record_unjudged）；
    各处的弄坏开关下那一处不记（改前的样子；exception-unrecorded 下照原样抛出）。"""
    unjudged = []
    try:
        code, message = decide(hook_input, unjudged)
    except Exception as error:
        if os.environ.get("CONTINUATION_GUARD_BREAK") == "exception-unrecorded":   # 弄坏开关：不接住，照原样抛出（改前的样子：进程退 1，放行、不记检出）
            raise
        code, message = 0, None
        unjudged = [("exception-unrecorded", f"判的时候抛异常（{type(error).__name__}: {error}），这一次没判")]
    if code:
        print(message, file=sys.stderr)
        return code
    problems = [text for switch, text in unjudged if os.environ.get("CONTINUATION_GUARD_BREAK") != switch]
    if problems:
        record_unjudged(hook_input, problems, hook_dir)
    return 0

def record_unjudged(hook_input, problems, hook_dir):
    """放行而有判不了的几处：照「放行但必记检出」检出记录追加一行（结局只记，findings 以「续做闸没判全」起头），stderr 写一句记进了哪里。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    session = hook_input.get("session_id")
    entry = {"time": datetime.now(timezone.utc).isoformat(), "session_id": session if isinstance(session, str) and session else "读不到",
             "agent_id": hook_input.get("agent_id"), "agent_type": hook_input.get("agent_type") or "主 agent", "transcript_path": hook_input.get("transcript_path"),
             "command": f"SendMessage 给 {tool_input.get('to') or '?'}", "findings": [f"续做闸没判全：{text}" for text in problems], "outcome": "只记"}
    path = detections_path(hook_dir)
    recorded = "检出没记成：取不到检出记录的位置（AGENT_HOOK_DETECTIONS 没设，同目录 lib_work_root.py 也读不到）"
    if path:
        try:
            os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
            with open(path, "a", encoding="utf-8") as handle:
                handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
            recorded = f"检出记进 {path}"
        except OSError as error:
            recorded = f"检出没记成：写不了 {path}（{error}）"
    print(f"! continuation-guard.sh：{'；'.join(problems)}；照常放行，{recorded}", file=sys.stderr)

sys.exit(main())
PY
