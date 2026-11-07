# 逐写独立判第二轮攻方腿承接：读复跑产物、对着跑前预测判（opus，2026-11-06）

轮名 per-write-judging-r2；批次 records/2026-11-05-删轻型并入重型.md#并包；承接 `research/prompts/per-write-judging-r2-opus-output.md`（下称「前一份报告」）。攻击面同前一条攻方腿（第二轮分工「第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方」）。这一条只读产物，对着前一份报告第二到第四节跑前写死的 R1–R14、X1–X6 判；没改预测、没改模型、没跑新东西。

**数的来历**：下面每个数都是量的——主 agent 在副本上整份复跑前一条腿的模型（任务 8699，`type=succeeded exit=0`，第 3 次尝试），由我读产物抄来。它们是副本装置上的数，照 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节不进 kb。标「推的」的是我对产物的解释，产物里没有逐态可对的行。

产物目录 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/rerun-work/out/`（主 agent 给的），四段：

| 文件 | 行数（wc -l） | sha256 |
|---|---|---|
| `counterexamples.txt` | 440 | `a8f8d355187003cd3fa978df5ef897d5c3aa28c3367f48416334564833bf0154` |
| `user_actions.txt` | 645 | `c5441f87827ee203c28fb7004e3851b8250e6f39be228cc6056ff1933432bf6d` |
| `unseen_shapes.txt` | 447 | `22a34116fdc68c6c152fd53534d1628b542bf344b20783019ed680c21c0443ee` |
| `small_segments_exhaustive.txt` | 792 | `efe1d874e7744686b68b58592f734401ebe3e43d511c1b463e3b956f930bc7ee` |

任务日志我存了一份在草稿目录（`task-server/task log 8699 --all` 原样输出，231 行，`task-8699.log`）。四段各自跑完：「name=rerun_part part=counterexamples lines=440」（`task-8699.log:119`）；「name=rerun_part part=user_actions lines=645」（`task-8699.log:153`）；「name=rerun_part part=unseen_shapes lines=447」（`task-8699.log:193`）；「name=rerun_part part=small_segments_exhaustive lines=792」（`task-8699.log:215`）。

引产物一律写成「整行」（`文件名:行号`），文件名指上面那个目录里的文件（`task-8699.log` 指草稿目录里的）。整行抄，带 `checker=` 长串的行也照抄。四段里 `unrepresentable` 不为 0 的行数是 0（`grep -c 'unrepresentable=[1-9]'` 四个文件各报 0），所以四种代表法在每个被比的态上都凑得出，前一份报告第一节「`unrepresentable` 应恒为 0，不为 0 说明取样写错」那一条没打中。

## 各格判定一览

| 格 | 判定 | 依据（细节在各节） |
|---|---|---|
| 预测 R1–R14 | 13 条中，R7 不中 | 第一节。R4 红不红层中、红项层有预测外的差（pair 族 4、device_lattice 族 1734）；R13 的机理句（「只差一份记录副本」那几态）产物里没有逐态行可对 |
| 预测 X1–X6 | 5 条中，X6 不中 | 第一节。X6 要的态（两组盖写都整组落）是那个小段的全集，按切法归下一段的空集，不在小段被判的 14 个态里 |
| 反例 A 一对（`late_unit`，三种 C2 + 小段全量） | 甲两种合成、乙按并都打中，乙按增减不中（红不红、红项两层都 0） | 第二节表。四种代表法的错全是「全组合判绿、代表判红」；没有一态是漏红 |
| 反例 A 两对（`late_two_units`，取样 + 小段全量） | 四种合成都打中，四种的差数逐格相同 | 第二节表。打中不分辨臂：病根在四种共用的「只判空集、单写与组内子态」，判别要的「两组都整组落」不在任何一种判的态里 |
| 反例 B（`record_before_units`，三种 C2 + 小段全量） | 没造出来：每段 `red=0`，四种代表法差全是 0 | 第二节表、第三节 L2。只抽了这一次，照「否定结论一次不算」不支撑「没有」 |
| L1–L4 | L1 量到（同反例 A）；L2 没造出；L3 搬迁副本单写翻红坐实、四种代表法不差；L4 取样 48703 态全绿 | 第三节 |
| 4 类敏感取样点 | 混合种类段两条臂红项层都有差（`units_records_root` 四种都漏 I-9.1）；多组被盖：甲漏项、乙多项；小段全量见 X1–X6；搬迁半类全不差 | 第三节 |
| 判据收下的段上差出在哪 | 收下（`readable_criterion_accepts=true`）的段里，四种代表法有差的 6 段空集态全红；空集态绿而有差的段是 0 | 第三节末，草稿目录 `accepted-segments.txt`。取样所得，不是全量 |
| 方法的限定 | 前三段全是取样（每段 `judged_is_full_domain=false`，判的态是被判段全量的 0.008%–0.024%）；小段全量是加了屏障的历史 | 第四节 |

## 一、跑前预测逐条判（前一份报告第二节、第四节的 R1–R14、X1–X6）

判法：按预测字面逐句对产物行；预测里「=0」「>0」没写是哪一列的，按它所在那一句点名的列读（多为 `red_or_not_differs`），另一列有差的另记「预测外」。

### R1 中

三种 C2 的 `unmodified` 每段 `family=all red=0`、四种代表法两种差都是 0、单写与空集不差：

- 「PROTO_SINGLES arm=overwrite/unmodified segment=12 differing_from_empty=0」（`counterexamples.txt:42`）
- 「PROTO_FAMILY arm=overwrite/unmodified segment=12 family=all states=13439 red=0 whole_differs_from_empty=0」（`counterexamples.txt:78`）
- 「PROTO_SINGLES arm=overwrite/unmodified segment=16 differing_from_empty=0」（`counterexamples.txt:85`）
- 「PROTO_FAMILY arm=overwrite/unmodified segment=16 family=all states=13439 red=0 whole_differs_from_empty=0」（`counterexamples.txt:121`）
- 「PROTO_SINGLES arm=sequential_write_two_data_units/unmodified segment=12 differing_from_empty=0」（`user_actions.txt:61`）
- 「PROTO_FAMILY arm=sequential_write_two_data_units/unmodified segment=12 family=all states=13440 red=0 whole_differs_from_empty=0」（`user_actions.txt:97`）
- 「PROTO_SINGLES arm=sequential_write_two_data_units/unmodified segment=16 differing_from_empty=0」（`user_actions.txt:104`）
- 「PROTO_FAMILY arm=sequential_write_two_data_units/unmodified segment=16 family=all states=15007 red=0 whole_differs_from_empty=0」（`user_actions.txt:140`）
- 「PROTO_SINGLES arm=new_inode/unmodified segment=12 differing_from_empty=0」（`user_actions.txt:355`）
- 「PROTO_FAMILY arm=new_inode/unmodified segment=12 family=all states=13440 red=0 whole_differs_from_empty=0」（`user_actions.txt:391`）
- 「PROTO_SINGLES arm=new_inode/unmodified segment=16 differing_from_empty=0」（`user_actions.txt:398`）
- 「PROTO_FAMILY arm=new_inode/unmodified segment=16 family=all states=4176 red=0 whole_differs_from_empty=0」（`user_actions.txt:434`）

四种代表法的 `family=all` 行：`counterexamples.txt:79-82`、`counterexamples.txt:122-125`、`user_actions.txt:98-101`、`user_actions.txt:141-144`、`user_actions.txt:392-395`、`user_actions.txt:435-438`，每行 `red_or_not_differs=0 red_items_differ=0 unrepresentable=0`（区间裸写，行太多不逐行抄；主 agent 可 `sed -n` 核）。

### R2 中

- 「PROTO_SEGMENT arm=overwrite/late_unit segment=16 writes=26 kinds=UnitWrite over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=true copy_groups=13 tearable=0 judged=true full_domain_states=67108863 judged_states=15955 judged_is_full_domain=false」（`counterexamples.txt:17`）
- 「PROTO_SEGMENT arm=overwrite/late_two_units segment=16 writes=28 kinds=UnitWrite over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=true copy_groups=14 tearable=0 judged=true full_domain_states=268435455 judged_states=15028 judged_is_full_domain=false」（`counterexamples.txt:26`）

### R3 中（四个分句全中）

`overwrite/late_unit` 的 C2 单元段（段 16）。空集态红项非空：「PROTO_BASELINE arm=overwrite/late_unit segment=16 red_items={"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4"} whole=consulted=failed@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=1 ignored=failed@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=1 checker=HHHHHHHHHVHHHHHHHHHVHHHHVHVHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHPHHHHHNHNNNNNNNNHHHHNNNNHNNVHHHNHNHPHHNHHHHHNNHHH records=false/false」（`counterexamples.txt:170`）

目标态 {x0,x1} 绿：「PROTO_TARGET arm=overwrite/late_unit segment=16 target=moved_copies_only state=[0,1] actual={} whole_equals_empty=false per_write_union=Some({"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4"}) copy_group_union=Some({"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4"})」（`counterexamples.txt:228`）

三种并/甲在 `group_lattice` 族 4095、`pair` 族 1、`all_but_one` 族 24、`device_lattice` 族 0（红不红层）：

- 「PROTO_FAMILY arm=overwrite/late_unit segment=16 family=group_lattice states=8190 red=4095 whole_differs_from_empty=4095」（`counterexamples.txt:194`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=group_lattice representation=per_write_union compared=8190 red_or_not_differs=4095 red_items_differ=4095 unrepresentable=0」（`counterexamples.txt:195`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=group_lattice representation=per_write_delta compared=8190 red_or_not_differs=4095 red_items_differ=4095 unrepresentable=0」（`counterexamples.txt:196`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=group_lattice representation=copy_group_union compared=8190 red_or_not_differs=4095 red_items_differ=4095 unrepresentable=0」（`counterexamples.txt:197`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=pair representation=per_write_union compared=325 red_or_not_differs=1 red_items_differ=49 unrepresentable=0」（`counterexamples.txt:185`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=all_but_one representation=per_write_union compared=26 red_or_not_differs=24 red_items_differ=26 unrepresentable=0」（`counterexamples.txt:190`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=device_lattice representation=per_write_union compared=6477 red_or_not_differs=0 red_items_differ=3258 unrepresentable=0」（`counterexamples.txt:200`）

`copy_group_delta` 各族两种差都是 0：

- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=support representation=copy_group_delta compared=39 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:178`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=pair representation=copy_group_delta compared=325 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:188`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=all_but_one representation=copy_group_delta compared=26 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:193`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=group_lattice representation=copy_group_delta compared=8190 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:198`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=device_lattice representation=copy_group_delta compared=6477 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:203`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=target representation=copy_group_delta compared=1 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:213`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=all representation=copy_group_delta compared=15954 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:218`）

预测外的一条（红项层）：两种并合成在 `device_lattice` 族红项差 3258（第 200、202 行），`per_write_delta` 那一族是 0（第 201 行）。

### R4 红不红层中；红项层不中（预测外的差）

`overwrite/late_two_units` 段 16。目标态四份绿：「PROTO_TARGET arm=overwrite/late_two_units segment=16 target=moved_copies_only state=[0,1,2,3] actual={} whole_equals_empty=false per_write_union=Some({"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4"}) copy_group_union=Some({"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4"})」（`counterexamples.txt:334`）

`copy_group_delta` 在 `group_lattice`、`all_but_one`、`pair`、`device_lattice` 四族：

- 「PROTO_FAMILY arm=overwrite/late_two_units segment=16 family=group_lattice states=6423 red=4821 whole_differs_from_empty=3254」（`counterexamples.txt:297`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=group_lattice representation=copy_group_delta compared=6423 red_or_not_differs=1602 red_items_differ=1602 unrepresentable=0」（`counterexamples.txt:301`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=all_but_one representation=copy_group_delta compared=28 red_or_not_differs=24 red_items_differ=28 unrepresentable=0」（`counterexamples.txt:296`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=pair representation=copy_group_delta compared=378 red_or_not_differs=0 red_items_differ=4 unrepresentable=0」（`counterexamples.txt:291`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=device_lattice representation=copy_group_delta compared=7217 red_or_not_differs=0 red_items_differ=1734 unrepresentable=0」（`counterexamples.txt:306`）

`group_lattice` 族 1602 > 1000、约为 6423 的四分之一，与预测的量级合；「等于含两对四份的态数」产物没有逐态行可数。能算的是：6423 − 4821 = 1602，即这一族的绿态全被 `copy_group_delta` 判红（推的：绿态恰是「两对四份都落」的那些，因为只有两对都落时前缀指的单元才全在）。`pair` 族、`device_lattice` 族 `red_or_not_differs=0` 照预测，但 `red_items_differ` 是 4 与 1734，预测没说到；按「两列都为 0」这一最弱读法，R4 在这两族的红项层记不中。

预测外的另一条：这一段四种代表法逐族的两种差完全相同（第 298–301、318–321 行），`copy_group_delta` 并不比别的三种少错一态。单写那两行只出现在第二对上（第一对的单写与空集整份相同）：

- 「PROTO_SINGLE_DIFFERS arm=overwrite/late_two_units segment=16 state=[2] write=UnitWrite@dev0+823361536x32768 red_items={"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4"} red_items_equal_empty=false」（`counterexamples.txt:274`）
- 「PROTO_SINGLES arm=overwrite/late_two_units segment=16 differing_from_empty=2」（`counterexamples.txt:276`）

### R5 中

`record_checker:claimed_state_missing_unit` 在 `counterexamples.txt`、`user_actions.txt` 里出现 0 次（`grep -c` 两个文件各报 0）；四段里出现它的行只在 `planted_two_groups` 与它的小段（15 行、6 行）和 `PROTO_PLANT_CHOICE` 行上，三条 late 臂一次都没有。

### R6 中

`overwrite/record_before_units` 段 17：

- 「PROTO_BASELINE arm=overwrite/record_before_units segment=17 red_items={} whole=consulted=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 ignored=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 checker=HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHHHHHHHNHNNNNNNNNHHHHNNNNHNNHHHHNHNHHHHNHHHHHNNHHH records=false/false」（`counterexamples.txt:379`）
- 「PROTO_FAMILY arm=overwrite/record_before_units segment=17 family=all states=13439 red=0 whole_differs_from_empty=0」（`counterexamples.txt:421`）
- 「PROTO_COMPARE arm=overwrite/record_before_units segment=17 family=all representation=per_write_union compared=13439 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:422`）
- 「PROTO_COMPARE arm=overwrite/record_before_units segment=17 family=all representation=copy_group_delta compared=13439 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`counterexamples.txt:425`）
- 「PROTO_TARGET arm=overwrite/record_before_units segment=17 target=every_group_but_822214656 state=[2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23] actual={} whole_equals_empty=true per_write_union=Some({}) copy_group_union=Some({})」（`counterexamples.txt:426`）

十二个「只缺一整组」的目标态全绿、与空集整份相同（第 426–437 行）。

### R7 不中

`sequential_write_two_data_units/record_before_units` 段 17：`red=0`，四种代表法不差。推翻观测「`red=0`」出现了。

- 「PROTO_ARM arm=sequential_write_two_data_units/record_before_units moved=C2 的记录段（JournalRecord@dev0+16793600x4096、JournalRecord@dev1+16793600x4096、JournalRecord@dev0+16797696x4096、JournalRecord@dev1+16797696x4096）挪到 C2 单元段之前 judged_segments=[12, 17] judged_root_index=104 torn_images=0」（`user_actions.txt:22`）
- 「PROTO_BASELINE arm=sequential_write_two_data_units/record_before_units segment=17 red_items={} whole=consulted=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 ignored=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 checker=HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHHHHHHHNHNNNNNNNNHHHHNNNNHNNHHHHNHNHHHHNHHHHHNNHHH records=false/false」（`user_actions.txt:292`）
- 「PROTO_FAMILY arm=sequential_write_two_data_units/record_before_units segment=17 family=all states=15065 red=0 whole_differs_from_empty=0」（`user_actions.txt:334`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/record_before_units segment=17 family=all representation=per_write_union compared=15065 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:335`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/record_before_units segment=17 family=all representation=copy_group_delta compared=15065 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:338`）
- 「PROTO_TARGET arm=sequential_write_two_data_units/record_before_units segment=17 target=every_group_but_822214656 state=[2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27] actual={} whole_equals_empty=true per_write_union=Some({}) copy_group_union=Some({})」（`user_actions.txt:339`）

记录段确实有两条记录、四份（第 22 行 `moved=` 列了 `16793600` 与 `16797696` 两处各两盘），十四个「只缺一整组」的目标态全绿、与空集整份相同（第 339–352 行）。按前一份报告 R7 的「推翻之后怎么改」：「顺序写也凑不出反例 B」，反例 B 降为「只在某种点名形状上」，写进没打中的形状。为什么绿，产物不答（推的两种可能：这些态里根没落，只落了记录的那个前缀是恢复认得的合法结局；或每条记录点名的单元不是我读的那样）。

### R8 中

`new_inode/record_before_units` 段 17 与 R6 同（空集绿、`red=0`、四种不差、十个「只缺一整组」目标态全绿，第 633–642 行）：

- 「PROTO_BASELINE arm=new_inode/record_before_units segment=17 red_items={} whole=consulted=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 ignored=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 checker=HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHHHHHHHNHNNNNNNNNHHHHNNNNHNNHHHHNHNHHHHNHHHHHNNHHH records=false/false」（`user_actions.txt:586`）
- 「PROTO_FAMILY arm=new_inode/record_before_units segment=17 family=all states=4177 red=0 whole_differs_from_empty=0」（`user_actions.txt:628`）
- 「PROTO_COMPARE arm=new_inode/record_before_units segment=17 family=all representation=per_write_union compared=4177 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:629`）
- 「PROTO_COMPARE arm=new_inode/record_before_units segment=17 family=all representation=copy_group_delta compared=4177 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:632`）

### R9 中

两种 C2 上 `late_unit` 与 R3 同形：三种并/甲不为 0，`copy_group_delta` 仍是 0；`group_lattice` 族的数随 C2 的组数变（顺序写 15 组，随机取，3563；建 inode 11 组，全取，1023）。

- 「PROTO_FAMILY arm=sequential_write_two_data_units/late_unit segment=16 family=all states=16405 red=12547 whole_differs_from_empty=8227」（`user_actions.txt:233`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/late_unit segment=16 family=all representation=per_write_union compared=16405 red_or_not_differs=3858 red_items_differ=8227 unrepresentable=0」（`user_actions.txt:234`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/late_unit segment=16 family=all representation=per_write_delta compared=16405 red_or_not_differs=3858 red_items_differ=3858 unrepresentable=0」（`user_actions.txt:235`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/late_unit segment=16 family=all representation=copy_group_union compared=16405 red_or_not_differs=3858 red_items_differ=8227 unrepresentable=0」（`user_actions.txt:236`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/late_unit segment=16 family=all representation=copy_group_delta compared=16405 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:237`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/late_unit segment=16 family=group_lattice representation=per_write_union compared=7260 red_or_not_differs=3563 red_items_differ=3563 unrepresentable=0」（`user_actions.txt:214`）
- 「PROTO_COMPARE arm=sequential_write_two_data_units/late_unit segment=16 family=group_lattice representation=copy_group_delta compared=7260 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:217`）
- 「PROTO_FAMILY arm=new_inode/late_unit segment=16 family=all states=7269 red=5958 whole_differs_from_empty=3868」（`user_actions.txt:527`）
- 「PROTO_COMPARE arm=new_inode/late_unit segment=16 family=all representation=per_write_union compared=7269 red_or_not_differs=1311 red_items_differ=3868 unrepresentable=0」（`user_actions.txt:528`）
- 「PROTO_COMPARE arm=new_inode/late_unit segment=16 family=all representation=per_write_delta compared=7269 red_or_not_differs=1311 red_items_differ=1311 unrepresentable=0」（`user_actions.txt:529`）
- 「PROTO_COMPARE arm=new_inode/late_unit segment=16 family=all representation=copy_group_union compared=7269 red_or_not_differs=1311 red_items_differ=3868 unrepresentable=0」（`user_actions.txt:530`）
- 「PROTO_COMPARE arm=new_inode/late_unit segment=16 family=all representation=copy_group_delta compared=7269 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:531`）
- 「PROTO_COMPARE arm=new_inode/late_unit segment=16 family=group_lattice representation=per_write_union compared=2046 red_or_not_differs=1023 red_items_differ=1023 unrepresentable=0」（`user_actions.txt:508`）
- 「PROTO_COMPARE arm=new_inode/late_unit segment=16 family=group_lattice representation=copy_group_delta compared=2046 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`user_actions.txt:511`）

两种 C2 的目标态 {x0,x1} 都绿（第 247、541 行），`copy_group_delta` 各族 0/0（第 197–237 行里每五行一行、第 491–531 行同）。

### R10 中

- 「PROTO_SEGMENT arm=overwrite/relocation_copies segment=16 writes=26 kinds=UnitWrite over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=true copy_groups=13 tearable=0 judged=true full_domain_states=67108863 judged_states=15922 judged_is_full_domain=false」（`unseen_shapes.txt:34`）
- 「PROTO_FAMILY arm=overwrite/relocation_copies segment=16 family=all states=15921 red=0 whole_differs_from_empty=0」（`unseen_shapes.txt:343`）
- 「PROTO_COMPARE arm=overwrite/relocation_copies segment=16 family=all representation=per_write_union compared=15921 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`unseen_shapes.txt:344`）
- 「PROTO_COMPARE arm=overwrite/relocation_copies segment=16 family=all representation=copy_group_delta compared=15921 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`unseen_shapes.txt:347`）
- 「PROTO_TARGET arm=overwrite/relocation_copies segment=16 target=relocation_copies_only state=[0,1] actual={} whole_equals_empty=true per_write_union=Some({}) copy_group_union=Some({})」（`unseen_shapes.txt:348`）

### R11 中

空集态红项含 I-2.1；两个搬迁副本各一行单写差、各多出 I-1.13；四种代表法 `red_or_not_differs=0`，`per_write_union` 红项差 0：

- 「PROTO_BASELINE arm=overwrite/relocation_copies_over_a_missing_source segment=16 red_items={"pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.4"} whole=consulted=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 ignored=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 checker=HHHHHHHHHVHHHHHHHHHVHHHHHHVHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHHHHHHHNHNNNNNNNNHHHHNNNNHNNHHHHNHNHHHHNHHHHHNNHHH records=false/false」（`unseen_shapes.txt:394`）
- 「PROTO_SINGLE_DIFFERS arm=overwrite/relocation_copies_over_a_missing_source segment=16 state=[0] write=UnitWrite@dev0+823803904x32768 red_items={"pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.4"} red_items_equal_empty=false」（`unseen_shapes.txt:395`）
- 「PROTO_SINGLE_DIFFERS arm=overwrite/relocation_copies_over_a_missing_source segment=16 state=[1] write=UnitWrite@dev1+823803904x32768 red_items={"pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.4"} red_items_equal_empty=false」（`unseen_shapes.txt:396`）
- 「PROTO_FAMILY arm=overwrite/relocation_copies_over_a_missing_source segment=16 family=all states=15900 red=15900 whole_differs_from_empty=8098」（`unseen_shapes.txt:438`）
- 「PROTO_COMPARE arm=overwrite/relocation_copies_over_a_missing_source segment=16 family=all representation=per_write_union compared=15900 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`unseen_shapes.txt:439`）
- 「PROTO_COMPARE arm=overwrite/relocation_copies_over_a_missing_source segment=16 family=all representation=copy_group_delta compared=15900 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`unseen_shapes.txt:442`）

### R12 中

- 「PROTO_SEGMENT arm=overwrite/rotation_with_units segment=15 writes=26 kinds=UnitWrite+SystemConfigurationSlot over_an_earlier_write=2 over_nonzero_base=2 readable_criterion_accepts=false copy_groups=13 tearable=2 judged=true full_domain_states=150994943 judged_states=48704 judged_is_full_domain=false」（`unseen_shapes.txt:17`）
- 「PROTO_BASELINE arm=overwrite/rotation_with_units segment=15 red_items={} whole=consulted=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 ignored=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 checker=HHHHHHHHHHHHHHHHHHHHHHHHHHHHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHHHHHHHNHNNNNNNNNHHHHNNNNHNNHHHHNHNHHHHNHHHHHNNHHH records=false/false」（`unseen_shapes.txt:92`）
- 「PROTO_FAMILY arm=overwrite/rotation_with_units segment=15 family=all states=48703 red=0 whole_differs_from_empty=0」（`unseen_shapes.txt:129`）
- 「PROTO_COMPARE arm=overwrite/rotation_with_units segment=15 family=all representation=per_write_union compared=48703 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`unseen_shapes.txt:130`）
- 「PROTO_COMPARE arm=overwrite/rotation_with_units segment=15 family=all representation=copy_group_delta compared=48703 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`unseen_shapes.txt:133`）

`readable_criterion_accepts=false`、`tearable=2`、`full_domain_states=150994943` 都照预测；判了 48703 个取样态，全绿。

### R13 中（机理句产物里没有可对的行）

- 「PROTO_SEGMENT arm=overwrite/units_records_root segment=16 writes=27 kinds=UnitWrite+JournalRecord+RootRecordFua over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=false copy_groups=14 tearable=0 judged=true full_domain_states=134217727 judged_states=14635 judged_is_full_domain=false」（`unseen_shapes.txt:26`）
- 「PROTO_FAMILY arm=overwrite/units_records_root segment=16 family=all states=14634 red=5601 whole_differs_from_empty=5604」（`unseen_shapes.txt:216`）
- 「PROTO_COMPARE arm=overwrite/units_records_root segment=16 family=all_but_one representation=per_write_union compared=27 red_or_not_differs=2 red_items_differ=26 unrepresentable=0」（`unseen_shapes.txt:197`）
- 「PROTO_COMPARE arm=overwrite/units_records_root segment=16 family=all_but_one representation=per_write_delta compared=27 red_or_not_differs=2 red_items_differ=26 unrepresentable=0」（`unseen_shapes.txt:198`）
- 「PROTO_COMPARE arm=overwrite/units_records_root segment=16 family=all_but_one representation=copy_group_union compared=27 red_or_not_differs=2 red_items_differ=26 unrepresentable=0」（`unseen_shapes.txt:199`）
- 「PROTO_COMPARE arm=overwrite/units_records_root segment=16 family=all_but_one representation=copy_group_delta compared=27 red_or_not_differs=2 red_items_differ=26 unrepresentable=0」（`unseen_shapes.txt:200`）
- 「PROTO_SINGLE_DIFFERS arm=overwrite/units_records_root segment=16 state=[26] write=RootRecordFua@dev0+7344128x512 red_items={"journal_consulted_oracle:recovered_to_an_older_root", "journal_ignored_oracle:read_failed", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:root_without_record"} red_items_equal_empty=false」（`unseen_shapes.txt:179`）

`all_but_one` 族 27 态里红 24、绿 3，四种代表法各差 2 态。预测的机理（「只差一份记录副本」那几态全组合绿、根单写红，代表留着它）产物里没有逐态行，差的那 2 态是哪 2 态不知道（推的）。预测外的一条见第三节：这一段四种代表法在红项层都漏 I-9.1。

### R14 中

- 「PROTO_SEGMENT arm=overwrite/planted_two_groups segment=16 writes=28 kinds=UnitWrite over_an_earlier_write=4 over_nonzero_base=0 readable_criterion_accepts=false copy_groups=14 tearable=0 judged=true full_domain_states=268435455 judged_states=15057 judged_is_full_domain=false」（`unseen_shapes.txt:29`）
- 「PROTO_PLANT_CHOICE flow=overwrite node=UnitWrite@dev0+823345152x16384、UnitWrite@dev1+823345152x16384 file_unreadable=true red_items={"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:claimed_state_missing_unit"}」（`unseen_shapes.txt:3`）
- 「PROTO_FAMILY arm=overwrite/planted_two_groups segment=16 family=target states=3 red=3 whole_differs_from_empty=3」（`unseen_shapes.txt:275`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups segment=16 family=group_lattice representation=copy_group_union compared=6402 red_or_not_differs=0 red_items_differ=1589 unrepresentable=0」（`unseen_shapes.txt:263`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups segment=16 family=all representation=per_write_union compared=15056 red_or_not_differs=0 red_items_differ=5219 unrepresentable=0」（`unseen_shapes.txt:281`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups segment=16 family=all representation=per_write_delta compared=15056 red_or_not_differs=0 red_items_differ=5219 unrepresentable=0」（`unseen_shapes.txt:282`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups segment=16 family=all representation=copy_group_union compared=15056 red_or_not_differs=0 red_items_differ=1685 unrepresentable=0」（`unseen_shapes.txt:283`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups segment=16 family=all representation=copy_group_delta compared=15056 red_or_not_differs=0 red_items_differ=1685 unrepresentable=0」（`unseen_shapes.txt:284`）

三个目标态都红（第 297–299 行）。祖先组遮后代组的项那一格的具体态：「PROTO_TARGET arm=overwrite/planted_two_groups segment=16 target=both_planted_groups state=[24,25,26,27] actual={"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:claimed_state_missing_unit"} whole_equals_empty=false per_write_union=Some({"pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.4"}) copy_group_union=Some({"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:claimed_state_missing_unit"})」（`unseen_shapes.txt:297`）

（真判没有 I-1.13，`copy_group_union` 合成出 I-1.13。）十个 `PROTO_PLANT_CHOICE` 行全是 `file_unreadable=true`（第 3–12 行），臂挑了第一个。

### X1 中

七条臂共 29 个小段，`judged_is_full_domain=true` 29 行、`=false` 0 行（`grep -c` 数的）；整个段落判的态数等于被判小段的全量：「PROTO_PLANNED_STATES part=small_segments_exhaustive judged=89359 full_domain_of_the_judged_segments=89359 ceiling=1000000」（`small_segments_exhaustive.txt:85`）

`overwrite/unmodified/chunked` 四个小段 `red=0`、四种代表法不差：

- 「PROTO_FAMILY arm=overwrite/unmodified/chunked segment=12 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:104`）
- 「PROTO_FAMILY arm=overwrite/unmodified/chunked segment=13 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:127`）
- 「PROTO_FAMILY arm=overwrite/unmodified/chunked segment=17 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:150`）
- 「PROTO_FAMILY arm=overwrite/unmodified/chunked segment=18 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:173`）

（四种代表法行：第 105–108、128–131、151–154、174–177 行，全 0/0。）

### X2 中

C2 切出的第一个小段是段 17（12 写、6 组，含 X 一对）：

- 「PROTO_SEGMENT arm=overwrite/late_unit/chunked segment=17 writes=12 kinds=UnitWrite over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=true copy_groups=6 tearable=0 judged=true full_domain_states=4095 judged_states=4095 judged_is_full_domain=true」（`small_segments_exhaustive.txt:30`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=every_proper_subset representation=per_write_union compared=4094 red_or_not_differs=1023 red_items_differ=3071 unrepresentable=0」（`small_segments_exhaustive.txt:240`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=every_proper_subset representation=per_write_delta compared=4094 red_or_not_differs=1023 red_items_differ=1023 unrepresentable=0」（`small_segments_exhaustive.txt:241`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=every_proper_subset representation=copy_group_union compared=4094 red_or_not_differs=1023 red_items_differ=3071 unrepresentable=0」（`small_segments_exhaustive.txt:242`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=every_proper_subset representation=copy_group_delta compared=4094 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`small_segments_exhaustive.txt:243`）

C1 的两个小段与 C2 的其余两个小段全是 0：

- 「PROTO_FAMILY arm=overwrite/late_unit/chunked segment=12 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:196`）
- 「PROTO_FAMILY arm=overwrite/late_unit/chunked segment=13 family=all states=1022 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:219`）
- 「PROTO_FAMILY arm=overwrite/late_unit/chunked segment=18 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:276`）
- 「PROTO_FAMILY arm=overwrite/late_unit/chunked segment=19 family=all states=2 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:299`）

全量判下，`every_proper_subset` 4094 态里绿 1023 态（4094 − 3071），三种并/甲把这 1023 个绿态全判红；1023 = 2^10 − 1，即含 {x0,x1}、其余 10 个写任取、去掉全集（全集归下一小段的空集）（推的，与「补回只要 X 两份都落」相容；产物没逐态行）。

### X3 中

- 「PROTO_SEGMENT arm=overwrite/late_two_units/chunked segment=13 writes=8 kinds=UnitWrite over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=true copy_groups=4 tearable=0 judged=true full_domain_states=255 judged_states=255 judged_is_full_domain=true」（`small_segments_exhaustive.txt:38`）
- 「PROTO_SEGMENT arm=overwrite/late_two_units/chunked segment=17 writes=12 kinds=UnitWrite over_an_earlier_write=0 over_nonzero_base=0 readable_criterion_accepts=true copy_groups=6 tearable=0 judged=true full_domain_states=4095 judged_states=4095 judged_is_full_domain=true」（`small_segments_exhaustive.txt:42`）
- 「PROTO_COMPARE arm=overwrite/late_two_units/chunked segment=17 family=every_proper_subset representation=copy_group_delta compared=4094 red_or_not_differs=255 red_items_differ=2303 unrepresentable=0」（`small_segments_exhaustive.txt:369`）
- 「PROTO_FAMILY arm=overwrite/late_two_units/chunked segment=17 family=all states=4094 red=3839 whole_differs_from_empty=3071」（`small_segments_exhaustive.txt:370`）

预测外：四种代表法在这一小段的两种差逐格相同（第 366–369、371–374 行），与取样段的 R4 同。

### X4 中

`overwrite/record_before_units/chunked` 四个小段全 0：

- 「PROTO_FAMILY arm=overwrite/record_before_units/chunked segment=12 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:451`）
- 「PROTO_FAMILY arm=overwrite/record_before_units/chunked segment=13 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:474`）
- 「PROTO_FAMILY arm=overwrite/record_before_units/chunked segment=18 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:497`）
- 「PROTO_FAMILY arm=overwrite/record_before_units/chunked segment=19 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:520`）

### X5 中

`relocation_copies/chunked` 三个小段全绿、不差（同 R10）：

- 「PROTO_FAMILY arm=overwrite/relocation_copies/chunked segment=16 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:622`）
- 「PROTO_FAMILY arm=overwrite/relocation_copies/chunked segment=17 family=all states=4094 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:645`）
- 「PROTO_FAMILY arm=overwrite/relocation_copies/chunked segment=18 family=all states=2 red=0 whole_differs_from_empty=0」（`small_segments_exhaustive.txt:668`）

`relocation_copies_over_a_missing_source/chunked` 在搬迁副本所在的小段（段 17）与 R11 同：空集红含 I-2.1，两个单写各多 I-1.13，四种代表法不差：

- 「PROTO_BASELINE arm=overwrite/relocation_copies_over_a_missing_source/chunked segment=17 red_items={"pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.4"} whole=consulted=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 ignored=file_read@Some((InstanceGeneration(1), CheckpointTxg(4)))/fallbacks=0 checker=HHHHHHHHHVHHHHHHHHHVHHHHHHVHHHNHHHHHNHHHHHHHHHHHHHHHHHHHNHHHHNHHHHHHHHHNHNNNNNNNNHHHHNNNNHNNHHHHNHNHHHHNHHHHHNNHHH records=false/false」（`small_segments_exhaustive.txt:720`）
- 「PROTO_SINGLE_DIFFERS arm=overwrite/relocation_copies_over_a_missing_source/chunked segment=17 state=[0] write=UnitWrite@dev0+823803904x32768 red_items={"pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.4"} red_items_equal_empty=false」（`small_segments_exhaustive.txt:721`）
- 「PROTO_SINGLE_DIFFERS arm=overwrite/relocation_copies_over_a_missing_source/chunked segment=17 state=[1] write=UnitWrite@dev1+823803904x32768 red_items={"pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.4"} red_items_equal_empty=false」（`small_segments_exhaustive.txt:722`）
- 「PROTO_FAMILY arm=overwrite/relocation_copies_over_a_missing_source/chunked segment=17 family=all states=4094 red=4094 whole_differs_from_empty=3071」（`small_segments_exhaustive.txt:739`）
- 「PROTO_COMPARE arm=overwrite/relocation_copies_over_a_missing_source/chunked segment=17 family=all representation=per_write_union compared=4094 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`small_segments_exhaustive.txt:740`）
- 「PROTO_COMPARE arm=overwrite/relocation_copies_over_a_missing_source/chunked segment=17 family=all representation=copy_group_delta compared=4094 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`small_segments_exhaustive.txt:743`）

### X6 不中（预测的那一态不在小段的取值域里）

- 「PROTO_SEGMENT arm=overwrite/planted_two_groups/chunked segment=18 writes=4 kinds=UnitWrite over_an_earlier_write=4 over_nonzero_base=0 readable_criterion_accepts=false copy_groups=2 tearable=0 judged=true full_domain_states=15 judged_states=15 judged_is_full_domain=true」（`small_segments_exhaustive.txt:62`）
- 「PROTO_FAMILY arm=overwrite/planted_two_groups/chunked segment=18 family=all states=14 red=14 whole_differs_from_empty=14」（`small_segments_exhaustive.txt:593`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups/chunked segment=18 family=all representation=copy_group_union compared=14 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`small_segments_exhaustive.txt:596`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups/chunked segment=18 family=all representation=copy_group_delta compared=14 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`small_segments_exhaustive.txt:597`）
- 「PROTO_COMPARE arm=overwrite/planted_two_groups/chunked segment=18 family=all representation=per_write_union compared=14 red_or_not_differs=0 red_items_differ=6 unrepresentable=0」（`small_segments_exhaustive.txt:594`）

`copy_group_union` 红项差 0，推翻观测「0」出现。原因读得出来：这个小段 4 个写，`judged_states=15` 是空集加 14 个非空真子集，全集 {四份都落} 不在里面——按切法它是下一段的空集，而模型不判记录段，所以这一态在小段全量那一半里**哪一段都没判**。取样段里遮挡出现在 `both_planted_groups` 那一态（R14 引的第 297 行，其余 C2 单元没落），小段里能出现两组都整组落的态只有这个被排掉的全集。所以 X6 打的是预测本身的缺陷：预测写「15 态里」，而 15 态里不含它要的那一格（推的部分：遮挡只在两组都整组落时出现，这一句产物里只有 R14 那几个态可对）。按 R14 的「推翻之后怎么改」照字面应降「乙组间被遮」为整份层，但 R14 本身在取样段上中了，所以我不按 X6 改 R14 的判定，只记「小段全量没有量到这一格」。

照另一列看，`per_write_union`、`per_write_delta` 在这一小段红项差 6（第 589、590、594、595 行）：两份都被盖的那一组，单写各只报 4 项，两份都盖才报出读不出那一串（例：「PROTO_EXAMPLE arm=overwrite/planted_two_groups/chunked segment=18 representation=per_write_union state=[0,1] actual={"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:claimed_state_missing_unit"} represented={"pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.4"}」（`small_segments_exhaustive.txt:598`））。这是候选甲在副本对上的合取，第三节另记。

## 二、反例 A、B：候选甲、乙在两层上的中与不中

格里是 `family=all` 那一行的 `red_or_not_differs / red_items_differ`（「红不红」层 / 「红项」层，被比态里与全组合真判不一致的态数）；「中」= 这一层差不为 0，即这个候选在这条臂上判错了至少一态。甲 = `per_write_union`（并）、`per_write_delta`（增减）；乙 = `copy_group_union`（并）、`copy_group_delta`（增减）。代表法的定义照前一份报告「复跑命令与模型文件」一节那张表。

| 臂（段） | 判法与态数 | 真判红 | 甲·并 | 甲·增减 | 乙·并 | 乙·增减 | 引的行 |
|---|---|---|---|---|---|---|---|
| A 一对 `overwrite/late_unit`（段 16） | 取样 15954 | 11562 | 4392 / 8182 中 | 4392 / 4392 中 | 4392 / 8182 中 | 0 / 0 不中 | `counterexamples.txt:214-218` |
| A 一对 `sequential_write_two_data_units/late_unit`（段 16） | 取样 16405 | 12547 | 3858 / 8227 中 | 3858 / 3858 中 | 3858 / 8227 中 | 0 / 0 不中 | `user_actions.txt:233-237` |
| A 一对 `new_inode/late_unit`（段 16） | 取样 7269 | 5958 | 1311 / 3868 中 | 1311 / 1311 中 | 1311 / 3868 中 | 0 / 0 不中 | `user_actions.txt:527-531` |
| A 一对 `overwrite/late_unit/chunked`（小段 17） | 全量 4094 | 3071 | 1023 / 3071 中 | 1023 / 1023 中 | 1023 / 3071 中 | 0 / 0 不中 | `small_segments_exhaustive.txt:244-248` |
| A 两对 `overwrite/late_two_units`（段 16） | 取样 15027 | 13332 | 1695 / 3945 中 | 1695 / 3945 中 | 1695 / 3945 中 | 1695 / 3945 中 | `counterexamples.txt:317-321` |
| A 两对 `overwrite/late_two_units/chunked`（小段 17） | 全量 4094 | 3839 | 255 / 2303 中 | 255 / 2303 中 | 255 / 2303 中 | 255 / 2303 中 | `small_segments_exhaustive.txt:370-374` |
| B `overwrite/record_before_units`（段 17） | 取样 13439 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | `counterexamples.txt:421-425` |
| B `sequential_write_two_data_units/record_before_units`（段 17） | 取样 15065 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | `user_actions.txt:334-338` |
| B `new_inode/record_before_units`（段 17） | 取样 4177 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | `user_actions.txt:628-632` |
| B `overwrite/record_before_units/chunked`（C2 小段 18、19） | 全量 4094 + 4094 | 0 | 0 / 0 | 0 / 0 | 0 / 0 | 0 / 0 | `small_segments_exhaustive.txt:497-501`、`small_segments_exhaustive.txt:520-524` |

前面没整行抄过的几行：

- 「PROTO_FAMILY arm=overwrite/late_unit segment=16 family=all states=15954 red=11562 whole_differs_from_empty=8182」（`counterexamples.txt:214`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=all representation=per_write_union compared=15954 red_or_not_differs=4392 red_items_differ=8182 unrepresentable=0」（`counterexamples.txt:215`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=all representation=per_write_delta compared=15954 red_or_not_differs=4392 red_items_differ=4392 unrepresentable=0」（`counterexamples.txt:216`）
- 「PROTO_COMPARE arm=overwrite/late_unit segment=16 family=all representation=copy_group_union compared=15954 red_or_not_differs=4392 red_items_differ=8182 unrepresentable=0」（`counterexamples.txt:217`）
- 「PROTO_FAMILY arm=overwrite/late_two_units segment=16 family=all states=15027 red=13332 whole_differs_from_empty=7662」（`counterexamples.txt:317`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=all representation=per_write_union compared=15027 red_or_not_differs=1695 red_items_differ=3945 unrepresentable=0」（`counterexamples.txt:318`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=all representation=per_write_delta compared=15027 red_or_not_differs=1695 red_items_differ=3945 unrepresentable=0」（`counterexamples.txt:319`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=all representation=copy_group_union compared=15027 red_or_not_differs=1695 red_items_differ=3945 unrepresentable=0」（`counterexamples.txt:320`）
- 「PROTO_COMPARE arm=overwrite/late_two_units segment=16 family=all representation=copy_group_delta compared=15027 red_or_not_differs=1695 red_items_differ=3945 unrepresentable=0」（`counterexamples.txt:321`）
- 「PROTO_FAMILY arm=overwrite/late_unit/chunked segment=17 family=all states=4094 red=3071 whole_differs_from_empty=3071」（`small_segments_exhaustive.txt:244`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=all representation=per_write_union compared=4094 red_or_not_differs=1023 red_items_differ=3071 unrepresentable=0」（`small_segments_exhaustive.txt:245`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=all representation=per_write_delta compared=4094 red_or_not_differs=1023 red_items_differ=1023 unrepresentable=0」（`small_segments_exhaustive.txt:246`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=all representation=copy_group_union compared=4094 red_or_not_differs=1023 red_items_differ=3071 unrepresentable=0」（`small_segments_exhaustive.txt:247`）
- 「PROTO_COMPARE arm=overwrite/late_unit/chunked segment=17 family=all representation=copy_group_delta compared=4094 red_or_not_differs=0 red_items_differ=0 unrepresentable=0」（`small_segments_exhaustive.txt:248`）
- 「PROTO_COMPARE arm=overwrite/late_two_units/chunked segment=17 family=all representation=per_write_union compared=4094 red_or_not_differs=255 red_items_differ=2303 unrepresentable=0」（`small_segments_exhaustive.txt:371`）
- 「PROTO_COMPARE arm=overwrite/late_two_units/chunked segment=17 family=all representation=per_write_delta compared=4094 red_or_not_differs=255 red_items_differ=2303 unrepresentable=0」（`small_segments_exhaustive.txt:372`）
- 「PROTO_COMPARE arm=overwrite/late_two_units/chunked segment=17 family=all representation=copy_group_union compared=4094 red_or_not_differs=255 red_items_differ=2303 unrepresentable=0」（`small_segments_exhaustive.txt:373`）
- 「PROTO_COMPARE arm=overwrite/late_two_units/chunked segment=17 family=all representation=copy_group_delta compared=4094 red_or_not_differs=255 red_items_differ=2303 unrepresentable=0」（`small_segments_exhaustive.txt:374`）

读这张表的几句（算术是我做的，量是产物里的）：

1. **错的方向全是「错红」**。反例 A 的六行里，每种代表法的 `red_or_not_differs` 都恰等于「态数 − 真判红」（15954 − 11562 = 4392，16405 − 12547 = 3858，7269 − 5958 = 1311，4094 − 3071 = 1023，15027 − 13332 = 1695，4094 − 3839 = 255）。两种并合成含 red(∅)，而这几段空集态都红，所以它们恒红，差数等于绿态数是定义推出来的；两种增减合成的差数也恰好等于绿态数，推的：同样是把绿态判红、没有一态漏红。所以在这几条臂上，候选的错不会让一条本该红的用例变绿；变的是逐态的红绿串（前一份报告 ②之一 说的「入库的判定块与对拍比的那一串」）。
2. **合成规则分辨一对、不分辨两对**。一对那一格只有乙·增减不中，前一份报告 ②之一 的演绎（并合成恒红、只有增减能减掉 red(∅) 里的项）在三种 C2 与小段全量上都坐实了。两对那一格四种同数，乙·增减不比别的好。
3. **反例 B 一格没有被打的对象**：三种 C2 与小段全量上 `red=0`，任何候选在这一臂上都判不错，这一列量不出甲乙的高低。它不支撑「反例 B 不存在」：只抽了一次，臂只造了「记录段整段挪到单元段前」这一种形状。
4. **取样数**：每格的态数就是表里「判法与态数」那一列；取样段判的是被判段全量的极小一部分（第四节）。

前一份报告第五节改法表里能对上的格，量的结果（主 agent 任务 8699，副本）：甲·并 A 一对中、A 两对中；甲·增减 A 一对中、A 两对中；乙·并 A 一对中（第一轮报「不中」那一句按并合成不成立）、A 两对中；乙·增减 A 一对不中、A 两对中。反例 B 两列（「施加的那条记录点名一组」「点名多组」）这一轮都没造出，量不到。

## 三、漏掉的破坏条件 L1–L4 与 4 类敏感取样点各量出了什么

### L1 前缀引用本段要写的槽（「补回」）

量到。三种 C2 的 `late_unit` 与覆盖写的 `late_two_units`，空集态都红（读不出、I-1.13、I-2.1、I-4.8、I-7.2、I-7.4），目标态（挪走的副本全落）绿；逐写判的错见第二节表。L1 坐实，且只在 C2 单元段上出现：同一条臂的 C1 单元段（段 12）四种代表法全不差（例：「PROTO_FAMILY arm=overwrite/late_unit segment=12 family=all states=7271 red=0 whole_differs_from_empty=0」（`counterexamples.txt:164`））。

### L2 记录先于点名单元

没造出（R6、R7、R8、X4）。三种 C2 上空集态绿、`red=0`、「只缺一整组」的目标态全绿且与空集整份相同。顺序写那一种有两条记录（第 22 行），也没红。

### L3 诞生代号不挡的在飞单元（搬迁副本）

- 前缀完整（`relocation_copies`）：取样 15921 态全绿、四种不差（R10）。「在飞单元不进判定」在这一形上没有被量到的反例。
- 前缀缺一份源副本（`relocation_copies_over_a_missing_source`）：空集态红（I-2.1、I-4.8、I-7.4），每个搬迁副本单写都多出 I-1.13（R11 第 395、396 行），所以**在飞的搬迁副本确实进了判定、单写就改了红项**；15900 态全红，四种代表法两层都不差，这一形上多写之间是「或」，单写合成得出。小段全量（段 17，4094 态）同（X5）。
- 限度照前一份报告：臂是字节层面的搬迁，没走 `publish_relocation_batch`。

### L4 系统配置轮换与单元写同段

取样 48703 态全绿、四种不差（R12）。b8 的 3^m·(n+1) 那一格在这条流上没咬着。

### 4 类敏感取样点

| 类 | 臂 | 量出了什么 |
|---|---|---|
| 混合种类段（b8） | `rotation_with_units` | 同 L4：全绿、不差 |
| 混合种类段（b6） | `units_records_root` | 真判红 5601 / 14634；红不红层四种各差 2 态（`all_but_one` 族）；**红项层四种各差 3791 态**，预测外 |
| 多组同时被盖、有引用关系 | `planted_two_groups` | 红不红层四种全不差；红项层甲 5219、乙 1685（`family=all`）；甲是少报、乙是多报 |
| 小段全量对照 | 七条只有单元写的臂 | X1–X5 中、X6 不中（第一节） |
| 整理/搬迁（半类） | `relocation_copies*` | 同 L3 |

`units_records_root` 那 3791 态的红项差是什么，示例行给了：真判多出 `pool_checker:I-9.1`，四种代表法都没有（写 22 与根 26 一起落才出现，单写各自都不报它；写 22 按并段次序是单元写，推的）：「PROTO_EXAMPLE arm=overwrite/units_records_root segment=16 representation=per_write_union state=[22,26] actual={"journal_consulted_oracle:recovered_to_an_older_root", "journal_ignored_oracle:read_failed", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "pool_checker:I-9.1", "record_checker:root_without_record"} represented={"journal_consulted_oracle:recovered_to_an_older_root", "journal_ignored_oracle:read_failed", "pool_checker:I-2.1", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:root_without_record"}」（`unseen_shapes.txt:221`）

这是一次「组合才出现的红项」，四种合成都合不出来（推的：I-9.1 是单元与根之间的合取）。这一段 `readable_criterion_accepts=false`（第 26 行），按前一份报告 R13 的说法 D1 本来就把它挡在逐写判外，所以它不改「判据收下的段」上的结论，改的是「混合段能不能逐写判」：红项层不能。

`planted_two_groups` 的两种错，只盖子组那一态：「PROTO_TARGET arm=overwrite/planted_two_groups segment=16 target=child_group_only state=[24,25] actual={"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:claimed_state_missing_unit"} whole_equals_empty=false per_write_union=Some({"pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.4"}) copy_group_union=Some({"journal_consulted_oracle:read_failed", "journal_ignored_oracle:read_failed", "pool_checker:I-1.13", "pool_checker:I-2.1", "pool_checker:I-3.15", "pool_checker:I-4.8", "pool_checker:I-7.2", "pool_checker:I-7.4", "record_checker:claimed_state_missing_unit"})」（`unseen_shapes.txt:298`）

候选甲在两份都被盖的那一组上少报读不出那一串（单写只报 4 项，两份都盖才读不出）；候选乙在「两组都整组盖」那一态多报 I-1.13（第 297 行，祖先组遮了后代组）。这一段也是判据不收的（`over_an_earlier_write=4`，第 29 行）。

### 判据收下的段上，差都出在空集态红的段上

我从四个产物里把每个被判段的三样抽出来（`readable_criterion_accepts`、空集态红项空不空、`family=all` 四行里有没有不为 0 的），表在草稿目录 `accepted-segments.txt`（57 行，抽法 `accepted_segments.py`，都在草稿目录）。数出来：

| 收下的段 | 空集态红 | 四种代表法有差 | 段数 |
|---|---|---|---|
| 是 | 是 | 是 | 6 |
| 是 | 否 | 是 | 0 |
| 是 | 是 | 否 | 4 |

有差的 6 段：三种 C2 的 `late_unit` 段 16、`late_two_units` 段 16、两条 late 臂的小段 17。空集态红而不差的 4 段：`relocation_copies_over_a_missing_source` 的段 16 与它的小段 17、18、19。所以在这份产物里，判据收下的段上「空集态红」是四种代表法出错的必要条件、不是充分条件。这一条只在这几条臂的取样上看过，被攻过零轮；它与前一份报告第五节「J(∅) 红就退回全组合」那一条改法相容（退回在这 6 段上都会触发，另在 4 段上白白触发），那条改法违反第一轮正文第四节第 3 条「读段表就能判」的问题照旧。

## 四、方法上的限定

### 哪些结论是取样判出来的

- `counterexamples`、`user_actions`、`unseen_shapes` 三段的每个被判段都是 `judged_is_full_domain=false`。判的态占被判段全量：「PROTO_PLANNED_STATES part=counterexamples judged=96190 full_domain_of_the_judged_segments=407896056 ceiling=1000000」（`counterexamples.txt:39`）（约 0.024%）；「PROTO_PLANNED_STATES part=user_actions judged=130413 full_domain_of_the_judged_segments=1692401652 ceiling=1000000」（`user_actions.txt:58`）（约 0.008%）；「PROTO_PLANNED_STATES part=unseen_shapes judged=148476 full_domain_of_the_judged_segments=729808888 ceiling=1000000」（`unseen_shapes.txt:47`）（约 0.020%）。四段合计判了 464438 态（96190 + 130413 + 148476 + 89359）。
- 族内全不全：`support`、`single`、`pair`、`all_but_one` 在各自那一类里是全的；`group_lattice` 在组数 ≤ 13 时全取（态数 = 2^G − 2：12 组 4094、13 组 8190、11 组 2046、10 组 1022，去掉空集与全集），14 组、15 组时随机取（6402–6450、7260，与「取 8192 次、可重复」的去重期望相合，推的）；`device_lattice` 在每盘 ≤ 12 写时全取（2 × 4095 = 8190 等），超过时随机；`random` 每段 1000。
- 所以 R3 的 4095、R9 建 inode 的 1023 是「组格这一族里全的」，不是「这段全量里全的」：组格只含「若干整组两份全落、其余组一份不落」的态。「一组只落一份、另几组整组落」「几组各落一份」这类混合态只有 `random` 1000 个与 `device_lattice` 在碰。反例 A 的差在这些混合态上也出现（`random` 族：`counterexamples.txt:205-208` 里三种 273、`copy_group_delta` 0），但量只有 1000 个。
- 第二节表里「乙·增减在一对上不中」是取样 15954 + 16405 + 7269 态加小段全量 4094 态上的 0，不是 2^26、2^30、2^22 全量上的 0。
- 每段全集都不判（`family=all` 的态数 = `judged_states` − 1，空集另算基线）：全集是下一段的空集。取样段里这无妨（下一段的空集在下一段判）；但模型不判记录段与根段，所以每条臂最后一个被判单元段的全集在这份产物里哪儿都没判。

### 小段全量那一段落和原流差在哪

- 小段是「在单元写之间按副本组多发几道屏障」的历史：每个小段的前缀是更早的小段全落。原流上跨两个小段各落一部分的态（例：X 一对只落一份、同时 C2 后面某组整组落）在小段里不存在。`late_unit` 的 C2 单元段原流 26 写、全量 67108863 态（`counterexamples.txt:17`），切成 12、12、2 写三个小段后合计 4095 + 4095 + 3 态。
- 被挪的那一对排在 C2 第一个小段里（`small_segments_exhaustive.txt:30`），所以「补回」只在这一个小段上出现，后两个小段的前缀已经含 X，空集态绿（`small_segments_exhaustive.txt:259`、`small_segments_exhaustive.txt:282`）。
- 最后一个小段的全集（C2 单元全落、记录没落）归记录段的空集，不在任何被判段里——X6 打不着就是因为这个。
- 所以小段全量能说的是「在这几个改了屏障的实现上，四种代表法在全部崩溃状态上的差」，不能直接说原流；它与取样段同方向（X2 对 R3、X3 对 R4、X5 对 R11）是相容，不是原流全量的证明。

### 结论能外推到哪

- **能**：在这条流（取号 → 暖机 → 新池新建文件 A → 覆盖写 C1 3000 字节 → C2 三种之一）上、这几条改写表造的坏实现上，候选甲两种合成、候选乙并合成在「补回」那一格判错（错红），乙增减在一对上不错、两对上错。
- **不能**：C1 换别的动作或别的大小、前缀更长的流、三对以上被挪、`late_*` 之外的写晚形状，这一轮都没扫（C1 固定、`unseen_shapes` 五条臂只在覆盖写上跑）；今天对的实现走不出这些写表（前一份报告「这条腿自己的限度」）；状态集合是模型自己按定义取的，没经 `enumerate_crash_states`，撕裂只在 `rotation_with_units` 的 2 个原地覆写上有（`unseen_shapes.txt:13` 的 `torn_images=2`）。
- 这是一次复跑、一份样本。打中的格（反例 A）照「一次就值得去核」交主 agent 核；没打中的（反例 B、L2、L4、R10 的全绿）照 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测——否定结论尤其不算」不支撑「没有」。我这条腿读的是同一次复跑，不是第二次抽样。

### 打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」）

| 打中的格 | 分不分辨臂 | 系统看不看得到判别它的东西 | 满足的是哪个分句 | 跑前给的改法在这几格上还中不中 |
|---|---|---|---|---|
| 反例 A 一对 | 分辨合成规则：只有乙·增减不中 | 看得到：x0、x1 是同一组，乙·增减判的组内子态里有 {x0,x1} | 候选定义没写合成分句（前一份报告 ②之一）；不是判据 D1–D5 的哪一条（那几条我没判） | 「J(∅) 红就退回」：这几段空集态都红，会退回全组合，推的不中；「引擎先算 R(P)」：推的，没实现 |
| 反例 A 两对 | 不分辨：四种同数，病根在四种共用的「只判空集、单写与组内子态」 | 看不到：判别要的「X、Y 两组都整组落」不是任何一种代表法判的态，属「判别子观测不到」 | 同上，另加「组间独立」这一共用前提不成立 | 同上；乙·增减对这一格不起作用 |
| `planted_two_groups` 红项层 | 分辨：甲少报、乙多报，数不同（5219 对 1685） | 甲看不到「同组两份都盖」；乙看不到「两组都整组盖」 | 段不被判据收下（`over_an_earlier_write=4`），不是判据收下的段上的错 | 「J(∅) 红就退回」不触发（空集态绿，`unseen_shapes.txt:234`），对这一格不起作用 |
| `units_records_root` 红项层 | 不分辨：四种同数 3791 | 看不到：I-9.1 只在写 22 与根同落时出现 | 段不被判据收下（混合种类） | 同上，空集态绿（`unseen_shapes.txt:178`），不起作用 |

## 没打中的形状

都是同一次复跑、一份样本，不支撑「没有」：

- 反例 B「记录段整段挪到单元段之前」：覆盖写、顺序写两个数据单元、建一个 inode 三种 C2 取样（13439、15065、4177 态）与覆盖写的小段全量（C2 两个小段各 4094 态）全绿。顺序写那种两条记录也没红（R7 不中）。没试的形状：只挪一条记录、记录与单元交错、根也提前、记录点名的单元跨两次发布。
- 系统配置轮换与单元写同段（L4、R12）：48703 态全绿。
- 搬迁副本落在空槽、前缀完整（R10、X5 前一半）：取样 15921 态、小段全量 4095 + 4095 + 3 态全绿。
- 三种 C2 的 `unmodified`：每段全绿、四种代表法不差（R1、X1）。

## 这条腿自己的限度

- 只读产物，没读模型源码核「族怎么取、代表法怎么合成」；族与代表法的含义照前一份报告第一节与第二节的文字，没对着 `proto/src/main.rs` 核（推的部分已标）。
- 差的方向（全是错红）是由计数算出来的，产物没有逐态的「真判红、代表绿」计数列；两种增减合成那一半是推的。
- R13 的机理、R4 的「等于含两对四份的态数」、X2 的 1023 是哪些态，产物里没有逐态行，只做了相容检查。
- 没跑第二次抽样；`random` 族的种子在不在源码里固定，我没核，复跑同一命令是不是逐字节相同不知道（前一份报告「要主 agent 跑的命令」第 1 条写的 `same=` 比较，要先把四个输出放进模型目录的 `out/` 才比得了）。
- 没读辩方腿报告（禁读），也没读辩方腿的模型目录。

## 要主 agent 跑的命令

这一条腿的结论不需要再跑什么。两件交主 agent 定、不是要经服务交的命令：

1. 四个产物现在只在仓外 `/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/rerun-work/out/`。判决要引它们的话，照共用约束「报告」一节落进 `research/results/` 或模型目录的 `out/`（前一份报告让承接的腿拷进模型目录 `out/`，那个目录不在我这条腿的写范围里，我没拷）。sha256 见报告开头那张表。
2. 想核「判据收下的段上差都出在空集态红的段上」那张表：`python3 -I /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack-read/accepted_segments.py counterexamples.txt user_actions.txt unseen_shapes.txt small_segments_exhaustive.txt`，在产物目录里起；它只读文本、不跑项目代码，输出与草稿目录 `accepted-segments.txt` 比（57 行；`grep -c 'accepts=true empty_red=True nonzero=True'` 应为 6，`accepts=true empty_red=False nonzero=True` 应为 0）。

## 没做什么

- 没往统一任务服务交任务；没写模型、没改前一份报告的模型与预测；判决要引的数是主 agent 任务 8699 的复跑产物，在副本装置上，没在入库装置上重做。
- 不判正推、辩方那几格；不替主 agent 采纳。第三节末那条「空集态红是必要条件」与它牵到的改法，只在这份产物上看过，被攻过零轮。
- 没拷产物进仓（上一节第 1 条）。

## 草稿目录里留的

`/home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack-read/`：`render.py`（把报告里的引文占位换成产物整行）、`accepted_segments.py` 与它的输出 `accepted-segments.txt`、三份过滤后的产物摘录 `*.filtered.txt`、`task-8699.log`、报告分段草稿 `part*.md` / `part*.out`、认领留的 `.owner`、`.writer-lock`。都是普通文本，没有仓副本、编译目录或库拷贝。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/per-write-judging-r2-opus-s2-output.md`（只给仓根）末行：

```
  ✓ 核了 154 处引文、35 处裸行号，对上 0 处，没判 189 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```

产物在仓外，只给仓根时全部「路径认不出」。加上产物目录与草稿目录作根（`--root /home/fy5090/code/singlefs --root /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack/rerun-work/out --root /home/fy5090/.local/share/singlefs/work/per-write-judging-r2-attack-read`）末行：

```
  ✓ 核了 154 处引文、35 处裸行号，对上 154 处，没判 35 处（其中裸行号没有引文可比 35 处；逐处列在上面）
```
