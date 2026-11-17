#!/usr/bin/env python3
# admission: always 每次调都在这一刻的树上编、跑这一次给的包，上一次的结论不替这一次作保
# run-condition: command cargo
"""一趟 cargo test 拆成按测试目标并行起：合入后验证（scripts/gate/merge-and-verify.sh）测试那一步与门禁 crash-point-replay 按条复用那几趟用它。

用法：
  test-targets-parallel.py --jobs <N> -- <cargo test 的参数…>      例：--jobs 31 -- --offline --no-fail-fast -p singlefs-core -p singlefs-verification
  test-targets-parallel.py --jobs <N> --binaries <清单> -- <cargo test 的参数…>   照 --prebuild-into 写的清单直接起编好的测试二进制（下面 ⑥）
  test-targets-parallel.py --jobs <N> [--binaries <清单>] --drop-sources <文件> -- <cargo test 的参数…>   文件里写的目标根文件对应的测试目标这一趟不起（下面 ⑦）
  test-targets-parallel.py --prebuild-into <目录> -- <cargo test 的参数…>        只编、把测试二进制放进 <目录>、写清单，不跑（下面 ⑥）
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
       线程类变量：每一件（与 ④ 那一条文档测试）的环境里，scripts/hosts/capped.sh 的 variable_names 清单里除 CARGO_BUILD_JOBS 之外的每个变量（装置各自读的线程变量、
         SINGLEFS_THREAD_CAP 这类，capped.sh 对整条命令设成登记数 N），与 RUST_TEST_THREADS 一起设成这一件的份额（文档测试取 N）：同时在跑的各件，每个变量的值合起来也不超过 N。
         清单每一趟从那份脚本现读（TEST_TARGETS_PARALLEL_CAPPED_SCRIPT 设了照它，自证用），这里不另抄一份；CARGO_BUILD_JOBS 是编译作业数，不跟着份额走。
         清单读不到（文件不在、找不到 variable_names 块、块里有写不成变量名的行、除 CARGO_BUILD_JOBS 外一个变量都没有）就一件都不派、退 1：
         装置的线程变量没跟着分摊，各件合起来会超登记数，不照只设 RUST_TEST_THREADS 往下跑。
     用时表：cargo metadata 给的 target_directory 下的 singlefs-test-target-durations.tsv（一行「<秒><制表符><包> <挑法>」，跑完收回时把这一趟每个目标的用时
       （分了片的是各片之和）写回、别的行照留，先写临时文件再改名）；TEST_TARGETS_PARALLEL_DURATIONS 设了照它（自证用）；
       metadata 里没有 target_directory 又没设的，不读不写、照编译输出的次序起。
     每件的输出（标准输出与标准错误合成一份）写自己的文件、退出码写另一个文件，
     全部跑完按派活的次序逐个回读、转打到标准输出（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）。
     收回来的退出码文件数与派出去的件数对不上，整趟判红、退 1。
     进度（TargetsProgress）：回读在全部跑完之后，跑的时候管道里没有行，由一个线程每 PROGRESS_SECONDS 秒看一次（TEST_TARGETS_PARALLEL_PROGRESS_SECONDS 设了照它，自证用；不是正数退 2）：
       这段时间有件跑完、或在跑的件的输出文件变长，才往标准输出打一行
       TEST_TARGETS_PARALLEL_PROGRESS seconds=<开跑多久> targets=<目标个数> finished=<跑完的件数> running=<在跑的件数> log_bytes=<在跑的件的输出合计字节> running_targets=<在跑的「包 挑法」，「；」接，没有写「无」>；
       都没有就不打：卡住的一趟日志不涨，统一任务服务照样判得出（task-server/task_server_lib.py 的 stalled）。名字带 _PROGRESS：门禁 crash-point-replay 每一趟 cargo 的管道照这个后缀转出来。
  ④ 文档测试：没有显式目标选择（--lib、--test 等）时，要测的包里库目标 doctest 为真的（cargo metadata），最后一条 cargo test <参数> -p <这些包> --doc 一起跑（③ 都跑完了，RUST_TEST_THREADS 取 N）。
  ⑤ 内存分片（脚本自己判，超过这一台能给的内存就分片跑，每一片都在它以内；同时开的件数按 N 开满，只在装不下时才少开）：
     量法、上限的取法、记录名与改名换上的写法、多机配置的找法都用 scripts/mutation/crates-mutation-rows.py 的（memory_module 读进来），不另写一套：
     上限：这一台能给的内存，它的 host_memory_capacity（多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项）。不取统一任务服务交的 TASK_SERVER_MEMORY_CAP_BYTES，
       也不拿自己 cgroup 那一层的 memory.max 收窄：那两样是这一次尝试拿到的额度，不是这一台能给多少，拿它们除会把并行压到几件、峰值记小、下一次给的额度还是小。
       这一台那一项取不到（多机配置不在、没写、认不出这一台）时取它的 task_memory_cap_bytes（服务交的变量；没有读多机配置那一项；memory.max 是有限值时两者取小）。
       两样都取不到的不按内存分，一片开满（同时至多 N 件），说明照打。
     底座（base_usage_of）：每一片派活之前现量自己 cgroup 那一层（连同下面的子 cgroup，cgroup.procs 逐个读）里每个进程的用量加总，
       单个进程的口径是它的 process_usage_bytes（/proc/<pid>/status 的 RssAnon + RssShmem + VmPTE）。它的底座读数只量起那一道的父进程自己；这里量整个 cgroup，
       因为起这一份的门禁链（合入后验证、门禁阶段的 bash 与 python）占着的也是同一份内存。cgroup 读不到只量本进程，说明照打。可用量 = 上限 − 底座。
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
  ⑥ 编一处、跑多处（门禁 crash-point-replay 分片的计划那一步编好，各片照清单直接起，各片不再编）：
     --prebuild-into <目录>：照 ① 编、照 ② 列目标，每个目标的测试二进制硬链接进 <目录>/deps/（不在同一个文件系统拷一份：place_binary 用 objcopy --strip-debug 直接写出
       去掉调试段的一份，原件不动，调试段占二进制的九成以上而核判器输入读的键表在 .rodata；objcopy 不在 PATH 上、退非 0、写出空文件时整份拷），二进制所在目录上一层 build/*/out 里的
       PREBUILT_TABLE_NAMES 两张表（门禁核判器输入变没变读它们）照同样的相对位置拷进 <目录>/build/（同一份表只拷一份，不为每个目标各拷一遍）；先放在 <目录>.partial，放完换上。
       清单 <目录>/targets.tsv：头一行「# test-targets-parallel prebuilt<TAB><编的那一次的 cargo 参数，JSON 数组>」，之后一个目标一行
       「<包><TAB><挑法><TAB><放进去的二进制，没放进去写 -><TAB><cargo 那一行 Running 里的源文件><TAB><包目录><TAB><crate 名>」；末行
       TEST_TARGETS_PARALLEL_PREBUILT targets=<目标数> placed=<放进去的> missing=<没放进去的> manifest=<清单>。编不过照 cargo 的退出码退，别的照下面「退出码」。
     --binaries <清单>：清单的 cargo 参数与这一次的（第二个 `--` 之前、连同 -p 那几对）逐个相同才用，不同、读不了、写法不对打一行原因、照 ① 编再跑。
       用得上时不编：目标照清单（编的那一次 cargo 列的就是这几个），二进制在且可执行的直接起，写 - 或文件不在的那几个照 ③ 经 cargo 起（cargo 自己编它）。
       直接起的一件：在包目录里起「<二进制> <测试二进制的参数>」，环境补上 cargo 给测试进程设的 CARGO_MANIFEST_DIR、CARGO_PKG_NAME、CARGO_CRATE_NAME、CARGO_PRIMARY_PACKAGE、CARGO；
       输出照旧一件一份文件（各收各的），开头先写 cargo 那一行「     Running <源文件> (<二进制>)」（库与 bin 的源文件前面带 unittests，同 cargo），
       退非 0 时照 cargo 的写法补「error: test failed, to rerun pass `<挑法>`」与「process didn't exit successfully: `<命令>` (exit status: <码>)」
       （死于信号写「(signal: <号>, <名>: <说明>)」），这一件退 CARGO_TEST_FAILURE_EXIT：门禁按 Running 行切段、认被停的那几行与经 cargo 时相同。
       按测试函数分片要列测试函数的（⑤），直接起的照样起二进制带 --list --format terse。
  ⑦ 不起别的片领的目标（门禁 crash-point-replay 分片的一片只起归它的测试目标）：
     --drop-sources <文件>：一行一个目标根文件（相对包目录，库与 bin 不带 unittests，例 src/lib.rs、tests/alpha.rs、src/bin/tool.rs），目标的源文件（清单第四列、没清单时是编译产物的 src_path
       相对包目录，同 ⑥ 的 Running 行去掉 unittests）在里面的这一趟不起：它们的测试函数在这一片的计划里全记 registered、本来就全被 --skip 排掉，起了只是白起一个进程、读一遍二进制。
       源文件对不上的目标照起；清单点名了全部目标时一个都不去、打一行；清单读不了退 1、一件都不派。
  测试二进制的参数：<cargo test 的参数…> 里第二个 `--` 之后的（--nocapture、--exact、--skip <名> 这类）原样接在 ③ 每一件与 ④ 那一条的 `--` 之后，
     一个字不改、不挑（门禁 crash-point-replay 的按条复用拿 --exact --skip 跳过键相同的测试函数）；按测试函数分的片照 ⑤ 另接全名。
  末行 TEST_TARGETS_PARALLEL dispatched=<派出的件数> collected=<收回> failed=<红的件数> jobs=<同时至多几件> test_threads=<每件的线程数，各件不同时写 最少-最多> doc=<yes|no|failed>
     memory_shards=<分了几片> shard_pieces=<每片几件，逗号分> shard_jobs=<每片同时开几件，逗号分> bound=<memory|threads> memory_cap=<字节|none> available=<最后一片的可用量字节|none>；
     末行之前一行「· 内存分片：」写分了几片、每片同时开几件、卡在内存还是线程。
退出码：都绿 0；有件或文档测试红 101；编不过照 cargo 的；撞内存上限 250；收回与派出对不上、cargo metadata 读不出、列不出要分片的目标的测试函数、
  读不进 crates-mutation-rows.py、读不出 capped.sh 的线程变量清单、--prebuild-into 认不出产物属于哪个包或放不进目录 1；用法错（含进度间隔不是正数）2。
线程数只照 --jobs（调用方经 capped.sh 给登记值），这里不现算、不写常量；内存只决定一片里挑哪几件，不动线程数。
弄坏开关（只给自证证红用：--selftest 本身与它起的子进程认，平常跑设了退 2、点名只在自证里认），一项一行，下一行写打开后判错的自证格：
  TEST_TARGETS_PARALLEL_BREAK=serial  同时只起一件（一个测试二进制接一个地跑）
      「四个目标同时在跑」「空出来的线程分给后起的、长的多分」「服务交的上限与 memory.max 都很小而这一台能给的内存充裕时分片与同时开的件数不变」（同时只起一件，每件都拿满 N 条线程）
  TEST_TARGETS_PARALLEL_BREAK=drop-collection  回读时丢掉最后派出的那一件、也不核收回的件数
      「收回的条数等于派出的」「分片与不分片的逐条结局相同」「直接起编好的测试二进制与经 cargo 起的逐条结局相同」（分了片的那一趟丢的是 beta 的一片、不分片的那一趟丢的是 tool，照清单直接起二进制的那一趟同样丢 tool）
  TEST_TARGETS_PARALLEL_BREAK=ignore-red  件退非 0 也按绿算、退 0
      「红的目标照报、退 101」
  TEST_TARGETS_PARALLEL_BREAK=even-threads  每件的 RUST_TEST_THREADS 一律取 ⌈N ÷ min(N, 片里的件数)⌉（空出来的线程不分给后起的）
      「空出来的线程分给后起的、长的多分」「线程类变量按份额设给每一件」（每件都拿 2 条，同时在跑的各件合起来超过 N）
  TEST_TARGETS_PARALLEL_BREAK=history-ignored  不看用时表，照编译输出的次序起
      「长的先起」「空出来的线程分给后起的、长的多分」（最长的 tool 不先起，多出来的那条线程落到先起的 lib 上）
  TEST_TARGETS_PARALLEL_BREAK=test-arguments-dropped  ③ 每一件与 ④ 那一条都不接测试二进制的参数（--exact --skip 那几对丢了，键相同的测试函数照跑）
      「测试二进制的参数原样交给每个目标」
  TEST_TARGETS_PARALLEL_BREAK=memory-ignored  不看内存上限：一片开满（同时至多 N 件），也不按测试函数分片（会撞上限）
      「可用量装不下 --jobs 份时同时在跑的峰值之和不超过可用量」「单个目标超可用量时按测试函数分片、各片都在可用量以内」「分片与不分片的逐条结局相同」「一片只剩一条还超时按撞内存上限退出、不硬跑」「这一台那一项没写时退回服务交的上限」
  TEST_TARGETS_PARALLEL_BREAK=memory-by-task-cap  上限只取服务交的 TASK_SERVER_MEMORY_CAP_BYTES 与自己 cgroup 的 memory.max，不取这一台能给的内存
      「服务交的上限与 memory.max 都很小而这一台能给的内存充裕时分片与同时开的件数不变」
  TEST_TARGETS_PARALLEL_BREAK=memory-no-task-fallback  这一台那一项取不到时不退回服务交的上限，不按内存分
      「这一台那一项没写时退回服务交的上限」
  TEST_TARGETS_PARALLEL_BREAK=function-shards-off  单个目标的估值超过可用量也不按测试函数分片，整个起、单独占一片
      「单个目标超可用量时按测试函数分片、各片都在可用量以内」「分片与不分片的逐条结局相同」「一片只剩一条还超时按撞内存上限退出、不硬跑」
  TEST_TARGETS_PARALLEL_BREAK=function-shard-last-dropped  按测试函数分片时丢掉列出来的最后一条（那一条哪一片都不在）
      「分片与不分片的逐条结局相同」
  TEST_TARGETS_PARALLEL_BREAK=memory-cap-short-run-anyway  一条测试函数的估值就超过可用量时照整个单独起一片硬跑，不按撞内存上限退 250
      「一片只剩一条还超时按撞内存上限退出、不硬跑」
  TEST_TARGETS_PARALLEL_BREAK=peaks-unrecorded  量到的峰值不写进记录（只在这一趟里用）
      「每个测试目标量到的峰值记进共享根」
  TEST_TARGETS_PARALLEL_BREAK=progress-silent  不起报进度的线程（跑的时候管道里一行没有）
      「每过一段时间报一行：跑完几件、在跑哪几个」「输出文件变长也报一行」
  TEST_TARGETS_PARALLEL_BREAK=progress-line-renamed  进度行的名字不带 _PROGRESS（门禁 crash-point-replay 的管道不转它）
      「每过一段时间报一行：跑完几件、在跑哪几个」「输出文件变长也报一行」
  TEST_TARGETS_PARALLEL_BREAK=progress-every-interval  每过一个间隔都报，不看这段时间有没有变化（卡住的一趟日志照涨，服务判不出卡住）
      「没有件跑完、输出文件没变长的那段时间不报」
  TEST_TARGETS_PARALLEL_BREAK=progress-ignores-log-growth  只看有没有件跑完，不看输出文件变长（一个目标跑得久、一直有输出，这段时间不报）
      「输出文件变长也报一行」
  TEST_TARGETS_PARALLEL_BREAK=binaries-through-cargo  给了 --binaries 也不看清单：照 ① 编、每个目标经 cargo 起
      「直接起编好的测试二进制与经 cargo 起的逐条结局相同」「清单里没放二进制的目标仍经 cargo 起」
  TEST_TARGETS_PARALLEL_BREAK=debug-info-kept-on-copy  --prebuild-into 链不成而拷的二进制不去调试段，整份拷
      「拷的二进制去掉调试段、判器核对的表只拷一份」
  TEST_TARGETS_PARALLEL_BREAK=tables-copied-per-target  --prebuild-into 判器核对要读的表为每个目标各拷一遍
      「拷的二进制去掉调试段、判器核对的表只拷一份」
  TEST_TARGETS_PARALLEL_BREAK=dropped-targets-started  --drop-sources 点名的目标照旧起
      「不起 --drop-sources 点名的目标」
  TEST_TARGETS_PARALLEL_BREAK=missing-binary-dropped  清单里没放二进制（写 -、文件不在）的目标不派
      「清单里没放二进制的目标仍经 cargo 起」
  TEST_TARGETS_PARALLEL_BREAK=cargo-failure-lines-unwritten  直接起的测试二进制退非 0 时不补 cargo 的那几行、这一件照它自己的退出码退
      「直接起的测试二进制退非 0 时照 cargo 的写法报、这一件退 101」
  TEST_TARGETS_PARALLEL_BREAK=thread-variables-unscoped  只把 RUST_TEST_THREADS 设成这一件的份额，清单里的别的线程变量照环境里的登记数交给每一件
      「线程类变量按份额设给每一件」（每一件的装置线程变量应等于它的 RUST_TEST_THREADS、同时在场的各件合起来不超登记数）
  TEST_TARGETS_PARALLEL_BREAK=capped-list-unchecked  清单读不到也照起，只设 RUST_TEST_THREADS
      「线程类变量按份额设给每一件」（清单读不到应当一件都不派、退 1）
自证（--selftest）：PATH 最前面放一份假 cargo——metadata 打一个包 demo（库带文档测试），--no-run 打四个测试目标的产物（lib、test alpha、test beta、bin tool，
  另有一份不是测试的库产物与一份构建脚本产物，不算目标），跑某个目标时记下起止时刻、进程号与 RUST_TEST_THREADS、挑中的测试函数、睡 3 秒，
  每个目标默认一条测试函数 <目标>_case，beta 那一条打 FAILED、退 101；--list 照参数里的 --skip 与位置全名打「<全名>: test」。
  每一趟都换掉 cgroup 的读法（GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 指到临时目录：memory.max 写 max，cgroup.procs 里是自证进程自己）、
  多机配置指到不在的文件（给了这一台能给的内存的那几趟，指到临时目录里写好的一份：HOSTS 写 127.0.0.1 与一个文档用的假地址，SLICE_MEMORY_CAP_BY_HOST 里这一台那一项写那一趟的数、假地址那一项写小得多的数）、
  峰值记录放临时目录，TASK_SERVER_MEMORY_CAP_BYTES 只在点名它的那几趟给（不给就摘掉）。--jobs 4、不给上限：
  「四个目标同时在跑」四条的起止区间两两重叠（最晚的起点早于最早的终点）、每条的 RUST_TEST_THREADS 是 1；
  「收回的条数等于派出的」末行 dispatched=4 collected=4，四个目标的 test result 行都在标准输出里；
  「红的目标照报、退 101」退 101，FAILED 那一行与「-p demo --test beta」都在输出里；
  「编译与文档测试」Compiling 那一行转到了标准错误，文档测试跑了恰好一次。
  另两趟各带一份用时表（tool 30 秒、beta 20 秒、lib 5 秒、alpha 1 秒）：
  「长的先起」--jobs 2：tool 与 beta 先起，lib 与 alpha 的起点都不早于前两条里先跑完的那一条的终点；跑完用时表里四个目标都写回了这一趟的用时；
  「空出来的线程分给后起的、长的多分」--jobs 5：最长的 tool 拿 2 条线程、另三条各 1 条，合起来正好 5 条。
  「线程类变量按份额设给每一件」假 capped.sh 的 variable_names 里放 CARGO_BUILD_JOBS、RUST_TEST_THREADS 与两个只在这里有的装置线程变量（环境里都先设成 5，同 capped.sh 5），
    它后面另有一块 device_only_variable_names（里面的名字不该被设）；--jobs 5、带用时表：假 cargo 记下每一件与文档测试收到的这几个变量，
    两个装置线程变量等于那一件的 RUST_TEST_THREADS（tool 2、另三条 1、文档测试 5）、每个变量在同时在跑的各件上合起来不超过 5、CARGO_BUILD_JOBS 仍是 5、device_only 那个没设；
    另一趟把清单位置指到不在的文件：退 1、输出点名 capped.sh、一件都没起、没编。
  再一趟带测试二进制的参数 `-- --nocapture --exact --skip alpha_case --skip tool_case`（--jobs 4；假 cargo 跑某个目标时另记一行收到的参数）：
  「测试二进制的参数原样交给每个目标」四个目标与文档测试那一条收到的参数都以 `-- --nocapture --exact --skip alpha_case --skip tool_case` 收尾。
  内存的七格（峰值记录预先写进临时目录，--jobs 4；前四格这一台能给的内存 10 GiB，多机配置里假地址那一项写 1 GiB，读错了那一项这几格就红）：
  「可用量装不下 --jobs 份时同时在跑的峰值之和不超过可用量」四个目标各记 3 GiB：任何一件起跑那一刻在跑的各件记的峰值之和不超过 10 GiB，四个都跑了，
    末行 memory_shards=2 shard_pieces=3,1 bound=memory；
  「单个目标超可用量时按测试函数分片、各片都在可用量以内」lib、alpha、tool 各记 1 GiB，beta 记 16 GiB、有四条测试函数（beta_case_two 判 FAILED），
    测试二进制的参数 `-- --exact --skip beta_case_three`：beta 列出三条、每片一条（⌊约 9.9 GiB × 3 ÷ 16 GiB⌋ = 1），beta 起了不止一件、每件挑中的少于列出的条数，
    任何一件起跑那一刻在跑的估值（beta 一件按 16 GiB × 条数 ÷ 列出的条数）之和不超过 10 GiB；
  「分片与不分片的逐条结局相同」同一份记录与参数、不给上限再跑一趟，两趟标准输出里的「test <名> ... <结局>」逐条相同，且分了片的那一趟 beta 起了不止一件；
  「一片只剩一条还超时按撞内存上限退出、不硬跑」tool 记 40 GiB、只有一条测试函数：退 250，输出里有「撞内存上限」、tool_case 与 memory-cap-hit，假 cargo 一件都没起；
  「服务交的上限与 memory.max 都很小而这一台能给的内存充裕时分片与同时开的件数不变」这一台能给 50 GiB、服务交 4 GiB、假 cgroup 的 memory.max 写 3 GiB，四个目标各记 3 GiB：
    末行 memory_shards=1 shard_pieces=4 shard_jobs=4 bound=threads（一片装得下四件、四件同时开；照服务交的 4 GiB 或 memory.max 的 3 GiB 除会分片、会撞内存上限）；
  「这一台那一项没写时退回服务交的上限」多机配置不在、服务交 10 GiB，四个目标各记 3 GiB：末行 memory_shards=2 shard_pieces=3,1 bound=memory；
  「每个测试目标量到的峰值记进共享根」不给上限、没有记录，alpha 那一件另起一个 python3 占 64 MiB：跑完记录里「demo --test alpha」的峰值不小于 48 MiB、条数写 1。
  进度的三格（进度间隔 1 秒）：--jobs 1 一趟（目标一个接一个跑，每个睡 3 秒、睡完才有输出）：
  「每过一段时间报一行：跑完几件、在跑哪几个」至少两行 finished 不小于 1，每行的 running_targets 正是照派活次序排第 finished 个的目标；
  「没有件跑完、输出文件没变长的那段时间不报」这一趟至少跑 12 秒、至多 8 行（四件跑完、四件打结果行各至多一次）；
  「输出文件变长也报一行」--jobs 4、最先起的 lib 每 0.5 秒打一行：有一行 finished=0、running_targets 里有 demo --lib。
  编好的测试二进制的三格（⑥）：假 cargo 的 --no-run 打的产物指到临时目录里的四个假测试二进制（各把活转给假 cargo 跑同一个目标，带 SELFTEST_THROUGH_BINARY：
  假 cargo 记「binary <目标> <工作目录名>」不记「args」、不打它自己的 Running 行），每次 --no-run 往调用记录写一行 build；
  「直接起编好的测试二进制与经 cargo 起的逐条结局相同」--prebuild-into 退 0、清单四个目标都放进去了、构建输出里的表拷过去了；再 --binaries 那一趟（--jobs 4）
    一回都没编（没有 build 行）、四个目标都是二进制起的（binary 行，工作目录是包目录 demo、没有 args 行），四段 Running 行的源文件是 unittests src/lib.rs、tests/alpha.rs、
    tests/beta.rs、unittests src/bin/tool.rs，「test <名> ... <结局>」与经假 cargo 的第一趟逐条相同、同退 101；
  「清单里没放二进制的目标仍经 cargo 起」清单里 alpha 那一行写 -、beta 放进去的二进制删掉：alpha、beta 经假 cargo 起（args 行），lib、tool 是二进制起的，逐条结局仍与第一趟相同；
  「直接起的测试二进制退非 0 时照 cargo 的写法报、这一件退 101」cargo_style_failure 退 0 不补；退 101 补「error: test failed, to rerun pass `--test beta`」、这一件退 101；
    死于 SIGTERM、SIGKILL 补的那一行认得出门禁认被停的写法（「process didn't exit successfully: … (signal: 15, SIGTERM」），死于 SIGSEGV 写 signal: 11, SIGSEGV。
  「拷的二进制去掉调试段、判器核对的表只拷一份」PATH 最前面另有一份假 objcopy（目标 = 源再加一行「# 去掉了调试段」），TEST_TARGETS_PARALLEL_LINK_FAILS 设上让硬链接一律失败：
    --prebuild-into 退 0、四个目标都放进去、四份都以「# 去掉了调试段」收尾，输出写「硬链接 0、拷 4，其中去掉调试段 4」与「判器核对的表拷 2 份」（四个目标共用一个构建输出目录里的两张表）。
  「不起 --drop-sources 点名的目标」--drop-sources 的清单点名 tests/alpha.rs 与 src/bin/tool.rs：经 cargo 起的一趟与 --prebuild-into 之后照清单直接起的一趟都只起 lib、beta（末行 dispatched=2 collected=2、
    输出写「别的片领的测试目标这一趟不起：2 个」）；清单点名全部四个目标时一个都不去、打「会去掉全部 4 个目标：不去」；清单不在退 1、一件都不派。
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
                  "分片与不分片的逐条结局相同", "一片只剩一条还超时按撞内存上限退出、不硬跑",
                  "服务交的上限与 memory.max 都很小而这一台能给的内存充裕时分片与同时开的件数不变", "这一台那一项没写时退回服务交的上限", "每个测试目标量到的峰值记进共享根",
                  "每过一段时间报一行：跑完几件、在跑哪几个", "没有件跑完、输出文件没变长的那段时间不报", "输出文件变长也报一行",
                  "直接起编好的测试二进制与经 cargo 起的逐条结局相同", "清单里没放二进制的目标仍经 cargo 起",
                  "直接起的测试二进制退非 0 时照 cargo 的写法报、这一件退 101", "拷的二进制去掉调试段、判器核对的表只拷一份", "不起 --drop-sources 点名的目标", "线程类变量按份额设给每一件")   # 自证的格名，判错行照抄
DURATIONS_VARIABLE = "TEST_TARGETS_PARALLEL_DURATIONS"   # 用时表换位置（自证用）；不设取 target_directory 下的 DURATIONS_FILE_NAME
DURATIONS_FILE_NAME = "singlefs-test-target-durations.tsv"
REPOSITORY = os.path.realpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", ".."))
MEMORY_MODULE_PATH = os.path.join(REPOSITORY, "scripts", "mutation", "crates-mutation-rows.py")   # 量法、上限的取法与记录的写法都用它的（文件头 ⑤）
PEAKS_DIRECTORY_VARIABLE = "TEST_TARGETS_PARALLEL_PEAKS_DIRECTORY"   # 峰值记录换位置（自证用）；不设取共享根下的 PEAKS_DIRECTORY_NAME
PEAKS_DIRECTORY_NAME = "test-targets-parallel-peaks"
CAPPED_SCRIPT_PATH = os.path.join(REPOSITORY, "scripts", "hosts", "capped.sh")   # 线程类变量清单每一趟现读它的 variable_names（文件头 ③「线程类变量」）
CAPPED_SCRIPT_VARIABLE = "TEST_TARGETS_PARALLEL_CAPPED_SCRIPT"   # 清单换位置（自证用）；不设取 CAPPED_SCRIPT_PATH
CAPPED_LIST_FORM = re.compile(r"^variable_names=\(\n(.*?)^\)", re.M | re.S)
VARIABLE_NAME_FORM = re.compile(r"[A-Z][A-Z0-9_]*")
CARGO_JOBS_VARIABLE = "CARGO_BUILD_JOBS"   # 编译作业数，不跟着每一件的份额走
PIECE_VARIABLE = "TEST_TARGETS_PARALLEL_PIECE"   # 设在每一件的环境里：采样线程照它认这一份的子进程是哪一件
PROGRESS_SECONDS = 300                # 报进度的线程隔多久看一次（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔）
PROGRESS_SECONDS_VARIABLE = "TEST_TARGETS_PARALLEL_PROGRESS_SECONDS"   # 进度间隔换一个（自证用）
PROGRESS_LINE_NAME = "TEST_TARGETS_PARALLEL_PROGRESS"
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
PREBUILT_MANIFEST_NAME = "targets.tsv"                       # --prebuild-into 写的清单（文件头 ⑥）
PREBUILT_MANIFEST_HEADER = "# test-targets-parallel prebuilt"   # 清单头一行的开头，后接制表符与编的那一次的 cargo 参数（JSON 数组）
PREBUILT_TABLE_NAMES = ("judging_units_keys.tsv", "watched_file_digests.tsv")   # 二进制上一层 build/*/out 里一起放进去的表（门禁核判器输入读它们）
LINK_FAILS_VARIABLE = "TEST_TARGETS_PARALLEL_LINK_FAILS"   # 自证里设成非空：--prebuild-into 的硬链接一律当失败（只在自证底下认）
CARGO_TEST_FAILURE_EXIT = 101         # cargo test 有测试二进制退非 0 时自己退的码：直接起的那一件照它退（文件头 ⑥）
CARGO_SIGNAL_DESCRIPTIONS = {1: "SIGHUP: hangup", 2: "SIGINT: interrupt", 3: "SIGQUIT: quit", 4: "SIGILL: illegal instruction",
                             6: "SIGABRT: process abort signal", 7: "SIGBUS: access to undefined memory", 8: "SIGFPE: erroneous arithmetic operation",
                             9: "SIGKILL: kill", 11: "SIGSEGV: invalid memory reference", 13: "SIGPIPE: write on a pipe with no one to read it",
                             15: "SIGTERM: termination signal"}   # cargo 报测试二进制死于信号时写的「<名>: <说明>」（门禁认被停读 2、9、15 这三种）


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


def capped_thread_variables(path):
    """读 capped.sh 的 variable_names 清单（文件头 ③「线程类变量」）：交 (清单里除 CARGO_BUILD_JOBS 之外的变量名, None)；读不到交 (None, 原因)。"""
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except OSError as error:
        return None, f"读不了 {path}（{error}）"
    block = CAPPED_LIST_FORM.search(text)
    if block is None:
        return None, f"{path} 里找不到 variable_names=( 那一块"
    names = []
    for line in block.group(1).splitlines():
        word = line.split("#", 1)[0].strip()
        if not word:
            continue
        if not VARIABLE_NAME_FORM.fullmatch(word):
            return None, f"{path} 的 variable_names 里「{word}」不是一个变量名（要一行一个）"
        if word != CARGO_JOBS_VARIABLE and word not in names:
            names.append(word)
    if not names:
        return None, f"{path} 的 variable_names 里除 {CARGO_JOBS_VARIABLE} 之外一个变量都没有"
    return names, None


def thread_environment(variables, threads):
    """一件（或文档测试）的线程类变量：清单里的变量与 RUST_TEST_THREADS 都取这一件的份额（文件头 ③「线程类变量」）；弄坏开关 thread-variables-unscoped 只设 RUST_TEST_THREADS。"""
    share = str(threads)
    return {**{name: share for name in ([] if broken("thread-variables-unscoped") else variables)}, "RUST_TEST_THREADS": share}


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


def test_units_of(artifacts, names):
    """编译产物里的测试目标（文件头 ②）：交 ([(包, 挑法, 产物)…], None)；有一个认不出属于哪个包、怎么挑，交 (None, 那一个产物)。"""
    units = []
    for artifact in artifacts:
        if not artifact.get("executable") or not (artifact.get("profile") or {}).get("test"):
            continue
        selector = selector_of(artifact.get("target") or {})
        package = names.get(artifact.get("package_id"))
        if selector is None or package is None:
            return None, artifact
        units.append((package, selector, artifact))
    return units, None


def running_source_of(artifact):
    """cargo test 那一行「Running <源文件> (<二进制>)」里的源文件（文件头 ⑥）：产物的 src_path 相对它的包目录，库与 bin 前面带 unittests。"""
    target = artifact.get("target") or {}
    source = target.get("src_path") or target.get("name") or ""
    package_directory = os.path.dirname(artifact.get("manifest_path") or "")
    if source and package_directory and os.path.isabs(source):
        relative = os.path.relpath(source, package_directory)
        if not relative.startswith(os.pardir):
            source = relative
    kinds = set(target.get("kind") or [])
    return f"unittests {source}" if kinds & LIBRARY_KINDS or "bin" in kinds else source


class PrebuiltBinary:
    """清单里放了二进制的一个目标（文件头 ⑥）：直接起它要的四样。"""

    def __init__(self, binary, source, package_directory, crate_name):
        self.binary, self.source, self.package_directory, self.crate_name = binary, source, package_directory, crate_name


def place_binary(executable, placed):
    """把一个测试二进制放进 placed（文件头 ⑥）：同一个文件系统硬链接；链不成就拷，拷的时候去掉调试段（objcopy --strip-debug：直接从原件写出去掉调试段的一份，不改原件；
    它读的键表在 .rodata、不在调试段里）；objcopy 不在 PATH 上、退非 0、写出空文件时整份拷。交 "linked"、"stripped"、"copied"；整份拷也不成抛 OSError。
    弄坏开关 debug-info-kept-on-copy：拷的时候不去调试段。自证里 TEST_TARGETS_PARALLEL_LINK_FAILS 非空时硬链接一律当失败（同一个文件系统里造出跨文件系统的拷）。"""
    try:
        if os.environ.get(LINK_FAILS_VARIABLE) and inside_selftest():
            raise OSError("自证里造的硬链接失败")
        os.link(executable, placed)
        return "linked"
    except OSError:
        pass
    # strip_tool 在本函数里只赋这一次 shutil.which 的结果：算单元键的库（lib_function_units.py 文件头「python 的命令」，path_program）只认这一种写法的
    # 命令词是根外的程序；开关不并进赋值（x if c else shutil.which(…) 它折不出、拒算 subprocess-command-unfolded），另在条件里判
    strip_tool = shutil.which("objcopy")
    if strip_tool and not broken("debug-info-kept-on-copy"):
        try:
            completed = subprocess.run([strip_tool, "--strip-debug", executable, placed], stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            if completed.returncode == 0 and os.path.getsize(placed) > 0:
                return "stripped"
        except OSError:
            pass
        if os.path.lexists(placed):
            os.remove(placed)
    shutil.copy2(executable, placed)
    return "copied"


def prebuild(directory, arguments):
    """--prebuild-into（文件头 ⑥）：编、列目标、把测试二进制与构建输出里的两张表放进 <目录>、写清单；交退出码。"""
    cargo_arguments, packages, _test_arguments = split_arguments(arguments)
    build_exit, artifacts = build(cargo_arguments, packages)
    if build_exit != 0:
        print(f"  ✗ cargo test --no-run 退 {build_exit}：测试二进制没编成，一个都没放进 {directory}")
        print("  → 怎么办：看上面 cargo 打的编译错误；照清单起的那一方读不到清单，自己照常编")
        return build_exit
    metadata, problem = read_metadata(cargo_arguments)
    if metadata is None:
        print(f"  ✗ 读不出包名：{problem}，一个都没放进 {directory}")
        print("  → 怎么办：在树根跑 cargo metadata --no-deps --format-version 1 看它为什么退")
        return 1
    units, unknown = test_units_of(artifacts, metadata[0])
    if units is None:
        print(f"  ✗ 认不出这个测试产物属于哪个包、怎么挑：{unknown.get('package_id')} {unknown.get('target')}，一个都没放进 {directory}")
        print("  → 怎么办：看 cargo metadata 与编译产物的 package_id 对不对得上；对不上就是 cargo 的输出格式变了，照它改 selector_of / read_metadata")
        return 1
    final = directory.rstrip(os.sep)
    staging = final + ".partial"
    shutil.rmtree(staging, ignore_errors=True)
    rows, missing, linked, copied, stripped, table_copies = [], [], 0, 0, 0, 0
    copied_tables = set()   # 已拷过的表（路径、修改时刻、大小）：同一份表不为每个目标各拷一遍
    try:
        os.makedirs(os.path.join(staging, "deps"))
        for package, selector, artifact in units:
            executable = artifact["executable"]
            name = os.path.basename(executable)
            placed = os.path.join(staging, "deps", name)
            try:
                placed_how = place_binary(executable, placed)
                linked += placed_how == "linked"
                copied += placed_how != "linked"
                stripped += placed_how == "stripped"
            except OSError as error:
                missing.append(f"{package} {' '.join(selector)}（{error}）")
                placed = None
            # 门禁核判器输入（crash-amplification-lib.sh 的 crash_amplification_judging_inputs_unchanged）照二进制所在目录上一层的 build/*/out 找表，放进去的照同样的相对位置放
            for table_directory in glob_build_outputs(executable):
                build_name = os.path.basename(os.path.dirname(table_directory))
                for table_name in PREBUILT_TABLE_NAMES:
                    source_table = os.path.join(table_directory, table_name)
                    if os.path.isfile(source_table):
                        source_stat = os.stat(source_table)
                        stamp = (source_table, source_stat.st_mtime_ns, source_stat.st_size)
                        if stamp in copied_tables and not broken("tables-copied-per-target"):
                            continue
                        os.makedirs(os.path.join(staging, "build", build_name, "out"), exist_ok=True)
                        shutil.copy2(source_table, os.path.join(staging, "build", build_name, "out", table_name))
                        copied_tables.add(stamp)
                        table_copies += 1
            target = artifact.get("target") or {}
            rows.append("\t".join([package, " ".join(selector), os.path.join(final, "deps", name) if placed else "-", running_source_of(artifact),
                                   os.path.dirname(artifact.get("manifest_path") or ""), (target.get("name") or "").replace("-", "_")]))
        with open(os.path.join(staging, PREBUILT_MANIFEST_NAME), "w", encoding="utf-8") as handle:
            handle.write(f"{PREBUILT_MANIFEST_HEADER}\t{json.dumps(cargo_arguments + package_options(packages), ensure_ascii=False)}\n")
            handle.writelines(row + "\n" for row in rows)
        shutil.rmtree(final, ignore_errors=True)
        os.replace(staging, final)
    except OSError as error:
        shutil.rmtree(staging, ignore_errors=True)
        print(f"  ✗ 编好的测试二进制放不进 {final}：{error}")
        print("  → 怎么办：看那个目录可不可写、盘满没满；照清单起的那一方读不到清单，自己照常编")
        return 1
    manifest = os.path.join(final, PREBUILT_MANIFEST_NAME)
    print(f"  ✓ 编了一回（cargo test … --no-run），{len(units)} 个测试目标的二进制放进 {final} {len(units) - len(missing)} 个（硬链接 {linked}、拷 {copied}，其中去掉调试段 {stripped}）、"
          f"判器核对的表拷 {table_copies} 份；没放的 {len(missing)} 个{('：' + '；'.join(missing)) if missing else ''}（照清单起时它们经 cargo 起）")
    print(f"{SUMMARY_PREFIX}_PREBUILT targets={len(units)} placed={len(units) - len(missing)} missing={len(missing)} manifest={manifest}")
    return 0


def glob_build_outputs(executable):
    """测试二进制所在目录上一层 build/*/out 里的目录（文件头 ⑥）。"""
    parent = os.path.join(os.path.dirname(os.path.abspath(executable)), os.pardir, "build")
    if not os.path.isdir(parent):
        return []
    return sorted(os.path.join(parent, name, "out") for name in os.listdir(parent) if os.path.isdir(os.path.join(parent, name, "out")))


def read_prebuilt_manifest(path, cargo_words):
    """--binaries 给的清单（文件头 ⑥）→ ([(包, 挑法, PrebuiltBinary 或 None, 源文件)…], None)；读不了、写法不对、编的那一次的 cargo 参数与这一次的不同交 (None, 原因)。
    二进制写 - 或文件不在、不可执行的那一行交 None：照 ③ 经 cargo 起。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as error:
        return None, f"读不了：{error}"
    head, _, recorded = (lines[0] if lines else "").partition("\t")
    if head != PREBUILT_MANIFEST_HEADER:
        return None, f"头一行不是「{PREBUILT_MANIFEST_HEADER}<TAB><cargo 参数>」"
    try:
        recorded_words = json.loads(recorded)
    except ValueError as error:
        return None, f"头一行的 cargo 参数不是 JSON（{error}）"
    if recorded_words != cargo_words:
        return None, f"编的那一次的 cargo 参数 {' '.join(map(str, recorded_words))} 与这一次的 {' '.join(cargo_words)} 不同"
    rows = []
    for line in lines[1:]:
        fields = line.split("\t")
        if len(fields) != 6 or not fields[0] or not fields[1]:
            return None, f"有一行不是六列「<包><TAB><挑法><TAB><二进制><TAB><源文件><TAB><包目录><TAB><crate 名>」：{line[:200]}"
        package, selector, binary, source, package_directory, crate_name = fields
        usable = binary != "-" and os.path.isfile(binary) and os.access(binary, os.X_OK)
        rows.append((package, selector.split(" "), PrebuiltBinary(binary, source, package_directory, crate_name) if usable else None, source))
    if not rows:
        return None, "一个目标都没有"
    return rows, None


def cargo_test_environment(entry, package, environment):
    """直接起一个测试二进制时补上的 cargo 给测试进程设的那几个变量（文件头 ⑥）。"""
    return {"CARGO_MANIFEST_DIR": entry.package_directory, "CARGO_PKG_NAME": package, "CARGO_CRATE_NAME": entry.crate_name,
            "CARGO_PRIMARY_PACKAGE": "1", "CARGO": shutil.which("cargo", path=environment.get("PATH")) or "cargo"}


def cargo_style_failure(exit_code, selector, command):
    """直接起的测试二进制退出之后（文件头 ⑥）：交 (照 cargo test 补的那几行, 这一件的退出码)。退 0 不补；退非 0 照 cargo 的写法补、退 CARGO_TEST_FAILURE_EXIT。
    弄坏开关 cargo-failure-lines-unwritten 下不补、照它自己的退出码。"""
    if exit_code == 0 or broken("cargo-failure-lines-unwritten"):
        return "", exit_code
    status = (f"signal: {-exit_code}, {CARGO_SIGNAL_DESCRIPTIONS.get(-exit_code, f'SIG{-exit_code}: signal')}" if exit_code < 0
              else f"exit status: {exit_code}")
    return (f"error: test failed, to rerun pass `{' '.join(selector)}`\n\nCaused by:\n"
            f"  process didn't exit successfully: `{' '.join(command)}` ({status})\n"), CARGO_TEST_FAILURE_EXIT


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
    上限取这一台能给的内存（module.host_memory_capacity），取不到才取这一条拿到的上限（module.task_memory_cap_bytes）。
    读不进 crates-mutation-rows.py 抛 MemoryToolsUnavailable（不开满）。弄坏开关 memory-ignored：上限交 None；
    memory-by-task-cap：不取这一台能给的内存；memory-no-task-fallback：这一台那一项取不到时不退回这一条拿到的上限。"""
    try:
        module = memory_module()
    except Exception as error:   # 它读的库有一份写坏了的各种异常都在这里：照实报，整趟不派
        raise MemoryToolsUnavailable(f"{type(error).__name__}：{error}") from error
    if broken("memory-ignored"):
        cap, cap_text = None, "弄坏开关 memory-ignored：不看内存上限"
    else:
        try:
            host_cap, host_text = (None, "弄坏开关 memory-by-task-cap：不取这一台能给的内存") if broken("memory-by-task-cap") else module.host_memory_capacity(environment)
        except Exception as error:   # 多机配置读不了一类：这一项取不到，退回服务交的上限，说明里点名
            host_cap, host_text = None, f"{type(error).__name__}：{error}"
        if host_cap is not None:
            cap, cap_text = host_cap, f"这一台能给的内存：{host_text}；不取服务交的上限与自己 cgroup 的 memory.max"
        elif broken("memory-no-task-fallback"):
            cap, cap_text = None, f"弄坏开关 memory-no-task-fallback：这一台能给的内存取不到（{host_text}），也不退回这一条拿到的上限"
        else:
            cap, task_text = module.task_memory_cap_bytes(environment)
            cap_text = f"这一台能给的内存取不到（{host_text}），取这一条拿到的上限：{task_text}"
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


class TargetsProgress:
    """报进度的线程（文件头 ③「进度」）：每 seconds 秒看一次，这段时间有件跑完、或在跑的件的输出文件变长才打一行；只在派活那一段起着，回读之前停。"""

    def __init__(self, seconds, targets):
        self.seconds, self.targets = seconds, targets
        self.lock = threading.Lock()
        self.running = {}            # 件号 → (目标，输出文件)
        self.finished = 0
        self.reported_finished = 0   # 上一次看的时候跑完了几件
        self.seen_bytes = {}         # 件号 → 上一次看到的输出字节
        self.started = time.monotonic()
        self.stopped = threading.Event()
        self.thread = threading.Thread(target=self.loop, daemon=True)

    def start(self):
        if not broken("progress-silent"):
            self.thread.start()

    def stop(self):
        self.stopped.set()
        if self.thread.is_alive():
            self.thread.join()

    def piece_started(self, piece, target, log_path):
        with self.lock:
            self.running[piece] = (target, log_path)

    def piece_finished(self, piece):
        with self.lock:
            self.running.pop(piece, None)
            self.finished += 1

    def loop(self):
        while not self.stopped.wait(self.seconds):
            self.report_if_changed()

    def report_if_changed(self):
        with self.lock:
            running, finished = dict(self.running), self.finished
        sizes = {}
        for piece, (_target, log_path) in running.items():
            try:
                sizes[piece] = os.path.getsize(log_path)
            except OSError:
                sizes[piece] = 0
        grew = any(size > self.seen_bytes.get(piece, 0) for piece, size in sizes.items()) and not broken("progress-ignores-log-growth")
        changed = grew or finished != self.reported_finished
        self.seen_bytes, self.reported_finished = sizes, finished
        if not changed and not broken("progress-every-interval"):
            return
        name = "TEST_TARGETS_PARALLEL_STATUS" if broken("progress-line-renamed") else PROGRESS_LINE_NAME
        shown = "；".join(target for _piece, (target, _log_path) in sorted(running.items())) or "无"
        line = (f"{name} seconds={time.monotonic() - self.started:.0f} targets={self.targets} finished={finished} running={len(running)} "
                f"log_bytes={sum(sizes.values())} running_targets={shown}\n")
        try:
            os.write(1, line.encode("utf-8"))   # 一行一次写完：主线程只在派活的间隙打整行、打完就 flush，不夹进它的半行
        except OSError:
            self.stopped.set()                  # 接输出的那一段没了，后面的进度没人读；件照跑、结局照回读


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


def run(jobs, arguments, binaries=None, drop_sources=None):
    progress_text = os.environ.get(PROGRESS_SECONDS_VARIABLE, str(PROGRESS_SECONDS))
    try:
        progress_seconds = float(progress_text)
    except ValueError:
        progress_seconds = 0.0
    if not 0 < progress_seconds < math.inf:
        print(f"  ✗ {PROGRESS_SECONDS_VARIABLE}={progress_text} 不是正数，一个目标都没跑")
        print(f"  → 怎么办：unset {PROGRESS_SECONDS_VARIABLE}（它只给自证用，平常取 {PROGRESS_SECONDS} 秒），或写成正数再跑")
        return 2
    thread_variables, thread_problem = capped_thread_variables(os.environ.get(CAPPED_SCRIPT_VARIABLE) or CAPPED_SCRIPT_PATH)
    if thread_variables is None:
        if not broken("capped-list-unchecked"):
            print(f"  ✗ 线程类变量清单读不到（{thread_problem}）：一件都没派")
            print("  → 怎么办：看 scripts/hosts/capped.sh 的 variable_names 块还在不在、是不是一行一个变量名；装置的线程变量没跟着分摊，各件合起来会超登记数，所以不照只设 RUST_TEST_THREADS 往下跑")
            return 1
        thread_variables = []
    cargo_arguments, packages, test_arguments = split_arguments(arguments)
    unit_arguments, selected_targets = execution_arguments(cargo_arguments)
    # 编好的测试二进制清单（文件头 ⑥）：用得上就不编；弄坏开关 binaries-through-cargo 下不看清单
    prebuilt_rows = None
    if binaries is not None:
        prebuilt_rows, prebuilt_problem = ((None, "弄坏开关 binaries-through-cargo：不看清单") if broken("binaries-through-cargo")
                                           else read_prebuilt_manifest(binaries, cargo_arguments + package_options(packages)))
        if prebuilt_rows is None:
            print(f"  ! 编好的测试二进制清单 {binaries} 不用（{prebuilt_problem}）：照常编、每个目标经 cargo 起")
    artifacts = []
    if prebuilt_rows is None:
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
    units, direct, unit_sources = [], {}, []   # direct：目标下标 → 直接起的 PrebuiltBinary（文件头 ⑥）；unit_sources：每个目标 cargo 那一行 Running 里的源文件
    if prebuilt_rows is not None:
        through_cargo = []
        for package, selector, entry, source in prebuilt_rows:
            if entry is None:
                through_cargo.append(unit_key(package, selector))
                if broken("missing-binary-dropped"):
                    continue
            else:
                direct[len(units)] = entry
            units.append((package, selector))
            unit_sources.append(source)
        print(f"  · 编好的测试二进制清单 {binaries}：{len(prebuilt_rows)} 个测试目标，这一趟不编；直接起 {len(direct)} 个，经 cargo 起 {len(units) - len(direct)} 个"
              + (f"（清单里没放二进制或文件不在：{'、'.join(through_cargo)}"
                 + ("；弄坏开关 missing-binary-dropped：这几个不派" if broken("missing-binary-dropped") else "") + "）" if through_cargo else ""))
    else:
        found_units, unknown = test_units_of(artifacts, names)
        if found_units is None:
            print(f"  ✗ 认不出这个测试产物属于哪个包、怎么挑：{unknown.get('package_id')} {unknown.get('target')}")
            print("  → 怎么办：看 cargo metadata 与编译产物的 package_id 对不对得上；对不上就是 cargo 的输出格式变了，照它改 selector_of / read_metadata")
            return 1
        units = [(package, selector) for package, selector, _artifact in found_units]
        unit_sources = [running_source_of(artifact) for _package, _selector, artifact in found_units]
    if drop_sources is not None:
        # 这一趟不起的目标（文件头 ⑦）：源文件（去掉库与 bin 前面的 unittests）在清单里的去掉；去掉就一个都不剩的不去（宁多起，不让一件都不起）；清单读不了不派
        try:
            with open(drop_sources, encoding="utf-8") as handle:
                dropped_roots = {line.rstrip("\n") for line in handle if line.strip()}
        except OSError as error:
            print(f"  ✗ --drop-sources 的清单 {drop_sources} 读不了（{error}），一件都没派")
            print("  → 怎么办：清单一行一个目标根文件（门禁 crash-point-replay 写在计划输出旁的 .other-shard-roots），看文件在不在")
            return 1
        keep = (list(range(len(units))) if broken("dropped-targets-started")
                else [index for index in range(len(units)) if unit_sources[index].removeprefix("unittests ") not in dropped_roots])
        if keep and len(keep) < len(units):
            print(f"  · 别的片领的测试目标这一趟不起：{len(units) - len(keep)} 个（清单 {drop_sources}），起 {len(keep)} 个")
            direct = {new: direct[old] for new, old in enumerate(keep) if old in direct}
            units = [units[index] for index in keep]
        elif not keep:
            print(f"  ! --drop-sources 的清单 {drop_sources} 会去掉全部 {len(units)} 个目标：不去，照旧全起")
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
            entry = direct.get(unit)
            if entry is None:
                command = ["cargo", "test"] + unit_arguments + ["-p", package] + selector + ["--"] + passed_arguments + ["--list", "--format", "terse"]
                completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment)
            else:
                command = [entry.binary] + passed_arguments + ["--list", "--format", "terse"]
                completed = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, cwd=entry.package_directory or None,
                                           env=dict(environment, **cargo_test_environment(entry, package, environment)))
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
    progress = TargetsProgress(progress_seconds, len(units))
    progress.start()
    scratch = tempfile.mkdtemp(prefix="test-targets-parallel-")
    thread_counts, elapsed, dispatched, rounds = {}, {}, [], []
    stop_reason, stop_detail, available = None, None, None

    def run_piece(piece, item, threads):
        package, selector = units[item.unit]
        piece_arguments = passed_arguments if item.names is None else libtest_arguments_for_names(passed_arguments, item.names)
        entry = direct.get(item.unit)
        if entry is None:
            command = ["cargo", "test"] + unit_arguments + ["-p", package] + selector + (["--"] + piece_arguments if piece_arguments else [])
            piece_directory, piece_environment = None, {}
        else:
            # 直接起编好的测试二进制（文件头 ⑥）：包目录里起、补上 cargo 给测试进程设的变量
            command = [entry.binary] + piece_arguments
            piece_directory, piece_environment = entry.package_directory or None, cargo_test_environment(entry, package, environment)
        sampler.register(piece, keys[item.unit], len(item.names or ()), threads)
        log_path = os.path.join(scratch, f"{piece}.log")
        piece_started = time.time()
        progress.piece_started(piece, unit_key(package, selector), log_path)
        with open(log_path, "w", encoding="utf-8") as output:
            if entry is not None:
                # cargo 那一行 Running 写在这一件输出的开头：门禁照它按测试二进制切段（文件头 ⑥）
                output.write(f"     Running {entry.source} ({entry.binary})\n")
                output.flush()
            try:
                exit_code = subprocess.run(command, stdout=output, stderr=subprocess.STDOUT, cwd=piece_directory,
                                           env={**environment, **piece_environment, **thread_environment(thread_variables, threads), PIECE_VARIABLE: str(piece)}).returncode
            except OSError as error:
                output.write(f"  ! {command[0]} 起不来（{error}）：这一件按退 {CARGO_TEST_FAILURE_EXIT} 算\n")
                exit_code = CARGO_TEST_FAILURE_EXIT
            if entry is not None:
                failure_text, exit_code = cargo_style_failure(exit_code, selector, command)
                output.write(failure_text)
        elapsed[piece] = time.time() - piece_started
        progress.piece_finished(piece)
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
    progress.stop()
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
                                  + (["--"] + passed_arguments if passed_arguments else []), env={**environment, **thread_environment(thread_variables, jobs)}).returncode
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
        print(f"  → 怎么办：上限取自这一台能给的内存时，是点名的测试函数一条要的内存比整台给的还多：改那条测试（拆小它造的数据）；上限取自服务交的额度时"
              f"（这一台那一项取不到，见上面「内存分片」那一行），服务照 memory-cap-hit 加额度重排；"
              f"峰值记录偏大时删掉 {peaks_directory or '峰值记录'} 下这个目标的那一份（文件头 ⑤「记在哪」）再跑")
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
# 测试函数照 SELFTEST_FUNCTIONS（一行「<目标> <全名> <ok|FAILED>」），没写这个目标的一条 <目标>_case（beta 的 FAILED）；--list 打挑中的「<全名>: test」；
# SELFTEST_CHATTY 点名的目标睡的时候每 0.5 秒打一行；--no-run 往调用记录写一行 build；
# 假测试二进制转过来的（环境里有 SELFTEST_THROUGH_BINARY）记「binary <目标> <工作目录名> <参数>」不记「args」，也不打它自己的 Running 行；
# SELFTEST_THREAD_VARIABLES 点名的变量（空格隔开）收到什么，记一行「variables <目标或 doc> <进程号> <变量>=<值>…」（没设写 unset）
record_variables() {
  local recorded="" variable
  for variable in ${SELFTEST_THREAD_VARIABLES:-}; do recorded+=" $variable=${!variable:-unset}"; done
  [[ -z "${SELFTEST_THREAD_VARIABLES:-}" ]] || echo "variables $1 $$$recorded" >> "$SELFTEST_CALLS"
}
case "$1" in
  metadata) cat "$SELFTEST_METADATA"; exit 0 ;;
  test) ;;
  *) echo "假 cargo 不认：$*" >&2; exit 2 ;;
esac
shift
arguments=" $* "
if [[ "$arguments" == *" --no-run "* ]]; then
  echo "build" >> "$SELFTEST_CALLS"
  echo "   Compiling demo v0.1.0 (/selftest/demo)" >&2
  cat "$SELFTEST_ARTIFACTS"
  exit 0
fi
name=lib
if [[ "$arguments" == *" --doc "* ]]; then name=doc; elif [[ "$arguments" =~ --(test|bin)\ ([a-z_]+) ]]; then name="${BASH_REMATCH[2]}"; fi
if [[ "$name" == doc ]]; then
  echo "doc" >> "$SELFTEST_CALLS"
  echo "args doc $*" >> "$SELFTEST_CALLS"
  record_variables doc
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
if [[ -n "${SELFTEST_THROUGH_BINARY:-}" ]]; then echo "binary $name $(basename "$PWD") $*" >> "$SELFTEST_CALLS"; else echo "args $name $*" >> "$SELFTEST_CALLS"; fi
echo "picked $$ $name ${selected[*]}" >> "$SELFTEST_CALLS"
echo "start $name $(date +%s.%N) ${RUST_TEST_THREADS:-unset} $$" >> "$SELFTEST_CALLS"
record_variables "$name"
if [[ "${SELFTEST_ALLOCATE:-}" == "$name" ]]; then
  python3 -c 'import time; held = bytearray(b"\x01") * (64 << 20); time.sleep(3)'
elif [[ "${SELFTEST_CHATTY:-}" == "$name" ]]; then
  for tick in 1 2 3 4 5 6; do echo "chatty $name $tick"; sleep 0.5; done
else
  sleep 3
fi
[[ -n "${SELFTEST_THROUGH_BINARY:-}" ]] || echo "     Running $name (target/debug/deps/$name-0123456789abcdef)" >&2
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
SELFTEST_MEMORY_CAP_BYTES = 10 * SELFTEST_GIBIBYTE   # 内存那几格给的这一台能给的内存、或服务交的上限（文件头「自证」，各格写明）


SELFTEST_TARGET_SOURCES = {"demo": ("src/lib.rs", ""), "alpha": ("tests/alpha.rs", "--test alpha"), "beta": ("tests/beta.rs", "--test beta"),
                            "tool": ("src/bin/tool.rs", "--bin tool")}   # 自证的四个测试目标：源文件（相对包目录 demo）与假测试二进制转给假 cargo 时挑它的参数
FAKE_TEST_BINARY = r'''#!/usr/bin/env bash
# test-targets-parallel.py 自证的假测试二进制（文件头「自证」编好的测试二进制那三格）：把活转给 PATH 上的假 cargo 跑同一个目标，
# 带 SELFTEST_THROUGH_BINARY：假 cargo 记「binary」不记「args」、不打它自己的 Running 行
SELFTEST_THROUGH_BINARY=1 exec cargo test {selector} -- "$@"
'''


FAKE_OBJCOPY = r'''#!/usr/bin/env bash
# test-targets-parallel.py 自证的假 objcopy（--prebuild-into 拷测试二进制时去调试段用）：objcopy --strip-debug <源> <目标>，目标 = 源的内容再加一行「# 去掉了调试段」
# （仍是能起的脚本），目标的权限同源；往调用记录写一行「strip <源的文件名>」
[[ "$1" == --strip-debug && $# -eq 3 ]] || { echo "假 objcopy 只认 --strip-debug <源> <目标>：$*" >&2; exit 2; }
echo "strip $(basename "$2")" >> "$SELFTEST_CALLS"
{ cat -- "$2"; echo "# 去掉了调试段"; } > "$3"
chmod --reference="$2" "$3"
'''


SELFTEST_DEVICE_VARIABLES = ("SELFTEST_DEVICE_SLOTS", "SELFTEST_POOL_SLOTS")   # 假 capped.sh 的 variable_names 里放的两个装置线程变量（线程类变量那一格）；名字不带 THREADS、JOBS、WORKERS，这份脚本并不真读它们
SELFTEST_UNLISTED_VARIABLE = "SELFTEST_UNLISTED_SLOTS"   # 假 capped.sh 里 variable_names 之外的另一块里的名字：不该被设


def selftest_multi_host_configuration(shared_root, host_bytes):
    """自证里给「这一台能给的内存」用的多机配置原文：HOSTS 写 127.0.0.1（在每一台的网卡上）与一个文档用的假地址，SLICE_MEMORY_CAP_BY_HOST 里这一台那一项写 host_bytes
    （字节数要是 GiB 的整数倍，写成 <n>G），假地址那一项写 1G（读错了那一项就得到小得多的数）；别的键是判法要的最小写法。"""
    return ("HOSTS=127.0.0.1 192.0.2.70\n"
            "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.70=/selftest/other-cargo\n"
            "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.70=/selftest/other-builds\n"
            f"SLICE_MEMORY_CAP_BY_HOST=127.0.0.1={host_bytes // SELFTEST_GIBIBYTE}G 192.0.2.70=1G\n"
            "QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\n"
            f"ENABLE_ACROSS_MACHINES=0\nENABLE_GPU=0\nNFS_SHARED_ROOTS={shared_root}\n")


def selftest_run(works, jobs, history=None, test_arguments=(), memory_cap=None, peaks=None, functions=None, allocate=None, progress_seconds=None, chatty=None,
                 prebuilt=None, prebuild_report=None, thread_variables=None, capped_missing=False, host_memory=None, scope_memory_max=None, link_fails=False,
                 drop_sources=None, drop_sources_absent=False):
    """拿假 cargo 跑一趟：交 (完成的进程, 假 cargo 记的调用行, 跑完的用时表或 None, 这一趟的临时工作目录)。history 给了就写成这一趟的用时表（{「<包> <挑法>」: 秒}）；
    test_arguments 给了就接在第二个 `--` 之后（测试二进制的参数）；memory_cap 是这一趟的 TASK_SERVER_MEMORY_CAP_BYTES（None 摘掉）；
    host_memory 是这一台能给的内存字节数（GiB 的整数倍；给了就在临时工作目录写一份多机配置、SLICE_MEMORY_CAP_BY_HOST 里这一台那一项取它，None 是多机配置指到不在的文件）；
    scope_memory_max 是假 cgroup 的 memory.max 字节数（None 写 max）；link_fails 为真时 --prebuild-into 的硬链接一律当失败（环境里设 TEST_TARGETS_PARALLEL_LINK_FAILS），拷的那一路走假 objcopy；
    peaks 是预先写进峰值记录的 {「<包> <挑法>」: 字节}；functions 是 [(目标, 全名, 结局)]；allocate 是要另起 python3 占 64 MiB 的目标名；
    progress_seconds 是这一趟的进度间隔（None 摘掉那个变量）；chatty 是睡的时候一直打行的目标名。
    thread_variables 给了（变量名的元组）就在临时工作目录写一份假 capped.sh（variable_names 里 CARGO_BUILD_JOBS、RUST_TEST_THREADS 加这几个名字，后面另有一块放 SELFTEST_UNLISTED_VARIABLE）、
    清单位置指到它、环境里这几个变量与 CARGO_BUILD_JOBS 都先设成 jobs（同 capped.sh 设的登记数），假 cargo 记下它们收到的值；capped_missing 是把清单位置指到不在的文件。
    drop_sources 给了（目标根文件的元组）就写成临时工作目录里的清单、带 --drop-sources 起；drop_sources_absent 为真带一个不在的清单。
    prebuilt 是 "all" 或 "missing" 时先起一趟 --prebuild-into（放进临时工作目录的 prebuilt/，完成的进程与清单原文放进 prebuild_report），"missing" 再把清单里 alpha 那一行的
    二进制改写成 -、删掉放进去的 beta，清掉调用记录，这一趟带 --binaries 那份清单起（文件头「自证」编好的测试二进制那三格）。
    这一趟的临时工作目录（峰值记录在它的 peaks/ 下）记进 works，由调用方判完再删。"""
    work = tempfile.mkdtemp(prefix="test-targets-parallel-selftest-")
    works.append(work)
    package_directory = os.path.join(work, "demo")
    deps_directory = os.path.join(work, "target", "debug", "deps")
    package_id = "path+file:///selftest/demo#0.1.0"
    metadata = {"packages": [{"id": package_id, "name": "demo", "targets": [
        {"kind": ["lib"], "name": "demo", "doctest": True}, {"kind": ["test"], "name": "alpha", "doctest": False},
        {"kind": ["test"], "name": "beta", "doctest": False}, {"kind": ["bin"], "name": "tool", "doctest": False}]}],
        "workspace_members": [package_id]}

    def artifact(kinds, name, test, executable=True):
        source = os.path.join(package_directory, SELFTEST_TARGET_SOURCES.get(name, ("build.rs", ""))[0])
        return json.dumps({"reason": "compiler-artifact", "package_id": package_id, "manifest_path": os.path.join(package_directory, "Cargo.toml"),
                           "target": {"kind": kinds, "name": name, "src_path": source}, "profile": {"test": test},
                           "executable": os.path.join(deps_directory, f"{name}-0123") if executable else None})
    artifacts = [artifact(["lib"], "demo", False, executable=False), artifact(["custom-build"], "build-script-build", False),
                 artifact(["lib"], "demo", True), artifact(["test"], "alpha", True), artifact(["test"], "beta", True), artifact(["bin"], "tool", True)]
    # 四个假测试二进制与它们上一层构建输出里的表（--prebuild-into 照同样的相对位置拷过去）
    os.makedirs(deps_directory)
    os.makedirs(package_directory)
    for name, (_source, selector) in SELFTEST_TARGET_SOURCES.items():
        with open(os.path.join(deps_directory, f"{name}-0123"), "w", encoding="utf-8") as handle:
            handle.write(FAKE_TEST_BINARY.replace("{selector}", selector))
        os.chmod(os.path.join(deps_directory, f"{name}-0123"), 0o755)
    table_directory = os.path.join(work, "target", "debug", "build", "demo-1", "out")
    os.makedirs(table_directory)
    for table_name in PREBUILT_TABLE_NAMES:
        with open(os.path.join(table_directory, table_name), "w", encoding="utf-8") as handle:
            handle.write(f"# 自证的 {table_name}\n")
    bin_directory = os.path.join(work, "bin")
    os.makedirs(bin_directory)
    fake = os.path.join(bin_directory, "cargo")
    with open(fake, "w", encoding="utf-8") as handle:
        handle.write(FAKE_CARGO)
    os.chmod(fake, 0o755)
    fake_objcopy = os.path.join(bin_directory, "objcopy")
    with open(fake_objcopy, "w", encoding="utf-8") as handle:
        handle.write(FAKE_OBJCOPY)
    os.chmod(fake_objcopy, 0o755)
    paths = {name: os.path.join(work, name) for name in ("metadata.json", "artifacts.jsonl", "calls.txt", "durations.tsv", "functions.txt", "cgroup-listing", "peaks")}
    with open(paths["metadata.json"], "w", encoding="utf-8") as handle:
        json.dump(metadata, handle)
    with open(paths["artifacts.jsonl"], "w", encoding="utf-8") as handle:
        handle.write("\n".join(artifacts) + "\n")
    if history is not None:
        write_durations(paths["durations.tsv"], history)
    with open(paths["functions.txt"], "w", encoding="utf-8") as handle:
        handle.writelines(f"{target} {function} {outcome}\n" for target, function, outcome in (functions or ()))
    # cgroup 的读法换成临时目录（文件头「自证」）：memory.max 写 max（不收窄；scope_memory_max 给了写它），cgroup.procs 里是自证进程自己（底座就是它）
    scope_directory = os.path.join(work, "cgroup", "selftest.scope")
    os.makedirs(scope_directory)
    with open(os.path.join(scope_directory, "memory.max"), "w", encoding="ascii") as handle:
        handle.write("max\n" if scope_memory_max is None else f"{scope_memory_max}\n")
    with open(os.path.join(scope_directory, "cgroup.procs"), "w", encoding="ascii") as handle:
        handle.write(f"{os.getpid()}\n")
    with open(paths["cgroup-listing"], "w", encoding="ascii") as handle:
        handle.write("0::/selftest.scope\n")
    for key, peak in (peaks or {}).items():
        memory_module().write_atomically(os.path.join(key_directory(paths["peaks"], key), f"{1:020d}-1-selftest"), f"{peak} 1 1\t{key}\n")
    environment = {name: value for name, value in os.environ.items() if name not in ("TASK_SERVER_MEMORY_CAP_BYTES", PROGRESS_SECONDS_VARIABLE)}
    environment.update({"PATH": bin_directory + os.pathsep + os.environ.get("PATH", ""), "SELFTEST_METADATA": paths["metadata.json"],
                        "SELFTEST_ARTIFACTS": paths["artifacts.jsonl"], "SELFTEST_CALLS": paths["calls.txt"], "SELFTEST_FUNCTIONS": paths["functions.txt"],
                        DURATIONS_VARIABLE: paths["durations.tsv"], PEAKS_DIRECTORY_VARIABLE: paths["peaks"],
                        "GATE_MUTATION_CGROUP_LISTING": paths["cgroup-listing"], "GATE_MUTATION_CGROUP_ROOT": os.path.join(work, "cgroup"),
                        "SINGLEFS_MULTI_HOST_CONFIG": os.path.join(work, "multi-host.env.absent")})
    if memory_cap is not None:
        environment["TASK_SERVER_MEMORY_CAP_BYTES"] = str(memory_cap)
    if host_memory is not None:
        configuration = os.path.join(work, "multi-host.env")
        with open(configuration, "w", encoding="utf-8") as handle:
            handle.write(selftest_multi_host_configuration(work, host_memory))
        environment["SINGLEFS_MULTI_HOST_CONFIG"] = configuration
    if thread_variables is not None:
        capped_path = os.path.join(work, "capped.sh")
        with open(capped_path, "w", encoding="utf-8") as handle:
            handle.write("variable_names=(\n    CARGO_BUILD_JOBS\n    RUST_TEST_THREADS\n" + "".join(f"    {name}\n" for name in thread_variables)
                         + f")\ndevice_only_variable_names=({SELFTEST_UNLISTED_VARIABLE})\n")
        environment[CAPPED_SCRIPT_VARIABLE] = capped_path
        environment.update({name: str(jobs) for name in (CARGO_JOBS_VARIABLE, *thread_variables)})
        environment["SELFTEST_THREAD_VARIABLES"] = " ".join((CARGO_JOBS_VARIABLE, *thread_variables, SELFTEST_UNLISTED_VARIABLE))
    if capped_missing:
        environment[CAPPED_SCRIPT_VARIABLE] = os.path.join(work, "capped.absent")
    if allocate is not None:
        environment["SELFTEST_ALLOCATE"] = allocate
    if link_fails:
        environment[LINK_FAILS_VARIABLE] = "1"
    if progress_seconds is not None:
        environment[PROGRESS_SECONDS_VARIABLE] = str(progress_seconds)
    if chatty is not None:
        environment["SELFTEST_CHATTY"] = chatty
    binaries_option = []
    if prebuilt is not None:
        prebuilt_directory = os.path.join(work, "prebuilt")
        report = subprocess.run([sys.executable, os.path.realpath(__file__), "--prebuild-into", prebuilt_directory, "--", "--offline", "--no-fail-fast", "-p", "demo"],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=environment, cwd=work, timeout=300)
        manifest = os.path.join(prebuilt_directory, PREBUILT_MANIFEST_NAME)
        manifest_text = open(manifest, encoding="utf-8").read() if os.path.isfile(manifest) else ""
        if prebuild_report is not None:
            prebuild_report.update({"completed": report, "manifest": manifest_text, "directory": prebuilt_directory})
        if prebuilt == "missing" and manifest_text:
            rows = [line.split("\t") for line in manifest_text.splitlines()]
            for row in rows[1:]:
                if row[1] == "--test alpha":
                    row[2] = "-"
                elif row[1] == "--test beta" and os.path.exists(row[2]):
                    os.remove(row[2])
            with open(manifest, "w", encoding="utf-8") as handle:
                handle.writelines("\t".join(row) + "\n" for row in rows)
        if os.path.exists(paths["calls.txt"]):
            os.remove(paths["calls.txt"])
        binaries_option = ["--binaries", manifest]
    drop_option = []
    if drop_sources is not None or drop_sources_absent:
        drop_path = os.path.join(work, "drop-sources.txt")
        if drop_sources is not None:
            with open(drop_path, "w", encoding="utf-8") as handle:
                handle.writelines(f"{root}\n" for root in drop_sources)
        drop_option = ["--drop-sources", drop_path]
    completed = subprocess.run([sys.executable, os.path.realpath(__file__), "--jobs", str(jobs), *binaries_option, *drop_option, "--", "--offline", "--no-fail-fast", "-p", "demo"]
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


def selftest_variable_rows(calls):
    """假 cargo 记的调用行 → ([(目标, 起点, 终点, RUST_TEST_THREADS, {变量: 值})], 文档测试收到的 {变量: 值})，按进程号配 start、end 与 variables 三种行。"""
    recorded = {line.split()[2]: (line.split()[1], dict(word.split("=", 1) for word in line.split()[3:])) for line in calls if line.startswith("variables ")}
    ends = {line.split()[3]: float(line.split()[2]) for line in calls if line.startswith("end ")}
    pieces = []
    for line in calls:
        if line.startswith("start "):
            words = line.split()
            pieces.append((words[1], float(words[2]), ends.get(words[4], float("inf")), words[3], recorded.get(words[4], (words[1], {}))[1]))
    documents = [values for name, values in recorded.values() if name == "doc"]
    return pieces, (documents[0] if documents else {})


def selftest_heaviest_overlap(invocations, estimate_of):
    """任何一件起跑那一刻在跑的各件估值之和的最大（estimate_of(目标, 挑中的全名) → 字节）。"""
    return max((sum(estimate_of(name, picked) for name, start, end, picked in invocations if start <= moment < end) for _name, moment, _end, _picked in invocations), default=0)


def selftest_progress_rows(stdout):
    """标准输出里的进度行 → [(finished, running_targets)]。"""
    form = re.compile(PROGRESS_LINE_NAME + r" seconds=[0-9]+ targets=4 finished=([0-9]+) running=[0-9]+ log_bytes=[0-9]+ running_targets=(.+)")
    return [(int(found.group(1)), found.group(2)) for found in (form.fullmatch(line) for line in stdout.splitlines()) if found]


def selftest_outcomes(stdout):
    return {found.group(1): found.group(2) for found in re.finditer(r"^test (\S+) \.\.\. (ok|FAILED)$", stdout, re.M)}


def selftest_prebuilt_cells(works):
    """编好的测试二进制那三格（文件头「自证」与 ⑥）：交判错的说明，一格一句。"""
    failures = []
    through_cargo, _, _, _ = selftest_run(works, 4)
    expected = selftest_outcomes(through_cargo.stdout)
    report = {}
    direct, calls, _, _ = selftest_run(works, 4, prebuilt="all", prebuild_report=report)
    prebuild_completed = report.get("completed")
    manifest_rows = [line.split("\t") for line in report.get("manifest", "").splitlines()[1:]]
    placed = [row for row in manifest_rows if len(row) == 6 and row[2] != "-" and os.path.isfile(row[2])]
    tables_copied = os.path.isfile(os.path.join(report.get("directory", ""), "build", "demo-1", "out", PREBUILT_TABLE_NAMES[0]))
    through_binary = {line.split()[1]: line.split()[2] for line in calls if line.startswith("binary ")}
    through_args = sorted(line.split()[1] for line in calls if line.startswith("args ") and line.split()[1] != "doc")
    sources = sorted(re.findall(r"^     Running (.+) \(", direct.stdout, re.M))
    wanted_sources = sorted(f"unittests {source}" if source.startswith("src/") else source for source, _selector in SELFTEST_TARGET_SOURCES.values())
    prebuild_text = prebuild_completed.stdout[-300:] if prebuild_completed is not None else "没起"
    if (prebuild_completed is None or prebuild_completed.returncode != 0 or len(placed) != 4 or not tables_copied
            or f"{SUMMARY_PREFIX}_PREBUILT targets=4 placed=4 missing=0" not in prebuild_completed.stdout
            or "build" in calls or through_args or through_binary != {"lib": "demo", "alpha": "demo", "beta": "demo", "tool": "demo"}
            or sources != wanted_sources or selftest_outcomes(direct.stdout) != expected or direct.returncode != through_cargo.returncode):
        failures.append(f"直接起编好的测试二进制与经 cargo 起的逐条结局相同：--prebuild-into 应当退 0、四个目标都放进去、表拷过去，--binaries 那一趟不编、四个目标都是二进制在包目录 demo 里起、"
                        f"Running 行的源文件是 {wanted_sources}、逐条结局与退出码同经假 cargo 的那一趟；实际 --prebuild-into 退 "
                        f"{prebuild_completed.returncode if prebuild_completed is not None else '（没起）'}、放进去 {len(placed)} 个、表拷没拷 {tables_copied}（输出尾部 {prebuild_text!r}），"
                        f"调用记录里 build {calls.count('build')} 行、经 cargo 的 {through_args}、二进制起的 {through_binary}、Running 行 {sources}，"
                        f"逐条结局 {sorted(selftest_outcomes(direct.stdout).items())} 对 {sorted(expected.items())}，退 {direct.returncode} 对 {through_cargo.returncode}")
    partial, calls, _, _ = selftest_run(works, 4, prebuilt="missing", prebuild_report={})
    through_binary_names = sorted(line.split()[1] for line in calls if line.startswith("binary "))
    through_args = sorted(line.split()[1] for line in calls if line.startswith("args ") and line.split()[1] != "doc")
    if through_binary_names != ["lib", "tool"] or through_args != ["alpha", "beta"] or "build" in calls \
            or selftest_outcomes(partial.stdout) != expected or partial.returncode != through_cargo.returncode:
        failures.append(f"清单里没放二进制的目标仍经 cargo 起：清单里 alpha 写 -、beta 的二进制删掉，应当 alpha、beta 经假 cargo 起、lib、tool 是二进制起的、不编、逐条结局与退出码同经假 cargo 的那一趟；"
                        f"实际经 cargo 的 {through_args}、二进制起的 {through_binary_names}、build {calls.count('build')} 行，"
                        f"逐条结局 {sorted(selftest_outcomes(partial.stdout).items())}，退 {partial.returncode}")
    # 门禁 crash-point-replay 认被停照这一句的写法（它的 crash_point_replay_note_stop）
    stop_form = re.compile(r"process didn't exit successfully: .*\(signal: (2, SIGINT|9, SIGKILL|15, SIGTERM)")
    green = cargo_style_failure(0, ["--lib"], ["/selftest/lib-1"])
    red_text, red_exit = cargo_style_failure(101, ["--test", "beta"], ["/selftest/beta-1", "--exact"])
    stopped_text, stopped_exit = cargo_style_failure(-15, ["--lib"], ["/selftest/lib-1"])
    killed_text, _killed_exit = cargo_style_failure(-9, ["--lib"], ["/selftest/lib-1"])
    crashed_text, _crashed_exit = cargo_style_failure(-11, ["--lib"], ["/selftest/lib-1"])
    if (green != ("", 0) or red_exit != CARGO_TEST_FAILURE_EXIT or "error: test failed, to rerun pass `--test beta`" not in red_text
            or stopped_exit != CARGO_TEST_FAILURE_EXIT or not stop_form.search(stopped_text) or not stop_form.search(killed_text)
            or "(signal: 11, SIGSEGV" not in crashed_text):
        failures.append(f"直接起的测试二进制退非 0 时照 cargo 的写法报、这一件退 101：退 0 应当不补、退 101 与死于信号应当补 cargo 那几行并退 {CARGO_TEST_FAILURE_EXIT}，"
                        f"实际退 0 交 {green!r}、退 101 交 ({red_text!r}, {red_exit})、SIGTERM 交 ({stopped_text!r}, {stopped_exit})、SIGKILL {killed_text!r}、SIGSEGV {crashed_text!r}")
    return failures


def selftest_strip_cell(works):
    """拷的二进制去调试段、判器核对的表只拷一份那一格（文件头 ⑥ 与「自证」）：交判错的说明。"""
    report = {}
    selftest_run(works, 4, prebuilt="all", prebuild_report=report, link_fails=True)
    completed = report.get("completed")
    manifest_rows = [line.split("\t") for line in report.get("manifest", "").splitlines()[1:]]
    placed = [row[2] for row in manifest_rows if len(row) == 6 and row[2] != "-" and os.path.isfile(row[2])]
    marked = [path for path in placed if open(path, encoding="utf-8").read().endswith("# 去掉了调试段\n")]
    text = completed.stdout if completed is not None else ""
    if (completed is None or completed.returncode != 0 or len(placed) != 4 or len(marked) != 4
            or "硬链接 0、拷 4，其中去掉调试段 4" not in text or "判器核对的表拷 2 份" not in text):
        return [f"拷的二进制去掉调试段、判器核对的表只拷一份：硬链接一律失败时 --prebuild-into 应当退 0、四个目标都放进去、四份都是 objcopy --strip-debug 写出的（末行带「# 去掉了调试段」）、"
                f"输出里写「硬链接 0、拷 4，其中去掉调试段 4」与「判器核对的表拷 2 份」（四个目标共用一个构建输出目录里的两张表）；"
                f"实际退 {completed.returncode if completed is not None else '（没起）'}、放进去 {len(placed)} 个、去了调试段的 {len(marked)} 个，输出 {text[-400:]!r}"]
    return []


def selftest_drop_cell(works):
    """不起 --drop-sources 点名的目标那一格（文件头 ⑦ 与「自证」）：交判错的说明。"""
    failures = []
    dropped = ("tests/alpha.rs", "src/bin/tool.rs")
    through_cargo, calls, _, _ = selftest_run(works, 4, drop_sources=dropped)
    started = sorted(line.split()[1] for line in calls if line.startswith("start "))
    summary = [line for line in through_cargo.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    report = {}
    direct, direct_calls, _, _ = selftest_run(works, 4, prebuilt="all", prebuild_report=report, drop_sources=dropped)
    direct_started = sorted(line.split()[1] for line in direct_calls if line.startswith("binary "))
    direct_summary = [line for line in direct.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    if (started != ["beta", "lib"] or not summary or "dispatched=2 collected=2" not in summary[-1] or "别的片领的测试目标这一趟不起：2 个" not in through_cargo.stdout
            or direct_started != ["beta", "lib"] or not direct_summary or "dispatched=2 collected=2" not in direct_summary[-1] or "build" in direct_calls):
        failures.append(f"不起 --drop-sources 点名的目标：清单点名 tests/alpha.rs、src/bin/tool.rs，经 cargo 起的一趟与照编好的清单直接起的一趟都应当只起 lib、beta、末行 dispatched=2 collected=2、"
                        f"输出写「别的片领的测试目标这一趟不起：2 个」，实际经 cargo 起了 {started}、末行 {summary}，直接起了 {direct_started}、末行 {direct_summary}")
    everything = ("src/lib.rs", "tests/alpha.rs", "tests/beta.rs", "src/bin/tool.rs")
    kept, calls, _, _ = selftest_run(works, 4, drop_sources=everything)
    kept_started = sorted(line.split()[1] for line in calls if line.startswith("start "))
    if kept_started != ["alpha", "beta", "lib", "tool"] or "会去掉全部 4 个目标：不去" not in kept.stdout:
        failures.append(f"不起 --drop-sources 点名的目标：清单点名全部四个目标时一个都不去、打一行「会去掉全部 4 个目标：不去」，实际起了 {kept_started}，输出尾部 {kept.stdout[-300:]!r}")
    absent, calls, _, _ = selftest_run(works, 4, drop_sources_absent=True)
    if absent.returncode != 1 or "读不了" not in absent.stdout or any(line.startswith("start ") for line in calls):
        failures.append(f"不起 --drop-sources 点名的目标：清单不在时应当退 1、输出点名读不了、一件都不派，实际退 {absent.returncode}、起了 {sum(1 for line in calls if line.startswith('start '))} 件，输出尾部 {absent.stdout[-300:]!r}")
    return failures


def selftest_thread_variable_cell(works, history):
    """线程类变量那一格（文件头「自证」）：交判错的说明。"""
    failures = []
    jobs = 5
    _, calls, _, _ = selftest_run(works, jobs, history, thread_variables=SELFTEST_DEVICE_VARIABLES)
    pieces, document = selftest_variable_rows(calls)
    watched = (*SELFTEST_DEVICE_VARIABLES, "RUST_TEST_THREADS")
    off_share = [(name, variable, values.get(variable)) for name, _start, _end, threads, values in pieces for variable in SELFTEST_DEVICE_VARIABLES if values.get(variable) != threads]
    heaviest = {variable: max((sum(int(values.get(variable, "0")) for _n, other_start, other_end, _t, values in pieces if other_start <= moment < other_end)
                               for _name, moment, _end, _threads, _values in pieces), default=0) for variable in watched}
    cargo_jobs_moved = [name for name, _start, _end, _threads, values in pieces if values.get(CARGO_JOBS_VARIABLE) != str(jobs)]
    unlisted_set = [name for name, _start, _end, _threads, values in pieces if values.get(SELFTEST_UNLISTED_VARIABLE) != "unset"]
    if (len(pieces) != 4 or off_share or any(total > jobs for total in heaviest.values()) or cargo_jobs_moved or unlisted_set
            or any(document.get(variable) != str(jobs) for variable in SELFTEST_DEVICE_VARIABLES)):
        failures.append(f"线程类变量按份额设给每一件：--jobs {jobs}、环境里装置线程变量先都是 {jobs}，假 capped.sh 的 variable_names 点名的两个装置变量应当等于每一件自己的 RUST_TEST_THREADS、"
                        f"同时在跑的各件合起来每个变量不超过 {jobs}、文档测试收到 {jobs}、{CARGO_JOBS_VARIABLE} 仍是 {jobs}、{SELFTEST_UNLISTED_VARIABLE} 没设；"
                        f"实际起了 {len(pieces)} 件、与份额不等的 {off_share}、各变量同时在跑的合计最大 {heaviest}、{CARGO_JOBS_VARIABLE} 变了的 {cargo_jobs_moved}、"
                        f"{SELFTEST_UNLISTED_VARIABLE} 被设了的 {unlisted_set}、文档测试收到 {document}")
    refused, calls, _, _ = selftest_run(works, jobs, capped_missing=True)
    if refused.returncode != 1 or "capped.sh" not in refused.stdout or "一件都没派" not in refused.stdout or any(line.startswith(("start ", "build")) for line in calls):
        failures.append(f"线程类变量按份额设给每一件：清单位置指到不在的文件时应当退 1、输出点名 capped.sh 与「一件都没派」、没编也没起件，"
                        f"实际退 {refused.returncode}、起了 {sum(1 for line in calls if line.startswith('start '))} 件、编了 {calls.count('build')} 次，输出尾部 {refused.stdout[-300:]!r}")
    return failures


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
    # 内存的七格（文件头 ⑤ 与「自证」）
    gibibyte, cap = SELFTEST_GIBIBYTE, SELFTEST_MEMORY_CAP_BYTES
    even = {"demo --lib": 3 * gibibyte, "demo --test alpha": 3 * gibibyte, "demo --test beta": 3 * gibibyte, "demo --bin tool": 3 * gibibyte}
    completed, calls, _, _ = selftest_run(works, 4, host_memory=cap, peaks=even)
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
    sharded, calls, _, _ = selftest_run(works, 4, test_arguments=skipped, host_memory=cap, peaks=lopsided, functions=beta_functions)
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
    completed, calls, _, _ = selftest_run(works, 4, host_memory=cap, peaks=oversized)
    started = [line for line in calls if line.startswith("start ")]
    if completed.returncode != MEMORY_CAP_HIT_EXIT or "撞内存上限" not in completed.stdout or "tool_case" not in completed.stdout \
            or "memory-cap-hit" not in completed.stdout or started:
        failures.append(f"一片只剩一条还超时按撞内存上限退出、不硬跑：tool 记 40 GiB、只有一条、这一台能给的内存 10 GiB，应当退 {MEMORY_CAP_HIT_EXIT}、点名 tool_case 与 memory-cap-hit、一件都不起，"
                        f"实际退 {completed.returncode}、起了 {len(started)} 件，输出尾部 {completed.stdout[-300:]!r}")
    # 服务交的上限与 memory.max 都很小、这一台能给的内存充裕：分片与同时开的件数不因它们变小（文件头 ⑤「上限」）
    completed, calls, _, _ = selftest_run(works, 4, memory_cap=4 * gibibyte, host_memory=50 * gibibyte, scope_memory_max=3 * gibibyte, peaks=even)
    summary = [line for line in completed.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    if not summary or "memory_shards=1 shard_pieces=4 shard_jobs=4 " not in summary[-1] or "bound=threads" not in summary[-1]:
        failures.append(f"服务交的上限与 memory.max 都很小而这一台能给的内存充裕时分片与同时开的件数不变：这一台能给 50 GiB、服务交 4 GiB、假 cgroup 的 memory.max 写 3 GiB、四个目标各记 3 GiB，"
                        f"末行应当 memory_shards=1 shard_pieces=4 shard_jobs=4 bound=threads，实际末行 {summary}、退 {completed.returncode}，输出尾部 {completed.stdout[-300:]!r}")
    # 这一台那一项取不到（多机配置不在）：退回服务交的上限，照它分片（文件头 ⑤「上限」）
    completed, calls, _, _ = selftest_run(works, 4, memory_cap=cap, peaks=even)
    summary = [line for line in completed.stdout.splitlines() if line.startswith(SUMMARY_PREFIX + " ")]
    if not summary or "memory_shards=2 shard_pieces=3,1 " not in summary[-1] or "bound=memory" not in summary[-1]:
        failures.append(f"这一台那一项没写时退回服务交的上限：多机配置不在、服务交 10 GiB、四个目标各记 3 GiB，末行应当 memory_shards=2 shard_pieces=3,1 bound=memory，"
                        f"实际末行 {summary}、退 {completed.returncode}，输出尾部 {completed.stdout[-300:]!r}")
    _, _, _, work = selftest_run(works, 4, allocate="alpha")
    alpha_key = "demo --test alpha"
    alpha_peak, alpha_note = measured_target_peak(os.path.join(work, "peaks"), alpha_key)
    alpha_directory = key_directory(os.path.join(work, "peaks"), alpha_key)
    alpha_records = [open(os.path.join(alpha_directory, name), encoding="utf-8").read() for name in sorted(os.listdir(alpha_directory))] if os.path.isdir(alpha_directory) else []
    if alpha_peak is None or alpha_peak < 48 << 20 or not any(record.split(" ")[1:2] == ["1"] for record in alpha_records):
        failures.append(f"每个测试目标量到的峰值记进共享根：alpha 那一件另起一个 python3 占 64 MiB，跑完记录里「{alpha_key}」的峰值应当不小于 48 MiB、条数写 1，"
                        f"实际峰值 {alpha_peak}（{alpha_note or '读得出'}）、记录 {alpha_records}")
    # 进度的三格（文件头 ③「进度」与「自证」）
    one_by_one = ["demo --lib", "demo --test alpha", "demo --test beta", "demo --bin tool"]   # 没有用时表，照编译输出的次序起
    run_started = time.monotonic()
    completed, _, _, _ = selftest_run(works, 1, progress_seconds=1)
    run_seconds = time.monotonic() - run_started
    rows = selftest_progress_rows(completed.stdout)
    misnamed = [row for row in rows if row[1] != "无" and (row[0] >= len(one_by_one) or row[1] != one_by_one[row[0]])]
    if sum(1 for finished, _ in rows if finished >= 1) < 2 or misnamed:
        failures.append(f"每过一段时间报一行：跑完几件、在跑哪几个：--jobs 1、进度间隔 1 秒，应当至少两行 finished 不小于 1、每行的 running_targets 正是照派活次序排第 finished 个的目标，实际 {rows}")
    printed = [line for line in completed.stdout.splitlines() if line.startswith(PROGRESS_LINE_NAME + " ")]
    if run_seconds < 12 or len(printed) > 8:
        failures.append(f"没有件跑完、输出文件没变长的那段时间不报：这一趟应当跑满 12 秒、至多 8 行（四件跑完、四件打结果行各至多一次），实际跑了 {run_seconds:.1f} 秒、{len(printed)} 行")
    completed, _, _, _ = selftest_run(works, 4, progress_seconds=1, chatty="lib")
    rows = selftest_progress_rows(completed.stdout)
    if not any(finished == 0 and "demo --lib" in targets.split("；") for finished, targets in rows):
        failures.append(f"输出文件变长也报一行：--jobs 4、lib 每 0.5 秒打一行，还没有件跑完时应当有一行 finished=0、running_targets 里有 demo --lib，实际 {rows}")
    failures += selftest_prebuilt_cells(works)
    failures += selftest_strip_cell(works)
    failures += selftest_drop_cell(works)
    failures += selftest_thread_variable_cell(works, history)
    if failures:
        for failure in failures:
            print(f"  ✗ test-targets-parallel 自检：{failure}")  # gate-lint:detail
        print(f"  → 看 run() 的派活、线程记账、用时表、内存分片、峰值记录、回读与条数核对、TargetsProgress；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ test-targets-parallel 自检通过（{len(SELFTEST_CELLS)} 格：{'、'.join(SELFTEST_CELLS)}）")
    return 0


def main():
    arguments = sys.argv[1:]
    if arguments == ["--selftest"]:
        sys.exit(selftest())
    if len(arguments) >= 3 and arguments[0] == "--prebuild-into" and arguments[1] and arguments[2] == "--":
        sys.exit(prebuild(arguments[1], arguments[3:]))
    binaries, drop_sources = None, None
    if len(arguments) >= 5 and arguments[2] == "--binaries" and arguments[3]:
        binaries = arguments[3]
        arguments = arguments[:2] + arguments[4:]
    if len(arguments) >= 5 and arguments[2] == "--drop-sources" and arguments[3]:
        drop_sources = arguments[3]
        arguments = arguments[:2] + arguments[4:]
    if len(arguments) < 3 or arguments[0] != "--jobs" or not arguments[1].isdigit() or int(arguments[1]) < 1 or arguments[2] != "--":
        print("  ✗ 用法：test-targets-parallel.py --jobs <正整数> [--binaries <清单>] [--drop-sources <文件>] -- <cargo test 的参数…>；或 --prebuild-into <目录> -- <cargo test 的参数…>", file=sys.stderr)
        print("  → 例：test-targets-parallel.py --jobs 31 -- --offline --no-fail-fast -p singlefs-core；线程数是登记值（每台默认重活线程数，由 task-server 套的 capped.sh 设），自检：--selftest", file=sys.stderr)
        sys.exit(2)
    sys.exit(run(int(arguments[1]), arguments[3:], binaries, drop_sources))


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
