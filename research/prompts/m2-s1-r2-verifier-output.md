# m2-s1 增补1 第二轮 核查员报告

**核对表写进本文件。核对表里的 ✗ 不免除主 agent 对推论的逐条现查——这一句照抄自派发提示。**

## 0. 判别力自证（先做，必须判 ✗）

抽的引用：Sonnet 报告第 9 行 `crates/singlefs-core/src/recovery.rs:821` 的 `pub fn replay_journal(`。
在草稿目录（`/tmp/claude-1000/three-way-verifier/draft/`）把行号加 1（改核第 822 行），核同一句原文：

```
$ sed -n '822p' crates/singlefs-core/src/recovery.rs
    reader: &dyn PoolReader,
```

第 822 行是 `reader: &dyn PoolReader,`，不是 `pub fn replay_journal(`——判 ✗。核查方法能分辨行号错位，往下按此法核。

## 1. 快照对不上：13 个文件里 4 个 `sha256sum -c` 判 FAILED

```
$ sha256sum -c research/prompts/m2-s1-r2-start-snapshot.sha256
research/prompts/_m2-s1-r2-body.md: FAILED
.claude/kb/checks-owed.md: FAILED
.claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md: FAILED
research/prompts/m2-s1-forks.md: FAILED
（其余 9 个 OK）
```

逐个倒推（`git show HEAD:<路径> | sha256sum` 与快照比）：

| 文件 | HEAD 版 sha256 是否等于快照 | 结论 |
|---|---|---|
| `_m2-s1-r2-body.md` | **相等** | 快照 = HEAD（commit `495bede` 里新增的那份）；工作区里现在的差异是 2026-09-21 那次 D25 指针修正（body.md 自己头部写明的那处），核引用时用 `git show HEAD:...` 取值即可，与快照逐字节相同 |
| `research/prompts/m2-s1-forks.md` | **相等** | 快照 = HEAD；工作区未提交差异是主 agent 后续把第 5、8 行状态从「开着」改成「够判」，与这一轮三条腿的引用无关 |
| `.claude/kb/experiments/155-…md` | **相等** | 快照 = HEAD；工作区未提交差异是同一次 `495bede` 改名带来的字段表段落调整（10 行），与三条腿引用的行号无关（下面核对时逐条现查过） |
| `.claude/kb/checks-owed.md` | **不相等**（HEAD `958fe7…`、其父 `495bede^` `9cc50b…`、`4105e70^` `540a32…`、`0ec0be2^` `5b4870…` 都不等于快照 `d0492c…`） | 快照时点的原样在已知的 git 版本链上找不到，倒推不出精确原文。**无碍**：核过三条腿的全部引用，没有一条引 `checks-owed.md`，此文件的核实缺口不影响本轮判定 |

`crates/` 未随附单独快照（本轮不是代码轮的意义上的「新代码」，是设计轮引现有代码作证据），按派发提示「以盘上现状为准」处理；`crates/singlefs-core/src/{recovery,transaction,mount}.rs`、`crates/singlefs-format/src/lib.rs` 现在都在被另一位实现员（拆 `SystemConfiguration`）编辑中（`git status --porcelain` 判有）。逐条核对时区分「HEAD（腿交回时的状态）」与「当前工作区（实现员正在改）」两个版本，凡是两者不一致的都单独标「行号漂移」，不判 ✗。

## 2. 云端辩方（Sonnet）`research/prompts/m2-s1-r2-sonnet-output.md`（152 行，sha256 与自报一致：现算 `3da0d2209c89c00b01a6c47efd5b3429c29486c7d7ce84ccfd7da0b5a46be39b`）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `singlefs-format/src/lib.rs:145,148,139,151,155`（journal 五个常量） | ✓ 全部逐字匹配 | `sed -n '139p;145p;148p;151p;155p' crates/singlefs-format/src/lib.rs` |
| `singlefs-format/src/lib.rs:132-133`（树 ID 水位两常量） | ✓ 匹配 | `sed -n '130,134p' crates/singlefs-format/src/lib.rs` |
| `singlefs-core/src/journal.rs:81-93`（`JournalRecord` 结构体，四个 `new_*` 字段在 89-92） | ✓ 逐行匹配（81 struct，89 new_tree_table，90 new_mapping_root，91 new_tree_identifier_watermark，92 new_rollback_floor，93 named） | `sed -n '81,93p' crates/singlefs-core/src/journal.rs` |
| `singlefs-core/src/journal.rs:27,29,31,70-77`（`locations`/`birth_tree`/`key_tail`/`mapping_key`） | ✓ 匹配 | `sed -n '27p;29p;31p;70,77p' crates/singlefs-core/src/journal.rs` |
| `singlefs-core/src/records.rs:260-270`（`TreeTableEntry`，单一 `root` 字段） | ✓ 匹配 | `sed -n '260,270p' crates/singlefs-core/src/records.rs` |
| `singlefs-core/src/recovery.rs:821-917`（`replay_journal` 全函数，声称除 882-896 校验和验证外无树遍历、无分配器调用） | ✓ 逐行核过 821-918 区间，`grep -n "allocator\|Allocator"` 零命中，无第二次磁盘读、无树节点解析 | `sed -n '821,918p' crates/singlefs-core/src/recovery.rs \| grep -n allocator` |
| `singlefs-core/src/recovery.rs:905-914`（四个 `new_*` 字段整体搬进 `rebuilt`） | ✓ 逐字匹配（905 `rebuilt = RootRecord {`……914 `};`） | 同上 |
| `singlefs-core/src/recovery.rs:922`「`TREE_KIND_INODE => Some(8)`」 | **✗ 行号错**：922 行实为 `TREE_KIND_EXTENT => Some(24)`；`TREE_KIND_INODE => Some(8)` 在第 **923** 行。此区间不在实现员当前编辑的 diff hunk 范围内（HEAD 与工作区一致），不是行号漂移，是引用本身错了一行。位于报告「什么观测会推翻这一条」的自我留白段，非判据主干 | `grep -n "TREE_KIND_EXTENT => Some\|TREE_KIND_INODE => Some" crates/singlefs-core/src/recovery.rs` |
| `singlefs-harness/src/crash.rs:461`/`461-466`（`RecordCheck` 结构，两个 `bool` 字段） | ✓ 匹配（461 `pub struct RecordCheck {`） | `sed -n '455,467p' crates/singlefs-harness/src/crash.rs` |
| `singlefs-harness/src/crash.rs:477-493`（`publishes_in`） | ✓ 匹配 | `sed -n '477,493p' crates/singlefs-harness/src/crash.rs` |
| `singlefs-harness/src/crash.rs:520-602 附近`（`check_records_against`） | ✓（原句用「附近」自留余地，`check_records_against` 起于约 520 行，范围内） | `sed -n '518,525p;598,604p' crates/singlefs-harness/src/crash.rs` |
| D23（journal 的角色与格式） `23-journal的角色与格式.md:340,350`（已定项 14 标题、⚠️ 第六条） | ✓ 逐字匹配 | `sed -n '340p;350p' .claude/kb/decisions/23-journal的角色与格式.md` |
| D23 `:392,394`（已定项 15 标题、定案） | ✓ 逐字匹配 | `sed -n '392p;394p' 同上` |
| D23 `:396,403`（射程、欠 C284） | ✓ 逐字匹配 | `sed -n '396p;403p' 同上` |
| D16（发布语义） `16-发布语义.md:99,101`（已定项 4 标题、定案） | ✓ 逐字匹配 | `sed -n '99p;101p' .claude/kb/decisions/16-发布语义.md` |
| D19（块指针的结构与宽度预算） `19-块指针的结构与宽度预算.md:94`（中央映射） | ✓ 匹配（此文件不在这一轮 13 个快照文件内，核的是当前主树；内容一致，无历史版本可比） | `grep -n "中央映射.*解引用与释放判定的唯一入口" .claude/kb/decisions/19-块指针的结构与宽度预算.md` |
| D23 `23-journal的角色与格式.md:669`「已定项 20 标题」、`:671`「journal 记录不是『事务的全部』…」 | **✗ 误写成背景材料行号**：`research/prompts/_m2-s1-r2-background.md` 第 **669** 行正是这句标题、第 **673** 行正是这句引文（671 与 673 差 2，同一段落内），而 `.claude/kb/decisions/23-journal的角色与格式.md` 本身只有 614 行，669/671 根本不存在；该文件里这两句实际的行号是**第 468 行**（标题）与**第 472 行**（引句），此文件在快照里且与当前一致，非行号漂移 | `wc -l .claude/kb/decisions/23-journal的角色与格式.md`；`grep -n "已经写到盘上的东西的发布指令" .claude/kb/decisions/23-journal的角色与格式.md research/prompts/_m2-s1-r2-background.md` |

**Sonnet 计数：核了 17 处，✓ 15 处，✗ 2 处（`recovery.rs:922` 行号错一行；`23-journal的角色与格式.md:669/671` 误写成背景材料行号）。核不动：0。**

### 2.1 主 agent 要求复核的两处（回抄原样，不判推论）

**(a) `replay_journal` 只搬四个 `new_*` 字段**：主 agent 观测与 Sonnet 报告一致，已在上表核实——`recovery.rs:821-918`（HEAD 与当前工作区在此区间相同，实现员正在改的部分是别处的 `Superblock → SystemConfiguration` 改名，diff hunk 不落在 821-918）确认只有 905-914 行的整体字段搬运，882-896 行只做校验和验证，无树遍历、无分配器调用。

**(b)「`new_tree_table` 因 W1 已写全部祖先而天然正确」——原样抄两处**：

第一处（Sonnet 报告第 86 行，第 2 节 I1 的逐字段表）：
> `new_tree_table` | 指向这次 fsync 新写的树表单元；这张表里被改的那棵树（inode / extent）的条目根指针 = 这次 fsync 刚 COW 完的新子树根（因为 W1 已经写了「全部祖先」，即从叶到根整条链已经落盘），其余树的条目原样搬 | `records.rs:263` `TreeTableEntry` 只有一个 `root` 字段，没有绕开整条 COW 链的第二条路径（第 1 节）

第二处（Sonnet 报告第 142 行，第 5 节判定一览）：
> I1 | **过**（W2 施加停在换字段） | `recovery.rs:821-917` 的 `replay_journal` 对四个 `new_*` 字段只做整体搬运，无树遍历、无分配器调用；四字段逐一核过取值，`new_tree_table` 因 W1 已写全部祖先而天然正确 |

两处上下文都是在判**W2**（W1 之上再加「每次 fsync 另写一个新树表单元」）：句中「W1 已写全部祖先」说的是 W1/W2 共有的基线行为——写脏叶到该树自己根之间的整条 COW 链（「叶到根」，不含树表本身），这是正文第六节 W1 定义里「写脏数据单元、脏叶与**全部祖先**」那半句；「新写一个树表单元」是 W2 独有的、比 W1 多做的那一步（同一行紧接着的「指向这次 fsync 新写的树表单元」）。两处都没有断言「W1 写树表」——树表单元的写入被明确记在 W2 名下，「因 W1 已写全部祖先」只用来说明 W2 新写的树表单元里要填的那个根指针**已经算好、天然正确**，不需要另外一次树遍历。这与正文第六节「W1 按定义不写树表单元（树表是四样固定点之一，延到 checkpoint）」不矛盾——Sonnet 没有说 W1 写树表，只是拿 W1 的「已写祖先」这个前提去支撑「W2 新写树表单元里的值算得对」。

## 3. 云端攻方（Opus）`research/prompts/m2-s1-r2-opus-output.md`（452 行，sha256 与自报一致：现算 `5d190eda92d760f868877acdf6aa22be66ec71808f8bcd4d8b698b73152466c1`）

### 3.1 `crates/` 引用（先取 HEAD 副本 `/tmp/claude-1000/three-way-verifier/draft/*.HEAD`，再核当前工作区差异）

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `transaction.rs:1321`（`fn admission_of_one_publish(`）、`:1326-1327`（两行注释） | **✓，但有行号漂移 9 行**：HEAD 副本上逐字匹配（1321 = `fn admission_of_one_publish(`，1326-1327 = 「分配记录树第一版只有一个节点…」两行注释，与报告逐字相同）；当前工作区（实现员仍在改 `transaction.rs`，`git diff HEAD --stat` 显示改动中）里两处都后移到 1330 / 1335-1336——同一段内容，行号因这一轮结束后另一位会话的编辑整体下移 9 行，非 Opus 引错 | `git show HEAD:crates/singlefs-core/src/transaction.rs > draft/transaction.rs.HEAD`；`grep -n "admission_of_one_publish\|分配记录树第一版只有一个节点" draft/transaction.rs.HEAD` 与当前树对比 |
| `unit.rs:126`（`index_node_entry_capacity`） | ✓ 逐字匹配（此文件未被实现员触碰，HEAD = 当前） | `sed -n '126p' crates/singlefs-core/src/unit.rs` |
| `transaction.rs:1068-1071`（`PublishShape` 文档注释+结构体） | ✓ 逐字匹配 HEAD | `sed -n '1068,1073p' draft/transaction.rs.HEAD` |
| `transaction.rs:1093`（`rewritten_roles`） | ✓ 匹配 HEAD | `grep -n "fn rewritten_roles" draft/transaction.rs.HEAD` |
| `transaction.rs:1095-1106`（角色 push 逻辑） | ✓ 匹配 HEAD | `sed -n '1095,1106p' draft/transaction.rs.HEAD` |
| `transaction.rs:1108-1112`（`ExtentRoot`/`InodeLeaf`/`InodeRoot` 三角色） | ✓ 匹配 HEAD | `sed -n '1108,1112p' draft/transaction.rs.HEAD` |
| `transaction.rs:1078`（`ROW_PUBLISH`）、`:1085`（`EMPTY_PUBLISH`） | ✓ 匹配 HEAD | `sed -n '1076,1090p' draft/transaction.rs.HEAD` |
| `transaction.rs:1294`（`publish_sequence_admission`） | ✓ 匹配 HEAD | `sed -n '1292,1296p' draft/transaction.rs.HEAD` |
| `mount.rs:609`（`raise_rollback_floor`）、`:693`（`file: None,`） | ✓ 匹配（HEAD 与当前树一致） | `sed -n '605,610p;690,694p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:1108`（`mount_writable`） | ✓ 匹配 | `sed -n '1108p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:725-727`（`publish_rows_on_file_version` 头注+签名）、`:746`（`file: None,`） | ✓ 逐字匹配（头注两行与报告整句抄逐字相同） | `sed -n '725,727p;744,747p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:979`（`establish_instance`） | ✓ 匹配 | `sed -n '979p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:758`（`publish_empty_after`）、`:772`（`file: None,`）、`:923`（`warm_up_publish_txgs`） | ✓ 匹配 | `sed -n '756,759p;770,773p;923p' crates/singlefs-core/src/mount.rs` |
| `singlefs-format/src/lib.rs:185`（`ROOT_RING_REGIONS: u64 = 3`）、`:139`（`JOURNAL_RECORD_BYTES: u64 = 4096`） | ✓ 匹配 | `grep -n "^pub const JOURNAL_RECORD_BYTES\|^pub const ROOT_RING_REGIONS" crates/singlefs-format/src/lib.rs` |
| `mount.rs:803-804`（拒绝注释整句）、`:805`（`fn refuse_instance_rows_on_version_without_file`） | ✓ 逐字匹配 | `sed -n '800,807p' draft/mount.rs.HEAD` |
| `mount.rs:224`（`rebuild_previous_version`） | ✓ 匹配 | `sed -n '220,226p' draft/mount.rs.HEAD` |
| `mount.rs:1236-1238`（回退注释整句）、`:1240`（`RollbackToVersionWithoutFileUnsupported`） | ✓ 逐字匹配 | `sed -n '1234,1241p' draft/mount.rs.HEAD` |
| `mount.rs:871`（`refuse_publishes_before_acquisition_that_do_not_pass_admission`） | ✓ 匹配 | `grep -n "refuse_publishes_before_acquisition" draft/mount.rs.HEAD` |
| `mount.rs:880-884`（`shapes` 构造代码块） | ✓ 逐字匹配 | `sed -n '878,886p' draft/mount.rs.HEAD` |
| `mount.rs:779-783`（`assert_eq!` 整块） | ✓ 逐字匹配 | `sed -n '777,784p' draft/mount.rs.HEAD` |
| `mount.rs:605`（头注）、`:288,285`（`reclaim_floor` 头注） | ✓ 匹配 | `sed -n '605,610p' crates/singlefs-core/src/mount.rs` |
| `mount.rs:1185`（`mount_rollback`）、`:622`（`RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion`） | ✓ 匹配 | `grep -n "^pub fn mount_rollback\|RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion" crates/singlefs-core/src/mount.rs` |

**crates/ 计数：核了 23 处，✓ 23 处（其中 2 处——`transaction.rs:1321`、`:1326-1327`——内容一致但对当前工作区有 9 行漂移，已标注，不计 ✗），✗ 0 处。**

### 3.2 kb 引用与背景材料引用

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `23-journal的角色与格式.md:534`「任一操作的最坏 journal 占用 ≤ 环大小 / F」 | ✓ 逐字匹配（这一句是 Opus 自己指出的「引文偏差」三方对照之一，核实为真） | `sed -n '534p' .claude/kb/decisions/23-journal的角色与格式.md` |
| `.claude/kb/invariants.md:76`「环大小 ≥ F × 任一事务的最坏 journal 占用」 | ✓ 逐字匹配 | `sed -n '76p' .claude/kb/invariants.md` |
| `.claude/kb/invariants.md:1202-1208`（K9′ 复用与 I-8.4 边界的注释段） | **✗ 误写成背景材料行号**：`.claude/kb/invariants.md` 全文只有 458 行，1202-1208 不存在；`research/prompts/_m2-s1-r2-background.md` 第 **1202-1208** 行正是这一段（逐字核对，1202 起「⚠️ I-8.4（重放幂等） 的射程有一条已实测的边界…」）；该段在 `invariants.md` 里的真实行号是**第 97 行起**（同一段落一直到约 101 行） | `wc -l .claude/kb/invariants.md`；`grep -n "I-8.4（重放幂等） 的射程有一条已实测的边界" .claude/kb/invariants.md research/prompts/_m2-s1-r2-background.md` |
| `research/prompts/m2-s1-r1-opus-output.md:102`（K9′ 定义整行） | ✓ 逐字匹配 | `sed -n '102p' research/prompts/m2-s1-r1-opus-output.md` |
| `25-目标负载优先级.md:20`「一次 fsync 带 8 个叶子，落在 1 条共享脊柱上」 | ✓ 逐字匹配 | `sed -n '20p' .claude/kb/decisions/25-目标负载优先级.md` |
| `16-发布语义.md:121`（T_time/T_dirty 定案整段） | ✓ 逐字匹配 | `sed -n '121p' .claude/kb/decisions/16-发布语义.md` |
| `16-发布语义.md:130`（记录数=数据单元数） | ✓ 匹配 | `sed -n '130p' .claude/kb/decisions/16-发布语义.md` |
| `16-发布语义.md:132`（T_dirty 与 T_time 两种规则） | ✓ 逐字匹配 | `sed -n '132p' .claude/kb/decisions/16-发布语义.md` |
| `16-发布语义.md:138`「本机 2785 fsync/秒」 | ✓ 逐字匹配 | `sed -n '138p' .claude/kb/decisions/16-发布语义.md` |
| `16-发布语义.md:169`（持久顺序+系统配置槽） | ✓ 逐字匹配 | `sed -n '169p' .claude/kb/decisions/16-发布语义.md` |
| `03-空间分配.md:126`（崩溃丢掉未发布 checkpoint 整段） | ✓ 逐字匹配 | `sed -n '126p' .claude/kb/decisions/03-空间分配.md` |
| `03-空间分配.md:180`（界 3 定义整段） | ✓ 逐字匹配 | `sed -n '180p' .claude/kb/decisions/03-空间分配.md` |
| `03-空间分配.md:144`（C122 条目数上界口径） | ✓ 匹配 | `sed -n '144p' .claude/kb/decisions/03-空间分配.md` |
| E155 产物数（139776、32768、60/[5,1]、45/[4,1]、360 组、97/125@k=8、66/94@k=8、0.354/0.402/0.404、24 行 unconverged、186 行 saturated），标「整行抄自背景材料第四节」 | ✓ 全部数字逐字/逐数核实与 `research/results/e155-second-run-fsync-write-volume-2026-09-19-stage2.out` 一致（见第 5 节） | 见第 5 节命令 |

**kb/背景材料计数：核了 14 处，✓ 13 处，✗ 1 处（`invariants.md:1202-1208` 误写成背景材料行号）。**

### 3.3 模型复跑

```
$ cd /tmp/claude-1000/three-way-verifier/opus-model && sha256sum interval_accounting.py
f19cb886fd4bd6b8021dfd85802e32e7a2a614150dc452c52c8684c5630324f4  interval_accounting.py
$ nice -n 19 python3 interval_accounting.py > rerun.out 2>&1; echo $?
0
```

原样末行（`rerun.out` 第 57 行，`I3 表 2` 最后一行）：
```
  wal_full-K9′ | 64 | 812 | 402 | 480 | 882 | 拒（AllocationRecordsExceedOneNode）
```

比对：`I7 表 1`、`I7 表 3`、`I3 表 1` 三张表在报告正文里逐字复述，与复跑输出**逐字节相同**（含 `I7 表 3` 的 `读法 A 触发时刻 s: None`）；`I7 表 2`（默认环 768 MiB、F=3 时顶到 2.9415 s）与报告第 5 节引用的「2.94 s」一致；`I3 表 2`（报告没有整表贴出，只引用了 N≈55、N≈73 两个推算点）用复跑数据反推：wal_full-K9′ 的「间隔新增条目」随 N 线性（斜率 7.5/N），402+7.5N=812 ⇒ N≈54.67→55；乙-M-K9′ 斜率 5.625/N，402+5.625N=812 ⇒ N≈72.9→73，两个都与报告一致。sha256 与自报一致，**未发现任何数字被改写**。

## 4. 本地攻方（英文，抽两次）

`m2-s1-r2-local-attack-output-s1.md`、`-s2.md`；核对表 `m2-s1-r2-local-attack-translation-audit.md`；运行记录 `m2-s1-r2-local-attack-runlog.md`。

### 4.1 运行记录复核（干净判定、字词损坏闸）

```
$ nice -n 19 python3 research/scripts/corruption-check.py research/prompts/m2-s1-r2-local-attack-output-s1.md
绿 …s1.md  cjk=0 words=837 fffd=0 汉字复读=0 英文复读=0 反引号落单=0 星号落单=0 粘连=0 实词自复读=0 缩写自粘=0
$ nice -n 19 python3 research/scripts/corruption-check.py research/prompts/m2-s1-r2-local-attack-output-s2.md
绿 …s2.md  cjk=0 words=569 …
$ nice -n 19 python3 research/scripts/oov-check.py research/prompts/m2-s1-r2-local-attack-output-s1.md
绿 …s1.md  生词=15 拼接=0   生词: Falsifier dimensionless recomputation
$ nice -n 19 python3 research/scripts/oov-check.py research/prompts/m2-s1-r2-local-attack-output-s2.md
绿 …s2.md  生词=16 拼接=0   生词: Falsifier dimensionless contradicting superlinearly
```

四个数字（words=837/569、生词=15/16）与运行记录表格逐字相同。✓

### 4.2 逐字核对表（`-translation-audit.md`）核实

核对表本身列了 18 行「英文项 / 原文文件:行 / 首稿缺的 / 定稿」（A 组 9 行对应 I5，B 组 9 行对应 I6），逐行去源文件取对应行、比对核对表引的原文与「缺/加」的定性是否属实：

| 组 | 引用 | 核的结果 | 命令 |
|---|---|---|---|
| A | `16-发布语义.md:169`（FACT TABLE 1 Ruling） | ✓ 核对表说「缺 fsync 返回条件多一句…新实例双盘确认」——现查该行原文确有这半句且未见于本地腿提示英文中，缺失属实 | `sed -n '169p' .claude/kb/decisions/16-发布语义.md` |
| A | `16-发布语义.md:174`（Scope note） | ✓ 「D25 推导的两个」被改写成不点编号的说法，语义未变，核对表定性准确 | `sed -n '174p' .claude/kb/decisions/16-发布语义.md` |
| A | `_m2-s1-r2-body.md:85,86,88`（甲B/wal_full/Arm-B-M 定义） | ✓ 三行均逐字匹配（用 HEAD 副本核，见第 1 节），核对表标的「删去中间版细节」「『挂载后第一次发布』被泛化」两处缺失属实 | `git show HEAD:research/prompts/_m2-s1-r2-body.md \| sed -n '85p;86p;88p'` |
| A | `155-…md:66,70,69`（FACT TABLE 3 行 a/c/d） | ✓ 逐字匹配；「加了 first」「删验证出处说明」两处定性属实 | `sed -n '66p;69p;70p' ".claude/kb/experiments/155-每次持久化的写量三种fsync形态与反事实上界.md"` |
| A | `_m2-s1-r2-body.md:59`（Q6.5 定义，行 b 泛化） | ✓ 逐字匹配（HEAD 副本），「加了对 Arm-B-M 的推广」属实（英文自陈是推广，非逐字引用） | 同上 HEAD 副本 `sed -n '59p'` |
| A | `records/2026-09-19-里程碑二遗留收拢.md:143`（D25 用户答复原文） | ✓ 逐字匹配（「根据硬盘和电脑的性能来做」原句在），「sized to」收窄的定性属实 | `sed -n '143p' records/2026-09-19-里程碑二遗留收拢.md` |
| B | `16-journal的角色WALvs意图日志.md:217,63-71,73,78-80`（FACT TABLE 5/6/7） | ✓ 56 格峰值表（streams=2/4/8/16/32/64/128 × batch 8 列）逐格与源表相同，无一处改动；78-80 拟合误差「中位 9.6%、最大 20.5%」逐字匹配 | `sed -n '63,71p;73p;78,80p;217p' ".claude/kb/experiments/16-journal的角色WALvs意图日志.md"` |
| B | 同文件 `:256,274,268-272`（FACT TABLE 8） | ✓ 扇出公式、树高规则、五行实测表（512/1024/4096/16384/65536 对应扇出 9/7/5/4/3）逐字匹配；「加 floor()」「加恰好 1 TiB 算术」两处添加，核对表已如实单列且给出反推依据，未隐藏 | `sed -n '256p;268,274p' 同上` |
| B | `18-块里携带什么信息.md:215`、`08-核心索引结构.md:160,232`（FACT TABLE 9） | ✓ 数据单元/索引节点宽度 32768/16384、inode 树扇出 135、树表条目 200 字节，逐字匹配；「加警告『别把 135 当通用扇出』」「删打包记录单元一列」两处如实列出 | `sed -n '215p' .claude/kb/decisions/18-块里携带什么信息.md`；`sed -n '160p;232p' .claude/kb/decisions/08-核心索引结构.md` |
| B | `singlefs-format/src/lib.rs:139,148,151,155`（FACT TABLE 9 行 d） | ✓ 与 Sonnet 引用同一组常量，核对表称「已现查代码逐行核对」，复核一致 | 同第 2 节 lib.rs 命令 |

样本本身的事实表数字（Section 2 问题 4/5/6 的比值计算、问题 1 的树高判定）用的正是 FACT TABLE 6/8 的数——上表已确认这些表与源文件一致，样本据此算出的比值（batch=10 时 4.727、streams=16 峰值高于原 3.91 等）复核算术无误（135³=2,460,375、135⁴=332,150,625、2⁴⁰/32768=33,554,432，逐位验证）。

**本地腿计数：核了 18 处转述 + 4 处运行记录数字，全部 ✓，✗ 0 处。核不动：s1/s2 两次调用时的历史进程状态（`ps` 检查结果）无法回溯复核，见「没做什么」。**

**s1/s2 两次答复对 I5 给出相反的假设（s1 假设固定到达间隔 Δt>0 ⇒ 等待时间随 k 线性涨；s2 假设同时到达 Δt=0 ⇒ 等待时间恒为 L，不随 k 变），本地腿运行记录第 20 行已如实标出「不在本报告判断范围，交主 agent」——这不是核查员该判的推论分歧，原样转交。**

## 5. 总计数

| 腿 | 核了几处 | ✓ | ✗ | 核不动 |
|---|---|---|---|---|
| 云端辩方 Sonnet | 17 | 15 | 2 | 0 |
| 云端攻方 Opus（crates） | 23 | 23 | 0 | 0 |
| 云端攻方 Opus（kb/背景材料） | 14 | 13 | 1 | 0 |
| 本地攻方（转述+运行记录） | 22 | 22 | 0 | 1（历史进程状态） |
| **合计** | **76** | **73** | **3** | **1** |

三处 ✗：
1. Sonnet `recovery.rs:922` 引 `TREE_KIND_INODE => Some(8)`，实际在第 923 行（922 是 `TREE_KIND_EXTENT => Some(24)`）——单纯行号错一行，非行号漂移，出现在自我留白段。
2. Sonnet `23-journal的角色与格式.md:669`/`:671` 引已定项 20 标题与引句，误用了 `research/prompts/_m2-s1-r2-background.md` 第 669/673 行的行号；kb 文件本身只 614 行，669/671 不存在；同一段落在 kb 文件里的真实行号是第 468/472 行。
3. Opus `.claude/kb/invariants.md:1202-1208` 引 K9′/I-8.4 边界注释段，误用了 `research/prompts/_m2-s1-r2-background.md` 第 1202-1208 行的行号；`invariants.md` 全文只 458 行，1202-1208 不存在；同一段落在 kb 文件里的真实行号是第 97 行起。

三处 ✗ 里的引文**内容本身**（Sonnet 的四字段搬运论证、Opus 的 K9′/I-8.4 边界论证）在真实行号处均逐字核实无误——错的只是行号，判据依赖的那句话确实存在、确实是那句话。这一点不构成对推论正确性的判断，只说明「行号能核对上」这条证据链在这三处断了，主 agent 复核时需要自己去真实行号处再核一遍原文，不能信这三处报告写的行号。

## 6. 没做什么

- 不判 I1–I8 任何一格该不该过、W2/L2 构不构造得出、I3/I4/I7 打不打中——那是主 agent 的推论判断，不归核查员。
- 不判 s1/s2 两份本地样本对 I5「等待时间随 k 线性涨还是恒定」给出的两种假设哪个对；只核实两次调用都干净（干净判定见 4.1）、核对表本身诚实（见 4.2）。
- `.claude/kb/checks-owed.md` 快照时点的原文无法从现有 git 版本链倒推出精确原样（HEAD 及其三个父提交的哈希都与快照不符，改动经过比「改名+去重」更复杂，倒推不到位）；因三条腿均未引用此文件，未核，未记 ✗，按「分不清」单列于第 1 节。
- 未核实本地腿运行记录里「跑前 `ps` 检查、仅另一会话在跑 `gate.sh --staged`」这句话——历史进程状态事后无法复现，核不动。
- 未复核 Opus 报告第六、七节「没打中的形状」与「这条腿自己的限度」里陈述的推理是否成立（例如 λ=2785 敏感性、ρ 外推的合理性）——这些是论证本身，不是引用或产物，不归本轮核查范围。
- 未编译 Rust、未跑任何门禁阶段（`.claude/gate.d/stage-owners.tsv` 里没有 `three-way-verifier` 这一行）。
- 未改动三条腿任何一份产出、未改 `crates/` 或 kb 任何一个字节；`crates/{recovery,transaction,mount}.rs`、`singlefs-format/src/lib.rs` 当前被另一位实现员编辑中，全程只读不写，核对时按需拷贝 `git show HEAD:...` 到 `/tmp/claude-1000/three-way-verifier/draft/` 复核，不动工作区。
- 未做 git 写操作。
