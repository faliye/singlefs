#!/usr/bin/env python3
# admission: always 判的是此刻被判的仓、它的 crates/mutations.tsv 与 git common-dir 里的按条记录，上一次的结论不替这一次作保
# run-condition: none 算底座指纹要的 git、cargo、rustc 由 admission.py 的 listed_input_files 与 toolchain_line 起、起不来报错退 2；--selftest 用自己造的假 cargo 与假内存包装，不编译
"""门禁 checker-tier-crates-mutation-replay（crates 变异表复跑）的判法与按条记录，连同门禁 code-source-discipline 静态判的那几样：一份判法只在这里，两道阶段都调它。

变异表一行成形 = 六段制表符分隔（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名）；替换文可以空（删掉原文：改坏时原文换成空串），
其余五段一段都不许空（malformed_reason，read_table 照它读；checker-tier-crates-mutation-replay、static-check、import 同一份）。

checker-tier-crates-mutation-replay 经 import 调 run_stage（阶段文件里那几行内嵌 python）；这里不给整张表开命令行入口：重型测试闸（.claude/hooks/heavy-test-guard.sh）
按阶段文件名认 checker-tier-crates-mutation-replay，另开一个入口它认不出。

子命令（<根> 是被判仓的根）：
  static-check <根> [--table <变异表>]  checker-tier-crates-mutation-replay 派活之前判的每一样（row_problems，checker-tier-crates-mutation-replay 的 prechecks 调的是同一个函数）连同拷贝范围 ②，
                                        每处一行（制表符分隔），退 0 一处都没有、退 1 有、退 2 读不了登记表或变异表：
                                          MALFORMED <行号> <变异名> <为什么>             这一行不成形（见上）
                                          ANCHOR   <行号> <变异名> <为什么>              原文在「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
                                          NAME     <行号> <变异名> <测试名>              必须红的测试名不是一个用例名（带了正则符号或空白）
                                          ROW      <行号> <变异名> <文件>                ① 变异表这一行改的文件不在 checker-tier-crates-mutation-replay 每片拷的范围里
                                          STRAY    <行号> <变异名> <段名> <转义>         原文或替换文里有 \\n 以外的反斜杠转义
                                          LITERAL  <文件> <行号> <字面量> <解析到的路径> ② crates/ 下 .rs 里以 "../ 起头、按所在 crate 的根解析之后
                                                                                         逃出 crates/、又不在拷贝范围里的字符串字面量（只 code-source-discipline 判，checker-tier-crates-mutation-replay 靠基线）
                                          COPIED   <拷贝范围，空格分隔>
                                          CHECKED  <成形的行数> <.rs 份数> <逃出 crates/ 的字面量处数（在范围里的与不在的合计）>
                                        门禁 code-source-discipline 调它；--table 不给就读 <根>/crates/mutations.tsv
  base-fingerprint <根>                 这棵树此刻的底座指纹：打「<sha256> <文件数>」；算不出退 2
  records-directory <根>                这棵树此刻那一格按条记录目录的路径（双机分片的驱动拷记录用）；算不出退 2
  row-plan <根>                         双机分片的驱动（research/scripts/mutation-shard-run.sh）分行用：每一条成形的变异打一行
                                          <行号> <行键> <有没有作数的抓到记录，1 或 0> <上一次现跑的秒数，没记过写 ->
                                        （制表符分隔；「作数」与 run_stage 复用的判法同一份，SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 时一律 0）；
                                        退 0；读不了表、上限写错、算不出底座指纹退 2
  import <根> <别处的记录目录>          把另一台写的按条记录导进本机：逐条核它的底座指纹等于本机此刻算的、它的行键是本机变异表里某一行
                                        （连同本机这一趟的每条内存上限与限时）算得出的、文件名与内容对得上；符的写进本机那一格记录目录
                                        （同目录临时文件写完改名），不符的拒、逐条列出。退 0 全收；退 1 有拒的；退 2 用法错或算不出底座指纹
  --selftest                            自证（假 cargo、假内存包装，在临时仓里跑 run_stage；格数由成功行现算）

拷贝范围 = .claude/gate.d/stage-inputs.tsv 里 checker-tier-crates-mutation-replay 那一行登记的路径（读本模块所在那一份仓的登记表：判别力样本的目录里没有登记表），
  底下每一段都不叫 target（各 crate 自己的编译目录）。checker-tier-crates-mutation-replay 照它拷每片的源码副本，code-source-discipline 照它静态判；判法只有 is_copied_into_each_shard 一处。
  ② 管不到的：路径拼在变量里、环境变量给的、format! 拼出来的、不以 "../ 起头而运行期再 join("..") 的；#[path = …] 不算（编译期带进来的源文件，
  副本里编得过就在）；指到仓根本身（"../../" 这一类）按不在范围里算（之后拼上的是哪一份判不出）。

按条记录（checker-tier-crates-mutation-replay 每条判完立刻写，下一趟只跑没有作数记录的行）：
  放在被判那棵树的 git common-dir 下 singlefs-mutation-rows/<底座指纹>/，一条一个文件：mutation.<行键>（变异的判定）、baseline.<行键>（基线）。
  底座指纹 = checker-tier-crates-mutation-replay 登记的输入路径下逐文件清单（git 列的，减去 crates/mutations.tsv）+ 登记行 + 阶段脚本 + 本模块 + 这一趟用的内存包装
    + 工具链 + 构建环境。算法照 admission.py 的 stage_fingerprint，import 它的 listed_input_files、manifest_of_files、toolchain_line、
    build_environment_lines 来用；阶段脚本、本模块与内存包装按内容进（名字里不带绝对路径，主工作区与临时 worktree、两台机器上算出同一个数）。
  行键 = sha256（这一行六段原样用制表符接起来 + 这一趟的每条内存上限 + 每条限时）。
  只有「抓到」与基线 ok 的记录复用，别的档每趟重跑；过了 SINGLEFS_REUSE_HOURS（默认 24）小时不复用（口径照 admission.py 的 check_stage_marker）；
  SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 一律不复用（照旧写新记录）。
  singlefs-mutation-rows/ 底下只留最新的 3 个底座指纹目录（按修改时刻，这一趟的那个算最新），更早的开跑时删；别处一概不删。记录里不写主机名。
  GATE_MUTATION_RECORDS_HOME=<目录> 设了，按条记录与用时表放在它底下、不放 git common-dir（双机分片的驱动给第二台设：那棵树每趟新拷、跑完删，
    记录与用时表要跨趟留着）；run_stage、records-directory、import、row-plan 都认它。
  用时表：同一个 git common-dir 下 singlefs-mutation-row-timings/<这一行六段原样的 sha256>，跨底座指纹沿用，记上一次现跑的用时与超没超时；
    import 不导用时（另一台的快慢不代表本机）。
  用时从包装起跑到返回分两段记：排队（调包装到命令在 scope 里起来）与跑（起来到返回）。命令起来的那一刻由包在命令外面的一层 bash 记进
    这一趟源码副本总目录底下的一个文件（不在那一条的 TMPDIR 里，不进「删前留有几个文件」）；包装的峰值表仍按原命令记键（RUN_WITH_MEMORY_CAP_KEY）。

基线（派变异之前，按批在没改坏的副本上跑一遍点名的测试）：
  批的键 = 这一行 cargo 参数里 `--` 之前那一段 + `--` 之后以 `--` 起头的开关；过滤词取这一批要跑基线的各行 `--` 之后非开关词的并集，
  有一行没有过滤词就整批不带过滤词。要单独带值的开关（`--skip 名字`、`--test-threads 4`）写成 `--test-threads=4`：分开写时那个值会被当成过滤词。
  同样经内存包装、同样的上限与限时。每一行的点名测试在基线输出里要恰好对上一个用例、结果都是 ok，否则这一行记「基线不绿」：
  不算抓到、不算没红、它的变异不跑，整道判红。已有作数的抓到记录或基线 ok 记录的行不再跑基线。

弄坏开关（GATE_MUTATION_BREAK，只给证红用，逗号分隔可给几个）：
  nobaseline           跳过基线：点名的测试在没改坏的副本上本来就红的行照样记抓到（checker-tier-crates-mutation-replay 红样本、本模块自证都必须判错）
  basewithoutsource    底座指纹不含 crates/ 下的被测源码：改一份被测源码之后照样全部复用（自证必须判错）
  timingsignored       派活不看用时表（自证「上一次跑得久的先派」那一格必须判错）
  importunchecked      import 不核行键与底座指纹，照单全收（自证「行键对不上的记录被拒」那一格必须判错）
  keepmutationtmp      每条变异的 TMPDIR 不删（checker-tier-crates-mutation-replay 两份样本都必须判错）
  emptyreplacementrefused  替换文空着也判不成形（checker-tier-crates-mutation-replay 绿样本里的删行变异、code-source-discipline 绿样本里的删行变异、本模块自证「删行变异照常判」那一格都必须判错）
只供测试的开关：GATE_MUTATION_STOP_AFTER_ROWS=<k> 现跑的变异判完 k 条就不再派活、等在跑的收完，整道判红退出（造「跑到一半被杀」）。
"""
import calendar
import collections
import concurrent.futures
import contextlib
import fnmatch
import hashlib
import heapq
import importlib.util
import io
import multiprocessing
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
STAGE_REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(STAGE_REPOSITORY, ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402

STAGE = "checker-tier-crates-mutation-replay.sh"   # 登记表的键、阶段脚本进底座指纹，都按这个文件名
STAGE_SCRIPT = os.path.join(STAGE_REPOSITORY, ".claude", "gate.d", STAGE)
MODULE_PATH = os.path.realpath(__file__)
DEFAULT_MEMORY_CAP_RUNNER = os.path.join(SCRIPT_DIRECTORY, "run-with-memory-cap.sh")
TABLE = "crates/mutations.tsv"
RECORDS_DIRECTORY_NAME = "singlefs-mutation-rows"
TIMINGS_DIRECTORY_NAME = "singlefs-mutation-row-timings"
BASE_DIRECTORIES_KEPT = 3
FINGERPRINT_FORM = re.compile(r"[0-9a-f]{64}")
RECORD_KINDS = ("mutation", "baseline")
REUSABLE_VERDICT = {"mutation": "caught", "baseline": "ok"}
RECORD_HEADER = ("# 门禁 checker-tier-crates-mutation-replay 的按条记录（research/scripts/crates-mutation-rows.py 写，一条一个文件）：只有「抓到」与基线 ok 的、"
                 "底座指纹与行键都对得上又没过期的复用。不进工作树，别手改。\n")
# 拷目录时跳过的名字（各 crate 自己的编译目录）。拷贝范围由它与登记的路径一起定：prepare_shard 照它们拷，is_copied_into_each_shard 照它们判。
IGNORED_WHEN_COPYING_INTO_EACH_SHARD = ["target"]
MEMORY_CAP_HIT_EXIT = 250            # run-with-memory-cap.sh：撞了这一条自己的上限（scope 的 Result 是 oom-kill、自己的 oom 计数不为 0）
MEMORY_CAP_UNAVAILABLE_EXIT = 251    # run-with-memory-cap.sh：带上限的 scope 或 slice 的总上限起不来
MEMORY_ADMISSION_REFUSED_EXIT = 252  # run-with-memory-cap.sh：排队等满还放不下，命令没跑
TIME_LIMIT_HIT_EXIT = 253            # run-with-memory-cap.sh：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT（scope 的 Result 是 timeout）
SLICE_TOTAL_HIT_EXIT = 254           # run-with-memory-cap.sh：被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀）
DEFAULT_MEMORY_MAX_PER_MUTATION = "4G"         # 依据见 checker-tier-crates-mutation-replay 文件头
MEMORY_MAX_DIRECTIVE = re.compile(r"^#\s*每条变异的内存上限[：:]\s*(\S+)\s*$")
DEFAULT_TIMEOUT_SECONDS_PER_MUTATION = "1800"  # 依据见 checker-tier-crates-mutation-replay 文件头
TIMEOUT_DIRECTIVE = re.compile(r"^#\s*每条变异的超时秒数[：:]\s*(\S+)\s*$")
DEFAULT_LONG_ROW_SECONDS = "300"               # 上一次用时不短于它（或超时）的行先派：GATE_MUTATION_LONG_ROW_SECONDS
TIMEOUT_KILL_GRACE_SECONDS = 30                # 到点发 TERM 之后再等这么久，还不退就发 KILL（交给包装的 RUN_WITH_MEMORY_CAP_KILL_GRACE）
PER_MUTATION_TEMPORARY_DIRECTORY_NAME = "per-mutation-tmp"
START_STAMP_DIRECTORY_NAME = "started-at"
# 包在命令外面的那一层：命令在 scope 里起来的那一刻写进 $1，再 exec 原命令（用时分排队与跑两段，见文件头）
START_STAMP_SCRIPT = 'printf "%s" "$EPOCHREALTIME" > "$1"; shift; exec "$@"'
VERDICTS_KILLED_BY_WRAPPER = ("timeout", "memory", "squeezed")
VERDICTS_THAT_DID_NOT_RUN = ("refused", "unavailable")
TEST_NAME_FORM = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*")
BYTES_PER_STAT_BLOCK = 512   # os.stat 的 st_blocks 按 512 字节一块数（POSIX）
EXIT_SELECTION_ONLY = 3      # 只判了选中的行、都抓到：不是整道结论，门禁汇总记失败，不会被当成通过
# 源码里一处字符串字面量或注释的起头（找字面量时跳过注释用）
LITERAL_OR_COMMENT_START = re.compile(r'//|/\*|b?r#*"|b?"|\'')
PATH_ATTRIBUTE_BEFORE_LITERAL = re.compile(r"#\s*\[\s*path\s*=\s*$")
PATH_ATTRIBUTE_AFTER_LITERAL = re.compile(r"\s*\]")
# run_stage 读的环境变量（自证起每一格之前从调用方的环境里清掉，弄坏开关除外）
STAGE_VARIABLES = ("GATE_MUTATION_WORKERS", "GATE_MUTATION_CARGO_JOBS", "GATE_MUTATION_MEMORY_MAX", "GATE_MUTATION_TIMEOUT",
                   "GATE_MUTATION_ROW_SELECTION", "GATE_MUTATION_START_OVER", "GATE_MUTATION_LONG_ROW_SECONDS",
                   "GATE_MUTATION_STOP_AFTER_ROWS", "GATE_MUTATION_TARGET_DIR", "GATE_MUTATION_MEMORY_CAP_RUNNER",
                   "GATE_MUTATION_RECORDS_HOME", "SINGLEFS_GATE_FULL", "SINGLEFS_REUSE_HOURS")

Row = collections.namedtuple("Row", "line_number name path old new arguments expected fields")
FIELD_NAMES = ("变异名", "文件", "原文", "替换文", "cargo test 参数", "必须红的测试名")
REPLACEMENT_FIELD = FIELD_NAMES.index("替换文")   # 六段里只有它可以空：替换文空着就是把原文删掉
# 派活之前（checker-tier-crates-mutation-replay）与静态（code-source-discipline 经 static-check）判的那几样，打印与输出都照这个次序
ROW_PROBLEM_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY")
_ADMISSION_MODULE = []


def admission_module():
    """research/scripts/admission.py（底座指纹的算法、git common-dir、登记表都用它的），第一次用到时才读进来。"""
    if not _ADMISSION_MODULE:
        specification = importlib.util.spec_from_file_location("admission_for_crates_mutation_rows", os.path.join(SCRIPT_DIRECTORY, "admission.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _ADMISSION_MODULE.append(module)
    return _ADMISSION_MODULE[0]


def break_is(environment, switch_name):
    return switch_name in environment.get("GATE_MUTATION_BREAK", "").split(",")


def say(text):
    print(text, flush=True)


def note(text):
    print(text, file=sys.stderr, flush=True)


# ── 变异表 ────────────────────────────────────────────────────────────────────

def unescape(text):
    return text.replace("\\n", "\n")


def malformed_reason(fields, environment):
    """一行拆出来的几段不成形的原因，成形回 None。成形 = 六段；替换文可以空（删掉原文：改坏时原文换成空串），其余五段一段都不许空。
    弄坏开关 emptyreplacementrefused（只给证红用）让替换文空着也算不成形。"""
    if len(fields) != len(FIELD_NAMES):
        return f"不是六段（{len(fields)} 段）"
    empty = [FIELD_NAMES[index] for index, field in enumerate(fields)
             if field == "" and (index != REPLACEMENT_FIELD or break_is(environment, "emptyreplacementrefused"))]
    return "、".join(f"「{name}」" for name in empty) + "空着" if empty else None


def read_table(path, environment=None):
    """(成形的行, [(行号, 变异名, 为什么不成形)], 表头「每条变异的内存上限」, 表头「每条变异的超时秒数」)。成形照 malformed_reason：
    checker-tier-crates-mutation-replay、static-check（门禁 code-source-discipline）与 import 都照这一份读，没有第二份判法。environment 不给就读本进程的环境（只看弄坏开关）。"""
    environment = os.environ if environment is None else environment
    rows, malformed = [], []
    memory_max_from_table = timeout_from_table = None
    with open(path, encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, 1):
            line = line.rstrip("\n")
            directive = MEMORY_MAX_DIRECTIVE.match(line)
            if directive:
                memory_max_from_table = directive.group(1)
            directive = TIMEOUT_DIRECTIVE.match(line)
            if directive:
                timeout_from_table = directive.group(1)
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            reason = malformed_reason(fields, environment)
            if reason:
                malformed.append((line_number, fields[0] or "（变异名空着）", reason))
                continue
            rows.append(Row(line_number, fields[0], fields[1], unescape(fields[2]), unescape(fields[3]), fields[4], fields[5], tuple(fields)))
    return rows, malformed, memory_max_from_table, timeout_from_table


def limits_of(memory_max_from_table, timeout_from_table, environment):
    """(每条内存上限, 每条限时秒数, [(卡在哪, 下一步)])。三处取，先到先用：表头那一行、环境变量、默认。"""
    problems = []
    memory_max = memory_max_from_table or environment.get("GATE_MUTATION_MEMORY_MAX", "").strip() or DEFAULT_MEMORY_MAX_PER_MUTATION
    if not re.fullmatch(r"[1-9][0-9]*[KMGT]", memory_max):
        problems.append((f"每条变异的内存上限写成了「{memory_max}」，不是 systemd 的写法",
                         "写成正整数加 K / M / G / T，单位必写（不带单位的数 systemd 当字节数），例 4G、512M（表头那一行「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。"))
    timeout_text = timeout_from_table or environment.get("GATE_MUTATION_TIMEOUT", "").strip() or DEFAULT_TIMEOUT_SECONDS_PER_MUTATION
    timeout_seconds = int(timeout_text) if re.fullmatch(r"[1-9][0-9]*", timeout_text) else None
    if timeout_seconds is None:
        problems.append((f"每条变异的限时写成了「{timeout_text}」，不是正整数秒",
                         "写成正整数秒，例 1800（表头那一行「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT）。"))
    return memory_max, timeout_seconds, problems


# ── 拷贝范围（checker-tier-crates-mutation-replay 拷进每片的、code-source-discipline 静态判的，同一份判法）──────────────────────

def copied_paths(stage_repository=STAGE_REPOSITORY):
    """登记表 checker-tier-crates-mutation-replay 那一行登记的路径（目录去掉末尾的斜杠）。登记表读不了、没有那一行抛 admission 的 RegistrationError。"""
    admission = admission_module()
    input_paths, _rows = admission.resolve_input_paths(admission.read_registration_rows(stage_repository), STAGE)
    return [input_path.strip().rstrip("/") for input_path in input_paths if input_path.strip()]


def is_copied_into_each_shard(path, copied):
    """一个仓根起的路径在不在每片的源码副本里：落在 copied 的某一项之内，而且那一项底下的每一段路径都不叫
    IGNORED_WHEN_COPYING_INTO_EACH_SHARD 里的名字。绝对路径、用 `..` 跳出去的、仓根本身都不在。"""
    normalized = os.path.normpath(path)
    if os.path.isabs(normalized):
        return False
    for item in copied:
        if normalized == item:
            return True
        if normalized.startswith(item + os.sep):
            below_item = normalized[len(item) + 1:].split(os.sep)
            return not any(fnmatch.fnmatch(part, pattern) for part in below_item for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD)
    return False


def rows_outside_copy(rows, copied):
    return [row for row in rows if not is_copied_into_each_shard(row.path, copied)]


def stray_escapes(segment):
    """一段里 \\n 以外的反斜杠转义：本模块与 mutate.sh 只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑（C427）。"""
    return [match.group(0) for match in re.finditer(r"\\(.)", segment) if match.group(1) != "n"]


def row_problems(root, rows, malformed, copied):
    """派活之前一个工作进程都不起就能判的那几样，checker-tier-crates-mutation-replay（prechecks）与门禁 code-source-discipline（static-check）都调这一份。回 {类别: [明细]}，类别照 ROW_PROBLEM_KINDS：
      MALFORMED (行号, 变异名, 为什么)   不成形（read_table 判的：六段，只有替换文可以空）
      ANCHOR    (行号, 变异名, 为什么)   原文在被判仓里「文件」那一段指的源码里不是恰好命中一次，或文件不在、读不了
      NAME      (行号, 变异名, 测试名)   必须红的测试名不是一个用例名（带了正则符号或空白）
      ROW       (行号, 变异名, 文件)     文件不在每片拷的范围里
      STRAY     (行号, 变异名, 段名, 转义) 原文或替换文里有 \\n 以外的反斜杠转义"""
    problems = {kind: [] for kind in ROW_PROBLEM_KINDS}
    problems["MALFORMED"] = list(malformed)
    for row in rows:
        try:
            with open(os.path.join(root, row.path), encoding="utf-8") as handle:
                count = handle.read().count(row.old)
        except FileNotFoundError:
            problems["ANCHOR"].append((row.line_number, row.name, f"文件 {row.path} 不存在"))
        except (OSError, UnicodeDecodeError) as error:
            problems["ANCHOR"].append((row.line_number, row.name, f"文件 {row.path} 读不了（{type(error).__name__}）"))
        else:
            if count != 1:
                problems["ANCHOR"].append((row.line_number, row.name, f"原文在 {row.path} 里命中 {count} 次，要恰好 1 次"))
        if not TEST_NAME_FORM.fullmatch(row.expected):
            problems["NAME"].append((row.line_number, row.name, row.expected))
        for segment_name, segment in (("原文", row.fields[2]), ("替换文", row.fields[3])):
            for escape in stray_escapes(segment):
                problems["STRAY"].append((row.line_number, row.name, segment_name, escape))
    problems["ROW"] = [(row.line_number, row.name, row.path) for row in rows_outside_copy(rows, copied)]
    return problems


def string_literals(text):
    """源码里每一处字符串字面量（"…"、r#"…"#、b"…"）的 (起, 止)，注释里的不算；字面量的止点照 admission.py 的 end_of_literal。"""
    end_of_literal = admission_module().end_of_literal
    position = 0
    while True:
        found = LITERAL_OR_COMMENT_START.search(text, position)
        if found is None:
            return
        start, token = found.start(), found.group(0)
        if token == "//":
            line_end = text.find("\n", start)
            if line_end < 0:
                return
            position = line_end
            continue
        if token == "/*":
            depth, cursor = 0, start
            while cursor < len(text):
                if text.startswith("/*", cursor):
                    depth, cursor = depth + 1, cursor + 2
                elif text.startswith("*/", cursor):
                    depth, cursor = depth - 1, cursor + 2
                    if depth == 0:
                        break
                else:
                    cursor += 1
            position = cursor
            continue
        if token[0] in "br" and start > 0 and (text[start - 1].isalnum() or text[start - 1] == "_"):
            position = start + 1   # 标识符的末一个字母，不是字面量的前缀
            continue
        end = end_of_literal(text, start)
        if end is None:
            position = start + 1
            continue
        if token != "'":
            yield start, end
        position = end


def literal_value(literal):
    """字面量里引号之间的原文：r#"…"#、b"…" 都去掉前缀与引号；转义不还原（路径里用不到）。"""
    raw = re.match(r'^b?r(#*)"(.*)"\1$', literal, re.S)
    if raw:
        return raw.group(2)
    plain = re.match(r'^b?"(.*)"$', literal, re.S)
    return plain.group(1) if plain else literal


def escaping_literals(root, copied):
    """crates/<crate>/ 下每份 .rs（跳过 target）里以 "../ 起头的字符串字面量，按 crates/<crate>/ 解析之后逃出 crates/ 的：
    返回 ([(文件, 行号, 字面量, 解析到的路径, 在不在拷贝范围里)], 扫了几份 .rs)。#[path = …] 里的不算。"""
    found, scanned = [], 0
    crates_directory = os.path.join(root, "crates")
    if not os.path.isdir(crates_directory):
        return found, scanned
    for crate_name in sorted(os.listdir(crates_directory)):
        crate_root = os.path.join(crates_directory, crate_name)
        if not os.path.isdir(crate_root):
            continue
        for current_directory, directory_names, file_names in os.walk(crate_root):
            directory_names[:] = sorted(name for name in directory_names
                                        if not any(fnmatch.fnmatch(name, pattern) for pattern in IGNORED_WHEN_COPYING_INTO_EACH_SHARD))
            for file_name in sorted(file_names):
                if not file_name.endswith(".rs"):
                    continue
                path = os.path.join(current_directory, file_name)
                with open(path, encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                scanned += 1
                for start, end in string_literals(text):
                    literal = text[start:end]
                    value = literal_value(literal)
                    if not value.startswith("../"):
                        continue
                    if PATH_ATTRIBUTE_BEFORE_LITERAL.search(text[max(0, start - 200):start]) and PATH_ATTRIBUTE_AFTER_LITERAL.match(text, end):
                        continue
                    resolved = os.path.normpath(os.path.join("crates", crate_name, value))
                    if resolved == "crates" or resolved.startswith("crates" + os.sep):
                        continue
                    found.append((os.path.relpath(path, root), text.count("\n", 0, start) + 1, literal, resolved,
                                  is_copied_into_each_shard(resolved, copied)))
    return found, scanned


# ── 底座指纹、行键、按条记录与用时表 ──────────────────────────────────────────

def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


def base_fingerprint(root, environment, memory_cap_runner):
    """(底座指纹, 文件数)：见文件头「按条记录」。登记表没有 checker-tier-crates-mutation-replay 那一行抛 RegistrationError，git 列不出、读不了、工具链跑不出抛 InputManifestError。"""
    admission = admission_module()
    stage_rows = admission.rows_of_key(admission.read_registration_rows(STAGE_REPOSITORY), STAGE)
    if not stage_rows:
        raise admission.RegistrationError(f"{admission.REGISTRATION_TABLE} 里没有 {STAGE} 这一行")
    input_paths = [input_path for row in stage_rows for input_path in row.input_paths]
    listed_files = [name for name in admission.listed_input_files(root, input_paths) if name != TABLE.encode()]
    if break_is(environment, "basewithoutsource"):
        listed_files = [name for name in listed_files if not name.startswith(b"crates/")]
    if not listed_files:
        raise admission.InputManifestError(f"登记的路径（{' '.join(input_paths)}）下 git 一个文件都列不出来（减去 {TABLE} 之后）")
    registration_text = "".join(row.fingerprint_text() for row in stage_rows).encode("utf-8", "surrogateescape")
    named_lines = [(f"<登记行：{STAGE}>", registration_text),
                   (f"<阶段脚本：.claude/gate.d/{STAGE}>", read_bytes(STAGE_SCRIPT)),
                   ("<共用模块：research/scripts/crates-mutation-rows.py>", read_bytes(MODULE_PATH)),
                   ("<内存包装：这一趟每条经它跑的那一份>", read_bytes(memory_cap_runner)),
                   admission.toolchain_line(root)] + admission.build_environment_lines(root, environment)
    _manifest, fingerprint, file_count = admission.manifest_of_files(root, listed_files, named_lines)
    return fingerprint, file_count


def row_key(row, memory_max, timeout_seconds):
    text = "\t".join(row.fields) + f"\nmemory_max={memory_max}\ntimeout_seconds={timeout_seconds}\n"
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def row_content_hash(row):
    return hashlib.sha256("\t".join(row.fields).encode("utf-8", "surrogateescape")).hexdigest()


def encode_value(value):
    return str(value).replace("\\", "\\\\").replace("\n", "\\n")


def decode_value(value):
    return re.sub(r"\\(.)", lambda escape: "\n" if escape.group(1) == "n" else escape.group(1), value)


def record_text(fields):
    return RECORD_HEADER + "".join(f"{key}={encode_value(value)}\n" for key, value in fields.items())


def read_record(path):
    """按条记录或用时表的一格：{键: 值}；不在、读不了交 None。"""
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return None
    fields = {}
    for line in lines:
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        fields[key] = decode_value(value)
    return fields


def write_atomically(path, text):
    """同目录排他建临时文件、写完改名换上（照 admission.py 的 write_stage_marker）。"""
    handle_number, temporary_path = tempfile.mkstemp(dir=os.path.dirname(path), prefix=os.path.basename(path) + ".partial.")
    try:
        with os.fdopen(handle_number, "w", encoding="utf-8", errors="surrogateescape") as handle:
            handle.write(text)
        os.replace(temporary_path, path)
    except OSError:
        with contextlib.suppress(OSError):
            os.unlink(temporary_path)
        raise


def utc_now_text():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def reuse_refusal(environment):
    """一律不复用的理由（照写新记录）；复用时交 None。"""
    for variable in ("SINGLEFS_GATE_FULL", "GATE_MUTATION_START_OVER"):
        if environment.get(variable, "") == "1":
            return f"{variable}=1"
    return None


def record_is_fresh(record, environment):
    """没过复用上限：SINGLEFS_REUSE_HOURS（默认 24；写成不是整数的按 0 算，一律不复用），口径照 admission.py 的 check_stage_marker。"""
    try:
        finished_epoch = calendar.timegm(time.strptime(record.get("finished_utc", ""), "%Y-%m-%dT%H:%M:%SZ"))
    except ValueError:
        return False
    hours_text = environment.get("SINGLEFS_REUSE_HOURS", "") or "24"
    try:
        reuse_limit_hours = int(hours_text)
    except ValueError:
        reuse_limit_hours = 0
    return max(0, int(time.time()) - finished_epoch) // 3600 < reuse_limit_hours


def reusable_record(records_directory, kind, key, base, environment):
    """这一行这一类作数的记录：文件在、类别、底座指纹、行键都对得上、判定是能复用的那一档、没过期；不然交 None。"""
    record = read_record(os.path.join(records_directory, f"{kind}.{key}"))
    if (record is None or record.get("kind") != kind or record.get("base_fingerprint") != base or record.get("row_key") != key
            or record.get("verdict") != REUSABLE_VERDICT[kind] or not record_is_fresh(record, environment)):
        return None
    return record


def records_home(root, environment):
    """按条记录与用时表放在哪个目录底下：GATE_MUTATION_RECORDS_HOME 设了就是它，不设是被判那棵树的 git common-dir；取不到抛 InputManifestError。"""
    configured = environment.get("GATE_MUTATION_RECORDS_HOME", "").strip()
    if configured:
        return configured
    admission = admission_module()
    common_directory = admission.git_common_directory(root)
    if common_directory is None:
        raise admission.InputManifestError(f"{root} 不是 git 工作树（取不到 git common-dir）")
    return common_directory


def prune_base_directories(records_root, current):
    """只留最新的 BASE_DIRECTORIES_KEPT 个底座指纹目录（这一趟的先 touch 成最新），更早的删掉；只删 records_root 底下名字是 64 位十六进制的目录。
    返回删了的目录名。"""
    current_directory = os.path.join(records_root, current)
    os.makedirs(current_directory, exist_ok=True)
    os.utime(current_directory)
    others = [entry for entry in os.scandir(records_root)
              if entry.is_dir(follow_symlinks=False) and FINGERPRINT_FORM.fullmatch(entry.name) and entry.name != current]
    others.sort(key=lambda entry: (-entry.stat(follow_symlinks=False).st_mtime, entry.name))
    removed = []
    for entry in others[BASE_DIRECTORIES_KEPT - 1:]:
        shutil.rmtree(os.path.join(records_root, entry.name), ignore_errors=True)
        removed.append(entry.name)
    return removed


def read_timing(timings_directory, content_hash):
    """(上一次跑的秒数, 上一次超没超时)；没有记过交 None。"""
    record = read_record(os.path.join(timings_directory, content_hash))
    if record is None:
        return None
    try:
        return float(record.get("seconds_running", "")), record.get("timed_out") == "1"
    except ValueError:
        return None


# ── 基线的批、派活的次序、libtest 的结果行 ────────────────────────────────────

def split_at_double_dash(arguments):
    words = arguments.split()
    if "--" in words:
        index = words.index("--")
        return words[:index], words[index + 1:]
    return words, []


def batch_key(arguments):
    before, after = split_at_double_dash(arguments)
    return tuple(before), tuple(word for word in after if word.startswith("--"))


def baseline_batches(rows):
    """{批的键: [这一批的行，按表序]}，批按第一行在表里的次序排。"""
    batches = collections.OrderedDict()
    for row in rows:
        batches.setdefault(batch_key(row.arguments), []).append(row)
    return batches


def baseline_arguments(key, rows):
    """这一批基线的 cargo test 参数：键里的两段，加这一批各行过滤词的并集（有一行没有过滤词就一个都不带）。"""
    before, switches = key
    filters = []
    for row in rows:
        row_filters = [word for word in split_at_double_dash(row.arguments)[1] if not word.startswith("-")]
        if not row_filters:
            filters = []
            break
        filters += [word for word in row_filters if word not in filters]
    tail = list(switches) + filters
    return list(before) + (["--"] + tail if tail else [])


def crate_of(path):
    """`crates/<crate 名>/…` 里的 crate 名：同一个工作进程连着改同一个 crate，增量编译才稳。"""
    parts = path.split("/")
    return parts[1] if len(parts) > 2 and parts[0] == "crates" else path


def rows_grouped_by_crate(rows):
    """按 crate 归堆、堆内保持表序，再把几堆首尾相接（行多的堆在前）。"""
    groups = {}
    for row in rows:
        groups.setdefault(crate_of(row.path), []).append(row)
    ordered = []
    for crate_name in sorted(groups, key=lambda name: (-len(groups[name]), name)):
        ordered.extend(groups[crate_name])
    return ordered


def dispatch_order(rows, timings_directory, long_row_seconds, environment):
    """(派变异的次序, 前移了几条)：上一次用时不短于 long_row_seconds 或上一次超时的行先派、按用时从长到短，其余照按 crate 归堆的次序。"""
    if break_is(environment, "timingsignored"):
        return rows_grouped_by_crate(rows), 0
    long_rows = []
    for row in rows:
        timing = read_timing(timings_directory, row_content_hash(row))
        if timing is not None and (timing[1] or timing[0] >= long_row_seconds):
            long_rows.append((timing[0], row))
    long_rows.sort(key=lambda item: (-item[0], item[1].line_number))
    moved_lines = {row.line_number for _seconds, row in long_rows}
    return [row for _seconds, row in long_rows] + rows_grouped_by_crate([row for row in rows if row.line_number not in moved_lines]), len(long_rows)


def results_by_test(expected, output):
    """libtest 的结果行 `test <模块路径>::<名字> ... ok|FAILED|ignored` 里对上点名的那几条：{完整名: [结果…]}。
    #[should_panic] 的用例在名字与 ` ... ` 之间多一段 ` - should panic`；点名可以省掉模块路径。"""
    pattern = re.compile(r"^test ((?:\S+::)?" + re.escape(expected) + r")(?: - should panic)? \.\.\. (\S+)", re.M)
    results = {}
    for result_line in pattern.finditer(output):
        results.setdefault(result_line.group(1), []).append(result_line.group(2))
    return results


def output_tail(output, lines=8):
    return "\n".join(output.splitlines()[-lines:])


def failure_excerpt(output, full_name):
    """libtest 给这一条打的 `---- <名字> stdout ----` 那一段的头几行；没有就交 None。"""
    marker = f"---- {full_name} stdout ----"
    index = output.find(marker)
    if index < 0:
        return None
    return "\n".join(output[index:].splitlines()[:6])


def wrapper_outcome(returncode, memory_max, timeout_seconds):
    """包装自己的几种结局：(档, 那一句)；是命令自己的退出码交 None。"""
    outcomes = {
        TIME_LIMIT_HIT_EXIT: ("timeout", f"{timeout_seconds} 秒没跑完（超时）"),
        MEMORY_CAP_HIT_EXIT: ("memory", f"撞了内存上限 {memory_max}（内存撞顶）"),
        SLICE_TOTAL_HIT_EXIT: ("squeezed", "被 slice 的总上限挤掉（整个 slice 满了，内核挑了这一条杀）"),
        MEMORY_ADMISSION_REFUSED_EXIT: ("refused", "内存不够排不上（包装等满了等待上限），没跑"),
        MEMORY_CAP_UNAVAILABLE_EXIT: ("unavailable", "带内存上限的 scope 起不来，没跑"),
    }
    return outcomes.get(returncode)


def baseline_problem(row, returncode, output, memory_max, timeout_seconds):
    """这一行的点名测试在基线输出里不是「恰好一个用例、结果都是 ok」时交那一句（接在「在没改坏的副本上」后面），是的交 None。"""
    outcome = wrapper_outcome(returncode, memory_max, timeout_seconds)
    if outcome is not None:
        return f"没判成：那一批{outcome[1]}"
    results = results_by_test(row.expected, output)
    if len(results) == 1 and all(result == "ok" for result in next(iter(results.values()))):
        return None
    if len(results) > 1:
        return f"对上了 {len(results)} 个不同的用例（{'、'.join(sorted(results))}），判不出是哪一个；把点名写全到模块路径"
    if results:
        full_name, outcomes = next(iter(results.items()))
        return f"的结果是 {'、'.join(outcomes)}（{full_name}）"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        return f"一行都没出现：那一批编不过（退出码 {returncode}）"
    return f"一行都没出现（那一批退出码 {returncode}；参数筛不到它，或名字认不出）"


def verdict_of_run(row, returncode, output, memory_max, timeout_seconds):
    """一条变异跑完之后判哪一档：(档, 那一条要打印的话)。"""
    tail = output_tail(output)
    outcome = wrapper_outcome(returncode, memory_max, timeout_seconds)
    # 包装报的几种结局先判：被停、被杀的测试二进制可能已经打出了半截输出，不许拿它判抓到或没红
    if outcome is not None:
        verdict, sentence = outcome
        extra = {"timeout": f"，点名的测试 {row.expected} 没来得及红", "memory": f"，点名的测试 {row.expected} 没来得及红",
                 "squeezed": "，这一条的结果不算数"}.get(verdict, "")
        return verdict, f"{TABLE}:{row.line_number} {row.name}：{sentence}{extra}\n{tail}"
    # 点名省掉模块路径时在输出里只许对上一个用例：对上两个就分不出红的是不是点名的那一个，按没判成记，不按「有一个红了」算抓到。
    results = results_by_test(row.expected, output)
    if len(results) > 1:
        return "failure", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 在输出里对上了 {len(results)} 个不同的用例"
                           f"（{'、'.join(sorted(results))}），判不出红的是哪一个；把点名写全到模块路径\n{tail}")
    if returncode != 0 and any("FAILED" in outcomes for outcomes in results.values()):
        return "caught", caught_text(row)
    if results:
        return "failure", f"{TABLE}:{row.line_number} {row.name}：{row.expected} 没红（退出码 {returncode}）\n{tail}"
    if re.search(r"^error(\[E\d+\])?: ", output, re.M) or "could not compile" in output:
        # 替换文写进源码之后编不过：既不算被抓也不算没红，是一条无效变异（.claude/rules/mutation-sampling.md 第八类）。
        return "invalid", (f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 一行都没出现，输出里有 error 行"
                           f"（替换文编不过，或测试进程在跑到它之前被杀，或名字认不出；退出码 {returncode}）\n{tail}")
    return "failure", f"{TABLE}:{row.line_number} {row.name}：点名的测试 {row.expected} 没跑到（退出码 {returncode}）\n{tail}"


def caught_text(row):
    return f"  ✓ {row.name}：{row.expected} 红了"


# ── 工作进程 ──────────────────────────────────────────────────────────────────

WORKER_STATE = {}


def init_worker(sequence_counter, shard_root, context):
    """工作进程启动时跑一次：领一个稳定的片号、建自己的源码副本与编译目录。片号从共享计数器领，活是动态领的（checker-tier-crates-mutation-replay 文件头）。"""
    with sequence_counter.get_lock():
        sequence_counter.value += 1
        worker_sequence = sequence_counter.value
    work, target_directory = prepare_shard(worker_sequence, shard_root, context)
    environment = dict(context["environment"])
    environment["CARGO_TARGET_DIR"] = target_directory
    WORKER_STATE.update(work=work, environment=environment, context=context, shard_root=shard_root,
                        per_mutation_temporary_root=os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME))
    # 源码副本建在父进程的 shard_root 底下、由父进程收尾时整个删；编译目录留着跨轮复用。
    # 不在这里挂 atexit：进程池的工作进程走 os._exit 退出，atexit 在这里一次都不跑。


def prepare_shard(worker_sequence, shard_root, context):
    """一个工作进程自己的一份源码副本加一个编译目录。第 1 片用共用的编译目录（跨轮热着），其余片 -w<片号> 自己冷编译一次，不从共用目录拷种子。
    ⚠️ 拷的时候不留原来的修改时刻（shutil.copy，不用 copy2）：编译目录跨轮复用，里面最后一次编的可能是上一轮改坏的源码、
    带着上一轮副本的 CARGO_MANIFEST_DIR；副本留着原来较早的修改时刻，cargo 就判它不用重编，没改坏的基线跑的是上一轮改坏的二进制
    （2026-09-28 样本里实测：基线报 attempt to subtract with overflow、读 litmus 读到上一轮已经删掉的副本路径）。
    修改时刻取拷的那一刻，每片头一次编译把工作区的几个 crate 重编一遍（外部依赖不在副本里，照旧热着）。"""
    work = tempfile.mkdtemp(prefix=f"shard-{worker_sequence}-", dir=shard_root)
    for item in context["copied"]:
        source = os.path.join(context["root"], item)
        if not os.path.exists(source):
            continue
        target = os.path.join(work, item)
        if os.path.isdir(source):
            shutil.copytree(source, target, ignore=shutil.ignore_patterns(*IGNORED_WHEN_COPYING_INTO_EACH_SHARD), copy_function=shutil.copy)
        else:
            os.makedirs(os.path.dirname(target), exist_ok=True)
            shutil.copy(source, target)
    shared = context["shared_target_directory"]
    return work, (shared if worker_sequence == 1 else f"{shared}-w{worker_sequence}")


def measure_and_remove_temporary_directory(directory, context):
    """数下一条的 TMPDIR 里有几个文件、占盘与表观各多少字节，再整个删掉。回 (文件数, 占盘字节, 表观字节, 删完之后它还在不在)。"""
    file_count = allocated_bytes = apparent_bytes = 0
    for current_directory, _subdirectory_names, file_names in os.walk(directory):
        for file_name in file_names:
            try:
                file_status = os.lstat(os.path.join(current_directory, file_name))
            except OSError:
                continue   # 数的时候已经没了（还没退干净的进程在收尾），不算
            file_count += 1
            allocated_bytes += file_status.st_blocks * BYTES_PER_STAT_BLOCK
            apparent_bytes += file_status.st_size
    if not break_is(context["environment"], "keepmutationtmp"):
        shutil.rmtree(directory, ignore_errors=True)
    return file_count, allocated_bytes, apparent_bytes, os.path.lexists(directory)


def run_under_wrapper(arguments, temporary_directory, stamp_name):
    """经内存包装跑一次 cargo test --offline <参数>：(subprocess 的结果, 用时)。限时交给包装（从起跑算、不算排队），TMPDIR 是这一次自己的。"""
    state, context = WORKER_STATE, WORKER_STATE["context"]
    stamp_path = os.path.join(state["shard_root"], START_STAMP_DIRECTORY_NAME, stamp_name)
    command = ["cargo", "test", "--offline"] + arguments
    environment = dict(state["environment"], TMPDIR=temporary_directory,
                       RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(context["timeout_seconds"]),
                       RUN_WITH_MEMORY_CAP_KILL_GRACE=str(TIMEOUT_KILL_GRACE_SECONDS),
                       RUN_WITH_MEMORY_CAP_KEY=" ".join(command))
    called = time.time()
    run = subprocess.run(["bash", context["memory_cap_runner"], context["memory_max"], "bash", "-c", START_STAMP_SCRIPT, "started-at", stamp_path]
                         + command, cwd=state["work"], env=environment, capture_output=True, text=True, errors="replace")
    returned = time.time()
    started = None
    with contextlib.suppress(OSError, ValueError):
        with open(stamp_path, encoding="utf-8") as handle:
            started = float(handle.read().strip().replace(",", "."))
    with contextlib.suppress(OSError):
        os.unlink(stamp_path)
    queued = (started - called) if started is not None else (returned - called)
    running = (returned - started) if started is not None else 0.0
    return run, {"seconds_total": f"{returned - called:.3f}", "seconds_queued": f"{max(0.0, queued):.3f}",
                 "seconds_running": f"{max(0.0, running):.3f}"}


def write_record_quietly(kind, key, fields):
    """写一条按条记录；写不进去交那一句（交回父进程报在 stderr，下一趟这一条照跑），写成交 None。"""
    path = os.path.join(WORKER_STATE["context"]["records_directory"], f"{kind}.{key}")
    try:
        write_atomically(path, record_text(fields))
    except OSError as error:
        return f"{path}：{error}"
    return None


def judge_batch(batch):
    """进程池里的一格活：一批基线（没改坏的副本上跑一遍这一批各行点名的测试）。回 (每行的 (行号, 问题或 None), 那一批的尾巴, 写不进去的记录)。"""
    key, batch_rows = batch
    context = WORKER_STATE["context"]
    arguments = baseline_arguments(key, batch_rows)
    temporary_directory = tempfile.mkdtemp(prefix=f"baseline-{batch_rows[0].line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    try:
        run, seconds = run_under_wrapper(arguments, temporary_directory, f"baseline-{batch_rows[0].line_number}")
    finally:
        measure_and_remove_temporary_directory(temporary_directory, context)
    output = run.stdout + run.stderr
    judged, record_problems = [], []
    for row in batch_rows:
        problem = baseline_problem(row, run.returncode, output, context["memory_max"], context["timeout_seconds"])
        excerpt = None
        if problem is not None:
            results = results_by_test(row.expected, output)
            excerpt = failure_excerpt(output, next(iter(results))) if len(results) == 1 else None
        judged.append((row.line_number, problem, excerpt or output_tail(output)))
        failed_to_write = write_record_quietly("baseline", context["row_keys"][row.line_number], dict(
            kind="baseline", base_fingerprint=context["base"], row_key=context["row_keys"][row.line_number],
            row_content_hash=row_content_hash(row), table_line=row.line_number, name=row.name, file=row.path,
            memory_max=context["memory_max"], timeout_seconds=context["timeout_seconds"],
            verdict="ok" if problem is None else "not-ok", problem=problem or "", batch_arguments=" ".join(arguments),
            batch_exit_code=run.returncode, finished_utc=utc_now_text(), **seconds))
        if failed_to_write:
            record_problems.append(failed_to_write)
    return judged, record_problems


def judge_row(row):
    """进程池里的一格活：一条变异。建它自己的 TMPDIR、改坏那一处、跑点名的测试、还原、删掉那个 TMPDIR，判完立刻写按条记录与用时表。
    回 ((行号, 档, 那一条要打印的话, 删 TMPDIR 的结果), 写不进去的记录)。"""
    context = WORKER_STATE["context"]
    full = os.path.join(WORKER_STATE["work"], row.path)
    with open(full, encoding="utf-8") as handle:
        pristine = handle.read()
    assert pristine.count(row.old) == 1, (row.path, row.name)
    temporary_directory = tempfile.mkdtemp(prefix=f"line-{row.line_number}-", dir=WORKER_STATE["per_mutation_temporary_root"])
    with open(full, "w", encoding="utf-8") as handle:
        handle.write(pristine.replace(row.old, row.new))
    try:
        run, seconds = run_under_wrapper(row.arguments.split(), temporary_directory, f"line-{row.line_number}")
    finally:
        with open(full, "w", encoding="utf-8") as handle:
            handle.write(pristine)
        # 不论退出码是几都删（含限时、撞顶被杀）：被杀的测试 Drop 守卫不跑，它建的镜像只有这里删得掉
        removal = measure_and_remove_temporary_directory(temporary_directory, context)
    verdict, text = verdict_of_run(row, run.returncode, run.stdout + run.stderr, context["memory_max"], context["timeout_seconds"])
    if verdict in VERDICTS_KILLED_BY_WRAPPER:
        file_count, allocated_bytes, _apparent_bytes, is_still_there = removal
        first_line, line_break, rest = text.partition("\n")
        text = (first_line + f"；它的临时目录{'没删掉' if is_still_there else '已整个删掉'}（删前留有 {file_count} 个文件、占盘 {allocated_bytes} 字节）"
                + line_break + rest)
    key = context["row_keys"][row.line_number]
    record_problems = []
    failed_to_write = write_record_quietly("mutation", key, dict(
        kind="mutation", base_fingerprint=context["base"], row_key=key, row_content_hash=row_content_hash(row),
        table_line=row.line_number, name=row.name, file=row.path, memory_max=context["memory_max"], timeout_seconds=context["timeout_seconds"],
        verdict=verdict, text=text, removal_files=removal[0], removal_allocated_bytes=removal[1], removal_apparent_bytes=removal[2],
        removal_left=int(removal[3]), exit_code=run.returncode, finished_utc=utc_now_text(), **seconds))
    if failed_to_write:
        record_problems.append(failed_to_write)
    if verdict not in VERDICTS_THAT_DID_NOT_RUN and verdict != "squeezed":
        timing_path = os.path.join(context["timings_directory"], row_content_hash(row))
        try:
            write_atomically(timing_path, record_text(dict(row_content_hash=row_content_hash(row), seconds_running=seconds["seconds_running"],
                                                           seconds_queued=seconds["seconds_queued"], timed_out=int(verdict == "timeout"),
                                                           finished_utc=utc_now_text())))
        except OSError as error:
            record_problems.append(f"{timing_path}：{error}")
    return (row.line_number, verdict, text, removal), record_problems


# ── 整道 ──────────────────────────────────────────────────────────────────────

def worker_count_for(row_count, environment):
    """开几个工作进程，连同头一行要报的话。两项取最小、至少 1：核数的一半（至多 16，GATE_MUTATION_WORKERS 顶掉它）、这一趟要现跑的条数。
    一条变异的挂钟 = 重编译那几个 crate + 跑点名的那个测试，测试那一段是单线程的，所以开得比核数 ÷ 4 多，再用 GATE_MUTATION_CARGO_JOBS
    把每个 cargo 的编译并行度收住。内存不在这里收：每条经 run-with-memory-cap.sh 排队，放不下的在它那里等。"""
    bounds = []
    requested = environment.get("GATE_MUTATION_WORKERS", "").strip()
    if requested:
        bounds.append(("GATE_MUTATION_WORKERS", int(requested), f"GATE_MUTATION_WORKERS 给 {int(requested)} 个"))
    else:
        processor_count = os.cpu_count() or 4
        by_processor = max(1, min(16, processor_count // 2))
        bounds.append(("核数", by_processor, f"核数 {processor_count} 的一半、至多 16，给 {by_processor} 个"))
    bounds.append(("这一趟要现跑的条数", row_count, f"这一趟要现跑 {row_count} 条（没有作数的抓到记录的）"))
    smallest = min(count for _name, count, _text in bounds)
    binding = "、".join(f"「{name}」" for name, count, _text in bounds if count == smallest)
    floor_note = "（那一项不足 1，按 1 个开）" if smallest < 1 else ""
    worker_count = max(1, smallest)
    head_line = (f"  … 这一轮开 {worker_count} 个工作进程（各项取最小、至少 1）：{'；'.join(text for _name, _count, text in bounds)}。"
                 f"卡在{binding}这一项{floor_note}；内存不按进程数收：每条经 run-with-memory-cap.sh 排队，放不下的等（slice 的总上限兜底）")
    return worker_count, head_line


def run_pool(context, worker_count, batches, ordered_rows, baseline_ok_lines, stop_after):
    """一个进程池里先派基线、再派变异：一批基线判完，它 ok 的行进变异的队；已有基线 ok 的行一开始就在队里。
    同时在跑的至多 worker_count 格，空出一格补一格（谁先跑完谁再领），基线优先、变异按 ordered_rows 的次序。
    回 dict：mutation（变异的判定）、baseline（{行号: (问题或 None, 尾巴)}）、submitted、received、stopped、left_behind、record_problems。"""
    sequence_counter = multiprocessing.Value("i", 0)
    shard_root = tempfile.mkdtemp(prefix="singlefs-mutation-replay-")
    try:
        per_mutation_temporary_root = os.path.join(shard_root, PER_MUTATION_TEMPORARY_DIRECTORY_NAME)
        os.mkdir(per_mutation_temporary_root)
        os.mkdir(os.path.join(shard_root, START_STAMP_DIRECTORY_NAME))
        order_index = {row.line_number: index for index, row in enumerate(ordered_rows)}
        rows_by_line = {row.line_number: row for row in ordered_rows}
        ready, serial = [], 0
        for index, batch in enumerate(batches):
            heapq.heappush(ready, (0, index, serial, "baseline", batch))
            serial += 1
        for row in ordered_rows:
            if row.line_number in baseline_ok_lines:
                heapq.heappush(ready, (1, order_index[row.line_number], serial, "mutation", row))
                serial += 1
        outcome = dict(mutation=[], baseline={}, submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
        in_flight = {}
        total = len(batches) + len(ordered_rows)
        with concurrent.futures.ProcessPoolExecutor(max_workers=worker_count, initializer=init_worker,
                                                    initargs=(sequence_counter, shard_root, context)) as pool:
            while True:
                while ready and len(in_flight) < worker_count and not outcome["stopped"]:
                    _phase, _index, _serial, kind, item = heapq.heappop(ready)
                    in_flight[pool.submit(judge_batch if kind == "baseline" else judge_row, item)] = kind
                    outcome["submitted"] += 1
                if not in_flight:
                    break
                done, _pending = concurrent.futures.wait(list(in_flight), return_when=concurrent.futures.FIRST_COMPLETED)
                for future in done:
                    kind = in_flight.pop(future)
                    result, record_problems = future.result()   # 工作进程里抛出来的异常在这里原样炸出去：并行不许把失败吃掉
                    outcome["received"] += 1
                    outcome["record_problems"] += record_problems
                    if kind == "baseline":
                        for line_number, problem, tail in result:
                            outcome["baseline"][line_number] = (problem, tail)
                            if problem is None and line_number in order_index:
                                heapq.heappush(ready, (1, order_index[line_number], serial, "mutation", rows_by_line[line_number]))
                                serial += 1
                    else:
                        outcome["mutation"].append(result)
                        if stop_after is not None and len(outcome["mutation"]) >= stop_after:
                            outcome["stopped"] = True
                    note(f"  … 已判 {outcome['received']} 格、至多 {total} 格（基线的批与变异合计；基线不绿的行不派变异）")
        outcome["left_behind"] = sorted(os.listdir(per_mutation_temporary_root))
        outcome["left_behind_contents"] = {name: (sorted(os.listdir(os.path.join(per_mutation_temporary_root, name)))
                                                  if os.path.isdir(os.path.join(per_mutation_temporary_root, name)) else ["（不是目录）"])
                                           for name in outcome["left_behind"]}
        outcome["per_mutation_temporary_root"] = per_mutation_temporary_root
        return outcome
    finally:
        shutil.rmtree(shard_root, ignore_errors=True)


def read_selection(path, rows):
    """GATE_MUTATION_ROW_SELECTION 那份文件：一行一个变异表行号（# 起头的行与空行不看）。回 (选中的行号集合, [问题])。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        return set(), [f"读不了 {path}：{error}"]
    selected, problems = set(), []
    row_lines = {row.line_number for row in rows}
    for index, line in enumerate(lines, 1):
        text = line.strip()
        if not text or text.startswith("#"):
            continue
        if not text.isdigit():
            problems.append(f"{path} 第 {index} 行「{text}」不是一个行号")
        elif int(text) not in row_lines:
            problems.append(f"{path} 第 {index} 行：{TABLE} 第 {text} 行不是一条成形的变异")
        else:
            selected.add(int(text))
    if not selected and not problems:
        problems.append(f"{path} 里一个行号都没有")
    return selected, problems


def print_rejection(headline, details, howto_lines):
    say(f"  ✗ {headline}")
    for detail in details:
        say(f"      {detail}")
    for index, howto_line in enumerate(howto_lines):
        say(("     → 怎么办：" if index == 0 else "       ") + howto_line)


def prechecks(root, rows, malformed, environment):
    """派活之前先判、一个工作进程都不起的那几样（row_problems，与门禁 code-source-discipline 经 static-check 判的是同一份）。回 (拷贝范围, 判红了没有)。"""
    admission = admission_module()
    try:
        copied = copied_paths()
    except admission.RegistrationError as error:
        copied, reason = [], str(error)
    else:
        reason = "那一行一条路径都没登记"
    if not copied:
        print_rejection(f"读不到 .claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行登记的路径（{reason}）：判不出每片的源码副本该拷哪些", [],
                        [f"单独执行 python3 research/scripts/admission.py paths \"{STAGE_REPOSITORY}\" {STAGE} 看它报什么；那一行不在就照别的行给本阶段登记它读的路径。"])
        return None, True
    # 锚点只在真仓上核：文件落在拷贝范围之外时，工作进程在自己那份副本里打不开它，所以拷贝范围也在派活之前判。有一样红就一个工作进程都不起。
    problems = row_problems(root, rows, malformed, copied)
    not_started = "这一轮一个工作进程都没起；门禁 code-source-discipline 静态判同样几样（不跑变异），改完先跑它。"
    if problems["MALFORMED"]:
        print_rejection(f"{TABLE} 有几行不成形（六段制表符分隔：{'、'.join(FIELD_NAMES)}；替换文可以空，那是删掉原文，其余五段不许空）：",
                        [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["MALFORMED"]],
                        ["补上空着的那一段，制表符删补到恰好六段；原文 / 替换文里的换行写成 \\n。要删掉原文就让替换文空着（两个制表符挨着）。", not_started])
    if problems["ANCHOR"]:
        print_rejection("变异表的锚点腐化：", [f"{TABLE}:{line_number} {name}：{why}" for line_number, name, why in problems["ANCHOR"]],
                        ["改代码时把变异表里的原文一起改到今天的写法（锚点腐化的那条变异等于没跑过，mutation-sampling.md 第七类）。", not_started])
    if problems["NAME"]:
        print_rejection("变异表有几行的「必须红的测试名」不是一个用例名（带了正则符号或空白），按字面在测试输出里永远找不到：",
                        [f"{TABLE}:{line_number} {name}：「{expected}」" for line_number, name, expected in problems["NAME"]],
                        ["那一列写成用例名本身（例 new_pool_file_creation_self_release_is_one_slot，或带模块路径 tests::new_pool_file_creation_self_release_is_one_slot），",
                         "去掉 $、^ 这类符号；只写名字时本阶段已经按「在输出里只许对上一个用例」判，不用再靠 $ 锚定。", not_started])
    if problems["ROW"]:
        print_rejection("变异表有几行的文件不在每片拷的范围里（锚点在真仓上核得过，工作进程在自己的源码副本里却打不开它）：",
                        [f"{TABLE}:{line_number} {name}：{path}" for line_number, name, path in problems["ROW"]],
                        [f"把那个路径（或它所在的目录）登记进 .claude/gate.d/stage-inputs.tsv 本阶段那一行（现在是 {'、'.join(copied)}），"
                         "或者把这条变异改到拷贝范围之内的文件上。", not_started])
    if problems["STRAY"]:
        print_rejection("变异表有几行的原文或替换文里有 \\n 以外的反斜杠转义（这里只把 \\n 还原成换行，别的反斜杠按字面写进源码，那条变异编不过、等于没跑）：",
                        [f"{TABLE}:{line_number} {name}：{segment}里的 {escape}" for line_number, name, segment, escape in problems["STRAY"]],
                        ["去掉那个反斜杠（例 \\&report.outcome 写成 &report.outcome），改完单跑那一行证明点名的测试红。", not_started])
    if any(problems.values()):
        return None, True
    return copied, False


def run_stage(root, environment=None):
    """门禁 checker-tier-crates-mutation-replay 的整道：判定打 stdout（按表的行号排序，与进程数、复用了多少都无关），进度与复用几条打 stderr。回退出码。"""
    environment = dict(os.environ if environment is None else environment)
    admission = admission_module()
    table_path = os.path.join(root, TABLE)
    if not os.path.isfile(table_path):
        print_rejection(f"没有 {TABLE}", [], ["建一张六段制表符分隔的表（变异名、文件、原文、替换文、cargo test 参数、必须红的测试名），每一处三方打中之后的改法留一条。"])
        return 1
    try:
        rows, malformed, memory_max_from_table, timeout_from_table = read_table(table_path, environment)
    except (OSError, UnicodeDecodeError) as error:
        print_rejection(f"读不了 {TABLE}（{type(error).__name__}：{error}）", [], ["转成 UTF-8 文本再跑；读不了的表一条都判不了。"])
        return 1
    # 不成形的行（read_table 判的）在 prechecks 里与别的几样一起报：整张表的不成形行不因选行而放过
    if not rows and not malformed:
        print_rejection(f"{TABLE} 里一条成形的变异都没有", [], ["至少一条：三方打中之后的每一处改法，留一条「改回去它就红」的变异。"])
        return 1
    memory_max, timeout_seconds, limit_problems = limits_of(memory_max_from_table, timeout_from_table, environment)
    for headline, howto_line in limit_problems:
        print_rejection(headline, [], [howto_line])
    if limit_problems:
        return 1
    numeric_settings = {}
    for variable, default in (("GATE_MUTATION_LONG_ROW_SECONDS", DEFAULT_LONG_ROW_SECONDS), ("GATE_MUTATION_STOP_AFTER_ROWS", "")):
        text = environment.get(variable, "").strip() or default
        if text and not re.fullmatch(r"[1-9][0-9]*", text):
            print_rejection(f"{variable} 写成了「{text}」，不是正整数", [], [f"写成正整数（{variable} 的意思见 research/scripts/crates-mutation-rows.py 文件头），或者不设它。"])
            return 1
        numeric_settings[variable] = int(text) if text else None
    selection_path = environment.get("GATE_MUTATION_ROW_SELECTION", "").strip()
    if selection_path and rows:
        selected, selection_problems = read_selection(selection_path, rows)
        if selection_problems:
            print_rejection(f"GATE_MUTATION_ROW_SELECTION 指的那份选行文件不对，一行都没判：", selection_problems,
                            [f"一行写一个 {TABLE} 里成形的变异所在的行号（# 起头的行与空行不看）；行号照被判的那一份表现数。"])
            return 1
        rows = [row for row in rows if row.line_number in selected]
    copied, refused = prechecks(root, rows, malformed, environment)
    if refused:
        return 1
    memory_cap_runner = environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip() or DEFAULT_MEMORY_CAP_RUNNER
    try:
        base, _base_file_count = base_fingerprint(root, environment, memory_cap_runner)
        common_directory = records_home(root, environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print_rejection(f"算不出底座指纹（{error}），按条记录没处对：一条都没跑", [],
                        ["被判的要是 git 工作树（git ls-files 列得出 crates/）、cargo -V 与 rustc -V 跑得通；",
                         f"单跑 python3 research/scripts/crates-mutation-rows.py base-fingerprint {root} 看它报什么。"])
        return 1
    records_root = os.path.join(common_directory, RECORDS_DIRECTORY_NAME)
    records_directory = os.path.join(records_root, base)
    timings_directory = os.path.join(common_directory, TIMINGS_DIRECTORY_NAME)
    os.makedirs(timings_directory, exist_ok=True)
    pruned = prune_base_directories(records_root, base)
    refusal = reuse_refusal(environment)
    skip_baseline = break_is(environment, "nobaseline")
    row_keys = {row.line_number: row_key(row, memory_max, timeout_seconds) for row in rows}
    reused, baseline_ok_lines, reused_baseline_count = {}, set(), 0
    for row in rows:
        if refusal:
            break
        record = reusable_record(records_directory, "mutation", row_keys[row.line_number], base, environment)
        if record is not None:
            try:
                removal = (int(record.get("removal_files", "")), int(record.get("removal_allocated_bytes", "")),
                           int(record.get("removal_apparent_bytes", "")), record.get("removal_left") == "1")
            except ValueError:
                record = None
        if record is not None:
            reused[row.line_number] = (row.line_number, "caught", caught_text(row), removal)
            baseline_ok_lines.add(row.line_number)
        elif reusable_record(records_directory, "baseline", row_keys[row.line_number], base, environment) is not None:
            baseline_ok_lines.add(row.line_number)
            reused_baseline_count += 1
    candidates = [row for row in rows if row.line_number not in reused]
    if skip_baseline:
        baseline_ok_lines |= {row.line_number for row in candidates}
    needs_baseline = [row for row in candidates if row.line_number not in baseline_ok_lines]
    batches = list(baseline_batches(needs_baseline).items())
    ordered_candidates, moved_count = dispatch_order(candidates, timings_directory, numeric_settings["GATE_MUTATION_LONG_ROW_SECONDS"], environment)
    # stderr 头一行：复用几条、现跑几条、前移几条（stdout 不写这些：判定输出与进程数、复用了多少都无关）
    note(f"  … 按条记录：复用抓到 {len(reused)} 条、复用基线 ok {reused_baseline_count} 行；现跑基线 {len(batches)} 批 {len(needs_baseline)} 行、"
         f"现跑变异至多 {len(candidates)} 条；前移 {moved_count} 条（上一次用时 ≥ {numeric_settings['GATE_MUTATION_LONG_ROW_SECONDS']} 秒或上一次超时的先派，"
         f"按用时从长到短）；底座指纹 {base[:16]}…，记录在 {records_directory}"
         + (f"；不复用：{refusal}" if refusal else "") + (f"；删了更早的底座指纹目录 {len(pruned)} 个" if pruned else "")
         + ("；基线没跑：GATE_MUTATION_BREAK=nobaseline（只给证红用）" if skip_baseline else ""))
    outcome = dict(mutation=[], baseline={}, submitted=0, received=0, stopped=False, left_behind=[], record_problems=[])
    worker_count, cargo_jobs = 0, "-"
    if candidates:
        memory_cap_check = subprocess.run(["bash", memory_cap_runner, "--check", memory_max], capture_output=True, text=True, errors="replace",
                                          env=environment)
        if memory_cap_check.returncode != 0:
            print_rejection(f"带内存上限（每条 {memory_max}）的 systemd scope 起不来（{memory_cap_runner} --check 退 {memory_cap_check.returncode}），一条变异都没跑：",
                            (memory_cap_check.stdout + memory_cap_check.stderr).splitlines(),
                            ["照上面那几行修用户级 systemd / D-Bus（systemctl --user status）再跑这一道；不许拿掉上限去跑，无界分配的变异会把整机拖进 OOM。"])
            return 1
        worker_count, worker_count_head_line = worker_count_for(len(candidates), environment)
        note(worker_count_head_line)
        # 每个 cargo 的编译并行度：进程数 × 它 ≈ 核数，免得 N 个 cargo 各自按核数开 rustc、互相抢
        cargo_jobs = environment.get("GATE_MUTATION_CARGO_JOBS", "").strip() or str(max(1, (os.cpu_count() or 4) // worker_count))
        environment["CARGO_BUILD_JOBS"] = cargo_jobs
        default_target = os.path.join(environment.get("GATE_CROSS_RUN_TMPDIR") or environment.get("TMPDIR") or "/tmp", "singlefs-crates-mutation-target")
        context = dict(root=root, copied=copied, environment=environment, memory_max=memory_max, timeout_seconds=timeout_seconds,
                       memory_cap_runner=memory_cap_runner, base=base, records_directory=records_directory, timings_directory=timings_directory,
                       row_keys=row_keys, shared_target_directory=environment.get("GATE_MUTATION_TARGET_DIR", "").strip() or default_target)
        try:
            outcome = run_pool(context, worker_count, batches, ordered_candidates, baseline_ok_lines, numeric_settings["GATE_MUTATION_STOP_AFTER_ROWS"])
        except concurrent.futures.process.BrokenProcessPool:
            # 一个工作进程被硬杀（OOM、段错误）时当场抛这个，而不是安静地少产出几项
            print_rejection("有工作进程中途死了，这一轮的变异没跑全", [],
                            ["多半是内存不够被 OOM 杀的——GATE_MUTATION_WORKERS 调小再跑一遍（判完的那几条有按条记录，下一趟只跑剩下的）；",
                             "单跑 GATE_MUTATION_WORKERS=1 能跑完就是并发度的问题，还死就去看那一条变异本身。"])
            return 1
    for problem in outcome["record_problems"]:
        note(f"  ! 按条记录或用时表没写成（下一趟这一条照跑，不会复用）：{problem}")
    # 派出去多少格就要收回来多少格：对不上整道红，不许少跑一条还报绿（command-safety.md）
    if outcome["submitted"] != outcome["received"]:
        print_rejection(f"派出去 {outcome['submitted']} 格（基线的批与变异），只收回 {outcome['received']} 格", [],
                        ["这是分片并发自己的完整性闸红了，不是变异的问题；把 GATE_MUTATION_WORKERS=1 再跑一遍看串行下是不是全的，",
                         "是就去查领活与收束那一段（research/scripts/crates-mutation-rows.py 的 run_pool）。"])
        return 1
    if outcome["stopped"]:
        print_rejection(f"只供测试的开关 GATE_MUTATION_STOP_AFTER_ROWS={numeric_settings['GATE_MUTATION_STOP_AFTER_ROWS']}："
                        f"现跑的变异判完 {len(outcome['mutation'])} 条就没再派活，这一轮没跑全", [],
                        ["这个开关只给自证造「跑到一半被杀」用，门禁里别设它；判完的那几条已经写了按条记录，去掉它再跑一趟只跑剩下的。"])
        return 1
    judgements = list(reused.values()) + list(outcome["mutation"])
    baseline_problems = {}
    for row in candidates:
        problem, tail = outcome["baseline"].get(row.line_number, (None, ""))
        if problem is not None:
            baseline_problems[row.line_number] = problem
            judgements.append((row.line_number, "baseline",
                               f"{TABLE}:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem}\n{tail}", None))
    judged_lines = {judgement[0] for judgement in judgements}
    missing = [row for row in rows if row.line_number not in judged_lines]
    if missing:
        print_rejection(f"有 {len(missing)} 行一个判定都没有（没有作数的记录、基线也没判出它来）", [f"{TABLE}:{row.line_number} {row.name}" for row in missing],
                        ["这是按条记录与派活自己的完整性闸红了，不是变异的问题；GATE_MUTATION_START_OVER=1 整道重跑一遍，还缺就去查 run_stage 与 run_pool 那一段。"])
        return 1
    return report(rows, judgements, outcome, memory_max, timeout_seconds, selection_path, skip_baseline, dict(
        root=root, worker_count=worker_count, cargo_jobs=cargo_jobs, reused=len(reused), reused_baseline=reused_baseline_count,
        fresh_mutations=len(outcome["mutation"]), fresh_batches=len(batches), records_directory=records_directory))


def report(rows, judgements, outcome, memory_max, timeout_seconds, selection_path, skip_baseline, counts):
    """按变异表的行号排序打印判定、计数、基线与临时目录，判红的逐栏列出；回退出码。"""
    judgements.sort(key=lambda judgement: judgement[0])
    by_verdict = collections.defaultdict(list)
    for _line_number, verdict, text, _removal in judgements:
        by_verdict[verdict].append(text)
    caught, failures, invalid = by_verdict["caught"], by_verdict["failure"], by_verdict["invalid"]
    memory_hits, timeouts, squeezed = by_verdict["memory"], by_verdict["timeout"], by_verdict["squeezed"]
    refused, unavailable, baseline_bad = by_verdict["refused"], by_verdict["unavailable"], by_verdict["baseline"]
    for text in caught:
        say(text)
    # 各栏各数各的；「计数：」这一行的格式别的定义与规则照栏读（mutation-triage），基线另起一行报
    say(f"  计数：抓到 {len(caught)} 条、没红 {len(failures)} 条、无效 {len(invalid)} 条、内存撞顶 {len(memory_hits)} 条、超时 {len(timeouts)} 条、"
        f"被总上限挤掉 {len(squeezed)} 条、排不上没跑 {len(refused)} 条、scope 起不来没跑 {len(unavailable)} 条（共 {len(rows)} 条）")
    if skip_baseline:
        say("  基线：没跑（GATE_MUTATION_BREAK=nobaseline，只给证红用）")
    else:
        say(f"  基线：没改坏的副本上点名的测试恰好对上一个用例、结果是 ok 的 {len(rows) - len(baseline_bad)} 行，基线不绿 {len(baseline_bad)} 行"
            f"（按 cargo 参数分 {len(baseline_batches(rows))} 批；共 {len(rows)} 行）")
    removals = [removal for _line, _verdict, _text, removal in judgements if removal is not None]
    killed_removals = [removal for _line, verdict, _text, removal in judgements if removal is not None and verdict in VERDICTS_KILLED_BY_WRAPPER]
    left_behind = outcome["left_behind"]
    removal_state = "每条结束时不论退出码都整个删掉了" if not left_behind else f"跑完有 {len(left_behind)} 个没删掉"
    say(f"  临时目录：{len(removals)} 条变异各一个自己的 TMPDIR，{removal_state}；删前合计 {sum(removal[0] for removal in removals)} 个文件、"
        f"占盘 {sum(removal[1] for removal in removals)} 字节（表观 {sum(removal[2] for removal in removals)} 字节），"
        f"其中被包装停掉或杀掉的 {len(killed_removals)} 条（超时、内存撞顶、被总上限挤掉）留下 {sum(removal[0] for removal in killed_removals)} 个文件、"
        f"占盘 {sum(removal[1] for removal in killed_removals)} 字节")
    sections = [
        (baseline_bad, f"有几行基线不绿（基线不绿 {len(baseline_bad)} 行；点名的测试在没改坏的副本上就不是恰好一个 ok，改坏之后红了也证明不了什么，这几行的变异没跑）：",
         ["先看点名的测试读的文件在不在拷贝范围里（.claude/gate.d/stage-inputs.tsv 本阶段那一行；bash .claude/gate.d/code-source-discipline.sh 静态判这一样），",
          "不在就登记进那一行；再在真仓上单跑它（照那一行的 cargo test 参数），真仓上也红就是测试本来就坏，先修测试；对上多个用例的把点名写全到模块路径。"]),
        (invalid, f"有变异无效（无效 {len(invalid)} 条；替换文写进源码之后编不过，那条行为今天零变异覆盖）：",
         ["先看尾巴分来源，不是去补用例。有 process didn't exit successfully 与信号的，是测试进程被杀，先查是谁杀的；点名的名字里带 $ 这类字符的，改那一行的测试名；其余是替换文编不过——",
          "先看反斜杠：表里只有 \\n 会被还原成换行，\\& \\\" 这类原样写进源码就编不过；",
          "别的编译错就在副本里把替换后的那一行 cargo check 一遍，改成编得过、而且真会改行为的写法（.claude/rules/mutation-sampling.md 第八类）。"]),
        (failures, f"有变异没红（没红 {len(failures)} 条）：",
         ["没红 = 那处改法没有任何测试守着：先造一条会红的用例再改代码（show-me-test.md）；「没跑到」多半是 cargo test 参数选错了范围。"]),
        (memory_hits, f"有变异撞了内存上限（内存撞顶 {len(memory_hits)} 条，每条上限 {memory_max}）：",
         ["撞顶 = 这条破坏让被测代码无界分配，点名的测试没来得及红——不算抓到，也不是没红。换一处改法让它红在点名的测试上，",
          "或者给被测代码加一道先红的界（循环步数上限、分配量的断言）；正常的测试确实要这么多内存，才调大上限（表头「# 每条变异的内存上限：…」或 GATE_MUTATION_MEMORY_MAX）。"]),
        (timeouts, f"有变异超时（超时 {len(timeouts)} 条，每条限 {timeout_seconds} 秒）：",
         ["超时 = 这条破坏让被测代码不终止了，或者点名的测试在并发下就要跑这么久——不算抓到，也不是没红。先在副本里只改坏这一处、",
          "不限时单跑它那一行的 cargo test 参数，看它结束不结束：不结束就换一处改法让它红在点名的测试上，或者给被测代码加一道先红的界（循环步数上限）；",
          "结束得了就调大限时（表头「# 每条变异的超时秒数：…」或 GATE_MUTATION_TIMEOUT），别把它记成抓到。"]),
        (squeezed, f"有变异被 slice 的总上限挤掉（被总上限挤掉 {len(squeezed)} 条；这几条没撞各自的上限 {memory_max}，是 slice 里合起来满了）：",
         ["这几条的结果不算数，重跑（抓到的有按条记录，只重跑没作数的）；常这样说明包装的峰值表记少了（同时跑的几条实际占的比表里记的多），",
          "跑的时候 bash research/scripts/run-with-memory-cap.sh --status 看 slice 里是谁在占，别拿掉包装去跑。"]),
        (refused, f"有变异排不上没跑（排不上没跑 {len(refused)} 条；包装等满了等待上限，slice 还是放不下）：",
         ["bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，等那几件重活跑完再重跑；", "别拿掉包装去跑——排不上说明此刻整机放不下。"]),
        (unavailable, f"跑到一半带内存上限的 scope 起不来了（{len(unavailable)} 条没跑）：",
         ["修好用户级 systemd / D-Bus（systemctl --user status）再重跑；不许拿掉上限去跑。"]),
    ]
    for items, headline, howto_lines in sections:
        if items:
            print_rejection(headline, items, howto_lines)
    if left_behind:
        print_rejection(f"有 {len(left_behind)} 个临时目录跑完没删掉（{outcome['per_mutation_temporary_root']} 底下还剩这几个；line-<行号> 是变异、baseline-<行号> 是那一批基线，行号是 {TABLE} 的）：",
                        [f"{name}：里面 {len(outcome['left_behind_contents'][name])} 项，前几项 {'、'.join(outcome['left_behind_contents'][name][:5]) or '（空）'}"
                         for name in left_behind],
                        ["看 judge_row / judge_batch 末尾 measure_and_remove_temporary_directory 那一步跑到没有、删不删得掉（只读的目录、没退干净的进程又把它建回来）；",
                         "这一道收尾时连同源码副本一起删掉，现场只剩上面列的名字。别拿掉这一步：被杀的测试 Drop 守卫不跑，它建的镜像全堆在这一轮的 TMPDIR 里，一轮能堆几百 G。"])
    red = bool(invalid or failures or memory_hits or timeouts or squeezed or refused or unavailable or baseline_bad or left_behind)
    note(f"  … 这一轮 {counts['worker_count']} 个工作进程，每个 cargo 编译并行度 {counts['cargo_jobs']}，每条变异内存上限 {memory_max}、限时 {timeout_seconds} 秒；"
         f"复用抓到 {counts['reused']} 条、复用基线 ok {counts['reused_baseline']} 行，现跑变异 {counts['fresh_mutations']} 条、现跑基线 {counts['fresh_batches']} 批")
    if selection_path:
        # 只判了一部分：不写全绿标记；都抓到也退 3（门禁汇总记失败，不会被当成通过），有红退 1
        say(f"  … 只判了选中的 {len(rows)} 行，不是整道结论（GATE_MUTATION_ROW_SELECTION={selection_path}；"
            f"{'有红，见上面' if red else '选中的都红在点名的测试上'}；按条记录写进了 {counts['records_directory']}）")
        return 1 if red else EXIT_SELECTION_ONLY
    if red:
        return 1
    # ⚠️ 进程数、复用几条不写进 stdout：它们是两次跑唯一不同的输入，写进成功行就让「GATE_MUTATION_WORKERS=1 与 =16 的 stdout 逐字相同」当场为假
    say(f"  ✓ crates 变异表复跑：{len(rows)} 条变异各自红在点名的测试上（原文都恰好命中一次；点名的测试在没改坏的副本上都恰好一个 ok；"
        f"每条在内存上限 {memory_max} 里跑、内存撞顶 0 条，每条经 run-with-memory-cap.sh 排队、被总上限挤掉与排不上 0 条，每条限时 {timeout_seconds} 秒、超时 0 条，"
        f"每条一个自己的 TMPDIR、结束时都删掉了；工作进程动态领活——谁先跑完谁再领下一条，不按条数预先切片；输出按表的行号排序、与进程数和复用了多少都无关）")
    write_stage_marker_with_counts(counts["root"], dict(reused_caught_rows=counts["reused"], reused_baseline_ok_rows=counts["reused_baseline"],
                                                        freshly_run_mutation_rows=counts["fresh_mutations"], freshly_run_baseline_batches=counts["fresh_batches"],
                                                        row_records=counts["records_directory"]))
    return 0


def write_stage_marker_with_counts(root, counts):
    """判绿之后写这一道这批输入的全绿标记（admission.py 的 write_stage_marker），再把复用几条、现跑几条接在末尾（同目录临时文件写完改名）。写不成只报不红。"""
    admission = admission_module()
    try:
        fingerprint, file_count, _paths = admission.stage_fingerprint(root, STAGE)
        marker_path = admission.write_stage_marker(root, STAGE, fingerprint, file_count)
        with open(marker_path, encoding="utf-8", errors="surrogateescape") as handle:
            text = handle.read()
        write_atomically(marker_path, text + "".join(f"{key}={value}\n" for key, value in counts.items()))
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        say(f"  ! 没写成这一道的全绿标记（{error}）：下一趟这一道照跑，不会复用")


# ── 命令行 ────────────────────────────────────────────────────────────────────

def command_static_check(arguments):
    if not arguments or len(arguments) not in (1, 3) or (len(arguments) == 3 and arguments[1] != "--table"):
        print("ERROR\t用法：crates-mutation-rows.py static-check <根> [--table <变异表>]")
        return 2
    root = arguments[0]
    table_path = arguments[2] if len(arguments) == 3 else os.path.join(root, TABLE)
    admission = admission_module()
    try:
        copied = copied_paths()
    except admission.RegistrationError as error:
        print(f"ERROR\t读不到 .claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行：{error}")
        return 2
    if not copied:
        print(f"ERROR\t.claude/gate.d/stage-inputs.tsv 里 {STAGE} 那一行一条路径都没登记")
        return 2
    rows, malformed = [], []
    if os.path.isfile(table_path):
        try:
            rows, malformed, _memory, _timeout = read_table(table_path)
        except (OSError, UnicodeDecodeError) as error:
            print(f"ERROR\t读不了 {table_path}：{error}")
            return 2
    problems = row_problems(root, rows, malformed, copied)
    literals, scanned = escaping_literals(root, copied)
    outside_literals = [literal for literal in literals if not literal[4]]
    print("COPIED\t" + " ".join(copied))
    for kind in ROW_PROBLEM_KINDS:
        for detail in problems[kind]:
            print("\t".join([kind] + [str(item) for item in detail]))
    for relative, line_number, literal, resolved, _inside in outside_literals:
        print(f"LITERAL\t{relative}\t{line_number}\t{literal}\t{resolved}")
    print(f"CHECKED\t{len(rows)}\t{scanned}\t{len(literals)}")
    return 1 if any(problems.values()) or outside_literals else 0


def command_base_fingerprint(arguments, want_directory=False):
    if len(arguments) != 1:
        print(f"  ✗ 用法：crates-mutation-rows.py {'records-directory' if want_directory else 'base-fingerprint'} <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    admission = admission_module()
    environment = dict(os.environ)
    try:
        fingerprint, file_count = base_fingerprint(arguments[0], environment, environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip()
                                                   or DEFAULT_MEMORY_CAP_RUNNER)
        common_directory = records_home(arguments[0], environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print(f"  ✗ 算不出底座指纹：{error}")
        print("     → 怎么办：被判的要是 git 工作树、cargo -V 与 rustc -V 跑得通、.claude/gate.d/stage-inputs.tsv 里有 checker-tier-crates-mutation-replay 那一行")
        return 2
    print(os.path.join(common_directory, RECORDS_DIRECTORY_NAME, fingerprint) if want_directory else f"{fingerprint} {file_count}")
    return 0


def command_row_plan(arguments):
    """每一条成形的变异一行：行号、行键、有没有作数的抓到记录、上一次现跑的秒数（见文件头 row-plan）。"""
    if len(arguments) != 1:
        print("  ✗ 用法：crates-mutation-rows.py row-plan <根>")
        print("     → 怎么办：<根> 是被判那棵树的仓根（git 工作树）")
        return 2
    root = arguments[0]
    admission = admission_module()
    environment = dict(os.environ)
    try:
        rows, _malformed, memory_max_from_table, timeout_from_table = read_table(os.path.join(root, TABLE), environment)
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {os.path.join(root, TABLE)}：{error}")
        print("     → 怎么办：先让这张表在、是 UTF-8；读不了就一行都分不出去")
        return 2
    memory_max, timeout_seconds, limit_problems = limits_of(memory_max_from_table, timeout_from_table, environment)
    if limit_problems:
        print(f"  ✗ {limit_problems[0][0]}")
        print(f"     → 怎么办：{limit_problems[0][1]}")
        return 2
    try:
        base, _file_count = base_fingerprint(root, environment, environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip() or DEFAULT_MEMORY_CAP_RUNNER)
        home = records_home(root, environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print(f"  ✗ 算不出底座指纹：{error}")
        print("     → 怎么办：被判的要是 git 工作树、cargo -V 与 rustc -V 跑得通、.claude/gate.d/stage-inputs.tsv 里有 checker-tier-crates-mutation-replay 那一行")
        return 2
    records_directory = os.path.join(home, RECORDS_DIRECTORY_NAME, base)
    timings_directory = os.path.join(home, TIMINGS_DIRECTORY_NAME)
    refusal = reuse_refusal(environment)
    for row in rows:
        key = row_key(row, memory_max, timeout_seconds)
        settled = not refusal and reusable_record(records_directory, "mutation", key, base, environment) is not None
        timing = read_timing(timings_directory, row_content_hash(row))
        print(f"{row.line_number}\t{key}\t{1 if settled else 0}\t{'-' if timing is None else f'{timing[0]:.3f}'}")
    return 0


def command_import(arguments):
    if len(arguments) != 2:
        print("  ✗ 用法：crates-mutation-rows.py import <根> <别处的记录目录>")
        print("     → 怎么办：<别处的记录目录> 是另一台 singlefs-mutation-rows/<底座指纹>/ 拷过来的那一份")
        return 2
    root, source_directory = arguments
    admission = admission_module()
    environment = dict(os.environ)
    try:
        rows, _malformed, memory_max_from_table, timeout_from_table = read_table(os.path.join(root, TABLE), environment)
    except (OSError, UnicodeDecodeError) as error:
        print(f"  ✗ 读不了 {os.path.join(root, TABLE)}：{error}")
        print("     → 怎么办：行键要按本机的变异表算，先让这张表在、是 UTF-8")
        return 2
    memory_max, timeout_seconds, limit_problems = limits_of(memory_max_from_table, timeout_from_table, environment)
    if limit_problems:
        print(f"  ✗ {limit_problems[0][0]}")
        print(f"     → 怎么办：{limit_problems[0][1]}")
        return 2
    try:
        base, _file_count = base_fingerprint(root, environment, environment.get("GATE_MUTATION_MEMORY_CAP_RUNNER", "").strip() or DEFAULT_MEMORY_CAP_RUNNER)
        common_directory = records_home(root, environment)
    except (admission.RegistrationError, admission.InputManifestError, OSError) as error:
        print(f"  ✗ 算不出本机的底座指纹：{error}")
        print("     → 怎么办：被判的要是 git 工作树、cargo -V 与 rustc -V 跑得通；算不出就一条都不导")
        return 2
    if not os.path.isdir(source_directory):
        print(f"  ✗ {source_directory} 不是目录")
        print("     → 怎么办：给另一台 singlefs-mutation-rows/<底座指纹>/ 拷过来的那一份目录")
        return 2
    local_keys = {row_key(row, memory_max, timeout_seconds) for row in rows}
    records_directory = os.path.join(common_directory, RECORDS_DIRECTORY_NAME, base)
    os.makedirs(records_directory, exist_ok=True)
    accepted, rejected = [], []
    for name in sorted(os.listdir(source_directory)):
        path = os.path.join(source_directory, name)
        record = read_record(path) if os.path.isfile(path) else None
        if record is None:
            rejected.append((name, "不是一份读得出的记录文件"))
            continue
        problems = []
        kind, key = record.get("kind", ""), record.get("row_key", "")
        if kind not in RECORD_KINDS:
            problems.append(f"类别「{kind}」认不出（只认 {'、'.join(RECORD_KINDS)}）")
        if name != f"{kind}.{key}":
            problems.append("文件名与记录里的类别、行键对不上")
        if record.get("base_fingerprint") != base:
            problems.append(f"底座指纹 {record.get('base_fingerprint', '')[:16]}… 不是本机此刻的 {base[:16]}…")
        if key not in local_keys:
            problems.append(f"行键不是本机 {TABLE} 里任何一行（连同本机这一趟的每条上限 {memory_max}、限时 {timeout_seconds} 秒）算得出的")
        if break_is(environment, "importunchecked"):
            problems = []
        if problems:
            rejected.append((name, "；".join(problems)))
            continue
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            write_atomically(os.path.join(records_directory, name), handle.read())
        accepted.append(name)
    print(f"  … 导入：收了 {len(accepted)} 条、拒了 {len(rejected)} 条（写进 {records_directory}）")
    if rejected:
        print(f"  ✗ 拒了 {len(rejected)} 条另一台的按条记录（底座指纹或行键与本机此刻对不上，照收会让本机复用别的输入下的结论）：")
        for name, reason in rejected:
            print(f"      {name}：{reason}")
        print("     → 怎么办：两台的仓要在同一个提交、同一份工作区改动上（底座指纹不同多半是源码或工具链不一样），变异表同一份、上限与限时同一套；")
        print("       对齐之后在另一台重跑那几行再导；拒掉的那几行本机 checker-tier-crates-mutation-replay 照跑，不会少判。")
        return 1
    return 0


# ── 自证 ──────────────────────────────────────────────────────────────────────

FAKE_CARGO = r'''#!/usr/bin/env python3
# 自证用的假 cargo：-V 打一行版本；test 读 crates/tiny/src/lib.rs 里「// TEST <名字> NEEDS <原文>」「// TEST <名字> READS <相对 crate 根的路径>」，
# 原文在、文件在就 ok，不在就 FAILED；每起一次往 $FAKE_CARGO_LOG 追加一行「<参数>\t<lib.rs 的 sha256 前 12 位>」
import hashlib, os, sys
arguments = sys.argv[1:]
if arguments[:1] in (["-V"], ["--version"]):
    print("cargo 0.0.0 (crates-mutation-rows selftest)")
    sys.exit(0)
if arguments[:1] != ["test"]:
    sys.exit(2)
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
with open(os.environ["FAKE_CARGO_LOG"], "a", encoding="utf-8") as log:
    log.write(" ".join(arguments[1:]) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
filters = [word for word in after if not word.startswith("-")]
failed = False
for line in source.splitlines():
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, how, what = words[2], words[3], words[4]
    if filters and not any(word in "tests::" + name for word in filters):
        continue
    ok = (what in source.replace(line, "")) if how == "NEEDS" else os.path.isfile(os.path.join("crates", "tiny", what))
    failed = failed or not ok
    print(f"test tests::{name} ... {'ok' if ok else 'FAILED'}")
sys.exit(101 if failed else 0)
'''
FAKE_MEMORY_CAP_RUNNER = '''#!/usr/bin/env bash
# 自证用的假内存包装：--check 退 0；其余去掉上限那一个参数、原样执行命令
if [[ "${1:-}" == "--check" ]]; then exit 0; fi
shift
exec "$@"
'''
SELFTEST_SOURCE = '''// 本模块自证的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
'''
SELFTEST_TABLE_ROWS = [
    "add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left * right }\t-p tiny --lib -- adds\tadds",
    "double-becomes-triple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 3 }\t-p tiny --lib -- doubles\tdoubles",
    "negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --lib -- negates\tnegates",
    "halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --lib -- halves\thalves",
]
SELFTEST_READS_OUTSIDE_TEST = "// TEST reads_answer READS ../../fixture-data/answer.txt\n"
SELFTEST_READS_OUTSIDE_ROW = "reads-outside\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ 1 - value }\t-p tiny --lib -- reads_answer\treads_answer"
# 替换文空着的删行变异（真表里删掉一句调用、删掉 litmus 里一道屏障的那一类）
SELFTEST_DELETION_ROW = "halve-deleted\tcrates/tiny/src/lib.rs\t{ value / 2 }\t\t-p tiny --lib -- halves\thalves"
# 派活之前那几样各踩一样的行（接在 4 行正常变异与删行变异之后：变异表第 7–11 行），ROW 那一行改的 docs/outside.rs 另写进样本仓
SELFTEST_STATIC_PROBLEM_ROWS = [
    ("MALFORMED", "empty-expected\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\t"),
    ("ANCHOR", "anchor-missing\tcrates/tiny/src/lib.rs\t{ left / right }\t{ left }\t-p tiny --lib -- adds\tadds"),
    ("NAME", "name-anchored\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left + 1 }\t-p tiny --lib -- adds\tadds$"),
    ("ROW", "outside-copy\tdocs/outside.rs\tpub fn outside() {}\tpub fn outside() { }\t-p tiny --lib -- adds\tadds"),
    ("STRAY", "stray-escape\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ \\&value * 2 }\t-p tiny --lib -- doubles\tdoubles"),
]


def selftest_write(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


def build_selftest_repository(directory, table_rows, extra_source=""):
    """一棵临时 git 仓：Cargo.toml、crates/tiny、变异表、仓根下拷贝范围之外的 fixture-data/answer.txt。"""
    selftest_write(os.path.join(directory, "Cargo.toml"), '[workspace]\nmembers = ["crates/tiny"]\n')
    selftest_write(os.path.join(directory, "crates", "tiny", "Cargo.toml"), '[package]\nname = "tiny"\nversion = "0.1.0"\n')
    selftest_write(os.path.join(directory, "crates", "tiny", "src", "lib.rs"), SELFTEST_SOURCE + extra_source)
    selftest_write(os.path.join(directory, TABLE), "# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n" + "".join(row + "\n" for row in table_rows))
    selftest_write(os.path.join(directory, "fixture-data", "answer.txt"), "42\n")
    subprocess.run(["git", "init", "-q", directory], check=True)


class Selftest:
    def __init__(self, work):
        self.work = work
        self.cells = 0
        self.failures = []
        self.bin_directory = os.path.join(work, "bin")
        selftest_write(os.path.join(self.bin_directory, "cargo"), FAKE_CARGO, executable=True)
        self.runner = os.path.join(work, "fake-memory-cap.sh")
        selftest_write(self.runner, FAKE_MEMORY_CAP_RUNNER, executable=True)
        self.log = os.path.join(work, "cargo.log")
        self.base_environment = {key: value for key, value in os.environ.items() if key not in STAGE_VARIABLES}
        self.base_environment.update(PATH=self.bin_directory + os.pathsep + os.environ.get("PATH", ""), FAKE_CARGO_LOG=self.log,
                                     GATE_MUTATION_MEMORY_CAP_RUNNER=self.runner, GATE_MUTATION_TARGET_DIR=os.path.join(work, "target"),
                                     GATE_MUTATION_WORKERS="1")

    def run(self, root, **changes):
        """在 root 上跑一趟 run_stage：(退出码, stdout, stderr, 这一趟假 cargo 起的每一行)。"""
        with contextlib.suppress(FileNotFoundError):
            os.unlink(self.log)
        environment = dict(self.base_environment, **changes)
        stdout, stderr = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
            exit_code = run_stage(root, environment)
        invocations = []
        if os.path.exists(self.log):
            with open(self.log, encoding="utf-8") as handle:
                invocations = handle.read().splitlines()
        return exit_code, stdout.getvalue(), stderr.getvalue(), invocations

    def check(self, label, condition, detail=""):
        self.cells += 1
        if not condition:
            self.failures.append(f"{label}{('：' + detail) if detail else ''}")


def mutation_invocations(invocations, pristine_hash):
    """假 cargo 的记录里改坏了源码的那几次（lib.rs 的哈希不是没改坏的那一份）。"""
    return [line for line in invocations if not line.endswith("\t" + pristine_hash)]


def source_hash(root):
    with open(os.path.join(root, "crates", "tiny", "src", "lib.rs"), encoding="utf-8") as handle:
        return hashlib.sha256(handle.read().encode()).hexdigest()[:12]


def replace_in_file(path, old, new):
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    assert text.count(old) == 1, (path, old)
    selftest_write(path, text.replace(old, new))


def run_selftest():
    work = tempfile.mkdtemp(prefix="crates-mutation-rows-selftest-")
    saved_environment = dict(os.environ)
    try:
        selftest = Selftest(work)
        # admission.toolchain_line 用本进程的 PATH 起 cargo -V，import 与 records-directory 读本进程的环境：与 run_stage 那一份对齐
        os.environ.clear()
        os.environ.update(selftest.base_environment)
        run_selftest_cells(selftest, work)
    finally:
        os.environ.clear()
        os.environ.update(saved_environment)
        shutil.rmtree(work, ignore_errors=True)
    if selftest.failures:
        print(f"  ✗ crates-mutation-rows.py 自证有 {len(selftest.failures)} 格判错（共 {selftest.cells} 格）：")
        for failure in selftest.failures:
            print(f"      {failure}")
        print("     → 怎么办：设了 GATE_MUTATION_BREAK 的，这就是要的结果（那个开关把对应的判法弄坏了）；没设的，照上面那几格去看 run_stage、base_fingerprint、")
        print("       dispatch_order、command_import、escaping_literals、read_table 与 row_problems 里对应的那一段。")
        return 1
    print(f"  ✓ crates-mutation-rows.py 自证 {selftest.cells} 格都对（复用、改一行只重跑那一行、改源码全部重跑、跑到一半续跑、基线不绿、"
          "久的先派、选行只判一部分退 3、导入核行键与底座指纹、只留 3 个底座指纹目录、记录里没有主机名、拷贝范围两样、"
          "替换文空着的删行变异照常判、派活之前那几样 checker-tier-crates-mutation-replay 与 static-check 同一份判法）")
    return 0


def run_selftest_cells(selftest, work):
    repository = os.path.join(work, "repository")
    build_selftest_repository(repository, SELFTEST_TABLE_ROWS)
    pristine = source_hash(repository)
    # 一：头一趟全跑；第二趟输入不变，假 cargo 一次都不起、stdout 逐字相同
    first = selftest.run(repository)
    selftest.check("头一趟判绿", first[0] == 0, first[1] + first[2])
    selftest.check("头一趟起 1 批基线 + 4 条变异", len(first[3]) == 5, "\n".join(first[3]))
    second = selftest.run(repository)
    selftest.check("第二趟输入不变：cargo 一次都不起", second[0] == 0 and second[3] == [], "\n".join(second[3]))
    selftest.check("第二趟 stdout 与头一趟逐字相同（判定与复用了多少无关）", second[1] == first[1], second[1])
    selftest.check("第二趟 stderr 头一行报复用 4 条", "复用抓到 4 条" in (second[2].splitlines() or [""])[0], second[2])
    # 二：进程数不同、一律不复用，stdout 仍逐字相同
    parallel = selftest.run(repository, GATE_MUTATION_WORKERS="4", GATE_MUTATION_START_OVER="1")
    selftest.check("GATE_MUTATION_WORKERS=4 与 =1 的 stdout 逐字相同", parallel[1] == first[1], parallel[1])
    selftest.check("GATE_MUTATION_START_OVER=1 一律不复用", len(parallel[3]) == 5 and "不复用：GATE_MUTATION_START_OVER=1" in parallel[2], parallel[2])
    full = selftest.run(repository, SINGLEFS_GATE_FULL="1")
    selftest.check("SINGLEFS_GATE_FULL=1 一律不复用", len(full[3]) == 5, "\n".join(full[3]))
    expired = selftest.run(repository, SINGLEFS_REUSE_HOURS="0")
    selftest.check("过了 SINGLEFS_REUSE_HOURS 不复用", len(expired[3]) == 5, "\n".join(expired[3]))
    # 三：只改一行的替换文，只那一行重跑（它自己的基线 + 它的变异）
    table_path = os.path.join(repository, TABLE)
    replace_in_file(table_path, "{ value * 3 }", "{ value * 4 }")
    one_row = selftest.run(repository)
    changed = mutation_invocations(one_row[3], pristine)
    selftest.check("只改一行的替换文：只那一行重跑", one_row[0] == 0 and len(one_row[3]) == 2 and len(changed) == 1 and "doubles" in changed[0],
                   "\n".join(one_row[3]))
    # 四：改一份被测源码（锚点照旧命中），全部重跑；弄坏开关 basewithoutsource 下会误判成全部复用
    replace_in_file(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), "pub fn add(", "// 改了一行注释\npub fn add(")
    pristine = source_hash(repository)
    source_changed = selftest.run(repository)
    selftest.check("改一份被测源码：全部重跑（1 批基线 + 4 条变异）", source_changed[0] == 0 and len(source_changed[3]) == 5, "\n".join(source_changed[3]))
    # 五：跑到一半停掉（只供测试的开关），下一趟只跑剩下的；基线已判过 ok 的不再跑。先清掉这一格底座的记录，从零起
    shutil.rmtree(run_command_capturing(command_base_fingerprint, [repository], want_directory=True)[1].strip(), ignore_errors=True)
    halfway =selftest.run(repository, GATE_MUTATION_START_OVER="1", GATE_MUTATION_STOP_AFTER_ROWS="2")
    selftest.check("跑完 2 条就退：判红、说明是只供测试的开关", halfway[0] == 1 and "GATE_MUTATION_STOP_AFTER_ROWS=2" in halfway[1]
                   and len(mutation_invocations(halfway[3], pristine)) == 2, halfway[1] + "\n".join(halfway[3]))
    resumed = selftest.run(repository)
    selftest.check("下一趟只跑剩下的 2 条、基线不再跑", resumed[0] == 0 and len(resumed[3]) == 2 and len(mutation_invocations(resumed[3], pristine)) == 2,
                   "\n".join(resumed[3]))
    # 六：上一次跑得久（或超时）的先派，按用时从长到短；stderr 头一行报前移几条
    timings_directory = os.path.join(repository, ".git", TIMINGS_DIRECTORY_NAME)
    table_rows, _malformed, _memory, _timeout = read_table(table_path)
    long_rows = {"negates": ("400", "0"), "halves": ("1800", "1")}
    for row in table_rows:
        if row.expected in long_rows:
            seconds, timed_out = long_rows[row.expected]
            write_atomically(os.path.join(timings_directory, row_content_hash(row)),
                             record_text(dict(seconds_running=seconds, timed_out=timed_out, finished_utc=utc_now_text())))
    ordered = selftest.run(repository, GATE_MUTATION_START_OVER="1")
    order = [line.split("\t")[0].split()[-1] for line in mutation_invocations(ordered[3], pristine)]
    ordered_head_line = (ordered[2].splitlines() or [""])[0]
    selftest.check("上一次超时的、跑得久的先派（按用时从长到短）", order[:2] == ["halves", "negates"] and "前移 2 条" in ordered_head_line,
                   f"派活次序 {order}；{ordered_head_line}")
    # 七：选行只判一部分：都抓到退 3、末行写明不是整道结论；不写全绿标记
    selection_file = os.path.join(work, "selection.txt")
    selftest_write(selection_file, "".join(f"{row.line_number}\n" for row in table_rows[:2]))
    selected = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=selection_file, GATE_MUTATION_START_OVER="1")
    selftest.check("选 2 行：都抓到退 3，末行写明只判了选中的 2 行", selected[0] == EXIT_SELECTION_ONLY
                   and "只判了选中的 2 行，不是整道结论" in selected[1].splitlines()[-1] and len(mutation_invocations(selected[3], pristine)) == 2,
                   selected[1])
    selftest_write(os.path.join(work, "bad-selection.txt"), "99999\n")
    bad_selection = selftest.run(repository, GATE_MUTATION_ROW_SELECTION=os.path.join(work, "bad-selection.txt"))
    selftest.check("选行文件里的行号不是一条变异：判红、一格都没跑", bad_selection[0] == 1 and bad_selection[3] == [], bad_selection[1])
    # 八：导入另一台的记录：同一份内容的另一棵仓先全收，之后一格都不跑、stdout 与本机现跑逐字相同；行键或底座指纹对不上的拒
    reference = selftest.run(repository)
    other = os.path.join(work, "other")
    build_selftest_repository(other, [line for line in open(table_path, encoding="utf-8").read().splitlines()[1:]])
    shutil.copyfile(os.path.join(repository, "crates", "tiny", "src", "lib.rs"), os.path.join(other, "crates", "tiny", "src", "lib.rs"))
    source_records = run_command_capturing(command_base_fingerprint, [repository], want_directory=True)[1].strip()
    imported = run_command_capturing(command_import, [other, source_records])
    after_import = selftest.run(other)
    selftest.check("导入同一底座的记录：全收", imported[0] == 0, imported[1])
    selftest.check("导入之后另一棵仓一格都不跑、stdout 与本机逐字相同", after_import[3] == [] and after_import[1] == reference[1], after_import[1] + after_import[2])
    forged_directory = os.path.join(work, "forged")
    os.makedirs(forged_directory)
    genuine_name = sorted(name for name in os.listdir(source_records) if name.startswith("mutation."))[0]
    genuine = read_record(os.path.join(source_records, genuine_name))
    forged_key = hashlib.sha256(b"no such row").hexdigest()
    selftest_write(os.path.join(forged_directory, f"mutation.{forged_key}"), record_text(dict(genuine, row_key=forged_key)))
    foreign_base = dict(genuine, base_fingerprint=hashlib.sha256(b"another base").hexdigest())
    selftest_write(os.path.join(forged_directory, genuine_name), record_text(foreign_base))
    forged = run_command_capturing(command_import, [other, forged_directory])
    selftest.check("导入一条行键对不上的记录：拒、列出来", forged[0] == 1 and f"mutation.{forged_key}" in forged[1] and "行键不是本机" in forged[1], forged[1])
    selftest.check("导入一条底座指纹对不上的记录：拒", "底座指纹" in forged[1] and "拒了 2 条" in forged[1], forged[1])
    # 九：只留最新的 3 个底座指纹目录，别的名字不动
    records_root = os.path.dirname(source_records)
    for index in range(4):
        stale = os.path.join(records_root, hashlib.sha256(f"stale {index}".encode()).hexdigest())
        os.makedirs(stale)
        os.utime(stale, (1000 + index, 1000 + index))
    keep_me = os.path.join(records_root, "not-a-fingerprint")
    os.makedirs(keep_me)
    selftest.run(repository)
    remaining = sorted(name for name in os.listdir(records_root) if FINGERPRINT_FORM.fullmatch(name))
    selftest.check("只留最新的 3 个底座指纹目录（这一趟的在里面），别的名字不动", len(remaining) == 3 and os.path.basename(source_records) in remaining
                   and os.path.isdir(keep_me), f"{remaining}")
    # 十：记录里不写主机名
    host_name = os.uname().nodename
    record_texts = [open(os.path.join(source_records, name), encoding="utf-8").read() for name in os.listdir(source_records)]
    selftest.check("记录里不写主机名", len(host_name) >= 3 and record_texts and not any(host_name in text for text in record_texts), host_name)
    # 十一：基线不绿（点名的测试读一个副本里没有的文件）：不算抓到、整道判红；弄坏开关 nobaseline 下误判成抓到
    outside = os.path.join(work, "outside")
    build_selftest_repository(outside, SELFTEST_TABLE_ROWS + [SELFTEST_READS_OUTSIDE_ROW], SELFTEST_READS_OUTSIDE_TEST)
    baseline_red = selftest.run(outside)
    selftest.check("点名的测试读副本里没有的文件：记基线不绿、不算抓到、整道判红", baseline_red[0] == 1 and "基线不绿 1 行" in baseline_red[1]
                   and "reads-outside：基线不绿——点名的测试 reads_answer 在没改坏的副本上的结果是 FAILED" in baseline_red[1]
                   and "计数：抓到 4 条" in baseline_red[1], baseline_red[1])
    # 十二：每片的副本不留原来的修改时刻（留着的话，跨轮复用的编译目录里上一轮改坏的二进制会被 cargo 当成新的，基线跑的是改坏的代码）
    stale_source = os.path.join(repository, "crates", "tiny", "src", "lib.rs")
    os.utime(stale_source, (1000, 1000))
    shard_root = os.path.join(work, "shard-root")
    os.makedirs(shard_root)
    shard_work, _target = prepare_shard(1, shard_root, dict(copied=["crates", "Cargo.toml"], root=repository, shared_target_directory=os.path.join(work, "t")))
    copied_mtime = os.stat(os.path.join(shard_work, "crates", "tiny", "src", "lib.rs")).st_mtime
    selftest.check("每片的副本不留原来的修改时刻（取拷的那一刻）", copied_mtime > 1000 + 3600, f"副本的修改时刻 {copied_mtime}")
    # 十三：拷贝范围两样（code-source-discipline 调的那一份判法）
    copied = ["crates", "Cargo.toml", "Cargo.lock", "litmus"]
    selftest.check("拷贝范围：litmus/ 下的文件在、docs/ 下的不在、target 底下的不在",
                   is_copied_into_each_shard("litmus/a.litmus", copied) and not is_copied_into_each_shard("docs/a.rs", copied)
                   and not is_copied_into_each_shard("crates/x/target/debug/a", copied) and not is_copied_into_each_shard("../crates/a", copied))
    scope_root = os.path.join(work, "scope")
    selftest_write(os.path.join(scope_root, "crates", "demo", "tests", "paths.rs"),
                   '#[path = "../../outside/mod.rs"]\nmod outside;\n// "../../in-a-comment/x"\n/* "../../in-a-block/x" */\n'
                   'const A: &str = "../../litmus";\nconst B: &str = r#"../../fixture-data/x"#;\nconst C: &str = "../other-crate/src";\n'
                   'const D: &str = "../../";\nlet e = b"../../bytes/x";\n')
    literals, scanned = escaping_literals(scope_root, copied)
    outside_literals = sorted((line, resolved) for _file, line, _literal, resolved, inside in literals if not inside)
    inside_literals = [resolved for _file, _line, _literal, resolved, inside in literals if inside]
    selftest.check("逃出 crates/ 的字面量：#[path]、注释、crates/ 之内的不算，litmus 在范围里，原始字符串、字节串、指到仓根的判不在",
                   scanned == 1 and inside_literals == ["litmus"] and outside_literals == [(6, "fixture-data/x"), (8, "."), (9, "bytes/x")],
                   f"在范围里 {inside_literals}；不在 {outside_literals}")
    # 十四：替换文空着的删行变异：成形、照常改坏源码（原文换成空串）、判抓到；弄坏开关 emptyreplacementrefused 下判不成形、这一格判错
    deletion = os.path.join(work, "deletion")
    build_selftest_repository(deletion, SELFTEST_TABLE_ROWS + [SELFTEST_DELETION_ROW])
    deleted = selftest.run(deletion)
    deleted_hash = hashlib.sha256(SELFTEST_SOURCE.replace("{ value / 2 }", "", 1).encode()).hexdigest()[:12]
    selftest.check("替换文空着的删行变异：成形、原文换成空串去跑、判抓到", deleted[0] == 0 and "✓ halve-deleted：halves 红了" in deleted[1]
                   and any(line.endswith("\t" + deleted_hash) for line in deleted[3]), deleted[1] + "\n".join(deleted[3]))
    # 十五：派活之前那几样只有一份判法：static-check（门禁 code-source-discipline 调它）与 run_stage 的 prechecks 对同一张表报同样几行，删行变异（第 6 行）两边都不报
    static_root = os.path.join(work, "static")
    build_selftest_repository(static_root, SELFTEST_TABLE_ROWS + [SELFTEST_DELETION_ROW] + [row for _kind, row in SELFTEST_STATIC_PROBLEM_ROWS])
    selftest_write(os.path.join(static_root, "docs", "outside.rs"), "pub fn outside() {}\n")
    expected_problems = [(kind, 7 + index) for index, (kind, _row) in enumerate(SELFTEST_STATIC_PROBLEM_ROWS)]
    static = run_command_capturing(command_static_check, [static_root])
    reported = [(line.split("\t")[0], int(line.split("\t")[1])) for line in static[1].splitlines() if line.split("\t")[0] in ROW_PROBLEM_KINDS]
    selftest.check("static-check：不成形、锚点、测试名、拷贝范围、转义五样各报一行，删行变异不报", static[0] == 1 and reported == expected_problems,
                   static[1])
    staged = selftest.run(static_root)
    selftest.check("checker-tier-crates-mutation-replay 派活之前判的是同一份：同样几行判红、删行变异不报、假 cargo 一次都没起",
                   staged[0] == 1 and staged[3] == [] and all(f"{TABLE}:{line_number} " in staged[1] for _kind, line_number in expected_problems)
                   and f"{TABLE}:6 " not in staged[1], staged[1])


def run_command_capturing(command, arguments, **keywords):
    stdout = io.StringIO()
    with contextlib.redirect_stdout(stdout):
        exit_code = command(arguments, **keywords)
    return exit_code, stdout.getvalue()


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return run_selftest()
    commands = {"static-check": command_static_check, "base-fingerprint": command_base_fingerprint,
                "records-directory": lambda rest: command_base_fingerprint(rest, want_directory=True), "import": command_import,
                "row-plan": command_row_plan}
    if not arguments or arguments[0] not in commands:
        print("  ✗ 用法：crates-mutation-rows.py {static-check|base-fingerprint|records-directory|import|row-plan} … 或 --selftest", file=sys.stderr)
        print("     → 怎么办：各子命令的参数见文件头；整张表的复跑只经门禁 checker-tier-crates-mutation-replay.sh（bash .claude/gate.d/checker-tier-crates-mutation-replay.sh）", file=sys.stderr)
        return 2
    return commands[arguments[0]](arguments[1:])


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
