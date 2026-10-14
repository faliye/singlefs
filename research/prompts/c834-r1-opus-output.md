# C834（择到的系统配置实例代号随盘序变） 第一轮攻方腿（Opus）报告（2026-10-15）

## 复跑命令与模型文件

模型目录 `research/prompts/c834-r1-opus-model/`。复跑（整条经内存包装，跑全量约 45 分钟挂钟（按本轮两遍各约 20 分钟估），31 线程）：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/c834-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c834-r1/repo \
  --work <新的草稿目录>
```

末行判结果：`name=rerun_compare aggregate_all=same aggregate_rollback=same`（与 `out/aggregate-all.txt`、`out/aggregate-rollback.txt` 逐行相同）。

| 文件 | sha256 |
|---|---|
| `.cargo/config.toml` | 891b14e98b472f513f13dea65ff5b2a7cf1429b97a29b2c488f06e90ee095fc5 |
| `.cargo/mutants.toml` | 6ea6e326095098250e41ab4ce358d280f9bbb5f0351c19a2981a69630e12d881 |
| `Cargo.lock` | 29a0614414dfae4d629fae09326f70ac3df19b3a1c425562252d22e9b493d388 |
| `Cargo.toml` | 138fb5e3b088a057d6f80a902c03b4724c12e06ec1bf3bdaf49fc7e7c07f438b |
| `out/aggregate-all.txt` | 79f56e4691a587b3eb1c1cbc926194d6d1455a8702f8afc87ed1e3097901cbab |
| `out/aggregate-rollback.txt` | b09f86642bfb1383f17c56d6cd9285a2ed9fe646bd22e3e52a2512ec58ed7e81 |
| `out/existing-tests-by-candidate.txt` | 60e0db1a5fd734dfe95f3ed28ce11e1c0d311216dd83086c524c4d2ab89d8e37 |
| `out/g1-clean-today-ordinals-0-8.txt` | 6eebda7d82b663f987a7478cab36d1a160c50ccec4d42ac7572bceb26bab2d29 |
| `patch/apply_candidates.py` | 3f048caeb9ec987f5aaf5398f2136b264d42cb8865c933496521ab79edad6007 |
| `src/main.rs` | 39f87501714236ef507e1be630394c402bf05e31eff9235e2a63667bfc8364c9 |
| `tools/aggregate.py` | 8a4f8c4495105d99a7204e19629763eb007162057dca3152affc76fc9f59d9ff |
| `tools/pieces.py` | e3ab30bffbdae4ce52b469c9cfc117b09765bf19faf99de18b083d95a450c76b |
| `tools/rerun.sh` | 9e84396930afd275053f07a3936d79456f3d0560759279bb487af5ca268387a4 |
| `tools/run-batch-faults.sh` | 62e9d2440c1c4900cea84ad11cbe23b4071c96622996ddc7b72ec375f48ad430 |
| `tools/run-batch.sh` | da13af4a4c76e04bdd2182f199a793c32eeb4351780ee2927301692c88885812 |

（`.cargo/` 两份是冻结副本 `.cargo/` 原样拷来的，编 RocksDB 要它的 `BINDGEN_EXTRA_CLANG_ARGS`。）

装置是什么：冻结副本的拷贝上用 `patch/apply_candidates.py` 只改那一判（`crates/singlefs-core/src/mount.rs` 的 `placements_in_the_accounts_of_newer_readable_roots` 里 `None if … <= root.instance` 那一支的条件）与它的调用处，换成环境变量 `C834_CANDIDATE` 选的六个候选之一；`today` 与冻结副本同一个条件（`chosen <= root.instance`），另外只多读系统配置槽与 R 两个指针所在的槽（只读）。原型 `src/main.rs` 用入库 harness 的搭建 `tests/common_unwitnessed_publish/mod.rs`（`#[path]` 引进，它再引 `tests/common_admission_pool/mod.rs`）：384 槽小盘、第一个文件、覆盖写 5 次、再录一次覆盖写 R=(1,9)（P=(1,8)），崩在 R 的根槽之后、轮换之前；再录取新实例的那次挂载 M1 的写流，按崩溃点重放的枚举域逐个崩溃状态（`singlefs_checker_tier::crash::crash_state_enumeration_tables` + `crash_facts::extract_flow_facts`，每段 `full_expansion`，原地覆写取三态），每个状态：跑池级 checker；再做下一次可写挂载 M2，两种交盘次序（`forward` = 盘 0、盘 1，`reversed` = 盘 1、盘 0）各一次，记那一判的结局、M2 写行那次发布的根落盘之前写掉了 R 的几份单元、那一刻池级 checker 判红的不变量。

以下的数全是**副本上的数**（攻方拷贝 + 攻方原型），不是入库装置上的数。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 |
|---|---|---|
| G1 现象坐实 | **打中（坐实）**：取号只落一块盘的崩溃状态，两种交盘次序那一判走不同支；放过那一支 M2 写掉 R 的 14 份单元，那一刻 I-2.1、I-4.8、I-7.2、I-7.4 红 | 量过（副本） |
| G1 另一形（不靠盘序） | **打中**：取号两盘都落了、之后一个字都没写（「取了号没写」），今天的判法两种次序都放过、同样写掉 R 的 14 份单元、同样四条红 | 量过（副本） |
| G2 候选「择到那一份」（今天） | 出局：随盘序变；「取了号没写」两序都放过→红；S2（下文）放过→红 | 量过（副本） |
| G2 候选「跨盘取最大」（外部审核） | 出局：取号只落一块盘稳定放过→红（两序都红）；「取了号没写」放过→红；S2 放过→红；回卷历史放过→红 | 量过（副本） |
| G2 候选「逐盘取最小」 | 出局：「取了号没写」两序都放过→红；S2 放过→红；取号回卷历史放过→红 | 量过（副本） |
| G2 候选「照 I-7.7 第 ② 句」 | 出局：池级 checker 实现的读法就是「各盘最大号里最小的那个」，与「逐盘取最小」同一个量，同上三形都红 | 量过「逐盘取最小」；「② 句读法 = 逐盘取最小」是读 checker 注释推的 |
| G2 候选「要求有带更大号的根或 journal 记录」 | 出局：S2（M1 隔离了 R 的单元、写了记录、写行根没落）放过→红；`urJ` 一形里 M1 的记录撕在 R 的记录槽上的那一个状态拒（该放过拒，且盘上原样、之后每次挂都这样拒） | 量过（副本） |
| G2 攻方自提「R 的两个指针槽里读出更大号的单元头」 | 出局：入库 `urJ` 用例红（该放过拒）；`urJ` 历史 4 个状态全拒。被攻过零轮 | 量过（副本） |
| G2 参照臂「一律拒」 | 不中「写掉单元」那一类；入库 `urJ` 用例红（该放过拒：代码注释里写的「再也做不成可写挂载」） | 量过（副本） |
| G3 口径 | **打中**：仓里至少四个不同的量——取号（全盘全槽最大并上根环）、池级 checker I-7.7（各盘最大里最小）、这一判（盘表第一块盘世代号最大那一槽）、取号之前逐盘核（每盘世代号最大那一槽）；回卷之后「每盘世代号最大那一槽」与「每盘全部槽最大」在同一块盘上就不同，量出它改结局 | 量过（回卷历史，副本）；处数是读代码数的 |
| G4 用例与变异 | **打中**：C834 那一行写的用例判据「两次判法必须相同」被「跨盘取最大」满足而照样红（两序都放过）；入库两份用例 25 条在今天、最大、最小、更大号记录四个候选下全绿，分不开它们 | 量过（副本） |

一句话：G2 列的候选与攻方自提的一个，每一个都在至少一段量过的历史上判错；能分开「写掉过」与「没写掉过」的那件事（R 的单元此刻还在不在），在那一判的时刻系统读不到——R 的账与映射正因为读不出才走到这一支（四句见第六节）。

## 一、G1：现象坐实

### 历史（每一步与许可它的那一句）

1. 起池、第一个文件、覆盖写 5 次，再覆盖写一次 R=(1,9)，崩在 R 的根槽落盘之后、系统配置轮换之前（入库搭建 `crash_image_with_the_root_persisted_and_the_rotation_lost`，R 没被系统配置见证）。
2. M1：取新实例的可写挂载，盘上原样（前缀 `clean`）；M1 择到 R、取号 2、写行、暖机。录下它的 38 次写、9 段。段 0 就是取号那两写（盘 0、盘 1 各一槽，同一段：中间没有屏障）：`name=c834_segment prefix=clean segment=0 writes=SystemConfigurationSlot@d0,SystemConfigurationSlot@d1`。取号逐盘写、两写之后才一道屏障（`crates/singlefs-core/src/transaction.rs:886` 「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」是号怎么算；逐盘写见同一份文件 `write_acquired_instance`），所以「只落盘 0」「只落盘 1」都是枚举域里的状态。
3. 崩在段 0 里：序号 2、5（盘 0 落了号 2、盘 1 还是 1）、序号 6、7（反过来）。
4. M2：同一组盘，R 的树表、中央映射树根与 R 那条记录读回全零（攻方 Y2 那一形的三样，入库用例 `crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs:257` 「fn a_newer_readable_root_whose_only_record_mapping_root_and_tree_table_read_back_zero_refuses_the_writable_mount_before_any_write(」那一条造的就是这三样）。R 被择、走读失败、系统配置没见证它，退回 P；R 是比所选那一版新、记录全读不出、映射现算不成的根，走到那一判。
5. 那一判读的量：`crates/singlefs-core/src/mount.rs:3124` 「system_configuration.quantities.journal_instance,」，即 `crates/singlefs-core/src/recovery.rs:1147` 「None => chosen = Some(best_on_device),」取的盘表第一块盘那一份（每盘取世代号大的那一槽：`crates/singlefs-core/src/recovery.rs:1139` 「if one.quantities.slot_generation > zero.quantities.slot_generation {」）。条件 `crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」`crates/singlefs-core/src/mount.rs:2079` 「<= root.instance =>」，不成立就 `crates/singlefs-core/src/mount.rs:2089` 「None => continue,」。

### 原样输出（`out/g1-clean-today-ordinals-0-8.txt`，候选 `today` = 冻结副本的判法）

```
name=c834_state prefix=clean candidate=today ordinal=2 segment=0 per_device=[(0, 2), (1, 1)] later_records=0 r_units_overwritten_in_state=0 state_red=- order=forward judge=chosen=2,verdict=pass m2=mounted:instance=3:effective=(1,8) writes=38 r_units_overwritten_before_row_root=14 r_root_in_ring_before_row_root=true red_before_row_root=I-2.1+I-4.8+I-7.2+I-7.4
name=c834_state prefix=clean candidate=today ordinal=2 segment=0 per_device=[(0, 2), (1, 1)] later_records=0 r_units_overwritten_in_state=0 state_red=- order=reversed judge=chosen=1,verdict=refuse m2=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords writes=0 r_units_overwritten_before_row_root=0 red_before_row_root=-
name=c834_state prefix=clean candidate=today ordinal=6 segment=0 per_device=[(0, 1), (1, 2)] later_records=0 r_units_overwritten_in_state=0 state_red=- order=forward judge=chosen=1,verdict=refuse m2=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords writes=0 r_units_overwritten_before_row_root=0 red_before_row_root=-
name=c834_state prefix=clean candidate=today ordinal=6 segment=0 per_device=[(0, 1), (1, 2)] later_records=0 r_units_overwritten_in_state=0 state_red=- order=reversed judge=chosen=2,verdict=pass m2=mounted:instance=3:effective=(1,8) writes=38 r_units_overwritten_before_row_root=14 r_root_in_ring_before_row_root=true red_before_row_root=I-2.1+I-4.8+I-7.2+I-7.4
name=c834_state prefix=clean candidate=today ordinal=8 segment=1 per_device=[(0, 2), (1, 2)] later_records=0 r_units_overwritten_in_state=0 state_red=- order=forward judge=chosen=2,verdict=pass m2=mounted:instance=3:effective=(1,8) writes=38 r_units_overwritten_before_row_root=14 r_root_in_ring_before_row_root=true red_before_row_root=I-2.1+I-4.8+I-7.2+I-7.4
name=c834_state prefix=clean candidate=today ordinal=8 segment=1 per_device=[(0, 2), (1, 2)] later_records=0 r_units_overwritten_in_state=0 state_red=- order=reversed judge=chosen=2,verdict=pass m2=mounted:instance=3:effective=(1,8) writes=38 r_units_overwritten_before_row_root=14 r_root_in_ring_before_row_root=true red_before_row_root=I-2.1+I-4.8+I-7.2+I-7.4
```

读法：`per_device` 是（盘，那块盘全部自证过的槽里最大的实例代号）；`state_red=-` 是 M2 之前这个崩溃状态本身池级 checker 不红、R 的单元一份不少；`r_units_overwritten_before_row_root=14` 是 M2 写行那次发布的根落盘之前，R 写出的 14 份单元写（7 个单元、两块盘各一份）全被 M2 写掉；那一刻 R 的根还在根环里（`r_root_in_ring_before_row_root=true`），池级 checker 判红 I-2.1、I-4.8、I-7.2、I-7.4。

### 结局

- 取号只落一块盘的 4 个状态（两个前缀各 4 个，`out/aggregate-all.txt` 的 `class=number_on_one_device_only candidate=today` 那几行）：交号大的那块盘在前，那一判放过、M2 做成、写掉 R 的单元；交号小的那块在前，拒在任何写之前（`writes=0`）。**同一个崩溃状态、同一组盘，只换交盘次序，结局在「拒」与「做成并写掉 R 的单元」之间翻。**
- 放过那一支之后的写违反的不变量：`crates/singlefs-checker/invariants/02-根环.md:11` 「回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效」（I-7.4：R 按 R 自己的实例表有效，它引用的块被重新分配了），与 `crates/singlefs-checker/invariants/02-根环.md:9` 「其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验」（I-7.2：R 是 S 里代号最大者，走不完）。I-2.1、I-4.8 两条红是 checker 同一刻一起报的，没逐条核它们的判据，只当旁证。
- 另一形，不靠盘序：序号 8 起的 16384 个状态（前缀 `clean`；前缀 `tree_table_and_record` 是 16387 个）取号两盘都落了、M1 之后只写了它自己的单元、没碰 R 的单元，今天的判法两种次序都放过、都写掉 R 的 14 份单元、都四条红（`out/aggregate-all.txt` 的 `class=number_taken_no_record_r_units_intact candidate=today` 两行，`states=16384`）。也就是说 C834 那一行担心的「放过」不只出在盘序不同时：**取号写全落、别的一个字都没写就崩**，号已经比 R 大，那一判照样当「那条根之后已有一次挂载取过号、写掉了它的单元」放过。代码注释的前提 `crates/singlefs-core/src/mount.rs:2070` 「那条根之后已有一次挂载取过号、写掉了它的单元、记录槽也被那次挂载的记录占了」在这一形上不成立：取过号，没写掉。

推翻条件：在入库装置上重造序号 2（或 8）那个状态、同样三段读回全零，M2 在两种次序下结局相同，或 M2 做成而写行根落盘之前 R 的单元一份不少，就推翻这一节。

## 二、G2：每个候选在哪段历史上判错

### 五段历史（都在冻结副本拷贝上录、全量枚举 M1 写流的崩溃状态）

| 代号 | 历史 | M1 时读回全零的段（前缀） | M2 时读回全零的段 | 那一刻 R 的单元 | 该判 |
|---|---|---|---|---|---|
| 甲 | 取号只落一块盘 | 无（`clean`）或 R 的树表与记录（`tree_table_and_record`） | R 的树表、映射根、记录（`y2`） | 一份不少 | 拒（或护住） |
| 乙 | 取号两盘都落了，M1 只写了它自己的单元、没碰 R 的、没写记录（「取了号没写」，序号 8 是一个字都没写的那一个） | 同上 | 同上 | 一份不少 | 拒 |
| 丙 | M1 看得见 R 的映射（只是树表与记录读不出），按 `C_NEWER_ROOTS` 从映射现算、隔离了 R 的单元，把写行记录写进 R 的记录槽，写行根没落就崩 | `tree_table_and_record` | R 的树表与映射根（`tree_and_mapping`；R 的记录槽已被 M1 的记录占了，不再藏） | 一份不少 | 拒 |
| 丁 | 入库 `urJ` 那一形：M1 时 R 的根槽、R 写出的全部单元、R 的记录读回全零，M1 看不见 R，写掉 R 的单元与记录槽，写行根没落就崩 | `root_units_and_record` | 无（盘上原样，`none`） | M1 已写掉（M2 之前这个状态本身就红） | 放过（拒了就只要盘不变每次都拒） |
| 戊 | 取号回卷：M1 取号两写之后那道屏障在盘 0 上报错，两盘回卷成旧号（`crates/singlefs-core/src/transaction.rs:914` 「⇒ 已写出的那几份回卷成旧代号」），M1 报失败；枚举这段写流的 17 个崩溃状态 | 无 | `y2` | 一份不少 | 拒 |

丁的前缀与入库用例 `urJ` 那一形同造法（`crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 文件头那一段写的「连 R 的记录也读回全零」）；丙是攻方新造的形。

### 结局（`out/aggregate-all.txt`、`out/aggregate-rollback.txt`，两种交盘次序结局逐格相同的只写一次；「放过·红」= 放过、M2 写行根落盘之前写掉 R 的 14 份单元、I-2.1 + I-4.8 + I-7.2 + I-7.4 红）

| 候选（`C834_CANDIDATE`） | 甲（每前缀 4 个状态） | 乙（16384 / 16387 个） | 丙（3 个） | 丁（4 个） | 戊（17 个里走到那一判的） |
|---|---|---|---|---|---|
| 择到那一份（`today`，今天） | **随盘序变**：每种次序 2 放过·红、2 拒 | **放过·红**（两序） | **放过·红** | 放过（对） | 回卷写全落的那一个（序号 16）拒（对）；回卷写没落全的 6 个放过·红 |
| 跨盘取最大（`largest`） | **放过·红**（4 个、两序都是） | **放过·红** | **放过·红** | 放过（对） | **序号 16 放过·红** |
| 逐盘取最小（`smallest`） | 拒（对） | **放过·红** | **放过·红** | 放过（对） | **序号 16 放过·红**；另 8 个放过·红 |
| 有带更大号的根或记录（`later_root_or_record`） | 拒（对） | 拒（对） | **放过·红** | 3 放过、**1 拒**（序号 16395：M1 的记录撕在 R 的记录槽上，两条都读不出） | 拒（对） |
| 攻方自提：R 的树表 / 映射根所在槽读出号更大的单元头（`pointer_slots_overwritten`） | 拒（对） | 拒（对） | 拒（对） | **4 个全拒** | 拒（对） |
| 参照臂：一律拒（`always_refuse`） | 拒 | 拒 | 拒 | **4 个全拒** | 拒 |

几行原样（`out/aggregate-all.txt`）：

```
name=c834_cell prefix=tree_table_and_record/tree_and_mapping class=number_taken_records_written_r_units_intact candidate=later_root_or_record order=forward judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=3 example_ordinal=16396
name=c834_cell prefix=root_units_and_record/none class=number_taken_and_r_units_overwritten candidate=later_root_or_record order=forward judge=refuse outcome=refused states=1 example_ordinal=16395
name=c834_cell prefix=root_units_and_record/none class=number_taken_and_r_units_overwritten candidate=pointer_slots_overwritten order=forward judge=refuse outcome=refused states=4 example_ordinal=16395
name=c834_cell prefix=root_units_and_record/none class=number_taken_and_r_units_overwritten candidate=today order=forward judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=4 example_ordinal=16395
```

（丁那一行的 `mounted_overwrote_r_units` 与红是 M2 之前就有的：丁的状态本身 `state_red=I-2.1+I-4.8+I-7.2+I-7.4`、`r_units_overwritten_in_state=14`，M2 放过没有让它更坏。）

`out/aggregate-rollback.txt` 原样（戊，回卷写全落的序号 16 在 `class=number_taken_no_record_r_units_intact` 里，因为分类按「每盘全部槽最大」，回卷之后每盘还有一槽是 2）：

```
name=c834_cell prefix=acquisition_rolled_back/y2 class=number_taken_no_record_r_units_intact candidate=smallest order=forward judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=9 example_ordinal=8
name=c834_cell prefix=acquisition_rolled_back/y2 class=number_taken_no_record_r_units_intact candidate=today order=forward judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=6 example_ordinal=8
name=c834_cell prefix=acquisition_rolled_back/y2 class=number_taken_no_record_r_units_intact candidate=today order=forward judge=refuse outcome=refused states=3 example_ordinal=10
```

### 逐个候选

- **跨盘取最大（外部审核）**：C834 那一行推的「会让它稳定走放过那一支」（`.claude/kb/checks-owed.md:555` 「外部审核建议的「跨盘取槽代号最大」会让它稳定走放过那一支」）量到了：甲两序都放过，而那一刻 R 的单元一份不少，放过就写掉。它对盘序无关，但无关在错的那一边。
- **逐盘取最小**：甲上对；乙、丙上错（两盘都落了号，最小也比 R 大）；戊上错：回卷之后每块盘上都还有一槽号 2（世代号小的那一槽），「每盘全部自证过的槽取最大」再取最小仍是 2。
- **照 I-7.7 第 ② 句**：池级 checker 的实现把 ① ② 两句合成一个量，`crates/singlefs-checker/src/walk.rs:4191` 「② 各盘不等时较大者不出现在它们里」那一行接着写的就是「各盘最大号里最小的那个」；拿它当那一判的凭据就是「逐盘取最小」，同样在乙、丙、戊上错（推的：没单独实现这一臂，靠与「逐盘取最小」同一个量）。把第 ② 句读成「较大者出现在根、记录或单元写序里才算取过号」时，丙里 M1 的记录与写行单元都带号 2，照样放过·红（推的，没实现）。
- **要求有带更大号的根或 journal 记录**：丙打它：带号 2 的记录在，R 的单元却一份不少（M1 当时从映射现算、隔离了它们）。丁的序号 16395 也打它：M1 那条记录写在 R 的记录槽上、撕了，两条都读不出，号 2 在盘上只剩系统配置里，它拒；盘上原样不变，之后每次可写挂载都这样拒。
- **攻方自提的指针槽读法**（被攻过零轮）：甲、乙、丙、戊都对，丁 4 个状态全拒，入库 `urJ` 用例因此红（第四节）。原因没查到底：推测 M1 写到 R 的树表 / 映射根那一槽的是一个 32 KiB 单元的后半截，槽首没有单元头可解（推的，没核）。

推翻条件：任一行在入库装置上重造后结局与上表不同（例：丙那 3 个状态里「有带更大号的根或记录」拒、或放过之后 R 的单元一份不少），那一格作废。

## 三、G3：仓里读实例代号的口径不是同一个量

`grep -rn 'quantities.journal_instance\|\.journal_instance\b' crates/singlefs-core/src/ crates/singlefs-checker/src/` 在冻结副本上命中的读处（写处、转交处略），逐处读了代码：

| 处 | 读的量 | 出处 |
|---|---|---|
| 取号算号 | 全部盘、全部自证过的槽取最大，再与根环里全部根取最大，加一 | `crates/singlefs-core/src/transaction.rs:886` 「let highest_on_disk = highest_system_configuration_instance(pool).max(highest_root);」 |
| 取号回卷写成的「旧代号」 | 同上那个最大（不并根环） | `crates/singlefs-core/src/transaction.rs:1029` 「let previous_instance = highest_system_configuration_instance(pool);」 |
| 池级 checker I-7.7 | 每盘全部自证过的槽取最大，再跨盘取最小 | `crates/singlefs-checker/src/walk.rs:4191` 「② 各盘不等时较大者不出现在它们里」 |
| 取号之前逐盘核（落后支） | 每块盘自己世代号最大的那一槽，逐盘各判 | `crates/singlefs-core/src/mount.rs:5436` 「instance_generation: newest_self_verified.quantities.journal_instance,」（`crates/singlefs-core/src/mount.rs:5485` 同一句） |
| 这一判 | 盘表第一块交得出的盘上世代号最大的那一槽 | `crates/singlefs-core/src/mount.rs:3124` 「system_configuration.quantities.journal_instance,」 |

两种不同在量上分得出：

1. **跨盘怎么合**：第一块（这一判）/ 取最大（取号）/ 取最小（checker）。甲那几个状态上三者是 2 / 2 / 1 或 1 / 2 / 1。
2. **一块盘里取哪一槽**：世代号最大那一槽（这一判、落后支）/ 全部槽取最大（取号、checker）。回卷之后同一块盘上两者不同：取号写进较旧那一槽（号 2，世代号 g+1），回卷写进另一槽（号 1，世代号 g+2）。戊的序号 16 上今天的判法读到 1（拒，对），「逐盘取最小」与「跨盘取最大」读到 2（放过·红）——量过，见第二节。

所以 G3 的答案不是「改成与别处同一个量」：别处的三个量里，两个（取号的、checker 的）在戊上让这一判放过·红，一个（世代号最大那一槽）就是今天这一判的读法、在甲上随盘序变。哪个量都不是「那条根之后已有一次挂载写掉了它的单元」的凭据（第六节第二句）。

推翻条件：在入库装置上回卷之后读回的「每盘世代号最大那一槽」与「每盘全部槽最大」相同（例：回卷写的不是另一槽），第 2 条作废。

## 四、G4：会红的用例与变异

1. **C834 那一行写的用例判据挡不住「跨盘取最大」**：那一行要的是 `.claude/kb/checks-owed.md:555` 「两种交盘次序各做一次可写挂载，两次判法必须相同」。「跨盘取最大」在甲上两种次序都放过、都写掉 R 的 14 份单元——判法相同，用例绿，盘红。用例要断言结局本身：两种次序都在任何写之前拒（或都做成而写行根落盘之前 R 的单元一份不少），只断言「相同」不够。
2. **只造甲不够**：乙（取了号没写，序号 8）上今天的判法两序一致地放过·红，C834 那一行的用例造不出它；「逐盘取最小」过得了甲、过不了乙。用例至少要甲、乙两形，乙不靠盘序。
3. **入库用例分不开候选**：入库两份（`writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs` 16 条、`recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 9 条）在副本上逐个候选跑（`out/existing-tests-by-candidate.txt` 原样）：

```
candidate=today
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
candidate=largest
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
candidate=smallest
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
candidate=later_root_or_record
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
candidate=pointer_slots_overwritten
test a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts ... FAILED
test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
candidate=always_refuse
test a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts ... FAILED
test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

   红的那一条停在 `crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs:131` 「.expect("可写挂载退回 P、建得出上一版");」，拒因 `AccountOfAnUnappliedPublishWithIncompleteReadableRecords`（R=(1,9)）。今天、最大、最小、更大号记录四个候选在这 25 条上一条不红：哪个候选改进来，入库用例都看不出它在甲、乙、丙、戊上的差别；Y2 那一条（第 257 行那个函数）只造了「没人取过号」的形，四个候选都拒。
4. **变异「那一判改回读择到那一份」**：要靠用例里甲那一形的两种次序各断言拒才红（量过：`today` 在甲上一序放过）。另要一条「改成跨盘取最大」的变异，靠同一条用例（甲上两序都放过·红）红；「改成逐盘取最小」的变异要靠乙那一形红（量过：`smallest` 在乙上放过·红、在甲上拒）。
5. **新条款对已有入库用例改不改结局**：上表那四个候选不改（25 条全绿）；凡是在丁上拒的（攻方自提那一臂、一律拒）改：`urJ` 那一条从「做成」变成「拒」。C694 的 B 那几条（`abandoned_root_whose_account_is_unreadable_passes_when_the_current_account_records_later_units_over_its_slots.rs`）没在候选下跑过（没做，见末节）。

推翻条件：入库装置上同样换候选跑这两份，四个候选里有一个红，第 3 条作废。

## 五、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

1. **分不分辨臂**：每一格单看都分辨：甲分开「今天、跨盘取最大」与其余；乙分开「今天、最大、最小」与其余；丙只留下「指针槽」与「一律拒」；戊分开「最大、最小」与其余；丁反过来分开「指针槽、一律拒、更大号记录（1 个状态）」与其余。合起来没有一个候选在五段上全对——全部出局的那一种，病根在共用前提里：**「拿一个号（或带号的东西在不在）判 R 的单元写没写掉」**。丙与丁两段上系统配置都是 2 / 2、都有（或都可能有）带号 2 的记录，一段 R 的单元一份不少、一段全被写掉。这一格要另立一笔账先修，不拿它判哪个「号的读法」更好。
2. **系统当时看不看得到判别它的东西**：要分的是「R 的单元此刻还在不在」。走到这一支的前提就是 R 的账读不出、映射现算不成，系统不知道 R 的单元在哪，号与记录又在丙、丁上逐条相同 ⇒ 对四个按号或按记录判的候选是「判别子观测不到」，照那一行办：改判据，改完是新一轮。系统只剩 R 根记录里两个指针（树表、映射根）的落点读得到：丙上那两槽读回全零（读故障），丁上那两槽是 M1 写下的字节——原则上分得开，攻方自提那一臂就是想读这个，但量到它在丁上 4 个状态全拒（原因没查清，第二节末）。
3. **满足的是判据字面的哪一个分句**：跑前条款「某候选被造出判错的历史 ⇒ 那个候选出局」，观测一栏写「候选在其上判出与不变量相反的结局」。甲、乙、丙、戊上的放过·红满足它：放过之后 M2 写行根落盘之前 R 的单元被写掉，I-7.4、I-7.2 红。**丁上的「拒」不满足它**：拒了不违反哪条不变量，是可用性代价（代码注释里写的「拒了就再也做不成可写挂载」，入库 `urJ` 用例为它而设）。照字面：今天、跨盘取最大、逐盘取最小（连同 I-7.7 第 ② 句那个量）、有更大号的根或记录，四个出局；攻方自提的指针槽一臂与「一律拒」参照臂照字面站得住，代价都在丁上。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：逐格见第二节表。在甲、乙、丙、戊（写掉单元那一类）上都不中的只有指针槽一臂与一律拒；两者在丁上都中（该放过拒）。没有一个改法在五段上都不中。

## 六、改法与各修哪一格

自提的改法只在攻方模型上量过、被攻过零轮。

| 改法 | 甲 | 乙 | 丙 | 丁 | 戊 | 入库 25 条 |
|---|---|---|---|---|---|---|
| 指针槽：R 的树表或映射根所在槽读出号比 R 大的单元头才放过（`pointer_slots_overwritten`） | 修（量过） | 修（量过） | 修（量过） | 不修，4 个全拒（量过） | 修（量过） | 1 条红（量过） |
| 一律拒（删掉「比它新就不交也不拒」那一支，`always_refuse`，走记录不齐那一支、重读一次仍不成拒；`crates/mutations.tsv` 里「删除线第38批 C_MAPPING 较新可读根一条记录都读不出、映射现算不成也拒」那一行意思相近、写法不同：它在那一支直接报 `AccountOfAnAbandonedRoot`、不重读） | 修（量过） | 修（量过） | 修（量过） | 不修，4 个全拒（量过） | 修（量过） | 1 条红（量过） |
| 指针槽改读法：那两槽读出来不是全零、又验不过 R 指针里记的单元校验和，才当写掉过 | 修（推的） | 修（推的） | 修（推的：丙那两槽读回全零） | 修（推的：丁那两槽是 M1 的字节） | 修（推的） | 没跑 |
| 一律拒 + 给管理员一条显式出口（承认那条根作废之后再挂） | 修（推的） | 修（推的） | 修（推的） | 用户动作换掉自动放过（推的） | 修（推的） | 没跑 |

指针槽改读法自己的软处（推的，没造）：「读回全零」是本仓的读故障模型；真盘上同一槽被位翻转、撕裂写坏而 R 的单元其实还在别处，它会当成「写掉过」放过——丙的一个变体（M2 时那两槽读回的是坏字节而不是全零）就会打它。

## 七、没打中的形状与取样范围

- 四个前缀 × 6 个候选 × 两种次序，M1 写流的崩溃状态全量：`clean` 20511 个、`tree_table_and_record` 20516 个（两种 M2 读故障各一遍）、`root_units_and_record` 20516 个、取号回卷 17 个；每个前缀 9 段（回卷 2 段），段 1 是 M1 写行那次发布的 14 次单元写（16383 个子集状态）。
- 走不到那一判的状态（每个前缀约 4116 个：M1 写行根已落到一块盘上之后，R 被 M1 的实例表抛弃，走被抛弃根那一判，拒因 `AccountOfAnAbandonedRoot` 或做成而 R 的单元一份不少）：这一轮的候选碰不到它们，没打中。
- 前缀 `clean` 里 M1 的记录已落的 3 个状态：那一判放过，但 M2 没写掉 R 的单元、不红（M1 是在 R 上建的，R 的单元记在现行账里）。
- 只试了：两块盘；384 槽小盘一种几何；R 是一条记录的覆盖写（没试三条记录的顺序写）；覆盖写次数 k = 5 一个值；M2 的读故障三种（`y2`、`tree_and_mapping`、`none`）；取号回卷只试了「取号两写之后那道屏障在盘 0 报错」一种（整池第 3 次设备屏障调用）。

## 八、这条腿自己的限度

- 全部是副本上的数：攻方拷贝的 `crates/` 上装了候选开关（`today` 与冻结副本同一个条件，多出的只是只读的读盘与一行记录），原型是攻方写的。要引进判决，得在入库装置上重做（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。
- 「读回全零」是本仓的读故障模型（入库用例同一个包装 `DeviceReadingBackAllZeroOnRanges`）；M1、M2 两次挂载各藏不同的段，这一形可达性的前提是读故障在两次挂载之间会变——与入库 `ur` / `urJ` / Y2 那几形同一个前提。
- M2 只看了「写行那次发布的根落盘之前」那一刻与 M2 的结局，没有再枚举 M2 自己写流的崩溃状态；M2 之后的用户动作没扫：打中的那一写发生在 M2 写行那次发布里，在任何用户动作之前。M1 里没有用户动作（它是一次挂载），前缀的不同只是读故障落在哪。
- 状态分类按「每盘全部自证过的槽取最大」，回卷历史里回卷写全落的状态因此被分进「取号两盘都落了」一类（第二节已注明），结局按序号逐个看过。
- I-2.1、I-4.8 两条红没逐条核判据，只当旁证；论证只靠 I-7.4、I-7.2。
- 攻方自提那一臂在丁上为什么全拒，没查到底（推测见第二节末）。

### 取样、估时与分批

- 起跑前 `ps -o pid,args -u "$(id -u)"`：别的会话有一条 `cargo test --release -p singlefs-harness --test probe_gpu_table_counts --no-run` 在跑，没有性能测量，加 `nice -n 19` 照跑。
- 估时：先跑一小段（`clean run 0 20511 1000`，21 个状态、两种次序）挂钟 2.22 秒，去掉起池与 M1 的 0.29 秒，约 0.09 秒一个状态（两次 M2 + 两次 checker）；`y2` 读故障那一遍全量 6 × 41027 = 246162 个状态 ≈ 22600 CPU 秒，29 件并行估 13 分钟挂钟，不到 40 分钟，没缩。
- 状态数：`y2` 那一遍 246162、丙丁那一遍6 × 2 × 20516 = 246192、回卷 6 × 17 = 102，合计约 49 万；每次跑之前用 `crash_state_count_with_torn_in_place_overwrites` 算过一段历史的全量（20511、20516、17），都不超过约 10⁶。
- 分批（每批不带限时，`.rc` 文件数与件数逐批对上，退出码全 0：`y2` 那一遍 259 个、丙丁那一遍 276 个）：`y2` 那一遍开头一批 29 件 × 1800 个状态挂钟 252.71 秒（超过前台 240 秒，被工具挪进后台，等完成通知、没写轮询），之后改成 230 件 × 900 个状态分 8 批（29 件一批，末批 27 件），挂钟 123.33、119.35、126.43、115.39、49.96、121.12、115.32、110.60 秒；丙丁那一遍 276 件 × 至多 900 个状态分 10 批，挂钟 111.67、114.31、110.97、112.28、114.17、147.98、116.95、168.67、118.94、101.94 秒；回卷 6 次各 17 个状态，一次跑完。

## 九、没做什么

- 没在入库装置上跑；没跑 checker 档测试、门禁阶段、轻型三条，没起 GPU。
- G5（落在 D22 已定项 8、D23 已定项 14 哪一处）不在攻方分工里，没做。
- C694 的 B 那几条入库用例没在候选下跑。
- 没试三条记录的顺序写 R、别的 k、别的几何与盘数、别的回卷时机。
- 没给攻方自提的指针槽改读法写实现（第六节第三行全是推的）。
- 原型只跑在自己造的流上；搭池用的是 harness 的 `tests/common_unwitnessed_publish/mod.rs`（它引 `tests/common_admission_pool/mod.rs`），这两份不是崩溃枚举用例文件，但后者被登记的崩溃枚举用例 `crash_enumeration_floor_raise_pushed_by_admission.rs` 引着（它自己文件头写的），这里写明给主 agent 判。

## 十、草稿与产物去向

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/c834-r1-opus/`：冻结副本拷贝（`tree/`、`tree-candidates/`）与编译目录（`target/`、`target-tests/`）交回前已删；留下 `runs/`（逐状态原样输出约 371M，模型目录 `out/` 只收了汇总与摘录，原样行太大不入库）、`model/`（与模型目录同一份）、几份编译与测试日志。
- 入库的产物只有本报告与模型目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c834-r1-opus-output.md --root /home/fy5090/code/singlefs` 末行：

```
  ✓ 核了 22 处引文、0 处裸行号，对上 22 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
