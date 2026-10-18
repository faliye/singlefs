"""三方轮开工快照的冲突：要改的文件落在还没判完的三方轮的开工快照里就拒。判「哪几轮开着、快照里列了哪些文件、放行行认不认、拒绝说明怎么写」只在这一份。

两处用这一份：
  .claude/hooks/runner-dispatch-guard.sh   ⑥ 派 kb-scribe、implementation-writer 那一刻：提示正文与它点名的规格文件里出现的路径（「要改的文件」由派发闸自己找）
  scripts/gate/apply-writer-patch.py       往主工作区打实现员补丁之前：补丁改到的文件（crates.patch 的 --- a/ 与 +++ b/ 两侧，变异表增删算 crates/mutations.tsv）

开着的三方轮（open_three_way_rounds）：research/prompts/<轮>-snapshot/ 在、_<轮>-body.md 在、<轮>-main-verification.md 不在；
  research/prompts/abandoned-rounds.tsv 登记了这一轮、而且登记行过得了 lib_abandoned_rounds.py 的判法的（用户叫停、不写判决）不算；登记了而过不了的照旧算开着，原因记进 notes。
  <轮>-halted.md 这一支已撤：文件还在的，开着的轮在 notes 里提醒改道登记表。读不到 lib_abandoned_rounds.py 时不收任何叫停的轮，notes 写一句。
快照清单（snapshot_listed_paths）：快照目录里文件名含 sha256 的每一份，一行「<64 位十六进制哈希><空白>[*]<路径>」；路径从第一段 .claude/ 或 crates/ 起归一成仓根相对路径
  （./.claude/…、tree/crates/…、defs/.claude/… 都认），归一不了的行跳过；读不了的清单记进 notes、跳过，不当成撞快照。
放行行（ruled_snapshot_conflicts）：一行以「快照冲突已判：<轮名> <理由>」开头（行首可以有列表记号与粗体星号），理由非空、不是 <理由> 这种占位；只认这一行，不跨到下一行。
  派发闸在派发提示里找，apply-writer-patch.py 认命令行 --snapshot-ruled 给的那几句。
拒绝说明（refusal_text）：逐轮列撞了的文件（至多 SNAPSHOT_CONFLICTS_SHOWN_LIMIT 个），出路三条：等判决写完、单起一行写放行行、用户已叫停的在登记表收掉。

弄坏开关：调用方传进来 broken(<名>) → 真假（派发闸传 RUNNER_DISPATCH_GUARD_BREAK 那一份，名字与它文件头那几项相同：snapshot-body-optional、snapshot-closed-round-open、
  snapshot-abandoned-ignored、snapshot-abandoned-unchecked、snapshot-loose-lines、snapshot-no-normalize、snapshot-unreadable-aborts、snapshot-unreadable-denies、
  snapshot-valve-ignored、snapshot-valve-loose）；打开后红在哪一格写在调用方的文件头。不传就一项都不开。
"""
import os
import re
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
try:   # 叫停轮登记表一行怎么判：同目录的 lib_abandoned_rounds.py（读不到时不收任何叫停的轮，notes 写一句）
    import lib_abandoned_rounds as abandoned_library
    abandoned_library_error = None
except Exception as error:  # 文件不在、语法错都算读不到
    abandoned_library, abandoned_library_error = None, error

SNAPSHOT_DIRECTORY_SUFFIX = "-snapshot"
SNAPSHOT_LIST_NAME_MARK = "sha256"
# 快照清单的一行：64 位十六进制哈希、空白、路径（sha256sum 二进制模式在路径前加 *）
SNAPSHOT_LIST_LINE = re.compile(r"^\s*[0-9a-fA-F]{64}\s+\*?(\S.*?)\s*$")
# 清单里的路径从第一段 .claude/ 或 crates/ 起取（./.claude/…、tree/crates/…、defs/.claude/… 都归一成仓库根相对路径）
REPOSITORY_PATH_START = re.compile(r"(?:^|/)((?:\.claude|crates)/)")
# 放行口：一行以「快照冲突已判：<轮名> <理由>」开头（行首可以有列表记号与粗体星号）；只在这一行里找，不跨到下一行
SNAPSHOT_CONFLICT_RULED_LINE = re.compile(r"^[ \t>*_\-]*快照冲突已判[* \t]*[：:][* \t]*([^\s，,：:；;。]+)[ \t，,：:；;。]+(.*?)[ \t]*$", re.M)
SNAPSHOT_CONFLICT_RULED_ANYWHERE = re.compile(r"快照冲突已判[*\s]*[：:][*\s]*([^\s，,：:；;。]*)")
SNAPSHOT_CONFLICTS_SHOWN_LIMIT = 20
NONE_WORDS = {"", "无", "没有", "无。", "-", "—", "空", "none"}
ABANDONED_ROUNDS_TABLE = abandoned_library.TABLE_PATH if abandoned_library is not None else os.path.join("research", "prompts", "abandoned-rounds.tsv")
SNAPSHOT_HALTED_SUFFIX = "-halted.md"   # 已撤的一支：文件还在时只在拒绝说明里提醒改道，不认


def never(_name):
    return False


def open_three_way_rounds(project_root, notes=None, broken=never, abandoned_modes=None, who="派发闸"):
    """还没判完的三方轮：[(轮名, 快照目录)]（判法见文件头「开着的三方轮」）。abandoned_modes 是交给 lib_abandoned_rounds.judge_table 的弄坏开关集合的取法，
    不给取那份库自己的（LIB_ABANDONED_ROUNDS_BREAK）；取的时候抛 ValueError（开关写错）就这一次不收叫停的轮、记进 notes。"""
    prompts_directory = os.path.join(project_root, "research", "prompts")
    if not os.path.isdir(prompts_directory):
        return []
    candidates = []
    for entry_name in sorted(os.listdir(prompts_directory)):
        snapshot_directory = os.path.join(prompts_directory, entry_name)
        if not entry_name.endswith(SNAPSHOT_DIRECTORY_SUFFIX) or not os.path.isdir(snapshot_directory):
            continue
        round_name = entry_name[:-len(SNAPSHOT_DIRECTORY_SUFFIX)]
        has_body = os.path.isfile(os.path.join(prompts_directory, f"_{round_name}-body.md"))
        has_verdict = os.path.exists(os.path.join(prompts_directory, f"{round_name}-main-verification.md"))
        if round_name and (has_body or broken("snapshot-body-optional")) and (not has_verdict or broken("snapshot-closed-round-open")):
            candidates.append((round_name, snapshot_directory))
    judgement = None
    if candidates and not broken("snapshot-abandoned-ignored"):
        if abandoned_library is None:
            if notes is not None:
                notes.append(f"! {who}读不到同目录的 lib_abandoned_rounds.py（{abandoned_library_error!r}）：叫停的轮这一次都没收")
        else:
            rows, read_error = abandoned_library.read_rows(project_root)
            if read_error is not None:
                if notes is not None:
                    notes.append(f"! {who}读不了 {ABANDONED_ROUNDS_TABLE}（{read_error}）：叫停的轮这一次都没收")
            else:
                try:
                    modes = abandoned_modes() if abandoned_modes is not None else abandoned_library.breaks()
                    judgement = abandoned_library.judge_table(project_root, rows, modes, only_names={name for name, _ in candidates})
                except ValueError as error:
                    if notes is not None:
                        notes.append(f"! {error}：叫停的轮这一次都没收")
    rounds = []
    for round_name, snapshot_directory in candidates:
        if judgement is not None and any(row.name == round_name for row in judgement.rows):
            problem = None if broken("snapshot-abandoned-unchecked") else abandoned_library.round_problem(judgement, round_name)
            if problem is None:
                continue
            if notes is not None:
                notes.append(f"! {ABANDONED_ROUNDS_TABLE} 登记了 {round_name}，但收不了这一轮：{problem}")
        if notes is not None and os.path.isfile(os.path.join(prompts_directory, f"{round_name}{SNAPSHOT_HALTED_SUFFIX}")):
            notes.append(f"! research/prompts/{round_name}{SNAPSHOT_HALTED_SUFFIX} 在，但{who}不再认这一支：叫停的轮改在 {ABANDONED_ROUNDS_TABLE} 登记一行")
        rounds.append((round_name, snapshot_directory))
    return rounds


def repository_path_of_snapshot_line(line, broken=never):
    """快照清单的一行归一成的仓库根相对路径；不是「<哈希>  <路径>」的形状、或路径里没有 .claude/ 与 crates/ 起头的一段，返回 None。"""
    if broken("snapshot-loose-lines"):
        fields = line.split()
        return fields[-1] if fields else None
    match = SNAPSHOT_LIST_LINE.match(line)
    if match is None:
        return None
    listed_path = match.group(1)
    if broken("snapshot-no-normalize"):
        return listed_path
    start = REPOSITORY_PATH_START.search(listed_path)
    if start is None:
        return None
    normalized = os.path.normpath(listed_path[start.start(1):])
    return None if normalized.startswith("..") else normalized


def snapshot_listed_paths(snapshot_directory, notes, broken=never, who="派发闸"):
    """快照目录里文件名含 sha256 的每份清单读出来的路径：{仓库根相对路径: 清单文件}。认不出的行跳过；读不了的清单记进 notes、跳过。"""
    listed = {}
    for list_name in sorted(os.listdir(snapshot_directory)):
        if SNAPSHOT_LIST_NAME_MARK not in list_name:
            continue
        list_path = os.path.join(snapshot_directory, list_name)
        try:
            with open(list_path, encoding="utf-8", errors="replace") as list_file:
                lines = list_file.read().splitlines()
        except OSError as error:
            if broken("snapshot-unreadable-aborts"):
                raise
            if broken("snapshot-unreadable-denies"):
                listed[f"<读不了的清单 {list_name}>"] = list_path
            notes.append(f"! {who}（快照那一条）读不了快照清单 {list_path}：{error}；这一份跳过，没当成撞快照")
            continue
        for line in lines:
            repository_path = repository_path_of_snapshot_line(line, broken)
            if repository_path is not None:
                listed.setdefault(repository_path, list_path)
    return listed


def ruled_snapshot_conflicts(text, broken=never, none_words=NONE_WORDS):
    """text 里写了「快照冲突已判：<轮名> <理由>」的轮名（理由非空、不是 <理由> 这种占位）；弄坏开关 snapshot-valve-loose 下出现就交 {"*"}（放行全部轮）。"""
    if broken("snapshot-valve-ignored"):
        return set()
    if broken("snapshot-valve-loose"):
        return {"*"} if SNAPSHOT_CONFLICT_RULED_ANYWHERE.search(text) else set()
    ruled = set()
    for match in SNAPSHOT_CONFLICT_RULED_LINE.finditer(text):
        reason = match.group(2).strip()
        if reason not in none_words and not (reason.startswith("<") and reason.endswith(">")):
            ruled.add(match.group(1))
    return ruled


def conflicts_of_paths(project_root, paths, ruled, notes, broken=never, source_name="补丁", who="apply-writer-patch.py"):
    """要改的文件（仓根相对路径的集合）撞了哪几轮的快照：[(轮名, 快照目录, [(路径, 出现在哪, 清单文件)])]；放行行点名了的轮不算。"""
    conflicts = []
    for round_name, snapshot_directory in open_three_way_rounds(project_root, notes, broken, who=who):
        listed = snapshot_listed_paths(snapshot_directory, notes, broken, who=who)
        hits = sorted(path for path in paths if path in listed)
        if hits and round_name not in ruled and "*" not in ruled:
            conflicts.append((round_name, snapshot_directory, [(path, source_name, listed[path]) for path in hits]))
    return conflicts


def refusal_text(project_root, subject, conflicts, notes, wait_action, valve_place, this_time="派发"):
    """拒绝说明：subject 写「派 <类型> 要改的文件」这一类，wait_action 写「再派」「再打」，valve_place 写放行行写在哪（「在派发提示里单起一行写」这一类），
    this_time 写这一次是什么（「派发」「打补丁」）。"""
    listing = []
    for round_name, snapshot_directory, hits in conflicts:
        listing.append(f"  三方轮 {round_name}（开工快照 {os.path.relpath(snapshot_directory, project_root)}/，"
                       f"research/prompts/{round_name}-main-verification.md 还不在）：")
        for path, source_name, list_path in hits[:SNAPSHOT_CONFLICTS_SHOWN_LIMIT]:
            listing.append(f"    {path}（出现在{source_name}；快照清单 {os.path.basename(list_path)}）")
        if len(hits) > SNAPSHOT_CONFLICTS_SHOWN_LIMIT:
            listing.append(f"    （另有 {len(hits) - SNAPSHOT_CONFLICTS_SHOWN_LIMIT} 个）")
    round_names = "、".join(round_name for round_name, _, _ in conflicts)
    valve_lines = "；".join(f"「快照冲突已判：{round_name} <理由>」" for round_name, _, _ in conflicts)
    return (f"✗ {subject}落在还没判完的三方轮 {round_names} 的开工快照里：\n" + "\n".join(listing) + "\n"
            "→ 规矩：.claude/rules/implementation-workflow.md「代码轮派腿之前记一份开工快照」：腿跑着的时候主 agent 不改这些文件，要改的等判决时一起改。\n"
            f"→ 怎么办：等 {round_names} 的判决写完{wait_action}，或把这几处改动并进判决；判过这次{this_time}不碰那一轮的证据（例如只读不改），"
            f"{valve_place}{valve_lines}（理由不许空）；用户已叫停、不会写判决的轮，在 research/prompts/abandoned-rounds.tsv 加一行收掉它"
            "（轮名、日期 YYYY-MM-DD、为什么撂下、去向，制表符分隔，同一轮只一行；门禁 doc-process-records 的 abandoned-rounds 格读同一张表、判同一份，这里不收那一格就红），"
            "行里写出处「records/<文件>.md 第 N 行〔<那一行里含这一轮名字的一段原话，逐字照抄>〕」：〔〕里的原话在 records/ 下那份现存文件里恰好找到一次（行号只当提示），"
            "命中几处时取行号提示落着的那一处，找不到时只在行号提示那一行有这一轮的名字、去掉改名表（.claude/kb/term-renames.md）的新旧名与日期之后〔〕逐字落在那一行里才取那一行（〔〕不写转述）；"
            "取到的那一行里要有这一轮的名字、除这一轮外没有别的轮名、并有「」括起来的用户原话（去掉空白后至少 6 个字），"
            "不认这一轮自己的材料、规则与定义文件、仓外路径（含 ..）、同一行写着别的轮叫停原话的行，之后这一轮的快照不再挡任何派发。"
            + ("\n" + "\n".join(notes) if notes else ""))
