# 代码欠账批甲簇代码轮：正推腿（Sonnet）报告

日期：2026-10-14。轮名：code-backlog-b2-code-r1。立场：正推（核「代码与测试做的是不是条款说的那件事」）。格：K1、K2、K3、K4。

被判代码只取冻结副本 `/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/`（下称冻结副本）；引代码写冻结副本的绝对路径加行号，引 kb 写 kb 文件名加那份文件自己的行号（我在 kb 文件里 `grep -nF` 现查的）。

## 0　开工前复核（量的）

背景材料的「实现今天的样子」写了三份 harness/core 文件与三份 kb 的 sha256；我对冻结副本与主工作区 kb 各算了一遍，与快照一致：

```
$ sha256sum crates/singlefs-core/src/mount.rs crates/singlefs-harness/src/history.rs crates/singlefs-harness/src/model_comparison.rs   （冻结副本里）
975e77aff09e7f56f8b3cd7af6173fcc82e31f144349dd77850c44668c51c49a  mount.rs
fe1a8e3cd3e7bc3ac876dc7fa28b5813bd51013c6158712a5226f97f9f754dbd  history.rs
89afae46498091ea13dab0935d83965892ed6c61f28e33775eb5ae35d14e74dd  model_comparison.rs
$ sha256sum .claude/kb/decisions/23-journal的角色与格式.md .claude/kb/decisions/19-块指针的结构与宽度预算.md .claude/kb/checks-owed.md crates/mutations.tsv   （主工作区里）
565620bcead49240809b3788d09345f1e339007874b22207006c45f8b4cc77ef  23-journal的角色与格式.md
0e9029723d43752aa6d42fdbb271369c3ae3cccbfdbf556bbf4cd53cd15e9a6f  19-块指针的结构与宽度预算.md
cfea72c9bb015cb5dc6b3c8501d258c15c41add96d45cc52aa1142897f04eacd  checks-owed.md
8ec2c98d26301fab7f09c33e2a9abdd577a82c0d093da874653136d9022354d0  mutations.tsv   （主工作区这份与快照同值；变异表我读的是冻结副本里那份，同值）
```

干活的方式：冻结副本拷两份（基线、变异用），各用自己的编译目录，编译与测试都经 `scripts/hosts/run-with-memory-cap.sh 16G` 加 `capped.sh`，同一时刻一条 cargo。跑过的只有 `singlefs-core` 的 `--lib` 用例与 `singlefs-harness` 的三个测试文件（`rollback_by_a_forward_publish.rs`、`rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs`、我加的探针文件），没起门禁，没跑 checker 档，没跑全 harness 档。

模型目录 `research/prompts/code-backlog-b2-code-r1-sonnet-model/`：`tools/rerun.sh` 一条命令复跑本报告里全部「量的」数（S1–S6 六组，输出写进新草稿目录的 `out/`，与模型目录 `out/` 逐行比）：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/code-backlog-b2-code-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录>
```

`tools/rerun.sh` 分 S1–S8 八组（S1 交付的六条 B3 用例，S2 追加表 B3 五处改回去，S3 探针 P1、P3，S4 harness 交付的回退测试文件含慢档，S5 去掉 C572 那一步，S6 K2 探针，S7 整行替换八行逐行跑，S8 替换第 1 行换到 core 用例上），共 22 份 `out/*.txt`；整套跑过四遍（前两遍是只有 S1–S6 的旧版，后两遍是 S1–S8 的定稿版）；定稿版第二遍末行 `name=rerun_compare differing=0`（22 份与第一遍逐行同；耗时去掉、按行排序后比）。

## 条款原文（本报告反复用到的几句，kb 自己的行号）

- 「只隔离其中**只被被抛弃根引用**的槽」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）——B3 的条款句。
- 「同样逐盘验，有一份读不出或对不上就拒（用户定案，原话在变更史）」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）与同一行的「其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）——C572 的条款句与它的射程句。
- 「复活集里的每个单元（cur 的账记成已释放、这一次改回已分配的那些）在每块盘上的那一份都要读得出、校验和对得上，有一份不行就拒」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）——复活集那一判。
- 「隔离单元（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：任一份核出对不上、两份都留在已分配的）除外，留在已分配」（`.claude/kb/decisions/23-journal的角色与格式.md:383`）——「释放」一格怎么认隔离单元。
- 「核出对不上时隔离那个槽，发布照成」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`）与「任一份核出对不上，这个单元在每块盘上的分配记录都留在」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）。
- C572 立账行：「回退目标引用的隔离单元也逐盘验」（`.claude/kb/checks-owed.md:377`）；C554 ③ 的判法：「照条款改成按槽展开，进代码批，不交岔路」（`.claude/kb/checks-owed.md:365`）。

## K1　B3 五处按槽展开，撑不撑得住「只隔离其中只被被抛弃根引用的槽」

**判定：对得上**（五处都按槽、六条用例各钉一处，改回去各自红）。**另有两条没钉到的东西，不算对不上，但要交主 agent**：豁免先于判重的次序没有用例钉住（量的）；分配位置着的槽被两条被抛弃根交叠罩着时，改后比改前少隔离一槽（量的，能不能走到是推的，条款没写这一格，判不了对不对）。

### K1.1　五处各是什么、改回去哪条用例红（量的）

条款：「只隔离其中**只被被抛弃根引用**的槽」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）；C554 ③ 把五处认成「按起点槽认的共五处」并判「照条款改成按槽展开，进代码批，不交岔路」（`.claude/kb/checks-owed.md:365`）。

按槽展开的两个公用件：现行账那一份集合的收法是「slots_covered_by_a_placement(record.device, record.slot, u64::from(record.span_slots))」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1577`）；判重与豁免合在一处：「.filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1752`），交回的槽逐槽交给分配器，每槽的跨度是「const SPAN_OF_ONE_SLOT: u64 = 1;」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1723`）。

| 处 | 冻结副本里的位置 | 追加变异（`crates/mutations.tsv`，冻结副本里同值）改成什么 | 改回去后红的用例（原样输出在模型目录 `out/s2-b3-mutation-N.txt`） |
|---|---|---|---|
| ① 现行账集合 | 「slots_covered_by_a_placement(record.device, record.slot, u64::from(record.span_slots))」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1577`） | 只收记录起点槽 | `a_current_account_record_exempts_every_slot_it_covers_from_the_placements_of_an_abandoned_root_read_from_its_account`（只此一条红，5 过 1 红） |
| ② 候选根集合 | 「slots_covered_by_a_placement(*device, *slot, *span_slots)」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1591`） | 只收候选根落点的起点槽 | `a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement`（只此一条） |
| ③ 被抛弃根那一圈 | 「for slot_to_isolate in slots_of_a_placement_to_isolate_now(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1645`），豁免参数「&referenced_by_candidates,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1649`） | 起点槽被豁免或已隔离就整份跳过 | 三条红：`two_abandoned_roots_sharing_a_start_slot_isolate_both_slots_of_the_wider_placement_in_either_read_order`、`a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_an_abandoned_roots_wider_placement_isolated`、①的那一条 |
| ④ 根槽读坏的被抛弃根 | 「for slot_to_isolate in slots_of_a_placement_to_isolate_now(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1675`），豁免参数「&referenced_by_candidates,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1679`） | 同③ | `a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_recorded_for_an_abandoned_root_whose_ring_slot_read_bad_isolated`（只此一条） |
| ⑤ C 那一圈 | 「for slot_to_isolate in slots_of_a_placement_to_isolate_now(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1701`），豁免参数「&referenced_by_the_current_account,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1705`） | 同③ | `a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_a_placement_named_by_unapplied_records_isolated`（只此一条） |

交付的样子六条全绿（`out/s1-b3-delivered.txt`：「test result: ok. 6 passed」）。五处一一对得上条款的「槽」，没有第六处遗漏：我在 `mount.rs` 与 `allocator.rs` 里 grep 了 `(device.0, slot.0)`、`record.slot.0)`、`BTreeSet<(u32, u64)>`，按起点槽收集合的写法只剩 `sort_key` 里的「(self.device.0, self.slot.0)」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/allocator.rs:76`，排序用，不是豁免或判重）。

### K1.2　三形与 C554 那一轮攻方的三形是同一件事（读码加对照）

攻方报告：「形一：两条被抛弃根 A1 (s, 跨 1)、A2 (s, 跨 2)」（`research/prompts/m3-c554-c572-r1-opus-output.md:181`），形二是现行账 (s, 跨 1) 加被抛弃根 (s, 跨 2)，形三是 C 那一圈 (s, 跨 2) 加现行账 (s, 跨 1)。交付用例的对应：形一 = `two_abandoned_roots_sharing_a_start_slot_…_in_either_read_order`（两种读出次序都断言 `[(true, false), (true, false)]`，并且两次结局相等）；形二 = `a_current_account_record_at_the_same_start_slot_leaves_the_second_slot_of_an_abandoned_roots_wider_placement_isolated`；形三 = `…_of_a_placement_named_by_unapplied_records_isolated`。位置、跨度、断言（起点不隔离、后一槽隔离、发不出去）与攻方探针一致。用例另加了第四处（根槽读坏那一圈）与第一、二处（两份豁免集合）各一条，攻方只点名过五处、没给后三条的形。

### K1.3　末槽的 checked_add 与 expect 在五处来路上成立（读码）

「let end_slot = slot.0.checked_add(span_slots).expect(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1735`）依赖「进影子账的落点都判过落在单元区里」。五处的落点来路：

- 现行账记录（①与⑤的豁免）：挂载路径 `recovery.rs` 判过「allocation_records_fit_the_pool_geometry(reader, unit_area_start, &roots.allocation.records)?;」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/recovery.rs:5387`）；抬 F 路径传的是内存里现行那一版的账「&current.allocation_records,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:4566`），来源同一份账。
- 候选根与被抛弃根从盘上读账的落点：`placements_referenced_by_root` 里逐条经「placement_lies_in_the_unit_area_of_its_device(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:2555`），从映射现算的那一支经同一个判「placement_lies_in_the_unit_area_of_its_device(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:2481`）。
- 水位之上没施加的记录点名的落点：过滤时逐条经同一个判「placement_lies_in_the_unit_area_of_its_device(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1908`）。
- 根环表里记着的、根槽读坏记着的：都是上面三类在挂载那一遍交进根环表的同一批值，抬 F 那一串只是取回（`readable_roots_for_raising_the_floor` 交回的 `placements_recorded_for_abandoned_roots_whose_ring_slot_read_bad`）——这一类我没有逐行追到写表处，写「推的」。
- 类型上也走不到：槽号是盘上 6 字节（小于 2^48），跨度是单元种类给的 1 或 2（那里的注释写着「槽号是盘上的 6 字节（< 2⁴⁸）、跨度 ≤ 2¹⁶ 由调用方的字段宽给出，两个加法都装得进 u64」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/recovery.rs:2003`）），expect 在这些来路上不会触发。

### K1.4　分配器一侧对同一槽幂等，两块盘一致（读码加量）

分配器置隔离位：「if !self.isolated[index] {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/allocator.rs:614`），已置着什么都不动，所以按槽一槽一槽交、与改前整段交，落到的位图相同。判重集合的键类型是「BTreeSet<(u32, u64)>」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1573`），即 (设备号, 槽号)，两块盘各自判；从盘上读账的根每块盘各推一份同槽落点，写法是「for (identity, _) in devices {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:2551`），映射现算那一支同样是「for (identity, _) in devices {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:2480`），两块盘得到的槽序列相同（用例五里盘 0 的断言 `vec![true, false, false]` 是量的，盘 1 没断言）。

### K1.5　豁免先于判重的次序：六条用例没钉住（量的）

次序的写法在 `slots_of_a_placement_to_isolate_now` 的过滤里，先看 `exempt`，再 `isolated.insert`：「.filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1752`）；注释说「与改成按槽之前的判重同一个次序：先看豁免，再判重」。次序有后果：被抛弃根那一圈豁免掉（候选根罩着）的槽，如果先进了 `isolated`，C 那一圈（只豁免现行账）就会因为判重而不再隔离它，而条款要求 C 那一圈「只除去现行账里仍分配着的槽」（`.claude/kb/decisions/23-journal的角色与格式.md:392` 的预隔离一段）。

- 取样点一（对调次序）：把过滤改成「isolated.insert(*covered) && !exempt.contains(covered)」，交付的六条用例仍 `6 passed`（`out/s3-probes-order-swapped.txt` 里六条原有用例都 ok）。
- 探针 P1（我加的，`tools/core-probes.rs.txt`）：被抛弃根落点 (s, 跨 1) 被候选根（mkfs 那一版的实例表）罩着，水位之上没施加的记录也点名 (s, 跨 1)，现行账空。交付的样子 P1 绿（s 被 C 那一圈隔离）；对调次序后 P1 红。
- 现象：次序是对的，但钉它的东西只有探针；入库的六条没有。**不是实现偏离条款，是用例没钉条款这一格**。推翻它的现象：入库的别处用例（我没跑全 harness 档）里有一条在对调次序后红，那它不是盲区。

### K1.6　分配位置着的槽被两条被抛弃根交叠罩着：比改前少隔离一槽（量的，能不能走到是推的）

改前（起点槽判重）：同一起点的第二份落点整份跳过；起点不同、有交叠的两份落点各自走各自的。改后（按槽判重）：交叠的槽只由先过的那份隔离，后过的那份跳过。分配器那一侧有两个函数对「分配位已置着」的槽处置不同：根环表里记着的被抛弃根用 `isolate_the_slots_not_allocated…`（分配位置着的槽不隔离，回收时由根环表补），读盘得来、表里没记着的（抬 F 那一串）用 `isolate_abandoned`（不管分配位，整段隔离；源码注释写明「回收时没有一份补得上它们」）。

- 探针 P3（`tools/core-probes.rs.txt`，直调 `isolate_slots_referenced_only_by_abandoned_roots`）：盘 0 的起点 + 2 槽分配位置着（已释放、在 defer 窗口里，不在现行账未释放记录里）；被抛弃根 A 表里记着 (起点 + 1, 跨 2)，先过；被抛弃根 B 读盘得来（mkfs 那一版：实例表 (起点, 跨 2)、树表 (起点 + 2, 跨 1)），后过。交付的样子：盘 0 起点 + 0..2 的隔离位 `[true, true, false]`（起点 + 2 没隔离）；把追加变异第三处改回起点槽判重：`[true, true, true]`（`out/s3-probes-delivered.txt`、`out/s3-probes-site3-back-to-start-slot.txt`）。
- 差在哪：起点 + 2 被 B 引用、分配位置着，B 不在根环表里，回收出来时只有 A 的表项补隔离它；A 先离开根环，B 还在，起点 + 2 就可能被发出去。改前 B 自己用 `isolate_abandoned` 把它隔离了。
- 能不能走到（推的）：要抬 F 那一串里同时有一条表里记着、一条表里没记着的被抛弃根，两份落点起点不同却交叠（不同时间线的单元跨度不同、复用同一段槽），交叠槽分配位置着。我没用公开入口造出这个形；可写挂载那一遍传「PlacementsOfAbandonedRoots::ReadFromTheirAccounts,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:3159`），恒走 `isolate_the_slots_not_allocated…`，没有这个混搭，所以这个差只在抬 F 那一串。
- 条款对它说了什么：「分配器多查环里每一个可读根的账、只隔离其中**只被被抛弃根引用**的槽」（`.claude/kb/decisions/23-journal的角色与格式.md:390`）没写读盘得来的被抛弃根回收时靠谁护；代码注释写着「落点里分配位置着的槽一律不隔离」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:1696`），条款里对应的是 D28 第九项，不在我的材料里。**判不了对不对**；它是改后相对改前的一格行为差，量的，要不要算偏离交主 agent。推翻它的现象：抬 F 那一串里表里没记着的被抛弃根只出现在没有交叠的位置，或回收时另有一处补隔离罩住 B（我读到的 `placements_the_isolation_still_protects` 只看表项）。

### K1.7　六条用例没罩、公开入口能走的形（读码，没造）

- 可写挂载那一遍传「PlacementsOfAbandonedRoots::ReadFromTheirAccounts,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:3159`）：六条用例里没有一条用这一臂（用例模块里只出现「PlacementsOfAbandonedRoots::RecordedInTheRootRingTableOtherwiseReadFromTheirAccounts(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:9928`））；五处按槽展开的逻辑两臂共用，这一臂多出来的只有「恒用 isolate_the_slots_not_allocated…」，用例五用空表读盘走的是另一个分支（`reclaim_isolates_what_this_root_references` 为假）。
- 跨度三以上：用例都取跨度 1 与 2；映射现算处把单元种类换成跨度只有两个值，「crate::unit::UNIT_CLASS_DATA | crate::unit::UNIT_CLASS_PACKED => 2,」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:2471`）与下一行的索引节点 1，所以取样点没有缺口（别的来路的跨度没逐处查，读码）。
- 其余同起点不同跨度的形都在六条里；`isolated` 键里有设备号，两块盘互不判重，这一点没有专门的用例（两块盘同槽的落点在用例五里读盘得来，但只断言盘 0）。

## K2　C572 隔离单元怎么认

**判定：对得上**（在材料给的取样点上认得对；漏拒与多拒各查过）；**一条没钉到**：「现行那一版引用着就不验」这一半，入库用例都不钉（量的）；**一条判不了**：两条位置项不同槽时的认法（条款没写，我没造）。

### K2.1　代码认的是什么

`resurrection_against_the_current_account` 里，一个目标单元的两条位置项逐条对现行账：没记录就拒、仍分配而分配代不同就拒、已释放的进复活集、「Some(_still_allocated_under_the_same_generation) => {}」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:7813`）不动；之后「} else if !is_referenced_by_the_current_version {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:7821`）——这个单元没有一条位置项在复活集里、且「.any(|location| slots_of_the_units_of_the_current_version.contains(&location.slot));」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:7818`）不成立，才进隔离单元清单。`slots_of_the_units_of_the_current_version` 是「current.units.iter().map(|unit| unit.slot).collect();」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:8211`）。

条款：隔离单元 =「R_old 引用、cur 的账里仍分配而同分配代的单元里被隔离的那些（D19（块指针的结构与宽度预算） 已定项 5 硬规则 1：读盘核出一份对不上、两份都留在已分配）」（`.claude/kb/decisions/23-journal的角色与格式.md:388`），「其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」（同一行）；D19 说「任一份核出对不上，这个单元在每块盘上的分配记录都留在」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）已分配。

### K2.2　恰好等于条款说的「被隔离的那些」吗

- **漏拒方向（隔离单元被当成现行共用而漏验）**：隔离单元的槽两块盘记录都留在已分配（D19:113），不会被分配器再发出去，所以现行那一版的任何单元都不会落在这个槽上（I-5.4：同一块盘上记录罩住的槽互不相交，`recovery.rs` 的 `allocation_records_fit_the_pool_geometry` 判）。「cur 那一版恰好引用同一槽」要求现行那一版的某个单元起点槽等于这个槽，而这个槽上记录的单元是被释放的那个，两者只能是同一个单元，那它就不是隔离单元了。结论：这个方向不漏（推的，靠记录互不相交）。
- **多拒方向（不是隔离来的单元被多验）**：被多验的 = 目标单元，每条位置项都「仍分配、分配代相同」，又没有一个落在 `current.units` 的槽上。`current.units` 的内容：发布出来的版本是「这一版全部角色的单元：这次重写的是新装的，没重写的从上一版照抄」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/transaction.rs:2847`）；从盘上重建的版本在 `recovery.rs` 里逐角色装：数据单元、extent 树各节点、inode 叶容器、inode 根、分配记录树各节点、`deleted_inodes` 根（「deletion_tree_units.push(unit(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/recovery.rs:3109-3115`））、墓碑 / 谱系 / 退役各树、记账树与映射树各节点、树表、实例表；`unit.slot` 取 `locations[0].slot`。目标单元只会是 `is_a_unit_taken_from_the_rollback_target` 那一组（数据、extent、inode 树各角色加 `deleted_inodes` 根），全在 `units` 里。所以 `deleted_inodes` 队列的根单元在 `current.units` 里（读码，没造删除历史去量）。
- **量的**：我造了「现行版本与目标共用单元、其中一份坏」的历史（`src/probe_k2_shared_unit_is_not_verified.rs`：隔离修好后回退到 A 得 A 的副本 A2，再把 A2 引用的盘 1 上那一份改坏，再回退到 A）。探针一：数据单元那一份坏；探针二：A2 的数据单元、extent 根、inode 叶容器、inode 根四个角色在盘 1 上的那一份全改坏。两个探针第二次回退都做成（`out/s6-k2-probes-delivered.txt`：「PROBE-SHARED second rollback = Ok(0)」「PROBE-ALL-ROLES second rollback = Ok(0)」），没有一个角色被误认成隔离单元。`DeletedInodesRoot` 这一角色没造（需要删除历史）。

### K2.3　这一半入库用例钉不钉（量的，对不上）

入库的两条（`rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs`）一条拒（目标引用的隔离单元一份坏）、一条阳性对照（那一份写回原样，回退做成）。阳性对照对「共用单元带坏份也不验」这一半没有分辨力：它两份都对得上，验与不验都做成。

- 取样点：把「} else if !is_referenced_by_the_current_version {」改成「} else if true || !is_referenced_by_the_current_version {」（共用单元也验，多拒）。入库的两条仍绿，`rollback_by_a_forward_publish.rs` 的 25 条（含慢档）仍绿；只有我的两个探针红（`out/s6-k2-probes-guard-dropped.txt`：`IsolatedUnitCopyUnreadableOrMismatched`，数据单元、盘 1）。别的测试文件我没跑（重型测试不跑）。
- 对不上的两句并排：条款「其余 cur 的账里仍分配的单元分配器不会发出去，不在逐盘验之内」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）  ←→  代码「} else if !is_referenced_by_the_current_version {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:7821`）有这一道，入库用例里没有一条在它被去掉时红。差在用例没钉条款这一句，代码本身对。量的。
- 推翻它的现象：入库的别的测试文件里有一条在去掉这一道时红。

### K2.4　两条位置项不同槽（判不了）

代码对目标单元的两条位置项逐条处理（`record_for(location.device, location.slot)`），`.any(...)` 只要有一条位置项的槽在现行那一版的槽集合里，就算现行引用着（集合不带设备号，现行单元的槽取 `locations[0].slot`）。条款没写两条位置项不同槽的映射条目怎么认（D19:107 之后只写位置项指池外的盘、两条指同一块盘当映射条目损坏）。我没造出这样的映射条目，读码的结论：不同槽时目标单元只要有一个槽碰到现行单元的槽就不验，另一个槽上那一份不验。判不了，交主 agent。

## K3　C572 拒在什么时候

**判定：对得上**。

条款：「在任何写之前拒」（`.claude/kb/decisions/23-journal的角色与格式.md:388`）那一格；回退本身「不新增实例、不取号、不写实例表行、不施加也不删除任何记录」（`.claude/kb/decisions/23-journal的角色与格式.md:378`）。

- **次序（读码）**：入口是「pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:8109`），复活判（`resurrection_against_the_current_account`）之后依次「every_copy_of_the_resurrected_units_reads_back(&*devices, &units_to_check_on_every_device)?;」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:8223`）、「every_copy_of_the_isolated_units_the_target_references_reads_back(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:8224`）；动分配器的第一句是「let allocator_before_the_rollback = allocator.clone();」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:8290`），在新一步之后。两个验读都经 `&*devices`（只读借用）、`BlockDevice::read_at` 是 `&self`。入口到新一步之间的调用：`allocator.frozen_publish()`（读）、`caller_inputs_agreeing_with_the_disk`、`instance_table_chain_of_root`、`rollback_candidate`（传 `&ring_slots_known_to_hold_a_root_by(allocator)`）、`rebuild_version_in_the_unit_area_starting_at`、`allocation_records_under_root_in_the_unit_area_starting_at`，签名上都是只读借用。没有盘上写、没有分配器状态变化。
- **量的**：入库拒绝用例断言拒之后三样不动：盘（`disk_snapshot`）、现行版本「assert_eq!(pool.output, current_before, "现行版本不动");」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:177`）、分配器记录。`disk_snapshot` 里的「recorded_operations: stream.operations().len(),」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/tests/common/mod.rs:342`）是录制流的步数；录制设备只对写、整段清零、屏障推进流：写「self.stream.push(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/lib.rs:386`）、清零「self.stream.push(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/lib.rs:410`）、屏障「self.stream.push(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/lib.rs:425`）；读是「self.inner.read_at(offset, buffer)」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/lib.rs:372`），不记。所以「步数不变」= 一个写、屏障、清零都没发，这个核法站得住；它没有逐字节比单元区，但任何字节变化都要经过被记录的写。该用例交付的样子绿；把 C572 那一步去掉（追加变异最后一行）它红：「.expect_err("目标引用的隔离单元盘 1 那一份对不上：在任何写之前拒");」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:160`）那一句落空（`out/s5-c572-step-removed.txt`：拒绝用例 FAILED、阳性对照 ok）。
- **两种拒因同时成立报哪一个（读码，没造）**：复活集那一判先于隔离单元那一判，所以两份都坏时报 `ResurrectedUnitCopyUnreadableOrMismatched`；复活判里的 `UserVisibleUnitWithoutItsRecordInTheCurrentAccount` 与 `UserVisibleUnitStillAllocatedUnderAnotherGeneration` 在整个循环里先于任何逐盘验。条款没给拒因的先后（「在任何写之前拒」那一格只列了拒的条件），新一步接在已有拒因末尾，没有改变任何已有拒因的次序。我没能造出一份历史让一个目标同时有复活单元坏份与隔离单元坏份（A 只有一个数据单元），所以这条是读码，不是量的。
- **`first_copy_that_does_not_read_back` 抽出来后复活集那一判逐字不变（读码加量）**：改前的循环是「外层按单元，内层按位置项，每份读一次，第一份读不出或校验和不符就拒」，改后是「fn first_copy_that_does_not_read_back<Reader: PoolReader + ?Sized>(」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:7884`），里面 `units.iter().find_map`、`locations.iter().find`，次序与读法同；拒因仍由 `every_copy_of_the_resurrected_units_reads_back` 包成 `ResurrectedUnitCopyUnreadableOrMismatched`，隔离那一支的包法是「Some((unit, location)) => Err(RollbackError::IsolatedUnitCopyUnreadableOrMismatched {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-core/src/mount.rs:7854`）。量的：原来钉它的 `rollback_by_a_forward_publish.rs` 的 `refusal_before_any_write_when_data_unit_of_the_first_version_corrupt_on_device_one_…` 断言「RollbackError::ResurrectedUnitCopyUnreadableOrMismatched {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs:410`）且盘是盘 1、槽是目标那个槽，交付的样子 25 条（含慢档）全绿（`out/s4-harness-delivered.txt`）。
- 推翻它的现象：入口到 8224 行之间有一处我没列出的写或对分配器的 `&mut` 调用；或某个历史下新一步之前已经有一次录制流步数增加。

## K4　harness 补的两臂与用例、变异

**判定：两臂与用例对得上；变异：追加六行对得上，整行替换八行里有一行点名的测试不红（量的，改前也不红，不是这次改动引入的）。**

### K4.1　两臂与成员名字串

- `model_comparison.rs` 两处：「| RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/model_comparison.rs:797`）落在映射到 `Unexplained` 的那一串里，紧挨 `ResurrectedUnitCopyUnreadableOrMismatched`（第 796 行）；另一处「| RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. }」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/model_comparison.rs:989`）落在返回 `None` 的那一串，紧挨第 988 行。`history.rs` 一处：「RollbackError::IsolatedUnitCopyUnreadableOrMismatched { .. } => {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/history.rs:3164`）给出同名字串，紧挨第 3161 行的复活集那一支。三处都是与同类成员对称的补法，没有通配臂。
- 随机历史与故障注入里会不会被当成不该有的分歧（推的）：映射函数的文档写着这一类成员是「两块等大盘的历史里它们都不该出现」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/model_comparison.rs:7`）；模型里没有隔离这个概念（`grep -c -i quarantin crates/singlefs-harness/src/model.rs` 输出 0），所以模型一侧没有对应的预期结局；一旦出现就是 `Unexplained`，与复活集那个成员同处置。能不能出现：历史操作表「pub enum HistoryOperation {」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/src/history.rs:475`）里没有一步在已发布单元的某份副本上造坏字节；唯一让单元读回异常的是 `CrashRecoveryAbandoningTheNewestRoot`，它的文档说 C554 乙之后这一步「不再造出被抛弃的根」。所以随机历史走不到新拒因（推的，没跑随机历史，重型）。推翻它的现象：随机历史或故障注入档里任何一段把一份已释放单元的读回搞坏、再回退到引用它的那一版，那时 `Unexplained` 会把它记成分歧，而模型还预测回退做成。
- 对不上的地方：无。

### K4.2　新用例文件两条

- 隔离是不是条款说的那种：用例先把盘 1 上数据单元那一份的一个字节翻掉，再覆盖写换掉它，释放之前读盘核出盘 1 对不上、盘 0 对得上，两块盘的记录都留在已分配。前提断言钉了：隔离清单是 `[IntactButAnotherCopyFailed(盘 0), ChecksumMismatch(盘 1)]`，两块盘上那条记录都「!record.is_released && record.generation == CheckpointTxg(3)」，现行那一版不引用那个槽：「second.units.iter().all(|unit| unit.slot != data_slot),」（`/home/fy5090/.local/share/singlefs/work/code-backlog-b2-code-r1/repo/crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs:132`）。这正是 D19 说的「核出对不上时隔离那个槽，发布照成」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`）与「任一份核出对不上，这个单元在每块盘上的分配记录都留在」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）已分配。只覆盖了「核出对不上（校验和不符）」这一种，没有「读盘本身失败」那一种；两种在释放那一步走同一个函数，验读这一步也走同一个函数，所以不单列。
- 盘上不动怎么核：见 K3（录制流步数加系统配置槽加读得出的根；读不进步数，写都进）。
- 阳性对照（写回原样后回退做成）的分辨力：它对「去掉 C572 那一步」不红（`out/s5-c572-step-removed.txt` 里它 ok），所以它的作用只是证明拒绝用例拒的是坏份、不是隔离本身；对「共用单元也验」那种多拒它同样不红（K2.3）。

### K4.3　追加六行变异：改的是不是点名的那一处、换取样点还红不红

- 六行的原文在冻结副本 `mount.rs` 里各恰好命中一次，且与冻结副本 `crates/mutations.tsv` 里的行逐字相同（`tools/anchors2.py`：14 行输出都是「1 identical 1」，六行追加加八行替换）。
- 每行改的是点名的那一处行为（K1.1 的表：①②各只让对应用例红，③让三条红，④⑤各只让点名那条红；第六行去掉 C572 那一步，拒绝用例红、阳性对照不红）。
- 换取样点（量的）：取样点一，豁免与判重次序对调（K1.5）六条不红，探针 P1 红；取样点二，「共用单元也验」（K2.3）入库用例不红，我的探针红。这两个取样点是本报告新加，入库的变异表里没有对应行。

### K4.4　整行替换八行：锚点与是否仍红

锚点：八行在冻结副本 `mount.rs` 里各恰好命中一次（`tools/anchors2.py`，量的，上面那 14 行输出）。每行按它自己的参数跑、看点名的测试红不红（`out/s7-row0.txt` … `s7-row7.txt`）：

| 行 | 变异名 | 点名的测试 | 结果 |
|---|---|---|---|
| 0 | 步 4：影子账一个槽都不隔离 | `with_the_shadow_ledger_a_publish_after_a_recovery_keeps_off_the_abandoned_data_slot_and_the_abandoned_root_reads_the_third_content` | 红 |
| 1 | 步 4：影子账不豁免候选根引用的槽（保守读法） | `the_plain_remount_after_a_recovery_that_abandoned_roots_keeps_the_isolation` | **不红**（点名的测试跑了、过了） |
| 2 | 增补 2 收口第 27 行：只铺起点那一槽 | `an_abandoned_root_older_than_the_version_the_system_configuration_witnessed_…` | 红 |
| 3 | 第 35 批 K4-1 C 那一圈照旧豁免候选根引用的槽 | `a_power_loss_before_the_row_root_of_the_mount_still_reads_back_a_candidate_root_…` | 红 |
| 4 | 第 10z 批第九块 loop-span1 | `unmount_after_the_root_ring_slot_of_an_abandoned_root_turns_unreadable_…` | 红 |
| 5 | 第十四块 H2（1536 槽盘） | `wide_protection_after_releasing_a_large_batch_counts_no_slot_twice_…` | 红 |
| 6 | 第十四块 H2（384 槽盘双扣） | `wide_protection_at_the_mount_isolates_no_slot_still_counted_as_allocated_on_the_384_slot_devices` | 红 |
| 7 | 第十六块 抬 F 那一串双扣 | `defer_slots_an_abandoned_root_whose_ring_slot_reads_bad_references_are_not_counted_twice_by_a_floor_raise` | 红 |

第 1 行不红，量的两遍：冻结副本上（`out/s7-row1.txt`）；以及 git HEAD f111c3f3 的树（`git archive HEAD Cargo.toml Cargo.lock .cargo crates litmus` 解到草稿目录，把 HEAD 的 `crates/mutations.tsv` 里那一行「if referenced_by_candidates.contains(&key) || !isolated.insert(key) {」换成「if !isolated.insert(key) {」，跑同一条命令）：点名的测试同样「ok」。所以这一行改前改后都不红，**不是这次改动引入的**，但它是八行里被「整行替换」的一行，替换之后仍然没红。换取样点（量的）：同一个变异在交付的 core 用例上红两条（`a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement`、`a_current_account_record_exempts_every_slot_it_covers_from_the_placements_of_an_abandoned_root_read_from_its_account`，`out/s8-row1-at-the-core-unit-tests.txt`）。照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」：点名的那条 harness 慢档用例是取样点不敏感（要补敏感的取样点，不是留档），交付的 core 用例就是敏感的取样点；表里这一行的「必须红的测试名」该换成它们之一，否则门禁 checker-tier-crates-mutation-replay 整表复跑会报这一行没红（那一道是重型，我没跑，见末节）。推翻它的现象：门禁整表复跑里这一行是红的（那就是我这边的跑法与门禁不同）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 | 对得上 | B3 五处都按槽认豁免与判重，条款句 `23-journal…:390`；六条用例各钉一处，追加变异改回去各自红（量的）。另两条交主 agent：豁免先于判重的次序入库用例没钉（探针 P1，量的）；分配位置着的槽被「表里记着」与「读盘得来」两条被抛弃根交叠罩着时，改后比改前少隔离一槽（探针 P3，量的；能不能走到是推的，条款没写，判不了对不对）。 |
| K2 | 对得上；一半用例没钉；一处判不了 | 「现行单元里没有一个落在它的槽上」恰好认出被隔离的单元（四个角色造出来量过，漏拒与多拒两个方向都推过）；「现行那一版引用着就不验」这一道，去掉它入库用例全不红（量的，条款射程句 `23-journal…:388` 的后半句没被钉）；两条位置项不同槽没造、条款没写，判不了。 |
| K3 | 对得上 | 新一步排在复活集逐盘验之后、`allocator.clone()` 之前，入口到那里全是只读借用，拒绝用例断言盘与分配器不动（录制流步数不变，量的）；`first_copy_that_does_not_read_back` 抽出后复活集那一判次序、读法、拒因不变，25 条慢档含复活集拒绝用例全绿；两因同时成立报复活集那个（读码，没造）。 |
| K4 | 两臂与用例对得上；变异一行点名的测试不红 | 三处补法与同类成员对称，模型没有隔离概念，随机历史走不到新拒因（推的）；新用例造的是 D19 说的那种隔离；六行追加变异各红在点名处；八行替换锚点各命中一次，其中第 1 行（步 4 不豁免候选根）点名的 harness 慢档用例改前改后都不红（量的，HEAD 上同），core 用例上红。 |

## 要主 agent 跑的命令（我没跑，重型或超出本轮范围）

- 门禁整表变异复跑里 `步 4：影子账不豁免候选根引用的槽（保守读法）` 那一行（K4.4）：`SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 16G bash .claude/gate.d/checker-tier-crates-mutation-replay.sh` 在仓根跑，我没起任何门禁脚本。
- 全 harness 档（含 `random_histories.rs`）：K2.3 说「共用单元也验」入库用例不红，只覆盖了我跑的三个测试文件；别的文件（`unlink_*`、`presumed_clause_checks.rs`、`random_histories.rs` 等）我没跑，不排除其中有一条能钉住。
- 复核我的探针数：把 `research/prompts/code-backlog-b2-code-r1-sonnet-model/` 拷进新草稿目录，经 `scripts/hosts/run-with-memory-cap.sh 16G` 跑 `tools/rerun.sh`（命令在第 0 节），末行应是 `name=rerun_compare differing=0`。

## 没做什么

- 不判别的腿的格：没有替主 agent 采纳或出判决；「对得上 / 对不上 / 判不了」是这一条腿读出来的。
- 没起门禁阶段、没跑 checker 档、没跑全 harness 档、没起 GPU；没造 `deleted_inodes` 队列未走完的删除历史（K2 那一角色是读码）；没造两条位置项不同槽的映射条目；没造经公开入口走到 B3 混搭形（K1.6）的历史；没造两种拒因同时成立的历史（K3）。
- 没读被判文件在主工作区里的字面（冻结副本与主工作区 sha256 相同，但引用与行号都取自冻结副本）；没读禁读清单里的攻方报告与攻方模型。
- 校验步（第三步）不是这条腿的活：本报告的量的数各自写了命令与产物，但「换一条独立路子再做一遍」由主 agent 判决时现查。
  ✓ 核了 77 处引文、0 处裸行号，对上 77 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
