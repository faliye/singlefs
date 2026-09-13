#!/usr/bin/env python3
"""比对「八、根槽写路径的段序列登记表」与 E142 产物的 `name=segments` 行，逐字核对。

用法：
    check-segment-registry.py [--root 仓库根目录]
    check-segment-registry.py --selftest

为什么要有它：`.claude/kb/checks-owed.md` C316（提交步骤的登记位有四处且互不相同） 收窄之后
剩两半，第①半是「kb 登记表八与产物 `name=segments` 行之间没有逐字比对（表是人抄的）」——
`.claude/kb/first-txn-layout.md` 「八、根槽写路径的段序列登记表」里每一行的段序列数字串
（`4+1+1+1+2` 这种）都是人从 E142（第一个事务的干跑） 产物里抄过来的，抄错一位、
或者产物重跑之后表没跟着改，都没有任何东西会报警。这个脚本把「抄的对不对」变成一条会红的检查。

比对哪几行、path 怎么定：
- 表里前三条数据行（mkfs 种根 / 空发布（暖机） / 普通发布）各自在「出处」栏写着
  `产物 `name=segments path=<名字>`` ，这个 path 就是从那一栏原样解析出来的，不是另抄一份写死的映射表。
- 「实例切换 / 管理员回退」「抬 F 的空发布」两行标了 **预想**（没写成字节），跳过，计入跳过数。
- 表下面那句 ⚠️ 提示（「发布与下一次发布之间没有屏障……整条流 `…`、NNN 个状态」）是第四条产物路径
  （`post_mkfs_stream`）唯一的登记位——它不在表格里，是一句独立的提示。这个脚本按八节开头
  那句「产物的 `name=segments` 四行（`path=mkfs / warm_up / transaction / post_mkfs_stream`）」声明的
  四个 path，减去前三条数据行已经占用的三个，剩下那一个按排除法归给这句提示；
  减出来的路径集合大小若不是恰好一个，直接判成结构性错误（八节的写法变了，这段解析要跟着改），
  不猜、不悄悄跳过。
- 产物路径本身也不写死：从 `research/scripts/replay.sh` 的复跑登记表里找 `E142|...` 那一行，
  取它的「入库产物」那一列，这样产物重跑之后换了文件名，这个脚本自动跟着换，不用改代码。

自证会红：`--selftest` 把当前这份 first-txn-layout.md 与 replay.sh 指向的真实产物，
分别拷进两个临时目录（`green/` 原样、`red/` 改坏了 mkfs 那一行的一个段序列数字），
断言 `green/` 判绿、`red/` 判红；改坏之前先确认那个数字串在整份文件里只出现一次，
免得改错了别的地方也跟着变。
"""
import argparse
import dataclasses
import pathlib
import re
import shutil
import sys
import tempfile

LAYOUT_RELATIVE_PATH = pathlib.Path('.claude/kb/first-txn-layout.md')
REPLAY_SCRIPT_RELATIVE_PATH = pathlib.Path('research/scripts/replay.sh')
RESULTS_DIRECTORY_RELATIVE_PATH = pathlib.Path('research/results')

SECTION_EIGHT_HEADING_PATTERN = re.compile(r'^## 八、.*$', re.MULTILINE)
NEXT_HEADING_PATTERN = re.compile(r'^## ', re.MULTILINE)
DECLARED_PRODUCT_PATHS_PATTERN = re.compile(r'`path=([^`]+)`')
MERGED_STREAM_NOTE_PATTERN = re.compile(r'整条流\s*`([0-9]+(?:\+[0-9]+)+)`\s*、\s*(\d+)\s*个状态')
ROW_PRODUCT_PATH_PATTERN = re.compile(r'path=([A-Za-z0-9_]+)')
SEGMENT_SEQUENCE_PATTERN = re.compile(r'`([0-9]+(?:\+[0-9]+)+)`')
STEP_KINDS_PATTERN = re.compile(r'种类\s*`(\[[^`]+\])`')
UNESCAPED_PIPE_PATTERN = re.compile(r'(?<!\\)\|')
OPERATION_COUNT_PATTERN = re.compile(r'(\d+)\s*次操作')
CRASH_STATE_COUNT_PATTERN = re.compile(r'(\d+)\s*个(?:崩溃)?状态')
E142_REPLAY_ROW_PATTERN = re.compile(r'^E142\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)$', re.MULTILINE)
PRODUCT_SEGMENT_LINE_PREFIX = 'E7RESULT name=segments '
PRODUCT_SEGMENT_LINE_REQUIRED_KEYS = ('path', 'operations', 'segments', 'closed_form')


class CheckError(Exception):
    """结构性问题：文件找不到、格式认不出。跟「数字对不上」是两类失败，分开报。"""


@dataclasses.dataclass(frozen=True)
class RegistryEntry:
    row_label: str
    origin_description: str  # '表格行' 或 '整条流提示'——只用来在成功摘要里分开计数，不参与比对
    is_anticipated: bool
    product_path_name: str | None
    segment_sequence_text: str | None
    operation_count: int | None
    crash_state_count: int | None
    step_kinds_text: str | None  # 「种类 `[...]|[...]`」那一串：每段的步骤种类多重集，与产物 kinds= 字段逐字比


def default_repository_root():
    """脚本自己住在 <仓库根>/research/scripts/ 下，向上两级就是仓库根。"""
    return pathlib.Path(__file__).resolve().parents[2]


def locate_e142_product_path(repository_root):
    """从 replay.sh 的复跑登记表里读 E142 那一行，取它的入库产物文件名。

    不写死产物文件名：产物重跑之后 replay.sh 那一行会换成新文件名，这里跟着换。
    """
    replay_script_path = repository_root / REPLAY_SCRIPT_RELATIVE_PATH
    if not replay_script_path.is_file():
        raise CheckError(f'找不到 {replay_script_path}，E142 产物的路径定不下来（本该从这份表里的那一行读）')
    replay_script_text = replay_script_path.read_text(encoding='utf-8')
    match = E142_REPLAY_ROW_PATTERN.search(replay_script_text)
    if match is None:
        raise CheckError(f'{replay_script_path} 里找不到「E142|...」那一行——它的复跑登记表格式变了？')
    product_file_name = match.group(3)
    if not product_file_name:
        raise CheckError(f'{replay_script_path} 里 E142 那一行的入库产物列是空的')
    return repository_root / RESULTS_DIRECTORY_RELATIVE_PATH / product_file_name


def extract_section_eight(layout_text):
    """截出「## 八、……」到下一个「## 」标题之间的正文（含表格与表下面的提示段落）。"""
    heading_match = SECTION_EIGHT_HEADING_PATTERN.search(layout_text)
    if heading_match is None:
        raise CheckError('first-txn-layout.md 里找不到「## 八、」这个标题，登记表挪地方了？')
    section_start = heading_match.end()
    remainder = layout_text[section_start:]
    next_heading_match = NEXT_HEADING_PATTERN.search(remainder)
    section_end = section_start + next_heading_match.start() if next_heading_match else len(layout_text)
    return layout_text[section_start:section_end]


def parse_registry_table_rows(section_text):
    """取出第一段连续的 `|...|` 行：第一行是表头、第二行是分隔行，其余是数据行。"""
    table_lines = []
    started = False
    for line in section_text.split('\n'):
        stripped = line.strip()
        if stripped.startswith('|') and stripped.endswith('|'):
            table_lines.append(stripped)
            started = True
        elif started:
            break
    if len(table_lines) < 3:
        raise CheckError('八节里没找到登记表（至少要有表头行、分隔行、一条数据行）')
    separator_line = table_lines[1]
    if not re.fullmatch(r'\|[\s:|-]+\|', separator_line):
        raise CheckError(f'登记表表头下面那一行看起来不是分隔行：{separator_line!r}')
    data_rows = []
    for data_line in table_lines[2:]:
        # 单元格里的字面竖线按 markdown 写成 `\|`（种类那一串里段与段之间就是它），切格时只认没转义的竖线，切完再还原。
        raw_cells = UNESCAPED_PIPE_PATTERN.split(data_line.strip('|'))
        cells = [cell.strip().replace('\\|', '|') for cell in raw_cells]
        data_rows.append(cells)
    return data_rows


def row_to_entry(row_cells):
    """把表格一行的四个单元格，翻成一条可比对的登记项。

    path 从「出处」栏（第三列）里的 `path=...` 原样解析，不是另写一份映射表；
    段序列数字串、操作次数、崩溃状态数都从「段序列」栏（第二列）里的具体写法抠出来，
    表里没写的字段留 None——「表里没写」与「写了但对不上」是两回事，不能混。
    """
    if len(row_cells) < 4:
        raise CheckError(f'登记表这一行列数不对（要 4 列）：{row_cells!r}')
    row_label, segment_cell, source_cell = row_cells[0], row_cells[1], row_cells[2]

    is_anticipated = '预想' in segment_cell

    path_match = ROW_PRODUCT_PATH_PATTERN.search(source_cell)
    product_path_name = path_match.group(1) if path_match else None

    segment_sequence_matches = SEGMENT_SEQUENCE_PATTERN.findall(segment_cell)
    segment_sequence_text = segment_sequence_matches[-1] if segment_sequence_matches else None

    operation_count_match = OPERATION_COUNT_PATTERN.search(segment_cell)
    operation_count = int(operation_count_match.group(1)) if operation_count_match else None

    crash_state_count_match = CRASH_STATE_COUNT_PATTERN.search(segment_cell)
    crash_state_count = int(crash_state_count_match.group(1)) if crash_state_count_match else None

    step_kinds_match = STEP_KINDS_PATTERN.search(segment_cell)
    step_kinds_text = step_kinds_match.group(1) if step_kinds_match else None

    return RegistryEntry(
        row_label=row_label,
        origin_description='表格行',
        is_anticipated=is_anticipated,
        product_path_name=product_path_name,
        segment_sequence_text=segment_sequence_text,
        operation_count=operation_count,
        crash_state_count=crash_state_count,
        step_kinds_text=step_kinds_text,
    )


def parse_declared_product_paths(section_text):
    """读开头那句「四行（`path=mkfs / warm_up / transaction / post_mkfs_stream`）」声明的路径集合。

    只认反引号里**紧跟 `path=`** 的那一段：表格「出处」栏里的反引号内容是
    `name=segments path=mkfs`，以 `name=` 开头，不会被这条正则误认成这句声明。
    """
    match = DECLARED_PRODUCT_PATHS_PATTERN.search(section_text)
    if match is None:
        raise CheckError('八节里找不到「`path=mkfs / warm_up / …`」这句声明，产物路径集合定不下来')
    return [path.strip() for path in match.group(1).split('/')]


def build_merged_stream_entry(section_text, table_entries):
    """表下面那句 ⚠️ 提示，是第四条产物路径（合并整条录制流）唯一的登记位。

    它没有自己的「出处」栏可以直接解析 path，所以用排除法：开头声明的四个路径，
    减去前面表格行已经占用的那几个，应该恰好剩一个——剩不下恰好一个就是结构性错误，
    不猜、不悄悄按字面写死 `post_mkfs_stream`。
    """
    declared_paths = parse_declared_product_paths(section_text)
    merged_note_match = MERGED_STREAM_NOTE_PATTERN.search(section_text)
    if merged_note_match is None:
        raise CheckError(
            '八节里找不到「整条流 `...`、N 个状态」这句提示——它是第四条产物路径（合并整条录制流）唯一的登记位'
        )
    merged_segment_sequence_text = merged_note_match.group(1)
    merged_crash_state_count = int(merged_note_match.group(2))
    # 「种类 `[...]`」紧跟在那句提示的状态数后面（同一个括号里），只在提示之后、下一个句号之前找。
    note_tail = section_text[merged_note_match.end():]
    note_tail = note_tail.split('）', 1)[0]
    merged_step_kinds_match = STEP_KINDS_PATTERN.search(note_tail)
    merged_step_kinds_text = merged_step_kinds_match.group(1) if merged_step_kinds_match else None

    claimed_paths = {entry.product_path_name for entry in table_entries if entry.product_path_name is not None}
    remaining_paths = [path for path in declared_paths if path not in claimed_paths]
    if len(remaining_paths) != 1:
        raise CheckError(
            '开头「四行（`path=...`）」声明的路径集合是 ' + '、'.join(declared_paths) +
            '，表格行已经占用 ' + '、'.join(sorted(claimed_paths)) +
            f'，排除之后应该剩一条给「整条流」那句提示，实际剩 {remaining_paths} 条——'
            '八节的写法变了，这段解析要跟着改，不能再按排除法自动认领'
        )
    merged_stream_path_name = remaining_paths[0]

    return RegistryEntry(
        row_label='⚠️ 整条流（发布与下一次发布之间没有屏障那句提示）',
        origin_description='整条流提示',
        is_anticipated=False,
        product_path_name=merged_stream_path_name,
        segment_sequence_text=merged_segment_sequence_text,
        operation_count=None,
        crash_state_count=merged_crash_state_count,
        step_kinds_text=merged_step_kinds_text,
    )


def build_registry_entries(section_text):
    """表格里的数据行，加上表下面那句 ⚠️ 提示，合成完整的登记项清单。"""
    table_entries = [row_to_entry(row_cells) for row_cells in parse_registry_table_rows(section_text)]
    merged_stream_entry = build_merged_stream_entry(section_text, table_entries)
    return table_entries + [merged_stream_entry]


def parse_product_segment_lines(product_text):
    """产物里 `E7RESULT name=segments path=... operations=... segments=... closed_form=...` 那几行。

    按空格切成 `key=value` 词元，认必要的四个字段，外加可选的 `kinds=[...]`
    （第五次跑起每段的步骤种类多重集，D17 已定项 2 的「每段步骤种类集合」）；
    其余字段原样忽略，产物这一行以后再加字段也不用改这个脚本。
    """
    product_segment_lines = {}
    for line in product_text.split('\n'):
        if not line.startswith(PRODUCT_SEGMENT_LINE_PREFIX):
            continue
        fields = {}
        for token in line[len(PRODUCT_SEGMENT_LINE_PREFIX):].split(' '):
            if '=' not in token:
                continue
            key, _, value = token.partition('=')
            fields[key] = value
        missing_keys = [key for key in PRODUCT_SEGMENT_LINE_REQUIRED_KEYS if key not in fields]
        if missing_keys:
            raise CheckError(f'产物这一行缺 {"、".join(missing_keys)} 字段，认不出来：{line!r}')
        try:
            operation_count = int(fields['operations'])
            closed_form_crash_state_count = int(fields['closed_form'])
        except ValueError as error:
            raise CheckError(f'产物这一行的 operations 或 closed_form 不是整数：{line!r}') from error
        product_segment_lines[fields['path']] = {
            'operation_count': operation_count,
            'segment_sequence_text': fields['segments'],
            'closed_form_crash_state_count': closed_form_crash_state_count,
            'step_kinds_text': fields.get('kinds'),
        }
    if not product_segment_lines:
        raise CheckError('产物里一行 `E7RESULT name=segments ...` 都没读到，产物的格式是不是变了')
    return product_segment_lines


def mutate_one_segment_number(layout_text):
    """--selftest 用：把八节表格第一条可比对的行，段序列的最后一项加 1。

    先确认原样的反引号串（比如 `` `4+1+1+1+2` ``）在整份文件里只出现一次，
    免得替换的时候顺手改坏了别的地方——那样测出来的「判红」证明的是别的东西，不是这条检查本身。
    """
    section_text = extract_section_eight(layout_text)
    for row_cells in parse_registry_table_rows(section_text):
        entry = row_to_entry(row_cells)
        if entry.is_anticipated or entry.segment_sequence_text is None:
            continue
        original_segment_sequence_text = entry.segment_sequence_text
        original_span = f'`{original_segment_sequence_text}`'
        occurrence_count = layout_text.count(original_span)
        if occurrence_count != 1:
            raise CheckError(
                f'「{entry.row_label}」这一行的段序列 {original_span} 在整份文件里出现了 '
                f'{occurrence_count} 次，不是 1 次——自证选的这一行定位不到唯一位置，换一行，或者查一下文件是不是重复了'
            )
        terms = original_segment_sequence_text.split('+')
        terms[-1] = str(int(terms[-1]) + 1)
        mutated_segment_sequence_text = '+'.join(terms)
        mutated_layout_text = layout_text.replace(original_span, f'`{mutated_segment_sequence_text}`', 1)
        return mutated_layout_text, entry.row_label, original_segment_sequence_text, mutated_segment_sequence_text
    raise CheckError('八节的登记表里找不到一条「不是预想、且带段序列数字串」的行，自证没法造出一个可判红的改动')


def mutate_one_step_kind(layout_text):
    """--selftest 用：把八节表格第一条带「种类」的行，第一段里第一个种类的重数改掉（×4 → ×5，没有重数就加 ×2）。

    段序列一个字不动，所以这一刀只有「种类」那一半的比对能看见。
    """
    section_text = extract_section_eight(layout_text)
    for row_cells in parse_registry_table_rows(section_text):
        entry = row_to_entry(row_cells)
        if entry.is_anticipated or entry.step_kinds_text is None:
            continue
        # 表格单元格里的竖线在文件里写成 `\\|`，定位原文要按转义后的样子找。
        original_span = '`' + entry.step_kinds_text.replace('|', '\\|') + '`'
        if layout_text.count(original_span) != 1:
            raise CheckError(f'「{entry.row_label}」这一行的种类串在整份文件里不是恰好出现 1 次，自证定位不到唯一位置')
        first_segment_end = entry.step_kinds_text.index(']')
        first_segment = entry.step_kinds_text[:first_segment_end]
        first_kind = first_segment[1:].split(',')[0]
        if '×' in first_kind:
            kind_name, _, multiplicity = first_kind.partition('×')
            mutated_kind = f'{kind_name}×{int(multiplicity) + 1}'
        else:
            mutated_kind = f'{first_kind}×2'
        mutated_kinds_text = entry.step_kinds_text.replace(first_kind, mutated_kind, 1)
        mutated_span = '`' + mutated_kinds_text.replace('|', '\\|') + '`'
        mutated_layout_text = layout_text.replace(original_span, mutated_span, 1)
        return mutated_layout_text, entry.row_label, first_kind, mutated_kind
    raise CheckError('八节的登记表里找不到一条带「种类 `[...]`」的行，自证没法造出一个只改种类的改动')


def perform_check(repository_root):
    """跑一遍完整比对，返回 (退出码, 要打印的每一行)。"""
    try:
        layout_path = repository_root / LAYOUT_RELATIVE_PATH
        if not layout_path.is_file():
            return 2, [f'  ✗ 找不到 {layout_path}']
        layout_text = layout_path.read_text(encoding='utf-8')

        product_path = locate_e142_product_path(repository_root)
        if not product_path.is_file():
            return 2, [f'  ✗ replay.sh 指到了 {product_path}，但这份产物不在']
        product_text = product_path.read_text(encoding='utf-8')

        section_text = extract_section_eight(layout_text)
        entries = build_registry_entries(section_text)
        product_segment_lines = parse_product_segment_lines(product_text)
    except CheckError as error:
        return 2, [f'  ✗ {error}']

    mismatches = []
    compared_table_row_count = 0
    compared_note_count = 0
    skipped_count = 0
    for entry in entries:
        if entry.is_anticipated:
            skipped_count += 1
            continue
        if entry.product_path_name is None:
            mismatches.append(f'{entry.row_label}：这一行不是预想，「出处」栏里却没有 `path=...` 引用，登记表这一行本身要修')
            continue
        if entry.segment_sequence_text is None:
            mismatches.append(f'{entry.row_label}：这一行不是预想，却没有可比对的段序列数字串')
            continue
        product_entry = product_segment_lines.get(entry.product_path_name)
        if product_entry is None:
            mismatches.append(
                f'{entry.row_label}：登记表引用 path={entry.product_path_name}，'
                f'产物 {product_path.name} 里没有这个 path 的 name=segments 行'
            )
            continue
        if entry.segment_sequence_text != product_entry['segment_sequence_text']:
            mismatches.append(
                f'{entry.row_label}（path={entry.product_path_name}）：kb 写的段序列是 '
                f'`{entry.segment_sequence_text}`，产物里是 `{product_entry["segment_sequence_text"]}`'
            )
            continue
        if entry.operation_count is not None and entry.operation_count != product_entry['operation_count']:
            mismatches.append(
                f'{entry.row_label}（path={entry.product_path_name}）：kb 写的操作次数是 '
                f'{entry.operation_count}，产物里 operations 是 {product_entry["operation_count"]}'
            )
            continue
        if entry.crash_state_count is not None and entry.crash_state_count != product_entry['closed_form_crash_state_count']:
            mismatches.append(
                f'{entry.row_label}（path={entry.product_path_name}）：kb 写的崩溃状态数是 '
                f'{entry.crash_state_count}，产物里 closed_form 是 {product_entry["closed_form_crash_state_count"]}'
            )
            continue
        if entry.step_kinds_text is None:
            mismatches.append(
                f'{entry.row_label}（path={entry.product_path_name}）：这一行没有写「种类 `[...]`」——'
                '每段的步骤种类多重集是等价类判据的一半（D17 已定项 2），登记表要照产物 kinds= 字段抄上'
            )
            continue
        if product_entry['step_kinds_text'] is None:
            mismatches.append(
                f'{entry.row_label}（path={entry.product_path_name}）：kb 写了种类 `{entry.step_kinds_text}`，'
                '产物这一行却没有 kinds= 字段——产物是第五次跑之前的，要重跑 E142'
            )
            continue
        if entry.step_kinds_text != product_entry['step_kinds_text']:
            mismatches.append(
                f'{entry.row_label}（path={entry.product_path_name}）：kb 写的种类是 '
                f'`{entry.step_kinds_text}`，产物里 kinds 是 `{product_entry["step_kinds_text"]}`'
            )
            continue
        if entry.origin_description == '表格行':
            compared_table_row_count += 1
        else:
            compared_note_count += 1

    if mismatches:
        lines = [f'  ✗ 段序列登记表与 E142 产物的 name=segments 行对不上，共 {len(mismatches)} 处：']
        for mismatch in mismatches:
            lines.append(f'     {mismatch}')
        lines.append('     → 怎么办：表是抄错了，就照产物改 .claude/kb/first-txn-layout.md 那一行；')
        lines.append('       产物是过期了，就重跑 E142（第一个事务的干跑），更新 research/results/ 下的文件，')
        lines.append('       再让 research/scripts/replay.sh 里 E142 那一行的入库产物名字跟上。')
        return 1, lines

    compared_total_count = compared_table_row_count + compared_note_count
    lines = [
        f'  ✓ 比对了 {compared_total_count} 处登记（表格 {compared_table_row_count} 行 + '
        f'整条流提示 {compared_note_count} 处，段序列与每段步骤种类多重集都与 {product_path.name} 的 name=segments 行逐字一致），'
        f'跳过 {skipped_count} 条标预想的表格行'
    ]
    return 0, lines


def build_fixture_root(root_path, layout_text, replay_script_text, product_file_name, product_text):
    """在临时目录里按仓库的相对路径摆好三份文件，供 --selftest 各拿一份去跑真正的比对逻辑。"""
    layout_path = root_path / LAYOUT_RELATIVE_PATH
    replay_script_path = root_path / REPLAY_SCRIPT_RELATIVE_PATH
    product_path = root_path / RESULTS_DIRECTORY_RELATIVE_PATH / product_file_name
    for path in (layout_path, replay_script_path, product_path):
        path.parent.mkdir(parents=True, exist_ok=True)
    layout_path.write_text(layout_text, encoding='utf-8')
    replay_script_path.write_text(replay_script_text, encoding='utf-8')
    product_path.write_text(product_text, encoding='utf-8')
    return root_path


def run_self_test():
    real_repository_root = default_repository_root()
    try:
        layout_text = (real_repository_root / LAYOUT_RELATIVE_PATH).read_text(encoding='utf-8')
        replay_script_text = (real_repository_root / REPLAY_SCRIPT_RELATIVE_PATH).read_text(encoding='utf-8')
        real_product_path = locate_e142_product_path(real_repository_root)
        product_text = real_product_path.read_text(encoding='utf-8')
        (mutated_layout_text, mutated_row_label,
         original_segment_sequence_text, mutated_segment_sequence_text) = mutate_one_segment_number(layout_text)
        (kind_mutated_layout_text, kind_mutated_row_label,
         original_kind_text, mutated_kind_text) = mutate_one_step_kind(layout_text)
    except CheckError as error:
        print(f'  ✗ --selftest 备料就失败了：{error}')
        print('     → 怎么办：先跑一次不带 --selftest 的检查，确认真实仓库能正常读出登记表与产物。')
        return 1

    # 默认落 tempfile 的标准临时目录（尊重 TMPDIR 环境变量），不写死到某一次会话的 scratchpad 路径——
    # 这份脚本要在任何环境、任何一次门禁运行里都能跑，不能只认一个特定的临时目录。
    workspace_path = pathlib.Path(tempfile.mkdtemp(prefix='check-segment-registry-selftest-'))
    try:
        green_root = build_fixture_root(
            workspace_path / 'green', layout_text, replay_script_text, real_product_path.name, product_text,
        )
        red_root = build_fixture_root(
            workspace_path / 'red', mutated_layout_text, replay_script_text, real_product_path.name, product_text,
        )
        kind_red_root = build_fixture_root(
            workspace_path / 'kind-red', kind_mutated_layout_text, replay_script_text, real_product_path.name, product_text,
        )

        green_exit_code, green_lines = perform_check(green_root)
        red_exit_code, red_lines = perform_check(red_root)
        kind_red_exit_code, _kind_red_lines = perform_check(kind_red_root)
    finally:
        shutil.rmtree(workspace_path, ignore_errors=True)

    problems = []
    if green_exit_code != 0:
        problems.append('未改动的拷贝本该判绿，实际退出码 ' + str(green_exit_code) + '：\n       ' + '\n       '.join(green_lines))
    if red_exit_code == 0:
        problems.append(
            f'把「{mutated_row_label}」那一行的段序列从 `{original_segment_sequence_text}` 改成 '
            f'`{mutated_segment_sequence_text}` 之后，检查仍然判绿——没有判别力'
        )
    if kind_red_exit_code == 0:
        problems.append(
            f'把「{kind_mutated_row_label}」那一行第一段的种类 `{original_kind_text}` 改成 '
            f'`{mutated_kind_text}`（段序列不动）之后，检查仍然判绿——种类那一半没有判别力'
        )

    if problems:
        print('  ✗ --selftest 没通过：')
        for problem in problems:
            print(f'     {problem}')
        print('     → 怎么办：看清楚上面改坏的是哪一行、期望判红的地方是不是真的判了红，')
        print('       再回头检查 perform_check 里对应字段的比较逻辑。')
        return 1

    print(
        '  ✓ --selftest 通过：未改动的拷贝判绿（退出码 0）；'
        f'把「{mutated_row_label}」的段序列改成 `{mutated_segment_sequence_text}` 之后判红（退出码 {red_exit_code}）；'
        f'只把「{kind_mutated_row_label}」第一段的种类 `{original_kind_text}` 改成 `{mutated_kind_text}` 也判红（退出码 {kind_red_exit_code}）'
    )
    return 0


def main():
    parser = argparse.ArgumentParser(
        description='比对 first-txn-layout.md 「八、根槽写路径的段序列登记表」与 E142 产物的 name=segments 行'
    )
    parser.add_argument(
        '--root', type=pathlib.Path, default=None,
        help='仓库根目录；默认按这个脚本自己的位置推算（research/scripts/ 的上两级）',
    )
    parser.add_argument(
        '--selftest', action='store_true',
        help='自证：改坏拷贝里的一个段序列数字，确认检查会转红；未改动的拷贝确认判绿。忽略 --root。',
    )
    arguments = parser.parse_args()

    if arguments.selftest:
        sys.exit(run_self_test())

    repository_root = arguments.root.resolve() if arguments.root else default_repository_root()
    exit_code, message_lines = perform_check(repository_root)
    for line in message_lines:
        print(line)
    sys.exit(exit_code)


if __name__ == '__main__':
    main()
