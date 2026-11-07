# 逐写独立判 第三轮 攻方腿报告（Opus，2026-11-06）

批次：records/2026-11-05-删轻型并入重型.md#并包。立场：攻方。攻击面：第三轮正文第一节那一件（E6 在 16 段上的读数是误报还是真点名），连同派发提示「关的是哪几条」的三条。
没读禁读清单里的文件。草稿目录 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r3-attack`（认领过）。
标注：「量的」= 第二轮产物里现有的行（任务 8700），我只读、用命令数；「算的」= 我对量的数做的算术；「推的」= 读代码推的、没跑；「待主 agent 跑」= 本腿写好的探针还没跑出的读数。本腿一条命令都没交服务。

## 各格判定一览

| 格 | 判定 | 依据（详见各节） | 状态 |
|---|---|---|---|
| 1. 16 段 E6 读数 2：误报还是真点名 | 预测 16 段全是宽度误报：第二轮探针把每个点名位置按 32768 字节罩，码 2 节点读者只读 16384；E2 在 24 段全是 0，罩中的只能是被点名节点后一个槽上的新写 | 第一节 | 推的，待主 agent 跑 P1–P3 |
| 2. 哪怕一段真点名、且乙与全组合不同的状态 | 没打中（攻方想打穿「误报」那一边，没造出今天的流上的真点名） | 第一节、「没打中的形状」 | 推的；探针在真点名段上自动开宇宙真判 |
| 3. 收窄后的 E6（新名 E7，按点名项类标签取读者真读的宽）放不放过反例 B | 不放过：反例 B 的记录点名的正是它前面那个单元的起点槽，任何宽度都罩中 | 第二节 2.2 | 推的，臂上待主 agent 跑 P4、P5 |
| 4. E6 与 E7 都不看记录新根段的四条指针 | 打中判据字面缺一个分句（推的）；今天的流上预测读数 0，不改态数 | 第二节 2.3 | 推的，待主 agent 跑 P6 |
| 5. 收下的段上逐态全量核对（不取样） | 命令给了（现有 5 个崩溃点重放项，全量展开）；但在今天干净的实现上这项核对分辨不了乙与「只判 S=∅」，核过也不证乙对 | 第二节 2.4 | 打中方法层（推的 + 第二轮 9 个宇宙全绿是量的）；退化候选那一列待主 agent 跑 P7 |
| 6. E7 下四条流的态数 | 471、355、378、404（与第二轮辩方算的同数），条件：E7 收下全部 24 段，且 E5 照辩方当成满足 | 第二节 2.5 | 算的（段表是量的），待主 agent 跑 P3 |
| 7. 第二轮「约 6%」 | 核得出：1 − 1379926285 / 1463812286 = 5.73% | 第三节 | 量的 + 算的 |
| 8. 第二轮「91 对里 78 对有差」「243/1456」 | 核得出；限定：量它的代码是并合成，91 对的空集态全绿，所以与增减合成同数 | 第三节 | 量的 |
| 9. 第二轮「判据之内只在段 20 量过 3 次共 3072 态」 | 核得出；限定：这 3072 态里实际红的态数产物没打，三个计数器全 0 只说明合成与真判一致 | 第三节 | 量的 |
| 10. 第二轮复跑汇总漏行 | 核得出：四份分产物含 PROBE_ 的行 622 行，汇总 out.txt 618 行，漏 4 行（每个测试第一行） | 第三节 | 量的 |
| 11. 背景材料附录 | 对不上：附录抄的是 kb 实验 E6（加密算法选型）与 E1–E4，与本轮判据 E1–E6 同号不同物；本腿不引附录 | 「这条腿自己的限度」 | 量的（读材料） |

## 复跑命令与模型文件

模型目录 `research/prompts/per-write-judging-r3-opus-model/`。探针是三段文本，追加在冻结副本两份测试文件末尾（与第二轮辩方探针同一种做法）：

| 文件 | 行数 | sha256 | 做什么 |
|---|---|---|---|
| `probe/r2_tightened_classifier.rs.in` | 369 | `2fec45005c2e2c105612102dac7f4c71c684ab8bbfb19205d548a9eddac2aec2` | 第二轮辩方的分类器原样拷来（与 `research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in` 同 sha256），E1–E4 与组数照它判、不另写 |
| `probe/e6_readout_common.rs.in` | 367 | `c37577c004b76b2f68c4c3dcfee27bd82a9b25a786acfae0e43a3767f5a38215` | 逐段逐记录读出 E6 点名：实现自己的读者 `JournalRecord::parse` 解点名项，第二轮手写字段表再解一遍逐条比；四种宽度各数一遍；收窄后各流态数 |
| `probe/e6_readout_unlink.rs.in` | 265 | `a4da873a839aa51469e1cc1303564bfad9f463ec587ac3311532b32b6ce3823e` | 删除流与两条写错次序的臂（record_before_units、late_unit）；小宇宙逐态真判、与乙·增减、乙·并、只判 S=∅ 三种合成比 |
| `probe/e6_readout_pairing.rs.in` | 31 | `1648736af6a9085b770e523c9e5ea512dcbd7d040c4ca23d11ca5382cf83aef3` | 对拍三条流，只读不判 |
| `tools/rerun.sh` | 213 | `f1551c0eaca1057776cb5d0976353153d2d22124615ede85a631381b74212133` | 拷自 `scripts/agents/three-way-rerun-template.sh`，只改 `rerun_steps`（前 79 行与后 80 行与模板逐字相同，`diff` 核过） |

冻结副本：`/home/fy5090/.local/share/singlefs/work/per-write-judging-r3-attack/frozen`（`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs <它> --with-test-inputs` 拷出，开工那一刻的工作区，50M；`.deliverables` 写成「交接」）。两份测试文件与仓里逐字节相同（`cmp` 核过），里面没有 `per_write_judging` 字样。

`rerun.sh` 怎么接得上重排：模板要 `--work` 是空目录，所以 `--work` 只占位，进度与产物落在 `<--work>-state`；每一步写 `.partial`、做完才改名，再起时做完的步跳过。汇总不用 `grep -h '^PROBE_'`，改成 `grep -o 'PROBE_E6_[A-Z_]* .*'`，并核「取出的行数 = 含 `PROBE_E6_` 的行数」、两份都要有 `PROBE_E6_DONE` 行，对不上退 1。最后才把产物拷进 `--work`。
探针不调任何枚举函数：每段态数用 `crash_state_count_with_torn_in_place_overwrites` 按段算；真判只在小宇宙上，跑前打 `PROBE_E6_PLANNED_STATES total=`（预计 5120，真点名段每段再加 1024），超过 10^6 断言失败。

## 跑前写死的预测（跑出来之后不改，只往下另记）

读法：「段」是写表段号；四条真实流的标签照探针打的 `label=`：`unmodified`（删除流）、`second-deletion-batch-quarantined-data-unit-records-carry-the-mark`、`three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first`、`deletion-through-the-last-batch`。

| 号 | 预测 | 看哪几行 | 什么读数推翻它 | 推翻了我怎么改 |
|---|---|---|---|---|
| P1 | 删除流 7 个单元段的 `r2_classifier_e6`、`r2_e6`、`wide_e6` 三列逐段等于第二轮读数（段 8、20 为 0，段 12、16、24、28、32 为 2）；四条真实流 24 段 `parse_mismatches=0`、`unparsed_records=0` | `PROBE_E6_SEGMENT` | 任一段三列不等，或有解码不一致 | 三列不等：探针没复现第二轮，先查写表是不是同一张，后面各条全部作废；解码不一致：两条解码路有一条错，按实现的读者那一条（`wide_e6`）判，第二轮读数作废 |
| P2 | 四条真实流上每一行 `PROBE_E6_HIT`：`source=named`、`named_class=2`、`bytes_read=16384`、`gap_slots=1`、`overlaps_under_the_class_width=false`、`exact_start=false`、`named_region_written_earlier_at=Some(…)` 且 `earlier_class=Some(2)`、`earlier_birth_txg` 等于 `named_birth_txg`、`same_unit_as_the_write=false`；每段读数 2 的那两个记录写是同一条记录的两盘副本（`record_counter` 相同、`record_device` 一个 0 一个 1） | `PROBE_E6_HIT` | 任一行 `overlaps_under_the_class_width=true` 或 `gap_slots` 不是 1 | 那一段改判真点名（第一节表那一行翻面），用它的 `PROBE_E6_SYNTH label=unmodified-truly-named` 行判乙与全组合差不差 |
| P3 | 24 段 `class_width_e6=0`、`exact_start_e6=0`、`pointer_e6=0`，16 段 `judged=width_false_positive`、8 段 `judged=no_reading`，24 段 `narrowed_passes=true`；`PROBE_E6_FLOW` 的 `reduced_total_narrowed` 四条流依次 471、355、378、404，`reduced_total_r2` 依次 1379926285、1094713579、1090650353、1111490806，`engine_total` 与 `summed_by_segment` 都等于 1463812286、1178599580、1107689646、1195376807 | `PROBE_E6_SEGMENT`、`PROBE_E6_FLOW` | 任一段 `narrowed_passes=false`，或四个总数有一个不等 | 那几段照「现在」的全量态数算，第二节 2.5 的态数换成探针打的数 |
| P4 | record_before_units 段 12：`r2_e6=6`、`class_width_e6=4`、`exact_start_e6=4`、`narrowed_passes=false`；`exact_start=true` 的 `PROBE_E6_HIT` 行 `same_unit_as_the_write=true`、`named_region_written_earlier_at=None` | `PROBE_E6_SEGMENT label=record_before_units segment=12` | `class_width_e6=0` 或 `narrowed_passes=true` | E7 放过了反例 B，第二节 2.2 撤回，E7 不能用 |
| P5 | late_unit 段 16：`r2_e6=4`、`class_width_e6=2`、`exact_start_e6=2`、`narrowed_passes=false` | `PROBE_E6_SEGMENT label=late_unit segment=16` | `class_width_e6=0` | 同 P4 |
| P6 | 新根段指针那一路：四条真实流 24 段 `pointer_e6=0`；record_before_units 段 12 `pointer_e6` 至少 2（B 那次发布末条记录的新树表指针指着段 12 里写的节点） | `PROBE_E6_SEGMENT` 的 `pointer_e6` | 真实流上非 0，或臂上为 0 | 真实流非 0：E7 那一个指针分句在今天的流上就拦段，态数换成探针打的；臂上为 0：指针那一路被点名项罩住，2.3 只剩字面缺口、没有量到的形状 |
| P7 | 删除流段 12、16 各一个宇宙（5 组 10 写 1024 态）：`empty_set_red=false`、`actual_red_states=0`，六个差数全 0，其中 `empty_set_only_items_differ=0` | `PROBE_E6_SYNTH label=unmodified-rejected-in-round-two` | `actual_red_states>0` | 2.4 的「判别不了」只在实际全绿时成立；有红态就看三种合成谁差，2.4 改写 |
| P8 | late_unit 段 16 的宇宙：`empty_set_red=true`、`group_delta_items_differ=0`、`group_union_red_differs>0`、`empty_set_only_red_differs>0`。record_before_units 段 12 的两个宇宙不预测红绿，只记 | `PROBE_E6_SYNTH label=late_unit-universe0`、`label=record_before_units-universe*` | `group_delta_items_differ>0` | 乙·增减在反例 A 一对上也有差，第二轮「乙·增减一对不中」（取样）被推翻 |

`PROBE_E6_PLANNED_STATES` 预测 `universes=5 total=5120`（P2 有段翻面时每段多 1 个宇宙、1024 态）。两份测试各有一行 `PROBE_E6_DONE`。

## 一、E6 在 16 段上：误报还是真点名

### 1.1 推导（正推，推的；判决要的读数在 P1–P3）

1. 第二轮辩方的 E6 字面就写死了 32768：`research/prompts/per-write-judging-r2-sonnet-output.md:97`：「每条 `JournalRecord` 写点名的（盘身份，槽号）都不与 W 里任何写罩住的范围相交（点名单元按 32768 字节算）」；探针照写：`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:248`：「let named_end = named_start + DATA_UNIT_BYTES_OF_THE_POOL;」。
2. 实现里读点名单元的三处都按点名项的类标签取宽，码 2 节点只读 16384：恢复 `crates/singlefs-core/src/recovery.rs:904`：「fn unit_bytes_for_class(unit_class: u8) -> Option<usize> {」、`crates/singlefs-core/src/recovery.rs:909`：「UNIT_CLASS_INDEX_NODE => Some(usize::try_from(NODE_BYTES).expect("16384")),」，验点名单元读的就是这个宽 `crates/singlefs-core/src/recovery.rs:5356`：「fn copy_named_by_the_location_entry_verifies<Reader: PoolReader + ?Sized>(」；可写挂载的隔离按同一张表、不认识的码取两槽 `crates/singlefs-core/src/recovery.rs:917`：「pub(crate) fn span_in_slots_of_a_named_unit(named: &crate::journal::NamedUnit) -> u64 {」；池级 checker `crates/singlefs-checker/src/walk.rs:13203`：「let unit_bytes = match MappingKeyUnitClass::of(named.unit_class) {」。位置条目的槽是 16 KiB 槽：`crates/singlefs-core/src/pointer.rs:33`：「/// 位置条目：设备身份 4 + 16 KiB 槽号 6 + 密文校验和 4（加密关时是整单元 CRC-32C）。」
3. 24 个真实单元段上 E2 两列全 0（量的，第二轮产物，本腿用命令数）：

```
$ grep -ohE 'PROBE_TIGHT_SEGMENT label=(unmodified|second-deletion[^ ]*|three-unit[^ ]*|deletion-through-the-last-batch) segment=[0-9]+ .*' out-classifier-and-arms.txt out-pairing-flows.txt | grep -v e1_only_unit_writes=false > …/real-unit-segments.txt
24
      8 e6_records_in_prefix_naming_a_slot=0
     16 e6_records_in_prefix_naming_a_slot=2
e2_over_an_earlier_write      24 e2_over_an_earlier_write=0
e2_over_nonzero_base      24 e2_over_nonzero_base=0
e3_without_header      24 e3_without_header=0
e3_unaligned      24 e3_unaligned=0
e3_claimed_bytes_differ      24 e3_claimed_bytes_differ=0
e4_not_beyond_mount_root      24 e4_not_beyond_mount_root=0
```
（命令里的两份产物在 `research/prompts/per-write-judging-r2-sonnet-model/out/` 下；那一次命令末尾另算了 1 − 1379926285 / 1463812286，输出 0.057306528850926686，见第三节。）
4. 合起来：记录 R 点名单元 U 在 (盘 d, 槽 s)。今天的持久顺序是单元先、记录后，U 在 R 之前写过 [s·16384, s·16384 + 宽(U))。这一段的写若罩进这一截，就是盖在更早的写上，E2 必非 0。E2 全 0，所以 32768 罩中的只剩两种：(甲) U 是码 2 节点，罩中的是 [s+1 槽, s+2 槽)——U 之后那个从没写过的槽，这一段把新单元放在了那里，宽度误报；(乙) U 那一截在 R 之前根本没写过——记录先于它点名的单元，反例 B 的形状，真点名。数据单元与打包单元（32768）点名时罩中必盖旧写，被 E2 排掉。
5. 分开甲乙的读数：`PROBE_E6_HIT` 的 `named_class`、`gap_slots`、`overlaps_under_the_class_width` 与 `named_region_written_earlier_at`（P2）。甲：`named_class=2`、`gap_slots=1`、类宽不相交、点名那一截在记录之前写过；乙：类宽相交、点名那一截之前没写过。

旁证（量的，第二轮产物）：四条流里第一个单元段（段 8）都是 0，读数只有 0 与 2 两档；删除流每一步恰一条记录、两盘各一份（`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:270`：「assert_eq!(record_copies.len(), 2, "删除的每一步一条记录、两盘各一份");」），与「上一次发布的末条记录点名的最后一个节点 s，这一次发布的第一个单元落在 s+1」一致（推的：分配器连着往后放；本腿没逐行读分配器）。24 段的 `diag_records_of_same_identity` 全 0（量的）：前缀里没有一条记录带这一段单元自己的 (实例, 诞生代号)，与反例 B 臂上的 4 不同。它排不掉「更早一次发布的记录点名了这一段的单元」，所以只当旁证。

### 1.2 逐段判定（预测，依据是第二轮读数那一行；判定翻不翻看探针那一行）

| 流 | 段 | 写数 | 第二轮读数（量的） | 判定（推的） | 定案看 |
|---|---|---|---|---|---|
| unmodified | 12 | 28 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:85`：「PROBE_TIGHT_SEGMENT label=unmodified segment=12 writes=28 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | `PROBE_E6_SEGMENT label=unmodified segment=12` 的 `judged=` |
| unmodified | 16 | 30 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:89`：「PROBE_TIGHT_SEGMENT label=unmodified segment=16 writes=30 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | 同上，segment=16 |
| unmodified | 24 | 24 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:97`：「PROBE_TIGHT_SEGMENT label=unmodified segment=24 writes=24 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=24 |
| unmodified | 28 | 22 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:101`：「PROBE_TIGHT_SEGMENT label=unmodified segment=28 writes=22 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=28 |
| unmodified | 32 | 24 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:105`：「PROBE_TIGHT_SEGMENT label=unmodified segment=32 writes=24 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=32 |
| second-deletion…carry-the-mark | 12 | 30 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:95`：「segment=12 writes=30 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | 该流 segment=12 |
| second-deletion…carry-the-mark | 20 | 24 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:103`：「segment=20 writes=24 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=20 |
| second-deletion…carry-the-mark | 24 | 22 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:107`：「segment=24 writes=22 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=24 |
| three-unit…applies-first | 12 | 30 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:128`：「segment=12 writes=30 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=12 |
| three-unit…applies-first | 20 | 16 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:136`：「segment=20 writes=16 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=20 |
| three-unit…applies-first | 24 | 16 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:140`：「segment=24 writes=16 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=24 |
| three-unit…applies-first | 28 | 24 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:144`：「segment=28 writes=24 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=28 |
| deletion-through-the-last-batch | 12 | 30 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:165`：「PROBE_TIGHT_SEGMENT label=deletion-through-the-last-batch segment=12 writes=30 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=12 |
| deletion-through-the-last-batch | 20 | 24 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:173`：「PROBE_TIGHT_SEGMENT label=deletion-through-the-last-batch segment=20 writes=24 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=20 |
| deletion-through-the-last-batch | 24 | 22 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:177`：「PROBE_TIGHT_SEGMENT label=deletion-through-the-last-batch segment=24 writes=22 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=24 |
| deletion-through-the-last-batch | 28 | 24 | `research/prompts/per-write-judging-r2-sonnet-model/out/out-pairing-flows.txt:181`：「PROBE_TIGHT_SEGMENT label=deletion-through-the-last-batch segment=28 writes=24 … e6_records_in_prefix_naming_a_slot=2」 | 宽度误报 | segment=28 |

「点名的槽与点名它的记录」（记录写下标、记录序号、点名项的类、出生树、出生代号、盘与槽、与这一段哪个写相交、差几个槽）逐条由 `PROBE_E6_HIT` 打出，这一表跑出之前不填数。

## 二、收窄 E6 之后

### 2.1 收窄后的判据（新名 E7；只在我的探针上写过、被攻过零轮）

E7：这一段最低的写之前的每个 journal 记录写，用实现的读者（`crates/singlefs-core/src/journal.rs:847`：「pub fn parse(bytes: &[u8], expected_filesystem_identifier: u64) -> (record: Option<Self>)」）解出：
- 每个点名项的每条位置条目 (盘 d, 槽 s) 罩 [s·16384, s·16384 + 宽)，宽按点名项的类标签取读者真读的那一档：码 2 是 16384，码 1、码 3 与不认识的码 32768（同 1.1 第 2 条那三处）；
- 新根段四条指针（新树表、新映射根、新实例表、新分配记录树根）的每条位置条目罩 [s·16384, s·16384 + 16384)（槽 0 的空指针不算）；
- 读者拒的记录退回 E6 的罩法（手写字段表、每项 32768）；

这些区间都不与这一段任何写罩住的范围相交。探针里就是 `narrowed_passes` 的条件（`class_width_e6 == 0 && pointer_e6 == 0`，其余判据照第二轮分类器）。

E7 比 E6 松在哪、严在哪（写明收严、放宽各在哪，照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」那一节的三步）：
- 松：码 2 点名项的第二个 16 KiB 槽不再算。这一截没有任何读者经这条记录去读（1.1 第 2 条那三处都按类宽读），E6 拦它防不了什么；
- 严：多看新根段指针（2.3），E6 不看；
- 不变：数据单元、打包单元与不认识的码照旧按 32768。
按那一节的字面，E7 不是「只许收严」：它在码 2 那一截上放宽了。所以 E7 是判据的新形态，不是 E6 的澄清；这是第三轮里冒出来的零轮形态，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二段，不为它再开一轮，进交用户表标「零轮」。

### 2.2 E7 放不放过反例 B（记录先写、点名的单元后写）

推的：反例 B 里记录 R 在前缀、它点名的单元 U 在这一段，R 点名的槽就是 U 写的起点槽，[s·16384, s·16384 + 宽) 含 U 的第一个字节，不管宽取 16384 还是 32768 都相交。R 与 U 同段（中间没有屏障）的被 E1 拦（段里不全是单元写）。R 点名的类标签与 U 实际的长度不一样（坏记录）时，罩的起点仍是 s，照样相交。E7 不再拦的只有「码 2 点名、这一段的写从 s+1 起」，而那一截 R 的读者不读。
量的待主 agent 跑：P4（record_before_units 段 12 `class_width_e6=4`）、P5（late_unit 段 16 `class_width_e6=2`）。P4、P5 任一为 0，2.2 撤回。

### 2.3 E6、E7 之外还有一条路：记录的新根段指针（打中判据字面，推的）

施加一次发布之后，新根取那次发布末条记录带的新根段：`crates/singlefs-core/src/recovery.rs:5303`：「/// [`replay_journal`] 交给取前缀核的「新根从哪来」：核不看它，施加了发布就交回最后施加的那次发布末条带的那一项，」。新根段里四条是带位置条目的节点指针（`crates/singlefs-core/src/journal.rs` 的 `JournalRecord` 字段 `new_tree_table`、`new_mapping_root`、`new_instance_table`、`new_allocation_record_tree_root`）。施加前验的是点名项（`every_named_unit_of_the_record_verifies`），我没找到验这四条指针指的单元的那一步（推的：只读了这一个验证函数，没逐行读 `replay_journal` 的其余部分）。所以「前缀里一条记录的新根指针指着这一段要写的槽、而那个槽不在它的点名项里」时，施加之后的根直接读这一段在飞的写，E6 的字面（只看点名项）看不见。
四句：分不分辨臂——不分辨甲乙，是两者共用的门槛缺一个分句；系统看不看得到——看得到，记录字节里就有；满足的分句——E6 字面「每条 `JournalRecord` 写点名的（盘身份，槽号）」只说点名项；改法——E7 补了指针分句。今天的写者是不是恒把新根指的节点也写进点名项，本腿没读写者代码（推的）。P6 预测真实流上读数 0（不改态数），record_before_units 段 12 至少 2（那几条记录挪到单元之前，新树表指针指着段 12 里的节点）。

### 2.4 收下的段上逐态全量核对：命令给了，但在今天的实现上它判别不了乙（打中方法层）

命令（不取样，全量展开；删除流 7 个单元段分在五个按功能切的崩溃点重放项里，段号照 `.claude/gate.d/crash-point-replay-items.tsv` 第 24–28 行的说明）在「要主 agent 跑的命令」一节第 2 条。它判的是全组合每个状态，入口自己断言整条流干净（例：`crates/singlefs-verification/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:937`：「fn full_enumeration_of_the_unlink_batches_after_the_multi_record_publish_is_exhaustive_and_clean() {」）。

为什么核过也不证乙（推的，演绎）：乙（不论增减还是并）、甲、以及「只判 S=∅、别的状态一律照抄它」的退化候选，用到的每个代表状态都在这一段的全量枚举域里。全组合判下这一段每个状态都绿、红项都空时，任何一种合成交出的都是空集，与全组合逐态相同。所以「逐态全量核乙与全组合等价」在干净的实现上恒成立，连退化候选也过；它分不出乙对不对。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」，这是「打中不分辨臂」：病根在共用前提（被测实现今天干净），不能拿它判乙。
量的旁证：第二轮在段 12、16、20 上 9 个宇宙（9216 态）三种比较全是 0、空集态全绿（`research/prompts/per-write-judging-r2-sonnet-model/out/out-universes.txt:25`：「PROBE_SYNTH label=unmodified-trial0 segment=20 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0」，段 12、16 那 6 行同形）。那几个计数器打不出「实际红了几态」，所以本腿的探针加了 `actual_red_states` 与退化候选两列（P7 预测都为 0）。
要能判别，得让被测实现有 bug：拿崩溃点重放的 crates 变异，把「全组合判出红的变异」与「乙的代表状态也判出红、且乙·增减逐态合成出同样红项的变异」两张名单比。这个装置今天没有（推的，被攻过零轮；本腿没写，不在这一轮的射程里，交主 agent 定要不要登记实验）。

### 2.5 E7 下的态数（算的，条件写在前面）

条件：E7 收下全部 24 段（P3），且 E5 照第二轮辩方当成满足。辩方自己写了 E5 没实现前不能据此降态：`research/prompts/per-write-judging-r2-sonnet-output.md:96`：「当前没有机器检查它的实现（第一节第 6 行）」，同一句接着说在有之前这一条算未满足、不能据此降态。

| 流 | 全量（量的，第二轮 `engine_total`） | 第二轮判据下（量的） | E7 下（算的） |
|---|---|---|---|
| unmodified | 1463812286 | 1379926285 | 471 |
| second-deletion…carry-the-mark | 1178599580 | 1094713579 | 355 |
| three-unit…applies-first | 1107689646 | 1090650353 | 378 |
| deletion-through-the-last-batch | 1195376807 | 1111490806 | 404 |

算法：全量减去各单元段的 2^写数 − 1（单元段的全量态数，第二轮 `engine_states` 与此逐段相等，量的），再加各段 `states_group_form`（量的）。unmodified：单元段全量合计 1463812089，非单元段 197，加组合态 274 得 471；另三条非单元段 161、180、173，得 355、378、404。与第二轮辩方第五节算的四个数相同（两边共用同一张段表，不是独立校验）。探针跑出的 `reduced_total_narrowed` 是这一表的量的版本（P3）。

E5 那一条（推的，本腿没核完）：池级 checker 逐槽扫单元区（`crates/singlefs-checker/src/walk.rs:5537`：「fn scanned_tree_identifiers(」）。读的槽来自镜像里写过的候选槽时，S=∅ 那一遍不读在飞的写，E5 的字面恒过；可落下的写在别的状态里照样被扫到，所以 E5 按 S=∅ 的读迹判是判别子观测不到的那一类。它不归本轮攻击面，只记在限度里。

## 三、第二轮报出的数能不能从产物核出来

都只读 `research/prompts/per-write-judging-r2-sonnet-model/out/` 下的产物、用命令数（量的）；算术是算的。

| 第二轮的数 | 出处 | 核法与原样输出 | 结论 |
|---|---|---|---|
| 约 6%（整条删除流 1,463,812,286 → 1,379,926,285） | `research/prompts/per-write-judging-r2-main-verification.md:17`：「整条流从 1,463,812,286 只降到 1,379,926,285」 | 产物 `research/prompts/per-write-judging-r2-sonnet-model/out/out-classifier-and-arms.txt:109`：「PROBE_TIGHT_FLOW label=unmodified segments=36 unit_segments=7 passing_unit_segments=2 engine_total=1463812286 reduced_total_group_form=1379926285」；`python3 -c "print(1-1379926285/1463812286)"` 输出 `0.057306528850926686` | 核得出，5.73%。差 83886001 恰是段 8、20 两段（2^24 − 1 − 37）+（2^26 − 1 − 40），算的 |
| 24 段里 E2、E3、E4 全 0、E6 16 段为 2 | 同上那一行判决 | 1.1 第 3 条那段命令与输出 | 核得出 |
| 91 对里 78 对有差（243/1456 个状态） | `research/prompts/per-write-judging-r2-main-verification.md:23`：「91 对里 78 对有差（243/1456 个状态）」 | 汇总行 `research/prompts/per-write-judging-r2-sonnet-model/out/out-two-groups.txt:325`：「PROBE_TWO_GROUPS_SUMMARY pairs=91 skipped=0 pairs_where_yi_items_differ=78 pairs_where_yi_red_differs=0 pairs_where_jia_items_differ=91」。逐行重数 91 行 `PROBE_SYNTH label=two-groups…`（awk 取 `yi_items_differ`）输出 `pairs 91 pairs_yi_items_differ 78 states_yi_items_differ 243 states 1456` | 核得出。两个限定见下 |
| 判据之内只在段 20 量过 3 次共 3072 态 | `research/prompts/per-write-judging-r2-main-verification.md:19`：「判据之内有量的只有段 20 的 3 次、共 3072 个状态」 | 段 20 三行 `research/prompts/per-write-judging-r2-sonnet-model/out/out-universes.txt:25-27`，每行 `states=1024`；段 12、16 的 6 行在第二轮判据外（那两段 E6 为 2） | 核得出，3 × 1024 = 3072 |
| 复跑汇总漏行 | 第二轮判决「核查怎么做的」 | 四份分产物含 `PROBE_` 的行 190、107、15、310（合 622），行首是 `PROBE_` 的 189、106、14、309；`out.txt` 618 行 | 核得出，漏 4 行，每份第一行（接在 libtest 的「test … ... 」后面） |

78/91 的两个限定（量的 + 推的）：
1. 量它的代码合成的是并：`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_dynamic_unlink.rs.in:116`：「yi_items.extend(outcomes[usize::try_from(mask & group_mask).expect("掩码")].1.iter().cloned());」，起点是空集态的红项。第二轮判决的交用户表把它记在「候选乙（按单元副本组判、增减合成）」名下。91 对的空集态全绿（`grep -o 'baseline_red=[a-z]*' out-two-groups.txt | sort | uniq -c` 输出 `91 baseline_red=false`），空集态没有红项可减，增减与并逐态同值（推的，演绎），所以数照样成立；换到空集态红的段上，这个数不能直接转给增减合成。
2. 差全在红项层：`pairs_where_yi_red_differs=0`，没有一态红绿判反。这 91 对是把一组单元挪到上一版单元上（盖旧写），E2 拦，在 E6、E7 收下的范围之外。

3072 的限定：三个计数器全 0 只说明合成与真判一致，产物没打这 3072 态里实际红了几态；照 2.4，实际全绿时这项比较分不出乙与退化候选。E7 收下段 12、16 之后（P3），第二轮那 6 个宇宙也进了判据之内，判据之内取样变成 9 个宇宙 9216 态，但同样全是空集态绿的那一类。

## 要主 agent 跑的命令

1. 探针整份复跑（判决要引的 P1–P8 读数都在这一条）。在哪起：任意目录（路径都是绝对的，都在共享根下）。
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/per-write-judging-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/per-write-judging-r3-attack/frozen --work /home/fy5090/.local/share/singlefs/work/per-write-judging-r3-attack/rerun-work`
   看什么：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里两行 `name=rerun_extract part=unlink …`、`part=pairing …`（`lines_with_probe` 等于 `extracted`、都大于 0），一行 `name=rerun_output file=… lines=<N>`，一行 `name=rerun_compare same=not_compared`（模型目录里还没有 out/）。产物在 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r3-attack/rerun-work/` 下：`out.txt`（两份探针行合起来）、`out-unlink.txt`、`out-pairing.txt`（cargo 原样输出）。撞上限重排时照原样再起同一条，做完的步跳过。编不过（探针没编过，只过了 rustfmt 的语法检查）就新派攻方腿承接修，`out-unlink.txt` 或 `out-unlink.partial` 里有编译错误。
   跑完请把 `rerun-work/out.txt` 拷进 `research/prompts/per-write-judging-r3-opus-model/out/out.txt`，新派同立场的腿（「承接：research/prompts/per-write-judging-r3-opus-output.md」）对着第「跑前写死的预测」一节逐条判。状态目录 `rerun-work-state/tree` 是仓副本，用完删；编译目录归服务（`task-server/task clean <号>`）。
2. 删除流 7 个单元段逐态全量判（第二节 2.4 的「不取样」核对；它在今天的实现上分不出乙，见 2.4）。在哪起：仓根。
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=崩溃点重放 删除流的新建文件、两次顺序写、删除意图、两批、最后一批五项; 范围=5 项; 为什么=E7 收下的删除流七个单元段逐态全量判，核全组合下每个状态都干净' task-server/task submit -- bash scripts/crash/crash-point-replay.sh --item unlink-after-the-multi-record-publish-first-file-creation --item unlink-after-the-multi-record-publish-sequential-writes --item unlink-after-the-multi-record-publish-unlink-intent --item unlink-after-the-multi-record-publish-unlink-batches --item unlink-after-the-multi-record-publish-unlink-final-batch`
   看什么：`TASK_SERVER_RESULT` 的 `exit=0`，log 里五项各自的通过行、没有 `✗` 行。整条流全量 1463812286 态（量的，第二轮 `engine_total`），判定库里有的块复用。对拍三条流没有崩溃点重放项（`.claude/gate.d/crash-point-replay-items.tsv` 里没有对拍测试文件的项，`grep` 核过）；它们的逐态全量判是对拍测试文件里各自那条 GPU 与 CPU 对拍用例，那几条比的是两种判器逐态相同，不断言每态都绿，本腿没核它们的断言（推的），所以对拍三条流的「不取样」核对本腿给不出合用的命令。

## 没打中的形状

攻方要打的是「16 段是误报」那一边：想找今天的流上一条前缀记录真点名这一段在飞的写。试过的形状（都是推的，取样范围是四条真实流 24 段的全部前缀记录，不取样）：
- 释放、隔离的单元的槽被这一段复用：复用就是盖旧写，24 段 E2 全 0（量的），排掉。
- 记录点名 32768 的数据单元或打包单元：点名那一截在记录之前写满过，罩中必盖旧写，排掉（1.1 第 4 条）。
- 两份副本在两块盘上：位置条目带盘身份，探针按盘比，不跨盘。
- 点名项是密文、解出来是乱码：恢复与池级 checker 都直接解点名项的明文字段（1.1 第 2 条那几处），探针另用实现的读者解、与手写字段表逐条比（P1 的 `parse_mismatches`）。
- 更晚的发布的记录：不在前缀里，这一段的崩溃状态里后面的段一个都不落。
- 新根段指针：找到一条 E6 字面看不见的路（2.3），今天的流上预测读数 0。
- 反向：第二轮报 0 的 8 段会不会漏了真点名：E7 只在码 2 那一截比 E6 松，那 8 段 E6 为 0，E7 也必为 0（推的），另由 P3 量。

## 这条腿自己的限度

- 一条命令都没跑：第一节、第二节的判定都是推的或待主 agent 跑的预测；探针没编过，只用 `rustfmt --check` 看过两份拼起来的追加文本都能解析（只有排版差异、没有 `error`）。
- 探针追加在验证包两个已有测试目标里（只在副本里），与第二轮辩方探针（任务 8700）同一种做法；它不调枚举函数、流不是拷来的，但「测试目标不放进验证包」那一条怎么读，交主 agent 判。
- E1–E4 与组数照第二轮辩方分类器原样判（单元头偏移是它另写的一份）；那份有错，E7 下的计数跟着错。
- 小宇宙真判用 `judge_crash_image`，版本表只有 A、B、C 三版带文件的；段 12、16 与两条臂上恢复落在 A 或 B，够用。真点名段若在段 20 之后（P2 翻面时才跑），恢复落到没有文件的那几版，这个判器会多报 oracle 红（推的），只影响那一步的红项，不影响三种合成之间的比。
- E5 没核（2.5 末段只是记一笔）；E7 下的态数以 E5 当成满足为条件。
- 背景材料的附录与小节清单抄的是 `.claude/kb/experiments/06-加密算法选型.md` 与 E1–E4 四份实验页（`research/prompts/_per-write-judging-r3-background.md` 第 29 行起的清单），与本轮判据 E1–E6 只是同号；本腿一处没引附录，主 agent 写材料的那一步可能把正文里的判据号当成了实验号。
- 草稿目录里留的：`frozen/`（仓副本，50M，`.deliverables` 写成「交接」，主 agent 跑第 1 条要用）；`fmtcheck/`、`real-unit-segments.txt`、`minimal-copy.log` 是普通文件。没建编译目录。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳。
- 副本上的数不算入库装置上的数；本腿没量出任何数。
- 没往统一任务服务交任务；判决要引的数没复跑（命令在「要主 agent 跑的命令」一节）。
- 2.4 提的「拿 crates 变异判别乙」的装置没写，被攻过零轮。
