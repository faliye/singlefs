# 逐写独立判 第二轮 辩方腿（承接）报告：读探针产物

批次：records/2026-11-05-删轻型并入重型.md#并包。立场：辩方。承接：`research/prompts/per-write-judging-r2-sonnet-output.md`（前一条辩方腿，探针没编没跑）。这一份读主 agent 跑出的任务 8700（`type=succeeded exit=0`，第 4 次尝试，主 agent 给的结局）的产物，只读、不起任何命令。

标注：「量」= 产物文件里的行（引整行、带文件名与行号，行号是 `grep -nH` 现取的）；「算」= 我对这些行用 `grep`、`awk` 与 `tools/tabulate.py` 数出来的；「读」= 我读探针源码或前一份报告；「推」= 产物里没有。产物目录（只读）：`/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-defense-rerun/`；引整行时写 `out-<名>.txt:行号:`，指这个目录下的文件。

## 读了什么

| 文件 | 行数（量，`wc -l`） | sha256（量） |
|---|---|---|
| `out-classifier-and-arms.txt`（删除流的静态判据，含四条造坏的臂） | 263 | `a728cf2752854b65e88df31873d19bb1d2970710b9a0d2f0e4145ac28d1c409e` |
| `out-pairing-flows.txt`（对拍三条流的静态判据） | 189 | `66425907a448a7bb00159e9ee6154fd4c806770e8118f9b74987cb6a351577f5` |
| `out-universes.txt`（小宇宙全量对照，含阳性对照） | 34 | `a9b98a64bfb5b686824e6cc58643abef0f5ea9f1ad7dc8cbe82ead4a6c56de86` |
| `out-two-groups.txt`（两组同被盖，91 对） | 329 | `1f3524cdcb1c1aacd99cc7bde934dda5142259661ed14af55ad89451d9ff8364` |
| `out.txt`（汇总，618 行 `PROBE_`） | 618 | `80888f5a95e8b4454f58cae36e93583c478aa0452e5913ca60e47afe8e22eae5` |
| `log8700.txt`（`task-server/task log 8700 --all` 存的，在我的草稿目录） | 153 | 未取 |
| `tools/tabulate.py`（我写的，只读 `out-two-groups.txt`，按类标签分组、拆红项差，在草稿目录） | 29 | `e28164fda96c287e162bb702058de35a71a25b5b7ee7dc4c8ad214b8cdbe7209` |
| `tabulate-out.txt`（`tabulate.py` 这一次的原样输出，8 行，在我的草稿目录） | 8 | `55e40a3783123cbd7b725f69139fb32390b5dd0a549824fe86cdb1cd15de1eb6` |

`log8700.txt` 没取 sha256：它是服务日志的拷贝，下文不引它的行。

探针与上一份报告模型目录一致（量）：任务 8700 的树里两份测试文件末尾追加的内容，逐字节等于 `research/prompts/per-write-judging-r2-sonnet-model/probe/` 里三份 `.rs.in` 的拼接（`sha256sum` 对两对，各自相同）。

一处产物形状要先说：`out.txt` 是 `grep '^PROBE_'` 汇的，每份 `out-*.txt` 里紧跟在 `test … ... ` 之后的第一条 `PROBE_` 行行首不是 `PROBE_`，被汇总漏掉，共 4 行（量：各文件 `grep -c 'PROBE_'` 比 `grep -c '^PROBE_'` 各多 1，`out-classifier-and-arms.txt` 189 对 190、`out-pairing-flows.txt` 106 对 107、`out-universes.txt` 14 对 15、`out-two-groups.txt` 309 对 310；四个行首数相加 618 等于 `out.txt` 行数）。被漏的是 `out-classifier-and-arms.txt:70` 的盘 0 单元区行、`out-pairing-flows.txt:79` 的第一条流态数行、`out-universes.txt:16` 的段 12 组数行、`out-two-groups.txt:16` 的第一对。下文引整行都从 `out-*.txt` 取，不从 `out.txt` 取。

## 总述（每一条都在后面有表与整行）

1. **判据收严后的 Q1 与 Q6 的核心读数没中**：删除流 7 个单元段里只有 2 个 `passes=true`，对拍三条流各 2 个；整条流的态数是 1379926285、1094713579、1090650353、1111490806，不是预测的 471、355、378、404（Q1、Q6 的「态数」半句不中，「引擎总数」半句中）。唯一让段不过的是 E6：24 个真实单元段里 `e6_records_in_prefix_naming_a_slot` 非 0 的有 16 个，每个都恰好是 2；E2、E3、E4 的各列在这 24 行里全是 0（算）。
2. **E6 的基线不是 0，所以臂上的 `passes=false` 不分辨臂**：反例 A 动的段 16、反例 B 动的段 12、写长截短动的段 12，在未改动的流里本来就是 `passes=false`。分辨得出臂的是各自那一列：A 看 `e4_not_beyond_mount_root`（0 变 2），写长看 `e3_claimed_bytes_differ`（0 变 1），B 看 E6 的增量（2 变 6）与诊断列 `diag_records_of_same_identity`（0 变 4，这一列不在 `passes` 里）。五个 `PROBE_TIGHT_FLOW` 行的 `passing_unit_segments` 全是 2，不分辨臂。
3. **乙的组内合成：红不红一层在三个格里 13744 个状态上与逐态真判全部相同；红项一层在没盖更早写的小宇宙与阳性对照里相同（12288 个状态），在「两组同被盖」91 对里 78 对不同（243 个状态，共 1456 个）**，Q9 的预测（0 对）不中。两个方向都有差：乙多报 I-3.10、I-3.15、I-1.13，也有 3 个状态少报 I-3.1、I-3.11（算，`tabulate-out.txt`）。这 91 对按构造挪到更早的写上，E2 该拒；但产物里没有对这些挪过的写跑分类器的行（读）。
4. **前提**：E3 的对齐、头可解、声明等于写长三半句，E4 的整数比较，E6 的点名项字节解析（B 臂能认出 4 条记录），「每组两份」（`group_sizes={2: N}` 51 行全是）这一趟量到了；E3「落在单元区」只量到写集分开、没有区边界；E5 仍是推（没有实现）；E6「正常流上记录只点名更早发布的单元」被这一趟的读数顶回去了（见第三节）。
5. 没有一行判决要我撤销或维持；这一份只交读数与判定。

## 一、前一份报告第六节的预测 Q1 到 Q9，逐条判

预测原文在 `research/prompts/per-write-judging-r2-sonnet-output.md` 第 206 到 214 行（预测那一格整格抄）。判：中 / 不中 / 产物里没有这一格。引的产物整行放在本节表后的「整行」小节，行号指各 `out-*.txt` 自己的行号。

| 号 | 预测（前一份报告里那一格，整格） | 判 | 对应的产物行 | 说明 |
|---|---|---|---|---|
| Q1 | 「`label=unmodified`：7 个单元段 `passes=true`，`e2_*`、`e3_*`、`e4_not_beyond_mount_root`、`e6_records_in_prefix_naming_a_slot` 全 0，`group_sizes={2: N}`（段 12、16、20 的 N 是 14、15、13），`engine_total=1463812286`，`reduced_total_group_form=471`」（`research/prompts/per-write-judging-r2-sonnet-output.md:206`） | **不中**（分项见右） | `out-classifier-and-arms.txt:81`、`:85`、`:89`、`:93`、`:97`、`:101`、`:105`、`:109` | 中：`e2_*`、`e3_*`、`e4_not_beyond_mount_root` 在 7 个单元段上全 0；`group_sizes` 段 12、16、20 是 `{2: 14}`、`{2: 15}`、`{2: 13}`；`engine_total=1463812286`。不中：7 个单元段只有段 8、20 是 `passes=true`；`e6_records_in_prefix_naming_a_slot` 在段 12、16、24、28、32 是 2（不是 0）；`reduced_total_group_form=1379926285`（不是 471）。 |
| Q2 | 「`label=late_unit`：段 16 `e4_not_beyond_mount_root=2`、`diag_birth_differs_from_publish=2`、`passes=false`」（`research/prompts/per-write-judging-r2-sonnet-output.md:207`） | **中** | `out-classifier-and-arms.txt:127` | 三个读数都对。但段 16 在未改动的流里也是 `passes=false`（`:89`），所以 `passes=false` 不是这一臂独有；分辨它的是 `e4_not_beyond_mount_root` 从 0 变 2。 |
| Q3 | 「`label=record_before_units`：段 12 `e6_records_in_prefix_naming_a_slot` ≥ 1，`passes=false`，`e4_not_beyond_mount_root=0`」（`research/prompts/per-write-judging-r2-sonnet-output.md:208`） | **中**（字面） | `out-classifier-and-arms.txt:161`（对比 `:85`） | 字面三项都对：`e6=6`、`passes=false`、`e4=0`。但未改动的段 12 已经是 `e6=2`、`passes=false`（`:85`），所以字面成立不构成 B 被拒的证据；分辨 B 的是 E6 从 2 变 6，与诊断列 `diag_records_of_same_identity` 从 0 变 4。 |
| Q4 | 「`label=short_write`：段 12 `e3_claimed_bytes_differ=1`，`passes=false`」（`research/prompts/per-write-judging-r2-sonnet-output.md:209`） | **中**（字面） | `out-classifier-and-arms.txt:198`（对比 `:85`） | `e3_claimed_bytes_differ=1`，`passes=false`。基线同理：段 12 本来就 `passes=false`；分辨的是 E3 那一列从 0 变 1。 |
| Q5 | 「`label=mixed_segment`：段 12 的行是 `e1_only_unit_writes=false`」（`research/prompts/per-write-judging-r2-sonnet-output.md:210`） | **中** | `out-classifier-and-arms.txt:235` | `kinds={"SystemConfigurationSlot", "UnitWrite"}`、`e1_only_unit_writes=false`；该行没有 `e2_` 字段（推翻条件「出现 `e2_over_an_earlier_write` 行」没发生）。这一格的 `passes` 不是比较的东西：混合段只判 E1。 |
| Q6 | 「对拍三条流：`PROBE_TIGHT_ENGINE states=` 依次 1178599580、1107689646、1195376807；`reduced_total_group_form` 依次 355、378、404」（`research/prompts/per-write-judging-r2-sonnet-output.md:211`） | **前半中、后半不中** | `out-pairing-flows.txt:79`、`:111`、`:112`、`:148`、`:149`、`:185` | 引擎总数三个都对。`reduced_total_group_form` 是 1094713579、1090650353、1111490806，不是 355、378、404（每条流 2 个单元段过）。 |
| Q7 | 「`unmodified-trial*`（段 12、16、20 各 3 次）：`baseline_red=false`，`yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0`」（`research/prompts/per-write-judging-r2-sonnet-output.md:212`） | **中** | `out-universes.txt:17`、`:18`、`:19`、`:21`、`:22`、`:23`、`:25`、`:26`、`:27` | 9 次取样（段 12、16、20 各 3 次）每次 5 组、10 个写、1024 个状态，四个差都是 0。段 12、16 在 E6 下不过（上面），所以对 E 判据放行的段只有段 20 的 3 次。 |
| Q8 | 「`planted-trial*`（段 16 挪过去一组）：`yi_red_differs=0 yi_items_differ=0`，`jia_items_differ` 约 256（两份都落的状态数：2^(10−2)；至少 1）」（`research/prompts/per-write-judging-r2-sonnet-output.md:213`） | **中** | `out-universes.txt:28`、`:29`、`:30` | 3 次，每次 `yi_red_differs=0 yi_items_differ=0`，`jia_items_differ=256`（恰 256）、`jia_red_differs=0`。 |
| Q9 | 「`PROBE_TWO_GROUPS_SUMMARY`：`pairs_where_yi_items_differ=0`，`pairs_where_jia_items_differ` ≥ 1」（`research/prompts/per-write-judging-r2-sonnet-output.md:214`） | **不中**（`yi` 半句）；`jia` 半句中 | `out-two-groups.txt:325` | `pairs=91 skipped=0 pairs_where_yi_items_differ=78 pairs_where_yi_red_differs=0 pairs_where_jia_items_differ=91`。预测说 `yi` 非零就是祖先后代遮红项、乙的组间独立被推翻：红项层被推翻，红不红层 91 对全对。 |

**Q1 没中的原因只有 E6 一列（算）**：24 个真实单元段（删除流 7、对拍三条流 5+6+6）里，`e2_over_an_earlier_write`、`e2_over_nonzero_base`、`e3_without_header`、`e3_unaligned`、`e3_claimed_bytes_differ`、`e4_not_beyond_mount_root`、`diag_records_of_same_identity` 七列全是 0；`e6_records_in_prefix_naming_a_slot` 非 0 的有 16 行，每行恰好 2，`passes=true` 8 行、`passes=false` 16 行。这 24 行存在我的草稿目录的 `real-unit-lines.txt`，数法是 `grep -o "<列>=[0-9]*"` 加 `awk` 求和。

**471、355、378、404 这四个数本身是对的（算）**：把每条流 `states_group_form` 在单元段上的和加上非单元段的引擎态数再加 1：删除流 1+196+274=471，第二条 1+160+194=355，第三条 1+179+198=378，第四条 1+172+231=404。所以前一份报告的组数闭式没算错；错的是「这些段都过判据」这个前提，被 E6 挡掉的段按今天的态数留着。

**Q3 与 Q4 的字面成立是个陷阱**：在单元段 12 上，造坏的臂和未改动的流同样 `passes=false`。要判「判据把这一臂拒了」，必须看那一臂自己对应的一列（A 看 E4，写长看 E3，B 看 E6 的增量），不能看 `passes`。

### 整行（`out-classifier-and-arms.txt`，删除流；格式 `文件:行号:原行`）

```
out-classifier-and-arms.txt:81:PROBE_TIGHT_SEGMENT label=unmodified segment=8 writes=24 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=0 diag_records_of_same_identity=0 group_sizes={2: 12} passes=true states_group_form=37 engine_states=Some(16777215)
out-classifier-and-arms.txt:85:PROBE_TIGHT_SEGMENT label=unmodified segment=12 writes=28 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=2 diag_records_of_same_identity=0 group_sizes={2: 14} passes=false states_group_form=43 engine_states=Some(268435455)
out-classifier-and-arms.txt:89:PROBE_TIGHT_SEGMENT label=unmodified segment=16 writes=30 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=2 diag_records_of_same_identity=0 group_sizes={2: 15} passes=false states_group_form=46 engine_states=Some(1073741823)
out-classifier-and-arms.txt:93:PROBE_TIGHT_SEGMENT label=unmodified segment=20 writes=26 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=0 diag_records_of_same_identity=0 group_sizes={2: 13} passes=true states_group_form=40 engine_states=Some(67108863)
out-classifier-and-arms.txt:97:PROBE_TIGHT_SEGMENT label=unmodified segment=24 writes=24 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=2 diag_records_of_same_identity=0 group_sizes={2: 12} passes=false states_group_form=37 engine_states=Some(16777215)
out-classifier-and-arms.txt:101:PROBE_TIGHT_SEGMENT label=unmodified segment=28 writes=22 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=2 diag_records_of_same_identity=0 group_sizes={2: 11} passes=false states_group_form=34 engine_states=Some(4194303)
out-classifier-and-arms.txt:105:PROBE_TIGHT_SEGMENT label=unmodified segment=32 writes=24 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=2 diag_records_of_same_identity=0 group_sizes={2: 12} passes=false states_group_form=37 engine_states=Some(16777215)
out-classifier-and-arms.txt:109:PROBE_TIGHT_FLOW label=unmodified segments=36 unit_segments=7 passing_unit_segments=2 engine_total=1463812286 reduced_total_group_form=1379926285
out-classifier-and-arms.txt:127:PROBE_TIGHT_SEGMENT label=late_unit segment=16 writes=32 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=2 diag_birth_differs_from_publish=2 e6_records_in_prefix_naming_a_slot=4 diag_records_of_same_identity=4 group_sizes={2: 16} passes=false states_group_form=49 engine_states=None
out-classifier-and-arms.txt:147:PROBE_TIGHT_FLOW label=late_unit segments=36 unit_segments=7 passing_unit_segments=2 engine_total=1 reduced_total_group_form=1
out-classifier-and-arms.txt:161:PROBE_TIGHT_SEGMENT label=record_before_units segment=12 writes=28 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=0 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=6 diag_records_of_same_identity=4 group_sizes={2: 14} passes=false states_group_form=43 engine_states=None
out-classifier-and-arms.txt:184:PROBE_TIGHT_FLOW label=record_before_units segments=36 unit_segments=7 passing_unit_segments=2 engine_total=1 reduced_total_group_form=1
out-classifier-and-arms.txt:198:PROBE_TIGHT_SEGMENT label=short_write segment=12 writes=28 kinds={"UnitWrite"} e2_over_an_earlier_write=0 e2_over_nonzero_base=0 e3_without_header=0 e3_unaligned=0 e3_claimed_bytes_differ=1 e4_not_beyond_mount_root=0 diag_birth_differs_from_publish=0 e6_records_in_prefix_naming_a_slot=2 diag_records_of_same_identity=0 group_sizes={2: 14} passes=false states_group_form=43 engine_states=None
out-classifier-and-arms.txt:222:PROBE_TIGHT_FLOW label=short_write segments=36 unit_segments=7 passing_unit_segments=2 engine_total=1 reduced_total_group_form=1
out-classifier-and-arms.txt:235:PROBE_TIGHT_SEGMENT label=mixed_segment segment=12 writes=30 kinds={"SystemConfigurationSlot", "UnitWrite"} e1_only_unit_writes=false engine_states=None
out-classifier-and-arms.txt:259:PROBE_TIGHT_FLOW label=mixed_segment segments=36 unit_segments=6 passing_unit_segments=2 engine_total=1 reduced_total_group_form=1
```

### 整行（`out-pairing-flows.txt`，对拍三条流）

```
out-pairing-flows.txt:79:test probe_tightened_criterion_over_the_three_billion_state_pairing_flows ... PROBE_TIGHT_ENGINE flow=second-deletion-batch-quarantined-data-unit-records-carry-the-mark states=1178599580
out-pairing-flows.txt:111:PROBE_TIGHT_FLOW label=second-deletion-batch-quarantined-data-unit-records-carry-the-mark segments=28 unit_segments=5 passing_unit_segments=2 engine_total=1178599580 reduced_total_group_form=1094713579
out-pairing-flows.txt:112:PROBE_TIGHT_ENGINE flow=three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first states=1107689646
out-pairing-flows.txt:148:PROBE_TIGHT_FLOW label=three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first segments=32 unit_segments=6 passing_unit_segments=2 engine_total=1107689646 reduced_total_group_form=1090650353
out-pairing-flows.txt:149:PROBE_TIGHT_ENGINE flow=deletion-through-the-last-batch states=1195376807
out-pairing-flows.txt:185:PROBE_TIGHT_FLOW label=deletion-through-the-last-batch segments=32 unit_segments=6 passing_unit_segments=2 engine_total=1195376807 reduced_total_group_form=1111490806
```

### 整行（`out-universes.txt`，小宇宙与阳性对照）

```
out-universes.txt:16:test per_write_judging_tightened_dynamic::probe_group_synthesis_against_the_judged_states_of_small_universes ... PROBE_UNIVERSE_GROUPS label=unmodified segment=12 groups=14
out-universes.txt:17:PROBE_SYNTH label=unmodified-trial0 segment=12 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:18:PROBE_SYNTH label=unmodified-trial1 segment=12 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:19:PROBE_SYNTH label=unmodified-trial2 segment=12 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:20:PROBE_UNIVERSE_GROUPS label=unmodified segment=16 groups=15
out-universes.txt:21:PROBE_SYNTH label=unmodified-trial0 segment=16 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:22:PROBE_SYNTH label=unmodified-trial1 segment=16 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:23:PROBE_SYNTH label=unmodified-trial2 segment=16 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:24:PROBE_UNIVERSE_GROUPS label=unmodified segment=20 groups=13
out-universes.txt:25:PROBE_SYNTH label=unmodified-trial0 segment=20 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:26:PROBE_SYNTH label=unmodified-trial1 segment=20 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:27:PROBE_SYNTH label=unmodified-trial2 segment=20 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=0
out-universes.txt:28:PROBE_SYNTH label=planted-trial0 segment=16 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=256
out-universes.txt:29:PROBE_SYNTH label=planted-trial1 segment=16 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=256
out-universes.txt:30:PROBE_SYNTH label=planted-trial2 segment=16 groups=5 writes=10 states=1024 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=256
```

### 整行（`out-two-groups.txt`，91 对的汇总与第一对）

```
out-two-groups.txt:16:test per_write_judging_tightened_dynamic::probe_two_planted_groups_over_every_pair_of_units_of_the_earlier_publish ... PROBE_SYNTH label=two-groups-0-1-classes-1-1 segment=16 groups=2 writes=4 states=16 baseline_red=false yi_red_differs=0 yi_items_differ=0 jia_red_differs=0 jia_items_differ=7
out-two-groups.txt:325:PROBE_TWO_GROUPS_SUMMARY pairs=91 skipped=0 pairs_where_yi_items_differ=78 pairs_where_yi_red_differs=0 pairs_where_jia_items_differ=91
```

### 整行（补：混合臂并进的第 11 段、单元区行，下文第二、三节引）

```
out-classifier-and-arms.txt:70:test per_write_judging_tightened_dynamic::probe_tightened_criterion_over_the_unlink_stream_and_two_misordered_arms ... PROBE_TIGHT_AREA label=unmodified device=0 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16826368)
out-classifier-and-arms.txt:71:PROBE_TIGHT_AREA label=unmodified device=1 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16826368)
out-classifier-and-arms.txt:160:PROBE_TIGHT_SEGMENT label=record_before_units segment=11 writes=6 kinds={"JournalRecord", "SystemConfigurationSlot"} e1_only_unit_writes=false engine_states=None
out-pairing-flows.txt:80:PROBE_TIGHT_AREA label=second-deletion-batch-quarantined-data-unit-records-carry-the-mark device=0 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16814080)
out-pairing-flows.txt:81:PROBE_TIGHT_AREA label=second-deletion-batch-quarantined-data-unit-records-carry-the-mark device=1 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16814080)
out-pairing-flows.txt:113:PROBE_TIGHT_AREA label=three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first device=0 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16818176)
out-pairing-flows.txt:114:PROBE_TIGHT_AREA label=three-unit-file-applied-only-by-its-records-beside-the-version-the-next-mount-applies-first device=1 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16818176)
out-pairing-flows.txt:150:PROBE_TIGHT_AREA label=deletion-through-the-last-batch device=0 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16818176)
out-pairing-flows.txt:151:PROBE_TIGHT_AREA label=deletion-through-the-last-batch device=1 lowest_unit_write_offset=822149120 highest_non_unit_write_end=Some(16818176)
```

### 我的数法输出（`tabulate-out.txt`，在我的草稿目录 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-defense-read/`，8 行原样）

```
tabulate-out.txt:1:CLASSPAIR 1-1 pairs=1 pairs_yi_items_differ=0
tabulate-out.txt:2:CLASSPAIR 1-2 pairs=22 pairs_yi_items_differ=20
tabulate-out.txt:3:CLASSPAIR 1-3 pairs=2 pairs_yi_items_differ=2
tabulate-out.txt:4:CLASSPAIR 2-2 pairs=55 pairs_yi_items_differ=49
tabulate-out.txt:5:CLASSPAIR 2-3 pairs=11 pairs_yi_items_differ=7
tabulate-out.txt:6:MISMATCH_LINES 218
tabulate-out.txt:7:ONLY_IN_SYNTHESIZED {'pool_checker:I-1.13': 12, 'pool_checker:I-3.10': 184, 'pool_checker:I-3.15': 187}
tabulate-out.txt:8:ONLY_IN_ACTUAL {'pool_checker:I-3.1': 3, 'pool_checker:I-3.11': 3}
```

## 二、收严判据 E1–E6 与 D5 在四类取样点与反例 A、B 两臂上成不成立；乙在两层与全组合差不差

判据原文在 `research/prompts/per-write-judging-r2-sonnet-output.md` 第 92 到 98 行（E1 到 E6 与退回规则各一行；D5 是 E5 的同一条）。「成立」= 这一格的读数与该条的设计一致（该拒的拒了、该收的收了）。四类取样点：混合种类的段、写长与头声明不符、两组同被盖、小宇宙全量对照（含阳性对照）。A、B 两臂：`late_unit`（一对副本挪到段 16 开头）、`record_before_units`（段 13 整段四个记录写挪到段 12 单元之前）。真实流 = 删除流与对拍三条流共 24 个单元段。

读探针源码得的两条构造限制（读）：
- 反例 A 只造了一对副本：`.find(|(left_write, right_write)| {`（`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_dynamic_unlink.rs.in:323`）取第一对两盘同偏移同长的写；「两对副本」那一形没造。反例 B 只造了整段记录挪动：`let records_of_the_publish: Vec<usize> = prepared.segments[13].clone();`（同文件 `:336`），「记录只点名一个单元」那一形没造。
- 动态两条测试（小宇宙、两组同被盖）只调 `classifier::groups_of_segment`，没有对挪过的写调 `classify_flow`：`let groups_after = classifier::groups_of_segment(&planted_writes, &prepared.segments[16]);`（同文件 `:196`）。所以这两类取样点上 E1 到 E6 的判定，产物里没有这一格。

### 2.1 判据元素 × 取样点与臂

| 元素 | 混合种类的段 | 写长与头声明不符 | 两组同被盖 | 小宇宙（未改动）与阳性对照 | 反例 A（`late_unit`） | 反例 B（`record_before_units`） | 24 个真实单元段 |
|---|---|---|---|---|---|---|---|
| E1 只有单元写 | **成立**：`out-classifier-and-arms.txt:235` 是 `e1_only_unit_writes=false` | 不涉及：`:198` 段 12 全是 `UnitWrite` | 产物里没有 | 产物里没有 | 不涉及：段 16 全是 `UnitWrite`（`:127`） | 并了记录的是第 11 段，该段 `e1_only_unit_writes=false`（`:160`）；被拒的不是段 12 | 成立：24 段没有混合段（算：删除流 `label=unmodified` 与对拍三条流的 `kinds` 里没有逗号） |
| E2 不盖更早的写、基镜像为零 | 产物里没有（E1 已拒，行里没有 `e2_` 字段） | 0（`:198`） | 产物里没有（读：`plant_over` 把写挪到同盘更早的写的偏移上，按构造 E2 该拒，但没跑） | 未改动段 12、16、20 都是 0（`:85`、`:89`、`:93`）；阳性对照那一格没有 | 0（`:127`）：E2 不拦 A | 0（`:161`）：E2 不拦 B | 成立：`e2_over_an_earlier_write`、`e2_over_nonzero_base` 24 行全是 0（算） |
| E3 头可解、对齐、声明等于写长、落在单元区 | 产物里没有 | **成立**：`e3_claimed_bytes_differ=1`（`:198`） | 产物里没有 | 产物里没有 | 0（`:127`） | 0（`:161`） | 前三半句成立：`e3_without_header`、`e3_unaligned`、`e3_claimed_bytes_differ` 24 行全是 0（算）；「落在单元区」没有机器判（分类器不判），见第三节 |
| E4 头的实例/诞生代号超过挂载根 | 产物里没有 | 0（`:198`） | 产物里没有 | 产物里没有 | **成立**：`e4_not_beyond_mount_root=2`（`:127`） | 0（`:161`）：E4 不拦 B | 成立：24 行全是 0（算）；「前缀里没有根槽写」那一支没走到（每个单元段前面都有根槽写：删除流段 3、6、10 等 `kinds={"RootRecordFua"}`） |
| E5（D5）S=∅ 的读迹与每个写的字节范围不相交 | 产物里没有 | 产物里没有 | 产物里没有 | 产物里没有 | 产物里没有 | 产物里没有 | 产物里没有：库里没有记读包装，探针没实现它（读：`passes` 的各项里没有读迹，`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:267` 起到 `:275`） |
| E6 前缀记录不点名本段的槽 | 产物里没有 | `e6=2`（`:198`），与未改动的段 12 同（`:85`），不分辨 | 产物里没有 | 产物里没有 | 2 变 4，诊断列 `diag_records_of_same_identity=4`（`:127`，对比 `:89` 的 2）：E6 对 A 也有响应 | 2 变 6，诊断列 0 变 4（`:161`，对比 `:85`）：**成立（看增量）** | **不成立**：16 行非 0、每行恰 2、`diag_records_of_same_identity` 24 行全 0（算）；基线就把这 16 段拒了 |
| 退回规则（整段退回） | 成立：整段不进组形（`:235` 没有 `states_group_form`） | 成立（段 12 `passes=false`） | 产物里没有 | 产物里没有 | 成立（段 16 `passes=false`），但基线同 | 成立（段 12 `passes=false`），但基线同 | 16 段被 E6 退回，态数留在今天的引擎态数（`out-classifier-and-arms.txt:109`：1379926285） |

**E6 的基线 2 是什么，产物答不了（推）**：整行里只有「记录写数」，没有记录点名的盘与槽号、被点名单元的类、与哪个写的哪个偏移相交。两种读法产物上都站得住：（一）宽度读法：E6 把每个被点名单元按 32768 字节算（`let named_end = named_start + DATA_UNIT_BYTES_OF_THE_POOL;`，`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:248`；判据原文写的就是这一句，`research/prompts/per-write-judging-r2-sonnet-output.md:97`），而码 2 节点只占 16384（`claimed_bytes: NODE_BYTES_OF_THE_POOL,`，同探针 `:62`），点名一个节点时会多罩住紧挨着的下一个槽；基线的 2 是紧邻槽的误报。（二）真点名：前一发布的记录真点名了这一段要写的槽（「记录先于单元」在今天的真流里就有）。两种读法各自的间接读数：基线命中的记录身份都和本段单元头不同（`diag_records_of_same_identity` 24 行全 0），两种读法都不排除；段 8 命中为 0 是因为它前面没有单元段，其余命中为 0 的段（删除流段 20，对拍流的段 16）为什么是 0，产物里读不出。没有一行把这两种读法分开。

### 2.2 候选乙在「红不红」「红项」两层与逐态真判（全组合）差不差，取样数

「全组合」= 宇宙里所选几组的全部写落与不落的 2^宽度 个状态，每个都真判（`judge_crash_image`），宇宙之外的写一个不落（读：`let universe: Vec<usize> = groups.iter().flatten().copied().collect();` 一族在 `research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_dynamic_unlink.rs.in:97` 起；宽度上限 18，见 `:99`）。乙的合成 = S=∅ 的红项并上每组 S∩组 的红项；甲 = S=∅ 并上每个单写的红项。两层各自的差 = 真判与合成不等的状态数。

| 格 | 取样（量） | 状态数 | 乙 红不红 | 乙 红项 | 甲 红不红 | 甲 红项 | 放行 E 判据吗 |
|---|---|---|---|---|---|---|---|
| 未改动段 12 | 3 次，每次 5 组 10 个写（`out-universes.txt:17`、`:18`、`:19`；该段共 14 组） | 3×1024=3072 | 0 | 0 | 0 | 0 | 不放行（E6=2，`out-classifier-and-arms.txt:85`） |
| 未改动段 16 | 3 次，每次 5 组 10 个写（`:21`、`:22`、`:23`；共 15 组） | 3072 | 0 | 0 | 0 | 0 | 不放行（E6=2，`out-classifier-and-arms.txt:89`） |
| 未改动段 20 | 3 次，每次 5 组 10 个写（`:25`、`:26`、`:27`；共 13 组） | 3072 | 0 | 0 | 0 | 0 | 放行（`out-classifier-and-arms.txt:93` `passes=true`） |
| 阳性对照：段 16 一组挪到段 12 一个单元的两份副本上 | 3 次，每次 5 组 10 个写，含被挪的那一组（`:28`、`:29`、`:30`） | 3072 | 0 | 0 | 0 | 每次 256，共 768 | 不放行（按构造 E2 拒，没跑，读） |
| 两组同被盖：段 12 每一对单元被段 16 两组盖上 | 91 对，每对 2 组 4 个写（`out-two-groups.txt:325`） | 91×16=1456 | 0（91 对全 0） | **243 个状态不同，涉及 78 对**（算，`tabulate-out.txt:1` 到 `:5` 与汇总行） | 0（91 对全 0） | 每对 7，共 637 | 不放行（按构造 E2 拒，没跑，读） |
| 合计（算） | — | 13744 | 0 | 243 | 0 | 1405 | — |

**读法与限定**

1. **红不红一层，乙与甲都没差（13744 个状态上都是 0）**：这一层分不开甲与乙，也分不开「只判 S=∅」，因为所有格 `baseline_red=false`；在未改动段的三格里，产物没有打「有多少个状态是红的」，所以这三格（9216 个状态）里的红不红一层可能是「全绿对全绿」（推：没有这一列）。阳性对照与两组同被盖里真有红状态（甲在这两格里红项不同，`jia_items_differ` 非 0）。
2. **红项一层，乙在没盖更早写的格与阳性对照里 12288 个状态全对；在两组同被盖里 91 对中 78 对不对**，Q9 的预测不中。差的方向两个都有（算，`tabulate-out.txt:7`、`:8`，按每对只打前 3 个状态、218 行）：合成多报 `pool_checker:I-3.10`（184 行）、`pool_checker:I-3.15`（187 行）、`pool_checker:I-1.13`（12 行）；真判多出 `pool_checker:I-3.1`、`pool_checker:I-3.11`（各 3 行）。
3. **按单元类分（算，`tabulate-out.txt:1` 到 `:5`；类标签是各组第一个写的头类，码 1、码 3 是 32768 字节的数据单元，码 2 是 16384 字节的节点，读 `research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:62` 与 `:43`、`:50`）**：1-1 只有 1 对且乙对；1-2 共 22 对、差 20 对；1-3 共 2 对、差 2 对；2-2 共 55 对、差 49 对；2-3 共 11 对、差 7 对。差的不只出在含节点的对（1-3 两对都是 32768 字节的两类单元，两对都差）；这条探针不知道两个被盖的单元之间有没有引用关系，所以「祖先后代遮红项」这个读法在这份产物上既没坐实也没推翻（样本里不含引用关系的标注）。
4. **这些不放行 E 判据的格，判不了乙在判据之内对不对**：判据之内、乙与逐态真判都有量的只有未改动段 20 的 3 次（3072 个状态），而且红项一层是不是「全空对全空」读不出（第 1 点）。在判据之内，同一段里两组有引用关系（新节点指向同段新单元）的取样，这份产物里没有这一格（随机取 5 组，没打是否含引用）。
5. **取样数的量级**：红项一层的差只在 91 对每对 16 个状态里抓到；小宇宙每格只有 3 次随机取 5 组（14、15、13 组里的 5 组，没有穷举 C(组数,5)），所以「没差」读数的置信度只有这 9 次与那 3 次。
6. **按推翻规则的最弱读法记一次输（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列——失败条款打中的时候怎么办」）**：乙的「红项一层与全组合相同」在「两组同被盖」上输了（78 对 243 个状态）；这一格在判据之外，输的是乙的口径，没有输给判据之内的任何一格。收严要写明收严在哪：把「组间独立」限定在「两个被盖的单元之间无引用」，或者让被盖的写一律退回；后一条正是 E2 已经做的。

## 三、前一份报告里的态数前提，这一趟量到了哪几条、哪几条仍是推

前提的原话在 `research/prompts/per-write-judging-r2-sonnet-output.md:30`（总述第 4 点）与 `:139`（第五节「前提」）；派发提示称「第四节」，我按这两处读。每行：这一趟的读数（量 / 算 / 读 / 推）与什么会推翻。

| 前提 | 这一趟 | 读数 | 什么现象推翻 |
|---|---|---|---|
| E3 头可解、码标签是 1/2/3 | **量到** | `e3_without_header` 在 24 个真实单元段上全是 0（算）；分类器对 magic 不对或码不在 1/2/3 的头返回「不可解」（读：`_outside_the_three_unit_classes => None,`，`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:66`） | 某个单元段 `e3_without_header` 非 0 |
| E3 偏移按槽对齐 | **量到** | `e3_unaligned` 24 行全是 0（算） | 同上，非 0 |
| E3 头声明的字节数等于写长（含「写长只有 32768 与 16384 两种」） | **量到，两个方向都有读数** | 24 个真实行 `e3_claimed_bytes_differ` 全是 0（算）；声明值只取 32768 或 16384（读：`:43`、`:50`、`:62`），所以真实流上写长只有这两种；截短臂 `:198` 是 1 | 真实流某段非 0；或截短臂是 0 |
| E3 写落在单元区内 | **半量，仍是推** | 分类器不判这一半；只有写集的分开：四条流每块盘 `lowest_unit_write_offset=822149120`，非单元写最高末尾最大 16826368（`out-classifier-and-arms.txt:71` 与 `out-pairing-flows.txt:80`、`:81`、`:113`、`:114`、`:150`、`:151`）。区边界没读（环与系统配置区的真实上界不等于「今天写过的最高末尾」），所以「单元写会不会落进还没写过的环槽」没有量 | 读出区边界后某个真实单元写落在边界之外，或边界以内有非单元写 |
| E4 实例与诞生代号超过挂载根 | **量到（比较本身）；两支没走到** | 24 真实行 `e4_not_beyond_mount_root` 全是 0；A 臂 `:127` 是 2。没走到的：「前缀里没有根槽写」那一支（读：`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_classifier.rs.in:188` 的注释说它按「不超」处理；真实流每个单元段前面都有根槽写，删除流 `out-classifier-and-arms.txt:76`、`:79`、`:83` 是段 3、6、10）；写表里的挂载根与池级 checker 的挂载根是不是同一个（推，上一份报告 `:105` 的说法没有这一趟的读数） | 构造一条单元段前没有根槽写的流，看 E4 是不是整段退回；或池级 checker 的挂载根与写表读出的不同 |
| E6 的字节半句（点名项解析） | **量到：能解析；但前一份报告的推「今天的流上点名的是更早发布的单元」（`research/prompts/per-write-judging-r2-sonnet-output.md:104`）被顶回去** | B 臂 E6 增量 +4 且 `diag_records_of_same_identity=4`（`out-classifier-and-arms.txt:161`，对比 `:85`），说明头 496 字节、每项 56 字节、两份位置的解法能认出点名；A 臂 +2（`:127` 对比 `:89`）。但真实流 16/24 段 E6=2、同身份诊断全 0：所谓「只有记录点名了从没写过的槽时它才独立拦」（`research/prompts/per-write-judging-r2-sonnet-output.md:30`）在这一趟里变成了「它每隔几段就拦今天的正常流」；原因是宽度读法还是真点名，产物分不开（第二节） | 打出每个 E6 命中的记录点名的（盘，槽号）、被点名单元的类与相交的写偏移：全是紧邻槽（宽度造成）则按类取宽度重算；有真重叠则 E6 的判法要重新说 |
| E5（D5） | **仍是推** | 探针没实现；每格「产物里没有」 | 实现记读包装后某个被收下的段上读迹相交 |
| 每个偏移恰两个写（组大小） | **量到组大小；每个偏移的恰两个写只量到一半** | `group_sizes={2: N}`：删除流与对拍三条流 24 行加四条臂 27 行共 51 行全部只有 2 一个键（算，`grep -o 'group_sizes={[^}]*}'`）；N 与写数一致（段 12：28 个写、14 组，`out-classifier-and-arms.txt:85`）。分组是「同偏移或同归并键」两条边连通，所以 51 行说的是「每组恰两个写」；「每个偏移恰两个写」没单独打（反例 A 的构造 `.expect("第 12 段有两盘同偏移的一对副本")` 找到了一对，读：`research/prompts/per-write-judging-r2-sonnet-model/probe/tightened_dynamic_unlink.rs.in:323` 起） | 某段 `group_sizes` 出现 2 以外的键，或有写只和同归并键的写成组、没有同偏移的对子 |

另外这一趟把前一份报告里「算的、前提待量」的三类数中的两类升成了量：对拍三条流的引擎总数（`out-pairing-flows.txt:79`、`:112`、`:149`，与前一份报告 Q6 里写的三个数逐位相同）与单元段数（5、6、6）；段表里每个单元段的组数 N（上表）。乙的态数那一列没升成量：被 E6 挡掉的段按今天的数留着，要等 E6 修正后重跑。

## 四、给判决的读法（辩方）与什么现象会推翻

1. **E1 到 E4 站得住，E5 没有读数，E6 的字面打中不分辨臂。** E1、E3、E4 在各自该拒的臂上拒了，在 24 个真实单元段上全放行；E2 在真实流与四条臂上都是 0，它不拦 A、B，拦的是「盖更早写」那一类（这一趟只在阳性对照与两组同被盖里按构造用到，没跑分类器）。E6 在 16/24 个真实段上命中，A、B、写长三臂改动的段在基线里就已被它拒掉，所以它不分辨臂，按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 的「打中不分辨臂」处理：病根是各条臂共用的前提，不拿它判臂，只看分辨得出臂的那几列（E4、E3、E6 增量与诊断）。推翻：打出命中记录的（盘，槽号）与类之后，若命中是真点名而不是宽度，则 E6 不是「写错了宽度」而是在今天的真流里拦着一种真实形态，要另立一题。
2. **E6 要改，只能是改一个新条而不是「澄清」**：E6 是前一份报告新加的一条（没被攻过），按字面输了一次（16/24 段退回，态数从预测的 471 变成 1379926285）。任何把宽度按类取、或把「点名槽」换成「同身份」的改法，都比字面要松，要按「记一次输、只许收严、写明收严在哪」另走一轮，不在这一趟里直接写进判据。
3. **乙：红不红一层在这一趟全部格上与逐态真判无差；红项一层在判据之外的两组同被盖上有差（78/91 对，243/1456 个状态）；判据之内只有段 20 一格有量，且那一格红项是否「全空」读不出。** 乙不能写成「判据内两层都与全组合相同」；能写的是「红不红：13744 个状态 0 差；红项：没盖更早写的 12288 个状态 0 差」。推翻：打出每格红状态数，若段 20 那三次有红状态而仍 0 差，则判据之内红项一层的读数才算有牙；若判据之内、同段两组有引用的取样出现差，乙的组间独立在判据之内也不成立。
4. **反例 A、B 与各候选（只判 S=∅、甲、乙）的格这一趟仍是推。** 这一趟的 A、B 臂只做静态判定、不枚举状态（`probe_tightened_criterion_over_the_unlink_stream_and_two_misordered_arms` 里 `engine_states=None`），所以「候选在 A、B 的状态上红不红、红项」没有读数；上一份报告 `research/prompts/per-write-judging-r2-sonnet-output.md:66` 到 `:73` 那张格子仍是推的。能读到的只有「判据在 A、B 的那一对副本、那一整段记录上拒不拒」：A 靠 E4（2），B 靠 E6 增量（2 变 6，诊断 0 变 4）。只造了 A 的一对副本、B 的整段记录，「两对副本」「点名一个单元」两形没有读数。
5. **取样的覆盖要说清**：对判据放行的段（删除流段 8、20，对拍三条流各自的段 8 与 16），动态对照只做了删除流段 20；段 8 与对拍三条流没有逐态对照。

## 要主 agent 跑的命令

没有。这一趟只读产物，没写新探针、没改模型、没起命令；上一份报告的整份复跑已由任务 8700 跑完。主 agent 若要重跑那份脚本，注意 `tools/rerun.sh` 第 95 行的汇总用 `grep -h '^PROBE_'`，会漏每个测试第一行（和 `test … ... ` 同一行的那一条）；重跑前改成不带 `^` 的 `grep -h 'PROBE_'` 才完整（读：脚本现取 `grep -nF "grep -h '^PROBE_'" research/prompts/per-write-judging-r2-sonnet-model/tools/rerun.sh`）。继续做的话，要新派同立场的腿（派发提示带「承接：research/prompts/per-write-judging-r2-sonnet-s2-output.md」），它的探针需要补的读数（本报告里「产物里没有这一格」的几处）：E6 命中的（盘，槽号）、被点名单元的类与相交写偏移；对挪过的写跑分类器；每格红状态数；A 的两对副本与 B 的点名一个单元；判据放行的段上的逐态对照。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务；判决要引的数没复跑，本报告里的读数在主 agent 复跑出之前对判决都是「线索」：产物是任务 8700 主 agent 跑的，我读的是它的原样输出。
- 没读禁读清单里的两份攻方报告；没碰别的腿这一轮的报告。
- 没起编译与探针，没建仓副本与编译目录；我的草稿目录里只有 `log8700.txt`、`tools/tabulate.py`、`tabulate-out.txt`、`real-unit-lines.txt`、`cite.txt`、`cite2.txt`（`real-unit-lines.txt` 的 sha256 `b6bad10a0828712d664dbe01cedd46ffe8d2533dbdb614089029c87314260ddb`），`du -sh` 84K，不用清；主 agent 的复跑目录（50M，里面有 `tree/`）不是我建的，我没动。
- 没读 `crates/` 下的检查器与核心代码（E3「落在单元区」的区边界、E6 点名项的真实字段宽度、记录与单元的发布次序都没读源码）；上面凡写「推」的都是这个原因。
- E5（读迹）没有实现，没有任何读数；「每个偏移恰两个写」只量到每组恰两个写。
- 没分辨 E6 基线命中是宽度误报还是真点名；没验两对副本形的 A、点名一个单元形的 B；没量候选在 A、B 状态上的两层。
- 没做 git 写操作。
