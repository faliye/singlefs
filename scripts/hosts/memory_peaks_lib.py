"""内存峰值表、取键与「这台此刻还放得下多少」的共用库：scripts/hosts/run-with-memory-cap.sh（内存包装）与
scripts/hosts/host-dispatch.py（统一派发器）都 import 它，取键、读峰值表、读账、算余量只在这一份里写。

被 import 的库，除自证（python3 scripts/hosts/memory_peaks_lib.py --selftest）之外不单独跑、不写准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md「不判的三类」）；
import 本身不起子进程、不开文件。用法（脚本按路径跑时 sys.path[0] 就是这个目录；bash 里写 python3 -B -c 'import sys; sys.path.insert(0, <目录>); …'）：
    import memory_peaks_lib as peaks
    key = peaks.command_key(命令的各个词, 环境里的 RUN_WITH_MEMORY_CAP_KEY)
派发器把这份源码整份送到第二台跑（python3 -c），在那边算那台的余量：所以这里只用标准库，不 import 同目录的别的文件；
  文件末尾的自证入口只在按文件路径跑（有 __file__）、参数正好是 --selftest 时起，经 python3 -c 送过去的那一份不起。

键（与峰值表第四列相同）：给了 RUN_WITH_MEMORY_CAP_KEY 就用它；不给就把命令的各个词用空格接起来，去掉测试二进制名里 -<16 位十六进制> 的哈希；
  两种都把制表符、换行换成空格，再按 normalized_key 换算（下面三条），截到 KEY_LENGTH_LIMIT 个字。
  ① 剥掉线程包装 capped.sh N：词的文件名是 capped.sh、后面那个词不以 - 开头、再后面还有词的，这两个词连同紧挨在前面的 shell 名（bash、sh 这类）一起去掉
    （与 .claude/hooks/lib_shell_words.py 剥 WRAPPERS_TAKING_ONE_ARGUMENT 同一认法：N 写数还是 default 都剥；不看是不是在命令位置上，峰值表里没有 capped.sh 当参数的键）。
    经 capped.sh 起与不经起的同一条命令落一个键，与资源提醒钩子交给 host-resources.py 的剥掉包装之后的词对得上。
  ② 本机路径换成固定记号：工作根与过渡期旧根（取法同 .claude/hooks/lib_work_root.py 的 scratch_roots：SINGLEFS_WORK_ROOT 或 ~/.local/share/singlefs/work，
    与 /tmp/claude-<uid>；这里只用标准库、送到第二台的那一份也要算得出，不 import 它，自证里核两边取到的一样）下面那一段——路径里有 /target/ 的，
    根到 target 之前那一截（副本与草稿目录，多深都算）；没有的，根下第一层目录——换成 SCRATCH_TOKEN；仓根（本库在 <仓根>/scripts/hosts/ 下）换成 REPOSITORY_TOKEN；
    家目录换成 HOME_TOKEN。先换工作根、再换仓根、最后换家目录（前两样都在家目录底下）。路径要从词首、空白、= : 引号逗号左括号之后起才认。
    两份副本、两个草稿目录里的同一条命令落一个键；从工作根下的副本里 import 这份库时，它认的仓根是那份副本（已被工作根那一条换掉），主仓的路径只换得到家目录那一截。
  ③ 换算是幂等的：换过的键再换一次不变（读表时对每一行的键照样换算，见下面「旧键」）。
旧键（这套换算之前记的）：read_peak_rows、read_duration_table 读表时把每一行的键照 normalized_key 换算，换算后落到同一个键的几行并成一行
  （最近一次实测取量的时刻最晚的那一行；近几次按各行最近一次的时刻从新到旧接起来，至多 RECENT_MEASUREMENTS 次；用时表只留时刻最晚的那一行）。
  包装下一次写表时整张表按换算后的键写回，旧键就此换完。
  旧键原本就被截到 KEY_LENGTH_LIMIT 个字、换算又缩短了的（截掉的尾巴找不回来），换算后的键只是新键的开头一截：read_peak_rows 交回的表（PeakRows）
  按键取不到时，退到这种行里「是这个键的开头」的那几行（几行并成一行交回）；包装往表里写这个键时（rows[键] = …）顺手删掉被它接上的那几行。
峰值表的口径与列见 run-with-memory-cap.sh 文件头「峰值表」一段；这里只读不写（写在包装那一边，同一把锁里）。
  记账与预测都按「近几次」（第六列，至多 RECENT_MEASUREMENTS 次）读：预测取其中最大的，其中有一次撞顶就算撞过顶（recorded_peak）；
  包装的排队、派发器的 predict、scripts/hosts/host-resources.py 都经 recorded_peak 读，不各写一份。
  read_peak_table 照旧交四段（峰值、时刻、上限、线程数），只认四列、五列的旧包装还在跑时 import 的是这一份，它按四段写回。
账（${状态目录}/ledger/<unit>.json）与「还没涨到量的那一截」的算法见 run-with-memory-cap.sh 文件头「排队」一段；这里的函数就是那一段的实现。
用时表（每条命令上一次跑了多久）：scripts/hosts/host-dispatch.py 派出去、跑完的那一条由它经 record_duration 记一行；
  派发器与 scripts/hosts/host-resources.py 经 recorded_duration 读，判「历史用时超过 LONG_RUN_SECONDS 而只派到一台」。
  默认 ${XDG_STATE_HOME:-~/.local/state}/singlefs/run-durations.tsv（跨重启、不进仓），HOST_DISPATCH_DURATIONS 可换；键同峰值表。
  列见 DURATION_TABLE_HEADER；同一个键只留最近一次。峰值表不加这一列：在跑的旧包装只认它认得的那几列，多一列它写回时就丢了。
"""
import contextlib
import fcntl
import json
import os
import re
import subprocess
import sys

MEBIBYTE = 1024 * 1024
KEY_LENGTH_LIMIT = 300
TEST_BINARY_HASH = re.compile(r"-[0-9a-f]{16}\b")
# 键里本机路径换成的固定记号（文件头「键」②）
SCRATCH_TOKEN = "〈工作根下的目录〉"
REPOSITORY_TOKEN = "〈仓根〉"
HOME_TOKEN = "〈家目录〉"
THREAD_WRAPPER_NAME = "capped.sh"                       # 文件头「键」①：剥掉的线程包装
SHELL_NAMES = {"bash", "sh", "dash", "zsh", "ksh"}       # 起它的 shell 名，与 .claude/hooks/lib_shell_words.py 的 SHELL_NAMES 同一组
WORK_ROOT_VARIABLE = "SINGLEFS_WORK_ROOT"                # 工作根的取法同 .claude/hooks/lib_work_root.py（文件头「键」②）
DEFAULT_WORK_ROOT = "~/.local/share/singlefs/work"
PATH_START = r"(?<![^\s=:'\",(])"                        # 路径从词首、空白、= : 引号逗号左括号之后起才认
# 只给自证判红用的弄坏开关（MEMORY_PEAKS_LIB_BREAK，只在 --selftest 里读进来；别处 import 这份库时一律不认，见文件末尾「自证」）
BREAK_VARIABLE = "MEMORY_PEAKS_LIB_BREAK"
KNOWN_BREAK_MODES = ("scratch-paths-kept", "capped-kept", "stored-keys-raw", "truncated-keys-unmatched", "outside-selftest-unnamed")
_break_modes = frozenset()
_path_replacements_cache = {}   # path_replacements 建好的那一份（键是此刻的 SINGLEFS_WORK_ROOT、HOME 与弄坏开关）
SIZE_UNITS = {"K": 1024, "M": MEBIBYTE, "G": 1024 * MEBIBYTE, "T": MEBIBYTE * MEBIBYTE}
# 峰值表里查不到这条命令时按多少算（包装的 auto、派发器的预测都用它）：取 scripts/experiment-apparatus/replay.sh 的 REPLAY_MEMORY_CAP 默认值，
# 与 scripts/hosts/multi-host-run.sh 本机默认上限同一个数；本仓日常的 cargo test 与实验装置的峰值表里绝大多数在它以下。推的，没逐条量过
DEFAULT_MEMORY_PREDICTION_TEXT = "8G"
# 峰值表第六列「近几次」记最近这么多次实测（run-with-memory-cap.sh 文件头「峰值表」一段）：撞顶之后要连着这么多次干净跑完才回到按峰值记账
RECENT_MEASUREMENTS = 5
RECENT_HIT, RECENT_CLEAN = "hit", "clean"   # 第六列里每一次的标记：撞没撞顶
# 上限 = 预测 × CAP_FACTOR（派发器）；内存包装判「这一次峰值比记账大了这么多倍以上：工作量可能变了」也用它。依据写在 host-dispatch.py 文件头「常量的依据」
CAP_FACTOR = 1.5
DEFAULT_SLICE = "singlefs-heavy.slice"
DEFAULT_RESERVE_TEXT = "20G"          # 与 run-with-memory-cap.sh 的 DEFAULT_RESERVE 同一个数，依据写在它文件头「余量」一段
# 历史用时或预计挂钟超过它、而派法只占一台，派发器拒、资源报告提醒：用户的常设规矩「凡是超过2两小时的脚本都要考虑多机 分片 跑」
# （records/2026-09-16-subagent拆分提案.md 第五十九节第 5 条）
LONG_RUN_SECONDS = 2 * 3600
DURATION_TABLE_HEADER = [
    "# 每条命令上一次经 scripts/hosts/host-dispatch.py 派出去、跑完时的用时：派发器建、写，派发器与 scripts/hosts/host-resources.py 读（判长活只派一台），别手改。",
    "# 口径：派发器起派活那条链（multi-host-run.sh，带 --gpu 时连同清场包装）到它退出的挂钟秒数，向上取整；含拷树、排队与取回，偏大不偏小。",
    "#   那条链退 2、251、252、255（命令没跑成）或派发器收到 TERM / INT / HUP（跑到一半）的不记。只在派它的那台成立，不进仓。",
    "# 列（制表符分隔）：用时秒数、量的时刻（ISO 8601，Z 结尾）、那条链的退出码、跑在哪台（local / peer）、键（同内存峰值表）。",
]


def command_key(words, explicit_key=None):
    """峰值表的键（文件头「键」）。words 是命令的各个词；explicit_key 是 RUN_WITH_MEMORY_CAP_KEY（空串当没给）。"""
    if explicit_key:
        joined = explicit_key
    else:
        joined = TEST_BINARY_HASH.sub("", " ".join(words))
    joined = joined.replace("\t", " ").replace("\n", " ")
    return normalized_key(joined)[:KEY_LENGTH_LIMIT]


def scratch_roots():
    """工作根与过渡期旧根（展开成绝对路径、去掉末尾的 /；取法同 .claude/hooks/lib_work_root.py 的 scratch_roots，文件头「键」②）。"""
    configured = os.environ.get(WORK_ROOT_VARIABLE) or DEFAULT_WORK_ROOT
    roots = [os.path.normpath(os.path.abspath(os.path.expanduser(configured))), f"/tmp/claude-{os.getuid()}"]
    return tuple(dict.fromkeys(roots))


def repository_root():
    """本库所在的仓根（本库在 <仓根>/scripts/hosts/ 下）；经 python3 -c 整份送去跑、没有 __file__ 时交空串（那边不取键）。"""
    library = globals().get("__file__")
    if not library:
        return ""
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(library))))


def without_thread_wrapper(text):
    """文件头「键」①：剥掉 capped.sh N（连同紧挨在前面的 shell 名）。按空格切词、按空格接回（切开再接回逐字不变）。"""
    if "capped-kept" in _break_modes or THREAD_WRAPPER_NAME not in text:
        return text
    words = text.split(" ")
    index = 0
    while index < len(words):
        word = words[index]
        if (os.path.basename(word) == THREAD_WRAPPER_NAME and index + 2 < len(words)
                and words[index + 1] and not words[index + 1].startswith("-")):
            start = index - 1 if index > 0 and words[index - 1] in SHELL_NAMES else index
            del words[start:index + 2]
            index = start
            continue
        index += 1
    return " ".join(words)


def path_replacements():
    """[(根的字面, 编译好的正则, 记号)]，按换的次序（工作根与旧根、仓根、家目录）：按此刻的 SINGLEFS_WORK_ROOT、HOME 建一次，取值不变就复用
    （读一张几万行的表时每一行都要换，每行现算正则与 realpath 要多花十几倍的时间）。"""
    cache_key = (os.environ.get(WORK_ROOT_VARIABLE), os.environ.get("HOME"), "scratch-paths-kept" in _break_modes)
    cached = _path_replacements_cache.get(cache_key)
    if cached is not None:
        return cached
    replacements = []
    if "scratch-paths-kept" not in _break_modes:
        for root in scratch_roots():
            escaped = re.escape(root)
            replacements.append((root, re.compile(PATH_START + rf"{escaped}(?:/[^\s/]+)+?(?=/target(?:/|\s|$))|" + PATH_START + rf"{escaped}/[^\s/]+|"
                                                  + PATH_START + rf"{escaped}(?=\s|$)"), SCRATCH_TOKEN))
    for root, token in ((repository_root(), REPOSITORY_TOKEN), (os.path.normpath(os.path.expanduser("~")), HOME_TOKEN)):
        if root and root != "/":
            replacements.append((root, re.compile(PATH_START + re.escape(root) + r"(?=/|\s|$)"), token))
    _path_replacements_cache.clear()
    _path_replacements_cache[cache_key] = replacements
    return replacements


def with_local_paths_replaced(text, replacements=None):
    """文件头「键」②：工作根与旧根下那一段、仓根、家目录换成固定记号。replacements 是 path_replacements() 交的（读整张表时建一次传进来）。"""
    for literal, pattern, token in (path_replacements() if replacements is None else replacements):
        if literal in text:
            text = pattern.sub(token, text)
    return text


def normalized_key(text, replacements=None):
    """键的换算（文件头「键」①②③）：取键与读表都经它；幂等。"""
    return with_local_paths_replaced(without_thread_wrapper(text), replacements)


def stored_key(text, replacements=None):
    """表里一行的键换算之后是什么：(换算后截到 KEY_LENGTH_LIMIT 的键, 是不是被截过又被换算缩短了的旧键)。"""
    if "stored-keys-raw" in _break_modes:
        return text, False
    key = normalized_key(text, replacements)[:KEY_LENGTH_LIMIT]
    return key, len(text) >= KEY_LENGTH_LIMIT and key != text


def size_bytes(text):
    """上限写法（正整数加 K / M / G / T）→ 字节；认不出交 None。"""
    if len(text or "") < 2 or text[-1] not in SIZE_UNITS or not text[:-1].isdigit():
        return None
    return int(text[:-1]) * SIZE_UNITS[text[-1]]


def human(byte_count):
    if byte_count >= 1024 ** 3:
        return f"{byte_count / 1024 ** 3:.1f} GiB"
    return f"{byte_count / 1024 ** 2:.0f} MiB"


def default_peak_table():
    """本机的 ${XDG_DATA_HOME:-~/.local/share}/singlefs/memory-peaks.tsv：仓外的运行状态，worktree 与第二台的副本里跑的都读写本机那一份。"""
    return os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share"), "singlefs", "memory-peaks.tsv")


def parse_recent(text):
    """第六列「近几次」→ [(峰值字节, 撞没撞顶)]（新的在前）；写坏的（空、某一次不是「<字节>:hit|clean」）交 None，调用方当没有这一列。"""
    recent = []
    for item in (text or "").split(","):
        peak_text, separator, flag = item.partition(":")
        if not separator or not peak_text.isdigit() or flag not in (RECENT_HIT, RECENT_CLEAN):
            return None
        recent.append((int(peak_text), flag == RECENT_HIT))
    return recent[:RECENT_MEASUREMENTS] or None


def format_recent(recent):
    return ",".join(f"{peak}:{RECENT_HIT if hit else RECENT_CLEAN}" for peak, hit in recent[:RECENT_MEASUREMENTS])


class PeakRows(dict):
    """read_peak_rows 交回的表（键 → 五段）：按键取（get、[]、in）取不到时，退到「被截过又被换算缩短了的旧键」里是这个键开头一截的那几行，
    几行并成一行交回（文件头「旧键」）；往表里写一个键时删掉被它接上的那几行旧键。别的用法与 dict 一样。"""

    def __init__(self):
        super().__init__()
        self.truncated_keys = set()

    def truncated_matches(self, key):
        if "truncated-keys-unmatched" in _break_modes or not isinstance(key, str):
            return []
        return sorted(stored for stored in self.truncated_keys if stored != key and key.startswith(stored) and stored in self.keys())

    def __missing__(self, key):
        matches = self.truncated_matches(key)
        if not matches:
            raise KeyError(key)
        return merged_peak_row([dict.__getitem__(self, stored) for stored in matches])

    def get(self, key, default=None):
        try:
            return self[key]
        except KeyError:
            return default

    def __contains__(self, key):
        return dict.__contains__(self, key) or bool(self.truncated_matches(key))

    def __setitem__(self, key, value):
        for stored in self.truncated_matches(key):
            dict.__delitem__(self, stored)
            self.truncated_keys.discard(stored)
        dict.__setitem__(self, key, value)


def merged_peak_row(rows):
    """换算后落到同一个键的几行并成一行（文件头「旧键」）：最近一次实测取量的时刻最晚的那一行，近几次按各行时刻从新到旧接起来、至多 RECENT_MEASUREMENTS 次。
    只有一行时原样交回（没动到的四列、五列行写回时照旧是四列、五列）。"""
    if len(rows) == 1:
        return rows[0]
    ordered = sorted(rows, key=lambda row: row[1], reverse=True)
    recent = [measurement for row in ordered for measurement in recent_of(row)][:RECENT_MEASUREMENTS]
    newest = ordered[0]
    return (newest[0], newest[1], newest[2], newest[3], recent)


def read_peak_rows(path):
    """键 → (峰值字节, 量的时刻, 那一次的上限写法, 线程数或 None, 近几次或 None)；表不在交空表。前三列是最近一次实测。
    四列（加线程数之前记的）、五列（加近几次之前记的）与六列的行都认；没有第六列（或它写坏了）的近几次是 None，读近几次用 recent_of。
    每一行的键照 normalized_key 换算，落到同一个键的几行并成一行；交回的是 PeakRows（文件头「旧键」）。"""
    rows = PeakRows()
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return rows
    grouped, untruncated = {}, {}
    replacements = path_replacements()
    for line in lines:
        fields = line.split("\t")
        if line.startswith("#") or len(fields) not in (4, 5, 6) or not fields[0].isdigit():
            continue
        key, truncated = stored_key(fields[3], replacements)
        grouped.setdefault(key, []).append((int(fields[0]), fields[1], fields[2], fields[4] if len(fields) >= 5 else None,
                                            parse_recent(fields[5]) if len(fields) == 6 else None))
        untruncated.setdefault(key, False)
        untruncated[key] = untruncated[key] or not truncated
    for key, group in grouped.items():
        dict.__setitem__(rows, key, merged_peak_row(group))
        if not untruncated[key]:
            rows.truncated_keys.add(key)   # 落到这个键的每一行都是被截过又被换算缩短了的旧键：它只是新键的开头一截
    return rows


def read_peak_table(path):
    """键 → (峰值字节, 量的时刻, 那一次的上限写法, 线程数或 None)：read_peak_rows 去掉近几次。只认四段的旧包装还在跑时读的是这一份。"""
    return {key: row[:4] for key, row in read_peak_rows(path).items()}


def recent_of(row):
    """一行的近几次 [(峰值字节, 撞没撞顶)]，新的在前。没有第六列的旧行当成只有一次：那一行的峰值，到了那一次的上限就算撞了顶
    （撞了自己的上限、或按峰值表放进来又被挤掉，都记成上限，真峰值不知道、只知道不小于它）。"""
    if row[4] is not None:
        return row[4]
    recorded_cap = size_bytes(row[2])
    return [(row[0], recorded_cap is not None and row[0] >= recorded_cap)]


def recorded_peak(path, key, latest_only=False):
    """(预测的峰值字节, 撞过顶没有)；表里没有这条交 (None, False)。预测取近几次里最大的，近几次里有一次撞顶就算撞过顶（滞回：
    撞顶之后要连着 RECENT_MEASUREMENTS 次干净跑完才回到按峰值记账；峰值一路变小时记账不跟着变小）。
    latest_only 只看最近一次（只给包装与派发器的弄坏开关用：改前的判法）。"""
    row = read_peak_rows(path).get(key)
    if row is None:
        return None, False
    recent = recent_of(row)[:1] if latest_only else recent_of(row)
    return max(peak for peak, _ in recent), any(hit for _, hit in recent)


def default_duration_table(environment):
    """用时表在哪（文件头「用时表」）：HOST_DISPATCH_DURATIONS 给了就用它，不给在 ${XDG_STATE_HOME:-~/.local/state}/singlefs/ 下。"""
    state_home = environment.get("XDG_STATE_HOME") or os.path.join(environment.get("HOME") or "/", ".local", "state")
    return environment.get("HOST_DISPATCH_DURATIONS") or os.path.join(state_home, "singlefs", "run-durations.tsv")


def read_duration_table(path):
    """键 → (用时秒数, 量的时刻, 退出码, 跑在哪台)；path 是空串或表不在交空表。键照峰值表换算（文件头「旧键」）。"""
    rows = {}
    if not path:
        return rows
    replacements = path_replacements()
    with contextlib.suppress(FileNotFoundError):
        with open(path, encoding="utf-8") as handle:
            for line in handle.read().splitlines():
                fields = line.split("\t")
                if line.startswith("#") or len(fields) != 5 or not fields[0].isdigit():
                    continue
                key = stored_key(fields[4], replacements)[0]   # 旧键照峰值表换算（文件头「旧键」）；落到同一个键的几行只留时刻最晚的那一行
                if key not in rows or fields[1] >= rows[key][1]:
                    rows[key] = (int(fields[0]), fields[1], fields[2], fields[3])
    return rows


def recorded_duration(path, key):
    """(用时秒数, 量的那一天)；表里没有这条交 (None, None)。"""
    row = read_duration_table(path).get(key)
    return (None, None) if row is None else (row[0], row[1][:10])


def record_duration(path, key, seconds, exit_code, host, measured_at):
    """用时表里这个键换成这一次的（同一把锁里读、改、先写临时文件再改名）；path 是空串时不记。"""
    if not path:
        return
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path + ".lock", "a", encoding="utf-8") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        rows = read_duration_table(path)
        rows[key] = (int(seconds), measured_at, str(exit_code), host)
        lines = DURATION_TABLE_HEADER + ["\t".join([str(value[0]), value[1], value[2], value[3], row_key]) for row_key, value in sorted(rows.items())]
        temporary = f"{path}.{os.getpid()}.partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")
        os.replace(temporary, path)


def unreclaimable_bytes(cgroup_directory):
    """cgroup 里回收不掉的用量：memory.current 减 active_file 与 inactive_file（页缓存回收得掉）。
    目录不在（scope 还没起，或已经收尾删了）交 0；目录在而文件读不了交 None。"""
    try:
        with open(os.path.join(cgroup_directory, "memory.current"), encoding="utf-8") as handle:
            current = int(handle.read())
        file_pages = 0
        with open(os.path.join(cgroup_directory, "memory.stat"), encoding="utf-8") as handle:
            for line in handle:
                name, value = line.split()
                if name in ("active_file", "inactive_file"):
                    file_pages += int(value)
    except FileNotFoundError:
        return None if os.path.isdir(cgroup_directory) else 0
    except (OSError, ValueError):
        return None
    return max(0, current - file_pages)


def resident_bytes(slice_directory, slice_used):
    """slice 里不属于任何 scope 的常驻用量：slice 回收不掉的用量 slice_used 减去它底下每个子 cgroup（scope）回收不掉的用量之和，不小于 0。
    账上在跑的与别的 scope 都跑完、收尾之后，这一截也不会少（scope 退出后挂回 slice 的 shmem、内核对象之类）；包装用它判「等多久都放不下」
    （run-with-memory-cap.sh 文件头「排队」一段）。slice 目录读不了就当没有子 cgroup、全算常驻（偏多不偏少）。"""
    try:
        names = os.listdir(slice_directory)
    except OSError:
        return slice_used
    children = 0
    for name in names:
        child = os.path.join(slice_directory, name)
        if os.path.isdir(child):
            children += unreclaimable_bytes(child) or 0
    return max(0, slice_used - children)


def process_start_time(process_id):
    """/proc/<pid>/stat 的第 22 段（进程起始时刻，开机以来的时钟滴答）：进程号被复用时它不同。进程不在交 None。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8") as handle:
            return handle.read().rsplit(")", 1)[1].split()[19]
    except (OSError, IndexError):
        return None


def ledger_entry_is_dead(entry):
    """账上这一笔的包装已经不在：此刻读不到它的起始时刻、账上记的起始时刻是空的（包装不在时放行记下的），或两者对不上（进程号被复用）。
    两边都读不到不算对得上：None 比 None 相等，这一笔会一直当活的。"""
    recorded_start_time = entry.get("pid_start_time")
    current_start_time = process_start_time(entry["pid"])
    return recorded_start_time is None or current_start_time is None or current_start_time != recorded_start_time


def live_ledger_entries(ledger_directory, remove_dead=True, keep_dead=False):
    """账上还在跑的每一笔。remove_dead：包装已经死了的那一笔删掉（要在包装的锁里调；派发器只读、不删）。
    keep_dead：死了的也当活的（只给包装的弄坏开关 keepstale 用）。账的目录不在交空。"""
    entries = []
    try:
        names = sorted(os.listdir(ledger_directory))
    except FileNotFoundError:
        return entries
    for name in names:
        if not name.endswith(".json"):
            continue
        path = os.path.join(ledger_directory, name)
        try:
            with open(path, encoding="utf-8") as handle:
                entry = json.load(handle)
        except (OSError, ValueError):
            continue   # 刚删掉的；写的时候先写临时文件再改名，读不到半截
        if not keep_dead and ledger_entry_is_dead(entry):
            if remove_dead:
                with contextlib.suppress(FileNotFoundError):
                    os.remove(path)
            continue
        entries.append(entry)
    return entries


def pending_bytes(entries, slice_directory):
    """账上每条放行了还在跑的 max(0, 它要的量 − 它的 scope 此刻回收不掉的用量) 之和：刚起跑的还没涨到量的那一截。"""
    return sum(max(0, entry["need"] - (unreclaimable_bytes(os.path.join(slice_directory, entry["unit"] + ".scope")) or 0)
                   if slice_directory else entry["need"]) for entry in entries)


def meminfo(path="/proc/meminfo"):
    values = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            name, rest = line.split(":", 1)
            values[name] = int(rest.split()[0]) * 1024
    return values


def default_state_directory(environment):
    """包装的锁与账在哪：与 run-with-memory-cap.sh 的 STATE_DIRECTORY 同一个取法。"""
    return environment.get("RUN_WITH_MEMORY_CAP_STATE_DIR") or os.path.join(
        environment.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}", "singlefs-heavy-admission")


def slice_directory_of(slice_name, cgroup_root):
    """slice 的 cgroup 目录；slice 还没起、或查不了用户级 systemd 交空串。"""
    try:
        cgroup_path = subprocess.run(["systemctl", "--user", "show", "-p", "ControlGroup", "--value", slice_name],
                                     capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=20).stdout.strip()
    except (OSError, subprocess.TimeoutExpired):
        return ""
    return cgroup_root.rstrip("/") + cgroup_path if cgroup_path else ""


def headroom(environment):
    """这台此刻的内存余量（派发器选机器用；第二台上由派发器把这份源码送过去跑）。读的环境变量与包装的同名：
    RUN_WITH_MEMORY_CAP_STATE_DIR、RUN_WITH_MEMORY_CAP_SLICE、RUN_WITH_MEMORY_CAP_CGROUP_ROOT、RUN_WITH_MEMORY_CAP_RESERVE；
    SINGLEFS_DISPATCH_PROBE_MEMINFO 只给派发器的自证换掉 /proc/meminfo。交一个 dict（字节）：
      mem_total、mem_available        /proc/meminfo
      pending、running                包装账上在跑的条数与它们还没涨到量的那一截之和
      slice_total、slice_used         slice 的总上限（slice 在就读它的 memory.max，不在按整机内存减余量算）与 slice 里回收不掉的用量
      free                            mem_available − pending：此刻整机真放得下多少
      slice_room                      slice_total − slice_used − pending：包装排队那一步此刻放得下多少"""
    memory = meminfo(environment.get("SINGLEFS_DISPATCH_PROBE_MEMINFO") or "/proc/meminfo")
    cgroup_root = environment.get("RUN_WITH_MEMORY_CAP_CGROUP_ROOT") or "/sys/fs/cgroup"
    slice_directory = slice_directory_of(environment.get("RUN_WITH_MEMORY_CAP_SLICE") or DEFAULT_SLICE, cgroup_root)
    state_directory = default_state_directory(environment)
    entries = live_ledger_entries(os.path.join(state_directory, "ledger"), remove_dead=False)
    pending = pending_bytes(entries, slice_directory)
    slice_total = None
    if slice_directory:
        with contextlib.suppress(OSError, ValueError):
            with open(os.path.join(slice_directory, "memory.max"), encoding="utf-8") as handle:
                slice_total = int(handle.read())
    if slice_total is None:
        reserve = size_bytes(environment.get("RUN_WITH_MEMORY_CAP_RESERVE") or DEFAULT_RESERVE_TEXT) or 0
        slice_total = max(0, (memory["MemTotal"] - reserve) // MEBIBYTE * MEBIBYTE)
    slice_used = (unreclaimable_bytes(slice_directory) or 0) if slice_directory else 0
    return {"mem_total": memory["MemTotal"], "mem_available": memory["MemAvailable"], "pending": pending, "running": len(entries),
            "slice_total": slice_total, "slice_used": slice_used, "free": memory["MemAvailable"] - pending,
            "slice_room": slice_total - slice_used - pending}


# ── 自证（python3 scripts/hosts/memory_peaks_lib.py --selftest）：只调本库的函数、在临时目录里写假峰值表与假用时表，不起子进程 ──
# 弄坏开关（逗号连几个；只在自证里读进来，import 这份库的包装、派发器、看门狗一律不认；平常跑设了它退 2 并点名它只在自证里认，见 refuse_break_outside_selftest；
# 认不出的值退 2）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（「」里是失败行原文；末尾一句「✗ memory_peaks_lib.py 自证 N 格里 M 格不对」是汇总）：
#   MEMORY_PEAKS_LIB_BREAK=scratch-paths-kept  取键不换工作根与旧根下那一段（改前的样子）
#       「✗ 自证格不对：两份副本路径下同一条命令落一个键」
#       「✗ 自证格不对：经 capped.sh 与不经落一个键」（那一格里连副本路径的那一例要落到〈工作根下的目录〉，开关下没换）
#       「✗ 自证格不对：旧键读得到：换算后落到同一个键的几行并成一行」（样本旧键带副本路径）
#   MEMORY_PEAKS_LIB_BREAK=capped-kept  取键不剥 capped.sh N（改前的样子）
#       「✗ 自证格不对：经 capped.sh 与不经落一个键」
#       「✗ 自证格不对：旧键读得到：换算后落到同一个键的几行并成一行」（样本旧键带 capped.sh）
#   MEMORY_PEAKS_LIB_BREAK=stored-keys-raw  读表不换算表里的键（改前的样子：旧键读不到）
#       「✗ 自证格不对：旧键读得到：换算后落到同一个键的几行并成一行」
#       「✗ 自证格不对：被截过又被换算缩短了的旧键读得到，写回时被新键接上」
#   MEMORY_PEAKS_LIB_BREAK=truncated-keys-unmatched  PeakRows 不退到被截过的旧键
#       「✗ 自证格不对：被截过又被换算缩短了的旧键读得到，写回时被新键接上」
#   MEMORY_PEAKS_LIB_BREAK=outside-selftest-unnamed  refuse_break_outside_selftest 什么都不交（改前的样子：平常跑设了开关只打「单独跑只认 --selftest」，不点名变量）
#       「✗ 自证格不对：平常跑设了弄坏开关退 2 并点名 MEMORY_PEAKS_LIB_BREAK 只在自证里认」
def self_test_cells(work):
    """交回 [(格名, 对不对, 说明)]。work 是临时目录。"""
    cells = []
    work_root, legacy_root = scratch_roots()[0], scratch_roots()[-1]
    repository = repository_root()
    home = os.path.normpath(os.path.expanduser("~"))

    copies = [[f"{work_root}/task-alpha/tree/target/debug/deps/journal_ring_fields-0123456789abcdef", "--exact", "ring"],
              [f"{work_root}/task-beta/copy/nested/target/debug/deps/journal_ring_fields-fedcba9876543210", "--exact", "ring"],
              [f"{legacy_root}/-home-sample/session/scratchpad/rerun/target/debug/deps/journal_ring_fields-00112233445566aa", "--exact", "ring"]]
    copy_keys = [command_key(words) for words in copies]
    no_target = [command_key(["python3", "-I", "tools/numbers_model.py", f"{work_root}/{name}", "j2_q"]) for name in ("task-alpha", "task-beta")]
    other = command_key([f"{work_root}/task-alpha/tree/target/debug/deps/other_test_name-0123456789abcdef", "--exact", "ring"])
    wanted = f"{SCRATCH_TOKEN}/target/debug/deps/journal_ring_fields --exact ring"
    local_paths = command_key([f"{home}/.cargo/bin/cargo", "build", f"--manifest-path={repository}/Cargo.toml"])
    wanted_local = f"{HOME_TOKEN}/.cargo/bin/cargo build --manifest-path={REPOSITORY_TOKEN}/Cargo.toml"
    cells.append(("两份副本路径下同一条命令落一个键",
                  len(set(copy_keys)) == 1 and copy_keys[0] == wanted and len(set(no_target)) == 1 and other != copy_keys[0]
                  and work_root not in copy_keys[0] + no_target[0] and local_paths == wanted_local,
                  f"三份副本的键 {copy_keys}；没有 target 的两条 {no_target}；换了测试名的 {other}；要的 {wanted}；家目录与仓根 {local_paths}（要的 {wanted_local}）"))

    plain = ["cargo", "test", "-p", "singlefs-core"]
    wrapped = [["bash", "scripts/hosts/capped.sh", "31"] + plain, ["bash", f"{repository}/scripts/hosts/capped.sh", "default"] + plain,
               ["scripts/hosts/capped.sh", "4"] + plain]
    prefixed = ["env", "A=1", "nice", "-n", "19", "bash", f"{repository}/scripts/hosts/capped.sh", "4", "cargo", "build"]
    copied = ["bash", f"{repository}/scripts/hosts/capped.sh", "6", f"{work_root}/probe-one/target/release/probe", "cm", "384"]
    lone = ["bash", "scripts/hosts/capped.sh", "31"]
    wrapped_keys = [command_key(words) for words in wrapped]
    cells.append(("经 capped.sh 与不经落一个键",
                  set(wrapped_keys) == {command_key(plain)} and command_key(prefixed) == command_key(["env", "A=1", "nice", "-n", "19", "cargo", "build"])
                  and command_key(copied) == command_key([f"{work_root}/probe-two/target/release/probe", "cm", "384"])
                  and command_key(lone) == " ".join(lone),
                  f"经 capped.sh 的 {wrapped_keys}，不经的 {command_key(plain)}；带前缀的 {command_key(prefixed)}；连副本路径的 {command_key(copied)}；"
                  f"capped.sh 后面没有命令的 {command_key(lone)}（照原样）"))

    table = os.path.join(work, "memory-peaks.tsv")
    gib = 1024 ** 3
    old_newer = f"bash {repository}/scripts/hosts/capped.sh 31 {work_root}/old-copy/tree/target/release/deps/scan_glue_memory"
    old_older = f"bash scripts/hosts/capped.sh 12 {work_root}/other-copy/target/release/deps/scan_glue_memory"
    truncated_full = f"{work_root}/long-copy/tree/target/release/deps/" + "long_test_name_" * 30 + " --exact case"
    truncated_raw = truncated_full[:KEY_LENGTH_LIMIT]
    with open(table, "w", encoding="utf-8") as handle:
        handle.write("# 自证造的峰值表（换算之前的旧键）\n")
        handle.write(f"{5 * gib}\t2026-10-20\t8G\t{old_newer}\t31\t{5 * gib}:clean\n")
        handle.write(f"{9 * gib}\t2026-10-10\t9G\t{old_older}\t12\n")
        handle.write(f"{3 * gib}\t2026-10-01\t8G\t{truncated_raw}\n")
    fresh_key = command_key([f"{work_root}/fresh-copy/target/release/deps/scan_glue_memory-0123456789abcdef"])
    peak, hit = recorded_peak(table, fresh_key)
    rows = read_peak_rows(table)
    durations = os.path.join(work, "run-durations.tsv")
    with open(durations, "w", encoding="utf-8") as handle:
        handle.write(f"9000\t2026-10-20\t0\tlocal\t{old_newer}\n")
    seconds, _ = recorded_duration(durations, fresh_key)
    idempotent = all(normalized_key(key) == key for key in copy_keys + no_target + wrapped_keys + [fresh_key, other])
    cells.append(("旧键读得到：换算后落到同一个键的几行并成一行",
                  peak == 9 * gib and hit and fresh_key in rows and len(rows.get(fresh_key)[4] or []) == 2 and seconds == 9000 and idempotent,
                  f"新键 {fresh_key}：近几次里最大 {peak}、撞过顶 {hit}；表里的键 {sorted(rows)}；用时表读到 {seconds}；换算幂等 {idempotent}"))

    truncated_query = command_key(truncated_full.split(" "))
    found = recorded_peak(table, truncated_query)[0]
    rows = read_peak_rows(table)
    old = rows.get(truncated_query)
    rows[truncated_query] = (4 * gib, "2026-10-25", "8G", None, [(4 * gib, False)] + (recent_of(old) if old else []))
    leftover = [key for key in rows if key != truncated_query and truncated_query.startswith(key)]
    cells.append(("被截过又被换算缩短了的旧键读得到，写回时被新键接上",
                  found == 3 * gib and old is not None and not leftover and rows[truncated_query][4] == [(4 * gib, False), (3 * gib, False)],
                  f"新键（{len(truncated_query)} 个字）读到 {found}；写回之后还剩的开头一截旧键 {leftover}；新键的近几次 {rows[truncated_query][4]}"))

    roots_note = "没读 .claude/hooks/lib_work_root.py"
    same_roots = False
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("lib_work_root_for_memory_peaks", os.path.join(repository, ".claude", "hooks", "lib_work_root.py"))
        work_root_library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(work_root_library)
        same_roots = tuple(work_root_library.scratch_roots()) == scratch_roots()
        roots_note = f"lib_work_root 的 {tuple(work_root_library.scratch_roots())}，这里的 {scratch_roots()}"
    except (OSError, ImportError, AttributeError, ValueError) as error:
        roots_note = f"读不了 lib_work_root.py：{type(error).__name__}: {error}"
    cells.append(("工作根与旧根的取法与 lib_work_root.py 的 scratch_roots 一样", same_roots, roots_note))

    probe_arguments = ["--force", "--break-guard-probe-unknown-option"]
    refused = refuse_break_outside_selftest(probe_arguments, {BREAK_VARIABLE: "capped-kept"})
    named = [line for line in refused if BREAK_VARIABLE in line and "自证" in line]
    passed_plain = refuse_break_outside_selftest(probe_arguments, {}) == []
    passed_selftest = refuse_break_outside_selftest(["--selftest"], {BREAK_VARIABLE: "capped-kept"}) == []
    cells.append((f"平常跑设了弄坏开关退 2 并点名 {BREAK_VARIABLE} 只在自证里认",
                  bool(named) and passed_plain and passed_selftest,
                  f"设了开关、参数 {probe_arguments} 时交回 {refused}；没设开关时放行 {passed_plain}；参数正好 --selftest 时放行 {passed_selftest}"))
    return cells


def refuse_break_outside_selftest(arguments, environment):
    """平常跑（参数不是正好 --selftest）而环境里设了 MEMORY_PEAKS_LIB_BREAK：交回拒绝的那几行（点名变量、说它只在自证里认）；放行交回空表。
    自证里拿假环境在进程里调它（不起子进程）；弄坏开关 outside-selftest-unnamed 只经 _break_modes 读，平常跑时 _break_modes 是空的。"""
    value = environment.get(BREAK_VARIABLE, "")
    if arguments == ["--selftest"] or not value or "outside-selftest-unnamed" in _break_modes:
        return []
    return [f"  ✗ {BREAK_VARIABLE}={value} 只在自证（python3 scripts/hosts/memory_peaks_lib.py --selftest）里认，平常跑设了它拒跑",
            f"  → 怎么办：unset {BREAK_VARIABLE} 再跑；要看开关弄红哪几格，设着它跑 --selftest（开关表在文件末尾「自证」那一段）"]


def run_self_test():
    global _break_modes
    import tempfile
    modes = {mode for mode in os.environ.get("MEMORY_PEAKS_LIB_BREAK", "").split(",") if mode}
    unknown = modes - set(KNOWN_BREAK_MODES)
    if unknown:
        print(f"  ✗ MEMORY_PEAKS_LIB_BREAK 认不出 {'、'.join(sorted(unknown))}（认得的：{'、'.join(KNOWN_BREAK_MODES)}）")
        print("  → 怎么办：去掉它，或只写认得的项")
        return 2
    _break_modes = frozenset(modes)
    with tempfile.TemporaryDirectory(prefix="memory-peaks-lib-selftest.") as work:
        cells = self_test_cells(work)
    failed = 0
    for name, ok, note in cells:
        if ok:
            print(f"  · 对：{name}")
            continue
        failed += 1
        print(f"  ✗ 自证格不对：{name}")  # gate-lint:detail
        print(f"       | {note}")
    if failed:
        print(f"  ✗ memory_peaks_lib.py 自证 {len(cells)} 格里 {failed} 格不对（逐格在上面）")
        print("  → 怎么办：按不对的那一格修 command_key / normalized_key / read_peak_rows / PeakRows；MEMORY_PEAKS_LIB_BREAK 设着的话本来就该红")
        return 1
    print(f"  ✓ memory_peaks_lib.py 自证 {len(cells)} 格都对（副本与草稿目录路径换成记号、剥 capped.sh N、旧键换算后并成一行读得到、被截过的旧键读得到并在写回时被接上、工作根取法与 lib_work_root.py 一样、平常跑设了弄坏开关退 2 并点名变量）")
    return 0


if __name__ == "__main__" and globals().get("__file__"):
    if sys.argv[1:] != ["--selftest"]:
        for refusal in refuse_break_outside_selftest(sys.argv[1:], os.environ):
            print(refusal, file=sys.stderr)
        print("  ✗ memory_peaks_lib.py 是被 import 的库，单独跑只认 --selftest", file=sys.stderr)
        print("  → 怎么办：python3 scripts/hosts/memory_peaks_lib.py --selftest；取键、读表 import 它（文件头「用法」）", file=sys.stderr)
        sys.exit(2)
    sys.exit(run_self_test())
