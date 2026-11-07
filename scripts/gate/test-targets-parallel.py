#!/usr/bin/env python3
# admission: always 每次调都在这一刻的树上编、跑这一次给的包，上一次的结论不替这一次作保
# run-condition: command cargo
"""一趟 cargo test 拆成按测试目标并行起：合入后验证（scripts/gate/merge-and-verify.sh）测试那一步与门禁 crash-point-replay 按条复用那几趟用它。

用法：
  test-targets-parallel.py --jobs <N> -- <cargo test 的参数…>      例：--jobs 31 -- --offline --no-fail-fast -p singlefs-core -p singlefs-verification
  test-targets-parallel.py --selftest
为什么：cargo test 一个测试二进制接一个地跑，用例少的二进制跑的时候核就空着，第二台上只用到约 5 核（登记 31）
（records/2026-10-18-里程碑三收尾与近期欠账.md 第 780 行）。
做法：
  ① 编：cargo test <参数> --no-run --message-format=json-render-diagnostics，一次把全部测试二进制编好（cargo 自己按它的 -j 并行编）；
     Compiling 那几行（cargo 打在标准错误）照原样转出去，合入后验证的重编核对读它们。编不过照 cargo 的退出码退，一个目标都不跑。
  ② 列目标：编译输出里 profile.test 为真、带 executable 的产物，一个产物一个目标；包名经 cargo metadata --no-deps 按 package_id 取；
     挑法照 cargo test 认的写：lib 类 → --lib，test → --test <名>，bin → --bin <名>，example → --example <名>，bench → --bench <名>。
  ③ 并行跑：每个目标一条 cargo test <参数里去掉 -p 那几对> -p <包> <挑法>（照 scripts/gate/rerun-failed-tests.py 那样按目标分开起；产物已编好，
     cargo 只核新鲜就跑，运行时的环境与工作目录照 cargo 自己给的）。一条叫一件；按测试函数分了片的目标（下面⑤）一片是一件。
     派活一片一片来（⑤ 定每一片挑哪几件；没有内存上限或一片装得下全部时只有一片），片里线程按 N 条记账（.claude/kb/milestone/03-third-txn.md 记的尾巴空核：
     剩下的件少于 N 时空出来的核没人用）：
       次序：长的先起。用时表（每个目标上一次跑了多少秒，见下）里没有的目标先起（照编译输出的次序），有的按用时从长到短；
       线程：每起一件，RUST_TEST_THREADS 取 max(1, ⌈此刻空着的线程 ÷ 这一片里还没起的件数⌉)、不超过空着的；一件跑完把它的线程放回，再起下一件。
       还没起的比空着的线程多时每件 1 条线程；空着的比还没起的多（开头 N 比件数大，或尾巴上几件一齐跑完）时把多出来的分给后起的、先起的多分一条。
       同时在算的用例线程合起来不超过 N；一片跑完、线程全数放回，再起下一片。
     用时表：cargo metadata 给的 target_directory 下的 singlefs-test-target-durations.tsv（一行「<秒><制表符><包> <挑法>」，跑完收回时把这一趟每个目标的用时
       （分了片的是各片之和）写回、别的行照留，先写临时文件再改名）；TEST_TARGETS_PARALLEL_DURATIONS 设了照它（自证用）；
       metadata 里没有 target_directory 又没设的，不读不写、照编译输出的次序起。
     每件的输出（标准输出与标准错误合成一份）写自己的文件、退出码写另一个文件，
     全部跑完按派活的次序逐个回读、转打到标准输出（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
     收回来的退出码文件数与派出去的件数对不上，整趟判红、退 1。
  ④ 文档测试：没有显式目标选择（--lib、--test 等）时，要测的包里库目标 doctest 为真的（cargo metadata），最后一条 cargo test <参数> -p <这些包> --doc 一起跑（③ 都跑完了，RUST_TEST_THREADS 取 N）。
  ⑤ 内存分片（内存上限不改，脚本自己判，超过这一条拿到的内存就分片跑，每一片都在上限以内）：
     量法、上限的取法、记录名与改名换上的写法、多机配置的找法都用 scripts/mutation/crates-mutation-rows.py 的（memory_module 读进来），不另写一套：
     上限：它的 task_memory_cap_bytes（统一任务服务交的 TASK_SERVER_MEMORY_CAP_BYTES；没有读多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项；
       自己 cgroup 那一层的 memory.max 是有限值时是硬顶、两者取小）。取不到上限的不按内存分，一片开满（同时至多 N 件），说明照打。
     底座（base_usage_of）：每一片派活之前现量自己 cgroup 那一层（连同下面的子 cgroup，cgroup.procs 逐个读）里每个进程的用量加总，
       单个进程的口径是它的 process_usage_bytes（/proc/<pid>/status 的 RssAnon + RssShmem + VmPTE）。它的底座读数只量起那一道的父进程自己；这里量整个 cgroup，
       因为起这一份的门禁链（合入后验证、门禁阶段的 bash 与 python）都算在同一个上限里。cgroup 读不到只量本进程，说明照打。可用量 = 上限 − 底座。
     每件的峰值（量与记，照它文件头「每个工作进程的峰值」）：采样线程每 PEAK_SAMPLE_SECONDS 秒量一次每一件的 cargo 连同它的整棵进程树（它的 process_tree_usage；
       这一份的直接子进程顺 /proc/self/task/<tid>/children 找，哪一件是谁照子进程环境里的 TEST_TARGETS_PARALLEL_PIECE 认），记这一件到此刻的最大；
       比上一次写下的涨了 PEAK_WRITE_STEP_BYTES 以上就写进记录（它的 write_atomically：同目录临时文件、改名换上；整组被杀时已写的留着，记下的比真峰值至多少一个步长），
       跑完再写一次，带上这一件跑了几条测试函数（它日志里 libtest 的「running <N> tests」加和）。比采样间隔还短的尖峰采不到（推的）。
       记在哪：共享根下 test-targets-parallel-peaks/<键的 sha256 前 24 位>/<记录名>（记录名用它的 new_worker_peak_record_name：起写的时刻纳秒 20 位、进程号、8 位随机字符），
       内容一行「<峰值字节> <这一件跑的测试函数条数，没读到写 0> <这一件的 RUST_TEST_THREADS><制表符><键>」；键是「<包> <挑法>」接上影响产物的 cargo 参数
       （去掉 --offline、--no-fail-fast、--locked、--frozen、-q、-v 这几样之后的，--release、--features 这类留着）。共享根：多机配置（它的 test_runs_multi_host_configuration 找的那一份）
       NFS_SHARED_ROOTS 里不含本仓、也不在本仓之内的几段里排在最前的一段（同它的 run_records_directory，不经 ~ 与 $HOME 取，两台读写同一份）。
       TEST_TARGETS_PARALLEL_PEAKS_DIRECTORY 设了照它（自证用）；没设而在自证底下（inside_selftest）的不读不写；配置里没有这一项的不读不写；不读不写时只用这一趟量到的。
       读（measured_target_peak）：每个键只看名字排在最后的 PEAKS_RECENT 份，更旧的读的时候删掉；目标的峰值取这几份与这一趟已经量到的里最大的。
       有一份名字或内容不是这一版的写法、或读不了，这个键照没有实测办，打一行点名那一份与出路（删掉它再跑）。
     估：有实测的目标照它的峰值估（量的，不按这一趟的条数缩放）；没有实测的取保守值，就是可用量本身（一件占满，单独一片）。
     挑（memory_bound_round_of）：每一片派活之前照可用量从还没跑的件里按派活次序挑，放得下（这一片已挑的估值之和加它不超过可用量）就挑上，放不下的留给后面的片；
       一片跑完（量到的写进记录、并进这一趟量到的）再估、再挑下一片。片里照 ③ 派活与分线程。
     单个目标的估值超过可用量（fitted_items）：照 cargo test <参数> -p <包> <挑法> -- <测试二进制的参数> --list --format terse 列出它这一趟要跑的测试函数（N 条；libtest 列的时候
       照样认位置过滤与 --skip），每片至多 k = ⌊可用量 × N ÷ 峰值⌋ 条，按列出的次序均分成 ⌈N ÷ k⌉ 片（一片的全名合起来超过 ARGUMENT_BYTES_PER_PIECE 字节的再分细）；
       每片的估值按「目标的峰值 × 片里条数 ÷ N」算（推的：峰值未必随条数成比例）。每片的命令在测试二进制的参数里去掉原来的位置过滤词（列出来的已经过滤过）、
       没有 --exact 的补上，再接这几条的全名，--skip 照留：一片跑的正好是它那几条。一片跑完量到的照记；再估时目标的峰值涨到让某一片装不下的，那一片照新的峰值再分细。
       列出 0 条的照整个起、估 0（一条都不跑）。harness = false 的目标不认 --list，认不出（本仓此刻没有）。
     撞内存上限：k 是 0（一条测试函数的估值就超过可用量），或可用量不到 1 字节（底座不小于上限），不硬跑：已经跑完的片照回读、照报，之后退 250
       （统一任务服务错误类型表里 memory-cap-hit 那一行，task-server/task_server_lib.py 文件头「错误类型」；同 crates-mutation-rows.py 的「撞内存上限」），
       红句点名目标、测试函数、估值、上限与底座，文档测试不跑。
     结局：分片只改哪几件一起起，测试二进制的参数（门禁 crash-point-replay 按条复用的 --exact --skip 那几对）、按条复用的记库、哨兵与键都不变；
       各片的输出照样各带 cargo 的「Running …」一段，逐条结局合起来与不分片时相同（自证「分片与不分片的逐条结局相同」那一格核）。
  测试二进制的参数：<cargo test 的参数…> 里第二个 `--` 之后的（--nocapture、--exact、--skip <名> 这类）原样接在 ③ 每一件与 ④ 那一条的 `--` 之后，
     一个字不改、不挑（门禁 crash-point-replay 的按条复用拿 --exact --skip 跳过键相同的测试函数）；按测试函数分的片照 ⑤ 另接全名。
  末行 TEST_TARGETS_PARALLEL dispatched=<派出的件数> collected=<收回> failed=<红的件数> jobs=<同时至多几件> test_threads=<每件的线程数，各件不同时写 最少-最多> doc=<yes|no|failed>
     memory_shards=<分了几片> shard_pieces=<每片几件，逗号分> shard_jobs=<每片同时开几件，逗号分> bound=<memory|threads> memory_cap=<字节|none> available=<最后一片的可用量字节|none>；
     末行之前一行「· 内存分片：」写分了几片、每片同时开几件、卡在内存还是线程。
退出码：都绿 0；有件或文档测试红 101；编不过照 cargo 的；撞内存上限 250；收回与派出对不上、cargo metadata 读不出、列不出要分片的目标的测试函数、
  读不进 crates-mutation-rows.py 1；用法错 2。
线程数只照 --jobs（调用方经 capped.sh 给登记值），这里不现算、不写常量；内存只决定一片里挑哪几件，不动线程数。
弄坏开关（只给自证证红用：--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认），一项一行，下一行写打开后判错的自证格：
  TEST_TARGETS_PARALLEL_BREAK=serial  同时只起一件（一个测试二进制接一个地跑）
      「四个目标同时在跑」「空出来的线程分给后起的、长的多分」（同时只起一件，每件都拿满 N 条线程）
  TEST_TARGETS_PARALLEL_BREAK=drop-collection  回读时丢掉最后派出的那一件、也不核收回的件数
      「收回的条数等于派出的」「分片与不分片的逐条结局相同」（分了片的那一趟丢的是 beta 的一片、不分片的那一趟丢的是 tool）
  TEST_TARGETS_PARALLEL_BREAK=ignore-red  件退非 0 也按绿算、退 0
      「红的目标照报、退 101」
  TEST_TARGETS_PARALLEL_BREAK=even-threads  每件的 RUST_TEST_THREADS 一律取 ⌈N ÷ min(N, 片里的件数)⌉（空出来的线程不分给后起的）
      「空出来的线程分给后起的、长的多分」
  TEST_TARGETS_PARALLEL_BREAK=history-ignored  不看用时表，照编译输出的次序起
      「长的先起」「空出来的线程分给后起的、长的多分」（最长的 tool 不先起，多出来的那条线程落到先起的 lib 上）
  TEST_TARGETS_PARALLEL_BREAK=test-arguments-dropped  ③ 每一件与 ④ 那一条都不接测试二进制的参数（--exact --skip 那几对丢了，键相同的测试函数照跑）
      「测试二进制的参数原样交给每个目标」
  TEST_TARGETS_PARALLEL_BREAK=memory-ignored  不看内存上限：一片开满（同时至多 N 件），也不按测试函数分片（会撞上限）
      「可用量装不下 --jobs 份时同时在跑的峰值之和不超过可用量」「单个目标超可用量时按测试函数分片、各片都在可用量以内」「分片与不分片的逐条结局相同」「一片只剩一条还超时按撞内存上限退出、不硬跑」
  TEST_TARGETS_PARALLEL_BREAK=function-shards-off  单个目标的估值超过可用量也不按测试函数分片，整个起、单独占一片
      「单个目标超可用量时按测试函数分片、各片都在可用量以内」「分片与不分片的逐条结局相同」「一片只剩一条还超时按撞内存上限退出、不硬跑」
  TEST_TARGETS_PARALLEL_BREAK=function-shard-last-dropped  按测试函数分片时丢掉列出来的最后一条（那一条哪一片都不在）
      「分片与不分片的逐条结局相同」
  TEST_TARGETS_PARALLEL_BREAK=memory-cap-short-run-anyway  一条测试函数的估值就超过可用量时照整个单独起一片硬跑，不按撞内存上限退 250
      「一片只剩一条还超时按撞内存上限退出、不硬跑」
  TEST_TARGETS_PARALLEL_BREAK=peaks-unrecorded  量到的峰值不写进记录（只在这一趟里用）
      「每个测试目标量到的峰值记进共享根」
自证（--selftest）：PATH 最前面放一份假 cargo——metadata 打一个包 demo（库带文档测试），--no-run 打四个测试目标的产物（lib、test alpha、test beta、bin tool，
  另有一份不是测试的库产物与一份构建脚本产物，不算目标），跑某个目标时记下起止时刻、进程号与 RUST_TEST_THREADS、挑中的测试函数、睡 3 秒，
  每个目标默认一条测试函数 <目标>_case，beta 那一条打 FAILED、退 101；--list 照参数里的 --skip 与位置全名打「<全名>: test」。
  每一趟都换掉 cgroup 的读法（GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 指到临时目录：memory.max 写 max，cgroup.procs 里是自证进程自己）、
  多机配置指到不在的文件、峰值记录放临时目录，内存上限只照这一趟给的 TASK_SERVER_MEMORY_CAP_BYTES（不给就摘掉）。--jobs 4、不给上限：
  「四个目标同时在跑」四条的起止区间两两重叠（最晚的起点早于最早的终点）、每条的 RUST_TEST_THREADS 是 1；
  「收回的条数等于派出的」末行 dispatched=4 collected=4，四个目标的 test result 行都在标准输出里；
  「红的目标照报、退 101」退 101，FAILED 那一行与「-p demo --test beta」都在输出里；
  「编译与文档测试」Compiling 那一行转到了标准错误，文档测试跑了恰好一次。
  另两趟各带一份用时表（tool 30 秒、beta 20 秒、lib 5 秒、alpha 1 秒）：
  「长的先起」--jobs 2：tool 与 beta 先起，lib 与 alpha 的起点都不早于前两条里先跑完的那一条的终点；跑完用时表里四个目标都写回了这一趟的用时；
  「空出来的线程分给后起的、长的多分」--jobs 5：最长的 tool 拿 2 条线程、另三条各 1 条，合起来正好 5 条。
  再一趟带测试二进制的参数 `-- --nocapture --exact --skip alpha_case --skip tool_case`（--jobs 4；假 cargo 跑某个目标时另记一行收到的参数）：
  「测试二进制的参数原样交给每个目标」四个目标与文档测试那一条收到的参数都以 `-- --nocapture --exact --skip alpha_case --skip tool_case` 收尾。
  内存的五格（上限 10 GiB，峰值记录预先写进临时目录，--jobs 4）：
  「可用量装不下 --jobs 份时同时在跑的峰值之和不超过可用量」四个目标各记 3 GiB：任何一件起跑那一刻在跑的各件记的峰值之和不超过 10 GiB，四个都跑了，
    末行 memory_shards=2 shard_pieces=3,1 bound=memory；
  「单个目标超可用量时按测试函数分片、各片都在可用量以内」lib、alpha、tool 各记 1 GiB，beta 记 16 GiB、有四条测试函数（beta_case_two 判 FAILED），
    测试二进制的参数 `-- --exact --skip beta_case_three`：beta 列出三条、每片一条（⌊约 9.9 GiB × 3 ÷ 16 GiB⌋ = 1），beta 起了不止一件、每件挑中的少于列出的条数，
    任何一件起跑那一刻在跑的估值（beta 一件按 16 GiB × 条数 ÷ 列出的条数）之和不超过 10 GiB；
  「分片与不分片的逐条结局相同」同一份记录与参数、不给上限再跑一趟，两趟标准输出里的「test <名> ... <结局>」逐条相同，且分了片的那一趟 beta 起了不止一件；
  「一片只剩一条还超时按撞内存上限退出、不硬跑」tool 记 40 GiB、只有一条测试函数：退 250，输出里有「撞内存上限」、tool_case 与 memory-cap-hit，假 cargo 一件都没起；
  「每个测试目标量到的峰值记进共享根」不给上限、没有记录，alpha 那一件另起一个 python3 占 64 MiB：跑完记录里「demo --test alpha」的峰值不小于 48 MiB、条数写 1。
"""
import concurrent.futures
import fnmatch
import hashlib
import importlib.util
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

BREAK_VARIABLE = "TEST_TARGETS_PARALLEL_BREAK"
LIBRARY_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}
SELECTOR_OPTION_BY_KIND = {"test": "--test", "bin": "--bin", "example": "--example", "bench": "--bench"}
SUMMARY_PREFIX = "TEST_TARGETS_PARALLEL"
SELFTEST_CELLS = ("四个目标同时在跑", "收回的条数等于派出的", "红的目标照报、退 101", "编译与文档测试", "长的先起",
                  "空出来的线程分给后起的、长的多分", "测试二进制的参数原样交给每个目标",
                  "可用量装不下 --jobs 份时同时在跑的峰值之和不超过可用量", "单个目标超可用量时按测试函数分片、各片都在可用量以内",
                  "分片与不分片的逐条结局相同", "一片只剩一条还超时按撞内存上限退出、不硬跑", "每个测试目标量到的峰值记进共享根")   # 自证的格名，判错行照抄
DURATIONS_VARIABLE = "TEST_TARGETS_PARALLEL_DURATIONS"   # 用时表换位置（自证用）；不设取 target_directory 下的 DURATIONS_FILE_NAME
DURATIONS_FILE_NAME = "singlefs-test-target-durations.tsv"
REPOSITORY = os.path.realpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", ".."))
MEMORY_MODULE_PATH = os.path.join(REPOSITORY, "scripts", "mutation", "crates-mutation-rows.py")   # 量法、上限的取法与记录的写法都用它的（文件头 ⑤）
PEAKS_DIRECTORY_VARIABLE = "TEST_TARGETS_PARALLEL_PEAKS_DIRECTORY"   # 峰值记录换位置（自证用）；不设取共享根下的 PEAKS_DIRECTORY_NAME
PEAKS_DIRECTORY_NAME = "test-targets-parallel-peaks"
PIECE_VARIABLE = "TEST_TARGETS_PARALLEL_PIECE"   # 设在每一件的环境里：采样线程照它认这一份的子进程是哪一件
PEAK_SAMPLE_SECONDS = 0.1             # 采样线程隔多久量一次每一件的整棵进程树
PEAK_WRITE_STEP_BYTES = 16 << 20      # 一件的最大比上一次写下的涨了这么多才重写记录：记下的比真峰值至多少这么多
PEAKS_RECENT = 32                     # 每个键只看最新的这么多份记录（一趟一件一份；推的：够盖住几十趟，旧的偏大读数也挤得掉）
ARGUMENT_BYTES_PER_PIECE = 1 << 19    # 按测试函数分的一片里全名合起来至多这么多字节（推的：远在 Linux 一条命令的参数上限以下）
MEMORY_CAP_HIT_EXIT = 250             # 统一任务服务错误类型表里 memory-cap-hit 那一行的退出码（task-server/task_server_lib.py 文件头「错误类型」）
CARGO_ARGUMENTS_NOT_IN_KEY = {"--offline", "--no-fail-fast", "--locked", "--frozen", "-q", "--quiet", "-v", "--verbose"}   # 不改产物的 cargo 参数，不进峰值记录的键
LIBTEST_OPTIONS_WITH_VALUE = {"--skip", "--test-threads", "--format", "--logfile", "--color", "-Z", "--shuffle-seed"}   # 测试二进制的参数里后面跟一个值的选项
RECORD_NAME_FORM = re.compile(r"[0-9]{20}-[0-9]+-[a-z0-9]{8}")
RECORD_CONTENT_FORM = re.compile(r"([1-9][0-9]*) ([0-9]+) ([1-9][0-9]*)\t(.+)\n")
LISTED_FUNCTION_FORM = re.compile(r"(.+): (?:test|benchmark)")
RUNNING_COUNT_FORM = re.compile(r"running ([0-9]+) tests?")
FUNCTIONS_SHOWN = 5                   # 红句与分片那一行里至多列几条测试函数的全名


def broken(item):
    return os.environ.get(BREAK_VARIABLE, "") == item


def split_arguments(arguments):
    """cargo test 的参数 → (去掉 -p / --package 之后的参数, 点名的包, `--` 之后交给测试二进制的参数)。"""
    if "--" in arguments:
        cut = arguments.index("--")
        cargo_arguments, test_arguments = arguments[:cut], arguments[cut + 1:]
    else:
        cargo_arguments, test_arguments = arguments, []
    kept, packages, position = [], [], 0
    while position < len(cargo_arguments):
        argument = cargo_arguments[position]
        if argument in ("-p", "--package") and position + 1 < len(cargo_arguments):
            packages.append(cargo_arguments[position + 1])
            position += 2
            continue
        if argument.startswith("--package="):
            packages.append(argument.split("=", 1)[1])
        elif argument.startswith("-p") and len(argument) > 2:
            packages.append(argument[2:])
        else:
            kept.append(argument)
        position += 1
    return kept, packages, test_arguments


def package_options(packages):
    return [word for package in packages for word in ("-p", package)]


def execution_arguments(arguments):
    """目标已经由编译产物选定；逐目标执行时去掉会扩大范围的选择参数。"""
    kept, selected_targets, position = [], False, 0
    target_flags = {"--lib", "--bins", "--tests", "--examples", "--benches", "--all-targets", "--doc"}
    target_values = {"--bin", "--test", "--example", "--bench"}
    while position < len(arguments):
        argument = arguments[position]
        option = argument.split("=", 1)[0]
        if option in target_flags or option in target_values:
            selected_targets = True
            position += 2 if option in target_values and "=" not in argument else 1
            continue
        if option in {"--workspace", "--all", "--exclude"}:
            position += 2 if option == "--exclude" and "=" not in argument else 1
            continue
        kept.append(argument)
        position += 1
    return kept, selected_targets


def selector_of(target):
    """编译产物的 target → cargo test 挑它的参数；认不出的种类交 None。"""
    kinds = set(target.get("kind") or [])
    if kinds & LIBRARY_KINDS:
        return ["--lib"]
    for kind, option in SELECTOR_OPTION_BY_KIND.items():
        if kind in kinds:
            return [option, target.get("name", "")]
    return None


def read_metadata(cargo_arguments):
    """元数据跟随清单与配置：交 (ID 到包名, 文档测试开关, 默认或 --workspace 选中的成员, target 目录)；读不出交 None 与原因。"""
    command = ["cargo", "metadata", "--no-deps", "--format-version", "1"]
    position = 0
    while position < len(cargo_arguments):
        argument = cargo_arguments[position]
        option = argument.split("=", 1)[0]
        if option in {"--manifest-path", "--config", "--features", "-F", "-Z"}:
            command.append(argument)
            if "=" not in argument and position + 1 < len(cargo_arguments):
                position += 1
                command.append(cargo_arguments[position])
        elif argument in {"--offline", "--locked", "--frozen", "--all-features", "--no-default-features"}:
            command.append(argument)
        position += 1
    completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if completed.returncode != 0:
        return None, f"cargo metadata 退 {completed.returncode}：{completed.stderr.strip()[-300:]}"
    try:
        metadata = json.loads(completed.stdout)
    except ValueError as error:
        return None, f"cargo metadata 打出来的不是 JSON（{error}）"
    names, doctests = {}, {}
    for package in metadata.get("packages", []):
        names[package.get("id")] = package.get("name")
        doctests[package.get("name")] = any(set(target.get("kind") or []) & LIBRARY_KINDS and target.get("doctest")
                                            for target in package.get("targets", []))
    workspace = "--workspace" in cargo_arguments or "--all" in cargo_arguments
    selected = metadata.get("workspace_members", []) if workspace else metadata.get("workspace_default_members", metadata.get("workspace_members", []))
    excluded = [argument.split("=", 1)[1] for argument in cargo_arguments if argument.startswith("--exclude=")]
    excluded += [cargo_arguments[index + 1] for index, argument in enumerate(cargo_arguments[:-1]) if argument == "--exclude"]
    members = [names[member] for member in selected if member in names
               and not any(fnmatch.fnmatchcase(names[member], pattern) for pattern in excluded)]
    return (names, doctests, members, metadata.get("target_directory")), None


def durations_path(target_directory):
    """用时表在哪（文件头 ③「用时表」）：TEST_TARGETS_PARALLEL_DURATIONS 设了照它，不设取 target_directory 下的那一份；都没有交 None（不读不写）。"""
    if os.environ.get(DURATIONS_VARIABLE):
        return os.environ[DURATIONS_VARIABLE]
    return os.path.join(target_directory, DURATIONS_FILE_NAME) if target_directory else None


def read_durations(path):
    """用时表 → {「<包> <挑法>」: 秒}；文件不在、行写坏的跳过。"""
    durations = {}
    if not path or not os.path.isfile(path):
        return durations
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            seconds, _, key = line.rstrip("\n").partition("\t")
            try:
                durations[key] = float(seconds)
            except ValueError:
                continue
    return durations


def write_durations(path, durations):
    """用时表整份写回：先写同目录临时文件再改名；写不进不碍这一趟的结论。"""
    if not path:
        return
    try:
        temporary = f"{path}.{os.getpid()}.tmp"
        with open(temporary, "w", encoding="utf-8") as handle:
            for key, seconds in sorted(durations.items()):
                handle.write(f"{seconds:.3f}\t{key}\n")
        os.replace(temporary, path)
    except OSError:
        pass


def unit_key(package, selector):
    return f"{package} {' '.join(selector)}"


def threads_for_next(free, remaining, jobs, total):
    """下一件拿几条线程（文件头 ③「线程」）：max(1, ⌈空着的 ÷ 片里还没起的⌉)、不超过空着的；弄坏开关 even-threads 一律 ⌈N ÷ min(N, 片里的件数)⌉。"""
    if broken("even-threads"):
        return max(1, math.ceil(jobs / max(1, min(jobs, total))))
    return max(1, min(free, math.ceil(free / max(1, remaining))))


def build(cargo_arguments, packages):
    """① 编：交 (退出码, [产物 JSON…])；非 JSON 的标准输出行照原样打出去。"""
    command = ["cargo", "test"] + cargo_arguments + package_options(packages) + ["--no-run", "--message-format=json-render-diagnostics"]
    completed = subprocess.run(command, stdout=subprocess.PIPE, text=True)
    artifacts = []
    for line in completed.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            print(line)
            continue
        if isinstance(message, dict) and message.get("reason") == "compiler-artifact":
            artifacts.append(message)
        elif isinstance(message, dict) and message.get("reason") == "compiler-message":
            rendered = (message.get("message") or {}).get("rendered")
            if rendered:
                sys.stderr.write(rendered)
    sys.stdout.flush()
    return completed.returncode, artifacts


_MEMORY_MODULE = []


def memory_module():
    """scripts/mutation/crates-mutation-rows.py（量法、上限的取法、记录名与改名换上、多机配置的找法，文件头 ⑤），第一次用到时才读进来。"""
    if not _MEMORY_MODULE:
        specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_test_targets_parallel", MEMORY_MODULE_PATH)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _MEMORY_MODULE.append(module)
    return _MEMORY_MODULE[0]


class MemoryToolsUnavailable(Exception):
    """crates-mutation-rows.py 读不进来：量不了内存、取不到上限。"""


def memory_settings(environment, roots):
    """这一趟的上限与峰值记录放哪（文件头 ⑤「上限」「记在哪」）：(上限字节或 None, 上限的说明, 峰值记录目录或 None, 目录的说明)。
    读不进 crates-mutation-rows.py 抛 MemoryToolsUnavailable（不开满）。弄坏开关 memory-ignored：上限交 None。"""
    try:
        module = memory_module()
    except Exception as error:   # 它读的库有一份写坏了的各种异常都在这里：照实报，整趟不派
        raise MemoryToolsUnavailable(f"{type(error).__name__}：{error}") from error
    cap, cap_text = (None, "弄坏开关 memory-ignored：不看内存上限") if broken("memory-ignored") else module.task_memory_cap_bytes(environment)
    directory, directory_text = peaks_directory_of(environment, roots)
    return cap, cap_text, directory, directory_text


def peak_key(unit_arguments, package, selector):
    """峰值记录的键（文件头 ⑤「记在哪」）：「<包> <挑法>」接上影响产物的 cargo 参数。"""
    return " ".join([package, *selector, *(argument for argument in unit_arguments if argument not in CARGO_ARGUMENTS_NOT_IN_KEY)])


def key_directory(peaks_directory, key):
    return os.path.join(peaks_directory, hashlib.sha256(key.encode("utf-8", "surrogateescape")).hexdigest()[:24])


def peaks_directory_of(environment, roots):
    """峰值记录放哪（文件头 ⑤「记在哪」）：(目录或 None, 说明)。roots 是不许落进去的几棵树（被测的树与本仓）。"""
    given = environment.get(PEAKS_DIRECTORY_VARIABLE, "").strip()
    if given:
        return given, f"{PEAKS_DIRECTORY_VARIABLE} 指的 {given}"
    if inside_selftest():
        return None, f"在自证底下又没设 {PEAKS_DIRECTORY_VARIABLE}：不读不写共享根里的峰值记录，只用这一趟量到的"
    module = memory_module()
    configuration, present = module.test_runs_multi_host_configuration(environment)
    if not present:
        return None, f"多机配置 {configuration} 不在：峰值记录没处放，不读不写，只用这一趟量到的"
    admission = module.admission_module()
    try:
        shared_roots = admission.configuration_value(configuration, "NFS_SHARED_ROOTS")
    except admission.InputManifestError as error:
        return None, f"{error}：峰值记录不读不写，只用这一趟量到的"
    repositories = [os.path.realpath(path) for path in roots]

    def overlaps(candidate):
        real = os.path.realpath(candidate)
        return any(real == repository or real.startswith(repository.rstrip(os.sep) + os.sep) or repository.startswith(real.rstrip(os.sep) + os.sep)
                   for repository in repositories)

    candidates = [path for path in shared_roots.split(":") if os.path.isabs(path) and not overlaps(path)]
    if not candidates:
        return None, f"多机配置 {configuration} 里没有不含本仓的 NFS_SHARED_ROOTS（两台同一绝对路径的共享根）：峰值记录不读不写，只用这一趟量到的"
    return os.path.join(candidates[0], PEAKS_DIRECTORY_NAME), f"多机配置 {configuration} 的 NFS_SHARED_ROOTS 里的 {candidates[0]}"


def measured_target_peak(peaks_directory, key):
    """一个键的峰值记录（文件头 ⑤「读」）：(最新 PEAKS_RECENT 份里最大的字节或 None, 说明)。没有记录交 (None, "")；有一份不是这一版的写法交 (None, 点名与出路)。
    读的时候删掉比最新那几份更旧的。"""
    if peaks_directory is None:
        return None, ""
    directory = key_directory(peaks_directory, key)
    try:
        names = sorted(name for name in os.listdir(directory) if not name.startswith("."))
    except FileNotFoundError:
        return None, ""
    except OSError as error:
        return None, f"峰值记录 {directory} 读不了（{type(error).__name__}：{error}）；这个目标照没有实测办，出路：删掉 {directory} 再跑"
    for stale in names[:-PEAKS_RECENT]:
        try:
            os.remove(os.path.join(directory, stale))
        except OSError:
            pass
    peaks = []
    for name in names[-PEAKS_RECENT:]:
        path = os.path.join(directory, name)
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                content = handle.read()
        except OSError as error:
            return None, f"峰值记录 {path} 读不了（{type(error).__name__}：{error}）；这个目标照没有实测办，出路：删掉那一份再跑"
        found = RECORD_CONTENT_FORM.fullmatch(content)
        if not RECORD_NAME_FORM.fullmatch(name) or found is None or found.group(4) != key:
            return None, (f"峰值记录 {path} 不是这一版的写法（名字要是「<纳秒 20 位>-<进程号>-<8 位随机字符>」，内容一行「<峰值字节> <条数> <线程数><制表符>{key}」）；"
                          "这个目标照没有实测办，出路：删掉那一份再跑")
        peaks.append(int(found.group(1)))
    return (max(peaks) if peaks else None), ""


def base_usage_of(environment):
    """底座（文件头 ⑤「底座」）：(字节, 说明)。自己 cgroup 那一层连同子 cgroup 里每个进程的用量加总；读不到 cgroup 只量本进程。
    cgroup 的两处读法同 crates-mutation-rows.py 的 task_memory_cap_bytes（GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 只给自证换）。"""
    module = memory_module()
    listing = environment.get("GATE_MUTATION_CGROUP_LISTING", "").strip() or module.CGROUP_LISTING
    root = environment.get("GATE_MUTATION_CGROUP_ROOT", "").strip() or module.CGROUP_ROOT
    own = module.process_usage_bytes(os.getpid()) or 0
    try:
        with open(listing, encoding="utf-8") as handle:
            scope = next((line[3:] for line in handle.read().splitlines() if line.startswith("0::")), None)
    except OSError as error:
        return own, f"读不到 {listing}（{type(error).__name__}：{error}），只量到本进程"
    if scope is None:
        return own, f"{listing} 里没有 cgroup v2 的那一行（0::<路径>），只量到本进程"
    directory = os.path.join(root, scope.strip().lstrip("/"))
    processes = set()
    for current, _directories, files in os.walk(directory):
        if "cgroup.procs" not in files:
            continue
        try:
            with open(os.path.join(current, "cgroup.procs"), encoding="ascii") as handle:
                processes.update(int(word) for word in handle.read().split())
        except (OSError, ValueError):
            continue
    if not processes:
        return own, f"{directory} 下读不到 cgroup.procs，只量到本进程"
    total = sum(module.process_usage_bytes(pid) or 0 for pid in sorted(processes))
    return total, f"{directory} 里 {len(processes)} 个进程（RssAnon + RssShmem + VmPTE 加总）"


def own_child_processes():
    """这一份此刻的直接子进程号：顺 /proc/self/task/<tid>/children（各件是工作线程起的，子进程挂在起它的那条线程下面）。"""
    children = []
    try:
        threads = os.listdir("/proc/self/task")
    except OSError:
        return children
    for thread in threads:
        try:
            with open(f"/proc/self/task/{thread}/children", encoding="ascii") as handle:
                children += [int(word) for word in handle.read().split()]
        except (OSError, ValueError):
            continue
    return children


def piece_of_process(pid):
    """子进程 pid 是哪一件：它环境里 TEST_TARGETS_PARALLEL_PIECE 的值；不是一件（列测试函数、文档测试那几条）或读不到交 None。"""
    try:
        with open(f"/proc/{pid}/environ", "rb") as handle:
            entries = handle.read().split(b"\0")
    except OSError:
        return None
    marker = PIECE_VARIABLE.encode("ascii") + b"="
    for entry in entries:
        if entry.startswith(marker) and entry[len(marker):].isdigit():
            return int(entry[len(marker):])
    return None


class PeakSampler:
    """采样线程（文件头 ⑤「每件的峰值」）：每 PEAK_SAMPLE_SECONDS 秒量一次每一件的整棵进程树，记每件到此刻的最大；涨了一个步长就写一次记录。"""

    def __init__(self, environment, peaks_directory):
        self.environment = environment
        self.peaks_directory = None if broken("peaks-unrecorded") else peaks_directory
        self.lock = threading.Lock()
        self.peaks = {}          # 件号 → 到此刻的最大字节
        self.written = {}        # 件号 → (记录名, 写下的字节)
        self.pieces = {}         # 件号 → (键, 条数, 线程数)：开跑时登记
        self.finished = set()
        self.problems = []
        self.stopped = threading.Event()
        self.thread = threading.Thread(target=self.loop, daemon=True)

    def start(self):
        self.thread.start()

    def stop(self):
        self.stopped.set()
        self.thread.join()

    def register(self, piece, key, functions, threads):
        with self.lock:
            self.pieces[piece] = (key, functions, threads)

    def loop(self):
        while not self.stopped.is_set():
            self.sample()
            self.stopped.wait(PEAK_SAMPLE_SECONDS)

    def sample(self):
        for pid in own_child_processes():
            piece = piece_of_process(pid)
            if piece is None:
                continue
            usage = memory_module().process_tree_usage(pid, self.environment)
            if usage is None:
                continue
            with self.lock:
                if piece in self.finished or piece not in self.pieces:
                    continue
                if usage > self.peaks.get(piece, 0):
                    self.peaks[piece] = usage
                written = self.written.get(piece)
                if written is None or self.peaks[piece] >= written[1] + PEAK_WRITE_STEP_BYTES:
                    self.write(piece, self.pieces[piece][1])

    def write(self, piece, functions):
        """把这一件到此刻的最大写进记录（调用方拿着 self.lock）；写不进记一条、不碍这一件的结局。"""
        key, _registered, threads = self.pieces[piece]
        peak = self.peaks.get(piece, 0)
        if self.peaks_directory is None or peak <= 0:
            return
        name = self.written[piece][0] if piece in self.written else memory_module().new_worker_peak_record_name()
        path = os.path.join(key_directory(self.peaks_directory, key), name)
        try:
            memory_module().write_atomically(path, f"{peak} {functions} {threads}\t{key}\n")
        except OSError as error:
            self.problems.append(f"峰值记录没写进 {path}（{type(error).__name__}：{error}）；下一趟少这一份读数，不碍这一趟的结局")
            return
        self.written[piece] = (name, peak)

    def finish(self, piece, functions):
        """一件跑完：带上跑了几条测试函数再写一次，之后采样线程不再碰它；交这一件量到的最大（没采到交 None）。"""
        with self.lock:
            self.write(piece, functions)
            self.finished.add(piece)
            return self.peaks.get(piece)


class WorkItem:
    """要起的一件：unit 是目标的下标；names 是按测试函数分的这一片的全名（None 是整个目标），functions_in_target 是分片那一刻列出的条数；estimate 是估值字节。"""

    def __init__(self, unit, names=None, functions_in_target=0, estimate=0, estimate_source=""):
        self.unit, self.names, self.functions_in_target = unit, names, functions_in_target
        self.estimate, self.estimate_source = estimate, estimate_source


class MemoryCapHit(Exception):
    """一条测试函数的估值就超过可用量（文件头 ⑤「撞内存上限」）。"""

    def __init__(self, unit, names, peak, total):
        super().__init__(unit)
        self.unit, self.names, self.peak, self.total = unit, names, peak, total


class ListingFailed(Exception):
    """要分片的目标列不出测试函数。"""


def split_names(names, at_most):
    """按列出的次序把全名均分成每片至多 at_most 条（一片的全名合起来超过 ARGUMENT_BYTES_PER_PIECE 字节的再分细）；弄坏开关 function-shard-last-dropped 下丢掉最后一条。"""
    if broken("function-shard-last-dropped"):
        names = names[:-1]
    count = max(1, math.ceil(len(names) / at_most))
    while True:
        size, extra = divmod(len(names), count)
        chunks, start = [], 0
        for index in range(count):
            end = start + size + (1 if index < extra else 0)
            chunks.append(names[start:end])
            start = end
        if count >= len(names) or all(sum(len(name) + 1 for name in chunk) <= ARGUMENT_BYTES_PER_PIECE for chunk in chunks):
            return [chunk for chunk in chunks if chunk]
        count += 1


def fitted_items(pending, available, peak_of, list_names):
    """给每一件定估值（文件头 ⑤「估」「单个目标的估值超过可用量」）：估值超过可用量的目标按测试函数分片，分到一条还超抛 MemoryCapHit。"""
    fitted = []
    for item in pending:
        peak, source = peak_of(item.unit)
        if item.names is None:
            estimate = available if peak is None else peak
            if estimate <= available:
                fitted.append(WorkItem(item.unit, None, 0, estimate, source))
                continue
            if broken("function-shards-off"):
                fitted.append(WorkItem(item.unit, None, 0, available, f"{source}，超过可用量也不分片（弄坏开关 function-shards-off），单独占一片"))
                continue
            names = list_names(item.unit)
            total = len(names)
            if total == 0:
                fitted.append(WorkItem(item.unit, None, 0, 0, f"{source}，列出 0 条测试函数，只起不跑"))
                continue
        else:
            names, total = item.names, item.functions_in_target
            estimate = available if peak is None else math.ceil(peak * len(names) / total)
            if estimate <= available:
                fitted.append(WorkItem(item.unit, names, total, estimate, f"{source} × {len(names)} ÷ {total}（推的）"))
                continue
        at_most = available * total // peak
        if at_most < 1:
            if broken("memory-cap-short-run-anyway"):
                fitted.append(WorkItem(item.unit, item.names, total, available, f"{source}，一条就超过可用量也照起（弄坏开关 memory-cap-short-run-anyway）"))
                continue
            raise MemoryCapHit(item.unit, names, peak, total)
        for chunk in split_names(names, at_most):
            fitted.append(WorkItem(item.unit, chunk, total, math.ceil(peak * len(chunk) / total), f"{source} × {len(chunk)} ÷ {total}（推的）"))
    return fitted


def memory_bound_round_of(pending, available):
    """下一片挑哪几件（文件头 ⑤「挑」）：按派活次序，这一片已挑的估值之和加它不超过可用量就挑上；交 (挑上的, 估值之和)。"""
    chosen, total = [], 0
    for item in pending:
        if total + item.estimate <= available:
            chosen.append(item)
            total += item.estimate
    return chosen, total


def libtest_arguments_for_names(passed_arguments, names):
    """按测试函数分的一片交给测试二进制的参数（文件头 ⑤）：去掉原来的位置过滤词，没有 --exact 的补上，再接这几条的全名。"""
    kept, position = [], 0
    while position < len(passed_arguments):
        argument = passed_arguments[position]
        if argument in LIBTEST_OPTIONS_WITH_VALUE and position + 1 < len(passed_arguments):
            kept += passed_arguments[position:position + 2]
            position += 2
            continue
        if argument.startswith("-"):
            kept.append(argument)
        position += 1
    return kept + ([] if "--exact" in kept else ["--exact"]) + list(names)


def functions_run_in(log_path):
    """一件的日志里 libtest 报的「running <N> tests」加和；读不了交 0。"""
    try:
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            return sum(int(found.group(1)) for found in (RUNNING_COUNT_FORM.fullmatch(line.strip()) for line in handle) if found)
    except OSError:
        return 0


def format_bytes(size):
    return "读不到" if size is None else memory_module().format_size(size)


def run(jobs, arguments):
    cargo_arguments, packages, test_arguments = split_arguments(arguments)
    unit_arguments, selected_targets = execution_arguments(cargo_arguments)
    build_exit, artifacts = build(cargo_arguments, packages)
    if build_exit != 0:
        print(f"  ✗ cargo test --no-run 退 {build_exit}：测试二进制没编成，一个目标都没跑")
        print("  → 怎么办：看上面 cargo 打的编译错误；编成了再跑这一步")
        return build_exit
    metadata, problem = read_metadata(cargo_arguments)
    if metadata is None:
        print(f"  ✗ 读不出包名与文档测试：{problem}")
        print("  → 怎么办：在树根跑 cargo metadata --no-deps --format-version 1 看它为什么退；读不出包名就派不了按目标分开的 cargo test")
        return 1
    names, doctests, members, target_directory = metadata
    selected_packages = members
    if packages and not ({"--workspace", "--all"} & set(cargo_arguments)):
        selected_packages = []
        for specification in packages:
            pattern = specification.split("#")[-1].split("@")[0]
            matched = [name for name in doctests if fnmatch.fnmatchcase(name, pattern)]
            matched += [name for identity, name in names.items() if identity == specification and name not in matched]
            if not matched and not selected_targets:
                print(f"  ✗ 无法把包选择 {specification} 对应到元数据，文档测试范围判不了")
                print("  → 怎么办：用 cargo metadata 列出的包名传 -p，不能把未识别的包当成没有文档测试")
                return 1
            selected_packages.extend(name for name in matched if name not in selected_packages)
    units = []
    for artifact in artifacts:
        if not artifact.get("executable") or not (artifact.get("profile") or {}).get("test"):
            continue
        selector = selector_of(artifact.get("target") or {})
        package = names.get(artifact.get("package_id"))
        if selector is None or package is None:
            print(f"  ✗ 认不出这个测试产物属于哪个包、怎么挑：{artifact.get('package_id')} {artifact.get('target')}")
            print("  → 怎么办：看 cargo metadata 与编译产物的 package_id 对不对得上；对不上就是 cargo 的输出格式变了，照它改 selector_of / read_metadata")
            return 1
        units.append((package, selector))
    history_path = durations_path(target_directory)
    history = {} if broken("history-ignored") else read_durations(history_path)
    # 长的先起（文件头 ③「次序」）：用时表里没有的先起、照编译输出的次序，有的按用时从长到短
    order = sorted(range(len(units)), key=lambda index: (unit_key(*units[index]) in history, -history.get(unit_key(*units[index]), 0.0), index))
    environment = dict(os.environ)
    # 测试二进制的参数原样接在 `--` 之后（文件头「测试二进制的参数」）；弄坏开关 test-arguments-dropped 下丢掉
    passed_arguments = [] if broken("test-arguments-dropped") else test_arguments
    try:
        cap, cap_text, peaks_directory, peaks_text = memory_settings(environment, (os.getcwd(), REPOSITORY))
    except MemoryToolsUnavailable as error:
        print(f"  ✗ 读不进 {MEMORY_MODULE_PATH}（{error}）：量不了内存、取不到上限，一件都没派")
        print("  → 怎么办：先让 python3 scripts/mutation/crates-mutation-rows.py --help 跑得起来（它读的库有一份写坏了就在这里报），再跑这一步")
        return 1
    keys = [peak_key(unit_arguments, *unit) for unit in units]
    recorded, record_notes = {}, []
    for key in sorted(set(keys)):
        recorded[key], record_note = measured_target_peak(peaks_directory, key)
        if record_note:
            record_notes.append(record_note)
    measured_now = {}   # 键 → 这一趟量到的最大
    listed = {}         # 目标下标 → 列出的测试函数全名

    def peak_of(unit):
        key = keys[unit]
        values = [value for value in (recorded.get(key), measured_now.get(key)) if value]
        if not values:
            return None, "没有实测，取保守值（可用量本身）"
        return max(values), f"量的 {format_bytes(max(values))}"

    def list_names(unit):
        if unit not in listed:
            package, selector = units[unit]
            command = ["cargo", "test"] + unit_arguments + ["-p", package] + selector + ["--"] + passed_arguments + ["--list", "--format", "terse"]
            completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment)
            if completed.returncode != 0:
                raise ListingFailed(f"-p {package} {' '.join(selector)} 列测试函数退 {completed.returncode}：{completed.stderr.strip()[-300:]}")
            listed[unit] = [found.group(1) for found in (LISTED_FUNCTION_FORM.fullmatch(line.strip()) for line in completed.stdout.splitlines()) if found]
        return listed[unit]

    for note in record_notes:
        print(f"  ! {note}")
    print(f"  · 内存分片：上限 {format_bytes(cap) if cap is not None else '没有'}（{cap_text}）；峰值记录：{peaks_directory or '不读不写'}（{peaks_text}）")
    sys.stdout.flush()
    sampler = PeakSampler(environment, peaks_directory)
    if not os.path.exists(f"/proc/self/task/{os.getpid()}/children"):
        print("  ! 这台内核没有 /proc/<pid>/task/<tid>/children：量不了每件的峰值，这一趟不记；没有实测的目标照保守值估")
    sampler.start()
    scratch = tempfile.mkdtemp(prefix="test-targets-parallel-")
    thread_counts, elapsed, dispatched, rounds = {}, {}, [], []
    stop_reason, stop_detail, available = None, None, None

    def run_piece(piece, item, threads):
        package, selector = units[item.unit]
        piece_arguments = passed_arguments if item.names is None else libtest_arguments_for_names(passed_arguments, item.names)
        command = ["cargo", "test"] + unit_arguments + ["-p", package] + selector + (["--"] + piece_arguments if piece_arguments else [])
        sampler.register(piece, keys[item.unit], len(item.names or ()), threads)
        log_path = os.path.join(scratch, f"{piece}.log")
        piece_started = time.time()
        with open(log_path, "w", encoding="utf-8") as output:
            exit_code = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT,
                                       env=dict(environment, RUST_TEST_THREADS=str(threads), **{PIECE_VARIABLE: str(piece)})).returncode
        elapsed[piece] = time.time() - piece_started
        peak = sampler.finish(piece, functions_run_in(log_path) or len(item.names or ()))
        if peak:
            measured_now[keys[item.unit]] = max(peak, measured_now.get(keys[item.unit], 0))
        with open(os.path.join(scratch, f"{piece}.exit"), "w", encoding="utf-8") as handle:
            handle.write(f"{exit_code}\n")

    pending = [WorkItem(index) for index in order]
    try:
        while pending:
            if cap is None:
                chosen, chosen_total = list(pending), None
            else:
                base, base_text = base_usage_of(environment)
                available = cap - base
                if available < 1:
                    stop_reason, stop_detail = "memory-cap-hit", f"底座 {format_bytes(base)}（{base_text}）不小于上限 {format_bytes(cap)}，一件都放不下"
                    break
                try:
                    pending = fitted_items(pending, available, peak_of, list_names)
                except MemoryCapHit as hit:
                    package, selector = units[hit.unit]
                    shown = "、".join(hit.names[:FUNCTIONS_SHOWN]) + (f"…另有 {len(hit.names) - FUNCTIONS_SHOWN} 条" if len(hit.names) > FUNCTIONS_SHOWN else "")
                    stop_reason = "memory-cap-hit"
                    stop_detail = (f"测试目标 -p {package} {' '.join(selector)} 量的峰值 {format_bytes(hit.peak)}、这一趟要跑 {hit.total} 条，一条测试函数就估到 "
                                   f"{format_bytes(math.ceil(hit.peak / hit.total))}（峰值 × 1 ÷ {hit.total}，推的），可用量只有 {format_bytes(available)}"
                                   f"（上限 {format_bytes(cap)} − 底座 {format_bytes(base)}，{base_text}）：分到一片只剩一条还装不下；是这几条测试函数：{shown}")
                    break
                chosen, chosen_total = memory_bound_round_of(pending, available)
                listing = "、".join(f"-p {units[item.unit][0]} {' '.join(units[item.unit][1])}"
                                   + (f"（{len(item.names)} 条：{'、'.join(item.names[:FUNCTIONS_SHOWN])}{'…' if len(item.names) > FUNCTIONS_SHOWN else ''}）" if item.names else "")
                                   + f" 估 {format_bytes(item.estimate)}（{item.estimate_source}）" for item in chosen[:8])
                print(f"  · 内存分片：这一片 {len(chosen)} 件（还没跑的 {len(pending)} 件里），估值合计 {format_bytes(chosen_total)}，可用 {format_bytes(available)}"
                      f"（上限 {format_bytes(cap)} − 底座 {format_bytes(base)}，{base_text}）：{listing}{'…' if len(chosen) > 8 else ''}")
                sys.stdout.flush()
            chosen_ids = {id(item) for item in chosen}
            pending = [item for item in pending if id(item) not in chosen_ids]
            concurrent_jobs = 1 if broken("serial") else max(1, min(jobs, len(chosen)))
            rounds.append((len(chosen), concurrent_jobs))
            # 派活（文件头 ③「线程」）：线程按 --jobs 记账，每起一件按此刻空着的与片里还没起的分；一件跑完放回它的线程再起下一件
            round_pending, running, free = list(chosen), {}, jobs
            with concurrent.futures.ThreadPoolExecutor(max_workers=concurrent_jobs) as executor:
                while round_pending or running:
                    while round_pending and len(running) < concurrent_jobs and (free > 0 or not running or broken("even-threads")):
                        item = round_pending.pop(0)
                        threads = jobs if broken("serial") else threads_for_next(free, len(round_pending) + 1, jobs, len(chosen))
                        piece = len(dispatched)
                        dispatched.append(item)
                        thread_counts[piece] = threads
                        free -= threads
                        running[executor.submit(run_piece, piece, item, threads)] = threads
                    done, _ = concurrent.futures.wait(list(running), return_when=concurrent.futures.FIRST_COMPLETED)
                    for future in done:
                        free += running.pop(future)
                        future.result()
    except ListingFailed as error:
        stop_reason, stop_detail = "listing-failed", str(error)
    sampler.stop()
    for problem in sampler.problems[:3]:
        print(f"  ! {problem}")
    if history_path and not broken("history-ignored"):
        merged = read_durations(history_path)
        per_unit = {}
        for piece, seconds in elapsed.items():
            key = unit_key(*units[dispatched[piece].unit])
            per_unit[key] = per_unit.get(key, 0.0) + seconds
        merged.update(per_unit)
        write_durations(history_path, merged)
    collected, failed = 0, []
    collected_pieces = list(enumerate(dispatched))
    if broken("drop-collection") and collected_pieces:
        collected_pieces = collected_pieces[:-1]
    for piece, item in collected_pieces:
        package, selector = units[item.unit]
        log_path, exit_path = os.path.join(scratch, f"{piece}.log"), os.path.join(scratch, f"{piece}.exit")
        if not os.path.exists(exit_path):
            continue
        collected += 1
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            sys.stdout.write(handle.read())
        with open(exit_path, encoding="utf-8") as handle:
            exit_text = handle.read().strip()
        if exit_text != "0":
            failed.append(f"-p {package} {' '.join(selector)}{f'（按测试函数分的一片，{len(item.names)} 条）' if item.names else ''}（退 {exit_text}）")
    sys.stdout.flush()
    doc_state = "no"
    doc_packages = [] if selected_targets or stop_reason else [package for package in selected_packages if doctests.get(package)]
    if doc_packages:
        doc_exit = subprocess.run(["cargo", "test"] + unit_arguments + package_options(doc_packages) + ["--doc"]
                                  + (["--"] + passed_arguments if passed_arguments else []), env=dict(environment, RUST_TEST_THREADS=str(jobs))).returncode
        doc_state = "yes" if doc_exit == 0 else "failed"
        if doc_exit != 0:
            failed.append(f"--doc（退 {doc_exit}）")
    for name in os.listdir(scratch):
        os.remove(os.path.join(scratch, name))
    os.rmdir(scratch)
    counts = sorted(set(thread_counts.values())) or [1]
    threads_text = str(counts[0]) if len(counts) == 1 else f"{counts[0]}-{counts[-1]}"
    memory_bound = len(rounds) > 1 or any(item.names is not None for item in dispatched) or stop_reason == "memory-cap-hit"
    pieces_text = ",".join(str(count) for count, _jobs in rounds) or "0"
    jobs_text = ",".join(str(round_jobs) for _count, round_jobs in rounds) or "0"
    if cap is None:
        print(f"  · 内存分片：没有内存上限（{cap_text}），不按内存分：一片 {len(dispatched)} 件、同时开 {jobs_text} 件，卡在线程（--jobs {jobs} 与件数）")
    elif not rounds:
        print("  · 内存分片：" + ("一片都没派（为什么见下面的红句），卡在内存" if stop_reason else "没有要跑的件"))
    elif memory_bound:
        print(f"  · 内存分片：分了 {len(rounds)} 片，每片 {pieces_text.replace(',', '、')} 件、同时开 {jobs_text.replace(',', '、')} 件，卡在内存"
              f"（上限 {format_bytes(cap)}，可用量装不下全部件的估值{'，有目标按测试函数分了片' if any(item.names is not None for item in dispatched) else ''}）")
    else:
        print(f"  · 内存分片：一片装得下全部 {len(dispatched)} 件（可用 {format_bytes(available)}），同时开 {jobs_text} 件，卡在线程（--jobs {jobs} 与件数）")
    print(f"{SUMMARY_PREFIX} dispatched={len(dispatched)} collected={collected} failed={len(failed)} jobs={max((round_jobs for _count, round_jobs in rounds), default=1)} "
          f"test_threads={threads_text} doc={doc_state} memory_shards={len(rounds)} shard_pieces={pieces_text} shard_jobs={jobs_text} "
          f"bound={'memory' if memory_bound else 'threads'} memory_cap={cap if cap is not None else 'none'} available={available if available is not None else 'none'}")
    if collected != len(dispatched) and not broken("drop-collection"):
        print(f"  ✗ 派出去 {len(dispatched)} 件，收回 {collected} 个退出码：有件的结局丢了，这一趟不算数")
        print("  → 怎么办：整趟重跑；还对不上就看临时目录里各件的 .exit 为什么没写（件被杀、磁盘写不进）")
        return 1
    if stop_reason == "memory-cap-hit":
        print(f"  ✗ 撞内存上限：{stop_detail}；按撞内存上限退出（退 {MEMORY_CAP_HIT_EXIT}，统一任务服务错误类型表里 memory-cap-hit 那一行），不硬跑；"
              f"已经跑完的 {collected} 件照上面回读，没跑的 {len(pending)} 件与文档测试这一趟没跑")
        print(f"  → 怎么办：服务照 memory-cap-hit 加额度重排；到了那台的总上限还装不下，就是点名的测试函数一条要的内存比整台给的还多：改那条测试（拆小它造的数据），"
              f"或峰值记录偏大时删掉 {peaks_directory or '峰值记录'} 下这个目标的那一份（文件头 ⑤「记在哪」）再跑")
        return MEMORY_CAP_HIT_EXIT
    if stop_reason == "listing-failed":
        print(f"  ✗ 要按测试函数分片的目标列不出测试函数：{stop_detail}；没跑的 {len(pending)} 件这一趟没跑")
        print("  → 怎么办：单跑那一条 cargo test … -- --list --format terse 看它为什么退（harness = false 的目标不认 --list）；列得出来再跑这一步")
        return 1
    if failed and not broken("ignore-red"):
        print(f"  ✗ 红的测试目标 {len(failed)} 个：{'；'.join(failed)}")
        print("  → 怎么办：只重跑红的那几条用例：python3 scripts/gate/rerun-failed-tests.py <这一趟的日志> -p <包>")
        return 101
    return 0


FAKE_CARGO = r'''#!/usr/bin/env bash
# test-targets-parallel.py 自证的假 cargo：metadata、--no-run 打录好的 JSON；跑一个目标时记起止时刻、进程号、RUST_TEST_THREADS 与挑中的测试函数、睡 3 秒；
# 测试函数照 SELFTEST_FUNCTIONS（一行「<目标> <全名> <ok|FAILED>」），没写这个目标的一条 <目标>_case（beta 的 FAILED）；--list 打挑中的「<全名>: test」
case "$1" in
  metadata) cat "$SELFTEST_METADATA"; exit 0 ;;
  test) ;;
  *) echo "假 cargo 不认：$*" >&2; exit 2 ;;
esac
shift
arguments=" $* "
if [[ "$arguments" == *" --no-run "* ]]; then
  echo "   Compiling demo v0.1.0 (/selftest/demo)" >&2
  cat "$SELFTEST_ARTIFACTS"
  exit 0
fi
name=lib
if [[ "$arguments" == *" --doc "* ]]; then name=doc; elif [[ "$arguments" =~ --(test|bin)\ ([a-z_]+) ]]; then name="${BASH_REMATCH[2]}"; fi
if [[ "$name" == doc ]]; then
  echo "doc" >> "$SELFTEST_CALLS"
  echo "args doc $*" >> "$SELFTEST_CALLS"
  echo "   Doc-tests demo"
  echo "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
  exit 0
fi
after=(); seen=0
for word in "$@"; do
  if (( seen )); then after+=("$word"); elif [[ "$word" == -- ]]; then seen=1; fi
done
functions=(); outcomes=()
if [[ -n "${SELFTEST_FUNCTIONS:-}" && -f "$SELFTEST_FUNCTIONS" ]]; then
  while read -r target function outcome; do
    if [[ "$target" == "$name" ]]; then functions+=("$function"); outcomes+=("$outcome"); fi
  done < "$SELFTEST_FUNCTIONS"
fi
if (( ${#functions[@]} == 0 )); then
  functions=("${name}_case")
  if [[ "$name" == beta ]]; then outcomes=(FAILED); else outcomes=(ok); fi
fi
skips=" "; listing=0; picked=" "; position=0
while (( position < ${#after[@]} )); do
  word="${after[$position]}"
  case "$word" in
    --skip) skips+="${after[$((position + 1))]} "; position=$((position + 2)); continue ;;
    --format) position=$((position + 2)); continue ;;
    --list) listing=1 ;;
    -*) ;;
    *) picked+="$word " ;;
  esac
  position=$((position + 1))
done
selected=(); selected_outcomes=()
for index in "${!functions[@]}"; do
  function="${functions[$index]}"
  [[ "$skips" == *" $function "* ]] && continue
  [[ "$picked" != " " && "$picked" != *" $function "* ]] && continue
  selected+=("$function"); selected_outcomes+=("${outcomes[$index]}")
done
if (( listing )); then
  echo "listed $name ${#selected[@]}" >> "$SELFTEST_CALLS"
  for function in ${selected[@]+"${selected[@]}"}; do echo "$function: test"; done
  exit 0
fi
echo "args $name $*" >> "$SELFTEST_CALLS"
echo "picked $$ $name ${selected[*]}" >> "$SELFTEST_CALLS"
echo "start $name $(date +%s.%N) ${RUST_TEST_THREADS:-unset} $$" >> "$SELFTEST_CALLS"
if [[ "${SELFTEST_ALLOCATE:-}" == "$name" ]]; then
  python3 -c 'import time; held = bytearray(b"\x01") * (64 << 20); time.sleep(3)'
else
  sleep 3
fi
echo "     Running $name (target/debug/deps/$name-0123456789abcdef)" >&2
echo "running ${#selected[@]} tests"
passed=0; failed=0; code=0
for index in "${!selected[@]}"; do
  echo "test ${selected[$index]} ... ${selected_outcomes[$index]}"
  if [[ "${selected_outcomes[$index]}" == FAILED ]]; then failed=$((failed + 1)); code=101; else passed=$((passed + 1)); fi
done
if (( failed )); then
  echo "test result: FAILED. $passed passed; $failed failed; 0 ignored; 0 measured; 0 filtered out"
else
  echo "test result: ok. $passed passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
fi
echo "end $name $(date +%s.%N) $$" >> "$SELFTEST_CALLS"
exit "$code"
'''

SELFTEST_GIBIBYTE = 1 << 30
SELFTEST_MEMORY_CAP_BYTES = 10 * SELFTEST_GIBIBYTE   # 内存那几格给的上限（文件头「自证」）


def selftest_run(works, jobs, history=None, test_arguments=(), memory_cap=None, peaks=None, functions=None, allocate=None):
    """拿假 cargo 跑一趟：交 (完成的进程, 假 cargo 记的调用行, 跑完的用时表或 None, 这一趟的临时工作目录)。history 给了就写成这一趟的用时表（{「<包> <挑法>」: 秒}）；
    test_arguments 给了就接在第二个 `--` 之后（测试二进制的参数）；memory_cap 是这一趟的 TASK_SERVER_MEMORY_CAP_BYTES（None 摘掉）；
    peaks 是预先写进峰值记录的 {「<包> <挑法>」: 字节}；functions 是 [(目标, 全名, 结局)]；allocate 是要另起 python3 占 64 MiB 的目标名。
    这一趟的临时工作目录（峰值记录在它的 peaks/ 下）记进 works，由调用方判完再删。"""
    package_id = "path+file:///selftest/demo#0.1.0"
    metadata = {"packages": [{"id": package_id, "name": "demo", "targets": [
        {"kind": ["lib"], "name": "demo", "doctest": True}, {"kind": ["test"], "name": "alpha", "doctest": False},
        {"kind": ["test"], "name": "beta", "doctest": False}, {"kind": ["bin"], "name": "tool", "doctest": False}]}],
        "workspace_members": [package_id]}

    def artifact(kinds, name, test, executable=True):
        return json.dumps({"reason": "compiler-artifact", "package_id": package_id, "target": {"kind": kinds, "name": name},
                           "profile": {"test": test}, "executable": f"/selftest/target/debug/deps/{name}-0123" if executable else None})
    artifacts = [artifact(["lib"], "demo", False, executable=False), artifact(["custom-build"], "build-script-build", False),
                 artifact(["lib"], "demo", True), artifact(["test"], "alpha", True), artifact(["test"], "beta", True), artifact(["bin"], "tool", True)]
    work = tempfile.mkdtemp(prefix="test-targets-parallel-selftest-")
    works.append(work)
    bin_directory = os.path.join(work, "bin")
    os.makedirs(bin_directory)
    fake = os.path.join(bin_directory, "cargo")
    with open(fake, "w", encoding="utf-8") as handle:
        handle.write(FAKE_CARGO)
    os.chmod(fake, 0o755)
    paths = {name: os.path.join(work, name) for name in ("metadata.json", "artifacts.jsonl", "calls.txt", "durations.tsv", "functions.txt", "cgroup-listing", "peaks")}
    with open(paths["metadata.json"], "w", encoding="utf-8") as handle:
        json.dump(metadata, handle)
    with open(paths["artifacts.jsonl"], "w", encoding="utf-8") as handle:
        handle.write("\n".join(artifacts) + "\n")
    if history is not None:
        write_durations(paths["durations.tsv"], history)
    with open(paths["functions.txt"], "w", encoding="utf-8") as handle:
        handle.writelines(f"{target} {function} {outcome}\n" for target, function, outcome in (functions or ()))
    # cgroup 的读法换成临时目录（文件头「自证」）：memory.max 写 max（不收窄），cgroup.procs 里是自证进程自己（底座就是它）
    scope_directory = os.path.join(work, "cgroup", "selftest.scope")
    os.makedirs(scope_directory)
    with open(os.path.join(scope_directory, "memory.max"), "w", encoding="ascii") as handle:
        handle.write("max\n")
    with open(os.path.join(scope_directory, "cgroup.procs"), "w", encoding="ascii") as handle:
        handle.write(f"{os.getpid()}\n")
    with open(paths["cgroup-listing"], "w", encoding="ascii") as handle:
        handle.write("0::/selftest.scope\n")
    for key, peak in (peaks or {}).items():
        memory_module().write_atomically(os.path.join(key_directory(paths["peaks"], key), f"{1:020d}-1-selftest"), f"{peak} 1 1\t{key}\n")
    environment = {name: value for name, value in os.environ.items() if name != "TASK_SERVER_MEMORY_CAP_BYTES"}
    environment.update({"PATH": bin_directory + os.pathsep + os.environ.get("PATH", ""), "SELFTEST_METADATA": paths["metadata.json"],
                        "SELFTEST_ARTIFACTS": paths["artifacts.jsonl"], "SELFTEST_CALLS": paths["calls.txt"], "SELFTEST_FUNCTIONS": paths["functions.txt"],
                        DURATIONS_VARIABLE: paths["durations.tsv"], PEAKS_DIRECTORY_VARIABLE: paths["peaks"],
                        "GATE_MUTATION_CGROUP_LISTING": paths["cgroup-listing"], "GATE_MUTATION_CGROUP_ROOT": os.path.join(work, "cgroup"),
                        "SINGLEFS_MULTI_HOST_CONFIG": os.path.join(work, "multi-host.env.absent")})
    if memory_cap is not None:
        environment["TASK_SERVER_MEMORY_CAP_BYTES"] = str(memory_cap)
    if allocate is not None:
        environment["SELFTEST_ALLOCATE"] = allocate
    completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--jobs", str(jobs), "--", "--offline", "--no-fail-fast", "-p", "demo"]
                               + (["--", *test_arguments] if test_arguments else []),
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment, cwd=work, timeout=300)
    calls = open(paths["calls.txt"], encoding="utf-8").read().splitlines() if os.path.exists(paths["calls.txt"]) else []
    after = read_durations(paths["durations.tsv"]) if history is not None else None
    return completed, calls, after, work


def selftest_invocations(calls):
    """假 cargo 记的调用行 → [(目标, 起点, 终点, [挑中的全名])]，按进程号配起止。"""
    picked = {line.split()[1]: line.split()[3:] for line in calls if line.startswith("picked ")}
    ends = {line.split()[3]: float(line.split()[2]) for line in calls if line.startswith("end ")}
    invocations = []
    for line in calls:
        if line.startswith("start "):
            words = line.split()
            invocations.append((words[1], float(words[2]), ends.get(words[4], float("inf")), picked.get(words[4], [])))
    return invocations


def selftest_heaviest_overlap(invocations, estimate_of):
    """任何一件起跑那一刻在跑的各件估值之和的最大（estimate_of(目标, 挑中的全名) → 字节）。"""
    return max((sum(estimate_of(name, picked) for name, start, end, picked in invocations if start <= moment < end) for _name, moment, _end, _picked in invocations), default=0)


def selftest_outcomes(stdout):
    return {found.group(1): found.group(2) for found in re.finditer(r"^test (\S+) \.\.\. (ok|FAILED)$", stdout, re.M)}


def selftest():
    works = []   # 每一趟的临时工作目录：峰值记录那一格跑完还要读，判完一并删
    try:
        return selftest_cells(works)
    finally:
        for work in works:
            shutil.rmtree(work, ignore_errors=True)


def selftest_cells(works):
    failures = []
    completed, calls, _, _ = selftest_run(works, 4)
    starts = {line.split()[1]: (float(line.split()[2]), line.split()[3]) for line in calls if line.startswith("start ")}
    ends = {line.split()[1]: float(line.split()[2]) for line in calls if line.startswith("end ")}
    if sorted(starts) != ["alpha", "beta", "lib", "tool"] or sorted(ends) != sorted(starts) \
            or max(moment for moment, _ in starts.values()) >= min(ends.values()) or {threads for _, threads in starts.values()} != {"1"}:
        failures.append(f"四个目标同时在跑：四条的起止区间应当两两重叠（最晚的起点早于最早的终点）、每条 RUST_TEST_THREADS=1，实际起点 {starts}、终点 {ends}")
    summary = [line for line in completed.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    result_lines = [line for line in completed.stdout.splitlines() if line.startswith("test result: ")]
    if not summary or "dispatched=4 collected=4" not in summary[-1] or len(result_lines) != 5:
        failures.append(f"收回的条数等于派出的：末行应当是 dispatched=4 collected=4、四个目标加文档测试共 5 行 test result，实际末行 {summary}、test result {len(result_lines)} 行")
    if completed.returncode != 101 or "test beta_case ... FAILED" not in completed.stdout or "-p demo --test beta" not in completed.stdout:
        failures.append(f"红的目标照报、退 101：实际退 {completed.returncode}，输出尾部 {completed.stdout[-400:]!r}")
    if "Compiling demo v0.1.0" not in completed.stderr or calls.count("doc") != 1:
        failures.append(f"编译与文档测试：Compiling 那一行应当转到标准错误、文档测试跑恰好一次，实际标准错误 {completed.stderr[-300:]!r}、文档测试 {calls.count('doc')} 次")
    # 用时表：tool 最长、beta 次之（文件头 ③「次序」「线程」）
    history = {"demo --bin tool": 30.0, "demo --test beta": 20.0, "demo --lib": 5.0, "demo --test alpha": 1.0}
    _, calls, after, _ = selftest_run(works, 2, history)
    starts = {line.split()[1]: float(line.split()[2]) for line in calls if line.startswith("start ")}
    ends = {line.split()[1]: float(line.split()[2]) for line in calls if line.startswith("end ")}
    first_end = min((ends[name] for name in ("tool", "beta") if name in ends), default=None)
    if sorted(starts) != ["alpha", "beta", "lib", "tool"] or first_end is None \
            or not all(starts[name] >= first_end for name in ("lib", "alpha")) or sorted(after or {}) != sorted(history) \
            or any(abs((after or {}).get(key, 0.0) - history[key]) < 0.5 for key in history):
        failures.append(f"长的先起：--jobs 2 时用时表里最长的 tool 与 beta 应当先起、lib 与 alpha 等前两条有一条跑完才起，跑完用时表写回这一趟四个目标的用时，"
                        f"实际起点 {starts}、终点 {ends}、跑完的用时表 {after}")
    _, calls, _, _ = selftest_run(works, 5, history)
    threads = {line.split()[1]: line.split()[3] for line in calls if line.startswith("start ")}
    if threads != {"tool": "2", "beta": "1", "lib": "1", "alpha": "1"}:
        failures.append(f"空出来的线程分给后起的、长的多分：--jobs 5、四个目标时最长的 tool 应当拿 2 条线程、另三条各 1 条（合起来 5 条），实际 {threads}")
    # 测试二进制的参数（文件头「测试二进制的参数」）
    passed = ["--nocapture", "--exact", "--skip", "alpha_case", "--skip", "tool_case"]
    _, calls, _, _ = selftest_run(works, 4, test_arguments=passed)

    def shown_calls(recorded):
        return "；".join(name + "：" + " ".join(words) for name, words in sorted(recorded.items()))
    received = {line.split()[1]: line.split()[2:] for line in calls if line.startswith("args ")}
    wanted_tail = ["--", *passed]
    if sorted(received) != ["alpha", "beta", "doc", "lib", "tool"] or any(words[-len(wanted_tail):] != wanted_tail for words in received.values()):
        failures.append("测试二进制的参数原样交给每个目标：四个目标与文档测试那一条收到的参数都应当以「" + " ".join(wanted_tail) + "」收尾，"
                        "实际 " + shown_calls(received))
    # 内存的五格（文件头 ⑤ 与「自证」）
    gibibyte, cap = SELFTEST_GIBIBYTE, SELFTEST_MEMORY_CAP_BYTES
    even = {"demo --lib": 3 * gibibyte, "demo --test alpha": 3 * gibibyte, "demo --test beta": 3 * gibibyte, "demo --bin tool": 3 * gibibyte}
    completed, calls, _, _ = selftest_run(works, 4, memory_cap=cap, peaks=even)
    invocations = selftest_invocations(calls)
    heaviest = selftest_heaviest_overlap(invocations, lambda name, picked: 3 * gibibyte)
    summary = [line for line in completed.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    if sorted(name for name, *_ in invocations) != ["alpha", "beta", "lib", "tool"] or heaviest > cap \
            or not summary or "memory_shards=2 shard_pieces=3,1 " not in summary[-1] or "bound=memory" not in summary[-1]:
        failures.append(f"可用量装不下 --jobs 份时同时在跑的峰值之和不超过可用量：四个目标各记 3 GiB、上限 10 GiB，任何一刻在跑的峰值之和应当不超过上限、"
                        f"末行 memory_shards=2 shard_pieces=3,1 bound=memory，实际在跑的最多合计 {heaviest / gibibyte:.1f} GiB、起了 {[name for name, *_ in invocations]}、末行 {summary}")
    lopsided = {"demo --lib": gibibyte, "demo --test alpha": gibibyte, "demo --test beta": 16 * gibibyte, "demo --bin tool": gibibyte}
    beta_functions = [("beta", "beta_case_one", "ok"), ("beta", "beta_case_two", "FAILED"), ("beta", "beta_case_three", "ok"), ("beta", "beta_case_four", "ok")]
    skipped = ["--exact", "--skip", "beta_case_three"]
    sharded, calls, _, _ = selftest_run(works, 4, test_arguments=skipped, memory_cap=cap, peaks=lopsided, functions=beta_functions)
    invocations = selftest_invocations(calls)
    beta_runs = [picked for name, _start, _end, picked in invocations if name == "beta"]
    beta_listed = next((int(line.split()[2]) for line in calls if line.startswith("listed beta ")), 0)
    heaviest = selftest_heaviest_overlap(invocations, lambda name, picked: 16 * gibibyte * len(picked) / max(1, beta_listed) if name == "beta" else gibibyte)
    if beta_listed < 1 or len(beta_runs) < 2 or any(len(picked) >= beta_listed for picked in beta_runs) or heaviest > cap:
        failures.append(f"单个目标超可用量时按测试函数分片、各片都在可用量以内：beta 记 16 GiB、上限 10 GiB，应当列出它的测试函数、起不止一件、每件挑中的少于列出的条数、"
                        f"任何一刻在跑的估值（beta 一件按 16 GiB × 条数 ÷ 列出的条数）之和不超过上限，实际列出 {beta_listed} 条、beta 各件挑中 {beta_runs}、"
                        f"在跑的最多合计 {heaviest / gibibyte:.1f} GiB")
    unsharded, _, _, _ = selftest_run(works, 4, test_arguments=skipped, peaks=lopsided, functions=beta_functions)
    if selftest_outcomes(sharded.stdout) != selftest_outcomes(unsharded.stdout) or len(beta_runs) < 2:
        failures.append(f"分片与不分片的逐条结局相同：两趟的「test <名> ... <结局>」应当逐条相同、且分了片的那一趟 beta 起了不止一件，"
                        f"实际分片 {sorted(selftest_outcomes(sharded.stdout).items())}、不分片 {sorted(selftest_outcomes(unsharded.stdout).items())}、beta 起了 {len(beta_runs)} 件")
    oversized = {"demo --lib": gibibyte, "demo --test alpha": gibibyte, "demo --test beta": gibibyte, "demo --bin tool": 40 * gibibyte}
    completed, calls, _, _ = selftest_run(works, 4, memory_cap=cap, peaks=oversized)
    started = [line for line in calls if line.startswith("start ")]
    if completed.returncode != MEMORY_CAP_HIT_EXIT or "撞内存上限" not in completed.stdout or "tool_case" not in completed.stdout \
            or "memory-cap-hit" not in completed.stdout or started:
        failures.append(f"一片只剩一条还超时按撞内存上限退出、不硬跑：tool 记 40 GiB、只有一条、上限 10 GiB，应当退 {MEMORY_CAP_HIT_EXIT}、点名 tool_case 与 memory-cap-hit、一件都不起，"
                        f"实际退 {completed.returncode}、起了 {len(started)} 件，输出尾部 {completed.stdout[-300:]!r}")
    _, _, _, work = selftest_run(works, 4, allocate="alpha")
    alpha_key = "demo --test alpha"
    alpha_peak, alpha_note = measured_target_peak(os.path.join(work, "peaks"), alpha_key)
    alpha_directory = key_directory(os.path.join(work, "peaks"), alpha_key)
    alpha_records = [open(os.path.join(alpha_directory, name), encoding="utf-8").read() for name in sorted(os.listdir(alpha_directory))] if os.path.isdir(alpha_directory) else []
    if alpha_peak is None or alpha_peak < 48 << 20 or not any(record.split(" ")[1:2] == ["1"] for record in alpha_records):
        failures.append(f"每个测试目标量到的峰值记进共享根：alpha 那一件另起一个 python3 占 64 MiB，跑完记录里「{alpha_key}」的峰值应当不小于 48 MiB、条数写 1，"
                        f"实际峰值 {alpha_peak}（{alpha_note or '读得出'}）、记录 {alpha_records}")
    if failures:
        for failure in failures:
            print(f"  ✗ test-targets-parallel 自检：{failure}")  # gate-lint:detail
        print(f"  → 看 run() 的派活、线程记账、用时表、内存分片、峰值记录、回读与条数核对；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ test-targets-parallel 自检通过（{len(SELFTEST_CELLS)} 格：{'、'.join(SELFTEST_CELLS)}）")
    return 0


def main():
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    if len(arguments) < 3 or arguments[0] != "--jobs" or not arguments[1].isdigit() or int(arguments[1]) < 1 or arguments[2] != "--":
        print("  ✗ 用法：test-targets-parallel.py --jobs <正整数> -- <cargo test 的参数…>", file=sys.stderr)
        print("  → 例：test-targets-parallel.py --jobs 31 -- --offline --no-fail-fast -p singlefs-core；线程数是登记值（每台默认重活线程数，由 task-server 套的 capped.sh 设），自检：--selftest", file=sys.stderr)
        sys.exit(2)
    sys.exit(run(int(arguments[1]), arguments[3:]))


BREAK_VARIABLES = (BREAK_VARIABLE,)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/test-targets-parallel.py --selftest", file=sys.stderr)
    return 2


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    main()
