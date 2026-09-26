#!/usr/bin/env bash
# PreToolUse hook（Bash）：重型测试只在提交代码时、或用户要求时跑，子 agent 只跑自己那一份；不合的当场拒绝（退出 2，stderr 写原因与出路），同时记进检出记录交主 agent 看。
# hook-events: PreToolUse
# gate-similar: bash-command-detector.sh 同挂 PreToolUse[Bash]、判同一条命令，切词已抽成 lib_shell_words.py 两边共用；它判命令的形状（没超时的等待循环、后台里又放后台、起看门狗的写法），对谁都一样，起看门狗的错误写法与前台没超时的等待循环拒绝、其余只记不拦；这里判命令跑不跑重型测试，按 agent_type 与 SINGLEFS_HEAVY_TESTS 前缀放行、不合就拒，还把执行的脚本读进去逐层判，判定与看门狗共用 lib_heavy_tests.py
# gate-similar: pattern-process-guard.sh 同挂 PreToolUse[Bash]、也在执行前拒绝；它是上游 SOP 的钩子，判按模式找进程（pgrep / pkill 带 -f、killall），对谁都拒、没有放行条件，判据与 shell-lint 的 S2、S3 同一份；重型测试（层 0、QEMU、herd7）是本工程自己的装置，不归上游管，也不许在 .claude/singlefs-ai-sop/ 里就地改
# gate-similar: runner-dispatch-guard.sh 管同一条规矩（子 agent 不跑重型测试），但挂 PreToolUse[Agent|Task]、在派发那一刻判派发提示里的中文句子（只在「跑 / 复跑 / 执行 / 运行 / bash / 起」的宾语是重型阶段时才拒，否定、转述、引号里的都不判）；这里挂 PreToolUse[Bash]、在执行那一刻判真要跑的命令与它执行的脚本，一边判自然语言、一边判 shell 命令，判法没有能共用的
# gate-similar: write-guard.sh 同样按 agent_type 分谁能做什么，但挂 PreToolUse[Write|Edit]，判的是写哪个文件、写进什么字（整份覆盖未跟踪文件、写范围表、撇号类字符）；这里判 Bash 命令，放行表是各 agent 能跑哪几类重型测试，与写范围表没有共用的行
# gate-similar: continuation-guard.sh 挂 PreToolUse[SendMessage]，判的是收件的子 agent 在会话记录里交回过没有、被中断过没有；这里挂 PreToolUse[Bash]，判命令本身，两边的对象与输入没有交集
# gate-similar: kb-scribe-followup.sh 也看 Bash，但挂 PostToolUse、命令已经跑完，只管书记官写 kb 之后跑相关门禁阶段、只记不拦；这里在执行之前判、不合就拒，管的是每个 agent 跑的重型测试
# gate-overlap:copy-kept bash-command-detector.sh 两边 main() 开头那十行（认 --selftest、从 stdin 读 hook 的 JSON、不是对象就放行）是入口，读不到共用模块时照样要跑：那时还要拿这份 JSON 带着 session_id 记一条检出，看门狗按 session_id 认本会话；抽进 lib_shell_words.py 或另一个同目录模块，模块读不到的那一刻入口跟着一起没了（两个 hook 自检里「hook 旁边没有共用模块」那一例走的就是这条路）
#
# 为什么：用户 2026-09-24 定「subagent任务派发的门禁里面写上 禁止跑0层测试等这种重测试 就跑自己相关的测试就好了」，
# 同日再定「项目内 全量崩溃 qemu herd7 等重型的测试任务 禁止平时调用 仅仅在每次代码提交时候跑 或者要求时候再跑 门禁写清楚。 提交代码必须跑。 subagent 严禁跑」、
# 更正「崩溃验证员和门禁分诊员 跑各自的部分就好了。不需要跑全量」、补「实在是任务需要跑的时候 弹窗」。定义里写一句「不跑」守不住：
# 同一天几个实现员在各自的副本里跑全量 `cargo test --all` 与层 0 快档，满载下每遍四五个小时（records/2026-09-16-subagent拆分提案.md 第四十节那张表第 4、10、12、14 行）。
# 同日一个实现员把验证链写成 /tmp 下的脚本再用后台 Bash 起它，链里有一步属于「子 agent 一律拒」的一类，这道闸只看到「执行一个脚本」、放行了
# （同一张表第 15 行）：所以命令位置上执行的脚本文件要读进去判。
# 子 agent 自测时常在同一条命令里先用 heredoc 写出脚本、紧接着执行它，判的那一刻文件还不在，按「不存在」记检出，一小时里叫醒主 agent 六七次
# （同一张表第 18 行）：所以同一条命令里前面写出的脚本拿写出的内容判，不记「不存在」。
# 双引号里没转义的反引号把 `crates/…/walk.rs` 当命令替换执行，这个 `.rs` 被当 shell 脚本读进去、里面像路径的词再当脚本读，一路递归到第 6 层，
# 一条命令 100 秒没判完、拖慢每个 agent 的每条命令（同一张表第 18 行）：所以直接执行的只读 shell 脚本，同一份脚本一次判定里只读一遍、只判一遍。
#
# 重型测试，按类（只认命令位置）：
#   层 0          cargo test 会跑到名字含 layer0 的测试二进制：--test 的名字含 layer0、--test 的通配命中它、
#                 或不带目标选择（带 --tests / --all-targets 也算）而包里有这种二进制；.claude/gate.d/54-*；
#                 直接执行名字含 layer0 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）
#   QEMU          .claude/gate.d/55-*、qemu-system-*、research/scripts/vm-bench.sh（--selftest 也起虚机，照算）
#   herd7         .claude/gate.d/57-*、.claude/scripts/lkmm.sh、herd7
#   crates 变异整表 .claude/gate.d/59-*、research/scripts/mutate.sh 的参数里有 crates/mutations.tsv
#   全量测试      cargo test 带 --workspace / --all；在工作区根（清单有 [workspace] 没有 [package]：仓根与 research/ 都是）上
#                 不带 -p / --test / --lib / --bin 的 cargo test；不挑目标而包的范围是工作区全部成员的 cargo test
#                 （research/ 只有 e7-index-bench 一个成员，在它里面裸跑等于全量）；.claude/scripts/check.sh
#   全部实验复跑  .claude/gate.d/87-*
#   整轮门禁      gate.sh、research/scripts/gate-staged.sh（--selftest 不算）
#   E152 装置     e152-file-system-benchmark（直接起、或 cargo run 它）、research/scripts/e152-run.sh
#   .claude/gate.d/ 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑、不用带前缀。
# 谁、带什么才放行（都要带环境变量 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，别的值或没带一律拒）：
#   主 agent（输入里没有 agent_type）：上面每一类；
#   crash-verifier：层 0、55 号与 qemu-system-*、herd7、crates 变异整表（vm-bench.sh、全量测试、整轮门禁、E152 拒）；
#   gate-triage：整轮门禁（gate.sh、gate-staged.sh）与 87 号（gate.sh 里 54 / 55 / 57 / 59 靠「输入没变就复用上一次全绿判定」，直接调它们拒）；
#   其余子 agent：一律拒，带不带前缀都拒。
# 前缀认三种写法：写在命令前（`SINGLEFS_HEAVY_TESTS=commit bash …`）、写进 `env` 的参数、同一行前面的 `export`；
# 往 `bash -c '…'`、`capped.sh N …`、`nice`、`timeout` 这类包装里面传。git 的 pre-commit hook 由 git 起，不经这道闸。
# 写进脚本文件再执行的，读脚本正文、逐条照同一张表判：命令位置上是 `bash|sh 文件`（不带 -c）、`./x.sh` 或 `路径/x.sh`、`source 文件` / `. 文件`，
#   经 capped.sh N、nice、timeout、env、前缀变量包一层的同样认；脚本里再起脚本的递归读，读到第 MAXIMUM_SCRIPT_DEPTH（6）层为止，再往里判「看不全」。
#   外层命令带的前缀与 export 往脚本里传，脚本里某一行自己写的前缀同样认；谁能跑什么照上面那张表，不因为写在脚本里而变。
#   拒绝说明写出脚本路径与行号（`外层脚本:行 → 里一层的脚本:行`）。
#   按名字判的仓内脚本（.claude/gate.d/ 下的阶段，gate.sh、gate-staged.sh、mutate.sh、capped.sh、run-with-memory-cap.sh 这几个在各自的仓内位置上）不读正文：
#   算不算重型、带什么参数才算，已经在名字那一格判完（lib_heavy_tests.py 的 judged_by_name）。
#   直接执行的（`./x`、`路径/x`，经包装的同样）只当两种文件是脚本：`#!` 指到 shell（SHELL_NAMES：bash / sh / dash / zsh / ksh，`#!/usr/bin/env` 转一层也算），
#   或没有 `#!`、扩展名是 `.sh`（执行的名字或它指到的文件）；开头是编译产物（ELF 魔数、NUL 字节）的也不算。别的一律不读、不记：
#   `.rs`、`.py`、`.md`、`.tsv` 这类、没扩展名又没 `#!` 的文本、`#!` 指到 python 这类别的解释器的。判它只看开头 SCRIPT_HEAD_BYTES（4 KiB），判在大小上限之前；
#   还不在的，只有扩展名是 `.sh` 的记「不存在」。`bash x`、`sh x`、`source x`、`. x` 显式交给 shell 的不看扩展名与 `#!`，照读。
#   读不到的——文件不存在、不是文本、超过 MAXIMUM_SCRIPT_BYTES（1 MiB）、嵌套超过上限看不全——这条命令照常执行，stderr 报一句并记一条检出；
#   不是普通文件的（目录、设备）不读、不记。脚本调回它自己（或绕一圈回来）不再读第二遍。
#   一次判定里（JudgingState）同一份文件按实际路径只从盘上读一遍；同一份脚本判过一遍，再遇到时直接用那一遍的结果，拒绝说明与检出照这一处的路径与行号写。
#   复用要这几样都相同：实际路径、执行时的当前目录、SINGLEFS_HEAVY_TESTS 的值、同一条命令里写出的文件自那一遍起没变过（written_version）、起它的那一条经没经内存包装；
#   那一遍撞上了嵌套上限、绕回了它外层的脚本、或自己写出了文件的，结果跟着从哪里起、什么时候起走，不复用，再遇到时重判（正文仍不重读）。
# 同一条命令里前面写出、后面执行的脚本（WrittenScript），执行时拿写出的内容判，先于盘上那一份（执行时盘上已经是写出的内容），不记「不存在」：
#   heredoc 喂给 cat 再输出重定向到那个文件（`cat > x.sh <<'EOF'`、`cat <<'EOF' > x.sh`；cat 带了文件参数或别的选项的不算），
#   heredoc 喂给 tee（参数里的文件，连同它的输出重定向），拿 heredoc 正文当脚本内容；
#   `cp 源 目的`（也认 `cp 源… 目录`、`-t 目录`），源在盘上就读源文件，源是同一条命令里更前面写出的就接着用那份内容；
#   追加（`>>`、`tee -a`）接在这条命令里前面写出的内容、或盘上那份（读得了的文本）后面；盘上那份读不了的不认，照旧读盘上。
#   脚本里再写出、再执行的同样认，写出的文件对外层后面的命令也算数。读法与判定同盘上的脚本（开头的 `#!`、大小上限、递归读）。
#   认不出的写法（printf、echo、python 写文件、mv、install、管道 `cat <<EOF | tee x.sh`、变量拼出的目标）照旧：还不在的按「不存在」记检出，已经在的读盘上那份。
#   正文不带引号的 heredoc（`<<EOF`）写进文件之前 shell 还要展开 `$` 与命令替换，这里判的是展开之前的原文。
# 命令位置上的词里带没展开的 `$`、反引号、括号、引号残片、重定向、通配字符、逗号或等号（NOT_A_LITERAL_PATH）的，不当脚本路径去读；
#   命令替换 `$(…)` 与反引号整段留在所在的词里，里面的命令按一条完整的命令另判（lib_shell_words.py）。
# 看不见的（照常放行、不记检出）：变量里拼出来的与通配（`*`、`?`、`[`）写出来的命令与脚本路径、eval、`<<<` 喂给 shell 的字符串；
#   喂给 python 这类非 shell 解释器的程序（`python3 x.py` 里用 subprocess 起的命令；heredoc 正文先剥掉）；
#   make（Makefile 里起的命令）与 xargs 起的命令；cd 到变量路径之后的裸 cargo test（认不出在不在工作区根）与相对路径的脚本；
#   命令词不带斜杠、从 PATH 里找到的脚本（直接执行的 `x.sh`）；同一条命令里用 WrittenScript 认不出的写法先写出再执行的脚本——
#   执行时文件还不在的按「不存在」放行并记检出，已经在的读到的是旧内容；source 进来的文件里 export 的变量对外层后面命令的影响。
# 另一道，与重型不重型无关：子 agent 跑编译出来的代码——cargo test / t / run / r / bench（test、bench 带 --no-run 的只编不跑，不算）、
#   直接执行 cargo 编出来的二进制（lib_heavy_tests.runs_compiled_code）——要经 research/scripts/run-with-memory-cap.sh 跑，不经它的拒
#   （它先判整机放不放得下、放不下排队，撞了上限只杀这一条；records/2026-09-16-subagent拆分提案.md 第四十节第 30 行）。放在这里而不另起一个 hook：
#   判的是同一批对象（Bash 命令里的 cargo test / run 与测试二进制）、同一个触发点、同一套切词与读脚本。
#   算经它包着的：它剥出来的那条命令、它 `bash -c` 里的、它起的脚本里的（往里读时接着算，lib_shell_words 的 memory_capped）；
#   包装外面的命令替换 `$(…)` 先于包装执行，不算。主 agent 不判这一道。按名字判的仓内脚本（门禁阶段、mutate.sh 这些）不读正文，
#   它们里面起的 cargo 不在这一道的射程里（59 号与 mutate.sh 自己每条经包装跑）。喂给 shell 的 heredoc 正文里的命令不算在挂它的那条命令的包装里：
#   `run-with-memory-cap.sh 4G bash <<EOF` 会被误拒，写成脚本文件或 bash -c。HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1 时不判这一道（只给自检证明它会红）。
# 判定（一条命令是不是重型、属于哪一类）在同目录的 lib_heavy_tests.py，看门狗在进程这一层复用同一份；
# 切词、切简单命令、认命令位置、剥前缀与包装、跟 cd 与 export 在同目录的 lib_shell_words.py（与 bash-command-detector.sh 共用），由 lib_heavy_tests.py 导入。
# 两份都按文件路径导入，这里只留谁能跑什么、读脚本与拒绝；lib_shell_words.py 的函数在 .claude/hooks/ 别的文件里再定义一份，门禁 63 号判红。
# 读不到它们时这条命令照常执行，stderr 报一句并记一条检出。
#
#   heavy-test-guard.sh             # 从 stdin 读 hook 的 JSON；拒绝退出 2，放行退出 0
#   heavy-test-guard.sh --selftest  # 在临时工作区里走一遍拒绝与放行（连同 lib_heavy_tests.py 的自检）；HEAVY_TEST_GUARD_DISABLE_CHECK=1、
#                                   # HEAVY_TEST_GUARD_IGNORE_WRITTEN_SCRIPTS=1（不认同一条命令里写出的脚本）、HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1（不判经没经内存包装）时自检必须判红
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
# python 程序从文件描述符 3 读，标准输入留给 hook 的 JSON（与 write-guard.sh 同一个坑）。
python3 /dev/fd/3 "$HOOK_DIR" "$@" 3<<'PY'
import importlib.util, json, os, re, shutil, subprocess, sys, tempfile, time
from datetime import datetime, timezone
from typing import NamedTuple

def heavy_tests_library_path(hook_dir):
    return os.path.join(hook_dir, "lib_heavy_tests.py")

def load_heavy_tests(library_path):
    """判定的共用模块（它再导入同目录的 lib_shell_words.py）；读不到或导入出错就抛异常，由调用方处理。"""
    spec = importlib.util.spec_from_file_location("lib_heavy_tests", library_path)
    library = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(library)
    return library

try:
    heavy_tests = load_heavy_tests(heavy_tests_library_path(sys.argv[1]))
    shell_words, shared_library_error = heavy_tests.shell_words, None
except Exception as error:  # 文件不在、语法错、导入时抛的都算读不到
    heavy_tests, shell_words, shared_library_error = None, None, error

def shared_library_problem(hook_dir):
    return f"读不到共用模块 {heavy_tests_library_path(hook_dir)}（它再导入同目录的 lib_shell_words.py）：{shared_library_error!r}"

DEFAULT_DETECTIONS = "/tmp/claude-1000/agent-hook-detections.jsonl"
OCCASION_VARIABLE = "SINGLEFS_HEAVY_TESTS"
OCCASIONS = {"commit", "user-request"}
MAXIMUM_SCRIPT_DEPTH = 6        # 从命令文本起一层层读进去的脚本最多几层；再往里判「看不全」
MAXIMUM_SCRIPT_BYTES = 1 << 20  # 脚本超过这么大不读，判「读不到」
# 直接执行的文件先读开头这么多判是不是 shell 脚本（ELF 魔数、NUL 字节、`#!` 指到别的解释器、没有 `#!` 而扩展名不是 .sh），不是就不读也不记，之后才判大小上限：
# 编译产物常有几 MB，先判大小会把它记成「超过上限、没判」（2026-09-24 现场：research/target/release/e49_chain_width 5124256 字节）
SCRIPT_HEAD_BYTES = 4096
SHELL_SCRIPT_EXTENSION = ".sh"  # 直接执行、没有 `#!` 的文件，只有这个扩展名的当 shell 脚本读

# 子 agent 自己那一份（kind 见 lib_heavy_tests.KIND_CATEGORY）；不在表里的子 agent 一样也不许
AGENT_KINDS = {
    "crash-verifier": {"layer0-stage", "layer0-cargo", "layer0-binary", "qemu-stage", "qemu-system", "herd7-stage", "lkmm", "herd7",
                       "crates-mutation-stage", "crates-mutation-mutate"},
    "gate-triage": {"gate-sh", "gate-staged", "replay-all-stage"},
}

class HeavyUse(NamedTuple):
    test: object           # lib_heavy_tests.HeavyTest
    occasion: str | None   # 这一条带的 SINGLEFS_HEAVY_TESTS 值
    frames: tuple          # 读到它经过的脚本：((脚本路径, 行号或 None), …)，最后一格是它自己所在的那一行；命令文本里直接写的是 ()

class Notice(NamedTuple):
    frames: tuple          # 同 HeavyUse.frames：出在哪个脚本的哪一行，命令文本里直接写的是 ()
    text: str              # 读不到、看不全的是什么

class HeavyScan(NamedTuple):
    uses: list             # [HeavyUse]
    notices: list          # [Notice]：读不到、看不全的脚本，这条命令照常执行，记检出
    scripts_read: list     # 读进去判过的脚本路径，判一遍记一次，复用上一遍的结果不记（放行时分得出「读了、里面没有」与「根本没读」）
    hit_depth_limit: bool  # 这一层或更里面有脚本因为嵌套超过上限没读：结果跟着从第几层起走
    lowest_cycle_frame: int | None  # 这一层或更里面绕回外层脚本而没读时，绕回的那一格在帧里的最小下标；没绕回是 None
    uncapped: list = []    # [UncappedRun]：不经内存包装跑编译出来的代码的每一处（子 agent 要拒的那一道）

class UncappedRun(NamedTuple):
    detail: str            # lib_heavy_tests.runs_compiled_code 的说明（cargo test、直接执行 cargo 编出来的二进制 …）
    frames: tuple          # 同 HeavyUse.frames

class WrittenScript(NamedTuple):
    text: str | None          # heredoc 写出的正文；cp 拷来的是 None
    copied_from: str | None   # cp 拷来的：源文件的绝对路径；heredoc 写出的是 None

class FileOnDisk(NamedTuple):
    head: bytes            # 开头 SCRIPT_HEAD_BYTES 字节（判是不是 shell 脚本用）；读不了是 b""
    data: bytes | None     # 整份；超过 MAXIMUM_SCRIPT_BYTES 或读不了是 None
    size: int
    error: str | None      # 读不了的原因（OSError 的 strerror）；读得了是 None

class JudgingState:
    """一次判定里命令文本与读进去的各层脚本共用的：写出的文件、从盘上读过的文件、判过的脚本。"""
    def __init__(self):
        self.written = {}          # 实际路径 → WrittenScript：同一条命令里到这里为止写出的文件（record_files_written）
        self.written_version = 0   # written 每变一次加一；判过的脚本只在它自那一遍起没变时复用
        self.files = {}            # 实际路径 → FileOnDisk：一次判定里同一份文件只从盘上读一遍
        self.files_read = []       # 从盘上读过的文件的实际路径，按读的次序，每份一次
        self.judged = {}           # (实际路径, 当前目录, SINGLEFS_HEAVY_TESTS 的值, written_version, 经没经内存包装) → HeavyScan，
                                   # 帧去掉了起它那一行之上的几格（复用时接上这一处的）

class Decision(NamedTuple):
    code: int              # 0 放行，2 拒绝
    message: str | None    # 拒绝时写给 stderr 的原因与出路
    notices: list          # 读不到、看不全的脚本（写好的句子，出在脚本里的带路径与行号）
    scripts_read: list     # 同 HeavyScan.scripts_read
    files_read: list       # 同 JudgingState.files_read

# 命令位置上的词里有这些字符，就是没展开的变量、命令替换、通配、引号残片或别的语言的源码残片（逗号、等号），不是一个能读的路径
NOT_A_LITERAL_PATH = set("$`()'\"<>|&;*?[]{}=,\n")

LAUNCHER_WORDS = {"shell": "bash / sh 起的", "source": "source 读进来的", None: "直接执行的"}

def frames_text(frames):
    return " → ".join(f"{path}:{line}" if line else f"{path}（行号认不出）" for path, line in frames)

def command_lines(script_text):
    """脚本里每条命令（剥掉前缀与包装之后的词序列）出现在哪几行，按出现次序；
    逐个逻辑行（反斜杠续行接上）单独切词，喂给非 shell 命令的 heredoc 正文先换成同样多的空行，行号不变。"""
    lines = shell_words.strip_data_heredocs(script_text, keep_line_count=True).split("\n")
    found, number = {}, 0
    while number < len(lines):
        first_line, logical = number + 1, lines[number]
        while logical.endswith("\\") and number + 1 < len(lines):
            number += 1
            logical += "\n" + lines[number]
        number += 1
        for command in shell_words.commands_at_command_position(logical).commands:
            found.setdefault(tuple(command.words), []).append(first_line)
    return found

def literal_path(command, word):
    """命令里的一个词当文件路径（按这条命令的当前目录解开、取实际路径）；带没展开的变量、通配这类的，或目录认不出，交 None。"""
    if not word or NOT_A_LITERAL_PATH & set(word):
        return None
    resolved = shell_words.resolve_path(command.directory, os.path.expanduser(word))
    return None if resolved is None else os.path.realpath(resolved)

def heredoc_write_targets(command):
    """这一条 cat / tee 把喂给它的 heredoc 写进了哪些文件：[(目标词, 是不是追加)]；不是这种写法交 []。"""
    name, arguments = os.path.basename(command.words[0]), command.words[1:]
    if name not in ("cat", "tee") or not command.standard_input_heredocs:
        return []
    targets = []
    if name == "cat":
        if any(argument != "-" for argument in arguments):  # 带了文件参数（写出的是那些文件）或别的选项（-n 这类改了内容）
            return []
    else:
        only_files = False
        for argument in arguments:
            if not only_files and argument == "--":
                only_files = True
            elif only_files or not argument.startswith("-") or argument == "-":
                if argument != "-":
                    targets.append(argument)
        appends = any(argument in ("-a", "--append") or (argument.startswith("-") and not argument.startswith("--") and "a" in argument[1:])
                      for argument in arguments if argument != "--")
        targets = [(target, appends) for target in targets]
    for operator, target in command.output_redirections:
        if operator == ">&" and (target.isdigit() or target == "-"):
            continue
        targets.append((target, operator in (">>", "&>>")))
    return targets

def copy_pairs(command):
    """`cp` 这一条拷了哪些文件：[(源词, 目的词)]；不是 cp、或参数认不出，交 []。"""
    if os.path.basename(command.words[0]) != "cp":
        return []
    operands, target_directory, no_target_directory, only_operands = [], None, False, False
    arguments, position = command.words[1:], 0
    while position < len(arguments):
        argument = arguments[position]
        if not only_operands and argument == "--":
            only_operands = True
        elif not only_operands and argument in ("-t", "--target-directory", "-S", "--suffix"):
            if argument in ("-t", "--target-directory") and position + 1 < len(arguments):
                target_directory = arguments[position + 1]
            position += 1
        elif not only_operands and argument.startswith("--target-directory="):
            target_directory = argument.split("=", 1)[1]
        elif not only_operands and argument.startswith("-") and argument != "-":
            no_target_directory = no_target_directory or argument == "--no-target-directory" or (
                not argument.startswith("--") and "T" in argument[1:])
        else:
            operands.append(argument)
        position += 1
    if target_directory is not None:
        return [(source, os.path.join(target_directory, os.path.basename(source))) for source in operands]
    if len(operands) < 2:
        return []
    sources, destination = operands[:-1], operands[-1]
    destination_path = literal_path(command, destination)
    into_directory = not no_target_directory and (len(sources) > 1 or destination.endswith("/")
                                                   or (destination_path is not None and os.path.isdir(destination_path)))
    if into_directory:
        return [(source, os.path.join(destination, os.path.basename(source))) for source in sources]
    return [(sources[0], destination)]

def file_on_disk(path, state):
    """盘上一份普通文件的内容：一次判定里同一份（按实际路径认）只读一遍，读过的记进 state.files_read。"""
    real_path = os.path.realpath(path)
    known = state.files.get(real_path)
    if known is not None:
        return known
    try:
        size = os.path.getsize(real_path)
        with open(real_path, "rb") as handle:
            data = handle.read(MAXIMUM_SCRIPT_BYTES + 1) if size <= MAXIMUM_SCRIPT_BYTES else None
            head = handle.read(SCRIPT_HEAD_BYTES) if data is None else data[:SCRIPT_HEAD_BYTES]
        if data is not None and len(data) > MAXIMUM_SCRIPT_BYTES:  # 量过大小之后才长大的
            size, data = len(data), None
        known = FileOnDisk(head, data, size, None)
    except OSError as error:
        known = FileOnDisk(b"", None, 0, error.strerror)
    state.files[real_path] = known
    state.files_read.append(real_path)
    return known

def text_on_disk(path, state):
    """追加之前盘上那份的正文：还不在交 ""，不是普通文件、超过读的上限、读不了、不是 UTF-8 文本交 None。"""
    if not os.path.lexists(path):
        return ""
    if not os.path.isfile(path):
        return None
    found = file_on_disk(path, state)
    if found.data is None:
        return None
    try:
        return found.data.decode("utf-8")
    except UnicodeDecodeError:
        return None

def record_files_written(command, state):
    """这一条命令写出了哪些文件，记进 state.written（实际路径 → WrittenScript），后面的命令执行它们时拿这份内容判；每变一处 written_version 加一。"""
    if os.environ.get("HEAVY_TEST_GUARD_IGNORE_WRITTEN_SCRIPTS") == "1":
        return
    written = state.written
    def remember(path, entry):
        written[path] = entry
        state.written_version += 1
    def forget(path):
        if written.pop(path, None) is not None:
            state.written_version += 1
    body = command.standard_input_heredocs[-1].text if command.standard_input_heredocs else None
    for target, appends in heredoc_write_targets(command):
        path = literal_path(command, target)
        if path is None:
            continue
        if not appends:
            remember(path, WrittenScript(body, None))
            continue
        entry = written.get(path)
        before = (text_on_disk(path, state) if entry is None else entry.text if entry.text is not None
                  else text_on_disk(entry.copied_from, state))
        if before is None:
            forget(path)  # 追加到读不了的文件上：不认，照旧读盘上那份
        else:
            remember(path, WrittenScript(before + body, None))
    for source, destination in copy_pairs(command):
        source_path, destination_path = literal_path(command, source), literal_path(command, destination)
        if source_path is None or destination_path is None:
            continue
        if source_path in written:
            remember(destination_path, written[source_path])
        elif os.path.isfile(source_path):
            remember(destination_path, WrittenScript(None, source_path))
        else:
            forget(destination_path)  # 源不在：认不出，执行时照旧按盘上有没有判

def has_shell_script_extension(path):
    """执行的名字、或它指到的文件，扩展名是 .sh。"""
    return path.endswith(SHELL_SCRIPT_EXTENSION) or os.path.realpath(path).endswith(SHELL_SCRIPT_EXTENSION)

def script_path_of(command, written):
    """命令位置上这一条执行的是不是一个要读进去的脚本文件：返回 (路径, None) 要读；(None, 找不到的说明) 记检出；(None, None) 不读也不记。
    written 是同一条命令里前面写出的文件（record_files_written），在里面的算找得到；直接执行的还不在，扩展名不是 .sh 的不读也不记。"""
    word = command.words[0]
    if command.launcher == "python" or word.startswith("-") or NOT_A_LITERAL_PATH & set(word):
        return None, None
    expanded = os.path.expanduser(word)
    if command.launcher is None and "/" not in expanded:
        return None, None
    resolved = shell_words.resolve_path(command.directory, expanded)
    if resolved is None:
        return None, None
    if os.path.realpath(resolved) in written or os.path.lexists(resolved):
        return resolved, None
    if command.launcher is not None and "/" not in expanded:
        found = shutil.which(expanded, mode=os.F_OK)
        if found:
            return os.path.abspath(found), None
    if command.launcher is None and not has_shell_script_extension(resolved):
        return None, None
    return None, f"{LAUNCHER_WORDS[command.launcher]}脚本 {resolved} 不存在"

def shebang_interpreter(data):
    """`#!` 那一行指到的解释器名（`#!/usr/bin/env bash` 取 bash）；没有 `#!` 交 None。"""
    if not data.startswith(b"#!"):
        return None
    end = data.find(b"\n")
    words = data[2:end if end >= 0 else len(data)].decode("utf-8", "replace").split()
    if not words:
        return ""
    interpreter = os.path.basename(words[0])
    if interpreter == "env":
        rest = [word for word in words[1:] if not word.startswith("-") and "=" not in word]
        interpreter = os.path.basename(rest[0]) if rest else ""
    return interpreter

def executed_file_is_shell_script(executed_path, head):
    """直接执行的文件当不当 shell 脚本读，只看开头：编译产物（ELF 魔数、NUL 字节）不算；有 `#!` 的看它指不指到 shell；
    没有 `#!` 的看扩展名是不是 .sh（`.rs`、`.py`、`.md`、没扩展名的文本都不算）。"""
    if head.startswith(b"\x7fELF") or b"\0" in head:
        return False
    interpreter = shebang_interpreter(head)
    if interpreter is not None:
        return interpreter in shell_words.SHELL_NAMES
    return has_shell_script_extension(executed_path)

def decoded_script_text(data, path, launcher, executed_path):
    """已经读到手的脚本字节（盘上读的、或同一条命令里 heredoc 写出的）：返回值同 script_text。"""
    how = LAUNCHER_WORDS[launcher]
    if launcher is None and not executed_file_is_shell_script(executed_path, data[:SCRIPT_HEAD_BYTES]):
        return None, None
    if len(data) > MAXIMUM_SCRIPT_BYTES:
        return None, f"{how}脚本 {path} 有 {len(data)} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    if b"\0" in data:
        return None, f"{how}脚本 {path} 不是文本（有 NUL 字节）"
    try:
        return data.decode("utf-8"), None
    except UnicodeDecodeError:
        return None, f"{how}脚本 {path} 不是文本（不是 UTF-8）"

def script_text(path, launcher, state, executed_path):
    """盘上 path 那份当脚本读（executed_path 是执行的名字，直接执行的按它的扩展名判）：返回 (正文, None) 要读；(None, 读不到的原因) 记检出；
    (None, None) 不是普通文件（目录、设备）或直接执行的不是 shell 脚本，不读也不记。"""
    how = LAUNCHER_WORDS[launcher]
    if not os.path.isfile(path):
        return None, None
    found = file_on_disk(path, state)
    if found.error is not None:
        return None, f"{how}脚本 {path} 读不了（{found.error}）"
    if launcher is None and not executed_file_is_shell_script(executed_path, found.head):
        return None, None
    if found.data is None:
        return None, f"{how}脚本 {path} 有 {found.size} 字节，超过读的上限 {MAXIMUM_SCRIPT_BYTES} 字节"
    return decoded_script_text(found.data, path, launcher, executed_path)

def script_text_to_judge(path, launcher, state):
    """执行时这个脚本里是什么：同一条命令里前面写出的拿写出的内容（cp 拷来的读源文件），其余读盘上那份；返回值同 script_text。"""
    entry = state.written.get(os.path.realpath(path))
    if entry is None:
        return script_text(path, launcher, state, path)
    if entry.copied_from is not None:
        return script_text(entry.copied_from, launcher, state, path)
    return decoded_script_text(entry.text.encode("utf-8"), path, launcher, path)

def lower_cycle_frame(first, second):
    """两个「绕回外层的那一格」取靠外的那个；None 是没绕回。"""
    if first is None:
        return second
    return first if second is None else min(first, second)

def heavy_uses(text, directory, environment=None, frames_above=(), script=None, state=None, memory_capped=False):
    """text 里命令位置上的每一处重型用法；执行的是脚本文件就读进去，用同一套切词与判定递归判。
    script 是 text 所在的脚本（命令文本本身是 None），frames_above 是读到它为止经过的外层脚本与起它的那一行；
    state 是这一次判定里外各层共用的 JudgingState：脚本里写出的，外层后面的命令也看得见；同一份脚本只读一遍，判过的按键复用。
    memory_capped：text 整个跑在 run-with-memory-cap.sh 里（起它所在脚本的那一条经它包着）；不经它跑编译出来的代码的每一处记进 uncapped。"""
    if state is None:
        state = JudgingState()
    uses, notices, scripts_read, occurrences, uncapped = [], [], [], {}, []
    hit_depth_limit, lowest_cycle_frame = False, None
    line_index = None
    def frames_of(key, occurrence):
        nonlocal line_index
        if script is None:
            return frames_above
        if line_index is None:
            line_index = command_lines(text)
        lines = line_index.get(key, [])
        return frames_above + ((script, lines[occurrence] if occurrence < len(lines) else None),)
    for command in shell_words.commands_at_command_position(text, directory, environment, memory_capped=memory_capped).commands:
        key = tuple(command.words)
        occurrence = occurrences.get(key, 0)
        occurrences[key] = occurrence + 1
        record_files_written(command, state)
        compiled = heavy_tests.runs_compiled_code(command.words, command.directory)
        if compiled and not command.memory_capped:
            uncapped.append(UncappedRun(compiled, frames_of(key, occurrence)))
        test = heavy_tests.classify(command.words, command.directory)
        if test:
            uses.append(HeavyUse(test, command.environment.get(OCCASION_VARIABLE), frames_of(key, occurrence)))
            continue
        if heavy_tests.judged_by_name(command.words[0], command.directory):
            continue
        path, missing = script_path_of(command, state.written)
        if path is None and missing is None:
            continue
        here = frames_of(key, occurrence)
        if missing:
            notices.append(Notice(here, f"{missing}，里面的命令没判"))
            continue
        real_path = os.path.realpath(path)
        enclosing = [os.path.realpath(frame_path) for frame_path, _ in here]
        if real_path in enclosing:
            lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, enclosing.index(real_path))
            continue
        if len(here) + 1 > MAXIMUM_SCRIPT_DEPTH:
            hit_depth_limit = True
            notices.append(Notice(here, f"{LAUNCHER_WORDS[command.launcher]}脚本 {path} 是第 {len(here) + 1} 层，"
                                        f"超过读的上限 {MAXIMUM_SCRIPT_DEPTH} 层，看不全，里面的命令没判"))
            continue
        body, problem = script_text_to_judge(path, command.launcher, state)
        if problem:
            notices.append(Notice(here, f"{problem}，里面的命令没判"))
            continue
        if body is None:
            continue
        judged_key = (real_path, command.directory, command.environment.get(OCCASION_VARIABLE), state.written_version, command.memory_capped)
        earlier = state.judged.get(judged_key)
        if earlier is not None:
            uses += [use._replace(frames=here + use.frames) for use in earlier.uses]
            notices += [notice._replace(frames=here + notice.frames) for notice in earlier.notices]
            uncapped += [run._replace(frames=here + run.frames) for run in earlier.uncapped]
            continue
        scripts_read.append(path)
        inner = heavy_uses(body, command.directory, command.environment, here, path, state, command.memory_capped)
        uses += inner.uses
        notices += inner.notices
        uncapped += inner.uncapped
        scripts_read += inner.scripts_read
        hit_depth_limit = hit_depth_limit or inner.hit_depth_limit
        lowest_cycle_frame = lower_cycle_frame(lowest_cycle_frame, inner.lowest_cycle_frame)
        # 它自己在帧里的下标是 len(here)：绕回比它更外的、撞上嵌套上限的、判的时候写出了文件的，换一处起就可能不一样，不复用
        depends_on_where_started = inner.hit_depth_limit or (inner.lowest_cycle_frame is not None and inner.lowest_cycle_frame < len(here))
        if not depends_on_where_started and state.written_version == judged_key[3]:
            state.judged[judged_key] = HeavyScan([use._replace(frames=use.frames[len(here):]) for use in inner.uses],
                                                 [notice._replace(frames=notice.frames[len(here):]) for notice in inner.notices],
                                                 [], False, None,
                                                 [run._replace(frames=run.frames[len(here):]) for run in inner.uncapped])
    return HeavyScan(uses, notices, scripts_read, hit_depth_limit, lowest_cycle_frame, uncapped)

POLICY = ("→ 规矩：重型测试（层 0、QEMU、herd7、crates 变异整表、全量测试、整轮门禁、全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；"
          "子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）与 fmt / clippy / build；"
          "主 agent 在提交流程里跑要带 `SINGLEFS_HEAVY_TESTS=commit`，用户要求时带 `SINGLEFS_HEAVY_TESTS=user-request`。\n"
          "→ 各自那一份：crash-verifier 只跑 55、57、59 号与 qemu-system、lkmm.sh / herd7、crates 变异整表（54 号快档在 gate.sh --staged 里，全量由主 agent 跑）；"
          "gate-triage 只跑 `gate.sh` 整轮与 87 号（54、55、57、59 靠「输入没变就复用上一次全绿判定」）；两个都要带那个前缀，都不跑全量 `cargo test`。"
          "`.claude/gate.d/` 下其余阶段不是重型，谁都能跑。\n"
          "→ 提交之外任务确实要跑的：主 agent 先弹窗问用户，用户同意了才带 `SINGLEFS_HEAVY_TESTS=user-request` 跑；"
          "子 agent 在交回里写明要跑什么、为什么，交主 agent 去问（派发提示里点名要你跑的也一样，写明被这道闸拒了）。")
MEMORY_CAP_POLICY = ("→ 规矩：子 agent 跑编译出来的代码（cargo test / cargo run / cargo bench、直接执行 cargo 编出来的二进制）要经内存包装跑："
                     "`bash research/scripts/run-with-memory-cap.sh <上限> <命令>`（上限例 4G）——它先判整机放不放得下、放不下就排队，撞了上限只杀这一条，"
                     "不把整机拖进 OOM；写在脚本里的，整条脚本经它跑：`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`。"
                     "只编不跑的（cargo build / clippy、cargo test --no-run）不用经它；要限时设 RUN_WITH_MEMORY_CAP_TIME_LIMIT=<秒>，别在包装外面套 timeout（排队的时间也会算进去）。")
SCRIPT_POLICY = ("→ 这一步写在脚本里：命令位置上执行的脚本（bash / sh / ./ / source 起的，经 capped.sh / nice / timeout / env 包一层的，脚本里再起的脚本）"
                 "读进去逐条照同一张表判。把那一步从脚本里拿掉再跑，或照上一条交主 agent。")

def refusal_reason(kind, occasion, agent_type):
    """放行返回 None；拒绝返回一句原因。"""
    who = agent_type or "主 agent"
    category = heavy_tests.KIND_CATEGORY[kind]
    if agent_type and kind not in AGENT_KINDS.get(agent_type, set()):
        return f"{who} 不跑「{category}」"
    if occasion not in OCCASIONS:
        carried = "没带" if occasion is None else f"带的是 {OCCASION_VARIABLE}={occasion}"
        return f"{who} 跑「{category}」要带 {OCCASION_VARIABLE}=commit 或 =user-request，这一条{carried}"
    return None

def decide(hook_input, project_root):
    """返回 Decision：code 0 放行、2 拒绝；notices 是读不到、看不全的脚本，放不放行都照记。"""
    if os.environ.get("HEAVY_TEST_GUARD_DISABLE_CHECK") == "1":
        return Decision(0, None, [], [], [])
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    command = tool_input.get("command") or ""
    agent_type = hook_input.get("agent_type") or None
    directory = hook_input.get("cwd") or project_root
    state = JudgingState()
    scan = heavy_uses(command, directory, state=state)
    notices = [f"脚本 {frames_text(notice.frames)} 里{notice.text}" if notice.frames else notice.text for notice in scan.notices]
    refused, in_script = [], False
    for use in scan.uses:
        reason = refusal_reason(use.test.kind, use.occasion, agent_type)
        if reason:
            where = f"脚本 {frames_text(use.frames)} 里的 " if use.frames else ""
            in_script = in_script or bool(use.frames)
            refused.append(f"{where}{use.test.detail}（{use.test.category}）：{reason}")
    uncapped_refused = []
    if agent_type and os.environ.get("HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP") != "1":
        for run in scan.uncapped:
            where = f"脚本 {frames_text(run.frames)} 里的 " if run.frames else ""
            in_script = in_script or bool(run.frames)
            uncapped_refused.append(f"{where}{run.detail}（{agent_type} 不经 run-with-memory-cap.sh）")
    if not refused and not uncapped_refused:
        return Decision(0, None, notices, scan.scripts_read, state.files_read)
    lines, policies = [], []
    if refused:
        lines += [f"✗ 重型测试被拒：{refused[0]}"] + [f"  另有：{entry}" for entry in refused[1:]]
        policies.append(POLICY)
    if uncapped_refused:
        lines += [f"✗ 不经内存包装跑编译出来的代码被拒：{uncapped_refused[0]}"] + [f"  另有：{entry}" for entry in uncapped_refused[1:]]
        policies.append(MEMORY_CAP_POLICY)
    if in_script:
        policies.append(SCRIPT_POLICY)
    message = "\n".join(lines + policies)
    return Decision(2, message, notices, scan.scripts_read, state.files_read)

def record_detection(hook_input, findings, detections_path):
    tool_input = hook_input.get("tool_input") if isinstance(hook_input.get("tool_input"), dict) else {}
    entry = {
        "time": datetime.now(timezone.utc).isoformat(),
        "session_id": hook_input.get("session_id"),
        "agent_id": hook_input.get("agent_id"),
        "agent_type": hook_input.get("agent_type") or "主 agent",
        "transcript_path": hook_input.get("transcript_path"),
        "command": (tool_input.get("command") or "")[:500],
        "findings": findings,
    }
    try:
        os.makedirs(os.path.dirname(detections_path), exist_ok=True)
        with open(detections_path, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass

def build_sample_scripts(work):
    """自检用的脚本：被拒的那一条（cargo test --all）写在 s.sh 第 4 行；另有嵌套、只含轻命令、读不到、看不全与不读的几种。"""
    def write(relative, content, mode="w"):
        path = os.path.join(work, relative)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, mode) as handle:
            handle.write(content)
    write("s.sh", "#!/usr/bin/env bash\n# 验证链：先编，再跑\nset -euo pipefail\ncargo test --all\n")
    write("outer.sh", "#!/usr/bin/env bash\necho 起里一层\nbash s.sh\n")
    write("own-prefix.sh", "SINGLEFS_HEAVY_TESTS=commit cargo test --all\n")
    write("light.sh", "#!/usr/bin/env bash\nbash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract\necho \"cargo test --all\"\n"
                      "cat > note.md <<'EOF'\ncargo test --all\nEOF\ncargo clippy --all-targets -- -D warnings\n")
    write("loop.sh", "#!/usr/bin/env bash\n[ -n \"${AGAIN:-}\" ] || AGAIN=1 bash loop.sh\n")
    for level in range(1, 8):
        write(f"deep{level}.sh", f"bash deep{level + 1}.sh\n" if level < 7 else "cargo test --all\n")
    write("blob.bin", b"\x00\x01cargo test --all\n", "wb")
    write("big.sh", "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "cargo test --all\n")
    write("tool", "#!/usr/bin/env python3\nimport subprocess\nsubprocess.run(['cargo', 'test', '--all'])\ncargo test --all\n")
    write("prog", b"\x7fELF\x02\x01\x01\x00\x00\ncargo test --all\n", "wb")
    # 超过读的上限的编译产物与 python 程序：先认出不是 shell 脚本、不读也不记，轮不到大小上限（2026-09-24 现场误记过 e49_chain_width）
    write("bigprog", b"\x7fELF\x02\x01\x01\x00" + b"\x00" * (MAXIMUM_SCRIPT_BYTES + 64) + b"\ncargo test --all\n", "wb")
    write("bigtool", "#!/usr/bin/env python3\n" + "# 填充\n" * (MAXIMUM_SCRIPT_BYTES // 8 + 1) + "cargo test --all\n")
    write("elsewhere/mutate.sh", "#!/usr/bin/env bash\ncargo test --all\n")
    write("research/scripts/gate-staged.sh", "#!/usr/bin/env bash\nbash .claude/scripts/gate.sh --staged\n")
    # 查名字与数组赋值：照 .claude/scripts/fetch-deps.sh 第 44 行、research/scripts/change-touches-crates.sh 第 61 行原样
    write("command-lookup.sh", "#!/usr/bin/env bash\n  if command -v herd7 >/dev/null 2>&1; then ok \"herd7 已在\"; return 0; fi\n")
    write("array-assignment.sh", "#!/usr/bin/env bash\n  ((${#prefixes[@]})) || prefixes=(\"crates/\")\nchains=(./s.sh ./outer.sh)\n")
    # 命令替换的两种写法，照 .claude/singlefs-ai-sop/scripts/doc-lint.sh 第 43 行、第 70 行原样：切碎之后的片段不许当成脚本路径去读
    write("doc-lint-line-43.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n")
    write("doc-lint-line-70.sh", "#!/usr/bin/env bash\n"
          "DOC_LINT_FAMILY=\"$(sed -n 's/^family=//p' \"$(dirname \"${BASH_SOURCE[0]}\")/../I18N\" 2>/dev/null || true)\"\n")
    write("substitution-fragments.sh", "#!/usr/bin/env bash\ncd $(dirname $0)/.. && echo $((total/2)) `dirname $0`/x.sh\n"
          "$(dirname \"$0\")/helper.sh --flag\n`dirname $0`/helper.sh\n"
          "out=\"$(bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract 2>&1)\"\n")
    # 不经内存包装跑一个测试目标：子 agent 起它要拒（经包装起它放行）
    write("unwrapped.sh", "#!/usr/bin/env bash\ncargo test -p singlefs-core --test core_contract\n")
    write("substitution-heavy.sh", "#!/usr/bin/env bash\n  set -- \"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/..\" && pwd)\"\n"
          "summary=\"$(cargo test --all 2>&1 | tail -3)\"\n")
    # 直接执行时不当脚本读的文本：Rust 源码的文档注释里反引号括着彼此的路径与 `cargo test --all`，当 shell 读就一路互相读下去（2026-09-25 现场一条命令 100 秒没判完）
    checker_sources = ["walk.rs", "image.rs", "lib.rs"]
    for name in checker_sources:
        others = "` 与 `".join(f"crates/singlefs-checker/src/{other}" for other in checker_sources if other != name)
        write(f"crates/singlefs-checker/src/{name}", f"//! 池级 checker：见 `{others}`\n/// 跑法：`cargo test --all`\npub fn check_pool_image() {{}}\n")
    write("notes.md", "# 验证链\ncargo test --all\n")
    write("plain-text", "cargo test --all\n")
    write(".claude/singlefs-ai-sop/scripts/doc-lint.sh", "#!/usr/bin/env bash\necho doc-lint\n")
    # 直接执行时当脚本读的：没扩展名而 `#!` 指到 shell 的，扩展名不对而 `#!` 指到 shell 的
    write("runner", "#!/usr/bin/env bash\ncargo test --all\n")
    write("mislabeled.py", "#!/bin/sh\ncargo test --all\n")
    # 一条命令里同一份脚本起几遍：twice.sh 三种起法各起一遍 s.sh；cyc-a.sh 与 cyc-b.sh 互相调；rel.sh 按相对路径起的脚本随当前目录变
    write("twice.sh", "#!/usr/bin/env bash\nbash s.sh\n./s.sh\nsource s.sh\n")
    write("cyc-a.sh", "bash cyc-b.sh\ncargo test --all\n")
    write("cyc-b.sh", "bash cyc-a.sh\n")
    write("rel.sh", "bash ./inner-rel.sh\n")
    write("inner-rel.sh", "echo 轻\n")
    write("sub/inner-rel.sh", "cargo test --all\n")
    write("has-missing.sh", "bash missing.sh\n")
    write("outer-missing.sh", "echo 起里一层\nbash has-missing.sh\n")

# 2026-09-25 现场一条命令 100 秒没判完：双引号里没转义的反引号把 `crates/singlefs-checker/src/walk.rs` 当命令替换执行（原样，cwd 换成自检的临时工作区）
SLOW_REPRODUCTION_COMMAND = 'python3 research/scripts/replace-once.py .claude/kb/verification-build.md "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，24 条）" "池级 checker（\\`crates/singlefs-checker\\` 的 \\`walk::check_pool_image\\`，26 条：第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽））" && python3 research/scripts/replace-once.py .claude/kb/milestone/02-second-txn.md "checker 新接的只有 I-3.8（实例表行唯一且低于挂载根），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判，预想清单里其余各条（I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽） 那一族起）还没有 checker；" "checker 新接了 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽）（后两条按候选集里每条根各判一格：走读它引用的单元时校验和对不上或头用不了就红，坏镜像是 mkfs 树表单元——只有第 0 代根还引用——改内容重封 / 抹成零；`crates/singlefs-checker/src/walk.rs`、`image.rs` 清单 26 条），I-3.1（已分配统计对得上） 与 I-2.1（校验和与内容匹配） 改按回退候选集判；预想清单里其余各条（I-3.4（可用空间扣待删占用） 起）还没有 checker；" && bash .claude/singlefs-ai-sop/scripts/doc-lint.sh . 2>&1 | tail -1'
SLOW_REPRODUCTION_SECONDS = 1.0  # 复现输入那一条要在这么多秒内判完

def selftest(hook_dir):
    if heavy_tests is None:
        print(f"  ✗ 自检：{shared_library_problem(hook_dir)}")
        print("    → 怎么办：恢复 .claude/hooks/lib_heavy_tests.py 与 lib_shell_words.py（判定与切词只有那两份，别在 hook 里再抄一份），再跑 --selftest")
        return 1
    work = tempfile.mkdtemp(prefix="heavy-test-guard-")
    results = []
    try:
        heavy_tests.build_sample_workspace(work)
        build_sample_scripts(work)
        writer, crash, triage, scribe = "implementation-writer", "crash-verifier", "gate-triage", "kb-scribe"
        commit, request = "SINGLEFS_HEAVY_TESTS=commit ", "SINGLEFS_HEAVY_TESTS=user-request "
        layer0_binary = "./target/release/deps/first_transaction_step_seven_layer0-0123456789abcdef"
        # (说明, agent_type（None 是主 agent）, 命令, 该拒 2 / 该放 0[, 该记几条检出, 该读进去几份脚本]；后两列不写是 0、0)
        cases = [
            ("实现员跑 cargo test --all", writer, "cargo test --all", 2),
            ("实现员跑层 0 测试目标", writer, "cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
            ("实现员跑 check.sh", writer, "bash .claude/scripts/check.sh", 2),
            ("实现员经 capped.sh 跑 --workspace", writer, "bash research/scripts/capped.sh 4 cargo test --workspace", 2),
            ("实现员经 run-with-memory-cap.sh 跑 --workspace", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test --workspace", 2),
            ("实现员 run-with-memory-cap.sh --selftest", writer, "bash research/scripts/run-with-memory-cap.sh --selftest", 0),
            ("实现员在 bash -c 里跑 --all", writer, "bash -c 'cargo test --all'", 2),
            ("实现员在仓根裸跑 cargo test", writer, "cargo test", 2),
            ("实现员 cd 进 harness 裸跑", writer, "cd crates/singlefs-harness && cargo test --release", 2),
            ("实现员 -p harness 不挑目标", writer, "cargo test -p singlefs-harness", 2),
            ("实现员 --test 通配命中层 0", writer, "cargo test -p singlefs-harness --test '*'", 2),
            ("实现员前缀一串再跑 --workspace", writer, "nice -n 19 timeout 600 env A=1 cargo +stable --offline test --workspace", 2),
            ("实现员在 research 工作区根裸跑", writer, "cd research && cargo test --release", 2),
            ("实现员在 research 唯一的成员里裸跑", writer, "cd research/e7-index-bench && cargo test", 2),
            ("实现员带前缀也拒", writer, commit + "cargo test --all", 2),
            ("实现员 mutate.sh 跑 crates 整表", writer, "cd research && bash scripts/mutate.sh x y ../crates/mutations.tsv", 2),
            ("实现员起 qemu-system", writer, "qemu-system-x86_64 -enable-kvm -m 512", 2),
            ("实现员 vm-bench.sh --selftest", writer, "bash research/scripts/vm-bench.sh --selftest", 2),
            ("实现员跑 herd7", writer, "herd7 -conf linux-kernel.cfg litmus/a.litmus", 2),
            ("实现员跑 lkmm.sh", writer, "bash .claude/scripts/lkmm.sh", 2),
            ("实现员跑 gate-staged.sh", writer, "bash research/scripts/gate-staged.sh", 2),
            ("实现员喂给 bash 的 heredoc 里跑 --all", writer, "bash <<'EOF'\ncargo test --all\nEOF", 2),
            ("实现员直接执行层 0 测试二进制", writer, layer0_binary + " --test-threads 4", 2),
            ("书记员跑 87 号", scribe, "nice -n 19 bash .claude/gate.d/87-replay.sh", 2),
            ("通用 agent 跑 54 号", "general-purpose", "bash .claude/gate.d/54-layer0-replay.sh", 2),
            ("崩溃验证员跑 54 号不带前缀", crash, "bash .claude/gate.d/54-layer0-replay.sh", 2),
            ("崩溃验证员带前缀跑 gate.sh", crash, commit + "bash .claude/scripts/gate.sh --staged", 2),
            ("崩溃验证员带前缀跑 cargo test --all", crash, commit + "cargo test --all", 2),
            ("崩溃验证员带前缀起 E152", crash, commit + "./research/target/release/e152-file-system-benchmark", 2),
            ("崩溃验证员带前缀跑 vm-bench.sh", crash, commit + "bash research/scripts/vm-bench.sh run", 2),
            ("门禁分诊不带前缀跑 gate.sh --staged", triage, "bash .claude/scripts/gate.sh --staged", 2),
            ("门禁分诊带前缀直接调 57 号", triage, commit + "bash .claude/gate.d/57-lkmm.sh", 2),
            ("门禁分诊带前缀直接调 59 号", triage, commit + "bash .claude/gate.d/59-crates-mutation-replay.sh", 2),
            ("主 agent 裸跑 gate.sh --staged", None, "bash .claude/scripts/gate.sh --staged", 2),
            ("主 agent 带 =whatever", None, "SINGLEFS_HEAVY_TESTS=whatever bash .claude/scripts/gate.sh --staged", 2),
            ("主 agent 第二条命令没带前缀", None, commit + "cargo test --all; bash .claude/scripts/gate.sh", 2),
            ("主 agent 裸跑 E152 的 run 脚本", None, "bash research/scripts/e152-run.sh /tmp/x", 2),
            ("主 agent 裸 cargo run E152", None, "cd research && cargo run --release --bin e152-file-system-benchmark", 2),
            ("主 agent 带前缀跑 --all", None, commit + "cargo test --all", 0),
            ("主 agent 带 =user-request 跑 gate.sh", None, request + "bash .claude/scripts/gate.sh --staged", 0),
            ("主 agent export 之后跑 54 号全量", None, "export SINGLEFS_HEAVY_TESTS=commit && bash /tmp/wt/.claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
            ("主 agent 前缀往包装里传", None, "nice -n 19 env SINGLEFS_HEAVY_TESTS=commit bash research/scripts/capped.sh 8 bash -c 'cargo test --workspace'", 0),
            ("主 agent 跑一个测试目标不是重型", None, "cargo test -p singlefs-core --test core_contract", 0),
            ("崩溃验证员带前缀跑 54 号全量", crash, commit + "bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
            ("崩溃验证员带前缀跑层 0 测试目标", crash,
             commit + "nice -n 19 bash research/scripts/run-with-memory-cap.sh 16G cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 0),
            ("崩溃验证员带前缀直接执行层 0 测试二进制", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G " + layer0_binary, 0),
            ("崩溃验证员带 =user-request 跑 55 号", crash, request + "bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
            ("崩溃验证员带前缀跑 59 号", crash, "GATE_MUTATION_TARGET_DIR=/tmp/t " + commit + "nice -n 19 bash .claude/gate.d/59-crates-mutation-replay.sh", 0),
            ("门禁分诊带前缀跑 gate.sh --staged", triage, commit + "nice -n 19 bash .claude/scripts/gate.sh --staged", 0),
            ("门禁分诊带前缀跑 87 号", triage, commit + "bash .claude/gate.d/87-replay.sh", 0),
            ("执行员跑 12 号（轻阶段）", "experiment-runner", "nice -n 19 bash .claude/gate.d/12-no-prime-marks.sh", 0),
            ("书记员跑 20 号（轻阶段）", scribe, "nice -n 19 bash .claude/gate.d/20-kb-shape.sh", 0),
            ("通用 agent 跑 63 号（轻阶段）", "general-purpose", "bash .claude/gate.d/63-agent-write-scope.sh", 0),
            ("主 agent 不带前缀跑 47 号（轻阶段）", None, "bash .claude/gate.d/47-research-script-selftests.sh", 0),
            ("实现员跑自己动到的测试目标", writer,
             "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --test second_transaction_step_one_overwrite", 0),
            ("实现员 clippy --all-targets", writer, "cargo clippy --all-targets -- -D warnings", 0),
            ("实现员 build --all-targets", writer, "cargo build --offline --all-targets", 0),
            ("实现员 grep 层 0 的名字", writer, "grep -n layer0 x", 0),
            ("实现员读 54 号脚本", writer, "cat .claude/gate.d/54-layer0-replay.sh", 0),
            ("实现员 echo 引号里的命令", writer, 'echo "cargo test --all"', 0),
            ("实现员写进笔记的 heredoc 正文", writer, "cat > note.md <<'EOF'\ncargo test --all\nEOF", 0),
            ("实现员 bash -n 只查语法", writer, "bash -n .claude/scripts/check.sh", 0),
            ("实现员 -p harness --lib", writer, "bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-harness --lib", 0),
            ("实现员 mutate.sh 跑 research 的表", writer, "cd research && bash scripts/mutate.sh e1 e7-index-bench/src/bin/e1.rs mutations/e1.tsv", 0),
            ("实现员 capped.sh --selftest", writer, "bash research/scripts/capped.sh --selftest", 0),
            ("实现员 cd 进没有层 0 的包裸跑", writer, "cd crates/singlefs-core && bash ../../research/scripts/run-with-memory-cap.sh 4G cargo test", 0),
            ("实现员 git diff 点名重型脚本", writer, "git diff -- crates/mutations.tsv .claude/gate.d/54-layer0-replay.sh", 0),
            ("执行员在 research 里跑自己的 bin", "experiment-runner",
             "cd research && bash scripts/run-with-memory-cap.sh 16G cargo test --release --bin e160-random-small-read-share", 0),
            # 跑编译出来的代码经没经内存包装（与重型不重型无关）：子 agent 不经它的拒，经它包着的（连同 bash -c、它起的脚本）放行，主 agent 不判
            ("实现员不经内存包装跑自己的测试目标", writer, "cargo test -p singlefs-harness --test second_transaction_step_one_overwrite", 2),
            ("执行员不经内存包装 cargo run 实验二进制", "experiment-runner", "cd research && cargo run --release --bin e160-random-small-read-share", 2),
            ("实现员 cargo t 简写不经内存包装", writer, "cargo t -p singlefs-core --lib", 2),
            ("执行员直接执行实验二进制不经内存包装", "experiment-runner", "./research/target/release/e142_region_diff_independent", 2),
            ("实现员经 capped.sh 限了线程、没限内存", writer, "bash research/scripts/capped.sh 4 cargo test -p singlefs-core --lib", 2),
            ("实现员包装外面的命令替换不算在包装里", writer, "bash research/scripts/run-with-memory-cap.sh 4G echo \"$(cargo test -p singlefs-core --lib)\"", 2),
            ("实现员命令替换里不经内存包装跑", writer, "out=\"$(cargo test -p singlefs-core --lib 2>&1)\"", 2),
            ("实现员经内存包装再经 capped.sh", writer, "bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/capped.sh 4 cargo test -p singlefs-core --lib", 0),
            ("实现员 capped.sh 在外、内存包装在里", writer, "capped.sh 4 run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员 timeout 套在内存包装外面", writer, "timeout 900 bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --lib", 0),
            ("实现员内存包装里 bash -c 跑", writer, "bash research/scripts/run-with-memory-cap.sh 4G bash -c 'cd crates/singlefs-core && cargo test --lib'", 0),
            ("实现员经内存包装直接执行测试二进制", writer, "bash research/scripts/run-with-memory-cap.sh 4G ./target/debug/deps/core_contract-0123456789abcdef", 0),
            ("实现员只编不跑 cargo test --no-run", writer, "cargo test --no-run -p singlefs-core", 0),
            ("实现员 cargo build 不用经内存包装", writer, "cargo build --release -p singlefs-core", 0),
            ("实现员起的脚本里有不经内存包装的 cargo test", writer, "bash unwrapped.sh", 2, 0, 1),
            ("实现员经内存包装起那份脚本", writer, "bash research/scripts/run-with-memory-cap.sh 4G bash unwrapped.sh", 0, 0, 1),
            ("实现员 heredoc 写出不经内存包装的 cargo run 再起它", writer,
             "cat > gen-unwrapped.sh <<'EOF'\ncargo run --release --bin e160-random-small-read-share\nEOF\nbash gen-unwrapped.sh", 2, 0, 1),
            ("主 agent 不经内存包装跑测试目标：这一道不判主 agent", None, "cargo test -p singlefs-core --lib", 0),
            ("崩溃验证员带前缀不经内存包装跑层 0 测试目标", crash, commit + "cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0", 2),
            ("崩溃验证员带前缀经内存包装跑 54 号全量", crash,
             commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
            ("崩溃验证员带前缀经内存包装跑 55 号", crash, commit + "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/55-qemu-first-transaction.sh", 0),
            ("崩溃验证员带前缀经内存包装跑 57 号", crash, commit + "bash research/scripts/run-with-memory-cap.sh 8G bash .claude/gate.d/57-lkmm.sh", 0),
            ("门禁分诊带前缀经内存包装跑 gate.sh --staged", triage, commit + "bash research/scripts/run-with-memory-cap.sh 24G bash .claude/scripts/gate.sh --staged", 0),
            ("主 agent 带前缀经内存包装跑 54 号全量", None,
             commit + "bash research/scripts/run-with-memory-cap.sh 16G bash /tmp/wt/.claude/gate.d/54-layer0-replay.sh --full /tmp/wt", 0),
            ("实现员经内存包装跑 54 号照拒（重型那一道）", writer, "bash research/scripts/run-with-memory-cap.sh 16G bash .claude/gate.d/54-layer0-replay.sh", 2),
            ("实现员 gate-staged.sh --selftest（仓内那份按名字判，不读正文）", writer, "bash research/scripts/gate-staged.sh --selftest", 0),
            # 写进脚本文件再执行：s.sh 第 4 行是 cargo test --all（上面第一例，实现员会被拒的那一条）。末两列是该记几条检出、该读进去几份脚本
            ("实现员 bash 起含它的脚本", writer, "bash s.sh", 2, 0, 1),
            ("实现员 ./ 直接执行含它的脚本", writer, "./s.sh", 2, 0, 1),
            ("实现员经 capped.sh 4 bash 起含它的脚本", writer, "capped.sh 4 bash s.sh", 2, 0, 1),
            ("实现员 source 含它的脚本", writer, "source s.sh", 2, 0, 1),
            ("实现员 . 读进含它的脚本、nice 与 timeout 包一层", writer, "nice -n 19 timeout 600 . s.sh", 2, 0, 1),
            ("实现员 cd 之后按绝对路径起含它的脚本、输出落文件", writer, f"cd {work} && bash {work}/s.sh > {work}/chain.log 2>&1", 2, 0, 1),
            ("实现员起的脚本里再调含它的脚本", writer, "bash outer.sh", 2, 0, 2),
            ("实现员起仓外同名的 mutate.sh（不按名字判，读正文）", writer, "bash elsewhere/mutate.sh x y mutations/e1.tsv", 2, 0, 1),
            ("实现员起的脚本那一行自己带前缀也拒", writer, "bash own-prefix.sh", 2, 0, 1),
            ("主 agent 带放行前缀执行含它的脚本", None, commit + "bash s.sh", 0, 0, 1),
            ("主 agent export 之后执行含它的脚本", None, "export SINGLEFS_HEAVY_TESTS=commit; ./s.sh", 0, 0, 1),
            ("主 agent 不带前缀执行含它的脚本", None, "bash s.sh", 2, 0, 1),
            ("主 agent 执行的脚本那一行自己带前缀", None, "bash own-prefix.sh", 0, 0, 1),
            ("实现员起不含重型命令的脚本", writer, "bash light.sh", 0, 0, 1),
            ("实现员起调回自己的脚本（只读一遍）", writer, "bash loop.sh", 0, 0, 1),
            ("实现员 capped.sh 后面不带斜杠的命令词从 PATH 找，不读当前目录的同名脚本", writer, "bash research/scripts/capped.sh 4 s.sh", 0, 0, 0),
            ("实现员 ./ 执行 python 程序不读（射程之外）", writer, "./tool", 0, 0, 0),
            ("实现员 ./ 执行编译出来的程序不读", writer, "./prog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的编译产物：不读也不记", writer, "./bigprog", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 python 程序：不读也不记", writer, "./bigtool", 0, 0, 0),
            ("实现员 ./ 执行超过读的上限的 shell 脚本：放行并记检出", writer, "./big.sh", 0, 1, 0),
            ("实现员起 doc-lint.sh 第 43 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-43.sh", 0, 0, 1),
            ("实现员起 doc-lint.sh 第 70 行那种命令替换：片段不当路径", writer, "bash doc-lint-line-70.sh", 0, 0, 1),
            ("实现员起的脚本里命令替换、反引号、算术后面接路径：片段不当路径", writer, "bash substitution-fragments.sh", 0, 0, 1),
            ("实现员起的脚本里命令替换里跑 --all", writer, "bash substitution-heavy.sh", 2, 0, 1),
            ("实现员起的脚本里 command -v herd7：查名字不是跑", writer, "bash command-lookup.sh", 0, 0, 1),
            ("实现员起的脚本里数组赋值的元素：不当命令、不当路径", writer, "bash array-assignment.sh", 0, 0, 1),
            ("实现员直接跑 command -v herd7", writer, "command -v herd7 >/dev/null && echo 在", 0),
            ("实现员 command herd7（不带 -v）照拒", writer, "command herd7 -version", 2),
            ("实现员 bash 起一个目录：不读也不记", writer, "bash crates", 0),
            # 进程替换 `<(…)`：括号里的命令另按一条完整的命令判，`)` 之后的词是外层命令的参数（第一例是现场误拒的原句）
            ("实现员 diff <(git show …check.sh) check.sh：) 之后的 check.sh 是 diff 的参数，放行、不记检出", writer,
             "diff <(git show HEAD:.claude/scripts/check.sh) .claude/scripts/check.sh", 0, 0, 0),
            ("实现员 diff <(bash check.sh) x：括号里真执行全量测试，照拒", writer, "diff <(bash .claude/scripts/check.sh) x", 2),
            # 喂给 python 的 heredoc 正文不扫：那一行别处出现 x.sh、前一条是 bash，都不算喂给 shell；bash light.sh 那一条证明这一例真的判过
            ("实现员同一行里起 .sh 之后把 python 源码喂给 python3 -：正文不当命令", writer,
             "sed -n 1p light.sh > body.py && bash light.sh && python3 - <<'PY'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nusage = \"\"\"\ncargo test --all\n\"\"\"\nPY", 0, 0, 1),
            # 喂给 bash 的 heredoc 里出现 python 那种写法：`source =` 不算读脚本，带逗号的词不当路径
            ("实现员喂给 bash 的正文里有 source = 与带逗号的路径：不当脚本", writer,
             "bash light.sh; bash <<'EOF'\nsource = open(\"body.py\", encoding=\"utf-8\").read()\n"
             "body = open(\"/tmp/claude-1000/nowhere/hook_body.py\", encoding=\"utf-8\")\nEOF", 0, 0, 1),
            ("实现员 bash 起通配写出来的脚本路径：不当路径、不记检出", writer, "bash ./chain-*.sh", 0, 0, 0),
            ("实现员起不存在的脚本：放行并记检出", writer, "bash missing.sh", 0, 1, 0),
            ("实现员 ./ 执行不存在的脚本：放行并记检出", writer, "./missing.sh", 0, 1, 0),
            ("实现员 bash 起不是文本的文件：放行并记检出", writer, "bash blob.bin", 0, 1, 0),
            ("实现员起超过上限的脚本：放行并记检出", writer, "bash big.sh", 0, 1, 0),
            ("实现员起的脚本嵌套 7 层：读到第 6 层、看不全，放行并记检出", writer, "bash deep1.sh", 0, 1, 6),
            # 同一条命令里先写出、再执行的脚本：拿写出的内容判，不记「不存在」。heredoc 正文里是第一例那条被拒的 cargo test --all
            ("实现员 cat > 文件 <<EOF 写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\ncargo test --all\nEOF\nbash gen.sh", 2, 0, 1),
            ("实现员 cat <<EOF > 文件 写出含 --all 的脚本、chmod 之后 ./ 执行", writer,
             "cat <<'EOF' > gen2.sh\ncargo test --all\nEOF\nchmod +x gen2.sh && ./gen2.sh", 2, 0, 1),
            ("实现员 tee 文件 <<EOF 写出含 --all 的脚本再 source", writer,
             "tee gen3.sh <<'EOF' >/dev/null\ncargo test --all\nEOF\nsource gen3.sh", 2, 0, 1),
            ("实现员 <<-EOF（去制表符）写出含 --all 的脚本再 bash 起它", writer,
             "cat > gen4.sh <<-EOF\n\tcargo test --all\n\tEOF\nbash gen4.sh", 2, 0, 1),
            ("实现员 cd 进子目录写出、回来按相对路径起", writer,
             "mkdir -p gen-dir && cd gen-dir && cat > here.sh <<'EOF'\ncargo test --all\nEOF\ncd .. && bash gen-dir/here.sh", 2, 0, 1),
            ("实现员 heredoc 盖掉盘上不含重型命令的 light.sh 再起它：判写出的内容", writer,
             "cat > light.sh <<'EOF'\ncargo test --all\nEOF\nbash light.sh", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里再写出、再起含 --all 的脚本", writer,
             "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\ncargo test --all\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh", 2, 0, 2),
            ("实现员 bash -c 里写出含 --all 的脚本再起它", writer,
             "bash -c 'cat > gen-c.sh <<EOF\ncargo test --all\nEOF\nbash gen-c.sh'", 2, 0, 1),
            ("实现员命令替换里写出含 --all 的脚本再起它", writer,
             "out=\"$(cat > gen-sub.sh <<'EOF'\ncargo test --all\nEOF\nbash gen-sub.sh)\"", 2, 0, 1),
            ("实现员 heredoc 写出的脚本里写出另一份，外层后面再起那一份", writer,
             "cat > gen-maker.sh <<'EOF'\ncat > gen-made.sh <<'IN'\ncargo test --all\nIN\nEOF\nbash gen-maker.sh && bash gen-made.sh", 2, 0, 2),
            ("实现员先写出含 --all 的一行、再 >> 追加一行轻的：前面那行还在", writer,
             "cat > gen8.sh <<'EOF'\ncargo test --all\nEOF\ncat >> gen8.sh <<'EOF'\necho 尾\nEOF\nbash gen8.sh", 2, 0, 1),
            ("实现员 tee -a 往盘上含 --all 的 s.sh 追加一行轻的：盘上那几行还在", writer,
             "tee -a s.sh <<'EOF'\necho 尾\nEOF\nbash s.sh", 2, 0, 1),
            ("实现员 cat -n 带选项写出：认不出，照旧记检出", writer,
             "cat -n > gen-numbered.sh <<'EOF'\ncargo test --all\nEOF\nbash gen-numbered.sh", 0, 1, 0),
            ("实现员 heredoc 写出不含重型命令的脚本再起它：放行、不记检出", writer,
             "cat > gen-light.sh <<'EOF'\nbash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test core_contract\necho \"cargo test --all\"\nEOF\n"
             "bash gen-light.sh", 0, 0, 1),
            ("主 agent 带前缀起 heredoc 写出的含 --all 的脚本", None,
             "cat > gen5.sh <<'EOF'\ncargo test --all\nEOF\n" + commit + "bash gen5.sh", 0, 0, 1),
            ("实现员 heredoc 写出 python 程序再 ./ 执行：不读也不记", writer,
             "cat > gen.py <<'EOF'\n#!/usr/bin/env python3\nprint('cargo test --all')\nEOF\n./gen.py", 0, 0, 0),
            ("实现员 cp 盘上含 --all 的 s.sh 出来再起：读源文件", writer, "cp s.sh copied.sh && bash copied.sh", 2, 0, 1),
            ("实现员 cp -f 到目录里再起：读源文件", writer, "cp -f s.sh elsewhere/ && bash elsewhere/s.sh", 2, 0, 1),
            ("实现员 cp 盘上不含重型命令的 light.sh 出来再起：放行、不记检出", writer, "cp light.sh copied-light.sh && ./copied-light.sh", 0, 0, 1),
            ("实现员 cp 同一条命令里 heredoc 写出的脚本再起", writer,
             "cat > gen6.sh <<'EOF'\ncargo test --all\nEOF\ncp gen6.sh gen7.sh && bash gen7.sh", 2, 0, 1),
            ("实现员 cp 的源不在：照旧记检出", writer, "cp nowhere.sh copied-missing.sh && bash copied-missing.sh", 0, 1, 0),
            ("实现员 printf 写出再起：认不出，照旧记检出", writer, "printf 'cargo test --all\\n' > printed.sh && bash printed.sh", 0, 1, 0),
            # 直接执行的只读 shell 脚本：`#!` 指到 shell，或没有 `#!` 而扩展名是 .sh；别的不读、不记
            ("实现员 ./ 直接执行 .rs 文件：不当脚本读、不记", writer, "./crates/singlefs-checker/src/walk.rs", 0, 0, 0),
            ("实现员 ./ 直接执行 .md 文件：不当脚本读、不记", writer, "./notes.md", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、没 #! 的文本：不当脚本读、不记", writer, "./plain-text", 0, 0, 0),
            ("实现员 ./ 直接执行还不在的 .rs：不记「不存在」", writer, "./crates/singlefs-checker/src/missing.rs", 0, 0, 0),
            ("实现员 ./ 直接执行还不在、没扩展名的：不记「不存在」", writer, "bash research/scripts/run-with-memory-cap.sh 4G ./target/release/not-built-yet", 0, 0, 0),
            ("实现员 ./ 直接执行没扩展名、#! 指到 bash 的脚本：读进去照拒", writer, "./runner", 2, 0, 1),
            ("实现员 ./ 直接执行 .py 名字、#! 指到 sh 的脚本：读进去照拒", writer, "./mislabeled.py", 2, 0, 1),
            ("实现员 bash 起 .rs 文件：显式交给 shell 的照读，里面反引号括着的 .rs 不再读", writer, "bash crates/singlefs-checker/src/walk.rs", 2, 0, 1),
            ("实现员复现输入那一条（反引号里的 walk.rs）：放行、不记检出，只读 doc-lint.sh", writer, SLOW_REPRODUCTION_COMMAND, 0, 0, 1),
            # 同一份脚本一次判定里只读一遍、只判一遍；换了当前目录、前缀的值、写出的内容，或那一遍撞上嵌套上限的，重判
            ("实现员同一条命令里起含 --all 的 s.sh 两遍：只判一遍", writer, "bash s.sh; ./s.sh", 2, 0, 1),
            ("实现员起的脚本里三种起法各起一遍 s.sh：s.sh 只判一遍", writer, "bash twice.sh", 2, 0, 2),
            ("主 agent 两遍都带前缀起 s.sh：只判一遍、放行", None, commit + "bash s.sh; " + commit + "bash s.sh", 0, 0, 1),
            ("主 agent 先带前缀、再不带起 s.sh：前缀的值不同重判，第二遍照拒", None, commit + "bash s.sh; bash s.sh", 2, 0, 2),
            ("实现员先起盘上轻的 inner-rel.sh、再 heredoc 盖成含 --all 的再起：写出的变了重判", writer,
             "bash inner-rel.sh; cat > inner-rel.sh <<'EOF'\ncargo test --all\nEOF\nbash inner-rel.sh", 2, 0, 2),
            ("实现员先起里面起了不存在脚本的 has-missing.sh、再经 outer-missing.sh 起它：检出照复用的那一遍记两条", writer,
             "bash has-missing.sh; bash outer-missing.sh", 0, 2, 2),
            ("实现员同一份 rel.sh 换个当前目录再起：按相对路径起的脚本变了，重判", writer, "bash rel.sh; cd sub && bash ../rel.sh", 2, 0, 4),
            ("实现员先经 7 层嵌套碰到 deep6.sh、再直接起它：那一遍撞上嵌套上限，重判", writer, "bash deep1.sh; bash deep6.sh", 2, 1, 8),
            ("实现员先起 cyc-a.sh（里面绕回自己）、再起 cyc-b.sh：cyc-b.sh 那一遍绕回外层，重判", writer, "bash cyc-a.sh; bash cyc-b.sh", 2, 0, 3),
        ]
        for case in cases:
            label, agent_type, command, want = case[:4]
            want_notices, want_read = case[4:6] if len(case) > 4 else (0, 0)
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            decision = decide(hook_input, work)
            results.append((label, want, decision.code))
            results.append((f"{label}：记几条检出", want_notices, len(decision.notices)))
            results.append((f"{label}：读进去几份脚本", want_read, len(decision.scripts_read)))
        # 同一份文件一次判定里只从盘上读一遍；直接执行的 .rs 要读开头判是不是脚本，判完不当脚本读，它里面括着的文件不碰
        def disk_reads(command, relative, agent_type=writer):
            hook_input = {"tool_name": "Bash", "cwd": work, "tool_input": {"command": command}}
            if agent_type:
                hook_input["agent_type"] = agent_type
            return decide(hook_input, work).files_read.count(os.path.realpath(os.path.join(work, relative)))
        for label, command, relative, agent_type, want in [
                ("同一条命令里起 s.sh 两遍：s.sh 从盘上读一遍", "bash s.sh; ./s.sh", "s.sh", writer, 1),
                ("起的脚本里三种起法各起一遍 s.sh：s.sh 从盘上读一遍", "bash twice.sh", "s.sh", writer, 1),
                ("先带前缀、再不带起 s.sh：重判也不重读", commit + "bash s.sh; bash s.sh", "s.sh", None, 1),
                ("直接执行 .rs 两遍：判它是不是脚本要看开头，开头也只读一遍", "./crates/singlefs-checker/src/walk.rs; ./crates/singlefs-checker/src/walk.rs",
                 "crates/singlefs-checker/src/walk.rs", writer, 1),
                ("复现输入那一条：walk.rs 只看开头，它括着的 image.rs 不碰", SLOW_REPRODUCTION_COMMAND, "crates/singlefs-checker/src/image.rs", writer, 0)]:
            results.append((f"{label}（读盘次数）", want, disk_reads(command, relative, agent_type)))
        # 复现输入那一条要在 SLOW_REPRODUCTION_SECONDS 秒内判完
        started = time.monotonic()
        decide({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": SLOW_REPRODUCTION_COMMAND}}, work)
        elapsed = time.monotonic() - started
        results.append((f"复现输入那一条在 {SLOW_REPRODUCTION_SECONDS} 秒内判完（这一遍 {elapsed:.3f} 秒）", 1, int(elapsed < SLOW_REPRODUCTION_SECONDS)))
        # 拒绝说明写出脚本路径与行号：直接起的写到 s.sh 第 4 行，嵌套的连外层起它的那一行一起写
        for label, command, location in [("拒绝说明写出脚本路径与行号", "bash s.sh", f"脚本 {work}/s.sh:4 里的 cargo test"),
                                         ("嵌套的拒绝说明写出每一层的路径与行号", "bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 cargo test"),
                                         ("命令替换里的拒绝说明写出它所在那一行", "bash substitution-heavy.sh", f"脚本 {work}/substitution-heavy.sh:3 里的 cargo test"),
                                         ("heredoc 写出的脚本，拒绝说明写出它的路径与正文里的行号",
                                          "cat > gen.sh <<'EOF'\n#!/usr/bin/env bash\ncargo test --all\nEOF\nbash gen.sh", f"脚本 {work}/gen.sh:2 里的 cargo test"),
                                         ("heredoc 写出的脚本里再写出的，逐层写出路径与行号",
                                          "cat > gen-outer.sh <<'EOF'\ncat > gen-inner.sh <<'IN'\ncargo test --all\nIN\nbash gen-inner.sh\nEOF\nbash gen-outer.sh",
                                          f"脚本 {work}/gen-outer.sh:4 → {work}/gen-inner.sh:1 里的 cargo test"),
                                         ("复用上一遍的结果，拒绝说明照这一处的路径与行号写",
                                          "bash s.sh; bash outer.sh", f"脚本 {work}/outer.sh:3 → {work}/s.sh:4 里的 cargo test"),
                                         ("绕回外层的那一遍不复用，从别处起时照读回来", "bash cyc-a.sh; bash cyc-b.sh",
                                          f"脚本 {work}/cyc-b.sh:1 → {work}/cyc-a.sh:2 里的 cargo test")]:
            message = decide({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": command}}, work).message or ""
            results.append((label, 1, int(location in message and "这一步写在脚本里" in message)))
        # 复用上一遍的检出，照这一处的路径与行号写
        reused_notices = decide({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                 "tool_input": {"command": "bash has-missing.sh; bash outer-missing.sh"}}, work).notices
        results.append(("复用上一遍的检出，照这一处的路径与行号写", 1,
                        int(any(f"脚本 {work}/outer-missing.sh:2 → {work}/has-missing.sh:1 里" in notice for notice in reused_notices))))
        # 切词：source / . 后面跟的像一个路径才算读脚本（python 源码里 `source = …` 那种不算）
        for label, command, want in [("切词：source = … 不算读脚本", "source = open", [("source", None)]),
                                     ("切词：. = … 不算读脚本", ". = x", [(".", None)]),
                                     ("切词：. ./env.sh 算读脚本", ". ./env.sh", [("./env.sh", "source")])]:
            got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position(command, work).commands]
            results.append((label, want, got))
        # 切词：进程替换的另一种写法 `>(…)` 同 `<(…)`：里面的命令另判，`)` 之后的词是外层命令的参数
        got = [(found.words[0], found.launcher) for found in shell_words.commands_at_command_position("tee >(bash run.sh) out.log", work).commands]
        results.append(("切词：tee >(bash run.sh) out.log 里 run.sh 另判、out.log 是 tee 的参数", [("run.sh", "shell"), ("tee", None)], got))
        # 切词：heredoc 正文留不留，看它挂在哪条命令上（从标准输入读命令的 bash / sh 才留），不看同一行别处有没有 shell 的名字
        for label, command, kept in [("切词：前一条起了 x.sh、heredoc 喂给 python3 -：正文剥掉", "bash light.sh && python3 - <<'PY'\nBODY\nPY", False),
                                     ("切词：heredoc 喂给 bash：正文留着", "bash <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 capped.sh 4 bash -s：正文留着", "capped.sh 4 bash -s -- a <<'EOF'\nBODY\nEOF", True),
                                     ("切词：heredoc 喂给 bash -c：正文剥掉", "bash -c 'cat' <<'EOF'\nBODY\nEOF", False),
                                     ("切词：heredoc 喂给 bash 起的脚本：正文剥掉", "bash light.sh <<'EOF'\nBODY\nEOF", False)]:
            results.append((label, kept, "BODY" in shell_words.strip_data_heredocs(command)))
        # 共用判定模块自己的自检（cargo 命令行与直接执行的测试二进制两种输入）
        results.append(("lib_heavy_tests.py 的自检", 0, heavy_tests.selftest()))
        # 走真实入口：从标准输入喂 JSON，拒绝退出码 2、stderr 带出路，放行退出码 0；拒绝落进检出记录
        script = os.path.join(hook_dir, "heavy-test-guard.sh")
        detections = os.path.join(work, "detections.jsonl")
        environment = dict(os.environ, CLAUDE_PROJECT_DIR=work, AGENT_HOOK_DETECTIONS=detections)
        def count_detections():
            return sum(1 for _ in open(detections, encoding="utf-8")) if os.path.exists(detections) else 0
        refused = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:实现员跑 --all 拒绝（退出码 2）", 2, refused.returncode))
        results.append(("stdin:拒绝时 stderr 写了出路与前缀", 1,
                        int("→" in refused.stderr and "SINGLEFS_HEAVY_TESTS=commit" in refused.stderr and "弹窗" in refused.stderr)))
        allowed = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "tool_input": {"command": commit + "cargo test --all"}}))
        results.append(("stdin:主 agent 带前缀放行（退出码 0）", 0, allowed.returncode))
        recorded = count_detections()
        results.append(("stdin:拒绝落进检出记录", 1, recorded))
        # 走真实入口：实现员起含被拒命令的脚本，退出码 2、stderr 写出脚本路径与行号
        in_script = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                   input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash s.sh"}}))
        results.append(("stdin:实现员起含被拒命令的脚本拒绝（退出码 2）", 2, in_script.returncode))
        results.append(("stdin:脚本里的拒绝 stderr 写出脚本路径与行号", 1, int(f"{work}/s.sh:4" in in_script.stderr)))
        # 走真实入口：实现员不经内存包装跑测试目标，退出码 2、stderr 写出经包装跑的写法
        uncapped_run = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                      input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer,
                                                        "tool_input": {"command": "cargo test -p singlefs-core --lib"}}))
        results.append(("stdin:实现员不经内存包装跑测试目标拒绝（退出码 2）", 2, uncapped_run.returncode))
        results.append(("stdin:不经内存包装的拒绝 stderr 写了出路", 1,
                        int("→" in uncapped_run.stderr and "run-with-memory-cap.sh <上限> <命令>" in uncapped_run.stderr
                            and "不经内存包装跑编译出来的代码被拒" in uncapped_run.stderr)))
        recorded = count_detections()
        # 走真实入口：脚本文件不存在时照常放行（退出码 0），stderr 点名那个文件，并记一条检出
        missing = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                 input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "bash missing.sh"}}))
        results.append(("stdin:脚本文件不存在照常放行（退出码 0）", 0, missing.returncode))
        results.append(("stdin:脚本文件不存在时 stderr 点名它、记一条检出", 1,
                        int(f"{work}/missing.sh" in missing.stderr and count_detections() == recorded + 1)))
        recorded = count_detections()
        # 走真实入口：同一条命令里 heredoc 写出不含重型命令的脚本再起它，照常放行（退出码 0），stderr 不报、不记检出
        written_light = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen-light.sh <<'EOF'\necho 轻\nEOF\nbash gen-light.sh"}}))
        results.append(("stdin:heredoc 写出的轻脚本照常放行（退出码 0）", 0, written_light.returncode))
        results.append(("stdin:heredoc 写出的轻脚本 stderr 不报「不存在」、不记检出", 1,
                        int("不存在" not in written_light.stderr and count_detections() == recorded)))
        written_heavy = subprocess.run(["bash", script], capture_output=True, text=True, env=environment,
                                       input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {
                                           "command": "cat > gen.sh <<'EOF'\ncargo test --all\nEOF\nbash gen.sh"}}))
        results.append(("stdin:heredoc 写出的含 --all 的脚本拒绝（退出码 2）", 2, written_heavy.returncode))
        results.append(("stdin:heredoc 写出的脚本的拒绝 stderr 写出它的路径与行号", 1, int(f"{work}/gen.sh:1" in written_heavy.stderr)))
        recorded = count_detections()
        # 走真实入口：hook 旁边没有共用模块时照常放行（退出码 0），stderr 点名两份模块，并记一条检出
        lonely_hook_directory = os.path.join(work, "hook-without-shared-modules")
        os.makedirs(lonely_hook_directory)
        shutil.copy(script, lonely_hook_directory)
        without_library = subprocess.run(["bash", os.path.join(lonely_hook_directory, "heavy-test-guard.sh")], capture_output=True, text=True, env=environment,
                                         input=json.dumps({"tool_name": "Bash", "cwd": work, "agent_type": writer, "tool_input": {"command": "cargo test --all"}}))
        results.append(("stdin:读不到共用模块照常放行（退出码 0）", 0, without_library.returncode))
        results.append(("stdin:读不到共用模块时 stderr 点名两份模块、记一条检出", 1,
                        int("lib_heavy_tests.py" in without_library.stderr and "lib_shell_words.py" in without_library.stderr
                            and count_detections() == recorded + 1)))
    finally:
        shutil.rmtree(work)
    failures = [item for item in results if item[1] != item[2]]
    for label, want, got in failures:
        print(f"  ✗ 自检：{label} 应当是 {want}，实际 {got}")  # gate-lint:detail
    if failures:
        print("    → 看 heavy_uses()、script_path_of()、executed_file_is_shell_script()、file_on_disk()、script_text()、command_lines()、refusal_reason() "
              "与共用的 lib_heavy_tests.py、lib_shell_words.py；"
              "HEAVY_TEST_GUARD_DISABLE_CHECK 设着的话这里本来就该红")
        return 1
    refused_count = sum(1 for item in results if item[1] == 2)
    print(f"  ✓ 自检通过（查了 {len(results)} 种，其中该拒 {refused_count} 种）：子 agent 跑层 0、全量测试、check.sh、整轮门禁、QEMU、herd7、crates 变异整表拒绝，"
          "崩溃验证员与门禁分诊带前缀跑各自那一份放行、越出那一份或不带前缀拒绝，主 agent 带 commit / user-request 放行、不带或带别的值拒绝；"
          "写进脚本再执行的（bash / ./ / capped.sh / source / 嵌套）读进去照判、拒绝写出路径与行号，脚本读不到或看不全的放行并记检出；"
          "同一条命令里 heredoc（cat / tee）写出或 cp 拷出再执行的拿写出的内容判、不记「不存在」，认不出的写法照旧记检出；"
          "直接执行的只读 `#!` 指到 shell 的与没 `#!` 的 .sh（.rs、.md、没扩展名的文本不读不记），同一份脚本一次判定里只读一遍、只判一遍；"
          "子 agent 不经 run-with-memory-cap.sh 跑 cargo test / run / bench 与 cargo 编出来的二进制拒绝、经它包着的（连同 bash -c 与它起的脚本）放行；"
          "只跑动到的测试目标、fmt / clippy / build、54 / 55 / 57 / 59 / 87 之外的门禁阶段、把名字当参数的放行")
    return 0

def main():
    hook_dir = sys.argv[1]
    if len(sys.argv) > 2 and sys.argv[2] == "--selftest":
        return selftest(hook_dir)
    try:
        hook_input = json.load(sys.stdin)
    except Exception:
        return 0
    if not isinstance(hook_input, dict):
        return 0
    detections_path = os.environ.get("AGENT_HOOK_DETECTIONS") or DEFAULT_DETECTIONS
    if heavy_tests is None:
        warning = f"heavy-test-guard.sh {shared_library_problem(hook_dir)}，这条命令没判、照常执行"
        print(f"  ! {warning}", file=sys.stderr)
        record_detection(hook_input, [warning], detections_path)
        return 0
    project_root = os.environ.get("CLAUDE_PROJECT_DIR") or os.path.dirname(os.path.dirname(hook_dir))
    try:
        decision = decide(hook_input, project_root)
    except Exception as error:
        print(f"  ! heavy-test-guard.sh 没判成（{error!r}），这条命令照常执行", file=sys.stderr)
        return 0
    findings = [f"重型测试闸没判全：{notice}" for notice in decision.notices]
    for notice in decision.notices:
        print(f"  ! heavy-test-guard.sh：{notice}；这条命令照常执行，记了一条检出", file=sys.stderr)
    if decision.code:
        print(decision.message, file=sys.stderr)
        findings.append(decision.message.splitlines()[0].removeprefix("✗ "))
    if findings:
        record_detection(hook_input, findings, detections_path)
    return decision.code

sys.exit(main())
PY
