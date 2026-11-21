# lib_project_code_runs.py 的部分模块：自证：样本工作区与样本登记表、临时服务与参数读法与登记表的自证格、selftest 与它的开关轮。
# 入口 lib_project_code_runs.py 按 PROJECT_CODE_RUNS_PART_NAMES 的次序 import 这一份（它 import 引导那一份与 lib_project_code_runs_base.py、lib_project_code_runs_launchers.py，用到的名字写显式 import），不单独执行；
# 弄坏开关的名单与红的格登记在入口 lib_project_code_runs.py 里，开关表写在它的文件头。
import ast
import contextlib
import glob
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from lib_project_code_runs_base import (
    ACTIVE_SESSIONS_SCRIPT, CRASH_REPLAY_AND_HOST_LIBRARY_FILES, DISPATCH_LAUNCHER_DIRECTORY, DISPATCH_LAUNCHER_OPTIONS, DISPATCH_LAUNCHER_SCRIPTS,
    GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES, GATE_STAGE_DIRECTORY, GATE_STAGE_FILES, HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS, MUTATION_PYTHON_LIBRARY_FILES,
    OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK, OWN_SCRIPT_LIBRARY_FILES, OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK, OWN_SCRIPT_LIBRARY_TABLE_LIMIT, OWN_SCRIPT_UNRECOGNIZED_SOURCES,
    SELFTEST_REGISTRY_NAME, STAGE_SELFTEST_SCRIPT,
    TASK_SERVER_CLIENT, TEMPORARY_SERVICE_BINARY_OPTION, TEMPORARY_SERVICE_SOURCE_MARK, TEMPORARY_SERVICE_VARIABLE,
    TEMPORARY_SERVICE_WRAPPER_DIRECTORY, TEMPORARY_SERVICE_WRAPPER_MODE, TEMPORARY_SERVICE_WRAPPER_SCRIPT, checked_cells, file_sha256, gate_stage_name, libtest_lists_only,
    own_library_digest, own_script_digest, project_code_use, read_selftest_registry, runs_compiled_code, selftest_registry_problems, shell_words, stage_selftest_problems,
    starts_a_build, subagent_task_server_refusal, task_server_command_bypass, task_server_guarded, task_server_library, task_server_submission,
    task_server_text_bypass,
)
from lib_project_code_runs_bootstrap import (
    BREAK_VARIABLE, PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME, PROJECT_CODE_RUNS_PART_NAMES, break_is_set,
)
from lib_project_code_runs_launchers import (
    CLAUDE_EXECUTABLE_PATH_WORDS, EXTRA_LAUNCHER_NAMES, FIND_PATH_WORD, PARALLEL_INPUT_WORD, SOURCED_LIBRARY_DEPTH, SOURCED_LIBRARY_LOWERED_DEPTH, XARGS_INPUT_WORD, brace_expansion,
    cargo_arguments_expanded, classify, command_under_dispatch_launcher, command_under_gpu_quiesce, command_under_launcher,
    dispatched_under_memory_cap, environment_under_temporary_service, judged_by_name, launcher_names, program_uses_in_command_text,
    registered_selftest, script_body_unresolved_launch, script_directory_expanded, script_source_reading, shell_source_lines, source_family, temporary_service_bare_selftest,
    temporary_service_bare_stage_samples, temporary_service_command, temporary_service_table_problems, unrecognized_launch,
)
import lib_project_code_runs_base
import lib_project_code_runs_launchers


def build_sample_workspace(work):
    """自证与 project-code-guard.sh 自证共用的虚工作区：仓根的工作区清单与两个成员（core、验证包），launcher 自证读的环境文件，
    门禁样本（喂给 stage-selftest.sh 的几份阶段目录与它们比的样本仓 index-reference），别处树里路径尾部相同的拷贝。"""
    def write(relative, text):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    write("Cargo.toml", '[workspace]\nresolver = "2"\nmembers = ["crates/singlefs-core", "crates/singlefs-verification"]\n')
    write("crates/singlefs-core/Cargo.toml", '[package]\nname = "singlefs-core"\nversion = "0.0.0"\n')
    write("crates/singlefs-core/tests/core_contract.rs", "")
    write("crates/singlefs-verification/Cargo.toml", '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n')
    write("crates/singlefs-verification/tests/overwrite_in_one_instance.rs", "#[test]\nfn a_daily_case() {}\n")
    write("alias.env", '# 样本：systemd-run -p EnvironmentFile= 读的\nCARGO_ALIAS_XT="test -p singlefs-verification"\n')
    write("plain.env", "# 样本\nRUST_BACKTRACE=1\n")
    # 门禁样本（文件头「门禁样本」）：这份文件所在仓里那几道阶段的原样拷贝，与喂给 stage-selftest.sh 的几份阶段目录
    def registered_stage(stage):
        try:
            with open(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, stage), encoding="utf-8") as handle:
                return handle.read()
        except OSError:
            return f"# 这份文件所在仓里没有 {stage}（门禁样本那几格会红）\n"
    for stage in ("doc-text.sh", "doc-decisions.sh"):
        write(f".claude/gate.d/{stage}", registered_stage(stage))
    for gate_directory, stages in (("gd-registered", {"doc-text.sh": registered_stage("doc-text.sh"), "doc-decisions.sh": registered_stage("doc-decisions.sh")}),
                                   ("gd-unregistered", {"brand-new-stage.sh": "#!/usr/bin/env bash\n# 样本：登记表外的阶段\n"})):
        for stage, body in stages.items():
            write(f"{gate_directory}/.claude/gate.d/{stage}", body)
            write(f"{gate_directory}/.claude/gate.d/fixtures/{stage}/red/expect", "exit=1\n")
    write("gd-registered/.claude/gate.d/code-tooling.sh", "#!/usr/bin/env bash\n# 样本：没有样本目录的阶段，stage-selftest.sh 不喂它\n")
    # stage-selftest.sh 喂的样本与暂存区比（文件头「门禁样本」）：index-reference 是临时建的样本仓（自检把 SAMPLE_INDEX_REPOSITORY 换到它），
    # 暂存区里是 gd-registered 那两道的样本；另造几份与它不同的阶段目录：样本里多一份 setup.sh、内容改过、执行位改过、少了文件，与样本仓自己的阶段目录（暂存之后才新加的一份样本）
    reference = os.path.join(work, SAMPLE_INDEX_REFERENCE_NAME)
    for stage in ("doc-text.sh", "doc-decisions.sh"):
        write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/{stage}/red/expect", "exit=1\n")
    git_environment = {name: value for name, value in os.environ.items() if not name.startswith("GIT_")}
    for git_arguments in (["init", "-q"], ["add", "-A"]):
        subprocess.run(["git", "-C", reference, *git_arguments], check=True, capture_output=True, env=git_environment)
    write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/doc-text.sh", registered_stage("doc-text.sh"))
    write(f"{SAMPLE_INDEX_REFERENCE_NAME}/.claude/gate.d/fixtures/doc-text.sh/green/expect", "exit=0\n")
    for gate_directory, extra in (("gd-sample-setup", ("red/setup.sh", "#!/usr/bin/env bash\ncargo test --release -p singlefs-verification\n")),
                                  ("gd-sample-edited", ("red/expect", "exit=0\n")), ("gd-sample-mode", None), ("gd-sample-missing", None)):
        write(f"{gate_directory}/.claude/gate.d/doc-text.sh", registered_stage("doc-text.sh"))
        if gate_directory == "gd-sample-missing":
            os.makedirs(os.path.join(work, gate_directory, ".claude", "gate.d", "fixtures", "doc-text.sh", "green"))   # 只剩一个空的样本目录
        elif extra is None or extra[0] != "red/expect":
            write(f"{gate_directory}/.claude/gate.d/fixtures/doc-text.sh/red/expect", "exit=1\n")
        if extra:
            write(f"{gate_directory}/.claude/gate.d/fixtures/doc-text.sh/{extra[0]}", extra[1])
    os.chmod(os.path.join(work, "gd-sample-mode", ".claude", "gate.d", "fixtures", "doc-text.sh", "red", "expect"), 0o755)
    # 自证登记表按真实路径认：别处树里路径尾部相同的拷贝、指到本仓那一份的符号链接
    write("evil-tree/scripts/kb/claim-experiment.sh", "#!/usr/bin/env bash\n# 改过的自证\nherd7 x.litmus\n")
    write("evil-tree/.claude/singlefs-ai-sop/scripts/stage-selftest.sh", "#!/usr/bin/env bash\n# 样本：改过的 stage-selftest.sh，不认点名\n")
    os.makedirs(os.path.join(work, "linked-tree", "scripts", "gate"))
    os.symlink(os.path.join(HOOK_REPOSITORY, "scripts", "gate", "admission.py"), os.path.join(work, "linked-tree", "scripts", "gate", "admission.py"))
    # case 的模式行（攻方扫描脚本里 case 的那一行原样）：切词把模式切成一条命令
    write("case-pattern.sh", "#!/usr/bin/env bash\nkey=\"$1\"; root=\"$2\"\ncase \"$key\" in\n  .claude/gate.d/lib/*.py) cmd=(python3 \"$root/$key\") ;;\n"
                             "  *) cmd=(bash \"$root/$key\" --selftest) ;;\nesac\necho \"${cmd[@]}\"\n")


# 自检临时建的样本仓（build_sample_workspace 造在临时工作区里，自检把 SAMPLE_INDEX_REPOSITORY 换到它）
SAMPLE_INDEX_REFERENCE_NAME = "index-reference"
SAMPLE_SELFTEST_REGISTRY = (
    "# 样本登记表（lib_project_code_runs 自证造的）\n"
    "scripts/gate/sample-selftest.sh\tselftest\t# 样本：假驱动\n"
    "scripts/gate/sample-bad-kind.sh\tmedium\t# 样本：种类写坏了\n"
    "scripts/gate/sample-short-reason.sh\tselftest\t# 短\n"
    "scripts/gate/sample-pat*.sh\tselftest\t# 样本：文件名里带星号，命令词带通配时不当登记过的自证\n"
    "scripts/gate/sample-missing.sh\tselftest\t# 样本：文件不在\n"
    "elsewhere/sample-outside.sh\tselftest\t# 样本：不在登记目录里\n"
    "scripts/gate/sample-selftest.sh\tselftest\t# 样本：重复的一行\n"
    ".claude/gate.d/sample-stage.sh\tsamples:.sample-tools\t# 样本：样本根放了假工具\n"
    ".claude/gate.d/sample-stage-plain.sh\tsamples\t# 样本：样本只读文本\n"
    ".claude/gate.d/sample-stage-wrong.sh\tselftest\t# 样本：门禁阶段写成自证的种类\n"
    ".claude/gate.d/sample-stage-escape.sh\tsamples:../outside\t# 样本：假工具目录跳出样本根\n"
    "scripts/gate/sample-samples-misplaced.sh\tsamples\t# 样本：样本档的种类写在研究脚本上\n"
    "scripts/experiment-apparatus/sample_device/sample_orchestrator.py\tselftest\t# 样本：实验装置那一层的编排脚本，自证用假驱动\n"
    "scripts/experiment-apparatus/sample_device/deeper/sample_too_deep.py\tselftest\t# 样本：装置目录下面再深一层\n"
    "scripts/sample_top.py\tselftest\t# 样本：直接放在 scripts/ 下、不在哪个分类目录里\n"
    "task-server/sample_server_lib.py\tselftest\t# 样本：统一任务服务那一层的脚本，自证用假驱动\n"
    "prune-server/sample_control\tselftest\t# 样本：判定库服务那一层的脚本，自证只起假的 ip、ssh、systemctl\n"
    "gpu-verification-server/sample_control\tselftest\t# 样本：GPU 批处理验证服务那一层的脚本，自证只起假的 ssh、systemctl\n"
    ".claude/gate.d/lib-sample-library.py\tselftest\t# 样本：门禁目录顶层以 lib- 起头的共用库，自证用假驱动\n"
    ".claude/gate.d/lib/sample-library.py\tselftest\t# 样本：门禁共用库目录里的库，直接执行就是跑自证\n")
# 样本表里该报出来的行（键）：种类认不出、理由太短、不在登记目录里、重复、文件不在
SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS = ["scripts/gate/sample-bad-kind.sh", "scripts/gate/sample-short-reason.sh", "elsewhere/sample-outside.sh",
                                         "scripts/gate/sample-selftest.sh", "scripts/gate/sample-missing.sh", ".claude/gate.d/sample-stage-wrong.sh",
                                         ".claude/gate.d/sample-stage-escape.sh", "scripts/gate/sample-samples-misplaced.sh",
                                         "scripts/experiment-apparatus/sample_device/deeper/sample_too_deep.py", "scripts/sample_top.py"]
# 样本登记表里实验装置那一层的那个装置目录（文件头「实验装置那一层」）
SAMPLE_DEVICE_LAYER = "scripts/experiment-apparatus/sample_device/"
# 引导那几格（部分文件读不到、编不过、截空）拷进临时目录的文件：入口、它载入的 lib_shell_words 那三份、引导那一份与每一份部分文件，逐个写出来，
# 算键库（scripts/gate/lib_function_units.py 文件头「运行期读的数据文件」一）折得出拷的是哪几份，判法的键只连它们、不连整个 .claude/hooks；
# 引导那一份与 PROJECT_CODE_RUNS_PART_NAMES 的每一份都在里面由自证「引导：拷进临时目录的文件里有引导那一份与每一份部分文件」核
PART_PROBE_COPIED_FILES = (
    "lib_project_code_runs.py", "lib_shell_words.py", "lib_shell_words_core.py", "lib_shell_words_embedded.py", "lib_project_code_runs_bootstrap.py",
    "lib_project_code_runs_base.py", "lib_project_code_runs_launchers.py", "lib_project_code_runs_selftest.py",
)
# 「本仓那一份照表里的字面路径读」那一格挑的几份：按名字判的、门禁阶段、派发起法各一份，库每张库表一份（own_library_digest 逐张各挑一圈，漏挑哪一张那一格列出它的那一份）；
# 路径逐个写出来，那一格照它们读本仓那一份的 sha256 来比
OWN_DIGEST_PROBE_SCRIPTS = ("scripts/mutation/mutate.sh", ".claude/gate.d/doc-text.sh", "scripts/hosts/multi-host-run.sh")
OWN_DIGEST_PROBE_LIBRARIES = (".claude/gate.d/lib/stage-cells.sh", "scripts/hosts/heavy-threads.sh", "scripts/mutation/lib_mutation_units.py")
# 「库表：」那一格逐张量份数的几张库表（名字, 表）：与 lib_project_code_runs_base.py 拼整张 OWN_SCRIPT_LIBRARY_FILES 的那几张逐张相同、次序相同（那一格另核拼起来等于整张）。
# 弄坏开关 own-library-tables-merged 下那一格换成量拼成的整张一张
OWN_SCRIPT_LIBRARY_NAMED_TABLES = (
    ("GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES", GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES),
    ("CRASH_REPLAY_AND_HOST_LIBRARY_FILES", CRASH_REPLAY_AND_HOST_LIBRARY_FILES),
    ("MUTATION_PYTHON_LIBRARY_FILES", MUTATION_PYTHON_LIBRARY_FILES),
)
# 同族拷贝连同 source 的库一起比那两格拷过去的：一道 source 了 lib/ 下几份库的门禁阶段与它引的每一份库（盘上有的）
FAMILY_LIBRARY_PROBE_FILES = (
    ".claude/gate.d/crash-point-replay.sh", ".claude/gate.d/lib/stage-cells.sh", ".claude/gate.d/lib/crash-point-replay-selection-and-plan.sh",
    ".claude/gate.d/lib/crash-point-replay-package-pass-verdicts.sh", ".claude/gate.d/lib/crash-point-replay-crash-case-cells.sh",
    ".claude/gate.d/lib/crash-point-replay-feature-card-and-audit-cells.sh", ".claude/scripts/preflight.sh",
)
# 经变量 source 的库照样比的那四格：(样本树名, 入口, 改过的那一份库)，入口连同它引的每一份库原样拷过去（copy_with_own_libraries）、只改那一份
STALE_VARIABLE_LIBRARY_PROBES = (
    ("doc-text-stage-cells", ".claude/gate.d/doc-text.sh", ".claude/gate.d/lib/stage-cells.sh"),
    ("doc-text-content-and-history-cells", ".claude/gate.d/doc-text.sh", ".claude/gate.d/lib/doc-text-content-and-history-cells.sh"),
    ("code-tooling-selftest-cells", ".claude/gate.d/code-tooling.sh", ".claude/gate.d/lib/code-tooling-selftest-cells.sh"),
    ("multi-host-run-peer-host-lib", "scripts/hosts/multi-host-run.sh", "scripts/hosts/peer-host-lib.sh"),
)
# 先加 source 行、后建库那两格：样本仓里这一道阶段末尾加的一行与它引的新库
TWO_STEP_STAGE = ".claude/gate.d/crash-point-replay.sh"
TWO_STEP_LIBRARY = ".claude/gate.d/lib/two-step-library.sh"
TWO_STEP_SOURCE_LINE = '\nsource "$(dirname "${BASH_SOURCE[0]}")/lib/two-step-library.sh"\n'
# shell_source_lines 那一格的样本正文（当成 entry.sh 读）：仓里三张表的脚本用到的几种写法各一行，另有赋值不一致的、认不出的变量、注释与嵌在里面的 python 赋值
SHELL_SOURCE_FORMS_SAMPLE = (
    'STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"\n'
    'REPO_ROOT="$(cd "$STAGE_DIR/../.." && pwd)"\n'
    'LIBRARY_DIRECTORY="$STAGE_DIR/lib"\n'
    'own_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"\n'
    'own_directory="$(dirname "$own_script")"\n'
    'RESEARCH="$(cd "$(dirname "$0")/../apparatus" && pwd)"\n'
    'CHANGED_PATHS="$(cd "$STAGE_DIR/../.." && pwd)/scripts/gate/changed-paths.sh"\n'
    'source "$(dirname "${BASH_SOURCE[0]}")/lib/direct.sh"\n'
    'source "$STAGE_DIR/lib/stage-cells.sh"\n'
    'source "${REPO_ROOT}/scripts/hosts/heavy-threads.sh"\n'
    '. "$LIBRARY_DIRECTORY/dotted.sh"\n'
    'source "$own_directory/own-selftest.sh"\n'
    'source "$RESEARCH/long-run-lib/identity.sh"\n'
    'source "$CHANGED_PATHS"\n'
    'twice="$(cd "$(dirname "$0")" && pwd)"\n'
    'twice=/elsewhere\n'
    'source "$twice/twice.sh"\n'
    'source "$UNKNOWN/unknown.sh"\n'
    'cd "$(dirname "$0")/../.." || exit 2\n'
    'source scripts/hosts/run-build-directory.sh\n'
    '# source "$STAGE_DIR/lib/commented.sh"\n'
    '        source = io.TextIOWrapper(sys.stdin.buffer)\n'
)
SHELL_SOURCE_FORMS_EXPECTED = [
    ('"$(dirname "${BASH_SOURCE[0]}")/lib/direct.sh"', "lib/direct.sh"), ('"$STAGE_DIR/lib/stage-cells.sh"', "lib/stage-cells.sh"),
    ('"${REPO_ROOT}/scripts/hosts/heavy-threads.sh"', "../../scripts/hosts/heavy-threads.sh"), ('"$LIBRARY_DIRECTORY/dotted.sh"', "lib/dotted.sh"),
    ('"$own_directory/own-selftest.sh"', "own-selftest.sh"), ('"$RESEARCH/long-run-lib/identity.sh"', "../apparatus/long-run-lib/identity.sh"),
    ('"$CHANGED_PATHS"', "../../scripts/gate/changed-paths.sh"), ('"$twice/twice.sh"', None), ('"$UNKNOWN/unknown.sh"', None),
    ("scripts/hosts/run-build-directory.sh", "../../scripts/hosts/run-build-directory.sh"),
]
# 不经识别函数数 source 行那一格认的行（与 shell_source_lines 分开写的另一条正则，注释行另外跳过）：命令位置上（行首、; & | ( { ! ) 之后、then / do / else / elif / if /
# while / until 之后，中间可以垫命令前的赋值与 builtin / command）的 source 与 .，与引号里交给 eval 或带 -c 的程序的代码起头的 source 与 .；后面跟着 = 的不算
INDEPENDENT_SOURCE_LINE = re.compile(r"""(?:^|[;&|({!)]|\b(?:then|do|else|elif|if|while|until)\b)\s*(?:(?:[A-Za-z_]\w*=\S*|builtin|command)\s+)*(?:source|\.)\s+(?![\s=])"""
                                     r"""|(?:\beval|\s-[A-Za-z]*c[A-Za-z]*)\s+["']\s*(?:(?:builtin|command)\s+)?(?:source|\.)\s+(?![\s=])""")
# 认 source 行的几种写法各一格（shell_source_lines）：正文前面先垫一行认得出的目录变量 X，(格名, 正文, 垫的那一行之后交回的位置)；认出的交 lib/a.sh，认不出的交 None
SHELL_SOURCE_DIRECTORY_ASSIGNMENT = 'X="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"\n'
SHELL_SOURCE_FORM_CASES = (
    ("source 的写法：case 分支 ) 之后的 source 认出", 'case "${MODE:-}" in\n  gpu) source "$X/lib/a.sh" ;;\nesac\n', ["lib/a.sh"]),
    ("source 的写法：命令前垫赋值（QUIET=1 source）认出", 'QUIET=1 source "$X/lib/a.sh"\n', ["lib/a.sh"]),
    ("source 的写法：builtin source 认出", 'builtin source "$X/lib/a.sh"\n', ["lib/a.sh"]),
    ("source 的写法：command . 认出", 'command . "$X/lib/a.sh"\n', ["lib/a.sh"]),
    ("source 的写法：elif 条件里的 source 认出", 'if [[ -n "${Y:-}" ]]; then :\nelif source "$X/lib/a.sh"; then :\nfi\n', ["lib/a.sh"]),
    ("source 的写法：source <(…) 交回、位置交 None（进认不出）", 'source <(cat "$X/lib/a.sh")\n', [None]),
    ("source 的写法：&& 之后的 . <(…) 交回、位置交 None（进认不出）", '[[ -n "${Y:-}" ]] && . <("$X/lib/a.sh" --print-environment)\n', [None]),
    ("source 的写法：eval 串里起头的 source 交回、位置交 None（进认不出）", 'eval "source \\"$X/lib/a.sh\\""\n', [None]),
    ("source 的写法：bash -c 串里起头的 source 交回、位置交 None（进认不出）", "bash -c 'source \"$1\" && f' child \"$X/lib/a.sh\"\n", [None]),
)
# 认不出的写法接在门禁阶段 TWO_STEP_STAGE 末尾那几格：(样本树名, 格名里写的写法, 接上的那一行)，引的都是新建的 TWO_STEP_LIBRARY
UNRECOGNIZED_SOURCE_APPENDED_FORMS = (
    ("process-substitution", "source <(…)", '\nsource <(cat "$(dirname "${BASH_SOURCE[0]}")/lib/two-step-library.sh")\n'),
    ("eval-string", "eval 串", "\neval 'source lib/two-step-library.sh'\n"),
    ("child-shell-string", "bash -c 串", '\nbash -c \'source "$1" && true\' sample "$(dirname "${BASH_SOURCE[0]}")/lib/two-step-library.sh"\n'),
)
# 登记按行认那一格：样本仓里这一份库（它照写 "$library" 的那一条 source 登记过）末尾加一个函数，照写同一个参数另 source 一份别的库（peer-host-lib.sh）
REGISTERED_SOURCE_LIBRARY = ".claude/gate.d/lib/crash-point-replay-crash-case-cells.sh"
REGISTERED_ARGUMENT_REPEATED_SOURCE = ('\n\ncrash_point_replay_sample_peer_value() {\n'
                                       '  local library="$crash_point_replay_stage_repository/scripts/hosts/peer-host-lib.sh"\n'
                                       '  (source "$library" && peer_host_configured_value SAMPLE) 2>/dev/null\n}\n')
REGISTERED_ARGUMENT_REPEATED_MARK = "peer_host_configured_value SAMPLE"


def copy_with_own_libraries(location, tree):
    """本仓 location（相对仓根）那一份与它引的每一份库（script_source_reading，本仓盘上有的）原样拷进 tree 的同一位置。"""
    own_path = os.path.join(HOOK_REPOSITORY, location)
    for relative in [os.path.basename(location)] + script_source_reading(own_path, location).libraries:
        copied = os.path.normpath(os.path.join(os.path.dirname(location), relative))
        if os.path.isfile(os.path.join(HOOK_REPOSITORY, copied)):
            os.makedirs(os.path.dirname(os.path.join(tree, copied)), exist_ok=True)
            shutil.copyfile(os.path.join(HOOK_REPOSITORY, copied), os.path.join(tree, copied))


def appended_stage_trees(work, name, appended_file, addition, created_files=()):
    """样本仓 <work>/<name>-own：本仓 TWO_STEP_STAGE 与它引的库原样拷过去（copy_with_own_libraries），appended_file（相对仓根）末尾接上 addition，
    created_files 里的 (相对仓根的路径, 正文) 逐份新建；<work>/<name>-copy 是它逐字节相同的拷贝。交 (样本仓, 拷贝)。"""
    own, copy = os.path.join(work, f"{name}-own"), os.path.join(work, f"{name}-copy")
    copy_with_own_libraries(TWO_STEP_STAGE, own)
    with open(os.path.join(own, appended_file), "a", encoding="utf-8") as handle:
        handle.write(addition)
    for created, body in created_files:
        os.makedirs(os.path.dirname(os.path.join(own, created)), exist_ok=True)
        with open(os.path.join(own, created), "w", encoding="utf-8") as handle:
            handle.write(body)
    shutil.copytree(own, copy)
    return own, copy


def own_directory_source(name):
    """按自己所在目录 source name 的那个参数：`"$(dirname "${BASH_SOURCE[0]}")/<name>"`。"""
    return '"$(dirname "${BASH_SOURCE[0]}")/' + name + '"'


def source_reading_cells(work, results):
    """认 source 行与往库里读的几样（文件头 shell_source_lines、script_source_reading）：几种写法各一格、认不出又没登记的写法让拷贝照不同判、
    登记按「文件、参数、第几条」认、库再引回入口不多占一层、撞上深度上限照不同判。样本都建在 work 里，本仓那一边要换的经 own_repository_swapped 换；
    调用方在 FAMILY_OWN_REPOSITORY 是 None 的那一段里调（样本树不当本仓那一份）。格记进 results，交判了几格。开关红在哪几格写在文件头开关表。"""
    before = len(results)
    for label, form, want in SHELL_SOURCE_FORM_CASES:
        form_lines = shell_source_lines(SHELL_SOURCE_DIRECTORY_ASSIGNMENT + form, "entry.sh")[0]
        results.append((label, want, [location for number, _written, location in form_lines if number > 1]))
    for tree_name, form, addition in UNRECOGNIZED_SOURCE_APPENDED_FORMS:
        own, copy = appended_stage_trees(work, f"unrecognized-{tree_name}", TWO_STEP_STAGE, addition, ((TWO_STEP_LIBRARY, "# 样本：认不出的写法引的库\n"),))
        with own_repository_swapped(own, OWN_SCRIPT_LIBRARY_FILES + (TWO_STEP_LIBRARY,)):
            results.append((f"同族：认不出的 source 写法（{form}）没登记，逐字节相同的门禁阶段拷贝照不同判（读正文）", False, judged_by_name(TWO_STEP_STAGE, copy)))
    # 登记按行认：同一份库里照写同一个参数的那一条另起一份，它排的第几条对不上登记，进认不出（表那一格的 unregistered 就是三张表的脚本读出来的这一项）
    own, copy = appended_stage_trees(work, "registered-argument-repeated", REGISTERED_SOURCE_LIBRARY, REGISTERED_ARGUMENT_REPEATED_SOURCE)
    with open(os.path.join(own, REGISTERED_SOURCE_LIBRARY), encoding="utf-8") as handle:
        repeated_numbers = [number for number, line in enumerate(handle.read().split("\n"), 1) if REGISTERED_ARGUMENT_REPEATED_MARK in line]
    with own_repository_swapped(own, OWN_SCRIPT_LIBRARY_FILES):
        repeated_judged = judged_by_name(TWO_STEP_STAGE, copy)
        repeated_reading = script_source_reading(os.path.join(copy, TWO_STEP_STAGE), TWO_STEP_STAGE)
    repeated_library = os.path.relpath(REGISTERED_SOURCE_LIBRARY, os.path.dirname(TWO_STEP_STAGE))
    results.append(("同族：登记按行认：同一份里多出一条照写同一个参数的 source，不被登记过的那一条吸收，进认不出、逐字节相同的门禁阶段拷贝照不同判",
                    (False, [(repeated_library, number, '"$library"') for number in repeated_numbers]),
                    (repeated_judged, [entry for entry in repeated_reading.unrecognized if entry[2] == '"$library"'])))
    # 库再引回入口：入口先记成读过，不当库再读一遍、不往下多占一层：链长到上限前一层（SOURCED_LIBRARY_DEPTH - 2 份）照读完、cut 是空的
    reread_root = os.path.join(work, "library-reread")
    os.makedirs(reread_root)
    chain_length = SOURCED_LIBRARY_DEPTH - 2
    reread_files = {"entry.sh": f"source {own_directory_source('back-reference.sh')}\nsource {own_directory_source('depth-1.sh')}\n",
                    "back-reference.sh": f"source {own_directory_source('entry.sh')}\n"}
    for depth in range(1, chain_length + 1):
        reread_files[f"depth-{depth}.sh"] = f"source {own_directory_source(f'depth-{depth + 1}.sh')}\n" if depth < chain_length else "# 样本：链尾\n"
    for reread_name, reread_body in reread_files.items():
        with open(os.path.join(reread_root, reread_name), "w", encoding="utf-8") as handle:
            handle.write(reread_body)
    reread = script_source_reading(os.path.join(reread_root, "entry.sh"))
    results.append(("source 的库：库再引回入口时入口不当库再读、不往下多占一层（链长到深度上限前一层的照读完）",
                    (["back-reference.sh"] + [f"depth-{depth}.sh" for depth in range(1, chain_length + 1)], []), (reread.libraries, reread.cut)))
    # 撞上深度上限：门禁阶段末尾接一条长 SOURCED_LIBRARY_DEPTH 的链（lib/depth-<层>.sh 逐层引下一层），落在上限那一层的进 cut，逐字节相同的拷贝照不同判
    chain_files = [f".claude/gate.d/lib/depth-{depth}.sh" for depth in range(1, SOURCED_LIBRARY_DEPTH + 1)]
    chain_bodies = [(chain_file, f"source {own_directory_source(f'depth-{depth + 1}.sh')}\n" if depth < SOURCED_LIBRARY_DEPTH else "# 样本：链尾\n")
                    for depth, chain_file in enumerate(chain_files, 1)]
    own, copy = appended_stage_trees(work, "library-depth-cut", TWO_STEP_STAGE, f"\nsource {own_directory_source('lib/depth-1.sh')}\n", chain_bodies)
    with own_repository_swapped(own, OWN_SCRIPT_LIBRARY_FILES + tuple(chain_files)):
        cut_judged = judged_by_name(TWO_STEP_STAGE, copy)
        cut_reading = script_source_reading(os.path.join(copy, TWO_STEP_STAGE), TWO_STEP_STAGE)
    results.append(("同族：库往下读撞上深度上限、有一份没读完的，逐字节相同的门禁阶段拷贝照不同判（读正文）",
                    (False, [f"lib/depth-{SOURCED_LIBRARY_DEPTH}.sh"]), (cut_judged, cut_reading.cut)))
    return len(results) - before


@contextlib.contextmanager
def own_repository_swapped(root, library_files):
    """只给自证用：这一段里「本仓那一边」（own_script_digest、own_library_digest 读的仓根与几张库表）换成样本仓 root 与 library_files，出来换回。
    library_files 整张放进门禁阶段那一张库表，另两张换成空的（own_library_digest 逐张挑，挑到的就是 library_files 里的），拼成的整张也换成它。"""
    saved = (lib_project_code_runs_base.HOOK_REPOSITORY, lib_project_code_runs_base.GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES,
             lib_project_code_runs_base.CRASH_REPLAY_AND_HOST_LIBRARY_FILES, lib_project_code_runs_base.MUTATION_PYTHON_LIBRARY_FILES,
             lib_project_code_runs_base.OWN_SCRIPT_LIBRARY_FILES)
    lib_project_code_runs_base.HOOK_REPOSITORY = root
    lib_project_code_runs_base.GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES = tuple(library_files)
    lib_project_code_runs_base.CRASH_REPLAY_AND_HOST_LIBRARY_FILES = ()
    lib_project_code_runs_base.MUTATION_PYTHON_LIBRARY_FILES = ()
    lib_project_code_runs_base.OWN_SCRIPT_LIBRARY_FILES = tuple(library_files)
    try:
        yield
    finally:
        (lib_project_code_runs_base.HOOK_REPOSITORY, lib_project_code_runs_base.GATE_STAGE_AND_PREFLIGHT_LIBRARY_FILES,
         lib_project_code_runs_base.CRASH_REPLAY_AND_HOST_LIBRARY_FILES, lib_project_code_runs_base.MUTATION_PYTHON_LIBRARY_FILES,
         lib_project_code_runs_base.OWN_SCRIPT_LIBRARY_FILES) = saved


class EqualLocation(str):
    """与 location 比相等、哈希相同，字面却是 <elsewhere>/<location> 的位置：按相等从字面表里挑、照表里的字面拼路径读的，读到本仓那一份；
    拿传进来的位置拼路径读的，读的是 elsewhere 底下那一份（自证不建它，读不到交 None）。"""
    def __new__(cls, location, elsewhere):
        equal = super().__new__(cls, os.path.join(elsewhere, location))
        equal.location = location
        return equal

    def __eq__(self, other):
        return self.location == other

    def __ne__(self, other):
        return not self.__eq__(other)

    def __hash__(self):
        return hash(self.location)


def location_joined_digest(location):
    """弄坏开关 own-script-digest-location-joined 下自证拿它顶替 own_script_digest：先查三张字面表，查中拿传进来的位置拼路径读。"""
    listed = (set(KNOWN_SCRIPT_LOCATIONS.values()) | {os.path.join(GATE_STAGE_DIRECTORY, stage) for stage in GATE_STAGE_FILES}
              | {os.path.join(DISPATCH_LAUNCHER_DIRECTORY, launcher) for launcher in DISPATCH_LAUNCHER_SCRIPTS})
    return file_sha256(os.path.join(HOOK_REPOSITORY, location)) if location in listed else None


@contextlib.contextmanager
def selftest_registry_swapped(registry, root):
    """只给自证用（平常跑的判法不经它，读的是这份文件旁边那份表）：这一段里登记表换成 registry、键相对 root 认，出来换回。"""
    saved = lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT
    lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT = registry, root
    try:
        yield
    finally:
        lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT = saved


def temporary_service_cells(work, results):
    """要临时服务的自证登记表的读法与裸 --selftest 的判法（文件头 temporary_service_bare_selftest）：在 work 里写样本的自证登记表、要临时服务的表与它们点名的脚本，
    把 SELFTEST_REGISTRY_PATH、SELFTEST_REGISTRY_ROOT、TEMPORARY_SERVICE_TABLE_PATH 换过去判几条命令，判完换回；再核仓里那张表与自证登记表、脚本源码对得上。
    格记进 results，交回判了几格。弄坏开关红的格写在文件头开关表。"""
    saved = lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT, lib_project_code_runs_base.TEMPORARY_SERVICE_TABLE_PATH
    sample_registry = os.path.join(work, "sample-temporary-service-registry.tsv")
    sample_table = os.path.join(work, "sample-temporary-service-table.tsv")
    marked = f"# 样本：开跑先调 {TEMPORARY_SERVICE_SOURCE_MARK}\n"
    for relative, text in (("scripts/gate/sample-needs-service.sh", marked), ("scripts/gate/sample-own-server.py", marked), ("scripts/gate/sample-short-reason.sh", marked),
                           ("scripts/gate/sample-marked-unlisted.py", marked), ("scripts/gate/sample-listed-unmarked.sh", "# 样本：没有核对\n"),
                           ("scripts/gate/sample-not-registered.sh", marked), ("scripts/gate/sample-plain.sh", "# 样本：不要临时服务\n"),
                           ("elsewhere-tree/scripts/gate/sample-needs-service.sh", marked),
                           # 拆分族：入口里没有核对、记号在入口 import / source 的同目录部分文件里（标了记号的一对）；族里哪一份都没有记号的一对；import 的模块在别的目录里的
                           ("scripts/gate/sample-split-marked.py", "# 样本：拆分族的入口，没有核对\nfrom sample_split_marked_part import value\n"),
                           ("scripts/gate/sample_split_marked_part.py", marked),
                           ("scripts/gate/sample-split-unmarked.py", "# 样本：拆分族的入口，族里哪一份都没有核对\nfrom sample_split_unmarked_part import value\n"),
                           ("scripts/gate/sample_split_unmarked_part.py", "# 样本：拆分族的部分文件，没有核对\n"),
                           ("scripts/gate/sample-split-marked.sh", '# 样本：拆分族的入口，没有核对\nsource "$sample_directory/sample-split-marked-part.sh"\n'),
                           ("scripts/gate/sample-split-marked-part.sh", marked),
                           ("scripts/gate/sample-split-unmarked.sh", '# 样本：拆分族的入口，族里哪一份都没有核对\nsource "$sample_directory/sample-split-unmarked-part.sh"\n'),
                           ("scripts/gate/sample-split-unmarked-part.sh", "# 样本：拆分族的部分文件，没有核对\n"),
                           ("scripts/gate/sample-elsewhere-import.py", "# 样本：import 的模块在别的目录里，不是同目录的部分文件\nfrom sample_elsewhere_part import value\n"),
                           ("scripts/crash/sample_elsewhere_part.py", marked),
                           # 名字像部分文件而入口没载入的（记号只在它里面）；被入口 import 的共用库与被字符串点名的别的脚本（名字不像部分文件，记号只在它们里面）：都不在族里
                           ("scripts/gate/sample-name-like.py", "# 样本：入口，什么都没载入，没有核对\n"),
                           ("scripts/gate/sample_name_like_shadow.py", marked),
                           ("scripts/gate/sample-imports-shared.py", "# 样本：入口，用几份脚本共用的库里的一个小工具，没有核对\nimport sample_shared_checks\n"),
                           ("scripts/gate/sample_shared_checks.py", marked),
                           ("scripts/gate/sample-quotes-sibling.py", '# 样本：入口，字符串里点名同目录另一份脚本，没有核对\nHELPER = "sample-own-checker.py"\n'),
                           ("scripts/gate/sample-own-checker.py", marked),
                           # 载入按语法认：注释里点名、文档串里写 import 行的不算载入；字面量参数的 importlib.import_module 算载入
                           ("scripts/gate/sample-comment-named.py",
                            '# 样本：入口，注释里点名 "sample_comment_named_shadow.py"，文档串里写 import 行，都没有载入它\n"""用法说明：\n    import sample_comment_named_shadow   # 别这样写\n"""\n'),
                           ("scripts/gate/sample_comment_named_shadow.py", marked),
                           ("scripts/gate/sample-import-module.py",
                            '# 样本：入口，按字面量参数的 importlib.import_module 载入同目录的部分文件\nimport importlib\nsample_import_module_part = importlib.import_module("sample_import_module_part")\n'),
                           ("scripts/gate/sample_import_module_part.py", marked),
                           # 门禁阶段入口的部分文件在 .claude/gate.d/lib/ 下：入口给绝对路径时命名那一半也要认出来
                           ("absolute-stage/.claude/gate.d/sample-stage.sh", '# 样本：门禁阶段入口，source 它 lib/ 下的部分文件\nsource "$STAGE_DIRECTORY/lib/sample-stage-cells.sh"\n'),
                           ("absolute-stage/.claude/gate.d/lib/sample-stage-cells.sh", marked)):
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
            handle.write(text)
    stage_samples_directory = os.path.join(work, "gd-temporary-service", ".claude", "gate.d")
    for stage, text in (("needs-service.sh", marked), ("plain-stage.sh", "# 样本：不要临时服务\n")):
        os.makedirs(os.path.join(stage_samples_directory, "fixtures", stage, "red"), exist_ok=True)
        with open(os.path.join(stage_samples_directory, stage), "w", encoding="utf-8") as handle:
            handle.write(text)
    with open(sample_registry, "w", encoding="utf-8") as handle:
        handle.write("".join(f"scripts/gate/{name}\tselftest\t# 样本：自证用假的驱动\n" for name in (
            "sample-needs-service.sh", "sample-own-server.py", "sample-short-reason.sh", "sample-marked-unlisted.py", "sample-listed-unmarked.sh", "sample-plain.sh",
            "sample-split-marked.py", "sample-split-unmarked.py", "sample-split-marked.sh", "sample-split-unmarked.sh", "sample-elsewhere-import.py",
            "sample-name-like.py", "sample-imports-shared.py", "sample-quotes-sibling.py")))
    with open(sample_table, "w", encoding="utf-8") as handle:
        handle.write("# 样本：要临时服务的自证登记表\n"
                     "scripts/gate/sample-needs-service.sh\trequired\t# 样本：开跑先核临时服务，裸跑退 2\n"
                     "scripts/gate/sample-own-server.py\town-server\t# 样本：自己起服务，裸跑不退 2\n"
                     "scripts/gate/sample-short-reason.sh\trequired\t# 短\n"
                     "scripts/gate/sample-bad-judgement.sh\tmaybe\t# 样本：判定写坏的一行\n"
                     "scripts/gate/sample-needs-service.sh\trequired\t# 样本：键重复的一行\n"
                     "scripts/gate/sample-listed-unmarked.sh\trequired\t# 样本：登记了 required 而源码里没有核对\n"
                     "scripts/gate/sample-not-registered.sh\trequired\t# 样本：自证登记表里没有这一份\n"
                     "scripts/gate/sample-missing.sh\trequired\t# 样本：文件不在\n"
                     "scripts/gate/sample-split-marked.py\trequired\t# 样本：拆分族里记号在同目录的部分文件里\n"
                     "scripts/gate/sample-split-unmarked.py\trequired\t# 样本：拆分族里哪一份都没有核对\n"
                     "scripts/gate/sample-split-marked.sh\trequired\t# 样本：拆分族里记号在同目录的部分文件里\n"
                     "scripts/gate/sample-split-unmarked.sh\trequired\t# 样本：拆分族里哪一份都没有核对\n"
                     "scripts/gate/sample-elsewhere-import.py\trequired\t# 样本：记号在别的目录的模块里，不在族里\n"
                     "scripts/gate/sample-name-like.py\trequired\t# 样本：记号在名字像部分文件而入口没载入的文件里，不在族里\n"
                     "scripts/gate/sample-imports-shared.py\trequired\t# 样本：记号在入口 import 的共用库里，不在族里\n"
                     "scripts/gate/sample-quotes-sibling.py\trequired\t# 样本：记号在入口字符串里点名的别的脚本里，不在族里\n")
    before = len(results)
    needs_service = "scripts/gate/sample-needs-service.sh"
    wrapped = f"{TEMPORARY_SERVICE_WRAPPER_MODE} -- bash {needs_service} --selftest"
    try:
        lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT, lib_project_code_runs_base.TEMPORARY_SERVICE_TABLE_PATH = sample_registry, work, sample_table
        bare = temporary_service_bare_selftest([needs_service, "--selftest"], work, {})
        results.append(("要临时服务：required 行的裸 --selftest 交整条包装写法", (True, True),
                        (bare is not None, bare is not None and f"task-server/task submit -- python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {wrapped}" in bare)))
        results.append(("要临时服务：命令前设了临时服务的标记，裸 --selftest 放行", None,
                        temporary_service_bare_selftest([needs_service, "--selftest"], work, {TEMPORARY_SERVICE_VARIABLE: "/sample/server.sock"})))
        marked_environment = environment_under_temporary_service({"PATH": "/sample"})
        results.append(("要临时服务：包装剥开重判时补上的标记让里面那条放行", (True, None),
                        (marked_environment.get(TEMPORARY_SERVICE_VARIABLE, "") != "" and marked_environment.get("PATH") == "/sample",
                         temporary_service_bare_selftest([needs_service, "--selftest"], work, marked_environment))))
        results.append(("要临时服务：own-server 行的裸 --selftest 不拒", None, temporary_service_bare_selftest(["scripts/gate/sample-own-server.py", "--selftest"], work, {})))
        results.append(("要临时服务：带了 --selftest 之外参数的不拒", None, temporary_service_bare_selftest([needs_service, "--selftest", "--cells", "x"], work, {})))
        results.append(("要临时服务：表里没有的脚本不拒", None, temporary_service_bare_selftest(["scripts/gate/sample-plain.sh", "--selftest"], work, {})))
        results.append(("要临时服务：别的树里路径尾部相同的不拒", None,
                        temporary_service_bare_selftest([os.path.join(work, "elsewhere-tree", needs_service), "--selftest"], work, {})))
        results.append(("要临时服务：理由太短的一行不算登记", None, temporary_service_bare_selftest(["scripts/gate/sample-short-reason.sh", "--selftest"], work, {})))
        sample_problems = temporary_service_table_problems(sample_table, sample_registry, work)
        found_keys = sorted({match.group(1) for problem in sample_problems for match in [re.match(r"^(?:第 \d+ 行 )?(\S+)：", problem)] if match})
        results.append(("要临时服务：样本表写坏的、两边对不上的几行逐条报出来（判定、理由、键重复、文件不在、没登记、源码里没有核对、漏登）",
                        sorted(f"scripts/gate/sample-{name}" for name in ("bad-judgement.sh", "listed-unmarked.sh", "marked-unlisted.py", "missing.sh", "needs-service.sh",
                                                                          "not-registered.sh", "short-reason.sh", "split-unmarked.py", "split-unmarked.sh", "elsewhere-import.py",
                                                                          "name-like.py", "imports-shared.py", "quotes-sibling.py")),
                        found_keys))
        # 拆分族（source_family）：记号挪进入口 import / source 的同目录部分文件的 required 行不报；族里哪一份都没有记号的、记号在别的目录的模块里的照旧报「源码里没有核对」
        # （开关 temporary-service-mark-entry-only 下这三格与上面那一格、仓里的表那一格红）
        reported_unmarked = lambda name: any(problem.startswith(f"scripts/gate/{name}：登记了") for problem in sample_problems)
        results.append(("要临时服务：拆分族里记号挪进同目录部分文件的 required 行不报", (False, False),
                        (reported_unmarked("sample-split-marked.py"), reported_unmarked("sample-split-marked.sh"))))
        results.append(("要临时服务：拆分族里哪一份都没有记号的 required 行照旧报源码里没有核对", (True, True, True),
                        (reported_unmarked("sample-split-unmarked.py"), reported_unmarked("sample-split-unmarked.sh"), reported_unmarked("sample-elsewhere-import.py"))))
        family_of = lambda name: [os.path.relpath(member, work) for member in source_family(os.path.join(work, "scripts", "gate", name))]
        results.append(("要临时服务：拆分族：python 入口连同它 import 的同目录模块", ["scripts/gate/sample-split-marked.py", "scripts/gate/sample_split_marked_part.py"],
                        family_of("sample-split-marked.py")))
        results.append(("要临时服务：拆分族：shell 入口连同它 source 的同目录库", ["scripts/gate/sample-split-marked.sh", "scripts/gate/sample-split-marked-part.sh"],
                        family_of("sample-split-marked.sh")))
        results.append(("要临时服务：拆分族：别的目录里的模块不在族里", ["scripts/gate/sample-elsewhere-import.py"], family_of("sample-elsewhere-import.py")))
        # 族取命名与载入两样的交集：名字像部分文件而入口没载入的不算（开关 temporary-service-family-by-name 与 temporary-service-family-by-directory 下红），
        # 入口 import 的共用库与字符串里点名的别的脚本不算（开关 temporary-service-family-by-loading 与 temporary-service-family-by-directory 下红）；记号只在它们里面，required 行照旧报
        results.append(("要临时服务：拆分族：名字像部分文件而入口没载入的同目录文件不在族里，记号只在它里面的 required 行照旧报源码里没有核对",
                        (["scripts/gate/sample-name-like.py"], True),
                        (family_of("sample-name-like.py"), reported_unmarked("sample-name-like.py"))))
        results.append(("要临时服务：拆分族：入口 import 的共用库与字符串里点名的别的脚本不在族里，记号只在它们里面的 required 行照旧报源码里没有核对",
                        ((["scripts/gate/sample-imports-shared.py"], True), (["scripts/gate/sample-quotes-sibling.py"], True)),
                        ((family_of("sample-imports-shared.py"), reported_unmarked("sample-imports-shared.py")),
                         (family_of("sample-quotes-sibling.py"), reported_unmarked("sample-quotes-sibling.py")))))
        # 载入按语法认：注释与文档串里写的不算（开关 temporary-service-family-by-text 下红，by-name 与 by-directory 下也红），字面量参数的 import_module 算
        # （by-text 与 mark-entry-only 与 by-directory 下红）；门禁阶段入口给绝对路径、lib/ 下的部分文件由调用方给候选（absolute-paths 与 mark-entry-only 下红）
        results.append(("要临时服务：拆分族：入口注释里点名、文档串里写 import 行的同目录文件不在族里", ["scripts/gate/sample-comment-named.py"], family_of("sample-comment-named.py")))
        results.append(("要临时服务：拆分族：入口按字面量参数的 importlib.import_module 载入的同目录部分文件在族里",
                        ["scripts/gate/sample-import-module.py", "scripts/gate/sample_import_module_part.py"], family_of("sample-import-module.py")))
        absolute_stage = os.path.join(work, "absolute-stage")
        results.append(("要临时服务：拆分族：门禁阶段入口给绝对路径时 lib/ 下 source 的部分文件在族里",
                        [".claude/gate.d/sample-stage.sh", ".claude/gate.d/lib/sample-stage-cells.sh"],
                        [os.path.relpath(member, absolute_stage) for member in source_family(
                            os.path.join(absolute_stage, ".claude", "gate.d", "sample-stage.sh"),
                            [os.path.join(absolute_stage, ".claude", "gate.d", "lib", "sample-stage-cells.sh")])]))
        results.append(("要临时服务：python 脚本的整条写法起头是 python3", f"python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {TEMPORARY_SERVICE_WRAPPER_MODE} -- python3 scripts/gate/sample-own-server.py --selftest",
                        temporary_service_command("scripts/gate/sample-own-server.py")))
        whole_command = temporary_service_command(needs_service)
        results.append(("要临时服务：整条写法不带 --server-binary 与尖括号占位", (False, False),
                        (TEMPORARY_SERVICE_BINARY_OPTION in whole_command, "<" in whole_command or ">" in whole_command)))
        refusal_text = temporary_service_bare_selftest([needs_service, "--selftest"], work, {}) or ""
        results.append(("要临时服务：裸 --selftest 的拒绝说明里没有尖括号占位", False, "<" in refusal_text or ">" in refusal_text))
    finally:
        lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT, lib_project_code_runs_base.TEMPORARY_SERVICE_TABLE_PATH = saved
    results.append((f"要临时服务：仓里的表（{lib_project_code_runs_base.TEMPORARY_SERVICE_TABLE_PATH}）与自证登记表、脚本源码对得上", [],
                    temporary_service_table_problems(lib_project_code_runs_base.TEMPORARY_SERVICE_TABLE_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, HOOK_REPOSITORY)))
    incident = "scripts/mutation/crates-mutation-rows.py"
    real_bare = temporary_service_bare_selftest([incident, "--selftest"], HOOK_REPOSITORY, {})
    results.append(("要临时服务：仓里 crates-mutation-rows.py 的裸 --selftest 被点名、给整条包装写法", (True, True),
                    (real_bare is not None, real_bare is not None
                     and f"{TEMPORARY_SERVICE_WRAPPER_MODE} -- python3 {incident} --selftest" in real_bare)))
    results.append(("要临时服务：仓里 crates-mutation-rows.py 的 --selftest 在包装底下放行", None,
                    temporary_service_bare_selftest([incident, "--selftest"], HOOK_REPOSITORY, environment_under_temporary_service({}))))
    # 门禁样本档（stage-selftest.sh --item 点名）：点名的阶段源码里调了临时服务核对的，外面没套包装就被点名、给整条包装写法；不要的、套了包装的不拒
    own_stage_selftest = os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
    sample_stage_words = [own_stage_selftest, stage_samples_directory, "--item", "needs-service.sh", "--item", "plain-stage.sh"]
    bare_samples = temporary_service_bare_stage_samples(sample_stage_words, work, {})
    wrapped_prefix = f"task-server/task submit -- python3 {TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT} {TEMPORARY_SERVICE_WRAPPER_MODE} -- bash "
    results.append(("要临时服务：样本档点名要临时服务的一道、外面没套：说明点名那一道（不点不要的那一道）、给整条包装写法", (True, True, True),
                    (bare_samples is not None,
                     bare_samples is not None and f"{wrapped_prefix}{shlex.join(sample_stage_words)}" in bare_samples,
                     bare_samples is not None and bare_samples.startswith("门禁样本档 needs-service.sh（"))))
    results.append(("要临时服务：样本档点名的一道要临时服务、包装剥开重判补上了标记：放行", None,
                    temporary_service_bare_stage_samples(sample_stage_words, work, environment_under_temporary_service({}))))
    results.append(("要临时服务：样本档只点名不要临时服务的一道：不拒", None,
                    temporary_service_bare_stage_samples([own_stage_selftest, stage_samples_directory, "--item", "plain-stage.sh"], work, {})))
    # 仓里的两道阶段：阶段源码原样拷进自证自己造的门禁目录、给它们建样本档目录，判法只看阶段源码里有没有核对，不看仓里的样本档目录在不在
    # （拷的源照字面路径读，算键库把这两道收进键；仓里样本档目录挪走、改名不改这两格的判定）
    real_stage_directory = os.path.join(work, "gd-repository-stages", ".claude", "gate.d")
    for real_stage in ("crates-mutation-replay.sh", "doc-text.sh"):
        os.makedirs(os.path.join(real_stage_directory, "fixtures", real_stage, "red"), exist_ok=True)
    shutil.copyfile(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "crates-mutation-replay.sh"), os.path.join(real_stage_directory, "crates-mutation-replay.sh"))
    shutil.copyfile(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "doc-text.sh"), os.path.join(real_stage_directory, "doc-text.sh"))
    real_stage_words = [own_stage_selftest, real_stage_directory, "--item", "crates-mutation-replay.sh"]
    real_samples = temporary_service_bare_stage_samples(real_stage_words, work, {})
    results.append(("要临时服务：仓里门禁 crates-mutation-replay.sh 的样本档裸写被点名、给整条包装写法", (True, True),
                    (real_samples is not None,
                     real_samples is not None and f"{wrapped_prefix}{shlex.join(real_stage_words)}" in real_samples)))
    results.append(("要临时服务：仓里门禁 doc-text.sh 的样本档不要临时服务：不拒", None,
                    temporary_service_bare_stage_samples([own_stage_selftest, real_stage_directory, "--item", "doc-text.sh"], work, {})))
    return len(results) - before


def argument_reading_cells(results):
    """阶段参数里 --check 点名的格（checked_cells）与 libtest 参数里的 --list（libtest_lists_only）怎么读：lib_run_scope.py 的范围声明闸靠这两个判法数格数、认「只列不跑」。
    格记进 results，交判了几格。弄坏开关 checked-cells-last-only、libtest-list-by-presence 红的格写在文件头开关表。"""
    before = len(results)
    for label, arguments, want in (
            ("阶段参数 --check：--check a,b 点名两格", ["--check", "a,b"], {"a", "b"}),
            ("阶段参数 --check：--check=a 的写法", ["--check=a"], {"a"}),
            ("阶段参数 --check：--check=a 与 --check b 给两次取并集", ["--check=a", "--check", "b"], {"a", "b"}),
            ("阶段参数 --check：--check 给三次（含 --check= 写法）取并集", ["--check", "a", "--check", "b,c", "--check=d"], {"a", "b", "c", "d"}),
            ("阶段参数 --check：逗号之间的空段不算一格", ["--check", "a,,b,"], {"a", "b"}),
            ("阶段参数 --check：没给 --check 交 None（全跑）", ["--item", "x", "/repo"], None),
            ("阶段参数 --check：--check 写在参数表末尾、后面缺值，交 None", ["/repo", "--check"], None),
            ("阶段参数 --check：--check 给了空串，给了但一格都没点名，交空集", ["--check", ""], set())):
        results.append((label, want, checked_cells(arguments)))
    for label, arguments, want in (
            ("libtest 参数：--list 单独一个是只列", ["--list"], True),
            ("libtest 参数：--ignored --test-threads 4 --list 是只列", ["--ignored", "--test-threads", "4", "--list"], True),
            ("libtest 参数：--skip --list（--list 是 --skip 的值）不是只列", ["--include-ignored", "--skip", "--list"], False),
            ("libtest 参数：--logfile --list（--list 是日志文件名）不是只列", ["--logfile", "--list"], False),
            ("libtest 参数：-- 之后的 --list 是过滤词，不是只列", ["--", "--list"], False),
            ("libtest 参数：-- 隔开两段之后的 --list 是过滤词，不是只列", ["--include-ignored", "the_full_case", "--", "--list"], False),
            ("libtest 参数：--skip=--list 是一个词、没有 --list 选项，不是只列", ["--ignored", "--skip=--list"], False),
            ("libtest 参数：没有 --list 不是只列", ["--include-ignored", "the_full_case"], False),
            ("libtest 参数：一个参数都没有不是只列", [], False)):
        results.append((label, want, libtest_lists_only(arguments)))
    return len(results) - before


def selftest_registry_cells(work, results):
    """自证登记表的读法：在 work 里写样本表与它点名的文件（sample-missing.sh 不写），把 SELFTEST_REGISTRY_PATH 换过去判几条命令，判完换回；
    再核仓里那份表（这份文件旁边）没有写坏的行、指到的文件在这份文件所在的仓里都在。格记进 results，交回判了几格。"""
    real_registry, real_root = lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT
    sample_registry = os.path.join(work, "sample-selftest-registry.tsv")
    with open(sample_registry, "w", encoding="utf-8") as handle:
        handle.write(SAMPLE_SELFTEST_REGISTRY)
    for relative in ("scripts/gate/sample-selftest.sh", "scripts/gate/sample-bad-kind.sh", "scripts/gate/sample-short-reason.sh", "scripts/gate/sample-pat*.sh", "elsewhere/sample-outside.sh",
                     ".claude/gate.d/sample-stage.sh", ".claude/gate.d/sample-stage-plain.sh", ".claude/gate.d/lib/sample-library.py", "elsewhere-tree/scripts/gate/sample-selftest.sh",
                     *(SAMPLE_DEVICE_LAYER + name for name in ("sample_orchestrator.py", "deeper/sample_too_deep.py", "sample_unregistered.py")),
                     "scripts/sample_top.py", ".claude/gate.d/lib-sample-library.py", ".claude/gate.d/lib-sample-unregistered.py", "task-server/sample_server_lib.py",
                     "prune-server/sample_control", "gpu-verification-server/sample_control", f"{TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT}",
                     "elsewhere-tree/" + SAMPLE_DEVICE_LAYER + "sample_orchestrator.py"):
        os.makedirs(os.path.dirname(os.path.join(work, relative)), exist_ok=True)
        with open(os.path.join(work, relative), "w", encoding="utf-8") as handle:
            handle.write("# 样本\n")
    before = len(results)
    # (说明, 命令词, 参数, 该交的行的种类；None 是不当登记过的自证)
    sample_cases = [
        ("样本表：登记过的单独 --selftest 认", "scripts/gate/sample-selftest.sh", ["--selftest"], "selftest"),
        ("样本表：带了 --selftest 之外参数的不认", "scripts/gate/sample-selftest.sh", ["--selftest", "--cells", "x"], None),
        ("样本表：不带 --selftest 的不认", "scripts/gate/sample-selftest.sh", [], None),
        ("样本表：命令词带通配的不当登记过的自证", "scripts/gate/sample-pat*.sh", ["--selftest"], None),
        ("样本表：种类写坏的一行不算登记", "scripts/gate/sample-bad-kind.sh", ["--selftest"], None),
        ("样本表：理由太短的一行不算登记", "scripts/gate/sample-short-reason.sh", ["--selftest"], None),
        ("样本表：别的树里路径尾部相同的脚本不算登记", os.path.join(work, "elsewhere-tree", "scripts", "gate", "sample-selftest.sh"), ["--selftest"], None),
        ("样本表：门禁目录顶层 lib- 起头的共用库登记过，单独 --selftest 认", ".claude/gate.d/lib-sample-library.py", ["--selftest"], "selftest"),
        ("样本表：门禁目录顶层 lib- 起头的共用库不带参数不认，只认单独一个 --selftest", ".claude/gate.d/lib-sample-library.py", [], None),
        ("样本表：门禁共用库目录里的库登记过，不带参数直接执行也认", ".claude/gate.d/lib/sample-library.py", [], "selftest"),
        ("样本表：门禁共用库目录里的库登记过，单独 --selftest 认", ".claude/gate.d/lib/sample-library.py", ["--selftest"], "selftest"),
        ("样本表：门禁共用库目录里的库带 --selftest 之外的参数不认", ".claude/gate.d/lib/sample-library.py", ["show", "HEAD", "x.tsv"], None),
        ("样本表：门禁目录顶层 lib- 起头的共用库没登记的不认", ".claude/gate.d/lib-sample-unregistered.py", ["--selftest"], None),
        ("样本表：门禁目录顶层 lib- 起头的共用库带 --selftest 之外的参数不认", ".claude/gate.d/lib-sample-library.py", ["show", "HEAD", "x.tsv"], None),
        ("样本表：装置那一层登记过的单独 --selftest 认", SAMPLE_DEVICE_LAYER + "sample_orchestrator.py", ["--selftest"], "selftest"),
        ("样本表：装置那一层没登记的 --selftest 不认", SAMPLE_DEVICE_LAYER + "sample_unregistered.py", ["--selftest"], None),
        ("样本表：装置那一层登记了而带 --cells 的不认", SAMPLE_DEVICE_LAYER + "sample_orchestrator.py", ["--selftest", "--cells", "x"], None),
        ("样本表：别的树里装置那一层路径尾部相同的不认", os.path.join(work, "elsewhere-tree", SAMPLE_DEVICE_LAYER + "sample_orchestrator.py"), ["--selftest"], None),
        # 统一任务服务那一层（task-server/）：登记了的单独 --selftest 认；`task-server/task submit -- …` 里那条带的 --selftest 不当客户端自己的自证
        # （LIB_PROJECT_CODE_RUNS_BREAK=task-server-directory-unrecognized 下头一格红；第二格对任何开关都不红，只钉这句话在「参数以 submit 起头不是单独的 --selftest」的判法下成立）
        ("样本表：task-server 那一层登记过的单独 --selftest 认", "task-server/sample_server_lib.py", ["--selftest"], "selftest"),
        ("样本表：task-server/task submit -- 里那条带 --selftest，不当客户端自己的自证", TASK_SERVER_CLIENT, ["submit", "--", "python3", "task-server/sample_server_lib.py", "--selftest"], None),
        # 判定库服务那一层（prune-server/）：同上（LIB_PROJECT_CODE_RUNS_BREAK=prune-server-directory-unrecognized 下这一格红）
        ("样本表：prune-server 那一层登记过的单独 --selftest 认", "prune-server/sample_control", ["--selftest"], "selftest"),
        # GPU 批处理验证服务那一层（gpu-verification-server/）：同上（LIB_PROJECT_CODE_RUNS_BREAK=gpu-verification-server-directory-unrecognized 下这一格红）
        ("样本表：gpu-verification-server 那一层登记过的单独 --selftest 认", "gpu-verification-server/sample_control", ["--selftest"], "selftest"),
        ("样本表：包装 --server-binary -- 里那条带 --selftest，不当包装自己的自证", f"{TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT}",
         [TEMPORARY_SERVICE_WRAPPER_MODE, TEMPORARY_SERVICE_BINARY_OPTION, "/sample/prune-server", "--", "bash", "scripts/gate/sample-selftest.sh", "--selftest"], None),
    ]
    try:
        lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT = sample_registry, work
        for label, word, arguments, want in sample_cases:
            row = registered_selftest(word, arguments, work)
            results.append((label, want, row.kind if row else None))
        found_keys = sorted(match.group(1) for problem in selftest_registry_problems(sample_registry, work)
                            for match in [re.match(r"^第 \d+ 行 (\S+)：", problem)] if match)
        results.append(("样本登记表里写坏的几行逐条报出来（种类、理由、目录、重复、文件不在、门禁阶段那几行的种类与假工具目录）", sorted(SAMPLE_SELFTEST_REGISTRY_PROBLEM_KEYS), found_keys))
        rows = read_selftest_registry(sample_registry)[0]
        stage_row, plain_row = rows.get(".claude/gate.d/sample-stage.sh"), rows.get(".claude/gate.d/sample-stage-plain.sh")
        results.append(("样本表：门禁阶段的样本档那一行带假工具目录照收", ("samples", ".sample-tools"), (stage_row.kind, stage_row.tools_directory) if stage_row else None))
        results.append(("样本表：门禁阶段的样本档那一行不带假工具目录照收", ("samples", ""), (plain_row.kind, plain_row.tools_directory) if plain_row else None))
        lib_project_code_runs_base.SELFTEST_REGISTRY_PATH = os.path.join(work, "no-such-registry.tsv")
        results.append(("登记表读不了：一律按没登记", None, registered_selftest("scripts/gate/sample-selftest.sh", ["--selftest"], work)))
    finally:
        lib_project_code_runs_base.SELFTEST_REGISTRY_PATH, lib_project_code_runs_base.SELFTEST_REGISTRY_ROOT = real_registry, real_root
    temporary_service_cells(work, results)
    results.append((f"仓里的登记表（{real_registry}）没有写坏的行、指到的文件在 {HOOK_REPOSITORY} 里都在", [],
                    selftest_registry_problems(real_registry, HOOK_REPOSITORY)))
    return len(results) - before


def selftest():
    """在临时工作区里判外部程序名单、包在命令外面的启动器、花括号与通配展开、本仓代码与编译的认法、认不出起什么的写法、登记表与门禁样本、经 task-server 起的几样；返回退出码。"""
    work = tempfile.mkdtemp(prefix="lib-project-code-runs-")
    results = []
    real_sample_index = lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY
    # 登记表那几格判的是本仓里那几份脚本：当前目录写本仓（登记表的键按解析后的真实路径认，临时工作区里同名的不算）
    repository = HOOK_REPOSITORY
    try:
        build_sample_workspace(work)
        lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = work
        lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY = os.path.join(work, SAMPLE_INDEX_REFERENCE_NAME)
        harness = os.path.join(work, "crates", "singlefs-verification")
        alias_to_harness = "CARGO_ALIAS_XT=test -p singlefs-verification"
        inner = ["herd7", "x.litmus"]

        def uses_in_process(argv, cwd):
            """一个在跑的进程（argv 与 cwd）：包装（/usr/bin/time、flock、systemd-run 这一类，gpu-quiesce-run.sh）剥掉之后里面认出的外部程序用法。"""
            launched = command_under_launcher(argv, cwd) or command_under_gpu_quiesce(argv, cwd)
            if launched is not None:
                inner_words, inner_directory = launched
                return uses_in_process(inner_words, shell_words.resolve_path(cwd, inner_directory) if inner_directory else cwd)
            return program_uses_in_command_text(shlex.join(argv), cwd)

        # 外部程序名单（文件头「外部程序名单」）：命中的算 external-program，只问版本的 herd7 放行；本仓的代码、会编的 cargo 由别的判法认，这里交 None。
        # 弄坏开关 external-programs-unlisted 下除「不在名单」的几格之外全红（应当是 external-program，实际 None）；herd7-version-flag-counted 下「只问版本」的两格红
        external_cases = [
            ("外部程序：qemu-system-x86_64", ["qemu-system-x86_64", "-nographic"], work, "external-program"),
            ("外部程序：qemu-system-aarch64 写绝对路径", ["/usr/bin/qemu-system-aarch64", "-M", "virt"], work, "external-program"),
            ("外部程序：herd7 跑 litmus", ["herd7", "x.litmus"], work, "external-program"),
            ("外部程序：herd7 带 -conf 选项", ["herd7", "-conf", "linux-kernel.cfg", "x.litmus"], work, "external-program"),
            ("外部程序：e152-file-system-benchmark 光名字从 PATH 起", ["e152-file-system-benchmark", "--device", "x"], work, "external-program"),
            ("外部程序：herd7 -version 只问版本，放行", ["herd7", "-version"], work, None),
            ("外部程序：herd7 --version 只问版本，放行", ["herd7", "--version"], work, None),
            ("外部程序：herd7 -version 之后还带别的参数不算只问版本", ["herd7", "-version", "x.litmus"], work, "external-program"),
            ("不在名单：qemu-img 查镜像信息", ["qemu-img", "info", "disk.img"], work, None),
            ("不在名单：名字只是以 herd7 起头的别的程序", ["herd7-helper", "x"], work, None),
            ("不在名单：cargo test 由 starts_a_build 与 runs_compiled_code 认", ["cargo", "test", "-p", "singlefs-core"], work, None),
            ("不在名单：echo 字面写 herd7", ["echo", "herd7", "x.litmus"], work, None),
        ]
        for label, argv, cwd, want in external_cases:
            found = classify(argv, cwd)
            results.append((label, want, found.kind if found else None))
        # lib_shell_words 前缀表之外、包在命令外面照样起那条命令的程序（command_under_launcher）：里面那条外部程序照认
        launcher_cases = [
            ("/usr/bin/time -v 包一层", ["/usr/bin/time", "-v", *inner], work, "external-program"),
            ("time -f 格式 -o 文件包一层", ["time", "-f", "%M", "-o", "/tmp/t", *inner], work, "external-program"),
            ("systemd-run --working-directory= 包一层", ["systemd-run", "--user", "--scope", "--working-directory=crates/singlefs-verification", *inner], work, "external-program"),
            ("flock -w 秒数 锁文件 包一层", ["flock", "-w", "5", "/tmp/lock", *inner], work, "external-program"),
            ("flock 锁文件 -c 字符串", ["flock", "/tmp/lock", "-c", "herd7 x.litmus"], work, "external-program"),
            ("rustup run --install stable 包一层", ["rustup", "run", "--install", "stable", *inner], work, "external-program"),
            ("chrt -i 0 包一层", ["chrt", "-i", "0", *inner], work, "external-program"),
            ("prlimit --as= 包一层", ["prlimit", "--as=8000000000", *inner], work, "external-program"),
            ("systemd-run --user --scope -p MemoryMax=8G 包一层", ["systemd-run", "--user", "--scope", "-q", "-p", "MemoryMax=8G", *inner], work, "external-program"),
            ("strace -f -o 文件包一层", ["strace", "-f", "-o", "/tmp/trace", *inner], work, "external-program"),
            ("perf stat -e 事件包一层", ["perf", "stat", "-e", "cycles", *inner], work, "external-program"),
            ("perf record -g -- 之后", ["perf", "record", "-g", "--", *inner], work, "external-program"),
            ("perf mem record 包一层", ["perf", "mem", "record", *inner], work, "external-program"),
            ("nice 里 /usr/bin/time 里 flock 一层套一层", ["nice", "-n", "19", "/usr/bin/time", "-v", "flock", "/tmp/l", *inner], work, "external-program"),
            ("strace -p 进程号：没有要起的命令", ["strace", "-p", "1234"], work, None),
            ("chrt -p 查优先级：没有要起的命令", ["chrt", "-p", "1234"], work, None),
            ("rustup show 不是 rustup run", ["rustup", "show"], work, None),
            ("perf report 不起命令", ["perf", "report", "-i", "perf.data"], work, None),
            ("/usr/bin/time 包一个不在名单的命令", ["/usr/bin/time", "-v", "true"], work, None),
            # 短选项合写：逐个字母查表，头一个要值的字母之后剩下的是值、没剩下取下一个词（开关 launcher-whole-word-options 下前四格红）
            ("strace -fo 文件（短选项合写）", ["strace", "-fo", "/tmp/s", *inner], work, "external-program"),
            ("flock -xw 10 锁文件（短选项合写）", ["flock", "-xw", "10", "/tmp/l", *inner], work, "external-program"),
            ("systemd-run --user --scope -qu 名字（短选项合写）", ["systemd-run", "--user", "--scope", "-qu", "k1", *inner], work, "external-program"),
            ("/usr/bin/time -ao 文件（短选项合写）", ["/usr/bin/time", "-ao", "/tmp/t", *inner], work, "external-program"),
            ("strace -o文件（值贴着写）", ["strace", "-o/tmp/s", *inner], work, "external-program"),
            # 要值的长选项、值另起一个词：跳过那个值（开关 launcher-long-options-partial 下前四格红）
            ("systemd-run --user --scope --expand-environment no（值另起一个词）包一层", ["systemd-run", "--user", "--scope", "--expand-environment", "no", *inner], work, "external-program"),
            ("strace --output 文件（值另起一个词）包一层", ["strace", "--output", "/tmp/s", *inner], work, "external-program"),
            ("strace --string-limit 200 包一层", ["strace", "--string-limit", "200", *inner], work, "external-program"),
            ("strace -c --summary-col calls（长选项只写前缀）包一层", ["strace", "-c", "--summary-col", "calls", *inner], work, "external-program"),
            ("strace --quiet（不带值的长选项）包一层", ["strace", "--quiet", *inner], work, "external-program"),
            ("systemd-run --user --scope --collect --same-dir（不带值的长选项）包一层", ["systemd-run", "--user", "--scope", "--collect", "--same-dir", *inner], work, "external-program"),
            ("two-logs.sh 日志出两份的包装", ["scripts/hosts/two-logs.sh", "run-one", "--", *inner], work, "external-program"),
            ("two-logs.sh --quiet 日志出两份的包装", ["scripts/hosts/two-logs.sh", "--quiet", "run-one", "--", *inner], work, "external-program"),
            ("run-build-directory.sh 这一趟自己的编译目录的包装", ["scripts/hosts/run-build-directory.sh", "--", *inner], work, "external-program"),
            ("gpu-quiesce-run.sh 清场之后起命令的包装", ["scripts/crash/gpu-quiesce-run.sh", "--", *inner], work, "external-program"),
            # 临时 prune-server 的包装：给了 --server-binary 的剥开、里面那条照认（开关 temporary-service-wrapper-unseen 下这一格红）；没给的不剥（现编或退 2，里面那条看不见）
            ("临时 prune-server 的包装给了 --server-binary 剥开里面那条",
             [f"{TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT}", TEMPORARY_SERVICE_WRAPPER_MODE, TEMPORARY_SERVICE_BINARY_OPTION, "/sample/prune-server", "--", *inner],
             work, "external-program"),
            ("临时 prune-server 的包装没给 --server-binary 不剥", [f"{TEMPORARY_SERVICE_WRAPPER_DIRECTORY}/{TEMPORARY_SERVICE_WRAPPER_SCRIPT}", TEMPORARY_SERVICE_WRAPPER_MODE, "--", *inner], work, None),
            ("systemd-run -E 设的不是别名（值的末段叫 strace 也不当包装剥）", ["systemd-run", "--user", "--scope", "-E", "TRACER=/usr/bin/strace", *inner], work, "external-program"),
        ]
        for label, argv, cwd, want in launcher_cases:
            found = uses_in_process(argv, cwd)
            results.append((label, want, found[0].kind if found else None))
        # 启动器设的环境变量带进里面那条命令（交回那条命令的最前面写 名=值，再切一遍时当它的环境变量）：
        # systemd-run -E / --setenv= / -qE 与 -p Environment= / EnvironmentFile=（开关 launcher-drops-setenv、launcher-drops-property-environment 下「True」的格红），
        # strace -E / --env（开关 strace-drops-env 下「True」的格红）
        environment_cases = [
            ("systemd-run -E 设别名", ["systemd-run", "--user", "--scope", "-E", alias_to_harness, "cargo", "xt"], True),
            ("systemd-run --setenv= 设别名", ["systemd-run", "--user", "--scope", "--setenv=" + alias_to_harness, "cargo", "xt"], True),
            ("systemd-run -qE 合写设别名", ["systemd-run", "--user", "--scope", "-qE", alias_to_harness, "cargo", "xt"], True),
            ("strace -E 设别名", ["strace", "-f", "-E", alias_to_harness, "cargo", "xt"], True),
            ("strace --env= 设别名", ["strace", "--env=" + alias_to_harness, "cargo", "xt"], True),
            ("strace --env 值另起一个词设别名", ["strace", "--env", alias_to_harness, "cargo", "xt"], True),
            ("strace -fE 合写设别名", ["strace", "-fE", alias_to_harness, "cargo", "xt"], True),
            ("strace -E 只清变量（不带 =）", ["strace", "-E", "CARGO_ALIAS_XT", "cargo", "xt"], False),
            ("systemd-run -p Environment= 设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", 'Environment="' + alias_to_harness + '"', "cargo", "xt"], True),
            ("systemd-run --property=Environment= 设两个变量、第二个是别名",
             ["systemd-run", "--user", "--wait", "--pipe", '--property=Environment=A=1 "' + alias_to_harness + '"', "cargo", "xt"], True),
            ("systemd-run --prop Environment=（长选项只写前缀）设别名", ["systemd-run", "--user", "--wait", "--pipe", "--prop", 'Environment="' + alias_to_harness + '"', "cargo", "xt"], True),
            ("systemd-run -p EnvironmentFile= 文件里设别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "alias.env"), "cargo", "xt"], True),
            ("systemd-run -p Environment= 设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "Environment=RUST_BACKTRACE=1", "cargo", "xt"], False),
            ("systemd-run -p EnvironmentFile= 文件里设的不是别名", ["systemd-run", "--user", "--wait", "--pipe", "-p", "EnvironmentFile=" + os.path.join(work, "plain.env"), "cargo", "xt"], False),
        ]
        for label, argv, want in environment_cases:
            launched = command_under_launcher(argv, work)
            results.append((f"启动器环境：{label}", want, bool(launched) and alias_to_harness in launched[0]))
        # 花括号起法照命令文本判（lib_shell_words.py 照 bash 展开引号外的花括号、命令词展开成几份时 bash 起的是头一份、其余是它的参数）：
        # 开关 brace-expansion-literal 下「花括号」开头照拒的各格红，brace-sequence-literal 下序列那两格红，brace-command-words-bare 下「花括号写在包装上」一格红，
        # quoted-braces-expanded 下引号里的两格红（应当是 None，实际 external-program）
        os.makedirs(os.path.join(work, "tools"), exist_ok=True)
        for tool in ("herd7", "herd7-helper", "nice"):
            with open(os.path.join(work, "tools", tool), "w", encoding="utf-8") as handle:
                handle.write("#!/bin/sh\n")
        for label, argv, want in (
                ("花括号：命令词里的花括号展开（herd{7,x} 起的是 herd7）", ["bash", "-c", "herd{7,x} x.litmus"], "external-program"),
                ("花括号：命令词整个写成花括号（{herd7,x.litmus}）", ["bash", "-c", "{herd7,x.litmus}"], "external-program"),
                ("花括号：序列展开（qemu-system-{x86_64,aarch64}）", ["bash", "-c", "qemu-system-{x86_64,aarch64} -nographic"], "external-program"),
                ("花括号：花括号写在包装上（{nice,-n,5} herd7 x）", ["bash", "-c", "{nice,-n,5} herd7 x.litmus"], "external-program"),
                ("花括号：命令词位置单引号 {herd7,x.litmus} 照字面放行", ["bash", "-c", "'{herd7,x.litmus}'"], None),
                ("花括号：命令词位置双引号 {herd7,x.litmus} 照字面放行", ["bash", "-c", '"{herd7,x.litmus}"'], None),
                ("花括号：参数里的花括号展开不影响放行（echo herd{7,x}）", ["bash", "-c", "echo herd{7,x}"], None)):
            found = uses_in_process(argv, work)
            results.append((label, want, found[0].kind if found else None))
        # 展开出来的本仓的代码、编译（project_code_program_use；开关 expanded-project-code-unjudged 下「展开」开头照拒的各格红）
        for label, argv, want in (
                ("展开：通配起本仓的脚本经 capped.sh（scripts/hosts/capped.s? 4 true）", ["bash", "-c", "bash scripts/hosts/capped.s? 4 true"], "project-code"),
                ("展开：花括号拼出的是只读命令不拒（{ls,-l}）", ["bash", "-c", "{ls,-l}"], None),
                ("展开：通配起文本工具 scripts/kb/replace-onc?.py 不拒（谁都能直接跑）", ["bash", "-c", "python3 scripts/kb/replace-onc?.py --selftest-never"], None)):
            found = uses_in_process(argv, repository)
            results.append((label, want, found[0].kind if found else None))
        # 没展开的词直接交 classify（命令文本里引号外的花括号 lib_shell_words.py 先展开成头一份当命令，其余是参数，到不了 classify）：
        # 命令词展开成几份的，每一份单独当命令判。开关 brace-sequence-literal 下头一格红（应当是 external-program，实际 None），expanded-project-code-unjudged 下第二格红
        for label, words, cwd, want in (
                ("花括号：数字序列展开（herd{1..9} 里有 herd7）", ["herd{1..9}", "x.litmus"], work, "external-program"),
                ("展开：花括号拼出 cargo test（{cargo,test,-p,singlefs-core}）", ["{cargo,test,-p,singlefs-core}"], repository, "project-code")):
            found = classify(words, cwd)
            results.append((label, want, found.kind if found else None))
        # 命令词带通配：按 directory 展开、按展开出来的那几份判（开关 pattern-command-words-literal 下「通配」开头照拒的各格红）
        for label, words, cwd, want in (
                ("通配：tools/her?7 展开成 tools/herd7", ["tools/her?7", "x.litmus"], work, "external-program"),
                ("通配：tools/herd* 展开成 herd7 与 herd7-helper，其中有 herd7", ["tools/herd*", "x.litmus"], work, "external-program"),
                ("通配：零命中按字面判", ["tools/no-such-*", "x.litmus"], work, None),
                ("通配：展开成不在名单的程序", ["tools/herd7-h*", "x"], work, None),
                ("通配：tools/herd[^x] 的 [^…] 照 bash 当取反", ["tools/herd[^x]", "x.litmus"], work, "external-program"),
                ("通配：POSIX 字符类 tools/her[[:alpha:]]7 交给 bash 展开", ["tools/her[[:alpha:]]7", "x.litmus"], work, "external-program"),
                ("通配：展开出来的是 nice 包装，包装里的 herd7 照认", ["tools/ni?e", "-n", "5", "herd7", "x.litmus"], work, "external-program")):
            found = classify(words, cwd)
            results.append((label, want, found.kind if found else None))
        # here-string 与管道喂给从标准输入读命令的 shell 的字面文本当命令判（lib_shell_words.py 的 shell_standard_input；
        # LIB_SHELL_WORDS_BREAK=shell-standard-input-unread 下前三格红：应当是 external-program，实际 None）
        standard_input_cases = [
            ("标准输入：here-string 喂给 bash 跑 herd7", "bash <<< 'herd7 x.litmus'", "external-program"),
            ("标准输入：printf 经管道喂给 bash 跑 herd7", "printf 'herd7 x.litmus\\n' | bash", "external-program"),
            ("标准输入：echo 经管道喂给 sh 跑 herd7", "echo 'herd7 x.litmus' | sh", "external-program"),
            ("标准输入：printf 经管道喂给 bash -c（代码不从标准输入读）不算", "printf 'herd7 x.litmus' | bash -c 'cat'", None),
        ]
        for label, text, want in standard_input_cases:
            found = program_uses_in_command_text(text, work)
            results.append((label, want, found[0].kind if found else None))
        # 命令词写在变量里：前面赋过值的换成值再判（command_word_expanded；开关 command-word-variables-unexpanded 下这一格红）
        for label, text, want in (("命令词变量：S=herd7 之后 $S 起 herd7", "S=herd7; $S x.litmus", "external-program"),
                                  ("命令词变量：没赋过值的 $T 看不出起的是什么，不算外部程序", "$T x.litmus", None)):
            found = program_uses_in_command_text(text, work)
            results.append((label, want, found[0].kind if found else None))
        # bash 起不来时带 POSIX 字符类的命令词照拒（开关 posix-brackets-unresolved-literal 与 posix-brackets-by-python-glob 下这一格红）
        real_bash, lib_project_code_runs_base.BASH_FOR_PATTERNS = lib_project_code_runs_base.BASH_FOR_PATTERNS, os.path.join(work, "no-such-bash")
        try:
            bash_missing_cases = [("bash 起不来时 POSIX 字符类起外部程序照拒", ["bash", "tools/her[[:alpha:]]7", "x.litmus"], repository, "pattern-unresolved")]
            for label, argv, cwd, want in bash_missing_cases:
                found = uses_in_process(argv, cwd)
                results.append((label, want, found[0].kind if found else None))
        finally:
            lib_project_code_runs_base.BASH_FOR_PATTERNS = real_bash
        # 花括号展开与 bash 一致：同一批词交给 bash（set -f 关掉通配，只剩花括号展开）逐个比（开关 brace-expansion-literal 下大半红，brace-sequence-literal 下序列那几格红）
        brace_words = ["{a,b}", "a{b,c}d", "{a{b,c}}", "{a,b{c,d}}e", "{a,b", "{{a,b}", "{a,b}}", "{a}", "{}", "{,}", "x{,}", "{,a}", "herd{7,x}",
                       "{a,{b}}", "a{{b}}", "a{b{c,d}", "{a,b}{", "}{a,b}", "{}a,b}", "..{},aa,}a.", "ab{}1..,}{.{", "{1..3}", "{3..1}", "{01..10}", "{1..10..3}",
                       "{a..e}", "{a..e..2}", "{1..a}", "{-2..2}", "{-01..3}", "{1..-03}", "{0..00}", "{1..05..2}", "{01..100..33}", "{100..00..50}", "{+1..3}", "{1..3..0}",
                       "{1...3}", "{a..}", "x{1..3}y{a,b}", "{1..2}{3..4}", "{a,b}{c,d}{e,f}"]
        # 词从参数交进去、eval 照字面那一行（set -- <词>）解析，与把每个词写进脚本正文同一种展开；脚本正文是字面串（门禁按条复用收闭包时折得出）
        brace_expansion_script = 'set -f\nfor word in "$@"; do\n  eval "set -- $word"\n  printf \'%s\\0\' "$#" "$@"\ndone\n'
        answer = subprocess.run(["bash", "-c", brace_expansion_script, "bash", *brace_words], capture_output=True, text=True).stdout.split("\0")
        position = 0
        try:
            for word in brace_words:
                count = int(answer[position])
                results.append((f"花括号展开与 bash 一致：{word}", answer[position + 1:position + 1 + count], brace_expansion(word)))
                position += 1 + count
        except (ValueError, IndexError):
            results.append(("花括号展开与 bash 一致：bash 的输出读得出", True, False))
        # 自证登记表的读法：换成样本表（写坏的几种行都有）判；仓里那份表另核没有写坏的行、指到的文件都在
        argument_cases = argument_reading_cells(results)
        registry_cases = selftest_registry_cells(work, results)
        # 门禁样本（文件头「门禁样本」）：stage-selftest.sh 喂的样本要登记在表里、与 SAMPLE_INDEX_REPOSITORY 的 HEAD 加暂存区逐字节相同才盯；只信本仓那一份 stage-selftest.sh。
        # 弄坏开关 stage-samples-fixtures-unchecked 下「样本」开头改过的五格红，stage-items-as-directory 下 --item / --list-items 的格红，stage-items-unjudged 下点名的几道不判的格红，
        # stage-feed-script-by-name 下别处树里拷贝不带 --item 的格红，stage-items-script-by-name 下别处树里拷贝带 --item 的格红，stage-directory-pwd-unexpanded 下 $PWD 的格红
        own_gate_selftest = os.path.join(repository, KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
        evil_gate_selftest = os.path.join(work, "evil-tree", KNOWN_SCRIPT_LOCATIONS[STAGE_SELFTEST_SCRIPT])
        fed = lambda name: os.path.join(work, name, ".claude", "gate.d")
        stage_sample_cases = [
            # (说明, 命令词, 参数, 当前目录, 环境变量, 该不该盯)
            ("样本：喂的样本多一份 setup.sh", own_gate_selftest, [fed("gd-sample-setup")], work, None, False),
            ("样本：内容改过", own_gate_selftest, [fed("gd-sample-edited")], work, None, False),
            ("样本：执行位改过", own_gate_selftest, [fed("gd-sample-mode")], work, None, False),
            ("样本：少了文件", own_gate_selftest, [fed("gd-sample-missing")], work, None, False),
            ("样本：暂存之前新加的样本（样本仓自己的阶段目录）", own_gate_selftest, [fed(SAMPLE_INDEX_REFERENCE_NAME)], work, None, False),
            ("样本档全登记、与暂存区相同的阶段目录", own_gate_selftest, [fed("gd-registered")], work, None, True),
            ("有登记表外阶段的目录", own_gate_selftest, [fed("gd-unregistered")], work, None, False),
            ("阶段目录写在变量里", own_gate_selftest, ["$GD"], work, None, False),
            ("--item 点名一道登记过的", own_gate_selftest, [fed("gd-unregistered"), "--item", "doc-text.sh"], work, None, True),
            ("--list-items 只列、不喂样本", own_gate_selftest, [fed("gd-unregistered"), "--list-items"], work, None, True),
            ("--item 点名不存在的阶段（它自己退 2、一份样本都不喂）", own_gate_selftest, [fed("gd-unregistered"), "--item", "no-such-stage.sh"], work, None, True),
            ("阶段目录写成 $PWD/…、--item 点名一道登记过的", own_gate_selftest, ["$PWD/gd-registered/.claude/gate.d", "--item", "doc-text.sh"], work, None, True),
            ("--item 与值之间夹的 --force 照 preflight 摘掉", own_gate_selftest, [fed("gd-registered"), "--item", "--force", "doc-text.sh"], work, None, True),
            ("--item 点名里夹一道登记表外的", own_gate_selftest, [fed("gd-unregistered"), "--item", "doc-text.sh", "--item", "brand-new-stage.sh"], work, None, False),
            ("--item 一个值里带空格点到登记表外的", own_gate_selftest, [fed("gd-unregistered"), "--item", "brand-new-stage.sh doc-text.sh"], work, None, False),
            ("--item 点名的样本多一份 setup.sh", own_gate_selftest, [fed("gd-sample-setup"), "--item", "doc-text.sh"], work, None, False),
            ("--item 点名写在变量里", own_gate_selftest, [fed("gd-registered"), "--item", "$STAGE"], work, None, False),
            ("阶段目录写成 $PWD/…、命令里另设了 PWD 的不换", own_gate_selftest, ["$PWD/gd-registered/.claude/gate.d", "--item", "doc-text.sh"], work, {"PWD": "/elsewhere"}, False),
            ("别处树里的 stage-selftest.sh 拷贝带 --list-items", evil_gate_selftest, [fed("gd-registered"), "--list-items"], work, None, False),
            ("别处树里的 stage-selftest.sh 拷贝 --item 点名一道", evil_gate_selftest, [fed("gd-registered"), "--item", "doc-text.sh"], work, None, False),
            ("别处树里的 stage-selftest.sh 拷贝不带 --item", evil_gate_selftest, [fed("gd-registered")], work, None, False),
            ("相对着写、当前目录不在本仓的 stage-selftest.sh 不带 --item", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", [fed("gd-registered")], work, None, False),
        ]
        for label, word, arguments, cwd, environment, want in stage_sample_cases:
            results.append((f"门禁样本：{label}", want, stage_selftest_problems(word, arguments, cwd, environment) == ""))
        refused_setup = stage_selftest_problems(own_gate_selftest, [fed("gd-sample-setup")], work)
        results.append(("门禁样本：样本与暂存区不同的说明点名那份文件", True, "red/setup.sh（暂存区里没有）" in refused_setup))
        refused_item = stage_selftest_problems(own_gate_selftest, [fed("gd-unregistered"), "--item", "doc-text.sh", "--item", "brand-new-stage.sh"], work)
        results.append(("门禁样本：--item 点名里夹登记表外的，说明点名那一道、不点没毛病的那一道", True,
                        "brand-new-stage.sh 的样本档没在登记表里" in refused_item and "doc-text.sh 的" not in refused_item))
        name_cases = [
            ("门禁阶段按名字判", ".claude/gate.d/doc-text.sh", work, True),
            ("去了编号的门禁阶段按名字判", ".claude/gate.d/lkmm.sh", work, True),
            ("门禁目录顶层的共用库 lib-*.sh 不是阶段", ".claude/gate.d/lib-sample.sh", work, False),
            ("lib/ 底下的共用库不是阶段", ".claude/gate.d/lib/stage-cells.sh", work, False),
            ("仓内位置上的 gate-staged.sh 按名字判", "scripts/gate/gate-staged.sh", work, True),
            ("仓内位置上的 mutation-shard-run.sh 按名字判", "scripts/mutation/mutation-shard-run.sh", work, True),
            ("仓内位置上的 broad-mutation.py 按名字判", "scripts/mutation/broad-mutation.py", work, True),
            ("规范副本里的 stage-selftest.sh 按名字判（放行时不读正文）", ".claude/singlefs-ai-sop/scripts/stage-selftest.sh", work, True),
            ("仓外的同名 broad-mutation.py 不按名字判", "/tmp/claude-1000/somewhere/broad-mutation.py", work, False),
            ("从 research 里相对着写的 mutate.sh", "scripts/mutation/mutate.sh", os.path.join(work, "research"), True),
            ("仓外的同名 mutate.sh 不按名字判", "/tmp/claude-1000/somewhere/mutate.sh", work, False),
            ("仓内位置上的 crash-amplification.sh 按名字判（不读正文）", "scripts/crash/crash-amplification.sh", repository, True),
            ("仓内位置上的 build-patched-qemu.sh 按名字判（不读正文）", "scripts/devices/build-patched-qemu.sh", work, True),
            ("仓内位置上的 crash-point-replay.sh 按名字判（不读正文）", "scripts/crash/crash-point-replay.sh", repository, True),
            ("没登记的脚本", "run-chain.sh", work, False),
            ("路径尾部相同的别处 crash-point-replay.sh 拷贝不按名字判（读正文）", os.path.join(work, "copy-tree", KNOWN_SCRIPT_LOCATIONS["crash-point-replay.sh"]), work, False),
        ]
        os.makedirs(os.path.join(work, "copy-tree", os.path.dirname(KNOWN_SCRIPT_LOCATIONS["crash-point-replay.sh"])), exist_ok=True)
        with open(os.path.join(work, "copy-tree", KNOWN_SCRIPT_LOCATIONS["crash-point-replay.sh"]), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\necho 样本拷贝\n")
        for label, word, directory, want in name_cases:
            results.append((label, want, judged_by_name(word, directory)))
        # 跑不跑编译出来的代码：真 / 假
        compiled_cases = [
            ("cargo test 一个测试目标", ["cargo", "test", "-p", "singlefs-core", "--test", "core_contract"], True),
            ("cargo nextest run", ["cargo", "nextest", "run", "-p", "singlefs-core"], True),
            ("cargo miri test", ["cargo", "miri", "test", "-p", "singlefs-core"], True),
            ("cargo llvm-cov 不带子命令", ["cargo", "llvm-cov", "-p", "singlefs-core"], True),
            ("cargo mutants", ["cargo", "mutants", "-p", "singlefs-core"], True),
            ("--config 定的别名", ["cargo", "--config", 'alias.xt="test"', "xt"], True),
            ("cargo nextest list 不跑", ["cargo", "nextest", "list"], False),
            ("cargo llvm-cov report 不跑", ["cargo", "llvm-cov", "report"], False),
            ("cargo t 简写", ["cargo", "t", "--lib"], True),
            ("cargo run 实验二进制", ["cargo", "run", "--release", "--bin", "e160-random-small-read-share"], True),
            ("cargo r 简写", ["cargo", "r"], True),
            ("cargo bench", ["cargo", "bench"], True),
            ("cargo bench --no-run 只编不跑", ["cargo", "bench", "--no-run"], False),
            ("cargo mutants --help 只打帮助，不起 cargo test", ["cargo", "mutants", "--help"], False),
            ("cargo mutants -V -h 只打版本与帮助，不起 cargo test", ["cargo", "mutants", "-V", "-h"], False),
            ("--config 写在子命令之后的别名", ["cargo", "xt", "--config", 'alias.xt="test"'], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "test"], True),
            ("cargo test --no-run 只编不跑", ["cargo", "test", "--no-run", "-p", "singlefs-core"], False),
            ("-- 之后的 --no-run 是测试二进制的参数", ["cargo", "test", "--", "--no-run"], True),
            ("cargo build", ["cargo", "build", "--release"], False),
            ("cargo b 简写是 build", ["cargo", "b"], False),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("直接执行测试二进制", [os.path.join("target", "debug", "deps", "overwrite_in_one_instance-0123456789abcdef")], True),
            ("直接执行 research 里的实验二进制", ["scripts/experiment-apparatus/target/release/e142_region_diff_independent"], True),
            ("直接执行名字里带 target 的自定编译目录里的二进制", ["/tmp/singlefs-crates-mutation-target/release/tiny"], True),
            ("带目标三元组的", ["./target/x86_64-unknown-linux-musl/release/new_pool_file_creation_on_device"], True),
            ("deps 下的 .d 依赖文件", [os.path.join("target", "release", "deps", "tiny.d")], False),
            ("名字只当参数", ["ls", "target/release/tiny"], False),
            ("不在 target 底下的程序", ["./prog"], False),
        ]
        for label, words, want in compiled_cases:
            results.append((f"跑编译出来的代码：{label}", want, runs_compiled_code(words, work) is not None))
        # 起不起编译（与谁起、在哪都无关，starts_a_build）：开关 build-subcommands-unjudged 下 cargo 会编的那几格红（应当是 True，实际 False），
        # build-scripts-unjudged 下 compile-then-swap.py 起编译的两格红
        build_cases = [
            ("cargo build", ["cargo", "build", "--offline", "--all-targets"], True),
            ("cargo b 简写", ["cargo", "b"], True),
            ("cargo check", ["cargo", "check", "-p", "singlefs-core"], True),
            ("cargo clippy", ["cargo", "clippy", "--all-targets"], True),
            ("cargo test --no-run 只编不跑也算", ["cargo", "test", "--offline", "--no-run", "-p", "singlefs-verification", "--release"], True),
            ("cargo doc", ["cargo", "doc", "--no-deps"], True),
            ("cargo run", ["cargo", "run", "--bin", "e160-random-small-read-share"], True),
            ("全局选项在子命令前", ["cargo", "+stable", "--offline", "-C", "crates/singlefs-core", "build"], True),
            ("--config 定的别名当会编", ["cargo", "--config", 'alias.xb="build"', "xb"], True),
            ("cargo fmt 不编", ["cargo", "fmt", "--all"], False),
            ("cargo metadata 不编", ["cargo", "metadata", "--format-version", "1", "--no-deps"], False),
            ("cargo tree 不编", ["cargo", "tree"], False),
            ("cargo clean 不编", ["cargo", "clean"], False),
            ("cargo 只带全局选项", ["cargo", "--version"], False),
            ("python3 起 compile-then-swap.py", ["python3", "-B", "scripts/hosts/compile-then-swap.py", "draft.rs", "target.rs", "--scratch", "s"], True),
            ("直接执行别处树里的 compile-then-swap.py", ["/elsewhere/scripts/hosts/compile-then-swap.py", "draft.rs", "target.rs", "--scratch", "s"], True),
            ("compile-then-swap.py 单独一个 --selftest 用假 cargo，不算", ["python3", "scripts/hosts/compile-then-swap.py", "--selftest"], False),
            ("python3 -c 的代码里写着那份脚本的名字不算", ["python3", "-c", "print('compile-then-swap.py')"], False),
            ("不起 cargo 的脚本", ["python3", "scripts/hosts/heavy_threads.py"], False),
        ]
        for label, words, want in build_cases:
            results.append((f"起编译：{label}", want, starts_a_build(words, work) is not None))

        # 一百三十七节那一批（records/2026-10-29-一百三十七节那一轮调度记录.md 件 1）：名单外启动器（开关 extra-launchers-unseen 下「启动器：」开头的格红，应当是起的那条命令，实际 None）
        extra_launcher_cases = [
            ("启动器：xargs 接在命令后面的读进来的词", ["xargs", "cargo", "test", "-p", "singlefs-verification"], ["cargo", "test", "-p", "singlefs-verification", XARGS_INPUT_WORD]),
            ("启动器：xargs -I{} 换掉替换串", ["xargs", "-I{}", "sh", "-c", "cargo test {}"], ["sh", "-c", f"cargo test {XARGS_INPUT_WORD}"]),
            ("启动器：xargs 不给命令是 echo", ["xargs", "-0"], ["echo", XARGS_INPUT_WORD]),
            ("启动器：find -exec 的命令、{} 换成找到的路径", ["find", ".", "-name", "s.sh", "-exec", "bash", "{}", ";"], ["bash", FIND_PATH_WORD]),
            ("启动器：find 两个 -exec 连成 sh -c", ["find", ".", "-exec", "true", ";", "-execdir", "cargo", "test", "+"], ["sh", "-c", "true ; cargo test"]),
            ("启动器：find -exec 没有收尾的 ; 什么都不起", ["find", ".", "-exec", "cargo", "test"], None),
            ("启动器：ssh 把 host 之后的词交给远端 shell", ["ssh", "-p", "22", "localhost", "cargo test --workspace"], ["sh", "-c", "cargo test --workspace"]),
            ("启动器：doas -u 用户", ["doas", "-u", "root", "cargo", "test"], ["cargo", "test"]),
            ("启动器：numactl -N0 贴着写", ["numactl", "-N0", "cargo", "test"], ["cargo", "test"]),
            ("启动器：numactl --membind 值另起一个词", ["numactl", "--membind", "0", "cargo", "test"], ["cargo", "test"]),
            ("启动器：script -qec 的代码", ["script", "-qec", "cargo test --workspace", "/dev/null"], ["sh", "-c", "cargo test --workspace"]),
            ("启动器：unshare --user", ["unshare", "--user", "cargo", "test"], ["cargo", "test"]),
            ("启动器：setpriv --reuid=", ["setpriv", "--reuid=1000", "cargo", "test"], ["cargo", "test"]),
            ("启动器：nsenter -t 进程号", ["nsenter", "-t", "1", "-m", "cargo", "test"], ["cargo", "test"]),
            ("启动器：su -c 的代码", ["su", "-c", "cargo test", "me"], ["sh", "-c", "cargo test"]),
            ("启动器：runuser -u 用户 --", ["runuser", "-u", "me", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：sg 组 -c", ["sg", "users", "-c", "cargo test"], ["sh", "-c", "cargo test"]),
            ("启动器：watch -n 1", ["watch", "-n", "1", "cargo", "test"], ["sh", "-c", "cargo test"]),
            ("启动器：gdb --args", ["gdb", "-batch", "--args", "cargo", "test"], ["cargo", "test"]),
            ("启动器：scl enable 集合 --", ["scl", "enable", "x", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：parallel ::: 之前是命令", ["parallel", "-j", "2", "cargo", "test", ":::", "a"], ["cargo", "test", PARALLEL_INPUT_WORD]),
            ("启动器：bwrap --bind 带两个值", ["bwrap", "--bind", "/", "/", "--dev", "/dev", "cargo", "test"], ["cargo", "test"]),
            ("启动器：ltrace -o 文件", ["ltrace", "-o", "t.log", "cargo", "test"], ["cargo", "test"]),
            ("启动器：valgrind --tool=", ["valgrind", "--tool=memcheck", "cargo", "test"], ["cargo", "test"]),
            ("启动器：faketime 时间戳", ["faketime", "now", "cargo", "test"], ["cargo", "test"]),
            ("启动器：unbuffer -p", ["unbuffer", "-p", "cargo", "test"], ["cargo", "test"]),
            ("启动器：cpulimit -l 50 --", ["cpulimit", "-l", "50", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：dbus-run-session --", ["dbus-run-session", "--", "cargo", "test"], ["cargo", "test"]),
            ("启动器：firejail --quiet", ["firejail", "--quiet", "cargo", "test"], ["cargo", "test"]),
            ("启动器：proot -r 根", ["proot", "-r", "/", "cargo", "test"], ["cargo", "test"]),
            ("启动器：chroot 新根", ["chroot", "/", "cargo", "test"], ["cargo", "test"]),
            ("启动器：cgexec -g", ["cgexec", "-g", "cpu:x", "cargo", "test"], ["cargo", "test"]),
        ]
        for label, words, want in extra_launcher_cases:
            got = command_under_launcher(words, work)
            results.append((label, want, got[0] if got else None))
        results.append(("启动器：名单外的几个与 launcher_names() 一致", True, EXTRA_LAUNCHER_NAMES <= launcher_names()))
        # 派发起法（records/2026-10-29-一百三十七节那一轮调度记录.md#8；开关 dispatch-launchers-unseen 下交回命令的那一格与「派发起法：multi-host-run.sh --peer -- <脚本> --selftest」一格红）
        dispatch, multi_host = "scripts/hosts/host-dispatch.py", "scripts/hosts/multi-host-run.sh"
        dispatch_launcher_cases = [
            ("启动器：multi-host-run.sh --peer 带 --memory-cap、--label 各一个值", [multi_host, *"--peer --memory-cap 2G --label x --".split(), "cargo", "test"],
             ["cargo", "test"]),
            ("启动器：multi-host-run.sh --selftest 是它自己的自证，后面写了 -- 也不剥", [multi_host, "--selftest", "--", "cargo", "test", "--workspace"], None),
            ("启动器：别的目录里的 multi-host-run.sh --peer 不剥", ["tools/multi-host-run.sh", "--peer", "--", "cargo", "test", "--workspace"], None),
        ]
        for label, words, want in dispatch_launcher_cases:
            got = command_under_dispatch_launcher(words, work)
            results.append((label, want, got[0] if got else None))
        foreign_dispatch = os.path.join(work, "foreign-tree", "scripts", "hosts", "multi-host-run.sh")
        os.makedirs(os.path.dirname(foreign_dispatch), exist_ok=True)
        with open(foreign_dispatch, "w", encoding="utf-8") as handle:
            handle.write('#!/usr/bin/env bash\nwhile [[ $# -gt 0 && $1 != -- ]]; do shift; done; shift; exec "$@"\n')
        for label, want, got in (
                ("派发起法：multi-host-run.sh --peer -- <脚本> --selftest 不当派发器自己的自证", None,
                 getattr(registered_selftest(os.path.join(HOOK_REPOSITORY, multi_host), ["--peer", "--label", "x", "--", "python3", "scripts/agents/dispatch-prompt.py", "--selftest"], work),
                         "kind", None)),
                ("派发起法：host-dispatch.py --selftest 照它自己的自证认（登记过）", "selftest", getattr(registered_selftest(os.path.join(HOOK_REPOSITORY, dispatch), ["--selftest"], work), "kind", None)),
                ("派发起法：--peer 剥开的那条算经内存包装", True, dispatched_under_memory_cap([multi_host, "--peer", "--", "cargo", "test"], work)),
                ("派发起法：别处树里内容不同的 multi-host-run.sh --peer 拷贝剥开的不算经内存包装", False,
                 dispatched_under_memory_cap([foreign_dispatch, "--peer", "--", "cargo", "test"], work))):
            results.append((label, want, got))
        # 认不出起什么的写法（开关 unrecognized-launches-allowed 下「认不出：」开头照拒的格红，应当是 True，实际 False）
        unrecognized_cases = [
            ("认不出：命令词是变量", ["$c", "-p", "hi"], None, True),
            ("认不出：命令词是命令替换", ["$(which cargo)", "test", "--workspace"], None, True),
            ("认不出：命令词展开出来夹着空白", ["cargo test --workspace"], None, True),
            ("认不出：bash 起的脚本是进程替换", ["<(echo cargo test)"], "shell", True),
            ("认不出：source 进程替换", ["source", "<(cat s.sh)"], None, True),
            ("认不出：cargo 子命令是命令替换", ["cargo", "$(echo test)", "--workspace"], None, True),
            ("认不出：cargo test 的参数是变量", ["cargo", "test", "$F"], None, True),
            ("认不出：cargo test 的参数是位置参数", ["cargo", "test", "$@"], None, True),
            ("认得出：cargo build 的参数是变量（不跑编出来的代码）", ["cargo", "build", "$F"], None, False),
            ("认得出：cargo test 的 --target-dir 写成变量", ["cargo", "test", "-p", "singlefs-verification", "--target-dir", "$T"], None, False),
            ("认得出：cargo test 的 -- 之后写成变量", ["cargo", "test", "-p", "singlefs-verification", "--", "$FILTER"], None, False),
            ("认得出：字面命令", ["ls", "$HOME"], None, False),
            ("认得出：命令词是 $CLAUDE_CODE_EXECPATH（检出钩子 ⑬ 判起 claude）", ["$CLAUDE_CODE_EXECPATH", "-p", "hi"], None, False),
            ("认得出：命令词是 ${CLAUDE_CODE_EXECPATH}（检出钩子 ⑬ 判起 claude）", ["${CLAUDE_CODE_EXECPATH}", "--print", "hi"], None, False),
        ]
        for label, words, launcher, want in unrecognized_cases:
            results.append((label, want, unrecognized_launch(words, launcher) is not None))
        # 起 claude 的那一组与检出钩子同一组：只读定义它的那一份部分文件（字面路径，算键库收得进键），取它那一行的字面集合；那一行挪去别的部分文件的，这一格交 None 红
        detector_constant_path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "bash_command_detector_scope_and_index.py")
        with open(detector_constant_path, encoding="utf-8") as detector_source_handle:
            detector_words = re.search(r"^CLAUDE_EXECUTABLE_PATH_WORDS = (\{.*\})$", detector_source_handle.read(), re.MULTILINE)
        results.append(("认得出：起 claude 的命令词与检出钩子 bash_command_detector_scope_and_index.py 的 CLAUDE_EXECUTABLE_PATH_WORDS 同一组", set(CLAUDE_EXECUTABLE_PATH_WORDS),
                        set(ast.literal_eval(detector_words.group(1))) if detector_words else None))
        # 读进来的脚本正文里经变量起的自证、门禁阶段、入口脚本、cargo（一百三十七节件 1b 第 3 条；开关 script-body-launches-unjudged 下「照认不出」的三格红，应当是 True，实际 False）
        draft_script = os.path.join(os.sep, "draft-outside-the-repositories", "runs.sh")   # work 在这一段照「本仓」认（FAMILY_OWN_REPOSITORY），草稿脚本取在它外面
        body_cases = [
            ("脚本正文：数组变量的定义里写着 --selftest，照认不出", ["${cmds[$i]}"], None, {"cmds": ["(bash $R/.claude/hooks/x.sh --selftest)"], "i": ["${!cmds[@]}"]}, (), draft_script, True),
            ("脚本正文：\"$@\" 转发、起它的参数里写着 --selftest，照认不出", ["$@"], None, {}, ("bash", "/tree/.claude/hooks/x.sh", "--selftest"), draft_script, True),
            ("脚本正文：cargo test 的参数是解不出的变量，照认不出", ["cargo", "test", "$F"], None, {"F": ["$(cat f)"]}, (), draft_script, True),
            ("脚本正文：\"$@\" 转发、参数里没有这几样，不判", ["$@"], None, {}, ("ls", "-l"), draft_script, False),
            ("脚本正文：bash 起的 $X 定义里只有普通脚本，不判", ["$X"], "shell", {"X": ["/tree/report.sh"]}, (), draft_script, False),
            ("脚本正文：脚本起它自己（$0 --selftest），不判", ["$0", "--selftest"], "shell", {}, (), draft_script, False),
            ("脚本正文：本仓里的脚本不判", ["${cmds[$i]}"], None, {"cmds": ["(bash $R/x.sh --selftest)"]}, (), os.path.join(HOOK_REPOSITORY, "scripts", "hosts", "multi-host-run.sh"), False),
        ]
        for label, words, launcher, definitions, positional, script_path, want in body_cases:
            results.append((label, want, script_body_unresolved_launch(words, launcher, definitions, positional, script_path) is not None))
        # cargo 参数写成前面赋过值的变量（开关 cargo-arguments-unexpanded 下这格红）
        results.append(("cargo 参数展开：F=--workspace 之后 cargo test $F", ["cargo", "test", "--workspace"],
                        cargo_arguments_expanded(["cargo", "test", "$F"], {"F": "--workspace"})))
        # 读正文时按脚本所在目录 source 的库换成字面路径（开关 sourced-libraries-unread 下这格红）
        results.append(("source 的库：$(dirname \"${BASH_SOURCE[0]}\") 换成脚本所在目录", 'source "/tree/scripts/../lib.sh"',
                        script_directory_expanded('source "$(dirname "${BASH_SOURCE[0]}")/../lib.sh"', "/tree/scripts/x.sh")))
        # 同族拷贝比库时认 source 的写法（shell_source_lines）：仓里用到的几种逐种认出，赋值不一致的变量、认不出的变量交 None，注释与 source = 不算
        # （开关 script-directory-variables-skipped 下只剩直接接字面路径的那一条，这一格红）
        results.append(("source 的库：经变量与顶层 cd 引的几种写法逐种认出，赋值不一致的、认不出的变量交 None（shell_source_lines）", SHELL_SOURCE_FORMS_EXPECTED,
                        [(written, location) for _number, written, location in shell_source_lines(SHELL_SOURCE_FORMS_SAMPLE, "entry.sh")[0]]))
        # 库里看得到引它的入口设的变量；认不出又没登记的那一行交回（same_content_as_own 照它读正文）、不静默跳过
        # （开关 unrecognized-sources-skipped 下认不出的那一行不交回、script-directory-variables-skipped 下两份库都认不出，这一格红）
        reading_root = os.path.join(work, "source-reading", ".claude", "gate.d")
        os.makedirs(os.path.join(reading_root, "lib"))
        for reading_file, reading_text in (("entry.sh", 'STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"\nsource "$STAGE_DIR/lib/part.sh"\n'),
                                           ("lib/part.sh", 'source "$STAGE_DIR/lib/inner.sh"\nsource "$UNKNOWN/unknown.sh"\n'), ("lib/inner.sh", "# 样本\n")):
            with open(os.path.join(reading_root, reading_file), "w", encoding="utf-8") as handle:
                handle.write(reading_text)
        sample_reading = script_source_reading(os.path.join(reading_root, "entry.sh"))
        results.append(("source 的库：库里看得到引它的入口设的变量，认不出又没登记的那一行交回、不静默跳过（script_source_reading）",
                        (["lib/part.sh", "lib/inner.sh"], [("lib/part.sh", 2, '"$UNKNOWN/unknown.sh"')]), (sample_reading.libraries, sample_reading.unrecognized)))
        # 同族脚本只认本仓那一份（第九十八节 ②；开关 same-family-by-path-tail 下「同族：」开头三格红，应当是 False / broad-mutation，实际 True / None）
        lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = None
        try:
            elsewhere = os.path.join(work, "family-elsewhere")
            for relative in (".claude/gate.d/doc-text.sh", "scripts/mutation/mutate.sh", "scripts/mutation/broad-mutation.py"):
                os.makedirs(os.path.dirname(os.path.join(elsewhere, relative)), exist_ok=True)
                with open(os.path.join(elsewhere, relative), "w", encoding="utf-8") as handle:
                    handle.write("cargo test --workspace\n")
            results.append(("同族：别处树里的门禁阶段拷贝不按名字判（读正文）", False, judged_by_name(".claude/gate.d/doc-text.sh", elsewhere)))
            results.append(("同族：别处树里的 mutate.sh 拷贝不按名字判（读正文）", False, judged_by_name("scripts/mutation/mutate.sh", elsewhere)))
            results.append(("同族：本仓那一份 mutate.sh 照旧按名字判", True, judged_by_name("scripts/mutation/mutate.sh", repository)))
            # 与本仓那一份逐字节相同的拷贝照本仓那一份认（family-copy-content-unchecked 下这格红）
            identical = os.path.join(work, "family-identical")
            os.makedirs(os.path.join(identical, "scripts", "mutation"), exist_ok=True)
            shutil.copyfile(os.path.join(HOOK_REPOSITORY, KNOWN_SCRIPT_LOCATIONS["mutate.sh"]), os.path.join(identical, "scripts", "mutation", "mutate.sh"))
            results.append(("同族：与本仓那一份逐字节相同的 mutate.sh 拷贝照按名字判", True, judged_by_name("scripts/mutation/mutate.sh", identical)))
            # 门禁阶段与派发起法的拷贝同一口径：本仓那一份的 sha256 照 own_script_digest 从字面表里挑（family-copy-content-unchecked 下这两格红；
            # own-script-digest-stages-unlisted 下门禁阶段那一格红）
            os.makedirs(os.path.join(identical, GATE_STAGE_DIRECTORY), exist_ok=True)
            shutil.copyfile(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "doc-text.sh"), os.path.join(identical, GATE_STAGE_DIRECTORY, "doc-text.sh"))
            results.append(("同族：与本仓那一份逐字节相同的门禁阶段拷贝照按名字判", True, judged_by_name(".claude/gate.d/doc-text.sh", identical)))
            os.makedirs(os.path.join(identical, os.path.dirname(multi_host)), exist_ok=True)
            shutil.copyfile(os.path.join(HOOK_REPOSITORY, multi_host), os.path.join(identical, multi_host))
            results.append(("派发起法：与本仓那一份逐字节相同的 multi-host-run.sh --peer 拷贝剥开的算经内存包装", True,
                            dispatched_under_memory_cap([os.path.join(identical, multi_host), "--peer", "--", "cargo", "test"], work)))
            # 连同 source 的库一起比（本仓那一边的库照 own_library_digest 从 OWN_SCRIPT_LIBRARY_FILES 里挑）：crash-point-replay.sh 与它引的每一份库原样拷过去照按名字判，
            # 库改过一份的读正文（family-copy-content-unchecked、own-script-digest-stages-unlisted、own-library-digest-unlisted 下前一格红；same-family-by-path-tail 下后一格红）
            for library_tree in ("family-with-libraries", "family-stale-library"):
                for family_library_copied in FAMILY_LIBRARY_PROBE_FILES:
                    os.makedirs(os.path.dirname(os.path.join(work, library_tree, family_library_copied)), exist_ok=True)
                    shutil.copyfile(os.path.join(HOOK_REPOSITORY, family_library_copied), os.path.join(work, library_tree, family_library_copied))
            with open(os.path.join(work, "family-stale-library", ".claude", "gate.d", "lib", "stage-cells.sh"), "a", encoding="utf-8") as handle:
                handle.write("# 样本：这一份库与本仓那一份不同\n")
            results.append(("同族：连同 source 的库都与本仓那一份逐字节相同的门禁阶段拷贝照按名字判（crash-point-replay.sh 与它 lib/ 下的库）", True,
                            judged_by_name(".claude/gate.d/crash-point-replay.sh", os.path.join(work, "family-with-libraries"))))
            results.append(("同族：source 的库与本仓那一份不同的门禁阶段拷贝不按名字判（读正文）", False,
                            judged_by_name(".claude/gate.d/crash-point-replay.sh", os.path.join(work, "family-stale-library"))))
            # 经变量 source 的库（`X="$(cd "$(dirname "$0")" && pwd)"` 再 `source "$X/…"`）照样比：入口连同它引的每一份库拷过去、只改点名的那一份，
            # 门禁阶段的拷贝不按名字判、派发起法剥开的不算经内存包装（开关 script-directory-variables-skipped 下四格红；same-family-by-path-tail 下门禁阶段那三格红）
            for stale_tree_name, stale_location, stale_library in STALE_VARIABLE_LIBRARY_PROBES:
                stale_tree = os.path.join(work, "family-stale-variable-library", stale_tree_name)
                copy_with_own_libraries(stale_location, stale_tree)
                os.makedirs(os.path.dirname(os.path.join(stale_tree, stale_library)), exist_ok=True)
                with open(os.path.join(stale_tree, stale_library), "a", encoding="utf-8") as handle:
                    handle.write("# 样本：这一份库与本仓那一份不同\n")
                if os.path.dirname(stale_location) == GATE_STAGE_DIRECTORY:
                    results.append((f"同族：{stale_location} 经变量 source 的 {stale_library} 与本仓那一份不同的拷贝不按名字判（读正文）", False,
                                    judged_by_name(stale_location, stale_tree)))
                else:
                    results.append((f"派发起法：{stale_location} 经变量 source 的 {stale_library} 与本仓那一份不同的拷贝剥开的不算经内存包装", False,
                                    dispatched_under_memory_cap([os.path.join(stale_tree, stale_location), "--peer", "--", "cargo", "test"], work)))
            # 先加 source 行、后建库（两步）：样本仓里这一道阶段末尾加一行按自己所在目录 source 的新库，库表里也点名了它；本仓那一边换成样本仓（own_repository_swapped）。
            # 库还没建时本仓那一份引的库盘上没有，逐字节相同的拷贝照不同认、读正文；两边都建了、逐字节相同之后照按名字判
            # （same-family-by-path-tail 下「库还没建」那一格红；family-copy-content-unchecked、own-script-digest-stages-unlisted、own-library-digest-unlisted、own-library-table-short 下「后建库」那一格红）
            two_step_own, two_step_copy = os.path.join(work, "two-step-own"), os.path.join(work, "two-step-copy")
            copy_with_own_libraries(TWO_STEP_STAGE, two_step_own)
            with open(os.path.join(two_step_own, TWO_STEP_STAGE), "a", encoding="utf-8") as handle:
                handle.write(TWO_STEP_SOURCE_LINE)
            shutil.copytree(two_step_own, two_step_copy)
            with own_repository_swapped(two_step_own, OWN_SCRIPT_LIBRARY_FILES + (TWO_STEP_LIBRARY,)):
                results.append(("同族：先加 source 行、库还没建：本仓那一份引的库盘上没有，逐字节相同的拷贝不按名字判（读正文）", False,
                                judged_by_name(TWO_STEP_STAGE, two_step_copy)))
                for two_step_root in (two_step_own, two_step_copy):
                    with open(os.path.join(two_step_root, TWO_STEP_LIBRARY), "w", encoding="utf-8") as handle:
                        handle.write("# 样本：后建的库\n")
                results.append(("同族：先加 source 行、后建库：两边都建好、逐字节相同的拷贝照按名字判", True, judged_by_name(TWO_STEP_STAGE, two_step_copy)))
            # 认 source 行的几种写法、认不出又没登记的照不同判、登记按行认、库的深度（source_reading_cells）
            source_reading_cases = source_reading_cells(work, results)
        finally:
            lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = work
        # 本仓那一份从哪几张字面表里挑（own_script_digest）：门禁目录顶层的阶段一道不落地写在 GATE_STAGE_FILES 里，按名字判的每一份、每一道阶段、派发起法的每一份都挑得到
        # （own-script-digest-stages-unlisted 下「门禁目录顶层列出来的阶段…」「按位置都挑得到」两格红：前一格每一道都是 False，后一格列出每一道阶段）。
        # 门禁目录顶层的阶段照 glob 列（*.sh，gate_stage_name 认的）、逐道读盘上那一份的 sha256：算键库把列出来的每一份收进这一条判法的键，加一道阶段键就变
        listed_stage_digests = {}
        for listed_stage_path in sorted(glob.glob(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY, "*.sh"))):
            if gate_stage_name(os.path.join(GATE_STAGE_DIRECTORY, os.path.basename(listed_stage_path)), None) == os.path.basename(listed_stage_path) \
                    and os.path.isfile(listed_stage_path):
                listed_stage_digests[os.path.basename(listed_stage_path)] = file_sha256(os.path.join(HOOK_REPOSITORY, GATE_STAGE_DIRECTORY,
                                                                                                    os.path.basename(listed_stage_path)))
        listed_stages = sorted(listed_stage_digests)
        results.append(("同族：门禁目录顶层列出来的阶段与 GATE_STAGE_FILES 逐个相同、每一道从表里挑出的 sha256 与盘上那一份相同",
                        [(listed_stage, True) for listed_stage in sorted(GATE_STAGE_FILES)],
                        [(listed_stage, own_script_digest(os.path.join(GATE_STAGE_DIRECTORY, listed_stage)) == listed_stage_digests[listed_stage])
                         for listed_stage in listed_stages]))
        own_locations = (sorted(set(KNOWN_SCRIPT_LOCATIONS.values())) + [os.path.join(GATE_STAGE_DIRECTORY, name) for name in listed_stages]
                         + [os.path.join(DISPATCH_LAUNCHER_DIRECTORY, name) for name in sorted(DISPATCH_LAUNCHER_OPTIONS)])
        results.append(("同族：本仓那一份的 sha256 按位置都挑得到（KNOWN_SCRIPT_LOCATIONS 的每个值、门禁目录顶层的每一道阶段、DISPATCH_LAUNCHER_OPTIONS 的每个键）", [],
                        [location for location in own_locations if own_script_digest(location) is None]))
        results.append(("同族：字面表之外的位置不读、交 None（scripts/hosts/two-logs.sh 在盘上也一样）", None, own_script_digest("scripts/hosts/two-logs.sh")))
        # 本仓那一份照表里的字面路径读、不拿传进来的位置拼（own_script_digest、own_library_digest）：传一个与表里那一项相等、字面却指到 work 底下的位置（EqualLocation），
        # 读到的照样是本仓那一份。弄坏开关 own-script-digest-location-joined 下换成先查表、查中拿传进来的位置拼路径读的那一种（location_joined_digest），这一格列出三份脚本；
        # own-script-digest-stages-unlisted 下列出门禁阶段那一份，own-library-digest-unlisted 下列出每张库表那一份（三份库）
        equal_location_root = os.path.join(work, "equal-location-elsewhere")
        if break_is_set("own-script-digest-location-joined"):
            probed_digests = [location_joined_digest(EqualLocation(probe_script, equal_location_root)) for probe_script in OWN_DIGEST_PROBE_SCRIPTS]
        else:
            probed_digests = [own_script_digest(EqualLocation(probe_script, equal_location_root)) for probe_script in OWN_DIGEST_PROBE_SCRIPTS]
        probed_digests += [own_library_digest(EqualLocation(probe_library, equal_location_root)) for probe_library in OWN_DIGEST_PROBE_LIBRARIES]
        read_digests = ([file_sha256(os.path.join(HOOK_REPOSITORY, probe_script)) for probe_script in OWN_DIGEST_PROBE_SCRIPTS]
                        + [file_sha256(os.path.join(HOOK_REPOSITORY, probe_library)) for probe_library in OWN_DIGEST_PROBE_LIBRARIES])
        probed_locations = OWN_DIGEST_PROBE_SCRIPTS + OWN_DIGEST_PROBE_LIBRARIES
        results.append(("同族：本仓那一份照表里的字面路径读，不拿传进来的位置拼（与表里那一项相等、字面指到别处的位置照样读到本仓那一份）", [],
                        [probed_locations[position] for position in range(len(probed_locations))
                         if probed_digests[position] is None or probed_digests[position] != read_digests[position]]))
        # 同族拷贝比库时本仓那一边逐张从几张库表里挑（own_library_digest），几张拼成的整张 OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本正文现算的库（script_source_reading，连同登记的认不出的那几行点名的库，
        # 相对仓根；盘上有没有都算）逐个相同，表里每一份都在本仓盘上（读得到 sha256），每份脚本引的库都在深度上限之内读完（cut 是空的）。
        # 开关 own-library-table-short 下比的表少 OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK 那一项，own-library-table-phantom 下比的表多出盘上没有的 OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK，
        # library-depth-limit-lowered 下「读完」那一项照 SOURCED_LIBRARY_LOWERED_DEPTH 读，这一格红
        table_script_locations = (sorted(set(KNOWN_SCRIPT_LOCATIONS.values())) + [os.path.join(GATE_STAGE_DIRECTORY, table_stage) for table_stage in GATE_STAGE_FILES]
                                  + [os.path.join(DISPATCH_LAUNCHER_DIRECTORY, table_launcher) for table_launcher in DISPATCH_LAUNCHER_SCRIPTS])
        libraries_sourced_by_table_scripts, unregistered_sources, registered_sources, accounted_source_lines = set(), set(), set(), set()
        table_scripts_cut_by_depth = []
        for table_script in table_script_locations:
            table_reading = script_source_reading(os.path.join(HOOK_REPOSITORY, table_script), table_script)
            libraries_sourced_by_table_scripts.update(os.path.normpath(os.path.join(os.path.dirname(table_script), relative)) for relative in table_reading.libraries)
            unregistered_sources.update(f"{os.path.normpath(os.path.join(os.path.dirname(table_script), relative))}:{number} {written}"
                                        for relative, number, written in table_reading.unrecognized)
            registered_sources.update(table_reading.registered)
            for relative, number, written, located, ordinal in table_reading.lines:
                read_file = os.path.normpath(os.path.join(os.path.dirname(table_script), relative))
                if located is not None or (read_file, written, ordinal) in OWN_SCRIPT_UNRECOGNIZED_SOURCES:
                    accounted_source_lines.add((read_file, number))
            depth_reading = (script_source_reading(os.path.join(HOOK_REPOSITORY, table_script), table_script, SOURCED_LIBRARY_LOWERED_DEPTH)
                             if break_is_set("library-depth-limit-lowered") else table_reading)
            if depth_reading.cut:
                table_scripts_cut_by_depth.append(table_script)
        compared_library_table = [listed for listed in OWN_SCRIPT_LIBRARY_FILES
                                  if not (listed == OWN_SCRIPT_LIBRARY_DROPPED_BY_BREAK and break_is_set("own-library-table-short"))]
        listed_libraries_missing = [listed for listed in OWN_SCRIPT_LIBRARY_FILES if file_sha256(os.path.join(HOOK_REPOSITORY, listed)) is None]
        if break_is_set("own-library-table-phantom"):
            compared_library_table.append(OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK)
            if file_sha256(os.path.join(HOOK_REPOSITORY, OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK)) is None:
                listed_libraries_missing.append(OWN_SCRIPT_LIBRARY_PHANTOM_BY_BREAK)
        results.append(("同族：OWN_SCRIPT_LIBRARY_FILES 与三张表里每份脚本引的库逐个相同，表里每一份都在本仓盘上，每份脚本引的库都在深度上限之内读完",
                        (sorted(libraries_sourced_by_table_scripts), [], []),
                        (sorted(compared_library_table), listed_libraries_missing, table_scripts_cut_by_depth)))
        # 每张库表的份数不超过 OWN_SCRIPT_LIBRARY_TABLE_LIMIT（own_library_digest 照每一张拼出的读路径，算键库都折得出：lib_project_code_runs_base.py 里那个常量上方的注释），
        # 量的几张按次序拼起来就是整张 OWN_SCRIPT_LIBRARY_FILES（拼整张的那一处多一张、这里漏量它时这一项红）。弄坏开关 own-library-tables-merged 下量拼成的整张一张，这一格红
        measured_library_tables = ((("OWN_SCRIPT_LIBRARY_FILES", OWN_SCRIPT_LIBRARY_FILES),) if break_is_set("own-library-tables-merged")
                                   else OWN_SCRIPT_LIBRARY_NAMED_TABLES)
        results.append((f"库表：每张库表不超过 {OWN_SCRIPT_LIBRARY_TABLE_LIMIT} 份（算键库折得出 own_library_digest 逐张读的是哪几份库），量的几张拼起来就是整张 OWN_SCRIPT_LIBRARY_FILES",
                        ([], True),
                        ([(table_name, len(table)) for table_name, table in measured_library_tables if len(table) > OWN_SCRIPT_LIBRARY_TABLE_LIMIT],
                         tuple(listed for _table_name, table in measured_library_tables for listed in table) == OWN_SCRIPT_LIBRARY_FILES)))
        # 认不出写法的 source 行（script_source_reading 的 unrecognized）都登记了，登记的每一条在那几份正文里都还读得到（开关 script-directory-variables-skipped 下登记的那几行不再交回，
        # quoted-shell-sources-skipped 下引号里那几条不再交回，unrecognized-sources-registered-by-argument 下排在后面的登记用不到，这一格红）
        results.append(("同族：三张表里的脚本与它们引的库里，认不出写法的 source 行都登记在 OWN_SCRIPT_UNRECOGNIZED_SOURCES 里、登记的每一条都还在", ([], []),
                        (sorted(unregistered_sources), sorted(set(OWN_SCRIPT_UNRECOGNIZED_SOURCES) - registered_sources))))
        # 不经识别函数的核法：三张表里的脚本与 OWN_SCRIPT_LIBRARY_FILES 里的 shell 库逐行读，INDEPENDENT_SOURCE_LINE 认得的每一行（注释行不算）都要是上面认出或登记过的行
        # （识别函数漏认一种写法时这里多出那几行；开关 script-directory-variables-skipped、quoted-shell-sources-skipped 下这一格红）
        counted_source_lines = set()
        for counted_file in table_script_locations + list(OWN_SCRIPT_LIBRARY_FILES):
            if counted_file.endswith(".py"):
                continue
            try:
                with open(os.path.join(HOOK_REPOSITORY, counted_file), encoding="utf-8", errors="replace") as handle:
                    counted_text = handle.read()
            except OSError:
                continue
            counted_source_lines.update((counted_file, number) for number, line in enumerate(counted_text.split("\n"), 1)
                                        if not line.lstrip().startswith("#") and INDEPENDENT_SOURCE_LINE.search(line))
        results.append(("同族：三张表里的脚本与 OWN_SCRIPT_LIBRARY_FILES 里的 shell 库，命令位置上与 eval、-c 串里起头的 source / . 每一行都认出了或登记过（不经识别函数逐行数）", [],
                        sorted(f"{counted_file}:{number}" for counted_file, number in counted_source_lines - accounted_source_lines)))
        # 经 task-server 起（文件头「经 task-server 起」）：本仓的代码怎么认、名单上直接跑的、submit 后面那条、绕开服务的写法。
        # 开关 project-scripts-unjudged 下「本仓代码：」照拒的那几格红（应当是 nobody，实际 None）；text-tools-refused 下「名单：」谁都能跑的几格红（应当是 anyone，实际 nobody）；
        # main-agent-tools-refused 下「名单：主 agent」放行那几格红，active-sessions-unlisted 下「名单：主 agent 查活动会话的 active-sessions.py」一格红；task-server-submission-unparsed 下「提交：」认出的三格红（实际 None）；
        # task-server-paths-unjudged 下「绕开：」点到库、密钥、socket 的三格红（应当是 True，实际 False）；task-server-library-loads-unjudged 下载入共用库的那几格红，
        # 载入那一判别的几条开关各红哪几格写在文件头开关表；admitted-variable-unjudged 下「绕开：」设准入开关的两格红；
        # task-server-scripts-unjudged 下「绕开：直接起服务本体」一格红；task-server-units-unjudged 下「绕开：systemctl」照拒的三格红；
        # task-server-builds-unjudged 下「绕开：」删编译目录的七格红（应当是 True，实际 False）；text-tools-refused 下「名单：提交客户端的 clean」一格红
        copy_root = os.path.join(work, "task-server-copy")
        for relative in ("scripts/kb/replace-once.py", TASK_SERVER_CLIENT):
            os.makedirs(os.path.dirname(os.path.join(copy_root, relative)), exist_ok=True)
            if os.path.isfile(os.path.join(HOOK_REPOSITORY, relative)):
                shutil.copyfile(os.path.join(HOOK_REPOSITORY, relative), os.path.join(copy_root, relative))
        changed_root = os.path.join(work, "task-server-changed-copy")
        os.makedirs(os.path.join(changed_root, "scripts", "kb"), exist_ok=True)
        with open(os.path.join(changed_root, "scripts", "kb", "replace-once.py"), "w", encoding="utf-8") as handle:
            handle.write("import subprocess\nsubprocess.run(['cargo', 'test', '--all'])\n")
        os.makedirs(os.path.join(changed_root, "task-server"), exist_ok=True)
        with open(os.path.join(changed_root, TASK_SERVER_CLIENT), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env python3\nimport subprocess, sys\nsubprocess.run(sys.argv[sys.argv.index('--') + 1:])\n")
        with open(os.path.join(work, "own-draft-helper.sh"), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 草稿里自己的辅助脚本\necho hi\n")
        with open(os.path.join(work, "renamed-with-header.py"), "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env python3\n# admission: always 样本\n# run-condition: none 样本\nprint(1)\n")
        def project_level(words, directory, launcher):
            found = project_code_use(words, directory, launcher)
            return None if found is None else found.allowed
        for label, words, directory, launcher, want in (
                ("本仓代码：python3 起本仓的脚本", ["scripts/gate/hook-swap.py", "--dry-run"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：bash 起本仓的钩子（喂钩子）", [".claude/hooks/write-guard.sh"], HOOK_REPOSITORY, "shell", "nobody"),
                ("本仓代码：仓副本里改过正文的文本工具不算名单上的", ["scripts/kb/replace-once.py", "a", "b", "c"], changed_root, "python", "nobody"),
                ("本仓代码：改名拷走、文件头带准入声明", ["renamed-with-header.py"], work, "python", "nobody"),
                ("本仓代码：python3 -m pytest 起单测", ["pytest.py", "scripts/hosts/tests"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：文本工具带 --selftest 不在名单的参数里", ["scripts/kb/replace-once.py", "--selftest"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：work-directory-ownership.sh 的 sweep 不在名单的参数里", ["scripts/hosts/work-directory-ownership.sh", "sweep"], HOOK_REPOSITORY, "shell", "nobody"),
                ("本仓代码：gate-overlap.py 不带 --list", [".claude/singlefs-ai-sop/scripts/gate-overlap.py", "."], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：minimal-copy.py 带 --selftest 不在名单的参数里", ["scripts/hosts/minimal-copy.py", "--selftest"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：active-sessions.py 带 --selftest 不在名单的参数里", [ACTIVE_SESSIONS_SCRIPT, "--selftest"], HOOK_REPOSITORY, "python", "nobody"),
                ("本仓代码：lib_repository_copy.py 的 link-into 不在名单的参数里",
                 ["scripts/hosts/lib_repository_copy.py", "link-into", "/x", "--root", HOOK_REPOSITORY, "--workspace", "scripts/experiment-apparatus"],
                 HOOK_REPOSITORY, "python", "nobody"),
                ("名单：replace-once.py 定点改", ["scripts/kb/replace-once.py", "a", "b", "c"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：仓副本里逐字节相同的 replace-once.py", ["scripts/kb/replace-once.py", "a", "b", "c"], copy_root, "python", "anyone"),
                ("名单：认领草稿目录", ["scripts/hosts/work-directory-ownership.sh", "claim", "/x"], HOOK_REPOSITORY, "shell", "anyone"),
                ("名单：gate-overlap.py --list", [".claude/singlefs-ai-sop/scripts/gate-overlap.py", "--list"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：提交客户端的 wait", [TASK_SERVER_CLIENT, "wait", "7"], HOOK_REPOSITORY, None, "anyone"),
                ("名单：提交客户端的 clean", [TASK_SERVER_CLIENT, "clean", "7"], HOOK_REPOSITORY, None, "anyone"),
                ("名单：task-server/control check", ["task-server/control", "check"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：task-server/control bookings", ["task-server/control", "bookings"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：task-server/control key-stats", ["task-server/control", "key-stats"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：改标签的 relabel-item.py", ["scripts/kb/relabel-item.py", "D19", "6", "--dry-run"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：备份原文件的 copy-originals.py", ["scripts/agents/copy-originals.py", "copy", "/x", "a.md"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：直接起 ledger-append-section.py 不拒", ["scripts/kb/ledger-append-section.py", "--title", "x", "--body-file", "-", "--dry-run"],
                 HOOK_REPOSITORY, "python", "anyone"),
                ("名单：直接起 append-record.py 不拒", ["scripts/kb/append-record.py", "records/x.md", "-", "--dry-run"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：直接起 quote-rust-items.py 不拒", ["scripts/kb/quote-rust-items.py", "crates/x/src/lib.rs::item"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：建仓副本的 minimal-copy.py", ["scripts/hosts/minimal-copy.py", HOOK_REPOSITORY, "/x/copy", "--with-test-inputs"], HOOK_REPOSITORY, "python", "anyone"),
                ("名单：打副本闭包的 lib_repository_copy.py entries", ["scripts/hosts/lib_repository_copy.py", "entries", HOOK_REPOSITORY, "--test-inputs"],
                 HOOK_REPOSITORY, "python", "anyone"),
                ("名单：主 agent 停进程的 proc.py", [".claude/singlefs-ai-sop/scripts/proc.py", "stop", "1"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control stop", ["task-server/control", "stop"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control pause", ["task-server/control", "pause", "--reason", "x"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control recognized-card-lease", ["task-server/control", "recognized-card-lease", "show"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 task-server/control resume", ["task-server/control", "resume"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 的 gpu-verification-server/control start", ["gpu-verification-server/control", "start"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：主 agent 直接起服务自证", ["task-server/task_server_lib.py", "--selftest"], HOOK_REPOSITORY, "python", "main-agent"),
                ("名单：服务自证带别的参数不认", ["task-server/task_server_lib.py", "--selftest", "x"], HOOK_REPOSITORY, "python", "nobody"),
                ("名单：主 agent 起定时提醒的 remind.sh", ["scripts/agents/remind.sh", "30", "x"], HOOK_REPOSITORY, "shell", "main-agent"),
                ("名单：主 agent 查活动会话的 active-sessions.py", [ACTIVE_SESSIONS_SCRIPT, "--within-minutes", "60"], HOOK_REPOSITORY, "python", "main-agent"),
                ("不是本仓代码：草稿里自己的辅助脚本", ["own-draft-helper.sh"], work, "shell", None),
                ("不是本仓代码：PATH 上的 cargo（由 starts_a_build 认）", ["cargo", "build"], HOOK_REPOSITORY, None, None),
                ("不是本仓代码：盘上不在的脚本", ["not-here-137.sh"], work, "shell", None),
                ("不是本仓代码：python3 -m py_compile（语法检查）", ["py_compile.py", "x.py"], work, "python", None)):
            results.append((label, want, project_level(words, directory, launcher)))
        for label, words, directory, want in (
                ("提交：submit -- 后面那条", [TASK_SERVER_CLIENT, "submit", "--", "cargo", "build"], HOOK_REPOSITORY, (["cargo", "build"], None)),
                ("提交：--cwd 照这一条的目录解开", [TASK_SERVER_CLIENT, "submit", "--cwd", "crates", "--", "ls"], HOOK_REPOSITORY,
                 (["ls"], os.path.join(HOOK_REPOSITORY, "crates"))),
                ("提交：仓副本里逐字节相同的客户端", [TASK_SERVER_CLIENT, "submit", "--", "ls"], copy_root, (["ls"], None)),
                ("提交：--manual 不带值，-- 后面那条照认", [TASK_SERVER_CLIENT, "submit", "--manual", "--", "cargo", "build"], HOOK_REPOSITORY, (["cargo", "build"], None)),
                ("提交：--full-gate 不带值，-- 后面那条照认", [TASK_SERVER_CLIENT, "submit", "--full-gate", "--", "cargo", "build"], HOOK_REPOSITORY, (["cargo", "build"], None)),
                ("提交：--whole-package-confirmed 带一个值，-- 后面那条照认", [TASK_SERVER_CLIENT, "submit", "--whole-package-confirmed", "/sample/confirmed", "--", "cargo", "test"], HOOK_REPOSITORY,
                 (["cargo", "test"], None)),
                ("提交：几个选项混写，--cwd 照解开", [TASK_SERVER_CLIENT, "submit", "--full-gate", "--cwd", "crates", "--whole-package-confirmed", "/sample/confirmed", "--manual", "--", "ls"], HOOK_REPOSITORY,
                 (["ls"], os.path.join(HOOK_REPOSITORY, "crates"))),
                ("提交：--whole-package-confirmed 把 -- 当值取走之后没有 --，是用法错，不认", [TASK_SERVER_CLIENT, "submit", "--whole-package-confirmed", "--", "ls"], HOOK_REPOSITORY, None),
                ("提交：--whole-package-confirmed=<文件> 客户端不认，是用法错，不认", [TASK_SERVER_CLIENT, "submit", "--whole-package-confirmed=/sample/confirmed", "--", "ls"], HOOK_REPOSITORY, None),
                ("提交：认不出的选项是用法错，不认", [TASK_SERVER_CLIENT, "submit", "--sample-unknown", "--", "ls"], HOOK_REPOSITORY, None),
                ("提交：没写 -- 的是用法错，不认", [TASK_SERVER_CLIENT, "submit", "cargo", "build"], HOOK_REPOSITORY, None),
                ("提交：wait 不是提交", [TASK_SERVER_CLIENT, "wait", "7"], HOOK_REPOSITORY, None),
                ("提交：别处同名而内容不同的 task 不认", ["task-server/task", "submit", "--", "ls"], changed_root, None)):
            results.append((label, want, task_server_submission(words, directory)))
        # 子 agent 起提交客户端只许 status、log（subagent_task_server_refusal；project-code-guard.sh 与 run-scope-guard.sh 都调它）
        for label, words, directory, launcher, want in (
                ("子 agent 起客户端：submit 拒", [TASK_SERVER_CLIENT, "submit", "--", "ls"], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT} submit"),
                ("子 agent 起客户端：python3 起的 submit 拒", [TASK_SERVER_CLIENT, "submit", "--", "ls"], HOOK_REPOSITORY, "python", f"{TASK_SERVER_CLIENT} submit"),
                ("子 agent 起客户端：仓副本里逐字节相同的客户端 wait 拒", [TASK_SERVER_CLIENT, "wait", "7"], copy_root, None, f"{TASK_SERVER_CLIENT} wait"),
                ("子 agent 起客户端：cancel 拒", [TASK_SERVER_CLIENT, "cancel", "7"], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT} cancel"),
                ("子 agent 起客户端：clean 拒", [TASK_SERVER_CLIENT, "clean", "7"], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT} clean"),
                ("子 agent 起客户端：没写子命令拒", [TASK_SERVER_CLIENT], HOOK_REPOSITORY, None, f"{TASK_SERVER_CLIENT}（没写子命令）"),
                ("子 agent 起客户端：status 放", [TASK_SERVER_CLIENT, "status", "7"], HOOK_REPOSITORY, None, None),
                ("子 agent 起客户端：log 放", [TASK_SERVER_CLIENT, "log", "7", "--all"], HOOK_REPOSITORY, None, None),
                ("子 agent 起客户端：别处同名而内容不同的 task 不认（由 project_code_use 照本仓的代码判）", ["task-server/task", "submit", "--", "ls"], changed_root, None, None),
                ("子 agent 起客户端：control check 不是提交客户端", ["task-server/control", "check"], HOOK_REPOSITORY, "python", None)):
            results.append((label, want, subagent_task_server_refusal(words, directory, launcher)))
        # 确认单打的确认命令（task_server_lib.confirm_command，出路本身要过得了别的闸）：主 agent 直接起不拒，子 agent 起照 submit 那样拒
        library, library_problem = task_server_library()
        confirm_words = shlex.split(library.confirm_command(7, 2, 1)) if library is not None else None
        results.append(("名单：确认单打的 task confirm 主 agent 直接起", "anyone",
                        project_level(confirm_words, HOOK_REPOSITORY, None) if confirm_words else library_problem))
        results.append(("子 agent 起客户端：确认单打的 task confirm 拒", f"{TASK_SERVER_CLIENT} confirm",
                        subagent_task_server_refusal(confirm_words, HOOK_REPOSITORY, None) if confirm_words else library_problem))
        guarded, guarded_problem = task_server_guarded()
        results.append(("绕开：读得到 task-server/task_server_lib.py 的名字与缺省值", None, guarded_problem))
        if guarded is not None:
            for label, text, want in (
                    ("绕开：点到库文件名", "sqlite3 ~/.local/state/singlefs/task-server/task-server.sqlite3 .tables", True),
                    ("绕开：点到签名密钥文件名", "cat ~/.config/singlefs/task-server-signing.key", True),
                    ("绕开：点到 socket", "curl --unix-socket /run/user/1000/singlefs-task-server/server.sock http://x/health", True),
                    ("绕开：python 代码载入 task_server_lib", "python3 -c 'import task_server_lib'", True),
                    ("绕开：python 代码按文件载入 task_server_lib.py（spec_from_file_location）",
                     "python3 -c \"import importlib.util as u; s = u.spec_from_file_location('m', 'task-server/task_server_lib.py'); m = u.module_from_spec(s); s.loader.exec_module(m)\"", True),
                    ("绕开：python 代码按文件载入 task_server_lib.py（runpy.run_path）", "python3 -c \"import runpy; runpy.run_path('task-server/task_server_lib.py')\"", True),
                    ("绕开：python 代码按文件载入 task_server_lib.py（exec(open(…))）", "python3 -c \"exec(open('task-server/task_server_lib.py').read())\"", True),
                    ("绕开：python 代码先 open().read() 存进变量再 exec", "python3 -c \"source = open('task-server/task_server_lib.py').read(); exec(source)\"", True),
                    ("绕开：喂给 python 的 heredoc 里先 compile 再 exec", "python3 - <<'PY'\ncode = compile(open('task-server/task_server_lib.py').read(), 'lib', 'exec')\nexec(code)\nPY", True),
                    ("绕开：喂给 python 的 heredoc 里 exec( 与 open( 拆在两行", "python3 - <<'PY'\nexec(\n    open('task-server/task_server_lib.py').read())\nPY", True),
                    ("绕开：python 代码 import 一串里逗号后面是 task_server_lib", "python3 -c \"import sys; sys.path.insert(0, 'task-server'); import json, task_server_lib\"", True),
                    ("绕开：python 代码 runpy.run_module 按模块名起 task_server_lib", "python3 -c \"import runpy, sys; sys.path.insert(0, 'task-server'); runpy.run_module('task_server_lib')\"", True),
                    ("绕开：python 代码先拼出路径再 spec_from_file_location",
                     "python3 -c \"import importlib.util as u; p = 'task-server/' + 'task_server_lib' + '.py'; s = u.spec_from_file_location('m', p); m = u.module_from_spec(s); s.loader.exec_module(m)\"", True),
                    ("绕开：写进 .py 文件的 heredoc 正文里按文件载入照拒", "cat > /sample/load-probe.py <<'EOF'\nexec(open('task-server/task_server_lib.py').read())\nEOF", True),
                    ("绕开：python 从标准输入读的程序是管道前一段 cat 出来的 task_server_lib.py", "cat task-server/task_server_lib.py | python3 -", True),
                    ("绕开：只读的 grep 里写到 task_server_lib.py 不算", "grep -n 'task_server_lib.py' task-server/task_server.py", False),
                    ("绕开：只读的 sed -n 与 wc 里写到 task_server_lib.py 不算", "sed -n '1,40p' task-server/task_server_lib.py; wc -l \"task-server/task_server_lib.py\"", False),
                    ("绕开：只读的 grep 搜载入调用名（带括号）不算", "grep -n 'spec_from_file_location(' task-server/task_server_lib.py", False),
                    ("绕开：只读的 grep -nF 抄自证里那一行样本不算", "grep -nF \"runpy.run_path('task-server/task_server_lib.py')\" .claude/hooks/lib_project_code_runs_selftest.py", False),
                    ("绕开：只读的 sed -n 按载入调用名找行不算", "sed -n '/exec(open(/p' task-server/task_server_lib.py", False),
                    ("绕开：写进 .md 文件的 heredoc 正文里写到载入调用与文件名不算", "cat > /sample/notes.md <<'EOF'\nexec(open('task-server/task_server_lib.py').read())\nEOF", False),
                    ("绕开：命令文本里 export 准入开关", f"export {guarded.admitted_variable}=1.1", True),
                    ("绕开：python 代码里写 environ 的准入开关", f"python3 -c \"import os; os.environ['{guarded.admitted_variable}'] = '1'\"", True),
                    ("绕开：只读日志目录不算", "ls ~/.local/state/singlefs/task-server/logs", False),
                    ("绕开：提交客户端本身不算", "task-server/task submit -- cargo build", False),
                    ("绕开：python 代码 shutil.rmtree 服务的编译目录", f"python3 -c \"import shutil; shutil.rmtree('{guarded.build_directories[0]}/task-x-7-1')\"", True),
                    ("绕开：python 代码里删别处的目录不算", "python3 -c \"import shutil; shutil.rmtree('/tmp/not-a-task-server-directory')\"", False),
                    ("绕开：task-server/task clean 不算", "task-server/task clean 7", False)):
                results.append((label, want, bool(task_server_text_bypass(text)[0])))
            build_root = guarded.build_directories[0]
            home = os.path.expanduser("~")
            tilde_root = "~" + build_root[len(home):] if build_root.startswith(home + os.sep) else build_root
            state_builds = guarded.build_directories[-1]
            for label, words, environment, want in (
                    ("绕开：命令前设准入开关", ["ls"], {guarded.admitted_variable: "1"}, True),
                    ("绕开：直接起服务本体", ["task-server/task_server.py", "serve"], {}, True),
                    ("绕开：systemctl --user stop 服务单元", ["systemctl", "--user", "stop", guarded.server_unit], {}, True),
                    ("绕开：systemctl --user kill 一次尝试的单元", ["systemctl", "--user", "kill", guarded.task_unit_prefix + "main-3-1.service"], {}, True),
                    ("绕开：systemctl --user stop 通配", ["systemctl", "--user", "stop", guarded.task_unit_prefix + "*"], {}, True),
                    ("绕开：systemctl --user status 服务单元不算", ["systemctl", "--user", "status", guarded.server_unit], {}, False),
                    ("绕开：systemctl stop 别的单元不算", ["systemctl", "--user", "stop", "other.service"], {}, False),
                    ("绕开：rm -rf 服务的编译目录", ["rm", "-rf", "--", os.path.join(build_root, "task-x-7-1")], {}, True),
                    ("绕开：rm 写 ~ 起头的编译目录", ["rm", "-r", tilde_root + "/task-x-7-1/target"], {}, True),
                    ("绕开：rm 状态目录下的 builds", ["rm", "-rf", os.path.join(state_builds, "task-x-7-1")], {}, True),
                    ("绕开：find -delete 编译目录", ["find", build_root, "-name", "*.rlib", "-delete"], {}, True),
                    ("绕开：find -exec rm 编译目录", ["find", "-L", build_root, "-type", "d", "-exec", "rm", "-rf", "{}", "+"], {}, True),
                    ("绕开：rm -rf 编译目录的上级", ["rm", "-rf", os.path.dirname(build_root)], {}, True),
                    ("绕开：du 编译目录不算", ["du", "-sh", build_root], {}, False),
                    ("绕开：find 编译目录不删不算", ["find", build_root, "-maxdepth", "1"], {}, False),
                    ("绕开：rm 别处的目录不算", ["rm", "-rf", "/tmp/not-a-task-server-directory"], {}, False)):
                launcher = "python" if words[0].endswith(".py") else None
                results.append((label, want, bool(task_server_command_bypass(words, environment, HOOK_REPOSITORY, launcher)[0])))
        # 部分文件读不到、编不过：按文件路径导入这份模块（调用方的导入法）抛 PartFileUnreadable、说明点名那一份，不抛读文件错与语法错
        # （弄坏开关 part-unreadable-passes 下抛原来的错、说明里没有「部分文件」，读不到与编不过的四格红；编得过而被截空的几格靠 part-incomplete-passes 红：缺名字抛 ImportError）；
        # 拷一份这几个文件到临时目录里删、写坏或截空其中一份，各导入一次（起子进程，不碰这个进程的模块表）；
        # 拷的目录取 .claude/hooks 的相对位置：传给子进程的库路径尾巴是 .claude/hooks/lib_project_code_runs.py，算键库（scripts/gate/lib_function_units.py）按它认出载入的就是仓里这一份
        part_probe_directory = os.path.join(work, "library-copy", ".claude", "hooks")
        os.makedirs(part_probe_directory)
        part_probe_source = os.path.dirname(os.path.abspath(__file__))
        part_probe_code = ("import importlib.util, sys\n"
                           "spec = importlib.util.spec_from_file_location('lib_project_code_runs_probe', sys.argv[1])\n"
                           "module = importlib.util.module_from_spec(spec)\n"
                           "try:\n"
                           "    spec.loader.exec_module(module)\n"
                           "except Exception as error:\n"
                           "    print(type(error).__name__ + '\\t' + str(error))\n"
                           "else:\n"
                           "    print('loaded\\t')\n")
        results.append(("引导：拷进临时目录的文件里有引导那一份与每一份部分文件", [],
                        [name for name in (PROJECT_CODE_RUNS_BOOTSTRAP_FILE_NAME,) + PROJECT_CODE_RUNS_PART_NAMES if name not in PART_PROBE_COPIED_FILES]))
        # 拷部分文件那一圈的写法照这份文件自己的语法树核（算键库只折得出本模块里只赋一次的字面元组）：往 part_probe_directory 里拷的每一圈只照 PART_PROBE_COPIED_FILES 迭代，
        # 往 part_probe_directory 拷的 shutil.copy* 只在这几圈里（推导式、map、while 里另拷的不算圈），PART_PROBE_COPIED_FILES 在整个模块里只绑定一次（模块顶层那一处：
        # 另有 +=、带注解的再赋值、函数里 global 之后再赋、for / with / 推导式 / 海象 / del / import as 绑它的都算多绑）、是全由字面串组成的元组。
        # 开关 part-probe-copy-loop-tuple-added 下核的语法树里那一圈换成字面元组加 PROJECT_CODE_RUNS_PART_NAMES，part-probe-copied-files-tuple-added 下 PART_PROBE_COPIED_FILES 的值换成
        # 同样的相加；part-probe-copied-files-augmented、part-probe-copied-files-annotated、part-probe-copied-files-global-rebound 下核的语法树里分别多一处 +=、带注解的再赋值、
        # 函数里 global 再赋，part-probe-copy-beside-loop 下多一句推导式另往 part_probe_directory 拷一份；这几条下这一格红
        with open(os.path.abspath(__file__), encoding="utf-8") as own_source_handle:
            own_source_tree = ast.parse(own_source_handle.read())
        tuple_added = ast.parse('("lib_project_code_runs.py",) + PROJECT_CODE_RUNS_PART_NAMES', mode="eval").body
        for widening_switch, widening_text in (
                ("part-probe-copied-files-augmented", 'PART_PROBE_COPIED_FILES += ("lib_writer_lock.py",)\n'),
                ("part-probe-copied-files-annotated", 'PART_PROBE_COPIED_FILES: tuple = PART_PROBE_COPIED_FILES + ("lib_writer_lock.py",)\n'),
                ("part-probe-copied-files-global-rebound", 'def widen_part_probe_copied_files():\n    global PART_PROBE_COPIED_FILES\n'
                                                           '    PART_PROBE_COPIED_FILES = PART_PROBE_COPIED_FILES + ("lib_writer_lock.py",)\n'),
                ("part-probe-copy-beside-loop", '[shutil.copy(os.path.join(part_probe_source, extra_name), part_probe_directory) for extra_name in ("lib_writer_lock.py",)]\n')):
            if break_is_set(widening_switch):
                own_source_tree.body.extend(ast.parse(widening_text).body)
        copy_loops = [node for node in ast.walk(own_source_tree) if isinstance(node, ast.For) and isinstance(node.target, ast.Name)
                      and any(isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and isinstance(call.func.value, ast.Name)
                              and call.func.value.id == "shutil" and call.func.attr.startswith("copy")
                              and any(isinstance(name, ast.Name) and name.id == "part_probe_directory" for argument in call.args for name in ast.walk(argument))
                              and any(isinstance(name, ast.Name) and name.id == node.target.id for argument in call.args for name in ast.walk(argument))
                              for statement in node.body for call in ast.walk(statement))]
        copies_inside_loops = {id(call) for copy_loop in copy_loops for statement in copy_loop.body for call in ast.walk(statement)}
        copies_beside_loops = [ast.unparse(call) for call in ast.walk(own_source_tree)
                               if isinstance(call, ast.Call) and isinstance(call.func, ast.Attribute) and isinstance(call.func.value, ast.Name)
                               and call.func.value.id == "shutil" and call.func.attr.startswith("copy")
                               and any(isinstance(name, ast.Name) and name.id == "part_probe_directory" for argument in call.args for name in ast.walk(argument))
                               and id(call) not in copies_inside_loops]
        if break_is_set("part-probe-copy-loop-tuple-added"):
            for copy_loop in copy_loops:
                copy_loop.iter = tuple_added
        copied_files_assignments = [node for node in own_source_tree.body if isinstance(node, ast.Assign)
                                    and any(isinstance(target, ast.Name) and target.id == "PART_PROBE_COPIED_FILES" for target in node.targets)]
        copied_files_bindings = [node for node in ast.walk(own_source_tree)
                                 if (isinstance(node, ast.Name) and node.id == "PART_PROBE_COPIED_FILES" and isinstance(node.ctx, (ast.Store, ast.Del)))
                                 or (isinstance(node, (ast.Global, ast.Nonlocal)) and "PART_PROBE_COPIED_FILES" in node.names)
                                 or (isinstance(node, ast.alias) and node.asname == "PART_PROBE_COPIED_FILES")]
        if break_is_set("part-probe-copied-files-tuple-added"):
            for copied_files_assignment in copied_files_assignments:
                copied_files_assignment.value = tuple_added
        results.append(("引导：拷部分文件那一圈只照 PART_PROBE_COPIED_FILES 迭代、往探针目录拷的只有这一圈，它在模块里只绑定一次（不另 +=、不带注解再赋、不在函数里 global 再赋）、"
                        "是字面串的元组（算键库折得出拷的是哪几份）",
                        (["PART_PROBE_COPIED_FILES"], 1, True, []),
                        (sorted({ast.unparse(copy_loop.iter) for copy_loop in copy_loops}), len(copied_files_bindings),
                         all(isinstance(assignment.value, ast.Tuple) and all(isinstance(element, ast.Constant) and isinstance(element.value, str) for element in assignment.value.elts)
                             for assignment in copied_files_assignments),
                         copies_beside_loops)))
        for part_name in PROJECT_CODE_RUNS_PART_NAMES:
            for damage, damage_label in (("missing", "读不到"), ("syntax-error", "编不过"), ("empty", "截空")):
                for copied_name in PART_PROBE_COPIED_FILES:
                    shutil.copy(os.path.join(part_probe_source, copied_name), part_probe_directory)
                if damage == "missing":
                    os.remove(os.path.join(part_probe_directory, part_name))
                else:
                    with open(os.path.join(part_probe_directory, part_name), "w", encoding="utf-8") as handle:
                        handle.write("def broken(:\n" if damage == "syntax-error" else "")
                probe = subprocess.run([sys.executable, "-B", "-c", part_probe_code, os.path.join(part_probe_directory, "lib_project_code_runs.py")],
                                       capture_output=True, text=True, timeout=120)
                probe_lines = probe.stdout.strip().splitlines()
                probe_name, _separator, probe_text = (probe_lines[-1] if probe_lines else "").partition("\t")
                results.append((f"引导:{damage_label}部分文件 {part_name} 抛 PartFileUnreadable、点名那一份", ("PartFileUnreadable", True),
                                (probe_name, f"部分文件 {os.path.join(part_probe_directory, part_name)}" in probe_text)))
    finally:
        lib_project_code_runs_base.SAMPLE_INDEX_REPOSITORY = real_sample_index
        lib_project_code_runs_launchers.FAMILY_OWN_REPOSITORY = None
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ lib_project_code_runs 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 classify()、external_program_use()、word_expansion_use()、pattern_expansion_use()、brace_expansion()、command_under_launcher()、launcher_option()、"
              "runs_compiled_code()、starts_a_build()、test_invocation()、cargo_subcommand()、judged_by_name()、same_content_as_own()、own_script_digest()、own_library_digest()、script_source_reading()、shell_source_lines()、path_without_symbolic_links()、registered_selftest()、"
              "selftest_registry_at()、selftest_registry_problems()、stage_selftest_problems()、stage_selftest_arguments()、stage_selftest_item_problems()、stage_feed_problem()、"
              "project_code_use()、project_relative_path()、directly_run_level()、task_server_submission()、task_server_text_bypass()、task_server_command_bypass()、"
              "task_server_guarded()、unrecognized_launch()、script_body_unresolved_launch()、temporary_service_bare_selftest()、"
              "lib_shell_words.py 的 python_invocation() 与 TEST_BINARY_PATH / BUILT_BINARY_PATH / COMPILED_CODE_SUBCOMMANDS / TEXT_TOOLS_RUN_DIRECTLY / MAIN_AGENT_TOOLS_RUN_DIRECTLY / "
              f"KNOWN_SCRIPT_LOCATIONS / GATE_STAGE_FILES / DISPATCH_LAUNCHER_SCRIPTS / OWN_SCRIPT_LIBRARY_FILES / OWN_SCRIPT_UNRECOGNIZED_SOURCES / PART_PROBE_COPIED_FILES / LAUNCHER_OPTIONS_WITH_VALUE / EXTERNAL_PROGRAM_NAMES 几张表与登记表 {SELFTEST_REGISTRY_NAME}；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ lib_project_code_runs 自检通过（查了 {len(results)} 种：外部程序名单 {len(external_cases)} 种、"
          f"/usr/bin/time、flock、systemd-run 这一类包在外面的 {len(launcher_cases)} 种与它们设的环境变量 {len(environment_cases)} 种、"
          f"bash 起不来时的 POSIX 字符类 {len(bash_missing_cases)} 种、喂给 shell 的标准输入 {len(standard_input_cases)} 种、花括号展开与 bash 对照 {len(brace_words)} 种、"
          f"门禁样本 {len(stage_sample_cases) + 2} 种、样本登记表与仓里那份登记表 {registry_cases} 种、"
          f"阶段参数 --check 与 libtest 参数 --list 的读法 {argument_cases} 种、"
          f"认 source 行的写法与往库里读 {source_reading_cases} 种、跑不跑编译出来的代码 {len(compiled_cases)} 种）")
    return 0
