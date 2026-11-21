# 派发闸 runner-dispatch-guard.sh 的 python 部分之一：自证（样本提示、真提示、selftest）、--check-prompt、弄坏开关只在自证里认的拒绝、main 与读 hook 输入的入口流程。
# 入口 runner-dispatch-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 RUNNER_DISPATCH_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from __main__ import BATCH_LINE_AGENTS, RUNNER_DISPATCH_PART_NAMES, SELFTEST_MARK_CONTENT, SELFTEST_MARK_VARIABLE
from runner_dispatch_guard_locks_and_verdicts import (
    ADDRESSED_HEADING, NOTATIONS_FORM, ZERO_ROUND_VERDICT_PATH, acquire_writer_locks, combined_refusal, handover_lock_verdict, judge_dispatch,
    notations_library, record_unjudged, registration_has_segment, report_location_verdict, scratch_directories_named, scratch_line_fence_ambiguity,
    scratch_line_root_verdict, zero_round_reading_verdict,
)
from runner_dispatch_guard_prompt_checks import (
    ADMISSION_RETIRED_VARIABLE, CODE_BLOCK_LIBRARY_VARIABLE, SCRIPT_HOOK_DIR, SUBAGENT_CONCURRENCY_LIMIT, UNJUDGED_NOTES, WRITER_GUARDED_AGENTS,
    agent_definition_fields, break_switch_is, code_block_library_note_printed, designer_admission_verdict, kb_scribe_spec_verdict, leg_round_names,
    live_registry_rows, prompt_scratch_directories, prompt_scratch_line_error, quoted_spans, report_file_name_verdict, report_scratch_directories,
    rewrite_registry, scope_patterns, session_dispatch_events, shell_words, shell_words_error, snapshot_library, specification_texts,
    three_way_legs_error, three_way_report_types, three_way_report_verdict, work_roots, work_roots_error,
)
import runner_dispatch_guard_locks_and_verdicts
import runner_dispatch_guard_prompt_checks

# 自证里换成替身的模块级名字 → 定义它、读它的模块：替身要换在那个模块里（import 来的名字是另一份，换在这一份里读它的函数看不到）
REPLACEABLE_NAME_OWNERS = {
    "write_scope": runner_dispatch_guard_prompt_checks,
    "prompt_scratch_line": runner_dispatch_guard_prompt_checks,
    "three_way_legs": runner_dispatch_guard_prompt_checks,
    "writer_lock": runner_dispatch_guard_prompt_checks,
    "registry_path": runner_dispatch_guard_prompt_checks,
    "NOTATIONS_LIBRARY_PATH": runner_dispatch_guard_locks_and_verdicts,
    "decide": runner_dispatch_guard_locks_and_verdicts,
}

SELFTEST_BATCH_RECORD = os.path.join("records", "selftest-batch.md")   # ⑱ 批次一行点名的调度记录：自检的每个样本仓里都建这一份（write_batch_record）
SELFTEST_BATCH_LINE = f"批次：{SELFTEST_BATCH_RECORD}#1\n"
FOOTER = "开工先读 `.claude/agent-common.md`\n规格检查已判：自检样本不带规格\n" + SELFTEST_BATCH_LINE
SELFTEST_UNTOUCHED_LINE = "不碰：无\n"   # ⑱ 实现员、工具员要的不碰一行（自检样本默认补）
SELFTEST_ADDRESSED_SECTION = "\n## 关的是哪几条\n\n- 自检样本\n\n"   # ⑱ 有定义的类型要的那一节（自检样本默认补）

def write_batch_record(project_root):
    """自检样本仓里建 ⑱ 批次一行点名的那份调度记录（已在就不动）。"""
    path = os.path.join(project_root, SELFTEST_BATCH_RECORD)
    if not os.path.isfile(path):
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("# 自检样本的调度记录\n")

# ⑳ 自证读真提示（拆分提案第九十节 ⑳、㉘、㉜）：从会话记录取出的派发提示原样（加照做法改出的几份），逐份只看报告类两种拒。
# 那 14 份逐字节拷进钩子自己的样本目录（相对本钩子所在目录：研究目录下一次归档会删），目录里 sources.tsv 写来历与整份 sha256。
# 另 7 份（拆分提案第九十节 ㉗、㉙）：landing-audit-r4、r5、r6 六份真腿提示从各自会话记录的首条提示原样取出，加 r4 攻方那一份改一处（禁读清单写成本腿自己的报告）。
# 每行：文件名、整份的 sha256（与 sources.tsv 逐份相同；前 14 份另与 research/prompts/landing-audit-r3-opus-output.md 列的逐份相同；读到的对不上那一格判红）、类型、报告文件名拒不拒、三方腿报告路径拒不拒、依据。
REAL_PROMPTS_DIRECTORY = os.path.join("fixtures", "runner-dispatch-guard", "real-prompts")
REAL_PROMPT_SOURCES = "sources.tsv"
REAL_PROMPT_EXPECTATIONS = (
    ("a3652270eb80e4fd0.txt", "7155650c7fea1113699845bd7df795a9aaf99dd674d74fc78b985016bcc38156", "three-way-attack", False, False, "第二轮攻方腿：报告行写在句号之后、形态对"),
    ("a3652270eb80e4fd0-delivery-colon.txt", "9832e848f67c4eef8878e60190dba887f304e3478835d9c45703aaca712f7c87", "three-way-attack", False, False, "「交付：报告 …-output.md」写法（第三轮判决 K2-3）"),
    ("a3652270eb80e4fd0-section-colon.txt", "4dda75158e387a8c7b1c40719210ace881515d302d3e22d20caedad0b9624105", "three-way-attack", False, False, "「照正文「交付」：报告写进 …-output.md」写法（第三轮判决 K2-3）"),
    ("a303d33b0ccd5e68f.txt", "e021deda1917b2dbd91c814040ff7dfb93ff82eb1af7124d2edbb4410d2c2bd4", "tooling-writer", False, False, "报告是草稿目录下的 result.md"),
    ("aa9ec3423c1f54bba.txt", "d365fba837d920d040f7c38cf6ce04a11f0e391505448669493fc07d8369fd81", "implementation-writer", True, False, "自己的报告行写草稿目录下的 report.md"),
    ("aa9ec3423c1f54bba-own-result.txt", "7bca8a78304266e42377a376b7c21386d6ed4cb793a13ecadb510043937ebb9e", "implementation-writer", False, False, "自己的报告行改成 result.md，转述别处的 report.md 不判（第三轮判决 K2-1）"),
    ("a4737c6f3660f4f7e.txt", "089035c008e9c601d71f245626dfef6aff114491c7f9a5c0f63e41f5b0e3ddb2", "experiment-runner", True, False, "自己的报告行写草稿目录下的 report.md"),
    ("a4737c6f3660f4f7e-own-result.txt", "c469195bef2c16482a1aded21a490a7a36f2009ce7a66b902247e3483c498c2c", "experiment-runner", False, False, "自己的报告行改成 result.md，转述别处的 report.md 不判（第三轮判决 K2-1）"),
    ("a28fe10b58d88beec.txt", "537865dd7d924786d4633c60888070975639e4ff2c9a1c5c1e508ee5dcb12ebb", "kb-scribe", True, False, "自己的报告行在行首写 report.md"),
    ("a86c5c8fb8ec28789.txt", "e6604276c0d2617952e59d0a6dfa59a446b283ebf292ae20e7e02a99d2162408", "implementation-writer", True, False, "自己的报告行在行首写 report.md"),
    ("a0a2dfee322e26786.txt", "f38acb831b6696f967c02caa39ed99ab70a2250bf49d4c6817136a02c6635b0d", "experiment-designer", True, False, "逗号之后、落在本提示草稿目录下的 report.md（第三轮判决 K2-2）"),
    ("a789e7091d91430aa.txt", "d93588072534e3b3f246b8c587c32697f8adb9a8699ea71f83ef6711cf3c9378", "experiment-designer", True, False, "逗号之后、落在本提示草稿目录下的 report.md（第三轮判决 K2-2）"),
    ("a8afb2185cfa840cc.txt", "dfe14fd65f857490f9534b49d9256ef2a752b5c344e90303c792c9e8d87ff1d9", "experiment-designer", True, False, "逗号之后、落在本提示草稿目录下的 report.md（第三轮判决 K2-2）"),
    ("a9717cf07fb6e7c21.txt", "a192f6e07576e2ba18bd4bcd70fd6c81c624793bc40524309537f0a0d9d3c466", "tooling-writer", False, False,
     "草稿目录只写在句中、没有行首「草稿目录：」一行，只判裸文件名（第四轮判决 K2-4：认法收成骨架那一行，这一份不照骨架写，不再罩它）"),
    # landing-audit-r4、r5、r6 三轮的六份真腿提示（拆分提案第九十节 ㉗、㉙）：禁读清单列同一轮另一条腿的 -output.md、报告行写本腿的，照放行
    ("a61301bfaed0a3433.txt", "7b83634a68c54b15fbf6b7b40f1b8f041576f58766f68abd0a7e3f48b42e21ec", "three-way-forward", False, False, "landing-audit-r4 正推腿"),
    ("a75e9e30847f57bcd.txt", "b217748962c061fcb3f7308f83e6cb1e6d01c6e65b892ea48118ee2eb6af3e39", "three-way-attack", False, False, "landing-audit-r4 攻方腿"),
    ("a77dffd2065a0ba7e.txt", "97d2076d75dc6eb2c2f006b29c869cce988e997dfc89fb71c6263c3400c38bf8", "three-way-attack", False, False, "landing-audit-r5 攻方腿"),
    ("ab7f880bc2d28e03a.txt", "fddaabd60132380380ea6e47933d8854d843fc02be6bef9536ec9996906d1875", "three-way-defense", False, False, "landing-audit-r5 辩方腿，正文里另点名前一轮两条腿的报告"),
    ("a8a3bd6f093e19673.txt", "6e5a05847e8c97bf50c8a9081961ca8d51fa1c99f6c1cbf47f66b06543ef7740", "three-way-attack", False, False, "landing-audit-r6 攻方腿"),
    ("a42154ccf8829dc32.txt", "ca1f0e92b75c450f5bd90e28d996991211943e6f50d4b6f856979903962e9657", "three-way-defense", False, False, "landing-audit-r6 辩方腿，正文里另点名前两轮四条腿的报告"),
    ("a75e9e30847f57bcd-forbidden-own.txt", "36c137d3a17e5f68c5c2e231c99321368f9cd80e6f22c50f161673a18bd68055", "three-way-attack", False, True,
     "三方腿真提示改一处：禁读清单写成本腿自己的报告，拒"),
)

REAL_PROMPT_WORK_SUFFIX = "/.local/share/singlefs/work"   # 真提示里 ~/ 写的工作根

def real_prompt_home():
    """⑳ 真提示里 ~/.local/share/singlefs/work/… 写的草稿目录与报告，~ 展开之后要落在配置的工作根底下，两台判得才一样（第二台的家目录不是共享工作根往上四层的那一层）：
    配置的工作根以 /.local/share/singlefs/work 收尾的，交它前面那一段（判真提示的那几格拿它当家目录）；别的交 None（家目录照旧）。
    弄坏开关 real-prompts-home-unaligned 下交 None。"""
    try:
        root = work_roots.configured_work_root()
    except ValueError:   # 数据根没配置
        return None
    if break_switch_is("real-prompts-home-unaligned") or not root.endswith(REAL_PROMPT_WORK_SUFFIX):
        return None
    return root[:-len(REAL_PROMPT_WORK_SUFFIX)]

def real_prompt_cells(directory, project_root):
    """⑳ 读真提示的那几格 → ("ran", cases)，每份一格：(「真提示:<文件>（<类型>，<依据>）」, (sha256, 报告文件名拒不拒, 三方腿报告路径拒不拒) 应得, 实得)；
    三方腿形态按 project_root 下定义 frontmatter 的 leg: three-way 判，判的时候 HOME 换成 real_prompt_home 交的那一段（判完换回）。
    目录不在、少一份、sha256 对不上的那几格都判红，不报本次未跑（判决 landing-audit-r5 K2-3a）。
    弄坏开关 real-prompts-absent-green 下目录不在、少一份的那几格记绿，real-prompts-digest-unchecked 下读到的不核 sha256。"""
    home, saved_home = real_prompt_home(), os.environ.get("HOME")
    if home is not None:
        os.environ["HOME"] = home
    try:
        return "ran", real_prompt_cells_judged(directory, project_root)
    finally:
        if home is not None:
            if saved_home is None:
                os.environ.pop("HOME", None)
            else:
                os.environ["HOME"] = saved_home

def real_prompt_cells_judged(directory, project_root):
    """real_prompt_cells 逐份判的那一段 → 格的列表。"""
    cells = []
    for name, digest, kind, name_refused, form_refused, why in REAL_PROMPT_EXPECTATIONS:
        label = f"真提示:{name}（{kind}，{why}）"
        try:
            with open(os.path.join(directory, name), "rb") as handle:
                raw = handle.read()
        except OSError as error:
            cells.append((label, True, True) if break_switch_is("real-prompts-absent-green") else
                         (label, (digest, name_refused, form_refused), f"读不了（钩子样本目录 {directory} 不在或少了这一份）：{error}"))
            continue
        prompt = raw.decode("utf-8", errors="replace")
        got = (digest if break_switch_is("real-prompts-digest-unchecked") else hashlib.sha256(raw).hexdigest(), report_file_name_verdict(kind, prompt, project_root) is not None,
               three_way_report_verdict(kind, prompt, project_root) is not None)
        cells.append((label, (digest, name_refused, form_refused), got))
    return cells

def real_prompt_sources(directory):
    """⑳ 样本目录里 sources.tsv 钉的 [(文件名, sha256)]，按文件名排序（# 起头的行与空行不算）；读不了交 None。"""
    try:
        with open(os.path.join(directory, REAL_PROMPT_SOURCES), encoding="utf-8") as handle:
            rows = [line.rstrip("\n").split("\t") for line in handle if line.strip() and not line.startswith("#")]
    except OSError:
        return None
    return sorted((row[0], row[1] if len(row) > 1 else "") for row in rows)

def selftest(hook_dir):
    os.environ[SELFTEST_MARK_VARIABLE] = SELFTEST_MARK_CONTENT   # 自证起的钩子子进程（环境原样继承）靠它认：弄坏开关只在自证里认
    if runner_dispatch_guard_prompt_checks.prompt_scratch_line is None:
        print(f"  ✗ 自检读不到草稿目录一行的认法 {os.path.join(hook_dir, 'lib_prompt_scratch_line.py')}：{prompt_scratch_line_error!r}")
        print("    → 怎么办：恢复那份库（「草稿目录：」一行怎么认只在那里，⑯ ⑰ 与完工清理闸、交回闸共用），再跑 --selftest")
        return 1
    if work_roots is None:
        print(f"  ✗ 自检读不到工作根的定义 {os.path.join(hook_dir, 'lib_work_root.py')}：{work_roots_error!r}")
        print("    → 怎么办：恢复那份库（工作根与会话临时目录根只在那里定义），SINGLEFS_WORK_ROOT_BREAK 只写它认的值，再跑 --selftest")
        return 1
    if runner_dispatch_guard_prompt_checks.three_way_legs is None:
        print(f"  ✗ 自检读不到三方腿类型表的读法 {os.path.join(hook_dir, 'lib_three_way_legs.py')}：{three_way_legs_error!r}")
        print("    → 怎么办：恢复那份库（三方腿类型表只登记在定义 frontmatter 的 leg: three-way，读法只在那份库里），再跑 --selftest")
        return 1
    if shell_words is None:
        print(f"  ✗ 自检读不到共用切词模块 {os.path.join(hook_dir, 'lib_shell_words.py')}：{shell_words_error!r}")
        print("    → 怎么办：恢复那份库（引号区间含不含某个位置只在那里判，三个钩子共用），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp()
    write_batch_record(work)   # ⑱ 批次一行点名的调度记录
    not_run = []  # 本次未跑的格（⑳ 真提示目录不在这类）：不记绿，成功那句里逐个列出，GATE_NOT_RUN_FILE 在时各追加一行
    other_directories = []  # 快照那一条另起的临时目录，finally 里一起删
    state_root = tempfile.mkdtemp()  # 实现员登记放自检自己的状态目录，不碰真会话的
    other_directories.append(state_root)
    os.environ["DISPATCH_GUARD_STATE_ROOT"] = state_root
    saved_detections = os.environ.get("AGENT_HOOK_DETECTIONS")
    os.environ["AGENT_HOOK_DETECTIONS"] = os.path.join(state_root, "detections.jsonl")   # 起钩子本身的那几格记的检出落在自检自己的目录里，不进真会话的检出记录
    try:
        os.makedirs(os.path.join(work, ".claude", "kb", "experiments"))
        # E153 的实验页：原登记跑过、第 2 次重跑跑过一段（标题里写着），第 3 次重跑的登记还没跑过（⑪ 按这一份登记有没有过一段判续做）
        open(os.path.join(work, ".claude", "kb", "experiments", "153-样本.md"), "w").write("## E153 样本 —— 部分已跑\n\n### 第 2 次重跑（自检样本）\n\n第一段的读数。\n")
        first = "跑前登记：research/prompts/e160-preregistration.md\n"
        again = "跑前登记：research/prompts/e153-preregistration.md\n"
        # 默认给每个样本提示补上开工先读共用约束与规格检查放行行，实现员再补一行各不相同的要动文件：
        # 这样各条样本只判它自己要判的那一条；要判那几条本身的样本用 footer 换掉或关掉
        writer_case_counter = [0]
        def case_in(project_root, label, subagent_type, prompt, want, footer=True, report=True, batch=True, ignored=(), standing=True):
            footer_text = FOOTER if footer is True else (footer or "")
            text = prompt + ("\n" + footer_text if footer_text else "")
            if standing and footer_text and subagent_type in WRITER_GUARDED_AGENTS and "不碰" not in text:   # ⑱ 不碰一行：默认补「不碰：无」，要判缺行的样本传 standing=False
                text += ("" if text.endswith("\n") else "\n") + SELFTEST_UNTOUCHED_LINE
            if (standing and footer_text and agent_definition_fields(project_root, subagent_type) is not None
                    and not ADDRESSED_HEADING.search(text)):   # ⑱ 关的是哪几条：有定义的类型默认补一节，要判缺节的样本传 standing=False
                text += ("" if text.endswith("\n") else "\n") + SELFTEST_ADDRESSED_SECTION
            write_batch_record(project_root)
            if batch and subagent_type in BATCH_LINE_AGENTS and "批次" not in text:   # ⑱ 那十种要批次一行：默认补一行点名样本仓里那份记录，要判缺行的样本传 batch=False
                text += ("" if text.endswith("\n") else "\n") + SELFTEST_BATCH_LINE
            if subagent_type == "implementation-writer" and footer_text and "要动的 crates 文件" not in text:
                writer_case_counter[0] += 1
                text += f"要动的 crates 文件：crates/selftest/src/case{writer_case_counter[0]}.rs\n"
            if report and subagent_type in three_way_report_types(project_root) and "报告" not in text:   # ⑰ 三方腿要报告行：默认补一行合形态的，要判缺行的样本传 report=False
                text += ("" if text.endswith("\n") else "\n") + "报告：research/prompts/selftest-r1-opus-output.md\n"
            hook_input = {"tool_name": "Agent", "tool_input": {"subagent_type": subagent_type, "prompt": text}}
            if ignored:   # 只判别的几条的样本：ignored 点名的那几条（条名）不算，它们各有自己的格（⑰ 三方腿形态的样本不带行首「报告：」一行，那一条另判）
                return (label, want, 2 if any(name not in ignored for name, _ in judge_dispatch(hook_input, project_root, hook_dir)[0]) else 0)
            return (label, want, runner_dispatch_guard_locks_and_verdicts.decide(hook_input, project_root, hook_dir)[0])
        def case(label, subagent_type, prompt, want, footer=True, report=True, batch=True, ignored=(), standing=True):
            return case_in(work, label, subagent_type, prompt, want, footer, report, batch, ignored, standing)
        # 快照那一条的样本仓：r-open 开着（有正文、没判决）；r-closed 已判完；r-nobody 有快照没正文（普查一类）；r-odd 开着，清单里全是认不出的行
        def write_sample(project_root, relative_path, content):
            path = os.path.join(project_root, relative_path)
            os.makedirs(os.path.dirname(path), exist_ok=True)
            with open(path, "w", encoding="utf-8") as sample_file:
                sample_file.write(content)
        # 三方腿类型表登记在定义的 frontmatter（leg: three-way）：样本仓 work 里三种腿各一份定义，三方报告那几格才按三方腿判
        for leg_name in ("three-way-forward", "three-way-attack", "three-way-defense"):
            write_sample(work, f".claude/agents/{leg_name}.md", f"---\nname: {leg_name}\nleg: three-way\n---\n")
        digest = "0123456789abcdef" * 4
        write_sample(work, "research/prompts/_r-open-body.md", "# r-open 正文\n")
        write_sample(work, "research/prompts/r-open-snapshot/kb-sha256.txt",
                     f"{digest}  ./.claude/kb/checks-owed.md\n{digest}  ./.claude/kb/decisions/18-块里携带什么信息.md\n")
        write_sample(work, "research/prompts/r-open-snapshot/crates-sha256.txt",
                     f"{digest}  tree/crates/singlefs-core/src/mount.rs\n{digest}  defs/.claude/agents/crash-verifier.md\n")
        write_sample(work, "research/prompts/_r-closed-body.md", "# r-closed 正文\n")
        write_sample(work, "research/prompts/r-closed-snapshot/sha256sums.txt", f"{digest}  .claude/kb/decisions/03-空间分配.md\n")
        write_sample(work, "research/prompts/r-closed-main-verification.md", "# r-closed 判决\n")
        write_sample(work, "research/prompts/r-nobody-snapshot/sources.sha256", f"{digest}  crates/singlefs-core/src/allocator.rs\n")
        write_sample(work, "research/prompts/_r-odd-body.md", "# r-odd 正文\n")
        write_sample(work, "research/prompts/r-odd-snapshot/odd-sha256.txt",
                     f"快照时刻\n# 注释\n{digest}  README.md\n{digest}  research/prompts/r-odd-model.md\nnot-a-hash  .claude/kb/pitfalls.md\n")
        # 另一个样本仓：一份清单读不了（悬空的符号链接），另一份读得了
        unreadable_work = tempfile.mkdtemp()
        write_batch_record(unreadable_work)   # ⑱ 批次一行点名的调度记录
        other_directories.append(unreadable_work)
        write_sample(unreadable_work, "research/prompts/_r-broken-body.md", "# r-broken 正文\n")
        write_sample(unreadable_work, "research/prompts/r-broken-snapshot/kb-sha256.txt", f"{digest}  .claude/kb/checks-owed.md\n")
        os.symlink(os.path.join(unreadable_work, "不在的文件"), os.path.join(unreadable_work, "research/prompts/r-broken-snapshot/gone-sha256.txt"))
        # 规格文件放在工作根下（不看弄坏开关，configured_work_root），另一份放会话临时目录根下；派发闸只读这两处，自检的 TMPDIR 之外，finally 里删
        new_root, session_temporary_root = work_roots.configured_work_root(), work_roots.claude_session_temporary_root()
        home = os.path.expanduser("~")
        home_new_root = "~" + new_root[len(home):] if new_root.startswith(home + "/") else new_root
        for scratch_root in (new_root, session_temporary_root):
            os.makedirs(scratch_root, exist_ok=True)
        specification_directory = tempfile.mkdtemp(prefix="runner-dispatch-guard-selftest-", dir=new_root)
        other_directories.append(specification_directory)
        session_temporary_specification_directory = tempfile.mkdtemp(prefix="runner-dispatch-guard-selftest-", dir=session_temporary_root)
        other_directories.append(session_temporary_specification_directory)
        session_temporary_specification_path = os.path.join(session_temporary_specification_directory, "spec.md")
        write_sample(session_temporary_specification_directory, "spec.md", "1. 文件：.claude/kb/checks-owed.md\n   旧串：| C120 | …\n   新串：| C120 | …\n")
        specification_path = os.path.join(specification_directory, "spec.md")
        write_sample(specification_directory, "spec.md", "1. 文件：.claude/kb/checks-owed.md\n   旧串：| C120 | …\n   新串：| C120 | …\n")
        missing_specification_path = os.path.join(specification_directory, "missing-spec.md")
        collided = "规格：改 `.claude/kb/checks-owed.md` 里 C120 那一行。\n" + FOOTER
        forwarded_refusal = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}, work, hook_dir)[1] or ""
        handover_report = os.path.join(specification_directory, "previous-runner", "result.md")   # ⑪ 续做的「承接：」点名的上一段报告（在工作根下，真有这一份）
        write_sample(specification_directory, os.path.join("previous-runner", "result.md"), "# 上一段执行员的报告（自检样本）\n")
        handover_line = f"承接：{handover_report}\n"
        cases = [
            case("不是执行员", "experiment-designer", "随便", 0),
            case("只修锚点", "experiment-runner", "只修锚点：表 e153", 0),
            case("第一段点名了岔路", "experiment-runner", first + "这一段回答的岔路：岔路 1、岔路 4\n", 0),
            case("第一段没点名岔路", "experiment-runner", first, 2),
            case("第一段岔路写无", "experiment-runner", first + "这一段回答的岔路：无\n", 2),
            case("续做点名了还差的行", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n" + handover_line, 0),
            case("续做没写还差", "experiment-runner", again + "这一段回答的岔路：岔路 1\n" + handover_line, 2),
            case("续做还差写无", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：无\n" + handover_line, 2),
            case("重跑登记已有过一段的算续做，没写还差拒", "experiment-runner", "research/prompts/e153-r2-prereg.md\n这一段回答的岔路：岔路 3\n" + handover_line, 2),
            case("新的重跑登记第一段只要岔路放行", "experiment-runner", "跑前登记：research/prompts/e153-r3-prereg.md\n这一段回答的岔路：岔路 3\n", 0),
            case("重跑登记已有过一段、写了还差与承接放行", "experiment-runner",
                 "research/prompts/e153-r2-prereg.md\n这一段回答的岔路：岔路 3\n上一段岔路表里还差：岔路 3 的支线\n" + handover_line, 0),
            case("续做没写承接拒", "experiment-runner", again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n", 2),
            case("续做承接指到不在的报告拒", "experiment-runner",
                 again + "这一段回答的岔路：岔路 1\n上一段岔路表里还差：岔路 1 的崩溃支线\n" + f"承接：{specification_directory}/not-there/result.md\n", 2),
        ]
        # ── 派发提示的结构：叫子 agent 跑项目代码的句子、共用约束、定义要的输入、写范围、实现员文件与登记、并发、书记员规格、设计员准入、变异表 ──
        defs_work = tempfile.mkdtemp()
        write_batch_record(defs_work)   # ⑱ 批次一行点名的调度记录
        other_directories.append(defs_work)
        write_sample(defs_work, ".claude/agents/three-way-local-attack.md", "---\nname: three-way-local-attack\nmodel: sonnet\nrequired-inputs: 禁读清单, 草稿目录, 提示文件\n---\n")
        write_sample(defs_work, ".claude/agents/three-way-verifier.md", "---\nname: three-way-verifier\nmodel: sonnet  # 注释\nrequired-inputs: 快照, 草稿目录, 报告路径|-verifier-output.md\n---\n")
        write_sample(defs_work, ".claude/agents/experiment-runner.md", "---\nname: experiment-runner\ndescription: 实验执行员：样本\nmodel: opus\nrequired-inputs: 草稿目录\n---\n")
        write_sample(defs_work, ".claude/agents/implementation-writer.md", "---\nname: implementation-writer\ndescription: 实现员：样本\nmodel: opus\n---\n")
        write_sample(defs_work, ".claude/agents/mutation-triage.md", "---\nname: mutation-triage\nmodel: sonnet\n---\n")
        write_sample(defs_work, ".claude/agents/clerk.md", "---\nname: clerk\nmodel: sonnet\nrequired-inputs: 草稿目录, 报告\n---\n")
        write_sample(defs_work, ".claude/agents/scope-probe.md", "---\nname: scope-probe\nmodel: sonnet\n---\n")
        write_sample(defs_work, ".claude/agents/tooling-writer.md", "---\nname: tooling-writer\nmodel: opus\n---\n")   # ⑤ 工具员撞文件那几格（有定义，不另写开工先读）
        write_sample(defs_work, ".claude/hooks/agent-write-scope.tsv",
                     "# 样本\nexperiment-runner\tresearch/results/**\t样本\nexperiment-runner\tresearch/prompts/e*-r*-prereg.md\t样本\nexperiment-runner\t$SINGLEFS_WORK_ROOT/**\t样本\n"
                     "implementation-writer\tcrates/**\t样本\nimplementation-writer\t$SINGLEFS_WORK_ROOT/**\t样本\nscope-probe\tcrates/**\t样本：写范围里没有工作根\n")
        batch_lines = (SELFTEST_BATCH_LINE   # defs_work 里建了批次一行点名的那份记录
                      + SELFTEST_UNTOUCHED_LINE + SELFTEST_ADDRESSED_SECTION)   # ⑱ 不碰一行与关的是哪几条那一节（各类型都带，只有判它们的样本不带）
        defs_case = lambda label, subagent_type, prompt, want: case_in(defs_work, label, subagent_type, batch_lines + prompt, want, footer=False)
        cases += [
            # 提示正文里叫子 agent 跑项目代码的句子（①，project_code_run_sentences）：许可的说法、要求的说法一样拒，否定句、归主 agent 的、交回的写法、「」里的转述放行
            # （开关 project-code-run-sentences-unjudged 下「拒」的格都红）
            case("项目代码:提示里没有额外的固定行照放", "implementation-writer", "改完交回。", 0),
            case("项目代码:正文许可跑 cargo test -p singlefs-verification 拒", "clerk", "可以跑 `cargo test -p singlefs-verification --lib` 看绿。", 2),
            case("项目代码:正文要求跑崩溃点重放入口带 --item 拒", "clerk", "请跑 `bash scripts/crash/crash-point-replay.sh --item x`。", 2),
            # 重型入口名起头在距离里、整条比距离长（开关 heavy-entry-start-unjudged 下第一格与上一格红）；否定句照放行（否定先于重型入口的判法）
            case("项目代码:正文要求跑形式化那一道带 --item 拒", "clerk", "请跑 `bash .claude/gate.d/formal-verification.sh --item x /root`。", 2),
            case("项目代码:否定句不跑崩溃点重放入口放行", "clerk", "不跑 `bash scripts/crash/crash-point-replay.sh --item x`，写进报告交回。", 0),
            case("项目代码:正文叫它自己跑 --selftest 拒", "clerk", "自己跑一下 --selftest 核一遍。", 2),
            case("项目代码:正文许可编译 cargo build 拒", "clerk", "允许跑 `cargo build -p singlefs-core` 看编不编得过。", 2),
            case("项目代码:正文直接写跑门禁阶段拒", "clerk", "改完跑 `bash .claude/gate.d/doc-text.sh .` 核一遍。", 2),
            case("项目代码:正文叫它执行样本档拒", "clerk", "执行样本档，逐份看红绿。", 2),
            case("项目代码:否定句不跑 cargo test 放行", "clerk", "不跑 cargo test，也不跑 --selftest。", 0),
            case("项目代码:归主 agent 的句子放行", "clerk", "cargo test 由主 agent 提交时跑。", 0),
            case("项目代码:同一句里前面提到主 agent 的子句放行", "clerk", "要主 agent 跑：`cargo test -p singlefs-verification --lib`，你不跑。", 0),
            case("项目代码:「」里转述的原句放行", "clerk", "两腿提示写过「可以跑 cargo test -p singlefs-verification」，这一句不许再写。", 0),
            case("项目代码:交回的写法放行", "clerk", "要跑的命令写进报告「要主 agent 跑的命令」一节交回。", 0),
            case("项目代码:读文件、改文件、查清单的说法放行", "clerk", "读 `.claude/hooks/lib_project_code_runs.py`，用 Edit 改，`ls scripts/kb/` 看有哪些工具。", 0),
            # 认领行 --purpose 后面的用途文字不判（without_claim_purpose）：引号里、没加引号写到行尾的、㉒ 与 ⑲ 的逐行判句子都先摘；认领行之外的句子照拒
            # （开关 claim-purpose-judged 下除「认领行之外」那一格外都红）
            case("项目代码:认领行 --purpose 引号里写着复跑、自证、弄坏开关，不当叫子 agent 跑", "clerk",
                 "认领：bash scripts/hosts/work-directory-ownership.sh claim /x/purpose-sample --purpose 'task-server 批 58：全仓开关复跑里 task_server_lib.py 那 82 处——自证在负载下的时限红、点名写错、两条没红的开关'\n", 0),
            case("项目代码:认领行之外写请跑 --selftest 照拒", "clerk",
                 "认领：bash scripts/hosts/work-directory-ownership.sh claim /x/purpose-sample --purpose '自证在负载下的时限红'\n请跑 `python3 scripts/gate/sample.py --selftest` 核一遍。\n", 2),
            case("项目代码:认领行 --purpose 没加引号、写到行尾也不判", "clerk",
                 "认领：bash scripts/hosts/work-directory-ownership.sh claim /x/purpose-sample --purpose 复跑自证与弄坏开关的用途\n", 0),
            case("任务服务:认领行 --purpose 里写经 task-server/task submit 交自证，不当叫子 agent 交任务", "clerk",
                 "认领：bash scripts/hosts/work-directory-ownership.sh claim /x/purpose-sample --purpose '经 task-server/task submit 交自证'\n", 0),
            case("门禁阶段:认领行 --purpose 里写跑门禁阶段，不当叫子 agent 起", "clerk",
                 "认领：bash scripts/hosts/work-directory-ownership.sh claim /x/purpose-sample --purpose '改完跑 bash .claude/gate.d/doc-text.sh 的用途'\n", 0),
            case("项目代码:文本工具 scripts/kb/replace-once.py 放行", "clerk", "跑 `python3 scripts/kb/replace-once.py 文件 旧 新` 定点改。", 0),
            case_in(defs_work, "共用约束:通用 agent 没写开工先读", "general-purpose", batch_lines + "通用已判：要 WebSearch 查外部文档\n查一个文件。", 2, footer=False),
            case_in(defs_work, "共用约束:通用 agent 写了开工先读", "general-purpose", batch_lines + "通用已判：要 WebSearch 查外部文档\n开工先读 `.claude/agent-common.md`。查一个文件。", 0, footer=False),
            case_in(defs_work, "通用:general-purpose 没写放行行的拒", "general-purpose", batch_lines + "开工先读 `.claude/agent-common.md`。查一个文件。", 2, footer=False),
            case_in(defs_work, "通用:claude 类型同样拒", "claude", batch_lines + "开工先读 `.claude/agent-common.md`。查一个文件。", 2, footer=False),
            case_in(defs_work, "通用:Explore 放行", "Explore", batch_lines + "开工先读 `.claude/agent-common.md`。找文件。", 0, footer=False),
            case_in(defs_work, "骨架:占位没填的拒", "Explore", batch_lines + "开工先读 `.claude/agent-common.md`。\n草稿目录：<填 草稿目录>\n找文件。", 2, footer=False),
            case_in(defs_work, "骨架:占位填完放行", "Explore", batch_lines + f"开工先读 `.claude/agent-common.md`。\n草稿目录：{home_new_root}/x/\n找文件。", 0, footer=False),
            defs_case("通用:clerk 有定义放行", "clerk", f"草稿目录 {home_new_root}/c/，报告 {home_new_root}/c/result.md", 0),
            case("书记员:给判决不给规格放行（写回员自己起草）", "kb-scribe", "判决 research/prompts/r-closed-main-verification.md，条目 C120。", 0,
                 footer="开工先读 `.claude/agent-common.md`\n"),
            defs_case("输入:本地攻方缺禁读清单（09-23 那一次）", "three-way-local-attack", f"提示文件 research/prompts/r-local-attack.md，草稿目录 {home_new_root}/r/", 2),
            defs_case("输入:本地攻方齐了", "three-way-local-attack", f"提示文件 research/prompts/r-local-attack.md，草稿目录 {home_new_root}/r/，禁读清单：别的腿的产出", 0),
            defs_case("输入:核查员缺草稿目录（09-24 那一次）", "three-way-verifier", "快照 research/prompts/r-snapshot/，报告路径 research/prompts/r-verifier-output.md", 2),
            defs_case("输入:核查员缺快照（09-25 那一次）", "three-way-verifier", f"草稿目录 {home_new_root}/v/，报告 research/prompts/r-verifier-output.md", 2),
            defs_case("输入:核查员同义词认得", "three-way-verifier", f"快照 x，草稿目录 {home_new_root}/v/，写 research/prompts/r-verifier-output.md", 0),
            defs_case("写范围:执行员报告写进 research/prompts（09-26 那一次）", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n报告路径：research/prompts/e142-r15-runner-result.md\n这一段回答的岔路：岔路 1\n", 2),   # 原样是 …-runner-report.md；文件名改掉 report，只判写范围这一条（⑰ 不叠上来）
            defs_case("写范围:「报告：」一行点名写范围外的路径也拒", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n报告：research/prompts/e142-r15-runner-result.md\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:「报告：」一行点名工作根下的路径放行", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n报告：{home_new_root}/e/result.md\n这一段回答的岔路：岔路 1\n", 0),            defs_case("写范围:执行员去更新问题单", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n更新 research/prompts/m2-rbf1-e142-questions.md 的状态\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:实现员写进 invariants.md", "implementation-writer",
                      "要动的 crates 文件：crates/a/src/one.rs\n顺手写进 .claude/kb/invariants.md 的状态列\n", 2),
            case_in(defs_work, "写范围:登记了工作根的类型报告写进会话临时目录根拒", "experiment-runner",
                    batch_lines + f"草稿目录 {session_temporary_root}/e/\n报告写进 {session_temporary_root}/e/result.md\n这一段回答的岔路：岔路 1\n", 2,
                    footer=False, ignored=("报告落点",)),   # 只判写范围这一条：会话临时目录根下的报告另被报告落点（⑱）拒，那一条有自己的格
            defs_case("写范围:否定句里的路径不判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n不要写进 research/prompts/x-report.md\n这一段回答的岔路：岔路 1\n", 0),
            # 转述别人要写什么的不判（scope_mention_aimed_elsewhere）：动词所在子句里点名了主 agent、别的类型的称呼，列表在「做成的事：」之下；对子 agent 说的、写给它自己类型的称呼、别的执行者在前一个子句里的照判
            defs_case("写范围:主 agent 之后写进的路径是转述不判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n主 agent 收齐之后把判决写进 research/prompts/x-main-verification.md。\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:别的类型的称呼之后写进的路径是转述不判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n实现员之后把补丁写进 crates/a/src/one.rs，你只写登记。\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:做成的事清单里的更新不判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n这一阶段做成的事：\n- 更新 .claude/kb/tooling.md 的钩子一节\n- 改了检出钩子\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:称呼之后又对子 agent 说话的写进仍判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n主 agent 收齐之后你把判决写进 research/prompts/x-main-verification.md。\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:写给这个类型自己的称呼仍判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n执行员把判决写进 research/prompts/x-main-verification.md。\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:别的执行者在前一个子句里、动词所在子句里没有的仍判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n主 agent 另派实现员收尾，结论写进 research/prompts/x-main-verification.md。\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:按主 agent 给的格式写进仍判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n按主 agent 给的格式把判决写进 research/prompts/x-main-verification.md。\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:要做的事清单里的更新仍判", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n这一阶段要做的事：\n- 更新 research/prompts/x-main-verification.md 的状态\n这一段回答的岔路：岔路 1\n", 2),
            defs_case("写范围:报告写进工作根放行（~ 写法）", "experiment-runner",
                      f"草稿目录 {home_new_root}/e/\n报告写进 {home_new_root}/e/result.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:报告写进工作根放行（展开的绝对路径）", "experiment-runner",
                      f"草稿目录 {new_root}/e/\n报告写进 {new_root}/e/result.md\n这一段回答的岔路：岔路 1\n", 0),
            defs_case("写范围:没登记工作根的类型报告写进工作根拒", "scope-probe", f"草稿目录 {home_new_root}/p/\n报告写进 {home_new_root}/p/result.md\n", 2),
            case_in(defs_work, "写范围:没登记工作根的类型报告写进会话临时目录根拒", "scope-probe", batch_lines + f"草稿目录 {session_temporary_root}/p/\n报告写进 {session_temporary_root}/p/result.md\n", 2,
                    footer=False, ignored=("报告落点",)),   # 只判写范围这一条：会话临时目录根下的报告另被报告落点（⑱）拒，那一条有自己的格
            defs_case("变异分诊:没给表", "mutation-triage", "跑 2697 条广谱变异。", 2),
            defs_case("变异分诊:给了表", "mutation-triage", "跑 crates/mutations.tsv 里 m2-rollback 那几条。", 0),
            defs_case("执行员:只补落产物放行", "experiment-runner", f"草稿目录 {home_new_root}/e/\n只补落产物：E142 那份。", 0),
        ]
        # 实现员：要动的 crates 文件一行、件数上限、与在跑的实现员撞文件（登记在自检自己的状态目录里）
        state_root = tempfile.mkdtemp()
        other_directories.append(state_root)
        os.environ["DISPATCH_GUARD_STATE_ROOT"] = state_root
        session_work = tempfile.mkdtemp()
        write_batch_record(session_work)   # ⑱ 批次一行点名的调度记录
        other_directories.append(session_work)
        def session_case(label, subagent_type, prompt, want, transcript_name, model=None, project_root=defs_work):
            tool_input = {"subagent_type": subagent_type, "prompt": batch_lines + prompt}
            if model:
                tool_input["model"] = model
            hook_input = {"tool_name": "Agent", "tool_input": tool_input, "session_id": transcript_name,
                          "transcript_path": os.path.join(session_work, transcript_name + ".jsonl")}
            return (label, want, runner_dispatch_guard_locks_and_verdicts.decide(hook_input, project_root)[0])
        def write_session(transcript_name, records):
            with open(os.path.join(session_work, transcript_name + ".jsonl"), "a", encoding="utf-8") as handle:
                for record in records:
                    handle.write(json.dumps(record, ensure_ascii=False) + "\n")
        def stamp(seconds_ago):
            return datetime.fromtimestamp(time.time() - seconds_ago, timezone.utc).isoformat().replace("+00:00", "Z")
        def launch(agent_id, model, prompt, seconds_ago=600):
            return {"timestamp": stamp(seconds_ago), "toolUseResult": {"status": "async_launched", "agentId": agent_id, "resolvedModel": model, "prompt": prompt}}
        def handed_back(agent_id, seconds_ago=60):
            return {"timestamp": stamp(seconds_ago), "content": f'<agent-message from="{agent_id}">\n[Subagent hand-back] 报告'}
        def notification(agent_id, status, summary="", seconds_ago=60):
            return {"timestamp": stamp(seconds_ago), "content": f"<task-notification>\n<task-id>{agent_id}</task-id>\n<status>{status}</status>\n<summary>{summary}</summary>\n</task-notification>"}
        writer_one = "要动的 crates 文件：crates/a/src/one.rs crates/a/src/two.rs\n"
        write_session("registry", [])
        cases.append(session_case("实现员:第一件登记下来", "implementation-writer", writer_one, 0, "registry"))
        cases.append(session_case("实现员:第二件撞了在跑的那件", "implementation-writer", "要动的 crates 文件：crates/a/src/two.rs、crates/a/src/three.rs\n", 2, "registry"))
        cases.append(session_case("实现员:第三件不撞", "implementation-writer", "要动的 crates 文件：crates/a/src/three.rs\n", 0, "registry"))
        write_session("registry", [launch("a00000000000000a1", "claude-opus-5", batch_lines + writer_one), handed_back("a00000000000000a1")])
        cases.append(session_case("实现员:第一件交回之后不再撞", "implementation-writer", "要动的 crates 文件：crates/a/src/two.rs\n", 0, "registry"))
        cases.append(session_case("实现员:没写要动的文件", "implementation-writer", "改挂载准入。", 2, "registry"))
        cases.append(session_case("实现员:文件多也不拒（份数不设上限）", "implementation-writer", "要动的 crates 文件：" + " ".join(f"crates/b/src/f{n}.rs" for n in range(12)) + "\n", 0, "registry"))
        cases.append(session_case("实现员:带 mutations.tsv 的一件登记下来", "implementation-writer", "要动的 crates 文件：crates/mutations.tsv crates/a/src/five.rs\n", 0, "registry"))
        cases.append(session_case("实现员:mutations.tsv 与在跑的重叠不算撞", "implementation-writer", "要动的 crates 文件：crates/mutations.tsv crates/a/src/six.rs\n", 0, "registry"))
        # ⑤ 推广到工具员（拆分提案第十九节 ②）：工具员「要改的文件」登记进同一份表，工具员对工具员、工具员对实现员都算撞；
        # 开关 writer-collision-implementation-only（只判实现员）下「两个工具员撞」「工具员撞在跑的实现员」「实现员撞在跑的工具员」三格红
        write_session("tooling", [])
        cases.append(session_case("工具员:第一件登记下来", "tooling-writer", "要改的文件：.claude/hooks/sample-guard.sh、scripts/agents/sample.py\n", 0, "tooling"))
        cases.append(session_case("工具员:两个工具员撞文件拒", "tooling-writer", "要改的文件：scripts/agents/sample.py\n", 2, "tooling"))
        cases.append(session_case("工具员:一节「## 要改的文件」下的列表照认，撞了拒", "tooling-writer", "## 要改的文件\n\n- .claude/hooks/sample-guard.sh\n\n## 出口\n\n做完\n", 2, "tooling"))
        cases.append(session_case("工具员:目录罩住在跑工具员的文件也算撞", "tooling-writer", "要改的文件：scripts/agents/\n", 2, "tooling"))
        cases.append(session_case("工具员:实现员登记下来", "implementation-writer", "要动的 crates 文件：crates/a/src/shared.rs\n", 0, "tooling"))
        cases.append(session_case("工具员:工具员撞在跑的实现员拒", "tooling-writer", "要改的文件：crates/a/src/shared.rs\n", 2, "tooling"))
        cases.append(session_case("工具员:另一个工具员登记下来", "tooling-writer", "要改的文件：crates/b/src/tooling-owned.rs\n", 0, "tooling"))
        cases.append(session_case("工具员:实现员撞在跑的工具员拒", "implementation-writer", "要动的 crates 文件：crates/b/src/tooling-owned.rs\n", 2, "tooling"))
        cases.append(session_case("工具员:不撞放行", "tooling-writer", "要改的文件：scripts/gate/other.py\n", 0, "tooling"))
        # 并发：上限加 2 个派出去、1 个交回、1 个失败，在跑的正好是上限（不分模型）
        write_session("busy", [launch(f"a000000000000b{n:02d}", "claude-opus-5-5[1m]" if n % 2 else "claude-sonnet-5", f"活 {n}") for n in range(SUBAGENT_CONCURRENCY_LIMIT + 2)]
                      + [handed_back("a000000000000b00"), notification("a000000000000b01", "failed")])
        write_session("absent", [launch(f"a00000000000ab{n:02d}", "claude-opus-5", f"活 {n}") for n in range(SUBAGENT_CONCURRENCY_LIMIT)]
                      + [{"timestamp": stamp(30), "toolUseResult": "Error: No task found with ID: a00000000000ab00. Running background agents: a00000000000ab01 (活 1)"},
                         {"timestamp": stamp(20), "toolUseResult": "Error: No task found with ID: a00000000000ab09. Running background agents: a00000000000ab01 (活 1)"}])
        write_session("light", [launch(f"a000000000000c{n:02d}", "claude-opus-5", f"活 {n}") for n in range(SUBAGENT_CONCURRENCY_LIMIT - 2)]
                      + [launch("a000000000000c99", "claude-sonnet-5", "别的")])
        cases += [
            session_case(f"并发:已有 {SUBAGENT_CONCURRENCY_LIMIT} 个在跑再派", "experiment-runner", f"草稿目录 {home_new_root}/e/\n只补落产物", 2, "busy"),
            session_case("并发:派 sonnet 一样算", "experiment-runner", f"草稿目录 {home_new_root}/e/\n只补落产物", 2, "busy", model="sonnet"),
            session_case("并发:TaskStop 报找不到的不算在跑", "experiment-runner", f"草稿目录 {home_new_root}/e/\n只补落产物", 0, "absent"),
            session_case(f"并发:{SUBAGENT_CONCURRENCY_LIMIT - 1} 个在跑放行", "experiment-runner", f"草稿目录 {home_new_root}/e/\n只补落产物", 0, "light"),
            session_case("并发:没有定义的类型一样算", "undefined-adhoc-type", "开工先读 `.claude/agent-common.md`。", 2, "busy"),
        ]
        # 书记员的规格：点名的规格文件交给 kb-spec-check.py（仓根是样本仓 work）
        write_sample(work, ".claude/kb/tooling.md", "# 工具\n\n| 甲 | 旧值 |\n\n## 历史版本\n")
        good_spec = os.path.join(specification_directory, "good-spec.json")
        write_sample(specification_directory, "good-spec.json", json.dumps([{"file": ".claude/kb/tooling.md", "old": "| 甲 | 旧值 |", "new": "| 甲 | 新值 |", "basis": "样本"}], ensure_ascii=False))
        bad_spec = os.path.join(specification_directory, "bad-spec.json")
        write_sample(specification_directory, "bad-spec.json", json.dumps([{"file": ".claude/kb/tooling.md", "old": "| 乙 | 不在的行 |", "new": "| 乙 | 新 |", "basis": "样本"}], ensure_ascii=False))
        no_valve = batch_lines + "开工先读 `.claude/agent-common.md`\n"
        cases += [
            case("规格:点名的规格检查过了", "kb-scribe", f"逐条规格在 {good_spec}，照写。", 0, footer=no_valve),
            case("规格:点名的规格旧串不在", "kb-scribe", f"逐条规格在 {bad_spec}，照写。", 2, footer=no_valve),
            case("规格:一份认得出的规格都没有", "kb-scribe", "照判决写回 C120。", 2, footer=no_valve),
            case("规格:写了放行行", "kb-scribe", "照判决写回 C120。\n规格检查已判：用户口述的定案，没有规格文件\n", 0, footer=no_valve),
        ]
        # 设计员的重跑登记：准入判单元键没变（假的 admission.py 退 77）就拒
        for name, code in (("admission-unchanged.py", 77), ("admission-changed.py", 0)):
            write_sample(work, name, f"import sys\nsys.exit({code})\n")
        # 退 2 一律拒：准入判法自己报的两种句子（照 scripts/gate/admission.py 的样子打在 stderr）出路是接准入；python 自己退 2（打不开脚本、用法错）与什么都没打的退 2 出路是先让判法能跑
        write_sample(work, "admission-unregistered.py", "import sys\nprint('  ✗ 准入登记里没有实验键 E999（.claude/gate.d/stage-inputs.tsv）', file=sys.stderr)\nsys.exit(2)\n")  # 样本造的拒绝（→ 出路不归它，被测的派发闸照它的退出码与标准错误判）
        write_sample(work, "admission-registration-wrong.py", "import sys\nprint('  ✗ E142 的准入登记有错：入口写错了', file=sys.stderr)\nsys.exit(2)\n")  # 样本造的拒绝（→ 出路不归它，被测的派发闸照它的退出码与标准错误判）
        write_sample(work, "admission-python-exit-2.py", "import sys\nprint(\"python3: can't open file 'admission.py': [Errno 2] No such file or directory\", file=sys.stderr)\nsys.exit(2)\n")
        write_sample(work, "admission-usage-exit-2.py", "import sys\nprint('  ✗ 用法：admission.py experiment <项目根> <实验键>', file=sys.stderr)\nsys.exit(2)\n")  # 样本造的拒绝（→ 出路不归它，被测的派发闸照它的退出码与标准错误判）
        write_sample(work, "admission-silent-exit-2.py", "import sys\nsys.exit(2)\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-unchanged.py")
        cases += [
            case("准入:单元键没变不写重跑登记", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2),
            case("准入:写了放行行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。\n准入已判：换了一条臂\n", 0),
            case("准入:不是重跑登记不判", "experiment-designer", "写跑前登记 research/prompts/e170-preregistration.md。", 0),
        ]
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-unregistered.py")
        cases += [
            case("准入:准入登记里没有这个实验键不写重跑登记", "experiment-designer", "写重跑登记 research/prompts/e999-r2-prereg.md。", 2),
            case("准入:准入登记里没有这个实验键、写了放行行", "experiment-designer", "写重跑登记 research/prompts/e999-r2-prereg.md。\n准入已判：换了一条臂\n", 0),
        ]
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-registration-wrong.py")
        cases.append(case("准入:这个实验的准入登记有错不写重跑登记", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2))
        # 判法本身跑不起来（脚本不在、python 自己退 2、用法错、什么都没打）：退 2 一律拒，写了放行行的放；拒绝说明的出路是先让 admission.py 能跑，不是接准入
        unrunnable_refusals = []
        for sample, label in (("admission-no-such-script.py", "不在"), ("admission-python-exit-2.py", "退 2 且是 python 自己退的（打不开脚本）"),
                              ("admission-usage-exit-2.py", "退 2 且输出不是准入判法的句子（用法错）"), ("admission-silent-exit-2.py", "退 2 且什么都没打")):
            os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, sample)
            cases.append(case(f"准入:admission.py {label}也拒", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2))
            unrunnable_refusals.append(designer_admission_verdict("写重跑登记 research/prompts/e142-r19-prereg.md。", work, hook_dir) or "")
        cases.append(case("准入:admission.py 退 2 且什么都没打、写了放行行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。\n准入已判：判法在修，用户知道\n", 0))
        cases.append(("准入:判法跑不起来的拒绝说明出路是先让 admission.py 能跑，不是接准入", [True] * 4,
                      ["先让 admission.py 能跑" in text and "准入已判" in text and "stage-inputs.tsv 接这个实验的准入" not in text for text in unrunnable_refusals]))
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-unregistered.py")
        registration_refusal = designer_admission_verdict("写重跑登记 research/prompts/e142-r19-prereg.md。", work, hook_dir) or ""
        cases.append(("准入:判法报了没登记的拒绝说明出路是接准入，不写先让判法能跑", True,
                      "stage-inputs.tsv 接这个实验的准入" in registration_refusal and "先让 admission.py 能跑" not in registration_refusal))
        # 退 0 而实验键拒算（假的那一份照 admission.py 的样子：stderr 一句拒算、stdout 产物头写 sha256=refused）：判不了「没变」，放行（记检出那一半在下面「没判全」那几格里）
        write_sample(work, "admission-key-refused.py",
                     "import sys\nprint('  ! E142 的实验键拒算（自证造的）：照放行，下一趟也判不了「没变」', file=sys.stderr)\nprint('E7INPUT name=input_fingerprint key=E142 sha256=refused units=3')\nsys.exit(0)\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-key-refused.py")
        cases.append(case("准入:admission.py 退 0 而实验键拒算照常放行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 0))
        # 环境里设着已删的强制重跑变量：admission.py 见到它照用法错退 2（假的那一份照它的样子打那一句与出路；没设就退 0、照放），派发闸不清它、照退 2 拒，
        #   拒绝说明的出路是从环境里去掉它、要重跑先改输入，不是接准入、也不是先让判法能跑
        write_sample(work, "admission-retired-variable.py",
                     f"import os, sys\nif '{ADMISSION_RETIRED_VARIABLE}' in os.environ:\n"
                     f"    print('  ✗ {ADMISSION_RETIRED_VARIABLE} 删了：没有越过「输入没变」的强制重跑，环境里设着它就什么都不做（用法错）', file=sys.stderr)\n"
                     f"    print('     → 怎么办：unset {ADMISSION_RETIRED_VARIABLE} 再来', file=sys.stderr)\n    sys.exit(2)\nsys.exit(0)\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-retired-variable.py")
        saved_retired_variable = os.environ.get(ADMISSION_RETIRED_VARIABLE)
        os.environ[ADMISSION_RETIRED_VARIABLE] = "自证造的残留理由"
        try:
            cases.append(case("准入:环境里设着已删的强制重跑变量时设计员派发被拒", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2))
            retired_variable_refusal = designer_admission_verdict("写重跑登记 research/prompts/e142-r19-prereg.md。", work, hook_dir) or ""
        finally:
            if saved_retired_variable is None:
                del os.environ[ADMISSION_RETIRED_VARIABLE]
            else:
                os.environ[ADMISSION_RETIRED_VARIABLE] = saved_retired_variable
        cases.append(("准入:已删的强制重跑变量那一拒的出路是从环境里去掉它、要重跑先改输入", True,
                      f"去掉 {ADMISSION_RETIRED_VARIABLE}" in retired_variable_refusal and "先改输入" in retired_variable_refusal
                      and "stage-inputs.tsv 接这个实验的准入" not in retired_variable_refusal and "先让 admission.py 能跑" not in retired_variable_refusal))
        # 前提没齐（假的 admission.py 照它的样子在 stderr 打一句 ✗ 与一条前提、退 3）也拒，写了放行行的放
        write_sample(work, "admission-precondition-missing.py",
                     "import sys\nprint('  ✗ 前提没齐，不许开跑 E142：共 1 条', file=sys.stderr)\nprint('       第 6 行的状态是「开着」', file=sys.stderr)\nsys.exit(3)\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-precondition-missing.py")
        cases += [
            case("准入:前提没齐不写重跑登记", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 2),
            case("准入:前提没齐、写了放行行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。\n准入已判：前提由用户当面确认过\n", 0),
        ]
        precondition_refusal = designer_admission_verdict("写重跑登记 research/prompts/e142-r19-prereg.md。", work, hook_dir) or ""
        cases.append(("准入:前提没齐的拒绝说明点名 admission.py 报的前提与放行行", True,
                      "第 6 行的状态是「开着」" in precondition_refusal and "准入已判" in precondition_refusal))
        # admission.py 退别的码（假的那一份照崩溃的样子在 stderr 打一行、退 1）：判不了，照常放行（记检出那一半在下面「没判全」那几格里）
        write_sample(work, "admission-crashed.py", "import sys\nprint('Traceback (most recent call last): 自证造的崩溃', file=sys.stderr)\nsys.exit(1)\n")
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-crashed.py")
        cases.append(case("准入:admission.py 退别的码（崩溃退 1）照常放行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 0))
        os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = os.path.join(work, "admission-changed.py")
        cases.append(case("准入:输入变了放行", "experiment-designer", "写重跑登记 research/prompts/e142-r19-prereg.md。", 0))
        cases += [
            # 快照：派书记员 / 实现员时要改的文件落在还没判完的三方轮的开工快照里，拒绝
            case("快照:书记员撞开着的轮（清单写 ./.claude/）", "kb-scribe", collided, 2),
            case("快照:实现员撞开着的轮（清单写 tree/crates/）", "implementation-writer", "改 crates/singlefs-core/src/mount.rs 的挂载准入，带测试。", 2),
            case("快照:提示写项目根起的绝对路径、中文文件名", "kb-scribe", f"改 {work}/.claude/kb/decisions/18-块里携带什么信息.md 已定项 11 的射程。", 2),
            case("快照:清单写 defs/.claude/ 也归一", "kb-scribe", "规格：改 `.claude/agents/crash-verifier.md` 的写范围一节。", 2),
            case("快照:规格文件里点名的照拒（路径后面紧跟中文）", "kb-scribe", f"逐条规格在 {specification_path}里，照做。报告写到同一目录。", 2),
            case("快照:规格文件写成 ~ 起头也认", "kb-scribe", f"逐条规格在 {home_new_root}{specification_path[len(new_root):]}，照做。", 2),
            case("快照:会话临时目录根下的规格文件里点名的照拒", "kb-scribe", f"逐条规格在 {session_temporary_specification_path}，照做。", 2),
            case("快照:路径后面连着别的路径字符不算", "implementation-writer", "对照 crates/singlefs-core/src/mount.rs.orig 那份旧拷贝。", 0),
            case("快照:撞的是已判完的轮，放行", "kb-scribe", "规格：改 `.claude/kb/decisions/03-空间分配.md` 已定项 2。", 0),
            case("快照:有快照没正文的不是三方轮，放行", "implementation-writer", "改 crates/singlefs-core/src/allocator.rs 的分配顺序。", 0),
            case("快照:有放行行，放行", "kb-scribe", collided + "快照冲突已判：r-open 只读它核 C120 的行号，不改那一行\n", 0),
            case("快照:放行行写的是别的轮，照拒", "kb-scribe", collided + "快照冲突已判：r-closed 只读\n", 2),
            case("快照:放行行没写理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n", 2),
            case("快照:放行行没写理由、下一行不算它的理由，照拒", "kb-scribe", collided + "快照冲突已判：r-open\n报告写到草稿目录。\n", 2),
            case("快照:放行行理由是占位，照拒", "kb-scribe", collided + "- 快照冲突已判：r-open <理由>\n", 2),
            case("快照:拒绝信息原样转贴，句中的放行模板不算", "kb-scribe", forwarded_refusal, 2),
            case("快照:派别的 agent 放行（设计员）", "experiment-designer", collided, 0),
            case("快照:派别的 agent 放行（实验设计员）", "experiment-designer", collided, 0),
            case("快照:清单里认不出的行不拒", "kb-scribe", "规格：改 README.md、research/prompts/r-odd-model.md 与 `.claude/kb/pitfalls.md`。", 0),
            case("快照:规格文件不存在不拒", "kb-scribe", f"逐条规格在 {missing_specification_path}，草稿目录 {specification_directory}/，报告写 {specification_directory}/result.md。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，不拒", "kb-scribe", "规格：改 `.claude/kb/pitfalls.md`。", 0),
            case_in(unreadable_work, "快照:一份清单读不了，读得了的那份撞了照拒", "kb-scribe", collided, 2),
        ]
        # ⑱ 常设要求的几行：批次（batch-unchecked、batch-line-optional、batch-file-unchecked、batch-label-digits-only）、草稿目录落点（scratch-line-root-unchecked、
        # scratch-line-legacy-root-accepted）、自证在哪跑（selftest-location-unchecked）；
        # ⑰ 三方腿要行首「报告：」一行（leg-report-line-optional）
        footer_without_batch = FOOTER.replace(SELFTEST_BATCH_LINE, "")
        cases += [
            case("批次:十种类型没写批次一行拒", "implementation-writer", "改完交回。", 2, footer=footer_without_batch, batch=False),
            case("批次:别的类型没写批次一行不判", "sweep", "改完交回。", 0, footer=footer_without_batch, batch=False),
            case("批次:点名的调度记录不在拒", "implementation-writer", "批次：records/not-there.md#3\n改完交回。", 2, footer=footer_without_batch),
            case("批次:别的类型写了批次一行照判点名的调度记录", "sweep", "批次：records/not-there.md#3\n改完交回。", 2, footer=footer_without_batch),
            case("批次:不是 records/<文件>.md#<批号> 的写法拒", "implementation-writer", "批次：第 12 批\n改完交回。", 2, footer=footer_without_batch),
            case("批次:路径里带 .. 拒", "implementation-writer", "批次：records/../records/selftest-batch.md#1\n改完交回。", 2, footer=footer_without_batch),
            case("批次:批号写批名放行", "implementation-writer", "批次：records/selftest-batch.md#第 3 批后半\n改完交回。", 0, footer=footer_without_batch),
            case("批次:批号后面接句号与别的话放行", "implementation-writer", "批次：`records/selftest-batch.md#4-3a`。回复用中文。\n改完交回。", 0, footer=footer_without_batch),
            case("批次:闭合围栏里的批次一行不算", "implementation-writer", "```\n批次：records/selftest-batch.md#1\n```\n改完交回。", 2, footer=footer_without_batch, batch=False),
            case("草稿目录落点:落在会话临时目录根拒", "experiment-designer", f"草稿目录：{session_temporary_root}/b12-standing/\n写跑前登记。", 2),
            case("草稿目录落点:落在工作根之外拒", "experiment-designer", "草稿目录：/var/tmp/b12-standing/\n写跑前登记。", 2),
            case("草稿目录落点:写相对路径拒", "experiment-designer", "草稿目录：scratch/b12-standing/\n写跑前登记。", 2),
            case("草稿目录落点:写工作根本身拒", "experiment-designer", f"草稿目录：{home_new_root}/\n写跑前登记。", 2),
            case("草稿目录落点:~ 起头、落在工作根下放行", "experiment-designer", f"草稿目录：{home_new_root}/b12-standing/\n写跑前登记。", 0),
            case("草稿目录落点:$SINGLEFS_WORK_ROOT/ 起头放行", "experiment-designer", "草稿目录：$SINGLEFS_WORK_ROOT/b12-standing/\n写跑前登记。", 0),
            case("草稿目录落点:闭合围栏里的会话临时目录根示例行不判", "experiment-designer", f"```\n草稿目录：{session_temporary_root}/b12-standing/\n```\n写跑前登记。", 0),
            case("自证在哪跑:指到工作根下的仓副本拒", "tooling-writer", f"自证在哪跑：{home_new_root}/b12-standing/repo 里那一份\n改完交回。", 2),
            case("自证在哪跑:指到会话临时目录根拒", "tooling-writer", f"自证在哪跑：{session_temporary_root}/b12-standing/\n改完交回。", 2),
            case("自证在哪跑:本仓主工作区放行", "tooling-writer", "自证在哪跑：本仓主工作区那一份\n改完交回。", 0),
            case("自证在哪跑:不跑放行", "tooling-writer", "自证在哪跑：不跑\n改完交回。", 0),
            case("三方报告:没有行首「报告：」一行、句中写报告写进拒", "three-way-attack",
                 "轮名 selftest-r1。照攻击面逐格找反例。报告写进 `research/prompts/selftest-r1-opus-output.md`。\n", 2),
            case("三方报告:行首「报告：」一行放行", "three-way-attack", "轮名 selftest-r1。\n报告：research/prompts/selftest-r1-opus-output.md\n", 0),
            case("三方报告:「报告：」一行只在闭合围栏里拒", "three-way-attack",
                 "轮名 selftest-r1。报告写进 research/prompts/selftest-r1-opus-output.md。\n```\n报告：research/prompts/selftest-r1-opus-output.md\n```\n", 2),
        ]
        # ⑱ 批次一行认代码块：照 CommonMark 认（markdown-it-py），闭合了的围栏块里的行不判；「批次：」标签行在 CommonMark 认法与宽的逐行认法下遮不遮不一致就拒。
        # 好的一行点名样本仓里那份记录，坏的一行写法认不出（不看记录在不在，batch-file-unchecked、batch-label-digits-only 下照拒）；写了几行的每一行都判。
        # 开关 code-fence-loose、code-block-line-rule、code-block-library-missing、code-block-ambiguity-unchecked、code-block-fallback-unmasked 各红在文件头点名的格
        batch_good, batch_bad = SELFTEST_BATCH_LINE, "批次：第 12 批\n"
        inline_backticks_line = "```cargo test -p singlefs-verification --lib``` 是上一次的起法，照它起。\n"   # 行首写在一行里的三反引号：行内代码，不开围栏（开关 code-fence-loose 下当开头）
        fence_ambiguous_list_item = "- 上一次派发的原文：\n  ```\n  " + batch_good + "```\n" + batch_bad + "上一次的骨架：\n```\n" + batch_good + "```\n"
        def batch_case(label, prompt, want):
            return case(label, "clerk", prompt, want, footer=footer_without_batch, batch=False)
        cases += [
            batch_case("批次:代码块里的示例行不判放行", batch_good + "喂法示例：\n```\n" + batch_bad + "```\n", 0),
            batch_case("批次:缩进的围栏里的示例行不判放行", batch_good + "  ```text\n  " + batch_bad + "  ```\n", 0),
            batch_case("批次:代码块之后行首的那一行照判", "```\n" + batch_good + "```\n" + batch_bad, 2),
            batch_case("批次:没闭合的围栏遮不住后面的行", batch_good + "```\n" + batch_bad, 2),
            batch_case("批次:行首行内三反引号不开围栏，后面写坏的一行照判拒", inline_backticks_line + batch_bad + "上一次派发的原文：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:行内三反引号之后真代码块里的示例行不判放行", batch_good + inline_backticks_line + "上一次派发的原文：\n```\n" + batch_bad + "```\n", 0),
            batch_case("批次:缩进 4 格的三反引号不开围栏，后面一行照判拒", batch_good + "    ```\n" + batch_bad + "```\n", 2),
            batch_case("批次:波浪线块里露出一行三个反引号，块外写坏的一行照判拒",
                       "上一次给工具员讲围栏时贴的样子：\n~~~\n```\n~~~\n" + batch_bad + "上一次派发的原文：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:列表项里没闭合的围栏在列表项结束处结束，后面写坏的一行照判拒",
                       "- 上一次的起法：\n  ```\n  cargo test -p singlefs-verification --lib\n- 这一次照它起。\n\n" + batch_bad + "上一次派发的原文：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:HTML 注释里的三个反引号不开块，后面写坏的一行照判拒", "<!-- 旧写法：\n```\n-->\n" + batch_bad + "上一次派发的原文：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:波浪线代码块里的示例行不判放行", batch_good + "上一次派发的原文：\n~~~\n" + batch_bad + "~~~\n", 0),
            batch_case("批次:嵌套列表里缩进 4 格的围栏里的示例行不判放行", batch_good + "- 上一次：\n  - 派发原文：\n    ```\n    " + batch_bad + "    ```\n", 0),
            batch_case("批次:引用块里的围栏里的示例行不判放行", batch_good + "> 上一次派发的原文：\n> ```\n> " + batch_bad + "> ```\n", 0),
            # 标签行在 CommonMark 认法与宽的逐行认法下遮不遮不一致就拒（code_block_ambiguous_label_lines；开关 code-block-ambiguity-unchecked 下这几格红）
            batch_case("批次:列表项里开的围栏关在顶格，两种认法不一致拒", fence_ambiguous_list_item, 2),
            batch_case("批次:引用块里开的围栏关的那一行漏 >，两种认法不一致拒",
                       "> 上一次派发的原文：\n> ```\n> " + batch_good + "```\n" + batch_bad + "上一次的骨架：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:<details> 里有空行，两种认法不一致拒", "<details><summary>上一次派发的原文</summary>\n```\n" + batch_good + "\n活：照上一次的骨架改。\n```\n</details>\n\n"
                       + batch_bad + "上一次的骨架：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:关的那一行多缩进 4 格，两种认法不一致拒",
                       "上一次派发的原文：\n```\n" + batch_good + "    ```\n" + batch_bad + "上一次的骨架：\n```\n" + batch_good + "```\n", 2),
            batch_case("批次:开四个反引号关三个，两种认法不一致拒",
                       "上一次派发的原文：\n````\n" + batch_good + "```\n" + batch_bad + "上一次的骨架：\n````\n" + batch_good + "````\n", 2),
            batch_case("批次:四个反引号套三个反引号的示例，两种认法不一致拒", batch_good + "上一次给工具员讲围栏的原文：\n````\n```\n" + batch_bad + "```\n````\n", 2),
            batch_case("批次:外层波浪线套三个反引号的示例，两种认法一致放行", batch_good + "上一次给工具员讲围栏的原文：\n~~~\n```\n" + batch_bad + "```\n~~~\n", 0),
            batch_case("批次:粗体与列表记号也认", "- **批次**：第 12 批\n", 2),
            batch_case("批次:每一行都要对", batch_good + batch_bad, 2),
        ]
        fence_ambiguity_code, fence_ambiguity_message = runner_dispatch_guard_locks_and_verdicts.decide({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk",
                                                                                                  "prompt": fence_ambiguous_list_item + footer_without_batch}}, work, hook_dir)
        cases.append(("批次:两种认法不一致的拒绝说明逐行点名、叫把围栏写规整", True,
                      fence_ambiguity_code == 2 and "两种认法不一致" in (fence_ambiguity_message or "")
                      and "第 5 行「批次：第 12 批」：照 CommonMark 认在代码块里" in (fence_ambiguity_message or "") and "把围栏写规整" in (fence_ambiguity_message or "")))
        # 载不进 markdown-it-py（CODE_BLOCK_LIBRARY_VARIABLE 设成一个不在的模块名）：退回宽的逐行认法遮代码块、不比两种认法（开关 code-block-fallback-unmasked 下一行都不遮，前两格红）
        os.environ[CODE_BLOCK_LIBRARY_VARIABLE] = "markdown_it_absent_in_dispatch_guard_selftest"
        code_block_library_note_printed.append(True)   # 自检里那一句「载不进」不往 stderr 写
        cases += [
            batch_case("批次:载不进 markdown-it-py 时退回逐行认法，闭合了的反引号围栏里的示例行不判放行", batch_good + "上一次派发的原文：\n```\n" + batch_bad + "```\n", 0),
            batch_case("批次:载不进 markdown-it-py 时退回逐行认法，波浪线围栏里的示例行不判放行", batch_good + "上一次派发的原文：\n~~~\n" + batch_bad + "~~~\n", 0),
            batch_case("批次:载不进 markdown-it-py 时退回逐行认法，代码块之后行首的那一行照判", "```\n" + batch_good + "```\n" + batch_bad, 2),
        ]
        del os.environ[CODE_BLOCK_LIBRARY_VARIABLE]
        code_block_library_note_printed.clear()
        # ⑱ 报告落点（拆分提案第五十六节 ③；开关 report-line-root-unchecked 下两格拒的红）、
        # 不碰一行与关的是哪几条（第一百二十二节 ⑨ 的 D3、A5；untouched-line-unchecked、addressed-section-unchecked 下拒的那几格红）；
        # ⑲ 门禁阶段（第六十四节 ①；gate-stage-unchecked 下拒的四格红，gate-stage-negation-judged 下否定句那一格红，gate-stage-light-allowed 下命名四格与 --list 两格与拒绝说明那一格红）
        write_sample(work, ".claude/gate.d/code-source-discipline.sh", "#!/usr/bin/env bash\n# 自检样本：⑲ 按文件名认 .claude/gate.d/ 这一层的阶段\n")
        write_sample(work, ".claude/gate.d/doc-text.sh", "#!/usr/bin/env bash\n# 自检样本\n")
        write_sample(work, ".claude/gate.d/code-tooling.sh", "#!/usr/bin/env bash\n# 自检样本：叙述句那几格点名它\n")
        write_sample(work, ".claude/gate.d/crash-point-replay.sh", "#!/usr/bin/env bash\n# 自检样本：叙述句那几格点名它\n")
        batch_lines_bare = batch_lines.replace(SELFTEST_ADDRESSED_SECTION, "")
        clerk_inputs = f"草稿目录 {home_new_root}/c/，报告 {home_new_root}/c/result.md\n"
        cases += [
            case("报告落点:落在会话临时目录根拒", "clerk", f"报告：{session_temporary_root}/b12-location/result.md\n改完交回。", 2),
            case("报告落点:落在别的 /tmp 外的目录也拒", "clerk", "报告：/var/tmp/b12-location/result.md\n改完交回。", 2),
            case("报告落点:工作根下放行", "clerk", f"报告：{home_new_root}/b12-location/result.md\n改完交回。", 0),
            case("报告落点:仓里的相对路径不判落点", "clerk", "报告：research/prompts/b12-location-result.md\n改完交回。", 0),
            case("报告落点:报告行里单独一个 / 是分隔记号、不当路径", "clerk", f"报告：{home_new_root}/b12-location/result.md\n报告：上面几步各一节；表头「已攻 / 未攻」。\n", 0),
            case("不碰:实现员没写不碰一行拒", "implementation-writer", "改完交回。", 2, standing=False),
            case("不碰:工具员没写不碰一行拒", "tooling-writer", "改完交回。", 2, standing=False),
            case("不碰:工具员写在长标签里（别的会话在改、你不碰的文件）放行", "tooling-writer", "别的会话此刻在改、你不碰的文件：scripts/a.py\n改完交回。", 0, standing=False),
            case("不碰:不碰一行空着、下面接列表放行", "tooling-writer", "不碰：\n- scripts/a.py\n\n改完交回。", 0, standing=False),
            case_in(defs_work, "关的是哪几条:有定义的类型没写那一节拒", "clerk", batch_lines_bare + clerk_inputs, 2, footer=False, standing=False),
            case_in(defs_work, "关的是哪几条:节下是空的拒", "clerk", batch_lines_bare + clerk_inputs + "\n## 关的是哪几条\n\n## 出口\n\n做完\n", 2, footer=False, standing=False),
            case_in(defs_work, "关的是哪几条:写成「## 要回答的问题」也认", "clerk", batch_lines_bare + clerk_inputs + "\n## 要回答的问题\n\n1. 样本\n", 0, footer=False, standing=False),
            case("关的是哪几条:没有定义的类型不判", "clerk", "改完交回。", 0, standing=False),
            case("门禁阶段:叫子 agent 跑 --check mutation-tables 单格拒（拆分提案第六十四节的原句）", "clerk", "你可以跑 `code-source-discipline.sh --check mutation-tables`。", 2),
            case("门禁阶段:写全路径叫子 agent 跑一道拒", "clerk", "改完跑 `bash .claude/gate.d/doc-text.sh .` 看绿。", 2),
            case("门禁阶段:命名检查只点名那四格也拒（子 agent 不往服务交任务）", "clerk", "交回前跑 `bash .claude/gate.d/code-source-discipline.sh --check vague-names,test-file-names,ordinal-names,data-names <根>`。", 2),
            case("门禁阶段:否定句「不跑门禁阶段」放行", "clerk", "不跑门禁阶段（.claude/gate.d/ 下的任何一道，单格点名也不跑），也不跑 `code-source-discipline.sh --check mutation-tables`。", 0),
            case("门禁阶段:要主 agent 跑的那一行不判", "clerk", "要主 agent 跑的命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-text.sh 的 clock-times 格; 范围=1 格; 为什么=核' bash .claude/gate.d/doc-text.sh --check clock-times .`", 0),
            case("门禁阶段:转述别的会话的子 agent 在跑不判", "clerk", "别的会话的执行员可能在跑 `doc-text.sh`，与你的文件无关。", 0),
            case("门禁阶段:--list 只查清单也拒（子 agent 不往服务交任务）", "clerk", "先跑 `bash .claude/gate.d/doc-text.sh --list` 看格名。", 2),
            # ㉒ 叫子 agent 自己交任务的句子（task-submission-unchecked 下拒的三格红）
            case("交任务:正文叫子 agent 经 task-server/task submit 交自证拒", "clerk",
                 "改完把自证交给服务：`task-server/task submit -- bash .claude/hooks/x.sh --selftest`，再用 run_in_background 起 `task-server/task wait <号>`。", 2),
            case("交任务:正文许可子 agent 自己经统一任务服务交 harness 测试拒", "clerk", "harness 测试你可以自己经统一任务服务交。", 2),
            case("交任务:骨架改前的「自证在哪跑」那一句拒", "clerk",
                 "自证在哪跑：本仓主工作区那一份；自证与样本档经 task-server 提交（`task-server/task submit -- <脚本> --selftest`，再用 run_in_background 起 `task-server/task wait <号>`）", 2),
            case("交任务:否定句不往统一任务服务交任务放行", "clerk", "子 agent 不往统一任务服务交任务，`task-server/task submit`、`task-server/task wait` 一条都不起。", 0),
            case("交任务:写进报告由主 agent 交的 submit 整条不判", "clerk",
                 "把命令拼成 `task-server/task submit -- bash x.sh --selftest` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交。", 0),
            # 交回说法、没写「主 agent」的句子不判（hand-back-sentences-judged 下这两格红）
            case("交任务:只说拼成提交整条、写进报告的交回节、没写主 agent 不判", "clerk",
                 "自证命令照 `task-server/task submit -- bash .claude/hooks/x.sh --selftest` 拼成提交整条、写进报告的交回节。", 0),
            case("交任务:只说列进带引号的「要主 agent 跑的命令」一节、引号外没写主 agent 不判", "clerk",
                 "改过的钩子照 `task-server/task submit -- bash .claude/hooks/x.sh --selftest` 列进「要主 agent 跑的命令」一节。", 0),
            case("交任务:只读的 status、log 放行", "clerk", "排队看 `task-server/task status 7`，日志看 `task-server/task log 7`。", 0),
            case("交任务:引号里转述的不判", "clerk", "旧骨架写的是「自证与样本档经 task-server 提交，再用 run_in_background 起 task wait」。", 0),
            # 叙述句（narrated_sentence、text_rewrite_before、without_quoted_text）：说文字改成什么、讲已经发生的现象、讲已出结局的任务、外层引文里套着内层引文，
            # 都不是叫子 agent 交任务、起门禁阶段；叫它自己做的句子照拒（各开关打开后哪几格红写在文件头）
            case("叙述:实验页复跑块改成经任务服务交的入口，不当叫子 agent 交任务", "clerk", "（E07、E53、E159 三份实验页的复跑块改成经任务服务交的入口）", 0),
            case("叙述:这三页只改了复跑几行的写法、直起虚机脚本改成经任务服务交的入口，不当叫子 agent 交任务", "clerk",
                 "这三页这一次只改了复跑那几行的写法（直起虚机脚本改成经任务服务交的入口），数、结论、产物没动", 0),
            case("叙述:现象标签起头的句子说门禁经任务服务分片交时撞内存上限，不当叫子 agent 交任务", "clerk",
                 "现象：门禁 crash-point-replay 经任务服务分片交时，计划那一步每次都要连撞三次内存上限才起得来", 0),
            case("叙述:外层引文套两对内层引文，说门禁改记本次未跑、文件头说法跟着改，整段是转述不当叫子 agent 起门禁阶段", "clerk",
                 "原话：「2. 门禁 crash-point-replay 对「GPU 服务不收某一族待判行」改记本次未跑（照 .claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」那张表"
                 "「只跑了一部分」那一行：逐项报本次未跑、不算过），`.claude/gate.d/crash-point-replay.sh:330` 附近文件头的说法跟着改」", 0),
            case("叙述:已出结局的任务（任务号与退 1）说开关复跑，不当叫子 agent 起门禁阶段", "clerk",
                 "1. 派发闸换上之后的开关复跑（任务 20751，`code-tooling.sh --check break-switch-replay --item .claude/hooks/runner-dispatch-guard.sh`）退 1", 0),
            case("叙述:现象标签只放行它那一句，同一行后面叫它自己经统一任务服务交的句子照拒", "clerk", "现象：门禁卡住。请你自己经统一任务服务交自证。", 2),
            case("叙述:现象标签起头、同一句里又叫它自己交的句子照拒", "clerk", "现象：门禁卡住了，请你自己经统一任务服务交一遍。", 2),
            case("叙述:带任务号与退几、又叫它自己再交一遍的句子照拒", "clerk", "任务 20751 退 1，请你自己经统一任务服务再交一遍。", 2),
            case("叙述:带任务号与退几、又叫它自己跑门禁阶段的句子照拒", "clerk", "任务 20751 退 1，请你自己跑 `bash .claude/gate.d/doc-text.sh .`。", 2),
            case("叙述:改成可用之后再经任务服务交的句子照拒", "clerk", "把入口改成可用之后再经任务服务交一遍。", 2),
        ]
        gate_stage_refusal = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "clerk", "prompt": "你可以跑 `code-source-discipline.sh --check mutation-tables`。\n" + FOOTER}}, work, hook_dir)
        cases.append(("门禁阶段:拒绝说明引那一句、点名格与判成哪一类、给否定句与交主 agent 的出路", True,
                      all(piece in (gate_stage_refusal[1] or "") for piece in ("code-source-discipline.sh --check mutation-tables：门禁阶段，子 agent 一律不起", "不跑门禁阶段（.claude/gate.d/ 下的任何一道",
                                                                              "要主 agent 跑的命令", "→ 怎么办"))))
        batch_refusal = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "clerk", "prompt": "批次：records/not-there.md#3\n" + footer_without_batch}}, work, hook_dir)
        cases.append(("批次:拒绝说明点名不在的调度记录、给照抄的一行", True,
                      all(piece in (batch_refusal[1] or "") for piece in ("records/not-there.md 在本仓仓根与各棵工作树里都不在", "批次：records/<调度记录>.md#<批号>"))))
        script = os.path.join(hook_dir, "runner-dispatch-guard.sh")
        # ⑱ 零轮改法的 A/B 读数（zero-round-reading-unchecked 下拒的四格与说明那一格红，zero-round-reading-number-unchecked 下没有任务号那一格红；
        #   判据库那一判的格见下面，zero-round-reading-unchecked 与 zero-round-reading-library-unjudged 下红）
        zero_round_reading_line = "A/B 读数：任务 11964，白推 18105 串、写进 0 次\n"
        zero_round_change = "把腿给的零轮改法写进产品代码。\n"
        cases += [
            case("A/B 读数:实现员提示写了零轮、没有 A/B 读数一行拒", "implementation-writer", "把腿给的零轮改法「宁大勿小」写进产品代码。\n改完交回。", 2),
            case("A/B 读数:提示写了零轮、带 A/B 读数一行放行", "implementation-writer", zero_round_change + zero_round_reading_line + "改完交回。", 0),
            case("A/B 读数:A/B 读数一行没有任务号拒", "implementation-writer", zero_round_change + "A/B 读数：探针里量过，写进 0 次\n改完交回。", 2),
            case("A/B 读数:不含零轮的实现员提示照旧放行", "implementation-writer", "把判决里的改法写进产品代码。\n改完交回。", 0),
            case("A/B 读数:A/B 读数写在句中、不在行首拒", "implementation-writer", "把腿给的零轮改法写进产品代码，已有 A/B 读数：任务 11964。\n改完交回。", 2),
            case("A/B 读数:A/B 读数一行只在闭合围栏里拒", "implementation-writer", zero_round_change + "```\n" + zero_round_reading_line + "```\n改完交回。", 2),
            case("A/B 读数:别的类型提到零轮不判", "tooling-writer", "判决里有一条被攻过零轮的改法，写进规则。\n改完交回。", 0),
        ]
        zero_round_refusal = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "implementation-writer",
                                                    "prompt": zero_round_change + FOOTER + SELFTEST_UNTOUCHED_LINE + "要动的 crates 文件：crates/selftest/src/zero-round-message.rs\n"}}, work, hook_dir)
        cases.append(("A/B 读数:拒绝说明点名规则文件、给照抄的一行与先在探针里 A/B 的出路", True,
                      all(piece in (zero_round_refusal[1] or "") for piece in (".claude/rules/three-way-inference.md", "A/B 读数：任务 <号>", "先在探针里对这一改法 A/B", "被打中的那一支"))))
        # ⑱ 读数一行交判据库按判决里的写法判一遍（zero-round-reading-library-unjudged 下拒的五格、说明那一格与逐条对拍那一格红）：
        #   判据库在判决里拒的四条（另带 task=、同字并列的号、反引号里的 task=、task log 加号）与引用块里的一行，派发闸也拒；两臂两个号、单臂一个号放行；七条两边逐条结局相同
        reading_verdict_lines = (
            ("读数一行另带 task= 的号，判决里照拒、派发闸当场拒", "A/B 读数：任务 12400，结局 task=12400 type=succeeded exit=0", 2),
            ("读数一行另带同字并列的号，判决里照拒、派发闸当场拒", "A/B 读数：任务 12400（同 12401、12402 两臂）", 2),
            ("读数一行把 task= 写在反引号里，判决里照拒、派发闸当场拒", "A/B 读数：任务 12400，结局 `TASK_SERVER_RESULT task=12400 type=succeeded exit=0`", 2),
            ("读数一行另带 task log 加号，判决里照拒、派发闸当场拒", "A/B 读数：任务 12400；task-server/task log 12400 末行", 2),
            ("引用块里的读数一行，判决里照拒、派发闸当场拒", "> A/B 读数：任务 12400", 2),
            ("两臂两个任务号的读数一行，判决里放行、派发闸放行", "A/B 读数：任务 12400 与任务 12401", 0),
            ("单臂一个任务号的读数一行，判决里放行、派发闸放行", "A/B 读数：任务 12400", 0),
        )
        reading_verdict_cases = [case(f"A/B 读数:{what}", "implementation-writer", zero_round_change + line + "\n改完交回。", want) for what, line, want in reading_verdict_lines]
        cases += reading_verdict_cases
        reading_library = notations_library()
        cases.append(("A/B 读数:七条读数一行派发闸与判据库逐条结局相同",
                      [2 if reading_library.new_violation_hits(NOTATIONS_FORM, "", line, ZERO_ROUND_VERDICT_PATH) else 0 for _, line, _ in reading_verdict_lines],
                      [item[2] for item in reading_verdict_cases]))
        reading_library_refusal = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "implementation-writer",
                                                         "prompt": zero_round_change + reading_verdict_lines[0][1] + "\n" + FOOTER + SELFTEST_UNTOUCHED_LINE
                                                         + "要动的 crates 文件：crates/selftest/src/zero-round-library-message.rs\n"}}, work, hook_dir)
        cases.append(("A/B 读数:判据库拒的读数一行，拒绝说明点名那一处、给能抄进判决的一行与另带的号换成什么", True,
                      all(piece in (reading_library_refusal[1] or "") for piece in ("判决里照拒的写法", "任务服务的任务号「task=12400」", "A/B 读数：任务 <号>，<量到什么>",
                                                                                      "结局写 type= 与 exit= 那一段", ".claude/rules/three-way-inference.md",
                                                                                      "「task <号>」", "写在同一行"))))
        def run_hook(project_root, stdin_text):
            return subprocess.run(["bash", script], input=stdin_text, capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=project_root))
        # 各条都判完一次列全：缺三样的提示一次报出三样（RUNNER_DISPATCH_GUARD_BREAK=stop-at-first 下红）；
        # 缺输入的说明写明按字面子串认、给照抄的一行（inputs-hint-dropped 下红）；红在别的开关下的写在文件头
        three_missing = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "three-way-local-attack",
                                               "prompt": f"草稿目录 {home_new_root}/r/\n提示文件：<填 提示文件>\n" + SELFTEST_ADDRESSED_SECTION}}, defs_work, hook_dir)
        cases.append(("一次列全:缺两样的提示一次报出两样（骨架占位、禁读清单）", (2, True),
                      (three_missing[0], all(fragment in (three_missing[1] or "") for fragment in
                                             ("有 2 处不成立", "占位没填", "缺它定义要的输入：禁读清单")))))
        cases.append(("输入:缺的输入写明按字面子串认、给照抄的一行", True,
                      all(fragment in (three_missing[1] or "") for fragment in ("按字面子串", "写「禁读：」不算", "\n禁读清单：<填 禁读清单>\n"))))
        # --check-prompt：同一份判法；放行时不登记实现员（登记文件一个字节不变），写明 ⑤ ⑦ 派的那一刻另判（check-mode-registers 下红）
        registry_file = runner_dispatch_guard_prompt_checks.registry_path({})
        def registry_bytes():
            return open(registry_file, "rb").read() if os.path.exists(registry_file) else None
        writer_prompt_path = os.path.join(defs_work, "writer-prompt.md")
        write_sample(defs_work, "writer-prompt.md", batch_lines + "要动的 crates 文件：crates/a/src/check-mode.rs\n")
        registry_before = registry_bytes()
        checked_prompt = subprocess.run(["bash", script, "--check-prompt", writer_prompt_path, "--type", "implementation-writer", "--root", defs_work],
                                        capture_output=True, text=True, env=dict(os.environ))
        cases.append(("检查模式:--check-prompt 放行时不登记实现员、写明派的那一刻另判的两条", (0, True, True),
                      (checked_prompt.returncode, registry_bytes() == registry_before, "⑤" in checked_prompt.stdout and "⑦" in checked_prompt.stdout)))
        snapshot_refusal = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": collided}}))
        cases.append(("stdin:快照撞了拒绝（退出码 2）", 2, snapshot_refusal.returncode))
        cases.append(("stdin:快照拒绝的 stderr 写了轮名、文件与出路", 1,
                      int(all(needle in snapshot_refusal.stderr for needle in ("r-open", ".claude/kb/checks-owed.md", "等 r-open 的判决写完再派", "快照冲突已判：r-open")))))
        broken_json = run_hook(work, "{不是 JSON")
        cases.append(("stdin:JSON 解析不了放行", 0, broken_json.returncode))
        cases.append(("stdin:JSON 解析不了时 stderr 写了错误", 1, int("JSON" in broken_json.stderr)))
        unreadable_list = run_hook(unreadable_work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": "规格：改 `.claude/kb/pitfalls.md`。\n" + FOOTER}}))
        cases.append(("stdin:快照清单读不了放行", 0, unreadable_list.returncode))
        cases.append(("stdin:快照清单读不了时 stderr 写了是哪一份", 1, int("读不了快照清单" in unreadable_list.stderr and "gone-sha256.txt" in unreadable_list.stderr)))
        # 读不了的快照清单分得出来（lib_snapshot_conflicts.py 的 unreadable）：样本仓里 r-kinds 开着，一份读得了的清单、一份是目录、一份没有读权限、一份空的；
        # r-empty-snapshot 只有一份空清单（没有正文，不是开着的轮，只给库那一格直接读）。读不了的两份分出来，空的不算读不了；派发照放行，
        # stderr 点名那一轮与读不了的那几份、写明放行但必记检出，检出记录多一行（结局只记）；清单都读得了的放行不记、不打那一句
        kinds_work = tempfile.mkdtemp()
        write_batch_record(kinds_work)   # ⑱ 批次一行点名的调度记录
        other_directories.append(kinds_work)
        write_sample(kinds_work, "research/prompts/_r-kinds-body.md", "# r-kinds 正文\n")
        write_sample(kinds_work, "research/prompts/r-kinds-snapshot/kb-sha256.txt", f"{digest}  .claude/kb/checks-owed.md\n")
        write_sample(kinds_work, "research/prompts/r-kinds-snapshot/empty-sha256.txt", "")
        write_sample(kinds_work, "research/prompts/r-kinds-snapshot/locked-sha256.txt", f"{digest}  .claude/kb/invariants.md\n")
        write_sample(kinds_work, "research/prompts/r-empty-snapshot/empty-sha256.txt", "")
        kinds_snapshot = os.path.join(kinds_work, "research", "prompts", "r-kinds-snapshot")
        folder_list, locked_list = os.path.join(kinds_snapshot, "folder-sha256.txt"), os.path.join(kinds_snapshot, "locked-sha256.txt")
        os.makedirs(folder_list)
        os.chmod(locked_list, 0)
        locked_readable = os.access(locked_list, os.R_OK)   # 以 root 跑时去掉读权限也读得了：那一份不算进期望
        def listed_and_unreadable(snapshot_directory, round_name):
            found = []
            try:
                return snapshot_library.snapshot_listed_paths(snapshot_directory, [], break_switch_is, round_name=round_name, unreadable=found), found
            except OSError as error:   # 弄坏开关 snapshot-unreadable-aborts 下抛出：这一格判红，不拖垮整份自检
                return {"<抛出>": repr(error)}, found
        kinds_listed, kinds_unreadable = listed_and_unreadable(kinds_snapshot, "r-kinds")
        cases.append(("快照:清单是目录或没有读权限，结果里分得出读不了", True,
                      sorted(path for _, path, _ in kinds_unreadable) == sorted([folder_list] + ([] if locked_readable else [locked_list]))
                      and all(round_name == "r-kinds" and reason for round_name, _, reason in kinds_unreadable)
                      and set(kinds_listed) == {".claude/kb/checks-owed.md"} | ({".claude/kb/invariants.md"} if locked_readable else set())))
        empty_listed, empty_unreadable = listed_and_unreadable(os.path.join(kinds_work, "research", "prompts", "r-empty-snapshot"), "r-empty")
        cases.append(("快照:清单读得了而是空的，不算读不了", True, empty_listed == {} and empty_unreadable == []))
        detections_file = os.environ["AGENT_HOOK_DETECTIONS"]
        def detection_entries():
            entries = []
            if os.path.exists(detections_file):
                with open(detections_file, encoding="utf-8") as detections_handle:
                    for line in detections_handle:
                        try:
                            entries.append(json.loads(line))
                        except ValueError:
                            entries.append({"坏行": line})
            return entries
        entries_before = len(detection_entries())
        kinds_run = run_hook(kinds_work, json.dumps({"tool_name": "Agent", "session_id": "kinds-session",
                                                     "tool_input": {"subagent_type": "kb-scribe", "prompt": "规格：改 `.claude/kb/pitfalls.md`。\n" + FOOTER}}))
        kinds_entries = detection_entries()[entries_before:]
        cases.append(("stdin:清单是目录或没有读权限照样放行", 0, kinds_run.returncode))
        cases.append(("stdin:快照清单读不了时 stderr 点名那一轮与那份清单、写明放行但必记检出", 1,
                      int(all(needle in kinds_run.stderr for needle in ("r-kinds", "folder-sha256.txt", "放行但必记检出", "→ 怎么办", "检出记进"))
                          and (locked_readable or "locked-sha256.txt" in kinds_run.stderr))))
        cases.append(("stdin:快照清单读不了时检出记录多一行（结局只记）", 1,
                      int(len(kinds_entries) == 1 and kinds_entries[0].get("outcome") == "只记" and kinds_entries[0].get("session_id") == "kinds-session"
                          and any("folder-sha256.txt" in finding for finding in kinds_entries[0].get("findings") or []))))
        entries_before = len(detection_entries())
        plain_run = run_hook(work, json.dumps({"tool_name": "Agent", "session_id": "plain-session",
                                               "tool_input": {"subagent_type": "kb-scribe", "prompt": "规格：改 `.claude/kb/pitfalls.md`。\n" + FOOTER}}))
        cases.append(("stdin:清单都读得了的放行不记检出、不打放行但必记检出", 1,
                      int(plain_run.returncode == 0 and len(detection_entries()) == entries_before and "放行但必记检出" not in plain_run.stderr)))
        # ⑥ 自己出错照常放行，照「放行但必记检出」记一条检出（开关 snapshot-error-unrecorded 下红）：共用模块的 open_three_way_rounds 换成抛异常的替身，
        # 进程里调 decide（record_detections 为真，同钩子入口）
        if snapshot_library is not None:
            real_open_rounds = snapshot_library.open_three_way_rounds
            def exploding_open_rounds(*arguments, **options):
                raise RuntimeError("自检造的快照判法出错")
            entries_before = len(detection_entries())
            snapshot_library.open_three_way_rounds = exploding_open_rounds
            try:
                error_code, error_notes = runner_dispatch_guard_locks_and_verdicts.decide({"tool_name": "Agent", "session_id": "snapshot-error-session",
                                                  "tool_input": {"subagent_type": "kb-scribe", "description": "自检",
                                                                 "prompt": "规格：改 `.claude/kb/pitfalls.md`。\n" + FOOTER}}, kinds_work, hook_dir, record_detections=True)
            finally:
                snapshot_library.open_three_way_rounds = real_open_rounds
            error_entries = detection_entries()[entries_before:]
            cases.append(("快照:自己出错照常放行并记一条检出（结局只记）", (0, 1, "只记", True),
                          (error_code, len(error_entries), error_entries[0].get("outcome") if error_entries else None,
                           bool(error_entries) and error_entries[0].get("session_id") == "snapshot-error-session"
                           and all(str(finding).startswith("派发闸没判全、照常放行：⑥ 快照那一条自己出错") for finding in error_entries[0].get("findings") or [None])
                           and "检出记进" in (error_notes or ""))))
        # 坏 JSON、⑯ 读不到 lib_writer_lock.py 两处放行时照「放行但必记检出」各记一条（拆分提案第九十四节 ①）：
        # 开关 json-error-unrecorded、writer-lock-library-unrecorded 下各自那一格红
        entries_before = len(detection_entries())
        json_unrecorded_run = run_hook(work, "{不是 JSON")
        json_entries = detection_entries()[entries_before:]
        cases.append(("stdin:JSON 解析不了放行时检出记录多一行（结局只记）", (0, 1, "只记", True),
                      (json_unrecorded_run.returncode, len(json_entries), json_entries[0].get("outcome") if json_entries else None,
                       bool(json_entries) and all(str(finding).startswith("派发闸没判全、照常放行：hook 的 JSON 读不懂") for finding in json_entries[0].get("findings") or [None]))))
        def library_missing_entries(global_name, label, hook_input, take_locks=False):
            """进程里把同目录的一份库当成读不到（模块级的那个名字换成 None），照钩子入口调 decide（record_detections 为真）：→ (退出码, 新记的检出, 提醒)。"""
            owner = REPLACEABLE_NAME_OWNERS[global_name]
            saved_library = owner.__dict__[global_name]
            before = len(detection_entries())
            owner.__dict__[global_name] = None
            try:
                code, library_notes = runner_dispatch_guard_locks_and_verdicts.decide(hook_input, work, hook_dir, take_locks=take_locks, record_detections=True)
            finally:
                owner.__dict__[global_name] = saved_library
            return code, detection_entries()[before:], library_notes or ""
        lock_library_directory = os.path.join(specification_directory, "writer-lock-library")
        lock_code, lock_entries, lock_notes = library_missing_entries("writer_lock", "写者锁", {
            "tool_name": "Agent", "session_id": "writer-lock-library-session",
            "tool_input": {"subagent_type": "clerk", "description": "自检",
                           "prompt": f"草稿目录：{lock_library_directory}/\n报告：{lock_library_directory}/result.md\n" + FOOTER}}, take_locks=True)
        cases.append(("写者锁:读不到 lib_writer_lock.py 照常放行并记一条检出（结局只记）", (0, 1, "只记", True),
                      (lock_code, len(lock_entries), lock_entries[0].get("outcome") if lock_entries else None,
                       bool(lock_entries) and lock_entries[0].get("session_id") == "writer-lock-library-session"
                       and any("lib_writer_lock.py" in str(finding) for finding in lock_entries[0].get("findings") or []) and "检出记进" in lock_notes)))
        # 判不了而放行的另几处（拆分提案第九十四节 ②）：每一处造一个判不了的输入，在进程里调那一段，核它记进了 UNJUDGED_NOTES；
        #   各处的弄坏开关 <处>-unrecorded 下那一处不记，对应的一格红；判的时候抛异常那一处照钩子入口调 judge_hook_input、核检出记录多一行
        import types
        def raising(*arguments, **keywords):
            raise RuntimeError("自证造的异常")
        def raising_oserror(*arguments, **keywords):
            raise OSError("自证造的读不了")
        repository_root = os.path.dirname(os.path.dirname(SCRIPT_HOOK_DIR))
        def with_globals(replaced, thunk):
            owners = {name: REPLACEABLE_NAME_OWNERS[name] for name in replaced}
            saved = {name: owners[name].__dict__[name] for name in replaced}
            for name, value in replaced.items():
                owners[name].__dict__[name] = value
            try:
                return thunk()
            finally:
                for name, value in saved.items():
                    owners[name].__dict__[name] = value
        def with_admission_script(path, thunk):
            saved_script = os.environ.get("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT")
            os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = path
            try:
                return thunk()
            finally:
                if saved_script is None:
                    os.environ.pop("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT", None)
                else:
                    os.environ["RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT"] = saved_script
        def with_failing_subprocess(thunk):
            saved_run = subprocess.run
            subprocess.run = raising_oserror
            try:
                return thunk()
            finally:
                subprocess.run = saved_run
        unreadable_spec = os.path.join(specification_directory, "unreadable-spec.md")
        open(unreadable_spec, "w").close()
        os.chmod(unreadable_spec, 0)
        page_root = tempfile.mkdtemp(prefix="runner-dispatch-guard-page-")
        os.makedirs(os.path.join(page_root, ".claude", "kb", "experiments"))
        locked_page = os.path.join(page_root, ".claude", "kb", "experiments", "100-样本.md")
        open(locked_page, "w").close()
        os.chmod(locked_page, 0)
        os.makedirs(os.path.join(page_root, ".claude", "agents"))
        locked_definition = os.path.join(page_root, ".claude", "agents", "three-way-locked.md")
        open(locked_definition, "w").write("---\nleg: three-way\n---\n")
        os.chmod(locked_definition, 0)
        unjudged_probes = [
            ("scope-library", "④ 读不到 lib_write_scope.py", lambda: with_globals({"write_scope": None}, lambda: scope_patterns(repository_root, "tooling-writer"))),
            ("scope-table", "④ 写范围表读不了", lambda: with_globals({"write_scope": types.SimpleNamespace(load_scopes=raising_oserror)},
                                                                      lambda: scope_patterns(repository_root, "tooling-writer"))),
            ("transcript", "主会话记录读不了", lambda: with_failing_subprocess(lambda: session_dispatch_events(os.path.join(hook_dir, "runner-dispatch-guard.sh")))),
            ("spec-check", "kb-spec-check.py 跑不了", lambda: with_failing_subprocess(lambda: kb_scribe_spec_verdict("规格：改 `.claude/kb/pitfalls.md`。\n", work, hook_dir))),
            ("admission", "admission.py 跑不了", lambda: with_failing_subprocess(lambda: designer_admission_verdict("写 research/prompts/e100-r2-prereg.md\n", work, hook_dir))),
            ("admission", "admission.py 退 0 而实验键拒算", lambda: with_admission_script(os.path.join(work, "admission-key-refused.py"),
                                                                              lambda: designer_admission_verdict("写重跑登记 research/prompts/e142-r19-prereg.md。", work, hook_dir))),
            ("admission", "admission.py 退 0、2、3、77 之外的码", lambda: with_admission_script(os.path.join(work, "admission-crashed.py"),
                                                                              lambda: designer_admission_verdict("写重跑登记 research/prompts/e142-r19-prereg.md。", work, hook_dir))),
            ("snapshot-spec", "⑥ 提示点名的规格读不了", lambda: specification_texts(f"规格：{unreadable_spec}\n", [])),
            ("scratch-line", "⑯ ⑰ 读不到 lib_prompt_scratch_line.py", lambda: with_globals({"prompt_scratch_line": None}, lambda: prompt_scratch_directories("草稿目录：/x\n"))),
            ("leg-types", "⑰ 读不到 lib_three_way_legs.py", lambda: with_globals({"three_way_legs": None}, lambda: three_way_report_types(work))),
            ("leg-definitions", "⑰ 有一份定义读不了", lambda: three_way_report_types(page_root)),
            ("fence-compare", "⑯ 两种围栏认法比的时候出错", lambda: with_globals({"prompt_scratch_line": types.SimpleNamespace(scratch_line_values=raising)},
                                                                            lambda: scratch_line_fence_ambiguity("草稿目录：/x\n"))),
            ("lock-create", "⑯ 建不了写者锁", lambda: with_globals({"writer_lock": types.SimpleNamespace(new_lock=lambda *arguments: {}, create_lock=raising_oserror)},
                                                                lambda: acquire_writer_locks({}, "提示", "clerk", [(os.path.join(page_root, "no-scratch"), None)]))),
            ("handover-lock", "⑯ 承接来的副本所在草稿目录的写者锁判不了", lambda: with_globals(
                {"writer_lock": types.SimpleNamespace(find_lock_directory=lambda target, roots, any_entry=False: "/sentinel-scratch", lock_path=lambda scratch: scratch + "/.writer-lock",
                                                      lock_state=lambda scratch: ("unreadable", "自检造的判不了"))},
                lambda: handover_lock_verdict(["/sentinel-scratch/result.md"], "implementation-writer", {}, {}, "", "digest", False))),
            ("scratch-root", "⑱ 认不出草稿目录落在哪", lambda: with_globals({"prompt_scratch_line": None}, lambda: scratch_line_root_verdict("clerk", "草稿目录：/x\n"))),
            ("experiment-page", "⑪ 实验页读不了", lambda: registration_has_segment(page_root, 100, 2)),
            ("registry-read", "⑤ 实现员与工具员的登记读不了", lambda: with_globals({"registry_path": lambda hook_input: page_root}, lambda: live_registry_rows({}, {}, {}))),
            ("registry-write", "⑤ 实现员与工具员的登记写不了", lambda: rewrite_registry(os.path.join(unreadable_spec, "sub", "implementation-writers.tsv"), [])),
            ("notations-library", "⑱ 读不到判据库 lib-forbidden-notations.py",
             lambda: with_globals({"NOTATIONS_LIBRARY_PATH": os.path.join(work, "no-such-lib-forbidden-notations.py")},
                                  lambda: zero_round_reading_verdict("implementation-writer", zero_round_change + zero_round_reading_line))),
        ]
        saved_stderr = sys.stderr
        for site, description, probe in unjudged_probes:
            del UNJUDGED_NOTES[:]
            sys.stderr = open(os.devnull, "w")
            try:
                probe()
            finally:
                sys.stderr.close()
                sys.stderr = saved_stderr
            cases.append((f"没判全:{description}记一处（{site}）", True, bool(UNJUDGED_NOTES)))
        del UNJUDGED_NOTES[:]
        os.chmod(unreadable_spec, 0o600)
        os.chmod(locked_page, 0o600)
        os.chmod(locked_definition, 0o600)
        shutil.rmtree(page_root, ignore_errors=True)
        # 记进 UNJUDGED_NOTES 的几处放行时真的落进检出记录：读不到 lib_three_way_legs.py（⑰ 那一处）照钩子入口调 decide（record_detections 为真）
        notes_directory = os.path.join(specification_directory, "unjudged-notes")
        entries_before = len(detection_entries())
        sys.stderr = open(os.devnull, "w")
        try:
            notes_code, _ = with_globals({"three_way_legs": None}, lambda: runner_dispatch_guard_locks_and_verdicts.decide(
                {"tool_name": "Agent", "session_id": "unjudged-notes-session",
                 "tool_input": {"subagent_type": "clerk", "description": "自检",
                                "prompt": f"草稿目录：{notes_directory}/\n报告：{notes_directory}/result.md\n" + FOOTER}},
                work, hook_dir, record_detections=True))
        finally:
            sys.stderr.close()
            sys.stderr = saved_stderr
        notes_entries = detection_entries()[entries_before:]
        cases.append(("没判全:放行时几处没判全落进检出记录（读不到 lib_three_way_legs.py，结局只记）", (0, 1, "只记", True),
                      (notes_code, len(notes_entries), notes_entries[0].get("outcome") if notes_entries else None,
                       bool(notes_entries) and notes_entries[0].get("session_id") == "unjudged-notes-session"
                       and any("lib_three_way_legs.py" in str(finding) for finding in notes_entries[0].get("findings") or []))))
        entries_before = len(detection_entries())
        sys.stderr = open(os.devnull, "w")
        try:
            raised_code = with_globals({"decide": raising}, lambda: judge_hook_input({"tool_name": "Agent", "session_id": "exception-session",
                                                                                     "tool_input": {"subagent_type": "clerk", "description": "自检", "prompt": "x"}}, work))
        except Exception as error:   # 弄坏开关 exception-unrecorded：照原样抛出
            raised_code = f"抛出 {type(error).__name__}"
        finally:
            sys.stderr.close()
            sys.stderr = saved_stderr
        raised_entries = detection_entries()[entries_before:]
        cases.append(("入口:判的时候抛异常照常放行并记一条检出（结局只记）", (0, 1, "只记", True),
                      (raised_code, len(raised_entries), raised_entries[0].get("outcome") if raised_entries else None,
                       bool(raised_entries) and raised_entries[0].get("session_id") == "exception-session"
                       and all(str(finding).startswith("派发闸没判全、照常放行：判的时候抛异常") for finding in raised_entries[0].get("findings") or [None]))))
        for label, prompt, want in (("stdin:续做没写还差", again + "这一段回答的岔路：岔路 1\n", 2),
                                    ("stdin:第一段点名了岔路", first + "这一段回答的岔路：岔路 1\n", 0)):
            completed = subprocess.run(["bash", script],
                                       input=json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "experiment-runner", "prompt": prompt + FOOTER}}),
                                       capture_output=True, text=True, env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
            cases.append((label, want, completed.returncode))
        permission = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk",
            "prompt": "可以跑 `cargo test -p singlefs-verification --lib` 看绿。\n" + FOOTER}}))
        cases.append(("stdin:正文许可跑项目代码拒绝（退出码 2）", 2, permission.returncode))
        cases.append(("stdin:许可句拒绝的 stderr 引了那一句、写了出路", 1,
                      int("可以跑 `cargo test -p singlefs-verification --lib` 看绿" in permission.stderr and "→ 怎么办：删掉那几句" in permission.stderr)))
        # 撤掉的 ⑭ ⑮ 那几行写在提示里，经钩子入口真派照样放行（开关 retired-lines-refused 下这一格红）
        retired_stdin = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk",
                                                                                   "prompt": "线程上限：16\n预计挂钟：超过 2 小时\n多机：每台线程按此刻空闲现算\n" + FOOTER}}))
        cases.append(("stdin:派 clerk 写了撤掉的几行放行（退出码 0）", 0, retired_stdin.returncode))
        # 引导载不全任何一份部分文件（读不到、编不过、截空）：拷一份钩子目录（入口加部分文件），删掉、写坏或截成空文件其中一份，按缺依赖的处置拒（退 2、stderr 点名那一份），不抛错退 1、不静默放行；
        # 截空各一份：第一份（prompt-checks）截空、带 main 的那一份（selftest）截空都在下面的格里；开关 part-unreadable-passes 下引导照原样抛错、进程退 1，下面几格红
        bootstrap_hooks = os.path.join(work, "bootstrap-copy", ".claude", "hooks")
        os.makedirs(bootstrap_hooks)
        bootstrap_input = json.dumps({"tool_name": "Agent", "session_id": "bootstrap-part", "tool_input": {"subagent_type": "clerk", "prompt": "x\n" + FOOTER}})
        for part_name in RUNNER_DISPATCH_PART_NAMES:
            for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过"), ("emptied", "截空")):
                for copied_name in ("runner-dispatch-guard.sh",) + RUNNER_DISPATCH_PART_NAMES:
                    shutil.copy(os.path.join(hook_dir, copied_name), bootstrap_hooks)
                if damage == "missing":
                    os.remove(os.path.join(bootstrap_hooks, part_name))
                else:
                    open(os.path.join(bootstrap_hooks, part_name), "w", encoding="utf-8").write("" if damage == "emptied" else "def broken(:\n")
                bootstrap_run = subprocess.run(["bash", os.path.join(bootstrap_hooks, "runner-dispatch-guard.sh")], input=bootstrap_input, capture_output=True, text=True,
                                               env=dict(os.environ, CLAUDE_PROJECT_DIR=work))
                cases.append((f"引导:{damage_label}部分文件 {part_name} 按拒判（退 2、点名那一份）", (2, True),
                              (bootstrap_run.returncode, f"部分文件 {os.path.join(bootstrap_hooks, part_name)}" in bootstrap_run.stderr and "按拒判" in bootstrap_run.stderr)))
        # ── ⑥ 用户叫停、不写判决的轮：research/prompts/abandoned-rounds.tsv 登记一行收快照；<轮>-halted.md 这一支已撤 ──
        # 出处是 records/ 下现存文件的行号加〔〕原话，原话在那份文件里恰好找到一次（行号只当提示），找到的那一行里有这一轮的名字与「」括的原话。开关 snapshot-abandoned-ignored 下放行的几格红；snapshot-abandoned-unchecked 下「照拒」各格与说明那一格红；
        # snapshot-abandoned-source-any-path、snapshot-abandoned-source-escapes、snapshot-abandoned-line-unread、snapshot-abandoned-round-name-unchecked、snapshot-abandoned-quote-short-ok 各红在文件头点名的格
        outside_directory = tempfile.mkdtemp()
        other_directories.append(outside_directory)
        write_sample(outside_directory, "note.md", "r-ab-outside 叫停：「仓外的一份笔记六个字以上」\n")
        write_sample(outside_directory, "link-target.md", "r-ab-symlink 叫停：「符号链接指到仓外的六个字以上」\n")
        write_sample(outside_directory, "research-note.md", "r-ab-research-outside 叫停：「从 research 起头拼出仓外的六个字以上」\n")
        outside_from_records = os.path.relpath(os.path.join(outside_directory, "note.md"), os.path.join(work, "records"))
        outside_from_research = os.path.relpath(os.path.join(outside_directory, "research-note.md"), os.path.join(work, "research"))
        write_sample(work, "records/ab-sample.md",
                     "- 用户 2026-10-11：r-ab-ok 叫停，「增加流程限制  测试代码不需要打， 改了直接测。」（样本）\n"
                     "- 用户 2026-10-11：「这是一句和任何一轮都无关的话」\n"
                     "- 用户 2026-10-11：r-ab-other 叫停，「别的轮的叫停原话也有六个字」\n"
                     "- 用户 r-ab-short 叫停：「增加流程」\n"
                     "- 用户 r-ab-noquote 叫停，没有引原话\n"
                     "- 用户 r-ab-edge2 叫停：「名字更长的另一轮的原话」\n"
                     "- 用户 r-ab-colon 叫停：「冒号加行号的写法也认」\n"
                     "- 用户 r-ab-badrow 叫停：「写坏的登记行不收的样本」\n"
                     "- r-ab-unrel r-ab-badline 的名字只在这一行出现，没有原话\n"
                     "- 用户 2026-10-12：r-ab-away-r1 叫停：「这一轮先停掉不做了」；r-ab-multi-r1 照常跑，等腿交回。\n"     # 10  同一行别的轮的叫停原话（形如 <名>-r<数字>）
                     "- 用户 r-ab-pair 与 r-ab-ok 一起叫停：「两轮一起先停下来吧」\n"                                     # 11  同一行是登记表里别的轮
                     "- 用户 2026-10-12：r-ab-real-r1 叫停：「真轮名带 r1 的写法也认」\n"                                    # 12  轮名形如 <名>-r<数字>、行里只有它
                     "- 用户 r-ab-baddate 叫停：「日期写成十月十三日的样本」\n"                                             # 13
                     "- 用户 r-ab-dup 叫停：「同一轮登记两行的样本」\n"                                                    # 14
                     "- 用户 r-ab-nameform 叫停：「别的轮名不带 r 数字的样本」；ab-close-b1 照常跑\n"                          # 15  同一行别的轮名不以 -r<数字> 结尾，但仓里有它的正文
                     "- 用户 r-ab-unfiled 叫停：「别的名字仓里没有材料的样本」；ab-nofile-b2 照常跑\n"                        # 16  同一行另一个名字，仓里没有任何它的材料
                     "- 用户 r-ab-formonly 叫停：「别的轮名只有形没有材料」；ab-ghost-r2 照常跑\n"                            # 17  同一行另一个名字形如 <名>-r<数字>，仓里没有任何它的材料
                     "- 用户 r-ab-drift 叫停：「行号挪了原话还在的样本」\n"                                                 # 18  登记行写第 2 行，〔〕原话在这一行（行号只当提示）
                     "- 用户 r-ab-gone 叫停：「原话后来改了字的样本」\n"                                                    # 19  登记行〔〕里写「叫停了」，这一行没有那几个字
                     "- 用户 r-ab-twice 叫停：「同一段原话写了两遍的样本」\n"                                               # 20  与下一行逐字相同：〔〕原话命中两次
                     "- 用户 r-ab-twice 叫停：「同一段原话写了两遍的样本」\n"                                               # 21
                     "- 用户 r-ab-plain 叫停：「只写行号没带原话的样本」\n"                                                 # 22  登记行只写第 22 行，没带〔〕原话
                     "- 用户 r-ab-pinned 叫停：「同一段原话写了两遍、行号落在其中一处」\n"                                    # 23  与下一行逐字相同：〔〕原话命中两次，行号提示写第 24 行
                     "- 用户 r-ab-pinned 叫停：「同一段原话写了两遍、行号落在其中一处」\n"                                    # 24
                     "- 用户 r-ab-reworded 叫停：「原话改了字而行号那一行还有轮名」\n"                                        # 25  登记行〔〕里写「已叫停」，找不到；行号提示的这一行有这一轮的名字
                     "- 补记 r-ab-twice：「另一处提到这一轮的六个字以上」\n"                                                 # 26  r-ab-twice 的行号提示：带名字与原话、不含〔〕那几个字（line-number-read 下收）
                     "- 用户 r-ab-renamed 叫停：「新称样本那一轮先停掉吧」\n"                                                  # 27  登记行〔〕写改名前的「旧称样本」，样本改名表登记了旧称样本 → 新称样本
                     "- 用户 r-ab-dated 叫停（用户说「删了日期之后的六个字以上」）\n"                                         # 28  登记行〔〕带着后来删掉的日期 2026-10-12 history-kept: 测试输入里的日期是数据
                     "- 用户 r-ab-fullparen 叫停：「日期写在全角括号里后来连括号删掉」\n"                                      # 29  登记行〔〕写「r-ab-fullparen（2026-10-12）叫停」，记录里那一对括号连日期删了 history-kept: 测试输入里的日期是数据
                     "- 用户 r-ab-halfparen 叫停：「日期写在半角括号里后来连括号删掉」\n")                                     # 30  同上，半角括号「(2026-10-12)」 history-kept: 测试输入里的日期是数据
        write_sample(work, ".claude/kb/term-renames.md", "# 术语改名对照表（自检样本）\n\n<!-- term-renames:table -->\n| 旧 | 新 | 匹配 | 是什么 |\n|---|---|---|---|\n"
                     "| 旧称样本 | 新称样本 | 整串 | 自检样本：r-ab-renamed 的出处那一行登记之后照这一行改了名 |\n")
        os.symlink(os.path.join(outside_directory, "link-target.md"), os.path.join(work, "records", "ab-link.md"))
        write_sample(work, ".claude/rules/ab-rule-sample.md", "r-ab-rule 的叫停：「规则文件里的一句六个字以上的话」\n")
        write_sample(work, "research/prompts/_r-ab-dotdot-body.md", "r-ab-dotdot 正文「路径里带点点的六个字以上」\n")
        write_sample(work, "research/prompts/_r-ab-away-r1-body.md", "# 别的轮（仓里认得的轮名：有正文）\n")      # r-ab-multi-r1 那一行写着它
        write_sample(work, "research/prompts/_ab-close-b1-body.md", "# 别的轮（轮名不以 -r<数字> 结尾，仓里有正文）\n")   # r-ab-nameform 那一行写着它；ab-nofile-b2 没有任何文件
        registered_on = "2026-10-12\t"
        abandoned_rounds = [   # (轮名, 登记行里轮名之后的各列（日期起）或 None（不登记）, 应得, 项名)
            ("r-ab-ok", registered_on + "用户叫停\trecords/ab-sample.md 第 1 行〔r-ab-ok 叫停〕", 0, "快照:叫停的轮在登记表里有出处完整的一行，放行"),
            ("r-ab-colon", registered_on + "用户叫停\trecords/ab-sample.md:7〔r-ab-colon 叫停〕", 0, "快照:出处写成 文件:行号 也认，放行"),
            ("r-ab-none", None, 2, "快照:登记表里没有这一轮，照拒"),
            ("r-ab-nosource", registered_on + "用户叫停了这一轮\t不写判决", 2, "快照:登记行没有出处，照拒"),
            ("r-ab-noline", registered_on + "用户叫停\trecords/ab-sample.md", 2, "快照:出处没写行号，照拒"),
            ("r-ab-badline", registered_on + "用户叫停\trecords/ab-sample.md 第 99 行〔r-ab-badline 叫停了这一轮〕", 2, "快照:出处的行号超出文件、〔〕原话也找不到，照拒"),
            ("r-ab-drift", registered_on + "用户叫停\trecords/ab-sample.md 第 2 行〔r-ab-drift 叫停〕", 0, "快照:出处的行号对不上而〔〕原话恰好找到一次（行号只当提示），放行"),
            ("r-ab-gone", registered_on + "用户叫停\trecords/ab-sample.md 第 2 行〔r-ab-gone 叫停了〕", 2, "快照:〔〕里的原话在出处文件里找不到（被删或改了字），照拒"),   # 行号提示那一行没有这一轮的名字
            ("r-ab-twice", registered_on + "用户叫停\trecords/ab-sample.md 第 26 行〔r-ab-twice 叫停〕", 2, "快照:〔〕里的原话在出处文件里命中两次，照拒"),   # 行号提示不在命中的两处里
            ("r-ab-pinned", registered_on + "用户叫停\trecords/ab-sample.md 第 24 行〔r-ab-pinned 叫停〕", 0, "快照:〔〕里的原话命中两次而行号提示落在其中一处，取那一行，放行"),
            ("r-ab-reworded", registered_on + "用户叫停\trecords/ab-sample.md 第 25 行〔r-ab-reworded 已叫停〕", 2, "快照:〔〕原话找不到、行号提示那一行有这一轮的名字与「」原话而〔〕对不上（转述或改了字，误登记），照拒"),
            ("r-ab-renamed", registered_on + "用户叫停\trecords/ab-sample.md 第 27 行〔r-ab-renamed 叫停：「旧称样本那一轮先停掉吧」〕", 0, "快照:〔〕是改名前的原话、行号提示那一行照改名表换了词，取那一行，放行"),
            ("r-ab-dated", registered_on + "用户叫停\trecords/ab-sample.md 第 28 行〔r-ab-dated 叫停（2026-10-12，用户说「删了日期之后的六个字以上」）〕", 0, "快照:〔〕带着记录里后来删掉的日期，取行号提示那一行，放行"),
            # 日期单独写在括号里、记录里连括号删掉：去掉日期之后剩下空括号，也要去掉（LIB_ABANDONED_ROUNDS_BREAK=empty-parentheses-kept 下这两格红）
            ("r-ab-fullparen", registered_on + "用户叫停\trecords/ab-sample.md 第 29 行〔r-ab-fullparen（2026-10-12）叫停〕", 0,
             "快照:〔〕带着写在全角括号里、记录里后来连括号删掉的日期，取行号提示那一行，放行"),
            ("r-ab-halfparen", registered_on + "用户叫停\trecords/ab-sample.md 第 30 行〔r-ab-halfparen (2026-10-12) 叫停〕", 0,
             "快照:〔〕带着写在半角括号里、记录里后来连括号删掉的日期，取行号提示那一行，放行"),
            ("r-ab-plain", registered_on + "用户叫停\trecords/ab-sample.md 第 22 行", 2, "快照:出处没带〔〕原话，照拒"),
            ("r-ab-unrel", registered_on + "用户叫停\trecords/ab-sample.md 第 2 行〔这是一句和任何一轮都无关的话〕", 2, "快照:那一行没有这一轮的名字（引无关的一句），照拒"),
            ("r-ab-borrow", registered_on + "用户叫停\trecords/ab-sample.md 第 3 行〔r-ab-other 叫停〕", 2, "快照:那一行是别的轮的叫停原话，照拒"),
            ("r-ab-short", registered_on + "用户叫停\trecords/ab-sample.md 第 4 行〔r-ab-short 叫停〕", 2, "快照:原话不到 6 个字，照拒"),
            ("r-ab-noquote", registered_on + "用户叫停\trecords/ab-sample.md 第 5 行〔r-ab-noquote 叫停〕", 2, "快照:那一行没有「」括的原话，照拒"),
            ("r-ab-edge", registered_on + "用户叫停\trecords/ab-sample.md 第 6 行〔r-ab-edge2 叫停〕", 2, "快照:那一行的名字是更长的另一轮名（前后连着轮名字符），照拒"),
            ("r-ab-missing", registered_on + "用户叫停\trecords/不在的文件.md 第 1 行〔r-ab-missing 叫停〕", 2, "快照:出处点名的文件不在，照拒"),
            ("r-ab-self", registered_on + "用户叫停\tresearch/prompts/_r-ab-self-body.md 第 1 行〔r-ab-self 正文〕", 2, "快照:出处指这一轮自己的正文，照拒"),
            ("r-ab-halted-cite", registered_on + "用户叫停\tresearch/prompts/r-ab-halted-cite-halted.md 第 1 行〔r-ab-halted-cite 叫停〕", 2, "快照:出处指这一轮自己的 halted 文件，照拒"),
            ("r-ab-rule", registered_on + "用户叫停\t.claude/rules/ab-rule-sample.md 第 1 行〔r-ab-rule 的叫停〕", 2, "快照:出处指规则文件，照拒"),
            ("r-ab-dotdot", registered_on + "用户叫停\trecords/../research/prompts/_r-ab-dotdot-body.md 第 1 行〔r-ab-dotdot 正文〕", 2, "快照:出处路径里有 ..（解到仓内别处），照拒"),
            ("r-ab-outside", registered_on + f"用户叫停\trecords/{outside_from_records} 第 1 行〔r-ab-outside 叫停〕", 2, "快照:出处用 .. 指到仓外，照拒"),
            ("r-ab-symlink", registered_on + "用户叫停\trecords/ab-link.md 第 1 行〔r-ab-symlink 叫停〕", 2, "快照:出处是 records/ 下指到仓外的符号链接，照拒"),
            ("r-ab-research-outside", registered_on + f"用户叫停\tresearch/{outside_from_research} 第 1 行〔r-ab-research-outside 叫停〕", 2, "快照:出处从 research/ 起头 .. 到仓外，照拒"),
            ("r-ab-badrow", registered_on + "records/ab-sample.md 第 8 行〔r-ab-badrow 叫停〕", 2, "快照:登记行不是四列（门禁也判红），不收"),
            ("r-ab-halted-file", None, 2, "快照:只写 halted 文件、没登记，照拒（这一支已撤）"),
            # 同一张表两边一个判法（lib_abandoned_rounds.py）：下面这几行门禁 abandoned-rounds 格同样判红
            ("r-ab-multi-r1", registered_on + "用户叫停\trecords/ab-sample.md 第 10 行〔r-ab-multi-r1 照常跑〕", 2, "快照:出处那一行同时写着别的轮的叫停原话（轮名形如 <名>-r<数字>），照拒"),
            ("r-ab-pair", registered_on + "用户叫停\trecords/ab-sample.md 第 11 行〔r-ab-pair 与 r-ab-ok 一起叫停〕", 2, "快照:出处那一行同时写着登记表里别的轮名，照拒"),
            ("r-ab-real-r1", registered_on + "用户叫停\trecords/ab-sample.md 第 12 行〔r-ab-real-r1 叫停〕", 0, "快照:轮名形如 <名>-r<数字>、出处那一行只有这一轮的名字，放行"),
            ("r-ab-nameform", registered_on + "用户叫停\trecords/ab-sample.md 第 15 行〔r-ab-nameform 叫停〕", 2, "快照:出处那一行同时写着仓里认得的别的轮名（不以 -r<数字> 结尾），照拒"),
            ("r-ab-unfiled", registered_on + "用户叫停\trecords/ab-sample.md 第 16 行〔r-ab-unfiled 叫停〕", 0, "快照:出处那一行另有仓里没有任何材料、也不是 <名>-r<数字> 形的名字，不当别的轮名，放行"),
            ("r-ab-formonly", registered_on + "用户叫停\trecords/ab-sample.md 第 17 行〔r-ab-formonly 叫停〕", 2, "快照:出处那一行同时写着 <名>-r<数字> 形而仓里没有材料的别的轮名，照拒"),
            ("r-ab-baddate", "10月13日\t用户叫停\trecords/ab-sample.md 第 13 行〔r-ab-baddate 叫停〕", 2, "快照:日期不是 YYYY-MM-DD（门禁也判红），出处合格也不收"),
            ("r-ab-dup", registered_on + "用户叫停\trecords/ab-sample.md 第 14 行〔r-ab-dup 叫停〕", 2, "快照:同一轮登记两行（两行出处都合格，门禁也判红），不收"),
            ("r-ab-nosource-new", registered_on + "用户叫停，原话见调度记录\t不写判决", 2, "快照:出处没写，2026-10-12 的行照拒（门禁同样判红）"),
            ("r-ab-nosource-dated", "2026-10-11\t用户叫停，原话见调度记录\t不写判决", 2, "快照:出处没写，2026-10-11 的行也照拒（不按日期免；门禁同样判红）"),
        ]
        write_sample(work, "research/prompts/r-ab-halted-cite-halted.md", "r-ab-halted-cite 叫停「这是一句自己写的六个字以上的话」\n")
        write_sample(work, "research/prompts/r-ab-halted-file-halted.md", "出处：records/ab-sample.md 第 1 行「增加流程限制 测试代码不需要打，改了直接测。」\n")
        table_lines = ["# 样本：叫停的三方轮次"]
        abandoned_messages = {}
        for abandoned_round, abandoned_row, abandoned_want, abandoned_label in abandoned_rounds:
            write_sample(work, f"research/prompts/{abandoned_round}-snapshot/kb-sha256.txt", f"{digest}  .claude/kb/{abandoned_round}.md\n")
            if not os.path.exists(os.path.join(work, "research", "prompts", f"_{abandoned_round}-body.md")):
                write_sample(work, f"research/prompts/_{abandoned_round}-body.md", f"# {abandoned_round} 正文\n" if abandoned_round != "r-ab-self"
                             else "r-ab-self 正文「这是一句自己写的六个字以上的话」\n")
            if abandoned_row is not None:
                table_lines.append(f"{abandoned_round}\t{abandoned_row}")   # 轮名、日期、为什么撂下、去向；r-ab-badrow 只有三列
        table_lines.append(f"r-ab-dup\t{registered_on}用户叫停（第二行）\trecords/ab-sample.md 第 14 行〔r-ab-dup 叫停〕")   # r-ab-dup 的第二行：出处合格，两行都合格，只因同一轮两行不收
        write_sample(work, "research/prompts/abandoned-rounds.tsv", "\n".join(table_lines) + "\n")
        for abandoned_round, abandoned_row, abandoned_want, abandoned_label in abandoned_rounds:
            abandoned_prompt = f"规格：改 `.claude/kb/{abandoned_round}.md` 里一行。\n" + FOOTER
            abandoned_hook_input = {"tool_name": "Agent", "tool_input": {"subagent_type": "kb-scribe", "prompt": abandoned_prompt}}
            abandoned_code, abandoned_messages[abandoned_round] = runner_dispatch_guard_locks_and_verdicts.decide(abandoned_hook_input, work, hook_dir)
            cases.append((abandoned_label, abandoned_want, abandoned_code))
        def abandoned_message_has(abandoned_round, *fragments):
            return all(fragment in (abandoned_messages[abandoned_round] or "") for fragment in fragments)
        cases.append(("快照:不合格的拒绝说明写了没收的原因（原话找不到、没有名字、原话太短、路径里有 ..、指到仓外、没有出处）", True,
                      abandoned_message_has("r-ab-badline", "收不了这一轮", "找不到〔r-ab-badline 叫停了这一轮〕")
                      and abandoned_message_has("r-ab-unrel", "没有这一轮的名字 r-ab-unrel")
                      and abandoned_message_has("r-ab-short", "至少 6 个字")
                      and abandoned_message_has("r-ab-dotdot", "路径里有 ..")
                      and abandoned_message_has("r-ab-symlink", "不在", "符号链接")
                      and abandoned_message_has("r-ab-outside", "路径里有 ..")
                      and abandoned_message_has("r-ab-nosource", "没有出处", "records/<文件>.md 第 N 行")))
        cases.append(("快照:拒绝说明的出路改道登记表、不再写 halted", True,
                      abandoned_message_has("r-ab-none", "research/prompts/abandoned-rounds.tsv 加一行", "doc-process-records", "records/<文件>.md 第 N 行")
                      and "-halted.md 收掉它" not in (abandoned_messages["r-ab-none"] or "")))
        cases.append(("快照:halted 文件还在时拒绝说明提醒这一支已撤、改道登记表", True,
                      abandoned_message_has("r-ab-halted-file", "r-ab-halted-file-halted.md 在，但派发闸不再认这一支")))
        cases.append(("快照:写坏的登记行在说明里点名第几行", True, abandoned_message_has("r-ab-badrow", "不是四列、或有空列")))
        cases.append(("快照:原话找不到、命中两次、没带原话的拒绝说明写了原因", True,
                      abandoned_message_has("r-ab-gone", "找不到〔r-ab-gone 叫停了〕这段原话", "行号提示的第 2 行里也没有这一轮的名字 r-ab-gone")
                      and abandoned_message_has("r-ab-twice", "命中 2 次（第 20、21 行），行号提示的第 26 行不在其中")
                      and abandoned_message_has("r-ab-plain", "没带〔〕括的原话")))
        cases.append(("快照:〔〕原话找不到、行号提示那一行对不上的拒绝说明写了去掉改名与日期之后对不上", True,
                      abandoned_message_has("r-ab-reworded", "找不到〔r-ab-reworded 已叫停〕这段原话", "行号提示的第 25 行有这一轮的名字，但去掉改名表")))
        cases.append(("快照:别的轮名、日期写坏、同一轮两行的拒绝说明写了原因", True,
                      abandoned_message_has("r-ab-multi-r1", "除这一轮外还有别的轮名 r-ab-away-r1")
                      and abandoned_message_has("r-ab-pair", "除这一轮外还有别的轮名 r-ab-ok")
                      and abandoned_message_has("r-ab-nameform", "除这一轮外还有别的轮名 ab-close-b1")
                      and abandoned_message_has("r-ab-formonly", "除这一轮外还有别的轮名 ab-ghost-r2")
                      and abandoned_message_has("r-ab-baddate", "日期不是 YYYY-MM-DD：10月13日")
                      and abandoned_message_has("r-ab-dup", "登记了不止一行")))
        # ── ⑰ 报告行：四种标签的报告行正文里的文件名带 report（不分大小写）拒，result.md 放行；三方腿另判形态 ──
        # 报告行点名的路径只在落在本提示草稿目录下时判：样本都带骨架那一行「草稿目录：」（这几格直接调 decide，不建写者锁；
        # 走 stdin 的那一格另用不带草稿目录一行的提示，不在真工作根里建锁）
        report_scratch_line = f"草稿目录：{home_new_root}/w/\n"
        report_name_head = report_scratch_line + f"报告：{home_new_root}/w/"
        cases += [
            case("报告名:报告：一行写 report.md 拒", "clerk", report_name_head + "report.md\n", 2),
            case("报告名:名字里带 report 的别的文件也拒", "clerk", report_name_head + "final-report.md\n", 2),
            case("报告名:大写也拒（不分大小写）", "clerk", report_name_head + "Report.MD\n", 2),
            case("报告名:粗体与列表记号也认", "clerk", report_scratch_line + f"- **报告**：{home_new_root}/w/report.md\n", 2),
            case("报告名:反引号包着的路径也认", "clerk", report_scratch_line + f"报告：`{home_new_root}/w/report.md`\n", 2),
            case("报告名:一行写几个路径，有一个带 report 就拒", "clerk", report_name_head + f"result.md，副本 {home_new_root}/w/report.md\n", 2),
            case("报告名:代码块之后行首的那一行照判", "clerk", "```\n" + report_name_head + "result.md\n```\n" + report_name_head + "report.md\n", 2),
            case("报告名:result.md 放行", "clerk", report_name_head + "result.md\n", 0),
            case("报告名:目录名带 report、文件名不带放行", "clerk", f"草稿目录：{home_new_root}/reports/\n报告：{home_new_root}/reports/result.md\n", 0),
            case("报告名:代码块里的示例行不判放行", "clerk", report_name_head + "result.md\n示例：\n```\n" + report_name_head + "report.md\n```\n", 0),
            case("报告名:只提到 report 这个词、不是文件名的放行", "clerk", report_name_head + "result.md，不要叫 report\n", 0),
            # 四种标签共用一份报告行解析（开关 report-line-labels-split 下这四格红）
            case("报告名:「报告路径：」一行写 report.md 也拒", "clerk", report_scratch_line + f"报告路径：{home_new_root}/w/report.md\n", 2),
            case("报告名:「报告写进」后面写 report.md 也拒", "clerk", report_scratch_line + f"报告写进 {home_new_root}/w/report.md\n", 2),
            case("报告名:「报告写到」后面写 report.md 也拒", "clerk", report_scratch_line + f"报告写到：{home_new_root}/w/report.md\n", 2),
            case("报告名:「交回报告：」这类前面连着字的不是报告行", "clerk", report_name_head + "result.md\n交回报告：不要写成 notes/report.md 这种名字\n", 0),
            # 标签只在行首（列表记号、粗体、编号之后也算）或句首认（开关 report-line-unanchored 下引号与句中两格红）
            case("报告名:正文引号里的报告标签不认", "clerk", report_name_head + "result.md\n"
                 f"另判一处：骨架「报告：」一行默认填 `<草稿目录>/report.md`。\n转述『报告路径：{home_new_root}/w/report.md』这一句。\n"
                 f"“报告写进 {home_new_root}/w/report.md”是旧写法。\n照 `报告：{home_new_root}/w/report.md` 那样写的会被拒。\n", 0),
            # 按草稿目录判（开关 report-line-position-anchored 下逗号之后、转述、没草稿目录一行、交付写法几格红）
            # 草稿目录写在句中、没有行首「草稿目录：」一行的（不照骨架写的工具员提示）：本提示的草稿目录认不出，只判裸文件名
            case("报告名:草稿目录只写在句中（没有行首「草稿目录：」一行），带目录的报告路径不判", "clerk",
                 f"批次：x#1。回复用中文。草稿目录 `{home_new_root}/w/`（自己建，排他），报告 `{home_new_root}/w/report.md`，交回只写结论。\n", 0,
                 ignored=("批次",)),
            case("报告名:句中逗号之后、落在本提示草稿目录下的「报告写到」照认", "clerk",
                 f"草稿目录：{home_new_root}/w\n两份登记写进副本，报告写到 `{home_new_root}/w/report.md`：两份登记的路径。\n", 2),
            case("报告名:句首转述别的草稿目录下的 report.md 不判", "clerk", report_name_head + "result.md\n"
                 f"底稿：{home_new_root}/other-runner/x.rs（编过、clippy 0；报告 {home_new_root}/other-runner/report.md）。\n"
                 f"上一批照批 9 改过。报告 `{home_new_root}/w9-pins/report.md`，数去补丁现查。\n", 0),
            case("报告名:没写草稿目录一行时，带目录的路径不判", "clerk", f"报告：{home_new_root}/w/report.md\n", 0),
            case("报告名:没写草稿目录一行时，裸文件名照判", "clerk", "报告写到 report.md\n", 2),
            case("报告名:草稿目录一行点名的不在工作根下时，带目录的路径不判", "clerk", "草稿目录：/var/singlefs-selftest-elsewhere/\n报告：/var/singlefs-selftest-elsewhere/report.md\n", 0,
                 ignored=("草稿目录落点", "报告落点")),
            case("报告名:草稿目录一行点名的不在工作根下时，裸文件名照判", "clerk", "草稿目录：/var/singlefs-selftest-elsewhere/\n报告写到 report.md\n", 2, ignored=("草稿目录落点",)),
            case("报告名:不带目录的裸文件名照判", "clerk", report_scratch_line + "报告写到草稿目录的 report.md\n", 2),
            case("报告名:「<草稿目录>/report.md」写法照判", "clerk", "报告：<草稿目录>/report.md\n", 2),
            case("报告名:三方腿报告行写的本腿报告路径 research/prompts/<名>-output.md 名字里带 report 照判", "three-way-attack", "报告：research/prompts/x-report-r1-opus-output.md\n", 2),
            case("报告名:三方腿整份提示里本腿轮名下的报告路径名字里带 report 照判", "three-way-defense",
                 "背景材料：research/prompts/_x-r1-background.md\n照正文交付，分段落盘到 research/prompts/x-r1-sonnet-report-output.md。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("报告名:三方腿的报告路径以 SUMMARY 开头照判", "three-way-forward", "报告：research/prompts/SUMMARY-x-r1-sonnet-output.md\n", 2),
            case("报告名:三方腿的报告路径名字里不带 report 放行", "three-way-attack", "报告：research/prompts/x-r1-opus-output.md\n", 0),
            case("报告名:别的类型报告行写 research/prompts/ 下名字带 report 的，不在草稿目录下当转述不判", "clerk", report_name_head + "result.md\n"
                 "上一轮的报告 research/prompts/x-report-r1-opus-output.md 照读。\n", 0),
            case("报告名:句号之后的报告标签照认", "clerk", report_scratch_line + f"轮名：x-r1。报告：{home_new_root}/w/report.md。\n", 2),
            case("报告名:编号列表记号之后的报告标签照认", "clerk", report_scratch_line + f"1. **报告**：{home_new_root}/w/report.md\n", 2),
            # 紧跟否定词的文件名不判（开关 report-name-negation-judged 下这一格红）
            case("报告名:报告行里否定词后面点名的 report.md 不判", "clerk", report_name_head + "result.md（不要叫 report.md）\n", 0),
            # 「」『』“”照整份配对，折到下一行的引文后半截照样算引号里的（拆分提案第九十节 ㉟；开关 quote-pairs-per-line 下这两格红）
            case("报告名:跨行的「」引文里的报告标签不认", "clerk", report_name_head + "result.md\n转述上一轮的提示「骨架改之前写的是\n"
                 f"报告：{home_new_root}/w/report.md」，现在不这么写。\n", 0),
            ("引号:「」照整份配对、可以跨行、配最近的那个开头，反引号按行配对", [(5, 8), (10, 13), (14, 22)], sorted(quoted_spans("a「b\nc『d』\ne`x`f「g\n`h\ni」"))),
            case("报告名:没写报告行不判", "clerk", "改完交回。\n", 0),
        ]
        # 「本提示的草稿目录」只认行首「草稿目录：」一行：转述里的别处不当本提示的草稿目录，
        # 「草稿目录在 <路径>」不是那一行、只判裸文件名。开关 LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere 下转述那两格红
        relay_head = report_scratch_line + f"报告：{home_new_root}/w/result.md\n"
        cases += [
            case("草稿目录一行:转述里「，草稿目录 <别处>」下的 report.md 不判", "clerk", relay_head
                 + f"前一段交回在先，草稿目录 {home_new_root}/previous/ 里有它的探针，报告 {home_new_root}/previous/report.md 是那一段写的，只读不改。\n", 0),
            case("草稿目录一行:列表里「- 草稿目录 <别处>」下的 report.md 不判", "clerk", relay_head
                 + f"要读的：\n- 草稿目录 {home_new_root}/previous/ 下的 logs/\n- 报告 {home_new_root}/previous/report.md（前一段的）\n", 0),
            case("草稿目录一行:「草稿目录在 <路径>」不是那一行，带目录的 report.md 不判", "clerk",
                 f"草稿目录在 {home_new_root}/w（自己建）\n报告：{home_new_root}/w/report.md\n", 0),
            case("草稿目录一行:「草稿目录在 <路径>」的提示照判裸文件名", "clerk", f"草稿目录在 {home_new_root}/w（自己建）\n报告写到草稿目录的 report.md\n", 2),
        ]
        # ⑯ ⑰ 调的是同一份认法：把 lib_prompt_scratch_line.py 的 scratch_directories 换成替身，两处交回的都是替身给的
        library_directories = runner_dispatch_guard_prompt_checks.prompt_scratch_line.scratch_directories
        runner_dispatch_guard_prompt_checks.prompt_scratch_line.scratch_directories = lambda prompt, include_roots=False: ["/sentinel-from-library"] + (["/sentinel-root"] if include_roots else [])
        try:
            routed = (report_scratch_directories("没有草稿目录一行"), scratch_directories_named("没有草稿目录一行"))
        finally:
            runner_dispatch_guard_prompt_checks.prompt_scratch_line.scratch_directories = library_directories
        cases.append(("草稿目录一行:⑰ 与 ⑯ 调的是 lib_prompt_scratch_line.py 的 scratch_directories", (["/sentinel-from-library"], ["/sentinel-from-library"]), routed))
        # 以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 .md 工具层同样拒（拆分提案第九十节 ㉑，实测 b9-prefix-probe.md）；名字中间或末尾带的、不是 .md 的放行
        # （开关 report-name-prefixes-unchecked 下拒的几格与拒绝说明那一格红）
        cases += [
            case("报告名:草稿目录下以 SUMMARY 开头的 .md 拒", "clerk", report_name_head + "SUMMARY.md\n", 2),
            case("报告名:草稿目录下以 findings 开头的 .md 拒（小写）", "clerk", report_name_head + "findings-x.md\n", 2),
            case("报告名:草稿目录下以 Analysis 开头的 .md 拒（大小写混写）", "clerk", report_name_head + "Analysis-notes.md\n", 2),
            case("报告名:不带目录的裸文件名以 summary 开头的 .md 照判", "clerk", report_scratch_line + "报告写到草稿目录的 summary.md\n", 2),
            case("报告名:前缀在名字中间或末尾的放行", "clerk", report_name_head + "x-summary.md\n", 0),
            case("报告名:以 summary 开头但不是 .md 的放行", "clerk", report_name_head + "summary.txt\n", 0),
            # 扩展名照工具层也不分大小写（实测 b12-scratch-lib.md「6.1 工具层真调」：SUMMARY.MD 拒、summary.markdown 放；开关 report-name-prefix-extension-lowercase 下 .MD 那一格红）
            case("报告名:以 SUMMARY 开头、扩展名写成大写 .MD 的照拒（工具层同样拒）", "clerk", report_name_head + "SUMMARY.MD\n", 2),
            case("报告名:以 summary 开头的 .markdown 放行（工具层放）", "clerk", report_name_head + "summary.markdown\n", 0),
            case("报告名:别的草稿目录下以 SUMMARY 开头的 .md 当转述不判", "clerk", report_scratch_line + f"报告：{home_new_root}/other/SUMMARY.md\n", 0),
        ]
        prefix_name_code, prefix_name_message = runner_dispatch_guard_locks_and_verdicts.decide({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk", "prompt": report_name_head + "FINDINGS.md\n"
                                                                                             + FOOTER}}, work, hook_dir)
        cases.append(("报告名:以三个前缀开头的 .md 拒绝说明点名文件名、三个前缀与 result.md 的写法", True,
                      prefix_name_code == 2 and f"「{home_new_root}/w/FINDINGS.md」" in (prefix_name_message or "")
                      and "SUMMARY、FINDINGS、ANALYSIS" in (prefix_name_message or "") and "草稿目录下的 result.md" in (prefix_name_message or "")))
        report_name_input = {"tool_name": "Agent", "tool_input": {"subagent_type": "clerk", "prompt": report_name_head + "report.md\n" + FOOTER}}
        report_name_code, report_name_message = runner_dispatch_guard_locks_and_verdicts.decide(report_name_input, work, hook_dir)
        cases.append(("报告名:拒绝说明点名文件名、规矩与 result.md 的写法", True,
                      report_name_code == 2 and "报告行点名的文件名里带 report" in (report_name_message or "") and f"「{home_new_root}/w/report.md」" in (report_name_message or "")
                      and "Subagents should return findings as text" in (report_name_message or "") and "→ 怎么办" in (report_name_message or "")
                      and "草稿目录下的 result.md" in (report_name_message or "")))
        # ⑱ 两处落点的拒绝说明（written_work_root）：工作根不在 ~ 下（第二台的家目录是另一个，工作根是共享的那一份）时写工作根的绝对路径，不写成 ~/../…；
        #   这一格在进程里把 SINGLEFS_WORK_ROOT 换成一个不在家目录下的根、直接调两处判法，判完换回（开关 scratch-line-root-written-under-home 下红）
        outside_home_root = "/nonexistent-work-root-outside-home/work"
        saved_work_root = os.environ.get(work_roots.WORK_ROOT_VARIABLE)
        os.environ[work_roots.WORK_ROOT_VARIABLE] = outside_home_root
        try:
            outside_scratch_message = scratch_line_root_verdict("clerk", "草稿目录：/var/tmp/b12-root-written/\n") or ""
            outside_report_message = report_location_verdict("clerk", "报告：/var/tmp/b12-root-written/result.md\n", work) or ""
        finally:
            if saved_work_root is None:
                os.environ.pop(work_roots.WORK_ROOT_VARIABLE, None)
            else:
                os.environ[work_roots.WORK_ROOT_VARIABLE] = saved_work_root
        cases.append(("落点说明:工作根不在 ~ 下时草稿目录与报告两处拒绝说明都写工作根的绝对路径", True,
                      not os.path.expanduser("~").startswith("/nonexistent-work-root-outside-home")
                      and all(f"不在工作根 {outside_home_root}/ 底下" in message and "~/.." not in message for message in (outside_scratch_message, outside_report_message))))
        report_name_refusal = run_hook(work, json.dumps({"tool_name": "Agent", "tool_input": {"subagent_type": "clerk", "prompt": "报告：report.md\n" + FOOTER}}))
        cases.append(("stdin:报告文件名带 report 拒绝（退出码 2）", 2, report_name_refusal.returncode))
        cases.append(("stdin:报告文件名拒绝的 stderr 写了 result.md 的写法与骨架", 1,
                      int(all(needle in report_name_refusal.stderr for needle in ("报告行点名的文件名里带 report", "result.md", "dispatch-prompt.py", "→ 怎么办")))))
        # 报告行里还有占位的：只交 ⑬，④ 不判写范围（实现员写范围里没有 research/）
        placeholder_report = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "experiment-runner", "prompt": batch_lines + f"草稿目录 {home_new_root}/e/\n"
                                                    "报告：research/prompts/<填 轮>-runner-result.md\n这一段回答的岔路：岔路 1\n"}}, defs_work, hook_dir)
        cases.append(("写范围:报告行里还有占位的交骨架占位那一条、写范围不判", True,
                      placeholder_report[0] == 2 and "占位没填" in (placeholder_report[1] or "") and "写范围之外" not in (placeholder_report[1] or "")))
        # ⑰ 三方腿的报告形态（开关 three-way-report-unchecked 下拒的几格红，report-line-labels-split 下括注那一格红）
        cases += [
            case("三方报告:正推腿报告写 research/prompts/<轮>-<模型>-output.md 放行", "three-way-forward", "报告：research/prompts/x-r1-sonnet-output.md\n", 0),
            case("三方报告:攻方腿报告写 research/prompts/x-r1-opus-output.md 放行", "three-way-attack", "报告：research/prompts/x-r1-opus-output.md\n", 0),
            case("三方报告:报告路径写成仓根下的绝对路径放行", "three-way-defense", f"报告路径：{work}/research/prompts/x-r1-sonnet-output.md\n", 0, ignored=("三方腿报告行",)),
            case("三方报告:报告路径一行写在括注里的形态、后面接真路径放行", "three-way-forward",
                 "报告路径（形态 `research/prompts/<轮>-sonnet-output.md`）：research/prompts/x-r1-sonnet-output.md\n", 0, ignored=("三方腿报告行",)),
            case("三方报告:攻方腿报告写草稿目录下的 result.md 拒", "three-way-attack",
                 f"草稿目录：{home_new_root}/x-r1-opus/\n报告：{home_new_root}/x-r1-opus/result.md\n", 2),
            case("三方报告:正推腿报告写草稿目录下的 result.md 拒", "three-way-forward",
                 f"草稿目录：{home_new_root}/x-r1-sonnet/\n报告：{home_new_root}/x-r1-sonnet/result.md\n", 2),
            case("三方报告:草稿目录下的 result.md 写在逗号之后照拒", "three-way-attack",
                 f"草稿目录：{home_new_root}/x-r1-opus/\n轮名：x-r1，报告 {home_new_root}/x-r1-opus/result.md\n", 2, ignored=("三方腿报告行",)),
            case("三方报告:「交付：报告 …-output.md」写法放行", "three-way-attack",
                 "轮名：x-r1。前几轮的判决：无（第一轮）。交付：报告 research/prompts/x-r1-opus-output.md（分段落盘）。\n", 0, ignored=("三方腿报告行",)),
            case("三方报告:报告行之外不在引号里、本腿轮名下的 research/prompts/<名>-output.md 也认放行", "three-way-defense",
                 "背景材料：research/prompts/_x-r1-background.md\n照正文第五节交付，分段落盘到 research/prompts/x-r1-sonnet-output.md，交回只写路径与判定一览。\n", 0, report=False, ignored=("三方腿报告行",)),
            case("三方报告:只在引号或反引号里出现的 -output.md 不算，拒", "three-way-defense",
                 "背景材料：research/prompts/_x-r1-background.md\n形态照 `research/prompts/x-r1-sonnet-output.md`；「research/prompts/x-r0-sonnet-output.md」是上一轮的。\n", 2, report=False, ignored=("三方腿报告行",)),
            # 本腿的报告只认报告行与本腿轮名下的；禁读清单里的是别的腿的（拆分提案第九十节 ㉗；开关 leg-report-forbidden-counted 下禁读那三格红、leg-report-any-round 下别的轮那两格红）
            case("三方报告:禁读清单里另一条腿的 -output.md 不当本腿的报告，拒", "three-way-forward",
                 "背景材料：research/prompts/_x-r1-background.md\n禁读清单：research/prompts/x-r1-opus-output.md（同一轮攻方腿的报告）\n照正文交付，回复只写路径。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:禁读清单冒号后空着、往下列表项里另一条腿的 -output.md 也不当本腿的报告，拒", "three-way-forward",
                 "背景材料：research/prompts/_x-r1-background.md\n禁读清单：\n- research/prompts/x-r1-opus-output.md\n- research/prompts/x-r1-opus-model/\n照正文交付。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:标题「禁读」一节里的 -output.md 不当本腿的报告，拒", "three-way-forward",
                 "背景材料：research/prompts/_x-r1-background.md\n## 禁读\n\nresearch/prompts/x-r1-opus-output.md 与它的模型目录。\n\n## 交付\n\n照正文交付。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:整份提示里只有别的轮的 -output.md（背景里的上一轮腿报告），拒", "three-way-defense",
                 "背景材料：research/prompts/_x-r2-background.md\n前一轮腿报告 research/prompts/x-r1-opus-output.md 在背景材料里。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:轮名 x-r1 不认 x-r10 下的 -output.md，拒", "three-way-defense", "轮名：x-r1\n交付到 research/prompts/x-r10-sonnet-output.md。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:轮名取「轮名」标签，整份提示里本腿轮名下的 -output.md 认，放行", "three-way-defense",
                 "轮名 `x-r2`（第二轮）。照正文交付，分段落盘到 research/prompts/x-r2-sonnet-output.md。\n", 0, report=False, ignored=("三方腿报告行",)),
            case("三方报告:禁读清单只列另一条腿的报告，报告行写本腿的放行", "three-way-attack",
                 "禁读清单：research/prompts/x-r1-sonnet-output.md\n报告：research/prompts/x-r1-opus-output.md\n", 0),
            ("三方报告:本腿轮名取背景材料一行与「轮名」标签，引号里的标签不算", ["x-r2", "y-r1"],
             leg_round_names("背景材料：research/prompts/_x-r2-background.md（正文 research/prompts/_x-r2-body.md）\n轮名：y-r1。\n「轮名：z-r9」是转述。\n")),
            # 禁读清单里点到本腿自己的报告拒（拆分提案第九十节 ㉙；开关 leg-report-forbidden-own-unchecked 下这两格与说明那一格红）
            case("三方报告:禁读清单里点到本腿自己的报告路径拒（landing-audit-r4 攻方提示头一版的写法）", "three-way-attack",
                 "背景材料：research/prompts/_x-r1-background.md\n禁读清单：research/prompts/x-r1-opus-output.md（同一轮的正推腿报告）\n报告：research/prompts/x-r1-opus-output.md\n", 2),
            case("三方报告:禁读清单里点到本腿自己的报告，写在反引号里也拒", "three-way-attack",
                 "禁读清单：`research/prompts/x-r1-opus-output.md`、`research/prompts/x-r1-opus-model/`\n报告：research/prompts/x-r1-opus-output.md\n", 2),
            # 跨行的「」引文里的 -output.md 不当本腿的报告（拆分提案第九十节 ㉟；开关 quote-pairs-per-line 下这一格红）
            case("三方报告:跨行的「」引文里的 -output.md 不当本腿的报告，拒", "three-way-defense",
                 "背景材料：research/prompts/_x-r1-background.md\n转述「上一份提示写的是\n分段落盘到 research/prompts/x-r1-sonnet-output.md」，这一份没写报告。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:转述别的草稿目录下的报告不当自己的报告行", "three-way-forward",
                 f"报告：research/prompts/x-r1-sonnet-output.md\n上一轮核查员交回了。报告 {home_new_root}/x-r0-verifier/result.md，数照它。\n", 0),
            case("三方报告:辩方腿没写报告行拒", "three-way-defense", "照判决辩护。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:报告行写别的仓内文件拒", "three-way-attack", "报告：research/prompts/x-r1-attack.md\n", 2),
            case("三方报告:别的类型不判形态（clerk 写 result.md）", "clerk", report_name_head + "result.md\n", 0),
            case("三方报告:句号之后的报告行照认放行", "three-way-attack",
                 "轮名：x-r1。报告：research/prompts/x-r1-opus-output.md。模型目录：research/prompts/x-r1-opus-model/。\n", 0, ignored=("三方腿报告行",)),
            case("三方报告:正文引号里的「报告：」不当报告行", "three-way-attack", "报告：research/prompts/x-r1-opus-output.md\n"
                 "另判一处：骨架「报告：」一行默认填 `<草稿目录>/result.md`。\n", 0),
        ]
        forbidden_own_code, forbidden_own_message = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "three-way-attack", "prompt": "背景材料：research/prompts/_x-r1-background.md\n"
                                                                           "禁读清单：research/prompts/x-r1-opus-output.md\n报告：research/prompts/x-r1-opus-output.md\n"
                                                                           + FOOTER}}, work, hook_dir)
        cases.append(("三方报告:禁读清单点到本腿报告的拒绝说明点名那一份与改法", True,
                      forbidden_own_code == 2 and "禁读清单里点到了本腿自己的报告：「research/prompts/x-r1-opus-output.md」" in (forbidden_own_message or "")
                      and "禁读清单只列别的腿的报告与模型目录" in (forbidden_own_message or "") and "→ 怎么办" in (forbidden_own_message or "")))
        three_way_code, three_way_message = runner_dispatch_guard_locks_and_verdicts.decide({"tool_input": {"subagent_type": "three-way-attack", "prompt": f"草稿目录：{home_new_root}/x-r1-opus/\n报告：{home_new_root}/x-r1-opus/result.md\n"
                                                                    + FOOTER}}, work, hook_dir)
        cases.append(("三方报告:拒绝说明给出形态与骨架", True,
                      three_way_code == 2 and "research/prompts/<轮>-<模型>-output.md" in (three_way_message or "")
                      and f"「{home_new_root}/x-r1-opus/result.md」" in (three_way_message or "") and "dispatch-prompt.py three-way-attack" in (three_way_message or "")
                      and "handback-guard.sh" in (three_way_message or "") and "→ 怎么办" in (three_way_message or "")))
        # 三方腿报告路径一处认：没填的 <轮> 占位、子目录下的都不认；形态调的是 lib_three_way_legs.py 那一份
        # （开关 leg-report-placeholder-kept 下占位那一格红）
        cases += [
            case("三方报告:报告行写没填的 <轮> 占位、整份提示没有别的 -output.md，拒", "three-way-attack", "报告：research/prompts/<轮>-opus-output.md\n", 2),
            case("三方报告:报告行写子目录下的 -output.md，拒", "three-way-attack", "报告：research/prompts/x-r1/x-r1-opus-output.md\n", 2),
            case("三方报告:整份提示里只在子目录下有 -output.md，拒", "three-way-defense", "照正文交付，分段落盘到 research/prompts/x-r1/x-r1-sonnet-output.md。\n", 2, report=False, ignored=("三方腿报告行",)),
            # 只认解到仓根下 research/prompts/ 的（与交回闸 ② 同一处认法）：仓外副本里同名的不认（开关 leg-report-outside-root-kept 下这一格红）
            case("三方报告:整份提示里只有仓外同名的 …/research/prompts/…-output.md，拒", "three-way-defense",
                 f"背景材料：research/prompts/_x-r1-background.md\n照正文交付，分段落盘到 {os.path.join(work, 'elsewhere-copy')}/research/prompts/x-r1-sonnet-output.md。\n", 2, report=False, ignored=("三方腿报告行",)),
            case("三方报告:整份提示里写仓根起头的绝对路径，放行", "three-way-defense",
                 f"背景材料：research/prompts/_x-r1-background.md\n照正文交付，分段落盘到 {work}/research/prompts/x-r1-sonnet-output.md。\n", 0, report=False, ignored=("三方腿报告行",)),
        ]
        # 报告行那一支、整份提示那一支与交回闸 ② 都经 lib_three_way_legs.py 的 leg_report_mentions_in_trees 一个函数（拆分提案第九十节 ㉗）：
        # 把它换成替身，两支交回的都是替身认的（开关 leg-report-placeholder-kept、leg-report-line-root-only、leg-report-outside-root-kept 下这一格红）
        library_mentions_in_trees = runner_dispatch_guard_prompt_checks.three_way_legs.leg_report_mentions_in_trees
        runner_dispatch_guard_prompt_checks.three_way_legs.leg_report_mentions_in_trees = lambda text, project_root, trees=None: [(match, project_root) for match in re.finditer(r"sentinel/x-r1-from-library\.md", text or "")]
        try:
            routed_legs = (three_way_report_verdict("three-way-attack", "报告：sentinel/x-r1-from-library.md\n", work) is None,
                           three_way_report_verdict("three-way-attack", "背景材料：research/prompts/_x-r1-background.md\n照正文交付到 sentinel/x-r1-from-library.md。\n", work) is None,
                           three_way_report_verdict("three-way-attack", "报告：research/prompts/x-r1-opus-output.md\n", work) is None)
        finally:
            runner_dispatch_guard_prompt_checks.three_way_legs.leg_report_mentions_in_trees = library_mentions_in_trees
        cases.append(("三方报告:报告行与整份提示两处形态都经 lib_three_way_legs.py 的 leg_report_mentions_in_trees", (True, True, False), routed_legs))
        # 同一组写法，报告行那一支、整份提示那一支与交回闸 ② 的认法（leg_report_mentions_in_trees）三处判得一样
        leg_forms = [("research/prompts/x-r1-opus-output.md", True), ("./research/prompts/x-r1-opus-output.md", True), (f"{work}/research/prompts/x-r1-opus-output.md", True),
                     (f"../{os.path.basename(work)}/research/prompts/x-r1-opus-output.md", True), ("research/prompts/<轮>-opus-output.md", False),
                     ("research/prompts/x-r1/x-r1-opus-output.md", False), (f"{work}/elsewhere-copy/research/prompts/x-r1-opus-output.md", False)]
        truth = [want for _written, want in leg_forms]
        agreed = ([three_way_report_verdict("three-way-attack", f"报告：{written}\n", work) is None for written, _want in leg_forms],
                  [three_way_report_verdict("three-way-defense", f"背景材料：research/prompts/_x-r1-background.md\n照正文交付，分段落盘到 {written}。\n", work) is None
                   for written, _want in leg_forms],
                  [bool(runner_dispatch_guard_prompt_checks.three_way_legs.leg_report_mentions_in_trees(written, work)) for written, _want in leg_forms])
        cases.append(("三方报告:报告行、整份提示与交回闸 ② 三处对同一组写法判得一样", (truth, truth, truth), agreed))
        # $SINGLEFS_WORK_ROOT/ 起头的报告路径先照 lib_work_root.py 展开再认（交回闸 ② 同一种解法，b12-handback.md「(d)」）：工作根这几格临时设成样本仓的上一层，
        # $SINGLEFS_WORK_ROOT/<样本仓目录名>/ 展开后就是样本仓本身（一棵登记过的树）；展开后不在任何一棵树里的照拒（开关 leg-report-work-root-token-kept 下放行那两格红）
        saved_work_root = os.environ.get(work_roots.WORK_ROOT_VARIABLE)
        os.environ[work_roots.WORK_ROOT_VARIABLE] = os.path.dirname(work)
        try:
            work_root_written = f"$SINGLEFS_WORK_ROOT/{os.path.basename(work)}/research/prompts/x-r1-opus-output.md"
            work_root_cells = [
                ("三方报告:报告行写 $SINGLEFS_WORK_ROOT/ 起头、照 lib_work_root.py 展开后解到本仓一棵树的报告路径，放行", True,
                 three_way_report_verdict("three-way-attack", f"报告：{work_root_written}\n", work) is None),
                ("三方报告:整份提示里写 ${SINGLEFS_WORK_ROOT}/ 起头、展开后解到本仓一棵树的报告路径，放行", True,
                 three_way_report_verdict("three-way-defense", "背景材料：research/prompts/_x-r1-background.md\n照正文交付，分段落盘到 ${SINGLEFS_WORK_ROOT}/"
                                          f"{os.path.basename(work)}/research/prompts/x-r1-sonnet-output.md。\n", work) is None),
                ("三方报告:报告行写 $SINGLEFS_WORK_ROOT/ 起头、展开后不在本仓任何一棵树里的，拒", False,
                 three_way_report_verdict("three-way-attack", "报告：$SINGLEFS_WORK_ROOT/elsewhere-copy/research/prompts/x-r1-opus-output.md\n", work) is None),
            ]
        finally:
            if saved_work_root is None:
                os.environ.pop(work_roots.WORK_ROOT_VARIABLE, None)
            else:
                os.environ[work_roots.WORK_ROOT_VARIABLE] = saved_work_root
        cases += work_root_cells
        # ⑳ 真提示（拆分提案第九十节 ⑳）与 landing-audit-r4 到 r6 的六份真腿提示（第九十节 ㉗、㉙）：逐份只看报告类的两种拒（报告文件名、三方腿报告路径），
        # 期望照九十节 ⑳、㉗、㉙
        # 样本在钩子自己的样本目录里，sources.tsv 与期望表逐份同名同 sha256；目录不在、少一份、改了一个字节都判红，不报本次未跑
        real_directory, repository_root = os.path.join(hook_dir, REAL_PROMPTS_DIRECTORY), os.path.dirname(os.path.dirname(hook_dir))
        real_status, real_cases = real_prompt_cells(real_directory, repository_root)
        cases += real_cases
        if real_status != "ran":
            not_run.append(real_status)
        cases.append(("真提示样本:sources.tsv 与期望表逐份同名同 sha256", sorted((name, digest) for name, digest, *_ in REAL_PROMPT_EXPECTATIONS),
                      real_prompt_sources(real_directory)))
        def failing_names(cells):
            return [label.split("（", 1)[0][len("真提示:"):] for label, want, got in cells if want != got]
        absent_status, absent_cases = real_prompt_cells(os.path.join(work, "不在的目录"), repository_root)
        cases.append((f"真提示样本:目录不在时 {len(REAL_PROMPT_EXPECTATIONS)} 份逐份判红、不报本次未跑", ("ran", len(REAL_PROMPT_EXPECTATIONS), len(REAL_PROMPT_EXPECTATIONS)),
                      (absent_status, len(absent_cases), len(failing_names(absent_cases)))))
        # 样本目录拷两份进自证的临时目录：一份删掉最后一份提示，一份在它末尾多写一个空格（样本目录不在时这两份也不在，两格照样红）
        altered_name, altered = REAL_PROMPT_EXPECTATIONS[-1][0], {}
        for variant in ("missing-one", "one-byte"):
            altered[variant] = os.path.join(work, "real-prompts-altered", variant)
            if os.path.isdir(real_directory):
                shutil.copytree(real_directory, altered[variant])
        if os.path.isdir(real_directory):
            os.remove(os.path.join(altered["missing-one"], altered_name))
            with open(os.path.join(altered["one-byte"], altered_name), "ab") as handle:
                handle.write(b" ")
        cases.append(("真提示样本:少一份时只有那一份判红", [altered_name], failing_names(real_prompt_cells(altered["missing-one"], repository_root)[1])))
        cases.append(("真提示样本:一份末尾多一个空格时只有那一份判红", [altered_name], failing_names(real_prompt_cells(altered["one-byte"], repository_root)[1])))
        # 真提示里草稿目录只写在句中、~/ 起头的那一份（a9717cf07fb6e7c21.txt）：草稿目录一行的认法放宽（LIB_PROMPT_SCRATCH_LINE_BREAK=label-anywhere）之后，那一句里的目录落进工作根底下、
        # 报告文件名判拒；家目录与配置的工作根对不上时 ~/ 也要展开到工作根上，这一格把家目录换到别处再判，判得与家目录无关（开关 real-prompts-home-unaligned 下红）
        sentence_only_name = "a9717cf07fb6e7c21.txt"
        if real_prompt_home() is None and not break_switch_is("real-prompts-home-unaligned"):
            not_run.append("⑳ 配置的工作根不以 /.local/share/singlefs/work 收尾，真提示里 ~/ 写的工作根对不上它，换家目录判真提示那一格没跑")
        else:
            moved = {name: os.environ.get(name) for name in ("HOME", "LIB_PROMPT_SCRATCH_LINE_BREAK")}
            os.environ["HOME"] = os.path.join(work, "other-home")
            os.environ["LIB_PROMPT_SCRATCH_LINE_BREAK"] = "label-anywhere"
            try:
                relaxed_cells = real_prompt_cells(real_directory, repository_root)[1]
            finally:
                for name, value in moved.items():
                    if value is None:
                        os.environ.pop(name, None)
                    else:
                        os.environ[name] = value
            relaxed_got = next((got for label, _want, got in relaxed_cells if label.startswith(f"真提示:{sentence_only_name}（")), None)
            cases.append(("真提示:家目录换到别处、草稿目录一行认法放宽之后，只写在句中、~/ 起头的草稿目录照样落在工作根底下，报告文件名判拒", True,
                          isinstance(relaxed_got, tuple) and relaxed_got[1]))
        # 三方腿类型表（拆分提案第九十节 ⑯）：⑰ 读到的类型集合与登记处（定义 frontmatter 的 leg: three-way 一行）逐个相同。样本仓里的登记故意与写死的三种不同：
        # 攻方没登记、辩方的值带行尾注释、另登记一种不以 three-way 起头的 sample-leg、材料员没登记、正文里写的不算；本仓按 frontmatter 现扫，至少要有一种。
        # 再经 decide 判一遍：样本仓里 sample-leg 报告写草稿目录下的 result.md 拒，没登记的攻方不按三方腿判（开关 leg-types-hardcoded 下样本仓那三格红）
        legs_work = tempfile.mkdtemp()
        write_batch_record(legs_work)   # ⑱ 批次一行点名的调度记录
        other_directories.append(legs_work)
        for leg_name, leg_line in (("three-way-forward", "leg: three-way\n"), ("three-way-attack", ""), ("three-way-defense", "leg: three-way  # 注释\n"),
                                   ("three-way-materials", ""), ("sample-leg", "leg: three-way\n")):
            write_sample(legs_work, f".claude/agents/{leg_name}.md", f"---\nname: {leg_name}\n{leg_line}---\n正文里写的\nleg: three-way\n不算\n")
        frontmatter_scanned_legs = runner_dispatch_guard_prompt_checks.three_way_legs.three_way_leg_types_scanned   # 「frontmatter 里写着什么」：同一份库里不拆键值的另一种读法
        repository_root = os.path.dirname(os.path.dirname(hook_dir))
        repository_legs = frontmatter_scanned_legs(repository_root)
        leg_result_prompt = f"草稿目录 {home_new_root}/x-r1/\n报告：{home_new_root}/x-r1/result.md\n"
        cases += [
            ("三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同", frontmatter_scanned_legs(legs_work), sorted(three_way_report_types(legs_work))),
            ("三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种", (repository_legs, True), (sorted(three_way_report_types(repository_root)), bool(repository_legs))),
            case_in(legs_work, "三方腿类型表:样本仓登记的 sample-leg 报告写草稿目录下的 result.md 拒", "sample-leg", leg_result_prompt, 2),
            case_in(legs_work, "三方腿类型表:样本仓没登记的攻方报告写草稿目录下的 result.md 不按三方腿判", "three-way-attack", leg_result_prompt, 0),
        ]
        # 撤掉的 ⑭ ⑮ 与 ⑱ 写死台数那一判（records/2026-10-30-编译绕过内存总上限.md 批 3b）：线程上限、预计挂钟与长活几行写不写、写得对不对，正文写没写死台数，
        # 派发闸都不再因为它们拒；开关 retired-lines-refused 下写了那几行的各格红
        cases += [
            case("撤掉的行:线程上限一行写括注、公式、不等于登记值不拒", "clerk", "线程上限：7 ÷ 2（本机），第二台 16\n线程上限已判：\n", 0),
            case("撤掉的行:多机一行写死台数与额度、多机已判没写理由、分不了不到 8 个字不拒", "clerk",
                 "预计挂钟：超过 2 小时\n多机：本机 31 线程 30G、第二台 31 线程 50G\n多机已判：\n分不了：多机不行\n", 0),
            case("撤掉的行:正文写死本机与第二台各几线程、两台合计内存不拒", "clerk", "第 11–13 步改成两台分片跑，本机 31 线程、第二台 31 线程，内存两台合计 80G。\n", 0),
        ]
        for retired_type in sorted(BATCH_LINE_AGENTS):
            retired_extra = {"mutation-triage": "变异表 crates/mutations.tsv。\n", "experiment-runner": "只补落产物。\n"}.get(retired_type, "")   # 各类型自己要的那一行（⑩ 变异表、⑪ 岔路）
            cases += [
                case(f"撤掉的行:派 {retired_type} 不写线程上限与预计挂钟放行", retired_type, retired_extra + "改完交回。", 0),
                case(f"撤掉的行:派 {retired_type} 写了线程上限与预计挂钟超过 2 小时、别的长活几行都没写放行", retired_type,
                     retired_extra + "线程上限：16\n预计挂钟：超过 2 小时，推的：一千行 × 每行 2 分钟\n", 0),
            ]
        # ── ⑯ 草稿目录写者锁：经钩子入口真派（SINGLEFS_WORK_ROOT 指到自检自己的工作根），锁与会话记录都在临时目录 ──
        # 开关 writer-lock-unchecked、writer-lock-live-ignored、writer-lock-stale-kept、writer-lock-valve-ignored、writer-lock-root-locked 各红在文件头点名的格
        lock_root = tempfile.mkdtemp()
        other_directories.append(lock_root)
        lock_transcript = os.path.join(session_work, "locks.jsonl")
        write_session("locks", [])
        def lock_launch(agent_id, text):
            return launch(agent_id, "claude-opus-5", text, seconds_ago=0)
        def lock_prompt(directory_name, note):
            return batch_lines + f"草稿目录：{lock_root}/{directory_name}/\n报告：{lock_root}/{directory_name}/result.md\n{note}\n"
        def lock_dispatch(directory_name, note, session="locks"):
            text = lock_prompt(directory_name, note)
            lock_input = {"tool_name": "Agent", "session_id": session, "transcript_path": os.path.join(session_work, session + ".jsonl"),
                          "tool_input": {"subagent_type": "clerk", "prompt": text}}
            completed = subprocess.run(["bash", script], input=json.dumps(lock_input), capture_output=True, text=True,
                                       env=dict(os.environ, CLAUDE_PROJECT_DIR=defs_work, SINGLEFS_WORK_ROOT=lock_root))
            return completed, text
        def lock_of(directory_name):
            return runner_dispatch_guard_prompt_checks.writer_lock.read_lock(os.path.join(lock_root, directory_name)) if runner_dispatch_guard_prompt_checks.writer_lock is not None else None
        def lock_is_for(directory_name, text):
            lock = lock_of(directory_name)
            return lock is not None and lock["prompt_sha256"] == hashlib.sha256(text.encode("utf-8")).hexdigest()
        first_run, first_text = lock_dispatch("w1", "第一件")
        first_lock = lock_of("w1")
        cases.append(("写者锁:第一次派发建锁放行", 0, first_run.returncode))
        cases.append(("写者锁:建出的锁带会话号、提示 sha256 与随机串", True,
                      first_lock is not None and first_lock["session_id"] == "locks" and bool(first_lock["nonce"])
                      and first_lock["prompt_sha256"] == hashlib.sha256(first_text.encode("utf-8")).hexdigest()))
        write_session("locks", [lock_launch("a0000000000000aa1", first_text)])
        second_run, _ = lock_dispatch("w1", "第二件")
        cases.append(("写者锁:同一目录再派、持锁的还在跑拒", 2, second_run.returncode))
        cases.append(("写者锁:拒绝说明点名持锁者、目录与已判行的写法", True,
                      all(needle in second_run.stderr for needle in ("a0000000000000aa1", f"{lock_root}/w1", "写者锁已判：", "→ 怎么办"))))
        cases.append(("写者锁:拒绝之后锁没动", True, first_lock is not None and lock_of("w1") is not None and lock_of("w1")["nonce"] == first_lock["nonce"]))
        write_session("locks", [handed_back("a0000000000000aa1", seconds_ago=-2)])
        third_run, third_text = lock_dispatch("w1", "第三件")
        cases.append(("写者锁:持锁的已交回，再派接手", 0, third_run.returncode))
        cases.append(("写者锁:接手之后锁换成新一次派发的", True, lock_is_for("w1", third_text)))
        write_session("locks", [lock_launch("a0000000000000bb2", third_text)])
        valve_run, valve_text = lock_dispatch("w1", "第四件\n写者锁已判：持锁的是客户端重启后留下的旧进程，已确认失效")
        cases.append(("写者锁:写了已判行接手", 0, valve_run.returncode))
        cases.append(("写者锁:已判行接手之后锁换成新一次派发的", True, lock_is_for("w1", valve_text)))
        other_run, _ = lock_dispatch("w2", "别的目录")
        cases.append(("写者锁:别的目录不冲突", 0, other_run.returncode))
        lock_dispatch("w3", "没起来的一")
        unlaunched_run, _ = lock_dispatch("w3", "没起来的二")
        cases.append(("写者锁:没起来的持锁者 10 分钟内算在跑，再派拒", 2, unlaunched_run.returncode))
        aged_lock = lock_of("w3")
        if aged_lock is not None:
            aged_lock["dispatched_at"] -= 700
            with open(runner_dispatch_guard_prompt_checks.writer_lock.lock_path(os.path.join(lock_root, "w3")), "w", encoding="utf-8") as aged_file:
                json.dump(aged_lock, aged_file, ensure_ascii=False)
        aged_run, _ = lock_dispatch("w3", "没起来的三")
        cases.append(("写者锁:没起来的持锁者超过 10 分钟，再派接手", 0, aged_run.returncode))
        lock_dispatch("w6", "派发失败后重试的一件")
        retry_run, _ = lock_dispatch("w6", "派发失败后重试的一件")
        cases.append(("写者锁:同一份提示没起来就重派，当重试接手", 0, retry_run.returncode))
        root_input = {"tool_name": "Agent", "session_id": "locks", "transcript_path": lock_transcript,
                      "tool_input": {"subagent_type": "clerk", "prompt": batch_lines + f"草稿目录：{lock_root}/\n报告：{lock_root}/result.md\n"}}
        subprocess.run(["bash", script], input=json.dumps(root_input), capture_output=True, text=True,
                       env=dict(os.environ, CLAUDE_PROJECT_DIR=defs_work, SINGLEFS_WORK_ROOT=lock_root))
        cases.append(("写者锁:草稿目录写成工作根本身不建锁", False, os.path.exists(os.path.join(lock_root, ".writer-lock"))))
        check_prompt_path = os.path.join(defs_work, "lock-check-prompt.md")
        write_sample(defs_work, "lock-check-prompt.md", batch_lines + f"草稿目录：{lock_root}/w9/\n报告：{lock_root}/w9/result.md\n检查模式\n")
        checked_lock = subprocess.run(["bash", script, "--check-prompt", check_prompt_path, "--type", "clerk", "--root", defs_work],
                                      capture_output=True, text=True, env=dict(os.environ, SINGLEFS_WORK_ROOT=lock_root))
        cases.append(("写者锁:--check-prompt 不建锁、写明派的那一刻另判", (0, False, True),
                      (checked_lock.returncode, os.path.exists(os.path.join(lock_root, "w9", ".writer-lock")), "⑯" in checked_lock.stdout)))
        # ⑯ 锁文件在而判不了（不是 JSON、锁的位置是目录、缺 prompt_sha256）：与写闸第九道同一口径拒，点名那把锁、出路是主 agent 核过之后删掉再派，
        # 已判行不接手；开关 writer-lock-unreadable-absent（当没锁，建锁撞上它报「抢先建了」，缺字段而派发已久的直接接手）下点名的几格红
        broken_lock_path = os.path.join(lock_root, "w10", ".writer-lock")
        os.makedirs(os.path.join(lock_root, "w10"), exist_ok=True)
        with open(broken_lock_path, "w", encoding="utf-8") as broken_lock_file:
            broken_lock_file.write("{不是 JSON")
        os.makedirs(os.path.join(lock_root, "w11", ".writer-lock"))
        os.makedirs(os.path.join(lock_root, "w12"), exist_ok=True)
        with open(os.path.join(lock_root, "w12", ".writer-lock"), "w", encoding="utf-8") as partial_lock_file:
            json.dump({"session_id": "locks", "dispatched_at": time.time() - 7200, "dispatched_date": "2000-01-01", "subagent_type": "clerk",
                       "transcript_path": lock_transcript, "nonce": "0123456789abcdef"}, partial_lock_file)
        broken_run, _ = lock_dispatch("w10", "锁不是 JSON")
        cases.append(("写者锁:锁不是 JSON，派发拒", 2, broken_run.returncode))
        cases.append(("写者锁:判不了的锁的拒绝点名那把锁、判不了与删掉再派的出路", True,
                      all(needle in broken_run.stderr for needle in (broken_lock_path, "判不了谁持锁", f"rm {broken_lock_path}", "→ 怎么办"))))
        cases.append(("写者锁:判不了的锁拒绝之后锁没动", "{不是 JSON", open(broken_lock_path, encoding="utf-8").read()))
        cases.append(("写者锁:锁的位置是目录，派发拒", 2, lock_dispatch("w11", "锁的位置是目录")[0].returncode))
        cases.append(("写者锁:锁缺 prompt_sha256、派发已久，派发拒不接手", 2, lock_dispatch("w12", "锁缺字段")[0].returncode))
        cases.append(("写者锁:判不了的锁写了已判行也拒", 2, lock_dispatch("w10", "写者锁已判：自检里核过这个目录已经没人在写")[0].returncode))
        # ⑯「草稿目录：」一行照 lib_prompt_scratch_line.py 的围栏认法（CommonMark，没收尾的遮到文末）与照逐行配围栏遮不遮不一致：拒、不建锁，拒绝说明叫把围栏写规整
        # （拆分提案第九十节 ㉝ 的跟进；开关 writer-lock-fence-ambiguity-unchecked 下拒的三格与拒绝说明那一格红：③ 按字面找得到「草稿目录」照样放行，⑯ 取不到目录、不建锁）
        def fence_dispatch(text):
            fence_input = {"tool_name": "Agent", "session_id": "locks", "transcript_path": lock_transcript, "tool_input": {"subagent_type": "clerk", "prompt": text}}
            return subprocess.run(["bash", script], input=json.dumps(fence_input), capture_output=True, text=True,
                                  env=dict(os.environ, CLAUDE_PROJECT_DIR=defs_work, SINGLEFS_WORK_ROOT=lock_root))
        unclosed_text = batch_lines + "示例：\n```\n" + f"草稿目录：{lock_root}/w13/\n报告：{lock_root}/w13/result.md\n"
        unclosed_line = unclosed_text.split("\n").index(f"草稿目录：{lock_root}/w13/") + 1
        unclosed_run = fence_dispatch(unclosed_text)
        cases.append(("写者锁:「草稿目录：」一行照 CommonMark 被没收尾的围栏遮住、照逐行配围栏不遮，拒", 2, unclosed_run.returncode))
        cases.append(("写者锁:两种围栏认法不一致的拒绝说明逐行写两种认法、叫把围栏写规整", True,
                      all(needle in unclosed_run.stderr for needle in (f"第 {unclosed_line} 行", "照 CommonMark 认在代码块里，照逐行配围栏认不在代码块里", "把围栏写规整", "lib_prompt_scratch_line.py", "→ 怎么办"))))
        cases.append(("写者锁:两种围栏认法不一致时不建锁", False, os.path.exists(os.path.join(lock_root, "w13", ".writer-lock"))))
        indented_run = fence_dispatch(batch_lines + "    ```\n" + f"草稿目录：{lock_root}/w14/\n    ```\n报告：{lock_root}/w14/result.md\n")
        cases.append(("写者锁:「草稿目录：」一行照逐行配围栏被缩进 4 格的围栏遮住、照 CommonMark 不遮，拒", 2, indented_run.returncode))
        fenced_example_run = fence_dispatch(batch_lines + "示例：\n```\n" + f"草稿目录：{lock_root}/w15-example/\n```\n草稿目录：{lock_root}/w15/\n报告：{lock_root}/w15/result.md\n")
        cases.append(("写者锁:示例行在闭合了的围栏里、真的那一行在围栏外，两种认法一致，放行并只在围栏外那一个目录建锁", (0, True, False),
                      (fenced_example_run.returncode, os.path.exists(os.path.join(lock_root, "w15", ".writer-lock")),
                       os.path.exists(os.path.join(lock_root, "w15-example", ".writer-lock")))))
        write_sample(defs_work, "fence-check-prompt.md", batch_lines + "```\n" + f"草稿目录：{lock_root}/w16/\n报告：{lock_root}/w16/result.md\n")
        fence_checked = subprocess.run(["bash", script, "--check-prompt", os.path.join(defs_work, "fence-check-prompt.md"), "--type", "clerk", "--root", defs_work],
                                       capture_output=True, text=True, env=dict(os.environ, SINGLEFS_WORK_ROOT=lock_root))
        cases.append(("写者锁:--check-prompt 同样拒两种围栏认法不一致的「草稿目录：」一行", 2, fence_checked.returncode))
        # ⑯ ⑤ 派发被同一事件上别的钩子（Agent 事件上除本闸还有 borrowed-convention-guard.sh）拒掉：本闸放行时已经建了写者锁、登了撞文件，那次派发没起来。
        # 主会话记录里那次派发的结果是错误结果的，改过提示的重派照放；回执已落进记录、子 agent 在跑的，与还没出结果的（同一条消息里并行派的上一件）照拒
        # （开关 refused-dispatch-kept、unclaimed-lock-always-retry、refused-dispatch-result-unchecked、refused-dispatch-any-earlier 各红在文件头点名的格）
        def dispatch_call(call_id, text, seconds_ago, subagent_type="clerk"):
            return {"type": "assistant", "timestamp": stamp(seconds_ago),
                    "message": {"role": "assistant", "content": [{"type": "tool_use", "id": call_id, "name": "Agent",
                                                                  "input": {"subagent_type": subagent_type, "prompt": text}}]}}
        def dispatch_refusal(call_id, seconds_ago):
            return {"type": "user", "timestamp": stamp(seconds_ago),
                    "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": call_id, "is_error": True,
                                                              "content": "PreToolUse:Agent hook error: [borrowed-convention-guard.sh]: 自检造的拒绝"}]}}
        def lock_unchanged(directory_name, before):
            return before is not None and lock_of(directory_name) is not None and lock_of(directory_name)["nonce"] == before["nonce"]
        refused_session = "refused-locks"
        refused_note = "被别的钩子拒掉的一件"
        write_session(refused_session, [dispatch_call("toolu_refused_lock", lock_prompt("w20", refused_note), 5)])
        refused_run, _ = lock_dispatch("w20", refused_note, refused_session)
        write_session(refused_session, [dispatch_refusal("toolu_refused_lock", 4)])
        redispatch_run, redispatch_text = lock_dispatch("w20", "改过提示的重派", refused_session)
        cases.append(("写者锁:同一事件上别的钩子拒了这次派发，改过提示的重派照放、锁换成重派的", (0, 0, True), (refused_run.returncode, redispatch_run.returncode, lock_is_for("w20", redispatch_text))))
        running_note = "回执已落进记录的一件"
        write_session(refused_session, [dispatch_call("toolu_running_lock", lock_prompt("w21", running_note), 5)])
        _, running_text = lock_dispatch("w21", running_note, refused_session)
        running_lock = lock_of("w21")
        write_session(refused_session, [launch("a0000000000000cc3", "claude-opus-5", running_text, seconds_ago=0)])
        running_redispatch, _ = lock_dispatch("w21", "回执落进记录之后改过提示的派发", refused_session)
        cases.append(("写者锁:上一次派发的回执已落进记录、子 agent 在跑，改过提示的派发照拒、锁不动", (2, True), (running_redispatch.returncode, lock_unchanged("w21", running_lock))))
        pending_note = "并行派的上一件"
        write_session(refused_session, [dispatch_call("toolu_pending_lock", lock_prompt("w22", pending_note), 5)])
        lock_dispatch("w22", pending_note, refused_session)
        pending_lock = lock_of("w22")
        pending_redispatch, _ = lock_dispatch("w22", "并行派的下一件", refused_session)
        cases.append(("写者锁:上一次派发还没出结果（同一条消息里并行派的），改过提示的派发照拒、锁不动", (2, True), (pending_redispatch.returncode, lock_unchanged("w22", pending_lock))))
        repeated_note = "先被拒、原样重派的一件"
        repeated_text = lock_prompt("w23", repeated_note)
        write_session(refused_session, [dispatch_call("toolu_repeated_old", repeated_text, 120)])
        lock_dispatch("w23", repeated_note, refused_session)
        write_session(refused_session, [dispatch_refusal("toolu_repeated_old", 100)])
        repeated_old_lock = lock_of("w23")
        if repeated_old_lock is not None:   # 原来那次派发建的锁挪到 60 秒前：原样重派的 tool_use（5 秒前）在它之后
            repeated_old_lock["dispatched_at"] -= 60
            with open(runner_dispatch_guard_prompt_checks.writer_lock.lock_path(os.path.join(lock_root, "w23")), "w", encoding="utf-8") as repeated_file:
                json.dump(repeated_old_lock, repeated_file, ensure_ascii=False)
        write_session(refused_session, [dispatch_call("toolu_repeated_new", repeated_text, 5)])
        lock_dispatch("w23", repeated_note, refused_session)
        repeated_lock = lock_of("w23")
        repeated_redispatch, _ = lock_dispatch("w23", "原样重派之后又改过提示的一件", refused_session)
        cases.append(("写者锁:同一份提示先被拒、原样重派后还没出结果，改过提示的派发照拒、锁不动", (2, True), (repeated_redispatch.returncode, lock_unchanged("w23", repeated_lock))))
        refused_files = "要改的文件：scripts/gate/refused-sample-one.py\n"
        write_session("refused-registry", [dispatch_call("toolu_refused_registry", batch_lines + refused_files, 5, "tooling-writer")])
        registered_run = session_case("", "tooling-writer", refused_files, 0, "refused-registry")[2]
        write_session("refused-registry", [dispatch_refusal("toolu_refused_registry", 4)])
        reregistered_run = session_case("", "tooling-writer", "改过提示的重派\n" + refused_files, 0, "refused-registry")[2]
        cases.append(("工具员:同一事件上别的钩子拒了这次派发，改过提示、要改的文件一样的重派照放", (0, 0), (registered_run, reregistered_run)))
        pending_files = "要改的文件：scripts/gate/refused-sample-two.py\n"
        write_session("refused-registry", [dispatch_call("toolu_pending_registry", batch_lines + pending_files, 5, "tooling-writer")])
        pending_registered = session_case("", "tooling-writer", pending_files, 0, "refused-registry")[2]
        pending_collided = session_case("", "tooling-writer", "并行派的下一件\n" + pending_files, 2, "refused-registry")[2]
        cases.append(("工具员:上一次派发还没出结果（同一条消息里并行派的），改过提示、要改的文件一样的派发照拒", (0, 2), (pending_registered, pending_collided)))
        # ⑯ 承接别的写者的副本：实现员、工具员的提示带「承接：」，承接行与标签带「副本」的行点名的路径在别的草稿目录里，那里的锁在已不在跑的写者手里、提示没有「写者锁已判」行就拒，
        # 拒绝说明给整行写法，原样加进提示就放行、放行后接手那个目录的锁；锁在不在跑的写者手里、锁判不了、没有承接行、别的类型、草稿目录就是承接来的目录都不拒
        # （开关 handover-lock-unchecked、handover-lock-valve-unapplied、handover-lock-unrecorded 与 writer-lock-unchecked、writer-lock-valve-ignored 各红在文件头点名的格）
        handover_session = "handover-locks"
        handover_transcript = os.path.join(session_work, handover_session + ".jsonl")
        handover_counter = [0]
        write_session(handover_session, [])
        def handover_old_directory(name, holder, running):
            old_scratch = os.path.join(lock_root, name)
            old_lock = runner_dispatch_guard_prompt_checks.writer_lock.new_lock(handover_session, "前一个写者的提示", "implementation-writer", handover_transcript, now=time.time() - 400)
            runner_dispatch_guard_prompt_checks.writer_lock.create_lock(old_scratch, old_lock)
            runner_dispatch_guard_prompt_checks.writer_lock.try_claim(old_scratch, old_lock, holder)
            write_session(handover_session, [launch(holder, "claude-opus-5", "前一个写者的提示", seconds_ago=300)] + ([] if running else [handed_back(holder, seconds_ago=100)]))
            return old_lock
        def handover_dispatch(subagent_type, scratch_name, report_name, extra=""):
            handover_counter[0] += 1
            files = {"implementation-writer": f"要动的 crates 文件：crates/a/src/handover{handover_counter[0]}.rs\n",
                     "tooling-writer": f"要改的文件：scripts/gate/handover-sample-{handover_counter[0]}.py\n"}.get(subagent_type, "")
            text = (batch_lines + f"草稿目录：{lock_root}/{scratch_name}/\n报告：{lock_root}/{scratch_name}/result.md\n" + files
                    + (f"承接：{lock_root}/{report_name}/result.md\n" if report_name else "") + extra)
            handover_input = {"tool_name": "Agent", "session_id": handover_session, "transcript_path": handover_transcript,
                              "tool_input": {"subagent_type": subagent_type, "prompt": text}}
            completed = subprocess.run(["bash", script], input=json.dumps(handover_input), capture_output=True, text=True,
                                       env=dict(os.environ, CLAUDE_PROJECT_DIR=defs_work, SINGLEFS_WORK_ROOT=lock_root))
            return completed, text
        stale_holder = "a0000000000000ee5"
        stale_lock = handover_old_directory("hv1", stale_holder, running=False)
        handover_red, _ = handover_dispatch("tooling-writer", "hv1-new", "hv1")
        suggested_line = next((line.strip() for line in handover_red.stderr.splitlines() if line.strip().startswith("写者锁已判：")), "")
        cases.append(("写者锁承接:承接来的副本在别的草稿目录里，锁在已不在跑的写者手里、提示没写已判行，工具员派发拒", 2, handover_red.returncode))
        cases.append(("写者锁承接:拒绝说明点名目录、持锁者与整行「写者锁已判：」写法", True,
                      all(needle in handover_red.stderr for needle in (f"{lock_root}/hv1", stale_holder, "写者锁已判：", "→ 怎么办"))))
        cases.append(("写者锁承接:拒绝说明里取得到整行写法", True, bool(suggested_line)))
        cases.append(("写者锁承接:拒绝之后旧锁没动、新草稿目录没建锁", (True, False), (lock_unchanged("hv1", stale_lock), lock_of("hv1-new") is not None)))
        handover_green, handover_green_text = handover_dispatch("tooling-writer", "hv1-new", "hv1", suggested_line + "\n")
        cases.append(("写者锁承接:拒绝说明里的整行原样加进提示就放行", 0, handover_green.returncode))
        cases.append(("写者锁承接:已判行放行后旧目录的锁换成这一次派发的、新草稿目录也建了锁", (True, True),
                      (lock_is_for("hv1", handover_green_text), lock_is_for("hv1-new", handover_green_text))))
        implementer_lock = handover_old_directory("hv2", "a0000000000000ee6", running=False)
        implementer_red = handover_dispatch("implementation-writer", "hv2-new", "hv2")[0]
        cases.append(("写者锁承接:实现员同样拒、拒绝说明给已判行、锁不动", (2, True, True), (implementer_red.returncode, "写者锁已判：" in implementer_red.stderr, lock_unchanged("hv2", implementer_lock))))
        copy_lock = handover_old_directory("hv3", "a0000000000000ee7", running=False)
        cases.append(("写者锁承接:标签带「副本」的行点名的路径所在目录有已不在跑的写者的锁，拒", (2, True),
                      (handover_dispatch("tooling-writer", "hv3-new", "hv3-report", f"副本：{lock_root}/hv3/copy\n")[0].returncode, lock_unchanged("hv3", copy_lock))))
        running_lock = handover_old_directory("hv4", "a0000000000000ee8", running=True)
        cases.append(("写者锁承接:持锁的写者还在跑不拒、锁不动", (0, True), (handover_dispatch("tooling-writer", "hv4-new", "hv4")[0].returncode, lock_unchanged("hv4", running_lock))))
        bare_lock = handover_old_directory("hv5", "a0000000000000ee9", running=False)
        cases.append(("写者锁承接:没有承接行不判（只有副本行）", (0, True), (handover_dispatch("tooling-writer", "hv5-new", "", f"副本：{lock_root}/hv5/copy\n")[0].returncode, lock_unchanged("hv5", bare_lock))))
        clerk_lock = handover_old_directory("hv6", "a0000000000000eea", running=False)
        cases.append(("写者锁承接:不是实现员、工具员的类型不判", (0, True), (handover_dispatch("clerk", "hv6-new", "hv6")[0].returncode, lock_unchanged("hv6", clerk_lock))))
        os.makedirs(os.path.join(lock_root, "hv7"), exist_ok=True)
        with open(os.path.join(lock_root, "hv7", ".writer-lock"), "w", encoding="utf-8") as unreadable_handover_lock:
            unreadable_handover_lock.write("{不是 JSON")
        unreadable_handover = handover_dispatch("tooling-writer", "hv7-new", "hv7")[0]
        cases.append(("写者锁承接:承接来的副本所在目录的锁判不了，照常放行", 0, unreadable_handover.returncode))
        cases.append(("写者锁承接:锁判不了的 stderr 点名那把锁并说这一处没判", True, f"{lock_root}/hv7/.writer-lock" in unreadable_handover.stderr and "承接这一处没判" in unreadable_handover.stderr))
        handover_old_directory("hv8", "a0000000000000eeb", running=False)
        same_run, same_text = handover_dispatch("tooling-writer", "hv8", "hv8")
        cases.append(("写者锁承接:草稿目录就是承接来的目录，照旧自动接手、不要已判行", (0, True), (same_run.returncode, lock_is_for("hv8", same_text))))
        # ⑯ 「写者锁已判：<理由>」那一行点名的目录（不靠承接行、副本行，也不是「草稿目录：」那一行点名的）：持锁者已交回或已停的，派发时把锁换给这一次派发，新派的子 agent 写那个目录认作持锁者、
        # 旧写者不再是；持锁者还在跑的拒、点名它、锁不动（开关 valve-named-directories-ignored、valve-named-live-taken 与 writer-lock-unchecked、writer-lock-valve-ignored、valve-reason-ignored 各红在文件头点名的格）
        named_holder = "a0000000000000ef0"
        handover_old_directory("hv9", named_holder, running=False)
        named_line = f"写者锁已判：持锁的 {named_holder} 已交回，本次接手它留在 {lock_root}/hv9/repo 的副本\n"
        named_run, named_text = handover_dispatch("tooling-writer", "hv9-new", "", named_line)
        named_agent = "a0000000000000ef1"
        write_session(handover_session, [launch(named_agent, "claude-opus-5", named_text, seconds_ago=0)])
        named_lock = lock_of("hv9")
        named_roles = tuple(runner_dispatch_guard_prompt_checks.writer_lock.writer_verdict(os.path.join(lock_root, "hv9"), named_lock, agent, handover_transcript)[0] if named_lock else None
                            for agent in (named_agent, named_holder))
        cases.append(("写者锁已判点名:承接别的目录里已交回写手的副本，已判行点名那个目录，派发放行、锁换成这一次派发的、新派的子 agent 认作持锁者、旧写手不是", (0, True, ("holder", "other")),
                      (named_run.returncode, lock_is_for("hv9", named_text), named_roles)))
        live_holder = "a0000000000000ef2"
        live_old_lock = handover_old_directory("hv10", live_holder, running=True)
        live_line = f"写者锁已判：持锁的 {live_holder} 已交回，本次接手它留在 {lock_root}/hv10/repo 的副本\n"
        live_run, _ = handover_dispatch("tooling-writer", "hv10-new", "", live_line)
        cases.append(("写者锁已判点名:点名的目录里持锁者还在跑，拒、点名持锁者与目录、锁不动、新草稿目录没建锁", (2, True, True, False),
                      (live_run.returncode, live_holder in live_run.stderr and f"{lock_root}/hv10" in live_run.stderr,
                       lock_unchanged("hv10", live_old_lock), lock_of("hv10-new") is not None)))
        # 已判行只写理由、不写目录：承接来的副本所在目录的锁照旧由承接那一支接手（已判行写了目录的，上面「已判点名」那一支也接手同一个目录，盖住承接这一支；这一格的已判行不点名目录，只有承接那一支接手，
        # 开关 handover-lock-valve-unapplied 下红）
        bare_holder = "a0000000000000ef3"
        handover_old_directory("hv11", bare_holder, running=False)
        bare_run, bare_text = handover_dispatch("tooling-writer", "hv11-new", "hv11", f"写者锁已判：持锁的 {bare_holder} 已交回，本次接手承接来的副本\n")
        cases.append(("写者锁承接:已判行只写理由、不写目录，承接来的副本所在目录的锁照旧换成这一次派发的、新草稿目录也建了锁", (0, True, True),
                      (bare_run.returncode, lock_is_for("hv11", bare_text), lock_is_for("hv11-new", bare_text))))
        # ⑤ ⑯ 放行之前先问同一事件上别的钩子（sibling_refusals）：会拒（退 2）的这一次不建写者锁、不登撞文件，主会话记录里还读不到那次被拒（下面的会话记录是空的）时，改过提示的重派也照放；
        # 别的钩子不拒的派发照常登记，要改的文件一样的再派照拒；只挂在 Bash 上的钩子不问。开关 sibling-hooks-unconsulted 下前两格红
        sibling_work = tempfile.mkdtemp()
        other_directories.append(sibling_work)
        shutil.copytree(defs_work, sibling_work, dirs_exist_ok=True)
        write_sample(sibling_work, "sibling-refuses-when-marked.sh",
                     '#!/usr/bin/env bash\n[[ -e "$(dirname "${BASH_SOURCE[0]}")/refuse-now" ]] && { echo "✗ 自检造的同一事件上的拒绝" >&2; exit 2; }\nexit 0\n')  # 样本造的拒绝（→ 出路不归它）
        write_sample(sibling_work, "sibling-bash-only-refuses.sh", '#!/usr/bin/env bash\necho "✗ 自检造的、只挂在 Bash 上的钩子" >&2\nexit 2\n')  # 样本造的拒绝（→ 出路不归它）
        write_sample(sibling_work, ".claude/settings.json", json.dumps({"hooks": {"PreToolUse": [
            {"matcher": "Agent|Task", "hooks": [{"type": "command", "command": 'bash "$CLAUDE_PROJECT_DIR"/sibling-refuses-when-marked.sh'}]},
            {"matcher": "Bash", "hooks": [{"type": "command", "command": 'bash "$CLAUDE_PROJECT_DIR"/sibling-bash-only-refuses.sh'}]}]}}))
        refuse_marker = os.path.join(sibling_work, "refuse-now")
        write_session("siblings", [])
        def sibling_dispatch(directory_name, note):
            text = batch_lines + f"草稿目录：{lock_root}/{directory_name}/\n报告：{lock_root}/{directory_name}/result.md\n要改的文件：scripts/gate/sibling-sample.py\n{note}\n"
            sibling_input = {"tool_name": "Agent", "session_id": "siblings", "transcript_path": os.path.join(session_work, "siblings.jsonl"),
                             "tool_input": {"subagent_type": "tooling-writer", "prompt": text}}
            return subprocess.run(["bash", script], input=json.dumps(sibling_input), capture_output=True, text=True,
                                  env=dict(os.environ, CLAUDE_PROJECT_DIR=sibling_work, SINGLEFS_WORK_ROOT=lock_root))
        open(refuse_marker, "w").close()
        refused_elsewhere = sibling_dispatch("sb1", "同一事件上别的钩子会拒的一件")
        cases.append(("同一事件上别的钩子:会拒这次派发，本闸放行、不建写者锁", (0, False), (refused_elsewhere.returncode, lock_of("sb1") is not None)))
        os.remove(refuse_marker)
        cases.append(("同一事件上别的钩子:会拒的那一次没登撞文件，要改的文件一样的重派照放", 0, sibling_dispatch("sb2", "别的钩子这回不拒、改过提示的重派").returncode))
        cases.append(("同一事件上别的钩子:不拒的派发照常登记，要改的文件一样的再派照拒（只挂在 Bash 上的钩子不问）", 2, sibling_dispatch("sb3", "重派登记下来之后再来一件").returncode))
    finally:
        os.environ.pop("DISPATCH_GUARD_STATE_ROOT", None)
        if saved_detections is None:
            os.environ.pop("AGENT_HOOK_DETECTIONS", None)
        else:
            os.environ["AGENT_HOOK_DETECTIONS"] = saved_detections
        os.environ.pop("RUNNER_DISPATCH_GUARD_ADMISSION_SCRIPT", None)
        shutil.rmtree(work)
        for directory in other_directories:
            shutil.rmtree(directory, ignore_errors=True)
    failures = [item for item in cases if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当返回 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 decide() 与它调的各条判法（project_code_run_sentences、agent_definition_fields、out_of_scope_mentions、implementation_writer_verdict、concurrency_verdict、"
              "kb_scribe_spec_verdict、designer_admission_verdict、snapshot_conflict_verdict）；RUNNER_DISPATCH_GUARD_BREAK 设着的话这里本来就该红")
        return 1
    real_count = sum(1 for case_item in cases if str(case_item[0]).startswith("真提示:"))
    real_summary = (f"；⑳ 钩子样本目录 {REAL_PROMPTS_DIRECTORY} 里的真提示 {real_count} 份（整份 sha256 钉在 {REAL_PROMPT_SOURCES} 与期望表）逐份对上九十节 ⑳ 与两轮判决的期望，"
                    "目录不在、少一份、改了一个字节都判红" if not not_run else
                    "；本次未跑 " + str(len(not_run)) + " 格：" + "；".join(not_run))
    for line in not_run:
        print(f"  ! {line}")
        if os.environ.get("GATE_NOT_RUN_FILE"):
            with open(os.environ["GATE_NOT_RUN_FILE"], "a", encoding="utf-8") as not_run_file:
                not_run_file.write(f"runner-dispatch-guard.sh --selftest：{line}\n")
    print(f"  ✓ 自检通过（查了 {len(cases)} 种）：正文许可或叫子 agent 跑项目代码（cargo、本仓的脚本、门禁阶段、自证、样本档）的句子拒、否定句、归主 agent 的、交回的写法与转述放行；"
          "内置通用类型不带「通用已判」放行行的拒，Explore 与 clerk 放行；骨架占位没填完的拒；不读共用约束的类型没写开工先读的拒；定义 required-inputs 缺一组的拒、同义词认得；提示点名写范围外的路径拒、否定句里的不判，"
          "工作根（~ 写法与绝对路径）与会话临时目录根下的报告按写范围表里展开的 $SINGLEFS_WORK_ROOT 判（会话临时目录根下的判越界），规格文件在工作根（含 ~ 写法）与会话临时目录根下都读；"
          "写回员给判决不给规格的放行；"
          "实现员没写要动的 crates 文件、超上限、与在跑的实现员撞文件的拒，交回之后不再撞；opus 在跑满上限的拒、sonnet 与用户放行行放行，"
          "书记员规格检查红或一份规格都认不出的拒；设计员重跑登记准入判单元键没变、登记有错、前提没齐的拒，准入退别的码放行并记一处没判全；"
          "变异分诊员没给表的拒；执行员没点名岔路、续做没写还差或还差写无的拒，只修锚点与只补落产物放行；"
          "派书记员或实现员时要改的文件在没判完的三方快照里就拒，已判完的轮、没正文的快照、写了放行行的、派别的 agent 的、认不出的清单行、不在的规格与读不了的清单放行；"
          "派十种类型写不写线程上限、预计挂钟与长活几行都放行（线程上限写括注、公式、不等于登记值，预计挂钟超过 2 小时而分片、续跑、多机、复用都没写，多机一行写死台数，正文写死本机与第二台各几线程）；"
          "各条都判完、不成立的一次列全，缺输入的写明按字面子串认哪几个字并给照抄的一行；--check-prompt 拿同一份判法核提示、不碰实现员登记；"
          "快照那一条：用户叫停的轮在 abandoned-rounds.tsv 登记一行、出处是 records/ 下现存文件的行号加〔〕原话（原话恰好找到一次，行号对不上照收；"
          "命中两次而行号提示落在其中一处、或找不到而行号提示那一行有这一轮的名字、去掉改名表的新旧名与日期之后〔〕落在那一行里，取提示那一行；取到的那一行有这一轮的名字和「」括的原话）的放行，"
          "没登记、没出处、没行号、没带〔〕原话、原话找不到而提示那一行没有名字、原话找不到而提示那一行去掉新旧名与日期之后对不上（〔〕写的是转述）、命中两次而提示不在其中、"
          "那一行没有名字或原话太短或名字连着别的轮名或另写着仓里认得的别的轮名、没写出处（不按日期免）、指到自己的材料或规则文件或带 .. 或符号链接指到仓外、登记行写坏、只写 halted 文件的照拒并写没收的原因；"
          "批次一行认代码块照 CommonMark（闭合了的围栏块里的行不判：波浪线、列表、引用块里的围栏都认，行首的 ```…```、缩进 4 格的、波浪线块与 HTML 注释里露出的三个反引号不开块，列表项里没闭合的围栏在列表项结束处结束；标签行照 CommonMark 与照宽的逐行认法遮不遮不一致拒，载不进 markdown-it-py 时退回宽的逐行认法），写了几行的每一行都判；"
          "草稿目录写者锁：第一次派发建锁，持锁的还在跑再派拒，交回了、没起来超过 10 分钟或写了已判行接手，别的目录与工作根本身不冲突，--check-prompt 不建锁，"
          "锁文件在而判不了（不是 JSON、锁的位置是目录、缺 prompt_sha256 而派发已久）的拒、点名那把锁与删掉再派的出路、锁不动、写了已判行也不接手；"
          "快照那一条自己出错照常放行并记一条检出；"
          "写范围只判要这个子 agent 写的「写进 / 写到 / 更新」：动词所在子句里点名了主 agent、别的类型的称呼的、在「做成的事：」之下的列表项是转述别人的事，放行，对子 agent 说了话的、写给它自己类型的称呼的、别的执行者在前一个子句里的照判；"
          "报告行：「报告」「报告路径」「报告写进」「报告写到」四种标签共用一份解析，写范围只认行首（列表记号、粗体、编号之后）或句首的、点名的路径交写范围判；"
          "文件名与三方形态两判在哪都认、只判落在本提示草稿目录下的路径与裸文件名（本提示的草稿目录只认行首「草稿目录：」一行，转述里的、列表里的、「草稿目录在」都不认，⑯ ⑰ 调的是 lib_prompt_scratch_line.py 那一份），"
          "正文里的文件名带 report（不分大小写）的、以 SUMMARY、FINDINGS、ANALYSIS 开头（不分大小写）的 .md 拒，result.md、x-summary.md、summary.txt 放行，"
          "目录名带 report 不碍事，别的目录下的（转述别的子 agent 的报告）、没写草稿目录时带目录的、代码块里的示例行、「交回报告：」这类前面连着字的、「」『』“”（照整份配对、可以跨行）与反引号里引着的不判，"
          "紧跟否定词的文件名不判，有占位的那一行只交骨架占位；三方腿（类型读定义 frontmatter 的 leg: three-way，样本仓与本仓读到的都与 frontmatter 逐个相同）要有一处本腿的报告 research/prompts/<轮>-<模型>-output.md"
          "（报告行点名的，或报告行之外不在引号里、不在禁读清单里、名字以本腿轮名起头的），"
          "报告行写草稿目录下的 result.md、或本腿的报告一处都没有（只在引号、反引号、禁读清单里的、别的轮的不算）、只有没填的 <轮> 占位或子目录下的拒，禁读清单点到本腿自己的报告拒，"
          "本腿报告路径的名字带 report 拒；报告行、整份提示与交回闸 ② 三处形态都经 lib_three_way_legs.py 的 leg_report_mentions_in_trees、对同一组写法判得一样；"
          "三方腿没有行首「报告：」一行（只在句中写报告写进、只在围栏里写）的拒；"
          "常设要求的几行：十种类型没写批次一行、批次不是 records/<文件>.md#<批号>、点名的调度记录不在、路径带 .. 的拒，批号写批名或后面接别的话的放行；"
          "草稿目录落在会话临时目录根、工作根之外、写相对路径或工作根本身的拒，~ 与 $SINGLEFS_WORK_ROOT/ 起头落在工作根下的放行；"
          "自证在哪跑指到工作根或会话临时目录根下的路径拒，本仓主工作区与不跑放行；"
          "工具员「要改的文件」与实现员登记在同一份表里互比，两个工具员撞、工具员与实现员互撞都拒，目录罩住的也算，不撞放行；"
          "执行员按这一份登记有没有过一段判续做：新的 r<n> 第一段只要岔路，跑过一段的同一份登记缺还差、缺承接或承接指到不在的报告拒；"
          "报告行点名的绝对路径落在工作根与仓外的拒；实现员、工具员没有不碰一行的拒；有定义的类型没有「## 关的是哪几条」一节或节下空的拒；"
          "实现员提示写到「零轮」而没有带任务号的行首「A/B 读数：」一行的拒（句中写、只在围栏里写同样拒），带了的、不含零轮的、别的类型放行；"
          "读数一行判决里照拒的写法（另带 task=、同字并列的号、task log 加号、引用块里的）按判据库当场拒，两臂两个号与单臂放行、两边逐条结局相同；"
          "叫子 agent 起 .claude/gate.d/ 下一道（含 --check 单格、命名四格、--list）的拒，否定句、要主 agent 跑的、转述别人在跑的放行；"
          "许可或要求子 agent 自己往统一任务服务交任务（submit、wait、run_in_background、经统一任务服务交）的句子拒，许可子 agent 跑项目代码（cargo、本仓的脚本、门禁阶段、自证）的也拒，否定句、交主 agent 的、只读 status / log、引号里的放行；"
          "草稿目录写者锁与工具员撞文件登记认被同一事件上别的钩子拒掉的派发：那次派发的结果是错误结果的，改过提示的重派照放；回执已落进记录、子 agent 在跑的，与还没出结果的（并行派的上一件）照拒；"
          "坏 JSON、读不到 lib_writer_lock.py 照常放行并各记一条检出"
          + real_summary)
    return 0

CHECK_PROMPT_USAGE = ("  ✗ 用法：runner-dispatch-guard.sh --check-prompt <提示文件> --type <类型> [--model <模型>] [--root <仓根>]\n"
                      "  → 怎么办：平时由 python3 scripts/agents/dispatch-prompt.py --check <提示文件> --type <类型> 替你这样调")

def check_prompt_main(arguments, hook_dir):
    """--check-prompt：派之前拿派发闸同一份判法（judge_dispatch，check_only）核一份提示，不成立的一次列全、退 2，都成立退 0；用法错退 2。"""
    rest, options = list(arguments), {}
    path = rest.pop(0) if rest and not rest[0].startswith("--") else None
    while rest:
        flag = rest.pop(0)
        if flag not in ("--type", "--model", "--root") or not rest:
            path = None
            break
        options[flag] = rest.pop(0)
    if path is None or "--type" not in options:
        print(CHECK_PROMPT_USAGE)
        return 2
    try:
        prompt = open(path, encoding="utf-8").read()
    except OSError as error:
        print(f"  ✗ 读不了提示文件 {path}：{error}\n  → 怎么办：给一个存在的提示文件路径")
        return 2
    project_root = options.get("--root") or os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    tool_input = {"subagent_type": options["--type"], "prompt": prompt}
    if "--model" in options:
        tool_input["model"] = options["--model"]
    refusals, notes, unjudged = judge_dispatch({"tool_name": "Agent", "tool_input": tool_input}, project_root, hook_dir, check_only=True)
    later = f"；派的那一刻另判：{'、'.join(unjudged)}" if unjudged else ""
    if refusals:
        print(combined_refusal(options["--type"], refusals))
    if notes:
        print(notes)
    if refusals:
        print(f"→ 派发闸照这份提示会拒 {len(refusals)} 处（{'、'.join(name for name, _ in refusals)}），照上面逐处改完再核{later}")
        return 2
    print(f"  ✓ 派发闸照这份提示放行（{options['--type']}，查了 {len(prompt.splitlines())} 行提示）{later}")
    return 0

# 弄坏开关只在自证里认：命令行不是单独一个 --selftest、也不是自证起的子进程（RUNNER_DISPATCH_GUARD_SELFTEST_MARK 对得上）而 BREAK_SWITCH_VARIABLES 里有非空的
# （派发闸自己的 RUNNER_DISPATCH_GUARD_BREAK，与它调的共用库 lib_prompt_scratch_line.py 的 LIB_PROMPT_SCRATCH_LINE_BREAK：库在钩子进程里读它，平常跑设着就改了「草稿目录」一行的认法），
# 读 hook 的 JSON 之前就退 2、点名变量只在自证里认，不管值是什么。标记（SELFTEST_MARK_VARIABLE、SELFTEST_MARK_CONTENT）定义在入口的引导里：引导在这几份载入之前就要认它，
# 它的内容同一句也写在 scripts/agents/dispatch-prompt.py 的 selftest() 里：它的自证起 --check-prompt 的子进程，要带上派发闸的弄坏开关。
BREAK_SWITCH_VARIABLES = ("RUNNER_DISPATCH_GUARD_BREAK", "LIB_PROMPT_SCRATCH_LINE_BREAK")

def refuse_break_outside_selftest(arguments):
    setting = [name for name in BREAK_SWITCH_VARIABLES if os.environ.get(name)]
    if not setting or arguments == ["--selftest"] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    named = "、".join(f"{name}={os.environ[name]}" for name in setting)
    print(f"✗ runner-dispatch-guard.sh：{named} 是弄坏开关，只在自证（runner-dispatch-guard.sh --selftest）里认，平常跑设了就拒，这一次派发没判", file=sys.stderr)
    print(f"→ 怎么办：去掉 {' '.join(setting)} 再跑；要证红就跑 {setting[0]}=<项> bash .claude/hooks/runner-dispatch-guard.sh --selftest", file=sys.stderr)
    return 2

def main():
    hook_dir = sys.argv[1]
    refused = refuse_break_outside_selftest(sys.argv[2:])
    if refused is not None:
        return refused
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    if len(sys.argv) > 2 and sys.argv[2] == "--check-prompt":
        return check_prompt_main(sys.argv[3:], hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except ValueError as error:
        if not break_switch_is("json-error-silent"):
            print(f"! 派发闸读不懂 hook 的 JSON，这一次放行：{error}", file=sys.stderr)
        if break_switch_is("json-error-denies"):
            return 2
        if not break_switch_is("json-error-unrecorded"):   # 照「放行但必记检出」：拿不到会话号与派的类型，记下读不懂的原因
            print(record_unjudged({}, [f"hook 的 JSON 读不懂（{error}），这一次一条都没判"]), file=sys.stderr)
        return 0
    if not isinstance(hook_input, dict):   # 合法 JSON 但不是对象：同样一条都判不了
        print(f"! 派发闸收到的 hook 输入不是 JSON 对象（{type(hook_input).__name__}），这一次放行", file=sys.stderr)
        if not break_switch_is("json-error-unrecorded"):
            print(record_unjudged({}, [f"hook 的输入不是 JSON 对象（{type(hook_input).__name__}），这一次一条都没判"]), file=sys.stderr)
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    if work_roots is None:
        print(f"! 派发闸读不到 {os.path.join(hook_dir, 'lib_work_root.py')}（{work_roots_error!r}）：提示里工作根与会话临时目录根下的路径这一次没判", file=sys.stderr)
    return judge_hook_input(hook_input, project_root)

def judge_hook_input(hook_input, project_root):
    """读得懂的 hook 输入 → 退出码。判的时候抛异常照「放行但必记检出」照常放行、记一条检出（弄坏开关 exception-unrecorded 下照原样抛出：进程退 1、不记）。"""
    try:
        code, message = runner_dispatch_guard_locks_and_verdicts.decide(hook_input, project_root, take_locks=True, record_detections=True)
    except Exception as error:
        if break_switch_is("exception-unrecorded"):
            raise
        print(f"! 派发闸判的时候抛异常，这一次放行：{type(error).__name__}: {error}", file=sys.stderr)
        code, message = 0, record_unjudged(hook_input, [f"判的时候抛异常（{type(error).__name__}: {error}），这一次没判全"])
    if message:
        print(message, file=sys.stderr)
    return code
