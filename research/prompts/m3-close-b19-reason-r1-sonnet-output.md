# m3-close-b19-reason-r1 正推腿（Sonnet）报告（2026-10-30）

立场：从 `crates/` 的写者、恢复与判法推「应该是什么」，逐格与题面的读法并排。三格：Q1（C934 的前提）、Q2（三处保守简化的代价）、Q3（C937 两个方向）。读过正文 `research/prompts/_m3-close-b19-reason-r1-body.md`、第二轮判决 `research/prompts/m3-close-b19-design-r2-main-verification.md`；没读禁读清单里的两份。每个数标「量的」（命令跑出来的）或「推的」（读码或模型推的），并写明是哪份文件 / 哪份产物。

## 0 复跑与文件

模型目录 `research/prompts/m3-close-b19-reason-r1-sonnet-model/`（纯 Python 标准库，不起 cargo、不碰 GPU）。复跑一条命令（交回前在新 shell 里整份跑过，七步全 `identical`、退 0）：

```
SINGLEFS_RUN_SCOPE='种类=model; 跑什么=三方腿 Python 小模型 rerun.sh; 范围=research/prompts/m3-close-b19-reason-r1-sonnet-model; 为什么=复跑核 out/ 基线' \
  bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-reason-r1-sonnet-model/tools/rerun.sh \
  --repo <仓根> --frozen <任一存在的目录> --work <新的草稿目录> [--model <模型目录>]
```

原样输出（交回前最后一次）：`name=q1_scene_and_direction_identical lines=32`、`name=q2_port_check_against_opus_identical lines=1`、`name=q2_scale1_identical lines=243`、`name=q2_scale4_identical lines=216`、`name=q2_defense_port_identical lines=112`、`name=q3_span_readings_identical lines=71`、`name=q2_summary_identical lines=153`，退出码 0。`--frozen` 只是模板要的参数，这些模型不读冻结副本里的 crates（只读仓里第二轮两腿的 `q7_cost_model.py`、`q7_cost.py`）。线程取「线程上限：31」（`capped.sh 31`）。

| 文件 | sha256 |
|---|---|
| `out/q1_scene_and_direction.txt` | `655f5f904226f83e654b944e5404867a378f964e0d4ed971ccc8104f41f17851` |
| `out/q2_defense_port.txt` | `de2e643a237ff9e060c5f772b5c25a4b473b1e4836b1b9fddaeef54a84442bb9` |
| `out/q2_port_check_against_opus.txt` | `80a0cd179ccca67e4b955353ecff97eca7bb3be864bb9df17fa8e49505ccd0f3` |
| `out/q2_scale1.txt` | `b8d86b43418cbc17dd1ccbe9a500037cf641193e88a00181cc5965d76eafc8ae` |
| `out/q2_scale4.txt` | `da636743fc8a61c1f072699b2df5fc3bddd5ea014733a670f5303223bffc6d2d` |
| `out/q2_summary.txt` | `32def9d372c952a6b3b939f31ce5952bd6a6489eb359dfe2c285d6f91686d58f` |
| `out/q3_span_readings.txt` | `cd5d971cffadc20c0376c8ebc0df77e80918fd70c3090c37fab1dbfdb96fcca6` |
| `q1/q1_scene_and_direction.py` | `8d127c81bfa99e8a8784b370459625fb415d6e5134dcde651d56cede0c323249` |
| `q2/check_against_opus.py` | `b5f04fcdf71d214412ad9b3461616edd040de22a02851b21305e746c30cc3c26` |
| `q2/q2_defense_port.py` | `a90e3c3183cc6d5bea0251d1cdd62f3293a537ead1813fe15f9db0a2221bcb79` |
| `q2/q2_three_simplifications.py` | `ff4a394542a05856a7e01a724af0d26f7d59b3c97aa9d4eb4de20b183788e419` |
| `q2/summarize_q2.py` | `a8e047b89caa9aad1f307be5c1e75fcf4b7421376d32cccced03ebcba55e7bea` |
| `q3/q3_span_readings.py` | `c1f1bb15635fdfa1ca70d62458ca60ebe50385cb98b03bf55811de347ae7270a` |
| `tools/rerun.sh` | `657965c573edd30e410a61a9b3f43340ffdfe61b35303072186acd4b9a1debef` |

开工快照核对（`sha256sum -c research/prompts/m3-close-b19-reason-r1-snapshot/crates-sha256.txt` 七份全 OK；`kb-sha256.txt` 里 `.claude/kb/checks-owed.md` 与 `.claude/kb/decisions/16-发布语义.md` 两份 FAILED，别的 OK）：引 crates 的行号是快照那一份（此刻逐字相同），引 16-发布语义.md 的行号是此刻的。

## 一、Q1　C934「实例中间的一版根槽一定落了盘」

### 总括

| 问 | 结论 | 标 |
|---|---|---|
| 1 写者根槽写失败时实例停不停 | **造不出**（今天的写者）：经分配器的发布失败即冻结，同一实例下一版发出之前原样重发必须成功；零单元发布失败整个挂载返回错误、实例号烧掉 | 读码推的；FUA 合约是前提 |
| 2 「u 的记录落了、u 的根没落」下一次挂载是新实例 | **造得出**（崩在记录屏障之后、根槽 FUA 之前），但丁读的是 **u 那个实例**在抬 F 那条根的实例表里的行 T，这一形里它等于 u，不是 u + 1；`u ≥ T` 那一格报认不出，所以不会被错认。题面「T = u + 1 > u」混了「新实例第一个 txg」与「u 那个实例的行 T」 | 读码 + 小模型推的 |
| 2b 重建上限不低于真上限 | **推的这句不成立**：假如缝被打开，重建上限高于、等于、低于真上限三种都有（模型 20 000 段合法历史：S=4 三种为 3246 / 9024 / 1097，S=8 为 89 / 9593 / 454），低于的让合法抬 F 被判红 | 小模型量的 |
| 3 钉槽 p 能否认出 | **认不出，也不用它**：p 只记挂载那一刻读坏或读回全零的槽，且一条根记录只带一个槽的编码；重建这一段（walk.rs 8740–9340）一处不读 p | 读码推的 |

### Q1.1 写者在根槽写失败时实例停不停

写者动作与代码：

- 落盘那几步（写单元、屏障、记录、根槽、系统配置槽）任一步失败，`persist_the_publish_or_freeze_it` 把分配器换回发布之前的样子并冻结这次发布：「allocator.freeze_publish(FrozenPublish {」（`crates/singlefs-core/src/transaction.rs:1575`），换回那一句在 1574 行的 `std::mem::replace(allocator, allocator_before_this_publish)`。
- 下一次发布的第一道：「fn refuse_while_a_publish_is_frozen」（`crates/singlefs-core/src/transaction.rs:1545`），调用点 1835（`publish_instance_table_on_version_without_file`）、9477（`publish_first_file`）、10094、12130（`publish_version_of_trees_relocating_units_after_a_check`，它的落盘在 12158）。冻结着就在任何读写之前拒。
- 清掉冻结的只有「pub fn resend_the_frozen_publish」（`crates/singlefs-core/src/transaction.rs:1594`）：逐字节重发，再失败原样再冻结（1604 行 `allocator.freeze_publish(frozen);`）。`singlefs-core/src` 里它没有非注释的调用点（`grep -rn 'resend_the_frozen_publish(' crates/singlefs-core/src | grep -v '///\|//!'` 零命中）：重发由库的使用者调，库内只负责拒。
- kb 对得上：「发布不接受失败。一次发布失败之后把它冻结」（`.claude/kb/decisions/23-journal的角色与格式.md:409`）。

所以同一实例下一版发出之前，u 的单元、记录、根槽 FUA、系统配置轮换、末尾屏障全部成功；FUA 返回时已持久：「FUA 写：返回时已持久」（`crates/singlefs-core/src/block_device.rs:71`）。

逐点答：

1. **重发「成功」而字节没落到盘**：在这份 `BlockDevice` 合约下走不到（FUA 返回即持久、末尾还有屏障）；要走到只能设备不诚实，那不是合法历史，我没为它建模型。坏槽是读侧事件：写成功之后介质读不出，镜像上该槽是 `DoesNotReadBack`，丁在这一槽报「读不出：抬 F 那一刻它住的是什么无从认」（`crates/singlefs-checker/src/walk.rs:9004`），整条部分没判，不会错认。被钉的槽见 Q1.3。
2. **零单元发布不冻结那一支**：「零单元发布（`publish_without_units`）不经分配器、不冻结」（`crates/singlefs-core/src/transaction.rs:1510`），1671 行同口径。它在 singlefs-core/src 里的调用点只有四处（命令与原样输出）：
   ```
   $ grep -rn 'publish_without_units(' crates/singlefs-core/src | grep -v '^\S*:[0-9]*:\s*//'
   crates/singlefs-core/src/mount.rs:5436:            let published = publish_without_units(
   crates/singlefs-core/src/mount.rs:5528:            let (_, rehearsed) = prepare_the_publish_without_units(
   crates/singlefs-core/src/mount.rs:5588:                let (_, rehearsed) = prepare_the_publish_without_units(
   crates/singlefs-core/src/mount.rs:8060:            publish_without_units(
   crates/singlefs-core/src/transaction.rs:1364:        let output = publish_without_units(
   crates/singlefs-core/src/transaction.rs:1662:    let (writes, output) = prepare_the_publish_without_units(pool, previous_root, plan);
   ```
   其中真发的只有 mount.rs 5436（暖机的 `publish_empty_after`）、8060（写行）与 transaction.rs 1364（mkfs 暖机）。失败一律往外返回：「.map_err(|cause| publish_failed_after_acquisition(&pool, &[], cause))?;」（`crates/singlefs-core/src/mount.rs:8103`）、暖机循环「let next = publish_empty_after(&mut pool, &mut allocator, &current, instance).map_err(」（`crates/singlefs-core/src/mount.rs:8120`）、mkfs 暖机「.map_err(|cause| WarmUpFailed {」（`crates/singlefs-core/src/transaction.rs:1377`）。挂载整体报错，取过的号烧掉，下一次挂载给中间实例写 (i, 0, 0)：「fn instance_rows_to_write」（`crates/singlefs-core/src/mount.rs:5980`）。**同一实例接着发下一版走不到**。`publish_without_units` 是 `pub fn`，库外的调用方可以失败后自己再调（装置 `e158_root_choice_repair.rs` 的 `apply_publish_without_units` 就是这么用的），那是调用方的事，不是写者代码挡住的（推的）。
3. **D16 已定项 1 的「一次处置里根槽写失败 ≤ k_tol 次」对应哪一段**：它是定环深的参数，不是一段写者逻辑：「每区槽数 S 的下界 = k_tol + 2」（`crates/singlefs-format/src/lib.rs:293`）、准入一次最多发的数「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」（`crates/singlefs-core/src/mount.rs:4453`，B = 4 + 2 k_tol）。写者里**没有跳过失败那一版继续发**的一段：失败那一版冻结重发（上面）。「写失败的槽按旧内容算，在飞、没持久的发布不算」（`.claude/kb/decisions/16-发布语义.md:36`）在写者里的对应是冻结时分配器换回发布之前的样子（crates/singlefs-core/src/transaction.rs:1574），重发成功才换成发布之后的那一份。

**前提与它的寿命**：这条「造不出」靠的是今天没有「根槽写失败就推进 txg、不重发」。kb 自己把它排在后面：「探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项，随失败处置的代码在后面的里程碑做」（`.claude/kb/decisions/23-journal的角色与格式.md:409`）；`grep -rn '根槽写失败推进' crates --include=*.rs | wc -l` 输出 0（crates 里没有）。那一段一落地，同一实例就会在 u 的根没落盘时继续发 u + 1，行 T > u、u 的记录在 journal、u 的根从没落过，这条缝就真开了；D16 那句「写失败的槽按旧内容算」正是那个形的会计。

**什么现象会推翻 Q1.1 的结论**：① 写者代码里有一条路在根槽写失败后不冻结、不原样重发就建下一版（grep `freeze_publish` 之外的失败处置）；② 设备在 FUA 写返回成功后丢字节（崩溃重放里出现根槽 FUA 已返回而崩溃后读不到的状态）；③ 零单元发布的失败被某个调用方吞掉、同一实例接着发。

### Q1.2 「u 的记录落了、u 的根没落」，下一次挂载是新实例

**造得出，写者动作与崩溃点**（S = 4，三个区域；每次发布的持久顺序是「单元 → 屏障 → 记录 → 屏障 → 根槽 FUA → 系统配置轮换 → 屏障」，见「屏障 → 空记录 → 屏障 → 根槽 FUA → 系统配置槽轮换 → 屏障」（`crates/singlefs-core/src/transaction.rs:1648`）与 1630 起的 `persist_the_root_then_rotate_the_system_configuration`）：

1. 实例 1 发布 txg 1..20，根都落了盘。
2. txg 21：单元、记录都持久，**崩在根槽 FUA 之前**。盘上：槽 (0,3)（落点式 `(txg mod 3, (txg div 3) mod 4)`）住着 txg 9；journal 里有 (实例 1, txg 21) 带提交标记的记录。
3. 下一次可写挂载：择根 txg 20，施加记录 21，所选版 (1, 21)，它的根哪块盘的根环里都没有：「没写进根环：可写挂载所选那一版是在择到的根上施加 journal 记录得来的」（`crates/singlefs-core/src/mount.rs:799`）。施加出来的根 txg 就是记录的：「checkpoint_txg: record.checkpoint_txg,」（`crates/singlefs-core/src/recovery.rs:5299`）。
4. 写给实例 1 的那一行取所选根的 txg：「selected_root_txg: effective_root.checkpoint_txg,」（`crates/singlefs-core/src/mount.rs:9232`），即 **行 (1, T = 21, W)**；新实例 2 的第一个 txg 另算：「checkpoint_txg_after_the_version(highest_ring_txg.max(highest_record_txg))」（`crates/singlefs-core/src/mount.rs:1420`）= max(20, 21) + 1 = 22。行是随实例 2 的写行发布（txg 22）带进实例表的（「rows.extend_from_slice(rows_written);」`crates/singlefs-core/src/mount.rs:1313`）。
5. 实例 2 接着发 22..33；抬 F 的根 r = 26；最后那张镜像里 txg 33 盖掉了槽 (0,3)。

**丁在这张盘面上认出什么**（小模型 `q1/q1_scene_and_direction.py`，逐槽移植 `root_in_the_ring_slot_at_the_raise`，原样输出在 `out/q1_scene_and_direction.txt`）。镜像槽 → txg：(0,0):24 (0,1):27 (0,2):30 (0,3):33 (1,0):25 (1,1):28 (1,2):31 (1,3):22 (2,0):26 (2,1):29 (2,2):32 (2,3):23。r = 26 之下：

- 坐着比 26 小的根的 4 槽：(0,0)→24、(1,0)→25、(1,3)→22、(2,3)→23，照原样认。
- 被盖的 8 槽取落点式在 26 之下最大的正 txg u：(0,1)→15、(0,2)→18、(1,1)→16、(1,2)→19、(2,0)→14、(2,1)→17、(2,2)→20，都是实例 1 的中间版且行 T = 21 > u，认作落了盘；**(0,3)→21：条文要 u 与抬 F 那条根同一实例，或它那个实例的行 T > u 才认作那一刻住着 u；实例 1 ≠ 2，行 T = 21，21 < 21 不成立**：
  「let its_root_landed = version.instance == raising_instance」（`crates/singlefs-checker/src/walk.rs:9033`）、「if !its_root_landed {」（`crates/singlefs-checker/src/walk.rs:9038`）⇒ 这一槽 `Err`，整条 `RingNotReconstructed`，I-7.9 在这条根上报**部分没判**，不报成立也不报红。注释自己写明这一格：「u 是某个实例的 T 或在被抛弃的时间线上（u ≥ T：根槽可以没落、那一槽仍住着上一圈的根）」（`crates/singlefs-checker/src/walk.rs:8979`），条文同句：「u ≥ T 或那个实例没有行（根槽落没落盘判不了）」（`crates/singlefs-checker/invariants/02-根环.md:16`）。GPU 内核同判：「its_root_landed = its_instance_has_a_row && every_row_of_its_instance_is_after_it;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1885`）。
- 所以「这一形重建会把 u 当成那一刻的住户」**在今天的实现上不成立**：把 T 当成 22 才会成立。22 是实例 2 的第一个 txg，不是任何一行的 T（行里只有 `InstanceTableRow` 的 `published_checkpoint_txg`，「published_checkpoint_txg: u64,」`crates/singlefs-checker/src/walk.rs:1081`，写入口就是上面 9232 行）。
- 取行 T = 22 当**假设**（模型里把它当「缝被打开」）：丁把 (0,3) 认作 txg 21，重建的环 txg = 14..25 共 12 条，真环是 9、14..20、22..25。三组非空发布的摆法下，重建上限与真上限：高于（14 对 9）、等于（15 对 15）、低于（21 对 22），见 `out/q1_scene_and_direction.txt` 前三段。

**重建出的上限不低于真上限这句不成立**（主 agent 推的那句，按式子逐项核）：上限是「min(每块盘上最新的有效根的 txg, 第 4 新的不同状态的非空有效根的 txg，不足 4 个取最旧有效根)」，式子在「fn rollback_floor_ceiling_from」（`crates/singlefs-checker/src/walk.rs:8688`）：`newest_valid_root_txg_on_every_device.min(fourth_newest_non_empty_or_oldest_valid)`（8699）。把真住户 v 换成更新的 u 之后，它对 min 的两项都不单调：

1. 「非空」是跟**前一条有效根**比出来的：「if this_root != previous_root {」（`crates/singlefs-checker/src/walk.rs:8794`）、「previous_valid_root_pointers = pointers;」（8807）。u 挪到更后，它自己和它后面那一条的前一条都变了；u 的状态与前一条相同就不算非空，v 若原本计入则计数少一个，第 4 新的往旧里退。
2. 按状态去重：「if !counted_states.contains(&state) {」（8729）；u 的状态若与更新的某条相同，不计入。
3. v 若本来就无效（低于抬之前的 F，或被抛弃），u 有效就是**多出一条有效根**；它比环里别的有效根都旧时成了「最旧有效根」，不足 4 个非空时上限取它，拉低；多出的根若落在原来没有有效根的盘上，「每块盘上最新的有效根」那一项也变成 min 里多一个小数。
4. 同一块盘上把旧根换成新根，这块盘的最新有效根不会降；上限会降，来源只有上面第 1–3 样。

量的（模型 `q1/…py`，admission 写者 20 000 段合法历史，在抬 F 那一刻根没落的版本 u 所在槽还住着更旧的根 v 的 (抬 F, u) 对，原样输出在 `out/q1_scene_and_direction.txt` 末尾）：

| S | 对数 | 重建上限高于真上限 | 相等 | 低于真上限（都让合法抬 F 被判红） | 其中 v 本来无效 / v 有效 |
|---|---|---|---|---|---|
| 4 | 13 367 | 3 246 | 9 024 | 1 097 | 420 / 677 |
| 8 | 10 136 | 89 | 9 593 | 454 | 331 / 123 |

即便缝被打开，方向也是「多数漏红、少数误红」，不是「只会漏红」。**要求的「坏写者把 F 抬进这条缝里、丁判成立」的盘面**：在今天的实现上**不存在**，因为这条缝在 9033–9038 行已经被 `u ≥ T` 那一格关死（场景输出：T = 21 时 I-7.9 部分没判）；缝被打开的假设下，「高于」那一组（上限 14、真上限 9）里，F 取 10..14 的坏写者抬 F 根会被判成立，而真上限只有 9，这是假设下的盘面，不是今天的盘面。

**什么现象会推翻 Q1.2**：① 任一路径让实例 i 的行 T 大于一个根没落盘的 u（要么写者在根没落时继续发——即 Q1.1 的前提被推翻，要么恢复把所选版的 txg 记得比施加出来的那一版大）；② `previous_row.selected_root_txg` 不取所选版 txg（9232 行改了）；③ 恢复施加记录之后把施加出来的根补写进根环（`NotWritten` 那一支消失，则 u 的根落了，认作住户反而是对的）。

### Q1.3 钉槽 p 能不能把第 1、2 问的形认出来

**认不出，也不用它。**

- p 管的是挂载那一刻读坏或读回全零的槽：「钉槽 p 存钉着的那一槽的编码」（`.claude/kb/decisions/16-发布语义.md:39`），钉值的算法在 `txgs_ring_slots_bad_at_mount_can_hold`（`crates/singlefs-core/src/mount.rs:2937`）与 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held`（`crates/singlefs-core/src/mount.rs:2994`）；根记录只带**一个**槽的编码，并且「钉着的槽不得住着本实例写的根」（`.claude/kb/decisions/16-发布语义.md:39`，I-7.19）。
- 第 2 问那一槽 (0,3) 在新实例挂载那一刻是**读得出的**（住着上一圈的根 txg 9，自证过），不读坏、不全零，所以不钉；它在 r 的根记录里没有任何痕迹。年轻的环上那一槽若还是全零（从没写过），挂载会钉它，但钉的是可能住过的最旧 txg（公式见 `crates/singlefs-core/src/mount.rs:2994` 起的 `oldest_txg_a_ring_slot_bad_at_mount_can_have_held`），不说 u 的根落没落。
- 重建这一段根本不读 p：`awk 'NR>=8740 && NR<=9340 && /pin|钉/' crates/singlefs-checker/src/walk.rs` 无输出（读 p 的是 9756 起的 `pinned_txg_of_the_ring_slot` / `pinned_ring_slot_of_the_root`，只用于 I-7.18、I-7.19）。
- 「读坏的槽重建时报认不出，但第 2 问的槽读得出、坐着更新的根」这一问：读得出、坐着 txg ≥ r 的根走 `OverwrittenSinceTheRaise`，由落点式 u 加 journal 认（上面 Q1.2），不是 p 管的；读坏的走 `DoesNotReadBack`（`crates/singlefs-checker/src/walk.rs:9004` 那条 Err），整条部分没判。这两支互不相交，p 不在其中。

**什么现象会推翻 Q1.3**：重建要靠 p 才能把 `u ≥ T` 那一格从认不出变成认得出（今天不是；要这样改得先让 p 能带多槽、能带「这一槽 u 的根没落」的信息，动格式）。

## 二、Q2　C934 三处保守简化各让「部分没判」多出多少

### 口径与模型（推的模型，不是入库装置）

- 历史与第二轮攻方模型同一份：`research/prompts/m3-close-b19-design-r2-opus-model/q7/q7_cost_model.py` 的 R = 3、区域→盘 (0,1,0)、`rand_steps`（20..60 步）、p_nonempty 0.35、p_lost_root 0.04、p_clean_crash 0.03、p_raise 0.08、种子 0..19 999、写者 admission（合法抬 F）与 bad（越界抬 F）。我把它的丁（`judge_ding`）改写成按槽的 `q2/q2_three_simplifications.py:judge`，对着 `root_in_the_ring_slot_at_the_raise`（`crates/singlefs-checker/src/walk.rs:8986`）逐格移植。**移植核对**：全放开那一档（A 不限层、B 放开、C 无）与原模型的丁逐根比，`out/q2_port_check_against_opus.txt` = 「compared 12483 raise roots; differing 0」（每个 S、J、写者各 1000 种子）；J = 128 admission 的 20 000 种子上部分没判 801 对 15 626 的原数 798 之差已查出是我的层数上限写成了 2，改成不限层后 `q2_scale1.txt` 里这一档是 5.1% / 2.2%，与第二轮报的 5.1% / 2.2% 一致。
- 三处简化各是一个开关：**A** 只认一层，不往 u − 3S 推（`crates/singlefs-checker/src/walk.rs:8981` 的「只看一层：落点式住进这一槽的再上一圈 u − 3S 不往下认」）：放开程度 0 不往下 / 1 往下一层 / 2 不限层；**B** u 是行 T、被抛弃、实例没有行都认不出（`crates/singlefs-checker/src/walk.rs:9038` 的 `!its_root_landed` 那一支）：放开 = 「两种都算（u 落了 / 没落），结论一致才判」；**C** 被盖掉的第 0 代根在抬之前 F = 0 时认不出（`crates/singlefs-checker/src/walk.rs:8963` 起 `largest_positive_txg_the_ring_slot_names_below` 返回 None 之后的 `Err`）：放开 = 把第 0 代根当已知的空树根。
- 第二轮模型没有第 0 代根，C 没东西可量；我另建了 genesis 变体（mkfs 在三个区域的槽 0 各种一条 txg 0 的根，写者的上限也把它算进去）。**genesis=0 的数就是第二轮口径**，genesis=1 才有 C。
- 模型里认不出的情形只出现「u 是行 T」一种（干净崩溃留下的 T 与根没落留下的 T 都算，没有分开量）；「被抛弃时间线上的记录」「实例没有行」模型里恒为 0，没量。
- 命令：`bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-b19-reason-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <任一目录> --work <新目录>`（带 `SINGLEFS_RUN_SCOPE`）；每步与 `out/` 逐字节比，我交回前在新 shell 里跑过：七步全 `identical`、退 0。

### 三处各一行（J = 128，合法抬 F 的根部分没判的比例；genesis=1，种子 20 000，S = 4 / S = 8）

| 简化 | 从今天的实现单独放开它 | 只留它、其余两处放开 | 备注 |
|---|---|---|---|
| 今天（三处都不放） | 31.7% / 54.0% | — | 量的：`out/q2_scale1.txt` |
| A 只认一层 | 31.7% / 54.0%（不变） | 27.0% / 14.4% | 单独放开 A 不变，因为 B 还在挡；A 要和 B 一起才起作用 |
| B 行 T / 被抛弃 / 没有行 | 27.7% / 30.7% | 31.3% / 44.1% | 单独放开 B 只能解开「老一圈没有根」的那一类（环里还没转满一圈） |
| C 第 0 代根 | 31.3% / 44.1% | 5.9% / 18.5% | S = 8 的年轻环上占大头 |
| A（一层）+ B | 6.9% / 18.5% | | |
| A（不限层）+ B | 5.9% / 18.5% | | |
| 三处全放开 | 5.0% / 2.1% | | 第二轮丁的口径 + 第 0 代根 |

没有第 0 代根（genesis=0，即第二轮同一份历史）：今天 31.9% / 42.0%；单独放开 A 31.9% / 42.0%、单独放开 B 27.2% / 13.6%；A（不限层）+ B 5.1% / 2.2%（= 第二轮报的数）。所有格 red = 0（合法抬 F 的根一条也没判红）。

**这些数都带两个没扫过的旋钮**，按 `evidence-discipline.md`「不许先有结论再建模型」那一节的口径列出来，不拿一个点当结论：

1. **环转了几圈**。第二轮的历史 20..60 步，S = 8 时一圈 24 个 txg，历史只有 20..60 步，环转不了几圈（推的）。再把步数乘 4（`q2_scale4.txt`，80..240 步，J = 128，genesis=1）：今天 32.3% / 52.3%；单独放开 A、B、C 都不变（32.3% / 52.3%）；A（一层）+ B 7.9% / 7.1%；A（不限层）+ B 6.0% / 2.4%；全放开 6.0% / 2.4%；只留 C 不放 6.0% / 2.4%。长寿的池里 C 不占份额（第 0 代根早被盖掉），B 单独没有用，A 与 B 绑在一起（要「两种都算」就得往下认那一种）。**年轻池里 B、C 是大头，长寿池里 A 加 B 是大头**。
2. **journal 罩多深**（J）。J = 8：S = 4 全部、S = 8 约 91%–96% 部分没判，开关几乎不改它（`out/q2_scale1.txt` 的 J=8 行）；J = 32：今天 31.7% / 69.7%，全放开 7.9% / 54.8%（推的：J 条只罩最后 J 次发布，抬 F 之后每多发一次就把 u 往环外挤，S = 8 时 u 离末尾更远）；J = 128 才是上面的表。

### 辩方模型那一档（第二轮 `q7_cost.py` 的历史与数法）

`q2/q2_defense_port.py` 抄了它的 `run`（多记每一版的实例 / F / 状态 / 落没落），先断言与原 `run` 逐字相同，再把丁换成上面的按槽判法；journal 口径照它（只留最近 J 个非空发布的记录，状态没变的发布当推得出）。原报的数是「J 够深就逐字重建」的构造，没有行 T 与落没落盘这些变量：

| S | 种类 | 今天（三处都不放） | 只放开 B | A（不限层）+ B | 全放开 | 第二轮原数（J ≥ 24） |
|---|---|---|---|---|---|---|
| 4 | 越界抬 F 的图像里判红的比例（J = 24；真违例图像 3 801 张） | 2 808（73.9%） | 2 962（77.9%） | 3 336（87.8%） | 3 336（87.8%） | 3 536（93.0%） |
| 8 | 同上（7 684 张） | 6 171（80.3%） | 7 019（91.3%） | 7 235（94.2%） | 7 235（94.2%） | 7 236（94.2%） |
| 4 | 合法图像（20 000 张）里有一条抬 F 根部分没判的 | 899（4.5%） | 803 | 30（0.15%） | 30 | 0 |
| 8 | 同上 | 2 718（13.6%） | 753 | 51（0.26%） | 51 | 0 |

量的：`out/q2_defense_port.txt`。S = 8 的全放开 7 235 对原数 7 236 差 1 张；S = 4 差 200 张（3 336 对 3 536）：原模型的丁「逐字重建」，我的按槽判法在根没落那一槽上「两种都算」，两种结论不一致的 209 张图像退成部分没判，这是第二轮那个构造多给的。辩方模型没有第 0 代根，C 不适用。

### 局限（照实列）

- 模型无读故障：实现里槽读不出（`DoesNotReadBack`）与树表单元已被复用（树表指针 None，落在 `CeilingBetweenItsBounds`）只会让部分没判更多，本表是下界。
- genesis 变体里「第 0 代根当已知的空树根」是推的：实现里写成认不出的理由是「已被盖掉、树表无从读」（「它已被盖掉、树表无从读」，`crates/singlefs-checker/src/walk.rs:9015`），mkfs 的第 0 代根树表 0 条按构造就是空树，放开 C 是把这个事实用上；没有在真 checker 上量。
- 辩方模型的 journal 口径（零单元发布当推得出）比实现宽：实现要 u 带提交标记的记录恰好一个实例，空发布的记录同样占环。所以辩方一档的部分没判也是下界。
- 干净崩溃 T 命中与根没落 T 命中没有分开量：B 的 26.3 / 42.0 个点（只留 B 其余放开，S = 4 / S = 8）里两者各占多少这一轮没拆。

**什么现象会推翻 Q2**：① 真 checker 在一批合法崩溃镜像上跑，今天的丁部分没判的比例明显低于上表的 31.7% / 54.0%（说明模型的崩溃频率比真写者高得多）；② `out/q2_port_check_against_opus.txt` 不再是 differing 0（移植漂了）；③ 辩方模型的原数 3 536 / 7 236 在入库装置上被复现而我的 3 336 不是（说明按槽判法多退了）。

## 三、Q3　C937 两个方向都放过，漏不漏

### 机制与口径

取宽在 `crates/singlefs-checker/src/walk.rs`：记录自述的 RCW 三判「fn allocated_statistic_lies_within_its_bounds」（10336）不成立时，把最新根里起点槽单元头用得了、跨度与那一类槽数不符的未释放记录改记成那一类的槽数——「Some(slots_of_the_class) if slots_of_the_class != record.span_slots => {」（`crates/singlefs-checker/src/walk.rs:10372`，两个方向都改）——再算一份界，「fn allocated_statistic_lies_within_the_bounds_of_either_span_reading」（10391）取两份宽的那一边：(L) 取两份 |E(θ_high)| 的小者；(P) 只查「记录自述那份里不在记录里的槽」是否在改记那份里也不在（10408），即**至少一份含它就放过**；(U) 取两份 |E(t)| 的大者再与记录自述那份的 |Ω ∪ U| 取小（10421）。调用点 21344–21363：取宽成立就 `part_of_the_range_left_unjudged("I-3.1", …)`，否则 `judge_naming_the_slots`。

**小世界**（`q3/q3_span_readings.py`，推的，不是入库装置）：盘 0 一块盘、9 槽；真占用 = 至多 3 个不相交的单元（类 1 一槽 / 类 2 两槽），遍历走到的槽 = 单元占的槽；每个单元一条未释放记录，跨度取 {类 − 1, 类, 类 + 1} 或没有记录；可加一条一槽游离记录；记账 A 取 0..9 槽；θ_high = t = 5。四个函数（10279 / 10336 / 10360 / 10391）逐字移植；`walk.rs` 的五条单测（25653、25673、25687、25698、25712）的同一批输入过移植，期望逐条对上（`out/q3_span_readings.txt` 末尾「walk.rs 单测同一批输入」五行都是「对上」）：窄 25653 与 25673 期望 (false, Some(true))，宽 25687 同，另有遍历走到而没记录的槽 25698 期望 (false, Some(false))，单元的记录整条不见 25712 期望 (false, None)。共 922 950 个盘面。I-3.18 红 = 有起点槽单元头用得了的未释放记录跨度 ≠ 那一类槽数；I-5.4 红 = 同盘任意两条记录罩住的槽相交（含已释放，`crates/singlefs-checker/src/walk.rs:9535` 起 `judge_allocation_record_ranges`）。**没跑真 checker**（不编 cargo）：下面「真 checker 上」的判定都是读码加已有测试断言推的。

### 两张取宽后成立的最小镜像

| | 窄：记录跨 1 槽罩 2 槽的单元 | 宽：记录跨 3 槽罩 2 槽的单元 |
|---|---|---|
| 真 checker 上的实例 | 新池新建文件那份镜像，数据单元（码 1，槽 50180，两槽）两盘上的记录改跨 1：已有测试 `an_unreleased_allocation_record_spanning_one_slot_of_a_two_slot_data_unit_reddens_only_the_span_invariant`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3660`），我读了断言没跑 | 同一份镜像，同一条记录改跨 3：单元区里 50180 的下一个单元是 50240：「(50240, 16384),」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:132`），50182..50239 没有任何单元，多罩的槽 50182 空着；**没有现成用例**，推的 |
| I-3.1 按记录自述 | 不成立：(P) 遍历走到的槽 50181 不在记录里；上沿少一槽 | 不成立：(L) 记账 2 槽 < 记录罩住 3 槽 |
| 取宽之后 | 成立 → **部分没判**，点名 I-3.18 | 成立 → **部分没判**，点名 I-3.18 |
| 红的不变量 | **只 I-3.18**（已有测试断言：除 I-3.18、I-3.1 之外每一条与干净镜像逐项相同） | I-3.18；I-5.4 绿（推的：没有别的记录罩 50182）；别的不变量没核，**复核不了**（没跑真 checker） |
| I-3.18 违例说明（`crates/singlefs-checker/src/walk.rs:19323` 的格式） | 「{版本名}：盘 0 槽 50180 的分配记录（未释放）跨 1 槽，而它罩住的单元头里是码 1，那一类占 2 槽」 | 「{版本名}：盘 0 槽 50180 的分配记录（未释放）跨 3 槽，而它罩住的单元头里是码 1，那一类占 2 槽」 |
| I-3.1 部分没判说明（`crates/singlefs-checker/src/walk.rs:21358`） | 「盘 0：遍历走到而不在记录里的槽 [50181]、记账对记录的上下沿配不上的那一块，全落在跨度段与起点单元那一类的槽数不符的未释放记录（I-3.18 判它）罩着的单元里：那几个单元真占几槽在两种数法之间说不准，记账对记录那三判没判」（测试断言含「遍历走到而不在记录里的槽 [50181]」与「（I-3.18 判它）」） | 同一格式，槽表是 **[]**（宽向没有遍历走到而不在记录里的槽），一个槽号都没有 |
| 说明说全了吗 | **说全**：I-3.18 给出跨度与类的槽数（少 1 槽），I-3.1 说明点名第二槽 50181；记账照单元（A = 2）时没有统计病，照记录（A = 1）时统计漏的就是这一槽 | **说全**：I-3.18 给出 3 对 2（多罩 1 槽）；记账照单元（A = 2）时统计没病，只有记录病。I-3.1 的说明里没有槽号也没有数——要靠 I-3.18 那一行读出多罩的是 50182 |

一处缺口，两个方向都有：I-3.1 的部分没判说明**不带记账的数与界**（原来判红的说明带记账的已分配、下沿、上沿的数，末尾列「遍历走到而不在下沿那一份里的槽」，`crates/singlefs-checker/src/walk.rs:21386`）。读者看不出记账是照单元记的还是照记录记的；窄镜像 A = 1（统计漏 50181）与 A = 2（统计没病）在说明上无差别，要自己去读记账行。

### 有没有真病被 I-3.18 / I-5.4 的红说明盖不到（小世界全枚举）

`out/q3_span_readings.txt`（922 950 个盘面）：I-3.1 取宽后（两个方向都改）holds 1 080 / **partial 16 021** / red 905 849；只放窄（`!=` 换成 `>`）holds 1 080 / partial 10 897 / red 910 973。取宽后 partial 的 16 021 个盘面里，I-3.18 无一例外是红的；按「改记的记录是哪个方向」分：

| 类 | 盘面数 | 记账 A 与真值 | 真病被 I-3.18 / I-5.4 说全了吗 |
|---|---|---|---|
| 纯窄，记账照单元（A_true）或照记录（A_rec），没有别的记录病 | 1 500 | 只有这两个值 | 说全 |
| 纯窄，A 严格介于 A_rec 与 A_true 之间（至少两条窄记录，记账只漏了其中一部分） | 226 | 两个值之外 | I-3.18 给出每条窄记录，漏几槽要自己加；介于的值本身没人说 |
| 纯窄 + 一条游离记录（起点槽没有单元头，其罩住的 1 槽补平了窄记录少算的 1 槽） | 4 052 | | **游离记录没人提**（它的补偿让下沿 / 上沿照样过）|
| 纯宽，没有别的记录病 | 2 631（其中多罩的槽另被一条记录罩着、I-5.4 红的 772） | 只有 A_true | 说全（I-3.18；有重叠时加 I-5.4） |
| **纯宽 + 邻居一槽单元没有记录**（宽记录多罩的槽恰是它） | 501（再加游离记录的 1 850） | 501 个里 A − A_true = −1 的 355、= 0 的 146 | **没人提邻居没有记录**，也没人提记账漏了它的 1 槽 |
| 窄宽混合 | 5 261 | | 含上面几种的叠加 |

- **宽向特有的一形**：单元 (起点 0, 1 槽)、(起点 1, 1 槽)，第二个单元的记录丢了，第一个单元的记录跨 2 槽（多罩的槽恰是第二个单元），记账少算第二个单元那 1 槽（A = 1，真占 2 槽）。按记录自述三判不成立（下沿 2 槽 > 记账 1 槽），取宽后成立 → I-3.1 部分没判，I-3.18 红，说明是「槽 0 的记录跨 2 槽而单元头里那一类占 1 槽」：说的是多罩，没提第二个单元没有记录、记账漏了它。`out/q3_span_readings.txt` 的「宽 邻居一槽单元没有记录 A=1」一行。修掉宽记录（改回 1 槽）之后，(P) 才会冒出「遍历走到而不在记录里的槽 [1]」——先后暴露，不是一次说全。
  - 这一形的前半（宽记录盖住了没有记录的邻居）**取宽之前就能藏**：记账照单元记（A = 2）时按记录自述三判就成立（「宽 邻居一槽单元没有记录 A=2」一行：自述成立、holds），在这份小世界里这样的盘面有 209 个；取宽新增的是 A = A_true − 1 那一档。
- **题面点的三种攻法逐个答**：① 宽记录多罩的槽被另一条记录合法罩住（I-5.4 红）：E 是按槽去重的集合（`counted_at`），记账照单元时 I-3.1 **自述就成立**，取宽没参与；记账把重叠数两遍（A = 3）时两个方向都判红（「宽 多罩的槽被邻居的记录合法罩住 A=2 / A=3」两行）——没藏。② 多罩的槽上有别的单元被遍历走到、(P) 两份至少一份含它就放过：就是上面那一形，**确实漏**（条件：邻居没记录、记账又少算那一槽）。③ 窄记录少罩的槽被别的记录罩住、统计多算：A = 2 时自述就成立，A = 3 时两个方向都红（「窄 少罩的槽被一条游离记录罩住 A=2 / A=3」两行）——没藏。

### 只放窄一个方向（10372 的 `!=` 改 `>`）够不够

- **以「宽向放过的真病里 I-3.18 说不到的」为尺子：够。** 宽向取宽额外放过的、I-3.18 说不到的只有上面「纯宽 + 邻居没记录」那一类（501 + 1 850 个盘面，只放窄时它们都是 I-3.1 红）；窄向自己留下的 4 052 + 226 两类，只放窄同样留着（`narrow_only=partial`），不因为不放宽向而少。
- **以「一个病不两处红」为尺子：不够。** 纯宽的 2 631 个盘面（含最小宽镜像 A = 2）在只放窄时变成 I-3.1 红加 I-3.18 红，两条红指同一个病；而 I-3.1 那条红的说明按下沿写（「下沿：checker 现算的门槛」，`crates/singlefs-checker/src/walk.rs:21385`，接着是上沿与遍历走到而不在下沿那一份里的槽）会读成「记账比记录少一槽」，其实记账是对的、错在记录。这不藏任何东西，是说明会误导。
- 宽向与窄向在安全上不对称：窄记录少罩的槽「在盘上没有归属、可被重新分配」（I-3.18 条文：「单元后面那几槽在盘上没有归属、可被重新分配」，`crates/singlefs-checker/invariants/05-空间记账.md:25`），是不安全的一侧；宽记录多罩的槽只是占着没人用的槽（泄漏，安全的一侧）。这是读条文得的，没有量。
- 真 checker 上构造「宽 + 邻居没记录 + 记账少一槽」的镜像：助手 `remove_allocation_record`（`crates/singlefs-harness/tests/checker_known_bad_images.rs:3719`）、`set_allocation_record_span`（775）加一处记账行改写可以造，**我没造、没跑**（不编 cargo），这一形在真 checker 上的判定是模型推的。

**什么现象会推翻 Q3 的结论**：① 真 checker 上「宽 + 邻居没记录 + A = A_true − 1」那张镜像的 I-3.1 不是部分没判（说明我的移植漏了一道判）；② 有另一条已实现的不变量逐单元核「遍历走到的单元一定有自己的未释放记录」且在这张镜像上红（那这一形就有人说，缺口缩小）——我只读了 I-3.1、I-3.15、I-3.18、I-5.4 的条文，没有把 113 条全读一遍；③ `out/q3_span_readings.txt` 末尾那五行不再是「对上」（移植漂了）。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q1 问 1 根槽写失败时实例停不停 | 造不出（推的） | 经分配器的发布失败即冻结、重发成功才建下一版（crates/singlefs-core/src/transaction.rs:1575、1545、1594）；零单元发布失败整个挂载返回错误（crates/singlefs-core/src/mount.rs:8103、8120）；前提：FUA 返回即持久，且「根槽写失败推进 txg」没实现（kb 23-…:409 排在后面；crates 零命中） |
| Q1 问 2 记录落了根没落 + 新实例 | 造得出，但丁不会错认（读码 + 模型推的） | 行 T = u 不是 u + 1；`u ≥ T` 在 crates/singlefs-checker/src/walk.rs:9033–9038 报认不出，I-7.9 部分没判；缝在今天的实现上是关着的 |
| Q1 问 2b 重建上限不低于真上限 | 不成立（量的，小模型） | 缝若被打开：S=4 高 3 246 / 等 9 024 / 低 1 097，S=8 高 89 / 等 9 593 / 低 454；低的让合法抬 F 被判红 |
| Q1 问 3 钉槽 p | 认不出，也不用（读码推的） | p 只管挂载时读坏 / 全零的槽、一条根只带一个槽；重建那段不读 p |
| Q2 简化 A 只认一层 | 单独放开 0 变化；只留它其余放开 +22.0 / +12.3 个点（S=4 / S=8，量的，模型） | 要和 B 一起才起作用；长寿池里是大头 |
| Q2 简化 B 行 T / 被抛弃 / 没有行 | 单独放开 −4.0 / −23.3 个点；只留它 +26.3 / +42.0（量的，模型） | 只量了「行 T」一种；单独放开只解开老一圈没有根的那类 |
| Q2 简化 C 第 0 代根 | 单独放开 −0.4 / −9.9；只留它 +0.9 / +16.4（量的，模型） | 年轻池（第一圈内的抬 F）才有，长寿池 0 |
| Q2 今天与全放开 | 31.7% / 54.0% 对 5.0% / 2.1%（量的，模型） | 第二轮 5.1% / 2.2% 与 93.0% / 94.2% 是没有这三处简化时的数；辩方一档越界抬 F 判红今天 73.9% / 80.3%，全放开 87.8% / 94.2% |
| Q3 窄镜像 | 只红 I-3.18；I-3.1 部分没判；说明说全 | 已有测试断言（crates/singlefs-harness/tests/checker_known_bad_images.rs:3660），没跑 |
| Q3 宽镜像 | I-3.18 红（推的：I-5.4 绿）；I-3.1 部分没判；说明说全，I-3.1 的槽表是空 | 没有现成用例，推的；别的不变量没核 |
| Q3 有没有真病说不全 | 有一形：宽记录盖住没有记录的邻居单元且记账少一槽（501 + 1 850 个小世界盘面） | 取宽之前就能藏的一形（209 个）被取宽扩了一档；窄向另有游离记录补偿（4 052）和记账介于（226） |
| Q3 只放窄够不够 | 以「I-3.18 说不到的真病」为尺子够；以「一病一红」为尺子不够 | 纯宽 2 631 个盘面会双红，红说明会误导 |

## 五、没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决。
- **没编 cargo、没跑真 checker、没跑 GPU、没跑任何入库测试**（派发令不许）：所有「真 checker 上的判定」是读码、读已有测试断言、小模型推的，分别标了。Q3 窄镜像是仓里已有的用例，我读了断言没跑；宽镜像与「宽 + 邻居没记录」是推的。
- Q2 的模型不是入库装置：没有读故障、没有树表单元被复用、没有被抛弃时间线上的记录与缺行的实例；journal 口径（辩方一档）比实现宽。数是下界，不是真 checker 的部分没判率。
- 没把 113 条不变量全读一遍：「真病没人说」只对着 I-3.1、I-3.15、I-3.18、I-5.4 四条条文说。
- 开工快照里两份 kb（`.claude/kb/checks-owed.md`、`.claude/kb/decisions/16-发布语义.md`）此刻的 sha256 与快照不同（别的会话改过），引用的 16-发布语义.md 行号是此刻的；crates 七份与快照逐份相同。
- 没读禁读清单里的两份文件。
- 草稿目录里的副本在交回前已删（见交回正文的 du）。

