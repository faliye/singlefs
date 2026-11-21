#!/usr/bin/env python3
# admission: always 读的是调用方这一次给的基准行数与此刻行数，不存结论
# run-condition: none 纯函数，只用 python 标准库，不读文件、不起子进程
"""滞后表「只缩不涨」认「一份文件拆成多份」的共用写法：拆分族。

拆分族：基准里一行点名一份入口文件，它的族是入口本身，加按它的拆法命名的部分文件。入口的后缀决定认哪几种拆法（命名取自仓里已拆的文件，路径都是仓内相对路径）：
  .rs 入口 <目录>/<名>.rs：<目录>/<名>/ 下（含再往下各层）的全部 .rs（含 <名>/main.rs）。
    对的是把 foo.rs 拆成 foo.rs 加 foo/ 下的子模块、把 tests/x.rs、src/bin/x.rs 挪成 tests/x/main.rs、src/bin/x/main.rs 加同目录子模块这类拆法。
  .rs 入口 <目录>/main.rs 与 <目录>/mod.rs（入口自己就是目录的入口文件）：<目录>/ 下（含再往下各层）的全部 .rs。
    对的是基准里点名的已经是 x/main.rs 或 x/mod.rs，之后又把里面的代码拆到同目录与下层的子模块这类拆法。
  Python 拆法（入口是 .py 或 .sh）：入口同目录下的 <模块名>_<部分>.py，模块名是入口文件名去掉后缀、「-」换成「_」（Python 模块名里不许有「-」）。
    例：task-server/task_server_lib.py 加 task_server_lib_commands.py；scripts/mutation/broad-mutation.py 加 broad_mutation_base.py；scripts/mutation/prove-red.sh 加 prove_red_core.py；.claude/hooks/write-guard.sh 加 write_guard_decisions.py。
  钩子拆法（入口是 .sh）：入口同目录下的 <名>-<部分>.sh 与 <名>-<部分>.py。
    例：scripts/gate/merge-and-verify.sh 加 merge-and-verify-steps-lib.sh。
  shell 拆法（入口是 .claude/gate.d/ 下的 .sh，即门禁阶段）：.claude/gate.d/lib/ 下的 <名>-<部分>.sh 与 <名>-<部分>.py。
    例：.claude/gate.d/code-tooling.sh 加 .claude/gate.d/lib/code-tooling-source-scan-cells.sh；.claude/gate.d/broad-mutation.sh 加 .claude/gate.d/lib/broad-mutation-samples.py。
  <部分>非空、不含「/」；部分文件再拆出的文件名以部分文件名起头，同样落在入口的族里（verus_proof_check_selftest.py 的 verus_proof_check_selftest_fixtures.py）。
  别的后缀（.py 入口的 <名>-<部分>.py、.sh 入口在 lib/ 之外别的子目录里的文件）没有族，只认入口自己。
  搬迁登记一个旧前缀只能换一个新前缀（.claude/gate.d/lib-lag-baseline.py 文件头「搬迁登记」），一份旧文件拆成几份新文件它换算不了，所以由比法这里认族。
  同目录、同后缀而名字对不上拆法的文件（别的 .rs、别的脚本、同名不同后缀的文件）、别的目录的路径是族外。

比法：每个此刻的路径先认出它属于基准里的哪一行（family_owner）：它自己就在基准里的取自己；否则取族里含它的那些基准行里族前缀（目录加部分文件名的前缀）最长的那一行（嵌套时取最近的一层，例：task_server_lib_commands.py 归 task_server_lib.py，不归 task_server.py；a/b.rs 与 a/main.rs 都在基准里时 a/b/c.rs 归 a/b.rs）；
  都不含的是族外。归到同一行基准的几个路径，计数之和不超过基准那一行就不算新加；之和超了，这几个路径都判红；族外的路径判红。
  计数是调用方定的单位：comment-history-lag.tsv 是命中的行数；verus-trait-impls-outside-lag.tsv 一行登记算 1，按（trait、实现类型）分组各比各的。

调它的（各自留键、数与报法，认族与比之和只在这一份；都用 import 载入，同在 scripts/gate/ 下）：
  scripts/gate/comment-history-check.py 的 lag_growth（门禁 doc-text 的 comment-history 格，键是路径，数是命中行数）；
  scripts/gate/verus-proof-check.py 的 trait_impl_lag_growth（门禁 formal-verification 的 verus-proof 格，按（trait、类型）分组比，经 grouped_family_growth，文件落在族内）。

别的判法要认族，用这三个函数（sys.path 里放 scripts/gate，再 import lib_lag_split_family as split_family；路径都是仓内相对路径，这一份不读文件系统）：
  split_family.in_family(入口路径, 路径)                  路径是不是入口的族里的文件（入口自己算）
  split_family.family_members(入口路径, 路径列表)         路径列表（调用方给，例如 git ls-files 的输出）里属于入口的族的路径，次序照列表，交列表
  split_family.family_owner(路径, 基准里的路径们)         路径归到基准里的哪一个入口（自己在基准里取自己，否则取族前缀最长的那一个），族外交 None
键带路径的表（排除表一行一个「路径加锚串」）按键比，用这一个函数（调它的是门禁 code-tooling 的 heavy-thread-defaults、hardcoded-quota、repository-copy-closure 三格）：
  split_family.keyed_family_growth(基准里的键们, 此刻的键们)   键是（路径, 路径之外的那一段）；此刻一个键的路径归到基准里的哪个入口（family_owner），
                                                             按（入口, 段）比计数之和（与 family_growth 同一种比法）：此刻同一组各键的计数之和不超过
                                                             基准里那个入口下同一段的计数就不算新加，超了这一组的键都交出来，族外与段对不上的交出来；次序照此刻。
                                                             计数是调用方给的：给 {键: 计数} 的按它比（heavy-thread-defaults 给锚串在那份文件里命中的被判行数，
                                                             拆分把同一处写法分到两份文件、各登一行，命中之和不超过基准那一份里的命中数就不涨）；
                                                             给集合或列表的每个键算 1（列表里重复的照次数算；hardcoded-quota、repository-copy-closure 两格比的是行数）
认族、之和相等绿、之和超了红、族外红、tests/x.rs 挪成 tests/x/main.rs 绿、main.rs 与 mod.rs 入口认同目录兄弟绿、Python 拆法绿、钩子拆法绿、shell 拆法绿、同目录不相干文件族外红，
  都由那两份的自证与它们的弄坏开关证；按键认族由三格的样本证（三格各有一条弄坏开关，打开后传 unrecognized）；按键比计数之和由这一份自己的 --selftest 证。

用法（这一份只被 import；直接执行只跑自证）：
  python3 scripts/gate/lib_lag_split_family.py --selftest   逐格喂 keyed_family_growth 几组（基准, 此刻），比交出的判涨键；全对退 0，有一格不对退 1，用法错退 2
自证的弄坏开关 LAG_SPLIT_FAMILY_BREAK=<项>（只给 --selftest 证红用：设了而不是 --selftest 退 2；开关名映射成下面 defects 的同名一项交给被测函数）。
  逐项写开关名、弄坏哪一处、打开后自证哪几格红（失败行「✗ lib_lag_split_family.py 自证：<格名>：…」）：
  LAG_SPLIT_FAMILY_BREAK=keyed-count-unchecked  keyed_family_growth 不比计数，只看（入口, 段）在不在基准里
      「✗ lib_lag_split_family.py 自证：同族新文件照抄基准入口的两个锚串，命中之和超过基准：判涨族里那一组的四个键」
      「✗ lib_lag_split_family.py 自证：入口没拆、命中比基准多：判涨那一个键」
      「✗ lib_lag_split_family.py 自证：基准那一份原样留着、族里新文件又命中一处：判涨这一组的两个键」
      「✗ lib_lag_split_family.py 自证：只给键（集合与列表）时每个键算 1：同一锚串在族里两份文件各登一行、基准一行，判涨这一组的两个键」
  LAG_SPLIT_FAMILY_BREAK=sum-unchecked  keyed_family_growth 不比之和：每个键自己的计数与基准那一组比
      「✗ lib_lag_split_family.py 自证：同族新文件照抄基准入口的两个锚串，命中之和超过基准：判涨族里那一组的四个键」
      「✗ lib_lag_split_family.py 自证：基准那一份原样留着、族里新文件又命中一处：判涨这一组的两个键」
      「✗ lib_lag_split_family.py 自证：只给键（集合与列表）时每个键算 1：同一锚串在族里两份文件各登一行、基准一行，判涨这一组的两个键」

弄坏开关的接口（只给调用方的弄坏开关与这一份的自证用，平常传空）：defects 是下面几项的集合，各由调用方自己的开关名映射过来：
  unrecognized   不认族：只认路径自己（拆分之后的新文件都成了族外）
  main-excluded  族里不含 <名>/main.rs
  wide           族认宽：同目录下的任何 .rs 都算在族里（同目录的兄弟文件放过）；脚本入口的族认宽到拆法认的那几个目录里的任何 .py、.sh（后缀照拆法）
  sum-unchecked  不比之和：归到同一行基准的每个路径各自与基准那一行比（family_growth 与 keyed_family_growth 都认）
  directory-entry-unrecognized  不认 main.rs、mod.rs 入口的同目录兄弟：它们的族退回 <目录>/main/、<目录>/mod/ 下的 .rs（同目录与下层的子模块都成了族外）
  python-split-unrecognized  不认 Python 拆法：<模块名>_<部分>.py 都成了族外
  hook-split-unrecognized    不认钩子拆法：同目录的 <名>-<部分>.sh、<名>-<部分>.py 都成了族外
  shell-split-unrecognized   不认 shell 拆法：.claude/gate.d/lib/ 下的 <名>-<部分>.sh、<名>-<部分>.py 都成了族外
  keyed-count-unchecked      keyed_family_growth 不比计数：（入口, 段）在基准里就放过，同族新文件照抄已有的段、命中再多都不涨
"""
import os
import sys
from collections import Counter, namedtuple

UNRECOGNIZED = "unrecognized"
MAIN_EXCLUDED = "main-excluded"
WIDE = "wide"
SUM_UNCHECKED = "sum-unchecked"
DIRECTORY_ENTRY_UNRECOGNIZED = "directory-entry-unrecognized"
PYTHON_SPLIT_UNRECOGNIZED = "python-split-unrecognized"
HOOK_SPLIT_UNRECOGNIZED = "hook-split-unrecognized"
SHELL_SPLIT_UNRECOGNIZED = "shell-split-unrecognized"
KEYED_COUNT_UNCHECKED = "keyed-count-unchecked"
KNOWN_DEFECTS = (UNRECOGNIZED, MAIN_EXCLUDED, WIDE, SUM_UNCHECKED, PYTHON_SPLIT_UNRECOGNIZED, HOOK_SPLIT_UNRECOGNIZED, SHELL_SPLIT_UNRECOGNIZED,
                 DIRECTORY_ENTRY_UNRECOGNIZED, KEYED_COUNT_UNCHECKED)
# 自证的弄坏开关（文件头「自证的弄坏开关」）：变量名与认得的项
BREAK_VARIABLE = "LAG_SPLIT_FAMILY_BREAK"
SELFTEST_BREAKS = (KEYED_COUNT_UNCHECKED, SUM_UNCHECKED)
SELFTEST_LABEL = "lib_lag_split_family.py 自证"

GATE_STAGE_DIRECTORY = ".claude/gate.d"
SCRIPT_SUFFIXES = (".py", ".sh")
DIRECTORY_ENTRY_NAMES = ("main.rs", "mod.rs")

# 一个判红的路径：path 是此刻的路径，own 是它自己的计数；owner 是归到的基准行的路径（族外是 None）；total 是归到同一行的几个路径的计数之和（族外等于 own）；
# allowed 是基准那一行的计数（族外是 0）；members 是归到同一行的路径有几个
Growth = namedtuple("Growth", ["path", "own", "owner", "total", "allowed", "members"])


def is_directory_entry(path, defects=frozenset()):
    """.rs 路径是不是目录的入口文件（main.rs、mod.rs）；弄坏开关 directory-entry-unrecognized 下一律不是。"""
    return DIRECTORY_ENTRY_UNRECOGNIZED not in defects and path.endswith(".rs") and path.rpartition("/")[2] in DIRECTORY_ENTRY_NAMES


def family_directory(path, defects=frozenset()):
    """.rs 入口的族里子模块所在目录的前缀：目录的入口文件（main.rs、mod.rs）是 <目录>/，别的 <目录>/<名>.rs 是 <目录>/<名>/；不是 .rs 的路径没有这一种族目录，交 None。"""
    if not path.endswith(".rs"):
        return None
    if is_directory_entry(path, defects):
        return path[:len(path) - len(path.rpartition("/")[2])]
    return path[:-len(".rs")] + "/"


def script_recipes(owner):
    """.py 或 .sh 入口认哪几种拆法：[(这一种拆法的弄坏开关名, 部分文件所在目录的前缀, 部分文件名的前缀, 部分文件的后缀们)]；别的后缀交 []。"""
    suffix = owner[-len(".py"):]
    if suffix not in SCRIPT_SUFFIXES:
        return []
    directory, _slash, name = owner.rpartition("/")
    stem = name[:-len(suffix)]
    own_directory = directory + "/" if directory else ""
    recipes = [(PYTHON_SPLIT_UNRECOGNIZED, own_directory, stem.replace("-", "_") + "_", (".py",))]
    if suffix == ".sh":
        recipes.append((HOOK_SPLIT_UNRECOGNIZED, own_directory, stem + "-", SCRIPT_SUFFIXES))
        if directory == GATE_STAGE_DIRECTORY:
            recipes.append((SHELL_SPLIT_UNRECOGNIZED, directory + "/lib/", stem + "-", SCRIPT_SUFFIXES))
    return recipes


def script_family_prefix_length(owner, path, defects=frozenset()):
    """path 是 .py 或 .sh 入口 owner 的部分文件时，认出它的那一种拆法的前缀（目录加部分文件名的前缀）有多长，不是交 None。
    部分文件：落在某一种拆法的目录里、后缀对、名字以那一种拆法的前缀起头且前缀之后还有部分名、不再往下一层目录。"""
    for recipe, directory, prefix, suffixes in script_recipes(owner):
        if recipe in defects or not path.startswith(directory) or not path.endswith(suffixes):
            continue
        name = path[len(directory):]
        if "/" in name:
            continue
        if WIDE in defects:
            return len(directory)
        if name.startswith(prefix) and name[len(prefix):-len(".py")]:
            return len(directory) + len(prefix)
    return None


def family_prefix_length(owner, path, defects=frozenset()):
    """path 在 owner 这一行基准的族里时，认出它的族前缀有多长（owner 自己交它的整条路径的长度）：族是 owner 自己，或 owner 按它的拆法命名的部分文件（文件头逐种写）；
    前缀越长族越近，嵌套的几个族都含 path 时 family_owner 取最长的；不在族里交 None。"""
    if path == owner:
        return len(path)
    if UNRECOGNIZED in defects:
        return None
    prefix = family_directory(owner, defects)
    if prefix is None:
        return script_family_prefix_length(owner, path, defects)
    if not path.endswith(".rs"):
        return None
    if WIDE in defects:
        same_directory = owner.rsplit("/", 1)[0] + "/" if "/" in owner else ""
        return len(same_directory) if path.startswith(same_directory) else None
    if MAIN_EXCLUDED in defects and not is_directory_entry(owner, defects) and path == prefix + "main.rs":
        return None
    return len(prefix) if path.startswith(prefix) else None


def in_family(owner, path, defects=frozenset()):
    """path 是不是 owner 这一行基准的族里的文件：owner 自己，或 owner 按它的拆法命名的部分文件（文件头逐种写）。"""
    return family_prefix_length(owner, path, defects) is not None


def family_members(owner, paths, defects=frozenset()):
    """paths（仓内相对路径的列表）里 owner 的族里的路径，次序照 paths；owner 自己在 paths 里才算在内。"""
    return [path for path in paths if in_family(owner, path, defects)]


def family_owner(path, base_paths, defects=frozenset()):
    """path 归到的基准行的路径：自己在基准里取自己，否则取族里含它的基准路径里族前缀最长（最近一层）的一个，前缀一样长取路径更长的；族外交 None。"""
    if path in base_paths:
        return path
    lengths = {owner: family_prefix_length(owner, path, defects) for owner in base_paths}
    owners = [owner for owner, length in lengths.items() if length is not None]
    return max(owners, key=lambda owner: (lengths[owner], len(owner), owner)) if owners else None


def family_growth(base_counts, current_counts, defects=frozenset()):
    """基准 {路径: 计数} 与此刻 {路径: 计数}（都是调用方按自己的键整理好的，一份基准行一个路径）比，交判红的 [Growth]，次序是此刻的路径出现的次序。
    族外的路径交 owner=None；归到同一行基准的几个路径之和超过那一行的，这几个路径都交（owner 是那一行）。"""
    owned, growth = {}, []
    for path, count in current_counts.items():
        owner = family_owner(path, base_counts, defects)
        if owner is None:
            growth.append(Growth(path, count, None, count, 0, 1))
        else:
            owned.setdefault(owner, []).append((path, count))
    for owner, rows in owned.items():
        total, allowed = sum(count for _path, count in rows), base_counts[owner]
        for path, count in rows:
            if (count if SUM_UNCHECKED in defects else total) > allowed:
                growth.append(Growth(path, count, owner, total, allowed, len(rows)))
    order = {path: index for index, path in enumerate(current_counts)}
    return sorted(growth, key=lambda item: order[item.path])


def keyed_counts(keys):
    """{键: 计数}（照原样，计数是非负整数）、集合或列表（每出现一次算 1）→ 次序照给的次序的 {键: 计数}。"""
    if isinstance(keys, dict):
        return dict(keys)
    counts = {}
    for key in keys:
        counts[key] = counts.get(key, 0) + 1
    return counts


def keyed_family_growth(base_keys, current_keys, defects=frozenset()):
    """键带路径的行（排除表一行一个（路径, 锚串））的比法：两边都是（路径, 段）的 {键: 计数}、集合或列表（keyed_counts：集合与列表每个键算 1）；
    交此刻判涨的键，次序照此刻、不重复。此刻一个键的路径先归到基准里的入口（family_owner），按（入口, 段）分组，组里各键的计数相加，
    与基准里那个入口下同一段的计数比（与 family_growth 比之和同一种比法）：超了这一组的键都交出来；族外、段对不上的照交。
    弄坏开关 keyed-count-unchecked：不比计数，只看（入口, 段）在不在基准里；sum-unchecked：不比之和，每个键自己的计数与基准那一组比。"""
    allowed = keyed_counts(base_keys)
    current = keyed_counts(current_keys)
    base_paths = {path for path, _segment in allowed}
    groups = {key: (family_owner(key[0], base_paths, defects), key[1]) for key in current}
    if KEYED_COUNT_UNCHECKED in defects:
        return [key for key, group in groups.items() if group not in allowed]
    totals = {}
    for key, group in groups.items():
        totals[group] = totals.get(group, 0) + current[key]
    return [key for key, group in groups.items()
            if (current[key] if SUM_UNCHECKED in defects else totals[group]) > allowed.get(group, 0)]


def grouped_family_growth(base_groups, current_groups, defects=frozenset()):
    """分组的比法：两边都是 {组: {路径: 计数}}，各组各比各的（组由调用方定，verus 那一张是（trait、实现类型））；此刻有而基准里没有的组，路径全是族外。
    交 [(组, Growth)]。"""
    return [(group, item) for group, counts in current_groups.items() for item in family_growth(base_groups.get(group, {}), counts, defects)]


# 自证喂 keyed_family_growth 的几组：(格名, 基准里的键, 此刻的键, 要交出的判涨键)。计数照 heavy-thread-defaults 给的写（锚串在那份文件里命中的被判行数）；
# 路径照仓里一份 Python 拆法的入口（admission.py 与它的部分文件）造
CORE_COUNT_ANCHOR = "os.cpu_count(), len(os.sched_getaffinity(0))"
NPROC_ANCHOR = '"nproc"'
ENTRY = "scripts/gate/admission.py"
PART = "scripts/gate/admission_base.py"
OTHER_PART = "scripts/gate/admission_selftest_cells.py"
NEW_PART = "scripts/gate/admission_thread_probe.py"
OUTSIDE = "scripts/gate/thread_probe.py"
KEYED_CELLS = (
    ("拆分把基准那一份里的两处命中分到两份部分文件、各登一行，命中之和等于基准：不涨",
     {(ENTRY, NPROC_ANCHOR): 2},
     {(PART, NPROC_ANCHOR): 1, (OTHER_PART, NPROC_ANCHOR): 1},
     []),
    ("同族新文件照抄基准入口的两个锚串，命中之和超过基准：判涨族里那一组的四个键",
     {(ENTRY, CORE_COUNT_ANCHOR): 1, (ENTRY, NPROC_ANCHOR): 1},
     {(PART, CORE_COUNT_ANCHOR): 1, (PART, NPROC_ANCHOR): 1, (NEW_PART, CORE_COUNT_ANCHOR): 1, (NEW_PART, NPROC_ANCHOR): 1},
     [(PART, CORE_COUNT_ANCHOR), (PART, NPROC_ANCHOR), (NEW_PART, CORE_COUNT_ANCHOR), (NEW_PART, NPROC_ANCHOR)]),
    ("拆分照实挪：命中各挪进拆出的部分文件、数不变，不涨",
     {(ENTRY, CORE_COUNT_ANCHOR): 1, (ENTRY, NPROC_ANCHOR): 1},
     {(PART, CORE_COUNT_ANCHOR): 1, (PART, NPROC_ANCHOR): 1},
     []),
    ("入口没拆、命中比基准多：判涨那一个键",
     {(ENTRY, NPROC_ANCHOR): 1},
     {(ENTRY, NPROC_ANCHOR): 2},
     [(ENTRY, NPROC_ANCHOR)]),
    ("基准那一份原样留着、族里新文件又命中一处：判涨这一组的两个键",
     {(ENTRY, NPROC_ANCHOR): 1},
     {(NEW_PART, NPROC_ANCHOR): 1, (ENTRY, NPROC_ANCHOR): 1},
     [(NEW_PART, NPROC_ANCHOR), (ENTRY, NPROC_ANCHOR)]),
    ("族外新文件照抄锚串：判涨",
     {(ENTRY, CORE_COUNT_ANCHOR): 1},
     {(PART, CORE_COUNT_ANCHOR): 1, (OUTSIDE, CORE_COUNT_ANCHOR): 1},
     [(OUTSIDE, CORE_COUNT_ANCHOR)]),
    ("族里新文件写基准里没有的锚串：判涨",
     {(ENTRY, CORE_COUNT_ANCHOR): 1},
     {(PART, CORE_COUNT_ANCHOR): 1, (NEW_PART, "multiprocessing.cpu_count()"): 1},
     [(NEW_PART, "multiprocessing.cpu_count()")]),
    ("只给键（集合与列表）时每个键算 1：同一锚串在族里两份文件各登一行、基准一行，判涨这一组的两个键",
     {(ENTRY, NPROC_ANCHOR)},
     [(PART, NPROC_ANCHOR), (OTHER_PART, NPROC_ANCHOR)],
     [(PART, NPROC_ANCHOR), (OTHER_PART, NPROC_ANCHOR)]),
    ("只给键时基准里同一个键登了两行（列表）：此刻族里两行照放过",
     [(ENTRY, NPROC_ANCHOR), (ENTRY, NPROC_ANCHOR)],
     [(PART, NPROC_ANCHOR), (OTHER_PART, NPROC_ANCHOR)],
     []),
)


def selftest(defects):
    """逐格喂 KEYED_CELLS，比交出的判涨键（连次序）；→ 不对的格数。"""
    wrong = 0
    for name, base_keys, current_keys, wanted in KEYED_CELLS:
        got = keyed_family_growth(base_keys, current_keys, defects)
        if got == wanted:
            print(f"  ✓ {SELFTEST_LABEL}：{name}")
        else:
            wrong += 1
            print(f"  ✗ {SELFTEST_LABEL}：{name}：交出 {got}，要 {wanted}")
    print(f"  {'✓' if not wrong else '✗'} {SELFTEST_LABEL}：{len(KEYED_CELLS)} 格，{len(KEYED_CELLS) - wrong} 格对、{wrong} 格不对"
          + (f"（开着弄坏开关 {BREAK_VARIABLE}={','.join(sorted(defects))}）" if defects else ""))
    return wrong


def main(arguments):
    switches = [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]
    if arguments != ["--selftest"]:
        print("用法：python3 scripts/gate/lib_lag_split_family.py --selftest（这一份只被 import；直接执行只跑自证）", file=sys.stderr)
        if switches:
            print(f"  ✗ 设了 {BREAK_VARIABLE}：弄坏开关只给 --selftest 证红用", file=sys.stderr)
        return 2
    unknown = [item for item in switches if item not in SELFTEST_BREAKS]
    if unknown:
        print(f"  ✗ {BREAK_VARIABLE} 里有不认得的项 {unknown}（认得的：{', '.join(SELFTEST_BREAKS)}）", file=sys.stderr)
        return 2
    return 1 if selftest(frozenset(switches)) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
