# C834（择到的系统配置实例代号随盘序变） 第三轮攻方腿（Opus）报告（2026-10-15）

只攻指针槽读法（下称 `pointer_per_kind`）与映射根变体（下称 `mapping_root_only`），格 T1、T2（正文「三、格」）。以下的数全是**副本上的数**：冻结副本 `~/.local/share/singlefs/work/c834-r1/repo/` 的拷贝，装上攻方候选开关（只改 `crates/singlefs-core/src/mount.rs` 那一判与调用处），加攻方原型，不是入库装置上的数。

## 复跑命令与模型文件

模型目录 `research/prompts/c834-r3-opus-model/`（由第二轮攻方模型改来）。复跑（整条经内存包装；本轮负载 75–105 时编译约 10 分钟（三次续编的合计，粗估）、前 31 件 1543.73 秒、其余 31 件约 436 秒）：

```
bash scripts/hosts/run-with-memory-cap.sh 24G bash research/prompts/c834-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/c834-r1/repo \
  --work <新的草稿目录>
```

末行判结果：`name=rerun_compare all_same`（`out/` 四份排序后逐行相同：`geometry-survey.txt`、`urj-image.txt`、`agg-before-row-root.txt`、`excerpts.txt`）。汇总里例子序号取那一格最小的序号，与件的先后无关，不用去字段。这条腿自己没有把 `rerun.sh` 从头跑一遍（见「这条腿自己的限度」）。

| 文件 | sha256 |
|---|---|
| `.cargo/config.toml` | 891b14e98b472f513f13dea65ff5b2a7cf1429b97a29b2c488f06e90ee095fc5 |
| `Cargo.lock` | 52e7aab4ccae57fe5587c81ef7acaeeeca9b8181a18ee8b81956ad2e22109c3c |
| `.cargo/mutants.toml` | 6ea6e326095098250e41ab4ce358d280f9bbb5f0351c19a2981a69630e12d881 |
| `Cargo.toml` | 014ec33e81dc9ded6cc6dfc9643f6fec05304fd0e427e73cf983aa797c4ebe53 |
| `out/agg-before-row-root.txt` | ed4283d81baaa87f62c6655aa9a8b3f5cb94002906c68292edbdfa9c22cdb069 |
| `out/excerpts.txt` | 9076b91c763c9f06202dd59fb15239854d5c944e232895e9878e6548ea758154 |
| `out/geometry-survey.txt` | 3e0ace129a28f0733c738b3ec2c82a121effc87ecfd45d01b0788a0c748c871e |
| `out/urj-image.txt` | 9e5e7c064eb02c63966d1bab9e3507f51d34f95e4481ea190e2b0606ffdbd192 |
| `patch/apply_candidates.py` | e8809da3c025e7af7b5e60227805c3373a88615e9ac4c1ef8df2fe845f219354 |
| `src/main.rs` | de3ce2757a33328de339204bd34607441cbe355aff55727324465eca144a8b17 |
| `src/prefix.rs` | fcb76beda8219f6d435830aecf441d564cde27333b103085003bc1c0b1c8299b |
| `tools/aggregate.py` | 5d9cfef0dd5384d64369db47c53357f338b08bb0db9d2c80729c92c2525afa54 |
| `tools/faultsets.txt` | 8dc4a023ede24d5cfffacf81b62a5dcf3593f745d4ffe2347ffe651a78c0fd89 |
| `tools/pieces-o5-first.txt` | d48613237716d9bc76f56732f39e14522130f2ca25001c964e8f2fc3f060a2b2 |
| `tools/pieces-rest.txt` | 973e0288e20b39d225c719436b51647df86c20f1a4c35d4c529c446935818585 |
| `tools/rerun.sh` | af62dd74d54a4a79afa58f137a190157c8262e938044554afec8338e5283d65a |
| `tools/run-batch.sh` | f720f38ca20c72eabf50c62b5afc3a56c1989f64570dae9fbadebbf9ab48d6f9 |
| `tools/survey-batch.sh` | efddd2cf07bde164d970148968258f5300e29d503f3d0c2102e1f4a09788ff55 |
| `tools/survey-numeric.txt` | 8a28537f7db0c4849f5778b45d2c7dccfe918345e6bfe3e2841d46d91f368540 |
| `tools/survey-prefixes.txt` | 040b748a1b86ba6d38a2afc869ffa2d1d85ee346b6533423bf068ba0cb8c2100 |
| `tools/survey-summary.py` | 1d3135c75693b4cc871663a20334ba8b702461eec83a96568f932fdef87c5d70 |

`.cargo/` 两份是冻结副本 `.cargo/` 原样（随第二轮模型拷来）；`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段，第 1 到 73 行与「# 自证：模板拷进」起的尾段与模板逐字相同（`diff` 量过）。

装置是什么：

- **补丁** `patch/apply_candidates.py`：第二轮攻方补丁加三样凭据。`mapping_root_only` 逐字同第二轮辩方 `mapping_root_only`（映射根两个落点各读 16 KiB、只认解得开的索引节点、实例代号比 R 大）；`pointer_per_kind` 原样（四个落点，索引节点按 16 KiB、数据单元与打包记录单元按 32 KiB 解，有一个比 R 新就放过）；攻方自提的 `pointer_pair` / `mapping_pair`（零轮，第四节）。诊断 `slots` 照旧。
- **原型** `src/main.rs`、`src/prefix.rs`：入库搭建 `common_unwitnessed_publish`（`#[path]` 引进）录 R，崩在 R 的根槽之后、轮换之前；M1 是入库 `urJ` 那一形的可写挂载（R 的根槽、R 写出的全部单元、R 的全部记录读回全零，M1 看不见 R、退回 P、取号 2、写行）。M1 的写流按崩溃点重放的枚举域（`crash_state_enumeration_tables` + `extract_flow_facts`，每段 `full_expansion`，原地覆写取三态）逐个崩溃状态展开，每个状态跑池级 checker，再做被判的那次挂载 M2，两种交盘次序。M2 的读故障按 R 的**每一份**分开点：`tt0`/`tt1`（R 的树表在盘表第一、第二条位置条目上那一份）、`mr0`/`mr1`（映射根同上）、`rec`（R 的全部记录，两块盘），`none` 不藏。判法同第二轮：先照 today 跑，走到那一判就按记下的凭据算每个候选拒还是放过、再强制走另一支跑一遍。
- **换几何**：`prefix.rs` 把 R 之前的前缀从「覆盖写 k 次」换成一串用户动作（`o` 覆盖写、`i` 建一个 inode、`w` 顺序写两个数据单元），录法逐行照入库 `record_one_publish_after_overwrites`；`survey` 模式只看 M1 写行根之前全落时 R 的四个指针槽上是什么。
- **R 此前的样子**（汇总里的 `before=`）：`intact` R 写的单元一份不少；`degraded_walkable` 有几份被 M1 盖了、**每个单元还剩一份**、池级 checker 不红 I-7.2；`gone` 有单元两份都没了。结局 `mounted_overwrote:gone=N:red=…` 是 M2 写行根落盘之前那一刻两份都没了的 R 单元数与那一刻红的不变量。

## 各格判定一览

| 格 | 判定 | 量过 / 推的 |
|---|---|---|
| T1 两份副本只盖掉一份（新形，第二轮没有这一格，零轮） | **两个变体都打中**：M1 崩在单元写那一段里、只落了一块盘上那几份，R 每个单元还剩一份、I-7.2 不红；M2 时 R 的另一份树表、另一份映射根、R 的记录读回全零（Y2 那六段的子集）。`tt1+mr1+rec`：两个变体都放过，324 个状态两序都写掉剩下那一份、I-7.2 转红；`tt1+mr0+mr1+rec`：`pointer_per_kind` 放过、写掉，972 个状态两序，`mapping_root_only` 拒 | 量过（副本，全量枚举到 M1 写行根之前） |
| T1 指针槽上落着打包记录单元（整个） | 造出：前缀 `ooww` 等 7 种几何下 M1 的打包记录单元（实例表，`packed:2`）落在 R 的树表槽上、映射根还是 R 自己的。读故障 `none` 走不到那一判（映射现算得出）；`mr0+mr1+rec` 下 4104 个状态 `pointer_per_kind` 放过（R 的树表已两份都没了，不丢可读的东西）、`mapping_root_only` 拒（可用性代价） | 量过 |
| T1 指针槽上落着打包记录单元的后半截 | 造出：前缀 `oiww` 等 3 种几何（R 是三条记录的顺序写）下 M1 的打包记录单元从 R 映射根的前一槽起、后半截盖住映射根槽，那 16 KiB **读回全零**，与读故障藏住逐字节相同。`rec` 下 1032 个状态 `pointer_per_kind` 放过（树表槽 `idx:2`）、`mapping_root_only` 拒；R 已两份没了 | 量过 |
| T1 R 有多条记录 | 造出：R 是三条记录的顺序写时 M1 的写行记录只盖掉 R 的第 1 条，第 2、3 条读得出；`urJ` 盘面上 M2 在读故障 `none` 下**走不到那一判**，全部候选同拒在「记录有缺口」那一支（`PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify`）。两个变体不起作用 | 量过（写行根之前全落那一个盘面；全量只跑了 `rec`） |
| T1 多层中央映射树 | 没造（推的）：一片映射叶装 294 条，384 槽小盘装不下 295 个进映射的单元；换大盘之后 M1 落在新的全空聚簇段、不碰 R 的单元，走不到那一判，要造得同时调到「大盘装满」，本轮没做 | 推的 |
| T1 撕裂写、位翻转落在指针槽上 | 没造（推的）：三类单元的解析都核头校验和与整单元载荷 CRC，撕裂或翻转的槽解不出、不会被当成「更新的单元」，只会把两个变体推向拒；单元写在崩溃点重放的枚举域里本来不取撕裂态 | 推的（读码） |
| T2 分得开两个变体的历史 | **造出，`pointer_per_kind` 判错**：序号 1032（M1 只落了盘 0 上盖 R 树表那一写），M2 读故障 `tt1+mr0+mr1+rec`，`pointer_per_kind` 放过、写掉 R 剩下的那一份、I-7.2 转红，`mapping_root_only` 拒（对）。另有反方向分得开而判错只在可用性一边的几形（`mapping_root_only` 拒、R 已两份没了） | 量过 |
| 跑前条款 | 第一行「写掉还在用的单元」那一分句：`pointer_per_kind` 一段（T2）、两个变体同一段（`tt1+mr1+rec`）各两序判错 ⇒ 两个都出局，前提是主 agent 认「每个单元还剩一份、I-7.2 不红」的 R 是还在用的（第三节第 3 句） | 量过 + 交主 agent 判 |

一句话：两个变体读的是「某个指针槽上有一份比 R 新的单元」，它推不出「R 的单元已被写掉」：池里每个单元两块盘各一份，一次没走完的写行发布可以只盖掉一块盘上的那一份，R 照样走得完；这时另一份被读故障藏住，两个变体凭盖掉的那一份放过，被判的那次挂载按同一套落点把剩下那一份也写掉。

## 一、T1：第二轮没造出的形

### 1. 换几何：M1 的写落到 R 的指针槽上的是什么

`survey` 模式在 26 种「覆盖写 k 次」（k = 0–10、12、14，R 是覆盖写或三条记录的顺序写）与 206 种前缀（`o`/`i`/`w` 按个数排，总长 1–9）上各录一次 R、做一次 M1、看 M1 写行根之前全落时 R 的四个指针槽（`out/geometry-survey.txt`，232 行）。按四个槽读出什么分（同一种的几何数）：

| M1 盖掉 R 的单元 | tt0 / tt1 | mr0 / mr1 | 几何数 |
|---|---|---|---|
| 一份没盖 | `idx:1`（R 自己的） | `idx:1` | 200（含 176 种前缀与 24 种 k） |
| 盖了别的单元 | `idx:1` | `idx:1` | 5 |
| 盖了映射根 | `idx:1` | `idx:2` | 11（含三条记录、k=5 那一种） |
| 全盖 | `idx:2` | `idx:2` | 6（含入库那一档：覆盖写、k=5） |
| 树表被打包记录单元盖住 | `packed:2` | `idx:1` | 7 |
| 映射根被打包记录单元的后半截盖住 | `idx:2` | `zero` | 3 |

（几何数按 `out/geometry-survey.txt` 里 `r_copies_covered_by_m1` 是否为 0 与 `tt0=`、`mr0=` 两段归类，用 python 数的，合计 232；表里的 k 那几行来自 `runs/survey`，前缀那几行来自 `runs/prefix`，两份都汇进同一个文件。）

M1 落点的规律（读码 + 上表）：M1 取号之后的写行发布按「开放段 = 单元区内最低的 64 槽对齐全空段」放提交内生块（`crates/singlefs-core/src/allocator.rs:5` 「//! 提交内生块（D3（空间分配） 已定项 5 / 已定项 10）：从开放聚簇段 bump，开放段 = 单元区内最低的 64 槽对齐全空段，bump 只在内存；」）。M1 看不见 R，R 写出的槽在它眼里是空的；只有 R 的单元跨进第 1536 槽起的那一段时（R 的那一段在 M1 眼里全空），M1 才从 1536 起往上写、盖到 R。第一件是 32 KiB 的打包记录单元（实例表）占 1536–1537，之后是 16 KiB 的索引节点。所以 R 的指针槽落在 1536 上就被打包记录单元的头盖住，落在 1537 上就被它的后半截盖住，后半截是零填充，读回全零。

### 2. 指针槽上是打包记录单元（整个）：前缀 `ooww`

历史：起池、第一个文件，覆盖写两次、顺序写两个数据单元两次（`ooww`），再录 R（覆盖写，一条记录）；M1 是 `urJ` 那一形，取号 2、写行，M1 的打包记录单元落在 1536–1537，R 的树表就在 1536（两份），映射根在 1535 没被盖（`out/geometry-survey.txt` 的 `case=overwrite-ooww` 那一行：`tt0=1536:packed:2 … mr0=1535:idx:1`）。许可每一步的条款与第二轮相同：M1 那一形是入库 `urJ` 用例（`crates/singlefs-harness/tests/recovery_and_both_mounts_fall_back_to_the_version_the_system_configuration_witnessed.rs:200` 「fn a_root_whose_units_and_record_read_back_zero_at_the_mount_falls_back_to_the_witnessed_version_in_recovery_and_both_mounts(」）只把前缀换掉；崩在 M1 写行根之前是崩溃点重放枚举域里的状态。

M1 写流 24623 个崩溃状态（`crash_state_count_with_torn_in_place_overwrites`），本轮只枚举写行根落盘之前那 16400 个（序号 0–16399，见「这条腿自己的限度」）。

- 读故障 `none`：走不到那一判（写行根之前全落那一个盘面上 `not_reached`，`out/urj-image.txt`）：R 的映射根两份都在，映射现算得出。R 的树表两份都没了，R 已走不完，挂载照常做成、不再写掉 R 的单元。
- 读故障 `mr0+mr1+rec`（M2 时 R 的映射根两份、R 的记录读回全零），全量 16400 个状态，走到那一判的 4104 个（`out/agg-before-row-root.txt` 原样）：

```
name=c834_cell history=urj_overwrite_pooww/mr0+mr1+rec before=gone state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=mapping_root_only order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4104 example_ordinal=11
name=c834_cell history=urj_overwrite_pooww/mr0+mr1+rec before=gone state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=mapping_root_only order=reversed judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=4104 example_ordinal=11
name=c834_cell history=urj_overwrite_pooww/mr0+mr1+rec before=gone state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=forward judge=pass outcome=mounted_r_kept states=4104 example_ordinal=11
name=c834_cell history=urj_overwrite_pooww/mr0+mr1+rec before=gone state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=reversed judge=pass outcome=mounted_r_kept states=4104 example_ordinal=11
```

结局：两个变体在这 4104 个状态上分得开。R 此前已两份没了树表（I-7.2 红），`pointer_per_kind` 放过、写行根落盘前没再写掉 R 的单元，没写掉还在用的东西；`mapping_root_only` 拒（只认索引节点，树表槽上的打包记录单元它不看），是可用性代价。这一形要 M2 时映射根两份再加一处读故障，入库用例没有，按跑前条款字面不算「入库用例认的合法态」。

### 3. 指针槽上是打包记录单元的后半截：前缀 `oiww`、R 是三条记录

历史：覆盖写一次、建一个 inode、顺序写两个数据单元两次（`oiww`），再录 R（顺序写三个数据单元，三条记录）；M1 同上。M1 的打包记录单元占 1536–1537，R 的映射根在 1537，被它的后半截盖住；后半截是零填充，那 16 KiB 读回全零；R 的树表在 1538，被 M1 的索引节点盖住（`case=seq3-oiww` 那一行：`tt0=1538:idx:2 … mr0=1537:zero`）。这是「指针槽上是全零而读故障一处都没有」的一形：读法看到的东西与读故障藏住 R 自己那一份逐字节相同。

- 读故障 `none`：写行根之前全落那一个盘面上**走不到那一判**，每个候选同拒（下一小节「R 有多条记录」）。
- 读故障 `rec`（R 的三条记录读回全零），全量 16400 个状态，走到那一判的 1032 个（1027 + 5，`out/agg-before-row-root.txt`）：`pointer_per_kind` 放过、`mounted_r_kept`；`mapping_root_only` 拒。原样（两类各取正向一行，反向同数）：

```
name=c834_cell history=urj_seq3_poiww/rec before=gone state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=mapping_root_only order=forward judge=refuse outcome=refused:AccountOfAnUnappliedPublishWithIncompleteReadableRecords states=1027 example_ordinal=23
name=c834_cell history=urj_seq3_poiww/rec before=gone state_red=I-2.1+I-4.8+I-7.2+I-7.4 candidate=pointer_per_kind order=forward judge=pass outcome=mounted_r_kept states=1027 example_ordinal=23
```

结局同上一形：R 已两份没了，`pointer_per_kind` 放过不丢东西，`mapping_root_only` 拒是可用性代价；读故障是「R 的记录在 M1 时就读回全零、到 M2 还没好」，与 M1 那一形同一批段。

### 4. R 有多条记录

R 是三条记录的顺序写（入库搭建 `SequentialWriteOfThreeDataUnits`）时，M1 看不见 R、它的写行记录按 P 之后的计数器落，只盖掉 R 的第 1 条；R 的第 2、3 条（带末条标志的那一条在里面）读得出。M2 在读故障 `none` 下拿到的是「水位之上一次没施加的发布，读得出的记录缺序号 1」，走记录有缺口那一支，不到那一判。原样（`out/urj-image.txt`，k=5 那一档，写行根之前全落）：

```
name=c834_state history=urj_seq3_k5 faults=none ordinal=16399 segment=3 later_records=[2] r_in_ring=true r_copies_overwritten=14 r_units_gone=7 on_disk=mr0:idx:2,mr1:idx:2,tt0:idx:1,tt1:idx:1 state_red=I-2.1+I-4.8+I-7.2+I-7.4+I-8.6+I-8.9 order=forward creds=not_reached cands=all:-:refused:NewerStateStillUnreadableAfterOneReread:PublishAboveTheWaterMarkMissingRecordsWhoseNamedUnitsDoNotVerify:writes=0
```

`oiww` 那一档同一个拒因（`urj_seq3_poiww faults=none` 那两行）。两个变体在这里不起作用：拒的是相邻那一支。**零轮**：入库 `urJ` 用例的 R 是一条记录，R 换成三条记录之后同一段历史（M1 看不见 R、写掉 R 的单元，之后盘不变）每次可写挂载都拒，直到 R 那两条记录被环盖掉；这是「记录有缺口时拒可写」那一支的可用性代价，不归 C834 这两个变体，交主 agent 定挂不挂进未定项清单。R 有多条记录时要走到那一判，得 M2 时 R 的记录全读不出（`rec`），那时的结局见上一小节与第二节。

### 5. 多层中央映射树：没造（推的）

一片映射叶装几条：`crates/singlefs-core/src/unit.rs:151` 「pub fn index_node_entry_capacity(key_width: usize, entry_width: usize) -> usize {」按 (16384 − 头 − 预留) / 条目宽算；映射 key 27 字节、条目 55 字节（`crates/singlefs-format/src/lib.rs:95` 「pub const MAPPING_ENTRY_BYTES: u64 = 55;」），头 86 + 2×27 = 140、预留 29，(16384 − 169) / 55 = 294（推的算术，没跑这个函数）。映射树要长到两层，得有 295 个以上进映射的单元；384 槽小盘单元区（环窗口补足那 1007 槽常驻扣着）装不下（数据单元 2 槽一个）。换大盘：M1 的提交内生块落在最低的全空聚簇段，大盘上总有全空段，M1 不碰 R 的单元，R 的账读得出，走不到那一判（第 1 小节的规律）；要造「映射根读得出、下层节点被盖」得同时把大盘几乎装满、让 R 的下层节点跨进 M1 的开放段，本轮没做。推的结局：映射根读得出而下层节点读不出时，映射根两槽是 R 自己的（实例代号 = R），两个变体都只能靠树表槽，`mapping_root_only` 一律拒；R 的下层节点已被盖时 R 已走不完，拒是可用性代价，与第 3 小节同类。推翻条件：在一个两层映射树的几何上造出 M1 盖了 R 的下层节点、R 的指针槽一处没盖的状态，`pointer_per_kind` 照样放过而 R 此前走得完。

### 6. 撕裂写、位翻转落在指针槽上：没造（推的）

三类单元的解析都先判头校验和、再判整单元载荷 CRC（索引节点那一判 `crates/singlefs-core/src/unit.rs:401` 「check_payload_checksum(bytes, header_end, header_end - 10)?;」），撕裂或翻转过的槽解不出，在两个变体那里落成「不是更新的单元」，只会把它们推向拒，推不出错放。崩溃点重放的枚举域里单元写不取撕裂态（`crates/singlefs-checker-tier/src/crash.rs:2085` 「/// 1. 不是单元写：单元写是 COW，落在分配器交出来的槽上，那一槽的旧内容没有谁还要（回收谓词管着），撕裂与没持久一样，照旧两态；」）。**零轮**：这一条的前提「那一槽的旧内容没有谁还要」在 `urJ` 一族上不成立——M1 写的正是 R 还要的槽；撕裂态下 R 那一份既不是旧字节也不是新单元。按上面那一判，撕裂只让两个变体多拒，不改本报告的结论；它对别的判法、对 checker 的影响没查，交主 agent 定挂不挂进未定项清单。

## 二、T2：分得开两个变体的历史，且其中一个判错

### 1. `pointer_per_kind` 放过、写掉还走得完的 R；`mapping_root_only` 拒（序号 1032）

历史（覆盖写 k=5，入库 `urJ` 那一档几何）：

1. R=(1,9)（P=(1,8)）的根槽落了、系统配置轮换没落（入库搭建原样）。
2. M1：R 的根槽、R 写出的全部单元、R 的记录读回全零（入库 `urJ` 那一形），M1 退回 P、取号 2、写行。M1 写行发布的单元写在一段里（两块盘同槽、各 7 次），**崩在这一段里、只落了盘 0 上写 1542 槽那一次**：1542 是 R 的树表在盘 0 上那一份。崩溃点重放的枚举域把一段里的写逐个子集展开，这是其中一个状态（序号 1032，`segment=1`）。此刻 R 写出的 20 份里只有 1 份被盖（`r_copies_overwritten=1`、`r_units_gone=0`），池级 checker 红 I-2.1、I-4.8、I-7.4（那一份被复用），**不红 I-7.2**：R 走得完。
3. M2（被判的那次挂载）：R 在盘 1 上那一份树表、R 的映射根两份、R 的记录读回全零（`tt1+mr0+mr1+rec`，入库 Y2 用例那六段的子集；Y2 用例 `crates/singlefs-harness/tests/writable_mount_isolates_newer_readable_roots_and_mapping_recomputed_accounts_when_records_are_missing.rs:257` 「fn a_newer_readable_root_whose_only_record_mapping_root_and_tree_table_read_back_zero_refuses_the_writable_mount_before_any_write(」藏的是 R 的记录、映射根、树表各两份）。读故障在两次挂载之间会变，与第二轮三次挂载、入库 `ur` / `urJ` / Y2 同一个前提；这里 M2 的六处读故障里五处是 M1 时就藏着的段（M1 藏了 R 的全部单元与记录），只是 M1 之后好了一部分。R 的账读不出（树表盘 0 那一份是 M1 的索引节点，按 R 的位置条目校验和对不上，`crates/singlefs-core/src/recovery.rs:399` 「if crc32_castagnoli(&bytes) == location.unit_checksum {」；盘 1 那一份读回全零），记录一条都读不出，映射现算不成，走到那一判（`crates/singlefs-core/src/mount.rs:2078` 「None if journal_instance_in_the_system_configuration」）。

原样（`out/excerpts.txt` 第 3 行，正向；第 4 行反向，凭据与结局逐字相同）：

```
name=c834_state history=urj_overwrite_k5 faults=tt1+mr0+mr1+rec ordinal=1032 segment=1 later_records=[] r_in_ring=true r_copies_overwritten=1 r_units_gone=0 on_disk=mr0:idx:1,mr1:idx:1,tt0:idx:2,tt1:idx:1 state_red=I-2.1+I-4.8+I-7.4 order=forward creds=once;chosen:2,pointer_per_kind:true,mapping_root_only:false,pointer_pair:false,mapping_pair:false,slots:tt0:idx:2,tt1:zero,mr0:zero,mr1:zero cands=today:pass:mounted:eff=(1,8):inst=3:new=13:total=14:gone=7:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;pointer_per_kind:pass:mounted:eff=(1,8):inst=3:new=13:total=14:gone=7:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;mapping_root_only:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;pointer_pair:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;mapping_pair:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;always_refuse:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
```

读法：`on_disk=… tt0:idx:2,tt1:idx:1`：盘上 R 的树表盘 0 那一份是 M1 的、盘 1 那一份还是 R 的；`slots=tt0:idx:2,tt1:zero,mr0:zero,mr1:zero`：M2 读到的四个槽。`pointer_per_kind:true` → 放过 → M2 写行根落盘前新写掉 R 的 13 份、R 的 10 个单元里 7 个两份都没了，R 还在根环里，那一刻 I-7.2 红。`mapping_root_only:false` → 拒在任何写之前（`writes=0`）。全量（M1 写行根之前 16400 个状态，`tt1+mr0+mr1+rec`）：R 走得完的 972 个状态 `pointer_per_kind` 两序都放过且写掉（`states=648` 与 `states=324` 两行），`mapping_root_only` 两序都拒（`out/agg-before-row-root.txt` 里 `urj_overwrite_k5/tt1+mr0+mr1+rec before=degraded_walkable` 那几行）。

### 2. 两个变体都放过、写掉还走得完的 R（序号 1288）

同一段历史，M1 崩在同一段里、落了盘 0 上 1541（R 的映射根盘 0 那一份）与 1542 两次写；M2 读故障 `tt1+mr1+rec`（盘 1 上那份树表与映射根、R 的记录）。原样（`out/excerpts.txt` 第 1 行，正向；第 2 行反向同）：

```
name=c834_state history=urj_overwrite_k5 faults=tt1+mr1+rec ordinal=1288 segment=1 later_records=[] r_in_ring=true r_copies_overwritten=2 r_units_gone=0 on_disk=mr0:idx:2,mr1:idx:1,tt0:idx:2,tt1:idx:1 state_red=I-2.1+I-4.8+I-7.4 order=forward creds=once;chosen:2,pointer_per_kind:true,mapping_root_only:true,pointer_pair:false,mapping_pair:false,slots:tt0:idx:2,tt1:zero,mr0:idx:2,mr1:zero cands=today:pass:mounted:eff=(1,8):inst=3:new=12:total=14:gone=7:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;pointer_per_kind:pass:mounted:eff=(1,8):inst=3:new=12:total=14:gone=7:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;mapping_root_only:pass:mounted:eff=(1,8):inst=3:new=12:total=14:gone=7:r_in_ring=true:red=I-2.1+I-4.8+I-7.2+I-7.4;pointer_pair:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;mapping_pair:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0;always_refuse:refuse:refused:NewerStateStillUnreadableAfterOneReread:AccountOfAnUnappliedPublishWithIncompleteReadableRecords:writes=0
```

`mapping_root_only:true`（映射根盘 0 那一份是 M1 的 `idx:2`）、`pointer_per_kind:true` → 两个都放过 → 写掉 R 剩下的那一份、7 个单元两份都没了、I-7.2 红。全量：R 走得完的 324 个状态两序都这样（`urj_overwrite_k5/tt1+mr1+rec before=degraded_walkable` 那几行，`states=216` 与 `states=108`）。`pointer_pair`、`mapping_pair`、一律拒在这 324 个与上一小节 972 个状态上都拒。

### 3. 反方向分得开、只在可用性一边

`mapping_root_only` 拒而 `pointer_per_kind` 放过、R 此前已两份没了的几形：第一节第 2、3 小节（4104 个、1032 个状态），以及 `urj_overwrite_k5/tt1+mr0+mr1+rec before=gone`。这些格放过不丢可读的东西，拒是可用性代价；都要 M2 时多一处读故障（映射根或记录），入库用例没有这一形，按跑前条款字面不算判错。

## 三、打中之后的四句（`.claude/singlefs-ai-sop/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）

对象：第二节第 1、2 小节两段（序号 1032 一类 972 个状态、序号 1288 一类 324 个状态）。

1. **分不分辨臂**：第 1 小节分辨 `pointer_per_kind`（放过·写掉）与 `mapping_root_only`（拒）；第 2 小节不分辨这两个（同放过·写掉），两个变体共用的前提是「四个（或两个）槽里有一个读出比 R 新的单元 ⇒ R 的单元已被写掉」，today 的那一判注释写的是同一个前提（`crates/singlefs-core/src/mount.rs:2070` 「// 那条根之后已有一次挂载取过号、写掉了它的单元、记录槽也被那次挂载的记录占了（入库用例 `urJ` 那一形），映射本来就现算不成，」）。两小节都分辨这两个变体与 `pointer_pair`、`mapping_pair`、一律拒（后三者拒）。
2. **被判的系统当时看不看得到判别它的东西**：要分的是「R 的每个单元是不是还剩一份完整的」。M2 看到的是一份更新的单元加一份全零；全零是读故障藏住了 R 那一份，还是那一槽真的被写成全零（第一节第 3 小节：打包记录单元的后半截读回全零），从那一槽的字节上分不出——这一半属「判别子观测不到」。另一半看得到：同一个指针的**两份**都读出比 R 新的单元，那个单元两块盘都没了，R 一定走不完，放过不丢可读的东西。两个变体把四个槽「有一个」并起来，丢了这半个看得到的判别子。
3. **满足的是判据字面的哪一个分句**：正文跑前条款第一行（`research/prompts/_c834-r3-body.md:67` 「| 一个变体被造出判错的历史（写掉还在用的单元，或该做成的拒且那一形是入库用例认的合法态） ⇒ 那个变体出局 | 一段可复现的历史，两序之一判错 |」）的前一分句「写掉还在用的单元」：R 在根环里、那一判只看没被抛弃的根，M2 之前池级 checker 不红 I-7.2（`crates/singlefs-checker/invariants/02-根环.md:9` 那一行，太长不抄，指过去），走读两份都读不到才记失败（`crates/singlefs-checker/src/walk.rs:1600` 「.push(format!("{what} 两份都读不到对得上的"));」），放过之后写行根落盘前 7 个单元两份都没了、I-7.2 红。两序都判错。**要主 agent 判的一处**：M2 之前那个状态已红 I-2.1、I-4.8、I-7.4（M1 复用了 R 的一份），与第二轮判决对「丁只藏记录」那一形不算打中的理由（那一形放过之前已红 I-7.2、R 已走不完）不同：这里 R 走得完。认「每个单元还剩一份、走得完」的 R 是还在用的，两个变体都出局（`pointer_per_kind` 两段、`mapping_root_only` 一段）；不认，这两段不算打中，T2 只剩可用性一边的分法（第二节第 3 小节）。
4. **跑前条款给的每个改法在打中的那几格上还中不中**：条款对单个变体只给「出局」；第四节的改法在这几格上逐个核了。

## 四、改法与各修哪一格

攻方自提的 `pointer_pair`（树表或映射根两个指针里有一个，**它的两份**都读出自证过、实例代号比 R 大的单元，才放过）与 `mapping_pair`（只看映射根两份、只认索引节点），只在我的模型上量过、被攻过零轮。「量过」= 本轮副本上的原样输出（`out/agg-before-row-root.txt`、`out/urj-image.txt`）；「推的」= 按代码推、没实现或没跑。格里的数是走到那一判的状态数（两序同数）。

| 改法 | 1032 一类（972，R 走得完） | 1288 一类（324，R 走得完） | `urJ` 盘面 `none`（k=5，4 个走到） | `ooww` `mr0+mr1+rec`（4104，R 已没） | `oiww` `rec`（1032，R 已没） | k=5 `tt1+mr0+mr1+rec` 里 R 已没的（11964） | 入库 harness 用例 |
|---|---|---|---|---|---|---|---|
| `pointer_per_kind` | **放过·写掉**（量过） | **放过·写掉**（量过） | 放过（量过） | 放过（量过） | 放过（量过） | 放过 7228、拒 4736（量过） | 第二轮量过 142 过 |
| `mapping_root_only` | 拒（量过） | **放过·写掉**（量过） | 放过（量过） | 拒（量过） | 拒（量过） | 拒（量过） | 没跑 |
| `pointer_pair`（零轮） | 拒（量过） | 拒（量过） | 放过（量过） | 放过（量过） | 放过（量过） | 拒（量过） | 没跑；推的：`urJ` 那一条盘面四槽都是 `idx:2`，放过、转绿 |
| `mapping_pair`（零轮） | 拒（量过） | 拒（量过） | 放过（量过） | 拒（量过） | 拒（量过） | 拒（量过） | 没跑 |
| 一律拒 | 拒（量过） | 拒（量过） | **拒**（量过；`urJ` 用例红，第一轮量过） | 拒（量过） | 拒（量过） | 拒（量过） | `urJ` 那一条红（第一轮量过） |

几点：

- `pointer_pair` 放过 ⇒ 那个指针指的单元两块盘都没了 ⇒ R 走不完：这是推的（读码：R 的根记录直接指着树表与映射根，走读两份都读不到就失败），在本轮全部 R 走得完的状态上它都拒（量过）。它的代价是 R 已没、却有一份被读故障藏住的状态拒（k=5 那 11964 个里 `pointer_per_kind` 放过的 7228 个，`pointer_pair` 都拒；量过），与 `mapping_root_only` 在「打包记录单元落在树表槽」那一形上拒（`mapping_pair` 同拒）。
- 「两份都比 R 新」在 R 的两份落在同一块盘上时退化成一份；本仓池两块盘、同一个指针两条位置条目两盘同槽，没造单盘的形（推的）。
- 撕裂（若取撕裂态）落在一份上、另一份是更新单元时 `pointer_pair` 拒，R 那个单元两份都已没，是可用性代价（推的，没造）。

## 五、没打中的形状与取样范围

全部在 384 槽小盘、两块盘上；R 之前的前缀与 R 的种类见第一节第 1 小节；M1 一律是 `urJ` 那一形。全量枚举的（历史，M2 读故障）都只枚举 M1 写行根落盘之前那 16400 个状态（序号 0–16399，16399 是写行根之前全落、即入库 `urJ` 用例的盘面；写行根落了之后 R 被 M1 的实例表抛弃，那一判只看没被抛弃的根，走不到）：

| 历史 / M2 读故障 | 枚举的状态 | R 走得完的状态里两个变体 | 走到那一判、R 已没的状态 |
|---|---|---|---|
| k=5 / `none` | 16400（全） | 2912 个都走不到那一判 | 4 个，两个变体都放过、不再写掉 |
| k=5 / `tt1+mr0+mr1+rec` | 16400（全） | `pointer_per_kind` 972 放过·写掉；`mapping_root_only` 972 拒 | 11964 个，见第四节表 |
| k=5 / `tt1+mr1+rec` | 16400（全） | 两个都 324 放过·写掉 | 6148 个，两个都放过 6136、另 12 个放过而没再写掉 |
| `ooww` / `mr0+mr1+rec` | 16400（全） | 8192 个都走不到 | 4104，`pointer_per_kind` 放过、`mapping_root_only` 拒 |
| `oiww`（三条记录）/ `rec` | 16400（全） | 8192 个都走不到 | 1032，同上 |
| k=5 / 另 14 组读故障（`tt0`、`tt1`、`mr0`、`mr1` 的其余子集加 `rec`） | 3721（序号 0–3719 与 16399，部分取样） | `tt0+mr0+mr1+rec` 243 个 `pointer_per_kind` 放过·写掉、`mapping_root_only` 拒；`tt0+mr0+rec`、`tt0+mr1+rec`、`tt1+mr0+rec` 各 81 个、`tt0+tt1+mr0+rec`、`tt0+tt1+mr1+rec` 各 243 个两个都放过·写掉；`tt0+tt1+mr0+mr1+rec`（Y2 六段）728 个都拒（对）；只藏记录、只藏一处或只藏映射根两份的 8 组都走不到 | 部分取样，不进结论 |
| 写行根之前全落那一个盘面：k=5 三条记录 / `none`、`rec`、`tt1+mr1+rec`；`oiww` / 另三组；`ooww` / 另三组 | 1 个 | — | 见 `out/urj-image.txt` |

部分取样那一行的数（量过）只说明打中不止两组读故障，全量没跑，不当「没打中」用。

试过、没造出来的：

- 读故障 `none`（入库 `urJ` 那一形不加读故障）下 R 走得完而两个变体放过：k=5 全量 0 个；`ooww`、`oiww` 只跑了写行根之前全落那一个盘面，走不到或拒。
- 「R 的单元一份不少、指针槽上却是更新的单元」：没有。更新的单元出现在某个槽上，那一槽上 R 那一份就没了；打中的两段都是「R 那一份没了、另一份还在、另一份被读故障藏住」。
- 多层映射树、撕裂、位翻转：第一节第 5、6 小节，没造。

## 六、这条腿自己的限度

- 全部是副本上的数：补丁只改那一判与调用处，`today` 与冻结副本同判；原型是攻方写的。要引进判决，得在入库装置上重做（`.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做`）。
- 「先照 today 跑、再强制走另一支」靠的是那一判之前的路径对每个候选相同；本轮全部走到那一判的（状态，次序，读故障）那一判只走到一次（输出里 `creds=many` 零行，`grep -c 'creds=many'` 量过各件输出为 0）。
- 历史缩短：每段 M1 写流只枚举写行根落盘之前那 16400 个状态（缩前 k=5 是 20516、`oiww` 24633、`ooww` 24623）；缩掉的是写行根落了之后的状态，那时 R 已被抛弃，那一判不看它（推的，读码 `placements_in_the_accounts_of_newer_readable_roots` 只取 `!is_abandoned(root)` 的根）。留下的 16400 个一个不落、原地覆写取三态。
- 读故障组缩了：k=5 只有 3 组全量（`none`、`tt1+mr0+mr1+rec`、`tt1+mr1+rec`），另 14 组只跑了序号 0–3719 与 16399；`ooww` 只全量 `mr0+mr1+rec`，`oiww` 只全量 `rec`。缩的理由是挂钟：前 31 件（每件 120 个状态 × 17 组 × 两序）在负载 75–105 下跑了 1543.73 秒，照这个速度 17 组全量要再跑五批、约两小时；缩成 3 组之后其余 31 件（三段历史合计 70835 个（状态，读故障）对：12679 × 3 + 16399 + 16399）实际 436 秒（那时负载降了）。
- 用户动作没放开扫：打中发生在 M2 写行那次发布的根落盘之前，在任何用户动作之前；M1、M2 都是挂载自己。
- M2 的读故障是 R 的那几段读回全零（本仓读故障模型，入库用例同一个包装）；它要读故障在两次挂载之间变，与第二轮三次挂载、入库 `ur`/`urJ`/Y2 同一个前提。打中那两段 M2 的读故障是 M1 那批段的子集。
- 「R 走得完」按池级 checker 不红 I-7.2 判；「还在用」的认定交主 agent（第三节第 3 句）。
- `r_unit_writes=20`：本轮录的 R（覆盖写、k=5）写出 20 份单元写、10 个单元（`name=c834_history` 行，量过）；第二轮攻方报告写「14 份单元写（7 个单元）」，与本轮数的不同，我没去对第二轮那一句是怎么数的。
- 编译约 10 分钟是三次续编的合计，粗估（第一次被 `timeout 235` 停在 librocksdb-sys 里，后一次被工具挪到后台跑完）。
- 换几何只扫了 `o`/`i`/`w` 按个数排的前缀（总长 1–9）与 k = 0–10、12、14，没扫别的排列、别的用户动作、别的盘宽。

### 取样、估时与分批

- 起跑前 `ps -o pid,args -u "$(id -u)"`：别的会话有 `cargo build`（含另一条腿的 `c834-r3-sonnet` 编译目录，没读它的内容）、rustc、rocksdb 的 c++ 在跑，后来另一条腿的原型件 31 个在跑；没有性能测量（`qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`），加 `nice -n 19` 照跑，没等锁。
- 小段估时：k=5 序号 100–110 × 17 组 × 两序 340 行 16.30 秒（含录 R 与 M1 约 2.4 秒）。
- 分批（每批不设限时，`.rc` 文件数与件数逐批对上，退出码全 0）：k=5 前 31 件一批（31 个 `.rc` 全 0，1543.73 秒）；其余 31 件一批（`batch=0 launched=31 rc_files=31`，全 0，约 436 秒，按 `.err` 建时与 `batch.log` 末次写时相减）；换几何 `survey` 206 件 7 批（每批 `nonzero=0`，合计 26.11 秒），k 那 26 件逐个跑（62 秒）。两次批跑超过 240 秒被工具挪到后台，用 `proc.py wait` 等，没写轮询。
- 状态数（`crash_state_count_with_torn_in_place_overwrites`，每段跑前算）：k=5 20516、三条记录 k=5 20521、`oiww` 24633、`ooww` 24623；每段都不到约 10⁶，本腿实际枚举的（不重复的）状态 49200（三段各 16400，写行根之前全落那一个盘面算在里面），另加三条记录 k=5 那一个单独的盘面，远低于约 10⁷。

## 七、没做什么

- 没在入库装置上跑；没跑 checker 档测试、门禁阶段、轻型三条，没起 GPU。
- T3、T4、T5 不归攻方，没做；第三节第 3 句与第一节第 4、6 小节的两处「零轮」只列出来，没往下查。
- 没造多层映射树、撕裂态、位翻转（第一节第 5、6 小节）。
- k=5 的另 14 组读故障、`ooww` 与 `oiww` 的其余读故障没全量；`oiww`、`ooww` 的读故障 `none` 只跑了写行根之前全落那一个盘面。
- 没实现「记录只认建在 R 上的」之类别的改法；`pointer_pair`、`mapping_pair` 没跑入库 harness 用例（第四节表最后一列是推的）。
- 没把 `rerun.sh` 从头跑一遍：全程要编一次 librocksdb-sys 再跑两批，前台 4 分钟放不下；只核了它的头尾两段与模板逐字相同、`bash -n` 过。
- 原型只跑自己造的流；搭池用入库 harness 的 `tests/common_unwitnessed_publish/mod.rs`（它引 `tests/common_admission_pool/mod.rs`），没拷崩溃枚举用例，与第二轮攻方同样写明给主 agent 判；测试目标不在 checker 档包里，原型是一个独立的二进制，直接执行它（经内存包装），没起 `cargo run`。
- 没读禁读清单里的文件与目录；`ps` 输出里看到过另一条腿编译目录的路径名，没进去读。

## 要主 agent 跑的命令

- 复跑：第一段「复跑命令与模型文件」那一条（工作目录随意；经 `run-with-memory-cap.sh 24G`；线程取 `RERUN_THREADS`，默认 31；编 librocksdb-sys 那一步 `rerun.sh` 里自带 `BINDGEN_EXTRA_CLANG_ARGS`，取 `scripts/crash/crash-amplification-lib.sh` 的 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS` 两个目录）。挂钟超过 4 分钟，要后台跑。看末行 `name=rerun_compare all_same`。
- 想补齐 k=5 的另 14 组读故障：在复跑的草稿目录里，用同一个二进制与 `tools/run-batch.sh`，件表照 `tools/pieces-rest.txt` 的 k=5 那几行把读故障组换成 `tools/faultsets.txt` 里另外 14 组；估约两小时（负载 75–105 时），没跑。

## 八、草稿与产物去向

- 草稿目录 `/home/fy5090/.local/share/singlefs/work/c834-r3-opus/`：仓副本 `tree-candidates/` 与编译目录 `target/` 交回前删掉了；留下 `runs/`（逐状态原样输出约 161M：`o5/` 前 31 件、`mix2/` 其余 31 件、`urjimage/`、`survey/`、`prefix/`、`probe/`）、`aux/`（件表、读故障组、改补丁的脚本）、`model/`（与模型目录同一份）、五份编译日志。原样输出太大不入库，模型目录 `out/` 只收汇总与摘录。
- 入库的产物只有本报告与模型目录 `research/prompts/c834-r3-opus-model/`。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/c834-r3-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_c834-r3-background.md` 末行（没判的 1 处是 I-7.2 那一行只指路不抄；本报告引的代码文件与冻结副本 sha256 逐个相同，量过）：

```
  ✓ 核了 12 处引文、1 处裸行号，对上 12 处，没判 1 处（其中裸行号没有引文可比 1 处；逐处列在上面）
```
