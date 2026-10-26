#!/usr/bin/env python3
# admission: always 每一次调都按此刻的会话记录与此刻的仓算账，上一次的结论不替这一次作保
# run-condition: command git
"""agent 体系的账表：子 agent 与主 agent 的用量、每类的挂钟与等待、派发与闸的拒绝、看门狗的退出、每个子 agent 的结局。

用法：
    agent-cost.py usage    [--root 会话目录] [--since YYYY-MM-DD] [--until YYYY-MM-DD]   # 子 agent 按类型的调用、上下文、读缓存、折算；主会话各自的账
    agent-cost.py timing   [--root …] [--since …] [--until …]      # 每类的挂钟中位、等待占比、没交回的；三方一轮的关键路径
    agent-cost.py dispatch [--root …] [--since …] [--until …]      # 派发按类型、派发闸与钩子的拒绝、重复派发、消息、看门狗退出码
    agent-cost.py outcome  [--root …] [--since …] [--until YYYY-MM-DD] [--repo 仓根] [--summary] [--jobs N] [--out 文件]
                                                                   # 每个子 agent 一行的结局账，再汇总；--until 只收那一天及之前开工的（含那一天，不给不设上界）；
                                                                   # --summary 只打汇总；--jobs 读会话记录的进程数（默认 12 与每台默认重活线程数取小）；成功句报 --since、--until 的取值
                                                                   # --out 把同一份输出（连同成功句）落进那个文件：排他新建，文件已在就拒（退 2，什么都不算、不写）、
                                                                   # 给出路；写完回读，与标准输出逐字相同才退 0，不同就删掉这次建的文件、退 1；窗口里没有子 agent（退 77）时不建文件。
                                                                   # 每批收尾落账的写法：outcome --since <批开工那天> --until <收尾那天> --summary --out <那一轮调度记录目录>/<轮>-outcome-<日期>.txt
    agent-cost.py --selftest                                       # 造假会话目录与假仓算一遍，钉住数；弄坏开关打开时必须判红

口径（与 records/2026-09-28-agent体系用量时间与调度分析.md 第一节同）：一次调用是一条带 usage 的 assistant 消息、按消息 id 去重；
上下文 = cache_read + cache_creation + input；折算 = 新输入 ×1 + 读缓存 ×0.1 + 5 分钟缓存写 ×1.25 + 1 小时缓存写 ×2 + 输出 ×5（相对量，不是账单）；
等待 = 相邻两次调用间隔超过 3 分钟。第一版分析用的五份脚本在 research/prompts/agent-cost-2026-09-28/，这一份把它们收进仓、加自证。
四个子命令收哪些子 agent 按起点（首条记录的时刻）筛，「交回」按送达认（调过 SubagentHandback、后面有它对应、不是错误的 tool_result；被交回闸拒的不算）：
两样都只在同目录 subagent_transcript.py 里定义，续做闸 .claude/hooks/continuation-guard.sh 与看门狗 agent-watch.py 取的是同一份。
--since、--until 写的日期按东京日期（Asia/Tokyo，DATE_ZONE）切天，与调度记录、存档文件名写的日期同一种（.claude/agent-common.md「规则怎么读」一节，日期用 TZ=Asia/Tokyo date +%F 取）：
--since D 从东京 D 日零点起，--until D 到东京 D+1 日零点为止（不含），四个子命令一样；会话记录里的时间戳带时区，按时刻比，不按它写的日期比。
主会话（<会话目录>/<会话 id>.jsonl）在 usage、dispatch 里按每条记录自己的时刻筛（records_between）：时刻落在窗口里的才算，一份跨几天的主会话只收窗口里那几天的调用与派发；
不带时刻的元数据条（标题、快照这类）不收；会话文件修改时刻早于 --since 的整份跳过（不读，它里面不会有更晚的记录）。usage 主会话那一行的「子agent」数起点落在窗口里的。

outcome 的口径（桶名照 records/2026-10-22-子agent结果落地审计.md「按结局分」；起点按子 agent 首条记录的时间，不按文件修改时间；交回的正文取最近一次送达的那一次）：
  写入落地：子 agent 每次成功的写，取新增的行（去首尾空白后不短于 12 个字，Edit 去掉 old_string 里已有的），比对
    「git 历史加进的行（所有分支，--since 往前多取 6 天）∪ git worktree list 列出的每棵工作树里的行（含主工作树，已跟踪与未跟踪未忽略的文件）」。
    认的写：Edit / Write / MultiEdit；Bash 里的 heredoc 写文件（cat / tee 加 >、>>）、python 写文件（heredoc、-c、先 heredoc 写成再执行的脚本，
    按字符串字面量取行）、scripts/kb 下 replace-once.py / replace-batch.py / insert-row.py 的实写（带 --dry-run 的不算）；tool_result 报错的调用不算。
  引用：交回之后，主会话与后来的子 agent 的工具调用里点到它交回里写的路径、它的 agent 号、或它草稿目录的目录名（路径里带变量、通配、花括号、
    循环拼出来的，把那一段当通配与目录名对；同一条命令里赋过的变量先展开）。交回里的路径取到空白、引号、括号、全角标点为止，扩展名后面直接粘着的汉字说明也去掉，
    去掉粘在后面的全角括号与说明文字。两个子 agent 共用同一个目录名时，按目录名点到的只算在下一个共用者开工之前（按完整路径、agent 号点到的不受此限）。
  草稿目录：子 agent 派发提示（首条记录）里行首「草稿目录：」那一行点名的路径，取第一个；认法只在 .claude/hooks/lib_prompt_scratch_line.py 的 scratch_line_values
    （列表、引用记号、粗体起头的，句中转述的，闭合了的围栏代码块里的都不认；原来一行里哪都认，见弄坏开关 draft-anywhere-in-line）。
  桶：三方腿类型（--repo 那棵树的 .claude/agents 里定义 frontmatter 写了 leg: three-way 的，经 .claude/hooks/lib_three_way_legs.py 读；
    派发闸、交回闸与骨架读同一处，拆分提案第九十节 ⑯；材料员这类以 three-way 起头而没登记的按别的类型分桶）交回了的分两桶：交回里点名的 research/prompts/<轮>-*-output.md 有一份在工作树（git worktree list 列出的每一棵）
    或 git 历史（所有分支加 HEAD，删过的也算）里、且同轮的 research/prompts/<轮>-main-verification.md 也在（轮名取 -output 前面最长的、有判决的那一段前缀）的
    记 verdict-archived，其余记 three-way-report-or-verdict-missing（攻方 landing-audit-r2 K3-2：原来按类型名一律记有效）；
    没交回的按最后一句分 cut-by-usage-limit、cut-by-api-error、in-flight（最后一条记录距此刻不到
    30 分钟）、stopped-salvaged（停掉，但后来有人点到）、stopped-abandoned；交回了的写了不少于 10 行的按落地率分 written-landed（不低于 70%）、
    written-partial（30% 到 70%）、written-overwritten（更低，主 agent 点到过）、written-unused（更低，没人点到）；其余按引用分 report-used-by-main、
    report-used-by-later-sub、report-unreferenced。
    70%、30%、10 行这三个门槛（LANDED_AT、PARTIAL_AT、WRITTEN_MINIMUM）没有出处；换值之后各桶怎么变的读数见
    research/prompts/landing-audit-r2-main-verification.md K3-1。
  返工字样：描述里带 REWORK_WORDS 的一个词（修（「修订」不算：跑前登记的修订是设计员与执行员的正常步骤）、重跑、重派、再派、重做、返工、续做、续派、接续、接手、接着做、（续）、补写、补做、改名、改号、改钉值、第 2 次及以后，与英文的 redispatch、rerun、renumber、retry、fix、fixes、fixed）。「后续批」这类只说次序的不算；fixture、fixtures、fixed-script 这类以 fix 起头的别的词不算（fix 后面要是词尾，连字符也不算词尾）。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证哪几项红（项名是 selftest() 里那一项失败句冒号前的名字，括号里是那条「✗ 自检：」的开头）：
  AGENT_COST_BREAK=no-dedupe  read_transcript() 不按消息 id 去重，同一个消息 id 的流式重复行各算一次
      子 agent 用量（「✗ 自检：子 agent 用量：应当 1 个、3 次调用（同一个消息 id 的重复行只算一次）」）
  AGENT_COST_BREAK=no-path-normalize  交回里的路径不在全角括号与说明文字处截断，主会话命令里的变量、通配、花括号、目录名一概不展开不对
      路径归一化（「✗ 自检：路径归一化：」）
  AGENT_COST_BREAK=no-bash-writes  Bash 里的 heredoc、python、replace-once / replace-batch / insert-row 的写一概不认，只认 Edit / Write / MultiEdit
      Bash 写入（「✗ 自检：Bash 写入：」）
  AGENT_COST_BREAK=narrow-rework-words  返工字样退回只认「修、重跑、续做、改名、改号」五个词（单字「修」把「修订」也算进去）
      返工词表（「✗ 自检：返工词表：」）
  AGENT_COST_BREAK=fix-no-word-end  返工字样里的 fix 退回只要求前面是词界（fixture、fixtures、fixed-script 也算返工）
      返工词表（「✗ 自检：返工词表：」）
  AGENT_COST_BREAK=day-cut-by-record-zone  --since、--until 的日期退回按会话记录时间戳的时区切天，不按东京日期
      东京日期切天（「✗ 自检：东京日期切天：」）、主会话按记录时刻切天（「✗ 自检：主会话按记录时刻切天：」）
  AGENT_COST_BREAK=three-way-by-type  三方腿退回按类型名一律记 verdict-archived，不核交回点名的报告与同轮的判决
      三方腿判决不在（「✗ 自检：三方腿判决不在：」）
      连带红的格：「✗ 自检：花费：每个子 agent 折算 1000 + 5 × 100 = 1500，合计应当 31500、三方腿 1 个应当 1500」
  AGENT_COST_BREAK=three-way-history-unread  三方腿只在工作树里找报告与判决，不读 git 历史
      三方腿判决在（「✗ 自检：三方腿判决在：」）
      连带红的格：「✗ 自检：花费：每个子 agent 折算 1000 + 5 × 100 = 1500，合计应当 31500、三方腿 1 个应当 1500」
  AGENT_COST_BREAK=leg-types-by-prefix  三方腿类型退回按类型名 three-way 开头认（three_way_leg_types_of、is_three_way_leg），不读定义 frontmatter 的 leg: three-way
      三方腿类型表（「✗ 自检：三方腿类型表：」）
  AGENT_COST_BREAK=draft-anywhere-in-line  草稿目录退回改前的认法：派发提示里「草稿目录：」在一行里哪都认（句中转述、列表里的、闭合了的围栏代码块里的也认），取第一个
      草稿目录认法（「✗ 自检：草稿目录认法：」）
  AGENT_COST_BREAK=main-session-by-mtime  records_between() 不看每条记录的时刻，主会话整份收（改前：会话文件修改时刻不早于 --since 就整份算进去）
      主会话按记录时刻切天（「✗ 自检：主会话按记录时刻切天：」）
  AGENT_COST_BREAK=outcome-out-clobber  outcome --out 不排他：文件已在也不拒，整份覆盖
      结局账落盘（「✗ 自检：结局账落盘：」）
同目录 subagent_transcript.py 的两个开关（判法在那一份，开关表也在它的文件头），打开后这里红的项：
  SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered  被交回闸拒的交回也算送达
      交回与起点（「✗ 自检：交回与起点：outcome 应当收…」「✗ 自检：交回与起点：timing 应当收 3 个…」）
  SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime  起点取文件修改时刻，不读首条记录
      交回与起点（「✗ 自检：交回与起点：outcome 应当收…」）、结局账落盘（「✗ 自检：结局账落盘：」：样本的起点被当成此刻，窗口里没有子 agent）
      连带红的格：「✗ 自检：东京日期切天：outcome --since 2026-10-21 --until 2026-10-21 应当收东京 2026-10-21 零点刚过与深夜开工的两个」
"""
import ast
import bisect
import collections
import concurrent.futures
import contextlib
import datetime
import glob
import io
import json
import multiprocessing
import os
import re
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import warnings
import zoneinfo
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "hosts"))
import heavy_threads  # noqa: E402  每台默认重活线程数只在这一份里取（登记位 .claude/heavy-thread-default）
import subagent_transcript as transcripts  # noqa: E402  子 agent 会话记录的「交回」与「起点」只在这一份里定义（续做闸、看门狗同取它）
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "hooks"))
import lib_three_way_legs as three_way_legs  # noqa: E402  三方腿类型表只登记在定义 frontmatter 的 leg: three-way，读法只在这一份里（派发闸、交回闸、骨架同取它）
import lib_prompt_scratch_line as prompt_scratch_line  # noqa: E402  派发提示里「草稿目录：」那一行的唯一认法（派发闸、交回闸、完工清理闸、看门狗取的是同一份）

REPOSITORY_ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
# Claude Code 按项目的绝对路径把 / 换成 - 起会话目录名（同 scripts/agents/watch.sh 的 ${root//\//-}）
DEFAULT_ROOT = os.path.join(os.path.expanduser("~/.claude/projects"), REPOSITORY_ROOT.replace("/", "-"))
BROKEN = os.environ.get("AGENT_COST_BREAK", "")
UTC = datetime.timezone.utc   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
DATE_ZONE = zoneinfo.ZoneInfo("Asia/Tokyo")   # clock-times:allow --since、--until 的日期按东京日期切天，与调度记录、存档文件名写的日期同一种
AGENT_ID = re.compile(r"a[0-9a-f]{16}")


def day_start(day):
    """--since / --until 写的 YYYY-MM-DD → 那一天零点（东京日期）；弄坏开关 day-cut-by-record-zone 下退回按会话记录时间戳的时区切。"""
    zone = UTC if BROKEN == "day-cut-by-record-zone" else DATE_ZONE   # clock-times:allow 弄坏开关退回改前按会话记录时间戳的时区切天
    return datetime.datetime.fromisoformat(day).replace(tzinfo=zone)


def window_bounds(since_day, until_day=None):
    """账的窗口：[--since 那一天零点, --until 第二天零点)，都按东京日期；until_day 为 None 不设上界。"""
    since = day_start(since_day)
    return since, (day_start(until_day) + datetime.timedelta(days=1) if until_day else None)


def local_day(moment):
    """一个时刻落在东京日期的哪一天（每个子 agent 一行的「开始日期」）。"""
    return moment.astimezone(DATE_ZONE).date()
PRIME_MARKS = re.compile("[" + "".join(chr(code) for code in (0x2032, 0x2033, 0x2034, 0x02B9, 0x02BA)) + "]")   # 输出里不留角标字符：门禁 doc-text 的 prime-marks 格扫全仓，抄进记录的原文会被它判红


def timestamp_of(record):
    try:
        return datetime.datetime.fromisoformat(record.get("timestamp", "").replace("Z", "+00:00"))
    except ValueError:
        return None


def records(path):
    with open(path, errors="replace") as handle:
        for line in handle:
            try:
                yield json.loads(line)
            except ValueError:
                continue


def text_of(content):
    return content if isinstance(content, str) else (json.dumps(content, ensure_ascii=False) if content else "")


def folded(totals):
    return (totals["input"] + totals["cache_read"] * 0.1 + totals["write_5m"] * 1.25 + totals["write_1h"] * 2 + totals["output"] * 5)


def context_of(usage):
    return usage.get("cache_read_input_tokens", 0) + usage.get("cache_creation_input_tokens", 0) + usage.get("input_tokens", 0)


def median(values):
    return statistics.median(values) if values else 0


def recent(path, since):
    return datetime.datetime.fromtimestamp(os.path.getmtime(path), tz=UTC) >= since   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点


def records_between(path, since, until=None):
    """主会话记录里时刻落在 [since, until) 的那些条（文件头「主会话 … 按每条记录自己的时刻筛」；窗口由 window_bounds 按东京日期给）。
    不带时刻的元数据条不收；时间戳不带时区的按零时区补上。弄坏开关 main-session-by-mtime 下整份收（改前的样子）。"""
    for record in records(path):
        if BROKEN == "main-session-by-mtime":
            yield record
            continue
        moment = timestamp_of(record)
        if moment is None:
            continue
        if moment.tzinfo is None:
            moment = moment.replace(tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，不带时区的按它补上
        if moment >= since and (until is None or moment < until):
            yield record


def window_label(since, until=None):
    """usage、timing、dispatch 标题里的窗口：「<since> 起」，给了 --until 再接「、到 <until 那一天> 为止」（until 是上界，东京日期第二天零点）。"""
    return f"{since.date()} 起" + (f"、到 {(until.astimezone(DATE_ZONE) - datetime.timedelta(days=1)).date()} 为止" if until else "")


def usage_key(message, record):
    """一次调用的去重键：同一个消息 id 的流式重复行只算一次（弄坏开关 no-dedupe 退回按记录号）。"""
    return (message.get("id") or record.get("uuid")) if BROKEN != "no-dedupe" else record.get("uuid") or id(record)


def usage_totals(calls):
    """一串去重后的 usage 加成五个量；read_transcript 与 outcome 的每个子 agent 花费同走这一处。"""
    totals = collections.Counter()
    for usage in calls:
        creation = usage.get("cache_creation") or {}
        totals["input"] += usage.get("input_tokens", 0)
        totals["cache_read"] += usage.get("cache_read_input_tokens", 0)
        totals["write_5m"] += creation.get("ephemeral_5m_input_tokens", usage.get("cache_creation_input_tokens", 0) if not creation else 0)
        totals["write_1h"] += creation.get("ephemeral_1h_input_tokens", 0)
        totals["output"] += usage.get("output_tokens", 0)
    return totals


def read_transcript(path, since=None, until=None):
    """一份会话记录：去重后的调用（按次序）、首末时刻、交回时刻（最近一次送达的那一次，认法在 subagent_transcript.py）、工具调用计数。
    给了 since 只读时刻落在 [since, until) 的记录（records_between，主会话用）；不给整份读（子 agent 按起点筛过了）。"""
    seen, order, first, last = {}, [], None, None
    tools = collections.Counter()
    handbacks = transcripts.HandbackTracker()
    for record in (records(path) if since is None else records_between(path, since, until)):
        moment = timestamp_of(record)
        if moment:
            first = first or moment
            last = moment
        handbacks.feed(record)
        if record.get("type") != "assistant":
            continue
        message = record.get("message", {})
        usage = message.get("usage")
        if usage:
            key = usage_key(message, record)
            if key not in seen:
                order.append(key)
            seen[key] = usage
        for block in message.get("content", []) if isinstance(message.get("content"), list) else []:
            if isinstance(block, dict) and block.get("type") == "tool_use":
                tools[block.get("name")] += 1
    calls = [seen[key] for key in order]
    totals = usage_totals(calls)
    handback = handbacks.delivered["delivered_at"] if handbacks.delivered else None
    return {"calls": calls, "totals": totals, "first": first, "last": last, "handback": handback, "tools": tools,
            "wall": (last - first).total_seconds() if first and last else 0}


def subagent_files(root, since, until=None):
    """起点（首条记录的时刻，subagent_transcript.started_between）落在 [since, until) 里的子 agent：(类型, 会话记录路径)。"""
    for meta in sorted(glob.glob(os.path.join(root, "*", "subagents", "agent-*.meta.json"))):
        transcript = meta.replace(".meta.json", ".jsonl")
        if os.path.exists(transcript) and transcripts.started_between(transcript, since, until):
            try:
                agent_type = json.load(open(meta)).get("agentType", "?")
            except (OSError, ValueError):
                agent_type = "?"
            yield agent_type, transcript


def command_usage(root, since, until=None):
    by_type = collections.defaultdict(collections.Counter)
    first_contexts = collections.defaultdict(list)
    for agent_type, transcript in subagent_files(root, since, until):
        data = read_transcript(transcript)
        if not data["calls"]:
            continue
        totals = data["totals"]
        totals["n"] = 1
        totals["calls"] = len(data["calls"])
        totals["folded"] = folded(totals)
        by_type[agent_type].update(totals)
        first_contexts[agent_type].append(context_of(data["calls"][0]))
    lines = [f"## 子 agent 按类型（{window_label(since, until)}）", f"{'类型':26}{'个数':>5}{'调用':>7}{'起步上下文中位':>12}{'读缓存(亿)':>10}{'5分钟写(亿)':>11}{'输出(万)':>9}{'折算(亿)':>9}"]
    grand = collections.Counter()
    for agent_type, totals in sorted(by_type.items(), key=lambda item: -item[1]["folded"]):
        lines.append(f"{agent_type:26}{totals['n']:>5}{totals['calls']:>7}{median(first_contexts[agent_type]):>12.0f}{totals['cache_read'] / 1e8:>10.2f}"
                     f"{totals['write_5m'] / 1e8:>11.2f}{totals['output'] / 1e4:>9.0f}{totals['folded'] / 1e8:>9.2f}")
        grand.update(totals)
    lines.append(f"{'合计':26}{grand['n']:>5}{grand['calls']:>7}{'':>12}{grand['cache_read'] / 1e8:>10.2f}{grand['write_5m'] / 1e8:>11.2f}{grand['output'] / 1e4:>9.0f}{grand['folded'] / 1e8:>9.2f}")
    lines.append("")
    lines.append(f"## 主会话（{window_label(since, until)}，按每条记录的时刻筛）")
    lines.append(f"{'会话':10}{'调用':>7}{'读缓存(亿)':>10}{'1小时写(亿)':>11}{'折算(亿)':>9}{'平均上下文(万)':>12}{'挂钟(时)':>9}{'子agent':>8}")
    main_grand = collections.Counter()
    for transcript in sorted(glob.glob(os.path.join(root, "*.jsonl"))):
        if not recent(transcript, since):
            continue
        data = read_transcript(transcript, since, until)
        if not data["calls"]:
            continue
        totals = data["totals"]
        subagents = sum(1 for path in glob.glob(transcript[:-6] + "/subagents/*.jsonl") if transcripts.started_between(path, since, until))
        lines.append(f"{os.path.basename(transcript)[:8]:10}{len(data['calls']):>7}{totals['cache_read'] / 1e8:>10.2f}{totals['write_1h'] / 1e8:>11.2f}"
                     f"{folded(totals) / 1e8:>9.2f}{totals['cache_read'] / max(1, len(data['calls'])) / 1e4:>12.0f}{data['wall'] / 3600:>9.1f}{subagents:>8}")
        totals["calls"] = len(data["calls"])
        totals["folded"] = folded(totals)
        main_grand.update(totals)
    lines.append(f"{'合计':10}{main_grand['calls']:>7}{main_grand['cache_read'] / 1e8:>10.2f}{main_grand['write_1h'] / 1e8:>11.2f}{main_grand['folded'] / 1e8:>9.2f}")
    return lines, by_type, main_grand


def command_timing(root, since, until=None):
    by_type = collections.defaultdict(list)
    for agent_type, transcript in subagent_files(root, since, until):
        data = read_transcript(transcript)
        if not data["calls"]:
            continue
        moments = [timestamp_of(record) for record in records(transcript) if record.get("type") == "assistant" and record.get("message", {}).get("usage")]
        moments = [moment for moment in moments if moment]
        gaps = [(later - earlier).total_seconds() for earlier, later in zip(moments, moments[1:])]
        waiting = sum(gap for gap in gaps if gap > 180)
        by_type[agent_type].append((data["wall"], waiting, data["handback"] is None))
    lines = [f"## 每类的挂钟与等待（{window_label(since, until)}）", f"{'类型':26}{'个数':>5}{'挂钟中位(分)':>11}{'挂钟p90':>8}{'等待占挂钟':>10}{'没交回':>7}"]
    total_wall = total_wait = 0
    for agent_type, rows in sorted(by_type.items(), key=lambda item: -len(item[1])):
        walls = sorted(row[0] / 60 for row in rows)
        wall_sum = sum(row[0] for row in rows)
        wait_sum = sum(row[1] for row in rows)
        total_wall += wall_sum
        total_wait += wait_sum
        lines.append(f"{agent_type:26}{len(rows):>5}{median(walls):>11.0f}{walls[int(len(walls) * 0.9)] if walls else 0:>8.0f}"
                     f"{(wait_sum / wall_sum if wall_sum else 0):>10.0%}{sum(1 for row in rows if row[2]):>7}")
    lines.append(f"合计挂钟 {total_wall / 3600:.0f} 小时，其中等待 {total_wait / 3600:.0f} 小时（{(total_wait / total_wall if total_wall else 0):.0%}）")
    return lines, by_type


def command_dispatch(root, since, until=None):
    dispatches = collections.Counter()
    launched = 0
    rejections = collections.defaultdict(collections.Counter)
    repeats = collections.Counter()
    messages = collections.Counter()
    notifications = collections.Counter()
    batch_sizes = []
    serial = 0
    for transcript in sorted(glob.glob(os.path.join(root, "*.jsonl"))):
        if not recent(transcript, since):
            continue
        session = os.path.basename(transcript)[:8]
        pending, dispatch_moments, handback_moments, id2command = {}, [], [], {}
        for record in records_between(transcript, since, until):
            record_type = record.get("type")
            message = record.get("message", {})
            content = message.get("content")
            moment = timestamp_of(record)
            if record_type == "assistant" and isinstance(content, list):
                for block in content:
                    if not (isinstance(block, dict) and block.get("type") == "tool_use"):
                        continue
                    name = block.get("name")
                    tool_input = block.get("input") or {}
                    if name in ("Agent", "Task"):
                        dispatches[tool_input.get("subagent_type", "?")] += 1
                        repeats[(session, (tool_input.get("description") or "")[:40])] += 1
                        pending[block.get("id")] = moment
                    elif name == "Bash":
                        id2command[block.get("id")] = tool_input.get("command", "")
                    elif name == "SendMessage":
                        to = str(tool_input.get("to", ""))
                        messages["给子 agent" if AGENT_ID.fullmatch(to) else "给别的会话"] += 1
            elif record_type == "user":
                text = text_of(content)
                if isinstance(content, list):
                    for block in content:
                        if not (isinstance(block, dict) and block.get("type") == "tool_result"):
                            continue
                        body = text_of(block.get("content"))
                        if block.get("tool_use_id") in pending:
                            started = pending.pop(block["tool_use_id"])
                            result = record.get("toolUseResult")
                            if isinstance(result, dict) and result.get("agentId"):
                                launched += 1
                                if started:
                                    dispatch_moments.append(started)
                        if "hook error" in body and "✗" in body:
                            hook = re.search(r"hook error: \[bash [^\]]*?/([a-z\-]+\.sh)\]", body)
                            reason = re.search(r"✗ ?(.{0,40})", body)
                            reason_text = PRIME_MARKS.sub("[角标]", re.sub(r"\d+", "N", re.sub(r"a[0-9a-f]{16}", "<id>", reason.group(1) if reason else "")))
                            rejections[hook.group(1) if hook else "?"][reason_text[:40]] += 1
                if "<task-notification>" in text:
                    status = re.search(r"<status>(\w+)</status>", text)
                    task = re.search(r"<task-id>([^<]+)</task-id>", text)
                    tool_use = re.search(r"<tool-use-id>([^<]+)</tool-use-id>", text)
                    command = id2command.get(tool_use.group(1), "") if tool_use else ""
                    exit_code = re.search(r"exit code (\d+)", text)
                    if task and AGENT_ID.fullmatch(task.group(1)):
                        kind = "子 agent"
                        if moment:
                            handback_moments.append(moment)
                    elif "watch.sh" in command or "agent-watch.py" in command:
                        kind = f"看门狗 退 {exit_code.group(1) if exit_code else '-'}"
                    else:
                        kind = "别的后台命令"
                    notifications[(kind, status.group(1) if status else "?")] += 1
        dispatch_moments.sort()
        current = 1
        for earlier, later in zip(dispatch_moments, dispatch_moments[1:]):
            if (later - earlier).total_seconds() <= 90:
                current += 1
            else:
                batch_sizes.append(current)
                current = 1
        if dispatch_moments:
            batch_sizes.append(current)
        handback_moments.sort()
        for started in dispatch_moments:
            index = bisect.bisect_right(handback_moments, started)
            if index > 0 and (started - handback_moments[index - 1]).total_seconds() <= 600:
                serial += 1
    lines = [f"## 派发（{window_label(since, until)}，主会话按每条记录的时刻筛）：Agent 工具调用 {sum(dispatches.values())} 次，真正起来的 {launched} 个"]
    lines += [f"  {agent_type:28}{count}" for agent_type, count in dispatches.most_common()]
    lines.append("## 钩子拒绝（主会话侧，按钩子与原因）")
    for hook, reasons in sorted(rejections.items(), key=lambda item: -sum(item[1].values())):
        lines.append(f"  {hook}: {sum(reasons.values())}")
        lines += [f"      {count:4d} {reason}" for reason, count in reasons.most_common(5)]
    repeated = [(key, count) for key, count in repeats.items() if count >= 3]
    lines.append(f"## 同一描述在同一会话派第三次以上：{len(repeated)} 种描述、{sum(count for _, count in repeated)} 次派发"
                 f"（占 {sum(count for _, count in repeated) / max(1, sum(dispatches.values())):.0%}）")
    lines.append(f"## 派发批（相邻 90 秒内算一批）：中位 {median(batch_sizes):.0f} 个，只派 1 个的批占 "
                 f"{sum(1 for size in batch_sizes if size == 1) / max(1, len(batch_sizes)):.0%}；交回后 10 分钟内补派 {serial} 次（{serial / max(1, launched):.0%}）")
    lines.append(f"## 主 agent 发的消息：{dict(messages)}")
    lines.append("## 后台任务通知（类别、状态）")
    lines += [f"  {count:5d} {kind} {status}" for (kind, status), count in notifications.most_common(12)]
    return lines, dispatches, rejections, launched


# ---------------------------------------------------------------------------------------------------------------------------------
# outcome：每个子 agent 交回之后落没落地、被谁用上
# ---------------------------------------------------------------------------------------------------------------------------------
LINE_MINIMUM = 12                  # 一行去首尾空白后不短于这么多个字才算一条可比对的行
HISTORY_MARGIN_DAYS = 6            # git 历史往 --since 之前多取几天
# 三个门槛没有出处（文件头「桶」一段）；换值之后各桶怎么变见 research/prompts/landing-audit-r2-main-verification.md K3-1
LANDED_AT, PARTIAL_AT, WRITTEN_MINIMUM = 0.7, 0.3, 10
IN_FLIGHT_MINUTES = 30
REWORK_WORDS_BEFORE_FIX = r"修(?!订)|重跑|重派|再派|重做|返工|续做|续派|接续|接手|接着做|（续）|补写|补做|改名|改号|改钉值|第\s*[2-9]\d*\s*次|redispatch|re-?run|renumber|retry|"
# fix、fixes、fixed 后面要是词尾（连字符不算）：fixture、fixtures、fixed-script 不算（攻方 landing-audit-r2 K3-4）
REWORK_WORDS = re.compile(REWORK_WORDS_BEFORE_FIX + r"\bfix(?:es|ed)?(?![\w-])", re.IGNORECASE)
REWORK_WORDS_FIX_PREFIX_FOR_BREAK = re.compile(REWORK_WORDS_BEFORE_FIX + r"\bfix", re.IGNORECASE)
NARROW_REWORK_WORDS_FOR_BREAK = re.compile("修|重跑|续做|改名|改号")


def rework_words():
    if BROKEN == "fix-no-word-end":
        return REWORK_WORDS_FIX_PREFIX_FOR_BREAK
    return NARROW_REWORK_WORDS_FOR_BREAK if BROKEN == "narrow-rework-words" else REWORK_WORDS
USAGE_LIMIT_TAIL = re.compile(r"limit · resets|weekly limit|usage limit")
API_ERROR_TAIL = re.compile(r"API Error|Not logged in|Overloaded")
REPOSITORY_MARKS = ("crates", ".claude", "scripts", "research", "records", "briefs", "litmus")
NOT_REPOSITORY = ("projects", "plans", "todos", "shell-snapshots")   # ~/.claude/ 底下的，不是仓里的 .claude
EVENT_TOOLS = ("Bash", "Read", "Edit", "Write", "MultiEdit", "Agent", "Task", "SendMessage", "Grep", "Glob")
EVENT_TEXT_LIMIT = 30000
PATH_CHARS = r"[A-Za-z0-9_.\-/~@+=%$一-鿿]"   # ASCII 加汉字（目录名里有「甲」「乙」这类）；全角标点、括号、空白不在里面，路径到那里为止
WORK_PATH = re.compile(r"(?:~|/home/[^/\s]+)/\.local/share/singlefs/work/" + PATH_CHARS + r"+|/tmp/claude-\d+/" + PATH_CHARS + r"+|research/prompts/" + PATH_CHARS + r"+")
# 改前的路径写法：只在标点处截断，全角括号与说明文字会粘进路径（弄坏开关 no-path-normalize 退回它）
OLD_WORK_PATH = re.compile(r"(?:~|/home/[^/\s]+)/\.local/share/singlefs/work/[^\s'\"`)>\]，。；、：:,]+|/tmp/claude-\d+/[^\s'\"`)>\]，。；、：:,]+|research/prompts/[^\s'\"`)>\]，。；、：:,]+")
PATH_TOKEN = re.compile(r"[^\s'\"`()<>|;&]*/[^\s'\"`()<>|;&]*")
VARIABLE_ASSIGNMENT = re.compile(r"(?:^|[\s;&(])([A-Za-z_][A-Za-z0-9_]*)=(\"[^\"\n]*\"|'[^'\n]*'|[^\s;&|()]+)")
VARIABLE_REFERENCE = re.compile(r"\$(?:\{([A-Za-z_]\w*)\}|([A-Za-z_]\w*))")
WILDCARD_CHARS = re.compile(r"[$*?{]")
NAME_MINIMUM = 6                   # 目录名用作键，至少这么长、带连字符（草稿目录都是这样起名的）；再短的什么都能碰上
FILE_LIKE = re.compile(r"\.[A-Za-z0-9]{1,6}$")
GENERIC_NAMES = {"work", "draft", "tmp", "report", "results", "prompts", "patch", "scratch"}
BUCKET_GLOSS = [
    ("written-landed", "写了、不低于 70% 的行进了提交或工作树"),
    ("written-partial", "写了、30% 到 70%"),
    ("verdict-archived", "三方腿、交回点名的 -output.md 在仓、同轮有判决"),
    ("three-way-report-or-verdict-missing", "三方腿、交回点名的 -output.md 不在仓或同轮没有判决"),
    ("report-used-by-main", "只交报告、交回后主 agent 点到"),
    ("report-used-by-later-sub", "只交报告、只有后来的子 agent 点到"),
    ("report-unreferenced", "只交报告、没人点到"),
    ("written-overwritten", "写了、低于 30%、主 agent 点到过"),
    ("written-unused", "写了、低于 30%、没人点到"),
    ("cut-by-usage-limit", "没交回、用量撞顶"),
    ("cut-by-api-error", "没交回、API 错"),
    ("stopped-salvaged", "没交回、被停掉、后来有人点到"),
    ("stopped-abandoned", "没交回、被停掉、没人点到"),
    ("in-flight", "没交回、还在跑"),
]
# 调度记录「按结局分」那张表的五行各含哪几桶；不在任何一行里的归「其余」
RECORD_TABLE_ROWS = [
    ("写了东西、不低于 70% 的行进了提交或工作树", ("written-landed",)),
    ("写了东西、30% 到 70%", ("written-partial",)),
    ("三方腿", ("verdict-archived",)),
    ("只交报告、交回后主 agent 有动作点到", ("report-used-by-main",)),
    ("看起来没效果的几桶", ("report-unreferenced", "written-overwritten", "written-unused", "report-used-by-later-sub", "stopped-abandoned", "cut-by-usage-limit")),
]


def repository_relative(path, allow_relative=False):
    """路径里最靠外的仓内顶层目录起的相对路径；认不出是仓内文件的回 None。"""
    if not path or not isinstance(path, str) or "\n" in path:
        return None
    parts = path.split("/")
    for index, segment in enumerate(parts):
        if segment in REPOSITORY_MARKS and (index > 0 or allow_relative):
            if segment == ".claude" and index + 1 < len(parts) and parts[index + 1] in NOT_REPOSITORY:
                continue
            return "/".join(parts[index:])
    return None


def significant_lines(text):
    return [line for line in (raw.strip() for raw in (text or "").splitlines()) if len(line) >= LINE_MINIMUM]


def moment_of(raw):
    try:
        return datetime.datetime.fromisoformat(raw.replace("Z", "+00:00"))
    except ValueError:
        return None


def shell_words(text):
    """shlex 切词；引号没闭合时读到哪算哪。"""
    lexer = shlex.shlex(text[:60000], posix=True, punctuation_chars=True)
    lexer.whitespace_split = True
    words = []
    try:
        for word in lexer:
            words.append(word)
    except ValueError:
        pass
    return words


HEREDOC = re.compile(r"(?<!<)<<(?!<)-?[ \t]*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")
OPERATOR_WORD = re.compile(r"^[;&|()<>]+$")
WRITE_HINT = re.compile(r"write_text\(|write_bytes\(|\.write\(|writelines\(|open\([^)]*[\"'][wax]b?\+?[\"']")
SCRIPT_CALL = re.compile(r"(?:^|[\s/'\"])(replace-once|replace-batch|insert-row)\.py(?=[\s'\";|&]|$)")
INSERT_ROW_VALUE_FLAGS = {"--after", "--before", "--after-literal", "--before-literal", "--line", "--absent"}


def heredoc_bodies(command):
    """命令里的每个 heredoc：(同一行里 << 之前的字, << 之后同一行的字, 正文)。"""
    found, position = [], 0
    while True:
        match = HEREDOC.search(command, position)
        if not match:
            return found
        line_start = command.rfind("\n", 0, match.start()) + 1
        line_end = command.find("\n", match.end())
        if line_end < 0:
            return found
        tag = match.group(2)
        body, consumed = [], 0
        for line in command[line_end + 1:].split("\n"):
            consumed += len(line) + 1
            if line.strip() == tag:
                break
            body.append(line)
        found.append((command[line_start:match.start()], command[match.end():line_end], "\n".join(body)))
        position = line_end + 1 + consumed


def heredoc_target(before, after):
    """cat > 文件 <<EOF、tee [-a] 文件 <<EOF、cat <<EOF > 文件 三种写法的目标文件；认不出回 None。"""
    for text, pattern in ((before, r"(?:>>?|\btee\b(?:\s+-\w+)*)\s*(\S+)\s*$"), (after, r"^\s*>>?\s*(\S+)")):
        match = re.search(pattern, text)
        if match:
            return match.group(1).strip("'\"")
    return None


def python_literal_lines(code):
    """python 代码里写文件的那一类：字符串字面量里的行（replace 的旧串里已有的行除外）；没有写文件的动作、或没有指向仓内文件的路径，回空。"""
    if not WRITE_HINT.search(code):
        return []
    try:
        with warnings.catch_warnings():
            warnings.simplefilter("ignore")   # 命令里嵌的 python 常带 shell 转义，ast 的 SyntaxWarning 不是要报的东西
            tree = ast.parse(code)
    except (SyntaxError, ValueError):
        return []
    constants, old_lines, targets = [], set(), 0
    for node in ast.walk(tree):
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            constants.append(node.value)
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and node.func.attr == "replace" and node.args:
            if isinstance(node.args[0], ast.Constant) and isinstance(node.args[0].value, str):
                old_lines.update(significant_lines(node.args[0].value))
    lines = []
    for value in constants:
        if len(value) < 400 and not re.search(r"\s", value) and repository_relative(value, allow_relative=True):
            targets += 1
            continue
        lines += [line for line in significant_lines(value) if line not in old_lines]
    return lines if targets else []


def python_code_of(command, state):
    """命令里喂给 python 的代码：heredoc 正文、-c 的参数、先 heredoc 写成再执行的脚本。"""
    codes = []
    for before, _, body in heredoc_bodies(command):
        if re.search(r"\bpython3?\b", before):
            codes.append(body)
    for match in re.finditer(r"\bpython3?\s+(?:-[A-Za-z]\s+)*-c\s", command):
        words = shell_words(command[match.end() - 3:])
        if len(words) >= 2 and words[0] == "-c":
            codes.append(words[1])
    for match in re.finditer(r"\bpython3?\s+(?:-[A-Za-z]\s+)*(\S+\.py)\b", command):
        if match.group(1) in state["files"]:
            codes.append(state["files"][match.group(1)])
    return codes


def script_writes(command, state):
    """replace-once.py / replace-batch.py / insert-row.py 的实写：[(目标文件, 新增行)]。"""
    writes = []
    for match in SCRIPT_CALL.finditer(command):
        name, flags, arguments, values = match.group(1), set(), [], {}
        words, position = shell_words(command[match.end():]), 0
        while position < len(words):
            word = words[position]
            position += 1
            if OPERATOR_WORD.match(word) or re.fullmatch(r"python3?", word):
                break   # 下一条命令开始了
            if name == "insert-row" and word in INSERT_ROW_VALUE_FLAGS:
                values.setdefault(word, words[position] if position < len(words) else "")
                position += 1
            elif word.startswith("--"):
                flags.add(word)
            else:
                arguments.append(word)
        if name == "replace-once" and len(arguments) >= 3:
            old_lines = set(significant_lines(arguments[1]))
            writes.append((arguments[0], [line for line in significant_lines(arguments[2]) if line not in old_lines]))
        elif name == "insert-row" and "--dry-run" not in flags and arguments and "--line" in values:
            writes.append((arguments[0], significant_lines(values["--line"])))
        elif name == "replace-batch" and "--dry-run" not in flags and arguments:
            spec_text = state["files"].get(arguments[0])
            if spec_text is None:
                try:
                    with open(os.path.expanduser(arguments[0]), errors="replace") as handle:
                        spec_text = handle.read(5_000_000)
                except OSError:
                    spec_text = ""
            try:
                spec = json.loads(spec_text)
            except ValueError:
                continue
            for item in spec if isinstance(spec, list) else []:
                if isinstance(item, dict):
                    old_lines = set(significant_lines(item.get("old")))
                    writes.append((item.get("file"), [line for line in significant_lines(item.get("new")) if line not in old_lines]))
    return writes


def bash_written_lines(command, state):
    """一条成功的 Bash 命令写进仓内文件的新增行：(确定写进去的行, 只作加分的行)。heredoc 写文件和三个定点替换脚本的新串是确定写进去的；
    python 代码里的字符串字面量只是写了什么的上限（模板、命令、说明文字也在里面），只在仓里找得到时才算，找不到不扣分。
    state["files"] 记这个子 agent 用 heredoc 写出过的文件（后面执行脚本、读规格要用）。"""
    if BROKEN == "no-bash-writes":
        return [], []
    lines, credit = [], []
    for before, after, body in heredoc_bodies(command):
        target = heredoc_target(before, after)
        if target:
            state["files"][target] = body
            if repository_relative(target, allow_relative=True):
                lines += significant_lines(body)
    for code in python_code_of(command, state):
        credit += python_literal_lines(code)
    for target, new_lines in script_writes(command, state):
        if repository_relative(target, allow_relative=True):
            lines += new_lines
    return lines, credit


def written_lines(uses, errors):
    """一个子 agent 全部成功的写的新增行：(确定写进去的行, 只作加分的行)；同一行写几次算几次，与 usage 的逐次口径一致。"""
    state, lines, credit = {"files": {}}, [], []
    for tool_id, (name, tool_input) in uses.items():
        if tool_id in errors:
            continue
        if name == "Edit":
            if repository_relative(tool_input.get("file_path")):
                old_lines = set(line.strip() for line in (tool_input.get("old_string") or "").splitlines())
                lines += [line for line in significant_lines(tool_input.get("new_string")) if line not in old_lines]
        elif name == "MultiEdit":
            if repository_relative(tool_input.get("file_path")):
                for edit in tool_input.get("edits") or []:
                    old_lines = set(line.strip() for line in (edit.get("old_string") or "").splitlines())
                    lines += [line for line in significant_lines(edit.get("new_string")) if line not in old_lines]
        elif name == "Write":
            if repository_relative(tool_input.get("file_path")):
                lines += significant_lines(tool_input.get("content"))
        elif name == "Bash":
            strict, extra = bash_written_lines(tool_input.get("command") or "", state)
            lines += strict
            credit += extra
    return lines, credit


def event_text(name, tool_input):
    if name == "Bash":
        text = tool_input.get("command") or ""
    elif name in ("Read", "Grep", "Glob"):
        text = "\n".join(str(tool_input.get(key) or "") for key in ("file_path", "path", "pattern"))
    elif name in ("Edit", "Write"):
        text = (tool_input.get("file_path") or "") + "\n" + (tool_input.get("new_string") or tool_input.get("content") or "")
    elif name == "MultiEdit":
        text = (tool_input.get("file_path") or "") + "\n" + "\n".join(edit.get("new_string") or "" for edit in tool_input.get("edits") or [])
    elif name in ("Agent", "Task"):
        text = tool_input.get("prompt") or ""
    else:
        text = json.dumps(tool_input, ensure_ascii=False)
    return text[:EVENT_TEXT_LIMIT]


def raw_timestamp(line):
    index = line.rfind('"timestamp":"')
    return line[index + 13:index + 13 + 40].split('"', 1)[0] if index >= 0 else None


def draft_directory_of(prompt):
    """派发提示点名的草稿目录（照写的样子，没展开）：行首「草稿目录：」那几行里的第一个，没有交 None。认法只在 .claude/hooks/lib_prompt_scratch_line.py 的 scratch_line_values。
    弄坏开关 draft-anywhere-in-line 下退回改前：「草稿目录：」在一行里哪都认，取第一个。"""
    if BROKEN == "draft-anywhere-in-line":
        found = re.search(r"草稿目录[：:]\s*(" + PATH_CHARS + "+)", prompt or "")
        return found.group(1) if found else None
    values = prompt_scratch_line.scratch_line_values(prompt or "")
    return values[0] if values else None


def scan_subagent(path):
    """一个子 agent 的会话记录：开始与结束、交回正文、最后一句、花费、写入的新增行、工具调用的文本（供引用比对）。
    只解析 assistant 行、报错的 user 行、开头的 user 行与交回快筛留下的行。起点（首条记录的时刻）与交回（最近一次送达的那一次的正文）照 subagent_transcript.py。"""
    calls, uses, errors, events = {}, {}, set(), []
    end_raw = prompt = None
    last_text = ""
    handbacks = transcripts.HandbackTracker()
    with open(path, errors="replace") as handle:
        for number, line in enumerate(handle):
            raw = raw_timestamp(line)
            if raw:
                end_raw = raw
            is_assistant = '"type":"assistant"' in line
            is_handback_line = handbacks.wants(line)
            if not is_assistant and not is_handback_line and not (number == 0 or '"is_error":true' in line or (prompt is None and '"type":"user"' in line)):
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            if is_handback_line:
                handbacks.feed(record)
            message = record.get("message") if isinstance(record.get("message"), dict) else {}
            content = message.get("content")
            if record.get("type") == "user":
                if prompt is None:
                    prompt = content if isinstance(content, str) else " ".join(block.get("text", "") for block in content or [] if isinstance(block, dict))
                for block in content if isinstance(content, list) else []:
                    if isinstance(block, dict) and block.get("type") == "tool_result" and block.get("is_error"):
                        errors.add(block.get("tool_use_id"))
                continue
            if not is_assistant:
                continue
            if message.get("usage"):
                calls[usage_key(message, record)] = message["usage"]
            stamp = None
            for block in content if isinstance(content, list) else []:
                if not isinstance(block, dict):
                    continue
                if block.get("type") == "text":
                    last_text = block.get("text") or last_text
                elif block.get("type") == "tool_use":
                    name, tool_input = block.get("name"), block.get("input") or {}
                    if name == transcripts.HANDBACK_TOOL:   # 送没送达、正文是哪一次的，由上面的 handbacks 认
                        continue
                    if block.get("id") not in uses and name in ("Edit", "Write", "MultiEdit", "Bash"):
                        uses[block.get("id")] = (name, tool_input)
                    if name in EVENT_TOOLS:
                        if stamp is None:
                            moment = moment_of(raw or "")
                            stamp = moment.timestamp() if moment else 0.0
                        tokens = event_tokens(event_text(name, tool_input))
                        if tokens:
                            events.append((stamp, False, tokens))
    written = written_lines(uses, errors)
    draft = draft_directory_of(prompt)
    handback = handbacks.delivered["message"] if handbacks.delivered else None
    return {"start": transcripts.first_record_moment(path), "end": moment_of(end_raw or ""), "handback": handback, "last_text": last_text[-3000:],
            "totals": usage_totals(list(calls.values())), "lines": written[0], "credit_lines": written[1], "events": events,
            "draft": normalize_path(draft) if draft else None}


def scan_main_session(path):
    """主会话的工具调用文本（供引用比对）。"""
    events = []
    with open(path, errors="replace") as handle:
        for line in handle:
            if '"type":"assistant"' not in line or '"tool_use"' not in line:
                continue
            try:
                record = json.loads(line)
            except ValueError:
                continue
            message = record.get("message") if isinstance(record.get("message"), dict) else {}
            moment = moment_of(record.get("timestamp") or "")
            for block in message.get("content") if isinstance(message.get("content"), list) else []:
                if isinstance(block, dict) and block.get("type") == "tool_use" and block.get("name") in EVENT_TOOLS and moment:
                    tokens = event_tokens(event_text(block["name"], block.get("input") or {}))
                    if tokens:
                        events.append((moment.timestamp(), True, tokens))
    return events


def normalize_path(path):
    path = re.sub(r"^/home/[^/]+/", "~/", path).replace("$HOME/", "~/").replace("${HOME}/", "~/")
    path = re.sub(r"(\.[A-Za-z0-9]{1,6})[一-鿿].*$", r"\1", path)   # 扩展名后面直接粘着的说明文字
    return path.rstrip(".,=/")


def handback_paths(text):
    """交回正文里点名的产物路径：取到空白、引号、括号、全角标点为止，扩展名后面直接粘着的汉字说明也去掉；弄坏开关退回只在标点处截断。"""
    pattern = OLD_WORK_PATH if BROKEN == "no-path-normalize" else WORK_PATH
    return sorted(set(normalize_path(match.group(0)) for match in pattern.finditer(text or "")))


def path_directory_name(path):
    """路径所在的那个具名目录：工作根（或 /tmp/claude-N/）底下直接的一层，且路径恰好是这一层目录本身或它里面的一个文件；更深的返回 None
    （同一个父目录底下分给几个子 agent 各一个子目录时，父目录名是共用的，认不出是谁的）。"""
    parts = path.split("/")
    if path.startswith("~/.local/share/singlefs/work/") and 6 <= len(parts) <= 7:
        return parts[5]
    if path.startswith("/tmp/claude-") and 4 <= len(parts) <= 5:
        return parts[3]
    return None


def expand_variables(text):
    """同一条命令里赋过值的变量先展开（$NAME、${NAME}，$HOME 记成 ~）；没赋过的原样留着，当通配对。"""
    values = {"HOME": "~"}
    for match in VARIABLE_ASSIGNMENT.finditer(text):
        values[match.group(1)] = match.group(2).strip("\"'")
    for _ in range(3):
        expanded = VARIABLE_REFERENCE.sub(lambda match: values.get(match.group(1) or match.group(2), match.group(0)), text)
        if expanded == text:
            break
        text = expanded
    return text


def wildcard_regex(component):
    """路径的一段里带变量、星号、问号、花括号的，转成正则；字面字符不足 4 个的（什么都能对上）回 None。"""
    pieces, literal, index = [], 0, 0
    while index < len(component):
        char = component[index]
        if char == "$":
            match = re.match(r"\$(?:\{[^}]*\}|[A-Za-z_]\w*)", component[index:])
            if match:
                pieces.append(".+")
                index += match.end()
                continue
        if char == "*":
            pieces.append(".*")
        elif char == "?":
            pieces.append(".")
        elif char == "{":
            end = component.find("}", index)
            if end < 0:
                return None
            body = component[index + 1:end]
            if "," in body:
                alternatives = body.split(",")
                pieces.append("(?:" + "|".join(re.escape(alternative) for alternative in alternatives) + ")")
                literal += min(len(alternative) for alternative in alternatives)
            else:
                pieces.append(".+")
            index = end + 1
            continue
        else:
            pieces.append(re.escape(char))
            literal += 1
        index += 1
    return re.compile("".join(pieces)) if literal >= 4 else None


def event_tokens(text):
    """一条工具调用文本里和引用有关的记号，各带一个字符的标签：i 是 agent 号，p 是工作目录一类的路径（连同各级上层目录），
    n 是目录名一类的路径段（带连字符、不像文件名），w 是带变量、星号、问号、花括号的这样一段。
    弄坏开关 no-path-normalize 时只剩 agent 号与旧写法认出的路径，变量不展开、路径段不取。"""
    normalize = BROKEN != "no-path-normalize"
    tokens = set()
    if normalize and "$" in text:
        text = expand_variables(text)
    tokens.update("i:" + match.group(0) for match in AGENT_ID.finditer(text))
    for match in (WORK_PATH if normalize else OLD_WORK_PATH).finditer(text):
        parts = normalize_path(match.group(0)).split("/")
        tokens.update("p:" + "/".join(parts[:length]) for length in range(len(parts), 2, -1))
    if normalize:
        for token in PATH_TOKEN.findall(text):
            for component in token.split("/"):
                component = component.rstrip(".,:;")
                if len(component) < NAME_MINIMUM or "-" not in component or FILE_LIKE.search(component):
                    continue
                tokens.add(("w:" if WILDCARD_CHARS.search(component) else "n:") + component)
    return tokens


def index_references(events, wanted_names, wanted_paths, wanted_ids):
    """所有工具调用的记号里，点到哪些目录名、路径、agent 号：{键: (主会话的时刻们, 子 agent 的时刻们)}，时刻升序。"""
    hits = collections.defaultdict(lambda: ([], []))
    wildcard_names = {}
    for stamp, is_main, tokens in events:
        keys = set()
        for token in tokens:
            tag, body = token[0], token[2:]
            if tag == "i" and body in wanted_ids:
                keys.add(("id", body))
            elif tag == "p" and body in wanted_paths:
                keys.add(("path", body))
            elif tag == "n" and body in wanted_names:
                keys.add(("name", body))
            elif tag == "w":
                if body not in wildcard_names:
                    pattern = wildcard_regex(body)
                    wildcard_names[body] = [name for name in wanted_names if pattern.fullmatch(name)] if pattern else []
                keys.update(("name", name) for name in wildcard_names[body])
        for key in keys:
            hits[key][0 if is_main else 1].append(stamp)
    for main_stamps, sub_stamps in hits.values():
        main_stamps.sort()
        sub_stamps.sort()
    return hits


def referenced_between(stamps, begin, limit):
    index = bisect.bisect_right(stamps, begin)
    return index < len(stamps) and stamps[index] < limit


def git_output(arguments, environment):
    result = subprocess.run(["git", *arguments], capture_output=True, env=environment)
    if result.returncode != 0:
        raise RuntimeError(f"git {' '.join(arguments[:3])} 退出 {result.returncode}：{result.stderr.decode('utf-8', 'replace').strip()[:300]}")
    return result.stdout


SHARED = {}   # 要比对的行（建在 fork 工作进程之前，子进程继承同一份，哈希种子也一致）


def line_set(lines):
    """要比对的行：按 utf-8 字节查（快路径），和按字符串哈希查（行首尾是非 ASCII 空白时的慢路径）。"""
    by_bytes = {line.encode("utf-8"): hash(line) for line in lines}
    return by_bytes, set(by_bytes.values())


def matching_hash(stripped):
    """一行（字节、已去首尾 ASCII 空白）若是要比对的行之一，回它的哈希，否则 None。"""
    if len(stripped) < LINE_MINIMUM:
        return None
    by_bytes, hashes = SHARED["lines"]
    found = by_bytes.get(stripped)
    if found is not None:
        return found
    if stripped[0] >= 0x80 or stripped[-1] >= 0x80:
        line = stripped.decode("utf-8", "replace").strip()
        if len(line) >= LINE_MINIMUM and hash(line) in hashes:
            return hash(line)
    return None


def file_matches(tree, names):
    """一棵工作树里这几个文件的行里，要比对的有哪些（二进制和超大的跳过）。"""
    found = set()
    for name in names:
        path = os.path.join(tree, name.decode("utf-8", "replace"))
        try:
            if os.path.getsize(path) > 200_000_000:
                continue
            with open(path, "rb") as handle:
                if b"\0" in handle.read(4096):
                    continue
                handle.seek(0)
                for raw in handle:
                    match = matching_hash(raw.strip())
                    if match is not None:
                        found.add(match)
        except OSError:
            continue
    return found


def history_and_worktree_lines(repository, since, lines, workers):
    """lines 里哪些在 git 历史加进的行或某棵工作树的文件里出现过：返回（找到的行的哈希集合, 工作树个数）。git 出错直接抛。"""
    environment = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0")
    SHARED["lines"] = line_set(lines)
    found = set()
    begin = (since - datetime.timedelta(days=HISTORY_MARGIN_DAYS)).date().isoformat()
    listing = git_output(["-C", repository, "worktree", "list", "--porcelain"], environment).decode("utf-8", "replace")
    trees = [line[len("worktree "):] for line in listing.splitlines() if line.startswith("worktree ")]
    chunks = []
    for tree in trees:
        if os.path.isdir(tree):
            names = [name for name in git_output(["-C", tree, "ls-files", "-co", "--exclude-standard", "-z"], environment).split(b"\0") if name]
            chunks += [(tree, names[start:start + 300]) for start in range(0, len(names), 300)]
    pool = concurrent.futures.ProcessPoolExecutor(max_workers=workers, mp_context=multiprocessing.get_context("fork")) if workers > 1 else None
    try:
        futures = [pool.submit(file_matches, tree, names) for tree, names in chunks] if pool else []
        with tempfile.TemporaryFile() as log_errors:
            log = subprocess.Popen(["git", "-C", repository, "log", "-p", "-U0", "--format=", "--no-color", "--no-ext-diff", "--no-textconv",
                                    f"--since={begin}", "--branches", "HEAD"], stdout=subprocess.PIPE, stderr=log_errors, env=environment)
            for raw in log.stdout:
                if raw[:1] == b"+" and raw[:3] != b"+++":
                    match = matching_hash(raw[1:].strip())
                    if match is not None:
                        found.add(match)
            if log.wait() != 0:
                log_errors.seek(0)
                raise RuntimeError(f"git log 退出 {log.returncode}：{log_errors.read().decode('utf-8', 'replace').strip()[:300]}")
        for future in futures:
            found |= future.result()
        if not pool:
            for tree, names in chunks:
                found |= file_matches(tree, names)
    finally:
        if pool:
            pool.shutdown()
    return found, len(trees)


def cut_reason(last_text):
    tail = last_text[-600:]
    if USAGE_LIMIT_TAIL.search(tail):
        return "用量撞顶"
    if API_ERROR_TAIL.search(tail):
        return "API 错"
    return "停掉"


PROMPTS_DIRECTORY = "research/prompts"
LEG_OUTPUT_MENTION = re.compile(r"research/prompts/([^/\s`'\"，。；：、（）()「」<>*]+)-output\.md")   # 交回里点名的三方腿报告（不进子目录）


def prompts_paths(repository):
    """research/prompts/ 下在工作树（git worktree list 列出的每一棵，已跟踪与未跟踪未忽略的）或 git 历史（所有分支加 HEAD，删过的也算）里出现过的路径。
    git 出错直接抛。弄坏开关 three-way-history-unread 下不读 git 历史。"""
    environment = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1", GIT_TERMINAL_PROMPT="0")
    found = set()
    listing = git_output(["-C", repository, "worktree", "list", "--porcelain"], environment).decode("utf-8", "replace")
    for tree in [line[len("worktree "):] for line in listing.splitlines() if line.startswith("worktree ")]:
        if os.path.isdir(tree):
            names = git_output(["-C", tree, "ls-files", "-co", "--exclude-standard", "-z", "--", PROMPTS_DIRECTORY], environment).split(b"\0")
            found |= {name.decode("utf-8", "replace") for name in names if name}
    if BROKEN != "three-way-history-unread":
        names = git_output(["-C", repository, "-c", "core.quotePath=false", "log", "--branches", "HEAD", "--no-renames", "--name-only", "--format=",
                            "--", PROMPTS_DIRECTORY], environment).decode("utf-8", "replace")
        found |= {name.strip() for name in names.splitlines() if name.strip()}
    return found


def leg_output_verdicts(handback_text, known):
    """交回里点名的 research/prompts/<名>-output.md 里，在 known（prompts_paths）里、同轮判决也在的：[(报告, 判决)]。
    轮名取 <名> 去掉开头的 _ 之后、按连字符从长到短的前缀里第一个有 research/prompts/<前缀>-main-verification.md 的。"""
    confirmed = []
    for stem in sorted(set(LEG_OUTPUT_MENTION.findall(handback_text or ""))):
        output = f"{PROMPTS_DIRECTORY}/{stem}-output.md"
        if output not in known:
            continue
        parts = stem.lstrip("_").split("-")
        for length in range(len(parts) - 1, 0, -1):
            verdict = f"{PROMPTS_DIRECTORY}/{'-'.join(parts[:length])}-main-verification.md"
            if verdict in known:
                confirmed.append((output, verdict))
                break
    return confirmed


def three_way_leg_types_of(repository):
    """outcome 两个三方腿桶收的类型：--repo 那棵树的 .claude/agents 里定义 frontmatter 写了 leg: three-way 的（lib_three_way_legs.py 的 three_way_leg_types）。
    弄坏开关 leg-types-by-prefix 下退回改前：类型名以 three-way 开头的都算（这里交那棵树里以 three-way 开头的定义名，给自证比集合用）。"""
    if BROKEN == "leg-types-by-prefix":
        return frozenset(os.path.basename(path)[: -len(".md")] for path in glob.glob(os.path.join(repository, ".claude", "agents", "three-way*.md")))
    return three_way_legs.three_way_leg_types(repository)


def is_three_way_leg(agent_type, leg_types):
    """这个子 agent 的类型进不进三方腿的两个桶；弄坏开关 leg-types-by-prefix 下按类型名 three-way 开头认（改前的判法）。"""
    return agent_type.startswith("three-way") if BROKEN == "leg-types-by-prefix" else agent_type in leg_types


def outcome_bucket(agent, used_main, used_any, now):
    if agent["handed_back"]:
        if agent.get("three_way_leg"):
            if BROKEN == "three-way-by-type" or agent.get("leg_verdicts"):
                return "verdict-archived"
            return "three-way-report-or-verdict-missing"
        if agent["written"] >= WRITTEN_MINIMUM:
            share = agent["landed"] / agent["written"]
            if share >= LANDED_AT:
                return "written-landed"
            if share >= PARTIAL_AT:
                return "written-partial"
            return "written-overwritten" if used_main else "written-unused"
        if used_main:
            return "report-used-by-main"
        return "report-used-by-later-sub" if used_any else "report-unreferenced"
    if agent["end"] and (now - agent["end"]).total_seconds() < IN_FLIGHT_MINUTES * 60:
        return "in-flight"
    if agent["cut"] == "用量撞顶":
        return "cut-by-usage-limit"
    if agent["cut"] == "API 错":
        return "cut-by-api-error"
    return "stopped-salvaged" if used_any else "stopped-abandoned"


def scan_job(kind, path):
    return scan_subagent(path) if kind == "sub" else scan_main_session(path)


def collect_outcome(root, since, repository, now, workers=1, until=None):
    """起点（首条记录的时刻）落在 [since, until) 里的每个子 agent 一行（until 为 None 不设上界）：
    [{agent, type, description, start, handed_back, cut, bucket, cost, rework, ...}]，与 git 历史、各工作树比对过。
    会话记录按文件分给 workers 个工作进程读（workers 为 1 时在本进程里顺序读）。"""
    subagents = []
    for meta in sorted(glob.glob(os.path.join(root, "*", "subagents", "agent-*.meta.json"))):
        transcript = meta.replace(".meta.json", ".jsonl")
        if os.path.exists(transcript) and transcripts.started_between(transcript, since, until):
            subagents.append((meta, transcript))
    sessions = [path for path in sorted(glob.glob(os.path.join(root, "*.jsonl"))) if recent(path, since)]
    jobs = [("sub", transcript) for _, transcript in subagents] + [("main", path) for path in sessions]
    if workers > 1 and len(jobs) > 1:
        with concurrent.futures.ProcessPoolExecutor(max_workers=workers, mp_context=multiprocessing.get_context("fork")) as pool:
            scans = [future.result() for future in [pool.submit(scan_job, *job) for job in jobs]]
    else:
        scans = [scan_job(*job) for job in jobs]
    agents, events = [], []
    for (meta, transcript), scan in zip(subagents, scans):
        events += scan["events"]
        try:
            with open(meta) as handle:
                described = json.load(handle)
        except (OSError, ValueError):
            described = {}
        handback_text = scan["handback"]
        paths = handback_paths(handback_text if handback_text else scan["last_text"])
        names = {name for name in [scan["draft"].split("/")[-1] if scan["draft"] else None] + [path_directory_name(path) for path in paths]
                 if name and len(name) >= NAME_MINIMUM and "-" in name and name not in GENERIC_NAMES}
        if scan["draft"]:
            paths = sorted(set(paths) | {scan["draft"]})
        description = re.sub(r"\s+", " ", described.get("description") or "")
        agents.append({"agent": os.path.basename(transcript)[6:-6], "type": described.get("agentType") or "?", "description": description,
                       "start": scan["start"], "end": scan["end"], "handed_back": handback_text is not None,
                       "cut": "-" if handback_text is not None else cut_reason(scan["last_text"]),
                       "cost": folded(scan["totals"]), "rework": bool(rework_words().search(description)),
                       "lines": scan["lines"], "credit_lines": scan["credit_lines"], "written": len(scan["lines"]), "landed": 0, "paths": paths, "names": names,
                       "handback_text": handback_text})
    for scan in scans[len(subagents):]:
        events += scan
    leg_types = three_way_leg_types_of(repository)
    if not leg_types and os.path.isdir(os.path.join(repository, ".claude", "agents")):
        print(f"  ! {repository}/.claude/agents 里一份 frontmatter 写了 leg: three-way 的定义都没有：这一次没有子 agent 进三方腿的两个桶", file=sys.stderr)
    for agent in agents:
        agent["three_way_leg"] = is_three_way_leg(agent["type"], leg_types)
    legs = [agent for agent in agents if agent["handed_back"] and agent["three_way_leg"]]
    if legs and BROKEN != "three-way-by-type":   # 三方腿：交回点名的报告在不在仓、同轮有没有判决
        known = prompts_paths(repository)
        for agent in legs:
            agent["leg_verdicts"] = leg_output_verdicts(agent["handback_text"], known)
    if any(agent["lines"] or agent["credit_lines"] for agent in agents):
        found, _ = history_and_worktree_lines(repository, since, {line for agent in agents for line in agent["lines"] + agent["credit_lines"]}, workers)
        for agent in agents:
            credited = sum(1 for line in agent["credit_lines"] if hash(line) in found)
            agent["landed"] = sum(1 for line in agent["lines"] if hash(line) in found) + credited
            agent["written"] = len(agent["lines"]) + credited
    directory_owners = collections.Counter(directory for agent in agents for directory in {path.rsplit("/", 1)[0] for path in agent["paths"] if path.count("/") >= 6})
    for agent in agents:   # 只有这一个子 agent 的产物在里面的目录，点到目录本身也算点到它
        agent["paths"] = sorted(set(agent["paths"]) | {path.rsplit("/", 1)[0] for path in agent["paths"] if path.count("/") >= 6 and directory_owners[path.rsplit("/", 1)[0]] == 1})
    wanted_names = {name for agent in agents for name in agent["names"]}
    wanted_paths = {path for agent in agents for path in agent["paths"]}
    wanted_ids = {agent["agent"] for agent in agents}
    hits = index_references(events, wanted_names, wanted_paths, wanted_ids)
    keys_of = lambda agent: [("id", agent["agent"])] + [("name", name) for name in agent["names"]] + [("path", path) for path in agent["paths"]]
    starts = collections.defaultdict(list)
    for agent in agents:
        for key in keys_of(agent):
            starts[key].append(agent["start"].timestamp())
    for agent in agents:
        used_main = used_sub = False
        begin = agent["start"].timestamp()
        end = agent["end"].timestamp() if agent["end"] else begin
        for key in keys_of(agent):
            later = [stamp for stamp in starts[key] if stamp > begin] if key[0] == "name" else []
            limit = min(later) if later else float("inf")   # 目录名是共用的（重派沿用同一个草稿目录）：下一个共用者开工之后，按名字点到的算不清是谁的
            main_stamps, sub_stamps = hits[key] if key in hits else ([], [])
            used_main = used_main or referenced_between(main_stamps, end, limit)
            used_sub = used_sub or referenced_between(sub_stamps, end, limit)
        agent["bucket"] = outcome_bucket(agent, used_main, used_main or used_sub, now)
        if agent["bucket"] == "in-flight":
            agent["cut"] = "还在跑"
    agents.sort(key=lambda agent: (agent["start"], agent["agent"]))
    return agents


def outcome_summary(agents):
    """汇总：按桶、按类型 × 桶、返工字样、没交回的按截断原因。"""
    total_cost = sum(agent["cost"] for agent in agents) or 1
    by_bucket = collections.defaultdict(lambda: [0, 0.0])
    by_type_bucket = collections.defaultdict(lambda: [0, 0.0])
    for agent in agents:
        for table, key in ((by_bucket, agent["bucket"]), (by_type_bucket, (agent["type"], agent["bucket"]))):
            table[key][0] += 1
            table[key][1] += agent["cost"]
    rework = [agent for agent in agents if agent["rework"]]
    cuts = collections.Counter(agent["cut"] for agent in agents if not agent["handed_back"])
    return {"total_cost": total_cost, "by_bucket": by_bucket, "by_type_bucket": by_type_bucket, "rework": (len(rework), sum(agent["cost"] for agent in rework)), "cuts": cuts}


def window_text(since, until_day=None):
    """账的窗口：「<since> 起」，给了 --until 再接「、到 <until> 为止」（until_day 是 --until 写的那一天，含当天）。"""
    return f"{since.date()} 起" + (f"、到 {until_day} 为止" if until_day else "")


def outcome_lines(agents, since, show_rows, until_day=None):
    lines = []
    if show_rows:
        lines.append(f"## 每个子 agent 一行（{window_text(since, until_day)}，{len(agents)} 个；折算花费单位百万）")
        lines.append("agent\t类型\t开始日期\t交回\t截断原因\t结局桶\t折算花费\t返工字样\t描述")
        for agent in agents:
            lines.append("\t".join([agent["agent"], agent["type"], local_day(agent["start"]).isoformat(), "交回" if agent["handed_back"] else "没交回", agent["cut"], agent["bucket"],
                                    f"{agent['cost'] / 1e6:.2f}", "返工字样" if agent["rework"] else "-", agent["description"][:80]]))
        lines.append("")
    summary = outcome_summary(agents)
    total_cost = summary["total_cost"]
    lines.append(f"## 按结局桶（{window_text(since, until_day)}，{len(agents)} 个，折算合计 {total_cost / 1e6:.2f} 百万）")
    width = max(len(bucket) for bucket, _ in BUCKET_GLOSS) + 2
    lines.append(f"{'桶':{width}}{'个数':>6}{'花费占比':>9}  说明")
    for bucket, gloss in BUCKET_GLOSS:
        count, cost = summary["by_bucket"].get(bucket, [0, 0.0])
        lines.append(f"{bucket:{width}}{count:>6}{cost / total_cost:>9.1%}  {gloss}")
    lines.append("")
    lines.append("## 并成调度记录「按结局分」那张表的五行")
    lines.append(f"{'行':44}{'个数':>6}{'花费占比':>9}")
    covered = set()
    for label, buckets in RECORD_TABLE_ROWS:
        covered.update(buckets)
        count = sum(summary["by_bucket"].get(bucket, [0, 0.0])[0] for bucket in buckets)
        cost = sum(summary["by_bucket"].get(bucket, [0, 0.0])[1] for bucket in buckets)
        lines.append(f"{label:40}{count:>6}{cost / total_cost:>9.1%}")
    rest = [bucket for bucket, _ in BUCKET_GLOSS if bucket not in covered]
    count = sum(summary["by_bucket"].get(bucket, [0, 0.0])[0] for bucket in rest)
    cost = sum(summary["by_bucket"].get(bucket, [0, 0.0])[1] for bucket in rest)
    lines.append(f"{'其余（' + '、'.join(rest) + '）':40}{count:>6}{cost / total_cost:>9.1%}")
    lines.append("")
    lines.append("## 按类型 × 结局桶（个数，花费占全部的比例）")
    for (agent_type, bucket), (count, cost) in sorted(summary["by_type_bucket"].items(), key=lambda item: (item[0][0], -item[1][0])):
        lines.append(f"  {agent_type:26}{bucket:{width}}{count:>6}{cost / total_cost:>9.1%}")
    lines.append("")
    rework_count, rework_cost = summary["rework"]
    lines.append(f"## 描述带返工字样（REWORK_WORDS，词表在文件头）的派发：{rework_count} 个，占个数 {rework_count / max(1, len(agents)):.1%}，占花费 {rework_cost / total_cost:.1%}")
    cuts = summary["cuts"]
    lines.append(f"## 没交回的 {sum(cuts.values())} 个，按截断原因：" + "、".join(f"{reason} {cuts[reason]}" for reason in ("用量撞顶", "API 错", "停掉", "还在跑") if cuts.get(reason)))
    return lines


def write_json_lines(path, items):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        for item in items:
            handle.write(json.dumps(item, ensure_ascii=False, separators=(",", ":")) + "\n")   # 与 Claude Code 写的会话记录同一种紧凑写法：outcome 按子串先筛行


def build_outcome_fixture(work):
    """造一个假会话目录与一个带历史、另挂了一棵工作树的假仓；每个结局桶至少一个答案已知的子 agent。返回 (会话目录, 仓, 此刻, {agent 号: 预期的桶})。"""
    root = os.path.join(work, "projects")
    session = os.path.join(root, "s2")
    os.makedirs(os.path.join(session, "subagents"))
    repository, linked_tree = os.path.join(work, "repository"), os.path.join(work, "linked-tree")
    home = "/home/tester/.local/share/singlefs/work"
    base = datetime.datetime(2026, 10, 5, 0, 0, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
    stamp = lambda minutes: (base + datetime.timedelta(minutes=minutes)).isoformat()
    counter = [0]

    def numbered(label, count):
        return [f"let {label}_{number} = compute_{label}({number});" for number in range(count)]

    def assistant(minutes, blocks, usage=None):
        counter[0] += 1
        message = {"id": f"msg-{counter[0]}", "role": "assistant", "content": blocks}
        if usage:
            message["usage"] = usage
        return {"type": "assistant", "uuid": f"uuid-{counter[0]}", "timestamp": stamp(minutes), "message": message}

    def use(tool_id, name, tool_input):
        return {"type": "tool_use", "id": tool_id, "name": name, "input": tool_input}

    def agent_records(start, steps, handback=None, last_text="做到一半", prompt="派发提示", failed=()):
        """steps: [(分钟, [块…])]；failed 里的 tool_use id 记一条报错的 tool_result。用量放在末条，每个子 agent 折算 1000 + 5 × 100 = 1500。"""
        usage = {"input_tokens": 1000, "output_tokens": 100, "cache_read_input_tokens": 0, "cache_creation_input_tokens": 0}
        out = [{"type": "user", "timestamp": stamp(start), "message": {"role": "user", "content": prompt}}]
        for minutes, blocks in steps:
            out.append(assistant(minutes, blocks))
        for tool_id in failed:
            out.append({"type": "user", "timestamp": stamp(start + 1), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": tool_id, "is_error": True, "content": "failed"}]}})
        end = (steps[-1][0] if steps else start) + 1
        if handback is not None:   # 交回要送达才算（subagent_transcript.py）：调用之后跟一条成功的 tool_result
            call_id = f"handback-{counter[0]}"
            out.append(assistant(end, [use(call_id, "SubagentHandback", {"message": handback})], usage))
            out.append({"type": "user", "timestamp": stamp(end), "message": {"role": "user", "content": [
                {"type": "tool_result", "tool_use_id": call_id, "content": [{"type": "text", "text": "{\"success\":true,\"message\":\"Report delivered to your caller.\"}"}]}]}})
        else:
            out.append(assistant(end, [{"type": "text", "text": last_text}], usage))
        return out

    identifiers = {}

    def add_agent(number, agent_type, description, records_list):
        identifier = f"a{number:016x}"
        write_json_lines(os.path.join(session, "subagents", f"agent-{identifier}.jsonl"), records_list)
        with open(os.path.join(session, "subagents", f"agent-{identifier}.meta.json"), "w", encoding="utf-8") as handle:
            json.dump({"agentType": agent_type, "description": description}, handle)
        identifiers[number] = identifier

    spec_lines, tool_lines, dry_lines = numbered("batch", 3), numbered("once", 4), numbered("dryrun", 8)
    insert_lines = [f"insert the row number {number} here now" for number in range(3)]
    git_environment = dict(os.environ, GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1", GIT_AUTHOR_NAME="t", GIT_AUTHOR_EMAIL="t@example.invalid",
                           GIT_COMMITTER_NAME="t", GIT_COMMITTER_EMAIL="t@example.invalid")

    def git(*arguments):
        subprocess.run(["git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false", *arguments], cwd=repository, env=git_environment, check=True, capture_output=True)

    def put(directory, name, lines):
        os.makedirs(os.path.join(directory, "crates"), exist_ok=True)
        with open(os.path.join(directory, "crates", name), "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")

    # 假仓：历史里加过又删掉的行、留在提交里的行、只在主工作树没提交的文件、只在挂出来的另一棵工作树里没提交的文件
    os.makedirs(repository)
    git("init", "-q")
    put(repository, "base.rs", ["fn base() {}"])
    git("add", "-A")
    git("commit", "-q", "-m", "base")
    put(repository, "history_only.rs", numbered("edit", 10))
    git("add", "-A")
    git("commit", "-q", "-m", "history only")
    git("rm", "-q", "crates/history_only.rs")
    git("commit", "-q", "-m", "remove again")
    put(repository, "tools.rs", tool_lines + insert_lines + spec_lines)
    slow_lines = [chr(0x3000) + f"全角空格开头的比对行 x{number} 用来走慢路径" for number in range(3)]   # 行首是非 ASCII 空白：字节快路径对不上，要走解码去空白的慢路径
    put(repository, "partial.rs", numbered("partial", 2) + slow_lines)
    git("add", "-A")
    git("commit", "-q", "-m", "tools and partial")
    # 三方腿的报告与判决：fx 轮报告在提交里、判决提交过又删掉（只在 git 历史里）；fy 轮报告只在主工作树没提交、没有判决；fz 轮判决只在另一棵工作树、报告哪里都没有
    def put_prompt(directory, name):
        os.makedirs(os.path.join(directory, "research", "prompts"), exist_ok=True)
        with open(os.path.join(directory, "research", "prompts", name), "w", encoding="utf-8") as handle:
            handle.write(f"# {name}\n")
    put_prompt(repository, "fx-r1-opus-output.md")
    put_prompt(repository, "fx-r1-main-verification.md")
    git("add", "-A")
    git("commit", "-q", "-m", "fx round")
    git("rm", "-q", "research/prompts/fx-r1-main-verification.md")
    git("commit", "-q", "-m", "fx verdict archived away")
    put(repository, "python_written.rs", numbered("python", 11))
    put_prompt(repository, "fy-r1-sonnet-output.md")
    git("worktree", "add", "-q", "-b", "linked", linked_tree)
    put(linked_tree, "heredoc_written.rs", numbered("heredoc", 11))
    put_prompt(linked_tree, "fz-r1-main-verification.md")
    # 三方腿类型表：假仓的定义里三种腿写 leg: three-way，材料员不写（以 three-way 起头、不是腿）
    os.makedirs(os.path.join(repository, ".claude", "agents"))
    for definition, leg_line in (("three-way-forward", "leg: three-way\n"), ("three-way-attack", "leg: three-way\n"), ("three-way-defense", "leg: three-way  # 注释\n"),
                                 ("three-way-materials", "")):
        with open(os.path.join(repository, ".claude", "agents", f"{definition}.md"), "w", encoding="utf-8") as handle:
            handle.write(f"---\nname: {definition}\n{leg_line}---\n")

    def report_handback(directory, tail="（298 行），sha256 略"):
        return f"报告：{home}/{directory}/report.md{tail}"

    def edit_to(name, lines):
        return {"file_path": f"{repository}/crates/{name}", "old_string": "", "new_string": "\n".join(lines)}

    # 写入落地：Edit 10 行在历史里（另有一次报错的 Write 不算）
    add_agent(1, "implementation-writer", "改共享代码", agent_records(0, [
        (1, [use("edit-1", "Edit", edit_to("history_only.rs", numbered("edit", 10)))]),
        (2, [use("write-1", "Write", {"file_path": f"{repository}/crates/nowhere.rs", "content": "\n".join(numbered("nowhere", 10))})])],
        handback="已改完", failed=("write-1",)))
    # Bash heredoc 写进挂出来的另一棵工作树
    heredoc_command = f"cat > {linked_tree}/crates/heredoc_written.rs <<'EOF'\n" + "\n".join(numbered("heredoc", 11)) + "\nEOF"
    add_agent(2, "implementation-writer", "Bash heredoc 写", agent_records(0, [(1, [use("bash-2", "Bash", {"command": heredoc_command})])], handback="已写"))
    # python 写文件：字符串字面量里的 11 行，在主工作树里
    python_command = ("python3 - <<'PY'\nfrom pathlib import Path\nPath(\"" + repository + "/crates/python_written.rs\").write_text(\"\"\"" + "\n".join(numbered("python", 11)) + "\n\"\"\")\nPY")
    add_agent(3, "implementation-writer", "python 写", agent_records(0, [(1, [use("bash-3", "Bash", {"command": python_command})])], handback="已写"))
    # 三个定点替换脚本：replace-once 4 行 + insert-row 3 行 + replace-batch 3 行都在历史里；--dry-run 的规格里另有 8 行没落地，算进去就掉到 partial
    spec = json.dumps([{"file": f"{repository}/crates/tools.rs", "old": "x", "new": "\n".join(spec_lines)}])
    dry_spec = json.dumps([{"file": f"{repository}/crates/tools.rs", "old": "x", "new": "\n".join(dry_lines)}])
    insert_commands = "".join(f"python3 scripts/kb/insert-row.py {repository}/crates/tools.rs --after-literal anchor --line '{line}'\n" for line in insert_lines)
    add_agent(4, "tooling-writer", "三个定点替换脚本", agent_records(0, [
        (1, [use("bash-4a", "Bash", {"command": f"python3 scripts/kb/replace-once.py {repository}/crates/tools.rs 'old text here' '" + "\n".join(tool_lines) + "'"})]),
        (2, [use("bash-4b", "Bash", {"command": insert_commands})]),
        (3, [use("bash-4c", "Bash", {"command": f"cat > /scratch/spec.json <<'EOF'\n{spec}\nEOF\npython3 scripts/kb/replace-batch.py /scratch/spec.json"})]),
        (4, [use("bash-4d", "Bash", {"command": f"cat > /scratch/dry.json <<'EOF'\n{dry_spec}\nEOF\npython3 scripts/kb/replace-batch.py --dry-run /scratch/dry.json"})])], handback="已改"))
    add_agent(5, "implementation-writer", "半截落地", agent_records(0, [(1, [use("edit-5", "Edit", edit_to("partial.rs", numbered("partial", 2) + slow_lines + numbered("gone", 5)))])], handback="已改"))
    add_agent(6, "implementation-writer", "写了但没落地且被点到", agent_records(0, [(1, [use("edit-6", "Edit", edit_to("lost.rs", numbered("lost", 10)))])],
                                                                          handback=report_handback("overwritten-draft"), prompt=f"草稿目录：{home}/overwritten-draft\n"))
    add_agent(7, "implementation-writer", "写了但没落地且没人点到", agent_records(0, [(1, [use("edit-7", "Edit", edit_to("lost2.rs", numbered("lost2", 10)))])], handback="已改"))
    add_agent(8, "three-way-forward", "三方腿", agent_records(0, [(1, [])], handback=f"报告 {repository}/research/prompts/fx-r1-opus-output.md，sha256 略"))
    # 引用：交回里的路径后面粘着全角括号与说明（主会话直接读它）；主会话用循环、花括号拼路径
    add_agent(9, "investigator", "查一件事", agent_records(0, [(1, [])], handback=f"报告：{home}/report-read-by-main.md（298 行），sha256 略"))   # 平铺的文件：没有目录键可退，路径粘着全角括号就对不上
    add_agent(10, "investigator", "循环拼路径", agent_records(0, [(1, [])], handback=report_handback("looped-draft-3", "。"), prompt=f"草稿目录：{home}/looped-draft-3\n"))
    add_agent(11, "clerk", "花括号拼路径", agent_records(0, [(1, [])], handback=report_handback("braced-draft-b", "，其余略"), prompt=f"草稿目录：{home}/braced-draft-b\n"))
    add_agent(12, "investigator", "只被子 agent 用", agent_records(0, [(1, [])], handback=report_handback("read-by-sub")))
    add_agent(13, "investigator", "重跑一次没人看", agent_records(0, [(1, [])], handback=report_handback("nobody-reads")))
    add_agent(14, "experiment-runner", "撞顶", agent_records(0, [(1, [])], last_text="You've hit your limit · resets soon"))
    add_agent(15, "experiment-runner", "接口错", agent_records(0, [(1, [])], last_text="API Error: 529 Overloaded"))
    add_agent(16, "experiment-runner", "停掉被续做", agent_records(0, [(1, [])]))
    add_agent(17, "experiment-runner", "停掉没人管", agent_records(100, [(101, [use("bash-17", "Bash", {"command": f"cat {home}/read-by-sub/report.md"})])]))
    add_agent(18, "experiment-runner", "还在跑", agent_records(300, [(301, [])]))
    add_agent(19, "three-way-attack", "三方腿同轮没有判决", agent_records(0, [(1, [])], handback="报告：research/prompts/fy-r1-sonnet-output.md（120 行）"))
    add_agent(20, "three-way-defense", "三方腿报告不在仓", agent_records(0, [(1, [])], handback="报告：research/prompts/fz-r1-opus-output.md"))
    add_agent(21, "three-way-materials", "材料员交回", agent_records(0, [(1, [])], handback="背景材料拼好了"))   # 以 three-way 起头、定义里没登记成腿：按引用分桶
    main_events = [
        assistant(50, [use("main-1", "Bash", {"command": "cat ~/.local/share/singlefs/work/report-read-by-main.md"})]),
        assistant(51, [use("main-2", "Bash", {"command": 'for i in 1 2 3; do cat "$W/looped-draft-$i/report.md"; done'})]),
        assistant(52, [use("main-3", "Bash", {"command": "ls ~/.local/share/singlefs/work/braced-draft-{b,c}/"})]),
        assistant(53, [use("main-4", "Bash", {"command": f"ls {home}/overwritten-draft"})]),
        assistant(54, [use("main-5", "SendMessage", {"to": identifiers[16], "message": "接着做"})]),
    ]
    write_json_lines(session + ".jsonl", main_events)
    expected = {1: "written-landed", 2: "written-landed", 3: "written-landed", 4: "written-landed", 5: "written-partial", 6: "written-overwritten", 7: "written-unused",
                8: "verdict-archived", 9: "report-used-by-main", 10: "report-used-by-main", 11: "report-used-by-main", 12: "report-used-by-later-sub",
                13: "report-unreferenced", 14: "cut-by-usage-limit", 15: "cut-by-api-error", 16: "stopped-salvaged", 17: "stopped-abandoned", 18: "in-flight",
                19: "three-way-report-or-verdict-missing", 20: "three-way-report-or-verdict-missing", 21: "report-unreferenced"}
    return root, repository, base + datetime.timedelta(minutes=303), {identifiers[number]: bucket for number, bucket in expected.items()}


def selftest_outcome():
    """outcome 的自证：18 个答案已知的子 agent（每个桶至少一个）、一个带历史、另挂了一棵工作树的假仓。返回 (项数, 失败句, 子 agent 个数)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-outcome-selftest-")
    failures, checked, expected = [], 0, {}
    try:
        root, repository, now, expected = build_outcome_fixture(work)
        since = datetime.datetime(2026, 10, 1, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        agents = collect_outcome(root, since, repository, now, workers=2)   # 走工作进程那一支
        actual = {agent["agent"]: agent["bucket"] for agent in agents}
        identifiers = sorted(expected)
        normalized = [identifiers[number - 1] for number in (6, 9, 10, 11, 12)]
        bash_written = [identifiers[number - 1] for number in (2, 3, 4)]
        checked += 1
        if len(agents) != len(expected):
            failures.append(f"子 agent 个数：应当 {len(expected)}，实际 {len(agents)}")
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in normalized if actual.get(name) != expected[name]}
        if wrong:
            failures.append(f"路径归一化：交回里粘着全角括号与说明文字的路径、主会话用循环与花括号拼的路径、只点到草稿目录名的，各自都要点得到（预期桶是 written-overwritten、report-used-by-main、report-used-by-later-sub），实际（预期, 实际）{wrong}")
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in bash_written if actual.get(name) != expected[name]}
        if wrong:
            failures.append(f"Bash 写入：heredoc、python、replace-once / insert-row / replace-batch 的实写都要算进落地（--dry-run 的不算），应得 written-landed，实际（预期, 实际）{wrong}")
        legs_confirmed = [identifiers[8 - 1]]
        legs_missing = [identifiers[number - 1] for number in (19, 20)]
        not_a_leg = [identifiers[21 - 1]]
        checked += 1
        wrong = {name: (bucket, actual.get(name)) for name, bucket in expected.items()
                 if name not in normalized + bash_written + legs_confirmed + legs_missing + not_a_leg and actual.get(name) != bucket}
        if wrong:
            failures.append(f"结局桶：其余每个桶的答案已知（报错的写不算、半截落地、被点到与没被点到的分开），实际和预期不符（预期, 实际）{wrong}")
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in legs_confirmed if actual.get(name) != expected[name]}
        if wrong:
            failures.append(f"三方腿判决在：交回点名的 research/prompts/fx-r1-opus-output.md 在提交里、同轮判决 fx-r1-main-verification.md 提交过又删掉（只在 git 历史里），"
                            f"应得 verdict-archived，实际（预期, 实际）{wrong}")
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in legs_missing if actual.get(name) != expected[name]}
        if wrong:
            failures.append(f"三方腿判决不在：报告在主工作树而同轮没有判决（fy-r1）、判决在另一棵工作树而报告哪里都没有（fz-r1），"
                            f"应得 three-way-report-or-verdict-missing，实际（预期, 实际）{wrong}")
        # 三方腿类型表（拆分提案第九十节 ⑯）：读到的类型集合与登记处（定义 frontmatter 的 leg: three-way 一行）逐个相同——假仓里材料员以 three-way 起头而没登记，
        # 本仓按 frontmatter 现扫、至少要有一种；材料员交回了照引用分桶，不进三方腿的两个桶（开关 leg-types-by-prefix 下这一项红）
        frontmatter_scanned_legs = three_way_legs.three_way_leg_types_scanned   # 「frontmatter 里写着什么」：同一份库里不拆键值的另一种读法
        fixture_legs, repository_legs = frontmatter_scanned_legs(repository), frontmatter_scanned_legs(REPOSITORY_ROOT)
        checked += 1
        wrong = {name: (expected[name], actual.get(name)) for name in not_a_leg if actual.get(name) != expected[name]}
        if (sorted(three_way_leg_types_of(repository)) != fixture_legs or not repository_legs
                or sorted(three_way_leg_types_of(REPOSITORY_ROOT)) != repository_legs or wrong):
            failures.append(f"三方腿类型表：假仓读到的 {sorted(three_way_leg_types_of(repository))} 应当与 frontmatter 的 {fixture_legs} 逐个相同，"
                            f"本仓读到的 {sorted(three_way_leg_types_of(REPOSITORY_ROOT))} 应当与 frontmatter 的 {repository_legs} 逐个相同、至少登记了一种，"
                            f"没登记的材料员应得 report-unreferenced，实际（预期, 实际）{wrong}")
        summary = outcome_summary(agents)
        checked += 1
        if summary["rework"] != (2, 3000) or summary["cuts"] != collections.Counter({"用量撞顶": 1, "API 错": 1, "停掉": 2, "还在跑": 1}):
            failures.append(f"汇总：返工字样应当 2 个 3000（「重跑一次没人看」「停掉被续做」）、没交回的应当用量撞顶 1、API 错 1、停掉 2、还在跑 1，实际 {summary['rework']}、{dict(summary['cuts'])}")
        checked += 1
        counted_as_rework = ["七·6 代码轮第二轮攻方腿（接续）", "实验执行：E181 第一段（接续）", "m3-batch28-code-r1 正推腿（续）", "7-2b 接手：M3 在核上证",
                             "准入条款簇三方攻方腿（重派）", "Runner E187 r2 (redispatch)", "Fix E193 guard flush bug and rerun", "E156 第 7 次收尾", "批 12 变异三行修红",
                             "Batch 9 merge repin and fixes", "E193 guard fixed", "fix the gate verdict"]
        not_counted_as_rework = ["七·6 后续批第二批实现", "E171 合谋测试第一段", "第 1 次执行员", "Batch 10 tooling", "乙簇三方正推腿", "E177 跑前登记修订",
                                 "C531 fixture registrations excluded", "C487 segments-step red fixture", "doc-experiments fixtures 补齐", "fixed-script 流的崩溃点枚举"]
        rework_misjudged = [description for description in counted_as_rework if not rework_words().search(description)] + \
                           [description for description in not_counted_as_rework if rework_words().search(description)]
        if rework_misjudged:
            failures.append(f"返工词表：接续、接手、重派、redispatch、fix、fixes、fixed、第 N 次这些续派要算返工，「后续批」「第 1 次」这类只说次序的、「修订」、"
                            f"fixture、fixtures、fixed-script 不算，判错的 {rework_misjudged}")
        checked += 1
        if abs(summary["total_cost"] - 1500 * len(expected)) > 1e-6 or summary["by_bucket"]["verdict-archived"] != [1, 1500]:
            failures.append(f"花费：每个子 agent 折算 1000 + 5 × 100 = 1500，合计应当 {1500 * len(expected)}、三方腿 1 个应当 1500，实际 {summary['total_cost']}、{summary['by_bucket'].get('verdict-archived')}")
        checked += 1
        printed = "\n".join(outcome_lines(agents, since, True))
        if f"{len(agents)} 个" not in printed or "report-used-by-main" not in printed or "返工字样" not in printed:
            failures.append("输出：每个子 agent 一行与汇总都要打出来，汇总标题要带个数")
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        failures.append(f"造假仓或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)   # 假仓里另挂的工作树在同一个临时目录里，一起删
    return checked, failures, len(expected)


def selftest_handback_and_start():
    """交回与起点：subagent_transcript.py 的两份共用样本（先被拒、后来送达 / 被拒之后再没送达）与一份 --until 那一天之后开工的。
    outcome 要判前一份交回、后一份没交回、第三份不收；timing 的「没交回」只数后一份。返回 (项数, 失败句)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-handback-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "projects")
        subagents = os.path.join(root, "s3", "subagents")
        os.makedirs(subagents)
        base = transcripts.SAMPLE_BASE
        start_day = base.replace(hour=0, minute=0)
        samples = {"a00000000000000a1": (transcripts.sample_refused_then_delivered(base), True),
                   "a00000000000000a2": (transcripts.sample_refused_only(base), False),
                   "a00000000000000a3": (transcripts.sample_refused_then_delivered(base + datetime.timedelta(days=2)), None)}   # --until 那一天之后开工：不收
        for identifier, (records_list, _) in samples.items():
            write_json_lines(os.path.join(subagents, f"agent-{identifier}.jsonl"), records_list)
            with open(os.path.join(subagents, f"agent-{identifier}.meta.json"), "w", encoding="utf-8") as handle:
                json.dump({"agentType": "clerk", "description": "交回样本"}, handle)
        since, until = start_day - datetime.timedelta(days=4), start_day + datetime.timedelta(days=1)   # --until 写样本开工的那一天（含那一天）
        agents = collect_outcome(root, since, work, base + datetime.timedelta(days=10), workers=1, until=until)
        got = {agent["agent"]: agent["handed_back"] for agent in agents}
        want = {identifier: delivered for identifier, (_, delivered) in samples.items() if delivered is not None}
        checked += 1
        if got != want:
            failures.append(f"交回与起点：outcome 应当收开工在 --until 那一天及之前的两个、先被拒后送达的算交回、被拒之后再没送达的不算（{want}），实际 {got}")
        _, timing = command_timing(root, since)
        rows = timing.get("clerk", [])
        checked += 1
        if len(rows) != 3 or sum(1 for row in rows if row[2]) != 1:
            failures.append(f"交回与起点：timing 应当收 3 个、「没交回」只数被拒之后再没送达的 1 个，实际 {len(rows)} 个、没交回 {sum(1 for row in rows if row[2])} 个")
    except (OSError, RuntimeError) as error:
        failures.append(f"交回与起点：造样本或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return checked, failures


def selftest_date_zone():
    """东京日期切天：--since D --until D 收东京 D 日零点刚过与 D 日深夜开工的两个，不收东京 D+1 日零点刚过开工的（会话记录时间戳的时区里它还是 D 日）；
    timing 照同一个 --since 收。返回 (项数, 失败句)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-date-zone-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "projects")
        subagents = os.path.join(root, "s4", "subagents")
        os.makedirs(subagents)
        day = "2026-10-21"
        midnight = datetime.datetime.fromisoformat(day).replace(tzinfo=DATE_ZONE)
        starts = {"a00000000000000d1": midnight + datetime.timedelta(minutes=30),                   # 东京 D 日零点刚过（会话记录时区里是前一天）：收
                  "a00000000000000d2": midnight + datetime.timedelta(hours=23, minutes=30),         # 东京 D 日深夜：收
                  "a00000000000000d3": midnight + datetime.timedelta(days=1, minutes=30)}           # 东京 D+1 日零点刚过（会话记录时区里还是 D 日）：不收
        for identifier, start in starts.items():
            write_json_lines(os.path.join(subagents, f"agent-{identifier}.jsonl"), transcripts.sample_refused_then_delivered(start.astimezone(UTC)))   # clock-times:allow 样本时间戳照会话记录写成另一时区
            with open(os.path.join(subagents, f"agent-{identifier}.meta.json"), "w", encoding="utf-8") as handle:
                json.dump({"agentType": "clerk", "description": "东京日期样本"}, handle)
        since, until = window_bounds(day, day)
        agents = collect_outcome(root, since, work, midnight + datetime.timedelta(days=10), workers=1, until=until)
        got = sorted(agent["agent"] for agent in agents)
        days = sorted(local_day(agent["start"]).isoformat() for agent in agents)
        checked += 1
        if got != ["a00000000000000d1", "a00000000000000d2"] or days != [day, day]:
            failures.append(f"东京日期切天：outcome --since {day} --until {day} 应当收东京 {day} 零点刚过与深夜开工的两个、开始日期都写 {day}，"
                            f"不收东京第二天零点刚过开工的，实际收了 {got}、开始日期 {days}")
        _, timing = command_timing(root, window_bounds(day)[0])
        checked += 1
        if len(timing.get("clerk", [])) != 3:
            failures.append(f"东京日期切天：timing --since {day} 应当收 3 个（东京 {day} 零点刚过开工的也收），实际 {len(timing.get('clerk', []))} 个")
    except (OSError, RuntimeError) as error:
        failures.append(f"东京日期切天：造样本或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return checked, failures


def selftest_main_session_window():
    """主会话按记录时刻切天：一份跨东京两天的主会话记录（前一天深夜两次调用、其中一次派 sweep；后一天零点刚过一次调用、派 clerk，
    这一次在会话记录时间戳的时区里还是前一天），--since 后一天只收后一天那一次，--since 前一天 --until 前一天只收前一天那两次；usage 与 dispatch 都核。
    返回 (项数, 失败句)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-main-window-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "projects")
        os.makedirs(root)
        day, later_day = "2026-10-20", "2026-10-21"
        midnight = datetime.datetime.fromisoformat(later_day).replace(tzinfo=DATE_ZONE)
        usage = {"input_tokens": 10, "cache_read_input_tokens": 1000, "cache_creation_input_tokens": 0, "output_tokens": 5}

        def call(identifier, moment, dispatched=None):
            content = ([{"type": "tool_use", "id": "t" + identifier, "name": "Agent", "input": {"subagent_type": dispatched, "description": identifier, "prompt": "x"}}]
                       if dispatched else [{"type": "text", "text": "x"}])
            return {"type": "assistant", "uuid": "u" + identifier, "timestamp": moment.astimezone(UTC).isoformat(),   # clock-times:allow 样本时间戳照会话记录写成 UTC
                    "message": {"id": "m" + identifier, "role": "assistant", "content": content, "usage": usage}}

        write_json_lines(os.path.join(root, "s5.jsonl"), [
            {"type": "ai-title", "title": "不带时刻的元数据条"},
            call("1", midnight - datetime.timedelta(hours=2), "sweep"), call("2", midnight - datetime.timedelta(minutes=30)),
            call("3", midnight + datetime.timedelta(minutes=30), "clerk")])
        for label, (since, until), want_calls, want_dispatches in (
                (f"--since {later_day}", window_bounds(later_day), 1, {"clerk": 1}),
                (f"--since {day} --until {day}", window_bounds(day, day), 2, {"sweep": 1})):
            _, _, main_grand = command_usage(root, since, until)
            _, dispatches, _, _ = command_dispatch(root, since, until)
            checked += 1
            if main_grand.get("calls") != want_calls or dict(dispatches) != want_dispatches:
                failures.append(f"主会话按记录时刻切天：{label} 时 usage 应当收主会话 {want_calls} 次调用、dispatch 应当收 {want_dispatches}"
                                f"（东京日期切天，一份跨两天的主会话只收窗口里那几条），实际 {main_grand.get('calls')} 次、{dict(dispatches)}")
    except (OSError, RuntimeError) as error:
        failures.append(f"主会话按记录时刻切天：造样本或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return checked, failures


def selftest_outcome_out():
    """结局账落盘：outcome --summary --out <文件> 排他新建，文件内容与标准输出逐字相同；同一个路径再跑一次拒（退 2、给出路）、已落的账一个字不动。
    返回 (项数, 失败句)。"""
    work = tempfile.mkdtemp(prefix="agent-cost-outcome-out-selftest-")
    failures, checked = [], 0
    try:
        root = os.path.join(work, "projects")
        subagents = os.path.join(root, "s6", "subagents")
        os.makedirs(subagents)
        base = transcripts.SAMPLE_BASE
        write_json_lines(os.path.join(subagents, "agent-a00000000000000e1.jsonl"), transcripts.sample_refused_then_delivered(base))
        with open(os.path.join(subagents, "agent-a00000000000000e1.meta.json"), "w", encoding="utf-8") as handle:
            json.dump({"agentType": "clerk", "description": "落账样本"}, handle)
        day = local_day(base).isoformat()
        ledger = os.path.join(work, "ledger.txt")
        arguments = ["outcome", "--root", root, "--repo", work, "--since", day, "--until", day, "--summary", "--jobs", "1", "--out", ledger]

        def run_main():
            captured = io.StringIO()
            with contextlib.redirect_stdout(captured):
                exit_code = main(arguments)
            return exit_code, captured.getvalue()

        exit_code, printed = run_main()
        written = open(ledger, encoding="utf-8").read() if os.path.exists(ledger) else None
        checked += 1
        if exit_code != 0 or written != printed or "按结局桶" not in printed or "1 个子 agent" not in printed or f"落进了 {ledger}" not in printed:
            failures.append(f"结局账落盘：outcome --summary --out 应当退 0、把汇总与成功句写进 {ledger}、回读与标准输出逐字相同，"
                            f"实际退出码 {exit_code}、文件{'不在' if written is None else ('与标准输出相同' if written == printed else '与标准输出不同')}，输出：{printed[-300:]}")
        exit_code, refused = run_main()
        again = open(ledger, encoding="utf-8").read() if os.path.exists(ledger) else None
        checked += 1
        if exit_code != 2 or again != written or "✗ --out 指的文件已经在" not in refused or "→ 怎么办" not in refused:
            failures.append(f"结局账落盘：同一个 --out 路径再跑一次应当拒（退 2、写明文件已在与出路）、已落的账一个字不动，"
                            f"实际退出码 {exit_code}、账{'没动' if again == written else '被改了'}，输出：{refused[-300:]}")
    except (OSError, RuntimeError) as error:
        failures.append(f"结局账落盘：造样本或算账出错：{error}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return checked, failures


def selftest_draft_directory():
    """草稿目录的认法（draft_directory_of）：派发提示写成会话记录，经 scan_subagent 读出来的 draft 只认行首「草稿目录：」那一行、取第一个。返回 (项数, 失败句)。"""
    mine, other = "~/.local/share/singlefs/work/draft-selftest-mine", "~/.local/share/singlefs/work/draft-selftest-previous"
    cases = [
        ("骨架那一行", f"# 派发\n\n草稿目录：{mine}\n报告：{mine}/result.md\n", mine),
        ("半角冒号、路径后面的括注截在括号前", f"草稿目录: {mine}（自己建）\n", mine),
        ("句中转述在前、行首那一行在后，取行首那一行", f"前一段的草稿目录：{other}（它写的探针）\n草稿目录：{mine}\n", mine),
        ("列表里转述的不认", f"- 草稿目录：{other}\n", None),
        ("闭合了的围栏代码块里的不认", f"```\n草稿目录：{other}\n```\n草稿目录：{mine}\n", mine),
        ("没有那一行的取不到", "派发提示里没有草稿目录\n", None),
    ]
    failures, work = [], tempfile.mkdtemp(prefix="agent-cost-draft-selftest-")
    try:
        for number, (label, prompt, want) in enumerate(cases):
            path = os.path.join(work, f"agent-a{number:016x}.jsonl")
            write_json_lines(path, [{"type": "user", "timestamp": "2026-10-01T00:00:00+00:00", "message": {"role": "user", "content": prompt}}])   # clock-times:allow 自证造的会话记录行，时间戳照会话记录的格式
            got = scan_subagent(path)["draft"]
            if got != want:
                failures.append(f"草稿目录认法：{label} 应当取到 {want}，实际 {got}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    return len(cases), failures


def selftest():
    work = tempfile.mkdtemp(prefix="agent-cost-selftest-")
    failures, checked = [], 0
    try:
        session = os.path.join(work, "s1")
        os.makedirs(os.path.join(session, "subagents"))
        base = datetime.datetime(2026, 9, 28, 1, 0, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        stamp = lambda minutes: (base + datetime.timedelta(minutes=minutes)).isoformat()
        def assistant(uuid, message_id, minutes, content, usage):
            return {"type": "assistant", "uuid": uuid, "timestamp": stamp(minutes), "message": {"id": message_id, "role": "assistant", "content": content, "usage": usage}}
        usage_a = {"input_tokens": 5, "cache_read_input_tokens": 100_000, "cache_creation_input_tokens": 20_000,
                   "cache_creation": {"ephemeral_5m_input_tokens": 20_000, "ephemeral_1h_input_tokens": 0}, "output_tokens": 300}
        sub = [
            {"type": "user", "timestamp": stamp(0), "message": {"role": "user", "content": "派发提示"}},
            assistant("u1", "m1", 1, [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "ls"}}], usage_a),
            assistant("u1b", "m1", 1, [{"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "ls"}}], usage_a),   # 流式重复行，同一个消息 id
            assistant("u2", "m2", 6, [{"type": "tool_use", "id": "t2", "name": "Bash", "input": {"command": "cargo test"}}], usage_a),   # 隔 5 分钟：算等待
            assistant("u3", "m3", 7, [{"type": "tool_use", "id": "t3", "name": "SubagentHandback", "input": {"message": "交回"}}], usage_a),
            {"type": "user", "timestamp": stamp(7), "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "t3", "content": "{\"success\":true}"}]}},
        ]
        write_json_lines(os.path.join(session, "subagents", "agent-a0000000000000001.jsonl"), sub)
        with open(os.path.join(session, "subagents", "agent-a0000000000000001.meta.json"), "w", encoding="utf-8") as handle:
            json.dump({"agentType": "sweep", "description": "样本"}, handle)
        main = [
            assistant("m-u1", "mm1", 0, [{"type": "tool_use", "id": "d1", "name": "Agent", "input": {"subagent_type": "sweep", "description": "回扫样本", "prompt": "x"}}],
                      {"input_tokens": 10, "cache_read_input_tokens": 500_000, "cache_creation_input_tokens": 1000, "cache_creation": {"ephemeral_5m_input_tokens": 0, "ephemeral_1h_input_tokens": 1000}, "output_tokens": 50}),
            {"type": "user", "timestamp": stamp(0.1), "toolUseResult": {"agentId": "a0000000000000001"},
             "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "d1", "content": "Async agent launched"}]}},
            assistant("m-u2", "mm2", 0.2, [{"type": "tool_use", "id": "d2", "name": "Agent", "input": {"subagent_type": "general-purpose", "description": "查一下", "prompt": "y"}}],
                      {"input_tokens": 10, "cache_read_input_tokens": 500_000, "cache_creation_input_tokens": 0, "output_tokens": 50}),
            {"type": "user", "timestamp": stamp(0.3),
             "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": "d2", "content": "PreToolUse:Agent hook error: [bash \"$CLAUDE_PROJECT_DIR\"/.claude/hooks/runner-dispatch-guard.sh]: ✗ 派 general-purpose：内置通用类型"}]}},
            {"type": "user", "timestamp": stamp(8), "message": {"role": "user", "content": "<task-notification><task-id>a0000000000000001</task-id><status>completed</status></task-notification>"}},
        ]
        write_json_lines(session + ".jsonl", main)
        since = datetime.datetime(2026, 1, 1, tzinfo=UTC)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        _, by_type, main_grand = command_usage(work, since)
        checked += 1
        sweep = by_type.get("sweep", {})
        if sweep.get("n") != 1 or sweep.get("calls") != 3 or sweep.get("cache_read") != 300_000 or sweep.get("write_5m") != 60_000 or sweep.get("output") != 900:
            failures.append(f"子 agent 用量：应当 1 个、3 次调用（同一个消息 id 的重复行只算一次）、读缓存 300000、5 分钟写 60000、输出 900，实际 {dict(sweep)}")
        checked += 1
        if main_grand.get("calls") != 2 or main_grand.get("write_1h") != 1000:
            failures.append(f"主会话用量：应当 2 次调用、1 小时写 1000，实际 {dict(main_grand)}")
        _, timing = command_timing(work, since)
        checked += 1
        rows = timing.get("sweep", [])
        if len(rows) != 1 or abs(rows[0][0] - 7 * 60) > 1 or abs(rows[0][1] - 5 * 60) > 1 or rows[0][2]:
            failures.append(f"时间：应当挂钟 7 分钟、等待 5 分钟、已交回，实际 {rows}")
        _, dispatches, rejections, launched = command_dispatch(work, since)
        checked += 1
        if dict(dispatches) != {"sweep": 1, "general-purpose": 1} or launched != 1 or sum(rejections.get("runner-dispatch-guard.sh", {}).values()) != 1:
            failures.append(f"派发：应当 sweep 1、general-purpose 1、真正起来 1、派发闸拒 1，实际 {dict(dispatches)}、{launched}、{dict(rejections)}")
    finally:
        shutil.rmtree(work, ignore_errors=True)
    outcome_checked, outcome_failures, outcome_agents = selftest_outcome()
    checked += outcome_checked
    failures += outcome_failures
    handback_checked, handback_failures = selftest_handback_and_start()
    checked += handback_checked
    failures += handback_failures
    zone_checked, zone_failures = selftest_date_zone()
    checked += zone_checked
    failures += zone_failures
    for extra_checked, extra_failures in (selftest_main_session_window(), selftest_outcome_out(), selftest_draft_directory()):
        checked += extra_checked
        failures += extra_failures
    for failure in failures:
        print(f"  ✗ 自检：{failure}")   # gate-lint:detail
    if failures:
        print("    → 怎么办：看 read_transcript() 的去重与 command_*() 的口径；outcome 看 scan_subagent()、handback_paths()、index_references()、bash_written_lines()；"
              "交回与起点看 subagent_transcript.py；东京日期切天看 window_bounds()；主会话按记录时刻切天看 records_between()；结局账落盘看 write_ledger_exclusively()；"
              "三方腿看 prompts_paths()、leg_output_verdicts()，三方腿类型表看 three_way_leg_types_of() 与定义 frontmatter 的 leg: three-way；"
              "AGENT_COST_BREAK 设着 no-dedupe、no-path-normalize、no-bash-writes、narrow-rework-words、fix-no-word-end、day-cut-by-record-zone、three-way-by-type、three-way-history-unread、leg-types-by-prefix、"
              "main-session-by-mtime、outcome-out-clobber 或 draft-anywhere-in-line，"
              "或 SUBAGENT_TRANSCRIPT_BREAK 设着的话这里本来就该红")
        return 1
    print(f"  ✓ 自检通过（查了 {checked} 项，其中 outcome 的 {outcome_agents} 个子 agent）：调用按消息 id 去重、读缓存与两档缓存写分开、等待按 3 分钟间隔算、派发只数真正起来的、派发闸的拒绝按钩子归类、"
          "每个结局桶都对得上、路径归一化、Bash 里的写入认得出、「修订」与 fixture 这类不算返工；三方腿报告在仓且同轮有判决才记 verdict-archived（判决只在 git 历史里也算），三方腿类型读定义 frontmatter 的 leg: three-way（假仓与本仓读到的都与 frontmatter 逐个相同）；"
          "交回按送达认（先被拒后送达的算、被拒之后再没送达的不算）、起点按首条记录、--until 含那一天、--since 与 --until 按东京日期切天；"
          "主会话按每条记录的时刻切天（跨两天的主会话只收窗口里那几条）；outcome --out 排他落账、回读与标准输出逐字相同、已在就拒；"
          "草稿目录只认派发提示行首「草稿目录：」那一行（认法同 lib_prompt_scratch_line.py）")
    return 0


def write_ledger_exclusively(path, text):
    """outcome --out：把 text 排他新建进 path、回读核逐字相同（文件头 --out 那一行）。交 (退出码, 说明)：成了是 (0, None)；
    文件已在是 2、写不进与回读对不上是 1，说明是一句带出路的 ✗；回读对不上时删掉这一次建的文件。弄坏开关 outcome-out-clobber 下不排他，已在的整份覆盖。"""
    try:
        with open(path, "w" if BROKEN == "outcome-out-clobber" else "x", encoding="utf-8") as handle:
            handle.write(text)
    except FileExistsError:
        return 2, (f"  ✗ --out 指的文件已经在：{path}（算账这段时间里别处建了它；结局账只排他新建，不覆盖）\n"
                f"  → 怎么办：换一个还没有的文件名重跑；旧的那份是谁建的先核清楚")
    except OSError as error:
        return 1, f"  ✗ 结局账写不进 {path}：{error}\n  → 怎么办：核那个目录能不能写、盘满没满，再换一个路径重跑"
    with open(path, encoding="utf-8") as handle:
        written = handle.read()
    if written != text:
        os.remove(path)
        return 1, (f"  ✗ 结局账写进 {path} 之后回读与要打出的输出不一样（写了 {len(text)} 字、读回 {len(written)} 字），这一次建的文件已删\n"
                f"  → 怎么办：核那个文件系统（网络盘、满了没有）与有没有别的进程同时写它，换到本机的目录重跑")
    return 0, None


def main(argv):
    if argv[:1] == ["--selftest"]:
        return selftest()
    if not argv or argv[0] not in ("usage", "timing", "dispatch", "outcome"):
        print("  ✗ 用法：agent-cost.py usage|timing|dispatch|outcome [--root 会话目录] [--since YYYY-MM-DD] | --selftest\n  → 怎么办：第一个参数写 usage、timing、dispatch 或 outcome")
        return 2
    root, since = DEFAULT_ROOT, datetime.datetime.now(UTC) - datetime.timedelta(days=7)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
    repository, summary_only, workers, out = REPOSITORY_ROOT, False, None, None
    since_given, until_day, until = None, None, None
    rest = argv[1:]
    while rest:
        flag = rest.pop(0)
        if flag == "--root" and rest:
            root = rest.pop(0)
        elif flag in ("--since", "--until") and rest and re.fullmatch(r"\d{4}-\d{2}-\d{2}", rest[0]):
            value = rest.pop(0)
            try:
                if flag == "--since":
                    since = window_bounds(value)[0]
                    since_given = value
                else:
                    until = window_bounds(value, value)[1]
                    until_day = value
            except (ValueError, OverflowError):
                print(f"  ✗ {flag} 不是可用的日期：{value}\n  → 怎么办：写真实存在的 YYYY-MM-DD 日期，结束日期还须能计算下一天")
                return 2
        elif flag == "--repo" and rest and argv[0] == "outcome":
            repository = rest.pop(0)
        elif flag == "--summary" and argv[0] == "outcome":
            summary_only = True
        elif flag == "--jobs" and rest and argv[0] == "outcome" and rest[0].isdigit() and int(rest[0]) >= 1:
            workers = int(rest.pop(0))
        elif flag == "--out" and rest and argv[0] == "outcome" and rest[0]:
            out = rest.pop(0)
        else:
            print(f"  ✗ 认不出参数 {flag}\n  → 怎么办：只认 --root、--since YYYY-MM-DD 与 --until YYYY-MM-DD（含那一天）；"
                  "outcome 另认 --repo 仓根、--summary、--jobs 读会话记录的进程数、--out 落账的文件")
            return 2
    if until is not None and until <= since:
        print(f"  ✗ --until {until_day} 早于 --since {since.date()}，窗口是空的\n  → 怎么办：--until 写不早于 --since 的那一天")
        return 2
    if not os.path.isdir(root):
        print(f"  ✗ 会话目录不在：{root}\n  → 怎么办：--root 给 ~/.claude/projects/<项目> 那个目录")
        return 2
    if argv[0] == "outcome":
        if workers is None:
            try:
                workers = min(12, heavy_threads.default_heavy_threads())
            except heavy_threads.HeavyThreadDefaultError as error:
                print(f"  ✗ 取不了读会话记录的默认进程数：{error}\n  → 怎么办：{error.howto}；或写 --jobs N")
                return 2
        if not os.path.isdir(repository):
            print(f"  ✗ 仓不在：{repository}\n  → 怎么办：--repo 给 git 仓的根目录")
            return 2
        if out is not None and os.path.lexists(out) and BROKEN != "outcome-out-clobber":
            print(f"  ✗ --out 指的文件已经在：{out}（结局账只排他新建，不覆盖已落的账）\n"
                  f"  → 怎么办：换一个还没有的文件名（例如文件名里带这一次 --until 的日期或批名）；旧的那份要作废，先核过它是不是别人要引的账，再自己删掉重跑")
            return 2
        if out is not None and not os.path.isdir(os.path.dirname(os.path.abspath(out))):
            print(f"  ✗ --out 的上层目录不在：{os.path.dirname(os.path.abspath(out))}\n  → 怎么办：先建好那一轮调度记录的目录，或把 --out 指到已有的目录里")
            return 2
        try:
            agents = collect_outcome(root, since, repository, datetime.datetime.now(UTC), workers, until)   # clock-times:allow 会话记录的时间戳是 UTC，按它比较与切日期，不是给人看的钟点
        except (RuntimeError, OSError) as error:
            print(f"  ✗ 比对仓里的行时出错：{error}\n  → 怎么办：确认 --repo 是 git 仓、git 在 PATH 上；错误原文在上面一句")
            return 1
        window = (f"--since {since.date()}" + ("" if since_given else "（没给，取 7 天前）")
                  + (f"、--until {until_day}（含那一天）" if until_day else "、--until 没给（不设上界）"))
        if not agents:
            print(f"  ✗ {window} 这个窗口里没有开工的子 agent，什么都没判\n  → 怎么办：--since 往前挪、--until 往后挪，或确认 --root 指的是 ~/.claude/projects/<项目> 那个目录")
            return 77
        text = ("\n".join(outcome_lines(agents, since, not summary_only, until_day)) + "\n"
                + f"  ✓ 查了 {len(agents)} 个子 agent（{window}，日期按东京日期（Asia/Tokyo）切天，按首条记录的时刻筛），逐个归了结局桶"
                + (f"；这份输出也落进了 {out}（排他新建，回读与标准输出逐字相同）" if out is not None else "") + "\n")
        if out is not None:
            exit_code, problem = write_ledger_exclusively(out, text)
            if exit_code:
                print(problem)
                return exit_code
        sys.stdout.write(text)
        return 0
    lines = {"usage": lambda: command_usage(root, since, until)[0], "timing": lambda: command_timing(root, since, until)[0],
             "dispatch": lambda: command_dispatch(root, since, until)[0]}[argv[0]]()
    print("\n".join(lines))
    return 0


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
