# C834（择到的系统配置实例代号随盘序变） 第二轮攻方腿（Opus）报告（2026-10-15）

只攻 K6 与指针槽读法（正文「三、格」R1、R2、R3）。以下的数全是**副本上的数**（冻结副本 `~/.local/share/singlefs/work/c834-r1/repo/` 的拷贝 + 攻方补丁 + 攻方原型），不是入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/c834-r2-opus-model/`。复跑（整条经内存包装；本轮在别的会话占着大半核的时候分 9 批跑件，各批挂钟相加 1378.24 秒，另加编译与入库用例四个候选各约 1 到 4 分钟；空闲时会快些，没量）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c834-r2-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c834-r1/repo \
  --work <新的草稿目录>
```

末行判结果：`name=rerun_compare all_same`（`out/` 里六份逐行相同：`nine.txt`、`agg-first.txt`、`agg-three.txt`、`agg-partial.txt`、`derived.txt`、`existing-tests-summary.txt`）。汇总里的 `example_ordinal` 取那一格里最小的序号，与件的先后无关，复跑比对不用去字段（第一轮那份取的是先到的那一个，所以不确定）。这条腿自己没有把 `rerun.sh` 从头跑一遍（见「这条腿自己的限度」），只核了 `nine` 一段重跑逐字节相同、`three_mount_tail` 前 8 个状态重跑逐行相同。

| 文件 | sha256 |
|---|---|
| `.cargo/config.toml` | 891b14e98b472f513f13dea65ff5b2a7cf1429b97a29b2c488f06e90ee095fc5 |
| `.cargo/mutants.toml` | 6ea6e326095098250e41ab4ce358d280f9bbb5f0351c19a2981a69630e12d881 |
| `Cargo.lock` | d524561400001bf5fef7699cefdcf81bee9507e54d1579cf546e3b91aeed8a6a |
| `Cargo.toml` | f56ff8f5cef0e477df61e2e788824a07a70afa22ec7941a946d346d848449512 |
| `out/agg-first.txt` | ebe4eda3b3f743c01c568ecef0158eca0e12aebd1cbf361a693ac75aa1ac8a7b |
| `out/agg-partial.txt` | cd91b760be40f7f4cd54efadb3d45004013d99c89a546a69690c8588d352c329 |
| `out/agg-three.txt` | 9dfdcce0d4e3c640228c9bd0ce8cb069f0b4c6fd6a4fc98b09be897d20acddc0 |
| `out/derived.txt` | fb2118984d0d9312cb7b075d8bcea9e67f31b6c9378b59645c328bdcfe8f0e29 |
| `out/excerpts.txt` | 32ff1dce02661803471495c10120b493e48ccaa8b6f8a97604c7859977525ebc |
| `out/existing-tests-summary.txt` | 5c6ee51b82bfe5dd0965e04da2fbaa01824f1f0b39350db5cbd679b0f0841f5a |
| `out/nine.txt` | fc53e91b29a377309274755720f883743ecb26f6929b28b9882bcb9519f8eee2 |
| `patch/apply_candidates.py` | ff39aba6925a2a52cb4304b0975899068afbbcc9133b1dfea066a014a783fd6d |
| `src/first_round_histories.rs` | 80b1fed2619bc1fdac20a631afaabd165890ca897d397bee951b3f6c387c093f |
| `src/main.rs` | 0e439058cc4a93b3f27ff4346a92f70367a835819fe35354f9ab36a7af9fa585 |
| `src/nine.rs` | 78aab095809139e778255cc324013e7f519c777f77784d35d8c15d327e9d8fd1 |
| `tools/aggregate.py` | 8d3cd65ee5a29b5bc53d52e92b519648b69acdd2edb7b70f5a035ceff135272d |
| `tools/derive.py` | 5735af7d2406480487900fb182a3d4a86a7c84bb112d97d4e3f4d9a34102b2f8 |
| `tools/rerun.sh` | 0979c13fd58e4fafbbd0097f7ad1370197e89fc1012006270cee9634343edd8a |
| `tools/run-batch.sh` | 9ae8b1a464467b92dcf21e565a67669f521042117fbab533d4bd1fb8a87a3162 |

（`.cargo/` 两份是冻结副本 `.cargo/` 原样拷来的；`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段，第 1 到 73 行与模板逐字相同。）

装置是什么：

- **补丁** `patch/apply_candidates.py`：只改冻结副本 `crates/singlefs-core/src/mount.rs` 的那一判（`crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」接 `crates/singlefs-core/src/mount.rs:2079` 「<= root.instance =>」）与它的调用处。那一判每走到一次，把下面几样凭据全算一遍记一行，再按候选判拒或放过：`chosen`（今天读的那个号）、`later_root_or_record`（K3）、`mapping_root_newer`（K6 多加的那一句，读法逐字照第一轮正推补丁：映射根两个落点各读 16 KiB、`parse_index_node`、实例代号比 R 大）、`pointer_r1`（第一轮攻方「指针槽读法」原型原样：四个落点各读 32 KiB，依次试打包记录单元、数据单元、索引节点）、`pointer_per_kind`（同一读法，只把索引节点改按 16 KiB 解）、`slots`（四个落点各读出什么，诊断）。候选：`today`（与冻结副本同判）、`k3`、`k6`、`pointer_r1`、`pointer_per_kind`、`always_refuse`。
- **原型** `src/`：入库搭建 `crates/singlefs-harness/tests/common_unwitnessed_publish/mod.rs`（`#[path]` 引进）起 384 槽小盘、第一个文件、覆盖写 5 次，录 R=(1,9)（P=(1,8)），崩在 R 的根槽之后、轮换之前。几段历史各录一次挂载写流，按崩溃点重放的枚举域（`crash_state_enumeration_tables` + `extract_flow_facts`，每段 `full_expansion`，原地覆写取三态）逐个崩溃状态跑池级 checker，再做被判的那次挂载，两种交盘次序各一次。被判的那次挂载先照 `today` 跑：没走到那一判，结局对每个候选相同；走到了，按记下的凭据算出每个候选拒还是放过，再强制走另一支跑一遍（那一判之前的路径对每个候选逐字相同）。全部 139562 个状态里那一判每次挂载至多走到一次（`creds=many` 零行），没有要逐个候选重跑的。
- 结局记：拒（拒因、写了几次）或做成（落到哪一版、写行那次发布的根落盘之前**这次挂载新写掉**的 R 单元份数、那一刻 R 的根还在不在根环里、那一刻池级 checker 红的不变量）。「R 的单元」= R 那次发布写出的 14 份单元写（7 个单元、两块盘各一份）；「新写掉」= 挂载前逐字节还是 R 写的、写行根落盘前已不是的那几份。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 |
|---|---|---|
| R1 K6 × 丙（第一轮攻方的丙） | **打中**：3 个状态两序都放过，被判的那次挂载写行根落盘前新写掉 R 的 14 份单元，那一刻 I-2.1、I-4.8、I-7.2、I-7.4 红；那一刻之前状态不红、R 的单元一份不少、R 的根在环里 | 量过（副本） |
| R1 K6 × 新造「三次挂载」（M0 建在 R 上的记录落了 → M2 放过、护住 R → M2 记录落了根没落 → M3 时 R 的账与 M0 那条记录读不出） | **打中**：3 个状态两序都放过，M3 新写掉 R 的 14 份单元，同四条红；之前不红、R 一份不少 | 量过（副本） |
| R1 K6 × 甲、乙、丁、戊与第一轮正推九段 | 没打中：甲乙戊全拒（对）；丁 4 个状态全放过（对，含 K3 拒的那个撕裂记录态 16395）；九段与第一轮正推的结局表逐格相同 | 量过（副本） |
| R2 指针槽读法 × 第一轮正推九段 | 第一轮原型原样（`pointer_r1`）：units_only、urj、urj_row_record_also_hidden 三段该放过而拒（两序）；改掉缓冲长度那一处（`pointer_per_kind`）：九段里该判的都判对，later_mount 一段拒（那一段放过或拒都不丢单元） | 量过（副本） |
| R2 丁上全拒的原因 | **原型的实现缺陷，不是读法本身**：丁那 4 个状态里 R 的树表、映射根四个落点都是 M1 写的索引节点、实例代号 2（`slots=tt0:idx:2,tt1:idx:2,mr0:idx:2,mr1:idx:2`）；第一轮原型拿 32 KiB 的缓冲去解索引节点，`parse_index_node` 见长度不是 16384 就报错，于是一个都认不出。按 16 KiB 解之后 4 个状态全放过、入库 `urJ` 用例转绿（13 份入库 harness 用例 142 条全过）。第一轮报告推的「32 KiB 单元的后半截」不成立 | 量过（副本） |
| R3 指针槽读法 × 全部新形 | 没打中写掉还在用的单元的：凡是 R 的单元此前一份不少的状态，四个落点要么读回全零（读故障藏着）、要么还是 R 自己的单元，一律拒 | 量过（副本） |
| R3 两个候选 × 丁只藏 R 的记录（M1 写行发布的单元只落了一部分） | 不分辨两个候选：2808 个状态两个候选都放过，被判的那次挂载把 R 剩下的单元也写掉（2796 个状态新写掉 ≥1 份）；之前 R 的映射根两份已被 M1 盖掉，状态已红（I-7.2、I-7.4 等）。算不算「写掉还在用的单元」交主 agent 判（第三节） | 量过（副本） |
| R3 两个候选 × 丁加 y2 读故障（二重故障） | 两个候选都拒 16388 个 R 的单元已被写掉的状态（可用性代价，不违反不变量；入库用例没有这一形） | 量过（副本） |
| R3 多层映射树、映射根被数据单元覆盖 | 没造出来：384 槽小盘映射树只有一层；全部走到那一判的状态里四个落点只出现「全零」与「索引节点」两种，没有数据单元落在 R 的指针槽上 | 没造（结论是推的） |
| 入库用例 | 13 份 harness 用例（引 `common_unwitnessed_publish` 的 12 份 + `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`）：today、K6、`pointer_per_kind` 各 142 过 0 败；`pointer_r1` 141 过 1 败（`urJ` 那一条） | 量过（副本） |

一句话：K6 被两段历史打中，都是同一个病根——K6 里继承自 K3 的「有带更大号的记录」那一句：带更大号的记录所属的发布建在 P 上、没覆盖 R 的单元（写它的那次挂载当时用别的东西护住了 R），这份护住到下一次挂载时没留下，记录却留下了。指针槽读法（改掉原型缺陷之后）在本轮全部历史上没被造出写掉还在用的单元，代价是读故障藏住指针槽时一律拒。

## 一、R1：K6 在各段历史上判成什么

### 打中一：丙（第一轮攻方的丙，K6 第一次在它上面跑）

历史（与第一轮攻方同一造法，每一步许可它的那一句见第一轮攻方报告第二节「五段历史」表的丙那一行）：M1 时 R 的树表与 R 的记录读回全零、映射读得出，M1 退回 P、按 `C_NEWER_ROOTS` 从映射现算隔离 R 的单元、把写行记录（实例代号 2）写进 R 的记录槽，写行根没落就崩；被判的那次挂载 M2 时 R 的树表与映射根读回全零（`tree_and_mapping`；R 的记录槽已被 M1 的记录占了）。

原样（`out/agg-first.txt`）：

```
name=c834_cell history=tree_table_and_record/tree_and_mapping class=later_records_r_units_intact state_red=- candidate=k6 order=forward judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=3 example_ordinal=16396
name=c834_cell history=tree_table_and_record/tree_and_mapping class=later_records_r_units_intact state_red=- candidate=k6 order=reversed judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=3 example_ordinal=16396
name=c834_cell history=tree_table_and_record/tree_and_mapping class=later_records_r_units_intact state_red=- candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=3 example_ordinal=16396
name=c834_cell history=tree_table_and_record/tree_and_mapping class=later_records_r_units_intact state_red=- candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=3 example_ordinal=16396
```

序号 16396 那一个状态的原样行（`out/excerpts.txt` 第 1、2 行，正向那一行，`cands=` 里每个候选一格）：

```
name=c834_state history=tree_table_and_record faults=tree_and_mapping ordinal=16396 segment=2 per_device=0:1/2,1:1/2 later_records=[2] r_in_ring=true r_units_overwritten_in_state=0 state_red=- order=forward creds=once;chosen:2,later_root_or_record:true,mapping_root_newer:false,pointer_r1:false,pointer_per_kind:false,slots:tt0:zero,tt1:zero,mr0:zero,mr1:zero cands=today:pass:mounted:eff=(1,8):inst=3:new=14:total=14:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;k3:pass:mounted:eff=(1,8):inst=3:new=14:total=14:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;k6:pass:mounted:eff=(1,8):inst=3:new=14:total=14:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;pointer_r1:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;pointer_per_kind:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;always_refuse:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
```

读法：`state_red=-`、`r_units_overwritten_in_state=0`：M2 之前这个状态池级 checker 不红、R 的单元一份不少；`r_in_ring=true`：R 的根在根环里。K6 那一格 `later_root_or_record:true`（M1 那条实例代号 2 的记录读得出）、`mapping_root_newer:false`（映射根两槽读回全零）——放过只靠继承自 K3 的那一句。放过之后 `new=14`：写行根落盘前新写掉 R 的 14 份单元，R 的根还在环里，那一刻 I-2.1、I-4.8、I-7.2、I-7.4 红。

为什么这一形本来就该打中 K6（推的，量的是上面那几行）：K6 = K3 的凭据「或」映射根槽凭据，凡是 K3 放过的 K6 都放过；第一轮已量到 K3 在丙上放过·红（第一轮判决第二节 G2 那一行）。K6 没在丙上跑过，这一轮补上：结局与 K3 逐格相同。

### 打中二：新造「三次挂载」（第一轮两腿都没造过；R3 列的「M2 自己写流的崩溃状态」）

历史（`src/main.rs` 的 `three_mount_tail`）：

1. 同第一轮：R=(1,9) 的根槽落了、系统配置轮换没落。
2. M0：无读故障，择 R（落到 (1,9)），取号 2，写行发布的单元与记录落了、写行根没落（第一轮正推的 `later_mount_records_landed_r_units_intact` 那一张盘面，摘要行 `m0=(instance=2 effective=(1,9)) m0_writes_before_row_root=18 image_a_red=- image_a_r_units_overwritten=0`）。许可它的是记录先于根落盘的写序：崩在记录之后、根槽之前是崩溃点重放枚举域里的状态（D23（journal 的角色与格式） 已定项 15 那一形）。
3. M2：R 的记录、树表、映射根读回全零（入库 Y2 用例那三段，`crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs:257` 「fn a_newer_readable_root_whose_only_record_mapping_root_and_tree_table_read_back_zero_refuses_the_writable_mount_before_any_write(」那一条造的就是这三段），M0 那条实例代号 2 的记录读得出。M2 退回 P、走到那一判，K6 凭 M0 那条记录放过（`crates/singlefs-core/src/mount.rs:2089` 「None => continue,」），取号 3、写行……（M2 那一判记下的凭据：`later_root_or_record=true`、`mapping_root_newer=false`、四个落点全零、判 pass；原样在每件输出开头那一行 `name=c834_history … m2_judge=…`，草稿目录 `runs/three/`）。M2 写下的没碰 R 的单元（下面每个状态 `r_units_overwritten_in_state=0`；第一轮正推那一条安全用例同一个结论）。护住 R 的是 M0 那条记录（它一读不出 M3 就写掉，见下面的对照）；具体靠的是记录点名的那一份还是由记录拼根读出的账，没分开查。
4. M2 的写流：缩短成从 M2 的第一条记录写起（取号与写行发布的 18 次单元写并进基镜像），之后 11 段 39 次写全量枚举，8227 个崩溃状态（缩法与理由见「这条腿自己的限度」）。
5. M3（被判的那次挂载）：R 的那三段照旧读回全零，另加 M0 那条记录读回全零（`y2_m0_record`）。读故障在两次挂载之间会变，与第一轮攻方丙丁、入库 `ur` / `urJ` / Y2 同一个前提（第一轮攻方报告 `research/prompts/c834-r1-opus-output.md:226` 「- 「读回全零」是本仓的读故障模型（入库用例同一个包装 `DeviceReadingBackAllZeroOnRanges`）；M1、M2 两次挂载各藏不同的段，这一形可达性的前提是读故障在两次挂载之间会变——与入库 `ur` / `urJ` / Y2 那几形同一个前提。」）。比 Y2 那三段多一处读故障。

原样（`out/agg-three.txt`，`y2_m0_record` 那一组走到那一判的全部格；其余 8223 个状态没走到那一判：每个候选都拒在被抛弃根那一支，拒因 `AccountOfAnAbandonedRoot`，按段号是 M2 的写行根已落之后的状态）：

```
name=c834_cell history=three_mount_tail/y2_m0_record class=later_records_r_units_intact state_red=- candidate=k6 order=forward judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=3 example_ordinal=1
name=c834_cell history=three_mount_tail/y2_m0_record class=later_records_r_units_intact state_red=- candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1 example_ordinal=0
name=c834_cell history=three_mount_tail/y2_m0_record class=later_records_r_units_intact state_red=- candidate=k6 order=reversed judge=pass outcome=mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4 states=3 example_ordinal=1
name=c834_cell history=three_mount_tail/y2_m0_record class=later_records_r_units_intact state_red=- candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1 example_ordinal=0
name=c834_cell history=three_mount_tail/y2_m0_record class=later_records_r_units_intact state_red=- candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4 example_ordinal=0
name=c834_cell history=three_mount_tail/y2_m0_record class=later_records_r_units_intact state_red=- candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4 example_ordinal=0
```

序号 1 那一个状态的原样行（`out/excerpts.txt` 第 5 行，正向）：

```
name=c834_state history=three_mount_tail faults=y2_m0_record ordinal=1 segment=0 per_device=0:3/2,1:3/2 later_records=[2,3] r_in_ring=true r_units_overwritten_in_state=0 state_red=- order=forward creds=once;chosen:3,later_root_or_record:true,mapping_root_newer:false,pointer_r1:false,pointer_per_kind:false,slots:tt0:zero,tt1:zero,mr0:zero,mr1:zero cands=today:pass:mounted:eff=(1,8):inst=4:new=14:total=14:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;k3:pass:mounted:eff=(1,8):inst=4:new=14:total=14:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;k6:pass:mounted:eff=(1,8):inst=4:new=14:total=14:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;pointer_r1:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;pointer_per_kind:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;always_refuse:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
```

读法：`per_device=0:3/2,1:3/2`：两块盘两槽是 3 与 2（M2 取过号 3）；`later_records=[2,3]`：盘上有 M0 那条（2，被读故障藏着）与 M2 那条（3）；`state_red=-`、`r_units_overwritten_in_state=0`、`r_in_ring=true`：M3 之前不红、R 的单元一份不少、R 在环里。K6 那一格 `later_root_or_record:true`（只剩 M2 那条读得出），`mapping_root_newer:false` → 放过 → `new=14`、四条红。M2 那条记录所属的发布建在 P 上（M2 落到 (1,8)），它的账里没有 R 的单元；M2 当时靠的是 M0 那条记录护住 R，这份护住没有落进 M2 自己的记录。

对照（同一段历史，M3 只藏 R 的那三段、M0 那条读得出，`y2`）：K6 放过的 4 个状态一份都没写掉（`out/agg-three.txt`）：

```
name=c834_cell history=three_mount_tail/y2 class=later_records_r_units_intact state_red=- candidate=k6 order=forward judge=pass outcome=mounted_r_units_kept:red=- states=4 example_ordinal=0
name=c834_cell history=three_mount_tail/y2 class=later_records_r_units_intact state_red=- candidate=k6 order=reversed judge=pass outcome=mounted_r_units_kept:red=- states=4 example_ordinal=0
```

也就是说护住 R 的只是 M0 那条记录；它一读不出，K6 拿来放过的凭据（M2 那条）护不住 R。第一轮正推写的那句推理（`research/prompts/c834-r1-sonnet-output.md:107` 「为什么「有更大号的 journal 记录」够放过（推的，只有这一段有量）：那条记录所属的发布建在 R 上时，它自己的账里就含 R 的单元，预隔离会护住它们；建在更旧的 P 上时，它的单元已经覆盖了 R 的单元。第二种若只盖掉 R 的一部分单元（没造这一形），剩下的没被护：判不了。」）的第二种情形在丙与这一段上都不成立：记录建在 P 上，而它的单元**没有**覆盖 R 的单元——写它的那次挂载当时用别的东西护住了 R（丙里是从映射现算的隔离，这里是 M0 那条建在 R 上的记录）。

推翻条件：在入库装置上重造序号 1 那个状态（M0 记录落了根没落 → M2 在 Y2 三段读故障下放过 → M2 记录落了根没落 → M3 再藏 M0 那条记录），照 K6 判，M3 放过而写行根落盘前 R 的单元一份不少，或 M3 拒，就推翻这一格。

### 没打中：甲、乙、丁、戊与第一轮正推九段

`out/agg-first.txt` 里 K6 那几格（两序结局逐格相同，只写正向；「全拒」= 拒在任何写之前）：

| 历史 / 被判那次挂载的读故障 | 状态类（R 的单元此前） | K6 | 该判 |
|---|---|---|---|
| 甲（`clean/y2`、`tree_table_and_record/y2`，号只落一块盘） | 一份不少 | 各 4 个全拒 | 拒（对） |
| 乙（同上两前缀，号两盘都落、没写记录） | 一份不少 | 16384、16387 个全拒 | 拒（对） |
| `clean/y2` 里 M1 记录落了根没落 | 一份不少 | 3 个放过，`mounted_r_units_kept` | 放过或拒都不丢（对） |
| 丁（`root_units_and_record/none`） | 已被 M1 写掉 14 份 | 4 个全放过（含 K3 拒的 16395：M1 的记录撕在 R 的记录槽上，映射根槽是 M1 的索引节点 `idx:2`） | 放过（对） |
| 戊（`acquisition_rolled_back/y2`，17 个） | 一份不少 | 走到那一判的 17 个全拒 | 拒（对） |

九段（`out/nine.txt`，每段两序、每个候选各挂一次）：K6 的结局与第一轮正推结局表逐格相同（baseline、partial、complete、两段回卷拒；units_only、urj、urj_row_record_also_hidden、later_mount 做成，做成的都没新写掉 R 的单元）。

## 二、R2：指针槽读法

### 在第一轮正推九段上各判成什么

`out/nine.txt`，每格 = 盘序 [盘 0, 盘 1] / [盘 1, 盘 0] 的结局，两序相同只写一个；「做成[新写掉 14]」= 做成且这次挂载写行根落盘前新写掉 R 的 14 份单元；「该判」照第一轮正推结局表：

| 历史 | today | K3 | K6 | 指针槽（第一轮原型原样） | 指针槽（索引节点按 16 KiB 解） | 该判 |
|---|---|---|---|---|---|---|
| baseline | 拒 | 拒 | 拒 | 拒 | 拒 | 拒 |
| partial | 做成[新写掉14]/拒 | 拒 | 拒 | 拒 | 拒 | 拒 |
| complete | 做成[新写掉14] | 拒 | 拒 | 拒 | 拒 | 拒 |
| units_only | 做成 | 拒 | 做成 | **拒** | 做成 | 做成 |
| urj | 做成 | 做成 | 做成 | **拒** | 做成 | 做成 |
| urj_row_record_also_hidden | 做成 | 拒 | 做成 | **拒** | 做成 | 做成 |
| later_mount_records_landed_r_units_intact | 做成 | 做成 | 做成 | 拒 | 拒 | 放过或拒都不丢 R 的单元 |
| acquisition_failed_on_device_1_rolled_back | 拒 | 拒 | 拒 | 拒 | 拒 | 拒 |
| acquisition_barrier_failed_rolled_back_on_both | 拒 | 拒 | 拒 | 拒 | 拒 | 拒 |

第一轮原型原样在九段里三段该放过而拒（都是 R 的单元已被写掉、入库 `urJ` 认的放过那一类）；改掉下面那一处之后九段全对。两种读法在 later_mount 都拒：那一段 R 的四个落点读回全零（`slots=tt0:zero,tt1:zero,mr0:zero,mr1:zero`），R 的单元一份不少，拒不违反不变量，代价是这一形（取号的那次挂载择了 R、记录落了根没落，之后 R 的三段读不出）做不成可写挂载、要等读故障消失。

### 丁上全拒的原因：原型的实现缺陷

丁（与 urj 同一类）那几个状态里，R 的树表与映射根四个落点上是 M1 写行发布写的索引节点，实例代号 2。原样（`out/nine.txt`，urj 正向两行）：

```
name=c834_nine_row history=urj per_device=0:1/2,1:1/2 image_red=I-2.1+I-4.8+I-7.2+I-7.4 r_units_overwritten_in_image=14 order=forward candidate=pointer_r1 judge=refuse slots=tt0:idx:2,tt1:idx:2,mr0:idx:2,mr1:idx:2 outcome=refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
name=c834_nine_row history=urj per_device=0:1/2,1:1/2 image_red=I-2.1+I-4.8+I-7.2+I-7.4 r_units_overwritten_in_image=14 order=forward candidate=pointer_per_kind judge=pass slots=tt0:idx:2,tt1:idx:2,mr0:idx:2,mr1:idx:2 outcome=mounted:eff=(1,8):inst=3:new=0:total=14:r_in_ring=true:red=-
```

丁全量（`out/agg-first.txt`）：

```
name=c834_cell history=root_units_and_record/none class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=forward judge=pass outcome=mounted_r_units_kept:red=- states=4 example_ordinal=16395
name=c834_cell history=root_units_and_record/none class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=reversed judge=pass outcome=mounted_r_units_kept:red=- states=4 example_ordinal=16395
name=c834_cell history=root_units_and_record/none class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_r1 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4 example_ordinal=16395
name=c834_cell history=root_units_and_record/none class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_r1 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4 example_ordinal=16395
```

原因（读码 + 上面的量）：第一轮原型每个落点只读一次 32 KiB（`research/prompts/c834-r1-opus-model/patch/apply_candidates.py:80` 「usize::try_from(singlefs_format::DATA_UNIT_BYTES).expect("32768"),」），再拿同一个缓冲去解索引节点（同一份文件 `:88` 「if let Ok(header) = crate::unit::parse_index_node(&bytes) {」）；`parse_index_node` 一进来就判长度（`crates/singlefs-core/src/unit.rs:391` 「if bytes.len() != usize::try_from(NODE_BYTES).expect("16384") {」），32 KiB 的缓冲永远解不开。四个落点上偏偏都是索引节点，于是一个凭据都认不出、全拒。第一轮报告推的原因（`research/prompts/c834-r1-opus-output.md:138` 「- **攻方自提的指针槽读法**（被攻过零轮）：甲、乙、丙、戊都对，丁 4 个状态全拒，入库 `urJ` 用例因此红（第四节）。原因没查到底：推测 M1 写到 R 的树表 / 映射根那一槽的是一个 32 KiB 单元的后半截，槽首没有单元头可解（推的，没核）。」）不成立：槽首就是一个完整的索引节点头。

**读法本身在 `urJ` 那一形上分得开**：该放过的 `urJ` 一类（丁、units_only、urj、urj_row_record_also_hidden）四个落点读出 `idx:2`；本轮全部该拒的状态（甲、乙、丙、戊、三次挂载、九段里该拒的五段）四个落点都读回全零（`zero`），见第三节的统计。在没有读故障盖住指针槽的那几形上，该放过与该拒的两边看到的不同。盖住了就分不开：丁再加 y2 读故障（第三节，R 的单元已被写掉、二重故障）四个落点同样读回全零，与该拒的甲乙逐条相同，读法在那里拒——错在拒的一边，是可用性代价，入库用例没有这一形。

### 入库用例

13 份 harness 用例（引 `common_unwitnessed_publish` 的 12 份 + `an_abandoned_roots_unreadable_account_is_recomputed_from_the_mapping.rs`，非 checker 档）在副本上四个候选各跑一遍（`out/existing-tests-summary.txt` 原样）：

```
candidate=today binaries=13 passed=142 failed=0 ignored=1 failed_tests=
candidate=k6 binaries=13 passed=142 failed=0 ignored=1 failed_tests=
candidate=pointer_per_kind binaries=13 passed=142 failed=0 ignored=1 failed_tests=
candidate=pointer_r1 binaries=13 passed=141 failed=1 ignored=1 failed_tests=a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts
```

第一轮那一条红（`urJ` 那一条）在 `pointer_r1` 下复现，在 `pointer_per_kind` 下转绿；K6 下全绿与第一轮正推量的一致。这 13 份在 today、K6、`pointer_per_kind` 三个候选下逐条同过：入库用例分不开这三个候选（K6 的两段打中都没有入库用例覆盖）。

推翻条件：在入库装置上把第一轮原型那一读法的索引节点改按 16 KiB 解，`urJ` 那一条仍红，或丁那 4 个状态仍拒，就推翻「实现缺陷」这一格。

## 三、R3：两个候选靠的盘上事实什么时候会被盖掉或读不出

### 两个候选在全部走到那一判的状态里各读到了什么

四个落点（R 的树表两份 `tt0`/`tt1`、映射根两份 `mr0`/`mr1`）在每个走到那一判的（状态，次序）上读出的东西（`zero` 读回全零；`idx:<n>` 按 16 KiB 解开的索引节点、实例代号 n），按「R 的单元在这个状态里此前是否一份不少」分（数的是（状态，次序）行，每个状态两行；草稿目录 `runs/slots-tally.txt`）：

| 历史 / 被判那次挂载的读故障 | R 的单元此前 | 四个落点读出 | 行数 |
|---|---|---|---|
| acquisition_rolled_back/y2 | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 34 |
| clean/y2 | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 32790 |
| root_units_and_record/none | 已被写掉 | `tt0:idx:2,tt1:idx:2,mr0:idx:2,mr1:idx:2` | 8 |
| root_units_and_record/record_only | 已被写掉 | `tt0:idx:1,tt1:idx:1,mr0:idx:2,mr1:idx:2` | 1184 |
| root_units_and_record/record_only | 已被写掉 | `tt0:idx:1,tt1:idx:2,mr0:idx:2,mr1:idx:2` | 1184 |
| root_units_and_record/record_only | 已被写掉 | `tt0:idx:2,tt1:idx:1,mr0:idx:2,mr1:idx:2` | 1184 |
| root_units_and_record/record_only | 已被写掉 | `tt0:idx:2,tt1:idx:2,mr0:idx:2,mr1:idx:2` | 2064 |
| root_units_and_record/y2 | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 24 |
| root_units_and_record/y2 | 已被写掉 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 32776 |
| three_mount_tail/y2 | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 8 |
| three_mount_tail/y2_m0_record | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 8 |
| tree_table_and_record/tree_and_mapping | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 8 |
| tree_table_and_record/y2 | 一份不少 | `tt0:zero,tt1:zero,mr0:zero,mr1:zero` | 32800 |

读法：R 的单元一份不少的行，四个落点**全部**读回全零（读故障藏着）；落点上出现实例代号比 R 大的索引节点的行，R 的单元都已被写掉。没有一行是「R 一份不少而指针槽里是更新的单元」——指针槽读法（按 16 KiB 解）在本轮没有放过一个 R 的单元还在的状态。映射根两槽同理：K6 的映射根那一句只在 R 已被写掉的行上为真；K6 的两段打中全靠继承自 K3 的那一句。

为什么造不出「R 一份不少而指针槽里是更新的单元」（推的，读码）：R 的指针槽只会被不知道 R 的那次挂载写（R 的根槽读不出、或 R 的记录也读不出的 `urJ` 一类）；知道 R 的挂载（丙里从映射现算的、三次挂载里靠记录预隔离的、择了 R 的）都不写 R 的单元。被不知道 R 的挂载写了，R 的那几份就已经没了，状态已红。

### 形一：M2 自己写流的崩溃状态（三次挂载）

见第一节打中二。缩短之后的 8227 个状态里，走到那一判的只有序号 0 到 3 那 4 个：段 0（M2 的两份记录写）里的 3 个——序号 0 盘上只有 M0 那条（`later_records=[2]`），序号 1、2 有 M2 那条（`[2,3]`，按段号推是只落了一份的两种）——与段 1（写行根那一写）里的序号 3；M2 的写行根一落，R 被 M2 的实例表抛弃，每个候选都拒在被抛弃根那一支（`refused:AccountOfAnAbandonedRoot`，8223 个状态，两种读故障一样）。没缩短的那段历史（M2 写行发布的 18 次单元写也枚举，270378 个状态）只跑了前 31000 个（`three_mount/y2_m0_record`，草稿 `runs/threefull/`）：都在 M2 的记录落盘之前，K6 与指针槽读法全拒（对），today 全放过·红；没跑全，不进结论。

### 形二：写行发布只盖掉 R 一部分单元（丁，被判那次挂载只藏 R 的记录）

历史：丁的 M1（R 的根槽、单元、记录读回全零，看不见 R），枚举 M1 写流的 20516 个崩溃状态；被判的那次挂载只让 R 的记录读回全零（`record_only`）。M1 写行发布那 14 次单元写只落了一部分的状态里，R 的映射根两份已被 M1 的索引节点盖掉、树表可能还是 R 的（`idx:1`），映射现算不成、R 的记录读不出，走到那一判。原样（`out/agg-partial.txt` 里走到那一判的格按候选与次序合计，草稿目录现算）：

```
('always_refuse', 'forward', 'refuse', 'refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords') 2808
('always_refuse', 'reversed', 'refuse', 'refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords') 2808
('k3', 'forward', 'refuse', 'refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords') 2808
('k3', 'reversed', 'refuse', 'refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords') 2808
('k6', 'forward', 'pass', 'mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4') 2796
('k6', 'forward', 'pass', 'mounted_r_units_kept:red=-') 12
('k6', 'reversed', 'pass', 'mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4') 2796
('k6', 'reversed', 'pass', 'mounted_r_units_kept:red=-') 12
('pointer_per_kind', 'forward', 'pass', 'mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4') 2796
('pointer_per_kind', 'forward', 'pass', 'mounted_r_units_kept:red=-') 12
('pointer_per_kind', 'reversed', 'pass', 'mounted_overwrote_r_units:red=I-2.1+I-4.8+I-7.2+I-7.4') 2796
('pointer_per_kind', 'reversed', 'pass', 'mounted_r_units_kept:red=-') 12
```

这 2808 个状态在被判那次挂载之前就红（`out/agg-partial.txt` 里它们的 `state_red` 四种：`I-2.1+I-4.8+I-7.2+I-7.4`、加 `I-9.1`、加 `I-3.1+I-3.11`、三样都加），每一种都含 I-7.2：R（根环里代号最大的那条）此前已走不完。两个候选都放过，被判的那次挂载把 R 剩下的单元也写掉（2796 个状态新写掉 ≥1 份）；K3 与一律拒在这里拒。

判（照第一轮判决对丁的读法）：不当打中。这一形与丁同类——R 已被一次看不见它的挂载写坏，放过让还没被写掉的那几份也写掉，拒则只要盘不变就每次都拒。它与丁的差别是放过这一支**多写掉了**几份（丁那 4 个状态 `new=0`），第一轮判决「放过没有让它更坏」那句在这一形上不成立，交主 agent 定这算不算「写掉还在用的单元」。不论怎么定，它不分辨 K6 与指针槽读法（两个逐状态同判）。

### 形三：丁再加 y2 读故障（二重故障）

丁的 M1 写流 20516 个状态，被判那次挂载再藏 R 的树表、映射根、记录：R 的单元已被写掉的 16388 个状态 K6 与指针槽读法都拒（today 放过）。原样（`out/agg-first.txt`）：

```
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4 candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=288 example_ordinal=824
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4 candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=288 example_ordinal=824
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=288 example_ordinal=824
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=288 example_ordinal=824
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4+I-9.1 candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=720 example_ordinal=779
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4+I-9.1 candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=720 example_ordinal=779
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4+I-9.1 candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=720 example_ordinal=779
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-3.1+I-3.11+I-4.8+I-7.2+I-7.4+I-9.1 candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=720 example_ordinal=779
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=8016 example_ordinal=20
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=8016 example_ordinal=20
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=8016 example_ordinal=20
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=8016 example_ordinal=20
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4+I-9.1 candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4452 example_ordinal=11
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4+I-9.1 candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4452 example_ordinal=11
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4+I-9.1 candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4452 example_ordinal=11
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.2+I-7.4+I-9.1 candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4452 example_ordinal=11
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4 candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1940 example_ordinal=10
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4 candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1940 example_ordinal=10
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4 candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1940 example_ordinal=10
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4 candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1940 example_ordinal=10
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4+I-9.1 candidate=k6 order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=972 example_ordinal=9
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4+I-9.1 candidate=k6 order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=972 example_ordinal=9
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4+I-9.1 candidate=pointer_per_kind order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=972 example_ordinal=9
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten state_red=I-2.1+I-4.8+I-7.4+I-9.1 candidate=pointer_per_kind order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=972 example_ordinal=9
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=k6 order=forward judge=not_reached outcome=mounted_r_units_kept:red=- states=16 example_ordinal=16400
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=k6 order=forward judge=not_reached outcome=refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion states=4100 example_ordinal=16402
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=k6 order=reversed judge=not_reached outcome=mounted_r_units_kept:red=- states=16 example_ordinal=16400
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=k6 order=reversed judge=not_reached outcome=refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion states=4100 example_ordinal=16402
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=pointer_per_kind order=forward judge=not_reached outcome=mounted_r_units_kept:red=- states=16 example_ordinal=16400
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=pointer_per_kind order=forward judge=not_reached outcome=refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion states=4100 example_ordinal=16402
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=pointer_per_kind order=reversed judge=not_reached outcome=mounted_r_units_kept:red=- states=16 example_ordinal=16400
name=c834_cell history=root_units_and_record/y2 class=r_units_already_overwritten_r_not_in_ring state_red=- candidate=pointer_per_kind order=reversed judge=not_reached outcome=refused:PublishWitnessedBySystemConfigurationNewerThanTheSelectedVersion states=4100 example_ordinal=16402
```

拒不违反不变量（可用性代价）；这一形入库用例没有，按跑前条款字面不算「该做成的拒且那一形是入库用例认的合法态」。两个候选同判，不分辨。

### 形四、形五：多层映射树、映射根被数据单元覆盖（没造出来）

- 多层映射树：384 槽小盘上中央映射树只有一层，这一轮没换几何（入库搭建写死这一档）。推的：映射根读得出、下面某个节点读不出时映射现算不成、走到那一判；那时映射根槽里还是 R 自己的索引节点（实例代号 = R），K6 的映射根那一句不成立、退回继承自 K3 的那一句，打中一、二的机制照样在；指针槽读法四个落点也是 R 自己的，拒。都不会多放过。
- 映射根被数据单元覆盖：上表全部走到那一判的行里，四个落点只出现 `zero` 与 `idx`；这一轮的几段历史里分配器把 M1 写行发布的索引节点放回 R 的索引节点槽，没造出数据单元落在 R 的指针槽上的状态。推的：数据单元从那一槽起头时，K6 的映射根那一句按 16 KiB 当索引节点解不开（类标签不对），退回 K3 那一句；`pointer_per_kind` 按 32 KiB 解得开数据单元头、放过；落在那一槽的是一个数据单元的后半截时两者都认不出，拒（可用性代价）。

## 四、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

对象：K6 的两段打中（丙、三次挂载）。

1. **分不分辨臂**：分辨。两段上 today、K3、K6 放过·红；指针槽读法（两种实现）、一律拒、K6 去掉继承自 K3 那一句只留映射根那一句（第一轮正推的变异 `k6_only_mapping_root`，`research/prompts/c834-r1-sonnet-model/patch/mount-c834-variants.patch:126` 「+        "k6_only_mapping_root" => root.mapping_root.locations.iter().any(|location| {」，下表按记下的凭据推出结局）都拒。它把 K6 与指针槽读法分开。
2. **被判的系统当时看不看得到判别它的东西**：看得到。要分的是「R 的单元此刻还在不在」。K6 自己读的两样里，映射根槽在两边不同：该放过的 `urJ` 一类读出 `idx:2`，这两段读回全零（第三节的表）；带更大号的记录在两边都有（丙里是 M1 那条，三次挂载里是 M2 那条；`urJ` 里是 M1 那条）。K6 用「或」把这两样并起来，看得到的判别子被继承自 K3 的那一句盖过去了。只看记录那一句（即 K3）才是「判别子观测不到」（第一轮判决已这样判 K3）；K6 的病在合法的组合方式，不在观测。
3. **满足的是判据字面的哪一个分句**：正文跑前条款第一行（`research/prompts/_c834-r2-body.md:44` 「| 一个候选被造出判错的历史（写掉还在用的单元，或该做成的拒且那一形是入库用例认的合法态） ⇒ 那个候选出局 | 一段可复现的历史，两序之一判错 |」）的前一个分句「写掉还在用的单元」：R 在根环里（`r_in_ring=true`）、按实例表有效（那一判只看没被抛弃的根，`crates/singlefs-core/src/mount.rs:2049` 「!is_abandoned(root)」）、被判那次挂载之前 R 的单元一份不少且池级 checker 不红；放过之后写行根落盘前新写掉 14 份，I-7.4（`crates/singlefs-checker/invariants/02-根环.md:11` 那一行，太长不抄，指过去）与 I-7.2（`crates/singlefs-checker/invariants/02-根环.md:9` 「| I-7.2 | 最新根可完整遍历 | S 中代号最大者，其 checkpoint 树根指针出发按 D4（校验和位置） 递归校验，能走完全部权威态（[decisions.md](../../../.claude/kb/decisions.md) D21（权威态与派生态的分界） 定义：单元 + 记账），无一处失配 | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::check_pool_image`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；崩溃点重放每个崩溃状态都判） |」）红；I-2.1、I-4.8 同一刻一起报，没逐条核它们的判据，只当旁证。两序都判错（条款只要两序之一）。不是可用性那一分句。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：条款对单个候选只给「出局」，没给改法；下一节表里列的几个改法在这两段上逐个核了：去掉继承自 K3 那一句（只留映射根槽）不中，指针槽读法不中，一律拒不中，「记录只认建在 R 上的」（推的）不中。

## 五、改法与各修哪一格

自提的改法只在攻方模型上量过、被攻过零轮。「量过」= 本轮副本上的原样输出（`out/` 里的文件）；「推的」= 按代码推、没实现没跑。K6 只留映射根那一句一列的「量过」是 `tools/derive.py` 按每个（状态，次序）上记下的 `mapping_root_newer` 推出拒或放过、再取同一状态上走那一支的候选的结局（`out/derived.txt`；那一判之前路径对每个候选相同，第一节装置说明）。

| 改法 | 丙 | 三次挂载（藏 M0 那条） | 甲乙戊 | 丁 | 九段 | 丁只藏记录（形二） | 丁加 y2（形三） | 入库 13 份 |
|---|---|---|---|---|---|---|---|---|
| K6 原样（第一轮正推） | **放过·红**（量过） | **放过·红**（量过） | 拒（量过） | 放过（量过） | 全对（量过） | 放过、多写掉 R 剩下的（量过） | 拒（量过） | 142 过（量过） |
| K6 只留映射根槽那一句（第一轮正推的变异） | 拒（量过，推出） | 拒（量过，推出） | 拒（量过，推出） | 放过（量过，推出） | later_mount 一段拒、其余同 K6（第一轮正推量过，`research/prompts/c834-r1-sonnet-output.md:170` 「| 删根与记录那一句，只留映射根（`k6_only_mapping_root`） | 补丁 `:126` | 红，2 格（later_mount 两序） |」） | 放过（量过，推出） | 拒（量过，推出） | 没跑 |
| 指针槽读法，索引节点按 16 KiB 解（`pointer_per_kind`） | 拒（量过） | 拒（量过） | 拒（量过） | 放过（量过） | 全对，later_mount 拒（量过） | 放过、多写掉（量过） | 拒（量过） | 142 过（量过） |
| K6 的记录那一句只认「所属发布建在 R 上」的记录（记录里能读出它那次发布的上一版是谁才行） | 拒（推的：M1 那条建在 P 上） | 拒（推的：M2 那条建在 P 上） | 拒（推的） | 放过（推的：映射根那一句） | later_mount 放过（推的：M0 那条建在 R 上） | 放过（推的） | 拒（推的） | 没跑 |
| 一律拒 | 拒（量过） | 拒（量过） | 拒（量过） | **拒**（量过） | units_only、urj、urj_row_record_also_hidden、later_mount 拒（第一轮量过） | 拒（量过） | 拒（量过） | `urJ` 那一条红（第一轮量过） |

K6 只留映射根槽那一句的推出原样（`out/derived.txt` 里与打中两段相关的四行）：

```
name=c834_derived history=three_mount_tail/y2_m0_record mapping_root_only:refuse:r_units_intact_before=True:order=forward states=4
name=c834_derived history=three_mount_tail/y2_m0_record mapping_root_only:refuse:r_units_intact_before=True:order=reversed states=4
name=c834_derived history=tree_table_and_record/tree_and_mapping mapping_root_only:refuse:r_units_intact_before=True:order=forward states=4
name=c834_derived history=tree_table_and_record/tree_and_mapping mapping_root_only:refuse:r_units_intact_before=True:order=reversed states=4
```

几点：

- 「K6 只留映射根槽」与「指针槽读法」在本轮走到那一判的全部（状态，次序）上逐行同判（`mapping_root_newer` 与 `pointer_per_kind` 两个凭据没有一行不同），九段上也同判；前者是后者的一半（只读映射根两槽、只认索引节点）。两者共同的软处是同一件：读故障藏住指针槽时一律拒（形三那一类可用性代价），与位翻转、撕裂写这类本仓读故障模型之外的坏字节（没造）。
- 「记录只认建在 R 上的」要记录里读得出它那次发布建在哪一版上；这一轮没查记录格式里有没有这个字段，没实现。
- 形二那一格（放过而多写掉 R 已坏之后剩下的单元）是 K6、指针槽读法、只留映射根槽三个改法共有的，要不要算错交主 agent（第三节形二）。

## 六、没打中的形状与取样范围

全部在 384 槽小盘、两块盘、R 是一次覆盖写（一条记录）、R 之前覆盖写 5 次这一档上；每段历史的崩溃状态全量枚举、两种交盘次序：

| 历史 / 被判那次挂载的读故障 | 状态数 | 对 K6 | 对指针槽读法（16 KiB） |
|---|---|---|---|
| `tree_table_and_record/tree_and_mapping`（丙） | 20516 | 打中（3 个） | 没打中 |
| `root_units_and_record/none`（丁） | 20516 | 没打中 | 没打中（第一轮原型原样全拒 4 个，缺陷） |
| `clean/y2`（甲乙） | 20511 | 没打中 | 没打中 |
| `tree_table_and_record/y2`（甲乙） | 20516 | 没打中 | 没打中 |
| `acquisition_rolled_back/y2`（戊） | 17 | 没打中 | 没打中 |
| `root_units_and_record/y2`（形三，新） | 20516 | 没打中（拒 16388 个已坏的，可用性代价） | 同左 |
| `root_units_and_record/record_only`（形二，新） | 20516 | 放过多写掉已坏 R 的剩余单元 2796 个，交主 agent | 同左 |
| `three_mount_tail/y2`（新） | 8227 | 没打中 | 没打中 |
| `three_mount_tail/y2_m0_record`（新） | 8227 | 打中（3 个） | 没打中 |
| 第一轮正推九段（固定盘面，不枚举） | 9 | 没打中 | 没打中（第一轮原型原样三段误拒，缺陷） |

合计 139562 个状态（另有没跑全的 `three_mount/y2_m0_record` 31000 个，不进结论）。每段历史跑之前用 `crash_state_count_with_torn_in_place_overwrites` 算过全量（上表状态数列，原型里也断言了不超过约 10⁶）；合起来远低于约 10⁷。

试过、没造出来的：

- 指针槽里是实例代号比 R 大的单元而 R 的单元一份不少的状态：上面 13 类读法里一行都没有（第三节表）。
- 映射根槽被数据单元或打包记录单元占住的状态：没出现（第三节形五）。
- 读故障之外的坏字节（位翻转、撕裂写落在指针槽上）：本仓读故障模型只有「读回全零」，没造。

## 七、这条腿自己的限度

- 全部是副本上的数：攻方补丁只改那一判与调用处（`today` 与冻结副本同判，多出的只是只读的读盘与一行记录）；原型是攻方写的。要引进判决，得在入库装置上重做（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。
- 「先照 today 跑、再强制走另一支」的办法：靠的是「那一判之前的路径对每个候选逐字相同、之后只看拒或放过」。本轮每个走到那一判的（状态，次序）那一判都只走到一次（`creds=many` 零行），九段那一份是每个候选直接跑的，与这个办法的结论一致（九段 K6、K3、today 与第一轮正推的表逐格相同）。
- 三次挂载缩短了历史：M2 的取号写与写行发布 18 次单元写并进基镜像，只枚举其后的 39 次写（8227 个状态）；缩前 270378 个状态。缩的理由是挂钟：开跑前小段估时（`three_mount` 序号 0–40 挂钟 4.80 秒、270300–270378 挂钟 6.27 秒，约 0.08 秒一个状态），当时本机负载 60 以上，按第一批 31 件 × 600 个状态 137.12 秒估，全量约 33 分钟，连同其余几段超过 40 分钟。缩掉的那 262151 个状态 M2 的记录都还没落、被判那次挂载在 `y2_m0_record` 下读不出任何更大号的记录（推的）；之后负载降下来时补跑了其中 31000 个，K6 与指针槽读法全拒，与推的相符。
- M3 的读故障比 Y2 那三段多一处（M0 那条记录）。不多这一处时（`y2`）K6 放过而没写掉。这一形要两次挂载之间读故障变化，与第一轮丙丁、入库 `ur`、`urJ`、Y2 同一个前提。
- 用户动作没放开扫：被判那次挂载的打中发生在它写行那次发布的根落盘之前，在任何用户动作之前；三次挂载里 M0、M2 都是挂载本身，不含用户动作。M0 择 R、M2 放过之后的写都是挂载自己定的。
- 状态分类里「已被写掉」按 R 写出的 14 份单元逐字节比；R 的单元被写掉之后还算不算「还在用」，本报告照第一轮判决对丁的读法写，形二那一格交主 agent。
- I-2.1、I-4.8 两条红没逐条核判据，只当旁证；论证只靠 I-7.4、I-7.2。
- 没把 `tools/rerun.sh` 从头跑一遍（子 agent 前台命令不超过 4 分钟，全量要分 9 批以上）；只核了 `nine` 重跑逐字节相同、`three_mount_tail` 前 8 个状态重跑逐行相同；汇总是确定的（例子序号取最小）。

### 取样、估时与分批

- 起跑前 `ps -o pid,args -u "$(id -u)"`：别的会话有两条 checker 档 `cargo test --release -p singlefs-checker-tier …`、一条 `checker-tier-crash-point-replay.sh --full`、一条 harness 档 `cargo test`、另一条腿的原型件在跑，没有性能测量（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`），加 `nice -n 19` 照跑；没等锁。
- 分批（每批不带限时，`.rc` 文件数与件数逐批对上，退出码全 0）：甲至戊与形三一张件表 176 件 × 至多 600 个状态，31 件一批跑了 4 批（挂钟 137.12、186.46、156.81、222.27 秒），剩下 52 件 18 件一批跑了 3 批（163.49、186.91、145.87 秒）；三次挂载 28 件一批（64.80 秒）；形二 30 件 × 至多 700 个状态一批（114.51 秒）；没跑全的 `three_mount` 31 件 × 1000 个状态一批（235.25 秒）。两次前台命令超过 240 秒被工具挪到后台（两批连跑的那一次、入库用例三个候选连跑的那一次），等完成通知、没写轮询。
- 入库用例：13 份在副本上编一次、四个候选各跑一遍，经 `run-with-memory-cap.sh 24G` 与 `capped.sh 31`。

## 八、没做什么

- 没在入库装置上跑；没跑 checker 档测试、门禁阶段、轻型三条，没起 GPU。
- R4（辩方那一格）不归攻方，没做。
- 没造多层映射树、没换几何与盘数、没试 R 是三条记录的顺序写、没试别的 k。
- 没实现「记录只认建在 R 上的」那一改法，没查记录格式里有没有「上一版」字段。
- 第一轮正推的另两份入库用例（`random_histories.rs`、`acquisition_writes_the_witnessed_journal_tail_so_two_crashed_acquisitions_still_witness_the_newest_publish.rs`）没在候选下跑。
- 原型只跑自己造的流；搭池用入库 harness 的 `tests/common_unwitnessed_publish/mod.rs`（它引 `tests/common_admission_pool/mod.rs`，后者被登记的崩溃枚举用例 `crash_enumeration_floor_raise_pushed_by_admission.rs` 引着），没有拷崩溃枚举用例；这一点与第一轮攻方同样写明给主 agent 判。

## 九、草稿与产物去向

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/c834-r2-opus/`：冻结副本拷贝（`tree/`、`tree-candidates/`）与编译目录（`target/`、`target-tests/`）交回前删掉；留下 `runs/`（逐状态原样输出约 272M、`slots-tally.txt`、`agg-threefull-partial.txt` 与入库用例日志，原样行太大不入库，模型目录 `out/` 只收汇总与摘录）、`model/`（与模型目录同一份）、几份编译日志。
- 入库的产物只有本报告与模型目录 `research/prompts/c834-r2-opus-model/`。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c834-r2-opus-output.md --root /home/fy5090/code/singlefs` 末行（没判的 3 处：两处引文里套着「」、脚本认不出，第 226、107 行逐字在原文件里 `grep -nF` 命中过；一处是 I-7.4 那一行只指路不抄）：

```
  ✓ 核了 12 处引文、3 处裸行号，对上 12 处，没判 3 处（其中裸行号没有引文可比 3 处；逐处列在上面）
```
