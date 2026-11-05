"""内存峰值表、取键与「这台此刻还放得下多少」的共用库：scripts/hosts/run-with-memory-cap.sh（内存包装，写峰值表）、
scripts/hosts/host-dispatch.py（task-server 载入的余量判法）与 task-server 都 import 它，取键、读峰值表、算余量只在这一份里写。

被 import 的库，除自证（python3 scripts/hosts/memory_peaks_lib.py --selftest）之外不单独跑、不写准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md「不判的三类」）；
import 本身不起子进程、不开文件。用法（脚本按路径跑时 sys.path[0] 就是这个目录；bash 里写 python3 -B -c 'import sys; sys.path.insert(0, <目录>); …'）：
    import memory_peaks_lib as peaks
    key = peaks.command_key(命令的各个词, 环境里的 RUN_WITH_MEMORY_CAP_KEY)
派发器把这份源码整份送到第二台跑（python3 -c），在那边算那台的余量：所以这里只用标准库，不 import 同目录的别的文件；
  文件末尾的自证入口只在按文件路径跑（有 __file__）、参数正好是 --selftest 时起，经 python3 -c 送过去的那一份不起。

键（与峰值表第四列相同）：给了 RUN_WITH_MEMORY_CAP_KEY 就用它（一段文本，按空格切成词）；不给就取命令的各个词（词表原样，一个词里的空格留在词里，不切），去掉测试二进制名里 -<16 位十六进制> 的哈希；
  两种都把制表符、换行换成空格，再按词表换算（normalized_words，下面四条），最后用空格接成键，截到 KEY_LENGTH_LIMIT 个字。
  ① 剥掉线程包装 capped.sh N：词的文件名是 capped.sh、后面那个词不以 - 开头、再后面还有词的，这两个词连同紧挨在前面的 shell 名（bash、sh 这类）一起去掉
    （与 .claude/hooks/lib_shell_words.py 剥 WRAPPERS_TAKING_ONE_ARGUMENT 同一认法：N 写数还是 default 都剥；不看是不是在命令位置上，峰值表里没有 capped.sh 当参数的键）。
    经 capped.sh 起与不经起的同一条命令落一个键：task-server 给每一次尝试套 capped.sh <N>，内存包装记的键与服务按提交的那条命令取的键对得上。
  ② 本机路径换成固定记号：工作根（取法同 .claude/hooks/lib_work_root.py 的 scratch_roots：SINGLEFS_WORK_ROOT，没设是数据根下的 work，数据根是 SINGLEFS_DATA_ROOT，
    没设照那份库读多机配置的 SHARED_DATA_ROOT（configured_data_root：按本库所在的仓根载入那份库调 shared_data_root，不另写一份读法；不经 ~ 取，第二台的家目录是另一个）；
    三样都没有、或经 python3 -c 整份送去跑而载不到那份库时不换工作根（没有就是没有，不补家目录下的缺省），自证里核两边取到的一样）下面那一段——路径里有 /target/ 的，
    根到 target 之前那一截（副本与草稿目录，多深都算）；没有的，根下第一层目录——换成 SCRATCH_TOKEN；仓根（本库在 <仓根>/scripts/hosts/ 下）换成 REPOSITORY_TOKEN；
    家目录换成 HOME_TOKEN。先换工作根、再换仓根、最后换家目录（前两样都在家目录底下）。路径要从词首、空白、= : 引号逗号左括号之后起才认。
    两份副本、两个草稿目录里的同一条命令落一个键；从工作根下的副本里 import 这份库时，它认的仓根是那份副本（已被工作根那一条换掉），主仓的路径只换得到家目录那一截。
  ③ 点名的条目、文件参数与弄坏开关的值换成固定记号（只差这几样的同一条命令落一个键，不各记一条、各从缺省额度试起）：
    照词表换，不按空格切：带空格的条目名、路径仍是一个词（条目名里的空格不会把后半截漏进键里），只有 -c 后面那一段脚本文本按空格切开、照一条命令再认；
    shell 分隔词（&& || ; |）这一个词把词表切成几条命令，逐条换；换完之后同类相邻的并成一个（「〈文件〉 〈文件〉」并成一个，
    「--item 〈条目〉 --item 〈条目〉」并成「--item 〈条目〉」）。
    条目（ITEM_TOKEN）：ITEM_OPTIONS（--item、--only、--skip、--exact）后面那个不以 - 起头的词，与写成「--item=<值>」的（换成「--item 〈条目〉」两个词）；
    格名（CELL_OPTIONS，--check）：门禁按格点名，格是一道门禁里有限的几个名字，用量差得远（一格 85 MiB、另一格 16.6 GiB），所以后面那个不以 - 起头的词（逗号连起来的几格整段）照留进键，
      不同的格或格的组合是不同的键；写成「--check=<值>」的换成「--check <值>」两个词（两种写法落一个键）；值是路径形态的当文件换成〈文件〉；
      cargo test / cargo nextest 的测试名筛选词（子命令之后、不是选项也不是选项值的位置参数，-- 之前与之后都算；nextest 的 run 照留）；
      环境变量里点名格与条目的（名字以 ITEM_VARIABLE_SUFFIXES 结尾、或在 ITEM_VARIABLES 里）的值。
    文件（FILE_TOKEN）：照留的那一个脚本或可执行文件之后的位置参数里写成路径形态的（path_shaped：带 /、以 FILE_EXTENSIONS 结尾、或以②的三个记号起头；
      以 - 起头、带 = 的不算）。照留的是命令词本身；命令词是解释器（INTERPRETER_NAMES）的照留它后面第一个不以 - 起头的词（-c 后面那段代码照一条命令再认，-m 后面的模块照留），
      是前缀命令（PREFIX_COMMANDS：env、nice、timeout 这类，连同它们的选项、选项值与 NAME=值）的照它后面那条命令再认。
    弄坏开关（BREAK_VALUE_TOKEN）：名字以 _BREAK 结尾的环境变量的值（逗号连起来的几项一起换），变量名照留。
    不换的：命令词、照留的脚本、子命令（cargo test 的 test）、选项本身、KEPT_VALUE_OPTIONS（-p、--test、--features、--manifest-path 这类）的值、
      cargo test 的 -- 之后 LIBTEST_VALUE_OPTIONS（--test-threads 这类）的值、--item 这类范围开关、不是路径形态的别的位置参数（数、名字）：这些决定跑什么、要多少内存。
  ④ 换算是幂等的：换过的键再换一次不变。
峰值表的口径与列见 run-with-memory-cap.sh 文件头「峰值表」一段；这里只读不写（写在包装那一边，表旁的 <表>.lock 那把锁里）。
  峰值表在哪（default_peak_table）：共享数据根下的 memory-peaks.tsv（.claude/hooks/lib_work_root.py 的 shared_data_path(PEAK_TABLE_NAME)），两台读写同一份，
    不经 ~、$HOME 取（第二台的家目录是另一个）；取不到（载不到那份库、数据根取不到、那份库不认这个名字）抛 PeakTableUnavailable，不退回家目录下的那一份。
    RUN_WITH_MEMORY_CAP_PEAKS 给了就用它指的那一份（自证与别的脚本的假世界换路径用）。
  哪一台（第七列，this_host）：这一台在多机配置主机数组 HOSTS 里的地址，照 scripts/hosts/multi_host_addresses.py 的 resolve（照这一台网卡上的地址认，判法同一份）取，
    一个进程取一次；取不到抛 PeakTableUnavailable。量的数各台各算：读的时候只认本台的行（read_peak_rows、recorded_peak），另一台的行原样留着、不拿来预测。
  读表只认这一版的行：七列（峰值字节、量的时刻、那一次的上限、键、线程数、近几次、哪一台），近几次每一次写成「<字节>:<hit|clean>:unreclaimable」；键按表里写的原样取，不换算。
  行不是这个形态的，整张表读不了：read_peak_rows 抛 TableRowRefused，报哪个文件、第几行、要几列，出路是删掉那一行（下次跑这条命令会重新记）或把整张表删了重记，不按列数分别认。
  「近几次」（第六列，至多 RECENT_MEASUREMENTS 次）的读法：recorded_peak 取其中最大的，其中有一次撞顶就算撞过顶（latest_only 只取最近一次）；
  读它的：scripts/mutation/crates-mutation-rows.py 的每格峰值与 E165（恢复重放有界穷举与变异杀伤界）的装置经 read_peak_rows，
  E197（扫环胶水内存随盘数）的驱动经 read_peak_rows 与 recorded_peak，不各写一份。task-server 不读峰值表定额度（照它自己库里的记录，task_server_lib.py 文件头「资源记录」）。
口径（第六列里每一次）：「<字节>:<hit|clean>:unreclaimable」是回收不掉的用量的峰值（包装跑的过程中每 PEAK_SAMPLE_SECONDS 秒采一次 unreclaimable_bytes 取最大，
  见 sample_unreclaimable_peak；撞顶只认 scope 的 memory.events 里 oom_kill 大于 0 或包装判的撞顶结局，见 judged_measurement）。
  记进表的这一次由内存包装经 judged_measurement 定（它也只在这一份里写），元素是 (峰值字节, 撞没撞顶, MEASURED_UNRECLAIMABLE)。
余量（headroom 交的 MARGIN_REPORT_KEY 那三项；task-server 拿它定每台单条的上限 = 那台 slice 总上限 − 这台此刻的余量，records/2026-10-31-task-server巡检.md 问题 36）：
  是什么：这台此刻 slice（DEFAULT_SLICE）里回收不掉、又不归任何在跑任务的内存——slice 自己记着、哪个子 cgroup 里都没有的那一截
    （已退出任务留下、内核还记在 slice 上的不可回收内存这一类：cgroup v2 的 memory.current 连同全部子孙记账，子 cgroup 删了之后它留下的 slab、页表这类
    挂回 slice（还挂在退出中的子 cgroup 上的也照算在 slice 的 memory.current 里），子 cgroup 的目录里已经没有它们）。
  怎么量（slice_margin_bytes）：回收不掉的用量照 unreclaimable_bytes 算（memory.current 减 memory.stat 的 active_file 与 inactive_file，剩 anon + shmem + kernel + sock），
    余量 = slice 的那一份 − slice 目录下每个子 cgroup（在跑任务的 scope）各自那一份之和，不小于 0；先读子 cgroup、后读 slice。
    页缓存（file 里 active_file、inactive_file 那两项，连同退出了的任务留下的）回收得掉，不算。
  取整（rounded_up_to_gibibyte）：按 1 GiB 向上取整：1.01G、1.1G 记 2G，2.1G 记 3G，0.7G 记 1G，恰好整数的（2.0G、0）不进位。
  量不到：slice 或它的子 cgroup 在而 memory.current / memory.stat 读不了、列不了子 cgroup、查不了用户级 systemd（slice_directory_of 交 None）——MARGIN_REPORT_KEY 交 None、
    原因写进 MARGIN_PROBLEM_KEY，不退回任何固定的数；slice 还没起（systemctl 答了、ControlGroup 是空的）或刚停（目录不在）量到 0。
  第二台：派发器把这份源码送过去跑 headroom（host-dispatch.py 的 probe_peer），量的是第二台自己的 slice，同一条路、同一个算法；每次读余量都现量，不存。
在跑的 scope（headroom 交的 SCOPES_REPORT_KEY；task-server 照它算每台每条在跑尝试还没涨到量的那一截，两台同一段，task_server.py 的 server_pending）：
  slice 下每个子 cgroup 一项，键是目录名（scope 的单元名），值是 {SCOPE_USED_KEY: 它此刻回收不掉的用量（unreclaimable_bytes，读不了是 None）, SCOPE_LABEL_KEY: 标签}。
  标签（scope_run_label）：照这个 cgroup 的 cgroup.procs 逐个读 /proc/<进程号>/environ，头一个带 RUN_LABEL_VARIABLE 的取那个值；都没有、读不了是空串。
    两台的内存包装开的 scope 里，进程的环境都带着 task-server 给这一次尝试设的标签（本机设在尝试的环境里，第二台经 multi-host-run.sh --peer 那一段 env 带过去），
    服务照它把 scope 对到尝试上；不是 task-server 起的 scope 标签是空串。
  slice 没起、目录不在、列不了子目录交空表：服务照「探测报告里没有这一条的 scope」记整份上限，不记 0。每次读余量都现量，不存。
"""
import contextlib
import os
import re
import subprocess
import sys
import time

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
DATA_ROOT_VARIABLE = "SINGLEFS_DATA_ROOT"                # 没设工作根时取数据根下的 work，数据根的取法同 lib_work_root.py 的 shared_data_root
FIXED_DATA_ROOT = "/var/tmp/singlefs-data-root-not-from-configuration"   # 只给弄坏开关 data-root-home-fallback：不读多机配置、退回一个写死的目录
PEAK_TABLE_NAME = "memory-peaks.tsv"                     # 共享数据根下峰值表的名字（.claude/hooks/lib_work_root.py 文件头「数据根」那张表；文件头「峰值表在哪」）
WORK_ENTRY = "work"
PATH_START = r"(?<![^\s=:'\",(])"                        # 路径从词首、空白、= : 引号逗号左括号之后起才认
# 文件头「键」③：点名的条目、文件参数、弄坏开关的值换成的固定记号与认法
ITEM_TOKEN = "〈条目〉"
FILE_TOKEN = "〈文件〉"
BREAK_VALUE_TOKEN = "〈开关〉"
FOLD_TOKENS = (ITEM_TOKEN, FILE_TOKEN, BREAK_VALUE_TOKEN)
ITEM_OPTIONS = ("--item", "--only", "--skip", "--exact")                     # 后面那个词（或 = 之后）是点名的条目，换成 ITEM_TOKEN
CELL_OPTIONS = ("--check",)                                                    # 后面那个词（或 = 之后）是点名的门禁格名，照留进键（路径形态的换成 FILE_TOKEN）
ITEM_VARIABLE_SUFFIXES = ("_SELFTEST_CELLS",)                                  # 环境变量名以它结尾的，值是点名的格
ITEM_VARIABLES = ("GATE_MUTATION_ROW_SELECTION",)                              # 环境变量名照写的，值是点名的条目
BREAK_VARIABLE_SUFFIX = "_BREAK"                                               # 环境变量名以它结尾的，值是弄坏开关的项
FILE_EXTENSIONS = (".md", ".tsv", ".json", ".txt", ".log", ".rs", ".py", ".sh", ".patch")
INTERPRETER_NAMES = SHELL_NAMES | {"python", "python3", "perl", "ruby", "node"}
VERSIONED_PYTHON = re.compile(r"^python3\.\d+$")
PREFIX_COMMANDS = {"env", "nice", "timeout", "setsid", "stdbuf", "ionice", "taskset", "time", "exec", "command"}
PREFIX_VALUE_OPTIONS = {"-u", "--unset", "-C", "--chdir", "-n", "--adjustment", "-k", "--kill-after", "-s", "--signal", "-c", "--class"}
PREFIX_DURATION = re.compile(r"^\d+(\.\d+)?[smhd]?$")                          # timeout 的时长
# 值照留的选项（决定跑什么）：cargo 的这些，别的命令里同名的也照留
KEPT_VALUE_OPTIONS = {"-p", "--package", "--test", "--bin", "--example", "--bench", "--features", "-F", "--manifest-path", "--target",
                      "--target-dir", "--profile", "--config", "-j", "--jobs", "--exclude", "--message-format", "--color", "-Z"}
LIBTEST_VALUE_OPTIONS = {"--test-threads", "--format", "--color", "--shuffle-seed", "-Z", "--logfile"}   # cargo test 的 -- 之后值照留的
CARGO_TEST_SUBCOMMANDS = ("test", "nextest")
SHELL_SEPARATORS = {"&&", "||", ";", "|"}
ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
# 只给自证判红用的弄坏开关（MEMORY_PEAKS_LIB_BREAK，只在 --selftest 里读进来；别处 import 这份库时一律不认，见文件末尾「自证」）
BREAK_VARIABLE = "MEMORY_PEAKS_LIB_BREAK"
KNOWN_BREAK_MODES = ("scratch-paths-kept", "capped-kept", "outside-selftest-unnamed", "record-memory-peak", "hit-by-peak", "kernel-counted",
                     "item-and-file-arguments-kept", "row-shape-unchecked", "data-root-home-fallback", "other-hosts-read", "peak-table-per-host",
                     "margin-floored", "margin-whole-carried", "margin-children-counted", "margin-unreadable-as-zero", "scope-label-unread",
                     "item-words-split", "check-names-folded")
PEAK_TABLE_COLUMNS = 7       # 峰值表一行：峰值字节、量的时刻、那一次的上限、键、线程数、近几次、哪一台（文件头「哪一台」）
_break_modes = frozenset()
_path_replacements_cache = {}   # path_replacements 建好的那一份（键是此刻的 SINGLEFS_WORK_ROOT、HOME 与弄坏开关）
SIZE_UNITS = {"K": 1024, "M": MEBIBYTE, "G": 1024 * MEBIBYTE, "T": MEBIBYTE * MEBIBYTE}
# 峰值表第六列「近几次」记最近这么多次实测（run-with-memory-cap.sh 文件头「峰值表」一段）：撞顶之后要连着这么多次干净跑完才回到按峰值记账
RECENT_MEASUREMENTS = 5
RECENT_HIT, RECENT_CLEAN = "hit", "clean"   # 第六列里每一次的标记：撞没撞顶
MEASURED_UNRECLAIMABLE = "unreclaimable"   # 第六列里每一次的第三段（文件头「口径」）
# 撞了顶的结局（内存包装 run_capped 交给 record_run 的）：own-cap 撞了自己的上限（scope 的 Result=oom-kill 而不是被总上限挤掉）；记的峰值是那一次的上限（真峰值不小于它）
HIT_OUTCOMES = ("own-cap",)
# 内存包装跑命令的过程中隔多久采一次 scope 回收不掉的用量（sample_unreclaimable_peak；run-with-memory-cap.sh 文件头「峰值表」一段）。
# 比这还短的尖峰可能采不到；每条命令一个采样进程、读两份小文件
PEAK_SAMPLE_SECONDS = 0.1
DEFAULT_SLICE = "singlefs-heavy.slice"
GIBIBYTE = 1024 * MEBIBYTE
# 余量（文件头「余量」）：headroom 交的 dict 里这三项
MARGIN_REPORT_KEY = "slice_margin"              # 这台此刻的余量（字节，按 GIBIBYTE 向上取整）；量不到是 None
MARGIN_MEASURED_KEY = "slice_margin_measured"   # 量到的字节（取整之前）；量不到是 None
MARGIN_PROBLEM_KEY = "slice_margin_problem"     # 量不到的原因；量到了是空串
PROBE_SLICE_DIRECTORY_VARIABLE = "SINGLEFS_DISPATCH_PROBE_SLICE_DIRECTORY"   # 只给派发器的自证：slice 的 cgroup 目录换成一份假的（不问 systemctl）
PROBE_PROC_DIRECTORY_VARIABLE = "SINGLEFS_DISPATCH_PROBE_PROC_DIRECTORY"     # 只给派发器的自证：读 scope 里进程环境的 /proc 换成一份假的
# 在跑的 scope（文件头「在跑的 scope」）：headroom 交的 dict 里这一项，与每个 scope 那一份里的两个键
SCOPES_REPORT_KEY = "slice_scopes"
CPU_REPORT_KEY = "cpu"                           # headroom 交的 CPU 负载那一项（cpu_report）
SLICE_STARTED_KEY = "slice_started"              # headroom 交的：slice 起了（slice_total 读的是它的 memory.max）为真；没起（slice_total 是按整机内存估的）为假
SCOPE_USED_KEY = "unreclaimable"
SCOPE_LABEL_KEY = "run_label"
RUN_LABEL_VARIABLE = "TASK_SERVER_ATTEMPT_LABEL"   # task-server 给每次尝试的命令设的尝试标签（两台都给；task_server_lib.py 的 ATTEMPT_LABEL_VARIABLE）
PROC_ROOT = "/proc"
DEFAULT_RESERVE_TEXT = "20G"          # 只给 headroom：slice 还没起（这台开机以来还没起过内存包装）时按整机内存减它估总上限；slice 起了照它的 memory.max（多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项）


def command_key(words, explicit_key=None):
    """峰值表的键（文件头「键」）。words 是命令的各个词（一个词里带空格的仍是一个词）；explicit_key 是 RUN_WITH_MEMORY_CAP_KEY（空串当没给，是一段文本，按空格切词）。
    弄坏开关 item-words-split 下把词表用空格接起来再切开（改前的样子：条目名、路径里带空格的，后半截漏进键里）。"""
    if explicit_key:
        parts = explicit_key.replace("\t", " ").replace("\n", " ").split(" ")
    else:
        parts = [TEST_BINARY_HASH.sub("", word).replace("\t", " ").replace("\n", " ") for word in words]
        if "item-words-split" in _break_modes:
            parts = " ".join(parts).split(" ")
    return " ".join(normalized_words(parts))[:KEY_LENGTH_LIMIT]


_work_root_library = []   # work_root_library 载入的那一份 lib_work_root（载过一次就留着；载不到记 None）
_this_host = []           # this_host 取到的这一台的地址（一个进程取一次）


def configured_data_root():
    """没设 SINGLEFS_DATA_ROOT 时的数据根（文件头「键」②）：照本库所在仓根的 .claude/hooks/lib_work_root.py 的 shared_data_root 读多机配置；
    载不到那份库（经 python3 -c 送去跑、没有 __file__）或它报取不到（DataRootUnconfigured）交空串。弄坏开关 data-root-home-fallback 下不读多机配置、退回写死的 FIXED_DATA_ROOT。"""
    if "data-root-home-fallback" in _break_modes:
        return FIXED_DATA_ROOT
    library = work_root_library()
    if library is None:
        return ""
    try:
        return library.shared_data_root()
    except ValueError:
        return ""


def work_root_library():
    """本库所在仓根的 .claude/hooks/lib_work_root.py（载过一次就留着）；载不到（经 python3 -c 送去跑、没有 __file__，或那份文件不在、载不起来）交 None。"""
    if not _work_root_library:
        library = None
        root = repository_root()
        # 路径写成 os.path.join(<根>, 字面尾巴) 直接交给 spec_from_file_location：判法单元的闭包（scripts/gate/lib_function_units.py）照字面尾巴认出载的是哪一份，
        # 经条件表达式赋的变量折不出、整个闭包拒算（spec-path-unfolded）
        if root and os.path.isfile(os.path.join(root, ".claude", "hooks", "lib_work_root.py")):
            try:
                import importlib.util
                specification = importlib.util.spec_from_file_location("lib_work_root_for_memory_peaks_keys", os.path.join(root, ".claude", "hooks", "lib_work_root.py"))
                library = importlib.util.module_from_spec(specification)
                specification.loader.exec_module(library)
            except (OSError, ImportError, SyntaxError):
                library = None
        _work_root_library.append(library)
    return _work_root_library[0]


def scratch_roots():
    """工作根（展开成绝对路径、去掉末尾的 /；取法同 .claude/hooks/lib_work_root.py 的 scratch_roots，文件头「键」②）；取不到交空元组（不换工作根）。"""
    configured = os.environ.get(WORK_ROOT_VARIABLE)
    if not configured:
        data_root = os.environ.get(DATA_ROOT_VARIABLE) or configured_data_root()
        if not data_root:
            return ()
        configured = os.path.join(data_root, WORK_ENTRY)
    return (os.path.normpath(os.path.abspath(os.path.expanduser(configured))),)


def repository_root():
    """本库所在的仓根（本库在 <仓根>/scripts/hosts/ 下）；经 python3 -c 整份送去跑、没有 __file__ 时交空串（那边不取键）。"""
    library = globals().get("__file__")
    if not library:
        return ""
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(library))))


def without_thread_wrapper(words):
    """文件头「键」①：剥掉 capped.sh N（连同紧挨在前面的 shell 名）。收词表、交词表。"""
    if "capped-kept" in _break_modes or not any(THREAD_WRAPPER_NAME in word for word in words):
        return words
    words = list(words)
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
    return words


def path_replacements():
    """[(根的字面, 编译好的正则, 记号)]，按换的次序（工作根、仓根、家目录）：按此刻的 SINGLEFS_WORK_ROOT、HOME 建一次，取值不变就复用
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


def with_local_paths_replaced(words, replacements=None):
    """文件头「键」②：工作根下那一段、仓根、家目录换成固定记号，逐词换。收词表、交词表。replacements 是 path_replacements() 交的（读整张表时建一次传进来）。"""
    replacements = path_replacements() if replacements is None else replacements
    replaced = []
    for word in words:
        for literal, pattern, token in replacements:
            if literal in word:
                word = pattern.sub(token, word)
        replaced.append(word)
    return replaced


def path_shaped(word):
    """文件头「键」③：位置参数写成路径形态的（带 /、以 FILE_EXTENSIONS 结尾、以②的三个记号起头；以 - 起头、带 = 的不算）。"""
    if not word or word.startswith("-") or "=" in word:
        return False
    bare = word.strip("'\"")
    return "/" in bare or bare.endswith(FILE_EXTENSIONS) or bare.startswith((SCRATCH_TOKEN, REPOSITORY_TOKEN, HOME_TOKEN))


def cell_name(value):
    """文件头「键」③：--check 后面的格名照留进键；路径形态的当文件换成 FILE_TOKEN（格名不是路径）。"""
    return FILE_TOKEN if path_shaped(value) else value


def folded_assignment(word):
    """NAME=值：点名格与条目的变量换值为 ITEM_TOKEN，弄坏开关换值为 BREAK_VALUE_TOKEN，别的照原样（文件头「键」③）。"""
    name, _, value = word.partition("=")
    if not value:
        return word
    if name.endswith(BREAK_VARIABLE_SUFFIX):
        return f"{name}={BREAK_VALUE_TOKEN}"
    if name.endswith(ITEM_VARIABLE_SUFFIXES) or name in ITEM_VARIABLES:
        return f"{name}={ITEM_TOKEN}"
    return word


def kept_command_position(words, index=0):
    """文件头「键」③：(照留的那一个脚本或可执行文件的下标, -c 后面那一段脚本文本的下标)。
    照留的是命令词本身；解释器后面那一个；前缀命令后面那条命令再认；没有交 len(words)。没有 -c 的脚本文本下标交 None。"""
    while index < len(words):
        word = words[index]
        name = os.path.basename(word)
        if ASSIGNMENT.match(word):
            index += 1
        elif name in PREFIX_COMMANDS:
            index += 1
            while index < len(words) and (words[index].startswith("-") or ASSIGNMENT.match(words[index]) or PREFIX_DURATION.match(words[index])):
                index += 2 if words[index] in PREFIX_VALUE_OPTIONS else 1
        elif name in INTERPRETER_NAMES or VERSIONED_PYTHON.match(name):
            index += 1
            while index < len(words) and words[index].startswith("-"):
                if words[index] == "-c":
                    return kept_command_position(words, index + 1)[0], index + 1
                if words[index] == "-m":
                    return index + 1, None
                index += 1
            return index, None
        else:
            return index, None
    return index, None


def expanded_script_words(words):
    """文件头「键」③：-c 后面那一段是脚本文本，不是一个词：按空格切开、照一条命令再认；别的词（带空格的条目名、路径）仍是一个词。
    按 shell 分隔词把词表切成几条命令，逐条找它的 -c 脚本文本。"""
    expanded, command = [], []
    for word in list(words) + [None]:
        if word is not None and word not in SHELL_SEPARATORS:
            command.append(word)
            continue
        script_index = kept_command_position(command)[1]
        if script_index is not None and script_index < len(command):
            command = command[:script_index] + command[script_index].split(" ") + command[script_index + 1:]
        expanded += command + ([] if word is None else [word])
        command = []
    return expanded


def cargo_test_subcommand(words, kept):
    """命令词是 cargo、子命令是 test 或 nextest 的，交子命令；别的交空串。"""
    if kept >= len(words) or os.path.basename(words[kept]) != "cargo":
        return ""
    index = kept + 1
    while index < len(words) and (words[index].startswith(("-", "+"))):
        index += 2 if words[index] in KEPT_VALUE_OPTIONS else 1
    return words[index] if index < len(words) and words[index] in CARGO_TEST_SUBCOMMANDS else ""


def folded_command_words(words):
    """一条命令（不含 shell 分隔词）的词换成③的记号（还没并相邻的）。"""
    kept = kept_command_position(words)[0]
    folded = [folded_assignment(word) if ASSIGNMENT.match(word) else word for word in words[:kept + 1]]
    subcommand = cargo_test_subcommand(words, kept)
    positionals_kept = (2 if subcommand == "nextest" else 1) if subcommand else 0   # cargo test 的 test；nextest 连同 run
    after_double_dash = False
    # 弄坏开关 check-names-folded 下 --check 后面的格名也折成 ITEM_TOKEN（改前的样子）
    item_options = ITEM_OPTIONS + CELL_OPTIONS if "check-names-folded" in _break_modes else ITEM_OPTIONS
    index = kept + 1
    while index < len(words):
        word = words[index]
        option, equals, attached = word.partition("=")
        following = words[index + 1] if index + 1 < len(words) else None
        value_options = LIBTEST_VALUE_OPTIONS if after_double_dash else KEPT_VALUE_OPTIONS
        if ASSIGNMENT.match(word):
            folded.append(folded_assignment(word))
        elif option in item_options and equals:
            folded += [option, ITEM_TOKEN]
        elif option in CELL_OPTIONS and attached:
            folded += [option, cell_name(attached)]
        elif word in item_options:
            folded.append(word)
            if following is not None and not following.startswith("-"):
                folded.append(ITEM_TOKEN)
                index += 1
        elif word in CELL_OPTIONS:
            folded.append(word)
            if following is not None and not following.startswith("-"):
                folded.append(cell_name(following))
                index += 1
        elif word == "--" and subcommand:
            folded.append(word)
            after_double_dash = True
        elif word in value_options and following is not None:
            folded += [word, following]
            index += 1
        elif word.startswith("-") or (subcommand and word.startswith("+")):
            folded.append(word)
        elif subcommand and positionals_kept:
            folded.append(word)
            positionals_kept -= 1
        elif subcommand:
            folded.append(ITEM_TOKEN)
        else:
            folded.append(FILE_TOKEN if path_shaped(word) else word)
        index += 1
    return folded


def without_item_and_file_arguments(words):
    """文件头「键」③：点名的条目、文件参数、弄坏开关的值换成记号，同类相邻的并成一个。幂等。收词表、交词表：词里的空格不切（只有 -c 后面那一段脚本文本切，expanded_script_words）。"""
    if "item-and-file-arguments-kept" in _break_modes:
        return words
    command, merged = [], []
    for word in expanded_script_words(words) + [None]:
        if word is not None and word not in SHELL_SEPARATORS:
            command.append(word)
            continue
        for folded in folded_command_words(command) + ([] if word is None else [word]):
            if merged and folded == merged[-1] and any(token in folded for token in FOLD_TOKENS):
                continue
            merged.append(folded)
            if len(merged) >= 4 and merged[-1] == ITEM_TOKEN and merged[-2:] == merged[-4:-2]:
                del merged[-2:]
        command = []
    return merged


def normalized_words(words, replacements=None):
    """键的换算（文件头「键」①②③④），收词表、交词表。"""
    return without_item_and_file_arguments(with_local_paths_replaced(without_thread_wrapper(words), replacements))


def normalized_key(text, replacements=None):
    """一段键文本（显式给的键、表里的键）的换算：按空格切成词表，经 normalized_words 换算，再用空格接回；幂等。"""
    return " ".join(normalized_words(text.split(" "), replacements))


def size_bytes(text):
    """上限写法（正整数加 K / M / G / T）→ 字节；认不出交 None。"""
    if len(text or "") < 2 or text[-1] not in SIZE_UNITS or not text[:-1].isdigit():
        return None
    return int(text[:-1]) * SIZE_UNITS[text[-1]]


def human(byte_count):
    if byte_count >= 1024 ** 3:
        return f"{byte_count / 1024 ** 3:.1f} GiB"
    return f"{byte_count / 1024 ** 2:.0f} MiB"


class PeakTableUnavailable(ValueError):
    """取不到峰值表的位置，或认不出这一台（文件头「峰值表在哪」「哪一台」）：str 是原因连出路。"""


def default_peak_table():
    """共享数据根下的 memory-peaks.tsv（文件头「峰值表在哪」）：两台读写同一份。取不到抛 PeakTableUnavailable，不退回家目录。
    弄坏开关 peak-table-per-host 下照改前取本机的 ${XDG_DATA_HOME:-~/.local/share}/singlefs/memory-peaks.tsv（两台各一份）。"""
    if "peak-table-per-host" in _break_modes:
        return os.path.join(os.environ.get("XDG_DATA_HOME") or os.path.expanduser("~/.local/share"), "singlefs", PEAK_TABLE_NAME)
    library = work_root_library()
    if library is None:
        raise PeakTableUnavailable("载不到 .claude/hooks/lib_work_root.py（经 python3 -c 送去跑、没有 __file__，或那份文件不在），峰值表没处找；"
                                   "出路：按文件路径 import 这份库，或设 RUN_WITH_MEMORY_CAP_PEAKS 指到峰值表")
    try:
        return library.shared_data_path(PEAK_TABLE_NAME)
    except ValueError as error:
        raise PeakTableUnavailable(f"取不到共享数据根下的 {PEAK_TABLE_NAME}（{error}）；出路：照那一句修多机配置（SHARED_DATA_ROOT、NFS_SHARED_ROOTS），"
                                   "或设 RUN_WITH_MEMORY_CAP_PEAKS 指到峰值表") from error


def this_host():
    """这一台在多机配置主机数组里的地址（文件头「哪一台」）：峰值表第七列。取不到抛 PeakTableUnavailable。"""
    if _this_host:
        return _this_host[0]
    root = repository_root()
    if not root:
        raise PeakTableUnavailable("经 python3 -c 整份送去跑、没有 __file__：载不到 scripts/hosts/multi_host_addresses.py，认不出这一台")
    try:
        import importlib.util
        specification = importlib.util.spec_from_file_location("multi_host_addresses_for_memory_peaks", os.path.join(root, "scripts", "hosts", "multi_host_addresses.py"))
        addresses = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(addresses)
        address = addresses.resolve().this
    except (OSError, ImportError, SyntaxError, ValueError) as error:
        raise PeakTableUnavailable(f"认不出这一台是多机配置 HOSTS 里的哪一项（scripts/hosts/multi_host_addresses.py：{error}）；"
                                   "出路：单跑 python3 scripts/hosts/multi_host_addresses.py this 看原因，照多机配置模板 multi-host.env.example 写 HOSTS") from error
    _this_host.append(address)
    return address


class TableRowRefused(ValueError):
    """表里有一行不是这一版的形态（文件头「峰值表的口径与列」）：消息点名文件、行号、要几列与出路；读表的调用方不分列数、不分版本，都走这一条。"""


def refused_row(path, number, found_columns, wanted_columns, detail):
    return TableRowRefused(f"{path} 第 {number} 行不是这一版的形态：{detail}，要 {wanted_columns} 列，这一行 {found_columns} 列；"
                           "出路：删掉那一行（下次跑这条命令会重新记），或把整张表删了重记")


def table_data_lines(path):
    """表里每一数据行：(行号, 按制表符切开的各列)；不是 # 起头也不是空行的都算数据行。表不在不交。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return
    for number, line in enumerate(lines, 1):
        if line and not line.startswith("#"):
            yield number, line.split("\t")


def parse_recent(text):
    """第六列「近几次」→ [(峰值字节, 撞没撞顶, MEASURED_UNRECLAIMABLE)]（新的在前；文件头「口径」）；
    写坏的（空、某一次不是「<字节>:hit|clean:unreclaimable」）交 None。"""
    recent = []
    for item in (text or "").split(","):
        peak_text, _, rest = item.partition(":")
        flag, _, basis = rest.partition(":")
        if not peak_text.isdigit() or flag not in (RECENT_HIT, RECENT_CLEAN) or basis != MEASURED_UNRECLAIMABLE:
            return None
        recent.append((int(peak_text), flag == RECENT_HIT, MEASURED_UNRECLAIMABLE))
    return recent[:RECENT_MEASUREMENTS] or None


def format_recent(recent):
    """近几次 → 第六列。"""
    return ",".join(f"{measurement[0]}:{RECENT_HIT if measurement[1] else RECENT_CLEAN}:{MEASURED_UNRECLAIMABLE}" for measurement in recent[:RECENT_MEASUREMENTS])


def read_peak_rows(path, host=None):
    """本台的行：键 → (峰值字节, 量的时刻, 那一次的上限写法, 线程数写法, 近几次)；表不在交空表。前三列是最近一次实测。
    host 是哪一台（第七列；不给是 this_host()）：另一台的行不交（文件头「哪一台」）。弄坏开关 other-hosts-read 下不看第七列，哪一台的行都交（同一个键后写的盖前写的）。
    行不是这一版的形态整张表读不了：抛 TableRowRefused（read_all_peak_rows）。"""
    rows = read_all_peak_rows(path)
    if not rows:
        return {}
    if "other-hosts-read" in _break_modes:
        return {key: row for (_row_host, key), row in rows.items()}
    wanted = host if host is not None else this_host()
    return {key: row for (row_host, key), row in rows.items() if row_host == wanted}


def read_all_peak_rows(path):
    """两台的行：(哪一台, 键) → (峰值字节, 量的时刻, 那一次的上限写法, 线程数写法, 近几次)；表不在交空表。内存包装写表时读它，另一台的行原样写回。
    行不是这一版的形态（七列、第一列是数、第六列每一次写成「<字节>:hit|clean:unreclaimable」、第七列不空）整张表读不了：抛 TableRowRefused（文件头「峰值表的口径与列」）。"""
    rows = {}
    for number, fields in table_data_lines(path):
        recent = parse_recent(fields[5]) if len(fields) == PEAK_TABLE_COLUMNS else None
        if len(fields) != PEAK_TABLE_COLUMNS or not fields[0].isdigit() or recent is None or not fields[6]:
            if "row-shape-unchecked" in _break_modes:   # 弄坏开关：不核行的形态，不对的行跳过
                continue
            detail = ("第一列不是数" if len(fields) == PEAK_TABLE_COLUMNS and not fields[0].isdigit()
                      else "第六列写不对（每一次要写成「<字节>:hit|clean:unreclaimable」）" if len(fields) == PEAK_TABLE_COLUMNS and recent is None
                      else "第七列（哪一台）是空的" if len(fields) == PEAK_TABLE_COLUMNS else "列数不对")
            raise refused_row(path, number, len(fields), PEAK_TABLE_COLUMNS, detail)
        rows[(fields[6], fields[3])] = (int(fields[0]), fields[1], fields[2], fields[4], recent)
    return rows


def recorded_peak(path, key, latest_only=False, host=None):
    """本台的 (预测的峰值字节, 撞过顶没有)；表里没有本台这一条交 (None, False)，调用方照「表里没有」办。host 同 read_peak_rows。
    预测取近几次里最大的，其中有一次撞顶就算撞过顶（滞回：撞顶之后要连着 RECENT_MEASUREMENTS 次干净跑完才回到按峰值记账；
    峰值一路变小时记账不跟着变小）。latest_only 只看最近一次（只给派发器的弄坏开关用：改前的判法）。"""
    row = read_peak_rows(path, host).get(key)
    if row is None:
        return None, False
    recent = row[4][:1] if latest_only else row[4]
    return max(measurement[0] for measurement in recent), any(measurement[1] for measurement in recent)



def judged_measurement(outcome, cap_bytes, recorded_bytes, unreclaimable_peak, oom_kill_count, record_memory_peak=False, hit_by_peak=False):
    """内存包装收尾时这一次记进峰值表的那一次（文件头「口径」；run-with-memory-cap.sh 文件头「峰值表」一段）：交近几次的一个元素，不记交 None。
    outcome 是 run_capped 交的结局；recorded_bytes 撞顶那三种（HIT_OUTCOMES）是那一次的上限，clean 是 scope 的 memory.peak；
    unreclaimable_peak 是跑的过程中采到的回收不掉的用量的峰值（采不到是 None）；oom_kill_count 是 scope 的 memory.events 里的 oom_kill（读不到当 0）。
      撞顶那三种：上限，撞了顶。
      clean：oom_kill 大于 0（Result 没报 oom-kill 而内核在这个 scope 里杀过进程）记成撞了顶、峰值记上限；否则记采到的回收不掉的峰值、没撞顶，采不到的不记。
        峰值到了上限而 oom_kill 是 0 的（页缓存涨满上限）照样是 clean。
      别的结局不记，交 None。
    record_memory_peak 与 hit_by_peak 是包装的弄坏开关（recordmemorypeak、hitbypeak）；本库的 record-memory-peak、hit-by-peak 同义。"""
    record_memory_peak = record_memory_peak or "record-memory-peak" in _break_modes
    hit_by_peak = hit_by_peak or "hit-by-peak" in _break_modes
    if outcome in HIT_OUTCOMES:
        return (recorded_bytes, True, MEASURED_UNRECLAIMABLE)
    if outcome != "clean":
        return None
    if hit_by_peak:   # 弄坏开关：照改前按 memory.peak 到没到上限判撞顶，不看 oom_kill
        hit = cap_bytes is not None and recorded_bytes is not None and recorded_bytes >= cap_bytes
    else:
        hit = bool(oom_kill_count) and cap_bytes is not None
    if hit:
        return (cap_bytes, True, MEASURED_UNRECLAIMABLE)
    if record_memory_peak:   # 弄坏开关：峰值照旧取 memory.peak（含页缓存）
        return None if recorded_bytes is None else (recorded_bytes, False, MEASURED_UNRECLAIMABLE)
    if unreclaimable_peak is None:
        return None
    return (unreclaimable_peak, False, MEASURED_UNRECLAIMABLE)


def sample_unreclaimable_peak(cgroup_directory_of, output_path, keep_going, interval_seconds=PEAK_SAMPLE_SECONDS, sleep=time.sleep):
    """跑的过程中隔 interval_seconds 秒采一次 cgroup 回收不掉的用量（unreclaimable_bytes），取最大（文件头「口径」）。
    cgroup_directory_of() 交此刻知道的 cgroup 目录（还不知道交空串，下一轮再问）；keep_going() 交 False 就停。
    每次最大值涨了就把「peak <字节>」先写 <output_path>.partial 再改名成 output_path（中途被停，读到的也是整的）。
    目录不在（scope 还没起或已收尾）、读不了的那一次不算采到。交采到的最大值，一次都没采到交 None（调用方不记）。"""
    best = None
    while True:
        directory = cgroup_directory_of()
        if directory and os.path.isdir(directory):
            value = unreclaimable_bytes(directory)
            if value is not None and os.path.isdir(directory) and (best is None or value > best):
                best = value
                with open(output_path + ".partial", "w", encoding="utf-8") as handle:
                    handle.write(f"peak {best}\n")
                os.replace(output_path + ".partial", output_path)
        if not keep_going():
            return best
        sleep(interval_seconds)


def peak_table_line(key, row, host=None):
    """峰值表的一行（row 是 read_peak_rows 那五段；host 是第七列，不给是 this_host()）；线程数没记的（None）写「未记」占住第五列。与内存包装 record_run 写表用同一份。"""
    peak, measured, cap, threads, recent = row
    return "\t".join([str(peak), measured, cap, key, threads if threads is not None else "未记", format_recent(recent), host if host is not None else this_host()])



def unreclaimable_bytes(cgroup_directory):
    """cgroup 里回收不掉的用量：memory.current 减 memory.stat 的 active_file 与 inactive_file（页缓存回收得掉）。
    cgroup v2 里 memory.current 约等于 memory.stat 的 anon + file + kernel + sock，shmem 记在 file 里而不在 active_file、inactive_file 里（它挂在 anon 的 LRU 上），
    所以剩下的是 anon + shmem（tmpfs 与共享内存，不开 swap 回收不掉）+ kernel（slab、kernel_stack、pagetables、percpu、vmalloc）+ sock。
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


def process_memory_bytes(cgroup_directory):
    """cgroup 里进程用的内存：memory.stat 的 anon 加 shmem（用户：「内核的资源是内核的资源 和 我们分配的资源无关」）。
    页缓存回收得掉、内核挂在 slice 上的（任务退出之后留下的 slab 这类）不是分出去的，都不算。目录不在交 0；目录在而文件读不了交 None。
    弄坏开关 kernel-counted：照改前算回收不掉的用量（连内核的一起）。"""
    if "kernel-counted" in _break_modes:
        return unreclaimable_bytes(cgroup_directory)
    try:
        total = 0
        with open(os.path.join(cgroup_directory, "memory.stat"), encoding="utf-8") as handle:
            for line in handle:
                name, value = line.split()
                if name in ("anon", "shmem"):
                    total += int(value)
    except FileNotFoundError:
        return None if os.path.isdir(cgroup_directory) else 0
    except (OSError, ValueError):
        return None
    return total


def process_start_time(process_id):
    """/proc/<pid>/stat 的第 22 段（进程起始时刻，开机以来的时钟滴答）：进程号被复用时它不同。进程不在交 None。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8") as handle:
            return handle.read().rsplit(")", 1)[1].split()[19]
    except (OSError, IndexError):
        return None


def meminfo(path="/proc/meminfo"):
    values = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            name, rest = line.split(":", 1)
            values[name] = int(rest.split()[0]) * 1024
    return values


def default_state_directory(environment):
    """内存包装的锁与峰值文件在哪：与 run-with-memory-cap.sh 的 STATE_DIRECTORY 同一个取法。"""
    return environment.get("RUN_WITH_MEMORY_CAP_STATE_DIR") or os.path.join(
        environment.get("XDG_RUNTIME_DIR") or f"/run/user/{os.getuid()}", "singlefs-heavy-admission")


def slice_directory_of(slice_name, cgroup_root):
    """slice 的 cgroup 目录；slice 还没起（systemctl 答了、ControlGroup 是空的）交空串；查不了用户级 systemd（起不来、超时、退非 0）交 None。"""
    try:
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "ControlGroup", "--value", slice_name],
                                   capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=20)
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    cgroup_path = completed.stdout.strip()
    return cgroup_root.rstrip("/") + cgroup_path if cgroup_path else ""


def rounded_up_to_gibibyte(byte_count):
    """按 1 GiB 向上取整（文件头「余量」：1.01G、1.1G 记 2G，2.1G 记 3G，0.7G 记 1G，恰好整数的不进位）。
    弄坏开关 margin-floored 下向下取整，margin-whole-carried 下恰好整数的也进一位。"""
    if "margin-floored" in _break_modes:
        return byte_count // GIBIBYTE * GIBIBYTE
    if "margin-whole-carried" in _break_modes:
        return (byte_count // GIBIBYTE + 1) * GIBIBYTE
    return -(-byte_count // GIBIBYTE) * GIBIBYTE


def slice_margin_bytes(slice_directory):
    """slice 自己挂着、哪个在跑的子 cgroup 里都没有的回收不掉的那一截（文件头「余量」）：→ (字节, 量不到的原因)。
    = unreclaimable_bytes(slice) − 每个子 cgroup 的 unreclaimable_bytes 之和，不小于 0。先读子 cgroup、后读 slice：两次读之间子 cgroup 涨的那一截算进余量（多留），
    不会少算。slice 目录不在（刚停）交 0；slice 或哪个子 cgroup 在而读不了交 None。弄坏开关 margin-children-counted 下不减子 cgroup（整个 slice 回收不掉的都算），
    margin-unreadable-as-zero 下读不了的当 0。"""
    try:
        children = sorted(entry.path for entry in os.scandir(slice_directory) if entry.is_dir(follow_symlinks=False))
    except FileNotFoundError:
        return 0, ""
    except OSError as error:
        return (0, "") if "margin-unreadable-as-zero" in _break_modes else (None, f"列不了 slice 的子 cgroup（{type(error).__name__}: {error}）")
    children_total = 0
    for child in children:
        used = unreclaimable_bytes(child)
        if used is None:
            if "margin-unreadable-as-zero" in _break_modes:
                continue
            return None, f"子 cgroup {os.path.basename(child)} 的 memory.current 或 memory.stat 读不了"
        if "margin-children-counted" not in _break_modes:
            children_total += used
    whole = unreclaimable_bytes(slice_directory)
    if whole is None:
        return (0, "") if "margin-unreadable-as-zero" in _break_modes else (None, "slice 的 memory.current 或 memory.stat 读不了")
    return max(0, whole - children_total), ""


def measured_margin(slice_directory):
    """headroom 交的余量那三项（文件头「余量」）：{MARGIN_REPORT_KEY: 取整后的字节或 None, MARGIN_MEASURED_KEY: 量到的字节或 None, MARGIN_PROBLEM_KEY: 原因}。
    slice_directory 是 slice_directory_of 交的：None 是查不了用户级 systemd（量不到），空串是 slice 还没起（里面一字节都没挂，量到 0）。"""
    if slice_directory is None:
        measured, problem = None, "查不了用户级 systemd（systemctl --user show 起不来、超时或退非 0），slice 的 cgroup 目录找不到"
    elif not slice_directory:
        measured, problem = 0, ""
    else:
        measured, problem = slice_margin_bytes(slice_directory)
    return {MARGIN_REPORT_KEY: None if measured is None else rounded_up_to_gibibyte(measured), MARGIN_MEASURED_KEY: measured, MARGIN_PROBLEM_KEY: problem}


def scope_run_label(cgroup_directory, proc_root=PROC_ROOT):
    """这个 cgroup 里的进程是 task-server 哪一次尝试起的（文件头「在跑的 scope」）：cgroup.procs 里头一个环境带 RUN_LABEL_VARIABLE 的进程，取那个值；
    都没有、读不了交空串。进程在两次读之间退了（environ 读不了）的看下一个。弄坏开关 scope-label-unread 下一律交空串。"""
    if "scope-label-unread" in _break_modes:
        return ""
    try:
        with open(os.path.join(cgroup_directory, "cgroup.procs"), encoding="utf-8") as handle:
            process_ids = [line.strip() for line in handle if line.strip().isdigit()]
    except OSError:
        return ""
    prefix = (RUN_LABEL_VARIABLE + "=").encode("ascii")
    for process_id in process_ids:
        try:
            with open(os.path.join(proc_root, process_id, "environ"), "rb") as handle:
                entries = handle.read().split(b"\0")
        except OSError:
            continue
        for entry in entries:
            if entry.startswith(prefix):
                return entry[len(prefix):].decode("utf-8", "replace").strip()
    return ""


def slice_scopes(slice_directory, proc_root=PROC_ROOT):
    """slice 下每个子 cgroup：{目录名: {SCOPE_USED_KEY: 回收不掉的用量（读不了是 None）, SCOPE_LABEL_KEY: 标签}}（文件头「在跑的 scope」）。
    slice_directory 是 slice_directory_of 交的：None（查不了 systemd）、空串（slice 还没起）、目录不在或列不了，都交空表。"""
    if not slice_directory:
        return {}
    try:
        children = sorted((entry.name, entry.path) for entry in os.scandir(slice_directory) if entry.is_dir(follow_symlinks=False))
    except OSError:
        return {}
    return {name: {SCOPE_USED_KEY: unreclaimable_bytes(path), SCOPE_LABEL_KEY: scope_run_label(path, proc_root)} for name, path in children}


def headroom(environment):
    """这台此刻的内存余量（task-server 经 host-dispatch.py 的 probe_local、probe_peer 读；第二台上由派发器把这份源码送过去跑）。读的环境变量与包装的同名：
    RUN_WITH_MEMORY_CAP_SLICE、RUN_WITH_MEMORY_CAP_CGROUP_ROOT、RUN_WITH_MEMORY_CAP_RESERVE（只在 slice 还没起时用，见 DEFAULT_RESERVE_TEXT）；
    SINGLEFS_DISPATCH_PROBE_MEMINFO、PROBE_SLICE_DIRECTORY_VARIABLE 与 PROBE_PROC_DIRECTORY_VARIABLE 只给派发器的自证换掉 /proc/meminfo、slice 的 cgroup 目录与读进程环境的 /proc。
    记账在 task-server 那一本，这里不读别的账。
    交一个 dict（字节）：
      mem_total、mem_available        /proc/meminfo
      slice_total、slice_used         slice 的总上限（slice 在就读它的 memory.max，不在按整机内存减 DEFAULT_RESERVE_TEXT 估）与 slice 里进程用的内存（process_memory_bytes：anon + shmem，内核的不算）
      SLICE_STARTED_KEY               slice_total 是不是读的 memory.max（没起是估的；task-server 拿多机配置里那台那一项换掉估的）
      free                            mem_available：此刻整机放得下多少
      slice_room                      slice_total − slice_used：slice 此刻放得下多少
      MARGIN_REPORT_KEY、MARGIN_MEASURED_KEY、MARGIN_PROBLEM_KEY   这台此刻的余量（measured_margin，文件头「余量」）
      SCOPES_REPORT_KEY               slice 下每个在跑的 scope 此刻回收不掉的用量与标签（slice_scopes，文件头「在跑的 scope」）
      CPU_REPORT_KEY                  这台的 CPU：{"load1": 1 分钟负载, "procs_running": 此刻可跑的进程数, "cpus": 核数}（cpu_report；task-server 的 CPU 闸读）"""
    memory = meminfo(environment.get("SINGLEFS_DISPATCH_PROBE_MEMINFO") or "/proc/meminfo")
    cgroup_root = environment.get("RUN_WITH_MEMORY_CAP_CGROUP_ROOT") or "/sys/fs/cgroup"
    slice_directory = (environment.get(PROBE_SLICE_DIRECTORY_VARIABLE)
                       or slice_directory_of(environment.get("RUN_WITH_MEMORY_CAP_SLICE") or DEFAULT_SLICE, cgroup_root))
    slice_total = None
    if slice_directory:
        with contextlib.suppress(OSError, ValueError):
            with open(os.path.join(slice_directory, "memory.max"), encoding="utf-8") as handle:
                slice_total = int(handle.read())
    started = slice_total is not None
    if slice_total is None:
        reserve = size_bytes(environment.get("RUN_WITH_MEMORY_CAP_RESERVE") or DEFAULT_RESERVE_TEXT) or 0
        slice_total = max(0, (memory["MemTotal"] - reserve) // MEBIBYTE * MEBIBYTE)
    slice_used = (process_memory_bytes(slice_directory) or 0) if slice_directory else 0
    return dict({"mem_total": memory["MemTotal"], "mem_available": memory["MemAvailable"],
                 "slice_total": slice_total, SLICE_STARTED_KEY: started, "slice_used": slice_used, "free": memory["MemAvailable"],
                 "slice_room": slice_total - slice_used,
                 SCOPES_REPORT_KEY: slice_scopes(slice_directory, environment.get(PROBE_PROC_DIRECTORY_VARIABLE) or PROC_ROOT),
                 CPU_REPORT_KEY: cpu_report(environment.get(PROBE_PROC_DIRECTORY_VARIABLE) or PROC_ROOT)}, **measured_margin(slice_directory))


def cpu_report(proc_root):
    """这台此刻的 CPU 负载：/proc/loadavg 的 1 分钟负载、/proc/stat 的 procs_running 与核数；读不了的那一项是 None。"""
    report = {"load1": None, "procs_running": None, "cpus": os.cpu_count()}
    with contextlib.suppress(OSError, ValueError, IndexError):
        with open(os.path.join(proc_root, "loadavg"), encoding="utf-8") as handle:
            report["load1"] = float(handle.read().split()[0])
    with contextlib.suppress(OSError, ValueError, IndexError):
        with open(os.path.join(proc_root, "stat"), encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("procs_running "):
                    report["procs_running"] = int(line.split()[1])
    return report


# ── 自证（python3 scripts/hosts/memory_peaks_lib.py --selftest）：只调本库的函数、在临时目录里写假峰值表，不起子进程 ──
# 弄坏开关（逗号连几个；只在自证里读进来，import 这份库的包装、派发器、看门狗一律不认；平常跑设了它退 2 并点名它只在自证里认，见 refuse_break_outside_selftest；
# 认不出的值退 2）。逐项写开关名、弄坏哪一处、打开后自证哪几格红（「」里是失败行原文；末尾一句「✗ memory_peaks_lib.py 自证 N 格里 M 格不对」是汇总）：
#   MEMORY_PEAKS_LIB_BREAK=scratch-paths-kept  取键不换工作根下那一段（改前的样子）
#       「✗ 自证格不对：两份副本路径下同一条命令落一个键」
#       「✗ 自证格不对：经 capped.sh 与不经落一个键」（那一格里连副本路径的那一例要落到〈工作根下的目录〉，开关下没换）
#   MEMORY_PEAKS_LIB_BREAK=capped-kept  取键不剥 capped.sh N（改前的样子）
#       「✗ 自证格不对：经 capped.sh 与不经落一个键」
#   MEMORY_PEAKS_LIB_BREAK=item-and-file-arguments-kept  取键不换条目、文件参数与弄坏开关的值（文件头「键」③；改前的样子）
#       「✗ 自证格不对：两份副本路径下同一条命令落一个键」（要的键里 --exact 后面是〈条目〉）
#       「✗ 自证格不对：只差条目、文件、开关值的命令落一个键」
#       「✗ 自证格不对：条目名里带空格的同一条命令落一个键，带空格的一个词仍是一个词」
#       「✗ 自证格不对：--check 后面的格名进键、不同格落不同键，--item 后面的照旧折」
#   MEMORY_PEAKS_LIB_BREAK=item-words-split  command_key 把词表用空格接起来再切开折叠（改前的样子：条目名、路径里带空格的，后半截漏进键里）
#       「✗ 自证格不对：条目名里带空格的同一条命令落一个键，带空格的一个词仍是一个词」
#   MEMORY_PEAKS_LIB_BREAK=check-names-folded  --check 后面的格名也折成〈条目〉（改前的样子：轻的几格与重的一格落一个键）
#       「✗ 自证格不对：--check 后面的格名进键、不同格落不同键，--item 后面的照旧折」
#   MEMORY_PEAKS_LIB_BREAK=row-shape-unchecked  读峰值表不核行的形态：不对的行跳过，不抛 TableRowRefused
#       「✗ 自证格不对：表里只认这一版的行，不对的行整张表读不了并点名行号、要几列与出路」
#   MEMORY_PEAKS_LIB_BREAK=data-root-home-fallback  没设 SINGLEFS_DATA_ROOT 时不读多机配置、退回一个写死的目录（FIXED_DATA_ROOT）
#       「✗ 自证格不对：工作根的取法与 lib_work_root.py 的 scratch_roots 一样（连同换了数据根、两个变量都不设而家目录不在）」
#   MEMORY_PEAKS_LIB_BREAK=other-hosts-read  read_peak_rows 不看第七列（哪一台），两台的行都交（改前一台一份表的读法）
#       「✗ 自证格不对：同一个键两台各一行：读的时候只认本台的那一行，另一台的照留」
#   MEMORY_PEAKS_LIB_BREAK=peak-table-per-host  default_peak_table 照改前取本机家目录下的 ${XDG_DATA_HOME:-~/.local/share}/singlefs/memory-peaks.tsv（两台各一份）
#       「✗ 自证格不对：默认的峰值表在共享数据根下（lib_work_root.py 的 shared_data_path），不在家目录下」
#   MEMORY_PEAKS_LIB_BREAK=outside-selftest-unnamed  refuse_break_outside_selftest 什么都不交（改前的样子：平常跑设了开关只打「单独跑只认 --selftest」，不点名变量）
#       「✗ 自证格不对：平常跑设了弄坏开关退 2 并点名 MEMORY_PEAKS_LIB_BREAK 只在自证里认」

#   口径那几格（measurement_cells，文件头「口径」）的三个开关；格名与红法是推的、照代码读出来的，交主 agent 打开开关跑一遍核：
#   MEMORY_PEAKS_LIB_BREAK=record-memory-peak  judged_measurement 的 clean 照旧记 memory.peak（含页缓存），采不到也记
#       「✗ 自证格不对：页缓存占了大半时记回收不掉的那一截，不是上限」（记的是上限 64M）
#       「✗ 自证格不对：oom_kill 大于 0 记撞顶、峰值记上限（Result 没报 oom-kill 也算），采不到的不记」（采不到的那一次也记了 memory.peak）
#       「✗ 自证格不对：峰值到了上限而 oom_kill 是 0 记 clean、不把上限往上推」（记的、读回的都是上限 64M）
#   MEMORY_PEAKS_LIB_BREAK=hit-by-peak  judged_measurement 的 clean 照旧按 memory.peak 到没到上限判撞顶，不看 oom_kill
#       「✗ 自证格不对：页缓存占了大半时记回收不掉的那一截，不是上限」（记成撞顶、记上限）
#       「✗ 自证格不对：oom_kill 大于 0 记撞顶、峰值记上限（Result 没报 oom-kill 也算），采不到的不记」（oom_kill 1 而 memory.peak 没到上限的记成 clean）
#       「✗ 自证格不对：峰值到了上限而 oom_kill 是 0 记 clean、不把上限往上推」（记成撞顶）
#   MEMORY_PEAKS_LIB_BREAK=kernel-counted  process_memory_bytes 照改前算回收不掉的用量（内核挂在 slice 上的也算进 slice 的用量）
#       「✗ 自证格不对：slice 的用量只算进程的内存（anon + shmem），内核与页缓存不算」
#   余量那几格（margin_cells，文件头「余量」）的四个开关：
#   MEMORY_PEAKS_LIB_BREAK=margin-floored  rounded_up_to_gibibyte 向下取整
#       「✗ 自证格不对：余量按 1 GiB 向上取整：slice 里量到 1.01G 记 2G」「… 1.1G 记 2G」「… 2.1G 记 3G」「… 0.7G 记 1G」（四格；2.0G 那一格不红）
#   MEMORY_PEAKS_LIB_BREAK=margin-whole-carried  rounded_up_to_gibibyte 恰好整数的也进一位
#       「✗ 自证格不对：余量按 1 GiB 向上取整：slice 里量到 2.0G 记 2G」
#   MEMORY_PEAKS_LIB_BREAK=margin-children-counted  slice_margin_bytes 不减在跑的子 cgroup（整个 slice 回收不掉的都算余量）
#       「✗ 自证格不对：余量只算 slice 自己挂着、子 cgroup 里没有的那一截（在跑任务的 scope 用的不算）」
#   MEMORY_PEAKS_LIB_BREAK=margin-unreadable-as-zero  slice_margin_bytes 读不了的 slice 与子 cgroup 当 0
#       「✗ 自证格不对：读不了 slice、读不了子 cgroup、查不了用户级 systemd 时量不到余量、不交数（slice 还没起的量到 0）」
#   在跑的 scope 那一格（scope_cells，文件头「在跑的 scope」）的开关：
#   MEMORY_PEAKS_LIB_BREAK=scope-label-unread  scope_run_label 不读进程的环境，标签一律空串（第二台的 scope 对不到尝试上）
#       「✗ 自证格不对：在跑的 scope 逐个报回收不掉的用量与标签（标签取进程环境里 TASK_SERVER_ATTEMPT_LABEL 的值，没有的是空串）」
SELFTEST_HOST = "192.0.2.1"         # 自证造的表里本台的地址（文档用的地址段；读表时显式给，不读这一台的多机配置）
SELFTEST_OTHER_HOST = "192.0.2.2"   # 自证造的表里另一台的地址


def self_test_cells(work):
    """交回 [(格名, 对不对, 说明)]。work 是临时目录。"""
    cells = []
    work_root = scratch_roots()[0]
    repository = repository_root()
    home = os.path.normpath(os.path.expanduser("~"))

    copies = [[f"{work_root}/task-alpha/tree/target/debug/deps/journal_ring_fields-0123456789abcdef", "--exact", "ring"],
              [f"{work_root}/task-beta/copy/nested/target/debug/deps/journal_ring_fields-fedcba9876543210", "--exact", "ring"],
              [f"{work_root}/task-gamma/rerun/target/debug/deps/journal_ring_fields-00112233445566aa", "--exact", "ring"]]
    copy_keys = [command_key(words) for words in copies]
    no_target = [command_key(["python3", "-I", "tools/numbers_model.py", f"{work_root}/{name}", "j2_q"]) for name in ("task-alpha", "task-beta")]
    other = command_key([f"{work_root}/task-alpha/tree/target/debug/deps/other_test_name-0123456789abcdef", "--exact", "ring"])
    wanted = f"{SCRATCH_TOKEN}/target/debug/deps/journal_ring_fields --exact {ITEM_TOKEN}"
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

    cells.extend(measurement_cells(work))
    cells.extend(key_fold_cells(work))
    cells.extend(row_shape_cells(work))
    cells.extend(host_column_cells(work))

    roots_note = "没读 .claude/hooks/lib_work_root.py"
    same_roots = False
    try:
        import importlib.util
        spec = importlib.util.spec_from_file_location("lib_work_root_for_memory_peaks", os.path.join(repository, ".claude", "hooks", "lib_work_root.py"))
        work_root_library = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(work_root_library)
        same_roots = tuple(work_root_library.scratch_roots()) == scratch_roots()
        roots_note = f"lib_work_root 的 {tuple(work_root_library.scratch_roots())}，这里的 {scratch_roots()}"
        # 再换一个数据根、不设工作根判一遍：两边都要落到那个数据根下的 work
        saved = {name: os.environ.pop(name, None) for name in (WORK_ROOT_VARIABLE, DATA_ROOT_VARIABLE)}
        os.environ[DATA_ROOT_VARIABLE] = os.path.join(work, "sample-data-root")
        try:
            sampled = (tuple(work_root_library.scratch_roots()), scratch_roots())
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        same_roots = same_roots and sampled[0] == sampled[1] == (os.path.join(work, "sample-data-root", WORK_ENTRY),)
        roots_note += f"；换了数据根之后 lib_work_root 的 {sampled[0]}，这里的 {sampled[1]}"
        # 两个变量都不设、家目录指到不在的地方（第二台的条件）：两边都照多机配置取（取不到时两边都不交），不退回家目录下那一份
        saved = {name: os.environ.pop(name, None) for name in (WORK_ROOT_VARIABLE, DATA_ROOT_VARIABLE, "HOME")}
        os.environ["HOME"] = "/nonexistent/x"
        try:
            try:
                library_roots = tuple(work_root_library.scratch_roots())
            except ValueError:
                library_roots = ()
            unset_roots = scratch_roots()
        finally:
            os.environ.pop("HOME", None)
            for name, value in saved.items():
                if value is not None:
                    os.environ[name] = value
        same_roots = same_roots and library_roots == unset_roots and not any(root.startswith("/nonexistent/") for root in unset_roots)
        roots_note += f"；两个变量都不设、家目录不在时 lib_work_root 的 {library_roots}，这里的 {unset_roots}"
    except (OSError, ImportError, AttributeError, ValueError) as error:
        roots_note = f"读不了 lib_work_root.py：{type(error).__name__}: {error}"
    cells.append(("工作根的取法与 lib_work_root.py 的 scratch_roots 一样（连同换了数据根、两个变量都不设而家目录不在）", same_roots, roots_note))

    probe_arguments = ["--force", "--break-guard-probe-unknown-option"]
    refused = refuse_break_outside_selftest(probe_arguments, {BREAK_VARIABLE: "capped-kept"})
    named = [line for line in refused if BREAK_VARIABLE in line and "自证" in line]
    passed_plain = refuse_break_outside_selftest(probe_arguments, {}) == []
    passed_selftest = refuse_break_outside_selftest(["--selftest"], {BREAK_VARIABLE: "capped-kept"}) == []
    kernel_slice = os.path.join(work, "kernel-only-slice")   # 照真机读数：任务都退了，只剩页缓存与内核挂在 slice 上
    os.makedirs(kernel_slice, exist_ok=True)
    with open(os.path.join(kernel_slice, "memory.current"), "w", encoding="utf-8") as handle:
        handle.write(f"{878 * MEBIBYTE}\n")
    with open(os.path.join(kernel_slice, "memory.stat"), "w", encoding="utf-8") as handle:
        handle.write(f"anon 0\nfile {751 * MEBIBYTE}\nkernel {126 * MEBIBYTE}\nshmem {1 * MEBIBYTE}\nactive_file {427 * MEBIBYTE}\ninactive_file {322 * MEBIBYTE}\n")
    counted = process_memory_bytes(kernel_slice)
    cells.append(("slice 的用量只算进程的内存（anon + shmem），内核与页缓存不算", counted == 1 * MEBIBYTE,
                  f"anon 0、shmem 1M、file 751M、kernel 126M 的 slice 算出 {counted} 字节（要的 {1 * MEBIBYTE}）"))
    cells.extend(margin_cells(work))
    cells.extend(scope_cells(work))
    cells.append((f"平常跑设了弄坏开关退 2 并点名 {BREAK_VARIABLE} 只在自证里认",
                  bool(named) and passed_plain and passed_selftest,
                  f"设了开关、参数 {probe_arguments} 时交回 {refused}；没设开关时放行 {passed_plain}；参数正好 --selftest 时放行 {passed_selftest}"))
    return cells


def fake_cgroup(directory, unreclaimable, file_pages=751 * MEBIBYTE):
    """假 cgroup 目录：回收不掉的用量是 unreclaimable 字节（memory.current = 它 + 页缓存 file_pages，页缓存一半 active、一半 inactive）。"""
    os.makedirs(directory, exist_ok=True)
    with open(os.path.join(directory, "memory.current"), "w", encoding="utf-8") as handle:
        handle.write(f"{unreclaimable + file_pages}\n")
    with open(os.path.join(directory, "memory.stat"), "w", encoding="utf-8") as handle:
        handle.write(f"anon 0\nfile {file_pages}\nkernel {unreclaimable}\nshmem 0\nactive_file {file_pages // 2}\ninactive_file {file_pages - file_pages // 2}\n")


MARGIN_ROUNDING_SAMPLES = ((1.01, 2), (1.1, 2), (2.1, 3), (0.7, 1), (2.0, 2))   # (slice 里量到的 GiB, 取整之后的 GiB)：文件头「余量」的取整


def margin_cells(work):
    """文件头「余量」那几格：取整各一格、只算子 cgroup 里没有的那一截、读不了时量不到。交 [(格名, 对不对, 说明)]。
    取整那几格的 slice 下没有子 cgroup（不受 margin-children-counted 弄红）；减子 cgroup 那一格核取整之前的字节（不受取整的两个开关弄红）。"""
    cells = []
    for measured_gibibytes, wanted_gibibytes in MARGIN_ROUNDING_SAMPLES:
        directory = os.path.join(work, f"margin-rounding-{measured_gibibytes}")
        fake_cgroup(directory, int(measured_gibibytes * GIBIBYTE))
        margin = measured_margin(directory)
        cells.append((f"余量按 1 GiB 向上取整：slice 里量到 {measured_gibibytes}G 记 {wanted_gibibytes}G",
                      margin[MARGIN_REPORT_KEY] == wanted_gibibytes * GIBIBYTE and margin[MARGIN_MEASURED_KEY] == int(measured_gibibytes * GIBIBYTE)
                      and not margin[MARGIN_PROBLEM_KEY],
                      f"量到 {int(measured_gibibytes * GIBIBYTE)} 字节的 slice 交 {margin}（要的余量 {wanted_gibibytes * GIBIBYTE}）"))

    with_child = os.path.join(work, "margin-with-running-scope")
    fake_cgroup(with_child, 3 * GIBIBYTE + GIBIBYTE // 2)   # slice 一共回收不掉 3.5G：在跑的 scope 3G、slice 自己挂着 0.5G
    fake_cgroup(os.path.join(with_child, "singlefs-memory-cap-selftest.scope"), 3 * GIBIBYTE, file_pages=200 * MEBIBYTE)
    measured, problem = slice_margin_bytes(with_child)
    cells.append(("余量只算 slice 自己挂着、子 cgroup 里没有的那一截（在跑任务的 scope 用的不算）", measured == GIBIBYTE // 2 and not problem,
                  f"slice 回收不掉 3.5G、它下面在跑的 scope 3G：量到 {measured} 字节（要的 {GIBIBYTE // 2}）、问题「{problem}」"))

    unreadable = os.path.join(work, "margin-unreadable")
    os.makedirs(unreadable, exist_ok=True)   # 目录在而没有 memory.current：读不了
    broken_child = os.path.join(work, "margin-unreadable-child")
    fake_cgroup(broken_child, GIBIBYTE)
    os.makedirs(os.path.join(broken_child, "singlefs-memory-cap-selftest.scope"), exist_ok=True)
    no_systemd = measured_margin(None)
    not_started = measured_margin("")
    results = [measured_margin(unreadable), measured_margin(broken_child), no_systemd]
    cells.append(("读不了 slice、读不了子 cgroup、查不了用户级 systemd 时量不到余量、不交数（slice 还没起的量到 0）",
                  all(result[MARGIN_REPORT_KEY] is None and result[MARGIN_MEASURED_KEY] is None and result[MARGIN_PROBLEM_KEY] for result in results)
                  and not_started[MARGIN_MEASURED_KEY] == 0 and isinstance(not_started[MARGIN_REPORT_KEY], int) and not not_started[MARGIN_PROBLEM_KEY],
                  f"slice 没有 memory.current 交 {results[0]}；子 cgroup 没有 memory.current 交 {results[1]}；查不了 systemd 交 {no_systemd}（三样都要 None 与原因）；"
                  f"slice 还没起交 {not_started}（要量到 0、没有原因）"))
    return cells



def scope_cells(work):
    """文件头「在跑的 scope」那一格：slice 下三个 scope，各自的用量照报；标签取 cgroup.procs 里头一个环境带 RUN_LABEL_VARIABLE 的进程（前一个进程已退、读不了 environ 的跳过），
    环境里没有它的 scope 标签是空串；slice 没起交空表。/proc 换成临时目录里的假的（文件里的进程号只在假 /proc 里有）。"""
    slice_directory = os.path.join(work, "scopes-slice")
    proc_root = os.path.join(work, "scopes-proc")
    fake_cgroup(slice_directory, 9 * GIBIBYTE)
    layout = {"singlefs-memory-cap-401-1.scope": (2 * GIBIBYTE, ["4011", "4012"]),
              "singlefs-memory-cap-402-2.scope": (3 * GIBIBYTE, ["4021"]),
              "singlefs-memory-cap-403-3.scope": (GIBIBYTE, ["4031"])}
    environments = {"4012": [b"HOME=/selftest-home", f"{RUN_LABEL_VARIABLE}=task-selftest-7-1".encode("ascii")],
                    "4021": [f"{RUN_LABEL_VARIABLE}=task-selftest-8-2".encode("ascii")],
                    "4031": [b"HOME=/selftest-home"]}   # 4011 在 cgroup.procs 里、假 /proc 里没有它（读之前退了）
    for name, (used, process_ids) in layout.items():
        fake_cgroup(os.path.join(slice_directory, name), used)
        with open(os.path.join(slice_directory, name, "cgroup.procs"), "w", encoding="utf-8") as handle:
            handle.write("".join(process_id + "\n" for process_id in process_ids))
    for process_id, entries in environments.items():
        os.makedirs(os.path.join(proc_root, process_id))
        with open(os.path.join(proc_root, process_id, "environ"), "wb") as handle:
            handle.write(b"\0".join(entries) + b"\0")
    scopes = slice_scopes(slice_directory, proc_root)
    wanted = {"singlefs-memory-cap-401-1.scope": {SCOPE_USED_KEY: 2 * GIBIBYTE, SCOPE_LABEL_KEY: "task-selftest-7-1"},
              "singlefs-memory-cap-402-2.scope": {SCOPE_USED_KEY: 3 * GIBIBYTE, SCOPE_LABEL_KEY: "task-selftest-8-2"},
              "singlefs-memory-cap-403-3.scope": {SCOPE_USED_KEY: GIBIBYTE, SCOPE_LABEL_KEY: ""}}
    not_started = (slice_scopes(""), slice_scopes(None), slice_scopes(os.path.join(work, "scopes-slice-absent"), proc_root))
    return [(f"在跑的 scope 逐个报回收不掉的用量与标签（标签取进程环境里 {RUN_LABEL_VARIABLE} 的值，没有的是空串）",
             scopes == wanted and not_started == ({}, {}, {}),
             f"三个 scope 交 {scopes}（要的 {wanted}）；slice 还没起、查不了 systemd、目录不在交 {not_started}（都要空表）")]


def key_fold_cells(work):
    """文件头「键」③那几格：只差条目、文件、开关值的落一个键，别的照分开，幂等。交 [(格名, 对不对, 说明)]。
    样本只用仓里相对的路径与仓根、家目录下的路径（不经工作根、不带 capped.sh）：别的开关不弄红这几格。"""
    cells = []
    repository = repository_root()
    home = os.path.normpath(os.path.expanduser("~"))
    replay = ["bash", "scripts/crash/crash-amplification.sh"]
    harness = ["cargo", "test", "-p", "singlefs-verification", "--test", "checker_known_bad_images", "--"]
    sections = ["python3", "-I", "scripts/kb/kb-sections.py"]
    selftest = ["python3", "scripts/hosts/memory_peaks_lib.py", "--selftest"]
    lone_server = ["python3", "task-server/task_server_lib.py", "--selftest"]
    mutation_replay = ["bash", ".claude/gate.d/crates-mutation-replay.sh"]
    point_replay = ["bash", ".claude/gate.d/crash-point-replay.sh", "--check"]
    groups = [
        [replay + ["--item", "alpha"], replay + ["--item", "beta", "--item", "gamma"], replay + ["--item=delta"]],
        [harness + ["stale_copy_of_the_child", "--exact"], harness + ["child_that_does_not_read_back", "another_case", "--exact"]],
        [["cargo", "test", "--offline", "plan_hands_out_sets", "--lib"], ["cargo", "test", "--offline", "walk_segment_by_segment", "--lib"]],
        [["cargo", "nextest", "run", "-p", "singlefs-core", "ring_reuse"], ["cargo", "nextest", "run", "-p", "singlefs-core", "slot_bad"]],
        [[f"{repository}/target/debug/deps/journal_ring_fields-0123456789abcdef", "--exact", "ring"],
         [f"{repository}/target/debug/deps/journal_ring_fields-fedcba9876543210", "--exact", "slot"]],
        [sections + ["records/a.md"], sections + [f"{repository}/.claude/kb/b.md", "c.tsv"], sections + [f"{home}/notes.txt"]],
        [["env", "MEMORY_PEAKS_LIB_BREAK=capped-kept"] + selftest, ["env", "MEMORY_PEAKS_LIB_BREAK=scratch-paths-kept,kernel-counted"] + selftest],
        [["TASK_SERVER_SELFTEST_CELLS=submit,wait"] + lone_server, ["TASK_SERVER_SELFTEST_CELLS=cancel"] + lone_server],
        [["env", "GATE_MUTATION_ROW_SELECTION=1,2"] + mutation_replay + [repository], ["env", "GATE_MUTATION_ROW_SELECTION=7"] + mutation_replay + [f"{home}/elsewhere"]],
        [point_replay + ["vague-names,data-names", repository], point_replay + ["vague-names,data-names", f"{repository}/sub"]],
        [["bash", "-c", "cat records/a.md && cargo test --offline one_name"], ["bash", "-c", "cat records/b.md && cargo test --offline two_name"]],
    ]
    group_keys = [[command_key(words) for words in group] for group in groups]
    pinned = {group_keys[0][0]: f"bash scripts/crash/crash-amplification.sh --item {ITEM_TOKEN}",
              group_keys[3][0]: f"cargo nextest run -p singlefs-core {ITEM_TOKEN}",
              group_keys[5][0]: f"python3 -I scripts/kb/kb-sections.py {FILE_TOKEN}",
              group_keys[6][0]: f"env MEMORY_PEAKS_LIB_BREAK={BREAK_VALUE_TOKEN} python3 scripts/hosts/memory_peaks_lib.py --selftest",
              group_keys[10][0]: f"bash -c cat {FILE_TOKEN} && cargo test --offline {ITEM_TOKEN}"}
    one_per_group = all(len(set(keys)) == 1 for keys in group_keys)
    distinct_groups = len({keys[0] for keys in group_keys}) == len(groups)
    cells.append(("只差条目、文件、开关值的命令落一个键",
                  one_per_group and distinct_groups and all(key == wanted for key, wanted in pinned.items()),
                  f"每组换算后的键 {[sorted(set(keys)) for keys in group_keys]}；钉住的 {[(key, wanted) for key, wanted in pinned.items() if key != wanted]}（空表是都对）"))

    pairs = [
        (["cargo", "test", "-p", "singlefs-core", "--lib"], ["cargo", "test", "-p", "singlefs-verification", "--lib"]),
        (["cargo", "test", "-p", "singlefs-verification", "--test", "new_pool_file_creation_publish"], ["cargo", "test", "-p", "singlefs-verification", "--test", "checker_known_bad_images"]),
        (["cargo", "test", "--offline", "--manifest-path", "scripts/experiments/Cargo.toml", "--features", "e163-gpu", "--bin", "e163"],
         ["cargo", "test", "--offline", "--manifest-path", "scripts/experiments/Cargo.toml", "--features", "e174-gpu", "--bin", "e163"]),
        (["cargo", "test", "--offline", "--manifest-path", "scripts/experiments/Cargo.toml", "--bin", "e163"],
         ["cargo", "test", "--offline", "--manifest-path", "tree/Cargo.toml", "--bin", "e163"]),
        (["bash", "scripts/crash/crash-point-replay.sh", "--item", "a"], ["bash", "scripts/crash/crash-point-replay.sh", "--list-items"]),
        (["python3", "scripts/kb/kb-sections.py", "records/a.md"], ["python3", "scripts/kb/stale-candidates.py", "records/a.md"]),
        (["nice", "-n", "19", "bash", "scripts/hosts/one.sh", "a.tsv"], ["nice", "-n", "19", "bash", "scripts/hosts/two.sh", "a.tsv"]),
        (["target/release/rollback_hole", "4", "1048576"], ["target/release/rollback_hole", "8", "1048576"]),
        (["env", "MEMORY_PEAKS_LIB_BREAK=capped-kept"] + selftest, selftest),
    ]
    pair_keys = [(command_key(left), command_key(right)) for left, right in pairs]
    cells.append(("只差包、测试目标、特性、清单、范围项、脚本、数与开关变量的落不同的键", all(left != right for left, right in pair_keys),
                  f"落到同一个键的 {[left for left, right in pair_keys if left == right]}（空表是都对）"))

    explicit = {"broad-mutation baseline one-test": command_key([], "broad-mutation baseline one-test"),
                "e197/split/3/half/64/2": command_key([], "e197/split/3/half/64/2")}
    every_key = [key for keys in group_keys for key in keys] + [key for pair in pair_keys for key in pair]
    changed = [key for key in every_key if normalized_key(key) != key]
    cells.append(("条目、文件、开关的换算幂等，显式给的键照原样", not changed and all(key == value for key, value in explicit.items()),
                  f"再换一次变了的 {changed}；显式键 {explicit}"))

    cells.append(spaced_words_cell(mutation_replay, repository, home))
    cells.append(cell_names_cell(repository, home))
    return cells


def spaced_words_cell(mutation_replay, repository, home):
    """词表里带空格的词仍是一个词（文件头「键」③）：crates 变异门禁的条目名带空格（--item 写成两个词的、写成 --item=<名> 的），只差条目名的落一个键；
    带空格的路径折成一个〈文件〉。样本不经工作根、不带 capped.sh（别的开关不弄红这一格）。"""
    unit_graph_item = "单元图 闭包外的边：条目级宏调用生成的类型不点名"
    compaction_switch_item = "批间切换整理搬迁流：请求的切换从不生效 GPU 那一批"
    item_keys = [command_key(mutation_replay + ["--quick", "--item", unit_graph_item, "--item", compaction_switch_item, repository]),
                 command_key(mutation_replay + ["--quick", "--item", compaction_switch_item, f"{home}/elsewhere"]),
                 command_key(mutation_replay + ["--quick", f"--item={unit_graph_item}", repository])]
    wanted_items = f"bash .claude/gate.d/crates-mutation-replay.sh --quick --item {ITEM_TOKEN} {FILE_TOKEN}"
    sections = ["python3", "-I", "scripts/kb/kb-sections.py"]
    path_keys = [command_key(sections + ["records/带 空格的 记录.md"]), command_key(sections + ["records/a.md"])]
    wanted_path = f"python3 -I scripts/kb/kb-sections.py {FILE_TOKEN}"
    return ("条目名里带空格的同一条命令落一个键，带空格的一个词仍是一个词",
            set(item_keys) == {wanted_items} and path_keys == [wanted_path] * 2,
            f"带空格的条目名三种写法的键 {item_keys}（要的都是 {wanted_items}）；带空格的路径与不带的 {path_keys}（要的都是 {wanted_path}）")


def cell_names_cell(repository, home):
    """--check 后面的格名进键（文件头「键」③）：不同的格或格的组合是不同的键；两种写法（--check <格>、--check=<格>）与不同的根落一个键；
    路径形态的值当文件；--item 后面的照旧折。"""
    code_discipline = ["bash", ".claude/gate.d/code-source-discipline.sh"]
    light_cells, heavy_cell = "vague-names,test-file-names,ordinal-names,data-names", "mutation-tables"
    keys = {"light": command_key(code_discipline + ["--check", light_cells, repository]),
            "heavy": command_key(code_discipline + ["--check", heavy_cell, repository]),
            "heavy-other-root": command_key(code_discipline + ["--check", heavy_cell, f"{home}/elsewhere"]),
            "heavy-attached": command_key(code_discipline + [f"--check={heavy_cell}", repository]),
            "path-value": command_key(code_discipline + ["--check", "records/a.md", repository]),
            "item-instead": command_key(code_discipline + ["--item", heavy_cell, repository])}
    wanted_heavy = f"bash .claude/gate.d/code-source-discipline.sh --check {heavy_cell} {FILE_TOKEN}"
    wanted_path_value = f"bash .claude/gate.d/code-source-discipline.sh --check {FILE_TOKEN}"   # 值与后面的根都是〈文件〉，相邻的并成一个
    wanted_item = f"bash .claude/gate.d/code-source-discipline.sh --item {ITEM_TOKEN} {FILE_TOKEN}"
    return ("--check 后面的格名进键、不同格落不同键，--item 后面的照旧折",
            keys["light"] != keys["heavy"] and keys["heavy"] == keys["heavy-other-root"] == keys["heavy-attached"] == wanted_heavy
            and keys["path-value"] == wanted_path_value and keys["item-instead"] == wanted_item,
            f"各例的键 {keys}（轻的几格与重的一格要不同；重的一格换根、写成 --check=<格> 的都是 {wanted_heavy}；路径形态的值当文件，是 {wanted_path_value}；"
            f"--item 的是 {wanted_item}）")


def row_shape_cells(work):
    """表里只认这一版的行（文件头「峰值表的口径与列」）：好行读得到；峰值表里四列的、五列的、六列的、第六列写成两段的、第一列不是数的、第七列是空的，
    整张表读不了：抛 TableRowRefused，消息点名文件、行号、要几列与出路。交 [(格名, 对不对, 说明)]。"""
    gib = 1024 ** 3
    unreclaimable = MEASURED_UNRECLAIMABLE
    host = SELFTEST_HOST
    good_peak = f"{3 * gib}\t2026-10-29T00:00:00Z\t8G\tselftest-good\t4\t{3 * gib}:clean:{unreclaimable},{2 * gib}:hit:{unreclaimable}\t{host}"  # clock-times:allow 自检输入假时间戳
    bad_peaks = {"四列": (f"{gib}\t2026-10-01\t4G\tselftest-four-columns", 4),
                 "五列": (f"{gib}\t2026-10-01\t4G\tselftest-five-columns\t31", 5),
                 "六列（没写哪一台）": (f"{gib}\t2026-10-01\t4G\tselftest-six-columns\t31\t{gib}:clean:{unreclaimable}", 6),
                 "第六列写成两段": (f"{gib}\t2026-10-01\t4G\tselftest-two-segments\t31\t{gib}:clean\t{host}", 7),
                 "第一列不是数": (f"x\t2026-10-01\t4G\tselftest-first-column\t31\t{gib}:clean:{unreclaimable}\t{host}", 7),
                 "第七列是空的": (f"{gib}\t2026-10-01\t4G\tselftest-host-empty\t31\t{gib}:clean:{unreclaimable}\t", 7)}

    def refusal_of(path):
        try:
            read_peak_rows(path, host)
        except TableRowRefused as error:
            return str(error)
        return None

    def table_with(name, header, *lines):
        path = os.path.join(work, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("\n".join([header, *lines]) + "\n")
        return path

    good_path = table_with("row-shape-good-peaks.tsv", "# 自证造的峰值表", good_peak)
    rows = read_peak_rows(good_path, host)
    good_ok = (rows == {"selftest-good": (3 * gib, "2026-10-29T00:00:00Z", "8G", "4", [(3 * gib, False, unreclaimable), (2 * gib, True, unreclaimable)])}  # clock-times:allow 自检输入假时间戳
               and recorded_peak(good_path, "selftest-good", host=host) == (3 * gib, True))
    problems = [] if good_ok else [f"好行读回 {rows}"]
    for index, (name, (line, columns)) in enumerate(bad_peaks.items()):
        path = table_with(f"row-shape-peaks-{index}.tsv", "# 自证造的峰值表", good_peak, line)
        message = refusal_of(path)
        if message is None or f"{path} 第 3 行" not in message or "要 7 列" not in message or f"这一行 {columns} 列" not in message or "出路" not in message:
            problems.append(f"峰值表里{name}的那一行读回的拒绝是 {message}")
    return [("表里只认这一版的行，不对的行整张表读不了并点名行号、要几列与出路", not problems,
             "；".join(problems) or f"好行读得到，{len(bad_peaks)} 种不对的行都被拒")]


def host_column_cells(work):
    """两台共用一张峰值表（文件头「峰值表在哪」「哪一台」）：同一个键两台各一行，读的时候只认本台的那一行；默认的表在共享数据根下。交 [(格名, 对不对, 说明)]。"""
    cells = []
    gib = 1024 ** 3
    unreclaimable = MEASURED_UNRECLAIMABLE
    path = os.path.join(work, "host-column-peaks.tsv")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("# 自证造的峰值表：同一个键两台各一行\n")
        handle.write(peak_table_line("selftest-shared", (2 * gib, "2026-11-01T00:00:00Z", "8G", "4", [(2 * gib, False, unreclaimable)]), SELFTEST_HOST) + "\n")  # clock-times:allow 自检输入假时间戳
        handle.write(peak_table_line("selftest-shared", (9 * gib, "2026-11-02T00:00:00Z", "16G", "4", [(9 * gib, True, unreclaimable)]), SELFTEST_OTHER_HOST) + "\n")  # clock-times:allow 自检输入假时间戳
    mine = recorded_peak(path, "selftest-shared", host=SELFTEST_HOST)
    other = recorded_peak(path, "selftest-shared", host=SELFTEST_OTHER_HOST)
    everything = read_all_peak_rows(path)
    cells.append(("同一个键两台各一行：读的时候只认本台的那一行，另一台的照留",
                  mine == (2 * gib, False) and other == (9 * gib, True) and set(everything) == {(SELFTEST_HOST, "selftest-shared"), (SELFTEST_OTHER_HOST, "selftest-shared")},
                  f"本台读回 {mine}（要 ({2 * gib}, False)），另一台读回 {other}（要 ({9 * gib}, True)），两台的行 {sorted(everything)}"))
    saved = os.environ.get(DATA_ROOT_VARIABLE)
    os.environ[DATA_ROOT_VARIABLE] = os.path.join(work, "shared-data-root")
    try:
        try:
            located = default_peak_table()
        except PeakTableUnavailable as error:
            located = f"取不到：{error}"
    finally:
        if saved is None:
            os.environ.pop(DATA_ROOT_VARIABLE, None)
        else:
            os.environ[DATA_ROOT_VARIABLE] = saved
    wanted = os.path.join(work, "shared-data-root", PEAK_TABLE_NAME)
    cells.append(("默认的峰值表在共享数据根下（lib_work_root.py 的 shared_data_path），不在家目录下",
                  located == wanted, f"SINGLEFS_DATA_ROOT 指到 {os.path.join(work, 'shared-data-root')} 时取到「{located}」（要 {wanted}）"))
    return cells


def measurement_cells(work):
    """口径那几格（文件头「口径」）：假 cgroup 目录上采样、judged_measurement 定记什么、recorded_peak 用哪几次。交 [(格名, 对不对, 说明)]。"""
    cells = []
    mib, gib = MEBIBYTE, 1024 ** 3
    cap = 64 * mib
    fake = os.path.join(work, "fake-scope")
    os.makedirs(fake, exist_ok=True)

    def fake_state(current, active_file, inactive_file, oom_kill=0):
        with open(os.path.join(fake, "memory.current"), "w", encoding="utf-8") as handle:
            handle.write(f"{current}\n")
        with open(os.path.join(fake, "memory.stat"), "w", encoding="utf-8") as handle:
            handle.write(f"anon {current - active_file - inactive_file}\nfile {active_file + inactive_file}\nkernel 0\n"
                         f"active_file {active_file}\ninactive_file {inactive_file}\nshmem 0\n")
        with open(os.path.join(fake, "memory.peak"), "w", encoding="utf-8") as handle:
            handle.write(f"{cap}\n")
        with open(os.path.join(fake, "memory.events"), "w", encoding="utf-8") as handle:
            handle.write(f"low 0\nhigh 0\nmax 9\noom 0\noom_kill {oom_kill}\n")

    # 页缓存把 memory.current 涨到上限 64M、其中 48M 是页缓存：三次采样依次是 40M 里 30M 页缓存、64M 里 48M、64M 里 60M，回收不掉的最大 16M
    states = [(40 * mib, 20 * mib, 10 * mib), (cap, 32 * mib, 16 * mib), (cap, 40 * mib, 20 * mib)]
    fake_state(*states[0])
    remaining = list(states[1:])

    def next_state():
        if not remaining:
            return False
        fake_state(*remaining.pop(0))
        return True

    output = os.path.join(work, "sampled-peak")
    sampled = sample_unreclaimable_peak(lambda: fake, output, next_state, sleep=lambda seconds: None)
    try:
        with open(output, encoding="utf-8") as handle:
            written = handle.read()
    except OSError as error:
        written = f"读不了：{error}"
    judged = judged_measurement("clean", cap, cap, sampled, 0)
    cells.append(("页缓存占了大半时记回收不掉的那一截，不是上限",
                  sampled == 16 * mib and written == f"peak {16 * mib}\n" and judged == (16 * mib, False, MEASURED_UNRECLAIMABLE),
                  f"采到的最大 {sampled}（要 {16 * mib}）、写下的「{written.strip()}」；memory.peak 等于上限 {cap}、oom_kill 0 时记的是 {judged}（要 ({16 * mib}, False, '{MEASURED_UNRECLAIMABLE}')）"))

    missing = sample_unreclaimable_peak(lambda: os.path.join(work, "no-such-scope"), os.path.join(work, "never-sampled"), lambda: False, sleep=lambda seconds: None)
    not_recorded = judged_measurement("clean", cap, cap, missing, 0)
    oom_clean = judged_measurement("clean", cap, 40 * mib, 30 * mib, 1)
    own_cap = judged_measurement("own-cap", cap, cap, 30 * mib, 0)
    cells.append(("oom_kill 大于 0 记撞顶、峰值记上限（Result 没报 oom-kill 也算），采不到的不记",
                  oom_clean == (cap, True, MEASURED_UNRECLAIMABLE) and own_cap == (cap, True, MEASURED_UNRECLAIMABLE)
                  and missing is None and not_recorded is None and not os.path.exists(os.path.join(work, "never-sampled")),
                  f"整段跑完而 oom_kill 1、memory.peak 40M 的记 {oom_clean}，撞了自己的上限的记 {own_cap}（都要 ({cap}, True, '{MEASURED_UNRECLAIMABLE}')）；"
                  f"scope 目录不在时采到 {missing}、记 {not_recorded}（都要 None）"))

    table = os.path.join(work, "measurement-peaks.tsv")
    full_page_cache = judged_measurement("clean", cap, cap, 16 * mib, 0)
    with open(table, "w", encoding="utf-8") as handle:
        handle.write(f"{16 * mib}\t2026-10-29T00:00:00Z\t64M\tselftest-page-cache\t4\t"  # clock-times:allow 自检输入假时间戳
                     + format_recent([full_page_cache or (cap, True, MEASURED_UNRECLAIMABLE), (12 * mib, False, MEASURED_UNRECLAIMABLE)]) + f"\t{SELFTEST_HOST}\n")
        handle.write(f"{20 * gib}\t2026-10-29T00:00:00Z\t36G\tselftest-new-basis\t31\t{20 * gib}:clean:{MEASURED_UNRECLAIMABLE},{12 * gib}:clean:{MEASURED_UNRECLAIMABLE}\t{SELFTEST_HOST}\n")  # clock-times:allow 自检输入假时间戳
    page_cache_peak, page_cache_hit = recorded_peak(table, "selftest-page-cache", host=SELFTEST_HOST)
    cells.append(("峰值到了上限而 oom_kill 是 0 记 clean、不把上限往上推",
                  full_page_cache == (16 * mib, False, MEASURED_UNRECLAIMABLE) and page_cache_peak == 16 * mib and not page_cache_hit,
                  f"memory.peak 等于上限 64M、oom_kill 0、采到 16M 的记 {full_page_cache}；表里读回 ({page_cache_peak}, 撞过顶 {page_cache_hit})"
                  f"（要 ({16 * mib}, 撞过顶 False)，不是上限 64M）"))

    new_peak, new_hit = recorded_peak(table, "selftest-new-basis", host=SELFTEST_HOST)
    new_text = f"{20 * gib}:clean:{MEASURED_UNRECLAIMABLE},{12 * gib}:clean:{MEASURED_UNRECLAIMABLE}"
    round_trip = format_recent(parse_recent(new_text) or [])
    cells.append(("新口径记录读回近几次里最大的、写回原样",
                  new_peak == 20 * gib and not new_hit and round_trip == new_text,
                  f"近几次「20G:clean:{MEASURED_UNRECLAIMABLE},12G:clean:{MEASURED_UNRECLAIMABLE}」读回 ({new_peak}, 撞过顶 {new_hit})（要 ({20 * gib}, False)）；"
                  f"读进来再写回「{round_trip}」"))
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
        print("  → 怎么办：按不对的那一格修 command_key / normalized_words / without_item_and_file_arguments / read_peak_rows / sample_unreclaimable_peak / judged_measurement / parse_recent / "
              "measured_margin / slice_margin_bytes / rounded_up_to_gibibyte / slice_scopes / scope_run_label；"
              "MEMORY_PEAKS_LIB_BREAK 设着的话本来就该红")
        return 1
    print(f"  ✓ memory_peaks_lib.py 自证 {len(cells)} 格都对（副本与草稿目录路径换成记号、剥 capped.sh N、只差条目文件开关值的落一个键而包与目标与档位分开、带空格的条目名一个词落一个键、--check 后面的格名进键、表里只认这一版的行而不对的行整张表读不了、"
          "同一个键两台各一行而只认本台的那一行、默认的峰值表在共享数据根下、页缓存占了大半时记回收不掉的那一截、oom_kill 大于 0 记撞顶而采不到的不记、峰值到了上限而没 OOM 的不把上限往上推、新口径读回近几次里最大的、"
          "工作根取法与 lib_work_root.py 一样、平常跑设了弄坏开关退 2 并点名变量、"
          f"余量按 1 GiB 向上取整（{len(MARGIN_ROUNDING_SAMPLES)} 个读数各一格）、余量只算子 cgroup 里没有的那一截、读不了时量不到余量、在跑的 scope 逐个报用量与标签）")
    return 0


if __name__ == "__main__" and globals().get("__file__"):
    if sys.argv[1:] != ["--selftest"]:
        for refusal in refuse_break_outside_selftest(sys.argv[1:], os.environ):
            print(refusal, file=sys.stderr)
        print("  ✗ memory_peaks_lib.py 是被 import 的库，单独跑只认 --selftest", file=sys.stderr)
        print("  → 怎么办：python3 scripts/hosts/memory_peaks_lib.py --selftest；取键、读表 import 它（文件头「用法」）", file=sys.stderr)
        sys.exit(2)
    sys.exit(run_self_test())
