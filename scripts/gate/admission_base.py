"""准入（入口 admission.py）的一块：常量、登记表 stage-inputs.tsv 的行读法、门禁与实验的准入条件、复跑表行与产物头的读法、崩溃枚举用例的行解析，以及别的块共用的小工具（多机配置、每台默认重活线程数、git 询问）。
入口 admission.py 把这一块的名字再导出。
"""

import importlib.util
import os
import re
import shutil
import subprocess
import sys

REGISTRATION_TABLE = ".claude/gate.d/stage-inputs.tsv"
REPLAY_SCRIPT = "scripts/experiment-apparatus/replay.sh"
RESULTS_DIRECTORY = "research/results"
GATE_GREEN_TREE_RECORD = "refs/sop/staged-green"   # 只有弄坏开关 tree-compare-reuse 读它（按树比的复用路）；平常一处都不读
BREAK_VARIABLE = "ADMISSION_BREAK"
HEADER_PREFIX = "E7INPUT "
HEADER_LINES_READ_AT_MOST = 20
FUNCTION_UNITS_LIBRARY = "lib_function_units.py"
VERDICT_LIBRARY_DIRECTORY_NAME = "singlefs-unit-verdicts"
RED_VERDICT_REASON = "上次判红"
CHANGED_UNITS_SHOWN_AT_MOST = 8
# 改判定域或结论的环境变量（文件头「单元键」的 variable 一行与「判定请求」的 variable:）：（名，或以 _ 收尾的前缀；怎么进键：value 进值、file 进那份文件的内容）。
#   前缀那几行里以 THREADS 收尾的不算（线程数不改结论）。名单照 crates/singlefs-verification 里
#   std::env::var / var_os 读的变量逐个过一遍定下：线程数、显卡、多机配置、库与日志放哪、只写不读这一类不改结论的不在里面
DOMAIN_ENVIRONMENT_VARIABLES = (
    ("SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY", "file"),    # 按段登记文件：登记了的段改展开、分片、核段写表摘要
    ("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES", "value"),  # 分片的段一趟判几个态
    ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO", "value"),       # 放量的展开深度
    ("SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO", "value"),               # 判器的展开深度
    ("SINGLEFS_CRASH_AMPLIFICATION_SHARE", "value"),              # 给定状态引擎这一台判几分之几
    ("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS", "value"),       # 几台各分多少
    ("SINGLEFS_CRASH_AMPLIFICATION_STAGE", "value"),              # 只跑哪一段（只录不判这一类）
    ("SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY", "value"),     # 只排派活计划、一块不判
    ("SINGLEFS_CRASH_INJECTION_", "value"),                       # 崩溃注入的种子、操作数、点数这一类取样参数
    ("SINGLEFS_BAD_DISK_", "value"),                              # 坏盘输入的取样参数
    ("SINGLEFS_RANDOM_HISTORY_", "value"),                        # 随机历史的取样参数
)
DOMAIN_VARIABLE_LITERAL = re.compile(r"""["']([A-Z][A-Z0-9_]*)["']""")

EXIT_ADMITTED = 0
EXIT_REGISTRATION_ERROR = 2
EXIT_PRECONDITION_MISSING = 3
EXIT_INPUTS_UNCHANGED = 77
EXIT_GATE_MUST_RUN = 0
EXIT_GATE_MAY_SKIP = 10
EXIT_GATE_PRECONDITION_MISSING = 1
EXIT_UNIT_KEY_REFUSED = 4
PROBE_TIMEOUT_SECONDS = 300
GATE_PRECONDITION_KINDS = ("command", "readwrite", "probe")

EXPERIMENT_KEY_FORM = re.compile(r"^E[0-9]+[A-Z0-9]*(/[a-z0-9][a-z0-9-]*)?$")
QUESTION_ROW_FORM = re.compile(r"^question-row=(?P<path>[^#]+)#(?P<row>[^:]+):(?P<pattern>.+)$")
PRODUCT_FIELD_FORM = re.compile(r"^product-field=(?P<key>[^:]+):(?P<line_name>[^:]+):(?P<field>[^=]+)=(?P<value>\S+)$")
REPLAY_ROW_FORM = re.compile(r"^(E[0-9]+[A-Z0-9]*)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$")
# 驱动里 "$(replay_frozen_root <名>)" 点名的冻结根：工作根下 replay-frozen-roots/<名>（replay.sh 的 replay_frozen_root），行键另收它的内容摘要（frozen_root_digest）
REPLAY_FROZEN_ROOT_CALL = re.compile(r"\$\(replay_frozen_root\s+([A-Za-z0-9._-]+)\)")
REPLAY_FROZEN_ROOTS_DIRECTORY = "replay-frozen-roots"
FROZEN_ROOT_SKIPPED_DIRECTORIES = frozenset((".git", "target", "__pycache__"))
REPLAY_ROW_KEYS_START = "ADMISSION_ROW_KEYS=$(cat <<'TSV'\n"
REPLAY_ROW_KEYS_END_LINE = "TSV"
REPLAY_ROW_HEADER_WRITERS = ("device", "replay")
GATE_CONDITION_FORM = re.compile(r"^(?P<kind>command|readwrite|probe|environment|copy-range)=(?P<value>\S+)$")
STAGE_CELL_KEY_FORM = re.compile(r"^[a-z0-9][a-z0-9-]*\.sh:[a-z0-9][a-z0-9-]*$")
ENTRY_FORM = re.compile(r"^(?:tests-of:[A-Za-z0-9_-]+|vm:[a-z0-9][a-z0-9-]*|@[^\s]+|[A-Za-z0-9_./+-]+(?:::[^\s]+)?)$")

# 崩溃枚举用例（门禁 crash-point-replay 逐条跑）
CRASH_CASE_KEY_PREFIX = "crash-case:"
CRASH_CASE_KEY_FORM = re.compile(r"^crash-case:[a-z0-9][a-z0-9-]*$")
CRASH_CASE_CONDITION_FORM = re.compile(r"^(?P<kind>test|count-line|exhaustive|threads|threads-variable|features|red-blocks|findings|blocks-line|shards-at-least)=(?P<value>\S+)$")
# 登记行的 shards-at-least=：这条用例经服务分片时至少分几片（文件头「崩溃枚举用例行」）；正整数、不带前导 0
CRASH_CASE_SHARDS_AT_LEAST_FORM = re.compile(r"^[1-9][0-9]*$")
CRASH_CASE_FEATURES_FORM = re.compile(r"^[a-z0-9][a-z0-9_-]*(,[a-z0-9][a-z0-9_-]*)*$")
# 弄坏开关 features-passed-to-every-case 给没登记 features= 的用例塞的占位特性（不是哪个包真有的特性）
UNREGISTERED_FEATURE_OF_BREAK = "unregistered-feature"
GPU_FEATURE = "gpu"
MULTI_HOST_CONFIGURATION_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
MULTI_HOST_CONFIGURATION_NAME = "multi-host.env"
GPU_QUIESCE_SCRIPT = "scripts/crash/gpu-quiesce-run.sh"   # 只给弄坏开关 gpu-case-quiesced-by-itself 用：清场归 GPU 批处理验证服务
SERVICE_CARD_VARIABLE = "GPU_CARDS_BY_PRIORITY"   # 调用方环境里有就原样交给命令、没有不设（卡归 GPU 批处理验证服务，.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）
BINDGEN_LIBRARY_SCRIPT = "scripts/crash/crash-amplification-lib.sh"
BINDGEN_VARIABLE = "BINDGEN_EXTRA_CLANG_ARGS"
BINDGEN_ASSIGNMENT_FORM = re.compile(r'^CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS="([^"]*)"', re.MULTILINE)
THREADS_VARIABLE_FORM = re.compile(r"^[A-Z_][A-Z0-9_]*$")
# 登记行的 red-blocks=：故障注入之后的红块由用例自己认下，门禁起用例按退出码判（文件头「崩溃枚举用例行」）
CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST = "judged-by-test"
# 登记行的 findings=：用例的环境守卫清掉了门禁给的发现日志变量，门禁给的那份发现日志空着不判（文件头「崩溃枚举用例行」）
CRASH_CASE_FINDINGS_NOT_WRITTEN = "not-written-by-the-case"
# 登记行的 blocks-line=：用例的枚举按设计 panic 在打块那一行之前，日志里没有那一行不判（文件头「崩溃枚举用例行」）
CRASH_CASE_BLOCKS_LINE_NOT_PRINTED = "not-printed-by-the-case"
# 缩崩溃域的展开深度变量：门禁 crash-point-replay 收尾写「缩过域、不算门禁通过」那一判（crash_point_replay_narrowing_variable）是同一份名单，两处一起改
CRASH_CASE_NARROWING_VARIABLES = ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO", "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")
CRASH_SHARE_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_SHARE"
CRASH_CASE_SHARE_FORM = re.compile(r"([0-9]+)/([1-9][0-9]*)")   # crash-case-command --share 的写法：<片号>/<片数>
CRASH_CASE_STAGES = ("record", "judge")   # crash-case-command --stage 认的值：record 只录、judge 只判（SINGLEFS_CRASH_AMPLIFICATION_STAGE 的值）
CRASH_STAGE_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_STAGE"
CRASH_HOST_WEIGHTS_VARIABLE = "SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS"
# 崩溃判定的跑法（crates/singlefs-verification/src/verdict_store.rs 的 CRASH_LIBRARY_RUN_KIND_ENVIRONMENT_VARIABLE）：命令里不设，调用方环境里的清掉
CRASH_LIBRARY_RUN_KIND_VARIABLE = "SINGLEFS_CRASH_LIBRARY_RUN_KIND"
CRASH_CASE_BLOCKS_PREFIX = "CRASH_CASE_BLOCKS "
CRASH_CASE_BLOCKS_FIELDS = ("blocks", "reused", "fresh", "red", "refused")
EXIT_BLOCKS_RED = 1
EXIT_BLOCKS_LINE_BAD = 4
CRASH_CASE_TEST_FORM = re.compile(r"^(?P<package>[A-Za-z0-9_-]+):(?P<target>[A-Za-z0-9_]+):(?P<function>[A-Za-z_][A-Za-z0-9_]*)$")
COUNT_LINE_PREFIX_FORM = re.compile(r"^[A-Z][A-Z0-9_]*$")
CRASH_POINT_REPLAY_PARALLEL_FINISHED_PREFIX = "CRASH_POINT_REPLAY_PARALLEL_FINISHED "
PASSED_ONE_TEST_FORM = re.compile(r"^test result: ok\. 1 passed; 0 failed; ")
POSITIVE_INTEGER_FORM = re.compile(r"[1-9][0-9]*")
CRASH_POINT_REPLAY_THREADS_VARIABLE = "SINGLEFS_CRASH_POINT_REPLAY_THREADS"
HEAVY_THREAD_DEFAULT_FILE = os.path.join(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))), ".claude", "heavy-thread-default")
THREAD_CAP_VARIABLE = "SINGLEFS_THREAD_CAP"
HEAVY_THREAD_COMMENT_OR_BLANK_FORM = re.compile(r"[ \t]*(#|$)")
HEAVY_THREAD_REGISTRATION_FORM = re.compile(r"[ \t]*([1-9][0-9]*)[ \t]+#[ \t]*[^ \t]")
DEFAULT_WORKER_THREAD_SOURCES = ("available_parallelism",)
IGNORE_ATTRIBUTE_FORM = re.compile(r"^#\s*\[\s*ignore\s*(?:\]|=)")
IGNORE_ATTRIBUTE_FORM_WITHOUT_SPACE = re.compile(r"^#\[\s*ignore\s*(?:\]|=)")  # 弄坏开关 ignore-attribute-without-space 换成不带空格的写法
ATTRIBUTE_START = re.compile(r"#!?\s*\[")
FUNCTION_QUALIFIER_AT_END = re.compile(r'(?:\bpub(?:\s*\([^()]*\))?|\basync|\bunsafe|\bconst|\bextern(?:\s*"[^"]*")?)$')
TEST_ATTRIBUTE_FORM = re.compile(r"#\s*\[\s*test\s*\]")
TEST_IGNORE_FORM = re.compile(r"#\s*\[\s*ignore\b")

# 构建环境的生效值（build_environment_lines；不进 compute_unit_key 的键，admission.py 文件头「单元键」）
CARGO_CONFIGURATION_FILE_NAMES = ("config", "config.toml")
TOOLCHAIN_FILE_NAMES = ("rust-toolchain", "rust-toolchain.toml")
BUILD_ENVIRONMENT_EXACT_VARIABLES = ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER")
# 编译档（CARGO_PROFILE_*、cargo 配置的 [profile.<名>]、增量编译开关）与线程数（jobs）不收：环境变量只认下面这几种
BUILD_ENVIRONMENT_VARIABLE_FORMS = (re.compile(r"^CARGO_BUILD_"), re.compile(r"^CARGO_TARGET_.+_RUSTFLAGS$"), re.compile(r"^CARGO_TARGET_.+_RUNNER$"))
BUILD_ENVIRONMENT_VARIABLES_LEFT_OUT = {"CARGO_BUILD_JOBS": "只定编译并行度（线程数）；scripts/hosts/capped.sh 与 mutate.sh 会设它",
                                        "CARGO_BUILD_TARGET_DIR": "只定编译目录放哪（路径）；task-server 认调用方给的这一个",
                                        "CARGO_BUILD_INCREMENTAL": "只定增量编译开不开（编译档）"}
# cargo 配置里收的那几张表与 build 表里不收的键（scripts/mutation/lib_mutation_units.py 的 CONFIGURATION_SECTIONS、BUILD_KEYS_LEFT_OUT 同一口径）：
# [profile.<名>] 不收，[build] 的 jobs（线程数）、target-dir（路径）、incremental（编译档）不收
BUILD_CONFIGURATION_SECTIONS = ("build", "target", "env")
BUILD_KEYS_LEFT_OUT = frozenset(("jobs", "target-dir", "incremental"))
BUILD_ENVIRONMENT_ALL_SETTINGS_BREAK = "build-environment-all-settings"
CONFIGURATION_ANCESTORS_UNREAD_BREAK = "cargo-configuration-ancestors-unread"
BUILD_ENVIRONMENT_PROGRAM_VARIABLES = ("RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER")
BUILD_ENVIRONMENT_PROGRAM_VARIABLE_FORMS = (re.compile(r"^CARGO_TARGET_.+_RUNNER$"),)
BUILD_ENVIRONMENT_COMPILER_VARIABLES = ("RUSTC", "CARGO_BUILD_RUSTC")
CONFIGURATION_PROGRAM_BUILD_KEYS = ("rustc-wrapper", "rustc-workspace-wrapper")

# 复跑表的行键（replay_row_request）
REPLAY_RESEARCH_DIRECTORY = "scripts/experiment-apparatus"
REPLAY_JUDGING_FUNCTIONS = ("replay_judge_fresh_product", "strip_admission_header", "strip_timing", "strip_load_fields", "timing_load_reading_raw", "timing_load_decide")
REPLAY_GPU_FEATURE_VARIABLE = "CHECKER_TIER_FEATURES"
REPLAY_SHELL_WORD = re.compile(r"[A-Za-z_][A-Za-z0-9_-]*")
REPLAY_PACKAGE_MENTION = re.compile(r"(?:^|\s)-p\s+([A-Za-z0-9_][A-Za-z0-9_-]*)")
REPLAY_PACKAGE_MENTION_DASHES_UNDER_BREAK = re.compile(r"(?:^|\s)-p\s+([A-Za-z0-9_-]+)")
REPLAY_BIN_MENTION = re.compile(r"--bin\s+([A-Za-z0-9_-]+)")
REPLAY_BIN_VARIABLE_MENTION = re.compile(r'--bin\s+"?\$')
REPLAY_FUNCTION_HEADER = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)\(\)\s*\{", re.M)
REPLAY_DEVICE_KEY_MENTION = re.compile(r"--package-directory\s+(\S+)\s+--binary\s+([A-Za-z0-9_-]+)")
VERIFICATION_PACKAGE = "singlefs-verification"


class RegistrationError(Exception):
    """登记表或调用写错了：退 2，出路是改登记表那一行或改调用。"""


class InputManifestError(Exception):
    """算不出一样输入：探针退非 0、读文件出错、工具链起不来。"""


class RegistrationRow:
    def __init__(self, key, entries, conditions, line_number, comment):
        self.key = key
        self.entries = entries
        self.conditions = conditions
        self.line_number = line_number
        self.comment = comment


# ── 登记表 ────────────────────────────────────────────────────────────────────

def read_registration_rows(root):
    """整张表的行；表不在返回空表。第一列是键，第二列是入口（- 是没有），第三列起到 # 之前是条件。"""
    table_path = os.path.join(root, REGISTRATION_TABLE)
    if not os.path.isfile(table_path):
        return []
    rows = []
    with open(table_path, encoding="utf-8", errors="surrogateescape") as handle:
        for line_number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.startswith("#"):
                continue
            columns = line.split("\t")
            conditions = []
            comment = ""
            for column in columns[2:]:
                if column.lstrip().startswith("#"):
                    comment = column.strip().lstrip("#").strip()
                    break
                conditions.extend(column.split())
            entries = [entry for entry in (columns[1].split() if len(columns) > 1 else []) if entry != "-"]
            rows.append(RegistrationRow(columns[0], entries, conditions, line_number, comment))
    return rows


def rows_of_key(rows, key):
    return [row for row in rows if row.key == key]


def resolve_entries(rows, key, referencing_keys=()):
    """这个键登记的全部入口；@<别的键> 展开成那个键的入口。"""
    own_rows = rows_of_key(rows, key)
    if not own_rows:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {key} 这一行")
    resolved = []
    for row in own_rows:
        for entry in row.entries:
            if not entry.startswith("@"):
                resolved.append(entry)
                continue
            referenced_key = entry[1:]
            if referenced_key == key or referenced_key in referencing_keys:
                raise RegistrationError(f"{REGISTRATION_TABLE} 第 {row.line_number} 行：{key} 的 @{referenced_key} 引用绕成了圈")
            resolved.extend(resolve_entries(rows, referenced_key, referencing_keys + (key,)))
    return list(dict.fromkeys(resolved))


def resolve_input_paths(rows, key):
    """门禁 crates-mutation-replay 的拷贝范围（第三列 copy-range=）：返回 (路径, 贡献了它的行)。不是任何判定的键：只答拷哪些文件进每片的源码副本。"""
    own_rows = rows_of_key(rows, key)
    if not own_rows:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里没有 {key} 这一行")
    paths = [token[len("copy-range="):] for row in own_rows for token in row.conditions if token.startswith("copy-range=")]
    if not paths:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里 {key} 那一行第三列没有 copy-range=（拷贝范围）")
    return paths, own_rows


def entry_file(entry):
    """入口所在的文件（仓根起）；tests-of: / vm: 这一类没有文件交 None。"""
    if entry.startswith(("tests-of:", "vm:", "@")):
        return None
    return entry.split("::", 1)[0]


def experiment_condition_file_problems(root, kind, parts):
    """实验行一条前提点名的文件不在仓里的说明（文件头「准入条件」）：question-row 那份问题单，product-field 是 replay.sh 给那个实验键登记的留存产物；
    same-run-field 读同一趟的输出、不点名仓里的文件。复跑表按行的准入键那一段写错抛 RegistrationError。"""
    if kind == "question-row":
        if not os.path.isfile(os.path.join(root, parts["path"])):
            return [f"点名的问题单 {parts['path']} 不在仓里（前提指的东西可以被归档删掉，登记就靠不住；上一段的结论改用 same-run-field= 在同一趟里现判）"]
        return []
    if kind == "product-field":
        products = replay_products_of(root, parts["key"])
        if not products:
            return [f"点名的实验键 {parts['key']} 在 {REPLAY_SCRIPT} 的复跑表里一行留存产物都没有"]
        return [f"点名的留存产物 {product} 不在仓里（research/ 可以整个删掉；上一段的结论改用 same-run-field= 在同一趟里现判）"
                for product in products if not os.path.isfile(os.path.join(root, product))]
    return []


def parse_gate_condition(token):
    """门禁行第三列的一条，切成 (种类, 各段)：command / readwrite 是 {"value"}，probe / environment 是 {"script", "arguments"}。
    认不出、脚本写成绝对路径或带 .. 都是登记错。"""
    match = GATE_CONDITION_FORM.match(token)
    if not match:
        raise RegistrationError(f"认不出的门禁准入条件：{token}（门禁行只认 command=<可执行名>、readwrite=<路径>、"
                                "probe=<仓内脚本>[:<参数>…]、environment=<仓内脚本>[:<参数>…]、copy-range=<路径>）")
    kind, value = match.group("kind"), match.group("value")
    if kind in ("probe", "environment"):
        script, *arguments = value.split(":")
        if not script or script.startswith("/") or ".." in script.split("/"):
            raise RegistrationError(f"门禁准入条件 {token} 的脚本要写成仓库根起的相对路径（不许绝对路径、不许带 ..）")
        return kind, {"script": script, "arguments": arguments}
    return kind, {"value": value}



# ── 函数单元键（lib_function_units.py 算闭包；这里拼工具链、依赖版本、判法单元与这条判定自己的内容输入）──────────

FUNCTION_UNITS_MODULE = {}


def function_units():
    """同目录的 lib_function_units.py，只载一次；找不到、载不进抛 UnitKeyError（调用方按「拒算」处置：这一条现判、不复用）。"""
    if "module" not in FUNCTION_UNITS_MODULE:
        path = os.path.join(os.path.dirname(os.path.realpath(__file__)), FUNCTION_UNITS_LIBRARY)
        if not os.path.isfile(path):
            raise UnitKeyError(f"找不到函数单元库 {path}（它与这一份同目录，随仓走）")
        specification = importlib.util.spec_from_file_location("singlefs_function_units", path)
        module = importlib.util.module_from_spec(specification)
        try:
            specification.loader.exec_module(module)
        except Exception as error:  # 载不进就算不出键：交给调用方按拒算处置，不当「没变」
            raise UnitKeyError(f"载不进函数单元库 {path}：{type(error).__name__}：{error}") from error
        FUNCTION_UNITS_MODULE["module"] = module
    return FUNCTION_UNITS_MODULE["module"]


class UnitKeyError(Exception):
    """算不出一把单元键（库载不进、入口找不到、工具链起不来）：这一条判定按拒算处置，红句与 --list-items 报出来。"""


def break_is_set(switch_name):
    return switch_name in os.environ.get(BREAK_VARIABLE, "").split(",")


def say(message):
    print(message, file=sys.stderr)



def parse_condition(token):
    """把一条准入条件切成 (种类, 各段)；认不出、正则编不过都是登记错。"""
    question_row = QUESTION_ROW_FORM.match(token)
    if question_row:
        try:
            re.compile(question_row.group("pattern"))
        except re.error as error:
            raise RegistrationError(f"准入条件 {token} 的正则编不过：{error}") from error
        return ("question-row", question_row.groupdict())
    product_field = PRODUCT_FIELD_FORM.match(token)
    if product_field:
        return ("product-field", product_field.groupdict())
    same_run_field = SAME_RUN_FIELD_FORM.match(token)
    if same_run_field:
        return ("same-run-field", same_run_field.groupdict())
    raise RegistrationError(f"认不出的准入条件：{token}（只认 question-row=<文件>#<行号>:<正则>、product-field=<实验键>:<行名>:<字段>=<值> 与 same-run-field=<行名>:<字段>=<值>）")


def header_fields(header_line):
    """E7INPUT 行的字段；reason= 取到行尾（理由里可以有空格）。"""
    body = header_line[len(HEADER_PREFIX):]
    reason_start = body.find(" reason=")
    reason = None
    if reason_start >= 0:
        reason = body[reason_start + len(" reason="):]
        body = body[:reason_start]
    fields = dict(token.split("=", 1) for token in body.split() if "=" in token)
    if reason is not None:
        fields["reason"] = reason
    return fields


def header_lines_of(product_path):
    header_lines = []
    try:
        with open(product_path, "rb") as handle:
            for _line_index in range(HEADER_LINES_READ_AT_MOST):
                raw_line = handle.readline()
                if not raw_line:
                    break
                text = raw_line.decode("utf-8", "replace").rstrip("\n")
                if not text.startswith(HEADER_PREFIX):
                    break
                header_lines.append(text)
    except OSError:
        return []
    return header_lines


class StoredFingerprint:
    def __init__(self, product_path, fingerprint, modified_at):
        self.product_path = product_path
        self.fingerprint = fingerprint
        self.modified_at = modified_at


def stored_fingerprints(root, key):
    """research/results/ 下（含子目录）头上记着这个键的输入指纹的每一份产物。"""
    found = []
    results_directory = os.path.join(root, RESULTS_DIRECTORY)
    for directory, _subdirectories, file_names in os.walk(results_directory):
        for file_name in sorted(file_names):
            product_path = os.path.join(directory, file_name)
            for header_line in header_lines_of(product_path):
                fields = header_fields(header_line)
                if fields.get("name") == "input_fingerprint" and fields.get("key") == key and fields.get("sha256"):
                    found.append(StoredFingerprint(os.path.relpath(product_path, root), fields["sha256"], os.path.getmtime(product_path)))
                    break
    return found


class ReplayRow:
    """复跑表的一行连同它的准入键：identity 是前三列原样（实验号|二进制|参数），product 是第四列，explicit 是按行的准入键那一段指明过它。"""

    def __init__(self, experiment, identity, product, key, header_writer, explicit):
        self.experiment = experiment
        self.identity = identity
        self.product = product
        self.key = key
        self.header_writer = header_writer
        self.explicit = explicit


def replay_row_key_entries(text):
    """replay.sh 里按行的准入键那一段：返回 ([(行号, 键, 前三列, 产物头谁写)], 写法错的说明)。没有这一段返回两个空表。"""
    entries, problems = [], []
    if REPLAY_ROW_KEYS_START not in text:
        return entries, problems
    if text.count(REPLAY_ROW_KEYS_START) != 1:
        return entries, [f"{REPLAY_SCRIPT} 里按行的准入键那一段（{REPLAY_ROW_KEYS_START.strip()}）有 {text.count(REPLAY_ROW_KEYS_START)} 处，只许一处"]
    start = text.index(REPLAY_ROW_KEYS_START) + len(REPLAY_ROW_KEYS_START)
    first_line_number = text[:start].count("\n") + 1
    for offset, line in enumerate(text[start:].split("\n")):
        if line == REPLAY_ROW_KEYS_END_LINE:
            break
        if not line.strip():
            continue
        where = f"{REPLAY_SCRIPT} 第 {first_line_number + offset} 行（按行的准入键）"
        columns = line.split("\t")
        if len(columns) != 3:
            problems.append(f"{where}要三列、制表符分隔（键、复跑表那一行的前三列、产物头谁写），实际 {len(columns)} 列")
            continue
        key, identity, header_writer = columns
        identity_parts = identity.split("|")
        if not EXPERIMENT_KEY_FORM.match(key):
            problems.append(f"{where}的键 {key} 不是实验键的写法（E<号> 或 E<号>/<方式>，方式只用小写字母、数字与连字符）")
        elif len(identity_parts) != 3:
            problems.append(f"{where}第二列 {identity} 要写成复跑表那一行的前三列「实验号|二进制|参数」，实际 {len(identity_parts)} 段")
        elif key.split("/")[0] != identity_parts[0]:
            problems.append(f"{where}的键 {key} 与它指的那一行的实验号 {identity_parts[0]} 不同")
        if header_writer not in REPLAY_ROW_HEADER_WRITERS:
            problems.append(f"{where}第三列 {header_writer} 认不出，产物头谁写只认 {'、'.join(REPLAY_ROW_HEADER_WRITERS)}")
        entries.append((first_line_number + offset, key, identity, header_writer))
    else:
        problems.append(f"{REPLAY_SCRIPT} 里按行的准入键那一段没有收尾的 {REPLAY_ROW_KEYS_END_LINE} 行")
    return entries, problems


def replay_rows_with_keys(root):
    """scripts/experiment-apparatus/replay.sh 复跑表的每一行连同它的准入键：返回 ([ReplayRow], 按行的准入键那一段写错的说明)。
    没被那一段指明的行，键是实验号、产物头由装置写。弄坏开关 ignore-row-keys 下那一段整个不认。"""
    replay_path = os.path.join(root, REPLAY_SCRIPT)
    if not os.path.isfile(replay_path):
        return [], []
    with open(replay_path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    table = []
    for line in text.split("\n"):
        match = REPLAY_ROW_FORM.match(line)
        if match:
            table.append((match.group(1), "|".join(match.group(1, 2, 3)), match.group(4)))
    entries, problems = ([], []) if break_is_set("ignore-row-keys") else replay_row_key_entries(text)
    chosen = {}
    for line_number, key, identity, header_writer in entries:
        hits = sum(1 for _experiment, row_identity, _product in table if row_identity == identity)
        if hits != 1:
            problems.append(f"{REPLAY_SCRIPT} 第 {line_number} 行给 {key} 指的「{identity}」在复跑表里有 {hits} 行，要恰好一行")
        elif identity in chosen:
            problems.append(f"{REPLAY_SCRIPT} 第 {line_number} 行又给「{identity}」指明了一次键（{key}），同一行只许指明一次")
        else:
            chosen[identity] = (key, header_writer)
    rows = []
    for experiment, identity, product in table:
        key, header_writer = chosen.get(identity, (experiment, "device"))
        rows.append(ReplayRow(experiment, identity, product, key, header_writer, identity in chosen))
    return rows, problems


def replay_products_of(root, experiment_key):
    """scripts/experiment-apparatus/replay.sh 登记表里准入键是这个键的那几行的留存产物（仓库根起的路径）；按行的准入键那一段写错抛 RegistrationError。"""
    rows, problems = replay_rows_with_keys(root)
    if problems:
        raise RegistrationError("；".join(problems))
    return [os.path.join(RESULTS_DIRECTORY, row.product) for row in rows if row.key == experiment_key]


def markdown_row_status(root, path, row_label):
    """那份文件里第一格是 row_label 的第一行表格行的最后一格；文件或行不在返回 None 与原因。"""
    full_path = os.path.join(root, path)
    if not os.path.isfile(full_path):
        return None, f"{path} 不在"
    with open(full_path, encoding="utf-8", errors="replace") as handle:
        for line in handle.read().split("\n"):
            stripped = line.strip()
            if not stripped.startswith("|"):
                continue
            cells = [cell.strip() for cell in re.split(r"(?<!\\)\|", stripped)[1:-1]]
            if cells and cells[0] == row_label:
                return cells[-1], None
    return None, f"{path} 里没有第一格是 {row_label} 的表格行"


def product_field_value(root, experiment_key, line_name, field):
    """replay.sh 登记给这个实验键的产物里，最后一行 `E7RESULT name=<行名>` 的字段值；取不到返回 None 与原因。"""
    products = replay_products_of(root, experiment_key)
    if len(products) != 1:
        raise RegistrationError(f"product-field 要 replay.sh 登记表里 {experiment_key} 恰好一行，实际 {len(products)} 行")
    product = products[0]
    full_path = os.path.join(root, product)
    if not os.path.isfile(full_path):
        return None, product, f"{product} 不在树里"
    last_line = None
    with open(full_path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith(f"E7RESULT name={line_name} ") or line.rstrip("\n") == f"E7RESULT name={line_name}":
                last_line = line.rstrip("\n")
    if last_line is None:
        return None, product, f"{product} 里没有 name={line_name} 这一行"
    fields = dict(token.split("=", 1) for token in last_line.split()[1:] if "=" in token)
    if field not in fields:
        return None, product, f"{product} 的 name={line_name} 行没有 {field} 字段"
    return fields[field], product, None


# same-run-field=<行名>:<字段>=<值>（文件头「准入条件」）：同一趟里先跑的那一段这一趟的输出，路径由调用方经这个环境变量交来；不读留存产物
SAME_RUN_FIELD_FORM = re.compile(r"^same-run-field=(?P<line_name>[^:]+):(?P<field>[^=]+)=(?P<value>\S+)$")
SAME_RUN_PRODUCT_VARIABLE = "SINGLEFS_ADMISSION_SAME_RUN_PRODUCT"


def last_result_line_field(path, line_name, field):
    """那份输出里最后一行 `E7RESULT name=<行名>` 的字段值；取不到交 (None, 原因)。"""
    last_line = None
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            if line.startswith(f"E7RESULT name={line_name} ") or line.rstrip("\n") == f"E7RESULT name={line_name}":
                last_line = line.rstrip("\n")
    if last_line is None:
        return None, f"{path} 里没有 name={line_name} 这一行"
    fields = dict(token.split("=", 1) for token in last_line.split()[1:] if "=" in token)
    if field not in fields:
        return None, f"{path} 的 name={line_name} 行没有 {field} 字段"
    return fields[field], None


def same_run_field_value(line_name, field):
    """same-run-field 的取值：SAME_RUN_PRODUCT_VARIABLE 指的那份同一趟的输出里最后一行 name=<行名> 的字段；取不到交 (None, 那份输出, 原因)。"""
    product = os.environ.get(SAME_RUN_PRODUCT_VARIABLE, "")
    if not product:
        return None, "", (f"{SAME_RUN_PRODUCT_VARIABLE} 没设：这一段要在同一趟里接在前一段之后跑，"
                          f"先跑前一段、把它这一趟的输出路径经 {SAME_RUN_PRODUCT_VARIABLE} 交给这一段（不读 research/results/ 下的留存产物）")
    if not os.path.isfile(product):
        return None, product, f"{SAME_RUN_PRODUCT_VARIABLE} 指的同一趟输出 {product} 不在"
    value, problem = last_result_line_field(product, line_name, field)
    return value, product, problem


def judge_preconditions(root, key, conditions):
    """② 前提：返回 (没齐的清单, 齐了的清单)，每项是一句说明。登记写错抛 RegistrationError。
    same-run-field 读调用方交来的同一趟输出（弄坏开关 same-run-field-unchecked 下不读、一律当齐了）。"""
    missing = []
    satisfied = []
    for token in conditions:
        kind, parts = parse_condition(token)
        if kind == "same-run-field":
            if break_is_set("same-run-field-unchecked"):
                satisfied.append(f"（弄坏开关 same-run-field-unchecked：没读同一趟的输出）{token}")
                continue
            value, product, problem = same_run_field_value(parts["line_name"], parts["field"])
            if problem is not None:
                missing.append(f"{problem}（要 name={parts['line_name']} 的 {parts['field']}={parts['value']}）")
            elif value == parts["value"]:
                satisfied.append(f"同一趟的输出 {product} 的 name={parts['line_name']} {parts['field']}={value}")
            else:
                missing.append(f"同一趟的输出 {product} 的 name={parts['line_name']} {parts['field']}={value}，要 {parts['value']}")
            continue
        if kind == "question-row":
            status, problem = markdown_row_status(root, parts["path"], parts["row"])
            if problem is not None:
                raise RegistrationError(f"{key} 的前提 {token} 读不到：{problem}")
            if re.search(parts["pattern"], status):
                satisfied.append(f"{parts['path']} 第 {parts['row']} 行的状态「{status}」匹配「{parts['pattern']}」")
            else:
                missing.append(f"{parts['path']} 第 {parts['row']} 行的状态是「{status}」，要匹配「{parts['pattern']}」")
        elif kind == "product-field":
            value, product, problem = product_field_value(root, parts["key"], parts["line_name"], parts["field"])
            if problem is not None:
                missing.append(f"{problem}（要 name={parts['line_name']} 的 {parts['field']}={parts['value']}）")
            elif value == parts["value"]:
                satisfied.append(f"{product} 的 name={parts['line_name']} {parts['field']}={value}")
            else:
                missing.append(f"{product} 的 name={parts['line_name']} {parts['field']}={value}，要 {parts['value']}")
    return missing, satisfied



# run_probe 起的探针脚本都是登记表第三列 probe= 与 environment= 点名的（lib_function_units.py 文件头「登记表点名的程序」）：
# 收 run_probe 的闭包时照这张表认出它起的是环境探针，不连、不拒算（前提由 gate-preconditions 每趟现判；探针打的工具版本不进任何判定的键）
REGISTRY_LISTED_PROGRAMS = {"run_probe": (REGISTRATION_TABLE, r"(?:^|\s)(?:probe|environment)=([^\s:]+)")}


def run_probe(root, parts):
    """bash <根>/<脚本> <参数…>，工作目录是根；返回 (退出码, stdout 字节, stderr 字节)。起不来、超时都算退 127。"""
    command = ["bash", os.path.join(root, parts["script"]), *parts["arguments"]]
    try:
        completed = subprocess.run(command, cwd=root, capture_output=True, timeout=PROBE_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return 127, b"", str(error).encode("utf-8", "replace")
    return completed.returncode, completed.stdout, completed.stderr


def probe_output_tail(output, errors):
    """探针输出的最后三行（stdout 与 stderr 接起来），给没齐的那一条当明细。"""
    lines = [line.strip() for line in (output + b"\n" + errors).decode("utf-8", "replace").split("\n") if line.strip()]
    return "｜".join(lines[-3:]) if lines else "一个字都没打"


def gate_conditions_of(rows, wanted_kinds):
    """这几行登记的门禁条件里种类在 wanted_kinds 之中的，[(条件原文, 种类, 各段)]；写错了抛 RegistrationError。"""
    found = []
    for row in rows:
        for token in row.conditions:
            kind, parts = parse_gate_condition(token)
            if kind in wanted_kinds:
                found.append((token, kind, parts))
    return found


def judge_gate_preconditions(root, rows):
    """门禁的前提逐条判：返回没齐的清单，每项一句说明。"""
    missing = []
    for token, kind, parts in gate_conditions_of(rows, GATE_PRECONDITION_KINDS):
        if kind == "command":
            if shutil.which(parts["value"]) is None:
                missing.append(f"{parts['value']} 不在 PATH 里（{token}）")
        elif kind == "readwrite":
            path = parts["value"] if os.path.isabs(parts["value"]) else os.path.join(root, parts["value"])
            if not os.access(path, os.R_OK | os.W_OK):
                missing.append(f"{parts['value']} 不在，或当前用户不可读写（{token}）")
        else:
            exit_code, output, errors = run_probe(root, parts)
            if exit_code != 0:
                missing.append(f"bash {' '.join([parts['script'], *parts['arguments']])} 退 {exit_code}（{token}）：{probe_output_tail(output, errors)}")
    return missing


def git_output(root, *git_arguments):
    completed = subprocess.run(["git", "-C", root, *git_arguments], capture_output=True, text=True, errors="replace")
    return completed.returncode, completed.stdout


class CrashCase:
    def __init__(self, row, package, target, function, count_lines, exhaustive_lines, thread_lines,
                 thread_variable=None, features=(), red_blocks_judged_by_test=False, findings_not_written_by_the_case=False,
                 blocks_line_not_printed_by_the_case=False, shards_at_least=0):
        self.row = row
        # 登记行的 shards-at-least=：经服务分片时至少分几片（没登记是 0）
        self.shards_at_least = shards_at_least
        # 登记行的 red-blocks=judged-by-test：故障注入之后的红块由用例自己认下，门禁起用例按退出码判、不按库里的红块判
        self.red_blocks_judged_by_test = red_blocks_judged_by_test
        # 登记行的 findings=not-written-by-the-case：门禁给的那份发现日志空着不判
        self.findings_not_written_by_the_case = findings_not_written_by_the_case
        # 登记行的 blocks-line=not-printed-by-the-case：日志里没有块那一行不判
        self.blocks_line_not_printed_by_the_case = blocks_line_not_printed_by_the_case
        # 登记行的 separate-run=<理由>：拆出来单独验证，门禁不点名崩溃枚举用例的那一趟不跑它（没登记是空串）。parse_crash_case 构造之后填：
        # 构造函数头的缺省值是 import 时执行的单元，进每一份载入本模块的闭包（文件头 build-environment-lines 那一条），这里不加参数
        self.separate_run_reason = ""
        # 登记行的 features=：用例要编进的 cargo 特性（没登记是空）；crash-case-command 在 -p <包> 之后带 --features
        self.features = tuple(features)
        # 用例读线程数的环境变量（登记行的 threads-variable=，没登记是 SINGLEFS_CRASH_POINT_REPLAY_THREADS）：crash-case-command 设它、判日志的记录行里记它
        self.thread_variable = thread_variable or CRASH_POINT_REPLAY_THREADS_VARIABLE
        self.key = row.key
        self.name = row.key[len(CRASH_CASE_KEY_PREFIX):]
        self.package = package
        self.target = target
        self.function = function
        self.count_lines = count_lines
        self.exhaustive_lines = exhaustive_lines
        self.thread_lines = thread_lines

    def test_text(self):
        return f"{self.package}:{self.target}:{self.function}"


def parse_crash_case(row):
    """崩溃枚举用例行的第三列：写法认不出、test= 不是恰好一条、exhaustive= / threads= 点名的前缀没登记成 count-line=、
    threads-variable= 不是环境变量名或多于一条、features= 写法不对或多于一条、
    red-blocks= 不是 judged-by-test 或多于一条，findings= 不是 not-written-by-the-case 或多于一条，blocks-line= 不是 not-printed-by-the-case 或多于一条，
    separate-run= 的理由不到 8 个字或多于一条，都抛 RegistrationError。
    separate-run= 在这个函数里认、不进 CRASH_CASE_CONDITION_FORM：模块级的赋值是 import 时执行的单元，进每一份载入本模块的闭包（文件头 build-environment-lines 那一条）。"""
    tests, count_lines, exhaustive_lines, thread_lines, thread_variables, feature_values = [], [], [], [], [], []
    red_blocks_values, findings_values, blocks_line_values, shards_at_least_values, separate_run_reasons = [], [], [], [], []
    separate_run_prefix = "separate-run="
    separate_run_reason_minimum_characters = 8
    for token in row.conditions:
        if token.startswith(separate_run_prefix):
            reason = token[len(separate_run_prefix):]
            if len(reason) < separate_run_reason_minimum_characters:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：separate-run= 的理由至少 {separate_run_reason_minimum_characters} 个字"
                                        "（写为什么拆出来单独验证、什么时候并回，不带空白）")
            separate_run_reasons.append(reason)
            continue
        match = CRASH_CASE_CONDITION_FORM.match(token)
        if not match:
            raise RegistrationError(f"崩溃枚举用例 {row.key} 的第三列认不出 {token}（只认 test=<包>:<测试目标>:<用例函数>、"
                                    f"count-line=<前缀>、exhaustive=<前缀>、threads=<前缀>、threads-variable=<环境变量名>、features=<特性,特性…>、"
                                    f"red-blocks={CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}、"
                                    f"findings={CRASH_CASE_FINDINGS_NOT_WRITTEN}、blocks-line={CRASH_CASE_BLOCKS_LINE_NOT_PRINTED}、shards-at-least=<正整数>、"
                                    f"{separate_run_prefix}<理由，不带空白、至少 {separate_run_reason_minimum_characters} 个字>）")
        kind, value = match.group("kind"), match.group("value")
        if kind == "shards-at-least":
            if not CRASH_CASE_SHARDS_AT_LEAST_FORM.match(value):
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：shards-at-least= 要写正整数（不带前导 0）")
            shards_at_least_values.append(int(value))
            continue
        if kind == "findings":
            if value != CRASH_CASE_FINDINGS_NOT_WRITTEN:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：findings= 只认 {CRASH_CASE_FINDINGS_NOT_WRITTEN}"
                                        "（用例的环境守卫清掉了门禁给的发现日志变量，门禁给的那份发现日志空着不判）")
            findings_values.append(value)
            continue
        if kind == "blocks-line":
            if value != CRASH_CASE_BLOCKS_LINE_NOT_PRINTED:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：blocks-line= 只认 {CRASH_CASE_BLOCKS_LINE_NOT_PRINTED}"
                                        "（用例的枚举按设计 panic 在打块那一行之前，日志里没有那一行不判）")
            blocks_line_values.append(value)
            continue
        if kind == "red-blocks":
            if value != CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：red-blocks= 只认 {CRASH_CASE_RED_BLOCKS_JUDGED_BY_TEST}"
                                        "（故障注入之后的红块由用例自己认下，门禁起用例按退出码判）")
            red_blocks_values.append(value)
            continue
        if kind == "test":
            test = CRASH_CASE_TEST_FORM.match(value)
            if not test:
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token} 要写成 test=<包>:<测试目标>:<用例函数>（字母、数字、_，包名另许 -）")
            tests.append(test)
            continue
        if kind == "features":
            if not CRASH_CASE_FEATURES_FORM.match(value) or len(set(value.split(","))) != len(value.split(",")):
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：features= 要写逗号隔开的 cargo 特性名（小写字母、数字、_、-，不重复）")
            feature_values.append(value)
            continue
        if kind == "threads-variable":
            if not THREADS_VARIABLE_FORM.match(value):
                raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：threads-variable= 要写用例读线程数的环境变量名（大写字母、数字、_）")
            thread_variables.append(value)
            continue
        if not COUNT_LINE_PREFIX_FORM.match(value):
            raise RegistrationError(f"崩溃枚举用例 {row.key} 的 {token}：前缀只许大写字母、数字、_，以字母开头")
        {"count-line": count_lines, "exhaustive": exhaustive_lines, "threads": thread_lines}[kind].append(value)
    if len(tests) != 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 要恰好一条 test=，实际 {len(tests)} 条")
    if len(thread_variables) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 threads-variable= 至多一条，实际 {len(thread_variables)} 条")
    if len(feature_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 features= 至多一条，实际 {len(feature_values)} 条")
    if len(red_blocks_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 red-blocks= 至多一条，实际 {len(red_blocks_values)} 条")
    if len(findings_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 findings= 至多一条，实际 {len(findings_values)} 条")
    if len(blocks_line_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 blocks-line= 至多一条，实际 {len(blocks_line_values)} 条")
    if len(shards_at_least_values) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 shards-at-least= 至多一条，实际 {len(shards_at_least_values)} 条")
    if len(separate_run_reasons) > 1:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 separate-run= 至多一条，实际 {len(separate_run_reasons)} 条")
    if len(set(count_lines)) != len(count_lines):
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 count-line= 有重复的前缀")
    unregistered = [prefix for prefix in exhaustive_lines + thread_lines if prefix not in count_lines]
    if unregistered:
        raise RegistrationError(f"崩溃枚举用例 {row.key} 的 exhaustive= / threads= 点名的 {' '.join(unregistered)} 没有登记成 count-line=")
    test = tests[0]
    thread_variable = None if break_is_set("thread-variable-ignored") else (thread_variables[0] if thread_variables else None)
    case = CrashCase(row, test.group("package"), test.group("target"), test.group("function"),
                     count_lines, exhaustive_lines, thread_lines, thread_variable,
                     feature_values[0].split(",") if feature_values else (), bool(red_blocks_values), bool(findings_values), bool(blocks_line_values),
                     shards_at_least_values[0] if shards_at_least_values and not break_is_set("shards-at-least-ignored") else 0)
    case.separate_run_reason = separate_run_reasons[0] if separate_run_reasons else ""
    return case


def crash_cases_of(rows):
    """登记表里全部崩溃枚举用例，照登记表的次序；一行写错抛 RegistrationError。"""
    return [parse_crash_case(row) for row in rows if CRASH_CASE_KEY_FORM.match(row.key)]


def crash_case_of_key(rows, key):
    own_rows = rows_of_key(rows, key)
    if not CRASH_CASE_KEY_FORM.match(key) or len(own_rows) != 1:
        raise RegistrationError(f"{REGISTRATION_TABLE} 里崩溃枚举用例 {key} 要恰好一行，实际 {len(own_rows)} 行")
    return parse_crash_case(own_rows[0])


def git_common_directory(root):
    return_code, output = git_output(root, "rev-parse", "--path-format=absolute", "--git-common-dir")
    common_directory = output.strip() if return_code == 0 else ""
    return common_directory or None

def fields_of_line(line):
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


def machine_core_count(environment):
    """本机核数：os.cpu_count() 与这个进程的 CPU 亲和（os.sched_getaffinity）取小，不认 OMP_NUM_THREADS 这一类环境变量。
    读不到抛 InputManifestError。弄坏开关 cores-from-nproc 下照旧取 nproc（在 environment 的 PATH 里找，它认 OMP_NUM_THREADS）。"""
    if break_is_set("cores-from-nproc"):
        nproc = shutil.which("nproc", path=environment.get("PATH")) or "nproc"
        try:
            completed = subprocess.run([nproc], capture_output=True, text=True, errors="replace", env=dict(environment))
        except OSError as error:
            raise InputManifestError(f"nproc 起不来：{error}") from error
        cores_text = completed.stdout.strip()
        if completed.returncode != 0 or not POSITIVE_INTEGER_FORM.fullmatch(cores_text):
            raise InputManifestError(f"nproc 退 {completed.returncode}、打了「{cores_text}」，不是正整数")
        return int(cores_text)
    counts = [count for count in (os.cpu_count(), len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else None) if count]
    if not counts:
        raise InputManifestError("os.cpu_count() 与 CPU 亲和都读不到本机核数")
    return min(counts)


# 与 scripts/hosts/heavy_threads.py 的 default_heavy_threads 同一个判法、在这里另写一份（这一份不 import scripts/hosts 下的库）。
# 两份由 --selftest 的「每台默认重活线程数：与 heavy_threads.py 对拍」逐格比
def registered_heavy_threads(environment, registration_file=None):
    """每台默认重活线程数：environment 里 SINGLEFS_THREAD_CAP 设了（非空）照它（要是正整数），否则读 registration_file（不给是 HEAVY_THREAD_DEFAULT_FILE）
    第一条非注释行「<正整数><空白>#<理由>」的那个数。取不了抛 InputManifestError（原因带出路），不退回按核数算。"""
    cap = environment.get(THREAD_CAP_VARIABLE, "")
    if cap:
        if not POSITIVE_INTEGER_FORM.fullmatch(cap):
            raise InputManifestError(f"{THREAD_CAP_VARIABLE}={cap} 不是正整数，取不了每台默认重活线程数（设成正整数，或 unset 它取 .claude/heavy-thread-default 的登记值）")
        return int(cap)
    path = registration_file or HEAVY_THREAD_DEFAULT_FILE
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            lines = handle.read().split("\n")
    except OSError as error:
        raise InputManifestError(f"每台默认重活线程数的登记文件 {path} 读不了：{error}（在仓根建 .claude/heavy-thread-default，"
                                 "第一条非注释行写「31  # <理由>」）") from error
    if lines and lines[-1] == "":
        lines.pop()
    for number, line in enumerate(lines, 1):
        if HEAVY_THREAD_COMMENT_OR_BLANK_FORM.match(line):
            continue
        found = HEAVY_THREAD_REGISTRATION_FORM.match(line)
        if found:
            return int(found.group(1))
        raise InputManifestError(f"{path} 第 {number} 行「{line}」不是「<正整数>  # <理由>」（数是正整数，数与 # 之间至少一个空格，# 后面写理由）")
    raise InputManifestError(f"{path} 里没有非注释行，读不出每台默认重活线程数（第一条非注释行写「31  # <理由>」）")


def multi_host_configuration_file(root, environment):
    """多机配置在哪（同 scripts/crash/crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file）：
    调用方环境里 SINGLEFS_MULTI_HOST_CONFIG 设了照它；没设取 <root> 的 git common-dir 的上一层（主工作树的根）下的 multi-host.env，取不到 common-dir 退回 <root>/multi-host.env。
    文件在不在不判。"""
    configured = environment.get(MULTI_HOST_CONFIGURATION_VARIABLE)
    if configured:
        return configured
    common_directory = git_common_directory(root)
    if common_directory is not None:
        return os.path.join(os.path.dirname(common_directory), MULTI_HOST_CONFIGURATION_NAME)
    return os.path.join(root, MULTI_HOST_CONFIGURATION_NAME)


def configuration_value(path, key):
    """配置文件里 <键>= 开头的第一行的值（去掉两头成对的一层引号，同 crash_amplification_configuration_value）；文件不在、键没写是空串，在而读不了抛 InputManifestError。"""
    if not os.path.isfile(path):
        return ""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise InputManifestError(f"多机配置 {path} 读不了（{error}），判不出 {key}")
    for line in lines:
        line = line.strip()
        if not line.startswith(key + "="):
            continue
        value = line[len(key) + 1:].strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        return value
    return ""


# test-plan 分进子进程算的那一段（文件头 test-plan「并行」）：fork 之前把共用的东西放这里，子进程照抄父进程的内存读它们
TEST_PLAN_SHARED = {}


REPLAY_STRIP_HEADER_COMMAND = "sed '/^E7INPUT /d'"


ENTRY_SCRIPT_NAME = "admission.py"   # 入口文件名：块文件里的 __file__ 是块自己的文件名，要认入口脚本名的地方用这个
