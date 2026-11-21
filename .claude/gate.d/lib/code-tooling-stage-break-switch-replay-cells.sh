#!/usr/bin/env bash
# 门禁 code-tooling 的格函数库：stage-break-switch-replay、break-switch-outside-selftest。
# 由 .claude/gate.d/code-tooling.sh 在参数解析与 cd 到项目根之后 source，不单独跑；格名表、stage_cell 登记、参数解析与汇总在那份入口里。
# 这份里是格函数与它们独用的辅助（python 判法库变量、表、常量）连同判据注释；读的 ROOT、STAGE_NAME、STAGE_DIRECTORY、STAGE_REPOSITORY 与
# 各格共用的全局变量由入口设好，别份库里的函数与变量在调用时才取。

# 开关复跑两格（stage-break-switch-replay、break-switch-replay）的进度间隔：300 秒（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟）；
# 样本根（judged_root_is_sample）里那一格的标记文件（一个正整数秒数）顶替它，样本不必等满 5 分钟。写的不是正整数：打拒绝与出路两行（调用方原样转出）、返回 1，一条都不跑。
switch_replay_progress_seconds() {
  local marker="$1" seconds=300
  if judged_root_is_sample && [[ -f "$marker" ]]; then
    seconds="$(<"$marker")"
    if [[ ! "$seconds" =~ ^[1-9][0-9]*$ ]]; then
      echo "  ✗ 样本根里的 $marker 不是一个正整数秒数，一条都没跑"
      echo "     → 怎么办：那份文件只写一个正整数（进度间隔的秒数）。"
      return 1
    fi
  fi
  printf '%s\n' "$seconds"
}

# 开关复跑两格共用的逐条进度（两格文件头「进度」；不各写一份）。装法照 BREAK_SWITCH_TABLE_LIBRARY：
#   progress_library = types.ModuleType("lib_switch_replay_progress"); exec(compile(<这一段>, "switch-replay-progress-library", "exec"), progress_library.__dict__)
# 只打行、只读 /proc 与本进程的 cgroup 记账，不碰判定：判对没有、收回几条照各格自己的判法。
SWITCH_REPLAY_PROGRESS_LIBRARY="$(cat <<'PY_SWITCH_REPLAY_PROGRESS'
import collections, contextlib, itertools, os, threading, time

MEMORY_SAMPLE_SECONDS = 5      # 报进度的线程每过这么多秒（进度间隔更短时照进度间隔）采一次后代进程的内存，峰值只从这些采样里取
LONGEST_RUNNING_SHOWN = 3      # 每次报总进度列出跑得最久的几趟
MACHINE_PROGRESS_PREFIX = "TASK_SERVER_PROGRESS"      # 行首顶格的机读进度行的头（done=<跑完几趟> total=<要跑几趟>；读日志的人与脚本认它，统一任务服务不读）
KIB_PER_MIB = 1024


def mib(kib):
    return "读不到" if kib is None else f"{kib / KIB_PER_MIB:.0f} MiB"


def proportional_kib(pid):
    """→ 这个进程的 Pss（KiB：共享页按共用它的进程数摊开，各进程相加不重算）；smaps_rollup 读不到退回 VmRSS；进程已经退了 → None。"""
    for path, field in ((f"/proc/{pid}/smaps_rollup", "Pss:"), (f"/proc/{pid}/status", "VmRSS:")):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                for row in handle:
                    if row.startswith(field):
                        return int(row.split()[1])
        except (OSError, ValueError, IndexError):
            continue
    return None


def command_words(pid):
    """→ 这个进程命令行的前三个词（说明峰值是谁占的）；读不到写「（读不到）」。"""
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            words = [word.decode("utf-8", "replace") for word in handle.read().split(b"\0") if word]
    except OSError:
        words = []
    return " ".join(words[:3])[:80] or "（读不到）"


def children_map():
    """→ {父进程号: [子进程号…]}：扫一遍 /proc/<进程号>/stat 的父进程号；读的时候已经退了的不算。"""
    children_of = collections.defaultdict(list)
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat", "rb") as handle:
                stat = handle.read()
        except OSError:
            continue
        # 命令名在括号里、自己可以带空格与括号：取最后一个「)」之后的字段，状态之后那一个是父进程号
        fields = stat[stat.rfind(b")") + 1:].split()
        if len(fields) >= 2 and fields[1].isdigit():
            children_of[int(fields[1])].append(int(name))
    return children_of


def subtree_of(children_of, root_pid):
    """→ root_pid 的全部后代进程号（不含它自己），按 children_map 连。"""
    found, pending = [], list(children_of.get(root_pid, []))
    while pending:
        pid = pending.pop()
        found.append(pid)
        pending.extend(children_of.get(pid, []))
    return found


def descendants_of(root_pid):
    """→ root_pid 的全部后代进程号（不含它自己）。"""
    return subtree_of(children_map(), root_pid)


def cgroup_charge_kib():
    """→ 本进程所在 cgroup 的 memory.current（KiB，含页缓存与 tmpfs）；读不到 → None。"""
    try:
        with open("/proc/self/cgroup", encoding="utf-8") as handle:
            relative = next(row.rstrip("\n")[3:] for row in handle if row.startswith("0::"))
        with open(f"/sys/fs/cgroup{relative}/memory.current", encoding="utf-8") as handle:
            return int(handle.read()) // 1024
    except (OSError, StopIteration, ValueError):
        return None


class ReplayProgress:
    """开关复跑的逐条进度。item_done：每条判完打一行「<格名> 第 k / N 条<对上没有>：<哪一条>，用了 S 秒」（k 按判完的先后数）；
    start：打开跑那一行，另起一条线程，每过 interval_seconds 秒打一行总进度、跑得最久的几趟、extra_lines 给的行与一行内存，
    每过 MEMORY_SAMPLE_SECONDS 秒（进度间隔更短时照它）采一次后代进程的内存记峰值；close：停线程，打收尾一行（逐条进度几行、派出去几条、内存峰值）。
    machine_readable：另在开跑时与每跑完一趟时打一行行首顶格的 TASK_SERVER_PROGRESS done=<跑完几趟> total=<要跑几趟>：一趟一报，判法等一份的趟齐了才集中判，
    按判完的条数报，done 是一阵一阵涨、看不出跑到哪了；trips 是开跑时要跑的趟数，trip_finished 每跑完一趟（取回的落盘趟算当场跑完）记一趟，
    trips_added 是派出去之前加上的（要重跑的那一趟），trips_left 是改成「跑完的加还要跑的」（基准趟没过、开关一条没起的那几份扣掉）。
    silent（弄坏开关 replay-progress-silent）：一行都不打、不起线程。行都以「·」起头、不带 ✗（机读行除外，它不带 ✗），打完立即冲出去
    （标准输出接到日志时 python 按块缓冲，不冲就要等整格跑完才落进日志，统一任务服务看日志满 10 分钟没涨就当卡住停掉）。"""

    def __init__(self, cell, total, interval_seconds, silent, extra_lines=None, machine_readable=False, trips=0):
        self.cell, self.total, self.interval_seconds, self.silent = cell, total, interval_seconds, silent
        self.machine_readable = machine_readable
        self.trips_total, self.trips_done = trips, 0   # 机读进度：要跑几趟、跑完几趟
        self.extra_lines = extra_lines or (lambda: [])
        self.sample_seconds = min(interval_seconds, MEMORY_SAMPLE_SECONDS)
        self.lock = threading.Lock()          # 管在跑的趟、计数与峰值；拿着它可以再拿 print_lock，反过来不许
        self.print_lock = threading.Lock()
        self.running_since = {}               # 在跑的一趟（自增号）→ (组, 说明, 起跑时刻)
        self.tokens = itertools.count()
        self.reported = 0                     # 打出去的逐条进度行数
        self.samples = 0
        self.peak = None                      # 后代进程 Pss 合计最大的那一次采样：(KiB, 进程数, cgroup 记账 KiB, 在跑的按组计数)
        self.peak_charge = None               # 采到过的 cgroup 记账最大值（KiB）
        self.largest = (0, "")                # 采到过的单个进程最大 Pss：(KiB, 命令前三个词)
        self.stop = threading.Event()
        self.thread = None

    def say(self, text):
        if self.silent:
            return
        with self.print_lock:
            print(f"  · {text}", flush=True)

    def say_machine(self):
        """机读进度一行，行首顶格（调用方拿着 self.lock 或还没起线程，跑完与要跑的趟数此刻不变）。"""
        if self.silent or not self.machine_readable:
            return
        with self.print_lock:
            print(f"{MACHINE_PROGRESS_PREFIX} done={self.trips_done} total={self.trips_total}", flush=True)

    def trip_finished(self):
        """跑完一趟（取回的落盘趟当场算跑完）：记一趟，打一行机读进度。"""
        with self.lock:
            self.trips_done += 1
            self.say_machine()

    def trips_added(self, count):
        """要多跑 count 趟（派出去之前调）：要跑的趟数加上，打一行机读进度。"""
        with self.lock:
            self.trips_total += count
            self.say_machine()

    def trips_left(self, count):
        """此刻还要跑 count 趟：要跑的趟数改成跑完的加 count，改了才打一行机读进度。"""
        with self.lock:
            if self.trips_total != self.trips_done + count:
                self.trips_total = self.trips_done + count
                self.say_machine()

    @contextlib.contextmanager
    def running(self, group, label):
        """包住起一趟子进程的那几行：跑着时算「在跑」，报总进度时列跑得最久的、采内存时按组记。"""
        token = next(self.tokens)
        with self.lock:
            self.running_since[token] = (group, label, time.monotonic())
        try:
            yield
        finally:
            with self.lock:
                del self.running_since[token]

    def item_done(self, verdict, label, seconds):
        if self.silent:
            return
        with self.lock:
            self.reported += 1
            self.say(f"{self.cell} 第 {self.reported} / {self.total} 条{verdict}：{label}，用了 {seconds:.0f} 秒")

    def start(self, headline):
        if self.silent:
            return
        self.say(headline)
        self.say_machine()
        self.thread = threading.Thread(target=self.beat, daemon=True)
        self.thread.start()

    def beat(self):
        ticks_per_report = max(1, self.interval_seconds // self.sample_seconds)
        tick, report = 0, 0
        while not self.stop.wait(self.sample_seconds):
            tick += 1
            try:
                memory = self.sample_memory()
            except OSError as error:   # 读 /proc 出错只丢这一次采样：报进度的线程一停，日志就不再涨
                memory = f"这一次没采成（{error}）"
            if tick % ticks_per_report:
                continue
            report += 1
            with self.lock:
                done, now = self.reported, time.monotonic()
                running = sorted(self.running_since.values(), key=lambda item: item[2])
            self.say(f"{self.cell} 进度（第 {report} 次报，间隔 {self.interval_seconds} 秒）：{self.total} 条里判完 {done} 条、在跑 {len(running)} 趟")
            for _group, label, since in running[:LONGEST_RUNNING_SHOWN]:
                self.say(f"{self.cell} 跑得最久的一趟：{label}，已跑 {now - since:.0f} 秒")
            for line in self.extra_lines():
                self.say(line)
            self.say(f"{self.cell} 内存：{memory}")

    def sample_memory(self):
        """采一次后代进程的内存，更新峰值 → 报进度用的一句。只在报进度的线程里调。"""
        total, measured = 0, 0
        for pid in descendants_of(os.getpid()):
            kib = proportional_kib(pid)
            if kib is None:
                continue
            measured += 1
            total += kib
            if kib > self.largest[0]:
                self.largest = (kib, command_words(pid))
        charge = cgroup_charge_kib()
        with self.lock:
            groups = collections.Counter(group for group, _label, _since in self.running_since.values())
        self.samples += 1
        if self.peak is None or total > self.peak[0]:
            self.peak = (total, measured, charge, groups)
        if charge is not None and (self.peak_charge is None or charge > self.peak_charge):
            self.peak_charge = charge
        return f"此刻后代进程 {measured} 个、Pss 合计 {mib(total)}、cgroup 记账 {mib(charge)}；到此刻 Pss 合计峰值 {mib(self.peak[0])}、cgroup 记账峰值 {mib(self.peak_charge)}"

    def close(self):
        """停报进度的线程，打收尾一行。逐条进度的行数与派出去的条数相等，才是每条判完都报了一行。"""
        self.stop.set()
        if self.thread is None:
            return
        self.thread.join()
        if self.peak is None:
            memory = f"内存没采到（整格跑得比一次采样间隔 {self.sample_seconds} 秒短）"
        else:
            total, measured, charge, groups = self.peak
            in_flight = "、".join(f"{group} {count} 趟" for group, count in groups.most_common()) or "无"
            memory = (f"内存采了 {self.samples} 次，后代进程 Pss 合计峰值 {mib(total)}（那一刻 {measured} 个进程、cgroup 记账 {mib(charge)}；在跑的按组：{in_flight}），"
                      f"cgroup 记账峰值 {mib(self.peak_charge)}，单个进程 Pss 最大 {mib(self.largest[0])}（{self.largest[1]}）")
        self.say(f"{self.cell} 逐条进度 {self.reported} 行、派出去 {self.total} 条；{memory}")
PY_SWITCH_REPLAY_PROGRESS
)"

# 开关复跑的并行度、指定范围与每趟的起法（break-switch-replay 格文件头「范围」「并行度」「每趟的进程组」；判定在格自己那一段）。装法照 BREAK_SWITCH_TABLE_LIBRARY：
#   parallel_library = types.ModuleType("lib_switch_replay_parallelism"); exec(compile(<这一段>, "switch-replay-parallelism-library", "exec"), parallel_library.__dict__)
# 起趟的 TripLauncher 也给 selftest-cells-alone 格用（同一份起法，不各写一份）。
SWITCH_REPLAY_PARALLELISM_LIBRARY="$(cat <<'PY_SWITCH_REPLAY_PARALLELISM'
import collections, contextlib, os, re, signal, subprocess, threading, time

SAFE_SHARE_PERCENT = 75                    # jobs_for 按内存上限算并行时（只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory 下）可用内存给开关趟用的份额；其余留给门禁主进程、采样漏掉的短尖峰与开关趟比基准趟大的部分
TRIP_STOP_GRACE_SECONDS = 3                # 对一趟的整组发 TERM 之后等它自己收尾的秒数，过了组里还有活的发 KILL；比 .claude/gate.d/lib/stage-cells.sh 收格的宽限（5 秒）短，门禁被停时这一层先把在跑的各趟收完
TRIP_SETTLE_SECONDS = 1                    # 发 KILL 之后等组里退干净的秒数；过了还在的记「停不掉」，这一趟占的内存额度不还
TRIP_EXIT_SETTLE_SECONDS = 2               # 一趟直接起的那一个退出之后，等组里别的进程自己退的秒数；过了还在的记「留下进程」，停整组
TRIP_POLL_SECONDS = 0.5                    # 等一趟时每隔这么久醒一次：看到没到点、门禁收没收到停的信号、直接起的那一个退了没有
TRIP_STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)
COMMAND_LINE_SHOWN = 120                   # 留下的进程逐个写进失败信息时，命令行只写前这么多个字符
JOBS_WITHOUT_MEMORY_LIMIT = 4              # 读不到内存上限、或上限是 max 时的并行条数（不超过登记的每台默认重活线程数）
TRIP_MEMORY_FALLBACK_BYTES = 4 * 2 ** 30   # 基准趟没量到进程树峰值时一趟的内存：整份 task_server_lib.py 自证一趟的 Pss 合计量到约 3.8 GiB，取整到 4 GiB
MIN_TRIP_BYTES = 64 * 2 ** 20              # 量到的一趟再小也按这个数占额度（一个解释器加几个子进程）
METER_INTERVAL_SECONDS = 0.25              # 量进程树峰值的采样间隔
MEBIBYTE = 2 ** 20
SELECTION_VARIABLE = re.compile(r"\b([A-Z][A-Z0-9_]*_SELFTEST_CELLS)=<格名")
CELL_IN_ITEM = re.compile(r"自证[：\s]([a-z0-9]+(?:-[a-z0-9]+)*)：")
CELL_IN_ITEM_COLON_ONLY = re.compile(r"自证：([a-z0-9]+(?:-[a-z0-9]+)*)：")
KNOB_FILE = ".break-switch-replay-jobs"


def selection_variable(text):
    """这份脚本有选格手段吗：文件里写着「<名字>_SELFTEST_CELLS=<格名…」的，交那个环境变量名（设成逗号隔开的格名，自证只跑点名的格）；没有交 None。"""
    found = SELECTION_VARIABLE.search(text)
    return found.group(1) if found else None


def cells_in(item, broken=None):
    """一条点名的项（失败行句首）里的格名：「自证：<格名>：」或「✗ 自证 <格名>：」里的那一段，一项里写几处取几处。
    弄坏开关 replay-cell-name-colon-only 下只认冒号那一种，「✗ 自证 <格名>：」读不出格名（那条开关与基准趟整份起）。"""
    return (CELL_IN_ITEM_COLON_ONLY if broken == "replay-cell-name-colon-only" else CELL_IN_ITEM).findall(item)


def parse_knobs(text):
    """样本根的 .break-switch-replay-jobs（一行一项「键=值」，空行与 # 开头的行不算）→ {键: 值}：memory-max 写字节数、max 或 unreadable，threads 写正整数。写错抛 ValueError。"""
    knobs = {}
    for row in text.split("\n"):
        row = row.strip()
        if not row or row.startswith("#"):
            continue
        key, _separator, value = row.partition("=")
        number = value.isdigit() and int(value) > 0
        if not (key == "memory-max" and (number or value in ("max", "unreadable")) or key == "threads" and number):
            raise ValueError(f"{KNOB_FILE} 里的「{row}」读不出来（要写 memory-max=<字节数|max|unreadable> 或 threads=<正整数>）")
        knobs[key] = value
    return knobs


def limiting_cgroup(cgroup_file="/proc/self/cgroup", cgroup_root="/sys/fs/cgroup"):
    """→ (那一层的目录或 None, 字节数或 None, 说明)：本进程所在的 cgroup 一路到根，memory.max 里数字最小的那一层；
    一层数字都没有：读成过的都是 max → (None, None, 是 max)，一层也没读成 → (None, None, 读不到)。"""
    try:
        with open(cgroup_file, encoding="utf-8") as handle:
            relative = next(row.rstrip("\n")[3:] for row in handle if row.startswith("0::"))
    except (OSError, StopIteration) as error:
        return None, None, f"读不到（本进程的 cgroup 路径读不出：{error!r}）"
    path, readable, numbers = relative.rstrip("/"), 0, []
    while True:
        try:
            with open(f"{cgroup_root}{path}/memory.max", encoding="utf-8") as handle:
                text = handle.read().strip()
        except OSError:
            text = None
        if text is not None:
            readable += 1
            if text.isdigit():
                numbers.append((int(text), f"{cgroup_root}{path}"))
        if not path:
            break
        path = path.rsplit("/", 1)[0]
    if numbers:
        limit, directory = min(numbers)
        return directory, limit, f"cgroup {relative} 一路到根的 memory.max 里最小的"
    if readable:
        return None, None, "是 max（沿 cgroup 往上都没设上限）"
    return None, None, "读不到（memory.max 一层也没读成）"


def cgroup_memory_limit(cgroup_file="/proc/self/cgroup", cgroup_root="/sys/fs/cgroup"):
    """→ (字节数或 None, 说明)：limiting_cgroup 那一层的 memory.max。"""
    return limiting_cgroup(cgroup_file, cgroup_root)[1:]


def memory_limit_of(knobs):
    """→ (字节数或 None, 说明)：样本根的 .break-switch-replay-jobs 写了 memory-max 就用它，否则读本进程的 cgroup。"""
    value = knobs.get("memory-max")
    if value is None:
        return cgroup_memory_limit()
    if value == "max":
        return None, "是 max（没设上限）"
    if value == "unreadable":
        return None, f"读不到（样本根的 {KNOB_FILE} 写了 unreadable）"
    return int(value), f"样本根的 {KNOB_FILE}"


def budget_of(limit_bytes):
    """这一趟可用内存里给开关趟用的那一份（字节）。"""
    return limit_bytes * SAFE_SHARE_PERCENT // 100


def jobs_for(threads, limit_bytes, trip_bytes):
    """按内存上限算的并行条数 = min(登记的每台默认重活线程数, ⌊份额 ÷ 每趟内存⌋)，至少 1；内存上限读不到或是 max（limit_bytes 是 None）：不超过线程数的 JOBS_WITHOUT_MEMORY_LIMIT 条。
    break-switch-replay、selftest-cells-alone 两格只在弄坏开关 replay-jobs-by-memory、cells-alone-jobs-by-memory 下用它（两格并行都取登记的线程数，各自文件头「并行度」「并行」）。"""
    if limit_bytes is None:
        return min(threads, JOBS_WITHOUT_MEMORY_LIMIT)
    return min(threads, max(1, budget_of(limit_bytes) // trip_bytes))


class TripHold:
    """Admission.slot 交给调用方的那一个：keep 置真时出去不还这一趟的内存额度（组里还有停不掉的进程，内存还占着）。"""
    __slots__ = ("keep",)

    def __init__(self):
        self.keep = False


class Admission:
    """起一趟自证之前过的关：同时在跑的不超过 max_running 趟，在跑的各趟内存估计之和加上要进来的这一趟不超过 budget_bytes（None 是不按内存限）；
    一趟都没在跑时不看估计，放一趟进来（至少 1 条）。
    内存估计：开跑前一份脚本的每一趟都是登记值（default_bytes）；基准趟跑完换成它量到的进程树峰值；这份脚本有一趟开关趟跑完之后，
    每趟按「每个单位占的最大字节 × 这一趟的单位数」算（单位是只起点名的格：一趟起几格就几个单位，整份起的趟是一个单位；
    每个单位占的是这份已跑完的开关趟里峰值除以单位数的最大值，比整份的基准趟小得多，起的格多的趟也不会被低估）。
    还名额：slot 等调用方那一趟返回才还（TripLauncher.run 核完组里一个不剩才返回）；调用方把 slot 交的 TripHold.keep 置真的，位置还、内存额度不还（一直算在 reserved 里）。"""

    def __init__(self, budget_bytes, max_running, paths, default_bytes):
        self.condition = threading.Condition()
        self.budget_bytes, self.max_running = budget_bytes, max_running
        self.baseline_bytes = {path: default_bytes for path in paths}
        self.per_unit = {}
        self.running, self.reserved = 0, 0

    def cost(self, path, units):
        if path in self.per_unit:
            return max(MIN_TRIP_BYTES, self.per_unit[path] * units)
        return self.baseline_bytes[path]

    def fits(self, path, units):
        if self.running >= self.max_running:
            return False
        if self.budget_bytes is None or not self.running:
            return True
        return self.reserved + self.cost(path, units) <= self.budget_bytes

    @contextlib.contextmanager
    def slot(self, path, units=1):
        """占一个位置跑一趟（这一趟起 units 个单位）：位置不够就等；占的是进来那一刻的估计，出去时按同一个数还（交出去的 TripHold.keep 置真的不还）。"""
        with self.condition:
            while not self.fits(path, units):
                self.condition.wait()
            cost = self.cost(path, units)
            self.running += 1
            self.reserved += cost
        hold = TripHold()
        try:
            yield hold
        finally:
            with self.condition:
                self.running -= 1
                if not hold.keep:
                    self.reserved -= cost
                self.condition.notify_all()

    def observe(self, path, is_baseline, units, peak_kib):
        """一趟跑完，量到的进程树峰值（KiB，没量到是 None）换进这份脚本的估计。"""
        if peak_kib is None:
            return
        measured = max(MIN_TRIP_BYTES, peak_kib * 1024)
        with self.condition:
            if is_baseline:
                self.baseline_bytes[path] = measured
            else:
                self.per_unit[path] = max(self.per_unit.get(path, 0), measured // max(1, units))
            self.condition.notify_all()


def process_table():
    """→ ({父进程号: [子进程号…]}, {进程组号: [进程号…]})：扫一遍 /proc/<进程号>/stat；读的时候已经退了的与僵尸不算。"""
    children_of, members_of = collections.defaultdict(list), collections.defaultdict(list)
    for name in os.listdir("/proc"):
        if not name.isdigit():
            continue
        try:
            with open(f"/proc/{name}/stat", "rb") as handle:
                stat = handle.read()
        except OSError:
            continue
        # 命令名在括号里、自己可以带空格与括号：取最后一个「)」之后的字段：状态、父进程号、进程组号
        fields = stat[stat.rfind(b")") + 1:].split()
        if len(fields) < 3 or fields[0] in (b"Z", b"X") or not (fields[1].isdigit() and fields[2].isdigit()):
            continue
        children_of[int(fields[1])].append(int(name))
        members_of[int(fields[2])].append(int(name))
    return children_of, members_of


class TripMeter:
    """起的每一趟自证的内存峰值：一条线程每隔 METER_INTERVAL_SECONDS 秒扫一遍 /proc，把在看的每个根进程底下整棵树与它那个进程组里的进程
    （TripLauncher 起的每趟自成一组，组号就是根的进程号；父进程没了、被收养走的照算）的 Pss 加起来记最大。
    采样量不到比间隔短的尖峰，峰值是下限（份额里留了余量）；一趟比一次采样间隔还短，release 交 None。progress 是开关复跑的逐条进度库（读 Pss 的函数在里面）。"""

    def __init__(self, progress, interval_seconds=METER_INTERVAL_SECONDS):
        self.progress, self.interval_seconds = progress, interval_seconds
        self.lock = threading.Lock()
        self.peaks = {}
        self.stop = threading.Event()
        self.thread = None

    def watch(self, pid):
        with self.lock:
            self.peaks[pid] = 0
            if self.thread is None:
                self.thread = threading.Thread(target=self.loop, daemon=True)
                self.thread.start()

    def release(self, pid):
        with self.lock:
            peak = self.peaks.pop(pid, 0)
        return peak or None

    def loop(self):
        while not self.stop.wait(self.interval_seconds):
            self.sample()

    def sample(self):
        with self.lock:
            roots = list(self.peaks)
        if not roots:
            return
        try:
            children_of, members_of = process_table()
        except OSError:
            return
        for root in roots:
            total, seen = 0, False
            for pid in {root, *self.progress.subtree_of(children_of, root), *members_of.get(root, ())}:
                kib = self.progress.proportional_kib(pid)
                if kib is not None:
                    total, seen = total + kib, True
            with self.lock:
                if seen and root in self.peaks and total > self.peaks[root]:
                    self.peaks[root] = total

    def close(self):
        self.stop.set()
        if self.thread is not None:
            self.thread.join()


def described(pid):
    """「<进程号> <命令行前 COMMAND_LINE_SHOWN 个字符>」：留下的进程逐个写进失败信息用。"""
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            command_line = handle.read().replace(b"\0", b" ").decode("utf-8", errors="replace").strip()
    except OSError:
        command_line = "（读不到命令行）"
    return f"{pid} {command_line[:COMMAND_LINE_SHOWN]}"


def group_members(group_id):
    """这个进程组里还没退出的进程号（僵尸不算），按 /proc 里的进程组号认。"""
    return sorted(process_table()[1].get(group_id, ()))


def wait_group_empty(group_id, seconds):
    """等这个进程组退干净，至多 seconds 秒 → 到点还在的进程号（空列表是退干净了）。"""
    deadline = time.monotonic() + seconds
    members = group_members(group_id)
    while members and time.monotonic() < deadline:
        time.sleep(0.1)
        members = group_members(group_id)
    return members


class TripLauncher:
    """起一趟自证（命令、环境、限时由调用方给），.claude/rules/command-safety-by-review.md「装置起的样本进程，放进它自己的进程组」：
    放进它自己的会话与进程组（start_new_session，组号就是直接起的那一个的进程号）；
    到点：对整组先 TERM，过 TRIP_STOP_GRACE_SECONDS 秒组里还有活的（僵尸不算）再 KILL，打一行「到点停整组」；
    直接起的那一个退出之后：等至多 TRIP_EXIT_SETTLE_SECONDS 秒，组里还有进程的逐个记进失败信息（「留下进程」），再照上面停整组；
    门禁收到 TERM、INT、HUP（install_stop_on_signals 装的处理）：在跑的各趟至多 TRIP_POLL_SECONDS 秒后醒来照上面停自己那一组，还没起的不再起；
    停完再等至多 TRIP_SETTLE_SECONDS 秒、按进程组号在 /proc 里核组里一个不剩，还在的记「停不掉」（stuck）。
    run 等核完才返回：调用方在 Admission.slot 里调它，名额核清才还；stuck 不空时调用方把 TripHold.keep 置真，内存额度不还。
    对组发信号之前先在 /proc 里核组里还有进程：组里一个都没有时组号可能已经不归这一趟，不发。
    管不到的：自己换了会话或进程组的后代（不在这一组里，停不到也核不到）。
    say 打「到点停整组」那一行。leader_only（弄坏开关 replay-trip-leader-only-stopped）：不开新会话，到点只 kill 直接起的那一个、不收组、不核，返回之后名额当场还。"""

    def __init__(self, say, leader_only=False):
        self.say, self.leader_only = say, leader_only
        self.stop_requested = threading.Event()
        self.stop_signal = []

    def install_stop_on_signals(self):
        """在主线程装 TERM、INT、HUP 的处理（开池之前调；不在主线程交 False、什么都不装）：记下收到了，在主线程抛 SystemExit(128 + 信号号)。"""
        if threading.current_thread() is not threading.main_thread():
            return False

        def stop(signal_number, _frame):
            self.stop_signal.append(signal_number)
            self.stop_requested.set()
            raise SystemExit(128 + signal_number)

        for signal_number in TRIP_STOP_SIGNALS:
            signal.signal(signal_number, stop)
        return True

    def stop_exit(self):
        return SystemExit(128 + (self.stop_signal[0] if self.stop_signal else signal.SIGTERM))

    def stop_group(self, group_id):
        """对这一组先 TERM，过 TRIP_STOP_GRACE_SECONDS 秒还有活的再 KILL → (发 TERM 时组里几个, 发 KILL 时还有几个)。组里一个都没有就不发。"""
        members = group_members(group_id)
        if not members:
            return 0, 0
        with contextlib.suppress(ProcessLookupError):
            os.killpg(group_id, signal.SIGTERM)
        remaining = wait_group_empty(group_id, TRIP_STOP_GRACE_SECONDS)
        if remaining:
            with contextlib.suppress(ProcessLookupError):
                os.killpg(group_id, signal.SIGKILL)
        return len(members), len(remaining)

    def run(self, command, environment, seconds, label, meter=None, merge_stderr=False, **popen):
        """起一趟 → {"timeout": 到点没跑完, "code": 直接起的那一个的退出码（到点是 None）, "stdout", "stderr"（merge_stderr 时是 None）,
        "problem": 留下进程与停不掉的拼成的一句（都没有是空串）, "stuck": 停不掉的那几个, "peak_kib": meter 量到的峰值}。popen 照传给 Popen（text 这类）。"""
        if self.stop_requested.is_set():
            raise self.stop_exit()
        error_stream = subprocess.STDOUT if merge_stderr else subprocess.PIPE
        empty = "" if popen.get("text") else b""
        if self.leader_only:
            timed_out = False
            with subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=error_stream, **popen) as process:
                if meter is not None:
                    meter.watch(process.pid)
                try:
                    stdout, stderr = process.communicate(timeout=seconds)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
                    stdout, stderr, timed_out = empty, empty, True
            return {"timeout": timed_out, "code": None if timed_out else process.returncode, "stdout": stdout, "stderr": stderr,
                    "problem": "", "stuck": [], "peak_kib": meter.release(process.pid) if meter is not None else None}
        with subprocess.Popen(command, env=environment, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=error_stream,
                              start_new_session=True, **popen) as process:
            if meter is not None:
                meter.watch(process.pid)
            outputs, timed_out, left = None, False, []
            deadline = time.monotonic() + seconds
            try:
                while True:
                    if self.stop_requested.is_set():
                        raise self.stop_exit()
                    try:
                        outputs = process.communicate(timeout=max(0.0, min(TRIP_POLL_SECONDS, deadline - time.monotonic())))
                        break
                    except subprocess.TimeoutExpired:
                        if time.monotonic() >= deadline:
                            timed_out = True
                            break
                        if process.poll() is not None:   # 直接起的那一个退了，管道还被组里别的进程拿着
                            break
            except BaseException:
                self.stop_group(process.pid)
                raise
            if timed_out:
                term_count, kill_count = self.stop_group(process.pid)
            else:
                remaining = wait_group_empty(process.pid, TRIP_EXIT_SETTLE_SECONDS)
                if remaining:
                    left = [described(pid) for pid in remaining]
                    self.stop_group(process.pid)
            if outputs is None:
                try:
                    outputs = process.communicate(timeout=TRIP_SETTLE_SECONDS)
                except subprocess.TimeoutExpired:   # 管道还被组外的进程拿着：不再等，收到的不要了
                    for stream in (process.stdout, process.stderr):
                        if stream is not None:
                            stream.close()
                    process.wait()
                    outputs = (empty, None if merge_stderr else empty)
            stuck = [described(pid) for pid in wait_group_empty(process.pid, TRIP_SETTLE_SECONDS)]
        if timed_out:
            self.say(f"到点停整组：{label}，限时 {seconds} 秒；发 TERM 时组里 {term_count} 个，过 {TRIP_STOP_GRACE_SECONDS} 秒还有 {kill_count} 个"
                     + ("、发了 KILL" if kill_count else "") + "；"
                     + (f"组里还剩 {len(stuck)} 个停不掉，这一趟的内存额度不还" if stuck else "组里一个不剩之后才还这一趟的名额"))
        parts = []
        if left:
            parts.append(f"退出之后它起的进程还留着 {len(left)} 个（等了 {TRIP_EXIT_SETTLE_SECONDS} 秒，已对整组 TERM、过 {TRIP_STOP_GRACE_SECONDS} 秒 KILL）：{'；'.join(left)}")
        if stuck:
            parts.append(f"收组之后组里还剩 {len(stuck)} 个停不掉的进程（这一趟的内存额度不还）：{'；'.join(stuck)}")
        return {"timeout": timed_out, "code": None if timed_out else process.returncode, "stdout": outputs[0], "stderr": outputs[1],
                "problem": "，".join(parts), "stuck": stuck, "peak_kib": meter.release(process.pid) if meter is not None else None}
PY_SWITCH_REPLAY_PARALLELISM
)"

# ── 格 stage-break-switch-replay：还 checks-owed.md C639（门禁阶段的弄坏开关没有复跑方）。
# 判据：被判的根里 .claude/gate.d/ 顶层每道有样本目录的阶段，文件头开关表（mutate.sh 写法，读法在 BREAK_SWITCH_TABLE_LIBRARY；点名的项写样本目录名）逐条：
#   在环境里设上那个开关，经这一道所在仓里的 .claude/singlefs-ai-sop/scripts/stage-selftest.sh <根>/.claude/gate.d --item <阶段> 喂这一道的全部样本，
#   判错的样本集合（stage-selftest 报「✗ <阶段> <样本> …」的那几份）要与那一条点名的样本目录名集合相等；不等判红，逐条列多出与少了的。
#   开关表一条都读不出、或有一条没写全的阶段判红。被喂的样本根里都有 .gate-cells 或 .gate-break-sample（C622：开关只在样本根上认）。
#   头注读不出判红：注释行里 # 之后头一个词是 <名字以 _BREAK 结尾的变量>=，读法却不认它是一条开关（值写错、值后面只隔一个空格接着写字），那一条就悄悄不复跑，判红、点名文件:行。
#   不算的：句中提到开关的（「弄坏开关 X_BREAK=值 下……」）、占位写法（X_BREAK=<项>）、不是注释的代码行。
#   样本 stage-break-switch-replay-unread-row-red（小阶段头注里一行值后面只隔一个空格）、stage-break-switch-replay-unread-row-green（句中提到的、占位的与代码行都有，判绿）。
#   弄坏开关 CODE_TOOLING_BREAK=stage-unread-rows-unchecked：不判这一条，stage-break-switch-replay-unread-row-red 判绿（期望退出 1，实测 0）。
#   各条互不依赖、各起各的 stage-selftest（样本各拷各的临时目录），同时跑几条照下面「并行度」，收回的条数与派出去的对不上整格红。
# 并行度（records/2026-11-17-subagent账本自上而下清欠.md「二百一十九、任务按核开满」：按核开满）：同时跑的条数取登记的每台默认重活线程数（.claude/heavy-thread-default），
#   不按内存收，理由同 break-switch-replay 格「并行度」（准许按内存上限定并发的入口登记在 .claude/memory-bounded-parallelism.tsv，这一格不在里面）：
#   内存上限由 task-server 照这条命令的记录给、照以往一路跑的峰值定，照它收并行就永远收在一路、也永远撞不上上限、服务也就不放大；
#   撞了上限服务照乘 1.5 放大、重排这一条，第二趟只补没判的（下面「按条复用」：一条判完就记）。
#   外层同时跑的条数 × 每条 stage-selftest 的 --jobs 不超过每台默认重活线程数：外层路数的上限 = min(每台默认重活线程数, 要现判的条数)，
#   里层每条 stage-selftest 带 --jobs ⌊每台默认重活线程数 ÷ 外层路数的上限⌋（至少 1）；同时跑的条数就是外层路数的上限。
#   stage-selftest 喂的每份样本里，阶段自己再开的并行不在这笔账里：样本的期望写着登记的线程数，不能经环境变量往下收，这一层收不住。
#   内存只写进开跑前那一行：这一台能给的取 scripts/hosts/task_memory.py 的 host_slice_cap_bytes（多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项），
#   这一条拿到的上限取同一份的 task_memory_cap_bytes（服务交的 TASK_SERVER_MEMORY_CAP_BYTES，再到多机配置那一项），取不到照写从哪没取到。
#   打的行：开跑前「并行度：同时最多 N 条（登记的每台默认重活线程数 T 条；这一台能给的 …；这一条拿到的上限 …；由服务定、撞了服务照乘 1.5 放大重排，不按它收并行）；
#   每条 stage-selftest --jobs M；外层 N 条 × 里层 M = P（不超过每台默认重活线程数 T）」；收尾「同时在跑最多 K 条（同时最多 N 条）」（K 是实测的）。
#   样本根的 .stage-break-switch-replay-memory（一行一项 capacity=<字节>、hard-cap=<字节>、threads=<正整数>、peak:<阶段文件名>=<字节>）顶替这一台能给的、这一条拿到的上限、
#   每台默认重活线程数与那一道阶段每条开关开满时的峰值（峰值只给弄坏开关 stage-replay-jobs-by-memory 用）；样本根上没有这份文件时内存两样都当取不到。写错一行判红、一条都不起。
#   样本 stage-break-switch-replay-jobs-registered-green（这一台能给 1 TiB、这一条拿到的上限 2.5 GiB、峰值 1 GiB、线程 3；四条开关、每条的小阶段两份样本各睡 2 秒：
#   同时最多 3 条、同时在跑最多 3 条、每条 --jobs 1，外层 3 条 × 里层 1 = 3）；stage-break-switch-replay-jobs-registered-red（同一组并行度、同一道小阶段，
#   invert-all 那一条点名的样本少写了 slot-red：只那一条点名的与实测不等，1 处不对）；stage-break-switch-replay-threads-split-green（能给 1 TiB、峰值 1 GiB、线程 9：
#   四条开关，外层 4 条 × 里层 --jobs 2 = 8，不超过 9）；stage-break-switch-replay-memory-knob-red（那份文件里一行读不出，判红）。
#   弄坏开关 CODE_TOOLING_BREAK=stage-replay-jobs-by-memory：同时跑的条数再照份额（这一台能给的与这一条拿到的上限里小的，扣掉本进程此刻的用量）÷ 样本根写的那道阶段的峰值收，
#   -jobs-registered-green 与 -jobs-registered-red 同时最多 2 条、判红；stage-replay-inner-jobs-unset：stage-selftest 不带 --jobs，
#   -jobs-registered-green 里「每条 stage-selftest --jobs 1」与 threads-split-green 里「--jobs 2」那两条 want 找不到；stage-replay-inner-threads-uncapped：里层 --jobs 仍取每台默认重活线程数、
#   不按外层路数分，-jobs-registered-green 与 -jobs-registered-red 里「外层 3 条 × 里层 1 = 3」、threads-split-green 里「外层 4 条 × 里层 2 = 8」那几条 want 找不到；
#   stage-replay-memory-knob-unchecked：读不出的行悄悄跳过，memory-knob-red 退 77（期望 1）。
# 不复跑的（逐条报本次未跑，往 $GATE_NOT_RUN_FILE 报一行）：
#   .claude/hooks/selftest-registry.tsv 里这一道的样本档没登记成 samples 的（样本档在登记表里才复跑）；
#   .claude/gate.d/stage-break-switch-replay-exempt.tsv 登记的（「<阶段文件名><TAB><开关=值，或 * 整道><TAB># 理由>」：头注还不是这种写法，
#   或点名的与实测不等、头注还没改准的那几条）。登记的一条在头注里已经没有了（改了名、删了）判红：那一行删掉。表只缩不涨（不判涨，靠 review）。
# 一道有样本目录的阶段都没有：退 77。真仓上一趟约 700 条，每条把那一道的全部样本串着喂一遍；第二台跑完整格的两趟是 695 条 5409 秒、704 条 2392 秒
#   （任务 3630、3922 的日志），整格要四十分钟到一个半小时。
# 进度（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔 5 分钟；写法在 SWITCH_REPLAY_PROGRESS_LIBRARY）：开跑打一行「stage-break-switch-replay 开跑：」
#   （条数、阶段数、同时最多几条）；每条判完打一行「stage-break-switch-replay 第 k / N 条<对上、没对上（点名的样本与实测不符）或没跑成（stage-selftest.sh 退 c）>：<阶段> <开关>，用了 S 秒」；
#   另起一条线程每过 300 秒打一行总进度（判完几条、在跑几趟）、跑得最久的三趟与一行内存（后代进程数、Pss 合计、cgroup 记账、到此刻的峰值），
#   每 5 秒采一次内存；收尾打一行「stage-break-switch-replay 逐条进度 R 行、派出去 N 条；」加内存峰值、峰值那一刻在跑的按阶段计数与单个最大的进程。
#   样本根里的 .stage-break-switch-replay-progress-seconds（一个正整数秒数）顶替 300 秒；写的不是正整数判红、一条都不跑（switch_replay_progress_seconds）。
# 样本：stage-break-switch-replay-red（一道小阶段：一条开关点名对、一条点名的与实测不等、一条没写点名的项；豁免表一行指到头注里没有的开关；
#   一道没登记的不跑，逐条点名判红）；stage-break-switch-replay-green（点名都对的两条、豁免一条、没登记的一道本次未跑，
#   报「复跑了 2 条开关（1 道阶段）」）。
#   stage-break-switch-replay-progress-green：样本根里 .stage-break-switch-replay-progress-seconds 写 1；一道小阶段两份样本、喂每份先睡 2 秒，两条开关各弄错一份、点名都对，
#   必须判绿，且输出里有开跑一行、「进度（第 1 次报」、跑得最久的一趟、内存一行、「第 1 / 2 条对上」「第 2 / 2 条对上」与「逐条进度 2 行、派出去 2 条」。
# 弄坏开关 CODE_TOOLING_BREAK=switch-replay-named-unchecked：不比点名的与实测的，stage-break-switch-replay-red 里「点名的与实测判错的不等」那条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=replay-progress-silent：不打进度、不起报进度的线程，stage-break-switch-replay-progress-green 里那几条进度的 want 找不到
#   （与 break-switch-replay 格同一条开关，各红各的进度样本）。
# 临时服务的二进制：每条开关起的 stage-selftest.sh 都套 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 包装，包装不带二进制又没有任务服务给的编译目录就不现编、退 2。
#   任务服务只给最外层命令编译目录（外层是这一道门禁，不算会编的），所以开跑前取一次（code_tooling_prune_server_binary：PRUNE_SERVER_BINARY 设了用它，没设照 prune-server-selftest 格的编法编进这一格自己的临时目录，跑完删），
#   每条都经 --server-binary 交；起每条前摘掉环境里的 PRUNE_SERVER_BINARY 与 CARGO_TARGET_DIR，二进制只认交的这一份，哪台机器上结局相同。取不到整格判红、一条开关都不起，出路写编译命令。二进制路径不进单元键。
#   样本根里没有 prune-server/，样本档靠外层包装设的 PRUNE_SERVER_BINARY（见「样本根只在临时 prune-server 底下判」那一段，裸跑样本档退 2）。
# 弄坏开关 CODE_TOOLING_BREAK=stage-replay-binary-not-passed：不取、不交二进制，套包装起的每条退 2、「没跑成（stage-selftest.sh 退 2）」，文件头开关表点名的七份样本判错。
# 按条复用（.claude/rules/implementation-workflow.md「复用重型门禁的判定：按每条判定的单元键判」；共用段在 UNIT_VERDICT_ROUNDS_LIBRARY）：
#   一条判定 = 一道阶段加它的一条开关，名字「<阶段文件名> <开关名>=<值>」，种类 code-tooling.sh:stage-break-switch-replay（prune-server 那一份库，unit-verdicts 族
#   singlefs-unit-verdicts/code-tooling.sh:stage-break-switch-replay，库根不进键）。键（scripts/gate/admission.py verdict-plan 算，不带路径、行号与限时）由这几样组成：
#   ① 那道阶段的单元闭包（入口 .claude/gate.d/<阶段>，shell 的 script-body 连同它调到、source 到的函数单元，去注释）；
#   ② 判法：这一道所在仓里 stage_break_switch_replay_judge_code 的闭包（令牌 STAGE_BREAK_SWITCH_REPLAY_JUDGE_TOKEN：开关表的读法与 STAGE_BREAK_SWITCH_REPLAY_JUDGE_LIBRARY）；
#   ③ line:switch=<开关名>=<值>；④ line:named=<点名的样本，排好用逗号连>（文件头是注释，不进单元指纹，另进这一行）；
#   ⑤ line:samples=<它喂的 fixtures/<阶段>/ 整棵的摘要：每个目录、每份文件的相对路径、执行位与内容、每个链接的指向>；
#   ⑥ line:feeder=<喂样本的 stage-selftest.sh、它旁边的 lib.sh 与临时服务包装三份内容的摘要>。
#   键相同、库里记着绿的不起（复用）；键对不上的、库里记着红的（原因以「上次判红」起头）现判；拒算的（闭包收不全）照共用段开跑前判红、一条都不起（UNIT_VERDICT_ROUNDS_LIBRARY 的 ItemVerdicts.stop_on_refusals）。
#   每趟开跑前 verdict-plan 抽复用哨兵、强制现判一条（种子打进日志；被判的根下有 .unit-verdict-sample/ 的样本不抽）；哨兵被现判、强制现判那一条与库里的不同，交 plan-failure 判红。
#   一条判完就记：判绿记绿，判红照同一把键记红、撤掉原有的绿（verdict-record --red-names）；收尾把这一趟判完的整份再交一次（哨兵在那一趟比）。
#   中途被打断，判完的那几条都在库里，同一条命令重跑就是续跑、只补没判的（.claude/rules/test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：
#   样本 stage-break-switch-replay-interrupted-resume（同一份判定库上三趟，code_tooling_interrupted_rounds：判完一条就停，第二趟现判 2、复用 1，第三趟全部复用）。
#   弄坏开关 CODE_TOOLING_BREAK=stage-replay-recorded-at-end：只在收尾记，第二趟现判 3，「现判 2、复用 1」那条 want 找不到。
#   成功句报「按条复用：现判 N、复用 M、拒算 R」；全部复用时报「全部复用，这一格一条都没起」、退 0。算不出（verdict-plan 退非 0、没打 PLAN）照全部现判跑完、判红；
#   prune-server 不可用（退 75）一条都不起、这一格退 75。
#   逐项点名（这一道「逐项点名」一节）：项是 <阶段文件名> 或 <阶段文件名>:<开关名>=<值>，只算、只起点到的那几条，键与判法照整格。
#   样本：stage-break-switch-replay-reuse-three-questions（两道小阶段三条开关；复用三问：change.sh 改 alpha-stage.sh 的 judged_verdict，affected 是它的两条；四趟都绿、三问都过）；
#     stage-break-switch-replay-samples-in-key（同样两道；change.sh 只给 alpha-stage.sh 的一份样本的 expect 多加一句照样对得上的 want：样本内容进键，alpha 的两条现判）；
#     stage-break-switch-replay-red-over-green（.unit-verdict-sample/prior/ 照判绿记两条；red-over-green 下一律现判，样本根的 .red-switch 让 swallow-red 那一条点错，
#     判红记红、撤绿，再算一次计划：复用 1 条、现判 1 条、原因「上次判红」）；stage-break-switch-replay-item-green、stage-break-switch-replay-item-unknown（逐项点名）。
#   弄坏开关（文件头开关表）CODE_TOOLING_BREAK=replay-plan-ignored、replay-red-not-recorded、stage-replay-samples-unkeyed、stage-replay-key-whole-gate-directory、
#     items-ignored、item-unknown-accepted，各红在开关表点名的样本。
#   跑成了而点名的与实测不等的条，汇总前重算键（ItemVerdicts.key_changes，写法在共用段上方的注释）：与开跑前算的不同，那一条的失败行接「跑的过程中键变了（开跑前算的键、现在的键）：
#     组成清单有 N 处不同：…；出路：在冻结快照树里重判…」，键没变的失败行不变；这一趟没有这样的条就一个进程都不多起。
#   样本 stage-break-switch-replay-key-changed-red（那道小阶段每被喂一次，往这一棵树里自己的样本目录 fixtures/<阶段>/ 下追加一个文件：swallow-red 点错，键在跑的过程中变了）；
#     键没变的在 stage-break-switch-replay-red 里报「键变了 0 条」。弄坏开关 CODE_TOOLING_BREAK=key-change-not-rechecked、key-change-noted-always，各红在开关表点名的样本。
cell_stage_break_switch_replay() {
  [[ -d .claude/gate.d/fixtures ]] || { echo "  ! 没有 .claude/gate.d/fixtures，这一格无对象可判"; exit 77; }
  local jobs
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），复跑的并行度定不了，一条都没跑"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  local stage_replay_progress_seconds
  if ! stage_replay_progress_seconds="$(switch_replay_progress_seconds .stage-break-switch-replay-progress-seconds)"; then
    printf '%s\n' "$stage_replay_progress_seconds"
    exit 1
  fi
  # 每条开关起的 stage-selftest.sh 都套临时服务包装；包装不现编（任务服务只给最外层会编的命令编译目录，门禁在外层），开跑前取一次二进制交给每一条（这一格文件头「临时服务的二进制」）
  local stage_replay_binary="" stage_replay_scratch="" stage_replay_exit
  if [[ -f "$STAGE_REPOSITORY/scripts/prune-server/prune_server_client_selftest.py" && "${CODE_TOOLING_BREAK:-}" != stage-replay-binary-not-passed ]]; then
    stage_replay_scratch="$(mktemp -d)" || { echo "  ✗ 建不了放 prune-server 二进制的临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑这一格；一条开关都没起。"; exit 1; }
    if ! stage_replay_binary="$(code_tooling_prune_server_binary "$stage_replay_scratch" 2>"$stage_replay_scratch/server-binary.err")"; then
      echo "  ✗ 取不到 prune-server 二进制（$(tail -n 3 "$stage_replay_scratch/server-binary.err" | tr '\n' ' ' | clip_text 400)），每条开关起的 stage-selftest.sh 要套在临时服务底下，一条开关都没起"
      echo "     → 怎么办：修好 prune-server/ 的编译（cargo build --release --offline --manifest-path prune-server/Cargo.toml --bin prune-server）再跑这一格；或设 PRUNE_SERVER_BINARY=<编好的二进制的绝对路径>。"
      rm -rf -- "$stage_replay_scratch"
      exit 1
    fi
  fi
  # 复用三问（这一格文件头「按条复用」的样本）：样本根带 .unit-verdict-sample/three-questions/ 的，同一份判定库上跑四趟、判三问
  if judged_root_is_sample && [[ -d .unit-verdict-sample/three-questions ]]; then
    code_tooling_three_question_rounds stage-break-switch-replay stage_break_switch_replay_pass; stage_replay_exit=$?
  elif judged_root_is_sample && [[ -f .unit-verdict-sample/interrupted-resume/stop-after ]]; then
    code_tooling_interrupted_rounds stage-break-switch-replay stage_break_switch_replay_pass; stage_replay_exit=$?
  else
    stage_break_switch_replay_pass ""; stage_replay_exit=$?
  fi
  [[ -z "$stage_replay_scratch" ]] || rm -rf -- "$stage_replay_scratch"
  return "$stage_replay_exit"
}

# stage_break_switch_replay_pass <各条结局写到哪，空是不写>：这一格的一趟（算键、查库、起键对不上的、记库、判）；jobs 与 stage_replay_progress_seconds 取调用它的那一格的
stage_break_switch_replay_pass() {
  python3 - "$(stage_break_switch_replay_judge_code)" "$STAGE_REPOSITORY/.claude/hooks/lib_project_code_runs.py" "$STAGE_REPOSITORY/.claude/singlefs-ai-sop/scripts/stage-selftest.sh" \
    "$ROOT" "$jobs" "$STAGE_NAME" "${CODE_TOOLING_BREAK:-}" "${GATE_NOT_RUN_FILE:-}" "$SWITCH_REPLAY_PROGRESS_LIBRARY" "$stage_replay_progress_seconds" "$STAGE_REPOSITORY" \
    "$UNIT_VERDICT_ROUNDS_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/admission.py" "$STAGE_NAME:stage-break-switch-replay" "$STAGE_BREAK_SWITCH_REPLAY_JUDGE_TOKEN" "$STAGE_REPOSITORY" \
    "${CODE_TOOLING_PICKED[stage-break-switch-replay]:-}" "$(judged_root_is_sample && echo 1)" "$1" "$stage_replay_binary" <<'PY'
import concurrent.futures, glob, hashlib, importlib.util, os, re, shutil, subprocess, sys, tempfile, threading, time, types
judge_library = types.ModuleType("lib_stage_break_switch_replay_judge")
exec(compile(sys.argv[1], "stage-break-switch-replay-judge-library", "exec"), judge_library.__dict__)
table = judge_library   # 开关表的读法与判法在同一个模块里（stage_break_switch_replay_judge_code 打的两段）
runs_library_path, stage_selftest, root, jobs, own_stage, broken, not_run_file = sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]), sys.argv[6], sys.argv[7], sys.argv[8]
progress_library = types.ModuleType("lib_switch_replay_progress")
exec(compile(sys.argv[9], "switch-replay-progress-library", "exec"), progress_library.__dict__)
progress_seconds = int(sys.argv[10])     # 进度间隔（这一格文件头「进度」）
stage_repo = sys.argv[11]
rounds = types.ModuleType("lib_unit_verdict_rounds")
exec(compile(sys.argv[12], "unit-verdict-rounds-library", "exec"), rounds.__dict__)
admission, kind, judge_token, judge_root = sys.argv[13], sys.argv[14], sys.argv[15], sys.argv[16]
picked = [item for item in sys.argv[17].split("\n") if item]      # 逐项点名（这一道「逐项点名」一节）：空是整格
sample, states_path = sys.argv[18] == "1", sys.argv[19]
server_binary = sys.argv[20]     # 临时服务包装起的 prune-server 二进制（这一格开跑前取一次）；空是没取（只在弄坏开关 stage-replay-binary-not-passed 下）
task_memory_spec = importlib.util.spec_from_file_location("task_memory", os.path.join(sys.argv[11], "scripts/hosts/task_memory.py"))
task_memory = importlib.util.module_from_spec(task_memory_spec)   # 这一台能给的内存、这一条拿到的上限与本进程此刻的用量（只在那一份里写）
task_memory_spec.loader.exec_module(task_memory)
MEMORY_KNOB_FILE = ".stage-break-switch-replay-memory"   # 样本根造内存情形与线程数的那一份（这一格文件头「并行度」）
UNREAD_ROW = re.compile(r"^\s*#\s*[A-Z][A-Z0-9_]*_BREAK=(?!<)")   # 注释行头一个词写了 X_BREAK=（这一格文件头「头注读不出判红」）
picked_stages, picked_switches = set(), {}
for item in picked:
    item_stage, separator, item_switch = item.partition(":")
    if separator:
        picked_switches.setdefault(item_stage, set()).add(item_switch)
    else:
        picked_stages.add(item_stage)


def stage_picked(stage):
    return not picked or stage in picked_stages or stage in picked_switches


def switch_picked(stage, switch):
    return not picked or stage in picked_stages or switch in picked_switches.get(stage, ())


def unread_rows(text, entries):
    """头注写了 X_BREAK= 而读法不认的行：[(行号, 那一行)]。"""
    read = {entry["line"] for entry in entries}
    return [(number, row.strip()) for number, row in enumerate(text.split("\n"), 1) if UNREAD_ROW.match(row) and number not in read]


prune_wrapper = os.path.join(stage_repo, "scripts/prune-server/prune_server_client_selftest.py")
# 起 stage-selftest.sh 的共用起法（scripts/gate/lib_process_group.py）：每条放进它自己的会话，退出之后收整组、核会话里没有留下进程；
# 收到 TERM、INT、HUP 时各线程里在跑的那几条各自收组（处理在开池之前于主线程装上）
process_group_spec = importlib.util.spec_from_file_location("lib_process_group", os.path.join(stage_repo, "scripts/gate/lib_process_group.py"))
process_group = importlib.util.module_from_spec(process_group_spec)
process_group_spec.loader.exec_module(process_group)
process_group.install_stop_on_signals(only_defaults=True)
spec = importlib.util.spec_from_file_location("lib_project_code_runs", runs_library_path)
runs_library = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runs_library)
EXEMPT = ".claude/gate.d/stage-break-switch-replay-exempt.tsv"
registry, _ = runs_library.read_selftest_registry(os.path.join(root, ".claude/hooks/selftest-registry.tsv"))
problems, not_run, runs = [], [], []
exempt = {}
if os.path.isfile(EXEMPT):
    for number, line in enumerate(open(EXEMPT, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        columns = line.rstrip("\n").split("\t")
        if len(columns) != 3 or len(columns[2].lstrip("#").strip()) < 4:
            problems.append(f"{EXEMPT}:{number} 不是「<阶段文件名><TAB><开关=值，或 *><TAB># 理由>」（理由至少 4 个字）")
            continue
        exempt[(columns[0], columns[1])] = (number, columns[2].lstrip("#").strip(), [False])
stages = sorted(os.path.basename(path) for path in glob.glob(".claude/gate.d/*.sh") if os.path.isdir(f".claude/gate.d/fixtures/{os.path.basename(path)}"))
for stage in stages:
    whole = exempt.get((stage, "*"))
    if whole:
        whole[2][0] = True
        if stage_picked(stage):
            not_run.append(f"{stage}：整道登记在豁免表（{whole[1]}）")
        continue
    if not stage_picked(stage):
        for (exempt_stage, _switch), mark in exempt.items():
            if exempt_stage == stage:
                mark[2][0] = True   # 没点到的阶段不判它的豁免行（这一道「逐项点名」：点名单跑只判点到的项）
        continue
    row = registry.get(f".claude/gate.d/{stage}")
    if row is None or row.kind != runs_library.STAGE_SAMPLES:
        not_run.append(f"{stage}：样本档{'没登记' if row is None else '登记成 ' + row.kind}（要登记成 samples 或 samples:<假工具目录>），复跑的范围不含它")
        continue
    samples = sorted(name for name in os.listdir(f".claude/gate.d/fixtures/{stage}") if os.path.isdir(f".claude/gate.d/fixtures/{stage}/{name}"))
    stage_text = open(f".claude/gate.d/{stage}", encoding="utf-8", errors="replace").read()
    entries = table.switch_entries(stage_text)
    if broken != "stage-unread-rows-unchecked":
        problems.extend(f"{stage}:{number} 头注写了 *_BREAK= 却读不出（{row[:100]}）：这一条不会被复跑；是一条开关就写成「<变量>=<值>  <弄坏哪一处>」（值后面两个以上空格，或接（…）注），"
                        "只是提到开关的句子不把它放在 # 之后头一个词" for number, row in unread_rows(stage_text, entries))
    if not entries:
        problems.append(f"{stage}：文件头读不出一条「X_BREAK=<值>  <弄坏哪一处>」再下一行起点名样本的开关")
        continue
    for entry in entries:
        switch = f"{entry['name']}={entry['value']}"
        mark = exempt.get((stage, switch))
        if mark:
            mark[2][0] = True
        if not switch_picked(stage, switch):
            continue
        if mark:
            not_run.append(f"{stage} {switch}：登记在豁免表（{mark[1]}）")
            continue
        if table.incomplete(entry):
            problems.append(f"{stage}:{entry['line']} {switch} 缺{'、'.join(table.incomplete(entry))}")
            continue
        runs.append((stage, entry, switch, table.named_tokens(entry, samples), samples))
for (stage, switch), (number, _reason, used) in sorted(exempt.items(), key=lambda item: item[1][0]):
    if not used[0]:
        problems.append(f"{EXEMPT}:{number} {stage} {switch}：那一道的头注里已经没有这一条（或那一道没有样本目录），那一行删掉")



# 按条复用（这一格文件头「按条复用」）：一条判定 = 一道阶段加它的一条开关；键 = 那道阶段的单元闭包（入口 .claude/gate.d/<阶段>）、判法（judge_token）、
# 开关名与值、点名的样本、它喂的样本目录的内容（逐份文件的相对路径、执行位与内容）、喂样本的那几份程序的内容；不带路径、行号与限时
def file_digest(path):
    if not os.path.isfile(path):
        return "不在"
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def samples_digest(stage):
    """fixtures/<阶段>/ 整棵的摘要：每个目录、每份文件（相对路径、执行位、内容）与每个链接（指向）按路径排好连起来取 sha256。"""
    digest, base = hashlib.sha256(), f".claude/gate.d/fixtures/{stage}"
    for directory, subdirectories, files in os.walk(base):
        subdirectories.sort()
        digest.update(f"dir\0{os.path.relpath(directory, base)}\0".encode("utf-8", "surrogateescape"))
        linked = [name for name in subdirectories if os.path.islink(os.path.join(directory, name))]
        for name in sorted(files + linked):
            path = os.path.join(directory, name)
            relative = os.path.relpath(path, base)
            if os.path.islink(path):
                digest.update(f"link\0{relative}\0{os.readlink(path)}\0".encode("utf-8", "surrogateescape"))
            else:
                digest.update(f"file\0{relative}\0{'x' if os.access(path, os.X_OK) else '-'}\0{file_digest(path)}\0".encode("utf-8", "surrogateescape"))
    return digest.hexdigest()


def feeder_digest_now():
    return hashlib.sha256("".join(file_digest(path) for path in (stage_selftest, os.path.join(os.path.dirname(stage_selftest), "lib.sh"), prune_wrapper)).encode()).hexdigest()


feeder_digest = feeder_digest_now()
digest_of = {stage: samples_digest(stage) for stage in sorted({run[0] for run in runs})}


def item_name(stage, switch):
    return f"{stage} {switch}"


def request_line(run):
    stage, _entry, switch, named, _samples = run
    tokens = [judge_token, f"judge-root:{judge_root}", f"line:switch={switch}", "line:named=" + (",".join(sorted(named)) or "（无）"), f"line:feeder={feeder_digest}"]
    if broken != "stage-replay-samples-unkeyed":
        tokens.append(f"line:samples={digest_of[stage]}")
    if broken == "stage-replay-key-whole-gate-directory":
        tokens += [f"file:.claude/gate.d/{other}" for other in stages]
    return "\t".join([item_name(stage, switch), f".claude/gate.d/{stage}", *tokens])


def stage_replay_memory_knobs():
    """样本根的 MEMORY_KNOB_FILE（这一格文件头「并行度」）→ ({键: 值}, 读不出的那一行或 None)：一行一项 capacity=<字节>、hard-cap=<字节>、threads=<正整数>、
    peak:<阶段文件名>=<字节>，值都是正整数；空行与 # 开头的行不算。只在样本根上读；弄坏开关 stage-replay-memory-knob-unchecked：读不出的行悄悄跳过。"""
    knobs = {"peaks": {}}
    if not (sample and os.path.isfile(MEMORY_KNOB_FILE)):
        return knobs, None
    with open(MEMORY_KNOB_FILE, encoding="utf-8", errors="replace") as handle:
        for row in handle.read().split("\n"):
            row = row.strip()
            if not row or row.startswith("#"):
                continue
            key, _separator, value = row.partition("=")
            readable = value.isdigit() and int(value) > 0 and (key in ("capacity", "hard-cap", "threads") or key.startswith("peak:") and len(key) > len("peak:"))
            if not readable:
                if broken == "stage-replay-memory-knob-unchecked":
                    continue
                return knobs, row
            if key.startswith("peak:"):
                knobs["peaks"][key[len("peak:"):]] = int(value)
            else:
                knobs[key] = int(value)
    return knobs, None


def stage_replay_size(byte_count):
    return "取不到" if byte_count is None else task_memory.format_size(byte_count)


def stage_replay_memory_budget():
    """→ (份额字节或 None, 说明)：说明写这一台能给的内存与这一条拿到的上限、各自从哪来（开跑前那一行照写，这一格文件头「并行度」）；
    份额是两样里小的那一个扣掉本进程此刻的用量，两样都取不到交 None，只给弄坏开关 stage-replay-jobs-by-memory 收同时跑的条数用。
    样本根只认 MEMORY_KNOB_FILE 写的（样本的结局不随跑它那台机器变）。"""
    if sample:
        capacity = (knobs["capacity"], f"样本根 {MEMORY_KNOB_FILE} 写的") if "capacity" in knobs else (None, f"样本根 {MEMORY_KNOB_FILE} 没写 capacity")
        hard_cap = (knobs["hard-cap"], f"样本根 {MEMORY_KNOB_FILE} 写的") if "hard-cap" in knobs else (None, f"样本根 {MEMORY_KNOB_FILE} 没写 hard-cap")
    else:
        capacity = task_memory.host_slice_cap_bytes(os.environ)
        hard_cap = task_memory.task_memory_cap_bytes(os.environ)
    told = f"这一台能给的 {stage_replay_size(capacity[0])}（{capacity[1]}）；这一条拿到的上限 {stage_replay_size(hard_cap[0])}（{hard_cap[1]}）"
    limits = [value for value in (capacity[0], hard_cap[0]) if value is not None]
    if not limits:
        return None, told
    return max(1, min(limits) - (task_memory.process_usage_bytes(os.getpid()) or 0)), told


knobs, unreadable_knob = stage_replay_memory_knobs()
if unreadable_knob is not None:
    print(f"  ✗ 样本根的 {MEMORY_KNOB_FILE} 里「{unreadable_knob}」读不出来，一条开关都没起")
    print("     → 怎么办：一行一项：capacity=<字节>、hard-cap=<字节>、threads=<正整数>、peak:<阶段文件名>=<字节>，值都写正整数；判据在 code-tooling.sh 里这一格函数上方的注释「并行度」")
    sys.exit(1)
if "threads" in knobs:
    jobs = knobs["threads"]

requests = {}
for run in runs:
    requests[item_name(run[0], run[2])] = request_line(run)
plan_scratch = tempfile.mkdtemp(prefix="stage-break-switch-replay-plan.")
verdicts = rounds.ItemVerdicts(admission, os.getcwd(), kind, plan_scratch, "code-tooling.sh 的 stage-break-switch-replay 格", sample,
                               "bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay（逐条设上开关经 stage-selftest.sh 喂那一道的样本）",
                               broken != "replay-red-not-recorded")
if requests:
    verdicts.plan(list(requests.values()))
    if verdicts.unavailable:
        shutil.rmtree(plan_scratch, ignore_errors=True)
        print(f"  ✗ 门禁阶段的弄坏开关复跑这一格没判：prune-server 不可用（上面那一句），{len(requests)} 条一条都没起，这一格退 75")
        print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay")
        sys.exit(75)
    if not verdicts.ok:
        print(f"  ! 按条复用算不出（{verdicts.problem}）：这一趟每一条都起、一条都不记库")
eligible = runs
if broken != "replay-plan-ignored":
    runs = [run for run in runs if verdicts.decision(item_name(run[0], run[2]))[0] != rounds.REUSE]
judged_count = sum(1 for run in eligible if verdicts.decision(item_name(run[0], run[2]))[0] == rounds.JUDGE)
refused_count = sum(1 for run in eligible if verdicts.decision(item_name(run[0], run[2]))[0] == rounds.REFUSED)
reused_count = len(eligible) - judged_count - refused_count
if eligible:
    print(f"  · 按条复用：{len(eligible)} 条开关（{len({run[0] for run in eligible})} 道阶段），现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}", flush=True)
    shown = 0
    for run in eligible:
        decision, reason = verdicts.decision(item_name(run[0], run[2]))
        if decision == rounds.REUSE:
            continue
        shown += 1
        if shown <= 20:
            print(f"    {'拒算' if decision == rounds.REFUSED else '现判'}「{item_name(run[0], run[2])}」：{reason}")
    if shown > 20:
        print(f"    …现判与拒算另有 {shown - 20} 条没列")
    if reused_count <= 20 or sample:
        for run in eligible:
            decision, reason = verdicts.decision(item_name(run[0], run[2]))
            if decision == rounds.REUSE:
                print(f"    复用「{item_name(run[0], run[2])}」：{reason}")

tracker = progress_library.ReplayProgress("stage-break-switch-replay", len(runs), progress_seconds, broken == "replay-progress-silent")
stopper = rounds.SampleStop(sample, os.getcwd(), plan_scratch)     # 样本档造中途被停（code_tooling_interrupted_rounds）：那一趟只开一个线程

# 并行度（这一格文件头「并行度」）：同时跑的条数取登记的每台默认重活线程数与要现判的条数里小的，里层 --jobs 取线程数除以它；
# 外层路数 × 里层 --jobs 不超过线程数。内存只写进开跑前那一行，不按它收
budget, limits_text = stage_replay_memory_budget()
memory_text = f"{limits_text}；由服务定、撞了服务照乘 1.5 放大重排，不按它收并行"
max_running = 1 if stopper.armed else max(1, min(jobs, len(runs)))
stages_to_run = sorted({run[0] for run in runs})
# 弄坏开关 stage-replay-jobs-by-memory：同时跑的条数再照份额 ÷ 样本根写的峰值收（几道阶段取最大的那个；没写峰值、两样都取不到的不收）
known_peaks = [knobs["peaks"][stage] for stage in stages_to_run if stage in knobs["peaks"]]
if broken == "stage-replay-jobs-by-memory" and budget is not None and known_peaks:
    max_running = max(1, min(max_running, budget // max(known_peaks)))
    memory_text = (f"{limits_text}；份额 {stage_replay_size(budget)}、每条开关开满时的峰值 {stage_replay_size(max(known_peaks))}，"
                   f"份额容得下 {max_running} 条")
inner_jobs = jobs if broken == "stage-replay-inner-threads-uncapped" else max(1, jobs // max_running)
flight_lock = threading.Lock()
flight = {"now": 0, "most": 0}   # 此刻在跑几条、最多同时在跑几条（收尾那一行照它写）


def replay(run):
    stage, entry, switch, named, samples = run
    environment = {name: value for name, value in os.environ.items()
                   if not name.endswith("_BREAK") and name not in ("GATE_BASE", "GATE_STAGED_FROM", "GATE_DIFF_BASE", "GATE_NOT_RUN_FILE")}
    environment[entry["name"]] = entry["value"]
    with flight_lock:
        flight["now"] += 1
        flight["most"] = max(flight["most"], flight["now"])
    started = time.monotonic()
    with tracker.running(stage, f"{stage} {switch}"):
        cmd = ["bash", stage_selftest, os.path.join(root, ".claude/gate.d"), "--item", stage]
        if broken != "stage-replay-inner-jobs-unset":
            cmd += ["--jobs", str(inner_jobs)]
        if os.path.isfile(prune_wrapper):
            # 二进制只经 --server-binary 交：环境里带来的 PRUNE_SERVER_BINARY 与任务服务给的编译目录不替它作保，这一格在哪台机器上都判同一个结局
            for ambient in ("PRUNE_SERVER_BINARY", "CARGO_TARGET_DIR"):
                environment.pop(ambient, None)
            cmd = ["python3", prune_wrapper, "--with-temporary-service", *(["--server-binary", server_binary] if server_binary else []), "--"] + cmd
        done = process_group.run_in_own_session(cmd, cwd=root, env=environment, stdin=subprocess.DEVNULL)
    seconds = time.monotonic() - started
    with flight_lock:
        flight["now"] -= 1
    output = done.stdout + done.stderr
    leftover = done.problem()
    if leftover:
        problems.append(f"{stage}:{entry['line']} {switch}：stage-selftest.sh {leftover}")
    wrong = judge_library.wrong_samples(output, stage, samples)
    finished = judge_library.run_finished(done.returncode, output)
    result = (stage, entry, switch, named, wrong, finished, done.returncode, output)
    problem = judge_library.problem_of(*result, broken)
    verdict = ("对上" if problem is None else "没对上（点名的样本与实测不符）") if finished else f"没跑成（stage-selftest.sh 退 {done.returncode}）"
    tracker.item_done(verdict, f"{stage} {switch}", seconds)
    # 一条判完就记（这一格文件头「按条复用」）：判绿记绿，判红记红、撤同一把键原有的绿；中途被停，判完的那几条都在库里
    name = item_name(stage, switch)
    if broken != "stage-replay-recorded-at-end":
        verdicts.record([] if problem or leftover else [name], [name] if problem or leftover else [])
    stopper.passed()
    return result

collected = []
if runs:
    inner_text = f"--jobs {inner_jobs}" if broken != "stage-replay-inner-jobs-unset" else "不带 --jobs（照它自己的 STAGE_SELFTEST_JOBS 或核数）"
    print(f"  · stage-break-switch-replay 并行度：同时最多 {max_running} 条（登记的每台默认重活线程数 {jobs} 条；{memory_text}）；每条 stage-selftest {inner_text}；"
          f"外层 {max_running} 条 × 里层 {inner_jobs} = {max_running * inner_jobs}（不超过每台默认重活线程数 {jobs}）", flush=True)
    tracker.start(f"stage-break-switch-replay 开跑：{len(runs)} 条开关（{len(stages_to_run)} 道阶段），同时最多 {max_running} 条；每条判完打一行，每过 {progress_seconds} 秒打一行总进度")
with concurrent.futures.ThreadPoolExecutor(max_workers=max(max_running, 1)) as pool:
    collected = list(pool.map(replay, runs))
tracker.close()
if runs:
    print(f"  · stage-break-switch-replay 同时在跑最多 {flight['most']} 条（同时最多 {max_running} 条）", flush=True)
if len(collected) != len(runs):
    problems.append(f"派出去 {len(runs)} 条复跑，只收回 {len(collected)} 条")
green_names, red_names = [], []
# 点名的与实测不等（跑成了、判法判出不符）的那几条才重算键：跑的过程中键变了的，失败行里多一句（ItemVerdicts.key_changes）
mismatched = [result for result in collected if result[5] and judge_library.problem_of(*result, broken) is not None]
fresh_requests = {}
if mismatched:
    # 键里喂的样本目录与喂样本的程序的摘要是这一格自己算的、不在 verdict-plan 的闭包里，跑的过程中它们也会变：重算的请求按此刻的内容重拼
    feeder_digest = feeder_digest_now()
    digest_of = {stage: samples_digest(stage) for stage in sorted({result[0] for result in mismatched})}
    run_of = {item_name(run[0], run[2]): run for run in runs}
    fresh_requests = {item_name(result[0], result[2]): request_line(run_of[item_name(result[0], result[2])]) for result in mismatched}
key_change_notes = verdicts.key_changes([item_name(result[0], result[2]) for result in mismatched], fresh_requests)
for result in collected:
    problem = judge_library.problem_of(*result, broken)
    name = item_name(result[0], result[2])
    (green_names if problem is None else red_names).append(name)
    if problem is not None:
        problems.append(problem + (f"；{key_change_notes[name]}" if name in key_change_notes else ""))
verdicts.record(green_names, red_names, final=True)
if red_names:
    verdicts.reread_after_red()
verdicts.write_states(states_path, [item_name(run[0], run[2]) for run in eligible])
shutil.rmtree(plan_scratch, ignore_errors=True)
for line in not_run:
    print(f"  ! 本次未跑：{line}")
    if not_run_file and os.path.exists(not_run_file):
        with open(not_run_file, "a", encoding="utf-8") as handle:
            handle.write(f"{own_stage} stage-break-switch-replay：{line}\n")
if not stages:
    print("  ! .claude/gate.d/ 顶层一道有样本目录的阶段都没有，这一格无对象可判")
    sys.exit(77)
replayed_stages = sorted({stage for stage, *_rest in collected})
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 门禁阶段的弄坏开关复跑：{len(problems)} 处不对（复跑了 {len(collected)} 条、{len(replayed_stages)} 道阶段；本次未跑 {len(not_run)} 项）")  # gate-lint:summary
    print("     → 怎么办：点名的与实测不等的，照实测改那一道文件头那一条下面点名的样本（实测是对的才改头注；样本判错是因为判法坏了就修判法），"
          f"一时改不准的登记进 {EXEMPT}（只缩不涨）；头注还不是 mutate.sh 写法的整道登记 *；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
if verdicts.unavailable:
    print("  ✗ 门禁阶段的弄坏开关复跑：判完记库时 prune-server 不可用（上面那一句），这一趟的判定没记下，这一格退 75")
    print("     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再跑这一格：bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay")
    sys.exit(75)
if verdicts.red:
    print(f"  ✗ 门禁阶段的弄坏开关复跑判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；起了的 {len(collected)} 条都对上")
    print("     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再跑这一格：bash .claude/gate.d/code-tooling.sh --check stage-break-switch-replay；不能复用就判红，不退回全部现判当绿")
    sys.exit(1)
if not eligible:
    print(f"  ! 一条开关都没复跑（本次未跑 {len(not_run)} 项），这一格无对象可判")
    sys.exit(77)
reuse_text = f"按条复用：现判 {judged_count}、复用 {reused_count}、拒算 {refused_count}" if verdicts.ok else "按条复用算不出、全部现判"
if not collected:
    print(f"  ✓ 门禁阶段的弄坏开关复跑：点到的 {len(eligible)} 条都有键相同的判定，全部复用，这一格一条都没起（{reuse_text}）；本次未跑 {len(not_run)} 项（列在上面）")
    sys.exit(0)
print(f"  ✓ 门禁阶段的弄坏开关复跑：复跑了 {len(collected)} 条开关（{len(replayed_stages)} 道阶段：{'、'.join(replayed_stages)}），判错的样本与文件头点名的逐条相等；本次未跑 {len(not_run)} 项（列在上面）；{reuse_text}")
PY
}

# ── 格 break-switch-outside-selftest：还 checks-owed.md C622（门禁的弄坏开关平常跑也认）研究脚本与钩子那一半。
# 判据：被判的根里 scripts/ 与 .claude/hooks/ 下（fixtures 目录除外）的 .sh、.py，代码里读了名字以 _BREAK 结尾的变量的（读法同 break-switch-headers 格，注释行不算），
#   平常跑（不在自证里）时设上这个变量要被拒：脚本退非 0，且输出里有一行点名这个变量、说它只在自证里认（含「自证」或「selftest」）。
#   这一格逐份、逐个变量起一次探针：env <变量>=<文件头开关表里这个变量的第一个值（不取关守卫项，见下），没有表写 probe> bash|python3 <脚本> --force --break-guard-probe-unknown-option，
#   stdin 接空，最多 30 秒；那个选项没有哪份脚本认得，没有守卫的脚本只会照常跑或退用法错，说不出「只在自证里认」。
#   关守卫项（开关表里值是 guard-off 或 honor-outside-selftest、或说明里同时写着「平常跑」与「不拒」的那一项：它弄坏的就是守卫本身，单独设时平常跑照认）
#   只在它是唯一一项时认：这个变量的开关表里有关守卫项的，另起一次探针 env <变量>=<关守卫项>,<第一个值>（两次探针的第一个值都取表里第一条不是关守卫项的值，没有写 probe），
#   照样要被拒并点名（拆分提案第一百二十二节 ②：按逗号拆开、里面有关守卫项就放行的脚本，把关守卫项和别的项同写，别的开关平常跑也借道生效）。
#   还没改成这样的存量逐份登记在 .claude/gate.d/break-switch-outside-selftest-pending.tsv（「<相对仓根的路径><TAB># 理由>」），登记了的不起探针（探针对没有守卫的脚本
#   不保证无害，所以登记着的不探）；一行指到的文件不在、或代码里不读弄坏开关了，判红：那一行删掉。表只缩不涨（不判涨，靠 review）；改好一份删一行。
# 管不到的：守卫放在读完参数之后的脚本（探针的选项在那之前就被当用法错拒了，要先把守卫挪到参数解析之前，或登记成存量）；没有命令行入口、被别的程序 import 的库也算存量。
# 一份读开关的都没有：退 77。
# 样本：break-switch-outside-selftest-red（一份读开关、平常跑照常退 0，一份读开关、退 2 却不点名变量，一份守住的，登记表一行指到不在的文件、一行指到不读开关的，逐条点名判红）；
#   break-switch-outside-selftest-green（一份守住的，一份没守住而登记在存量表里，一份不读开关，报「读开关的 2 份：探了 1 份……待改 1 份」）。
#   break-switch-outside-selftest-guard-combined-red（一份按逗号拆开、里面有 guard-off 就放行，一份只认「平常跑」「不拒」写在说明里的关守卫项、同样拆开放行：
#   第一个值单独设都被拒，「关守卫项,第一个值」两份都没被拒，必须各点名一次）；break-switch-outside-selftest-guard-combined-green（两份关守卫项都只在唯一一项时认，
#   一份关守卫项排在表里第一条：第一个值取它后面那一条，报「关守卫项与第一个值同写的探针 2 次」）。
# 弄坏开关 CODE_TOOLING_BREAK=outside-selftest-unchecked：探针的结果一概当守住，break-switch-outside-selftest-red 与 break-switch-outside-selftest-guard-combined-red 里「没被拒」那几条 want 找不到。
# 弄坏开关 CODE_TOOLING_BREAK=guard-combination-unprobed：不起「关守卫项,第一个值」那一次探针，break-switch-outside-selftest-guard-combined-red 判绿（期望退出 1，实测 0），
#   break-switch-outside-selftest-guard-combined-green 里「关守卫项与第一个值同写的探针 2 次」那条 want 找不到（探了 0 次）。
# 拆分族（写法与认法在 BREAK_SWITCH_HEADER_LIBRARY 的 SplitFamilies）：入口拆出的部分文件（同目录的 <名>_<部分>.py、<名>-<部分>.sh、<名>-<部分>.py，门禁阶段的 lib/ 下 <阶段名>-<部分>.sh，
#   且入口的代码里提到它）不单独探：部分文件没有命令行入口，起它得到的是用法错或 Traceback，说不出「只在自证里认」。探针探入口，变量取入口加部分文件的代码读的全部，
#   值取整族（入口加部分文件）的文件头开关表里这个变量的值（表里没有的写 probe）；入口平常跑设了变量被拒并点名的照旧判。待改存量表只登记入口，登记了部分文件的点名删那一行。
#   复跑登记表点名的文件各自是入口，不当别人的部分文件；入口的代码里没提到的同族命名文件是独立的脚本，照单份探。
#   样本 break-switch-outside-selftest-split-family-green（入口 scripts/gate/divided.sh 自己不读开关、平常跑设了 DIVIDED_BREAK 照部分文件的守卫退 2 并点名只在自证里认；source 的 divided-core.sh 读开关；
#   同目录 divided-extra.sh 入口没提到、自己读 EXTRA_BREAK 且守住：报「读开关的 2 份：探了 2 份」，部分文件 divided-core.sh 不探）。
#   弄坏开关 CODE_TOOLING_BREAK=outside-selftest-split-family-unread：每份各探各的，divided-core.sh 也被当成脚本起来探、source 的库平常跑没守卫，break-switch-outside-selftest-split-family-green 判红
#   （期望退出 0，实测 1：「scripts/gate/divided-core.sh 读了 DIVIDED_BREAK：平常跑时设上它…没被拒」）；=outside-selftest-split-family-by-name：部分文件只按命名认，divided-extra.sh 被当成部分文件、
#   不单独探，「读开关的 2 份：探了 2 份」那条 want 找不到。
cell_break_switch_outside_selftest() {
  local pending=".claude/gate.d/break-switch-outside-selftest-pending.tsv" jobs
  if ! jobs="$(heavy_thread_default)"; then
    echo "  ✗ 取不了每台默认重活线程数（原因在上一行），探针的并行度定不了，一份都没探"
    echo "     → 怎么办：照上一行的出路修 .claude/heavy-thread-default（线程固定取登记值，SINGLEFS_THREAD_CAP 由服务套的 capped.sh 设，不自己设），再跑这一格；不退回按核数算。"
    exit 1
  fi
  python3 - "$BREAK_SWITCH_TABLE_LIBRARY" "$pending" "${CODE_TOOLING_BREAK:-}" "$STAGE_REPOSITORY/scripts/gate/lib_process_group.py" "$TREE_LISTING_LIBRARY" "$jobs" \
    "$BREAK_SWITCH_HEADER_LIBRARY" "$STAGE_REPOSITORY/scripts/gate/lib_lag_split_family.py" <<'PY'
import concurrent.futures, glob, importlib.util, os, subprocess, sys, types
tree = types.ModuleType("lib_tree_listing")
exec(compile(sys.argv[5], "tree-listing-library", "exec"), tree.__dict__)
table = types.ModuleType("lib_break_switch_table")
exec(compile(sys.argv[1], "break-switch-table-library", "exec"), table.__dict__)
exec(compile(sys.argv[7], "break-switch-header-library", "exec"), table.__dict__)   # 读了哪几个开关变量、拆分族的认法：与 break-switch-headers 格同一份
pending_path, broken = sys.argv[2:4]
process_group_spec = importlib.util.spec_from_file_location("lib_process_group", sys.argv[4])
process_group = importlib.util.module_from_spec(process_group_spec)
process_group_spec.loader.exec_module(process_group)
# 拆分族（这一格文件头「拆分族」）：复跑登记表点名的文件各自是入口；弄坏开关 outside-selftest-split-family-unread 下每份各探各的，outside-selftest-split-family-by-name 下部分文件只按命名认
families = table.SplitFamilies(table.load_split_family(sys.argv[8]), independent=table.replay_registry_paths(table.REPLAY_REGISTRY_PATH),
                               alone=broken == "outside-selftest-split-family-unread", by_name=broken == "outside-selftest-split-family-by-name")
PROBE_ARGUMENTS = ["--force", "--break-guard-probe-unknown-option"]
GUARD_OFF_VALUES = ("guard-off", "honor-outside-selftest")


def guard_off_value(entry):
    """这一条开关是不是关守卫项：值是 GUARD_OFF_VALUES 之一，或说明里同时写着「平常跑」与「不拒」。"""
    if entry["value"] in GUARD_OFF_VALUES:
        return True
    found = table.SWITCH_ENTRY.match(families.text_of(entry["file"]).split("\n")[entry["line"] - 1])
    rest = found.group("rest") if found else ""
    return "平常跑" in rest and "不拒" in rest


def probe(path, interpreter, name, value, found):
    """起一次探针 → (退出码或「超时」, 输出)。经共用起法 scripts/gate/lib_process_group.py 起：放进它自己的会话，超时与退出之后收整组，
    留下的进程另记一处问题（记进 found，这一格照红）。各份脚本的探针彼此独立，几份同时探，问题按脚本的次序汇总。"""
    done = process_group.run_in_own_session(["env", f"{name}={value}", *interpreter, path, *PROBE_ARGUMENTS], stdin=subprocess.DEVNULL, timeout=30)
    if done.problem():
        found.append(f"{path} 读了 {name}：探针（env {name}={value} … {' '.join(PROBE_ARGUMENTS)}）{done.problem()}")
    return ("超时" if done.timed_out else done.returncode), done.stdout + done.stderr


def refused(name, code_out, output):
    named = [row.strip() for row in output.split("\n") if name in row and ("自证" in row or "selftest" in row.lower())]
    return code_out not in (0, "超时") and bool(named)


def why_not(name, code_out, output):
    first = output.strip().split("\n")[0][:100] if output.strip() else "（没有输出）"
    return "退 0，照常跑" if code_out == 0 else ("超过 30 秒没返回" if code_out == "超时" else f"退 {code_out} 但输出里没有点名 {name} 只在自证里认的一行（首行：{first}）")


paths = sorted(path for directory in ("scripts", ".claude/hooks") for path in tree.files_under(directory, (".sh", ".py"), skipped=("target", "fixtures")))
problems, pending = [], {}
if os.path.isfile(pending_path):
    for number, line in enumerate(open(pending_path, encoding="utf-8"), 1):
        if not line.strip() or line.startswith("#"):
            continue
        path, _, reason = line.rstrip("\n").partition("\t")
        if len(reason.lstrip("#").strip()) < 4:
            problems.append(f"{pending_path}:{number} 不是「<路径><TAB># 理由>」（理由至少 4 个字）")
            continue
        pending[path] = number
readers, probed, waiting, combined_probes = {}, [], [], 0
for path in paths:
    if families.entry_of(path) != path:
        continue   # 部分文件读的开关，由所属入口那一份探
    names = families.switch_names(path)
    if names:
        readers[path] = names
def probe_reader(path, names):
    """一份读了开关的脚本（入口加它的部分文件读的变量）：逐个变量起探针（设值、关守卫项同写各一次）→ (这一份的问题们，同写探针的次数)。"""
    found, combined_count = [], 0
    interpreter = ["python3", "-B"] if path.endswith(".py") else ["bash"]
    entries = families.family_entries(path)
    for name in names:
        # 第一个值取表里第一条不是关守卫项的（关守卫项单独设时平常跑照认，拿它探守卫探不出东西），没有表或只有关守卫项写 probe
        guards = [entry["value"] for entry in entries if entry["name"] == name and guard_off_value(entry)]
        value = next((entry["value"] for entry in entries if entry["name"] == name and not guard_off_value(entry)), "probe")
        code_out, output = probe(path, interpreter, name, value, found)
        if broken != "outside-selftest-unchecked" and not refused(name, code_out, output):
            found.append(f"{path} 读了 {name}：平常跑时设上它（env {name}={value} … {' '.join(PROBE_ARGUMENTS)}）没被拒，{why_not(name, code_out, output)}")
        # 关守卫项与别的项同写（这一格文件头「关守卫项」一段）：另起一次探针，照样要被拒
        if not guards or broken == "guard-combination-unprobed":
            continue
        combined = f"{guards[0]},{value}"
        combined_count += 1
        code_out, output = probe(path, interpreter, name, combined, found)
        if broken != "outside-selftest-unchecked" and not refused(name, code_out, output):
            found.append(f"{path} 读了 {name}：关守卫项与别的项同写（env {name}={combined} … {' '.join(PROBE_ARGUMENTS)}）没被拒，{why_not(name, code_out, output)}"
                         f"——关守卫项 {guards[0]} 只在它是唯一一项时认，与别的项同写要照拒并点名")
    return found, combined_count
to_probe = []
for path, names in readers.items():
    if path in pending:
        waiting.append(path)
        continue
    to_probe.append((path, names))
# 各份脚本的探针彼此独立（各自一个会话、各自的输出）：按每台默认重活线程数同时探，问题按脚本的次序汇总，次序与逐份串行探时相同
process_group.install_stop_on_signals(only_defaults=True)   # 探针在池里的线程上，停止信号的处理要先在主线程装好
if to_probe:
    with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, min(int(sys.argv[6]), len(to_probe)))) as pool:
        outcomes = list(pool.map(lambda task: probe_reader(*task), to_probe))
    for (path, _names), (found, combined_count) in zip(to_probe, outcomes):
        problems.extend(found)
        combined_probes += combined_count
        probed.append(path)
for path, number in sorted(pending.items(), key=lambda item: item[1]):
    if not os.path.isfile(path):
        problems.append(f"{pending_path}:{number} 指到的 {path} 不在，那一行删掉")
    elif families.entry_of(path) != path:
        problems.append(f"{pending_path}:{number} {path} 是 {families.entry_of(path)} 的部分文件，探针探入口，存量登记只登记入口，这一行删掉")
    elif path not in readers:
        problems.append(f"{pending_path}:{number} {path} 代码里不读弄坏开关了，那一行删掉")
if not readers and not problems:
    print("  ! scripts/ 与 .claude/hooks/ 里没有读弄坏开关的脚本，这一格无对象可判")
    sys.exit(77)
if problems:
    for problem in problems:
        print(f"  ✗ {problem}")  # gate-lint:detail
    print(f"  ✗ 平常跑设弄坏开关：{len(problems)} 处不对（读开关的 {len(readers)} 份：探了 {len(probed)} 份、待改 {len(waiting)} 份）")  # gate-lint:summary
    print("     → 怎么办：让脚本在参数解析之前先看这个变量：平常跑（不是 --selftest，也不是自证起的子进程）设了就退 2，输出里点名变量、说它只在自证里认"
          "（样子照 scripts/experiment-apparatus/e180-reading-shards.sh、scripts/mutation/mutate.sh）；关守卫项（guard-off 这类）整值比，只在它是唯一一项时放行"
          "（样子照 .claude/gate.d/lib/stage-cells.sh 的 break-honored-everywhere），与别的项同写照拒并点名；一时改不完的存量登记进 "
          f"{pending_path}（只缩不涨），改好就删那一行；判据在 code-tooling.sh 里这一格函数上方的注释")
    sys.exit(1)
print(f"  ✓ 平常跑设弄坏开关：读开关的 {len(readers)} 份：探了 {len(probed)} 份（都被拒并点名变量只在自证里认）、待改 {len(waiting)} 份（{pending_path}，只缩不涨）；关守卫项与第一个值同写的探针 {combined_probes} 次，也都被拒")
PY
}
