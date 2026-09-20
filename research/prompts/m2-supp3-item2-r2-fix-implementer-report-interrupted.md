# m2-supp3-item2 代码三方第二轮改法：实现员报告

时刻：开工 2026-09-19 17:26 UTC（东京 2026-09-20 02:26）。照判决 `research/prompts/m2-supp3-item2-code-r2-main-verification.md` 第三节第 1–5 条做；74 号的新段归主 agent，没碰 `.claude/gate.d/`。开跑前 `ps` 看负载：没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，也没有别的 `cargo` / `gate.sh`（只有常驻的 vllm / ray 进程），没等锁。

## 一、五条逐条

1. **树表 0 条单列**：`crates/singlefs-core/src/mount.rs` 第 56 行恢复 `MountError::RollbackToVersionWithoutFileUnsupported(RollbackTarget)`（名字取第一轮之前那一个，里程碑第 141 行引的就是它），文档写明「在回退候选集里，第一版不支持」；`RollbackCandidateExclusion`（第 128 行）回到三个值 `NotInRing`、`BelowEffectiveFloor`、`OnAbandonedTimeline`；第 1240 行在任何写之前报新成员。穷举的调用方三处，都没写 `_ =>`：胶水 `crates/singlefs-harness/src/model_comparison.rs` 第 220 行映射到 `ModelRefusalReason::RollbackToVersionWithoutFileUnsupported`（模型里标「第一版不支持」的那一条，照代码今天的读法划进必须拒）、第 260 行（上限那张表）；`crates/singlefs-harness/src/history.rs` 第 1758 行（成员名）。两条互报的变异五段都红（见第二节）。
2. **小盘段**：盘宽做成取样点参数 `HistoryDeviceWidth`（`history.rs` 第 64 行：`FourGibibytes` / `UnitAreaOf384Slots`，后者照攻方副本：单元区 384 槽、journal 环 128 MiB），跑法收成 `HistoryExecution { per_step_checker, device_width }`（第 2580 行），`execute_history_with` / `run_history_campaign` / `shrink_*` 都改收它。原来写死 4 GiB 的 `HISTORY_DEVICE_BYTES` 与 `history_parameters()` 删掉，盘宽跟着 `HistoryPool` 走（镜像的 `device_size_in_bytes`、mkfs 参数、模型几何、分配器都从它取）。新比重 `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`（第 494 行），新段 `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（交接见第四节）。m2h、m2j 这一段红；**m2i 这一段红不了**（等大的两块盘走不到「小盘写满」，见第三节第 1 条），它由不等盘那条旧用例红。
3. **第四段照跑 checker**：`PerStepChecker` 加 `RunContinuingPastTheRingTurnForm`（`history.rs` 第 2552 行）：每一步写过盘之后跑 checker，判红时整份观察对得上「已知红第 0 条」那一形（`ring_turn_leaves_allocated_statistic_above_walked`，同一步里有 panic、执行器判出、模型对不上、别的不变量红就不算）只记一笔接着走（第 2743 行起），别的判红照停照分类。计数进 `HistoryTally::ring_turn_form_steps_noted` / `histories_with_the_ring_turn_form_noted`（第 1370 行），报告那一行第 1609 行。第四段改用这一档、改名 `allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count`，多一条断言 checker 真的跑过。m4a、m4b 第四段都红。另补单元测试 `allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records`（`crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs` 第 585 行），m4b 下红。耗时见第五节。
4. **抬 F、回退各一条逼近墙的写死用例**：`raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds`（随机历史测试文件第 549 行）、`rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds`（第 600 行）。今天的代码上两段都做成、之后正好 812 条；m1c、m1d 下各自那一条红（同时第四段也各红 1 段：m1c 种子 30、m1d 种子 31）。
5. **命名**：`crates/singlefs-harness/src/model.rs` 第 1921、1936 行改成 `raised_floor_carried_by_one_device_only_does_not_take_effect`、`carried_unit_keeps_the_generation_of_the_publish_that_wrote_it`；`crates/mutations.tsv` 第 153、154 行点名的测试名与过滤串跟着改。`naming-lint.sh` 原样末行：`  ✓ 命名纪律通过：查了 195 个 .rs 文件、38350 个声明的名字`（改之前是 `  ✗ 2 处名字不合命名纪律（查了 195 个 .rs 文件、38350 个声明的名字）`，点名的就是这两处）。

## 二、新测试与「改坏哪一行 → 哪条断言红」

做法：`rsync -a --exclude target --exclude .git` 拷仓到 `copies/proof`（自己的 target，debug），先跑不改动的基线，再逐条施加、跑点名测试所在的**整个测试二进制**、还原（从 `.pristine` 拷回并把 mtime 摸成现在，`scripts/mutate.py`）。脚本 `scripts/run-proofs.sh`，进度 `logs/proof-progress.txt`，每次跑的整份输出 `logs/proof/<变异>--<二进制>.log`。**基线红集为空**：随机历史、步 4、步 1、不等盘、harness lib 五个二进制基线全绿（`base--*.log`）。红点全在测试断言行上，没有 `debug_assert` 先红的（debug 下跑；release 没另跑）。证明开跑之后主工作区只多了三样：`cargo fmt` 的换行、几处文档注释、第三节第 3 条那条 F 之下的回退用例（它的证明另跑：把副本同步到最终的树再施加 tt0-B，见表里 tt0-B 那一行）——`logs/proof-copy-vs-final.diff`。

新测试（行号是函数所在行，现查）：

| 测试 | 位置 |
|---|---|
| `unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`（小盘段） | `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs:344` |
| `allocation_record_wall_sampling_with_the_checker_refuses_only_above_one_node_by_the_true_count`（第四段改成跑 checker、改名） | 同文件 `:300` |
| `raising_the_floor_with_a_second_empty_publish_that_fills_the_allocation_node_to_exactly_812_records_succeeds` | 同文件 `:549` |
| `rolling_back_to_a_root_whose_warm_up_fills_the_allocation_node_to_exactly_812_records_succeeds` | 同文件 `:600` |
| `rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor`（判决没点名，为 tt0-B 余量补的，见第三节第 3 条） | 同文件 `:702` |
| `allocated_statistic_equals_the_span_sum_of_the_allocation_records_past_six_hundred_records` | `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs:585` |
| `model_comparison::tests::each_rollback_candidate_exclusion_and_the_version_without_file_map_to_their_own_reasons`（旧名 `each_rollback_candidate_exclusion_maps_to_its_own_reason`，改写成三值加单列成员） | `crates/singlefs-harness/src/model_comparison.rs:380` |

改过断言的旧测试：`second_transaction_step_four_rollback.rs:99`（`rolling_back_to_a_warm_up_root…` 改断言新成员）、随机历史快档「各条路径都跑到了」那组成员名（同文件第 86 行）。

随机历史测试文件里的断言行（下表引用）：五段「没有新发现」依次在第 213（快档）、242（复用）、279（回退）、320（第四段）、363（小盘段）行；第四段「墙按真条数放行过」第 328 行；小盘段「用户数据那一处被『每块盘上都没有』拒过」第 373 行；812 条用例第 487 行、抬 F 用例第 568 行、回退用例第 625 行、F 之下回退用例第 727 行（都是 `run.ending == Completed`）。

| 变异（`crates/mutations.tsv` 行） | 改坏哪一行 | 红在哪（同二进制里同时红的；段数 = 那一段里以新发现收尾的段数） |
|---|---|---|
| tt0-A（165、166） | `mount.rs:1240` 新成员 → `RollbackTargetNotACandidate { exclusion: BelowEffectiveFloor }` | 随机历史五段全红：第 213 行快档 50/96、242 行复用 3/48、279 行回退 46/48、320 行第四段 2/32、363 行小盘段 1/32（签名「拒绝的理由」，模型答「回退到树表 0 条的根（第一版不支持）」）；步 4 二进制 `rolling_back_to_a_warm_up_root_without_a_file_version_is_refused_before_any_write` 红在第 99 行（实际报 `BelowEffectiveFloor`） |
| tt0-B（167、178） | `mount.rs` 第 1221–1224 行 `BelowEffectiveFloor` 那一处 → 报新成员 | 随机历史五段全红，**各只 1 段**：快档种子 6、复用种子 54、回退种子 2、第四段种子 2（第 137 步）、小盘段种子 21；步 4 二进制不红（没有断言 F 之下的用例）。补的 `rolling_back_below_the_effective_floor…` 红在第 727 行（模型答「回退目标低于 F_生效」，实现报 `RollbackToVersionWithoutFileUnsupported`）；副本同步到最终的树之后整二进制复跑，红的是这条用例加五段（`logs/proof/tt0-B-rerun--random-history.log`），基线先单跑过这条是绿的（`logs/proof/base2--below-floor.log`） |
| 胶水（168） | `model_comparison.rs:220` 新成员 → 映射成 `RollbackTargetBelowEffectiveFloor` | lib：红在 `model_comparison.rs:397`；随机历史五段全红，段数与 tt0-A 相同（50 / 3 / 46 / 2 / 1） |
| m2h（169） | `allocator.rs:743–744` 用户数据那一处 `NoAnswerOnAnyDevice => NoFreeSlotOnAnyDevice` → `SomeDevicesFull…` | 只红小盘段：第 363 行，8/32（种子 2, 4, 14, 17, 22, 27, 29, 30，签名「模型说该成、实现拒了」）；前四段 0 |
| m2i（170） | `allocator.rs:746–750` 用户数据那一处 `SomeDevicesWithoutAnswer => SomeDevicesFull…` → `NoFreeSlotOnAnyDevice` | 随机历史**五段都不红**（等大盘走不到）；不等盘二进制 `filling_the_smaller_device_to_the_end_of_its_unit_area_refuses_further_placements_instead_of_panicking` 红在第 275 行（`left: Err(NoFreeSlotOnAnyDevice)`） |
| m2j（171） | `allocator.rs:802–803` 提交内生块那一处 `NoAnswerOnAnyDevice` → `SomeDevicesFull…` | 只红小盘段：第 363 行，21/32 |
| m4a（172） | `mount.rs:292` 回收门槛取「最旧有效根 + 1」 | 第四段第 320 行 15/32（全是 I-3.1、记账少于遍历，例 `盘 0：记账的已分配 Some(2162688)，遍历全部有效根得到 2228224`）；小盘段第 363 行 15/32（11 段 I-3.1，4 段 I-2.1 + I-4.8 + I-7.4：复用的槽被盖、校验和对不上）；前三段 0 |
| m4b（173、174） | `transaction.rs:1749` 过 600 条「已分配」少一槽 | 第四段第 320 行 32/32、小盘段第 363 行 30/32（签名 I-3.1 + I-5.2）；步 1 二进制新用例红在第 610 行（第 37 次覆盖写、612 条，盘 0：`left: 6258688`、`right: 6275072`） |
| 只记不停退回判红就停（175） | `history.rs` 第 2746 行 `if !continues…` → `if true \|\| !continues…` | 第四段红在第 328 行（32 段全以已知红第 0 条收尾，墙一次都没走到）；小盘段红在第 373 行 |
| m1c（176） | `transaction.rs:1329` 准入只在抬 F 的调用栈里多算一个角色 | 抬 F 用例红在第 568 行；第四段第 320 行 1/32（种子 30） |
| m1d（177） | 同一行，只在回退的调用栈里多算 | 回退用例红在第 625 行；第四段第 320 行 1/32（种子 31） |
| W1（156，旧行复核） | `transaction.rs:1334` `>` → `>=` | 812 条用例第 487 行、抬 F 用例第 568 行、回退用例第 625 行、第四段第 320 行 9/32（与第一轮报的 9 段相同，跑 checker 之后不变） |

## 三、停下交主 agent 的与要主 agent 知道的

1. **m2i 在小盘段红不了，不是余量问题**：判决第三节第 2 条写「m2h、m2i、m2j……这一段要红」。m2i 换的是「有的盘答不出、有的盘答得出」那一臂（`allocator.rs:746–750`），两块等大的盘同样地变，这一臂走不到——攻方报告第 126 行自己也写了「在等大小盘上走不到……写明 A 对它不起作用」；模型的几何只有一个盘大小（判决第三节第 6 条），小盘段只能是等大盘。实测：m2i 下随机历史五段 0 段红（`logs/proof/m2i--random-history.log`）。我把 m2i 进 `crates/mutations.tsv` 第 170 行，点名的是不等盘那条旧用例（它红在第 275 行）。要不要让它「这一段红」得先有不等盘的历史对拍，归第 6 条那笔欠账。
2. **`check.sh` 变慢一倍多**：改之前随机历史那个二进制 debug 下 51.2 秒（开工时的树拷到 `copies/before` 上量，`logs/before-random-history-debug.log`），改之后 188–194 秒（`logs/check-after.log` 188.4 秒，`logs/proof/base--random-history.log` 194 秒），整道 `check.sh` 3:37.73（`/usr/bin/time -v`）；第一轮报告量的是 1:20.78–1:23.32。多出来的几乎全在第四、第五段：debug 下 checker 每一步都走一遍全部有效根。攻方在 release 下量的「每段约 0.55 秒」对的是 release；debug 下两段各 180 秒上下（五段在同一二进制里并行、各 16 线程，互相抢 32 核）。各段单跑的数见第五节。要不要为 `check.sh` 缩这两段（种子数、步数）、或只在 release 下跑它们，是门禁时长的取舍，我没动；改之前要重量判别力（小盘段 m2h 8/32、m2j 21/32；第四段 m4a 15/32、m4b 32/32、m1c / m1d 各 1/32 但另有写死用例）。
3. **tt0-B 在五段里各只红 1 段**：「低于 F」报成「树表 0 条」这一条（判决要的「把候选排除报成它」），快档 1/96、复用 1/48、回退 1/48、第四段 1/32、小盘段 1/32——四段里都红，但与 m1c、m1d 第一轮的余量一样薄。我照第 4 条的思路补了一条写死的用例 `rolling_back_below_the_effective_floor_is_refused_for_being_below_the_floor`（判决没点名），`crates/mutations.tsv` 第 178 行钉它。不要可以删那条用例与第 178 行，第 167 行（点名快档）照样在。
4. **`crates/mutations.tsv` 改了两行旧行**：第 153、154 行（第 2 件第一轮加的）的测试名随第 5 条改名、第 156 行随第四段改名改了点名的测试（`allocation_record_wall_sampling_without_the_checker…` → `…_with_the_checker…`）。不改这三行 59 号跑不到点名的测试。新行只在末尾追加（第 165–178 行，14 行）。
5. **kb 与 74 号样本里的旧名**（不归我改，列出来）：`.claude/gate.d/fixtures/74-model-differential.sh/{green,red}/model-differential-cargo-output.log` 里还有 `RollbackTargetNotACandidate(TargetVersionWithoutFileUnsupported)` 与 `allocation_record_wall_sampling_without_the_checker…`；74 号脚本头注释写着第四段「不跑池级 checker」、判别力引「第 146–164 行」；里程碑第 141 行引的 `RollbackToVersionWithoutFileUnsupported` 这一轮恢复了、又对得上；第 166 行引的 `RollbackTargetNotInRing` 在代码里仍是 `RollbackTargetNotACandidate { exclusion: NotInRing }`（第一轮的改法，这一轮没动）。`grep -rn 'HISTORY_DEVICE_BYTES\|history_parameters' .claude/kb` 零命中。
6. **判决之外的一处设计选择**：第三条「已知红第 0 条那一形只记不停」我按清单那一条的判定函数原样判（要求根环转过、只有 I-3.1 记账多算、同一步没有 panic / 执行器判出 / 模型对不上）。攻方探针的 observe 模式不看根环转没转、也不看同一步别的失败（`scripts/opus_r2_probe.rs` 第 57–62 行）；我的更窄：根环没转就出现的 I-3.1 多算照样停（落到已知红第 1 条或新发现）。基线 32 段都跑完、没有新发现，m4a / m4b 的段数与攻方同量级（攻方 m4a 135/256、m4b 256/256；这里 [0, 32) 上 15、32）。
7. 没有要坏盘才走得到、条款没写的新分支；没加 `todo!` / 新 `assert!`。

## 四、交主 agent：74 号的新段

- **测试**：`unit_area_wall_sampling_on_small_devices_passes_only_a_placement_refused_on_every_device`，在 `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（第 344 行），就在 74 号已经在跑的那个测试二进制里。
- **报告标题行**（经 `print_uncaptured` 写进标准输出，与另四段同一格式）：`── 随机历史：小盘上逼近单元区墙的取样点 ──`。
- **跑法**：种子 [0, 32)（`UNIT_AREA_WALL_SAMPLING_FIRST_SEED` / `_SEEDS`，同文件第 43–44 行）、每段 150 步（第 45 行）、比重 `GenerationWeights::TOWARD_THE_UNIT_AREA_WALL`、`PerStepChecker::RunContinuingPastTheRingTurnForm`、`HistoryDeviceWidth::UnitAreaOf384Slots`、线程数 = min(核数, 16)。报告第一行原样：`种子 [0, 32)，每段 150 步，比重：逼近单元区墙（小盘上落点拒绝那一格的取样点）；每一步之后跑池级 checker，已知红第 0 条那一形只记不停；两块单元区 384 槽的小盘（journal 环 128 MiB）`。
- **74 号要读的那一行**（格式与另四段相同，这一轮每段末尾都多了一个字段「单元区墙按区间放行 N 次」；74 号现在的 `sed 's/.*模型对拍 \([0-9]*\) 步.*/\1/p'` 不受影响）。今天的代码上 debug 与 release 逐字相同（确定性：同一批种子、同一份镜像字节）：

```
  模型对拍 4749 步：该拒而拒 649、区间里拒 325、该成而成 3775；比过根 4400 条、分配记录 78450 条、冷启动内容 52 次、抬 F 上限 165 次；回退到 txg = F_生效 > 0 的根做成 0 次；分配记录墙按镜像上的真条数放行 0 次；单元区墙按区间放行 312 次
```

- **预期计数**（`logs/check-after.log` 与 `logs/proof/base--random-history.log` 两次一样）：历史 32 段，跑完 32、以已知红收尾 {}、新发现 0；`Err 成员 PublishError::PlacementRefused(NoFreeSlotOnAnyDevice)：304 次`、`Err 成员 MountError::Publish(PublishError::PlacementRefused(NoFreeSlotOnAnyDevice))：8 次`（抬 F 的空发布被拒；可写挂载 Err 0 次）；`checker 跑了 3725 次；一个写都没发、沿用上一次结论的 1107 步；已知红第 0 条那一形只记不停 2542 步（32 段历史）`。测试自己断言：没有新发现、用户数据那一处被「每块盘上都没有」拒过 ≥ 1、「单元区墙按区间放行」≥ 1、checker 跑过 > 0。
- 若 74 号要多判一格：这一段「单元区墙按区间放行」> 0（为 0 说明落点拒绝没走到，等于第一轮 4 GiB 那种「判别子观测不到」）。
- **判别力**：m2h 8/32、m2j 21/32（`crates/mutations.tsv` 第 169、171 行）；release 下 [0, 128) 量过：基线 0/128、m2h 45/128、m2j 81/128，按 32 个一窗切四窗 m2h 每窗 8 / 14 / 14 / 9（`logs/probe-unit-area-round1.out`，探针在 `copies/probe`，比重同名；四组比重里选了这一组，另三组的数在同一份日志）。
- **第四段**标题不变（`── 随机历史：逼近分配记录墙的取样点 ──`），第一行变成 `…；每一步之后跑池级 checker，已知红第 0 条那一形只记不停；两块 4 GiB 的盘`，今天的计数：`模型对拍 4001 步：该拒而拒 504、区间里拒 335、该成而成 3162；…；分配记录墙按镜像上的真条数放行 326 次；单元区墙按区间放行 0 次`，`checker 跑了 3103 次；…；已知红第 0 条那一形只记不停 2032 步（32 段历史）`，历史 32 段全部跑完、新发现 0。74 号脚本头那句「这一段不跑池级 checker」要改。

## 五、第四段跑 checker 之后的耗时

主工作区、每段单跑（`cargo test … -- <过滤串> --nocapture`，同一二进制里别的测试滤掉；段内 16 线程），`/usr/bin/time`（`logs/time-segments.out`、`logs/time-before*-wall.log`）：

| 段 | debug 墙钟 / user | release 墙钟 / user |
|---|---|---|
| 第四段，改之前（不跑 checker，`copies/before`） | 15.4 s / 197.3 s | 4.9 s / 68.3 s |
| 第四段，改之后（跑 checker，已知红第 0 条只记不停） | 103.1 s / 1399.6 s | 11.1 s / 154.7 s |
| 小盘段（新） | 118.5 s / 1616.5 s | 11.6 s / 159.0 s |
| 快档（对照，没改） | 24.7 s / 334.3 s | 3.2 s / 44.3 s |

release 下第四段 32 段 11.1 秒、每段约 0.35 秒墙钟（16 线程并行；user 每段 4.8 秒），是改之前的 2.3 倍，与攻方「0.55 对 0.25」的倍数一致；debug 下是 6.7 倍。整个二进制在 `check.sh` 里五段并行跑，第四段报「用时 179.1 秒」、小盘段 188.4 秒（互相抢核），整道 `check.sh` 3:37.73（第三节第 2 条）。

## 六、check.sh 与门禁阶段

`nice -n 19 bash .claude/scripts/check.sh`（主工作区，最后一次 18:45:56–18:49:38 UTC；整份 `logs/check-final.log`，外面包了 `/usr/bin/time -v` 与一行 `echo "exit $?"`）末尾原样：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
```

四段 ✓（格式、clippy、构建、单测）；`grep -c '^test result: ok'` = 36、`grep -c FAILED` = 0；`Elapsed (wall clock) time: 3:41.62`、`Exit status: 0`。前一次（加 F 之下那条回退用例之前）3:37.73，同样全绿（`logs/check-after.log`）。

阶段归属表登记给实现员的三个阶段（`awk … stage-owners.tsv` 列出 33、53、74），逐个原样末行：

| 阶段 | 原样末行 | 退出码 |
|---|---|---|
| 33-mutation-tables.sh | `  ✓ 142 个实验二进制都有成形的变异表，1509 条变异的原文各命中源码一次；crates/mutations.tsv 173 条的原文各命中源码一次（本阶段不跑变异，只验装置在、锚点对得上）` | 0 |
| 53-format-const-placeholders.sh | `  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））` | 0 |
| 74-model-differential.sh | `      随机历史：逼近分配记录墙的取样点：模型对拍 4001 步：该拒而拒 504、区间里拒 335、该成而成 3162；比过根 4017 条、分配记录 71998 条、冷启动内容 62 次、抬 F 上限 171 次；回退到 txg = F_生效 > 0 的根做成 0 次；分配记录墙按镜像上的真条数放行 326 次；单元区墙按区间放行 0 次` | 0（19 秒，release 产物是计时那一步编好的） |

74 号今天只查它名单上的四段，小盘段它还没列（第四节）。`naming-lint.sh` 最后一次原样末行：`  ✓ 命名纪律通过：查了 195 个 .rs 文件、38425 个声明的名字`。

## 七、这一轮写过的文件

- `crates/singlefs-core/src/mount.rs`（第 1 条：成员与枚举、第 1240 行）
- `crates/singlefs-harness/src/model_comparison.rs`（第 1 条：映射、上限表、那条 lib 测试改写改名）
- `crates/singlefs-harness/src/model.rs`（第 5 条两处改名；新计数 `unit_area_wall_refusals_in_the_interval`，第 554、571、1435 行）
- `crates/singlefs-harness/src/history.rs`（第 1–3 条：成员名、`HistoryDeviceWidth`、`HistoryExecution`、`PerStepChecker::RunContinuingPastTheRingTurnForm`、小盘比重、计数与报告行、删 `HISTORY_DEVICE_BYTES` / `history_parameters`）
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（小盘段、第四段改跑 checker 并改名、三条写死用例、大档加 `SINGLEFS_RANDOM_HISTORY_CHECKER` / `SINGLEFS_RANDOM_HISTORY_DEVICES` 与 `wall` / `unit-area-wall` 两组比重、快档成员名）
- `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs`（m4b 那条单元测试）
- `crates/singlefs-harness/tests/second_transaction_step_four_rollback.rs`（断言新成员）
- `crates/mutations.tsv`：末尾追加第 165–178 行（14 行），变异名依次：tt0-A（快档）、tt0-A（步 4）、tt0-B（快档）、胶水、m2h、m2i、m2j、m4a、m4b（第四段）、m4b（步 1 单元测试）、只记不停退回、m1c、m1d、tt0-B（F 之下的写死用例）；另改旧行第 153、154、156 行的测试名（第三节第 4 条）。
- `cargo fmt --all` 改了 `history.rs` 与随机历史测试文件的换行（它们在上面的清单里）。没碰 `crates/singlefs-checker/`、`allocator.rs`、`transaction.rs`、`lib.rs`、另两个测试文件（`git diff --stat` 里它们是开工前就有的第 2 件改动）。

相对开工快照（`baseline-snapshot/crates-litmus-before.tar`）的增删行：mount.rs +12 −11；model_comparison.rs +15 −13；model.rs +9 −2；history.rs +239 −80；随机历史测试 +312 −24；步 1 测试 +51；步 4 测试 +3 −6；mutations.tsv +17 −3。

`git diff --stat -- crates litmus` 原样（含开工前就有的第 2 件改动；`model.rs`、`model_comparison.rs` 是未跟踪文件，不在 stat 里）：

```
 crates/mutations.tsv                               |   33 +
 crates/singlefs-checker/src/walk.rs                |   38 +
 crates/singlefs-core/src/allocator.rs              |    6 +-
 crates/singlefs-core/src/mount.rs                  |   42 +-
 crates/singlefs-core/src/transaction.rs            |   29 +-
 crates/singlefs-harness/src/history.rs             | 1104 ++++++++++++++++----
 crates/singlefs-harness/src/lib.rs                 |    2 +
 .../tests/second_transaction_step_four_rollback.rs |   34 +-
 .../tests/second_transaction_step_one_overwrite.rs |   62 +-
 ..._transaction_supplement_three_random_history.rs |  684 +++++++++++-
 ...ion_supplement_two_commit_generated_fallback.rs |   12 +-
 ...d_transaction_supplement_two_unequal_devices.rs |   76 +-
 12 files changed, 1840 insertions(+), 282 deletions(-)
```

## 八、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异表整道（59 号）归 `crash-verifier`；没提交。
- 没改 74 号（新段、头注释、样本里的旧名归主 agent），没改 kb（第三节第 5 条列的几处）。
- 变异证明只在 debug 下跑、每条跑点名二进制的整个；新增的 14 行在 59 号的脚本下没有整表复跑过（它的形态是 `cargo test --offline <参数>` 过滤到点名那条，我跑的是整个二进制，红点包含点名的那条）。m1c、m1d 两行靠调用栈里的函数名（`raise_rollback_floor`、`mount_rollback`），与第 143、144 行一样要带调试符号的构建、函数改名会悄悄失效。
- 小盘段的比重只在 release、[0, 128) × 150 步上比过四组（`logs/probe-unit-area-round1.out`）；没在别的种子区间、别的步数上量基线误红与判别力的窗口分布到更大范围。
- 没加层 0 流或崩溃点重放用例。`run_history_campaign` 不报进度（第 1 件留下的形态，`implementation-workflow.md`「跑的过程中报进度」那一条），这一轮没补。
- 草稿与产物没进 `research/results/`（写范围闸不放行）：全部在 `research/prompts/m2-supp3-item2-r2-fix-implementer/`——`logs/proof/`（每条变异与基线的整二进制输出）、`logs/proof-progress.txt`、`logs/probe-unit-area-round1.out` 与 `logs/probe-unit-*.log`（小盘段选比重）、`logs/time-*.log` 与 `logs/time-segments.out`（计时）、`logs/check-after.log` / `logs/check-final.log`、`logs/gate-*.log`、`logs/before-random-history-debug.log`、`logs/proof-copy-vs-final.diff`、`logs/source-at-proof-start.sha256`；脚本 `scripts/run-proofs.sh`、`scripts/mutate.py`、`scripts/probe-unit-area.sh`、`scripts/time-segments.sh`、`scripts/new-rows.tsv`；变异定义 `proof-mutants/`；副本 `copies/{before,probe,proof}`（带 target，几 GB，可删；探针测试文件 `copies/probe/crates/singlefs-harness/tests/probe_unit_area_wall.rs` 只在副本里）。门禁 69 号在变异表改了之后要 `research/results/` 里有一份不比它旧的产物，要不要拷由主 agent 定。

**推翻条件**：59 号在主工作区逐条施加第 165–178 行时有一条点名的测试没红；或 74 号 / 别的机器上小盘段、第四段基线出现新发现（这两段现在跑 checker，已知红第 0 条之外的 checker 判红都会让它们红——m4a、m4b 之外若有合法状态被 checker 判红，会以基线误红的形态出现）；或有人指出「树表 0 条的根不在回退候选集里」的条款原文（那样第 1 条的成员归属就反过来）。
