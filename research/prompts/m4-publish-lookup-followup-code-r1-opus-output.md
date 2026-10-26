# m4-publish-lookup-followup-code-r1：攻方腿（Opus）报告（2026-10-22）

立场：正文第四节「云端攻方」那一行。量的都是冻结副本的拷贝（草稿目录 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1-opus/` 下），是副本上的数，不是入库装置上的数。
改前的写法 = 改动全文那份材料第一个 diff 围栏（七·6 后续批 `crates.patch`）反向打到拷贝上（`patch -R -p1`，无 fuzz）。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-followup-code-r1-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs \
  --frozen ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1/repo \
  --work <新的草稿目录> \
  --model research/prompts/m4-publish-lookup-followup-code-r1-opus-model
```

脚本拷冻结副本三份（after 改后、before 改前、probe 改后加探针），依次跑 T3 变异（基线与 MA1–MA5，快档全部不忽略的用例 + 见证用例）、T3 慢档（MA5 单独、MA1+MA2+MA3+MA4 一起）、
T1 差分（release，种子分到 31 条线程）、T2 探针（release，六个场景各一条线程）、T4 构建脚本元数据；最后在输出里逐行找 `out/rerun-expected.txt` 的每一行，打 `name=rerun_compare …`。
本机整条跑一次的结果见末尾「这条腿自己的限度」一节。模板自带的 `--selftest` 在换过 `rerun_steps` 之后不再适用（它的第一格按模板那一段判），没当判据。

模型目录各文件的 sha256：

```
aa7db9cb15dfe0f54e35f413fe7ea3ffed43e6a77aa212ee947ed44e20f9c9cd  ./mutations/attack-mutations.tsv
527a66450110ce8daa955f8c0b61792eccc3aa846b6416c069174fd0b846a24f  ./out/rerun-expected.txt
c545512ce34c04a29dd613884296f94654cf43679315f4cfd80d3cafcd72704e  ./tests/attack_followup_differential.rs
a0a40a69932c5b43a0827ce2963f9bc5f55317381ef0c1cb2fee81d551f61beb  ./tests/attack_followup_probe.rs
76ed763ca8a26f4248fa24dca82c42ab659db02922161e68ee2883ceb2082473  ./tests/attack_followup_witness.rs
8355553ebcdc27cebe35b7e494038a194fe8ad2fd3baf37c7f8b4cf116b157cc  ./tools/apply_mutation.py
ee388a0527b6fe8ec8acae112de91d7d1c2a473dc49905575fe47bc0a4d80c52  ./tools/differential.sh
f0940e1fd3aeb4cfdb805004b0d08fb9d0bfcfc45a27d57b3fbaa95a1c0295c1  ./tools/probe.patch
cacc331ac690a154b398da06268c5509f71dc3efb0fe4074560e8a4980af3b87  ./tools/probe.sh
078cd4b42db862e2ff865c6eb898483e655d88ed8562005844d66599165916d4  ./tools/rerun.sh
23426e5f4444248700fa4c17f65ebe7c2642d4a00441cf6a5d160d59eb766fda  ./tools/run_mutant.sh
a1a98583e4041a106ad046b5658ef33642e48e92f652bf49c0f2147b75e39c63  ./tools/t4_links_metadata.sh
```

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 七处改法的行为 | **没打中** | 512 段随机合法历史 × 64 步（33792 行），改前改后每一步之后两块盘镜像的 sha256、结局、留存条目摘要逐行相同；4000 次留存单元搬迁走了新的按 key 索引；零 panic。读码四处也对得上 |
| T2 射程 | **打中一处，另报两处范围外** | `take_the_mapping_positions_over_the_location_hints` 末尾按角色在 `units` 上线性找，整理每发一批对搬走的每一项走一遍：一批 8 项比较次数 1004 → 4076 → 16364（单元 159 → 599 → 2354），是七·6 在核源落点那一处去掉的同一种形状（每项 O(P)），不在七处也不在 8 处；同一句在恢复（挂载）时对全部映射条目走，2741311 次（P²/2）。8 处与 `mount.rs` 第 5529 行没打中（量的是常数 × P） |
| T3 计数与用例的判别力 | **打中** | 五条变异 MA1–MA5 快档 10 条全绿；MA1–MA4 连慢档也绿，MA5 只有忽略的慢档红（建索引增长 335%）。病根：发布路径上建索引次数只判「至少」或不判、建在计数作用域外的记进没人判的那一处、多层码 2 树照抄只有 1 个节点时整数遍判法空真 |
| T4 `links` 与构建脚本 | **打中（低概率的编法）** | 环境或 `--config env` 里带 `CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1`、不开特性：rustc 编 `singlefs-core` 不带这个 cfg，构建脚本却交出 `enabled_features=LOOKUP_WORK_COUNTERS`；checker 档判器键表多一行 `build-feature singlefs-core lookup-work-counters`，64 组里 56 组的键变了；之后去掉那个变量再编，构建脚本不重跑，元数据留着旧的 |

## T1　七处改法的行为

**判定：没打中。** 两条路子，不共用代码：读码逐处对（下面四条），与改前改后同一串随机合法操作逐步比镜像（`tools/differential.sh`，副本上的数）。

读码（冻结副本行号）：

- 索引取最先一项：建索引按表的次序收、留每个身份最先那一项（`crates/singlefs-core/src/transaction.rs:3554`：「SortedIdentityIndex::keeping_the_first_value_of_each_identity(」），
  三份新索引（留存条目按 key、第一个文件之外的文件按 inode 号、批量读者手里的单元按角色）都走它；查不到时同 `iter().find` 交 `None`。
- 上一版单元索引挪到装 extent 树之前：建它的那一句在把 `previous` 换成「搬走的单元换到目的地的那份拷贝」之后
  （`crates/singlefs-core/src/transaction.rs:13157`：「let previous: Option<&TransactionOutput> =」，`crates/singlefs-core/src/transaction.rs:13295`：「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」），
  与改前第 14049 行（改前行号）建的是同一份；索引借着那一版，到最后一次用为止那一版改不了（借用检查守）。
- `relocation_checked_against_the_version` 删掉 `previous` 之后，取字节与找留存条目都经 `previous_read_by_key`，它在同一个函数里按同一个 `previous` 建
  （`crates/singlefs-core/src/transaction.rs:9791`：「previous_read_by_key.get_or_insert_with(|| {」），一批里不换版本。
- 查不到时：取字节那一句 panic 文案同 `TransactionOutput::unit`（`crates/singlefs-core/src/transaction.rs:3582` 与 `crates/singlefs-core/src/transaction.rs:4073` 都是「.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)」）；
  `PlannedUnitNotInTheVersion` 与 `DeadlistEntryNotReleasable` 两处的条件仍是「按 key 找不到」，只换了找法（`crates/singlefs-core/src/transaction.rs:6212`：「.ok_or_else(|| PublishError::DeadlistEntryNotReleasable {」）。
  照抄多层码 2 树节点那一句多了一个 `expect` 文案，但它前一句先 `previous.expect`，两个 `Option` 由 `publish_admitted` 一起给，`previous` 是 `Some` 时索引也是 `Some`，新文案走不到（推的）。

差分（`tools/differential.sh`，`tests/attack_followup_differential.rs`）：同一个种子在改前、改后两份拷贝里各跑一段历史，每一步之后打「操作、结局（成败与错误变体名、Debug 文本长度）、两块盘镜像写过的扇区的 sha256 前 12 字节、
留存条目数与其中重复 key 数、第一个文件之外的文件数与重复 inode 号数、单元数、活快照数」。操作：建 inode、按 inode 号顺序写（1–9 单元）、覆盖写一个单元以内、建快照（活快照少于 3 个时）、
删快照、unlink、经会话整理一趟、**直接发一批整理**（从进映射的数据单元与留存的数据单元映射条目里随机取至多批大小那么多项，有留存条目时每项一半机会取留存的）、抬 F 到准入上限；批大小按种子取 1、2、3、8。原样输出（本机，release）：

```
name=differential seeds=512 steps=64 lines=33792 before_vs_after=identical
name=differential_coverage op=new_inodes steps=3431 ok=3431 err=0
name=differential_coverage op=write steps=10167 ok=10167 err=0
name=differential_coverage op=overwrite steps=1925 ok=1925 err=0
name=differential_coverage op=create_snapshot steps=2460 ok=2460 err=0
name=differential_coverage op=destroy_snapshot steps=1533 ok=1533 err=0
name=differential_coverage op=unlink steps=1001 ok=1001 err=0
name=differential_coverage op=compaction steps=4783 ok=4783 err=0
name=differential_coverage op=relocate steps=3802 ok=3802 err=0
name=differential_coverage op=raise_floor steps=3666 ok=3666 err=0
name=differential_coverage panics=0 kept_relocated_total=4000 histories_with_kept_relocation=432 max_dupkept=0 max_dupfiles=0
```

T1 问的「同一 key 有几条留存条目、同一 inode 有几个文件时取哪一条」：33792 个状态里留存条目重复 key 数与文件重复 inode 号数都恒为 0（`max_dupkept=0 max_dupfiles=0`），合法操作造不出重复；
重复时取哪一条只能靠读码（取最先一项，上面第一条）。查不到的那几支（`PlannedUnitNotInTheVersion`、`DeadlistEntryNotReleasable`、取字节 panic）差分一次都没走到（`err=0`、`panics=0`），只读码判。

**什么现象会推翻**：同一串合法操作在改前改后写出不同的镜像行；或者造出留存条目重复 key 的合法历史、且重复的两条落点不同。

## T2　射程判得对不对

**判定：打中一处（发布与整理路径上按项调、没改的线性查找）；8 处与 `mount.rs` 第 5529 行没打中；另报两处表外的平方。**

量法：探针补丁 `tools/probe.patch` 在 probe 拷贝里给几处线性查找的比较闭包各加一次计数（按线程记，不改行为），`tests/attack_followup_probe.rs` 造池（第一个文件之外 F 个八单元的文件，F = 16、64、256，逐个经会话写），
在每档池上读：一批 8 项的整理发批（`publish_relocation_batch`，源取第二个文件 / 最后写的文件的 8 个数据单元）、整份重写一个 8 单元文件、整份重写四分之一的文件之后抬 F、重挂（恢复）；
另在一个快照下整份重写一个 N 单元的文件（N = 8、32、128）。六个场景各一条线程，release。

### 打中：`take_the_mapping_positions_over_the_location_hints` 末尾按角色在 `units` 上找

整理每发一批，`publish_admitted` 把上一版拷一份、把搬走那几把 key 的映射位置盖上去（`crates/singlefs-core/src/transaction.rs:13143`：「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」）；
盖的时候对 `mapped_units` 里每一条问一次 key 搬没搬，搬了的那一项在 `units` 上从头线性找它的角色（`crates/singlefs-core/src/transaction.rs:3762`：「for (identity, key) in &version.mapped_units {」，
`crates/singlefs-core/src/transaction.rs:3767`：「.find(|unit| unit.identity == *identity)」）。一批 K 项就是 K 次、每次 O(P)——与七·6 在 `relocation_checked_against_the_version` 去掉的那一种（计划里每一项在 `mapped_units` 上线性找）同形，
不在七处、不在 8 处、不在 `mount.rs` 第 5529 行。原样输出（probe 拷贝，`mount_placements_find=0 kept_contains=0` 两栏从行里删了，别的照抄）：

```
name=probe files=16 units=159 mapped=156 op=relocation_batch_of_8_from_the_second_file mapping_positions_find=108 allocation_node_find=28 first_file_rewritten_contains=10 mapping_positions_calls=1 record_roles_contains=0
name=probe files=16 units=159 mapped=156 op=relocation_batch_of_8_from_the_last_file mapping_positions_find=1004 allocation_node_find=25 first_file_rewritten_contains=8 mapping_positions_calls=1 record_roles_contains=0
name=probe files=64 units=599 mapped=592 op=relocation_batch_of_8_from_the_second_file mapping_positions_find=108 allocation_node_find=66 first_file_rewritten_contains=16 mapping_positions_calls=1 record_roles_contains=0
name=probe files=64 units=599 mapped=592 op=relocation_batch_of_8_from_the_last_file mapping_positions_find=4076 allocation_node_find=45 first_file_rewritten_contains=9 mapping_positions_calls=1 record_roles_contains=0
name=probe files=256 units=2354 mapped=2337 op=relocation_batch_of_8_from_the_second_file mapping_positions_find=108 allocation_node_find=220 first_file_rewritten_contains=16 mapping_positions_calls=1 record_roles_contains=0
name=probe files=256 units=2354 mapped=2337 op=relocation_batch_of_8_from_the_last_file mapping_positions_find=16364 allocation_node_find=115 first_file_rewritten_contains=10 mapping_positions_calls=1 record_roles_contains=0
```

单元数 ×3.77、×3.93 时，搬最后那个文件的一批比较次数 ×4.06、×4.01（每项约 0.87 × 单元数）；搬第二个文件的一批恒 108（`find` 在表头附近就停）——每项的代价是那个单元在 `units` 里的位置，整理按段选，位置随段走。
一趟整理把一个池扫一遍要发约 P / K 批，合起来是 P² / 2 这一阶（推的，没量整趟）。`mapping_positions_calls=1`：一次发布只调一次，不是定点重复。

同一句在恢复时（`crates/singlefs-core/src/recovery.rs:4147`：「take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {」）对每一条映射条目都找，是 P² / 2：

```
name=probe files=16 units=159 mapped=156 op=remount mapping_positions_find=12246 mount_placements_find=8328 kept_contains=0 allocation_node_find=156 first_file_rewritten_contains=54 mapping_positions_calls=1 record_roles_contains=0
name=probe files=64 units=599 mapped=592 op=remount mapping_positions_find=176715 mount_placements_find=40248 kept_contains=0 allocation_node_find=396 first_file_rewritten_contains=68 mapping_positions_calls=1 record_roles_contains=0
name=probe files=256 units=2354 mapped=2337 op=remount mapping_positions_find=2741311 mount_placements_find=149710 kept_contains=0 allocation_node_find=990 first_file_rewritten_contains=64 mapping_positions_calls=1 record_roles_contains=0
```

2337² / 2 ≈ 2.73 × 10⁶，与 2741311 同阶。这一半在挂载路径上，不在七·6 的「发布与整理路径」字面里，单列。

四句：① 分不分辨臂——分辨「七·6 射程已收全」与「发布整理路径上还有按项线性找」两种说法，同一批整理上一次量就分开；② 系统看得到——比较次数是产品代码自己做的；
③ 满足正文 T2「发布与整理路径上还有没有按项调、随表长平方涨的线性查找漏在七处与 8 处之外（… 在 `units` …上）」那一分句：调用点在整理发批的发布里、按项、查 `units`；
④ 跑前条款没给改法，不适用。**推翻条件**：整理产品路径的批里搬走的单元恒落在 `units` 表头常数位置之内（那样每项是常数）；或 `destination_of_key` 为空的那一支（只供测试的开关）才是产品路径。

### 没打中：8 处单次调用与 `mount.rs` 第 5529 行

- 8 处：逐个找到调用它的函数（冻结副本行号；背景材料与 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 第 180 行写的 4103、4119、6452、6481、7115、7780、10146、12839 是**改前**文件的行号，
  在今天的冻结副本里那几行是别的东西，例如第 4103 行是 `mapping_entries_kept_for_the_snapshots_by_key` 的 `&self,`；改后它们在 4131、4147、6546、6575、7209、7874、10262、12963）。
  调它们的地方都不在按单元或按文件的循环里（`admission.rs` 按树各一次、`plan_the_files_other_than_the_first_after_this_publish` 的闭包对这次写的那一个 inode 调、删除批按一个 inode 调），
  是读码，没给它们单加探针。
- `mount.rs` 第 5529 行（`crates/singlefs-core/src/mount.rs:5529`：「.map(|role| (*role, output.unit(*role).slot))」）是**按这次重写的每个角色**各线性找一遍，不是一次 O(P)；
  但它只对抬 F 那一串空发布与取号之后的预演发布调，那几次发布重写的角色数是常数级，量出来是常数 × 单元数：

```
name=probe files=16 units=159 mapped=156 op=raise_floor raised=true mapping_positions_find=0 mount_placements_find=7410 kept_contains=0 allocation_node_find=150 first_file_rewritten_contains=48 mapping_positions_calls=0 record_roles_contains=0
name=probe files=64 units=599 mapped=592 op=raise_floor raised=true mapping_positions_find=0 mount_placements_find=32052 kept_contains=0 allocation_node_find=330 first_file_rewritten_contains=54 mapping_positions_calls=0 record_roles_contains=0
name=probe files=256 units=2354 mapped=2337 op=raise_floor raised=true mapping_positions_find=0 mount_placements_find=126438 kept_contains=0 allocation_node_find=810 first_file_rewritten_contains=54 mapping_positions_calls=0 record_roles_contains=0
```

  比值 ×4.33、×3.94，线性；常数约 54（7410 / 159 ≈ 47，126438 / 2354 ≈ 54）。「挂载路径」这个判法只对一半：同一个函数也经会话准入被空间拒之后推的抬 F 走到
  （`crates/singlefs-core/src/mounted_session.rs:3494`：「match push_one_floor_raise_within_the_admission_budget(」→ `mount.rs` 的 `raise_the_floor_through` → `crates/singlefs-core/src/mount.rs:4860`：「let placements_taken: Vec<PlacementsTakenByOnePublish> = publishes」），
  那是用户改动的发布路径。阶数上与主 agent 的判断相容（每次 O(P)、常数约 50），名字判错，不影响结论。

### 范围外另报（不在 `units`、`mapped_units`、文件表、留存条目表上，表是这一次发布自己的清单）

| 处 | 量的 | 随什么平方 |
|---|---|---|
| 释放两处 `kept_for_the_snapshots.contains`（`crates/singlefs-core/src/transaction.rs:4338` 与 `crates/singlefs-core/src/transaction.rs:4820`） | 快照下整份重写 N 单元：N = 8、32、128 时 882、3636、49962 次 | N（这一次换下的角色数 × 留给快照的角色数） |
| 记录点名 `data_roles_named_before_the_last_record.contains`（`crates/singlefs-core/src/transaction.rs:11713`） | 同一场景 420、2790、31242 次 | N（这一次写的数据单元数） |

原样输出：

```
name=probe rewrite_units=8 op=whole_rewrite_under_a_snapshot mapping_positions_find=0 mount_placements_find=0 kept_contains=882 allocation_node_find=28 first_file_rewritten_contains=23 mapping_positions_calls=0 record_roles_contains=420
name=probe rewrite_units=32 op=whole_rewrite_under_a_snapshot mapping_positions_find=0 mount_placements_find=0 kept_contains=3636 allocation_node_find=25 first_file_rewritten_contains=45 mapping_positions_calls=0 record_roles_contains=2790
name=probe rewrite_units=128 op=whole_rewrite_under_a_snapshot mapping_positions_find=0 mount_placements_find=0 kept_contains=49962 allocation_node_find=42 first_file_rewritten_contains=145 mapping_positions_calls=0 record_roles_contains=31242
```

它们随一次写的大小平方，不随池大小；七·6 的题面是随池大小，这两处不算七·6 的漏，交主 agent 看要不要另记。照抄分配记录树节点那一处（`allocation_node_find`）与照抄第一个文件数据单元那一处（`first_file_rewritten_contains`）在量的范围里是一次发布 O(P) 或更小，没打中。

## T3　计数与新用例判的是不是产品调用点上的事

**判定：打中（五条变异，快档全绿；其中一条慢档红）。** 量的是冻结副本的拷贝（副本上的数，不是入库装置上的数）。

被打中的判法是三样：

1. 发布路径上只判「至少」或根本不判建索引次数的几处，每项重建不红。删快照那一处只判 `released >= 4`、`index_lookups >= released`、`index_builds >= 1`
   （`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1675`：「&& kept_entries_released.index_builds >= 1,」）；
   整份重写时按 inode 号找文件那一处在发布路径上只判查的次数（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:371`：「work.index_lookups >= data_units_replaced,」），
   恰好等于 `(1, replaced_roles)` 只判直接调释放判定入口那一次（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1528`：「(1, replaced_roles),」）。
2. 建索引记在哪一处，取决于建的那一句在不在 `lookup_work::entering` 的作用域里。照抄 extent 树节点那一处的作用域只包查、不包建
   （`crates/singlefs-core/src/transaction.rs:11975`：「let _inside_the_carried_nodes = lookup_work::entering(LookupSite::CarriedExtentTreeNodes);」），
   在它前面每个节点现建一份索引，建的次数记进「没有点名的那一处」（`None`），快档没有一条判它；`assert_every_lookup_examines_at_most_a_binary_search` 只判看过的项数，不判建的次数
   （`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:225`：「let examined_at_most = work.index_lookups * binary_digits_of(work.largest_index_looked_up);」）。
3. 整数遍判法在只照抄一个节点时是空的。整份重写那一条用例里多层码 2 树照抄的节点只有 1 个（见证行 `CarriedNodesOfTheMultiLevelTrees … index_lookups: 1`），
   走一遍该查 1 次，「查的次数是 1 的整数倍」恒真；记账树、deadlist 树的节点在这条用例里一个都没照抄，它们退回线性找不红。

五条变异（`research/prompts/m4-publish-lookup-followup-code-r1-opus-model/mutations/attack-mutations.tsv`，打法 `tools/apply_mutation.py`，原文各恰好命中一次）：

| 名 | 改了哪一句 | 快档 10 条（`publish_lookup_by_identity` 全部不忽略的用例） | 见证计数（改前 → 改后） | 慢档（忽略的那一条增长用例） |
|---|---|---|---|---|
| MA1 | 删快照取放掉条目留存的映射条目：`get_or_insert_with` 换成 `insert`，每条现建一份按 key 的索引（`crates/singlefs-core/src/transaction.rs:6209`） | 全绿 | 删快照那一处建索引 2 → 16（查 16 次不变） | 见下 |
| MA2 | 快照判定那一圈：每个换下的角色现建一份按 inode 号找文件的读者（`crates/singlefs-core/src/transaction.rs:6128`） | 全绿 | 48 文件整份重写，按 inode 号找文件那一处建 8 → 24 | 见下 |
| MA3 | 核两条位置项与读盘核校验和共用的换下角色清单：每个角色现建一份读者（`crates/singlefs-core/src/transaction.rs:4854`） | 全绿 | 同上 8 → 40 | 见下 |
| MA4 | 照抄多层码 2 树节点：只有中央映射树走索引，记账树、deadlist 树退回在 `units` 上线性找（`crates/singlefs-core/src/transaction.rs:14919` 之后那一句） | 全绿 | 照抄的多层码 2 树节点只有 1 个（中央映射树的），查 1 次不变 | 见下 |
| MA5 | 照抄 extent 树节点：在计数作用域之外每个节点现建一份上一版单元索引（`crates/singlefs-core/src/transaction.rs:11975` 之前） | 全绿 | 没有点名那一处建索引 5 → 52 | **红**：`index_builds_growth_in_percent=335`（门槛 200，未变异 100） |

快档那一列是 `run_mutant.sh` 打的原样（去掉了挂钟那一段）：五条都是 `test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out failed=[]`。见证计数原样（`tests/attack_followup_witness.rs`，不判只打，基线与变异各一行）：

```
name=witness mutant=baseline witness files=8 destruction site=KeptMappingEntriesReleasedByTheSnapshotDestruction LookupWork { index_builds: 2, index_lookups: 16, index_entries_examined: 42, largest_index_looked_up: 8, mapping_leaf_decodes: 0 }
name=witness mutant=MA1 witness files=8 destruction site=KeptMappingEntriesReleasedByTheSnapshotDestruction LookupWork { index_builds: 16, index_lookups: 16, index_entries_examined: 42, largest_index_looked_up: 8, mapping_leaf_decodes: 0 }
```

（MA2、MA3、MA5 的见证行在 `out/rerun-expected.txt`：按 inode 号找文件那一处 `index_builds: 24`、`index_builds: 40`，没有点名那一处 `index_builds: 52`；基线分别是 8、8、5。）

慢档（`compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units`，`--ignored`）原样：

```
name=slow_mutant mutant=MA5 build_rc=0 rc=101 test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 10 filtered out failed=[compaction_batches_decode_each_routed_mapping_leaf_once_and_examine_index_entries_sublinearly_from_512_to_2048_data_units ]
name=slow_mutant mutant=MA1+MA2+MA3+MA4 build_rc=0 rc=0 test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out failed=[]
```

基线慢档本机 `growth_in_percent=124 index_builds_growth_in_percent=100`，MA5 是 `index_builds_growth_in_percent=335`，MA1+MA2+MA3+MA4 是 `100`（读数在草稿目录 `slow-logs/`）。
MA1–MA4 在慢档里看不见的理由：慢档只量整理那几趟，整理不删快照（MA1）；每一批的项数在两档池上都是 K = 8，每项重建让建索引次数按项数涨、不按池大小涨，比值不动（MA2、MA3）；MA4 改的两棵树在这几趟里照抄不到。

T3 正文点名的其余几问：

- 「恰好等于」的断言数的是产品调用点：是。三处释放、核源落点、照抄三处读的都是产品代码里 `lookup_work::entering` 那几处的计数；漏的是作用域外与没断言的那几处（上面三样），不是数错了地方。
- 释放三处直接调公开入口：发布路径（`crates/singlefs-core/src/transaction.rs:11234`、`:11244`、`:11460` 附近三处调用，冻结副本行号）调的是同一个函数体，函数体里改回线性被直接调那一条抓到（实现员的四行变异）；
  只在发布路径的调用者里改的（MA2 在快照判定那一圈、MA3 在两处共用的换下角色清单）抓不到。
- 2 项、8 项两档：核源落点那一处按 `(建, 解叶, 查)` 恰好等于判，每项重建（变异表里攻方 RC 那一行）让建的次数按项数涨、对不上恰好等于的 3（推的，这一轮没重跑 RC）；这一处判得出三类，没打中。
- 整数遍判法：照抄 extent 树节点那一处一遍 47 次、只走 1 遍，改一部分凑不成整数遍；多层码 2 树那一处一遍 1 次，空真（MA4）。
- 证红之后的类型别名与三处 clippy 写法：读码判，没打中。`KeptMappingEntriesOfAVersionByKey` 只出现在结构体字段的类型里，19 行变异的原文与变异文都不碰它；
  `assert_the_work_is_whole_passes_of` 里 `checked_div(...).unwrap_or(...)`（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1407`：「.checked_div(builds_of_one_pass)」）在走一遍不建索引时取查的次数，与「除数为 0 另算」同义。改前那三处写法我看不到原文，这一条是推的，没重跑 19 行证红。

四句（对五条变异合起来答）：① 不分辨臂——这一格判的是测试的判别力，不是两种查法谁对；② 系统看得到——计数是产品代码自己记的，测试读得到；缺的是断言，不是观测；
③ 满足正文 T3「按批大小取样…分不分得出每项重建、逐 key 查、改回线性三类」与「整数遍判法…会不会让一个改坏的写法凑成整数遍而不红」两个分句（MA1–MA3、MA5 是每项重建，MA4 是改回线性、整数遍空真）；
④ 跑前条款没给改法。**推翻条件**：仓里另有一条不在 `publish_lookup_by_identity` 里的用例（或门禁的变异表行）在 MA1–MA5 下红。我只跑了这一个测试目标的快档与它的慢档，没跑别的目标。

## T4　`singlefs-core` 加 `links` 与构建脚本

**判定：打中一种编法（环境里带 `CARGO_FEATURE_*`）；其余三问没打中。**

构建脚本把进程环境里所有以 `CARGO_FEATURE_` 起头的变量都当成 cargo 开的特性（`crates/singlefs-core/build.rs:9`：「let mut enabled_feature_environment_names: Vec<String> = std::env::vars_os()」，
`crates/singlefs-core/build.rs:11`：「.filter_map(|name| name.strip_prefix("CARGO_FEATURE_").map(str::to_string))」），只登记了 `build.rs` 自己变才重跑（`crates/singlefs-core/build.rs:18`：「println!("cargo::rerun-if-changed=build.rs");」）。
cargo 不清掉从外面带进来的同名变量，`--config` 的 `[env]` 也照样设进构建脚本的环境。`tools/t4_links_metadata.sh` 每格一个新编译目录，只编 `singlefs-core`（`-v`，看 rustc 那一行有没有 `--cfg 'feature="lookup-work-counters"'`），原样输出：

```
name=t4 cell=plain build_script_ran=yes rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=]
name=t4 cell=feature build_script_ran=yes rustc_cfg_lookup_work_counters=on build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]
name=t4 cell=environment build_script_ran=yes rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]
name=t4 cell=config_env build_script_ran=yes rustc_cfg_lookup_work_counters=off build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]
name=t4 cell=stale build_script_ran=no rustc_cfg_lookup_work_counters=not-recompiled build_script_output=[cargo::metadata=enabled_features=LOOKUP_WORK_COUNTERS]
name=t4 cell=checker_tier_keys_plain table=found groups=64 build_feature_lines=[]
name=t4 cell=checker_tier_keys_environment table=found groups=64 build_feature_lines=[build-feature singlefs-core lookup-work-counters]
name=t4 cell=checker_tier_keys_compare groups_with_a_different_line=56 of 64
```

`environment`：`CARGO_FEATURE_LOOKUP_WORK_COUNTERS=1 cargo build -p singlefs-core`；`config_env`：`--config 'env.CARGO_FEATURE_LOOKUP_WORK_COUNTERS="1"'`；`stale`：在 `environment` 那个编译目录里去掉变量再编一次。
后三行：同样两种设法下 `cargo check -p singlefs-checker-tier --no-default-features`（库本身因 `compile_error` 编不过，构建脚本照跑），读它写进 `OUT_DIR` 的判器键表。
编出来的 `singlefs-core` 两边都不带计数，键表却一边按「开着」算，64 组里 56 组的键不同。

四句：① 分辨的是「元数据 = cargo 真开的特性」与「不等」两种情形，这一种编法下不等；② 构建脚本看不到区别——它看到的只有环境变量，cargo 设的与外面带进来的同名同值，这是判别子观测不到
（`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」表第二行那一类），只靠环境变量取不出；③ 满足正文 T4「`build.rs` 交出的特性名是不是 cargo 并过之后的那一份」与「`rerun-if-changed=build.rs` 会不会让特性变了而元数据没重算」两个分句
（后一句：特性真变时 cargo 换一个编译单元、构建脚本照跑，`feature` 格可见；只有外带变量变了才不重跑，`stale` 格）；④ 跑前条款没给改法。
后果按上一轮判决的口径：今天计数对行为中性，错的键最坏是该复用的不复用、或开计数的二进制顶着不开的键（推的，没量）。这一种编法要人在环境或 cargo 配置里写 `CARGO_FEATURE_` 变量，仓里的脚本今天不这样写（没全仓搜，推的）。
**推翻条件**：cargo 在起构建脚本前清掉外带的 `CARGO_FEATURE_*`（换一版 cargo 重跑这一格，`environment` 行变成 `enabled_features=`）。

没打中的三问：

- 产品与发布构建：构建脚本只打 `cargo::metadata` 与 `rerun-if-changed`，没有 `rustc-link-lib`、`rustc-cfg`、`rustc-env`，不生成代码；`plain` 格 rustc 不带计数 cfg。没比较发布构建的产物字节（没量）。
- 并过之后的特性：`feature` 格（`--features lookup-work-counters`，cargo 真开）元数据带上了；dev 依赖并上来的那一种没单独量（要编 checker 档的测试目标，按定义不起），读码同理。
- 同名 `links`：冻结副本里 `links = ` 只有 `crates/singlefs-core/Cargo.toml:10` 这一处（`grep -rn 'links\s*=' --include=Cargo.toml`），`scripts/experiment-apparatus/Cargo.toml` 不依赖 `singlefs-core`，撞不了。

## 我提的改法（只在我的模型上量过、被攻过零轮）

下面的改法我一条都没实现、没跑，格里全是「推的」；「量过」只指打中那一格是量出来的。

| 改法 | 修哪一格 | 打中的那几格上还中不中 |
|---|---|---|
| 甲：`take_the_mapping_positions_over_the_location_hints` 末尾先按 `units` 建一次「角色 → 第一个下标」的表，再逐条改那一项的槽（只改最先那一项，同 `find`） | T2 打中那处，发布与恢复两半 | 推的：每批 K 项变成 O(P log P) 一次建表加 K 次查；恢复从 P² 变成 P log P |
| 乙：用例在发布路径上对删快照、按 inode 号找文件两处判建索引次数恰好等于「走了几遍」（遍数从查的次数或别处取），并判没有点名那一处（`None`）的建索引次数在两档大小下不涨 | T3 的 MA1、MA2、MA3、MA5 | 推的：四条变异都让建的次数按项数涨，判得出；MA4 不中 |
| 丙：整份重写那一条用例的池里先建快照、删出够多的 deadlist 条目，让记账树或 deadlist 树照抄到不止一个节点，再按树分开数 | T3 的 MA4 与整数遍空真 | 推的 |
| 丁：构建脚本改读 cargo 设的 `CARGO_CFG_FEATURE`（cargo 自己按这次编译的特性给，不取外带的同名变量；这个变量哪一版 cargo 起有、外带时会不会被盖掉没查），或在交出前把本包清单 `[features]` 里没开的名字剔掉 | T4 | 推的；后一种只防住清单外的名字，`LOOKUP_WORK_COUNTERS` 是清单里的名字，防不住这一次打中的那一格 |

## 没打中的形状（取样范围）

- T1：512 段历史 × 64 步，九种操作（见 T1 的覆盖行），批大小 1、2、3、8，活快照至多 3 个，文件 1–9 个单元；比的是每一步之后的镜像 sha256（前 12 字节）与几样内存摘要，没枚举崩溃状态（这一格比的是写出的镜像，不是崩溃恢复）。
  走不到的：查不到时的三条错误路径、留存条目重复 key、同一 inode 两个文件。
- T2：探针只加在我挑的六处（三处范围外、一处打中、两处挂载/抬 F），池最大 2354 个单元；8 处是读码，没加探针。
- T3：只造了「每项重建」「部分改回线性」两类；没造「索引照建、查也照记、另外再走一遍线性」那一类（上一轮判决已列为判法看不见的一类，不重复）。
- T4：没造 `RUSTFLAGS` 里 `--cfg feature=…` 那一种——它进 `CARGO_ENCODED_RUSTFLAGS`，checker 档构建脚本的工具链行里有，键会变（读码，推的）。

## 这条腿自己的限度

- 复跑：上面那条命令本机整条跑过一次，退 0，末行原样 `name=rerun_compare 54 of 54 lines of out/rerun-expected.txt found`（草稿目录 `rerun-run.txt`，新草稿目录 `rerun-work/`，跑完已删）。
- 变异只打在 `transaction.rs`、只判 `publish_lookup_by_identity` 这一个测试目标（快档 10 条 + 慢档 1 条）；别的测试目标在这五条变异下红不红没量。
- 探针改了产品代码（加计数），比较次数是探针数出来的；探针本身没有会红的检查（没证它在每一处都数到），只靠「搬第二个文件恒 108、搬最后一个按单元数线性涨」这种与位置吻合的形状旁证。
- T1 差分与上一轮攻方的差分同一路子（每一步比镜像），这一轮多了直接发整理批与留存单元的搬迁；崩溃状态没有枚举。
- T4 只量了 `singlefs-core` 构建脚本与 checker 档构建脚本写的键表，没跑判法单元键之后的复用（要 checker 档的测试，不起）。
- 中途停了重跑一次：差分第一版是 debug 档、种子在一条线程上串行，调度方看门狗报只用 0.5 核，照它的消息用 `proc.py stop` 停了改前那一趟（进程 1111905，改后那一趟已跑完 32 段、逐行与改前比之前停的，那一份不作数），
  改成 release、种子按轮转分到 `SINGLEFS_THREAD_CAP` 条线程（capped.sh 设成 31）再跑；报告里的差分数都是重跑那一份。探针同样改成六个场景各一条线程、计数按线程记。
- 中途收到调度方第二条消息（第二台跑 E185，不往第二台挪）：我从头到尾没用第二台。

## 负载与在哪跑

全部在本机、经 `scripts/hosts/capped.sh 31` 与 `run-with-memory-cap.sh 16G`、`nice -n 19` 跑，同一时刻只起一条 cargo。开跑时 `ps` 看到别的会话的 checker 档 `cargo test` 与 harness 测试在跑、没有性能测量进程；
资源提醒报本机 32 核在用约 32 核（loadavg 约 80）、第二台同样占满。第一次直接执行测试二进制时内存包装排队约 1.5 分钟（测试本身 2.19 秒）。

## 要主 agent 跑的命令

无。没改仓里的产品代码与脚本；崩溃轻型、变异轻型与这一格无关，没有要交的。

## 没做什么

- 没跑任何门禁阶段、checker 档测试、`gate.sh`，没起 GPU，没用第二台。
- 没判正推那几格；没替主 agent 采纳；副本上的数都不是入库装置上的数，要引得在入库装置上重做。
- 没重跑实现员的 19 行变异证红；类型别名与 clippy 那三处是读码判的。
- 改法甲到丁一条都没实现。
- 草稿目录里自己建的仓副本（after、before、probe、rerun-work）与编译目录（target-after、target-before、target-probe、target-t4-*）交回前已删；日志与草稿留在 `~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1-opus/`。
