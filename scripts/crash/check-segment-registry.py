#!/usr/bin/env python3
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: none 只读仓里的文本（与它点名的会话记录），除了 python3 之外没有环境要求
"""比对「八、根槽写路径的段序列登记表」与钉住活代码的用例，逐字核对；钉不到用例的行才与 E142 产物的 `name=segments` 行比。

用法：
    check-segment-registry.py [--root 仓库根目录]
    check-segment-registry.py [--root 仓库根目录] --print-mkfs-segments-line
    check-segment-registry.py [--root 仓库根目录] --print-segments-lines
    check-segment-registry.py --selftest

退出码：0 全部相符；1 有对不上的；2 结构认不出、文件不在；77 没有对不上的，但有几处登记只能与 E142 产物比而产物不在（本次未跑，不记通过）。

`--print-mkfs-segments-line`：不比对，只把八节登记表「mkfs 种根」那一行登记的段序列、次操作、崩溃状态数与种类串
拼成虚机二进制打的那种行（`E7RESULT name=segments path=mkfs operations=… segments=… closed_form=… kinds=…`，
字段次序同 crates 里真设备二进制的输出）打到 stdout，一行；只读布局文件，不读 replay.sh 与产物。
门禁 checker-tier-qemu-device-streams 拿它与虚机输出的 `path=mkfs` 那一行逐字比（那一行的权威是钉住活代码的用例，
不是 E142 的产物）。那一行找不到、不止一行、四样缺一样，退 2。
`--print-segments-lines`：同样不比对，把八节开头「`path=…`」声明的每一条路径（表格里出处写 `path=<名>` 的行与整条流那句提示）
各拼成一行同样形态的 `E7RESULT name=segments path=<名> …`，按声明的次序打到 stdout；只读布局文件。
门禁 checker-tier-qemu-device-streams 拿它与虚机输出里 `path=mkfs` 之外那几行逐字比。声明的路径没有登记行、一条路径登记了两处、四样缺一样，退 2。

为什么要有它：`.claude/kb/checks-owed.md` C316（提交步骤的登记位有四处且互不相同） 收窄之后
剩两半，第①半是「kb 登记表八与产物 `name=segments` 行之间没有逐字比对（表是人抄的）」——
`.claude/kb/layout/01-first-txn.md` 「八、根槽写路径的段序列登记表」里每一行的段序列数字串
（`4+1+1+1+2` 这种）是人抄的，抄错一位、或者活代码改了之后表没跟着改，都没有任何东西会报警。
这个脚本把「抄的对不对」变成一条会红的检查。比的对象是钉在活代码上的用例，不是留存产物：
research/ 整个可删（.claude/rules/directory-layout.md「research/：可以整个删掉的研究目录」），产物不能当这一格的依据。

比对哪几行、跟谁比：
- 出处栏指到一条钉住活代码的用例（反引号里一个 `<文件>.rs` 加一个测试函数名，mkfs 种根那一行就是）：
  与那条用例里写死的串比（compare_against_pinning_test：段序列串、种类串都要在用例里逐字出现）。
- 出处栏写 `产物 `name=segments path=<名>``，以及表下面那句 ⚠️ 整条流提示（它占声明里剩下的那一条 path）：
  <名> 在 PATH_PINNING_TESTS 里登记了钉住它的 crates 用例的，与那条用例比（compare_against_path_pinning_test）：
  用例函数体里要有一条 `(<次操作>, "<段序列>"[.to_string()], <崩溃状态数>)` 的断言（kb 没写次操作或状态数的那一样不比），
  种类串要逐字出现。今天四条路径（instance_acquisition、warm_up、transaction、post_mkfs_stream）都登记在那里，
  钉它们的是 `crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 的 `recorded_paths_match_the_registered_segment_sequences`，
  它由 `cargo test` 钉在活代码上——串对不上活代码，那条用例先红。
  <名> 没登记钉住它的用例的，才与 E142 产物比：产物路径从 `scripts/experiment-apparatus/replay.sh` 复跑登记表里 `E142|...` 那一行的
  「入库产物」那一列取；那份产物不在，这几行记「本次未跑」（退 77，不判红）。
- 「实例切换 / 管理员回退」「抬 F 的空发布」两行标了 **预想**（没写成字节）的跳过，计入跳过数；标「装置钉住」的表格行只计数，
  覆盖写固定脚本流那句另与钉它的用例核数组、写数与闭式（check_device_pinned_notes）。
- 表下面那句 ⚠️ 提示是第四条产物路径（`post_mkfs_stream`）唯一的登记位——它不在表格里。这个脚本按八节开头
  那句「产物的 `name=segments` 四行（`path=…`）」声明的路径，减去表格行已经占用的，剩下那一个按排除法归给这句提示；
  减出来的路径集合大小若不是恰好一个，直接判成结构性错误（八节的写法变了，这段解析要跟着改），不猜、不悄悄跳过。

自证会红：`--selftest` 的输入是当前这份 layout/01-first-txn.md、它点名的 crates 用例文件（活代码，不是生成物）
与脚本旁边的样本 `scripts/crash/check-segment-registry-product-sample.txt`（行形照 E142 第二十五次跑主产物第 35–41、46、60 行，
name=segments 五行的数换成现行钉值，出处写在样本文件头；只用来测产物行的解析）；不读 replay.sh 指的产物，也不读 research/ 下任何文件。要与产物比的那一支，产物行由自证照登记行现造。
各格（项名是 run_self_test() 里 problems 那一条的开头）：
  未改动的拷贝（不摆产物）判绿；mkfs 那一行段序列改一个数、种类改一个重数、覆盖写固定脚本流的段序列改一个数，各判红；
  覆盖写固定脚本流的登记句改成认不出的形状（「N 次写」写成「N 笔写」，一句都认不出），判红；
  钉它的用例文件拷贝里状态数只剩带下划线的 Rust 写法（例 `1_678_377_095`，不带下划线的写法一处不留），判绿；
  八节开头「`path=…`」声明里多写一条 `selftest_unclaimed_path`，退 2 并点出实际剩几条；
  第一条出处写 path= 的表格行段序列改一个数、次操作改一个数，与钉它的 crates 用例比，各判红；
  同一行里同一串出现两次（段序列栏一处、出处栏一处）：自证造的小布局表上只换段序列栏那一处、出处栏那一处不动（真仓备料的几处也照段序列栏换）；
  把那条路径从 PATH_PINNING_TESTS 里拿掉（只能与产物比）：产物不在退 77 并点名那一行，摆上照登记现造的产物判绿，产物里段序列改一个数判红；
  样本的产物行解析出五条路径、都带种类；`--print-mkfs-segments-line` 那一支三份（照登记打一行、段序列跟着变、改名与抄成两行退 2）；
  `--print-segments-lines` 打出的行数等于声明的路径数、按产物行解析回来与登记相符，第一条路径的段序列改一个数打出的行跟着变。

弄坏开关（只给 --selftest 证红用）。逐项写开关名、弄坏哪一处、打开后自证红的那几格（「」里是失败行的开头：自证逐条以 ✗ 起头列出判错的格，末尾一句「✗ --selftest 没通过：N 项」是汇总）：
  CHECK_SEGMENT_REGISTRY_BREAK=accept-extra-path  build_merged_stream_entry() 把「排除之后恰好剩一条」当成「至少剩一条」判（`!= 1` 当 `< 1`）
      多声明一条 path（「✗ 八节开头「`path=…`」声明里多写了一条 `selftest_unclaimed_path`」）
  CHECK_SEGMENT_REGISTRY_BREAK=path-rows-to-product  perform_check() 不认 PATH_PINNING_TESTS，出处写 path= 的行一律与 E142 产物比
      不摆 replay.sh 与产物的未改动拷贝定不下产物路径、退 2（「✗ 未改动的拷贝（不摆产物）本该判绿」与它转述的「✗ 找不到 …/scripts/experiment-apparatus/replay.sh，E142 产物的路径定不下来」）；
      与钉它的用例比的两格同样退 2（「✗ 把「…」（path=instance_acquisition）的段序列…本该退 1」、「✗ 把「…」的「6 次操作」改成「7 次操作」之后…本该退 1」）；
      覆盖写固定脚本流那两格的拷贝同样不摆产物、退 2（「✗ 覆盖写固定脚本流的登记句改成认不出的形状…本该退 1」、「✗ 钉它的用例文件拷贝里状态数只剩带下划线的写法…本该判绿」）；
      摆上产物那一格比不上别的路径（「✗ 只能与产物比、摆上照登记现造的产物之后本该判绿」）
  CHECK_SEGMENT_REGISTRY_BREAK=product-required  要与产物比而产物不在时退回改之前的判法：退 2 记红，不报本次未跑
      （「✗ 只能与产物比、产物不在的那一行本该退 77（本次未跑）」）
  CHECK_SEGMENT_REGISTRY_BREAK=unrecognized-note-accepted  check_device_pinned_notes() 一句覆盖写固定脚本流的登记都认不出时照改之前的判法当成没有要核的（不记对不上）
      （「✗ 覆盖写固定脚本流的登记句改成认不出的形状…本该退 1」）
  CHECK_SEGMENT_REGISTRY_BREAK=underscore-kept  check_device_pinned_notes() 比状态数之前照改之前不去掉数字间的下划线
      （「✗ 钉它的用例文件拷贝里状态数只剩带下划线的写法…本该判绿」）
  CHECK_SEGMENT_REGISTRY_BREAK=segment-cell-unscoped  「同一行两处同串」那一格里 mutate_one_segment_number() 照改之前在整行里换（同一行里那一串出现两次就备料失败）
      （「✗ 同一行里同一串出现两次（段序列栏一处、出处栏一处）时自证本该只换段序列栏那一处」）
"""
import argparse
import dataclasses
import os
import pathlib
import re
import shutil
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

LAYOUT_RELATIVE_PATH = pathlib.Path('.claude/kb/layout/01-first-txn.md')
REPLAY_SCRIPT_RELATIVE_PATH = pathlib.Path('scripts/experiment-apparatus/replay.sh')
RESULTS_DIRECTORY_RELATIVE_PATH = pathlib.Path('research/results')
# 自证测产物行解析用的样本（E142 第二十五次跑主产物 e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out 第 35–41、46、60 行原样摘抄），跟着脚本放
PRODUCT_SAMPLE_PATH = pathlib.Path(__file__).resolve().parent / 'check-segment-registry-product-sample.txt'
PRODUCT_SAMPLE_PATHS = ('mkfs', 'instance_acquisition', 'warm_up', 'transaction', 'post_mkfs_stream')

# 出处写 `path=<名>` 的登记行与整条流提示按名字找钉住它的 crates 用例：(<用例文件>, <测试函数名>)。
# 这条用例把 mkfs 之后每一条录制路径的 (次操作, 段序列, 闭式) 与种类串钉成绝对值，由 cargo test 钉在活代码上。
# 新加一条 path 的登记行，先在 crates 里有钉住它的用例、登记进这里；没登记的那条才退回与 E142 产物比（产物不在报本次未跑）。
NEW_POOL_FILE_CREATION_PINNING_TEST = ('crates/singlefs-harness/tests/new_pool_file_creation_publish.rs',
                                       'recorded_paths_match_the_registered_segment_sequences')
PATH_PINNING_TESTS = {
    'instance_acquisition': NEW_POOL_FILE_CREATION_PINNING_TEST,
    'warm_up': NEW_POOL_FILE_CREATION_PINNING_TEST,
    'transaction': NEW_POOL_FILE_CREATION_PINNING_TEST,
    'post_mkfs_stream': NEW_POOL_FILE_CREATION_PINNING_TEST,
}

SECTION_EIGHT_HEADING_PATTERN = re.compile(r'^## 八、.*$', re.MULTILINE)
NEXT_HEADING_PATTERN = re.compile(r'^## ', re.MULTILINE)
DECLARED_PRODUCT_PATHS_PATTERN = re.compile(r'`path=([^`]+)`')
MERGED_STREAM_NOTE_PATTERN = re.compile(r'整条流\s*`([0-9]+(?:\+[0-9]+)+)`\s*、\s*(\d+)\s*个状态')
ROW_PRODUCT_PATH_PATTERN = re.compile(r'path=([A-Za-z0-9_]+)')
SEGMENT_SEQUENCE_PATTERN = re.compile(r'`([0-9]+(?:\+[0-9]+)*)`')  # 一条路径可以只有一段（取号那一行是 `2`），所以 + 与 * 之间取 *
STEP_KINDS_PATTERN = re.compile(r'种类\s*`(\[[^`]+\])`')
UNESCAPED_PIPE_PATTERN = re.compile(r'(?<!\\)\|')
OPERATION_COUNT_PATTERN = re.compile(r'(\d+)\s*次操作')
CRASH_STATE_COUNT_PATTERN = re.compile(r'(\d+)\s*个(?:崩溃)?状态')
E142_REPLAY_ROW_PATTERN = re.compile(r'^E142\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)$', re.MULTILINE)
DEVICE_PINNED_NOTE_PATTERN = re.compile(
    r'覆盖写固定脚本流的段序列\s*`([0-9]+(?:\+[0-9]+)+)`\s*、\s*(\d+)\s*次写\s*、\s*(\d+)\s*个状态[^\n]*?装置钉住[^\n]*?`(crates/[^`]+\.rs)`'
)  # 里程碑「覆盖写、释放、回退与复用」的固定脚本没有干跑产物，整条流由 harness 用例里钉的数组守着：数字串、写数、状态数、钉它的用例文件
# 用例函数体在下一个顶格的条目处结束（rustfmt 之后测试文件里的条目都顶格写）
NEXT_TOP_LEVEL_ITEM_PATTERN = re.compile(r'^(?:#\[|#!\[|fn |pub |mod |const |static |struct |enum |impl |use |type |trait |macro_rules!)', re.MULTILINE)
PRODUCT_SEGMENT_LINE_PREFIX = 'E7RESULT name=segments '
PRODUCT_SEGMENT_LINE_REQUIRED_KEYS = ('path', 'operations', 'segments', 'closed_form')
MKFS_ROW_LABEL = 'mkfs 种根'  # 八节登记表第一列的原文；--print-mkfs-segments-line 按它认那一行
MKFS_PRODUCT_PATH_NAME = 'mkfs'  # 虚机二进制与 E142 产物里 mkfs 那一行的 path= 值
EXTRA_DECLARED_PATH_NAME = 'selftest_unclaimed_path'  # --selftest 往八节开头的 path= 声明里多写的那一条（没有表格行占它）
SELFTEST_DUPLICATE_ROW_LABEL = '自证造的一行（同一串写两处）'  # --selftest「同一行两处同串」那一格的小布局表里那一行的第一列
SELFTEST_PRODUCT_FILE_NAME = 'selftest-e142.out'  # --selftest 要与产物比的那一支：临时仓里 replay.sh 的 E142 行指的产物名


def broken(switch):
    """弄坏开关（文件头的开关表）：只给 --selftest 证红用，平时不设。"""
    return os.environ.get('CHECK_SEGMENT_REGISTRY_BREAK') == switch


class CheckError(Exception):
    """结构性问题：文件找不到、格式认不出。跟「数字对不上」是两类失败，分开报。"""


@dataclasses.dataclass(frozen=True)
class RegistryEntry:
    row_label: str
    origin_description: str  # '表格行' 或 '整条流提示'——只用来在成功摘要里分开计数，不参与比对
    is_anticipated: bool
    is_device_pinned: bool  # 段序列标「装置钉住」：从覆盖写固定脚本流推得、不与用例或产物逐行比，只计数
    product_path_name: str | None
    segment_sequence_text: str | None
    operation_count: int | None
    crash_state_count: int | None
    step_kinds_text: str | None  # 「种类 `[...]|[...]`」那一串：每段的步骤种类多重集，与用例或产物 kinds= 字段逐字比
    source_cell: str = ''  # 「出处」栏原文：出处指到一条钉住活代码的用例时按它比（C485）


def default_repository_root():
    """脚本自己住在 <仓库根>/scripts/ 下，向上两级就是仓库根。"""
    return pathlib.Path(__file__).resolve().parents[2]


def locate_e142_product_path(repository_root):
    """从 replay.sh 的复跑登记表里读 E142 那一行，取它的入库产物文件名。只在有登记行要与产物比时才调。

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
        raise CheckError('layout/01-first-txn.md 里找不到「## 八、」这个标题，登记表挪地方了？')
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
    is_device_pinned = '装置钉住' in segment_cell

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
        is_device_pinned=is_device_pinned,
        product_path_name=product_path_name,
        source_cell=source_cell,
        segment_sequence_text=segment_sequence_text,
        operation_count=operation_count,
        crash_state_count=crash_state_count,
        step_kinds_text=step_kinds_text,
    )


def parse_declared_product_paths(section_text):
    """读开头那句「四行（`path=instance_acquisition / warm_up / transaction / post_mkfs_stream`）」声明的路径集合。

    只认反引号里**紧跟 `path=`** 的那一段：表格「出处」栏里的反引号内容是
    `name=segments path=warm_up`，以 `name=` 开头，不会被这条正则误认成这句声明。
    """
    match = DECLARED_PRODUCT_PATHS_PATTERN.search(section_text)
    if match is None:
        raise CheckError('八节里找不到「`path=instance_acquisition / warm_up / …`」这句声明，路径集合定不下来')
    return [path.strip() for path in match.group(1).split('/')]


def build_merged_stream_entry(section_text, table_entries):
    """表下面那句 ⚠️ 提示，是第四条路径（合并整条录制流）唯一的登记位。

    它没有自己的「出处」栏可以直接解析 path，所以用排除法：开头声明的路径，
    减去前面表格行已经占用的那几个，应该恰好剩一个——剩不下恰好一个就是结构性错误，
    不猜、不悄悄按字面写死 `post_mkfs_stream`。
    次操作与种类串紧跟在那句提示的状态数后面（同一个全角括号闭合之前），只在那一段里找。
    """
    declared_paths = parse_declared_product_paths(section_text)
    merged_note_match = MERGED_STREAM_NOTE_PATTERN.search(section_text)
    if merged_note_match is None:
        raise CheckError(
            '八节里找不到「整条流 `...`、N 个状态」这句提示——它是第四条路径（合并整条录制流）唯一的登记位'
        )
    merged_segment_sequence_text = merged_note_match.group(1)
    merged_crash_state_count = int(merged_note_match.group(2))
    note_tail = section_text[merged_note_match.end():]
    note_tail = note_tail.split('）', 1)[0]
    merged_step_kinds_match = STEP_KINDS_PATTERN.search(note_tail)
    merged_step_kinds_text = merged_step_kinds_match.group(1) if merged_step_kinds_match else None
    merged_operation_count_match = OPERATION_COUNT_PATTERN.search(note_tail)
    merged_operation_count = int(merged_operation_count_match.group(1)) if merged_operation_count_match else None

    claimed_paths = {entry.product_path_name for entry in table_entries if entry.product_path_name is not None}
    remaining_paths = [path for path in declared_paths if path not in claimed_paths]
    remaining_path_count_is_wrong = len(remaining_paths) < 1 if broken('accept-extra-path') else len(remaining_paths) != 1
    if remaining_path_count_is_wrong:
        raise CheckError(
            '开头「四行（`path=...`）」声明的路径集合是 ' + '、'.join(declared_paths) +
            '，表格行已经占用 ' + '、'.join(sorted(claimed_paths)) +
            f'，排除之后应该剩一条给「整条流」那句提示，实际剩 {len(remaining_paths)} 条（{"、".join(remaining_paths) or "无"}）——'
            '八节的写法变了，这段解析要跟着改，不能再按排除法自动认领'
        )
    merged_stream_path_name = remaining_paths[0]

    return RegistryEntry(
        row_label='⚠️ 整条流（发布与下一次发布之间没有屏障那句提示）',
        origin_description='整条流提示',
        is_anticipated=False,
        is_device_pinned=False,
        product_path_name=merged_stream_path_name,
        segment_sequence_text=merged_segment_sequence_text,
        operation_count=merged_operation_count,
        crash_state_count=merged_crash_state_count,
        step_kinds_text=merged_step_kinds_text,
    )


def build_registry_entries(section_text):
    """表格里的数据行，加上表下面那句 ⚠️ 提示，合成完整的登记项清单。"""
    table_entries = [row_to_entry(row_cells) for row_cells in parse_registry_table_rows(section_text)]
    merged_stream_entry = build_merged_stream_entry(section_text, table_entries)
    return table_entries + [merged_stream_entry]


def segments_line_of(path_name, entry):
    """一条登记项拼成虚机二进制打的那种 `E7RESULT name=segments path=… operations=… segments=… closed_form=… kinds=…` 行。
    四样缺一样是 CheckError（登记表的写法变了，不猜）。"""
    missing_fields = [name for name, value in (
        ('段序列数字串', entry.segment_sequence_text),
        ('「N 次操作」', entry.operation_count),
        ('「N 个崩溃状态」', entry.crash_state_count),
        ('「种类 `[...]`」', entry.step_kinds_text),
    ) if value is None]
    if missing_fields:
        raise CheckError(f'「{entry.row_label}」那一处登记里认不出 {"、".join(missing_fields)}')
    return (f'{PRODUCT_SEGMENT_LINE_PREFIX}path={path_name} operations={entry.operation_count} '
            f'segments={entry.segment_sequence_text} closed_form={entry.crash_state_count} kinds={entry.step_kinds_text}')


def registered_mkfs_segments_line(layout_text):
    """八节登记表「mkfs 种根」那一行，拼成 `E7RESULT name=segments path=mkfs operations=… segments=… closed_form=… kinds=…`。

    四样（段序列数字串、次操作、崩溃状态数、种类串）都从 row_to_entry 解析出来，与比对那一支是同一份解析；
    那一行不是恰好一行、标了预想、或四样缺一样，都是 CheckError（登记表的写法变了，不猜）。
    """
    section_text = extract_section_eight(layout_text)
    entries = [row_to_entry(row_cells) for row_cells in parse_registry_table_rows(section_text)]
    matched_entries = [entry for entry in entries if entry.row_label == MKFS_ROW_LABEL]
    if len(matched_entries) != 1:
        raise CheckError(f'八节登记表里第一列是「{MKFS_ROW_LABEL}」的行有 {len(matched_entries)} 行，不是 1 行——那一行改了名、被删了或被重复登记')
    entry = matched_entries[0]
    if entry.is_anticipated:
        raise CheckError(f'「{MKFS_ROW_LABEL}」那一行标了预想，没有可比的登记值')
    return segments_line_of(MKFS_PRODUCT_PATH_NAME, entry)


def registered_path_segments_lines(layout_text):
    """八节开头「`path=…`」声明的每一条路径，按声明的次序各拼一行（表格里出处写 path= 的行与整条流那句提示）。"""
    section_text = extract_section_eight(layout_text)
    declared_paths = parse_declared_product_paths(section_text)
    entries_by_path = {}
    for entry in build_registry_entries(section_text):
        if entry.product_path_name is None:
            continue
        if entry.product_path_name in entries_by_path:
            raise CheckError(f'path={entry.product_path_name} 登记了两处（「{entries_by_path[entry.product_path_name].row_label}」与「{entry.row_label}」）')
        entries_by_path[entry.product_path_name] = entry
    lines = []
    for path_name in declared_paths:
        entry = entries_by_path.get(path_name)
        if entry is None:
            raise CheckError(f'八节开头声明了 path={path_name}，表格里却没有出处写它的那一行')
        if entry.is_anticipated:
            raise CheckError(f'path={path_name} 那一行标了预想，没有可比的登记值')
        lines.append(segments_line_of(path_name, entry))
    return lines


def compare_against_pinning_test(entry, repository_root):
    """出处指到一条钉住活代码的用例时，把登记表那一行与用例里写死的串比。

    返回 None 表示「出处里没有这种引用」，让调用方照旧报缺 `path=`；
    返回一张（可能为空的）不符清单表示比过了。

    判据只有一条：用例里那个函数体内，要逐字出现登记表这一行的段序列串与种类串。
    用例自己由 `cargo test` 钉在活代码上——串对不上活代码，那条用例先红。
    """
    source_cell = entry.source_cell
    file_match = re.search(r'`([^`]+\.rs)`', source_cell)
    function_match = re.search(r'`([a-z_][a-z0-9_]*)`', source_cell)
    if not file_match or not function_match:
        return None
    test_path = repository_root / file_match.group(1)
    if not test_path.exists():
        return [f'{entry.row_label}：出处指的用例文件 {file_match.group(1)} 不存在']
    text = test_path.read_text(encoding='utf-8')
    marker = f'fn {function_match.group(1)}('
    start = text.find(marker)
    if start < 0:
        return [f'{entry.row_label}：出处指的用例文件里找不到 `fn {function_match.group(1)}`']
    body = text[start:]
    problems = []
    if entry.segment_sequence_text and f'"{entry.segment_sequence_text}"' not in body:
        problems.append(
            f'{entry.row_label}（钉住活代码的用例 {function_match.group(1)}）：'
            f'kb 写的段序列是 `{entry.segment_sequence_text}`，用例里找不到这个串'
        )
    if entry.step_kinds_text and f'"{entry.step_kinds_text}"' not in body:
        problems.append(
            f'{entry.row_label}（钉住活代码的用例 {function_match.group(1)}）：'
            f'kb 写的种类串与用例里写死的对不上'
        )
    return problems


def pinning_test_function_body(text, function_name):
    """用例文件里 `fn <名>(` 起、到下一个顶格条目之前的那一段；找不到回 None。"""
    marker = f'fn {function_name}('
    start = text.find(marker)
    if start < 0:
        return None
    following = NEXT_TOP_LEVEL_ITEM_PATTERN.search(text, start + len(marker))
    return text[start:following.start() if following else len(text)]


def compare_against_path_pinning_test(entry, repository_root, pinning_test):
    """出处写 path=<名>（或整条流提示）、<名> 登记在 PATH_PINNING_TESTS 里的那一处，与钉它的 crates 用例比。返回不符清单。

    判据（文件头「比对哪几行、跟谁比」）：用例函数体里有一条 `(<次操作>, "<段序列>"[.to_string()], <崩溃状态数>)`，
    种类串逐字出现。比之前把函数体的空白压成一个空格、数字之间的下划线去掉（rustfmt 会折行，clippy 要长整数分组）。
    """
    test_relative_path, function_name = pinning_test
    label = f'{entry.row_label}（path={entry.product_path_name}，钉住它的用例 {function_name}）'
    test_path = repository_root / test_relative_path
    if not test_path.is_file():
        return [f'{label}：钉住这条路径的用例文件 {test_relative_path} 不在']
    body = pinning_test_function_body(test_path.read_text(encoding='utf-8'), function_name)
    if body is None:
        return [f'{label}：{test_relative_path} 里找不到 `fn {function_name}`']
    normalized_body = re.sub(r'(?<=\d)_(?=\d)', '', re.sub(r'\s+', ' ', body))
    sequence = entry.segment_sequence_text
    pinned_triple_pattern = re.compile(
        r'\(\s*(\d+)\s*,\s*"' + re.escape(sequence) + r'"(?:\.to_string\(\))?\s*,\s*(\d+)\s*,?\s*\)'
    )
    pinned_triples = [(int(operations), int(closed_form)) for operations, closed_form in pinned_triple_pattern.findall(normalized_body)]
    problems = []
    if not pinned_triples:
        problems.append(f'{label}：kb 写的段序列是 `{sequence}`，用例里没有钉这个段序列的 (次操作, 段序列, 闭式) 断言')
    elif not any((entry.operation_count is None or operations == entry.operation_count)
                 and (entry.crash_state_count is None or closed_form == entry.crash_state_count)
                 for operations, closed_form in pinned_triples):
        pinned_text = '、'.join(f'({operations}, "{sequence}", {closed_form})' for operations, closed_form in pinned_triples)
        problems.append(
            f'{label}：kb 写 {entry.operation_count if entry.operation_count is not None else "（没写）"} 次操作、'
            f'{entry.crash_state_count if entry.crash_state_count is not None else "（没写）"} 个崩溃状态，用例里钉的是 {pinned_text}'
        )
    if entry.step_kinds_text is None:
        problems.append(f'{label}：这一处没有写「种类 `[...]`」——每段的步骤种类多重集是等价类判据的一半（D17 已定项 2），照钉它的用例抄上')
    elif f'"{entry.step_kinds_text}"' not in body:
        problems.append(f'{label}：kb 写的种类 `{entry.step_kinds_text}` 在用例里找不到逐字相同的串')
    return problems


def parse_product_segment_lines(product_text):
    """产物里 `E7RESULT name=segments path=... operations=... segments=... closed_form=...` 那几行。

    按空格切成 `key=value` 词元，认必要的四个字段，外加可选的 `kinds=[...]`
    （第五次跑起每段的步骤种类多重集，D17 已定项 2 的「每段步骤种类集合」）；
    其余字段原样忽略，产物这一行以后再加字段也不用改这个脚本。`name=segments_three_state` 这类别的行名不算。
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


def compare_against_product(entry, product_entry, product_name):
    """钉不到 crates 用例的那一处登记与 E142 产物的那一行比，返回不符清单（空表 = 相符）。"""
    where = f'{entry.row_label}（path={entry.product_path_name}）'
    if product_entry is None:
        return [f'{entry.row_label}：登记表引用 path={entry.product_path_name}，产物 {product_name} 里没有这个 path 的 name=segments 行']
    if entry.segment_sequence_text != product_entry['segment_sequence_text']:
        return [f'{where}：kb 写的段序列是 `{entry.segment_sequence_text}`，产物里是 `{product_entry["segment_sequence_text"]}`']
    if entry.operation_count is not None and entry.operation_count != product_entry['operation_count']:
        return [f'{where}：kb 写的操作次数是 {entry.operation_count}，产物里 operations 是 {product_entry["operation_count"]}']
    if entry.crash_state_count is not None and entry.crash_state_count != product_entry['closed_form_crash_state_count']:
        return [f'{where}：kb 写的崩溃状态数是 {entry.crash_state_count}，产物里 closed_form 是 {product_entry["closed_form_crash_state_count"]}']
    if entry.step_kinds_text is None:
        return [f'{where}：这一行没有写「种类 `[...]`」——每段的步骤种类多重集是等价类判据的一半（D17 已定项 2），登记表要照产物 kinds= 字段抄上']
    if product_entry['step_kinds_text'] is None:
        return [f'{where}：kb 写了种类 `{entry.step_kinds_text}`，产物这一行却没有 kinds= 字段——产物是第五次跑之前的，要重跑 E142']
    if entry.step_kinds_text != product_entry['step_kinds_text']:
        return [f'{where}：kb 写的种类是 `{entry.step_kinds_text}`，产物里 kinds 是 `{product_entry["step_kinds_text"]}`']
    return []


def closed_form_crash_state_count(segment_lengths):
    """D13（验证路线） 已定项 4 的闭式：1 + Σ(2^|段| − 1)。"""
    return 1 + sum((1 << length) - 1 for length in segment_lengths)


def check_device_pinned_notes(section_text, repository_root):
    """八节里「覆盖写固定脚本流的段序列 `…`、N 次写、M 个状态 … 装置钉住 … `crates/….rs`」那些句子。

    这条流没有干跑产物，登记表能核的是三件事：数字串加起来是不是它自称的写数、
    闭式算出来是不是它自称的状态数、钉它的那份用例文件里有没有这个数组（rustfmt 会把长数组折行，
    所以把用例文件的空白压成一个空格再找 `[a, b, c]`）与那个状态数。
    返回 (mismatches, checked_descriptions)。
    """
    mismatches = []
    checked_descriptions = []
    for match in DEVICE_PINNED_NOTE_PATTERN.finditer(section_text):
        segment_sequence_text, claimed_operation_count, claimed_crash_state_count, test_file_relative_path = (
            match.group(1), int(match.group(2)), int(match.group(3)), match.group(4)
        )
        segment_lengths = [int(term) for term in segment_sequence_text.split('+')]
        label = f'覆盖写固定脚本流 `{segment_sequence_text}`'
        if sum(segment_lengths) != claimed_operation_count:
            mismatches.append(f'{label}：数字串加起来是 {sum(segment_lengths)} 次写，kb 写的是 {claimed_operation_count}')
        computed_crash_state_count = closed_form_crash_state_count(segment_lengths)
        if computed_crash_state_count != claimed_crash_state_count:
            mismatches.append(
                f'{label}：闭式 1 + Σ(2^|段| − 1) 算出 {computed_crash_state_count} 个状态，kb 写的是 {claimed_crash_state_count}'
            )
        test_file_path = repository_root / test_file_relative_path
        if not test_file_path.is_file():
            mismatches.append(f'{label}：装置钉住指向的用例文件 {test_file_relative_path} 不在')
            continue
        normalized_test_text = re.sub(r'\s+', ' ', test_file_path.read_text(encoding='utf-8'))
        array_text = '[' + ', '.join(str(length) for length in segment_lengths) + ']'
        # rustfmt 把长数组折行之后 `[` 后面、`]` 前面各多一个换行，压成空白再按「可有可无的空白」找
        array_pattern = re.compile(r'\[\s*' + re.escape(', '.join(str(length) for length in segment_lengths)) + r'\s*\]')
        array_count = len(array_pattern.findall(normalized_test_text))
        if array_count == 0:
            mismatches.append(f'{label}：用例文件 {test_file_relative_path} 里没有钉这个数组 `{array_text}`')
        # Rust 的长整数字面量带下划线分组（clippy unreadable_literal 要求，例 `6_649_413_746`），比状态数之前把数字之间的下划线去掉
        digits_joined_test_text = normalized_test_text if broken('underscore-kept') else re.sub(r'(?<=\d)_(?=\d)', '', normalized_test_text)
        if str(claimed_crash_state_count) not in digits_joined_test_text:
            mismatches.append(f'{label}：用例文件 {test_file_relative_path} 里没有钉状态数 {claimed_crash_state_count}')
        checked_descriptions.append(f'{label}（{claimed_operation_count} 次写、{claimed_crash_state_count} 个状态，数组在 {test_file_relative_path} 里出现 {array_count} 次）')
    if not checked_descriptions and not broken('unrecognized-note-accepted'):
        # 一句都认不出时不许当成「没有要核的」放过：那句登记被改了形状，这一格就再没人核（扫到 0 项不是通过）
        mismatches.append('八节里一句「覆盖写固定脚本流的段序列 `…`、N 次写、M 个状态 … 装置钉住 … `crates/….rs`」的登记都认不出：登记句的形状改了，覆盖写固定脚本流就没人核')
    return mismatches, checked_descriptions


def replace_once_inside(layout_text, region_text, original_span, mutated_span, where):
    """--selftest 用：只在 `region_text`（八节，或八节表里的那一行）里把 `original_span` 换成 `mutated_span`，再把那一段放回原文。
    要求那一串在这一段里恰好出现一次、这一段在整份文件里恰好出现一次：kb 别处写着同一串不碍事，换错了地方的也不会换。"""
    if region_text.count(original_span) != 1:
        raise CheckError(f'{where}里 {original_span} 出现了 {region_text.count(original_span)} 次，不是 1 次——自证定位不到唯一位置')
    if layout_text.count(region_text) != 1:
        raise CheckError(f'{where}这一段在整份文件里出现了 {layout_text.count(region_text)} 次，不是 1 次——自证放不回原处')
    return layout_text.replace(region_text, region_text.replace(original_span, mutated_span, 1), 1)


def registry_row_line_of(section_text, row_label, original_span):
    """八节登记表里以 `row_label` 起头、又含 `original_span` 的那一行原文（自证只在这一行里动刀）。"""
    lines = [line for line in section_text.split('\n') if line.strip().startswith('|') and row_label in line and original_span in line]
    if len(lines) != 1:
        raise CheckError(f'八节登记表里「{row_label}」又含 {original_span} 的行有 {len(lines)} 行，不是 1 行——自证定位不到唯一那一行')
    return lines[0]


def mutate_device_pinned_note(layout_text):
    """--selftest 用：把第一句「覆盖写固定脚本流的段序列」的最后一项加 1——闭式与用例里的数组都该对不上。"""
    section_text = extract_section_eight(layout_text)
    match = DEVICE_PINNED_NOTE_PATTERN.search(section_text)
    if match is None:
        raise CheckError('八节里找不到「覆盖写固定脚本流的段序列 `…`、N 次写、M 个状态 … 装置钉住 … `crates/….rs`」这句登记，自证没法造出可判红的改动')
    original_segment_sequence_text = match.group(1)
    original_span = f'`{original_segment_sequence_text}`'
    terms = original_segment_sequence_text.split('+')
    terms[-1] = str(int(terms[-1]) + 1)
    mutated_segment_sequence_text = '+'.join(terms)
    return (replace_once_inside(layout_text, section_text, original_span, f'`{mutated_segment_sequence_text}`', '八节'),
            original_segment_sequence_text, mutated_segment_sequence_text, match.group(4))


def mutate_device_pinned_note_shape(layout_text):
    """--selftest 用：把八节里每一句「覆盖写固定脚本流的段序列」登记的「N 次写」改成「N 笔写」，句子留着、数不动，DEVICE_PINNED_NOTE_PATTERN 一句都认不出。
    返回 (改过的全文, 改前那一段, 改后那一段)。改完还认得出一句就抛 CheckError（造不出「认不出」的输入）。"""
    section_text = extract_section_eight(layout_text)
    matches = list(DEVICE_PINNED_NOTE_PATTERN.finditer(section_text))
    if not matches:
        raise CheckError('八节里找不到「覆盖写固定脚本流的段序列 `…`、N 次写、M 个状态 … 装置钉住 … `crates/….rs`」这句登记，自证没法造出认不出的形状')
    mutated_text = layout_text
    for match in matches:
        operations_match = re.search(r'\d+\s*次写', match.group(0))
        original_span = operations_match.group(0)
        mutated_span = original_span.replace('次写', '笔写', 1)
        sentence_mutated = match.group(0).replace(original_span, mutated_span, 1)
        mutated_text = replace_once_inside(mutated_text, extract_section_eight(mutated_text), match.group(0), sentence_mutated, '八节')
    if DEVICE_PINNED_NOTE_PATTERN.search(extract_section_eight(mutated_text)):
        raise CheckError('把「N 次写」改成「N 笔写」之后八节里还认得出覆盖写固定脚本流的登记句，自证造不出认不出的形状')
    return mutated_text, original_span, mutated_span


def underscore_grouped_pinned_test_text(test_text, crash_state_count):
    """--selftest 用：钉它的用例文件里把状态数的每一处十进制写法（不带下划线的）换成三位一组带下划线的 Rust 写法（例 `1_678_377_095`）。
    返回 (改过的正文, 换了几处, 带下划线的那一串)。换完还剩不带下划线的写法、或一处都没换，抛 CheckError。"""
    plain_text = str(crash_state_count)
    grouped_text = f'{crash_state_count:_}'
    plain_pattern = re.compile(r'(?<![\d_])' + re.escape(plain_text) + r'(?![\d_])')
    rewritten_text, replaced_count = plain_pattern.subn(grouped_text, test_text)
    if '_' not in grouped_text:
        raise CheckError(f'状态数 {plain_text} 不到四位，写不成带下划线的形式，自证造不出那一格的输入')
    if plain_pattern.search(rewritten_text):
        raise CheckError(f'用例文件里换完还剩不带下划线的状态数 {plain_text}，自证造不出「只有带下划线的钉值」的输入')
    if replaced_count == 0 and grouped_text not in rewritten_text:
        raise CheckError(f'用例文件里既没有 {plain_text} 也没有 {grouped_text}，自证造不出那一格的输入')
    return rewritten_text, replaced_count, grouped_text


def segment_cell_replaced(row_line, original_span, mutated_span, where):
    """--selftest 用：只在登记表那一行的「段序列」栏（第二列）里换：row_to_entry 取的段序列是这一栏里最后一个数字串，这里换的也是这一栏里最后一处。
    出处栏里引的同一串（取号那一行写着「两轮的 `2+2` 由……钉住」）一个字不动。切栏只认没转义的竖线（与 parse_registry_table_rows 同一个认法），切完原样接回。"""
    pieces = UNESCAPED_PIPE_PATTERN.split(row_line)
    if len(pieces) < 4 or original_span not in pieces[2]:
        raise CheckError(f'{where}的段序列栏里找不到 {original_span}——自证定位不到那一栏')
    cell = pieces[2]
    at = cell.rfind(original_span)
    pieces[2] = cell[:at] + mutated_span + cell[at + len(original_span):]
    return '|'.join(pieces)


def mutate_one_segment_number(layout_text, row_label=None, whole_line=False):
    """--selftest 用：把八节表格第一条可比对的行（给了 row_label 就只看第一列是它的那一行），段序列的最后一项加 1。

    只在那一行的段序列栏里换（`segment_cell_replaced`），免得替换的时候顺手改坏了别的地方——那样测出来的「判红」证明的是别的东西，不是这条检查本身；
    同一行的出处栏里引着同一串也不碍事。whole_line=True 是改前的换法（在整行里换，同一行里那一串出现两次就定位不到）：
    只给「同一行两处同串」那一格在弄坏开关 segment-cell-unscoped 下用，真仓备料的几处一律照段序列栏换。
    """
    section_text = extract_section_eight(layout_text)
    for row_cells in parse_registry_table_rows(section_text):
        entry = row_to_entry(row_cells)
        if entry.is_anticipated or entry.is_device_pinned or entry.segment_sequence_text is None:
            continue
        if row_label is not None and entry.row_label != row_label:
            continue
        original_segment_sequence_text = entry.segment_sequence_text
        original_span = f'`{original_segment_sequence_text}`'
        row_line = registry_row_line_of(section_text, entry.row_label, original_span)
        terms = original_segment_sequence_text.split('+')
        terms[-1] = str(int(terms[-1]) + 1)
        mutated_segment_sequence_text = '+'.join(terms)
        where = f'「{entry.row_label}」那一行'
        if whole_line:
            mutated_layout_text = replace_once_inside(layout_text, row_line, original_span, f'`{mutated_segment_sequence_text}`', where)
        else:
            mutated_row_line = segment_cell_replaced(row_line, original_span, f'`{mutated_segment_sequence_text}`', where)
            mutated_layout_text = replace_once_inside(layout_text, row_line, row_line, mutated_row_line, where)
        return mutated_layout_text, entry.row_label, original_segment_sequence_text, mutated_segment_sequence_text
    raise CheckError('八节的登记表里找不到一条「不是预想、且带段序列数字串」的行，自证没法造出一个可判红的改动')


def mutate_operation_count(layout_text, row_label):
    """--selftest 用：把第一列是 row_label 的那一行「N 次操作」加 1（段序列与种类一个字不动）。"""
    section_text = extract_section_eight(layout_text)
    entries = [row_to_entry(row_cells) for row_cells in parse_registry_table_rows(section_text)]
    entry = next((entry for entry in entries if entry.row_label == row_label), None)
    if entry is None or entry.operation_count is None:
        raise CheckError(f'八节登记表里找不到第一列是「{row_label}」、写了「N 次操作」的行，自证没法改它的次操作')
    original_span = f'{entry.operation_count} 次操作'
    row_line = registry_row_line_of(section_text, row_label, original_span)
    mutated_span = f'{entry.operation_count + 1} 次操作'
    return replace_once_inside(layout_text, row_line, original_span, mutated_span, f'「{row_label}」那一行'), original_span, mutated_span


def mutate_one_step_kind(layout_text):
    """--selftest 用：把八节表格第一条带「种类」的行，第一段里第一个种类的重数改掉（×4 → ×5，没有重数就加 ×2）。

    段序列一个字不动，所以这一刀只有「种类」那一半的比对能看见。
    """
    section_text = extract_section_eight(layout_text)
    for row_cells in parse_registry_table_rows(section_text):
        entry = row_to_entry(row_cells)
        if entry.is_anticipated or entry.is_device_pinned or entry.step_kinds_text is None:
            continue
        # 表格单元格里的竖线在文件里写成 `\\|`，定位原文要按转义后的样子找。
        original_span = '`' + entry.step_kinds_text.replace('|', '\\|') + '`'
        row_line = registry_row_line_of(section_text, entry.row_label, original_span)
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
        mutated_layout_text = replace_once_inside(layout_text, row_line, original_span, mutated_span, f'「{entry.row_label}」那一行')
        return mutated_layout_text, entry.row_label, first_kind, mutated_kind
    raise CheckError('八节的登记表里找不到一条带「种类 `[...]`」的行，自证没法造出一个只改种类的改动')


def mutate_declared_paths_with_extra(layout_text):
    """--selftest 用：八节开头那句「`path=a / b / …`」声明末尾多写一条没有表格行占它的路径。

    声明读的是八节里第一处「`path=…`」（parse_declared_product_paths），所以只在八节里换那一处；
    表格行与整条流提示一个字不动，排除之后就从剩一条变成剩两条。返回 (改后的全文, 声明原文, 改后的声明, 改后排除之后该剩几条)。
    """
    section_text = extract_section_eight(layout_text)
    declared_match = DECLARED_PRODUCT_PATHS_PATTERN.search(section_text)
    if declared_match is None:
        raise CheckError('八节里找不到「`path=instance_acquisition / warm_up / …`」这句声明，自证没法往里多写一条路径')
    declared_paths = parse_declared_product_paths(section_text)
    if EXTRA_DECLARED_PATH_NAME in declared_paths:
        raise CheckError(f'八节开头的声明里已经有 {EXTRA_DECLARED_PATH_NAME}，自证起的多余路径名要换一个')
    claimed_paths = {entry.product_path_name for entry in (row_to_entry(cells) for cells in parse_registry_table_rows(section_text))
                     if entry.product_path_name is not None}
    original_span = declared_match.group(0)
    mutated_span = original_span[:-1] + f' / {EXTRA_DECLARED_PATH_NAME}`'
    expected_remaining_count = len([path for path in declared_paths + [EXTRA_DECLARED_PATH_NAME] if path not in claimed_paths])
    return (replace_once_inside(layout_text, section_text, original_span, mutated_span, '八节'),
            original_span, mutated_span, expected_remaining_count)


def perform_check(repository_root, path_pinning_tests=None):
    """跑一遍完整比对，返回 (退出码, 要打印的每一行)。path_pinning_tests 默认取 PATH_PINNING_TESTS（--selftest 拿掉一条，造「只能与产物比」）。"""
    if path_pinning_tests is None:
        path_pinning_tests = PATH_PINNING_TESTS
    if broken('path-rows-to-product'):
        path_pinning_tests = {}
    product_path = None
    product_segment_lines = None
    try:
        layout_path = repository_root / LAYOUT_RELATIVE_PATH
        if not layout_path.is_file():
            return 2, [f'  ✗ 找不到 {layout_path}',
                       f'     → 怎么办：段序列登记表在 {LAYOUT_RELATIVE_PATH}「八、」；挪了位置就改这个脚本的 LAYOUT_RELATIVE_PATH。']
        layout_text = layout_path.read_text(encoding='utf-8')
        section_text = extract_section_eight(layout_text)
        entries = build_registry_entries(section_text)
        # 只有出处写 path=<名> 而 <名> 没登记钉住它的 crates 用例的那几处才要 E142 产物；一处都没有就不去找产物
        entries_needing_product = [entry for entry in entries
                                   if not entry.is_anticipated and not entry.is_device_pinned
                                   and entry.product_path_name is not None and entry.segment_sequence_text is not None
                                   and entry.product_path_name not in path_pinning_tests]
        if entries_needing_product:
            product_path = locate_e142_product_path(repository_root)
            if product_path.is_file():
                product_segment_lines = parse_product_segment_lines(product_path.read_text(encoding='utf-8'))
            elif broken('product-required'):
                return 2, [f'  ✗ replay.sh 指到了 {product_path}，但这份产物不在',
                           '     → 怎么办：（弄坏开关 product-required 下的旧判法）照复跑登记重跑 E142。']
    except CheckError as error:
        return 2, [f'  ✗ {error}',
                   f'     → 怎么办：照上面那句修：{LAYOUT_RELATIVE_PATH}「八、」一节的写法（标题、开头「`path=…`」声明、表格、整条流那句提示）变了，就对着这个脚本文件头写的认法改；'
                   'replay.sh 或它的 E142 行不在，钉不到 crates 用例的那几处就定不下产物路径：给那条路径登记钉住它的用例（PATH_PINNING_TESTS），或把复跑登记补回来。']

    mismatches = []
    compared_table_row_count = 0
    compared_note_count = 0
    path_pinned_count = 0
    product_compared_count = 0
    skipped_count = 0
    device_pinned_row_count = 0
    live_pinned_row_count = 0
    not_run_entries = []
    path_pinning_files = sorted({pinning_test[0] for pinning_test in path_pinning_tests.values()})
    for entry in entries:
        if entry.is_anticipated:
            skipped_count += 1
            continue
        if entry.is_device_pinned:
            device_pinned_row_count += 1
            continue
        if entry.product_path_name is None:
            # 出处指到一条钉住活代码的用例（`<文件>.rs` 加一个测试函数名）时，按那条用例里写死的串比：
            # mkfs 那一行 2026-09-22 起就是这一种（C485（segment-registry 格不读活代码，段序列变了照样绿））。
            pinned = compare_against_pinning_test(entry, repository_root)
            if pinned is None:
                mismatches.append(f'{entry.row_label}：这一行不是预想，「出处」栏里既没有 `path=...` 引用、也没有指到一条钉住活代码的用例（`<文件>.rs` 加测试函数名），登记表这一行本身要修')
            else:
                mismatches.extend(pinned)
                live_pinned_row_count += 1
            continue
        if entry.segment_sequence_text is None:
            mismatches.append(f'{entry.row_label}：这一行不是预想，却没有可比对的段序列数字串')
            continue
        if entry.product_path_name in path_pinning_tests:
            problems = compare_against_path_pinning_test(entry, repository_root, path_pinning_tests[entry.product_path_name])
            path_pinned_count += 1
        elif product_segment_lines is None:
            not_run_entries.append(entry)       # 只能与产物比、产物不在：本次未跑，不判红
            continue
        else:
            problems = compare_against_product(entry, product_segment_lines.get(entry.product_path_name), product_path.name)
            product_compared_count += 1
        if problems:
            mismatches.extend(problems)
            continue
        if entry.origin_description == '表格行':
            compared_table_row_count += 1
        else:
            compared_note_count += 1

    device_pinned_mismatches, device_pinned_descriptions = check_device_pinned_notes(section_text, repository_root)
    mismatches.extend(device_pinned_mismatches)

    if mismatches:
        lines = [f'  ✗ 段序列登记表与钉住活代码的用例（钉不到用例的行与 E142 产物的 name=segments 行；覆盖写固定脚本流与钉它的用例）对不上，共 {len(mismatches)} 处：']
        for mismatch in mismatches:
            lines.append(f'     {mismatch}')
        lines.append('     → 怎么办：表是抄错了，就照钉住那条路径的用例改 .claude/kb/layout/01-first-txn.md 那一行（用例由 cargo test 钉在活代码上，权威在用例）；')
        lines.append('       用例文件或函数改了名，照今天的名字改出处栏或这个脚本的 PATH_PINNING_TESTS；')
        lines.append('       钉不到用例、只与产物比的那一行，产物过期了就重跑 E142，再让 scripts/experiment-apparatus/replay.sh 里 E142 那一行的入库产物名字跟上。')
        return 1, lines

    compared_total_count = compared_table_row_count + compared_note_count
    summary = (
        f'比对了 {compared_total_count} 处登记（表格 {compared_table_row_count} 行 + 整条流提示 {compared_note_count} 处；'
        f'其中 {path_pinned_count} 处与钉住那条路径的 crates 用例比（{"、".join(path_pinning_files) or "无"}），'
        f'{product_compared_count} 处与 E142 产物{f" {product_path.name} " if product_compared_count else ""}的 name=segments 行比，'
        '段序列、次操作、崩溃状态数与每段步骤种类多重集都相符），'
        f'跳过 {skipped_count} 条标预想的表格行；'
        f'{live_pinned_row_count} 条的出处指到钉住活代码的用例、按那条用例里写死的串比（不按产物比）；'
        f'{device_pinned_row_count} 条标「装置钉住」的表格行不逐行比（形状从覆盖写固定脚本流推得），'
        f'覆盖写固定脚本流的登记 {len(device_pinned_descriptions)} 处与钉它的用例相符：' + '；'.join(device_pinned_descriptions)
    )
    if not_run_entries:
        return 77, [
            f'  ! 本次未跑 {len(not_run_entries)} 处登记：' + '、'.join(f'{entry.row_label}（path={entry.product_path_name}）' for entry in not_run_entries) +
            f' 没有钉住它的 crates 用例（PATH_PINNING_TESTS 里没登记），只能与 E142 产物比，而 replay.sh 指的 {product_path} 不在'
            '（research/ 整个可删，产物不在不判红，也不记通过）',
            '     → 怎么办：在 crates 里找到（或写）钉住这条路径的用例，登记进 scripts/crash/check-segment-registry.py 的 PATH_PINNING_TESTS；'
            '要按产物判就照复跑登记重跑 E142，把产物放回 research/results/。',
            f'  · 其余：{summary}',
        ]
    return 0, [f'  ✓ {summary}']


def perform_print_mkfs_segments_line(repository_root):
    """--print-mkfs-segments-line：返回 (退出码, 要打印的每一行)。成功时恰好一行，就是拼好的那一行。"""
    layout_path = repository_root / LAYOUT_RELATIVE_PATH
    try:
        if not layout_path.is_file():
            raise CheckError(f'找不到 {layout_path}')
        return 0, [registered_mkfs_segments_line(layout_path.read_text(encoding='utf-8'))]
    except CheckError as error:
        return 2, [
            f'  ✗ 取不出八节登记表「{MKFS_ROW_LABEL}」那一行的登记值：{error}',
            f'     → 怎么办：看 {LAYOUT_RELATIVE_PATH}「八、」登记表第一列是「{MKFS_ROW_LABEL}」的那一行：恰好一行，'
            '段序列栏里写全 `数字串`、「N 次操作」、「N 个崩溃状态」与「种类 `[...]`」（照钉它的那条用例写）。',
        ]


def perform_print_segments_lines(repository_root):
    """--print-segments-lines：返回 (退出码, 要打印的每一行)。成功时每条声明的路径一行。"""
    layout_path = repository_root / LAYOUT_RELATIVE_PATH
    try:
        if not layout_path.is_file():
            raise CheckError(f'找不到 {layout_path}')
        return 0, registered_path_segments_lines(layout_path.read_text(encoding='utf-8'))
    except CheckError as error:
        return 2, [
            f'  ✗ 取不出八节登记表里 path= 那几条路径的登记值：{error}',
            f'     → 怎么办：看 {LAYOUT_RELATIVE_PATH}「八、」开头「`path=…`」那句声明与表格里出处写 path= 的行、整条流那句提示：'
            '每条声明的路径恰好一处登记，写全 `数字串`、「N 次操作」、「N 个崩溃状态」与「种类 `[...]`」（照钉它的那条用例写）。',
        ]


def build_fixture_root(root_path, layout_text, pinned_test_files, replay_script_text=None, product=None):
    """在临时目录里按仓库的相对路径摆好布局文件与它点名的用例文件；要与产物比的那一支另摆 replay.sh 与 (产物名, 产物正文)。"""
    files = {LAYOUT_RELATIVE_PATH: layout_text, **{pathlib.Path(relative): text for relative, text in pinned_test_files.items()}}
    if replay_script_text is not None:
        files[REPLAY_SCRIPT_RELATIVE_PATH] = replay_script_text
    if product is not None:
        files[RESULTS_DIRECTORY_RELATIVE_PATH / product[0]] = product[1]
    for relative_path, text in files.items():
        path = root_path / relative_path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')
    return root_path


def run_self_test():
    real_repository_root = default_repository_root()
    try:
        layout_text = (real_repository_root / LAYOUT_RELATIVE_PATH).read_text(encoding='utf-8')
        product_sample_text = PRODUCT_SAMPLE_PATH.read_text(encoding='utf-8')
        section_text = extract_section_eight(layout_text)
        entries = build_registry_entries(section_text)
        (mutated_layout_text, mutated_row_label,
         original_segment_sequence_text, mutated_segment_sequence_text) = mutate_one_segment_number(layout_text)
        (kind_mutated_layout_text, kind_mutated_row_label,
         original_kind_text, mutated_kind_text) = mutate_one_step_kind(layout_text)
        (pinned_mutated_layout_text, original_pinned_text,
         mutated_pinned_text, pinned_test_relative_path) = mutate_device_pinned_note(layout_text)
        # 八节开头的 path= 声明多写一条没有表格行占它的路径，排除之后剩两条，要退 2（CheckError 那一支）
        (extra_path_layout_text, _original_declaration,
         mutated_declaration, expected_extra_path_remaining_count) = mutate_declared_paths_with_extra(layout_text)
        # 出处写 path= 的第一条表格行：与钉它的 crates 用例比的那一支（段序列一刀、次操作一刀），与只能与产物比的那一支（拿掉它的用例登记）
        path_entry = next((entry for entry in entries
                           if entry.origin_description == '表格行' and entry.product_path_name in PATH_PINNING_TESTS
                           and not entry.is_anticipated and not entry.is_device_pinned), None)
        if path_entry is None:
            raise CheckError('真仓八节登记表里没有一条出处写 path=、又登记在 PATH_PINNING_TESTS 里的表格行，与 crates 用例比的那一支没得测')
        (path_mutated_layout_text, _path_row_label,
         original_path_sequence_text, mutated_path_sequence_text) = mutate_one_segment_number(layout_text, path_entry.row_label)
        (operations_mutated_layout_text, original_operations_span, mutated_operations_span) = mutate_operation_count(layout_text, path_entry.row_label)
        unpinned_tests = {name: test for name, test in PATH_PINNING_TESTS.items() if name != path_entry.product_path_name}
        replay_script_text = f'E142|@driver_e142||{SELFTEST_PRODUCT_FILE_NAME}|exact\n'
        product_green_text = segments_line_of(path_entry.product_path_name, path_entry) + '\n'
        terms = path_entry.segment_sequence_text.split('+')
        terms[-1] = str(int(terms[-1]) + 1)
        product_red_sequence_text = '+'.join(terms)
        product_red_text = product_green_text.replace(f' segments={path_entry.segment_sequence_text} ', f' segments={product_red_sequence_text} ', 1)
        if product_red_text == product_green_text:
            raise CheckError('照登记现造的产物行里改不动段序列，要与产物比的那一支造不出红的输入')
        # --print-mkfs-segments-line 那一支的三份：mkfs 那一行段序列最后一项加 1；那一行第一列改名（0 行）；那一行抄两遍（2 行）
        (mkfs_mutated_layout_text, _mkfs_row_label,
         original_mkfs_sequence_text, mutated_mkfs_sequence_text) = mutate_one_segment_number(layout_text, MKFS_ROW_LABEL)
        mkfs_row_line = registry_row_line_of(section_text, MKFS_ROW_LABEL, f'| {MKFS_ROW_LABEL} |')
        mkfs_renamed_layout_text = replace_once_inside(layout_text, mkfs_row_line, f'| {MKFS_ROW_LABEL} |', f'| {MKFS_ROW_LABEL}（改名） |',
                                                       f'「{MKFS_ROW_LABEL}」那一行')
        mkfs_duplicated_layout_text = replace_once_inside(layout_text, section_text, mkfs_row_line, mkfs_row_line + '\n' + mkfs_row_line, '八节')
        mkfs_entries = [entry for entry in entries if entry.row_label == MKFS_ROW_LABEL]
        if len(mkfs_entries) != 1:
            raise CheckError(f'真仓八节登记表里第一列是「{MKFS_ROW_LABEL}」的行有 {len(mkfs_entries)} 行，不是 1 行')
        expected_mkfs_fields = {
            'operation_count': mkfs_entries[0].operation_count,
            'segment_sequence_text': mkfs_entries[0].segment_sequence_text,
            'closed_form_crash_state_count': mkfs_entries[0].crash_state_count,
            'step_kinds_text': mkfs_entries[0].step_kinds_text,
        }
        declared_paths = parse_declared_product_paths(section_text)
        entries_by_path = {entry.product_path_name: entry for entry in entries if entry.product_path_name is not None}
        # 自证要摆进临时目录的用例文件：覆盖写固定脚本流那句点名的、八节里反引号点名的 .rs、PATH_PINNING_TESTS 登记的
        # （不摆的话那一支找不到文件、整道自检红，而红的理由与它要证明的事无关——`sop-first.md`：造一个该被拦的输入，不是备料不全）
        pinned_test_files = {}
        wanted_relative_paths = [match.group(4) for match in DEVICE_PINNED_NOTE_PATTERN.finditer(section_text)]
        wanted_relative_paths += [match.group(1) for match in re.finditer(r'`([^`]+\.rs)`', section_text)]
        wanted_relative_paths += [pinning_test[0] for pinning_test in PATH_PINNING_TESTS.values()]
        for relative_path in wanted_relative_paths:
            candidate = real_repository_root / relative_path
            if relative_path not in pinned_test_files and candidate.is_file():
                pinned_test_files[relative_path] = candidate.read_text(encoding='utf-8')
        # 覆盖写固定脚本流那句改成认不出的形状（「N 次写」写成「N 笔写」）：一句都认不出要判红，不当「没有要核的」放过
        (shape_mutated_layout_text, original_shape_span, mutated_shape_span) = mutate_device_pinned_note_shape(layout_text)
        # 钉它的用例文件的临时拷贝里，状态数只剩带下划线的 Rust 写法（clippy unreadable_literal 要的分组）：比之前去掉数字间的下划线，要判绿
        underscore_test_files = dict(pinned_test_files)
        underscore_descriptions = []
        for match in DEVICE_PINNED_NOTE_PATTERN.finditer(section_text):
            if match.group(4) not in underscore_test_files:
                raise CheckError(f'覆盖写固定脚本流那句点名的用例文件 {match.group(4)} 不在，带下划线那一格造不出输入')
            (underscore_test_files[match.group(4)], underscore_replaced_count,
             underscore_grouped_text) = underscore_grouped_pinned_test_text(underscore_test_files[match.group(4)], int(match.group(3)))
            underscore_descriptions.append(f'{match.group(4)} 里状态数换成 {underscore_grouped_text}（换了 {underscore_replaced_count} 处）')
    except (CheckError, OSError) as error:
        print(f'  ✗ --selftest 备料就失败了：{error}')
        print('     → 怎么办：先跑一次不带 --selftest 的检查，确认真实仓库能正常读出登记表与钉它的用例；'
              f'样本 {PRODUCT_SAMPLE_PATH.name} 跟着脚本放在 scripts/crash/ 下，不在就从 git 找回来。')
        return 1

    # 默认落 tempfile 的标准临时目录（尊重 TMPDIR 环境变量），不写死到某一次会话的 scratchpad 路径
    workspace_path = pathlib.Path(tempfile.mkdtemp(prefix='check-segment-registry-selftest-'))
    try:
        def fixture(name, text, **options):
            return build_fixture_root(workspace_path / name, text, pinned_test_files, **options)

        green_root = fixture('green', layout_text)
        green_exit_code, green_lines = perform_check(green_root)
        red_exit_code, _red_lines = perform_check(fixture('red', mutated_layout_text))
        kind_red_exit_code, _kind_red_lines = perform_check(fixture('kind-red', kind_mutated_layout_text))
        pinned_red_exit_code, _pinned_red_lines = perform_check(fixture('pinned-red', pinned_mutated_layout_text))
        shape_red_exit_code, shape_red_lines = perform_check(fixture('pinned-shape-red', shape_mutated_layout_text))
        underscore_green_exit_code, underscore_green_lines = perform_check(
            build_fixture_root(workspace_path / 'pinned-underscore-green', layout_text, underscore_test_files))
        extra_path_exit_code, extra_path_lines = perform_check(fixture('extra-path', extra_path_layout_text))
        path_red_exit_code, path_red_lines = perform_check(fixture('path-red', path_mutated_layout_text))
        operations_red_exit_code, operations_red_lines = perform_check(fixture('operations-red', operations_mutated_layout_text))
        product_absent_exit_code, product_absent_lines = perform_check(
            fixture('product-absent', layout_text, replay_script_text=replay_script_text), unpinned_tests)
        product_green_exit_code, product_green_lines = perform_check(
            fixture('product-green', layout_text, replay_script_text=replay_script_text,
                    product=(SELFTEST_PRODUCT_FILE_NAME, product_green_text)), unpinned_tests)
        product_red_exit_code, product_red_lines = perform_check(
            fixture('product-red', layout_text, replay_script_text=replay_script_text,
                    product=(SELFTEST_PRODUCT_FILE_NAME, product_red_text)), unpinned_tests)
        mkfs_green_exit_code, mkfs_green_lines = perform_print_mkfs_segments_line(green_root)
        mkfs_red_exit_code, mkfs_red_lines = perform_print_mkfs_segments_line(fixture('mkfs-red', mkfs_mutated_layout_text))
        mkfs_renamed_exit_code, mkfs_renamed_lines = perform_print_mkfs_segments_line(fixture('mkfs-renamed', mkfs_renamed_layout_text))
        mkfs_duplicated_exit_code, mkfs_duplicated_lines = perform_print_mkfs_segments_line(fixture('mkfs-duplicated', mkfs_duplicated_layout_text))
        lines_green_exit_code, lines_green_lines = perform_print_segments_lines(green_root)
        lines_red_exit_code, lines_red_lines = perform_print_segments_lines(workspace_path / 'path-red')
    finally:
        shutil.rmtree(workspace_path, ignore_errors=True)

    problems = []
    if green_exit_code != 0:
        problems.append('未改动的拷贝（不摆产物）本该判绿，实际退出码 ' + str(green_exit_code) + '：\n       ' + '\n       '.join(green_lines))
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
    if pinned_red_exit_code == 0:
        problems.append(
            f'把覆盖写固定脚本流的段序列 `{original_pinned_text}` 改成 `{mutated_pinned_text}` 之后，检查仍然判绿——'
            f'装置钉住那一半（闭式、{pinned_test_relative_path} 里的数组）没有判别力'
        )
    if shape_red_exit_code != 1 or '都认不出' not in '\n'.join(shape_red_lines):
        problems.append(
            f'覆盖写固定脚本流的登记句改成认不出的形状（「{original_shape_span}」写成「{mutated_shape_span}」）之后本该退 1、点出「登记都认不出」，'
            f'实际退 {shape_red_exit_code}：' + ' / '.join(shape_red_lines)
        )
    if underscore_green_exit_code != 0:
        problems.append(
            f'钉它的用例文件拷贝里状态数只剩带下划线的写法（{"；".join(underscore_descriptions)}）之后本该判绿，'
            f'实际退 {underscore_green_exit_code}：' + ' / '.join(underscore_green_lines)
        )
    if extra_path_exit_code != 2 or f'实际剩 {expected_extra_path_remaining_count} 条' not in '\n'.join(extra_path_lines):
        problems.append(
            f'八节开头「`path=…`」声明里多写了一条 `{EXTRA_DECLARED_PATH_NAME}`（{mutated_declaration}，没有表格行占它）之后，'
            f'检查本该退 2、消息点出「实际剩 {expected_extra_path_remaining_count} 条」，实际退 {extra_path_exit_code}：' + ' / '.join(extra_path_lines)
        )
    pinning_function_name = PATH_PINNING_TESTS[path_entry.product_path_name][1]
    if path_red_exit_code != 1 or pinning_function_name not in '\n'.join(path_red_lines):
        problems.append(
            f'把「{path_entry.row_label}」（path={path_entry.product_path_name}）的段序列从 `{original_path_sequence_text}` 改成 `{mutated_path_sequence_text}` 之后，'
            f'与钉它的用例 {pinning_function_name} 比本该退 1 并点名那条用例，实际退 {path_red_exit_code}：' + ' / '.join(path_red_lines)
        )
    if operations_red_exit_code != 1 or pinning_function_name not in '\n'.join(operations_red_lines):
        problems.append(
            f'把「{path_entry.row_label}」的「{original_operations_span}」改成「{mutated_operations_span}」之后（段序列与种类不动），'
            f'与钉它的用例比本该退 1，实际退 {operations_red_exit_code}：' + ' / '.join(operations_red_lines)
        )
    if product_absent_exit_code != 77 or path_entry.row_label not in '\n'.join(product_absent_lines):
        problems.append(
            f'只能与产物比、产物不在的那一行本该退 77（本次未跑）并点名「{path_entry.row_label}」，'
            f'实际退 {product_absent_exit_code}：' + ' / '.join(product_absent_lines)
        )
    if product_green_exit_code != 0:
        problems.append(f'只能与产物比、摆上照登记现造的产物之后本该判绿，实际退 {product_green_exit_code}：' + ' / '.join(product_green_lines))
    if product_red_exit_code != 1 or f'产物里是 `{product_red_sequence_text}`' not in '\n'.join(product_red_lines):
        problems.append(
            f'只能与产物比、产物里段序列改成 `{product_red_sequence_text}` 之后本该退 1 并点出产物里的值，'
            f'实际退 {product_red_exit_code}：' + ' / '.join(product_red_lines)
        )
    try:
        sample_segment_lines = parse_product_segment_lines(product_sample_text)
    except CheckError as error:
        sample_segment_lines = {'认不出': str(error)}
    if sorted(sample_segment_lines) != sorted(PRODUCT_SAMPLE_PATHS) or any(
            not isinstance(fields, dict) or fields['step_kinds_text'] is None for fields in sample_segment_lines.values()):
        problems.append(
            f'样本 {PRODUCT_SAMPLE_PATH.name} 的产物行本该解析出 {"、".join(PRODUCT_SAMPLE_PATHS)} 五条路径、都带种类'
            f'（name=segments_three_state 这类别的行名不算），实际是 {sample_segment_lines}'
        )
    if mkfs_green_exit_code != 0 or len(mkfs_green_lines) != 1:
        problems.append(
            f'--print-mkfs-segments-line 对未改动的拷贝本该退 0、打一行，实际退出码 {mkfs_green_exit_code}、{len(mkfs_green_lines)} 行：\n       '
            + '\n       '.join(mkfs_green_lines)
        )
    else:
        try:
            parsed_mkfs_fields = parse_product_segment_lines(mkfs_green_lines[0]).get(MKFS_PRODUCT_PATH_NAME)
        except CheckError as error:
            parsed_mkfs_fields = f'产物行的解析认不出它（{error}）'
        if parsed_mkfs_fields != expected_mkfs_fields:
            problems.append(
                f'--print-mkfs-segments-line 打的 `{mkfs_green_lines[0]}` 按产物行解析回来是 {parsed_mkfs_fields}，'
                f'登记表那一行是 {expected_mkfs_fields}——拼出来的行与登记的四样对不上'
            )
    if (mkfs_red_exit_code != 0 or len(mkfs_red_lines) != 1 or f' segments={mutated_mkfs_sequence_text} ' not in mkfs_red_lines[0]
            or mkfs_red_lines == mkfs_green_lines):
        problems.append(
            f'把「{MKFS_ROW_LABEL}」那一行的段序列从 `{original_mkfs_sequence_text}` 改成 `{mutated_mkfs_sequence_text}` 之后，'
            f'--print-mkfs-segments-line 打的不是带新段序列的那一行（退出码 {mkfs_red_exit_code}）：' + ' / '.join(mkfs_red_lines)
        )
    if mkfs_renamed_exit_code != 2:
        problems.append(
            f'把「{MKFS_ROW_LABEL}」那一行第一列改名之后，--print-mkfs-segments-line 本该退 2，实际退 {mkfs_renamed_exit_code}：' + ' / '.join(mkfs_renamed_lines)
        )
    if mkfs_duplicated_exit_code != 2:
        problems.append(
            f'把「{MKFS_ROW_LABEL}」那一行抄成两行之后，--print-mkfs-segments-line 本该退 2，实际退 {mkfs_duplicated_exit_code}：' + ' / '.join(mkfs_duplicated_lines)
        )
    expected_path_lines = [segments_line_of(name, entries_by_path[name]) for name in declared_paths if name in entries_by_path]
    try:
        parsed_back = parse_product_segment_lines('\n'.join(lines_green_lines)) if lines_green_exit_code == 0 else {}
    except CheckError:
        parsed_back = {}
    if (lines_green_exit_code != 0 or lines_green_lines != expected_path_lines or len(lines_green_lines) != len(declared_paths)
            or sorted(parsed_back) != sorted(declared_paths)):
        problems.append(
            f'--print-segments-lines 对未改动的拷贝本该退 0、按声明的 {len(declared_paths)} 条路径（{"、".join(declared_paths)}）各打一行，'
            f'实际退 {lines_green_exit_code}、{len(lines_green_lines)} 行：' + ' / '.join(lines_green_lines)
        )
    if lines_red_exit_code != 0 or not any(f'path={path_entry.product_path_name} ' in line and f' segments={mutated_path_sequence_text} ' in line
                                           for line in lines_red_lines):
        problems.append(
            f'把「{path_entry.row_label}」的段序列改成 `{mutated_path_sequence_text}` 之后，--print-segments-lines 打的 path={path_entry.product_path_name} 那一行本该跟着变，'
            f'实际退 {lines_red_exit_code}：' + ' / '.join(lines_red_lines)
        )

    # 同一行里同一串出现两次（段序列栏一处、出处栏一处，取号那一行就是这样）：自证造一份小布局表，段序列栏那一处换成新串、出处栏那一处不动
    duplicate_layout_text = (
        '## 八、根槽写路径的段序列登记表\n\n| 路径 | 段序列 | 出处 | 在不在 |\n|---|---|---|---|\n'
        f'| {SELFTEST_DUPLICATE_ROW_LABEL} | [两段] ⇒ `2+2`，6 次操作、7 个崩溃状态 | 两轮的 `2+2` 由自证钉住 | 在 |\n\n## 九、下一节\n')
    try:
        duplicate_mutated_text, _, _, duplicate_mutated_sequence = mutate_one_segment_number(
            duplicate_layout_text, SELFTEST_DUPLICATE_ROW_LABEL, whole_line=broken('segment-cell-unscoped'))
        duplicate_row = next(line for line in duplicate_mutated_text.split('\n') if line.startswith(f'| {SELFTEST_DUPLICATE_ROW_LABEL} |'))
        duplicate_cells = UNESCAPED_PIPE_PATTERN.split(duplicate_row)
        duplicate_ok = (duplicate_mutated_sequence == '2+3' and '`2+3`' in duplicate_cells[2] and '`2+2`' not in duplicate_cells[2]
                        and '`2+2`' in duplicate_cells[3])
        duplicate_note = f'改完那一行是 {duplicate_row}'
    except CheckError as error:
        duplicate_ok, duplicate_note = False, f'备料失败：{error}'
    if not duplicate_ok:
        problems.append(f'同一行里同一串出现两次（段序列栏一处、出处栏一处）时自证本该只换段序列栏那一处、出处栏那一处不动：{duplicate_note}')

    if problems:
        for problem in problems:
            print(f'  ✗ {problem}')  # gate-lint:detail
        print(f'  ✗ --selftest 没通过：{len(problems)} 项（上面逐条列着）')
        print('     → 怎么办：看清楚上面改坏的是哪一行、期望判红的地方是不是真的判了红，')
        print('       再回头检查 perform_check 里对应那一支（与钉住的用例比、与产物比、装置钉住）的比较逻辑；CHECK_SEGMENT_REGISTRY_BREAK 设着的话这里本来就该红。')
        return 1

    print(
        '  ✓ --selftest 通过（20 格，输入是真仓的布局表八、它点名的 crates 用例与脚本旁边的产物样本，不读 research/）：'
        '未改动的拷贝不摆产物判绿（退出码 0）；'
        f'把「{mutated_row_label}」的段序列改成 `{mutated_segment_sequence_text}` 之后判红（退出码 {red_exit_code}）；'
        f'只把「{kind_mutated_row_label}」第一段的种类 `{original_kind_text}` 改成 `{mutated_kind_text}` 也判红（退出码 {kind_red_exit_code}）；'
        f'把覆盖写固定脚本流的段序列改成 `{mutated_pinned_text}` 也判红（退出码 {pinned_red_exit_code}）；'
        f'覆盖写固定脚本流的登记句改成认不出的形状（「{mutated_shape_span}」）判红（退出码 {shape_red_exit_code}）；'
        f'用例拷贝里状态数只剩带下划线的写法（{"；".join(underscore_descriptions)}）判绿（退出码 {underscore_green_exit_code}）；'
        f'八节开头的 path= 声明多写一条 `{EXTRA_DECLARED_PATH_NAME}` 退 {extra_path_exit_code}、消息点出实际剩 {expected_extra_path_remaining_count} 条；'
        f'「{path_entry.row_label}」的段序列改成 `{mutated_path_sequence_text}`、「{original_operations_span}」改成「{mutated_operations_span}」，与钉它的用例 {pinning_function_name} 比各退 1；'
        f'它只能与产物比时，产物不在退 {product_absent_exit_code}（本次未跑）、照登记现造的产物判绿、产物段序列改成 `{product_red_sequence_text}` 退 {product_red_exit_code}；'
        f'样本 {PRODUCT_SAMPLE_PATH.name} 解析出 {len(sample_segment_lines)} 条路径；'
        f'--print-mkfs-segments-line 打出「{MKFS_ROW_LABEL}」那一行（`{mkfs_green_lines[0]}`，按产物行解析回来四样与登记相符），'
        f'段序列改成 `{mutated_mkfs_sequence_text}` 跟着变，那一行改名、抄成两行都退 2；'
        f'--print-segments-lines 打出 {len(lines_green_lines)} 行（{"、".join(declared_paths)}），段序列改了跟着变；'
        '同一行里段序列栏与出处栏写着同一串时，自证只换段序列栏那一处'
    )
    return 0


def main():
    parser = argparse.ArgumentParser(
        description='比对 layout/01-first-txn.md 「八、根槽写路径的段序列登记表」与钉住活代码的用例（钉不到用例的行与 E142 产物的 name=segments 行）'
    )
    parser.add_argument(
        '--root', type=pathlib.Path, default=None,
        help='仓库根目录；默认按这个脚本自己的位置推算（scripts/ 的上两级）',
    )
    parser.add_argument(
        '--selftest', action='store_true',
        help='自证：改坏拷贝里的段序列、种类、次操作、覆盖写固定脚本流的段序列，或在 path= 声明里多写一条路径，确认检查会转红；'
             '只能与产物比的那一支产物不在退 77；未改动的拷贝确认判绿。忽略 --root。',
    )
    printing = parser.add_mutually_exclusive_group()
    printing.add_argument(
        '--print-mkfs-segments-line', action='store_true',
        help='不比对：把八节登记表「mkfs 种根」那一行拼成 E7RESULT name=segments path=mkfs … 打一行（门禁 checker-tier-qemu-device-streams 用）',
    )
    printing.add_argument(
        '--print-segments-lines', action='store_true',
        help='不比对：把八节开头声明的每一条路径的登记拼成 E7RESULT name=segments path=<名> … 按声明次序各打一行（门禁 checker-tier-qemu-device-streams 用）',
    )
    arguments = parser.parse_args()

    if arguments.selftest:
        sys.exit(run_self_test())

    repository_root = arguments.root.resolve() if arguments.root else default_repository_root()
    if arguments.print_mkfs_segments_line:
        exit_code, message_lines = perform_print_mkfs_segments_line(repository_root)
    elif arguments.print_segments_lines:
        exit_code, message_lines = perform_print_segments_lines(repository_root)
    else:
        exit_code, message_lines = perform_check(repository_root)
    for line in message_lines:
        print(line)
    sys.exit(exit_code)


if __name__ == '__main__':
    preflight(__file__)
    main()
