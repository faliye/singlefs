"""内存峰值表、取键与「这台此刻还放得下多少」的共用库：scripts/hosts/run-with-memory-cap.sh（内存包装）与
scripts/hosts/host-dispatch.py（统一派发器）都 import 它，取键、读峰值表、读账、算余量只在这一份里写。

被 import 的库，不单独跑、不写准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md「不判的三类」）；
import 本身不起子进程、不开文件。用法（脚本按路径跑时 sys.path[0] 就是这个目录；bash 里写 python3 -B -c 'import sys; sys.path.insert(0, <目录>); …'）：
    import memory_peaks_lib as peaks
    key = peaks.command_key(命令的各个词, 环境里的 RUN_WITH_MEMORY_CAP_KEY)
派发器把这份源码整份送到第二台跑（python3 -c），在那边算那台的余量：所以这里只用标准库，不 import 同目录的别的文件。

键（与峰值表第四列相同）：给了 RUN_WITH_MEMORY_CAP_KEY 就用它；不给就把命令的各个词用空格接起来，去掉测试二进制名里 -<16 位十六进制> 的哈希；
  两种都把制表符、换行换成空格，截到 KEY_LENGTH_LIMIT 个字。
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

MEBIBYTE = 1024 * 1024
KEY_LENGTH_LIMIT = 300
TEST_BINARY_HASH = re.compile(r"-[0-9a-f]{16}\b")
SIZE_UNITS = {"K": 1024, "M": MEBIBYTE, "G": 1024 * MEBIBYTE, "T": MEBIBYTE * MEBIBYTE}
# 峰值表里查不到这条命令时按多少算（包装的 auto、派发器的预测都用它）：取 scripts/experiments/replay.sh 的 REPLAY_MEMORY_CAP 默认值，
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
    return joined[:KEY_LENGTH_LIMIT]


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


def read_peak_rows(path):
    """键 → (峰值字节, 量的时刻, 那一次的上限写法, 线程数或 None, 近几次或 None)；表不在交空表。前三列是最近一次实测。
    四列（加线程数之前记的）、五列（加近几次之前记的）与六列的行都认；没有第六列（或它写坏了）的近几次是 None，读近几次用 recent_of。"""
    rows = {}
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return rows
    for line in lines:
        fields = line.split("\t")
        if line.startswith("#") or len(fields) not in (4, 5, 6) or not fields[0].isdigit():
            continue
        rows[fields[3]] = (int(fields[0]), fields[1], fields[2], fields[4] if len(fields) >= 5 else None,
                           parse_recent(fields[5]) if len(fields) == 6 else None)
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
    """键 → (用时秒数, 量的时刻, 退出码, 跑在哪台)；path 是空串或表不在交空表。"""
    rows = {}
    if not path:
        return rows
    with contextlib.suppress(FileNotFoundError):
        with open(path, encoding="utf-8") as handle:
            for line in handle.read().splitlines():
                fields = line.split("\t")
                if line.startswith("#") or len(fields) != 5 or not fields[0].isdigit():
                    continue
                rows[fields[4]] = (int(fields[0]), fields[1], fields[2], fields[3])
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
