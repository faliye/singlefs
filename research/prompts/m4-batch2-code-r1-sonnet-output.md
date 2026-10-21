# m4-batch2-code-r1 云端正推腿（Sonnet）报告

日期 2026-10-13。立场：核「代码做的是不是判决与题面说的那件事」，分到 T1–T9 全部；T7 只核 G5 的两态与「不支持一字不动」的前提。
引文约定：代码路径一律是冻结副本 `~/.local/share/singlefs/work/m4-batch2-code-r1/repo/` 下的 `crates/…`，行号是那份文件自己的行号；判决与 kb 写仓里那份文件自己的行号。
判定词：一致 / 冲突 / 规则没说（含「本批没做」）。正文第二节「已知」清单里的项不当打中，只在后果超出清单写的范围时单列。

## 0 环境与量法

- 冻结副本核对：`sha256sum -c research/prompts/m4-batch2-code-r1-snapshot/crates-sha256.txt`（在冻结副本根里跑）16 行全部 OK，见本节末尾的原样输出。
- 开跑前 `ps -o pid,args -u` 看到的：别的会话的 `cargo test`（变异整表重放与覆盖率编译）在跑；没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`。我的每条 cargo 都加 `nice -n 19`、经 `bash scripts/hosts/run-with-memory-cap.sh 8G bash scripts/hosts/capped.sh 8 …` 起，同一时刻只一条；包装排队等过 40 秒、10 秒、79 秒，没有退出码 245 或 250–254 的结局。
- 草稿目录 `~/.local/share/singlefs/work/m4-batch2-code-r1-forward/`：`repo/` 是冻结副本 `crates/`、`Cargo.toml`、`Cargo.lock` 的拷贝（我只在里面加了 `probe_forward_snapshot.rs`、`probe_forward_scan_rebuild.rs`、`probe_forward_tearable.rs` 三个测试文件，并临时改过几行做变异，改完已用冻结副本的文件盖回）；`build.sh` 是起 cargo 的壳；日志 `build1.log`、`probe1.log`、`mut2.log`、`mut3.log`、`mut4.log`、`mut5.log`。
- 模型目录 `research/prompts/m4-batch2-code-r1-sonnet-model/`：`probes/` 是两段追加进现成测试文件的探针测试，`tools/rerun.sh` 五步复跑（探针加三组变异），`out/expected.txt` 是我这次跑出来的摘要。复跑命令在本报告「要主 agent 跑的命令」一节。
- 没有写 Rust 以外的模型；下面的数都是在冻结副本的拷贝上跑原有测试加我的探针得到的，不是抽象模型。
- 不跑：门禁阶段、checker 档测试、GPU、`crash_amplification*`、WGSL。凡要这些才能核的，写「复核不了」。

冻结副本 sha 核对的原样输出（命令：`cd ~/.local/share/singlefs/work/m4-batch2-code-r1/repo; sha256sum -c …/crates-sha256.txt`）：
crates/singlefs-core/src/block_device.rs: OK
crates/singlefs-core/src/lib.rs: OK
crates/singlefs-core/src/mounted_read.rs: OK
crates/singlefs-core/src/records.rs: OK
crates/singlefs-core/src/recovery.rs: OK
crates/singlefs-core/src/logged_operations.rs: OK
crates/singlefs-core/src/region_scan.rs: OK
crates/singlefs-core/src/compaction_plan.rs: OK
crates/singlefs-format/src/lib.rs: OK
crates/singlefs-checker/src/rebuild.rs: OK
crates/singlefs-checker/src/walk.rs: OK
crates/singlefs-checker/src/image.rs: OK
crates/singlefs-checker/src/lib.rs: OK
crates/singlefs-checker-tier/src/crash_judge_gpu.rs: OK
crates/singlefs-checker-tier/src/device_log.rs: OK
crates/mutations.tsv: OK

## T1 快照条目的配对判定

**判定**：四样落在判决字面上：一致。六种镜像的落点：一致。读侧之外 S1 点名的 checker 三处：规则没说本批做不做（本批没做）。用例对「码 14 比码 15 多一条」没有覆盖：我构造了这个镜像并量到一个存活变异。

1. 判决 S1 要的读侧条件，原文：
   - `research/prompts/m4-snapshot-r1-main-verification.md:30`：「码 14 = 快照 inode 根、码 15 = 快照 extent 根」，「头 ID = 活头 inode 树的树 ID」，「码 15 的头 ID = 配对的码 14 的树 ID」。
   - 同文件 `:65`（岔路 1 用户定案）：「第一版同时活着的只读快照至多 10 个」。
   - 代码四样各在一行：条数相等 `crates/singlefs-core/src/recovery.rs:2422`「if snapshot_inode_root_entries.len() != snapshot_extent_root_entries.len() {」；一一配对 `:2435`「&& extent_root_already_paired.is_none()」；至多 10 条 `:2442`「if snapshot_inode_root_entries.len() > SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION {」；码 14 头 ID `:2450`「Some(TreeIdentifier(inode_root_entry.head_identifier)) != live_inode_tree」。常量 `crates/singlefs-core/src/records.rs:438`「pub const TREE_KIND_SNAPSHOT_INODE_ROOT: u16 = 14;」、`:441`「pub const TREE_KIND_SNAPSHOT_EXTENT_ROOT: u16 = 15;」、`:444`「pub const SNAPSHOTS_AT_MOST_IN_THE_FIRST_VERSION: usize = 10;」。
   - 这四样就是 S1 与岔路 1 对读侧写下的全部字面条件（头 ID 两条、配对、上限）；「条数相等」是「一一配对」的另一半，代码把它放在配对之前单判。判决没有写别的读侧条件。
2. 六种镜像各落在哪条拒绝上（`recovery.rs:2327–2342` 四条文案；经 `From` 转成 `RecoveryFailure::UnitMalformed`）：

   | 镜像 | 落点 | 证据 |
   |---|---|---|
   | 两条码 15 指同一条码 14 | 第二条找不到「头 ID 指着它又还没被配过」的码 14 ⇒ `…WithoutItsOwn…` | `:2435` 加后面的 `.ok_or(`；用例 `two_snapshot_extent_roots_claiming_the_same_snapshot_inode_root_are_refused_by_every_reader`（五个读者）|
   | 码 15 比码 14 多一条 | `…CountsDiffer` | `:2422`；用例 `a_tree_table_with_one_more_snapshot_extent_root_than_snapshot_inode_roots_…` |
   | 码 14 比码 15 多一条 | `…CountsDiffer` | `:2422`；**六条用例里没有这个镜像**；我的探针 P2（三个码 14、两个码 15）五个读者都报「树表里快照 inode 根与快照 extent 根的条数不相等」，见下 |
   | 第 11 条 | 先过条数与配对，再在 `:2442` 拒 ⇒ `MoreSnapshotsThanTheFirstVersionAllows` | 用例 `an_eleventh_snapshot_is_refused_by_every_reader_while_ten_are_rebuilt`（10 对放行、11 对拒）|
   | 码 14 的头 ID 指别的头 | `:2450` ⇒ `SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree`；表里没有码 2 条目时同一句 | 用例 `a_snapshot_inode_root_whose_head_is_not_the_live_inode_tree_…` |
   | 码 15 的头 ID 指不到码 14 | `…WithoutItsOwn…` | 用例 `a_snapshot_extent_root_whose_head_is_not_any_snapshot_inode_root_…` |

3. `tree_table_entries_each_kind_at_most_once` 没把别的种类放过去：只有码 14、15 在 `crates/singlefs-core/src/recovery.rs:2485`「if is_a_snapshot_entry {」里 `continue`，其余种类仍走 `:2489`「if !kinds_seen.insert(entry.kind) {」判两条。别的种类两条的拒绝由原有的变异行 `crates/mutations.tsv:934` 与用例文件 `core_review_tree_table_duplicates_and_slot_one_search.rs` 管（我没复跑这条用例）。
4. 「码 14 比码 15 多一条」的缺口是量到的：把 `recovery.rs:2422` 的 `!=` 改成 `<`（模型目录 `tools/rerun.sh` 的 s3 步），六条用例加我的合法两对探针共 7 条通过，只有 P2 探针（码 14 多一条）失败，panic 在 `crates/singlefs-core/src/recovery.rs:2459`「extent_root: extent_root_entry.expect(」。原样输出（`mut3.log`，经 `tools/summarize.sh`）：
   `test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out` 与 `test probe_five_readers_when_there_is_one_more_snapshot_inode_root_than_extent_roots ... FAILED`。含义：条数相等判的「码 14 > 码 15」那半边没有用例钉住，变异表（`crates/mutations.tsv:4007`「m4 快照格式 S1：不判码 14 与码 15 条数相等」）只置 `false &&`，也只靠「码 15 多」那条红。这半边一旦写歪，读树表的五处会在 `.expect` 上 panic 而不是报树表损坏。
5. 8 行变异（`crates/mutations.tsv:4005–4012`）每行点名的用例：4005、4006 点名同一条（两对快照重建）；4010、4011 点名同一条（第 11 个）；4007、4008、4009、4012 各点名一条。4008 与 4009 的改法文本逐字相同（都是补配给还空着的那条），只有点名的用例不同。点名的用例与行的对应关系我读到了，8 行我没有逐行复跑，「各自只靠一条用例红」不能确认也不能推翻。
6. S1 的 checker 三处没做：`research/prompts/m4-snapshot-r1-main-verification.md:30` 写「I-9.16 ③、I-9.27、I-9.25 与 core 的 `tree_table_entries_each_kind_at_most_once` 各加 14、15 一格」。本批只做了 core 那一处。读代码得出（推的、没在快照镜像上跑过池级 checker）：`crates/singlefs-checker/src/walk.rs:4390`「fn judge_tree_table_entry_kind_counts(」把种类码 14 出现两条判成 I-9.16 ③ 违例（`:4410`「first_kind_carried_more_than_once.is_none(),」）；`:4293`「_ => Self::NotJudged,」使 I-9.27 对码 14、15 不判；`:2393`「previous_snapshot_txg == 0」对非 inode 种类要求为 0，而 S1 要码 14 抄捕获时活头的值（本批 `TreeTableEntry` 还没有这个字段，见 T2）。题面一·2 行只列 core 的文件，所以这不算本批的缺，只是合法的两对快照镜像今天在池级 checker 上会红。

**什么会推翻**：码 14 比码 15 多一条的镜像若在 `mut3` 那一步仍是 8 条全绿；或 `recovery.rs:2422` 的 `<` 变异下 P2 不 panic；或 S1 之外还有一条判决写下的读侧条件被我漏读（我只读了 S1 行与岔路 1 行）。

## T2 读侧兜底与还没加的字段

**判定**：`mounted_read.rs` 那处 match：一致（封闭、无兜底）。五个读树表的读者对合法的快照条目表态不一：三个放行、两个拒，S1 没写它们该怎么办——规则没说。`RebuiltVersionWithItsSnapshotEntries`：今天没有产品消费者。`previous_snapshot_txg` 那 8 字节：读出即丢，core 里没有别的判定读它（池级 checker 的 I-9.25 按字节读，见第 5 点）。

1. `crates/singlefs-core/src/mounted_read.rs:432`「let tree_table_entries = tree_table_entries_each_kind_at_most_once(&tree_table.entries)」之后，`:437–441` 的 match 列全了 `TreeTableEntriesRefusal` 的六个成员（`:441`「| TreeTableEntriesRefusal::SnapshotInodeRootEntryHeadIsNotTheLiveInodeTree => {」），没有 `_ =>`。挂载读路径取树只经 `:445`「let entry_of_kind = |kind: u16| {」按种类挑，码 14、15 从来不被挑中：被当成「树表里多出来的条目」，放行、不读它们的根。
2. 读树表的五个读者（用例文件 `crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs` 的 `TreeTableReaderOutcomes`）在**合法**两对快照条目的树表上的结局，我用探针 P1（`probes/append_to_snapshot_test.rs`）量了：
   - `rebuild_version`：放行（`crates/singlefs-core/src/recovery.rs:3359` 起的 `TransactionOutput` 把全部条目含码 14、15 带进 `tree_table_entries`）。
   - `allocation_records_under_root`：放行（只 `.find` 种类 `:2064`「.find(|entry| entry.kind == TREE_KIND_ALLOCATION)」）。
   - `open_pool_for_read`：放行（第 1 点）。
   - `walk_to_file`：**拒**。`crates/singlefs-core/src/recovery.rs:5504`「let key_width = key_width_for_kind(entry.kind).ok_or(RecoveryFailure::UnitMalformed {」，而 `:4016`「_ => None,」使码 14、15 没有 key 宽，报「树的种类没登记」。
   - `user_visible_tree_root_pointers`：**拒**。`:3473`「_ => {」兜底，`:3475` 同一句「树的种类没登记」。
   探针 P1 原样输出（`probe1.log`）：
PROBE_P1 valid_two_pairs: TreeTableReaderOutcomes { rebuild_version: Ok(()), allocation_records_under_root: Ok(()), walk_to_file: Err(UnitMalformed { what: "树的种类没登记" }), user_visible_tree_root_pointers: Err(UnitMalformed { what: "树的种类没登记" }), open_pool_for_read: Ok(()) }

3. 对照判决：S1（`research/prompts/m4-snapshot-r1-main-verification.md:30`）没写五个读者对合法快照条目各怎么办，只写了哪几处要「各加 14、15 一格」。`crates/singlefs-core/src/recovery.rs:2469`「读树表取条目的每一处都经这一处」是真的（五处都先过它），但「过了」不等于「消费」：合法快照表在 `walk_to_file`（冷启动走读）与 `user_visible_tree_root_pointers`（抬 F 数非空根）上今天被拒。用例文件头 `crates/singlefs-harness/tests/tree_table_entries_carry_the_previous_snapshot_txg_and_admit_snapshot_kinds.rs:4`「四样任一样不成立，读树表的五个读者都判树表损坏」只钉了「坏表五个都拒」，没有一条用例在合法表上跑全五个（第一条用例只跑 `rebuild_version_with_its_snapshot_entries`）。含义：用例名里的 `admit_snapshot_kinds`（放行快照种类）今天只对三个读者为真。这属于下一批（建快照入口）要定的事，不是本批的错；但本批的文档说「读树表的每一处都经这一处」，容易被读成「五处都放行」。
4. `RebuiltVersionWithItsSnapshotEntries`（`crates/singlefs-core/src/recovery.rs:2667`「pub struct RebuiltVersionWithItsSnapshotEntries {」）今天没有产品消费者：对 `RebuiltVersionWithItsSnapshotEntries` 与 `rebuild_version_with_its_snapshot_entries` 两个名字在 `crates` 下 `grep -rn --include=*.rs`，除 `recovery.rs` 自己外只有上面那个用例文件（22、295、310、390 行）命中，零条产品代码。产品路径的 `rebuild_version` 不交快照条目；写者每次发布从头装十二条树表条目（`crates/singlefs-core/src/transaction.rs:8649`「let tree_table_entries = vec![」），不带上一版的码 14、15——建快照那一批必须先改这里，否则可写挂载接着一个带快照的版本发布一次就把快照条目丢了（读代码得出，没跑：今天造不出带快照的可写版本）。
5. `previous_snapshot_txg` 那 8 字节：`crates/singlefs-core/src/records.rs:512`「let _previous_snapshot_txg = reader.get_u64();」读出即丢，下一行 `birth_txg` 接着读，不会被读成别的量；`to_bytes` 在 `:473` 写死 0（「writer.put_u64(0); // previous_snapshot_txg」）。含义：码 14 条目里捕获的活头值今天写不出来（`TreeTableEntry` 没有这个字段，已知清单第 3 批），读回也不保；本批的 `parse` 对这 8 字节任何值都接受。池级 checker 的 I-9.25 才读这 8 字节（`crates/singlefs-checker/src/walk.rs:2393`「previous_snapshot_txg == 0」，非 inode 种类要求为 0），见 T1 第 6 点。
6. flags：`crates/singlefs-core/src/records.rs:504`「if reader.get_u16() != 0 {」整字拒，所以 S4 要的 flags 位 0（销毁中）今天读不进来；`records.rs:436` 的文档已写明「第一版读者还不认它」，一致。

**什么会推翻**：P1 在另一次跑里 `walk_to_file` 或 `user_visible_tree_root_pointers` 放行合法快照表；或 grep 在冻结副本里找到产品代码对 `RebuiltVersionWithItsSnapshotEntries` 的调用。

## T3 logged_ops 条目的格式

**判定**：字节偏移与判决 G1 字段表逐个对得上：一致。三条拒绝与 D26 已定项 2「零格式位」：一致（相容）。一节点 369：一致（算得出）。round-trip 用例钉了绝对偏移，不是只比自己：一致。另有两处不属于本批的缺口：读侧没接线、一个拒绝变体没有用例。

1. 字段表对偏移。判决 `research/prompts/m4-gc-r1-main-verification.md:32`：「key 10 = 种类 2 + 签发 txg 8」，「value 34 定宽（整理 = R 的设备 4 + 起始槽 6 + 槽数 2、c0 8、segs0 8、补零 6），条目 44、一节点 369 条」，「R 的解码拒不对齐的值（起始槽 64 对齐、槽数 = 64）」。写者 `crates/singlefs-core/src/logged_operations.rs:195` 的 `to_bytes`：种类 u16 在 0、签发 txg u64 在 2，`:199` 断言 key 止于 10；设备 u32 在 10；起始槽 6 字节在 14（`:203`「writer.assert_position(14,」）；槽数 u16 在 20；`c0` 在 22；`segs0` 在 30；补零 6 字节在 38（`:214`「writer.assert_position(38,」）；`:218` 断言总长 44。4 + 6 + 2 + 8 + 8 + 6 = 34，加 key 的 10 = 44。格式常量 `crates/singlefs-format/src/lib.rs:190`「pub const LOGGED_OPERATIONS_KEY_BYTES: u64 = 10;」、`:194`「pub const LOGGED_OPERATIONS_VALUE_BYTES: u64 = 34;」、`:197`「pub const LOGGED_OPERATIONS_ENTRY_BYTES: u64 = 44;」、`:201`「pub const LOGGED_OPERATIONS_ENTRIES_PER_NODE: u64 = 369;」，各带 `format-const:` 标记；`:657` 的单测把前三个拆回字段和（2 + 8、4 + 6 + 2 + 8 + 8 + 6、key + value）。
2. 369 算得出：码 2 节点头 = 不含 key 区间的 86（`crates/singlefs-format/src/lib.rs:47`「pub const INDEX_NODE_HEADER_BYTES_WITHOUT_KEY_RANGE: u64 = 86;」）+ 2 × key 宽 10 + nonce / MAC / 算法类型预留 29（`:32`「pub const NONCE_MAC_ALGORITHM_RESERVED_BYTES: u64 = 29;」）= 135；(16384 − 135) ÷ 44 = 369.3，取整 369。同一式子对判决 S3 的 deadlist（key 43、value 空）给 (16384 − 201) ÷ 43 = 376.3 ⇒ 376，与 `research/prompts/m4-snapshot-r1-main-verification.md` S3 行写的「叶 376」一致（算术，没跑）。`:679`「assert_eq!(LOGGED_OPERATIONS_ENTRIES_PER_NODE, 369,」另钉字面值；harness 用例 `crates/singlefs-harness/tests/logged_operations_intent_record_round_trips.rs:109` 拿 core 的 `index_node_entry_capacity(10, 44)` 现算再比 369。
3. 三条拒绝（宽 ≠ 44、种类码 0 保留、`of_one_cluster_segment` 拒不对齐）对 D26 已定项 2：`.claude/kb/decisions/26-后台整理与放置回收.md:51` 写「零格式位——不新增任何盘上字段，`c0` 住意图记录」，射程是「这个谓词」（目标态判据）不新增盘上字段。三条拒绝都是读侧对「树里一条普通 key 的 value」的解码政策（`.claude/kb/decisions/08-核心索引结构.md:112`「意图本身就是一条普通 key」），不新增字段、不改新池新建文件的字节，所以相容。`segs0` 是 G1 加进条目的第二个量（判决 G1 写明「只作观测量，停机不靠它」），D26 那一句没有禁止同一条意图记录里有观测量；这是我对 D26 句子射程的读法（推的）。
4. 拒绝落在代码里：宽 `crates/singlefs-core/src/logged_operations.rs:228`「if bytes.len() != entry_bytes() {」；种类 `:48`「_reserved_kind_code => None,」（`of_code` 只认 1，0 与 2 都拒）；槽数 `:260`「if u64::from(slot_count) != CLUSTER_SEGMENT_SLOTS {」；补零 `:267`「if padding.iter().any(|byte| *byte != 0) {」；R 不对齐 `:79`「if !start_slot.0.is_multiple_of(CLUSTER_SEGMENT_SLOTS) {」。
5. round-trip 用例钉绝对偏移：`crates/singlefs-harness/tests/logged_operations_intent_record_round_trips.rs:4`「期望的字节按判决 G1 的字段表手写，不从被测的写者反推」，`:42`「vec![0x01, 0x00],」起把八段期望字节逐段手写成小端字节，`:53` 与 `to_bytes()` 整体比；另有字节序、拒收五种各一条。不是 `.claude/singlefs-ai-sop/rules/test-discipline.md`「只让多条臂互相比，测不出「所有臂一起错」」那种自比。
6. 本批没有的：（a）没有读侧接线——`crates/singlefs-core/src/recovery.rs:4015`「| TREE_KIND_LOGGED_OPERATIONS => None,」仍使非空的 logged_ops 根在 `walk_to_file` 报「树的种类没登记」，`:4011` 的注释「logged_ops 的字段表留到第一个写者落地时定」已过时；`logged_operations.rs` 没有产品调用者（除 harness 用例外 `grep -rl` 只命中 `region_scan.rs`、`compaction_plan.rs` 两个同批新模块）。属下一批。（b）`CompactionRegionRefusal::StartBeyondTheSixByteSlotNumber` 没有用例（从 `parse` 走不到，6 字节读出来恒小于 2^48，只能从 `of_one_cluster_segment` 传大槽号进来）。

**什么会推翻**：用 `ByteWriter` 之外的手写字节对 `to_bytes` 的 44 字节逐位比出一处偏移不同；或节点头式子在码 2 节点实建出来的容量不是 369（本批没有实建节点的用例，只有式子，所以这一条今天证不了「实建」）。

## T4 区域扫描的四类判定

**判定**：类的集合与 G9 一致（五类 = G9 四类加隔离槽单列）。判类优先级：规则没说（G9 没定次序，实现自定），且次序的每一次交叉没有用例，我量到两个存活变异。「头读不出、无条目指着」判不一致不判已死：与 G9 字面重叠，实现取不一致优先——规则有重叠，不算冲突也不算一致。跨 R 边界：拒整个扫描，不是拒一条。

1. G9 原文（`research/prompts/m4-gc-r1-main-verification.md:40`）：「分四类：可搬、钉住（豁免三类或被根直接指着）、已死（有记录、映射里没人指向它——第一版只报告不动，N2 会红）、不一致（头读不出或两来源对不上）」，「攻方的「搬不动」类 = 钉住 ∪ 隔离槽（隔离槽单列，让 K-c 丢意图）」。代码 `crates/singlefs-core/src/region_scan.rs:101`「pub enum AllocationInRegion {」五个成员：可搬、钉住、隔离、已死、不一致。
2. 优先级（`region_scan.rs:226` 的 `classify_allocation`）：钉住 `:237`「if pinned_slots.range(covered_slots.clone()).next().is_some() {」→ 隔离 `:244`「if entries_pointing_here.is_empty() && quarantined_slots.range(covered_slots).next().is_some() {」→ 读头 `:247`「let header = match read_unit_header(record, read_unit_bytes) {」→ 类标签 `:260` → key 与位置 `:269` 起。G9 没写次序，所以这个次序是实现自定（已知清单第 23 行已记「判类优先级钉住 → 隔离 → 读头 → 类标签 → key 与位置」）；它没有与 G9 任一条字面冲突。
3. 隔离槽仍有映射条目指着时按普通单元往下判：D19 写的是隔离发生时「逻辑上照样释放（映射条目去掉）」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:107`「核出对不上时隔离那个槽，发布照成」），所以正常写出来的镜像里隔离槽没有条目指着，`:244` 的 `entries_pointing_here.is_empty()` 在正常路径上恒真。条目仍指着隔离槽只会出现在坏镜像里；那时往下走读头，头校验和是单元自带的（`parse_*` 判），**没有**拿映射位置项里的整单元校验和去比（`region_scan.rs` 的 `CentralMappingEntry` 带 `LocationEntry`，分类里没读 `unit_checksum`；用例的 `locations_at` 把它写成 0，`:464` 一带）。所以一个「位置项校验和对不上、单元自带校验和过得去」的单元会判成可搬——今天无路可走到（没有产品调用者），G3 搬迁前要在批里再核一次。规则没说。
4. 优先级交叉没有用例：我做了两个变异放进模型目录 `tools/rerun.sh` 的 s4、s5 步——（甲）`:237` 改成「钉住且不在隔离集里才判钉住」（钉住 × 隔离同槽时隔离赢）；（乙）`:244` 去掉 `entries_pointing_here.is_empty() &&`（隔离槽有条目指着也判隔离）。两个变异同时打上，核心 lib 的 `region_scan::` 5 条用例与 harness 的 `compaction_plan_chooses_regions_and_batches_by_the_clauses` 7 条用例全绿，原样结果行（`mut4.log`、`mut5.log`，经 `tools/summarize.sh`）：
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 254 filtered out; finished in 0.04s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

   含义：`mutations.tsv` 里 region_scan 的行（`crates/mutations.tsv:4017–4019`、`:4030`、`:4031`）管的是「不核映射位置」「不核 key」「不核类标签」「不看钉住」「放过跨边界」，没有一行管隔离那一支或次序；这两个变异都是存活的。优先级的每一次交叉（钉住 × 隔离、隔离 × 有条目指着、钉住 × 头读不出）都没有一个槽同时落在两类里的用例。
5. 头读不出、无条目指着判「不一致」，G9 字面（同一行 `:40`）：已死 =「有记录、映射里没人指向它」，不一致 =「头读不出或两来源对不上」。两句在「无条目指着且头读不出」「无条目指着且头里的 key 映射在别处」两种形上都成立。实现取不一致优先：已死只留给无条目指着、头读得出、头里的 key 映射里也没有、不在隔离集里的那一支（`crates/singlefs-core/src/region_scan.rs:269`「if entries_pointing_here.is_empty() && !header_key_is_mapped {」），其余落 `:283`「RegionScanInconsistency::HeaderKeyDisagreesWithTheMappingLocation,」或读头失败的 `:247` 一支。用例 `a_unit_whose_mapping_entry_points_at_another_slot_is_inconsistent_not_dead`（`region_scan.rs:660`）把这个取法钉成名字。与 G9 字面不冲突（两个定义本来重叠，没有给次序），但它决定了「已死」这一类（第一版只报告）比 G9 字面窄，下游若按 G9 字面把「映射里没人指向它」都当已死来处置，会与实现不一致。规则没说。
6. 只看 R 那块盘、只比槽号（K-n）在两盘不对称时：
   - 记录只在另一块盘上、R 那块盘上没有对应记录：`crates/singlefs-core/src/region_scan.rs:182`「.filter(|record| record.device == device && !record.is_released)」把它过滤掉，那一个落点在 R 的扫描里不存在（既不分类也不报），`choose_region` 的活槽数同样只数 R 那块盘（`crates/singlefs-core/src/compaction_plan.rs:60`「.filter(|record| record.device == unit_area.device && !record.is_released)」）。漏掉的是「只在对盘上有记录的单元」，没有任何一类能装它。
   - 钉住与隔离集按槽号登记、不分盘：对盘上某槽被钉住会把 R 那块盘上同号槽的单元判成钉住（过保守，不会搬），反方向不会漏。
   - 映射条目的两个位置指向不同槽号（不对称）时，`points_at(设备, 槽)` 按设备精确比，R 在哪块盘就用哪块盘那一份；而 `PlannedRelocation.source` 只有 `Placement`（槽 + 跨度，没有盘）。所以计划表达不了「两盘不同槽」。D2 已定项 10 定第一版两盘同槽，这是该前提的后果，不是错。规则没说。
7. 跨 R 边界的记录：拒整个扫描，不是拒那一条——`crates/singlefs-core/src/region_scan.rs:190`「return Err(」在分类之前，用例 `a_record_straddling_the_region_boundary_is_refused_before_any_classification` 钉住。后果超出已知清单写的范围：`choose_region` 把跨边界记录的槽按槽数进两个段（`crates/singlefs-core/src/compaction_plan.rs:63`「for slot in record.slot.0..record.slot.0 + u64::from(record.span_slots) {」），那个段仍有资格，下一次 `choose_region` 照样选它，扫描照样整个拒——选段那一层没有「这个 R 上次被拒」的信息，一块坏记录能让同一个 R 反复被选中又反复被拒（读代码得出；今天没有调用者，没跑）。

**什么会推翻**：甲、乙任一变异下 harness 或核心的 12 条用例有红的；或 `region_scan.rs:244` 的隔离分支在正常写出的镜像上能碰到「有条目指着」的槽（D19 `:107` 说隔离时条目已去掉）；或另一块盘上的孤记录在 `scan_region` 里有一类能装。

## T5 选段与计划

**判定**：三个输入、三条排除、greedy 与同分取最小槽号、`plan_batch` 只交（映射 key, 源落点）：与 G2 一致。资格「至少一个活槽」与 G2「至少一个能搬的单元」不同：已知清单已记；我补的是它的后果方向。批内同一个 key 两份副本：经 `scan_region` 进不来。

1. G2 原文（`research/prompts/m4-gc-r1-main-verification.md:33`）：「`choose_region` 的输入补三样：钉住的槽（根记录指着的树表、映射根、实例表链与映射树各节点）、本次挂载开过的段（`cluster_segments`）、活意图的 R；排开装着豁免单元的段与开放段，要求 R 里至少一个能搬的单元」，「`plan_batch` 只交（映射 key, 源落点），目的落点由那一批发布时的分配器按 D3 已定项 8 现取」，「选段判据第一版 greedy（活槽最少者先）」。
2. 三条排除各在一行（`crates/singlefs-core/src/compaction_plan.rs`）：开放段 `:85`「let opened_in_this_mount = cluster_segments.contains(start_slot);」；活意图的 R `:86–87`「live_intent_region.is_some_and(|region| region.start_slot() == *start_slot);」；装着钉住槽的段 `:88`「let holds_a_pinned_slot = pinned_slots.range(*start_slot..end_slot).next().is_some();」。资格在 `:89–92` 一个布尔式里合取，最后一项 `:92`「&& *live_slots > 0」。三条各有一条用例与一行变异（`crates/mutations.tsv:4020`、`:4021`、`:4022`、`:4024`）。`choose_region` 的签名（`:45`）确是三个补充输入加判据，函数体里没有设备读，纯。
3. greedy 的分：`:36`「pub const fn greedy_score(live_slots_in_segment: u64) -> u64 {」，活槽数 = R 那块盘上未释放记录罩住的槽数，按记录数（`:60–63` 逐记录逐槽累加），**不按 `scan_region` 的分类**；同分取起点槽号最小：`:96`「.min_by_key(|(start_slot, live_slots)| (greedy_score(*live_slots), *start_slot))」，用例 `a_tie_goes_to_the_segment_with_the_lowest_start_slot`、变异 `crates/mutations.tsv:4032`。
4. 空批与排他：`choose_region` 不看分类，所以一个活槽全是已死 / 不一致 / 隔离单元的段照样有资格（已知清单第 24 行）。我补的后果方向：greedy 是「活槽最少者先」，一个只剩一条已死记录的段分数是 2（一个两槽单元），低于多数含真活数据的段，所以它常常先被选中；选段不带「上次搬不动」的记忆，下一次仍然是它。`plan_batch`（`:133`「pub fn plan_batch(region_scan: &RegionScan, batch: CompactionBatchUnits) -> Vec<PlannedRelocation> {」）在这个段上交空表，返回类型分不出「搬空了」与「搬不动」，调用方只能回头看 `RegionScan.allocations`。「R 还排他着」在本批无从发生（没有任何代码签发意图或置排他），那一半是下一批的事。读代码得出，没跑；今天没有调用者。
5. 一批大小与同 key 两份副本：`CompactionBatchUnits::new(0)` 交 `None`（`:113–114`「NonZeroUsize::new(units).map(Self)」），一批至少 1；`plan_batch` 取可搬单元按槽升序的前 K 个（`:136`「.take(batch.units())」），K 差一各有变异（`crates/mutations.tsv:4025`、`:4033`）。同一个 key 的两份副本：R 那块盘上只有一份会被映射条目指着，另一份无条目指着、头里的 key 映射在别处，`scan_region` 判不一致（`region_scan.rs:283`），不会进可搬；两盘同槽的两份副本只扫 R 那块盘（T4 第 6 点），所以一个 key 在计划里至多一项。`plan_batch` 自己不去重，但它的输入 `RegionScan` 的字段是公开的，手造一份重复 key 的扫描结果它会照交。
6. 目的落点不在计划里：`crates/singlefs-core/src/compaction_plan.rs:125`「pub struct PlannedRelocation {」只有 `mapping_key` 与 `source` 两个字段（`:126–127`），与 G2 一致。

**什么会推翻**：`choose_region` 在 harness 用例之外给出了一个全是已死单元的段而 `plan_batch` 没交空表；或 `PlannedRelocation` 多出目的落点字段。

## T6 discard 口子的 trait 与两个后端

**判定**：默认实现对越界与没对齐交错误、与写零一路同一个判据：一致。文件后端「不支持」不退回写 0、别的错照交、信号打断重打：一致于 G5 与代码文档。长度 0 在四个后端上结局不同：规则没说。打断重打无上限：规则没说（与写零同口径）。5 行没证红的变异：harness 的 2 行我证了红；checker 档 3 行复核不了。

1. G5 原文（`research/prompts/m4-gc-r1-main-verification.md:36`）：「块设备 trait 加一个有默认实现（返回「不支持」并计数）的 `discard_at`，录制设备、故障注入设备、内存盘必须覆盖；文件后端复用打洞（`punch_hole_keeping_the_file_length`）；真盘 ioctl 留到真盘线」。代码：trait 默认 `crates/singlefs-core/src/block_device.rs:193`「fn discard_at(」，`:204`「Ok(DiscardOutcome::Unsupported)」；录制设备 `crates/singlefs-harness/src/lib.rs:454`、故障注入 `crates/singlefs-harness/src/fault_injection.rs:1133`、内存盘 `crates/singlefs-harness/src/memory_pool.rs:210` 各自覆盖；文件后端 `block_device.rs:649`；`O_DIRECT` 后端 `:940`。G5 写「返回「不支持」并计数」，而默认实现自己没有状态、不计数（文档 `:189–190` 写明由调用方按返回值数或后端自己数），计数放在录制器与两个产品后端：与 G5 字面有一点出入，实现员已在文档里写明，不影响行为。
2. 默认实现：核对齐与越界再交 `Unsupported`。越界与没对齐交 **错误**（`BlockDeviceError::Unaligned` / `OutOfRange`），不交 `Unsupported`——它调的是 `block_device.rs:455` 的 `check_aligned_and_in_range`，`write_zeroes_at` 的两个产品后端（文件 `:449` 的 `check_request`、`O_DIRECT` `:926`）调同一个函数，判据一致。用例 `a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step` 里没对齐的请求拿到 `Unaligned`、不数。
3. 文件后端（`block_device.rs:649` 起）：先 `:654`「self.check_request(discarded_offset, discarded_length)?;」；`PunchingUnsupported` 交 `Unsupported` 并计数，`:693–696`「HolePunchFailure::PunchingUnsupported => {」…「return Ok(DiscardOutcome::Unsupported);」，不写 0（写零那一路在 `:620` 的同名分支里会退回逐块写 0——两条路的差别就在这一支）；别的错 `:702`「HolePunchFailure::OtherFailure => {」照交 `InputOutput`；被打断 `:698`「HolePunchFailure::InterruptedBySignal => {」原样重打，**没有次数上限**，口径与写零相同（`:660–661` 的注释写明「轮数的上界是这期间送到的信号数」）。G5 与 D26 没写上限，规则没说。
4. 长度 0：文件后端 `:656`「if discarded_length == 0 {」记一次 `Applied` 并交回；内存稀疏盘 `memory_pool.rs:210` 起的实现也交 `Applied`（零长清零）；`O_DIRECT` 后端 `:940–948` 与默认实现对长度 0 交 `Unsupported`（`O_DIRECT` 还计一次「不支持」）。同一个请求四个后端两种结局；录制器对 `Applied` 的长度 0 会记一步长 0 的 Discard（`crates/singlefs-harness/src/lib.rs:462`）。G5 说「按 64 槽对齐的整段发」，产品路径不会发长度 0，所以今天无害；规则没说。
5. `DiscardLanding` 的测试开关：`crates/singlefs-core/src/block_device.rs:309`「const INITIAL_DISCARD_LANDING: DiscardLanding = DiscardLanding::PunchHoleOrAnswerUnsupported;」，两个构造入口（`:402`、`:438`）都用它，只有 `:340` 的 `with_test_only_discard_landing` 能换；平常路径不认三个 `ActAsIf…` 开关。一致于 `.claude/rules/fs-design.md` 的「只供测试的开关」写法。
6. `O_DIRECT`：`:940–948` 核过请求就交 `Unsupported` 并计数，不发系统调用；与 G5「真盘 ioctl 留到真盘线」一致。
7. 19 行变异里没证红的 5 行（行在 `crates/mutations.tsv:4035–4053`）：
   - harness 的 2 行，我在副本上证了红（`tools/rerun.sh` s2 步）：`crates/mutations.tsv:4037`「m4 discard 口子：录制器不数 discard 的结局」→ `block_devices_record_and_apply_discards.rs:217`（不支持那次没数进）与 `:129`（交给了底层那次没数进）两条断言红；`:4040`「m4 discard 口子：切段把 discard 归进写单元」→ `:273`（种类串 `[zero_fill,unit_write,barrier]` 变成 `[unit_write×2,barrier]`）红。原样结果行：
test a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step ... FAILED
test the_recorder_records_an_applied_discard_as_one_step_without_contents ... FAILED
test segments_count_a_discard_as_one_plain_write_with_two_crash_states ... FAILED
thread 'a_backend_keeping_the_default_answers_unsupported_and_the_recorder_counts_it_without_a_step' panicked at crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs:217:5:
thread 'the_recorder_records_an_applied_discard_as_one_step_without_contents' panicked at crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs:129:5:
thread 'segments_count_a_discard_as_one_plain_write_with_two_crash_states' panicked at crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs:273:5:
test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
   （上面几行是 `mut2.log` 里匹配 FAILED、test result、panicked 的行，只去掉了线程号和耗时。两行变异一起打上，但两行各红在不同断言：4037 红 `:129`、`:217`，4040 红 `:273`，可分。）
   - checker 档的 3 行（`crates/mutations.tsv:4051`、`:4052`、`:4053`）：复核不了（要跑 `singlefs-checker-tier`）。读代码得出各靠谁红：4051 点名的 `device_log::tests::a_discard_in_the_recorded_stream_must_reach_the_device_log_as_the_same_range_in_the_same_order`（`crates/singlefs-checker-tier/src/device_log.rs:898`）断言期望侧投出一条 Discard，期望侧改回 `continue`（`:196–204` 那段）就红；4052 靠 `new_pool_file_creation_device_log_check` 这个 bin 内联的 `a_discard_the_program_recorded_but_the_device_never_received_is_red`；4053 靠同一个 bin 的 `the_new_pool_file_creation_rerun_issues_no_discard_on_either_device`（该用例断言 `expected_discards`、`observed_discards` 都是 0，见 `crates/singlefs-checker-tier/src/bin/new_pool_file_creation_device_log_check.rs:1024–1025`；把条数数成 FLUSH 条数后，这条流里 FUA 写带出的 FLUSH 不为 0，断言红。读代码得出，没跑。）
8. 包装设备里没覆盖 `discard_at` 的（已知清单已记，这里只列实际的一份）：`crates/singlefs-core/src/mounted_session.rs:427` 的 `DeviceKeepingWritesInMemory`、`crates/singlefs-harness/src/read_tally.rs:194` 的 `ReadLoggingBlockDevice`；`grep -rln "fn discard_at" crates` 只命中 `block_device.rs`、harness 的 `lib.rs`、`memory_pool.rs`、`fault_injection.rs` 四个文件。

**什么会推翻**：默认实现对没对齐的请求交了 `Unsupported` 而不是错误；或文件后端在 `PunchingUnsupported` 之后动了盘上字节；或 `mut2` 那一步里 4037、4040 对应的断言在另一次跑里不红。

## T7 录制、两态与设备日志（只核 G5 的两态与「不支持一字不动」）

**判定**：录制器只记 `Applied`、`Unsupported` 只计数：一致于 G5 与 trait 文档。两态在 harness 这一层（写表、内存盘、切段）是 G5 说的两态：一致。**但**崩溃枚举器（checker 档 `crash.rs`）的撕裂判据对「长于一个扇区、基镜像里有非零字节」的 discard 会给第三态：与 G5、与 `memory_pool.rs` 的注释冲突（读代码得出，复核不了）。「不支持一字不动」这个前提：由 trait 合同与各后端自己保证，没有哪一层替它设防；唯一用真系统调用保证的那一支没有被任何用例走到。

1. G5 原文（`research/prompts/m4-gc-r1-main-verification.md:36`）：「录制流加 discard 一步、崩溃点重放当「范围写零」枚举两态、设备日志判器判每个 Discard 落在可发区间里且在那次发布最后一道屏障之后」。D13 已定项 4（`.claude/kb/decisions/13-验证路线.md:134`）：「一次写取两态（持久 / 没持久）」，「「没持久」的位置上，镜像里放的是崩溃前那个位置的旧字节，不是零」。
2. harness 层的两态，一致。写表里 discard 进 `RetainedWrite`，内容是 `WrittenContents::Zeros { length }`（`crates/singlefs-harness/src/memory_pool.rs:613`「RecordedOperationKind::Discard => {」起）；切段把它归 `StepKind::ZeroFill`（`crates/singlefs-harness/src/segments.rs:68`「RecordedOperationKind::Discard => StepKind::ZeroFill,」）；用例 `segments_count_a_discard_as_one_plain_write_with_two_crash_states`（`crates/singlefs-harness/tests/block_devices_record_and_apply_discards.rs:273` 起）两个态各读一次：持久读回全 0，没持久读回旧字节（0xC3），状态数 `1 + (2^2 − 1)`。
3. 第三态（冲突）。两边整行并排：
   - G5：`research/prompts/m4-gc-r1-main-verification.md:36`「崩溃点重放当「范围写零」枚举两态」。
   - harness 注释 `crates/singlefs-harness/src/memory_pool.rs:612`：「// 只取这两态，不是原地覆写（种类按动作归进整段清零，`crash` 的撕裂判据不给它第三态）。」
   - 实际判据 `crates/singlefs-checker-tier/src/crash.rs:2156–2162`：`StepKind::ZeroFill` 与 `JournalRecord` 等一起落在「`is_copy_on_write = false`」那一支；`:2162`「if is_copy_on_write || write.length_in_bytes() <= SECTOR_BYTES {」只对「单元写」与「不长于一个扇区」返回 false；之后 `:2166`「let base_holds_nonzero_bytes_there = base.devices.get(&write.device).is_some_and(|device| {」看基镜像那一段有没有非零字节。一次 16 KiB 的 discard（一个单元）或 1 MiB 的整段 discard（G5 说的 64 槽对齐整段）罩在用过的槽上，基镜像或更早的带字节写里那一段是非零的 ⇒ 判成原地覆写 ⇒ 多出第三态「新旧都读不出」。
   - D13 `.claude/kb/decisions/13-验证路线.md:134`：「原地覆写（不是单元写、长于一个扇区、罩住的范围里原来有东西的写」…「多取第三态「新旧都读不出」」。字面上一次罩在旧内容上的长 discard 恰好是这一类。另一面：`crates/singlefs-checker-tier/src/crash.rs:2065`「那一槽的旧内容没有谁还要（回收谓词管着）」是给单元写免第三态的理由，被 discard 的槽按 G5 也是已回收、没人要的旧内容，这条理由同样适用——所以该判两态还是三态是要定的事，两边的字都在。
   本批的用例用 512 字节的 discard（`block_devices_record_and_apply_discards.rs` 里 `discard_at(…, 512)`），不长于一个扇区，所以看不到这件事。我没有在这一批里跑它：它在 `singlefs-checker-tier`，不许跑。模型目录 `probes/checker_tier_probe_forward_tearable.rs` 是一个只编不跑、也**没有编过**的探针（checker 档包要现编 RocksDB，副本里编不过 `stdbool.h` 那一关，我没有为它去补环境）：16 KiB discard 罩在 0xC3 旧字节上时 `TearableInPlaceOverwrites::of(...).contains(0)` 应当为真，512 字节的对照应当为假。产品路径这一批不发 discard，所以今天没有一个状态受影响；下一批发 discard 的那一刻，要么 `crash.rs` 把 discard 排除出撕裂判据，要么 G5 改口为三态，判决里要定一个。
4. 「设备不保证读回 0」：trait 文档 `crates/singlefs-core/src/block_device.rs:185`「交回 [`DiscardOutcome::Applied`] 之后，那一段在持久之前读回旧字节或 0 都可能，持久之后读回 0 还是旧字节由设备定」。这句话承认持久之后读回旧字节也可能；模型的镜像集合 {全 0, 旧字节} 已包含这两种读回，所以集合没有第三种内容——除非设备读回既不是 0 也不是旧字节（例如有的盘对 deallocate 后的读返回不确定数据），那是模型之外。G5 没写这一档。规则没说。
5. 「不支持就一个字节没动」的保证层：
   - trait 合同 `crates/singlefs-core/src/block_device.rs:189`「**默认实现只给不认 discard 的后端**：核过请求就交回 [`DiscardOutcome::Unsupported`]，一个字节不动。」——对实现者的约定，类型系统不设防。
   - 默认实现 `:193–204`：只核请求、不写。`O_DIRECT` 后端 `:940–948`：不发系统调用。文件后端 `:693–696`：`fallocate` 报 `EOPNOTSUPP` / `ENOSYS` 才交 `Unsupported`，内核对这两个错误是动手之前就返回的，这是内核语义；代码里没有任何东西核它。
   - 用例对这一支用的是测试开关 `DiscardLanding::ActAsIfPunchingWereUnsupported`（`:1890` 起的 `a_discard_where_hole_punching_is_unsupported_answers_unsupported_and_leaves_the_old_bytes`），它**不调 `fallocate`**，直接当作报了 `EOPNOTSUPP`；真调用报不支持的那条路（比如镜像文件放在不支持打洞的文件系统上）没有用例，也走不到（本机文件系统都支持）。
   - 录制器 `crates/singlefs-harness/src/lib.rs:472`「DiscardOutcome::Unsupported => {}」：不支持的 discard 不进流，崩溃枚举对它什么都不模型；所以前提一旦被某个后端违反（返回 `Unsupported` 却动了字节），重放不会发现。没有一层把「Unsupported 之后读回的字节」跟之前比。含义：这个前提的保证层是各后端自己，加一条覆盖它的用例的话，只能在真后端上做。
6. 设备日志判器（`device_log.rs`）本批只做了期望侧投出 Discard（`crates/singlefs-checker-tier/src/device_log.rs:196–204`），G5 说的另两半（每个 Discard 落在可发区间里、在那次发布最后一道屏障之后）没有代码——今天没有 discard 发出者，它们没有对象；属下一批。

**什么会推翻第 3 点**：探针里 16 KiB discard 的 `contains(0)` 为假；或 `crash.rs:2148` 以后另有一处在调用 `is_tearable_in_place_overwrite` 之前把 `ZeroFill` 剔出去。**什么会推翻第 5 点**：有一个用例在真 `fallocate` 报 `EOPNOTSUPP` 的文件系统上跑过。

## T8 扫描重建与 I-1.13 的不适用口径

**判定**：必红世界 ① ③：一致。必红世界 ②：题面字面是「一个码 1 单元头」，实现要两份副本都抹才红（一份只降级）——冲突，已被代码文档里的「读法」条收口，见第 2 点。常量表之下 ① ② 仍红：一致（量过）。常量表这第四类不适用理由不在题面列的清单里：规则没说（主 agent 口径）。I-1.13 的违例不是 I-3.1 / I-3.11 的连带：一致（量过）。输入禁区没被绕过：一致。

1. 题面（`.claude/kb/milestone/04-fourth-txn.md:107`）：「判定 Holds / Violated（列出第一处差异：表名、key、两边的值）/ NotApplicable（最新根读不出、实例表读不全）」，射程（`:111`）「回退过的镜像上这一条报 NotApplicable 并写理由」。代码：`crates/singlefs-checker/src/walk.rs:12539` 的 `judge_scan_rebuild_agrees_with_the_walked_index` 判一次，接在 `check_pool_image` 全部判定之后、同一份 `judgements`（调用在 `:13715`）；实例表读不全 `:12548` 一支报不适用；违例说明写「表名差 N 处，第一处 key、两边的值」。
2. 三个必红世界。题面 `:109`：「把一个码 1 单元头的对象 key 清零」，后一句「那条 key 两边数量不同，比对红」。我跑了探针 P3（`probes/append_to_scan_rebuild_test.rs`），列出每个世界的全部红项（`probe1.log` 原样）：
PROBE_P3 world1: ["I-2.1=Violated", "I-3.1=Violated", "I-3.11=Violated", "I-4.8=Violated", "I-7.4=Violated", "I-1.13=Violated"]
PROBE_P3 world2_both: ["I-2.1=Violated", "I-4.8=Violated", "I-7.2=Violated", "I-7.4=Violated", "I-1.13=Violated"]
PROBE_P3 world2_one: ["I-2.1=Violated", "I-4.8=Violated", "I-7.4=Violated"]
PROBE_P3 unlink_no_other_inodes: ["I-1.13=NotApplicable(扫描重建：退役树里有记录，叶容器退役过；退役记录的读侧没有条款（C768），第一版不接)"]

   读法：
   - ①（`world1`）I-1.13 红，且在三张表里只差「中央映射」一处（用例 `crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs:174` 起断言「「中央映射」差 1 处」与两边落点都写进说明）。与题面一致。另红的是 I-2.1、I-3.1、I-3.11、I-4.8、I-7.4。
   - ②（`world2_both`）两份副本都抹：I-1.13 红在「每个 inode 的数据单元集合」（同一个用例文件 `:200` 起）。**只抹一份**（`world2_one`）：I-1.13 不红（`:255` 起的 `zeroing_the_object_key_of_one_copy_leaves_the_scan_rebuild_holding` 钉住「成立」），另红 I-2.1、I-4.8、I-7.4。所以题面「把一个码 1 单元头的对象 key 清零」在两盘池上字面照做（抹一份副本）并**不**让 I-1.13 红；实现的读法写在 `crates/singlefs-checker/src/rebuild.rs:17`「读法：中央映射同一 key 两边的落点集合不同时，两边各只留读得出的副本再比，掉一份副本的降级镜像不判红（主 agent 2026-10-13 认，条款没写）。」两边整行并排：题面 `.claude/kb/milestone/04-fourth-txn.md:109` 说清零一个单元头的 key 就「那条 key 两边数量不同，比对红」；实现 `world2_one` 一份时 I-1.13 成立。冲突在字面，读法已是主 agent 认的；若要让「抹一份」也红，要改读法而不是改用例。
   - ③ 白名单单测：`crates/singlefs-checker/src/rebuild.rs:638`「fn the_use_list_of_the_scan_rebuild_is_exactly_the_format_parsing_whitelist() {」比 `use` 清单与白名单逐项（双向：多的、少的都红），`:653`「fn the_scan_rebuild_names_no_crate_item_outside_the_whitelist_by_a_path() {」另核非测试代码里每个 `crate::` 路径与 `super::`。往重建模块里加一行 `walk_*` 调用，要么要一条 `use`（第一条红），要么要写 `crate::walk::…`（第二条红）；函数体里的 `use` 不在语句开头，第一条看不见，但第二条会抓到它的 `crate::` 路径。我没有真往里加一行去看它红（读代码得出）。
3. 常量表 `INVARIANTS_WHOSE_VIOLATION_MEANS_THE_WALKED_INDEX_IS_DAMAGED`（`walk.rs:12515`，四项 I-1.11、I-1.12、I-4.2、I-5.1）：
   - 世界 ① ②（含一份）的红项里一个都没有这四条（见 P3 输出），所以表不触发，两个世界仍红：一致。
   - 按整份 `judgements` 查：`crates/singlefs-checker/src/walk.rs:12559`「.find(|(invariant, _)| judgements.violation_count(invariant) > 0)」，`violation_count`（`crates/singlefs-checker/src/image.rs:382`）是整个镜像上这条不变量判过的违例次数，包括走读别的候选根（older candidate root）时判的。含义：这四条里任一条在一个**更旧的候选根**上红，最新根的 I-1.13 也被置成不适用，而最新根的索引本身可能完好。受影响的是本来就已经有这四条之一在红的镜像，合法镜像上这四条不红，所以合法镜像不会因此被判不适用。代价方向是：同一镜像上两个独立的损坏（一个让这四条之一红，一个让扫描重建对不上）只报前者。
   - 这第四类不适用理由不在题面列的清单里（题面 `:107` 只列最新根读不出、实例表读不全，射程 `:111` 另加回退过）；代码里写明是「主 agent 2026-10-13 定的口径」（`crates/singlefs-checker/src/walk.rs:12509`「I-1.13 比对之前查的那几条（主 agent 2026-10-13 定的口径）」）。规则没说。
4. 「映射条目改指一份没分配的副本」那个镜像（世界 ①）上的违例是真对不上，不是连带：`judge_scan_rebuild…` 只经常量表那四条取 `judgements`（`walk.rs:12557–12561`），I-3.1、I-3.11 不在表里；扫描重建那一侧只读单元头与码 3 容器的记录，不读映射树（`rebuild.rs` 不调 `walk`，见第 2 点 ③）；用例断言的说明里两边落点来自扫描到的原槽和遍历方向映射里的新槽。
5. `ScanRebuildNotDecided` 的 11 个成员（`crates/singlefs-checker/src/rebuild.rs:109` 起）哪几格真实写路径走得到：
   - 量到的：`RetiredLeafContainersWithoutTheReadSideOfC768`（`rebuild.rs:123`）——把唯一的文件 unlink 到底，叶容器退役，P3 的 `unlink_no_other_inodes` 报 I-1.13 不适用并写「扫描重建：退役树里有记录，叶容器退役过」。用例把这个场景躲开（先多建两个空 inode，`scan_rebuild_agrees_with_the_walked_index.rs:163` 起）。这一格不在已知清单（已知清单只记删除中与覆盖写变短）；题面 `:111` 写了「退役记录读侧（C768）第一版不接」但没写这类镜像报不适用。含义：四·2 验收的随机历史里只要有 inode 被删空，那一版镜像上 I-1.13 报不适用。
   - 用例有的：删除中、文件变短（已知清单）。回退（`RolledBack…`）：有单元用例（`rebuild.rs:1095`），我没跑真回退的镜像。
   - 其余七格（码 1 写序相同、码 3 全序键相同、同版副本载荷校验和不同、当前版本容器无副本解得开、墓碑区间不是整对象、两对象同一数据单元位置、同一 inode 号在两片容器）：我没有找到真实写路径能走到的证据；看起来要么要同一事务号把同一个对象写两次（前两格）、要么只有损坏镜像才有（第三、四格）、要么要区间墓碑或 inode 号复用这类第一版不写的东西（后三格）（推的，没量）。

**什么会推翻**：把唯一的文件 unlink 到底之后 I-1.13 仍然成立（P3 的 `unlink_no_other_inodes` 行不是不适用）；或世界 ① ② 的红项里出现 I-1.11、I-1.12、I-4.2、I-5.1 之一。

## T9 GPU 覆盖表进、内核不置位

**判定**：覆盖表收下标 92、文档写明内核不置位：与用户 2026-10-13 的定案字面一致。编译期断言只证「表是完整的恒等列表」，不证内核判了什么。**先例 I-4.4 抄错了一格**：I-4.4 在 CPU 一侧恒报不适用，对拍字上两边天然同为 0，所以「不比」不需要任何屏蔽；I-1.13 在 CPU 一侧真判（成立 / 违例），GPU 恒 0，对拍字在每个 CPU 判得了 I-1.13 的状态上都不等——而「不比」那一半没有接（已知清单第 3 批）。GPU 对拍我不能跑，红在哪一条断言是读代码得出的，复核不了。

1. 用户定案：`research/prompts/m4-gc-r1-main-verification.md:69`「先进覆盖表但内核不置位，记欠账」（用户原话；同一行的落法写「照 I-4.4（下标 73）的先例」）。代码：`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:154`「pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[」，末行 `:158` 到 92；文档 `:151–152` 写「下标 92 的 I-1.13（扫描重建与遍历方向的索引相同） 第一版池级 checker 在 CPU 上判、内核不置位，GPU 与 CPU 对拍对这一条不比」。`crates/singlefs-checker/src/image.rs:117`「pub const IMPLEMENTED_INVARIANTS: [&str; 93] = [」，I-1.13 是最后一项（下标 92）。
2. 编译期断言证什么：`crash_judge_gpu.rs:162` 起的 `const _: () = { assert!(…len() == IMPLEMENTED_INVARIANTS.len() …); … assert!(COVERED[position] == position …) }`——覆盖表长度等于 `IMPLEMENTED_INVARIANTS` 的长度（93），且第 k 项是 k；`:109–116` 另两条断言：不变量数 ≤ 违例位容量 128。证不了：内核有没有置位（覆盖表在整个工作树里只被这两处断言和 `image.rs:115` 的一句注释引用，没有任何逻辑读它）；新立一条不变量而忘了加表，编译不过——这是它存在的目的。主 agent 实测 `cargo check --features gpu` 退 0 说的就是这一层。
3. I-4.4 先例今天怎么落的：CPU 一侧无条件报不适用，`crates/singlefs-checker/src/walk.rs:13418–13421`「judgements.not_applicable(」…「INTENT_DESCRIBES_ONLY_THE_TARGET_STATE,」，所以 `pool_checker_violated_and_judged_bits`（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:407`）对下标 73 两个位都给 0（`:421` 的 `NotApplicable | PartiallyJudged` 一支不置位）；GPU 内核也不置位，两边永远相等。测试里另有具名的例外：`crates/singlefs-checker-tier/tests/crash_enumeration_new_pool_file_creation_stream.rs:252`「let not_applicable_in_the_first_version = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];」、`crates/singlefs-harness/tests/checker_known_bad_images.rs:3169`「const INVARIANTS_WITHOUT_A_BAD_IMAGE: [&str; 1] = [INTENT_DESCRIBES_ONLY_THE_TARGET_STATE];」。
4. I-1.13 抄的是什么、少了什么：抄了覆盖表一项加文档一段。没抄、也没法照抄的是「两边恒等」这一条——I-1.13 在 CPU 一侧是真判：`walk.rs:12539` 的函数在能比时报成立或违例（`:12615` 起三张表逐项比），`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:420`「InvariantVerdict::Holds => (violated, judged | bit),」把它记进判到位的第 92 位，违例时记进违例位第 92 位；GPU 一侧 `crash_judge_gpu.rs:274` 起的 `cross_check_words_of_gpu_verdict` 从内核结论字读（`:281`、`:294`），内核不写第 92 位，恒 0。`CrossCheckWords` 逐字段比（`derive(PartialEq)`）。没有屏蔽：`crash_judge_gpu.rs` 与 `crash_judge_conclusions.rs` 里没有任何对下标 92 或对不变量下标的屏蔽。
5. 会红的断言（读代码得出，没跑，复核不了）：对拍用例 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 每个状态走 `:2603`「if gpu_cross_check_words != *cpu_cross_check_words {」→ `cross_check_mismatches` 加一；`:2607`「|| gpu_checker_bits != *cpu_checker_bits」（CPU 判违例的状态，违例位不等）；`:2608`「|| gpu_cross_check_words.pool_checker_judged_bits」（判到位不等：**CPU 上 I-1.13 成立也不等**，所以不限于违例态）。同样读 `cross_check_words_of_cpu_judgement` / `cross_check_words_of_gpu_verdict` 的还有 `crash_judge_gpu_tables_at_their_capacity.rs`、`gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core.rs`、`src/crash_amplification.rs:10654` 一带（后者按规定我没碰）。这些用例是 GPU 的、归提交时主 agent 跑。含义：在「不比」接上之前，这一批合进去之后，GPU 对拍对每个 CPU 判得了 I-1.13 的崩溃状态都会报不等；已知清单把「不比」写成第 3 批要接的，这里补的是它没接时的后果面（不限于违例态）。
6. 另一处同类（也读代码得出）：`crash_enumeration_new_pool_file_creation_stream.rs:272` 起对每条 `IMPLEMENTED_INVARIANTS` 按流给「评估过的状态数」期望，没有给 I-1.13 的专门分支，落到 `:304`「} else {」→ `every_state`（要求每个状态都评估过它，且违例数为 0）。这条流上有些状态 I-1.13 会不适用（实例表读不全、尚无文件的早期状态等），期望是不是 `every_state` 我没法核；checker 档，复核不了。

**什么会推翻第 4、5 点**：对拍字比较前另有一处把第 92 位屏蔽掉（`grep` 在 `crash_judge_gpu.rs`、`crash_judge_conclusions.rs` 与上面三份用例里找不到，但我没有读完所有读 `CrossCheckWords` 的文件）；或 CPU 一侧 I-1.13 在崩溃枚举用的入口上恒报不适用（崩溃点重放走的是 `check_pool_image`，`walk.rs:13715` 的调用无条件）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 四样配对 vs S1 与岔路 1 | 一致 | 条数相等、一一配对、至多 10、码 14 头 ID，各在 `recovery.rs:2422 / 2435 / 2442 / 2450` |
| T1 六种镜像的拒绝落点 | 一致 | 探针 P2 量到「码 14 多一条」五个读者同一句 |
| T1 六条用例与八行变异 | 缺口 | 「码 14 比码 15 多一条」无用例；`!=` 改 `<` 六条全绿，P2 在 `recovery.rs:2459` panic |
| T1 S1 点名的 checker 三处 | 规则没说（本批没做） | 合法快照镜像在池级 checker 上 I-9.16 ③ 会红（读代码得出） |
| T2 `mounted_read.rs` match | 一致 | 六个成员列全，无 `_ =>`；码 14、15 从不被挑中 |
| T2 五个读者对合法快照表 | 规则没说 | 三个放行、`walk_to_file` 与 `user_visible_tree_root_pointers` 报「树的种类没登记」（探针 P1） |
| T2 `RebuiltVersionWithItsSnapshotEntries` | 无产品消费者 | 只有用例命中；写者每次从头装十二条树表条目 |
| T2 `previous_snapshot_txg` 8 字节 | 一致 | `records.rs:512` 读出即丢，写时写死 0 |
| T3 偏移与宽度 | 一致 | 10 / 34 / 44 逐段对上，round-trip 用例手写期望字节 |
| T3 一节点 369 | 一致 | (16384 − 135) ÷ 44；只有式子，没有实建节点的用例 |
| T3 三条拒绝 vs D26 零格式位 | 一致（相容） | 读侧解码政策，不新增盘上字段 |
| T3 读侧接线 | 规则没说（下一批） | `key_width_for_kind(logged_ops)` 仍是 None |
| T4 五类 vs G9 | 一致 | G9 四类加隔离槽单列 |
| T4 判类优先级 | 规则没说 | 次序自定；两个优先级变异在 12 条用例下全存活 |
| T4 头读不出判不一致 vs G9「已死」 | 规则有重叠 | 实现取不一致优先，已死比 G9 字面窄 |
| T4 K-n 两盘不对称 | 规则没说 | 另一块盘上的孤记录扫不到；计划只有槽没有盘 |
| T4 跨 R 边界 | 一致（后果补充） | 拒整个扫描；选段不记上次被拒，同一 R 会反复被选 |
| T5 三条排除与 greedy | 一致 | `compaction_plan.rs:85 / 86 / 88`，同分取最小槽号 `:96` |
| T5 资格与空批 | 已知项（补后果方向） | 只剩已死记录的段分数低、常先被选；`plan_batch` 交空表分不出「搬空」与「搬不动」 |
| T6 默认实现的错误口径 | 一致 | 同 `check_aligned_and_in_range`，错误不是 `Unsupported` |
| T6 文件后端三支 | 一致 | 不支持不退回写 0；别的错照交；打断重打无上限（规则没说上限） |
| T6 长度 0 | 规则没说 | 四个后端两种结局 |
| T6 五行没证红的变异 | 2 行证红、3 行复核不了 | harness 两行在 `block_devices_record_and_apply_discards.rs:129 / 217 / 273` 红 |
| T7 harness 层两态 | 一致 | 持久读回 0、没持久读回旧字节 |
| T7 撕裂判据的第三态 | 冲突（读代码得出，复核不了） | G5 与 `memory_pool.rs:612` 说两态；`crash.rs:2156–2166` 对长 discard 给三态 |
| T7「不支持一字不动」 | 规则没说 | 靠各后端自己；真 `fallocate` 报不支持那支无用例 |
| T8 必红世界 ① ③ | 一致 | ① 红在中央映射一张表；③ 白名单双向比 |
| T8 必红世界 ② | 冲突（读法已收口） | 题面抹一个单元头；实现要两份副本都抹 |
| T8 常量表 | 一致 + 规则没说 | ① ② 之下仍红；第四类不适用理由是主 agent 口径 |
| T8 I-1.13 违例是否 I-3.1 / I-3.11 连带 | 一致（不是连带） | 常量表不含这两条，扫描侧不读映射树 |
| T8 不适用成员的可达性 | 补充 | 退役树一支在 unlink 到底后可达，不在已知清单 |
| T9 覆盖表收下标 92 | 一致 | 表是恒等列表，编译期断言只证完整 |
| T9 照 I-4.4 先例 | 抄错一格（读代码得出，复核不了） | I-4.4 CPU 恒不适用；I-1.13 CPU 真判，第 92 位判到位不等不限于违例态 |

## 要主 agent 跑的命令

1. 复跑我量的数（轻型，harness 与核心 lib，不是 checker 档；首次要编译，预计几分钟到十几分钟）。工作目录仓根 `~/.local/share/singlefs/work/m4-predev`；内存上限 8G，线程 8（脚本里每条 cargo 已经 `capped.sh 8`）：
   `bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-batch2-code-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen ~/.local/share/singlefs/work/m4-batch2-code-r1/repo --work <新的草稿目录>`
   看最后一行 `name=sonnet_forward_rerun_matches_expected result=same`；`result=differs` 时 diff 在上面。不要对这份 `rerun.sh` 跑 `--selftest`：模板自带的自证测的是模板那一段步骤，我换了。
2. T7 第三态探针（checker 档，重型，我没编过也没跑）：把 `research/prompts/m4-batch2-code-r1-sonnet-model/probes/checker_tier_probe_forward_tearable.rs` 拷成 `crates/singlefs-checker-tier/tests/probe_forward_tearable.rs`，提交时跑 `SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/hosts/capped.sh 8 cargo test -p singlefs-checker-tier --test probe_forward_tearable -- --nocapture`。看两行 `PROBE_T7`：16384 字节那行 `contains=true` 说明第三态确实出现；512 字节对照应为 `contains=false`。探针可能要补 `use`，编不过就按报错改。
3. T9 的 GPU 对拍（提交时本来就要跑的）：`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 用 `--features verdict-store,gpu`。看 `CRASH_JUDGE_GPU_STATE` 行里 `gpu=` 与 `cpu=` 两个 `CrossCheckWords` 的 `pool_checker_judged_bits` 是否只差第 92 位（`1 << 92`）。我预期有；没有则第 4、5 点作废。
4. 变异表里 `crates/mutations.tsv:4051–4053` 三行（checker 档）的证红：归 `checker-tier-crates-mutation-replay` 轻型 / 提交时整表。

## 没做什么

- 没判别：不替主 agent 采纳或出判决；T8 第 2 点的「冲突」与 T7 第 3 点的「冲突」只并排两边，用哪边由主 agent 定。
- 没跑任何门禁阶段、checker 档测试、GPU、`crash_amplification*`、WGSL。T7 第三态、T9 的红在哪条断言、`mutations.tsv` 的 3 行 checker 档变异：都是读代码得出，标了「复核不了」。
- 没逐行复跑 `crates/mutations.tsv:4005–4012` 的 8 行、`:4035–4053` 里除 4037、4040 以外的行；只复跑了我自己提的变异（`!=` 改 `<`；两个优先级变异）和 4037、4040。
- 没读 `.claude/kb/decisions/08-核心索引结构.md` 已定项 8 全文（只引了已定项 5 里「意图本身就是一条普通 key」那一行）、D5 已定项 9 / 14、D3 已定项 7。T1 对 D8 已定项 9 / D5 已定项 14 的核对只到 S1 判决行与岔路 1 行为止。
- 没看 `crates/singlefs-core/src/transaction.rs` 里写者装树表的全部逻辑，只看了 `:8649` 那一个 `vec![`；「建快照那一批必须先改这里」是从这一处推的。
- 没读禁读清单里的任何文件（攻方的报告、模型目录、草稿目录；四份实现员报告里「停下交主 agent」之外的结论句）。读过的实现员报告只有 `m4-discard-writer/report.md`，为了确认「5 行没证红」是哪几行；我没抄它的结论。

## 临时文件与副本

- 草稿目录 `~/.local/share/singlefs/work/m4-batch2-code-r1-forward/`：保留 `build.sh`、各 `*.log`、探针与 `.at` 草稿；编译目录 `target/`（1.9G）、副本 `repo/`、`stubwork/` 在交回前删掉（理由：模型目录的 `rerun.sh` 能在新草稿目录里重建，不需要留着）。
