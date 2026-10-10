# E165 第 5 次重跑用事实表：journal 记录格式自第 4 次重跑以来的改动（2026-10-04）

批次 `records/2026-09-30-里程碑三开工第一轮.md#9`（9-7 拆出的 E165 事实表，记录第 552 行）。只读，仓里没改任何文件。草稿 `/tmp/claude-1000/m3-r1/batch9-e165-facts/`（`journal.diff`、`recovery.diff` 是第 4 次根与今天工作区的 `diff` 原样，204 行与 1040 行）。

## 0　先说结论

1. 第 4 次重跑跑在「7-2 打进工作区之前」的那一版上（根 `/tmp/claude-1000/e165-r4-segment1b/root`，开工清单文件时间 2026-10-02 17:35 UTC = 日本时间 10-03 02:35；7-2 补丁 10-03 09:39 日本时间才打进，记录第 456 行）。那一版是旧格式：新根段 188、记录头 311、点名项 67（`diff` 原样见第 1 节）。
2. 格式改动全部来自 **7-2**；9-3、9-5 两个补丁都没改记录格式、没改 `journal.rs` 与 `recovery.rs`：9-3 只在 GPU 判器里照 7-2 的格式重写读法，9-5 加清扫器与读坏槽处置（第 2 节逐条核）。
3. E165 装置对记录格式**直接调 `crates/singlefs-core` 的代码**（`JournalRecord::to_bytes` / `parse`、`back_chain_of`、`record_offset`、`scan_journal`、`replay_journal`、`choose_root`），没有抄编解码；本地抄的常量只有 7 个（都不是这三个被改的宽），启动时逐个回比。装置源码只因 `JournalRecord` 多三个字段补了五处结构体字面量。
4. 要注意的不是宽度而是**施加规则**：施加一条记录时，实例表指针、分配记录树根、卸载记号现在取记录的，不再抄所选根。装置里 r2 / r3 档的 J7 判据与 `to_root_rebuilt_over` 还照旧规则登记，今天与新规则对不上（实现员在源码注释里自己写了，`e165_replay_bounded_enumeration.rs:380-387`）。第 4 次的 root_choice_faults 一族 KJ3 只比六个投影字段（`VersionFields`），不比这三项。
5. 你写的「新根段三字段（首个文件、卸载记号、实例表指针）」要改一处：新根段后三项是**实例表指针、分配记录树根指针、卸载记号**；「首个文件、卸载」是 `recovery.rs:3596-3598` 注释里点名的两种发布，旧规则（照抄所选根）在这两种发布上施加出错根（`research/prompts/m3-ring-r3-main-verification.md` 第二节第 1、2 行，注释原文引）。

## 1　格式改了哪几处，落点（今天的工作区，行号现取）

「第 4 次那一版」指冻结根里的文件；`diff` 是 `diff <冻结根的文件> <今天的文件>`，下表引的是原样的行。

| # | 改动 | 第 4 次那一版 → 今天 | 落点（今天的文件:行） | 依据 |
|---|---|---|---|---|
| 1a | 新根段宽 | 188 → 364 | `crates/singlefs-format/src/lib.rs:220` `pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 364;`；注释 218-219 写「树表指针 86 + 中央映射树根指针 86 + 树 ID 水位 8 + 回退下界 F 8 + 实例表指针 86 + 分配记录树根指针 86 + 卸载记号 4」 | `diff` 冻结根 `lib.rs` 与今天：`203,204c218,220` 与 `< pub const JOURNAL_NEW_ROOT_SEGMENT_BYTES: u64 = 188;` |
| 1b | 记录头宽 | 311 → 487 | `lib.rs:223` `pub const JOURNAL_HEADER_BYTES: u64 = 487;` | 同上 `206,207c222,223`、`< pub const JOURNAL_HEADER_BYTES: u64 = 311;` |
| 1c | 点名项容量 | 67 → 64 | `lib.rs:229` `pub const JOURNAL_NAMED_ENTRIES_PER_RECORD: u64 = 64;` | 同上 `212,213c228,229`；点名项宽 56（`lib.rs:226`）、记录 4096（`lib.rs:213`）未变；(4096 − 487) ÷ 56 = 64 |
| 1d | 格式单测钉值 | 188 / 311 / 67 → 364 / 487 / 64 | `lib.rs:426-429`（三个 `assert_eq!`）；新根段加和算式 `lib.rs:537-539` | `diff` `410,411c426,427`、`413c429`、`488,489c538,539` |
| 2a | 新根段多三个字段 | 无 → `new_instance_table: NodePointer`、`new_allocation_record_tree_root: NodePointer`、`new_unmount_marker: UnmountMarker` | `crates/singlefs-core/src/journal.rs:166-168`（结构体 `JournalRecord`，`journal.rs:149` 起）；`use crate::root_record::UnmountMarker` 在 `journal.rs:19` | `journal.diff` `162a164,168` 与 `18a19` |
| 2b | 编码（写） | 4 项 → 7 项，新根段顺序：树表指针、映射根指针、水位、F，**实例表指针、分配记录树根指针、卸载记号 4 字节（`flags_bits()`，位 0）** | `journal.rs:249-252` 写后三项；`journal.rs:255` 断言段宽 `.expect("364")`；`journal.rs:266` 载荷起点 `.expect("487")` | `journal.diff` `242a249,252`、`245c255`、`256c266` |
| 2c | 解码（读） | 读 4 项 → 读 7 项；卸载记号位 0 之外有位为 1 ⇒ 整条记录当损坏（`parse` 返回 `None`）；新根段四条指针头都判加密、压缩、extent 偏移是第一版取值 | `journal.rs:335-346` 读后三项（`journal.rs:345-346` 调 `UnmountMarker::of_unmount_marker_field_of_a_new_root_segment`）；`journal.rs:380-382` 装进返回值；记号解码本体在 `crates/singlefs-core/src/root_record.rs:43` | `journal.diff` `323a335,346`、`356a380,382`、`268,269c278,280`；`root_record.rs:43-48` |
| 2d | 偏移（盘上布局） | fsid [287,295)、MAC [295,311) → 实例表指针 [287,373)、分配记录树根指针 [373,459)、卸载记号 [459,463)、fsid [463,471)、MAC [471,487) | 写进了 `journal.rs` 的单测：`journal.rs:450-452`（fsid [463,471)）、`journal.rs:455`（MAC [471,487)）、`journal.rs:465`（三项往返）、`journal.rs:535`（卸载记号 `bytes[459..463]`） | `journal.diff` `421,423,426,427c…`、`431a528,544` |
| 2e | 反向链覆盖的头宽 | 前一条记录的 311 字节头 → 487 字节头 | `journal.rs:174-176` `back_chain_of`；`recovery.rs:3399` 注释 | `journal.diff` `166c172`、`170c176`；`recovery.diff` `3099c3399` |
| 3 | **施加：从记录取七项** | 旧：取记录的 instance、checkpoint_txg、树表、映射根、水位、F，**其余三项照抄所选根**（`unmount_marker: NotWritten…`、`instance_table: root.instance_table`、`allocation_record_tree_root: root.allocation_record_tree_root`）→ 新：七项全取记录，只有 fsid 取所选根 | `crates/singlefs-core/src/recovery.rs:3594-3611`（`replay_journal`，`pub fn replay_journal` 在 `recovery.rs:3467`）：`:3601` `unmount_marker: record.new_unmount_marker,`、`:3607` `instance_table: record.new_instance_table,`、`:3609` `allocation_record_tree_root: record.new_allocation_record_tree_root,`；注释 3596-3598 | `recovery.diff` `3296c3596,3598`、`3299,3302c3601`、`3308c3607`、`3310,3312c3609` |
| 4 | 三步里没动的 | `choose_root`、`scan_journal` 函数体 | `choose_root` 今天 `recovery.rs:1268-1305`（第 4 次根里 1255-1292，逐行 `diff` 相同）；`scan_journal` `recovery.rs:3338`（函数体 `diff` 只差 `:3399` 一句注释里的 311 → 487） | 命令 `diff <(sed -n 1255,1292p 冻结根) <(sed -n 1268,1305p 今天)` 输出「choose_root body … identical」 |

## 2　这些改动出自哪一批

出处：`records/2026-09-30-里程碑三开工第一轮.md` 的行（现取）、三份补丁报告、`git log`。提交是按类型分笔的（一笔「格式常量」「核心层」「checker 档」……，2026-10-03 UTC），一笔里混着多批，**从提交号分不出批次**；批次靠记录与补丁文件清单定。

| # | 改动 | 出自 | 依据 |
|---|---|---|---|
| A | 用户定案：新根段回到甲（加三个字段）；窗口五处同改；读错判法先验证 R2 | 第 5 批 5-1 `m3-ring-r3`（终轮三方）的判决，用户 2026-10-02 | 记录第 353 行（「用户 2026-10-02 定：回到甲（三字段）；窗口五处同改；读错判法先验证 R2」）、第 377 行（5-1 行）；判决 `research/prompts/m3-ring-r3-main-verification.md` |
| B | kb 里的格式常量 311 → 487、188 → 364、67 → 64（三个 `format-const:` 标记） | 第 5 批写回员乙（`/tmp/claude-1000/m3-r1/batch5-kb-b/report.md`）；记录说明「`crates/` 还没跟上，不许单独提交」 | 记录第 368 行（「头 311 → 487、新根段 188 → 364、点名项 67 → 64（三个 format-const 标记改了值）。**这是格式改动、`crates/` 还没跟上」）；第 384 行（「格式常量改了……`crates/` 里还没有根环实现」） |
| C | **`crates/` 里的常量、编解码、施加、`JournalRecord` 多三个字段、卸载记号位 0 之外当损坏** | **第 7 批 7-2 根环实现**（implementation-writer，补丁 `/tmp/claude-1000/m3-r1/batch7-root-ring/patch/`，报告 `…/patch/report.md`） | 记录第 441 行（7-2 行：「新根段三字段与恢复施加（首个文件、卸载、实例表指针）」）；第 455 行（交回；「卸载记号位 0 之外非 0 当记录损坏」是用户 2026-10-03 弹窗定的第一条）；第 456 行（「7-2 补丁 10-03 09:39 打进工作区」）。补丁文件清单含 `crates/singlefs-format/src/lib.rs`、`crates/singlefs-core/src/journal.rs`、`recovery.rs`、`root_record.rs`、`e165_replay_bounded_enumeration.rs`（命令：`grep '^diff --git' …/batch7-root-ring/patch/crates.patch`）；报告第一节第 1 行「头 487、新根段 364、点名项 64；记录带实例表指针、分配记录树根、卸载记号（4 字节，与根记录 flags 同编码）；恢复施加七项全取记录」 |
| D | 7-2 为让 E165 装置编得过，补五处 `JournalRecord` 字面量 | 同 C（7-2 报告「清单外动了的文件」表第一行） | 报告原文：「补的值一律取 r1 档那份常量（新加 `fields_beyond_the_projection_carried_by_every_generated_record`），r1 档逐态不变，r2 / r3 档的 J7 判据照……」；今天落在 `e165_replay_bounded_enumeration.rs:380-387`（函数）与五处字面量（第 3 节） |
| E | 7-2 里另有一项不是格式、但碰了 E165 第 4 次的前提：系统配置读错取 R2（`recovery::system_configuration_witness_is_undecidable`，`recovery.rs:3322`；`mount.rs:5360` 用） | 同 C | 7-2 报告第一节第 3 行；第 4 次实验页第 616 行写着「闸取的是实现今天的取甲……R2 进了代码之后这一族要重跑」（第 4 次实验页 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:616`） |
| F | 9-3 GPU 簇：GPU 判器里照 C 的格式重写读法（RS5：恢复内核收卸载记号其余位、新根段四条指针头判全、施加末条取七项；RS6：走读内核施加版四条指针全取记录；I-8.10 后三段） | 第 9 批 9-3（补丁 `/tmp/claude-1000/m3-r1/batch9-gpu/patch/`）；**不改 `crates/singlefs-core`、`singlefs-format`** | 补丁文件清单（`grep '^diff --git'`）只有 `crash_judge_checker.wgsl`、`crash_judge_checker_common.wgsl`、`crash_judge_common.wgsl`、`crash_judge_dispatch.rs`、`crash_judge_gpu.rs`、一个 `tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`；记录第 526 行（9-3 行）、第 538 行（交回）；报告第 2、3 行 |
| G | 9-5 core 簇：RS1（读坏槽处置）、RS2 用例、RS3 抬 F 上限、`mount_transparency.rs` 改 R2、清扫器 `instance_table_sweep.rs` | 第 9 批 9-5（补丁 `/tmp/claude-1000/m3-r1/batch9-core2/patch/`）；**不改 journal 记录格式、不改 `journal.rs`、`recovery.rs`、format `lib.rs`** | 补丁文件清单只有 `allocator.rs`、`instance_table_sweep.rs`、`lib.rs`、`mount.rs`、`mount_transparency.rs`、`crash_injection.rs` 与几个用例；`grep -n "新根段\|journal.rs\|JOURNAL_\|487\|364\|replay_journal\|recovery.rs\|格式" …/batch9-core2/patch/report.md` 命中 0 行；记录第 528 行、第 541 行 |

落进 git 的提交（2026-10-03 UTC）：格式常量在 `c3e9d9ed 格式常量模块小改`（`git log -1 -- crates/singlefs-format/src/lib.rs`），`journal.rs`、`recovery.rs`、`root_record.rs` 在 `5d637469 核心层源码调整，新增索引与分配相关模块`，E165 装置在 `13ca0a19 崩溃检查档测试与源码对齐，用例按模块与内容起名`（**这是 `e165_replay_bounded_enumeration.rs` 在 git 里的第一笔**：`git log -- 该文件` 只有这一笔，17677 行整份新增；第 4 次那一版的装置只在冻结根里，sha256 `4ecfa61fda6e2c0764e748533bba37abcfbed86c4c7129dac67d7f80539ac031`，与 4 次实验页写的相同）。9-3、9-5 的补丁在工作区里未提交（`git status`：`crash_judge_common.wgsl` 是 `M `，`crash_judge_gpu.rs` 是 `MM`，`instance_table_sweep.rs` 是 `A `）。

## 3　E165 装置哪几段、哪几个文件读到这些格式

装置文件清单现查：`crates/singlefs-checker-tier/Cargo.toml` 依赖 `singlefs-core`、`singlefs-format`（第 10、11 行），**不依赖 `singlefs-replay-prefix`**（`grep -n "singlefs-replay-prefix" crates/singlefs-checker-tier/Cargo.toml` 无命中）；`crates/singlefs-replay-prefix/src/lib.rs` 的 `CoreRecord<RootSource>` 对新根段里装什么不过问（`lib.rs:66-76`，`new_root_source` 注释「核不看它」），E165 只经 `singlefs_core::recovery::replay_journal` 间接走到它。

装置本体：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`（今天 17677 行，sha256 `d6937857c6613a085a16f775a7c71cce3ba75084ab9cb0e4d3e6680fd282b690`；第 4 次那一版 17623 行 `4ecfa61f…`；两版 `diff` 144 行，除五处补三字段与 `:380-387` 新函数外，是 `const {}` 包断言与几处变量改名，与格式无关）。

| # | 装置的哪一段（今天的行） | 读到什么 | 直接调 `crates/` 还是自己抄 | 这次格式改动对它的影响 |
|---|---|---|---|---|
| 1 | 开头导入 `:45-67`；文件头注释 `:4-5` | `singlefs_core::journal::{back_chain_of, record_offset, JournalRecord, …}`、`singlefs_core::recovery::{choose_root, scan_journal, replay_journal, …}`、`root_record::{RootRecord, UnmountMarker}` | 直接调（`use`） | 无宽度常量；结构体多三个字段，编译要补字面量 |
| 2 | 生成一条记录槽 `generate_slot` `:629-669`：`record.to_bytes()` `:635`、`JournalRecord::parse(&bytes, …)` `:636`、`back_chain_of(&bytes)` `:669`、`record_offset(…)` `:661` | 记录的整条编解码（头 487、新根段 364 全在 core 里） | **直接调 core**，装置不碰字节偏移 | 自动随 core 变；停机条款 H4 要求 `to_bytes` 再 `parse` 等于原记录（`:641`），新字段往返不过会当场停 |
| 3 | 五处 `JournalRecord { … }` 结构体字面量：`generate_history` `:883`（新三项在 `:903-911`）、`generate_the_anchor_slot` `:4795`（`:4809-4814`）、`generate_a_record_slot` `:4844`（`:4868-4873`）、`hand_made_record` `:7584`（`:7602-7607`）、`root_choice_publish_material` `:9118`（`:9138-9143`） | 记录的字段 | 直接构造 core 的结构体；三项值取 `fields_beyond_the_projection_carried_by_every_generated_record()`（`:384`）= r1 档时间线那一份常量（`RunProfile::R1.fields_beyond_the_projection_of_version(0)`） | 7-2 为编得过加的；值是「所选根的那三项」，所以 r1 档逐态不变；**r2 / r3 档里根的这三项随版本变（`:339`），记录里却一律是 r1 那一份，二者不再同值**（见第 4 点） |
| 4 | 投影之外字段与 J7：`FieldsBeyondTheProjection` `:390`；`to_root_rebuilt_over` `:597-604`（实例表指针、分配记录树根、fsid 抄 `chosen_root`，卸载记号写 `NotWritten…`）；理想恢复期望 `outcome_of_the_expectation` `:1854`（用于 `:1866`）、`returned_at_version` `:2904`、`off_timeline_root` `:2926`；判据 J7a/J7b/J7c：枚举 `:2783-2790`、`:2800-2830`，判据函数 `judge_the_fields_beyond_the_projection` `:3284`（调用 `:3669`、`:5304`）；对照 `:5153`；自检 `check_the_fields_beyond_the_projection_of_the_r2_generator` `:8005`（`:8100`、`:8122`） | 旧施加规则：实例表指针、分配记录树根、fsid 抄所选根，卸载记号「由记录施加出来的那一版不是卸载写的」 | **自己抄了一份旧规则**（装置本地写的期望，不调 core 的施加） | **与今天 `recovery.rs:3601-3609` 不一致**：施加现在把这三项换成记录里的。源码注释 `:380-387` 自己写了「J7 那几行判据照『投影之外抄所选根』登记，与新的施加规则对不上，要重新登记」。r2 / r3 档（合法一族、对抗一族、多盘一族）里这是会有结果变化的一段；r1 档与根环一族（下一行）不受影响 |
| 5 | 根环择根一族（r3 `root_choice`、r4 `root_choice_faults`）：`root_choice_fields_beyond_the_projection` `:9029`（取 r1 档常量）、`root_material_without_records` `:9054`（`root.to_slot(512)` 再 `RootRecord::parse_slot`，TH9）、KJ3 比较 `VersionFields::of_root` `:10227`、`:10233`、`:13894`、`:13900`（六个投影字段：实例代号、txg、树表、映射根、水位、F） | 根槽 457 字节格式、记录格式 | 调 core（`to_slot`、`parse_slot`、`choose_root`、`scan_journal`、`replay_journal`：`:10191`、`:10202`、`:10212` 与 `:13859`、`:13869`、`:13879`） | KJ3 只比六个投影字段，**不比这三项**，所以三项改动不改判据；但被测的三步现在是今天的 `replay_journal`（`recovery.rs:3467`），与第 4 次那份比，只有施加那一段不同（第 1 节第 3、4 行） |
| 6 | 本地抄的常量（7 个）`:73-86`：`LOCAL_JOURNAL_RECORD_BYTES` 4096、`LOCAL_JOURNAL_SAFETY_FACTOR` 3、`LOCAL_JOURNAL_RING_START_SLOT` 1024、`LOCAL_SLOT_BYTES` 16384、`LOCAL_FIRST_JOURNAL_COUNTER` 1、`LOCAL_INDEX_NODE_UNIT_BYTES` 16384、`LOCAL_UNIT_CLASS_INDEX_NODE` 2；回比 `cross_check_local_constants_against_the_implementation` `:7508` 与根环一族的 `cross_check_root_choice_local_constants_against_the_implementation` `:10966`（用 `singlefs_format::ROOT_RECORD_BYTES`、`ROOT_RING_*`、`WARM_UP_EMPTY_PUBLISHES` 等） | 值抄自 kb | **自己抄了一份**，启动时对着 `singlefs_format` 回比（H2 / TH2） | **这 7 个与 311 / 188 / 67 无关**，没有一个是被改的宽；装置里没有 `JOURNAL_HEADER_BYTES`、`JOURNAL_NEW_ROOT_SEGMENT_BYTES`、`JOURNAL_NAMED_ENTRIES_PER_RECORD` 任何一个名字（`grep -c` 各 0），也没有 487、188、364、67 的字面量 |
| 7 | 点名项 | 每条记录点名项数 | 装置每条记录只放一个点名项（`:912`）或 `Vec::new()`（`:4816`、`:4875`、`:7609`、`:9145`） | 容量 67 → 64 碰不到（最多 1 项） |
| 8 | 过时的注释 | 「311 字节头」 | 注释 | `:619`、`:9091` 两处注释还写 311 字节头（代码不受影响；`:72` 的 `23-journal的角色与格式.md:311` 是 kb 文件行号，不是头宽） |

装置之外、E165 登记的驱动与脚本（`research/scripts/`）：

| # | 文件 | 读什么 | 抄还是调 | 影响 |
|---|---|---|---|---|
| 9 | `e165-replay-mutation-kill-bounds.sh`（3365 行，今天 sha256 `3428a78e…` 与第 4 次页写的相同） | 只读 `crates/singlefs-core/src/recovery.rs` 与 `crates/singlefs-replay-prefix/src/lib.rs` 的整份 sha256、两个函数区间（`choose_root_slot`、`choose_root`）与 `crates/mutations.tsv` 的 sha256（`:192-193` 路径常量，`R4_REGISTERED` `:340` 起），不读记录格式 | 自己写死 | **r4 档钉的值今天全不符**：`recovery.rs` 第 4 次钉 `264664af…`、今天 `4454f6d3eab170121ccb92778e7d4525cdaa6a10a6c6bf16d7c213d0edc22632`；`lib.rs`（replay-prefix）钉 `40cd99b5…`、今天 `9966f42f396d22c2a90bdd35864557edd7ce45285be3d6b796ba9f8d18aa2fa4`（`diff` 只差 `:3072` 一处注释，7 行）；`crates/mutations.tsv` 钉 `db2981d0…`、今天 `d6ab937702fd9eed6f149d6d0b1fb6f8d43decf53dfaac2ae81422ff490f2720`；`choose_root` 今天在 `recovery.rs:1268-1305`（钉的是 1255-1292，体相同、只是下移 13 行），`choose_root_slot` 今天在 replay-prefix `lib.rs:3980`（钉 3977-4016 起、下移 3 行一起核）。这些钉只在 r4 档与 `--root` 指的冻结根上用；r5 要新档与新根 |
| 10 | `e165-pin-red-examples.py`（1236 行）、`e165-compare-kill-bounds.py`（308 行）、`replay.sh` 的 E165 十六行（`:224-238`） | 产物文件名、装置命令行 | 不读格式 | 无 |
| 11 | 第 4 次的根 `/tmp/claude-1000/e165-r4-segment1b/root`（还在，已核 `ls`） | 旧格式整份副本 | 冻结 | 只用来对照旧产物；r5 的产物要在今天的工作区上新建根 |

## 4　没做什么

- 没跑任何编译与测试，没验证今天的 `e165_replay_bounded_enumeration.rs` 编得过、自检全绿（编译归主 agent / 事务员；今天只读了源码）。
- 没判 r2 / r3 档的 J7 行在新施加规则下会红多少（要跑 r2 / r3 档，重型）；只指出装置里的期望与 `recovery.rs:3601-3609` 的规则不一致，推的：J7a 的实例表指针与分配记录树根两行、J7b 的卸载记号一行会在「施加了记录」的态上因规则变而有读数变化，没量过。
- 没查第 4 次登记与实验页里哪几句要改（如 `.claude/kb/experiments/165-…md:616` 的「闸取的是实现今天的取甲」现在与 7-2 的 R2 不符；第 4 次产物对应的实现是旧版），那归 r5 登记设计员。
- 没核 `m3-ring-r3-main-verification.md` 第二节第 1、2 行的原文，只引 `recovery.rs:3596-3598` 注释对它的转述（文件在仓里，`ls` 在）。
- 没动仓里任何文件；没提交。

什么现象会推翻这张表：① 第 1 节第 3、4 行说 `choose_root`、`scan_journal` 与第 4 次相同——若另一个会话又改了 `recovery.rs`（`git status` 现示 `M`，今天只有一处注释差，`git diff HEAD --stat` 是 2 增 1 删）这就会变；② 第 3 节第 6 行说本地抄的 7 个常量与三个被改的宽无关——若装置新加了对 `JOURNAL_HEADER_BYTES` 的引用（现 `grep -c` 为 0）就不成立；③ 第 2 节 G 行说 9-5 不改记录格式——若合入后的 `git diff` 里 `journal.rs` 有它的改动就推翻。
