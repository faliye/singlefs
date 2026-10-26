"""长活声明的判法：预计挂钟超过 2 小时的活，要说清分片单元、续跑入口、机器与额度取自多机配置与现查三样；分不了的写一行缺哪一段。
判据：.claude/rules/test-script-discipline.md「用上现有的硬件加速」与「判定可复用：按输入存，按前缀剪」。
三处用这一份：scripts/hosts/detached-run.sh 的 start（--wall-clock over-2h）与 long-run-check 判被起脚本的文件头；
.claude/hooks/runner-dispatch-guard.sh 判派 experiment-designer 与带重活的几种类型（线程上限那张类型表）的提示；scripts/hosts/host-dispatch.py 判派法占几台（started_script、script_header_declaration、judge_script_header）；
.claude/hooks/bash-command-detector.sh 的 ⑭ 判主 agent 用 run_in_background 起的命令（background_long_runs，被起的脚本与文件头声明照同一份认法读）。

声明的四个键（脚本文件头 / 派发提示的写法）：
  shards       # long-run-shards: <分片单元：按什么切、几片怎么定>                         分片：<…>
  resume       # long-run-resume: <续跑入口：同一条命令重起只补没做完的，怎么认做完的>       续跑：<…>
  hosts        # long-run-hosts: <机器与额度怎么取>                                         多机：<…>
               （脚本文件头与派发提示都按「登记值」读：这一行逐字等于 CANONICAL_HOSTS_SENTENCE 那一句规范写法（可带句末句号），后面不许接任何说明，说明另起一行写；不再用现算词表；
                 只有 LIVE_HOSTS_EXEMPT_TABLE 登记的脚本（sha256 对得上的那一版）文件头按「现算」读：点名 --emit-hosts 与 host-resources.py，见 judge 的 hosts_reading）
  unshardable  # long-run-unshardable: <缺哪一段、为什么>                                   分不了：<…>
派发提示另有一行「预计挂钟：2 小时内」或「预计挂钟：超过 2 小时」（后面可接逗号与推的依据）。

判法（judge）：
  - 三样里值不空、不是「无 / 没有 / -」、不是 <…> 占位的算写了。
  - hosts 写了的，按 hosts_reading 读：
      "registered"（默认：派发提示与脚本文件头走这一种，用户 2026-10-12 定、2026-10-13 把文件头也并过来）：这一行逐字等于 CANONICAL_HOSTS_SENTENCE 那一句（台数点名 --emit-hosts、
        每台线程点名 .claude/heavy-thread-default 的登记值），可带句末句号（「。」或「.」），后面不许接任何说明（「；……」「，……」都拒）；别的写法一律拒，不用现算词表去拦。
        要说明的另起一行写：脚本文件头里写成不带 long-run-hosts: 的普通注释行（同一个键写第二行会按次序接在后面，接出来就不再逐字等于规范句），
        派发提示里写成不以「多机：」起头的普通正文行。
        规范句写在这一处，骨架（scripts/agents/dispatch-prompt.py）、scripts/hosts/detached-run.sh 与 scripts/hosts/host-dispatch.py 的说明引它，不另抄词表。
        文件头写的读法与脚本做的事是否一致，静态判法看不到，由写文件头的人保证。
      "live"（只有 LIVE_HOSTS_EXEMPT_TABLE 登记、sha256 也对得上的脚本文件头走这一种，judge_script_header 查表）：要点名机器清单的读法 --emit-hosts（scripts/crash/crash-point-replay-shard-configuration-check.sh）与现查余量的 host-resources.py。
    两种都不许写死额度（HARD_CODED_QUOTA：数字后面跟线程、核、台、K/M/G/T，或 --threads N、-j N、--memory-cap N、--shard k/n、THREADS=N 这类）。
  - 三样缺的，要有 unshardable 一行：至少 UNSHARDABLE_MINIMUM_CHARACTERS 个字，并逐个点名缺的那几样的词（分片 / 续跑 / 多机）。
脚本文件头：#! 之后连着的注释块（# 起头的行与空行，到第一行别的为止）；同一个键写了几行，按次序接起来。
脚本文件头的读法（judge_script_header）：脚本相对本库所在仓根的路径在 LIVE_HOSTS_EXEMPT_TABLE（.claude/long-run-live-hosts-exempt，一行「路径  sha256  # 理由」）里、
  而且表里登记的 sha256 等于这份脚本现在文件的 sha256 的，按 "live" 读，别的一律 "registered"；在表里而 sha256 对不上的（脚本改过了）按 "registered" 读，另加一条 sha256 对不上的问题（判红）。
  表里只有按现算写、被 sha256 钉住不能改的脚本；指向不存在的路径、sha256 对不上、脚本已经按登记值写齐、按现算读也不齐、比基准多出的行、
  基准那一版里同一路径的 sha256 变了的行（改了脚本正文再把表里的 sha256 重钉，也算涨），都由 exempt-check 判红（只缩不涨按「路径 + sha256」成对比）。
命令行里写死的线程与内存（command_quota_problem；records/2026-10-23-subagent拆分提案.md 第五十九节第 1 条与 ⑥b）：script 子命令（detached-run.sh 的 start --wall-clock over-2h
  与 long-run-check）另判 -- 之后那条命令的每个词：capped.sh 后面不是 default 的数、run-with-memory-cap.sh 后面不是 auto 的上限、--threads / --jobs / --workers /
  --lanes / --test-threads / --memory-cap / --memory-max / -j 带的数（贴着写或另起一个词，-j8 也算）、名字里有 THREADS / JOBS / WORKERS / LANES / MEMORY / THREAD_CAP 的
  赋值写成数（E168_THREADS=10、GATE_VERUS_MEMORY_MAX=16G）。有这样的词、多机配置又开着双机（two_hosts_on：crash-point-replay-shard-configuration-check.sh --emit-hosts
  退 0、机器清单里本机外至少 1 台；不连任何一台）时判不齐，出路同第五十九节第 1 条：按 --emit-hosts 与 host-resources.py --json 分配（线程写 capped.sh default、
  内存写 run-with-memory-cap.sh auto），或在被起脚本的 # long-run-unshardable: 写明分不了、点名多机。双机关着、配置判不过或判不了、
  被起的脚本没写多机一行而分不了一行点名了多机的，照放，stderr 打一行「·」说明写死了什么、为什么不拒。
被起的脚本（started_script）：命令的词里，含 / 或以 .sh / .py 结尾、按命令的当前目录解得到普通文件、以 .sh / .py 结尾或开头是 #! 的，
  跳过包装（WRAPPER_SCRIPTS）之后的第一份；一份都没有的（bash -c '…' 一长串、直接起 cargo 或编出来的二进制）判不出。
  scripts/hosts/two-logs.sh [--quiet] <名> -- <命令…>（日志出两份的包装）不当被起的脚本：照 lib_heavy_tests.command_under_two_logs 剥开（重型闸与范围声明闸剥它用的就是这一个，不另写一份），
  被起的是 `--` 之后那条命令里的脚本；认它按名字与位置（scripts/hosts 这一层），别处的同名拷贝与 --from-log、--selftest 不剥（照普通脚本判，它们的文件头没有长活声明）。

用法：
  python3 lib_long_run.py script <命令的当前目录> -- <命令与参数…>
      判被起脚本的声明与命令行里写死的线程与内存：齐了退 0、stdout 打一行；不齐退 1、stderr 逐条 ✗ 加 → 出路；用法错与弄坏开关写错退 2
  python3 lib_long_run.py exempt-check <仓根> [--base <gate 基准提交>]
      判 LIVE_HOSTS_EXEMPT_TABLE：写法、路径在不在、登记的 sha256 对不对得上、脚本是不是真要靠现算读法才齐，给了 --base 的再比只缩不涨（按路径 + sha256 成对比：基准那一版里没有的路径、同一路径的 sha256 换了的都判红；
      基准那一版在仓根里经 .claude/gate.d/lib-lag-baseline.py 的 baseline_text 取（滞后表与基准比的共用读法）：基准里还没有这张表不比，git 取不到判红；
      基准那一版可以还是不带 sha256 的旧写法，那样的行只比路径；
      搬迁登记表 .claude/gate.d/path-moves.tsv 有此刻生效的行时（lib-lag-baseline.py 的 effective_moves：这一行不在 gate 基准那一版的登记表里、gate 基准那一版里旧前缀下有文件、新前缀下没有文件的行，提交进基准之后不再生效）：
      表里的路径已由 baseline_text 照生效的行换成新前缀；同一路径 sha256 不同的行，再按搬迁之前的路径从基准里取那份脚本
      （lib-lag-baseline.py 的 baseline_file_bytes），这一行的脚本要自己搬了家（基准里取到的是搬迁之前的路径，不是这条路径本身），
      它的 sha256 要是基准那一版表里登记的，把内容里生效的旧前缀全换成新前缀（rewrite_moved_bytes）之后
      与此刻的文件逐字节相同的，不算重钉、放行；差一个字节、取不到、或基准那一版文件的 sha256 对不上表的，照旧判红，红行后面接一句为什么没认成搬迁）；
      好退 0、stdout 打一行（带行数）；有问题退 1、stderr 逐条 ✗ 加 → 出路；仓根下没有这张表退 77；用法错与弄坏开关写错退 2（门禁 code-tooling 的 long-run-header-exempt 格调它）
  python   import lib_long_run（把 .claude/hooks 放进 sys.path），调 prompt_declaration、duplicate_prompt_problems（派发提示里「分片：」「续跑：」「多机：」同一标签写了两行就判红，只有派发闸用）、judge、judge_script_header

弄坏开关 LIB_LONG_RUN_BREAK=<项>（逗号连几个；只给用它的几份的自证判红用，认不出的值抛 ValueError）。打开后红在哪几格写在各份的文件头：
  partial-accepted            三样写了一样就当齐了
  hosts-unchecked             hosts 一行不看内容（不核两份现查、不核写死的额度）
  hosts-canonical-unchecked   "registered" 读法下，hosts 一行不核是不是规范句（写了就放，只剩写死的额度那一条）（派发闸与脚本文件头都红）
  hosts-suffix-accepted       "registered" 读法下，规范句后面接任何说明都放行（只核以规范句起头）（派发闸与脚本文件头都红）
  header-live-reading         脚本文件头退回按 "live" 读（点名 host-resources.py 才齐；登记值那一句反而拒）
  live-exempt-ignored         不查 LIVE_HOSTS_EXEMPT_TABLE，登记了的脚本也按 "registered" 读
  exempt-path-unchecked       exempt-check 不核表里的路径在不在（只 code-tooling 的 long-run-header-exempt 格用）
  exempt-needless-unchecked   exempt-check 不核脚本是不是已经按登记值写齐、按现算读是不是齐（只 long-run-header-exempt 格用）
  exempt-sha256-unchecked     豁免表里登记的 sha256 不比（exempt-check 不核、judge_script_header 在表里的一律按 "live" 读）（exempt-check 的格与 detached-run.sh 的自证用）
  exempt-growth-unchecked     exempt-check 不比基准（路径涨了与 sha256 重钉两样都不比；只 long-run-header-exempt 格用）
  exempt-repin-unchecked      exempt-check 比基准时只比路径、不比 sha256（基准里同一路径的 sha256 换了也放；只 long-run-header-exempt 格用）
  exempt-path-moves-ignored   exempt-check 比 sha256 时不认登记的搬迁（搬了家、内容只差路径的脚本也算重钉）：
                              门禁 code-tooling 的样本 long-run-header-exempt-moved-green 期望退出 0、实测 1（「登记的 sha256 … 与基准那一版里的 … 不同」；只 long-run-header-exempt 格用）
  exempt-unmoved-accepted     exempt-check 认搬迁时不核这一行的脚本自己搬没搬家（自己没搬、只是正文里的路径换了的也当搬迁放行）：
                              门禁 code-tooling 的样本 long-run-header-exempt-moved-unmoved-red 期望退出 1、实测 0（只 long-run-header-exempt 格用）
  unshardable-unchecked       unshardable 一行不看字数、不看点没点名缺的那几样
  script-unresolved-accepted  判不出被起的脚本时当成齐了（只 detached-run.sh 用）
  wrappers-not-skipped        不跳过包装，第一份脚本就算被起的那份（只 detached-run.sh 用）
  two-logs-not-unwrapped      不剥 two-logs.sh，它自己就算被起的那份脚本（它的文件头没有长活声明，经它起的三样齐的脚本被判不齐）（只 detached-run.sh 用）
  command-quota-unchecked     不判 -- 之后命令行里写死的线程与内存（改前的样子）（只 detached-run.sh 用）
  command-quota-hosts-ignored 命令行里写死了线程或内存就拒，不看多机配置开没开双机、不看分不了一行点没点名多机（只 detached-run.sh 用）
  background-history-ignored       background_long_runs 不读用时表（历史用时一律当没有）（只 bash-command-detector.sh 的 ⑭ 用）
  background-header-ignored        background_long_runs 没有历史时不看被起脚本文件头的长活声明（只 ⑭ 用）
  background-detached-unrecognized background_long_runs 不认 detached-run.sh start --wall-clock over-2h 的放行口，-- 之后那条照判（只 ⑭ 用）
  background-short-entries-judged  background_long_runs 不认看门狗、合入后验证、派发器这几种短的入口（只 ⑭ 用）
  background-read-only-judged-by-header  background_long_runs 不认只读用法（登记为 light 的单独 --selftest、单独的 --list-items、--help、-h），被起脚本写了长活声明的照声明拒（改前的样子；只 ⑭ 用）
  background-undeclared-recorded   background_long_runs 认得出被起的脚本、没有历史也没有声明的，照旧交进 undeterminable（改前的样子：每条都记一条检出；只 ⑭ 用）
  background-table-error-raised    background_long_runs 用时表读不了时照抛异常（改前的样子：调用方记「没判成」，不交进 undeterminable；只 ⑭ 用）

主 agent 用 run_in_background 起的命令判不判得出超过 2 小时（background_long_runs；bash-command-detector.sh 的 ⑭ 调，records/2026-10-23-subagent拆分提案.md 第五十九节 ⑥a，
  用户 2026-10-26 定「执行前拒」）：命令位置上的每一条命令（lib_shell_words 切，cd 跟着走）逐条判：
  历史用时：scripts/hosts/memory_peaks_lib.py 的用时表（default_duration_table，键同内存峰值表 command_key；经 bash / sh / python3 起的，带上起法与不带的两种键各查一次，取大的），
    超过 memory_peaks_lib.LONG_RUN_SECONDS 的交进 refused，不到的放过；
  没有历史的：BACKGROUND_SHORT_ENTRIES 放过；被起的脚本（started_script，与 script 子命令同一份认法）后面跟的参数是只读用法的放过（READ_ONLY_ARGUMENT_FORMS：
    正好是 --list-items、--help 或 -h；正好是 --selftest 而那份自证在 .claude/hooks/selftest-registry.tsv 登记为 light 的，照 lib_heavy_tests.selftest_judged_light 判；
    长活声明管的是真跑，只读用法不按声明判，有历史用时的照历史判）；被起脚本文件头写了长活声明（script_header_declaration，四个键写了任一个）的按声明算超过 2 小时、交进 refused；
    认得出被起的脚本、文件头没有 # long-run-* 键的判为不是长活，放过不记；认不出被起的脚本的不判。
  真判不了的交进 undeterminable（调用方放行并记检出）：用时表读不了（整条命令记一条；写了长活声明的也不拒，另各记一条）、被起脚本的文件头读不了、
    文件头有 # long-run-* 键而值都没写（空、无或占位：有声明但读不出）。
  detached-run.sh start --wall-clock over-2h 起的放过（-- 之后那条的声明由 detached-run.sh 自己判）；--wall-clock within-2h 起的判 -- 之后那条；list、watch、stop 放过。
"""
import hashlib
import importlib.util
import os
import re
import shlex
import subprocess
import sys
from typing import NamedTuple

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
import lib_shell_words as shell_words  # noqa: E402
import lib_heavy_tests  # noqa: E402

file_sha256 = lib_heavy_tests.file_sha256   # 文件内容的 sha256：共用重型测试判定模块那一份，不另写

BREAK_VARIABLE = "LIB_LONG_RUN_BREAK"
KNOWN_BREAK_MODES = ("partial-accepted", "hosts-unchecked", "unshardable-unchecked", "script-unresolved-accepted", "wrappers-not-skipped",
                     "two-logs-not-unwrapped", "hosts-canonical-unchecked", "hosts-suffix-accepted", "header-live-reading",
                     "live-exempt-ignored", "exempt-path-unchecked", "exempt-needless-unchecked", "exempt-growth-unchecked", "exempt-repin-unchecked", "exempt-sha256-unchecked",
                     "exempt-path-moves-ignored", "exempt-unmoved-accepted", "command-quota-unchecked", "command-quota-hosts-ignored",
                     "background-history-ignored", "background-header-ignored", "background-detached-unrecognized", "background-short-entries-judged",
                     "background-read-only-judged-by-header", "background-undeclared-recorded", "background-table-error-raised")
WALL_CLOCK_THRESHOLD_HOURS = 2
WALL_CLOCK_WITHIN, WALL_CLOCK_OVER = "within-2h", "over-2h"
REQUIRED_KEYS = ("shards", "resume", "hosts")
KEY_WORDS = {"shards": "分片", "resume": "续跑", "hosts": "多机"}   # unshardable 一行要点名的词
HEADER_KEYS = {"long-run-shards": "shards", "long-run-resume": "resume", "long-run-hosts": "hosts", "long-run-unshardable": "unshardable"}
HEADER_SPELLING = {key: f"# {name}:" for name, key in HEADER_KEYS.items()}
PROMPT_LABELS = {"分片": "shards", "续跑": "resume", "多机": "hosts", "分不了": "unshardable"}
PROMPT_SPELLING = {key: f"{label}：" for label, key in PROMPT_LABELS.items()}
WALL_CLOCK_LABEL = "预计挂钟"
HOST_READS = ("--emit-hosts", "host-resources.py")
HOSTS_READING_LIVE, HOSTS_READING_REGISTERED = "live", "registered"
# 登记值读法下多机一行的规范写法（逐字）：派发提示的「多机：」行与脚本文件头的 # long-run-hosts: 只认这一句（可带句末句号），后面不许接任何说明，说明另起一行写。
# 规范句只写在这一处：骨架、detached-run.sh、host-dispatch.py 的说明与自证样本引它，不另抄一份
CANONICAL_HOSTS_SENTENCE = "台数从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台线程取 .claude/heavy-thread-default 的登记值，不现算、不写常量"
HOSTS_SENTENCE_ENDINGS = ("", "。", ".")   # 规范句本身可带的句末句号（全角或半角）；别的结尾都算接了说明
LIVE_HOSTS_EXEMPT_TABLE = ".claude/long-run-live-hosts-exempt"   # 文件头仍按现算写、不能改的脚本：一行「路径  sha256  # 理由」，路径相对仓根，sha256 是被豁免那份脚本文件的 sha256（64 位小写十六进制）
LIVE_HOSTS_EXEMPT_ROW = re.compile(r"^(?P<path>[^\s#]+)[ \t]{2,}(?P<sha256>[0-9a-f]{64})[ \t]{2,}#[ \t]*(?P<reason>\S.*?)\s*$")
LIVE_HOSTS_EXEMPT_BASE_ROW = re.compile(r"^(?P<path>[^\s#]+)[ \t]{2,}(?:(?P<sha256>[0-9a-f]{64})(?:[ \t]|$))?")   # 基准那一版：路径必取；sha256 有就取（基准可能还是不带 sha256 的旧写法，那样的行只比路径）
EXEMPT_REASON_MINIMUM_CHARACTERS = 4
PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))   # 本库在 <仓根>/.claude/hooks/ 下
# 取 gate 基准那一版的豁免表：滞后表与基准比的共用读法（门禁目录里那一份，exempt-check --base 才载入）
LAG_BASELINE_LIBRARY = os.path.join(os.path.dirname(os.path.dirname(os.path.realpath(__file__))), "gate.d", "lib-lag-baseline.py")
UNSHARDABLE_MINIMUM_CHARACTERS = 8
NONE_VALUES = {"", "无", "无。", "没有", "-", "—", "空", "none"}
PLACEHOLDER = re.compile(r"^<[^>\n]*>$")
HARD_CODED_QUOTA = re.compile(
    r"(?<![A-Za-z0-9_./\-])\d+(?:\.\d+)?\s*(?:个线程|线程|核|台|[KMGTkmgt]i?[Bb]?(?![A-Za-z]))"
    r"|(?:--threads|--jobs|--workers|--lanes|--memory-cap|--memory-max|(?<![A-Za-z\-])-j)[=\s]\s*\d+"
    r"|--shard[=\s]\s*\d+/\d+"
    r"|\b[A-Z_]*(?:THREADS|JOBS|MEMORY|CAP)[A-Z_]*=\d+")
LINE_START = r"^[ \t>*_\-]*"
WALL_CLOCK_LINE = re.compile(LINE_START + WALL_CLOCK_LABEL + r"[* \t]*[：:][* \t]*(.*)$", re.M)
WALL_CLOCK_WITHIN_VALUE = re.compile(r"^2\s*小时内")
WALL_CLOCK_OVER_VALUE = re.compile(r"^超过\s*2\s*小时")
# 包一层再起命令的脚本：跳过它们找被起的那份（lib_shell_words 认的两个带一个参数的包装，加上把命令挪到别处跑、清场后跑、脱离会话跑的三个）
WRAPPER_SCRIPTS = set(shell_words.WRAPPERS_TAKING_ONE_ARGUMENT) | {"multi-host-run.sh", "gpu-quiesce-run.sh", shell_words.DETACHED_RUN_SCRIPT_NAME}
SCRIPT_SUFFIXES = (".sh", ".py")
HEADER_READ_BYTES = 65536
# 命令行里写死的线程与内存（文件头「命令行里写死的线程与内存」）
QUOTA_WRAPPER_DEFAULTS = {"capped.sh": "default", "run-with-memory-cap.sh": "auto"}   # 两个包装各自照规范取法写的那个参数
QUOTA_OPTIONS = ("--threads", "--jobs", "--workers", "--lanes", "--test-threads", "--memory-cap", "--memory-max", "-j")
QUOTA_JOINED_JOBS = re.compile(r"^-j\d+$")
QUOTA_ASSIGNMENT = re.compile(r"^([A-Za-z_][A-Za-z0-9_]*)=(.*)$")
QUOTA_VARIABLE_NAME = re.compile(r"THREADS|JOBS|WORKERS|LANES|MEMORY|THREAD_CAP")
QUOTA_VALUE = re.compile(r"^\d+(?:\.\d+)?(?:[KMGTkmgt](?:i?B)?)?$")
# 多机配置开没开双机：判法与机器清单只在这一份（--emit-hosts 不连任何一台）
CONFIGURATION_CHECK = os.path.join(PROJECT_ROOT, "scripts", "crash", "crash-point-replay-shard-configuration-check.sh")
CONFIGURATION_CHECK_TIMEOUT_SECONDS = 60


def breaks():
    modes = {mode for mode in os.environ.get(BREAK_VARIABLE, "").split(",") if mode}
    unknown = modes - set(KNOWN_BREAK_MODES)
    if unknown:
        raise ValueError(f"{BREAK_VARIABLE} 认不出 {'、'.join(sorted(unknown))}（认得的：{'、'.join(KNOWN_BREAK_MODES)}）")
    return modes


def is_written(value):
    if value is None:
        return False
    text = value.strip()
    return text not in NONE_VALUES and not PLACEHOLDER.match(text)


def canonical_hosts_problem(hosts, modes=frozenset()):
    """登记值读法下 hosts 一行是不是规范写法：逐字等于 CANONICAL_HOSTS_SENTENCE，可带句末句号（HOSTS_SENTENCE_ENDINGS），后面不许接任何说明。
    合返回 None，不合返回一句原因（接在「<…> 一行」后面）。"""
    text = hosts.strip()
    if "hosts-suffix-accepted" in modes and text.startswith(CANONICAL_HOSTS_SENTENCE):
        return None
    if text in {CANONICAL_HOSTS_SENTENCE + ending for ending in HOSTS_SENTENCE_ENDINGS}:
        return None
    if not text.startswith(CANONICAL_HOSTS_SENTENCE):
        return (f"不是规范写法「{CANONICAL_HOSTS_SENTENCE}」（逐字）：写的是「{text[:80]}」；登记值读法下这一行只认这一句（可带句末句号），"
                "别的说法（台数线程的取法换了词、加了括注、写了现算或折算）一律不认")
    rest = text[len(CANONICAL_HOSTS_SENTENCE):]
    return (f"在规范句「{CANONICAL_HOSTS_SENTENCE}」后面接了说明「{rest[:60]}」：这一行逐字等于规范句（可带句末句号），后面不许接任何说明；"
            "说明另起一行写（脚本文件头里写成不带 long-run-hosts: 的普通注释行，派发提示里写成不以「多机：」起头的普通正文行）")


def judge(declared, spelling, hosts_reading=HOSTS_READING_REGISTERED):
    """declared：键 → 值（没写的键不在里面）；spelling：键 → 那一行的写法（报问题时用）；hosts_reading：hosts 一行按哪种读（HOSTS_READING_REGISTERED 或 HOSTS_READING_LIVE）。
    交回问题的列表，空的是齐了。"""
    if hosts_reading not in (HOSTS_READING_LIVE, HOSTS_READING_REGISTERED):
        raise ValueError(f"hosts_reading 只认 {HOSTS_READING_LIVE}、{HOSTS_READING_REGISTERED}，给的是 {hosts_reading!r}")
    modes = breaks()
    problems = []
    written = {key for key in REQUIRED_KEYS if is_written(declared.get(key))}
    if "partial-accepted" in modes and written:
        written = set(REQUIRED_KEYS)
    hosts = declared.get("hosts")
    if "hosts" in written and is_written(hosts) and "hosts-unchecked" not in modes:
        if hosts_reading == HOSTS_READING_REGISTERED:
            canonical = None if "hosts-canonical-unchecked" in modes else canonical_hosts_problem(hosts, modes)
            if canonical:
                problems.append(f"{spelling['hosts']} 一行{canonical}")
        else:
            unnamed = [read for read in HOST_READS if read not in hosts]
            if unnamed:
                problems.append(f"{spelling['hosts']} 一行没点名 {'、'.join(unnamed)}：机器清单只从 crash-point-replay-shard-configuration-check.sh --emit-hosts 读，每台的线程与内存从 host-resources.py --json 现算")
        quotas = [match.group(0).strip() for match in HARD_CODED_QUOTA.finditer(hosts)]
        if quotas:
            problems.append(f"{spelling['hosts']} 一行写死了额度：{'、'.join(quotas)}（台数、线程、内存要按多机配置与现查算，不写常量）")
    missing = [key for key in REQUIRED_KEYS if key not in written]
    if missing:
        unshardable = declared.get("unshardable")
        names = "、".join(f"{KEY_WORDS[key]}（{spelling[key]}）" for key in missing)
        if not is_written(unshardable):
            problems.append(f"缺 {names}，也没有 {spelling['unshardable']} 一行写明缺哪一段")
        elif "unshardable-unchecked" not in modes:
            text = unshardable.strip()
            if len(text) < UNSHARDABLE_MINIMUM_CHARACTERS:
                problems.append(f"{spelling['unshardable']} 一行只有 {len(text)} 个字，至少 {UNSHARDABLE_MINIMUM_CHARACTERS} 个字写明缺哪一段、为什么")
            unnamed = [KEY_WORDS[key] for key in missing if KEY_WORDS[key] not in text]
            if unnamed:
                problems.append(f"{spelling['unshardable']} 一行没点名缺的 {'、'.join(unnamed)}（缺的 {names} 每一样都要在这一行里写出它的词）")
    return problems


def script_header_declaration(path):
    """脚本文件头注释块里的四个键：键 → 值（同一个键几行按次序用空格接起来）。读不了抛 OSError。"""
    with open(path, "rb") as handle:
        text = handle.read(HEADER_READ_BYTES).decode("utf-8", errors="replace")
    declared = {}
    for index, line in enumerate(text.split("\n")):
        stripped = line.strip()
        if index == 0 and stripped.startswith("#!"):
            continue
        if stripped and not stripped.startswith("#"):
            break
        match = re.match(r"#\s*(long-run-[a-z]+)\s*:\s*(.*)$", stripped)
        if match and match.group(1) in HEADER_KEYS:
            key = HEADER_KEYS[match.group(1)]
            declared[key] = (declared[key] + " " + match.group(2)).strip() if key in declared else match.group(2).strip()
    return declared


def live_hosts_exempt_rows(root=PROJECT_ROOT):
    """LIVE_HOSTS_EXEMPT_TABLE 的行：[(行号, 路径, sha256, 理由)]，写法不对的行 路径 与 sha256 是 None、理由 是原行；表不在是 None。"""
    table = os.path.join(root, LIVE_HOSTS_EXEMPT_TABLE)
    if not os.path.isfile(table):
        return None
    rows = []
    with open(table, encoding="utf-8") as handle:
        for number, line in enumerate(handle.read().split("\n"), 1):
            if not line.strip() or line.lstrip().startswith("#"):
                continue
            found = LIVE_HOSTS_EXEMPT_ROW.match(line)
            if found and len(found.group("reason")) >= EXEMPT_REASON_MINIMUM_CHARACTERS and not os.path.isabs(found.group("path")):
                rows.append((number, os.path.normpath(found.group("path")), found.group("sha256"), found.group("reason")))
            else:
                rows.append((number, None, None, line))
    return rows


def header_exemption(script, root=PROJECT_ROOT):
    """脚本在豁免表里的情形：(None, None) 不在表里（或表读不了）；(登记的 sha256, 这份脚本现在文件的 sha256) 在表里。现在的读不了是 (登记的, None)。"""
    try:
        rows = live_hosts_exempt_rows(root)
    except OSError:
        return None, None
    relative = os.path.relpath(os.path.realpath(script), os.path.realpath(root))
    for _number, path, registered, _reason in rows or []:
        if path == relative:
            try:
                return registered, file_sha256(os.path.realpath(script))
            except OSError:
                return registered, None
    return None, None


def header_hosts_reading(script, root=PROJECT_ROOT):
    """脚本文件头的 # long-run-hosts: 按哪种读：登记在 LIVE_HOSTS_EXEMPT_TABLE 里、登记的 sha256 又等于这份脚本现在的 sha256 的按 "live"，别的一律 "registered"
    （在表里而 sha256 对不上的也按 "registered" 读，judge_script_header 另报 sha256 对不上）。"""
    modes = breaks()
    if "header-live-reading" in modes:
        return HOSTS_READING_LIVE
    if "live-exempt-ignored" in modes:
        return HOSTS_READING_REGISTERED
    registered, current = header_exemption(script, root)
    if registered is None:
        return HOSTS_READING_REGISTERED
    if "exempt-sha256-unchecked" in modes or registered == current:
        return HOSTS_READING_LIVE
    return HOSTS_READING_REGISTERED


def judge_script_header(script, declared, root=PROJECT_ROOT):
    """被起脚本文件头的声明按 header_hosts_reading 判：detached-run.sh 的 start 与 long-run-check、host-dispatch.py 文件头 ⓪ 都调这一个。
    脚本在豁免表里而登记的 sha256 对不上（脚本改过了）：按登记值读法判，另加一条 sha256 对不上的问题，一定判红。"""
    problems = judge(declared, HEADER_SPELLING, header_hosts_reading(script, root))
    modes = breaks()
    if "header-live-reading" not in modes and "live-exempt-ignored" not in modes and "exempt-sha256-unchecked" not in modes:
        registered, current = header_exemption(script, root)
        if registered is not None and registered != current:
            problems.insert(0, f"{os.path.relpath(os.path.realpath(script), os.path.realpath(root))} 在 {LIVE_HOSTS_EXEMPT_TABLE} 里登记的 sha256 是 {registered}，"
                               f"这份脚本现在的是 {current or '（读不了）'}：豁免只给登记的那一版文件，脚本改过就不再按现算读；文件头改按登记值读法写（多机一行用规范句），或把改过的脚本连新 sha256 交主 agent 问用户")
    return problems


def exempt_table_problems(root, base_table=None, moved_unchanged=None):
    """LIVE_HOSTS_EXEMPT_TABLE 的问题：[(说什么, 怎么办)] 与行数。base_table 是基准那一版的表文本（None 不比只缩不涨）。表不在返回 (None, 0)。
    moved_unchanged(路径, 基准那一版表里这条路径的 sha256 集合)：搬迁登记表有内容行时给（exempt_main），交 None 是「照登记的搬迁换算之后与基准那一版逐字节相同」，
    交一句话是没认成搬迁的原因（接在重钉那一条后面）；不给（None）时重钉照旧判红。"""
    rows = live_hosts_exempt_rows(root)
    if rows is None:
        return None, 0
    modes = breaks()
    problems = []
    for number, path, registered_sha256, reason in rows:
        if path is None:
            problems.append((f"第 {number} 行不是「<相对仓根的路径>  <被豁免脚本的 sha256，64 位小写十六进制>  # 理由」（各段之间至少两个空格，理由至少 {EXEMPT_REASON_MINIMUM_CHARACTERS} 个字，路径不写绝对路径）：{reason[:120]}",
                             "照这个写法改；表里只放文件头仍按现算写、被 sha256 钉住不能改的脚本，sha256 取 sha256sum <脚本> 的前一栏"))
            continue
        full = os.path.join(root, path)
        if not os.path.isfile(full):
            if "exempt-path-unchecked" not in modes:
                problems.append((f"第 {number} 行指向不存在的路径：{path}", "脚本搬了就改路径，删了或改成按登记值写就删掉这一行；豁免只缩不涨"))
            continue
        if "exempt-sha256-unchecked" not in modes:
            try:
                current_sha256 = file_sha256(full)
            except OSError as error:
                problems.append((f"第 {number} 行的 {path} 读不了：{error}", "看它的读权限"))
                continue
            if current_sha256 != registered_sha256:
                problems.append((f"第 {number} 行的 {path} 现在的 sha256 是 {current_sha256}，表里登记的是 {registered_sha256}（脚本改过了，豁免只给登记的那一版文件）",
                                 "这份脚本不该再改：把改动撤回去；确要改它，文件头按登记值读法写（多机一行用规范句）并删掉这一行，或交主 agent 问用户再改表里的 sha256"))
                continue
        if "exempt-needless-unchecked" in modes:
            continue
        try:
            declared = script_header_declaration(full)
        except OSError as error:
            problems.append((f"第 {number} 行的 {path} 读不了：{error}", "看它的读权限"))
            continue
        if not judge(declared, HEADER_SPELLING, HOSTS_READING_REGISTERED):
            problems.append((f"第 {number} 行的 {path} 文件头已经按登记值读法写齐了，不需要豁免", "删掉这一行"))
        elif judge(declared, HEADER_SPELLING, HOSTS_READING_LIVE):
            problems.append((f"第 {number} 行的 {path} 文件头按现算读法也不齐：{'；'.join(judge(declared, HEADER_SPELLING, HOSTS_READING_LIVE))}",
                             "豁免只放行按现算读法写齐的脚本；补齐它的文件头，或删掉这一行并改成登记值读法"))
    if base_table is not None and "exempt-growth-unchecked" not in modes:
        base_rows = {}   # 路径 → 基准那一版里这个路径登记的 sha256 的集合；旧写法（不带 sha256）的行记一个 None，那样的路径只比路径
        for line in base_table.split("\n"):
            found = LIVE_HOSTS_EXEMPT_BASE_ROW.match(line)
            if found and not line.lstrip().startswith("#"):
                base_rows.setdefault(os.path.normpath(found.group("path")), set()).add(found.group("sha256"))
        for number, path, registered_sha256, _reason in rows:
            if path is None:
                continue
            if path not in base_rows:
                problems.append((f"第 {number} 行 {path} 是基准那一版里没有的（豁免表涨了）", "豁免只缩不涨：新脚本的文件头按登记值写；确实要加的，交主 agent 问用户再加"))
            elif "exempt-repin-unchecked" not in modes and None not in base_rows[path] and registered_sha256 not in base_rows[path]:
                moved_note = ""
                if moved_unchanged is not None and "exempt-path-moves-ignored" not in modes:
                    why_not_moved = moved_unchanged(path, base_rows[path])
                    if why_not_moved is None:
                        continue
                    moved_note = f"；{why_not_moved}"
                problems.append((f"第 {number} 行 {path} 登记的 sha256 {registered_sha256} 与基准那一版里的 {'、'.join(sorted(base_rows[path]))} 不同（豁免表重钉了：改了脚本正文再换表里的 sha256 也算涨）{moved_note}",
                                 "豁免只缩不涨，表里的 sha256 不能换：把脚本的改动撤回去（sha256 对回基准那一版）；文件头已能按登记值读法写的，改写文件头并删掉这一行；确要保留改动并重钉的，交主 agent 问用户"))
    return problems, len(rows)


def starts_with_shebang(path):
    try:
        with open(path, "rb") as handle:
            return handle.read(2) == b"#!"
    except OSError:
        return False


def started_script(words, directory):
    """命令的词里被起的那份脚本的绝对路径；判不出交 None（写法在文件头「被起的脚本」）。"""
    modes = breaks()
    skip_wrappers = "wrappers-not-skipped" not in modes
    for index, word in enumerate(words):
        if "two-logs-not-unwrapped" not in modes:
            under_two_logs = lib_heavy_tests.command_under_two_logs(words[index:], directory)   # two-logs.sh <名> -- <命令…>：被起的是 `--` 之后那条里的脚本
            if under_two_logs is not None:
                return started_script(under_two_logs[0], directory)
        if not ("/" in word or word.endswith(SCRIPT_SUFFIXES)):
            continue
        path = os.path.normpath(word if os.path.isabs(word) else os.path.join(directory, word))
        if not os.path.isfile(path):
            continue
        if not (word.endswith(SCRIPT_SUFFIXES) or starts_with_shebang(path)):
            continue
        if skip_wrappers and os.path.basename(os.path.realpath(path)) in WRAPPER_SCRIPTS:
            continue
        return path
    return None


def command_quotas(words):
    """-- 之后那条命令里写死的线程与内存：逐个交回原样的写法（「capped.sh 31」「--memory-cap 6G」「E168_THREADS=10」）；没有交空列表。写法在文件头「命令行里写死的线程与内存」。"""
    found = []
    for index, word in enumerate(words):
        following = words[index + 1] if index + 1 < len(words) else None
        name = os.path.basename(word)
        if name in QUOTA_WRAPPER_DEFAULTS and following is not None and not following.startswith("-"):
            if following != QUOTA_WRAPPER_DEFAULTS[name]:
                found.append(f"{name} {following}")
            continue
        option, separator, value = word.partition("=")
        if option in QUOTA_OPTIONS:
            value = value if separator else following
            if value is not None and QUOTA_VALUE.match(value):
                found.append(f"{option}{'=' if separator else ' '}{value}")
            continue
        if QUOTA_JOINED_JOBS.match(word):
            found.append(word)
            continue
        assignment = QUOTA_ASSIGNMENT.match(word)
        if assignment and QUOTA_VARIABLE_NAME.search(assignment.group(1).upper()) and QUOTA_VALUE.match(assignment.group(2)):
            found.append(word)
    return found


def two_hosts_on():
    """多机配置开没开双机：(开着与否, 一句说明)。经 crash-point-replay-shard-configuration-check.sh --emit-hosts（不连任何一台）判；
    判不过、判不了的按关着算，说明里写为什么。"""
    try:
        result = subprocess.run(["bash", CONFIGURATION_CHECK, "--emit-hosts", PROJECT_ROOT], capture_output=True, text=True,
                                stdin=subprocess.DEVNULL, timeout=CONFIGURATION_CHECK_TIMEOUT_SECONDS)
    except (OSError, subprocess.TimeoutExpired) as error:
        return False, f"判不了多机配置（{type(error).__name__}: {error}），按双机关着算"
    first_line = next((line.strip() for line in (result.stdout + "\n" + result.stderr).splitlines() if line.strip()), "")
    if result.returncode == 0:
        count = re.search(r"^MULTI_HOST_COUNT=(\d+)$", result.stdout, re.M)
        machines = int(count.group(1)) if count else 0
        if machines >= 1:
            return True, f"多机配置开着双机（机器清单里本机外 {machines} 台，crash-point-replay-shard-configuration-check.sh --emit-hosts 退 0）"
        return False, "多机配置开着而机器清单里本机外 0 台，按双机关着算"
    if result.returncode == 3:
        return False, first_line[:160]
    return False, f"多机配置判不过（crash-point-replay-shard-configuration-check.sh --emit-hosts 退 {result.returncode}：{first_line[:120]}），按双机关着算"


def command_quota_problem(words, declared):
    """script 子命令另判的一条（文件头「命令行里写死的线程与内存」）：(问题或 None, 不拒时的说明或 None)。"""
    modes = breaks()
    if "command-quota-unchecked" in modes:
        return None, None
    quotas = command_quotas(words)
    if not quotas:
        return None, None
    written = "、".join(quotas)
    if "command-quota-hosts-ignored" not in modes:
        if not is_written(declared.get("hosts")) and is_written(declared.get("unshardable")) and KEY_WORDS["hosts"] in declared["unshardable"]:
            return None, f"-- 之后的命令行里写死了 {written}；被起的脚本写了分不了、点名了多机，不拒"
        on, note = two_hosts_on()
        if not on:
            return None, f"-- 之后的命令行里写死了 {written}；{note}，不拒"
    else:
        note = "弄坏开关 command-quota-hosts-ignored：不看多机配置"
    return (f"-- 之后的命令行里写死了线程或内存：{written}，而{note}：超过 2 小时的活要按多机配置分到各台，台数、每台线程与内存不写常量"), None


# 主 agent 用 run_in_background 起的命令（文件头「主 agent 用 run_in_background 起的命令」）
MEMORY_PEAKS_LIBRARY = os.path.join(PROJECT_ROOT, "scripts", "hosts", "memory_peaks_lib.py")   # 用时表的读法与取键，按路径载入（只用标准库、载入不起子进程）
BACKGROUND_SHORT_ENTRIES = frozenset({"watch.sh", "agent-watch.py", "merge-and-verify.sh", "host-dispatch.py"})   # 没有历史时也放过的短入口（派发器自己判长活）
READ_ONLY_ARGUMENT_FORMS = (["--list-items"], ["--help"], ["-h"])   # 被起脚本后面正好跟这几种之一的只读，不按长活声明判（单独的 --selftest 另看登记表）
BACKGROUND_LAUNCHERS = {"shell": ("bash", "sh"), "python": ("python3", "python")}   # lib_shell_words 剥掉的起法：用时表里的键可能带着它


class BackgroundLongRuns(NamedTuple):
    refused: list          # [(命令的写法, 依据)]：判为超过 2 小时
    undeterminable: list   # [(命令的写法, 为什么判不了)]：用时表读不了、被起脚本的文件头读不了、有 # long-run-* 键而值没写


def memory_peaks_library():
    specification = importlib.util.spec_from_file_location("memory_peaks_lib", MEMORY_PEAKS_LIBRARY)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def detached_wall_clock(arguments):
    """detached-run.sh 脚本名之后的参数（--force 已摘）里 --wall-clock 的值：`--wall-clock X` 或 `--wall-clock=X`，-- 之后的不算；没写交 None。"""
    for index, word in enumerate(arguments):
        if word == "--":
            break
        if word == "--wall-clock" and index + 1 < len(arguments):
            return arguments[index + 1]
        if word.startswith("--wall-clock="):
            return word.split("=", 1)[1]
    return None


def background_history(peaks, rows, words, launcher, environment):
    """用时表里这条命令的 (秒数, 量的那一天)：不带起法与带上 bash / sh / python3 的键各查一次，取大的；都没有交 (None, None)。"""
    explicit = (environment or {}).get("RUN_WITH_MEMORY_CAP_KEY")
    found = []
    for candidate in [list(words)] + [[name, *words] for name in BACKGROUND_LAUNCHERS.get(launcher, ())]:
        row = rows.get(peaks.command_key(candidate, explicit))
        if row is not None:
            found.append((row[0], row[1][:10]))
    return max(found) if found else (None, None)


def started_script_arguments(words, directory, script):
    """被起的脚本在 words 里写成哪个词、后面跟的参数：(那个词, 参数)；按 directory 解到 script 的第一个词，找不到交 None。"""
    for index, word in enumerate(words):
        if ("/" in word or word.endswith(SCRIPT_SUFFIXES)) and os.path.normpath(word if os.path.isabs(word) else os.path.join(directory, word)) == script:
            return word, list(words[index + 1:])
    return None


def read_only_usage(word, arguments, directory, environment):
    """被起脚本后面跟的参数是不是只读用法（文件头「主 agent 用 run_in_background 起的命令」）：是交一句写法，不是交 None。"""
    if "background-read-only-judged-by-header" in breaks():
        return None
    if arguments in READ_ONLY_ARGUMENT_FORMS:
        return f"只读用法 {arguments[0]}"
    if arguments == ["--selftest"] and lib_heavy_tests.selftest_judged_light([word, "--selftest"], directory, environment):
        return "登记为 light 的单独 --selftest"
    return None


def background_long_runs(command_text, directory, environment=None):
    """主 agent 用 run_in_background 起的命令：交 BackgroundLongRuns（判法在文件头「主 agent 用 run_in_background 起的命令」）。
    用时表读不了的交进 undeterminable（弄坏开关 background-table-error-raised 下照抛，由调用方当判不了）；memory_peaks_lib.py 载不进照抛。"""
    modes = breaks()
    peaks = memory_peaks_library()
    table = peaks.default_duration_table(os.environ)
    refused, undeterminable = [], []
    table_problem = None
    if "background-history-ignored" in modes:
        rows = {}
    else:
        try:
            rows = peaks.read_duration_table(table)
        except (OSError, ValueError) as error:
            if "background-table-error-raised" in modes:
                raise
            rows, table_problem = {}, f"用时表 {table} 读不了（{type(error).__name__}: {error}）"
            undeterminable.append((command_text[:160], f"{table_problem}，这条命令的历史用时判不了"))
    for command in shell_words.commands_at_command_position(command_text, directory, environment).commands:
        words, launcher, where = list(command.words), command.launcher, command.directory
        if not words:
            continue
        name = os.path.basename(words[0])
        if name == shell_words.DETACHED_RUN_SCRIPT_NAME or shell_words.detached_run_script(words[0], where):
            inner = shell_words.detached_run_start_command(words[1:])
            if inner is None:
                continue   # list、watch、stop、--selftest：不起长活
            if (detached_wall_clock(shell_words.detached_run_arguments_after_preflight(words[1:])) == WALL_CLOCK_OVER
                    and "background-detached-unrecognized" not in modes):
                continue   # 经 start --wall-clock over-2h 起：声明由 detached-run.sh 自己判
            words, launcher, name = list(inner), None, os.path.basename(inner[0])
        shown = shlex.join(words)[:160]
        seconds, day = background_history(peaks, rows, words, launcher, command.environment)
        if seconds is not None:
            if seconds > peaks.LONG_RUN_SECONDS:
                refused.append((shown, f"历史用时 {seconds / 3600:.1f} 小时（用时表 {table}，量于 {day}），超过 2 小时"))
            continue
        if "background-short-entries-judged" not in modes and name in BACKGROUND_SHORT_ENTRIES:
            continue
        script = started_script(words, where) if where else None
        if script is None:
            continue   # 认不出被起的脚本（直接起 cargo、二进制、小工具，或 cd 到算不出的目录之后的相对路径）：没有历史就不判
        position = started_script_arguments(words, where, script)
        if position is not None and read_only_usage(*position, where, command.environment):
            continue   # 只读用法：长活声明管的是真跑
        try:
            declared = script_header_declaration(script)
        except OSError as error:
            undeterminable.append((shown, f"被起的脚本 {script} 的文件头读不了（{error}），判不了写没写长活声明"))
            continue
        if "background-header-ignored" in modes:
            declared = {}
        written = [key for key in (*REQUIRED_KEYS, "unshardable") if is_written(declared.get(key))]
        if written and table_problem:
            undeterminable.append((shown, f"被起的脚本 {script} 文件头写了长活声明，但{table_problem}，判不了有没有更短的历史用时，这一次不拒"))
        elif written:
            refused.append((shown, f"没有历史用时，被起的脚本 {script} 文件头写了长活声明（{'、'.join(HEADER_SPELLING[key] for key in written)}），按声明算超过 2 小时"))
        elif declared:
            undeterminable.append((shown, f"被起的脚本 {script} 文件头有 {'、'.join(HEADER_SPELLING[key] for key in declared)} 而值都没写（空、无或占位），"
                                          "判不了按不按长活算"))
        elif "background-undeclared-recorded" in modes:
            undeterminable.append((shown, f"用时表 {table} 里没有这条的历史用时，被起的脚本 {script} 文件头也没写 # long-run-* 声明"))
    return BackgroundLongRuns(refused, undeterminable)


def prompt_declaration(prompt):
    """派发提示里的预计挂钟与四行：交回 (挂钟, 声明)。挂钟是 None（没写那一行）、WALL_CLOCK_WITHIN、WALL_CLOCK_OVER，或认不出时那一行的原值。"""
    wall_clock = None
    match = WALL_CLOCK_LINE.search(prompt)
    if match:
        value = match.group(1).strip()
        wall_clock = (WALL_CLOCK_WITHIN if WALL_CLOCK_WITHIN_VALUE.match(value)
                      else WALL_CLOCK_OVER if WALL_CLOCK_OVER_VALUE.match(value) else value)
    declared = {}
    for label, key in PROMPT_LABELS.items():
        line = re.search(LINE_START + label + r"[* \t]*[：:][* \t]*(.*)$", prompt, re.M)
        if line:
            declared[key] = line.group(1).strip()
    return wall_clock, declared


def duplicate_prompt_problems(prompt):
    """派发提示里「分片：」「续跑：」「多机：」（REQUIRED_KEYS 那三样）哪一个写了两行及以上：prompt_declaration 只读第一行，第二行写什么都没人判，所以同一标签出现两次判红。
    交回问题的列表（每个重复的标签一条，点名每一行的开头），空的是没有重复。判的是传进来的整段文字：要先去掉代码块的，调用方先去（runner-dispatch-guard.sh 的 without_code_blocks）。"""
    problems = []
    for label, key in PROMPT_LABELS.items():
        if key not in REQUIRED_KEYS:
            continue
        values = [found.group(1).strip() for found in re.finditer(LINE_START + label + r"[* \t]*[：:][* \t]*(.*)$", prompt, re.M)]
        if len(values) >= 2:
            shown = "；".join(f"第 {order} 处「{value[:30]}」" for order, value in enumerate(values[:4], 1))
            problems.append(f"「{label}：」写了 {len(values)} 行（{shown}）：只写一行，只有第一行会被读；要补充的另起一行，写成不以「{label}：」起头的普通正文")
    return problems


def script_main(directory, words):
    """命令行 script 子命令：退 0 齐了，1 不齐或判不出。"""
    script = started_script(words, directory)
    if script is None:
        if "script-unresolved-accepted" in breaks():
            print("  ✓ 长活声明：判不出被起的脚本，照齐了算（弄坏开关 script-unresolved-accepted）")
            return 0
        print("  ✗ 长活声明：判不出被起的是哪份脚本（-- 之后没有一个词是脚本文件；bash -c '…' 一长串、直接起 cargo 或编出来的二进制都判不出），"
              "超过 2 小时的活读不到分片、续跑、多机三样的声明", file=sys.stderr)
        print("     → 怎么办：把这条命令写进一份脚本，文件头写 # long-run-shards: / # long-run-resume: / # long-run-hosts:（分不了的写 # long-run-unshardable: 点名缺哪一段），"
              "改成 -- bash <那份脚本> 起；确实 2 小时内跑得完的写 --wall-clock within-2h", file=sys.stderr)
        return 1
    try:
        declared = script_header_declaration(script)
    except OSError as error:
        print(f"  ✗ 长活声明：读不了被起的脚本 {script}：{error}", file=sys.stderr)
        print("     → 怎么办：核对 -- 之后那份脚本的路径与读权限再起", file=sys.stderr)
        return 1
    problems = judge_script_header(script, declared)
    quota_problem, quota_note = command_quota_problem(words, declared)
    if quota_note:
        print(f"  · 长活声明（{script}）：{quota_note}", file=sys.stderr)
    if problems or quota_problem:
        for problem in problems:
            print(f"  ✗ 长活声明（{script}）：{problem}", file=sys.stderr)  # gate-lint:detail
        if problems:
            print(f"     → 怎么办：在 {script} 文件头的注释块里补齐（判据 .claude/rules/test-script-discipline.md「用上现有的硬件加速」「判定可复用：按输入存，按前缀剪」）："
                  "# long-run-shards: 分片单元；# long-run-resume: 同一条命令重起只补没做完的；"
                  f"# long-run-hosts: {CANONICAL_HOSTS_SENTENCE}（逐字这一句，可带句末句号，后面不接任何说明；要说明另起一行写普通注释）；"
                  "分不了的写 # long-run-unshardable: <点名缺的分片 / 续跑 / 多机与为什么，至少 8 个字>"
                  f"（{LIVE_HOSTS_EXEMPT_TABLE} 里登记的脚本只要 sha256 对得上就按现算读，那张表只缩不涨，不是给新脚本加的）", file=sys.stderr)
        if quota_problem:
            print(f"  ✗ 长活声明（{script}）：{quota_problem}", file=sys.stderr)
            print("     → 怎么办：改成按 crash-point-replay-shard-configuration-check.sh --emit-hosts 与 host-resources.py --json 分配——台数从 --emit-hosts 读、"
                  "每台线程取登记值（capped.sh 写 default，不写数）、内存写 run-with-memory-cap.sh auto（按峰值表取）或由被起的脚本按 host-resources.py --json 现查，"
                  f"写进被起脚本、文件头的 # long-run-hosts: 照这一句「{CANONICAL_HOSTS_SENTENCE}」；"
                  "或在被起脚本的文件头写 # long-run-unshardable: 写明为什么分不了（点名缺的多机那一段）", file=sys.stderr)
        return 1
    how = "；".join(f"{KEY_WORDS.get(key, '分不了')}：{declared[key]}" for key in (*REQUIRED_KEYS, "unshardable") if is_written(declared.get(key)))
    print(f"  ✓ 长活声明齐了（{script}，判了 {len(declared)} 个键）：{how}")
    return 0


def moved_script_unchanged(lag_baseline, base, root, moves, path, base_sha256s, kept):
    """重钉的那一行是不是只差登记的搬迁：按搬迁之前的路径从基准里取那份脚本，取到的要是搬迁之前的路径（这一行的脚本自己搬了家），sha256 要在基准那一版表里登记的集合里，
    换成新前缀之后与此刻的文件逐字节相同交 None（并把 path 记进 kept）；不是的交一句原因。"""
    data, taken, problem = lag_baseline.baseline_file_bytes(base, path, cwd=root)
    if problem is not None:
        return f"按搬迁之前的路径取基准那一版的脚本没取到：{problem.sentence(120)}"
    if data is None:
        return f"基准 {base} 里没有 {path}，照 .claude/gate.d/path-moves.tsv 倒推的搬迁之前的路径也没有"
    if os.path.normpath(taken) == os.path.normpath(path) and "exempt-unmoved-accepted" not in breaks():
        return f"{path} 自己没搬家（基准 {base} 里同一路径就有），正文里的路径换了也不按搬迁认"
    if hashlib.sha256(data).hexdigest() not in base_sha256s:
        return f"基准里的 {taken} 的 sha256 {hashlib.sha256(data).hexdigest()} 不是基准那一版表里登记的，没法当成被钉住的那一版去比"
    try:
        with open(os.path.join(root, path), "rb") as handle:
            current = handle.read()
    except OSError as error:
        return f"读不了此刻的 {path}：{error}"
    if lag_baseline.rewrite_moved_bytes(data, moves) == current:
        kept.append(path)
        return None
    return (f"基准里的 {taken} 照 .claude/gate.d/path-moves.tsv 把旧前缀换成新前缀之后，与此刻的 {path} 仍不逐字节相同"
            "（除了登记的路径还改了别处，不算搬迁）")


def exempt_main(root, base):
    """命令行 exempt-check 子命令：退 0 好，1 有问题，77 仓根下没有这张表。base 是 gate 基准提交（None 不比只缩不涨）：
    基准那一版经 LAG_BASELINE_LIBRARY 的 baseline_text 在仓根里取，基准里还没有这张表不比，git 取不到判红。
    搬迁登记表有此刻生效的行时（effective_moves），重钉的行再经 moved_script_unchanged 核是不是只差生效的登记搬迁（文件头「用法」exempt-check 那一段）。"""
    base_table = None
    moved_unchanged = None
    moved_kept = []
    if base is not None:
        try:
            import importlib.util
            lag_baseline_spec = importlib.util.spec_from_file_location("lag_baseline", LAG_BASELINE_LIBRARY)
            lag_baseline = importlib.util.module_from_spec(lag_baseline_spec)
            lag_baseline_spec.loader.exec_module(lag_baseline)
            base_table, problem = lag_baseline.baseline_text(base, LIVE_HOSTS_EXEMPT_TABLE, cwd=root)
            moves, problem = lag_baseline.effective_moves(base, root) if problem is None else ([], problem)
        except (OSError, ImportError, ValueError, AttributeError) as error:
            print(f"  ✗ 取不了基准 {base} 那一版的 {LIVE_HOSTS_EXEMPT_TABLE}：{type(error).__name__}: {error}", file=sys.stderr)
            print(f"     → 怎么办：核对 {LAG_BASELINE_LIBRARY} 在不在、LAG_BASELINE_BREAK 写没写对；取不到基准那一版不是「基准里还没有这张表」", file=sys.stderr)
            return 1
        if problem is not None:
            print(f"  ✗ {problem.sentence(120)}，只缩不涨没比", file=sys.stderr)
            print("     → 怎么办：看 git 能不能读到基准那一版（上面 git ls-tree / git show 那一步的报错），修好再跑这一格；取不到基准那一版不是「基准里还没有这张表」", file=sys.stderr)
            return 1
        if moves:
            def moved_unchanged(path, base_sha256s):
                return moved_script_unchanged(lag_baseline, base, root, moves, path, base_sha256s, moved_kept)
    try:
        problems, count = exempt_table_problems(root, base_table, moved_unchanged)
    except OSError as error:
        print(f"  ✗ 读不了 {os.path.join(root, LIVE_HOSTS_EXEMPT_TABLE)}：{error}", file=sys.stderr)
        print("     → 怎么办：看它的读权限", file=sys.stderr)
        return 1
    if problems is None:
        print(f"  ⊘ 本次无对象可判：{root} 下没有 {LIVE_HOSTS_EXEMPT_TABLE}")
        return 77
    if problems:
        print(f"  ✗ {LIVE_HOSTS_EXEMPT_TABLE} 有 {len(problems)} 处不对（共 {count} 行）：", file=sys.stderr)  # gate-lint:summary
        for what, _howto in problems:
            print(f"     {what}", file=sys.stderr)  # gate-lint:detail
        print(f"     → 怎么办：{'；'.join(sorted({howto for _what, howto in problems}))}", file=sys.stderr)
        return 1
    note = "与基准那一版比没涨" if base_table is not None else "没给基准（基准那一版里还没有这张表，或不在 git 仓里），只缩不涨这一条没比"
    if moved_kept:
        note += f"（其中 {len(moved_kept)} 行照 .claude/gate.d/path-moves.tsv 登记的搬迁换算之后与基准那一版逐字节相同：{'、'.join(moved_kept)}）"
    print(f"  ✓ {LIVE_HOSTS_EXEMPT_TABLE} {count} 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；{note}")
    return 0


def main(argv):
    try:
        breaks()
    except ValueError as error:
        print(f"  ✗ {error}", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE}，或只写认得的项", file=sys.stderr)
        return 2
    if len(argv) >= 3 and argv[0] == "script" and argv[2] == "--" and len(argv) > 3:
        return script_main(argv[1], argv[3:])
    if len(argv) == 2 and argv[0] == "exempt-check":
        return exempt_main(argv[1], None)
    if len(argv) == 4 and argv[0] == "exempt-check" and argv[2] == "--base":
        return exempt_main(argv[1], argv[3])
    print("  ✗ 用法：lib_long_run.py script <命令的当前目录> -- <命令与参数…>；lib_long_run.py exempt-check <仓根> [--base <gate 基准提交>]", file=sys.stderr)
    print("     → 怎么办：照文件头「用法」一段写；平常经 scripts/hosts/detached-run.sh long-run-check -- <命令…> 调", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
