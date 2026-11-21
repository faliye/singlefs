#!/usr/bin/env bash
# 门禁 doc-process-records 的格函数库：abandoned-rounds、knowledge-sync、agent-def-adversarial-review、milestone-close-owed、owed-destination-form、deferred-items-registered、ledger-section-numbers、sealed-ledgers-unchanged、code-round-checklist。
# 由 .claude/gate.d/doc-process-records.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ── abandoned-rounds ────────────────────────────────────────────────────────
check_abandoned_rounds() {
  python3 - "$ABANDONED_ROUNDS_LIBRARY" <<'PY'
import importlib.util, json, os, re, subprocess, sys

prompts_dir = "research/prompts"
registry_path = "research/prompts/abandoned-rounds.tsv"
kb_dir = ".claude/kb"

if not os.path.isdir(prompts_dir):
    print(f"  ! 没有 {prompts_dir}，这一格无对象可判")
    sys.exit(77)

round_name = r"(?P<round>[a-z0-9][a-z0-9-]*-r[0-9]+)"
dispatch_forms = (
    re.compile(r"^_" + round_name + r"-body\.md$"),
    re.compile(r"^" + round_name + r"-opus\.md$"),
    re.compile(r"^" + round_name + r"-sonnet\.md$"),
    re.compile(r"^" + round_name + r"-local-attack\.md$"),
    re.compile(r"^" + round_name + r"-local-defense\.md$"),
)

# 树上现有的，加上归档进版本库的那些：只看树的话，一轮的腿文件一被归档，撂下登记表里那一行就报「认不出它发过腿」。
archived_names = set()
archive_log_error = None                     # 在 git 仓里而 git log 失败：归档的轮次读不到，收尾判红
inside_repository = subprocess.run(["git", "rev-parse", "--is-inside-work-tree"],
                                   capture_output=True, text=True).returncode == 0


def deleted_prompt_paths_log():
    """全部历史里在 prompts_dir 下被删掉的路径：交回 git log --all --diff-filter=D --name-only 的那种结果（带 returncode、stdout、stderr）。
    走全部历史要好几秒，而历史只增不改：结果连同当时全部 ref 的端点记在 git 目录下，下一次只走新端点够得着、旧端点够不着的那几个提交，并进记下的结果。
    旧端点有一个从现在的 ref 够不着了（分支删了、历史改写过），或记下的那一份读不了，整份重走。每个提交的删除各算各的，并出来的集合与整份重走相同。"""
    def git(*arguments):
        return subprocess.run(["git", "-c", "core.quotepath=false", *arguments], capture_output=True, text=True)

    def whole():
        return git("log", "--all", "--diff-filter=D", "--format=", "--name-only", "--", prompts_dir)

    tips = git("rev-parse", "--all")
    common = git("rev-parse", "--git-common-dir")
    if tips.returncode != 0 or common.returncode != 0 or not tips.stdout.strip():
        return whole()
    current_tips = sorted(set(tips.stdout.split()))
    cache_path = os.path.join(common.stdout.strip(), "singlefs-gate-cache", "deleted-prompt-paths.json")
    cached_tips, cached_paths = None, None
    try:
        with open(cache_path, encoding="utf-8") as handle:
            recorded = json.load(handle)
        if recorded.get("directory") == prompts_dir and isinstance(recorded.get("tips"), list) and isinstance(recorded.get("paths"), list):
            cached_tips, cached_paths = [str(tip) for tip in recorded["tips"]], [str(path) for path in recorded["paths"]]
    except (OSError, ValueError):
        pass
    result = None
    if cached_tips:
        unreachable = git("rev-list", "--count", *cached_tips, "--not", "--all")
        if unreachable.returncode == 0 and unreachable.stdout.strip() == "0":
            added = git("log", *current_tips, "--not", *cached_tips, "--diff-filter=D", "--format=", "--name-only", "--", prompts_dir)
            if added.returncode == 0:
                paths = sorted(set(cached_paths) | {line for line in added.stdout.split("\n") if line.strip()})
                result = subprocess.CompletedProcess(added.args, 0, "\n".join(paths) + ("\n" if paths else ""), "")
    if result is None:
        result = whole()
        if result.returncode != 0:
            return result
        paths = sorted({line for line in result.stdout.split("\n") if line.strip()})
    try:
        os.makedirs(os.path.dirname(cache_path), exist_ok=True)
        temporary = f"{cache_path}.{os.getpid()}"
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump({"directory": prompts_dir, "tips": current_tips, "paths": paths}, handle)
        os.replace(temporary, cache_path)
    except OSError:
        pass
    return result


if inside_repository:
    log = deleted_prompt_paths_log()
    if log.returncode != 0:
        archive_log_error = f"git log 退出码 {log.returncode}：{(log.stderr.strip().splitlines() or ['（git 没说原因）'])[-1]}"
    else:
        # 与树上一样只认 research/prompts/ 顶层：子目录里被删的（腿在自己模型目录里造的门禁样本
        # research/prompts/<轮>-<模型>-model/gate-samples/…/_old-topic-r1-body.md、快照里抄的 kb）不是这一轮发出去的腿，
        # 取 basename 会把它们当成轮次文件。弄坏开关 ABANDONED_ROUNDS_BREAK=archived-subdirectories-read：子目录里的也取 basename（改前的读法）。
        read_subdirectories = os.environ.get("ABANDONED_ROUNDS_BREAK", "") == "archived-subdirectories-read"
        for entry in log.stdout.split("\n"):
            entry = entry.strip()
            if not entry.startswith(prompts_dir + "/"):
                continue
            if "/" in entry[len(prompts_dir) + 1:] and not read_subdirectories:
                continue
            archived_names.add(os.path.basename(entry))
# 不在 git 仓里（样本目录）时只看树：那里没有归档可读，判据不变宽
prompt_files = sorted(set(name for name in os.listdir(prompts_dir)
                          if os.path.isfile(os.path.join(prompts_dir, name))) | archived_names)
dispatch_files_by_round = {}
for name in prompt_files:
    matched_round = None
    for form in dispatch_forms:
        match = form.match(name)
        if match:
            matched_round = match.group("round")
            break
    if matched_round is not None:
        dispatch_files_by_round.setdefault(matched_round, []).append(name)

if not dispatch_files_by_round:
    print(f"  ! {prompts_dir} 下没有一个文件名认得出发过腿的轮次（_<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md），这一格无对象可判")
    sys.exit(77)

# kb 小节：一个标题行到下一个标题行之间。只读还没有判决文件的轮要用到的那几条。
rounds_without_verdict_file = [name for name in dispatch_files_by_round if not os.path.isfile(os.path.join(prompts_dir, f"{name}-main-verification.md"))]
kb_sections = []
if rounds_without_verdict_file and os.path.isdir(kb_dir):
    for directory, _subdirectories, file_names in os.walk(kb_dir):
        for file_name in sorted(file_names):
            if not file_name.endswith(".md"):
                continue
            path = os.path.join(directory, file_name)
            section_lines = []
            section_heading = ""
            for line in open(path, encoding="utf-8"):
                if re.match(r"^#{1,6} ", line):
                    if section_lines:
                        kb_sections.append((path, section_heading, "".join(section_lines)))
                    section_lines = [line]
                    section_heading = line.strip()
                else:
                    section_lines.append(line)
            if section_lines:
                kb_sections.append((path, section_heading, "".join(section_lines)))

# 「**判决」后面（去掉紧跟的 **、冒号与空白）接着否定词的不算判过：「**判决**：没判」「**判决：待判**」是在说还没判（文件头 abandoned-rounds 判据 ②）。
# 弄坏开关 ABANDONED_ROUNDS_BREAK=negated-kb-verdict-counted 退回改前：小节里有「**判决」就算
KB_VERDICT_MARK = re.compile(r"\*\*判决")
KB_VERDICT_NEGATIONS = ("没判", "未判", "待判")
negated_kb_verdict_counted = os.environ.get("ABANDONED_ROUNDS_BREAK", "") == "negated-kb-verdict-counted"


def kb_section_has_verdict(text):
    for mark in KB_VERDICT_MARK.finditer(text):
        following = re.sub(r"^[\s*：:]+", "", text[mark.end():mark.end() + 24])
        if negated_kb_verdict_counted or not following.startswith(KB_VERDICT_NEGATIONS):
            return True
    return False


def kb_verdict_location(name):
    citation = re.compile(r"research/prompts/_?" + re.escape(name) + r"-")
    for path, heading, text in kb_sections:
        if citation.search(text) and kb_section_has_verdict(text):
            return f"{path}「{heading}」"
    return None

verdict_file_rounds = []
kb_verdict_rounds = []
rounds_without_any_verdict = []
for name in sorted(dispatch_files_by_round):
    if name not in rounds_without_verdict_file:
        verdict_file_rounds.append(name)
    elif kb_verdict_location(name):
        kb_verdict_rounds.append(name)
    else:
        rounds_without_any_verdict.append(name)

registered_line_numbers = {}
registered_reasons = {}                      # 轮名 → 第三列「为什么撂下」，判「腿一条没派」那一格要读它
malformed_registry_rows = []
source_problem_rows = []
# 登记行一行收不收只在 lib_abandoned_rounds.py 一份（形状、同一轮重复、出处三样），派发闸 ⑥ 调同一份：不收就是红
library_path = sys.argv[1]
try:
    sys.dont_write_bytecode = True
    library_spec = importlib.util.spec_from_file_location("lib_abandoned_rounds", library_path)
    abandoned_library = importlib.util.module_from_spec(library_spec)
    library_spec.loader.exec_module(abandoned_library)
    library_modes = abandoned_library.breaks()
except Exception as error:   # 文件不在、语法错、弄坏开关写错都算读不到
    print(f"  ✗ 读不到登记行的共用判法 {library_path}（{error!r}）")
    print("     → 怎么办：登记行怎么判只有那一份（派发闸 ⑥ 也调它），恢复它，别在这一格另抄一份；LIB_ABANDONED_ROUNDS_BREAK 只写它认的值。")
    sys.exit(1)
registry_rows, registry_error = abandoned_library.read_rows(os.getcwd())
if registry_error is not None:
    print(f"  ✗ 读不了 {registry_path}（{registry_error}）：登记撂下的轮这一次没判")
    print("     → 怎么办：恢复这张表（文件、编码、权限），读不到不是「没有登记」。")
    sys.exit(1)
registry_judgement = abandoned_library.judge_table(os.getcwd(), registry_rows, library_modes)
for row in registry_rows:
    if row.number in registry_judgement.shape:
        malformed_registry_rows.append(registry_judgement.shape[row.number])
        continue
    registered_line_numbers.setdefault(row.name, []).append(row.number)
    if len(row.fields) >= 3:
        registered_reasons[row.name] = row.fields[2]
    if row.number in registry_judgement.sources:
        source_problem_rows.append(f"{row.name}：{registry_judgement.sources[row.number]}")

duplicated_registrations = [f"{name}（第 {', '.join(map(str, numbers))} 行）" for name, numbers in registry_judgement.duplicates.items()]
# 理由里明写没派腿的，不要求认出腿文件：这种轮本来就没有腿文件，正文一被归档就永远认不出，登记表那一行比文件名更权威。
no_leg_form = re.compile(r"腿一条没派|没有派腿|一条腿都没派")
registered_unknown_rounds = [name for name in sorted(registered_line_numbers)
                             if name not in dispatch_files_by_round
                             and not no_leg_form.search(registered_reasons.get(name, ""))]
registered_rounds_with_verdict = [name for name in sorted(registered_line_numbers) if name in verdict_file_rounds or name in kb_verdict_rounds]
unjudged_unregistered_rounds = [name for name in rounds_without_any_verdict if name not in registered_line_numbers]
abandoned_rounds = [name for name in rounds_without_any_verdict if name in registered_line_numbers]

failed = False
if archive_log_error:
    failed = True
    print(f"  ✗ 归档进版本库的腿文件读不到（{archive_log_error}）：只按树判，已归档、既没判决也没登记的轮次这一轮没判")
    print("     → 怎么办：按 git 的报错修好仓库（git fsck 看缺了哪些对象，从远端或备份找回）再跑；读不到归档不是「没有归档」。")
if unjudged_unregistered_rounds:
    failed = True
    print("  ✗ 这些三方轮次发过腿，却没有判决、也不在撂下登记表里：")  # gate-lint:summary
    for name in unjudged_unregistered_rounds:
        print(f"     {name}（{'、'.join(dispatch_files_by_round[name])}）")  # gate-lint:detail
    print(f"     → 怎么办：补判决（写 {prompts_dir}/<轮>-main-verification.md），或在 {registry_path} 加一行：轮名、日期、为什么撂下、去向（四列，制表符分隔）。")
    print("               腿还在跑的轮次，等腿交齐、判决写完再跑门禁；只在 kb 里提到这一轮而没有「**判决」的小节不算判过，「**判决**：」后面写没判、未判、待判的也不算。")
if malformed_registry_rows:
    failed = True
    print(f"  ✗ {registry_path} 这些行写坏了：")  # gate-lint:summary
    for entry in malformed_registry_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：每行四列用制表符分隔：轮名、日期（YYYY-MM-DD）、为什么撂下、去向；四列都要有内容，# 开头的行是注释。")
if duplicated_registrations:
    failed = True
    print("  ✗ 这些轮次在撂下登记表里登记了不止一行：")  # gate-lint:summary
    for entry in duplicated_registrations:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：合成一行，理由与去向写在同一行里。")
if source_problem_rows:
    failed = True
    print(f"  ✗ {registry_path} 这些行的出处收不了（派发闸 ⑥ 读同一张表、判同一份，不收就是红；每一行都要出处）：")  # gate-lint:summary
    for entry in source_problem_rows:
        print(f"     {entry}")  # gate-lint:detail
    print("     → 怎么办：在那一行里写出处「records/<文件>.md 第 N 行〔<那一行里含这一轮名字的一段原话，逐字照抄>〕」（或 records/<文件>.md:N〔…〕）：〔〕里的原话在 records/ 下那份现存文件里要恰好找到一次"
          "（行号只当提示；找不到就照现在的原文逐字重抄，不写转述——只经过术语改名、删日期才找不到的，行号写到那一行照收；命中多次就把行号写成含这一轮名字的那一处，或多抄几个字），"
          "取到的那一行里要有这一轮的名字、除这一轮外没有别的轮名、并有一句「」括起来的用户原话（去掉空白后至少 6 个字）；")
    print("               这一轮自己的材料（research/prompts/<轮>*）、规则与定义文件、仓外路径、同一行写着别的轮叫停原话的行都不认，另找只写这一轮的那一行。")
if registered_unknown_rounds:
    failed = True
    print(f"  ✗ 撂下登记表登记了这些轮，{prompts_dir} 里认不出它们发过腿（没有 _<轮>-body.md、<轮>-opus.md、<轮>-sonnet.md、<轮>-local-attack.md、<轮>-local-defense.md）：")  # gate-lint:summary
    for name in registered_unknown_rounds:
        print(f"     {name}")  # gate-lint:detail
    print("     → 怎么办：轮名多半写错了，改成目录里文件名的前缀；那一轮的文件搬过家就照 .claude/rules/path-moves.md 改；确实没有这一轮就删掉这一行。")
if registered_rounds_with_verdict:
    failed = True
    print("  ✗ 这些轮次登记了撂下，却已经有判决——两处说反话：")  # gate-lint:summary
    for name in registered_rounds_with_verdict:
        where = f"{prompts_dir}/{name}-main-verification.md" if name in verdict_file_rounds else kb_verdict_location(name)
        print(f"     {name}：判决在 {where}")  # gate-lint:detail
    print("     → 怎么办：判决是这一轮的现状，就删掉登记表里那一行；判决那一处其实没判，就把它改成不再写「**判决」或删掉那份判决文件，再留登记。")
if failed:
    sys.exit(1)

checked = len(dispatch_files_by_round)
print(f"  ✓ 发过腿的三方轮次都有判决或撂下登记（查了 {checked} 轮：判决文件 {len(verdict_file_rounds)} 轮、kb 小节里的判决 {len(kb_verdict_rounds)} 轮、登记撂下 {len(abandoned_rounds)} 轮）")
sourced_rows = len(registry_rows) if abandoned_library.source_required(library_modes) else 0
print(f"    登记表 {len(registry_rows)} 行：形状与同一轮重复都查了，出处按派发闸 ⑥ 同一份判法查了 {sourced_rows} 行")
moved_rows = [f"{row.name}：出处 {registry_judgement.moved[row.number][0]} 写的第 {registry_judgement.moved[row.number][1]} 行，原话现在在第 {registry_judgement.moved[row.number][2]} 行"
              for row in registry_rows if row.number in registry_judgement.moved]
if moved_rows:
    print(f"    出处行号对不上而〔〕原话恰好找到一次、照收的 {len(moved_rows)} 行（行号只当提示，顺手把行号改成现在的）：{'；'.join(moved_rows)}")
hinted_rows = [f"{row.name}：出处 {registry_judgement.hinted[row.number][0]} 第 {registry_judgement.hinted[row.number][1]} 行：{registry_judgement.hinted[row.number][2]}"
               for row in registry_rows if row.number in registry_judgement.hinted]
if hinted_rows:
    print(f"    出处〔〕原话命中几处或找不到、照行号提示取那一行照收的 {len(hinted_rows)} 行（取到的那一行是不是用户为这一轮说的，靠主 agent 核）：{'；'.join(hinted_rows)}")
PY
}

# ── knowledge-sync ──────────────────────────────────────────────────────────
check_knowledge_sync() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  # 同步记录这一轮新增的行（未跟踪的整份算新增）：旧记录里这一轮新加的「改了 / 补了」行也要判载体在不在改动范围里
  load_added_record_lines || {
    echo "  ✗ 取不到同步记录这一轮新增了哪些行（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到新增行时旧记录里新加的处置行没法判，不是通过。"
    return 1
  }
  # 改动清单经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  python3 - "$gate_base" <(printf '%s\n' "$changed_since_gate") <(printf '%s\n' "$added_record_lines") "$TRIGGER_TABLE" "$STAGE_NAME" <<'PY'
import os, re, subprocess, sys

base = sys.argv[1]
with open(sys.argv[2], encoding="utf-8", errors="replace") as handle:
    changed_files = sorted({path for path in handle.read().split("\n") if path})
changed_set = set(changed_files)
# 同步记录这一轮新增的行：{记录路径: {整行}}（gate_added_lines 的「路径<TAB>行文」）
added_lines_by_record = {}
with open(sys.argv[3], encoding="utf-8", errors="replace") as handle:
    for added in handle.read().split("\n"):
        if "\t" in added:
            added_path, added_text = added.split("\t", 1)
            added_lines_by_record.setdefault(added_path, set()).add(added_text)

# 触发文件的清单只读外层传进来的 TRIGGER_TABLE（与 batch-scope 格同一份），不在这里再存一份：两格算的范围要一致。
TRIGGER_TABLE = sys.argv[4]
STAGE_NAME = sys.argv[5]
if not os.path.isfile(TRIGGER_TABLE):
    print(f"  ✗ 读不到触发文件清单 {TRIGGER_TABLE}")
    print("     → 怎么办：那份表是触发文件的唯一登记位，这一格与 batch-scope 格都读它；恢复它，别在脚本里再存一份清单。")
    sys.exit(1)
trigger_patterns = []
for raw in open(TRIGGER_TABLE, encoding="utf-8").read().split("\n"):
    body = raw.split("\t")[0].strip()
    if not body or body.startswith("#"):
        continue
    trigger_patterns.append(re.compile(body))
if not trigger_patterns:
    print(f"  ✗ 触发文件清单 {TRIGGER_TABLE} 里一条正则都没有")
    print("     → 怎么办：一行一条「<python 正则><制表符>#<这一类是什么>」，空表等于这道检查整个关掉。")
    sys.exit(1)
trigger_files = [path for path in changed_files if any(pattern.search(path) for pattern in trigger_patterns)]
outside_trigger_count = len(changed_files) - len(trigger_files)
if not trigger_files:
    print(f"  ! 这次改动没碰规则、agent、hook、门禁、脚本与实现，这一格无对象可判（改动范围 {len(changed_files)} 个文件，基准 {base}）")
    sys.exit(77)

marker_line = "<!-- knowledge-sync -->"
record_name = re.compile(r"^research/prompts/[^/]+-sync\.md$")
records = []
fresh_records = set()
uncommitted_records = set()
unmarked_candidates = []
for path in changed_files:
    if not record_name.search(path) or not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    if any(line.strip() == marker_line for line in text.split("\n")):
        records.append((path, text))
        # 这一轮新写的记录：基准那一版里没有这个文件。只是被改过的旧记录不算，它早先的行点名的是更早一轮改的载体。
        # 不用 git diff --name-status：它看不见未跟踪的新文件，而这一轮刚写出来还没 add 的记录正是那一种。
        exists_at_base = subprocess.run(
            ["git", "cat-file", "-e", f"{base}:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_base:
            fresh_records.add(path)
        # ⑤ 的基准是 HEAD，不是 GATE_BASE：它问这一次提交要带上的记录带没带原始材料；
        # 拿 GATE_BASE 会连上一个提交里写好、已经冻结的记录一起判（gate-ok 不存在时基准退回 HEAD~1）。
        exists_at_head = subprocess.run(
            ["git", "cat-file", "-e", f"HEAD:{path}"],
            capture_output=True, text=True).returncode == 0
        if not exists_at_head:
            uncommitted_records.add(path)
    else:
        unmarked_candidates.append(path)

fence_open = re.compile(r"^ {0,3}(`{3,}|~{3,})")
heading = re.compile(r"^#{1,2}\s")


def second_level_sections(text):
    """二级标题整行 → 小节体 [(行号, 行, 是否在代码围栏里)]；同名标题只认第一次，代码围栏里的 # 行不算标题。"""
    sections = {}
    current_body = None
    open_fence = None
    for line_number, line in enumerate(text.split("\n"), 1):
        if open_fence is None:
            fence_match = fence_open.match(line)
            if fence_match:
                open_fence = fence_match.group(1)
                if current_body is not None:
                    current_body.append((line_number, line, True))
                continue
            if heading.match(line):
                title = line.rstrip()
                if title.startswith("## ") and title not in sections:
                    sections[title] = []
                    current_body = sections[title]
                else:
                    current_body = None
                continue
            if current_body is not None:
                current_body.append((line_number, line, False))
        else:
            if re.match(r"^ {0,3}" + re.escape(open_fence[0]) + "{" + str(len(open_fence)) + r",}\s*$", line):
                open_fence = None
            if current_body is not None:
                current_body.append((line_number, line, True))
    return sections


def table_cells(row):
    inner = row.strip()
    if inner.startswith("|"):
        inner = inner[1:]
    if inner.endswith("|") and not inner.endswith("\\|"):
        inner = inner[:-1]
    return [cell.strip() for cell in re.split(r"(?<!\\)\|", inner)]


carrier_form = re.compile(r"^(?P<path>\S(?:.*\S)?):(?P<line>[1-9][0-9]*)$")
separator_cell = re.compile(r"^:?-{3,}:?$")

missing_triggers = [path for path in trigger_files if not any(path in text for _record_path, text in records)]
search_problems = []
table_problems = []
carrier_problems = []
disposition_start_problems = []
empty_reason_problems = []
out_of_range_problems = []
disposition_counts = {"改了": 0, "补了": 0, "不改": 0}
evidence_problems = []
evidence_rows_total = 0
evidence_record_count = 0


def tables_in(section_body):
    """小节体里的几张表：连着的表格行算一张，代码围栏里的行不算。返回 [[(行号, 整行), …], …]。"""
    runs = []
    previous_was_table_line = False
    for line_number, line, fenced in section_body:
        is_table_line = (not fenced) and line.lstrip().startswith("|")
        if is_table_line and not previous_was_table_line:
            runs.append([])
        if is_table_line:
            runs[-1].append((line_number, line))
        previous_was_table_line = is_table_line
    return runs


def newline_count(path):
    """`wc -l` 报的数：数换行符，不补末尾。不以换行结尾的文件两者差 1，取一个口径，别让写记录的人猜。"""
    with open(path, "rb") as handle:
        return handle.read().count(b"\n")


for record_path, text in records:
    sections = second_level_sections(text)
    if "## 搜索" not in sections:
        search_problems.append(f"{record_path}：没有「## 搜索」小节")
    elif not any(line.strip() for _number, line, _fenced in sections["## 搜索"]):
        search_problems.append(f"{record_path}：「## 搜索」小节是空的")
    # ⑤ 原始证据只对还没进 HEAD 的记录判：已提交的记录改不得，它点名的材料也归档了。
    if record_path in uncommitted_records:
        evidence_record_count += 1
        if "## 原始证据" not in sections:
            evidence_problems.append(f"{record_path}：没有「## 原始证据」小节")
        else:
            evidence_tables = tables_in(sections["## 原始证据"])
            if not evidence_tables:
                evidence_problems.append(f"{record_path}：「## 原始证据」小节里没有表")
            elif len(evidence_tables) > 1:
                evidence_problems.append(f"{record_path}：「## 原始证据」小节里有 {len(evidence_tables)} 张表，只许一张（第二张从第 {evidence_tables[1][0][0]} 行起）")
            else:
                evidence_rows = evidence_tables[0]
                evidence_header_number, evidence_header_line = evidence_rows[0]
                if table_cells(evidence_header_line) != ["材料", "路径", "行数"]:
                    evidence_problems.append(f"{record_path}:{evidence_header_number}：「## 原始证据」的表头不是 | 材料 | 路径 | 行数 |，实际是 {evidence_header_line.strip()}")
                elif (len(evidence_rows) < 2 or len(table_cells(evidence_rows[1][1])) != 3
                      or not all(separator_cell.match(cell) for cell in table_cells(evidence_rows[1][1]))):
                    evidence_problems.append(f"{record_path}:{evidence_header_number + 1}：「## 原始证据」表头下面一行不是三格的分隔行 |---|---|---|")
                elif len(evidence_rows) < 3:
                    evidence_problems.append(f"{record_path}:{evidence_header_number}：「## 原始证据」的表一行都没有——判错一行之后拿不出任何原始材料，正是这一条要拦的")
                else:
                    for row_number, row_line in evidence_rows[2:]:
                        cells = table_cells(row_line)
                        location = f"{record_path}:{row_number}"
                        evidence_rows_total += 1
                        if len(cells) != 3:
                            evidence_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
                            continue
                        material, evidence_path, declared_lines = cells
                        if len(evidence_path) >= 2 and evidence_path.startswith("`") and evidence_path.endswith("`"):
                            evidence_path = evidence_path[1:-1].strip()
                        if not material:
                            evidence_problems.append(f"{location}：材料格是空的，要写这份材料是什么（事实表 / 候选表 / 逐行判定报告）")
                        if evidence_path.startswith(("/", "./", "../")) or "/../" in evidence_path:
                            evidence_problems.append(f"{location}：路径不是从仓库根起写的：{evidence_path}")
                        elif not os.path.isfile(evidence_path):
                            evidence_problems.append(f"{location}：{evidence_path} 不在仓里——原始材料没入库，判错之后无从复盘")
                        elif not re.fullmatch(r"[0-9]+", declared_lines):
                            evidence_problems.append(f"{location}：行数格不是十进制整数：{declared_lines or '（空）'}")
                        else:
                            actual_lines = newline_count(evidence_path)
                            if actual_lines != int(declared_lines):
                                evidence_problems.append(f"{location}：行数写 {declared_lines}，{evidence_path} 实际 {actual_lines}（wc -l）")
    if "## 命中处置" not in sections:
        table_problems.append(f"{record_path}：没有「## 命中处置」小节")
        continue
    table_runs = tables_in(sections["## 命中处置"])
    if not table_runs:
        table_problems.append(f"{record_path}：「## 命中处置」小节里没有表")
        continue
    if len(table_runs) > 1:
        table_problems.append(f"{record_path}：「## 命中处置」小节里有 {len(table_runs)} 张表，只许一张（第二张从第 {table_runs[1][0][0]} 行起）")
        continue
    table_rows = table_runs[0]
    header_number, header_line = table_rows[0]
    if table_cells(header_line) != ["载体", "原句", "处置"]:
        table_problems.append(f"{record_path}:{header_number}：「## 命中处置」的表头不是 | 载体 | 原句 | 处置 |，实际是 {header_line.strip()}")
        continue
    if len(table_rows) < 2 or len(table_cells(table_rows[1][1])) != 3 or not all(separator_cell.match(cell) for cell in table_cells(table_rows[1][1])):
        table_problems.append(f"{record_path}:{header_number + 1}：表头下面一行不是三格的分隔行 |---|---|---|")
        continue
    for row_number, row_line in table_rows[2:]:
        cells = table_cells(row_line)
        location = f"{record_path}:{row_number}"
        if len(cells) != 3:
            table_problems.append(f"{location}：这一行拆出 {len(cells)} 格，要三格")
            continue
        carrier_text, _original_sentence, disposition = cells
        if len(carrier_text) >= 2 and carrier_text.startswith("`") and carrier_text.endswith("`"):
            carrier_text = carrier_text[1:-1].strip()
        carrier_match = carrier_form.match(carrier_text)
        carrier_path = None
        if not carrier_match:
            carrier_problems.append(f"{location}：载体格不是「路径:行号」：{carrier_text}")
        else:
            candidate_path = carrier_match.group("path")
            if candidate_path.startswith(("/", "./", "../")) or "/../" in candidate_path:
                carrier_problems.append(f"{location}：载体路径不是从仓库根起写的：{candidate_path}")
            elif not os.path.exists(candidate_path) and candidate_path not in changed_set:
                carrier_problems.append(f"{location}：载体路径 {candidate_path} 在仓里不存在，也不在改动范围里")
            else:
                carrier_path = candidate_path
        if disposition.startswith("改了") or disposition.startswith("补了"):
            disposition_counts[disposition[:2]] += 1
            # 这一轮写的处置行才判：新写的记录整份都是；旧记录只判这一轮新加的那几行（早先的行点名的是更早一轮改的载体）
            written_this_round = (os.environ.get("KNOWLEDGE_SYNC_BREAK", "") == "old-rows-rejudged"
                                  or record_path in fresh_records or row_line in added_lines_by_record.get(record_path, set()))
            if (written_this_round
                    and carrier_path is not None and carrier_path not in changed_set):
                out_of_range_problems.append(f"{location}：处置写「{disposition[:2]}」，载体 {carrier_path} 不在改动范围里")
        elif disposition.startswith("不改："):
            disposition_counts["不改"] += 1
            if not disposition[len("不改："):].strip():
                empty_reason_problems.append(f"{location}：「不改：」后面没写理由")
        else:
            disposition_start_problems.append(f"{location}：处置开头不合法：{disposition or '（空）'}")

sync_step = ("阶段同步怎么做：阶段任务结束时派 sweep 做阶段同步（sweep 定义里的第四种活），主 agent 判完每处命中写 "
             f"research/prompts/<阶段>-sync.md，格式见 .claude/gate.d/{STAGE_NAME} 文件头「knowledge-sync」那一段。")
failed = False
if missing_triggers:
    failed = True
    print(f"  ✗ {len(missing_triggers)} 个触发文件没在任何同步记录里点名（改动范围里带 <!-- knowledge-sync --> 的 research/prompts/*-sync.md 共 {len(records)} 份，一份都没按路径提到它）：")
    for path in missing_triggers:
        print(f"      {path}")
    if unmarked_candidates:
        print("     这几份文件名像同步记录，但没有只写 <!-- knowledge-sync --> 的那一行，不算：")
        for path in unmarked_candidates:
            print(f"      {path}")
    print("     → 怎么办：这一阶段还没做知识同步就先做；做过的，把上面每个文件按路径写全补进同步记录的「触发文件」一行。")
if search_problems:
    failed = True
    print(f"  ✗ {len(search_problems)} 份同步记录缺「## 搜索」或它是空的：")
    for entry in search_problems:
        print(f"      {entry}")
    print("     → 怎么办：加「## 搜索」小节（标题逐字写、后面不加字），写回扫用的每条命令与它的命中计数。")
if table_problems:
    failed = True
    print(f"  ✗ {len(table_problems)} 处「## 命中处置」的表写坏了：")
    for entry in table_problems:
        print(f"      {entry}")
    print("     → 怎么办：「## 命中处置」下面恰好一张表，表头逐字是 | 载体 | 原句 | 处置 |，第二行 |---|---|---|；每行三格，原句里的 | 写成 \\|；没有命中也留表头（0 行）。")
if carrier_problems:
    failed = True
    print(f"  ✗ {len(carrier_problems)} 行载体格不是仓库根起的「路径:行号」或指不到文件：")
    for entry in carrier_problems:
        print(f"      {entry}")
    print("     → 怎么办：载体格写仓库根起的 路径:行号（例 .claude/agent-common.md:120），路径要在仓里现存；别写成相对 research/prompts/ 的路径。")
if disposition_start_problems:
    failed = True
    print(f"  ✗ {len(disposition_start_problems)} 行处置开头不合法（只认「改了」「补了」「不改：」）：")
    for entry in disposition_start_problems:
        print(f"      {entry}")
    print("     → 怎么办：每处命中先判完再写：改了写「改了：…」，新加的写「补了：…」，不动写「不改：理由」；别写「待定」「看过」。")
if empty_reason_problems:
    failed = True
    print(f"  ✗ {len(empty_reason_problems)} 行「不改：」没写理由：")
    for entry in empty_reason_problems:
        print(f"      {entry}")
    print("     → 怎么办：「不改：」后面写不改的理由，例：冻结证据 / 说的是那一次发生的事 / 同一个词、不同的事。")
if evidence_problems:
    failed = True
    print(f"  ✗ {len(evidence_problems)} 处「## 原始证据」写坏了，或点名的材料不在仓里、行数对不上：")
    for entry in evidence_problems:
        print(f"      {entry}")
    print("     → 怎么办：这一轮新写的同步记录要有「## 原始证据」小节，小节里恰好一张表，表头逐字 | 材料 | 路径 | 行数 |，至少一行；")
    print("               把这一阶段的事实表、候选表与每份逐行判定报告从 /tmp 挪进 research/prompts/ 跟着记录一起提交，行数写 wc -l 报的数。")
if out_of_range_problems:
    failed = True
    print(f"  ✗ {len(out_of_range_problems)} 行处置写「改了」「补了」而载体不在改动范围里（记录说改了，文件没动）：")
    for entry in out_of_range_problems:
        print(f"      {entry}")
    print("     → 怎么办：去把那一处真的改了；判下来不该改的，处置写成「不改：理由」；也对一遍载体路径是不是写成了别的文件。")
if failed:
    print(f"     → {sync_step}")
    sys.exit(1)
hit_rows = sum(disposition_counts.values())
print(f"  ✓ 触发文件 {len(trigger_files)} 个都在同步记录里点名（同步记录 {len(records)} 份；命中 {hit_rows} 行："
      f"改了 {disposition_counts['改了']} / 补了 {disposition_counts['补了']} / 不改 {disposition_counts['不改']}；基准 {base}）；"
      f"这一次提交新带的同步记录 {evidence_record_count} 份，点名的 {evidence_rows_total} 份原始材料都在仓里、行数对得上；"
      f"改动范围里另有 {outside_trigger_count} 个文件不在触发范围")
PY
}

# ── agent-def-adversarial-review ────────────────────────────────────────────
check_agent_def_adversarial_review() {
  ((inside_git)) || { not_git_repository; return; }
  load_changed_since_gate || {
    echo "  ✗ 取不到这次改动碰了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到改动范围时这一格什么都没比，不是通过。"
    return 1
  }
  check_review_pending_table || return 1
  check_review_exempt_frozen "$AGENT_DEF_EXEMPT_TABLE" || return 1
  local index
  local -a forms=()
  for ((index = 1; index < ${#GOVERNANCE_CLASS_NAMES[@]}; index++)); do
    forms+=("${GOVERNANCE_CLASS_PATTERNS[index]}"$'\t'"${GOVERNANCE_CLASS_EXCLUDES[index]}")
  done
  check_review_transition_table "$GOVERNANCE_TRANSITION_TABLE" "项目规则、技能、钩子、门禁、判定库服务、统一任务服务、脚本这几类新扩进来的被管文件（定义与共用约束已在管，不进这张表）" "${forms[@]}" || return 1
  load_transition_rows "$GOVERNANCE_TRANSITION_TABLE"
  local objects verdicts definition class why verdict_count class_text=""
  local -a exempted=() missing=() stale=() unhashed=() transitioned=()
  local -A class_counts=()
  local count=0
  objects="$(governance_objects)"
  if [[ -z "$objects" ]]; then
    echo "  ! 这次改动没碰被管的治理文件（.claude/agents/、.claude/agent-common.md、.claude/main-agent.md、.claude/rules/、.claude/skills/、.claude/hooks/ 的钩子与库、.claude/gate.d/ 顶层的门禁与 lib/、scripts/ 下的脚本、task-server/、scripts/prune-server/、prune-server/），这一格无对象可判（基准 $gate_base）"
    return 77
  fi
  load_added_since_gate || {
    echo "  ✗ 取不到这次改动新增了哪些路径（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态（基准要存在、索引没坏）再跑；取不到新增的判决文件时这一格什么都没比，不是通过。"
    return 1
  }
  verdicts="$(new_verdict_paths)"
  # 豁免表的格式与认法在 user_exemption_reason
  local exempt_table="$AGENT_DEF_EXEMPT_TABLE"
  while IFS=$'\t' read -r class definition; do
    [[ -z "$definition" ]] && continue
    count=$((count + 1))
    class_counts[$class]=$((${class_counts[$class]:-0} + 1))
    # 登记在过渡挂账表里的路径不要求点名；判法与 crates-adversarial-review 同一份：review_judge_named 判点名与快照，又改过的与没点名的先查豁免表
    if [[ -n "${transition_rows[$definition]+set}" ]]; then transitioned+=("$definition：${transition_rows[$definition]}"); continue; fi
    review_judge_named "$definition" "$verdicts"
    case "$review_outcome" in
      named) continue ;;
      unhashed) unhashed+=("$definition"); continue ;;
    esac
    why="$(user_exemption_reason "$exempt_table" "$definition")"
    if [[ -n "$why" ]]; then exempted+=("$definition：$why"); continue; fi
    if [[ "$review_outcome" == stale ]]; then stale+=("$definition（$review_outcome_note）"); continue; fi
    missing+=("$definition$(review_mention_note "$definition" "$verdicts")")
  done <<<"$objects"
  if ((${#stale[@]})); then
    review_report_stale "改过的治理文件" "$exempt_table" "${stale[@]}"
  fi
  if ((${#missing[@]})); then
    echo "  ✗ 改过的治理文件里 ${#missing[@]} 份没有三方判决点名（这次改动新写的 research/prompts/*-main-verification.md 里一份都没在「$REVIEWED_SECTION_HEADING」一节里按路径列出它；正文别处提到不算，节里写了没攻、没判这类否定词的行也不算；被改过的旧判决不算）："   # gate-lint:summary
    printf '      %s\n' "${missing[@]}"   # gate-lint:detail
    echo "     → 怎么办：走一轮三方（.claude/rules/three-way-inference.md），判决落 research/prompts/<轮>-main-verification.md，"
    echo "               在判决里写一节，标题一行逐字是「$REVIEWED_SECTION_HEADING」（二级标题），节里一行一个，逐个按路径列出改过的每一份治理文件"
    echo "               （定义与共用约束、规则、技能、钩子、门禁、scripts/ 下的脚本、两个服务；节到下一个一、二级标题为止；打中的写回文件再改一轮）："
    echo "                 $REVIEWED_SECTION_HEADING"
    echo "                 - .claude/agents/<名>.md：<这一轮判了它的什么>"
    echo "               点名写进新的判决，不往旧判决里补；豁免表 $exempt_table 不再加行（表里已有的行照旧生效）；"
    echo "               存量没补完三方的登记在挂账表 $GOVERNANCE_TRANSITION_TABLE（只缩不涨，新改的文件不往里加）。"
    return 1
  fi
  ((${#stale[@]})) && return 1
  verdict_count="$(grep -c . <<<"$verdicts" || true)"
  for class in "${GOVERNANCE_CLASS_NAMES[@]}"; do
    [[ -n "${class_counts[$class]+set}" ]] && class_text+="${class_text:+、}${class} ${class_counts[$class]} 份"
  done
  echo "  ✓ 改过的治理文件 ${count} 份都有去向（${class_text}；新写的判决文件 ${verdict_count} 份，另有 ${#exempted[@]} 份按 $exempt_table 豁免，另有 ${#transitioned[@]} 份按 $GOVERNANCE_TRANSITION_TABLE 挂账、不要求点名，基准 $gate_base）"
  if ((${#exempted[@]})); then
    echo "    豁免的 ${#exempted[@]} 份（按内容哈希认，那份文件再改一个字就不再豁免）："
    printf '      %s\n' "${exempted[@]}"
  fi
  if ((${#transitioned[@]})); then
    echo "    挂账的 ${#transitioned[@]} 份（登记的路径这一格不要求点名；只缩不涨，补完一轮三方删一行）："
    printf '      %s\n' "${transitioned[@]}"
  fi
  review_report_snapshot_coverage "那一轮没有开工快照、或快照清单里没有它的行" ${unhashed[@]+"${unhashed[@]}"}
  report_review_pending_in_use "$verdicts"
}

# ── milestone-close-owed ────────────────────────────────────────────────────
# 判据在文件头「milestone-close-owed」；判法与弄坏开关都在 $MILESTONE_CLOSE_CHECK，这里只调它的 --closed、照它的退出码（0 绿、1 红、77 无对象、2 用法错按红）
check_milestone_close_owed() {
  if [[ ! -f "$MILESTONE_CLOSE_CHECK" ]]; then
    echo "  ✗ 判法 $MILESTONE_CLOSE_CHECK 不在，判不了收口的里程碑还有没有点名它的开着的账"
    echo "     → 怎么办：那份脚本随仓走，从 git 找回再跑；判法只在那一份，别在这一格里另抄"
    return 1
  fi
  python3 "$MILESTONE_CLOSE_CHECK" --closed "$ROOT"
}

# ── owed-destination-form ───────────────────────────────────────────────────
# 判据在文件头「owed-destination-form」；判法与弄坏开关都在 $MILESTONE_CLOSE_CHECK（与 milestone-close-owed 同一份），这里只调它的 --destination-form、照它的退出码（0 绿、1 红、77 无对象、2 用法错按红）
check_owed_destination_form() {
  if [[ ! -f "$MILESTONE_CLOSE_CHECK" ]]; then
    echo "  ✗ 判法 $MILESTONE_CLOSE_CHECK 不在，判不了欠账表的去向写法"
    echo "     → 怎么办：那份脚本随仓走，从 git 找回再跑；判法只在那一份，别在这一格里另抄"
    return 1
  fi
  python3 "$MILESTONE_CLOSE_CHECK" --destination-form "$ROOT"
}

# ── deferred-items-registered ───────────────────────────────────────────────
# 判据在文件头「deferred-items-registered」；词表是上面的 DEFERRAL_WORDS，弄坏开关 DEFERRED_ITEMS_BREAK 写在文件头那一段
check_deferred_items_registered() {
  ((inside_git)) || { not_git_repository; return; }
  report_ledger_registry_problems || return 1
  load_added_records_directory_lines || {
    echo "  ✗ 取不到 records/ 下这一轮新增了哪些行（基准 $gate_base）"
    echo "     → 怎么办：按上面 git 的报错修好仓库状态再跑；取不到新增行时延后项登没登记没法判，不是通过。"
    return 1
  }
  # 新增行经进程替换当文件传：当成一个命令行参数传时，单个参数超过 128 KiB 就起不来（Linux 的 MAX_ARG_STRLEN）。
  # 封存的几份照登记表的次序一行一份传进去
  python3 - <(printf '%s\n' "$added_records_directory_lines") "$DEFERRAL_WORDS" "${DEFERRED_ITEMS_BREAK:-}" "$DEFERRAL_REGISTRY_TABLE" "$OWED_LIBRARY" \
    "$current_ledger" "$(printf '%s\n' ${sealed_ledgers[@]+"${sealed_ledgers[@]}"})" <<'PY'
import importlib.util, os, re, sys

added_file, words_text, broken, REGISTRY_TABLE, OWED_LIBRARY = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]
CURRENT_LEDGER = sys.argv[6]
SEALED_LEDGERS = [path for path in sys.argv[7].split("\n") if path]
sys.dont_write_bytecode = True
DEFERRAL_WORDS = [word for word in words_text.split() if word]
CHECKS_OWED = ".claude/kb/checks-owed.md"
# 登记表里的几份拆分提案：current 那一份旧账本（开着的节）先找，没有再照登记表的次序找封存的（一字不改，做完的节照旧指得到）；几份都是登记处
ALL_LEDGERS = [CURRENT_LEDGER] + SEALED_LEDGERS
ALL_LEDGERS_TEXT = "、".join(ALL_LEDGERS)
# 「延后项」是在说「延后的那几件」这个话题（「延后项无去向 8/54」），不是在延后什么；「进延后项」「记进延后项」是把这一件延后，照旧算
DEFERRAL_NOUN = re.compile(r"延后项" if "into-noun-skipped" in broken else r"(?<!进)延后项")
HEADING = re.compile(r"^ {0,3}#{1,6}(?:[ \t]|$)")
# 同一个分句里带「N/M」的「延后」是在报统计数（「延后当时就登进欠账表的 6/54」），也不是在延后什么；分句按逗号、句号、分号、冒号与表格竖线切
CLAUSE_BREAK = re.compile(r"([，。；;：:|])")
RATIO = re.compile(r"[0-9]+\s*/\s*[0-9]+")


def without_topic_mentions(text):
    """去掉只在点话题的「延后」：名词「延后项」，与和统计数同在一个分句里的「延后」。"""
    if "statistics-counted" not in broken:
        text = "".join(piece.replace("延后", "") if RATIO.search(piece) else piece for piece in CLAUSE_BREAK.split(text))
    return text if "deferral-noun-counted" in broken else DEFERRAL_NOUN.sub("", text)


CIRCLED = "①②③④⑤⑥⑦⑧⑨⑩⑪⑫⑬⑭⑮⑯⑰⑱⑲⑳㉑㉒㉓㉔㉕㉖㉗㉘㉙㉚㉛㉜㉝㉞㉟㊱㊲㊳㊴㊵㊶㊷㊸㊹㊺㊻㊼㊽㊾㊿"
CHINESE_DIGITS = {"零": 0, "〇": 0, "一": 1, "二": 2, "两": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
C_NUMBER = re.compile(r"(?<![A-Za-z0-9_])C([0-9]+)(?![0-9])")
SECTION_ITEMS = re.compile(rf"第\s*(?P<section>[零〇一二两三四五六七八九十百]+|[0-9]+)\s*节(?:[^{CIRCLED}\n。；]{{0,16}}?(?P<items>[{CIRCLED}](?:\s*[、，,和与及]?\s*[{CIRCLED}])*))?")
# 行里没写「拆分提案」时只认节号后面紧跟行首序号的写法（「九十节 ㉟」「第九十节（行首序号 ②、③）」），别的文档的「第三节 … ①」不当成它
SECTION_ITEMS_BARE = re.compile(rf"(?<![零〇一二两三四五六七八九十百0-9])第?\s*(?P<section>[零〇一二两三四五六七八九十百]+|[0-9]+)\s*节\s*(?:[（(]\s*)?(?:行首序号\s*)?(?P<items>[{CIRCLED}](?:\s*[、，,和与及]?\s*[{CIRCLED}])*)")
# 「拆分提案」之后到它的节号之间许隔的：路径尾巴 .md 与 :行号、收尾的反引号、「（2026-10-23 份）」这类括注、空白；同一行别处的「第 N 节」说的是别的文件
SPLIT_PROPOSAL_ANCHOR = re.compile(r"拆分提案(?:\.md(?::[0-9]+)?)?`?(?:\s*[（(][^（()）\n]{0,40}[）)])*\s*")
FILE_LINE = re.compile(r"(?P<path>(?:\.{0,2}/)?[\w.\-一-鿿]+(?:/[\w.\-一-鿿]+)*\.[A-Za-z0-9]+):(?P<line>[0-9]+)(?:[-–](?P<end>[0-9]+))?")
FENCE = re.compile(r"^\s*(```|~~~)")


def chinese_number(text):
    if text.isdigit():
        return int(text)
    total, current = 0, 0
    for character in text:
        if character == "百":
            total, current = total + (current or 1) * 100, 0
        elif character == "十":
            total, current = total + (current or 1) * 10, 0
        else:
            current = CHINESE_DIGITS[character]
    return total + current


def outside_quotes(text):
    """去掉「…」（配平）与 `…` 里的字：引原话、引规则里的词不算这一行在说延后。"""
    kept, depth, in_code = [], 0, False
    for character in text:
        if character == "`" and depth == 0:
            in_code = not in_code
            continue
        if in_code:
            continue
        if character == "「":
            depth += 1
            continue
        if character == "」" and depth:
            depth -= 1
            continue
        if depth == 0:
            kept.append(character)
    return "".join(kept)


def read_lines(path):
    try:
        return open(path, encoding="utf-8", errors="replace").read().split("\n")
    except OSError:
        return None


owed_lines = read_lines(CHECKS_OWED)
registered_c, paid_c = set(), set()
if owed_lines is not None:
    # 开着与已还清的切法只在 lib-owed.py 一份（「已还清」整行标题之前开着、之后到「## 历史版本」已还清）；认不出那一行标题时全文的行都当开着的（那份库的 open_names）
    owed_specification = importlib.util.spec_from_file_location("lib_owed", OWED_LIBRARY)
    owed_library = importlib.util.module_from_spec(owed_specification)
    owed_specification.loader.exec_module(owed_library)
    owed_table = owed_library.read_owed_table(CHECKS_OWED)
    registered_c = {int(name[1:]) for name in owed_table.open_names}
    paid_c = {int(name[1:]) for name in owed_table.paid_names}
    if "paid-c-numbers-accepted" in broken:
        registered_c |= paid_c
def section_items(lines):
    """→ {节号: {行首序号}}：「## <节号>、」标题起一节，节里行首是「| ③ |」的行；同一个节号出现两次的并成一节。"""
    items = {}
    section = None
    for row in lines or ():
        heading = re.match(r"^##\s+([零〇一二两三四五六七八九十百]+)、", row)
        if heading:
            section = chinese_number(heading.group(1))
            items.setdefault(section, set())
            continue
        if row.startswith("## "):
            section = None
            continue
        item = re.match(rf"^\|\s*([{CIRCLED}])\s*\|", row)
        if item and section is not None:
            items[section].add(item.group(1))
    return items


# 封存的几份照登记表的次序读；sealed-chain-cut 只读头一份
sealed_lines = {path: read_lines(path) for path in SEALED_LEDGERS[:1 if "sealed-chain-cut" in broken else None]}
sealed_items = [section_items(lines) for lines in sealed_lines.values()]
# 现行账本读不到就照实报（不退回只认封存的）；ledger-ignored 不读它，ledger-missing-tolerated 读不到当空的、不报
ledger_lines = [] if "ledger-ignored" in broken else read_lines(CURRENT_LEDGER)
if ledger_lines is None and "ledger-missing-tolerated" in broken:
    ledger_lines = []
ledger_items = section_items(ledger_lines)
# 登记处表：一份一行「<从仓根起的路径><TAB><为什么它本身就是登记处>」；路径要在树里、理由 8 个字以上，写坏的那一行不认、逐行报
registries, registry_problems = [], []
registry_rows = read_lines(REGISTRY_TABLE) if os.path.isfile(REGISTRY_TABLE) else []
for index, row in enumerate(registry_rows or [], start=1):
    if not row.strip() or row.startswith("#"):
        continue
    fields = row.split("\t")
    if "registry-rows-unchecked" in broken:
        registries.append(fields[0].strip())
        continue
    if len(fields) != 2:
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：不是「路径<TAB>理由」两段（读出 {len(fields)} 段）")
    elif not os.path.isfile(fields[0]):
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：{fields[0]} 不在树里（写从仓根起的路径）")
    elif len(fields[1].strip()) < 8:
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：{fields[0]} 的理由不到 8 个字（写为什么这份文件本身就是登记处）")
    elif fields[0] in registries:
        registry_problems.append(f"{REGISTRY_TABLE}:{index}：{fields[0]} 登记了两次")
    else:
        registries.append(fields[0])


def registry_named(text):
    """表格行里，带延后类词的那一格之后的某一格点名的登记处（去向格）；前后不接路径里的字，`staged-gate.md.old` 不算 staged-gate.md。
    不是表格行、或登记处只在延后类词那一格与它之前（被延后的那件事本身）的，不算。"""
    if "registry-table-ignored" in broken:
        return []
    cells = text.strip().strip("|").split("|") if text.lstrip().startswith("|") else [text]
    if "registry-cell-unchecked" in broken:
        later = text
    elif len(cells) < 2:
        return []
    else:
        word_cell = next((index for index, cell in enumerate(cells)
                          if any(word in without_topic_mentions(outside_quotes(cell)) for word in DEFERRAL_WORDS)), None)
        if word_cell is None:
            return []
        later = "|".join(cells[word_cell + 1:])
    return [path for path in registries
            if re.search(rf"(?<![\w./\-一-鿿]){re.escape(path)}(?![\w./\-一-鿿])", later)]


def section_matches(text):
    """→ 这一行里当拆分提案节号认的命中，按出现次序（文件头判据 ②）：紧跟「拆分提案」的那一个照 SECTION_ITEMS 认，别处照 SECTION_ITEMS_BARE 认。"""
    if "拆分提案" not in text:
        return list(SECTION_ITEMS_BARE.finditer(text))
    if "split-proposal-section-anywhere" in broken:
        return list(SECTION_ITEMS.finditer(text))
    anchored = [match for match in (SECTION_ITEMS.match(text, anchor.end()) for anchor in SPLIT_PROPOSAL_ANCHOR.finditer(text)) if match]
    spans = [match.span() for match in anchored]
    elsewhere = [match for match in SECTION_ITEMS_BARE.finditer(text)
                 if not any(start < match.end() and match.start() < end for start, end in spans)]
    return sorted(anchored + elsewhere, key=lambda match: match.start())


def pointers(text, own_path, own_number):
    """→ [(写法, 指得到吗, 原文, 指不到的原因)]；own_path、own_number 是这一行自己（行号找不到时 own_number 是 None）。"""
    found = []
    for match in C_NUMBER.finditer(text):
        number = int(match.group(1))
        ok = owed_lines is not None and number in registered_c
        if ok:
            why = ""
        elif number in paid_c:
            why = f"C{number} 在 {CHECKS_OWED} 的「已还清」表里，已还清的不算登记（延后的事另登一行开着的）"
        else:
            why = f"{CHECKS_OWED} 里没有「| C{number} |」这一行"
        found.append(("C 号", ok, match.group(0), why))
    for match in section_matches(text):
        section = chinese_number(match.group("section"))
        # 先找现行账本，没有再照登记表的次序找封存的：序号在几份同号的节里任一份有就算指得到（封存的里重号的节在现行账本里改了号）
        sources = (ledger_items,) if "history-fallback-dropped" in broken else (ledger_items, *sealed_items)
        present = [items_by_section[section] for items_by_section in sources if section in items_by_section]
        items = re.findall(f"[{CIRCLED}]", match.group("items") or "")
        if not present:
            found.append(("拆分提案节号", False, f"第{match.group('section')}节", f"{ALL_LEDGERS_TEXT} 里都没有第 {section} 节"))
        elif not items:
            rows = present[0]   # 只写节号的：看照现行、封存次序头一份有这一节的
            ok = not rows
            found.append(("拆分提案节号", ok, f"第{match.group('section')}节", "" if ok else f"那一节有行首序号（{len(rows)} 行），要写是哪一行"))
        for item in items if present else ():
            ok = any(item in rows for rows in present)
            found.append(("拆分提案节号与序号", ok, f"第{match.group('section')}节 {item}", "" if ok else f"{ALL_LEDGERS_TEXT} 的第 {section} 节里都没有行首是「{item}」的一行"))
    for match in FILE_LINE.finditer(text):
        path, number = match.group("path"), int(match.group("line"))
        last = int(match.group("end")) if match.group("end") else number
        target = os.path.normpath(path)
        # 指到这一行自己的不算登记（弄坏开关 self-pointer-accepted 下不判这一样）
        if (target == os.path.normpath(own_path) and own_number is not None and number <= own_number <= max(number, last)
                and "self-pointer-accepted" not in broken):
            found.append(("文件:行", False, match.group(0), "指的是这一行自己"))
            continue
        # records/ 下只认登记表里的拆分提案：调度记录与别的记录是写延后的地方，不是登记处（弄坏开关 records-pointers-accepted 下不判这一样）
        if (target.startswith("records/") and target not in ALL_LEDGERS
                and "records-pointers-accepted" not in broken):
            found.append(("文件:行", False, match.group(0), f"records/ 下只认登记表里的拆分提案（{ALL_LEDGERS_TEXT}），别的记录不是登记处"))
            continue
        lines = read_lines(path) if os.path.isfile(path) else None
        ok = lines is not None and 1 <= number <= len(lines) - (1 if lines and lines[-1] == "" else 0)
        why = "" if ok else (f"{path} 不在树里（只写文件名的要写从仓根起的路径）" if lines is None else f"{path} 只有 {len(lines) - (1 if lines[-1] == '' else 0)} 行")
        found.append(("文件:行", ok, match.group(0), why))
    for path in registry_named(text):
        found.append(("登记处", True, path, ""))
    return found


with open(added_file, encoding="utf-8", errors="replace") as handle:
    added = [line.rstrip("\n").split("\t", 1) for line in handle if "\t" in line]
file_cache, used, judged, problems, scanned = {}, {}, [], [], 0
skipped_quoted = skipped_fenced = skipped_registry = skipped_noun = skipped_heading = 0
for path, text in added:
    if not path.endswith(".md"):
        continue
    scanned += 1
    visible = text if "quotes-counted" in broken else outside_quotes(text)
    if "noun-line-skipped" in broken and DEFERRAL_NOUN.search(visible):
        skipped_noun += 1
        continue
    verbs = without_topic_mentions(visible)
    words = [word for word in DEFERRAL_WORDS if word in verbs]
    if not words:
        if any(word in visible for word in DEFERRAL_WORDS):
            skipped_noun += 1
        elif any(word in text for word in DEFERRAL_WORDS):
            skipped_quoted += 1
        continue
    if ((path in SEALED_LEDGERS and "registry-rows-judged" not in broken)
            or (path == CURRENT_LEDGER and "ledger-rows-judged" not in broken)):
        skipped_registry += 1   # 登记表里的拆分提案本身是登记处（编号小节），它们里面的行不判
        continue
    lines = file_cache.setdefault(path, read_lines(path) or [])
    start = used.get((path, text), 0)
    number = next((index + 1 for index in range(start, len(lines)) if lines[index] == text), None)
    used[(path, text)] = number if number else start
    if number is not None:
        in_fence = False
        for index in range(number - 1):
            if FENCE.match(lines[index]):
                in_fence = not in_fence
        if in_fence and "fenced-lines-judged" not in broken:
            skipped_fenced += 1
            continue
    if HEADING.match(text) and "headings-judged" not in broken:
        skipped_heading += 1   # 标题是在点话题（「### 延后项的登记」），延后什么写在节里的行上
        continue
    where = f"{path}:{number}" if number else f"{path}（行号找不到）"
    judged.append(where)
    found = pointers(text, path, number)
    if any(ok or "pointers-unresolved-accepted" in broken for _kind, ok, _text, _why in found):
        continue
    if found:
        detail = "；".join(f"{kind}「{written}」指不到：{why}" for kind, _ok, written, why in found)
        problems.append(f"{where}：带「{'」「'.join(words)}」，写的登记指不到——{detail}")
    else:
        problems.append(f"{where}：带「{'」「'.join(words)}」，同一行没写 C 号、拆分提案的节号与行首序号、文件:行或登记处")
missing = [path for path, lines in ((CHECKS_OWED, owed_lines), (CURRENT_LEDGER, ledger_lines), *sealed_lines.items()) if lines is None]
if missing:
    print(f"  ✗ 读不到 {'、'.join(missing)}：C 号与拆分提案的节号指不指得到判不了")
    print("     → 怎么办：从 git 找回那份文件（git log --diff-filter=D --name-only -- <路径>），修好再单跑这一格（bash .claude/gate.d/doc-process-records.sh --check deferred-items-registered）")
    sys.exit(1)
if registry_problems:
    for problem in registry_problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ 登记处表 {REGISTRY_TABLE} 里 {len(registry_problems)} 行写坏了，这几行不当登记处认")
    print(f"     → 怎么办：照 {REGISTRY_TABLE} 文件头写成「从仓根起的路径<TAB>为什么它本身就是登记处（8 个字以上）」，一份一行、文件要在树里；不再是登记处的整行删掉")
skipped_counts = (skipped_quoted, skipped_fenced, skipped_registry, skipped_noun, skipped_heading)
skipped_text = (f"；不判的：词只在「」或反引号里的 {skipped_quoted} 行、围栏里的 {skipped_fenced} 行、拆分提案（登记处本身）里的 {skipped_registry} 行、"
                f"「延后」只在点话题的（名词「延后项」、统计数）{skipped_noun} 行、标题 {skipped_heading} 行"
                if any(skipped_counts) else "")
if not judged:
    print(f"  ! 这次改动里 records/ 下新增的行（{scanned} 行）没有带延后类词（{'、'.join(DEFERRAL_WORDS)}）的，这一格无对象可判{skipped_text}")
    sys.exit(1 if registry_problems else 77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    print(f"  ✗ records/ 下这次新增的带延后类词的行里 {len(problems)} 行没登记去向（判了 {len(judged)} 行）")
    print("     → 怎么办：照 .claude/main-agent.md「一轮怎么开、怎么收」第 2 条在同一行写登记的编号或 文件:行"
          f"（.claude/kb/checks-owed.md 开着的 C 号、拆分提案的「第<节号>节 <行首序号>」（旧账本 {CURRENT_LEDGER} 与封存的 {'、'.join(SEALED_LEDGERS) or '无'} 里已有的节照旧认，都不往里登）、"
          "或从仓根起的 <路径>:<行号>（records/ 下只认登记表里的拆分提案，不指这一行自己）），先在那里登一行再写延后；subagent 与协作工具的事不延后、不登账，当场修（第 4 条）；"
          f"去向是提交时那一套流程的，同一行点名 {REGISTRY_TABLE} 里登记的那份文件（写全路径）")
    sys.exit(1)
if registry_problems:
    sys.exit(1)
print(f"  ✓ records/ 下这次新增的带延后类词的行：判了 {len(judged)} 行，都写着指得到的登记（C 号、拆分提案的节号与序号、文件:行或登记处；records/ 新增的 .md 行共 {scanned} 行）{skipped_text}")
PY
}

# ── ledger-section-numbers ──────────────────────────────────────────────────
# 判据在文件头「ledger-section-numbers」；只看树，不用改动范围；判的是登记表里 current 那一份
check_ledger_section_numbers() {
  report_ledger_registry_problems || return 1
  python3 - "$current_ledger" "${LEDGER_SECTION_NUMBERS_BREAK:-}" <<'PY'
import collections, re, sys

path, broken = sys.argv[1], sys.argv[2]
try:
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
except FileNotFoundError:
    print(f"  ! 树里没有 {path}，这一格无对象可判")
    sys.exit(77)
except (OSError, UnicodeDecodeError) as error:
    print(f"  ✗ 读不了 {path}（{error}）：节号重不重、索引全不全这一次没判")
    print("     → 怎么办：修好那份文件的编码或权限再单跑这一格（bash .claude/gate.d/doc-process-records.sh --check ledger-section-numbers）；读不了不是没有重号。")
    sys.exit(1)

NUMERALS = "零〇一二两三四五六七八九十百千"
HEADING = re.compile(rf"^##\s+([{NUMERALS}]+)、")
INDEX_ROW = re.compile(rf"^\|\s*([{NUMERALS}]+)")   # 第一格以节号起头（「一百六十七（2026-10-23 那份里的第二个五十六）」只取「一百六十七」）；表头「节号」与分隔行不是
FENCE = re.compile(r"^ {0,3}(```|~~~)")
headings = collections.defaultdict(list)     # 节号 → 标题所在的行号
index_rows = collections.defaultdict(list)   # 节号 → 索引表里的行号
index_seen = in_index = in_fence = False
for number, line in enumerate(lines, 1):
    if FENCE.match(line):
        in_fence = not in_fence
        continue
    if in_fence:
        continue
    if line.startswith("## "):
        in_index = line.rstrip() == "## 索引"
        index_seen = index_seen or in_index
        found = HEADING.match(line)
        if found:
            headings[found.group(1)].append(number)
        continue
    if in_index:
        found = INDEX_ROW.match(line)
        if found:
            index_rows[found.group(1)].append(number)


def at(numbers):
    return "第 " + "、".join(map(str, numbers)) + " 行"


problems = []
if not index_seen:
    problems.append("没有「## 索引」一节（文件开头那张一节一行的表）")
for section, numbers in headings.items():
    if len(numbers) != 1 and "duplicate-headings-tolerated" not in broken:
        problems.append(f"「## {section}、」标题重号 {len(numbers)} 处（{at(numbers)}）")
    if index_seen and "index-unchecked" not in broken:
        rows = index_rows.get(section, [])
        if not rows:
            problems.append(f"节号 {section}（标题在{at(numbers)}）在索引表里没有一行")
        elif len(rows) != 1:
            problems.append(f"节号 {section} 在索引表里有 {len(rows)} 行（{at(rows)}），该是 1 行")
if "index-unchecked" not in broken:
    for section, rows in index_rows.items():
        if section not in headings:
            problems.append(f"索引表{at(rows)}的节号 {section} 没有对应的「## {section}、」一节")

if problems:
    for problem in problems:
        print(f"  ✗ {path}：{problem}")   # gate-lint:detail
    print(f"  ✗ {path} 的节号与索引有 {len(problems)} 处对不上（{len(headings)} 个节号）")
    print("     → 怎么办：新登一节用 python3 scripts/kb/ledger-append-section.py --title '<标题（含日期）>' --body-file <正文文件>（现取最大节号加一，索引行与节一次写入），不手写新节的标题；撞了号的改成下一个空号、索引表同步改；"
          "索引里少的补一行、多的删掉；没有例外，撞了的一律改号")
    sys.exit(1)
heading_count = sum(len(numbers) for numbers in headings.values())
row_count = sum(len(rows) for rows in index_rows.values())
print(f"  ✓ {path}：{len(headings)} 个节号（标题 {heading_count} 处、索引表 {row_count} 行）不重号、各在索引表里一行")
PY
}

# ── sealed-ledgers-unchanged ────────────────────────────────────────────────
# 判据在文件头「sealed-ledgers-unchanged」；只看树与登记表，不用改动范围
check_sealed_ledgers_unchanged() {
  report_ledger_registry_problems || return 1
  if ((${#sealed_ledgers[@]} == 0)); then
    echo "  ! 登记表 $LEDGER_REGISTRY 里没有 sealed 行，这一格无对象可判"
    return 77
  fi
  local index ledger digest actual changed=0
  for index in "${!sealed_ledgers[@]}"; do
    ledger="${sealed_ledgers[$index]}"
    digest="${sealed_ledger_digests[$index]}"
    if [[ ! -f "$ledger" ]]; then
      echo "  ✗ 封存的 $ledger 不在树里（登记的 sha256 $digest）"   # gate-lint:detail
      changed=$((changed + 1))
      continue
    fi
    # 弄坏开关 DOC_PROCESS_RECORDS_BREAK=sealed-hash-unchecked：只看在不在，不比 sha256
    [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == sealed-hash-unchecked ]] && continue
    # 从标准输入读：文件名带反斜杠时 sha256sum 会在输出行首加一个反斜杠
    actual="$(sha256sum < "$ledger")"
    actual="${actual%% *}"
    if [[ "$actual" != "$digest" ]]; then
      echo "  ✗ 封存的 $ledger 封存之后被改了：此刻 sha256 $actual，登记的是 $digest"   # gate-lint:detail
      changed=$((changed + 1))
    fi
  done
  if ((changed)); then
    echo "  ✗ 封存的 ${#sealed_ledgers[@]} 份拆分提案里 $changed 份封存之后被改了或不在了"
    echo "     → 怎么办：封存账本不改，新内容不往账本里登：把上面点名的那份恢复成封存时那一版（git log -- <路径> 找到内容的 sha256 与登记的相同的提交，git show <提交>:<路径> > <路径>），"
    echo "        要记的新内容：subagent 与协作工具的问题当场修（.claude/main-agent.md「一轮怎么开、怎么收」第 4 条），别的延后项登 .claude/kb/checks-owed.md（第 2 条）；用户定了重新封存的，才改 $LEDGER_REGISTRY 里那一行的 sha256"
    return 1
  fi
  echo "  ✓ 封存的 ${#sealed_ledgers[@]} 份拆分提案（${sealed_ledgers[*]}）内容都与登记的 sha256 相同"
}

# ── code-round-checklist ───────────────────────────────────────────────────
# 判据在文件头「code-round-checklist」；只看树，不用改动范围
# 这一份正文要不要判第九格：不在只判八格的名单里（第三个参数起是名单里的路径），而标题日期不早于第九格的截止日、或标题没有日期
code_round_checklist_wants_placement() { # <正文> <标题里的日期，没有就是空> <名单里的路径…>
  local body="$1" date="$2" listed
  shift 2
  # 弄坏开关 DOC_PROCESS_RECORDS_BREAK=code-round-checklist-placement-unchecked：一份都不判；code-round-checklist-placement-everywhere：每份都判
  [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-placement-unchecked ]] && return 1
  [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-placement-everywhere ]] && return 0
  # 弄坏开关 DOC_PROCESS_RECORDS_BREAK=code-round-checklist-eight-cell-list-ignored：不看名单
  if [[ "${DOC_PROCESS_RECORDS_BREAK:-}" != code-round-checklist-eight-cell-list-ignored ]]; then
    for listed in "$@"; do
      [[ "$body" == "$listed" ]] && return 1
    done
  fi
  [[ -z "$date" || ! "$date" < "$CODE_ROUND_CHECKLIST_PLACEMENT_CUTOFF" ]]
}

check_code_round_checklist() {
  local prompts_directory="research/prompts"
  local body heading date label state checked=0 placement_checked=0 line number=0 listed_path growth_output growth_kind growth_text
  local growth_note="只缩不涨没比（这一格没判）"
  local row_pattern=$'^([^\t#]+)\t+#[[:space:]]*[^[:space:]]' comment_pattern='^[[:space:]]*#'
  local -a bodies=("$prompts_directory"/_*-code-r[0-9]*-body.md)
  local -a before_cutoff=() undated=() problems=() labels=() eight_cell_list=() eight_cell_rows=() list_problems=()
  # 只判八格的名单：一行「<路径><TAB># 理由>」；写不成这样的行、指的正文不在了的行判红（弄坏开关 code-round-checklist-eight-cell-list-stale-accepted 下不报）
  if [[ -f "$CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE" ]]; then
    while IFS= read -r line || [[ -n "$line" ]]; do
      number=$((number + 1))
      [[ -z "${line//[[:space:]]/}" || "$line" =~ $comment_pattern ]] && continue
      if [[ ! "$line" =~ $row_pattern ]]; then
        list_problems+=("$CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE 第 $number 行写不成「<正文路径><TAB># 理由>」：${line:0:160}")
        continue
      fi
      listed_path="${BASH_REMATCH[1]}"
      listed_path="${listed_path%"${listed_path##*[![:space:]]}"}"
      eight_cell_list+=("$listed_path")
      eight_cell_rows+=("$number"$'\t'"$listed_path")
      [[ -f "$listed_path" ]] || list_problems+=("$CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE 第 $number 行的 $listed_path 不在了：正文归档、改名之后删掉这一行（名单只缩不长）")
    done < "$CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE"
  fi
  if [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-eight-cell-list-stale-accepted || "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-unjudged ]]; then
    list_problems=()
  fi
  # 名单只缩不涨：与 gate 基准那一版名单比（照 crate-roles 滞后表那一套，共用 lib-lag-baseline.py 的 baseline_rows），多出的路径判红；
  # 基准里还没有这张表、被判的根不在 git 仓里不比（成功句写明），git 取不到基准那一版判红（弄坏开关 code-round-checklist-eight-cell-list-growth-accepted 下不比）
  if [[ -f "$CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE" && "${DOC_PROCESS_RECORDS_BREAK:-}" != code-round-checklist-unjudged ]]; then
    if [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-eight-cell-list-growth-accepted ]]; then
      growth_note="只缩不涨没比（弄坏开关 code-round-checklist-eight-cell-list-growth-accepted）"
    elif growth_output="$(python3 - "$CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE" "$gate_base" "$inside_git" "$LAG_BASELINE_LIBRARY" ${eight_cell_rows[@]+"${eight_cell_rows[@]}"} <<'PY'
import importlib.util, sys

table, base, inside_git, library = sys.argv[1], sys.argv[2], sys.argv[3] == "1", sys.argv[4]
listed = [row.split("\t", 1) for row in sys.argv[5:]]
lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", library)
lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
lag_baseline_spec.loader.exec_module(lag_baseline)
if not inside_git or not base:
    print("note\t只缩不涨没比（被判的根不在 git 仓里）")
    sys.exit(0)
baseline_rows, baseline_problem = lag_baseline.baseline_rows(base, table)
if baseline_problem is not None:
    print(f"problem\t{baseline_problem.sentence()}，名单只缩不涨没法比")
    sys.exit(0)
if baseline_rows is None:
    print(f"note\t只缩不涨没比（基准 {base} 里还没有这张表）")
    sys.exit(0)
base_paths = {fields[0].strip() for _number, fields in baseline_rows}
grown = [(number, path) for number, path in listed if path not in base_paths]
for number, path in grown:
    print(f"problem\t{table} 第 {number} 行的 {path} 是基准 {base} 之后新加的：名单只缩不长，新写的代码轮正文照判九格")
if not grown:
    print(f"note\t与基准 {base} 那一版比没涨（基准 {len(base_paths)} 行）")
PY
)"; then
      while IFS=$'\t' read -r growth_kind growth_text; do
        case "$growth_kind" in
          problem) list_problems+=("$growth_text") ;;
          note) growth_note="$growth_text" ;;
        esac
      done <<<"$growth_output"
    else
      list_problems+=("拿名单与基准那一版比的那段 python 退了非 0（原话在上面），只缩不长没比")
    fi
  fi
  for body in "${bodies[@]}"; do
    heading="$(head -n 1 "$body")"
    date="$(grep -oE '[0-9]{4}-[0-9]{2}-[0-9]{2}' <<<"$heading" | head -n 1)"
    if [[ -z "$date" ]]; then
      undated+=("$body")
    elif [[ "$date" < "$CODE_ROUND_CHECKLIST_CUTOFF" ]]; then
      before_cutoff+=("$body（$date）")
      continue
    fi
    checked=$((checked + 1))
    # 弄坏开关 DOC_PROCESS_RECORDS_BREAK=code-round-checklist-unjudged：不判
    [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-unjudged ]] && continue
    labels=("${CODE_ROUND_CHECKLIST_CELLS[@]}")
    if code_round_checklist_wants_placement "$body" "$date" ${eight_cell_list[@]+"${eight_cell_list[@]}"}; then
      labels+=("$CODE_ROUND_CHECKLIST_PLACEMENT_CELL")
      placement_checked=$((placement_checked + 1))
    fi
    for label in "${labels[@]}"; do
      state="$(awk -v heading="$CODE_ROUND_CHECKLIST_HEADING_PREFIX$label" '
        { line = $0; sub(/[ \t]+$/, "", line) }
        line == heading { found = 1; next }
        found && /^#/ { exit }
        found && NF { filled = 1; exit }
        END { print (found ? (filled ? "filled" : "empty") : "missing") }' "$body")"
      case "$state" in
        missing) problems+=("$body：没有「$CODE_ROUND_CHECKLIST_HEADING_PREFIX$label」这一格") ;;
        # 弄坏开关 DOC_PROCESS_RECORDS_BREAK=code-round-checklist-answers-unchecked：只看标题在不在
        empty) [[ "${DOC_PROCESS_RECORDS_BREAK:-}" == code-round-checklist-answers-unchecked ]] \
                 || problems+=("$body：「$CODE_ROUND_CHECKLIST_HEADING_PREFIX$label」底下没写答案（到下一个标题为止一行字都没有）") ;;
      esac
    done
  done
  if ((${#bodies[@]} == 0)); then
    echo "  ! research/prompts/ 下没有 _<题>-code-r<数字>-body.md，这一格无对象可判"
    return 77
  fi
  if ((checked == 0)); then
    echo "  ! 代码轮正文 ${#bodies[@]} 份的标题日期都早于 $CODE_ROUND_CHECKLIST_CUTOFF（规范落地之前写的，不回改），这一格无对象可判：${before_cutoff[*]}"
    return 77
  fi
  if ((${#list_problems[@]})); then
    echo "  ✗ 只判八格的名单 $CODE_ROUND_CHECKLIST_EIGHT_CELL_TABLE 有 ${#list_problems[@]} 行不对（名单 ${#eight_cell_list[@]} 行）："   # gate-lint:summary
    printf '     %s\n' "${list_problems[@]}"   # gate-lint:detail
    echo "     → 怎么办：删掉指的正文不在了的那一行与基准之后新加的那一行（名单只缩不长，不往里加新正文：新写的代码轮正文写全九格），写坏的行改成「<正文路径><TAB># 理由>」；取不到基准那一版的，照上面 git 的原话修好仓，再单跑 bash .claude/gate.d/doc-process-records.sh --check code-round-checklist"
  fi
  if ((${#problems[@]})); then
    echo "  ✗ 代码轮正文的检查单没写全（${#problems[@]} 处，查了 $checked 份）："   # gate-lint:summary
    printf '     %s\n' "${problems[@]}"   # gate-lint:detail
    echo "     → 怎么办：照 .claude/rules/implementation-workflow.md「代码轮正文的检查单」，每格一个「$CODE_ROUND_CHECKLIST_HEADING_PREFIX<格名>」小节，底下写答案与证据（文件:行）；九个格名逐字照那张表（标题日期早于 $CODE_ROUND_CHECKLIST_PLACEMENT_CUTOFF 的旧正文与只判八格的名单里的正文只判前八格）"
  fi
  ((${#problems[@]} + ${#list_problems[@]})) && return 1
  echo "  ✓ 代码轮正文的检查单都点名并写了答案（检查了 $checked 份：判九格的 $placement_checked 份、判八格的 $((checked - placement_checked)) 份；其中标题没有日期、照要查的算的 ${#undated[@]} 份；只判八格的名单 ${#eight_cell_list[@]} 行，指的正文都在）"
  echo "    只判八格的名单：$growth_note"
  if ((${#before_cutoff[@]})); then
    echo "    没查 ${#before_cutoff[@]} 份（标题日期早于 $CODE_ROUND_CHECKLIST_CUTOFF，规范落地之前写的）：${before_cutoff[*]}"
  else
    echo "    没查 0 份"
  fi
}
