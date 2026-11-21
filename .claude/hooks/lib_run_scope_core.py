# lib_run_scope.py 的部分模块：判法本体——常量、弄坏开关表、范围声明的认法与判定、检出记录、hook_main、refuse_break_outside_selftest。
# 入口 lib_run_scope.py 按 RUN_SCOPE_PART_NAMES 的次序 import 这一份并把名字全部再导出；自证与调用方换掉的模块变量（project_code_runs、shell_words、submit_flag_asks_in_text）在这一份里换，这一份的函数读的是它们。
# 弄坏开关的名单与红的格登记在这一份的 BREAK_SWITCH_CELLS 里，开关表写在入口 lib_run_scope.py 的文件头。
from lib_run_scope_bootstrap import (
    BREAK_VARIABLE_NAME, GUARD_SCRIPT_NAME, HOOK_DIRECTORY, selftest_marked,
)

import importlib.util
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import unicodedata
from datetime import datetime, timezone
from typing import NamedTuple

sys.dont_write_bytecode = True
sys.path.insert(0, HOOK_DIRECTORY)
import lib_run_scope_target_sources as target_sources  # noqa: E402
DECLARATION_VARIABLE = "SINGLEFS_RUN_SCOPE"
SCOPE_KINDS = ("whole", "items", "target")
SEGMENT_KEYS = ("种类", "跑什么", "范围", "为什么")
SEGMENT_START = re.compile(r"^(种类|跑什么|范围|为什么)\s*=\s*(.*)$", re.S)
REASON_MINIMUM_CHARACTERS = 12
# 不说理由的词：为什么一段里去掉它们再数字数
REASON_FILLER_WORDS = ("确认一下", "跑一下", "跑一遍", "要跑", "验证", "看看")
COMMIT_WHOLE_REASON = "提交前整轮"
WHOLE_RANGE_WORDS = ("整轮", "整包")
KIND_DESCRIPTIONS = {"whole": "整轮（whole）", "items": "点名的几项（items）", "target": "点名的测试目标（target）"}
LAST_RUNS_VARIABLE = "SINGLEFS_LAST_RUNS_DIRECTORY"
LAST_RUNS_ENTRY = "last-runs"   # 没设 SINGLEFS_LAST_RUNS_DIRECTORY 时取数据根下的这一项（同目录 lib_work_root.py 的 shared_data_path，两台读同一份）
DETECTIONS_VARIABLE = "AGENT_HOOK_DETECTIONS"
MAXIMUM_SCRIPT_DEPTH = 6
MAXIMUM_SCRIPT_BYTES = 1 << 20
SCRIPT_HEAD_BYTES = 4096
TEMPLATE_COMMAND_LIMIT = 2000
# 模板照抄整条命令时先摘掉命令开头已有的那份声明（不然照抄出来是两份，bash 取后一份，等于没改）
LEADING_DECLARATION = re.compile(r"\A\s*" + DECLARATION_VARIABLE + r"=(?:'[^']*'|\"(?:[^\"\\]|\\.)*\"|\S+)[ \t]+")
# 命令位置上的词里带这些字符的不当脚本路径去读（变量、命令替换、通配、引号残片、重定向）
UNREADABLE_PATH_CHARACTERS = set("$`*?[]{}()<>|;&'\"=,")
# 门禁阶段只打清单、自证、帮助、工具版本的参数：带了就不要声明
STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items", "--selftest", "--help", "-h", "--tool-versions"})
CRASH_REPLAY_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
CRASH_AMPLIFICATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
SELFTEST_ONLY_QUERY_ARGUMENTS = frozenset({"--selftest", "--help", "-h"})
MERGE_QUERY_ARGUMENTS = frozenset({"--list-items", "--selftest", "--help", "-h"})
REPLAY_QUERY_ARGUMENTS = frozenset({"--selftest", "--admission-only", "--list-items", "--help", "-h"})
# 崩溃放量的分项驱动与双机驱动带一个值的选项（照两份的用法行：分项驱动 --test-filter、--label，双机驱动 --test-filter、--label、--skip）
CRASH_DRIVER_OPTIONS_WITH_VALUE = frozenset({"--test-filter", "--label", "--skip"})
CARGO_NAMED_TARGET_OPTIONS = ("--test", "--bin", "--example", "--bench")
CARGO_SINGLE_TARGET_FLAGS = ("--lib", "--doc")
CARGO_EVERY_TARGET_FLAGS = frozenset({"--tests", "--bins", "--examples", "--benches", "--all-targets"})
CARGO_WHOLE_WORKSPACE_FLAGS = frozenset({"--workspace", "--all"})
GATE_WHOLE_LOCATIONS = (".claude/singlefs-ai-sop/scripts", ".claude/scripts")
# 放宽检查的提交标识（task-server/task submit 的选项）→ 给用户看的作用；判法在 submit_flag_asks
LOOSENING_SUBMIT_FLAGS = {"--full-gate": "免两小时提醒", "--whole-package-confirmed": "整包测试放行", "promote": "把在排的任务提前到队头"}
SUBMIT_FLAG_USER_RULE = "用户定了对话不许自己用这种标识：要用必须用户在这里点同意，不同意就拒绝这一条。"
# 提交时才许不指定范围地跑的六道门禁（.claude/skills/crash-test/commit-stages.md 开头点名的，task-server/task_server_lib.py 的 COMMIT_GATE_ENTRIES 同一组）
# 与崩溃点重放、崩溃放量两份入口（同文件 UNSCOPED_RUN_ENTRIES）：不带选条参数的这一形经 task-server/task submit 交时，`--` 之前要带放宽标识（LOOSENING_SUBMIT_FLAGS）
COMMIT_GATE_STAGE_NAMES = ("crash-point-replay.sh", "qemu-device-streams.sh", "lkmm.sh", "crates-mutation-replay.sh", "formal-verification.sh", "broad-mutation.sh")
UNSCOPED_COMMIT_ENTRIES = tuple(f".claude/gate.d/{name}" for name in COMMIT_GATE_STAGE_NAMES) + ("scripts/crash/crash-point-replay.sh", "scripts/crash/crash-amplification.sh")
COMMIT_PROBLEM_PREFIX = "提交时才许跑："   # 这一类说明不是声明写得不对，拒绝说明里不附声明模板
# 指定范围的那几趟崩溃状态数压在 20 万量级（.claude/rules/verification.md「一个门禁：两种跑法」）：按登记表与测试源码判点到的目标或项会不会走崩溃枚举、读哪一个缩域变量，
#   命令里写了 --crash-expand-up-to 或环境里设了它读的那一个的放（crash_domain_problem）
CRASH_SHRINK_OPTION = "--crash-expand-up-to"
CRASH_SHRINK_VARIABLES = target_sources.CRASH_SHRINK_VARIABLES   # 变量名与读测试源码的那一段在 lib_run_scope_target_sources.py
FULL_DOMAIN_REASON_PATTERN = re.compile(r"全域理由\s*[：:]\s*(.*)", re.S)   # 为什么一段里写明确要全域的理由
FULL_DOMAIN_REASON_MINIMUM_CHARACTERS = 12
STAGE_INPUTS_TABLE = ".claude/gate.d/stage-inputs.tsv"
REPLAY_ITEMS_TABLE = ".claude/gate.d/crash-point-replay-items.tsv"
CRASH_AMPLIFICATION_ENTRY = "scripts/crash/crash-amplification.sh"
MUTATION_TABLE = "crates/mutations.tsv"
VERIFICATION_PACKAGE = "singlefs-verification"
VERIFICATION_TESTS_DIRECTORY = "crates/singlefs-verification/tests"
CRASH_CASE_TEST_WORD = re.compile(r"(?:^|\s)test=" + re.escape(VERIFICATION_PACKAGE) + r":(\w+):\w+")   # stage-inputs.tsv 的 crash-case: 行里 test=<包>:<目标>:<用例>
AMPLIFICATION_ITEM_LINE = re.compile(r"^\s*\$'[a-z0-9][a-z0-9-]*\\t(\w+)\\t\w+'\s*$")   # crash-amplification.sh 的 ITEMS 里一行：$'分项\t测试目标\t用例'
# Rust 源码里注释、字符串与字符字面量的起头（rust_code_without_comments）：// 与 /* 起注释，r"、r#" 起原始字符串，" 起字符串，'x'、'\n'、'\u{…}' 是字符字面量（别的 ' 是生命周期）
RUST_LEXICAL_START = re.compile(r"""//|/\*|\br(#*)"|"|'(?:\\u\{[0-9a-fA-F]{1,6}\}|\\.|[^\\'\n])'""")
RUST_BLOCK_COMMENT_EDGE = re.compile(r"/\*|\*/")
RUST_STRING_LITERAL = re.compile(r'"(?:\\.|[^"\\])*"', re.S)
CRASH_SUBJECT_DESCRIPTIONS = {"replay-items": "崩溃点重放入口点名的项", "cargo-targets": "cargo test 点名的验证包测试目标",
                              "prove-red": "证红脚本点名的变异行", "mutation-items": "变异门禁点名的行"}
CRASH_SUBJECT_BREAK_SWITCHES = {"replay-items": "crash-domain-replay-items-unjudged", "cargo-targets": "crash-domain-cargo-targets-unjudged",
                                "prove-red": "crash-domain-prove-red-unjudged", "mutation-items": "crash-domain-mutation-items-unjudged"}
# 弄坏开关 → 打开后红的自证格，写全（文件头「弄坏开关」一段逐项照这张表写；开关轮核红的格与这里逐格相等，并核每个开关名都写在 run-scope-guard.sh 文件头里）
BREAK_SWITCH_CELLS = {
    "declaration-optional": ["task-server 提交带 --full-gate 的门禁整道：没带声明拒", "task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒", "门禁整道：没带声明拒", "崩溃点重放不指定范围跑：没带声明拒", "崩溃放量全部分项：没带声明拒", "崩溃放量双机驱动：没带声明拒", "replay.sh 全部实验：没带声明拒", "gate-staged.sh 没带声明拒", "cargo test 整工作区：没带声明拒", "bash -c 里的门禁没带声明拒", "here-string 喂给 bash 的 cargo test 整工作区：没带声明拒", "printf 经管道喂给 bash 的门禁：没带声明拒", "echo 经管道喂给 sh 的 cargo test 整工作区：没带声明拒", "capped.sh 包着的 cargo test 没带声明拒", "env 带别的变量、没带声明拒", "nice 与 timeout 包着、没带声明拒", "flock 包着的门禁没带声明拒", "gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", "/usr/bin/time 包着的 cargo test 没带声明拒", "驱动脚本里一行缺声明拒", "外面带声明、驱动脚本里一行缺声明拒", "驱动脚本再起驱动脚本、里面缺声明拒", "直接执行的驱动脚本缺声明拒", "同一条命令里 heredoc 写出的驱动脚本缺声明拒", "入口：没带声明拒（退出码 2）", "入口：拒绝说明写跑的是什么、照抄模板与只跑红的写法", "入口：拒绝说明附上次红的清单", "入口：没有上次红的记录时照写", "入口：被拒的记一行（session_id、范围声明被拒、outcome 被拒）", "驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒"],
    "segments-unchecked": ["声明缺段：没写为什么拒", "声明值空：范围一段是空的拒", "声明写两次：种类写两次拒", "声明种类认不出拒"],
    "kind-unchecked": ["门禁 --item 声明成 whole：种类对不上拒", "merge-and-verify 声明成 whole：拒", "cargo test 整包声明成 target：种类对不上拒", "声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面"],
    "count-unchecked": ["门禁 --check 两格写成 1 格：范围数对不上拒", "whole 范围没写整轮或整包拒", "cargo test 两个目标写成 3 个：范围数对不上拒", "cargo test --exact 两个测试写成 1 个：范围数对不上拒", "崩溃点重放点名一项另加展开深度：写成两项拒（深度的值不是项）", "merge-and-verify 点名两项写成 1 项：范围数对不上拒",
                        "merge-and-verify 两份补丁写成 1 份：范围数对不上拒", "范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法", "范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验"],
    "reason-unchecked": ["gate.sh 为什么只写验证拒", "为什么太短拒", "为什么只写要跑验证拒"],
    "subagent-submissions-allowed": ["子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）", "子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）",
                                     "入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回", "入口：命令开头已有声明时模板只留一份"],
    "scripts-unread": ["驱动脚本里一行缺声明拒", "外面带声明、驱动脚本里一行缺声明拒", "驱动脚本再起驱动脚本、里面缺声明拒", "直接执行的驱动脚本缺声明拒", "同一条命令里 heredoc 写出的驱动脚本缺声明拒",
                       "入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）", "驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒",
                       "task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）", "入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）"],
    "wrappers-opaque": ["flock 包着的门禁没带声明拒", "gpu-quiesce-run.sh 包着的崩溃点重放没带声明拒", "/usr/bin/time 包着的 cargo test 没带声明拒"],
    "allowed-unrecorded": ["入口：放行的记进检出记录（四段与命令）"],
    "last-runs-unread": ["入口：拒绝说明附上次红的清单"],
    "query-forms-declared": ["门禁 --list 不要声明", "门禁 --selftest 不要声明", "崩溃点重放 --list-items 不要声明", "replay.sh --admission-only 不要声明", "replay.sh --list-items 不要声明", "cargo test --no-run 不要声明", "cargo test -- --list 不要声明", "merge-and-verify --list-items 不要声明"],
    "replay-list-items-declared": ["replay.sh --list-items 不要声明"],
    "merge-as-whole": ["merge-and-verify 声明成 whole：拒", "merge-and-verify 两份补丁：声明成 items 放", "merge-and-verify 两份补丁另加展开深度：写成 2 份补丁放（深度的值不是补丁）",
                       "merge-and-verify 点名两项：声明成 items 放", "task-server 提交合入后验证带 --item：放"],
    "merge-list-items-declared": ["merge-and-verify --list-items 不要声明"],
    "tooling-scripts-read": ["仓里的工具脚本不读：放行"],
    "cargo-targets-as-whole": [target_sources.READS_NO_VARIABLE_CELL, "cargo test 两个目标、带对的声明：放", "cargo test --lib --exact 两个测试、带对的声明：放", "子 agent cargo test 点名目标、带对的声明：放", "capped.sh 外面带声明放", "驱动脚本里每一行都带声明：放", "驱动脚本路径写成赋过字面值的变量、里面每一行都带声明：放",
                              "cargo test 点名不走崩溃枚举的目标：放", "task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", "cargo test 点名别的包的目标：这一条不判，放",
                              "cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了",
                              "task-server 提交 cargo test 点名 --exact 的测试函数：放"],
    "template-keeps-declaration": ["入口：命令开头已有声明时模板只留一份"],
    "failure-unrecorded": ["入口：hook 输入不是 JSON 对象（坏 JSON、数组）放行、stderr 写诊断、各记一条检出", "入口：判的时候抛异常放行、stderr 报一句、记一条检出", "入口：here-string 喂给 bash 的是变量放行、记一条检出",
                           "入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）",
                           "入口：判法文件读不到时记一条检出（结局只记、会话号照输入）", "入口：判法文件读不到、没设 AGENT_HOOK_DETECTIONS 时记进 lib_work_root.py 的默认位置"],
    "judge-crash-unrecorded": ["入口：判法文件在而导入就出错时放行、stderr 报一句、记一条检出（结局只记、会话号照输入）",
                               "入口：判法的部分文件读不到：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable",
                               "入口：判法的部分文件编不过：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable",
                               "入口：判法的部分文件截空：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable"],
    "driver-unread-unrecorded": ["入口：驱动脚本读不了时放行、记一条检出（范围声明闸判不了）"],
    "honor-outside-selftest": ["入口：平常跑设了弄坏开关退 2、点名它只在自证里认", "入口：标记文件内容对不上照拒"],
    "honor-outside-selftest-with-others": ["入口：关守卫项与别的项同写：带标记的子进程里也退 2、点名它"],
    "task-server-submissions-unread": ["task-server 提交的门禁整道：没带声明拒", "task-server 提交的 cargo test 整工作区：没带声明拒",
                                       "task-server 提交带 --full-gate 的门禁整道：没带声明拒", "task-server 提交带 --manual 的门禁整道：没带声明拒",
                                       "task-server 提交带 --whole-package-confirmed 的 cargo test 整工作区：没带声明拒",
                                       "子 agent 经 task-server 提交崩溃点重放不指定范围跑、带对的声明：拒（子 agent 不往服务交任务）", "子 agent 经 task-server 提交点名目标、带对的声明：拒（子 agent 不往服务交任务）",
                                       "入口：子 agent 经 task-server 交验证命令拒、说明写写进报告交回", "入口：命令开头已有声明时模板只留一份",
                                       "task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒",
                                       "task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒", "task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）",
                                       "入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒",
                                       "task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）", "task-server 提交 cargo test 点名目标、不点名测试函数：拒", "task-server 提交证红不点名变异：拒",
                                       "task-server 提交门禁点起测试的格、不带 --item：拒", "入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）",
                                       "task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）", "task-server 提交 bash -c、里面是带分片标记的入口：拒（套在 bash -c 里不分片）",
                                       "task-server 提交 bash -c 里带分片标记的入口、task_server_lib 载不进：放行、记一处判不了", "入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）"],
    "submit-flags-unjudged": ["提交带 --full-gate：弹窗", "提交带 --whole-package-confirmed：弹窗", "范围声明写在前面：弹窗", "bash -c 里面：弹窗", "&& 连着的后一段：弹窗", "--cwd 在标识前面：弹窗", "两个标识都带：都点名",
                              "python3 起客户端：弹窗", "cd 进 task-server 再起：弹窗", "cd 到算不出的目录再起：弹窗", "cd 到算不出的目录再起 ./task：弹窗", "eval 字面代码：弹窗", "赋过字面值的变量写在参数上：认出标识", "数组写在参数上：认出标识", "标识写成等号带值：弹窗",
                              "参数是变量：弹窗认不出", "数组全部元素写在参数上：弹窗认不出", "eval 给出的参数是变量：弹窗认不出", "子命令位是变量：弹窗认不出", "参数是命令替换：弹窗认不出", "xargs 读进来的参数：弹窗认不出", "flock 包着：弹窗", "python3 -c 里字面写着：弹窗",
                              "驱动脚本里：弹窗", "交给服务的那一条里又提交：弹窗", "脚本里的转发函数：弹窗", "命令词是赋过字面值的变量：弹窗", "eval 赋过字面值的变量：弹窗", "判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了",
                              "入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）", "入口：参数是变量弹窗、理由写认不出带没带标识"],
    "submit-flags-by-substring": ["grep 这个词：不弹窗", "commit message 带这个词：不弹窗", "--why 里提到这个词：不弹窗", "交给服务的那一条里提到这个词：不弹窗", "echo 这条命令的文字：不弹窗", "范围声明的值里提到这个词：不弹窗"],
    "submit-flag-variables-unread": ["参数是变量：弹窗认不出", "数组全部元素写在参数上：弹窗认不出", "eval 给出的参数是变量：弹窗认不出", "子命令位是变量：弹窗认不出", "参数是命令替换：弹窗认不出", "xargs 读进来的参数：弹窗认不出", "入口：参数是变量弹窗、理由写认不出带没带标识"],
    "submit-flag-after-separator": ["交给服务的那一条里提到这个词：不弹窗", "交给服务的那一条参数是变量：不弹窗", "xargs 读进来的落在交给服务的那一条里：不弹窗"],
    "submit-flag-first-option-only": ["--cwd 在标识前面：弹窗", "两个标识都带：都点名"],
    "submit-flag-client-by-path-only": ["cd 到算不出的目录再起：弹窗", "cd 到算不出的目录再起 ./task：弹窗"],
    "submit-flag-nesting-unread": ["xargs 读进来的参数：弹窗认不出", "flock 包着：弹窗", "python3 -c 里字面写着：弹窗", "驱动脚本里：弹窗", "交给服务的那一条里又提交：弹窗", "脚本里的转发函数：弹窗", "命令词是赋过字面值的变量：弹窗", "eval 赋过字面值的变量：弹窗"],
    "submit-flag-subagents-asked": ["子 agent 带标识：不弹窗（由项目代码闸拒）"],
    "submit-flag-raise-propagated": ["判放宽标识时抛异常：不牵连范围声明、提到标识的照弹窗、记一处判不了"],
    "submit-flag-ask-silent": ["入口：提交带 --full-gate 弹窗（退 0、ask、理由写标识、作用、命令全文与不许自己用）", "入口：参数是变量弹窗、理由写认不出带没带标识", "入口：判的时候抛异常、命令文字里提到标识：弹窗"],
    "commit-whole-flag-unchecked": ["task-server 提交不指定范围的崩溃点重放门禁、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交六道提交时门禁各自不指定范围、没带放宽标识：都拒",
                                    "task-server 提交不指定范围的崩溃点重放入口、没带放宽标识：拒", "task-server 提交不指定范围的崩溃放量入口、没带放宽标识：拒（提交时带 --full-gate）", "task-server 提交不指定范围的崩溃放量入口、只带 --whole-package-confirmed：拒（整包确认顶替不了 --full-gate）",
                                    "入口：不指定范围的提交没带放宽标识拒（退出码 2、说明写 commit-stages.md 与 --full-gate、不附声明模板）"],
    "crash-domain-replay-items-unjudged": ["崩溃点重放点名会走崩溃枚举的项、没缩域：拒", "崩溃点重放点名放量分项的项、没缩域：拒", "崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒", "入口：崩溃域没缩拒（退出码 2、说明写缩域变量与全域理由的写法）", "崩溃点重放点名的项不在合表里：放行、记一处判不了",
                                           "崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）"],
    "crash-domain-cargo-targets-unjudged": [target_sources.READS_JUDGE_VARIABLE_CELL, "cargo test 点名登记过的崩溃枚举目标、没缩域：拒", "cargo test 点名源码里读缩域变量的目标、没缩域：拒", "cargo test 点名登记过的放量分项目标、没缩域：拒", "cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量",
                                            "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒"],
    "crash-domain-prove-red-unjudged": ["证红点名会走崩溃枚举的变异行、没缩域：拒", "证红点名的变异名不在变异表里：放行、记一处判不了", "证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "crash-domain-mutation-items-unjudged": ["变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒", "变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒", "变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个", "变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", "变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-registry-unread": ["cargo test 点名登记过的崩溃枚举目标、没缩域：拒", "cargo test 点名登记过的放量分项目标、没缩域：拒", "证红点名会走崩溃枚举的变异行、没缩域：拒", "变异门禁点名会走崩溃枚举的行（变异名）、没缩域：拒", "变异门禁点名会走崩溃枚举的行（行号）、没缩域：拒",
                                     "变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", "证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", "变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", "变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-source-scan-skipped": [target_sources.READS_JUDGE_VARIABLE_CELL, "cargo test 点名源码里读缩域变量的目标、没缩域：拒", "cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量", "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）",
                                        "变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-shrink-ignored": ["崩溃点重放点名会走崩溃枚举的项、带 --crash-expand-up-to：放", "崩溃点重放点名会走崩溃枚举的项、命令前带放量缩域变量：放", "task-server 提交崩溃点重放点名项、-- 之后 env 带缩域变量：放",
                                    "task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", "证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放", "变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放",
                                    "cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了"],
    "crash-domain-reason-ignored": ["崩溃点重放点名会走崩溃枚举的项、为什么里写了全域理由：放"],
    "crash-domain-reason-length-unchecked": ["崩溃点重放点名会走崩溃枚举的项、全域理由太短：拒"],
    "crash-domain-full-gate-judged": ["task-server 提交带 --full-gate、崩溃点重放点名项没缩域：这一条不判，放"],
    "crash-domain-unknown-unrecorded": ["崩溃点重放点名的项不在合表里：放行、记一处判不了", "证红点名的变异名不在变异表里：放行、记一处判不了", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了", "证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "crash-domain-target-variable-fixed": ["变异门禁点名登记过的崩溃枚举目标的行、没缩域：拒绝说明的出路点名放量那个变量", "证红点名会走崩溃枚举的变异行、命令前带放量缩域变量：放", "变异门禁点名会走崩溃枚举的行、命令前带放量缩域变量：放", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个"],
    "crash-domain-target-variable-source-unread": ["cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量", "task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放",
                                                   "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", "cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了"],
    "crash-domain-variable-comments-counted": ["task-server 提交 cargo test 点名崩溃枚举目标、-- 之后 env 带缩域变量：放", "cargo test 点名源码里只读判器缩域变量的目标、没缩域：拒绝说明的出路点名判器那个变量"],
    "crash-domain-shrink-variable-unchecked": ["崩溃点重放点名会走崩溃枚举的项、命令前只带判器缩域变量：拒（入口读的是放量那个）", "cargo test 点名代码里只读判器缩域变量的目标、命令前只带放量缩域变量：拒（注释里提到放量那个不算读它）", "变异门禁点名登记过的崩溃枚举目标的行、只带判器缩域变量：拒，说明点名该设放量那个"],
    "crash-domain-extra-variable-refused": ["cargo test 点名代码里只读判器缩域变量的目标、两个缩域变量都带：放"],
    "crash-domain-variable-unknown-refused": ["cargo test 点名登记过而读不到源码的目标、带缩域变量：判不出读哪一个，放行、记一处判不了"],
    "count-hint-unprinted": ["范围数对不上：一格加两项数成 3 项，说明写整段 范围=3 项 与格项相加的数法", "范围数对不上：实验复跑点一个实验写成 2 行，说明写整段 范围=1 个实验", "声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面"],
    "variable-built-hint-dropped": ["声明成 items 而 --item 是变量拼的：命令数成整轮，说明写整段 范围=整轮 与 把 --item 写成字面"],
    "driver-path-variables-unexpanded": ["驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒", "task-server 提交里起的驱动脚本路径写成赋过字面值的变量：读进去、里面一行缺声明拒"],
    "crash-domain-named-tree-unread": ["证红点名副本里新登记的崩溃枚举目标的变异行（--copy 写字面路径）、没缩域：拒", "task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "变异门禁点名副本里新写的行（根参数指到副本）、没缩域：拒", "证红点名的变异名副本里与主工作区都没有：放行、记一处判不了、说明写读过的树", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "crash-domain-tree-fallback-dropped": ["变异门禁根参数指到副本、点名的行副本里没有而主工作区有：退回主工作区那一份判、没缩域：拒"],
    "crash-domain-tree-variables-unexpanded": ["task-server 提交证红、--copy 写成赋过字面值的变量、点名副本里新登记的崩溃枚举目标的变异行、没缩域：拒", "task-server 提交 --cwd 写成赋过字面值的变量指到副本、cargo test 点名副本里新登记的崩溃枚举目标、没缩域：拒", "证红 --copy 写成展开不出值的变量：放行、记一处判不了、说明点名那个变量"],
    "unnamed-submissions-unjudged": ["task-server 提交合入后验证不带 --item：拒（启动测试与验证要点名）", "task-server 提交 cargo test 点名目标、不点名测试函数：拒", "task-server 提交证红不点名变异：拒",
                                     "task-server 提交门禁点起测试的格、不带 --item：拒", "入口：经 submit 交的 cargo test 不点名测试函数拒（退出码 2、说明写没点名与改成点名的样子 --exact、不附声明模板、记一行被拒）"],
    "unnamed-full-gate-dropped": ["task-server 提交带 --full-gate 的门禁整道、submit 前面带对的声明：放", "task-server 提交带 --full-gate、证红不点名变异：放（提交时那一趟不判点名）"],
    "unnamed-confirmed-dropped": ["提交带 --whole-package-confirmed：弹窗", "task-server 提交带 --whole-package-confirmed、证红不点名变异：放（用户确认过的整包不判点名）"],
    "sharded-wrapped-unjudged": ["task-server 提交驱动脚本、里面那一行是带分片标记的入口：拒（套在驱动脚本里不分片）", "task-server 提交 bash -c、里面是带分片标记的入口：拒（套在 bash -c 里不分片）",
                                 "task-server 提交 bash -c 里带分片标记的入口、task_server_lib 载不进：放行、记一处判不了", "入口：交给服务的驱动脚本里套着带分片标记的入口拒（退出码 2、说明点名入口与驱动脚本:行、出路与拼好的提交）"],
    "part-incomplete-passes": ["入口：判法的部分文件截空：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable"],
    "part-unreadable-passes": ["入口：判法的部分文件读不到：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable",
                               "入口：判法的部分文件编不过：放行、stderr 点名那一份、判法退 70、记一条检出、按文件路径导入抛 PartFileUnreadable"],
}
BREAK_SWITCH_CELLS.update(target_sources.BREAK_SWITCH_CELLS)   # 读测试源码那一段的开关与红的格登记在 lib_run_scope_target_sources.py

project_code_runs = None
shell_words = None


def guard_switch_on(name):
    """弄坏开关 RUN_SCOPE_GUARD_BREAK 里有没有这一项；有不认识的项抛 ValueError（免得拼错的开关被当成没开）。"""
    switches = [item for item in os.environ.get(BREAK_VARIABLE_NAME, "").split(",") if item]
    unknown = [item for item in switches if item not in BREAK_SWITCH_CELLS]
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE_NAME} 里有认不出的项 {'、'.join(unknown)}，只认 {'、'.join(BREAK_SWITCH_CELLS)}")
    return name in switches


def load_judging_libraries():
    """按文件路径导入这份库同目录的 lib_project_code_runs.py（它再导入同目录的 lib_shell_words.py）。读不到或导入出错照抛，由调用方放行并报一句。"""
    global project_code_runs, shell_words
    if project_code_runs is None:
        spec = importlib.util.spec_from_file_location("lib_project_code_runs", os.path.join(HOOK_DIRECTORY, "lib_project_code_runs.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        project_code_runs, shell_words = module, module.shell_words
    return project_code_runs


class RunShape(NamedTuple):
    kind: str            # whole / items / target
    what: str            # 这条命令跑的是什么（给 agent 看的一句）
    count: int | None    # items 与 target 的 N
    unit: str            # N 的单位：格、项、个实验、个测试目标、个测试
    range_hint: str      # 模板里范围一段现算好的写法
    script_key: str | None   # 上次红的清单按它找（脚本相对仓根的路径）；cargo test 没有
    family: str          # stage、crash、replay、merge、gate-whole、cargo


class FoundRun(NamedTuple):
    frames: tuple        # 读到它经过的驱动脚本：((脚本路径, 行号或 None), …)；命令文本里直接写的是 ()
    words: list          # 剥掉前缀与包装之后的那一条
    shape: RunShape
    declaration: str | None   # 这一条看得到的 SINGLEFS_RUN_SCOPE；没有是 None
    client_refusal: str | None = None   # 经 task-server/task submit 交给服务的那一条：lib_project_code_runs.subagent_task_server_refusal 对交它的那一条客户端命令的说明（子 agent 照它拒）
    submit: tuple | None = None   # 经 task-server/task submit 交的：交它的那一条在 `--` 之前带的放宽标识（可以是空元组）；不经 submit 起的是 None


class CrashSubject(NamedTuple):
    frames: tuple         # 读到它经过的驱动脚本，同 FoundRun.frames
    kind: str             # replay-items、cargo-targets、prove-red、mutation-items（CRASH_SUBJECT_DESCRIPTIONS 的键）
    words: tuple          # 剥掉前缀与包装之后的那一条
    directory: str | None
    environment: dict     # 这一条看得到的环境赋值（命令前的赋值、env、export），缩域变量与范围声明都从这里取
    submit: tuple | None  # 同 FoundRun.submit
    variables: dict | None = None   # 这一条之前赋过的 shell 变量（名字 → 已展开的值，None 是算不出）：命令点名的那棵树写成 `$名` 时照它展开（point_tree_words）


class ScanState:
    """一次判定里各层共用：同一条命令里前面用 heredoc 写出的文件（实际路径 → 正文），各层判不了的、喂给 shell 的输入（每处一句）。"""
    def __init__(self):
        self.written = {}
        self.unjudged = []
        self.flag_scripts_read = set()   # 判放宽标识时读过的驱动脚本（实际路径）：一次判定里每份只读一遍
        self.submit = None   # 正往里判的那一条 task-server/task submit 带的放宽标识；不在 submit 里是 None
        self.submit_frames = None   # 正往里判的那一条 submit 所在的那一层（frames）：同一层的（含剥了包装的）才是交给服务的那一条，读进去的驱动脚本里的不算
        self.crash_subjects = []   # 命令位置上每一条崩溃域闸要判的指定范围的命令（CrashSubject）
        self.submitted_runs = []   # 经 task-server/task submit 交给服务的那一条里命令位置上的每一条（SubmittedRun）：点没点名照 unnamed_submission_problem 判
        self.current_submission = None   # 正往里判的那一条 task-server/task submit 交给服务的原命令：(`--` 之后的词, 任务的当前目录, 所在那一层 frames)；不在 submit 里是 None
        self.wrapped_candidates = []   # 交给服务的那一条里（含 bash -c、包装与读进去的驱动脚本里）命令位置上的每一条（WrappedCandidate）：是不是套着分片入口照 sharded_wrapped_entry 判


class SubmittedRun(NamedTuple):
    frames: tuple         # 同 FoundRun.frames
    words: tuple          # 剥掉前缀与包装之后的那一条
    directory: str | None
    submit: tuple         # 交它的那一条 submit 带的放宽标识（同 FoundRun.submit）


class WrappedCandidate(NamedTuple):
    frames: tuple                # 同 FoundRun.frames
    words: tuple                 # 剥掉前缀与包装之后的那一条
    launcher: str | None         # words[0] 是经什么起的脚本（lib_shell_words.CommandAtPosition.launcher）：拼直接提交的那一条时补回 bash / python3
    directory: str | None        # 跑这一条时的当前目录
    declaration: str | None      # 这一条看得到的 SINGLEFS_RUN_SCOPE；没有是 None
    submitted_words: tuple       # 交给服务的原命令（`--` 之后的词）
    task_directory: str | None   # 任务的当前目录（--cwd 给了的取它，没给取交它的那一条 submit 所在的目录）
    submission_frames: tuple     # 交它的那一条 submit 所在那一层的 frames


def without_force(arguments):
    """各入口的 preflight 先摘掉每一个 --force，判的是摘完的参数。"""
    return [argument for argument in arguments if argument != "--force"]


def named_option_values(arguments, option):
    """`--item X` 与 `--item=X` 点名的值，去重、按出现次序。"""
    values, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument == option and position + 1 < len(arguments):
            values.append(arguments[position + 1])
            position += 2
            continue
        if argument.startswith(option + "="):
            values.append(argument[len(option) + 1:])
        position += 1
    return list(dict.fromkeys(value for value in values if value))


def query_form(arguments, query_arguments):
    """只查清单、自证、帮助这类不要声明的跑法。弄坏开关 query-forms-declared 下一律当真跑。"""
    return not guard_switch_on("query-forms-declared") and any(argument in query_arguments for argument in arguments)


def counted_text(count, unit):
    return f"{count} {unit}"


def stage_run_shape(stage, arguments):
    rest = without_force(arguments)
    if query_form(rest, STAGE_QUERY_ARGUMENTS):
        return None
    key = f".claude/gate.d/{stage}"
    cells = sorted(project_code_runs.checked_cells(rest) or set())
    items = named_option_values(rest, "--item")
    if cells or items:
        parts = ([f"格 {'、'.join(cells)}"] if cells else []) + ([f"项 {'、'.join(items)}"] if items else [])
        unit = "格" if cells and not items else "项"
        count = len(cells) + len(items)
        return RunShape("items", f"门禁 {stage} 点名的{'，'.join(parts)}", count, unit, counted_text(count, unit), key, "stage")
    return RunShape("whole", f"门禁 {stage} 的全部格（不带 --check / --item）", None, "", "整轮", key, "stage")


def crash_run_shape(name, arguments):
    rest = without_force(arguments)
    key = f"scripts/crash/{name}"
    if name == "crash-point-replay.sh":
        if query_form(rest, CRASH_REPLAY_QUERY_ARGUMENTS):
            return None
        items = named_option_values(rest, "--item")
        if items:
            return RunShape("items", f"崩溃点重放入口点名的项 {'、'.join(items)}", len(items), "项", counted_text(len(items), "项"), key, "crash")
        return RunShape("whole", "崩溃点重放入口不指定范围跑全部项", None, "", "整轮", key, "crash")
    if name == "crash-amplification.sh":
        if query_form(rest, CRASH_AMPLIFICATION_QUERY_ARGUMENTS):
            return None
        items = named_option_values(rest, "--item")
        if items:
            return RunShape("items", f"崩溃放量点名的分项 {'、'.join(items)}", len(items), "项", counted_text(len(items), "项"), key, "crash")
        return RunShape("whole", "崩溃放量的全部分项（不带 --item）", None, "", "整轮", key, "crash")
    if query_form(rest, SELFTEST_ONLY_QUERY_ARGUMENTS):
        return None
    target, filtered, position = None, False, 0
    while position < len(rest):
        argument = rest[position]
        if argument in CRASH_DRIVER_OPTIONS_WITH_VALUE:
            filtered = filtered or argument == "--test-filter"
            position += 2
            continue
        if not argument.startswith("-") and target is None:
            target = argument
        position += 1
    driver = "分项驱动" if name == "crash-amplification-item.sh" else "双机驱动"
    unit = "个测试" if filtered else "个测试目标"
    return RunShape("target", f"崩溃放量{driver}起测试目标 {target or '（没给）'}{'（带 --test-filter）' if filtered else ''}", 1, unit,
                    counted_text(1, unit), key, "crash")


def replay_run_shape(arguments):
    rest = without_force(arguments)
    query_arguments = REPLAY_QUERY_ARGUMENTS - {"--list-items"} if guard_switch_on("replay-list-items-declared") else REPLAY_QUERY_ARGUMENTS
    if query_form(rest, query_arguments):
        return None
    numbers = list(dict.fromkeys(argument for argument in rest if not argument.startswith("-")))
    key = "scripts/experiment-apparatus/replay.sh"
    if numbers:
        return RunShape("items", f"实验复跑点名的实验 {'、'.join(numbers)}", len(numbers), "个实验", counted_text(len(numbers), "个实验"), key, "replay")
    return RunShape("whole", "实验复跑的全部实验（不带实验号）", None, "", "整轮", key, "replay")


MERGE_OPTIONS_WITH_VALUE = frozenset({"--item", "--crash-expand-up-to"})   # merge-and-verify 带一个值的选项（照它的用法行）：值不是补丁目录


def merge_patch_directories(arguments):
    """merge-and-verify 命令上给的补丁目录：不以 - 起头的词，减掉 --item、--crash-expand-up-to 后面那个值（--item=X 的写法不占下一个词），去重、按出现次序。"""
    found, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in MERGE_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if not argument.startswith("-"):
            found.append(argument)
        position += 1
    return list(dict.fromkeys(found))


def merge_run_shape(arguments):
    """合入后验证只跑补丁改到的测试目标，种类恒是 items（不带 --item 也不是 whole）：带 --item（给几次取并集，--item=X 也认）范围数是点名的项数，
    不带时是给的补丁目录个数（merge_patch_directories）；带 --list-items 只列这一趟可点的项，是问询、不要声明。
    弄坏开关 merge-as-whole 下照整轮（whole）判，merge-list-items-declared 下 --list-items 也要声明。"""
    rest = without_force(arguments)
    key = "scripts/gate/merge-and-verify.sh"
    query_arguments = MERGE_QUERY_ARGUMENTS - {"--list-items"} if guard_switch_on("merge-list-items-declared") else MERGE_QUERY_ARGUMENTS
    if query_form(rest, query_arguments):
        return None
    if guard_switch_on("merge-as-whole"):
        return RunShape("whole", "合入后验证（弄坏开关 merge-as-whole：照整轮判）", None, "", "整轮", key, "merge")
    items = named_option_values(rest, "--item")
    if items:
        return RunShape("items", f"合入后验证点名的项 {'、'.join(items)}（测试那一步只跑点名的测试目标；打补丁、fmt、clippy、build 照旧，验证包那一步只交点名的验证包目标里的）",
                        len(items), "项", counted_text(len(items), "项"), key, "merge")
    directories = merge_patch_directories(rest)
    return RunShape("items", f"合入后验证 {len(directories)} 份补丁（{'、'.join(directories) or '没给补丁目录'}；只跑补丁改到的与变异行点名的测试目标，判不出的那几种测试不跑、退 3）",
                    len(directories), "份补丁", counted_text(len(directories), "份补丁"), key, "merge")


def cargo_run_shape(arguments, directory):
    command = project_code_runs.cargo_subcommand(arguments, directory)
    if command is None or command.subcommand not in ("test", "t"):
        return None
    rest = command.rest
    separator = rest.index("--") if "--" in rest else len(rest)
    before, after = rest[:separator], rest[separator + 1:]
    if "--no-run" in before and not guard_switch_on("query-forms-declared"):
        return None
    if project_code_runs.libtest_lists_only(after) and not guard_switch_on("query-forms-declared"):
        return None
    packages, targets, filters, every_target, whole_workspace = [], [], [], False, False
    position = 0
    while position < len(before):
        word = before[position]
        following = before[position + 1] if position + 1 < len(before) else ""
        if word in ("-p", "--package"):
            packages.append(following)
            position += 2
            continue
        if word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
        elif word.startswith("-p") and not word.startswith("--") and len(word) > 2:
            packages.append(word[2:])
        elif word in CARGO_NAMED_TARGET_OPTIONS:
            targets.append(f"{word} {following}")
            position += 2
            continue
        elif word.startswith(tuple(option + "=" for option in CARGO_NAMED_TARGET_OPTIONS)):
            targets.append(word.replace("=", " ", 1))
        elif word in CARGO_SINGLE_TARGET_FLAGS:
            targets.append(word)
        elif word in CARGO_EVERY_TARGET_FLAGS:
            every_target = True
        elif word in CARGO_WHOLE_WORKSPACE_FLAGS:
            whole_workspace = True
        elif word in project_code_runs.CARGO_TEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        elif not word.startswith("-"):
            filters.append(word)
        position += 1
    position = 0
    while position < len(after):
        word = after[position]
        if word in project_code_runs.LIBTEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        if not word.startswith("-"):
            filters.append(word)
        position += 1
    packages = list(dict.fromkeys(package for package in packages if package))
    targets = list(dict.fromkeys(targets))
    filters = list(dict.fromkeys(filters))
    package_text = f"-p {'、'.join(packages)} " if packages else ""
    if whole_workspace or every_target or not targets or guard_switch_on("cargo-targets-as-whole"):
        if whole_workspace or not packages:
            return RunShape("whole", f"cargo test {package_text}不挑目标（整工作区，或当前目录那个包的全部测试目标）", None, "", "整轮", None, "cargo")
        return RunShape("whole", f"cargo test 整包 {'、'.join(packages)}（不挑目标，全部测试二进制）", None, "", "整包", None, "cargo")
    if "--exact" in after and filters:
        return RunShape("target", f"cargo test {package_text}{'、'.join(targets)} 里按全名点名的测试 {'、'.join(filters)}", len(filters), "个测试",
                        counted_text(len(filters), "个测试"), None, "cargo")
    return RunShape("target", f"cargo test {package_text}的测试目标 {'、'.join(targets)}", len(targets), "个测试目标",
                    counted_text(len(targets), "个测试目标"), None, "cargo")


def run_shape_of(words, directory):
    """剥完包装的一条命令是不是要声明的验证脚本：是就交 RunShape，不是（或是只查清单的写法）交 None。"""
    if not words or project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    word, arguments = words[0], words[1:]
    name = os.path.basename(word)
    stage = project_code_runs.gate_stage_name(word, directory)
    if stage is not None:
        return stage_run_shape(stage, arguments)
    if name in ("crash-point-replay.sh", "crash-amplification.sh", "crash-amplification-item.sh", "crash-amplification-two-hosts.sh") \
            and project_code_runs.located_in(word, directory, "scripts/crash"):
        return crash_run_shape(name, arguments)
    if name == "replay.sh" and project_code_runs.located_in(word, directory, "scripts/experiment-apparatus"):
        return replay_run_shape(arguments)
    if name == "merge-and-verify.sh" and project_code_runs.located_in(word, directory, "scripts/gate"):
        return merge_run_shape(arguments)
    if name == "gate-staged.sh" and project_code_runs.located_in(word, directory, "scripts/gate"):
        if query_form(without_force(arguments), SELFTEST_ONLY_QUERY_ARGUMENTS):
            return None
        return RunShape("whole", "提交前的整轮门禁（gate-staged.sh 起整轮 gate.sh --staged）", None, "", "整轮", "scripts/gate/gate-staged.sh", "gate-whole")
    if name == "gate.sh" and any(project_code_runs.located_in(word, directory, location) for location in GATE_WHOLE_LOCATIONS):
        return RunShape("whole", "整轮门禁 gate.sh", None, "", "整轮", ".claude/singlefs-ai-sop/scripts/gate.sh", "gate-whole")
    if name == "cargo":
        return cargo_run_shape(arguments, directory)
    return None


class ScopeDeclaration(NamedTuple):
    segments: dict
    problems: list


def parse_scope_declaration(text):
    """SINGLEFS_RUN_SCOPE 的值按 ; 切成四段；值里自带的 ; 接到上一段后面。"""
    segments, problems, current = {}, [], None
    for raw in text.split(";"):
        part = raw.strip()
        found = SEGMENT_START.match(part)
        if found:
            key, value = found.group(1), found.group(2).strip()
            if key in segments:
                problems.append(f"「{key}」一段写了两次")
            segments[key] = value
            current = key
        elif current is not None:
            if part:
                segments[current] = f"{segments[current]}; {part}"
        elif part:
            problems.append(f"认不出「{part[:40]}」：每段写成 键=值，键只认 种类、跑什么、范围、为什么")
    for key in SEGMENT_KEYS:
        if key not in segments:
            problems.append(f"缺「{key}」一段")
        elif not segments[key]:
            problems.append(f"「{key}」一段是空的")
    kind = segments.get("种类")
    if kind and kind not in SCOPE_KINDS:
        problems.append(f"种类「{kind}」认不出，只认 whole、items、target")
    return ScopeDeclaration(segments, problems)


def reason_character_count(reason):
    """为什么一段的字数：去掉不说理由的词，再只数字母与数字（汉字算字母），空白与标点不算。"""
    for filler in REASON_FILLER_WORDS:
        reason = reason.replace(filler, "")
    return sum(1 for character in reason if unicodedata.category(character)[0] in ("L", "N"))


def commit_gate_problem(shape, submit):
    """提交时才许不指定范围地跑的那几道门禁与入口（UNSCOPED_COMMIT_ENTRIES）：经 task-server/task submit 交、`--` 之前一个放宽标识（LOOSENING_SUBMIT_FLAGS）都没带的，
    交一句说明（出路写在里面）；别的交 None。不经 submit 起的（直接跑）不在这里判，由 project-code-guard.sh 拒。弄坏开关 commit-whole-flag-unchecked 下不判。"""
    if submit is None or shape.kind != "whole" or shape.script_key not in UNSCOPED_COMMIT_ENTRIES or guard_switch_on("commit-whole-flag-unchecked"):
        return None
    if shape.script_key == CRASH_AMPLIFICATION_ENTRY:
        if "--full-gate" in submit:
            return None
        return (f"{COMMIT_PROBLEM_PREFIX}{shape.what}，这一条经 submit 交却没带 --full-gate：崩溃放量全部分项每次提交必跑、与六道同列，"
                "照 .claude/skills/crash-test/commit-stages.md 带 --full-gate（整包确认标识 --whole-package-confirmed 顶替不了它）；平时改成点名项（--item <项>）")
    if any(flag in submit for flag in LOOSENING_SUBMIT_FLAGS):
        return None
    if shape.script_key.startswith(".claude/gate.d/"):
        return (f"{COMMIT_PROBLEM_PREFIX}{shape.what}，这一条经 submit 交却没带 --full-gate："
                "提交时照 .claude/skills/crash-test/commit-stages.md 带 --full-gate（会弹窗要用户确认）；平时改成点名项或格（--item <项> / --check <格>）")
    return (f"{COMMIT_PROBLEM_PREFIX}{shape.what}，这一条经 submit 交却既没带 --full-gate、也没带 --whole-package-confirmed："
            "提交时的崩溃点重放走门禁 .claude/gate.d/crash-point-replay.sh，照 .claude/skills/crash-test/commit-stages.md 带 --full-gate（会弹窗要用户确认）；"
            "平时改成点名项（--item <项>）")


def count_basis(shape):
    """闸数范围一段那个数的办法，按命令的形状写一句（范围对不上的说明里用）。"""
    if shape.family == "stage":
        return "--check 点名的格（逗号隔的逐个算）与 --item 点名的项各自去重后相加；有 --item 单位写项，只有 --check 单位写格"
    if shape.family == "crash":
        if shape.kind == "target":
            return "起一个测试目标算 1 个（带 --test-filter 的单位写个测试）"
        return "--item 点名的项去重后数（--crash-expand-up-to 后面的深度值不算项）"
    if shape.family == "replay":
        return "命令里不以 - 起头的词当实验号、去重后数，单位是个实验（范围里别的单位，如「行」「次」，不当实验数）"
    if shape.family == "merge":
        if shape.unit == "项":
            return "--item 点名的项去重后数"
        return "不带 --item 时数补丁目录份数（不以 - 起头的词，--item、--crash-expand-up-to 后面的值不算）"
    if shape.family == "cargo":
        if shape.unit == "个测试":
            return "`--` 之后带 --exact 又给了筛选词时数筛选词个数"
        return "--test / --bin / --example / --bench 点名的目标与 --lib、--doc 各算一个"
    return ""


def variable_built_words(words):
    """命令的参数里写成变量、命令替换的词（含 `--item=$名`）：闸只数写成字面的 --item / --check，这些词里拼出来的数不到。"""
    return [word for word in words[1:] if "$" in word or "`" in word]


def variable_built_hint(words):
    """范围或种类对不上的说明末尾加的一句：命令里有变量拼的词时点名它们、出路是把 --item 写成字面。弄坏开关 variable-built-hint-dropped 下不写。"""
    built = [] if guard_switch_on("variable-built-hint-dropped") else variable_built_words(words or [])
    if not built:
        return ""
    shown = "、".join(word if len(word) <= 40 else word[:40] + "…" for word in built[:3])
    return f"；命令里的 {shown} 是变量或命令替换，闸只数写成字面的 --item / --check，数不到里面拼的：把 --item 写成字面（--check 的格名同样写成字面）"


def declaration_to_write(shape):
    """说明里写这条命令该写的整段声明（种类与范围，数与单位都是闸自己数的那一个）。弄坏开关 count-hint-unprinted 下不写。"""
    if guard_switch_on("count-hint-unprinted"):
        return ""
    return f"；这条命令该写「种类={shape.kind}」「范围={shape.range_hint}」"


def scope_problems(shape, declaration_text, agent_type, client_refusal=None, submit=None, words=None):
    """这一条要声明的命令哪里不合：交回一串说明，空的是放行。client_refusal 是交它的那一条客户端命令照 lib_project_code_runs.subagent_task_server_refusal 判出的说明
    （经 task-server/task submit 交的才有）：子 agent 交的照它拒，种类不论（子 agent 一律不往统一任务服务交任务，
    records/2026-11-17-subagent账本自上而下清欠.md 第一百七十四节；判定与 project-code-guard.sh 的 ② 是同一处，这里不另写）。子 agent 直接起的验证命令这里只判声明，
    直接跑项目代码由 project-code-guard.sh 的 ① 拒。弄坏开关 subagent-submissions-allowed 下子 agent 交的不拒（许子 agent 自己交）。
    submit 是交它的那一条 task-server/task submit 带的放宽标识（FoundRun.submit）：主 agent 交的不指定范围的提交时门禁与入口没带放宽标识的照 commit_gate_problem 拒。
    种类或范围数对不上的说明写这条命令该写的整段「种类=…」「范围=<数> <单位>」（declaration_to_write，数与单位是闸数出来的那一个）、数法（count_basis）；
    words 是剥完包装的命令词，里面有变量拼的词时另写「把 --item 写成字面」（variable_built_hint）。"""
    problems = []
    if agent_type and client_refusal and not guard_switch_on("subagent-submissions-allowed"):
        problems.append(f"子 agent 不往统一任务服务交任务：这一条经 {client_refusal} 交的是{shape.what}。{project_code_runs.SUBAGENT_HAND_BACK_WAY_OUT}")
    commit_problem = None if agent_type else commit_gate_problem(shape, submit)
    if commit_problem:
        problems.append(commit_problem)
    if declaration_text is None:
        if not guard_switch_on("declaration-optional"):
            problems.append("没带范围声明")
        return problems
    declaration = parse_scope_declaration(declaration_text)
    if not guard_switch_on("segments-unchecked"):
        problems += declaration.problems
    segments = declaration.segments
    declared_kind = segments.get("种类")
    if declared_kind in SCOPE_KINDS and declared_kind != shape.kind:
        if not guard_switch_on("kind-unchecked"):
            problems.append(f"声明的是{KIND_DESCRIPTIONS[declared_kind]}，命令跑的是{KIND_DESCRIPTIONS[shape.kind]}：{shape.what}"
                            f"{declaration_to_write(shape)}{variable_built_hint(words)}")
    range_text = segments.get("范围", "")
    if range_text and declared_kind == shape.kind and not guard_switch_on("count-unchecked"):
        if shape.kind in ("items", "target"):
            numbers = re.findall(r"\d+", range_text)
            if not numbers or int(numbers[0]) != shape.count:
                if guard_switch_on("count-hint-unprinted"):
                    problems.append(f"范围写的是「{range_text}」，命令点名的是 {shape.count} {shape.unit}：范围一段开头写这个数（阿拉伯数字）{variable_built_hint(words)}")
                else:
                    problems.append(f"范围写的是「{range_text}」，闸数出这条命令点名的是 {shape.count} {shape.unit}（{shape.what}；数法：{count_basis(shape)}）"
                                    f"{declaration_to_write(shape)}，范围一段开头是阿拉伯数字{variable_built_hint(words)}")
        elif shape.kind == "whole" and not any(word in range_text for word in WHOLE_RANGE_WORDS):
            problems.append(f"范围写的是「{range_text}」，命令跑的是整轮：whole 的范围写「整轮」或「整包」")
    reason = segments.get("为什么", "")
    if reason and not guard_switch_on("reason-unchecked") and not (shape.family == "gate-whole" and COMMIT_WHOLE_REASON in reason):
        counted = reason_character_count(reason)
        if counted < REASON_MINIMUM_CHARACTERS:
            problems.append(f"为什么一段只有 {counted} 个字（去掉空白、标点与「要跑」「验证」这类不说理由的词之后）：至少 {REASON_MINIMUM_CHARACTERS} 个字，"
                            "写为什么这么跑、这个范围为什么合适")
    return problems


def submit_loosening_flags(words):
    """task-server/task submit 在 `--` 之前带的放宽标识（LOOSENING_SUBMIT_FLAGS 的键），去重、按出现次序。words 是 task_server_submission 认过的客户端命令：`--` 之前的选项都是写出来的。"""
    separator = words.index("--") if "--" in words else len(words)
    return tuple(dict.fromkeys(word for word in words[2:separator] if word in LOOSENING_SUBMIT_FLAGS))


def verification_test_targets(test_arguments):
    """cargo test 的参数（test 之后的）里 --test 点名的验证包测试目标，去重、按出现次序。点名的包里没有验证包、带 --no-run、只列用例的交空表。"""
    separator = test_arguments.index("--") if "--" in test_arguments else len(test_arguments)
    before, after = test_arguments[:separator], test_arguments[separator + 1:]
    if "--no-run" in before or project_code_runs.libtest_lists_only(after):
        return []
    packages, targets, position = [], [], 0
    while position < len(before):
        word = before[position]
        following = before[position + 1] if position + 1 < len(before) else ""
        if word in ("-p", "--package"):
            packages.append(following)
            position += 2
            continue
        if word == "--test":
            targets.append(following)
            position += 2
            continue
        if word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
        elif word.startswith("--test="):
            targets.append(word.split("=", 1)[1])
        elif word.startswith("-p") and not word.startswith("--") and len(word) > 2:
            packages.append(word[2:])
        elif word in project_code_runs.CARGO_TEST_OPTIONS_WITH_VALUE:
            position += 2
            continue
        position += 1
    if packages and VERIFICATION_PACKAGE not in packages:
        return []
    return list(dict.fromkeys(target for target in targets if target))


def crash_subject_kind(words, directory):
    """这一条是不是崩溃域闸要判的四种指定范围的形状：崩溃点重放入口点名项（replay-items）、cargo test 点名验证包测试目标（cargo-targets）、
    证红脚本 scripts/mutation/prove-red.sh（prove-red）、变异门禁 crates-mutation-replay.sh 点名行（mutation-items）；是交种类，不是交 None。
    只查清单、自证、帮助的写法不算。"""
    if not words or project_code_runs.ENVIRONMENT_ASSIGNMENT_WORD.match(words[0]):
        return None
    word, arguments = words[0], without_force(list(words[1:]))
    name = os.path.basename(word)
    if project_code_runs.gate_stage_name(word, directory) == "crates-mutation-replay.sh":
        listing = any(argument in STAGE_QUERY_ARGUMENTS for argument in arguments)
        return "mutation-items" if named_option_values(arguments, "--item") and not listing else None
    if name == "crash-point-replay.sh" and project_code_runs.located_in(word, directory, "scripts/crash"):
        listing = any(argument in CRASH_REPLAY_QUERY_ARGUMENTS for argument in arguments)
        return "replay-items" if named_option_values(arguments, "--item") and not listing else None
    if name == "prove-red.sh" and project_code_runs.located_in(word, directory, "scripts/mutation"):
        return "prove-red" if len(arguments) >= 4 and arguments[0] == "--copy" else None
    if name == "cargo":
        command = project_code_runs.cargo_subcommand(arguments, directory)
        if command is not None and command.subcommand in ("test", "t") and verification_test_targets(list(command.rest)):
            return "cargo-targets"
    return None


def crash_shrink_option_written(subject):
    """命令里写了 --crash-expand-up-to（崩溃点重放入口认它）。"""
    return any(word == CRASH_SHRINK_OPTION or word.startswith(CRASH_SHRINK_OPTION + "=") for word in subject.words)


def crash_shrink_variables_written(subject):
    """这一条看得到的环境里设了（值非空）的缩域变量，照 CRASH_SHRINK_VARIABLES 的次序。"""
    return [name for name in CRASH_SHRINK_VARIABLES if subject.environment.get(name)]


def full_domain_reason_written(declaration_text):
    """范围声明的为什么一段里写了「全域理由：<理由>」，理由去掉不说理由的词之后至少 FULL_DOMAIN_REASON_MINIMUM_CHARACTERS 个字。弄坏开关 crash-domain-reason-length-unchecked 下写了就算。"""
    if not declaration_text:
        return False
    reason = parse_scope_declaration(declaration_text).segments.get("为什么", "")
    found = FULL_DOMAIN_REASON_PATTERN.search(reason)
    if found is None:
        return False
    return guard_switch_on("crash-domain-reason-length-unchecked") or reason_character_count(found.group(1)) >= FULL_DOMAIN_REASON_MINIMUM_CHARACTERS


def read_text_lines(path):
    """文件按换行切成的行；读不了（不在、不是 UTF-8、没权限）交 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.read().split("\n")
    except (OSError, UnicodeDecodeError):
        return None


def rust_code_without_comments(text):
    """Rust 源码去掉 // 与 /* */（可嵌套）注释之后的文本，字符串、原始字符串（r"…"、r#"…"#）与字符字面量照留：
    判点到的目标读哪个缩域变量只认代码里写的变量名，注释里提到的不算。没收尾的字符串与注释照到文末算。"""
    kept, position, length = [], 0, len(text)
    while True:
        found = RUST_LEXICAL_START.search(text, position)
        if found is None:
            kept.append(text[position:])
            return "".join(kept)
        kept.append(text[position:found.start()])
        token = found.group(0)
        if token == "//":
            end = text.find("\n", found.end())
            position = length if end < 0 else end
        elif token == "/*":
            depth, position = 1, found.end()
            while depth:
                edge = RUST_BLOCK_COMMENT_EDGE.search(text, position)
                if edge is None:
                    position = length
                    break
                depth, position = depth + (1 if edge.group(0) == "/*" else -1), edge.end()
        elif token.startswith("r"):
            closing = '"' + found.group(1)
            end = text.find(closing, found.end())
            position = length if end < 0 else end + len(closing)
            kept.append(text[found.start():position])
        elif token == '"':
            string = RUST_STRING_LITERAL.match(text, found.start())
            position = string.end() if string else length
            kept.append(text[found.start():position])
        else:
            kept.append(token)
            position = found.end()


def registry_root(directory, project_root):
    """登记表所在的树的根：从命令的当前目录往上找带 STAGE_INPUTS_TABLE 的那一层，找不到取 project_root。"""
    current = os.path.abspath(directory) if directory else None
    while current:
        if os.path.isfile(os.path.join(current, STAGE_INPUTS_TABLE)):
            return current
        parent = os.path.dirname(current)
        if parent == current:
            break
        current = parent
    return project_root


def registered_crash_targets(root):
    """登记表里走崩溃枚举的验证包测试目标：stage-inputs.tsv 的 crash-case: 行点名的，加崩溃放量入口 ITEMS 里分项点名的；交 (目标集合, 读不了登记表的说明或 None)。
    弄坏开关 crash-domain-registry-unread 下交空集合。"""
    if guard_switch_on("crash-domain-registry-unread"):
        return set(), None
    lines = read_text_lines(os.path.join(root, STAGE_INPUTS_TABLE))
    if lines is None:
        return set(), f"读不了 {os.path.join(root, STAGE_INPUTS_TABLE)}"
    targets = set()
    for line in lines:
        if line.startswith("crash-case:"):
            targets.update(CRASH_CASE_TEST_WORD.findall(line))
    amplification, inside = read_text_lines(os.path.join(root, CRASH_AMPLIFICATION_ENTRY)) or [], False
    for line in amplification:
        if line.startswith("ITEMS=("):
            inside = True
        elif inside and line.startswith(")"):
            break
        elif inside:
            found = AMPLIFICATION_ITEM_LINE.match(line)
            if found:
                targets.add(found.group(1))
    return targets, None


def crash_target_enumerates(root, target, registered):
    """验证包测试目标会不会走崩溃枚举：登记过的是；没登记的看测试源码里有没有读缩域变量（读它的用例能缩域，就是会枚举全域）。
    测试源码是 tests/<目标>.rs，或目录式目标 tests/<目标>/main.rs 连同目录底下的全部 .rs（lib_run_scope_target_sources.py 的 target_source_text）。
    没登记又读不到源码的交 None（判不出）。弄坏开关 crash-domain-source-scan-skipped 下没登记的一律交 False。"""
    if target in registered:
        return True
    if guard_switch_on("crash-domain-source-scan-skipped"):
        return False
    text = target_sources.target_source_text(os.path.join(root, VERIFICATION_TESTS_DIRECTORY), target, read_text_lines, guard_switch_on)
    return None if text is None else any(name in text for name in CRASH_SHRINK_VARIABLES)


def target_shrink_variables(root, target, registered):
    """会走崩溃枚举的验证包测试目标读哪几个缩域变量（照 CRASH_SHRINK_VARIABLES 的次序；放行照它判，拒绝说明照它点名）：
    登记过的（崩溃枚举用例与放量分项经放量引擎）读放量那个；测试源码的代码里（rust_code_without_comments：注释里提到的不算）写了哪个变量名就读哪个，两样都占的两个都交。
    判不出的交 None：读不到测试源码（tests/<目标>.rs 与 tests/<目标>/main.rs 都没有，或名字不是一个词），或没登记而代码里两个变量名都没写。目录式目标读 main.rs 连同目录底下的全部 .rs。
    弄坏开关 crash-domain-target-variable-source-unread 下不读源码、一律交放量那个；crash-domain-target-variable-fixed 下读得到源码的一律交判器那个；
    crash-domain-variable-comments-counted 下注释里写的变量名也算。"""
    amplification, judge = CRASH_SHRINK_VARIABLES
    if guard_switch_on("crash-domain-target-variable-source-unread"):
        return [amplification]
    text = target_sources.target_source_text(os.path.join(root, VERIFICATION_TESTS_DIRECTORY), target, read_text_lines, guard_switch_on)
    if text is None:
        return None
    if guard_switch_on("crash-domain-target-variable-fixed"):
        return [judge]
    code = text if guard_switch_on("crash-domain-variable-comments-counted") else rust_code_without_comments(text)
    found = [name for name in CRASH_SHRINK_VARIABLES if name in code or (name == amplification and target in registered)]
    return found or None


class CrashDomains(NamedTuple):
    """崩溃域闸对一条命令点到的目标或项判出的四样（crash_subject_domains 交）。"""
    enumerating: list    # 会走崩溃枚举的（说明）
    unknown: list        # 判不出会不会走的（说明）
    variables: list      # 会走的那几个读的缩域变量的并（照 CRASH_SHRINK_VARIABLES 的次序）
    undetermined: list   # 会走而判不出读哪一个缩域变量的（说明）


def consulted_trees(roots):
    """崩溃域闸这一趟读的树：roots 排在前面的优先（crash_domain_trees）；弄坏开关 crash-domain-tree-fallback-dropped 下只读排头那一棵，里面没有的不退回别的树。"""
    return roots[:1] if guard_switch_on("crash-domain-tree-fallback-dropped") else list(roots)


def target_domains(roots, targets, whence):
    """点名的验证包测试目标逐个判，交 CrashDomains。每个目标取 roots 里第一棵判得出来的树的结论（登记过，或测试源码读得到），这一棵里判不出的才退回下一棵；
    登记表读不了的树只信「会走」、不信「不走」（排在最后一棵的照信）。whence 写这些目标是谁点的，进判不出的说明。"""
    trees = consulted_trees(roots)
    registrations = {root: registered_crash_targets(root) for root in trees}
    enumerating, variables, undetermined, unknown, notes = [], set(), [], [], []
    for target in targets:
        decided = None
        for position, root in enumerate(trees):
            registered, problem = registrations[root]
            verdict = crash_target_enumerates(root, target, registered)
            if verdict is None or (problem and not verdict and position < len(trees) - 1):
                continue
            decided = (root, registered, verdict)
            break
        for root in ([decided[0]] if decided else trees):   # 登记表读不了的说明只记定下这个目标的那一棵（没定下的记读过的每一棵），每一棵一句
            problem = registrations[root][1]
            if problem and f"{whence}：{problem}，登记过的崩溃枚举目标没核" not in notes:
                notes.append(f"{whence}：{problem}，登记过的崩溃枚举目标没核")
        if decided is None:
            unknown.append(f"{whence}：测试目标 {target} 既没登记、也读不到 {VERIFICATION_TESTS_DIRECTORY}/{target}.rs 与 {VERIFICATION_TESTS_DIRECTORY}/{target}/main.rs，没判会不会走崩溃枚举"
                           f"{'（读过 ' + '、'.join(trees) + '）' if len(trees) > 1 else ''}")
            continue
        root, registered, verdict = decided
        if verdict:
            enumerating.append(f"测试目标 {target}")
            names = target_shrink_variables(root, target, registered)
            if names is None:
                undetermined.append(f"{whence}：测试目标 {target} 会走崩溃枚举，判不出它读哪一个缩域变量"
                                    f"（读不到 {VERIFICATION_TESTS_DIRECTORY}/{target}.rs 与 {VERIFICATION_TESTS_DIRECTORY}/{target}/main.rs，或没登记而代码里两个变量名都没写）")
            else:
                variables.update(names)
    return CrashDomains(enumerating, notes + unknown, [name for name in CRASH_SHRINK_VARIABLES if name in variables], undetermined)


def replay_item_domains(items, roots):
    """崩溃点重放入口点名的项逐个判：合表（REPLAY_ITEMS_TABLE）里这一项点了 crash-case 键或放量分项的会走崩溃枚举；几棵树里都有的取排在前面那一棵的。
    交 CrashDomains：入口照放量那个变量缩（它另认 CRASH_SHRINK_OPTION）。"""
    tables = [lines for lines in (read_text_lines(os.path.join(root, REPLAY_ITEMS_TABLE)) for root in consulted_trees(roots)) if lines is not None]
    if not tables:
        return CrashDomains([], [f"崩溃点重放入口点名的项 {'、'.join(items)}：读不了 {REPLAY_ITEMS_TABLE}，没判会不会走崩溃枚举"], [], [])
    enumerates_by_item = {}
    for lines in reversed(tables):   # 排在前面的树后写，盖住后面的
        for line in lines:
            if not line.strip() or line.startswith("#"):
                continue
            fields = line.split("\t#", 1)[0].split("\t")
            if len(fields) == 3:
                enumerates_by_item[fields[0].strip()] = any(field.strip() not in ("", "-") for field in fields[1:])
    enumerating = [f"项 {item}" for item in items if enumerates_by_item.get(item)]
    unknown = [f"崩溃点重放入口点名的项 {item} 不在 {REPLAY_ITEMS_TABLE} 里，没判会不会走崩溃枚举" for item in items if item not in enumerates_by_item]
    return CrashDomains(enumerating, unknown, [CRASH_SHRINK_VARIABLES[0]], [])


def mutation_row_targets(table_lines, item):
    """变异表里行号或变异名等于 item 的行（同名的几行都算）第五段点名的验证包测试目标；表里一行也没对上交 None。"""
    matched, targets = False, []
    for number, line in enumerate(table_lines, 1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 5 or item not in (str(number), fields[0]):
            continue
        matched = True
        try:
            targets += verification_test_targets(shlex.split(fields[4]))
        except ValueError:
            continue
    return targets if matched else None


def mutation_item_domains(items, roots, whence):
    """变异行逐个判：点名的变异名或行号在 roots 里每棵树的变异表（MUTATION_TABLE）里按次序找，取第一张对得上的表里那一行（读不了的表跳过），
    行点名的测试目标再按 target_domains 判（同一份 roots）。交 CrashDomains。"""
    trees = consulted_trees(roots)
    tables = [lines for lines in (read_text_lines(os.path.join(root, MUTATION_TABLE)) for root in trees) if lines is not None]
    if not tables:
        return CrashDomains([], [f"{whence}：读不了变异表 {MUTATION_TABLE}（读过 {'、'.join(trees)}），没判会不会走崩溃枚举"], [], [])
    targets, unknown = [], []
    for item in items:
        found = next((row_targets for row_targets in (mutation_row_targets(lines, item) for lines in tables) if row_targets is not None), None)
        if found is None:
            unknown.append(f"{whence}点名的 {item} 不在变异表 {MUTATION_TABLE} 里（读过 {'、'.join(trees)}），没判会不会走崩溃枚举")
        else:
            targets += found
    domains = target_domains(roots, list(dict.fromkeys(targets)), f"{whence}点名的行")
    return domains._replace(unknown=unknown + domains.unknown)


def stage_positional_words(arguments):
    """门禁阶段的参数里不是选项、也不是 --item / --check 的值的词（项目根）。"""
    found, position = [], 0
    while position < len(arguments):
        argument = arguments[position]
        if argument in ("--item", "--check"):
            position += 2
            continue
        if not argument.startswith("-"):
            found.append(argument)
        position += 1
    return found


def known_tree_word(word, variables):
    """点名一棵树的词（submit 的 --cwd 值、证红的 --copy 值、变异门禁的根参数）：写成 `$名`、`${名}` 的，用这段命令文本里前面赋过的值展开
    （lib_shell_words.py 的 expand_known_variables，$HOME 也认）；不带变量的原样交回，展开不出（没赋过的变量、命令替换、带空白）交 None。
    弄坏开关 crash-domain-tree-variables-unexpanded 下一律原样交回（`$名` 当成路径里的字面字符）。"""
    if ("$" not in word and "`" not in word) or guard_switch_on("crash-domain-tree-variables-unexpanded"):
        return word
    value = shell_words.expand_known_variables(word, variables or {})
    return value if value and not re.search(r"\s", value) else None


def submit_words_with_known_directory(words, variables):
    """task-server/task submit 在 `--` 之前的 --cwd 写成赋过字面值的变量的（`S=<副本>; task-server/task submit --cwd $S -- …`），把值换成展开之后的（known_tree_word），
    别的词不动；展开不出的照原样。"""
    separator = words.index("--") if "--" in words else len(words)
    option = project_code_runs.TASK_SERVER_SUBMIT_DIRECTORY_OPTION
    changed = list(words)
    for position in range(2, separator):
        if changed[position] == option and position + 1 < separator:
            changed[position + 1] = known_tree_word(changed[position + 1], variables) or changed[position + 1]
        elif changed[position].startswith(option + "="):
            changed[position] = option + "=" + (known_tree_word(changed[position][len(option) + 1:], variables) or changed[position][len(option) + 1:])
    return changed


def crash_domain_trees(subject, project_root):
    """崩溃域闸读的树（变异表、登记表、测试源码都按这个次序，前面的树里有的以它为准，没有的才退回后面的）：命令点明的树（证红 --copy 点名的副本、变异门禁的根参数，
    写成赋过字面值的变量的先展开）、命令当前目录那棵树（registry_root：submit 的 --cwd 指到副本时就是那个副本）、项目根（主工作区）。
    交 ([树的根…], [展开不出的说明…])：同一棵（realpath 相同）只留最前面那一份，盘上不是目录的不读。弄坏开关 crash-domain-named-tree-unread 下命令点明的树不读。"""
    arguments, base = without_force(list(subject.words[1:])), subject.directory or project_root
    named = arguments[1:2] if subject.kind == "prove-red" else list(reversed(stage_positional_words(arguments))) if subject.kind == "mutation-items" else []
    candidates, notes = [], []
    for word in ([] if guard_switch_on("crash-domain-named-tree-unread") else named):
        value = known_tree_word(word, subject.variables)
        if value is None:
            notes.append(f"{CRASH_SUBJECT_DESCRIPTIONS[subject.kind]}：命令点名的树 {word} 是展开不出值的变量，没读它指的那棵树，读的是命令当前目录那一棵与项目根")
        else:
            candidates.append(os.path.join(base, os.path.expanduser(value)))
    candidates += [registry_root(subject.directory, project_root), project_root]
    roots, seen = [], set()
    for candidate in candidates:
        real = os.path.realpath(candidate)
        if os.path.isdir(real) and real not in seen:
            seen.add(real)
            roots.append(candidate)
    return roots, notes


def crash_subject_domains(subject, roots, notes=()):
    """这一条点到的目标或项判出的 CrashDomains；roots 与 notes 是 crash_domain_trees 交的读的树与展开不出的说明（说明并进判不出的）。
    弄坏开关 CRASH_SUBJECT_BREAK_SWITCHES 里这一种的那一项下不判（四样都空）。"""
    if guard_switch_on(CRASH_SUBJECT_BREAK_SWITCHES[subject.kind]):
        return CrashDomains([], [], [], [])
    arguments = without_force(list(subject.words[1:]))
    if subject.kind == "replay-items":
        domains = replay_item_domains(named_option_values(arguments, "--item"), roots)
    elif subject.kind == "cargo-targets":
        command = project_code_runs.cargo_subcommand(arguments, subject.directory)
        domains = target_domains(roots, verification_test_targets(list(command.rest)) if command is not None else [], "cargo test")
    elif subject.kind == "prove-red":
        domains = mutation_item_domains(list(dict.fromkeys(arguments[3:])), roots, "证红")
    else:
        domains = mutation_item_domains(named_option_values(arguments, "--item"), roots, "变异门禁")
    return domains._replace(unknown=list(notes) + domains.unknown)


def crash_domain_problem(subject, project_root, state):
    """指定范围的命令点到会走崩溃枚举的目标或项而没缩对域：交一句说明（出路写在里面，缩域变量照点到的目标点名）；放行交 None。
    放行的有：交它的那一条 submit 带 --full-gate（不指定范围的提交时那一趟另判）、命令里写了 --crash-expand-up-to、范围声明的为什么一段写了全域理由、
    这一条看得到的环境里设了点到的目标读的每一个缩域变量（target_shrink_variables；多设一个不拒）、
    点到的目标里有判不出读哪一个的而环境里设了两个缩域变量之一（照有一个就放，判不出的那一处记进 state.unjudged）。
    点到的目标或项判不出会不会枚举的，放行并把说明记进 state.unjudged。
    弄坏开关：crash-domain-full-gate-judged 下带 --full-gate 的也判；crash-domain-shrink-ignored 下写了 --crash-expand-up-to、设了缩域变量的都当没缩；
    crash-domain-reason-ignored 下写了全域理由的也判；crash-domain-unknown-unrecorded 下判不出的（会不会走、读哪一个）都不记；
    crash-domain-variable-unknown-refused 下判不出读哪一个的照没缩对拒；crash-domain-shrink-variable-unchecked 下设了两个缩域变量之一就放、不核是不是读的那一个；
    crash-domain-extra-variable-refused 下多设了点到的目标不读的那一个也拒。
    变异表、登记表与测试源码读的树见 crash_domain_trees（命令点明的副本在前、没有的退回命令当前目录那棵与项目根，读不到的说明记进 state.unjudged）；
    弄坏开关 crash-domain-named-tree-unread 下命令点明的树不读，crash-domain-tree-fallback-dropped 下只读排头那一棵，crash-domain-tree-variables-unexpanded 下 `$名` 写的树不展开。"""
    if subject.submit is not None and "--full-gate" in subject.submit and not guard_switch_on("crash-domain-full-gate-judged"):
        return None
    shrink_ignored = guard_switch_on("crash-domain-shrink-ignored")
    if crash_shrink_option_written(subject) and not shrink_ignored:
        return None
    if full_domain_reason_written(subject.environment.get(DECLARATION_VARIABLE)) and not guard_switch_on("crash-domain-reason-ignored"):
        return None
    roots, tree_notes = crash_domain_trees(subject, project_root)
    domains = crash_subject_domains(subject, roots, tree_notes)
    unrecorded = guard_switch_on("crash-domain-unknown-unrecorded")
    if domains.unknown and not unrecorded:
        state.unjudged += domains.unknown
    if not domains.enumerating:
        return None
    written = [] if shrink_ignored else crash_shrink_variables_written(subject)
    needed = domains.variables
    if written and domains.undetermined and not guard_switch_on("crash-domain-variable-unknown-refused"):
        if not unrecorded:
            state.unjudged += [f"{entry}；命令里设了 {'、'.join(written)}，照有一个缩域变量就放行" for entry in domains.undetermined]
        return None
    if written and guard_switch_on("crash-domain-shrink-variable-unchecked"):
        return None
    missing = [name for name in needed if name not in written]
    extra = [name for name in written if name not in needed] if guard_switch_on("crash-domain-extra-variable-refused") else []
    if written and not missing and not extra and not domains.undetermined:
        return None
    reads = f"点到的目标读的是 {'、'.join(needed)}" if needed else "判不出点到的目标读哪一个缩域变量"
    named = needed or list(CRASH_SHRINK_VARIABLES)
    way_out = (f"出路：带点到的目标读的缩域变量（经服务交的写成 `-- env {' '.join(name + '=<n>' for name in named)} <命令…>`"
               f"{'，崩溃点重放入口也可以加 ' + CRASH_SHRINK_OPTION + ' <n>' if subject.kind == 'replay-items' else ''}），"
               f"或在 {DECLARATION_VARIABLE} 的为什么一段写「全域理由：<至少 {FULL_DOMAIN_REASON_MINIMUM_CHARACTERS} 个字，说明为什么这一趟要判全部崩溃状态>」"
               "（证红脚本不要求声明，要写理由就带一份四段声明）")
    head = f"{CRASH_SUBJECT_DESCRIPTIONS[subject.kind]}会走崩溃枚举（{'、'.join(domains.enumerating)}）"
    limit = "指定范围的那几趟崩溃状态数压在 20 万量级（.claude/rules/verification.md「一个门禁：两种跑法」）。"
    if not written:
        return (f"崩溃域没缩：{head}，命令与环境里都没带缩域（{CRASH_SHRINK_OPTION}、{'、'.join(CRASH_SHRINK_VARIABLES)}），"
                f"范围声明的为什么一段也没写全域理由；{reads}。{limit}{way_out}")
    wrong = [f"少了 {'、'.join(missing)}（它读的那一个没设，域没缩）"] if missing else []
    wrong += [f"多了 {'、'.join(extra)}（它不读）"] if extra else []
    wrong += domains.undetermined
    return f"崩溃域没缩对：{head}，{reads}，命令里设的是 {'、'.join(written)}：{'；'.join(wrong)}。{limit}{way_out}"


def unnamed_submission_problem(run, state):
    """经 task-server/task submit 交给服务的一条（SubmittedRun）点没点名：判据只调 task-server/task_server_lib.py 的 unnamed_run_problem
    （服务提交时拒 ⑦ 同一个函数；经 lib_project_code_runs.task_server_library 载入，不另写一份）。→ 那个函数交的 {problem, next_step}，点了名交 None。
    不指定范围的提交时门禁与两份崩溃入口（UNSCOPED_COMMIT_ENTRIES 的 whole 那一形）归 commit_gate_problem 判，这里不重复判。
    task_server_lib.py 载不进、判的时候抛了：放行，那一处记进 state.unjudged。
    弄坏开关 unnamed-full-gate-dropped 下不把 --full-gate 交给判据，unnamed-confirmed-dropped 下不把 --whole-package-confirmed 交给判据。"""
    shape = run_shape_of(list(run.words), run.directory)
    if shape is not None and shape.kind == "whole" and shape.script_key in UNSCOPED_COMMIT_ENTRIES:
        return None
    library, problem = project_code_runs.task_server_library()
    if library is None:
        state.unjudged.append(f"点没点名判不了（{problem}）：{shlex.join(run.words)[:200]}")
        return None
    full_gate = "--full-gate" in run.submit and not guard_switch_on("unnamed-full-gate-dropped")
    confirmed = "--whole-package-confirmed" in run.submit and not guard_switch_on("unnamed-confirmed-dropped")
    try:
        return library.unnamed_run_problem(list(run.words), run.directory, full_gate, confirmed)
    except Exception as error:   # 判据出事：放行，记一处判不了
        state.unjudged.append(f"点没点名判不了（task_server_lib.unnamed_run_problem 抛了 {error!r}）：{shlex.join(run.words)[:200]}")
        return None


LAUNCHER_PROGRAMS = {"shell": "bash", "source": "bash", "python": "python3"}   # CommandAtPosition.launcher → 起那份脚本的程序（剥掉的那个词补回来用）


def sharded_wrapped_entry(candidate, state):
    """交给服务的那一条里的一条命令（WrappedCandidate）是不是服务会分片的入口、而交给服务的那一条自己在命令位置上不是：是就交那份入口，不是交 None。
    判法只调 task_server_lib.py 的 sharded_entry（服务提交时认「这件要分片」的同一个函数，经 lib_project_code_runs.task_server_library 载入，不另写一份）：
    服务只看交给它的那一条的命令位置上的词认分片，入口套在驱动脚本或 bash -c 里认不出，整件不分片，门禁只走合并核账。交的是 sharded_entry 的返回
    （仓根下的相对路径，或声明了分片的脚本的绝对路径）。task_server_lib.py 载不进、当前目录算不出、判的时候抛了：放行，那一处记进 state.unjudged（同样的话只记一处）。"""
    library, problem = project_code_runs.task_server_library()
    submitted = shlex.join(candidate.submitted_words)[:200]
    unjudged = None
    if library is None:
        unjudged = f"分片入口套没套在驱动脚本或 bash -c 里判不了（{problem}）：{submitted}"
    elif candidate.directory is None or candidate.task_directory is None:
        unjudged = f"分片入口套没套在驱动脚本或 bash -c 里判不了（当前目录算不出）：{submitted}"
    else:
        try:
            if library.sharded_entry(list(candidate.submitted_words), candidate.task_directory):
                return None
            return library.sharded_entry(list(candidate.words), candidate.directory) or None
        except Exception as error:   # 判据出事：放行，记一处判不了
            unjudged = f"分片入口套没套在驱动脚本或 bash -c 里判不了（task_server_lib.sharded_entry 抛了 {error!r}）：{submitted}"
    if unjudged not in state.unjudged:
        state.unjudged.append(unjudged)
    return None


def candidate_command_words(candidate):
    """这一条命令照原样起时的词：脚本经 bash / python3 起的，补回被剥掉的那个程序。"""
    program = LAUNCHER_PROGRAMS.get(candidate.launcher)
    return ([program] if program else []) + list(candidate.words)


def direct_submission_command(candidate):
    """把这一条直接交给服务的整条命令：声明照这一条自己带的（没带的留尖括号让人填），当前目录取这一条跑时的目录，命令词在命令位置上。"""
    scope = f"{DECLARATION_VARIABLE}={shlex.quote(candidate.declaration)}" if candidate.declaration else f"{DECLARATION_VARIABLE}='<种类=…; 跑什么=…; 范围=…; 为什么=…>'"
    directory = f" --cwd {shlex.quote(candidate.directory)}" if candidate.directory else ""
    return f"{scope} task-server/task submit{directory} -- {shlex.join(candidate_command_words(candidate))}"


def repository_tooling_script(real_path):
    """仓里的工具脚本：落在某一层带 .claude/hooks/ 的目录下的 scripts/ 或 .claude/ 里（主工作区与仓副本都算）。"""
    current = os.path.dirname(real_path)
    while True:
        parent = os.path.dirname(current)
        if parent == current:
            return False
        if os.path.basename(current) in ("scripts", ".claude") and os.path.isdir(os.path.join(parent, ".claude", "hooks")):
            return True
        current = parent


def remember_heredoc_writes(command, state):
    """`cat > 文件 <<EOF`、`cat <<EOF > 文件`、`tee [-a] 文件 <<EOF`：把 heredoc 正文记成那个文件此刻的内容（追加的接在后面）。"""
    if not command.standard_input_heredocs or not command.words:
        return
    body = command.standard_input_heredocs[-1].text
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    targets = []
    if name == "cat" and not arguments:
        targets = [(target, operator in (">>", "&>>")) for operator, target in command.output_redirections if operator in (">", ">|", ">>", "&>", "&>>")]
    elif name == "tee":
        appending = any(argument in ("-a", "--append") for argument in arguments)
        targets = [(argument, appending) for argument in arguments if not argument.startswith("-")]
    for target, appending in targets:
        if UNREADABLE_PATH_CHARACTERS & set(target):
            continue
        resolved = shell_words.resolve_path(command.directory, os.path.expanduser(target))
        if resolved is None:
            continue
        real = os.path.realpath(resolved)
        earlier = ""
        if appending:
            earlier = state.written.get(real)
            if earlier is None:
                try:
                    with open(real, encoding="utf-8") as handle:
                        earlier = handle.read()
                except (OSError, UnicodeDecodeError):
                    earlier = ""
        state.written[real] = earlier + body


def interpreter_of_head(head):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。"""
    if not head.startswith(b"#!"):
        return None
    first = head[2:].split(b"\n", 1)[0].decode("utf-8", "replace").split()
    if not first:
        return ""
    program = os.path.basename(first[0])
    if program == "env":
        rest = [word for word in first[1:] if not word.startswith("-") and "=" not in word]
        return os.path.basename(rest[0]) if rest else ""
    return program


def script_path_word(command, variables):
    """bash / sh / source 起的脚本路径写成 `$名`、`${名}` 的（`S=<目录>; bash $S/x.sh`），用这段命令文本里前面赋过的值展开（variables 是外层交进来的，这一条自己的 command.variables 盖在上面；
    lib_shell_words.py 的 expand_known_variables，$HOME 也认）：展开得出、不带空白的换上，展开不出（没赋过的变量、for 的循环变量、命令替换）与别的写法照原样交回。
    弄坏开关 driver-path-variables-unexpanded 下一律照原样。"""
    word = command.words[0]
    if "$" not in word or "`" in word or command.launcher not in ("shell", "source") or guard_switch_on("driver-path-variables-unexpanded"):
        return word
    known = dict(variables or {})
    known.update(command.variables or {})
    value = shell_words.expand_known_variables(word, known)
    return value if value and not re.search(r"\s", value) else word


def driver_script_text(command, state, variables=None):
    """命令位置上执行的是不是要读进去的驱动脚本：是就交 (实际路径, 正文)，不是或读不到交 None。脚本路径写成赋过字面值的变量的先展开（script_path_word）。"""
    word = script_path_word(command, variables)
    if command.launcher == "python" or word.startswith("-") or UNREADABLE_PATH_CHARACTERS & set(word):
        return None
    expanded = os.path.expanduser(word)
    if command.launcher is None and "/" not in expanded:
        return None
    resolved = shell_words.resolve_path(command.directory, expanded)
    if resolved is None:
        return None
    real = os.path.realpath(resolved)
    if real in state.written:
        data = state.written[real].encode("utf-8")
    else:
        if not os.path.isfile(real):
            return None
        if repository_tooling_script(real) and not guard_switch_on("tooling-scripts-read"):
            return None
        try:
            with open(real, "rb") as handle:
                data = handle.read(MAXIMUM_SCRIPT_BYTES + 1)
        except OSError as error:   # 在而读不了：里面要声明的命令判不了，记一处（弄坏开关 driver-unread-unrecorded 下静默跳过）
            if not guard_switch_on("driver-unread-unrecorded"):
                state.unjudged.append(f"命令位置上执行的脚本 {real} 读不了（{type(error).__name__}: {error.strerror or error}），里面要声明的命令没判")
            return None
    if b"\0" in data[:SCRIPT_HEAD_BYTES]:
        return None
    if len(data) > MAXIMUM_SCRIPT_BYTES:   # 太大：是 shell 脚本的照判不了记一处（同上）
        interpreter = interpreter_of_head(data[:SCRIPT_HEAD_BYTES])
        shell_script = (command.launcher is not None or interpreter in shell_words.SHELL_NAMES
                        or (interpreter is None and (real.endswith(".sh") or resolved.endswith(".sh"))))
        if shell_script and not guard_switch_on("driver-unread-unrecorded"):
            state.unjudged.append(f"命令位置上执行的脚本 {real} 比 {MAXIMUM_SCRIPT_BYTES} 字节大，没读进去，里面要声明的命令没判")
        return None
    if command.launcher is None:
        interpreter = interpreter_of_head(data[:SCRIPT_HEAD_BYTES])
        if interpreter is None and not real.endswith(".sh") and not resolved.endswith(".sh"):
            return None
        if interpreter is not None and interpreter not in shell_words.SHELL_NAMES:
            return None
    if b"\0" in data:
        return None
    try:
        return real, data.decode("utf-8")
    except UnicodeDecodeError:
        return None


def script_command_line_numbers(script_text):
    """脚本里每条命令（剥完前缀与包装的词序列）出现在哪几行：逐个逻辑行（反斜杠续行接上）单独切，喂给非 shell 命令的 heredoc 正文换成空行、行号不变。"""
    numbered = {}
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    index = 0
    while index < len(lines):
        start_line, joined = index + 1, lines[index]
        while joined.endswith("\\") and index + 1 < len(lines):
            index += 1
            joined = joined + "\n" + lines[index]
        index += 1
        try:
            line_commands = shell_words.commands_at_command_position(joined).commands
        except Exception:
            continue
        for line_command in line_commands:
            numbered.setdefault(tuple(line_command.words), []).append(start_line)
    return numbered


def scope_runs_in_text(text, directory, environment=None, frames=(), script=None, state=None, variables=None):
    """text 里命令位置上每一条要声明的命令；包装剥掉往里判，执行的驱动脚本读进去递归判。
    variables 是外层这段命令文本里前面赋过的 shell 变量（名字 → 已展开的值）：交给服务的那一条与包装里面那一条是重新切词的另一段文本，里面看不到外层赋的值，
    由这里带进去（驱动脚本里起的不带：脚本是另一个 shell 进程，看不到外层没 export 的变量）。"""
    if state is None:
        state = ScanState()
    found, seen, numbered = [], {}, None
    scan = shell_words.commands_at_command_position(text, directory, environment)
    # 喂给从标准输入读命令的 shell、判不了的输入（lib_shell_words.py 的 shell_standard_input）：照常判字面的那一部分，这一处记进检出
    state.unjudged += [f"脚本 {script} 里{entry}" if script else entry for entry in scan.unjudged_shell_inputs]
    for command in scan.commands:
        key = tuple(command.words)
        occurrence = seen.get(key, 0)
        seen[key] = occurrence + 1
        if script is None:
            here = frames
        else:
            if numbered is None:
                numbered = script_command_line_numbers(text)
            line_numbers = numbered.get(key, [])
            here = frames + ((script, line_numbers[occurrence] if occurrence < len(line_numbers) else None),)
        remember_heredoc_writes(command, state)
        known = dict(variables or {})
        known.update(command.variables or {})
        # 经 task-server/task submit 交给服务的那一条（lib_project_code_runs.task_server_submission）：照一条命令接着判，声明认 submit 前面写的
        # （客户端把 SINGLEFS_RUN_SCOPE 当白名单变量带进任务）与 `--` 之后 env 写的（run-scope-guard.sh 文件头「经 task-server 提交」）；--cwd 写成赋过字面值的变量的先展开
        submitted = (None if guard_switch_on("task-server-submissions-unread")
                     else project_code_runs.task_server_submission(submit_words_with_known_directory(command.words, known), command.directory, command.launcher))
        if submitted is not None:
            submitted_words, submitted_directory = submitted
            client_refusal = project_code_runs.subagent_task_server_refusal(command.words, command.directory, command.launcher)
            outer_submit, state.submit = state.submit, submit_loosening_flags(list(command.words))
            outer_frames, state.submit_frames = state.submit_frames, here
            outer_submission, state.current_submission = state.current_submission, (tuple(submitted_words), submitted_directory or command.directory, here)
            try:
                found += [run._replace(client_refusal=client_refusal)
                          for run in scope_runs_in_text(shlex.join(submitted_words), submitted_directory or command.directory, command.environment, here, None, state, known)]
            finally:
                state.submit, state.submit_frames, state.current_submission = outer_submit, outer_frames, outer_submission
            continue
        launched = None
        if not guard_switch_on("wrappers-opaque"):
            launched = (project_code_runs.command_under_launcher(command.words, command.directory)
                        or project_code_runs.command_under_gpu_quiesce(command.words, command.directory))
        if launched is not None:
            inner_words, inner_directory = launched
            inner_start = shell_words.resolve_path(command.directory, inner_directory) if inner_directory else command.directory
            found += scope_runs_in_text(shlex.join(inner_words), inner_start, command.environment, here, None, state, known)
            continue
        # 交给服务的那一条（与它剥了包装的那一层）里命令位置上的每一条：点没点名事后照 unnamed_submission_problem 判（服务提交时拒 ⑦ 同一份判据）；
        #   读进去的驱动脚本里的不算（服务只判提交的这一条）。弄坏开关 unnamed-submissions-unjudged 下不收
        if state.submit is not None and here == state.submit_frames and not guard_switch_on("unnamed-submissions-unjudged"):
            state.submitted_runs.append(SubmittedRun(here, tuple(command.words), command.directory, state.submit))
        # 交给服务的那一条里每一条命令（含 bash -c、包装与读进去的驱动脚本里的）：服务只看交给它的那一条命令位置上的词认分片，套在里面的分片入口认不出，事后照 sharded_wrapped_entry 判。
        #   弄坏开关 sharded-wrapped-unjudged 下不收
        if state.current_submission is not None and not guard_switch_on("sharded-wrapped-unjudged"):
            state.wrapped_candidates.append(WrappedCandidate(here, tuple(command.words), command.launcher, command.directory, command.environment.get(DECLARATION_VARIABLE),
                                                             *state.current_submission))
        crash_kind = crash_subject_kind(command.words, command.directory)
        if crash_kind is not None:
            state.crash_subjects.append(CrashSubject(here, crash_kind, tuple(command.words), command.directory, dict(command.environment), state.submit, known))
        shape = run_shape_of(command.words, command.directory)
        if shape is not None:
            found.append(FoundRun(here, list(command.words), shape, command.environment.get(DECLARATION_VARIABLE), submit=state.submit))
            continue
        if guard_switch_on("scripts-unread") or len(here) >= MAXIMUM_SCRIPT_DEPTH:
            continue
        driver = driver_script_text(command, state, known)
        if driver is None:
            continue
        real, body = driver
        if real in {os.path.realpath(path) for path, _line in here if path}:
            continue
        inner_environment = {name: value for name, value in command.environment.items() if name != DECLARATION_VARIABLE}
        found += scope_runs_in_text(body, command.directory, inner_environment, here, real, state)
    return found


class SubmitFlagAsk(NamedTuple):
    flags: tuple          # 这一条 submit 带的放宽标识
    unread_words: tuple   # `--` 之前算不出的词（变量、命令替换、xargs 这类读进来的）：认不出带没带标识
    problem: str = ""     # 判法没判成时的一句：flags 是从命令文字里按提到算的


def submit_client_command(command):
    """这一条的命令词是不是提交客户端：路径尾部是 task-server/task 的（当前目录算不出、前面是变量的同样算）、当前目录算不出而写成 ./task 这样带路径的，
    或解得开路径、对得上本仓那一份的（lib_project_code_runs.task_server_client_subcommand：cd 进 task-server 再起的、链过去的）。
    认错了只多弹一次窗，认漏了等于放行，所以路径尾部相同就算。弄坏开关 submit-flag-client-by-path-only 下只认解得开路径的。"""
    word, client = command.words[0], project_code_runs.TASK_SERVER_CLIENT
    if command.launcher not in ("startup", "program") and not guard_switch_on("submit-flag-client-by-path-only"):
        normalized = os.path.normpath(word)
        if normalized == client or normalized.endswith("/" + client):
            return True
        if command.directory is None and "/" in word and os.path.basename(normalized) == os.path.basename(client):
            return True
    return project_code_runs.task_server_client_subcommand(command.words, command.directory, command.launcher) is not None


def submit_option_pieces(word, variables):
    """submit 选项位上的一个词经外层 shell 展开之后的几个词；算不出的（没赋过的变量、命令替换、xargs 这类读进来的占位词）交 None。记成 `(元素 …)` 的数组去掉括号。"""
    if not shell_words.has_expansion([word]):
        return [word]
    value = shell_words.expand_known_variables(word, variables)
    return None if value is None else value.strip("()").split()


def submit_flag_ask(command):
    """这一条是 task-server/task submit、`--` 之前带放宽标识或有算不出的词：交 SubmitFlagAsk；别的交 None。`--` 之后是交给服务的那一条的参数，客户端不当选项，这里也不判。
    弄坏开关 submit-flag-after-separator 下 `--` 之后的也判，submit-flag-first-option-only 下只判紧跟 submit 的一个词，submit-flag-variables-unread 下算不出的词不算。"""
    variables = command.variables or {}
    if len(command.words) < 2 or not submit_client_command(command):
        return None
    head = submit_option_pieces(command.words[1], variables)
    if head is None:   # 子命令位算不出：可能就是 submit
        return None if guard_switch_on("submit-flag-variables-unread") else SubmitFlagAsk((), (command.words[1],))
    arguments = head + list(command.words[2:])
    if arguments[:1] == ["promote"]:   # 任务提前到队头：只在弹窗点了同意之后才放行
        return SubmitFlagAsk(("promote",), ())
    if arguments[:1] != ["submit"]:
        return None
    flags, unread = [], []
    for word in arguments[1:]:
        if word == "--" and not guard_switch_on("submit-flag-after-separator"):
            break
        pieces = submit_option_pieces(word, variables)
        if pieces is None:
            unread.append(word)
        else:
            flags += [flag for piece in pieces for flag in LOOSENING_SUBMIT_FLAGS if piece == flag or piece.startswith(flag + "=")]
        if guard_switch_on("submit-flag-first-option-only"):
            break
    if guard_switch_on("submit-flag-variables-unread"):
        unread = []
    return SubmitFlagAsk(tuple(flags), tuple(unread)) if flags or unread else None


def texts_launched_by(command, function_launchers, state):
    """这一条里面还会起的几段命令文本：[(文本, 当前目录)]。交给服务的那一条、flock 这类包装里的那一条、脚本里转发函数的参数、程序里字面写着的、命令词上赋过字面值的变量展开之后的、
    命令位置上执行的驱动脚本的正文（一次判定里每份只读一遍）。"""
    words, directory, nested = command.words, command.directory, []
    if submit_client_command(command) and "--" in words:
        nested.append((shlex.join(words[words.index("--") + 1:]), directory))
    launched = (project_code_runs.command_under_launcher(words, directory) or project_code_runs.command_under_gpu_quiesce(words, directory)
                or project_code_runs.command_under_dispatch_launcher(words, directory))
    if launched is not None:
        inner_words, inner_directory = launched
        nested.append((shlex.join(inner_words), shell_words.resolve_path(directory, inner_directory) if inner_directory else directory))
    if command.launcher is None and os.path.basename(words[0]) in function_launchers:
        nested += [(shlex.join(words[start:]), directory) for start in range(1, len(words))]
    nested += [(text, directory) for text in shell_words.embedded_launches(command).texts]
    if shell_words.has_expansion(words[:1]):
        value = shell_words.expand_known_variables(words[0], command.variables or {})
        if value is not None:
            nested.append((" ".join([value] + [shlex.quote(word) for word in words[1:]]), directory))
    driver = driver_script_text(command, state)
    if driver is not None and driver[0] not in state.flag_scripts_read:
        state.flag_scripts_read.add(driver[0])
        nested.append((driver[1], directory))
    return nested


def submit_flag_asks_in_text(text, directory, environment, state, depth=0):
    """text 里命令位置上每一条带放宽标识的提交（SubmitFlagAsk）；里面还会起的接着判（texts_launched_by，弄坏开关 submit-flag-nesting-unread 下不往里判）。"""
    asks = []
    function_launchers = shell_words.launcher_functions(text)
    for command in shell_words.commands_at_command_position(text, directory, environment).commands:
        remember_heredoc_writes(command, state)
        ask = submit_flag_ask(command)
        if ask is not None:
            asks.append(ask)
        if depth >= MAXIMUM_SCRIPT_DEPTH or guard_switch_on("submit-flag-nesting-unread"):
            continue
        for nested_text, nested_directory in texts_launched_by(command, function_launchers, state):
            asks += submit_flag_asks_in_text(nested_text, nested_directory, command.environment, state, depth + 1)
    return asks


def flag_asks_by_mention(command_text, problem):
    """按命令文字里提到标识算（判法没判成时才这样算：宁可多弹一次窗）：提到了交一条 SubmitFlagAsk，没提到交空表。"""
    flags = tuple(flag for flag in LOOSENING_SUBMIT_FLAGS if flag in command_text)
    return [SubmitFlagAsk(flags, (), problem)] if flags else []


def submit_flag_asks(command_text, directory, agent_type, state):
    """这条命令里要用户确认的提交。子 agent 不判（它们起 submit 由 project-code-guard.sh 拒）。这一段抛了不牵连范围声明：记一处判不了，命令文字里提到标识的照弹窗。
    弄坏开关 submit-flag-subagents-asked 下子 agent 也判，submit-flags-unjudged 下一律不判，submit-flags-by-substring 下命令文字里提到就算，submit-flag-raise-propagated 下抛了不接住。"""
    try:
        if (agent_type and not guard_switch_on("submit-flag-subagents-asked")) or guard_switch_on("submit-flags-unjudged"):
            return []
        asks = submit_flag_asks_in_text(command_text, directory, None, ScanState())
        return asks + flag_asks_by_mention(command_text, "") if guard_switch_on("submit-flags-by-substring") else asks
    except Exception as error:
        if guard_switch_on("submit-flag-raise-propagated"):
            raise
        problem = f"放宽标识没判成（{error!r}），按命令文字里提到的算"
        state.unjudged.append(problem)
        return flag_asks_by_mention(command_text, problem)


def submit_flag_reason(asks, command_text):
    """给用户看的理由：带哪个标识、标识的作用、算不出的词、命令全文、对话不许自己用。"""
    flags = [flag for flag in LOOSENING_SUBMIT_FLAGS if any(flag in ask.flags for ask in asks)]
    unread = [word for ask in asks for word in ask.unread_words]
    lines = []
    if flags:
        lines.append("这条命令带放宽检查的标识 " + "、".join(f"{flag}（{LOOSENING_SUBMIT_FLAGS[flag]}）" for flag in flags) + "。")
    if unread:
        lines.append(f"参数里有变量，认不出带没带标识（{'、'.join(unread)}）。")
    lines += [ask.problem for ask in asks if ask.problem]
    return "\n".join(lines + [SUBMIT_FLAG_USER_RULE, f"命令全文：{command_text}"])


def ask_user_about_submit_flags(asks, command_text):
    """有要确认的就往标准输出打 permissionDecision 为 ask 的 JSON（退出码照 0）。弄坏开关 submit-flag-ask-silent 下不打；判的时候抛异常那一支也走这里，
    而认不出的项正是让 guard_switch_on 抛异常的那一种，所以直接读变量。"""
    if not asks or "submit-flag-ask-silent" in os.environ.get(BREAK_VARIABLE_NAME, "").split(","):
        return
    print(json.dumps({"hookSpecificOutput": {"hookEventName": "PreToolUse", "permissionDecision": "ask",
                                             "permissionDecisionReason": submit_flag_reason(asks, command_text)}}, ensure_ascii=False))


def last_runs_path(script_key):
    """上次红的清单：SINGLEFS_LAST_RUNS_DIRECTORY 设了放它底下，没设是数据根下的 last-runs；同目录 lib_work_root.py 读不到、取不到时抛它的异常。"""
    directory = os.environ.get(LAST_RUNS_VARIABLE)
    if not directory:
        if HOOK_DIRECTORY not in sys.path:
            sys.path.insert(0, HOOK_DIRECTORY)
        import lib_work_root
        directory = lib_work_root.shared_data_path(LAST_RUNS_ENTRY)
    return os.path.join(directory, script_key.replace("/", "__") + ".tsv")


def last_red_lines(shape):
    """上次这一道红的清单（给拒绝说明用的几行）。"""
    if shape.script_key is None:
        return ["上次红的：cargo test 没有清单文件，从上次的日志取：python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包>"]
    try:
        path = last_runs_path(shape.script_key)
    except (ImportError, OSError, ValueError) as error:
        return [f"上次红的：取不到上次红的清单在哪（同目录 lib_work_root.py 读不到或取不到数据根：{error!r}）"]
    if guard_switch_on("last-runs-unread"):
        return [f"上次红的：没有上次红的记录（{path}）"]
    try:
        with open(path, encoding="utf-8") as handle:
            names = [line.strip() for line in handle if line.strip() and not line.strip().startswith("#")]
    except (OSError, UnicodeDecodeError):
        return [f"上次红的：没有上次红的记录（{path} 不在或读不了；各道跑完写这份文件归下一批）"]
    if not names:
        return [f"上次红的：没有上次红的记录（{path} 是空的）"]
    return [f"上次红的（{path}，{len(names)} 项）：{'、'.join(names)}"]


def template_line(found, command_text):
    shape = found.shape
    declaration = f"{DECLARATION_VARIABLE}='种类={shape.kind}; 跑什么={shape.what}; 范围={shape.range_hint}; 为什么=<为什么这么跑、这个范围为什么合适>'"
    if found.frames:
        return f"  在 {frames_description(found.frames[-1:])} 那一行的命令前面加（已有的那份换掉）：{declaration}"
    bare = command_text if guard_switch_on("template-keeps-declaration") else LEADING_DECLARATION.sub("", command_text, count=1)
    if DECLARATION_VARIABLE in bare:
        return f"  把命令里已有的那份 {DECLARATION_VARIABLE} 换成：{declaration}"
    if len(bare) > TEMPLATE_COMMAND_LIMIT:
        return f"  这条命令 {len(bare)} 字，不整条贴；在起它的那一条前面加：{declaration}"
    return f"  {declaration} {bare}"


NARROWING_LINES = (
    "→ 先想范围合不合适：红了只跑红的，改了只点名改到的——",
    "  门禁：bash .claude/gate.d/<阶段>.sh --check <格>[,<格>…]（格名用 --list 列）；有逐项的门禁写 --item <项>（--list-items 列）",
    "  崩溃用例：bash .claude/gate.d/crash-point-replay.sh --check crash-case-replay --crash-cases-only --item crash-case:<用例名> <根>（--list-items 列）；"
    "崩溃放量 bash scripts/crash/crash-amplification.sh --item <分项>（--list-items 列）",
    "  cargo test：cargo test -p <包> --test <目标> -- <测试名> --exact；上次红的用 python3 scripts/gate/rerun-failed-tests.py <上次日志> -p <包> 打出只重跑它们的命令",
    "  merge-and-verify：挑测试目标在脚本里做（只跑补丁改到的与变异行点名的测试目标），种类写 items，不带 --item 范围写补丁目录的份数，为什么写明这一批补丁改到哪些目标；"
    "红了只重跑红的写 --skip-apply --item <包>:<目标根文件> <同一批补丁目录>（--list-items 列），范围写项数",
)
DECLARATION_RULE_LINES = (
    f"→ 声明写在命令开头：{DECLARATION_VARIABLE}='种类=…; 跑什么=…; 范围=…; 为什么=…'，四段用 ; 隔、各段 键=值；"
    "种类 whole（不带 --item / --check 的整轮、cargo test 整包或整工作区、整轮门禁）、items（带 --item / --check、merge-and-verify（带 --item 范围写项数，不带写补丁目录的份数）、replay.sh 带实验号）、"
    "target（cargo test 带 --test / --bin / --lib 目标、崩溃放量的分项与双机驱动）；"
    "范围：items 写「N 格 / N 项」、target 写「N 个测试目标」（带 --exact 筛选词的写「N 个测试」），N 是阿拉伯数字、等于点名的个数，whole 写「整轮」或「整包」；"
    f"为什么至少 {REASON_MINIMUM_CHARACTERS} 个字，只写「要跑」「验证」不算；整轮门禁的为什么写「提交前整轮」就放。驱动脚本里每一条验证命令那一行各自带声明。",
)


def frames_description(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)


def sharded_wrapped_lines(number, candidate, entry):
    """交给服务的那一条里套着分片入口的一条（WrappedCandidate、sharded_wrapped_entry 交的入口）：点名那份入口与它在驱动脚本里的 脚本:行（在 bash -c 或包装里的写这一点），
    出路写把那一行直接交给服务、给出拼好的那一条。"""
    in_script = len(candidate.frames) > len(candidate.submission_frames)
    where = f"脚本 {frames_description(candidate.frames[len(candidate.submission_frames):])} 里的 " if in_script else "交给服务的那一条里（bash -c 或包装里）的 "
    words = shlex.join(candidate_command_words(candidate))
    return [f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}",
            f"     ✗ 分片入口 {entry} 套在里面：交给服务的那一条是 {shlex.join(candidate.submitted_words)[:200]}，服务只认命令位置上的词是分片入口，这一条认不出，整件不分片，"
            "门禁只走合并核账，键对不上的项一条都不现判、一项没判就退 1",
            "     → 怎么办：把那一行直接交给服务，--item 照字面写进命令：",
            f"       {direct_submission_command(candidate)}"]


def refusal_text(refused, command_text, crash_refused=(), unnamed_refused=(), wrapped_refused=()):
    """被拒的每一条逐条写：命令、跑的是什么、哪里不合；声明写得不对的（说明不以 COMMIT_PROBLEM_PREFIX 起头的）另附上次红的清单、声明模板与只跑红的写法，
    只是提交时才许跑的、崩溃域没缩的、没点名的、套着分片入口的不附（说明里已写出路）。crash_refused 是崩溃域闸拒的：[(CrashSubject, 说明)]；
    unnamed_refused 是经 submit 交而没点名的：[(SubmittedRun, {problem, next_step})]，逐条写那一句与改成点名的样子；
    wrapped_refused 是经 submit 交而分片入口套在驱动脚本或 bash -c 里的：[(WrappedCandidate, 入口)]，逐条写 sharded_wrapped_lines。"""
    lines = [f"✗ 范围声明不合，这条命令没跑（{GUARD_SCRIPT_NAME}）："]
    declaration_refused = []
    for number, (found, problems) in enumerate(refused, start=1):
        where = f"脚本 {frames_description(found.frames)} 里的 " if found.frames else ""
        words = shlex.join(found.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     跑的是：{found.shape.what}；范围：{found.shape.range_hint}")
        lines += [f"     ✗ {problem}" for problem in problems]
        if any(not problem.startswith(COMMIT_PROBLEM_PREFIX) for problem in problems):
            declaration_refused.append(found)
            lines += [f"     {line}" for line in last_red_lines(found.shape)]
    for number, (subject, problem) in enumerate(crash_refused, start=len(refused) + 1):
        where = f"脚本 {frames_description(subject.frames)} 里的 " if subject.frames else ""
        words = shlex.join(subject.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     ✗ {problem}")
    for number, (run, found) in enumerate(unnamed_refused, start=len(refused) + len(crash_refused) + 1):
        where = f"脚本 {frames_description(run.frames)} 里的 " if run.frames else ""
        words = shlex.join(run.words)
        lines.append(f"  {number}. {where}{words[:240]}{'…' if len(words) > 240 else ''}")
        lines.append(f"     ✗ {found['problem']}")
        lines.append(f"     → 怎么办：{found['next_step']}")
    for number, (candidate, entry) in enumerate(wrapped_refused, start=len(refused) + len(crash_refused) + len(unnamed_refused) + 1):
        lines += sharded_wrapped_lines(number, candidate, entry)
    if declaration_refused:
        lines.append("→ 照抄声明模板，把为什么填上（写为什么这么跑、这个范围为什么合适）：")
        lines += [template_line(found, command_text) for found in declaration_refused]
        lines += list(NARROWING_LINES) + list(DECLARATION_RULE_LINES)
    return "\n".join(lines)


class ScopeDecision(NamedTuple):
    code: int            # 0 放行，2 拒绝
    message: str | None  # 拒绝时写给 stderr 的说明
    allowed: list        # 放行的每一条（FoundRun）
    refused: list        # 被拒的每一条：(FoundRun, [说明])
    unjudged: list = []  # 判不了的、喂给 shell 的输入（每处一句）：照常判别的，记进检出
    flag_asks: list = []  # 要用户确认的提交（SubmitFlagAsk）；拒绝时是空的，拒了就不弹窗
    crash_refused: list = []  # 崩溃域闸拒的每一条：(CrashSubject, 说明)
    unnamed_refused: list = []  # 经 submit 交而没点名的每一条：(SubmittedRun, {problem, next_step})
    wrapped_refused: list = []  # 经 submit 交而分片入口套在驱动脚本或 bash -c 里的每一条：(WrappedCandidate, 入口)


def decide_scope(hook_input, project_root):
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = ScanState()
    runs = scope_runs_in_text(command_text, directory, state=state)
    refused, allowed = [], []
    for found in runs:
        problems = scope_problems(found.shape, found.declaration, agent_type, found.client_refusal, found.submit, found.words)
        (refused if problems else allowed).append((found, problems) if problems else found)
    crash_refused = [(subject, problem) for subject, problem in ((subject, crash_domain_problem(subject, project_root, state)) for subject in state.crash_subjects) if problem]
    # 经 submit 交的那一条点没点名（子 agent 交的另被拒，不判）
    unnamed_refused = [] if agent_type else [(run, found) for run, found in ((run, unnamed_submission_problem(run, state)) for run in state.submitted_runs) if found]
    # 经 submit 交的那一条自己在命令位置上不是分片入口，而它的 bash -c、包装或读进去的驱动脚本里有（子 agent 交的另被拒，不判）
    wrapped_refused = [] if agent_type else [(candidate, entry) for candidate, entry in ((candidate, sharded_wrapped_entry(candidate, state)) for candidate in state.wrapped_candidates) if entry]
    flag_asks = submit_flag_asks(command_text, directory, agent_type, state)
    if refused or crash_refused or unnamed_refused or wrapped_refused:
        return ScopeDecision(2, refusal_text(refused, command_text, crash_refused, unnamed_refused, wrapped_refused), allowed, refused, state.unjudged, [], crash_refused, unnamed_refused,
                             wrapped_refused)
    return ScopeDecision(0, None, allowed, [], state.unjudged, flag_asks)


def detections_path_now():
    configured = os.environ.get(DETECTIONS_VARIABLE)
    if configured:
        return configured
    try:
        sys.path.insert(0, HOOK_DIRECTORY)
        import lib_work_root
        return lib_work_root.default_detections_path()
    except Exception:
        return None


def append_detection(path, entry):
    if not path:
        return
    try:
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass


def record_scope_decision(hook_input, decision, path):
    """放行的每一条记一行（会话号写在 scope_session_id，看门狗不当检出叫醒）；被拒的整条命令记一行（session_id、outcome 被拒，看门狗只记不叫醒）。"""
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = (tool_input.get("command") or "")[:2000]
    common = {"time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME, "agent_id": hook_input.get("agent_id"),
              "agent_type": hook_input.get("agent_type") or "主 agent", "transcript_path": hook_input.get("transcript_path"), "command": command_text}
    if not guard_switch_on("allowed-unrecorded"):
        for found in decision.allowed:
            declaration = parse_scope_declaration(found.declaration or "")
            entry = dict(common, scope_session_id=hook_input.get("session_id"), outcome="放行", run_scope={
                "where": frames_description(found.frames) if found.frames else "命令文本", "words": shlex.join(found.words)[:500],
                "kind_of_command": found.shape.kind, "what": found.shape.what, "declaration": found.declaration, "segments": declaration.segments})
            append_detection(path, entry)
    findings = [f"范围声明被拒：{found.shape.what}：{'；'.join(problems)}" for found, problems in decision.refused]
    findings += [f"范围声明被拒：{CRASH_SUBJECT_DESCRIPTIONS[subject.kind]}：{problem}" for subject, problem in decision.crash_refused]
    findings += [f"范围声明被拒：{found['problem']}" for _run, found in decision.unnamed_refused]
    findings += [f"范围声明被拒：分片入口 {entry} 套在驱动脚本或 bash -c 里，交给服务的那一条（{shlex.join(candidate.submitted_words)[:200]}）认不出分片" for candidate, entry in decision.wrapped_refused]
    if decision.unjudged and not failure_recording_dropped():   # 判不了的、喂给 shell 的输入与崩溃域闸判不出的目标或项：跟着这一条命令记（放行的写「只记」）
        findings += [f"范围声明闸判不了：{entry}" for entry in decision.unjudged]
    if findings:
        append_detection(path, dict(common, session_id=hook_input.get("session_id"), findings=findings,
                                    outcome="被拒" if decision.refused or decision.crash_refused or decision.unnamed_refused or decision.wrapped_refused else "只记"))


HOOK_INPUT_HEAD_BYTES = 200   # hook 输入不是 JSON 对象时，诊断里贴它开头这么多字节


def failure_recording_dropped():
    """弄坏开关 failure-unrecorded（判不了的只打 stderr、不记检出）设没设。不核别的项认不认得：判的时候抛异常那一支也读它，
    而认不出的项正是让 guard_switch_on 抛异常的那一种（自证拿它当注入）。"""
    return "failure-unrecorded" in os.environ.get(BREAK_VARIABLE_NAME, "").split(",")


def note_unjudged(warning, hook_input):
    """判不了、照常执行的一条：stderr 打一行，检出记录记一行（结局「只记」；hook 输入读不出来时会话号与 agent 写「读不到」）。"""
    print(f"  ! {warning}", file=sys.stderr)
    if failure_recording_dropped():
        return
    tool_input = hook_input.get("tool_input") if isinstance(hook_input, dict) and isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    known = hook_input if isinstance(hook_input, dict) else {}
    append_detection(detections_path_now(), {
        "time": datetime.now(timezone.utc).isoformat(), "hook": GUARD_SCRIPT_NAME,
        "session_id": known.get("session_id", "读不到"), "agent_id": known.get("agent_id", "读不到"),
        "agent_type": (known.get("agent_type") or "主 agent") if known else "读不到", "transcript_path": known.get("transcript_path", "读不到"),
        "command": (command if isinstance(command, str) else repr(command))[:2000], "findings": [warning], "outcome": "只记"})


def hook_main(hook_directory):
    raw = sys.stdin.buffer.read()
    try:
        hook_input = json.loads(raw)
        unreadable = None if isinstance(hook_input, dict) else f"解出来是 {type(hook_input).__name__}，不是对象"
    except Exception as error:   # 不是合法 JSON、不是 UTF-8
        hook_input, unreadable = None, f"解析出错：{error!r}"
    if unreadable:   # 照常执行，打一行可定位的诊断（钩子名、开头的字节）、记一条检出
        note_unjudged(f"{GUARD_SCRIPT_NAME} 收到的 hook 输入不是 JSON 对象（{unreadable}），这条命令没判、照常执行；"
                      f"开头 {len(raw[:HOOK_INPUT_HEAD_BYTES])} 字节（共 {len(raw)} 字节）：{raw[:HOOK_INPUT_HEAD_BYTES]!r}", None)
        return 0
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command_text = tool_input.get("command") or ""
    # 粗筛：这几样一个都没有的命令起不了验证脚本（cargo、shell 与 .sh、路径、source / .），也带不了放宽标识
    if not any(marker in command_text for marker in ("cargo", "sh", "/", "source", ". ", *LOOSENING_SUBMIT_FLAGS)):
        return 0
    try:
        load_judging_libraries()
        project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_directory))
        decision = decide_scope(hook_input, project_root)
        record_scope_decision(hook_input, decision, detections_path_now())
    except Exception as error:   # 判的时候抛了：照常执行，stderr 报一句、记一条检出
        note_unjudged(f"{GUARD_SCRIPT_NAME} 没判成（{error!r}），这条命令照常执行", hook_input)
        if not hook_input.get("agent_type"):   # 放宽标识同样没判：命令文字里提到的照弹窗
            ask_user_about_submit_flags(flag_asks_by_mention(command_text, f"{GUARD_SCRIPT_NAME} 没判成（{error!r}），按命令文字里提到的算。"), command_text)
        return 0
    if decision.code:
        print(decision.message, file=sys.stderr)
    ask_user_about_submit_flags(decision.flag_asks, command_text)
    return decision.code


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认（与入口脚本开头那一道同一个判法，免得直接起这份库绕过它）：参数不是单独一个 --selftest、自证标记文件对不上而 RUN_SCOPE_GUARD_BREAK 非空，
    交回拒绝的几行（点名变量、说它只在自证里认）；关守卫项 honor-outside-selftest 单独一项时放行，与别的项同写照拒（带标记又同写着 honor-outside-selftest-with-others 的除外）；放行交回空表。"""
    value = os.environ.get(BREAK_VARIABLE_NAME, "")
    if not value or arguments == ["--selftest"]:
        return []
    marked = selftest_marked()
    items = [item for item in value.split(",") if item]
    guard_off = "honor-outside-selftest" in items
    others = [item for item in items if item != "honor-outside-selftest"]
    lenient = "honor-outside-selftest-with-others" in others
    if guard_off and others and not (marked and lenient):
        return [f"  ✗ {BREAK_VARIABLE_NAME}={value}：关守卫项 honor-outside-selftest 只在它是唯一一项时认，与别的项（{'、'.join(others)}）同写照拒；弄坏开关只在自证里认",
                f"  → 怎么办：去掉 {BREAK_VARIABLE_NAME} 再跑；要证红就跑 {BREAK_VARIABLE_NAME}=<项> bash .claude/hooks/run-scope-guard.sh --selftest"]
    if not marked and not (guard_off and not others):
        return [f"  ✗ {BREAK_VARIABLE_NAME}={value} 是弄坏开关，只在自证（run-scope-guard.sh --selftest 与它起的子进程）里认；平常跑设了就拒，这一趟什么都没做",
                f"  → 怎么办：去掉 {BREAK_VARIABLE_NAME} 再跑；要证红就跑 {BREAK_VARIABLE_NAME}=<项> bash .claude/hooks/run-scope-guard.sh --selftest"]
    return []
