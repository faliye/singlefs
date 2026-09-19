# 崩溃一致性验证：m2-supp3-item1-r2

改动范围（`git diff --stat -- crates litmus`）：

```
 crates/mutations.tsv                               |  17 +
 crates/singlefs-harness/src/crash.rs               | 874 +++++++++++++++++++--
 crates/singlefs-harness/src/lib.rs                 |   6 +
 .../tests/second_transaction_step_zero_layer0.rs   |  71 +-
 4 files changed, 914 insertions(+), 54 deletions(-)
```

另有工作区未跟踪文件 `research/prompts/m2-supp3-item1-r2-crash-verifier-scope.txt`，以及已修改但不在 `crates`/`litmus` 范围内的 `.claude/gate.d/54-layer0-replay.sh`（这一份不计入本报告的 fingerprint，fingerprint 只钉 `crates` 与 `litmus`）。

只跑登记给 crash-verifier 的两道：54 号、59 号。55、57 号按派发要求不跑（本轮没碰 QEMU/herd7 罩的东西），旁证见「环境旁证」一节。

开跑前查负载（两阶段各查一次）：`ps -eo pid,args -u "$(id -u)"` 过滤 `cargo|gate.sh|qemu-system|vm-bench|e152-file-system-benchmark|fio|mutate|replay`，两次都只命中本会话自己这一串 `claude` 进程，没有别的门禁或性能测量在跑，两道都正常起跑。

## 门禁 54 号：层 0 全量重放（两条流）

| 项 | 值 |
|---|---|
| 退出码 | 0（`.log` 末行 `exit=0`） |
| 开跑 | 2026-09-19T00:27:56Z（UTC；东京 09:27:56） |
| 结束 | 2026-09-19T00:36:34Z（UTC；东京 09:36:34） |
| 总耗时 | 8 分 38 秒 |
| 线程数 | 32（`SINGLEFS_LAYER0_THREADS=32`，没设，取本机核数；`nproc`=32），两条流的成功行都报了同样的线程数与来源 |
| 第一条流用时（harness 自报） | `elapsed_seconds=7.3`（枚举阶段本身；不含编译与该二进制里其它快用例） |
| 第二条流用时（harness 自报） | `elapsed_seconds=490.3`（枚举阶段本身，约 8 分 10 秒） |

这是并行版第一次在主工作区跑（单线程那一趟 `research/prompts/m2-wave2-crash-verifier/54-layer0-replay.log` 总耗时 3 小时 50 分 49 秒：`START 2026-09-18T04:22:54Z` 到 `END 2026-09-18T08:13:43Z`），32 线程后总耗时降到 8 分 38 秒。

### 原样抄的三行 ✓

第一条流（第一个事务那条流）：

```
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
```

第一条流逐条不变量（CHECKER）：

```
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-1.3=262165/0 I-1.4=262165/0 I-1.6=262165/0 I-1.7=262165/0 I-2.1=262165/0 I-2.3=262165/0 I-2.4=262165/0 I-2.5=262165/0 I-3.1=4/0 I-3.8=262165/0 I-3.9=4/0 I-4.8=262165/0 I-5.1=262165/0 I-5.2=4/0 I-5.4=4/0 I-7.1=262165/0 I-7.2=262165/0 I-7.4=262165/0 I-7.6=262165/0 I-7.7=262165/0 I-7.8=262165/0 I-9.1=4/0 I-9.2=4/0 I-9.4=4/0 I-9.7=4/0 I-9.10=4/0 I-9.13=4/0 I-9.14=0/0
```

第二条流（两次发布那条流，多版本 oracle）：

```
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=4 no_file=262158 file_read=1842255 failed=0 journal_differing=24 verification_ran=42 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 I-1.3=2104413/0/0 I-1.4=2104413/0/0 I-1.6=2104413/0/0 I-1.7=2104413/0/0 I-2.1=2104413/0/0 I-2.3=2104413/0/0 I-2.4=2104413/0/0 I-2.5=2104413/0/0 I-3.1=1842252/0/262161 I-3.8=2104413/0/0 I-3.9=1842252/0/262161 I-4.8=2104413/0/0 I-5.1=2104413/0/0 I-5.2=1842252/0/262161 I-5.4=1842252/0/262161 I-7.1=2104413/0/0 I-7.2=2104413/0/0 I-7.4=2104413/0/0 I-7.6=2104413/0/0 I-7.7=2104413/0/0 I-7.8=2104413/0/0 I-9.1=1842252/0/262161 I-9.2=1842252/0/262161 I-9.4=1842252/0/262161 I-9.7=1842252/0/262161 I-9.10=1842252/0/262161 I-9.13=1842252/0/262161 I-9.14=1580105/0/524308 first_violation=none
```

### 与参照产物逐字比对

**两条流的计数行 vs 上午单线程那一趟 `research/prompts/m2-wave2-crash-verifier/54-layer0-replay.log`**（「：」之后的计数部分，不比前缀——前缀因这轮改成多线程而多了线程数与来源，属于本轮改动本身要新增的信息）：

- 第一条流：新跑的 `states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true` 与该文件第 5 行「：」之后的部分逐字相同（`diff` 空差）。
- 第二条流：新跑的 `states=2104413 closed_form=2104413 exhaustive=true violations=0 root_persisted_states=4 no_file=262158 file_read=1842255 failed=0 journal_differing=24 verification_ran=42 verification_failed=0 record_root_without_record=0 record_claimed_state_missing_unit=0 checker_violations=0 I-1.1=2104413/0/0 …（后略，与上面第二条 ✓ 行相同）… first_violation=none` 与该文件第 6 行「：」之后的部分逐字相同（`diff` 空差）。

**第一条流的 CHECKER 行 vs `research/prompts/m2-wave2-crash-verifier/54-supplement-first-txn-checker-line.log` 第 43 行**（`CHECKER ` 之后的部分）：新跑的 `record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 I-1.3=262165/0 I-1.4=262165/0 I-1.6=262165/0 I-1.7=262165/0 I-2.1=262165/0 I-2.3=262165/0 I-2.4=262165/0 I-2.5=262165/0 I-3.1=4/0 I-3.8=262165/0 I-3.9=4/0 I-4.8=262165/0 I-5.1=262165/0 I-5.2=4/0 I-5.4=4/0 I-7.1=262165/0 I-7.2=262165/0 I-7.4=262165/0 I-7.6=262165/0 I-7.7=262165/0 I-7.8=262165/0 I-9.1=4/0 I-9.2=4/0 I-9.4=4/0 I-9.7=4/0 I-9.10=4/0 I-9.13=4/0 I-9.14=0/0` 与该行逐字相同（`diff` 空差）。

三处都用 `diff <(新跑结果) <(参照文件那一行)` 现比过，三次都是空差（判定：完全相同）。**推翻条件**：任何一处 `diff` 有输出，就要判「多线程改法算出的数与单线程不一致」，红。

### 那两个时刻的负载

- 开跑前（2026-09-19T00:26:49Z）：`ps -eo pid,args -u "$(id -u)"` 过滤门禁/编译/性能测量相关模式，只命中本会话自己这串 `claude` 进程，没有别的 `cargo`、`gate.sh`、`qemu-system` 等。
- 跑完时（2026-09-19T00:38:00Z）：`uptime` 报 `load average: 5.82, 19.26, 16.20`；`ps --sort=-pcpu` 顶部是两个 `ray::RayWorkerProc.run`（各 24.9% CPU，运行了 584023 秒 ≈ 6.76 天，是本机常驻的 GPU 推理服务 vllm/Ray，与门禁 54 无关，不匹配「不做」一节要拦的 `qemu-system`/`vm-bench.sh`/`e152-file-system-benchmark`/`fio` 任何一个模式），没有别的 `cargo`/`gate.sh` 在跑。两个时刻都符合「不做」一节可以照常跑的条件。

## 门禁 59 号：crates 变异表复跑（145 行、140 条变异）

| 项 | 值 |
|---|---|
| 退出码 | 0（`.log` 末行 `exit=0`） |
| 开跑 | 2026-09-19T00:38:28Z（UTC；东京 09:38:28） |
| 结束 | 2026-09-19T00:42:29Z（UTC；东京 09:42:29） |
| 总耗时 | 4 分 1 秒 |
| 开跑前负载 | 与 54 号「跑完时」是同一次查（00:38:00Z 前后一分钟内），只有常驻的 Ray/vllm 进程，无别的 `cargo`/`gate.sh` |

`crates/mutations.tsv` 共 145 行，其中第 1–5 行是表头注释（`^#`），140 行是数据行（脚本自己数出 `140 条变异`，与 `grep -c '红了$'` 数出的 140 条一致）。

### 原样抄的成功行

```
  ✓ crates 变异表复跑：140 条变异各自红在点名的测试上（原文都恰好命中一次）
```

锚点腐化检查（原文命中次数≠1）没有触发——脚本走到了主循环并跑完全部 140 条，没有在锚点校验那一步提前退出。

### 报红 / 没红 / 没跑到 计数

| 结果 | 条数 | 逐条列出 |
|---|---|---|
| 报红（点名的测试判红） | 140 | 全部；见下方逐条清单（第 129–145 行） |
| 没红（测试跑了但没判红） | 0 | 无 |
| 没跑到（点名的测试没跑到） | 0 | 无 |

说明：`.log` 里字面含有「没跑到」「没红」的两处（`步 6：I-3.8 的判定没跑到…` `步 6：I-2.1 的判定没跑到…`）是两条变异**自己的名字**（这两条变异测的正是「层 0 报『不适用』时阴性结果要不要与没跑到分清」这件事本身），两行末尾都是「… 红了」，属于报红那一类，不是脚本判定的「没红」或「没跑到」。已现查：`grep -c '红了$'`=140，与 140 条变异总数相等，`grep -n '✗'` 对本文件无命中，脚本的失败分支（`有变异没红`）整段都没出现在输出里。

### 第 129–145 行各自的结果行（原样抄）

| TSV 行 | 结果行 |
|---|---|
| 129 | `✓ 增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了` |
| 130 | `✓ 增补 3 第 1 件：随机历史偏向抬 F 之后复用的取样点判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）：reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms 红了` |
| 131 | `✓ 增补 2 第 41 行 层 0 并行：丢掉第一片（状态数等于闭式的断言要红）：every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims 红了` |
| 132 | `✓ 增补 2 第 41 行 层 0 并行：相邻两片重叠（状态数等于闭式的断言要红）：every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims 红了` |
| 133 | `✓ 增补 2 第 41 行 层 0 并行：相邻两片重叠（切片单测）：state_slices_cover_every_state_exactly_once_in_ordinal_order 红了` |
| 134 | `✓ 增补 2 第 41 行 层 0 并行：按序号取状态时段内子集掩码取反：the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment 红了` |
| 135 | `✓ 增补 2 第 41 行 层 0 并行：并片时第一处违例取后面那一片的：one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread 红了` |
| 136 | `✓ 增补 2 第 41 行 层 0 并行：并片时 checker 每条不变量的第一处违例取后面那一片的：absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation 红了` |
| 137 | `✓ 增补 2 第 41 行 层 0 并行：不读 SINGLEFS_LAYER0_THREADS：worker_threads_come_from_the_environment_variable_before_available_parallelism 红了` |
| 138 | `✓ 增补 2 第 41 行 层 0 并行：SINGLEFS_LAYER0_THREADS=0 悄悄退回 1 个线程：zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back 红了` |
| 139 | `✓ 增补 3 第 1 件（代码三方第一轮第 1 条）：已知红第 1 条不看 F 是否落在回退留下的空档里：an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding 红了` |
| 140 | `✓ 增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了` |
| 141 | `✓ 增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了` |
| 142 | `✓ 增补 2 第 41 行 层 0 并行（代码三方第一轮 Y5m2）：并片时「不看 journal 那一遍的第一处」取后面那一片的：absorbing_a_following_slice_keeps_the_earlier_first_ignored_violation_when_both_slices_have_one 红了` |
| 143 | `✓ 增补 3 第 1 件（代码三方第二轮第 1 条，形态 a）：只在挂载写出的写行与暖机里复用改写已回收记录时不改分配代：row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked 红了` |
| 144 | `✓ 增补 3 第 1 件（代码三方第二轮第 1 条，形态 b）：挂载写出的写行与暖机里每次分配的分配代记成 txg − 1：row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked 红了` |
| 145 | `✓ 增补 3 第 1 件（代码三方第二轮第 2 条，几何补取样点）：随机历史快档也判出「复用时新记录罩住的已回收记录不删」（第 121、130 行同一处）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了` |

（141–145 行前缀的「✓」是我为表格统一加的，与 `.log` 原文一致；每一格里冒号前后与 `.log` 逐字相同，现查命令：`grep -n '^  ✓' /tmp/claude-1000/crash-m2-supp3-r2/59.log | tail -18`。）

## 指纹（每阶段开跑与结束各一次，crates + litmus）

| 时刻 | HEAD | diff_sha256 | untracked_sha256 |
|---|---|---|---|
| 54 开跑 2026-09-19T00:27:56Z | `58a5ebb49c8d502bf99cc54151210a54467f7cf6` | `d4c9043bc4af9d7d322570467e6b0ad2186d092c64eab9fc14ccce4118f41344` | `1690bbae28562cdda6daeeb805745ab925e9cdccb3d3693c89adcb707defc081` |
| 54 结束 2026-09-19T00:36:34Z | 同上 | 同上 | 同上 |
| 59 开跑 2026-09-19T00:38:28Z | 同上 | 同上 | 同上 |
| 59 结束 2026-09-19T00:42:29Z | 同上 | 同上 | 同上 |

四次指纹逐字相同：HEAD、`crates`+`litmus` 的已跟踪改动、未跟踪文件内容哈希，从 54 号开跑到 59 号结束全程没有变化。**这两个绿对应的是同一份源码**，两阶段之间、以及各阶段跑的过程中都没有人改过 `crates`/`litmus`。

程序上的一处偏差要说明：54 号的「开跑」指纹与起阶段的命令，我拆成了两条独立的 Bash 调用（先记指纹、再起后台），不是同一条命令里；59 号已改正，指纹与起阶段在同一条命令里一次性执行完。54 号这两条调用首尾相接、中间没有其它动作，且 54 号结束时的指纹与开跑时完全相同，可以证明这个间隙里源码确实没有变化，但形式上没有满足「同一条 Bash 命令」的要求，如实记在这里。

## 环境旁证（本轮不跑 55、57，仅按定义留证）

```
$ command -v herd7; echo exit=$?
exit=1
$ ls -l /dev/kvm
crw-rw---- 1 root kvm 10, 232 Sep 18 08:21 /dev/kvm
exit=0
$ command -v qemu-system-x86_64; echo exit=$?
/usr/bin/qemu-system-x86_64
exit=0
```

本机没有 `herd7`，`/dev/kvm` 与 `qemu-system-x86_64` 都在。这三条只是旁证，不据此对 55、57 号下判定——本轮按派发要求不跑这两道。

## 汇总表

| 阶段 | 退出码 | 耗时 | ✓ / ✗→ | 计数行 |
|---|---|---|---|---|
| 54（层 0 全量，两条流） | 0 | 8 分 38 秒（32 线程） | 三行 ✓ 全部原样抄见上，逐字比对全部为空差 | 见上「两条流的计数行」「CHECKER 行」两处 |
| 59（crates 变异表，145 行/140 条） | 0 | 4 分 1 秒 | `✓ crates 变异表复跑：140 条变异各自红在点名的测试上（原文都恰好命中一次）` | 报红 140、没红 0、没跑到 0 |
| 55（QEMU 真设备） | 没跑（不是门禁自己判的 77，是本轮按派发要求根本没执行这个脚本） | — | — | 按派发不跑 |
| 57（herd7/LKMM） | 没跑（同上，未执行） | — | — | 按派发不跑；本机也没装 herd7 |

## 结论

54 号、59 号两道全绿（退出码都是 0），层 0 全量重放的三行判定与两份参照产物逐字比对全部空差，crates 变异表 140 条全部报红、没有一条没红或没跑到。这两道的证据要求满足。

**推翻条件**：任何一处 `diff` 有输出，或者有人在 `crates`/`litmus` 未变的情况下复跑却得到不同计数，就要重新判；59 号任何一条从「红了」变成「没红」或「没跑到」，同样要重新判。

## 没做什么

- 没跑 55 号（QEMU 真设备）、57 号（herd7/LKMM）：按派发要求本轮不跑，只留了 `herd7`/`/dev/kvm`/`qemu-system-x86_64` 三条命令的原样输出当旁证，不据此对这两道下判定。
- 没修任何一处红（本轮两道都没红）；也没有判断这次的改动是不是该由这两道以外的门禁再看一遍——那归主 agent 或 `gate-triage`。
- 全绿只说明 54、59 两道自己的证据要求满足了；层 0 覆盖到哪几条流、哪些流没进这两道，要看 54 号脚本头部的 `gate-covers` 声明，不由本报告外推。
- 没有核对 `.claude/gate.d/54-layer0-replay.sh` 本身这次改动（多线程化）在设计/实现层面对不对——那是三方对抗与代码 review 的事，本轮只跑它、读它的判定输出。
- 54 号「开跑」指纹与起阶段命令没有写在同一条 Bash 调用里（见「指纹」一节的说明），59 号已按要求改正。
- 没有拿到 54 号两个测试二进制各自的完整 `test result:` 汇总行（cargo 原始输出只落在阶段脚本自己创建又删除的临时文件里，本阶段的 stdout 只透出 `LAYER0_PROGRESS` 行和最终 `✓`/`CHECKER` 行）；「每条流的用时」用的是 harness 自报的 `elapsed_seconds`（枚举阶段本身），不含各二进制里的编译时间与其它快用例，已在表里注明这一点。
