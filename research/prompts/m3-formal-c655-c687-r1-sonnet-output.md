# m3-formal-c655-c687-r1 云端正推（Sonnet）报告

轮：m3-formal-c655-c687-r1（第一轮）。腿：云端正推。批次：records/2026-10-01-里程碑三形式化接入.md#26。
代码引文读冻结副本 `/tmp/claude-1000/m3-formal-c655-c687-r1/tree/`，下文 `crates/…` 路径与行号都是那棵树里的；kb 行号取 kb 文件自己的行号（逐条 `grep -nF` 过）。
探针与原样日志入库在 `research/prompts/m3-formal-c655-c687-r1-sonnet-model/`；拷出的工作副本在 `/tmp/claude-1000/m3-formal-c655-c687-r1-sonnet/work/`（交回前已删，见末尾「没做什么」）。
「量」都是 harness 档小池探针（内存盘、单元区 240、256 或 384 槽的两块盘）；checker 档、GPU 判器没跑，凡涉及它们的判定都标「推的」。

## 0 速览（逐格判定表在末尾）

- Y1：十样字段分四类。必须相同（且是定值 0）：算法、nonce、MAC。按确定关系判：系统配置槽世代号、journal_tail（两条关系式，18 个盘行量到全成立）、`allocation_record_tree_root`（空性关系）、`mapping_root`（空性关系）。并入已有一行：墓碑 / 谱系 / deleted_inodes 三根指针并进「树表里 inode 树、extent 树的根指针」那一行（同一条续做规则），不并进分配记录。「两根指针并进分配记录、三根指针必须相同」那个辩方推法，前半对一半（指针字节确实不能比，但空性关系还要判），后半错（续做会改 deleted_inodes 根，量到）。
- Y2：题面十样之外：根记录 15 个字段里表里表外都没提的只有 magic 与自证校验和两个结构性字段；系统配置 46 行里表外没提的 5 行（节点大小、T_time、T_dirty、三水位、整槽校验和），另有 26 行条款说必须相同、判官看不见；树表条目九条根指针没比、其余字段也没比。字段清单今天没有一处机器可读的权威定义：`singlefs-format` 只有总宽，逐字段偏移散在 core、checker、GPU 三处与 kb 文本里。
- Y3：六格里 a、c、e（树表 / extent 树 / 分配记录树读不出）建议报「部分没判」（用户 2026-10-04 给 I-9.28 立的第四种判定），不判红：它们在合法写侧只在读故障域出现，在坏镜像上已被 I-2.1、I-4.8、I-7.4 判红；b（种类 1 不止一条）与 e 的「种类 3 条数不对」今天没有别的不变量判红，病根在树表结构，建议判红且归 I-9.16 加一句而不是归 I-5.5；d（某块盘上没有数据单元）在今天的两盘镜像写侧不出现、在多盘布局下合法，建议该盘跳过（对象为空），不判红；f（开放段回落误排）今天够不着的论证见 Y3。
- 判别力自证与坏镜像见各格；其中 Y3 的坏镜像在拷出的副本上跑出来的结论见 Y3 各格。

## Y1　表外字段各归哪一类

### Y1.0 先交代我读到的代码事实

挂载做的每一个动作怎么写这些字段（读码，行号在冻结副本里）：

- 取号：每块盘写一次系统配置槽，世代号 = 这块盘两槽里自证过的最大世代号 + 1（`crates/singlefs-core/src/transaction.rs:405-410` 一段，`+ 1;` 在第 410 行），tail 带取号那一刻读到的见证值、实例代号写新号。
- 每次发布末尾轮换一次系统配置槽：`CommitStep::RotateSystemConfigurationSlots` 逐盘调同一个 `write_system_configuration_slot`，所以每次发布每块盘世代号 +1（同一函数）；tail 写这次发布最后一条记录的计数器（`crates/singlefs-core/src/transaction.rs:1978`「journal_tail: last_counter_of_this_publish,」、`crates/singlefs-core/src/transaction.rs:1413`「journal_tail: plan.counter,」）。
- 写行那次发布与暖机：上一版带文件时走带文件的发布（不放文件，固定点单元全重写，`crates/singlefs-core/src/mount.rs:3350-3387`）；上一版树表 0 条时：行为空走零单元发布（照抄上一版的两根指针，`crates/singlefs-core/src/transaction.rs:1375`、`crates/singlefs-core/src/transaction.rs:1377`），行非空只重写实例表与分配记录树（`crates/singlefs-core/src/transaction.rs:1805` 新建分配记录树、`crates/singlefs-core/src/transaction.rs:1894` 写进根）。
- 行是否为空：`crates/singlefs-core/src/mount.rs:3719`「let first_row_instance = previous_row.instance.0.max(1);」起的区间 [max(上一实例, 1), 要取的号)，所以只有新实例号为 1（第一次可写挂载）时为空。
- 续做：`crates/singlefs-core/src/mount.rs:4947` 的 `continue_the_deletion_queue_until_it_is_empty` 一批一批发 unlink 的后续批，每批都是带文件的发布，会改 inode、extent、`deleted_inodes`、墓碑树的根。

### Y1.1 怎么量的（可复跑）

在冻结副本的拷贝上把 `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 整份拷成 `y1_probe.rs`，末尾追加探针（只读：造崩溃镜像、挂载、再用 harness 自己的 `read_recovery_fields` 读两次、逐项打印）。命令（工作目录 `/tmp/claude-1000/m3-formal-c655-c687-r1-sonnet/work`，`CARGO_TARGET_DIR` 指到草稿目录，`ps` 看到别的 cargo 与 e170 复跑在跑、无性能测量，加 `nice -n 19` 照常跑）：

```
nice -n 19 bash scripts/capped.sh 10 cargo test --offline -p singlefs-harness --test y1_probe --no-run
RUN_WITH_MEMORY_CAP_PARALLEL_REASON="三方正推腿与同会话别的腿并行各跑各的小池探针" nice -n 19 bash scripts/run-with-memory-cap.sh 6G bash scripts/capped.sh 10 <测试二进制> y1_probe_ --nocapture --test-threads=1
```

第一次不带理由那一次退出码 252：内存包装报「这个会话已经有一条日常的在跑，同一个会话同时只许一条；命令没跑」（包装自己的结局，不是结果）；第二次起带包装自己写明的并行理由（本会话里 Opus 腿与别的任务在同时跑）跑过，退出码 0，输出在 `…-sonnet-model/y1-probe-run2.log`（六个场景）、`y1-probe-run3.log`（世代号与 tail 的关系）、`y1-probe-run4.log`（有树而文件集合为空的一版）。
六个场景：mkfs 之后（树表 0 条、第一次挂载）、mkfs 之后第二次挂载（树表 0 条、行非空）、第一个文件之后、正常卸载之后、挂载推抬 F 的小盘、崩在 unlink 意图之后（挂载续做）；另加 unlink 走完之后的有树而文件集合为空的一版。

### Y1.2 逐字段

**① `mapping_root`（根记录偏移 256，86 字节）**

- 谁写：带文件的发布每次重建中央映射树并把新根写进根记录（`crates/singlefs-core/src/transaction.rs:8176`「let mapping_pointer = mapping_built.version.root_pointer();」、`crates/singlefs-core/src/transaction.rs:8606`「mapping_root: mapping_pointer,」）；零单元发布与树表 0 条上的写行发布照抄上一版（`crates/singlefs-core/src/transaction.rs:1375`、`crates/singlefs-core/src/transaction.rs:1893`「mapping_root: previous_root.mapping_root,」）。
- 量到：有文件 / 有树的 5 个场景里 `mapping_root_equal=false`（R1 与 R2 的指针字节不同）；树表 0 条的 2 个场景 `mapping_root_equal=true` 且 `mapping_root_zero=true`。（`y1-probe-run2.log`、`y1-probe-run4.log`）
- 判定：**不是「必须相同」**。归「模掉指针字节、改判两句」：甲，R2 的 `mapping_root` 全零 ⇔ R1 的全零（树表 0 条 ⇔ 全零，挂载不发树也不撤树；量到有树而无文件的一版两侧都非零）；乙，池级 checker 对 R2 镜像判绿（映射树的内容归 I-1.11 等判，并进「分配记录」那一行：`.claude/kb/decisions/13-验证路线.md:51`「不比整表：POSIX 内容相同，且池级 checker 对 R2 的镜像判绿」）。
- 与已定条款的关系：`.claude/kb/layout/01-first-txn.md:379` 只写「mkfs 与暖机全 0，发布后指 50251」，没有说发布之后每一版换不换；实现是每版都换。
- 推翻它的现象：有一个合法历史上，带文件的 R1 经只挂载后 `mapping_root` 字节不变，或树表 0 条的 R1 经挂载后映射根变成非零。
- 不分三种动作：B（挂载加一次发布）比的是「发布之前那一版」，同 A；C（二次崩溃）落回 v1 时字节相同，落到挂载写的版本时同 A。

**② `allocation_record_tree_root`（偏移 342，86 字节）**

- 谁写：带文件的一版恒写全零（`crates/singlefs-core/src/transaction.rs:8609`「allocation_record_tree_root: NodePointer::empty_root(),」，注释说两处都写就成了同一个量的两份手抄）；零单元发布照抄（`crates/singlefs-core/src/transaction.rs:1377`）；树表 0 条、行非空的写行发布写新根（`crates/singlefs-core/src/transaction.rs:1894`）。layout 里的原话：「mkfs、暖机与新池新建文件全 0；只有树表 0 条、写过行的那一版写非零，指写行那次发布写的那一片分配记录节点」（`.claude/kb/layout/01-first-txn.md:380`）。
- 量到：mkfs 之后第一次挂载 `alloc_root_equal=true`（全零）；mkfs 之后第二次挂载 `alloc_root_equal=false` 且 `alloc_root_zero=false`；带文件的 5 个场景全零不变。
- 判定：**按一条确定关系判**。R1 的树表非空（带树）⇒ R2 的 `allocation_record_tree_root` 必须全零；R1 的树表 0 条 ⇒ R2 非零 ⇔ 这次挂载取到的实例代号 ≥ 2（行非空），否则照抄 R1 的（全零）。「实例代号」在判官里已有（`new_instance`，`crates/singlefs-harness/src/mount_transparency.rs:1053`起），不用新输入。再加「池级 checker 对 R2 镜像判绿」（并进分配记录那一行，指针字节本身不比）。
- 这比辩方推的「并进分配记录那一行」多要求了空性这一句：只靠 checker 判绿，一个把非零指针写到带文件版本上的挂载 bug 只会引起走读多走一棵树，不一定红（`crates/singlefs-checker/src/walk.rs:1330`「if !parse_node_pointer(&record[342..428]).all_zero {」只是不为零就多走，没有判带文件的一版必须零）。
- 推翻它的现象：合法历史上 R1 树表 0 条、新实例号 ≥ 2，而 R2 的分配记录树根全零；或 R1 带树而 R2 非零。

**③ 墓碑、谱系、deleted_inodes 三根指针**

- 先纠一个归属：这三根指针不在根记录里，是树表条目里的根指针（树 ID 65544、65545、65546，种类 9、10、11）；根记录只有四条指针（树表、实例表、映射根、分配记录树根）。layout 的原文：「(65544, `deleted_inodes`, 9, 根指针零, 头 ID 65537)」（`.claude/kb/layout/01-first-txn.md:387`）。题面把它们列在「根记录的」后面，C655 欠账行与 D13 行 57 同样这么写，归属要改。
- 谁写：每次带文件的发布按 `deletion_tree_units` 写这三条的根（`crates/singlefs-core/src/transaction.rs:8236` 起 `TREE_KIND_DELETED_INODES`、之后墓碑、谱系各一条），没重写的照抄上一版指针：`crates/singlefs-core/src/transaction.rs:7551`「DeletedInodesRootOrigin::CarriedFromThePreviousVersion => Some(DeletionTreeNodeUnit {」，墓碑树、谱系树同理（`crates/singlefs-core/src/transaction.rs:7601-7602`、`crates/singlefs-core/src/transaction.rs:7632-7633`）。重写只发生在这次发布碰删除树时，挂载里就是续做。
- 读它的一侧：判官读树表只取两种种类，其余 `continue`：`crates/singlefs-core/src/recovery.rs:3214`「| TREE_KIND_LOGGED_OPERATIONS => continue,」，两根指针存在 `UserVisibleTreeRootPointers` 里（`crates/singlefs-harness/src/mount_transparency.rs:586` 起的取法也只认 `TREE_KIND_INODE`、`TREE_KIND_EXTENT`）。所以这三根今天一样没比，换种说法：表外有九条树表条目的根指针没比（种类 3、4、6、7、8、9、10、11、12）。
- 量到：没续做的场景里三根都 `root_equal=true`（`y1-probe-run2.log` 里 after_first_file、left_by_unmount、raise_floor 三段；有树而无文件的那一版里墓碑根非零且相等）；崩在 unlink 意图之后重挂的场景里 `kind=9`（deleted_inodes）根从非零变全零：`[cmp] entry kind=9 tree=65544 root_equal=false root_zero_before=false root_zero_after=true …`（`y1-probe-run2.log`），同一行的 kind=1、kind=2 也变。
- 判定：**并进「树表里 inode 树、extent 树的根指针」那一行，用同一条规则**：相同；挂载续做过时，等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的根指针。原行：「相同；挂载续做过时，按续做之后那一版判：等于挂载交回的续做各步里 txg 不大于 R2 那一版的最近一步的两根指针；树表单元自己的落点不比」（`.claude/kb/decisions/13-验证路线.md:49`）。改法写成对实现的改动：`user_visible_tree_root_pointers_of_a_published_version`（`crates/singlefs-harness/src/mount_transparency.rs:580`）与 `recovery::user_visible_tree_root_pointers`（`crates/singlefs-core/src/recovery.rs:3189`）把取的种类从 {1, 2} 扩到 {1, 2, 9, 10, 11}；续做各步交回的 `tree_table_entries` 本来就带全部条目，不要新输入。「三根指针必须相同」那一推法被量到的 kind=9 变化推翻。
- 种类 6、7、8、12（livelist、稀疏旁表、deadlist、logged_ops）：每次发布都写全零（`crates/singlefs-core/src/transaction.rs:8214`、`8220`、`8226`、`8266` 起的四条 `table_entry` 都给 `NodePointer::empty_root()`），量到全零不变。判：必须相同且全零（第一版没有它们的内容）；今天表里没写，没比。
- 种类 3、4（分配记录树、记账树）：带文件的发布每版重写这两棵（量到 `kind=3`、`kind=4` 的 `root_equal=false`），归「分配记录」一行：不比，池级 checker 对 R2 判绿。
- 推翻它的现象：续做过的场景里 kind=9 / 10 / 11 的 R2 根指针既不等于 R1 的、也不等于续做某一步的。

**④ 算法类型 1 + nonce 12 + MAC 16（根记录偏移 428–456）**

- 谁写：只有一处，`crates/singlefs-core/src/root_record.rs:111`「writer.put_u8(0); // 算法类型：未加密」与 `crates/singlefs-core/src/root_record.rs:112`「writer.skip(12 + 16); // nonce、MAC：第一版留位全 0」，全零；根记录的所有写路径都经 `RootRecord::to_slot`。
- 谁读：没有人。`RootRecord::parse_slot` 读到分配记录树根指针就停（`crates/singlefs-core/src/root_record.rs:154-157` 的断言校验位置到指针末尾为止），`RootRecord` 结构体里没有这三个字段；池级 checker 的 `RootView` 只取四个字段加整条 457 字节记录（`crates/singlefs-checker/src/lib.rs:318`「pub record_bytes: Vec<u8>,」，注释说「三个区域比对用」），`grep -nE '428|\[429|\[441' crates/singlefs-checker/src/*.rs` 里与根记录有关的命中只有 342..428（分配记录树根）这一处边界，没有任何判 428..457 的行（本腿现跑，原样输出在下面）。
- 判定：**必须相同，而且是定值 0**（layout 三行「已定（第一版留位）」：`.claude/kb/layout/01-first-txn.md:381`「| 算法类型 | 1 | 0（未加密） |」、`.claude/kb/layout/01-first-txn.md:382`、`.claude/kb/layout/01-first-txn.md:383`）。判官今天读不到：`RootRecord` 里没有这三样，harness 的两次只读恢复也不会读。要做得到，判官得读根槽原字节 [428, 457)，两版（R1 与 R2 各自比的那一版）都要全零。只比「R2 等于 R1」不够：R1 若是别家写的非零，R2 由挂载写成全零，字节不同却合法，所以「必须相同」写成「各自等于零」才不会误红。
- 另一条路（不在这一题的射程，列出来给主 agent）：把这三样读进 `RootRecord`，非零判该根损坏，同指针头 MAC / nonce 的现有读法（`crates/singlefs-core/src/pointer.rs:105-122` 那组 `PointerHeadFieldOutsideTheFirstVersion`）；那是新增读侧条款，要另走三方。
- 不分三种动作：三种动作下都不变。
- 推翻它的现象：合法的第一版写侧在这三处写过非零（`grep` 到 `to_slot` 之外对 428 之后字节的写），或 D9 的第一版留位条款已改成「加密关时也可以非零」。
- 本腿现跑的 grep 与它的原样输出（工作目录是冻结副本根 `/tmp/claude-1000/m3-formal-c655-c687-r1/tree/`）：

```
$ grep -nE '428|\[429|\[441|\[428' crates/singlefs-checker/src/*.rs
crates/singlefs-checker/src/walk.rs:1330:        if !parse_node_pointer(&record[342..428]).all_zero {
crates/singlefs-checker/src/walk.rs:1331:            self.walk_allocation_record_tree(&record[342..428], 0, "树表 0 条那一版的分配记录树");
crates/singlefs-checker/src/walk.rs:3746:    let allocation_record_tree_root = parse_node_pointer(&record[342..428]);
crates/singlefs-checker/src/walk.rs:4905:        allocation_node_pointers.push(parse_node_pointer(&record[342..428]));
crates/singlefs-checker/src/walk.rs:7010:const ROOT_RECORD_ALLOCATION_RECORD_TREE_ROOT: std::ops::Range<usize> = 342..428;
crates/singlefs-checker/src/walk.rs:7201:/// [180..188) = [130..138)、[188..274) = [170..256)、[274..360) = [342..428)、[360..364) = flags [20..24)
```

  （7201 那一行是 I-8.10 比新根段与根的偏移对照，里面的 [342..428) 仍是分配记录树根，没有 428 之后的。）

**⑤ 系统配置槽世代号（偏移 147，8 字节）**

- 谁写：每次系统配置写，世代号 = 这块盘两槽自证最大世代号 + 1（`crates/singlefs-core/src/transaction.rs:405-410`，`crates/singlefs-core/src/system_configuration.rs:459`「writer.put_u64(self.quantities.slot_generation);」）。取号一次、每次发布末尾一次、抬 F 先写系统配置一次，所以挂载一定使它变。kb 自己也这么说：「槽世代号每写一次 +1、journal tail 每次发布都涨」（`.claude/kb/decisions/22-单元原子性怎么合成.md:234`）。**不是「必须相同」。**
- 关系式（A：只挂载；B：挂载加一次发布）：对池里每块盘 d，R2 里这块盘两槽自证的最大世代号 = R1 里这块盘两槽自证的最大世代号 + 挂载（B 加上那次发布）的写表里写到盘 d 的系统配置槽写的条数。判官今天已经拿写表（`RetainedWrite` 带 `device`、`kind`，`StepKind::SystemConfigurationSlot`）来定抬 F 的界，不是新输入。
- 量到：9 个场景（含 +一次发布的变体）、18 个盘行，`gen_R2==gen_R1+n: true` 18 行全真（`y1-probe-run3.log`，例：`[raise_floor] dev=0 gen_R1=20 gen_R2=27 sysconf_writes_to_dev=7 => gen_R2==gen_R1+n: true`）。
- C（二次崩溃）：R2 的盘 d 最大世代号落在 [R1 的、R1 的 + 挂载写到盘 d 的系统配置写条数] 区间里（持久子集任意，只能判区间，推的：二次崩溃的持久子集语义这一腿没量）；槽轮换的一致由 I-7.16 判（`.claude/kb/invariants.md` 的 I-7.16 一行），不在这里重复。
- 推翻它的现象：任一场景里等式差一（比如暖机条数变化后系统配置写条数与发布数不同步）；或某个合法路径写系统配置而不进写表。

**⑥ journal_tail（偏移 469，8 字节）**

- 谁写：每次发布末尾轮换那一写带这次发布最后一条记录的计数器（`crates/singlefs-core/src/transaction.rs:8677`「journal_tail: last_counter_of_this_publish,」是带文件那一路；零单元发布 `crates/singlefs-core/src/transaction.rs:1413`「journal_tail: plan.counter,」）；取号那一写带见证值（`crates/singlefs-core/src/transaction.rs:482-483` 的注释：回卷写带的是取号那一写的同一个见证值）。读的一侧：`crates/singlefs-core/src/system_configuration.rs:530`「writer.put_u64(self.quantities.journal_tail);」写、`crates/singlefs-core/src/system_configuration.rs:604-606` 读。
- **不是「必须相同」**：kb 自己的话在 ⑤ 引过（tail 每次发布都涨）。
- 关系式（A、B）：R2 里每块盘两槽自证的最大 tail = 这块盘 journal 环里、实例代号等于 R2 系统配置 journal_instance 的记录的最大计数器。判官手里已有 `scan_journal` 的整张记录表（`crates/singlefs-harness/src/mount_transparency.rs:171` 读成 `records`），不是新输入。另加单调：R2 每盘 tail ≥ R1 同盘 tail。
- 量到：同 ⑤ 那 18 个盘行，`tail_R2==that: true` 18 行全真（`y1-probe-run3.log`，例：`[after_first_file+publish] dev=1 gen_R1=5 gen_R2=9 sysconf_writes_to_dev=4 => gen_R2==gen_R1+n: true | tail_R1=3 tail_R2=6 max_counter_of_instance(2)=Some(6) => tail_R2==that: true`）。
- C：只取单调与上界（R2 每盘 tail ≥ R1 同盘 tail；R2 环里有新实例的记录时 tail ≤ 其中最大计数器；只落了取号那一写时 tail = 见证值，新实例在环里一条记录也没有，那一格上界不判），推的。
- 推翻它的现象：一个合法历史上某盘 tail 小于这盘环里新实例最大计数器、而发布已经完整返回（轮换那一写漏写或漏轮换）；或 R2 tail 小于 R1 的。

**⑦ 题面没点名、但同属「系统运行量」、实现里每次发布都动的两样**

- 系统配置里的 `journal_instance`（偏移 477）：取号起每一写都带新实例号（量到 R2 四个槽都是新号，`y1-probe-run2.log` 各段 `[R2] sysconf` 的第四列）。判：A、B 下等于判官已算出的 `new_instance`，并进「实例代号」那一行。
- 系统配置里的 F（偏移 481）：已在「F 与 F 生效值」那一行里（生效值 = 根上与系统配置里读得出的最大值，`crates/singlefs-harness/src/mount_transparency.rs:153`起），不另归类。

### Y1.3 三种动作下的总表

| 字段 | 只挂载 / 挂载加一次发布（R2 对 R1） | 挂载途中二次崩溃 |
|---|---|---|
| `mapping_root` | 全零 ⇔ 全零；池级 checker 对 R2 判绿；指针字节不比 | 落回 v1：字节相同；落到挂载写的版本：同左 |
| `allocation_record_tree_root` | R1 带树 ⇒ R2 全零；R1 树表 0 条 ⇒ R2 非零 ⇔ 新实例号 ≥ 2；池级 checker 判绿；字节不比 | 同左，按 R2 落到的版本取 |
| 墓碑 / 谱系 / `deleted_inodes` 根（树表条目 9、10、11，不是根记录字段） | 并进两根指针那一行：相同；续做过则等于续做最近一步的 | 同左 |
| 树表条目 6、7、8、12 的根 | 必须相同且全零 | 同左 |
| 算法、nonce、MAC | 根槽 [428, 457) 两版都全零 | 同左 |
| 系统配置槽世代号 | 每盘最大世代号 = R1 的 + 写到这盘的系统配置写条数 | 区间 [R1 的, R1 的 + 写条数]（推的） |
| journal_tail | 每盘最大 tail = 环里新实例最大计数器；单调 | 单调 + 上界（推的） |

### Y1.4 对「第三轮辩方推的归法」的评价

C655 欠账行的推法：「第三轮辩方推：两根指针并进分配记录那一行，三根指针与算法三样必须相同，推的」。逐项：

- 两根指针并进分配记录那一行：对，指针字节不能比，量到 `mapping_root` 每个带文件场景都变；但「分配记录那一行」只写了池级 checker 判绿，漏掉空性关系，我多要求一句（见 ① ②）。
- 三根指针必须相同：错，续做会改 deleted_inodes 根（量到 kind=9 非零变全零）；它们该并进「两棵树的根指针」那一行（见 ③）。
- 算法、nonce、MAC 必须相同：对，但要补「定值 0、判官读根槽原字节」（见 ④），否则判官看不见这三样、「必须相同」是空话。

### Y1.5 判别力自证怎么造（对应 C655 还清条件的前半「判官」）

- 取号、写行、暖机、续做、抬 F 里哪一步写这些字段，都在上面逐字段列了行号。把判官补全之后，每一格各造一个「改坏 R2 的那一样」的用例，形态同现有 `crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs` 里每样一阳一阴的写法（`compared_version_mut` 改 R2 的字段再判，该文件 459 行）：
  - 世代号：把 R2 某盘最大世代号减 1 ⇒ 红，别的仍绿；
  - tail：把 R2 的 tail 改成 R1 的值 ⇒ 红；
  - 分配记录树根：带文件的 R2 写成非零 ⇒ 红；R1 树表 0 条且新实例号 ≥ 2 的 R2 改成全零 ⇒ 红；
  - 三根指针：用续做场景，把 R2 的 deleted_inodes 根改回 R1 的（续做前的）⇒ 红；
  - 算法 / nonce / MAC：根槽 428 之后任一字节改 1 再重封校验和 ⇒ 红。
- 这些我只写了形态，没有写进代码跑（判官还没改，红是对着我设想的判官说的）：标「推的」。

## Y2　字段清单的完整性与对账判官

### Y2.1 根记录：15 个字段，权威清单是 D22 已定项 7 的字段表

字段表在 `.claude/kb/decisions/22-单元原子性怎么合成.md` 已定项 7，行序与偏移那一句：「行序即盘上顺序，偏移按行累加：magic 0 / fsid 4 / flags 20 / 实例代号 24 / checkpoint_txg 28 / 树表单元指针 36 / 树 ID 水位 122 / 回退下界 F 130 / 自证校验和 138 / 实例表单元指针 170 / 中央映射树根指针 256 / 分配记录树根指针 342 / 算法类型 428 / nonce 429 / MAC 441。」（`.claude/kb/decisions/22-单元原子性怎么合成.md:151`），合计「合计 **457 字节**，512 槽内余量 55」（`.claude/kb/decisions/22-单元原子性怎么合成.md:154`）。15 行，对 D13 逐字段表与题面逐个过：

| 根记录字段 | D13 逐字段表里 | 题面（C655） | 落点 |
|---|---|---|---|
| magic 0（4） | 没有 | 没有 | 结构性：`parse_slot` 拒不符的（`crates/singlefs-core/src/root_record.rs:127`「|| bytes[..4] != ROOT_MAGIC」），判官不用比；清单里要有一行标「模掉（结构）」 |
| fsid 4（16） | 「池身份」那一行（经系统配置的 fsid 比） | 没有 | `parse_slot` 要求等于池 fsid，R1、R2 的根 fsid 恒等于池 fsid，不独立 |
| flags 20（4） | 「卸载记号」一行管第 0 位；其余位表里没说 | 没有 | 其余位非零 `parse_slot` 拒收（`crates/singlefs-core/src/root_record.rs:140`「if flags & !ROOT_RECORD_FLAG_UNMOUNT_MARKER != 0 {」），结构性 |
| 实例代号 24、checkpoint_txg 28、树 ID 水位 122、回退下界 F 130 | 各有一行 | — | 已归类 |
| 树表单元指针 36（86） | 「树表单元自己的落点不比」（括注里） | — | 已归类（不比字节、要读得出） |
| 自证校验和 138（32） | 没有 | 没有 | 派生字段，`parse_slot` 要求过；清单里标「模掉（派生）」 |
| 实例表单元指针 170（86） | 「实例表行」一行比行的内容，指针字节没说 | — | 同树表指针：不比字节 |
| 映射根 256、分配记录树根 342、算法 428、nonce 429、MAC 441 | 没有 | 有 | Y1 ①②④ |

根记录这一边，题面十样之外「表里表外都没提」的只有 magic 与自证校验和两个结构性字段，没有漏掉的有语义的字段。⚠️ 题面把墓碑 / 谱系 / deleted_inodes 三根指针列在根记录名下是归属错：它们是树表条目（Y1 ③），根记录没有这三个字段。

另有一层：四条指针各 86 字节里头部 50 字节（MAC 16、nonce 12、算法类型 1、压缩码、压后长度、extent 偏移……）在内存里没有对应字段（`crates/singlefs-core/src/pointer.rs:298`起 `NodePointer` 只有 `head`、`locations`、`instance`、`birth_sequence`，`crates/singlefs-core/src/pointer.rs:100-103` 的 `PointerHead` 只有出生树与出生 txg），`RootRecord` 的 `==` 看不见它们；它们归池级 checker 对走到的指针判（`crates/singlefs-checker/src/image.rs:1284` 的 `judge_pointer_mac_and_nonce_are_zero`），不归挂载透明。

### Y2.2 系统配置：46 行，三类口径

字段表在 `.claude/kb/layout/01-first-txn.md` 一「系统配置字段表」，共 46 行（`awk 'NR>=109 && NR<=154' … | wc -l` = 46，本腿现跑），偏移累加的那一句在 `.claude/kb/layout/01-first-txn.md:158`「**系统配置字段表的偏移**：行序即盘上顺序」，合计 489。D22 已定项 26 把它们分四档：不可变 389 / 可变 4 / 运行配置 36 / 运行量 60（`.claude/kb/decisions/22-单元原子性怎么合成.md:229`、`.claude/kb/decisions/22-单元原子性怎么合成.md:231`），core 里 `to_slot` 每段记在一档上、收尾断言四档字节数等于 389 / 4 / 36 / 60（`crates/singlefs-core/src/system_configuration.rs:395-425` 的 `finish`）。

我按三个口径把 46 行各过一遍（行号取自上面那张表，从 1 数）：

- **A 存进内存**（`SystemConfiguration`；其中不可变那 10 行判官比，实例代号、F 两行表里有行，世代号、tail 两行没人比）：不可变档里 fsid、devs、physical_block_size、journal 环起点、环长、S、根环起点、逐区域设备、io_min、槽距共 10 行，加本盘设备号（表里明写除外）；运行量里世代号、tail、实例代号、F。合计 15 行读回来存进 `SystemConfiguration`（`crates/singlefs-core/src/system_configuration.rs:612-640`）。其中世代号、tail 是 C655 题面的；实例代号、F 表里有行。
- **B 读了做准入判断、不存**：magic、格式版本、feature bits、整槽校验和、加密类型、在飞记录数上限、最坏占用、安全系数 F、单元区起始槽号共 9 行（`crates/singlefs-core/src/recovery.rs:754-830` 一段 `system_configuration_values_this_reader_accepts`）。池里能挂载的槽它们都落在读者收的范围里，判官不用比，但「必须相同」的口径要把它们算进清单。
- **C core 完全不读**：写入者身份、校验和算法标识、MAC、nonce 水位、KDF、MAC 长度声明、主密钥槽、节点大小、单元大小、落点粒度、位置条目宽度、N、journal 最大记录字节、R、P、chunk、w_max、g、映射来源、T_time、T_dirty、三水位共 22 行。

按 D13 表的字面「池身份与系统配置的不可变配置（本盘设备号除外） | 必须相同」（`.claude/kb/decisions/13-验证路线.md:41`）：不可变档 37 行（`crates/singlefs-core/src/system_configuration.rs:224-228` 的注释：自举头 8 行 + 加密预留 6 行 + 几何段除节点大小外 23 行）减本盘设备号，都归「必须相同」。判官 `immutable_configuration_is_the_same_except_this_device`（`crates/singlefs-harness/src/mount_transparency.rs:878`）只比内存里那 10 行的值，**26 行（B 里 8 行加 C 里 18 行）条款说必须相同、判官看不见**：判官自己的注释说「新加一个字段而这里没比，编译不过」，那只在加进 `SystemImmutableConfiguration` 结构体时成立，不读回来的字段加进格式，编译照过。

表里表外都没提、也不在不可变档里的 5 行：节点大小（可变档，D22 已定项 26：改它要重建索引，第一版没有这条路 ⇒ 判必须相同）；T_time、T_dirty、三水位（运行配置档：「每次挂载可重设，而且要回写盘上」，`.claude/kb/decisions/22-单元原子性怎么合成.md:231`；今天没有挂载选项通道，`to_slot` 总写内置默认，`crates/singlefs-core/src/system_configuration.rs:631`「// 可调值那 36 字节不读回来：挂载选项通道不存在」 ⇒ 判「模掉 R1、R2 = 内置默认」，这样 R1 若是别的写者写的非默认值，挂载重设不误红）；整槽校验和（模掉，派生，自证过即可）。

### Y2.3 树表条目

树表条目 200 字节（树 ID 8、条目长度 2、种类 2、flags 2、根指针 86、`previous_snapshot_txg` 8、诞生 txg 8、头 ID 8、预留 76；`crates/singlefs-core/src/records.rs:403-420` 的 `to_bytes`）。D13 表只有 inode、extent 两条的根指针一行。我量到（`y1-probe-run2.log`）全部 11 条里：种类 1、2 不续做时相等；3、4 每版重写；6、7、8、12 恒全零；9、10、11 续做时会变；树 ID、种类、诞生 txg、头 ID 在所有场景里相等（`tree_birth_txg: previous.tree_birth_txg()`，`crates/singlefs-core/src/mount.rs:3367`）。分类见 Y1 ③ 与表：根指针之外的字段「必须相同」，当前判官没比。

### Y2.4 对账判官的「字段清单」从哪取

四个候选，逐个比：

| 候选 | 有什么 | 缺什么 | 结论 |
|---|---|---|---|
| 格式常量 `singlefs-format` | 两个总宽 `ROOT_RECORD_BYTES`（`crates/singlefs-format/src/lib.rs:210`）、`SYSTEM_CONFIGURATION_BYTES`（`crates/singlefs-format/src/lib.rs:248`），都由门禁 27 号按 kb 里 `format-const` 标记绑住（`.claude/kb/decisions/22-单元原子性怎么合成.md:153`、`.claude/kb/layout/01-first-txn.md:156`）；core 里四档预算 389 / 4 / 36 / 60 是公开常量（`SystemImmutableConfiguration::FIELD_TABLE_BYTES` 等） | 没有逐字段名字与偏移 | 拿来当「合计」的对账：清单各行宽度之和必须等于它 |
| checker 的解析 | `RootView` 只取 4 个字段（`crates/singlefs-checker/src/lib.rs:311-319`）；系统配置只读 23 个偏移（layout 第 158 行列过） | 它读什么就只有什么：拿它当清单会把「checker 看不见的字段」一并变成看不见 | 不取 |
| core 的解析 | `parse_slot` 读 15 行，准入判 9 行 | 同上 | 不取 |
| kb 字段表 | 根记录 15 行（D22 已定项 7）、系统配置 46 行（layout 一）、树表条目 9 行（D22 已定项 7 下一段、layout 七），行序偏移齐全 | 是文本 | **唯一完整的权威**，但文本进不了编译期 |

建议的两层：

1. **清单写成 harness 里的一张表**（改 `crates/singlefs-harness/src/mount_transparency.rs`，新加 `FieldRange { name, offset, width, class }`，根记录 15 行、系统配置 46 行、树表条目 9 行），每行带一个归类（必须相同 / 必须为零 / 模掉并写明谁替它判 / 确定关系并写明式子）。它是「比什么」的唯一来源：判官按表逐行比两个槽的原字节，不再靠 `SystemImmutableConfiguration` 之类内存结构挑哪些字段比。
2. **三道对账**：
   - 对格式常量：表里各行宽度之和等于 `ROOT_RECORD_BYTES` / `SYSTEM_CONFIGURATION_BYTES` / 200，各档之和等于 389 / 4 / 36 / 60（`crates/singlefs-core/src/system_configuration.rs:249`、`:265`、`:286`、`:326` 的 `FIELD_TABLE_BYTES`），行与行首尾相接、无空洞无重叠；写成编译期断言或 harness 的轻用例。格式再加一个字段而改了总宽 ⇒ 红。
   - 对 kb 表：一条门禁格（归主 agent 与 tooling，不在这一轮）解析 D22 已定项 7 与 layout 一两张 markdown 表的（字段名、宽）序列，与清单表逐行比，不等判红。覆盖「同宽替换字段」「改名」这类总宽不变的改动。
   - 对 D13 逐字段表：D13 里那张十二行的表给每行一个稳定编号，清单表的每个「归类」引用一个编号，门禁格核两边编号集合相等：D13 表删一行、清单表引用它 ⇒ 红。这一条回答 C655 还清条件里「删掉表里一行必须红」中「表」指 D13 表的那一读。

### Y2.5 判别力自证怎么造

C655 还清条件：「逐字段表写成条款之后，根记录与系统配置的字段清单里有一个字段不在表里就判红；判别力自证：删掉表里一行，必须红」。「表里一行」有两读，各造一份：

- 读一，清单表删一行（harness 里的 `FieldRange` 表）：覆盖断言（首尾相接、总宽）当场红；再加行为面的一份——表驱动用例，对每一行：取一对合法的 R1、R2 槽，把 R2 槽在这一行的范围里翻一个字节（重封整槽校验和），判官该判 Violated 且违例名带这一行的名字（归类为「必须相同 / 必须为零 / 确定关系」的行），或判 Holds（「模掉」的行）；删表里一行之后这一行的翻字节落在「没有归类的字节」上，判官报「清单没覆盖字节 x」红。
- 读二，D13 逐字段表删一行：上面第三道对账红。
- 往判官里塞已知故障：把某一行的比较从判官里去掉（变异：`mutations.tsv` 里加「判官跳过某一类归类」那几行），翻字节用例必须红（那一份变异表整表是 checker 档，我没跑，只写形态）。

### Y2.6 改法对实现的改动（下一批实现员用）

- `crates/singlefs-core/src/recovery.rs`：新加一个只读函数，交回某一版根所在的根槽原字节（现有 `readable_roots` 只交回解析后的 `RootRecord`，`crates/singlefs-core/src/recovery.rs:1367`），以及择到的系统配置槽的原字节（`choose_system_configuration` 只交回解析后的，`crates/singlefs-core/src/recovery.rs:1052`）。由记录重建出来的版本没有根槽：这一版的 428–456 三样标「不适用」，别的行取记录新根段里有的那几个字段（`crates/singlefs-core/src/recovery.rs:3608-3609` 的施加处）。
- `crates/singlefs-harness/src/mount_transparency.rs`：`RecoveryFieldsRead`（第 93 行）加两个字段存两份原字节；`MountTransparencyField`（第 608 行）加成员，`ALL`（第 634 行）与 `basis`、`violation_name` 三处跟着改；`user_visible_tree_root_pointers_of_a_published_version`（第 580 行）把取的种类扩到 {1, 2, 9, 10, 11}；系统配置世代号与 tail 两条关系式读写表（`RetainedWrite` 带 `device`、`kind`）与 `scan_journal` 的记录表，都是判官手里已有的输入。
- 推翻它的现象（Y2 整节）：找到根记录或系统配置里一个字段，既不在 D22 已定项 7 / layout 一的表里、又在盘上被写；或者 `RootRecord::to_slot` / `SystemConfiguration::to_slot` 的写入顺序与 kb 表偏移不一致（`system_configuration_mutability_classes.rs` 的逐字节 sha256 钉住的是写入顺序，若它与 kb 不一致，清单取 kb 还是取代码要先判）。

## Y3　I-5.5 判不了的每一格

### Y3.0 现状与量法

CPU 判器函数头把这几格列过：「下面几格 `.claude/kb/invariants.md` I-5.5 那一行没写，判不了，记成射程里判不了的那一部分（没有违例时整条报不适用、带理由），不替条款定：」（`crates/singlefs-checker/src/walk.rs:4191`）。CPU 里它们落在 `crates/singlefs-checker/src/walk.rs:4212-4216`（树表读不出）、`:4226-4230`（条目宽不到 200）、`:4241-4245`（种类 1 不止一条）、`:4263-4266`（extent 树读不全）、`:4289-4293`（种类 3 不是恰好一条）、`:4300-4303`（分配记录树读不全）、`:4326-4329`（某块盘上一个数据单元都没有）七处 `part_of_the_range_not_judged`；GPU 判器 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 里同一批全是静默 `return` 或 `continue`：`:1921`「if (!tt.ok || tt.entry_count == 0u || tt.entry_width < 200u) {」、`:1943`「if (extent_entry_count != 1u || !every_birth_is_this_version」、`:1957`「if (!read_the_whole_extent_tree) {」、`:1967`「if (!has_any_data_unit || allocation_entry_count != 1u) {」、`:1971`「if (!r_allocation_tree(tt.base, allocation_entry + 14u)) {」、`:1975`「if (r_has_data_unit_on_device[device_index] == 0u) {」。

`part_of_the_range_not_judged` 的语义：没有违例时整条 I-5.5 报「不适用」（`crates/singlefs-checker/src/image.rs:250-252` 一段，`into_report` 里排在「成立」之前）。另有第四种判定「部分没判」：「部分没判（用户 2026-10-04 定，第四种判定）：判到的那一部分都对得上，射程里另有一部分这张镜像上判不了」（`crates/singlefs-checker/src/image.rs:36`），今天只有 I-9.28 报它：I-9.28 对「树表读不出、条目比字段表短」的先例判法是「部分没判，列出这些根」（`.claude/kb/invariants.md` I-9.28 一行，第 315 行；CPU 的说明在 `crates/singlefs-checker/src/walk.rs:4593`，调用在 `crates/singlefs-checker/src/walk.rs:4684`）。所以每一格不只两选项（红 / 不适用），还有这第三个，而且有现成先例。

**造坏镜像的方法**：在冻结副本的拷贝上把 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 整份拷成 `y3_probe.rs`（它自带 `build_pool`、`mutate_unit`——改一个单元、重封两道校验和、把新的整单元校验和沿引用链补到根槽——与 `check_pool_image`），从写完新池新建文件那份干净镜像出发（树表 11 条、txg 3 = 从 mkfs 水位第一次建树的那一版）逐格改一处，打印红的与 I-5.5 的判定。命令同 Y1.1，测试二进制 `y3_probe-*`，`TMPDIR` 指到草稿目录；原样输出在 `…-sonnet-model/y3-bad-run1.log`、`y3-bad-run2.log`。干净镜像上：`[y3] clean（新池新建文件，txg 3）: 红=[] 部分没判=[]` / `[y3]     I-5.5=Holds`。

### Y3.1 逐格

**a. 树表读不出、条目宽不到 200 字节**

- 合法写侧上出不出现：无故障、写侧合法的崩溃状态里不出现（根槽 FUA 在屏障之后，单元先落）；两种合法情形里出现：读故障域；回退候选集按 checker 只读根上带的 F 取、而系统配置里的 F 更高时，候选里有单元已被合法回收的旧根（I-7.4 一行自己写了「checker 今天只读根上带的 F，带 F 的根全坏时会在合法状态上误红这一条与别的按回退候选集判的几条」，`.claude/kb/invariants.md:58`；I-5.5 的版本就取自这个候选集，`crates/singlefs-checker/src/walk.rs:8189-8199`）。⇒ **不能判红**。
- 坏镜像上：量到已被判红。把树表单元两份清零：`[y3] a 树表单元两份全零（不补引用链）: 红=["I-2.1", "I-4.8", "I-7.2", "I-7.4"] 部分没判=["I-9.28"]`，同一镜像上 `I-5.5=NotApplicable("有一版的树表读不出：认不出它是不是从 mkfs 水位第一次建树的那一版")`（`y3-bad-run1.log`）。条目宽不到 200：走读守卫当场记「走读断了」，由 I-4.8、I-7.4 说话（`crates/singlefs-checker/src/walk.rs:1798`「/// 切之前先判一次，窄的如实记一条「走读断了」（I-4.8（近 K 代根校验和自洽） / I-7.4（近 K 代块未被复用） 由它说话），」；这一半我没造镜像，读码推的）。
- 判定：**与别的不变量重复，且在合法的 C556 窗口里会双重误红 ⇒ 不判红；报「部分没判」**（不是「不适用」：判到的别的版本成立不等于整条不适用，这是 I-9.28 同一类的先例）。
- 对实现的改动：CPU `crates/singlefs-checker/src/walk.rs:4212-4216` 与 `:4226-4230` 两处 `judgements.part_of_the_range_not_judged(invariant, …)` 改调 `judgements.partially_judged(invariant, 逐条列出：实例、txg、为什么没判)`（形态照 `crates/singlefs-checker/src/walk.rs:4684`）；GPU 不改：GPU 判器只有违例位（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1921`），「不适用 / 部分没判」只在 CPU 报告里分，层 0 把部分没判算不红。kb 里 I-5.5 那一行把「报射程里判不了」改写成「部分没判」。
- 判别力自证：不需要新坏镜像（不判红的那一格自证「不红」）：上面 a 镜像改前改后 `红` 列表一样（四条），I-5.5 由 `NotApplicable` 变 `PartiallyJudged`，用 `matches!(…, PartiallyJudged(_))` 断言。
- 推翻它的现象：一份合法写侧（含崩溃态、回退、读故障）镜像上，树表读不出而别的不变量全不红，且这一版恰是从 mkfs 水位建树的那一版——那时这一格是真漏判，要判红。

**b. 种类 1（extent 树）条目不止一条**

- 合法写侧上出不出现：不出现。写者按种类装树表、每种一条（`crates/singlefs-core/src/transaction.rs:8188-8271`）；读者拒收：「树表里同一种树有两条」（`crates/singlefs-core/src/recovery.rs:2296`），拒的那一行是「return Err(TreeTableEntriesRefusal::OneKindOfTreeAppearsTwice);」（`crates/singlefs-core/src/recovery.rs:2336`），重建那一版时先于任何读树就拒（`crates/singlefs-core/src/recovery.rs:2516` 起）。⇒ 可以判红。
- 与别的不变量重复吗：**不重复**。量到（`y3-bad-run2.log`）把 livelist 条目的种类改成 1、头 ID 也写 65537 之后：`[y3] b2 种类 1 两条，第二条头 ID 也写 65537: 红=[] 部分没判=[]`，`I-5.5=NotApplicable("有一版的树表里种类 1（extent 树）条目不止一条：认法 ① 读哪一条，I-5.5 那一行没写")`。不改头 ID 的那一份（`y3-bad-run1.log` 的 b）I-9.27 先红：`红=["I-9.27"]`，那是头 ID 取值红，不是种类重复红。I-9.16 的 CPU 实现明说不判重复：「同一种有两条时取先出现的那条（核心层把它判成损坏，这里不另立判定）」（`crates/singlefs-checker/src/walk.rs:3498`）。
- 判定：**判红，但归属不在 I-5.5**。病根是树表结构，凡读树表的不变量（I-9.27、I-5.5、I-3.x 走读）都有这个盲点；归到 I-9.16 加一句 ③「同一张树表里每种种类至多一条」，I-5.5 对这一格继续报「部分没判」且理由写「已由 I-9.16 ③ 判」。理由：① 本仓的纪律是每条不变量配一份「改坏了正好触发它」的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 文件头），I-5.5 管落点，在种类重复上红会让 I-5.5 与 I-9.16 谁来红说不清；② 不判在 I-9.16 的话以后别的不变量各自再碰一次同样的格。
- 备选（若主 agent 要「每格都落在 I-5.5 那一行」的字面）：`crates/singlefs-checker/src/walk.rs:4241-4245` 的 `part_of_the_range_not_judged` 改 `judgements.judge(invariant, false, …)`，GPU `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1943` 把「extent_entry_count != 1u」拆成「== 0u 照旧 return（不是这一类）」与「>= 2u 则 violate(INVARIANT_I_5_5)」。代价是 b2 这份镜像上 I-5.5 与（将来的）I-9.16 ③ 两条一起红。
- 判别力自证：b2 镜像（`y3_probe_bad_images_second_batch`）：改之前 `violated_invariants = []`、I-5.5 `NotApplicable`；改之后 `== ["I-9.16"]`（归 I-9.16）或 `== ["I-5.5"]`（备选）。再造一份「干净镜像」断言 I-9.16 仍 `Holds`。
- GPU 改法（归 I-9.16 的主方案）：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1661-1694` 的 `w_judge_tree_table_entries_ordering` 里在两层循环之后加「同种类两次出现」的一趟（条目数 ≤ 81，二重循环够用）。
- 推翻它的现象：某个合法写侧状态（含回退、多实例）上出现同一种类两条的树表——那时读者也该拒不了，这条结论连同读者的拒收一起错。

**c. 认出来的那一版 extent 树读不全**

- 合法写侧上出不出现：同 a（读故障域、C556 窗口里的旧根；无故障写侧里该版单元未被复用）。
- 坏镜像上：量到已被判红：`[y3] c extent 树根两份全零（不补引用链）: 红=["I-2.1", "I-4.8", "I-7.2", "I-7.4"] 部分没判=[]`，`I-5.5=NotApplicable("从 mkfs 水位第一次建树的那一版 extent 树读不全：它引用的数据单元数不全")`。节点形状错（层级、key 宽、孩子 key 对不上）由主走读的 I-1.1 / I-1.7 / I-1.10 判，这一半读码推的（`crates/singlefs-checker/src/walk.rs:4041-4044`、`:4068` 的 `return None` 对应主走读判红的那几处）。
- 判定：**与别的不变量重复 ⇒ 不判红；部分没判**。改法同 a：CPU `crates/singlefs-checker/src/walk.rs:4263-4266`，GPU `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1957` 不改。
- 推翻它的现象：extent 树读不全而 I-2.1、I-1.x 都不红的合法或坏镜像（节点形状错而走读不红）。

**d. 有数据单元而池里某块盘上一个都没有**

- 合法写侧上出不出现：今天的两盘镜像写侧不出现：mkfs 断言恰两块盘（`crates/singlefs-core/src/make_filesystem.rs:477`「"第一版跑 2 块盘（D2（RAID 条带策略） 已定项 9）"」），每个单元的两条位置条目取排序后最小的两块盘（`crates/singlefs-core/src/make_filesystem.rs:266`起 `location_entries`），分配器一个落点在每块盘上同槽（`crates/singlefs-core/src/allocator.rs:1546-1582` 的 `try_allocate_user_data`，要各盘答案一致）。**在多盘布局下合法**：格式留了设备数字段、w_max = 4、g = 4（layout 一表「条带宽度上界」「组大小」两行，`.claude/kb/layout/01-first-txn.md:143`、`:144`），每次写只落 w 列时，某块盘上没有这一版的数据单元是正常的。⇒ 不能判红。
- 坏镜像上：两种造法都量到已被判红：位置条目写成 (盘 1, 盘 1)：`红=["I-2.5"]`；写成 (盘 0, 盘 2)、池里没有盘 2：`红=["I-2.1", "I-4.8", "I-7.4"]`（`y3-bad-run1.log` d、`y3-bad-run2.log` d2）。两盘池里每个数据指针的两条位置条目只能是 (盘 0, 盘 1)，所以「某块盘上没有数据单元」在两盘池里只来自指针本身坏了，那几条已经红。
- 判定：**该盘跳过（它没有 I-5.5 的对象），不判红，也不再报「判不了」**。条款原文「每块盘上落点槽号最小的那一个」对没有数据单元的盘本来就空真。GPU 早就是这么做的（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1975` 的 `continue`），CPU 与它不一致。
- 对实现的改动：CPU `crates/singlefs-checker/src/walk.rs:4326-4329` 的 `None => judgements.part_of_the_range_not_judged(…)` 改为 `None => {}`（加注释写「这块盘没有这一版的数据单元：条款没有对象」）；GPU 不改。kb I-5.5 一行把这一格从「判不了」那串里删掉。
- 推翻它的现象：两盘池的合法写侧镜像上，某个数据指针的两条位置条目指到同一块盘，而 I-2.5 没红——那一格要重判。

**e. 种类 3（分配记录树）条目不是恰好一条；分配记录树读不全**

- e1（0 条）。合法写侧不出现：写者一定写一条（`crates/singlefs-core/src/transaction.rs:8201-8206`），读者重建时缺了就拒：「树表里没有那棵树」（`crates/singlefs-core/src/recovery.rs:2530`「what: "树表里没有那棵树",」，`pointer_of(TREE_KIND_ALLOCATION)?` 在 `crates/singlefs-core/src/recovery.rs:2535`）。与别的不变量重复吗：**不重复**，量到：把分配记录条目的种类改成没登记的 13：`[y3] e1 种类 3 条目 0 条（改成没登记的 13）: 红=[] 部分没判=[]`，`I-5.5=NotApplicable("从 mkfs 水位第一次建树的那一版树表里种类 3（分配记录树）条目不是恰好一条：落点按哪一棵分配记录树算，I-5.5 那一行没写")`——一条红都没有。
- e2（2 条）。合法写侧不出现（读者拒收，同 b）。量到：把记账那一条的种类改成 3：`红=["I-1.1", "I-4.8", "I-7.2", "I-7.4"]`——红，但靠的是那一条指着的其实是记账树、走读按分配记录树读它时对不上，不是「种类重复」这件事被判了：两条都指真分配记录树的重复不会靠它红。
- e3（读不全）。合法写侧同 a。量到：根节点两份清零：`红=["I-2.1", "I-4.8", "I-7.2", "I-7.4"]`，`I-5.5=NotApplicable("从 mkfs 水位第一次建树的那一版分配记录树读不全：落点算不出")`。
- 判定：e1、e2 与 b 同类：**判红，归 I-9.16 加 ④「树表非空时种类 1、2、3、4 各恰好一条」**（读者要的就是这四种：`crates/singlefs-core/src/recovery.rs:2533-2536`），I-5.5 继续报部分没判；e3 同 a、c：**部分没判**。
- 改法：CPU `crates/singlefs-checker/src/walk.rs:3466-3520` 的 `judge_tree_table_entries_ordering` 旁新加 ③ ④ 两道；GPU `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1661-1694`；I-5.5 的 CPU `crates/singlefs-checker/src/walk.rs:4289-4293`、`:4300-4303` 与 GPU `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1967`、`:1971` 不改判定，只改报告。
- 判别力自证：e1 镜像：改之前 `violated_invariants=[]` 且 I-5.5 `NotApplicable`，改之后 `==["I-9.16"]`。
- 推翻它的现象：合法写侧、读者不拒的树表缺种类 3（那时读者的拒收与我的 ④ 都错）。

**f. 开放段在提交内生块回落时的误排**

- 机制（读码）：CPU 拿「这一版树表单元在这块盘上那一份所在的 64 槽段」当这一版的开放段（`crates/singlefs-checker/src/walk.rs:4349-4360`，注释：树表单元最末从开放段 bump），选落点时整段排除：`crates/singlefs-checker/src/walk.rs:4394`「if let Some(segment) = open_cluster_segment.filter(|segment| segment.holds_slot(candidate))」。分配器回落那一支把 `open_segment` 置空（`crates/singlefs-core/src/allocator.rs:1648`「CommitGeneratedDeviceAnswer::FallBackToLowestFreeSlot(slot) => {」、`crates/singlefs-core/src/allocator.rs:1651`「self.open_segment = None;」），之后的提交内生块（含树表单元）落在「该盘单元区里槽号最小的空槽」。树表单元若因此落进数据单元所在的 [U, U + 64)，CPU 与 GPU（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1984`、`:1987`）把整段排除，期望落点跳到 U + 64，实际在 U + 4（或 U），误红。这一半读码成立，没造出镜像（造不出，见下）。
- 回落进得去的条件（读码）：每块盘上都没有全空 64 槽段：`crates/singlefs-core/src/allocator.rs:805`「match self.lowest_empty_segment() {」只认 `used_per_segment`、`isolated_per_segment`、`held_per_segment` 全 0 的段（`crates/singlefs-core/src/allocator.rs:761-776`），已释放而没回收的落点仍算 used；而且开放段装不下这个单元（新分配器 `open_segment` 起手是 None，`crates/singlefs-core/src/allocator.rs:1232`「open_segment: None,」）。这一版还得被认成「从 mkfs 水位第一次建树」：树 ID 仍是 65536，也就是之前没发过文件（或根环转过一圈把带文件的根都盖掉），所以能让池里没有全空段的历史只有「只暖机 / 只写行的挂载」与零单元发布这一族。
- 量到（`y3-fb-run1.log`、`y3-fb-run2.log`、`y3-scan-run2.log`）：在拷出的副本里给 `FallBackToLowestFreeSlot` 那一支加计数器（`allocator-fallback-counter.diff`），对单元区 240、256、384 槽三种小盘，从 mkfs 之后连续可写挂载 k 次再发第一个文件：准入开，k ∈ {1,2,3,4,6,8,10,12,16,20,24,30,40,60} 共 42 份，**回落 0 次**，每份历史都跑完；准入关（只供测试的开关），k ∈ {1,4,8,12,24,60} 共 18 份，**回落 0 次**；另对 k ≤ 30 的 36 份跑了池级 checker 整份判，I-5.5 全是 `Holds`、没有一条红。回落一次都没走到，所以误排这一格的镜像造不出。回落这一支只在测试里把空闲图改成「没有全空段」才进得去（`crates/singlefs-harness/tests/commit_generated_units_fall_back_when_the_segment_runs_out.rs:10-11` 的话：「没有全空段」的形态直接改空闲图造，用真发布占满 3312 个段要几千次发布）。
- 判定：**今天的写侧够不着**（推的 + 60 次 0 回落，不是证明）；写成什么：不改判定，**写成带触发条件的已知限度加一个会红的看门**：
  - I-5.5 一行里把「提交内生块回落时会误排，今天的准入下够不着」改写成两条可判的话：①「误排要求这一版发布时每块盘上都没有全空 64 槽段」；②「今天的写侧在『只暖机 / 只写行的挂载接发第一个文件』这一族里量到 0 次回落（单元区 240 / 256 / 384 槽，k ≤ 60）」。
  - 看门（harness 轻用例，不是 checker 档）：把我的 `y3_probe_fallback_count_with_the_admission` 改成断言 `FALLBACK_COUNT == 0`（需要分配器上留一个只读的回落计数，`crates/singlefs-core/src/allocator.rs` 加一个字段与读法），哪天这一族历史走进回落这一支，用例红，提醒回来重看 I-5.5 的开放段认法。红了之后的改法不能是「从盘上换个认法」：我分析过，盘上看不出回落（回落的单元与 bump 的单元都只是「某个槽上的单元」），要么在写侧把「这一版的开放段」落到盘上（格式改动，另走三方），要么放弃整段排除、只做单边检查——推的。
  - CPU / GPU 今天都不改。
- 推翻它的现象：任一合法写侧历史（含读故障、回退、多实例）上走到回落这一支、而这一版被认成从 mkfs 水位建树的那一版；或准入式、聚簇段长（64 槽）、回收门槛变了而这一族的扫描回落次数不再是 0。

### Y3.2 对实现的改动汇总（给下一批实现员）

| 格 | CPU（`crates/singlefs-checker/src/walk.rs`） | GPU |
|---|---|---|
| a 树表读不出 / 条目宽不到 200 | `:4212-4216`、`:4226-4230` 改 `partially_judged`，列出实例、txg、原因 | 不改（`crash_judge_checker_scan.wgsl:1921`） |
| b 种类 1 不止一条 | I-9.16 加 ③：`:3466-3520` 一旁新加一道；I-5.5 `:4241-4245` 保持「部分没判」写明「已由 I-9.16 ③ 判」 | `crash_judge_checker.wgsl:1661-1694` 加；`crash_judge_checker_scan.wgsl:1943` 不改 |
| c extent 树读不全 | `:4263-4266` 改 `partially_judged` | 不改（`:1957`） |
| d 某块盘没有数据单元 | `:4326-4329` 改成不报、不记 | 已是（`:1975`），不改 |
| e1、e2 种类 3 条数不对 | I-9.16 加 ④；I-5.5 `:4289-4293` 保持部分没判 | 同 b |
| e3 分配记录树读不全 | `:4300-4303` 改 `partially_judged` | 不改（`:1971`） |
| f 回落误排 | 不改；加会红的看门（分配器计数 + harness 轻用例） | 不改 |

kb 要动的行：`.claude/kb/invariants.md` I-5.5 一行（第 199 行，「判不了」那一串逐格改写），I-9.16 一行（第 303 行，加 ③ ④）；`.claude/kb/checks-owed.md` C687 还清，C689 不动。

**旁见（不在这一轮的格里，读码推的，没造镜像）**：GPU 的 I-9.16 ② 只循环七种种类，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1674`「for (var k = 0u; k < 7u; k = k + 1u) {」，种类取 1、2、3、4、6、7、8（`:1675-1678`）；CPU 查十一种（`crates/singlefs-checker/src/walk.rs:78-90` 的 `TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`，多出种类 9、10、11、12）。互换 `deleted_inodes` 与墓碑两条的种类，CPU 该红、GPU 不红。要核得跑 GPU 判器（checker 档，我没跑）。

### Y3.3 补：改之后（CPU 这一半在拷出的副本上做了、并量了）

在拷出的副本里照上面的改法改了 CPU 一半（`research/prompts/m3-formal-c655-c687-r1-sonnet-model/walk-y3-patch.diff`，对 `crates/singlefs-checker/src/walk.rs` 的 91 行 diff）：`judge_tree_table_entries_ordering` 里加 ③ ④ 两道；I-5.5 里 a、c、e3 三处 `part_of_the_range_not_judged` 改 `partially_judged`，d 那一处改成不报；b、e1、e2 的 I-5.5 仍报「不适用」。GPU 一半没动、没跑。同一批坏镜像改后再量，原样输出在 `y3-after-run1.log`（只加 ③ ④ 时）与 `y3-after2-run1.log`（两样都改后）：

| 镜像 | 改之前：红 / I-5.5 | 改之后：红 / I-5.5 |
|---|---|---|
| clean | `[]` / Holds | `[]` / Holds |
| a 树表读不出 | I-2.1、I-4.8、I-7.2、I-7.4 / NotApplicable | 同样四条红 / `PartiallyJudged("根（实例 1、txg 3）：树表读不出，认不出它是不是从 mkfs 水位第一次建树的那一版")` |
| b2 种类 1 两条（头 ID 也对） | `[]` / NotApplicable | `["I-9.16"]` / NotApplicable |
| b 种类 1 两条（头 ID 不对） | `["I-9.27"]` | `["I-9.16", "I-9.27"]` |
| c extent 树读不全 | 四条红 / NotApplicable | 同样四条红 / PartiallyJudged |
| d 数据指针都在盘 1 | `["I-2.5"]` / NotApplicable | `["I-2.5"]` / Holds |
| d2 数据指针 (盘 0, 盘 2) | I-2.1、I-4.8、I-7.4 / NotApplicable | 同样三条红 / Holds |
| e1 种类 3 零条 | `[]` / NotApplicable | `["I-9.16"]` / NotApplicable |
| e2 种类 3 两条 | I-1.1、I-4.8、I-7.2、I-7.4 | 加上 I-9.16 |
| e3 分配记录树读不全 | 四条红 / NotApplicable | 同样四条红 / PartiallyJudged |

回归（harness 档，拷出的副本上改后各跑一遍，原样输出同目录 `y3-after*-known-bad.log`、`y3-after-treetable-ordering.log`、`y3-after*-first-tree-building.log`、`y3-after-checker_narrow_invariants_and_abandoned_roots.log`）：`checker_known_bad_images` 69 个全过（含「干净镜像上每条不变量都成立、每份坏镜像只红它的目标」那条）、`tree_table_ordering_is_judged_by_the_cold_walk_and_the_checker` 8 个全过、`pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement` 9 个全过（里面是 I-5.5 的合法镜像：只暖机挂载 9 / 10 次、22 次零单元发布后写第一个文件等）、`checker_narrow_invariants_and_abandoned_roots` 20 个全过。所以 ③ ④ 没有在这些合法历史上误红，a、c、e3 改成部分没判没有碰坏已有断言（它们没有断言 I-5.5 在这几格是 `NotApplicable`）。副本上的数不进 kb，要入库那一版由主 agent 在入库装置上重做（`.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y1 `mapping_root` | 模掉指针字节，判空性等价 + checker 判绿（量过） | 带文件每版换，字节必不同；树表 0 条时全零不变 |
| Y1 `allocation_record_tree_root` | 确定关系（量过） | 带树 ⇒ 全零；树表 0 条 ⇒ 非零 ⇔ 新实例号 ≥ 2 |
| Y1 墓碑 / 谱系 / deleted_inodes 根 | 并进两根指针那一行，同一条续做规则（量过） | 不是根记录字段；kind=9 续做时变，「必须相同」错 |
| Y1 算法 / nonce / MAC | 必须相同且为 0；判官读根槽原字节（读码） | 写侧写常量 0，读侧与 checker 都不读，今天没人判 |
| Y1 槽世代号 | 确定关系（量过 18 盘行） | = R1 + 写到这盘的系统配置写条数 |
| Y1 journal_tail | 确定关系（量过 18 盘行） | = 环里新实例最大计数器，且单调 |
| Y2 字段清单完整性 | 根记录 15 字段只漏 magic、校验和；系统配置 46 行里 5 行表外、26 行判官看不见 | 题面把三根指针归到根记录是错的 |
| Y2 清单取处 | kb 字段表为权威，格式常量总宽对账，加一道 kb 表对账门禁 | 格式常量只有总宽，checker 与 core 的解析都是子集 |
| Y2 判别力自证 | 清单表删一行 ⇒ 覆盖断言红；D13 表删一行 ⇒ 编号对账红；翻字节表驱动用例 | 判官未改，用例形态是推的 |
| Y3 a 树表读不出 / 条目窄 | 部分没判，不红（量过 a） | 坏镜像上 I-2.1 / I-4.8 / I-7.2 / I-7.4 已红；C556 窗口里合法 |
| Y3 b 种类 1 不止一条 | 判红，归 I-9.16 ③（改前改后都量过 b2） | 今天没有别的不变量红；读者拒收，写侧不出 |
| Y3 c extent 树读不全 | 部分没判，不红（量过 c） | 同 a |
| Y3 d 某块盘没数据单元 | 该盘跳过，不红不报判不了（量过 d、d2） | 两盘写侧不出现、多盘布局合法；坏指针已被 I-2.5 / I-2.1 判红；GPU 本来就跳过 |
| Y3 e1、e2 种类 3 条数不对 | 判红，归 I-9.16 ④（改前改后都量过 e1、e2） | e1 一条红都没有；e2 红是偶然 |
| Y3 e3 分配记录树读不全 | 部分没判，不红（量过 e3） | 同 a |
| Y3 f 回落误排 | 今天够不着（推的，60 份 0 回落）；写成带条件的已知限度 + 会红的看门 | 误排机制读码成立，镜像造不出 |

## 没做什么

- 没跑 checker 档（`singlefs-checker-tier` 测试、GPU 判器、层 0 回放、变异整表）：凡涉及 GPU 的都是读码推的，包括 Y3 各格的 GPU 改法与旁见。
- 没改主仓任何文件。Y1.5、Y2.5 的「改之后红」是对着我设想的判官说的（判官未改），没量；Y3 的 CPU 一半在拷出的副本上改了并量了「改之后」（Y3.3），GPU 一半没改没跑。
- 二次崩溃（挂载途中）的世代号、tail、各指针关系没有量：只量了只挂载与挂载加一次发布；二次崩溃的持久子集语义靠推。
- Y1 的量只覆盖七个场景（mkfs 之后第一、二次挂载、第一个文件之后、正常卸载之后、推抬 F、崩在 unlink 意图之后、有树而无文件的一版）：管理员回退之后挂载、崩溃恢复抛弃根之后挂载、读故障下的挂载没量，各关系在那几条历史上没验。池都是两块盘，多盘没量。
- Y3 a 的「条目宽不到 200」没造镜像（读码：走读守卫记「走读断了」归 I-4.8 / I-7.4）。
- Y3 f 没造出回落的合法镜像（走不到）；`mount_and_one_publish` 之外的动作（回退、卸载后重挂再回退）没扫。
- 没把我的发现采成判决，也没替主 agent 定「b、e1、e2 归 I-9.16 还是归 I-5.5」：两个都写了、各自的代价写在 b 里。
- 草稿与拷出的副本（`/tmp/claude-1000/m3-formal-c655-c687-r1-sonnet/` 下的 `work/`、`target/`、`tmp/`）交回前已删，探针与原样日志在 `research/prompts/m3-formal-c655-c687-r1-sonnet-model/`；那里的 `.rs` 是追加段，不是整份测试（复现法见该目录 `README.md`）。
- 这个目录里的 `.rs` 带行内数字（树 ID、槽号等），门禁 `code-source-discipline` 的格式常量格若扫它会红：这归主 agent 判是否要进 `.claude/format-constants-exclude`。
