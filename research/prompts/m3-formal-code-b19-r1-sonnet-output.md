# 第 19 批代码轮第一轮：云端正推（Sonnet）报告 K1、K3

轮名 m3-formal-code-b19-r1。我读冻结副本 `/tmp/claude-1000/m3-formal-code-b19-r1/tree/`（下称「树」，引文里的 `crates/…` 路径都是树里的路径与行号），kb 引文写 kb 文件自己的行号。
主 agent 给的背景材料我只当导读，每一句都回源码、kb 现查；我没读别的腿的报告、模型目录与实现员报告。
模型与脚本在 `research/prompts/m3-formal-code-b19-r1-sonnet-model/`（说明见那里的 README.md），副本是 `/tmp/claude-1000/m3-formal-code-b19-r1-sonnet/ws`（冻结树的 crates 拷出，一字未改；只有副本里的 `walk.rs` 在变异期间被临时改、跑完还原，末尾 `git diff --no-index --quiet` 验过与冻结树逐字相同）。

## 0　先核的前提

- 冻结树与主仓此刻同一份：`walk.rs`、`crash_judge_checker_scan.wgsl`、`checker_known_bad_images.rs`、两个 E 装置 `cmp` 无差（命令：`cmp $T/crates/<f> crates/<f>`，五份都打印了 same）。
- kb 两份被引文件的 sha256 与快照清单一致：`invariants.md` 6ff077d4…、`03-空间分配.md` 2c23b3c2…（`sha256sum` 现跑，与 `kb-sha256.txt` 对上）。
- 19-1 的补丁在树里：`first_data_unit_slot_the_allocation_records_leave_on_a_device` 定义在「fn first_data_unit_slot_the_allocation_records_leave_on_a_device(」（`crates/singlefs-checker/src/walk.rs:4320`）。
- 变异表追加的 12 行在树里：`crates/mutations.tsv` 第 2292–2303 行，与改动全文末尾「变异追加」那 12 行同序同文。

## K1　f2 的现算对不对、会不会误红

### K1-1　f2 的函数逐句对条款

条款（I-5.5 一行）：「槽号等于这一版分配记录里 D3（空间分配） 已定项 8 第 1 条那条政策算出的那一对的起点槽号（起点最小、32768 对齐、两槽都空、不在开放段里）」（`.claude/kb/invariants.md:199`）。已定项 8 第 1 条：「起点槽号最小、起点 32768 对齐、跨度内两槽都空、不在任何开放的聚簇段里」（`.claude/kb/decisions/03-空间分配.md:163`）。

代码逐项：

| 条款的字 | CPU 代码 | 判 |
|---|---|---|
| 从 U 起 | 「let mut candidate = unit_area_start_slot;」（`crates/singlefs-checker/src/walk.rs:4335`）；U 是 64 的整数倍在读系统配置时判过：「if !unit_area_start_slot.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」（`crates/singlefs-checker/src/image.rs:376`） | 一致 |
| 32768 对齐 | 步长 `DATA_UNIT_SLOTS`＝32768 / 16384＝2（`SLOT_BYTES` 16384、`DATA_UNIT_BYTES` 32768、`CLUSTER_SEGMENT_SLOTS` 64 都在 `singlefs-format` 里现查过）；U 偶，候选恒偶 | 一致 |
| 两槽都空 | 「if (candidate..candidate + DATA_UNIT_SLOTS).all(|slot| !occupied_slots.contains(&slot)) {」（`crates/singlefs-checker/src/walk.rs:4342`）；被占集＝「&& (record.is_released || record.generation < version_checkpoint_txg)」（`crates/singlefs-checker/src/walk.rs:4331`） | 一致（「空」的口径见 K1-3） |
| 不在开放段 | 「candidate = segment.first_slot + CLUSTER_SEGMENT_SLOTS;」（`crates/singlefs-checker/src/walk.rs:4339`）；段取这一版树表单元在这块盘上那一份所在的段：「first_slot: location.slot - location.slot % CLUSTER_SEGMENT_SLOTS,」（`crates/singlefs-checker/src/walk.rs:4301`） | 一致（取法见 K1-3） |
| 每块盘上落点最小的那一个 | 「.and_modify(|smallest_slot| *smallest_slot = (*smallest_slot).min(location.slot))」（`crates/singlefs-checker/src/walk.rs:4220`） | 一致 |
| 这一版分配记录 | 种类 3 恰好一条：「let [allocation_entry] = allocation_entries.as_slice() else {」（`crates/singlefs-checker/src/walk.rs:4231`）；根指针取条目偏移 14（与 extent 条目同一处，同函数里 `[14..100]`） | 一致；「恰好一条」条款没写，是代码补的射程（K1-5） |
| 起点最小 | 候选只增不减、第一个合格就交回，终止性：每轮跳过的那一对里至少一个被占槽，或跳过开放段一次（代码注释的论证我重读了一遍，成立；槽号是 6 字节读出来的，`candidate + 2` 不会溢出） | 一致 |

对照真实分配器（`crates/singlefs-core/src/allocator.rs`）：「pub fn lowest_user_data_slot(」（`crates/singlefs-core/src/allocator.rs:737`）从 U 起按 2 步进，要求 `is_free(candidate)` 与 `is_free(candidate + 1)` 且不在 `cluster_segments` 里；`is_free` 另看隔离位与扣住位：「&& !self.isolated[self.index(slot)]」（`crates/singlefs-core/src/allocator.rs:523`）、「&& !self.held_until_floor_takes_effect[self.index(slot)]」（`crates/singlefs-core/src/allocator.rs:524`）。所以 f2 与实现差的恰是三样：回收门槛（已释放记录当占着）、隔离与扣住位、挂载内开过的全部段（f2 只排除树表单元那一段）。下面 K1-2、K1-3 逐样判。

### K1-2　合法历史上会不会误红（我跑了的部分）

做法：在副本里写探针 `b19_sonnet_probe.rs`（放进副本 `crates/singlefs-harness/tests/`）。历史由实现自己写：mkfs → 可写挂载 k 次 → 在最后一次挂载里推 z 次零单元发布 → 写第一个文件（默认 768 MiB 环、两块 4 GiB 内存稀疏盘）；每格打印数据单元相对 U 的偏移、I-5.5 判定、池级 checker 判红清单。cargo 经 `run-with-memory-cap.sh` 与 `capped.sh` 起，线程 4–8。

命令（工作目录副本根，`CARGO_TARGET_DIR` 指副本的 target）：`bash scripts/hosts/run-with-memory-cap.sh 4G bash scripts/hosts/capped.sh 4 cargo test -p singlefs-harness --test b19_sonnet_probe -- --nocapture`，`B19_KS`、`B19_ZS` 环境变量定格子。原始日志 `research/prompts/m3-formal-code-b19-r1-sonnet-model/out/scan1.log`（k=1、2 全 z=0..30，k=3 的 z=0..2，共 65 格；第一次起的串行扫，撞 240 秒前台上限被截断）与 `…/out/scan3.log`（k=3..14、z=0，12 格）。

汇总（命令数出来的）：

```
scan1: 65 of 65   （grep -c '^k=.*I-5.5=Holds reds=\[\]$' 对 grep -c '^k=')
scan3: 12 of 12
scan1 落点： 9 格 unit_offsets_from_U=[(0, 0), (1, 0)]，56 格 [(0, 4), (1, 4)]
scan3 落点： 5 格 [(0, 0), (1, 0)]（k=10..14），7 格 [(0, 4), (1, 4)]（k=3..9）
```

77 格里 I-5.5 全是 Holds（没有一格 NotApplicable、没有一格 Violated），池级 checker 判红清单全空。翻转点与条款、harness 用例注释的数对得上：k 个只暖机挂载的第一个文件，k ≤ 9 落 U + 4、k ≥ 10 落 U；k=2 时 z ≤ 21 落 U + 4、z ≥ 22 落 U（`scan1.log` 里 k=2 z=22 起偏移变 0）。即 f2 在「回收前、回收后、回收翻转点两侧」都与实现一致，第一次回收发生那一格（k=10、k=2 z=22）也不误红。

### K1-3　三样近似与开放段、「这一版自己分配的当空」逐样判（读代码；能跑的跑了）

代码注释自己列了四样复算不出的近似（`crates/singlefs-checker/src/walk.rs:4312` 起）。我按「它让 f2 比实现算得更靠前还是更靠后」判误红方向：f2 算得比实现靠后只会漏判，算得比实现靠前才会误红。

| 近似 | 对实现的方向 | 依据 | 判 |
|---|---|---|---|
| 已释放的记录一律当占着（回收门槛复算不出） | 只漏判，不误红 | 实现把回收过的槽当空、挑到它就改写那条记录：「existing.is_released = false;」（`crates/singlefs-core/src/allocator.rs:910`），落点不会跳过一个没被扣住、没被隔离、对子也空的已回收槽；所以这一版的记录里，实现落过的槽已不是已释放。77 格里 14 格落在 U（回收之后）全 Holds，就是它 | 一致；漏判 H1 已记 C689 |
| 抬 F 扣住位 | 不误红 | 扣住只加在刚回收的记录上：「.hold_until_floor_takes_effect(record.slot, u64::from(record.span_slots));」（`crates/singlefs-core/src/allocator.rs:1748`）；那条记录仍是已释放，f2 也当占着，两边同样跳过；扣住放开之后实现挑它、改写记录，f2 看到的是改写后的 | 一致（读代码推的，没造抬 F 的历史跑） |
| 保护期 R | 不误红 | 第一版 R 为空：「第一版 `R` 为空。」（`crates/singlefs-core/src/allocator.rs:780`） | 一致 |
| 隔离位（影子账） | 能误红，条件窄 | 隔离有两路。① 已回收又被被抛弃根引用的槽补隔离（`isolate_reclaimed_slots_an_abandoned_root_still_references`，`crates/singlefs-core/src/allocator.rs:1960`）：记录仍是已释放，f2 也当占着，同跳。② 只被被抛弃根引用的槽（「allocator.isolate_abandoned(device, slot, span_slots);」，`crates/singlefs-core/src/mount.rs:1492`）：这一版的分配记录里没有它们，f2 当空，实现跳过——只有这些槽低于实现真落点才误红。被抛弃根自己分配的块两类：数据单元（被抛弃的是带第一个文件的根时，下一个第一个文件的树 ID 不从 65536 起、I-5.5 一版不判：「树 ID 不从 65536 起，I-5.5 一版都不判」，`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:164`）；提交内生块（住开放段，开段取最低的全空段，「self.used_per_segment[*segment] == 0」，`crates/singlefs-core/src/allocator.rs:766`；而 mkfs 的树表要等第一个文件才释放，U 那一段在第一个文件之前一直非空，开放段不会是 U 那一段，被抛弃根的提交内生块都在 U + 64 以上） | 规则没说（kb 没把它列成近似）；young 池上推不出误红，没造被抛弃根的历史跑 |
| 开放段取这一版树表单元所在的段 | 挂载内开了不止一段时能误红 | 实现排除这次挂载开过的全部段：「let cluster_segments = &self.cluster_segments;」（`crates/singlefs-core/src/allocator.rs:1544`），f2 只排除树表单元那一段。从 mkfs 水位建树的那一版，同一次挂载里只有写行、暖机与第一个文件三次发布（零单元发布不 bump），写不满一段 | 一致（推的；77 格里只有一段开放） |
| 这一版自己分配的记录当空 | 必须这么读 | 条款「两槽都空」没交代这一版自己那条记录。实现落完记录写的是未释放、分配代＝这一版的 txg；若当占着，落点永远与实现错开。变异 2294（把 `<` 改 `<=`）被 `a_new_pool_file_on_the_default_journal_ring_holds_the_first_data_unit_placement` 杀（副本上跑，见 K3-5 的表） | 规则没说（kb 里一个字没有，`grep -nF 自己分配` 在两份 kb 里零命中）；代码取的是唯一自洽读法，一致 |

### K1-4　非默认环、多盘、第一个文件多单元

- 非默认环：U 来自所选系统配置偏移 417，读配置时判过是 64 的倍数（`crates/singlefs-checker/src/image.rs:376`），f2 与环长无关。128 MiB 环的用例在副本上跑绿：`research/prompts/m3-formal-code-b19-r1-sonnet-model/out/pool-checker-tests.log`，整份 8 个 `test … ok`，`test result: ok. 8 passed; 0 failed`。用例：「fn a_new_pool_file_on_a_128_mebibyte_journal_ring_holds_the_first_data_unit_placement() {」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:326`）。判一致。
- 多单元：「fn a_new_pool_file_of_two_data_units_holds_the_first_data_unit_placement_on_the_smallest_slot_of_each_device(」（`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs:595`）落 U + 4 与 U + 6，取每盘最小的 U + 4，成立；变异 2295（取最大）把它杀红。判一致。
- 多盘：f2 对每块盘各算一个落点；实现要各盘一致才分配（`agreement_across_devices`，`crates/singlefs-core/src/allocator.rs:164`），合法镜像上各盘落点同值，f2 的逐盘判与它同口径。harness 夹具只有两块盘（`region_devices` 三区落在两盘）；三块盘以上、盘不等大的合法镜像我没有夹具造，判不了：规则没说，没验。

### K1-5　判不了的几格是不是过宽

f2 新增的两格「种类 3 不是恰好一条」「分配记录树读不全」，连同原有的五格，在 CPU 里都记成射程判不了，不出红（`crates/singlefs-checker/src/walk.rs:4231` 起）。从 mkfs 水位建树的合法版本，树表里种类 3 必有且仅有一条（77 格 I-5.5 全是 Holds 而不是 NotApplicable，说明这七格一格都没触发）。结论：在 young 池、两块盘的合法历史上不过宽；这几格只在坏镜像或盘缺失时触发；坏实现若借它躲掉 I-5.5，要靠别的不变量兜，那一层我没量。判一致；推翻它：一份合法历史上 I-5.5 报 NotApplicable（上面七格的原因之一）。

### K1-6　kb 与代码对不上的两处（冲突，整行并排）

① 代码注释说四样近似都写进了 I-5.5 那一行，kb 那一行只写了一样。

- 代码：「都照实写进 `.claude/kb/invariants.md` I-5.5 那一行：」（`crates/singlefs-checker/src/walk.rs:4312`）
- kb：「checker 复算回收门槛是近似的」（`.claude/kb/invariants.md:199`）；同句在已定项 8 射程里也只有这一样：「checker 复算回收门槛是近似的」（`.claude/kb/decisions/03-空间分配.md:171`）

隔离位 / 扣住位 / R、开放段取树表单元所在段、这一版自己分配的记录当空，这三样在两份 kb 里一个字没有。

② 状态格还是改前的话。

- kb：「判据待第 19 批 19-1 改成 f2（按分配记录现算）；今天的实现仍写死 U + 4，回收之后会误红」（`.claude/kb/invariants.md:199`）
- 代码：「let mut candidate = unit_area_start_slot;」（`crates/singlefs-checker/src/walk.rs:4335`）——实现已是 f2

kb 文件与快照清单 sha256 一致（上面第 0 节），所以这不是我读晚了版本。背景材料说 19-3 已写回，写回的是条款句与已定项 8，状态格没动。

推翻 ①：在 kb 里找到那三样近似的字（`grep -nF` 命中），或快照之后 19-3 另有一处写了它们。推翻 ②：状态格里「今天的实现仍写死 U + 4」那一半在快照之后被改掉。

## K3　I-9.27 证据补丁与 19-2

### K3-1　I-9.27 条款与代码逐句（一致）

| 条款（`.claude/kb/invariants.md:314`） | 代码 | 判 |
|---|---|---|
| 种类 2 等于它自己的树 ID | `HeadIdentifierOfATreeTableEntry::ItsOwnTreeIdentifier => head_identifier == tree`（`crates/singlefs-checker/src/walk.rs` 的 `holds` 那个 match） | 一致 |
| 种类 1、9、10 等于这张表里种类 2 条目的树 ID，多于一条放宽成「等于其中某一条」，一条都没有判红 | 「inode_tree_identifiers.contains(&head_identifier)」（`crates/singlefs-checker/src/walk.rs:3519`）；空集 `contains` 为假，判红 | 一致 |
| 种类 3、4、6、7、8、11、12 等于 0；别的种类码不判 | `Zero` 臂列的是 `TREE_KIND_ALLOCATION | ACCOUNTING | LIVELIST | SPARSE_SIDE_TABLE | DEADLIST | LINEAGE | LOGGED_OPERATIONS`（`crates/singlefs-checker/src/walk.rs:3398` 起），常量值 3、4、6、7、8、11、12 在 `crates/singlefs-core/src/records.rs:375-385` 与 walk.rs 自己的常量里对过；`_ => NotJudged` | 一致 |
| 条目宽不到 124 字节整张不判 | 「.all(|entry| entry.len() >= TREE_TABLE_ENTRY_HEAD_IDENTIFIER_OFFSET + 8);」（`crates/singlefs-checker/src/walk.rs:3496`），偏移 `8 + 2 + 2 + 2 + 86 + 8 + 8`＝116（`crates/singlefs-checker/src/walk.rs:3372`），116 + 8＝124 | 一致 |
| 不拿字面 65537 比 | 全程比的是关系，没有字面 65537 | 一致 |

### K3-2　四张坏镜像各自证不证得到它点名的那一格

四张坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：种类 2「I-9.27 种类 2 那一支（判决」（第 6376 行注释）、种类 3「I-9.27 写 0 那一支（同一节证据缺口」（第 6390 行）、种类 9「I-9.27 种类 9 那一格（同一节证据缺口」（第 6402 行）、种类 10「I-9.27 种类 10 那一格」（第 6414 行），都经「fn rewrite_the_head_identifier_of_the_tree_table_entry_of_kind(」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6431`）：改之前先 `assert_eq` 头 ID 本来的取值（「种类 {kind} 那条的头 ID 本来的取值」，`crates/singlefs-harness/tests/checker_known_bad_images.rs:6462`），所以造的不是凭空镜像；该种类在 A 的树表里必须恰好一条，否则 panic。判据是「判红的该只有 {invariant}：{verdicts:?}」（`crates/singlefs-harness/tests/checker_known_bad_images.rs:6547`；即只红该镜像点名的那一条，这里是 I-9.27），干净镜像上它要真 Holds（「干净镜像上 {invariant} 要真被评估过且成立：{verdicts_on_the_clean_image:?}」，`crates/singlefs-harness/tests/checker_known_bad_images.rs:6537`）。

我在副本上做了两件事：

1. 基线：`each_pointer_journal_and_generation_field_bad_image_reddens_only_its_own_invariant` 跑绿（`research/prompts/m3-formal-code-b19-r1-sonnet-model/out/kbi-base.log`：`test result: ok. 1 passed; 0 failed`）。
2. 逐镜像归因：把那条断言的循环改成逐镜像打印（`kbi_print.rs`，差异在模型目录 `kbi_print.patch`），基线 22 张里第 17–21 号是 I-9.27，每张都是 `["I-9.27"]`。再把 `crates/mutations.tsv` 第 2298–2301 行（追加的四条 ⑤）逐条打在副本 `walk.rs` 上，用官方命令与打印版各跑一遍：

```
row 2298: exit=101 print_exit=0   IMG 18/22 I-9.27 -> []   （其余 17、19、20、21 仍 ["I-9.27"]）
row 2299: exit=101 print_exit=0   IMG 19/22 I-9.27 -> []   （其余仍红）
row 2300: exit=101 print_exit=0   IMG 20/22 I-9.27 -> []   （其余仍红）
row 2301: exit=101 print_exit=0   IMG 21/22 I-9.27 -> []   （其余仍红）
```

（`research/prompts/m3-formal-code-b19-r1-sonnet-model/out/mut/row2298-print.log` 等；基线是 `out/mut/base-print.log`。第 17 号是旧的 extent 树写 0，四条变异都不动它。）

行对应：2298「种类 2 的头 ID 不比自己的树 ID」→ 18 号（种类 2）；2299「种类 3、4、6、7、8、11、12 的头 ID 不比 0」→ 19 号（种类 3）；2300「deleted_inodes（种类 9）条目不判头 ID」→ 20 号；2301「墓碑树（种类 10）条目不判头 ID」→ 21 号。每条变异恰好被它对应的那一张杀、别的不受影响，官方测试 exit=101（失败）。判：四张各自证得到它点名的那一格。

### K3-3　「写 0 的七种」只证到一种（我自加的变异量出来的）

判决里这一缺口的原话是「各补一张坏镜像或一条用例」（`research/prompts/m3-formal-code-b17-r1-main-verification.md:21`），补一张是合判决字面的。但 19 号只改种类 3 的头 ID，条款里写 0 的有七种。我在副本 `walk.rs` 的 `of_tree_kind` 那个 match 里（`TREE_KIND_INODE` 那一臂之后）依次加一条「种类 X 一律不判」的变异（`kinds.tsv`，每行在 `crates/singlefs-core/src/records.rs` 的七种与 1、9、10 里取一种），跑官方测试：

```
种类 ALLOCATION(3) exit=101（杀）   ACCOUNTING(4) exit=0（活）   LIVELIST(6) exit=0（活）
SPARSE_SIDE_TABLE(7) exit=0（活）   DEADLIST(8) exit=0（活）     LINEAGE(11) exit=0（活）
LOGGED_OPERATIONS(12) exit=0（活）  EXTENT(1) exit=101（杀）     DELETED_INODES(9) exit=101（杀）
TOMBSTONE(10) exit=101（杀）
```

（`out/mutk/row*.log`；种类 2 那一行已由 2298 覆盖，没重复。）A 的树表里这七种都存在，我现列了：

```
KINDS (tree id, kind, head id): [(65536, 1, 65537), (65537, 2, 65537), (65538, 3, 0), (65539, 4, 0), (65541, 6, 0), (65542, 7, 0), (65543, 8, 0), (65544, 9, 65537), (65545, 10, 65537), (65546, 11, 0), (65547, 12, 0)]
```

所以每一种都造得出坏镜像（把头 ID 改成非 0），现在只有种类 3 有。判：「写 0 那一支」证到种类 3，另六种（4、6、7、8、11、12）从这批证据里证不到；是否要六张再补，条款与判决没写，归主 agent 定。推翻它：给那六种之一补上坏镜像后，对应「种类 X 一律不判」变异由活变杀。

### K3-4　两条库单测

两条在 `crates/singlefs-checker/src/walk.rs`：「fn an_extent_entry_naming_the_second_of_two_inode_tree_entries_holds_the_head_identifier_relation(」（`crates/singlefs-checker/src/walk.rs:9104`）与「fn an_extent_entry_naming_neither_of_two_inode_tree_entries_violates_the_head_identifier_relation(」（`crates/singlefs-checker/src/walk.rs:9120`）。基线在副本上 `cargo test -p singlefs-checker --lib` 跑那几条，4 个 `ok`（含 I-5.5 的两条库单测）。变异 2302（多条种类 2 时只认第一条）与 2303（`contains` 改成 `true`）在副本上各自让对应那条库单测 `FAILED`，退出码 101（`out/mut/row2302.log`、`row2303.log`）：2302 杀第一条、2303 杀第二条。判：两条各自证得到它点名的那一格。它们只管「多条种类 2 放宽」与「两条都对不上判红」，不管哪一条头 ID 对得上谁（比如认最后一条的读法两条都放过），那一点条款也没写。

### K3-5　I-5.5 的六条变异（K1 的证据，顺带跑了）

`crates/mutations.tsv` 第 2292–2297 行，官方命令各跑一遍，全部 exit=101，点名的测试 `FAILED`：2292（从 U + 4 起扫，k=10）、2293（z=22）、2294（自己分配的当占着）、2295（取最大）、2296（不排除开放段，库单测 `the_first_data_unit_slot_skips_the_open_cluster_segment_of_the_version`）、2297（已释放不当占着，库单测 `a_released_allocation_record_keeps_its_slots_out_of_the_first_data_unit_slot`）。日志 `out/mut/row2292.log` 至 `row2297.log`。

### K3-6　19-2：改名有没有动行为（一致）

补丁在改动全文第 1022–1300 行。我把它反向打回冻结副本的两份装置得到改前文件，再用 `rename_equivalence.py`（模型目录）比：去注释后切记号，两边记号数必须相等；记号不同的位置必须都是标识符对标识符；再按「最近的前一个 let 绑定」解析每个标识符，要求改前改后每个使用解析到同一个绑定位置（防止改名把一处使用挪到另一个仍在作用域里的同名绑定上）。原样输出：

```
== e165_replay_bounded_enumeration
记号数 改前 92339 改后 92339
不同的记号 27 个，其中不是标识符对标识符的 0 个
  content -> ['first_write_content', 'second_write_content', 'write_content_beside_the_unreadable_one', 'write_content_when_no_slot_reads']
  detail -> ['gate_refusal_proof_detail', 'second_read_proof_detail']
  passed -> ['gate_refusal_proof_passed', 'second_read_proof_passed']
  slot -> ['first_write_slot', 'second_write_slot', 'write_slot_beside_the_unreadable_one']
解析到的绑定位置前后不同的使用 0 处
== e171_three_oracle_collusion
记号数 改前 35654 改后 35654
不同的记号 27 个，其中不是标识符对标识符的 0 个
  checks -> ['allowed_refusal_checks', 'error_after_the_stop_checks', 'no_stop_checks', 'recorded_red_checks', 'silent_acquire_checks', 'unmount_stop_checks']
  failure -> ['acquire_failure', 'warm_up_failure']
  refused -> ['refused_acquire', 'refused_warm_up']
  started -> ['crash_judge_started', 'pool_checker_started', 'recovery_started']
解析到的绑定位置前后不同的使用 0 处
```

字符串字面量记号在两边逐个相等（它们是一个记号），所以输出格式与比较的字面量都没动。E165 输出行 `name=self_proof item={item} passed={passed} detail={detail}`（`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` 的 `for (item, passed, detail) in &proofs` 循环）取的是循环自己的绑定，不是被改名的那几个；改名后 `proofs` 里推进去的值仍是原来的布尔与 `detail.join(",")`。E171 一句注释：「模型这一步不交答：取号、暖机（两次都返回之后一起推一步）与卸载（`crates/singlefs-harness/src/model.rs` 里已有 `answer_unmount`」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:237`）；`model.rs` 里确有它：「pub fn answer_unmount(&self) -> Result<ModelAnswer, ModelDisagreement> {」（`crates/singlefs-harness/src/model.rs:2505`），装置里 `grep -n answer_unmount` 只命中注释那一行 237，没改调，与注释「本装置还没改调它」一致。「改调归 E171 第 1 次重跑」是写给后人的话，我没核（转述）。装置输出不变要等提交时复跑行核（checker 档，不归我）。判：一致，纯改名，没动比较或输出。推翻它：复跑 E165、E171 的输出与改前同一份产物逐行不同。

## 复核不了的事

- GPU 内核（`crash_judge_checker_scan.wgsl`）与 CPU 在同一镜像上是否同判：属 checker 档、要 GPU，没跑。我只读了代码，推的：落点候选集「U、开放段末、每条被占记录的末端上取到 2 的整数倍」包含最小合格起点，论证是若最小合格起点 p 大于 U，则 p − 2 那一对不合格，要么 p − 1 或 p − 2 被某条记录罩着、那条记录的末端必为 p − 1 或 p（上取后得 p），要么 p − 2 在开放段里、p 恰是段末——所以 `c_first_data_unit_slot_the_allocation_records_leave_on_a_device`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1812`）与 CPU 同值。记录字长 8 个字、第 6 个字是已释放标志，占用判「if (sw(a + 6u) == 0u && !lt64(generation, txg)) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1787`）与 CPU 的 `is_released || generation < txg` 同；种类 3 不是恰好一条就整版不置位：「if (!has_any_data_unit || allocation_entry_count != 1u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1901`）。GPU 记录数上限 `C_MAX_ALLOC_RECORDS` 2048（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:373`），超限记「判不了」，我没查那条路径上 I-5.5 的处置。这些是推的，K2 才是它的腿。
- 抬 F 扣住与隔离的合法历史：只读代码推，没造历史跑（K1-3）。
- 清零第一个文件根槽、让它只由 journal 记录施加出来的变体：我试了（`b19_sonnet_probe.rs` 的 `scan_record_only`，k=1、2、9、10、11 五格），I-5.5 都报「NotApplicable（这个镜像上没有它判的对象）」、判红清单空——这个造法没让 checker 走到「由记录施加出来的那一版」那一路，所以这一路上 f2 的开放段取法（树表指针取记录新根段里的）我没验到。
- E165、E171 装置输出不变：checker 档，归提交时复跑行。

## 判定一览

| 格 | 判定 | 一句话 | 推翻它的现象 |
|---|---|---|---|
| K1-1 f2 逐句对条款 | 一致 | 从 U 起、32768 对齐、两槽空、避开放段、每盘取最小，七项都对上 `walk.rs:4320-4347` 与 `allocator.rs:737` | 条款或实现的「起点最小」改了口径 |
| K1-2 合法历史误红（回收前后） | 一致（量过） | 77 格（k 次暖机挂载 × z 次零单元发布）I-5.5 全 Holds、判红清单全空，14 格落 U、63 格落 U + 4 | 一格 I-5.5 Violated 或 NotApplicable |
| K1-3 回收门槛近似 | 一致，只漏不误红 | 已回收的槽被实现挑中就改写记录；漏判 H1 记在 C689 | 一段合法历史上实现落在比 f2 更靠后的槽，且 U 到那槽之间有已回收、没被扣住没被隔离的对子 |
| K1-3 抬 F 扣住、保护期 R | 一致（推的） | 扣住只加在刚回收的已释放记录上，f2 同样当占着；第一版 R 为空 | 造一段抬 F 的合法历史 I-5.5 红 |
| K1-3 隔离位 | 规则没说；条件窄（推的） | 只被被抛弃根引用的槽 f2 当空、实现跳过；这些槽在 U + 64 以上或属带第一个文件的根（不判） | 一段合法历史里被抛弃根的块低于第一个文件的真落点 |
| K1-3 开放段取树表单元所在段 | 一致（推的），规则没说 | 实现排除本挂载开过的全部段，f2 只排一段；从 mkfs 水位建树的版本只有一段 | 同一次挂载里 bump 越过一段之后写第一个文件的合法历史上 I-5.5 红 |
| K1-3 这一版自己分配的当空 | 规则没说，唯一自洽读法 | 变异 2294 被杀 | — |
| K1-4 非默认环 / 多单元 | 一致（量过） | 128 MiB 环与两个数据单元的用例副本上过；变异 2295 被杀 | 用例红 |
| K1-4 多盘（>2 盘、不等大） | 规则没说，没验 | 夹具只有两块盘 | — |
| K1-5 判不了的几格过宽否 | 一致（young 池、两盘） | 77 格没有触发任何一格 | 合法历史上 I-5.5 报 NotApplicable |
| K1-6 ① 代码注释说四样近似写进了 kb | 冲突 | kb 只有「checker 复算回收门槛是近似的」，另三样一个字没有 | kb 里找到那三样的字 |
| K1-6 ② kb 状态格 | 冲突 | 状态格还写「今天的实现仍写死 U + 4」，实现已是 f2 | 状态格已被改 |
| K3-1 I-9.27 条款对代码 | 一致 | 五句逐句对上 | — |
| K3-2 四张坏镜像 | 证得到各自那一格 | 四条 ⑤ 变异各被恰好一张杀，别的镜像不受影响 | 打变异后对应镜像仍红 |
| K3-3 写 0 的七种 | 只证到种类 3 | 种类 4、6、7、8、11、12 一律不判的变异都活着，但 A 的树表里这六种都存在 | 给六种之一补镜像后变异转杀 |
| K3-4 两条库单测 | 证得到各自那一格 | 2302、2303 各杀一条 | — |
| K3-6 19-2 改名 | 一致，纯改名 | 两份装置记号流等长、27 处差全是标识符对标识符、使用解析的绑定位置 0 处变 | E165、E171 复跑输出与改前不同 |

## 没做什么

- 不判别 K2（落点挪到别的合格空槽、只在一块盘上挪、开放段算错、GPU 与 CPU 分歧）；K2 的猜想我只在 K1-3 里顺带写了方向，没当 K2 判。
- 没跑 checker 档（`singlefs-checker-tier` 的测试、GPU 对拍两条坏流、E165、E171 复跑）、没跑门禁、没跑 gate；没有要主 agent 跑的长命令。
- 没造抬 F、被抛弃根（隔离）、盘不等大、三块盘以上的合法历史。
- 没核「改调归 E171 第 1 次重跑」这句。
- 不替主 agent采纳或出判决；K1-6 两处冲突怎么改、K3-3 要不要补六张镜像，归主 agent。
- 清理：下面「草稿目录与删除」那一节写了我建的编译目录与副本的去处。

## 草稿目录与删除

- `/tmp/claude-1000/m3-formal-code-b19-r1-sonnet/target`（副本的 cargo 编译目录）与 `/tmp/claude-1000/m3-formal-code-b19-r1-sonnet/ws`（冻结树 crates 拷出的工作副本）：交回前我删，因为它们是可重建的编译产物与副本，要重跑从模型目录的 README.md 重建；`pre`、`post` 是 19-2 补丁反向打回的两份装置文件与改后文件，同样删。保留的是模型目录里的脚本与 `out/` 日志。
