# 派发闸 runner-dispatch-guard.sh 的 python 部分之一：草稿目录写者锁、批次与自证在哪跑与报告落点与不碰与关的是哪几条与零轮读数、门禁阶段、承接、decide 与检出记录、提示里叫跑项目代码的句子、同挂的兄弟钩子、judge_dispatch。
# 入口 runner-dispatch-guard.sh 的引导先静态核每一份（读得到、编得过、顶层绑着入口要的名字），再按 RUNNER_DISPATCH_PART_NAMES 的次序 import 它们；每一份是普通 python 模块，用到别份的名字写显式 import（不留环），不单独执行；
# 弄坏开关的名单、读开关的代码在这几份里，开关表与各开关红在哪几格写在入口的文件头。
import glob
import importlib.util
import json
import os
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from __main__ import BATCH_LINE_AGENTS, LINE_START, ZERO_ROUND_READING_LINE, ZERO_ROUND_TASK_NUMBER
from runner_dispatch_guard_prompt_checks import (
    ADDRESSED_TO_AGENT, BATCH_LABEL_LINE, CLAUSE_SPLIT, COMMON_CONSTRAINTS_READ, FINISHED_TASK_ID, FINISHED_TASK_OUTCOME, GENERAL_PURPOSE_TYPES,
    HEADING_LINE, MAIN_AGENT_MENTION, MUTATION_TABLE_MENTION, NARRATION_LABEL, NEGATION_BEFORE, NONE_WORDS, RETIRED_HOST_PAIR, RETIRED_LABEL_LINE,
    RUNNING_RECENT_SECONDS, RUN_OBJECT, RUN_OBJECT_DISTANCE, RUN_OBJECT_HEAVY, RUN_VERB, SCRATCH_FILE_MENTION, SCRATCH_MENTION_PREFIX,
    SCRIPT_HOOK_DIR, SENTENCE_QUOTED, SENTENCE_SPLIT, SKELETON_PLACEHOLDER, TASK_SUBMISSION_HAND_BACK, TASK_SUBMISSION_HAND_BACK_SECTION_QUOTED,
    TASK_SUBMISSION_MENTION, TASK_SUBMISSION_NEGATION_AFTER, TASK_SUBMISSION_NEGATION_BEFORE, TEXT_REWRITE, TEXT_REWRITE_BREAK,
    TEXT_REWRITE_TAIL_LIMIT, UNJUDGED_NOTES, UNLAUNCHED_GRACE_SECONDS, WRITER_GUARDED_AGENTS, agent_definition_fields, break_switch_is,
    code_block_ambiguity_refusal, code_block_ambiguous_label_lines, concurrency_verdict, designer_admission_verdict, dispatch_refusal_lookup,
    expanded_mention, hidden_lines_by_wide_line_rule, is_running, kb_scribe_spec_verdict, labelled_value, labelled_values_with_list,
    missing_required_inputs, note_unjudged, out_of_scope_mentions, prompt_scratch_directories, prompt_scratch_line_error, quoted_spans,
    register_writer, report_file_name_verdict, report_line_mentions, session_dispatch_events, snapshot_conflict_verdict, snapshot_library,
    three_way_report_types, three_way_report_verdict, valve_reason, without_claim_purpose, without_code_blocks, work_roots, work_roots_error,
    writer_collision_verdict, writer_lock_error,
)
import runner_dispatch_guard_prompt_checks

WRITER_LOCK_HOLDER_UNLAUNCHED = "一个刚派出去、派发回执还没落进主会话记录的子 agent"

SCRATCH_LINE_FENCE_RULE = ("→ 规矩：⑯ 只在行首「草稿目录：」一行点名的目录里建写者锁，那一行怎么认只在 .claude/hooks/lib_prompt_scratch_line.py（围栏照 CommonMark：没收尾的一直遮到文末）；"
                           "这一行照那份库认在代码块里、照逐行配围栏认不在（或反过来）时，③ 按字面子串找得到「草稿目录」照样放行，⑯ 却取不到这个目录、不建锁"
                           "（.claude/hooks/runner-dispatch-guard.sh 文件头 ⑯ 条）。\n")
SCRATCH_LINE_FENCE_HOWTO = ("→ 怎么办：把围栏写规整：每个围栏都补上收尾行（与开头同一种字符 ``` 或 ~~~、不比开头短、前面至多缩进三个空格），开头也至多缩进三个空格；"
                            "或者把「草稿目录：」一行挪到所有围栏之前。")

def scratch_line_fence_ambiguity(prompt):
    """⑯：「草稿目录：」一行（同目录 lib_prompt_scratch_line.py 的 scratch_line_values 单看这一行认得出路径的）在那份库的围栏认法（fenced_line_numbers，
    照 CommonMark，没收尾的遮到文末）与宽的逐行认法（hidden_lines_by_wide_line_rule，找不到关块的那一行不开块）下遮不遮不一致的：
    [(行号从 1 起, 那一行, 库遮不遮, 逐行认法遮不遮)]。读不到那份库、它抛异常（弄坏开关写错这类）交空表；弄坏开关 writer-lock-fence-ambiguity-unchecked 下交空表。"""
    if runner_dispatch_guard_prompt_checks.prompt_scratch_line is None or break_switch_is("writer-lock-fence-ambiguity-unchecked"):
        return []
    lines = prompt.split("\n")
    try:
        labelled = [index for index, line in enumerate(lines) if runner_dispatch_guard_prompt_checks.prompt_scratch_line.scratch_line_values(line)]
        library_hidden = runner_dispatch_guard_prompt_checks.prompt_scratch_line.fenced_line_numbers(lines) if labelled else set()
    except Exception as error:   # 弄坏开关写错这类
        note_unjudged("fence-compare", f"! 派发闸比「草稿目录：」一行的两种围栏认法出错（{type(error).__name__}：{error}），这一次没比")
        return []
    if not labelled:
        return []
    wide = hidden_lines_by_wide_line_rule(lines)
    return [(index + 1, lines[index], index in library_hidden, index in wide) for index in labelled if (index in library_hidden) != (index in wide)]

def scratch_directories_named(prompt):
    """⑯ 建锁的目录：提示里行首「草稿目录：<路径>」点名的（prompt_scratch_directories，与 ⑰ 同一份认法；根本身不算，弄坏开关 writer-lock-root-locked 下也算）。"""
    return prompt_scratch_directories(prompt, include_roots=break_switch_is("writer-lock-root-locked"))

HANDOVER_LOCK_AGENTS = ("implementation-writer", "tooling-writer")   # ⑯ 承接：会接着改上一个写者留在别的草稿目录里的副本的两种类型
HANDOVER_COPY_LABEL = re.compile(LINE_START + r"[^：:\n]*副本[^：:\n]*[：:]")   # 标签里带「副本」的行（承接来的副本写在这样的行里）

def written_scratch_path(mention):
    """文字里写的一处工作根下的路径（SCRATCH_FILE_MENTION 匹配到的那一截）→ 展开成绝对路径、去掉尾巴上连着的汉字与句末的点和冒号、归一之后的路径。"""
    return os.path.normpath(re.sub(r"[^\x00-\x7f]+$", "", expanded_mention(mention)).rstrip(".:"))

def valve_named_paths(prompt, scratches):
    """⑯ 「写者锁已判：<理由>」那一行的理由里点名的工作根下的路径（SCRATCH_FILE_MENTION 认的写法，展开成绝对路径）；本提示「草稿目录：」点名的目录（scratches）底下的不要，
    那些目录照旧由 writer_lock_verdict 的主循环接手。没有已判行（理由空着、是占位也算没有）交空表。
    弄坏开关 valve-named-directories-ignored 下交空表。"""
    reason = valve_reason(prompt, "写者锁已判")
    if reason is None or break_switch_is("valve-named-directories-ignored"):
        return []
    named = [os.path.normpath(scratch) for scratch in scratches]
    paths = []
    for mention in SCRATCH_FILE_MENTION.finditer(reason):
        path = written_scratch_path(mention.group())
        if path not in paths and not any(path == scratch or path.startswith(scratch + "/") for scratch in named):
            paths.append(path)
    return paths

def handover_copy_mentions(prompt, scratches):
    """⑯ 承接：提示里有行首「承接：<前一份报告>」（HANDOVER_LINE；值是 <填 …> 占位的、闭合了的代码块里的不算）时，「承接：」一行与标签里带「副本」的行点名的工作根下的路径
    （展开成绝对路径）；本提示「草稿目录：」点名的目录（scratches）底下的不要，那些目录照旧由 writer_lock_verdict 的主循环接手。没有这样的承接行交空表。
    弄坏开关 handover-lock-unchecked 下交空表。"""
    if break_switch_is("handover-lock-unchecked"):
        return []
    lines = without_code_blocks(prompt).split("\n")
    declared = [HANDOVER_LINE.match(line) for line in lines]
    if not any(found and not SKELETON_PLACEHOLDER.search(found.group(1)) for found in declared):
        return []
    named = [os.path.normpath(scratch) for scratch in scratches]
    mentions = []
    for line, found in zip(lines, declared):
        if not (found or HANDOVER_COPY_LABEL.match(line)):
            continue
        for mention in SCRATCH_FILE_MENTION.finditer(line):
            path = written_scratch_path(mention.group())
            if path not in mentions and not any(path == scratch or path.startswith(scratch + "/") for scratch in named):
                mentions.append(path)
    return mentions

def lock_holder_states(paths, launches, ended, current_transcript, new_digest, where, what):
    """⑯ 承接与「写者锁已判」行点名共用：路径往上最近一处有写者锁的目录（每个目录算一次）→ [(目录, 锁, 持锁的在不在跑, 持锁者的 agent 号或 None)]；
    锁是这一份提示建的不算。锁在而判不了的跳过，认路径或判持锁者出错的整处交 None，两种都照常放行，记一处没判全（site handover-lock；
    where 是说明里这些路径指的目录，what 是没判的那一处叫什么）。读不到共用库、LIB_WRITER_LOCK_BREAK 写错也算出错。"""
    states = []
    try:
        roots = work_roots.scratch_roots()
        for directory in dict.fromkeys(runner_dispatch_guard_prompt_checks.writer_lock.find_lock_directory(os.path.join(path, "probe"), roots, any_entry=True) for path in paths):
            if directory is None:
                continue
            state, lock = runner_dispatch_guard_prompt_checks.writer_lock.lock_state(directory)
            if state == "unreadable":
                note_unjudged("handover-lock", f"! 派发闸判不了{where}的写者锁 {runner_dispatch_guard_prompt_checks.writer_lock.lock_path(directory)}（{lock}）：⑯ {what}这一处没判，照常放行")
                continue
            if state != "ok" or lock.get("prompt_sha256") == new_digest:
                continue
            alive, holder = writer_lock_holder_state(directory, lock, launches, ended, current_transcript, new_digest)
            states.append((directory, lock, alive, holder))
    except Exception as error:
        note_unjudged("handover-lock", f"! 派发闸认{where}上的写者锁出错（{type(error).__name__}：{error}）：⑯ {what}这一处没判，照常放行")
        return None
    return states

def named_lock_verdict(paths, launches, ended, current_transcript, new_digest):
    """⑯ 「写者锁已判」行点名的路径（valve_named_paths）→ ([(目录, 锁, 持锁者)] 持锁的还在跑的, [(目录, 旧锁)] 持锁的已不在跑、放行后接手的)。
    已判行说持锁的已交回或已停：判出已不在跑的（已交回、已停、派发后没起来）换给这一次派发；判出还在跑的与已判行对不上，拒，不接手。
    弄坏开关 valve-named-live-taken 下还在跑的也接手。"""
    states = lock_holder_states(paths, launches, ended, current_transcript, new_digest, "「写者锁已判」行点名的目录", "已判行点名") if paths else None
    if not states:
        return [], []
    take_live = break_switch_is("valve-named-live-taken")
    return ([(directory, lock, holder) for directory, lock, alive, holder in states if alive and not take_live],
            [(directory, lock) for directory, lock, alive, _ in states if take_live or not alive])

def handover_lock_verdict(copies, subagent_type, launches, ended, current_transcript, new_digest, valve):
    """⑯ 承接 → (拒绝说明或 None, 放行后要接手的 [(草稿目录, 旧锁)])。copies 是 handover_copy_mentions 交的路径。
    路径往上最近一处有写者锁的目录里，锁不是这一份提示建的、持锁的已不在跑（已交回、已停、派发后没起来）的才算；持锁的还在跑的不判（承接行可能只是引它的报告）。
    提示里有「写者锁已判」行就接手它们；没有就拒，拒绝说明给整行写法（目录与持锁者照这一次现算）。
    锁在而判不了、认路径或判持锁者出错的，这一次整处不判、照常放行，记一处没判全（site handover-lock，lock_holder_states）。
    弄坏开关 handover-lock-valve-unapplied 下有已判行也不接手。"""
    if not copies:
        return None, []
    states = lock_holder_states(copies, launches, ended, current_transcript, new_digest, "承接来的副本所在草稿目录", "承接")
    stale = [(directory, lock, holder) for directory, lock, alive, holder in states or [] if not alive]
    if not stale:
        return None, []
    if valve:
        return None, ([] if break_switch_is("handover-lock-valve-unapplied") else [(directory, lock) for directory, lock, _ in stale])
    shown = "\n".join(f"  {directory}：锁在 {holder or WRITER_LOCK_HOLDER_UNLAUNCHED} 手里（派发 {lock.get('subagent_type') or '?'}，派发于 {str(lock.get('dispatched_date'))[:10]}），已不在跑"
                      for directory, lock, holder in stale[:6])
    reason = "；".join(f"{directory} 的写者锁在已交回或已停的 {holder} 手里" if holder else f"{directory} 的写者锁是一次没起来的派发留下的"
                       for directory, _, holder in stale[:6]) + "，本次承接它的副本，接手这些锁"
    return (f"✗ 派 {subagent_type} 的提示写了「承接：」，点名的副本在别的草稿目录里，那里的写者锁还在前一个写者手里，提示里没有「写者锁已判：」一行：\n{shown}\n"
            "→ 规矩：写者锁只在提示里「草稿目录：」点名的目录里接手（⑯）；承接来的副本在别的目录，锁不会换到这一件手里，新写者写那份副本时被写闸（.claude/hooks/write-guard.sh 第九道）拒，"
            "白花一趟派发（.claude/hooks/lib_writer_lock.py 文件头「派发接手」）。\n"
            "→ 怎么办：持锁的写者确已交回或已停，在提示里单起一行（整行照抄，目录与持锁者是这一次现算的）：\n"
            f"写者锁已判：{reason}\n"
            "放行之后接手这些目录的锁，前一个写者之后写它们被写闸拒；持锁的还在跑就等它交回。"), []

def writer_lock_holder_state(scratch, lock, launches, ended, current_transcript, new_digest):
    """锁的持有者还在不在跑 → (在跑吗, 持锁者的 agent 号或 None)。认领文件里的 agent 是持锁者；没认领的，在派发那份会话记录里按提示 sha256 与派发时刻找回执；
    都没有的看派发多久了（UNLAUNCHED_GRACE_SECONDS 以内算刚派出去；这一次派发的提示与锁里登记的是同一份、那一件又没起来的，当重试，不算在跑）。
    在跑的判法同 ⑦ 的 is_running。
    没起来的里面，那次派发被挂在同一事件上的别的钩子（或用户）拒掉的（dispatch_refusal_lookup）不论提示改没改都当不在跑；弄坏开关 unclaimed-lock-always-retry 下没认领的锁一律当不在跑，不看回执也不看有没有被拒。"""
    transcript = lock.get("transcript_path") or ""
    if transcript and transcript != current_transcript:   # 别的会话派的：读它那份记录
        launches, ended = session_dispatch_events(transcript)
    else:
        transcript = current_transcript
    now, dispatched = time.time(), lock["dispatched_at"]
    claim = runner_dispatch_guard_prompt_checks.writer_lock.read_claim(scratch, lock)
    holder = claim["agent_id"] if claim else None
    if holder is None and break_switch_is("unclaimed-lock-always-retry"):   # 弄坏开关：没认领的锁一律当重试接手（改过提示的也接手，不看派发回执）
        return False, None
    if holder is None:
        matching = [(moment, agent_id) for agent_id, (moment, _, digest) in launches.items()
                    if digest == lock.get("prompt_sha256") and moment >= dispatched - runner_dispatch_guard_prompt_checks.writer_lock.LAUNCH_SLACK_SECONDS]
        holder = max(matching)[1] if matching else None
    if holder is not None and holder in launches:
        return is_running(holder, launches, ended, transcript), holder
    if holder is not None:   # 认领过，派发回执在读不到的记录里：按认领之后的动静估
        return now - max(dispatched, claim.get("claimed_at", 0) if claim else 0) <= RUNNING_RECENT_SECONDS, holder
    if dispatch_refusal_lookup(transcript)(lock.get("prompt_sha256"), dispatched):
        return False, None   # 派发那一次被挂在同一事件上的别的钩子（或用户）拒掉，没起来：锁是本闸放行时留下的，提示改没改都接手
    if lock.get("prompt_sha256") == new_digest and not break_switch_is("writer-lock-retry-refused"):
        return False, None   # 同一份提示重派、上一次没起来（派发失败这类）：重试，不被自己上一次建的锁挡住
    return now - dispatched <= UNLAUNCHED_GRACE_SECONDS, None

def writer_lock_verdict(hook_input, prompt, subagent_type, launches, ended, session_judged, unjudged, library_errors=None):
    """⑯ → (拒绝说明或 None, 放行后要办的 [(草稿目录, 旧锁或 None)])。--check-prompt（session_judged 为假）不读锁、不判，记进 unjudged。
    读不到 lib_writer_lock.py 时不判、照常放行，照「放行但必记检出」往 library_errors 里记一句（弄坏开关 writer-lock-library-unrecorded 下不记）。"""
    if break_switch_is("writer-lock-unchecked"):
        return None, []
    ambiguous = scratch_line_fence_ambiguity(prompt)
    if ambiguous:
        return code_block_ambiguity_refusal(subagent_type, ambiguous, "「草稿目录：」", SCRATCH_LINE_FENCE_RULE, SCRATCH_LINE_FENCE_HOWTO), []
    directories = scratch_directories_named(prompt)
    copies = handover_copy_mentions(prompt, directories) if subagent_type in HANDOVER_LOCK_AGENTS else []
    valve = valve_reason(prompt, "写者锁已判") is not None and not break_switch_is("writer-lock-valve-ignored")
    named = valve_named_paths(prompt, directories) if valve else []
    if not directories and not copies and not named:
        return None, []
    if runner_dispatch_guard_prompt_checks.writer_lock is None:
        print(f"! 派发闸读不到同目录的 lib_writer_lock.py（{writer_lock_error!r}）：草稿目录写者锁这一次没建也没判", file=sys.stderr)
        if library_errors is not None and not break_switch_is("writer-lock-library-unrecorded"):
            library_errors.append(f"⑯ 草稿目录写者锁读不到同目录的 lib_writer_lock.py（{writer_lock_error!r}），{'、'.join(directories + copies + named)} 这一次没建锁也没判")
        return None, []
    if not session_judged:
        unjudged.append("⑯ 草稿目录写者锁（派的那一刻按锁文件与主会话记录判）")
        return None, []
    current_transcript = hook_input.get("transcript_path") or ""
    actions, conflicts, unreadable_locks = [], [], []
    for scratch in directories:
        if break_switch_is("writer-lock-unreadable-absent"):   # 弄坏开关：判不了的锁当没锁（接着排他建锁撞上它，报「抢先建了」）
            lock = runner_dispatch_guard_prompt_checks.writer_lock.read_lock(scratch)
        else:
            state, lock = runner_dispatch_guard_prompt_checks.writer_lock.lock_state(scratch)   # 锁文件在而判不了的与写闸第九道同一口径：「锁在而判不了」
            if state == "unreadable":
                unreadable_locks.append((scratch, lock))
                continue
        if lock is None:
            actions.append((scratch, None))
            continue
        alive, holder = writer_lock_holder_state(scratch, lock, launches, ended, current_transcript, runner_dispatch_guard_prompt_checks.writer_lock.prompt_digest(prompt))
        if break_switch_is("writer-lock-live-ignored"):
            alive = False
        if break_switch_is("writer-lock-stale-kept"):
            alive = True
        if alive and not valve:
            conflicts.append((scratch, lock, holder))
        else:
            actions.append((scratch, lock))
    new_digest = runner_dispatch_guard_prompt_checks.writer_lock.prompt_digest(prompt)
    named_conflicts, named_actions = named_lock_verdict(named, launches, ended, current_transcript, new_digest)
    handover_message, handover_actions = handover_lock_verdict(copies, subagent_type, launches, ended, current_transcript, new_digest, valve)
    if not conflicts and not unreadable_locks and not handover_message and not named_conflicts:
        taken = {}   # 同一个目录只接手一次（「草稿目录：」、承接与已判行点名到同一处时，取先出现的）
        for directory, lock in actions + named_actions + handover_actions:
            taken.setdefault(directory, (directory, lock))
        return None, list(taken.values())
    parts = []
    if unreadable_locks:
        shown = "\n".join(f"  {scratch}：锁 {runner_dispatch_guard_prompt_checks.writer_lock.lock_path(scratch)}（{problem}）" for scratch, problem in unreadable_locks[:6])
        removal = "；".join(f"rm {runner_dispatch_guard_prompt_checks.writer_lock.lock_path(scratch)}" for scratch, _ in unreadable_locks[:6])
        parts.append(f"✗ 派 {subagent_type} 的草稿目录里写者锁在、却判不了谁持锁：\n{shown}\n"
                     "→ 规矩：锁文件在而判不了（读不了、不是普通文件、不是 JSON 对象、缺 nonce / dispatched_at / prompt_sha256，lib_writer_lock.py 的 lock_state）的，"
                     "与写闸（.claude/hooks/write-guard.sh 第九道）同一口径按「锁在而判不了」办：不当没锁，也不接手，「写者锁已判」行在这里不接手判不了的锁。\n"
                     f"→ 怎么办：主 agent 核过这个目录现在归谁（.owner、在跑的子 agent）之后，删掉那把锁（{removal}；锁的位置是目录的 rm -r）再派，派发闸重建；"
                     "目录里还有子 agent 在写的，给这一件换一个草稿目录。")
    if conflicts:
        shown = "\n".join(f"  {scratch}：持锁的是 {holder or WRITER_LOCK_HOLDER_UNLAUNCHED}（派发 {lock.get('subagent_type') or '?'}，派发于 {str(lock.get('dispatched_date'))[:10]}）"
                          for scratch, lock, holder in conflicts[:6])
        parts.append(f"✗ 派 {subagent_type} 的草稿目录里已经有写者锁，持锁的子 agent 还在跑：\n{shown}\n"
                     "→ 规矩：同一个草稿目录同一时刻只许一个子 agent 写（两位写者交替写同一份副本、互删副本，.claude/kb/milestone/03-third-txn.md 待优化表「草稿目录写者锁」一条）；"
                     "派发时在目录里建 .writer-lock，写范围闸（.claude/hooks/write-guard.sh 第九道）拒不是持锁者的 Write / Edit。\n"
                     "→ 怎么办：等持锁的交回再派，或给这一件换一个草稿目录；持锁的确实已经失效（客户端重启后留下的旧进程、已经停掉的），在提示里单起一行「写者锁已判：<理由>」接手，"
                     "接手之后旧写者的 Write / Edit 被拒。")
    if named_conflicts:
        shown = "\n".join(f"  {scratch}：持锁的是 {holder or WRITER_LOCK_HOLDER_UNLAUNCHED}（派发 {lock.get('subagent_type') or '?'}，派发于 {str(lock.get('dispatched_date'))[:10]}）"
                          for scratch, lock, holder in named_conflicts[:6])
        removal = "；".join(f"rm {runner_dispatch_guard_prompt_checks.writer_lock.lock_path(scratch)}" for scratch, _, _ in named_conflicts[:6])
        parts.append(f"✗ 派 {subagent_type} 的提示里「写者锁已判：」那一行点名的目录，写者锁的持有者还在跑：\n{shown}\n"
                     "→ 规矩：「写者锁已判」行点名的目录，持锁的已交回或已停的才在派发时换给这一次派发的子 agent（.claude/hooks/lib_writer_lock.py 文件头「派发接手」）；"
                     "还在跑的不换，与已判行说的对不上，两位写者交替写同一份副本会互删。\n"
                     f"→ 怎么办：等持锁的交回再派；确已失效的，主 agent 核过这个目录现在归谁（.owner、在跑的子 agent）之后删掉那把锁（{removal}）再派；"
                     "那一行点名的目录不要换锁的，把它从那一行里去掉。")
    if handover_message:
        parts.append(handover_message)
    return "\n".join(parts), []

def acquire_writer_locks(hook_input, prompt, subagent_type, actions):
    """放行之后建锁（目录里没有锁的排他建，有锁的接手）。→ 建锁时被另一次派发抢先的拒绝说明，或 None。建不了锁（目录写不进）的只在 stderr 写一句、不拦派发。"""
    for scratch, previous in actions:
        lock = runner_dispatch_guard_prompt_checks.writer_lock.new_lock(hook_input.get("session_id"), prompt, subagent_type, hook_input.get("transcript_path"))
        try:
            if previous is None:
                if not runner_dispatch_guard_prompt_checks.writer_lock.create_lock(scratch, lock):
                    return (f"✗ 草稿目录 {scratch} 刚被另一次派发抢先建了写者锁。\n"
                            "→ 怎么办：等那一件交回，或给这一件换一个草稿目录；那一件确已失效，重派一次并在提示里单起一行「写者锁已判：<理由>」接手。")
            else:
                runner_dispatch_guard_prompt_checks.writer_lock.replace_lock(scratch, lock)
        except OSError as error:
            note_unjudged("lock-create", f"! 派发闸建不了草稿目录写者锁 {scratch}：{error}；这一次没锁")
    return None

# ⑱ 派发提示常设要求的几行（records/2026-09-16-subagent拆分提案.md 第九十节 ⑤；清单 research/prompts/subagent-landing-audit-2026-10-22/b12-standing-requirements.md）
# ⑱ 没写批次一行也拒的十种类型（scripts/agents/dispatch-prompt.py 的自证按这张表拿真定义打骨架、填完过派发闸）
BATCH_LINE = re.compile(LINE_START + r"批次[* \t]*[：:][* \t]*(.*?)[ \t]*$", re.M)
BATCH_VALUE = re.compile(r"`?(?P<file>records/[^`#\s]+\.md)#(?P<label>[^`\s<。；，,;（(][^`。；，,;（(\n]*?)`?(?:[。；，,;（(].*)?")   # records/<文件>.md#<批号>：批号不空、不是占位，写法不限数字；批号后面接句号、逗号、括注与别的话的照认
BATCH_VALUE_DIGITS_ONLY = re.compile(r"`?(?P<file>records/[^`#\s]+\.md)#(?P<label>\d+)`?")   # 只有弄坏开关 batch-label-digits-only 下还在用：批号只认数字
SELFTEST_LOCATION_LINE = re.compile(LINE_START + r"自证在哪跑[* \t]*[：:][* \t]*(.*?)[ \t]*$", re.M)
LEG_REPORT_LINE = re.compile(LINE_START + r"报告[* \t]*[：:]", re.M)   # 三方腿要的行首「报告：」一行（骨架打的那一行）

def batch_verdict(subagent_type, prompt, project_root):
    """⑱ 批次：提示里行首「批次：」的每一行（闭合围栏块里的不算，照 CommonMark 认，without_code_blocks）的值要形如 records/<文件>.md#<批号>（批号不空、不是占位），
    文件在本仓仓根或登记过的哪一棵工作树里（lib_three_way_legs.py 的 registered_trees；路径里不许有 ..），几行都要对；派 BATCH_LINE_AGENTS 那十种没有这一行也拒。
    「批次：」标签行在 CommonMark 认法与宽的逐行认法下遮不遮不一致就拒（code_block_ambiguous_label_lines）。值是 <填 …> 占位的交 ⑬。交拒绝说明，放行交 None。"""
    if break_switch_is("batch-unchecked"):
        return None
    ambiguous = code_block_ambiguous_label_lines(prompt, BATCH_LABEL_LINE)
    if ambiguous:
        return code_block_ambiguity_refusal(subagent_type, ambiguous, "「批次：」")
    values = [found.group(1).strip() for found in BATCH_LINE.finditer(without_code_blocks(prompt))]
    howto = ("→ 怎么办：写一行「批次：records/<调度记录>.md#<批号>」（例「批次：records/2026-10-22-子agent结果落地审计.md#12」），调度记录写现存的那一份；"
             "骨架 python3 scripts/agents/dispatch-prompt.py <类型> --batch <调度记录>#<批号> 填进这一行。")
    rule = "→ 规矩：每份派发提示写一行「批次：<调度记录>#<批号>」，批内冒出的发现只进下一轮的清单（.claude/main-agent.md「派发提示怎么写」）。\n"
    if not values:
        if subagent_type in BATCH_LINE_AGENTS and not break_switch_is("batch-line-optional"):
            return f"✗ 派 {subagent_type} 的提示里没有「批次：」一行。\n" + rule + howto
        return None
    trees = None
    problems = []
    for value in values:
        if SKELETON_PLACEHOLDER.search(value):
            continue
        shape = (BATCH_VALUE_DIGITS_ONLY if break_switch_is("batch-label-digits-only") else BATCH_VALUE).fullmatch(value)
        if shape is None:
            problems.append(f"「批次：{value[:80]}」认不出：要形如 records/<文件>.md#<批号>")
            continue
        relative = shape.group("file")
        if trees is None:
            trees, _error = runner_dispatch_guard_prompt_checks.three_way_legs.registered_trees(project_root) if runner_dispatch_guard_prompt_checks.three_way_legs is not None else ([project_root], None)
        found = ".." not in relative.split("/") and (break_switch_is("batch-file-unchecked")
                                                     or any(os.path.isfile(os.path.join(tree, relative)) for tree in (trees or [project_root])))
        if not found:
            problems.append(f"「批次：{value[:80]}」点名的 {relative} 在本仓仓根与各棵工作树里都不在")
    if not problems:
        return None
    return f"✗ 派 {subagent_type} 的提示" + "；".join(problems[:4]) + "。\n" + rule + howto

def written_work_root(root):
    """⑱ 拒绝说明里写的工作根：落在家目录下写成 ~/…，不在的写绝对路径（第二台的家目录是另一个，工作根是共享的那一份，不写成 ~/../…）。
    弄坏开关 scratch-line-root-written-under-home 下一律写成 ~/ 起头的相对写法。"""
    home = os.path.expanduser("~")
    if root.startswith(home + "/") or break_switch_is("scratch-line-root-written-under-home"):
        return "~/" + os.path.relpath(root, home)
    return root

def scratch_line_root_verdict(subagent_type, prompt):
    """⑱ 草稿目录落点：行首「草稿目录：」一行（lib_prompt_scratch_line.py 的 scratch_line_values，闭合围栏块里的不算）点名的目录要落在工作根（lib_work_root.py 的 configured_work_root：不看那份库的弄坏开关，
    $SINGLEFS_WORK_ROOT/、~/、$HOME/ 照那份库的 expand_written_path 展开）底下，工作根本身不算；会话临时目录根 /tmp/claude-<uid> 开机被清空，不认
    （弄坏开关 scratch-line-legacy-root-accepted 下认）。占位的交 ⑬。交拒绝说明，放行交 None。"""
    if break_switch_is("scratch-line-root-unchecked"):
        return None
    if runner_dispatch_guard_prompt_checks.prompt_scratch_line is None or work_roots is None:
        note_unjudged("scratch-root", f"! 派发闸读不到 lib_prompt_scratch_line.py 或 lib_work_root.py（{(prompt_scratch_line_error or work_roots_error)!r}），「草稿目录：」一行落在哪这一次没判")
        return None
    try:
        values = runner_dispatch_guard_prompt_checks.prompt_scratch_line.scratch_line_values(prompt)
        roots = [work_roots.configured_work_root()] + ([work_roots.claude_session_temporary_root()] if break_switch_is("scratch-line-legacy-root-accepted") else [])
    except Exception as error:   # 那两份库的弄坏开关写错这类
        note_unjudged("scratch-root", f"! 派发闸认「草稿目录：」一行落在哪出错（{type(error).__name__}：{error}），这一次没判")
        return None
    outside = []
    for value in values:
        expanded = work_roots.expand_written_path(value)
        path = os.path.normpath(expanded) if os.path.isabs(expanded) else None
        if path is None or not any(path.startswith(root + "/") for root in roots):
            outside.append(value)
    if not outside:
        return None
    root_written = written_work_root(work_roots.configured_work_root())
    return (f"✗ 派 {subagent_type} 的提示「草稿目录：」点名的 {'、'.join(outside[:4])} 不在工作根 {root_written}/ 底下"
            "（会话临时目录根 /tmp/claude-<uid> 开机被清空，不认；相对路径、工作根本身也不算）。\n"
            "→ 规矩：报告路径与草稿目录都在工作根下（.claude/agents/ 各定义「输入」一节；工作根的定义只在 .claude/hooks/lib_work_root.py）；"
            "长活的跑法脚本、判法与冻结树放在开机会清空的目录里，重启就没了。\n"
            f"→ 怎么办：改成「草稿目录：{root_written}/<这一件的名字>」（$SINGLEFS_WORK_ROOT/<名字> 也认），认领行与报告行跟着改。")

def selftest_location_verdict(subagent_type, prompt):
    """⑱ 自证在哪跑：行首「自证在哪跑：」一行（闭合围栏块里的不算）的值里点名了工作根或会话临时目录根下的路径（草稿目录、仓副本、scratchpad）就拒；「本仓主工作区」「不跑」放行，占位的交 ⑬。"""
    if break_switch_is("selftest-location-unchecked"):
        return None
    pointed = []
    for found in SELFTEST_LOCATION_LINE.finditer(without_code_blocks(prompt)):
        value = found.group(1).strip()
        if SKELETON_PLACEHOLDER.search(value):
            continue
        pointed += [match.group(0) for match in re.finditer(SCRATCH_MENTION_PREFIX + r"[^\s`'\"，。；、（）()「」]*", value)]
    if not pointed:
        return None
    return (f"✗ 派 {subagent_type} 的提示「自证在哪跑：」指到了草稿目录或仓副本：{'、'.join(pointed[:4])}。\n"
            "→ 规矩：自证登记 .claude/hooks/selftest-registry.tsv 按本仓路径认，副本、草稿里的拷贝跑 --selftest 被 .claude/hooks/project-code-guard.sh 拒。\n"
            "→ 怎么办：写「自证在哪跑：本仓主工作区那一份」；这一轮不跑自证的写「自证在哪跑：不跑」；副本里改过的脚本的自证写进报告「要主 agent 跑的命令」一节交主 agent。")

def leg_report_line_verdict(subagent_type, prompt, project_root):
    """⑰ 三方腿要一行行首「报告：」（LEG_REPORT_LINE，闭合围栏块里的不算）：没有就拒，句中「报告写进 …」不顶这一行。交拒绝说明，放行交 None。"""
    if break_switch_is("leg-report-line-optional") or subagent_type not in three_way_report_types(project_root):
        return None
    if LEG_REPORT_LINE.search(without_code_blocks(prompt)):
        return None
    return (f"✗ 派 {subagent_type} 的提示里没有行首「报告：」一行（句中写的「报告写进 …」不算这一行）。\n"
            "→ 规矩：三方腿的报告就是 research/prompts/<轮>-<模型>-output.md，写在骨架打的「报告：」一行（.claude/rules/three-way-inference.md「云端腿的报告要分段落盘」）。\n"
            f"→ 怎么办：单起一行「报告：research/prompts/<轮>-<模型>-output.md」（轮与模型换成这一轮的轮名与腿的模型），骨架 python3 scripts/agents/dispatch-prompt.py {subagent_type} 打的就是这一行。")

REPORT_LOCATION_ABSOLUTE = re.compile(r"(?:/|~/|\$\{?(?:HOME|SINGLEFS_WORK_ROOT)\}?/)[^/\s]")   # ⑱ 报告落点只判写成绝对路径的（单独一个 / 是分隔记号，不算）

def report_location_verdict(subagent_type, prompt, project_root):
    """⑱ 报告落点（常设要求 B7 的报告那一半，拆分提案第五十六节 ③）：报告行（report_line_mentions，④ 那一种行首句首的认法）点名的路径展开之后是绝对路径的，
    要落在工作根（lib_work_root.py 的 configured_work_root，不看那份库的弄坏开关）或本提示的仓根底下；/tmp 下（会话临时目录根 /tmp/claude-<uid> 在内）与别处的都拒。
    相对路径（research/prompts/… 这类仓里的）、裸文件名不判；占位的交 ⑬。交拒绝说明，放行交 None。"""
    if break_switch_is("report-line-root-unchecked") or work_roots is None:
        return None
    root = work_roots.configured_work_root()
    repository = os.path.normpath(os.path.abspath(project_root))
    outside = []
    for mention in report_line_mentions(prompt):
        named = mention["named"]
        if not named or mention["placeholder"] or not REPORT_LOCATION_ABSOLUTE.match(named):
            continue
        expanded = work_roots.expand_written_path(named)
        if not os.path.isabs(expanded):
            continue
        path = os.path.normpath(expanded)
        if path.startswith(root + "/") or path == repository or path.startswith(repository + "/"):
            continue
        if named not in outside:
            outside.append(named)
    if not outside:
        return None
    root_written = written_work_root(root)
    return (f"✗ 派 {subagent_type} 的提示报告行点名的 {'、'.join(outside[:4])} 不在工作根 {root_written}/ 底下，也不在仓里"
            "（/tmp 下的、会话临时目录根 /tmp/claude-<uid> 下的开机被清空）。\n"
            "→ 规矩：报告路径与草稿目录都在工作根下（.claude/agents/ 各定义「输入」一节；工作根的定义只在 .claude/hooks/lib_work_root.py），三方腿的报告在仓里 research/prompts/。\n"
            f"→ 怎么办：改成「报告：{root_written}/<这一件的名字>/result.md」（$SINGLEFS_WORK_ROOT/… 也认），与「草稿目录：」一行同一个目录。")

def untouched_files_verdict(subagent_type, prompt):
    """⑱ 不碰的文件（常设要求 D3）：派 implementation-writer、tooling-writer 要有一行行首标签里带「不碰」的（骨架打「不碰：<填 …>」），值不空（没有就写「无」）；
    值空着的，下面连着的列表项算值。占位的交 ⑬。交拒绝说明，放行交 None。"""
    if subagent_type not in WRITER_GUARDED_AGENTS or break_switch_is("untouched-line-unchecked"):
        return None
    if any(labelled_values_with_list(prompt, lambda found: "不碰" in found.group("label"))):
        return None
    definition = f".claude/agents/{subagent_type}.md"
    return (f"✗ 派 {subagent_type} 的提示里没有「不碰：…」一行（这一轮别的会话在改、这一件不碰的文件），或那一行是空的。\n"
            f"→ 规矩：{definition}「输入」一节要主 agent 给这一轮别的会话在改的文件（你不碰的）；常设要求 D3（research/prompts/subagent-landing-audit-2026-10-22/b12-standing-requirements.md）。\n"
            f"→ 怎么办：单起一行「不碰：<别的会话在改的文件，逐个列>」，没有就写「不碰：无」；骨架 python3 scripts/agents/dispatch-prompt.py {subagent_type} 打的就是这一行。")

ADDRESSED_HEADING = re.compile(r"^[ \t]{0,3}#{1,6}[ \t]*(?:关的是哪几条|要回答的问题)[^\n]*$", re.M)   # ⑱ 关的是哪几条（调查、调研类写要回答的问题）那一节的标题

def addressed_section_verdict(subagent_type, prompt, fields):
    """⑱ 关的是哪几条（常设要求 A5，只核节在）：派项目定义的类型（定义文件在），提示里要有一节标题是「关的是哪几条」或「要回答的问题」（闭合围栏块里的不算），
    节下到下一个标题之间至少一行不空。节里写的是不是已经抓到的问题、每条有没有验收，钩子看不到。交拒绝说明，放行交 None。"""
    if fields is None or break_switch_is("addressed-section-unchecked"):
        return None
    text = without_code_blocks(prompt)
    for heading in ADDRESSED_HEADING.finditer(text):
        following = HEADING_LINE.search(text, heading.end())
        if text[heading.end():following.start() if following else len(text)].strip():
            return None
    return (f"✗ 派 {subagent_type} 的提示里没有「## 关的是哪几条」一节，或节下是空的。\n"
            "→ 规矩：派一个 agent 之前说清它关的是哪几条已经抓到的问题，逐条列（.claude/main-agent.md「派发提示怎么写」；常设要求 A5，"
            "research/prompts/subagent-landing-audit-2026-10-22/b12-standing-requirements.md）；调查、调研类写「## 要回答的问题」。\n"
            f"→ 怎么办：加一节「## 关的是哪几条」，下面逐条列；骨架 python3 scripts/agents/dispatch-prompt.py {subagent_type} 打的就有这一节。")

# ⑲ 门禁阶段：提示叫子 agent 起 .claude/gate.d/ 下的一道（含 --check 单格）
ZERO_ROUND_MENTION = "零轮"   # ⑱ 提示里写到这个词，就是引了被攻过零轮的改法
ZERO_ROUND_GUARDED_AGENTS = frozenset({"implementation-writer"})   # ⑱ 零轮改法的 A/B 读数只判写产品代码的实现员

ZERO_ROUND_VERDICT_PATH = "research/prompts/x-main-verification.md"   # ⑱ A/B 读数那一行抄进判决之后落的文件：判据库按这类路径判那一行放不放（只有判决里放过行首「A/B 读数：任务 <号>」一行）
NOTATIONS_FORM = "task-and-session-numbers"   # ⑱ 判那一行用的判据库形态：任务号、括号里的会话短号与批号，写闸、检出钩子 ⑰、写入工具与门禁 doc-text 判的是同一个
NOTATIONS_LIBRARY_PATH = os.path.join(os.path.dirname(SCRIPT_HOOK_DIR), "gate.d", "lib-forbidden-notations.py")   # 与 write-guard.sh、门禁 doc-text 共用的判据库
NOTATIONS_LIBRARIES = {}   # 库路径 → 装好的模块：一个进程里同一份库只装一次

def notations_library():
    """判据库 lib-forbidden-notations.py：装过的直接交回，没装过的装一次并记下；读不到、语法错抛异常（失败不记，下一回再试）。"""
    library = NOTATIONS_LIBRARIES.get(NOTATIONS_LIBRARY_PATH)
    if library is None:
        specification = importlib.util.spec_from_file_location("lib_forbidden_notations", NOTATIONS_LIBRARY_PATH)
        library = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(library)
        NOTATIONS_LIBRARIES[NOTATIONS_LIBRARY_PATH] = library
    return library

def zero_round_reading_library_verdict(subagent_type, visible, readings):
    """⑱ 提示里的每一行「A/B 读数：」（readings 是它们在 visible 里的匹配）交判据库按判决里的写法判一遍（写前空串、路径 ZERO_ROUND_VERDICT_PATH）：
    库在判决里拒的写法，这里当场拒，出路写成能照抄进判决的一行。读不到判据库时这一处没判、照常放行，记一处没判全（notations-library）。
    弄坏开关 zero-round-reading-library-unjudged 不交判据库判，notations-library-unrecorded 下读不到判据库时只打 stderr、不记。交拒绝说明，放行交 None。"""
    if break_switch_is("zero-round-reading-library-unjudged"):
        return None
    try:
        library = notations_library()
    except Exception as error:   # 文件不在、语法错都算读不到
        note_unjudged("notations-library", f"! 派发闸读不到判据库 {NOTATIONS_LIBRARY_PATH}（{error!r}），A/B 读数那一行判决里放不放这一条没判")
        return None
    refused = []
    for reading in readings:
        start = visible.rfind("\n", 0, reading.start()) + 1
        end = visible.find("\n", reading.end())
        line = visible[start:len(visible) if end == -1 else end]
        refused += [(line.strip(), name, matched) for _, _, name, matched in library.new_violation_hits(NOTATIONS_FORM, "", line, ZERO_ROUND_VERDICT_PATH)]
    if not refused:
        return None
    listed = "".join(f"\n    {line}  → {name}「{matched}」" for line, name, matched in refused[:6])
    return (f"✗ 派 {subagent_type} 的提示里「A/B 读数：」那一行写了判决里照拒的写法（共 {len(refused)} 处）：{listed}\n"
            f"→ 规矩：这一行要原样抄进判决的零轮改法下面。判决里放过的只有行首「A/B 读数：」一行里按「任务 <号>」「任务号：<号>」「task <号>」写的任务号，值里至多 {library.ZERO_ROUND_READING_MOST_NUMBERS} 个，"
            "引用块（行首的 >，列表里的「- >」也算）里的不算；同一行里另带的别的写法的号（task=<号>、task-server/task log <号>、同 <号>、用「、」「/」「与」并列连着的号）与多出来的号，"
            "写进判决时写闸与写入工具照拒（.claude/rules/three-way-inference.md）。\n"
            "→ 怎么办：把那一行改成「A/B 读数：任务 <号>，<量到什么>」，两臂各写一个「任务 <号>」、写在同一行（另起一行的「任务 <号>」不是读数行，写进判决照拒）；"
            "另带的号换成那件事本身：结局写 type= 与 exit= 那一段，命令写成不带号的样子，日志里那一行只抄那一行的字、不抄日志路径；行首的 > 与「- >」去掉。")

def zero_round_reading_verdict(subagent_type, prompt):
    """⑱ 零轮改法的 A/B 读数（拆分提案第二百一十七节第 1 行）：派 implementation-writer，提示正文（闭合围栏块里的不算）里出现「零轮」，
    同一份提示要有一行行首「A/B 读数：」，值里带至少一个任务号（ZERO_ROUND_TASK_NUMBER）。缺了拒，拒绝说明点名 .claude/rules/three-way-inference.md、写出路。
    有读数行的，每一行再交判据库判一遍（zero_round_reading_library_verdict）：判决里照拒的写法当场拒。
    别的类型、不含「零轮」的提示不判。弄坏开关 zero-round-reading-unchecked 一进来就放行，zero-round-reading-number-unchecked 下有 A/B 读数那一行就放行、不核任务号。
    交拒绝说明，放行交 None。"""
    if subagent_type not in ZERO_ROUND_GUARDED_AGENTS or break_switch_is("zero-round-reading-unchecked"):
        return None
    visible = without_code_blocks(prompt)
    if ZERO_ROUND_MENTION not in visible:
        return None
    readings = list(ZERO_ROUND_READING_LINE.finditer(visible))
    if any(break_switch_is("zero-round-reading-number-unchecked") or ZERO_ROUND_TASK_NUMBER.search(reading.group("value")) for reading in readings):
        return zero_round_reading_library_verdict(subagent_type, visible, readings)
    return (f"✗ 派 {subagent_type} 的提示里写到「零轮」（判决或腿给的、被攻过零轮的改法），却没有行首「A/B 读数：…」一行，或那一行的值里没有任务号。\n"
            "→ 规矩：被攻过零轮的改法写进产品代码之前，先在探针里对这一改法 A/B，量它在被打中的那一支上换不换得到东西，读数的任务号写进派发提示"
            "（.claude/rules/three-way-inference.md）。\n"
            "→ 怎么办：先在探针里对这一改法 A/B（有它与没有它各跑一遍，读被打中的那一支上的量），再派；提示里单起一行「A/B 读数：任务 <号>，<量到什么>」，"
            "号是那两次探针复跑的统一任务服务任务号；提示里的「零轮」只是转述、并没有要写进代码的零轮改法的，把那个词改成别的说法。")

GATE_STAGE_DIRECTORY = os.path.join(".claude", "gate.d")
GATE_STAGE_MENTION = re.compile(r"(?<![\w.\-])(?P<path>(?:[\w.~${}\-]*/)*(?P<name>[A-Za-z0-9][\w\-]*\.sh))(?![\w\-])")
GATE_STAGE_RUN_VERB = re.compile(r"(?:复跑|重跑|跑|执行|运行)(?![过完出的了掉通绿红前员者]|不[了动起完过成])|起(?=[\s`*]*(?:[A-Za-z_][A-Za-z0-9_]*=\S*\s+)*(?:bash\s+|sh\s+)?[\w.~${}/\-]*\.sh)")
GATE_STAGE_WRAPPERS = frozenset({"run-with-memory-cap.sh", "capped.sh", "two-logs.sh", "run-build-directory.sh", "gpu-quiesce-run.sh"})
GATE_STAGE_SCRIPT_TOKEN = re.compile(r"[\w.~${}/\-]+\.(?:sh|py)(?![\w\-])")
GATE_STAGE_FLAG = re.compile(r"(?<![\w\-])--(?:list-items|list|write|tool-versions|selftest)(?![\w\-])")
GATE_STAGE_SENTENCE_END = re.compile(r"[。；;！？!?]")
GATE_STAGE_OTHER_ACTOR = re.compile(r"(?:别的|另一个|另外的?|其他|其它|兄弟|那个|上一个|前一个)[^，,。；;]{0,12}?(?:会话|agent|执行员|实现员|工具员|书记员|调查员|设计员|腿|线)")
GATE_STAGE_CELL_TOKEN = re.compile(r"(?<![\w\-./])[a-z][a-z0-9]*(?:-[a-z0-9]+)+(?![\w\-./])")

def gate_stage_names(project_root):
    """本提示仓根下 .claude/gate.d/ 这一层的阶段文件名（.sh，lib- 起头的共用库不算）。"""
    return {os.path.basename(path) for path in glob.glob(os.path.join(project_root, GATE_STAGE_DIRECTORY, "*.sh"))
            if not os.path.basename(path).startswith("lib-")}

def gate_stage_arguments(after, inside_backticks):
    """阶段名后面同一处写的参数：在反引号里的到反引号为止，不在的到逗号为止；没写 --check 而写了 <格名>（小写连字符的词）的，当 --check 点名那几格。"""
    tail = after.split("`", 1)[0] if inside_backticks else re.split(r"[，,。；;！？!?]", after, 1)[0]
    arguments = GATE_STAGE_FLAG.findall(tail)
    checked = re.search(r"(?<![\w\-])--check[ \t=]+(\S+)", tail)
    if checked:
        arguments += ["--check", checked.group(1).strip("`'\"）)")]
    else:
        cells = GATE_STAGE_CELL_TOKEN.findall(tail)
        if cells:
            arguments += ["--check", ",".join(cells)]
    return arguments

def gate_stage_sentences(line):
    """⑲ 一行按 。；;！？ 切句，反引号里的不切（命令里的 ; 与范围声明 SINGLEFS_RUN_SCOPE='…; …' 是命令的一部分）。→ [句子]。"""
    sentences, start = [], 0
    for end in GATE_STAGE_SENTENCE_END.finditer(line):
        if line.count("`", 0, end.start()) % 2 == 0:
            sentences.append(line[start:end.start()])
            start = end.end()
    sentences.append(line[start:])
    return sentences

def gate_stage_runs(prompt, project_root, library_errors=None):
    """⑲ 提示叫子 agent 起的门禁阶段 → [(那一句, 阶段, 参数, 判成哪一类)]。认法：逐行（闭合了的围栏块里的行、「」『』“”引着的不判），按 。；;！？ 切句，
    提到主 agent 的句子、叙述句（narrated_sentence）不判，阶段名前面紧挨着在说文字改成什么（text_rewrite_before）的不判；句子里点名 .claude/gate.d/ 这一层的阶段（写全路径的要指到 .claude/gate.d/，只写文件名的要在本提示仓根的 .claude/gate.d/ 里有这一份），
    它前面同一句里有跑 / 执行 / 运行 / 起（前面紧挨否定的不算），动词与阶段名之间除了内存包装、capped.sh 这几种包装没有别的脚本、没有 --item（stage-selftest.sh 喂样本那一种），
    就算叫子 agent 起它，参数照 gate_stage_arguments 取；参数是什么都算（命名检查那四格、--list、--list-items、--write、带 --item 的也算：
    子 agent 一律不往统一任务服务交任务，records/2026-11-17-subagent账本自上而下清欠.md 第一百七十四节）。
    弄坏开关 gate-stage-negation-judged 下前面紧挨否定的也算。"""
    stages = gate_stage_names(project_root)
    if not stages:
        return []
    found = []
    text = without_code_blocks(prompt)
    for line in prompt_lines_without_quotes(text):
        lead = ""
        for sentence in gate_stage_sentences(line):
            earlier, lead = lead, lead + sentence + "。"
            if MAIN_AGENT_MENTION.search(earlier + sentence):   # 同一行里前面提到主 agent 的（「要主 agent 跑的命令：…」）不判
                continue
            if narrated_sentence(sentence):
                continue
            for mention in GATE_STAGE_MENTION.finditer(sentence):
                name, path = mention.group("name"), mention.group("path")
                if name not in stages or (path != name and not path.endswith(GATE_STAGE_DIRECTORY + "/" + name)):
                    continue
                before = sentence[:mention.start()]
                if text_rewrite_before(before):
                    continue   # 在说文字改成什么，不是叫子 agent 起它
                verbs = list(GATE_STAGE_RUN_VERB.finditer(before))
                if not verbs:
                    continue
                verb = verbs[-1]
                if NEGATION_BEFORE.search(before[:verb.start()][-4:]) and not break_switch_is("gate-stage-negation-judged"):
                    continue
                if before[:verb.start()].endswith("在") or GATE_STAGE_OTHER_ACTOR.search(before[:verb.start()]):
                    continue   # 「在跑」是转述正在跑的，「别的会话的执行员会跑」是别人跑的，不是叫这个子 agent 跑
                gap = before[verb.end():]
                if "--item" in gap or any(os.path.basename(token) not in GATE_STAGE_WRAPPERS for token in GATE_STAGE_SCRIPT_TOKEN.findall(gap)):
                    continue
                arguments = gate_stage_arguments(sentence[mention.end():], before.count("`") % 2 == 1)
                found.append((sentence.strip(), name, arguments, "门禁阶段，子 agent 一律不起"))
    return found

def gate_stage_verdict(subagent_type, prompt, project_root, library_errors=None):
    """⑲ 交拒绝说明，放行交 None。主 agent 不经派发闸，这里只判派给子 agent 的提示。"""
    if break_switch_is("gate-stage-unchecked"):
        return None
    runs = gate_stage_runs(prompt, project_root, library_errors)
    if not runs:
        return None
    shown = "\n".join(f"  「{sentence[:160]}」（{name}{' ' + ' '.join(arguments) if arguments else ''}：{category}）" for sentence, name, arguments, category in runs[:6])
    return (f"✗ 派 {subagent_type} 的提示叫子 agent 起门禁阶段（.claude/gate.d/ 下的一道，--check 点名单格也算）：\n{shown}\n"
            "→ 规矩：子 agent 不起门禁阶段与各类 lint，也不往统一任务服务交任务：命名检查（code-source-discipline.sh --check 只点名 vague-names、test-file-names、ordinal-names、data-names）、"
            "--list、--list-items、--write、带 --item 的同样写进报告「要主 agent 跑的命令」一节交回（.claude/agent-common.md「门禁」一节）；"
            "执行那一刻 .claude/hooks/project-code-guard.sh 照拒，子 agent 白跑一段。\n"
            "→ 怎么办：删掉那几句，或改成否定句（「不跑门禁阶段（.claude/gate.d/ 下的任何一道，单格点名也不跑）」）；要核的格写成「要主 agent 跑的命令」交回，"
            "由主 agent 提交前照 .claude/skills/gate/staged-gate.md 跑；转述别人写过的原句放进「」里。")

INPUT_EXAMPLE_FALLBACK = ("禁读清单", "禁读：")   # ③ 拒绝说明里「近义的不算」的例子：缺的名字太短或是 ASCII 时用这一对

def missing_inputs_message(subagent_type, missing):
    """③ 缺定义 required-inputs 的拒绝说明：每组按字面子串找的是哪几个字，再给能照抄进提示的几行（与 dispatch-prompt.py 骨架同一个写法）。"""
    definition = f".claude/agents/{subagent_type}.md"
    if break_switch_is("inputs-hint-dropped"):
        return (f"✗ 派 {subagent_type} 的提示缺它定义要的输入：{'、'.join(missing)}（{definition} 的 required-inputs）。\n"
                f"→ 怎么办：照那份定义「输入（主 agent 必须给）」一节补齐，每一样写出它的名字（同义词用 | 分隔，写一个就算）。")
    groups = [[alternative.strip() for alternative in group.split("|") if alternative.strip()] for group in missing]
    first = groups[0][0]
    example = (first, first[:2] + "：") if len(first) >= 3 and not first.isascii() else INPUT_EXAMPLE_FALLBACK
    found_by = [f"  {alternatives[0]}：找「{'」「'.join(alternatives)}」{'这几个字' if len(alternatives) == 1 else '之一'}" for alternatives in groups]
    copy_lines = [f"{alternatives[0]}：<填 {group}>" for alternatives, group in zip(groups, missing)]
    return (f"✗ 派 {subagent_type} 的提示缺它定义要的输入：{'、'.join(alternatives[0] for alternatives in groups)}（{definition} 的 required-inputs）。\n"
            f"→ 认法：每一组按字面子串在整份提示里找，写在哪一行都算，近义的说法不算（例：要的是「{example[0]}」，写「{example[1]}」不算）：\n"
            + "\n".join(found_by) + "\n"
            f"→ 照抄这几行进提示，把 <填 …> 换成实际内容（骨架 python3 scripts/agents/dispatch-prompt.py {subagent_type} 打的就是这几行）：\n"
            + "\n".join(copy_lines) + "\n"
            f"→ 怎么办：每一样该写什么，照 {definition}「输入（主 agent 必须给）」一节。")

def runner_fork_verdict(prompt, project_root):
    """⑪ 执行员：这一段回答的岔路；续做时上一段岔路表里还差的行。交回拒绝说明，放行交 None。"""
    if "只修锚点" in prompt or ("只补落产物" in prompt and not break_switch_is("runner-backfill-refused")):
        return None
    answered = labelled_value(prompt, "这一段回答的岔路")
    if answered is None or answered in NONE_WORDS:
        return ("✗ 派 experiment-runner 的提示里没写「这一段回答的岔路：…」。\n"
                "→ 怎么办：对着岔路单（research/prompts/<轮>-forks.md）写明这一段的量让哪几行够判；"
                "说不出是哪一行，这一段就不该派（.claude/rules/three-way-inference.md「交岔路时写岔路单，派实验时带上它」）；"
                "只补落产物、没有岔路可答的，写「只补落产物」。")
    number_match = REGISTRATION_MENTION.search(prompt)
    if number_match is None:
        return None
    number, rerun = int(number_match.group(1)), number_match.group(2)
    if not registration_has_segment(project_root, number, int(rerun) if rerun else None):
        return None
    which = f"E{number_match.group(1)} 第 {rerun} 次重跑（{number_match.group(0)}）" if rerun else f"E{number_match.group(1)}（{number_match.group(0)}）"
    remaining = labelled_value(prompt, "上一段岔路表里还差")
    if remaining is None:
        return (f"✗ {which} 这一份登记已经有过一段，这是续做，派发提示里却没写「上一段岔路表里还差：…」。\n"
                "→ 怎么办：先读上一段交回的岔路表，把还开着的行点名写进提示；一行都不差就停下交岔路表，不续派。"
                "这一份登记还没跑过（新的 r<n> 第一段）的不算续做，只要「这一段回答的岔路：…」。")
    if remaining in NONE_WORDS:
        return (f"✗ {which} 上一段岔路表里一行都不差了，不续派。\n"
                "→ 怎么办：按岔路单交岔路表给用户；剩下的量在实验页标「够判后未跑」，状态写「已跑（够判）」。")
    return handover_problem(prompt, project_root, which)

REGISTRATION_MENTION = re.compile(r"research/prompts/e(\d+)-(?:preregistration|r(\d+)-prereg)\.md")
HANDOVER_LINE = re.compile(LINE_START + r"(?:承接|[上前]一(?:段|份)(?:执行员)?(?:的)?报告)[* \t]*[：:][* \t]*(.*?)[ \t]*$", re.M)   # ⑪ 续做要的「承接：<上一段的报告>」一行（常设要求 B8）
HANDOVER_PATH = re.compile(r"[^\s`'\"，。；：、（）()「」『』“”,;*<>]*/[^\s`'\"，。；：、（）()「」『』“”,;*<>]+")

def registration_has_segment(project_root, number, rerun):
    """⑪ 这一份登记有没有过一段：原登记（e<号>-preregistration.md）看实验页在不在；第 n 次重跑的登记（e<号>-r<n>-prereg.md）看实验页（中文那一份）里
    有没有一行标题（两到六个井号）写着「第 n 次重跑」（n 写阿拉伯数字或中文数字都认，「第 n 次重跑未跑 / 没跑」不算；执行员跑完一段写进去：
    「### 第 7 次重跑（…）」「#### 第 3 次重跑第二段」「### 2026-10-20：…（第五次重跑…」、标题行状态里的「第 2 次重跑已跑」）。  history-kept: 被判的标题写法示例，日期与重跑次数本身是数据
    弄坏开关 runner-fork-page-continuation 下：实验页在就算续做，不分是哪一份登记；runner-rerun-segments-ignored 下重跑的登记一律当第一段。"""
    pages = glob.glob(os.path.join(project_root, ".claude", "kb", "experiments", f"{number}-*.md"))
    if not pages:
        return False
    if rerun is None or break_switch_is("runner-fork-page-continuation"):
        return True
    if break_switch_is("runner-rerun-segments-ignored"):
        return False
    heading = re.compile(rf"^#{{2,6}}[ \t][^\n]*?第[ \t]*(?:(?<!\d){rerun}(?!\d)|{chinese_numeral(rerun)})[ \t]*次重跑(?!未跑|没跑)", re.M)
    for page in [page for page in pages if not page.endswith((".en.md", ".ja.md"))] or pages:
        try:
            if heading.search(open(page, encoding="utf-8", errors="replace").read()):
                return True
        except OSError as error:   # 读不了就当没跑过：只少要两行，不误拒
            note_unjudged("experiment-page", f"! 派发闸读不了实验页 {page}（{error}），按这一份登记还没跑过判")
    return False

def chinese_numeral(number):
    """1 到 99 的中文数字写法（一、十、十一、二十、二十三）；别的数交一个匹配不到的写法。"""
    digits = "零一二三四五六七八九"
    if not 1 <= number <= 99:
        return "(?!)"
    tens, ones = divmod(number, 10)
    return ("" if tens == 0 else ("十" if tens == 1 else digits[tens] + "十")) + (digits[ones] if ones else "")


def handover_problem(prompt, project_root, which):
    """⑪ 续做（常设要求 B8）：要有一行行首「承接：<上一段执行员的报告路径>」（「上一段报告：」「上一份报告：」也认），值里的路径至少一份在
    （~、$HOME、$SINGLEFS_WORK_ROOT 照 lib_work_root.py 展开；相对路径从本提示的仓根起）。占位的交 ⑬。交拒绝说明，放行交 None。弄坏开关 runner-handover-unchecked 下不判。"""
    if break_switch_is("runner-handover-unchecked"):
        return None
    values = [found.group(1) for found in HANDOVER_LINE.finditer(without_code_blocks(prompt))]
    if any(SKELETON_PLACEHOLDER.search(value) for value in values):
        return None
    howto = ("→ 规矩：主 agent 跑完新派一个同类 agent 接着做，派发提示指到前一份报告与这次的输出路径（.claude/agent-common.md「不做」一节「长活交回主 agent」；常设要求 B8）。\n"
             "→ 怎么办：单起一行「承接：<上一段执行员的报告路径>」（例「承接：~/.local/share/singlefs/work/<上一段>/result.md」）；骨架 python3 scripts/agents/dispatch-prompt.py experiment-runner 打的就是这一行。")
    if not values:
        return f"✗ {which} 这一份登记已经有过一段，这是续做，派发提示里却没写「承接：<上一段执行员的报告路径>」。\n" + howto
    written = [path for value in values for path in HANDOVER_PATH.findall(value)]
    for path in written:
        expanded = expanded_mention(path.rstrip("."))
        if os.path.isfile(expanded if os.path.isabs(expanded) else os.path.join(project_root, expanded)):
            return None
    return (f"✗ {which} 续做的「承接：」一行没点到一份在的报告（写的是 {'、'.join(written[:4]) or '「' + values[0][:60] + '」'}）。\n" + howto)

DISPATCH_UNJUDGED_PREFIX = "派发闸没判全、照常放行"   # 检出记录里这一类的 findings 起头；在看门狗只记不叫醒的那几种起头里（scripts/agents/agent-watch.py 的 UNJUDGED_DETECTION_PREFIXES），看门狗只记进报告、不叫醒主 agent

def record_unjudged(hook_input, findings):
    """放行而有几处判不了（⑥ 自己出错、读不到它的共用模块）：照「放行但必记检出」往检出记录追加一行（结局「只记」）。
    检出记录取 AGENT_HOOK_DETECTIONS，没设取同目录 lib_work_root.py 的 default_detections_path。交一行给人看的说明：记进了哪里，或为什么没记成。"""
    path = os.environ.get("AGENT_HOOK_DETECTIONS")
    if not path and work_roots is not None:
        try:
            path = work_roots.default_detections_path()
        except Exception:   # 工作根的弄坏开关写错这类：不知道往哪写，下面照打一句
            path = None
    if not path:
        return "! 检出没记成：取不到检出记录的位置（AGENT_HOOK_DETECTIONS 没设，同目录 lib_work_root.py 也读不到）"
    tool_input = hook_input.get("tool_input") or {}
    command = (f"派 {tool_input.get('subagent_type') or 'general-purpose'}：{tool_input.get('description') or ''}"[:500] if hook_input
               else "派发（hook 输入读不懂，不知道派的是什么）")
    entry = {"time": datetime.now(timezone.utc).isoformat(), "session_id": hook_input.get("session_id"), "agent_id": hook_input.get("agent_id"),
             "agent_type": hook_input.get("agent_type") or "主 agent", "transcript_path": hook_input.get("transcript_path"),
             "command": command,
             "findings": [f"{DISPATCH_UNJUDGED_PREFIX}：{text}" for text in findings], "outcome": "只记"}
    try:
        os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        return f"! 检出没记成：写不了 {path}（{error}）"
    return f"  照「放行但必记检出」，检出记进 {path}"

def decide(hook_input, project_root, hook_dir=None, take_locks=False, record_detections=False):
    """返回 (0, None 或给 stderr 的提醒) 放行；(2, 说明) 拒绝。各条都判完（judge_dispatch），不成立的几条一次列全，每条带自己的出路。
    take_locks：放行时真的在草稿目录里建写者锁（钩子入口传真，自检里直接调 decide 不建）。
    record_detections：放行而 ⑥ 有读不了的快照清单时往检出记录追加一行（lib_snapshot_conflicts.py 的 record_unreadable；钩子入口传真，自检里直接调 decide 不记）；
    放行而 ⑥ 自己出错（或读不到它的共用模块）时同样追加一行（record_unjudged；弄坏开关 snapshot-error-unrecorded 下不记）；
    放行而 ⑯ 读不到 lib_writer_lock.py 时也追加一行（各条的弄坏开关下那一条不往表里交）。"""
    unreadable, snapshot_errors, library_errors = [], [], []
    del UNJUDGED_NOTES[:]
    refusals, notes, _ = judge_dispatch(hook_input, project_root, hook_dir, take_locks=take_locks, snapshot_unreadable=unreadable, snapshot_errors=snapshot_errors,
                                        library_errors=library_errors)
    if not refusals:
        if record_detections and unreadable and snapshot_library is not None:
            tool_input = hook_input.get("tool_input") or {}
            recorded = snapshot_library.record_unreadable(
                project_root, unreadable, f"派 {tool_input.get('subagent_type') or 'general-purpose'}：{tool_input.get('description') or ''}",
                hook_input.get("session_id"), hook_input.get("agent_id"), hook_input.get("agent_type"), hook_input.get("transcript_path"))
            notes = "\n".join(text for text in (notes, recorded) if text)
        unrecorded = (snapshot_errors if not break_switch_is("snapshot-error-unrecorded") else []) + library_errors + list(UNJUDGED_NOTES)
        if record_detections and unrecorded:
            notes = "\n".join(text for text in (notes, record_unjudged(hook_input, unrecorded)) if text)
        return 0, notes
    subagent_type = (hook_input.get("tool_input") or {}).get("subagent_type") or "general-purpose"
    return 2, combined_refusal(subagent_type, refusals) + (f"\n{notes}" if notes else "")

def combined_refusal(subagent_type, refusals):
    """几条拒绝合成一份说明：一条的原样交回；几条的先写一共几处，再逐处带序号。"""
    if len(refusals) == 1:
        return refusals[0][1]
    blocks = [f"── 第 {number} 处（共 {len(refusals)} 处）──\n{message}" for number, (_, message) in enumerate(refusals, 1)]
    return (f"✗ 派 {subagent_type} 的提示有 {len(refusals)} 处不成立，各条都判完了，下面一次列全，逐处照各自的「→ 怎么办」改；"
            f"改完可以先跑 python3 scripts/agents/dispatch-prompt.py --check <提示文件> --type {subagent_type} 核一遍再派。\n" + "\n".join(blocks))

def without_quoted_text(text):
    """去掉「」『』“”引着的转述（①、㉒、⑲ 判句子之前先摘）：引文可以套、可以折到下一行（quoted_spans 的配法），摘的是最外层那一对整个，里面套的几对跟着摘掉，
    换成一个空格，引文里的换行照留（行数不变）；没配上的开头不算引文，反引号里的引号字符不算引号。
    弄坏开关 quote-nesting-flat 下退回只摘最内层、不跨行的一对（SENTENCE_QUOTED），外层引文里内层几对之间的字留着当正文判。"""
    if break_switch_is("quote-nesting-flat"):
        return SENTENCE_QUOTED.sub(" ", text)
    pieces, position = [], 0
    for start, stop in sorted(quoted_spans(text, keep_backtick_spans=False)):
        if start < position:
            continue   # 套在更外层引文里的一对：外层已经整个摘了
        pieces.append(text[position:start])
        pieces.append(" " + "\n" * text.count("\n", start, stop))
        position = stop
    pieces.append(text[position:])
    return "".join(pieces)

def prompt_lines_without_quotes(text):
    """提示正文逐行摘掉认领行的 --purpose 用途（without_claim_purpose）与引文（without_quoted_text）→ [行]。①、㉒、⑲ 逐行判句子的共同入口。"""
    return without_quoted_text("\n".join(without_claim_purpose(line) for line in text.split("\n"))).split("\n")

def narrated_sentence(sentence):
    """这一句是不是叙述、不是叫子 agent 做什么：句里有对子 agent 说的请 / 你 / 自己 / 可以 / 允许 / 务必（ADDRESSED_TO_AGENT）的不算；
    「现象：」「症状：」起头的（NARRATION_LABEL）、带任务号与退出码的（FINISHED_TASK_ID、FINISHED_TASK_OUTCOME）算。
    弄坏开关 phenomenon-sentences-judged、finished-task-sentences-judged 下各自那一种不放行。"""
    if ADDRESSED_TO_AGENT.search(sentence):
        return False
    if NARRATION_LABEL.search(sentence) and not break_switch_is("phenomenon-sentences-judged"):
        return True
    return bool(FINISHED_TASK_ID.search(sentence) and FINISHED_TASK_OUTCOME.search(sentence) and not break_switch_is("finished-task-sentences-judged"))

def text_rewrite_before(text_before):
    """那一处前面是不是在说文字改成什么：text_before 里最后一个改成 / 改为 / 改记（TEXT_REWRITE）到末尾不超过 TEXT_REWRITE_TAIL_LIMIT 个字、中间没有然后 / 之后 / 再 / 并 / 接着
    （TEXT_REWRITE_BREAK）。弄坏开关 text-rewrite-judged 下恒为假。"""
    if break_switch_is("text-rewrite-judged"):
        return False
    rewrites = list(TEXT_REWRITE.finditer(text_before))
    if not rewrites:
        return False
    tail = text_before[rewrites[-1].end():]
    return len(tail) <= TEXT_REWRITE_TAIL_LIMIT and not TEXT_REWRITE_BREAK.search(tail)

def clause_runs_project_code(clause):
    """一个子句是不是叫人去跑本仓的代码：跑 / 执行 / 起 / 交（RUN_VERB）前面四个字以内没有否定，后面 RUN_OBJECT_DISTANCE 个字以内有要跑的对象（RUN_OBJECT），
    而那个对象前面不是在说文字改成什么（text_rewrite_before）；
    重型入口的名字（RUN_OBJECT_HEAVY）起头在距离里就算，不要求整个落进去。弄坏开关 heavy-entry-start-unjudged 下不判后一种。"""
    for verb in RUN_VERB.finditer(clause):
        if NEGATION_BEFORE.search(clause[:verb.start()][-4:]):
            continue
        run_object = RUN_OBJECT.search(clause[verb.end():verb.end() + RUN_OBJECT_DISTANCE])
        if run_object is not None and not text_rewrite_before(clause[:verb.end() + run_object.start()]):
            return True
        heavy = None if break_switch_is("heavy-entry-start-unjudged") else RUN_OBJECT_HEAVY.search(clause[verb.end():])
        if heavy is not None and heavy.start() < RUN_OBJECT_DISTANCE and not text_rewrite_before(clause[:verb.end() + heavy.start()]):
            return True
    return False

def project_code_run_sentences(prompt):
    """提示正文里叫子 agent 跑项目代码的句子（①）：去掉「」『』“”里的转述（套着的几对整个去掉），按句再按子句切，子句 clause_runs_project_code 为真的，交那一句（提到主 agent 的子句、
    同一句里前面提到主 agent 的子句、叙述句 narrated_sentence 不判）。许可的说法（可以跑 cargo test）、要求的说法（请跑 bash scripts/…、自己跑一下 --selftest）一样拒；否定句（不跑 cargo test）不拒。
    弄坏开关 project-code-run-sentences-unjudged 下一句都不交。"""
    if break_switch_is("project-code-run-sentences-unjudged"):
        return []
    hits = []
    for text in prompt_lines_without_quotes(prompt):
        for sentence in SENTENCE_SPLIT.split(text):
            if narrated_sentence(sentence):
                continue
            separators = [found for found in CLAUSE_SPLIT.finditer(sentence) if sentence.count("`", 0, found.start()) % 2 == 0]
            bounds = [0] + [found.end() for found in separators] + [len(sentence)]
            main_agent_before = False
            for start, stop in zip(bounds, bounds[1:]):
                clause = sentence[start:stop]
                if MAIN_AGENT_MENTION.search(clause):
                    main_agent_before = True
                    continue
                if not main_agent_before and clause_runs_project_code(clause):
                    hits.append(sentence.strip())
                    break
    return hits


def task_submission_sentences(prompt):
    """提示正文里许可、要求子 agent 自己往统一任务服务交任务的句子（㉒）：逐行（闭合了的围栏块里的、「」『』“”引着的不判），
    按 。；;！？ 切句（反引号里的不切，gate_stage_sentences），一句里有 TASK_SUBMISSION_MENTION（task-server/task 的 submit / wait / cancel / clean，经 / 往 / 交给
    统一任务服务或 task-server，run_in_background）而那一处前面同一子句里八个字以内没有否定、后面同一子句里没有不起 / 不交 / 一律不这类的，交那一句；
    提到主 agent 的句子、叙述句（narrated_sentence）、有交回说法（TASK_SUBMISSION_HAND_BACK：写进报告、写进交回、要主 agent 跑的命令）的句子不判，
    那一处前面紧挨着在说文字改成什么（text_rewrite_before）的同样不判
    （「写进报告「要主 agent 跑的命令」一节、由主 agent 交」「拼成提交整条、写进报告的交回节」都是交回的写法）。只读的 task-server/task status、log 不算。
    弄坏开关 task-submission-unchecked 下一句都不交；hand-back-sentences-judged 下有交回说法、没写主 agent 的句子照判。"""
    if break_switch_is("task-submission-unchecked"):
        return []
    hits = []
    visible = without_code_blocks(prompt)
    if not break_switch_is("hand-back-sentences-judged"):
        visible = TASK_SUBMISSION_HAND_BACK_SECTION_QUOTED.sub("要主 agent 跑的命令", visible)
    for line in prompt_lines_without_quotes(visible):
        for sentence in gate_stage_sentences(line):
            if MAIN_AGENT_MENTION.search(sentence) or narrated_sentence(sentence):
                continue
            if TASK_SUBMISSION_HAND_BACK.search(sentence) and not break_switch_is("hand-back-sentences-judged"):
                continue
            separators = [0] + [found.end() for found in CLAUSE_SPLIT.finditer(sentence) if sentence.count("`", 0, found.start()) % 2 == 0]
            for mention in TASK_SUBMISSION_MENTION.finditer(sentence):
                clause_start = max(start for start in separators if start <= mention.start())
                before = sentence[clause_start:mention.start()]
                if TASK_SUBMISSION_NEGATION_BEFORE.search(before) or text_rewrite_before(before):
                    continue
                if TASK_SUBMISSION_NEGATION_AFTER.search(sentence[mention.end():]):
                    continue
                hits.append(sentence.strip())
                break
    return hits


SIBLING_HOOK_TIMEOUT_SECONDS = 30
THIS_HOOK_NAME = "runner-dispatch-guard.sh"
SIBLING_REFUSAL_EXIT = 2

def sibling_hook_commands(project_root, tool_name):
    """项目 .claude/settings.json 里 hooks.PreToolUse 中 matcher 认得 tool_name 的钩子命令，本闸自己（命令里点着本文件名）不算。
    matcher 是对工具名的正则（Agent|Task 这类），空或 * 认全部；读不了、形状不对交 []。"""
    try:
        with open(os.path.join(project_root, ".claude", "settings.json"), encoding="utf-8") as handle:
            groups = json.load(handle)["hooks"]["PreToolUse"]
    except (OSError, ValueError, KeyError, TypeError):
        return []
    commands = []
    for group in groups if isinstance(groups, list) else []:
        if not isinstance(group, dict):
            continue
        matcher = group.get("matcher") or "*"
        try:
            recognized = matcher == "*" or re.fullmatch(matcher, tool_name) is not None
        except re.error:
            recognized = False
        for hook in group.get("hooks") or []:
            command = hook.get("command") if isinstance(hook, dict) and hook.get("type", "command") == "command" else None
            if recognized and isinstance(command, str) and THIS_HOOK_NAME not in command:
                commands.append(command)
    return commands

def sibling_refusals(hook_input, project_root):
    """⑤ ⑯ 同一事件上别的钩子（Agent 事件上除本闸还有 borrowed-convention-guard.sh）会不会拒这次派发 → 会拒（退 2）的钩子命令列表。
    钩子并行跑、谁拒了这次派发就没起来，本闸放行时留下的写者锁与撞文件登记没人用；主会话记录里读被拒的那一次要等记录写进去，重派来得比它快就读不到，
    所以放行之前先把它们按同一份输入跑一遍。它自己跑不起来、超时的当它不拒。弄坏开关 sibling-hooks-unconsulted 下一律交空。"""
    if break_switch_is("sibling-hooks-unconsulted"):
        return []
    refusing = []
    for command in sibling_hook_commands(project_root, str(hook_input.get("tool_name") or "Agent")):
        try:
            completed = subprocess.run(["bash", "-c", command], input=json.dumps(hook_input), capture_output=True, text=True,
                                       timeout=SIBLING_HOOK_TIMEOUT_SECONDS, env=dict(os.environ, CLAUDE_PROJECT_DIR=project_root))
        except (OSError, subprocess.TimeoutExpired, ValueError):
            continue
        if completed.returncode == SIBLING_REFUSAL_EXIT:
            refusing.append(command)
    return refusing

def judge_dispatch(hook_input, project_root, hook_dir=None, check_only=False, take_locks=False, snapshot_unreadable=None, snapshot_errors=None, library_errors=None):
    """派发闸的全部判法，各条都判完：→ ([(条名, 拒绝说明)], 给 stderr 的提醒或 None, [没判的几条])。
    snapshot_unreadable 给了的（列表），⑥ 读不了的快照清单逐份追加进去（decide 放行时据此记检出）；snapshot_errors 给了的（列表），⑥ 自己出错的说明追加进去（同上）；
    library_errors 给了的（列表），⑯ 读不到 lib_writer_lock.py 的说明追加进去（同上）。
    check_only（dispatch-prompt.py --check 经 --check-prompt 调）：不读不写实现员登记、不读主会话记录，⑤ 撞文件与 ⑦ 并发交「没判」；
    不是 check_only 时实现员全过才登记。弄坏开关 stop-at-first 只交第一条（退回逐条试的旧样子）。"""
    unjudged = []
    if break_switch_is("disable-check"):
        return [], None, unjudged
    hook_dir = hook_dir or os.path.join(project_root, ".claude", "hooks")
    tool_input = hook_input.get("tool_input") or {}
    subagent_type = tool_input.get("subagent_type") or "general-purpose"
    prompt = tool_input.get("prompt") or ""
    refusals = []
    running = project_code_run_sentences(prompt)
    if running:
        shown = "\n".join(f"  「{sentence[:160]}」" for sentence in running[:6])
        refusals.append(("叫子 agent 跑项目代码的句子", f"✗ 派 {subagent_type} 的提示正文里有 {len(running)} 句许可或要求它跑项目代码（cargo、本仓的脚本、门禁阶段、自证、样本档、弄坏开关）：\n{shown}\n"
                   "→ 规矩：一切跑项目代码的命令只认经 task-server/task 起，子 agent 一律不交任务（.claude/rules/verification.md、.claude/agent-common.md「不做」一节「子 agent 不交任务」那一条）；"
                   "执行时 .claude/hooks/project-code-guard.sh 也拒。\n"
                   "→ 怎么办：删掉那几句，或改成否定句（「不跑 cargo test，也不跑 --selftest」）；要它核的写成「要主 agent 跑的命令」交回，由主 agent 经服务交；引用别人写过的原句放进「」里。"))
    submitting = task_submission_sentences(prompt)
    if submitting:
        shown = "\n".join(f"  「{sentence[:160]}」" for sentence in submitting[:6])
        refusals.append(("叫子 agent 交任务的句子", f"✗ 派 {subagent_type} 的提示正文里有 {len(submitting)} 句许可或要求它自己往统一任务服务交任务（submit、wait、cancel、clean、run_in_background）：\n{shown}\n"
                   "→ 规矩：子 agent 一律不往统一任务服务交任务（用户 2026-11-01 定，records/2026-11-17-subagent账本自上而下清欠.md 第一百七十四节）；"
                   "执行时 .claude/hooks/project-code-guard.sh 与 bash-command-detector.sh ⑩ 也拒。\n"
                   "→ 怎么办：删掉那几句，或改成否定句（「子 agent 不往统一任务服务交任务」）；要它交回的命令写成「拼成 submit 的整条、写进报告「要主 agent 跑的命令」一节，由主 agent 交」；"
                   "转述别人写过的原句放进「」里。"))
    gate_stage_message = gate_stage_verdict(subagent_type, prompt, project_root, library_errors)
    if gate_stage_message:
        refusals.append(("门禁阶段", gate_stage_message))
    if (subagent_type in GENERAL_PURPOSE_TYPES and valve_reason(prompt, "通用已判") is None
            and not break_switch_is("general-purpose-allowed")):
        refusals.append(("内置通用类型", f"✗ 派 {subagent_type}：内置通用类型起步就带项目 CLAUDE.md 与全部规则（约 13 万 token），一次性的活不派它。\n"
                   "→ 怎么办：subagent_type 写 clerk（.claude/agents/clerk.md，输入只要草稿目录与报告；难的活派发参数 model 写 opus）；"
                   "只读的搜索派 Explore；确要派内置类型，提示里单起一行「通用已判：<为什么 clerk 做不了>」。"))
    unfilled = [(number, line.strip()) for number, line in enumerate(prompt.split("\n"), 1) if SKELETON_PLACEHOLDER.search(line)]
    if unfilled and not break_switch_is("placeholders-ignored"):
        shown = "\n".join(f"  第 {number} 行：{line[:100]}" for number, line in unfilled[:8])
        refusals.append(("骨架占位", f"✗ 派 {subagent_type} 的提示骨架还有 {len(unfilled)} 处占位没填：\n{shown}\n"
                   "→ 怎么办：把每处 <填 …> 换成实际内容，不适用的整行删掉；python3 scripts/agents/dispatch-prompt.py --check <提示文件> 核到没有占位再派。"))
    fields = agent_definition_fields(project_root, subagent_type)
    if fields is None and not COMMON_CONSTRAINTS_READ.search(prompt) and not break_switch_is("general-purpose-free"):
        refusals.append(("开工先读共用约束", f"✗ {subagent_type} 不是项目定义的 agent，不读共用约束，提示里却没写「开工先读 `.claude/agent-common.md`」。\n"
                   "→ 怎么办：先看 .claude/main-agent.md「什么时候派哪个 agent」那张表有没有对应的定义（改门禁、钩子、脚本、定义派 tooling-writer，"
                   "查真假与机理派 investigator）；确要派它，提示里写一行「开工先读 `.claude/agent-common.md`」，长活照那份的「长活交回主 agent」写。"))
    missing = missing_required_inputs(fields or {}, prompt)
    if missing:
        refusals.append(("定义要的输入", missing_inputs_message(subagent_type, missing)))
    report_name_message = report_file_name_verdict(subagent_type, prompt, project_root)
    if report_name_message:
        refusals.append(("报告文件名", report_name_message))
    three_way_report_message = three_way_report_verdict(subagent_type, prompt, project_root)
    if three_way_report_message:
        refusals.append(("三方腿报告路径", three_way_report_message))
    outside = out_of_scope_mentions(project_root, subagent_type, prompt)
    if outside:
        refusals.append(("写范围", f"✗ 提示要 {subagent_type} 写进它写范围之外的路径：{'、'.join(outside)}（.claude/hooks/agent-write-scope.tsv）。\n"
                   "→ 怎么办：改成它写范围里的路径（报告与草稿放工作根 ~/.local/share/singlefs/work/ 下）；范围外的改动交该写的 agent 或主 agent 自己改。"))
    snapshot_code, snapshot_message = snapshot_conflict_verdict(subagent_type, prompt, project_root, snapshot_unreadable, snapshot_errors)
    notes = None
    if snapshot_code:
        refusals.append(("快照冲突", snapshot_message))
    else:
        notes = snapshot_message
    session_judged = not check_only or break_switch_is("check-mode-registers")
    launches, ended = session_dispatch_events(hook_input.get("transcript_path") or "") if session_judged else ({}, {})
    if session_judged:
        concurrency_message = concurrency_verdict(hook_input, launches, ended)
        if concurrency_message:
            refusals.append(("并发", concurrency_message))
    else:
        unjudged.append("⑦ 并发（派的那一刻按主会话记录判）")
    if break_switch_is("retired-lines-refused"):   # 撤掉的那几种标签行与成对写死的线程数照拒
        judged = without_code_blocks(prompt)
        retired = [line.strip() for line in judged.split("\n") if RETIRED_LABEL_LINE.match(line)] + [found.group(0) for found in RETIRED_HOST_PAIR.finditer(judged)]
        if retired:
            refusals.append(("撤掉的行", f"✗ 派 {subagent_type} 的提示里有撤掉的那几种行：{'、'.join(item[:40] for item in retired[:4])}（弄坏开关 retired-lines-refused）。"))
    for standing_name, standing_message in (("批次", batch_verdict(subagent_type, prompt, project_root)),
                                            ("草稿目录落点", scratch_line_root_verdict(subagent_type, prompt)),
                                            ("自证在哪跑", selftest_location_verdict(subagent_type, prompt)),
                                            ("三方腿报告行", leg_report_line_verdict(subagent_type, prompt, project_root)),
                                            ("报告落点", report_location_verdict(subagent_type, prompt, project_root)),
                                            ("不碰的文件", untouched_files_verdict(subagent_type, prompt)),
                                            ("关的是哪几条", addressed_section_verdict(subagent_type, prompt, fields)),
                                            ("零轮改法的 A/B 读数", zero_round_reading_verdict(subagent_type, prompt))):
        if standing_message:
            refusals.append((standing_name, standing_message))
    if subagent_type == "kb-scribe":
        spec_message = kb_scribe_spec_verdict(prompt, project_root, hook_dir)
        if spec_message:
            refusals.append(("书记员规格", spec_message))
    if subagent_type == "experiment-designer":
        admission_message = designer_admission_verdict(prompt, project_root, hook_dir)
        if admission_message:
            refusals.append(("重跑登记准入", admission_message))
    if subagent_type == "mutation-triage" and not MUTATION_TABLE_MENTION.search(prompt) and not break_switch_is("mutation-table-free"):
        refusals.append(("变异表", "✗ 派 mutation-triage 的提示里没有变异表（scripts/mutation/tables/<名>.tsv 或 crates/mutations.tsv 的条目名）。\n"
                   "→ 怎么办：给它表名与条目；没有表的广谱变异是实验，照 .claude/main-agent.md 那一行走 experiment-designer 登记。"))
    writer_files, writer_live_rows = None, []
    if subagent_type in WRITER_GUARDED_AGENTS and (subagent_type == "implementation-writer" or not break_switch_is("writer-collision-implementation-only")):
        if not session_judged:
            unjudged.append("⑤ 与本会话在跑的实现员、工具员撞文件（派的那一刻按登记判）")
        writer_message, writer_files, writer_live_rows = writer_collision_verdict(hook_input, prompt, subagent_type, launches, ended, registry=session_judged)
        if writer_message:
            refusals.append(("实现员要动的文件" if subagent_type == "implementation-writer" else "工具员要改的文件", writer_message))
    if subagent_type == "experiment-runner":
        runner_message = runner_fork_verdict(prompt, project_root)
        if runner_message:
            refusals.append(("执行员岔路", runner_message))
    writer_lock_message, lock_actions = None, []
    if take_locks or check_only:   # 钩子入口（take_locks）才真读锁、建锁；--check-prompt 只写「另判」；自检里直接调 decide 不碰锁文件
        writer_lock_message, lock_actions = writer_lock_verdict(hook_input, prompt, subagent_type, launches, ended, session_judged, unjudged, library_errors)
    if writer_lock_message:
        refusals.append(("草稿目录写者锁", writer_lock_message))
    if break_switch_is("stop-at-first"):
        refusals = refusals[:1]
    elsewhere = sibling_refusals(hook_input, project_root) if take_locks and not refusals else []   # 钩子入口才问；本闸自己要拒的不必问
    if elsewhere:
        notes = "\n".join(text for text in (notes, f"! 同一事件上别的钩子会拒这次派发（{'、'.join(command[:120] for command in elsewhere)}）：本闸这一次没建写者锁、没登撞文件，那次派发起不来，留着只会挡下一件") if text)
    if not refusals and take_locks and lock_actions and not elsewhere:
        lost_race_message = acquire_writer_locks(hook_input, prompt, subagent_type, lock_actions)
        if lost_race_message:
            refusals.append(("草稿目录写者锁", lost_race_message))
    if not refusals and writer_files is not None and session_judged and not elsewhere:
        register_writer(hook_input, prompt, writer_files, writer_live_rows, subagent_type)
    return refusals, notes, unjudged
