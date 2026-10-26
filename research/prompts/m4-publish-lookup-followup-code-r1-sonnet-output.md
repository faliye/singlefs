# m4-publish-lookup-followup-code-r1 云端正推腿报告（2026-10-22）

腿：three-way-forward（Sonnet）。立场：从补丁原文与冻结副本推「代码做的是不是七·6 代码轮第三轮判决第三节 1–4 条与用户定案要的那件事」。被判树：`~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1/repo/`（下称冻结副本），引代码的路径与行号都是冻结副本的；「改前」引 `research/prompts/_m4-publish-lookup-followup-code-r1-diff.md` 里 `-` 开头的行（我把那份补丁倒着打回冻结副本拷贝，`patch -R` 无报错，得到改前树）。

## 复跑与模型文件

模型目录 `research/prompts/m4-publish-lookup-followup-code-r1-sonnet-model/`。复跑（整条经内存包装，约半小时到一小时，视本机负载）：

```text
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/m4-publish-lookup-followup-code-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-publish-lookup-followup-code-r1/repo --work <新的草稿目录>
```

`RERUN_ONLY=check,baseline` 这类环境变量只跑几步（步骤名 `check,baseline,rows,extra,probe,equal`）。文件与 sha256：

```text
2bf8e9ebfb3746c831f8365ba1a2f27add2da6c39a0f803073e23aba8b98139a  mutants/extra.tsv
7fa695551c357cbd867a88265944db4d71d39957ad002badc32362591a92f3da  probe/probe_test_relocation.rs
954be87b40e29b531299247628bab3d5d7299ca9b3540357e4bc13e618c3b7ba  probe/probe_test.rs
420c02edde85efbf4b721be7d0d12b1367fda8a0597bfcb15207c1470c22c69e  tools/check_mutation_rows.py
6fc8ebf6d204fbf3980807bee2714320a1412d6b8f45c12afc28d2b1d2eeaab4  tools/instrument.py
39c48f901e5f79330bfdf648e8a1800bb3cc92f2b577c60713ecf47ebd31adb7  tools/rerun.sh
37c738048e3a57cbac45193663a9eaa20f9f73f3dfde0ae0e74f5ce55bdbb32d  tools/run_mutants.py
```

`out/` 里是我量出来的原样结果（sha256 在报告末尾）。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来、只改了 `rerun_steps`，所以模板自带的 `--selftest` 在它上面会红（自证期望的是模板那一段），这不是模板坏了。

## 结论一览

| 格 | 判定 |
|---|---|
| T1 七处改法的行为 | 成立 |
| T2 射程 | 部分不成立：8 处单次调用成立；`mount.rs` 5529 行判成挂载路径对一半；发布路径上漏了 `units` 按角色线性找（两处）与几处角色清单 `.contains`，整份写大文件时随写入量平方涨（量了） |
| T3 计数与新用例 | 部分成立：三条产品调用点用例数的是产品调用点，19 行变异在终稿上全红；留了三个没被守住的缺口（量了，变异全绿） |
| T4 `links` 与构建脚本 | 成立 |

环境：开跑前 `ps` 看到别的会话的 `cargo test`、`cargo build` 在跑，本机一分钟平均负载 70 以上；我的第一条 `cargo build` 在 `run-with-memory-cap.sh` 里排队约 9 分钟后起；之后的编译和测试都在本机，没用第二台。
## T1　七处改法的行为：成立

**索引取项规则**：「pairs.sort_by(|(left, _), (right, _)| left.cmp(right));」（`crates/singlefs-core/src/transaction.rs:3459`）是稳定排序，「pairs.dedup_by(|(later, _), (earlier, _)| later == earlier);」（`crates/singlefs-core/src/transaction.rs:3461`）丢后一项、留前一项。所以同一把 key（同一个 inode 号、同一个角色）在表里出现几次，索引都交回表里最先那一项，与 `iter().find` 同一项。排序用的 `Ord` 与比相等的 `==` 都是派生的，是同一个关系：「#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]」（`crates/singlefs-core/src/transaction.rs:2465`，`TransactionUnit`）、「#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]」（`crates/singlefs-core/src/address.rs:44`，`InodeNumber`）；映射 key 是 `&[u8]`，切片的 `Ord` 与 `==` 一致。

**七处逐处**（改前一列引补丁原文里 `-` 开头的行；改后一列引冻结副本）：

| 处 | 改前 | 改后 | 同一 key 几条时取哪一项 |
|---|---|---|---|
| 整理发批核源落点：找留存的映射条目 | 「.find(|(key, _)| *key == mapping_key)」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:331`） | 「.kept_mapping_entry_of_key(&mapping_key)」（`crates/singlefs-core/src/transaction.rs:10029`），里面「.get_or_insert_with(|| version.mapping_entries_kept_for_the_snapshots_by_key())」（`crates/singlefs-core/src/transaction.rs:9992`） | 表里 key 相同的最先一项 → 同 |
| 整理发批：取计划项的字节 | 「RelocatedUnitHolder::RoleOfThisVersion(role) => previous.unit(role).bytes.clone(),」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:341`） | 「.unit_of_role(role)」（`crates/singlefs-core/src/transaction.rs:10100`），它是「self.units_by_role.unit(identity)」（`crates/singlefs-core/src/transaction.rs:4783`） | `units` 里角色相同的最先一项 → 同 |
| 删快照的批：放掉的条目取留存的映射条目 | 「.find(|(key, _)| *key == entry.mapping_key)」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:253`） | 「.first_entry_of(&entry.mapping_key.as_slice())」（`crates/singlefs-core/src/transaction.rs:6210`） | 同 |
| 释放三处、快照判定圈：按 inode 号找上一版的文件 | 「if !has_a_placement_in_the_previous_version(previous, identity) {」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:78`） | 包装「pub(crate) struct FilesOtherThanTheFirstOfAVersionReadByInode<'version> {」（`crates/singlefs-core/src/transaction.rs:4607`），四个调用点 `:4366`、`:4854`、`:6128`（快照判定圈）与释放判定的建表 `:4335` | `files_other_than_the_first` 里 inode 相同的最先一项 → 同；查不到 `None` 照旧 |
| `build_extent_tree` 照抄节点 | 「let carried = previous_version.unit(role_in_the_previous_version);」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:375`） | 「let carried = units_of_the_previous_version」（`crates/singlefs-core/src/transaction.rs:11976`） | `units` 里角色相同的最先一项 → 同 |
| `build_extent_tree` 照抄下段节点：找文件 | 「previous_version.file_other_than_the_first(*inode)」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:389`） | 「.and_then(|previous_files| previous_files.first_entry_of(inode))」（`crates/singlefs-core/src/transaction.rs:12110`） | 同 |
| `build_multi_level_tree` 照抄节点 | 「let carried = previous_version」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:470`） | 「let carried = units_of_the_previous_version」（`crates/singlefs-core/src/transaction.rs:14920`） | 同 |

**上一版单元索引挪到装 extent 树之前**：改前在「-    let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」（`research/prompts/_m4-publish-lookup-followup-code-r1-diff.md:444`）删掉的那一行才建；改后在「let units_of_the_previous_version = previous.map(TransactionOutput::units_by_role);」（`crates/singlefs-core/src/transaction.rs:13295`）。两处都在 `previous` 改绑之后：「let previous: Option<&TransactionOutput> =」（`crates/singlefs-core/src/transaction.rs:13157`，整理的一批把上一版换成搬到目的地的拷贝）在 13295 之前，所以挪早前后索引建在同一份 `previous` 上。从建到最后一次用，`previous` 是 `Option<&TransactionOutput>` 的共享借用，索引又借着那张表（「索引借着那张表（`'table`）」，`crates/singlefs-core/src/transaction.rs:3534`），表改不了；`TransactionOutput` 里没有内部可变的字段（`grep -nE 'RefCell|Cell<|Mutex|UnsafeCell' transaction.rs` 在 `TransactionOutput` 的结构体定义（3144–3247 行）里零命中；整份文件里 `RefCell` 只用在 3370 行的线程局部计数器上）。所以索引不会过期。四个传下去的调用点都传同一个 `units_of_the_previous_version.as_ref()`：`:13335`、`:13533`、`:13555`、`:13849`。

**删参数 `previous`**：`relocation_checked_against_the_version` 的唯一调用者在「MappedUnitsOfAVersionReadByKey::of_the_version(previous)」（`crates/singlefs-core/src/transaction.rs:9792`）建读者，读者里存的 `version` 就是这份 `previous`（结构体字段「version: &'version TransactionOutput,」`crates/singlefs-core/src/transaction.rs:9963`），取字节走的单元索引是读者建的「units_by_role: version.units_by_role(),」（`crates/singlefs-core/src/transaction.rs:4772`）。同一份版本。

**查不到时**：`FirstEntryOfEachIdentity::unit` 与 `TransactionOutput::unit` 都 `.expect(EVERY_ROLE_LOOKED_UP_HAS_A_UNIT_IN_THE_VERSION)`（`crates/singlefs-core/src/transaction.rs:3582`、`crates/singlefs-core/src/transaction.rs:4073`），同一个常量，panic 文案逐字相同；`build_extent_tree` 里「.expect("照抄的那个文件的下段节点在上一版里")」（`crates/singlefs-core/src/transaction.rs:12120`）改前改后文字不变；`DeadlistEntryNotReleasable { KeptMappingEntryMissing }` 仍从 `ok_or_else` 出来（「refusal: DeadlistEntryReleaseRefusal::KeptMappingEntryMissing,」`crates/singlefs-core/src/transaction.rs:6214`），`PlannedUnitNotInTheVersion` 仍在「return Err(RelocationBatchRefused::PlannedUnitNotInTheVersion { mapping_key });」（`crates/singlefs-core/src/transaction.rs:10044`）。留存条目索引是第一次要找时才建，所以只在原来要走 `iter().find` 的那条路上才建，条件没变。

**改前改后盘镜像**：把补丁倒着打回冻结副本得到改前树（`patch -R` 无报错），两棵树各编、各跑同两条池等价用例，打出的 `images_sha256` 逐字相同：

```text
改前树  shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
改前树  shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
改后树  shape=PoolOfFiles { files: 16, data_units_per_file: 4 } batches_published=24 images_sha256=66357a801c8269e8af3fb3be5e76928281729a11e63f8529069cfcfa2f56fa49
改后树  shape=PoolOfFiles { files: 48, data_units_per_file: 8 } batches_published=44 images_sha256=7b3bb85d08a2822115a91a554d2ecd4fcbbf8f29a6291f621f9081decf90708e
```

范围：这是两串固定的操作（16 个四单元文件、48 个八单元文件，含整理发批），不是「任意合法操作」；其余靠上面的读码。

**推翻它**：任一处的索引与表上 `iter().find` 对同一把 key 取到不同的项；或改前改后树上这两条用例的 `images_sha256` 不同；或某条查不到的路径改后 panic 文案不同。我读到、跑到的都没有。
## T2　射程判得对不对：部分不成立

### 没改的 8 处：每次发布只调常数次——成立

行号是冻结副本的（主工作区开工时的 4103 … 12839 在这一份里依次是 4131 … 12963，同一批语句）。逐处列出调用者与每次发布被调的次数；没有一处在按项的循环里。

| 处 | 在哪个函数 | 调用者 | 每次发布被调的次数 |
|---|---|---|---|
| 「let root_header = parse_index_node(&self.unit(root_role).bytes)」（`crates/singlefs-core/src/transaction.rs:4131`） | `height_read_from_the_root_node_header` | `admission.rs` 的「file_version.height_read_from_the_root_node_header(」（`crates/singlefs-core/src/admission.rs:1164`），在 `checkpoint_cost_of_the_version_to_build_on_with_node_capacities` 里一处表达式，不在循环里 | 该函数每次发布尝试至多两次：准入判落点那一支「checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`crates/singlefs-core/src/transaction.rs:11129`），式子那一判经「let checkpoint_cost = checkpoint_cost_of_the_version_to_build_on_with_node_capacities(」（`crates/singlefs-core/src/admission.rs:1601`）与「&admission_reading_before_a_publish_on_a_ring_of(」（`crates/singlefs-core/src/transaction.rs:11212`）；每次 1 个 `.unit` |
| 「let root_header = parse_index_node(&self.unit(root_role).bytes)」（`crates/singlefs-core/src/transaction.rs:4147`） | `position_addressed_tree_heights_read_from_the_root_node_headers` | `admission.rs` 的「.position_addressed_tree_heights_read_from_the_root_node_headers()」（`crates/singlefs-core/src/admission.rs:1122`），同一个函数 | 同上两次 × 闭包 `height_of`（「let height_of = |root_role: TransactionUnit| {」`crates/singlefs-core/src/transaction.rs:4146`）至多 3 次 |
| 「parse_index_node(&previous_version.unit(TransactionUnit::RetirementRoot).bytes)」（`crates/singlefs-core/src/transaction.rs:6546`） | `retirement_tree_internal_levels_read_from_the_root_node_header` | `committed_reservation_for_tombstones_and_retirement_records_after_this_publish`（`crates/singlefs-core/src/transaction.rs:6512`）一次 | 该函数在「committed_reservation_for_tombstones_and_retirement_records_after_this_publish(」的两处调用（`crates/singlefs-core/src/transaction.rs:11215`、`crates/singlefs-core/src/transaction.rs:13499`）；后一处在记账行的取值闭包里，而 `accounting_entries_of_this_publish` 对每个池级行只取一次值（「value: pool_wide_value(row),」`crates/singlefs-core/src/transaction.rs:7449`）→ 至多两次 |
| 「parse_index_node(&previous_version.unit(TransactionUnit::TombstoneRoot).bytes)」（`crates/singlefs-core/src/transaction.rs:6575`） | `tombstone_tree_internal_levels_read_from_the_root_node_header` | 同上同一个函数 | 同上，至多两次 |
| 「previous.and_then(|previous_version| previous_version.file_other_than_the_first(inode))」（`crates/singlefs-core/src/transaction.rs:7209`） | `plan_the_files_other_than_the_first_after_this_publish` 里的闭包 `previous_file_of` | 一次调用只在三个互斥分支之一调一次（「if let Some(previous_file) = previous_file_of(inode) {」`crates/singlefs-core/src/transaction.rs:7221`、`:7274`，「let previous_file = previous_file_of(inode)」`:7239`）；该函数由 `ordinary_allocation_part` 调（`:7843`） | `ordinary_allocation_part` 每次发布调「.ordinary_allocation_part(previous, &trees, deletion_and_lineage)?」（`crates/singlefs-core/src/transaction.rs:11113`）一次，外加分配记录树固定点每一轮 `resolve` 里一次（「let ordinary_part = self.ordinary_allocation_part(previous, trees, deletion_and_lineage)?;」`crates/singlefs-core/src/transaction.rs:8068`，`resolve` 在「let resolved = plan.resolve(」`crates/singlefs-core/src/transaction.rs:11602` 的轮循环里；轮数上界「pub const ALLOCATION_RECORD_TREE_SETTLING_ROUNDS_LIMIT: usize = 64;」`crates/singlefs-core/src/transaction.rs:11680`，产品路径三四轮）→ 轮数加一次，每次 O(文件表)，与文件数无关地不被每个文件调 |
| 「.file_other_than_the_first(inode)」（`crates/singlefs-core/src/transaction.rs:7874`） | 删除的一批分支 | 同在 `ordinary_allocation_part` 里，一次调用走一个分支 | 同上，轮数加一次 |
| 「previous.file_other_than_the_first(inode).map_or(0, |file| {」（`crates/singlefs-core/src/transaction.rs:10262`） | `next_unlink_step_after` | 「let (txg, deletion_and_lineage) = next_unlink_step_after(」（`crates/singlefs-core/src/transaction.rs:10218`、`crates/singlefs-core/src/transaction.rs:10530`） | 每个 unlink 步骤的发布一次 |
| 「let bytes = &previous.unit(TransactionUnit::Data(rewrite.unit)).bytes;」（`crates/singlefs-core/src/transaction.rs:12963`） | `payload_of_the_rewritten_tail_data_unit` | 「payload_of_the_rewritten_tail_data_unit(previous, rewrite)?;」（`crates/singlefs-core/src/transaction.rs:10428`）与「let payload = payload_of_the_rewritten_tail_data_unit(carried, rewrite)」（`crates/singlefs-core/src/transaction.rs:13227`） | 截断与变长的尾单元，每次发布各至多一次 |

整次发布被拒后会话重试（空间不够时推抬 F，一次准入至多「pub const PUBLISHES_PER_ADMISSION_AT_MOST: usize = 8;」`crates/singlefs-core/src/mount.rs:4164` 次）是整条发布路径再走一遍，次数仍是常数。

### `mount.rs` 的 5529 行：判成「挂载路径」只对一半

「.map(|role| (*role, output.unit(*role).slot))」（`crates/singlefs-core/src/mount.rs:5529`）在 `placements_taken_by_a_file_version` 里按 `rewritten` 的每个角色在 `units` 上线性找，R 个角色 × P 个单元。它的调用者：「.map(placements_taken_by_a_file_version)」（`crates/singlefs-core/src/mount.rs:4862`）与「taken_by_every_publish.push(placements_taken_by_a_file_version(&rehearsed));」（`crates/singlefs-core/src/mount.rs:5005`），都在抬 F 那一串里（`raise_the_floor_through`）。抬 F 不只在挂载：会话在空间不够时经「match push_one_floor_raise_within_the_admission_budget(」（`crates/singlefs-core/src/mounted_session.rs:3494`）推一串，卸载也走它。所以 5529 行不是发布路径，但也不只在挂载时走，是「挂载、卸载、会话为空间推抬 F」三处，每次准入至多 8 串空发布各走一遍。R 是一次空发布重写的角色数（我没量，没起抬 F 的场景）。判成「不在七·6 射程」我同意，判成「挂载路径」不准确。

### 漏在七处与 8 处之外的按项线性查找：有

读 `transaction.rs` 里 `.find(`、`.position(`、`.contains(`、`.any(` 共 73 处（`grep -nE '\.find\(|\.position\(|\.contains\(|\.any\(|\.rposition\(' transaction.rs` 去掉注释行、取 `mod tests` 的 15006 行以前），逐处看过调用者，挑出发布与整理路径上、对本次重写的每个角色（或每个单元）各找一次的几处：

1. **`publish_admitted` 里对 `units` 按角色线性找，两处**：「.find(|unit| unit.identity == *identity)」（`crates/singlefs-core/src/transaction.rs:14576`，`written_units`，对 `rewritten` 的每个角色）、「.find(|unit| unit.identity == identity)」（`crates/singlefs-core/src/transaction.rs:14583`，`named_unit_of`，对每个点名的角色）。`units` 是这一版全部角色的单元，在「let mut units: Vec<PublishedUnit> = Vec::new();」（`crates/singlefs-core/src/transaction.rs:14180`）建、按 bump 次序排（数据单元在前，树节点在后）。这是 R × P，不是七处也不是 8 处。
2. **点名列表上的 `.contains`**：「.filter(|identity| !data_roles_named_before_the_last_record.contains(identity))」（`crates/singlefs-core/src/transaction.rs:11713`），对每个重写角色在「前 N−1 个数据角色」里找，N × N。
3. **留存角色清单上的 `.contains`，两处**：「.filter(|identity| !kept_for_the_snapshots.contains(identity))」（`crates/singlefs-core/src/transaction.rs:4338`，释放判定路径；`crates/singlefs-core/src/transaction.rs:4820`，`mapped_roles_released_by_this_publish`），快照下整份重写时清单里有 N 个角色，换下的也有 N 个。
4. 「!previous_roles_replaced.contains(&role)」（`crates/singlefs-core/src/transaction.rs:11619`），整理一批里每个搬迁项在上一版换下的角色清单里找，K × R；这一处我没起读数。

我在自己的副本上给 1–3 加了比较次数计数（`tools/instrument.py`，只写自己的副本），整份重写一个文件 n 个数据单元，单线程跑 `--exact … --test-threads=1`（并行时两条读数用例共用计数会串数，第一次并行跑的读数作废）。原样输出：

```text
PROBE scenario=whole_rewrite_no_snapshot n=32 units=48 rewritten=46 written_find=1127 named_find=1127 roles_named_contains=2883 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=whole_rewrite_no_snapshot n=64 units=80 rewritten=78 written_find=3159 named_find=3159 roles_named_contains=8883 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=whole_rewrite_no_snapshot n=128 units=144 rewritten=142 written_find=10295 named_find=10295 roles_named_contains=30099 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=whole_rewrite_no_snapshot n=256 units=276 rewritten=274 written_find=37949 named_find=37949 roles_named_contains=112455 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=whole_rewrite_under_a_snapshot n=64 units=81 rewritten=77 written_find=3097 named_find=3097 roles_named_contains=8694 kept_contains_release=5440 kept_contains_mapped_roles=5780
PROBE scenario=whole_rewrite_under_a_snapshot n=128 units=145 rewritten=141 written_find=10169 named_find=10169 roles_named_contains=29718 kept_contains_release=19008 kept_contains_mapped_roles=19668
PROBE scenario=whole_rewrite_under_a_snapshot n=256 units=280 rewritten=276 written_find=38528 named_find=38528 roles_named_contains=113985 kept_contains_release=108337 kept_contains_mapped_roles=73098
PROBE scenario=relocation_batch_of_4 n=8 units=87 rewritten=10 written_find=815 named_find=815 roles_named_contains=0 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=relocation_batch_of_4 n=16 units=159 rewritten=10 written_find=1535 named_find=1535 roles_named_contains=0 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=relocation_batch_of_4 n=32 units=307 rewritten=14 written_find=4193 named_find=4193 roles_named_contains=0 kept_contains_release=0 kept_contains_mapped_roles=0
PROBE scenario=relocation_batch_of_4 n=64 units=599 rewritten=16 written_find=9422 named_find=9422 roles_named_contains=0 kept_contains_release=0 kept_contains_mapped_roles=0
```

读法（n 是数据单元数；relocation 行的 n 是池里的文件数）：整份重写的 n 翻倍，`written_find` 从 n=64 到 n=256（四倍）涨 12.0 倍（3159 → 37949），n=128 到 n=256 涨 3.69 倍，`roles_named_contains` 同阶涨 12.7 倍（8883 → 112455）；随 n 的平方涨，不是线性。快照下整份重写，留存清单上的 `.contains` 从 5440 涨到 108337（19.9 倍）。整理的一批（4 项）上，`written_find` 随池大小只按 R × P 涨（R 是 10–16 个重写的角色，P 从 87 到 599，815 → 9422），是线性，不是平方——所以在七·6 点名的整理一批上这几处不是平方项，在用户整份写一个大文件的发布上是。

结论：8 处判得对；5529 判成挂载路径只对一半；漏网有，上面 1–3 在整份写大文件的发布上随写入量平方涨（`.claude/rules/fs-design.md@记账是事务的副产品，不是事后的遍历` 那张表对运行时决策路径写「不许，且代价不许随盘容量增长」），在整理一批上只是 R 次 O(P)。这几处是否并进七·6、记进「发布不再整版照抄」那一行，由主 agent 判。

另见两处读码时见到、没起读数的：`take_the_mapping_positions_over_the_location_hints` 对 `mapped_units` 的每把有目的地的 key 在 `units` 上线性找（「.find(|unit| unit.identity == *identity)」`crates/singlefs-core/src/transaction.rs:3767`），整理发批对上一版拷贝调它（「take_the_mapping_positions_over_the_location_hints(&mut relocated_version, |key| {」`crates/singlefs-core/src/transaction.rs:13143`），只有搬迁的那 K 把 key 有目的地，K × P；从盘上重建一版时 `recovery.rs` 对每把 key 调（「take_the_mapping_positions_over_the_location_hints(&mut rebuilt, |key| {」`crates/singlefs-core/src/recovery.rs:4147`）是 P × P，在挂载路径上，不在发布路径。区域扫描 `classify` 每条分配记录两遍扫全部映射条目（「.any(|entry| entry.key == header.mapping_key);」`crates/singlefs-core/src/region_scan.rs:311`），我没追调用次数。

**推翻它**：8 处里有一处被我漏看的调用者在按项循环里；或在不加计数的冻结副本上，整份写 n 单元的发布里 14576、14583 两行的比较次数不随 n 平方涨（我的读数是加了计数的副本量的，加的计数只在 `find` 闭包里加一，不改控制流）。
## T3　计数与新用例判的是不是产品调用点上的事：部分成立

### 五条新快档用例数的是什么

| 用例 | 数的是 | 是产品调用点吗 |
|---|---|---|
| `a_whole_rewrite_of_one_file_looks_up_each_replaced_and_carried_unit_through_indexes_built_once_at_every_site` | 释放三处：直接调三个公开入口（「placements_to_release_via_mapping_except_the_units_kept_for_the_snapshots(」`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1465`、「refuse_mapping_entries_that_do_not_name_two_pool_devices(」`:1477`、「copies_failing_the_release_checksum_check(」`:1487`），数记在入口里的 `entering`（`crates/singlefs-core/src/transaction.rs:4331`、`:4897`、`:4973`）；照抄三处与快照判定圈：真发一次 `publish_sequential_write_of_an_inode`（`:1309`） | 是。三个入口就是发布路径调的那三个函数（`crates/singlefs-core/src/transaction.rs:11234`、`:11244`、`:11460`）；用例传的是 9 个角色、留存清单 `&[]`、`NotRewrittenWhole`（`:1468`、`:1470`），与发布时的实参不同 |
| `a_whole_rewrite_under_a_snapshot_and_its_destruction_look_up_kept_mapping_entries_through_indexes_built_once` | 快照判定圈：真发一次重写；删快照：真经会话发 `DestroySnapshot`（「.publish_user_change(&mut pool.devices, UserChange::DestroySnapshot { snapshot })」`:1662`） | 是 |
| `relocation_batches_of_two_and_of_eight_units_build_their_indexes_once_and_look_up_each_planned_unit_twice` | 真发 `publish_relocation_batch`（`:1764`），数记在核源落点那一处（「let _inside_the_relocation_sources = lookup_work::entering(LookupSite::RelocationSources);」`crates/singlefs-core/src/transaction.rs:9784`） | 是 |
| `first_entry_of_each_identity_keeps_the_first_entry_of_identities_repeated_across_a_long_shuffled_table` | 合成表上直接建 `FirstEntryOfEachIdentity`，与线性找比 | 不是调用点，判的是索引这个类型本身（正推 M1） |
| `lookup_work_counts_one_entry_per_step_of_a_walk_from_either_end_and_one_lookup_per_search` | 合成表上直接读计数器 | 不是调用点，判的是计数器本身（攻方 RG） |

数在三个来源里都是产品代码自己 `count(...)`（索引建、查、看过几项、解叶）记的，用例不另建索引来数；期望值里用到的叶数 `routed_leaves` 来自产品的 `leaf_position_routed_to`。这符合 `.claude/kb/milestone/04-fourth-txn.md:142` 写的「增长判据与变异数产品调用点上的次数，不数测试自己另建的索引」。

### 19 行变异各自红的理由（终稿上重跑）

我把 `crates/mutations.tsv` 里 19 行「七·6 第三轮」逐行打在冻结副本的拷贝上（`tools/run_mutants.py`，每行原文都恰好命中一次，`tools/check_mutation_rows.py` 核过：19 行命中 1 次、替换文与原文不同、点名的测试在测试文件里有且只有一条），跑点名的那一条（`--exact`）。基线（未打变异）点名用例绿；19 行全红：

| 行 | 红在哪条断言 → 读数 |
|---|---|
| 核源落点：找留存条目退回线性 | 核源落点恰好等于（「(index_builds_of_the_batch, routed_leaves, 2 * items + routed_leaves)」`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1796` 起）：2 项那批建 2 份（期望 3）、查 4 次（期望 5） |
| 核源落点：取计划项字节退回线性 | 同一断言：建 3、查 4（期望 3、5） |
| 删快照：取留存条目退回线性 | 「released >= 4」那条（`:1673`）：查 0 次、建 2 份 |
| 释放：按 inode 号找文件退回线性 | 「(1, replaced_roles),」（`:1528`）：建 0、查 0（期望 1、9） |
| 照抄：extent 树节点退回线性 | 整数遍断言「(0, 0, carried),」（`:1565`）：照抄 47 个、查 0 次（0 遍） |
| 照抄：下段节点按 inode 找文件退回线性 | 同上：47 个、查 0 次 |
| 照抄：多层码 2 树节点退回线性 | 同上：1 个、查 0 次 |
| 攻方 RA：落点逐 key 查 | 核源落点恰好等于：查 4（期望 5），建 3 |
| 攻方 RB：按角色读者落点逐 key 查 | 释放判定路径那一处恰好等于：解叶 9 次（期望 2） |
| 攻方 RC：每项现建索引与读者 | 核源落点恰好等于：建 5 份（期望 3） |
| 攻方 RD：每查一个角色现建两份索引 | 释放判定路径恰好等于：建 18、查 18（期望 2、11） |
| 攻方 RE：快照判定圈每个角色现建索引 | 「&& snapshot_judgement.index_lookups >= 8 * snapshot_judgement.index_builds,」（`:1543`）：建 24、查 24 |
| 攻方 RF：核两条位置项退回线性 | 核两条位置项那一处恰好等于：建 0、查 0、解叶 9 |
| 正推 M2：快照判定圈进 deadlist 那支退回线性 | 删快照那条用例的整数遍断言：不是 (3, 1, 17) 的整数倍，实际 (3, 24, 24) |
| 正推 M3：释放判定路径退回线性 | 释放判定路径恰好等于：建 0、查 0、解叶 9 |
| 正推 M4：读盘核校验和退回线性 | 读盘核校验和那一处恰好等于：同上 |
| 正推 M7：每项另建一份 | 核源落点恰好等于：建 7 份（期望 3） |
| 正推 M1：稳定排序换不稳定 | 长表那条：身份 0 取到 (0, 256)，表里最先是 (0, 0) |
| 攻方 RG：不记「看过几项」 | 计数器自己那条：从前往后走 512 项，没记看过 |

**证红之后改的类型别名与三处 clippy 写法**：上面是在终稿（含这几处改动）上跑的，19 行全红，没有一行变成不红。改的三处是 `explicit_counter_loop`（`pool_with_files_written_once` 里 `zip`）、`manual_checked_ops`（「.checked_div(builds_of_one_pass)」`:1407` 连「.unwrap_or(work.index_lookups / lookups_of_one_pass);」`:1408`）、`int_plus_one`；`checked_div(0)` 得 `None` 再取查的次数除出的遍数，与「走一遍不建索引的按查的次数算」同义，`unwrap_or` 先求值的那个除数前面断言过 ≥ 1（`lookups_of_one_pass >= 1`）。

### 按批大小取样分不分得出三类

分得出。同一批用例里 2 项（1 进映射 + 1 留存）与 8 项（4 + 4）的核源落点都判恰好等于 (3, 叶数, 2 × 项数 + 叶数)（「relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_two, 1);」`:1836`、「relocate_a_batch_and_check_the_relocation_sources(&mut pool, &planned_batch_of_eight, 4);」`:1844`）：每项重建（RC、M7）建索引份数涨到 5、7；逐 key 查（RA）少一次按角色查单元；改回线性（前两行）少一次查、少一份索引。解叶次数在这个池里每批只有 1 片叶（`routed_leaves=1`），所以「逐 key 解叶」靠查的次数分，不靠解叶次数分。

### 整数遍判法会不会让改坏的写法凑成整数遍

会，有两类，都是不坏性能的：
- 每遍工作量乘一个整数（同一处按角色多查一遍）：`assert_the_work_is_whole_passes_of` 接受任意遍数（「passes >= 1」`:1410`）。我打了「照抄 extent 树节点按角色查两遍」，整份 `publish_lookup_by_identity` 10 条非 ignore 用例全绿（见下表）。
- 发布算定点多走几轮：遍数不设上界，轮数变多也是整数遍。
这两类是常数倍，不是随表长涨的，所以不算漏掉七·6 要抓的东西。凑成整数遍而改坏查法的情形（只有一部分遍数走线性）要改坏的写法在各遍里行为不同才造得出，七处的写法每遍同一段代码，我造不出。

### 三个没被守住的缺口（我打的变异，整份 `publish_lookup_by_identity` 的 10 条非 ignore 用例全绿；1 条 ignore 的增长慢档没跑）

| 变异 | 结果 | 为什么不红 |
|---|---|---|
| 删快照：每条放掉的条目现建留存条目索引（`.get_or_insert_with(…)` 换 `.insert(…)`，`crates/singlefs-core/src/transaction.rs:6209`） | 10 passed，绿 | 删快照那一处只判不等式：「released >= 4」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1673`）、「&& kept_entries_released.index_lookups >= released」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1674`）、「&& kept_entries_released.index_builds >= 1,」（`crates/singlefs-harness/tests/publish_lookup_by_identity.rs:1675`）；每条重建时建的份数等于放掉的条数，这三句都成立。通用的折半查找上界（`:1665`）也不看建了几份 |
| `mapped_roles_released_by_this_publish` 里每问一个角色现建按 inode 号的文件索引（`crates/singlefs-core/src/transaction.rs:4854`） | 绿 | 用例只对释放判定路径那一处判「按 inode 号找文件」的建与查（「release_judgement.of_the_site(LookupSite::FilesOfTheReplacedRolesInThePreviousVersion);」`:1522`）；核两条位置项、读盘核校验和、发布时的 `mapped_roles_released_by_this_publish` 里的文件索引（建表在 `crates/singlefs-core/src/transaction.rs:4817`）没人判 |
| 快照判定圈里每问一个角色现建按 inode 号的文件索引（`crates/singlefs-core/src/transaction.rs:6128`） | 绿 | 同上：这一处的文件索引记在 `FilesOfTheReplacedRolesInThePreviousVersion`，快照判定圈的断言只数 `SnapshotJudgement` 那一处 |
| 核源落点：留存条目索引不等第一次要找就在建按 key 的索引时一并建（`crates/singlefs-core/src/transaction.rs:9981`「kept_mapping_entries_by_key_built_at_the_first_lookup: None,」） | 绿 | 「let index_builds_of_the_batch = if kept_items == 0 { 2 } else { 3 };」（`:1795`）里「没有留存项就建 2 份」那一支，两批取样（`:1836`、`:1844`）的留存项都不是 0，没人走到 |
| 照抄 extent 树节点：按角色查两遍 | 绿 | 整数遍，见上 |
| 照抄 extent 树节点：索引照查，另在 `units` 上线性找一遍丢掉结果 | 绿 | 计数只数经索引的查，旁边多一遍线性找不进计数——这是计数这个办法的结构限制，七·6 第三轮判决第三节第 1 条要的「每处进计数、每处一条改回线性的变异」管得到改回去，管不到「多加一遍」 |

**推翻它**：把上表任一行的变异打进冻结副本的拷贝后，整份用例里有一条红。读数在 `out/rows-extra.tsv`；复跑 `tools/rerun.sh`。
## T4　`singlefs-core` 加 `links` 与构建脚本：成立

**产品构建与发布构建行为不变**：「links = "singlefs-core"」（`crates/singlefs-core/Cargo.toml:10`）只声明；`build.rs` 只往标准输出打两行：「"cargo::metadata=enabled_features={}",」（`crates/singlefs-core/build.rs:15`）与「println!("cargo::rerun-if-changed=build.rs");」（`crates/singlefs-core/build.rs:18`）。没有 `cargo::rustc-link-lib`、`cargo::rustc-link-search`、`cargo::rustc-cfg`、`cargo::rustc-env`，也不生成代码，所以不链接任何库、不改编出来的代码。我在自己的副本上编过：

```text
$ cargo build -p singlefs-core --offline -vv     （编译目录 target-after）
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.79s
$ cat target-after/debug/build/singlefs-core-310c941798beea37/output
cargo::metadata=enabled_features=
cargo::rerun-if-changed=build.rs
```

我没编 `--release`，两个 profile 里这个构建脚本是同一份代码、读的是同样的环境变量；`rust-version` 在本仓任何 `Cargo.toml` 里都没写（`grep -rn 'rust-version' Cargo.toml crates/*/Cargo.toml` 零命中），工具链是 cargo 1.98.0，`cargo::` 双冒号写法要 1.77 起。

**交出的特性名是 cargo 并过之后的那一份**：构建脚本读的是 cargo 给它的环境变量（「.filter_map(|name| name.strip_prefix("CARGO_FEATURE_").map(str::to_string))」`crates/singlefs-core/build.rs:11`），那些变量是 cargo 按这一次编译里本包并过之后的特性设的。我造了一个探针工作区（`singlefs-core` 用路径依赖）：一个包 `consumer` 普通依赖 `singlefs-core`、带构建脚本打印 `DEP_SINGLEFS_CORE_ENABLED_FEATURES`；一个包 `devfeat` 仿 harness：普通依赖加 dev 依赖里写 `features = ["lookup-work-counters"]`（「singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters"] }」`crates/singlefs-harness/Cargo.toml:18` 的形状）。原样输出：

```text
$ cargo build -p consumer --offline
warning: consumer@0.1.0: consumer sees DEP_SINGLEFS_CORE_ENABLED_FEATURES=Ok("")
$ cargo test --no-run -p consumer -p devfeat --offline
warning: consumer@0.1.0: consumer sees DEP_SINGLEFS_CORE_ENABLED_FEATURES=Ok("LOOKUP_WORK_COUNTERS")
$ cargo build -p consumer --offline        （再单编 consumer）
warning: consumer@0.1.0: consumer sees DEP_SINGLEFS_CORE_ENABLED_FEATURES=Ok("")
```

`singlefs-core` 的构建脚本输出按单元各在自己的目录里：`…-310c941798beea37`（上面单编核心层）与 `…-fbdf7caf95e60f6d`（探针工作区里单编 `consumer`）是 `enabled_features=`，`…-44e6f8c59115aead`（`consumer` 与 `devfeat` 一起编）是 `enabled_features=LOOKUP_WORK_COUNTERS`。即：harness 档的 dev 依赖打开的 `lookup-work-counters` 在里面，且只在一起编的那一次里；空串 cargo 收下（直接依赖方读到 `Ok("")`）。依赖方怎么把名字还原成清单里的特性名：`environment_name_of` 大写加 `-` 换 `_`（`crates/singlefs-unit-graph/src/build_features.rs:93`），认不出的名字报 `UnrecognizedFeature`（「return Err(BuildFeaturesError::UnrecognizedFeature {」`crates/singlefs-unit-graph/src/build_features.rs:193`）。

**`rerun-if-changed=build.rs` 会不会让特性变了而元数据没重算**：不会。上面三次编得出三个不同的输出目录，cargo 按本包的特性集分开建目录、各自跑一次构建脚本；`rerun-if-changed` 只管文件改动要不要重跑，特性变了是另一个单元，不靠它。第三次再单编 `consumer` 时读到的是空串，不是上一次的 `LOOKUP_WORK_COUNTERS`，也是证据。

**一处边角，不是错**：构建脚本把环境里全部 `CARGO_FEATURE_` 开头的变量都收进去，包括不是 cargo 给的。我在新的编译目录里 `CARGO_FEATURE_BOGUS=1 cargo build -p consumer`，输出文件是 `cargo::metadata=enabled_features=BOGUS`。依赖方遇到清单里没有的名字会报错（上面 `UnrecognizedFeature`），checker 档构建脚本把它当 panic（「依赖包在这一次编译里开了哪些特性取不全」，`crates/singlefs-checker-tier/build.rs:83`），所以是响亮地失败，不会静默算错键；而且这份输出留在编译目录里，之后环境变量去掉了也要等构建脚本重跑才会清掉，因为 `rerun-if-changed` 里没有 `rerun-if-env-changed`。

**直接依赖 `singlefs-core` 的别的包同名 `links` 撞不撞**：不撞。`links` 在冻结副本整棵树里只有这一处（`grep -rn '^links' --include=Cargo.toml .` 只命中「links = "singlefs-core"」这一行）；直接依赖它的两个包 `singlefs-harness`（没有构建脚本）、`singlefs-checker-tier`（有构建脚本，读元数据）都没声明 `links`。`scripts/experiment-apparatus/e176_crates_oracle/Cargo.toml:20` 另一个工作区里按路径依赖同一个包，同一个包身份不撞。`links` 同名冲突只在同一个依赖图里有两份不同的包身份时才出：本仓没有。

**推翻它**：`cargo build` / `cargo test` 在工作区里报 `links` 冲突或「package … specifies that it links to `singlefs-core` but does not have a custom build script」；或同一个依赖图里出现第二份 `singlefs-core`（不同路径）；或 `DEP_SINGLEFS_CORE_ENABLED_FEATURES` 在 harness 与 checker 档一起编时读到的不含 `LOOKUP_WORK_COUNTERS`。本轮探针里没出现。
## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 七处改法的行为 | 成立 | 七处索引取表里最先一项，与 `iter().find` 同一项；单元索引挪早前后建在同一份 `previous` 上；panic 文案与两个 `Err` 的条件不变；两串池的 `images_sha256` 改前改后逐字相同 |
| T2 8 处单次调用 | 成立 | 逐处追到调用者，都是每次发布常数次（含固定点每轮一次），没有在按项循环里的 |
| T2 `mount.rs` 5529 行 | 部分成立 | 不是发布路径，但抬 F 那一串在挂载、卸载与会话为空间不够推抬 F 时都走，不只挂载 |
| T2 有没有漏网 | 有 | `publish_admitted` 里 `units` 按角色线性找两处（14576、14583）与几处角色清单 `.contains`，整份写 n 单元的发布随 n 平方涨（n 四倍涨 12 倍）；整理一批上只是 R 次 O(P) |
| T3 数的是不是产品调用点 | 部分成立 | 三条用例数的是产品调用点；两条合成表用例判的是索引、计数器这两个类型本身；19 行变异在终稿上全红；三个缺口（删快照每条重建、文件索引在两处调用方每角色重建、无留存项的那一支）变异全绿；常数倍与「多加一遍线性找」按计数的办法看不到 |
| T4 `links` 与构建脚本 | 成立 | 只交元数据；cargo 并过的特性（含 dev 依赖打开的）都在，单编时为空串；按特性集分开输出；`links` 不撞；一处边角：环境里的 `CARGO_FEATURE_*` 会被收进去，依赖方遇到会响亮地失败 |

## 没做什么

- 不替主 agent 采纳或出判决；攻方腿的格不碰；没读禁读清单里的任何文件。
- 没跑增长慢档（`publish_lookup_by_identity` 里 1 条 ignore 的整理增长用例）、checker 档测试、门禁阶段、崩溃与变异轻型、GPU；没编 `--release`；没动冻结副本（所有改动都在自己的拷贝里，拷前 `find crates -type f -exec touch {} +`）。
- 没量抬 F 那一串空发布的 R；没追 `region_scan.rs` 区域扫描每条分配记录两遍扫映射条目的调用次数；没读 checker 档构建脚本与 `singlefs-unit-graph` 的整套判法，只核了 `singlefs-core` 这一侧交出什么、依赖方怎么还原名字、认不出时怎么报。
- 「改前改后盘镜像逐字节相同」只验了两串固定操作（16、48 个文件），没验任意合法操作。
- 协调方在我干活时来过一条消息：本轮不往第二台挪、本机排不上就排队或 `nice -n 19`。我的活一开始就全在本机（内存包装里排队），没起第二台的活，所以不用改。
- 要主 agent 跑的命令：无。

## 产物（`out/`，复跑 `tools/rerun.sh` 打出的原样，约 36 分钟）

```text
c30429939b2380920cf65de6ae7bd6985b4f0356c081fa130a23642711c8e691  out/baseline.txt
4bb20fba0bc1279301a61ec28b743273567f2049e94b4fcaf622fc86fffa22db  out/check-mutation-rows.txt
ad3389b5c12c11b3e3f6648622b0463fedee6ab67eb025b727eb8400e1d2acc7  out/equal-images.txt
1308e1ca96207b28e3e469233d22b6f6bd382954b1a6beea3ce1f74af7eeea30  out/probe.txt
36bd5be592476c0582ba5ebc03f9206a7a92c5dbb2c1a0c39602175bbe7a9b0e  out/rows-19.tsv
72772e8051d957b3b8e428d276cf041c0b66ef826decf5ca780209a70f7e476d  out/rows-extra.tsv
```

复跑打出的结局行（节选）：`name=baseline test result: ok. 5 passed; 0 failed`；`name=row idx=0 outcome=GREEN`（基线）、`idx=1…19` 全 `RED`；`name=extra idx=1…6` 全 `GREEN`（`10 passed; 0 failed; 1 ignored`）；`name=equal side=after/before` 两串摘要逐字相同。
