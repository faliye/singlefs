#!/usr/bin/env python3
# admission: always 判的是此刻那棵树里要拷哪些仓根条目、拷出来的副本编不编得过，上一次的结论不替这一次作保
# run-condition: command python3 rsync git
"""拷仓副本的范围：一份共用清单加它的闭包算法，全仓建仓副本的脚本都调这里，不各抄一份清单。

共用清单 COMMON_ENTRIES：Cargo.toml、Cargo.lock、.cargo、crates（与仓根有的工具链文件 rust-toolchain、rust-toolchain.toml）。
闭包（copy_closure）在清单上逐层现算，交 CopyClosure（entries 是要拷的仓根条目，reasons 是每一条为什么在，started 是 Rust 代码直接起的脚本）：
  · path 依赖：清单与调用方声明的各处里每份 Cargo.toml（三种依赖表、两种写法，target.<条件> 下的同名表，[workspace.dependencies]、[patch.*]，
    仓根那一份的 [workspace] members 去掉 exclude）经 path 指到的目录，连同那些目录自己的 Cargo.toml 再指到的；指到仓根之外的抛 ClosureRefused。
  · 编译期读的仓根文件：各包（上一条算出的、清单与声明里带 [package] 的目录）的 .rs 经 scripts/gate/lib_function_units.py 的 rust_lex 切词，
    include_str!、include_bytes!、include! 的参数是一个串字面量的按那份源文件所在目录解，是 concat!(env!("CARGO_MANIFEST_DIR"), …) 的按包目录解，
    concat!(env!("OUT_DIR"), …) 是编译时生成的、不算；#[path = "…"] 按源文件所在目录解。认不出的写法、绝对路径、落到仓根之外的、指到不在的文件，
    抛 ClosureRefused（逐处写 <文件>:<行>）：副本里读不到就编不过。
  · 运行期读的仓根文件：同一批 .rs 里以 ../、/../、{名}/../ 起头的串字面量（Path::new(env!("CARGO_MANIFEST_DIR")).join("../../…")、
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../…")、format!("{manifest_directory}/../../…") 这几种），按包目录解；
    .join("..") 起头、后面接着 .join("<路径>") 的一串（中间的 .parent() 退一层）同样按包目录解。指到的条目在仓里不在的记进 missing、不拷。
  · 被起、被 source 的传递依赖：上面两条指到的 .sh、.py 与带 #! 的文件逐个读，shell 认「<目录表达式>/<相对路径>.sh|.py」的写法
    （"${BASH_SOURCE[0]%/*}/…"、"$(dirname "${BASH_SOURCE[0]}")/…"、"$某某_directory/…" 这类），python 认同目录与 sys.path 加进来的目录里的 import、
    os.path.join(…, "<段>", …, "<名>.py") 与引号里的 "<名>.py"，都按那份脚本所在目录解，传递地收；注释行不算，指到不在的不收。
    python 里赋给一个名字的目录也认：<名> = os.path.dirname(os.path.realpath(__file__))（或 abspath）记成脚本所在目录，<名> = os.path.join(<已记的名>, "<段>", …)
    记成拼出来的目录；之后 os.path.join(<名>, "<段>", …) 按那个目录解，sys.path.insert(0, <名>) / sys.path.append(<名>) 把它加进 import 的搜索目录。
    同一个名字赋几次的，几个目录都记（不分作用域，指到不在的不收）。.claude/scripts/project_preflight.py 经这一条接上规范副本里的 preflight.py。
    起点是单独成条的脚本，加上面两条指到的每一份脚本（罩在别的目录条目里、不单独成条的照算）：收不收得进与 Rust 里引用的先后无关。
  · 测试代码：默认不扫（只给副本里编、起非测试代码的调用方）：各包 tests/、benches/、examples/ 下的文件，带 #[cfg(test)]（cfg 里只有 test、不带 not 与 any）
    或 #[test] 的项，文件头 #![cfg(test)] 的整份。test_inputs=True 时一起扫（副本里要跑测试的调用方：crates 变异、证红）。
  · 决不拷的 NEVER_COPIED：仓根的多机配置 multi-host.env（本机私有、被 git 忽略）。指到它的引用记进 left_out，不进副本；
    副本里起命令的一方用 environment_for_commands_in_the_copy 把主工作区那一份的绝对路径显式交给 SINGLEFS_MULTI_HOST_CONFIG。
调用方要多拷的，经 declared 参数逐条声明 (条目, 为什么)，为什么至少 4 个字；声明的目录同样跟 path 依赖、同样扫。
  · 声明的条目里的脚本也当追 source 链的起点（副本带 --with-gate-tables 而缺 .claude/scripts/preflight.sh（records/2026-10-31-task-server巡检.md），
    门禁 source 失败、参数被清空成全跑）：声明的单个脚本，与声明的目录里的每一份脚本（往下逐层，不进 SKIPPED_DIRECTORY_NAMES、target、target-* 与 DECLARED_SKIPPED_DIRECTORY_NAMES
    那几种目录：fixtures 下是门禁样本，副本里不起），连同它们 source、起、import 的仓内文件传递地收，为什么写 DECLARED_SCRIPTS_REASON 起头的那一句。
    这一支算不全的拒：从声明的条目里的脚本起，shell 里行首（可带 exec）是 source、.、bash、sh、python3、紧接着照脚本自己所在目录写的路径
    （"$(dirname "${BASH_SOURCE[0]}")/…"、"${BASH_SOURCE[0]%/*}/…"；ANCHORED_SHELL_REFERENCE，printf 与引号里的样本行不算）的文件在源树里不在的，抛 ClosureRefused、逐处写 <文件>:<行> 与缺的那一份（不拷半份）；python 的 import 与 $名/… 这类照目录变量拼的照旧指到不在的不收、不判。
    弄坏开关 declared-directories-not-followed 下声明的目录里的脚本不起头，declared-closure-gaps-accepted 下缺的照旧不收、不拒。
管不到的（靠写代码的人与真编核对）：路径拼在变量里、从环境变量取、format! 中间夹变量的；build.rs 经 .parent() 取仓根再读的（读的是
Cargo.toml、Cargo.lock 这类，已在清单里）；python 里不是上面那两种赋法记下的名字拼出的路径（函数返回值、参数、git 取的目录）、importlib 按变量路径载入的模块。
写法认不出的，改调用方的写法让它认得出（先把目录赋给一个名字，再 os.path.join(<名>, …)），不另加手抄清单；副本里要起、要读的脚本与数据经 declared 声明。

拷法 copy_entries：rsync -a --no-times --checksum --relative（单个文件的上级目录照建；拷过去的修改时刻是此刻），
不进名叫 .git、target、target-*、__pycache__ 的目录。被 .gitignore 挡着的规范副本 .claude/singlefs-ai-sop 下的条目照文件系统拷。

副本里起命令的环境 environment_for_commands_in_the_copy(<源树>, <环境>)：调用方的环境里 SINGLEFS_MULTI_HOST_CONFIG 设了照它，没设填
<源树所在的主工作区>/multi-host.env 的绝对路径（主工作区照 git rev-parse --git-common-dir 取，git worktree 里取主仓；不在 git 里取源树自己）。

用法：
  lib_repository_copy.py entries <仓根> [--test-inputs] [--with <条目>=<为什么>]…
      打算出的闭包：一行 REPOSITORY_COPY entry <条目>\t<为什么>，指到决不拷的打 REPOSITORY_COPY left_out，在仓里不在的打 REPOSITORY_COPY missing，
      末行 REPOSITORY_COPY entries=<条数> started=<Rust 直接起的脚本，逗号连>；闭包算不成打 ✗ 与出路、退 1
  lib_repository_copy.py link-into <副本根> --root <仓根> --workspace <工作区目录> [--present <条目>]… [--test-inputs]
      只拷了一个工作区（scripts/experiment-apparatus 这类）的副本：闭包里副本根下还没有的仓根条目逐个符号链接回仓里那一份（只读）；
      --present 点名副本里已经放好的（链过去的 research/results 这类）。闭包算不成退 2；末行一句「副本根下链了 N 份 <工作区>/ 之外的仓内文件」
  lib_repository_copy.py missing-in-copy <副本根> [--root <仓根>] [--test-inputs] [--with <条目>=<为什么>]…
      核一份仓副本缺不缺（entries_missing_in_copy）：闭包照 <仓根>（不给是这份脚本所在的仓）与同一组参数算，逐条看副本根下在不在；只看条目本身在不在，
      目录条目在就算在，不比里面的文件与内容（副本里的改动是调用方自己的）。缺的每条打 REPOSITORY_COPY missing_in_copy <条目>\t<为什么>，
      再打 ✗ 与出路（照 minimal-copy.py 另建副本，copy_rebuild_howto）、退 1；不缺的末行 REPOSITORY_COPY copy-complete entries=<闭包条数>、退 0；闭包算不成退 1。
      吃副本的脚本（scripts/mutation/prove-red.sh --copy）开跑前调同一个函数，不另抄
  lib_repository_copy.py configuration-path <源树>
      打副本里起命令要用的多机配置的绝对路径（bash 调用方 export 给 SINGLEFS_MULTI_HOST_CONFIG 用；环境里已设的照打它）
  lib_repository_copy.py verify-copy [--root <仓根>] [--work <目录>] [--test-inputs] [--with <条目>=<为什么>]… -- <cargo build 的参数…>
      真编核对（编译，经 task-server/task submit 交）：把闭包拷进 <工作目录>/tree（不是 git 工作树），在那里带调用方的参数起 cargo build，
      再在副本里逐个起 RUNTIME_PROBES 登记的运行期脚本（用副本环境，判退 0；判法 --emit-addresses 只打键名不打值），
      副本不在 git 工作树里时另去掉 SINGLEFS_MULTI_HOST_CONFIG 再起一次多机配置判法、判它拒（退非 0：照仓位置找不到配置）。
      工作目录：给了 --work 用它，没给用服务给的 $CARGO_TARGET_DIR/repository-copy-verify；编译目录是 $CARGO_TARGET_DIR，没给用 <工作目录>/target。
      收尾删副本（编译目录归服务）。末行 REPOSITORY_COPY_VERIFY passed checked=<格数> 或 failed …；退出码 0 过、1 不过、2 用法错
  lib_repository_copy.py --selftest
退出码：0 成；1 闭包算不成或核对不过；2 用法错。

弄坏开关 REPOSITORY_COPY_BREAK=<项>（只给 --selftest 证红用，逗号连几个；命令行在 --selftest 之外设了就拒、退 2，别的脚本 import 这一份时设了也不认）。项名、弄坏哪一处，下一行写打开后红在哪几格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  REPOSITORY_COPY_BREAK=path-dependencies-ignored  不跟 path 依赖
      「✗ 自证 path-dependencies-followed：」「✗ 自证 path-dependency-outside-root-refused：」「✗ 自证 join-chains-resolved：」
      「✗ 自证 declared-entries-need-a-reason：」「✗ 自证 repository-root-references-in-the-closure：」
  REPOSITORY_COPY_BREAK=outside-root-accepted  path 依赖指到仓根之外也不拒
      「✗ 自证 path-dependency-outside-root-refused：」
  REPOSITORY_COPY_BREAK=includes-ignored  不认 include_str! / include_bytes! / include!
      「✗ 自证 include-literals-resolved-by-file：」「✗ 自证 unrecognized-include-refused：」「✗ 自证 copies-the-closure-with-parent-directories：」
      「✗ 自证 cross-check-sees-file-relative-includes：」
  REPOSITORY_COPY_BREAK=runtime-literals-ignored  不认 ../ 起头的串字面量
      「✗ 自证 runtime-literals-resolved-by-crate：」「✗ 自证 test-code-left-out：」「✗ 自证 shell-sources-followed：」「✗ 自证 python-references-followed：」
      「✗ 自证 configuration-never-copied：」「✗ 自证 repository-root-references-in-the-closure：」「✗ 自证 verify-copy-builds-and-starts-the-probes-in-the-copy：」
      「✗ 自证 covered-scripts-followed：」
  REPOSITORY_COPY_BREAK=join-chains-ignored  不认 .join("..") 起头的一串
      「✗ 自证 join-chains-resolved：」
  REPOSITORY_COPY_BREAK=test-code-scanned  不带 test_inputs 也扫测试代码
      「✗ 自证 test-code-left-out：」「✗ 自证 runtime-probes-cover-every-started-script：」
  REPOSITORY_COPY_BREAK=test-inputs-ignored  带了 test_inputs 也不扫测试代码
      「✗ 自证 test-code-left-out：」
  REPOSITORY_COPY_BREAK=shell-sources-not-followed  不跟 shell 脚本 source、起的文件
      「✗ 自证 shell-sources-followed：」「✗ 自证 repository-root-references-in-the-closure：」「✗ 自证 covered-scripts-followed：」
  REPOSITORY_COPY_BREAK=python-references-not-followed  不跟 python 脚本 import、点名的文件
      「✗ 自证 python-references-followed：」「✗ 自证 python-directory-variables-followed：」
  REPOSITORY_COPY_BREAK=python-directory-variables-ignored  python 里赋给名字的目录不记，os.path.join(<名>, …) 按脚本所在目录解
      「✗ 自证 python-directory-variables-followed：」
  REPOSITORY_COPY_BREAK=never-copied-copied  多机配置也拷进副本
      「✗ 自证 configuration-never-copied：」
  REPOSITORY_COPY_BREAK=declared-reason-unchecked  声明的条目不写为什么也收
      「✗ 自证 declared-entries-need-a-reason：」
  REPOSITORY_COPY_BREAK=environment-configuration-unset  副本环境不设 SINGLEFS_MULTI_HOST_CONFIG
      「✗ 自证 copy-environment-names-the-main-configuration：」「✗ 自证 verify-copy-builds-and-starts-the-probes-in-the-copy：」
  REPOSITORY_COPY_BREAK=environment-configuration-by-tree  副本环境照源树自己的位置取配置，不认 git worktree 的主仓
      「✗ 自证 copy-environment-names-the-main-configuration：」
  REPOSITORY_COPY_BREAK=copy-without-parent-directories  拷单个文件的条目不建上级目录（rsync 不带 --relative，逐条拷到目的路径）
      「✗ 自证 copies-the-closure-with-parent-directories：」「✗ 自证 verify-copy-builds-and-starts-the-probes-in-the-copy：」
  REPOSITORY_COPY_BREAK=probe-dropped  RUNTIME_PROBES 里去掉多机配置判法那一条
      「✗ 自证 runtime-probes-cover-every-started-script：」
  REPOSITORY_COPY_BREAK=verify-probes-skipped  真编核对编完不起运行期脚本
      「✗ 自证 verify-copy-builds-and-starts-the-probes-in-the-copy：」
  REPOSITORY_COPY_BREAK=copy-completeness-unchecked  entries_missing_in_copy 一条都不报（副本缺闭包里的条目也当拷全了）
      「✗ 自证 copy-completeness-names-missing-entries：」
  REPOSITORY_COPY_BREAK=covered-scripts-not-followed  罩在目录条目里、不单独成条的脚本不追 source 链（只拿单独成条的脚本起头）
      「✗ 自证 covered-scripts-followed：」「✗ 自证 closure-independent-of-reference-order：」
  REPOSITORY_COPY_BREAK=declared-directories-not-followed  声明的目录里的脚本不当追 source 链的起点（副本缺 .claude/scripts/preflight.sh）
      「✗ 自证 declared-directory-scripts-followed：」「✗ 自证 declared-closure-gap-refused：」
  REPOSITORY_COPY_BREAK=declared-closure-gaps-accepted  声明的条目里的脚本照脚本所在目录 source、起的文件在源树里不在时照旧不收、不拒
      「✗ 自证 declared-closure-gap-refused：」
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import glob  # noqa: E402
import importlib.util  # noqa: E402
import io  # noqa: E402
import os  # noqa: E402
import re  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import time  # noqa: E402
import tomllib  # noqa: E402
from typing import NamedTuple  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
FUNCTION_UNITS_LIBRARY = os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_function_units.py")
BREAK_VARIABLE = "REPOSITORY_COPY_BREAK"
CONFIGURATION_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
CONFIGURATION_FILE_NAME = "multi-host.env"
# 共用清单：每一份仓副本都拷的仓根条目（不在的跳过：工具链文件仓里没有时不拷）
COMMON_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates", "rust-toolchain", "rust-toolchain.toml")
# 决不拷进副本的仓根条目与为什么
NEVER_COPIED = {CONFIGURATION_FILE_NAME: "多机配置是本机私有、被 git 忽略的那一份：副本里起命令的一方经 environment_for_commands_in_the_copy 把主工作区那一份的绝对路径交给 "
                                         + CONFIGURATION_VARIABLE + "，不照编它的那棵树的位置找"}
IGNORED_BY_GIT_PREFIX = ".claude/singlefs-ai-sop"   # 被 .gitignore 挡着的规范副本：按 git 列文件的拷法带不上，照文件系统拷
SKIPPED_DIRECTORY_NAMES = (".git", "__pycache__")
TEST_CODE_DIRECTORIES = ("tests", "benches", "examples")
DEPENDENCY_TABLE_NAMES = ("dependencies", "dev-dependencies", "build-dependencies", "dev_dependencies", "build_dependencies")
INCLUDE_MACROS = ("include_str", "include_bytes", "include")
CLOSING_OF = {"(": ")", "[": "]", "{": "}"}
MINIMUM_REASON_CHARACTERS = 4
SCRIPT_SUFFIXES = (".sh", ".py")
DECLARED_SKIPPED_DIRECTORY_NAMES = ("fixtures",)   # 声明的目录里不当起点的子目录（文件头「闭包」声明的条目那一条）：门禁样本档，副本里不起
DECLARED_SCRIPTS_REASON = "调用方声明的条目里的脚本 source、起"   # 那一支收进来的条目的为什么起头（minimal-copy.py 的弄坏开关照它认出这一支）
# 声明的条目那一支判「算不全」只认这一种行：行首（空白之后，可带 exec）是 source、.、bash、sh、python3，紧接着照脚本自己所在目录写的路径；
# 写在 printf、引号与 heredoc 里的样本行（行首是别的词）不认
ANCHORED_SHELL_REFERENCE = re.compile(r'^\s*(?:exec\s+)?(?:source|\.|bash|sh|python3)\s+"?(?:\$\(dirname "\$\{BASH_SOURCE\[0\]\}"\)|\$\{BASH_SOURCE\[0\]%/\*\})/((?:\.\./)*[\w.\-]+(?:/[\w.\-]+)*\.(?:sh|py))\b')
# 源码里粗筛：带这几种记号的 .rs 才切词（切词只为认出注释、串字面量与测试代码的边界）
CANDIDATE_SOURCE = re.compile(r'include(?:_str|_bytes)?!|#\[path|"(?:/|\{[^}"\n]*\}/)?\.\.(?:/|")')
FORMAT_DIRECTORY_PREFIX = re.compile(r'^\{[A-Za-z_][A-Za-z0-9_]*\}/')
SHELL_REFERENCE = re.compile(r'(?:\}|\)|_directory|_DIRECTORY|\$[A-Za-z_][A-Za-z0-9_]*)/((?:\.\./)*[\w.\-]+(?:/[\w.\-]+)*\.(?:sh|py))\b')
PYTHON_IMPORT = re.compile(r'^\s*(?:from\s+([A-Za-z_][\w.]*)\s+import\b|import\s+([A-Za-z_][\w.]*(?:\s*,\s*[A-Za-z_][\w.]*)*))')
PYTHON_QUOTED_SCRIPT = re.compile(r'''['"]((?:\.\./)*[\w.\-]+(?:/[\w.\-]+)*\.(?:py|sh))['"]''')
PYTHON_JOIN_CALL = re.compile(r'path\.join\(([^()]*(?:\([^()]*(?:\([^()]*\))?[^()]*\)[^()]*)*)\)')
PYTHON_QUOTED_SEGMENT = re.compile(r'''['"]([\w.\-/]+)['"]''')
# python 里赋给一个名字的目录（文件头「闭包」被起、被 source 的那一条）：<名> = <右边>，右边是脚本所在目录的写法或一个 os.path.join(…)
PYTHON_ASSIGNMENT = re.compile(r'^\s*([A-Za-z_]\w*)\s*=(?!=)\s*(\S.*?)\s*$')
PYTHON_SCRIPT_DIRECTORY = re.compile(r'^(?:\w+\.)*path\.dirname\(\s*(?:\w+\.)*path\.(?:realpath|abspath)\(\s*__file__\s*\)\s*\)$')
PYTHON_SYS_PATH_NAME = re.compile(r'sys\.path\.(?:insert\(\s*\d+\s*,|append\()\s*([A-Za-z_]\w*)\s*\)')
PYTHON_NAME = re.compile(r'[A-Za-z_]\w*')
# 真编核对在副本里起的运行期脚本（Rust 非测试代码直接起的每一份都要在这里有一条，格 runtime-probes-cover-every-started-script 判）：
# {tree} 换成副本根。多机配置判法只打键名（值是私有的，核对只看它在副本里判不判得过）
RUNTIME_PROBES = {
    "scripts/crash/crash-point-replay-shard-configuration-check.sh":
        ("bash", "{tree}/scripts/crash/crash-point-replay-shard-configuration-check.sh", "--emit-addresses"),
    ".claude/singlefs-ai-sop/scripts/preflight.py":
        ("python3", "{tree}/.claude/singlefs-ai-sop/scripts/preflight.py", "check", "{tree}/scripts/crash/crash-point-replay-shard-configuration-check.sh", "--"),
}
CONFIGURATION_CHECK = "scripts/crash/crash-point-replay-shard-configuration-check.sh"
PROBE_TIMEOUT_SECONDS = 300
VERIFY_WORK_DIRECTORY_NAME = "repository-copy-verify"
EXIT_REFUSED, EXIT_USAGE = 1, 2


class ClosureRefused(ValueError):
    """闭包算不成：path 依赖指到仓根之外、Cargo.toml 读不懂、编译期读的文件认不出或不在。str 是逐处的问题，以「；」连。"""


class CopyClosure(NamedTuple):
    entries: tuple          # 要拷的仓根条目（相对仓根，排好序；目录或单个文件）
    reasons: dict           # {条目: (为什么, …)}
    started: tuple          # Rust 非测试代码（test_inputs 时连测试代码）直接起的脚本（相对仓根）
    left_out: dict          # {指到的决不拷条目: (出处, …)}
    missing: dict           # {运行期指到、仓里不在的路径: (出处, …)}


SELFTEST_RUNNING = []   # run_cells_in_scratch 跑着时有一项：弄坏开关只在这时认


def requested_breaks():
    return [item for item in os.environ.get(BREAK_VARIABLE, "").split(",") if item]


def active_breaks():
    """此刻生效的弄坏开关：只在自证的格里认（别的脚本 import 这一份时外面设了 REPOSITORY_COPY_BREAK 也不生效；命令行那一层设了就拒，见 main）。"""
    return requested_breaks() if SELFTEST_RUNNING else []


def broken(item):
    return item in active_breaks()


def skipped_directory(name):
    return name in SKIPPED_DIRECTORY_NAMES or name == "target" or name.startswith("target-")


def is_covered(path, entries):
    """path（相对仓根）落在 entries 的某一条里面（等于或在它底下）。"""
    return any(path == entry or path.startswith(entry + "/") for entry in entries)


def relative_inside(root, absolute):
    """absolute 落在仓根里时交相对仓根的路径（仓根本身交 "."），落在外面交 None。"""
    relative = os.path.relpath(os.path.normpath(absolute), root)
    return None if relative == ".." or relative.startswith("../") else relative


# ── path 依赖 ──

def load_manifest(path):
    with open(path, "rb") as handle:
        return tomllib.load(handle)


def manifest_path_dependencies(manifest, manifest_directory):
    """一份 Cargo.toml 写了 path 的依赖指到的目录（绝对路径，不查在不在）：三种依赖表（两种写法）、target.<条件> 下的同名表、
    [workspace.dependencies] 与 [patch.*]。"""
    containers = [manifest, *(table for table in (manifest.get("target") or {}).values() if isinstance(table, dict))]
    tables = [container[name] for container in containers for name in DEPENDENCY_TABLE_NAMES if isinstance(container.get(name), dict)]
    workspace_dependencies = (manifest.get("workspace") or {}).get("dependencies")
    if isinstance(workspace_dependencies, dict):
        tables.append(workspace_dependencies)
    tables += [table for table in (manifest.get("patch") or {}).values() if isinstance(table, dict)]
    return [os.path.normpath(os.path.join(manifest_directory, specification["path"]))
            for table in tables for specification in table.values()
            if isinstance(specification, dict) and isinstance(specification.get("path"), str)]


def workspace_member_directories(manifest, manifest_directory):
    """[workspace] members（通配展开、去掉 exclude）里有 Cargo.toml 的目录（绝对路径）。"""
    workspace = manifest.get("workspace") or {}
    excluded = {os.path.normpath(os.path.join(manifest_directory, entry)) for entry in workspace.get("exclude") or []}
    members = []
    for pattern in workspace.get("members") or []:
        for candidate in sorted(glob.glob(os.path.join(manifest_directory, pattern))):
            candidate = os.path.normpath(candidate)
            if candidate not in excluded and os.path.isfile(os.path.join(candidate, "Cargo.toml")):
                members.append(candidate)
    return members


def manifests_under(path):
    if os.path.isfile(path):
        return [path] if os.path.basename(path) == "Cargo.toml" else []
    found = []
    for directory, subdirectories, names in os.walk(path):
        subdirectories[:] = sorted(name for name in subdirectories if not skipped_directory(name))
        if "Cargo.toml" in names:
            found.append(os.path.join(directory, "Cargo.toml"))
    return found


def path_dependency_entries(root, entries):
    """entries（相对仓根）里各份 Cargo.toml 经 path 指到 entries 之外、仓根之下的目录，逐层跟进，交 {相对仓根的目录: 出处}（出处是指到它的那份 Cargo.toml）。
    仓根那一份的 [workspace] members 算进来。指到仓根之外的、Cargo.toml 读不懂的抛 ClosureRefused。弄坏开关 path-dependencies-ignored：一个都不交。"""
    if broken("path-dependencies-ignored"):
        return {}
    root = os.path.realpath(root)
    covered = list(entries)
    added = {}
    pending = [manifest for entry in entries for manifest in manifests_under(os.path.join(root, entry))]
    visited = set()
    while pending:
        manifest_path = pending.pop()
        if manifest_path in visited:
            continue
        visited.add(manifest_path)
        directory = os.path.dirname(manifest_path)
        shown = os.path.relpath(manifest_path, root)
        try:
            manifest = load_manifest(manifest_path)
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise ClosureRefused(f"{shown} 读不懂（{error}）") from error
        targets = manifest_path_dependencies(manifest, directory)
        if directory == root:
            targets += workspace_member_directories(manifest, directory)
        for target in targets:
            relative = relative_inside(root, os.path.realpath(target))
            if relative is None and not broken("outside-root-accepted"):
                raise ClosureRefused(f"{shown} 的 path 依赖 {target} 在仓根 {root} 之外，副本带不出它的相对位置")
            if relative is None or relative == "." or is_covered(relative, covered):
                continue
            covered.append(relative)
            added[relative] = shown
            pending += manifests_under(target)
    return added


def package_directories(root, entries):
    """要扫 .rs 的目录（相对仓根，排好序）：entries 里每个带 [package] 的 Cargo.toml 所在目录，加顶上就有一份 Cargo.toml 的条目目录本身
    （只拷一个工作区时，工作区根上那份只有 [workspace] 也照扫它底下不属于哪个包的 .rs）。读不懂的不算（path_dependency_entries 那一步已报）。"""
    found = [entry for entry in entries if os.path.isfile(os.path.join(root, entry, "Cargo.toml"))]
    for entry in entries:
        for manifest_path in manifests_under(os.path.join(root, entry)):
            try:
                if "package" in load_manifest(manifest_path):
                    found.append(os.path.relpath(os.path.dirname(manifest_path), root))
            except (OSError, tomllib.TOMLDecodeError):
                continue
    return sorted(set(found))


# ── Rust 源码里指到仓根的路径 ──

_FUNCTION_UNITS = []


def function_units():
    """scripts/gate/lib_function_units.py（rust_lex、rust_string_value），第一次用到时才读进来。"""
    if not _FUNCTION_UNITS:
        specification = importlib.util.spec_from_file_location("lib_function_units_for_repository_copy", FUNCTION_UNITS_LIBRARY)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _FUNCTION_UNITS.append(module)
    return _FUNCTION_UNITS[0]


class RustReference(NamedTuple):
    path: str        # 相对仓根（normpath 过；可能落在仓根之外，以 .. 起头）
    where: str       # <文件>:<行>
    compile_time: bool


def matching_close(texts, opening_index):
    """texts[opening_index] 是 ( [ { 之一：交与它配对的那个右括号的下标（没配上交 len(texts)）。"""
    depth = 0
    for index in range(opening_index, len(texts)):
        if texts[index] in CLOSING_OF:
            depth += 1
        elif texts[index] in (")", "]", "}"):
            depth -= 1
            if depth == 0:
                return index
    return len(texts)


def attribute_is_test_only(texts, start, end):
    """texts[start:end] 是一个属性 [ … ] 里面的词：#[test]，或 cfg(…) 里点名 test、不带 not 与 any。"""
    words = texts[start:end]
    if words == ["test"]:
        return True
    return bool(words) and words[0] == "cfg" and "test" in words and not ({"not", "any"} & set(words))


def test_token_ranges(kinds, texts):
    """测试代码在词法单元里的下标区间 [起, 止)：#[cfg(test)] / #[test] 修饰的项（到它的 ; 或配对的 }），#![cfg(test)] 的整份。"""
    punctuation = function_units().PUNCT
    ranges = []
    index = 0
    count = len(texts)
    while index < count:
        if kinds[index] == punctuation and texts[index] == "#":
            inner = index + 1 < count and texts[index + 1] == "!"
            opening = index + (2 if inner else 1)
            if opening < count and texts[opening] == "[":
                closing = matching_close(texts, opening)
                if attribute_is_test_only(texts, opening + 1, closing):
                    if inner:
                        return [(0, count)]
                    item_end = closing + 1
                    while item_end < count and texts[item_end] == "#" and item_end + 1 < count and texts[item_end + 1] == "[":
                        item_end = matching_close(texts, item_end + 1) + 1
                    scan = item_end
                    while scan < count and texts[scan] not in ("{", ";"):
                        scan = matching_close(texts, scan) + 1 if texts[scan] in ("(", "[") else scan + 1
                    stop = matching_close(texts, scan) + 1 if scan < count and texts[scan] == "{" else scan + 1
                    ranges.append((index, stop))
                    index = stop
                    continue
                index = closing + 1
                continue
        index += 1
    return ranges


def string_value(kinds, texts, index):
    units = function_units()
    return units.rust_string_value(texts[index]) if index < len(texts) and kinds[index] == units.LITERAL else None


def include_argument_path(kinds, texts, opening, closing, file_directory, package_directory):
    """include 宏括号里的参数 → (解出的绝对路径, None)；concat!(env!("OUT_DIR"), …) 交 (None, None)；认不出交 (None, 为什么)。"""
    inside = list(range(opening + 1, closing))
    if inside and texts[inside[-1]] == ",":
        inside = inside[:-1]
    if len(inside) == 1:
        value = string_value(kinds, texts, inside[0])
        if value is None:
            return None, "参数不是一个串字面量"
        if os.path.isabs(value):
            return None, f"(\"{value}\") 是绝对路径"
        return os.path.normpath(os.path.join(file_directory, value)), None
    words = [texts[position] for position in inside]
    if words[:5] == ["concat", "!", "(", "env", "!"] and len(words) > 7 and words[5] == "(" and words[7] == ")":
        variable = string_value(kinds, texts, inside[6])
        if variable == "OUT_DIR":
            return None, None
        pieces = [string_value(kinds, texts, position) for position in inside[8:] if texts[position] not in (",", ")")]
        if variable == "CARGO_MANIFEST_DIR" and pieces and all(piece is not None for piece in pieces):
            return os.path.normpath(package_directory + "/" + "".join(pieces).lstrip("/")), None
    return None, "参数不是一个串字面量，也不是 concat!(env!(\"CARGO_MANIFEST_DIR\"), \"…\")（认不出它读哪份文件）"


def runtime_literal_path(value, package_directory):
    """一个串字面量的值若是指到包目录之外的写法（../、/../、{名}/../ 起头）→ 按包目录解出的绝对路径；不是交 None。"""
    stripped = FORMAT_DIRECTORY_PREFIX.sub("", value, count=1)
    if stripped.startswith("/../"):
        stripped = stripped[1:]
    if not stripped.startswith("../"):
        return None
    return os.path.normpath(os.path.join(package_directory, stripped))


def join_chain_path(kinds, texts, index, package_directory):
    """texts[index] 是 .join("..") 里的 ".."：沿后面的 .join("<路径>") / .parent() 走到这一串的末尾，交按包目录解出的绝对路径。"""
    parts = [".."]
    position = index + 2   # 跳过 ".." 与 )
    while position + 1 < len(texts) and texts[position] == ".":
        if texts[position + 1] == "join" and position + 4 < len(texts) and texts[position + 2] == "(" and texts[position + 4] == ")":
            value = string_value(kinds, texts, position + 3)
            if value is None:
                break
            parts.append(value)
            position += 5
        elif texts[position + 1] == "parent" and position + 3 < len(texts) and texts[position + 2] == "(" and texts[position + 3] == ")":
            parts.append("..")
            position += 4
        else:
            break
    return os.path.normpath(os.path.join(package_directory, *parts))


def rust_file_references(root, path, package_directory, scan_test_code):
    """一份 .rs 里指到它所在包目录之外的路径：[RustReference]，与认不出的 include 写法 [<文件>:<行>：…]。"""
    with open(path, encoding="utf-8", errors="surrogateescape") as handle:
        text = handle.read()
    if not CANDIDATE_SOURCE.search(text):
        return [], []
    units = function_units()
    kinds, texts, lines, _documents = units.rust_lex(text)
    skipped = [] if scan_test_code else test_token_ranges(kinds, texts)
    shown = os.path.relpath(path, root)
    file_directory = os.path.dirname(path)
    references, problems, consumed = [], [], set()
    skip_index = 0
    for index in range(len(texts)):
        while skip_index < len(skipped) and skipped[skip_index][1] <= index:
            skip_index += 1
        if skip_index < len(skipped) and skipped[skip_index][0] <= index:
            continue
        where = f"{shown}:{lines[index]}"
        word = texts[index]
        if (kinds[index] == units.IDENT and word in INCLUDE_MACROS and texts[index + 1:index + 2] == ["!"]
                and texts[index + 2:index + 3] in (["("], ["["], ["{"])) and not broken("includes-ignored"):
            closing = matching_close(texts, index + 2)
            consumed.update(range(index + 2, closing + 1))
            target, why = include_argument_path(kinds, texts, index + 2, closing, file_directory, package_directory)
            if why is not None:
                problems.append(f"{where}：{word}!{why if why.startswith('(') else ' 的' + why}")
            elif target is not None:
                references.append(RustReference(target, where, True))
            continue
        if word == "path" and texts[index - 2:index] == ["#", "["] and texts[index + 1:index + 2] == ["="] and not broken("includes-ignored"):
            value = string_value(kinds, texts, index + 2)
            if value is not None:
                consumed.add(index + 2)
                references.append(RustReference(os.path.normpath(os.path.join(file_directory, value)), where, True))
            continue
        if index in consumed or kinds[index] != units.LITERAL:
            continue
        value = units.rust_string_value(word)
        if value is None:
            continue
        if value == ".." and texts[index - 3:index] == [".", "join", "("] and texts[index + 1:index + 2] == [")"]:
            if not broken("join-chains-ignored"):
                references.append(RustReference(join_chain_path(kinds, texts, index, package_directory), where, False))
            continue
        target = None if broken("runtime-literals-ignored") else runtime_literal_path(value, package_directory)
        if target is not None:
            references.append(RustReference(target, where, False))
    return references, problems


def rust_sources(package_root, scan_test_code, nested_packages):
    """包目录底下的 .rs（不进编译目录与别的包目录；不扫测试代码时不进 tests/、benches/、examples/）。"""
    found = []
    for directory, subdirectories, names in os.walk(package_root):
        kept = []
        for name in sorted(subdirectories):
            full = os.path.join(directory, name)
            if skipped_directory(name) or full in nested_packages:
                continue
            if not scan_test_code and directory == package_root and name in TEST_CODE_DIRECTORIES:
                continue
            kept.append(name)
        subdirectories[:] = kept
        found += [os.path.join(directory, name) for name in sorted(names) if name.endswith(".rs")]
    return found


def rust_references(root, packages, scan_test_code):
    """各包（相对仓根）里指到包目录之外的路径与认不出的 include 写法（逐份读：粗筛掉的不切词）。"""
    absolute_packages = {os.path.join(root, package) for package in packages}
    jobs = [(os.path.join(root, package), path) for package in packages
            for path in rust_sources(os.path.join(root, package), scan_test_code, absolute_packages - {os.path.join(root, package)})]
    references, problems = [], []
    for package_root, path in jobs:
        found, refused = rust_file_references(root, path, package_root, scan_test_code)
        references += found
        problems += refused
    return references, problems


# ── 被起、被 source 的脚本 ──

def is_script(path):
    if path.endswith(SCRIPT_SUFFIXES):
        return True
    try:
        with open(path, "rb") as handle:
            return handle.read(2) == b"#!"
    except OSError:
        return False


def shell_references(text, directory):
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("#"):
            continue
        found += [(number, os.path.normpath(os.path.join(directory, match.group(1)))) for match in SHELL_REFERENCE.finditer(line)]
    return found


def join_base_argument(text):
    """os.path.join 括号里的参数正文 → 拼的起点那个参数（逗号前头那一段：到不在括号、引号里的逗号为止，去掉两头空白）。"""
    depth, quote = 0, None
    for index, character in enumerate(text):
        if quote:
            if character == quote:
                quote = None
        elif character in "'\"":
            quote = character
        elif character in "([{":
            depth += 1
        elif character in ")]}":
            depth -= 1
        elif character == "," and depth == 0:
            return text[:index].strip()
    return text.strip()


def python_references(text, directory):
    """一份 python 脚本点到的仓内文件：[(行号, 绝对路径)]（不查在不在）。文件头「闭包」被起、被 source 的那一条：import、os.path.join、引号里的脚本名，
    与赋给名字的目录。弄坏开关 python-directory-variables-ignored：名字不记，os.path.join(<名>, …) 按脚本所在目录解。"""
    lines = text.splitlines()
    search_directories = [directory]
    named = {}   # 赋给名字的目录 {名: [绝对路径, …]}：同一个名字赋几次的都记
    tracked = not broken("python-directory-variables-ignored")
    found = []
    for number, line in enumerate(lines, 1):
        if line.lstrip().startswith("#"):
            continue
        assignment = PYTHON_ASSIGNMENT.match(line) if tracked else None
        if assignment and PYTHON_SCRIPT_DIRECTORY.match(re.sub(r"\s+#.*$", "", assignment.group(2))):
            named.setdefault(assignment.group(1), []).append(directory)
        for call in PYTHON_JOIN_CALL.finditer(line):
            segments = PYTHON_QUOTED_SEGMENT.findall(call.group(1))
            if not segments:
                continue
            base_argument = join_base_argument(call.group(1))
            bases = named.get(base_argument, [directory]) if tracked and PYTHON_NAME.fullmatch(base_argument) else [directory]
            joined = [os.path.normpath(os.path.join(base, *segments)) for base in bases]
            if segments[-1].endswith(SCRIPT_SUFFIXES):
                found += [(number, path) for path in joined]
            elif "sys.path" in line:
                search_directories += joined
            whole_right_side = (assignment and re.fullmatch(r"\s*(?:#.*)?", line[call.end():]) is not None
                                and re.fullmatch(r"(?:\w+\.)*", line[assignment.start(2):call.start()]) is not None)
            if whole_right_side:
                named.setdefault(assignment.group(1), []).extend(joined)
        if tracked:
            for added in PYTHON_SYS_PATH_NAME.finditer(line):
                search_directories += named.get(added.group(1), [])
        found += [(number, os.path.normpath(os.path.join(directory, match.group(1)))) for match in PYTHON_QUOTED_SCRIPT.finditer(line)]
    for number, line in enumerate(lines, 1):
        match = PYTHON_IMPORT.match(line)
        if not match:
            continue
        modules = [match.group(1)] if match.group(1) else [name.strip() for name in match.group(2).split(",")]
        for module in modules:
            first = module.split(".")[0]
            for search in search_directories:
                for candidate in (os.path.join(search, first + ".py"), os.path.join(search, first, "__init__.py")):
                    found.append((number, os.path.normpath(candidate)))
    return found


def anchored_shell_references(text, directory):
    """shell 脚本里行首 source、起、照脚本自己所在目录写路径的文件（ANCHORED_SHELL_REFERENCE）：[(行号, 绝对路径)]，不查在不在；注释行不算。"""
    found = []
    for number, line in enumerate(text.splitlines(), 1):
        if line.lstrip().startswith("#"):
            continue
        found += [(number, os.path.normpath(os.path.join(directory, match.group(1)))) for match in ANCHORED_SHELL_REFERENCE.finditer(line)]
    return found


def script_closure(root, scripts, gaps=None):
    """scripts（相对仓根）连同它们 source、起、import 的仓内文件，传递地收：{相对仓根的路径: 出处}（起点的出处是空串）。
    gaps 给了一张表的：shell 脚本照自己所在目录 source、起的文件在源树里不在的，往里记一句「<脚本>:<行> 引到的 <路径>」（文件头「闭包」声明的条目那一条）。"""
    found = {script: "" for script in scripts}
    pending = list(scripts)
    while pending:
        current = pending.pop()
        path = os.path.join(root, current)
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        if current.endswith(".py") or text.startswith("#!/usr/bin/env python"):
            named = [] if broken("python-references-not-followed") else python_references(text, os.path.dirname(path))
        else:
            named = [] if broken("shell-sources-not-followed") else shell_references(text, os.path.dirname(path))
            if gaps is not None:
                for number, target in anchored_shell_references(text, os.path.dirname(path)):
                    if not os.path.isfile(target):
                        gaps.append(f"{current}:{number} 引到的 {relative_inside(root, target) or target}")
        for number, target in named:
            relative = relative_inside(root, target)
            if relative is None or relative == "." or relative in found or not os.path.isfile(target):
                continue
            found[relative] = f"{current}:{number}"
            pending.append(relative)
    return found


# ── 闭包 ──

def checked_declarations(declared):
    """declared：[(条目, 为什么)] → [(规整过的条目, 为什么)]；为什么不到 MINIMUM_REASON_CHARACTERS 个字、条目是绝对路径或跳出仓根的抛 ValueError。"""
    checked = []
    for entry, reason in declared:
        normalized = os.path.normpath(entry.strip().rstrip("/"))
        if os.path.isabs(normalized) or normalized == "." or normalized.startswith(".."):
            raise ValueError(f"声明的条目「{entry}」不是仓根下的相对路径")
        if len((reason or "").strip()) < MINIMUM_REASON_CHARACTERS and not broken("declared-reason-unchecked"):
            raise ValueError(f"声明的条目「{entry}」没写为什么（至少 {MINIMUM_REASON_CHARACTERS} 个字）")
        checked.append((normalized, reason.strip() if reason else ""))
    return checked


def declared_scripts(root, entries):
    """声明的条目里的脚本（文件头「闭包」声明的条目那一条）：entries（相对仓根）里是脚本的那一条本身，与是目录的那一条底下逐层的每一份脚本
    （不进 skipped_directory 那几种与 DECLARED_SKIPPED_DIRECTORY_NAMES）；交排好的相对仓根的路径表。弄坏开关 declared-directories-not-followed 下目录不往下找。"""
    found = set()
    for entry in entries:
        path = os.path.join(root, entry)
        if os.path.isfile(path) and is_script(path):
            found.add(entry)
        elif os.path.isdir(path) and not os.path.islink(path) and not broken("declared-directories-not-followed"):
            for directory, subdirectories, names in os.walk(path):
                subdirectories[:] = sorted(name for name in subdirectories if not skipped_directory(name) and name not in DECLARED_SKIPPED_DIRECTORY_NAMES)
                found.update(os.path.relpath(os.path.join(directory, name), root) for name in names if is_script(os.path.join(directory, name)))
    return sorted(found)


def copy_closure(root, declared=(), *, test_inputs=False, workspace=None):
    """root 这棵树要拷进副本的仓根条目（文件头「闭包」）。declared 是调用方多拷的 [(条目, 为什么)]；test_inputs 为真时连测试代码一起扫；
    workspace 给了（相对仓根的目录）时清单只有那个工作区目录本身（只拷一个工作区的副本，mutate.sh 那一种），不带 COMMON_ENTRIES。
    算不成抛 ClosureRefused；声明写坏了抛 ValueError。每次调都现读那棵树（不缓存：同一进程里树会变，自证与门禁都在同一棵树上改了再算）。"""
    root = os.path.realpath(root)
    declarations = checked_declarations(declared)
    scan_test_code = (test_inputs and not broken("test-inputs-ignored")) or broken("test-code-scanned")
    reasons = {}

    def add(entry, why):
        reasons.setdefault(entry, [])
        if why not in reasons[entry]:
            reasons[entry].append(why)

    base = (os.path.normpath(workspace),) if workspace else COMMON_ENTRIES
    for entry in base:
        if os.path.lexists(os.path.join(root, entry)):
            add(entry, "共用清单" if not workspace else "要拷的工作区")
    for entry, why in declarations:
        if os.path.lexists(os.path.join(root, entry)):
            add(entry, f"调用方声明：{why}")
    for entry, where in path_dependency_entries(root, sorted(reasons)).items():
        add(entry, f"path 依赖（{where}）")
    packages = package_directories(root, sorted(reasons))
    references, problems = rust_references(root, packages, scan_test_code)
    left_out, missing, started, reached_scripts = {}, {}, [], []
    for reference in references:
        relative = relative_inside(root, reference.path)
        if relative is None:
            if reference.compile_time:
                problems.append(f"{reference.where}：编译期读的 {reference.path} 落到仓根之外")
            continue
        if relative == ".":
            continue
        never = next((name for name in NEVER_COPIED if relative == name or relative.startswith(name + "/")), None)
        if never is not None and not broken("never-copied-copied"):
            left_out.setdefault(never, []).append(reference.where)
            continue
        exists = os.path.lexists(os.path.join(root, relative))
        if exists and not reference.compile_time and is_script(os.path.join(root, relative)) and relative not in started:
            started.append(relative)
        if exists and os.path.isfile(os.path.join(root, relative)) and is_script(os.path.join(root, relative)) and relative not in reached_scripts:
            reached_scripts.append(relative)   # 罩在目录条目里的也记：下面追 source 链从它起
        if is_covered(relative, list(reasons)):
            continue
        if not exists:
            if reference.compile_time:
                problems.append(f"{reference.where}：编译期读的 {relative} 在仓里没有")
            else:
                missing.setdefault(relative, []).append(reference.where)
            continue
        add(relative, ("编译期读" if reference.compile_time else "运行期读") + f"（{reference.where}）")
    if problems:
        raise ClosureRefused("；".join(problems))
    # 追 source 链的起点（文件头「闭包」被起、被 source 的那一条）：单独成条的脚本加 Rust 代码指到的每一份脚本。
    # 弄坏开关 covered-scripts-not-followed：只拿单独成条的，罩在目录条目里的不起头
    scripts = [entry for entry in reasons if os.path.isfile(os.path.join(root, entry)) and is_script(os.path.join(root, entry))]
    if not broken("covered-scripts-not-followed"):
        scripts += reached_scripts
    for path, where in sorted(script_closure(root, sorted(set(scripts))).items()):
        if where and not is_covered(path, list(reasons)):
            add(path, f"被 source、被起（{where}）")
    # 声明的条目里的脚本也当起点（文件头「闭包」声明的条目那一条）：单个脚本，与声明的目录里的每一份脚本；这一支算不全（照脚本所在目录引到的文件不在）的拒
    declared_starts = declared_scripts(root, [entry for entry, _ in declarations])
    gaps = []
    declared_found = script_closure(root, declared_starts, gaps)
    if gaps and not broken("declared-closure-gaps-accepted"):
        raise ClosureRefused("声明的条目里的脚本照自己所在目录 source、起的文件在源树里不在，闭包算不全、不拷半份："
                             + "；".join(sorted(set(gaps))) + "；补上那几份，或改掉引到它们的那一行")
    for path, where in sorted(declared_found.items()):
        if where and not is_covered(path, list(reasons)):
            add(path, f"{DECLARED_SCRIPTS_REASON}（{where}）")
    for entry in list(reasons):
        if entry == IGNORED_BY_GIT_PREFIX or entry.startswith(IGNORED_BY_GIT_PREFIX + "/"):
            add(entry, "被 .gitignore 挡着的规范副本，按 git 列文件的拷法带不上，照文件系统拷")
    for entry in [entry for entry in reasons if is_covered(entry, [other for other in reasons if other != entry])]:
        del reasons[entry]   # 落在另一条目录条目里面的（先认了文件、后认了它所在的目录）并进那一条
    return CopyClosure(tuple(sorted(reasons)), {entry: tuple(why) for entry, why in reasons.items()}, tuple(sorted(started)),
                       {name: tuple(places) for name, places in left_out.items()}, {name: tuple(places) for name, places in missing.items()})


def entries_missing_in_copy(root, copy_root, declared=(), *, test_inputs=False):
    """copy_root 这份副本缺的仓根条目：root 那棵树的闭包（copy_closure，同一组 declared 与 test_inputs）里副本根下不在的那几条。
    交 ([(条目, (为什么, …)), …]（照闭包的次序）, 闭包的条数)。只看条目本身在不在：目录条目在就算在，不比里面的文件与内容（副本里的改动是调用方自己的）。
    闭包算不成抛 ClosureRefused；声明写坏了抛 ValueError。missing-in-copy 子命令与吃副本的脚本（scripts/mutation/prove-red.sh --copy）都调这一份。
    弄坏开关 copy-completeness-unchecked：一条都不报。"""
    closure = copy_closure(root, declared, test_inputs=test_inputs)
    copy_root = os.path.realpath(copy_root)
    missing = [(entry, closure.reasons[entry]) for entry in closure.entries if not os.path.lexists(os.path.join(copy_root, entry))]
    return ([] if broken("copy-completeness-unchecked") else missing), len(closure.entries)


def copy_rebuild_howto(root):
    """副本缺闭包里的条目时的出路（missing-in-copy 与吃副本的脚本照这一句写）：照 minimal-copy.py 另建一份，不手补。"""
    return (f"仓副本只用 python3 scripts/hosts/minimal-copy.py {root} <新副本目录> --with-test-inputs 建（拷哪些由 scripts/hosts/lib_repository_copy.py 的闭包现算；"
            "建时给过的 --with 照给），副本里自己的改动搬进新副本再用；不手拷、不手补缺的条目")


# ── 拷与副本里的环境 ──

def copy_entries(root, destination, entries):
    """把 entries（相对 root）拷进 destination（文件头「拷法」）；rsync 退非 0 抛 OSError。"""
    present = [entry for entry in entries if os.path.lexists(os.path.join(root, entry))]
    os.makedirs(destination, exist_ok=True)
    excludes = ["--exclude", ".git", "--exclude", "target/", "--exclude", "target-*/", "--exclude", "__pycache__"]
    if broken("copy-without-parent-directories"):
        for entry in present:
            source = os.path.join(root, entry)
            suffix = "/" if os.path.isdir(source) else ""
            completed = subprocess.run(["rsync", "-a", "--no-times", "--checksum", *excludes, source + suffix, os.path.join(destination, entry) + suffix],
                                       capture_output=True, text=True, stdin=subprocess.DEVNULL)
            if completed.returncode != 0:
                raise OSError(f"rsync 退 {completed.returncode}：{completed.stderr.strip()[-300:]}")
        return
    if not present:
        return
    completed = subprocess.run(["rsync", "-a", "--no-times", "--checksum", "--sparse", "--relative", *excludes,
                                *["./" + entry for entry in present], destination.rstrip("/") + "/"],
                               cwd=root, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if completed.returncode != 0:
        raise OSError(f"rsync 退 {completed.returncode}：{completed.stderr.strip()[-300:]}")


def main_working_tree(source_root):
    """source_root 所在的主工作区（绝对路径）：git worktree 里是主仓那一棵；不在 git 里交 source_root 自己。"""
    source_root = os.path.realpath(source_root)
    if broken("environment-configuration-by-tree"):
        return source_root
    try:
        completed = subprocess.run(["git", "-C", source_root, "rev-parse", "--path-format=absolute", "--git-common-dir"],
                                   capture_output=True, text=True, stdin=subprocess.DEVNULL)
    except OSError:
        return source_root
    common = completed.stdout.strip()
    return os.path.dirname(os.path.realpath(common)) if completed.returncode == 0 and common else source_root


def configuration_path(source_root, environment=None):
    environment = os.environ if environment is None else environment
    return environment.get(CONFIGURATION_VARIABLE) or os.path.join(main_working_tree(source_root), CONFIGURATION_FILE_NAME)


def environment_for_commands_in_the_copy(source_root, environment=None):
    """副本里起命令用的环境（文件头「副本里起命令的环境」）：调用方的环境的一份拷贝，SINGLEFS_MULTI_HOST_CONFIG 设成主工作区那一份配置的绝对路径。"""
    copied = dict(os.environ if environment is None else environment)
    if not broken("environment-configuration-unset"):
        copied[CONFIGURATION_VARIABLE] = configuration_path(source_root, copied)
    return copied


def link_into(root, copy_root, closure, present):
    """闭包里副本根下还没有、也不落在 present 里的条目，逐个符号链接回仓里那一份；交链了的条目。"""
    linked = []
    for entry in closure.entries:
        if is_covered(entry, present):
            continue
        parts = entry.split("/")
        if any(os.path.islink(os.path.join(copy_root, *parts[:depth])) for depth in range(1, len(parts) + 1)) or os.path.lexists(os.path.join(copy_root, entry)):
            continue
        os.makedirs(os.path.dirname(os.path.join(copy_root, entry)) or copy_root, exist_ok=True)
        os.symlink(os.path.join(root, entry), os.path.join(copy_root, entry))
        linked.append(entry)
    return linked


# ── 真编核对 ──

def probes_in_force():
    """RUNTIME_PROBES（弄坏开关 probe-dropped：去掉多机配置判法那一条）。"""
    return {script: argv for script, argv in RUNTIME_PROBES.items() if not (broken("probe-dropped") and script == CONFIGURATION_CHECK)}


def inside_git_work_tree(path):
    completed = subprocess.run(["git", "-C", path, "rev-parse", "--is-inside-work-tree"], capture_output=True, text=True, stdin=subprocess.DEVNULL)
    return completed.returncode == 0 and completed.stdout.strip() == "true"


def run_probe(argv, tree, environment):
    """在副本里起一条运行期脚本：交 (退出码, 标准输出里等号前的键名, 标准错误末几行)。值不交出（多机配置的值是私有的）。"""
    command = [word.replace("{tree}", tree) for word in argv]
    try:
        completed = subprocess.run(command, cwd=tree, env=environment, capture_output=True, text=True, stdin=subprocess.DEVNULL,
                                   timeout=PROBE_TIMEOUT_SECONDS, start_new_session=True)
    except subprocess.TimeoutExpired:
        return 124, [], [f"超过 {PROBE_TIMEOUT_SECONDS} 秒"]
    keys = sorted({line.split("=", 1)[0] for line in completed.stdout.splitlines() if re.match(r"^[A-Z][A-Z0-9_]*=", line)})
    return completed.returncode, keys, [line for line in completed.stderr.splitlines() if line.strip()][-3:]


def verify_copy(root, work, target_directory, cargo_arguments, declared=(), test_inputs=False, probes=None, cargo="cargo", emit=print):
    """真编核对（文件头「用法」verify-copy）：交 (问题列表, 判了几格)。probes 不给用 RUNTIME_PROBES；cargo 是起哪一份 cargo（自证换假的）。"""
    probes = probes_in_force() if probes is None else probes
    problems, checked = [], 0
    closure = copy_closure(root, declared, test_inputs=test_inputs)
    tree = os.path.join(work, "tree")
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    try:
        copy_entries(root, tree, closure.entries)
        in_git = inside_git_work_tree(tree)
        emit(f"REPOSITORY_COPY_VERIFY tree entries={len(closure.entries)} inside_git_work_tree={str(in_git).lower()} "
             f"started={','.join(closure.started) or '-'}")
        environment = environment_for_commands_in_the_copy(root)
        environment["CARGO_TARGET_DIR"] = target_directory
        command = [cargo, "build", *cargo_arguments]
        completed = subprocess.run(command, cwd=tree, env=environment, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                   text=True, errors="replace")
        checked += 1
        errors = [line for line in completed.stdout.splitlines() if line.startswith("error")][:20]
        emit(f"REPOSITORY_COPY_VERIFY cargo exit={completed.returncode} command={' '.join(['cargo', *command[1:]])}")
        for line in errors:
            emit(f"    {line}")
        if completed.returncode != 0:
            problems.append(f"副本里 cargo build 退 {completed.returncode}（上面是 error 起头的几行）")
        if broken("verify-probes-skipped"):
            return problems, checked
        for script, argv in sorted(probes.items()):
            if not is_covered(script, closure.entries):
                continue
            exit_code, keys, tail = run_probe(argv, tree, environment)
            checked += 1
            emit(f"REPOSITORY_COPY_VERIFY probe={script} exit={exit_code} keys={','.join(keys) or '-'}")
            if exit_code != 0:
                problems.append(f"副本里起 {script} 退 {exit_code}：{tail}")
            if script == CONFIGURATION_CHECK:
                without = {name: value for name, value in environment.items() if name != CONFIGURATION_VARIABLE}
                refused_exit, _, _ = run_probe(argv, tree, without)
                emit(f"REPOSITORY_COPY_VERIFY probe={script} without_{CONFIGURATION_VARIABLE} exit={refused_exit} judged={str(not in_git).lower()}")
                if not in_git:
                    checked += 1
                    if refused_exit == 0:
                        problems.append(f"副本不在 git 工作树里、去掉 {CONFIGURATION_VARIABLE} 之后 {script} 照样退 0：判不出副本环境起了作用")
    finally:
        shutil.rmtree(tree, ignore_errors=True)
    return problems, checked


# ── 命令行 ──

def parse_declarations(words):
    declared = []
    for word in words:
        entry, separator, reason = word.partition("=")
        if not separator:
            raise ValueError(f"--with 要写成 <条目>=<为什么>，读到「{word}」")
        declared.append((entry, reason))
    return declared


def command_entries(arguments):
    root, declared_words, test_inputs = None, [], False
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word == "--test-inputs":
            test_inputs = True
        elif word == "--with" and index + 1 < len(arguments):
            declared_words.append(arguments[index + 1])
            index += 1
        elif not word.startswith("-") and root is None:
            root = word
        else:
            raise ValueError(f"认不出的参数「{word}」")
        index += 1
    if root is None:
        raise ValueError("要给 <仓根>")
    closure = copy_closure(root, parse_declarations(declared_words), test_inputs=test_inputs)
    for entry in closure.entries:
        print(f"REPOSITORY_COPY entry {entry}\t{'；'.join(closure.reasons[entry])}")
    for name, places in sorted(closure.left_out.items()):
        print(f"REPOSITORY_COPY left_out {name}\t{'、'.join(places)}\t{NEVER_COPIED[name]}")
    for name, places in sorted(closure.missing.items()):
        print(f"REPOSITORY_COPY missing {name}\t{'、'.join(places)}")
    print(f"REPOSITORY_COPY entries={len(closure.entries)} started={','.join(closure.started) or '-'}")
    return 0


def command_link_into(arguments):
    copy_root, root, workspace, present, test_inputs = None, None, None, [], False
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word in ("--root", "--workspace", "--present") and index + 1 < len(arguments):
            value = arguments[index + 1]
            if word == "--root":
                root = value
            elif word == "--workspace":
                workspace = value
            else:
                present.append(os.path.normpath(value))
            index += 1
        elif word == "--test-inputs":
            test_inputs = True
        elif not word.startswith("-") and copy_root is None:
            copy_root = word
        else:
            raise ValueError(f"认不出的参数「{word}」")
        index += 1
    if copy_root is None or root is None or workspace is None:
        raise ValueError("要给 <副本根>、--root <仓根>、--workspace <工作区目录>")
    root = os.path.realpath(root)
    try:
        closure = copy_closure(root, test_inputs=test_inputs, workspace=workspace)
    except ClosureRefused as error:
        for problem in str(error).split("；"):
            print(f"  ✗ 副本里读不到要编的仓内文件：{problem}", file=sys.stderr)   # gate-lint:detail
        print("  → 怎么办：编译期读的文件写成 include_str!(\"<相对这份源文件、落在仓里的路径>\") 或 concat!(env!(\"CARGO_MANIFEST_DIR\"), \"…\")，"
              "并让那份文件进仓；path 依赖指回仓根里面", file=sys.stderr)
        return EXIT_USAGE
    linked = link_into(root, os.path.realpath(copy_root), closure, [os.path.normpath(workspace), *present])
    if linked:
        print(f"副本根下链了 {len(linked)} 份 {os.path.normpath(workspace)}/ 之外的仓内文件（闭包算出的）：{'、'.join(linked)}", file=sys.stderr)
    return 0


def command_missing_in_copy(arguments):
    copy_root, root, declared_words, test_inputs = None, REPOSITORY_ROOT, [], False
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word in ("--root", "--with") and index + 1 < len(arguments):
            if word == "--root":
                root = arguments[index + 1]
            else:
                declared_words.append(arguments[index + 1])
            index += 1
        elif word == "--test-inputs":
            test_inputs = True
        elif not word.startswith("-") and copy_root is None:
            copy_root = word
        else:
            raise ValueError(f"认不出的参数「{word}」")
        index += 1
    if copy_root is None:
        raise ValueError("要给 <副本根>")
    if not os.path.isdir(copy_root):
        raise ValueError(f"副本根 {copy_root} 不是目录")
    root = os.path.realpath(root)
    missing, checked = entries_missing_in_copy(root, copy_root, parse_declarations(declared_words), test_inputs=test_inputs)
    for entry, reasons in missing:
        print(f"REPOSITORY_COPY missing_in_copy {entry}\t{'；'.join(reasons)}")
    if missing:
        print(f"  ✗ 副本 {copy_root} 缺 {len(missing)} 条闭包里的仓根条目（闭包共 {checked} 条，照 {root} 算）：{'、'.join(entry for entry, _ in missing)}，"
              "在副本里编、跑会读不到", file=sys.stderr)
        print(f"  → 怎么办：{copy_rebuild_howto(root)}", file=sys.stderr)
        return EXIT_REFUSED
    print(f"REPOSITORY_COPY copy-complete entries={checked}")
    return 0


def command_verify_copy(arguments):
    root, work, declared_words, test_inputs = REPOSITORY_ROOT, None, [], False
    if "--" not in arguments:
        raise ValueError("要在 -- 后面写 cargo build 的参数（例：-- --release --offline -p singlefs-verification --features gpu）")
    split = arguments.index("--")
    options, cargo_arguments = arguments[:split], arguments[split + 1:]
    if not cargo_arguments:
        raise ValueError("-- 后面没有 cargo build 的参数")
    index = 0
    while index < len(options):
        word = options[index]
        if word in ("--root", "--work", "--with") and index + 1 < len(options):
            value = options[index + 1]
            if word == "--root":
                root = value
            elif word == "--work":
                work = value
            else:
                declared_words.append(value)
            index += 1
        elif word == "--test-inputs":
            test_inputs = True
        else:
            raise ValueError(f"认不出的参数「{word}」")
        index += 1
    service_target = os.environ.get("CARGO_TARGET_DIR")
    if work is None:
        if not service_target:
            raise ValueError("没给 --work，环境里也没有服务给的 CARGO_TARGET_DIR")
        work = os.path.join(service_target, VERIFY_WORK_DIRECTORY_NAME)
    os.makedirs(work, exist_ok=True)
    target_directory = service_target or os.path.join(work, "target")
    problems, checked = verify_copy(os.path.realpath(root), os.path.realpath(work), target_directory, cargo_arguments,
                                    parse_declarations(declared_words), test_inputs)
    for problem in problems:
        print(f"  ✗ {problem}")   # gate-lint:detail
    if problems:
        print("  → 怎么办：cargo 报读不到的仓根文件，先看它是不是闭包漏认的写法（文件头「管不到的」），改成认得出的写法或经 --with 声明；"
              "运行期脚本退非 0 看它的末几行")
        print(f"REPOSITORY_COPY_VERIFY failed checked={checked} problems={len(problems)}")
        return EXIT_REFUSED
    print(f"REPOSITORY_COPY_VERIFY passed checked={checked}")
    return 0


# ── 自证 ──
OLD_MTIME_SECONDS = 1577836800     # 假树里的文件带着这个旧修改时刻（2020 年 1 月 1 日）
RECENT_SECONDS = 3600              # 「此刻」：拷过去的文件的修改时刻离现在不超过这么多秒
DEMO_MANIFEST = '[package]\nname = "demo"\nversion = "0.1.0"\nedition = "2021"\n'
ROOT_MANIFEST = '[workspace]\nresolver = "2"\nmembers = ["crates/demo"]\nexclude = ["served"]\n'
SELFTEST_GIT = ("git", "-c", "user.name=selftest", "-c", "user.email=selftest@example.invalid", "-c", "init.defaultBranch=main")


def write(tree, relative, text, mtime_seconds=OLD_MTIME_SECONDS):
    path = os.path.join(tree, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.utime(path, (mtime_seconds, mtime_seconds))
    return path


def make_root(base, name):
    """一棵假仓：工作区（crates/demo）、Cargo.lock、.cargo/，另带不该拷的 research/ 与编译目录。"""
    root = os.path.join(base, name, "root")
    write(root, "Cargo.toml", ROOT_MANIFEST)
    write(root, "Cargo.lock", "# lock\n")
    write(root, ".cargo/config.toml", "[build]\n")
    write(root, "crates/demo/Cargo.toml", DEMO_MANIFEST)
    write(root, "crates/demo/src/lib.rs", "pub fn value() -> u32 {\n    1\n}\n")
    write(root, "crates/demo/target/debug/artifact.bin", "x" * 100)
    write(root, "research/results/big.out", "r" * 100)
    return root


def closure_or_error(root, *arguments, **options):
    """copy_closure 的 (闭包, None) 或 (None, 抛出的错)。"""
    try:
        return copy_closure(root, *arguments, **options), None
    except (ClosureRefused, ValueError) as error:
        return None, error


def tree_files(tree):
    return sorted(os.path.relpath(os.path.join(directory, name), tree) for directory, _, names in os.walk(tree) for name in names)


# 独立的粗扫（格 repository-root-references-in-the-closure）：逐行正则、不切词，与闭包那一份判法不共用代码，两边对得上才算闭包没漏
COARSE_LITERAL = re.compile(r'"((?:\{[A-Za-z_][A-Za-z0-9_]*\})?/?\.\./[^"\\]*)"')
COARSE_INCLUDE_LINE = re.compile(r'\binclude(?:_str|_bytes)?!\(\s*(?:"|$)')
COARSE_JOIN_CHAIN = re.compile(r'\.join\(\s*"\.\."\s*\)((?:\s*\.join\(\s*"[^"]+"\s*\))+)')
COARSE_JOIN_PIECE = re.compile(r'\.join\(\s*"([^"]+)"\s*\)')
COARSE_TEST_MODULE = re.compile(r'(?:pub(?:\(crate\))? )?mod \w+ \{\s*$')
COARSE_SHELL_REFERENCE = re.compile(r'(?:\}|\)|_directory|_DIRECTORY)/((?:\.\./)*[\w.\-]+(?:/[\w.\-]+)*\.(?:sh|py))\b')


def coarse_test_lines(lines):
    inside, index = set(), 0
    while index < len(lines):
        if lines[index].rstrip() == "#[cfg(test)]" and index + 1 < len(lines) and COARSE_TEST_MODULE.match(lines[index + 1]):
            closing = index + 2
            while closing < len(lines) and lines[closing].rstrip() != "}":
                closing += 1
            inside.update(range(index, closing + 1))
            index = closing + 1
            continue
        index += 1
    return inside


def coarse_repository_root_references(root):
    """真仓里各包（crates/ 下的与它们 path 依赖到的仓根目录）非测试代码与 build.rs 指到仓根的路径，加 shell 脚本 source 的：{相对仓根的路径: 出处}。"""
    package_roots = [os.path.join(root, "crates", name) for name in sorted(os.listdir(os.path.join(root, "crates")))
                     if os.path.isfile(os.path.join(root, "crates", name, "Cargo.toml"))]
    found = {}
    for package_root in list(package_roots):
        with open(os.path.join(package_root, "Cargo.toml"), "rb") as handle:
            manifest = tomllib.load(handle)
        for table in [manifest.get(name, {}) for name in ("dependencies", "build-dependencies")]:
            for specification in table.values():
                if isinstance(specification, dict) and "path" in specification:
                    target = os.path.normpath(os.path.join(package_root, specification["path"]))
                    relative = os.path.relpath(target, root)
                    if not relative.startswith("crates/") and not relative.startswith(".."):
                        found.setdefault(relative, os.path.relpath(package_root, root) + "/Cargo.toml")
                        if target not in package_roots:
                            package_roots.append(target)
    for package_root in package_roots:
        for directory, subdirectories, names in os.walk(package_root):
            subdirectories[:] = sorted(name for name in subdirectories
                                       if name != "target" and not (directory == package_root and name in TEST_CODE_DIRECTORIES))
            for name in sorted(name for name in names if name.endswith(".rs")):
                path = os.path.join(directory, name)
                with open(path, encoding="utf-8", errors="replace") as handle:
                    text = handle.read()
                lines = text.splitlines()
                test_lines = coarse_test_lines(lines)
                for index, line in enumerate(lines):
                    if index in test_lines or line.lstrip().startswith("//"):
                        continue
                    include = COARSE_INCLUDE_LINE.search(line) or (index > 0 and COARSE_INCLUDE_LINE.search(lines[index - 1]))
                    for literal in COARSE_LITERAL.finditer(line):
                        value = re.sub(r'^\{[A-Za-z_][A-Za-z0-9_]*\}', "", literal.group(1)).lstrip("/")
                        base = directory if include and not literal.group(1).startswith("/") else package_root
                        found.setdefault(os.path.relpath(os.path.normpath(os.path.join(base, value)), root), f"{os.path.relpath(path, root)}:{index + 1}")
                for chain in COARSE_JOIN_CHAIN.finditer(text):
                    number = text.count("\n", 0, chain.start())
                    if number in test_lines:
                        continue
                    target = os.path.normpath(os.path.join(package_root, "..", *COARSE_JOIN_PIECE.findall(chain.group(1))))
                    found.setdefault(os.path.relpath(target, root), f"{os.path.relpath(path, root)}:{number + 1}")
    pending = [path for path in found if path.endswith(".sh") and os.path.isfile(os.path.join(root, path))]
    while pending:
        current = pending.pop()
        with open(os.path.join(root, current), encoding="utf-8", errors="replace") as handle:
            for number, line in enumerate(handle, 1):
                if line.lstrip().startswith("#"):
                    continue
                for reference in COARSE_SHELL_REFERENCE.finditer(line):
                    target = os.path.relpath(os.path.normpath(os.path.join(root, os.path.dirname(current), reference.group(1))), root)
                    if target not in found and os.path.isfile(os.path.join(root, target)):
                        found[target] = f"{current}:{number}"
                        if target.endswith(".sh"):
                            pending.append(target)
    return {path: where for path, where in found.items()
            if not path.startswith("..") and path != "." and not path.startswith("crates/") and os.path.lexists(os.path.join(root, path))}


def closure_against_the_coarse_scan(root):
    """root 上闭包与独立粗扫的交叉核：交 (闭包或 None, 算不成时抛出的错, 粗扫结果, 粗扫到而不在闭包与 NEVER_COPIED 里的 {路径: 出处}, 闭包里在仓里不在的条目)。"""
    closure, error = closure_or_error(root)
    coarse = coarse_repository_root_references(root)
    uncovered = {path: where for path, where in coarse.items() if closure is None or not is_covered(path, [*closure.entries, *NEVER_COPIED])}
    absent = [entry for entry in (closure.entries if closure else ()) if not os.path.lexists(os.path.join(root, entry))]
    return closure, error, coarse, uncovered, absent


def selftest_cells(base):
    """各格：交 (失败的格名列表, 查了几格)。每格自己一棵假仓（真仓那两格读本仓）。"""
    failures, checked = [], 0

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    # path-dependencies-followed：crates/demo 经 path 依赖仓根下的 served/（它在 target.<条件> 下再依赖 nested/）⇒ 两处都进闭包，unrelated/、research/ 不进
    root = make_root(base, "path-dependencies-followed")
    write(root, "crates/demo/Cargo.toml", DEMO_MANIFEST + '[dependencies]\nserved = { path = "../../served", optional = true }\n')
    write(root, "served/Cargo.toml", '[package]\nname = "served"\n[target.\'cfg(unix)\'.dependencies]\nnested = { path = "../nested" }\n')
    write(root, "served/src/lib.rs", "pub fn served() {}\n")
    write(root, "nested/Cargo.toml", '[package]\nname = "nested"\n')
    write(root, "unrelated/Cargo.toml", '[package]\nname = "unrelated"\n')
    closure, error = closure_or_error(root)
    wanted = (".cargo", "Cargo.lock", "Cargo.toml", "crates", "nested", "served")
    check("path-dependencies-followed", closure is not None and closure.entries == wanted,
          f"闭包应当是 {wanted}，实际 {closure.entries if closure else error}")

    # path-dependency-outside-root-refused：path 依赖指到仓根之外 ⇒ 抛 ClosureRefused、点名它与「仓根」
    root = make_root(base, "path-dependency-outside-root-refused")
    write(root, "crates/demo/Cargo.toml", DEMO_MANIFEST + '[dev-dependencies]\naway = { path = "../../../away" }\n')
    closure, error = closure_or_error(root)
    check("path-dependency-outside-root-refused", isinstance(error, ClosureRefused) and "away" in str(error) and "仓根" in str(error),
          f"path 依赖指到仓根之外应当拒、点名它，实际 {closure.entries if closure else error}")

    # include-literals-resolved-by-file：include_str! 的串按源文件目录解、concat!(env!("CARGO_MANIFEST_DIR"), …) 按包目录解（参数跨行）、
    # #[path] 按源文件目录解，concat!(env!("OUT_DIR"), …) 不算 ⇒ 三份仓根文件进闭包、为什么写「编译期读」
    root = make_root(base, "include-literals-resolved-by-file")
    write(root, "crates/demo/src/deep/reads.rs",
          'pub const THREADS: &str = include_str!("../../../../.claude/heavy-thread-default");\n'
          'pub const BLOB: &[u8] = include_bytes!(\n    concat!(env!("CARGO_MANIFEST_DIR"), "/../../data/blob.bin"),\n);\n'
          'include!(concat!(env!("OUT_DIR"), "/generated.rs"));\n#[path = "../../../../shared/module.rs"]\nmod borrowed;\n')
    for relative in (".claude/heavy-thread-default", "data/blob.bin", "shared/module.rs"):
        write(root, relative, "7\n")
    closure, error = closure_or_error(root)
    wanted = [".claude/heavy-thread-default", "data/blob.bin", "shared/module.rs"]
    check("include-literals-resolved-by-file", closure is not None and all(entry in closure.entries for entry in wanted)
          and all(closure.reasons[entry][0].startswith("编译期读（crates/demo/src/deep/reads.rs:") for entry in wanted if entry in closure.reasons),
          f"闭包里应当有 {wanted}、为什么写编译期读与出处行，实际 {closure.reasons if closure else error}")

    # unrecognized-include-refused：include_str!(env!(…)) 认不出、落到仓根之外、指到不在的文件 ⇒ 抛 ClosureRefused，三处逐行点名
    root = make_root(base, "unrecognized-include-refused")
    write(root, "crates/demo/src/reads.rs", 'pub const PIECE: &str = include_str!(env!("PIECE_PATH"));\n'
          'pub const FAR: &str = include_str!("../../../../outside.txt");\npub const GONE: &str = include_str!("../../../absent.txt");\n')
    closure, error = closure_or_error(root)
    text = str(error)
    check("unrecognized-include-refused", isinstance(error, ClosureRefused) and "crates/demo/src/reads.rs:1：include_str! 的参数不是一个串字面量" in text
          and "crates/demo/src/reads.rs:2：编译期读的" in text and "落到仓根之外" in text and "crates/demo/src/reads.rs:3：编译期读的 absent.txt 在仓里没有" in text,
          f"三处认不出、落到仓根之外、不在的编译期读应当逐行点名拒，实际 {closure.entries if closure else error}")

    # runtime-literals-resolved-by-crate：.join("../../…")、format!("{名}/../../…")、concat!(env!(…), "/../../…") 按包目录解 ⇒ 三份脚本进闭包、记成直接起的；
    # 注释里的不算；指到不在的记进 missing、不进闭包
    root = make_root(base, "runtime-literals-resolved-by-crate")
    write(root, "crates/demo/src/run.rs",
          'pub fn script() -> std::path::PathBuf {\n    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/check.sh")\n}\n'
          'pub fn tool() -> String {\n    let manifest_directory = env!("CARGO_MANIFEST_DIR");\n    format!("{manifest_directory}/../../tools/run.py")\n}\n'
          'pub const OTHER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/other.sh");\n// "../../scripts/in-comment.sh"\n'
          'pub const GONE: &str = "../../scripts/absent.sh";\n')
    for relative in ("scripts/check.sh", "tools/run.py", "tools/other.sh", "scripts/in-comment.sh"):
        write(root, relative, "#!/usr/bin/env bash\necho probe\n")
    closure, error = closure_or_error(root)
    wanted = ("scripts/check.sh", "tools/other.sh", "tools/run.py")
    check("runtime-literals-resolved-by-crate", closure is not None and all(entry in closure.entries for entry in wanted)
          and "scripts/in-comment.sh" not in closure.entries and closure.started == wanted and "scripts/absent.sh" in closure.missing
          and "scripts/absent.sh" not in closure.entries,
          f"三份脚本应当进闭包且记成直接起的、注释里的不进、不在的记进 missing，实际 {(closure.entries, closure.started, closure.missing) if closure else error}")

    # join-chains-resolved：仓根下的 served/（path 依赖）里 .join("..").join("<路径>") 跨行写、.join("..").join("scripts").join("deep.sh") ⇒ 两份进闭包
    root = make_root(base, "join-chains-resolved")
    write(root, "crates/demo/Cargo.toml", DEMO_MANIFEST + '[dependencies]\nserved = { path = "../../served" }\n')
    write(root, "served/Cargo.toml", '[package]\nname = "served"\n')
    write(root, "served/src/lib.rs",
          'pub fn tool(manifest_directory: &std::path::Path) -> std::path::PathBuf {\n    manifest_directory\n        .join("..")\n        .join(".claude/tool.py")\n}\n'
          'pub fn deep() -> std::path::PathBuf {\n    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("scripts").join("deep.sh")\n}\n')
    write(root, ".claude/tool.py", "print(1)\n")
    write(root, "scripts/deep.sh", "echo deep\n")
    closure, error = closure_or_error(root)
    check("join-chains-resolved", closure is not None and ".claude/tool.py" in closure.entries and "scripts/deep.sh" in closure.entries,
          f".join(\"..\") 起头的两串应当把 .claude/tool.py、scripts/deep.sh 带进闭包，实际 {closure.entries if closure else error}")

    # test-code-left-out：#[cfg(test)] 模块、#[test] 函数、tests/ 下的文件里指到的不进默认闭包，#[cfg(not(test))] 的进；带 test_inputs 四份都进
    root = make_root(base, "test-code-left-out")
    write(root, "crates/demo/src/lib.rs",
          'pub fn value() -> u32 {\n    1\n}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn reads() {\n        let _ = "../../scripts/test-module.sh";\n    }\n}\n'
          '#[test]\nfn outside_module() {\n    let _ = "../../scripts/test-function.sh";\n}\n#[cfg(not(test))]\npub const KEPT: &str = "../../scripts/not-test.sh";\n')
    write(root, "crates/demo/tests/reads.rs", 'const PATH: &str = "../../scripts/test-directory.sh";\n')
    test_only = ("scripts/test-directory.sh", "scripts/test-function.sh", "scripts/test-module.sh")
    for relative in (*test_only, "scripts/not-test.sh"):
        write(root, relative, "echo probe\n")
    plain, error = closure_or_error(root)
    with_tests, tests_error = closure_or_error(root, test_inputs=True)
    check("test-code-left-out", plain is not None and with_tests is not None and "scripts/not-test.sh" in plain.entries
          and not any(entry in plain.entries for entry in test_only) and all(entry in with_tests.entries for entry in (*test_only, "scripts/not-test.sh")),
          f"默认闭包只该有 scripts/not-test.sh、带 test_inputs 四份都在，实际默认 {plain.entries if plain else error}、带测试 {with_tests.entries if with_tests else tests_error}")

    # shell-sources-followed：Rust 起的 scripts/check.sh source ../lib/common.sh，后者 source "$here/deeper.sh" ⇒ 两份进闭包；注释掉的那一行、不在的文件不进
    root = make_root(base, "shell-sources-followed")
    write(root, "crates/demo/src/run.rs", 'pub const SCRIPT: &str = "../../scripts/check.sh";\n')
    write(root, "scripts/check.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/../lib/common.sh"\n'
          '# source "$(dirname "${BASH_SOURCE[0]}")/../lib/commented.sh"\nbash "$(dirname "${BASH_SOURCE[0]}")/absent.sh"\n')
    write(root, "lib/common.sh", 'here="${BASH_SOURCE[0]%/*}"\nsource "$here/deeper.sh"\n')
    write(root, "lib/deeper.sh", "deeper() { :; }\n")
    write(root, "lib/commented.sh", "commented() { :; }\n")
    closure, error = closure_or_error(root)
    check("shell-sources-followed", closure is not None and "lib/common.sh" in closure.entries and "lib/deeper.sh" in closure.entries
          and "lib/commented.sh" not in closure.entries,
          f"source 链上的 lib/common.sh、lib/deeper.sh 应当进闭包、注释掉的不进，实际 {closure.entries if closure else error}")

    # covered-scripts-followed：Rust 先指到 tools/ 整个目录、后指到它里面的 tools/run/launcher.sh（罩在目录条目里、不单独成条，Rust 直接起它）；
    # launcher.sh 起同目录的 inner.sh，inner.sh 起另一个顶层目录下的 hooks/lib_root.py ⇒ hooks/lib_root.py 进闭包、为什么写 inner.sh 那一行，同目录的 unused.py 不进
    # closure-independent-of-reference-order：同一棵仓把两处引用的先后倒过来（先指到脚本、后指到目录）再算 ⇒ 闭包逐条相同
    tools_line = 'pub const TOOLS: &str = "../../tools";\n'
    launcher_line = 'pub const LAUNCHER: &str = "../../tools/run/launcher.sh";\n'
    order_closures = {}
    for order, source in (("directory-first", tools_line + launcher_line), ("script-first", launcher_line + tools_line)):
        root = make_root(base, f"covered-scripts-{order}")
        write(root, "crates/demo/src/run.rs", source)
        write(root, "tools/run/launcher.sh", '#!/usr/bin/env bash\nbash "$(dirname "${BASH_SOURCE[0]}")/inner.sh"\n')
        write(root, "tools/run/inner.sh", 'python3 "$(dirname "${BASH_SOURCE[0]}")/../../hooks/lib_root.py"\n')
        write(root, "hooks/lib_root.py", "print('root')\n")
        write(root, "hooks/unused.py", "print('unused')\n")
        order_closures[order] = closure_or_error(root)
    closure, error = order_closures["directory-first"]
    check("covered-scripts-followed", closure is not None and "hooks/lib_root.py" in closure.entries and "hooks/unused.py" not in closure.entries
          and closure.reasons.get("hooks/lib_root.py", ("",))[0].startswith("被 source、被起（tools/run/inner.sh:1）")
          and "tools" in closure.entries and "tools/run/launcher.sh" in closure.started,
          f"罩在 tools/ 里的 launcher.sh 起 inner.sh、inner.sh 起 hooks/lib_root.py，hooks/lib_root.py 应当进闭包、为什么写 inner.sh 那一行，"
          f"实际 {(closure.reasons, closure.started) if closure else error}")
    reversed_closure, reversed_error = order_closures["script-first"]
    check("closure-independent-of-reference-order", closure is not None and reversed_closure is not None and closure.entries == reversed_closure.entries
          and closure.reasons.get("hooks/lib_root.py") == reversed_closure.reasons.get("hooks/lib_root.py"),
          f"两处引用的先后倒过来闭包应当逐条相同，实际先指到目录得 {closure.entries if closure else error}、"
          f"先指到脚本得 {reversed_closure.entries if reversed_closure else reversed_error}")

    # python-references-followed：Rust 起的 tools/run.py 同目录 import helper、sys.path 加 ../shared 再 import、引号里的 'proc.py'、os.path.join(…, "side.sh")
    # ⇒ 四份进闭包，同目录没被点名的 unused.py 不进
    root = make_root(base, "python-references-followed")
    write(root, "crates/demo/src/run.rs", 'pub const TOOL: &str = "../../tools/run.py";\n')
    write(root, "tools/run.py", "import os, sys\nsys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), '..', 'shared'))\n"
          "import helper\nfrom shared_module import thing\nPROCESS = 'proc.py'\nOTHER = os.path.join(os.path.dirname(__file__), '..', 'scripts', 'side.sh')\n")
    for relative in ("tools/helper.py", "shared/shared_module.py", "tools/proc.py", "scripts/side.sh", "tools/unused.py"):
        write(root, relative, "thing = 1\n")
    closure, error = closure_or_error(root)
    wanted = ("scripts/side.sh", "shared/shared_module.py", "tools/helper.py", "tools/proc.py")
    check("python-references-followed", closure is not None and all(entry in closure.entries for entry in wanted) and "tools/unused.py" not in closure.entries,
          f"闭包里应当有 {wanted}、没有 tools/unused.py，实际 {closure.entries if closure else error}")

    # python-directory-variables-followed：声明要起的 loader/finder.py 把脚本所在目录赋给 HERE、再拼出 SHARED = os.path.join(HERE, '..', 'canonical', 'scripts')，
    # 之后 os.path.join(SHARED, 'checker.py')、sys.path.insert(0, SHARED) 再 import helper_module（project_preflight.py 找规范副本 preflight.py 的写法）
    # ⇒ canonical/scripts/ 下这两份进闭包；同目录没被点名的 unused.py 不进
    root = make_root(base, "python-directory-variables-followed")
    write(root, "loader/finder.py", "import os\nimport sys\nHERE = os.path.dirname(os.path.realpath(__file__))\n\n\ndef locate():\n"
          "    shared = os.path.join(HERE, '..', 'canonical', 'scripts')\n    if os.path.isfile(os.path.join(shared, 'checker.py')):\n"
          "        sys.path.insert(0, shared)\n        import helper_module\n        return helper_module\n    return None\n")
    for relative in ("canonical/scripts/checker.py", "canonical/scripts/helper_module.py", "canonical/scripts/unused.py"):
        write(root, relative, "value = 1\n")
    closure, error = closure_or_error(root, [("loader/finder.py", "副本里要起这份脚本")])
    wanted = ("canonical/scripts/checker.py", "canonical/scripts/helper_module.py")
    check("python-directory-variables-followed", closure is not None and all(entry in closure.entries for entry in wanted)
          and "canonical/scripts/unused.py" not in closure.entries,
          f"闭包里应当有 {wanted}、没有 canonical/scripts/unused.py，实际 {closure.entries if closure else error}")

    # configuration-never-copied：代码里指到仓根的 multi-host.env ⇒ 不进闭包、记进 left_out
    root = make_root(base, "configuration-never-copied")
    write(root, "crates/demo/src/configuration.rs", 'pub const CONFIGURATION: &str = "../../multi-host.env";\n')
    write(root, CONFIGURATION_FILE_NAME, "HOSTS=selftest\n")
    closure, error = closure_or_error(root)
    check("configuration-never-copied", closure is not None and CONFIGURATION_FILE_NAME not in closure.entries and CONFIGURATION_FILE_NAME in closure.left_out,
          f"multi-host.env 不该进闭包、该记进 left_out，实际 {(closure.entries, closure.left_out) if closure else error}")

    # declared-entries-need-a-reason：声明的条目不写为什么 ⇒ ValueError；写了 ⇒ 进闭包、为什么带上；声明的目录里的 path 依赖照跟
    root = make_root(base, "declared-entries-need-a-reason")
    write(root, "litmus/a.litmus", "C a\n")
    write(root, "tools/pkg/Cargo.toml", '[package]\nname = "pkg"\n[dependencies]\nextra = { path = "../../extra" }\n')
    write(root, "extra/Cargo.toml", '[package]\nname = "extra"\n')
    _, unreasoned = closure_or_error(root, [("litmus", "")])
    closure, error = closure_or_error(root, [("litmus", "变异行锚在这里的文件上"), ("tools/pkg", "副本里要编这个包")])
    check("declared-entries-need-a-reason", isinstance(unreasoned, ValueError) and closure is not None and "litmus" in closure.entries
          and closure.reasons.get("litmus", ("",))[0] == "调用方声明：变异行锚在这里的文件上" and "extra" in closure.entries,
          f"不写为什么应当拒、写了进闭包并跟声明目录的 path 依赖，实际不写为什么得 {unreasoned!r}、写了得 {closure.reasons if closure else error}")

    # copy-environment-names-the-main-configuration：git worktree 里取主仓那一份的绝对路径；调用方设了的照它；不在 git 里取源树自己
    main = os.path.join(base, "copy-environment", "main")
    worktree = os.path.join(base, "copy-environment", "worktree")
    plain_tree = os.path.join(base, "copy-environment", "plain")
    os.makedirs(main)
    os.makedirs(plain_tree)
    git_ok = all(subprocess.run([*SELFTEST_GIT, *arguments], cwd=main, capture_output=True, stdin=subprocess.DEVNULL).returncode == 0
                 for arguments in (("init", "-q"), ("commit", "-q", "--allow-empty", "-m", "selftest"), ("worktree", "add", "-q", "--detach", worktree)))
    from_worktree = environment_for_commands_in_the_copy(worktree, {}).get(CONFIGURATION_VARIABLE)
    given = environment_for_commands_in_the_copy(worktree, {CONFIGURATION_VARIABLE: "/given/multi-host.env"}).get(CONFIGURATION_VARIABLE)
    from_plain = environment_for_commands_in_the_copy(plain_tree, {}).get(CONFIGURATION_VARIABLE)
    plain_wanted = (os.path.join(os.path.realpath(plain_tree), CONFIGURATION_FILE_NAME) if not inside_git_work_tree(plain_tree)
                    else from_plain)   # 临时目录本身落在某棵 git 工作树里时这一比判不了，只比前两样
    check("copy-environment-names-the-main-configuration", git_ok and from_worktree == os.path.join(os.path.realpath(main), CONFIGURATION_FILE_NAME)
          and given == "/given/multi-host.env" and from_plain == plain_wanted,
          f"worktree 里应当取主仓的 {os.path.join(main, CONFIGURATION_FILE_NAME)}、设了的照原样、不在 git 里取源树自己，实际 git 建仓 {git_ok}、"
          f"worktree 得 {from_worktree}、设了的得 {given}、不在 git 里得 {from_plain}")

    # copies-the-closure-with-parent-directories：单个文件的条目（.claude/heavy-thread-default）连上级目录拷进不在 git 里的目的地，
    # crates/ 里的编译目录不拷，拷过去的修改时刻是此刻
    root = make_root(base, "copies-the-closure-with-parent-directories")
    write(root, "crates/demo/src/reads.rs", 'pub const THREADS: &str = include_str!("../../../.claude/heavy-thread-default");\n')
    write(root, ".claude/heavy-thread-default", "7\n")
    write(root, ".claude/agents/other.md", "agent\n")
    destination = os.path.join(base, "copies-the-closure-with-parent-directories", "copy")
    closure, error = closure_or_error(root)
    copied, copy_error = [], None
    if closure is not None:
        try:
            copy_entries(root, destination, closure.entries)
            copied = tree_files(destination)
        except OSError as raised:
            copy_error = raised
    wanted = [".cargo/config.toml", ".claude/heavy-thread-default", "Cargo.lock", "Cargo.toml", "crates/demo/Cargo.toml", "crates/demo/src/lib.rs",
              "crates/demo/src/reads.rs"]
    fresh = copied == wanted and time.time() - os.stat(os.path.join(destination, ".claude/heavy-thread-default")).st_mtime < RECENT_SECONDS
    check("copies-the-closure-with-parent-directories", fresh, f"副本里应当正好是 {wanted}、修改时刻是此刻，实际 {copied or copy_error or error}")

    # cross-check-sees-file-relative-includes：下一格那一比（独立粗扫对闭包的交叉核）放在假仓上：include_str! 的串按源文件目录解到仓根下的
    # .claude/registered-file（按包目录解会落到仓根之外）⇒ 粗扫认到它、它在闭包里、交叉核不报缺。本仓编译期读仓根文件的写法都是
    # concat!(env!("CARGO_MANIFEST_DIR"), "/", "../../…")，按包目录解与按源文件目录解落在同一处，闭包不认 include 也照样收进来，这一格是 include 那一类的取样点
    root = make_root(base, "cross-check-sees-file-relative-includes")
    write(root, "crates/demo/src/deep/reads.rs", 'pub const REGISTERED: &str = include_str!("../../../../.claude/registered-file");\n')
    write(root, ".claude/registered-file", "7\n")
    closure, error, coarse, uncovered, absent = closure_against_the_coarse_scan(root)
    check("cross-check-sees-file-relative-includes", closure is not None and ".claude/registered-file" in coarse and not uncovered and not absent,
          f"粗扫应当认到按源文件目录解的 .claude/registered-file、它在闭包里，实际粗扫 {coarse}，不在闭包里的 {uncovered or '没有'}，"
          f"闭包里不在仓里的 {absent or '没有'}：{error or ''}")

    # repository-root-references-in-the-closure（本仓）：独立的粗扫指到的仓根条目（非测试代码与 build.rs 的字面路径、path 依赖、shell 的 source 链）
    # 都落在闭包里或决不拷的那几样里；闭包里的每一条在仓里都在
    closure, error, coarse, uncovered, absent = closure_against_the_coarse_scan(REPOSITORY_ROOT)
    check("repository-root-references-in-the-closure", closure is not None and not uncovered and not absent and coarse,
          f"本仓粗扫到 {len(coarse)} 处，不在闭包里的 {uncovered or '没有'}，闭包里不在仓里的 {absent or '没有'}：{error or ''}")

    # runtime-probes-cover-every-started-script（本仓）：Rust 非测试代码直接起的每一份脚本在 RUNTIME_PROBES 里都有一条（真编核对照它在副本里起）
    unprobed = [script for script in (closure.started if closure else ()) if script not in probes_in_force()]
    check("runtime-probes-cover-every-started-script", closure is not None and closure.started and not unprobed,
          f"直接起的 {closure.started if closure else error}，没登记起法的 {unprobed or '没有'}")

    # verify-copy-builds-and-starts-the-probes-in-the-copy：假 cargo 在副本里起（当前目录是副本、副本里有包的源码）、退 0；
    # 登记的运行期脚本在副本里起、拿到的 SINGLEFS_MULTI_HOST_CONFIG 是主工作区那一份；探针退 1 时判不过；副本收尾删掉
    root = make_root(base, "verify-copy")
    write(root, "crates/demo/src/run.rs", 'pub const PROBE: &str = "../../scripts/probe.sh";\n')
    write(root, "scripts/probe.sh", '#!/usr/bin/env bash\necho "PROBED=1"\nprintf "%s\\n" "$PWD|${SINGLEFS_MULTI_HOST_CONFIG:-unset}" >"$PROBE_LOG"\n'
          'exit "${PROBE_EXIT:-0}"\n')
    fake_cargo = write(base, "verify-copy/bin/cargo", '#!/usr/bin/env bash\nprintf "%s\\n" "$PWD" >"$CARGO_LOG"\n[[ -f crates/demo/src/lib.rs ]] || exit 101\n')
    os.chmod(fake_cargo, 0o755)
    work = os.path.join(base, "verify-copy", "work")
    probe_log, cargo_log = os.path.join(base, "verify-copy", "probe.log"), os.path.join(base, "verify-copy", "cargo.log")
    probes = {"scripts/probe.sh": ("bash", "{tree}/scripts/probe.sh")}
    saved = {name: os.environ.get(name) for name in ("PROBE_LOG", "CARGO_LOG", "PROBE_EXIT", CONFIGURATION_VARIABLE)}
    os.environ.update(PROBE_LOG=probe_log, CARGO_LOG=cargo_log)
    os.environ.pop(CONFIGURATION_VARIABLE, None)
    output = io.StringIO()
    green, green_checked, red, probed, built_in = ["没跑完"], 0, [], "", ""
    try:
        green, green_checked = verify_copy(root, work, os.path.join(work, "target"), ["--offline"], probes=probes, cargo=fake_cargo,
                                           emit=lambda line: output.write(line + "\n"))
        probed = open(probe_log, encoding="utf-8").read().strip() if os.path.isfile(probe_log) else ""
        built_in = open(cargo_log, encoding="utf-8").read().strip() if os.path.isfile(cargo_log) else ""
        os.environ["PROBE_EXIT"] = "1"
        red, _ = verify_copy(root, work, os.path.join(work, "target"), ["--offline"], probes=probes, cargo=fake_cargo, emit=lambda line: None)
    except (OSError, ValueError) as error:   # 拷不成、闭包算不成（弄坏开关下）：这一格记红，别的格照跑
        green = [f"抛了 {type(error).__name__}：{error}"]
    finally:
        for name, value in saved.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
    tree = os.path.join(work, "tree")
    probed_directory, _, probed_configuration = probed.partition("|")
    check("verify-copy-builds-and-starts-the-probes-in-the-copy",
          not green and green_checked == 2 and os.path.realpath(built_in or "/") == os.path.realpath(tree)
          and os.path.realpath(probed_directory or "/") == os.path.realpath(tree)
          and probed_configuration == configuration_path(root, {})
          and any("scripts/probe.sh" in problem for problem in red) and not os.path.lexists(tree) and "keys=PROBED" in output.getvalue(),
          f"假 cargo 应当在副本 {tree} 里起、探针在副本里起并拿到主工作区的配置路径、探针退 1 判不过、副本收尾删掉，实际问题 {green}、判了 {green_checked} 格、"
          f"cargo 在 {built_in or '没起'}、探针记 {probed or '没起'}、探针退 1 时问题 {red}、副本{'还在' if os.path.lexists(tree) else '删了'}")
    # copy-completeness-names-missing-entries：闭包里有声明的 litmus/（副本里要读的数据），一份副本拷全、一份缺 litmus/ ⇒
    # entries_missing_in_copy 对拷全的交空、查 5 条，对缺的只点名 litmus；missing-in-copy 子命令对缺的退 1、打 missing_in_copy litmus 那一行、出路指到 minimal-copy.py，
    # 对拷全的退 0、报闭包条数。两份副本照条目自己拷（不经 copy_entries：这一格只判核对，不受拷法那几条开关牵连）
    root = make_root(base, "copy-completeness")
    write(root, "litmus/never.litmus", "C never\n")
    completeness_reason = "自证：副本里要读的 litmus 样本"
    complete_copy = os.path.join(base, "copy-completeness", "complete")
    short_copy = os.path.join(base, "copy-completeness", "short")
    for copy_root, copied in ((complete_copy, (".cargo", "Cargo.lock", "Cargo.toml", "crates", "litmus")),
                              (short_copy, (".cargo", "Cargo.lock", "Cargo.toml", "crates"))):
        os.makedirs(copy_root)
        for entry in copied:
            source = os.path.join(root, entry)
            if os.path.isdir(source):
                shutil.copytree(source, os.path.join(copy_root, entry))
            else:
                shutil.copyfile(source, os.path.join(copy_root, entry))
    complete_missing, short_missing, complete_checked, completeness_error = None, None, 0, ""
    try:
        complete_missing, complete_checked = entries_missing_in_copy(root, complete_copy, [("litmus", completeness_reason)])
        short_missing, _ = entries_missing_in_copy(root, short_copy, [("litmus", completeness_reason)])
    except (ClosureRefused, ValueError) as error:
        completeness_error = f"抛了 {type(error).__name__}：{error}"
    completeness_runs = {}
    for name, copy_root in (("complete", complete_copy), ("short", short_copy)):
        standard_output, standard_error = io.StringIO(), io.StringIO()
        try:
            with contextlib.redirect_stdout(standard_output), contextlib.redirect_stderr(standard_error):
                exit_code = command_missing_in_copy([copy_root, "--root", root, "--with", f"litmus={completeness_reason}"])
        except (ClosureRefused, ValueError) as error:
            exit_code = f"抛了 {type(error).__name__}：{error}"
        completeness_runs[name] = (exit_code, standard_output.getvalue(), standard_error.getvalue())
    short_code, short_output, short_error = completeness_runs["short"]
    complete_code, complete_output, _ = completeness_runs["complete"]
    check("copy-completeness-names-missing-entries",
          not completeness_error and complete_missing == [] and complete_checked == 5 and [entry for entry, _ in short_missing or []] == ["litmus"]
          and short_code == EXIT_REFUSED and "REPOSITORY_COPY missing_in_copy litmus\t" in short_output and "minimal-copy.py" in short_error
          and complete_code == 0 and "REPOSITORY_COPY copy-complete entries=5" in complete_output,
          f"拷全的应当一条都不缺、闭包 5 条，缺 litmus/ 的应当只点名 litmus、子命令退 1 并给 minimal-copy.py 的出路；实际函数对拷全的交 {complete_missing}（闭包 {complete_checked} 条）、"
          f"对缺的交 {short_missing}{completeness_error}，子命令对缺的退 {short_code}、输出 {short_output.strip()!r} {short_error.strip()!r}，"
          f"对拷全的退 {complete_code}、输出 {complete_output.strip()!r}")

    # declared-directory-scripts-followed：声明了 tools/ 整个目录（副本带 --with-gate-tables 而缺 .claude/scripts/preflight.sh）：
    # tools/gate/stage.sh 照自己所在目录 source ../../lib/pre.sh，pre.sh 再起 ../vendor/inner.sh ⇒ 两份进闭包、为什么以 DECLARED_SCRIPTS_REASON 起头；
    # tools/fixtures/ 下的样本引到不在的文件、printf 里的样本行引到不在的文件，都不算缺；没被引到的 lib/unused.sh 不进
    root = make_root(base, "declared-directory-scripts-followed")
    write(root, "tools/gate/stage.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/../../lib/pre.sh"\n'
          "printf '%s\\n' 'source \"$(dirname \"${BASH_SOURCE[0]}\")/sample-absent.sh\"' > /dev/null\n")
    write(root, "tools/fixtures/sample/run.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/absent-in-fixture.sh"\n')
    write(root, "lib/pre.sh", 'bash "${BASH_SOURCE[0]%/*}/../vendor/inner.sh"\n')
    write(root, "vendor/inner.sh", "echo inner\n")
    write(root, "lib/unused.sh", "echo unused\n")
    closure, error = closure_or_error(root, [("tools", "副本里起它下面的门禁")])
    check("declared-directory-scripts-followed", closure is not None and "lib/pre.sh" in closure.entries and "vendor/inner.sh" in closure.entries
          and "lib/unused.sh" not in closure.entries and closure.reasons.get("lib/pre.sh", ("",))[0].startswith(DECLARED_SCRIPTS_REASON),
          f"声明的 tools/ 里的 stage.sh source 的 lib/pre.sh 与它起的 vendor/inner.sh 应当进闭包、为什么以「{DECLARED_SCRIPTS_REASON}」起头，lib/unused.sh 不进，"
          f"实际 {(closure.entries, closure.reasons.get('lib/pre.sh')) if closure else error}")

    # declared-closure-gap-refused：声明的 tools/ 里的 stage.sh 照自己所在目录 source 的 ../lib/absent.sh 在源树里不在 ⇒ 抛 ClosureRefused、点名那一行与缺的那一份
    root = make_root(base, "declared-closure-gap-refused")
    write(root, "tools/stage.sh", '#!/usr/bin/env bash\nsource "$(dirname "${BASH_SOURCE[0]}")/../lib/absent.sh"\n')
    closure, error = closure_or_error(root, [("tools", "副本里起它下面的门禁")])
    check("declared-closure-gap-refused", isinstance(error, ClosureRefused) and "tools/stage.sh:2" in str(error) and "lib/absent.sh" in str(error),
          f"声明的条目里的脚本 source 的文件在源树里不在应当拒、点名 tools/stage.sh:2 与 lib/absent.sh，实际 {closure.entries if closure else error}")
    return failures, checked


SWITCH_EXPECTED_CELLS = {   # 文件头「弄坏开关」那一段，两处要一致
    "path-dependencies-ignored": ["path-dependencies-followed", "path-dependency-outside-root-refused", "join-chains-resolved",
                                  "declared-entries-need-a-reason", "repository-root-references-in-the-closure"],
    "outside-root-accepted": ["path-dependency-outside-root-refused"],
    "includes-ignored": ["include-literals-resolved-by-file", "unrecognized-include-refused", "copies-the-closure-with-parent-directories",
                         "cross-check-sees-file-relative-includes"],
    "runtime-literals-ignored": ["runtime-literals-resolved-by-crate", "test-code-left-out", "shell-sources-followed", "python-references-followed",
                                 "configuration-never-copied", "repository-root-references-in-the-closure", "verify-copy-builds-and-starts-the-probes-in-the-copy",
                                 "covered-scripts-followed"],
    "join-chains-ignored": ["join-chains-resolved"],
    "test-code-scanned": ["test-code-left-out", "runtime-probes-cover-every-started-script"],
    "test-inputs-ignored": ["test-code-left-out"],
    "shell-sources-not-followed": ["shell-sources-followed", "repository-root-references-in-the-closure", "covered-scripts-followed"],
    "python-references-not-followed": ["python-references-followed", "python-directory-variables-followed"],
    "python-directory-variables-ignored": ["python-directory-variables-followed"],
    "never-copied-copied": ["configuration-never-copied"],
    "declared-reason-unchecked": ["declared-entries-need-a-reason"],
    "environment-configuration-unset": ["copy-environment-names-the-main-configuration", "verify-copy-builds-and-starts-the-probes-in-the-copy"],
    "environment-configuration-by-tree": ["copy-environment-names-the-main-configuration"],
    "copy-without-parent-directories": ["copies-the-closure-with-parent-directories", "verify-copy-builds-and-starts-the-probes-in-the-copy"],
    "probe-dropped": ["runtime-probes-cover-every-started-script"],
    "verify-probes-skipped": ["verify-copy-builds-and-starts-the-probes-in-the-copy"],
    "copy-completeness-unchecked": ["copy-completeness-names-missing-entries"],
    "covered-scripts-not-followed": ["covered-scripts-followed", "closure-independent-of-reference-order"],
    "declared-directories-not-followed": ["declared-directory-scripts-followed", "declared-closure-gap-refused"],
    "declared-closure-gaps-accepted": ["declared-closure-gap-refused"],
}


def run_cells_in_scratch():
    base = tempfile.mkdtemp(prefix="lib-repository-copy-selftest-")
    SELFTEST_RUNNING.append(True)
    try:
        return selftest_cells(base)
    finally:
        SELFTEST_RUNNING.pop()
        shutil.rmtree(base, ignore_errors=True)


def selftest():
    failures, checked = run_cells_in_scratch()
    if failures:
        # 汇总行不带格名：格名已在上面逐格的「✗ 自证 <格名>：」里，汇总行各开关趟下才是同一句，弄坏开关复跑才认它是汇总
        print(f"  ✗ lib_repository_copy.py 自证 {len(failures)} 格不对（查了 {checked} 格）")
        print(f"  → 不对的格：{'、'.join(failures)}")
        print(f"  → 怎么办：照上面逐格改 copy_closure / copy_entries / environment_for_commands_in_the_copy / verify_copy；{BREAK_VARIABLE} 设着的话本来就该红")
        return 1
    # 开关轮的并发：每台默认重活线程数（scripts/hosts/heavy_threads.py，登记位 .claude/heavy-thread-default），再按开关条数夹；取不了就拒，不退回按核数算
    sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
    import heavy_threads
    try:
        switch_round_threads = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 弄坏开关轮取不了每台默认重活线程数：{error}")
        print(f"  → 怎么办：{error.howto}；不退回按核数算")
        return 2
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), switch_round_threads))

    def one(switch):
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(os.environ, **{BREAK_VARIABLE: switch}), timeout=600, stdin=subprocess.DEVNULL)
        return switch, set(line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 "))

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(one, SWITCH_EXPECTED_CELLS))
    switch_failures = [] if len(results) == len(SWITCH_EXPECTED_CELLS) else [f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(results)} 个"]
    for switch, red in results:
        print(f"  · 弄坏开关 {BREAK_VARIABLE}={switch}：红在 {'、'.join(sorted(red)) or '一格都没有'}")
        missing = [cell for cell in SWITCH_EXPECTED_CELLS[switch] if cell not in red]
        if missing:
            switch_failures.append(f"{switch} 没红在 {'、'.join(missing)}（红的是 {'、'.join(sorted(red)) or '一格都没有'}）")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它在代码里弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头的表")
        return 1
    print(f"  ✓ lib_repository_copy.py 自证通过（查了 {checked} 格：path 依赖逐层跟进而指到仓根之外的拒、编译期 include 按源文件或包目录解而认不出的拒、"
          "运行期 ../ 字面量与 .join(\"..\") 串按包目录解、测试代码默认不扫而带 test_inputs 扫、shell 的 source 链与 python 的 import 传递地跟、"
          "罩在目录条目里的脚本照追 source 链而与引用的先后无关、声明的目录里的脚本照追 source 链而照脚本所在目录引到的文件不在就拒、"
          "python 里赋给名字的目录拼出的脚本与搜索目录照认、"
          "多机配置不进副本、声明要写为什么、副本环境取主工作区那一份配置、单个文件连上级目录拷、粗扫认得按源文件目录解的 include、本仓粗扫指到的都在闭包里、直接起的脚本都有起法、"
          "真编核对在副本里编与起探针、副本缺闭包里的条目逐条点名而拷全的不报；"
          f"弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


def main(arguments):
    if requested_breaks():
        print(f"  ✗ 弄坏开关 {BREAK_VARIABLE} 设在 --selftest 之外，什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；它只给 lib_repository_copy.py --selftest 证红用", file=sys.stderr)
        return EXIT_USAGE
    commands = {"entries": command_entries, "link-into": command_link_into, "missing-in-copy": command_missing_in_copy, "verify-copy": command_verify_copy}
    if arguments[:1] == ["configuration-path"] and len(arguments) == 2:
        print(configuration_path(arguments[1]))
        return 0
    if not arguments or arguments[0] not in commands:
        print("  ✗ lib_repository_copy.py 用法错：子命令只认 entries、link-into、missing-in-copy、configuration-path、verify-copy、--selftest", file=sys.stderr)
        print("  → 怎么办：照文件头「用法」写", file=sys.stderr)
        return EXIT_USAGE
    try:
        return commands[arguments[0]](arguments[1:])
    except ClosureRefused as error:
        print(f"  ✗ 仓副本的闭包算不成：{error}", file=sys.stderr)
        print("  → 怎么办：path 依赖指回仓根里面、Cargo.toml 修好；编译期读的文件写成认得出的写法并让它进仓（文件头「闭包」）", file=sys.stderr)
        return EXIT_REFUSED
    except ValueError as error:
        print(f"  ✗ lib_repository_copy.py 用法错：{error}", file=sys.stderr)
        print("  → 怎么办：照文件头「用法」写", file=sys.stderr)
        return EXIT_USAGE


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    sys.exit(main(sys.argv[1:]))
