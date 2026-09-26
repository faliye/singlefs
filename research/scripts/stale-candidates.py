#!/usr/bin/env python3
"""阶段同步的候选表：这一阶段哪些事实变了，仓里还有哪些现状句在说旧的。回扫员照表逐行判，不自己挑关键词、不整组放行。

用法（按次序）：
    stale-candidates.py --changes --base 提交 [--target 提交] --out 变更清单.md
        列出这一阶段在 kb 各文件「## 历史版本」节里新加的每一条变更记录（编号 H1、H2……），以及每个被改过的现状载体
        里删改前后的差异片段——写事实表的原料
    stale-candidates.py --check-facts 事实表.tsv --base 提交 [--target 提交]
        核事实表罩没罩全：每条 H 编号都要出现在某一行的「出处」列里；每行的检索词要是合法的正则，
        而且在基准那一版的现状句里至少命中一行（旧说法当时就在那里）；新立事项反过来，检索词在基准那一版要零命中
    stale-candidates.py --facts 事实表.tsv --base 提交 [--target 提交] --out 候选.tsv
        按事实表的检索词在全部现状载体里搜，每处命中一行候选
    stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]
        核报告有没有判到候选表里（分到的组里）的每一行
    stale-candidates.py --benchmark     拿仓里一段真实历史与一份盲写的事实表当阳性对照
    stale-candidates.py --selftest

事实表（制表符分隔，首行是表头）：
    编号	旧事实	新事实	检索词	出处
    F1	层 0 的负载只有第一个事务	层 0 两条流：……	层 0|崩溃点重放|录制流	H3;H17;提交 cae5092
  检索词是 Python 正则，写这件事实涉及的概念名词（「层 0」「录制流」「多次挂载」），新旧两种说法都会提到的那种；
  不写状态词，不照抄新说法的原句（「按整条流切段」只搜得到改好了的句子）；宁宽勿窄。
  几个词要同时出现就用 && 连（「E142&&改动计数」）；一行事实的检索词在结束那一版命中不许超过 500 行（MAXIMUM_CANDIDATE_LINES_PER_FACT），超了就是太宽，用 && 收窄。
  出处写这条事实来自哪几条变更记录（H 编号，--changes 给的）或哪个提交。
  旧事实写「只改措辞」的行只用来罩住变更记录，不出候选。
  旧事实以「新立：」开头的行是这一阶段之前仓里一个字都没提过的东西（新立的欠账、新门禁阶段），也不出候选；
  实现落地、实验跑完、欠账还清不是新立，是事实变了：旧事实写仓里原来怎么说它没有 / 还欠，检索词写那件事的概念名词。

报告（--check-report 判的就是这个）：
    ## 逐行判定
    | 组 | 载体 | 判定 | 改后的句子或理由 |
  候选表里每一行都要有一行；判定以「要改」「要补」「事件句不改」「不相干」「要人看」之一开头；
  最后一格不能空：要改的写改后的句子，其余写理由（至少 8 个字）。几份报告合起来判。

管不到的（三方 sweep-rel-r1 第一轮攻方腿打中、机械闸补不上）：
  写表人给一件真变了的事实选一个基准里从没出现过的检索词、再标「新立：」，新事实里又避开落地 / 实现这类字，过得了闸，
  这件事实的旧说法一行都进不了候选——判别子是写表人自己选的词，闸看不到；靠主 agent 读事实表。
  逐行判的理由引了原话、判定也是五种之一，照样可以是没读懂就判的；靠主 agent 在全部判定里抽查复判。

退出码：0 通过；2 参数或 git 出错；3 报告漏判或判定格式不对；4 阳性对照漏了已知的腐烂；5 自证失败；6 事实表没罩全。
"""
import argparse
import difflib
import hashlib
import os
import re
import subprocess
import sys
import tempfile

CARRIER_PATTERNS = [
    r'^CLAUDE\.md$',
    r'^README\.md$',
    r'^\.claude/agent-common\.md$',
    r'^\.claude/main-agent\.md$',
    r'^\.claude/agents/[^/]+\.md$',
    r'^\.claude/rules/[^/]+\.md$',
    r'^\.claude/skills/[^/]+/SKILL\.md$',
    r'^\.claude/handover/[^/]+/README\.md$',
    r'^\.claude/kb/.+\.md$',
    r'^records/[^/]+\.md$',
]
CHANGE_LOG_ONLY_PATTERNS = [
    r'^\.claude/kb/decisions-history',
    r'^\.claude/kb/experiments-history\.md$',
]
HISTORY_SECTION_TITLE = '## 历史版本'
HISTORY_ENTRY_HEADING = re.compile(r'^### .+$', re.M)
LINE_VERDICTS = ('要改', '要补', '事件句不改', '不相干', '要人看')
MINIMUM_REASON_LENGTH = 8
FACT_TABLE_COLUMNS = ['编号', '旧事实', '新事实', '检索词', '出处']
NEWLY_ADDED_OLD_FACT_PREFIX = '新立：'
NOT_NEWLY_ADDED_WORDS = re.compile(r'落地|实现|已有|跑完|还清|有了')
REWORDING_ONLY_OLD_FACT_PREFIX = '只改措辞'
MAXIMUM_CANDIDATE_LINES_PER_FACT = 500
REWORDING_HEADING_WORDS = re.compile(r'措辞|指代|写法|改名|搬|路径|一个字(不|没)改|错字|拼写|简称|格式对齐')
MINIMUM_QUOTED_CHARACTERS = 4
TERM_CONJUNCTION = '&&'


def compile_search_term(term):
    """「A&&B」→ 每一段各编一个正则，一行里全都命中才算命中。"""
    return [re.compile(part) for part in term.split(TERM_CONJUNCTION)]


def search_term_matches(compiled_parts, line):
    return all(part.search(line) for part in compiled_parts)

# 阳性对照：2026-09-18 知识腐烂回扫里主 agent 逐条现查坐实的 25 处腐烂（research/prompts/knowledge-rot-2026-09-18-sync.md），
# 行号是 BENCHMARK_TARGET 那一版的。只存「载体:行号」的 sha256 前 16 位：回扫员读得到这个文件，明文会被照抄成答案。
# 事实表 BENCHMARK_FACTS 由一个没读过这 25 处的 agent 照这一段历史的提交正文与变更记录盲写（第四版，
# research/prompts/sweep-acceptance-2026-09-18-v2-facts.md）；2026-09-19 三方第一轮之后按新闸把 9 行多余的 && 机械地退回第一个词
# （research/prompts/sweep-rel-r1-main-verification.md），罩住 25 处。对照守两样：罩住的不少于这个数、候选行数与冻结时相同。
# 答案在仓里另有明文（那份同步记录），对照只防工具被改窄，不当盲测用。
BENCHMARK_BASE = 'b1c8cef~1'
BENCHMARK_TARGET = '00c9d4f'
BENCHMARK_FACTS = 'research/scripts/fixtures/stale-candidates-benchmark-facts.tsv'
BENCHMARK_MINIMUM_COVERED = 25
BENCHMARK_EXPECTED_CANDIDATE_ROWS = 4190
BENCHMARK_EXPECTED_UNIQUE_LINES = 2821
BENCHMARK_KNOWN_STALE_LOCATION_HASHES = [
    '88600dee84e43cb1', '603dc36527671cbc', 'a93c0915fab7efa8', '948054ed6df42b0b', '7bb23ca4ed5f8432',
    '6287ca8bd47723bf', '201247a903f206ae', '16403eb11c52d13c', '82e4f62b752e4383', '84b95c231a2692a3',
    '767dc5b0a2eed64f', '43cec00ac6c7e52f', '7ec30e7996379b13', 'ef8cdf810547c225', '799d9c319db094c0',
    '8eb3cae28cc99670', 'bbed8920622afcc2', '4af43929ce1d4f2b', '9919777fff2f4391', '7c5fdaebc4e022d3',
    '873c19307057473f', '8dfe08b83d4f054f', '86b893c315e5795e', 'a3ae13e499fa522c', 'b4e110d8f010be83',
]


def normalize(text):
    """去掉空白、强调与表格记号，比原话时不受排版影响。"""
    return re.sub(r'[\s*`>#|]+', '', text)


def location_hash(location):
    return hashlib.sha256(location.encode('utf-8')).hexdigest()[:16]


def matches_any(path, patterns):
    return any(re.search(pattern, path) for pattern in patterns)


def is_current_state_carrier(path):
    return matches_any(path, CARRIER_PATTERNS) and not matches_any(path, CHANGE_LOG_ONLY_PATTERNS)


def run_git(arguments, repository_root):
    completed = subprocess.run(['git', '-C', repository_root, '-c', 'core.quotepath=false', *arguments],
                               capture_output=True, text=True)
    if completed.returncode != 0:
        raise RuntimeError(f'git {" ".join(arguments)} 失败：{completed.stderr.strip()}')
    return completed.stdout


def read_version(repository_root, revision, path):
    """revision 为 None 读工作区；文件不在返回空串。"""
    if revision is None:
        full_path = os.path.join(repository_root, path)
        if not os.path.isfile(full_path):
            return ''
        with open(full_path, encoding='utf-8', errors='replace') as handle:
            return handle.read()
    completed = subprocess.run(['git', '-C', repository_root, 'show', f'{revision}:{path}'],
                               capture_output=True, text=True)
    return completed.stdout if completed.returncode == 0 else ''


def all_paths(repository_root, revision):
    if revision is None:
        paths = (run_git(['ls-files'], repository_root).split('\n')
                 + run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n'))
    else:
        paths = run_git(['ls-tree', '-r', '--name-only', revision], repository_root).split('\n')
    return sorted({path for path in paths if path})


def changed_paths(repository_root, base, target):
    paths = run_git(['diff', '--name-only', base] + ([target] if target else []), repository_root).split('\n')
    if target is None:
        paths += run_git(['ls-files', '--others', '--exclude-standard'], repository_root).split('\n')
    return sorted({path for path in paths if path})


def current_state_lines(text):
    """历史版本节之前的行，(行号, 原文)。"""
    lines = []
    for line_number, line in enumerate(text.split('\n'), start=1):
        if line.strip() == HISTORY_SECTION_TITLE:
            break
        lines.append((line_number, line))
    return lines


def history_headings(text, path):
    """变更记录的标题：普通 kb 文件取「## 历史版本」之后的三级标题；变更史文件整份都是变更记录。"""
    if matches_any(path, CHANGE_LOG_ONLY_PATTERNS):
        section = text
    else:
        start = text.find('\n' + HISTORY_SECTION_TITLE)
        if start < 0:
            return []
        section = text[start:]
    return HISTORY_ENTRY_HEADING.findall(section)


def renamed_from(repository_root, base, target):
    """{新路径: 旧路径}：搬了目录、改了名的文件，历史标题要拿旧路径那一版去比。"""
    output = run_git(['diff', '--name-status', '-M', base] + ([target] if target else []), repository_root)
    renames = {}
    for line in output.split('\n'):
        columns = line.split('\t')
        if len(columns) == 3 and columns[0].startswith('R'):
            renames[columns[2]] = columns[1]
    return renames


def new_change_entries(repository_root, base, target):
    """[(H 编号, 文件, 标题)]，按文件与出现次序编号，同一 base / target 下稳定。"""
    entries = []
    renames = renamed_from(repository_root, base, target)
    for path in changed_paths(repository_root, base, target):
        if not path.startswith('.claude/kb/') or not path.endswith('.md'):
            continue
        base_path = renames.get(path, path)
        before = set(history_headings(read_version(repository_root, base, base_path), base_path))
        for heading in history_headings(read_version(repository_root, target, path), path):
            if heading not in before:
                entries.append((path, heading))
    return [(f'H{index}', path, heading) for index, (path, heading) in enumerate(entries, start=1)]


def changed_segments(base_text, target_text):
    """删改前后的差异片段 [(旧片段, 新片段)]：每一行被删的现状句配上同一文件新加的最像的一行，按字符比出改掉的那几段。"""
    base_lines = [line for _, line in current_state_lines(base_text)]
    target_lines = [line for _, line in current_state_lines(target_text)]
    target_set, base_set = set(target_lines), set(base_lines)
    added = [line for line in target_lines if line not in base_set and line.strip()]
    segments = []
    for removed in (line for line in base_lines if line not in target_set and line.strip()):
        replacement = max(added, default='', key=lambda candidate: difflib.SequenceMatcher(
            None, removed, candidate, autojunk=False).quick_ratio())
        matcher = difflib.SequenceMatcher(None, removed, replacement, autojunk=False)
        for operation, old_start, old_end, new_start, new_end in matcher.get_opcodes():
            if operation in ('replace', 'delete') and len(removed[old_start:old_end].strip()) >= 2:
                segments.append((removed[max(0, old_start - 12):old_end + 12].strip(),
                                 replacement[max(0, new_start - 12):new_end + 12].strip()))
    return segments


def write_changes(repository_root, base, target, output):
    output.write(f'# 变更清单 {base}..{target or "工作区"}\n\n## 变更记录（事实表的「出处」列要罩住每一个 H 编号）\n\n')
    output.write('| 编号 | 文件 | 标题 |\n|---|---|---|\n')
    for entry_id, path, heading in new_change_entries(repository_root, base, target):
        output.write(f'| {entry_id} | {path} | {heading[4:].replace("|", "｜")} |\n')
    output.write('\n## 现状载体里改掉的片段\n\n')
    for path in changed_paths(repository_root, base, target):
        if not is_current_state_carrier(path):
            continue
        segments = changed_segments(read_version(repository_root, base, path), read_version(repository_root, target, path))
        if not segments:
            continue
        output.write(f'### {path}\n\n')
        for old_segment, new_segment in segments:
            output.write(f'- 旧：{old_segment[:200]}\n  新：{new_segment[:200]}\n')
        output.write('\n')


def read_fact_table(path):
    with open(path, encoding='utf-8') as handle:
        rows = [line.rstrip('\n').split('\t') for line in handle if line.strip()]
    if not rows or rows[0][:len(FACT_TABLE_COLUMNS)] != FACT_TABLE_COLUMNS:
        raise ValueError(f'{path} 的表头要是：{" / ".join(FACT_TABLE_COLUMNS)}（制表符分隔）')
    facts = []
    for row in rows[1:]:
        if len(row) < len(FACT_TABLE_COLUMNS):
            raise ValueError(f'{path} 这一行少列：{row[0] if row else "（空）"}')
        facts.append(dict(zip(FACT_TABLE_COLUMNS, row)))
    return facts


def current_state_corpus(repository_root, target):
    return {path: current_state_lines(read_version(repository_root, target, path))
            for path in all_paths(repository_root, target) if is_current_state_carrier(path)}


def build_candidates(repository_root, target, facts):
    """[(组号, 旧事实, 新事实, 载体:行号, 原文)]。"""
    corpus = current_state_corpus(repository_root, target)
    candidates = []
    for fact in facts:
        if fact['旧事实'].startswith((REWORDING_ONLY_OLD_FACT_PREFIX, NEWLY_ADDED_OLD_FACT_PREFIX)):
            continue
        compiled_parts = compile_search_term(fact['检索词'])
        for path, lines in corpus.items():
            for line_number, line in lines:
                if search_term_matches(compiled_parts, line):
                    candidates.append((fact['编号'], fact['旧事实'], fact['新事实'], f'{path}:{line_number}', line.strip()))
    return candidates


def write_candidates(candidates, output):
    output.write('组\t旧事实\t新事实\t载体\t原文\n')
    for group, old_fact, new_fact, location, line in candidates:
        output.write(f'{group}\t{old_fact}\t{new_fact}\t{location}\t{line.replace(chr(9), " ")[:500]}\n')


def check_facts(repository_root, base, target, fact_path):
    try:
        facts = read_fact_table(fact_path)
    except ValueError as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：照本脚本文件头「事实表」那一段的格式写，五列用制表符分开')
        return 6
    entries = new_change_entries(repository_root, base, target)
    cited = set()
    for fact in facts:
        cited.update(re.findall(r'H\d+', fact['出处']))
    uncovered = [(entry_id, path, heading) for entry_id, path, heading in entries if entry_id not in cited]
    base_corpus = current_state_corpus(repository_root, base)
    target_corpus = current_state_corpus(repository_root, target)
    heading_by_id = {entry_id: heading for entry_id, _, heading in entries}
    broken, empty, too_broad, not_new, fake_rewording, needless_conjunction, gone = [], [], [], [], [], [], []
    for fact in facts:
        try:
            compiled_parts = compile_search_term(fact['检索词'])
        except re.error as error:
            broken.append(f'{fact["编号"]}（{error}）')
            continue
        if fact['旧事实'].startswith(REWORDING_ONLY_OLD_FACT_PREFIX):
            cited_headings = [heading_by_id[entry_id] for entry_id in re.findall(r'H\d+', fact['出处']) if entry_id in heading_by_id]
            if any(not REWORDING_HEADING_WORDS.search(heading) for heading in cited_headings):
                fake_rewording.append(fact['编号'])
            continue
        if fact['旧事实'].startswith(NEWLY_ADDED_OLD_FACT_PREFIX):
            base_hits = any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines)
            if base_hits or NOT_NEWLY_ADDED_WORDS.search(fact['新事实']):
                not_new.append(fact['编号'])
            continue
        target_hits = sum(1 for lines in target_corpus.values() for _, line in lines
                          if search_term_matches(compiled_parts, line))
        if target_hits > MAXIMUM_CANDIDATE_LINES_PER_FACT:
            too_broad.append(f'{fact["编号"]}（{target_hits} 行）')
        if target_hits == 0:
            gone.append(fact['编号'])
        if len(compiled_parts) > 1:
            first_part_hits = sum(1 for lines in target_corpus.values() for _, line in lines if compiled_parts[0].search(line))
            if first_part_hits <= MAXIMUM_CANDIDATE_LINES_PER_FACT:
                needless_conjunction.append(f'{fact["编号"]}（第一个词只命中 {first_part_hits} 行）')
        if not any(search_term_matches(compiled_parts, line) for lines in base_corpus.values() for _, line in lines):
            empty.append(fact['编号'])
    if uncovered or broken or empty or too_broad or not_new or fake_rewording or needless_conjunction or gone:
        for entry_id, path, heading in uncovered[:40]:
            print(f'  ✗ {entry_id} 没有任何一行事实罩着：{path} {heading[4:80]}')  # gate-lint:detail
        if broken:
            print(f'  ✗ 检索词不是合法的正则：{"、".join(broken)}')  # gate-lint:detail
        if empty:
            print(f'  ✗ 检索词在基准那一版的现状句里一处都没命中（多半照抄了新说法）：{"、".join(empty)}')  # gate-lint:detail
        if not_new:
            print(f'  ✗ 标成「新立：」却不是新东西（检索词在基准那一版有命中，或新事实写着落地 / 实现 / 已有 / 跑完 / 还清）：'  # gate-lint:detail
                  f'{"、".join(not_new)}')
        if fake_rewording:
            print(f'  ✗ 标成「只改措辞」的行引的变更记录，标题里看不出只改了措辞：{"、".join(fake_rewording)}')  # gate-lint:detail
        if needless_conjunction:
            print(f'  ✗ 用了 && 而第一个词自己不超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行，不许再收窄：{"、".join(needless_conjunction)}')  # gate-lint:detail
        if gone:
            print(f'  ✗ 检索词在结束那一版一处都没命中（照抄了被这一阶段改掉的旧句）：{"、".join(gone)}')  # gate-lint:detail
        if too_broad:
            print(f'  ✗ 检索词太宽，结束那一版命中超过 {MAXIMUM_CANDIDATE_LINES_PER_FACT} 行：{"、".join(too_broad)}')  # gate-lint:detail
        print(f'  ✗ 事实表没罩全：{len(uncovered)} 条变更记录没罩、{len(broken)} 行正则坏了、'  # gate-lint:summary
              f'{len(empty)} 行检索词在基准零命中、{len(gone)} 行在结束零命中、{len(too_broad)} 行检索词太宽、'
              f'{len(needless_conjunction)} 行多余的 &&、{len(not_new)} 行冒充新立、{len(fake_rewording)} 行冒充只改措辞')
        print('     → 怎么办：读 --changes 给的变更清单，每条 H 编号写进它说的那件事实的「出处」列（只改措辞的也写一行，旧新事实写「只改措辞」）；'
              '检索词写新旧说法都会提到的概念名词，基准与结束两版都要命中；太宽（超过上限）才用 && 并上第二个概念名词收窄；'
              '事实真变了的不许写成只改措辞或新立')
        return 6
    print(f'  ✓ 事实表罩全了：{len(entries)} 条变更记录都有出处，{len(facts)} 行事实的检索词都在基准那一版的现状句里命中（新立事项除外、它们在基准零命中）')
    return 0


def parse_group_ranges(text):
    """「F1-F12,F15」→ 组号集合；None 表示全部。"""
    if not text:
        return None
    selected = set()
    for part in text.split(','):
        match = re.fullmatch(r'([A-Z])(\d+)(?:-[A-Z]?(\d+))?', part.strip())
        if not match:
            if not re.fullmatch(r'[A-Z]\w*', part.strip()):
                raise ValueError(f'组号区间写不认得：{part}')
            selected.add(part.strip())
            continue
        low, high = int(match.group(2)), int(match.group(3) or match.group(2))
        if high < low or re.fullmatch(r'[A-Z]\d+-([A-Z])\d+', part.strip()) and part.strip().split('-')[1][0] != match.group(1):
            raise ValueError(f'组号区间写不认得（倒写，或两端字母不同）：{part}')
        selected.update(f'{match.group(1)}{number}' for number in range(low, high + 1))
    return selected


def table_rows_under(title, text):
    """某个二级标题下表格的数据行（去掉表头与分隔行），每行拆成单元格。"""
    rows, inside = [], False
    for line in text.split('\n'):
        if line.startswith('## '):
            inside = line.strip() == title
            continue
        if inside and line.startswith('|') and not re.match(r'^\|\s*-', line):
            rows.append([cell.strip().strip('`') for cell in line.strip().strip('|').split('|')])
    return [row for row in rows if row and row[0] != '组']


def read_candidate_rows(table_path):
    """[((组号, 载体:行号), 原文)]。"""
    rows = []
    with open(table_path, encoding='utf-8') as handle:
        next(handle)
        for row in handle:
            columns = row.rstrip('\n').split('\t')
            rows.append(((columns[0], columns[3]), columns[4] if len(columns) > 4 else ''))
    return rows


def quotes_the_line(reason, line):
    """理由里有没有这一行的原话（去掉空白与强调记号之后，至少 MINIMUM_QUOTED_CHARACTERS 个连续字符）。"""
    normalized_line, normalized_reason = normalize(line), normalize(reason)
    if len(normalized_line) < MINIMUM_QUOTED_CHARACTERS:
        return True
    return any(normalized_line[start:start + MINIMUM_QUOTED_CHARACTERS] in normalized_reason
               for start in range(len(normalized_line) - MINIMUM_QUOTED_CHARACTERS + 1))


def check_reports(table_path, report_paths, selected_groups=None):
    candidate_rows = read_candidate_rows(table_path)
    if selected_groups is not None:
        unknown = sorted(selected_groups - {key[0] for key, _ in candidate_rows})
        if unknown:
            print(f'  ✗ --groups 里有候选表没有的组：{"、".join(unknown[:20])}')
            print('     → 怎么办：组号照候选表第一列写，区间两端同一个字母、小号在前')
            return 2
        candidate_rows = [row for row in candidate_rows if row[0][0] in selected_groups]
    keys = [key for key, _ in candidate_rows]
    text_by_key = dict(candidate_rows)
    verdicts = {}
    for report_path in report_paths:
        with open(report_path, encoding='utf-8') as handle:
            for cells in table_rows_under('## 逐行判定', handle.read()):
                if len(cells) >= 3:
                    verdicts[(cells[0], cells[1])] = (cells[2], cells[3] if len(cells) > 3 else '')
    missing = [key for key in keys if key not in verdicts]
    malformed = []
    for key in keys:
        if key not in verdicts:
            continue
        verdict, reason = verdicts[key]
        if (not verdict.startswith(LINE_VERDICTS) or len(reason) < MINIMUM_REASON_LENGTH
                or not quotes_the_line(reason, text_by_key.get(key, ''))):
            malformed.append(key)
    if missing or malformed:
        if missing:
            print(f'  ✗ {len(missing)} 行候选没有逐行判定，例：'  # gate-lint:detail
                  + '；'.join(f'{group} {location}' for group, location in missing[:10]))
        if malformed:
            print(f'  ✗ {len(malformed)} 行的判定不以五种之一开头、最后一格不到 {MINIMUM_REASON_LENGTH} 个字、'  # gate-lint:detail
                  f'或最后一格没引这一行至少 {MINIMUM_QUOTED_CHARACTERS} 个字的原话，例：'
                  + '；'.join(f'{group} {location}' for group, location in malformed[:10]))
        print(f'  ✗ 报告没判全：共 {len(keys)} 行候选')  # gate-lint:summary
        print('     → 怎么办：在「## 逐行判定」表里给每一行候选一行 | 组 | 载体 | 判定 | 改后的句子或理由 |，'
              '判定以 要改 / 要补 / 事件句不改 / 不相干 / 要人看 开头，最后一格写改后的句子或理由，理由里引这一行的原话')
        return 3
    print(f'  ✓ 报告判全了：{len(keys)} 行候选都有逐行判定')
    return 0


def benchmark(repository_root):
    facts_path = os.path.join(repository_root, BENCHMARK_FACTS)
    if not os.path.isfile(facts_path):
        print(f'  ✗ 缺阳性对照的事实表 {BENCHMARK_FACTS}')
        print('     → 怎么办：它随仓走；被删了就从 git 历史里取回，不要照着答案重写')
        return 4
    facts = read_fact_table(facts_path)
    candidates = build_candidates(repository_root, BENCHMARK_TARGET, facts)
    covered = {location_hash(location) for _, _, _, location, _ in candidates}
    covered_known = sum(1 for known in BENCHMARK_KNOWN_STALE_LOCATION_HASHES if known in covered)
    unique_lines = len({location for _, _, _, location, _ in candidates})
    if (len(candidates), unique_lines) != (BENCHMARK_EXPECTED_CANDIDATE_ROWS, BENCHMARK_EXPECTED_UNIQUE_LINES):
        print(f'  ✗ 阳性对照的候选数变了：{len(candidates)} 行、{unique_lines} 个不同的行，冻结时是 '
              f'{BENCHMARK_EXPECTED_CANDIDATE_ROWS} 行、{BENCHMARK_EXPECTED_UNIQUE_LINES} 个不同的行')
        print('     → 怎么办：同一份事实表、同一段历史，候选只该随候选规则变；是有意改了规则就重数并改这两个常量，写明为什么变')
        return 4
    if covered_known < BENCHMARK_MINIMUM_COVERED:
        print(f'  ✗ 阳性对照只罩住 {covered_known} 处已知的腐烂，低于 {BENCHMARK_MINIMUM_COVERED}（{BENCHMARK_BASE}..{BENCHMARK_TARGET}，只存哈希）')
        print('     → 怎么办：候选规则或盲写的事实表被改窄了；看 build_candidates 与 current_state_corpus 的改动，'
              '或那份事实表的检索词是不是被收窄过')
        return 4
    print(f'  ✓ 阳性对照：已知的 {len(BENCHMARK_KNOWN_STALE_LOCATION_HASHES)} 处腐烂里 {covered_known} 处在候选里（下限 {BENCHMARK_MINIMUM_COVERED}）'
          f'（盲写事实表 {len(facts)} 行，候选 {len(candidates)} 行、{unique_lines} 个不同的行）')
    return 0


def selftest():
    with tempfile.TemporaryDirectory() as directory:
        def git(*arguments):
            subprocess.run(['git', '-C', directory, *arguments], check=True, capture_output=True)

        def write(relative_path, content):
            full_path = os.path.join(directory, relative_path)
            os.makedirs(os.path.dirname(full_path), exist_ok=True)
            with open(full_path, 'w', encoding='utf-8') as handle:
                handle.write(content)

        def quietly(function, *arguments):
            standard_output = sys.stdout
            with open(os.devnull, 'w') as silent:
                sys.stdout = silent
                try:
                    return function(*arguments)
                finally:
                    sys.stdout = standard_output

        git('init', '-q')
        git('config', 'user.email', 'selftest@example.invalid')
        git('config', 'user.name', 'selftest')
        write('CLAUDE.md', '# 项目\n\n层 0 的负载还只有第一个事务。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('.claude/kb/table-before-move.md', '# 字节表\n\n' + '字段一行。\n' * 20 + '\n## 历史版本\n\n### 2026-08-30：搬家之前就有的条目\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'base')
        os.makedirs(os.path.join(directory, '.claude/kb/layout'), exist_ok=True)
        os.rename(os.path.join(directory, '.claude/kb/table-before-move.md'),
                  os.path.join(directory, '.claude/kb/layout/01-first-txn.md'))
        write('CLAUDE.md', '# 项目\n\n层 0 的负载是两条流。\n')
        write('.claude/kb/checks-owed.md', '| C1（某检查） | 前置：层 0 只有一次挂载 |\n\n## 历史版本\n\n'
              '### 2026-09-02：层 0 扩成两条流\n\n### 2026-09-01：旧条目\n\n层 0 只有一次挂载\n')
        write('records/2026-01-01-log.md', '层 0 只有一次挂载的时候写的记录。\n')
        git('add', '-A')
        git('commit', '-q', '-m', 'stage')
        write('records/2026-01-02-draft.md', '层 0 只有一次挂载（还没进 git 的草稿）。\n')
        failures = []
        entries = new_change_entries(directory, 'HEAD~1', 'HEAD')
        if [heading for _, _, heading in entries] != ['### 2026-09-02：层 0 扩成两条流']:
            failures.append(f'新增变更记录没认对（改名的文件要按旧路径比）：{entries}')
        segments = changed_segments(read_version(directory, 'HEAD~1', 'CLAUDE.md'), read_version(directory, 'HEAD', 'CLAUDE.md'))
        if not any('只有第一个事务' in old for old, _ in segments):
            failures.append('差异片段里没有被改掉的「只有第一个事务」')
        header = '\t'.join(FACT_TABLE_COLUMNS) + '\n'
        good_row = 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0\tH1\n'
        red_tables = {
            'facts-uncovered.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0\t提交 abc\n', '漏了 H1'),
            'facts-empty.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t不存在的词\tH1\n', '检索词零命中'),
            'facts-new-wording.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t两条流\tH1\n', '检索词照抄新说法（基准零命中）'),
            'facts-gone.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t负载还只有\tH1\n', '检索词照抄被改掉的旧句（结束零命中）'),
            'facts-fake-new.tsv': (header + good_row + 'F2\t新立：层 0\t层 0 两条流\t层 0\tH1\n', '冒充新立（基准有命中）'),
            'facts-fake-new-words.tsv': (header + good_row + 'F2\t新立：两条流\t层 0 两条流落地\t两条流\tH1\n', '冒充新立（新事实写着落地）'),
            'facts-fake-rewording.tsv': (header + good_row + 'F2\t只改措辞\t只改措辞\t层 0\tH1\n', '冒充只改措辞（变更标题看不出改措辞）'),
            'facts-needless-and.tsv': (header + 'F1\t层 0 只有第一个事务\t层 0 两条流\t层 0&&一次挂载\tH1\n', '第一个词不宽却用了 &&'),
        }
        write('facts-good.tsv', header + good_row)
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 0:
            failures.append('写对了的事实表被判红')
        for name, (content, what) in red_tables.items():
            write(name, content)
            if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, name)) != 6:
                failures.append(f'{what}的事实表没被判红')
        saved_limit = globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT']
        globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT'] = 0
        if quietly(check_facts, directory, 'HEAD~1', 'HEAD', os.path.join(directory, 'facts-good.tsv')) != 6:
            failures.append('命中行数超上限的事实表没被判红')
        globals()['MAXIMUM_CANDIDATE_LINES_PER_FACT'] = saved_limit
        write('facts-build.tsv', header + good_row + 'F2\t只改措辞\t只改措辞\t层 0\tH1\n'
              'F3\t层 0 只有第一个事务\t层 0 两条流\t一次挂载\tH1\n')
        candidates = build_candidates(directory, 'HEAD', read_fact_table(os.path.join(directory, 'facts-build.tsv')))
        table_path = os.path.join(directory, 'candidates.tsv')
        with open(table_path, 'w', encoding='utf-8') as handle:
            write_candidates(candidates, handle)
        written = [key for key, _ in read_candidate_rows(table_path)]
        if ('F1', '.claude/kb/checks-owed.md:1') not in written:
            failures.append('还说「层 0 只有一次挂载」的现状句没进候选')
        if ('F1', 'records/2026-01-01-log.md:1') not in written:
            failures.append('records/ 下的现状句没写进候选表')
        if [location for group, location in written].count('.claude/kb/checks-owed.md:1') != 2:
            failures.append('同一行命中两件事实，候选表里没有各占一行')
        if any(group == 'F2' for group, _ in written):
            failures.append('只改措辞的行出了候选')
        if any(location.startswith('.claude/kb/checks-owed.md:') and location != '.claude/kb/checks-owed.md:1'
               for _, location in written):
            failures.append('历史版本节里的句子进了候选')
        command_output = os.path.join(directory, 'candidates-from-command.tsv')
        subprocess.run([sys.executable, os.path.abspath(__file__), '--facts', os.path.join(directory, 'facts-build.tsv'),
                        '--base', 'HEAD~1', '--target', 'HEAD', '--out', command_output],
                       cwd=directory, capture_output=True, text=True)
        if not os.path.isfile(command_output) or read_candidate_rows(command_output) != read_candidate_rows(table_path):
            failures.append('--facts 命令写出的候选表与 build_candidates 的结果不一致')
        worktree_locations = [row[3] for row in build_candidates(directory, None, read_fact_table(os.path.join(directory, 'facts-good.tsv')))]
        if 'records/2026-01-02-draft.md:1' not in worktree_locations:
            failures.append('结束在工作区时，没进 git 的新文件没进候选')
        if parse_group_ranges('F1-F2') != {'F1', 'F2'}:
            failures.append('组号区间没把两端都算进去')
        for bad_range in ('F2-F1', 'F1-G2'):
            try:
                parse_group_ranges(bad_range)
                failures.append(f'组号区间 {bad_range} 没被拒绝')
            except ValueError:
                pass
        rows = read_candidate_rows(table_path)
        def report(verdict, reason_of):
            return ('## 逐行判定\n| 组 | 载体 | 判定 | 改后的句子或理由 |\n|---|---|---|---|\n'
                    + ''.join(f'| {group} | {location} | {verdict} | {reason_of(line)} |\n' for (group, location), line in rows))
        write('complete.md', report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成层 0 的负载是两条流'))
        write('missing.md', report('要改', lambda line: '原话「' + line.replace('|', ' ') + '」改成两条流').rstrip('\n').rsplit('\n', 1)[0] + '\n')
        write('unknown-verdict.md', report('同意', lambda line: '原话「' + line.replace('|', ' ') + '」照旧'))
        write('short-reason.md', report('不相干', lambda line: normalize(line)[:7]))
        write('no-quote.md', report('不相干', lambda line: '这一行说的是别的事情，与这件事实无关，不用改'))
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')]) != 0:
            failures.append('判全了的报告被判漏')
        for name, what in (('missing.md', '少判一行'), ('unknown-verdict.md', '判定不是五种之一'),
                           ('short-reason.md', '理由只有 7 个字'), ('no-quote.md', '理由没引这一行的原话')):
            if quietly(check_reports, table_path, [os.path.join(directory, name)]) != 3:
                failures.append(f'{what}的报告没被判红')
        if quietly(check_reports, table_path, [os.path.join(directory, 'complete.md')], {'F9'}) != 2:
            failures.append('--groups 里写了候选表没有的组，没被拒绝')
    if failures:
        for failure in failures:
            print(f'  ✗ 自证失败：{failure}')  # gate-lint:detail
        print('  ✗ stale-candidates 自证没过')  # gate-lint:summary
        print('     → 怎么办：对照 selftest 里造的小仓，逐格看 new_change_entries、changed_segments、check_facts、'
              'build_candidates、check_reports 哪一支被改了')
        return 5
    print('  ✓ stale-candidates 自证通过：变更记录（含改名）与差异片段认对；事实表八种写法各判红、写对的判绿、超上限判红；'
          '候选收 records/ 与没进 git 的文件、同一行两件事实各占一行、只改措辞不出候选、历史版本节不进；'
          '组号区间含两端、倒写与跨字母拒绝、写了不存在的组拒绝；报告漏判、判定不认得、理由太短、理由没引原话都判红，--facts 命令写出的表与函数结果一致（27 格）')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    parser.add_argument('--base')
    parser.add_argument('--target')
    parser.add_argument('--out')
    parser.add_argument('--changes', action='store_true')
    parser.add_argument('--check-facts', metavar='事实表')
    parser.add_argument('--facts', metavar='事实表')
    parser.add_argument('--check-report', nargs='+', metavar='文件')
    parser.add_argument('--groups')
    parser.add_argument('--benchmark', action='store_true')
    parser.add_argument('--selftest', action='store_true')
    arguments = parser.parse_args()
    if arguments.selftest:
        return selftest()
    try:
        repository_root = run_git(['rev-parse', '--show-toplevel'], os.getcwd()).strip()
        if arguments.benchmark:
            return benchmark(repository_root)
        if arguments.check_report:
            if len(arguments.check_report) < 2:
                print('  ✗ --check-report 要候选表与至少一份报告')
                print('     → 怎么办：stale-candidates.py --check-report 候选.tsv 报告.md [报告.md …] [--groups F1-F12]')
                return 2
            return check_reports(arguments.check_report[0], arguments.check_report[1:], parse_group_ranges(arguments.groups))
        if not arguments.base:
            print('  ✗ 缺 --base')
            print('     → 怎么办：给阶段开始之前的提交，例：--base b1c8cef~1；用法见本脚本文件头')
            return 2
        if arguments.check_facts:
            return check_facts(repository_root, arguments.base, arguments.target, arguments.check_facts)
        if not arguments.out:
            print('  ✗ 缺 --out')
            print('     → 怎么办：--changes 与 --facts 都要 --out 指一个新文件（排他新建）')
            return 2
        if not arguments.changes and not arguments.facts:
            print('  ✗ 缺 --changes 或 --facts')
            print('     → 怎么办：先 --changes 出变更清单、写事实表、--check-facts 过了，再 --facts 出候选表')
            return 2
        with open(arguments.out, 'x', encoding='utf-8') as output:
            if arguments.changes:
                write_changes(repository_root, arguments.base, arguments.target, output)
                entry_total = len(new_change_entries(repository_root, arguments.base, arguments.target))
                print(f'  ✓ 变更清单 {arguments.out}：{entry_total} 条变更记录')
                return 0
            candidates = build_candidates(repository_root, arguments.target, read_fact_table(arguments.facts))
            write_candidates(candidates, output)
        print(f'  ✓ 候选表 {arguments.out}：{len(candidates)} 行（{len({row[3] for row in candidates})} 个不同的行）')
        return 0
    except (RuntimeError, ValueError) as error:
        print(f'  ✗ {error}')
        print('     → 怎么办：确认 --base / --target 是这个仓里存在的提交、事实表格式照文件头')
        return 2


if __name__ == '__main__':
    sys.exit(main())
