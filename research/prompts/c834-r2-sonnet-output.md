# C834 第二轮 辩方腿报告（Sonnet，2026-10-15）

立场：辩方。复核第一轮判决 `research/prompts/c834-r1-main-verification.md`：出局那几个候选的历史打不打得中留下的 K6 与指针槽读法；判决对这两个候选的定义有没有读错；R1、R2 两格在冻结副本拷贝上量。
冻结副本沿用 `/home/fy5090/.local/share/singlefs/work/c834-r1/repo/`，拷进草稿目录后 `sha256sum -c research/prompts/c834-r2-snapshot/crates-sha256.txt` 95 份全对上（不对的 0 份）。
模型与驱动在 `research/prompts/c834-r2-sonnet-model/`（说明见同目录 README.md）。行号：代码引冻结副本或模型目录里那份文件自己的行号，kb 引 kb 文件自己的行号，产物引 `out/` 里那份文件自己的行号；都是这一次 `grep -nF` 现取。量过的有命令输出，标「推的」的没量过。
全部数是拷贝上的数，不是入库装置上的数（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。
禁读清单里的文件没读。

## 0. 结论一览

| 格 | 判定 | 一句话 |
|---|---|---|
| R1 K6 | 判决够得着（没打中的另一半除外） | K6 实现成「K3 的凭据，或映射根槽凭据」，K3 被丙打中的那一句整个带进 K6：丙 3 个状态两种盘序 K6 都放过并写掉 R 的 14 份单元。K6 在甲、乙、丁、戊与正推的九段上全对 |
| R1 K6 的映射根那一半（新名 `mapping_root_only`，零轮） | 没打中 | 甲乙丙丁戊与九段（只有一段除外，下面写）全对；代价是拒掉两类「放过也不丢单元」的历史 |
| R2 指针槽读法 | 判决误读 | 丁上全拒不是读法本身分不开，是攻方原型读错了长度：索引节点只能按 16 KiB 解，原型一律按 32 KiB 读再解，索引节点那一支永远解不出 |
| R2 指针槽读法（改读长度，新名 `pointer_slots_fixed`，零轮） | 没打中 | 攻方五段全对（丁 4 个状态全放过）；入库 25 条绿；正推九段错 1 段（拒掉「放过也不丢单元」的那一段） |
| R4 出局候选的历史 | 部分够得着 | 读系统配置号的那批（今天、跨盘取最大、逐盘取最小、照 I-7.7 第 ② 句、各盘各槽取最小）的出局历史（甲、乙、戊、取号两盘落完、回卷）都不读号的两个候选没被打中；K3 的出局历史两条里，二重故障那条够不着 K6，丙那条同样打中 K6 |
| R4 「判别子观测不到」那一句 | 站得住，但管不到指针槽 | 对只读号与记录的候选站得住（`later_root_or_record` 在丙、丁上重现）；读指针槽的候选在丙与丁上看到的不同 |

## 1. 对判决的复核：定义有没有读错

判决留下两个候选的写法（`research/prompts/c834-r1-main-verification.md:20`）：「留下 K6 与指针槽读法两个零轮候选，都没被对方的历史攻过：K6 没在丙、丁、戊上跑，指针槽读法没在正推的九段上跑，且在丁上拒（可用性代价，不违反不变量）」。

两个候选的定义原文：
- K6，第一轮正推报告 `research/prompts/c834-r1-sonnet-output.md:16`：「K3 的凭据，再加「那条根点名的中央映射树根那个槽里现在是别的、自证过的、实例代号比它新的索引节点」」。实现是补丁 `research/prompts/c834-r1-sonnet-model/patch/mount-c834-variants.patch:104` 的「"k6" => {」那一支：根环里有更大号的根、或记录里有更大号的、或映射根槽里是更新的索引节点，三样有一样就放过。「再加」读成「多一样凭据，有一样就放」才与第一轮正推的九段表对得上（`units_only` 一段没有更大号的根或记录，K6 放过，靠的只有映射根那一样），所以 K6 是 K3 的超集。判决没有误读这一点，但判决没有写出「超集」带来的后果（下面 R1）。
- 指针槽读法，第一轮攻方报告 `research/prompts/c834-r1-opus-output.md:50`：「G2 攻方自提「R 的两个指针槽里读出更大号的单元头」」。攻方对丁全拒的原因写「原因没查到底：推测 M1 写到 R 的树表 / 映射根那一槽的是一个 32 KiB 单元的后半截，槽首没有单元头可解（推的，没核）」（`research/prompts/c834-r1-opus-output.md:138`）。判决把「丁上全拒」当成这个读法的性质（「且在丁上拒（可用性代价，不违反不变量）」），没有要求查到底。查到底的结果在 R2。

另一条定义上的出入：判决里写 K6 是「被攻过零轮」，但 K6 的 K3 半边（根或记录）就是判决自己用丙打出局的 K3，不是零轮的东西，只有「再加」的映射根那一半没被攻过。

## 2. R1：K6 在攻方五段与正推九段上

候选实现见 `research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py:176` 「let k6 = later_root_or_record || mapping_root_slot_holds_a_newer_index_node;」，映射根那一样读 16 KiB 再 `parse_index_node`（同文件 `:166` 「let mapping_root_slot_holds_a_newer_index_node = root.mapping_root.locations.iter().any(|location| {」）。

### 2.1 正推九段（第一轮的测试，改成读 `C834_CANDIDATE`），每格 = 盘序 [盘 0, 盘 1] / [盘 1, 盘 0]，两序相同只写一个；「做成[N]」= 做成且这次挂载新写掉 R 的 N 份单元

命令（每个候选一遍，经内存包装与 31 线程）：`C834_CANDIDATE=<候选> bash scripts/hosts/run-with-memory-cap.sh 24G bash scripts/hosts/capped.sh 31 cargo test -p singlefs-harness --test c834_order_of_devices_decides_the_newer_root_judgement -- --nocapture --test-threads=1`，汇总 `research/prompts/c834-r2-sonnet-model/tools/nine-matrix.py`，原样输出（`out/nine-matrix.txt`）：

```
历史 \ 候选 : today | later_root_or_record | k6 | mapping_root_only | pointer_slots_overwritten | pointer_slots_fixed | always_refuse
baseline : 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒
partial : 做成[14]/拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒
complete : 做成[14] | 拒 | 拒 | 拒 | 拒 | 拒 | 拒
units_only : 做成[0] | 拒 | 做成[0] | 做成[0] | 拒 | 做成[0] | 拒
urj : 做成[0] | 做成[0] | 做成[0] | 做成[0] | 拒 | 做成[0] | 拒
urj_row_record_also_hidden : 做成[0] | 拒 | 做成[0] | 做成[0] | 拒 | 做成[0] | 拒
later_mount_records_landed_r_units_intact : 做成[0] | 做成[0] | 做成[0] | 拒 | 拒 | 拒 | 拒
acquisition_failed_on_device_1_rolled_back : 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒
acquisition_barrier_failed_rolled_back_on_both : 拒 | 拒 | 拒 | 拒 | 拒 | 拒 | 拒
```

- 与第一轮正推的矩阵对：`today` 即那里的 k1，`later_root_or_record` 即 k3，`k6` 即 k6，三列的九行结局与 `research/prompts/c834-r1-sonnet-output.md` 第 2 节矩阵里同名列逐格相同（我的补丁是另写的，不是拷第一轮的补丁，所以这是对拍）。
- K6 九段全对，两种盘序结局相同。第一轮正推的 K6 结论在这里复现。
- `pointer_slots_overwritten`（攻方原型原样）在九段里 `units_only`、`urj`、`urj_row_record_also_hidden`、`later_mount…` 四段拒，其中 `urj` 即入库 urJ 那一形：与攻方「入库 urJ 用例红」一致。
- 九段里的 `later_mount_records_landed_r_units_intact`：`mapping_root_only` 与 `pointer_slots_fixed` 拒。第一轮正推自己在这一段的口径是「拒或放过都不丢 R 的单元」（`research/prompts/c834-r1-sonnet-output.md:73`，该行表格里写的是「拒或放过都不丢 R 的单元（下面「安全」那条用例钉；表按今天的结局钉「放过」）」），所以这一格是可用性代价，不是写掉单元。入库两份用例（25 条）在这两个候选下全绿，见 2.3。

### 2.2 攻方五段（崩溃状态全量，两种盘序）

原型是第一轮攻方的 `research/prompts/c834-r1-opus-model/`，只改了判决行里多带的字段，判决表照搬。控制：`later_root_or_record` 的 58 个格我这一遍与攻方入库的 `aggregate-all.txt` 去掉 `example_ordinal` 后逐行相同：

```
all control(later_root_or_record) identical, 58 cells
```

K6 的结局（`research/prompts/c834-r2-sonnet-model/out/aggregate-all.txt`、`aggregate-rollback.txt`；「写掉」= M2 写行根落盘之前新写掉 R 的 14 份单元、池级 checker 红 I-2.1、I-4.8、I-7.2、I-7.4）：

| 段 | K6 | 凭据 |
|---|---|---|
| 甲（取号只落一块盘） | 拒（对），两序 | `aggregate-all.txt` 里 `class=number_on_one_device_only` 的 judge=refuse 各 4 个状态；回卷表里同类 4 个 |
| 乙（取号两盘落完、没写） | 拒（对），两序 | `class=number_taken_no_record_r_units_intact` judge=refuse，clean/y2 16384 个、tree_table_and_record/y2 16387 个状态 |
| 丙（M1 隔离 R 的单元、写了带号的记录、写行根没落） | **放过并写掉（错），两序** | `aggregate-all.txt:137`（forward）、`:140`（reversed）：3 个状态，`judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4` |
| 丁（入库 urJ 那一形，M1 看不见 R、写掉 R 的单元） | 放过（对），两序 | `aggregate-all.txt:62`、`:70`：4 个状态，R 的单元在 M2 之前已被写掉 |
| 戊（取号回卷） | 拒（对），两序 | `aggregate-rollback.txt:6`、`:16`、`:28` 及 reversed 的 `:7`、`:17`、`:29`：4+4+9=17 个状态全拒 |

丙上的逐状态原样在 `research/prompts/c834-r2-sonnet-model/out/bing-states.txt`（草稿目录 `runs/` 里的原始输出按「候选 × M2 读故障 = tree_and_mapping × 判走到了」挑出来的 16 行，没改字）：序号 16396、16398、16399 各两序（该文件第 3 至 8 行），K6 的判行 `mapping_newer_index_node=false,k6=true`，M2 结局 `r_units_overwritten_before_row_root=14`；序号 16395 没有更大号的记录，K6 拒（第 1、2 行）。映射根槽（`map.d0.slot1541`）在 M2 的读故障下读不出（`here=none`）。

**K6 在丙上打不过的原因**：丙里带号 2 的记录在（`later_root_or_record=true`），而映射根槽读不出更新的节点，K6 的 K3 半边直接放过。这与判决里 K3 被丙打出局的那一句是同一个历史、同一个原因：
- 判决里的话：「K3（两腿各自打中：正推量到二重故障两段误拒，攻方量到丙误放）」（`research/prompts/c834-r1-main-verification.md:20`）。K3 在丙上误放这一条，K6 一字不差地继承；二重故障两段误拒这一条，K6 靠映射根那一半补回了（九段里 `units_only`、`urj_row_record_also_hidden` K6 做成），不继承。
- 所以判决里「留下 K6」那一步不成立：K6 不是零轮候选里没被攻过的，它被判决自己用来打 K3 的丙打中了。判定：**判决够得着 K6（丙那一半）**。

### 2.3 去掉 K3 半边的 K6（`mapping_root_only`，零轮，我自提）

```
nice -n 19 bash scripts/hosts/run-with-memory-cap.sh 24G bash scripts/hosts/capped.sh 31 bash research/prompts/c834-r2-sonnet-model/tools/run-batch-faults.sh …
```
头尾两段（序号 0–20 与 16000 起，覆盖甲、乙的头、丙、丁、不覆盖乙的中间约 16000 个状态，那一类里 K6 与 pointer_slots_fixed 全量已是全拒）加回卷 17 个状态加九段：
- 丙：`out/aggregate-mapping-tail.txt:43`（forward）、`:46`（reversed）：3 个状态 judge=refuse outcome=refused，对。
- 丁：`out/aggregate-mapping-tail.txt:24`、`:30`：4 个状态放过，对。
- 甲、乙、戊：拒，对（回卷 17 个状态全拒，`out/aggregate-rollback.txt` 里 `candidate=mapping_root_only`）。
- 代价：clean 前缀里 M1 建在 R 上、R 的单元一份不少的 3 个状态，放过本来不丢单元（K3 放过、`mounted_r_units_kept`），`mapping_root_only` 拒：`out/aggregate-mapping-tail.txt:12`、`:14`（各 3 个状态）；加九段里的 `later_mount…` 一段。这两类都是「放过也不丢单元」的态，拒了之后每次可写挂载都拒，盘不变。

## 3. R2：指针槽读法

### 3.1 丁上全拒的原因：攻方原型读错了长度

攻方原型的读法（第一轮模型，原样）：
- `research/prompts/c834-r1-opus-model/patch/apply_candidates.py:80`：「usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"),」，先读 32 KiB；
- 同文件 `:88`：「if let Ok(header) = crate::unit::parse_index_node(&bytes) {」，把这 32 KiB 交给 `parse_index_node`。
冻结副本里 `parse_index_node` 开头就按长度拒：`crates/singlefs-core/src/unit.rs:391`「if bytes.len() != usize::try_from(NODE_BYTES).expect("16384") {」（函数头在 `:390`，`NODE_BYTES` 是 16384）。所以交 32768 字节给它永远返回 `TooShort`，索引节点那一支从来解不出；树表与映射根指向的恰好都是索引节点。能解出的只剩打包记录单元与数据单元的头，而 M1 写在这两个指针槽上的是索引节点。

量过（我的补丁在判决行里记了每个指针槽的内容，`diag=`）。九段里的 `urj` 历史，M2 看到的树表槽与映射根槽（`research/prompts/c834-r2-sonnet-model/out/nine-pointer_slots_fixed.log:27`，其中映射根这一段）：

```
map.d0.slot1541:head=unreadable:here=index_node@2:from_previous_slot=none
```

`here=index_node@2` = 槽首按 16 KiB 读能解出实例代号 2 的索引节点；`from_previous_slot=none` = 从前一槽起读 32 KiB 解不出打包记录单元或数据单元，攻方的推测（「32 KiB 单元的后半截」）不成立。（`head=unreadable` 是我的诊断读 8 个字节没对齐被拒，与判无关。）

丁的 4 个状态（序号 16395、16396、16398、16399，judge 走到了的）、两种盘序，各候选的结局（`out/ding-summary.txt`，同一次用 `ding-summary.py` 汇总，原样）：

```
== pointer_slots_overwritten
16395 forward refuse refused later_records=0
== pointer_slots_fixed
16395 forward pass mounted(overwrote=14) later_records=0
== k6
16395 forward pass mounted(overwrote=14) later_records=0
== mapping_root_only
16395 forward pass mounted(overwrote=14) later_records=0
```

（其余序号与盘序同形；`overwrote=14` 是 M2 之前这个状态本身已被 M1 写掉 R 的单元，放过是对的，状态本身就红，M2 没让它更坏。完整 45 行见 `out/ding-summary.txt`。）

判：**判决对指针槽读法的出局（「且在丁上拒」当成读法的可用性代价）是误读**：丁上全拒是原型实现缺陷；读法本身（R 根记录里两个指针所在的槽里读出更大号的单元头）改成按单元类别取读盘长度（`pointer_slots_fixed`，`research/prompts/c834-r2-sonnet-model/patch/apply_candidates.py:190`）后，丁 4 个状态全放过、入库 urJ 用例不红。
推翻这一条的现象：在入库装置上（不是拷贝）按攻方原样读法重造丁，`urJ` 用例仍红且槽首能被 `parse_index_node` 在 16 KiB 长度下解出却仍被拒——那就不是长度的问题。

### 3.2 改过长度的读法（`pointer_slots_fixed`，零轮）在九段与五段上

- 九段（2.1 的矩阵）：8 段对，`later_mount_records_landed_r_units_intact` 拒（可用性代价，不丢单元）。
- 五段全量（`out/aggregate-all.txt`、`aggregate-rollback.txt`，覆盖与 K6 同，四个 M2 读故障前缀组合各 20511 或 20516 个状态，12 个（候选 × 前缀）覆盖行 complete=True）：甲、乙拒（对）；丙拒（对）：`out/aggregate-all.txt:149`、`:152`，3 个状态 judge=refuse outcome=refused；丁 4 个状态放过（对）；戊 17 个状态拒（对，`aggregate-rollback.txt`）。
- 入库两份用例（`writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs` 16 条与 `recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs` 9 条）逐候选，`out/existing-tests-by-candidate.txt`：

```
candidate=pointer_slots_fixed
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
candidate=pointer_slots_overwritten
test a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts ... FAILED
test result: FAILED. 15 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

后一组即攻方原型的「入库 urJ 用例红」，在我的拷贝上重现。`k6` 与 `mapping_root_only` 与 `pointer_slots_fixed` 一样 16 条与 9 条全绿（同一个文件）。

### 3.3 读法本身在「该放过」与「该拒」的某一形上看到的是否逐条相同

在已量的历史里：该放过而指针槽读不到更大号单元头的形，没有（丁、九段的 `units_only`、`urj`、`urj_row_record_also_hidden` 的槽上都是 `index_node@2`）。该拒的形（甲、乙、丙、戊、`baseline`、`partial`、`complete`、回卷两段）槽上都读不到更大号的单元头，拒。反过来，指针槽读法拒掉了三类「放过也不丢单元」的态：九段的 `later_mount…`、clean 前缀里 M1 建在 R 上的 3 个状态、tree_table_and_record/y2 里 5 个状态（K6 在这 5 个状态上也拒，所以这一类不是它独有的代价）。没有量到「读法放过而之后写掉还在用的单元」的形。（推的：没量过的形见 R3，不归我。）

## 4. R4：判决条目逐条（够得着 / 够不着、同样打中 / 没打中）

出局那串的原文：「出局：今天的判法、跨盘取最大（外部审核的建议）、逐盘取最小、照 I-7.7 第 ② 句、各盘各槽取最小、K3（两腿各自打中：正推量到二重故障两段误拒，攻方量到丙误放）」（`research/prompts/c834-r1-main-verification.md:20`）。

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| 出局：今天的判法（甲随盘序变、乙与取号两盘落完两序放过并写掉） | 打不中 K6、指针槽读法 | 这些历史上 K6、`pointer_slots_fixed`、`mapping_root_only` 都不读系统配置的号：九段 `partial`、`complete` 全拒（2.1 矩阵）；五段甲、乙全拒（2.2 表）。同样打中别的臂：不涉及 |
| 出局：跨盘取最大、逐盘取最小、照 I-7.7 第 ② 句、各盘各槽取最小 | 打不中两个留下的候选（够不着） | 这几个的出局历史是取号两盘落完、取号回卷、`urj` 被各盘各槽取最小误拒；K6 与指针槽读法在 `complete`、回卷两段（`acquisition_failed_on_device_1_rolled_back`、`acquisition_barrier_failed_rolled_back_on_both`）全拒，在 `urj` 放过（2.1 矩阵）；五段戊 17 个状态全拒（`out/aggregate-rollback.txt`） |
| 出局：K3，理由一「正推量到二重故障两段误拒」 | 够不着 K6（K6 的映射根那一半补上了），也打不中指针槽读法与 `mapping_root_only` | 九段 `units_only`、`urj_row_record_also_hidden`：K3 拒，K6、`mapping_root_only`、`pointer_slots_fixed` 做成（2.1 矩阵，K3 列即 `later_root_or_record`） |
| 出局：K3，理由二「攻方量到丙误放」 | **同样打中 K6**；打不中 `mapping_root_only`、`pointer_slots_fixed` | 丙：K6 放过并写掉 R 的 14 份单元，两序（`out/aggregate-all.txt:137`、`:140`，`out/bing-states.txt` 第 3 至 8 行）；`later_root_or_record` 控制重现（58 格与攻方逐行相同）；`mapping_root_only` 拒（`out/aggregate-mapping-tail.txt:43`、`:46`）、`pointer_slots_fixed` 拒（`out/aggregate-all.txt:149`、`:152`） |
| 「按号或按记录判的候选在丙与丁上看到的号与记录逐条相同而该判相反」 | 站得住，范围是「只读号与记录」 | 重现：丙里 `later_root_or_record` 放过并写掉，丁序号 16395（M1 的记录撕在 R 的记录槽上）它拒（`out/aggregate-all.txt:79`、`:88`，judge=refuse states=1；丁其余 3 个状态放过在 `:78`、`:87`；丙里它放过并写掉在 `:143`、`:146`）；而读映射根槽的候选在丙（槽上读不到更大号节点）与丁（`index_node@2`）看到的不同，所以这一句管不到 `mapping_root_only`，也管不到指针槽读法。K6 含 K3 半边，被这一句管到 |
| 「不靠号与记录能分开的只剩读盘上别的东西」 | 站得住 | 两个读指针槽的候选在丙、丁上分开（丙拒、丁放过），这是本轮量到的 |
| 「留下 K6 与指针槽读法两个零轮候选，都没被对方的历史攻过」 | 对 K6 不成立 | K6 被丙攻中，见上；指针槽读法被攻过的唯一一条（丁全拒）是原型缺陷（3.1） |
| 「指针槽读法没在正推的九段上跑」 | 已补 | 2.1 矩阵：原样读法错 4 段（含 urJ），改长度后错 1 段（`later_mount…`，可用性代价） |
| 「K6 没在丙、丁、戊上跑」 | 已补 | 2.2：丙错、丁对、戊对 |
| 「且在丁上拒（可用性代价，不违反不变量）」 | 误读 | 3.1：不是读法的代价，是读长度的缺陷 |

最后一格对「判决的两个留下的候选」的结论：两个候选各自的状态不是「都站得住」也不是「都站不住」，是**K6 出局（丙，同一个历史打中，不是新历史），指针槽读法（改读长度）站得住**，另有一个零轮的 `mapping_root_only` 与它同形（只读映射根那一槽，少读树表那一槽）。这一结论只在我拷贝上的五段与九段上成立，R3 列的形（多层映射树、写行发布只盖掉 R 一部分单元、映射根被数据单元覆盖、M2 自己写流的崩溃状态）我没量；这几形上两个读指针槽的候选怎么样归攻方腿。（推的）

## 5. 每条结论的推翻条件

- 「K6 被丙打中」：在入库装置上重造丙（M1 看得见 R 的映射、树表与记录读回全零，M1 隔离 R 的单元并把写行记录写进 R 的记录槽，写行根没落；M2 藏树表与映射根），K6 仍拒或 M2 之后 R 的单元一份不少。
- 「指针槽读法丁上全拒是读长度缺陷」：在入库装置上把读长度改成按单元类别后，丁仍有状态被拒，或入库 urJ 用例仍红。
- 「`pointer_slots_fixed` 与 `mapping_root_only` 没被打中」：任何一段 R 的单元还在、两个指针槽之一却是更大号的自证索引节点（例：写行发布只盖掉映射根一槽）的历史上它们放过并写掉其余单元；这一形我没造，归 R3。
- 「出局历史打不中两个读指针槽的候选」：复跑 `research/prompts/c834-r2-sonnet-model/tools/rerun.sh` 后 `complete` 或回卷两段里这两个候选放过并写掉 R 的单元。

## 6. 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果。
- R3（两个候选依赖的盘上事实被别的写盖掉或读不出）不归辩方：没造多层映射树、写行发布只盖掉 R 一部分、映射根被数据单元覆盖、M2 自己写流的崩溃状态。`pointer_slots_fixed` 与 `mapping_root_only` 在这些形上怎么样没量。
- 乙那一类里 `mapping_root_only` 只量了头尾两段（序号 0–20、16000 起），中间约 16000 个状态没跑；K6 与 `pointer_slots_fixed` 是全量。`pointer_slots_overwritten` 只在丁的 16 个状态与九段、入库用例上重现，没跑五段全量（攻方已跑）。
- `tools/rerun.sh` 是照模板改了 `rerun_steps` 写的，bash -n 通过；整条没有从头跑过一遍（约 40 分钟，机器忙时更长，我分批手跑了同样的步骤），交主 agent 在复核时跑：`bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c834-r2-sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c834-r1/repo --work <新的草稿目录> --model research/prompts/c834-r2-sonnet-model`，看末几行 `name=rerun_compare … =same`。
- 攻方原型依赖 `singlefs-checker-tier` 的库（崩溃状态枚举），我直接运行它编出来的二进制（经内存包装），没有跑 checker 档包的任何测试，没有起门禁阶段、GPU。若主 agent 认为这也算 checker 档，请在复核时说明。
- 开工时 `ps` 看到别的会话在跑 `cargo test --release -p singlefs-checker-tier …`（没有 qemu、vm-bench、fio、e152 这类性能测量），按定义加 `nice -n 19` 照跑；本机此刻负载高（loadavg 50 以上），几批分件跑到了 230 秒，被工具挪进后台等通知，没写轮询。
- 草稿目录里的冻结副本拷贝（`tree-candidates/`）与编译目录（`target/`、`target-model/`）交回前删；`out/runs/`（原始输出约 258M）一并删，要重现用 rerun.sh。
- 没读：禁读清单里的文件；外部审核报告原文。

## 7. cite-check 末行

命令：`python3 scripts/agents/cite-check.py research/prompts/c834-r2-sonnet-output.md --root /home/fy5090/code/singlefs`

```
  ✓ 核了 7 处引文、17 处裸行号，对上 7 处，没判 17 处（其中裸行号没有引文可比 9 处；逐处列在上面）
```
