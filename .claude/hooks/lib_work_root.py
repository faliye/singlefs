"""工作根：子 agent 的草稿目录、报告、补丁与冻结副本，钩子与看门狗的状态和检出记录，放在哪只在这一份里定义；Claude Code 的会话临时目录根也只在这里定义；
数据根（~/.local/share/singlefs 那一层）下哪几样两台共享、哪几样按台各一份，也只在这里定义。

数据根：环境变量 SINGLEFS_DATA_ROOT 设了就用它（统一任务服务给每次尝试带上服务那一台的数据根真路径，派到本机与第二台都带；要写绝对路径，不是的报错）；
  没设取多机配置里的 SHARED_DATA_ROOT（配置是 SINGLEFS_MULTI_HOST_CONFIG 指的那一份，没设取这份库所在仓的主工作树根上的 multi-host.env：
  主工作树照 <仓根>/.git 认，git worktree 的 .git 文件照它的 gitdir 与 commondir 认，不起 git）：要写绝对路径、落在同一份配置 NFS_SHARED_ROOTS 某一段下
  （等于那一段或在它下面），不合的报错。两样都没有就报错（DataRootUnconfigured，ValueError 的子类，说明里写出路），不补缺省、不退回家目录：
  环境相关的一切只由多机配置定，没写就是没有。数据根经 NFS 两台挂在同一绝对路径上，第二台的 ~ 是另一个目录：
  经 ~、$HOME 或 expanduser 拼出来的路径在第二台落进它本地那一份，两台读不到同一份。
  共享的那几样一律经 shared_data_path(<名>) 取（shell：python3 <仓根>/.claude/hooks/lib_work_root.py shared <名>；整个数据根：… data-root），
  不写成 ~/、$HOME/ 起头的字面量（门禁 code-tooling 的 shared-data-root-literals 格判，名单取 SHARED_DATA_ENTRIES）；按台的照旧经 $HOME 取。
  名字                          归类  为什么
  work                          共享  工作根（见下）：草稿、报告、补丁、日志、实验的片与运行目录（崩溃放量的运行目录是它下面的 crash-amplification），派到哪台跑的都要两台读得到
  last-runs                     共享  每道门禁与入口跑完写的上次红的清单，范围声明闸的拒绝说明读它
  cross-run-cache               共享  门禁开跑时刻文件 gate-run-times（门禁 test-environment 的 machine-clean 格写、读）
  broad-mutation-units          共享  广谱变异的库目录（判定表、清单表、覆盖表），提交门禁 broad-mutation 读它的汇总
  reuse                         共享  E171（三oracle合谋测试）装置的复用库
  gate-staged-logs              共享  scripts/gate/gate-staged.sh 的两份日志
  doc-consolidate-migrated.tsv  共享  scripts/kb/doc-consolidate.py 的迁过台账
  mutation-run-records          共享  scripts/mutation/crates-mutation-rows.py 的运行记录（timings/ 下的用时表与 target-occupancy.tsv 占盘时序），派到哪台跑都读写同一份
  fs-refs                       共享  外部文献与参照源码的固定点（scripts/kb/fetch-refs.sh 取回，scripts/kb/verify-citations.sh、scripts/kb/pdf-text.py 与门禁 doc-registries 的 citations 格读）
  kernel-tree                   共享  verify-citations.sh 核内核源码行的那棵内核树（非固定点），两台核同一棵
  memory-peaks.tsv              共享  内存峰值表两台读写同一份，每行带量它的是哪一台（scripts/hosts/memory_peaks_lib.py），换一台不作数的那一层按行上的台判
  formal-tools                  共享  Verus 与 z3（scripts/gate/verus-proof-check.py 默认从这里取安装目录）：两台同一绝对路径读同一份，第二台不另装
  rocksdb-prebuilt              按台  预编的 RocksDB 静态库照那一台的编译器与系统库编，换一台不能用
  qemu                          按台  打过补丁、照那一台的系统编出来的 QEMU
  task-server-builds            按台  统一任务服务在那一台给自己编的程序
  gpu-card-health               按台  那一台的显卡与内核日志游标，只对那一台有意义
  （名字是数据根下一层的目录名或文件名。两张名单是 SHARED_DATA_ENTRIES 与 PER_HOST_DATA_ENTRIES，表与名单逐项相同，--selftest 核。）

工作根：环境变量 SINGLEFS_WORK_ROOT 设了就用它，没设是数据根下的 work（shared_data_path("work")，开机不清）。
会话临时目录根 /tmp/claude-<uid>（claude_session_temporary_root）：Claude Code 给每个会话的临时目录 <根>/<项目目录名>/<会话号>/（scratchpad 在里面）都在它底下，开机被清空。
  它不是草稿与报告的位置：scratch_roots、is_under_scratch_root 与写范围表的 $SINGLEFS_WORK_ROOT/ 只认工作根，子 agent 写不进它，派发提示的草稿目录不认它。
  mention_prefix_pattern 照样挑出文字里落在它底下的路径，交给各判法判（派发闸的写范围判成越界、「自证在哪跑」拒，派发闸与交回闸照读点名的规格与报告）。
  Bash 检出 hook 的 ⑦（就地改已有脚本）与 ⑪（通删）、看门狗查超阈值的编译目录与仓副本、完工清理闸、scripts/hosts/clean-session-scratch.py 照旧管它。

用法：
  python  按文件路径导入（importlib.util.spec_from_file_location，与 lib_shell_words.py 的导入法相同），或把 .claude/hooks 放进 sys.path 再 import
  shell   work_root="$(python3 <仓根>/.claude/hooks/lib_work_root.py)"：打一行展开后的工作根，不建目录
          data_root="$(python3 <仓根>/.claude/hooks/lib_work_root.py data-root)"：打一行数据根；
          last_runs="$(python3 <仓根>/.claude/hooks/lib_work_root.py shared last-runs)"：打一行数据根下共享的那一项（名字不在 SHARED_DATA_ENTRIES 里退 2，按台的点名出路）
          取不到（库跑不起来、退非 0、打出空行）就停下报错，不退回 $HOME 下的写法；数据根取不到（SINGLEFS_DATA_ROOT 与多机配置的 SHARED_DATA_ROOT 都没有、
          写法不对、不在共享根下）时这三种都退 2、标准输出空、标准错误写出路
  自证    python3 <仓根>/.claude/hooks/lib_work_root.py --selftest：在内存里换环境判数据根、工作根与两张名单，命令行那几格起它自己的入口（SINGLEFS_WORK_ROOT_BREAK=<项> 时必须判红）
  写范围表 .claude/hooks/agent-write-scope.tsv 的路径模式以 $SINGLEFS_WORK_ROOT/ 起头的，由 expand_scope_pattern 展开成工作根那一条
  钩子判不了而照常放行时 record_unjudged_pass 往检出记录追加一行（结局「只记」）：ask-user-claim-guard.sh、
  timer-only-remind-guard.sh、session-start.sh 调它；读不到这一份的钩子 stderr 说一句没记成

弄坏开关 SINGLEFS_WORK_ROOT_BREAK=<项>（只给用它的钩子与脚本的自证判红用；这份库的命令行入口平常跑设了就退 2、点名变量只在自证里认，别的脚本的 --selftest 起的子进程照认）。
打开后红在哪几格：用它的各份自证里都有格，下面各列一份自证实测红的失败行（write-guard.sh、runner-dispatch-guard.sh 的 --selftest），别的钩子的格名见它们各自的文件头：
  SINGLEFS_WORK_ROOT_BREAK=new-root-unknown  work_root() 交回会话临时目录根（工作根退回撤之前的位置），scratch_roots() 与 mention_prefix_pattern() 只认它、不认工作根
      「✗ 自检：范围:内置 agent 写工作根放行」、「✗ 自检：范围:工作根下的报告目录」、「✗ 自检：范围:会话临时目录根下写拒（scratchpad 之外）」、「✗ 自检：范围:会话临时目录根下的 scratchpad 里写拒」、「✗ 自检：先编后换:experiment-runner 改草稿目录里的副本放行」、「✗ 自检：行尾空白:子 agent 写仓外文件也判」、「✗ 自检：stdin:实现员写工作根下的报告」、「✗ 自检：stdin:七次拒绝都落进检出记录」、「✗ 自检：写者锁:持锁者第一次写放行」、「✗ 自检：写者锁:持锁者续写放行」、「✗ 自检：写者锁:持锁者写子目录放行」、「✗ 自检：写者锁:拒绝说明点名持锁者与出路」、「✗ 自检：写者锁:回执还没落盘的乙先写放行、不认领、记一处没判全」、「✗ 自检：写者锁:真持锁者回执还没落盘时第一次写也放行、不认领」、「✗ 自检：写者锁:回执落盘之后持锁者写拿到锁」、「✗ 自检：写者锁:认领之后持锁者续写放行」、「✗ 自检：写者锁:主 agent 不判」、「✗ 自检：写者锁:没有锁的目录不判」、「✗ 自检：写者锁:主 agent 写坏锁的目录不判」、「✗ 自检：写者锁:判不了的锁的拒绝点名那份锁与出路」、「✗ 自检：写者锁:认领文件判不了的拒绝点名那份认领文件与出路」、「✗ 自检：写者锁:草稿根自己下面直接写不判」、「✗ 自检：写者锁:回执对不上的拒绝点名还没认领的持锁者的 agent_id」、「✗ 自检：写者锁:回执对不上又找不到持锁者的回执时，拒绝写锁登记的提示 sha256 前几位」、「✗ 自检：写者锁:子目录锁的持有者写自己的子目录放行」、「✗ 自检：写者锁:父目录锁的持有者写父目录放行（认领父目录）」、「✗ 自检：写者锁:父目录认领之后，子目录锁的持有者写自己的子目录仍放行」、「✗ 自检：写者锁:读不到库时记一处没判全」、「✗ 自检：写者锁:自己出错时记一处没判全」、「✗ 自检：stdin:第二个写者的拒绝点名持锁者、锁的目录与出路」、「✗ 自检：stdin:持锁者放行」、「✗ 自检：stdin:回执对不上的拒绝点名持锁者的 agent_id 与出路」、「✗ 自检：stdin:坏锁的拒绝点名那份锁与出路」、「✗ 自检：stdin:坏认领文件拒绝并点名那份认领文件（退出码 2）」、「✗ 自检：stdin:回执还没落盘的写放行、不认领，stderr 一行并记一条检出（结局只记）」、「✗ 自检：stdin:写者锁自己出错放行并记一条检出（结局只记）」
      （各份自证里「工作根下」那几格红）
  SINGLEFS_WORK_ROOT_BREAK=legacy-root-accepted  scratch_roots()（连同 is_under_scratch_root 与写范围的 $SINGLEFS_WORK_ROOT/）另认会话临时目录根
      「✗ 自检：范围:会话临时目录根下写拒（scratchpad 之外）」、「✗ 自检：范围:会话临时目录根下的 scratchpad 里写拒」
      （各份自证里「会话临时目录根下」写、认草稿目录与报告「拒」或「不认」的那几格红）
  SINGLEFS_WORK_ROOT_BREAK=session-temporary-mentions-unseen  mention_prefix_pattern() 不挑会话临时目录根底下的路径
      「✗ 自检：写范围:登记了工作根的类型报告写进会话临时目录根拒」、「✗ 自检：写范围:没登记工作根的类型报告写进会话临时目录根拒」、「✗ 自检：快照:会话临时目录根下的规格文件里点名的照拒」、「✗ 自检：自证在哪跑:指到会话临时目录根拒」
      （write-guard.sh 的自证不红；各份自证里「会话临时目录根下」的路径照判的那几格红）
  SINGLEFS_WORK_ROOT_BREAK=data-root-ignored  shared_data_root() 不看 SINGLEFS_DATA_ROOT，一律照多机配置取（统一任务服务带的那一份不认）
      「✗ lib_work_root 自证：数据根：设了 SINGLEFS_DATA_ROOT 就用它」、「✗ lib_work_root 自证：工作根：没设 SINGLEFS_WORK_ROOT 时是数据根下的 work」、
      「✗ lib_work_root 自证：共享的名字：交数据根下那一项」、「✗ lib_work_root 自证：命令行：data-root 与 shared <名> 照环境打」、
      「✗ lib_work_root 自证：命令行：不带参数照旧打工作根」、「✗ lib_work_root 自证：数据根：两样都有时 SINGLEFS_DATA_ROOT 盖过配置」
      （本库自己的 --selftest；用它的别的自证不设 SINGLEFS_DATA_ROOT 的不红）
  SINGLEFS_WORK_ROOT_BREAK=data-root-home-fallback  SINGLEFS_DATA_ROOT 与多机配置的 SHARED_DATA_ROOT 都没有时退回家目录下的 .local/share/singlefs（第二台落进它本地那一份）
      「✗ lib_work_root 自证：数据根：变量与配置都没有时报错、不补缺省」、「✗ lib_work_root 自证：命令行：变量与配置都没有时 data-root 退 2、标准输出空、说出路」
      （本库自己的 --selftest）
  SINGLEFS_WORK_ROOT_BREAK=data-root-configuration-unchecked  多机配置的 SHARED_DATA_ROOT 不核在不在 NFS_SHARED_ROOTS 某一段下
      「✗ lib_work_root 自证：数据根：配置的 SHARED_DATA_ROOT 不在 NFS_SHARED_ROOTS 任何一段下不收」
      （本库自己的 --selftest）
  SINGLEFS_WORK_ROOT_BREAK=trailing-comment-kept  configuration_values() 不去多机配置行尾的「 # …」注释（带注释的 NFS_SHARED_ROOTS 把数据根判成不在共享根下）
      「✗ lib_work_root 自证：数据根：多机配置的行尾注释去掉再判（NFS_SHARED_ROOTS 与 SHARED_DATA_ROOT 都带）」
      （本库自己的 --selftest）
  SINGLEFS_WORK_ROOT_BREAK=per-host-entry-accepted  shared_data_path() 不核名字，按台的与认不出的名字也交数据根下那一项
      「✗ lib_work_root 自证：按台的名字与认不出的名字不收」、「✗ lib_work_root 自证：命令行：shared 后面给按台的名字退 2 并点名出路」
      （本库自己的 --selftest）
  SINGLEFS_WORK_ROOT_BREAK=formal-tools-per-host  shared_data_path("formal-tools") 当按台的名字拒（Verus 与 z3 落进各台家目录下那一份）
      「✗ lib_work_root 自证：共享的名字：formal-tools 交数据根下那一项（Verus 与 z3 两台同一个绝对路径读同一份）」、「✗ lib_work_root 自证：共享的名字：交数据根下那一项」
      （本库自己的 --selftest）
不认识的值抛 ValueError，免得拼错的开关被当成「没开」。
"""
import json
import os
import re
import sys
from datetime import datetime, timezone

WORK_ROOT_VARIABLE = "SINGLEFS_WORK_ROOT"
DATA_ROOT_VARIABLE = "SINGLEFS_DATA_ROOT"
DATA_ROOT_CONFIGURATION_KEY = "SHARED_DATA_ROOT"     # 多机配置里写数据根的那一个键（文件头「数据根」）
SHARED_ROOTS_CONFIGURATION_KEY = "NFS_SHARED_ROOTS"   # 同一份配置里两台共享的根，冒号分隔；数据根要落在其中一段下
MULTI_HOST_CONFIGURATION_VARIABLE = "SINGLEFS_MULTI_HOST_CONFIG"
MULTI_HOST_CONFIGURATION_NAME = "multi-host.env"
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
DATA_ROOT_HOWTO = ("在多机配置（主工作树根的 multi-host.env，模板 multi-host.env.example）里写一行 SHARED_DATA_ROOT=<数据根的绝对路径>，"
                   "落在同一份配置 NFS_SHARED_ROOTS 的某一段下；或设 SINGLEFS_DATA_ROOT（统一任务服务给每次尝试都带）")
WORK_ENTRY = "work"
# 文件头「数据根」那张表的两半，次序同表
SHARED_DATA_ENTRIES = ("work", "last-runs", "cross-run-cache", "broad-mutation-units", "reuse", "gate-staged-logs",
                       "doc-consolidate-migrated.tsv", "mutation-run-records", "fs-refs", "kernel-tree",
                       "memory-peaks.tsv", "formal-tools")
PER_HOST_DATA_ENTRIES = ("rocksdb-prebuilt", "qemu", "task-server-builds", "gpu-card-health")
SCOPE_TOKEN = "$" + WORK_ROOT_VARIABLE
BREAK_VARIABLE = "SINGLEFS_WORK_ROOT_BREAK"
KNOWN_BREAK_MODES = ("new-root-unknown", "legacy-root-accepted", "session-temporary-mentions-unseen", "data-root-ignored", "data-root-home-fallback",
                     "data-root-configuration-unchecked", "per-host-entry-accepted", "trailing-comment-kept", "formal-tools-per-host")
DETECTIONS_FILE_NAME = "agent-hook-detections.jsonl"
SESSION_TEMPORARY_MENTION = r"/tmp/claude-\d+/"


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise ValueError(f"{BREAK_VARIABLE}={mode} 不认识，只认 {'、'.join(KNOWN_BREAK_MODES)}")
    return mode


def claude_session_temporary_root():
    """Claude Code 的会话临时目录根 /tmp/claude-<uid>（开机被清空）：每个会话的临时目录 <根>/<项目目录名>/<会话号>/ 在它底下。"""
    return f"/tmp/claude-{os.getuid()}"


def home_directory():
    return os.path.normpath(os.path.expanduser("~"))


class DataRootUnconfigured(ValueError):
    """取不到数据根（文件头「数据根」）：SINGLEFS_DATA_ROOT 没设或不是绝对路径，多机配置的 SHARED_DATA_ROOT 没写、写法不对或不在共享根下。str 是原因，出路在 DATA_ROOT_HOWTO。"""


def main_worktree_root(root):
    """root 所在仓的主工作树根：<root>/.git 是目录就是 root；是文件（git worktree）照它的 gitdir 与那里的 commondir 找到公共的 .git、取它的上一层；都不是交 root。不起 git。"""
    marker = os.path.join(root, ".git")
    if os.path.isdir(marker):
        return root
    try:
        with open(marker, encoding="utf-8") as handle:
            pointer = handle.read().strip()
    except (OSError, UnicodeDecodeError):
        return root
    if not pointer.startswith("gitdir:"):
        return root
    git_directory = os.path.normpath(os.path.join(root, pointer[len("gitdir:"):].strip()))
    try:
        with open(os.path.join(git_directory, "commondir"), encoding="utf-8") as handle:
            common_directory = os.path.normpath(os.path.join(git_directory, handle.read().strip()))
    except (OSError, UnicodeDecodeError):
        common_directory = git_directory
    return os.path.dirname(common_directory)


def multi_host_configuration_path(environment):
    """多机配置在哪（文件头「数据根」）：SINGLEFS_MULTI_HOST_CONFIG（相对的照当前目录取成绝对路径），没设取这份库所在仓的主工作树根上的 multi-host.env。文件在不在不判。"""
    configured = environment.get(MULTI_HOST_CONFIGURATION_VARIABLE, "")
    if configured:
        return os.path.abspath(configured)
    return os.path.join(main_worktree_root(REPOSITORY_ROOT), MULTI_HOST_CONFIGURATION_NAME)


def configuration_values(path):
    """多机配置里一行一个 KEY=值的那几行 → {键: 值}（值先去掉行尾「 # …」注释、再去掉两头成对的一层引号，同一个键取头一行，
    同 scripts/gate/admission.py 的 configuration_value 与 task-server/task_server_lib.py 的 configured_value 去行尾注释的写法）；
    文件不在交空表，在而读不了抛 DataRootUnconfigured。弄坏开关 trailing-comment-kept 下不去行尾注释。"""
    if not os.path.isfile(path):
        return {}
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise DataRootUnconfigured(f"多机配置 {path} 读不了（{error}），取不到数据根") from error
    values = {}
    for line in lines:
        name, separator, value = line.strip().partition("=")
        if not separator or name in values or not re.fullmatch(r"[A-Z][A-Z0-9_]*", name):
            continue
        value = value.strip()
        if break_mode() != "trailing-comment-kept":
            value = re.sub(r"\s+#.*$", "", value)
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
            value = value[1:-1]
        values[name] = value
    return values


def configured_data_root(environment):
    """多机配置里的数据根（文件头「数据根」）：SHARED_DATA_ROOT 要是绝对路径、落在同一份配置 NFS_SHARED_ROOTS 某一段下；没写、不合都抛 DataRootUnconfigured。
    弄坏开关 data-root-home-fallback 下没写时退回家目录下那一份，data-root-configuration-unchecked 下不核共享根。"""
    path = multi_host_configuration_path(environment)
    values = configuration_values(path)
    value = values.get(DATA_ROOT_CONFIGURATION_KEY, "")
    if not value:
        if break_mode() == "data-root-home-fallback":
            return os.path.join(home_directory(), ".local", "share", "singlefs")
        where = f"多机配置 {path} 里也没有 {DATA_ROOT_CONFIGURATION_KEY} 一行" if os.path.isfile(path) else f"多机配置 {path} 不在"
        raise DataRootUnconfigured(f"取不到数据根：环境里没设 {DATA_ROOT_VARIABLE}，{where}；不补缺省、不退回家目录")
    if not value.startswith("/") or (value != "/" and value.endswith("/")) or any(character.isspace() for character in value):
        raise DataRootUnconfigured(f"多机配置 {path} 的 {DATA_ROOT_CONFIGURATION_KEY}={value} 不是一个绝对路径（以 / 起头、不以 / 结尾、不含空白）")
    root = os.path.normpath(value)
    shared_roots = [os.path.normpath(segment) for segment in values.get(SHARED_ROOTS_CONFIGURATION_KEY, "").split(":") if segment]
    if break_mode() != "data-root-configuration-unchecked" and not any(root == shared or root.startswith(shared.rstrip("/") + "/") for shared in shared_roots):
        raise DataRootUnconfigured(f"多机配置 {path} 的 {DATA_ROOT_CONFIGURATION_KEY}={value} 不在同一份配置 {SHARED_ROOTS_CONFIGURATION_KEY} 的任何一段下"
                                   f"（{'、'.join(shared_roots) or '一段都没写'}）：第二台照同一个绝对路径读写，读不到本机这一份")
    return root


def shared_data_root(environment=None):
    """数据根（文件头「数据根」：SINGLEFS_DATA_ROOT，没设取多机配置的 SHARED_DATA_ROOT，都没有抛 DataRootUnconfigured；绝对路径，不带末尾的 /）；不建目录。
    environment 不给取 os.environ（调用方带着自己那份环境判的给它那一份，多机配置的位置也照这一份找）。"""
    environment = os.environ if environment is None else environment
    configured = "" if break_mode() == "data-root-ignored" else environment.get(DATA_ROOT_VARIABLE, "")
    if configured:
        if not os.path.isabs(configured):
            raise DataRootUnconfigured(f"{DATA_ROOT_VARIABLE}={configured} 不是绝对路径（不经 ~ 与相对路径取：第二台的家目录与当前目录都是另一个）")
        return os.path.normpath(configured)
    return configured_data_root(environment)


def shared_data_path(name, environment=None):
    """数据根下共享的那一项（文件头「数据根」表里归「共享」的名字）；按台的与认不出的名字抛 ValueError，说明里写出路。"""
    if name == "formal-tools" and break_mode() == "formal-tools-per-host":
        raise ValueError(f"{name} 是按台各一份的（改前的归类），不经数据根取；照旧经 $HOME 取那一台自己的")
    if name not in SHARED_DATA_ENTRIES and break_mode() != "per-host-entry-accepted":
        if name in PER_HOST_DATA_ENTRIES:
            raise ValueError(f"{name} 是按台各一份的（.claude/hooks/lib_work_root.py 文件头「数据根」表），不经数据根取；照旧经 $HOME 取那一台自己的")
        raise ValueError(f"{name} 不在数据根的共享名单里（只认 {'、'.join(SHARED_DATA_ENTRIES)}）；"
                         "新的一项先在 .claude/hooks/lib_work_root.py 文件头「数据根」表与 SHARED_DATA_ENTRIES 里登记、写明为什么共享")
    return os.path.join(shared_data_root(environment), name)


def configured_work_root(environment=None):
    """配置的工作根（SINGLEFS_WORK_ROOT，没设是数据根下的 work，展开成绝对路径），不看工作根的那三个弄坏开关：自证拿它造「工作根下」的样本路径。"""
    environment = os.environ if environment is None else environment
    configured = environment.get(WORK_ROOT_VARIABLE, "")
    if configured:
        return os.path.normpath(os.path.abspath(os.path.expanduser(configured)))
    return shared_data_path(WORK_ENTRY, environment)


def work_root():
    """展开后的工作根（绝对路径，不带末尾的 /）；不建目录。"""
    if break_mode() == "new-root-unknown":
        return claude_session_temporary_root()
    return configured_work_root()


def scratch_roots():
    """认作草稿与报告位置的根：只有工作根（弄坏开关 new-root-unknown 下 work_root() 就是会话临时目录根；legacy-root-accepted 下另认会话临时目录根，去重）。"""
    mode = break_mode()
    roots = [work_root()]
    if mode == "legacy-root-accepted":
        roots.append(claude_session_temporary_root())
    return tuple(dict.fromkeys(roots))


def expand_written_path(path):
    """文字里写的路径换成绝对写法：$SINGLEFS_WORK_ROOT/、${SINGLEFS_WORK_ROOT}/ 换成工作根，~/、$HOME/、${HOME}/ 换成家目录；别的原样。"""
    for prefix in (SCOPE_TOKEN, "${" + WORK_ROOT_VARIABLE + "}"):
        if path == prefix or path.startswith(prefix + "/"):
            return work_root() + path[len(prefix):]
    for prefix in ("$HOME", "${HOME}"):
        if path == prefix or path.startswith(prefix + "/"):
            return home_directory() + path[len(prefix):]
    return os.path.expanduser(path)


def is_under_scratch_root(path):
    """path（文字里的写法也认）落在工作根底下（含根本身；scratch_roots 认的根）。"""
    absolute = os.path.normpath(os.path.abspath(expand_written_path(path)))
    return any(absolute == root or absolute.startswith(root + "/") for root in scratch_roots())


def mention_prefix_pattern():
    """正则片段：文字里一条路径落在工作根或会话临时目录根底下时的开头那一截（含根后面的 /）：挑出来交给各判法判，挑出来不等于认作草稿位置。
    工作根认四种写法：展开后的绝对路径、$SINGLEFS_WORK_ROOT/（与 ${…}/），在家目录下时另认 ~/…、$HOME/…、${HOME}/…；会话临时目录根认 /tmp/claude-<数字>/。"""
    mode = break_mode()
    forms = []
    if mode != "new-root-unknown":
        root = work_root()
        written = [root + "/", SCOPE_TOKEN + "/", "${" + WORK_ROOT_VARIABLE + "}/"]
        home = home_directory()
        if root.startswith(home + "/"):
            relative = root[len(home) + 1:]
            written += [f"~/{relative}/", f"$HOME/{relative}/", "${HOME}/" + relative + "/"]
        forms += [re.escape(form) for form in written]
    if mode != "session-temporary-mentions-unseen":
        forms.append(SESSION_TEMPORARY_MENTION)
    return "(?:" + "|".join(forms) + ")" if forms else "(?!)"


def expand_scope_pattern(pattern, anchored=True):
    """写范围表的一条路径模式：以 $SINGLEFS_WORK_ROOT/ 起头的展开成每个草稿根（scratch_roots）各一条，别的原样（交回列表）。
    anchored 为假只给写范围闸的弄坏开关用：根后面不带 /，同名前缀的兄弟目录也被放行。"""
    if not pattern.startswith(SCOPE_TOKEN + "/"):
        return [pattern]
    rest = pattern[len(SCOPE_TOKEN):] if anchored else pattern[len(SCOPE_TOKEN) + 1:]
    return [root + rest for root in scratch_roots()]


def default_detections_path():
    """钩子检出记录的默认位置（各钩子与看门狗另认环境变量 AGENT_HOOK_DETECTIONS）。"""
    return os.path.join(work_root(), DETECTIONS_FILE_NAME)


def state_directory(name):
    """钩子与看门狗的状态目录：工作根下的 <name>。"""
    return os.path.join(work_root(), name)


def record_unjudged_pass(hook_name, hook_input, command, findings, path=None):
    """钩子判不了而照常放行时，照「放行但必记检出」往检出记录追加一行（结局「只记」，看门狗据此报给主 agent）。
    hook_input 不是 dict（读不懂的输入）时会话号写「读不到」；findings 是给人看的几句（写清哪道钩子、卡在哪）；command 是给人看的这一次调用。
    path 不给取 AGENT_HOOK_DETECTIONS，没设取 default_detections_path。交 (记进的路径或 None, 一句给人看的说明)：记成了说记进哪里，没记成说为什么。"""
    known = hook_input if isinstance(hook_input, dict) else {}
    session = known.get("session_id")
    entry = {"time": datetime.now(timezone.utc).isoformat(), "hook": hook_name,
             "session_id": session if isinstance(session, str) and session else "读不到",
             "agent_id": known.get("agent_id"), "agent_type": known.get("agent_type") or "主 agent",
             "transcript_path": known.get("transcript_path"), "command": str(command or "")[:2000],
             "findings": [str(finding) for finding in findings], "outcome": "只记"}
    try:
        target = path or os.environ.get("AGENT_HOOK_DETECTIONS") or default_detections_path()
    except ValueError as error:   # 工作根的弄坏开关写错
        return None, f"检出没记成：取不到检出记录的位置（{error}）"
    try:
        os.makedirs(os.path.dirname(target) or ".", exist_ok=True)
        with open(target, "a", encoding="utf-8") as handle:
            handle.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError as error:
        return None, f"检出没记成：写不了 {target}（{error}）"
    return target, f"照「放行但必记检出」，检出记进 {target}"


def started_by_selftest():
    """祖先进程里有一个的命令行参数正好是 --selftest 或 --selftest-cells：这一趟是别的脚本的自证起的（它们的自证带着本库的弄坏开关跑子进程）。平常跑没有。"""
    pid = os.getppid()
    for _ in range(64):
        if pid <= 1:
            return False
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                words = handle.read().split(b"\0")
            with open(f"/proc/{pid}/stat", encoding="utf-8", errors="replace") as handle:
                status = handle.read()
        except OSError:
            return False
        if b"--selftest" in words or b"--selftest-cells" in words:
            return True
        pid = int(status.rsplit(")", 1)[1].split()[1])
    return False


def selftest():
    """数据根、工作根与两张名单的取法；命令行那几格起本库自己的入口（环境只带这一格给的几个变量与弄坏开关）。逐格一行，退 0 全过、1 有格不对。"""
    import subprocess
    sample_data_root, sample_work_root = "/sample-data-root", "/sample-work-root"
    sample = {DATA_ROOT_VARIABLE: sample_data_root}
    failures, checked = [], 0

    def check(name, passed, detail=""):
        nonlocal checked
        checked += 1
        if not passed:
            failures.append(name)
            print(f"  ✗ lib_work_root 自证：{name}{('：' + detail) if detail else ''}")  # gate-lint:detail

    def command_line(*arguments, environment):
        kept = {name: value for name, value in os.environ.items()
                if name not in (DATA_ROOT_VARIABLE, WORK_ROOT_VARIABLE, MULTI_HOST_CONFIGURATION_VARIABLE)}   # 弄坏开关照带下去
        completed = subprocess.run([sys.executable, os.path.abspath(__file__), *arguments], env=dict(kept, **environment),
                                   capture_output=True, text=True, timeout=60, check=False)
        return completed.returncode, completed.stdout.strip(), completed.stderr

    def raises_value_error(name):
        try:
            shared_data_path(name, sample)
        except ValueError as error:
            return str(error)
        return ""

    def outcome(function, *arguments):
        """(值, 抛出的 ValueError 的说明)：取法抛了就交 (None, 说明)，没抛交 (值, "")。"""
        try:
            return function(*arguments), ""
        except ValueError as error:
            return None, str(error)

    import tempfile
    with tempfile.TemporaryDirectory(prefix="lib-work-root-selftest-") as directory:
        # 多机配置几种写法：只有 SHARED_DATA_ROOT 落在共享根下的、没写这个键的、落在共享根外的、写成相对路径的；文件不在的那一份不建
        configured_data_root_value = os.path.join(directory, "shared", "data-root")
        configurations = {
            "configured": f"NFS_SHARED_ROOTS={directory}/elsewhere:{directory}/shared\nSHARED_DATA_ROOT='{configured_data_root_value}'\n",
            "unwritten": f"NFS_SHARED_ROOTS={directory}/shared\nENABLE_ACROSS_MACHINES=0\n",
            "outside": f"NFS_SHARED_ROOTS={directory}/shared\nSHARED_DATA_ROOT={directory}/not-shared/data-root\n",
            "relative": f"NFS_SHARED_ROOTS={directory}/shared\nSHARED_DATA_ROOT=relative/data-root\n",
            "commented": f"NFS_SHARED_ROOTS={directory}/shared  # 两台在同一绝对路径上共享\nSHARED_DATA_ROOT='{configured_data_root_value}'  # 数据根\n",
        }
        for name, content in configurations.items():
            with open(os.path.join(directory, f"{name}.env"), "w", encoding="utf-8") as handle:
                handle.write(content)
        only_configuration = {MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(directory, "configured.env")}
        unwritten = {MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(directory, "unwritten.env")}
        absent = {MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(directory, "absent.env")}
        value, said = outcome(shared_data_root, only_configuration)
        check("数据根：只有多机配置时取配置的 SHARED_DATA_ROOT（引号去掉一层）", (value, said) == (configured_data_root_value, ""), f"交「{value}」，说「{said}」")
        check("数据根：两样都有时 SINGLEFS_DATA_ROOT 盖过配置", outcome(shared_data_root, dict(only_configuration, **sample)) == (sample_data_root, ""))
        commented_value, commented_said = outcome(shared_data_root, {MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(directory, "commented.env")})
        check("数据根：多机配置的行尾注释去掉再判（NFS_SHARED_ROOTS 与 SHARED_DATA_ROOT 都带）", (commented_value, commented_said) == (configured_data_root_value, ""),
              f"交「{commented_value}」，说「{commented_said}」")
        unwritten_value, unwritten_said = outcome(shared_data_root, unwritten)
        absent_value, absent_said = outcome(shared_data_root, absent)
        check("数据根：变量与配置都没有时报错、不补缺省",
              unwritten_value is None and absent_value is None and DATA_ROOT_CONFIGURATION_KEY in unwritten_said and "不在" in absent_said
              and DATA_ROOT_VARIABLE in unwritten_said,
              f"配置没写那一键交「{unwritten_value}」说「{unwritten_said}」；配置不在交「{absent_value}」说「{absent_said}」")
        outside_value, outside_said = outcome(shared_data_root, {MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(directory, "outside.env")})
        check("数据根：配置的 SHARED_DATA_ROOT 不在 NFS_SHARED_ROOTS 任何一段下不收", outside_value is None and SHARED_ROOTS_CONFIGURATION_KEY in outside_said,
              f"交「{outside_value}」，说「{outside_said}」")
        relative_value, _ = outcome(shared_data_root, {MULTI_HOST_CONFIGURATION_VARIABLE: os.path.join(directory, "relative.env")})
        variable_value, _ = outcome(shared_data_root, {DATA_ROOT_VARIABLE: "~/.local/share/singlefs"})
        check("数据根：配置写成相对路径、变量写成 ~ 起头都不收", (relative_value, variable_value) == (None, None), f"配置交「{relative_value}」，变量交「{variable_value}」")
        # 主工作树根：一棵带 .git 目录的仓，一棵 git worktree（.git 文件指到 <主仓>/.git/worktrees/w，那里的 commondir 写 ../..），一棵不是 git 仓的
        main_tree, linked_tree, plain_tree = (os.path.join(directory, name) for name in ("main-tree", "linked-tree", "plain-tree"))
        os.makedirs(os.path.join(main_tree, ".git", "worktrees", "w"))
        os.makedirs(linked_tree)
        os.makedirs(plain_tree)
        with open(os.path.join(main_tree, ".git", "worktrees", "w", "commondir"), "w", encoding="utf-8") as handle:
            handle.write("../..\n")
        with open(os.path.join(linked_tree, ".git"), "w", encoding="utf-8") as handle:
            handle.write(f"gitdir: {main_tree}/.git/worktrees/w\n")
        found = [main_worktree_root(tree) for tree in (main_tree, linked_tree, plain_tree)]
        check("多机配置的位置：主工作树根照 .git 目录、git worktree 的 commondir 认，不是 git 仓的取它自己", found == [main_tree, main_tree, plain_tree], f"认成 {found}")
        exit_code, output, error_text = command_line("data-root", environment=unwritten)
        check("命令行：变量与配置都没有时 data-root 退 2、标准输出空、说出路",
              exit_code == 2 and not output and DATA_ROOT_CONFIGURATION_KEY in error_text and "→ 怎么办" in error_text,
              f"退 {exit_code}，标准输出「{output}」")
        exit_code, output, _ = command_line("data-root", environment=only_configuration)
        check("命令行：只有多机配置时 data-root 打配置那一份", (exit_code, output) == (0, configured_data_root_value), f"退 {exit_code} 打「{output}」")
    check("数据根：设了 SINGLEFS_DATA_ROOT 就用它", outcome(shared_data_root, sample) == (sample_data_root, ""), str(outcome(shared_data_root, sample)))
    check("工作根：没设 SINGLEFS_WORK_ROOT 时是数据根下的 work", outcome(configured_work_root, sample) == (sample_data_root + "/work", ""), str(outcome(configured_work_root, sample)))
    check("工作根：设了 SINGLEFS_WORK_ROOT 盖过数据根",
          configured_work_root(dict(sample, **{WORK_ROOT_VARIABLE: sample_work_root})) == sample_work_root)
    shared_paths = [outcome(shared_data_path, name, sample)[0] for name in SHARED_DATA_ENTRIES]
    check("共享的名字：交数据根下那一项",
          shared_paths == [f"{sample_data_root}/{name}" for name in SHARED_DATA_ENTRIES], f"交 {shared_paths[:3]}…")
    check("共享的名字：formal-tools 交数据根下那一项（Verus 与 z3 两台同一个绝对路径读同一份）",
          outcome(shared_data_path, "formal-tools", sample) == (f"{sample_data_root}/formal-tools", ""), str(outcome(shared_data_path, "formal-tools", sample)))
    refusals = {name: raises_value_error(name) for name in (*PER_HOST_DATA_ENTRIES, "no-such-entry")}
    check("按台的名字与认不出的名字不收", all(refusals.values()) and "按台" in refusals["qemu"] and "登记" in refusals["no-such-entry"],
          "、".join(name for name, said in refusals.items() if not said))
    table_names = re.findall(r"^  ([a-z][a-z0-9.-]+) +(共享|按台)  ", __doc__ or "", re.M)
    check("两张名单与文件头的表逐项相同、互不重名",
          table_names == [(name, "共享") for name in SHARED_DATA_ENTRIES] + [(name, "按台") for name in PER_HOST_DATA_ENTRIES]
          and not set(SHARED_DATA_ENTRIES) & set(PER_HOST_DATA_ENTRIES), f"表里是 {table_names}")
    exit_code, output, _ = command_line("data-root", environment=sample)
    shared_exit, shared_output, _ = command_line("shared", "last-runs", environment=sample)
    check("命令行：data-root 与 shared <名> 照环境打", (exit_code, output, shared_exit, shared_output) == (0, sample_data_root, 0, sample_data_root + "/last-runs"),
          f"data-root 退 {exit_code} 打「{output}」，shared last-runs 退 {shared_exit} 打「{shared_output}」")
    exit_code, output, error_text = command_line("shared", "qemu", environment=sample)
    check("命令行：shared 后面给按台的名字退 2 并点名出路", exit_code == 2 and not output and "按台" in error_text and "→ 怎么办" in error_text,
          f"退 {exit_code}，标准输出「{output}」")
    exit_code, output, _ = command_line(environment=sample)
    check("命令行：不带参数照旧打工作根", (exit_code, output) == (0, sample_data_root + "/work"), f"退 {exit_code} 打「{output}」")
    exit_code, output, error_text = command_line("no-such-subcommand", environment=sample)
    check("命令行：认不出的参数退 2", exit_code == 2 and not output and "→ 怎么办" in error_text, f"退 {exit_code}")
    if failures:
        print(f"  ✗ lib_work_root 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")  # gate-lint:summary
        print("  → 怎么办：照上面点名的格修 .claude/hooks/lib_work_root.py 的 shared_data_root、configured_data_root、main_worktree_root、shared_data_path、configured_work_root 或命令行入口；"
              f"设了 {BREAK_VARIABLE} 的这一趟红是对的")
        return 1
    print(f"  ✓ lib_work_root 自证通过（查了 {checked} 格：数据根照变量、照多机配置、两样都没有报错、配置不在共享根下与写成相对路径不收、主工作树根的认法、"
          "工作根跟着数据根与被 SINGLEFS_WORK_ROOT 盖过、共享名单、按台名单不收、表与名单相同、命令行几种）")
    return 0


USAGE_HOWTO = "不带参数打工作根；data-root 打数据根；shared <名> 打数据根下共享的那一项；--selftest 跑自证"

if __name__ == "__main__":
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    if os.environ.get(BREAK_VARIABLE) and not started_by_selftest():
        # 弄坏开关只在自证里认：这份库的命令行入口（shell 脚本取工作根用）被别的脚本的自证带着开关起，平常跑设了就拒，不管值是什么
        print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证（用这份库的钩子与脚本的 --selftest 起的子进程）里认，平常跑设了就拒，这一趟什么都没做", file=sys.stderr)
        print(f"  → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> bash .claude/hooks/write-guard.sh --selftest", file=sys.stderr)
        sys.exit(2)
    try:
        if not arguments:
            print(work_root())
        elif arguments == ["data-root"]:
            print(shared_data_root())
        elif len(arguments) == 2 and arguments[0] == "shared":
            print(shared_data_path(arguments[1]))
        else:
            print(f"  ✗ 认不出的参数：{' '.join(arguments)}", file=sys.stderr)
            print(f"  → 怎么办：{USAGE_HOWTO}", file=sys.stderr)
            sys.exit(2)
    except DataRootUnconfigured as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"  → 怎么办：{DATA_ROOT_HOWTO}", file=sys.stderr)
        sys.exit(2)
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"  → 怎么办：名字照文件头「数据根」表写；弄坏开关只给自证判红用，平时不设 {BREAK_VARIABLE}", file=sys.stderr)
        sys.exit(2)
