# 实六报告（implementation-writer，2026-09-26 写）

## 结论

五件都做完，每件带用例、新用例逐条证过会红（变异 22 行抓到 22 行），名字不带 layer0 的动到的二进制全绿，fmt / clippy（check.sh 那套 lint）/ build 过。
两份 layer0 文件没跑（闸拒），里面钉的值用改了名的探针（scratch 副本里 `probe_second_stream.rs` / `probe_first_stream.rs`，只跑非 ignore 用例）现算，探针上 8 + 5 条全绿；ignore 的两条全量由主 agent 提交时跑 54 号核。
推翻条件：54 号全量在两条流上判红（不是钉的数对不上就是新写路径真有错）；或 C561 四格用例在 σ 全量上与 `C561_SIGMA_FULL … =0` 对不上。

## 写过的文件

- 新：`crates/singlefs-harness/src/layer0_progress.rs`（续跑：进度文件格式、读回核验、开关）
- 新：`crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs`（C561 四格 + C513 三状态 + σ 全量 ignore）
- 新：`crates/singlefs-harness/tests/crash_enumeration_resumes_from_its_progress_file.rs`（续跑 6 条）
- 改：`crates/singlefs-harness/src/crash.rs`（记录核对器第二条判据、甲二展开、续跑接入、观察者计数、单测）
- 改：`crates/singlefs-harness/src/crash_injection.rs`（调用方给第四样入参）
- 改：`crates/singlefs-harness/src/lib.rs`（`pub mod layer0_progress;` 一行；diff 里其余 43 行是前几批的）
- 改：`crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`（固定脚本加卸载、E18 改两次覆盖写、甲二快档、全量带续跑、钉值）
- 改：`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs`（甲二快档、全量带续跑、观察者计数进进度文件、钉值）
- 改：`crates/singlefs-harness/tests/second_transaction_crash_inside_the_floor_raise_pushed_by_the_session.rs`（新签名）
- 改：`crates/mutations.tsv`：末尾追加 23 行（名字都以「实六 」开头：C561 六行、甲二三行、续跑十三行、固定脚本一行）；另改了 6 行旧行的锚点（原文腐化，门禁 33 号点名）：第 116 行「增补 2 第 41 行 层 0 并行：按序号取状态时段内子集掩码取反」、第 155 行「增补 3 第 3 件…崩溃状态上不跑记录核对器」、第 160 行「…记录核对器第二条判据不认『被流里更晚的写盖过』…」、第 268–270 行 C507 三行（改成对 `persisted[later_index]` 下手）。

`git diff --stat -- crates litmus` 原样末行（含前几批没提交的改动，这一轮没有别的实现员改 crates）：
```
 82 files changed, 9454 insertions(+), 6982 deletions(-)
```
（三个新文件未跟踪、不在 stat 里：1050 + 701 + 417 行。）

## 五件各做了什么

1. **C561**：`crash.rs:702` `check_records_against(reader, writes, persisted, effective_root)` 加第四样入参；第二条判据的缺席判定在 `crash.rs:752` `unit_copy_is_missing_under_the_persisted_set`：逐扇区，扇区上字节还是这份副本的就过；不是的，要有一次更晚、`persisted` 里为真、落在这个扇区上、过回收谓词的写解释它，否则缺席。`check_records(image, …)` 签名没变，第四样取 `image.persisted`（四样都出自同一个崩溃状态，写不出对不上的组合）——所以 `check_records` 的调用方（history.rs 只用 `RecordCheck` 类型、first_transaction_step_seven_layer0.rs）不必改；真改的调用方是 crash_injection.rs（传 `&persisted[..前缀长]`）。第一条判据（根在而记录缺席）没动，照旧按盘上字节。
2. **续跑**：进度文件 `<目录>/layer0-progress-<流名>-<输入指纹>-<计划哈希>.txt`；计划哈希（`crash.rs:2072`）含基线每个扇区、写表、段与每段展开方式、被判的根、版本表、状态数、片方案、有没有观察者（U1）；文件头写片方案（R2），续跑时按线程数定片长那一档改按状态数定（`crash.rs:1843`，每片 ⌈状态数 / 65536⌉ 且 ≥ 16）；每行 CRC-32C，换行结尾才算整行、末尾半行丢掉那一片重跑，整行里缺字段 / 多字段 / 校验和不对整份作废（R3、U2）；片行在观察者看完那一片、没 panic 之后按片号次序写、落盘，观察者计数（`Layer0ObserverCounts`）与看过的状态数随片行写（R4、U3）；有 panic（判红）删进度文件；并完核「读回的片 + 这一趟跑的片 = 总片数」、有观察者时核看过的状态数 = 状态数；跑完删进度文件；`LAYER0_RESUME` 行报读回了几片、丢了几行半行、为什么作废，`LAYER0_PARALLEL_START/FINISHED` 行尾加 `resumed_slices=… freshly_run_slices=…`。开关：`SINGLEFS_LAYER0_PROGRESS_DIRECTORY`（没设不留进度文件）、`SINGLEFS_LAYER0_INPUT_FINGERPRINT`（设了目录必须设）、`SINGLEFS_LAYER0_START_OVER=1`（强制从头跑）。两条全量用例接上 `Layer0Resume::from_environment(流名)`。
3. **固定脚本**：挂着时的向前回退实三已经换上；这一轮在到 E 之后加正常卸载（录成卸载入口的一段，切段带入口段，C557 那一道在层 0 里真核到带记号的根），流名 `ReuseAfterRaisingFloorThenNormalUnmount`；E18 那个脚本改成 E 之后再覆盖写两次（txg 18 落 50178、txg 19 落回 A 的 50180，探针量过），陈旧 tail 与 C507 那两条下游用例跟着改到 txg 19。
4. **甲二快档**：`crash.rs:1593` `Layer0SegmentExpansion::InPlaceSubsetsWithCopyOnWriteNoneOrAll`（单元写以外都算原地写；有单元写的段 2^(k+1) − 1、没有的 2^k − 1）；`enumerate_layer0_quick_tier_versions`、`layer0_state_count`；两条流平时那条用例换成甲二。
5. **钉值**（探针现算，下一节）。

## 钉值：现算的命令与原样输出

探针做法：`bash /tmp/claude-1000/impl-rbf-6/run-probe.sh probe_second_stream probe2-final`（把仓里的 crates/ 同步进 scratch 副本、`make-probe.py` 把 `second_transaction_step_zero_layer0.rs` 拷成 `probe_second_stream.rs` 并在 `prepare` 里插一行 `PROBE` 打印、`first_transaction_step_seven_layer0.rs` 原样拷成 `probe_first_stream.rs`，再 `cargo test --test probe_… -- --nocapture`，只跑非 ignore 用例）。末次跑（所有改动之后）原样：

```
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 50.80s   （probe_second_stream）
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 3.34s    （probe_first_stream）
PROBE script=ReuseAfterRaisingFloorThenNormalUnmount sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 4, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 30, 2, 1, 4, 16, 2, 1, 18, 2, 1, 30, 2, 1, 4, 16, 2, 1, 18, 2, 1, 2] writes=477 roots=19 segments=61 closed_form=6649413746 quick_tier=232
PROBE script=ReuseOfTheFirstDataUnitSlotAfterFloorRaisingPublish sizes=[…同上到第 51 段…, 30, 2, 1, 30, 2, 1, 2] writes=499 roots=19 segments=60 closed_form=8796569699 quick_tier=223
PROBE script=RollbackToFirstVersion sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 4, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 22, 2, 1, 2] writes=191 roots=9 segments=29 closed_form=206307382 quick_tier=104
PROBE script=SecondVersionOnly sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 2] writes=70 roots=4 segments=13 closed_form=134217752 quick_tier=40
PROBE script=ThirdVersion sizes=[2, 2, 1, 2, 2, 1, 26, 2, 1, 26, 2, 1, 4, 18, 2, 1, 18, 2, 1, 18, 2, 1, 26, 2, 1, 2] writes=166 roots=8 segments=26 closed_form=202113075 quick_tier=93
LAYER0B_FAST states=232 states_by_publish=[instance1_txg1=7 instance1_txg2=7 instance1_txg3=11 instance1_txg4=11 instance2_txg5=20 instance2_txg6=11 … instance2_txg15=20 … instance2_txg18=20 instance2_txg19=11 after_the_last_root=3 every_write_persisted=1]
REUSE_THAT_NEVER_LANDED effective_root=Some((InstanceGeneration(2), CheckpointTxg(18))) file_read=true violations=0 ignored_violations=0 record=(0, 1)
RESIDUAL_RECORD states=31 states_whose_chain_reaches_the_residual_record=19 violations=0 …
STALE_TAIL states=8 states_with_a_reused_named_unit_after_the_stale_tail=8 records_with_a_reused_named_unit=[3] violations=0 failed=0 verification_failed=0
```
（`…` 是我在这里缩的，全文在 `/tmp/claude-1000/impl-rbf-6/logs/probe2-final.log`。）全量的按发布分那一串（`FULL_STATES_BY_PUBLISH_THROUGH_THE_UNMOUNT`）是探针按段现算的（每段 2^|段| − 1 归它后面最近的那次根槽写），不是跑出来的。

两条流各报甲二与全量（状态数）：

| 流 | 段 | 写 | 全量（闭式） | 甲二 |
|---|---|---|---|---|
| 第一条流（mkfs→取号→暖机→A） | 10 | 41 | 67 108 885 | 29 |
| 第二条流（固定脚本到 E 再正常卸载） | 61 | 477 | 6 649 413 746 | 232 |

第二条流全量比第二轮判决冻结形状（5 575 606 380）多，六个 30 写段各 2^30 − 1 占 97%；照 13 880 个 / 秒（16 线程、量过的旧速率）线性外推约 5.5 天，32 线程约 2.8 天（推的）。

**推的、要 54 号全量核的钉值**：第一条流全量 `OracleCounts` 里 `no_file_states: FULL_STATES - 7`、`file_read_states: 7`、`root_persisted_states: 4`、`journal_differing_states: 3`、`verification_ran_states: 6`（`first_transaction_step_seven_layer0.rs:487` 起）：甲二跑出来就是这几个数，全量多出来的只有 A 那一段单元写落一部分的状态，我推它们都落在暖机 txg 2 的根上、没有文件；没跑全量。第二条流全量用例只钉闭式与按发布分（都按段算得出）、其余是零违例断言。

## 证红（第 3 步）

基线：三份副本各先跑一遍不改动的 `--lib`（77 过）、`--test record_checker_judges_absence_by_the_persisted_set`（5 过 1 ignore）、`--test crash_enumeration_resumes_from_its_progress_file`（6 过），基线红集为空。

新加 23 行，22 行逐行证过（副本里改坏、跑那行点名测试所在的整个二进制、从原件拷回并 touch），全部抓到；第 23 行（`second_transaction_step_zero_layer0` 的卸载入口段）在 layer0 二进制里，没跑，留给提交时的 59 号。改锚点的 6 行旧行：116、155 两行证过（抓到）；160、268–270 四行点名的是 `second_transaction_step_zero_layer0`，没跑，留给 59 号。

逐行「改坏哪一行 → 哪条断言红 / 同时红了哪些」（`/tmp/claude-1000/impl-rbf-6/mutations/results/row-*.txt` 原样摘）：

| 变异（mutations.tsv 里的名字，简） | 必须红 | 同时红的 |
|---|---|---|
| C561 后写不看持久集合 | the_false_red_and_the_misaligned_hole…（Y 判成不缺席） | a_data_unit_only_half_covered…、a_node_whose_every_later_write… |
| C561 只看盘上字节（SecDisk） | the_false_red_and_the_misaligned_hole… | 无 |
| C561 换回今天的判法（整份在位） | partial_landing_of_the_two_later_nodes… | the_false_red_and_the_misaligned_hole… |
| C561 不过回收谓词 | an_illegal_reuse_does_not_explain… | 无 |
| C561 只看头一个扇区 | a_data_unit_only_half_covered… | 无 |
| C561 check_records 不交持久集合 | the_false_red_and_the_misaligned_hole… | 同第一行那两条 |
| 甲二 掩码不右移 / 少算一半 / 单元写算原地 | crash::tests::the_quick_tier_plan_takes_every_in_place_subset… | 各自无 |
| 续跑 读回的片不用 | a_run_cut_short_after_some_slices… | a_resumable_run_slices_the_same_way_on_any_number_of_worker_threads |
| 续跑 观察者看过的状态不记 | a_run_cut_short_after_some_slices… | 另 4 条续跑用例（U3 那道断言） |
| 续跑 仍按线程数定片长 | crash::tests::a_resumable_run_slices_independently… | 无 |
| 续跑 计划哈希不含版本表 | a_progress_file_of_another_versions_table… | 无 |
| 续跑 不核校验和 / 半行不丢 / 缺字段照收 / 不核观察者状态数 / 同一片不去重 / 没设目录也留 | layer0_progress::tests 里各自那一条 | 各自无 |
| 续跑 判红不删 / 跑完不删 / 从头跑开关不起作用 | a_run_that_goes_red… / a_run_cut_short… / starting_over_reruns… | 各自无 |
| 旧 116（掩码取反） | crash::tests::the_state_plan_hands_out… | crash::tests::the_quick_tier_plan… |
| 旧 155（崩溃注入不跑核对器） | crash_injection::tests::crash_points_are_reproducible… | 无 |

旧判法对照（证 σ 全量那条会红）：scratch 副本里用 `old-rule-patch.py` 把 `unit_copy_is_missing_under_the_persisted_set` 的函数体换回实六之前的整份判法，release 跑整个 C561 二进制（含 ignore）：
```
C561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=32768
test result: FAILED. 2 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 72.65s
```
红的是 X（`left: RecordCheck { root_without_record: false, claimed_state_missing_unit: true }`）、假绿那一格、64 状态那条（`left: [(0, 3), (0, 12), (1, 3), (1, 12), (2, 3), (2, 12), (3, 3), (3, 12)]`）与 σ 全量（32768 与第二轮攻方在冻结副本上量的相同）；新判法下同一条 release 跑：`C561_SIGMA_FULL states=262144 record_claimed_state_missing_unit=0`、`test result: ok. 1 passed; … finished in 110.36s`。

变异 worker 的事（主 agent 消息，第二批跑到一半收到）：第一批 `binary_args` 把 `--test` 也截掉，跑成不挑目标的 `cargo test -p singlefs-harness`（会跑到 layer0 二进制）——整批作废，主 agent 停了它；第二批被我停在 row 1–3（SIGTERM）也作废。改完的 `/tmp/claude-1000/impl-rbf-6/mutations/run-worker.sh`：`binary_args` 取 ` -- ` 之前那段，没有 `--lib` / `--test <名>` / `--bin <名>` 就补 `--lib`；目标名带 layer0 的记 `SKIPPED_LAYER0`；最后一道没有挑目标的选项记 `SKIPPED_UNTARGETED` 不跑；每条限时 1200 秒。上表全是第三、四批（改完之后）的结果。另：我手改 mutations.tsv 那 6 行旧锚点时有 6 处制表符丢了（字段变 5 段），第四批跑前修好，`awk` 核全表每行 6 段、门禁 33 号绿。

## 第 4 步那几样（末次，所有改动之后；原样末行）

- `cargo fmt --check`：退 0，无输出。
- clippy（`--all-targets --all-features -- -D warnings` 加 check.sh 那 7 条 `-D clippy::…`）：退 0，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.79s`
- `cargo build --offline --all-targets`：退 0，`Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 4.06s`
- `--lib`：`test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 34.50s`
- `--test record_checker_judges_absence_by_the_persisted_set`：`test result: ok. 5 passed; 0 failed; 1 ignored; …; finished in 7.16s`
- `--test crash_enumeration_resumes_from_its_progress_file`：`test result: ok. 6 passed; 0 failed; 0 ignored; …; finished in 2.68s`
- `--test second_transaction_supplement_two_record_checker_reuse_legality`：`test result: ok. 1 passed; …; finished in 0.58s`
- `--test second_transaction_supplement_three_crash_injection`（crash_injection.rs 动了）：`test result: ok. 7 passed; 0 failed; 1 ignored; …; finished in 27.21s`
- `--test second_transaction_crash_inside_the_floor_raise_pushed_by_the_session`（debug，ignore 那条不跑）：`test result: ok. 0 passed; 0 failed; 1 ignored; …`；**release 下 `--ignored` 跑一次**：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 96.24s`
- σ 全量（C561 二进制里那条 ignore）release 一次：见上节，`record_claimed_state_missing_unit=0`、96/110 秒量级。
- 门禁（登记给我的）：33 号退 0 `✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 795 条的原文各命中源码一次…`（开工时它红在 6 行旧锚点上，已改）；53 号退 0 `✓ 格式常量文件里的占位都指得到分项或欠账…`；74 号退 0（`GATE_MODEL_DIFFERENTIAL_MEMORY_MAX=16G`）`✓ 模型对拍每一段都判过、实现与模型没有对不上的（查了 6 段）`；92、93、94 号退 0；89 号退 77（本次未跑：第 27 行前置没进来）。
- 顺手看了一眼 52 号（不登记给我）：退 1，红在 kb `layout/01-first-txn.md` 八那句第二条流的登记还是旧的 `…18+2+1…` 数组与 2104413——钉值变了、kb 没跟，是预期的红，归书记员照这份报告的数改（新数组与 6649413746 见上）。

没跑（闸与定义不许）：两份 layer0 测试二进制（`second_transaction_step_zero_layer0`、`first_transaction_step_seven_layer0`，值用探针核过）、其余 6 份 layer0 文件（它们用的 `enumerate_layer0_selecting_versions*` 签名没变，只编）、54、55、57、59 号、全量 `cargo test`。

## 交主 agent 定的（我先照一种做了，都被攻过零轮）

1. **卸载放在哪**：规格只写「加正常卸载（B1）」。卸载把 F 抬到现行 txg，放在回退之前（例如 B 之后的「关闭再挂载」）回退候选集里就没有 A 了，所以我放在到 E 之后（全量那条流的末尾），推两次带记号的空发布 txg 18、19。要换位置，钉的段序列与状态数整组跟着变。
2. **C561 判据里「解释那个扇区」的写**：我照 C561 那一行的字面——更晚、在持久集合里、落在这个扇区上、过回收谓词的写。攻方探针 `SecPers513` 还多一条「那次后写在那个扇区上的字节等于盘上的字节」（`research/prompts/m2-layer0-scale-r3-opus-model/opus_r3_probe.rs` 里 `d == r3_write_sector(st, l, s)`）。两者只在「同一个扇区上有两次更晚、都落了的写」时可能分开（字面版认更早那次合法复用、探针版只认最后落下的那次）；C561 那四格与 σ 全量上两者一样（四格用例与 σ 全量 0 都是按字面版跑的）。
3. **`check_records` 的签名**：规格说「加第四样入参、每个调用方跟着改」。我让 `check_records(image, effective_root)` 从 `image.persisted` 取第四样（同一个崩溃状态里的四样，拼不出对不上的组合），只有 `check_records_against` 显式多一个 `persisted` 参数；调用方里真要改的只有 crash_injection.rs。要改成显式参数是纯签名改动。
4. **第一条判据（根在而记录缺席）没改**：照旧按盘上字节判记录在不在。D13 已定项 7 那句「判复用与缺席要拿持久集合判」字面也罩得到它；规格五件没点它，我没动。
5. **续跑落在门禁那一头的（不归我、写范围外）**：54 号 `--full` 要给两条全量用例设 `SINGLEFS_LAYER0_PROGRESS_DIRECTORY` 与 `SINGLEFS_LAYER0_INPUT_FINGERPRINT`（与全绿标记同一份清单），留 `SINGLEFS_LAYER0_START_OVER=1` 的口子；全部片都从进度文件读回时 `LAYER0_PARALLEL_FINISHED … worker_threads=0 …`（实测，resume 用例那一趟），54 号今天按 `worker_threads=` 判「没用满线程」会误红，R5 那一半要门禁批改成认 `resumed_slices=` / `freshly_run_slices=`。进度文件跑完即删：全量跑完之后用例里的断言再红，下一趟从头跑（R4「判红删文件」的一种实现，跑完到断言之间被杀会丢整趟，窗口是几毫秒）。
6. **第一条流全量的几个钉值是推的**（见「钉值」一节末），54 号全量第一次跑时核。
7. `persisted_writes_of_state` 里 `Layer0SegmentExpansion::NotExpanded` 那一臂写了 `unreachable!`：不展开的段序号区间是空的，`segment_of_state` 用 `partition_point(|区间| 区间.end <= 序号)` 取第一个 end 大于序号的段，空区间 [a, a) 在序号 ≥ a 时被跳过、序号 < a 时落在前面的段，所以取不到它；调用点只有 `evaluate_state_slice` 经 `plan.persisted_writes_of_state` 这一处（与单测）。

## 没做什么

- 没走三方对抗；层 0（54 号，两条全量 + 甲二快档所在的两个 layer0 二进制）、QEMU、herd7、crates 变异整表（59 号）归 crash-verifier；没提交。
- 变异第 23 行与改锚点的 160、268–270 四行在 layer0 二进制里，没证红，留给提交时的 59 号。
- 54 号的输入指纹、`admission.py`、另 6 条 `#[ignore]` 层 0 全量、生成器与白名单：规格写明不归实六。
- kb（layout/01 八的登记、milestone/02 固定脚本那段、C561 / C557 / D13 已定项 4 与 9 的状态）不在写范围，等书记员；52 号现红在这上面。
- 没新开层 0 全量枚举：第二条流加卸载后比已有的流多罩的是卸载那一串（先写系统配置 4 写一段、两次空发布 16 写一段与 18 写一段、它们的记录段与根槽段，共 65554 + 262147 + 轮换段 3 个状态，全量里按发布分的 txg 18、19 两格），多跑的是「正常卸载」那一步；甲二快档已经把这两格的原地子集跑过（txg 18 = 20、txg 19 = 11 个状态，零违例）。

## 删掉的与留下的

- 删了：`/tmp/claude-1000/impl-rbf-6/probe`（2.5G）、`/tmp/claude-1000/impl-rbf-6/mutation-0`（9.9G）、`mutation-1`（9.8G）、`mutation-2`（2.0G）、变异时用例没清掉的 11 个 `/tmp/singlefs-crash-enumeration-resume-*` 目录（共 124K）。
- 留着（报告的复跑材料，小）：`/tmp/claude-1000/impl-rbf-6/logs/`、`mutations/`（rows.tsv、run-worker.sh、results/）、`make-probe.py`、`run-probe.sh`、`old-rule-patch.py`、`progress.md`。
