#!/usr/bin/env python3
# admission: always 判的是此刻的合表、按段登记文件、stage-inputs.tsv 与 crash-amplification.sh 的 ITEMS，几毫秒读完；每次改其中一份都现判
# run-condition: none 只读仓里的三份文本，除了 python3 之外没有环境要求
"""崩溃点重放合表的读法与核法（一份判法，入口 scripts/crash/crash-point-replay.sh 与 scripts/crash/crash-case-check.py 的 placement 共用）。

合表 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头）：一项是一条流，或一条流里按功能切的一步（一条测试函数），点名它的两部分——
  崩溃点重放那一套（.claude/gate.d/stage-inputs.tsv 的 crash-case: 行）与放量那一套（scripts/crash/crash-amplification.sh 的 ITEMS）。
  这一项按条复用的测试目标不另登记：就是这两部分的测试目标（list 的第四段现推）；几项点名同一个测试目标里的部分时（一条流按步拆成几项，
  或几条流住一份测试文件），这个目标归表里第一个点名它的那一项，后面几项只跑自己点名的部分（item_test_targets；一个测试函数只归一项）。

用法：
    lib_crash_point_replay_items.py list [仓根]      逐行打「项名<TAB>crash-case 键（空格隔开，没有是 -）<TAB>放量分项（同上）<TAB>测试目标（合表不登记，
                                                     由前两段点名的部分现推：crash-case 行的 test= 与放量分项在 ITEMS 里的测试目标，去重、照先出现的次序）」；
                                                     合表核不过时不打，打问题、退 1
    lib_crash_point_replay_items.py check [仓根]     只核，核过打一句成功句（核了几项、几条 crash-case、几个放量分项）
    lib_crash_point_replay_items.py segments [仓根]  只核按段登记文件（见下「按段登记」），核过打一句成功句（几行、几项、几条流）；门禁 code-source-discipline 的
                                                     crash-point-replay-segments 格与两个入口 scripts/crash/crash-point-replay.sh、scripts/crash/crash-amplification.sh
                                                     开跑前都调它
    lib_crash_point_replay_items.py scoped-domain <guard|report> --root <仓根> --runs <运行记录目录> [--expand-up-to <n>] [--gpu <0|1>]
                                                     [--defect <名>] (--item <合表项> | --flow <分项>:<测试目标>)…
                                                     指定范围的崩溃域上限，写法见下「指定范围的崩溃域上限」；两个入口开跑前（guard）与
                                                     crash-point-replay.sh --list-items --with-keys（report）调它
    lib_crash_point_replay_items.py case-states <日志>   打门禁放的一条崩溃枚举用例日志里 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行 states= 之和，读不出退 1；
                                                     crash-point-replay.sh 每跑完一趟崩溃点重放部分用它取态数写进 .scope
    lib_crash_point_replay_items.py reader-features <验证包的 Cargo.toml> <真跑带的特性，逗号隔开，可空>
                                                     打读库程序要带的特性串（真跑带的特性加上包清单 [dev-dependencies] 里每个带特性的依赖的 <依赖>/<特性>），
                                                     清单读不了退 1；crash-point-replay.sh 的 --list-items --with-keys 与上面的 scoped-domain 同用这一份
  被 import 时用 read_items(root)、crash_case_rows(root)、pipeline_items(root)、item_test_targets(root)、problems(root)、crash_cases_named(root)、
  registered_functions(root)（合表点名的部分落到哪几个「测试目标、用例函数」，crash-case-check.py 的 placement 与 scripts/gate/admission.py 的 test-plan --exclude-registered 用）、segment_problems(root)；
  核法的判别力由 selftest_cells() 拿造出来的小仓核，它由 scripts/crash/crash-case-check.py --selftest 起（placement 读的就是这一份核法）。

按段登记（与引擎那一路实现的接口约定）：
  文件 .claude/gate.d/crash-point-replay-segments.tsv，一行 <项名>\t<流名>\t<段号>\t<段写数>\t<段写表摘要>（可再加一段 # 说明），# 开头的行是注释。
  入口开跑时往引擎送 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<这份文件的绝对路径>，引擎按第 2 列（流名）取这条流的行，登记了的段按这一趟的展开上限展开（写数不超过上限的展开全部真子集，超过的取甲二）。
  核法（segment_problems）：文件读得了；每行五段；项名是合表里的一项；流名匹配 ^[A-Za-z][A-Za-z0-9_]*$；段号与段写数是非负整数（十进制、不带前导 0）、
  段写数大于 0；摘要是 64 位小写十六进制（sha256）；同一（流名，段号）只一行。合表读不了照合表的问题报。

指定范围的崩溃域上限（.claude/rules/verification.md「一个门禁：两种跑法」：指定范围的那几趟崩溃状态数压在 20 万量级，不指定范围那一趟不管）：
  常量 SCOPED_CRASH_STATES_LIMIT = 200000（态）与退出码 SCOPED_DOMAIN_REFUSED_EXIT = 79 只在这里登记一处，crash-point-replay.sh 与 crash-amplification.sh 都取这一份。
  什么时候判：入口带了 --item、又没缩域（--crash-expand-up-to 与环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 都没设，或设的值不小于引擎展开上限常量，
  那等于没缩）。--expand-up-to 给了且小于引擎常量（crates/singlefs-verification/src/crash_amplification.rs 的
  EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION；读不到常量时给了就当缩过）guard 直接放行。不带 --item 的入口不调。
  一个点名项要现判多少态（estimate_subject；只数键对不上、要现判的，能复用的不算）：
    ① 没有崩溃点重放部分也没有放量部分（整趟才跑的那几类 whole-run:<类>）：0 态，来源 no-crash-domain。
    ② 读库程序 crash_library_reader 的 item 子命令（crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」）报这一项在库里录下 a 块、键对不上 b 块、复用 c 块：
       a 大于 0 且 b 为 0，现判 0 态，来源 reusable。
    ③ 否则取这一项各部分上一趟「不缩域」真跑的全域态数之和 P：
       放量分项取共享根（多机配置 NFS_SHARED_ROOTS 点名、SHARED_DATA_ROOT 所在）下 <runs>/entry-*/<分项>.log 里 CRASH_AMPLIFICATION 行（scripts/crash/crash-amplification.sh 每趟每项另存的那份），
       每行态数取 states=，没有取 states_checked_here + states_reused，各行相加；有行被叫停（stopped_early=true）、留了没判的块、展开深度各行不一、取不出的，这份日志不算；
       崩溃点重放部分取 <runs>/crash-point-replay-*/enumeration.<项>[.<用例>].log.scope（scripts/crash/crash-point-replay.sh 的 run_enumeration_part 每跑完一趟写：
       expand_up_to=full|<n>、case_states=<用例名>:<态数> …（每条用例一对，态数是门禁放的那条用例日志 log.<用例名> 里 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行 states= 之和，
       入口在跑那一台上用本库的 case-states 子命令取出来写进去，日志在不在别的机器上不要紧），判器展开深度也缩过的另有 judge_expand_up_to= 行，那一趟不算），
       这一项点名的每条用例都要有态数，缺一条这一趟记录不算。
       只认展开深度等于引擎常量的记录（缩过域的那一趟态数少，拿来估全域会估小）；同一展开深度取最新的一份（文件修改时刻）。每个部分都要有，缺一个 P 就算不出。
       P 算得出：a 大于 0 且 b 小于 a 时按块数比例估 ceil(P × b / a)，来源 prior-run-scaled；其余（库里没录这一项的块、读库程序没读到、b 等于 a）取 P，来源 prior-run。
       P 算不出：现判态数写 unknown，来源 unknown。
  guard 的判：先看上一趟各项 P 之和是否已不超上限（全部现判也不超），不超就放行、不读库；超了或有项算不出，才起读库程序（cargo run，环境与编译目录取调用方的）逐项读，
  再按上面估。全部点名项的估数之和超过 SCOPED_CRASH_STATES_LIMIT，或有项 unknown：打拒绝说明（标准错误）、退 SCOPED_DOMAIN_REFUSED_EXIT；
  说明给两条出路：① 带 --crash-expand-up-to <n> 缩域再交（各项在同一展开深度下都有记录、合计不超上限的最大那个 n 算得出就写出来，算不出提示先给小 n 看这一趟报的态数）；
  缩过域的那一趟结果行写「缩过域、不算门禁通过」；② 不缩域就留给提交时不指定范围的那一趟。读库程序没读到（编不过、库连不上）只记一句原因，估数改走 P，不单独判红。
  report 模式不判上限：每个点名项打一行 ITEM_DOMAIN item=<项> states_to_judge=<态数或 unknown> source=<no-crash-domain|reusable|prior-run-scaled|prior-run|unknown>（给 --list-items --with-keys 读）。
  --defect <名> 是入口的弄坏开关借道，平常不给，只有两个入口的 --selftest 起的子进程给：count-reusable（库里全部可复用的项也按 P 算，P 算得出才算）、
  unknown-as-zero（算不出的项按 0 态放行）、suggestion-dropped（拒绝说明不写算得出的 n）、shrunk-records-used（取最新的一份记录，不看展开深度）；入口里这几个开关的名字与各自该红的格写在两个入口的文件头。

核法（problems）：
  ① 每行三段（再加 # 说明）；项名是小写字母、数字、-，不重复，不含 layer0、amplification（新名字不用这两个词）；
  ② 第二段的每个键是 stage-inputs.tsv 里一行 crash-case: 的键，第三段的每一项是 ITEMS 里的分项名；两段不能都是 -；
  ③ stage-inputs.tsv 的每一条 crash-case: 与 ITEMS 的每一项都恰好被一项点名（漏了的、两项都点名的都红）。
  测试目标的归属（item_test_targets）不另判红：一个目标归表里第一个点名它里面部分的那一项，所以一条流按步拆项时，流那一项（带放量分项与流里别的用例）排在它的几步前面。
  合表或 stage-inputs.tsv 读不了算一处问题，不抛；放量入口 crash-amplification.sh 不在当成 ITEMS 是空的
  （合表点名了放量分项的那几项照样报「不是 ITEMS 里的分项」；崩溃点重放门禁的样本根里只有合表、没有放量入口）。
退出码：0 核过（list 打全表）；1 有问题（逐处列）；2 用法错。

弄坏开关（只给自证证红用）。逐项写开关名、弄坏哪一处、打开后 scripts/crash/crash-case-check.py --selftest 哪几格红
（格名是 selftest_cells() 里的格名，那份自证打「✗ 自检：合表核法 <格名>：…」）：
  CRASH_POINT_REPLAY_ITEMS_BREAK=unknown-part-accepted  ② 不核点名的键与分项在不在
      unknown-crash-case、unknown-pipeline-item
  CRASH_POINT_REPLAY_ITEMS_BREAK=unreferenced-accepted  ③ 不核每条 crash-case 与每个分项都被点名
      unreferenced-crash-case、unreferenced-pipeline-item
  CRASH_POINT_REPLAY_ITEMS_BREAK=duplicate-reference-accepted  ③ 不核同一条被两项点名
      crash-case-named-twice
  CRASH_POINT_REPLAY_ITEMS_BREAK=retired-words-accepted  ① 不核项名里的退役词
      retired-word-in-name
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-unknown-item-accepted  按段登记不核项名在不在合表里
      segments-unknown-item
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-numbers-unchecked  按段登记不核段号与段写数（非负整数、段写数大于 0）
      segments-bad-segment-number、segments-zero-writes
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-digest-unchecked  按段登记不核摘要是 64 位小写十六进制
      segments-bad-digest
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-duplicate-accepted  按段登记不核同一（流名，段号）只一行
      segments-duplicate
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-flow-name-unchecked  按段登记不核流名的写法
      segments-bad-flow-name
  CRASH_POINT_REPLAY_ITEMS_BREAK=segment-column-count-unchecked  按段登记不核每行五段（段数不对的行不报、跳过）
      segments-field-count
  CRASH_POINT_REPLAY_ITEMS_BREAK=shared-target-given-to-every-item  测试目标给点名它里面部分的每一项（一条流按步拆成几项，目标里别的测试函数跟着每一步各跑一遍）
      shared-target-first-item
"""
import argparse
import collections
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402

BROKEN = os.environ.get("CRASH_POINT_REPLAY_ITEMS_BREAK", "")
ITEMS_TABLE = os.path.join(".claude", "gate.d", "crash-point-replay-items.tsv")
REGISTRATION_TABLE = os.path.join(".claude", "gate.d", "stage-inputs.tsv")
PIPELINE_ENTRY = os.path.join("scripts", "crash", "crash-amplification.sh")
ITEM_NAME = re.compile(r"^[a-z0-9][a-z0-9-]*$")
RETIRED_WORDS = ("layer0", "amplification")
CRASH_CASE_ROW = re.compile(r"^(crash-case:[a-z0-9][a-z0-9-]*)\t[^\t]*\t(?:.*\s)?test=([\w-]+):(\w+):(\w+)")
PIPELINE_ITEM = re.compile(r"^\s*\$'([a-z0-9][a-z0-9-]*)\\t(\w+)\\t(\w+)'\s*$")
SEGMENTS_TABLE = os.path.join(".claude", "gate.d", "crash-point-replay-segments.tsv")
NONNEGATIVE_INTEGER = re.compile(r"^(?:0|[1-9][0-9]*)$")
SEGMENT_DIGEST = re.compile(r"^[0-9a-f]{64}$")
SEGMENT_FLOW_NAME = re.compile(r"^[A-Za-z][A-Za-z0-9_]*$")


class Item:
    def __init__(self, line_number, name, crash_cases, pipeline_items):
        self.line_number = line_number
        self.name = name
        self.crash_cases = crash_cases
        self.pipeline_items = pipeline_items


def words_of(field):
    return [] if field.strip() in ("", "-") else field.split()


def read_items(root):
    """→ (项的列表, 问题)。合表读不了是一处问题。"""
    path = os.path.join(root, ITEMS_TABLE)
    items, problems = [], []
    try:
        lines = open(path, encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了合表 {ITEMS_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        body = line.split("\t#", 1)[0]
        fields = body.split("\t")
        if len(fields) != 3:
            problems.append(f"{ITEMS_TABLE}:{number} 是 {len(fields)} 段，要三段（项名、crash-case 键、放量分项，制表符分隔）")
            continue
        items.append(Item(number, fields[0].strip(), words_of(fields[1]), words_of(fields[2])))
    return items, problems


def crash_case_rows(root):
    """stage-inputs.tsv 里 crash-case: 的行：{键: (测试目标, 用例函数)}；读不了抛 OSError。"""
    rows = {}
    for line in open(os.path.join(root, REGISTRATION_TABLE), encoding="utf-8"):
        match = CRASH_CASE_ROW.match(line)
        if match:
            rows[match.group(1)] = (match.group(3), match.group(4))
    return rows


def pipeline_items(root):
    """crash-amplification.sh 的 ITEMS：{分项名: (测试目标, 用例函数)}；文件不在交空表（文件头「核法」末句），在而读不了抛 OSError。"""
    found, inside = {}, False
    path = os.path.join(root, PIPELINE_ENTRY)
    if not os.path.exists(path):
        return found
    for line in open(path, encoding="utf-8"):
        if line.startswith("ITEMS=("):
            inside = True
            continue
        if inside and line.startswith(")"):
            break
        if inside:
            match = PIPELINE_ITEM.match(line)
            if match:
                found[match.group(1)] = (match.group(2), match.group(3))
    return found


def problems(root):
    """合表的问题清单（文件头「核法」）。"""
    items, found = read_items(root)
    try:
        cases = crash_case_rows(root)
    except OSError as error:
        return found + [f"读不了 {REGISTRATION_TABLE}：{error}"]
    try:
        pipeline_rows = pipeline_items(root)
    except OSError as error:
        return found + [f"读不了 {PIPELINE_ENTRY}：{error}"]
    seen_names = set()
    named_cases, named_pipeline = {}, {}
    for item in items:
        where = f"{ITEMS_TABLE}:{item.line_number} {item.name}"
        if not ITEM_NAME.match(item.name):
            found.append(f"{where}：项名只许小写字母、数字与 -")
        if item.name in seen_names:
            found.append(f"{where}：项名重复")
        seen_names.add(item.name)
        if BROKEN != "retired-words-accepted":
            for word in RETIRED_WORDS:
                if word in item.name:
                    found.append(f"{where}：项名里有退役的词 {word}（新名字用「崩溃点重放」那一族，判决第三节 J1）")
        if not item.crash_cases and not item.pipeline_items:
            found.append(f"{where}：第二、三段都是 -，这一项没有可跑的部分")
        for key in item.crash_cases:
            named_cases.setdefault(key, []).append(item.name)
            if key not in cases and BROKEN != "unknown-part-accepted":
                found.append(f"{where}：第二段的 {key} 不是 {REGISTRATION_TABLE} 里 crash-case: 的键")
        for name in item.pipeline_items:
            named_pipeline.setdefault(name, []).append(item.name)
            if name not in pipeline_rows and BROKEN != "unknown-part-accepted":
                found.append(f"{where}：第三段的 {name} 不是 {PIPELINE_ENTRY} 的 ITEMS 里的分项")
    if BROKEN != "unreferenced-accepted":
        for key in sorted(set(cases) - set(named_cases)):
            found.append(f"{REGISTRATION_TABLE} 的 {key} 没有被合表里哪一项点名（放进它那条流的那一项的第二段，或为它另起一项：一条流，或一条流里的一步）")
        for name in sorted(set(pipeline_rows) - set(named_pipeline)):
            found.append(f"{PIPELINE_ENTRY} 的 ITEMS 里的 {name} 没有被合表里哪一项点名（放进它那条流的那一项的第三段，或另起一项）")
    if BROKEN != "duplicate-reference-accepted":
        for part, owners in sorted(list(named_cases.items()) + list(named_pipeline.items())):
            if len(owners) > 1:
                found.append(f"{part} 被合表里 {len(owners)} 项点名（{'、'.join(owners)}）：一条只许归一项")
    return found


def item_test_targets(root):
    """合表每一项按条复用的测试目标：{项名: [测试目标]}。取自这一项点名的部分（crash-case 行的 test= 与放量分项在 ITEMS 里的测试目标），
    去重、照先出现的次序（先 crash-case 后放量分项）；一个目标只归表里第一个点名它里面部分的那一项，后面的项不再列它（文件头「合表」）。
    合表读不了是空表，stage-inputs.tsv 读不了抛 OSError。弄坏开关 shared-target-given-to-every-item 下每一项都列。"""
    items, _ = read_items(root)
    cases = crash_case_rows(root)
    pipeline_rows = pipeline_items(root)
    found, claimed = {}, set()
    for item in items:
        targets = [cases[key][0] for key in item.crash_cases if key in cases] + [pipeline_rows[name][0] for name in item.pipeline_items if name in pipeline_rows]
        own = [target for target in dict.fromkeys(targets) if target not in claimed or BROKEN == "shared-target-given-to-every-item"]
        claimed.update(own)
        found[item.name] = own
    return found


def crash_cases_named(root):
    """合表点名了的 crash-case: 键的集合（placement 用）；合表读不了是空集。"""
    items, _ = read_items(root)
    return {key for item in items for key in item.crash_cases}


def registered_functions(root):
    """合表点名的部分落到的 {(测试目标, 用例函数)}：点名的 crash-case: 行的 test=，点名的放量分项在 ITEMS 里的测试目标与用例函数。
    读不了 stage-inputs.tsv 抛 OSError；合表读不了是空集（placement 另经 problems() 报那一处）。"""
    items, _ = read_items(root)
    cases = crash_case_rows(root)
    pipeline_rows = pipeline_items(root)
    found = set()
    for item in items:
        found.update(cases[key] for key in item.crash_cases if key in cases)
        found.update(pipeline_rows[name] for name in item.pipeline_items if name in pipeline_rows)
    return found


def read_segments(root):
    """按段登记文件的行：→ ([(行号, 项名, 流名, 段号, 段写数, 摘要)], 问题)。文件读不了、段数不对各是一处问题。"""
    rows, found = [], []
    try:
        lines = open(os.path.join(root, SEGMENTS_TABLE), encoding="utf-8").read().splitlines()
    except OSError as error:
        return [], [f"读不了按段登记文件 {SEGMENTS_TABLE}：{error}"]
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.startswith("#"):
            continue
        fields = line.split("\t#", 1)[0].split("\t")
        if len(fields) != 5:
            if BROKEN != "segment-column-count-unchecked":
                found.append(f"{SEGMENTS_TABLE}:{number} 是 {len(fields)} 段，要五段（项名、流名、段号、段写数、段写表摘要，制表符分隔）")
            continue
        rows.append((number, *(field.strip() for field in fields)))
    return rows, found


def segment_problems(root):
    """按段登记文件的问题清单（文件头「按段登记」）。"""
    rows, found = read_segments(root)
    items, item_problems = read_items(root)
    if item_problems:
        found += [f"核按段登记要读合表：{problem}" for problem in item_problems]
    names = {item.name for item in items}
    seen = {}
    for number, name, flow, segment, writes, digest in rows:
        where = f"{SEGMENTS_TABLE}:{number}"
        if name not in names and BROKEN != "segment-unknown-item-accepted":
            found.append(f"{where}：项名 {name} 不是合表 {ITEMS_TABLE} 里的一项")
        if not SEGMENT_FLOW_NAME.match(flow) and BROKEN != "segment-flow-name-unchecked":
            found.append(f"{where}：流名「{flow}」不合写法（字母起头，只许字母、数字与 _；照引擎驱动行的 flow= 写）")
        if BROKEN != "segment-numbers-unchecked":
            if not NONNEGATIVE_INTEGER.match(segment):
                found.append(f"{where}：段号「{segment}」不是非负整数（十进制、不带前导 0）")
            if not NONNEGATIVE_INTEGER.match(writes) or writes == "0":
                found.append(f"{where}：段写数「{writes}」要是大于 0 的整数（十进制、不带前导 0）")
        if not SEGMENT_DIGEST.match(digest) and BROKEN != "segment-digest-unchecked":
            found.append(f"{where}：段写表摘要「{digest}」不是 64 位小写十六进制（sha256）")
        if (flow, segment) in seen and BROKEN != "segment-duplicate-accepted":
            found.append(f"{where}：流 {flow} 第 {segment} 段已在第 {seen[(flow, segment)]} 行登记过，同一条流同一段只一行")
        seen.setdefault((flow, segment), number)
    return found


def command_segments(root):
    found = segment_problems(root)
    if found:
        for problem in found:
            print(f"  ✗ {problem}")  # gate-lint:detail
        print(f"  ✗ 按段登记文件 {SEGMENTS_TABLE} 有 {len(found)} 处不对（逐处列在上面）")  # gate-lint:summary
        print(f"  → 怎么办：照那几处改 {SEGMENTS_TABLE}（写法在它的表头与 scripts/crash/lib_crash_point_replay_items.py 文件头「按段登记」）：每行五段，项名照合表写，"
              "流名照引擎驱动行的 flow= 写，段号与段写数写十进制整数、段写数大于 0，摘要写段写表的 sha256（64 位小写十六进制），同一条流同一段只留一行")
        return 1
    rows, _ = read_segments(root)
    print(f"  ✓ 崩溃点重放按段登记核过：{len(rows)} 行（{len({row[1] for row in rows})} 项、{len({row[2] for row in rows})} 条流），每行五段、项名都在合表里、"
          "流名合写法、段号与段写数是非负整数且段写数大于 0、摘要是 64 位小写十六进制、同一条流同一段只一行")
    return 0


def report_problems(found):
    for problem in found:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 崩溃点重放合表 {ITEMS_TABLE} 有 {len(found)} 处不对（逐处列在上面）")  # gate-lint:summary
    print(f"  → 怎么办：照那几处改合表（写法在 {ITEMS_TABLE} 表头）；新加的 crash-case: 行或放量分项要放进它那条流的那一项")


def command_list(root):
    found = problems(root)
    if found:
        report_problems(found)
        return 1
    items, _ = read_items(root)
    targets = item_test_targets(root)
    for item in items:
        print("\t".join([item.name, " ".join(item.crash_cases) or "-", " ".join(item.pipeline_items) or "-", " ".join(targets[item.name])]))
    return 0


def command_check(root):
    found = problems(root)
    if found:
        report_problems(found)
        return 1
    items, _ = read_items(root)
    case_count = sum(len(item.crash_cases) for item in items)
    pipeline_count = sum(len(item.pipeline_items) for item in items)
    print(f"  ✓ 崩溃点重放合表核过：{len(items)} 项，点名 {case_count} 条 crash-case 与 {pipeline_count} 个放量分项（各恰好一次）")
    return 0


SELFTEST_TABLE_HEADER = "# 样本合表\n"


def selftest_cells():
    """拿造出来的小仓核 problems() 的每一条核法：→ (格名的列表, [(格名, 判错的一句)])。CRASH_POINT_REPLAY_ITEMS_BREAK 设着时点名的格判错。"""
    work = tempfile.mkdtemp(prefix="crash-point-replay-items-selftest-")
    failures, cells = [], []

    def world(name, table_rows, crash_cases, pipeline_rows):
        root = os.path.join(work, name)
        os.makedirs(os.path.join(root, ".claude", "gate.d"))
        os.makedirs(os.path.dirname(os.path.join(root, PIPELINE_ENTRY)))
        with open(os.path.join(root, ITEMS_TABLE), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_TABLE_HEADER + "".join("\t".join(row) + "\t# 样本\n" for row in table_rows))
        with open(os.path.join(root, REGISTRATION_TABLE), "w", encoding="utf-8") as handle:
            for key, target in crash_cases:
                handle.write(f"{key}\tcrates/\ttest=singlefs-verification:{target}:the_case count-line=X\t# 样本\n")
        with open(os.path.join(root, PIPELINE_ENTRY), "w", encoding="utf-8") as handle:
            handle.write("ITEMS=(\n" + "".join(f"  $'{name}\\t{target}\\tthe_function'\n" for name, target in pipeline_rows) + ")\n")
        return root

    def expect(cell, root, wanted):
        """wanted 是 None：要 0 处问题；是串：要有一处问题含它。"""
        cells.append(cell)
        found = problems(root)
        if wanted is None and found:
            failures.append((cell, f"应当核过，实际 {found}"))
        elif wanted is not None and not any(wanted in problem for problem in found):
            failures.append((cell, f"应当报一处含「{wanted}」的问题，实际 {found}"))

    cases = [("crash-case:stream-one", "stream_one"), ("crash-case:stream-two", "stream_two")]
    pipeline_rows = [("stream-two-piped", "stream_two_piped")]
    good = [["stream-one", "crash-case:stream-one", "-"],
            ["stream-two", "crash-case:stream-two", "stream-two-piped"]]
    expect("clean", world("clean", good, cases, pipeline_rows), None)
    expect("unknown-crash-case", world("unknown-crash-case", good + [["ghost", "crash-case:ghost", "-"]], cases, pipeline_rows),
           "不是 .claude/gate.d/stage-inputs.tsv 里 crash-case: 的键")
    expect("unknown-pipeline-item", world("unknown-pipeline-item", good + [["ghost", "-", "ghost-piped"]], cases, pipeline_rows),
           "不是 scripts/crash/crash-amplification.sh 的 ITEMS 里的分项")
    expect("unreferenced-crash-case", world("unreferenced-crash-case", good[1:], cases, pipeline_rows), "crash-case:stream-one 没有被合表里哪一项点名")
    expect("unreferenced-pipeline-item",
           world("unreferenced-pipeline-item", [good[0], ["stream-two", "crash-case:stream-two", "-"]], cases, pipeline_rows),
           "stream-two-piped 没有被合表里哪一项点名")
    expect("crash-case-named-twice", world("crash-case-named-twice", good + [["stream-one-again", "crash-case:stream-one", "-"]], cases, pipeline_rows),
           "crash-case:stream-one 被合表里 2 项点名")
    expect("retired-word-in-name", world("retired-word-in-name", [["layer0-stream-one", "crash-case:stream-one", "-"], good[1]], cases, pipeline_rows),
           "项名里有退役的词 layer0")
    expect("both-parts-empty", world("both-parts-empty", good + [["nothing", "-", "-"]], cases, pipeline_rows), "第二、三段都是 -")
    entry_absent = world("pipeline-entry-absent", [good[0], ["stream-two", "crash-case:stream-two", "-"]], cases, [])
    os.remove(os.path.join(entry_absent, PIPELINE_ENTRY))
    expect("pipeline-entry-absent", entry_absent, None)
    named_root = world("registered-functions", good, cases, pipeline_rows)
    cells.append("registered-functions")
    wanted_functions = {("stream_one", "the_case"), ("stream_two", "the_case"), ("stream_two_piped", "the_function")}
    if registered_functions(named_root) != wanted_functions:
        failures.append(("registered-functions", f"合表点名的部分应当落到 {sorted(wanted_functions)}，实际 {sorted(registered_functions(named_root))}"))
    cells.append("item-test-targets")
    wanted_targets = {"stream-one": ["stream_one"], "stream-two": ["stream_two", "stream_two_piped"]}
    if item_test_targets(named_root) != wanted_targets:
        failures.append(("item-test-targets", f"每一项的测试目标应当由点名的部分推出 {wanted_targets}，实际 {item_test_targets(named_root)}"))
    # 一条流按步拆项：流那一项（复用机制那一条与放量分项）在前，两步各一项在后，三项点名的用例都在 stream_split 这一个目标里；目标只归流那一项
    split_cases = cases + [("crash-case:stream-split-rerun", "stream_split"), ("crash-case:stream-split-first-step", "stream_split"),
                           ("crash-case:stream-split-second-step", "stream_split")]
    split_rows = good + [["stream-split", "crash-case:stream-split-rerun", "stream-split-piped"],
                         ["stream-split-first-step", "crash-case:stream-split-first-step", "-"],
                         ["stream-split-second-step", "crash-case:stream-split-second-step", "-"]]
    split_root = world("shared-target-first-item", split_rows, split_cases, pipeline_rows + [("stream-split-piped", "stream_split")])
    cells.append("shared-target-first-item")
    split_found = problems(split_root)
    split_targets = item_test_targets(split_root)
    wanted_split = {"stream-one": ["stream_one"], "stream-two": ["stream_two", "stream_two_piped"], "stream-split": ["stream_split"],
                    "stream-split-first-step": [], "stream-split-second-step": []}
    if split_found or split_targets != wanted_split:
        failures.append(("shared-target-first-item", f"一条流按步拆成三项（流那一项在前）应当核过、目标 stream_split 只归 stream-split，"
                                                     f"要 {wanted_split}，实际问题 {split_found}、目标 {split_targets}"))

    # 按段登记（文件头「按段登记」）：每格一份小仓（合表就是上面那两项），只换按段登记文件
    digest = "0123456789abcdef" * 4

    def segments_world(name, segment_rows):
        root = world(name, good, cases, pipeline_rows)
        if segment_rows is not None:
            with open(os.path.join(root, SEGMENTS_TABLE), "w", encoding="utf-8") as handle:
                handle.write("# 样本按段登记\n" + "".join("\t".join(row) + "\n" for row in segment_rows))
        return root

    def expect_segments(cell, root, wanted):
        cells.append(cell)
        found = segment_problems(root)
        if wanted is None and found:
            failures.append((cell, f"按段登记应当核过，实际 {found}"))
        elif wanted is not None and not any(wanted in problem for problem in found):
            failures.append((cell, f"按段登记应当报一处含「{wanted}」的问题，实际 {found}"))

    # 一项几条流各登各的段：stream-two 那一项两条流都登第 0 段，不算重
    clean_segments = [["stream-one", "stream_one_Flow", "0", "7", digest], ["stream-one", "stream_one_Flow", "2", "12", digest + "\t# 说明"],
                      ["stream-two", "two_A", "0", "3", digest], ["stream-two", "two_B", "0", "5", digest]]
    expect_segments("segments-clean", segments_world("segments-clean", clean_segments), None)
    expect_segments("segments-header-only", segments_world("segments-header-only", []), None)
    expect_segments("segments-unknown-item", segments_world("segments-unknown-item", clean_segments + [["ghost", "ghost_flow", "0", "3", digest]]),
                    "项名 ghost 不是合表")
    expect_segments("segments-bad-flow-name", segments_world("segments-bad-flow-name", [["stream-one", "stream-one", "0", "3", digest]]),
                    "流名「stream-one」不合写法")
    expect_segments("segments-bad-segment-number", segments_world("segments-bad-segment-number", [["stream-one", "stream_one_Flow", "-1", "3", digest]]),
                    "段号「-1」不是非负整数")
    expect_segments("segments-zero-writes", segments_world("segments-zero-writes", [["stream-one", "stream_one_Flow", "0", "0", digest]]),
                    "段写数「0」要是大于 0 的整数")
    expect_segments("segments-bad-digest", segments_world("segments-bad-digest", [["stream-one", "stream_one_Flow", "0", "3", digest.upper()]]),
                    "不是 64 位小写十六进制")
    expect_segments("segments-duplicate", segments_world("segments-duplicate", clean_segments + [["stream-one", "stream_one_Flow", "2", "9", digest]]),
                    "流 stream_one_Flow 第 2 段已在第 3 行登记过")
    expect_segments("segments-field-count", segments_world("segments-field-count", [["stream-one", "0", "3", digest]]), "是 4 段，要五段")
    expect_segments("segments-table-missing", segments_world("segments-table-missing", None), "读不了按段登记文件")
    shutil.rmtree(work, ignore_errors=True)
    return cells, failures


# ── 指定范围的崩溃域上限（文件头「指定范围的崩溃域上限」）

SCOPED_CRASH_STATES_LIMIT = 200000     # 指定范围（--item）、没缩域的那一趟，要现判的崩溃状态数上限（态）；两个入口共用这一处
SCOPED_DOMAIN_REFUSED_EXIT = 79        # 超上限或算不出而拒：不编、不跑
ENGINE_CONSTANT_FILE = os.path.join("crates", "singlefs-verification", "src", "crash_amplification.rs")
ENGINE_EXPAND_CONSTANT = re.compile(r"pub const EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION: usize = ([0-9]+);")
VERIFICATION_MANIFEST = os.path.join("crates", "singlefs-verification", "Cargo.toml")
VERIFICATION_TESTS = "crates/singlefs-verification/tests"
AMPLIFICATION_LINE_PREFIX = "CRASH_AMPLIFICATION "
ENUMERATION_FINISHED_PREFIX = "CRASH_POINT_REPLAY_PARALLEL_FINISHED "
LIBRARY_ITEM_TAG = "CRASH_LIBRARY_ITEM"
SCOPED_DOMAIN_DEFECTS = ("count-reusable", "unknown-as-zero", "suggestion-dropped", "shrunk-records-used")
COUNT = re.compile(r"^[0-9]+$")

LibraryItem = collections.namedtuple("LibraryItem", "recorded to_judge reusable")
Estimate = collections.namedtuple("Estimate", "label states source text")


class Subject:
    """一个点名项：合表的一项（崩溃点重放部分的用例键、放量分项）或放量入口自己的一个分项；设点文件是读库程序要的测试目标文件。"""

    def __init__(self, label, case_keys, flows, targets):
        self.label = label
        self.case_keys = list(case_keys)
        self.flows = list(flows)
        self.setting_files = [f"{VERIFICATION_TESTS}/{target}.rs" for target in dict.fromkeys(targets)]

    def has_crash_domain(self):
        return bool(self.case_keys or self.flows)


def merged_subject(root, name):
    """合表里的一项 → Subject；整趟才跑的那几类（whole-run:<类>）没有崩溃域；别的名字合表里没有返回 None。"""
    if name.startswith("whole-run:"):
        return Subject(name, [], [], [])
    items, _ = read_items(root)
    cases = crash_case_rows(root)
    pipeline_rows = pipeline_items(root)
    for item in items:
        if item.name == name:
            targets = [cases[key][0] for key in item.crash_cases if key in cases] + [pipeline_rows[flow][0] for flow in item.pipeline_items if flow in pipeline_rows]
            return Subject(name, item.crash_cases, item.pipeline_items, targets)
    return None


def engine_expand_constant(root):
    """引擎放量展开上限常量（全量域的展开深度）；读不到是 None。"""
    try:
        with open(os.path.join(root, ENGINE_CONSTANT_FILE), encoding="utf-8") as handle:
            found = ENGINE_EXPAND_CONSTANT.search(handle.read())
    except OSError:
        return None
    return int(found.group(1)) if found else None


def fields_of(line):
    """一行 <标签> <名>=<值> … 的字段（空白隔开，制表符也算）。"""
    return dict(token.split("=", 1) for token in line.split()[1:] if "=" in token)


def modified_at(path):
    try:
        return os.path.getmtime(path)
    except OSError:
        return 0.0


def read_lines(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read().splitlines()
    except OSError:
        return None


def amplification_log_total(path):
    """一份放量分项日志 → (展开深度, 全部 CRASH_AMPLIFICATION 行的态数之和)；读不了、一行都没有、有行取不出、被叫停或留了没判的块、各行展开深度不一是 None。
    一行的态数取 states=，没有取 states_checked_here + states_reused。"""
    lines = read_lines(path)
    if lines is None:
        return None
    lines = [line for line in lines if line.startswith(AMPLIFICATION_LINE_PREFIX)]
    depths, total = set(), 0
    for line in lines:
        fields = fields_of(line)
        depth, checked = fields.get("expand_up_to", ""), fields.get("states_checked_here", "")
        if not COUNT.match(depth) or not COUNT.match(checked):
            return None
        if fields.get("stopped_early") == "true" or fields.get("unjudged_blocks", "0") != "0":
            return None
        if COUNT.match(fields.get("states", "")):
            total += int(fields["states"])
        elif COUNT.match(fields.get("states_reused", "")):
            total += int(checked) + int(fields["states_reused"])
        else:
            return None
        depths.add(int(depth))
    return (depths.pop(), total) if lines and len(depths) == 1 else None


def amplification_totals(runs_directory, flow):
    """一个放量分项在共享根 <runs>/entry-*/<分项>.log 里留下的记录 → {展开深度: 态数合计}，同一展开深度取最新的一份（插入次序就是新到旧）。"""
    pattern = os.path.join(glob.escape(runs_directory), "entry-*", glob.escape(flow) + ".log")
    totals = {}
    for path in sorted(glob.glob(pattern), key=modified_at, reverse=True):
        parsed = amplification_log_total(path)
        if parsed:
            totals.setdefault(parsed[0], parsed[1])
    return totals


def read_scope(path):
    """crash-point-replay.sh 每跑完一趟崩溃点重放部分写的 .scope 文件 → {键: 值}；读不了是 None。"""
    lines = read_lines(path)
    if lines is None:
        return None
    return dict(line.split("=", 1) for line in lines if "=" in line)


def case_label(key):
    return key.split(":", 1)[1] if key.startswith("crash-case:") else key


def case_log_states(path):
    """门禁放的一条崩溃枚举用例日志 → 里面 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行的 states= 之和；读不了、一行都没有、有行取不出、只判了本机那一份（blocks_left_to_other_hosts 不是 0）是 None。"""
    lines = read_lines(path)
    if lines is None:
        return None
    states, seen = 0, False
    for line in lines:
        if not line.startswith(ENUMERATION_FINISHED_PREFIX):
            continue
        fields = fields_of(line)
        if fields.get("blocks_left_to_other_hosts", "0") != "0" or not COUNT.match(fields.get("states", "")):
            return None
        states += int(fields["states"])
        seen = True
    return states if seen else None


def enumeration_totals(runs_directory, item_name, case_keys, constant):
    """一个合表项的崩溃点重放部分在 <runs>/crash-point-replay-*/enumeration.<项>[.<用例>].log.scope 里留下的记录 → {展开深度: 态数合计}。
    一趟的 .scope 文件合起来要罩住这一项点名的每条用例（case_states= 里每条用例一对「用例名:态数」）；缩过判器展开深度的、展开深度写 full 而引擎常量读不到的，这一趟不算。"""
    wanted = [case_label(key) for key in case_keys]
    pattern_root = os.path.join(glob.escape(runs_directory), "crash-point-replay-*")
    scope_paths = glob.glob(os.path.join(pattern_root, f"enumeration.{glob.escape(item_name)}.log.scope")) \
        + glob.glob(os.path.join(pattern_root, f"enumeration.{glob.escape(item_name)}.*.log.scope"))
    by_run = {}
    for path in scope_paths:
        by_run.setdefault(os.path.dirname(path), []).append(path)
    totals = {}
    for run_directory in sorted(by_run, key=lambda directory: max(modified_at(path) for path in by_run[directory]), reverse=True):
        depths, states_of_case, usable = set(), {}, True
        for path in by_run[run_directory]:
            scope = read_scope(path)
            depth_text = (scope or {}).get("expand_up_to", "")
            depth = constant if depth_text == "full" else int(depth_text) if COUNT.match(depth_text) else None
            if scope is None or depth is None or "judge_expand_up_to" in scope:
                usable = False
                break
            depths.add(depth)
            for pair in scope.get("case_states", "").split():
                label, _, count = pair.rpartition(":")
                if label and COUNT.match(count):
                    states_of_case[label] = int(count)
        if usable and len(depths) == 1 and all(label in states_of_case for label in wanted):
            totals.setdefault(depths.pop(), sum(states_of_case[label] for label in wanted))
    return totals


def prior_parts(subject, runs_directory, constant):
    """→ [(部分名, {展开深度: 态数})]：崩溃点重放部分一个（有用例键才有），放量分项各一个。"""
    parts = []
    if subject.case_keys:
        parts.append(("崩溃点重放部分", enumeration_totals(runs_directory, subject.label, subject.case_keys, constant)))
    for flow in subject.flows:
        parts.append((f"放量分项 {flow}", amplification_totals(runs_directory, flow)))
    return parts


def full_prior(parts, constant, defect):
    """全域（展开深度等于引擎常量）的各部分态数之和 → (合计, [])；有部分没有全域记录 → (None, [那几个部分的名])。
    defect 是 shrunk-records-used 时不看展开深度，取每个部分最新的一份。"""
    values, missing = [], []
    for name, totals in parts:
        if defect == "shrunk-records-used" and totals:
            value = next(iter(totals.values()))
        else:
            value = totals.get(constant) if constant is not None else None
        if value is None:
            missing.append(name)
        else:
            values.append(value)
    return (None, missing) if missing else (sum(values), [])


def suggested_depth(all_parts, constant, limit):
    """各点名项的每个部分都有记录的展开深度里，合计不超上限的最大那个（小于引擎常量）→ (展开深度, 合计)；算不出是 None。"""
    depths = None
    for parts in all_parts:
        for _, totals in parts:
            depths = set(totals) if depths is None else depths & set(totals)
    best = None
    for depth in sorted(depths or ()):
        if constant is not None and depth >= constant:
            continue
        total = sum(totals[depth] for parts in all_parts for _, totals in parts)
        if total <= limit:
            best = (depth, total)
    return best


def estimate_subject(subject, library_item, library_note, runs_directory, constant, defect):
    """一个点名项这一趟要现判多少态（文件头「指定范围的崩溃域上限」②③）。library_item 是读库程序报的 LibraryItem 或 None（library_note 写没读到的原因）。"""
    parts = prior_parts(subject, runs_directory, constant)
    if not subject.has_crash_domain():
        return Estimate(subject.label, 0, "no-crash-domain", "没有崩溃点重放部分也没有放量部分，没有崩溃域")
    total, missing = full_prior(parts, constant, defect)
    if library_item and library_item.recorded > 0 and library_item.to_judge == 0:
        if defect == "count-reusable" and total is not None:
            return Estimate(subject.label, total, "prior-run", f"（弄坏开关：可复用的也按上一趟全域真跑的 {total} 态算）")
        return Estimate(subject.label, 0, "reusable", f"库里录下 {library_item.recorded} 块，键都对得上，全部复用，现判 0 态")
    if total is None:
        why = library_note if library_item is None else (
            "库里还没有录这一项的块" if library_item.recorded == 0 else f"库里录下 {library_item.recorded} 块、键对不上 {library_item.to_judge} 块")
        reason = f"{why}；{'、'.join(missing)} 没有不缩域那一趟的真跑记录"
        if defect == "unknown-as-zero":
            return Estimate(subject.label, 0, "unknown", f"弄坏开关：算不出的按 0 态算；{reason}")
        return Estimate(subject.label, None, "unknown", reason)
    if library_item and library_item.recorded > 0 and library_item.to_judge < library_item.recorded:
        fresh = -(-total * library_item.to_judge // library_item.recorded)
        return Estimate(subject.label, fresh, "prior-run-scaled",
                        f"上一趟不缩域真跑共 {total} 态，库里录下 {library_item.recorded} 块、键对不上 {library_item.to_judge} 块，按块数比例估约 {fresh} 态")
    why = library_note if library_item is None else (
        "库里还没有录这一项的块" if library_item.recorded == 0 else f"库里录下的 {library_item.recorded} 块键都对不上")
    return Estimate(subject.label, total, "prior-run", f"上一趟不缩域真跑共 {total} 态，{why}，按全部现判算")


def reader_features(manifest_path, base):
    """读库程序要带的特性串：真跑带的特性 base（逗号隔开，可空）加上包清单 [dev-dependencies] 里每个带特性的依赖的 <依赖>/<特性>。清单读不了抛 OSError、ValueError。"""
    with open(manifest_path, "rb") as handle:
        manifest = tomllib.load(handle)
    features = [name for name in base.split(",") if name]
    for dependency, specification in sorted(manifest.get("dev-dependencies", {}).items()):
        if isinstance(specification, dict):
            features += [f"{dependency}/{feature}" for feature in specification.get("features", [])]
    return ",".join(features)


def read_library_item(root, base_features, subject):
    """起读库程序 crash_library_reader item 读这一项在库里的块账 → (LibraryItem 或 None, 没读到的原因)。环境与编译目录取调用方的。"""
    if not subject.setting_files:
        return None, "这一项没有设点文件"
    try:
        features = reader_features(os.path.join(root, VERIFICATION_MANIFEST), base_features)
    except (OSError, ValueError):
        features = base_features   # 包清单读不到（自证的小仓）就只带真跑的特性
    command = ["cargo", "run", "--release", "--quiet", "--offline", "-p", "singlefs-verification"] + (["--features", features] if features else []) \
        + ["--bin", "crash_library_reader", "--", "item", subject.label, *subject.setting_files]
    try:
        result = subprocess.run(command, cwd=root, capture_output=True, text=True, check=False)
    except OSError as error:
        return None, f"起不了读库程序（{error}）"
    if result.returncode != 0:
        tail = " / ".join(result.stderr.strip().splitlines()[-3:])
        return None, f"读库程序 crash_library_reader item 退 {result.returncode}（{tail}）"
    for line in result.stdout.splitlines():
        if line.split("\t", 1)[0] != LIBRARY_ITEM_TAG:
            continue
        fields = fields_of(line)
        if fields.get("item") == subject.label and all(COUNT.match(fields.get(name, "")) for name in ("recorded_blocks", "to_judge", "reusable")):
            return LibraryItem(int(fields["recorded_blocks"]), int(fields["to_judge"]), int(fields["reusable"])), ""
    return None, "读库程序的输出里没有认得出的 CRASH_LIBRARY_ITEM 那一行"


def refusal_text(estimates, all_parts, constant, defect):
    """拒绝说明（超上限、有项算不出各一句开头）→ 行的列表；每条拒绝都给下一步（两条出路）。"""
    unknown = [estimate for estimate in estimates if estimate.states is None]
    total = sum(estimate.states or 0 for estimate in estimates)
    if unknown:
        head = f"  ✗ 指定范围（--item）没缩域，点名的 {len(unknown)} 项这一趟要现判多少态算不出，不放行（上限 {SCOPED_CRASH_STATES_LIMIT} 态）：不编、不跑，退 {SCOPED_DOMAIN_REFUSED_EXIT}"
    else:
        head = f"  ✗ 指定范围（--item）没缩域，点名的 {len(estimates)} 项这一趟要现判约 {total} 态，超过指定范围的上限 {SCOPED_CRASH_STATES_LIMIT} 态：不编、不跑，退 {SCOPED_DOMAIN_REFUSED_EXIT}"  # gate-lint:summary
    lines = [head]
    for estimate in estimates:
        shown = "算不出" if estimate.states is None else f"约 {estimate.states} 态"
        lines.append(f"     · {estimate.label}：{shown}（{estimate.text}）")
    best = None if defect == "suggestion-dropped" else suggested_depth(all_parts, constant, SCOPED_CRASH_STATES_LIMIT)
    if best:
        advice = f"n 取 {best[0]} 时点名的项上一趟真跑共 {best[1]} 态，在上限以内，可以先试 --crash-expand-up-to {best[0]}"
    else:
        advice = "没有各项在同一展开深度下的缩域记录可推 n，先给一个小的 n（如 --crash-expand-up-to 4）跑一趟，看这一趟报的态数再放大"
    lines.append("  → 怎么办（二选一）：")
    lines.append(f"     ① 带 --crash-expand-up-to <n> 缩域再交：{advice}；缩过域的那一趟结果行写「缩过域、不算门禁通过」，不当门禁通过")
    lines.append("     ② 不缩域就留给提交时不指定范围的那一趟（.claude/rules/verification.md「一个门禁：两种跑法」：指定范围的崩溃状态数压在 20 万量级，不指定范围那一趟不管）")
    return lines


def run_scoped_domain(mode, root, runs_directory, expand_up_to, gpu, defect, subjects, stream=None, error_stream=None):
    """文件头「指定范围的崩溃域上限」。guard：放行返回 0，超上限或算不出返回 SCOPED_DOMAIN_REFUSED_EXIT；report：打 ITEM_DOMAIN 行返回 0。"""
    stream = stream or sys.stdout
    error_stream = error_stream or sys.stderr
    constant = engine_expand_constant(root)
    if mode == "guard" and expand_up_to is not None and (constant is None or expand_up_to < constant):
        print(f"  · 缩过域（展开深度上限缩到 {expand_up_to}）：指定范围的崩溃域上限 {SCOPED_CRASH_STATES_LIMIT} 态不判；这一趟缩过域、不算门禁通过", file=stream)
        return 0
    parts_of = {subject.label: prior_parts(subject, runs_directory, constant) for subject in subjects}
    if mode == "guard":
        bounds = [full_prior(parts_of[subject.label], constant, defect)[0] if subject.has_crash_domain() else 0 for subject in subjects]
        if all(bound is not None for bound in bounds) and sum(bounds) <= SCOPED_CRASH_STATES_LIMIT:
            print(f"  · 指定范围的崩溃域上限 {SCOPED_CRASH_STATES_LIMIT} 态：点名的 {len(subjects)} 项上一趟不缩域真跑共 {sum(bounds)} 态，全部现判也不超上限，放行", file=stream)
            return 0
    estimates = []
    for subject in subjects:
        library_item, note = (None, "没读库") if not subject.has_crash_domain() else read_library_item(root, "gpu" if gpu else "", subject)
        if library_item is None and subject.has_crash_domain():
            print(f"  ! {subject.label}：库里的块账没读到（{note}），改按上一趟真跑的态数估", file=error_stream)
        estimates.append(estimate_subject(subject, library_item, note, runs_directory, constant, defect))
    if mode == "report":
        for estimate in estimates:
            shown = "unknown" if estimate.states is None else str(estimate.states)
            print(f"ITEM_DOMAIN item={estimate.label} states_to_judge={shown} source={estimate.source}", file=stream)
        return 0
    total = sum(estimate.states or 0 for estimate in estimates)
    if all(estimate.states is not None for estimate in estimates) and total <= SCOPED_CRASH_STATES_LIMIT:
        print(f"  · 指定范围的崩溃域上限 {SCOPED_CRASH_STATES_LIMIT} 态：点名的 {len(estimates)} 项要现判约 {total} 态，不超上限，放行", file=stream)
        for estimate in estimates:
            print(f"     · {estimate.label}：约 {estimate.states} 态（{estimate.text}）", file=stream)
        return 0
    all_parts = [parts_of[subject.label] for subject in subjects if subject.has_crash_domain()]
    for line in refusal_text(estimates, all_parts, constant, defect):
        print(line, file=error_stream)
    return SCOPED_DOMAIN_REFUSED_EXIT


def command_scoped_domain(arguments):
    parser = argparse.ArgumentParser(prog="lib_crash_point_replay_items.py scoped-domain")
    parser.add_argument("mode", choices=("guard", "report"))
    parser.add_argument("--root", required=True)
    parser.add_argument("--runs", required=True)
    parser.add_argument("--expand-up-to", type=int, default=None)
    parser.add_argument("--gpu", choices=("0", "1"), default="0")
    parser.add_argument("--defect", choices=SCOPED_DOMAIN_DEFECTS, default="")
    parser.add_argument("--item", action="append", default=[])
    parser.add_argument("--flow", action="append", default=[])
    options = parser.parse_args(arguments)
    root = os.path.abspath(options.root)
    subjects = []
    try:
        for name in options.item:
            subject = merged_subject(root, name)
            if subject is None:
                print(f"  ✗ 合表里没有这一项：{name}", file=sys.stderr)
                print("  → 怎么办：项名照 crash-point-replay.sh --list-items 列的写", file=sys.stderr)
                return 2
            subjects.append(subject)
    except OSError as error:
        print(f"  ✗ 读不了合表要用的登记表（{error}）", file=sys.stderr)
        print("  → 怎么办：看 .claude/gate.d/stage-inputs.tsv 与 scripts/crash/crash-amplification.sh 在不在", file=sys.stderr)
        return 2
    for flow in options.flow:
        name, separator, target = flow.partition(":")
        if not separator or not name or not target:
            print(f"  ✗ --flow 要写 <分项>:<测试目标>，得到「{flow}」", file=sys.stderr)
            print("  → 怎么办：照 crash-amplification.sh 的 ITEMS 一行的头两段写", file=sys.stderr)
            return 2
        subjects.append(Subject(name, [], [name], [target]))
    if not subjects:
        print("  ✗ scoped-domain 一项都没点名", file=sys.stderr)
        print("  → 怎么办：给 --item <合表项> 或 --flow <分项>:<测试目标>", file=sys.stderr)
        return 2
    return run_scoped_domain(options.mode, root, options.runs, options.expand_up_to, options.gpu == "1", options.defect, subjects)


def command_case_states(arguments):
    if len(arguments) != 1:
        print("  ✗ case-states 要一个参数：门禁放的一条崩溃枚举用例日志 log.<用例名>", file=sys.stderr)
        print("  → 怎么办：lib_crash_point_replay_items.py case-states <日志>", file=sys.stderr)
        return 2
    states = case_log_states(arguments[0])
    if states is None:
        print(f"  ✗ 读不出 {arguments[0]} 里的态数（日志不在、没有 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行、有行取不出，或只判了本机那一份）", file=sys.stderr)
        print("  → 怎么办：这条用例这一趟的态数不记（下一趟估指定范围的上限时这一项的崩溃点重放部分算不出，走缩域那条出路）", file=sys.stderr)
        return 1
    print(states)
    return 0


def command_reader_features(arguments):
    if len(arguments) != 2:
        print("  ✗ reader-features 要两个参数：<Cargo.toml> <真跑带的特性，逗号隔开，可空>", file=sys.stderr)
        print("  → 怎么办：lib_crash_point_replay_items.py reader-features crates/singlefs-verification/Cargo.toml gpu", file=sys.stderr)
        return 2
    try:
        print(reader_features(arguments[0], arguments[1]))
    except (OSError, ValueError) as error:
        print(f"  ✗ 读不了 {arguments[0]} 的 [dev-dependencies]：{error}", file=sys.stderr)
        print("  → 怎么办：看那份清单可不可读、写法对不对；读库程序与真跑要编同一套特性，键才对得上", file=sys.stderr)
        return 1
    return 0


def main(arguments):
    if arguments and arguments[0] in ("list", "check", "segments") and len(arguments) <= 2:
        root = os.path.abspath(arguments[1]) if len(arguments) == 2 else REPOSITORY_ROOT
        return {"list": command_list, "check": command_check, "segments": command_segments}[arguments[0]](root)
    if arguments and arguments[0] == "scoped-domain":
        return command_scoped_domain(arguments[1:])
    if arguments and arguments[0] == "reader-features":
        return command_reader_features(arguments[1:])
    if arguments and arguments[0] == "case-states":
        return command_case_states(arguments[1:])
    print(f"  ✗ 认不出的参数 {' '.join(arguments) or '（没给）'}")
    print("  → 怎么办：lib_crash_point_replay_items.py list [仓根] | check [仓根] | segments [仓根] | scoped-domain … | reader-features …（核法的自证在 scripts/crash/crash-case-check.py --selftest）")
    return 2


def refuse_break_outside_selftest():
    """弄坏开关只在自证里认（文件头「弄坏开关」）：本库自己的命令行（list、check、segments）没有自证，核法的自证由 scripts/crash/crash-case-check.py --selftest 在进程里设；
    所以从命令行起而环境里设了 CRASH_POINT_REPLAY_ITEMS_BREAK（不管值是什么），一律退 2 并点名它只在自证里认；在参数解析之前判。"""
    if os.environ.get("CRASH_POINT_REPLAY_ITEMS_BREAK"):
        print("  ✗ CRASH_POINT_REPLAY_ITEMS_BREAK 只在自证里认（弄坏开关只给 scripts/crash/crash-case-check.py --selftest 证红用），这一趟不是自证", file=sys.stderr)
        print("     → 怎么办：去掉 CRASH_POINT_REPLAY_ITEMS_BREAK 再跑；要证红就跑 CRASH_POINT_REPLAY_ITEMS_BREAK=<项> python3 scripts/crash/crash-case-check.py --selftest", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    refusal = refuse_break_outside_selftest()
    if refusal:
        sys.exit(refusal)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
