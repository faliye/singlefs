# m2-supp3-item2 代码三方第二轮改法：实现员报告（接手第二会话）

时刻：接手 2026-09-19 22:18 UTC（东京 2026-09-20 07:18），收尾 22:35 UTC。前一个实现员（被会话限额打断，交接摘要 `handover.md`）把 `crates/` 的改动与变异证明都跑完了，它写到一半的报告我没续、原样留在同目录 `report-session-1-interrupted.md`（它没交回过）。这一份是重新核过现场之后写的，凡是标「本会话」的输出都是 2026-09-19 22:18–22:35 UTC 这一次真跑出来的；标「前一会话」的是 `logs/` 里 17:52–18:50 UTC 那一批日志（源码没变，见第一节）。
判决：`research/prompts/m2-supp3-item2-code-r2-main-verification.md` 第三节第 1–5 条（第 39–43 行）。74 号的新段归主 agent，没碰 `.claude/`。

## 一、现场核查（接手第一步）

1. **没有留着没还原的变异**。12 条变异定义（`proof-mutants/*/`）逐条拿 `old` 原文在主工作区的对应文件里数：**每条都恰好命中 1 次**（命中 1 次 = 那一处是原样，没被替换）。本会话原样输出：

```
continuation-stops-on-ring-turn-form	crates/singlefs-harness/src/history.rs:2746-2748	if !continues_past_the_ring_turn_form {
glue-tt0-as-below-floor	crates/singlefs-harness/src/model_comparison.rs:220-222	MountError::RollbackToVersionWithoutFileUnsupported(_) => {
m1c	crates/singlefs-core/src/transaction.rs:1329-1329	records_before_this_publish + rewritten.len() * allocator.devices.len();
m1d	crates/singlefs-core/src/transaction.rs:1329-1329	records_before_this_publish + rewritten.len() * allocator.devices.len();
m2h	crates/singlefs-core/src/allocator.rs:742-744	DeviceAgreement::Agreed(slot) => slot,
m2i	crates/singlefs-core/src/allocator.rs:746-751	DeviceAgreement::SomeDevicesWithoutAnswer(full_devices) => {
m2j	crates/singlefs-core/src/allocator.rs:801-803	DeviceAgreement::Agreed(answer) => answer,
m4a	crates/singlefs-core/src/mount.rs:292-292	oldest_valid_root.map_or(effective_floor, |oldest| effective_floor.max(oldest))
m4b	crates/singlefs-core/src/transaction.rs:1749-1749	device_map.allocated_slots() * SLOT_BYTES,
tt0-A-reported-as-below-floor	crates/singlefs-core/src/mount.rs:1240-1240	return Err(MountError::RollbackToVersionWithoutFileUnsupported(target));
tt0-B-below-floor-reported-as-tt0	crates/singlefs-core/src/mount.rs:1221-1224	return Err(MountError::RollbackTargetNotACandidate {
w1-row156	crates/singlefs-core/src/transaction.rs:1334-1334	if records_after_this_publish > allocation_node_capacity {
```

`find crates litmus -name '*.pristine' -o -name '*.orig' -o -name '*.rej'` 零命中（还原用的备份没留在仓里）。

2. **与证明开跑时的快照比**（`logs/source-at-proof-start.sha256`，59 个文件，17:52 UTC 取）：现在有 5 个文件的 sha256 不同——`history.rs`、`second_transaction_step_one_overwrite.rs`、`second_transaction_supplement_three_random_history.rs`、`mutations.tsv` 四个是前一会话 17:53–18:37 自己接着改的（证明做法见第三节），`crash.rs` **不是这一轮的**：`git diff` 只有一行文档注释（`/// 记录核对器（D13（验证路线） 故意不给编号；…` → `…（D13（验证路线） 已定项 7，故意不给它编号；…`），mtime 21:39 UTC，是别的会话在改；我没碰它。

3. **`nice -n 19 cargo test --release --workspace`**（本会话 22:20:18–22:20:48 UTC，`logs/workspace-release-test.log`）：`exit 0`，`Exit status: 0`，`Elapsed (wall clock) time (h:mm:ss or m:ss): 0:29.92`，`grep -c '^test result: ok'` = 36、`grep -c 'FAILED\|panicked'` = 0。全绿。

4. **证明日志按变异名点数**：`logs/proof/` 里 26 份，12 条变异 × 它点名的每个测试二进制 + 5 份基线 + 2 份补跑（`base2--below-floor.log`、`tt0-B-rerun--random-history.log`），`crates/mutations.tsv` 新增的 14 行逐行对得上一份日志（第四节的表）。**没有缺的，一条都没重跑。**

5. 开跑前 `ps -o pid,args -u "$(id -u)"`：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`；有别的会话的 `gate.sh`（pid 2844631 `--staged`、2844679 在 `/tmp/tmp.Own0rjzzQo/tree`、2845580 `selftest.sh`）在跑，照共用约束加 `nice -n 19` 照常跑，cargo 没报等锁（本会话的计时因此比前一会话高，见第六节）。

## 二、判决第三节五条逐条（行号本会话现查）

1. **「树表 0 条」单列**（判决第 39 行）：`crates/singlefs-core/src/mount.rs` 第 56 行 `RollbackToVersionWithoutFileUnsupported(RollbackTarget)` 单列回 `MountError`；`RollbackCandidateExclusion`（第 128 行）回到三个值 `NotInRing`（130）、`BelowEffectiveFloor`（132）、`OnAbandonedTimeline`（135），第 126 行的文档写明树表 0 条不在这里；第 1240 行在任何写之前 `return Err(...WithoutFileUnsupported(target))`。胶水 `crates/singlefs-harness/src/model_comparison.rs` 第 220–222 行映射到 `ModelRefusalReason::RollbackToVersionWithoutFileUnsupported`（模型「第一版不支持」那一类），第 260 行进上限表；`crates/singlefs-harness/src/history.rs` 第 1758–1759 行成员名。三个 `match` 都穷举，`grep -n '_ =>' mount.rs model_comparison.rs history.rs` 只剩 `history.rs:592`——那是 `match source.below(5)` 对整数取值的兜底（第 585–595 行），不是枚举，且不在这一轮的 diff 里（`git diff -U0` 的 hunk 是 `@@ -406,0 +574,9 @@`，585 起是原有代码）。变异 tt0-A / tt0-B / 胶水见第四节。
2. **小盘段**（判决第 40 行）：盘宽做成取样点参数 `HistoryDeviceWidth`（`history.rs` 第 64 行，`FourGibibytes` 66 / `UnitAreaOf384Slots` 71；单元区 384 槽 = 六个 64 槽聚簇段，第 74 行），跑法收成 `HistoryExecution { per_step_checker, device_width }`（第 2580 行）；新比重 `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`（第 490–494 行）；新段 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（测试文件第 344 行，断言：没有新发现 364、用户数据那一处被「每块盘上都没有」拒过 ≥ 1 在 374、模型在单元区墙区间里放行过 ≥ 1 在 381、checker 跑过 > 0 在 386）。m2h、m2j 这一段红；**m2i 这一段红不了**（第七节第 1 条，等大盘走不到那一臂），它由不等盘那条旧用例红。74 号的新段交接见第五节。
3. **第四段照跑 checker**（判决第 41 行）：`PerStepChecker::RunContinuingPastTheRingTurnForm`（`history.rs` 第 2545 枚举、2552 成员、2563 报告串、2746–2749 只记不停那一处），计数 `ring_turn_form_steps_noted` / `histories_with_the_ring_turn_form_noted`（第 1370–1371 行，报告行第 1609 行）。第四段改名 `allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count`（测试文件第 300 行），多断言 checker 真跑过（第 325 行）。另补单元测试 `allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records`（`crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs` 第 585 行：从第一个文件起连着覆盖写 49 次到装不下，逐版每盘核「已分配」= 这一版分配记录跨度之和 × 16 KiB，过 600 条的版本 13 个，第 623–627 行把 49 与 13 钉死）。耗时第六节。
4. **抬 F、回退各一条逼近墙的定向用例**（判决第 42 行）：`raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds`（测试文件第 549 行）、`rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds`（第 600 行），两条都跑 `UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES`（第 49–52 行）、断言 `run.ending == Completed`（568、625）。m1c、m1d 各钉一条。
5. **命名**（判决第 43 行）：`crates/singlefs-harness/src/model.rs` 第 1921 行 `raised_floor_carried_by_one_device_only_does_not_take_effect`、第 1936 行 `carried_unit_keeps_the_generation_of_the_publish_that_wrote_it`；`crates/mutations.tsv` 第 153、154 行点名的测试名跟着改。本会话 `bash .claude/singlefs-ai-sop/scripts/naming-lint.sh` 原样末行：`  ✓ 命名纪律通过：查了 195 个 .rs 文件、38425 个声明的名字`，退出码 0。

**五条都在，没有要补的。**（判决第 44 行第 6 条本来就不交实现员，归欠账表。）

## 三、新测试与「改坏哪一行 → 哪条断言红」

做法（前一会话做、本会话核）：`rsync -a --exclude target --exclude .git` 拷仓到 `copies/proof`（自己的 `CARGO_TARGET_DIR`，debug），先跑不改动的基线，再逐条施加、跑点名测试所在的**整个测试二进制**、还原（从 `.pristine` 拷回再摸 mtime，`scripts/mutate.py`）。脚本 `scripts/run-proofs.sh`，进度 `logs/proof-progress.txt`（末行 `18:30:20 全部跑完`），每次跑的整份输出 `logs/proof/<变异>--<二进制>.log`。**基线红集为空**：随机历史、步 4、步 1、不等盘、harness lib 五个二进制基线都 `exit 0`（`logs/proof/base--*.log`，`proof-progress.txt` 第 2–6 行）。红点都落在测试断言行上，没有 `debug_assert` 先红的（debug 下跑；release 下没另跑变异，见第八节）。

新测试一览（行号现查）：

| 测试 | 位置 |
|---|---|
| `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device` | `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:344` |
| `allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count`（第四段改跑 checker、改名） | 同文件 `:300` |
| `raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds` | 同文件 `:549` |
| `rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds` | 同文件 `:600` |
| `rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor`（判决没点名，为 tt0-B 余量补的） | 同文件 `:702` |
| `allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records` | `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs:585` |
| `model_comparison::tests::each_rollback_candidate_exclusion_and_the_version_without_file_map_to_their_own_reasons`（旧名 `each_rollback_candidate_exclusion_maps_to_its_own_reason`） | `crates/singlefs-harness/src/model_comparison.rs:380` |

改过断言的旧测试：`second_transaction_step_four_rollback.rs:99`（断言换成新成员 `MountError::RollbackToVersionWithoutFileUnsupported`，第 99–104 行）。

随机历史测试文件里下表引用的断言行（现查）：五段「没有新发现」的条件依次在第 214（快档）、243（复用）、280（回退）、321（第四段）、364（小盘段）行；第四段「墙按真条数放行过」第 332 行；小盘段「被『每块盘上都没有』拒过」第 374 行；四条写死用例的 `run.ending == Completed` 在第 487（812 条）、568（抬 F）、625（回退）、727（F 之下的回退）行。步 1 那条单元测试的断言第 610 行；步 4 第 99 行；胶水那条 lib 测试第 397 行。

## 四、新变异逐条：`crates/mutations.tsv` 第 165–178 行（14 行）与第 156 行复核

「段数」= 那一段里以新发现收尾的历史段数（`历史 N 段：… 新发现 X` 那一行，本会话从 `logs/proof/*.log` 里现取）；「红的测试」照 `logs/proof-progress.txt` 那一行。

| 行 | 变异 | 改坏哪一行 | 红在哪 |
|---|---|---|---|
| 165 | tt0-A：树表 0 条报成候选排除「低于 F」 | `mount.rs:1240` → `RollbackTargetNotACandidate { exclusion: BelowEffectiveFloor }` | 随机历史五段全红（新发现：快档 50/96、复用 3/48、回退 46/48、第四段 2/32、小盘段 1/32）；点名 `random_histories_fast_tier_…` |
| 166 | 同一处（点名步 4 的写死用例） | 同上 | 步 4 二进制 `rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write` 红在 `second_transaction_step_four_rollback.rs:99`（实际报 `BelowEffectiveFloor`） |
| 167 | tt0-B：候选排除「低于 F」报成「树表 0 条」 | `mount.rs:1221–1224` → 报 `RollbackToVersionWithoutFileUnsupported` | 随机历史五段全红，**各只 1 段**（快档 1/96、复用 1/48、回退 1/48、第四段 1/32、小盘段 1/32）；点名快档 |
| 168 | 胶水把「树表 0 条」映射成「低于 F」 | `model_comparison.rs:220–222` | lib 测试 `each_rollback_candidate_exclusion_and_the_version_without_file_map_to_their_own_reasons` 红在 `model_comparison.rs:397`；随机历史五段同 tt0-A（50 / 3 / 46 / 2 / 1） |
| 169 | m2h：用户数据那一处「每块盘上都没有」报成「小盘写满」 | `allocator.rs:742–744` | **只红小盘段**：新发现 8/32（第 364 行「没有新发现」）；前四段 0 |
| 170 | m2i：反向（「小盘写满」报成「每块盘上都没有」） | `allocator.rs:746–751` | 随机历史**五段都不红**（等大盘走不到那一臂，第七节第 1 条）；不等盘二进制 `filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking` 红 |
| 171 | m2j：提交内生块那一处同 m2h | `allocator.rs:801–803` | **只红小盘段**：新发现 21/32 |
| 172 | m4a：回收门槛取「最旧有效根 + 1」 | `mount.rs:292` | 第四段 15/32、小盘段 15/32（checker 判 I-3.1 记账少于遍历）；前三段 0 |
| 173 | m4b：分配记录过 600 条「已分配」少一槽 | `transaction.rs:1749` | 第四段 32/32、小盘段 30/32 |
| 174 | m4b 同一处（点名步 1 的单元测试） | 同上 | 步 1 二进制 `allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records` 红在 `second_transaction_step_one_overwrite.rs:610` |
| 175 | 「只记不停」退回成判红就停 | `history.rs:2746–2748` `if !continues…` → 恒真 | 第四段与小盘段都红：新发现 0 但 32 段全以已知红第 0 条收尾（`跑完 0、以已知红收尾 {0: 32}`），断言「墙按真条数放行过 ≥ 1」（第 332 行）与小盘段第 374 行红 |
| 176 | m1c：只在抬 F 的调用栈里准入多算一个角色 | `transaction.rs:1329` | 抬 F 定向用例红在测试文件第 568 行；第四段另红 1/32 |
| 177 | m1d：只在回退的调用栈里多算 | `transaction.rs:1329` | 回退定向用例红在第 625 行；第四段另红 1/32 |
| 178 | tt0-B 同一处（点名 F 之下的写死用例） | `mount.rs:1221–1224` | `rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor` 红在第 727 行；先在同一份副本上单跑这条是绿的（`logs/proof/base2--below-floor.log`：`test result: ok. 1 passed`），施加后整二进制复跑 `test result: FAILED. 11 passed; 6 failed; 2 ignored`，6 条红里含它（`logs/proof/tt0-B-rerun--random-history.log`） |
| 156（旧行复核） | W1：`>` 写成 `>=` | `transaction.rs:1334` | 812 条用例（第 487 行）、抬 F（568）、回退（625）、第四段 9/32；改跑 checker 之后与第一轮的 9 段相同 |

`crates/mutations.tsv` 本会话数出来 178 行（含 3 行表头注释）；门禁 33 号数的是 173 条变异，见第五节。

## 五、门禁与测试的原样输出（都是本会话跑的）

1. `nice -n 19 cargo test --release --workspace`（22:20:18–22:20:48 UTC）末尾原样：

```
	Command being timed: "nice -n 19 cargo test --release --workspace"
	User time (seconds): 548.74
	System time (seconds): 24.40
	Percent of CPU this job got: 1915%
	Elapsed (wall clock) time (h:mm:ss or m:ss): 0:29.92
	Exit status: 0
exit 0
```

（上面的 `/usr/bin/time -v` 原样连行，中间的内存与页错误行略去没贴；整份 `logs/workspace-release-test.log`。36 个 `test result: ok`、0 个 FAILED。）

2. `nice -n 19 bash .claude/scripts/check.sh`（22:22:06–22:26:18 UTC，`logs/check-handover.log`）末尾原样：

```
   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
```

四段都 ✓（第 4 行 `  ✓ 格式通过`、第 9 行 `  ✓ clippy 通过`、第 14 行 `  ✓ 构建通过`、第 795 行 `  ✓ 单测通过`）；`Elapsed (wall clock) time (h:mm:ss or m:ss): 4:11.91`、`Exit status: 0`、`exit 0`；36 个 `test result: ok`、0 个 FAILED。

3. 阶段归属表登记给 `implementation-writer` 的三个阶段（`awk -F'\t' -v me=implementation-writer … .claude/gate.d/stage-owners.tsv` 列出 33、53、74），逐个原样末行：

| 阶段 | 原样末行 | 退出码 |
|---|---|---|
| `33-mutation-tables.sh` | `  ✓ 142 个实验二进制都有成形的变异表，1509 条变异的原文各命中源码一次；crates/mutations.tsv 173 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上）` | 0（0 秒） |
| `53-format-const-placeholders.sh` | `  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））` | 0（0 秒） |
| `74-model-differential.sh` | `      随机历史：逼近分配记录墙的取样点：模型对拍 4001 步：该拒而拒 504、区间里拒 335、该成而成 3162；比过根 4017 条、分配记录 71998 条、冷启动内容 62 次、抬 F 上限 171 次；回退到 txg = F_生效 > 0 的根做成 0 次；分配记录墙按镜像上的真条数放行 326 次；单元区墙按区间放行 0 次` | 0（31 秒） |

4. `nice -n 19 bash .claude/singlefs-ai-sop/scripts/naming-lint.sh`：退出码 0，原样末行 `  ✓ 命名纪律通过：查了 195 个 .rs 文件、38425 个声明的名字`。

## 六、第四段跑 checker 之后的耗时

两批实测（同一份源码；`--nocapture` 单跑一段、同一二进制里别的测试滤掉，段内 16 线程）：**本会话** 22:26–22:35 UTC（有别的会话的 `gate.sh` / `selftest.sh` 同时在跑，抢核）与**前一会话** 18:42–18:45 UTC（机器空）。

| 段 | debug 墙钟（本会话 / 前一会话） | release 墙钟（本会话 / 前一会话） |
|---|---|---|
| 第四段（跑 checker，已知红第 0 条只记不停） | 128.2 s / 103.1 s | 20.5 s / 11.1 s |
| 小盘段（新） | 222.1 s / 118.5 s | 20.0 s / 11.6 s |
| 快档（对照，这一轮没改跑法） | 49.6 s / 24.7 s | 6.0 s / 3.2 s |
| 第四段改之前（不跑 checker，`copies/before` 上量） | — / 15.4 s | — / 4.9 s |

结论按机器空的那一批读：release 下第四段 32 段 11.1 秒、每段约 0.35 秒墙钟（改之前 4.9 秒 / 0.15 秒），跑 checker 让这一段贵 2.3 倍，与攻方量的「0.55 对 0.25」同一个倍数；debug 下 15.4 → 103.1 秒，贵 6.7 倍。本会话这一批每一项都被同时在跑的 `gate.sh` 抬高 1.2–1.9 倍，**倍数关系不变**。整道 `check.sh` 本会话 4:11.91（前一会话空机 3:41.62；第一轮改之前是 1:20.78–1:23.32）——多出来的几乎全在第四段与小盘段，debug 下 checker 每一步都走一遍全部有效根。要不要为 `check.sh` 缩这两段（种子数、步数）或只在 release 下跑，是门禁时长的取舍，我没动，交主 agent（改之前要重量判别力：小盘段 m2h 8/32、m2j 21/32；第四段 m4a 15/32、m4b 32/32）。

## 七、交主 agent：门禁 74 号的新段（74 号本身没碰）

- **测试**：`unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`，在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:344`，就在 74 号已经在跑的那个测试二进制里。
- **跑法**：种子 [0, 32)（常量 `UNIT_AREA_WALL_SAMPLING_FIRST_SEED` = 0 / `_SEEDS` = 32，同文件第 43–44 行）、每段 150 步（第 45 行）、比重 `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`、`PerStepChecker::RunContinuingPastTheRingTurnForm`、`HistoryDeviceWidth::UnitAreaOf384Slots`、线程数 min(核数, 16)。
- **标题行**（`print_uncaptured` 直接写标准输出，与另四段同一格式）：`── 随机历史：小盘上逼近单元区墙的取样点 ──`；紧接着一行原样：`种子 [0, 32)，每段 150 步，比重：逼近单元区墙（小盘上落点拒绝那一格的取样点）；每一步之后跑池级 checker，已知红第 0 条那一形只记不停；两块单元区 384 槽的小盘（journal 环 128 MiB）`。
- **74 号要读的那一行**（本会话 release 下现取，格式与另四段相同；这一轮每段这一行末尾都多了一个字段「单元区墙按区间放行 N 次」，74 号现在那条 `sed` 只取「模型对拍 (\d+) 步」，不受影响）：

```
  模型对拍 4749 步：该拒而拒 649、区间里拒 325、该成而成 3775；比过根 4400 条、分配记录 78450 条、冷启动内容 52 次、抬 F 上限 165 次；回退到 txg = F_生效 > 0 的根做成 0 次；分配记录墙按镜像上的真条数放行 0 次；单元区墙按区间放行 312 次
```

- **预期计数**（本会话 release 与前一会话 debug 两次逐字相同，确定性来自同一批种子、同一份镜像字节）：`历史 32 段：跑完 32、以已知红收尾 {}、新发现 0；根环转过一圈的 32 段；最高 txg 174`；`Err 成员 PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)：304 次`、`Err 成员 MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))：8 次`；`checker 跑了 3725 次；一个写都没发、沿用上一次结论的 1107 步；已知红第 0 条那一形只记不停 2542 步（32 段历史）`。测试自己断言的四条：没有新发现、`PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)` ≥ 1、「单元区墙按区间放行」≥ 1、`checker_runs > 0`。
- 74 号若要多判一格：这一段的「单元区墙按区间放行」> 0（为 0 就说明落点拒绝那一格没走到，等于第一轮 4 GiB 盘上「判别子观测不到」那种状态）。
- **判别力**：m2h 8/32、m2j 21/32（`crates/mutations.tsv` 第 169、171 行）；前一会话在 release、[0, 128) × 150 步上量过基线 0/128、m2h 45/128、m2j 81/128（`logs/probe-unit-area-round1.out`）。
- **第四段**标题没变（`── 随机历史：逼近分配记录墙的取样点 ──`），第一行现在是 `…；每一步之后跑池级 checker，已知红第 0 条那一形只记不停；两块 4 GiB 的盘`，本会话计数：`模型对拍 4001 步：…；分配记录墙按镜像上的真条数放行 326 次；单元区墙按区间放行 0 次`、`checker 跑了 3103 次；…；已知红第 0 条那一形只记不停 2032 步（32 段历史）`、`历史 32 段：跑完 32、以已知红收尾 {}、新发现 0`。74 号脚本头注释里「这一段不跑池级 checker」那句要改。

## 八、停下交主 agent 的

1. **m2i 在小盘段红不了，不是余量问题**（判决第 40 行要求「m2h、m2i、m2j……这一段要红」）：m2i 换的是「有的盘答不出、有的盘答得出」那一臂（`allocator.rs:746–751`），两块等大的盘同样地变，这一臂走不到。攻方自己也写了（`research/prompts/m2-supp3-item2-code-r2-opus-output.md:126`）：`| C. 反方向（「小盘写满」「各盘落点不一致」报成「每块盘都没有」，m2i 的同类）在等大小盘上走不到…`；攻方的小盘副本也是两块等大（`research/prompts/m2-supp3-item2-code-r2-opus-model/RERUN.md:5`「两块等大、单元区 384 槽的小盘」）。实测 m2i 下随机历史五段新发现全 0（`logs/proof/m2i--random-history.log`）。`crates/mutations.tsv` 第 170 行因此点名不等盘那条旧用例。要让它「这一段红」得先有盘不等大的历史对拍——正是判决第 44 行第 6 条并进 C368 的那笔欠账。
2. **`check.sh` 贵了一倍多**，取舍没动，见第六节末段。
3. **tt0-B 在五段里各只红 1 段**（快档 1/96、复用 1/48、回退 1/48、第四段 1/32、小盘段 1/32），余量与 m1c、m1d 第一轮一样薄。前一会话照判决第 42 行的思路补了一条判决没点名的定向用例 `rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor`（测试文件第 702 行）与第 178 行钉它。不要的话删这条用例与第 178 行，第 167 行照样在。
4. **`crates/mutations.tsv` 动了三行旧行**：第 153、154 行（跟第 5 条改名）、第 156 行（跟第四段改名，点名的测试从 `…without_the_checker…` 改成 `…with_the_checker…`）。不改这三行 59 号跑不到点名的测试。新行只在末尾追加。
5. **别处还留着旧名（不归我改，列给主 agent）**：`.claude/gate.d/fixtures/74-model-differential.sh/{green,red}/model-differential-cargo-output.log` 两份样本里有 `RollbackTargetNotACandidate(TargetVersionWithoutFileUnsupported)` 与 `allocation_record_wall_sampling_without_the_checker…`；`.claude/gate.d/74-model-differential.sh` 第 6 行头注释写着第四段「这一段不跑池级 checker」、第 9 行引 `crates/mutations.tsv` 第 146–164 行（现在到 178 行）；`.claude/kb/milestone/02-second-txn.md` 第 141 行引的 `RollbackToVersionWithoutFileUnsupported` 这一轮恢复了、又对得上，第 166 行引的 `RollbackTargetNotInRing` 在代码里仍是 `RollbackTargetNotACandidate { exclusion: NotInRing }`（第一轮的改法，这一轮没动）。`grep -rn 'HISTORY_DEVICE_BYTES\|history_parameters' .claude/kb` 零命中。
6. **判决之外的一处判定选择**：第 3 条的「已知红第 0 条那一形只记不停」按已知红清单那一条的判定函数原样判（要根环转过、只有 I-3.1 记账多算、同一步没有 panic / 执行器判出 / 模型对不上）；攻方探针的 observe 模式不看根环转没转、也不看同一步别的失败，我这边更窄（根环没转就出现的 I-3.1 多算照样停、落进新发现）。基线 32 段全跑完、新发现 0；m4a / m4b 的判出量级与攻方同级（攻方 m4a 135/256、m4b 256/256，这里 [0, 32) 上 15、32）。
7. **没有条款没写、要做设计判断的新分支**：这一轮改的都是已有拒绝的归属与报法（树表 0 条那一格的条款原文在 D16 已定项 1 / 已定项 9、D23 已定项 14，判决第 52–53 行核过不动条款），没有新加 `todo!`、没有新加 `assert!`、没有新的「第一版不支持」成员。
8. **别的会话在同一个仓里**：`crates/singlefs-harness/src/crash.rs` 21:39 UTC 被别人改了一行文档注释（第一节第 2 条），我没碰；暂存区没动过（本会话只跑读命令与 cargo / 门禁脚本，没有任何 git 写操作）。

## 九、这一轮写过的文件

`crates/` 的改动全部由前一会话（同一轮、同一份派发）写出，本会话一行没改，只核、只跑：

- `crates/singlefs-core/src/mount.rs`（判决第 1 条：成员单列、枚举回三值、第 1240 行）
- `crates/singlefs-harness/src/model_comparison.rs`（第 1 条：映射、上限表、那条 lib 测试改写改名）——未跟踪文件，不在 `git diff --stat` 里
- `crates/singlefs-harness/src/model.rs`（第 5 条两处改名；新计数 `unit_area_wall_refusals_in_the_interval`，第 554、571、1435 行）——未跟踪文件
- `crates/singlefs-harness/src/history.rs`（第 1–3 条：成员名、`HistoryDeviceWidth`、`HistoryExecution`、`PerStepChecker::RunContinuingPastTheRingTurnForm`、小盘比重、计数与报告行、删 `HISTORY_DEVICE_BYTES` 与 `history_parameters`）
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（小盘段、第四段改跑 checker 并改名、三条定向用例、大档的环境变量开关与两组比重、快档成员名）
- `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`（m4b 那条单元测试）
- `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`（断言换成新成员）
- `crates/mutations.tsv`：末尾追加第 165–178 行（14 行，变异名依次：tt0-A 快档、tt0-A 步 4、tt0-B 快档、胶水、m2h、m2i、m2j、m4a、m4b 第四段、m4b 步 1、只记不停退回、m1c、m1d、tt0-B 的 F 之下用例），另改旧行第 153、154、156 行的点名测试名（第八节第 4 条）

`git diff --stat -- crates litmus` 本会话原样（含开工前就有的第 2 件改动与别的会话那一行 `crash.rs`；`model.rs`、`model_comparison.rs` 未跟踪、不在 stat 里）：

```
 crates/mutations.tsv                               |   33 +
 crates/singlefs-checker/src/walk.rs                |   38 +
 crates/singlefs-core/src/allocator.rs              |    6 +-
 crates/singlefs-core/src/mount.rs                  |   42 +-
 crates/singlefs-core/src/transaction.rs            |   29 +-
 crates/singlefs-harness/src/crash.rs               |    2 +-
 crates/singlefs-harness/src/history.rs             | 1104 ++++++++++++++++----
 crates/singlefs-harness/src/lib.rs                 |    2 +
 .../tests/second_transaction_step_four_rollback.rs |   34 +-
 .../tests/second_transaction_step_one_overwrite.rs |   62 +-
 ..._transaction_supplement_three_random_history.rs |  684 +++++++++++-
 ...ion_supplement_two_commit_generated_fallback.rs |   12 +-
 ...d_transaction_supplement_two_unequal_devices.rs |   76 +-
 13 files changed, 1841 insertions(+), 283 deletions(-)
```

## 十、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 `crates/mutations.tsv` 整表复跑（59 号）归 `crash-verifier`；没提交、没做任何 git 写操作。
- 没改 74 号（新段、头注释、样本里的旧名归主 agent），没改 kb、没改 `research/`。
- 变异证明只在 debug 下跑、每条跑点名二进制的整个；本会话没重跑任何一条（源码与证明当时一致，第一节）。新增 14 行没在 59 号脚本下整表复跑过（它的形态是过滤到点名那一条，我跑的是整个二进制、红点包含点名那条）。第 176、177 行靠调用栈里的函数名（`raise_rollback_floor`、`mount_rollback`），要带调试符号的构建、函数改名会悄悄失效。
- 小盘段的比重只在 release、[0, 128) × 150 步上比过四组（`logs/probe-unit-area-round1.out`）；没在别的种子区间、别的步数上量基线误红与判别力的分布。
- 没加层 0 流或崩溃点重放用例（这一轮改的是模型对拍与拒绝归属，不新增落盘流）。`run_history_campaign` 跑的过程中不报进度，这一轮没补。
- 产物没进 `research/results/`（写范围闸不放行）：全在 `research/prompts/m2-supp3-item2-r2-fix-implementer/`——`logs/proof/`（每条变异与基线的整二进制输出）、`logs/proof-progress.txt`、`logs/probe-unit-*.log`、`logs/time-*.log` 与 `logs/handover-time-*.log`、`logs/check-after.log` / `check-final.log` / `check-handover.log`、`logs/workspace-release-test.log`、`logs/gate-*.log`、`logs/source-at-proof-start.sha256` 与本会话的 `logs/source-now-*.sha256`；脚本 `scripts/`（`run-proofs.sh`、`mutate.py`、`probe-unit-area.sh`、`time-segments.sh`、`recheck.sh`）；变异定义 `proof-mutants/`；副本 `copies/{before,probe,proof}`（带 target，几 GB，可删）。门禁 69 号要求变异表改了之后 `research/results/` 里有一份不比它旧的产物——要不要拷、拷哪几份由主 agent 定。
- 前一会话写到一半的报告没续，原样留在 `report-session-1-interrupted.md`（内容与这一份大体同，行号有几处差一，以这一份为准）。

**推翻条件**：59 号在主工作区逐条施加第 165–178 行时有一条点名的测试没红；或 74 号 / 别的机器上第四段、小盘段基线出现新发现（这两段现在跑 checker，已知红第 0 条之外的 checker 判红都会让它们红——m4a、m4b 之外若有合法状态被 checker 判红，会以基线误红的形态出现）；或有人拿出「树表 0 条的根不在回退候选集里」的条款原文（那样第 1 条的成员归属就反过来）。
