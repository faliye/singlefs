# W3 正推腿报告（m3-w3-r1，Sonnet 正推）

<!-- doc-lint:not-numbers K1 K2 K3 K4 W1 W2 W3 V1 V2 P1 C1 C2 C3 C4 -->

读法：引 kb 写 kb 文件自己的行号（`.claude/kb/...`，行号都是我在这一次 `grep -nF` 命中的）；引代码写冻结树里的绝对路径:行号（树根 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/`，交回闸拿活仓当根判引文，写相对路径会对不上活仓的行号）；引产物写 `research/results/e175-product-path-inode-watermark-regression-2026-10-08/` 里的文件:行号。「推的」是没量过的推论；「观测」是我这一次跑出来的。
候选记号：C1 实例表行预留字节，C2 记账里留历代水位行，C3 另立只追加的发号记录，C4 我提的别的形态（下文 K1 定义，C4 有 C4a、C4b 两支）。

## 〇、取证底座

### 0.1 条款（只引与这一轮有关的句子）

- D8 已定项 6 射程把崩溃恢复抛弃的那一段排除在时间线之外：「不含崩溃恢复抛弃的那一段（崩溃恢复落到较旧的根之后，被抛弃那一段里发过的号可以重发）」（`.claude/kb/decisions/08-核心索引结构.md:207`）。
- 同一行也把管理员回退抛弃的那一段算进时间线：「含管理员回退抛弃的那一段（回退那次发布的水位取 max，那一段里发过的号不重发）」（`.claude/kb/decisions/08-核心索引结构.md:207`）。
- D23 已定项 14 的管理员回退「水位」一格：「| 水位 | inode 号水位与树 ID 水位 = max(cur 那一版内存里的水位, 环里读得出的根的水位)，不跟着 R_old 退回去 |」（`.claude/kb/decisions/23-journal的角色与格式.md:385`）。
- 实例表行字段表：「`kind` = 0 行记录：`kind 1 | 实例代号 4 | 所选根的 checkpoint_txg 8 | 属于该实例的最大已施加事务号 W 8 | flags 1」（`.claude/kb/decisions/18-块里携带什么信息.md:303`），后接「预留 66」同一行。
- 行只在恢复与实例切换时写：「按实例代号唯一、后写覆盖，只在恢复与实例切换时写」（`.claude/kb/decisions/18-块里携带什么信息.md:310`）。
- 行的回收条件：「行可删当且仅当整轮清扫对池中每一个落点都得出了判定」（`.claude/kb/decisions/18-块里携带什么信息.md:315`，引句是该行前半句）。
- 记账每条根只带一代：「每条根的记账树只装这条根那一代的行，不留多代」（`.claude/kb/decisions/05-快照-空间记账机制.md:47`）。
- 水位是第 12 项统计量，带树维：「| 12 | inode 号水位（下一个可用 inode 号） | inode 号分配（D8（核心索引结构） 已定项 6：inode 号单调不复用，下一个号在被问到那一刻要有答案） | 不带 | **带**（只为可写头维护） |」（`.claude/kb/decisions/05-快照-空间记账机制.md:127`）。
- D15 判定表第一行：「| 新增字段在预留区 / TLV 尾部 | 忽略，继续正常工作 | compat | 否 | 没有语义依赖的纯加法，「忽略陌生字段」这条安全网够用 |」（`.claude/kb/decisions/15-格式冻结政策.md:185`）。
- D15 已定项 7 射程：「树表单元与实例表单元落哪一层还没有条款」（`.claude/kb/decisions/15-格式冻结政策.md:155`，引句是该行一段内的一句）。
- 冻结层登记：「| 打包记录类型 4（实例表记录，一片 370 条 88 字节记录、含链指针记录） | 组件 ① | 权威态 | 不退出 | D21（权威态与派生态的分界） 已定项 9 |」（`.claude/kb/freeze-layer-membership.md:50`）与「| 实例表单元（码 3 打包记录类型 4 的容器，物理指针由根记录直接持有） | 没有条款 | 权威态 | 不退出 | D21（权威态与派生态的分界） 已定项 9 |」（`.claude/kb/freeze-layer-membership.md:57`）。注：`freeze-layer-membership.md` 不在 `m3-w3-r1-snapshot/kb-sha256.txt` 的九份里，这两行是我读的今天的文件。
- I-9.6 的判据：「> 该树 inode 树内最大 key（回退候选集里任一版的 inode 树都算）」（`.claude/kb/invariants.md:294`，引句是该行一段内的一句）。
- I-9.29 的判据：「回退候选集里各版合起来（含由记录施加出来的那几版）同一个 inode 号只许对应一个对象出生代」（`.claude/kb/invariants.md:317`，引句是该行一段内的一句）。

### 0.2 代码（冻结树）

- 行结构只有三个字段：「pub selected_root_txg: CheckpointTxg,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:23`）与「pub applied_transaction_high_water: u64,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:24`）；写者把预留 66 字节写 0：「writer.skip(66);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:41`）；读者读完 flags 就收手，预留不读：「let flags = reader.get_u8();」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:58`）。
- 池级 checker 解一行只取前 21 字节：「fn parse_instance_table_row(row: &[u8]) -> InstanceTableRow {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:6336`）；I-3.14 只判 kind、链指针位置、flags：「fn judge_instance_table_page_record_forms(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:841`）。所以今天预留 66 字节里写什么，core 与 checker 都不拒（观测，读代码）。
- 写行：恢复给 [max(所选根的实例,1), 新实例) 写行，所选根那个实例写 (T, W)，中间实例写 (0,0)：「fn instance_rows_to_write(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:4861`）；W 取日志重放施加到的最大事务号：「applied_transaction_high_water: journal.maximum_applied_transaction,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7011`）；新行接在所选根那张表后面，不接别的表：「rows.extend_from_slice(rows_written);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1217`）。
- 判抛弃：「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1561`）。
- 实例切换没有写行路径：在 `mount.rs`、`transaction.rs`、`allocator.rs`、`instance_table.rs` 里搜「实例切换」的命中只有预留量与注释（观测：`grep -n '实例切换' mount.rs transaction.rs allocator.rs instance_table.rs`，命中 9 行，没有一行是写行函数；`grep -n 'fn .*switch'` 在这几份和 `mounted_session.rs`、`recovery.rs` 里除只供测试的开关外也没有切换函数）。写行只有可写挂载的 `establish_instance` 一处，删行只有 `instance_table_sweep.rs`。
- 发号历史在 `crates/` 里没有：冻结树根上 `grep -rniE 'inode_number_history|number_history|issuance_log|issued_range|highest_issued|issued_watermark|inode_high_water' crates --include=*.rs | wc -l` 输出 0；`grep -rnE 'inode_number_watermark_at|history_row|watermark_history' crates --include=*.rs` 只命中一个不相干的测试名「a_publish_without_a_previous_version_starts_the_inode_number_watermark_at_two_on_the_product_path」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:9458`）。所以下面四个候选的写者与读者都是「以后加在哪一处」，不是改现有的。
- 水位怎么算：发布路径起号与记账都读上一版记账里的水位：「let inode_number_watermark = match previous {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:8026`）；管理员回退的水位取环里读得出的根的最大值与现行内存值的 max：「ring.max(current.inode_number_watermark())」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7755`），调用点：「version_to_publish_the_rollback_after(current, &target_version, inode_number_watermark);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7758`）。V1 的补丁只改这一行的第三个参数。
- 记账 15 行一代：「pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:181`）；条目 34 字节：「pub const ACCOUNTING_ENTRY_BYTES: u64 = 34;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:145`）；叶头 159：「pub const ACCOUNTING_TREE_INDEX_NODE_HEADER_BYTES: u64 = 159;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:71`）。一个 16 KiB 叶装 ⌊(16384 − 159) / 34⌋ = 477 条（算术）。
- 根记录 457 字节、没有预留：「pub const ROOT_RECORD_BYTES: u64 = 457;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:210`）；它已经带一个树 ID 水位：「pub tree_identifier_watermark: u64,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/root_record.rs:72`）。
- checker 今天怎么判 I-9.6、I-9.29：I-9.6 只比最新根记账里的水位与最新根 inode 树内最大 key：「judgements.judge("I-9.6", watermark > largest_inode, || {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:11676`）；I-9.29 只比候选集里各版：「fn judge_one_object_birth_per_inode_number(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:10926`）。
- 崩溃状态数闭式：「/// E77（发布的持久顺序） 的闭式：崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/src/segments.rs:201`）。

### 0.3 E175 V1 那两次（观测）

- 第 389 行：「E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op92 inode=7 new_birth=77 older_births=[51] q9=Gone i9_29_at_step=holds」（`research/results/e175-product-path-inode-watermark-regression-2026-10-08/g-long-V1.out:389`）。
- 第 390 行：「E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op95 inode=8 new_birth=78 older_births=[52] q9=Gone i9_29_at_step=holds」（`research/results/e175-product-path-inode-watermark-regression-2026-10-08/g-long-V1.out:390`）。
- 我把冻结树拷到 `~/.local/share/singlefs/work/m3-w3-r1-sonnet/repo/`，施加 `patch-V1.diff`，放进 `device-source.rs`，自己编、只跑 `E175_INDICES=24` 的 G-long V1，E7RESULT 行（去掉 `elapsed_ns`、排序）与已发布的 `index=24` 十四行逐行相同（观测：`diff` 输出「SAME-as-published-index-24」）。下文的模型都在这份副本上长出来。

## 一、K1　留什么、留在哪

### 1.1 先把「判红」定成两条判法（四个候选都用它们，只换历史行的来源）

历史行 (g, H) 的意思：到 txg g 为止，这条时间线上小于 H 的号都发过了。D8 已定项 6 射程说时间线上号不复用、水位只增（`.claude/kb/decisions/08-核心索引结构.md:207` 的两句，见 0.1），所以同一条时间线上有：
- 判法 A（地板）：最新根记账里的水位 ≥ 每条历史行的 H。
- 判法 B（出生代截断）：最新根 inode 树里每条记录 (n, b)（和谱系上的墓碑 (n, b)），不存在历史行 (g, H) 使 H > n 且 b > g。号 n 小于 H 说明它在 g 之前就发过，出生代 b 晚于 g 的对象占着它就是重发。
- 「事件自己那条记录」：E175 每个发号事件有 (inode, new_birth)，把这一条单独拿去比判法 B（不看这一步镜像里别的记录），为的是不让同一段历史里更早的重发把这一步染红（装置：`research/prompts/m3-w3-r1-sonnet-model/w3-device.diff` 里 `w3_own_cut`）。
- 这是我自己想的判法，没有条款，池级 checker 里也没有对应的检查。推的部分：记账水位沿时间线不降（D23 已定项 14 的「水位」一格，0.1）⇒ A、B 在合法时间线上恒成立。

### 1.2 候选定义卡

C1 实例表行预留字节里记水位（两种值）：
- 字段与宽：行 88 字节，预留 66（见 0.2 的 `instance_table.rs:41`）；占 8 字节，偏移 22（1 + 4 + 8 + 8 + 1 = 22，算术），剩 58。读者今天不读预留（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:58`）、checker 只取前 21 字节（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:6336`），所以写进去不会被现有读者拒。
- 两种值：C1a = 重放之后那一版（所选根）的记账水位，行 (i, T_i, M_i) 即 (g, H) = (T_i, M_i)；C1b = 恢复那一刻根环里读得出的、该实例且 txg ≤ T_i 的根的水位最大值。
- 谁写、什么时候写：新实例在恢复里写行那次发布，行表接在所选根那张表后面：「rows.extend_from_slice(rows_written);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1217`）。落点：行来历结构「struct PreviousInstanceRow {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1256`）加一个字段，建它的地方「let previous_row = PreviousInstanceRow {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7008`），列行的函数「fn instance_rows_to_write(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:4861`）把它抄进所选根那一行、中间实例行（i, 0, 0）写 0；行结构「pub struct InstanceRow {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:21`）加字段，`to_bytes` 把「writer.skip(66);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:41`）拆成 8 + 58，`parse` 读它；checker 一侧「struct InstanceTableRow {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:743`）与 `parse_instance_table_row`（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:6336`）各加一格。C1b 另要一个像「fn highest_inode_number_watermark_in_the_ring<Device: BlockDevice>(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7438`）那样重建环里每条根的函数，恢复每次都跑一遍（今天只有管理员回退调它）。
- 什么时候删：跟着行删，行删在清扫发布上：「行可删当且仅当整轮清扫对池中每一个落点都得出了判定」（`.claude/kb/decisions/18-块里携带什么信息.md:315`，引句是该行前半句）。实例切换写行的路径 crates 里没有（0.2）。
- D15 层：行记录是「打包记录类型 4」「组件 ①」（`.claude/kb/freeze-layer-membership.md:50`），实例表单元本身「没有条款」（`.claude/kb/freeze-layer-membership.md:57`）。预留区新增字段按判定表第一行是 compat（`.claude/kb/decisions/15-格式冻结政策.md:185`）；但老写者每次写行都把预留写 0（`instance_table.rs:41`），老写者过一次手历史就静默归零，所以实际是「读者兼容、写者不兼容」（推的）。
- 每次发布多写：0 字节、0 单元（8 字节在已有的 88 字节行里，行表只在挂载那次整链重写，那一次本来就写）。C1b 另多读：每次恢复最多重建环里 24 条根（S = 8 乘 3 个区域，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:279` 与 :261，算术）。
- 崩溃点：不加写；挂载写行那一段（`crash_points_of_the_instance_table_sweep.rs` 与可写挂载各阶段的枚举）里的行字节变了，期望值要改；清扫发布删行时丢里程碑（见 K2）。

C2 记账里留历代水位行：
- 字段与宽：记账条目 34 字节（key 22 + value 8 + seq 4，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:145`），一个 16 KiB 叶装 ⌊(16384 − 159) / 34⌋ = 477 条（叶头 159，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:71`），今天一代 15 行，余 462 条（算术）。键仍是 (统计量, 树, 设备, 代)，代 = 发布的 txg。
- 两种留法：C2all = 每个「水位升了」的发布留一行 (g, H)，H 取该发布之后的水位；一行被更早的行盖住（更早的 g_early ≤ g 且 H_early ≥ H）就不留，留下的是严格上升的台阶（算术：盖住的行能抓到的 (n, b) 更早的那行都能抓到）。C2inst = 每个实例只留最高那一行，(g, H) = (第一次到最高水位的 txg, 最高水位)。
- 谁写、什么时候写：每次发布都写：记账行由「fn accounting_entries_of_this_publish(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:4879`）按这一代现算，水位那一行读上一版：「let inode_number_watermark = match previous {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:8026`）；历史行要从上一版记账行带进来（今天不带：D5 已定项 2「每条根的记账树只装这条根那一代的行，不留多代」，`.claude/kb/decisions/05-快照-空间记账机制.md:47`，要改成「水位历史行除外」）。管理员回退发布取现行那一版的记账行、只换水位那一格：「let accounting_entries = current」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7522`）与「value: if entry.statistic == STATISTIC_INODE_WATERMARK {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7530`），所以历史行默认跟现行那一版走。
- 什么时候删：写时不留被盖住的行；没有别的删法（推的：台阶压不得——把两行并成一行会让夹在中间的合法记录判红，见 K4 第 4 行）。
- D15 层：记账树节点布局第 4 层、键编码第 3 层都不变；多出来的是统计量登记表一项（「统计量登记表（标签值 + 维度 + 合并规则）」组件 ②，`.claude/kb/freeze-layer-membership.md` 同名行）和 D5 已定项 2 那句要改。读者兼容（老读者取代号最大那行），写者不兼容（老写者只写一代，历史静默归零）（推的）。
- 每次发布多写：叶里还有余量时 0 字节、0 单元（叶本来就整块重写）；历史行超过 462 条之后记账树从 1 层变 2 层：每次发布脏的节点多一个根，叶可能多一个（推的：新代的 15 行分在几个统计量段里，历史行只在水位那一段）。多层树里没改的节点是照抄不重写：「CodeTwoTreeNodeOrigin::CarriedFrom(previous_position) => {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:8959`）。
- 崩溃点：叶装得下时不加写；超过后加的写在同一段单元写里。

C3 另立只追加的发号记录：
- 字段与宽（我定的，推的）：新树，叶条目 = 起始号 8 + 个数 8 + 出生代 8 = 24 字节，键 8（起始号）；节点头按键宽 8 是 131（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:62`），一叶 ⌊(16384 − 131) / 24⌋ = 677 条（算术）。
- 谁写、什么时候写：建 inode 的发布写一条，起号读上一版水位：「let first_new_inode_number = previous.inode_number_watermark();」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:5905`，在 `publish_new_inodes` 里）；不建 inode 的发布照抄树根指针。回退发布取哪一版：树表条目里只有 extent、inode、deleted_inodes 三种换成目标那一版的：「root: if entry.kind == TREE_KIND_EXTENT」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7510`），别的种类默认跟现行那一版走。
- 什么时候删：不删（K4 第 4 行问能不能删）。
- D15 层：新增一棵 keyspace，树表是开放列表 ⇒ compat（`.claude/kb/decisions/15-格式冻结政策.md:190`）；键编码第 3 层、节点内部布局第 4 层各多一份；树表条目数今天写死 11：「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:188`），I-9.16 的第 ④ 道「十一种各恰好一条」跟着改 12，新池新建文件那次发布多发一个树 ID。
- 每次发布多写：建 inode 的发布 +1 个 16 KiB 叶（两份副本 32 KiB，叶装得下 677 条以内；超过后树长高，每次再 +1 个根）；journal 点名项 +1 个（56 字节，一条记录装 64 个，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:229`）；树表条目 +200 字节（树表单元本来就整块重写）。不建 inode 的发布 +0。
- 崩溃点：建 inode 的发布的单元写那一段多 2 个写（一个单元两份副本）；闭式「崩溃状态数 = 1 + Σ(2^|段| − 1)，|段| 按写数算」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/src/segments.rs:201`），那一段的 2^|段| 乘 4（算术）；E142 里一次第一个文件事务的单元段是 24 个写：「segments=24+2+1+2 closed_form=16777223」（`.claude/kb/experiments/142-新池新建文件的干跑.md:97`）。

C4 我提的两支：
- C4a 根记录里带一对 (HW, T_HW)：最高水位和第一次到它的 txg，共 16 字节。根记录 457 字节里没有空闲的预留（末尾 29 字节是加密的算法类型、nonce、MAC 留位，不能挪用：「writer.skip(12 + 16); // nonce、MAC：第一版留位全 0」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/root_record.rs:112`）），新字段只能放在 457 之后、写成 slot 宽补 0 的区里；根槽宽 = 物理块大小、mkfs 只拒窄于 457 的：「if u64::from(geometry.physical_block_size) < ROOT_RECORD_BYTES {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/make_filesystem.rs:359`）；放在 457 之后补 0 的区里，物理块 512 时空 55 字节，装得下 16（算术）。谁写：每次发布 `HW = max(上一版 HW, 这一版水位)`、水位首次抬到新高时 T_HW = 这次 txg，沿根传；管理员回退取现行那一版根的值，和树 ID 水位同一个做法：「.fold(current.root.tree_identifier_watermark, u64::max);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7747`）；恢复由所选根接。每次发布 0 字节多写（根槽本来整槽写）、0 单元；崩溃点不加写；什么时候删：不删，每次发布覆盖。新根段也得加这 16 字节，否则「崩在记录持久之后、根槽持久之前」由记录重建的根没有这对字段，树 ID 水位就在新根段里：「new_tree_identifier_watermark: root.tree_identifier_watermark,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/transaction.rs:1504`）；新根段 364 字节「pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 364;」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-format/src/lib.rs:220`）变 380，记录头 487 变 503，点名项仍装 ⌊(4096 − 503) / 56⌋ = 64（算术）；journal 记录格式是组件 ③：「冻结前置不早于第 2 层与落点粒度那一格，冻结时刻是提交它那份 toml 的 commit」（`.claude/kb/decisions/15-格式冻结政策.md:153`，引句是该行里的一段），新记录格式要走新记录类型加系统配置的 log-incompat 位。D15 层：根记录「没有条款」（`.claude/kb/freeze-layer-membership.md:55`）。读者兼容、写者不兼容（老写者写回 0，推的）。
- C4b 回退时给被越过的对象写墓碑：不取。理由有两条：回退目标可以是回退前的根（D23 已定项 14 候选集「根环里按 cur 的实例表判仍然有效」那一格，`.claude/kb/decisions/23-journal的角色与格式.md:382`，来回回退 A→B→A 合法），回到 A 时那些对象又活了，而 I-9.19 要求谱系上的墓碑不指活对象：「墓碑树里每条在谱系上的墓碑记录」（`.claude/kb/invariants.md:307`，引句是该行前半句）；另外回退要对比两棵 inode 树取差集，代价随对象数涨。

### 1.3 E175 V1 那两次（`g-long-V1.out` 第 389–390 行）逐候选（观测：模型在 V1 臂 G-long 第 24 段上的输出）

装置输出的事件行（我这一次重跑，与已发布的逐行相同，见 0.3）：「E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op92 inode=7 new_birth=77 older_births=[51] q9=Gone i9_29_at_step=holds」（`research/prompts/m3-w3-r1-sonnet-model/out/g-long-V1.out:3548`）与「E7RESULT name=event family=G-long arm=V1 index=24 kind=q1_same_timeline position=op95 inode=8 new_birth=78 older_births=[52] q9=Gone i9_29_at_step=holds」（`research/prompts/m3-w3-r1-sonnet-model/out/g-long-V1.out:3549`）。

同一步上各候选对「事件自己那条记录」的判（顺序 c1a c1b c2i c2all c3 c4，R 红，- 不红）：
- op92，inode 7 出生代 77：「own=7/77:-RRRRR i9_29=holds conflict_so_far=true wm=8 c1a=-B c1b=AB c2i=AB c2all=AB c3=R c4=AB」（`research/prompts/m3-w3-r1-sonnet-model/out/g-long-V1.out:3631`）。
- op95，inode 8 出生代 78：「own=8/78:-RRRRR i9_29=holds conflict_so_far=true wm=9 c1a=-B c1b=-B c2i=-B c2all=-B c3=R c4=-B」（`research/prompts/m3-w3-r1-sonnet-model/out/g-long-V1.out:3634`）。

读法：C1a 在这两步上整张镜像判红（`c1a=-B`），但事件自己那条记录它判不红（`own=` 里 c1a 是 `-`）；红是镜像里别的记录带出来的（推的：多半是同一段历史更早的重发，op19–op35 的号 5、6）。所以 C1a 抓不到这两次。
行表在这一步的样子（`rows=` 字段每项写成「实例:T/M_a/M_b」，实例 8 那一项的 T、M_a、M_b 是 55、7、9）：「55/7/9」（同一行 `rows=` 字段，`research/prompts/m3-w3-r1-sonnet-model/out/g-long-V1.out:3631`）：实例 8 最后一条被承认的根是 txg 55，那条根的水位 7（实例 8 里先建了号 7、8，出生代 51、52，都在 55 之内；之后实例 8 里的 V1 回退把水位压到了目标那一版的值），所以 C1a 记下的 M_a = 7；根环在挂载那一刻还留着回退之前的高水位根，C1b 的 M_b = 9，于是 C1b 的判法 B「H = 9 > 7 且 77 > 55」判红。这里「出生代 51、52 在实例 8 里」是行表上的 T 与出生代数字对出来的，推的，没有直接打印 51 属于哪个实例。

C1 的两处结构性缺口（观测加推的）：
- 行只在实例结束之后（下一次恢复）才有；最新实例自己没有行。定向段 W3-climb 在同一个会话里回退、建四个，没有挂载：「rows=-」整段为空，C1a 和 C1b 一步都不红（`research/prompts/m3-w3-r1-sonnet-model/out/d-V1.out:45` 的 `c1a=-- c1b=--`）。
- C1a 的值是「实例结束时所选根的水位」，实例里发生的回退早把它压低了，行里存的是压低之后的值；C1b 靠环里还留着回退之前的根，环 24 槽，再多 24 次发布就读不到了（推的：环长 24 是算术，没造出 24 次发布的定向段：我试了 `PublishOverwrite`、`PublishWithoutUnits`、`ring_root(0)`、抬 F 四种各凑 12 或 26 步，挂载时行表里的 T 都停在 9，说明那个状态上没做成发布；只有抬 F 那一种打出了原因，`NotApplicable(CurrentVersionWithoutFile)`，别三种没查原因，定向段没留在装置里）。

### 1.4 全部 V1 事件（观测，同一臂、同一份补丁；事件数与 E175 汇总的 `sum_q1` 逐族相同）

「事件自己那条记录」被各候选判红的个数（`research/prompts/m3-w3-r1-sonnet-model/expected/aggregate.txt` 汇总，命令 `bash rerun.sh compare` 的汇总段；模型里的名字对应：c1a = C1a，c1b = C1b，c2i = C2inst，c2all = C2all，c3 = C3，c4 = C4a）：

| 族（V1 臂） | 事件 | c1a | c1b | c2i | c2all | c3 | c4 | 其中 q9=Gone | Gone 里 c1a / c1b / 其余 |
|---|---|---|---|---|---|---|---|---|---|
| G-long 32 段 | 476 | 153 | 338 | 476 | 476 | 476 | 476 | 6 | 0 / 6 / 6 |
| F2 64 段 | 336 | 97 | 229 | 336 | 336 | 336 | 336 | 5 | 0 / 5 / 5 |
| F3a 1 464 段 | 975 | 52 | 172 | 975 | 975 | 975 | 975 | 0 | — |
| F1-broad 32 段 | 12 | 11 | 12 | 12 | 12 | 12 | 12 | 0 | — |
| F1-reuse 32 段 | 7 | 2 | 3 | 7 | 7 | 7 | 7 | 0 | — |
| F1-deleting 32 段 | 103 | 0 | 0 | 103 | 103 | 103 | 103 | 0 | — |
| F1-newest 32 段 | 16 | 10 | 14 | 16 | 16 | 16 | 16 | 0 | — |
| D 10 段（8 段登记的加 2 段定向） | 9 | 0 | 0 | 9 | 9 | 9 | 9 | 0 | — |
| 合计（用 `python3` 读汇总相加） | 1 934 | 325 | 768 | 1 934 | 1 934 | 1 934 | 1 934 | 11 | 0 / 11 / 11 |

产品臂（P）没有一个事件，各候选在 P 上判红的步数是 0：G-long 64 段 9 664 步、F3a 前 1 500 段 13 390 步、D 10 段 67 步（上面汇总文件里 `f3a-P`、`g-long-P`、`d-P` 各一段的「untainted red steps」都是 0）；F2 的 P 臂 256 段 15 616 步在早一版装置上跑过，同样 0（结果文件 `research/prompts/m3-w3-r1-sonnet-model/out/f2-P-earlier-device.out`）。合计 23 121 步加 15 616 步没有一步判红。

两条要说明的：
- c2all、c3 的 1 934 / 1 934 是上界：它们的历史用的是装置一路攒下的全部根与出生代账，等于 E175 判事件时用的那本账，不是某个产品容器的读法，数字一样是构造出来的。信息真有限的是 c1a、c1b、c2i、c4。
- c2i、c4 在这 1 934 个事件上也是全红，不等于它们没有漏洞：定向段 W3-climb 里水位爬过回退之前的最高水位之后，最后一步 c2i、c4 都判不红，镜像里却还活着三条重发的记录（号 2、3、4，出生代 10、11、12）：「own=- i9_29=violated conflict_so_far=true wm=6 c1a=-- c1b=-- c2i=-- c2all=-B c3=R c4=--」（`research/prompts/m3-w3-r1-sonnet-model/out/d-V1.out:48`，op8 那一步）；只有历史台阶完整的 c2all、c3 还红。原因（算术）：c2i、c4 的 g 取「第一次到最高水位」，水位爬过之后 g 往后挪，号 n < H 的记录 b 小于新的 g，判法 B 不再红；判法 A 因水位已回到 H 也不红。这一步 I-9.29 因回退之前的根还在环里判红，所以这一例在 E175 的口径下是「Still」，不是 Gone；它只说明 c2i、c4 的 B 判法有个爬过之后的盲区，不说明它们漏了 E175 那两次。

## 二、K2　和「崩溃恢复抛弃的时间线可以重发」怎么共存

### 2.1 谁分开两种重发（推的；机理读代码，没有一条合法抛弃重发的历史可跑）

D8 的处置是：同一条时间线上要红，被崩溃恢复抛弃的那一段里发过的号恢复之后可以重发（0.1）。四个候选分开它们的办法是同一个：历史只住在「所选根够得着的结构」里，新实例的第一次发布从所选根那一版接，被抛弃的那一段从来没进过新根的历史，检查时不用再判抛弃：
- C1：新行接在所选根那张表后面：「rows.extend_from_slice(rows_written);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:1217`），行值取自所选根（C1a）。「被抛弃」由行里的 T 切：「.any(|row| row.instance == instance && checkpoint_txg > row.selected_root_txg)」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1561`）。C1b 要把环里的根算进去，所以必须带同一刀 `txg ≤ T_i`；少了这一刀，环里被抛弃那一段的高水位会进行值，合法重发判红（误报）。
- C2：记账树是根的一部分，新实例第一次发布接的是重建出来的所选根那一版，历史行跟着那一版走。
- C3：新树的根指针住树表条目，随所选根；回退发布默认跟现行那一版走（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7510` 只把 extent、inode、deleted_inodes 三种换成目标的）。
- C4a：字段在根记录里，恢复接所选根的；管理员回退接现行那一版的。
- 对照：树 ID 水位反着做，取根环里全部根含被抛弃的 max：「被抛弃时间线上的根也算，它们仍承载那一代发过的号」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1341`，引句是该行注释里的半句），出处 D8 已定项 8 ②（`.claude/kb/decisions/08-核心索引结构.md:248`，引句：「树 ID 水位住根记录，取根环里全部根记录该字段的 max」）。树 ID 的处置是不重发，inode 号的处置是允许重发，所以 inode 号这边不能照抄树 ID 的取法。
- checker 一侧的好处（C2、C3、C4a）：判法 A、B 只读最新根自己的结构，不经过候选集的抛弃滤法；I-9.29 因此多出来的那道门（最新根的实例表读不出就判不了）不会落到这条新不变量上。C1 的历史本身就是实例表，没有这个好处。

### 2.2 它分不分得开，我这边量不到（观测）

- E175 的 26 份汇总（第一段 9 份、第二段 17 份）每一份的 `sum_q3b` 都是 0，P、V1、V2 三臂都一样：`grep -o 'sum_q3b=[0-9]*' summary-segment-one.out summary-segment-two.out | sort | uniq -c` 输出「9 summary-segment-one.out:sum_q3b=0」与「17 summary-segment-two.out:sum_q3b=0」。装置里没有一段历史造出过「跨被抛弃时间线的重发号」。
- 原因读代码：装置的「崩溃恢复抛弃根」操作造不出被抛弃的根：「读阶段重读一次仍读不出、取号之前拒可写（`MountError::NewerStateStillUnreadableAfterOneReread`），这一步因此造不出被抛弃的根。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-harness/src/history.rs:4533`）。
- 所以上面「合法重发不判红」只有机理，没有一次观测。要观测，得有一条崩溃状态用例：建 inode 的那次发布崩在 journal 记录写之前的每个前缀 → 可写挂载 → 再建 inode → 判。流可以改 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs`（它是 checker 档，我不跑），我没造。K4 第 6 行把它写成实验设计员的题。
- 一个相关的观察（读代码）：管理员回退取水位时看的是环里全部读得出的根，不分时间线：「let ring_roots = readable_roots(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7738`），`readable_roots` 的注释是「根环里全部自证过的根，每个可读根槽一条（回退候选集与影子账从这里取）。」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/recovery.rs:1374`）。所以崩溃恢复之后第一次管理员回退，水位可能被抬过被抛弃那一段发过的号（跳号，不重发）。对四个候选：C2、C3、C4a 的历史行里会出现一条 H 比时间线上真发过的号大的行；判法 A、B 仍成立（跳过的号没人再发，之后发的号都 ≥ 抬高后的水位），不会误报；C1 无关（回退不写行）。

### 2.3 管理员回退、实例切换、行回收三处，历史会不会丢，丢了是漏报还是误报

| 处 | C1 | C2 | C3 | C4a |
|---|---|---|---|---|
| 管理员回退（挂着时的一次向前发布，水位取 max，不写行，`.claude/kb/decisions/18-块里携带什么信息.md:310`） | 不写行，行表不变；回退压低的水位被实例结束时的恢复冻进 C1a（漏报，即 E175 那两次）。不会误报 | 历史行取现行那一版记账行（`mount.rs:7522` 起，只换水位那一格），不丢；有人把记账行整份改成取目标那一版时丢，丢了是漏报 | 默认跟现行走（`mount.rs:7510`），不丢；有人把种类加进「取目标」那三种时丢，漏报 | 取现行根的，不丢；漏报路径同 C2 |
| 实例切换 | crates 里没有写行路径（0.2）。条款里切换写的行「每个实例一行、取 ①–④ 里第一条适用的」（`.claude/kb/decisions/18-块里携带什么信息.md:312`）：C1 的值要为「被照旧事务的写序主人 k」取 k 最后发布的根的水位，要在挂载中重建那条根，读不出写 0 ⇒ 漏报 | 不受影响（重做接前一版） | 不受影响 | 不受影响 |
| 行回收（清扫发布，`.claude/kb/decisions/18-块里携带什么信息.md:315`） | 删行丢掉那一行的 (T, M)：漏报；剩下的行各自独立成立，不会误报。后面的行盖不住被删行的 B 判法（g 往后挪了），只盖得住 A；折叠规则（删行时把 H 并进下一行，推的，没实现）能保住 A | 与行表无关 | 无关 | 无关 |

三处丢历史都是漏报，没有哪一处会造出误报：历史行是下界（H 越小越松），丢行只会放松判法 A、B。

### 2.4 小结（K2 各格）

- 四个候选分开合法重发与同线重发靠的是「历史来自所选根」，不靠 checker 判抛弃；C1b 要补一刀 `txg ≤ T_i`，否则误报。合法重发那一形没有一次观测（2.2）。
- 回退、切换、回收三处只会丢历史（漏报），不会多历史（误报）；C1 三处都有口子，C2 / C3 / C4a 只有「有人改了回退取哪一版」这一个口子，该由变异挡（K3）。

## 三、K3　新不变量与 checker

### 3.1 文字（草，I-9.30；判法 A、B 的意思见 1.1，推的）

> I-9.30　水位不低于时间线上发过的号。设最新根的时间线上有一组发号历史行 (g, H)：g 是 txg，H 的意思是到 g 为止小于 H 的号都发过了（行从哪个结构读，见各候选）。① 最新根记账里的 inode 号水位 ≥ 每一行的 H。② 最新根 inode 树里每条记录 (n, b)，与最新根谱系上的每条墓碑 (n, b)：不存在一行使 H > n 且 b > g。③ 历史盖住最新根自己（C2、C3、C4a 才有）：最新根水位大于 mkfs 之后的初值时，历史里至少有一行 H ≥ 水位且 g ≤ 最新根的 txg。读历史的结构读不全、字段解不开 → 部分没判并点名；水位仍是初值、历史没有一行 → 不适用；老镜像（没开这套历史）整条不适用。推翻条件：找到一段合法历史，I-9.30 在其中某一步判红，而那一步的重发是崩溃恢复抛弃那一段里发过的号。

- 为什么分三句：①（A）抓「水位被压低」本身，水位压低的那一步就红，不用等重发；②（B）抓重发了的记录，水位已经爬回来也红；③ 是「历史根本没写」要红的唯一抓手——历史没写时 ①② 都没有行可比，读成「不适用」，变异放过去。C1 没有 ③：最新实例自己没有行，行里的 0 与「没记」同值，所以「不写」在 C1 上不红（推的）。
- 另一种改法（推的，没比）：A 并进 I-9.6 变第三半，B 并进 I-9.29（把历史行当作额外的版本放进跨版本表）。不并进去的好处是判别力分开记数、变异各一条（K3 第 3.3 节）。

### 3.2 checker 落点（冻结树）

- 判法 A：挨着 I-9.6 那一判，那里已经有最新根记账里的水位：「let inode_number_watermark = accounting」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:11668`），判在「judgements.judge("I-9.6", watermark > largest_inode, || {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:11676`）。读不全时的报法照 I-9.6 墓碑那一半：「Err(gap) => judgements.part_of_the_range_not_judged("I-9.6", gap.reason()),」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:11706`），不判红；I-9.5 的共用判法逐字是「读不全的版报 `part_of_the_range_not_judged`（射程里判不了，不判红）」（`.claude/kb/invariants.md:293`，引句是该行里的一句）。
- 判法 B：挨着 I-9.29 的跨版本判：函数「fn judge_one_object_birth_per_inode_number(」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:10926`），调用处「judge_one_object_birth_per_inode_number(&object_births, judgements);」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:10383`）。那一遍已经按版本读 inode 记录与谱系上的墓碑，B 在那里取最新那一版的记录比历史行。
- 历史行的来源，各候选各一处：C1 扩现有的行结构「struct InstanceTableRow {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:743`）与「instance_table_rows: Vec<InstanceTableRow>,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:431`）；C2 把记账行解成「self.accounting」按 (统计量, 设备) 覆盖的写法（`walk.rs:2250` 附近）旁边另收历史行；C3 照分配记录树另写一个走读（「fn walk_allocation_record_tree(」，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:3608`）；C4a 在根记录解析处多读 16 字节（实例表指针是「parse_node_pointer(&record[170..256])」，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:4765`）。
- C2 在这里有一处现有检查要改：I-3.13「每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`」（`.claude/kb/invariants.md:155`，引句是该行里的一句）。checker 里是「row_generation == accounting_version_txg,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:2239`）：带着更早代的历史行，今天这一判逐行红。两条路：放宽 I-3.13 并改 D5 已定项 2 那句「不留多代」（历史行不重写、叶不动，1.2 的成本按这一路算）；或者保留 I-3.13、历史行每代用新代重写（每次发布把全部台阶重写一遍，成本随台阶数涨，推的，K4 第 2 行量）。
- 收尾：`IMPLEMENTED_INVARIANTS` 84 → 85（「pub const IMPLEMENTED_INVARIANTS: [&str; 84] = [」，`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/image.rs:98`；违例位容量 128，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 里 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`，装得下）；GPU 判器要补一个内核（I-9.6 墓碑那一半的 GPU 判并在下标 41 那一位，见 I-9.6 那一行）；`invariants.md` 加一行；`crates/mutations.tsv` 加变异（下一节）。这些我没动。

### 3.3 变异（草案：写历史的代码还不存在，锚点取今天的相邻行）

一条已经能写出来的（变异就是 E175 的 V1 臂本身，补丁逐字取 `patch-V1.diff`）：

| 项 | 内容 |
|---|---|
| 名 | W3：回退那次发布的水位取目标那一版的（E175 V1） |
| 文件 | `crates/singlefs-core/src/mount.rs` |
| 原文（恰好命中一次） | `    let previous =\n        version_to_publish_the_rollback_after(current, &target_version, inode_number_watermark);`（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/mount.rs:7757`–`7758`） |
| 替换文 | `    let previous =\n        version_to_publish_the_rollback_after(current, &target_version, { let _ = inode_number_watermark; target_version.inode_number_watermark() });` |
| 必须红的测试 | 待写：在 `rollback_by_a_forward_publish` 那组用例里加一条「建 3 个 inode → 回退到更早的根 → 把回退前的高水位根挤出候选集（抬 F）→ 重新挂载 → 再建」，I-9.30 红 |

- 现状（观测）：今天没有任何已有的 mutations.tsv 行是这条（`grep -c 'target_version.inode_number_watermark' crates/mutations.tsv` 在冻结树里输出 0），E175 的 Gone 事件上 I-9.6、I-9.29 都不红（`g-long-V1.out:389`、`:390` 的 `i9_29_at_step=holds`，0.3）。
- 这条变异在各候选下的结果（观测，1.3、1.4）：C2、C3、C4a 红（G-long V1 的 6 个 Gone 事件、F2 的 5 个都红）；C1b 红（11 个 Gone 事件都红，但靠回退发生时环里还留着高水位根）；C1a 不红（0 / 11）。
- 「把历史写错或不写」的变异，每个候选一条（锚点是历史行写进去的相邻行，实现后补原文）：
  - C2、C4a：发布时不带上一版的历史（`HW = wm`，或历史行传空）⇒ ③ 红，红在第二次建 inode 的那次发布之后（推的）。
  - C2、C3：回退发布的历史取目标那一版的（`mount.rs:7522` 起的记账行、或 `mount.rs:7510` 的种类表里加一个种类）⇒ ① 或 ② 红，红在回退那一步（推的）。
  - C3：建 inode 的发布不追加记录 ⇒ ③ 红（推的）。
  - C1：行里写 0 ⇒ 不红（推的：没有 ③）；行里写成「所选根的水位 − 1」⇒ 只有 B 判到 T_i 之前的记录时才红，不稳。所以 C1 交不出「不写就红」的变异。

## 四、K4　代价要不要先量

### 4.1 已有的数（算术，除注明外是推的；观测只有 E175 那一半）

| 量 | C1 | C2 | C3 | C4a |
|---|---|---|---|---|
| 每次发布多写的字节 / 单元 | 0 / 0 | 叶装得下（r ≤ 462 行）时 0 / 0；之后见下一行 | 建 inode 的发布 +1 个 16 KiB 叶（两份副本 32 KiB）；不建的 0 | 0 / 0 |
| 容量拐点 | 无；行宽 88 不变，片数 = max(1, ⌈行数 / 369⌉)：「pub fn instance_table_pages_for_rows(rows: usize) -> usize {」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-core/src/instance_table.rs:122`）；行数 1 / 369 / 370 / 739 / 10 000 → 1 / 1 / 2 / 3 / 28 片，每次挂载写行整链重写 64 / 64 / 128 / 192 / 1 792 KiB（两份副本，算术）；这个数 C1 不改 | 叶 477 条、余 462 行；内部节点 ⌊(16384 − 159) / 108⌋ = 150 个孩子；2 层装 477 × 150 = 71 550 行 | 叶 677 条；内部节点 ⌊(16384 − 131) / (8 + 86)⌋ = 172 个孩子；2 层装 677 × 172 = 116 444 条 | 无 |
| 记账树变高（C2） | — | 462 行之后从 1 层变 2 层，每次发布脏的节点多一个根、可能多一个叶（推的：历史行聚在水位那一段）；保留 I-3.13 那一路则整张台阶每次发布重写 | — | — |
| 崩溃状态数 | 不变 | 叶装得下时不变；变高之后同一段单元写里多 k 个写 | 建 inode 的发布的单元写段多 2 个写，该段闭式乘 4（2^26 / 2^24，算术）；E142 的事务流单元段 24 个写「segments=24+2+1+2 closed_form=16777223」（`.claude/kb/experiments/142-新池新建文件的干跑.md:97`） | 不变 |
| 每次挂载多读 | C1b 最多重建环里 24 条根（S = 8 乘 3 个区域）；C1a 不多读（所选根已经重建） | 0 | 0 | 0 |
| journal 点名项 | 0 | 0（变高之后 +k） | 建 inode 的发布 +1 个，一条记录装 64 个 | 0 |

量过的（观测）：E175 的历史都很短，水位峰值最大 53（G-long P 64 段），一段历史里挂载次数最大 42（命令：对 `g-long-P.out` 的 `watermark_peak`、`mounts_or_recoveries_applied` 取最大，结果「watermark_peak_max=53」「mounts=1847 (max 42)」），所以 E175 上看不到任何容量拐点；要看拐点只能用计数模型。

### 4.2 要量的问题（单行，可原样发给实验设计员）

- Q1｜C1b 每次可写挂载为取 M_b 多读几条根、几个 16 KiB 单元、多少毫秒｜自变量：环里读得出的根数（1 / 8 / 24）× inode 数（1 / 233 / 31 000，叶数一层与多层）｜因变量：重建一条根的读次数、读字节、挂载挂钟增量｜取样点：环满 24 槽的池各取一遍｜判停：增量是不是挂载本来读量的整数倍以上（是则 C1b 的 M_b 路被否）。
- Q2｜C2 的每次发布脏节点数与树高｜自变量：历史行数 r（0 / 100 / 462 / 463 / 5 000 / 71 551）× 路线（放宽 I-3.13 且历史行不重写 / 保留 I-3.13 每代重写整张台阶）｜因变量：每次发布脏的记账节点数（16 KiB 单元）、写字节（含两份副本）、树高｜取样点：上述 r 各一个｜判停：两条路线的脏节点数在 r = 462 前后各差几个；形态照 E109、E110（纯算术、钉绝对值、变异表、进 `replay.sh`）。
- Q3｜C3 的追加记录数、叶数与树高随创建发布数的增长｜自变量：创建发布数（10^2 … 10^6）× 每次发布建 inode 数 k（1 / 10 / 100）｜因变量：记录数、叶数、树高、建 inode 的发布多写的单元数｜取样点：677 与 116 444 两个拐点两侧｜判停：追加到 116 445 条时树高 2 → 3 那一次发布多写几个单元。
- Q4｜台阶压不压得下去（C2all、C3 共用）｜自变量：每次发布建 inode 数 k、实例数、回退次数｜因变量：「被更早的行盖住就不留」之后留下的行数 / 建 inode 的发布数｜取样点：E175 类 150 步历史与 10^4 步合成负载｜判停：比值大于 1 不可能（一次发布至多一行）；比值接近 1 说明压不动，台阶无界，要不要给上界交用户。
- Q5｜各候选对第 0 层脚本流的闭式状态数增量｜自变量：候选（C1 / C2 / C3 / C4a）× 脚本流（`post_mkfs_stream` 与建 inode 的事务流）｜因变量：`closed_form_state_count` 的值与放量档的抽样数｜取样点：E142 的那几条流｜判停：一次算术，不要实验。
- Q6｜K2 的合法重发一形：崩溃恢复抛弃一次建 inode 的发布之后再建 inode，I-9.30 在每个前缀状态上红几次（应 0）｜自变量：崩溃点（建 inode 那次发布录制流的每个前缀）× 候选｜因变量：I-9.30 与 I-9.6、I-9.29 的违例数｜取样点：照 `crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs` 的流改（checker 档）｜判停：任一前缀红一次就推翻该候选的「历史来自所选根」。
- Q7｜C1 行回收之后丢的覆盖｜自变量：回收的行数 × 删行时折叠规则（不折叠 / H 并进下一行）｜因变量：E175 G-long V1 的 476 个事件里，删行之后 C1b 还抓到几个｜取样点：在模型里按 `instance_table_sweep` 的删行条件删最老的 j 行（j = 0 / 5 / 10）｜判停：覆盖掉得比不折叠多出一截就够用。

### 4.3 哪些候选之间还要用户再挑一次（推的，不替主 agent 判）

- C1a 抓不到那两次（1.3），不进岔路。C1b 靠环里还留着回退之前的根，是环窗口上的事（1.3 的缺口），且没有「不写就红」的变异（3.3）。
- C2 与 C3 信息量一样（台阶），差在容器：C2 要改 D5 已定项 2 与 I-3.13（3.2）、动统计量登记表；C3 要加一棵树，牵动树表条目数 11 → 12、I-9.16 第 ④ 道、新池新建文件那次发布的树 ID 数。这两个之间要看 Q2、Q3。
- C4a 信息量比台阶小（一对），E175 上抓得和台阶一样多（1.4），盲区在水位爬过之后（1.4 末）；形态上最便宜（0 单元），但动根记录和 journal 的新根段（组件 ③，1.2）。C4a 与 C2inst 的差别只在存在哪、盲区同形。
- 三条之间的取舍有一个数字上的分叉：要不要为「爬过之后」那个盲区付台阶无界的价（Q4）。

## 五、冲突并排抄（每处左边是条款或现有检查，右边是被判的候选在这一步的输出）

- C1a 对 E175 那两次：
  - 应该：「含管理员回退抛弃的那一段（回退那次发布的水位取 max，那一段里发过的号不重发）」（`.claude/kb/decisions/08-核心索引结构.md:207`）；那段时间线上的重发在单镜像上应判红。
  - 实际：「own=7/77:-RRRRR i9_29=holds conflict_so_far=true wm=8 c1a=-B c1b=AB c2i=AB c2all=AB c3=R c4=AB」（`research/prompts/m3-w3-r1-sonnet-model/out/g-long-V1.out:3631`），C1a 对事件自己那条记录是 `-`。
- C1（两种值）对同一会话里的回退加重发：
  - 应该：同上，水位压低的那一步就该红。
  - 实际：「own=2/10:--RRRR i9_29=violated conflict_so_far=true wm=3 c1a=-- c1b=-- c2i=AB c2all=AB c3=R c4=AB rows=-」（`research/prompts/m3-w3-r1-sonnet-model/out/d-V1.out:45`），行表为空，C1 一步不红。
- C2inst、C4a 对水位爬过原高水位之后：
  - 应该：同上，镜像里还活着重发的号 2、3、4，应判红。
  - 实际：「own=- i9_29=violated conflict_so_far=true wm=6 c1a=-- c1b=-- c2i=-- c2all=-B c3=R c4=-- rows=-」（`research/prompts/m3-w3-r1-sonnet-model/out/d-V1.out:48`），c2i、c4 不红。
- C2（保留历代行）对现有的 I-3.13：
  - 现有：「每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`」（`.claude/kb/invariants.md:155`，引句是该行里的一句）。
  - 候选带着更早代的行时，这一判逐行红：「row_generation == accounting_version_txg,」（`/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree/crates/singlefs-checker/src/walk.rs:2239`）。

## 六、判定一览

判定三种：一致（候选在这一格上与条款、现有代码一致，或达到格里的要求）、冲突（没达到或与现有检查相撞，并排抄在第五节）、规则没说。

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 C1a | 冲突 | 行里记所选根的水位：E175 那两次抓不到（0 / 11 个 Gone 事件，325 / 1 934 个事件），实例里的回退早把值压低了；最新实例没有行 |
| K1 C1b | 冲突 | 靠恢复那一刻环里还留着回退前的根：那两次抓得到（11 / 11），但 768 / 1 934，定向段里同会话回退加重发一步不红，环 24 槽之外读不到 |
| K1 C2inst | 一致（有盲区） | 每个实例记最高水位：1 934 / 1 934 抓到、P 臂 0 误报；水位爬过原高水位之后判不红（定向段）；要改 D5 已定项 2 与 I-3.13 |
| K1 C2all | 一致 | 台阶完整：1 934 / 1 934（上界，历史用的是装置攒下的全部根）；无界，没有可证的压法（Q4） |
| K1 C3 | 一致 | 同台阶，容器是新树：1 934 / 1 934（上界）；建 inode 的发布 +1 单元，崩溃状态闭式乘 4，树表条目数 11 → 12 |
| K1 C4a | 一致（有盲区） | 根记录带 (HW, T_HW)：1 934 / 1 934、P 臂 0 误报；盲区同 C2inst；0 单元多写，但动根记录和 journal 新根段（组件 ③） |
| K1 C4b | 规则没说 | 回退时给被越过的对象写墓碑：与 A→B→A 回退加 I-9.19 相撞，没有跑，不取 |
| K1 每次多写 / 崩溃点 | 一致 | 逐候选见 1.2 与 4.1，算术；没量的写了推的 |
| K2 分开合法与同线重发 | 规则没说 | 机理上历史来自所选根，四个候选同一办法；没有一次观测（E175 26 份汇总 `sum_q3b=0`，装置造不出被抛弃的根）；C1b 要补 `txg ≤ T_i` |
| K2 管理员回退 | 一致 | 三处都只会丢历史（漏报），不会多历史（误报）；C1 在回退上有口子（E175 那两次），C2 / C3 / C4a 只在「有人改了回退取哪一版」时丢 |
| K2 实例切换 | 规则没说 | crates 里没有切换写行的路径，只有条款（`.claude/kb/decisions/18-块里携带什么信息.md:312`）；C1 要在挂载中重建 k 最后发布的根，别的候选不受影响 |
| K2 行回收 | 一致 | 只有 C1 丢覆盖（漏报）；折叠规则是推的 |
| K3 新不变量文字 | 一致 | I-9.30 草（A、B、③）：文字、读不出时报部分没判、老镜像不适用都写了；C1 没有 ③ |
| K3 checker 落点 | 一致 | `walk.rs` 的 I-9.6 旁（A）与 I-9.29 旁（B），各候选的行来源各一处；C2 要改 I-3.13 |
| K3 变异 | 冲突（C1）/ 一致（其余，草案） | V1 臂本身是一条能写出的变异；「不写就红」只有 C2 / C3 / C4a 有 ③ 可靠；C1 交不出 |
| K4 代价要不要先量 | 一致 | 7 行问题单；已有的是算术，E175 历史太短（水位峰值 53），看不到拐点，C2 / C3 的树高与多写要计数模型 |

## 七、没做什么

- 没替主 agent 采纳或出判决；上面「一致 / 冲突 / 规则没说」是正推对着格里的要求和现有代码比的结果，不是选候选。
- 没写产品代码，也没改冻结树：候选的历史值是模型从装置观察到的根里算出来的（`research/prompts/m3-w3-r1-sonnet-model/README.md`），不是产品里某个写者写下的字节。C1b、C2、C3 的写者与读者代码都不存在，成本数是算术，不是量出来的。
- 没造「崩溃恢复抛弃一次建 inode 的发布之后再建」的状态，合法重发那一形没有观测（2.2、Q6）。
- B 判法没比谱系上的墓碑（装置的 `RootFacts` 里没有墓碑），只比 inode 记录。
- 没跑 checker 档（`singlefs-checker-tier` 的测试）、没跑 GPU 判器、没跑形式化验证、没动 `crates/mutations.tsv`；变异是草案。
- F2 的 P 臂 256 段是早一版装置（还没有 c4 与定向段）上跑的，结果文件拷在 `research/prompts/m3-w3-r1-sonnet-model/out/f2-P-earlier-device.out`，`rerun.sh` 不重跑它，也没有 `expected` 里的预期值。
- 复核不了的：产品里 C2 的真实脏节点数（要计数模型或实现）；C1b 的挂载读成本（Q1）；C3 的树高（Q3）。
- 删了的（`du -sh` 先记的大小）：`/home/fy5090/.local/share/singlefs/work/m3-w3-r1-sonnet/repo` 22M 与 `repoP` 22M（冻结树拷贝）、`target` 505M 与 `targetP` 505M（编译目录）、`/home/fy5090/.local/share/singlefs/work/m3-w3-r1-sonnet-rerun` 1.1G（`rerun.sh` 从空目录跑通的那一份，含两份冻结树拷贝与两个编译目录）。要复核：`SINGLEFS_REPO=/home/fy5090/.local/share/singlefs/work/m3-w3-r1/tree bash research/prompts/m3-w3-r1-sonnet-model/rerun.sh prepare`，接着 `build`、各步 `run`、`compare`（约 25 分钟，分步调，每步打出 DONE 为止）。
- 没删 `/home/fy5090/.local/share/singlefs/work/m3-w3-r1-sonnet`（22M）：草稿目录，里面是生成模型用的 `tools`、各次编译日志 `build*.log`、`out`（早一版装置与中间版本的原始输出）、`stage`（入库那 11 份结果文件的暂存），归草稿；要不要留由主 agent 定，不用时整个删掉。

## 八、什么现象会推翻上面的结论

- 「C1a 抓不到那两次」：在 `g-long-V1.out` 第 24 段（`E175_INDICES=24`）上，模型给出 C1a 对事件自己那条记录判红；或行表里出现一行 M_a ≥ 8 且 T < 77。
- 「C1b 抓得到、但靠环窗口」：造出环转过一圈之后（24 次以上发布）再挂载、再重发的定向段，C1b 仍然判红。
- 「C2inst、C3、C4a 在 E175 上不漏」与「P 臂 0 误报」：任一 P 臂历史上某一步某候选判红；或 V1 臂的某个事件（上面 1 934 个之一）某候选判不红。后者对 C2all、C3 不成立是构造出来的，只对 C2inst、C4a 有意义。
- 「C2inst、C4a 有爬过之后的盲区」：W3-climb 的 op8 那一步（`out/d-V1.out:48`）c2i 或 c4 判红。
- 「历史来自所选根，所以不用 checker 判抛弃」：一段合法历史（崩溃恢复抛弃一次建 inode 的发布之后再建），任一候选的 I-9.30 判红；或写行 / 新实例第一次发布接的不是所选根那一版。
- 「回退、切换、回收只会漏报不会误报」：找到一行历史行的 H 大于时间线上真实发过的最大号 + 1，而之后合法地发出了一个小于 H 的号（2.2 的跳号不算，跳过的号没人发）。
- 「C2 与 I-3.13 相撞」：在带更早代行的记账树上 `walk.rs:2239` 那一判不红。
- 「C3 崩溃状态乘 4」：建 inode 的事务流单元段的写数不是 24 → 26（Q5 一次算术）。
- 「C1 交不出『不写就红』的变异」：找到一个从最新根自己读得到的、C1 的应有行值（例如让最新实例也带一行）。

## 九、交回前的核对

`python3 research/scripts/cite-check.py research/prompts/m3-w3-r1-sonnet-output.md --root ~/.local/share/singlefs/work/m3-w3-r1/tree --root /home/fy5090/code/singlefs` 末行：见交回消息（每次改报告都重跑；这一节不再写数，免得写进来的数过期）。
