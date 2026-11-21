#!/usr/bin/env bash
# 门禁 doc-decisions 的格函数库：candidate-codes、inference-needs-verdict、three-way-leg-record、owed-refs-paid。
# 由 .claude/gate.d/doc-decisions.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法段、表、常量）连同判据注释；读的 ROOT 与入口里设好的各变量、bad、ok、howto 在调用时才取。

# ════ 格「候选代号不跨分项复用」════
# 同一份决策里，同一个候选代号（甲乙丙丁戊己庚辛壬癸，或单个大写拉丁字母）不许出现在两个分项的候选表里：
# 检索到其中一条时分不出这个代号属于哪一格（C256：D19 已定项 7 的丙是「都不存」，已定项 8 的草案也有丙，指的是别的东西）。
# 扫 .claude/kb/decisions/*.md「## 历史版本」之前、围栏之外的行，按分项小节切块：`### / #### 已定项 N` 与 `未定项 N` 到下一个同级或更高级标题；
#   块外的行（分项索引、连锁一节）不扫。
# 候选表认两种形态：表格某一列整格是单字代号（去掉 ** 与反引号后），且这一列至少有两个不同的代号；
#   行首列表「- 甲：」「甲）」「（甲）」「- **乙**：」，相距不超过 6 行的一组里至少有两个不同的代号。判不出候选表的块不判。
# 同一个代号在两个不同分项块的候选表里都出现判红，点名文件、代号与两处行号；同一块里的几张表共用一个代号不算。
# 带了分项前缀的写法（「已定项 7 的丙」）不是表格整格也不在行首，本来就不算。看不见：「甲（采用）」这类带括注的格、正文里的「候选甲」「臂乙」。
# 弄坏开关 DOC_DECISIONS_BREAK=candidate-codes-unchecked：复用不判红。
# 样本：candidate-codes-red 两个分项的候选表都用丙（一张表格、一张列表），判红；candidate-codes-green 两个分项代号不同、另一处写「已定项 7 的丙」，判绿。
grid_candidate_codes() (
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格跳过"; exit 77; }

  python3 - "$BREAK_SWITCH" <<'PY'
import glob, re, sys

BREAK = sys.argv[1]
CHECK_COMMAND = 'bash .claude/gate.d/doc-decisions.sh --check candidate-codes'
DECISIONS = '.claude/kb/decisions'
FENCE = re.compile(r'^[ \t]*```')
HEADING = re.compile(r'^(#{1,6})[ \t]+(.*?)[ \t]*$')
ITEM_HEADING = re.compile(r'^(已定项|未定项)[ \t]*(\d+)')
OWN_HISTORY = re.compile(r'^## 历史版本[ \t]*$')
SEPARATOR_ROW = re.compile(r'^[\s:|\-]+$')
HEAVENLY = '甲乙丙丁戊己庚辛壬癸'
CODE_CELL = re.compile(rf'^[{HEAVENLY}A-Z]$')
LIST_CODE = re.compile(rf'^[ \t]*(?:[-*][ \t]+)?\**[（(]?([{HEAVENLY}A-Z])\**[）)：:]')
LIST_GAP = 6
UNESCAPED_PIPE = re.compile(r'(?<!\\)\|')

files = sorted(glob.glob(f'{DECISIONS}/*.md'))
if not files:
    print(f'  ! 找不到 {DECISIONS} 下的决策文件，这一格无对象可判')
    sys.exit(77)


def cell_text(raw):
    return re.sub(r'[*`\s]', '', raw)


def blocks_of(path):
    """[(块名, [(行号, 行)])]：分项小节到下一个同级或更高级标题；历史节之后和围栏里的行不算。"""
    blocks, current, fenced = [], None, False
    for number, line in enumerate(open(path, encoding='utf-8').read().splitlines(), 1):
        if OWN_HISTORY.match(line):
            break
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        heading = HEADING.match(line)
        if heading:
            level = len(heading.group(1))
            if current is not None and level <= current[0]:
                blocks.append(current[1:])
                current = None
            item = ITEM_HEADING.match(heading.group(2))
            if item and level in (3, 4) and current is None:
                current = (level, f'{item.group(1)} {item.group(2)}', [])
            continue
        if current is not None:
            current[2].append((number, line))
    if current is not None:
        blocks.append(current[1:])
    return blocks


def table_groups(lines):
    """候选表：表格某一列整格是单字代号、这一列至少两个不同的代号。[[(代号, 行号)]]，一张表的一列一组。"""
    found, table = [], []
    for number, line in lines + [(0, '')]:
        if line.lstrip().startswith('|'):
            table.append((number, line))
            continue
        if table:
            columns = {}
            for row_number, row in table:
                if SEPARATOR_ROW.match(row.strip()):
                    continue
                cells = [cell_text(c) for c in UNESCAPED_PIPE.split(row.strip().strip('|'))]
                for index, cell in enumerate(cells):
                    if CODE_CELL.match(cell):
                        columns.setdefault(index, []).append((cell, row_number))
            for hits in columns.values():
                if len({code for code, _ in hits}) >= 2:
                    found.append(hits)
            table = []
    return found


def list_groups(lines):
    """候选列表：行首「- 甲：」「甲）」这类，相距不超过 LIST_GAP 行的一组里至少两个不同的代号。[[(代号, 行号)]]。"""
    hits = []
    for number, line in lines:
        match = LIST_CODE.match(line)
        if match:
            hits.append((match.group(1), number))
    found, group = [], []
    for hit in hits + [None]:
        if hit is not None and (not group or hit[1] - group[-1][1] <= LIST_GAP):
            group.append(hit)
            continue
        if len({code for code, _ in group}) >= 2:
            found.append(group)
        group = [hit] if hit is not None else []
    return found


scanned, tables, reused, all_codes = 0, 0, [], set()
for path in files:
    scanned += 1
    where = {}
    for name, lines in blocks_of(path):
        groups = table_groups(lines) + list_groups(lines)
        tables += len(groups)
        first = {}
        for group in groups:
            for code, number in group:
                first.setdefault(code, number)
        for code, number in first.items():
            where.setdefault(code, []).append((name, number))
            all_codes.add(code)
    for code, places in sorted(where.items()):
        if len(places) >= 2:
            reused.append((path, code, places))

if reused and BREAK == 'candidate-codes-unchecked':
    print(f'  ✓ 弄坏开关 DOC_DECISIONS_BREAK=candidate-codes-unchecked：{len(reused)} 个复用的候选代号没判')
    sys.exit(0)
if reused:
    print(f'  ✗ {len(reused)} 个候选代号在同一份决策的两个分项候选表里复用')  # gate-lint:summary
    for path, code, places in reused:
        text = '、'.join(f'{name}（第 {number} 行）' for name, number in places)
        print(f'     {path} 的候选代号「{code}」同时在 {text}的候选表里，检索到其中一条时分不出它属于哪一格')  # gate-lint:detail
    sys.exit(1)
if tables == 0:
    print(f'  ! 扫了 {scanned} 份决策，一处候选表都没判出来，这一格无对象可判')
    sys.exit(77)
print(f'  ✓ 同一份决策里的候选代号没有跨分项复用：扫了 {scanned} 份决策、{tables} 处候选表、{len(all_codes)} 个代号')
PY
)

# ════════════════ 依据块与挂账表：推论要有判决、三方要有腿况 ════════════════

# 格「推论要有判决」「三方要有腿况」与格「说未定其实已定」共用：读决策分项「**依据**：」块的条目、解析三方判决 / 腿输出的指针、读挂账表。
# 只有这三格用（脚本里读成模块），所以不另立库文件；挂账表的写法在 .claude/gate.d/decision-evidence-lag.tsv 文件头。
IFS= read -r -d '' DECISION_EVIDENCE_SOURCE <<'PY' || true
"""决策分项「**依据**：」块的读法、三方判决 / 腿输出指针的解析、挂账表的读法。"""
import fnmatch, os, re, subprocess

DECISIONS = '.claude/kb/decisions'
LEDGER = '.claude/gate.d/decision-evidence-lag.tsv'
FENCE = re.compile(r'^[ \t]*```')
HEADING = re.compile(r'^(#{1,6})[ \t]+(.*?)[ \t]*$')
ITEM_HEADING = re.compile(r'^(已定项|未定项)[ \t]*(\d+)')
OWN_HISTORY = re.compile(r'^##[ \t]*历史版本[ \t]*$')
BLOCK_START = re.compile(r'^\*\*依据\*\*[：:](.*)$')
BLOCK_END = re.compile(r'^\*\*(?:欠|射程|定案|题面|问题)\*\*[：:]')
ENTRY_START = re.compile(r'^(?:[-*+][ \t]+|\d+[.)][ \t]+|\|)')
BULLET = re.compile(r'^(?:[-*+]|\d+[.)])[ \t]+')
QUOTED = re.compile(r'「[^」]*」')
CELLS = ('inference-needs-verdict', 'three-way-leg-record', 'cross-decision-status')
SNIPPET_MINIMUM = 8
REASON_MINIMUM = 4


def read_entries(decisions=DECISIONS):
    """每份决策正文里每个分项的「依据」块拆成条目：(文件路径, 分项, 起始行号, 条目全文)，另返回读了几份决策。
    条目 = 一个顶格列表项 / 编号项 / 表格行，或块里空行之后的一段；折行与缩进的子项并进上一条。
    「## 历史版本」起、围栏里的行不读；分项取最近一个 `### / #### 已定项 N / 未定项 N` 标题。"""
    entries, files = [], 0
    for name in sorted(os.listdir(decisions)):
        if not name.endswith('.md'):
            continue
        files += 1
        path = f'{decisions}/{name}'
        item, inside, in_fence, previous_blank, current = None, False, False, False, None
        with open(path, encoding='utf-8') as handle:
            lines = handle.read().splitlines()
        for number, line in enumerate(lines, 1):
            if OWN_HISTORY.match(line):
                break
            if FENCE.match(line):
                in_fence = not in_fence
                continue
            if in_fence:
                continue
            heading = HEADING.match(line)
            closing = heading or BLOCK_END.match(line)
            if closing and current is not None:
                entries.append((path, current[0], current[1], ' '.join(current[2]).strip()))
                current = None
            if heading:
                inside, previous_blank = False, False
                title = ITEM_HEADING.match(heading.group(2))
                item = f'{title.group(1)} {title.group(2)}' if title and len(heading.group(1)) >= 3 else None
                continue
            if BLOCK_END.match(line):
                inside = False
                continue
            start = BLOCK_START.match(line)
            if start and item is not None:
                if current is not None:
                    entries.append((path, current[0], current[1], ' '.join(current[2]).strip()))
                    current = None
                inside, previous_blank = True, False
                if start.group(1).strip():
                    current = (item, number, [start.group(1).strip()])
                continue
            if not inside:
                continue
            if not line.strip():
                previous_blank = True
                continue
            begins = bool(ENTRY_START.match(line)) or previous_blank
            previous_blank = False
            if begins:
                if current is not None:
                    entries.append((path, current[0], current[1], ' '.join(current[2]).strip()))
                current = (item, number, [BULLET.sub('', line).strip()])
            elif current is not None:
                current[2].append(line.strip())
            else:
                current = (item, number, [line.strip()])
        if current is not None:
            entries.append((path, current[0], current[1], ' '.join(current[2]).strip()))
    return entries, files


def strip_quotes(text):
    """去掉「」引起来的原话（被引的不是这一条自己在断言）。"""
    return QUOTED.sub('', text)


def artifact_names(history=True):
    """research/prompts 与 research/results 下此刻在树里的每一级名字，加上 git 历史里出现过的路径的每一级名字（归档出了树、取得回来的那些）。
    返回 (名字集合, 在树里的个数, 只在历史里的个数, 历史是否读得出)。"""
    here, back, history_ok = set(), set(), False
    for top in ('research/prompts', 'research/results'):
        if os.path.isdir(top):
            for _directory, directories, files in os.walk(top):
                here.update(directories)
                here.update(files)
    if history:
        try:
            # --no-renames：改名检测对整批加删的提交要两两比内容；关掉它名字集合不变（改名前的旧名在它被加进来的那个提交里已经列过）。
            # 样本 evidence-pointer-renamed-history-green（判决文件改过名、被点名的是改名前的名字）。
            done = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--no-renames', '--name-only', '--format=', '--', 'research/prompts', 'research/results'],
                                  capture_output=True, text=True)
        except OSError:
            done = subprocess.CompletedProcess([], 127, '', '')
        if done.returncode == 0:
            history_ok = True
            for path in done.stdout.splitlines():
                parts = path.strip().split('/')
                if len(parts) > 2 and parts[0] == 'research':
                    back.update(parts[2:])
    return here | back, len(here), len(back - here), history_ok


EXPLICIT = re.compile(r'research/(?:prompts|results)/([^\s`，。；、）)」\]]+)')
BARE_FILE = re.compile(r'(?<![/\w.\-*{])([A-Za-z0-9_][\w.\-*{},]*\.(?:md|out|tsv))(?![\w])')
BARE_DIRECTORY = re.compile(r'`([A-Za-z0-9_][\w.\-*{},]*)/`')


def expand_braces(token):
    found = re.search(r'\{([^{}]*)\}', token)
    if not found:
        return [token]
    expanded = []
    for choice in found.group(1).split(','):
        expanded += expand_braces(token[:found.start()] + choice + token[found.end():])
    return expanded


def exists(name, names):
    for alternative in expand_braces(name):
        if '*' in alternative:
            if any(fnmatch.fnmatch(candidate, alternative) for candidate in names):
                return True
        elif alternative in names:
            return True
    return False


def pointer_verdict(text, names):
    """条目里点名的研究材料：(存在的, 写成 research/prompts|results/… 而不存在的)。
    research/ 开头的每一处都得存在；裸文件名与目录名只要有一个存在就算指到了，不存在的裸名字不判（可能是 kb 页）。"""
    explicit = []
    for found in EXPLICIT.finditer(text):
        rest = found.group(1).rstrip('/.')
        if rest:
            explicit.append(rest.split('/')[-1])
    bare = [m.group(1) for m in BARE_FILE.finditer(text)] + [m.group(1) for m in BARE_DIRECTORY.finditer(text)]
    return [n for n in explicit + bare if exists(n, names)], [n for n in explicit if not exists(n, names)]


def read_ledger(path=LEDGER):
    """(行, 问题, 表在不在)：行 = (行号, 格名, 文件路径, 位置, 片段, 理由)。"""
    rows, problems = [], []
    if not os.path.isfile(path):
        return rows, problems, False
    with open(path, encoding='utf-8') as handle:
        for number, raw in enumerate(handle, 1):
            line = raw.rstrip('\n')
            if not line.strip() or line.startswith('#'):
                continue
            cells = line.split('\t')
            if len(cells) != 5 or not cells[4].startswith('#'):
                problems.append(f'{path}:{number} 要五列（格名、文件路径、位置、对象里逐字的一段、「# 理由」），实际 {len(cells)} 列：{line[:60]}')
                continue
            cell, target, place, snippet, reason = cells
            if cell not in CELLS:
                problems.append(f'{path}:{number} 格名「{cell}」不是这张表管的三格（{"、".join(CELLS)}）')
            elif len(snippet.strip()) < SNIPPET_MINIMUM:
                problems.append(f'{path}:{number} 对象里逐字的一段不足 {SNIPPET_MINIMUM} 个字：{snippet}')
            elif len(reason.lstrip('#').strip()) < REASON_MINIMUM:
                problems.append(f'{path}:{number} 理由不足 {REASON_MINIMUM} 个字')
            else:
                rows.append((number, cell, target, place, snippet, reason))
    return rows, problems, True


def apply_ledger(cell, red, rows, ledger_path=LEDGER):
    """red = [(文件路径, 位置, 行号, 全文, 说明)]。返回 (仍红的, 被挂账罩住的条数, 表的问题)。
    每一行必须恰好罩住这一格此刻判红的一个对象：罩不住（文件不在、位置改了、片段改了、对象已经不红）判红，删掉那一行。"""
    problems, used = [], set()
    for number, row_cell, target, place, snippet, _reason in rows:
        if row_cell != cell:
            continue
        hits = [index for index, entry in enumerate(red) if entry[0] == target and entry[1] == place and snippet in entry[3]]
        if not hits:
            problems.append(f'{ledger_path}:{number} 登记的「{target} {place}」里含「{snippet[:30]}」的对象此刻在格「{cell}」不红'
                            f'（文件、位置或片段对不上，或已经改好）')
        elif len(hits) > 1:
            problems.append(f'{ledger_path}:{number} 「{target} {place}」里有 {len(hits)} 处都含「{snippet[:30]}」，这一行罩的对象不唯一，片段写长一点')
        else:
            used.add(hits[0])
    return [entry for index, entry in enumerate(red) if index not in used], len(used), problems


def base_ledger_text(diff_base, ledger_path=LEDGER):
    """基准那一版的挂账表正文：(正文, None)；不在 git 仓里（diff_base 空）、基准那一版没有这张表：(None, None)，不比；
    git 取不到基准那一版（基准名不在、对象库缺了那一份）：(None, 问题句)，调用方判红，不当成「基准里没有这张表」。
    三种结局的取法只在 lib-lag-baseline.py 的 baseline_text 一份（路径由环境变量 LIB_LAG_BASELINE 给），路径按当前目录算：样本根在别的仓里时不会读到那个仓的表。"""
    if not diff_base:
        return None, None
    import importlib.util
    spec = importlib.util.spec_from_file_location('lag_baseline', os.environ['LIB_LAG_BASELINE'])
    lag_baseline = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(lag_baseline)
    text, problem = lag_baseline.baseline_text(diff_base, ledger_path)
    return text, (None if problem is None else f'{problem.sentence()}，这张表只缩不涨没比')


def ledger_growth(cell, rows, diff_base):
    """(涨了的行, 问题句)：基准那一版的表里没有的 (格名, 路径, 位置, 片段) = 涨了的行（只缩不涨）；
    基准取不到时涨了的行是空表、问题句非空（调用方判红），基准里没有这张表、不在 git 仓里时两样都空。"""
    base_text, problem = base_ledger_text(diff_base)
    if base_text is None:
        return [], problem
    base = set()
    for raw in base_text.splitlines():
        cells = raw.split('\t')
        if raw.strip() and not raw.startswith('#') and len(cells) == 5:
            base.add(tuple(cells[:4]))
    return [row for row in rows if row[1] == cell and tuple(row[1:5]) not in base], None
PY

# ════ 格「推论要有判决」════
# 决策分项「**依据**：」块里自陈是推论的条目，要指到一份存在的 research/prompts/*-main-verification.md（主 agent 的判决）：
# 文件此刻在树里，或照 .claude/rules/directory-layout.md 归档出了树、在 git 历史里取得回来（git log --all 里出现过这条路径）。指不到判红（C402）。
# 条目 = 依据块里的一个顶格列表项 / 编号项 / 表格行，或空行之后的一段（折行并入）；「判决指针」要和推论记号在同一条目里。
# 记号表（先在真仓 grep 过：这些写法 .claude/kb/decisions/ 下的依据块里都出现过，引文「…」里的不算）：
#   「是推论」、「是推的」「，推的」「（推的」（前面是「不」的不算：「不是推的」），
#   「只走了本地腿」「云端腿欠着」「腿欠着」，
#   「没走三方」「没有走三方」「未走三方」「未三方论证」——这一组是用户裁决的写法（条目里写「用户……，没有走三方论证，可推翻」），
#     同一条目里有「用户」就不算：裁决人是用户，不是一条没走三方的推论；推论记号那三组不给这个豁免。
#   看不见：词表外的写法（「推出」「推导」「无实验」本身不算：那是在说没量，不是在说没走三方）、记号写在依据块之外的。
# 存量挂账：.claude/gate.d/decision-evidence-lag.tsv 里本格的行，文件头写了写法；一行必须恰好罩住此刻判红的一条，罩不住判红，表里新增的行（相对基准那一版）判红；
#   git 取不到基准那一版判红，不当成基准里没有这张表（取法 lib-lag-baseline.py 的 baseline_text，三格共用 base_ledger_text；样本 evidence-ledger-base-unreadable-red）。
# 弄坏开关 DOC_DECISIONS_BREAK=evidence-pointer-unchecked（点名了判决就算指到，不查存在）/ evidence-history-ignored（只认树里的，不读 git 历史）/
#   evidence-ledger-unjudged（挂账表的行不核罩没罩住、写得对不对）/ evidence-ledger-growth-unjudged（表涨了不判）/
#   inference-user-verdict-judged（带「用户」字样的「没走三方」也算推论）/ inference-negation-ignored（「不是推的」也算记号）。
# 样本：inference-needs-verdict-red（无判决、判决文件不存在各一条，另有一条带存在的判决不判）、inference-needs-verdict-green（带判决、「不是推的」、带「用户」字样的「没走三方」、被挂账罩住）、
#   evidence-ledger-red（挂账表写坏与罩不住）、evidence-ledger-grew-red（表相对基准涨了）、evidence-ledger-base-unreadable-red（基准那一版挂账表的对象缺了，三格都红）、
#   evidence-pointer-history-green（判决文件只在 git 历史里）。
grid_inference_needs_verdict() (
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格无对象可判"; exit 77; }
  load_diff_base

  DECISION_EVIDENCE_SOURCE="$DECISION_EVIDENCE_SOURCE" LIB_LAG_BASELINE="$LIB_LAG_BASELINE" python3 - "$BREAK_SWITCH" "$diff_base" <<'PY'
import os, re, sys, types

lib = types.ModuleType('decision_evidence')
exec(compile(os.environ['DECISION_EVIDENCE_SOURCE'], 'DECISION_EVIDENCE_SOURCE', 'exec'), lib.__dict__)
BREAK, DIFF_BASE = sys.argv[1], sys.argv[2]
CELL = 'inference-needs-verdict'
VERDICT = re.compile(r'([\w.\-]+-main-verification\.md)')
negation_guard = '' if BREAK == 'inference-negation-ignored' else '(?<!不)'
# (记号名, 正则, 同一条目里有「用户」就不算)
MARKERS = [
    ('是推论', re.compile(negation_guard + '是推论'), False),
    ('是推的', re.compile(negation_guard + '(?:是|，|（)推的'), False),
    ('只走了本地腿', re.compile('只走了本地腿|云端腿欠着|腿欠着'), False),
    ('没走三方', re.compile('(?:没有?走|未走)三方|未三方论证'), BREAK != 'inference-user-verdict-judged'),
]

if BREAK == 'evidence-pointer-unchecked':
    lib.exists = lambda name, names: True
names, in_tree, only_history, history_ok = lib.artifact_names(history=BREAK != 'evidence-history-ignored')
entries, files = lib.read_entries()
if not entries:
    print(f'  ! 扫了 {files} 份决策，一条「**依据**：」块里的条目都没读到，这一格无对象可判')
    sys.exit(77)

red, marked, pointed, user_exempt = [], 0, 0, 0
for path, item, number, text in entries:
    plain = lib.strip_quotes(text)
    hit = None
    for label, pattern, exempt_with_user in MARKERS:
        if pattern.search(plain):
            if exempt_with_user and '用户' in plain:
                user_exempt += 1
                continue
            hit = label
            break
    if not hit:
        continue
    marked += 1
    verdicts = VERDICT.findall(text)
    found = [v for v in verdicts if lib.exists(v, names)]
    if found:
        pointed += 1
        continue
    why = f'记号「{hit}」，' + (f'点名的判决 {"、".join(verdicts)} 在树里与 git 历史里都找不到' if verdicts else '同一条里没有指到任何 *-main-verification.md')
    red.append((path, item, number, text, why))

rows, ledger_problems, ledger_present = lib.read_ledger()
if BREAK == 'evidence-ledger-ignored':
    rows, ledger_problems = [], []
red, ledgered, stale = lib.apply_ledger(CELL, red, rows)
grown, base_problem = ([], None) if BREAK == 'evidence-ledger-growth-unjudged' else lib.ledger_growth(CELL, rows, DIFF_BASE)
table_problems = [] if BREAK == 'evidence-ledger-unjudged' else ledger_problems + stale
table_problems += [f'{lib.LEDGER}:{row[0]} 相对基准那一版新增了一行（{row[2]} {row[3]}）：这张表只缩不涨' for row in grown]
table_problems += [base_problem] if base_problem else []

if red or table_problems:
    if red:
        print(f'  ✗ 自陈是推论的依据条目没有指到存在的判决：{len(red)} 条')  # gate-lint:summary
        for path, item, number, text, why in red:
            print(f'     {path}:{number} {item}：{why}')  # gate-lint:detail
            print(f'       原文：{text[:110]}')
        print('     → 怎么办：在这一条里写上判决文件（`research/prompts/<轮>-main-verification.md`，在树里或 git 历史里取得回来）；')
        print('               推论还没走三方的，先走一轮三方（.claude/rules/three-way-inference.md），或把这一句改成观测，不要先登挂账表。')
    if table_problems:
        print(f'  ✗ 挂账表 {lib.LEDGER} 有 {len(table_problems)} 处不对')  # gate-lint:summary
        for problem in table_problems:
            print(f'     {problem}')  # gate-lint:detail
        print('     → 怎么办：这张表只收此刻判红、当场补不上判决的存量，一行一条：罩不住的行删掉（对象改好了就是它该退场），格式写坏的照表头改；')
        print('               新判红的条目补判决，别往表里加行；读不出、取不到基准那一版的，按括号里 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑。')
    sys.exit(1)
print(f'  ✓ 自陈是推论的依据条目都指到了存在的判决：扫 {files} 份决策、{len(entries)} 条依据条目，'
      f'带推论记号的 {marked} 条（指到判决 {pointed} 条，挂账 {ledgered} 条；用户定案的「没走三方」{user_exempt} 条不算）')
print(f'     判决文件：树里 {in_tree} 个名字，只在 git 历史里取得回来的 {only_history} 个' + ('' if history_ok else '（git 历史读不出，只认树里的）'))
PY
)

# ════ 格「三方要有腿况」════
# 决策分项「**依据**：」块里每一条写了「三方」的条目（C403），要二选一：
#   ① 指到研究材料：条目里点名了 research/prompts 或 research/results 下存在的判决、材料或腿输出（在树里，或在 git 历史里取得回来）；
#      写成 research/prompts/… 或 research/results/… 的每一处都得存在，裸文件名与目录名（`_d9q6-background.md`、`d2-item1-local.out`）有一个存在就算；
#   ② 同一条目里写明腿况缺失（「没有腿况记录」「腿况缺失」「材料没有留存」「查不到对应的提示」「仓里只留了」……），而且这一条不许写成阴性结论
#      （「都没找到」「构造不出」「没打中」「找不到反例」「没发现」「查不到任何」「零命中」「没有反例」「未发现」……）：
#      没有腿况就分不出「腿跑了没打中」与「腿根本没跑」（.claude/rules/three-way-inference.md「一条腿只抽一次样不算一次观测」）。
# 「三方」不含「第三方」；「没走三方论证」「未三方论证」这类否定的不是在说跑过三方，不算；「…」里引的话不进阴性结论的判断（被引来降级的原话）。
# 看不见：指针存在但不是这一轮的、「腿况在变更史」这类指到 kb 别处的说法（要指到 research/ 下才算指到）、把阴性结论整句包进「」绕过去的。
# 存量挂账：同格「推论要有判决」（.claude/gate.d/decision-evidence-lag.tsv，行里的格名是 three-way-leg-record）。
# 弄坏开关 DOC_DECISIONS_BREAK=evidence-pointer-unchecked / evidence-history-ignored / evidence-ledger-unjudged / evidence-ledger-growth-unjudged（同上）、
#   three-way-leg-negative-allowed（腿况缺失的条目写了阴性结论也放过）/ three-way-leg-quotes-judged（「」引文里的阴性写法也算）/ three-way-leg-negated-judged（「没走三方」也算在说三方）。
# 样本：three-way-leg-record-red（判决文件不存在、没有指针也没写腿况、腿况缺失却写了阴性结论各一条）、three-way-leg-record-green（指到存在的材料、腿况缺失、否定写法、
#   引文里的阴性写法、被挂账罩住）；evidence-ledger-red / evidence-ledger-grew-red / evidence-pointer-history-green 同上一格。
grid_three_way_leg_record() (
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格无对象可判"; exit 77; }
  load_diff_base

  DECISION_EVIDENCE_SOURCE="$DECISION_EVIDENCE_SOURCE" LIB_LAG_BASELINE="$LIB_LAG_BASELINE" python3 - "$BREAK_SWITCH" "$diff_base" <<'PY'
import os, re, sys, types

lib = types.ModuleType('decision_evidence')
exec(compile(os.environ['DECISION_EVIDENCE_SOURCE'], 'DECISION_EVIDENCE_SOURCE', 'exec'), lib.__dict__)
BREAK, DIFF_BASE = sys.argv[1], sys.argv[2]
CELL = 'three-way-leg-record'
CLAIM = re.compile(r'(?<!第)三方')
NEGATED = re.compile(r'(?:没有?走|未走|没开|不走|未经|没经过)三方|未三方')
MISSING = re.compile(r'没有腿况记录|腿况缺失|腿况没有留存|腿况未记录|未记录腿况|材料没有留存|查不到对应的提示|仓里只留了')
NEGATIVE = re.compile(r'都没找到|构造不出|没打中|找不到反例|没发现|查不到任何|零命中|没有反例|无反例|未发现|没有找到')

if BREAK == 'evidence-pointer-unchecked':
    lib.exists = lambda name, names: True
names, in_tree, only_history, history_ok = lib.artifact_names(history=BREAK != 'evidence-history-ignored')
entries, files = lib.read_entries()
if not entries:
    print(f'  ! 扫了 {files} 份决策，一条「**依据**：」块里的条目都没读到，这一格无对象可判')
    sys.exit(77)

red, claims, pointed, missing_stated = [], 0, 0, 0
for path, item, number, text in entries:
    claim_text = text if BREAK == 'three-way-leg-negated-judged' else NEGATED.sub('', text)
    if not CLAIM.search(claim_text):
        continue
    claims += 1
    found, absent = lib.pointer_verdict(text, names)
    if found and not absent:
        pointed += 1
        continue
    judged = text if BREAK == 'three-way-leg-quotes-judged' else lib.strip_quotes(text)
    stated = MISSING.search(judged)
    negative = None if BREAK == 'three-way-leg-negative-allowed' else NEGATIVE.search(judged)
    if stated and not negative and not absent:
        missing_stated += 1
        continue
    if absent:
        why = f'点名的 {"、".join(absent)} 在 research/ 的树里与 git 历史里都找不到'
    elif stated:
        why = f'写了腿况缺失，却同时写了阴性结论「{negative.group(0)}」：没有腿况就分不出腿没打中与腿没跑'
    else:
        why = '写了「三方」，既没有指到 research/prompts 或 research/results 下存在的判决、材料或腿输出，也没写腿况缺失'
    red.append((path, item, number, text, why))

rows, ledger_problems, ledger_present = lib.read_ledger()
if BREAK == 'evidence-ledger-ignored':
    rows, ledger_problems = [], []
red, ledgered, stale = lib.apply_ledger(CELL, red, rows)
grown, base_problem = ([], None) if BREAK == 'evidence-ledger-growth-unjudged' else lib.ledger_growth(CELL, rows, DIFF_BASE)
table_problems = [] if BREAK == 'evidence-ledger-unjudged' else ledger_problems + stale
table_problems += [f'{lib.LEDGER}:{row[0]} 相对基准那一版新增了一行（{row[2]} {row[3]}）：这张表只缩不涨' for row in grown]
table_problems += [base_problem] if base_problem else []

if red or table_problems:
    if red:
        print(f'  ✗ 依据里写了「三方」而没有腿况记录：{len(red)} 条')  # gate-lint:summary
        for path, item, number, text, why in red:
            print(f'     {path}:{number} {item}：{why}')  # gate-lint:detail
            print(f'       原文：{text[:110]}')
        print('     → 怎么办：在这一条里点名那一轮的判决或腿输出（research/prompts/<轮>-main-verification.md、<轮>-*-output.md，在树里或 git 历史里取得回来）；')
        print('               取不回来的，写明「没有腿况记录」之类的腿况缺失，并去掉「都没找到」「构造不出」这类阴性写法；要拿那一轮支撑新结论，先补跑一轮。')
    if table_problems:
        print(f'  ✗ 挂账表 {lib.LEDGER} 有 {len(table_problems)} 处不对')  # gate-lint:summary
        for problem in table_problems:
            print(f'     {problem}')  # gate-lint:detail
        print('     → 怎么办：这张表只收此刻判红、当场补不上指针的存量，一行一条：罩不住的行删掉（对象改好了就是它该退场），格式写坏的照表头改；')
        print('               新判红的条目补指针或腿况，别往表里加行；读不出、取不到基准那一版的，按括号里 git 的报错修好仓库状态（基准要存在、对象库没坏）再跑。')
    sys.exit(1)
print(f'  ✓ 依据里写了「三方」的条目都有腿况记录：扫 {files} 份决策、{len(entries)} 条依据条目，写了「三方」的 {claims} 条'
      f'（指到存在的材料 {pointed} 条，写明腿况缺失 {missing_stated} 条，挂账 {ledgered} 条）')
print(f'     材料文件：树里 {in_tree} 个名字，只在 git 历史里取得回来的 {only_history} 个' + ('' if history_ok else '（git 历史读不出，只认树里的）'))
PY
)

# ════ 格「欠段引的欠账已还清」════
# 决策中文正文的 `**欠**：` 行引了一笔已经挪进 checks-owed.md「已还清」表的账、又没说它还清了，读的人会把还清的账当开着的
# （records/2026-10-30-知识腐烂治理按现状.md「现状」表第 2 行；records/2026-10-23-项目进度与知识腐烂审计-深入复核.md R22–R29 是同类）。
# 判据：.claude/kb/decisions/ 下以数字起头的中文正文（<号>-<名>.md，不含 .en.md、.ja.md），「## 历史版本」那一行之前、以 `**欠**：` 起头的行里，
#   每个 C<编号>（按整个号认，C1 不命中 C10）算一处引账；紧跟在另一个 C<编号> 后面那一对配平的全角括号里的不算：那是别的账照抄的登记简称
#   （「C177（C34 作废之后被解锁的三处没复核）」里的 C34，records/2026-10-30-知识腐烂治理按现状.md「批 2」），括号没配平的不跳过。
#   引的账在已还清表里（分界照 lib-owed.py 的 read_owed_table）时，取从这个编号起、到这一行里下一处引账或分号（；或 ;）为止的那一截，连同它前面 OWED_REFS_PAID_PREFIX_CHARACTERS 个字；
#   同一行里这个号的每一处都这样取，任一处写了 OWED_REFS_PAID_MARKS 里的哪一个词，这一行里它的每一处都放过（先列「C445（…）、C446（…）」、后补「其中 C446（…） 已还清」的写法）；
#   一处都没写，判红，一行一个号打一处「路径:行号 C<编号>（简称）」。
#   checks-owed.md 不在、或认不出「已还清」标题（paid_heading_found 为假），整格红；一份中文正文都没有，退 77。
# 现状句（射程扩到欠段之外，records/2026-10-29-subagent拆分提案.md 第一百五十四节 ①）：决策中文正文里欠段之外的行，与 OWED_REFS_PAID_MILESTONE_DIRECTORY 下以数字起头的
#   中文正文（不含 .en.md、.ja.md）的每一行，「## 历史版本」那一行之前、围栏代码块之外，按 。；;！？!? 切句；引账的认法与欠段相同（别的 C 号简称括注里的不算），
#   一句里引的账在已还清表里、那一句把它当现欠、又没写明已还，判红，一行一个号打一处（同一行几句都这样的报几句）。表格行整行算一句（竖线不切）。
#   当现欠：那一句（去掉每个 C 号后面配平的全角简称括注，再去掉 OWED_REFS_PAID_STATUS_NOT_CLAIMS 里的词：欠账表、等待、对待、期待、看待）
#     带 OWED_REFS_PAID_STATUS_OWED_WORDS 里的一个词（欠、待、没做、未做、还开着、仍开着；还欠、仍欠、只欠、待补、还没做都含在里面），或写着「记在 C<编号>」（记在与号之间只隔空白）。
#   写明已还：那一句（去掉句里每个 C 号在 checks-owed.md 登记的简称原文，简称里的字不算写明）带 OWED_REFS_PAID_STATUS_PAID_MARKS 里的一个词
#     （已还、还清、销账、已关、关账、做完、已做、已完成；已还清含在已还里）。写明已还的不红，两样都带也不红；不带现欠说法的只是提到这笔账，不判。
#   与欠段共用滞后表（同一路径、锚命中的那一行、同一个 C 号就放过）；红行另起一句汇总，成功句另起一行「现状句：…」报扫了几份、几句、没写现欠说法的几个、滞后几处。
#   管不到的：句子里写的是别的意思而恰好带了那几个词的（「还清之前」照放过，「欠」「待」说的是同一句里别的号也照红）；只写简称不写号的；只靠前后文说现欠、那一句里没有那几个词的。
# 滞后表 OWED_REFS_PAID_LAG_TABLE（被判的根下）：一行「<路径><TAB>C<编号><TAB><行内锚><TAB># <为什么还没改>」，锚是那一行里至少 OWED_REFS_PAID_LAG_ANCHOR_CHARACTERS 个字的原文，理由至少 4 个字；
#   判红的一处落在某一条登记里（同一路径、锚恰好命中的那一行、同一个 C 号）就不红，成功句报「滞后 N 处」。表不在就是没有登记，判红的一律红。
#   收的是当场改不了的存量（那份决策在一轮还没判决的三方的开工快照里这类），不收新写的。
#   表自己判红的：列数不是四列、C 号写法不对、理由没写或不到 4 个字、锚不到 OWED_REFS_PAID_LAG_ANCHOR_CHARACTERS 个字、同一条登记两次、登记的路径不在、
#   锚在那份文件里命中的不是恰好一行、锚命中的那一行里没有这个 C 号、这个 C 号已不在已还清表里；登记的那一处此刻不红（改好了没删行）只打一行「!」提醒删掉，不判红。
#   只缩不涨：与 gate 基准（scripts/gate/changed-paths.sh 的 gate_diff_base gate）那一版按（路径、C 号、锚）比，基准那一版里没有的行判红（锚改了也算涨）；
#   基准那一版经 lib-lag-baseline.py 的 baseline_rows 取（与 doc-text.sh 的 dated-status 格同一种取法，登记的搬迁照换算）：
#   基准里没有这张表（初次登记）不比，git 取不到判红，被判的根不在 git 仓里不比，成功句写明比没比。
# 管不到的：接在 `**欠**：` 行下面的续行；写成「C 374」或只写简称的引用；还开着的账在欠段里写错了状态（归别的格与人）；
#   紧跟在别的 C 号后面的括注里真在引一笔已还清的账（照判据当简称跳过）。
# 样本（文件头「样本」一段逐份写了放了什么）：owed-refs-paid-red、owed-refs-paid-green、owed-refs-paid-heading-missing-red 判同一截与整格的分界；
#   owed-refs-paid-abbreviation-parenthesis-green 判简称括注；owed-refs-paid-same-line-mark-green、owed-refs-paid-same-line-unmarked-red 判同一行里同一个号；
#   owed-refs-paid-lag-green、owed-refs-paid-lag-growth-red、owed-refs-paid-lag-anchor-wrong-red、owed-refs-paid-lag-table-red 判滞后表；
#   owed-refs-paid-status-sentence-red、owed-refs-paid-status-sentence-green 判现状句。
# 弄坏开关（DOC_DECISIONS_BREAK 的值，打开后判错的样本在文件头开关表）：owed-refs-paid-table-unread 不读已还清表（当它是空的）；
#   owed-refs-paid-abbreviation-parenthesis-counted 别的 C 号简称括注里的 C 号照样算引账；owed-refs-paid-same-line-other-mark-ignored 每一处只看自己那一截；
#   owed-refs-paid-lag-unread 不读滞后表；owed-refs-paid-lag-growth-unjudged 滞后表不与 gate 基准那一版比；
#   owed-refs-paid-status-sentences-skipped 现状句不判（退回只判欠段）；owed-refs-paid-status-marks-ignored 现状句写了已还也照红；
#   owed-refs-paid-status-whole-line 现状句整行算一句（同一行别的句里写了还清也算写了）；owed-refs-paid-status-owed-words-ignored 不看现欠说法；
#   owed-refs-paid-status-old-marks 只认已还清、还清、销账；owed-refs-paid-status-claims-in-abbreviation 简称括注里的词也算现欠说法；
#   owed-refs-paid-status-not-claims-counted 不先剔掉欠账表、等待这类词；owed-refs-paid-status-ledgered-in-unseen「记在 C<编号>」不算现欠说法。
grid_owed_refs_paid() (
  [[ -d .claude/kb/decisions ]] || { echo "  ! 找不到 .claude/kb/decisions，这一格无对象可判"; exit 77; }
  load_diff_base
  python3 - .claude/kb/decisions .claude/kb/checks-owed.md "$LIB_OWED" "$LIB_LAG_BASELINE" "$BREAK_SWITCH" "$OWED_REFS_PAID_PREFIX_CHARACTERS" "$OWED_REFS_PAID_MARKS" \
    "$OWED_REFS_PAID_LAG_TABLE" "$OWED_REFS_PAID_LAG_ANCHOR_CHARACTERS" "$diff_base" "$OWED_REFS_PAID_MILESTONE_DIRECTORY" \
    "$OWED_REFS_PAID_STATUS_OWED_WORDS" "$OWED_REFS_PAID_STATUS_PAID_MARKS" "$OWED_REFS_PAID_STATUS_NOT_CLAIMS" <<'PY'
import importlib.util, os, re, sys

(decisions_directory, owed_path, owed_library_path, lag_baseline_library_path, BREAK, prefix_characters, marks_text,
 lag_table_path, lag_anchor_characters, diff_base, milestone_directory,
 status_owed_words_text, status_paid_marks_text, status_not_claims_text) = sys.argv[1:15]
prefix_characters = int(prefix_characters)
lag_anchor_characters = int(lag_anchor_characters)
paid_marks = marks_text.split()
status_owed_words = status_owed_words_text.split()
status_paid_marks = paid_marks if BREAK == "owed-refs-paid-status-old-marks" else status_paid_marks_text.split()
status_not_claims = status_not_claims_text.split()


def load_library(module_name, library_path):
    library_spec = importlib.util.spec_from_file_location(module_name, library_path)
    library = importlib.util.module_from_spec(library_spec)
    library_spec.loader.exec_module(library)
    return library


owed_library = load_library("owed", owed_library_path)
lag_baseline = load_library("lag_baseline", lag_baseline_library_path)

OWED_LINE_PREFIX = "**欠**："
OWED_NUMBER = re.compile(r"(?<![A-Za-z0-9_])C[0-9]+(?![0-9])")
OWED_NUMBER_SPELLING = re.compile(r"C[0-9]+")
SEGMENT_END = re.compile(r"[；;]")
NUMBERED_ORIGINAL = re.compile(r"^[0-9]+-.*\.md$")
TRANSLATION_SUFFIXES = (".en.md", ".ja.md")
ABBREVIATION_OPENING, ABBREVIATION_CLOSING = "（", "）"
LAG_COLUMNS = 4
LAG_REASON_CHARACTERS = 4
# 现状句按这几个标点切句；围栏代码块里的行不判
STATUS_SENTENCE_END = re.compile(r"[。；;！？!?]")
STATUS_FENCE = re.compile(r"^\s*(```|~~~)")
# 现状句里「记在 C<编号>」算把那一笔账当现欠（「记在变更史」这类不跟号的不算）
STATUS_LEDGERED_IN = re.compile(r"记在\s*C[0-9]")


def abbreviation_spans(line, numbers):
    """紧跟在每个 C 号后面那一对配平的全角括号：[(开括号的位置, 闭括号的位置)]；到行尾都没配平的不算。"""
    spans = []
    for number in numbers:
        if not line.startswith(ABBREVIATION_OPENING, number.end()):
            continue
        depth = 0
        for position in range(number.end(), len(line)):
            if line[position] == ABBREVIATION_OPENING:
                depth += 1
            elif line[position] == ABBREVIATION_CLOSING:
                depth -= 1
                if depth == 0:
                    spans.append((number.end(), position))
                    break
    return spans


def owed_references(line):
    """(这一行里算引账的 C 号, 落在别的 C 号简称括注里而跳过的个数)。"""
    numbers = list(OWED_NUMBER.finditer(line))
    if BREAK == "owed-refs-paid-abbreviation-parenthesis-counted":
        return numbers, 0
    spans = abbreviation_spans(line, numbers)
    counted = [number for number in numbers if not any(opening < number.start() < closing for opening, closing in spans)]
    return counted, len(numbers) - len(counted)


def status_claims_owed(line, sentence_start, sentence_end):
    """这一句把账当现欠没有：简称括注（连括号）与 OWED_REFS_PAID_STATUS_NOT_CLAIMS 的词剔掉之后，带现欠的词或「记在 C<编号>」。"""
    if BREAK == "owed-refs-paid-status-owed-words-ignored":
        return True
    spans = [] if BREAK == "owed-refs-paid-status-claims-in-abbreviation" else abbreviation_spans(line, list(OWED_NUMBER.finditer(line)))
    claim_text = "".join(character for position, character in enumerate(line[sentence_start:sentence_end], sentence_start)
                         if not any(opening <= position <= closing for opening, closing in spans))
    if BREAK != "owed-refs-paid-status-not-claims-counted":
        for word in status_not_claims:
            claim_text = claim_text.replace(word, "")
    if any(word in claim_text for word in status_owed_words):
        return True
    return BREAK != "owed-refs-paid-status-ledgered-in-unseen" and STATUS_LEDGERED_IN.search(claim_text) is not None


def status_marks_paid(sentence):
    """这一句写明已还没有：句里每个 C 号在欠账表登记的简称原文剔掉之后（简称里的「已还」不是写明），带 OWED_REFS_PAID_STATUS_PAID_MARKS 的词。"""
    if BREAK == "owed-refs-paid-status-marks-ignored":
        return False
    for code in {found.group(0) for found in OWED_NUMBER.finditer(sentence)}:
        registered_name = paid_names.get(code) or owed_table.open_names.get(code)
        if registered_name:
            sentence = sentence.replace(registered_name, "")
    return any(mark in sentence for mark in status_paid_marks)


originals = sorted(name for name in os.listdir(decisions_directory)
                   if NUMBERED_ORIGINAL.match(name) and not name.endswith(TRANSLATION_SUFFIXES))
translations = sorted(name for name in os.listdir(decisions_directory)
                      if NUMBERED_ORIGINAL.match(name) and name.endswith(TRANSLATION_SUFFIXES))
if not originals:
    print(f"  ! {decisions_directory} 下没有以数字起头的中文正文，这一格无对象可判")
    sys.exit(77)
owed_table = owed_library.read_owed_table(owed_path)
if not owed_table.file_found or not owed_table.paid_heading_found:
    print(f"  ✗ {owed_path} {'不在' if not owed_table.file_found else '里认不出「已还清」标题'}，分不出欠段引的账哪些已还清")
    print(f"     → 怎么办：欠账表按 lib-owed.py 的分界读（一行只写「已还清」的标题之后是已还清表）；标题改了就改回，表搬了家就照 .claude/rules/path-moves.md 改这一格的路径。")
    sys.exit(1)
paid_names = {} if BREAK == "owed-refs-paid-table-unread" else owed_table.paid_names

# ── 逐行找判红的处：一处 = (路径, 行号, C 号, 这一行里它没写还清的几次) ──
owed_line_count = reference_count = skipped_count = paid_reference_count = 0
marked_in_segment = marked_elsewhere_in_line = 0
red_places = []
for name in originals:
    path = os.path.join(decisions_directory, name)
    with open(path, encoding="utf-8") as decision_file:
        for line_number, line in enumerate(decision_file, 1):
            if owed_library.HISTORY_HEADING.match(line):
                break
            if not line.startswith(OWED_LINE_PREFIX):
                continue
            owed_line_count += 1
            references, skipped = owed_references(line)
            reference_count += len(references)
            skipped_count += skipped
            segment_marks = {}
            for index, number in enumerate(references):
                code = number.group(0)
                if code not in paid_names:
                    continue
                paid_reference_count += 1
                segment_end = references[index + 1].start() if index + 1 < len(references) else len(line)
                semicolon = SEGMENT_END.search(line, number.end(), segment_end)
                if semicolon:
                    segment_end = semicolon.start()
                judged_text = line[max(0, number.start() - prefix_characters):segment_end]
                segment_marks.setdefault(code, []).append(any(mark in judged_text for mark in paid_marks))
            for code, marks in segment_marks.items():
                marked_in_segment += marks.count(True)
                unmarked = marks.count(False)
                if not unmarked:
                    continue
                if any(marks) and BREAK != "owed-refs-paid-same-line-other-mark-ignored":
                    marked_elsewhere_in_line += unmarked
                    continue
                red_places.append((path, line_number, code, unmarked))

# ── 现状句：决策中文正文欠段之外的行与里程碑页中文正文的每一行，按句判（判据在这一格函数上方的注释「现状句」一段）──
milestone_originals = (sorted(name for name in os.listdir(milestone_directory)
                              if NUMBERED_ORIGINAL.match(name) and not name.endswith(TRANSLATION_SUFFIXES))
                       if os.path.isdir(milestone_directory) else [])
status_files = ([(os.path.join(decisions_directory, name), True) for name in originals]
                + [(os.path.join(milestone_directory, name), False) for name in milestone_originals])
status_sentence_count = status_reference_count = status_paid_count = status_marked_count = status_unclaimed_count = 0
status_red_places = []
if BREAK != "owed-refs-paid-status-sentences-skipped":
    for path, owed_lines_judged_elsewhere in status_files:
        in_fence = False
        with open(path, encoding="utf-8") as status_file:
            for line_number, line in enumerate(status_file, 1):
                if owed_library.HISTORY_HEADING.match(line):
                    break
                if STATUS_FENCE.match(line):
                    in_fence = not in_fence
                    continue
                if in_fence or (owed_lines_judged_elsewhere and line.startswith(OWED_LINE_PREFIX)):
                    continue
                references, _skipped = owed_references(line)
                if not references:
                    continue
                unmarked_codes = {}
                # 句的边界（弄坏开关 owed-refs-paid-status-whole-line 下整行算一句）；每处引账按位置归进它所在的那一句
                ends = [] if BREAK == "owed-refs-paid-status-whole-line" else [found.end() for found in STATUS_SENTENCE_END.finditer(line)]
                starts = [0] + ends
                sentence_codes = {}
                for number in references:
                    index = sum(1 for end in ends if end <= number.start())
                    sentence_codes.setdefault(index, []).append(number.group(0))
                for index, codes in sorted(sentence_codes.items()):
                    sentence_start = starts[index]
                    sentence_end = ends[index] if index < len(ends) else len(line)
                    status_sentence_count += 1
                    status_reference_count += len(codes)
                    if not any(code in paid_names for code in codes):
                        continue
                    marked = status_marks_paid(line[sentence_start:sentence_end])
                    claimed = status_claims_owed(line, sentence_start, sentence_end)
                    for code in codes:
                        if code not in paid_names:
                            continue
                        status_paid_count += 1
                        if marked:
                            status_marked_count += 1
                        elif not claimed:
                            status_unclaimed_count += 1
                        else:
                            unmarked_codes[code] = unmarked_codes.get(code, 0) + 1
                for code, unmarked in unmarked_codes.items():
                    status_red_places.append((path, line_number, code, unmarked))
red_places.extend(status_red_places)

# ── 滞后表：读、核写法、与 gate 基准那一版比 ──
lag_problems = []    # 一句一处
lag_covers = {}      # 写对了的登记：(路径, 行号, C 号) -> 表的行号
lag_growth = []      # (表的行号, 路径, C 号, 锚)
lag_row_count = 0
lag_note = ""
if BREAK == "owed-refs-paid-lag-unread":
    print(f"  ! 弄坏开关 owed-refs-paid-lag-unread：不读滞后表 {lag_table_path}，判红的处一律红")
    lag_note = "弄坏开关 owed-refs-paid-lag-unread：没读滞后表"
elif not os.path.isfile(lag_table_path):
    lag_note = "没有滞后表"
else:
    with open(lag_table_path, encoding="utf-8", errors="replace") as lag_file:
        lag_entries = lag_baseline.content_rows(lag_file.read())
    lag_row_count = len(lag_entries)
    first_entry_of = {}
    registered_lines = {}
    for entry_number, columns in lag_entries:
        place = f"{lag_table_path}:{entry_number}"
        if len(columns) != LAG_COLUMNS:
            lag_problems.append(f"{place} 有 {len(columns)} 列，要四列：<路径><TAB>C<编号><TAB><行内锚><TAB># <为什么还没改>")
            continue
        registered_path, code, anchor, reason = (column.strip() for column in columns)
        if not OWED_NUMBER_SPELLING.fullmatch(code):
            lag_problems.append(f"{place} 第二列「{code}」不是 C<编号> 的写法")
            continue
        if not reason.startswith("#") or len(reason.lstrip("#").strip()) < LAG_REASON_CHARACTERS:
            lag_problems.append(f"{place} 第四列要以 # 起头、写为什么还没改（至少 {LAG_REASON_CHARACTERS} 个字）")
            continue
        if len(anchor) < lag_anchor_characters:
            lag_problems.append(f"{place} 锚只有 {len(anchor)} 个字，要那一行里至少 {lag_anchor_characters} 个字的原文")
            continue
        entry_key = (os.path.normpath(registered_path), code, anchor)
        if entry_key in first_entry_of:
            lag_problems.append(f"{place} 与第 {first_entry_of[entry_key]} 行登记的是同一条（路径、C 号、锚都相同）")
            continue
        first_entry_of[entry_key] = entry_number
        if not os.path.isfile(registered_path):
            lag_problems.append(f"{place} 登记的路径不在：{registered_path}")
            continue
        if registered_path not in registered_lines:
            with open(registered_path, encoding="utf-8", errors="replace") as registered_file:
                registered_lines[registered_path] = registered_file.read().split("\n")
        anchored = [number for number, text in enumerate(registered_lines[registered_path], 1) if anchor in text]
        if len(anchored) != 1:
            shown = f"（第 {'、'.join(map(str, anchored[:5]))} 行）" if anchored else ""
            lag_problems.append(f"{place} 锚在 {registered_path} 里命中 {len(anchored)} 行{shown}，要恰好一行")
            continue
        anchored_text = registered_lines[registered_path][anchored[0] - 1]
        if code not in {found.group(0) for found in OWED_NUMBER.finditer(anchored_text)}:
            lag_problems.append(f"{place} 锚命中的 {registered_path}:{anchored[0]} 里没有 {code}")
            continue
        if code not in paid_names:
            lag_problems.append(f"{place} {code} 已不在 {owed_path} 的已还清表里（挪回了欠着的表，或表里没有这一笔）")
            continue
        lag_covers[(os.path.normpath(registered_path), anchored[0], code)] = entry_number
    if BREAK == "owed-refs-paid-lag-growth-unjudged":
        lag_note = "弄坏开关 owed-refs-paid-lag-growth-unjudged：没与 gate 基准比"
    elif not diff_base:
        lag_note = "被判的根不在 git 仓里，滞后表没与 gate 基准比"
    else:
        base_entries, base_problem = lag_baseline.baseline_rows(diff_base, lag_table_path, cwd=os.getcwd())
        if base_problem is not None:
            lag_problems.append(f"与 gate 基准比不了：{base_problem.sentence()}")
        elif base_entries is None:
            lag_note = f"gate 基准 {diff_base} 里还没有滞后表（初次登记），不比"
        else:
            base_keys = {(os.path.normpath(columns[0].strip()), columns[1].strip(), columns[2].strip())
                         for _number, columns in base_entries if len(columns) >= 3}
            for entry_number, columns in lag_entries:
                if len(columns) >= 3:
                    grown = (os.path.normpath(columns[0].strip()), columns[1].strip(), columns[2].strip())
                    if grown not in base_keys:
                        lag_growth.append((entry_number,) + grown)
            lag_note = f"与 gate 基准 {diff_base} 那一版比没涨（基准 {len(base_entries)} 行，此刻 {lag_row_count} 行）"

findings, lagged, status_findings, status_lagged = [], [], [], []
status_keys = {(os.path.normpath(path), line_number, code) for path, line_number, code, _unmarked in status_red_places}
for red_place in red_places:
    key = (os.path.normpath(red_place[0]), red_place[1], red_place[2])
    if key in status_keys:
        (status_lagged if key in lag_covers else status_findings).append(red_place)
    else:
        (lagged if key in lag_covers else findings).append(red_place)
red_keys = {(os.path.normpath(path), line_number, code) for path, line_number, code, _unmarked in red_places}

failed = False
if status_findings:
    failed = True
    for path, line_number, code, unmarked in status_findings:
        print(f"  ✗ {path}:{line_number} {code}（{paid_names[code]}）" + (f"，这一行里 {unmarked} 句都把它当现欠" if unmarked > 1 else ""))  # gate-lint:detail
    print(f"  ✗ 决策正文欠段之外与里程碑页的现状句里有 {len(status_findings)} 处引的账在 {owed_path} 的已还清表里、那一句把它当现欠"
          f"（写了{'、'.join(status_owed_words)}或「记在 C<编号>」）又没写{'、'.join(status_paid_marks)}之一"
          f"（扫 {len(status_files)} 份中文正文、{status_sentence_count} 句引账、{status_reference_count} 个 C 号；滞后表收着的 {len(status_lagged)} 处不算）")  # gate-lint:summary
    print("     → 怎么办：那一句写的是历史（当时欠、后来还了），就在那一句写明已还（「C<编号>（简称） 已还」）；写的是现状，就去掉把它当现欠的说法；"
          "那笔账其实没还清，就把 checks-owed.md 那一行挪回欠着的那张表；"
          "那份 kb 这一轮不许改的，照 .claude/gate.d/owed-refs-paid-lag.tsv 文件头登记一行（只缩不涨，判据在 doc-decisions.sh 里这一格函数上方的注释「现状句」一段）")
if findings:
    failed = True
    for path, line_number, code, unmarked in findings:
        print(f"  ✗ {path}:{line_number} {code}（{paid_names[code]}）" + (f"，这一行里 {unmarked} 次都没说还清" if unmarked > 1 else ""))  # gate-lint:detail
    print(f"  ✗ 决策正文的欠段里有 {len(findings)} 处引的账在 {owed_path} 的已还清表里、同一行里它的哪一截都没说还清了"
          f"（扫 {len(originals)} 份中文正文、{owed_line_count} 行欠段、{reference_count} 个 C 号；滞后表收着的 {len(lagged)} 处不算）")  # gate-lint:summary
    print("     → 怎么办：把那个 C<编号>（简称）连同只说它的那一截（括注、顿号、分号）从 **欠**： 行里去掉，去掉后这一行没有别的 C 号就写 **欠**：无。；")
    print("               那笔账其实没还清，就把 checks-owed.md 那一行挪回欠着的那张表；要留着这一处引用，就在同一截写明已还清（判据在 doc-decisions.sh 里这一格函数上方的注释）")
if lag_problems:
    failed = True
    print(f"  ✗ 滞后表 {lag_table_path} 有 {len(lag_problems)} 处不对：")  # gate-lint:summary
    for problem in lag_problems:
        print(f"     {problem}")  # gate-lint:detail
    print(f"     → 怎么办：一行「<路径><TAB>C<编号><TAB><那一行里至少 {lag_anchor_characters} 个字、在文件里只出现一次的原文><TAB># <为什么还没改，至少 {LAG_REASON_CHARACTERS} 个字>」；")
    print("               路径不在、锚找不到、那一行里没有这个号、号已挪回欠着的表的，多半是那一处已经改好或文件搬了家，删掉这一行；锚命中几行的换一段更长、只出现一次的原文。")
if lag_growth:
    failed = True
    print(f"  ✗ 滞后表比 gate 基准 {diff_base} 那一版多 {len(lag_growth)} 行（只缩不涨；路径、C 号或锚改了也算新加）：")  # gate-lint:summary
    for entry_number, path, code, anchor in lag_growth:
        print(f"     {lag_table_path}:{entry_number} {path} {code}  …{anchor}…")  # gate-lint:detail
    print("     → 怎么办：新写的欠段当场照判据改（去掉那一截或写明已还清），不往滞后表里加；那一行改过字、锚对不上了的，在同一次改动里把那一处改好、删掉它那一行，不改锚。")
for (path, line_number, code), entry_number in sorted(lag_covers.items(), key=lambda item: item[1]):
    if (path, line_number, code) not in red_keys:
        print(f"  ! 滞后表第 {entry_number} 行登记的 {path}:{line_number} {code} 此刻不红，删掉这一行（只缩不涨，删了不用补）")
if failed:
    sys.exit(1)
print(f"  ✓ 决策正文的欠段没有把已还清的账当开着的（扫 {len(originals)} 份中文正文、{owed_line_count} 行欠段、{reference_count} 个 C 号；"
      f"其中引已还清的 {paid_reference_count} 个：同一截写了{'、'.join(paid_marks)}之一的 {marked_in_segment} 个，"
      f"同一行里同一个号另一截写了的 {marked_elsewhere_in_line} 个；滞后 {len(lagged)} 处）")
print(f"    别的 C 号简称括注里的 C 号 {skipped_count} 个不算引账；滞后表 {lag_row_count} 行（{lag_note}）")
print(f"    没扫的：译文 {len(translations)} 份（以数字起头的 .en.md、.ja.md，欠段只认中文原文）")
if BREAK == "owed-refs-paid-status-sentences-skipped":
    print("    现状句：弄坏开关 owed-refs-paid-status-sentences-skipped，没判")
else:
    print(f"    现状句：扫 {len(status_files)} 份中文正文（决策 {len(originals)} 份的欠段之外、里程碑页 {len(milestone_originals)} 份），{status_sentence_count} 句引账、"
          f"{status_reference_count} 个 C 号；其中引已还清的 {status_paid_count} 个：那一句写了{'、'.join(status_paid_marks)}之一的 {status_marked_count} 个，"
          f"没写现欠说法的 {status_unclaimed_count} 个；滞后 {len(status_lagged)} 处")
PY
)

# ════ 登记的格函数：弄坏开关点名这一格就不跑、按判过了记，否则调这一格的判法 ════
# 前十四格的 stdin 接 /dev/null，与它们在后台跑时读到的一样。
break_skips() {
  if [[ "$BREAK_SWITCH" == "$1" ]]; then
    echo "  ✓ 弄坏开关 DOC_DECISIONS_BREAK=$1：这一格没判"
    exit 0
  fi
}
judge_kb_shape()                { break_skips kb-shape;                grid_kb_shape </dev/null; }
judge_item_ref_status()         { break_skips item-ref-status;         grid_item_ref_status </dev/null; }
judge_status_redundancy()       { break_skips status-redundancy;       grid_status_redundancy </dev/null; }
judge_cross_decision_status()   { break_skips cross-decision-status;   grid_cross_decision_status </dev/null; }
judge_settled_item_self_open()  { break_skips settled-item-self-open;  grid_settled_item_self_open </dev/null; }
judge_settled_ref_says_open()   { break_skips settled-ref-says-open;   grid_settled_ref_says_open </dev/null; }
judge_stale_open_items()        { break_skips stale-open-items;        grid_stale_open_items </dev/null; }
judge_settled_same_file()       { break_skips settled-same-file;       grid_settled_same_file </dev/null; }
judge_freeze_layer_membership() { break_skips freeze-layer-membership; grid_freeze_layer_membership </dev/null; }
judge_decision_items_sync()     { break_skips decision-items-sync;     grid_decision_items_sync </dev/null; }
judge_blocking_verdict()        { break_skips blocking-verdict;        grid_blocking_verdict </dev/null; }
judge_user_verdict_owed()       { break_skips user-verdict-owed;       grid_user_verdict_owed </dev/null; }
judge_decision_summary_width()  { break_skips decision-summary-width;  grid_decision_summary_width </dev/null; }
judge_write_frequency()         { break_skips write-frequency;         grid_write_frequency </dev/null; }
judge_entry_added()             { break_skips entry-added;             cell_entry_added; }
judge_shape()                   { break_skips shape;                   cell_shape; }
judge_history_loss()            { break_skips history-loss;            cell_history_loss; }
judge_candidate_codes()         { grid_candidate_codes </dev/null; }
judge_inference_needs_verdict() { break_skips inference-needs-verdict; grid_inference_needs_verdict </dev/null; }
judge_three_way_leg_record()    { break_skips three-way-leg-record;    grid_three_way_leg_record </dev/null; }
judge_owed_refs_paid()          { grid_owed_refs_paid </dev/null; }
