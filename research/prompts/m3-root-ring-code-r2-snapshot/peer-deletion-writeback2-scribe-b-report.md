# 写回员乙报告（批次 records/2026-10-01-删除与并发事务.md#写回2）

## 一、产物与原样输出

- 规格：`/tmp/claude-1000/m3-writeback2/scribe-b/spec.json`（64 条，sha256 `5a318e033f099e621d7e359a5343c33f4ecfab2ed11f37db469f426d0e02a445`）与同内容的 `spec.md`（sha256 `52dbfdbd02203eed3d1f52c6d42bcf014964e9347e55b5621f9f8acdd67967cb`）；生成脚本 `build_spec.py`、登记行新内容 `rows.py`、新增行 `newrows.py` 都在同一目录。**每一处的旧串与新串逐字在 `spec.md`**（含 20 行登记行的整行旧串与整行新串）；本报告第四节另列非登记行的短串，供转给快照线倒推。
- `kb-spec-check.py` 末行原样：`✓ 规格 64 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 64 条）`
- 写范围闸预检：7 个文件各退 0（检出记录 `precheck.jsonl`）。
- `replace-batch.py --dry-run` 原样：`✓ 64 处都命中得对（7 个文件），--dry-run 没写`
- `replace-batch.py` 实写原样（`replace-batch-output.txt`）：`✓ 64 处替换，写了 7 个文件（改名换上新 inode），回读一致`，退出码 0。
- 没跑 `doc-decisions.sh --write`（派发提示：并行写回，由主 agent 在全部交回后跑）；没碰 `decisions.md`、`decisions-history.md`、`decisions/`、`experiments/`，两份总表前后 sha256 相同（见第五节）。

## 二、7 条处理结论

1. **U1 欠账**：新增 C680（删除类发布被拒各算一次准入的计数没有用例）。现查：`crates/singlefs-core/src/mounted_session.rs` 第 591 行与 `crates/singlefs-core/src/mount.rs` 第 4771 行两处清零还在，`crates/mutations.tsv` 里 `publishes_pushed_in_this_admission` 0 处；写明用户定「记欠账，不加开关」。
2. **r3 判决第四节五条**：新增 C681（写账两处口径不同且错值不带出处）、C682（模型的映射树节点数上界只涨不落，GE）、C683（池级 checker 没有「掉了一块盘」的读法）、C684（双故障形下已分配统计该不该并被抛弃根的槽，读法甲 / 乙等用户定）、C685（LR5 小世界没罩到的四样）。取号前现查最大号 C679，插在 C679 之后；写后 C680–C685 各 1 行、无重号。LR5 补丁已在主仓（`walk.rs` 里有 `deferred_selection_merging_the_abandoned_roots_too_selects_the_same_slots_on_every_small_world_but_the_double_fault_form`、`crates/mutations.tsv` 有 LR5 行、新用例文件在），所以 C684 / C685 写的是已落地的现状。
3. **「部分已还」21 行**：C579（判据在单故障合法状态上也拒可写）早已在已还清表（今天第 587 行附近，D23 已定项 14 射程与变更史里都有「维持乙字面」），不动。其余 20 行逐行现查，**还欠的那一半今天都还在**（逐行依据见第三节），没有一行可以整条移进已还清表；20 行都改写成只写还欠的那一截，已还的那一半（条款已写、门禁已落）从登记行里拿掉。
4. **milestone/02 旧数**：第 76 行（8 个单元、两盘 16 次写 → 12 个单元、两盘 24 次写）、第 107 行后半句（改成「上一次发布的系统配置槽写自成一段、不与这次的单元写同段」，指到 layout/01 八的发布末尾屏障那条 ⚠️）、第 258 行（21 次写、8 个单元 → 29 次写、12 个单元）。现核：layout/01 零「写请求数」29 条 = 12 × 2 + 2 + 1 + 2，`overwrite_in_one_instance.rs` 钉 B 的 12 个单元 24 次写、落点 50253–50264。
5. **layout/01 第 413 行 ⚠️ 与管理员回退行**：重核单元组成（`transaction.rs` 的 `PublishShape::rewritten_roles`：实例表页 + 分配记录树 + 记账树 + 映射树 + 树表，分配记录树按位置寻址重写 5 个节点；`crash_enumeration_fixed_script_stream.rs` 钉写行段 18 个单元写、暖机与抬 F 空发布段各 16）⇒ 写行 9 个单元（18 次写），空发布 8 个单元（16 次写）。旧 ⚠️ 的前提（孤立看 10、8 对不上）不成立，改成相符；孤立看写成 `2+18+2+1+2` 与 `16+2+1+2`。管理员回退那一行补了发布尾屏障。
6. **U7**：现查 `grep 'format-const: \(TOMB\|LINEAGE\|DELETED\)'` 在 `crates/singlefs-format/src/lib.rs` 与 kb 都是 0 行，没加过。已在 `layout/03-third-txn.md`「一、形态」表后加五行标记（`TOMBSTONE_RECORD_BYTES = 40`、`TOMBSTONE_CONTAINER_RECORDS = 815`、`LINEAGE_RECORD_BYTES = 16`、`LINEAGE_CONTAINER_RECORDS = 2039`、`DELETED_INODES_ENTRY_BYTES = 8`），值取自 `lib.rs` 第 113–125 行；没带 `stale=`（新常量没有旧值）。**`code-source-discipline.sh --check format-constants` 被 heavy-test-guard 拒（提交时才跑的检查），没跑**，见第六节。
7. **回扫旧树值**：见第四节；按建议新串改了 `invariants.md` 2 处、`layout/01` 的第 48 行与 386、387 行、`layout/02` 2 处、`milestone/01` 9 处、`milestone/02` 6 处、`checks-owed.md` C618 1 处。未改：`layout/01` 第 36 行（等 E142 第 20 次重跑）、第 49、78 行（对方已改过）；「要人看」的 `milestone/01` 第 134、179 行不改。

## 三、20 行「部分已还」逐行现查（还欠的那一半今天还在不在）

都是只读现查，没跑测试。「在」= 2026-09-28 核实表（`records/2026-09-28-里程碑三第十项清单.md`「疑似已还的 52 行」）写的还欠，今天仍成立；「变」= 还欠的那一半今天比核实表又少了一块，登记行已按现值改。

| 行 | 结论 | 现查依据 |
|---|---|---|
| C66 | 在 | `.claude/gate.d/doc-registries.sh` field-refs 格文件头「「无分项」合法」；`layout/01-first-txn.md` 里「无分项」`grep -c` 为 0；field-projection 格文件头写「只判在不在，不判宽度」 |
| C78 | 在 | 同上（根槽段是同一张表）；C68 登记行「仍欠」② 仍开着 |
| C236 | 在 | `invariants.md` I-3.5 射程注「那条不变量还没立」；`walk.rs` 注释「条目格式还没有条款的树（livelist、稀疏旁表、deadlist…）」不走它们的条目 |
| C237 | 在 | `grep 'format-const: .*POINTER' .claude/kb` 0 行；`crates/mutations.tsv` 里 `POINTER_HEAD` 0 行；等式断言在 `crates/singlefs-format/src/lib.rs` 第 482、487 行 |
| C248 | 变 | 条款那一半已写：D19 已定项 8 射程「不进映射的三类（自举豁免）」点名树表单元；原前置「C196 未答」不成立（C196 登记行写映射根住根记录，条款已写）；挂载关键链检查 `grep 自举 crates/singlefs-checker/src` 只命中系统配置自举头 |
| C277 | 在 | 现测：一份只含 `` `invariants.md:1638` `` 的报告交 `cite-check.py`，输出「核了 0 处引文，对上 0 处，没判 0 处」 |
| C280 | 在 | `grep -rln '自举失败\|Bootstrap' crates --include=*.rs` 0 个文件；`invariants.md` 无「不进映射三类」行 |
| C297 | 变 | checker 已判子指针出生树（`walk.rs` 的 `child.birth_tree == identity.0`）；`crates/mutations.tsv` 里 `child.birth_tree` 0 行，无坏镜像用例（`corrupt_on_disk_content_is_refused_instead_of_panicking.rs` 只造了容器出生代加一那一形） |
| C307 | 变 | D19 已定项 10 已定 27 / 55；`parse_mapping_entry` 只拒窄于 55，`walk.rs` 对映射树 `GuardedForTheWalkOnly`，`field_table_width_of_tree_kind` 不认种类 5；代码注释「C307 还开着」过时（`records.rs` 第 534–535 行、`walk.rs` 多处、`mounted_read_by_offset.rs` 第 1251 行） |
| C308 | 在 | `layout/01` 里 `format-const` 只登总宽与单元头偏移（`DATA_UNIT_PAYLOAD_OFFSET`、`PACKED_UNIT_RECORDS_OFFSET`），无根记录 / 系统配置 / journal 头的逐字段偏移；field-projection 格不判偏移 |
| C309 | 变 | checker 自己有按位 CRC32C（`crates/singlefs-checker/src/lib.rs` 第 38 行 `crc32_castagnoli_bitwise`），核实表写的「checker 侧那份独立实现一行没有」不成立；仍欠：`CHECKSUM_ALGORITHM_CRC32_CASTAGNOLI` 只用在写槽（`system_configuration.rs` 第 454 行），读侧不判；`crates/mutations.tsv` 里 `checksum.rs` 0 行 |
| C312 | 变 | 位 0 已退役、位 1 是布局身份（D15 已定项 4）；core 读者「一个布局身份位都不带」有单测（`system_configuration.rs` 第 808 行），checker 第 265–270 行第三支只有 harness 用例造「只带位 0」「位 0 与位 1」，`mutations.tsv` 第 687、692、695 行只改位号或 core 读者 |
| C478 | 在 | `walk.rs` 第 2396 行注释「C478 … 还开着」，区间仍贴紧首末条目；D18 已定项 2 欠 C478 |
| C482 | 变 | 分支名已落地（`mount.rs` 第 793 行 `branch_name`、`shadow_ledger_branch`）；`rollback_by_a_forward_publish.rs` 第 1059 行只断言等于传进去那一臂的 `branch_name()`，`mutations.tsv` 第 242 行（原核实表写第 245 行）把两臂合成一个名字，该断言两边同变（推的，没跑） |
| C487 | 变 | 模型那一半与样本都追平（样本 mkfs 行 `operations=23`，`replay.sh` 登记 r19 产物，样本 PROVENANCE.txt 记 2026-10-02 真起虚机判绿）；仍欠：`checker-tier-qemu-device-streams.sh` 第 281 行那一支 diff 无红样本（red 与 direct-divergence-red 只造设备侧缺项） |
| C492 | 变 | R1 历史已走得通（`layout/01` 第 416 行；`a_formatted_pool_mounted_twice_…` 用例）；`crash_enumeration_writable_mount_of_a_formatted_pool.rs` 只覆盖一次可写挂载，层 0 上无 R1 用例；`mutations.tsv` 无「改回拒绝」行 |
| C504 | 在 | 守卫、坏法、变异行（`mutations.tsv` 第 259 行）都在；`research/prompts/*main-verification.md` 与 `records/` 里无点名 C504 的判决；提交时 checker 档没跑过（本轮不跑） |
| C506 | 变 | S 已是系统配置字段、挂载判区间（`root_ring.rs`）；`grep 'format-const: ROOT_RING' .claude/kb` 只有 `ROOT_RING_CHUNK_BYTES`，区间两条边仍无 kb 登记位；上界 16 在 `lib.rs` 标 placeholder |
| C511 | 变 | 批 13 的 harness 用例已在主仓（`tree_identifier_watermark_after_the_first_file_root_is_abandoned.rs`，变异行 `mutations.tsv` 第 317 行已换点名它）；它不经池级 checker，所以「checker 全绿、I-7.8 / I-9.14 不红」仍欠；回退到无文件版本那一形随「候选要带文件」不可达（`mutations.tsv` 第 726 行） |
| C564 | 在 | `stage-inputs.tsv` 现有 9 条 `crash-case:`；floor-raise、c561-sigma、tree-split、position-addressed 四行写明不留进度文件；floor-raise 与 position-addressed 无 `exhaustive=`；`ls $(git rev-parse --git-common-dir) | grep -c singlefs-crash-case-green` 为 1（只有 `layer0-first-stream`） |

## 四、非登记行的旧串与新串（编号 = spec.json / spec.md 里的第几条）

第 1–20 条是 20 行登记行（整行旧串、整行新串在 `spec.md`），第 21 条是 C679 行后插 C680–C685（同在 `spec.md`）。下面从第 22 条起，逐条。

**#22** `.claude/kb/checks-owed.md`
- 旧串：`FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: 7 }`
- 新串：`FirstFileVersionOnAVersionThatAlreadyHasAFile { tree_table_entries: 11 }`
**#23** `.claude/kb/checks-owed.md`
- 旧串：[invariants.md](invariants.md) 那一行的 ② 也只列七种 ⇒
- 新串：[invariants.md](invariants.md) 那一行的 ② 已按实现列十一种 ⇒
**#24** `.claude/kb/checks-owed.md`
- 旧串：；I-9.16（树表条目按树 ID 严格升序且合发号次序） 那一行 ② 照实现补全；判别力自证
- 新串：；判别力自证
**#25** `.claude/kb/invariants.md`
- 旧串：不写死 15：回退到无文件那一版之后再发第一个文件版本，八棵树从水位起重新发号
- 新串：不写死 65540：回退到无文件那一版之后再发第一个文件版本，十二棵树从水位起重新发号
**#26** `.claude/kb/invariants.md`
- 旧串：（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist；中央映射树不进树表）排
- 新串：（extent、inode、分配记录、记账、livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops；中央映射树不进树表）排
**#27** `.claude/kb/layout/01-first-txn.md`
- 旧串：deadlist 树 65543（D5（快照 / 空间记账机制） 已定项 11，2026-09-14 用户定案 day-1 注册）；
- 新串：deadlist 树 65543（D5（快照 / 空间记账机制） 已定项 11，2026-09-14 用户定案 day-1 注册）、`deleted_inodes` 树 65544、墓碑树 65545、谱系树 65546、logged_ops 树 65547（D18（块里携带什么信息） 已定项 24 第 6 条）；
**#28** `.claude/kb/layout/01-first-txn.md`
- 旧串：树 ID 段写 0、层级 0、key 宽 8、条目数 7、条目宽 200、声明长度 1400
- 新串：树 ID 段写 0、层级 0、key 宽 8、条目数 11、条目宽 200、声明长度 2200
**#29** `.claude/kb/layout/01-first-txn.md`
- 旧串：第 1 版 **7 条**（按树 ID 升序）
- 新串：第 1 版 **11 条**（按树 ID 升序）
**#30** `.claude/kb/layout/01-first-txn.md`
- 旧串：(65543, deadlist, 8, 根指针零, 头 ID 0)，七条诞生 txg 都是 3
- 新串：(65543, deadlist, 8, 根指针零, 头 ID 0)、(65544, `deleted_inodes`, 9, 根指针零, 头 ID 65537)、(65545, 墓碑树, 10, 根指针零, 头 ID 65537)、(65546, 谱系树, 11, 根指针零, 头 ID 0)、(65547, logged_ops, 12, 根指针零, 头 ID 0)，十一条诞生 txg 都是 3
**#31** `.claude/kb/layout/01-first-txn.md`
- 旧串：[根 FUA] [系统配置槽 × 2 盘]，孤立看与普通发布同型
- 新串：[根 FUA] [系统配置槽 × 2 盘] 屏障（发布末尾那道，D16（发布语义） 已定项 7），孤立看与普通发布同型
**#32** `.claude/kb/layout/01-first-txn.md`
- 旧串：[实例表单元（写行）+ 分配记录 + 记账 + 映射 + 树表 = 5 单元 × 2 盘 = 10]
- 新串：[实例表单元（写行）+ 分配记录树 5 个节点 + 记账 + 映射 + 树表 = 9 单元 × 2 盘 = 18]
**#33** `.claude/kb/layout/01-first-txn.md`
- 旧串：⇒ 孤立看 `2+10+2+1+2`；
- 新串：⇒ 孤立看 `2+18+2+1+2`；
**#34** `.claude/kb/layout/01-first-txn.md`
- 旧串：⚠️ 用例钉的写行那一段是 18 个单元写、之后暖机那一段是 16 个，与这一行和「空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元）」那一行孤立看的 10、8 对不上（C577（系统配置没见证到的最新根，乙罩不到） 之前的数组里已是 18、16），单元组成没按用例重核；
- 新串：用例钉的写行那一段 18 个单元写、之后暖机那一段 16 个，与这一行和「空发布（暖机，后续可写挂载的实例，写 c_max 个固定点单元）」那一行孤立看的 18、16 相符（分配记录树按位置寻址之后每次重写 5 个节点：两块盘的叶各一、层级 1 节点各一、根一，同新池新建文件的 t5..t9，D8（核心索引结构） 已定项 14）；
**#35** `.claude/kb/layout/01-first-txn.md`
- 旧串：[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `8+2+1+2`，与首次挂载的暖机不同型：记账树已经存在，空发布也重写记账行连带四个固定点单元（D16（发布语义） 已定项 9），这条脚本上 c_max = 4；
- 新串：[分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 单元 × 2 盘 = 16] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `16+2+1+2`，与首次挂载的暖机不同型：记账树已经存在，空发布也重写记账行连带分配记录树的 5 个节点、映射和树表单元（D16（发布语义） 已定项 9），这条脚本上一次空发布写 8 个单元；
**#36** `.claude/kb/layout/01-first-txn.md`
- 旧串：（第二条流第 20–21、24–25 段各 `2+16`；用例钉 16 个单元写，与孤立看的 8 对不上，写在「第一次之后的可写挂载（写行）」那一行）；
- 新串：（第二条流第 20–21、24–25 段各 `2+16`，用例钉的 16 个单元写与孤立看的 16 相符）；
**#37** `.claude/kb/layout/01-first-txn.md`
- 旧串：与后续挂载的暖机空发布同型——[分配记录 + 记账 + 映射 + 树表 = 4 单元 × 2 盘 = 8] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `8+2+1+2`，
- 新串：与后续挂载的暖机空发布同型——[分配记录树 5 个节点 + 记账 + 映射 + 树表 = 8 单元 × 2 盘 = 16] 屏障 [记录 × 2 盘] 屏障 [根 FUA] [系统配置槽 × 2 盘] 屏障 ⇒ 孤立看 `16+2+1+2`，
**#38** `.claude/kb/layout/01-first-txn.md`
- 旧串：（第二条流第 56–65 段 `2+2+16+2+1+2+16+2+1+2`；用例钉 16 个单元写，与孤立看的 8 对不上，写在「第一次之后的可写挂载（写行）」那一行）；
- 新串：（第二条流第 56–65 段 `2+2+16+2+1+2+16+2+1+2`；用例钉的 16 个单元写与孤立看的 16 相符）；
**#39** `.claude/kb/layout/02-second-txn.md`
- 旧串：发布 B 换下 A 的八个单元
- 新串：发布 B 换下 A 的十二个单元
**#40** `.claude/kb/layout/02-second-txn.md`
- 旧串：B 之后 11 槽
- 新串：B 之后 15 槽
**#41** `.claude/kb/layout/03-third-txn.md`
- 旧串：## 二、墓碑记录
- 新串：<!-- format-const: TOMBSTONE_RECORD_BYTES = 40 -->
<!-- format-const: TOMBSTONE_CONTAINER_RECORDS = 815 -->
<!-- format-const: LINEAGE_RECORD_BYTES = 16 -->
<!-- format-const: LINEAGE_CONTAINER_RECORDS = 2039 -->
<!-- format-const: DELETED_INODES_ENTRY_BYTES = 8 -->

## 二、墓碑记录
**#42** `.claude/kb/milestone/01-first-txn.md`
- 旧串：livelist 65541、稀疏旁表 65542、deadlist 65543，树 ID 水位发布后 **65548**
- 新串：livelist 65541、稀疏旁表 65542、deadlist 65543、`deleted_inodes` 65544、墓碑树 65545、谱系树 65546、logged_ops 65547，树 ID 水位发布后 **65548**
**#43** `.claude/kb/milestone/01-first-txn.md`
- 旧串：树表第 1 版 **7 条**（D19（块指针的结构与宽度预算） 已定项 11）
- 新串：树表第 1 版 **11 条**（D19（块指针的结构与宽度预算） 已定项 11）
**#44** `.claude/kb/milestone/01-first-txn.md`
- 旧串：装**七条**树表条目（extent、inode、分配记录、记账四条有根，livelist、稀疏旁表、deadlist 三条根指针为零；映射树的根住根记录）
- 新串：装**十一条**树表条目（extent、inode、分配记录、记账四条有根，livelist、稀疏旁表、deadlist、`deleted_inodes`、墓碑树、谱系树、logged_ops 七条根指针为零；映射树的根住根记录）
**#45** `.claude/kb/milestone/01-first-txn.md`
- 旧串：树的种类码按 D8（核心索引结构） 已定项 9 的登记表、树 ID 65536..65543）。
- 新串：树的种类码按 D8（核心索引结构） 已定项 9 的登记表、树 ID 65536..65547，中央映射树 65540 不进树表）。
**#46** `.claude/kb/milestone/01-first-txn.md`
- 旧串：D6（快照实现模型） 已定项 2（livelist 共享树 day-1 进树表：树表第 1 版第 6 条、根指针零
- 新串：D6（快照实现模型） 已定项 2（livelist 共享树 day-1 进树表：树表第 1 版第 5 条、根指针零
**#47** `.claude/kb/milestone/01-first-txn.md`
- 旧串：树表 6 条条目的 2 字节码按登记表写
- 新串：树表 11 条条目的 2 字节码按登记表写
**#48** `.claude/kb/milestone/01-first-txn.md`
- 旧串：- 分配记录树：一个根兼叶节点，装 20 条记录（mkfs 的两个单元、步 2 的数据单元、这一步与步 4 / 步 5 写出的 7 个提交内生块，每个两盘各一条；自己那条的落点在写出前已定，D3（空间分配） 已定项 5）。
- 新串：- 分配记录树：5 个节点（两片叶各装本盘 14 条记录、两个层级 1 节点、一个层级 2 的根，D8（核心索引结构） 已定项 14），装 28 条记录（mkfs 的两个单元、步 2 的数据单元、这一步与步 4 / 步 5 写出的 11 个提交内生块，每个两盘各一条；自己那五个节点的落点在写出前已定，D3（空间分配） 已定项 5）。
**#49** `.claude/kb/milestone/01-first-txn.md`
- 旧串：t7 50247、t8 50248；槽 50241 被 t3 的 32768 对齐跳过，空着。
- 新串：t7 50247、t8 50248、t9 50249、t10 50250、t11 50251、t12 50252；槽 50241 被 t3 的 32768 对齐跳过，空着。
**#50** `.claude/kb/milestone/01-first-txn.md`
- 旧串：新池新建文件 6 条条目、一个 16 KiB 根兼叶节点。
- 新串：新池新建文件 10 条条目（码 1 一条 + 码 2 / 码 3 九条）、一个 16 KiB 根兼叶节点。
**#51** `.claude/kb/milestone/01-first-txn.md`
- 旧串：新池新建文件的 8 个数据单元来自一次请求、按 key 升序取号
- 新串：新池新建文件的 12 个单元来自一次请求、按 key 升序取号
**#52** `.claude/kb/milestone/01-first-txn.md`
- 旧串：（8，一项含两盘的位置条目；
- 新串：（12，一项含两盘的位置条目；
**#53** `.claude/kb/milestone/01-first-txn.md`
- 旧串：t9 的反向链
- 新串：t13 的反向链
**#54** `.claude/kb/milestone/01-first-txn.md`
- 旧串：事务 21 条写请求
- 新串：事务 29 条写请求
**#55** `.claude/kb/milestone/02-second-txn.md`
- 旧串：新池新建文件那一档（8 个单元、两盘 16 次写）能
- 新串：新池新建文件那一档（12 个单元、两盘 24 次写）能
**#56** `.claude/kb/milestone/02-second-txn.md`
- 旧串：一次覆盖写的段序列预想与新池新建文件同型 `16+2+1+2`
- 新串：一次覆盖写的段序列预想与新池新建文件同型 `24+2+1+2`
**#57** `.claude/kb/milestone/02-second-txn.md`
- 旧串：验收（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs`）：段序列 `16+2+1+2`、
- 新串：验收（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs`）：段序列 `24+2+1+2`、
**#58** `.claude/kb/milestone/02-second-txn.md`
- 旧串：落点 t12 50182、提交内生块 50249–50256 接着 A 的 bump
- 新串：数据单元落 50182、提交内生块 50253–50264 接着 A 的 bump
**#59** `.claude/kb/milestone/02-second-txn.md`
- 旧串：六个提交内生块（extent 树根、inode 树根与叶容器、分配记录树根、记账树根、映射树根）与树表单元各 COW 出新版本
- 新串：十个提交内生块（extent 树根、inode 树根与叶容器、分配记录树五个节点、记账树根、映射树根）与树表单元各 COW 出新版本
**#60** `.claude/kb/milestone/02-second-txn.md`
- 旧串：（**预想** 8：数据单元 1 + 提交内生块 6 + 树表单元 1）
- 新串：（**预想** 12：数据单元 1 + 提交内生块 10 + 树表单元 1）
**#61** `.claude/kb/milestone/02-second-txn.md`
- 旧串：段序列 **预想** 与新池新建文件同型 `24+2+1+2`，且上一次发布的系统配置槽写与这次的单元写落在同一段（登记表末尾那条 ⚠️）。
- 新串：段序列 **预想** 与新池新建文件同型 `24+2+1+2`，且上一次发布的系统配置槽写自成一段、不与这次的单元写同段（发布末尾那道屏障，D16（发布语义） 已定项 7；[layout/01-first-txn.md](../layout/01-first-txn.md) 八「发布与下一次发布之间有发布末尾那道屏障」那条 ⚠️）。
**#62** `.claude/kb/milestone/02-second-txn.md`
- 旧串：释放 B 的八个落点
- 新串：释放 B 的十二个落点
**#63** `.claude/kb/milestone/02-second-txn.md`
- 旧串：（**预想** 21 次写、8 个单元 × 2 盘 + 记录 × 2 + 根 + 系统配置 × 2）
- 新串：（**预想** 29 次写、12 个单元 × 2 盘 + 记录 × 2 + 根 + 系统配置 × 2）
**#64** `.claude/kb/milestone/02-second-txn.md`
- 旧串：新池新建文件一条记录点名 8 个单元（1 个数据单元 + 7 个提交内生块）
- 新串：新池新建文件一条记录点名 12 个单元（1 个数据单元 + 11 个提交内生块）

## 五、改过的文件与前后 sha256

开工时（`sha-before.txt`）→ 收尾时（`sha-after.txt`）：

```
0ed898ddb6a6133e…
  -> ccedab35e76cfc45…  .claude/kb/checks-owed.md
244b83faf6eafeb9…
  -> d65c8500977ffa6a…  .claude/kb/invariants.md
059ef5bcf87dddc6…
  -> 7a5f291e3c190c8d…  .claude/kb/layout/01-first-txn.md
ce0cdc106a013b42…
  -> e11c1d570e35cd0a…  .claude/kb/layout/02-second-txn.md
2ba2a7032ae325ee…
  -> 8d87205712ba8f3b…  .claude/kb/layout/03-third-txn.md
4e8c6208f9a37e71…
  -> b7ec2fc35ca0a8e3…  .claude/kb/milestone/01-first-txn.md
cd2dd5ab966b35cf…
  -> f409b191248d793b…  .claude/kb/milestone/02-second-txn.md
24e02f837ca8b0cd…
  -> 24e02f837ca8b0cd…  .claude/kb/decisions.md
fdf7a27c254f753a…
  -> fdf7a27c254f753a…  .claude/kb/decisions-history.md
```

前 16 位已够区分；完整值在两份文件里。改过的 7 个：`checks-owed.md`、`invariants.md`、`layout/01-first-txn.md`、`layout/02-second-txn.md`、`layout/03-third-txn.md`、`milestone/01-first-txn.md`、`milestone/02-second-txn.md`（都在 `.claude/kb/` 下）。`decisions.md` 与 `decisions-history.md` 前后相同（`24e02f83…`、`fdf7a27c…`）。

## 六、要主 agent 跑的命令与要判的点

跑（我被 heavy-test-guard 拒或按定义不跑，都是提交前整轮门禁里本来就有的）：
1. `bash .claude/gate.d/code-source-discipline.sh --check format-constants`（工作目录仓根）：看是否绿，输出里读到的 kb 标记数比改前多 5（U7 五个标记在 `layout/03-third-txn.md`）。红了先看是不是 `DELETED_INODES_ENTRY_BYTES` 在 `crates/singlefs-checker/src/walk.rs` 里还有一份同名 usize 常量（值也是 8，应当两份都对得上）。
2. `doc-registries` 整道（checks-owed 的 table-shape、declared-counts、layout/01 的 segment-registry / field-table-sums / first-txn-hooks、second-txn-hooks）与 doc-lint 的编号引用格：我只跑了 kb-spec-check 的编号简称检查。layout/01 第 413–415 行的孤立看段序列我没找到门禁钉它们的地方（`grep` 门禁与脚本，除 QEMU 阶段的 28+2+1+2 外无命中），但没跑整道。
3. 每一处的值我只读代码与用例核过，没跑测试；C482 那条「两臂同变断言照样绿」是推的。

要判 / 要人看：
- **一处顺手改**：`invariants.md` I-9.16 ② 改成十一种之后，C676（GPU 判器的树表发号次序只判七种）里「invariants.md 那一行的 ② 也只列七种」与「I-9.16 那一行 ② 照实现补全」两句已过时，我用两处定点替换改了（第 22–24 条里的后两条）。C676 是形式化接入那条线的行，请转告它。
- **同一句里的派生数顺带改了两处**：`milestone/02` 第 76 行同一句的 `16+2+1+2` 改成 `24+2+1+2`；第 105 行「点名项预想 8：数据单元 1 + 提交内生块 6 + 树表单元 1」改成 12 / 10 / 1（与第 97 行改成十个提交内生块一致）。回扫报告里没有这两处。
- **没改的同族旧数**（都在 `milestone/02`）：第 145 行「实例表 + 分配记录 + 记账 + 映射 + 树表五个单元」与「每次重写四个固定点单元、释放上一版的四个」；第 344、360 行「重写四个固定点单元」「c_max = 4，写行那次是 5」。今天写行 9 个、暖机 8 个单元；这些是 2026-09-16 / 17 落地那天的读数，改不改交主 agent。
- **回扫报告「要人看」**：`milestone/01` 第 134 行（验收读数「8 把 key 罩住 6 把」）与第 179 行（变异读数「8 个单元 ⇒ 7 对」）没改；报告第 50 条（`decisions/16-发布语义.md` 第 57 行）和第 51 条（`experiments/171-三oracle合谋测试.md` 第 86 行）不在我的写范围。
- **U7 的标记放在 `layout/03-third-txn.md`**，不在定这些值的 `decisions/18-块里携带什么信息.md` 已定项 24 旁边（那是写回员甲的文件）。门禁只要求 kb 里有一处登记；要挪到决策正文旁边，交甲或主 agent。
- **代码注释里「C307 还开着」已过时**（`crates/singlefs-core/src/records.rs` 第 534–535 行、`crates/singlefs-checker/src/walk.rs` 多处、`crates/singlefs-harness/tests/mounted_read_by_offset.rs` 第 1251 行），C307 登记行里写了；改它们归实现批。
- C579 已在已还清表，我没动它。

## 七、没做什么

- 没判定案对不对；没写规格外的句子（上面列的两处派生数与 C676 两句是规格里的，已逐条在第四节）。
- 没跑任何门禁阶段、没跑 `doc-decisions.sh --write`、没提交；没改 `crates/`；没动 `decisions/`、`experiments/` 与两份变更史。
- 没在 `relabel-item.py` 上花功夫：这一批不翻分项状态。
- 草稿目录里没有仓副本或编译目录；`t/` 是 cite-check 现测用的一个临时 md，未删（不是编译目录）。
