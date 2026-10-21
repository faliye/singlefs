# 里程碑四第 2 批改动全文（2026-10-13）

四份实现员补丁原样拼接（都以工作树 `m4-predev` 合入前的树为基线；`m4-rebuild-writer-2` 的含第 1 批 `m4-rebuild-writer` 的全部）。

## m4-rebuild-writer-2：crates.patch

````diff
diff --git a/crates/singlefs-checker/invariants/01-块自描述.md b/crates/singlefs-checker/invariants/01-块自描述.md
--- a/crates/singlefs-checker/invariants/01-块自描述.md
+++ b/crates/singlefs-checker/invariants/01-块自描述.md
@@ -15,6 +15,7 @@
 | I-1.10 | 码 2 条目宽等于字段表宽 | 任一码 2 索引节点，头里自述的**条目宽**（D18（块里携带什么信息） 已定项 7 的自描述三段之一：key 宽 1 + 条目数 2 + 条目宽 2，偏移表在 D18（块里携带什么信息） 已定项 18、载荷布局在 D8（核心索引结构） 已定项 11）**等于**它那棵树登记的条目字段表宽度：extent 叶记录 112、inode 内部条目 120、分配记录 20、记账条目 34、记账树内部节点 108（key 22 + 子指针 86）。条目宽是**盘上读来的值**，`index_node_view` 只判了它 ≥ key 宽（extent 24、inode 8、分配记录 10、记账 22），够不着「装得下整条记录」——不判就按字段表的固定偏移切，切片当场越界（增补 3 第 6 件 panic 面普查第二族，C476（盘上内容可控时走得到的 panic 有 21 处） R1 / R3 / R4 / R13）。条目宽为 0 时条目数必须为 0（`声明长度 = 条目数 × 条目宽` 在宽 0 时恒成立，挡不住它），否则判该节点损坏（2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做）。判定顺序与 I-1.7（打包容器合法与判定顺序） 同一纪律：**先于**按字段表解条目。⚠️ **中央映射树还不判**：D19（块指针的结构与宽度预算） 已定项 10 已定映射树 key 一律 27 字节（码 2 / 码 3 末尾补零）、条目一律 55 字节，但读者只拒窄于 55 的条目，`field_table_width_of_tree_kind` 不认种类 5，宽于 55 的映射节点不判损坏，账在 C307（映射树两种 key 宽怎么装进一棵定宽 key 的树）；**树表条目（200 字节）两边都没写**，而走读照样按固定偏移切树表节点的条目——要不要把射程扩到它，账在 C476（盘上内容可控时走得到的 panic 有 21 处） | 已实现（池级 checker `crates/singlefs-checker` 的 `walk::walk_tree_table_entry`；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；条目宽为 0 时条目数非 0 那一条 2026-09-27 实审 A3-checker-2 落地：`index_node_view_judging_a_zero_entry_width`（`crates/singlefs-checker/src/walk.rs`）在解码失败理由是 `Verdict::EntryWidthZeroWithEntries`、且这棵树登记了条目字段表（`EntryFieldTableRegistration::Registered`）时报违例；树表条目与中央映射树不在射程内，只记走读失败；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`） |
 | I-1.11 | 映射 key 与单元头相符 | 任一中央映射条目，key 的类标签属于 D18（块里携带什么信息） 已定项 11 的登记表，且等于它的位置条目指的单元头里的类标签；key 里的出生 txg、实例代号、尾段（码 1 是写序的事务号 6 字节；码 2 / 码 3 是出生序号 4 字节，其后 2 字节补零恒 0）分别等于那个单元头里的诞生代号、写序实例代号、事务号或出生序号；key 里的出生树等于那个单元头里的出生树（码 1 / 码 3 取类身份段偏移 43，码 2 取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定）。不等判损坏。 | 已实现（池级 checker `walk::check_pool_image`，判定在 `crates/singlefs-checker/src/walk.rs` 的 `walk_central_mapping_entries`；key 各段的偏移按 D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的字段表另写一份，不用实现的解析；判到的射程：类标签在登记表里且等于被指单元头的，出生 txg / 实例代号 / 尾段等于单元头的，码 2 / 码 3 尾段后 2 字节补零恒 0，码 1 / 码 3 的出生树等于单元头偏移 43，码 2 的出生树随 C289（码 2 的出生树取哪个字段没写） 不判；映射根全零（无文件的一版）时报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`（映射里数据单元那条 key 的出生 txg 3 → 4）与 `crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs`。编号已落到代码：`crates/singlefs-checker/src/image.rs` 的常量 `MAPPING_KEY_MATCHES_THE_UNIT_HEADER` 写的值就是这个编号本身，I-1.11（映射 key 与单元头相符）） |
 | I-1.12 | 指针出生树等于被指单元头 | 任一被走读的指针，偏移 34 的出生树等于被指单元头里的出生树：码 1、码 3 取头偏移 43；码 2 的出生树取哪个字段随 C289（码 2 的出生树取哪个字段没写） 定，不判（只写进状态列）。指针各段落点依据 D19（块指针的结构与宽度预算） 已定项 8（指针携带被指单元映射 key 除类标签外的各段）。inode 内部条目到叶容器那一类由 I-9.2（条目身份与子头相符） 罩住，不重判。 | 已实现（池级 checker `walk::check_pool_image` 走读里的 `judge_pointer_birth_tree_against_the_unit_header`，只在两处调用：实例表链上每一片、extent 数据单元；码 1、码 3 取头偏移 43，码 2 不判，码 2 那一支不登记成「判了一部分」，那会让每份干净镜像整条报不适用；没有走到这两处指针的镜像报不适用；坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`；对应的账是 C294（指针里的 key 各段与被指单元头相等没有不变量）；`research/prompts/m3-inv-b-r1-main-verification.md` 第二节第 3 行） |
+| I-1.13 | 扫描重建与遍历方向的索引相同 | 只从扫描方向扫出的码 1 / 码 3 已发布单元头（加当前版本那几片码 3 容器的记录）重建的三张派生表——中央映射（每个当前版本、没被墓碑压住的码 1 单元与每片当前版本的码 3 容器，实例表的片除外（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12 豁免它）→ 扫到它的副本的落点）、每个 inode 的数据单元集合（(inode, 偏移) → 那个数据单元当前版本的映射 key）、inode 记录集合（inode 号 → 当前版本 inode 叶容器里那条记录原样）——与遍历方向从最新根读出的三张表（中央映射只取码 1 / 码 3 的条目）逐项相同（`.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」；D21（权威态与派生态的分界） 已定项 13）。已发布按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P2 的谓词判（挂载根取最新根，实例表行取它指着的那一版：这是重建方读根的唯一一处，与 I-1.8（归并后版本全序） 那一遍同一来源）；当前版本按定案 P3 择（码 1 写序最大者，码 3 (诞生代号, 实例代号, 事务号) 最大者，与 I-1.8 码 3 的全序键同一套）；墓碑判 key 死亡（D11（索引节点要不要留消息缓冲区） 已定项 4，第一版只有整对象墓碑）。落点：两边的落点集合相同即对得上，不同时两边各只留读得出的那几份再比（遍历方向按位置条目里的整单元校验和读回，扫描重建按头里的载荷校验和读回）。扫描方向与遍历方向共享的只有格式解析（D13（验证路线） 已定项 12） | 已实现（池级 checker `walk::check_pool_image` 末尾的 `judge_scan_rebuild_agrees_with_the_walked_index`，与别的不变量同一份判定；扫描重建本身在 `crates/singlefs-checker/src/rebuild.rs`，纯函数，`use` 清单与非测试代码里指名 crate 内一项的路径由那份文件末尾的白名单单测钉住（C410（checker 两个方向共享什么没有检查） 的会红形态）；违例写每张对不上的表差几处与第一处（表名、key、两边的值）；报不适用的：根环里没有自证过的根、最新根的实例表沿链读不全、遍历方向的索引已被别的不变量判损坏（比对之前查 I-1.11（映射 key 与单元头相符）、I-1.12（指针出生树等于被指单元头）、I-4.2（无被引用未提交块）、I-5.1（物理范围不重叠） 有没有违例，表在 `walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`，理由写明是哪一条红，主 agent 2026-10-13 定）、最新根下面的树表 / extent 树 / inode 树 / 中央映射树读不全，以及扫描重建停在条款没写之处（`rebuild::ScanRebuildNotDecided`：码 1 同一对象身份两版写序相同（C464（I-1.8 对码 1 的全序键写窄了））、码 3 两组全序键相同、同一版副本载荷校验和不同、当前版本的容器解不开、谱系树有记录即回退过的镜像（C615（扫盘重建对挂着时的回退没有输入））、退役树有记录（C768（退役记录的扫描重建读侧没有消费者））、墓碑区间不是整对象、删除中（墓碑压着的对象还有记录）、数据单元落在它那条记录的 size 之外（文件变短留下的，I-1.9（清扫水位内无可读垃圾） 未实现）、同一 (inode, 偏移) 上两个对象、同一 inode 号在两片当前容器里）；用例与必红世界（改掉映射叶一条条目的落点、清零码 1 单元头的对象 key）在 `crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs`；GPU 判器的覆盖表收了这一条（下标 92）、内核不置位，GPU 与 CPU 对拍对这一条不比（用户 2026-10-13 定「先进覆盖表但内核不置位，记欠账」，照 I-4.4（意图只描述目标态） 下标 73 的先例；GPU 扫描重建内核是欠账）） |
 
 > I-1（块自描述） 类的适用范围是**任一单元**（数据与元数据都在内，
 > D20（承重面：单元的原子性与自包含） 口径），判定字段的落点与宽度在
diff --git a/crates/singlefs-checker/invariants/README.md b/crates/singlefs-checker/invariants/README.md
--- a/crates/singlefs-checker/invariants/README.md
+++ b/crates/singlefs-checker/invariants/README.md
@@ -26,14 +26,14 @@
 ## 状态
 
 **当前里程碑是「删除与多次 COW」。** 磁盘格式已落成字节：mkfs、新池新建文件的写路径与冷启动恢复在 `crates/singlefs-core`；checker 在 `crates/singlefs-checker`，与实现只共享常量模块（D13（验证路线） 已定项 5）。
-池级 checker（`walk::check_pool_image`）判 92 条（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 的长度；状态列写「已实现」的行用 `grep -ch '^| I-.*已实现' crates/singlefs-checker/invariants/0*.md | awk '{s+=$1} END{print s}'` 数，也是 92 条，其中 I-3.5（引用区间的精确性） 是部分实现：出现规则已实现（下标 65），③ ④（deadlist 那两条）未实现，I-4.4（意图只描述目标态） 在清单里（下标 73）、第一版恒报不适用；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间）），2026-10-04 加 I-9.27（树表条目的头 ID 取值）、I-5.5（从 mkfs 水位建树那一版的 t1 落点） 与 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），2026-10-05 加里程碑三·3 删除线那五条：下标 69 的 I-9.5（删除队列里的号仍在树里）、下标 70 的 I-3.4（可用空间扣待删占用）、下标 71 的 I-4.7（删除中 inode 有账）、下标 72 的 I-4.6（中间态其余不变量成立） 与下标 73 的 I-4.4（意图只描述目标态），2026-10-06 加下标 83 的 I-3.14（实例表每片记录形态合法），2026-10-09 加下标 84 的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），2026-10-09 第 50 批加下标 85 到 89 的退役树那五条：I-9.31（退役树条目身份与子头相符）、I-9.32（退役记录合法）、I-9.33（谱系上的退役记录按身份不重复）、I-9.34（谱系上的退役记录不指本版活叶） 与 I-9.35（退役完备），同日第 53 批加下标 90、91 的 I-9.36（产品镜像里非最右谱系与退役容器装满） 与 I-9.37（测试镜像里谱系与退役容器不超旋钮值）），状态列写「已实现」的就是这 92 条（I-3.5（引用区间的精确性） 只算出现规则那一半，I-4.4（意图只描述目标态） 的判定恒是不适用）：除 I-4.4（意图只描述目标态） 外每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`，删除线那几份的搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`），崩溃点重放的每个崩溃状态都跑它。
+池级 checker（`walk::check_pool_image`）判 93 条（`crates/singlefs-checker/src/image.rs` 的 `IMPLEMENTED_INVARIANTS` 的长度；状态列写「已实现」的行用 `grep -ch '^| I-.*已实现' crates/singlefs-checker/invariants/0*.md | awk '{s+=$1} END{print s}'` 数，也是 93 条，其中 I-3.5（引用区间的精确性） 是部分实现：出现规则已实现（下标 65），③ ④（deadlist 那两条）未实现，I-4.4（意图只描述目标态） 在清单里（下标 73）、第一版恒报不适用；第一版 23 条，2026-09-16 里程碑「覆盖写、释放、回退与复用」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽），2026-09-18 加 I-3.9（释放代落在停止引用它的那一格区间里）、I-9.14（树表条目的诞生 txg 跨根不变） 与 I-5.4（分配记录罩住的槽互不相交），2026-09-21 加 I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 与 I-8.7（实例内事务号不重号），2026-09-22 加 I-1.10（码 2 条目宽等于字段表宽）、I-9.6（水位大于两处最大号）、I-9.12（分隔 key 落在孩子区间之外）、I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块），2026-09-23 加 I-8.8（前缀里的事务不被切开） 与 I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号），2026-09-24 加 I-7.9（回退下界 F 不高于抬 F 的上限）、I-9.15（inode 记录的 blocks 等于 ⌈size ÷ 512⌉）、I-3.11（已分配减 defer 等于最新根走读） 与 I-8.9（一次发布的记录序号连续且只有末条带标志），2026-09-25 加过 I-7.10（已停用·回退见证表删了） 与 I-7.11（已停用·见证择根删了）、2026-09-26 随回退见证删掉退役，同日加 I-7.12（系统配置 F 不低于同盘根上的 F），2026-09-27 加 I-1.11（映射 key 与单元头相符），同日加 I-7.13（系统配置池级字段在读者收的范围里） 与 I-9.16（树表条目按树 ID 严格升序且合发号次序），2026-09-28 加 I-7.14（系统配置本盘设备号等于盘在池里的身份），2026-10-02 加里程碑三二·1 那一批 16 条（I-1.12（指针出生树等于被指单元头）、I-3.12（实例表行 T 低于引用它的根的 txg）、I-3.13（记账行代段等于这一版的 txg）、I-4.1（重放后树可遍历）、I-4.9（非最新候选根引用的单元不晚于它）、I-7.5（根槽按判定宽度对齐）、I-7.15（各盘所选系统配置槽设备数相同）、I-7.16（系统配置槽世代号与槽号的关系）、I-7.17（根槽位置等于 txg 算出的位置）、I-8.1（环几何够大）、I-8.5（tail 槽有效集非空）、I-8.10（记录新根段与同代根相同）、I-8.11（点名项与被指单元头相同）、I-8.12（同实例记录的 txg 不减）、I-9.25（树表条目的 previous_snapshot_txg 取值）、I-9.26（inode 改动计数落在出生代与容器诞生代号之间）），2026-10-04 加 I-9.27（树表条目的头 ID 取值）、I-5.5（从 mkfs 水位建树那一版的 t1 落点） 与 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同），2026-10-05 加里程碑三·3 删除线那五条：下标 69 的 I-9.5（删除队列里的号仍在树里）、下标 70 的 I-3.4（可用空间扣待删占用）、下标 71 的 I-4.7（删除中 inode 有账）、下标 72 的 I-4.6（中间态其余不变量成立） 与下标 73 的 I-4.4（意图只描述目标态），2026-10-06 加下标 83 的 I-3.14（实例表每片记录形态合法），2026-10-09 加下标 84 的 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），2026-10-09 第 50 批加下标 85 到 89 的退役树那五条：I-9.31（退役树条目身份与子头相符）、I-9.32（退役记录合法）、I-9.33（谱系上的退役记录按身份不重复）、I-9.34（谱系上的退役记录不指本版活叶） 与 I-9.35（退役完备），同日第 53 批加下标 90、91 的 I-9.36（产品镜像里非最右谱系与退役容器装满） 与 I-9.37（测试镜像里谱系与退役容器不超旋钮值），2026-10-13 里程碑四四·2 加下标 92 的 I-1.13（扫描重建与遍历方向的索引相同）），状态列写「已实现」的就是这 93 条（I-3.5（引用区间的精确性） 只算出现规则那一半，I-4.4（意图只描述目标态） 的判定恒是不适用）：除 I-4.4（意图只描述目标态） 外每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`，删除线那几份的搭建与改法在 `crates/singlefs-harness/tests/common_deletion_in_progress/mod.rs`），崩溃点重放的每个崩溃状态都跑它。
 I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322（取号那一步的屏障怎么放没有条款） 定案改成 ① ②，崩溃点重放里 0 个状态判红；I-3.1（已分配统计对得上） 与 I-7.8（根记录树 ID 水位不低于全池最大树 ID） 的 checker 读法 2026-09-14 由用户收尾弹窗定甲，写在各自那一行。
 其余条目仍写「未实现」：它们要覆盖写、释放、回退与复用（释放、重用、残留）、journal 一侧的那一类检查、快照或加密，新池新建文件的镜像上判不出东西。
 
-违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；用了 92 位；`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列 92 个下标，就是 `IMPLEMENTED_INVARIANTS` 的每一条（下标 65 的 I-3.5（引用区间的精确性） 走读内核在回退候选集的版本都走完之后逐对判；66 是 I-9.27（树表条目的头 ID 取值）、67 是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）、68 是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）；69 是 I-9.5（删除队列里的号仍在树里）、70 是 I-3.4（可用空间扣待删占用）、71 是 I-4.7（删除中 inode 有账）、72 是 I-4.6（中间态其余不变量成立），这四条由扫描内核的删除态扫描在回退候选集的每一版上判，读不全的版不置位；73 是 I-4.4（意图只描述目标态），第一版池级 checker 恒报不适用、内核不置位；84 是 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），由扫描内核在最新根上判；85 是 I-9.31（退役树条目身份与子头相符），由走读内核在退役树的每条内部条目上判；86 到 89 是 I-9.32（退役记录合法）、I-9.33（谱系上的退役记录按身份不重复）、I-9.34（谱系上的退役记录不指本版活叶） 与 I-9.35（退役完备），由扫描内核在墓碑与谱系那一遍的每一版上判，I-9.35（退役完备） 扫完各版再跨相邻两版比，读不全的那一部分不置位；90 是 I-9.36（产品镜像里非最右谱系与退役容器装满），由扫描内核在墓碑与谱系那一遍判；91 是 I-9.37（测试镜像里谱系与退役容器不超旋钮值），编进旋钮值的那一份内核才判、没装旋钮的那一份恒不置位），没有「内核还没判」的那张表；I-9.6（水位大于两处最大号） 墓碑那一半也在扫描内核里判、并进下标 41 那一位；编译期断言覆盖表按下标升序、一条不缺一条不重（第 k 项是 k）（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。
+违例位是 128 位（4 个 u32 字，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `POOL_CHECKER_VIOLATED_BIT_CAPACITY`），位 i 对 `IMPLEMENTED_INVARIANTS` 的第 i 条（`pool_checker_violated_bits` 读成 u128）；用了 93 位；`POOL_CHECKER_COVERED_INVARIANT_INDEXES` 列 93 个下标，就是 `IMPLEMENTED_INVARIANTS` 的每一条（下标 65 的 I-3.5（引用区间的精确性） 走读内核在回退候选集的版本都走完之后逐对判；66 是 I-9.27（树表条目的头 ID 取值）、67 是 I-5.5（从 mkfs 水位建树那一版的 t1 落点）、68 是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）；69 是 I-9.5（删除队列里的号仍在树里）、70 是 I-3.4（可用空间扣待删占用）、71 是 I-4.7（删除中 inode 有账）、72 是 I-4.6（中间态其余不变量成立），这四条由扫描内核的删除态扫描在回退候选集的每一版上判，读不全的版不置位；73 是 I-4.4（意图只描述目标态），第一版池级 checker 恒报不适用、内核不置位；84 是 I-9.30（inode 号水位的台阶行与水位、记录、谱系对得上），由扫描内核在最新根上判；85 是 I-9.31（退役树条目身份与子头相符），由走读内核在退役树的每条内部条目上判；86 到 89 是 I-9.32（退役记录合法）、I-9.33（谱系上的退役记录按身份不重复）、I-9.34（谱系上的退役记录不指本版活叶） 与 I-9.35（退役完备），由扫描内核在墓碑与谱系那一遍的每一版上判，I-9.35（退役完备） 扫完各版再跨相邻两版比，读不全的那一部分不置位；90 是 I-9.36（产品镜像里非最右谱系与退役容器装满），由扫描内核在墓碑与谱系那一遍判；91 是 I-9.37（测试镜像里谱系与退役容器不超旋钮值），编进旋钮值的那一份内核才判、没装旋钮的那一份恒不置位；92 是 I-1.13（扫描重建与遍历方向的索引相同），第一版池级 checker 在 CPU 上判、内核不置位，GPU 与 CPU 对拍对这一条不比（用户 2026-10-13 定「先进覆盖表但内核不置位，记欠账」）），没有「内核还没判」的那张表；I-9.6（水位大于两处最大号） 墓碑那一半也在扫描内核里判、并进下标 41 那一位；编译期断言覆盖表按下标升序、一条不缺一条不重（第 k 项是 k）（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行判定的位宽加宽与 GPU 内核补齐）。
 
 <!-- invariant-count -->
-现共 115 条在用（编号至 I-9.37（测试镜像里谱系与退役容器不超旋钮值），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。
+现共 116 条在用（编号至 I-9.37（测试镜像里谱系与退役容器不超旋钮值），另有六条退役，编号不回收）。这一句是当前条数的权威登记位；「## 历史版本」里那几句「现共 N 条在用」记的是当时的数，不跟着改。
 
 ## 待补
 
diff --git a/crates/singlefs-checker/src/image.rs b/crates/singlefs-checker/src/image.rs
--- a/crates/singlefs-checker/src/image.rs
+++ b/crates/singlefs-checker/src/image.rs
@@ -109,7 +109,12 @@
 /// （`walk::check_pool_image_given_the_container_capacity_knobs`）。GPU 判器的扫描内核同判 I-9.36；第 56 批 G6 起旋钮值经扫描内核源码前面的常量
 /// 传进 GPU（`crash_judge_gpu` 的 `JudgeKernelSources::of_the_judge_given_the_container_capacity_knobs`），装了旋钮的那一棵 GPU 同判 I-9.37
 /// （墓碑旋钮同判 I-9.22）；崩溃点重放与放量走产品那一份（三棵都没装），I-9.22、I-9.37 在那里照旧不置位。
-pub const IMPLEMENTED_INVARIANTS: [&str; 92] = [
+///
+/// 第 93 条（下标 92）是只读扫描重建那一条（里程碑四四·2，题面 `.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」）：
+/// I-1.13（扫描重建与遍历方向的索引相同），同样只往末尾追加。判在 `walk::check_pool_image` 的末尾，扫描重建本身在 `rebuild` 模块。
+/// GPU 判器的覆盖表（`crash_judge_gpu` 的 `POOL_CHECKER_COVERED_INVARIANT_INDEXES`）收了下标 92、内核不置位，GPU 与 CPU 对拍对这一条不比
+/// （用户 2026-10-13 定「先进覆盖表但内核不置位，记欠账」，照下标 73 的 I-4.4 的先例）。
+pub const IMPLEMENTED_INVARIANTS: [&str; 93] = [
     "I-1.1",
     "I-1.2",
     "I-1.3",
@@ -202,7 +207,14 @@
     RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
     NON_RIGHTMOST_LINEAGE_AND_RETIREMENT_CONTAINERS_ARE_FULL_ON_A_PRODUCT_IMAGE,
     LINEAGE_AND_RETIREMENT_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOBS_ON_A_TEST_IMAGE,
+    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
 ];
+
+/// I-1.13（扫描重建与遍历方向的索引相同）：只从扫描方向扫出的单元头（加当前版本那几片码 3 容器的记录）重建的三张派生表——中央映射、
+/// 每个 inode 的数据单元集合、inode 记录集合——与遍历方向从最新根读出的三张表逐项相同（`.claude/kb/milestone/04-fourth-txn.md`
+/// 「四·1 题面」；D21（权威态与派生态的分界） 已定项 13，D13（验证路线） 已定项 12）。违例写第一处差异：表名、key、两边的值；
+/// 最新根读不出、实例表读不全、遍历方向的索引已被别的不变量判损坏、回退过的镜像与扫描重建停在条款没写之处的报不适用并写理由。登记在 `crates/singlefs-checker/invariants/01-块自描述.md`。
+pub const SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX: &str = "I-1.13";
 
 /// I-1.12：走读跟随的一条指针，偏移 34 的出生树等于被指单元头里的出生树——码 1、码 3 取头偏移 43；码 2 取哪个字段随 C289 定，
 /// 不判（inode 内部节点指着叶容器的那一类由 I-9.2 罩住，不重判）。
diff --git a/crates/singlefs-checker/src/lib.rs b/crates/singlefs-checker/src/lib.rs
--- a/crates/singlefs-checker/src/lib.rs
+++ b/crates/singlefs-checker/src/lib.rs
@@ -8,6 +8,7 @@
 
 pub mod image;
 pub mod position_addressed;
+mod rebuild;
 pub mod walk;
 
 use singlefs_format::{
@@ -700,6 +701,132 @@
     })
 }
 
+/// 码 1 数据单元与码 3 打包记录单元：I-1.8（归并后版本全序） 罩得到的两类，也是扫描重建（`rebuild` 模块）第一版重建的两类。
+/// 码 2 不在这里（`crates/singlefs-checker/invariants/01-块自描述.md` I-1.8 逐字「码 2 不进这条」：它的写序已收窄成只有实例代号，
+/// 多版在条目级合并、不在容器级择版本）。头里各段的偏移是格式解析——扫描方向与遍历方向之间唯一许共享的那一样
+/// （D13（验证路线） 已定项 12），所以住在 crate 根上，`walk` 与 `rebuild` 各自读它，谁也不调谁。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub(crate) enum ContentUnitClass {
+    /// 码 1 数据单元。
+    Data,
+    /// 码 3 打包记录单元。
+    PackedRecord,
+}
+
+/// 一类内容单元的头里要读的那几段。码 1 按 D18（块里携带什么信息） 已定项 7 的字段表（五元组 42：类标签 42、树 43、对象 51、
+/// 对象出生代 59、锚点偏移 67；诞生代号 75、fsid 83、写序 91、载荷 CRC 101、明文头末尾 105）；码 3 按已定项 11 的类身份段表
+/// （标签 42、出生树 43、类型 51、容器号 53、容器出生代 61、记录数 69、记录宽 71、诞生代号 73、fsid 81、载荷校验和 89、写序 93、
+/// 出生序号 103、明文头末尾 107）。
+pub(crate) struct ContentUnitHeaderOffsets {
+    pub(crate) plain_header_end: usize,
+    pub(crate) payload_checksum: usize,
+    pub(crate) filesystem_identifier: usize,
+    pub(crate) birth_tree: usize,
+    pub(crate) birth_txg: usize,
+    pub(crate) write_order: usize,
+    /// 码 3 的打包记录类型（2 字节）；码 1 头里没有这一段。
+    pub(crate) packed_record_type: Option<usize>,
+    /// 码 3 的出生序号（4 字节，中央映射 key 的尾段，D19（块指针的结构与宽度预算） 已定项 10）；码 1 头里没有这一段，
+    /// 它的 key 尾段是写序的事务号。
+    pub(crate) birth_sequence: Option<usize>,
+    /// 类身份段里与版本无关的那一段：码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代）。
+    pub(crate) object_key: std::ops::Range<usize>,
+    /// 归并键的两段：「类身份段全部字段含写序」**去掉载荷校验和那一段**。
+    /// 载荷校验和不进归并键——它正是「同一组的成员载荷相同」要比的那个量，进了键这一条就恒真。
+    pub(crate) merge_key_before_payload_checksum: std::ops::Range<usize>,
+    pub(crate) merge_key_after_payload_checksum: std::ops::Range<usize>,
+}
+
+/// 写序那 10 字节里事务号低 48 位从第 4 字节起（D18（块里携带什么信息） 已定项 7：实例代号 4 + 事务号 6）。
+pub(crate) const WRITE_ORDER_TRANSACTION_OFFSET: usize = 4;
+
+/// 中央映射 key 里各段的偏移（D19（块指针的结构与宽度预算） 已定项 6 / 10：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6；
+/// 尾段码 1 是事务号 6 字节，码 2 / 码 3 是出生序号 4 + 补零 2）。按字段表另写一份，不用实现的解析；
+/// 遍历方向（`walk` 判映射 key 与单元头相符）与扫描重建（`rebuild` 从单元头现算 key）各自读它。
+pub(crate) const MAPPING_KEY_UNIT_CLASS_OFFSET: usize = 0;
+pub(crate) const MAPPING_KEY_BIRTH_TREE_OFFSET: usize = 1;
+pub(crate) const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 1 + 8;
+pub(crate) const MAPPING_KEY_INSTANCE_OFFSET: usize = 1 + 8 + 8;
+pub(crate) const MAPPING_KEY_TAIL_OFFSET: usize = 1 + 8 + 8 + 4;
+/// 码 2 / 码 3 的尾段是出生序号 4 字节，其后补零 2 字节（D19（块指针的结构与宽度预算） 已定项 10「末尾补零到 27」）。
+pub(crate) const MAPPING_KEY_TAIL_PADDING: std::ops::Range<usize> =
+    MAPPING_KEY_TAIL_OFFSET + 4..MAPPING_KEY_TAIL_OFFSET + 6;
+
+/// 码 1 五元组里对象、对象出生代、锚点偏移三段的位置（D18（块里携带什么信息） 已定项 7：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67）。
+pub(crate) const DATA_UNIT_OBJECT_OFFSET: usize = 51;
+pub(crate) const DATA_UNIT_OBJECT_BIRTH_OFFSET: usize = 59;
+pub(crate) const DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET: usize = 67;
+
+/// 码 3 的打包记录类型（D18（块里携带什么信息） 已定项 11 的登记表）：墓碑 1、inode 2、退役 3（记录宽 34，判决 m3-retire-r1 第三节第 2 条）、
+/// 实例表 4、谱系 5（D18（块里携带什么信息） 已定项 24 第 5 条，不复用登记过的 3）。
+pub(crate) const PACKED_TYPE_TOMBSTONE: u16 = 1;
+pub(crate) const PACKED_TYPE_INODE: u16 = 2;
+pub(crate) const PACKED_TYPE_RETIREMENT: u16 = 3;
+pub(crate) const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
+pub(crate) const PACKED_TYPE_LINEAGE: u16 = 5;
+
+/// inode 记录 140 里对象出生代（偏移 8）与 `size`（偏移 40）的位置（D8（核心索引结构） 已定项 6 的偏移表），按字段表另写一份、不用实现的解析。
+pub(crate) const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+pub(crate) const INODE_RECORD_SIZE_OFFSET: usize = 40;
+
+/// 墓碑记录 40 字节里前四段的偏移（`.claude/kb/layout/03-third-txn.md`「二、墓碑记录」；D18（块里携带什么信息） 已定项 24 第 2 条：
+/// 对象 ID 8（被删的 inode 号）+ 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8），按字段表另写一份、不用实现的解析。
+pub(crate) const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
+pub(crate) const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+pub(crate) const TOMBSTONE_RECORD_RANGE_START_OFFSET: usize = 16;
+pub(crate) const TOMBSTONE_RECORD_RANGE_END_OFFSET: usize = 24;
+/// 第一版墓碑只许整对象：区间 [0, 2⁶⁴ − 1]（D18（块里携带什么信息） 已定项 24 第 2 条）。
+pub(crate) const TOMBSTONE_WHOLE_OBJECT_RANGE_START: u64 = 0;
+pub(crate) const TOMBSTONE_WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
+
+impl ContentUnitClass {
+    pub(crate) fn of(unit_class_tag: u8) -> Option<Self> {
+        match unit_class_tag {
+            1 => Some(ContentUnitClass::Data),
+            3 => Some(ContentUnitClass::PackedRecord),
+            _outside_the_two_content_unit_classes => None,
+        }
+    }
+
+    pub(crate) fn header_offsets(self) -> ContentUnitHeaderOffsets {
+        match self {
+            ContentUnitClass::Data => ContentUnitHeaderOffsets {
+                plain_header_end: 105,
+                payload_checksum: 101,
+                filesystem_identifier: 83,
+                birth_tree: 43,
+                birth_txg: 75,
+                write_order: 91,
+                packed_record_type: None,
+                birth_sequence: None,
+                object_key: 42..75,
+                merge_key_before_payload_checksum: 42..101,
+                merge_key_after_payload_checksum: 105..105,
+            },
+            ContentUnitClass::PackedRecord => ContentUnitHeaderOffsets {
+                plain_header_end: 107,
+                payload_checksum: 89,
+                filesystem_identifier: 81,
+                birth_tree: 43,
+                birth_txg: 73,
+                write_order: 93,
+                packed_record_type: Some(51),
+                birth_sequence: Some(103),
+                object_key: 42..69,
+                merge_key_before_payload_checksum: 42..89,
+                merge_key_after_payload_checksum: 93..107,
+            },
+        }
+    }
+
+    pub(crate) fn name(self) -> &'static str {
+        match self {
+            ContentUnitClass::Data => "码 1 数据",
+            ContentUnitClass::PackedRecord => "码 3 打包记录",
+        }
+    }
+}
+
 /// journal 记录头的偏移（D23（journal 的角色与格式） 已定项 4 的字段表）：magic 4 + 类型 2 + 算法类型 1 之后是记录标志 1；
 /// 事务号 8 与提交标记 1 之后紧跟本次发布内序号 4，再是反向链 4、载荷校验和 4、新根段 364、fsid 8、MAC 16，头到 487 为止。
 const JOURNAL_MAGIC: &[u8; 4] = b"SFSJ";
diff --git a/crates/singlefs-checker/src/rebuild.rs b/crates/singlefs-checker/src/rebuild.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-checker/src/rebuild.rs
@@ -0,0 +1,1192 @@
+//! 扫描重建（只读，不写回盘）：不读根环与树表，只从扫描方向扫出的单元头（加当前版本那几片码 3 容器的记录）重建「索引」。
+//! 第一版三张派生表：中央映射、每个 inode 的数据单元集合、inode 记录集合（`.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」；
+//! D21（权威态与派生态的分界） 已定项 13「救援、换树、迁移是同一条路径」）。分配记录树、记账树是派生态（D21（权威态与派生态的分界）
+//! 已定项 5），第一版不重建、不比对；deadlist / livelist 等 C301（deadlist 与 livelist 的态别没写） 定了再接。遍历方向从最新根读出的
+//! 三张表与这里重建的逐项比，比对在 `walk` 模块 `check_pool_image` 的末尾（I-1.13（扫描重建与遍历方向的索引相同））。
+//!
+//! 这里只有纯函数，分两步：[`choose_current_versions`] 从单元头择出每个对象身份的当前版本；调用方照
+//! [`CurrentVersions::packed_containers_whose_records_are_read`] 把那几片码 3 的整单元字节读来，交给 [`rebuild_index_tables`]
+//! 现算三张表。条款没写的那几格不猜：交回 [`ScanRebuildNotDecided`] 的一个成员（名字写明是哪条条款没定、第一版不认），调用方报不适用。
+//!
+//! 与遍历方向共享的只有格式解析（D13（验证路线） 已定项 12）：这里只用 `singlefs_format` 的常量与 crate 根上解单元头、
+//! 解码 3 记录的函数与偏移，不调走树的函数（`walk` 模块与它的节点缓存）。本文件末尾的单测把 `use` 清单与白名单逐项比，
+//! 并核非测试代码里每一处指名 crate 内一项的路径都落在白名单里（C410（checker 两个方向共享什么没有检查） 的会红形态）。
+//!
+//! 条款没逐字写、按条款推出来的四处读法（重建这一侧与 `walk` 里比对那一侧各两处）：
+//! - 读法：没有 inode 记录也没有墓碑压着的码 1 单元照认领，进数据单元集合与中央映射（主 agent 2026-10-13 认，条款没写）。
+//! - 读法：中央映射同一 key 两边的落点集合不同时，两边各只留读得出的副本再比，掉一份副本的降级镜像不判红（主 agent 2026-10-13 认，条款没写）。
+//! - 读法：三张表在两边都是空表（mkfs、暖机之后）判成立，不判不适用：比对真跑了（主 agent 2026-10-13 认，条款没写）。
+//! - 读法：每个 inode 的数据单元集合那张表的值比映射 key，不比落点，落点归中央映射那张表（主 agent 2026-10-13 认，条款没写）。
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use singlefs_format::{
+    DATA_UNIT_BYTES, DATA_UNIT_PAYLOAD_OFFSET, INODE_RECORD_BYTES, LINEAGE_RECORD_BYTES,
+    MAPPING_KEY_BYTES, RETIREMENT_RECORD_BYTES, TOMBSTONE_RECORD_BYTES,
+};
+
+use crate::{
+    packed_unit_view, read_six_byte_unsigned, read_u16, read_u32, read_u64, ContentUnitClass,
+    DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET, DATA_UNIT_OBJECT_BIRTH_OFFSET,
+    DATA_UNIT_OBJECT_OFFSET, INODE_RECORD_OBJECT_BIRTH_OFFSET, INODE_RECORD_SIZE_OFFSET,
+    MAPPING_KEY_BIRTH_TREE_OFFSET, MAPPING_KEY_BIRTH_TXG_OFFSET, MAPPING_KEY_INSTANCE_OFFSET,
+    MAPPING_KEY_TAIL_OFFSET, MAPPING_KEY_TAIL_PADDING, MAPPING_KEY_UNIT_CLASS_OFFSET,
+    PACKED_TYPE_INODE, PACKED_TYPE_INSTANCE_TABLE, PACKED_TYPE_LINEAGE, PACKED_TYPE_RETIREMENT,
+    PACKED_TYPE_TOMBSTONE, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET,
+    TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET, TOMBSTONE_RECORD_RANGE_END_OFFSET,
+    TOMBSTONE_RECORD_RANGE_START_OFFSET, TOMBSTONE_WHOLE_OBJECT_RANGE_END,
+    TOMBSTONE_WHOLE_OBJECT_RANGE_START, WRITE_ORDER_TRANSACTION_OFFSET,
+};
+
+/// 一份单元副本的落点：(设备, 起点槽)。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub(crate) struct UnitPlacement {
+    pub(crate) device: u32,
+    pub(crate) slot: u64,
+}
+
+/// 扫描方向扫出的一份码 1 / 码 3 单元副本：magic、类标签、头校验和、fsid 都过了，已发布谓词也判成已发布——
+/// 这两步由调用方做（`walk` 的扫描方向，与 I-1.8（归并后版本全序） 那一遍同一份扫描结果）。
+pub(crate) struct PublishedUnitCopy {
+    pub(crate) placement: UnitPlacement,
+    /// 明文头原样（`[0, 这一类的明文头末尾)`）。
+    pub(crate) plain_header: Vec<u8>,
+}
+
+/// 中央映射 key 27 字节（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10）：码 1 = 类标签 1 + 出生树 8 + 出生 txg 8 + 写序 10；
+/// 码 3 = 类标签 + 出生树 + 出生 txg + 实例代号 4 + 出生序号 4 + 补零 2。
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub(crate) struct CentralMappingKey(pub(crate) Vec<u8>);
+
+impl CentralMappingKey {
+    /// 判红时写进说明的样子：类标签、出生树、出生 txg、实例代号与尾段各自读出来。
+    pub(crate) fn describe(&self) -> String {
+        let key = &self.0;
+        let tail = match ContentUnitClass::of(key[MAPPING_KEY_UNIT_CLASS_OFFSET]) {
+            Some(ContentUnitClass::Data) => format!(
+                "事务号 {}",
+                read_six_byte_unsigned(key, MAPPING_KEY_TAIL_OFFSET)
+            ),
+            Some(ContentUnitClass::PackedRecord) | None => {
+                format!("出生序号 {}", read_u32(key, MAPPING_KEY_TAIL_OFFSET))
+            }
+        };
+        format!(
+            "（码 {}、出生树 {}、出生 txg {}、实例 {}、{tail}）",
+            key[MAPPING_KEY_UNIT_CLASS_OFFSET],
+            read_u64(key, MAPPING_KEY_BIRTH_TREE_OFFSET),
+            read_u64(key, MAPPING_KEY_BIRTH_TXG_OFFSET),
+            read_u32(key, MAPPING_KEY_INSTANCE_OFFSET),
+        )
+    }
+}
+
+/// 类身份段里与版本无关的那一段（码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代））：同一个对象身份的各版共用它。
+#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub(crate) struct ObjectIdentity(pub(crate) Vec<u8>);
+
+/// 每个 inode 的数据单元集合那张表的 key：extent key (0, inode, 偏移) 的后两段（D8（核心索引结构） 已定项 3：偏移是文件字节偏移）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+pub(crate) struct DataUnitPosition {
+    pub(crate) inode: u64,
+    pub(crate) offset_in_bytes: u64,
+}
+
+/// 重建出的三张表。
+pub(crate) struct RebuiltIndexTables {
+    /// 每个已发布、当前版本、活着的码 1 单元与每片当前版本的码 3 容器（实例表的片除外：它不进映射，D19（块指针的结构与宽度预算）
+    /// 已定项 8 / 已定项 12）→ 扫到它的副本的落点。
+    pub(crate) central_mapping: BTreeMap<CentralMappingKey, BTreeSet<UnitPlacement>>,
+    /// (inode, 偏移) → 那个数据单元当前版本的映射 key。
+    pub(crate) data_units_of_each_inode: BTreeMap<DataUnitPosition, CentralMappingKey>,
+    /// inode 号 → 当前版本 inode 叶容器里那条记录 140 字节原样。
+    pub(crate) inode_records: BTreeMap<u64, Vec<u8>>,
+}
+
+/// 扫描重建停在条款没写的地方：每个成员说清是哪条条款没定、第一版不认。调用方把 [`ScanRebuildNotDecided::reason`] 报成不适用，
+/// 不当成立、也不当违例。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub(crate) enum ScanRebuildNotDecided {
+    /// 码 1 同一对象身份上两版写序逐字节相同：C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 择写序最大者择不出，
+    /// 而码 1 的全序键要不要加诞生代号打头停在 C464（I-1.8 对码 1 的全序键写窄了）。
+    DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided,
+    /// 码 3 同一对象身份上两组 (诞生代号, 实例代号, 事务号) 相同：I-1.8（归并后版本全序） ② 不成立，择新键择不出。
+    PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18,
+    /// 同一版（归并键相同）的副本头里的载荷校验和不同：I-1.8（归并后版本全序） ① 判损坏、不择。
+    CopiesOfOneVersionCarryDifferentPayloadsUnderI18,
+    /// 当前版本那片码 3 容器（墓碑、inode、退役、谱系）没有一份副本解得开。
+    CurrentPackedContainerDecodesFromNoCopy,
+    /// 当前版本的谱系树容器里有记录：这是管理员回退过的镜像，按谱系父链择现行的读侧没有条款（C615（扫盘重建对挂着时的回退没有输入）），
+    /// 第一版不接。
+    RolledBackImageWithoutTheLineageReadSideOfC615,
+    /// 当前版本的退役树容器里有记录：叶容器退役过，退役记录的读侧没有条款（C768（退役记录的扫描重建读侧没有消费者）），第一版不接。
+    RetiredLeafContainersWithoutTheReadSideOfC768,
+    /// 墓碑的区间不是整对象 [0, 2⁶⁴ − 1]：第一版只写整对象（D18（块里携带什么信息） 已定项 24 第 2 条），区间墓碑怎么压 key 没有条款。
+    TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,
+    /// 墓碑压着的对象还有 inode 记录（删除中：意图那次写了墓碑、分批还没删完）：墓碑判 key 死亡（D11（索引节点要不要留消息缓冲区）
+    /// 已定项 4），而遍历方向这时照样读得到它没删完的数据单元——中间态上扫描重建认不认领它们，条款没写。
+    DeletionInProgressNotWrittenForTheScanRebuild,
+    /// 一个当前版本、没被墓碑压住的数据单元落在它那条 inode 记录的 size 罩的单元数之外（文件变短时上一版多出来的那几个）：
+    /// 它既没有更新的版本、也没有墓碑压住，按 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P4 与 I-1.9（清扫水位内无可读垃圾）
+    /// 是「没被压住的可读垃圾」，而 I-1.9 未实现、变短时写不写墓碑没有条款：认不认领它，第一版不定。
+    DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause,
+    /// 两个不同对象身份的当前版本数据单元落在同一个 (inode, 偏移) 上（树 ID 或对象出生代不同）：第一版只有一棵 extent 树，
+    /// 按哪一个认没有条款。
+    TwoObjectsClaimOneDataUnitPosition,
+    /// 同一个 inode 号出现在两片当前版本的 inode 叶容器里（I-9.4（容器号不超最小 key） 管的那一形）：按哪一条认没有条款。
+    InodeNumberInTwoCurrentContainers,
+}
+
+impl ScanRebuildNotDecided {
+    pub(crate) fn reason(self) -> &'static str {
+        match self {
+            ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided => {
+                "扫描重建：码 1 同一对象身份上两版写序逐字节相同，C113 定案 P3 择写序最大者择不出，码 1 的全序键停在 C464（I-1.8 对码 1 的全序键写窄了）"
+            }
+            ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18 => {
+                "扫描重建：码 3 同一对象身份上两组 (诞生代号, 实例代号, 事务号) 相同，I-1.8 ② 不成立，择不出当前版本"
+            }
+            ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderI18 => {
+                "扫描重建：同一版的副本头里的载荷校验和不同，I-1.8 ① 判损坏、不择"
+            }
+            ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy => {
+                "扫描重建：当前版本那片码 3 容器（墓碑、inode、退役、谱系）没有一份副本解得开"
+            }
+            ScanRebuildNotDecided::RolledBackImageWithoutTheLineageReadSideOfC615 => {
+                "扫描重建：谱系树里有记录，是管理员回退过的镜像；按谱系父链择现行的读侧没有条款（C615），第一版不接"
+            }
+            ScanRebuildNotDecided::RetiredLeafContainersWithoutTheReadSideOfC768 => {
+                "扫描重建：退役树里有记录，叶容器退役过；退役记录的读侧没有条款（C768），第一版不接"
+            }
+            ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion => {
+                "扫描重建：墓碑区间不是整对象 [0, 2⁶⁴ − 1]，第一版只写整对象（D18 已定项 24 第 2 条），区间墓碑怎么压 key 没有条款"
+            }
+            ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild => {
+                "扫描重建：墓碑压着的对象还有 inode 记录（删除中），墓碑判 key 死亡（D11 已定项 4）而遍历方向照样读得到没删完的数据单元，中间态上认不认领条款没写"
+            }
+            ScanRebuildNotDecided::DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause => {
+                "扫描重建：一个当前版本、没被墓碑压住的数据单元落在它那条 inode 记录的 size 之外（文件变短留下的），既没有更新的版本也没有墓碑压住（C113 定案 P4、I-1.9 未实现），认不认领第一版不定"
+            }
+            ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition => {
+                "扫描重建：两个不同对象身份的当前版本数据单元落在同一个 (inode, 偏移) 上，按哪一个认没有条款"
+            }
+            ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers => {
+                "扫描重建：同一个 inode 号出现在两片当前版本的 inode 叶容器里，按哪一条认没有条款"
+            }
+        }
+    }
+}
+
+/// C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 的择新键，按单元类分：码 1 取写序 (实例代号, 事务号) 最大者；
+/// 码 3 取 (诞生代号, 实例代号) 最大者，同一 checkpoint 里写过几次的由事务号定——这就是 I-1.8（归并后版本全序） 码 3 的全序键三元，
+/// 诞生代号打头。只在同一个对象身份的各版之间比：对象身份首字节是类标签，一个身份上的各版必然同类。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
+enum CurrentVersionOrderKey {
+    DataUnitWriteOrder {
+        instance: u32,
+        transaction: u64,
+    },
+    PackedRecordUnit {
+        birth_txg: u64,
+        instance: u32,
+        transaction: u64,
+    },
+}
+
+fn current_version_order_key_of(
+    class: ContentUnitClass,
+    plain_header: &[u8],
+) -> CurrentVersionOrderKey {
+    let offsets = class.header_offsets();
+    let instance = read_u32(plain_header, offsets.write_order);
+    let transaction = read_six_byte_unsigned(
+        plain_header,
+        offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET,
+    );
+    match class {
+        ContentUnitClass::Data => CurrentVersionOrderKey::DataUnitWriteOrder {
+            instance,
+            transaction,
+        },
+        ContentUnitClass::PackedRecord => CurrentVersionOrderKey::PackedRecordUnit {
+            birth_txg: read_u64(plain_header, offsets.birth_txg),
+            instance,
+            transaction,
+        },
+    }
+}
+
+/// 一个对象身份的一版：归并键相同的副本合成一版（I-1.8（归并后版本全序） 的归并：「类身份段全部字段含写序」去掉载荷校验和）。
+struct VersionOfAnObject {
+    class: ContentUnitClass,
+    /// 头里的载荷校验和：同一版的副本必须相同（I-1.8 ①）。
+    payload_checksum: u32,
+    /// 第一份副本的明文头（同一版的副本在归并键上逐字节相同，载荷校验和也相同，取哪一份都一样）。
+    plain_header: Vec<u8>,
+    placements: BTreeSet<UnitPlacement>,
+}
+
+/// 每个对象身份择出来的当前版本。
+pub(crate) struct CurrentVersions {
+    of_each_object: BTreeMap<ObjectIdentity, VersionOfAnObject>,
+}
+
+/// 记录要读的那几类码 3 容器：墓碑判 key 死亡、inode 记录是第三张表也是数据单元认不认领的依据、谱系与退役有记录时这一版不接。
+fn packed_record_type_whose_records_are_read(record_type: u16) -> Option<usize> {
+    let record_bytes = match record_type {
+        PACKED_TYPE_TOMBSTONE => TOMBSTONE_RECORD_BYTES,
+        PACKED_TYPE_INODE => INODE_RECORD_BYTES,
+        PACKED_TYPE_RETIREMENT => RETIREMENT_RECORD_BYTES,
+        PACKED_TYPE_LINEAGE => LINEAGE_RECORD_BYTES,
+        _instance_table_or_unregistered => return None,
+    };
+    Some(usize::try_from(record_bytes).expect("记录宽装得进 usize"))
+}
+
+fn packed_record_type_of(plain_header: &[u8]) -> u16 {
+    let offset = ContentUnitClass::PackedRecord
+        .header_offsets()
+        .packed_record_type
+        .expect("码 3 的头偏移表里有打包记录类型那一段");
+    read_u16(plain_header, offset)
+}
+
+impl CurrentVersions {
+    /// 要读记录的那几片当前版本码 3 容器（墓碑 1、inode 2、退役 3、谱系 5），连同扫到它的副本的落点：调用方把每一份的整单元字节读来，
+    /// 按对象身份交给 [`rebuild_index_tables`]。实例表的片不读（它不进映射，已发布谓词由调用方读最新根那一份）。
+    pub(crate) fn packed_containers_whose_records_are_read(
+        &self,
+    ) -> Vec<(ObjectIdentity, BTreeSet<UnitPlacement>)> {
+        self.of_each_object
+            .iter()
+            .filter(|(_, version)| {
+                version.class == ContentUnitClass::PackedRecord
+                    && packed_record_type_whose_records_are_read(packed_record_type_of(
+                        &version.plain_header,
+                    ))
+                    .is_some()
+            })
+            .map(|(identity, version)| (identity.clone(), version.placements.clone()))
+            .collect()
+    }
+}
+
+/// 从扫出的已发布副本择出每个对象身份的当前版本：同对象身份、同归并键的副本合成一版，各版之间按 [`CurrentVersionOrderKey`] 取最大。
+///
+/// 已发布谓词的输入（挂载根的实例代号与 txg、它指着的那一版实例表的行）是扫描重建允许读根的唯一一处：调用方读最新根那一份、
+/// 照 I-1.8（归并后版本全序） 那一遍同一来源滤好再交进来；这里不读根环、不读树表、不读任何树节点（码 2）。
+///
+/// # Errors
+/// 择不出或择了也不作数时交回 [`ScanRebuildNotDecided`]：码 1 两版写序相同（C464）、码 3 两组全序键相同、同一版副本载荷校验和不同。
+///
+/// # Panics
+/// 副本的类标签不是 1 / 3、或明文头短于这一类的明文头末尾：调用方（`walk` 的扫描方向）只交它按类标签与头校验和判过的副本，
+/// 到这里还不对就是那一步被跳过了。
+pub(crate) fn choose_current_versions(
+    copies: &[PublishedUnitCopy],
+) -> Result<CurrentVersions, ScanRebuildNotDecided> {
+    let mut versions_of_each_object: BTreeMap<
+        ObjectIdentity,
+        BTreeMap<Vec<u8>, VersionOfAnObject>,
+    > = BTreeMap::new();
+    for copy in copies {
+        let class = ContentUnitClass::of(copy.plain_header[6])
+            .expect("调用方只交码 1 / 码 3 的副本：扫描方向按类标签滤过");
+        let offsets = class.header_offsets();
+        assert!(
+            copy.plain_header.len() >= offsets.plain_header_end,
+            "调用方交的明文头到这一类的明文头末尾：扫描方向判头校验和时读过这一段"
+        );
+        let merge_key = [
+            &copy.plain_header[offsets.merge_key_before_payload_checksum.clone()],
+            &copy.plain_header[offsets.merge_key_after_payload_checksum.clone()],
+        ]
+        .concat();
+        let payload_checksum = read_u32(&copy.plain_header, offsets.payload_checksum);
+        let version = versions_of_each_object
+            .entry(ObjectIdentity(
+                copy.plain_header[offsets.object_key.clone()].to_vec(),
+            ))
+            .or_default()
+            .entry(merge_key)
+            .or_insert_with(|| VersionOfAnObject {
+                class,
+                payload_checksum,
+                plain_header: copy.plain_header[..offsets.plain_header_end].to_vec(),
+                placements: BTreeSet::new(),
+            });
+        if version.payload_checksum != payload_checksum {
+            return Err(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderI18);
+        }
+        version.placements.insert(copy.placement);
+    }
+    let mut of_each_object = BTreeMap::new();
+    for (identity, versions) in versions_of_each_object {
+        let mut ordered: Vec<(CurrentVersionOrderKey, VersionOfAnObject)> = versions
+            .into_values()
+            .map(|version| {
+                (
+                    current_version_order_key_of(version.class, &version.plain_header),
+                    version,
+                )
+            })
+            .collect();
+        ordered.sort_by_key(|(order_key, _)| *order_key);
+        let (current_order_key, current) =
+            ordered.pop().expect("一个对象身份是从至少一份副本建出来的");
+        if ordered
+            .last()
+            .is_some_and(|(order_key, _)| *order_key == current_order_key)
+        {
+            return Err(match current.class {
+                ContentUnitClass::Data => {
+                    ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided
+                }
+                ContentUnitClass::PackedRecord => {
+                    ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18
+                }
+            });
+        }
+        of_each_object.insert(identity, current);
+    }
+    Ok(CurrentVersions { of_each_object })
+}
+
+/// 一个当前版本的映射 key，从它的明文头现算（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 10 的 key 规则：
+/// `crates/singlefs-core/src/records.rs` 的 `mapping_key_for_data` 与 `mapping_key_for_node` 读的就是这几段，这里按字段表另写一份）。
+fn central_mapping_key_of_the_header(
+    class: ContentUnitClass,
+    plain_header: &[u8],
+) -> CentralMappingKey {
+    let offsets = class.header_offsets();
+    let mut key = vec![0u8; usize::try_from(MAPPING_KEY_BYTES).expect("27")];
+    key[MAPPING_KEY_UNIT_CLASS_OFFSET] = plain_header[6];
+    let birth_tree_bytes = MAPPING_KEY_BIRTH_TXG_OFFSET - MAPPING_KEY_BIRTH_TREE_OFFSET;
+    key[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TXG_OFFSET]
+        .copy_from_slice(&plain_header[offsets.birth_tree..offsets.birth_tree + birth_tree_bytes]);
+    let birth_txg_bytes = MAPPING_KEY_INSTANCE_OFFSET - MAPPING_KEY_BIRTH_TXG_OFFSET;
+    key[MAPPING_KEY_BIRTH_TXG_OFFSET..MAPPING_KEY_INSTANCE_OFFSET]
+        .copy_from_slice(&plain_header[offsets.birth_txg..offsets.birth_txg + birth_txg_bytes]);
+    let instance_bytes = MAPPING_KEY_TAIL_OFFSET - MAPPING_KEY_INSTANCE_OFFSET;
+    key[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_TAIL_OFFSET]
+        .copy_from_slice(&plain_header[offsets.write_order..offsets.write_order + instance_bytes]);
+    match class {
+        ContentUnitClass::Data => {
+            let transaction_start = offsets.write_order + WRITE_ORDER_TRANSACTION_OFFSET;
+            let tail_bytes = key.len() - MAPPING_KEY_TAIL_OFFSET;
+            key[MAPPING_KEY_TAIL_OFFSET..]
+                .copy_from_slice(&plain_header[transaction_start..transaction_start + tail_bytes]);
+        }
+        ContentUnitClass::PackedRecord => {
+            let birth_sequence = offsets
+                .birth_sequence
+                .expect("码 3 的头偏移表里有出生序号那一段");
+            let birth_sequence_bytes = MAPPING_KEY_TAIL_PADDING.start - MAPPING_KEY_TAIL_OFFSET;
+            key[MAPPING_KEY_TAIL_OFFSET..MAPPING_KEY_TAIL_PADDING.start].copy_from_slice(
+                &plain_header[birth_sequence..birth_sequence + birth_sequence_bytes],
+            );
+        }
+    }
+    CentralMappingKey(key)
+}
+
+/// 当前版本那片码 3 容器的记录：从调用方读来的副本里取第一份解得开、头与择出的那一版相同、记录类型与记录宽对得上的。
+fn records_of_a_current_container(
+    version: &VersionOfAnObject,
+    copies: &[Vec<u8>],
+    record_type: u16,
+    record_bytes: usize,
+) -> Result<Vec<Vec<u8>>, ScanRebuildNotDecided> {
+    copies
+        .iter()
+        .filter(|bytes| bytes.get(..version.plain_header.len()) == Some(&version.plain_header[..]))
+        .find_map(|bytes| {
+            packed_unit_view(bytes)
+                .ok()
+                .filter(|view| view.record_type == record_type && view.record_width == record_bytes)
+        })
+        .map(|view| view.records)
+        .ok_or(ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy)
+}
+
+/// 从当前版本与那几片容器的整单元字节现算三张表。
+///
+/// 认领的规矩，逐条有出处：数据单元按对象身份择当前版本（C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3）；
+/// 墓碑判 key 死亡（D11（索引节点要不要留消息缓冲区） 已定项 4）——第一版墓碑只写整对象，压住 (对象, 对象出生代) 的全部单元；
+/// 码 3 容器除实例表的片之外都进映射（D19（块指针的结构与宽度预算） 已定项 8 / 已定项 12 只豁免映射树自己、树表与实例表）。
+/// 条款没写的几格交回 [`ScanRebuildNotDecided`]，见各成员。
+///
+/// # Errors
+/// 见 [`ScanRebuildNotDecided`] 从 `CurrentPackedContainerDecodesFromNoCopy` 起的各成员。
+pub(crate) fn rebuild_index_tables(
+    current: &CurrentVersions,
+    container_copies: &BTreeMap<ObjectIdentity, Vec<Vec<u8>>>,
+) -> Result<RebuiltIndexTables, ScanRebuildNotDecided> {
+    let mut inode_records: BTreeMap<u64, Vec<u8>> = BTreeMap::new();
+    let mut tombstoned_objects: BTreeSet<(u64, u64)> = BTreeSet::new();
+    let mut central_mapping: BTreeMap<CentralMappingKey, BTreeSet<UnitPlacement>> = BTreeMap::new();
+    for (identity, version) in &current.of_each_object {
+        if version.class != ContentUnitClass::PackedRecord {
+            continue;
+        }
+        let record_type = packed_record_type_of(&version.plain_header);
+        if record_type != PACKED_TYPE_INSTANCE_TABLE {
+            central_mapping.insert(
+                central_mapping_key_of_the_header(version.class, &version.plain_header),
+                version.placements.clone(),
+            );
+        }
+        let Some(record_bytes) = packed_record_type_whose_records_are_read(record_type) else {
+            continue;
+        };
+        let records = records_of_a_current_container(
+            version,
+            container_copies.get(identity).map_or(&[], Vec::as_slice),
+            record_type,
+            record_bytes,
+        )?;
+        match record_type {
+            PACKED_TYPE_LINEAGE if !records.is_empty() => {
+                return Err(ScanRebuildNotDecided::RolledBackImageWithoutTheLineageReadSideOfC615);
+            }
+            PACKED_TYPE_RETIREMENT if !records.is_empty() => {
+                return Err(ScanRebuildNotDecided::RetiredLeafContainersWithoutTheReadSideOfC768);
+            }
+            PACKED_TYPE_TOMBSTONE => {
+                for record in &records {
+                    if read_u64(record, TOMBSTONE_RECORD_RANGE_START_OFFSET)
+                        != TOMBSTONE_WHOLE_OBJECT_RANGE_START
+                        || read_u64(record, TOMBSTONE_RECORD_RANGE_END_OFFSET)
+                            != TOMBSTONE_WHOLE_OBJECT_RANGE_END
+                    {
+                        return Err(ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion);
+                    }
+                    tombstoned_objects.insert((
+                        read_u64(record, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET),
+                        read_u64(record, TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET),
+                    ));
+                }
+            }
+            PACKED_TYPE_INODE => {
+                for record in records {
+                    if inode_records.insert(read_u64(&record, 0), record).is_some() {
+                        return Err(ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers);
+                    }
+                }
+            }
+            _empty_lineage_or_retirement_container => {}
+        }
+    }
+    let payload_capacity_in_bytes = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET;
+    let mut data_units_of_each_inode: BTreeMap<DataUnitPosition, CentralMappingKey> =
+        BTreeMap::new();
+    for version in current.of_each_object.values() {
+        if version.class != ContentUnitClass::Data {
+            continue;
+        }
+        let header = &version.plain_header;
+        let inode = read_u64(header, DATA_UNIT_OBJECT_OFFSET);
+        let object_birth = read_u64(header, DATA_UNIT_OBJECT_BIRTH_OFFSET);
+        let offset_in_bytes = read_u64(header, DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET);
+        let record_of_this_object = inode_records
+            .get(&inode)
+            .filter(|record| read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET) == object_birth);
+        if tombstoned_objects.contains(&(inode, object_birth)) {
+            if record_of_this_object.is_some() {
+                return Err(ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild);
+            }
+            continue;
+        }
+        if let Some(record) = record_of_this_object {
+            // 一条记录罩几个数据单元：size 为 0 时 1 个、否则 ⌈size ÷ 净荷容量⌉（实现 `crates/singlefs-core/src/write_request_split.rs`
+            // 的 `data_unit_count_of_a_sequential_write` 里 `div_ceil(...).max(1)`，I-4.6（中间态其余不变量成立） 那一行引的同一个式子）。
+            let data_units_the_size_covers = read_u64(record, INODE_RECORD_SIZE_OFFSET)
+                .div_ceil(payload_capacity_in_bytes)
+                .max(1);
+            if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {
+                return Err(
+                    ScanRebuildNotDecided::DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause,
+                );
+            }
+        }
+        let key = central_mapping_key_of_the_header(version.class, header);
+        let position = DataUnitPosition {
+            inode,
+            offset_in_bytes,
+        };
+        if data_units_of_each_inode
+            .insert(position, key.clone())
+            .is_some()
+        {
+            return Err(ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition);
+        }
+        central_mapping.insert(key, version.placements.clone());
+    }
+    Ok(RebuiltIndexTables {
+        central_mapping,
+        data_units_of_each_inode,
+        inode_records,
+    })
+}
+
+#[cfg(test)]
+mod tests {
+    use std::collections::{BTreeMap, BTreeSet};
+
+    use super::{
+        choose_current_versions, rebuild_index_tables, CurrentVersions, DataUnitPosition,
+        ObjectIdentity, PublishedUnitCopy, RebuiltIndexTables, ScanRebuildNotDecided,
+        UnitPlacement,
+    };
+    use crate::{
+        crc32_castagnoli_bitwise, PACKED_TYPE_INODE, PACKED_TYPE_LINEAGE, PACKED_TYPE_RETIREMENT,
+        PACKED_TYPE_TOMBSTONE,
+    };
+
+    /// 这个模块自己的源码：白名单单测读它的 `use` 清单与测试模块之前那一段里的路径。
+    const SOURCE_OF_THE_REBUILD_MODULE: &str = include_str!("rebuild.rs");
+
+    /// 扫描重建许用的每一项（C410（checker 两个方向共享什么没有检查） 的会红形态：两个方向共享的只有格式解析，D13（验证路线） 已定项 12）：
+    /// 标准库的两个容器、`singlefs_format` 的常量、crate 根上解单元头与码 3 记录的函数与偏移。`walk` 模块与它的节点缓存一项都不许有。
+    const ITEMS_THE_REBUILD_MAY_USE: &[&str] = &[
+        "std::collections::BTreeMap",
+        "std::collections::BTreeSet",
+        "singlefs_format::DATA_UNIT_BYTES",
+        "singlefs_format::DATA_UNIT_PAYLOAD_OFFSET",
+        "singlefs_format::INODE_RECORD_BYTES",
+        "singlefs_format::LINEAGE_RECORD_BYTES",
+        "singlefs_format::MAPPING_KEY_BYTES",
+        "singlefs_format::RETIREMENT_RECORD_BYTES",
+        "singlefs_format::TOMBSTONE_RECORD_BYTES",
+        "crate::packed_unit_view",
+        "crate::read_six_byte_unsigned",
+        "crate::read_u16",
+        "crate::read_u32",
+        "crate::read_u64",
+        "crate::ContentUnitClass",
+        "crate::DATA_UNIT_ANCHOR_OFFSET_IN_BYTES_OFFSET",
+        "crate::DATA_UNIT_OBJECT_BIRTH_OFFSET",
+        "crate::DATA_UNIT_OBJECT_OFFSET",
+        "crate::INODE_RECORD_OBJECT_BIRTH_OFFSET",
+        "crate::INODE_RECORD_SIZE_OFFSET",
+        "crate::MAPPING_KEY_BIRTH_TREE_OFFSET",
+        "crate::MAPPING_KEY_BIRTH_TXG_OFFSET",
+        "crate::MAPPING_KEY_INSTANCE_OFFSET",
+        "crate::MAPPING_KEY_TAIL_OFFSET",
+        "crate::MAPPING_KEY_TAIL_PADDING",
+        "crate::MAPPING_KEY_UNIT_CLASS_OFFSET",
+        "crate::PACKED_TYPE_INODE",
+        "crate::PACKED_TYPE_INSTANCE_TABLE",
+        "crate::PACKED_TYPE_LINEAGE",
+        "crate::PACKED_TYPE_RETIREMENT",
+        "crate::PACKED_TYPE_TOMBSTONE",
+        "crate::TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET",
+        "crate::TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET",
+        "crate::TOMBSTONE_RECORD_RANGE_END_OFFSET",
+        "crate::TOMBSTONE_RECORD_RANGE_START_OFFSET",
+        "crate::TOMBSTONE_WHOLE_OBJECT_RANGE_END",
+        "crate::TOMBSTONE_WHOLE_OBJECT_RANGE_START",
+        "crate::WRITE_ORDER_TRANSACTION_OFFSET",
+    ];
+
+    /// 测试模块那一行属性之前的源码：重建模块本身的代码（测试里引了谁不算重建模块调了谁）。
+    fn source_before_the_test_module() -> &'static str {
+        let (before, _) = SOURCE_OF_THE_REBUILD_MODULE
+            .split_once("#[cfg(test)]")
+            .expect("这个文件里有测试模块");
+        before
+    }
+
+    /// 测试模块之前那一段里的每条 `use` 声明展开成一项一个（只认一层花括号；嵌套判红，免得白名单被一种认不出的写法绕过去）。
+    fn items_used_by_the_rebuild_module() -> Vec<String> {
+        let mut items = Vec::new();
+        for statement in source_before_the_test_module().split(';') {
+            let code: String = statement
+                .lines()
+                .filter(|line| !line.trim_start().starts_with("//"))
+                .map(|line| format!("{line}\n"))
+                .collect();
+            let code = code.trim_start();
+            let Some(path) = ["use ", "pub use ", "pub(crate) use "]
+                .iter()
+                .find_map(|keyword| code.strip_prefix(keyword))
+            else {
+                continue;
+            };
+            let path: String = path.split_whitespace().collect();
+            match path.split_once("::{") {
+                Some((prefix, group)) => {
+                    let group = group.strip_suffix('}').expect("花括号收尾");
+                    assert!(
+                        !group.contains('{'),
+                        "use 声明里的花括号嵌套了一层以上（{path}）：白名单单测只认一层，改写成一层"
+                    );
+                    items.extend(
+                        group
+                            .split(',')
+                            .filter(|item| !item.is_empty())
+                            .map(|item| format!("{prefix}::{item}")),
+                    );
+                }
+                None => items.push(path),
+            }
+        }
+        items
+    }
+
+    #[test]
+    fn the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist() {
+        let used: BTreeSet<String> = items_used_by_the_rebuild_module().into_iter().collect();
+        let allowed: BTreeSet<String> = ITEMS_THE_REBUILD_MAY_USE
+            .iter()
+            .map(|item| (*item).to_string())
+            .collect();
+        let outside_the_whitelist: Vec<&String> = used.difference(&allowed).collect();
+        let whitelisted_but_unused: Vec<&String> = allowed.difference(&used).collect();
+        assert!(
+            outside_the_whitelist.is_empty() && whitelisted_but_unused.is_empty(),
+            "扫描重建的 use 清单与白名单逐项不同：白名单之外的 {outside_the_whitelist:?}、白名单里没用上的 {whitelisted_but_unused:?}（两个方向共享的只有格式解析，D13 已定项 12）"
+        );
+    }
+
+    #[test]
+    fn the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path() {
+        let source = source_before_the_test_module();
+        assert!(
+            !source.contains("super::"),
+            "扫描重建的非测试代码里出现了 super 路径：只许经 use 清单引白名单里的项"
+        );
+        let allowed_crate_items: BTreeSet<&str> = ITEMS_THE_REBUILD_MAY_USE
+            .iter()
+            .filter_map(|item| item.strip_prefix("crate::"))
+            .collect();
+        let mut named_outside_the_whitelist = Vec::new();
+        for (index, _) in source.match_indices("crate::") {
+            let item: String = source[index + "crate::".len()..]
+                .chars()
+                .take_while(|character| character.is_alphanumeric() || *character == '_')
+                .collect();
+            // 后面跟花括号的是 use 清单那一组（上一条单测逐项比过）。
+            if !item.is_empty() && !allowed_crate_items.contains(item.as_str()) {
+                named_outside_the_whitelist.push(item);
+            }
+        }
+        assert!(
+            named_outside_the_whitelist.is_empty(),
+            "扫描重建的非测试代码按路径指名了白名单之外的 crate 内一项 {named_outside_the_whitelist:?}：不许调走树的函数（walk 模块与它的节点缓存）"
+        );
+    }
+
+    const EXTENT_TREE: u64 = 65536;
+    const INODE_TREE: u64 = 65537;
+    const TOMBSTONE_TREE: u64 = 65545;
+    const LINEAGE_TREE: u64 = 65546;
+    const RETIREMENT_TREE: u64 = 65548;
+    const FIRST_INODE: u64 = 1;
+    const OBJECT_BIRTH_OF_THE_FIRST_INODE: u64 = 3;
+    /// 数据单元净荷容量（32768 − 134）：第二个单元的锚点偏移。
+    const SECOND_DATA_UNIT_OFFSET_IN_BYTES: u64 = 32768 - 134;
+    const INODE_RECORD_WIDTH: usize = 140;
+    const TOMBSTONE_RECORD_WIDTH: usize = 40;
+    const LINEAGE_RECORD_WIDTH: usize = 24;
+    const RETIREMENT_RECORD_WIDTH: usize = 34;
+
+    fn placement_on_device_zero(slot: u64) -> UnitPlacement {
+        UnitPlacement { device: 0, slot }
+    }
+
+    /// 码 1 数据单元一个版本的头里这一组单测要的几段。
+    struct DataUnitVersion {
+        tree: u64,
+        offset_in_bytes: u64,
+        birth_txg: u64,
+        instance: u32,
+        transaction: u64,
+    }
+
+    impl DataUnitVersion {
+        fn of_the_first_inode_at(offset_in_bytes: u64) -> Self {
+            Self {
+                tree: EXTENT_TREE,
+                offset_in_bytes,
+                birth_txg: 3,
+                instance: 1,
+                transaction: 1,
+            }
+        }
+    }
+
+    /// 一份码 1 副本的明文头：五元组 (1, 树, 第一个 inode, 它的对象出生代, 锚点偏移)、诞生代号与写序，其余全零
+    /// （重建不判头校验和，那一步归扫描方向）。
+    fn data_unit_copy(placement: UnitPlacement, version: &DataUnitVersion) -> PublishedUnitCopy {
+        let mut plain_header = vec![0u8; 105];
+        plain_header[..4].copy_from_slice(b"SFSU");
+        plain_header[6] = 1;
+        plain_header[42] = 1;
+        plain_header[43..51].copy_from_slice(&version.tree.to_le_bytes());
+        plain_header[51..59].copy_from_slice(&FIRST_INODE.to_le_bytes());
+        plain_header[59..67].copy_from_slice(&OBJECT_BIRTH_OF_THE_FIRST_INODE.to_le_bytes());
+        plain_header[67..75].copy_from_slice(&version.offset_in_bytes.to_le_bytes());
+        plain_header[75..83].copy_from_slice(&version.birth_txg.to_le_bytes());
+        plain_header[91..95].copy_from_slice(&version.instance.to_le_bytes());
+        plain_header[95..101].copy_from_slice(&version.transaction.to_le_bytes()[..6]);
+        PublishedUnitCopy {
+            placement,
+            plain_header,
+        }
+    }
+
+    /// 一片码 3 容器的类身份段与记录（这一组单测要的几段）。
+    struct PackedContainer<'records> {
+        birth_tree: u64,
+        record_type: u16,
+        container: u64,
+        record_width: usize,
+        records: &'records [Vec<u8>],
+        birth_txg: u64,
+    }
+
+    /// 一片码 3 容器的整单元 32768 字节，两道校验和都封上（`check_unit` 要它们对得上）：类身份段 (3, 出生树, 类型, 容器号, 容器出生代 3)、
+    /// 诞生代号、写序实例 1，记录从 136 起。
+    fn sealed_packed_container(container: &PackedContainer<'_>) -> Vec<u8> {
+        let mut unit = vec![0u8; 32768];
+        unit[..4].copy_from_slice(b"SFSU");
+        unit[4..6].copy_from_slice(&1u16.to_le_bytes());
+        unit[6] = 3;
+        let declared_length =
+            u16::try_from(container.records.len() * container.record_width).expect("装得进一片");
+        unit[8..10].copy_from_slice(&declared_length.to_le_bytes());
+        unit[42] = 3;
+        unit[43..51].copy_from_slice(&container.birth_tree.to_le_bytes());
+        unit[51..53].copy_from_slice(&container.record_type.to_le_bytes());
+        unit[53..61].copy_from_slice(&container.container.to_le_bytes());
+        unit[61..69].copy_from_slice(&3u64.to_le_bytes());
+        unit[69..71].copy_from_slice(
+            &u16::try_from(container.records.len())
+                .expect("条数")
+                .to_le_bytes(),
+        );
+        unit[71..73].copy_from_slice(
+            &u16::try_from(container.record_width)
+                .expect("记录宽")
+                .to_le_bytes(),
+        );
+        unit[73..81].copy_from_slice(&container.birth_txg.to_le_bytes());
+        unit[93..97].copy_from_slice(&1u32.to_le_bytes());
+        for (index, record) in container.records.iter().enumerate() {
+            assert_eq!(record.len(), container.record_width, "记录宽");
+            let start = 136 + index * container.record_width;
+            unit[start..start + container.record_width].copy_from_slice(record);
+        }
+        let payload_crc = crc32_castagnoli_bitwise(&unit[107..]);
+        unit[89..93].copy_from_slice(&payload_crc.to_le_bytes());
+        let header_checksum = crc32_castagnoli_bitwise(&unit[..107]);
+        unit[10..14].copy_from_slice(&header_checksum.to_le_bytes());
+        unit
+    }
+
+    fn copy_of_the_container(placement: UnitPlacement, unit: &[u8]) -> PublishedUnitCopy {
+        PublishedUnitCopy {
+            placement,
+            plain_header: unit[..107].to_vec(),
+        }
+    }
+
+    fn inode_record(inode: u64, object_birth: u64, size_in_bytes: u64) -> Vec<u8> {
+        let mut record = vec![0u8; INODE_RECORD_WIDTH];
+        record[..8].copy_from_slice(&inode.to_le_bytes());
+        record[8..16].copy_from_slice(&object_birth.to_le_bytes());
+        record[36..40].copy_from_slice(&1u32.to_le_bytes());
+        record[40..48].copy_from_slice(&size_in_bytes.to_le_bytes());
+        record
+    }
+
+    fn tombstone_record(inode: u64, object_birth: u64, range_end_in_bytes: u64) -> Vec<u8> {
+        let mut record = vec![0u8; TOMBSTONE_RECORD_WIDTH];
+        record[..8].copy_from_slice(&inode.to_le_bytes());
+        record[8..16].copy_from_slice(&object_birth.to_le_bytes());
+        record[24..32].copy_from_slice(&range_end_in_bytes.to_le_bytes());
+        record[32..40].copy_from_slice(&5u64.to_le_bytes());
+        record
+    }
+
+    fn inode_container_holding(container: u64, records: &[Vec<u8>]) -> Vec<u8> {
+        sealed_packed_container(&PackedContainer {
+            birth_tree: INODE_TREE,
+            record_type: PACKED_TYPE_INODE,
+            container,
+            record_width: INODE_RECORD_WIDTH,
+            records,
+            birth_txg: 3,
+        })
+    }
+
+    fn tombstone_container_holding(records: &[Vec<u8>]) -> Vec<u8> {
+        sealed_packed_container(&PackedContainer {
+            birth_tree: TOMBSTONE_TREE,
+            record_type: PACKED_TYPE_TOMBSTONE,
+            container: 1,
+            record_width: TOMBSTONE_RECORD_WIDTH,
+            records,
+            birth_txg: 5,
+        })
+    }
+
+    /// 扫到的副本连同那几片容器的整单元字节（调用方读回来交给第二步的那一份）：择当前版本、读容器、现算三张表。
+    fn rebuilt_from(
+        copies: &[PublishedUnitCopy],
+        containers: &[(UnitPlacement, Vec<u8>)],
+    ) -> Result<RebuiltIndexTables, ScanRebuildNotDecided> {
+        let current: CurrentVersions = choose_current_versions(copies)?;
+        let mut container_copies: BTreeMap<ObjectIdentity, Vec<Vec<u8>>> = BTreeMap::new();
+        for (identity, placements) in current.packed_containers_whose_records_are_read() {
+            let bytes: Vec<Vec<u8>> = containers
+                .iter()
+                .filter(|(placement, _)| placements.contains(placement))
+                .map(|(_, unit)| unit.clone())
+                .collect();
+            container_copies.insert(identity, bytes);
+        }
+        rebuild_index_tables(&current, &container_copies)
+    }
+
+    #[test]
+    fn the_current_version_of_a_data_unit_is_the_one_with_the_largest_write_order() {
+        let later_placement = placement_on_device_zero(50254);
+        let copies = [
+            data_unit_copy(
+                later_placement,
+                &DataUnitVersion {
+                    birth_txg: 4,
+                    instance: 2,
+                    transaction: 1,
+                    ..DataUnitVersion::of_the_first_inode_at(0)
+                },
+            ),
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion {
+                    birth_txg: 5,
+                    instance: 1,
+                    transaction: 9,
+                    ..DataUnitVersion::of_the_first_inode_at(0)
+                },
+            ),
+        ];
+        let tables =
+            rebuilt_from(&copies, &[]).expect("写序互不相同，择得出；没有墓碑也没有记录，照认领");
+        let placements: Vec<&BTreeSet<UnitPlacement>> = tables.central_mapping.values().collect();
+        assert_eq!(
+            placements,
+            vec![&BTreeSet::from([later_placement])],
+            "写序 (实例 2, 事务 1) 大于 (实例 1, 事务 9)：当前版本是实例 2 那一版（C113 定案 P3 码 1 取写序最大者），与诞生代号无关"
+        );
+    }
+
+    #[test]
+    fn two_data_unit_versions_with_the_same_write_order_are_not_decided_until_c464() {
+        let copies = [
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion {
+                    birth_txg: 4,
+                    transaction: 7,
+                    ..DataUnitVersion::of_the_first_inode_at(0)
+                },
+            ),
+            data_unit_copy(
+                placement_on_device_zero(50254),
+                &DataUnitVersion {
+                    birth_txg: 5,
+                    transaction: 7,
+                    ..DataUnitVersion::of_the_first_inode_at(0)
+                },
+            ),
+        ];
+        assert_eq!(
+            choose_current_versions(&copies).err(),
+            Some(ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided),
+            "两版写序逐字节相同、诞生代号不同：P3 择不出，码 1 的键停在 C464，交回没定、不猜"
+        );
+    }
+
+    #[test]
+    fn copies_of_one_version_with_different_payload_checksums_are_not_decided() {
+        let version = DataUnitVersion::of_the_first_inode_at(0);
+        let mut second_copy = data_unit_copy(placement_on_device_zero(50181), &version);
+        second_copy.plain_header[101] ^= 0xFF;
+        let copies = [
+            data_unit_copy(placement_on_device_zero(50180), &version),
+            second_copy,
+        ];
+        assert_eq!(
+            choose_current_versions(&copies).err(),
+            Some(ScanRebuildNotDecided::CopiesOfOneVersionCarryDifferentPayloadsUnderI18),
+            "归并键相同而头里的载荷校验和不同：I-1.8 ① 判损坏、不择"
+        );
+    }
+
+    #[test]
+    fn two_packed_versions_with_the_same_total_order_key_are_not_decided() {
+        let first = inode_container_holding(1, &[inode_record(FIRST_INODE, 3, 10)]);
+        let second = inode_container_holding(
+            1,
+            &[
+                inode_record(FIRST_INODE, 3, 10),
+                inode_record(FIRST_INODE + 1, 3, 10),
+            ],
+        );
+        let copies = [
+            copy_of_the_container(placement_on_device_zero(50242), &first),
+            copy_of_the_container(placement_on_device_zero(50254), &second),
+        ];
+        assert_eq!(
+            choose_current_versions(&copies).err(),
+            Some(ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18),
+            "同一片容器两版 (诞生代号, 实例代号, 事务号) 相同、记录数不同（归并键不同，是两组）：I-1.8 ② 不成立，择不出"
+        );
+    }
+
+    #[test]
+    fn the_inode_records_table_is_read_from_the_current_inode_container() {
+        let record = inode_record(FIRST_INODE, OBJECT_BIRTH_OF_THE_FIRST_INODE, 3000);
+        let container = inode_container_holding(1, std::slice::from_ref(&record));
+        let container_placement = placement_on_device_zero(50242);
+        let copies = [
+            copy_of_the_container(container_placement, &container),
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion::of_the_first_inode_at(0),
+            ),
+        ];
+        let tables = rebuilt_from(&copies, &[(container_placement, container)])
+            .expect("一个文件、一片 inode 叶容器：每一格都有条款");
+        assert_eq!(
+            tables.inode_records,
+            BTreeMap::from([(FIRST_INODE, record)]),
+            "inode 记录集合是当前版本那片叶容器里的记录原样"
+        );
+        assert_eq!(
+            tables
+                .data_units_of_each_inode
+                .keys()
+                .copied()
+                .collect::<Vec<_>>(),
+            vec![DataUnitPosition {
+                inode: FIRST_INODE,
+                offset_in_bytes: 0
+            }],
+            "size 3000 罩一个数据单元：偏移 0 那一个认领"
+        );
+        assert_eq!(
+            tables.central_mapping.len(),
+            2,
+            "中央映射：数据单元一条、inode 叶容器一条"
+        );
+    }
+
+    #[test]
+    fn a_whole_object_tombstone_kills_the_data_units_of_an_object_whose_record_is_gone() {
+        let tombstone_container = tombstone_container_holding(&[tombstone_record(
+            FIRST_INODE,
+            OBJECT_BIRTH_OF_THE_FIRST_INODE,
+            u64::MAX,
+        )]);
+        let tombstone_placement = placement_on_device_zero(50300);
+        let copies = [
+            copy_of_the_container(tombstone_placement, &tombstone_container),
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion::of_the_first_inode_at(0),
+            ),
+        ];
+        let tables = rebuilt_from(&copies, &[(tombstone_placement, tombstone_container)])
+            .expect("记录已删、墓碑压着：数据单元判死、不认领");
+        assert!(
+            tables.data_units_of_each_inode.is_empty(),
+            "墓碑判 key 死亡（D11 已定项 4）：被删对象的数据单元不进数据单元集合，实际 {:?}",
+            tables.data_units_of_each_inode
+        );
+        assert_eq!(tables.central_mapping.len(), 1, "中央映射只剩那片墓碑容器");
+    }
+
+    #[test]
+    fn a_tombstoned_object_that_still_has_its_record_is_a_deletion_in_progress_and_not_decided() {
+        let tombstone_container = tombstone_container_holding(&[tombstone_record(
+            FIRST_INODE,
+            OBJECT_BIRTH_OF_THE_FIRST_INODE,
+            u64::MAX,
+        )]);
+        let inode_container = inode_container_holding(
+            1,
+            &[inode_record(
+                FIRST_INODE,
+                OBJECT_BIRTH_OF_THE_FIRST_INODE,
+                3000,
+            )],
+        );
+        let tombstone_placement = placement_on_device_zero(50300);
+        let inode_placement = placement_on_device_zero(50242);
+        let copies = [
+            copy_of_the_container(tombstone_placement, &tombstone_container),
+            copy_of_the_container(inode_placement, &inode_container),
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion::of_the_first_inode_at(0),
+            ),
+        ];
+        assert_eq!(
+            rebuilt_from(
+                &copies,
+                &[
+                    (tombstone_placement, tombstone_container),
+                    (inode_placement, inode_container)
+                ]
+            )
+            .err(),
+            Some(ScanRebuildNotDecided::DeletionInProgressNotWrittenForTheScanRebuild),
+            "意图写了墓碑、记录还在（删除中）：中间态上认不认领没删完的数据单元条款没写，交回没定"
+        );
+    }
+
+    #[test]
+    fn a_data_unit_beyond_the_size_of_its_record_is_not_decided() {
+        let inode_container = inode_container_holding(
+            1,
+            &[inode_record(
+                FIRST_INODE,
+                OBJECT_BIRTH_OF_THE_FIRST_INODE,
+                3000,
+            )],
+        );
+        let inode_placement = placement_on_device_zero(50242);
+        let copies = [
+            copy_of_the_container(inode_placement, &inode_container),
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion::of_the_first_inode_at(0),
+            ),
+            data_unit_copy(
+                placement_on_device_zero(50182),
+                &DataUnitVersion::of_the_first_inode_at(SECOND_DATA_UNIT_OFFSET_IN_BYTES),
+            ),
+        ];
+        assert_eq!(
+            rebuilt_from(&copies, &[(inode_placement, inode_container)]).err(),
+            Some(ScanRebuildNotDecided::DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause),
+            "size 3000 只罩一个单元，第二个单元既没有更新的版本也没有墓碑压住：认不认领第一版不定"
+        );
+    }
+
+    #[test]
+    fn a_lineage_record_marks_a_rolled_back_image_that_is_not_decided() {
+        let lineage_container = sealed_packed_container(&PackedContainer {
+            birth_tree: LINEAGE_TREE,
+            record_type: PACKED_TYPE_LINEAGE,
+            container: 1,
+            record_width: LINEAGE_RECORD_WIDTH,
+            records: &[vec![7u8; LINEAGE_RECORD_WIDTH]],
+            birth_txg: 6,
+        });
+        let placement = placement_on_device_zero(50310);
+        let copies = [copy_of_the_container(placement, &lineage_container)];
+        assert_eq!(
+            rebuilt_from(&copies, &[(placement, lineage_container)]).err(),
+            Some(ScanRebuildNotDecided::RolledBackImageWithoutTheLineageReadSideOfC615),
+            "谱系树里有记录：回退过，谱系父链的读侧第一版不接（C615）"
+        );
+    }
+
+    #[test]
+    fn a_retirement_record_marks_retired_leaf_containers_that_are_not_decided() {
+        let retirement_container = sealed_packed_container(&PackedContainer {
+            birth_tree: RETIREMENT_TREE,
+            record_type: PACKED_TYPE_RETIREMENT,
+            container: 1,
+            record_width: RETIREMENT_RECORD_WIDTH,
+            records: &[vec![7u8; RETIREMENT_RECORD_WIDTH]],
+            birth_txg: 6,
+        });
+        let placement = placement_on_device_zero(50320);
+        let copies = [copy_of_the_container(placement, &retirement_container)];
+        assert_eq!(
+            rebuilt_from(&copies, &[(placement, retirement_container)]).err(),
+            Some(ScanRebuildNotDecided::RetiredLeafContainersWithoutTheReadSideOfC768),
+            "退役树里有记录：退役记录的读侧第一版不接（C768）"
+        );
+    }
+
+    #[test]
+    fn a_tombstone_whose_range_is_not_the_whole_object_is_not_decided() {
+        let tombstone_container = tombstone_container_holding(&[tombstone_record(
+            FIRST_INODE,
+            OBJECT_BIRTH_OF_THE_FIRST_INODE,
+            4096,
+        )]);
+        let placement = placement_on_device_zero(50300);
+        let copies = [copy_of_the_container(placement, &tombstone_container)];
+        assert_eq!(
+            rebuilt_from(&copies, &[(placement, tombstone_container)]).err(),
+            Some(ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion),
+            "墓碑区间止 4096、不是 2⁶⁴ − 1：第一版只写整对象，区间墓碑怎么压 key 没有条款"
+        );
+    }
+
+    #[test]
+    fn a_current_container_without_a_decodable_copy_is_not_decided() {
+        let inode_container = inode_container_holding(1, &[inode_record(FIRST_INODE, 3, 3000)]);
+        let mut torn_copy = inode_container.clone();
+        torn_copy[200] ^= 0xFF;
+        let placement = placement_on_device_zero(50242);
+        let copies = [copy_of_the_container(placement, &inode_container)];
+        assert_eq!(
+            rebuilt_from(&copies, &[(placement, torn_copy)]).err(),
+            Some(ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy),
+            "当前版本那片 inode 叶容器唯一一份载荷 CRC 对不上：解不开，交回没定"
+        );
+    }
+
+    #[test]
+    fn two_objects_at_one_data_unit_position_are_not_decided() {
+        let copies = [
+            data_unit_copy(
+                placement_on_device_zero(50180),
+                &DataUnitVersion::of_the_first_inode_at(0),
+            ),
+            data_unit_copy(
+                placement_on_device_zero(50254),
+                &DataUnitVersion {
+                    tree: EXTENT_TREE + 13,
+                    ..DataUnitVersion::of_the_first_inode_at(0)
+                },
+            ),
+        ];
+        assert_eq!(
+            rebuilt_from(&copies, &[]).err(),
+            Some(ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition),
+            "两棵树的数据单元落在同一个 (inode, 偏移)：第一版只有一棵 extent 树，按哪一个认没有条款"
+        );
+    }
+
+    #[test]
+    fn one_inode_number_in_two_current_containers_is_not_decided() {
+        let record = inode_record(FIRST_INODE, 3, 3000);
+        let first = inode_container_holding(1, std::slice::from_ref(&record));
+        let second = inode_container_holding(2, &[record]);
+        let first_placement = placement_on_device_zero(50242);
+        let second_placement = placement_on_device_zero(50254);
+        let copies = [
+            copy_of_the_container(first_placement, &first),
+            copy_of_the_container(second_placement, &second),
+        ];
+        assert_eq!(
+            rebuilt_from(
+                &copies,
+                &[(first_placement, first), (second_placement, second)]
+            )
+            .err(),
+            Some(ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers),
+            "容器 1 与容器 2 里都有 inode 1：按哪一条认没有条款"
+        );
+    }
+}
diff --git a/crates/singlefs-checker/src/walk.rs b/crates/singlefs-checker/src/walk.rs
--- a/crates/singlefs-checker/src/walk.rs
+++ b/crates/singlefs-checker/src/walk.rs
@@ -11,9 +11,14 @@
     CLUSTER_SEGMENT_SLOTS, DATA_UNIT_BYTES, EXTENT_LEAF_RECORD_BYTES,
     EXTENT_TREE_INTERNAL_ENTRY_BYTES, EXTENT_TREE_UPPER_LEAF_ENTRY_BYTES, INODE_INTERNAL_ENTRY,
     JOURNAL_RECORD_BYTES, JOURNAL_SAFETY_FACTOR, LINEAGE_CONTAINER_RECORDS, MAPPING_ENTRY_BYTES,
-    NODE_BYTES, NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
+    MAPPING_KEY_BYTES, NODE_BYTES, NODE_POINTER_BYTES, NONCE_MAC_ALGORITHM_RESERVED_BYTES,
     RETIREMENT_CONTAINER_RECORDS, SLOT_BYTES, TOMBSTONE_CONTAINER_RECORDS,
     TREE_IDENTIFIER_WATERMARK_AT_MKFS, TREE_TABLE_ENTRY_BYTES,
+};
+
+use crate::rebuild::{
+    choose_current_versions, rebuild_index_tables, CentralMappingKey, DataUnitPosition,
+    ObjectIdentity, PublishedUnitCopy, UnitPlacement,
 };
 
 use crate::position_addressed::{
@@ -65,7 +70,7 @@
     RETIREMENT_RECORDS_ACCOUNT_FOR_EVERY_LEAF_GONE_SINCE_THE_PARENT_VERSION,
     RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
     RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
-    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER,
+    RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     TOMBSTONES_ON_THE_LINEAGE_ARE_DISTINCT_AND_EVERY_TOMBSTONE_IS_WELL_FORMED,
     TOMBSTONE_CONTAINERS_STAY_WITHIN_THE_CAPACITY_KNOB_ON_A_TEST_IMAGE,
@@ -78,9 +83,17 @@
     back_chain_of_record_header, check_index_node_keys, check_internal_node_separators,
     check_journal_record, check_unit, checksum_field_holds, crc32_castagnoli_table,
     index_node_view, key_schema_for_tree_kind, packed_unit_view, read_six_byte_unsigned, read_u16,
-    read_u32, read_u64, NamedEntryView, Verdict, FORMAT_VERSION_OFFSET, KEY_SCHEMA_ACCOUNTING,
+    read_u32, read_u64, ContentUnitClass, NamedEntryView, Verdict, FORMAT_VERSION_OFFSET,
+    INODE_RECORD_OBJECT_BIRTH_OFFSET, INODE_RECORD_SIZE_OFFSET, KEY_SCHEMA_ACCOUNTING,
     KEY_SCHEMA_ALLOCATION, KEY_SCHEMA_CONTAINER_NUMBER, KEY_SCHEMA_EXTENT, KEY_SCHEMA_INODE,
-    KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE, UNIT_FORMAT_VERSION_THIS_CHECKER_READS,
+    KEY_SCHEMA_MAPPING, KEY_SCHEMA_TREE_TABLE, MAPPING_KEY_BIRTH_TREE_OFFSET,
+    MAPPING_KEY_BIRTH_TXG_OFFSET, MAPPING_KEY_INSTANCE_OFFSET, MAPPING_KEY_TAIL_OFFSET,
+    MAPPING_KEY_TAIL_PADDING, MAPPING_KEY_UNIT_CLASS_OFFSET, PACKED_TYPE_INODE,
+    PACKED_TYPE_INSTANCE_TABLE, PACKED_TYPE_LINEAGE, PACKED_TYPE_RETIREMENT, PACKED_TYPE_TOMBSTONE,
+    TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET, TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET,
+    TOMBSTONE_RECORD_RANGE_END_OFFSET, TOMBSTONE_RECORD_RANGE_START_OFFSET,
+    TOMBSTONE_WHOLE_OBJECT_RANGE_END, TOMBSTONE_WHOLE_OBJECT_RANGE_START,
+    UNIT_FORMAT_VERSION_THIS_CHECKER_READS, WRITE_ORDER_TRANSACTION_OFFSET,
 };
 
 const TREE_KIND_EXTENT: u16 = 1;
@@ -116,15 +129,8 @@
     TREE_KIND_LOGGED_OPERATIONS,
     TREE_KIND_RETIREMENT,
 ];
-const PACKED_TYPE_TOMBSTONE: u16 = 1;
-/// 退役记录的打包记录类型（D18（块里携带什么信息） 已定项 11 登记的 3；记录宽 34，判决 m3-retire-r1 第三节第 2 条）。
-const PACKED_TYPE_RETIREMENT: u16 = 3;
-const PACKED_TYPE_INODE: u16 = 2;
-/// 谱系记录的打包记录类型（D18（块里携带什么信息） 已定项 24 第 5 条，不复用登记过的 3）。
-const PACKED_TYPE_LINEAGE: u16 = 5;
 /// `deleted_inodes` 根的条目：key = inode 号 8 字节、无 value（D8（核心索引结构） 已定项 16）。
 const DELETED_INODES_ENTRY_BYTES: usize = 8;
-const PACKED_TYPE_INSTANCE_TABLE: u16 = 4;
 /// inode 树内部条目的类型段 0：子单元是码 2 的 inode 树内部节点（I-9.2（条目身份与子头相符）「类型段 ∈ {0, 2}」）。
 const INODE_ENTRY_TYPE_INTERNAL_NODE: u16 = 0;
 
@@ -230,14 +236,9 @@
 /// inode 记录里 nlink 的偏移（D8（核心索引结构） 已定项 6 字段表「24 mode / uid / gid / nlink，各 4」，4 字节；
 /// `.claude/kb/layout/03-third-txn.md`「inode 记录的 nlink 第一次为 0」那一行）。
 const INODE_RECORD_NLINK_OFFSET: usize = 36;
-/// 墓碑记录里对象 ID 的偏移（`.claude/kb/layout/03-third-txn.md`「二、墓碑记录」：偏移 0、8 字节，被删的 inode 号）。
-const TOMBSTONE_RECORD_OBJECT_IDENTIFIER_OFFSET: usize = 0;
 /// 码 3 单元头里写序的实例代号（D18（块里携带什么信息） 已定项 11 的类身份段表：写序 93 起，实例代号 4 字节在前）。
 const PACKED_UNIT_WRITE_ORDER_INSTANCE_OFFSET: usize = 93;
-/// inode 记录 140 里 `size` 与 `blocks` 两个字段的偏移（D8（核心索引结构） 已定项 6 的偏移表），按字段表另写一份、不用实现的解析。
-const INODE_RECORD_SIZE_OFFSET: usize = 40;
-/// inode 记录里对象出生代（偏移 8）与改动计数（偏移 88）的位置（D8（核心索引结构） 已定项 6 的字段表）：I-9.26 比这两个与容器诞生代号。
-const INODE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
+/// inode 记录里改动计数的位置（偏移 88，D8（核心索引结构） 已定项 6 的字段表）：I-9.26 比它、对象出生代与容器诞生代号。
 const INODE_RECORD_CHANGE_COUNT_OFFSET: usize = 88;
 /// 码 3 单元头里的诞生代号（D18（块里携带什么信息） 已定项 11 的类身份段表：偏移 73）。
 const PACKED_UNIT_BIRTH_TXG_OFFSET: usize = 73;
@@ -964,17 +965,6 @@
         }
     }
 }
-
-/// 中央映射 key 里各段的偏移（D19（块指针的结构与宽度预算） 已定项 6 / 10：类标签 1 + 出生树 8 + 出生 txg 8 + 实例代号 4 + 尾段 6；
-/// 尾段码 1 是事务号 6 字节，码 2 / 码 3 是出生序号 4 + 补零 2）。按字段表另写一份，不用实现的解析。
-const MAPPING_KEY_UNIT_CLASS_OFFSET: usize = 0;
-const MAPPING_KEY_BIRTH_TREE_OFFSET: usize = 1;
-const MAPPING_KEY_BIRTH_TXG_OFFSET: usize = 1 + 8;
-const MAPPING_KEY_INSTANCE_OFFSET: usize = 1 + 8 + 8;
-const MAPPING_KEY_TAIL_OFFSET: usize = 1 + 8 + 8 + 4;
-/// 码 2 / 码 3 的尾段是出生序号 4 字节，其后补零 2 字节（D19（块指针的结构与宽度预算） 已定项 10「末尾补零到 27」）。
-const MAPPING_KEY_TAIL_PADDING: std::ops::Range<usize> =
-    MAPPING_KEY_TAIL_OFFSET + 4..MAPPING_KEY_TAIL_OFFSET + 6;
 
 /// 单元头里与映射 key 的出生树那一段相比的字段（D19（块指针的结构与宽度预算） 已定项 6 的收严句）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
@@ -5119,17 +5109,25 @@
                 extent_tree
                     .data_pointers_of_inodes
                     .into_iter()
-                    .map(|(_, data_pointer)| data_pointer)
+                    .map(|data_pointer_of_an_inode| data_pointer_of_an_inode.pointer)
                     .collect(),
             )
         })
 }
 
 /// [`extent_tree_with_the_inode_of_each_data_pointer_without_judging`] 读出的一棵 extent 树：每个节点的指针（根在内），
-/// 与每个数据指针连同它属于哪个 inode 号。
+/// 与每个数据指针连同它属于哪个 inode 号、落在文件哪个字节偏移。
 struct ExtentTreeReadWithoutJudging {
     node_pointers: Vec<PointerView>,
-    data_pointers_of_inodes: Vec<(u64, PointerView)>,
+    data_pointers_of_inodes: Vec<DataPointerOfAnInode>,
+}
+
+/// extent 树里的一个数据指针，连同它所在 extent key (0, inode, 偏移) 的后两段：上段叶条目内联的那一个偏移是 0
+/// （走读 `walk_extent_data_pointer` 同一个读法），下段叶的 extent 叶记录取记录 key 偏移 16 那 8 字节。
+struct DataPointerOfAnInode {
+    inode: u64,
+    offset_in_bytes: u64,
+    pointer: PointerView,
 }
 
 /// [`extent_tree_without_judging`] 那一遍，每个数据指针带上它属于哪个 inode 号（上段叶条目的 inode 号；下段叶记录属于引它那条上段条目的号）：
@@ -5189,7 +5187,11 @@
                             inode,
                             pointer: inline_data_pointer,
                         } => {
-                            data_pointers.push((inode, parse_data_pointer(&inline_data_pointer)));
+                            data_pointers.push(DataPointerOfAnInode {
+                                inode,
+                                offset_in_bytes: 0,
+                                pointer: parse_data_pointer(&inline_data_pointer),
+                            });
                         }
                         ExtentUpperLeafEntryView::Malformed => return None,
                     }
@@ -5199,11 +5201,11 @@
                 if node.entry_width < extent_leaf_record_bytes() {
                     return None;
                 }
-                data_pointers.extend(
-                    node.entries
-                        .iter()
-                        .map(|record| (inode, parse_data_pointer(&record[24..112]))),
-                );
+                data_pointers.extend(node.entries.iter().map(|record| DataPointerOfAnInode {
+                    inode,
+                    offset_in_bytes: read_u64(record, 16),
+                    pointer: parse_data_pointer(&record[24..112]),
+                }));
             }
             (None, _) => {
                 if node.entry_width < internal_entry_bytes {
@@ -7159,35 +7161,6 @@
 
 /// 单元头校验和字段的偏移（共同前缀，三类同一处）。
 const UNIT_HEADER_CHECKSUM_OFFSET: usize = 10;
-/// 写序那 10 字节里事务号低 48 位从第 4 字节起（D18（块里携带什么信息） 已定项 7：实例代号 4 + 事务号 6）。
-const WRITE_ORDER_TRANSACTION_OFFSET: usize = 4;
-
-/// I-1.8（归并后版本全序） 罩得到的两类单元。码 2 不进这一条（`crates/singlefs-checker/invariants/01-块自描述.md` I-1.8 逐字「码 2 不进这条」：
-/// 它的写序已收窄成只有实例代号，多版在条目级合并、不在容器级择版本）。
-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
-enum ContentUnitClass {
-    /// 码 1 数据单元。
-    Data,
-    /// 码 3 打包记录单元。
-    PackedRecord,
-}
-
-/// 一类内容单元的头里，这一条要读的那几段。码 1 按 D18（块里携带什么信息） 已定项 7 的字段表（五元组 42、诞生代号 75、
-/// fsid 83、写序 91、载荷 CRC 101、明文头末尾 105）；码 3 按已定项 11 的类身份段表（标签 42、出生树 43、类型 51、容器号 53、
-/// 容器出生代 61、记录数 69、记录宽 71、诞生代号 73、fsid 81、载荷校验和 89、写序 93、出生序号 103、明文头末尾 107）。
-struct ContentUnitHeaderOffsets {
-    plain_header_end: usize,
-    payload_checksum: usize,
-    filesystem_identifier: usize,
-    birth_txg: usize,
-    write_order: usize,
-    /// 类身份段里与版本无关的那一段：码 1 的五元组、码 3 的（标签, 出生树, 类型, 容器号, 容器出生代）。
-    object_key: std::ops::Range<usize>,
-    /// 归并键的两段：「类身份段全部字段含写序」**去掉载荷校验和那一段**。
-    /// 载荷校验和不进归并键——它正是「同一组的成员载荷相同」要比的那个量，进了键这一条就恒真。
-    merge_key_before_payload_checksum: std::ops::Range<usize>,
-    merge_key_after_payload_checksum: std::ops::Range<usize>,
-}
 
 /// 已发布谓词的输入，按单元类的两种取法分成两个成员（`crates/singlefs-checker/invariants/01-块自描述.md` I-1.2（块头写序已发布） 那一行逐字：
 /// 「有行 (i, T_pub, W) ⇒ 码 1 要 b ≤ T_pub ∨ n ≤ W、码 2 / 3 要 b ≤ T_pub」）：码 1 那一支带事务号 n，
@@ -7246,46 +7219,6 @@
                     instance,
                 }
             }
-        }
-    }
-
-    fn of(unit_class_tag: u8) -> Option<Self> {
-        match unit_class_tag {
-            1 => Some(ContentUnitClass::Data),
-            3 => Some(ContentUnitClass::PackedRecord),
-            _outside_this_invariants_reach => None,
-        }
-    }
-
-    fn header_offsets(self) -> ContentUnitHeaderOffsets {
-        match self {
-            ContentUnitClass::Data => ContentUnitHeaderOffsets {
-                plain_header_end: 105,
-                payload_checksum: 101,
-                filesystem_identifier: 83,
-                birth_txg: 75,
-                write_order: 91,
-                object_key: 42..75,
-                merge_key_before_payload_checksum: 42..101,
-                merge_key_after_payload_checksum: 105..105,
-            },
-            ContentUnitClass::PackedRecord => ContentUnitHeaderOffsets {
-                plain_header_end: 107,
-                payload_checksum: 89,
-                filesystem_identifier: 81,
-                birth_txg: 73,
-                write_order: 93,
-                object_key: 42..69,
-                merge_key_before_payload_checksum: 42..89,
-                merge_key_after_payload_checksum: 93..107,
-            },
-        }
-    }
-
-    fn name(self) -> &'static str {
-        match self {
-            ContentUnitClass::Data => "码 1 数据",
-            ContentUnitClass::PackedRecord => "码 3 打包记录",
         }
     }
 }
@@ -7365,6 +7298,8 @@
     object_key: Vec<u8>,
     total_order_key: TotalOrderKey,
     payload_checksum: u32,
+    /// 明文头原样（`[0, 明文头末尾)`）：扫描重建（`rebuild` 模块）从它现算中央映射 key 与对象身份，这一份只交过去、这里不再解。
+    plain_header: Vec<u8>,
 }
 
 impl ScannedContentUnit {
@@ -7429,6 +7364,7 @@
         object_key: header[offsets.object_key].to_vec(),
         total_order_key,
         payload_checksum: read_u32(header, offsets.payload_checksum),
+        plain_header: header[..offsets.plain_header_end].to_vec(),
     })
 }
 
@@ -7463,13 +7399,7 @@
 /// 这一份在盘上读得出来、而且它的载荷与它自己头里那个载荷校验和对得上——「可读」的后半截。
 /// 头校验和过而载荷对不上的那一份不可读（撕裂、被盖），不进 I-1.8 的归并组。
 fn payload_checksum_holds_on_disk(reader: &dyn ImageReader, unit: &ScannedContentUnit) -> bool {
-    let offsets = unit.unit_class.header_offsets();
-    let Some(bytes) = reader.read(unit.device, unit.slot * SLOT_BYTES, data_unit_bytes()) else {
-        return false;
-    };
-    bytes.len() > offsets.plain_header_end
-        && read_u32(&bytes, offsets.payload_checksum)
-            == crc32_castagnoli_table(&bytes[offsets.plain_header_end..])
+    copy_payload_checksum_holds_on_disk(reader, unit.unit_class, unit.device, unit.slot)
 }
 
 /// I-1.8（归并后版本全序）：码 1 / 3 的可读已发布单元按「类身份段全部字段含写序」归并成组之后，
@@ -9328,14 +9258,38 @@
         .collect())
 }
 
-/// 一版 inode 树的全部记录（号 → nlink 与 size），只读不判：根层级 0 是零条目根（一条记录都没有，D8（核心索引结构） 已定项 6、16 第 5 条），
-/// 层级 ≥ 1 的往下走（[`collect_inode_records_below`]）。
+/// 一版 inode 树的全部记录（号 → nlink 与 size），只读不判：读法见 [`inode_record_bytes_without_judging`]。
 fn inode_records_without_judging(
     reader: &dyn ImageReader,
     root_pointer: &PointerView,
     index_cache: &mut IndexNodeCache,
     packed_cache: &mut PackedUnitCache,
 ) -> Result<BTreeMap<u64, InodeRecordOfTheDeletionScan>, DeletionScanGap> {
+    Ok(
+        inode_record_bytes_without_judging(reader, root_pointer, index_cache, packed_cache)?
+            .into_iter()
+            .map(|(inode, record)| {
+                (
+                    inode,
+                    InodeRecordOfTheDeletionScan {
+                        object_birth: read_u64(&record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
+                        nlink: read_u32(&record, INODE_RECORD_NLINK_OFFSET),
+                        size_in_bytes: read_u64(&record, INODE_RECORD_SIZE_OFFSET),
+                    },
+                )
+            })
+            .collect(),
+    )
+}
+
+/// 一版 inode 树的全部记录（号 → 记录原样），只读不判：根层级 0 是零条目根（一条记录都没有，D8（核心索引结构） 已定项 6、16 第 5 条），
+/// 层级 ≥ 1 的往下走（[`collect_inode_records_below`]）。删除态扫描取其中三段，扫描重建的比对（I-1.13）整条比。
+fn inode_record_bytes_without_judging(
+    reader: &dyn ImageReader,
+    root_pointer: &PointerView,
+    index_cache: &mut IndexNodeCache,
+    packed_cache: &mut PackedUnitCache,
+) -> Result<BTreeMap<u64, Vec<u8>>, DeletionScanGap> {
     let mut records = BTreeMap::new();
     if root_pointer.all_zero {
         return Ok(records);
@@ -9363,7 +9317,7 @@
     node: &crate::IndexNodeView,
     index_cache: &mut IndexNodeCache,
     packed_cache: &mut PackedUnitCache,
-    records: &mut BTreeMap<u64, InodeRecordOfTheDeletionScan>,
+    records: &mut BTreeMap<u64, Vec<u8>>,
 ) -> Result<(), DeletionScanGap> {
     if node.entry_width < inode_internal_entry_bytes() {
         return Err(DeletionScanGap::InodeTreeUnreadable);
@@ -9383,14 +9337,7 @@
                     return Err(DeletionScanGap::InodeTreeUnreadable);
                 }
                 for record in &container.view.records {
-                    records.insert(
-                        read_u64(record, 0),
-                        InodeRecordOfTheDeletionScan {
-                            object_birth: read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
-                            nlink: read_u32(record, INODE_RECORD_NLINK_OFFSET),
-                            size_in_bytes: read_u64(record, INODE_RECORD_SIZE_OFFSET),
-                        },
-                    );
+                    records.insert(read_u64(record, 0), record.clone());
                 }
             }
             InodeInternalEntryType::InternalNode => {
@@ -9429,12 +9376,14 @@
         cache,
     )
     .ok_or(DeletionScanGap::ExtentTreeUnreadable)?;
-    for (inode, _) in extent_tree
+    for data_pointer_of_an_inode in extent_tree
         .data_pointers_of_inodes
         .iter()
-        .filter(|(_, data_pointer)| !data_pointer.all_zero)
+        .filter(|data_pointer_of_an_inode| !data_pointer_of_an_inode.pointer.all_zero)
     {
-        *data_units_of_each_inode.entry(*inode).or_insert(0) += 1;
+        *data_units_of_each_inode
+            .entry(data_pointer_of_an_inode.inode)
+            .or_insert(0) += 1;
     }
     Ok(data_units_of_each_inode)
 }
@@ -10017,15 +9966,9 @@
     Ok(largest_object_identifier)
 }
 
-/// 墓碑记录 40 字节里另外四段的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8），
-/// 按字段表另写一份、不用实现的解析。
-const TOMBSTONE_RECORD_OBJECT_BIRTH_OFFSET: usize = 8;
-const TOMBSTONE_RECORD_RANGE_START_OFFSET: usize = 16;
-const TOMBSTONE_RECORD_RANGE_END_OFFSET: usize = 24;
+/// 墓碑记录里死亡 txg 的偏移（D18（块里携带什么信息） 已定项 24 第 2 条：对象 ID 8 + 对象出生代 8 + 区间起 8 + 区间止 8 + 死亡 txg 8；
+/// 前四段的偏移住 crate 根，扫描重建也读它们），按字段表另写一份、不用实现的解析。
 const TOMBSTONE_RECORD_DEATH_TXG_OFFSET: usize = 32;
-/// 第一版墓碑只许整对象：区间 [0, 2⁶⁴ − 1]（D18（块里携带什么信息） 已定项 24 第 2 条）。
-const TOMBSTONE_WHOLE_OBJECT_RANGE_START: u64 = 0;
-const TOMBSTONE_WHOLE_OBJECT_RANGE_END: u64 = u64::MAX;
 /// 谱系记录 24 字节：T_rb 8（回退那次发布的 txg）+ T_old 8（回退目标的 txg）+ H_cur 8（cur，即回退之前的现行那一版的 inode 号水位），
 /// 读作「T_rb 的父是 T_old」（D18（块里携带什么信息） 已定项 24 第 5 条；三段的逐字节次序条款没定，实现取 T_rb | T_old | H_cur，按字段表另写一份、不用实现的解析）。
 const LINEAGE_RECORD_ROLLBACK_TXG_OFFSET: usize = 0;
@@ -12189,6 +12132,542 @@
             )
         });
     }
+}
+
+/// 中央映射树的全部叶条目，只读不判（D8（核心索引结构） 已定项 11：多层码 2 树，key 宽 27；叶条目 55 = key 27 + 位置条目 14 × 2，
+/// 内部条目 = key 27 + 子指针 86）。有一个节点读不出、key 宽不是 27、条目宽窄于字段表、孩子层级不是父层级减一、子指针全零 ⇒ None。
+/// 递归的上界：每下一层层级减一，层级 0 停。
+fn central_mapping_leaf_entries_without_judging(
+    reader: &dyn ImageReader,
+    node_pointer: &PointerView,
+    level_the_parent_names: Option<u8>,
+    cache: &mut IndexNodeCache,
+    leaf_entries: &mut Vec<Vec<u8>>,
+) -> Option<()> {
+    let node = read_index_node_without_judging(reader, node_pointer, cache)?;
+    let key_width = KEY_SCHEMA_MAPPING.width();
+    if node.key_width != key_width
+        || level_the_parent_names.is_some_and(|level| level != node.level)
+    {
+        return None;
+    }
+    let Some(child_level) = node.level.checked_sub(1) else {
+        if node.entry_width < mapping_entry_bytes() {
+            return None;
+        }
+        leaf_entries.extend(node.entries.iter().cloned());
+        return Some(());
+    };
+    if node.entry_width < internal_entry_bytes(KEY_SCHEMA_MAPPING) {
+        return None;
+    }
+    for entry in &node.entries {
+        let child = parse_node_pointer(&entry[key_width..key_width + node_pointer_bytes()]);
+        if child.all_zero {
+            return None;
+        }
+        central_mapping_leaf_entries_without_judging(
+            reader,
+            &child,
+            Some(child_level),
+            cache,
+            leaf_entries,
+        )?;
+    }
+    Some(())
+}
+
+/// 一个码 1 数据指针（88 字节，D19（块指针的结构与宽度预算） 已定项 6）指的那个单元的中央映射 key：类标签 1 + 出生树 + 出生 txg + 写序 10，
+/// 各段取指针头里的（遍历方向自己那一边的读法，扫描重建从单元头现算另一边）。
+fn central_mapping_key_of_a_data_pointer(pointer: &PointerView) -> CentralMappingKey {
+    let mut key = vec![0u8; mapping_key_bytes()];
+    key[MAPPING_KEY_UNIT_CLASS_OFFSET] = MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT;
+    key[MAPPING_KEY_BIRTH_TREE_OFFSET..MAPPING_KEY_BIRTH_TXG_OFFSET]
+        .copy_from_slice(&pointer.birth_tree.to_le_bytes());
+    key[MAPPING_KEY_BIRTH_TXG_OFFSET..MAPPING_KEY_INSTANCE_OFFSET]
+        .copy_from_slice(&pointer.birth_txg.to_le_bytes());
+    key[MAPPING_KEY_INSTANCE_OFFSET..MAPPING_KEY_TAIL_OFFSET]
+        .copy_from_slice(&pointer.instance.to_le_bytes());
+    let tail_bytes = mapping_key_bytes() - MAPPING_KEY_TAIL_OFFSET;
+    key[MAPPING_KEY_TAIL_OFFSET..].copy_from_slice(&pointer.tail.to_le_bytes()[..tail_bytes]);
+    CentralMappingKey(key)
+}
+
+/// 中央映射 key 里码 1 的类标签（D18（块里携带什么信息） 已定项 11 的登记表）。
+const MAPPING_KEY_UNIT_CLASS_OF_A_DATA_UNIT: u8 = 1;
+
+fn mapping_key_bytes() -> usize {
+    usize::try_from(MAPPING_KEY_BYTES).expect("27")
+}
+
+/// 遍历方向从最新根读出的三张表（I-1.13（扫描重建与遍历方向的索引相同） 比对的那一边），只读不判。中央映射只留码 1 / 码 3 的条目：
+/// 扫描重建第一版只重建这两类（码 2 节点不进比对，码 0 与未登记码由 I-1.11（映射 key 与单元头相符） 判）。
+struct IndexTablesWalkedFromTheNewestRoot {
+    central_mapping: BTreeMap<CentralMappingKey, [PointerLocation; 2]>,
+    data_units_of_each_inode: BTreeMap<DataUnitPosition, CentralMappingKey>,
+    inode_records: BTreeMap<u64, Vec<u8>>,
+}
+
+/// 遍历方向那三张表读不全的是哪一样：I-1.13 报不适用并带这个理由（读不出、形状不对由主走读的各条判）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum IndexTableTheWalkDoesNotReadInFull {
+    TreeTable,
+    ExtentTree,
+    InodeTree,
+    CentralMappingTree,
+    ExtentTreeNamesOnePositionTwice,
+    CentralMappingTreeNamesOneKeyTwice,
+}
+
+impl IndexTableTheWalkDoesNotReadInFull {
+    fn reason(self) -> &'static str {
+        match self {
+            IndexTableTheWalkDoesNotReadInFull::TreeTable => {
+                "最新根的树表读不出、解不开、条目窄于字段表或同一种树出现两条：遍历方向那三张表读不全"
+            }
+            IndexTableTheWalkDoesNotReadInFull::ExtentTree => {
+                "最新根的 extent 树读不全：遍历方向的每个 inode 的数据单元集合读不全"
+            }
+            IndexTableTheWalkDoesNotReadInFull::InodeTree => {
+                "最新根的 inode 树读不全：遍历方向的 inode 记录集合读不全"
+            }
+            IndexTableTheWalkDoesNotReadInFull::CentralMappingTree => {
+                "最新根的中央映射树读不全：遍历方向的中央映射读不全"
+            }
+            IndexTableTheWalkDoesNotReadInFull::ExtentTreeNamesOnePositionTwice => {
+                "最新根的 extent 树里同一个 (inode, 偏移) 有两条：遍历方向的数据单元集合不是一张表"
+            }
+            IndexTableTheWalkDoesNotReadInFull::CentralMappingTreeNamesOneKeyTwice => {
+                "最新根的中央映射树里同一个 key 有两条：遍历方向的中央映射不是一张表"
+            }
+        }
+    }
+}
+
+/// 最新根下面的三张表：树表、extent 树、inode 树与中央映射树各读一遍（与删除态扫描、I-9.6 那几遍共用节点缓存）。
+fn index_tables_walked_from_the_newest_root(
+    reader: &dyn ImageReader,
+    newest_root_record: &[u8],
+    index_cache: &mut IndexNodeCache,
+) -> Result<IndexTablesWalkedFromTheNewestRoot, IndexTableTheWalkDoesNotReadInFull> {
+    let user_visible_tree_roots =
+        user_visible_tree_root_pointers(reader, newest_root_record, index_cache)
+            .ok_or(IndexTableTheWalkDoesNotReadInFull::TreeTable)?;
+    let mut data_units_of_each_inode = BTreeMap::new();
+    if let Some(extent_root) = user_visible_tree_roots
+        .extent_tree
+        .as_deref()
+        .map(parse_node_pointer)
+        .filter(|pointer| !pointer.all_zero)
+    {
+        let extent_tree = extent_tree_with_the_inode_of_each_data_pointer_without_judging(
+            reader,
+            &extent_root,
+            index_cache,
+        )
+        .ok_or(IndexTableTheWalkDoesNotReadInFull::ExtentTree)?;
+        for data_pointer_of_an_inode in extent_tree
+            .data_pointers_of_inodes
+            .iter()
+            .filter(|data_pointer_of_an_inode| !data_pointer_of_an_inode.pointer.all_zero)
+        {
+            let position = DataUnitPosition {
+                inode: data_pointer_of_an_inode.inode,
+                offset_in_bytes: data_pointer_of_an_inode.offset_in_bytes,
+            };
+            let key = central_mapping_key_of_a_data_pointer(&data_pointer_of_an_inode.pointer);
+            if data_units_of_each_inode.insert(position, key).is_some() {
+                return Err(IndexTableTheWalkDoesNotReadInFull::ExtentTreeNamesOnePositionTwice);
+            }
+        }
+    }
+    let inode_records = match user_visible_tree_roots
+        .inode_tree
+        .as_deref()
+        .map(parse_node_pointer)
+    {
+        Some(inode_root) => inode_record_bytes_without_judging(
+            reader,
+            &inode_root,
+            index_cache,
+            &mut PackedUnitCache::new(),
+        )
+        .map_err(|_gap| IndexTableTheWalkDoesNotReadInFull::InodeTree)?,
+        None => BTreeMap::new(),
+    };
+    let mapping_root = parse_node_pointer(&newest_root_record[ROOT_RECORD_MAPPING_ROOT_POINTER]);
+    let mut leaf_entries = Vec::new();
+    if !mapping_root.all_zero {
+        central_mapping_leaf_entries_without_judging(
+            reader,
+            &mapping_root,
+            None,
+            index_cache,
+            &mut leaf_entries,
+        )
+        .ok_or(IndexTableTheWalkDoesNotReadInFull::CentralMappingTree)?;
+    }
+    let mut central_mapping = BTreeMap::new();
+    for entry in leaf_entries
+        .iter()
+        .filter(|entry| ContentUnitClass::of(entry[MAPPING_KEY_UNIT_CLASS_OFFSET]).is_some())
+    {
+        let locations_pointer: Vec<u8> = [
+            vec![0u8; 50],
+            entry[mapping_key_bytes()..mapping_entry_bytes()].to_vec(),
+            vec![0u8; 8],
+        ]
+        .concat();
+        let key = CentralMappingKey(entry[..mapping_key_bytes()].to_vec());
+        if central_mapping
+            .insert(key, parse_node_pointer(&locations_pointer).locations)
+            .is_some()
+        {
+            return Err(IndexTableTheWalkDoesNotReadInFull::CentralMappingTreeNamesOneKeyTwice);
+        }
+    }
+    Ok(IndexTablesWalkedFromTheNewestRoot {
+        central_mapping,
+        data_units_of_each_inode,
+        inode_records,
+    })
+}
+
+/// I-1.13 比的三张表（判红时写进说明的表名）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+enum IndexTableOfTheScanRebuild {
+    CentralMapping,
+    DataUnitsOfEachInode,
+    InodeRecords,
+}
+
+impl IndexTableOfTheScanRebuild {
+    fn name(self) -> &'static str {
+        match self {
+            IndexTableOfTheScanRebuild::CentralMapping => "中央映射",
+            IndexTableOfTheScanRebuild::DataUnitsOfEachInode => "每个 inode 的数据单元集合",
+            IndexTableOfTheScanRebuild::InodeRecords => "inode 记录集合",
+        }
+    }
+}
+
+/// 一张表上逐 key 比出来的差异：每一处写成「key …：扫描重建 …、遍历方向 …」。
+struct DifferencesInOneTable {
+    table: IndexTableOfTheScanRebuild,
+    differences: Vec<String>,
+}
+
+fn describe_placements(placements: &BTreeSet<UnitPlacement>) -> String {
+    let named: Vec<String> = placements
+        .iter()
+        .map(|placement| format!("盘 {} 槽 {}", placement.device, placement.slot))
+        .collect();
+    format!("落点 {{{}}}", named.join("、"))
+}
+
+fn describe_inode_record(record: &[u8]) -> String {
+    format!(
+        "记录（对象出生代 {}、nlink {}、size {}）",
+        read_u64(record, INODE_RECORD_OBJECT_BIRTH_OFFSET),
+        read_u32(record, INODE_RECORD_NLINK_OFFSET),
+        read_u64(record, INODE_RECORD_SIZE_OFFSET)
+    )
+}
+
+/// 一份码 1 / 码 3 副本在盘上读得出来、载荷与它自己头里的载荷校验和对得上：「可读」的后半截（与 I-1.8 那一遍同一个判法）。
+fn copy_payload_checksum_holds_on_disk(
+    reader: &dyn ImageReader,
+    unit_class: ContentUnitClass,
+    device: u32,
+    slot: u64,
+) -> bool {
+    let offsets = unit_class.header_offsets();
+    let Some(bytes) = reader.read(device, slot * SLOT_BYTES, data_unit_bytes()) else {
+        return false;
+    };
+    bytes.len() > offsets.plain_header_end
+        && read_u32(&bytes, offsets.payload_checksum)
+            == crc32_castagnoli_table(&bytes[offsets.plain_header_end..])
+}
+
+/// 中央映射那张表逐 key 比。一个 key 两边都有时比落点：两边的落点集合相同就对得上；不同时两边各只留读得出来的那几份再比——
+/// 遍历方向那一份按位置条目里的整单元校验和读回、扫描重建那一份按头里的载荷校验和读回（可读的后半截只在两边不同时才现算：
+/// 整单元 CRC 是这里最贵的一步，崩溃点重放每个状态都跑这一遍）。少了一块盘、一份副本读不出的降级镜像因此不判红；
+/// 落点条目改成别的槽（读回对不上）、扫到的那一份不在映射里，判红。
+fn differences_in_the_central_mapping(
+    reader: &dyn ImageReader,
+    rebuilt: &BTreeMap<CentralMappingKey, BTreeSet<UnitPlacement>>,
+    walked: &BTreeMap<CentralMappingKey, [PointerLocation; 2]>,
+) -> Vec<String> {
+    let keys: BTreeSet<&CentralMappingKey> = rebuilt.keys().chain(walked.keys()).collect();
+    let mut differences = Vec::new();
+    for key in keys {
+        match (rebuilt.get(key), walked.get(key)) {
+            (Some(rebuilt_placements), Some(walked_locations)) => {
+                let walked_placements: BTreeSet<UnitPlacement> = walked_locations
+                    .iter()
+                    .map(|location| UnitPlacement {
+                        device: location.device,
+                        slot: location.slot,
+                    })
+                    .collect();
+                if *rebuilt_placements == walked_placements {
+                    continue;
+                }
+                let unit_class = ContentUnitClass::of(key.0[MAPPING_KEY_UNIT_CLASS_OFFSET])
+                    .expect("两边都只留码 1 / 码 3 的 key");
+                let readable_rebuilt: BTreeSet<UnitPlacement> = rebuilt_placements
+                    .iter()
+                    .copied()
+                    .filter(|placement| {
+                        copy_payload_checksum_holds_on_disk(
+                            reader,
+                            unit_class,
+                            placement.device,
+                            placement.slot,
+                        )
+                    })
+                    .collect();
+                let readable_walked: BTreeSet<UnitPlacement> = walked_locations
+                    .iter()
+                    .filter(|location| {
+                        reader
+                            .read(
+                                location.device,
+                                location.slot * SLOT_BYTES,
+                                data_unit_bytes(),
+                            )
+                            .is_some_and(|bytes| {
+                                crc32_castagnoli_table(&bytes) == location.checksum
+                            })
+                    })
+                    .map(|location| UnitPlacement {
+                        device: location.device,
+                        slot: location.slot,
+                    })
+                    .collect();
+                if readable_rebuilt != readable_walked {
+                    differences.push(format!(
+                        "key {}：扫描重建 {}（读得出的 {}）、遍历方向 {}（读得出的 {}）",
+                        key.describe(),
+                        describe_placements(rebuilt_placements),
+                        describe_placements(&readable_rebuilt),
+                        describe_placements(&walked_placements),
+                        describe_placements(&readable_walked)
+                    ));
+                }
+            }
+            (Some(rebuilt_placements), None) => differences.push(format!(
+                "key {}：扫描重建 {}、遍历方向没有这一条",
+                key.describe(),
+                describe_placements(rebuilt_placements)
+            )),
+            (None, Some(walked_locations)) => differences.push(format!(
+                "key {}：扫描重建没有这一条、遍历方向 {}",
+                key.describe(),
+                describe_placements(
+                    &walked_locations
+                        .iter()
+                        .map(|location| UnitPlacement {
+                            device: location.device,
+                            slot: location.slot,
+                        })
+                        .collect()
+                )
+            )),
+            (None, None) => unreachable!("key 取自两边的并集"),
+        }
+    }
+    differences
+}
+
+/// 值不带落点的两张表（每个 inode 的数据单元集合、inode 记录集合）逐 key 比：一边有一边没有、或两边的值不同，各记一处。
+fn differences_in_a_table_of_values<Key: Ord, Value: PartialEq>(
+    rebuilt: &BTreeMap<Key, Value>,
+    walked: &BTreeMap<Key, Value>,
+    describe_key: impl Fn(&Key) -> String,
+    describe_value: impl Fn(&Value) -> String,
+) -> Vec<String> {
+    let keys: BTreeSet<&Key> = rebuilt.keys().chain(walked.keys()).collect();
+    keys.into_iter()
+        .filter_map(|key| match (rebuilt.get(key), walked.get(key)) {
+            (Some(rebuilt_value), Some(walked_value)) if rebuilt_value == walked_value => None,
+            (rebuilt_value, walked_value) => Some(format!(
+                "key {}：扫描重建 {}、遍历方向 {}",
+                describe_key(key),
+                rebuilt_value.map_or_else(|| "没有这一条".to_string(), &describe_value),
+                walked_value.map_or_else(|| "没有这一条".to_string(), &describe_value)
+            )),
+        })
+        .collect()
+}
+
+/// 实例表沿最新根的链读不全时 I-1.13 的不适用理由：已发布谓词的输入不全，扫描方向滤不出哪些单元已发布。
+pub const SCAN_REBUILD_INSTANCE_TABLE_NOT_READ_IN_FULL: &str =
+    "最新根的实例表沿链读不全：已发布谓词的输入不全，扫描重建滤不出哪些单元已发布";
+
+/// I-1.13 比对之前查的那几条（主 agent 2026-10-13 定的口径）：这个镜像上它们里有一条已判违例，就是遍历方向的索引本身已被判损坏——
+/// 映射 key 与单元头不符、extent 指针的出生树与单元头不符、引用了判不成已发布的单元、同一版里一个单元被引用两次——两个方向对不上
+/// 是那一处损坏带出来的，I-1.13 报不适用、理由写明是哪一条红在哪，不另报一条违例。按表的次序取第一条违例了的。
+/// 只收在必红世界 ①（映射条目落点挪槽：另红 I-2.1、I-3.1、I-3.11、I-4.8、I-7.4）与 ②（数据单元两份副本的对象 key 清零：另红 I-2.1、
+/// I-4.8、I-7.2、I-7.4）上都不红的几条（2026-10-13 在 `scan_rebuild_agrees_with_the_walked_index.rs` 那两条用例的镜像上现查），
+/// 免得把两个必红世界一起判成不适用。
+pub const INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED: [(&str, &str); 4] = [
+    (
+        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,
+        "遍历方向的中央映射已判损坏：I-1.11（映射 key 与单元头相符） 违例，两个方向对不上是那一条映射 key 带出来的，扫描重建不另比",
+    ),
+    (
+        POINTER_BIRTH_TREE_MATCHES_THE_UNIT_HEADER,
+        "遍历方向的指针已判损坏：I-1.12（指针出生树等于被指单元头） 违例，两个方向对不上是那一条指针带出来的，扫描重建不另比",
+    ),
+    (
+        "I-4.2",
+        "遍历方向引用了判不成已发布的单元：I-4.2（无被引用未提交块） 违例，扫描方向按已发布谓词不认领它，两个方向对不上是那一处带出来的，扫描重建不另比",
+    ),
+    (
+        "I-5.1",
+        "遍历方向的 extent 指针已判损坏：I-5.1（物理范围不重叠） 违例，同一版里一个单元被两处引用，两个方向对不上是那一处带出来的，扫描重建不另比",
+    ),
+];
+
+/// I-1.13（扫描重建与遍历方向的索引相同）：扫描方向扫出的码 1 / 码 3 已发布单元（与 I-1.8 那一遍同一份扫描结果，已发布谓词同取最新根
+/// 指着的实例表）交给 `rebuild` 模块重建三张表，与遍历方向从最新根读出的三张表逐项比。判一次：三张表都逐项相同成立，否则违例，
+/// 说明里每张对不上的表写差几处与第一处（表名、key、两边的值）。实例表读不全、遍历方向的索引已被别的不变量判损坏
+/// （[`INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`]）、遍历方向那三张表读不全、扫描重建停在条款没写之处
+/// （`rebuild::ScanRebuildNotDecided`，回退过的镜像在其中）报不适用并带理由。只读：这一遍不写盘。
+fn judge_scan_rebuild_agrees_with_the_walked_index(
+    reader: &dyn ImageReader,
+    newest_root_record: &[u8],
+    content_units: &[ScannedContentUnit],
+    abandoned_roots_filtering: AbandonedRootsFilteringOfTheCandidateSet,
+    index_cache: &mut IndexNodeCache,
+    judgements: &mut Judgements,
+) {
+    match abandoned_roots_filtering {
+        AbandonedRootsFilteringOfTheCandidateSet::FilteredByTheNewestRootsInstanceTable => {}
+        AbandonedRootsFilteringOfTheCandidateSet::UnfilteredAsTheMainWalkStoppedBeforeTheLastPageOfTheNewestRootsInstanceTable => {
+            judgements.not_applicable(
+                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
+                SCAN_REBUILD_INSTANCE_TABLE_NOT_READ_IN_FULL,
+            );
+            return;
+        }
+    }
+    if let Some((_, reason)) = INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED
+        .iter()
+        .find(|(invariant, _)| judgements.violation_count(invariant) > 0)
+    {
+        judgements.not_applicable(SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX, reason);
+        return;
+    }
+    let walked =
+        match index_tables_walked_from_the_newest_root(reader, newest_root_record, index_cache) {
+            Ok(walked) => walked,
+            Err(table_not_read_in_full) => {
+                judgements.not_applicable(
+                    SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
+                    table_not_read_in_full.reason(),
+                );
+                return;
+            }
+        };
+    let published_copies: Vec<PublishedUnitCopy> = content_units
+        .iter()
+        .map(|unit| PublishedUnitCopy {
+            placement: UnitPlacement {
+                device: unit.device,
+                slot: unit.slot,
+            },
+            plain_header: unit.plain_header.clone(),
+        })
+        .collect();
+    let rebuilt = choose_current_versions(&published_copies).and_then(|current| {
+        let container_copies: BTreeMap<ObjectIdentity, Vec<Vec<u8>>> = current
+            .packed_containers_whose_records_are_read()
+            .into_iter()
+            .map(|(identity, placements)| {
+                let copies: Vec<Vec<u8>> = placements
+                    .iter()
+                    .filter_map(|placement| {
+                        reader.read(
+                            placement.device,
+                            placement.slot * SLOT_BYTES,
+                            data_unit_bytes(),
+                        )
+                    })
+                    .collect();
+                (identity, copies)
+            })
+            .collect();
+        rebuild_index_tables(&current, &container_copies)
+    });
+    let rebuilt = match rebuilt {
+        Ok(rebuilt) => rebuilt,
+        Err(not_decided) => {
+            judgements.not_applicable(
+                SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
+                not_decided.reason(),
+            );
+            return;
+        }
+    };
+    let tables = [
+        DifferencesInOneTable {
+            table: IndexTableOfTheScanRebuild::CentralMapping,
+            differences: differences_in_the_central_mapping(
+                reader,
+                &rebuilt.central_mapping,
+                &walked.central_mapping,
+            ),
+        },
+        DifferencesInOneTable {
+            table: IndexTableOfTheScanRebuild::DataUnitsOfEachInode,
+            differences: differences_in_a_table_of_values(
+                &rebuilt.data_units_of_each_inode,
+                &walked.data_units_of_each_inode,
+                |position| {
+                    format!(
+                        "（inode {}、偏移 {}）",
+                        position.inode, position.offset_in_bytes
+                    )
+                },
+                CentralMappingKey::describe,
+            ),
+        },
+        DifferencesInOneTable {
+            table: IndexTableOfTheScanRebuild::InodeRecords,
+            differences: differences_in_a_table_of_values(
+                &rebuilt.inode_records,
+                &walked.inode_records,
+                |inode| format!("inode {inode}"),
+                |record| describe_inode_record(record),
+            ),
+        },
+    ];
+    let every_table_agrees = tables.iter().all(|table| table.differences.is_empty());
+    judgements.judge(
+        SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
+        every_table_agrees,
+        || {
+            let tables_that_differ: Vec<String> = tables
+                .iter()
+                .filter(|table| !table.differences.is_empty())
+                .map(|table| {
+                    format!(
+                        "「{}」差 {} 处，第一处 {}",
+                        table.table.name(),
+                        table.differences.len(),
+                        table.differences[0]
+                    )
+                })
+                .collect();
+            format!(
+                "扫描重建的三张表与遍历方向从最新根读出的不同：{}",
+                tables_that_differ.join("；")
+            )
+        },
+    );
 }
 
 /// 池级 checker 的入口：每条第一版不变量都报，没评估到的报「不适用」并带理由。当三棵容器树都没装只供测试的容量旋钮（产品镜像）：
@@ -12469,6 +12948,10 @@
         root_ring_judgements.not_applicable(
             "I-7.3",
             "根环里一条自证过的根都没有：S 空，没有「代号最大者」可谈",
+        );
+        root_ring_judgements.not_applicable(
+            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
+            "根环里一条自证过的根都没有：没有最新根，遍历方向那三张表无从读、已发布谓词也没有输入",
         );
         return root_ring_judgements.into_report();
     }
@@ -13227,6 +13710,16 @@
             "最新那条根（txg {newest_txg}）的水位 {newest_root_watermark}，盘上出现过的最大树 ID {highest_seen}"
         )
     });
+    // I-1.13：扫描重建（只读、不写回）与遍历方向从最新根读出的索引逐项比，接在全部判定之后、同一份 judgements；
+    // 扫描结果用 I-1.8 那一遍的（已发布谓词同取最新根指着的实例表，这是重建方读根的唯一一处）。
+    judge_scan_rebuild_agrees_with_the_walked_index(
+        reader,
+        &roots[newest_index].2.record_bytes,
+        &content_units,
+        abandoned_roots_filtering,
+        &mut index_node_cache,
+        &mut judgements,
+    );
     judgements.into_report()
 }
 
diff --git a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
--- a/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
+++ b/crates/singlefs-checker-tier/src/crash_judge_gpu.rs
@@ -115,7 +115,7 @@
     "违例位读成一个 u128"
 );
 
-/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 92 条——三批 49 条，加里程碑三二·1 那一批 16 条
+/// 池级 checker 内核覆盖到的不变量（`IMPLEMENTED_INVARIANTS` 的下标）：全部 93 条——三批 49 条，加里程碑三二·1 那一批 16 条
 /// （判决 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 11 行），下标 65 的 I-3.5（引用区间的精确性，
 /// 走读内核在回退候选集的版本都走完之后逐对判），再加 C673（三 oracle 合谋测试的补 oracle 改法没做）⑤ ⑥ 那两条
 /// （下标 66 是 I-9.27，走读内核挨着 I-9.16 按表判；下标 67 是 I-5.5，扫描内核每一版重读树表与 extent 树判），
@@ -148,12 +148,14 @@
 /// 下标 90、91 是第 53 批谱系树与退役树的容器容量那两条（判决 `research/prompts/m3-batch50-code-r1-main-verification.md` 第三节 F2）：扫描内核在
 /// 墓碑与谱系那一遍判 I-9.36（谱系树读完容器那一处、退役树读完容器那一处，各一个调用点）；下标 91 的 I-9.37 同下标 79：编进了旋钮值的那一份内核
 /// 才判、没装旋钮的那一份恒不置位。第 56 批 G2 给 I-9.21、I-9.22、I-9.36、I-9.37 补「第一片容器号是 1、相邻加一、每片至少一条」，与 CPU 同判。
+/// 下标 92 的 I-1.13（扫描重建与遍历方向的索引相同） 第一版池级 checker 在 CPU 上判、内核不置位，GPU 与 CPU 对拍对这一条不比
+/// （用户 2026-10-13 定「先进覆盖表但内核不置位，记欠账」，照下标 73 的 I-4.4 的先例；GPU 扫描重建内核是欠账）。
 /// 用户 2026-10-04 定「改回要求全覆盖」：没有「内核还没判」那一张表。
 pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[
     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
     26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
     50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73,
-    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91,
+    74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92,
 ];
 
 // 覆盖表就是池级 checker 的每一条，按下标升序：第 k 项是 k。新立一条不变量而内核没跟上时这里编译不过，不悄悄让 GPU 那一位恒 0。
diff --git a/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs b/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
--- a/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
+++ b/crates/singlefs-harness/tests/checker_cross_links_malformed_nodes_and_mapping_entries.rs
@@ -834,9 +834,12 @@
     });
     write_unit_to_the_same_slot_on_both_devices(&mut image, SLOT_NOBODY_ALLOCATED, &copy);
     let red = violations(&image);
+    // I-1.13（扫描重建与遍历方向的索引相同） 跟着红：50180 那两份原样还在、都过已发布谓词，扫描重建认领它们，遍历方向的映射只记 50400。
+    // 它不进 I-1.13 比对前查的那张表（`walk::INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`）：这里红的 I-3.1、I-3.11
+    // 在 I-1.13 的必红世界 ①（映射条目落点挪槽）上也红，收进表就把那个必红世界一起判成不适用。
     assert_eq!(
         violated_names(&red),
-        ["I-3.1", "I-3.11"],
-        "映射条目指的那一份算进遍历：已分配统计两条对不上，别的不红：{red:?}"
-    );
-}
+        ["I-3.1", "I-3.11", "I-1.13"],
+        "映射条目指的那一份算进遍历：已分配统计两条对不上，扫描重建扫到的原落点遍历方向没有，别的不红：{red:?}"
+    );
+}
diff --git a/crates/singlefs-harness/tests/checker_known_bad_images.rs b/crates/singlefs-harness/tests/checker_known_bad_images.rs
--- a/crates/singlefs-harness/tests/checker_known_bad_images.rs
+++ b/crates/singlefs-harness/tests/checker_known_bad_images.rs
@@ -39,7 +39,7 @@
     RETIREMENT_RECORDS_ON_THE_LINEAGE_ARE_DISTINCT,
     RETIREMENT_RECORD_ON_THE_LINEAGE_NAMES_NO_LIVE_LEAF,
     RETIREMENT_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER, ROOT_RECORDS_LIVE_IN_THE_SLOT_THEIR_TXG_NAMES,
-    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH,
+    ROOT_SLOTS_ALIGNED_TO_THE_DECISION_WIDTH, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
     SOME_DEVICE_CARRIES_A_SELF_VERIFIED_SYSTEM_CONFIGURATION,
     SYSTEM_CONFIGURATION_CARRIES_ONLY_VALUES_THE_READER_ACCEPTS,
     SYSTEM_CONFIGURATION_OWN_DEVICE_NUMBER_IS_THE_DEVICE_IDENTITY,
@@ -1343,8 +1343,24 @@
             FIRST_DATA_UNIT_OF_THE_VERSION_BUILDING_TREES_FROM_THE_MKFS_WATERMARK,
             Box::new(move_the_data_unit_of_the_first_file_two_slots_further),
         ),
+        // I-1.13（扫描重建与遍历方向的索引相同） 的必红世界 ②（`scan_rebuild_agrees_with_the_walked_index.rs` 同一形）：A 的数据单元（50180）
+        // 两份副本头里的对象 key（五元组，42..75）清零、不重封——头校验和对不上，扫描重建判它损坏、不认领，遍历方向照样经 extent 树引用它。
+        // 读不出那个单元，I-2.1、I-4.8、I-7.2、I-7.4 跟着红，所以放在这一张表里（只判「它红」，不判「只红它」）。
+        (
+            SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,
+            Box::new(|image: &mut MemoryPool| {
+                for device in DEVICES {
+                    let mut first_sector = read(image, device, DATA_UNIT * SLOT, 512);
+                    first_sector[DATA_UNIT_OBJECT_KEY].fill(0);
+                    write(image, device, DATA_UNIT * SLOT, &first_sector);
+                }
+            }),
+        ),
     ]
 }
+
+/// 码 1 单元头里对象 key 那一段（五元组：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67，D18（块里携带什么信息） 已定项 7）。
+const DATA_UNIT_OBJECT_KEY: std::ops::Range<usize> = 42..75;
 
 /// 新池新建文件写出的那个数据单元（`DATA_UNIT`，32 KiB）原样搬到往后两槽（写完新池新建文件那份镜像上空着，下一个 32 KiB 对齐的槽对）：
 /// 两盘各写一份；`UNITS` 里引用它的单元（extent 树根里的数据指针、中央映射叶）把位置条目的槽号改成新的、重封，
@@ -3155,6 +3171,9 @@
 const NO_INODE_BEING_DELETED_IN_ANY_WALKED_VERSION: &str = "走读到的每一版里都没有处于删除中的 inode（nlink = 0 的记录、有数据单元而没有 nlink ≥ 1 记录的号）";
 const NO_RECORD_OF_NLINK_ZERO_IN_ANY_WALKED_VERSION: &str =
     "走读到的每一版 inode 树里都没有 nlink = 0 的记录";
+/// 回退过的镜像上 I-1.13（扫描重建与遍历方向的索引相同） 报的理由（`rebuild` 模块停在 C615 那一格）。
+const SCAN_REBUILD_ON_A_ROLLED_BACK_IMAGE: &str =
+    "扫描重建：谱系树里有记录，是管理员回退过的镜像；按谱系父链择现行的读侧没有条款（C615），第一版不接";
 
 #[test]
 fn the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target() {
@@ -3929,6 +3948,9 @@
             if invariant == LINEAGE_TREE_ENTRY_MATCHES_ITS_CHILD_HEADER {
                 // 回退那次发布写出谱系树（一片容器、一条谱系记录）：它的根条目真被评估过且成立。
                 InvariantVerdict::Holds
+            } else if invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX {
+                // 谱系树里有记录：回退过的镜像，按谱系父链择现行的读侧没有条款（C615），扫描重建第一版不接、报不适用（四·1 题面「射程」一段）。
+                InvariantVerdict::NotApplicable(SCAN_REBUILD_ON_A_ROLLED_BACK_IMAGE)
             } else {
                 expected_verdict_with_one_record_per_transaction(invariant)
             },
diff --git a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
--- a/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
+++ b/crates/singlefs-harness/tests/checker_narrow_invariants_and_abandoned_roots.rs
@@ -46,6 +46,10 @@
 
 /// 第一个文件版本的单元区只用到 50 26x 附近：这两个槽在两块盘上都没被分配、没被引用。
 const SLOT_NOBODY_ALLOCATED: u64 = 50_400;
+/// 手放进 [`SLOT_NOBODY_ALLOCATED`] 的第二片 inode 叶容器的出生序号：inode 树在第一个文件那一版（txg 3）里发过的号之外的一个。
+const SECOND_LEAF_CONTAINER_BIRTH_SEQUENCE: u32 = 200;
+/// 码 3 单元头里出生序号 4 字节的偏移（D18（块里携带什么信息） 已定项 11）。
+const PACKED_BIRTH_SEQUENCE_OFFSET: usize = 103;
 
 /// 根槽：判定宽度 512，自证校验和 32 字节在 138；实例代号在 24、checkpoint_txg 在 28；实例表指针 86 字节在 170
 /// （D22（单元原子性怎么合成） 已定项 7 的偏移表）。
@@ -855,7 +859,8 @@
 // ─── ⑤ I-9.4（容器号不超最小 key） 三句都判 ───
 
 /// 第一个文件版本那片叶容器 C1（容器号 1）改成装 inode 1 与 10 两条记录；新放一片 C2（容器号 `second_container`，装 inode 20 与 30），
-/// 根里加一条分隔 key 15 的类型段 2 条目指它。两片占的槽记进记账，inode 号水位抬到 31。
+/// 根里加一条分隔 key 15 的类型段 2 条目指它。C2 照写路径那样有自己的出生序号（头里与指它的指针里同写），中央映射里也给它加一条条目
+/// （I-1.13（扫描重建与遍历方向的索引相同） 在这几份上才真比得上）。两片占的槽记进记账，inode 号水位抬到 31。
 /// I-9.12（分隔 key 落在孩子区间之外） 在这几份上都成立：15 ≤ 20、15 > 1、15 > 10。
 fn image_with_two_leaf_containers(pool: &TreeSplitPool, second_container: u64) -> MemoryPool {
     let mut image = pool.memory_pool();
@@ -895,7 +900,11 @@
         unit
     };
     let first_container = container_with(1, [1, 10]);
-    let second_container_unit = container_with(second_container, [20, 30]);
+    // 第二片照写路径那样有自己的出生序号（同一棵树、同一 txg 里每片叶各发一个号），映射 key 才与第一片不同。
+    let mut second_container_unit = container_with(second_container, [20, 30]);
+    second_container_unit[PACKED_BIRTH_SEQUENCE_OFFSET..PACKED_BIRTH_SEQUENCE_OFFSET + 4]
+        .copy_from_slice(&SECOND_LEAF_CONTAINER_BIRTH_SEQUENCE.to_le_bytes());
+    reseal_unit_by_its_class(&mut second_container_unit);
     let units = units_found_on_device_zero(&image);
     replace_unit(&mut image, &units, leaf.slot.0, &first_container);
     write_unit_to_the_same_slot_on_both_devices(
@@ -919,6 +928,9 @@
             [start + LOCATION_ENTRY_CHECKSUM_OFFSET..start + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
             .copy_from_slice(&crc32_castagnoli(&second_container_unit).to_le_bytes());
     }
+    let pointer_birth_sequence = INODE_ENTRY_CHILD_POINTER_OFFSET + POINTER_BIRTH_SEQUENCE_OFFSET;
+    entry_to_the_second[pointer_birth_sequence..pointer_birth_sequence + 4]
+        .copy_from_slice(&SECOND_LEAF_CONTAINER_BIRTH_SEQUENCE.to_le_bytes());
     let entries = vec![root_header.entries[0].clone(), entry_to_the_second];
     let grown_root = rebuilt_index_node(
         &root_header,
@@ -929,8 +941,64 @@
         &entries,
     );
     replace_unit(&mut image, &units, inode_root_slot, &grown_root);
+    add_the_mapping_entry_of_a_unit_placed_by_hand(
+        &mut image,
+        &pool.output,
+        leaf.slot.0,
+        SECOND_LEAF_CONTAINER_BIRTH_SEQUENCE,
+        &second_container_unit,
+    );
     account_for_units_placed_by_hand(&mut image, &pool.output, 2, Some(31));
     image
+}
+
+/// 中央映射（产品容量下第一个文件版本是根兼叶）里给手放进 [`SLOT_NOBODY_ALLOCATED`] 的那一片码 3 加一条条目：照 `template_slot`
+/// 那一片（同树、同 txg、同实例）的条目抄，key 尾段的出生序号换成 `birth_sequence`，两条位置条目改指那个槽、整单元校验和换成它的；
+/// 按 key 重新排序、头里的区间贴紧首末条目，重建、补校验和。写路径上每片码 3 都进映射（D19（块指针的结构与宽度预算） 已定项 5）。
+fn add_the_mapping_entry_of_a_unit_placed_by_hand(
+    image: &mut MemoryPool,
+    output: &TransactionOutput,
+    template_slot: u64,
+    birth_sequence: u32,
+    unit_placed_by_hand: &[u8],
+) {
+    let mapping_slot = output.unit(TransactionUnit::MappingTree).slot.0;
+    let mapping = read_unit_on_device(image, 0, mapping_slot, node_bytes());
+    let header = parse_index_node(&mapping).expect("映射根解得开");
+    assert_eq!(header.level, 0, "产品容量下第一个文件版本的映射树是根兼叶");
+    let mut entries = header.entries.clone();
+    let mut added = entries
+        .iter()
+        .find(|entry| {
+            entry[MAPPING_KEY_CLASS_OFFSET] == UNIT_CLASS_PACKED
+                && read_six_byte_slot_at(entry, MAPPING_KEY_BYTES + LOCATION_ENTRY_SLOT_OFFSET)
+                    == template_slot
+        })
+        .expect("映射里有照着抄的那一片的条目")
+        .clone();
+    added[MAPPING_KEY_TAIL_OFFSET..MAPPING_KEY_TAIL_OFFSET + 4]
+        .copy_from_slice(&birth_sequence.to_le_bytes());
+    for location in 0..2 {
+        let start = MAPPING_KEY_BYTES + location * LOCATION_ENTRY_BYTES;
+        added[start + LOCATION_ENTRY_SLOT_OFFSET..start + LOCATION_ENTRY_SLOT_OFFSET + 6]
+            .copy_from_slice(&SLOT_NOBODY_ALLOCATED.to_le_bytes()[..6]);
+        added[start + LOCATION_ENTRY_CHECKSUM_OFFSET..start + LOCATION_ENTRY_CHECKSUM_OFFSET + 4]
+            .copy_from_slice(&crc32_castagnoli(unit_placed_by_hand).to_le_bytes());
+    }
+    entries.push(added);
+    entries.sort_by_key(|entry| mapping_key_order(entry));
+    let smallest_key = entries.first().expect("有条目")[..MAPPING_KEY_BYTES].to_vec();
+    let largest_key = entries.last().expect("有条目")[..MAPPING_KEY_BYTES].to_vec();
+    let grown = rebuilt_index_node(
+        &header,
+        header.level,
+        &smallest_key,
+        &largest_key,
+        header.birth_sequence,
+        &entries,
+    );
+    let units = units_found_on_device_zero(image);
+    replace_unit(image, &units, mapping_slot, &grown);
 }
 
 /// 阳性对照：第二片的容器号 11（> 左边最大 key 10、> 左边容器号 1）：三句都成立，一条都不红，I-9.4 真被评估过。
@@ -941,6 +1009,11 @@
     let red = violations(&image);
     assert_eq!(red, Vec::new(), "两片叶按序排好，一条都不许红：{red:?}");
     assert_eq!(verdict_of(&image, "I-9.4"), InvariantVerdict::Holds);
+    assert_eq!(
+        verdict_of(&image, "I-1.13"),
+        InvariantVerdict::Holds,
+        "两片叶各有自己的出生序号与映射条目：扫描重建的三张表与遍历方向逐项相同，真比过、成立"
+    );
 }
 
 /// 第二片的容器号 8：它装的 20、30 都 ≥ 8（第一句成立），容器号 1 < 8（第二句成立），左边容器的最大 key 10 不小于右边的容器号 8，
@@ -958,6 +1031,11 @@
         "借之后的两片叶是合法形状，一条都不许红：{red:?}"
     );
     assert_eq!(verdict_of(&image, "I-9.4"), InvariantVerdict::Holds);
+    assert_eq!(
+        verdict_of(&image, "I-1.13"),
+        InvariantVerdict::Holds,
+        "借之后的两片叶各有自己的出生序号与映射条目：扫描重建与遍历方向逐项相同"
+    );
 }
 
 /// 第二片的容器号 0：沿叶序容器号 1 → 0 往回走（第二句不成立；第三句跟着不成立）。违例说明取第一处，要是第二句那一格。
@@ -1438,6 +1516,17 @@
         "只该红映射 key 那一条：{red:?}"
     );
     assert!(red[0].1.contains("补零"), "说明要点出补零：{}", red[0].1);
+    match verdict_of(&image, "I-1.13") {
+        InvariantVerdict::NotApplicable(reason) => assert!(
+            reason.contains("I-1.11"),
+            "遍历方向的映射 key 已判损坏：I-1.13 报不适用，理由点名红的那一条，实际 {reason}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => panic!(
+            "映射 key 已被 I-1.11 判损坏的镜像上 I-1.13 应当报不适用（主 agent 2026-10-13 定的口径），实际 {other:?}"
+        ),
+    }
 }
 
 /// 数据单元（码 1）与 inode 叶容器（码 3）那条映射条目 key 里的出生树加一：与被指单元头里偏移 43 的出生树不再相等。改之前全绿。
diff --git a/crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs b/crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs
@@ -0,0 +1,264 @@
+//! 只读扫描重建与遍历方向的索引逐项比（里程碑四四·2，题面 `.claude/kb/milestone/04-fourth-txn.md`「四·1 题面」）：
+//! 池级 checker 的 I-1.13（扫描重建与遍历方向的索引相同）在新池新建文件、覆盖写几版、unlink 之后的镜像上成立；
+//! 必红世界 ① 改掉遍历方向中央映射叶里一条条目的落点、② 抹掉一个码 1 单元头的对象 key，各自判违例且差异落在题面写的那张表；
+//! 扫描重建停在条款没写之处的那几格（删除中、文件变短留下的单元）报不适用，不判成立也不判违例。
+//! 必红世界 ③（往重建模块里加一行调走树函数的代码）由 `crates/singlefs-checker/src/rebuild.rs` 末尾的白名单单测判。
+
+mod common;
+mod common_deletion_in_progress;
+mod common_pointer_head_corruption;
+
+use common::{
+    build_pool, file_content, overwrite_in_process_as_instance_one_and_advance_the_pool,
+    sequential_write_in_process_as_instance_one_and_advance_the_pool, BuiltPool,
+};
+use common_deletion_in_progress::{
+    content_of_three_data_units, image_after_the_intent_of_a_three_unit_file, pool_with_new_inodes,
+    unlink_to_the_end,
+};
+use common_pointer_head_corruption::{
+    newest_root, node_bytes, replace_the_unit_and_propagate_its_checksum, the_slot_of,
+    unit_bytes_at,
+};
+use singlefs_checker::image::{InvariantVerdict, SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX};
+use singlefs_checker::walk::check_pool_image;
+use singlefs_core::checksum::crc32_castagnoli;
+use singlefs_core::pointer::LocationEntry;
+use singlefs_core::transaction::FIRST_INODE_NUMBER;
+use singlefs_core::unit::seal_header_checksum;
+use singlefs_harness::memory_pool::MemoryPool;
+
+/// 中央映射树节点（码 2，key 宽 27）：载荷 CRC 在 76 + 2k、明文头到 86 + 2k（D18（块里携带什么信息） 已定项 18 的偏移表）；
+/// 叶条目 55 = key 27 + 位置条目 14 × 2，位置条目 = 设备身份 4 + 槽号 6 + 整单元校验和 4（D19（块指针的结构与宽度预算） 已定项 6 / 10）。
+const MAPPING_KEY_WIDTH: usize = 27;
+const MAPPING_ENTRY_BYTES: usize = 55;
+const FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY: usize = MAPPING_KEY_WIDTH + 4;
+/// 码 1 单元头里对象 key 那一段（五元组：类标签 42、树 43、对象 51、对象出生代 59、锚点偏移 67，D18（块里携带什么信息） 已定项 7）。
+const DATA_UNIT_OBJECT_KEY: std::ops::Range<usize> = 42..75;
+/// 单元头所在的第一扇区（内存盘按 512 字节的扇区写）。
+const HEADER_SECTOR_BYTES: usize = 512;
+/// 码 1 数据单元在中央映射 key 里的类标签。
+const MAPPING_KEY_CLASS_OF_A_DATA_UNIT: u8 = 1;
+/// 换上去的那个槽离原槽多远：单元区里新池新建文件之后没写过的一格（一个 32 KiB 单元占两槽，挪两槽正好落在下一个单元的位置上）。
+const SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY: u64 = 2;
+
+fn verdict_of_the_scan_rebuild(image: &MemoryPool) -> InvariantVerdict {
+    check_pool_image(image)
+        .into_iter()
+        .find(|(invariant, _)| *invariant == SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX)
+        .expect("I-1.13 在池级 checker 的清单里")
+        .1
+}
+
+fn violation_detail_of_the_scan_rebuild(image: &MemoryPool, what: &str) -> String {
+    match verdict_of_the_scan_rebuild(image) {
+        InvariantVerdict::Violated(detail) => detail,
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::NotApplicable(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("{what}：I-1.13 应当判违例，实际 {other:?}")
+        }
+    }
+}
+
+/// 一个和第一个文件一样长、内容不同的版本（第 `seed` 版）：覆盖写不改长度，上一版的单元全由新版本盖住。
+fn content_of_the_same_length(seed: usize) -> Vec<u8> {
+    file_content()
+        .iter()
+        .map(|byte| byte.wrapping_add(u8::try_from(seed).expect("版本序号小于 256")))
+        .collect()
+}
+
+/// 码 2 节点改过之后重封：先算载荷 CRC（罩 [86 + 2k, 末尾)、住 76 + 2k），再算头校验和（罩 [0, 86 + 2k)）。
+fn reseal_the_index_node(bytes: &mut [u8]) {
+    let key_width = usize::from(bytes[51]);
+    let header_end = 86 + 2 * key_width;
+    let payload_checksum_offset = 76 + 2 * key_width;
+    let payload_checksum = crc32_castagnoli(&bytes[header_end..]);
+    bytes[payload_checksum_offset..payload_checksum_offset + 4]
+        .copy_from_slice(&payload_checksum.to_le_bytes());
+    seal_header_checksum(bytes, header_end);
+}
+
+/// 必红世界 ①：最新根的中央映射树（新池新建文件之后是根兼叶）里第一个文件那个数据单元的条目，第一条位置条目的槽号挪到别的槽；
+/// 映射叶重封、新的整单元校验和沿引用链补到根槽。数据单元与 extent 树都不动。交回原来那个槽。
+fn move_the_first_location_of_the_data_unit_mapping_entry(image: &mut MemoryPool) -> u64 {
+    let mapping_root = newest_root(image).mapping_root;
+    let leaf_slot = the_slot_of(&mapping_root);
+    let mut leaf = unit_bytes_at(image, leaf_slot, node_bytes());
+    assert_eq!(
+        usize::from(leaf[51]),
+        MAPPING_KEY_WIDTH,
+        "中央映射树的根兼叶 key 宽 27"
+    );
+    assert_eq!(leaf[50], 0, "新池新建文件之后中央映射树只有一层：根就是叶");
+    let entries_start = 115 + 2 * MAPPING_KEY_WIDTH;
+    let entry_count = usize::from(u16::from_le_bytes(
+        leaf[82 + 2 * MAPPING_KEY_WIDTH..84 + 2 * MAPPING_KEY_WIDTH]
+            .try_into()
+            .expect("2 字节"),
+    ));
+    let data_unit_entry_start = (0..entry_count)
+        .map(|index| entries_start + index * MAPPING_ENTRY_BYTES)
+        .find(|start| leaf[*start] == MAPPING_KEY_CLASS_OF_A_DATA_UNIT)
+        .expect("中央映射里有第一个文件那个数据单元的条目");
+    let slot_field_start = data_unit_entry_start + FIRST_LOCATION_SLOT_OFFSET_IN_A_MAPPING_ENTRY;
+    let mut slot_bytes = [0u8; 8];
+    slot_bytes[..6].copy_from_slice(&leaf[slot_field_start..slot_field_start + 6]);
+    let original_slot = u64::from_le_bytes(slot_bytes);
+    let moved_slot = original_slot + SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY;
+    leaf[slot_field_start..slot_field_start + 6].copy_from_slice(&moved_slot.to_le_bytes()[..6]);
+    reseal_the_index_node(&mut leaf);
+    replace_the_unit_and_propagate_its_checksum(image, leaf_slot, &leaf);
+    original_slot
+}
+
+/// 必红世界 ②：第一个文件那个数据单元两份副本的头里，对象 key（五元组）清零、不重封——头校验和随之对不上，扫描重建判它损坏、不认领。
+/// 只抹一份时另一份照样扫得到、重建出的三张表不变，那是降级镜像，不是这个世界。
+fn zero_the_object_key_of_both_copies_of_the_data_unit(image: &mut MemoryPool, pool: &BuiltPool) {
+    for location in pool.output.data_pointers[0].locations {
+        zero_the_object_key_of_the_copy_at(image, location);
+    }
+}
+
+/// 一份码 1 副本头里的对象 key 清零、不重封：内存盘按扇区写，读回头所在那一扇区、改、整扇区写回。
+fn zero_the_object_key_of_the_copy_at(image: &mut MemoryPool, location: LocationEntry) {
+    let device = image
+        .devices
+        .get_mut(&location.device)
+        .expect("位置条目指的盘在池里");
+    let sector_start = location.slot.to_device_offset();
+    let mut first_sector = device.read(sector_start, HEADER_SECTOR_BYTES);
+    first_sector[DATA_UNIT_OBJECT_KEY].fill(0);
+    device.write(sector_start, &first_sector);
+}
+
+#[test]
+fn the_scan_rebuild_agrees_with_the_walked_index_after_the_new_pool_file_creation() {
+    let pool = build_pool("scan-rebuild-new-pool");
+    assert_eq!(
+        verdict_of_the_scan_rebuild(&pool.memory_pool()),
+        InvariantVerdict::Holds,
+        "新池新建文件之后：一个数据单元、一片 inode 叶容器，扫描重建的三张表与遍历方向逐项相同"
+    );
+}
+
+#[test]
+fn the_scan_rebuild_agrees_with_the_walked_index_after_three_overwrites_of_the_same_length() {
+    let mut pool = build_pool("scan-rebuild-overwrites");
+    for seed in 1..=3 {
+        overwrite_in_process_as_instance_one_and_advance_the_pool(
+            &mut pool,
+            &content_of_the_same_length(seed),
+        );
+    }
+    assert_eq!(
+        verdict_of_the_scan_rebuild(&pool.memory_pool()),
+        InvariantVerdict::Holds,
+        "同一个文件覆盖写三版（长度不变）：盘上四版数据单元与四版 inode 叶容器都过已发布谓词，按 C113 定案 P3 择出最后一版，与遍历方向逐项相同"
+    );
+}
+
+#[test]
+fn the_scan_rebuild_agrees_with_the_walked_index_after_an_unlink_to_the_end() {
+    let mut pool = pool_with_new_inodes("scan-rebuild-unlink", 2);
+    unlink_to_the_end(&mut pool, FIRST_INODE_NUMBER);
+    assert_eq!(
+        verdict_of_the_scan_rebuild(&pool.memory_pool()),
+        InvariantVerdict::Holds,
+        "再建两个空 inode 之后把第一个文件 unlink 到底：它的记录删掉、墓碑留下，盘上那个数据单元由墓碑判死（D11 已定项 4），叶容器里还有另两条记录、不退役，三张表与遍历方向逐项相同"
+    );
+}
+
+#[test]
+fn a_mapping_entry_whose_location_names_another_slot_reddens_the_scan_rebuild_in_the_central_mapping_only(
+) {
+    let pool = build_pool("scan-rebuild-moved-mapping-location");
+    let mut image = pool.memory_pool();
+    let original_slot = move_the_first_location_of_the_data_unit_mapping_entry(&mut image);
+    let detail =
+        violation_detail_of_the_scan_rebuild(&image, "映射叶里数据单元那条条目的落点挪了槽");
+    assert!(
+        detail.contains("「中央映射」差 1 处"),
+        "差异落在中央映射那张表、只差那一条：{detail}"
+    );
+    assert!(
+        detail.contains(&format!("盘 0 槽 {original_slot}"))
+            && detail.contains(&format!(
+                "盘 0 槽 {}",
+                original_slot + SLOTS_THE_MAPPING_LOCATION_IS_MOVED_BY
+            )),
+        "说明里写出两边的落点（扫描重建扫到的原槽、遍历方向映射里的新槽）：{detail}"
+    );
+    assert!(
+        !detail.contains("「每个 inode 的数据单元集合」") && !detail.contains("「inode 记录集合」"),
+        "extent 树与 inode 树都没动，另两张表不该有差异：{detail}"
+    );
+}
+
+#[test]
+fn a_data_unit_whose_object_key_is_zeroed_on_both_copies_reddens_the_scan_rebuild_in_the_data_unit_table(
+) {
+    let pool = build_pool("scan-rebuild-zeroed-object-key");
+    let mut image = pool.memory_pool();
+    zero_the_object_key_of_both_copies_of_the_data_unit(&mut image, &pool);
+    let detail = violation_detail_of_the_scan_rebuild(&image, "数据单元两份的对象 key 清零");
+    assert!(
+        detail.contains("「每个 inode 的数据单元集合」差 1 处")
+            && detail.contains("（inode 1、偏移 0）：扫描重建 没有这一条"),
+        "那个单元判损坏、不认领：数据单元集合里 (inode 1, 偏移 0) 只有遍历方向那一边有：{detail}"
+    );
+    assert!(
+        !detail.contains("「inode 记录集合」"),
+        "inode 叶容器没动，inode 记录集合不该有差异：{detail}"
+    );
+}
+
+#[test]
+fn a_deletion_in_progress_is_not_applicable_to_the_scan_rebuild() {
+    let image = image_after_the_intent_of_a_three_unit_file("scan-rebuild-deletion-in-progress");
+    match verdict_of_the_scan_rebuild(&image) {
+        InvariantVerdict::NotApplicable(reason) => assert!(
+            reason.contains("删除中"),
+            "意图之后墓碑压着、记录还在：报不适用，理由写删除中，实际 {reason}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("删除中的镜像上扫描重建停在条款没写之处，应当报不适用，实际 {other:?}")
+        }
+    }
+}
+
+#[test]
+fn a_data_unit_left_beyond_the_size_by_a_shorter_overwrite_is_not_applicable_to_the_scan_rebuild() {
+    let mut pool = build_pool("scan-rebuild-shorter-overwrite");
+    sequential_write_in_process_as_instance_one_and_advance_the_pool(
+        &mut pool,
+        &content_of_three_data_units(),
+    );
+    overwrite_in_process_as_instance_one_and_advance_the_pool(&mut pool, &file_content());
+    match verdict_of_the_scan_rebuild(&pool.memory_pool()) {
+        InvariantVerdict::NotApplicable(reason) => assert!(
+            reason.contains("size 之外"),
+            "三个单元的文件覆盖写成一个单元：后两个单元既没有新版本也没有墓碑，报不适用，实际 {reason}"
+        ),
+        other @ (InvariantVerdict::Holds
+        | InvariantVerdict::Violated(_)
+        | InvariantVerdict::PartiallyJudged(_)) => {
+            panic!("文件变短留下的单元认不认领第一版不定，应当报不适用，实际 {other:?}")
+        }
+    }
+}
+
+#[test]
+fn zeroing_the_object_key_of_one_copy_leaves_the_scan_rebuild_holding() {
+    let pool = build_pool("scan-rebuild-one-copy-zeroed");
+    let mut image = pool.memory_pool();
+    zero_the_object_key_of_the_copy_at(&mut image, pool.output.data_pointers[0].locations[0]);
+    assert_eq!(
+        verdict_of_the_scan_rebuild(&image),
+        InvariantVerdict::Holds,
+        "只抹一份：另一份照样扫得到，两边各只留读得出的落点再比，三张表相同（降级不判红）"
+    );
+}
````

## m4-rebuild-writer-2：mutations-append.tsv

````tsv
扫描重建比对少比数据单元集合那张表（I-1.13，四·2）	crates/singlefs-checker/src/walk.rs	                &rebuilt.data_units_of_each_inode,\n                &walked.data_units_of_each_inode,	                &walked.data_units_of_each_inode,\n                &walked.data_units_of_each_inode,	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- a_data_unit_whose_object_key_is_zeroed_on_both_copies	a_data_unit_whose_object_key_is_zeroed_on_both_copies_reddens_the_scan_rebuild_in_the_data_unit_table
扫描重建比对中央映射不比落点（I-1.13，四·2 必红世界 ①）	crates/singlefs-checker/src/walk.rs	                if readable_rebuilt != readable_walked {	                if false && readable_rebuilt != readable_walked {	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- a_mapping_entry_whose_location_names_another_slot	a_mapping_entry_whose_location_names_another_slot_reddens_the_scan_rebuild_in_the_central_mapping_only
扫描重建择现行取最旧那一版（C113 定案 P3，四·2）	crates/singlefs-checker/src/rebuild.rs	            ordered.pop().expect("一个对象身份是从至少一份副本建出来的");	            ordered.drain(..1).next().expect("一个对象身份是从至少一份副本建出来的");	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- the_scan_rebuild_agrees_with_the_walked_index_after_three_overwrites	the_scan_rebuild_agrees_with_the_walked_index_after_three_overwrites_of_the_same_length
扫描重建模块按路径调走树函数（白名单放行，C410，四·1 必红世界 ③）	crates/singlefs-checker/src/rebuild.rs	    let payload_capacity_in_bytes = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET;	    let payload_capacity_in_bytes = DATA_UNIT_BYTES - DATA_UNIT_PAYLOAD_OFFSET;\n    let _report_of_the_walk = crate::walk::check_pool_image;	-p singlefs-checker --lib -- rebuild::tests::the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path	rebuild::tests::the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path
扫描重建模块 use 清单多引走树的一项（白名单放行，C410，四·1 必红世界 ③）	crates/singlefs-checker/src/rebuild.rs	use crate::{\n    packed_unit_view,	use crate::walk::check_pool_image;\nuse crate::{\n    packed_unit_view,	-p singlefs-checker --lib -- rebuild::tests::the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist	rebuild::tests::the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist
扫描重建墓碑不判 key 死亡（D11 已定项 4，四·2）	crates/singlefs-checker/src/rebuild.rs	        if tombstoned_objects.contains(&(inode, object_birth)) {	        if false && tombstoned_objects.contains(&(inode, object_birth)) {	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- the_scan_rebuild_agrees_with_the_walked_index_after_an_unlink_to_the_end	the_scan_rebuild_agrees_with_the_walked_index_after_an_unlink_to_the_end
扫描重建删除中不停下（条款没写，四·2）	crates/singlefs-checker/src/rebuild.rs	            if record_of_this_object.is_some() {	            if false && record_of_this_object.is_some() {	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- a_deletion_in_progress_is_not_applicable_to_the_scan_rebuild	a_deletion_in_progress_is_not_applicable_to_the_scan_rebuild
扫描重建 size 之外的单元照认领（条款没写，四·2）	crates/singlefs-checker/src/rebuild.rs	            if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {	            if false && offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- a_data_unit_left_beyond_the_size_by_a_shorter_overwrite	a_data_unit_left_beyond_the_size_by_a_shorter_overwrite_is_not_applicable_to_the_scan_rebuild
扫描重建把实例表的片也放进映射（D19 已定项 8 / 12，四·2）	crates/singlefs-checker/src/rebuild.rs	        if record_type != PACKED_TYPE_INSTANCE_TABLE {	        if record_type == record_type {	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- the_scan_rebuild_agrees_with_the_walked_index_after_the_new_pool_file_creation	the_scan_rebuild_agrees_with_the_walked_index_after_the_new_pool_file_creation
扫描重建落点比对不看读不读得出（降级镜像，四·2）	crates/singlefs-checker/src/walk.rs	                if readable_rebuilt != readable_walked {	                if *rebuilt_placements != walked_placements {	-p singlefs-harness --test scan_rebuild_agrees_with_the_walked_index -- zeroing_the_object_key_of_one_copy_leaves_the_scan_rebuild_holding	zeroing_the_object_key_of_one_copy_leaves_the_scan_rebuild_holding
扫描重建择现行取最旧那一版（单测，C113 定案 P3）	crates/singlefs-checker/src/rebuild.rs	            ordered.pop().expect("一个对象身份是从至少一份副本建出来的");	            ordered.drain(..1).next().expect("一个对象身份是从至少一份副本建出来的");	-p singlefs-checker --lib -- rebuild::tests::the_current_version_of_a_data_unit_is_the_one_with_the_largest_write_order	rebuild::tests::the_current_version_of_a_data_unit_is_the_one_with_the_largest_write_order
扫描重建同键两版不停下（码 1，C464）	crates/singlefs-checker/src/rebuild.rs	        if ordered\n            .last()\n            .is_some_and(|(order_key, _)| *order_key == current_order_key)	        if false && ordered\n            .last()\n            .is_some_and(|(order_key, _)| *order_key == current_order_key)	-p singlefs-checker --lib -- rebuild::tests::two_data_unit_versions_with_the_same_write_order_are_not_decided_until_c464	rebuild::tests::two_data_unit_versions_with_the_same_write_order_are_not_decided_until_c464
扫描重建码 3 同键两版报成码 1 那一格（I-1.8 ②）	crates/singlefs-checker/src/rebuild.rs	                ContentUnitClass::PackedRecord => {\n                    ScanRebuildNotDecided::PackedRecordVersionsTiedOnTheTotalOrderKeyOfI18\n                }	                ContentUnitClass::PackedRecord => {\n                    ScanRebuildNotDecided::DataUnitVersionsTiedOnTheWriteOrderUntilC464IsDecided\n                }	-p singlefs-checker --lib -- rebuild::tests::two_packed_versions_with_the_same_total_order_key_are_not_decided	rebuild::tests::two_packed_versions_with_the_same_total_order_key_are_not_decided
扫描重建同一版副本载荷校验和不同照收（I-1.8 ①）	crates/singlefs-checker/src/rebuild.rs	        if version.payload_checksum != payload_checksum {	        if false && version.payload_checksum != payload_checksum {	-p singlefs-checker --lib -- rebuild::tests::copies_of_one_version_with_different_payload_checksums_are_not_decided	rebuild::tests::copies_of_one_version_with_different_payload_checksums_are_not_decided
扫描重建 inode 记录按错的号收（四·2）	crates/singlefs-checker/src/rebuild.rs	                    if inode_records.insert(read_u64(&record, 0), record).is_some() {	                    if inode_records.insert(read_u64(&record, 0) + 1, record).is_some() {	-p singlefs-checker --lib -- rebuild::tests::the_inode_records_table_is_read_from_the_current_inode_container	rebuild::tests::the_inode_records_table_is_read_from_the_current_inode_container
扫描重建墓碑不判 key 死亡（单测，D11 已定项 4）	crates/singlefs-checker/src/rebuild.rs	        if tombstoned_objects.contains(&(inode, object_birth)) {	        if false && tombstoned_objects.contains(&(inode, object_birth)) {	-p singlefs-checker --lib -- rebuild::tests::a_whole_object_tombstone_kills_the_data_units_of_an_object_whose_record_is_gone	rebuild::tests::a_whole_object_tombstone_kills_the_data_units_of_an_object_whose_record_is_gone
扫描重建删除中不停下（单测）	crates/singlefs-checker/src/rebuild.rs	            if record_of_this_object.is_some() {	            if false && record_of_this_object.is_some() {	-p singlefs-checker --lib -- rebuild::tests::a_tombstoned_object_that_still_has_its_record_is_a_deletion_in_progress_and_not_decided	rebuild::tests::a_tombstoned_object_that_still_has_its_record_is_a_deletion_in_progress_and_not_decided
扫描重建 size 之外的单元照认领（单测）	crates/singlefs-checker/src/rebuild.rs	            if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {	            if false && offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {	-p singlefs-checker --lib -- rebuild::tests::a_data_unit_beyond_the_size_of_its_record_is_not_decided	rebuild::tests::a_data_unit_beyond_the_size_of_its_record_is_not_decided
扫描重建谱系有记录照往下重建（C615）	crates/singlefs-checker/src/rebuild.rs	            PACKED_TYPE_LINEAGE if !records.is_empty() => {	            PACKED_TYPE_LINEAGE if records.is_empty() => {	-p singlefs-checker --lib -- rebuild::tests::a_lineage_record_marks_a_rolled_back_image_that_is_not_decided	rebuild::tests::a_lineage_record_marks_a_rolled_back_image_that_is_not_decided
扫描重建退役有记录照往下重建（C768）	crates/singlefs-checker/src/rebuild.rs	            PACKED_TYPE_RETIREMENT if !records.is_empty() => {	            PACKED_TYPE_RETIREMENT if records.is_empty() => {	-p singlefs-checker --lib -- rebuild::tests::a_retirement_record_marks_retired_leaf_containers_that_are_not_decided	rebuild::tests::a_retirement_record_marks_retired_leaf_containers_that_are_not_decided
扫描重建墓碑区间不核止（D18 已定项 24 第 2 条）	crates/singlefs-checker/src/rebuild.rs	                            != TOMBSTONE_WHOLE_OBJECT_RANGE_END	                            == TOMBSTONE_WHOLE_OBJECT_RANGE_START	-p singlefs-checker --lib -- rebuild::tests::a_tombstone_whose_range_is_not_the_whole_object_is_not_decided	rebuild::tests::a_tombstone_whose_range_is_not_the_whole_object_is_not_decided
扫描重建当前容器解不开当成空容器	crates/singlefs-checker/src/rebuild.rs	        .map(|view| view.records)\n        .ok_or(ScanRebuildNotDecided::CurrentPackedContainerDecodesFromNoCopy)	        .map_or(Ok(Vec::new()), |view| Ok(view.records))	-p singlefs-checker --lib -- rebuild::tests::a_current_container_without_a_decodable_copy_is_not_decided	rebuild::tests::a_current_container_without_a_decodable_copy_is_not_decided
扫描重建同一 (inode, 偏移) 两个对象照收	crates/singlefs-checker/src/rebuild.rs	            return Err(ScanRebuildNotDecided::TwoObjectsClaimOneDataUnitPosition);	            continue;	-p singlefs-checker --lib -- rebuild::tests::two_objects_at_one_data_unit_position_are_not_decided	rebuild::tests::two_objects_at_one_data_unit_position_are_not_decided
扫描重建同一 inode 号两片容器照收	crates/singlefs-checker/src/rebuild.rs	                        return Err(ScanRebuildNotDecided::InodeNumberInTwoCurrentContainers);	                        continue;	-p singlefs-checker --lib -- rebuild::tests::one_inode_number_in_two_current_containers_is_not_decided	rebuild::tests::one_inode_number_in_two_current_containers_is_not_decided
扫描重建比对前的损坏表少登记 I-1.11（四·2 口径 B①）	crates/singlefs-checker/src/walk.rs	        MAPPING_KEY_MATCHES_THE_UNIT_HEADER,\n        "遍历方向的中央映射已判损坏	        SCAN_REBUILD_AGREES_WITH_THE_WALKED_INDEX,\n        "遍历方向的中央映射已判损坏	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- a_mapping_key_whose_padding_after_the_birth_sequence_is_not_zero	a_mapping_key_whose_padding_after_the_birth_sequence_is_not_zero_reddens_only_the_mapping_key_invariant
扫描重建比对三张表恒判相同（I-1.13 的坏镜像登记进已知坏镜像表，四·2 口径 B②）	crates/singlefs-checker/src/walk.rs	    let every_table_agrees = tables.iter().all(|table| table.differences.is_empty());	    let every_table_agrees = true || tables.iter().all(|table| table.differences.is_empty());	-p singlefs-harness --test checker_known_bad_images -- the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
扫描重建码 3 的映射 key 不抄出生序号（两片叶容器各有出生序号，四·2 口径 B③）	crates/singlefs-checker/src/rebuild.rs	            key[MAPPING_KEY_TAIL_OFFSET..MAPPING_KEY_TAIL_PADDING.start].copy_from_slice(\n                &plain_header[birth_sequence..birth_sequence + birth_sequence_bytes],\n            );	            let _ = (birth_sequence, birth_sequence_bytes);	-p singlefs-harness --test checker_narrow_invariants_and_abandoned_roots -- two_leaf_containers_in_key_order_hold_every_invariant	two_leaf_containers_in_key_order_hold_every_invariant
扫描重建比对中央映射不比落点（映射条目改指一份没分配的副本，四·2 口径 B①）	crates/singlefs-checker/src/walk.rs	                if readable_rebuilt != readable_walked {	                if false && readable_rebuilt != readable_walked {	-p singlefs-harness --test checker_cross_links_malformed_nodes_and_mapping_entries -- a_mapping_entry_naming_an_unallocated_copy	a_mapping_entry_naming_an_unallocated_copy_counts_as_a_reference_and_reddens_the_allocated_bytes_invariants
````

## m4-snapshot-format-writer：crates.patch

````diff
--- a/crates/singlefs-core/src/records.rs
+++ b/crates/singlefs-core/src/records.rs
@@ -430,6 +430,18 @@
 pub const TREE_KIND_LOGGED_OPERATIONS: u16 = 12;
 /// 退役树：码 13，接在 logged_ops 之后追加（D8（核心索引结构） 已定项 9；判决 `research/prompts/m3-retire-r1-main-verification.md` 第三节第 1 条）。
 pub const TREE_KIND_RETIREMENT: u16 = 13;
+/// 只读快照的 inode 根：码 14，接在退役树之后追加（D8（核心索引结构） 已定项 9 只追加不重用；判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。一个快照一条，字段取值：根指针 = 建快照那一版活头 inode 树的根；
+/// 诞生 txg = S.txg（建快照那次独立发布的 `checkpoint_txg`，同一判决 S2）；`previous_snapshot_txg` = 捕获那一刻活头 inode 树条目里的值；
+/// 头 ID = 活头 inode 树的树 ID；flags 位 0 留给「销毁中」（同一判决 S4，第一版读者还不认它：flags 非 0 照 `TreeTableEntry::parse` 拒）。
+/// 一张树表里可以有多条（每个快照一条），与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`] 一一配对。
+pub const TREE_KIND_SNAPSHOT_INODE_ROOT: u16 = 14;
+/// 只读快照的 extent 根：码 15，接在码 14 之后追加（同上 S1）。根指针 = 建快照那一版活头 extent 树的根；头 ID = 与它配对的那条
+/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 条目的树 ID；`previous_snapshot_txg` 写 0（0 = 不适用，D5（快照 / 空间记账机制） 已定项 9）。
+pub const TREE_KIND_SNAPSHOT_EXTENT_ROOT: u16 = 15;
+/// 第一版同时活着的只读快照至多几个（一张树表里码 14 的条数上界）：用户 2026-10-13 定「10」（判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第七节岔路 1；单节点树表能装 34 个，取 10 是用户给的上界）。
+pub const SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION: usize = 10;
 
 /// 树表条目里根指针的起点：树 ID 8 + 条目长度 2 + 树的种类 2 + flags 2（`singlefs_format::TREE_TABLE_ENTRY_BYTES` 的字段表）。
 const TREE_TABLE_ENTRY_ROOT_POINTER_OFFSET_IN_BYTES: usize = 8 + 2 + 2 + 2;
--- a/crates/singlefs-core/src/recovery.rs
+++ b/crates/singlefs-core/src/recovery.rs
@@ -70,9 +70,10 @@
     mapping_key_for_data, mapping_key_for_node, parse_inode_internal_entry_or_refusal,
     parse_mapping_entry, AccountingEntry, InodeRecord, LineageRecord, PointerCarryingEntryRefusal,
     RetirementRecord, TombstoneRecord, TreeTableEntry, NLINK_OF_AN_INODE_BEING_DELETED,
-    STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING, TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST,
-    TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT, TREE_KIND_INODE, TREE_KIND_LINEAGE,
-    TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS, TREE_KIND_RETIREMENT,
+    SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, STATISTIC_INODE_WATERMARK, TREE_KIND_ACCOUNTING,
+    TREE_KIND_ALLOCATION, TREE_KIND_DEADLIST, TREE_KIND_DELETED_INODES, TREE_KIND_EXTENT,
+    TREE_KIND_INODE, TREE_KIND_LINEAGE, TREE_KIND_LIVELIST, TREE_KIND_LOGGED_OPERATIONS,
+    TREE_KIND_RETIREMENT, TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
     TREE_KIND_SPARSE_SIDE_TABLE, TREE_KIND_TOMBSTONE,
 };
 use crate::retirement_tree::RetirementContainerIndexInTree;
@@ -2322,13 +2323,40 @@
 /// 树表里同一种树出现两条时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
 pub const TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE: &str = "树表里同一种树有两条";
 
+/// 树表里码 14（快照 inode 根）与码 15（快照 extent 根）的条数不相等时，读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER: &str =
+    "树表里快照 inode 根与快照 extent 根的条数不相等";
+
+/// 树表里有一条码 15（快照 extent 根）的头 ID 不是任何一条码 14（快照 inode 根）的树 ID，或它指的那条码 14 前面已经有一条码 15 配上了：
+/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY: &str =
+    "树表里有一条快照 extent 根配不上一条只属于它的快照 inode 根";
+
+/// 树表里码 14（快照 inode 根）多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条：读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，
+/// `what` 取这一句。
+pub const TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS: &str =
+    "树表里的只读快照多于第一版允许的个数";
+
+/// 树表里有一条码 14（快照 inode 根）的头 ID 不等于同一张树表里活头 inode 树（码 2）条目的树 ID（表里没有码 2 条目时也报这一句）：
+/// 读树表的每一处都报 [`RecoveryFailure::UnitMalformed`]，`what` 取这一句。
+pub const TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE: &str =
+    "树表里有一条快照 inode 根的头 ID 不是活头 inode 树的树 ID";
+
 /// 树表的条目读不下去是哪一种（[`tree_table_entries_each_kind_at_most_once`] 交回）。封闭集合，`match` 不写通配臂。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub(crate) enum TreeTableEntriesRefusal {
     /// 一条条目解不开（条目长度、flags 未知位、预留 76 非零、根指针头部 extent 偏移非 0，`TreeTableEntry::parse`）。
     EntryMalformed,
-    /// 同一种树有两条。
+    /// 同一种树有两条（码 14、15 两种快照条目除外）。
     OneKindOfTreeAppearsTwice,
+    /// 码 14 与码 15 的条数不相等。
+    SnapshotInodeAndExtentRootEntryCountsDiffer,
+    /// 一条码 15 的头 ID 不是任何一条码 14 的树 ID，或它指的那条码 14 已经被前面一条码 15 配上了。
+    SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
+    /// 码 14 多于 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条。
+    MoreSnapshotsThanTheFirstVersionAllows,
+    /// 一条码 14 的头 ID 不等于这张树表里码 2 条目的树 ID（或表里没有码 2 条目）。
+    SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree,
 }
 
 impl From<TreeTableEntriesRefusal> for RecoveryFailure {
@@ -2340,17 +2368,110 @@
             TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => RecoveryFailure::UnitMalformed {
                 what: TREE_TABLE_CARRIES_ONE_KIND_OF_TREE_TWICE,
             },
+            TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER,
+                }
+            }
+            TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY,
+                }
+            }
+            TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS,
+                }
+            }
+            TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {
+                RecoveryFailure::UnitMalformed {
+                    what: TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE,
+                }
+            }
         }
     }
 }
 
+/// 一个只读快照在树表里的那一对条目：码 14（快照 inode 根）与头 ID 指着它的那条码 15（快照 extent 根），字段取值见
+/// [`TREE_KIND_SNAPSHOT_INODE_ROOT`] 与 [`TREE_KIND_SNAPSHOT_EXTENT_ROOT`]。重建只把它们读出来，不解它们指着的根。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct SnapshotTreeTableEntries {
+    pub inode_root: TreeTableEntry,
+    pub extent_root: TreeTableEntry,
+}
+
+/// 树表里的快照条目（码 14、15）配成对，按码 14 的盘上次序（即树 ID 升序，I-9.16 ①）交回，并判四样（判决
+/// `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1 与第七节岔路 1）：两种条数相等；每条码 15 的头 ID 恰是一条码 14 的树 ID、
+/// 且那条码 14 只被这一条码 15 配上（两样合起来是一一配对）；码 14 至多 [`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`] 条；每条码 14 的头 ID
+/// 等于这张树表里码 2（活头 inode 树）条目的树 ID。没有快照条目的树表交空表，一样都不判。
+///
+/// # Errors
+/// 按上面的次序第一样不成立的那一个：`SnapshotInodeAndExtentRootEntryCountsDiffer`、`SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry`、
+/// `MoreSnapshotsThanTheFirstVersionAllows`、`SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree`。
+pub(crate) fn snapshot_tree_table_entries_paired_one_to_one(
+    tree_table_entries: &[TreeTableEntry],
+) -> Result<Vec<SnapshotTreeTableEntries>, TreeTableEntriesRefusal> {
+    let snapshot_inode_root_entries: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT)
+        .collect();
+    let snapshot_extent_root_entries: Vec<&TreeTableEntry> = tree_table_entries
+        .iter()
+        .filter(|entry| entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT)
+        .collect();
+    if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {
+        return Err(TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer);
+    }
+    // 每条码 14 配上的那条码 15，下标与 `snapshot_inode_root_entries` 一一对应。
+    let mut extent_root_paired_with: Vec<Option<TreeTableEntry>> =
+        vec![None; snapshot_inode_root_entries.len()];
+    for extent_root_entry in &snapshot_extent_root_entries {
+        // 头 ID 指着它、又还没被前面哪条码 15 配上的那条码 14。
+        let paired_inode_root_position = snapshot_inode_root_entries
+            .iter()
+            .zip(&extent_root_paired_with)
+            .position(|(inode_root_entry, extent_root_already_paired)| {
+                inode_root_entry.tree.0 == extent_root_entry.head_identifier
+                    && extent_root_already_paired.is_none()
+            })
+            .ok_or(
+                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,
+            )?;
+        extent_root_paired_with[paired_inode_root_position] = Some(**extent_root_entry);
+    }
+    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {
+        return Err(TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows);
+    }
+    let live_inode_tree = tree_table_entries
+        .iter()
+        .find(|entry| entry.kind == TREE_KIND_INODE)
+        .map(|entry| entry.tree);
+    if snapshot_inode_root_entries.iter().any(|inode_root_entry| {
+        Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree
+    }) {
+        return Err(TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree);
+    }
+    Ok(snapshot_inode_root_entries
+        .into_iter()
+        .zip(extent_root_paired_with)
+        .map(|(inode_root_entry, extent_root_entry)| SnapshotTreeTableEntries {
+            inode_root: *inode_root_entry,
+            extent_root: extent_root_entry.expect(
+                "两种条数相等、每条码 15 配上一条还没配过的码 14（上面两判）：鸽巢下每条码 14 都配上了",
+            ),
+        })
+        .collect())
+}
+
 /// 树表单元的条目逐条解开，并判每一种树至多一条（审阅第 27 条）。写者每种树只写一条；同一种出现两条时取哪一条条款没有写，
 /// 取首条、取末条、取首条非空各有一个读者照着读过，同一张树表在几处读出不同的树 ⇒ 判损坏，一条都不取。
+/// 码 14、15 两种快照条目例外：每个快照一对，可以有多条，改判 [`snapshot_tree_table_entries_paired_one_to_one`] 那四样。
 /// 读树表取条目的每一处都经这一处：[`rebuild_version`]、[`allocation_records_under_root`]、[`walk_to_file`]、
 /// [`user_visible_tree_root_pointers`]、`mounted_read::open_pool_for_read`。
 ///
 /// # Errors
-/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`。
+/// 按条目次序第一处读不下去的：解不开 ⇒ `EntryMalformed`；它的种类（码 14、15 除外）前面已经有一条 ⇒ `OneKindOfTreeAppearsTwice`；
+/// 条目都过了之后快照条目配不成对 ⇒ [`snapshot_tree_table_entries_paired_one_to_one`] 的那四个成员。
 pub(crate) fn tree_table_entries_each_kind_at_most_once(
     entry_bytes_of_the_tree_table: &[Vec<u8>],
 ) -> Result<Vec<TreeTableEntry>, TreeTableEntriesRefusal> {
@@ -2359,11 +2480,18 @@
     for entry_bytes in entry_bytes_of_the_tree_table {
         let entry =
             TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;
+        let is_a_snapshot_entry = entry.kind == TREE_KIND_SNAPSHOT_INODE_ROOT
+            || entry.kind == TREE_KIND_SNAPSHOT_EXTENT_ROOT;
+        if is_a_snapshot_entry {
+            entries.push(entry);
+            continue;
+        }
         if !kinds_seen.insert(entry.kind) {
             return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);
         }
         entries.push(entry);
     }
+    snapshot_tree_table_entries_paired_one_to_one(&entries)?;
     Ok(entries)
 }
 
@@ -2518,12 +2646,41 @@
 /// 记账行里没有 inode 号水位那一行 ⇒ `InodeNumberWatermarkRowMissingFromTheAccountingTree`。
 ///
 /// 单元区从 `unit_area_start` 起（择到的系统配置按环长现算的那个）：这一版的分配记录落不落在单元区里按它判。
+/// 树表里的只读快照条目（码 14、15）由 [`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`] 交出，这一个丢掉它们。
 pub fn rebuild_version_in_the_unit_area_starting_at(
     reader: &dyn PoolReader,
     unit_area_start: UnitAreaStart,
     root: &RootRecord,
     record_standing_for_root: Option<JournalRecord>,
 ) -> Result<RebuiltVersion, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+        reader,
+        unit_area_start,
+        root,
+        record_standing_for_root,
+    )
+    .map(|rebuilt| rebuilt.version)
+}
+
+/// 重建出来的一版，连同它树表里的只读快照条目（判决 `research/prompts/m4-snapshot-r1-main-verification.md` 第三节 S1）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RebuiltVersionWithItsSnapshotEntries {
+    pub version: RebuiltVersion,
+    /// 码 14 与配对的码 15，按码 14 的树 ID 升序；树表里没有快照条目（含树表 0 条）时为空。只读出条目，不解它们指着的根。
+    pub snapshot_entries: Vec<SnapshotTreeTableEntries>,
+}
+
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]（判全、报错都与它相同），另把树表里码 14、15 的条目配成对交出来
+/// （[`snapshot_tree_table_entries_paired_one_to_one`]，配不成对的树表在 [`tree_table_entries_each_kind_at_most_once`] 那一步就拒了）。
+///
+/// # Errors
+/// 同 [`rebuild_version_in_the_unit_area_starting_at`]。
+pub fn rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+    reader: &dyn PoolReader,
+    unit_area_start: UnitAreaStart,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
     let node_bytes = usize::try_from(NODE_BYTES).expect("16384");
     let data_bytes = usize::try_from(DATA_UNIT_BYTES).expect("32768");
     // 树表 0 条的一版也要判：可写挂载接着它发的零单元发布照抄根记录里的实例表与树表指针。
@@ -2537,13 +2694,20 @@
         })?;
     tree_table_key_width_is_the_tree_tables(&tree_table)?;
     if tree_table.entries.is_empty() {
-        return Ok(RebuiltVersion::WithoutFile);
+        return Ok(RebuiltVersionWithItsSnapshotEntries {
+            version: RebuiltVersion::WithoutFile,
+            snapshot_entries: Vec::new(),
+        });
     }
     let record =
         record_standing_for_root.ok_or(RebuildVersionFailure::NoRecordStandingForFileVersion)?;
     let record_bytes = record.to_bytes();
     let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)
         .map_err(RecoveryFailure::from)?;
+    let snapshot_entries = snapshot_tree_table_entries_paired_one_to_one(&tree_table_entries)
+        .expect(
+            "tree_table_entries_each_kind_at_most_once 交回之前对同一串条目判过快照条目一一配对",
+        );
     // 可写挂载接着这一版发布，写者按种类装树表条目、断言树 ID 升序：盘上的次序与种类在这里先判（I-9.16），
     // 走得到那条断言的镜像在任何写之前拒。
     tree_table_entries_ascend_by_tree_identifier_and_in_the_issuing_order(&tree_table_entries)?;
@@ -3252,7 +3416,10 @@
             transaction_number_on_the_record_standing_for_this_root,
     };
     pointers_of_the_rebuilt_version_ascend_by_device(&rebuilt)?;
-    Ok(RebuiltVersion::of_a_version_of_trees(rebuilt))
+    Ok(RebuiltVersionWithItsSnapshotEntries {
+        version: RebuiltVersion::of_a_version_of_trees(rebuilt),
+        snapshot_entries,
+    })
 }
 
 /// 一条根的树表里用户可见的两棵树（inode 树、extent 树）的根指针，取树表条目里那 86 字节的盘上原样；树表里没有那棵树的条目时 `None`。
@@ -5531,6 +5698,24 @@
         reader,
         unit_area_start_of_the_pool(reader)?,
         root,
+        record_standing_for_root,
+    )
+}
+
+/// 同 [`rebuild_version`]（单元区起点先择一次系统配置现算），另交出树表里的只读快照条目
+/// （[`rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at`]）。给手里没有择好的系统配置的调用方（用例）。
+///
+/// # Errors
+/// 同 [`rebuild_version`]。
+pub fn rebuild_version_with_its_snapshot_entries(
+    reader: &dyn PoolReader,
+    root: &RootRecord,
+    record_standing_for_root: Option<JournalRecord>,
+) -> Result<RebuiltVersionWithItsSnapshotEntries, RebuildVersionFailure> {
+    rebuild_version_with_its_snapshot_entries_in_the_unit_area_starting_at(
+        reader,
+        unit_area_start_of_the_pool(reader)?,
+        root,
         record_standing_for_root,
     )
 }
--- a/crates/singlefs-core/src/mounted_read.rs
+++ b/crates/singlefs-core/src/mounted_read.rs
@@ -434,7 +434,11 @@
             TreeTableEntriesRefusal::EntryMalformed => {
                 OpenPoolForReadFailure::TreeTableEntryMalformed
             }
-            TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice => {
+            TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice
+            | TreeTableEntriesRefusal::SnapshotInodeAndExtentRootEntryCountsDiffer
+            | TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry
+            | TreeTableEntriesRefusal::MoreSnapshotsThanTheFirstVersionAllows
+            | TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {
                 OpenPoolForReadFailure::Walk(RecoveryFailure::from(refusal))
             }
         })?;
diff --git a/crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs b/crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs
@@ -0,0 +1,427 @@
+//! 树表里的只读快照条目（码 14 快照 inode 根、码 15 快照 extent 根，判决 `research/prompts/m4-snapshot-r1-main-verification.md`
+//! 第三节 S1 与第七节岔路 1）在核心层读树表那一步怎么判：一个快照一对，两种可以各有多条；两种条数相等、每条码 15 的头 ID 恰是一条
+//! 只属于它的码 14 的树 ID、码 14 至多 `SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION` 条、码 14 的头 ID 是活头 inode 树的树 ID，
+//! 四样任一样不成立，读树表的五个读者都判树表损坏；都成立时从盘上重建那一版把它们配成对交出来，不解它们指着的根。
+//! 树表是手造的：第一个文件那一版（txg 3）的十二条条目之后按树 ID 升序接上快照条目（树 ID 从那条根的水位起连发，根的水位跟着抬），
+//! 树表单元重建、写回它两条位置条目指的槽；读者拿换过树表指针校验和与水位的那条根去读。盘都是内存稀疏盘。
+
+use singlefs_core::address::{DeviceIdentity, TreeIdentifier};
+use singlefs_core::allocator::PoolAllocator;
+use singlefs_core::block_device::PhysicalBlockSizeInBytes;
+use singlefs_core::checksum::crc32_castagnoli;
+use singlefs_core::journal::JournalRecord;
+use singlefs_core::make_filesystem::{
+    allocator_after_make_filesystem, make_filesystem, MakeFilesystemParameters,
+};
+use singlefs_core::mounted_read::{open_pool_for_read, OpenPoolForReadFailure};
+use singlefs_core::records::{
+    TreeTableEntry, SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION, TREE_KIND_EXTENT, TREE_KIND_INODE,
+    TREE_KIND_SNAPSHOT_EXTENT_ROOT, TREE_KIND_SNAPSHOT_INODE_ROOT,
+};
+use singlefs_core::recovery::{
+    allocation_records_under_root, rebuild_version, rebuild_version_with_its_snapshot_entries,
+    user_visible_tree_root_pointers, walk_to_file, PoolReader, RebuildVersionFailure,
+    RecoveryFailure, SnapshotTreeTableEntries,
+    TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS,
+    TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY,
+    TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER,
+    TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE,
+};
+use singlefs_core::root_record::RootRecord;
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_core::system_configuration::SystemImmutableSizes;
+use singlefs_core::transaction::{
+    acquire_instance, publish_first_file, warm_up, FirstFile, PoolWriter, TransactionOutput,
+};
+use singlefs_core::unit::{build_index_node, parse_index_node};
+use singlefs_format::{JOURNAL_RING_DEFAULT_BYTES, NODE_BYTES};
+use singlefs_harness::memory_pool::{MemoryPool, SparseBlockDevice};
+
+const DEVICE_BYTES: u64 = 4 << 30;
+const FILESYSTEM_IDENTIFIER: [u8; 16] = *b"singlefs-snap-s1";
+const WRITE_TIME_SECONDS: u64 = 1_788_100_000;
+/// 内存稀疏盘按 512 字节扇区存：物理块宽 512。
+const SECTOR_PHYSICAL_BLOCK_SIZE: PhysicalBlockSizeInBytes = PhysicalBlockSizeInBytes(512);
+
+fn node_bytes() -> usize {
+    usize::try_from(NODE_BYTES).expect("16384")
+}
+
+/// 两块内存盘上 mkfs → 取号 → 暖机 → 第一个文件（txg 3）之后的池与第一个文件那次发布的输出。
+fn pool_after_the_first_file() -> (MemoryPool, TransactionOutput) {
+    let parameters = MakeFilesystemParameters {
+        filesystem_identifier: FILESYSTEM_IDENTIFIER,
+        region_devices: [DeviceIdentity(0), DeviceIdentity(1), DeviceIdentity(0)],
+        geometry: SystemImmutableSizes {
+            physical_block_size: 512,
+            minimum_input_output_bytes: 512,
+            fixed_structure_slot_spacing: SystemImmutableSizes::slot_spacing_for(512),
+            journal_ring_bytes: JOURNAL_RING_DEFAULT_BYTES,
+            root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+        },
+    };
+    let mut devices: Vec<(DeviceIdentity, SparseBlockDevice)> =
+        [DeviceIdentity(0), DeviceIdentity(1)]
+            .into_iter()
+            .map(|identity| {
+                (
+                    identity,
+                    SparseBlockDevice::new(DEVICE_BYTES, SECTOR_PHYSICAL_BLOCK_SIZE),
+                )
+            })
+            .collect();
+    let genesis = make_filesystem(&parameters, &mut devices).expect("默认几何上 mkfs");
+    let mut allocator: PoolAllocator =
+        allocator_after_make_filesystem(&parameters, &devices, &genesis);
+    let content: Vec<u8> = (0..3000u32)
+        .map(|index| u8::try_from((index * 137 + 3) % 251).expect("小于 256"))
+        .collect();
+    let first = {
+        let mut writer = PoolWriter::new(&parameters, devices.as_mut_slice());
+        let instance = acquire_instance(&mut writer).expect("取号 1");
+        let warmed = warm_up(&mut writer, &genesis.root, instance).expect("暖机");
+        publish_first_file(
+            &mut writer,
+            &mut allocator,
+            warmed.roots.last().expect("暖机写了两条根"),
+            FirstFile {
+                content: &content,
+                write_time_seconds: WRITE_TIME_SECONDS,
+            },
+            instance,
+            &warmed.last_record_bytes,
+        )
+        .expect("第一个文件")
+    };
+    let image = MemoryPool {
+        devices: devices
+            .iter()
+            .map(|(identity, device)| (*identity, device.image.clone()))
+            .collect(),
+        device_size_in_bytes: DEVICE_BYTES,
+    };
+    (image, first)
+}
+
+/// 一条根指着的树表单元，按它第一条位置条目读回来。
+fn tree_table_unit_of(image: &MemoryPool, root: &RootRecord) -> Vec<u8> {
+    let location = root.tree_table.locations[0];
+    PoolReader::read(
+        image,
+        location.device,
+        location.slot.to_device_offset(),
+        node_bytes(),
+    )
+    .expect("树表单元读得到")
+}
+
+/// 第一个文件那一版树表里的十二条条目（盘上原样、按盘上次序）。
+fn entries_of_the_first_file_version(image: &MemoryPool, root: &RootRecord) -> Vec<Vec<u8>> {
+    parse_index_node(&tree_table_unit_of(image, root))
+        .expect("树表单元解得开")
+        .entries
+}
+
+fn entry_of_kind(entries: &[Vec<u8>], kind: u16) -> TreeTableEntry {
+    entries
+        .iter()
+        .map(|entry_bytes| TreeTableEntry::parse(entry_bytes).expect("写者写的条目解得开"))
+        .find(|entry| entry.kind == kind)
+        .expect("第一个文件那一版的树表里这种树恰好一条")
+}
+
+/// 用 `entries` 照原树表单元的头重建一个树表单元（key 区间取首条与末条的树 ID），写回它两条位置条目指的槽（两块盘各一份），
+/// 交回一条树表指针两条校验和换过、树 ID 水位换成 `tree_identifier_watermark`、别的字段与 `root` 相同的根。
+fn root_whose_tree_table_carries(
+    image: &mut MemoryPool,
+    root: &RootRecord,
+    entries: &[Vec<u8>],
+    tree_identifier_watermark: u64,
+) -> RootRecord {
+    let header = parse_index_node(&tree_table_unit_of(image, root)).expect("树表单元解得开");
+    let key_of = |entry_bytes: &Vec<u8>| entry_bytes[..8].to_vec();
+    let rebuilt = build_index_node(
+        header.tree,
+        header.level,
+        header.key_width,
+        &key_of(entries.first().expect("树表有条目")),
+        &key_of(entries.last().expect("树表有条目")),
+        header.birth_txg,
+        &FILESYSTEM_IDENTIFIER,
+        header.instance,
+        header.birth_sequence,
+        u16::try_from(header.entry_width).expect("条目宽 200"),
+        entries,
+    );
+    let checksum = crc32_castagnoli(&rebuilt);
+    let mut rewritten_root = *root;
+    for location in &mut rewritten_root.tree_table.locations {
+        image
+            .devices
+            .get_mut(&location.device)
+            .expect("池里有这块盘")
+            .write(location.slot.to_device_offset(), &rebuilt);
+        location.unit_checksum = checksum;
+    }
+    rewritten_root.tree_identifier_watermark = tree_identifier_watermark;
+    rewritten_root
+}
+
+/// 一个快照照判决 S1 的字段取值造的那一对条目：码 14 抄活头 inode 树条目（根指针、`previous_snapshot_txg` 都是捕获那一刻活头的），
+/// 树 ID 换成 `snapshot_inode_root_tree`、诞生 txg 换成 S.txg（这里取那条根的 txg）、头 ID 是活头 inode 树的树 ID；
+/// 码 15 抄活头 extent 树条目（`previous_snapshot_txg` 是 0），树 ID 是码 14 的下一个号、头 ID 是码 14 的树 ID。
+fn snapshot_entry_pair(
+    live_inode_tree_entry: TreeTableEntry,
+    live_extent_tree_entry: TreeTableEntry,
+    snapshot_inode_root_tree: u64,
+    snapshot_txg: u64,
+) -> SnapshotTreeTableEntries {
+    let mut inode_root = live_inode_tree_entry;
+    inode_root.kind = TREE_KIND_SNAPSHOT_INODE_ROOT;
+    inode_root.tree = TreeIdentifier(snapshot_inode_root_tree);
+    inode_root.birth_txg.0 = snapshot_txg;
+    inode_root.head_identifier = live_inode_tree_entry.tree.0;
+    let mut extent_root = live_extent_tree_entry;
+    extent_root.kind = TREE_KIND_SNAPSHOT_EXTENT_ROOT;
+    extent_root.tree = TreeIdentifier(snapshot_inode_root_tree + 1);
+    extent_root.birth_txg.0 = snapshot_txg;
+    extent_root.head_identifier = snapshot_inode_root_tree;
+    SnapshotTreeTableEntries {
+        inode_root,
+        extent_root,
+    }
+}
+
+/// 第一个文件那一版加 `pair_count` 个快照：快照的树 ID 从那条根的水位起每个快照连发两个（判决 S2「新树号从水位连发两个」）。
+struct VersionWithSnapshots {
+    image: MemoryPool,
+    first: TransactionOutput,
+    original_entries: Vec<Vec<u8>>,
+    pairs: Vec<SnapshotTreeTableEntries>,
+}
+
+fn version_with_snapshot_pairs(pair_count: u64) -> VersionWithSnapshots {
+    let (image, first) = pool_after_the_first_file();
+    let original_entries = entries_of_the_first_file_version(&image, &first.root);
+    let live_inode_tree_entry = entry_of_kind(&original_entries, TREE_KIND_INODE);
+    let live_extent_tree_entry = entry_of_kind(&original_entries, TREE_KIND_EXTENT);
+    let watermark = first.root.tree_identifier_watermark;
+    let pairs = (0..pair_count)
+        .map(|snapshot_position| {
+            snapshot_entry_pair(
+                live_inode_tree_entry,
+                live_extent_tree_entry,
+                watermark + 2 * snapshot_position,
+                first.root.checkpoint_txg.0,
+            )
+        })
+        .collect();
+    VersionWithSnapshots {
+        image,
+        first,
+        original_entries,
+        pairs,
+    }
+}
+
+impl VersionWithSnapshots {
+    /// 十二条原样条目之后接上 `snapshot_entries`（调用方按树 ID 升序给），树 ID 水位抬到最大那个快照树 ID 之上。
+    fn root_carrying(&mut self, snapshot_entries: &[TreeTableEntry]) -> RootRecord {
+        let mut entries = self.original_entries.clone();
+        entries.extend(snapshot_entries.iter().map(TreeTableEntry::to_bytes));
+        let watermark = snapshot_entries
+            .iter()
+            .map(|entry| entry.tree.0 + 1)
+            .max()
+            .unwrap_or(self.first.root.tree_identifier_watermark);
+        let root = self.first.root;
+        root_whose_tree_table_carries(&mut self.image, &root, &entries, watermark)
+    }
+
+    fn entries_of_every_pair(&self) -> Vec<TreeTableEntry> {
+        self.pairs
+            .iter()
+            .flat_map(|pair| [pair.inode_root, pair.extent_root])
+            .collect()
+    }
+}
+
+/// 读树表的五个读者各自读一条根的结局（读下去了是 `Ok(())`）。
+#[derive(Debug, PartialEq, Eq)]
+struct TreeTableReaderOutcomes {
+    rebuild_version: Result<(), RebuildVersionFailure>,
+    allocation_records_under_root: Result<(), RecoveryFailure>,
+    walk_to_file: Result<(), RecoveryFailure>,
+    user_visible_tree_root_pointers: Result<(), RecoveryFailure>,
+    open_pool_for_read: Result<(), OpenPoolForReadFailure>,
+}
+
+fn outcomes_of_every_tree_table_reader(
+    image: &MemoryPool,
+    root: &RootRecord,
+    record_standing_for_root: &JournalRecord,
+) -> TreeTableReaderOutcomes {
+    let mut mapping_fallbacks = 0;
+    TreeTableReaderOutcomes {
+        rebuild_version: rebuild_version(image, root, Some(record_standing_for_root.clone()))
+            .map(|_rebuilt| ()),
+        allocation_records_under_root: allocation_records_under_root(image, root)
+            .map(|_records| ()),
+        walk_to_file: walk_to_file(image, root, &mut mapping_fallbacks).map(|_content| ()),
+        user_visible_tree_root_pointers: user_visible_tree_root_pointers(image, root)
+            .map(|_pointers| ()),
+        open_pool_for_read: open_pool_for_read(image, root).map(|_mounted| ()),
+    }
+}
+
+fn every_reader_refuses_the_tree_table_with(what: &'static str) -> TreeTableReaderOutcomes {
+    let damaged = RecoveryFailure::UnitMalformed { what };
+    TreeTableReaderOutcomes {
+        rebuild_version: Err(RebuildVersionFailure::Walk(damaged.clone())),
+        allocation_records_under_root: Err(damaged.clone()),
+        walk_to_file: Err(damaged.clone()),
+        user_visible_tree_root_pointers: Err(damaged.clone()),
+        open_pool_for_read: Err(OpenPoolForReadFailure::Walk(damaged)),
+    }
+}
+
+/// 两个快照（两对码 14 / 码 15）：同一种出现两条不再判「同一种树两条」，从盘上重建那一版照常建得出，快照条目按码 14 的树 ID 升序配成对交出来，
+/// 逐字段与写进去的相等；重建出来的那一版与不带快照条目那一份的读法（`rebuild_version`）交回的相同。
+/// 对照：第一个文件那一版原样（没有快照条目）交空表。
+#[test]
+fn a_tree_table_carrying_two_snapshot_pairs_is_rebuilt_and_hands_back_both_pairs() {
+    let mut version = version_with_snapshot_pairs(2);
+    let record = version.first.record.clone();
+    let unchanged = rebuild_version_with_its_snapshot_entries(
+        &version.image,
+        &version.first.root,
+        Some(record.clone()),
+    )
+    .expect("第一个文件那一版建得出");
+    assert_eq!(
+        unchanged.snapshot_entries,
+        Vec::new(),
+        "对照：没有快照条目的树表交空表"
+    );
+
+    let snapshot_entries = version.entries_of_every_pair();
+    let root = version.root_carrying(&snapshot_entries);
+    let rebuilt =
+        rebuild_version_with_its_snapshot_entries(&version.image, &root, Some(record.clone()))
+            .expect("两对快照条目的树表：重建照常建得出");
+    assert_eq!(
+        rebuilt.snapshot_entries, version.pairs,
+        "两对快照条目按码 14 的树 ID 升序配成对交出来，逐字段与写进去的相等"
+    );
+    assert_eq!(
+        Ok(rebuilt.version),
+        rebuild_version(&version.image, &root, Some(record)),
+        "重建出来的那一版与丢掉快照条目的那一份读法交回的相同"
+    );
+}
+
+/// 两对之外多一条码 15（头 ID 指着第一个快照的码 14）：码 14 两条、码 15 三条，条数不相等，五个读者都判树表损坏。
+#[test]
+fn a_tree_table_with_one_more_snapshot_extent_root_than_snapshot_inode_roots_is_refused_by_every_reader(
+) {
+    let mut version = version_with_snapshot_pairs(2);
+    let record = version.first.record.clone();
+    let mut snapshot_entries = version.entries_of_every_pair();
+    let mut lone_extent_root = version.pairs[0].extent_root;
+    lone_extent_root.tree = TreeIdentifier(version.pairs[1].extent_root.tree.0 + 1);
+    snapshot_entries.push(lone_extent_root);
+    let root = version.root_carrying(&snapshot_entries);
+    assert_eq!(
+        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
+        every_reader_refuses_the_tree_table_with(
+            TREE_TABLE_SNAPSHOT_INODE_AND_EXTENT_ROOT_ENTRY_COUNTS_DIFFER
+        ),
+        "码 14 两条、码 15 三条：五个读者都判树表损坏"
+    );
+}
+
+/// 两对，第二个快照的码 15 头 ID 改成一个不是任何码 14 的号（活头 inode 树的树 ID）：条数相等、这一条码 15 配不上码 14，
+/// 五个读者都判树表损坏。
+#[test]
+fn a_snapshot_extent_root_whose_head_is_not_any_snapshot_inode_root_is_refused_by_every_reader() {
+    let mut version = version_with_snapshot_pairs(2);
+    let record = version.first.record.clone();
+    let live_inode_tree = version.pairs[1].inode_root.head_identifier;
+    version.pairs[1].extent_root.head_identifier = live_inode_tree;
+    let snapshot_entries = version.entries_of_every_pair();
+    let root = version.root_carrying(&snapshot_entries);
+    assert_eq!(
+        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
+        every_reader_refuses_the_tree_table_with(
+            TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY
+        ),
+        "一条码 15 的头 ID 不是任何码 14 的树 ID：五个读者都判树表损坏"
+    );
+}
+
+/// 两对，两条码 15 的头 ID 都指第一个快照的码 14：条数相等、每条码 15 都指得到一条码 14，但第二个快照的码 14 没有码 15、
+/// 第一个的有两条，五个读者都判树表损坏（与「指不到」报同一个）。
+#[test]
+fn two_snapshot_extent_roots_claiming_the_same_snapshot_inode_root_are_refused_by_every_reader() {
+    let mut version = version_with_snapshot_pairs(2);
+    let record = version.first.record.clone();
+    version.pairs[1].extent_root.head_identifier = version.pairs[0].inode_root.tree.0;
+    let snapshot_entries = version.entries_of_every_pair();
+    let root = version.root_carrying(&snapshot_entries);
+    assert_eq!(
+        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
+        every_reader_refuses_the_tree_table_with(
+            TREE_TABLE_SNAPSHOT_EXTENT_ROOT_ENTRY_WITHOUT_ITS_OWN_SNAPSHOT_INODE_ROOT_ENTRY
+        ),
+        "两条码 15 配同一条码 14：五个读者都判树表损坏"
+    );
+}
+
+/// 第 11 个快照：`SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION`（10）对之外再多一对，五个读者都判树表损坏。
+/// 边界：恰好 10 对时重建照常建得出、交回 10 对。
+#[test]
+fn an_eleventh_snapshot_is_refused_by_every_reader_while_ten_are_rebuilt() {
+    let pairs_allowed = u64::try_from(SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION).expect("10");
+    let mut version = version_with_snapshot_pairs(pairs_allowed + 1);
+    let record = version.first.record.clone();
+    let every_snapshot_entry = version.entries_of_every_pair();
+    let ten_pairs_of_entries = &every_snapshot_entry[..2 * SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION];
+    let root_with_ten = version.root_carrying(ten_pairs_of_entries);
+    let rebuilt_with_ten = rebuild_version_with_its_snapshot_entries(
+        &version.image,
+        &root_with_ten,
+        Some(record.clone()),
+    )
+    .expect("恰好 10 对：重建照常建得出");
+    assert_eq!(
+        rebuilt_with_ten.snapshot_entries,
+        version.pairs[..SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION].to_vec(),
+        "恰好 10 对：10 对都交出来"
+    );
+
+    let root_with_eleven = version.root_carrying(&every_snapshot_entry);
+    assert_eq!(
+        outcomes_of_every_tree_table_reader(&version.image, &root_with_eleven, &record),
+        every_reader_refuses_the_tree_table_with(
+            TREE_TABLE_CARRIES_MORE_SNAPSHOTS_THAN_THE_FIRST_VERSION_ALLOWS
+        ),
+        "11 对：五个读者都判树表损坏"
+    );
+}
+
+/// 两对，第二个快照的码 14 头 ID 改成第一个快照码 14 的树 ID（不是活头 inode 树的树 ID）：配对照过，五个读者都判树表损坏。
+#[test]
+fn a_snapshot_inode_root_whose_head_is_not_the_live_inode_tree_is_refused_by_every_reader() {
+    let mut version = version_with_snapshot_pairs(2);
+    let record = version.first.record.clone();
+    version.pairs[1].inode_root.head_identifier = version.pairs[0].inode_root.tree.0;
+    let snapshot_entries = version.entries_of_every_pair();
+    let root = version.root_carrying(&snapshot_entries);
+    assert_eq!(
+        outcomes_of_every_tree_table_reader(&version.image, &root, &record),
+        every_reader_refuses_the_tree_table_with(
+            TREE_TABLE_SNAPSHOT_INODE_ROOT_ENTRY_HEAD_IS_NOT_THE_LIVE_INODE_TREE
+        ),
+        "码 14 的头 ID 不是活头 inode 树的树 ID：五个读者都判树表损坏"
+    );
+}
````

## m4-snapshot-format-writer：mutations-append.tsv

````tsv
m4 快照格式 S1：码 14、15 照旧判同一种树两条（两对快照条目的树表被拒）	crates/singlefs-core/src/recovery.rs	        if is_a_snapshot_entry {	        if is_a_snapshot_entry && false {	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	a_tree_table_carrying_two_snapshot_pairs_is_rebuilt_and_hands_back_both_pairs
m4 快照格式 S1：重建不交快照条目（配好的对丢掉、交空表）	crates/singlefs-core/src/recovery.rs	        version: RebuiltVersion::of_a_version_of_trees(rebuilt),\n        snapshot_entries,\n    })	        version: RebuiltVersion::of_a_version_of_trees(rebuilt),\n        snapshot_entries: snapshot_entries.into_iter().take(0).collect(),\n    })	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	a_tree_table_carrying_two_snapshot_pairs_is_rebuilt_and_hands_back_both_pairs
m4 快照格式 S1：不判码 14 与码 15 条数相等	crates/singlefs-core/src/recovery.rs	    if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {	    if false && snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	a_tree_table_with_one_more_snapshot_extent_root_than_snapshot_inode_roots_is_refused_by_every_reader
m4 快照格式 S1：放过不配对的码 15（头 ID 指不到码 14 时补配给还空着的那条）	crates/singlefs-core/src/recovery.rs	            .ok_or(\n                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,\n            )?;	            .or_else(|| extent_root_paired_with.iter().position(Option::is_none))\n            .ok_or(\n                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,\n            )?;	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	a_snapshot_extent_root_whose_head_is_not_any_snapshot_inode_root_is_refused_by_every_reader
m4 快照格式 S1：放过配同一条码 14 的第二条码 15（补配给还空着的那条）	crates/singlefs-core/src/recovery.rs	            .ok_or(\n                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,\n            )?;	            .or_else(|| extent_root_paired_with.iter().position(Option::is_none))\n            .ok_or(\n                TreeTableEntriesRefusal::SnapshotExtentRootEntryWithoutItsOwnSnapshotInodeRootEntry,\n            )?;	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	two_snapshot_extent_roots_claiming_the_same_snapshot_inode_root_are_refused_by_every_reader
m4 快照格式 岔路 1：放过第 11 个快照	crates/singlefs-core/src/recovery.rs	    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {	    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION + 1 {	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	an_eleventh_snapshot_is_refused_by_every_reader_while_ten_are_rebuilt
m4 快照格式 岔路 1：第 10 个快照也拒	crates/singlefs-core/src/recovery.rs	    if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {	    if snapshot_inode_root_entries.len() >= SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	an_eleventh_snapshot_is_refused_by_every_reader_while_ten_are_rebuilt
m4 快照格式 S1：不判码 14 的头 ID 是活头 inode 树的树 ID	crates/singlefs-core/src/recovery.rs	        Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree	        false && Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree	-p singlefs-harness --test tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds	a_snapshot_inode_root_whose_head_is_not_the_live_inode_tree_is_refused_by_every_reader
````

## m4-gc-plan-writer：crates.patch

````diff
diff --git a/crates/singlefs-core/src/compaction_plan.rs b/crates/singlefs-core/src/compaction_plan.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-core/src/compaction_plan.rs
@@ -0,0 +1,142 @@
+//! 整理的选段与计划（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2）：三层里的后两层，都是纯函数——
+//! 不读设备、不发写、不动分配器。`choose_region` 按记录挑一个 R，`crate::region_scan::scan_region` 扫它，`plan_batch` 从扫描结果里
+//! 取下一批要搬的（映射 key, 源落点）。**目的落点不在计划里**：由那一批发布时的分配器按 D3（空间分配） 已定项 8 现取
+//! （判决 G2，攻方 H-G2-1：缓存目的落点的臂撞上映射重复）。
+//!
+//! 判据显式（`.claude/rules/fs-design.md` 五条硬要求第 1 条）：第一版只有 greedy 一种（[`RegionChoiceCriterion::Greedy`]），
+//! 打分是 [`greedy_score`]；cost-benefit 与轮转留给 E10（老化：结构抗老化 vs 事后整理） 的臂。
+
+use std::collections::BTreeSet;
+use std::num::NonZeroUsize;
+
+use singlefs_format::CLUSTER_SEGMENT_SLOTS;
+
+use crate::address::{DeviceIdentity, SlotNumber};
+use crate::allocator::{AllocationRecord, Placement, UnitAreaStart};
+use crate::logged_operations::CompactionRegion;
+use crate::region_scan::RegionScan;
+
+/// 选段判据。封闭集合：加一种判据就加一个成员（判决 G2 的 S-b、S-c 由 E10 定要不要加）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionChoiceCriterion {
+    /// 活槽最少的段先（判决 G2 第一版）。
+    Greedy,
+}
+
+/// 一块盘的单元区：从哪个槽起、有几个槽。选段只在它的整段里挑（末尾凑不满 64 槽的那一截不是聚簇段，同 `lowest_empty_segment`）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct UnitAreaOfOneDevice {
+    pub device: DeviceIdentity,
+    pub start: UnitAreaStart,
+    pub slots: u64,
+}
+
+/// greedy 的打分：段里的活槽数（未释放记录罩住的槽数），分越低越先选。
+#[must_use]
+pub const fn greedy_score(live_slots_in_segment: u64) -> u64 {
+    live_slots_in_segment
+}
+
+/// 挑一个整理区域 R（判决 G2 `choose_region`）。资格四条全成立的段才算：不在这次挂载开过的聚簇段里（`cluster_segments`，段起点）、
+/// 不是活意图的 R、罩住的槽一个都不在 `pinned_slots` 里、至少一个活槽（R 那块盘上未释放记录罩住的槽）。
+/// 按判据打分取最低者，同分取起点槽号最小的；一段都没资格交回 `None`。
+/// 第一版两盘同槽：活槽数按 `unit_area.device` 那块盘的记录数，`cluster_segments`、活 R 与 `pinned_slots` 都只比槽号（判决 G1 攻方 K-n）。
+#[must_use]
+pub fn choose_region(
+    allocation_records: &[AllocationRecord],
+    unit_area: UnitAreaOfOneDevice,
+    cluster_segments: &BTreeSet<SlotNumber>,
+    live_intent_region: Option<CompactionRegion>,
+    pinned_slots: &BTreeSet<SlotNumber>,
+    criterion: RegionChoiceCriterion,
+) -> Option<CompactionRegion> {
+    let unit_area_start = unit_area.start.slot().0;
+    let full_segments = unit_area.slots / CLUSTER_SEGMENT_SLOTS;
+    let segment_slots = usize::try_from(CLUSTER_SEGMENT_SLOTS).expect("64");
+    let mut live_slots_per_segment =
+        vec![0u64; usize::try_from(full_segments).expect("一块盘的段数装得进 usize")];
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == unit_area.device && !record.is_released)
+    {
+        // 迭代上界是这条记录的跨度；只数落在整段里的槽。
+        for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {
+            let Some(offset_in_unit_area) = slot.checked_sub(unit_area_start) else {
+                continue;
+            };
+            let segment = usize::try_from(offset_in_unit_area / CLUSTER_SEGMENT_SLOTS)
+                .expect("段号装得进 usize");
+            if let Some(live_slots) = live_slots_per_segment.get_mut(segment) {
+                *live_slots += 1;
+            }
+        }
+    }
+    let eligible_segments = live_slots_per_segment
+        .iter()
+        .enumerate()
+        .map(|(segment, live_slots)| {
+            let start_slot = SlotNumber(
+                unit_area_start + u64::try_from(segment * segment_slots).expect("槽号装得进 u64"),
+            );
+            (start_slot, *live_slots)
+        })
+        .filter(|(start_slot, live_slots)| {
+            let end_slot = SlotNumber(start_slot.0 + CLUSTER_SEGMENT_SLOTS);
+            let opened_in_this_mount = cluster_segments.contains(start_slot);
+            let is_the_live_intent_region =
+                live_intent_region.is_some_and(|region| region.start_slot() == *start_slot);
+            let holds_a_pinned_slot = pinned_slots.range(*start_slot..end_slot).next().is_some();
+            !opened_in_this_mount
+                && !is_the_live_intent_region
+                && !holds_a_pinned_slot
+                && *live_slots > 0
+        });
+    let chosen_start_slot = match criterion {
+        RegionChoiceCriterion::Greedy => eligible_segments
+            .min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))
+            .map(|(start_slot, _live_slots)| start_slot),
+    }?;
+    Some(
+        CompactionRegion::of_one_cluster_segment(unit_area.device, chosen_start_slot).expect(
+            "单元区起点在聚簇段边界上（UnitAreaStart 的不变量），整段的起点都是 64 的倍数；槽号来自一块盘的单元区，装得进 6 字节",
+        ),
+    )
+}
+
+/// 一批至多搬几个单元（判决 G2 的 K）。至少 1；取值归 E10（老化：结构抗老化 vs 事后整理），这里只是参数。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionBatchUnits(NonZeroUsize);
+
+impl CompactionBatchUnits {
+    /// 0 交回 `None`：一批一个都不搬，意图永远推不完。
+    #[must_use]
+    pub fn new(units: usize) -> Option<Self> {
+        NonZeroUsize::new(units).map(Self)
+    }
+
+    #[must_use]
+    pub const fn units(self) -> usize {
+        self.0.get()
+    }
+}
+
+/// 计划里的一项：搬哪个单元（映射 key）、它现在在哪（源落点，两盘同槽）。目的落点不在这里（模块文档）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct PlannedRelocation {
+    pub mapping_key: Vec<u8>,
+    pub source: Placement,
+}
+
+/// 下一批要搬的：扫描结果里可搬的单元按槽升序取前 `batch` 个（判决 G2 `plan_batch`）。钉住、隔离、已死、不一致的一个都不进。
+/// 这一批发布之后它们离开 R，下一次扫描就不再有它们：各批互不相交，全部批的并集是 R 里可搬的 key 集合。
+#[must_use]
+pub fn plan_batch(region_scan: &RegionScan, batch: CompactionBatchUnits) -> Vec<PlannedRelocation> {
+    region_scan
+        .movable_units()
+        .take(batch.units())
+        .map(|(source, unit)| PlannedRelocation {
+            mapping_key: unit.mapping_key.clone(),
+            source,
+        })
+        .collect()
+}
diff --git a/crates/singlefs-core/src/lib.rs b/crates/singlefs-core/src/lib.rs
--- a/crates/singlefs-core/src/lib.rs
+++ b/crates/singlefs-core/src/lib.rs
@@ -17,6 +17,7 @@
 pub mod bytes;
 pub mod checksum;
 pub mod code_two_tree;
+pub mod compaction_plan;
 pub mod container_tree;
 pub mod deleted_inodes;
 pub mod extent_tree;
@@ -25,6 +26,7 @@
 pub mod instance_table_sweep;
 pub mod journal;
 pub mod lineage_tree;
+pub mod logged_operations;
 pub mod make_filesystem;
 pub mod mount;
 pub mod mounted_read;
@@ -32,6 +34,7 @@
 pub mod pointer;
 pub mod records;
 pub mod recovery;
+pub mod region_scan;
 pub mod retirement_tree;
 pub mod root_record;
 pub mod root_ring;
diff --git a/crates/singlefs-core/src/logged_operations.rs b/crates/singlefs-core/src/logged_operations.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-core/src/logged_operations.rs
@@ -0,0 +1,277 @@
+//! logged_ops 树的条目（D8（核心索引结构） 已定项 5 的意图树；字段表是判决 `research/prompts/m4-gc-r1-main-verification.md`
+//! 第三节 G1「单树、定宽联合条目」）：key 10 = 种类 2 + 签发 txg 8，value 34 定宽、按种类解，条目 44，一个码 2 节点装 369 条。
+//! 第一版只装整理意图（种类 1）：R（设备 4 + 起始槽 6 + 槽数 2）、`c0` 8、`segs0` 8、补零 6；别的种类码保留，读者拒收
+//! （照树表条目 flags「未知位一律拒收」的政策，D8（核心索引结构） 已定项 8）。
+//!
+//! **不存续做点、不存暂停标志**（判决 G1，攻方 F1 / F4）：暂停就是意图留着不推进；续做每批照目标态从 R 起点重扫，
+//! 删意图之前对整个 R 全量核目标态。
+//!
+//! 和 `crate::lineage_tree`、`crate::tombstone_tree` 一样只算、不发一个写：这里只有字段表的读写，签发、推进、删意图归发布入口。
+
+use singlefs_format::{
+    CLUSTER_SEGMENT_SLOTS, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
+};
+
+use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::bytes::{ByteReader, ByteWriter};
+
+/// logged_ops 条目的种类码：1 = 整理（判决 G1）。别的码保留：读者拒收，不当未来版本的条目猜着读。
+pub const LOGGED_OPERATION_KIND_COMPACTION: u16 = 1;
+
+/// 整理意图 value 末尾的补零：34 − (4 + 6 + 2 + 8 + 8)。读者遇到非零一律拒收（同树表条目预留位的政策）。
+const COMPACTION_INTENT_PADDING_BYTES: usize = 6;
+
+/// 槽号在盘上占 6 字节（D3（空间分配） 已定项 7）：起始槽要小于它才写得下。
+const SIX_BYTE_SLOT_NUMBER_LIMIT: u64 = 1 << 48;
+
+/// logged_ops 条目的种类。封闭集合：加一种意图就加一个成员，`match` 不写通配臂。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
+pub enum LoggedOperationKind {
+    /// 整理意图（D26（后台整理与放置回收） 已定项 2；提示类，D8（核心索引结构） 已定项 5）。
+    Compaction,
+}
+
+impl LoggedOperationKind {
+    /// 写进 key 前 2 字节的种类码。
+    #[must_use]
+    pub const fn code(self) -> u16 {
+        match self {
+            LoggedOperationKind::Compaction => LOGGED_OPERATION_KIND_COMPACTION,
+        }
+    }
+
+    /// 盘上读到的种类码认不认得：认不得的（保留码）交回 `None`。
+    #[must_use]
+    pub const fn of_code(kind_code: u16) -> Option<Self> {
+        match kind_code {
+            LOGGED_OPERATION_KIND_COMPACTION => Some(LoggedOperationKind::Compaction),
+            _reserved_kind_code => None,
+        }
+    }
+}
+
+/// 整理意图的区域 R：一块盘上一个 64 槽对齐的聚簇段（判决 G1：起始槽 64 对齐、槽数 = 64；第一版两盘同槽时 R 罩住每块盘上的同号槽，
+/// 攻方 K-n）。只从 [`CompactionRegion::of_one_cluster_segment`] 进来，所以槽数不存：恒是 [`CLUSTER_SEGMENT_SLOTS`]。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
+pub struct CompactionRegion {
+    device: DeviceIdentity,
+    start_slot: SlotNumber,
+}
+
+/// 造不出一个整理区域的两种起始槽。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum CompactionRegionRefusal {
+    /// 起始槽不是 64 的倍数：R 不对齐（判决 G1「R 的解码拒不对齐的值」）。
+    StartOffAClusterSegmentBoundary { start_slot: SlotNumber },
+    /// 起始槽装不进盘上 6 字节的槽号。
+    StartBeyondTheSixByteSlotNumber { start_slot: SlotNumber },
+}
+
+impl CompactionRegion {
+    /// `device` 上从 `start_slot` 起的那一个聚簇段。
+    ///
+    /// # Errors
+    /// 起始槽不是 64 的倍数 ⇒ `StartOffAClusterSegmentBoundary`；装不进 6 字节 ⇒ `StartBeyondTheSixByteSlotNumber`。
+    pub fn of_one_cluster_segment(
+        device: DeviceIdentity,
+        start_slot: SlotNumber,
+    ) -> Result<Self, CompactionRegionRefusal> {
+        if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {
+            return Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary { start_slot });
+        }
+        if start_slot.0 >= SIX_BYTE_SLOT_NUMBER_LIMIT {
+            return Err(CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber { start_slot });
+        }
+        Ok(Self { device, start_slot })
+    }
+
+    #[must_use]
+    pub const fn device(self) -> DeviceIdentity {
+        self.device
+    }
+
+    #[must_use]
+    pub const fn start_slot(self) -> SlotNumber {
+        self.start_slot
+    }
+
+    /// 槽数：恒是一个聚簇段。
+    #[must_use]
+    pub const fn slot_count(self) -> u64 {
+        CLUSTER_SEGMENT_SLOTS
+    }
+
+    /// R 之后的第一个槽（开区间的右端）。
+    #[must_use]
+    pub const fn end_slot(self) -> SlotNumber {
+        SlotNumber(self.start_slot.0 + CLUSTER_SEGMENT_SLOTS)
+    }
+
+    /// 这个槽号在不在 R 里（只看槽号：两盘同槽时 R 罩住每块盘上的同号槽）。
+    #[must_use]
+    pub const fn contains_slot(self, slot: SlotNumber) -> bool {
+        slot.0 >= self.start_slot.0 && slot.0 < self.start_slot.0 + CLUSTER_SEGMENT_SLOTS
+    }
+}
+
+/// 一条整理意图（D26（后台整理与放置回收） 已定项 1 / 已定项 2；字段表判决 G1）。三个字段都是值，不是增量
+/// （D8（核心索引结构） 已定项 5 硬要求 1）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct CompactionIntent {
+    pub region: CompactionRegion,
+    /// `c0`：签发那一刻的分配代；目标态是「R 内不存在分配代 ≤ `c0` 且未释放的落点」（D26 已定项 2）。
+    pub allocation_generation_at_issue: CheckpointTxg,
+    /// `segs0`：签发那次发布写下的记账第 11 项（全空聚簇段数）里 R 所在盘的值；只作观测量，停机不靠它（判决 G1 / G4）。
+    pub empty_cluster_segments_at_issue: u64,
+}
+
+/// logged_ops 条目的 value，按种类分。种类码从成员推出来，key 与 value 对不上的条目造不出来。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperation {
+    Compaction(CompactionIntent),
+}
+
+impl LoggedOperation {
+    #[must_use]
+    pub const fn kind(&self) -> LoggedOperationKind {
+        match self {
+            LoggedOperation::Compaction(_intent) => LoggedOperationKind::Compaction,
+        }
+    }
+}
+
+/// logged_ops 条目的 key：种类 + 签发 txg。全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11：小端存储不构成 memcmp 序），
+/// 先按种类、再按签发先后。一次发布至多签发一条（判决 G1 e1），所以 key 不带序号。
+#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
+pub struct LoggedOperationKey {
+    pub kind: LoggedOperationKind,
+    pub issued_txg: CheckpointTxg,
+}
+
+impl LoggedOperationKey {
+    #[must_use]
+    pub fn sort_key(&self) -> (u16, u64) {
+        (self.kind.code(), self.issued_txg.0)
+    }
+}
+
+/// logged_ops 树里的一条：签发那次发布的 txg 与它声明的意图。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub struct LoggedOperationEntry {
+    pub issued_txg: CheckpointTxg,
+    pub operation: LoggedOperation,
+}
+
+/// 读者拒收一条 logged_ops 条目的原因。条目是盘上读来的字节、不是我们的不变量：调用方把它当损坏处置。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum LoggedOperationEntryRefusal {
+    /// 条目宽（码 2 节点头里的盘上字段）不是字段表的 44。
+    WidthIsNotTheFieldTable { entry_bytes: usize },
+    /// 种类码是保留码。
+    UnknownKind { kind_code: u16 },
+    /// 整理意图的 R 起始槽不对齐、或装不进 6 字节。
+    CompactionRegionRefused(CompactionRegionRefusal),
+    /// 整理意图的 R 槽数不是一个聚簇段（64）。
+    CompactionRegionSlotCountIsNotOneClusterSegment { slot_count: u16 },
+    /// 整理意图 value 末尾那 6 字节补零有非零字节。
+    CompactionPaddingNotZero,
+}
+
+fn entry_bytes() -> usize {
+    usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44")
+}
+
+impl LoggedOperationEntry {
+    #[must_use]
+    pub const fn key(&self) -> LoggedOperationKey {
+        LoggedOperationKey {
+            kind: self.operation.kind(),
+            issued_txg: self.issued_txg,
+        }
+    }
+
+    /// 条目 44：种类 0 | 签发 txg 2 | 整理：R 的设备 10 | R 的起始槽 14 | R 的槽数 20 | `c0` 22 | `segs0` 30 | 补零 38，小端。
+    #[must_use]
+    pub fn to_bytes(&self) -> Vec<u8> {
+        let mut writer = ByteWriter::new(entry_bytes());
+        writer.put_u16(self.operation.kind().code());
+        writer.put_u64(self.issued_txg.0);
+        writer.assert_position(LOGGED_OPERATIONS_KEY_BYTES, "logged_ops 条目的 key");
+        match &self.operation {
+            LoggedOperation::Compaction(intent) => {
+                writer.put_u32(intent.region.device().0);
+                writer.assert_position(14, "整理意图 R 的起始槽");
+                writer.put_six_byte_unsigned(intent.region.start_slot().0);
+                writer.assert_position(20, "整理意图 R 的槽数");
+                writer.put_u16(
+                    u16::try_from(intent.region.slot_count())
+                        .expect("一个聚簇段 64 槽装得进 2 字节"),
+                );
+                writer.assert_position(22, "整理意图的 c0");
+                writer.put_u64(intent.allocation_generation_at_issue.0);
+                writer.assert_position(30, "整理意图的 segs0");
+                writer.put_u64(intent.empty_cluster_segments_at_issue);
+                writer.assert_position(38, "整理意图的补零");
+                writer.skip(COMPACTION_INTENT_PADDING_BYTES);
+            }
+        }
+        writer.assert_position(LOGGED_OPERATIONS_ENTRY_BYTES, "logged_ops 条目");
+        writer.into_bytes()
+    }
+
+    /// logged_ops 条目 44 的读者。
+    ///
+    /// # Errors
+    /// 宽不是 44 ⇒ `WidthIsNotTheFieldTable`；种类码保留 ⇒ `UnknownKind`；整理意图的 R 起始槽不是 64 的倍数 ⇒
+    /// `CompactionRegionRefused`；R 槽数不是 64 ⇒ `CompactionRegionSlotCountIsNotOneClusterSegment`；补零非零 ⇒ `CompactionPaddingNotZero`。
+    pub fn parse(bytes: &[u8]) -> Result<Self, LoggedOperationEntryRefusal> {
+        if bytes.len() != entry_bytes() {
+            return Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable {
+                entry_bytes: bytes.len(),
+            });
+        }
+        let mut reader = ByteReader::at(bytes, 0);
+        let kind_code = reader.get_u16();
+        let issued_txg = CheckpointTxg(reader.get_u64());
+        let kind = LoggedOperationKind::of_code(kind_code)
+            .ok_or(LoggedOperationEntryRefusal::UnknownKind { kind_code })?;
+        let operation = match kind {
+            LoggedOperationKind::Compaction => {
+                LoggedOperation::Compaction(parse_compaction_intent(&mut reader)?)
+            }
+        };
+        Ok(Self {
+            issued_txg,
+            operation,
+        })
+    }
+}
+
+/// 整理意图那 34 字节 value（读者已在 key 之后）。
+fn parse_compaction_intent(
+    reader: &mut ByteReader<'_>,
+) -> Result<CompactionIntent, LoggedOperationEntryRefusal> {
+    let device = DeviceIdentity(reader.get_u32());
+    let start_slot = SlotNumber(reader.get_six_byte_unsigned());
+    let slot_count = reader.get_u16();
+    let allocation_generation_at_issue = CheckpointTxg(reader.get_u64());
+    let empty_cluster_segments_at_issue = reader.get_u64();
+    let padding = reader.take(COMPACTION_INTENT_PADDING_BYTES);
+    if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {
+        return Err(
+            LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment {
+                slot_count,
+            },
+        );
+    }
+    if padding.iter().any(|byte| *byte != 0) {
+        return Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero);
+    }
+    let region = CompactionRegion::of_one_cluster_segment(device, start_slot)
+        .map_err(LoggedOperationEntryRefusal::CompactionRegionRefused)?;
+    Ok(CompactionIntent {
+        region,
+        allocation_generation_at_issue,
+        empty_cluster_segments_at_issue,
+    })
+}
diff --git a/crates/singlefs-core/src/region_scan.rs b/crates/singlefs-core/src/region_scan.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-core/src/region_scan.rs
@@ -0,0 +1,789 @@
+//! 区域扫描（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G9）：对一个整理区域 R 按槽升序看每条未释放的分配记录，
+//! 读单元头反查映射 key、查中央映射核位置，把它分成可搬、钉住、隔离、已死、不一致五类；已释放或没有记录的槽跳过。
+//! 「已死」第一版只报告不动（里程碑五的打包容器那一支不写，只保证这一类在）。
+//!
+//! 它在三层里打头（G2：`scan_region` → `crate::compaction_plan::choose_region` → `crate::compaction_plan::plan_batch`）：
+//! 读单元的那一步交给调用方给的闭包，这里只经 `crate::unit` 的三个 parse 解头，不发一个写、不动分配器。
+//!
+//! 第一版两盘同槽（D2（RAID 条带策略） 已定项 10），每个落点逐盘一条记录、两盘对称（D3（空间分配） 已定项 7；释放前核出对不上时
+//! 两盘一起留在已分配，D19（块指针的结构与宽度预算） 已定项 5）：分类按 R 那块盘上的记录做，钉住与隔离按槽号登记
+//! （R 罩住每块盘上的同号槽，判决 G1 攻方 K-n）。
+
+use std::collections::BTreeSet;
+
+use singlefs_format::{DATA_UNIT_BYTES, NODE_BYTES};
+
+use crate::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use crate::allocator::{AllocationRecord, Placement};
+use crate::logged_operations::CompactionRegion;
+use crate::pointer::{LocationEntry, NodePointer, PointerHead};
+use crate::records::{mapping_key_for_data, mapping_key_for_node};
+use crate::unit::{
+    parse_data_unit, parse_index_node, parse_packed_unit, UnitError, UNIT_CLASS_DATA,
+    UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+};
+
+/// 中央映射里的一条：27 字节 key 加两条位置条目（D19（块指针的结构与宽度预算） 已定项 10）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct CentralMappingEntry {
+    pub key: Vec<u8>,
+    pub locations: [LocationEntry; 2],
+}
+
+impl CentralMappingEntry {
+    /// 这条映射的位置条目里有没有一条指着 `device` 上的 `slot`。
+    #[must_use]
+    pub fn points_at(&self, device: DeviceIdentity, slot: SlotNumber) -> bool {
+        self.locations
+            .iter()
+            .any(|location| location.device == device && location.slot == slot)
+    }
+}
+
+/// 进映射的三类单元（D19（块指针的结构与宽度预算） 已定项 8）。封闭集合。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum MappedUnitClass {
+    /// 码 1 数据单元。
+    Data,
+    /// 码 2 索引节点。
+    IndexNode,
+    /// 码 3 打包记录单元。
+    Packed,
+}
+
+impl MappedUnitClass {
+    /// 单元类型标签，也是映射 key 的第一个字节。
+    #[must_use]
+    pub const fn tag(self) -> u8 {
+        match self {
+            MappedUnitClass::Data => UNIT_CLASS_DATA,
+            MappedUnitClass::IndexNode => UNIT_CLASS_INDEX_NODE,
+            MappedUnitClass::Packed => UNIT_CLASS_PACKED,
+        }
+    }
+}
+
+/// 一个可搬的单元：头里反查出的映射 key 在映射里、映射的位置就是这个槽（G9「可搬」）。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct MovableUnit {
+    pub mapping_key: Vec<u8>,
+    pub unit_class: MappedUnitClass,
+    /// 单元头里的诞生代号（搬迁不改它，D26（后台整理与放置回收） 已定项 5 第 1 条）。
+    pub birth_txg: CheckpointTxg,
+}
+
+/// 单元头为什么读不出。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum UnitHeaderUnreadable {
+    /// 读单元的闭包交回读不到。
+    DeviceReadFailed,
+    /// 记录的跨度不是 1（码 2）也不是 2（码 1 / 码 3）：三个 parse 都不收这样长的单元，不去读。
+    SpanIsNotAUnitFootprint { span_slots: u16 },
+    /// 读到了，parse 拒收（校验和、类标签、结构字段）。
+    RefusedByTheParser(UnitError),
+}
+
+/// 两个来源（单元头、中央映射）对不上的三种（G9「不一致」）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionScanInconsistency {
+    HeaderUnreadable(UnitHeaderUnreadable),
+    /// 指着这个槽的映射条目的 key 打头的类标签与单元头的类不符。
+    UnitClassDisagreesWithTheMappingKey {
+        header_unit_class: MappedUnitClass,
+        mapping_key_class_tag: u8,
+    },
+    /// 头里反查出的 key 在映射里的位置不是这个槽，或指着这个槽的是别的 key。
+    HeaderKeyDisagreesWithTheMappingLocation,
+}
+
+/// 一条未释放的分配记录分进哪一类。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub enum AllocationInRegion {
+    Movable(MovableUnit),
+    /// 罩住的槽里有钉住的（豁免三类或被根直接指着）：不搬。
+    Pinned,
+    /// 映射里没有条目指着它、它在隔离集里（释放前核出对不上，D19 已定项 5 硬规则 1）：搬不动，单列给丢意图那一步（G2 攻方 K-c）。
+    Quarantined,
+    /// 映射里没有条目指着它、头里的 key 映射里也没有、不在隔离集里：第一版只报告。
+    Dead,
+    Inconsistent(RegionScanInconsistency),
+}
+
+/// R 里一条未释放的分配记录与它的分类。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct ScannedAllocation {
+    /// 记录的落点（两盘同槽）。
+    pub placement: Placement,
+    pub class: AllocationInRegion,
+}
+
+/// 一次区域扫描的结果：R 里每条未释放的记录一项，按槽升序。
+#[derive(Clone, Debug, PartialEq, Eq)]
+pub struct RegionScan {
+    pub region: CompactionRegion,
+    pub allocations: Vec<ScannedAllocation>,
+}
+
+impl RegionScan {
+    /// 可搬的那几项，按槽升序：落点与单元。
+    pub fn movable_units(&self) -> impl Iterator<Item = (Placement, &MovableUnit)> {
+        self.allocations
+            .iter()
+            .filter_map(|allocation| match &allocation.class {
+                AllocationInRegion::Movable(unit) => Some((allocation.placement, unit)),
+                AllocationInRegion::Pinned
+                | AllocationInRegion::Quarantined
+                | AllocationInRegion::Dead
+                | AllocationInRegion::Inconsistent(_) => None,
+            })
+    }
+}
+
+/// 扫不了这个 R 的情形。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum RegionScanRefusal {
+    /// R 那块盘上有一条未释放的记录跨着 R 的边界（起点在 R 外而罩进 R，或起点在 R 里而罩出 R）。
+    /// 判决 G9 没定这样的记录归哪一类；分配器自己发的落点不跨段（单元区起点在段边界上、两槽单元起点偶数），
+    /// 只有盘上读来的坏记录才有（`recovery::allocation_records_fit_the_pool_geometry` 不判跨度与单元类别，C411）：第一版不支持。
+    RecordStraddlesTheRegionBoundaryUnsupported {
+        device: DeviceIdentity,
+        slot: SlotNumber,
+        span_slots: u16,
+    },
+}
+
+/// 单元头里反查出来的东西。
+struct UnitHeaderIdentity {
+    unit_class: MappedUnitClass,
+    mapping_key: Vec<u8>,
+    birth_txg: CheckpointTxg,
+}
+
+/// 扫一个 R：`allocation_records` 里 R 那块盘上起点在 R 里、未释放的每条记录分一类，按槽升序交回。
+/// `read_unit_bytes(盘, 槽, 字节数)` 读那个落点上的单元，读不到交 `None`。
+///
+/// # Errors
+/// R 那块盘上有未释放的记录跨着 R 的边界 ⇒ [`RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported`]，一条都不分类。
+pub fn scan_region<ReadUnitBytes>(
+    region: CompactionRegion,
+    allocation_records: &[AllocationRecord],
+    mapping_entries: &[CentralMappingEntry],
+    pinned_slots: &BTreeSet<SlotNumber>,
+    quarantined_slots: &BTreeSet<SlotNumber>,
+    mut read_unit_bytes: ReadUnitBytes,
+) -> Result<RegionScan, RegionScanRefusal>
+where
+    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
+{
+    let device = region.device();
+    let mut records_in_region: Vec<&AllocationRecord> = Vec::new();
+    for record in allocation_records
+        .iter()
+        .filter(|record| record.device == device && !record.is_released)
+    {
+        let record_end = record.slot.0 + u64::from(record.span_slots);
+        let starts_inside = region.contains_slot(record.slot);
+        let reaches_in_from_below =
+            record.slot.0 < region.start_slot().0 && record_end > region.start_slot().0;
+        let reaches_out_above = starts_inside && record_end > region.end_slot().0;
+        if reaches_in_from_below || reaches_out_above {
+            return Err(
+                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                    device,
+                    slot: record.slot,
+                    span_slots: record.span_slots,
+                },
+            );
+        }
+        if starts_inside {
+            records_in_region.push(record);
+        }
+    }
+    records_in_region.sort_by_key(|record| record.slot);
+    let allocations = records_in_region
+        .into_iter()
+        .map(|record| ScannedAllocation {
+            placement: Placement {
+                slot: record.slot,
+                span: u64::from(record.span_slots),
+            },
+            class: classify_allocation(
+                record,
+                mapping_entries,
+                pinned_slots,
+                quarantined_slots,
+                &mut read_unit_bytes,
+            ),
+        })
+        .collect();
+    Ok(RegionScan {
+        region,
+        allocations,
+    })
+}
+
+/// 一条未释放记录的分类，次序：钉住 → 隔离（映射里没人指着它）→ 读头 → 类标签 → key 与位置。
+fn classify_allocation<ReadUnitBytes>(
+    record: &AllocationRecord,
+    mapping_entries: &[CentralMappingEntry],
+    pinned_slots: &BTreeSet<SlotNumber>,
+    quarantined_slots: &BTreeSet<SlotNumber>,
+    read_unit_bytes: &mut ReadUnitBytes,
+) -> AllocationInRegion
+where
+    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
+{
+    let covered_slots = record.slot..SlotNumber(record.slot.0 + u64::from(record.span_slots));
+    if pinned_slots.range(covered_slots.clone()).next().is_some() {
+        return AllocationInRegion::Pinned;
+    }
+    let entries_pointing_here: Vec<&CentralMappingEntry> = mapping_entries
+        .iter()
+        .filter(|entry| entry.points_at(record.device, record.slot))
+        .collect();
+    if entries_pointing_here.is_empty() && quarantined_slots.range(covered_slots).next().is_some() {
+        return AllocationInRegion::Quarantined;
+    }
+    let header = match read_unit_header(record, read_unit_bytes) {
+        Ok(header) => header,
+        Err(unreadable) => {
+            return AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                unreadable,
+            ))
+        }
+    };
+    if let Some(entry) = entries_pointing_here
+        .iter()
+        .find(|entry| entry.key.first() != Some(&header.unit_class.tag()))
+    {
+        return AllocationInRegion::Inconsistent(
+            RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
+                header_unit_class: header.unit_class,
+                mapping_key_class_tag: entry.key.first().copied().unwrap_or_default(),
+            },
+        );
+    }
+    let header_key_is_mapped = mapping_entries
+        .iter()
+        .any(|entry| entry.key == header.mapping_key);
+    if entries_pointing_here.is_empty() && !header_key_is_mapped {
+        return AllocationInRegion::Dead;
+    }
+    let every_entry_pointing_here_carries_the_header_key = entries_pointing_here
+        .iter()
+        .all(|entry| entry.key == header.mapping_key);
+    if !entries_pointing_here.is_empty() && every_entry_pointing_here_carries_the_header_key {
+        return AllocationInRegion::Movable(MovableUnit {
+            mapping_key: header.mapping_key,
+            unit_class: header.unit_class,
+            birth_txg: header.birth_txg,
+        });
+    }
+    AllocationInRegion::Inconsistent(
+        RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
+    )
+}
+
+/// 读一条记录落点上的单元、解头、反查映射 key：跨度 1 当码 2 读 16 KiB，跨度 2 读 32 KiB 先当码 1、类标签是 3 再当码 3。
+/// key 的各段照写者的取法（D19（块指针的结构与宽度预算） 已定项 6 / 已定项 8）：码 1 = 树 ID + 诞生代号 + 写序，
+/// 码 2 = 树 ID + 诞生代号 + 实例代号 + 出生序号，码 3 = 出生树 + 诞生代号 + 写序里的实例代号 + 出生序号。
+fn read_unit_header<ReadUnitBytes>(
+    record: &AllocationRecord,
+    read_unit_bytes: &mut ReadUnitBytes,
+) -> Result<UnitHeaderIdentity, UnitHeaderUnreadable>
+where
+    ReadUnitBytes: FnMut(DeviceIdentity, SlotNumber, usize) -> Option<Vec<u8>>,
+{
+    let node_pointer_of = |birth_tree, birth_txg, instance, birth_sequence| NodePointer {
+        head: PointerHead {
+            birth_tree,
+            birth_txg,
+        },
+        locations: NodePointer::empty_root().locations,
+        instance,
+        birth_sequence,
+    };
+    match record.span_slots {
+        1 => {
+            let bytes = read_unit_bytes(
+                record.device,
+                record.slot,
+                usize::try_from(NODE_BYTES).expect("16384"),
+            )
+            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
+            let header =
+                parse_index_node(&bytes).map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
+            Ok(UnitHeaderIdentity {
+                unit_class: MappedUnitClass::IndexNode,
+                mapping_key: mapping_key_for_node(
+                    UNIT_CLASS_INDEX_NODE,
+                    node_pointer_of(
+                        header.tree,
+                        header.birth_txg,
+                        header.instance,
+                        header.birth_sequence,
+                    ),
+                ),
+                birth_txg: header.birth_txg,
+            })
+        }
+        2 => {
+            let bytes = read_unit_bytes(
+                record.device,
+                record.slot,
+                usize::try_from(DATA_UNIT_BYTES).expect("32768"),
+            )
+            .ok_or(UnitHeaderUnreadable::DeviceReadFailed)?;
+            match parse_data_unit(&bytes) {
+                Ok(header) => Ok(UnitHeaderIdentity {
+                    unit_class: MappedUnitClass::Data,
+                    mapping_key: mapping_key_for_data(
+                        PointerHead {
+                            birth_tree: header.identity.tree,
+                            birth_txg: header.birth_txg,
+                        },
+                        header.write_order,
+                    ),
+                    birth_txg: header.birth_txg,
+                }),
+                Err(UnitError::WrongUnitClass {
+                    found: UNIT_CLASS_PACKED,
+                    ..
+                }) => {
+                    let header = parse_packed_unit(&bytes)
+                        .map_err(UnitHeaderUnreadable::RefusedByTheParser)?;
+                    Ok(UnitHeaderIdentity {
+                        unit_class: MappedUnitClass::Packed,
+                        mapping_key: mapping_key_for_node(
+                            UNIT_CLASS_PACKED,
+                            node_pointer_of(
+                                header.identity.birth_tree,
+                                header.birth_txg,
+                                header.write_order.instance,
+                                header.birth_sequence,
+                            ),
+                        ),
+                        birth_txg: header.birth_txg,
+                    })
+                }
+                Err(refusal) => Err(UnitHeaderUnreadable::RefusedByTheParser(refusal)),
+            }
+        }
+        span_slots => Err(UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots }),
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use std::collections::BTreeMap;
+
+    use singlefs_format::{TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE, UNIT_AREA_START_SLOT};
+
+    use crate::address::{InstanceGeneration, TreeIdentifier};
+    use crate::pointer::BirthSequence;
+    use crate::unit::{
+        build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
+        WriteOrder, PACKED_TYPE_INODE,
+    };
+
+    const FILESYSTEM_IDENTIFIER: [u8; 16] = [7u8; 16];
+    const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
+    const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
+    const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
+    const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
+
+    /// 单元区里起点 50240 的那个聚簇段 [50240, 50304)。
+    fn region_under_test() -> CompactionRegion {
+        CompactionRegion::of_one_cluster_segment(DEVICE_ZERO, SlotNumber(UNIT_AREA_START_SLOT + 64))
+            .expect("50240 = 785 × 64")
+    }
+
+    fn slot_in_region(offset_in_slots: u64) -> SlotNumber {
+        SlotNumber(region_under_test().start_slot().0 + offset_in_slots)
+    }
+
+    /// 一块镜像：两块盘上每个落点各一份字节、各一条记录（两盘同槽），加中央映射的条目。
+    #[derive(Default)]
+    struct PoolUnderTest {
+        units: BTreeMap<(DeviceIdentity, SlotNumber), Vec<u8>>,
+        records: Vec<AllocationRecord>,
+        mapping_entries: Vec<CentralMappingEntry>,
+    }
+
+    impl PoolUnderTest {
+        fn place(&mut self, slot: SlotNumber, span_slots: u16, unit: Option<&[u8]>) {
+            for device in [DEVICE_ZERO, DEVICE_ONE] {
+                self.records.push(AllocationRecord {
+                    device,
+                    slot,
+                    span_slots,
+                    generation: BIRTH_TXG,
+                    is_released: false,
+                });
+                if let Some(bytes) = unit {
+                    self.units.insert((device, slot), bytes.to_vec());
+                }
+            }
+        }
+
+        fn release(&mut self, slot: SlotNumber) {
+            for record in self.records.iter_mut().filter(|record| record.slot == slot) {
+                record.is_released = true;
+            }
+        }
+
+        fn map(&mut self, key: Vec<u8>, slot: SlotNumber) {
+            self.mapping_entries.push(CentralMappingEntry {
+                key,
+                locations: locations_at(slot),
+            });
+        }
+
+        fn scan(
+            &self,
+            pinned_slots: &BTreeSet<SlotNumber>,
+            quarantined_slots: &BTreeSet<SlotNumber>,
+        ) -> Result<RegionScan, RegionScanRefusal> {
+            scan_region(
+                region_under_test(),
+                &self.records,
+                &self.mapping_entries,
+                pinned_slots,
+                quarantined_slots,
+                |device, slot, unit_bytes| {
+                    self.units
+                        .get(&(device, slot))
+                        .filter(|bytes| bytes.len() == unit_bytes)
+                        .cloned()
+                },
+            )
+        }
+    }
+
+    fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
+        [DEVICE_ZERO, DEVICE_ONE].map(|device| LocationEntry {
+            device,
+            slot,
+            unit_checksum: 0,
+        })
+    }
+
+    /// 一个码 1 数据单元与写者给它的映射 key（`transaction` 里 `mapping_key_for_data` 那一处的取法：出生树 = extent 树、
+    /// 出生 txg = 这次发布、写序 = 实例代号与事务号）。
+    fn data_unit(transaction: u64) -> (Vec<u8>, Vec<u8>) {
+        let write_order = WriteOrder {
+            instance: WRITING_INSTANCE,
+            transaction,
+        };
+        let bytes = build_data_unit(
+            DataUnitIdentity {
+                tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                object: 1,
+                object_birth: CheckpointTxg(3),
+                anchor_offset: 0,
+            },
+            BIRTH_TXG,
+            &FILESYSTEM_IDENTIFIER,
+            write_order,
+            &[0x5A; 100],
+        );
+        let key = mapping_key_for_data(
+            PointerHead {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+                birth_txg: BIRTH_TXG,
+            },
+            write_order,
+        );
+        (bytes, key)
+    }
+
+    fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
+        NodePointer {
+            head: PointerHead {
+                birth_tree: TreeIdentifier(tree),
+                birth_txg: BIRTH_TXG,
+            },
+            locations: NodePointer::empty_root().locations,
+            instance: WRITING_INSTANCE,
+            birth_sequence: BirthSequence(birth_sequence),
+        }
+    }
+
+    /// 一个码 2 节点与写者给它的映射 key（类标签 2、出生树 = 节点的树、出生 txg、实例代号、出生序号）。
+    fn index_node(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
+        let bytes = build_index_node(
+            TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+            0,
+            24,
+            &[0u8; 24],
+            &[0u8; 24],
+            BIRTH_TXG,
+            &FILESYSTEM_IDENTIFIER,
+            WRITING_INSTANCE,
+            BirthSequence(birth_sequence),
+            112,
+            &[],
+        );
+        let key = mapping_key_for_node(
+            UNIT_CLASS_INDEX_NODE,
+            node_pointer(TREE_IDENTIFIER_EXTENT, birth_sequence),
+        );
+        (bytes, key)
+    }
+
+    /// 一个码 3 inode 叶容器与写者给它的映射 key（类标签 3、出生树 = inode 树、出生 txg、实例代号、出生序号）。
+    fn packed_unit(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
+        let bytes = build_packed_unit(
+            PackedIdentity {
+                birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+                record_type: PACKED_TYPE_INODE,
+                container: 1,
+                container_birth: BIRTH_TXG,
+            },
+            140,
+            &[vec![1u8; 140]],
+            BIRTH_TXG,
+            &FILESYSTEM_IDENTIFIER,
+            WriteOrder {
+                instance: WRITING_INSTANCE,
+                transaction: 9,
+            },
+            BirthSequence(birth_sequence),
+        );
+        let key = mapping_key_for_node(
+            UNIT_CLASS_PACKED,
+            node_pointer(TREE_IDENTIFIER_INODE, birth_sequence),
+        );
+        (bytes, key)
+    }
+
+    fn scanned(offset_in_slots: u64, span: u64, class: AllocationInRegion) -> ScannedAllocation {
+        ScannedAllocation {
+            placement: Placement {
+                slot: slot_in_region(offset_in_slots),
+                span,
+            },
+            class,
+        }
+    }
+
+    fn inconsistent_by_key() -> AllocationInRegion {
+        AllocationInRegion::Inconsistent(
+            RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,
+        )
+    }
+
+    /// 判决 G9 的五类在一个 R 里各放至少一个，逐项钉分类、落点与可搬单元的 key / 类 / 诞生代；已释放的记录、R 外的记录、
+    /// 另一块盘上的镜像记录都不出现。
+    #[test]
+    fn every_class_of_the_region_scan_is_pinned_down_slot_by_slot() {
+        let mut pool = PoolUnderTest::default();
+        let (movable_data, movable_data_key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&movable_data));
+        pool.map(movable_data_key.clone(), slot_in_region(0));
+        let (pinned_node, _pinned_node_key) = index_node(0);
+        pool.place(slot_in_region(2), 1, Some(&pinned_node));
+        let (movable_node, movable_node_key) = index_node(1);
+        pool.place(slot_in_region(3), 1, Some(&movable_node));
+        pool.map(movable_node_key.clone(), slot_in_region(3));
+        let (movable_packed, movable_packed_key) = packed_unit(0);
+        pool.place(slot_in_region(4), 2, Some(&movable_packed));
+        pool.map(movable_packed_key.clone(), slot_in_region(4));
+        let (quarantined_data, _quarantined_key) = data_unit(2);
+        pool.place(slot_in_region(6), 2, Some(&quarantined_data));
+        let (dead_data, _dead_key) = data_unit(3);
+        pool.place(slot_in_region(8), 2, Some(&dead_data));
+        let (mismatched_data, _mismatched_key) = data_unit(4);
+        let (_other_data, other_key) = data_unit(5);
+        pool.place(slot_in_region(10), 2, Some(&mismatched_data));
+        pool.map(other_key, slot_in_region(10));
+        let (released_data, released_key) = data_unit(6);
+        pool.place(slot_in_region(12), 2, Some(&released_data));
+        pool.map(released_key, slot_in_region(12));
+        pool.release(slot_in_region(12));
+        let (outside_data, outside_key) = data_unit(7);
+        pool.place(slot_in_region(64), 2, Some(&outside_data));
+        pool.map(outside_key, slot_in_region(64));
+
+        let scan = pool
+            .scan(
+                &BTreeSet::from([slot_in_region(2)]),
+                &BTreeSet::from([slot_in_region(6)]),
+            )
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(scan.region, region_under_test());
+        assert_eq!(
+            scan.allocations,
+            vec![
+                scanned(
+                    0,
+                    2,
+                    AllocationInRegion::Movable(MovableUnit {
+                        mapping_key: movable_data_key,
+                        unit_class: MappedUnitClass::Data,
+                        birth_txg: BIRTH_TXG,
+                    })
+                ),
+                scanned(2, 1, AllocationInRegion::Pinned),
+                scanned(
+                    3,
+                    1,
+                    AllocationInRegion::Movable(MovableUnit {
+                        mapping_key: movable_node_key,
+                        unit_class: MappedUnitClass::IndexNode,
+                        birth_txg: BIRTH_TXG,
+                    })
+                ),
+                scanned(
+                    4,
+                    2,
+                    AllocationInRegion::Movable(MovableUnit {
+                        mapping_key: movable_packed_key,
+                        unit_class: MappedUnitClass::Packed,
+                        birth_txg: BIRTH_TXG,
+                    })
+                ),
+                scanned(6, 2, AllocationInRegion::Quarantined),
+                scanned(8, 2, AllocationInRegion::Dead),
+                scanned(10, 2, inconsistent_by_key()),
+            ],
+            "槽 +0 / +3 / +4 可搬（码 1 / 码 2 / 码 3），+2 钉住，+6 隔离，+8 已死，+10 头的 key 与指着它的映射条目不同；\
+             +12 已释放、+64 在 R 外，都不出现"
+        );
+        assert_eq!(scan.movable_units().count(), 3, "可搬三个");
+    }
+
+    /// 映射里这个 key 的位置挪到了 R 里另一个槽，原来那个槽上的单元头照旧：没有条目指着它，但头里的 key 在映射里，
+    /// 两个来源对不上，判「不一致」，不判「已死」（G9「头里的 key 与映射里的位置对不上」）。
+    #[test]
+    fn a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead() {
+        let mut pool = PoolUnderTest::default();
+        let (data, key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, Some(&data));
+        pool.map(key, slot_in_region(20));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(scan.allocations, vec![scanned(0, 2, inconsistent_by_key())]);
+    }
+
+    /// 映射条目指着这个槽，而单元头里反查出的 key 是另一把（头的写序与映射 key 的不同）：判「不一致」，不判可搬。
+    #[test]
+    fn a_unit_whose_header_key_differs_from_the_mapping_entry_pointing_at_it_is_inconsistent() {
+        let mut pool = PoolUnderTest::default();
+        let (_mapped_data, mapped_key) = data_unit(1);
+        let (data_with_another_write_order, _its_own_key) = data_unit(2);
+        pool.place(slot_in_region(0), 2, Some(&data_with_another_write_order));
+        pool.map(mapped_key, slot_in_region(0));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(scan.allocations, vec![scanned(0, 2, inconsistent_by_key())]);
+    }
+
+    /// 「不一致」的另外两种：头读不出（读不到、头校验和对不上、跨度不是单元的大小）与类标签和映射 key 不符。
+    #[test]
+    fn an_unreadable_header_or_a_class_tag_unlike_the_mapping_key_is_inconsistent() {
+        let mut pool = PoolUnderTest::default();
+        let (_never_written, unwritten_key) = data_unit(1);
+        pool.place(slot_in_region(0), 2, None);
+        pool.map(unwritten_key, slot_in_region(0));
+        let (mut damaged, damaged_key) = data_unit(2);
+        damaged[60] ^= 1;
+        pool.place(slot_in_region(2), 2, Some(&damaged));
+        pool.map(damaged_key, slot_in_region(2));
+        let (data, _data_key) = data_unit(3);
+        let (_node, node_key) = index_node(0);
+        pool.place(slot_in_region(4), 2, Some(&data));
+        pool.map(node_key, slot_in_region(4));
+        let (_three_slot_unit, three_slot_key) = data_unit(4);
+        pool.place(slot_in_region(6), 3, None);
+        pool.map(three_slot_key, slot_in_region(6));
+
+        let scan = pool
+            .scan(&BTreeSet::new(), &BTreeSet::new())
+            .expect("没有跨 R 边界的记录");
+
+        assert_eq!(
+            scan.allocations,
+            vec![
+                scanned(
+                    0,
+                    2,
+                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                        UnitHeaderUnreadable::DeviceReadFailed
+                    ))
+                ),
+                scanned(
+                    2,
+                    2,
+                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                        UnitHeaderUnreadable::RefusedByTheParser(UnitError::HeaderChecksumMismatch)
+                    ))
+                ),
+                scanned(
+                    4,
+                    2,
+                    AllocationInRegion::Inconsistent(
+                        RegionScanInconsistency::UnitClassDisagreesWithTheMappingKey {
+                            header_unit_class: MappedUnitClass::Data,
+                            mapping_key_class_tag: UNIT_CLASS_INDEX_NODE,
+                        }
+                    )
+                ),
+                scanned(
+                    6,
+                    3,
+                    AllocationInRegion::Inconsistent(RegionScanInconsistency::HeaderUnreadable(
+                        UnitHeaderUnreadable::SpanIsNotAUnitFootprint { span_slots: 3 }
+                    ))
+                ),
+            ]
+        );
+    }
+
+    /// R 那块盘上一条未释放的记录跨着 R 的边界（从下面罩进来、从里面罩出去）：判决 G9 没定它归哪一类，整次拒、一条都不分类。
+    #[test]
+    fn a_record_straddling_the_region_boundary_is_refused_before_any_classification() {
+        let (inside, inside_key) = data_unit(1);
+        let slot_below_the_region = SlotNumber(region_under_test().start_slot().0 - 1);
+
+        let mut reaching_in = PoolUnderTest::default();
+        reaching_in.place(slot_in_region(0), 2, Some(&inside));
+        reaching_in.map(inside_key.clone(), slot_in_region(0));
+        reaching_in.place(slot_below_the_region, 2, None);
+        assert_eq!(
+            reaching_in.scan(&BTreeSet::new(), &BTreeSet::new()),
+            Err(
+                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                    device: DEVICE_ZERO,
+                    slot: slot_below_the_region,
+                    span_slots: 2,
+                }
+            ),
+            "起点在 R 之下一槽、跨 2：罩进 R"
+        );
+
+        let mut reaching_out = PoolUnderTest::default();
+        reaching_out.place(slot_in_region(0), 2, Some(&inside));
+        reaching_out.map(inside_key, slot_in_region(0));
+        reaching_out.place(slot_in_region(63), 2, None);
+        assert_eq!(
+            reaching_out.scan(&BTreeSet::new(), &BTreeSet::new()),
+            Err(
+                RegionScanRefusal::RecordStraddlesTheRegionBoundaryUnsupported {
+                    device: DEVICE_ZERO,
+                    slot: slot_in_region(63),
+                    span_slots: 2,
+                }
+            ),
+            "起点在 R 末槽、跨 2：罩出 R"
+        );
+    }
+}
diff --git a/crates/singlefs-format/src/lib.rs b/crates/singlefs-format/src/lib.rs
--- a/crates/singlefs-format/src/lib.rs
+++ b/crates/singlefs-format/src/lib.rs
@@ -184,6 +184,21 @@
 
 /// extent 树上段一片叶罩几个 inode 号：(16384 − 163) ÷ 113 的整数部分（一个 inode 号至多一条上段叶条目，叶永远装得下）。
 pub const EXTENT_TREE_UPPER_LEAF_INODES: u64 = 143;
+
+/// logged_ops 树条目的 key：种类 2 + 签发 txg 8，按逐字段无符号整数、字段自左向右比较（D8（核心索引结构） 已定项 11；
+/// 判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1「单树、定宽联合条目」）。format-const: LOGGED_OPERATIONS_KEY_BYTES
+pub const LOGGED_OPERATIONS_KEY_BYTES: u64 = 10;
+
+/// logged_ops 树条目的 value，按种类解、定宽：种类 1（整理）= R 的设备 4 + 起始槽 6 + 槽数 2、`c0` 8、`segs0` 8、补零 6；
+/// 别的种类码保留（同一判决 G1）。format-const: LOGGED_OPERATIONS_VALUE_BYTES
+pub const LOGGED_OPERATIONS_VALUE_BYTES: u64 = 34;
+
+/// logged_ops 树条目：key 10 + value 34（同一判决 G1）。format-const: LOGGED_OPERATIONS_ENTRY_BYTES
+pub const LOGGED_OPERATIONS_ENTRY_BYTES: u64 = 44;
+
+/// 一个 logged_ops 码 2 节点装几条：(16384 − 头 `86 + 2 × 10 + 29`) ÷ 44 的整数部分（同一判决 G1「一节点 369 条」）。
+/// format-const: LOGGED_OPERATIONS_ENTRIES_PER_NODE
+pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 369;
 
 /// 新池新建文件这次发布写出的记账行数（D5（快照 / 空间记账机制） 已定项 8，2026-09-14 用户定案 15 行）。
 pub const NEW_POOL_FILE_CREATION_ACCOUNTING_ROWS: u64 = 15;
@@ -633,6 +648,35 @@
             index_node_header_bytes(MAPPING_KEY_BYTES),
             "中央映射树：key 宽 27"
         );
+    }
+
+    /// logged_ops 树条目的四个宽写成整数字面量，原来的算式留在这里，一个数不差（判决 `m4-gc-r1` 第三节 G1）：
+    /// key = 种类 2 + 签发 txg 8；整理那一种的 value = R 的设备 4 + 起始槽 6 + 槽数 2 + `c0` 8 + `segs0` 8 + 补零 6；
+    /// 一节点条数 = 节点里 key 宽 10 的码 2 头之后装得下几条 44。
+    #[test]
+    fn logged_operations_widths_equal_the_field_sums_and_the_node_capacity_they_stand_for() {
+        assert_eq!(
+            LOGGED_OPERATIONS_KEY_BYTES,
+            2 + 8,
+            "key = 种类 2 + 签发 txg 8"
+        );
+        assert_eq!(
+            LOGGED_OPERATIONS_VALUE_BYTES,
+            4 + 6 + 2 + 8 + 8 + 6,
+            "整理的 value = R 的设备 4 + 起始槽 6 + 槽数 2 + c0 8 + segs0 8 + 补零 6"
+        );
+        assert_eq!(
+            LOGGED_OPERATIONS_ENTRY_BYTES,
+            LOGGED_OPERATIONS_KEY_BYTES + LOGGED_OPERATIONS_VALUE_BYTES,
+            "条目 = key + value"
+        );
+        assert_eq!(
+            LOGGED_OPERATIONS_ENTRIES_PER_NODE,
+            (NODE_BYTES - index_node_header_bytes(LOGGED_OPERATIONS_KEY_BYTES))
+                / LOGGED_OPERATIONS_ENTRY_BYTES,
+            "一节点条数 = (16384 − key 宽 10 的码 2 头) ÷ 44 的整数部分"
+        );
+        assert_eq!(LOGGED_OPERATIONS_ENTRIES_PER_NODE, 369, "判决 G1 写的 369");
     }
 
     /// 系统配置的 489 与它所在的槽：槽宽 4096 时余 3607（D22 已定项 9「槽内余 3607」），装得下下一条 86 字节指针；
diff --git a/crates/singlefs-harness/tests/compaction_plan_chooses_regions_and_batches_by_the_clauses.rs b/crates/singlefs-harness/tests/compaction_plan_chooses_regions_and_batches_by_the_clauses.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/compaction_plan_chooses_regions_and_batches_by_the_clauses.rs
@@ -0,0 +1,465 @@
+//! 整理的选段与计划（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G2；锚点取自正推报告
+//! `research/prompts/m4-gc-r1-sonnet-output.md`「单测怎么钉绝对值」，锚点 4 照这一批的派发只钉条数与 key 集合）：
+//! - 锚点 1：段 50176 / 50240 / 50304 活槽 [60, 10, 0]，greedy 选段 50240，全空的段 50304 不选；
+//! - 锚点 2：段 50240 里有一个钉住的槽 ⇒ 改选段 50176；
+//! - 锚点 3：段 50240 在这次挂载开过的聚簇段里 ⇒ 改选段 50176；有资格的全被排除 ⇒ `None`；
+//! - 活意图的 R 不再选；同分取起点槽号最小的；
+//! - 锚点 4：K = 3、R 里三个可搬的数据单元 ⇒ 一批三条，key 集合就是那三把；
+//! - 守恒：一批批搬下去，每批 ≤ K、批间不交、并集 = R 里可搬的 key 集合，钉住 / 隔离 / 已死 / 不一致的一个都不进。
+//!
+//! 期望值按锚点手写，不从被测函数反推。
+
+use std::collections::{BTreeMap, BTreeSet};
+
+use singlefs_core::address::{
+    CheckpointTxg, DeviceIdentity, InstanceGeneration, SlotNumber, TreeIdentifier,
+};
+use singlefs_core::allocator::{AllocationRecord, Placement, UnitAreaStart};
+use singlefs_core::compaction_plan::{
+    choose_region, greedy_score, plan_batch, CompactionBatchUnits, PlannedRelocation,
+    RegionChoiceCriterion, UnitAreaOfOneDevice,
+};
+use singlefs_core::logged_operations::CompactionRegion;
+use singlefs_core::pointer::{BirthSequence, LocationEntry, NodePointer, PointerHead};
+use singlefs_core::records::{mapping_key_for_data, mapping_key_for_node};
+use singlefs_core::region_scan::{scan_region, CentralMappingEntry, RegionScan};
+use singlefs_core::unit::{
+    build_data_unit, build_index_node, build_packed_unit, DataUnitIdentity, PackedIdentity,
+    WriteOrder, PACKED_TYPE_INODE, UNIT_CLASS_INDEX_NODE, UNIT_CLASS_PACKED,
+};
+use singlefs_format::{TREE_IDENTIFIER_EXTENT, TREE_IDENTIFIER_INODE};
+
+const DEVICE_ZERO: DeviceIdentity = DeviceIdentity(0);
+const DEVICE_ONE: DeviceIdentity = DeviceIdentity(1);
+const BIRTH_TXG: CheckpointTxg = CheckpointTxg(7);
+const WRITING_INSTANCE: InstanceGeneration = InstanceGeneration(2);
+const FILESYSTEM_IDENTIFIER: [u8; 16] = [7u8; 16];
+
+/// 单元区三个整段 [50176, 50240)、[50240, 50304)、[50304, 50368)。
+const FIRST_SEGMENT: u64 = 50176;
+const SECOND_SEGMENT: u64 = 50240;
+const THIRD_SEGMENT: u64 = 50304;
+
+fn unit_area_of_three_segments() -> UnitAreaOfOneDevice {
+    let start = UnitAreaStart::of_the_default_journal_ring();
+    assert_eq!(
+        start.slot(),
+        SlotNumber(FIRST_SEGMENT),
+        "默认环下单元区从 50176 起"
+    );
+    UnitAreaOfOneDevice {
+        device: DEVICE_ZERO,
+        start,
+        slots: 3 * 64,
+    }
+}
+
+fn region_at(start_slot: u64) -> CompactionRegion {
+    CompactionRegion::of_one_cluster_segment(DEVICE_ZERO, SlotNumber(start_slot))
+        .expect("三个段的起点都是 64 的倍数")
+}
+
+/// 两块盘上同一个落点各一条未释放记录（两盘同槽）。
+fn records_on_both_devices(slot: u64, span_slots: u16) -> [AllocationRecord; 2] {
+    [DEVICE_ZERO, DEVICE_ONE].map(|device| AllocationRecord {
+        device,
+        slot: SlotNumber(slot),
+        span_slots,
+        generation: BIRTH_TXG,
+        is_released: false,
+    })
+}
+
+/// 锚点 1 的记录：段 50176 里 30 个两槽单元（活槽 60）、段 50240 里 5 个（活槽 10）、段 50304 里只有一条已释放的（活槽 0）。
+fn records_with_live_slots_sixty_ten_and_zero() -> Vec<AllocationRecord> {
+    let mut records = Vec::new();
+    for unit in 0..30 {
+        records.extend(records_on_both_devices(FIRST_SEGMENT + 2 * unit, 2));
+    }
+    for unit in 0..5 {
+        records.extend(records_on_both_devices(SECOND_SEGMENT + 2 * unit, 2));
+    }
+    for mut released in records_on_both_devices(THIRD_SEGMENT, 2) {
+        released.is_released = true;
+        records.push(released);
+    }
+    records
+}
+
+fn choose_greedily(
+    records: &[AllocationRecord],
+    cluster_segments: &BTreeSet<SlotNumber>,
+    live_intent_region: Option<CompactionRegion>,
+    pinned_slots: &BTreeSet<SlotNumber>,
+) -> Option<CompactionRegion> {
+    choose_region(
+        records,
+        unit_area_of_three_segments(),
+        cluster_segments,
+        live_intent_region,
+        pinned_slots,
+        RegionChoiceCriterion::Greedy,
+    )
+}
+
+/// 锚点 1：活槽 [60, 10, 0]，选活槽最少而至少有一个活槽的段 50240；段 50304 全空、没有可搬的，不选。
+#[test]
+fn greedy_chooses_the_segment_with_the_fewest_live_slots_but_not_an_empty_one() {
+    assert_eq!(greedy_score(10), 10, "greedy 的分就是活槽数");
+    assert_eq!(
+        choose_greedily(
+            &records_with_live_slots_sixty_ten_and_zero(),
+            &BTreeSet::new(),
+            None,
+            &BTreeSet::new(),
+        ),
+        Some(region_at(SECOND_SEGMENT))
+    );
+}
+
+/// 锚点 2：段 50240 里有一个钉住的槽 ⇒ 它没资格，改选段 50176。
+#[test]
+fn a_segment_holding_a_pinned_slot_is_not_chosen() {
+    assert_eq!(
+        choose_greedily(
+            &records_with_live_slots_sixty_ten_and_zero(),
+            &BTreeSet::new(),
+            None,
+            &BTreeSet::from([SlotNumber(SECOND_SEGMENT + 20)]),
+        ),
+        Some(region_at(FIRST_SEGMENT))
+    );
+}
+
+/// 锚点 3：段 50240 是这次挂载开过的聚簇段 ⇒ 改选段 50176；这两段都开过、段 50304 没有活槽 ⇒ 一段都没资格，`None`。
+#[test]
+fn a_segment_opened_in_this_mount_is_not_chosen_and_none_is_left_when_every_segment_is_excluded() {
+    let records = records_with_live_slots_sixty_ten_and_zero();
+    assert_eq!(
+        choose_greedily(
+            &records,
+            &BTreeSet::from([SlotNumber(SECOND_SEGMENT)]),
+            None,
+            &BTreeSet::new(),
+        ),
+        Some(region_at(FIRST_SEGMENT))
+    );
+    assert_eq!(
+        choose_greedily(
+            &records,
+            &BTreeSet::from([SlotNumber(FIRST_SEGMENT), SlotNumber(SECOND_SEGMENT)]),
+            None,
+            &BTreeSet::new(),
+        ),
+        None
+    );
+}
+
+/// 段 50240 就是活意图的 R ⇒ 改选段 50176（判决 G2「不在活 R」）。
+#[test]
+fn the_live_intent_region_is_not_chosen_again() {
+    assert_eq!(
+        choose_greedily(
+            &records_with_live_slots_sixty_ten_and_zero(),
+            &BTreeSet::new(),
+            Some(region_at(SECOND_SEGMENT)),
+            &BTreeSet::new(),
+        ),
+        Some(region_at(FIRST_SEGMENT))
+    );
+}
+
+/// 段 50176 与段 50304 活槽都是 4：同分取起点槽号最小的段 50176；段 50240 有 6 个活槽、分更高。
+#[test]
+fn a_tie_goes_to_the_segment_with_the_lowest_start_slot() {
+    let mut records = Vec::new();
+    for (segment_start, units) in [(FIRST_SEGMENT, 2), (SECOND_SEGMENT, 3), (THIRD_SEGMENT, 2)] {
+        for unit in 0..units {
+            records.extend(records_on_both_devices(segment_start + 2 * unit, 2));
+        }
+    }
+    assert_eq!(
+        choose_greedily(&records, &BTreeSet::new(), None, &BTreeSet::new()),
+        Some(region_at(FIRST_SEGMENT))
+    );
+}
+
+/// 一块镜像：两盘同槽各一份单元、各一条记录，加中央映射。整理 R 固定取段 50240。
+#[derive(Default)]
+struct PoolUnderTest {
+    units: BTreeMap<SlotNumber, Vec<u8>>,
+    records: Vec<AllocationRecord>,
+    mapping_entries: Vec<CentralMappingEntry>,
+}
+
+fn locations_at(slot: SlotNumber) -> [LocationEntry; 2] {
+    [DEVICE_ZERO, DEVICE_ONE].map(|device| LocationEntry {
+        device,
+        slot,
+        unit_checksum: 0,
+    })
+}
+
+impl PoolUnderTest {
+    fn place(&mut self, slot: u64, span_slots: u16, unit: Vec<u8>) {
+        self.records
+            .extend(records_on_both_devices(slot, span_slots));
+        self.units.insert(SlotNumber(slot), unit);
+    }
+
+    fn map(&mut self, key: &[u8], slot: u64) {
+        self.mapping_entries.push(CentralMappingEntry {
+            key: key.to_vec(),
+            locations: locations_at(SlotNumber(slot)),
+        });
+    }
+
+    /// 放一个可搬的单元：记录、字节、映射条目指着它。
+    fn place_movable(
+        &mut self,
+        slot: u64,
+        span_slots: u16,
+        (unit, key): (Vec<u8>, Vec<u8>),
+    ) -> Vec<u8> {
+        self.place(slot, span_slots, unit);
+        self.map(&key, slot);
+        key
+    }
+
+    fn scan(
+        &self,
+        pinned_slots: &BTreeSet<SlotNumber>,
+        quarantined_slots: &BTreeSet<SlotNumber>,
+    ) -> RegionScan {
+        scan_region(
+            region_at(SECOND_SEGMENT),
+            &self.records,
+            &self.mapping_entries,
+            pinned_slots,
+            quarantined_slots,
+            |_device, slot, unit_bytes| {
+                self.units
+                    .get(&slot)
+                    .filter(|bytes| bytes.len() == unit_bytes)
+                    .cloned()
+            },
+        )
+        .expect("没有跨 R 边界的记录")
+    }
+
+    /// 这一批发布之后的样子（模拟，不经发布路径）：每个搬走的 key 的映射条目改指 `destination_slots` 里依次取的槽（R 外），
+    /// 源落点两盘的记录改成已释放。
+    fn relocate(
+        &mut self,
+        batch: &[PlannedRelocation],
+        destination_slots: &mut impl Iterator<Item = u64>,
+    ) {
+        for relocation in batch {
+            let destination = destination_slots.next().expect("R 外的目的槽够用");
+            let entry = self
+                .mapping_entries
+                .iter_mut()
+                .find(|entry| entry.key == relocation.mapping_key)
+                .expect("计划里的 key 都在映射里");
+            entry.locations = locations_at(SlotNumber(destination));
+            for record in self
+                .records
+                .iter_mut()
+                .filter(|record| record.slot == relocation.source.slot)
+            {
+                record.is_released = true;
+            }
+        }
+    }
+}
+
+fn data_unit(transaction: u64) -> (Vec<u8>, Vec<u8>) {
+    let write_order = WriteOrder {
+        instance: WRITING_INSTANCE,
+        transaction,
+    };
+    let bytes = build_data_unit(
+        DataUnitIdentity {
+            tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+            object: 1,
+            object_birth: CheckpointTxg(3),
+            anchor_offset: 0,
+        },
+        BIRTH_TXG,
+        &FILESYSTEM_IDENTIFIER,
+        write_order,
+        &[0x5A; 100],
+    );
+    let key = mapping_key_for_data(
+        PointerHead {
+            birth_tree: TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+            birth_txg: BIRTH_TXG,
+        },
+        write_order,
+    );
+    (bytes, key)
+}
+
+fn node_pointer(tree: u64, birth_sequence: u32) -> NodePointer {
+    NodePointer {
+        head: PointerHead {
+            birth_tree: TreeIdentifier(tree),
+            birth_txg: BIRTH_TXG,
+        },
+        locations: NodePointer::empty_root().locations,
+        instance: WRITING_INSTANCE,
+        birth_sequence: BirthSequence(birth_sequence),
+    }
+}
+
+fn index_node(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
+    let bytes = build_index_node(
+        TreeIdentifier(TREE_IDENTIFIER_EXTENT),
+        0,
+        24,
+        &[0u8; 24],
+        &[0u8; 24],
+        BIRTH_TXG,
+        &FILESYSTEM_IDENTIFIER,
+        WRITING_INSTANCE,
+        BirthSequence(birth_sequence),
+        112,
+        &[],
+    );
+    let key = mapping_key_for_node(
+        UNIT_CLASS_INDEX_NODE,
+        node_pointer(TREE_IDENTIFIER_EXTENT, birth_sequence),
+    );
+    (bytes, key)
+}
+
+fn packed_unit(birth_sequence: u32) -> (Vec<u8>, Vec<u8>) {
+    let bytes = build_packed_unit(
+        PackedIdentity {
+            birth_tree: TreeIdentifier(TREE_IDENTIFIER_INODE),
+            record_type: PACKED_TYPE_INODE,
+            container: 1,
+            container_birth: BIRTH_TXG,
+        },
+        140,
+        &[vec![1u8; 140]],
+        BIRTH_TXG,
+        &FILESYSTEM_IDENTIFIER,
+        WriteOrder {
+            instance: WRITING_INSTANCE,
+            transaction: 9,
+        },
+        BirthSequence(birth_sequence),
+    );
+    let key = mapping_key_for_node(
+        UNIT_CLASS_PACKED,
+        node_pointer(TREE_IDENTIFIER_INODE, birth_sequence),
+    );
+    (bytes, key)
+}
+
+fn three_units() -> CompactionBatchUnits {
+    CompactionBatchUnits::new(3).expect("3 不是 0")
+}
+
+/// 锚点 4（只钉条数与 key 集合）：K = 3、R 里三个可搬的两槽数据单元 ⇒ 一批恰好三条，key 集合就是那三把，源落点是它们的落点。
+#[test]
+fn a_batch_of_three_over_three_movable_data_units_carries_exactly_their_keys() {
+    let mut pool = PoolUnderTest::default();
+    let mut expected_keys = BTreeSet::new();
+    for unit in 0..3 {
+        expected_keys.insert(pool.place_movable(SECOND_SEGMENT + 2 * unit, 2, data_unit(unit + 1)));
+    }
+
+    let batch = plan_batch(
+        &pool.scan(&BTreeSet::new(), &BTreeSet::new()),
+        three_units(),
+    );
+
+    assert_eq!(batch.len(), 3, "一批三条");
+    assert_eq!(
+        batch
+            .iter()
+            .map(|relocation| relocation.mapping_key.clone())
+            .collect::<BTreeSet<_>>(),
+        expected_keys
+    );
+    assert_eq!(
+        batch
+            .iter()
+            .map(|relocation| relocation.source)
+            .collect::<Vec<_>>(),
+        [0, 2, 4].map(|offset| Placement {
+            slot: SlotNumber(SECOND_SEGMENT + offset),
+            span: 2,
+        }),
+        "源落点按槽升序"
+    );
+    assert_eq!(CompactionBatchUnits::new(0), None, "一批 0 个造不出来");
+}
+
+/// 守恒：R 里 7 个可搬（码 1 × 4、码 2 × 2、码 3 × 1）与钉住、隔离、已死、不一致各一个，K = 3 一批批搬：
+/// 批大小 3、3、1，每批 ≤ K、批间不交，并集 = 7 把可搬的 key；搬完之后再计划是空批，剩下的四类原样还在。
+#[test]
+fn batches_never_overlap_and_their_union_is_every_movable_key_of_the_region() {
+    let mut pool = PoolUnderTest::default();
+    let mut movable_keys = BTreeSet::new();
+    for unit in 0..4 {
+        movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 2 * unit, 2, data_unit(unit + 1)));
+    }
+    movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 8, 1, index_node(0)));
+    movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 9, 1, index_node(1)));
+    movable_keys.insert(pool.place_movable(SECOND_SEGMENT + 10, 2, packed_unit(0)));
+    let (pinned, _pinned_key) = index_node(2);
+    pool.place(SECOND_SEGMENT + 12, 1, pinned);
+    let (quarantined, _quarantined_key) = data_unit(10);
+    pool.place(SECOND_SEGMENT + 14, 2, quarantined);
+    let (dead, _dead_key) = data_unit(11);
+    pool.place(SECOND_SEGMENT + 16, 2, dead);
+    let (mismatched, _mismatched_key) = data_unit(12);
+    let (_other, other_key) = data_unit(13);
+    pool.place(SECOND_SEGMENT + 18, 2, mismatched);
+    pool.map(&other_key, SECOND_SEGMENT + 18);
+    let pinned_slots = BTreeSet::from([SlotNumber(SECOND_SEGMENT + 12)]);
+    let quarantined_slots = BTreeSet::from([SlotNumber(SECOND_SEGMENT + 14)]);
+    let allocations_left_in_the_region = pool
+        .scan(&pinned_slots, &quarantined_slots)
+        .allocations
+        .len()
+        - movable_keys.len();
+    let mut destination_slots = (0..).map(|index| THIRD_SEGMENT + 2 * index);
+
+    let mut planned_keys = BTreeSet::new();
+    let mut batch_sizes = Vec::new();
+    // 迭代上界：7 个可搬、每批至少一个 ⇒ 至多 7 批，再加最后那一次空批。
+    for _batch_index in 0..8 {
+        let batch = plan_batch(&pool.scan(&pinned_slots, &quarantined_slots), three_units());
+        if batch.is_empty() {
+            break;
+        }
+        assert!(batch.len() <= 3, "每批 ≤ K = 3，这一批 {}", batch.len());
+        for relocation in &batch {
+            assert!(
+                planned_keys.insert(relocation.mapping_key.clone()),
+                "批间不交：一把 key 只进一批"
+            );
+        }
+        batch_sizes.push(batch.len());
+        pool.relocate(&batch, &mut destination_slots);
+    }
+
+    assert_eq!(batch_sizes, vec![3, 3, 1], "7 个可搬、K = 3");
+    assert_eq!(planned_keys, movable_keys, "并集 = R 里可搬的 key 集合");
+    let final_scan = pool.scan(&pinned_slots, &quarantined_slots);
+    assert_eq!(
+        final_scan.movable_units().count(),
+        0,
+        "搬完之后 R 里没有可搬的"
+    );
+    assert_eq!(
+        final_scan.allocations.len(),
+        allocations_left_in_the_region,
+        "钉住、隔离、已死、不一致四个原样还在 R 里"
+    );
+    assert_eq!(allocations_left_in_the_region, 4);
+}
diff --git a/crates/singlefs-harness/tests/logged_operations_intent_record_round_trips.rs b/crates/singlefs-harness/tests/logged_operations_intent_record_round_trips.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/logged_operations_intent_record_round_trips.rs
@@ -0,0 +1,190 @@
+//! logged_ops 树的整理意图条目（判决 `research/prompts/m4-gc-r1-main-verification.md` 第三节 G1）：
+//! 每个字段的偏移与宽度逐字节钉住、读回逐字段相等；key 按逐字段无符号整数比；一节点条数按 core 的节点容量式子现算；
+//! 读者的拒收各一条用例（不对齐的 R、槽数不是 64、补零非零、保留的种类码、条目宽不是 44）。
+//! 期望的字节按判决 G1 的字段表手写，不从被测的写者反推。
+
+use singlefs_core::address::{CheckpointTxg, DeviceIdentity, SlotNumber};
+use singlefs_core::logged_operations::{
+    CompactionIntent, CompactionRegion, CompactionRegionRefusal, LoggedOperation,
+    LoggedOperationEntry, LoggedOperationEntryRefusal, LoggedOperationKey, LoggedOperationKind,
+    LOGGED_OPERATION_KIND_COMPACTION,
+};
+use singlefs_core::unit::index_node_entry_capacity;
+use singlefs_format::{
+    LOGGED_OPERATIONS_ENTRIES_PER_NODE, LOGGED_OPERATIONS_ENTRY_BYTES, LOGGED_OPERATIONS_KEY_BYTES,
+};
+
+/// R 取单元区里起点 50304 的聚簇段：50176 + 128 = 50304 = 786 × 64。
+const REGION_START_SLOT: u64 = 50304;
+
+fn compaction_entry() -> LoggedOperationEntry {
+    LoggedOperationEntry {
+        issued_txg: CheckpointTxg(0x0102_0304_0506_0708),
+        operation: LoggedOperation::Compaction(CompactionIntent {
+            region: CompactionRegion::of_one_cluster_segment(
+                DeviceIdentity(0x0A0B_0C0D),
+                SlotNumber(REGION_START_SLOT),
+            )
+            .expect("50304 是 64 的倍数"),
+            allocation_generation_at_issue: CheckpointTxg(0x1112_1314_1516_1718),
+            empty_cluster_segments_at_issue: 0x2122_2324_2526_2728,
+        }),
+    }
+}
+
+/// 判决 G1 的字段表逐段：种类 2 | 签发 txg 8 | R 的设备 4 | R 的起始槽 6 | R 的槽数 2 | c0 8 | segs0 8 | 补零 6，小端，共 44。
+#[test]
+fn every_field_of_a_compaction_intent_sits_at_its_offset_and_reads_back() {
+    let entry = compaction_entry();
+    let bytes = entry.to_bytes();
+    let start_slot_bytes = REGION_START_SLOT.to_le_bytes();
+    let expected = [
+        vec![0x01, 0x00],
+        vec![0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
+        vec![0x0D, 0x0C, 0x0B, 0x0A],
+        start_slot_bytes[..6].to_vec(),
+        vec![64, 0],
+        vec![0x18, 0x17, 0x16, 0x15, 0x14, 0x13, 0x12, 0x11],
+        vec![0x28, 0x27, 0x26, 0x25, 0x24, 0x23, 0x22, 0x21],
+        vec![0; 6],
+    ]
+    .concat();
+    assert_eq!(
+        bytes, expected,
+        "种类 0 / 签发 txg 2 / 设备 10 / 起始槽 14 / 槽数 20 / c0 22 / segs0 30 / 补零 38"
+    );
+    assert_eq!(
+        u64::try_from(bytes.len()).expect("44"),
+        LOGGED_OPERATIONS_ENTRY_BYTES
+    );
+    assert_eq!(
+        LoggedOperationEntry::parse(&bytes),
+        Ok(entry),
+        "读回来逐字段相等"
+    );
+    assert_eq!(
+        entry.key(),
+        LoggedOperationKey {
+            kind: LoggedOperationKind::Compaction,
+            issued_txg: CheckpointTxg(0x0102_0304_0506_0708),
+        },
+        "key = 前 10 字节的两段"
+    );
+    assert_eq!(
+        u64::try_from(bytes[..10].len()).expect("10"),
+        LOGGED_OPERATIONS_KEY_BYTES
+    );
+}
+
+/// key 的全序按逐字段无符号整数比（D8（核心索引结构） 已定项 11）：签发 txg 1 在 256 之前，而两条条目的 key 字节按 memcmp 比是反的。
+#[test]
+fn logged_operation_keys_order_by_field_not_by_bytes() {
+    let entry_issued_at = |issued_txg: u64| LoggedOperationEntry {
+        issued_txg: CheckpointTxg(issued_txg),
+        operation: compaction_entry().operation,
+    };
+    let earlier = entry_issued_at(1);
+    let later = entry_issued_at(256);
+    assert!(
+        earlier.key().sort_key() < later.key().sort_key(),
+        "1 < 256 按整数比"
+    );
+    assert!(
+        earlier.to_bytes()[..10] > later.to_bytes()[..10],
+        "小端字节按 memcmp 比是反的，所以不能用它"
+    );
+}
+
+/// 一节点 369 条：core 的 `index_node_entry_capacity` 在 key 宽 10、条目宽 44 上现算，与格式常量一个数不差。
+#[test]
+fn a_logged_operations_node_holds_the_entries_the_node_capacity_formula_gives() {
+    let computed = index_node_entry_capacity(
+        usize::try_from(LOGGED_OPERATIONS_KEY_BYTES).expect("10"),
+        usize::try_from(LOGGED_OPERATIONS_ENTRY_BYTES).expect("44"),
+    );
+    assert_eq!(
+        u64::try_from(computed).expect("条数"),
+        LOGGED_OPERATIONS_ENTRIES_PER_NODE
+    );
+    assert_eq!(computed, 369, "(16384 − 135) ÷ 44 的整数部分");
+}
+
+fn bytes_with(offset: usize, replacement: &[u8]) -> Vec<u8> {
+    let mut bytes = compaction_entry().to_bytes();
+    bytes[offset..offset + replacement.len()].copy_from_slice(replacement);
+    bytes
+}
+
+/// R 的起始槽不是 64 的倍数：读者拒收（判决 G1「R 的解码拒不对齐的值」），写者那一侧也造不出这样的 R。
+#[test]
+fn a_compaction_region_whose_start_slot_is_off_a_cluster_segment_boundary_is_refused() {
+    let unaligned_start_slot = REGION_START_SLOT + 2;
+    assert_eq!(
+        LoggedOperationEntry::parse(&bytes_with(14, &unaligned_start_slot.to_le_bytes()[..6])),
+        Err(LoggedOperationEntryRefusal::CompactionRegionRefused(
+            CompactionRegionRefusal::StartOffAClusterSegmentBoundary {
+                start_slot: SlotNumber(unaligned_start_slot),
+            }
+        ))
+    );
+    assert_eq!(
+        CompactionRegion::of_one_cluster_segment(
+            DeviceIdentity(0),
+            SlotNumber(unaligned_start_slot)
+        ),
+        Err(CompactionRegionRefusal::StartOffAClusterSegmentBoundary {
+            start_slot: SlotNumber(unaligned_start_slot),
+        })
+    );
+}
+
+/// R 的槽数不是 64：读者拒收。
+#[test]
+fn a_compaction_region_whose_slot_count_is_not_one_cluster_segment_is_refused() {
+    assert_eq!(
+        LoggedOperationEntry::parse(&bytes_with(20, &128u16.to_le_bytes())),
+        Err(
+            LoggedOperationEntryRefusal::CompactionRegionSlotCountIsNotOneClusterSegment {
+                slot_count: 128
+            }
+        )
+    );
+}
+
+/// 补零那 6 字节里有非零字节：读者拒收（照树表条目预留位「非零判不可用」的政策）。
+#[test]
+fn a_compaction_intent_with_a_nonzero_padding_byte_is_refused() {
+    assert_eq!(
+        LoggedOperationEntry::parse(&bytes_with(43, &[1])),
+        Err(LoggedOperationEntryRefusal::CompactionPaddingNotZero)
+    );
+}
+
+/// 种类码不是 1（0 与 2 都是保留码）：读者拒收（照树表 flags「未知位一律拒收」）。
+#[test]
+fn a_logged_operation_of_a_reserved_kind_is_refused() {
+    assert_eq!(LOGGED_OPERATION_KIND_COMPACTION, 1);
+    for reserved_kind_code in [0u16, 2] {
+        assert_eq!(
+            LoggedOperationEntry::parse(&bytes_with(0, &reserved_kind_code.to_le_bytes())),
+            Err(LoggedOperationEntryRefusal::UnknownKind {
+                kind_code: reserved_kind_code
+            }),
+            "种类码 {reserved_kind_code} 是保留码"
+        );
+    }
+}
+
+/// 条目宽（码 2 节点头里的盘上字段）不是 44：窄一字节、宽一字节都拒收，不按固定偏移切下去。
+#[test]
+fn a_logged_operation_entry_whose_width_is_not_the_field_table_is_refused() {
+    let bytes = compaction_entry().to_bytes();
+    assert_eq!(
+        LoggedOperationEntry::parse(&bytes[..43]),
+        Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable { entry_bytes: 43 })
+    );
+    assert_eq!(
+        LoggedOperationEntry::parse(&[bytes.as_slice(), &[0]].concat()),
+        Err(LoggedOperationEntryRefusal::WidthIsNotTheFieldTable { entry_bytes: 45 })
+    );
+}
````

## m4-gc-plan-writer：mutations-append.tsv

````tsv
整理意图条目少写 segs0（那 8 字节留成 0）	crates/singlefs-core/src/logged_operations.rs	                writer.put_u64(intent.empty_cluster_segments_at_issue);	                writer.skip(8);	-p singlefs-harness --test logged_operations_intent_record_round_trips -- every_field_of_a_compaction_intent_sits_at_its_offset_and_reads_back	every_field_of_a_compaction_intent_sits_at_its_offset_and_reads_back
整理意图的 R 起始槽不核 64 对齐	crates/singlefs-core/src/logged_operations.rs	        if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {	        if false {	-p singlefs-harness --test logged_operations_intent_record_round_trips -- a_compaction_region_whose_start_slot_is_off_a_cluster_segment_boundary_is_refused	a_compaction_region_whose_start_slot_is_off_a_cluster_segment_boundary_is_refused
整理意图的 R 槽数不核是不是 64	crates/singlefs-core/src/logged_operations.rs	    if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {	    if false {	-p singlefs-harness --test logged_operations_intent_record_round_trips -- a_compaction_region_whose_slot_count_is_not_one_cluster_segment_is_refused	a_compaction_region_whose_slot_count_is_not_one_cluster_segment_is_refused
整理意图的补零不核非零	crates/singlefs-core/src/logged_operations.rs	    if padding.iter().any(|byte| *byte != 0) {	    if false {	-p singlefs-harness --test logged_operations_intent_record_round_trips -- a_compaction_intent_with_a_nonzero_padding_byte_is_refused	a_compaction_intent_with_a_nonzero_padding_byte_is_refused
区域扫描不核映射位置（头的 key 在映射里别处也判已死）	crates/singlefs-core/src/region_scan.rs	    if entries_pointing_here.is_empty() && !header_key_is_mapped {	    if entries_pointing_here.is_empty() {	-p singlefs-core --lib -- region_scan::tests::a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead	region_scan::tests::a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead
区域扫描不核头里反查出的 key（指着这个槽的条目是别的 key 也判可搬）	crates/singlefs-core/src/region_scan.rs	        .all(|entry| entry.key == header.mapping_key);	        .all(|_entry| true);	-p singlefs-core --lib -- region_scan::tests::a_unit_whose_header_key_differs_from_the_mapping_entry_pointing_at_it_is_inconsistent	region_scan::tests::a_unit_whose_header_key_differs_from_the_mapping_entry_pointing_at_it_is_inconsistent
区域扫描不核类标签与映射 key	crates/singlefs-core/src/region_scan.rs	        .find(|entry| entry.key.first() != Some(&header.unit_class.tag()))	        .find(|_entry| false)	-p singlefs-core --lib -- region_scan::tests::an_unreadable_header_or_a_class_tag_unlike_the_mapping_key_is_inconsistent	region_scan::tests::an_unreadable_header_or_a_class_tag_unlike_the_mapping_key_is_inconsistent
选段不看钉住的槽	crates/singlefs-core/src/compaction_plan.rs	                && !holds_a_pinned_slot	                && (holds_a_pinned_slot || true)	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- a_segment_holding_a_pinned_slot_is_not_chosen	a_segment_holding_a_pinned_slot_is_not_chosen
选段不排开这次挂载开过的聚簇段	crates/singlefs-core/src/compaction_plan.rs	            !opened_in_this_mount	            (opened_in_this_mount || true)	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- a_segment_opened_in_this_mount_is_not_chosen_and_none_is_left_when_every_segment_is_excluded	a_segment_opened_in_this_mount_is_not_chosen_and_none_is_left_when_every_segment_is_excluded
选段不排开活意图的 R	crates/singlefs-core/src/compaction_plan.rs	                && !is_the_live_intent_region	                && (is_the_live_intent_region || true)	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- the_live_intent_region_is_not_chosen_again	the_live_intent_region_is_not_chosen_again
选段 greedy 打分方向反（活槽最多的先）	crates/singlefs-core/src/compaction_plan.rs	            .min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))	            .max_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- greedy_chooses_the_segment_with_the_fewest_live_slots_but_not_an_empty_one	greedy_chooses_the_segment_with_the_fewest_live_slots_but_not_an_empty_one
选段收没有活槽的全空段	crates/singlefs-core/src/compaction_plan.rs	                && *live_slots > 0	                && *live_slots >= 0	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- greedy_chooses_the_segment_with_the_fewest_live_slots_but_not_an_empty_one	greedy_chooses_the_segment_with_the_fewest_live_slots_but_not_an_empty_one
计划一批多取一个（K 差一）	crates/singlefs-core/src/compaction_plan.rs	        .take(batch.units())	        .take(batch.units() + 1)	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- batches_never_overlap_and_their_union_is_every_movable_key_of_the_region	batches_never_overlap_and_their_union_is_every_movable_key_of_the_region
logged_ops 保留的种类码当整理收	crates/singlefs-core/src/logged_operations.rs	            _reserved_kind_code => None,	            _reserved_kind_code => Some(LoggedOperationKind::Compaction),	-p singlefs-harness --test logged_operations_intent_record_round_trips -- a_logged_operation_of_a_reserved_kind_is_refused	a_logged_operation_of_a_reserved_kind_is_refused
logged_ops 条目宽只拒窄的（宽于 44 照收）	crates/singlefs-core/src/logged_operations.rs	        if bytes.len() != entry_bytes() {	        if bytes.len() < entry_bytes() {	-p singlefs-harness --test logged_operations_intent_record_round_trips -- a_logged_operation_entry_whose_width_is_not_the_field_table_is_refused	a_logged_operation_entry_whose_width_is_not_the_field_table_is_refused
logged_ops key 的签发 txg 按字节序比	crates/singlefs-core/src/logged_operations.rs	        (self.kind.code(), self.issued_txg.0)	        (self.kind.code(), self.issued_txg.0.swap_bytes())	-p singlefs-harness --test logged_operations_intent_record_round_trips -- logged_operation_keys_order_by_field_not_by_bytes	logged_operation_keys_order_by_field_not_by_bytes
logged_ops 一节点条数常量差一（370）	crates/singlefs-format/src/lib.rs	pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 369;	pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 370;	-p singlefs-harness --test logged_operations_intent_record_round_trips -- a_logged_operations_node_holds_the_entries_the_node_capacity_formula_gives	a_logged_operations_node_holds_the_entries_the_node_capacity_formula_gives
区域扫描不看钉住的槽	crates/singlefs-core/src/region_scan.rs	    if pinned_slots.range(covered_slots.clone()).next().is_some() {	    if false {	-p singlefs-core --lib -- region_scan::tests::every_class_of_the_region_scan_is_pinned_down_slot_by_slot	region_scan::tests::every_class_of_the_region_scan_is_pinned_down_slot_by_slot
区域扫描放过跨 R 边界的记录	crates/singlefs-core/src/region_scan.rs	        if reaches_in_from_below || reaches_out_above {	        if false {	-p singlefs-core --lib -- region_scan::tests::a_record_straddling_the_region_boundary_is_refused_before_any_classification	region_scan::tests::a_record_straddling_the_region_boundary_is_refused_before_any_classification
选段同分取起点槽号最大的	crates/singlefs-core/src/compaction_plan.rs	            .min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))	            .min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), std::cmp::Reverse(*start_slot)))	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- a_tie_goes_to_the_segment_with_the_lowest_start_slot	a_tie_goes_to_the_segment_with_the_lowest_start_slot
计划一批少取一个（K 差一）	crates/singlefs-core/src/compaction_plan.rs	        .take(batch.units())	        .take(batch.units() - 1)	-p singlefs-harness --test compaction_plan_chooses_regions_and_batches_by_the_clauses -- a_batch_of_three_over_three_movable_data_units_carries_exactly_their_keys	a_batch_of_three_over_three_movable_data_units_carries_exactly_their_keys
logged_ops 一节点条数常量差一（格式常量模块自己的单测）	crates/singlefs-format/src/lib.rs	pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 369;	pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 370;	-p singlefs-format --lib -- tests::logged_operations_widths_equal_the_field_sums_and_the_node_capacity_they_stand_for	tests::logged_operations_widths_equal_the_field_sums_and_the_node_capacity_they_stand_for
````

## m4-discard-writer：crates.patch

````diff
diff --git a/crates/singlefs-core/src/block_device.rs b/crates/singlefs-core/src/block_device.rs
--- a/crates/singlefs-core/src/block_device.rs
+++ b/crates/singlefs-core/src/block_device.rs
@@ -1,4 +1,4 @@
-//! 块设备抽象：读、写、屏障、FUA 写、探测 `physical_block_size`、写零。
+//! 块设备抽象：读、写、屏障、FUA 写、探测 `physical_block_size`、写零、discard。
 //!
 //! 第一版两个后端（D17（实现分层与第三方管道） 已定项 5）：文件后端（测试镜像）与 `O_DIRECT` 后端（虚机里的真块设备）。
 //! 两个后端的持久语义相同：屏障 = `sync_data`（块设备上是一次 FLUSH），FUA 写 = 写完立刻 `sync_data`。
@@ -14,6 +14,12 @@
 //! 读回全 0、文件长度不变与真写 0 相同；持久同样由后面那次屏障给：打洞是元数据改动，ext4 的 `ext4_punch_hole`
 //! 把它记进 `fdatasync` 要等的那个日志事务（本机 6.17 内核源码 `fs/ext4/inode.c` 里调 `ext4_update_inode_fsync_trans(handle, inode, 1)`）。
 //! `O_DIRECT` 后端照旧真写 0：真设备上的块层计数与设备日志比对（checker 档 `device_log.rs` 的折叠）靠的是那些写。
+//!
+//! discard 是第七个动作（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「trait 形态」；D26（后台整理与放置回收）
+//! 未定项 9「还缺一个前提」：盘的 FTL 只有收到 discard 才知道哪些块不用了）。它与写零的差别在**结局由设备定**：
+//! 持久之后那一段读回 0 还是旧字节由设备定，模型（崩溃点重放）按「范围写零」枚举两态（D13（验证路线） 已定项 4：持久 → 0、没持久 → 旧字节）。
+//! 后端不认 discard 时交回 [`DiscardOutcome::Unsupported`]、一个字节不动；文件后端复用打洞，`O_DIRECT` 后端第一版不认（`BLKDISCARD` ioctl 留到真盘线）。
+//! 谁发、什么时候发（分配器的可发谓词、发布返回之后发）不在这一层。
 
 use std::fs::{File, OpenOptions};
 use std::io::{self, Seek, SeekFrom};
@@ -32,6 +38,38 @@
     ForceUnitAccess,
 }
 
+/// 一次 [`BlockDevice::discard_at`] 在这块盘上落成了什么（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测，调用方按它数）。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DiscardOutcome {
+    /// 后端把这一段交给了底层：之后读这一段可能是 0、也可能还是旧字节，持久之后是哪一个由设备定。
+    Applied,
+    /// 后端不认 discard：这一段一个字节没动，读回仍是旧字节。不是错——discard 只是告诉盘「这一段不用了」。
+    Unsupported,
+}
+
+/// 一个句柄上的 discard 各交回了哪一种结局：两项之和就是这个句柄上交回 `Ok` 的 discard 次数。
+#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
+pub struct DiscardOutcomeCounts {
+    pub applied_calls: u64,
+    pub unsupported_calls: u64,
+}
+
+impl DiscardOutcomeCounts {
+    /// 一次也没交回过。
+    pub const NONE: DiscardOutcomeCounts = DiscardOutcomeCounts {
+        applied_calls: 0,
+        unsupported_calls: 0,
+    };
+
+    /// 记下一次交回的结局。
+    pub fn note(&mut self, outcome: DiscardOutcome) {
+        match outcome {
+            DiscardOutcome::Applied => self.applied_calls += 1,
+            DiscardOutcome::Unsupported => self.unsupported_calls += 1,
+        }
+    }
+}
+
 /// 挂载时探测到的物理块大小；不许硬编码（D20（承重面：单元的原子性与自包含） 推论三）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 pub struct PhysicalBlockSizeInBytes(pub u32);
@@ -111,7 +149,10 @@
 /// 比它大只再快 1–2 ms 而暂存区从 4 MiB 长到 64 MiB（`O_DIRECT` 那条路每次调用要一块对齐暂存区）。
 pub const ZERO_FILL_CHUNK_BYTES: u64 = 4 * 1024 * 1024;
 
-/// 六个动作，只有这六个。介质是什么这里看不见。
+/// 七个动作，只有这七个（与文件头那一行同一份清单：读、写、屏障、FUA 写、探测物理块大小、写零、discard）。介质是什么这里看不见。
+///
+/// 第七个是 [`BlockDevice::discard_at`]：持久之后那一段读回 0 还是旧字节由设备定（不像写零那样必定是 0），
+/// 模型按「范围写零」枚举（D13（验证路线） 已定项 4 的两态：持久 → 0、没持久 → 旧字节）。
 pub trait BlockDevice {
     fn read_at(
         &self,
@@ -138,6 +179,30 @@
         offset: DeviceOffsetInBytes,
         length: u64,
     ) -> Result<(), BlockDeviceError>;
+    /// discard：告诉盘 `[offset, offset + length)` 这一段不用了。偏移与长度同样要按物理块对齐、不越过设备末尾，不合就报
+    /// [`BlockDeviceError::Unaligned`] / [`BlockDeviceError::OutOfRange`]、一个字节不动。
+    ///
+    /// 交回 [`DiscardOutcome::Applied`] 之后，那一段在持久之前读回旧字节或 0 都可能，持久之后读回 0 还是旧字节由设备定；
+    /// 调用方不许指望读回什么（D13（验证路线） 已定项 4：模型按「范围写零」枚举持久 / 没持久两态）。
+    /// 与写零一样没有 [`WriteDurability`] 参数：持久由后面的屏障给。
+    ///
+    /// **默认实现只给不认 discard 的后端**：核过请求就交回 [`DiscardOutcome::Unsupported`]，一个字节不动。
+    /// 默认实现没有状态、记不了数：「交回了几次不支持」由调用方按返回值数，或由覆盖了它的后端自己数（[`DiscardOutcomeCounts`]）。
+    /// 产品后端必须覆盖它（文件后端、`O_DIRECT` 后端都覆盖了）；包在别的设备外面的包装要转给里面那块盘，
+    /// 不覆盖就把里面那块盘认得的 discard 吞成「不支持」。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        check_aligned_and_in_range(
+            offset,
+            length,
+            self.probe_physical_block_size(),
+            self.size_in_bytes(),
+        )?;
+        Ok(DiscardOutcome::Unsupported)
+    }
     /// 屏障：之前发出的写全部持久之后才返回。
     fn barrier(&mut self) -> Result<(), BlockDeviceError>;
     fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes;
@@ -175,6 +240,25 @@
     },
 }
 
+/// 文件后端的 discard 怎么落到镜像文件上：复用写零的打洞（`punch_hole_keeping_the_file_length`）与同一段按 errno 分路的判定
+/// （`classify_hole_punch_failure`）；与写零不同的只在「不支持打洞」那一路：discard 不退回逐块写 0，交回 [`DiscardOutcome::Unsupported`]。
+#[derive(Clone, Copy, Debug, PartialEq, Eq)]
+pub enum DiscardLanding {
+    /// 打洞；报不支持打洞交回 [`DiscardOutcome::Unsupported`]、一个字节不动，被信号打断原样重打，报别的错照常交回
+    /// [`BlockDeviceError::InputOutput`]。
+    PunchHoleOrAnswerUnsupported,
+    /// 只供测试的开关（`.claude/rules/fs-design.md`「五条硬要求」第 2 条）：不调 `fallocate`，当作它报了 `EOPNOTSUPP`，
+    /// 与真失败走同一段判定，落进「不支持」那条路。
+    ActAsIfPunchingWereUnsupported,
+    /// 只供测试的开关：不调 `fallocate`，当作它报了 `EIO`，落进照常报错那条路。
+    ActAsIfPunchingFailedWithAnInputOutputError,
+    /// 只供测试的开关：这一次 discard 的前 `punches_interrupted_before_the_real_one` 次打洞不调 `fallocate`，每次都当作它报了 `EINTR`；
+    /// 之后那一次照 [`DiscardLanding::PunchHoleOrAnswerUnsupported`] 真调 `fallocate`。
+    ActAsIfTheFirstPunchesWereInterruptedBySignal {
+        punches_interrupted_before_the_real_one: u64,
+    },
+}
+
 /// 打洞失败之后走哪条路，按 `fallocate` 交回的 errno 分（用户 2026-09-30 定「只在不支持时退」）。
 #[derive(Clone, Copy, Debug, PartialEq, Eq)]
 enum HolePunchFailure {
@@ -215,8 +299,15 @@
     physical_block_size: PhysicalBlockSizeInBytes,
     zero_fill_landing: ZeroFillLanding,
     zero_fill_landing_counts: ZeroFillLandingCounts,
+    discard_landing: DiscardLanding,
+    discard_outcome_counts: DiscardOutcomeCounts,
+    /// discard 的打洞被信号打断、原样重打的次数（按打洞的次数数，不按 discard 的次数数）。
+    discard_punch_attempts_interrupted_by_signal: u64,
 }
 
+/// 两个构造入口起步时一样：discard 先打洞。
+const INITIAL_DISCARD_LANDING: DiscardLanding = DiscardLanding::PunchHoleOrAnswerUnsupported;
+
 /// 两个构造入口起步时一样：先打洞，计数从 0 起。
 const INITIAL_ZERO_FILL_LANDING: ZeroFillLanding =
     ZeroFillLanding::PunchHoleOrWriteZeroBytesIfUnsupported;
@@ -237,6 +328,26 @@
             physical_block_size: self.physical_block_size,
             zero_fill_landing: landing,
             zero_fill_landing_counts: self.zero_fill_landing_counts,
+            discard_landing: self.discard_landing,
+            discard_outcome_counts: self.discard_outcome_counts,
+            discard_punch_attempts_interrupted_by_signal: self
+                .discard_punch_attempts_interrupted_by_signal,
+        }
+    }
+
+    /// 只供测试：换掉 discard 的落地方式（[`DiscardLanding`] 的三个 `ActAsIf…` 开关各强制走一条路）。
+    #[must_use]
+    pub fn with_test_only_discard_landing(self, landing: DiscardLanding) -> Self {
+        Self {
+            file: self.file,
+            size_in_bytes: self.size_in_bytes,
+            physical_block_size: self.physical_block_size,
+            zero_fill_landing: self.zero_fill_landing,
+            zero_fill_landing_counts: self.zero_fill_landing_counts,
+            discard_landing: landing,
+            discard_outcome_counts: self.discard_outcome_counts,
+            discard_punch_attempts_interrupted_by_signal: self
+                .discard_punch_attempts_interrupted_by_signal,
         }
     }
 
@@ -245,6 +356,18 @@
         self.zero_fill_landing_counts
     }
 
+    /// 这个句柄上的 discard 各交回了哪一种结局（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测）。
+    #[must_use]
+    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
+        self.discard_outcome_counts
+    }
+
+    /// 这个句柄上 discard 的打洞被信号打断、原样重打了几次。
+    #[must_use]
+    pub fn discard_punch_attempts_interrupted_by_signal(&self) -> u64 {
+        self.discard_punch_attempts_interrupted_by_signal
+    }
+
     /// 排他地建出一个镜像文件并把它撑到 `size_in_bytes`；稀疏文件，不真占盘。
     /// 同名文件已经在那儿了就报 [`BlockDeviceError::ImageFileAlreadyExists`]，不接着用旧镜像：
     /// 测试的镜像名带进程号，而进程号会被系统复用、上一轮被杀留下的文件不会被清——接着用就是拿上一轮的盘面当这一轮的起点，
@@ -276,6 +399,9 @@
             physical_block_size,
             zero_fill_landing: INITIAL_ZERO_FILL_LANDING,
             zero_fill_landing_counts: NO_ZERO_FILL_LANDED_YET,
+            discard_landing: INITIAL_DISCARD_LANDING,
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
+            discard_punch_attempts_interrupted_by_signal: 0,
         })
     }
 
@@ -309,6 +435,9 @@
             physical_block_size,
             zero_fill_landing: INITIAL_ZERO_FILL_LANDING,
             zero_fill_landing_counts: NO_ZERO_FILL_LANDED_YET,
+            discard_landing: INITIAL_DISCARD_LANDING,
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
+            discard_punch_attempts_interrupted_by_signal: 0,
         })
     }
 
@@ -399,8 +528,8 @@
         libc::off_t::try_from(length).map_err(|_| io::Error::from(io::ErrorKind::InvalidInput))?;
     // SAFETY: `fallocate` 只读它的四个整数参数，不碰进程内存；描述符取自 `file`，
     // 而 `file` 借用到这次调用返回，调用期间不会被关掉或换成别的文件。偏移与长度由上面的 `try_from`
-    // 转成非负的 `off_t`，放不下就在转换处拒；越界的请求到不了这里：唯一的调用方 `write_zeroes_at` 先过 `check_request`
-    // （对齐、落在设备范围内），长度 0 也在 `write_zeroes_at` 里先交回、不到这里。
+    // 转成非负的 `off_t`，放不下就在转换处拒；越界的请求到不了这里：两个调用方 `write_zeroes_at` 与 `discard_at` 都先过 `check_request`
+    // （对齐、落在设备范围内），长度 0 也在它们里面先交回、不到这里。
     // 跨过文件末尾的打洞，文件内那一半照常打成 0，文件外的部分内核不做，都不碰内存。返回值在下面查了。
     let return_code = unsafe {
         libc::fallocate(
@@ -514,6 +643,69 @@
         }
     }
 
+    /// 复用写零的打洞：打成洞之后读这一段是全 0，文件长度不变（D13（验证路线） 已定项 4 的「持久 → 0」那一态在镜像文件上就是它）。
+    /// 文件系统不支持打洞时交回 [`DiscardOutcome::Unsupported`]、不退回逐块写 0（discard 不承诺读回 0，写 0 只是白写）；
+    /// 被信号打断原样重打；别的错照常报 [`BlockDeviceError::InputOutput`]。
+    fn discard_at(
+        &mut self,
+        discarded_offset: DeviceOffsetInBytes,
+        discarded_length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        self.check_request(discarded_offset, discarded_length)?;
+        // 长度 0：`fallocate(2)` 对它报 `EINVAL`，而没有一个字节要告诉盘不用了；照「交给了底层」交回，与内存稀疏盘结局相同。
+        if discarded_length == 0 {
+            self.discard_outcome_counts.note(DiscardOutcome::Applied);
+            return Ok(DiscardOutcome::Applied);
+        }
+        // 循环：每一轮打一次洞。跨轮的状态只有这一次 discard 里被信号打断的次数（只增）；
+        // 出口三个：打成洞、报不支持、报别的错。被打断就原样再打，轮数的上界是这期间送到的信号数。
+        let mut interrupted_in_this_discard: u64 = 0;
+        loop {
+            let punch_result = match self.discard_landing {
+                DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
+                    punches_interrupted_before_the_real_one,
+                } if interrupted_in_this_discard < punches_interrupted_before_the_real_one => {
+                    Err(io::Error::from_raw_os_error(libc::EINTR))
+                }
+                DiscardLanding::PunchHoleOrAnswerUnsupported
+                | DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal { .. } => {
+                    punch_hole_keeping_the_file_length(
+                        &self.file,
+                        discarded_offset,
+                        discarded_length,
+                    )
+                }
+                DiscardLanding::ActAsIfPunchingWereUnsupported => {
+                    Err(io::Error::from_raw_os_error(libc::EOPNOTSUPP))
+                }
+                DiscardLanding::ActAsIfPunchingFailedWithAnInputOutputError => {
+                    Err(io::Error::from_raw_os_error(libc::EIO))
+                }
+            };
+            let discard_punch_error = match punch_result {
+                Ok(()) => {
+                    self.discard_outcome_counts.note(DiscardOutcome::Applied);
+                    return Ok(DiscardOutcome::Applied);
+                }
+                Err(discard_punch_error) => discard_punch_error,
+            };
+            match classify_hole_punch_failure(&discard_punch_error) {
+                HolePunchFailure::PunchingUnsupported => {
+                    self.discard_outcome_counts
+                        .note(DiscardOutcome::Unsupported);
+                    return Ok(DiscardOutcome::Unsupported);
+                }
+                HolePunchFailure::InterruptedBySignal => {
+                    interrupted_in_this_discard += 1;
+                    self.discard_punch_attempts_interrupted_by_signal += 1;
+                }
+                HolePunchFailure::OtherFailure => {
+                    return Err(BlockDeviceError::InputOutput(discard_punch_error));
+                }
+            }
+        }
+    }
+
     fn barrier(&mut self) -> Result<(), BlockDeviceError> {
         self.file.sync_data().map_err(BlockDeviceError::InputOutput)
     }
@@ -618,6 +810,7 @@
     size_in_bytes: u64,
     physical_block_size: PhysicalBlockSizeInBytes,
     policy: PageCachePolicy,
+    discard_outcome_counts: DiscardOutcomeCounts,
 }
 
 impl DirectInputOutputBlockDevice {
@@ -652,8 +845,15 @@
             size_in_bytes,
             physical_block_size,
             policy,
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
         })
     }
+
+    /// 这个句柄上的 discard 各交回了哪一种结局；第一版每一次都是「不支持」（[`BlockDevice::discard_at`] 在这个后端上的文档）。
+    #[must_use]
+    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
+        self.discard_outcome_counts
+    }
 }
 
 impl BlockDevice for DirectInputOutputBlockDevice {
@@ -734,6 +934,20 @@
         })
     }
 
+    /// 第一版不认 discard：核过请求就交回 [`DiscardOutcome::Unsupported`]，一个字节不动、不发系统调用，两臂（绕开 / 走页缓存）一样。
+    /// 真块设备上的 discard 是 `BLKDISCARD` ioctl，留到真盘线（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「真盘 ioctl 留到真盘线」）；
+    /// 在那之前虚机里的设备侧日志里不会有 Discard 条目，宿主重跑那一侧要照这个结局投（checker 档 `device_log.rs`）。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        check_aligned_and_in_range(offset, length, self.physical_block_size, self.size_in_bytes)?;
+        self.discard_outcome_counts
+            .note(DiscardOutcome::Unsupported);
+        Ok(DiscardOutcome::Unsupported)
+    }
+
     fn barrier(&mut self) -> Result<(), BlockDeviceError> {
         self.file.sync_data().map_err(BlockDeviceError::InputOutput)
     }
@@ -1592,4 +1806,211 @@
             assert_eq!(scratch.as_slice().len(), length);
         }
     }
+
+    /// discard 用例的镜像：64 KiB、整个先填成 0xC3（稀疏镜像本来就读回全 0，不先弄脏就分不出「打过洞」与「从来没写过」），
+    /// 换上 `landing` 那一种 discard 落地方式。
+    const DISCARD_IMAGE_BYTES: u64 = 65536;
+    const DISCARD_DIRT_BYTE: u8 = 0xC3;
+
+    fn dirty_image_for_discards(
+        tag: &str,
+        landing: DiscardLanding,
+    ) -> (FileBackedBlockDevice, ImageRemovedOnDrop) {
+        let path = temporary_image_path(tag);
+        let mut device = FileBackedBlockDevice::create_image_file_exclusively(
+            &path,
+            DISCARD_IMAGE_BYTES,
+            PhysicalBlockSizeInBytes(512),
+        )
+        .expect("建镜像");
+        let dirt = vec![DISCARD_DIRT_BYTE; usize::try_from(DISCARD_IMAGE_BYTES).expect("64 KiB")];
+        device
+            .write_at(DeviceOffsetInBytes(0), &dirt, WriteDurability::Plain)
+            .expect("填脏");
+        (
+            device.with_test_only_discard_landing(landing),
+            ImageRemovedOnDrop(path),
+        )
+    }
+
+    fn sector_at(device: &FileBackedBlockDevice, offset: u64) -> Vec<u8> {
+        let mut sector = vec![0x5Au8; 512];
+        device
+            .read_at(DeviceOffsetInBytes(offset), &mut sector)
+            .expect("读一扇区");
+        sector
+    }
+
+    /// 文件后端的 discard 复用打洞：段内读回全 0、段外一个字节不动、文件长度不变，交回「交给了底层」并记一次。
+    #[test]
+    fn a_discard_on_the_file_backend_punches_the_range_back_to_zero_and_leaves_the_neighbours() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-punched",
+            DiscardLanding::PunchHoleOrAnswerUnsupported,
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 16384)
+            .expect("discard");
+        assert_eq!(outcome, DiscardOutcome::Applied);
+        for offset in [8192u64, 8192 + 16384 - 512] {
+            assert!(
+                sector_at(&device, offset).iter().all(|byte| *byte == 0),
+                "偏移 {offset} 在 discard 段内，打过洞读回全 0"
+            );
+        }
+        for offset in [8192u64 - 512, 8192 + 16384] {
+            assert!(
+                sector_at(&device, offset)
+                    .iter()
+                    .all(|byte| *byte == DISCARD_DIRT_BYTE),
+                "偏移 {offset} 在 discard 段外，不该被动"
+            );
+        }
+        assert_eq!(
+            device.file.metadata().expect("读文件元数据").len(),
+            DISCARD_IMAGE_BYTES,
+            "打洞不改文件长度"
+        );
+        assert_eq!(
+            device.discard_outcome_counts(),
+            DiscardOutcomeCounts {
+                applied_calls: 1,
+                unsupported_calls: 0
+            }
+        );
+        assert_eq!(
+            device.zero_fill_landing_counts(),
+            NO_ZERO_FILL_LANDED_YET,
+            "discard 不记进写零的落地计数"
+        );
+    }
+
+    /// 文件系统不支持打洞：交回「不支持」、一个 0 都不写（段里还是旧字节），记一次「不支持」。
+    #[test]
+    fn a_discard_where_hole_punching_is_unsupported_answers_unsupported_and_leaves_the_old_bytes() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-unsupported",
+            DiscardLanding::ActAsIfPunchingWereUnsupported,
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 16384)
+            .expect("discard");
+        assert_eq!(outcome, DiscardOutcome::Unsupported);
+        assert!(
+            sector_at(&device, 8192)
+                .iter()
+                .all(|byte| *byte == DISCARD_DIRT_BYTE),
+            "不支持时一个字节不动"
+        );
+        assert_eq!(
+            device.discard_outcome_counts(),
+            DiscardOutcomeCounts {
+                applied_calls: 0,
+                unsupported_calls: 1
+            }
+        );
+    }
+
+    /// 打洞报 `EIO`：照常交回块设备 I/O 错，不吞成「不支持」，段里还是旧字节，两种结局都不记。
+    #[test]
+    fn a_discard_whose_hole_punch_fails_with_an_input_output_error_is_reported_and_changes_nothing()
+    {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-input-output-error",
+            DiscardLanding::ActAsIfPunchingFailedWithAnInputOutputError,
+        );
+        let outcome = device.discard_at(DeviceOffsetInBytes(8192), 16384);
+        assert!(
+            matches!(&outcome, Err(BlockDeviceError::InputOutput(error)) if error.raw_os_error() == Some(libc::EIO)),
+            "EIO 照常报错：{outcome:?}"
+        );
+        assert!(sector_at(&device, 8192)
+            .iter()
+            .all(|byte| *byte == DISCARD_DIRT_BYTE));
+        assert_eq!(device.discard_outcome_counts(), DiscardOutcomeCounts::NONE);
+    }
+
+    /// 打洞连着被信号打断两次：原样重打到第三次，打成洞、交回「交给了底层」，被打断的次数记 2。
+    #[test]
+    fn a_discard_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-interrupted",
+            DiscardLanding::ActAsIfTheFirstPunchesWereInterruptedBySignal {
+                punches_interrupted_before_the_real_one: 2,
+            },
+        );
+        let outcome = device
+            .discard_at(DeviceOffsetInBytes(8192), 16384)
+            .expect("被打断之后重打成功");
+        assert_eq!(outcome, DiscardOutcome::Applied);
+        assert!(sector_at(&device, 8192).iter().all(|byte| *byte == 0));
+        assert_eq!(device.discard_punch_attempts_interrupted_by_signal(), 2);
+        assert_eq!(device.discard_outcome_counts().applied_calls, 1);
+    }
+
+    /// 没对齐、越过末尾的 discard 在动文件之前就拒，与读写同一条判据；段里还是旧字节。
+    #[test]
+    fn a_discard_off_the_block_boundaries_or_past_the_end_is_refused_before_touching_the_file() {
+        let (mut device, _removed) = dirty_image_for_discards(
+            "discard-refused",
+            DiscardLanding::PunchHoleOrAnswerUnsupported,
+        );
+        assert!(matches!(
+            device.discard_at(DeviceOffsetInBytes(100), 512),
+            Err(BlockDeviceError::Unaligned { .. })
+        ));
+        assert!(matches!(
+            device.discard_at(DeviceOffsetInBytes(DISCARD_IMAGE_BYTES - 512), 1024),
+            Err(BlockDeviceError::OutOfRange { .. })
+        ));
+        assert!(sector_at(&device, DISCARD_IMAGE_BYTES - 512)
+            .iter()
+            .all(|byte| *byte == DISCARD_DIRT_BYTE));
+        assert_eq!(device.discard_outcome_counts(), DiscardOutcomeCounts::NONE);
+    }
+
+    /// `O_DIRECT` 后端第一版不认 discard：两臂都交回「不支持」、一个字节不动、每次记一次；没对齐的照样拒。
+    #[test]
+    fn the_direct_backend_answers_every_discard_unsupported_on_both_arms_and_counts_it() {
+        for policy in [
+            PageCachePolicy::BypassWithDirectInputOutput,
+            PageCachePolicy::GoThroughPageCache,
+        ] {
+            let path = temporary_image_path("direct-discard");
+            let _removed = ImageRemovedOnDrop(path.clone());
+            let mut file = std::fs::File::create(&path).expect("建文件");
+            std::io::Write::write_all(&mut file, &[DISCARD_DIRT_BYTE; 65536]).expect("填脏");
+            drop(file);
+            let mut device = DirectInputOutputBlockDevice::open_existing(
+                &path,
+                policy,
+                PhysicalBlockSizeSource::Declared(PhysicalBlockSizeInBytes(512)),
+            )
+            .expect("打开（临时目录在 ext4 上）");
+            assert_eq!(
+                device
+                    .discard_at(DeviceOffsetInBytes(8192), 16384)
+                    .expect("discard"),
+                DiscardOutcome::Unsupported,
+                "{policy:?}"
+            );
+            assert!(matches!(
+                device.discard_at(DeviceOffsetInBytes(100), 512),
+                Err(BlockDeviceError::Unaligned { .. })
+            ));
+            let mut sector = vec![0u8; 512];
+            device
+                .read_at(DeviceOffsetInBytes(8192), &mut sector)
+                .expect("读");
+            assert!(sector.iter().all(|byte| *byte == DISCARD_DIRT_BYTE));
+            assert_eq!(
+                device.discard_outcome_counts(),
+                DiscardOutcomeCounts {
+                    applied_calls: 0,
+                    unsupported_calls: 1
+                },
+                "{policy:?}"
+            );
+        }
+    }
 }
diff --git a/crates/singlefs-harness/src/lib.rs b/crates/singlefs-harness/src/lib.rs
--- a/crates/singlefs-harness/src/lib.rs
+++ b/crates/singlefs-harness/src/lib.rs
@@ -11,7 +11,8 @@
 
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
 use singlefs_core::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+    BlockDevice, BlockDeviceError, DiscardOutcome, DiscardOutcomeCounts, PhysicalBlockSizeInBytes,
+    WriteDurability,
 };
 
 pub mod fault_injection;
@@ -38,12 +39,17 @@
     /// 整段清零：一次 [`BlockDevice::write_zeroes_at`] 记成**一步**，不是底下拆出来的那几次写
     /// （用户 2026-09-19 定案「录制流登记成一种新步骤、崩溃点重放认它」）。内容由 `offset` 与 `length` 全定：整段全 0。
     WriteZeroes,
+    /// discard：一次 [`BlockDevice::discard_at`] 交回 [`DiscardOutcome::Applied`] 时记**一步**；交回「不支持」的不记
+    /// （那一段一个字节没动，与「录制器只录落下了的写」同一个口径），只记进 [`SharedStream::discard_outcome_counts`]。
+    /// 崩溃点重放按「范围写零」枚举它（D13（验证路线） 已定项 4 的两态：持久 → 0、没持久 → 旧字节），切段时算一次普通写。
+    /// 内容由 `offset` 与 `length` 全定，不留字节。
+    Discard,
     Barrier,
 }
 
 impl RecordedOperationKind {
     /// 流文本（[`RecordedOperation::to_stream_line`] 的 `kind=`）与各导出装置结果行（`kind=`）里的种类名。新写的地方都调它，不再抄一份；
-    /// checker 档里还有四份装置各自手抄着同样四个名字（`src/bin/` 下 `e170_o3_spec_executor_export.rs`、`e171_three_oracle_collusion.rs`、
+    /// checker 档里还有四份装置各自手抄着同样五个名字（`src/bin/` 下 `e170_o3_spec_executor_export.rs`、`e171_three_oracle_collusion.rs`、
     /// `e171_three_oracle_collusion_after_the_oracle_fixes.rs`、`e142_new_pool_file_creation_write_dump_one_device.rs`；前三份另有一份
     /// 按名字反查种类的），没挪过来，改名时那四份要跟着改。
     #[must_use]
@@ -52,6 +58,7 @@
             RecordedOperationKind::Write => "write",
             RecordedOperationKind::WriteForceUnitAccess => "write_fua",
             RecordedOperationKind::WriteZeroes => "write_zeroes",
+            RecordedOperationKind::Discard => "discard",
             RecordedOperationKind::Barrier => "barrier",
         }
     }
@@ -123,7 +130,7 @@
 }
 
 impl RetainedOperation {
-    /// 这一步写到盘上的字节：普通写与 FUA 写是留下的内容（没开内容保留的流里是 `None`）；写清零按长度现铺 0
+    /// 这一步写到盘上的字节：普通写与 FUA 写是留下的内容（没开内容保留的流里是 `None`）；写清零与 discard 按长度现铺 0
     /// （录制器不为它留内容，整段全 0 由偏移与长度全定，见 [`RecordingBlockDevice`] 的 `write_zeroes_at`）；屏障没有字节，`None`。
     /// 写清零真的摆进内存：mkfs 那一次清 768 MiB，调用方只对窗口里的写调它。
     ///
@@ -142,6 +149,15 @@
                         "写清零的长度装得进 usize：64 位目标上恒成立"
                     )
             ]),
+            // discard 按「范围写零」重建（持久那一态读回 0），与写清零一样不留内容、按长度铺 0。
+            RecordedOperationKind::Discard => Some(
+                std::iter::repeat_n(
+                    0u8,
+                    usize::try_from(self.operation.length)
+                        .expect("discard 的长度装得进 usize：64 位目标上恒成立"),
+                )
+                .collect(),
+            ),
             RecordedOperationKind::Barrier => None,
         }
     }
@@ -169,6 +185,8 @@
     contents: Vec<Option<Vec<u8>>>,
     retain_contents: bool,
     entry_spans: Vec<RecordedEntrySpan>,
+    /// 经这条流的录制器发下去的 discard 各交回了哪一种结局：「不支持」的不进流，只在这里看得见。
+    discard_outcome_counts: DiscardOutcomeCounts,
 }
 
 /// 一个池里几块设备共用的录制流：按发出次序记，谁先发谁在前。
@@ -191,9 +209,20 @@
             contents: Vec::new(),
             retain_contents: true,
             entry_spans: Vec::new(),
+            discard_outcome_counts: DiscardOutcomeCounts::NONE,
         })))
     }
 
+    /// 经这条流的录制器发下去的 discard 各交回了哪一种结局（`.claude/rules/fs-design.md`「五条硬要求」第 4 条：分支可观测）。
+    #[must_use]
+    pub fn discard_outcome_counts(&self) -> DiscardOutcomeCounts {
+        self.0.borrow().discard_outcome_counts
+    }
+
+    fn note_discard_outcome(&self, outcome: DiscardOutcome) {
+        self.0.borrow_mut().discard_outcome_counts.note(outcome);
+    }
+
     /// 调 `run`（驱动这条流的一方调一个入口），把它期间录下的操作记成 `entry` 发的一段（[`RecordedEntrySpan`]）。
     /// `run` 报错也照记：报错之前已经发出去的写同样是这个入口发的。
     pub fn record_entry<Output>(
@@ -419,6 +448,32 @@
         Ok(())
     }
 
+    /// 转给里面那块盘，交回它的结局。「交给了底层」的记成**一步**（`RecordedOperationKind::Discard`，内容不留、哈希按全 0 记：
+    /// 崩溃点重放按「范围写零」重建它）；「不支持」的那一段一个字节没动，不进流，只记进这条流的
+    /// [`SharedStream::discard_outcome_counts`]。报错的不记。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        let outcome = self.inner.discard_at(offset, length)?;
+        self.stream.note_discard_outcome(outcome);
+        match outcome {
+            DiscardOutcome::Applied => self.stream.push(
+                RecordedOperation {
+                    device: self.device,
+                    kind: RecordedOperationKind::Discard,
+                    offset,
+                    length,
+                    content_hash: fnv1a_64_of_zeros(length),
+                },
+                None,
+            ),
+            DiscardOutcome::Unsupported => {}
+        }
+        Ok(outcome)
+    }
+
     fn barrier(&mut self) -> Result<(), BlockDeviceError> {
         self.inner.barrier()?;
         self.stream.push(
diff --git a/crates/singlefs-harness/src/segments.rs b/crates/singlefs-harness/src/segments.rs
--- a/crates/singlefs-harness/src/segments.rs
+++ b/crates/singlefs-harness/src/segments.rs
@@ -7,6 +7,9 @@
 //! 一次**普通写**属于哪一种由它的落点决定（第一版几何写死：系统配置槽 / 根环区域 / journal 环 / 单元区）；
 //! 整段清零按**动作**归类、不看落点：它在块层就是另一个命令（真设备上映射成 WRITE ZEROES），
 //! 不是「写在某处」的一次写。今天唯一的清零是 mkfs 清 journal 环。
+//! discard（[`RecordedOperationKind::Discard`]）同样按动作归类，归进整段清零那一种：崩溃点重放按「范围写零」枚举它
+//! （D13（验证路线） 已定项 4 的两态：持久 → 0、没持久 → 旧字节），切段时算一次普通写（不放行、不关段）。
+//! 种类串里它因此也写成 `zero_fill`；要不要给它单开一种步骤种类没有条款（D17（实现分层与第三方管道） 已定项 2 的封闭枚举），第一版不开。
 
 use std::collections::{BTreeMap, BTreeSet};
 
@@ -61,6 +64,8 @@
         match operation.kind {
             RecordedOperationKind::Barrier => StepKind::Barrier,
             RecordedOperationKind::WriteZeroes => StepKind::ZeroFill,
+            // discard 按「范围写零」枚举（文件头），与整段清零同一种。
+            RecordedOperationKind::Discard => StepKind::ZeroFill,
             RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                 let offset = operation.offset.0;
                 let system_configuration_end = SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE
@@ -116,8 +121,10 @@
     /// 当前段一个写都没有时屏障不关段（流首那道屏障、刚关过段之后别的盘补发的那一道），它并进即将开始的那一段。
     pub fn after(&mut self, operation: &RecordedOperation) -> SegmentAfterOperation {
         match operation.kind {
-            // 清零是普通写那一档：不做持久、不放行，段里多算一个写。
-            RecordedOperationKind::Write | RecordedOperationKind::WriteZeroes => {
+            // 清零与 discard 是普通写那一档：不做持久、不放行，段里多算一个写。
+            RecordedOperationKind::Write
+            | RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard => {
                 self.writes_in_current_segment += 1;
                 self.devices_with_unreleased_writes.insert(operation.device);
                 SegmentAfterOperation::StaysOpen
diff --git a/crates/singlefs-harness/src/memory_pool.rs b/crates/singlefs-harness/src/memory_pool.rs
--- a/crates/singlefs-harness/src/memory_pool.rs
+++ b/crates/singlefs-harness/src/memory_pool.rs
@@ -10,7 +10,7 @@
 use singlefs_core::address::{
     CheckpointTxg, DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration,
 };
-use singlefs_core::block_device::{BlockDeviceError, PhysicalBlockSizeInBytes};
+use singlefs_core::block_device::{BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes};
 use singlefs_core::recovery::PoolReader;
 use singlefs_core::system_configuration::unit_area_start_slot_recorded_in_the_slot;
 use singlefs_format::{
@@ -206,6 +206,16 @@
         self.image.zero_fill(offset, length);
         Ok(())
     }
+    /// 内存盘认 discard，把它当整段清零应用：之后读这一段是全 0（崩溃点重放「持久 → 0」那一态），交回「交给了底层」。
+    fn discard_at(
+        &mut self,
+        discarded_offset: DeviceOffsetInBytes,
+        discarded_length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        self.check_request(discarded_offset, discarded_length)?;
+        self.image.zero_fill(discarded_offset, discarded_length);
+        Ok(DiscardOutcome::Applied)
+    }
     fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {
         Ok(())
     }
@@ -354,6 +364,14 @@
                         .zero_fill(retained.operation.offset, retained.operation.length);
                     continue;
                 }
+                // discard 整个施加时取「持久 → 0」那一态（这一份是「整段流都落了」的镜像）。
+                RecordedOperationKind::Discard => {
+                    self.devices
+                        .get_mut(&retained.operation.device)
+                        .expect("录到的 discard 都落在池里的盘上")
+                        .zero_fill(retained.operation.offset, retained.operation.length);
+                    continue;
+                }
                 RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => self
                     .devices
                     .get_mut(&retained.operation.device)
@@ -581,6 +599,21 @@
                 writes.push(RetainedWrite {
                     device: retained.operation.device,
                     kind: geometry.classify(&retained.operation),
+                    is_force_unit_access: false,
+                    offset: retained.operation.offset,
+                    contents: WrittenContents::Zeros {
+                        length: retained.operation.length,
+                    },
+                });
+                stream_indexes.push(stream_index);
+                current.push(writes.len() - 1);
+            }
+            // discard 按「范围写零」进写表（D13（验证路线） 已定项 4）：持久那一态这一段是 0，没持久那一态是这次之前的旧字节；
+            // 只取这两态，不是原地覆写（种类按动作归进整段清零，`crash` 的撕裂判据不给它第三态）。
+            RecordedOperationKind::Discard => {
+                writes.push(RetainedWrite {
+                    device: retained.operation.device,
+                    kind: geometry.classify(&retained.operation),
                     is_force_unit_access: false,
                     offset: retained.operation.offset,
                     contents: WrittenContents::Zeros {
diff --git a/crates/singlefs-harness/src/fault_injection.rs b/crates/singlefs-harness/src/fault_injection.rs
--- a/crates/singlefs-harness/src/fault_injection.rs
+++ b/crates/singlefs-harness/src/fault_injection.rs
@@ -37,7 +37,7 @@
 use singlefs_checker::walk::check_pool_image;
 use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes, InstanceGeneration};
 use singlefs_core::block_device::{
-    BlockDevice, BlockDeviceError, PhysicalBlockSizeInBytes, WriteDurability,
+    BlockDevice, BlockDeviceError, DiscardOutcome, PhysicalBlockSizeInBytes, WriteDurability,
 };
 use singlefs_core::make_filesystem::MKFS_INSTANCE_GENERATION;
 use singlefs_core::recovery::{
@@ -577,6 +577,10 @@
     /// 真交给里面那块盘的屏障（被吞掉的那些不算）。
     pub barriers_forwarded: u64,
     pub barriers_swallowed: u64,
+    /// 交给里面那块盘、它交回「交给了底层」的 discard（不算进 `writes` 与 `written_bytes`：块层把 discard 另记一类）。
+    pub discards_applied: u64,
+    /// 交给里面那块盘、它交回「不支持」的 discard。
+    pub discards_answered_unsupported: u64,
 }
 
 impl FaultDeviceCounts {
@@ -591,6 +595,9 @@
                 - earlier.force_unit_access_writes,
             barriers_forwarded: self.barriers_forwarded - earlier.barriers_forwarded,
             barriers_swallowed: self.barriers_swallowed - earlier.barriers_swallowed,
+            discards_applied: self.discards_applied - earlier.discards_applied,
+            discards_answered_unsupported: self.discards_answered_unsupported
+                - earlier.discards_answered_unsupported,
         }
     }
 }
@@ -654,6 +661,10 @@
     Zeroes {
         length: u64,
     },
+    /// discard（`BlockDevice::discard_at`）：报了「交给了底层」，其实没交下去。
+    Discard {
+        length: u64,
+    },
 }
 
 /// 说谎的设备吞掉的那一次写（[`InjectedFault::WriteIsSwallowed`]）：落在哪、本该写下什么、它之前整池交下去了几次写（含写零）。
@@ -698,6 +709,16 @@
                 },
                 contents: None,
             },
+            SwallowedContents::Discard { length } => crate::RetainedOperation {
+                operation: RecordedOperation {
+                    device: self.device,
+                    kind: RecordedOperationKind::Discard,
+                    offset: self.offset,
+                    length: *length,
+                    content_hash: crate::fnv1a_64_of_zeros(*length),
+                },
+                contents: None,
+            },
         }
     }
 }
@@ -914,6 +935,29 @@
         }
     }
 
+    /// 一次 discard 交给了里面那块盘、它交回了 `outcome`。「交给了底层」的在整池写数里算一次（录制器在注入层里面时它在录制流里记一步，
+    /// 被吞的写排在第几次写之后按这个数）；「不支持」的不算（录制器不记它）。
+    fn note_forwarded_discard(&self, device: DeviceIdentity, outcome: DiscardOutcome) {
+        let mut state = self.0.borrow_mut();
+        match outcome {
+            DiscardOutcome::Applied => {
+                state.writes_forwarded_across_the_pool += 1;
+                state
+                    .per_device_counts
+                    .entry(device)
+                    .or_default()
+                    .discards_applied += 1;
+            }
+            DiscardOutcome::Unsupported => {
+                state
+                    .per_device_counts
+                    .entry(device)
+                    .or_default()
+                    .discards_answered_unsupported += 1;
+            }
+        }
+    }
+
     fn note_forwarded_barrier(&self, device: DeviceIdentity) {
         self.0
             .borrow_mut()
@@ -1083,6 +1127,43 @@
         Ok(())
     }
 
+    /// discard 与写零一样走写的注入判定（`FaultCallKind::Write`，落点 `offset`、长度 `length`）：崩溃点重放按「范围写零」看它。
+    /// `WriteFails` 让它报错、一个字节不动；`WriteIsSwallowed` 让它报「交给了底层」、其实没交下去（另记一笔被吞的写）；
+    /// 不注入就原样转给里面那块盘，交回它的结局。
+    fn discard_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<DiscardOutcome, BlockDeviceError> {
+        match self
+            .plan
+            .decide(FaultCallKind::Write, self.device, offset, length)
+        {
+            Some(InjectedFault::WriteFails) => {
+                return Err(injected_block_device_error("discard"));
+            }
+            Some(InjectedFault::WriteIsSwallowed) => {
+                self.plan.note_swallowed_write(
+                    self.device,
+                    offset,
+                    SwallowedContents::Discard { length },
+                    WriteDurability::Plain,
+                );
+                return Ok(DiscardOutcome::Applied);
+            }
+            Some(
+                InjectedFault::ReadFails
+                | InjectedFault::ReadReturnsCorruptedBytes { .. }
+                | InjectedFault::BarrierFails
+                | InjectedFault::BarrierIsSwallowed,
+            )
+            | None => {}
+        }
+        let outcome = self.inner.discard_at(offset, length)?;
+        self.plan.note_forwarded_discard(self.device, outcome);
+        Ok(outcome)
+    }
+
     fn barrier(&mut self) -> Result<(), BlockDeviceError> {
         self.plan.note_barrier_arrived(self.device);
         match self.plan.decide(
@@ -2664,7 +2745,7 @@
             .iter()
             .filter_map(|write| match &write.contents {
                 SwallowedContents::Bytes(bytes) => Some(root_identity_written_by(bytes)),
-                SwallowedContents::Zeroes { .. } => None,
+                SwallowedContents::Zeroes { .. } | SwallowedContents::Discard { .. } => None,
             })
             .map(|(instance, checkpoint_txg)| model_root_key(instance, checkpoint_txg))
             .collect();
@@ -3195,6 +3276,8 @@
                 force_unit_access_writes: 0,
                 barriers_forwarded: 1,
                 barriers_swallowed: 0,
+                discards_applied: 0,
+                discards_answered_unsupported: 0,
             }
         );
         assert!(plan.fired().is_empty(), "没开计划就不该注入");
diff --git a/crates/singlefs-harness/src/new_pool_file_creation_regions.rs b/crates/singlefs-harness/src/new_pool_file_creation_regions.rs
--- a/crates/singlefs-harness/src/new_pool_file_creation_regions.rs
+++ b/crates/singlefs-harness/src/new_pool_file_creation_regions.rs
@@ -346,9 +346,11 @@
             // 整段清零也是一次写调用，落点与长度同样拿去比表：这张表登记的是新池新建文件写了哪几段，
             // 表里没有「清一段」这一项，真在新池新建文件里清了一段就该落进 `writes_outside_the_table` 报出来
             //（今天唯一的清零是 mkfs 清 journal 环，不在新池新建文件的录制流里）。
+            // discard 同样拿去比表：表里没有这一项，新池新建文件里真发了 discard 就落进 `writes_outside_the_table`。
             RecordedOperationKind::Write
             | RecordedOperationKind::WriteForceUnitAccess
-            | RecordedOperationKind::WriteZeroes => {
+            | RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard => {
                 write_calls += 1;
             }
         }
diff --git a/crates/singlefs-harness/src/reference_filesystem_translation.rs b/crates/singlefs-harness/src/reference_filesystem_translation.rs
--- a/crates/singlefs-harness/src/reference_filesystem_translation.rs
+++ b/crates/singlefs-harness/src/reference_filesystem_translation.rs
@@ -214,9 +214,9 @@
             .filter_map(|(stream_index, retained)| {
                 match retained.operation.kind {
                     RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {}
-                    RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
-                        return None
-                    }
+                    RecordedOperationKind::WriteZeroes
+                    | RecordedOperationKind::Discard
+                    | RecordedOperationKind::Barrier => return None,
                 }
                 if geometry.classify(&retained.operation) != StepKind::RootRecordFua {
                     return None;
diff --git a/crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs b/crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs
new file mode 100644
--- /dev/null
+++ b/crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs
@@ -0,0 +1,409 @@
+//! discard 那个口子在 harness 档里的几样（`research/prompts/m4-gc-r1-main-verification.md` 第三节 G5「trait 形态」）：
+//! 内存盘把它当整段清零应用、录制器把交给了底层的那一次记成一步、不认 discard 的后端交回「不支持」且看得见、
+//! 切段把它当一次普通写（崩溃点重放按「范围写零」枚举两态，D13（验证路线） 已定项 4）、故障注入设备转给里面那块盘并注入得了失败。
+//! 产品路径这一批还不发 discard（谁发、什么时候发归下一批），这里直接调块设备。
+
+use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
+use singlefs_core::block_device::{
+    BlockDevice, BlockDeviceError, DiscardOutcome, DiscardOutcomeCounts, PhysicalBlockSizeInBytes,
+    WriteDurability,
+};
+use singlefs_core::recovery::PoolReader;
+use singlefs_core::root_ring::RootRingSlotsPerRegion;
+use singlefs_harness::fault_injection::{
+    FaultInjectingBlockDevice, FaultSchedule, InjectedFault, SharedFaultPlan,
+};
+use singlefs_harness::memory_pool::{
+    closed_form_state_count, writes_and_segments, CrashImage, MemoryPool, SparseBlockDevice,
+    WrittenContents,
+};
+use singlefs_harness::segments::{
+    segment_kinds_text, segment_sizes_text, split_into_segments, FixedGeometry, StepKind,
+};
+use singlefs_harness::{
+    fnv1a_64, fnv1a_64_of_zeros, RecordedOperationKind, RecordingBlockDevice, SharedStream,
+};
+
+const SECTOR: usize = 512;
+const DIRT_BYTE: u8 = 0xC3;
+/// 内存盘的大小：要装得下单元区里的落点（第一版几何里单元区从槽 50176 起，槽宽 16 KiB）。
+const DEVICE_BYTES: u64 = 1 << 30;
+/// 单元区里的一个落点：切段时普通写落在这里归「写单元」。
+const UNIT_OFFSET: u64 = 50176 * 16384;
+
+fn geometry() -> FixedGeometry {
+    FixedGeometry {
+        fixed_structure_slot_spacing: 4096,
+        journal_ring_bytes: 768 << 20,
+        root_ring_slots_per_region: RootRingSlotsPerRegion::AT_MAKE_FILESYSTEM,
+    }
+}
+
+fn sector_of(device: &impl BlockDevice, offset: u64) -> Vec<u8> {
+    let mut sector = vec![0x5Au8; SECTOR];
+    device
+        .read_at(DeviceOffsetInBytes(offset), &mut sector)
+        .expect("读一扇区");
+    sector
+}
+
+/// 内存盘：前四扇区先写脏，再 discard 第二、三扇区。
+fn memory_disk_with_two_sectors_discarded() -> (SparseBlockDevice, DiscardOutcome) {
+    let mut device = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
+    device
+        .write_at(
+            DeviceOffsetInBytes(0),
+            &[DIRT_BYTE; 4 * SECTOR],
+            WriteDurability::Plain,
+        )
+        .expect("写脏");
+    let outcome = device
+        .discard_at(DeviceOffsetInBytes(512), 1024)
+        .expect("discard");
+    (device, outcome)
+}
+
+/// 内存盘认 discard、当整段清零应用：段内读回全 0，段外一个字节不动，交回「交给了底层」。
+#[test]
+fn a_discard_on_the_memory_disk_reads_back_zero_and_leaves_the_neighbours() {
+    let (device, outcome) = memory_disk_with_two_sectors_discarded();
+    assert_eq!(outcome, DiscardOutcome::Applied);
+    for offset in [512u64, 1024] {
+        assert_eq!(
+            sector_of(&device, offset),
+            vec![0u8; SECTOR],
+            "偏移 {offset} 在 discard 段内，读回全 0"
+        );
+    }
+    for offset in [0u64, 1536] {
+        assert_eq!(
+            sector_of(&device, offset),
+            vec![DIRT_BYTE; SECTOR],
+            "偏移 {offset} 在 discard 段外，不该被动"
+        );
+    }
+}
+
+/// 录制器把交给了底层的 discard 记成一步：种类 `discard`、偏移长度照发、哈希按全 0 记、开了内容保留也不留字节；
+/// 这条流数到一次「交给了底层」。
+#[test]
+fn the_recorder_records_an_applied_discard_as_one_step_without_contents() {
+    let stream = SharedStream::retaining_contents();
+    let mut recorder = RecordingBlockDevice::with_shared_stream(
+        DeviceIdentity(0),
+        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
+        stream.clone(),
+    );
+    recorder
+        .write_at(
+            DeviceOffsetInBytes(0),
+            &[DIRT_BYTE; 2 * SECTOR],
+            WriteDurability::Plain,
+        )
+        .expect("写");
+    let outcome = recorder
+        .discard_at(DeviceOffsetInBytes(512), 512)
+        .expect("discard");
+    assert_eq!(outcome, DiscardOutcome::Applied);
+    let operations = stream.operations();
+    assert_eq!(operations.len(), 2, "一次写 + 一次 discard = 两步");
+    assert_eq!(operations[1].kind, RecordedOperationKind::Discard);
+    assert_eq!(operations[1].device, DeviceIdentity(0));
+    assert_eq!(operations[1].offset, DeviceOffsetInBytes(512));
+    assert_eq!(operations[1].length, 512);
+    assert_eq!(operations[1].content_hash, fnv1a_64(&[0u8; SECTOR]));
+    assert_eq!(
+        operations[1].to_stream_line(),
+        format!(
+            "device=0 kind=discard offset=512 length=512 hash={:016x}",
+            fnv1a_64_of_zeros(512)
+        )
+    );
+    let retained = stream.retained_operations();
+    assert!(retained[1].contents.is_none(), "discard 不留内容");
+    assert_eq!(
+        retained[1].written_bytes_rebuilding_zero_fills(),
+        Some(vec![0u8; SECTOR]),
+        "重建 discard 写下的字节：按长度铺 0（「范围写零」）"
+    );
+    assert_eq!(
+        stream.discard_outcome_counts(),
+        DiscardOutcomeCounts {
+            applied_calls: 1,
+            unsupported_calls: 0
+        }
+    );
+    assert_eq!(
+        sector_of(&recorder, 512),
+        vec![0u8; SECTOR],
+        "录制器把 discard 转给了里面那块盘"
+    );
+}
+
+/// 只实现六个旧动作、discard 用默认实现的一块盘（不认 discard 的后端）。
+struct DeviceKeepingTheDefaultDiscard {
+    bytes: Vec<u8>,
+}
+
+impl BlockDevice for DeviceKeepingTheDefaultDiscard {
+    fn read_at(
+        &self,
+        offset: DeviceOffsetInBytes,
+        buffer: &mut [u8],
+    ) -> Result<(), BlockDeviceError> {
+        let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
+        buffer.copy_from_slice(&self.bytes[start..start + buffer.len()]);
+        Ok(())
+    }
+    fn write_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        bytes: &[u8],
+        _durability: WriteDurability,
+    ) -> Result<(), BlockDeviceError> {
+        let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
+        self.bytes[start..start + bytes.len()].copy_from_slice(bytes);
+        Ok(())
+    }
+    fn write_zeroes_at(
+        &mut self,
+        offset: DeviceOffsetInBytes,
+        length: u64,
+    ) -> Result<(), BlockDeviceError> {
+        let start = usize::try_from(offset.0).expect("测试偏移装得进 usize");
+        let end = start + usize::try_from(length).expect("测试长度装得进 usize");
+        self.bytes[start..end].fill(0);
+        Ok(())
+    }
+    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
+        Ok(())
+    }
+    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
+        PhysicalBlockSizeInBytes(512)
+    }
+    fn size_in_bytes(&self) -> u64 {
+        u64::try_from(self.bytes.len()).expect("测试镜像装得进 u64")
+    }
+}
+
+/// 默认实现：交回「不支持」、一个字节不动；录制器不给它记一步，只把它数进「不支持」；没对齐的照样拒、不数。
+#[test]
+fn a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step() {
+    let stream = SharedStream::new();
+    let mut recorder = RecordingBlockDevice::with_shared_stream(
+        DeviceIdentity(0),
+        DeviceKeepingTheDefaultDiscard {
+            bytes: vec![DIRT_BYTE; 4 * SECTOR],
+        },
+        stream.clone(),
+    );
+    let outcome = recorder
+        .discard_at(DeviceOffsetInBytes(512), 1024)
+        .expect("discard");
+    assert_eq!(outcome, DiscardOutcome::Unsupported);
+    assert!(
+        stream.operations().is_empty(),
+        "「不支持」的 discard 不进录制流"
+    );
+    assert_eq!(
+        sector_of(&recorder, 512),
+        vec![DIRT_BYTE; SECTOR],
+        "不支持 discard 的后端一个字节不动"
+    );
+    assert!(matches!(
+        recorder.discard_at(DeviceOffsetInBytes(100), 512),
+        Err(BlockDeviceError::Unaligned { .. })
+    ));
+    assert_eq!(
+        stream.discard_outcome_counts(),
+        DiscardOutcomeCounts {
+            applied_calls: 0,
+            unsupported_calls: 1
+        },
+        "交回「不支持」的那一次看得见；被拒的那一次两边都不数"
+    );
+}
+
+/// 一次写单元、一次 discard（落在已经写过旧字节的地方）、一道屏障的流，带内容；前面先有一次落了盘的旧写当基线。
+fn base_pool_and_stream_with_one_discard() -> (MemoryPool, Vec<singlefs_harness::RetainedOperation>)
+{
+    let stream = SharedStream::retaining_contents();
+    let mut recorder = RecordingBlockDevice::with_shared_stream(
+        DeviceIdentity(0),
+        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
+        stream.clone(),
+    );
+    recorder
+        .write_at(
+            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
+            &[DIRT_BYTE; SECTOR],
+            WriteDurability::Plain,
+        )
+        .expect("旧写");
+    recorder.barrier().expect("旧写的屏障");
+    let base_operations = stream.retained_operations();
+    recorder
+        .write_at(
+            DeviceOffsetInBytes(UNIT_OFFSET),
+            &[7u8; SECTOR],
+            WriteDurability::Plain,
+        )
+        .expect("写单元");
+    recorder
+        .discard_at(DeviceOffsetInBytes(UNIT_OFFSET + 16384), 512)
+        .expect("discard");
+    recorder.barrier().expect("屏障");
+    let mut base = MemoryPool::with_devices(&[DeviceIdentity(0)], DEVICE_BYTES);
+    base.apply(&base_operations);
+    let operations = stream.retained_operations()[base_operations.len()..].to_vec();
+    (base, operations)
+}
+
+/// 切段把 discard 当一次普通写：种类按动作归进整段清零（`zero_fill`），不放行、不关段，与前面的单元写同段；
+/// 写表里它是一次长 512 的全 0 写，只取两态——持久时那一段读回 0，没持久时是旧字节。
+#[test]
+fn segments_count_a_discard_as_one_plain_write_with_two_crash_states() {
+    let (base, operations) = base_pool_and_stream_with_one_discard();
+    let recorded: Vec<_> = operations
+        .iter()
+        .map(|retained| retained.operation.clone())
+        .collect();
+    let kinds_by_segment = split_into_segments(&recorded, &geometry());
+    assert_eq!(segment_sizes_text(&kinds_by_segment), "2");
+    assert_eq!(
+        segment_kinds_text(&kinds_by_segment),
+        "[zero_fill,unit_write,barrier]"
+    );
+    let (writes, segments) = writes_and_segments(&operations, &geometry());
+    assert_eq!(segments, vec![vec![0, 1]], "单元写与 discard 同段");
+    assert_eq!(writes[1].kind, StepKind::ZeroFill);
+    assert_eq!(writes[1].contents, WrittenContents::Zeros { length: 512 });
+    assert!(!writes[1].is_force_unit_access);
+    assert_eq!(
+        closed_form_state_count(&segments),
+        1 + ((1 << 2) - 1),
+        "两次写、每次两态：1 + (2^2 − 1)"
+    );
+    let discard_persisted = CrashImage {
+        base: &base,
+        writes: &writes,
+        persisted: vec![false, true],
+    };
+    assert_eq!(
+        PoolReader::read(
+            &discard_persisted,
+            DeviceIdentity(0),
+            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
+            SECTOR
+        ),
+        Some(vec![0u8; SECTOR]),
+        "discard 持久那一态：那一段读回 0"
+    );
+    let discard_not_persisted = CrashImage {
+        base: &base,
+        writes: &writes,
+        persisted: vec![true, false],
+    };
+    assert_eq!(
+        PoolReader::read(
+            &discard_not_persisted,
+            DeviceIdentity(0),
+            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
+            SECTOR
+        ),
+        Some(vec![DIRT_BYTE; SECTOR]),
+        "discard 没持久那一态：那一段是旧字节"
+    );
+}
+
+/// 把录到的流整个施加到内存池上（「整段流都落了」那一份镜像）：discard 那一段清成 0。
+#[test]
+fn applying_a_recorded_discard_to_a_memory_pool_zero_fills_its_range() {
+    let (mut pool, operations) = base_pool_and_stream_with_one_discard();
+    pool.apply(&operations);
+    assert_eq!(
+        PoolReader::read(
+            &pool,
+            DeviceIdentity(0),
+            DeviceOffsetInBytes(UNIT_OFFSET + 16384),
+            SECTOR
+        ),
+        Some(vec![0u8; SECTOR])
+    );
+    assert_eq!(
+        PoolReader::read(
+            &pool,
+            DeviceIdentity(0),
+            DeviceOffsetInBytes(UNIT_OFFSET),
+            SECTOR
+        ),
+        Some(vec![7u8; SECTOR]),
+        "discard 之外的写照落"
+    );
+}
+
+/// 故障注入设备不注入时把 discard 原样转给里面那块盘、交回它的结局，记进这块盘的 discard 计数，不算进写数。
+#[test]
+fn the_fault_injecting_device_forwards_a_discard_to_the_device_inside() {
+    let plan = SharedFaultPlan::unarmed(geometry());
+    let mut device = FaultInjectingBlockDevice::new(
+        DeviceIdentity(0),
+        SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512)),
+        plan.clone(),
+    );
+    device
+        .write_at(
+            DeviceOffsetInBytes(2048),
+            &[DIRT_BYTE; SECTOR],
+            WriteDurability::Plain,
+        )
+        .expect("写脏");
+    let outcome = device
+        .discard_at(DeviceOffsetInBytes(2048), 512)
+        .expect("discard");
+    assert_eq!(outcome, DiscardOutcome::Applied);
+    assert_eq!(sector_of(&device, 2048), vec![0u8; SECTOR]);
+    let counts = plan.counts_of_device(DeviceIdentity(0));
+    assert_eq!(counts.discards_applied, 1);
+    assert_eq!(counts.discards_answered_unsupported, 0);
+    assert_eq!(counts.writes, 1, "discard 不算进写数");
+}
+
+/// 故障注入设备让 discard 报错：报块设备错，里面那块盘一个字节不动，也不记进 discard 计数。
+#[test]
+fn the_fault_injecting_device_makes_a_discard_fail_and_leaves_the_device_inside_untouched() {
+    let plan = SharedFaultPlan::armed(
+        geometry(),
+        FaultSchedule::the_nth_call_across_the_pool(InjectedFault::WriteFails, 1),
+    );
+    let mut sparse = SparseBlockDevice::new(DEVICE_BYTES, PhysicalBlockSizeInBytes(512));
+    sparse
+        .write_at(
+            DeviceOffsetInBytes(512),
+            &[DIRT_BYTE; SECTOR],
+            WriteDurability::Plain,
+        )
+        .expect("写脏（注入层外面）");
+    let mut device = FaultInjectingBlockDevice::new(DeviceIdentity(0), sparse, plan.clone());
+    let outcome = device.discard_at(DeviceOffsetInBytes(512), 512);
+    assert!(
+        matches!(outcome, Err(BlockDeviceError::InputOutput(_))),
+        "注入的失败交回块设备错：{outcome:?}"
+    );
+    assert_eq!(sector_of(&device, 512), vec![DIRT_BYTE; SECTOR]);
+    assert_eq!(plan.fired_count(), 1);
+    assert_eq!(plan.counts_of_device(DeviceIdentity(0)).discards_applied, 0);
+}
+
+/// 录制流的种类名：discard 是 `discard`，与另四种互不相同。
+#[test]
+fn the_discard_kind_has_its_own_stream_name() {
+    let names = [
+        RecordedOperationKind::Write.name(),
+        RecordedOperationKind::WriteForceUnitAccess.name(),
+        RecordedOperationKind::WriteZeroes.name(),
+        RecordedOperationKind::Barrier.name(),
+    ];
+    assert_eq!(RecordedOperationKind::Discard.name(), "discard");
+    assert!(!names.contains(&RecordedOperationKind::Discard.name()));
+}
diff --git a/crates/singlefs-harness/tests/write_accounting_per_structure.rs b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
--- a/crates/singlefs-harness/tests/write_accounting_per_structure.rs
+++ b/crates/singlefs-harness/tests/write_accounting_per_structure.rs
@@ -69,7 +69,8 @@
                 | RecordedOperationKind::WriteZeroes => {
                     sum.plus(calls_and_bytes(1, operation.length))
                 }
-                RecordedOperationKind::Barrier => sum,
+                // discard 不是一次写调用：写入口按结构记的写账里没有它。
+                RecordedOperationKind::Discard | RecordedOperationKind::Barrier => sum,
             }
         })
 }
diff --git a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
--- a/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
+++ b/crates/singlefs-harness/tests/rollback_by_a_forward_publish.rs
@@ -577,9 +577,11 @@
     let writes_of_the_rollback = operations_of_the_rollback
         .iter()
         .filter(|operation| match operation.kind {
+            // 切段时 discard 算一次写（`segments` 文件头），这里与段里的写数同一个口径。
             RecordedOperationKind::Write
             | RecordedOperationKind::WriteForceUnitAccess
-            | RecordedOperationKind::WriteZeroes => true,
+            | RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard => true,
             RecordedOperationKind::Barrier => false,
         })
         .count();
diff --git a/crates/singlefs-harness/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs b/crates/singlefs-harness/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs
--- a/crates/singlefs-harness/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs
+++ b/crates/singlefs-harness/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs
@@ -248,7 +248,8 @@
             RecordedOperationKind::Write
             | RecordedOperationKind::WriteForceUnitAccess
             | RecordedOperationKind::WriteZeroes => true,
-            RecordedOperationKind::Barrier => false,
+            // discard 不是一次写调用，写入口下面那一层的写数里没有它。
+            RecordedOperationKind::Discard | RecordedOperationKind::Barrier => false,
         })
         .fold(WriteCallsAndBytes::NONE, |sum, operation| {
             sum.plus(WriteCallsAndBytes {
diff --git a/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs b/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs
--- a/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs
+++ b/crates/singlefs-harness/tests/mkfs_bytes_judged_by_the_checker.rs
@@ -412,7 +412,8 @@
             RecordedOperationKind::Write
             | RecordedOperationKind::WriteForceUnitAccess
             | RecordedOperationKind::WriteZeroes => operation.length,
-            RecordedOperationKind::Barrier => 0,
+            // discard 一个字节都不写。
+            RecordedOperationKind::Discard | RecordedOperationKind::Barrier => 0,
         })
         .sum();
     assert_eq!(
diff --git a/crates/singlefs-checker-tier/src/device_log.rs b/crates/singlefs-checker-tier/src/device_log.rs
--- a/crates/singlefs-checker-tier/src/device_log.rs
+++ b/crates/singlefs-checker-tier/src/device_log.rs
@@ -147,7 +147,7 @@
     })
 }
 
-/// 程序的信念投到一块盘上：这块盘上的每个写照录制流原样，FUA 写之后跟一个 FLUSH，
+/// 程序的信念投到一块盘上：这块盘上的每个写照录制流原样，FUA 写之后跟一个 FLUSH，每次 discard 是一条同偏移同长度的 Discard，
 /// 这块盘自己的每道屏障是一个 FLUSH（录制器按设备记屏障，代码审阅第 1 条；`PoolWriter` 与 mkfs 的池屏障逐盘发，每块盘各记一步，
 /// 别的盘的屏障不投到这块盘上）。
 #[must_use]
@@ -191,6 +191,17 @@
                     events.push(DeviceEvent::Flush);
                 }
             }
+            // discard：录制流只记交回「交给了底层」的那几次（`RecordedOperationKind::Discard`），盘上该恰好收到同一段、同一个次序的一条
+            // Discard 条目；漏了、多了、段不同、次序不同，逐项比对都判红。不支持 discard 的后端交回「不支持」，流里没有这一步、期望侧也不摆。
+            RecordedOperationKind::Discard => {
+                if operation.device != device {
+                    continue;
+                }
+                events.push(DeviceEvent::Discard {
+                    offset: operation.offset,
+                    length: operation.length,
+                });
+            }
         }
     }
     events
@@ -880,4 +891,101 @@
             "最后一条接在末端、伸出段尾：一条都不折"
         );
     }
+
+    /// 录制流里一次 discard 投成盘上同一段、同一个次序的一条 Discard：一次写、一次 discard、一道屏障的流，
+    /// 盘上日志照收判绿；日志里缺那条 discard、段不同、与写换了次序，都在第一处不同的下标判红。
+    #[test]
+    fn a_discard_in_the_recorded_stream_must_reach_the_device_log_as_the_same_range_in_the_same_order(
+    ) {
+        let data = vec![7u8; 512];
+        let retained = |kind, offset, length, content_hash, contents| RetainedOperation {
+            operation: RecordedOperation {
+                device: DeviceIdentity(0),
+                kind,
+                offset: DeviceOffsetInBytes(offset),
+                length,
+                content_hash,
+            },
+            contents,
+        };
+        let stream = vec![
+            retained(
+                RecordedOperationKind::Write,
+                4096,
+                512,
+                fnv1a_64(&data),
+                Some(data.clone()),
+            ),
+            retained(
+                RecordedOperationKind::Discard,
+                8192,
+                16384,
+                fnv1a_64_of_zeros(16384),
+                None,
+            ),
+            retained(RecordedOperationKind::Barrier, 0, 0, 0, None),
+        ];
+        let expected = expected_device_events(&stream, DeviceIdentity(0));
+        assert_eq!(
+            expected,
+            vec![
+                DeviceEvent::Write {
+                    offset: DeviceOffsetInBytes(4096),
+                    length: 512,
+                    content_hash: fnv1a_64(&data),
+                },
+                DeviceEvent::Discard {
+                    offset: DeviceOffsetInBytes(8192),
+                    length: 16384,
+                },
+                DeviceEvent::Flush,
+            ],
+            "程序那一侧：discard 投成一条 Discard，排在写与屏障之间"
+        );
+        assert_eq!(
+            expected_device_events(&stream, DeviceIdentity(1)),
+            vec![],
+            "别的盘上的 discard 不投到这块盘上"
+        );
+        let divergence_against = |entries: &[(u64, u64, u64, Vec<u8>)]| {
+            let log = parse_device_log(&log_with(entries)).expect("解析");
+            compare_allowing_trailing_flushes(&expected, &log.events)
+                .divergence
+                .map(|(index, _, _)| index)
+        };
+        let write_entry = (8, 1, 0, data.clone());
+        let flush_entry = (0, 0, LOG_FLUSH_FLAG, Vec::new());
+        assert_eq!(
+            divergence_against(&[
+                write_entry.clone(),
+                (16, 32, LOG_DISCARD_FLAG, Vec::new()),
+                flush_entry.clone(),
+            ]),
+            None,
+            "盘上收到同一段的 discard：对得上"
+        );
+        assert_eq!(
+            divergence_against(&[write_entry.clone(), flush_entry.clone()]),
+            Some(1),
+            "程序发了 discard、盘上没收到：判红在那一项"
+        );
+        assert_eq!(
+            divergence_against(&[
+                write_entry.clone(),
+                (16, 16, LOG_DISCARD_FLAG, Vec::new()),
+                flush_entry.clone(),
+            ]),
+            Some(1),
+            "盘上收到的 discard 只有一半长：判红"
+        );
+        assert_eq!(
+            divergence_against(&[
+                (16, 32, LOG_DISCARD_FLAG, Vec::new()),
+                write_entry,
+                flush_entry,
+            ]),
+            Some(0),
+            "discard 跑到写前面：次序不同判红"
+        );
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
--- a/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
+++ b/crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs
@@ -273,6 +273,15 @@
     (writes, flushes)
 }
 
+/// 一串事件里有几条 discard：结果行的 `expected_discards` / `observed_discards`。新池新建文件那条写路一次 discard 都不发，两边都是 0；
+/// 逐项比对本来就把 Discard 按偏移、长度、次序比（`device_log::expected_device_events` 投出它），这两个数只是让结果行看得见。
+fn count_discards(events: &[DeviceEvent]) -> usize {
+    events
+        .iter()
+        .filter(|event| matches!(event, DeviceEvent::Discard { .. }))
+        .count()
+}
+
 /// 一块盘上的比对：结果行，与这块盘对不对得上。
 struct DeviceComparison {
     line: String,
@@ -350,10 +359,12 @@
     let zero_fill_bytes: u64 = zero_fills.iter().map(|(_, length)| *length).sum();
     DeviceComparison {
         line: format!(
-            "name=device_log device={} declared_entries={} expected_writes={expected_writes} expected_flushes={expected_flushes} observed_writes={observed_writes} observed_flushes={observed_flushes} zero_fills={} zero_fill_bytes={zero_fill_bytes} trailing_flushes={} expected_events_by_window={} divergence_window={divergence_window} divergence={divergence_text}",
+            "name=device_log device={} declared_entries={} expected_writes={expected_writes} expected_flushes={expected_flushes} observed_writes={observed_writes} observed_flushes={observed_flushes} expected_discards={} observed_discards={} zero_fills={} zero_fill_bytes={zero_fill_bytes} trailing_flushes={} expected_events_by_window={} divergence_window={divergence_window} divergence={divergence_text}",
             identity.0,
             log.declared_entries
                 .map_or_else(|| "NA".to_string(), |declared| declared.to_string()),
+            count_discards(&expected),
+            count_discards(&observed),
             zero_fills.len(),
             comparison.trailing_flushes,
             events_by_window.join(","),
@@ -489,9 +500,12 @@
         declared_zero_fills, expected_device_events, DeviceEvent, DeviceLog,
     };
     use singlefs_checker_tier::on_device_modes::OnDeviceRunMode;
-    use singlefs_core::address::DeviceIdentity;
+    use singlefs_core::address::{DeviceIdentity, DeviceOffsetInBytes};
     use singlefs_harness::scenario::e142_parameters;
-    use singlefs_harness::{fnv1a_64_of_zeros, SharedStream};
+    use singlefs_harness::{
+        fnv1a_64_of_zeros, RecordedOperation, RecordedOperationKind, RetainedOperation,
+        SharedStream,
+    };
 
     /// 门禁 55 号给虚机的两块 virtio 盘各 4096 MiB（`VM_DISK_MB=4096`）。
     const VIRTIO_DEVICE_BYTES: u64 = 4096 * 1024 * 1024;
@@ -931,4 +945,84 @@
         assert_eq!(window_of_event(4, &counts), "warm_up");
         assert_eq!(window_of_event(5, &counts), AFTER_THE_PROGRAM);
     }
+
+    /// 程序录制流里有一次 discard（盘 0 上一次 discard、一道屏障，记在新池新建文件那一段）：盘上日志收到同一段的 Discard 判绿；
+    /// 日志里没有那条 Discard 判红，第一处不一致落在新池新建文件那一段，结果行报程序 1 条、盘上 0 条。
+    #[test]
+    fn a_discard_the_program_recorded_but_the_device_never_received_is_red_and_one_it_received_is_green(
+    ) {
+        let operations = vec![
+            RetainedOperation {
+                operation: RecordedOperation {
+                    device: DeviceIdentity(0),
+                    kind: RecordedOperationKind::Discard,
+                    offset: DeviceOffsetInBytes(1 << 30),
+                    length: 1 << 20,
+                    content_hash: fnv1a_64_of_zeros(1 << 20),
+                },
+                contents: None,
+            },
+            RetainedOperation {
+                operation: RecordedOperation {
+                    device: DeviceIdentity(0),
+                    kind: RecordedOperationKind::Barrier,
+                    offset: DeviceOffsetInBytes(0),
+                    length: 0,
+                    content_hash: 0,
+                },
+                contents: None,
+            },
+        ];
+        let rerun = HostRerun {
+            operations,
+            window_ends: vec![(ProgramWindow::NewPoolFileCreation, 2)],
+        };
+        let log_of = |events: Vec<DeviceEvent>| DeviceLog {
+            sector_bytes: 512,
+            declared_entries: None,
+            events,
+        };
+        let received = compare_one_device(
+            &rerun,
+            DeviceIdentity(0),
+            &log_of(vec![
+                DeviceEvent::Discard {
+                    offset: DeviceOffsetInBytes(1 << 30),
+                    length: 1 << 20,
+                },
+                DeviceEvent::Flush,
+            ]),
+        );
+        assert!(received.matches, "{}", received.line);
+        assert_eq!(field(&received.line, "expected_discards"), Some("1"));
+        assert_eq!(field(&received.line, "observed_discards"), Some("1"));
+        let never_received =
+            compare_one_device(&rerun, DeviceIdentity(0), &log_of(vec![DeviceEvent::Flush]));
+        assert!(!never_received.matches, "{}", never_received.line);
+        assert_eq!(
+            field(&never_received.line, "divergence_window"),
+            Some("new_pool_file_creation"),
+            "{}",
+            never_received.line
+        );
+        assert_eq!(field(&never_received.line, "expected_discards"), Some("1"));
+        assert_eq!(field(&never_received.line, "observed_discards"), Some("0"));
+    }
+
+    /// 新池新建文件那条写路一次 discard 都不发：每块盘上程序那一侧与盘上那一侧都是 0 条。
+    #[test]
+    fn the_new_pool_file_creation_rerun_issues_no_discard_on_either_device() {
+        let rerun = rerun_of(OnDeviceRunMode::Direct);
+        for identity in [DeviceIdentity(0), DeviceIdentity(1)] {
+            let log = device_log_that_received(
+                &rerun,
+                identity,
+                flattened(events_by_window(&rerun, identity)),
+            );
+            let comparison = compare_one_device(&rerun, identity, &log);
+            assert!(comparison.matches, "{}", comparison.line);
+            assert_eq!(field(&comparison.line, "expected_discards"), Some("0"));
+            assert_eq!(field(&comparison.line, "observed_discards"), Some("0"));
+        }
+    }
 }
diff --git a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs
--- a/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs
+++ b/crates/singlefs-checker-tier/src/bin/e142_new_pool_file_creation_write_dump_one_device.rs
@@ -72,6 +72,7 @@
         RecordedOperationKind::Write => "write",
         RecordedOperationKind::WriteForceUnitAccess => "write_fua",
         RecordedOperationKind::WriteZeroes => "write_zeroes",
+        RecordedOperationKind::Discard => "discard",
         RecordedOperationKind::Barrier => "barrier",
     }
 }
@@ -83,13 +84,12 @@
         RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
             retained.contents.clone()
         }
-        RecordedOperationKind::WriteZeroes => {
-            Some(vec![
+        // discard 按「范围写零」重建，与写清零同一份。
+        RecordedOperationKind::WriteZeroes | RecordedOperationKind::Discard => Some(vec![
                 0u8;
                 usize::try_from(retained.operation.length)
                     .expect("窗口里的写清零长度装得进 usize")
-            ])
-        }
+            ]),
         RecordedOperationKind::Barrier => None,
     }
 }
diff --git a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
--- a/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
+++ b/crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs
@@ -913,6 +913,7 @@
         RecordedOperationKind::Write => "write",
         RecordedOperationKind::WriteForceUnitAccess => "write_fua",
         RecordedOperationKind::WriteZeroes => "write_zeroes",
+        RecordedOperationKind::Discard => "discard",
         RecordedOperationKind::Barrier => "barrier",
     }
 }
@@ -978,6 +979,7 @@
                 "write" => RecordedOperationKind::Write,
                 "write_fua" => RecordedOperationKind::WriteForceUnitAccess,
                 "write_zeroes" => RecordedOperationKind::WriteZeroes,
+                "discard" => RecordedOperationKind::Discard,
                 "barrier" => RecordedOperationKind::Barrier,
                 unknown => panic!("ops.tsv 里认不出的种类 {unknown:?}"),
             };
@@ -1028,7 +1030,7 @@
         let length = usize::try_from(operation.length).expect("写长装得进 usize");
         let bytes: Option<&[u8]> = match (landing, operation.kind) {
             ('0', _) => None,
-            ('2', RecordedOperationKind::WriteZeroes) => {
+            ('2', RecordedOperationKind::WriteZeroes | RecordedOperationKind::Discard) => {
                 let end_sector = (operation.offset + operation.length) / LOCAL_SECTOR_BYTES;
                 let covered: Vec<(u32, u64)> = sectors
                     .range((operation.device, first_sector)..(operation.device, end_sector))
@@ -1483,9 +1485,9 @@
                 payload.extend_from_slice(contents);
                 (start.to_string(), sha256_hexadecimal(contents))
             }
-            RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
-                ("-".to_string(), "-".to_string())
-            }
+            RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard
+            | RecordedOperationKind::Barrier => ("-".to_string(), "-".to_string()),
         };
         operations_text.push_str(&format!(
             "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs
@@ -1561,6 +1561,7 @@
         RecordedOperationKind::Write => "write",
         RecordedOperationKind::WriteForceUnitAccess => "write_fua",
         RecordedOperationKind::WriteZeroes => "write_zeroes",
+        RecordedOperationKind::Discard => "discard",
         RecordedOperationKind::Barrier => "barrier",
     }
 }
@@ -2195,9 +2196,9 @@
                 payload.extend_from_slice(contents);
                 (start.to_string(), sha256_hexadecimal(contents))
             }
-            RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
-                ("-".to_string(), "-".to_string())
-            }
+            RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard
+            | RecordedOperationKind::Barrier => ("-".to_string(), "-".to_string()),
         };
         operations_text.push_str(&format!(
             "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
diff --git a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
--- a/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
+++ b/crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs
@@ -2492,6 +2492,7 @@
         RecordedOperationKind::Write => "write",
         RecordedOperationKind::WriteForceUnitAccess => "write_fua",
         RecordedOperationKind::WriteZeroes => "write_zeroes",
+        RecordedOperationKind::Discard => "discard",
         RecordedOperationKind::Barrier => "barrier",
     }
 }
@@ -4555,9 +4556,9 @@
                 payload.extend_from_slice(contents);
                 (start.to_string(), sha256_hexadecimal(contents))
             }
-            RecordedOperationKind::WriteZeroes | RecordedOperationKind::Barrier => {
-                ("-".to_string(), "-".to_string())
-            }
+            RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard
+            | RecordedOperationKind::Barrier => ("-".to_string(), "-".to_string()),
         };
         operations_text.push_str(&format!(
             "{stream_index}\t{}\t{}\t{}\t{}\t{payload_start}\t{content_digest}\n",
@@ -7778,7 +7779,8 @@
         match retained.operation.kind {
             RecordedOperationKind::Write
             | RecordedOperationKind::WriteForceUnitAccess
-            | RecordedOperationKind::WriteZeroes => writes_seen += 1,
+            | RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard => writes_seen += 1,
             RecordedOperationKind::Barrier => {}
         }
         if writes_seen == writes_kept {
diff --git a/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs b/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs
--- a/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs
+++ b/crates/singlefs-checker-tier/tests/crash_segments_per_device_and_torn_in_place_overwrites.rs
@@ -232,12 +232,15 @@
                 RecordedOperationKind::Barrier => 0,
                 RecordedOperationKind::Write
                 | RecordedOperationKind::WriteForceUnitAccess
-                | RecordedOperationKind::WriteZeroes => 512,
+                | RecordedOperationKind::WriteZeroes
+                | RecordedOperationKind::Discard => 512,
             },
             content_hash: 0,
         },
         contents: match kind {
-            RecordedOperationKind::Barrier | RecordedOperationKind::WriteZeroes => None,
+            RecordedOperationKind::Barrier
+            | RecordedOperationKind::WriteZeroes
+            | RecordedOperationKind::Discard => None,
             RecordedOperationKind::Write | RecordedOperationKind::WriteForceUnitAccess => {
                 Some(vec![0u8; 512])
             }
````

## m4-discard-writer：mutations-append.tsv

````tsv
m4 discard 口子：内存盘的 discard 不清零（段里还是旧字节、照样交回交给了底层）	crates/singlefs-harness/src/memory_pool.rs	        self.image.zero_fill(discarded_offset, discarded_length);	        self.image.zero_fill(discarded_offset, 0);	-p singlefs-harness --test block_devices_record_and_apply_discards -- a_discard_on_the_memory_disk_reads_back_zero	a_discard_on_the_memory_disk_reads_back_zero_and_leaves_the_neighbours
m4 discard 口子：录制器不给交给了底层的 discard 记一步	crates/singlefs-harness/src/lib.rs	        match outcome {\n            DiscardOutcome::Applied => self.stream.push(	        match DiscardOutcome::Unsupported {\n            DiscardOutcome::Applied => self.stream.push(	-p singlefs-harness --test block_devices_record_and_apply_discards -- the_recorder_records_an_applied_discard	the_recorder_records_an_applied_discard_as_one_step_without_contents
m4 discard 口子：录制器不数 discard 的结局（「不支持」那一次看不见）	crates/singlefs-harness/src/lib.rs	        self.0.borrow_mut().discard_outcome_counts.note(outcome);	        let _ = (self, outcome);	-p singlefs-harness --test block_devices_record_and_apply_discards -- a_backend_keeping_the_default_answers_unsupported	a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step
m4 discard 口子：块设备 trait 的默认 discard 交回交给了底层（不认 discard 的后端被当成认）	crates/singlefs-core/src/block_device.rs	        Ok(DiscardOutcome::Unsupported)\n    }\n    /// 屏障：之前发出的写全部持久之后才返回。	        Ok(DiscardOutcome::Applied)\n    }\n    /// 屏障：之前发出的写全部持久之后才返回。	-p singlefs-harness --test block_devices_record_and_apply_discards -- a_backend_keeping_the_default_answers_unsupported	a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step
m4 discard 口子：分段漏记 discard（写表里没有它，崩溃点重放枚举不到它的两态）	crates/singlefs-harness/src/memory_pool.rs	            RecordedOperationKind::Discard => {\n                writes.push(RetainedWrite {	            RecordedOperationKind::Discard => {}\n            RecordedOperationKind::Discard => {\n                writes.push(RetainedWrite {	-p singlefs-harness --test block_devices_record_and_apply_discards -- segments_count_a_discard_as_one_plain_write	segments_count_a_discard_as_one_plain_write_with_two_crash_states
m4 discard 口子：切段把 discard 归进写单元（种类串里不是 zero_fill）	crates/singlefs-harness/src/segments.rs	            RecordedOperationKind::Discard => StepKind::ZeroFill,	            RecordedOperationKind::Discard => StepKind::UnitWrite,	-p singlefs-harness --test block_devices_record_and_apply_discards -- segments_count_a_discard_as_one_plain_write	segments_count_a_discard_as_one_plain_write_with_two_crash_states
m4 discard 口子：内存池整个施加录制流时 discard 不清零	crates/singlefs-harness/src/memory_pool.rs	                        .expect("录到的 discard 都落在池里的盘上")\n                        .zero_fill(retained.operation.offset, retained.operation.length);	                        .expect("录到的 discard 都落在池里的盘上")\n                        .zero_fill(retained.operation.offset, 0);	-p singlefs-harness --test block_devices_record_and_apply_discards -- applying_a_recorded_discard_to_a_memory_pool	applying_a_recorded_discard_to_a_memory_pool_zero_fills_its_range
m4 discard 口子：故障注入设备不把 discard 转给里面那块盘（报交给了底层，盘上没动）	crates/singlefs-harness/src/fault_injection.rs	        let outcome = self.inner.discard_at(offset, length)?;\n        self.plan.note_forwarded_discard(self.device, outcome);	        let outcome = DiscardOutcome::Applied;\n        self.plan.note_forwarded_discard(self.device, outcome);	-p singlefs-harness --test block_devices_record_and_apply_discards -- the_fault_injecting_device_forwards_a_discard	the_fault_injecting_device_forwards_a_discard_to_the_device_inside
m4 discard 口子：故障注入设备对 discard 注入不了失败（WriteFails 照常转下去）	crates/singlefs-harness/src/fault_injection.rs	            Some(InjectedFault::WriteFails) => {\n                return Err(injected_block_device_error("discard"));\n            }	            Some(InjectedFault::WriteFails) => {}	-p singlefs-harness --test block_devices_record_and_apply_discards -- the_fault_injecting_device_makes_a_discard_fail	the_fault_injecting_device_makes_a_discard_fail_and_leaves_the_device_inside_untouched
m4 discard 口子：录制流里 discard 的种类名打成 write_zeroes	crates/singlefs-harness/src/lib.rs	            RecordedOperationKind::Discard => "discard",	            RecordedOperationKind::Discard => "write_zeroes",	-p singlefs-harness --test block_devices_record_and_apply_discards -- the_discard_kind_has_its_own_stream_name	the_discard_kind_has_its_own_stream_name
m4 discard 口子：文件后端 discard 不打洞却交回交给了底层（段里还是旧字节）	crates/singlefs-core/src/block_device.rs	                    punch_hole_keeping_the_file_length(\n                        &self.file,\n                        discarded_offset,\n                        discarded_length,\n                    )	                    Ok(())	-p singlefs-core --lib -- block_device::tests::a_discard_on_the_file_backend_punches	a_discard_on_the_file_backend_punches_the_range_back_to_zero_and_leaves_the_neighbours
m4 discard 口子：文件后端打洞报不支持时 discard 报错（不交回不支持）	crates/singlefs-core/src/block_device.rs	                    self.discard_outcome_counts\n                        .note(DiscardOutcome::Unsupported);\n                    return Ok(DiscardOutcome::Unsupported);	                    return Err(BlockDeviceError::InputOutput(discard_punch_error));	-p singlefs-core --lib -- block_device::tests::a_discard_where_hole_punching_is_unsupported	a_discard_where_hole_punching_is_unsupported_answers_unsupported_and_leaves_the_old_bytes
m4 discard 口子：文件后端 discard 的打洞报 EIO 被吞成不支持	crates/singlefs-core/src/block_device.rs	                    return Err(BlockDeviceError::InputOutput(discard_punch_error));	                    return Ok(DiscardOutcome::Unsupported);	-p singlefs-core --lib -- block_device::tests::a_discard_whose_hole_punch_fails_with_an_input_output_error	a_discard_whose_hole_punch_fails_with_an_input_output_error_is_reported_and_changes_nothing
m4 discard 口子：文件后端 discard 被信号打断不重打（当成不支持）	crates/singlefs-core/src/block_device.rs	                HolePunchFailure::InterruptedBySignal => {\n                    interrupted_in_this_discard += 1;	                HolePunchFailure::InterruptedBySignal => {\n                    return Ok(DiscardOutcome::Unsupported);\n                }\n                HolePunchFailure::InterruptedBySignal => {\n                    interrupted_in_this_discard += 1;	-p singlefs-core --lib -- block_device::tests::a_discard_interrupted_by_a_signal_twice	a_discard_interrupted_by_a_signal_twice_in_a_row_is_punched_a_third_time
m4 discard 口子：文件后端 discard 不核对齐与越界（没对齐的请求照打洞）	crates/singlefs-core/src/block_device.rs	        self.check_request(discarded_offset, discarded_length)?;	        let _ = self.check_request(discarded_offset, discarded_length);	-p singlefs-core --lib -- block_device::tests::a_discard_off_the_block_boundaries_or_past_the_end	a_discard_off_the_block_boundaries_or_past_the_end_is_refused_before_touching_the_file
m4 discard 口子：O_DIRECT 后端的 discard 不数「不支持」	crates/singlefs-core/src/block_device.rs	        self.discard_outcome_counts\n            .note(DiscardOutcome::Unsupported);\n        Ok(DiscardOutcome::Unsupported)	        Ok(DiscardOutcome::Unsupported)	-p singlefs-core --lib -- block_device::tests::the_direct_backend_answers_every_discard_unsupported	the_direct_backend_answers_every_discard_unsupported_on_both_arms_and_counts_it
m4 discard 口子：设备日志判器的期望侧仍跳过 Discard（程序发了、盘上没收到也判绿）	crates/singlefs-checker-tier/src/device_log.rs	                events.push(DeviceEvent::Discard {\n                    offset: operation.offset,\n                    length: operation.length,\n                });	                continue;	-p singlefs-checker-tier --lib -- device_log::	device_log::tests::a_discard_in_the_recorded_stream_must_reach_the_device_log_as_the_same_range_in_the_same_order
m4 discard 口子：设备日志判器的期望侧仍跳过 Discard（宿主检查把盘上缺 discard 判成对得上）	crates/singlefs-checker-tier/src/device_log.rs	                events.push(DeviceEvent::Discard {\n                    offset: operation.offset,\n                    length: operation.length,\n                });	                continue;	-p singlefs-checker-tier --bin new_pool_file_creation_device_log_check	a_discard_the_program_recorded_but_the_device_never_received_is_red_and_one_it_received_is_green
m4 discard 口子：宿主检查结果行的 discard 条数数成 FLUSH 条数	crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs	        .filter(|event| matches!(event, DeviceEvent::Discard { .. }))	        .filter(|event| matches!(event, DeviceEvent::Flush))	-p singlefs-checker-tier --bin new_pool_file_creation_device_log_check	the_new_pool_file_creation_rerun_issues_no_discard_on_either_device
````
