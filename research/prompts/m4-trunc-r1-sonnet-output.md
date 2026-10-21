# m4-trunc-r1 云端正推腿（Sonnet）报告：截断的设计第一轮，W1–W9

日期：2026-10-16。立场：正文第四节分工表「云端正推（Sonnet）」那一行——从条款原文与 `crates/` 今天的代码逐格推，每格给候选、要动的文件与步骤、新写出的字节、代价，推荐一个并写依据，岔路写成候选 / 翻面观测 / 够判条件三列，不带倾向。
读的树：`/home/fy5090/.local/share/singlefs/work/m4-predev`（引的路径与行号都是这棵树里现取的；kb 行号是 kb 文件自己的行号）。方案按 `crates/` 今天的实现谈。
没读别的腿这一轮的产出（禁读清单：`research/prompts/m4-trunc-r1-opus-output.md` 与 `research/prompts/m4-trunc-r1-opus-model/`）。
标记：「量的」= 我这一轮在草稿副本里跑出来的；「读码推的」= 读代码推出，没跑；「条款」= kb 原文。

## 〇、先说五件事（后文每格展开；每件的推翻条件在各格里）

1. **载体与形（W1/W2）**：截断走 logged_ops 新种类码（条目装 inode、对象出生代、目标 size），意图那次发布就把记录的 size 写成 N 并重写尾单元，之后每批只释放尾部单元、不碰 inode 记录，最后一批在同一次发布里删条目。这条路让每个中间态都是「N 字节的文件，尾部还挂着没放的单元」，崩溃后只有「截前 / 截后」两种读法。备选是照 unlink 的形状让 size 随批下降（读者不用改、但批间崩溃读到的是前缀）。
2. **尾单元（W3）**：今天的读者在尾单元不重写时一律拒（量的，探针 C/D）；而且今天换下清单的判别 `roles.contains(Data(0))` 会在「只重写 Data(0)」的发布里把后面的单元也算作换下（量的，探针 F）——原样用它的话，截到一个单元以内（含截到 0）会在意图那次提前放掉还挂在树里的单元（函数层量的，端到端没跑）。这两处是实现的必改点，不是候选之争。
3. **洞（W5/W6）——条款内部打架**：D8 已定项 14 的「洞就是缺席」、D18 已定项 10 的读法 Z（size 之外判死、不写区间墓碑）和用户「变长留洞」三句，在「先缩短、再把 size 拉大」这条历史上不相容：缩短时放掉的单元还留在盘上，size 拉大之后读法 Z 的「⌈size ÷ 净荷⌉ 之内都活」把它们认回来，I-1.13 在合法历史上判红（量的，探针 B）。要用户定围栏（岔路 W6-1）。
4. **洞只会出现在尾部（W5）**：今天所有写入口都是从偏移 0 整份重写，新增的只有 truncate；所以洞恒是「从 0 起连续的 u 个单元 + 尾部缺席」。`TransactionOutput::data_pointers` 按下标当单元号这一前提因此不用拆（正文 W5 预设要改成映射，读码推的这里不必）。出现「在偏移 X 写」这种入口时这条才失效。
5. **意图队列的接口是一个走空加四处守卫（W1/W2）**：共用走空 `walk_the_deletion_queue_until_it_is_empty` 加一个来源，三处「队列空」的判断加一项，外加可写挂载准入那一判里「删除队列不空就把环窗口算作可动用」的一处（`crates/singlefs-core/src/admission.rs:448`，它只看 `deleted_inodes`）；此外 `DeletionStepOfThePublish` 与 `LoggedOperationsStepOfThePublish` 两个封闭枚举加成员：`grep -rn 'DeletionStepOfThePublish::' --include=*.rs crates` 在 `crates/` 下命中 41 行（全在 `crates/singlefs-core/src/transaction.rs`），`LoggedOperationsStepOfThePublish::` 命中 15 行（量的）；这是引用成员的行数，不是 `match` 个数，`match` 不写通配臂，加成员时每处都要过一遍。

## 一、这一轮我自己量的（草稿副本上的探针，三个，全部经 `scripts/hosts/run-with-memory-cap.sh 8G` 跑）

副本：`/home/fy5090/.local/share/singlefs/work/m4-trunc-r1-sonnet/frozen`（工作树 crates、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default 的干净拷贝，`transaction.rs` sha256 `e807c95450ccc282235ef360bd951f5a7ca5ceddd59565bc1ee6fdc965201d9b`，与工作树同值）。探针源码、核心层一个露出私有函数的垫片、重跑脚本、原样输出都在模型目录 `research/prompts/m4-trunc-r1-sonnet-model/`（`probes/`、`tools/rerun.sh`、`out/`）。重跑命令：

`SINGLEFS_HEAVY_TESTS=1 bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m4-trunc-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新草稿目录>`（本轮在 `rerun-work` 里跑过一遍，退出码 0，下面抄的是那一遍的输出）。

探针 A/B（`probes/trunc_probe_stale_unit_resurrection.rs`）：三个单元的文件覆盖写成一个单元（今天能走到的缩短），再把 inode 记录的 size 改成 3 × 32634（等于「只改记录的变长」留下的盘）：

```
A 缩短之后（今天能走到的镜像）: I-1.13 -> NotApplicable("扫描重建：一个当前版本、没被墓碑压住的数据单元落在它那条 inode 记录的 size 之外（文件变短留下的），既没有更新的版本也没有墓碑压住（C113 定案 P4、I-1.9 未实现），认不认领第一版不定")
B 再把记录的 size 改成三个净荷（模拟只改记录的变长）: I-1.13 -> Violated("扫描重建的三张表与遍历方向从最新根读出的不同：「中央映射」差 2 处，第一处 key （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）：扫描重建 落点 {盘 0 槽 50184、盘 1 槽 50184}、遍历方向没有这一条；「每个 inode 的数据单元集合」差 2 处，第一处 key （inode 1、偏移 32634）：扫描重建 （码 1、出生树 65536、出生 txg 4、实例 1、事务号 3）、遍历方向 没有这一条")
```
B 里池级 checker 只有 I-1.13 一条判红，别的不变量（含 I-9.15）全不红（输出只列了判违例的）。

探针 C/D/E（`probes/trunc_probe_readers_on_a_record_edited_size.rs`）：新池新建文件（3000 字节、一个单元）后只改 inode 记录的 size，不动单元：

```
C 缩到 2990（尾单元没重写）: 池级 checker 判违例的不变量 = []
C 缩到 2990（尾单元没重写）: read_at 拒：DataUnitDisagreesWithTheLookupPath { unit_index_in_file: DataUnitIndexInFile(0), invariant: "E142 走读同款", detail: "数据单元的声明长度与文件大小按净荷容量算出来的不符" }
D 变长到 3010（同一个单元内，尾单元没重写）: 池级 checker 判违例的不变量 = []
D 变长到 3010（同一个单元内，尾单元没重写）: read_at 拒：DataUnitDisagreesWithTheLookupPath { unit_index_in_file: DataUnitIndexInFile(0), invariant: "E142 走读同款", detail: "数据单元的声明长度与文件大小按净荷容量算出来的不符" }
E 变长到三个净荷（尾部是洞）: 池级 checker 判违例的不变量 = []
E 变长到三个净荷（尾部是洞）: open_file 拒：ExtentRecordCountDoesNotMatchTheFileSize { inode: InodeNumber(1), extent_records: 1, data_units_implied_by_the_file_size: 3, file_size_in_bytes: 97902 }
```
（C 的第一行 `mount_read_only` 本身没拒，拒在 `open_file` 之后的 `read_at`；输出完整行在 `out/trunc_probe_readers_on_a_record_edited_size.log`。）

探针 F（`probes/trunc_probe_roles_replaced_via_mapping.rs` 加垫片，垫片只在副本里把私有的 `roles_replaced_via_mapping` 露出来）：三个单元的上一版上，问「只重写 Data(i)」的发布换下哪些角色：

```
上一版数据单元数 = 3
只重写 Data(0) -> 换下清单 [Data(DataUnitIndexInFile(0)), Data(DataUnitIndexInFile(1)), Data(DataUnitIndexInFile(2))]
只重写 Data(1) -> 换下清单 [Data(DataUnitIndexInFile(1))]
只重写 Data(2) -> 换下清单 [Data(DataUnitIndexInFile(2))]
```

三个探针都只做了一次抽样（`.claude/rules/three-way-inference.md` 的「没打中一次不算」针对的是否定结论；这里是肯定的观测，且内存盘确定性：两遍输出内容相同，第一遍在草稿目录的 `probe-run.log`、`probe2-run.log`、`probe3-run.log`，第二遍是 `rerun-work`；第一遍 `--test-threads 4`、第二遍 1，只有 `test … ... ok` 那一行与第一行输出的拼接位置不同，diff 过）。它们量的是「今天的读者与判器对只改记录的盘说什么」，不是候选方案的数。

## 二、九格共用的骨架：符号、今天的实现缺口、推荐形态的状态机与新写出的字节

### 2.1 符号

P = 一个数据单元的净荷容量 32634（`crates/singlefs-core/src/unit.rs` 的 `data_unit_payload_capacity`，载荷偏移 134 = 头 105 + 预留 29，32768 − 134）。u = 现行那一版的数据单元个数（`TransactionOutput::data_pointers.len()`，`crates/singlefs-core/src/transaction.rs:2932`）。S = 现行 size，N = 目标 size。keep(N) = max(1, ⌈N ÷ P⌉)：有数据的 inode 截断之后至少留一个单元（长度 0 也留一个声明长度 0 的单元，`crates/singlefs-core/src/extent_tree.rs:11`：「一个单元都没有的文件写不出来」）。k = keep(N) − 1 是尾单元序号，t = N − k × P 是尾单元里该有的字节数（N 是 P 的整数倍且 N > 0 时 t = P，不用重写）。

### 2.2 今天的实现里，截断绕不开的四处（两处缺口、一道现成的网、一处读者；读码推的，探针 C/D/E/F 是量的）

1. **写入口只有「整份内容从偏移 0 重写」**：`FileVersionPlan` 的文档逐字「`content` 是整份文件内容（顺序写从偏移 0 写到末尾）」（`crates/singlefs-core/src/transaction.rs:4440`）；一个发布里的数据单元事务由 `PublishPlan::file_content_transactions` 从这份内容切出，空内容的发布给空表（`let file_content_transactions = self.file_content_transactions();`，`crates/singlefs-core/src/transaction.rs:6199`）；装单元只有两条臂——有文件版本就整份装，没有就整份照抄前 `carried_data_units` 个（`(None, Some(carried)) => carried_file_version_units(carried, resolved.carried_data_units),`，`crates/singlefs-core/src/transaction.rs:10666`）。「照抄前 k 个、重写第 k 个、其余留在树里」这第三种装法今天没有。
2. **换下清单的判别把「含 Data(0)」当成「整份重写」**：注释「// 文件内容重写恒从第 0 个单元写起；不含第 0 个单元的数据角色是删除批截掉的尾部」（`crates/singlefs-core/src/transaction.rs:3641`），代码是「`roles.contains(&TransactionUnit::Data(DataUnitIndexInFile::FIRST));`」（`crates/singlefs-core/src/transaction.rs:3644`）。探针 F 量的：只重写 Data(0) 时清单是 [Data(0), Data(1), Data(2)]（上一版 3 个单元）——尾部被当成同一次发布换下。这个函数有三个调用点（`crates/singlefs-core/src/transaction.rs:3535`、`:3801`、`:4853`，grep 数出来的）。
3. **换下清单两侧相等的断言**会当场 panic 守住任何一边漏改：「"resolve 判生死的那一组用户可见单元，就是释放判定路径按这次换下的全部角色走出来的用户可见那一组"」（`crates/singlefs-core/src/transaction.rs:9068`）。这是好事：缺口 2 改歪时测试会红，不会静默。
4. **读者三处按「声明长度 = min(size − 起点, P)」与「记录条数 = ⌈size ÷ P⌉」拒**：`crates/singlefs-core/src/mounted_read.rs:815`（条数）、`crates/singlefs-core/src/mounted_read.rs:1090`（声明长度）、`crates/singlefs-core/src/recovery.rs:5381`（条数）、`crates/singlefs-core/src/recovery.rs:5440`（声明长度）。探针 C/D/E 量的：只改记录的 size，读者三种都拒，池级 checker 一条不红。

### 2.3 洞只会出现在尾部（读码推的）

今天改文件内容的入口是 `publish_first_file`（`crates/singlefs-core/src/transaction.rs:7017`）、`publish_overwrite`（`:7112`）、`publish_sequential_write`（`:7168`），三个都接整份内容、从偏移 0 写；`publish_new_inodes`（`:7227`）建 size 0 的空 inode。新增的只有 truncate。所以任何历史里，一个 inode 在 extent 树里的单元是「从 0 起连续的前 u 个」，u < ⌈size ÷ P⌉.max(1) 的部分就是尾部的洞：变长只让 size 变大，整份重写把 0 .. ⌈len ÷ P⌉ 全部写出（洞在这时被写成 0 的单元）。推翻它的现象：出现任何「在偏移 X 写」的入口（POSIX 面的 write/pwrite、reflink、punch）。

### 2.4 推荐形态的状态机（称「先落目标·条目」，W1/W2/W3 的推荐合起来就是它）

| 步 | 发布内容 | 读者此时读到 | 崩溃落在这一步之后、续做之前 |
|---|---|---|---|
| 意图那次（一次发布） | inode 记录 size ← N（blocks、改动计数、时间照 unlink 意图的写法）；logged_ops 签发一条截断条目（inode、对象出生代、N）；N 不是 P 的整数倍且尾单元的声明长度大于 t 时，重写尾单元 k（声明长度 t，补零）；extent 树只重写罩着 k 的下段叶到根的那条路 | N 字节的文件，内容是旧内容的前 N 字节；单元 k+1 .. u−1 还挂在树里但读不到（size 之外） | 截后（N 字节）。只读挂载读到 N 字节的文件，挂载续做把剩下的批走完 |
| 每一批（0 到多次） | 从尾部放 K 个单元（`next_truncation_step`），只重写被截的那片下段叶到根；inode 记录不动 | 同上，一个字节不变 | 同上 |
| 最后一批（一次发布） | 放掉剩下的 u − keep 个单元（可以是 0），同一次发布删掉那条 logged_ops 条目（根指针置零） | N 字节的文件，树里没有 size 之外的单元 | 截后 |

意图那次之前崩 → 截前（S 字节，原样）。没有第三种读法：这是「先落目标」比「随批降」多买的东西（见 W2）。

### 2.5 推荐形态新写出的字节（照 `.claude/kb/layout/03-third-txn.md`「一、形态」那张表的列：形态 / 第一次出现 / 写成什么 / 决策分项 / 里程碑那一步；数是推的，没量）

| 形态 | 第一次出现在哪次发布 | 写成什么 | 决策分项 | 里程碑那一步 |
|---|---|---|---|---|
| 截断条目（logged_ops 种类码 2） | 意图那次；最后一批删 | 条目 44 字节 = key 10（种类 2 字节、取值 2，签发 txg 8 字节）+ value 34（inode 号 8、对象出生代 8、目标 size 8、补零 10，非零拒收）；宽仍是 `LOGGED_OPERATIONS_ENTRY_BYTES` 的 44，一个码 2 根装 369 条 | D26 已定项 2 的字段表加一张；D8 已定项 5 | 六·2 |
| logged_ops 根（码 2，树 ID 65547） | 意图那次（零根变一条）；最后一批根指针置零，不写它的节点 | 一个 16 KiB 码 2 根兼叶，条目数 1；与整理意图同树同根，两种条目并存时按 (种类, 签发 txg) 排序 | D8 已定项 5；`.claude/kb/layout/03-third-txn.md` 的「树表四条新条目」那一行 | 六·2 |
| inode 记录的 size / blocks / 改动计数 / 时间 | 意图那次（只这一次写记录） | 偏移 40 size = N，偏移 48 blocks = ⌈N ÷ 512⌉，偏移 88 改动计数 = 这次发布的 checkpoint_txg，偏移 64 起三个秒字段取调用方给的值；其余偏移照抄 | D8 已定项 6（blocks）、I-9.15 | 六·2 |
| 尾单元 k 新版（仅 t < 旧声明长度时） | 意图那次 | 码 1 数据单元 32768 字节：头 105 + 预留 29，声明长度 = t，载荷 = 旧载荷前 t 字节，之后补零到单元末尾（I-2.3），整单元校验和；五元组 (extent 树, inode, 对象出生代, 锚点偏移 = k × P) 不变，写序是这次发布的 (实例, 事务号)，诞生代号是这次的 txg | D4 已定项 2、D16 已定项 5、D19 已定项 6 | 六·2 |
| extent 树节点 | 意图那次（罩着 k 的下段叶到上段叶的路）；每一批（被截那片叶到根） | 照 `plan_the_extent_tree_after_a_deletion_batch` 的写法：整个落在留下那段里的节点原样照抄，别的重写；下段只剩一个单元时内联进上段叶条目（标签 2） | D8 已定项 14、已定项 16 第 2 条（K-R3-1） | 六·2、六·3 |
| 记账、分配记录、中央映射、树表、journal 记录 | 每次发布 | 照现有发布路径，不为截断新增字段 | D5、D3 | 六·2 |

## W1　意图载体与目标态

**判定**：规则没说 + 两句并存。D8 已定项 5 定了载体：「凡是可能修改无界数量 extent 的操作（truncate、collapse、删大目录），先把意图写进 logged_ops 树，再分批推进」（`.claude/kb/decisions/08-核心索引结构.md:108`），并把 truncate 归「承诺类」：「意图分两类：承诺类（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空」（`.claude/kb/decisions/08-核心索引结构.md:108`）。D8 已定项 16 为 unlink 开了例外：「不立 logged_ops（K-U1 L4）」（`.claude/kb/decisions/08-核心索引结构.md:481`）。截断的字段表没有条款（logged_ops 的字段表只有整理一张：「合计 44 字节：key 10、value 34 定宽，一个码 2 节点装 369 条；宽不等于 44 的条目一律拒收。」`.claude/kb/decisions/26-后台整理与放置回收.md:66`；种类码只登记了「码 1 = 整理；0 与别的码保留，读者拒收」`.claude/kb/decisions/26-后台整理与放置回收.md:57`）。两句各自成立，选哪句要用户定（岔路 W1-1）。

### 候选

| 候选 | 定义 | 要动 `crates/` 的哪几处（文件@步骤） | 新写出的字节 | 代价 / 风险 |
|---|---|---|---|---|
| W1-条目 | logged_ops 种类码 2 = 截断；value = inode 号 8、对象出生代 8、目标 size 8、补零 10。与整理同一张码 2 树、同一个 44 字节条目宽 | `crates/singlefs-core/src/logged_operations.rs`：常量旁加种类码（`:23`）、`LoggedOperationKind` 加成员与 `of_code`（`:52` 的 `_reserved_kind_code => None` 之前）、`LoggedOperation` 加成员、`to_bytes` / `parse` 按种类解 value、拒收原因加「补零非零」、旁边加 `live_truncation_intent_of_the_entries`（对照 `:354`）。`crates/singlefs-core/src/transaction.rs`：`LoggedOperationsStepOfThePublish`（`:4622`）加签发 / 删两个成员、`logged_operations_after_this_publish`（`:4638`）加两臂、签发时断言全池至多一条（照整理那条）。其余见 W2 的入口与走空 | 条目 44 字节；logged_ops 根 16 KiB（意图那次从零根变一条，最后一批根指针置零）；中央映射里这个根的一条 | 额外多写：意图那次一个根节点及其映射路径（推的）；两个封闭枚举加成员（〇节第 5 点的行数）；GPU 扫描内核与 CPU 判器要认新种类码（W7） |
| W1-记录 | 不立条目：意图那次把记录的 size 写成 N，「还没做完」= extent 树里的单元数多于 ⌈size ÷ P⌉.max(1)；队列由状态推出（今天只有 inode 1 有数据，推导是 O(1)） | `crates/singlefs-core/src/transaction.rs`：走空来源加一项（`walk_the_deletion_queue_until_it_is_empty`，`:8236` 起）；`crates/singlefs-core/src/mounted_read.rs`、`crates/singlefs-core/src/recovery.rs` 的条数判改成容忍多出的尾部（同 W2） | 0 字节（只有记录与 extent 树，没有条目、没有根） | 不能区分「在做」与「泄漏」：多出的单元谁也不认领就永远在树里，checker 的「有账」判不出红；写死 inode 1（同 `crates/singlefs-core/src/transaction.rs:8166` 的「// 第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批。」那个债）；字面违反 D8 已定项 5 定案里先写进 logged_ops 的那句 |
| W1-新树 | 记录推导加一棵派生队列树（同 `deleted_inodes`：key = inode 号 8 字节） | W1-记录的全部，再加树表种类码、`FileVersionTreeIdentifiers` 第 14 棵树、恢复 / 走读 / 回退 / GPU 内核各处认这棵树（`deleted_inodes` 在 `crates/singlefs-core/src` 下 10 个 `.rs` 文件里出现，`grep -l` 数的） | 8 字节 / 号的 key 加一个 16 KiB 根 | 比 W1-条目大一圈，换来的只是 O(1) 的队列；目标 size 仍然要从记录读，条目带来的「目标与记录互核」没有 |
| W1-预留位（弃） | 把目标 size 写进 inode 记录的预留字节 | `crates/singlefs-core/src/records.rs` 与 I-9.7、checker、GPU 内核 | 记录里 8 字节 | 「预留 20 不承重：记录内任何赋义都是 incompat」（`.claude/kb/decisions/08-核心索引结构.md:199`）；队列仍要另找；每个 inode 都背 8 字节。只为完整列出 |

### 子问题逐条

1. **任何中间态都读得出「没做完」与目标**：W1-条目靠条目（目标）加「树里单元数 > keep(N)」（还剩多少）；W1-记录靠记录的 size 加同一个差。两者在崩溃后重挂载都读得出。W1-条目多一样可核的东西：条目里的目标 = 记录的 size（W7 必红世界）。
2. **与 unlink 队列、整理意图并存谁先走**：走空的次序今天是「`deleted_inodes` 先走空，再走销毁中的快照（树号升序）」（`crates/singlefs-core/src/transaction.rs:8245`）；整理在两者之后（`// 次序（判决 G8）：删除续做在整理批之前。` `crates/singlefs-core/src/mounted_session.rs:1999`）。推荐把截断放在销毁中的快照之后、整理之前：删除类的东西只会让空间变多，且 D8 已定项 16 第 6 条的「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」（`.claude/kb/decisions/08-核心索引结构.md:486`）对三种来源同样成立。三者互不交叠的前提：一次用户改动之前队列先走空，所以同一时刻只有一种来源有活意图（会话里）；崩溃后重挂载可能同时见到几种（unlink 意图 + 截断意图不可能，因为发意图之前队列已走空——`"发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）"` `crates/singlefs-core/src/transaction.rs:8100`——但回退可以把它们带回，见岔路 W1-3）。
3. **续做入口放哪**：不新建入口，进共用走空。「队列空」的判断有四处要一起改，前三处是走空的守卫、第四处是准入：可写挂载续做 `crates/singlefs-core/src/mount.rs:6574`（`if current_version_of_trees.deleted_inodes.is_empty()` 起的早退）、会话里每次用户改动之前 `crates/singlefs-core/src/mounted_session.rs:1388`、整理开跑之前 `crates/singlefs-core/src/mounted_session.rs:2003`。建议在 `TransactionOutput` 上加一个「队列是否为空」的方法，这四处各改成调它（第四处：「Some(version) if !version.deleted_inodes.is_empty() => {」（`crates/singlefs-core/src/admission.rs:448`）是可写挂载准入那一判，队列不空就把环窗口算作可动用；有活截断条目时同样要成立，否则续做会被空间准入拦在挂载里。顺带看到、不归这一轮：这一处只看 `deleted_inodes`，不看 `snapshots_being_destroyed`，删快照没做完时是否该同样放开，我没核）（读码推的；现在三处各抄一份「两个字段都空」的判断，漏改一处就是「平时不续做」那类路径，C1 欠的就是它：`| C1 | 意图续做做成常规路径 |` `.claude/kb/checks-owed.md:758`）。
4. **一个 inode 同时一条还是可叠**：推荐全池至多一条活截断条目（照「全池活整理意图至多一条」和 `deleted_inodes` 至多一条 key）。会话里和续做里不会有第二条：每次用户改动先走空。叠加只可能来自回退（岔路 W1-3）。「截到更短的第二次截断落在第一次没做完时怎么合」因此在会话里不可达；意图的语义是目标态，同一 inode 上两条目标不同的条目没有合法的合并法（先收缩再变长不可交换），所以必须禁止而不是合并：发第二条的入口断言上一版没有截断条目，checker 判「至多一条」。
5. **对已在删除中（nlink = 0）的 inode 截断拒成什么**：会话里走空之后 nlink = 0 的记录已经不在（走空把它们删完），拒绝落在「没有这条记录」那一格。直接调发布入口时新加一个类型化错误，形状照 `UnlinkOfAnInodeThatIsNotALiveRecord`（`crates/singlefs-core/src/transaction.rs:8106` 的 `find(... nlink != NLINK_OF_AN_INODE_BEING_DELETED)` 一行就是 unlink 那边的同一判），在任何落盘之前返回。`UserChangeRefused::NoFileVersionToChange`（`crates/singlefs-core/src/mounted_session.rs:220`）已经罩着「删完之后」那一格，不用新增。
6. **N 差不到一个单元、一个单元也不放时要不要意图**：见岔路 W1-2。我的推荐是单一路径：放 0 个单元也走「意图 + 最后一批」，最后一批放 0 个（unlink 已有先例：`FinalBatch { data_units_released: 0 }` 可以是 0，`UnlinkStep::FinalBatch` 文档「可以是 0：没有数据单元的文件只有这一批」，`crates/singlefs-core/src/unlink.rs:125`）。依据：`.claude/rules/fs-design.md` 的「判据写不出来时，先只做一条，而且先做慢而正确的那条」（`.claude/rules/fs-design.md:130`）；快路是第二条分支，要按「每条分支必须能被测试强制进入」（`.claude/rules/fs-design.md:122`）加开关、加崩溃流。代价：放 0 个单元的小截断多一次发布（推的，约一次发布的写入量；unlink 的各次发布是 11–13 个单元，`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:18`）。

### 推荐与依据

推荐 **W1-条目**。依据：① D8 已定项 5 定案逐字要求；② 树、根角色、「删意图时根被换下」的路径整理已经写好（`        // 删光意图之后的 logged_ops 根（意图达成或丢掉的那一次）：这一版没有它，上一版的那一个照样换下、经映射释放。` `crates/singlefs-core/src/transaction.rs:6118`），截断只是第二个写者；③ 44 字节条目里 value 34 够装 24 字节，不改宽、不改 `format-const`；④ 唯一让「意图」和「记录」互相对账的候选。代价写在表里；W1-记录是唯一零字节的候选，要用户放弃 D8 已定项 5 定案那句才选得了。

推翻它的现象：W1-1 的计数模型显示条目路线在回退历史上泄漏或与记录失配（W1-3，回退对 logged_ops 取「随现行」被用户定死时就会这样）；或用户不愿再动 GPU 扫描内核（整理意图那一段从 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:5621` 起共 123 行，种类码写死为 1：`const G_LOGGED_OPERATION_KIND_COMPACTION: u32 = 1u;` `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:5554`，新种类码要在那里加分支，或走 W7-1 的"内核不置位"先例）。

### 岔路（候选 / 翻面观测 / 够判条件，不带倾向）

| 编号 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W1-1 载体 | 条目（logged_ops 种类 2）/ 记录推导（零字节，违 D8 已定项 5 定案）/ 新派生树 | ① 每次截断多写的字节数（条目：意图那次多一个 16 KiB 根及其映射路径，推的）；② 回退跨过未完成截断的历史上「有账」判红的次数 | 一个计数模型：同一批放开用户动作的历史上，三个候选各一个多写字节的绝对值与「有账」违例数；模型钉绝对值、带变异表（形态照 E109 / E110），目前没有这个数，是缺口 |
| W1-2 零放走快路 | 单一路径（放 0 也走最后一批）/ 加快路（放 0 个单元合并成一次发布，要判据、开关、崩溃流） | 用户负载里「放 0 个单元的截断」占截断的比例；快路省下的发布数 × 每次发布的写入量 | 一个负载取样（截断前后单元数分布）加产品路径上单次发布的写入量绝对值；比例低于用户愿意养第二条分支的线时不开快路（线由用户定） |
| W1-3 回退 | 回退取 R_old 的 logged_ops（条目与记录同一版）/ 随现行（条目可能与回退后的记录失配）/ 回退时丢截断条目（多出的单元泄漏） | 放开用户动作的历史里，回退落在意图与最后一批之间时 checker 红的次数和泄漏的单元数 | 该历史上三个候选各一个违例数；C621 已记同一题给整理（`| C621 | 整理落地时要动的删除条款没有清单外的检查 |` `.claude/kb/checks-owed.md:415`），两件事合在一轮定 |

## W2　分批与最后一批同事务

**判定**：条款可以直接照写在截断上（一致）：「意图声明目标状态，绝不声明增量。」（`.claude/kb/decisions/08-核心索引结构.md:122`）、「中间状态必须每一个都是合法的文件系统状态」（`.claude/kb/decisions/08-核心索引结构.md:124`）、「一批里每个动作施加时先核它要删的东西还在」（`.claude/kb/decisions/08-核心索引结构.md:125`）、「删除意图必须与最后一批工作在同一个事务里提交」（`.claude/kb/decisions/08-核心索引结构.md:120`）。规则没说两处：批与批之间记录的 size 取什么；批大小开关共用不共用。

### 候选（批间 size 与读者看到什么）

| 候选 | 定义 | 要动 `crates/` 的哪几处 | 每次发布新写出的字节 | 读者在中间态答什么 |
|---|---|---|---|---|
| W2-先落目标 | 意图那次 size ← N 并重写尾单元；每批只放尾部 K 个单元，不碰记录；最后一批放完剩下的并删条目 | 新文件 `crates/singlefs-core/src/truncate.rs`（纯计划，对应 `crates/singlefs-core/src/unlink.rs` 的 `next_unlink_step`，`:133`；extent 计划对应 `plan_the_extent_tree_after_a_deletion_batch`，那里 `let is_entirely_kept = |position: &ExtentLowerNodePosition| {` `crates/singlefs-core/src/unlink.rs:191` 要改成「整个落在 min(留下的, 被重写的尾单元) 之内才照抄」）；`crates/singlefs-core/src/transaction.rs`：`DeletionStepOfThePublish`（`:4534`）加三个成员、`ordinary_allocation_part`（`:6160`）的 inode 记录写入（只意图那次）、extent 计划（`let extent_tree = match (deletion_and_lineage.step, previous) {` `:6204`）、`logged_operations`（`let logged_operations = deletion_and_lineage` `:6315`）三处各加臂；`carried_file_version_units`（`:10439`）加「照抄 [0, k) 与 (k, u)、第 k 个换新」的一臂；`roles_replaced_via_mapping`（`:3632`）的判别改成显式参数（W3）；读者三处容忍多出的尾部单元（`crates/singlefs-core/src/mounted_read.rs:815`、`crates/singlefs-core/src/recovery.rs:5381` 及每单元那一判 `:5440`） | 意图那次：inode 叶容器 + inode 根 + 条目根 + 尾单元（若重写）+ 罩着 k 的 extent 路径 + 固定点；每批、最后一批：被截那片下段叶到根的路 + 上段叶到根的路 + 固定点（没有 inode 叶与根） | N 字节的文件，size 之外的单元读不到（`if end_exclusive > file_size_in_bytes {` `crates/singlefs-core/src/mounted_read.rs:916` 的越界拒）。崩溃后只读挂载读到的只有「截前 / 截后」 |
| W2-随批降 | 照 unlink：意图那次不改 size；每批把 size 写成「留下单元数 × P」（`size: size_after_the_batch_in_bytes,` `crates/singlefs-core/src/transaction.rs:8181`）；最后一批写 N 并重写尾单元（W3-终批重写） | 同上，但读者不用改（每个中间态单元数都 = ⌈size ÷ P⌉）；每批要重写 inode 记录（`publish_unlink_step` 的那一支，`crates/singlefs-core/src/transaction.rs:8147`）；批不带时间参数，写入时间照抄上一版（同 `crates/singlefs-core/src/transaction.rs:8164` 的做法） | 每批多一个 inode 叶容器（32 KiB）和 inode 根（16 KiB）；意图那次不写尾单元；最后一批写尾单元 | 批间崩溃读到的是前缀（size = 留下单元数 × P，内容是旧内容的前缀）：允许集合是 {截前, 每批之后的前缀, 截后}，其中前缀的个数取决于批大小 K |
| W2-边界先落（弃） | 意图那次把 size 落到 ⌈N ÷ P⌉ × P，最后一批才落到 N 并重写尾单元 | 两者的读者改动加起来 | 两者之和 | 既要读者容忍多出的单元，又有一个批间才存在的 size；被前两个各自的缺点叠加，只为完整列出 |

### 推荐与依据

推荐 **W2-先落目标**。依据（都是推的，没跑）：
1. **崩溃后读到什么是两元集合**，RefFS 与多版本 oracle 都不用知道批大小。RefFS 里现成的形状：删快照一次调用写多条根，注释写「前后两份快照集合都可能」（`crates/singlefs-reference-filesystem/src/lib.rs:231`，对应的函数在 `EnteredKind::DestroySnapshot => true,` 那一臂，`crates/singlefs-reference-filesystem/src/lib.rs:235`）；崩溃点重放的版本表只放有文件内容变化的版本，批不变内容就不进表（`pub struct CrashPointReplayVersionTable<'table> {` `crates/singlefs-checker-tier/src/crash.rs:785`，现状见 unlink 流「两批与最后一批不进表、照比它旧的最新那一项…判」`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:6`）。W2-随批降则要求每批进表、RefFS 允许集合带上 K。
2. **批不碰 inode 记录**：第 7 条①b「同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」（`.claude/kb/decisions/08-核心索引结构.md:487`）里 unlink 为批里的写入时间专门定过「各批照抄」；先落目标的批根本没有时间要写，续做的几批因此不依赖任何调用方给的量。
3. **每批少写两个单元**（inode 叶、根），C2 的斜率检查（`| C2 | 无界操作做成单事务 |` `.claude/kb/checks-owed.md:19`）更容易钉。
代价：读者三处要容忍「记录条数多于 ⌈size ÷ P⌉」（W2-随批降零改动）；池级 checker 的 I-1.13 要在遍历一侧也按 size 滤掉多出的单元（W6）。

推翻它的现象：用户要求只读挂载在批间崩溃之后必须读到「某个中间 size」（POSIX 的 truncate 在崩溃时并不承诺原子，所以这条要求不是必然的）；或读者容忍多出单元这件事在 W4/W6 里被证明会放过「泄漏」（即 W1-记录 那种没条目的形态，W1-条目 不受影响，因为 checker 会要求有条目）。

### 子问题逐条

- **批大小共用还是另立**：共用 `DeletionSwitches.batch_size`（`crates/singlefs-core/src/unlink.rs:28` 起的开关组，产品取 144，测试压小到 1..=144）。先例：删快照已经共用，注释写「删快照照 unlink 的过程做（第七节岔路 4），两件事的批同一个大小，开关不另立一个。」（`crates/singlefs-core/src/unlink.rs:107`）。另立要给 `DeletionSwitches` 加字段，所有构造点（`DeletionSwitches::PRODUCT` 与 `mount_writable_with_test_only_switches_and_deletion_switches` 链）跟着改，没有换来东西。
- **最后一批与截断意图同一次发布，落在哪**：`ordinary_allocation_part` 里三处同时有臂：extent 计划（`(DeletionStepOfThePublish::Batch { .. }, Some(previous_version)) => {` `crates/singlefs-core/src/transaction.rs:6205`，最后一批要新增一臂：单元放到 keep、尾部整段换下）、logged_ops（`let logged_operations = deletion_and_lineage` `crates/singlefs-core/src/transaction.rs:6315`，删条目那一臂；`        let logged_operations_root = if logged_operations.is_empty() {` `:6318` 随之置 `NoRoot`）、被换下的根（`        // 删光意图之后的 logged_ops 根（意图达成或丢掉的那一次）：这一版没有它，上一版的那一个照样换下、经映射释放。` `crates/singlefs-core/src/transaction.rs:6118`，已有，不用改）。三处在同一个函数里，所以「最后一批与删意图同一个事务」是一次 `resolve` 的产物，改坏其中任何一处（W9 的必红世界）都在同一个测试面上红。
- **续做的幂等判据，D8 已定项 16 第 7 条五条逐条写在截断上**（`.claude/kb/decisions/08-核心索引结构.md:487`）：① 计划相等——下一步只由「现行那一版的单元数 u、条目里的目标 N、批大小」定：`u − keep(N) ≤ K` 是最后一批，否则留下 `u − K` 个；纯函数，对应 `crates/singlefs-core/src/unlink.rs:133` 的 `next_unlink_step`。①b 两份盘逐字节相同——先落目标的批不带任何调用方给的量（记录不动），成立；意图那次的时间秒由调用方给，同 unlink。② 删完再续做一个写都不发——条目没了、u = keep，早退（三处守卫）。③ 每个写点截断、重开续做到底，终态逻辑内容与不崩溃的终态相同、每个崩溃态 checker 全绿——要新崩溃流（W9）；「逻辑内容」那串清单要加一项：logged_ops 条目多重集。④ 记账标签 4、6 等于现算——标签 4 恒 0（待删占用在「已分配」里，`| I-3.4 | 可用空间扣待删占用 |` 所在行 `crates/singlefs-checker/invariants/05-空间记账.md:13`），标签 6 的式子按墓碑与退役记录数算，截断不产生它们（若 W5/W6 选区间墓碑围栏则变，见 W6 岔路）。
- **「核它要删的东西还在」**（第 4 条）：批的计划每次从现行那一版重算、不缓存（`.claude/kb/decisions/08-核心索引结构.md:486` 的「会话里每一次用户改动……先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」同一原则），释放路径对每个单元核三样：在册、没释放过、跨度对得上（`placements_to_release_via_mapping` 的错误清单，`crates/singlefs-core/src/transaction.rs:3535` 起的循环）。
- **一批写入量不随文件大小增长**：先落目标的批 = unlink 的批去掉 inode 叶与根（读码推的），所以 unlink 那份斜率测试（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs`，快档 1–433 个单元、慢档 2304 与 4608）复制成截断的一份，再加一列「截断的批 ≤ 同尺寸 unlink 的批」。C2 那一行留着两个没答的问（「斜率为 0」是有界读还是最大值不随规模变）：我没替它们定，数是 31 / 45 / 53 / 55 / 73 次（1 / 2304 / 4608 / 4609 / 9216 个单元，攻方一次抽样，`.claude/kb/checks-owed.md:19`）——这组数随单元数涨，所以「斜率为 0」今天并不字面成立；这一格欠的是口径，不是截断独有。

### 准入类（岔路 W2-2）

截断各次发布算不算「删除类发布」（不判式子那一判、落点那一判照判）：今天是写死的穷举匹配（`    let demand_is_judged = match (allocator.space_admission(), deletion_and_lineage.step) {` `crates/singlefs-core/src/transaction.rs:8817`），上方注释「删除类发布（意图、各批、最后一批）不判式子那一判、落点那一判照判（D28（挂载期承诺量） 已定项 1：它们的空间罩在环窗口里）。」（`crates/singlefs-core/src/transaction.rs:8813`）。冲突点：D28 的 W 上界写明「这个上界在删除批只改尾叶时成立（D8（核心索引结构） 已定项 16）」（`.claude/kb/decisions/28-挂载期承诺量.md:29`），而截断意图那次重写一个数据单元加 inode 叶与根加条目根，不是「只改尾叶」。W 里「分配记录树之外那一项」24 = 「普通分配 16 + 固定点根 4 + 映射树根以下 4」（`crates/singlefs-core/src/admission.rs:383`）；截断意图那次的普通分配我按 2（数据单元）+ 下段路 + 上段路 + 2（inode 叶）+ 1（inode 根）+ 1（条目根）数，下段高 2、上段高 1 时是 9 槽（下段高 3 时 10 槽；推的，没算进测试），低于 16——但这个 16 的构成我没有拆开读，所以只能说「很可能罩得住」。

| 候选 | 定义 | 要动的处 | 代价 |
|---|---|---|---|
| W2-2a 当删除类 | 截断三种发布走「不判式子」那一臂，空间由环窗口罩；被落点拒时照 unlink 推抬 F | `crates/singlefs-core/src/transaction.rs:8817` 的匹配加臂；`crates/singlefs-core/src/mounted_session.rs:1028` 的 `refusal_is_short_of_space` 与挂载续做的抬 F 路径共用，不改；`crates/singlefs-core/src/admission.rs` 加一个单测钉「截断意图那次每盘新占槽 ≤ 24」 | W 上界要重证一遍（D28 已定项 1 的正文句要改），被攻过的「删除类不判式子」结论要把截断算进去 |
| W2-2b 当普通写 | 截断意图判式子（需求 = 这次新写的全部角色），被拒时用户得到空间不足 | 同一匹配里归 `NotADeletion` 那一臂 | 释放空间的操作可能被空间拒：与「释放空间这个操作本身不需要申请空间」（`作为要求原样成立` 所在行 `.claude/kb/decisions/26-后台整理与放置回收.md:87`）冲突 |

## W3　截断点落在单元中间：尾单元怎么 COW

**判定**：条款预见了尾单元重写，没写何时重写（规则没说）：「短 extent（小文件、尾块、被截断的 extent）补齐到整单元」（`.claude/kb/decisions/04-校验和位置.md:36`）；D19 已定项 6 把「截断的尾单元」列进「一个事务最多写一个单元的用户数据」的射程：「预分配、截断的尾单元、打洞两端、reflink 两端的拷贝都算」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:145`），并说「最好写进那一句」——D16 已定项 5 末段今天没有这半句（`.claude/kb/decisions/16-发布语义.md:155` 只有「一个事务最多写一个单元的用户数据」），属于 kb 要补的一处（我不写 kb）。实现有两处缺口（本报告 2.2 第 1、2 点），其中第 2 点是量的。

### 候选

| 候选 | 定义 | 能不能让每个中间态都不出现「读者判声明长度不符」 | 要动的处（`crates/` 文件@步骤） | 新写出的字节 |
|---|---|---|---|---|
| W3-意图重写 | 意图那次发布重写尾单元 k：声明长度 t，载荷 = 旧载荷前 t 字节，补零 | 能（与 W2-先落目标配）：意图之后每个中间态 size = N，k 的声明长度 = t = min(N − k·P, P) | `crates/singlefs-core/src/transaction.rs`：`FileVersionPlan` 旁加「只重写第 k 个单元」的输入（今天只有整份内容，`/// `content` 是整份文件内容（顺序写从偏移 0 写到末尾）` `:4440`）；`file_content_transactions`（`:6133`）对这个输入交一个事务；`carried_file_version_units`（`:10439`）加「照抄 [0, k)、第 k 个换新、照抄 (k, u)」一臂；`roles_replaced_via_mapping`（`:3632`）及它的三个调用点（`:3535`、`:3801`、`:4853`）去掉「含 Data(0) 就当整份重写」的推断，改成显式参数；`crates/singlefs-core/src/truncate.rs` 交「是否要重写、重写成几个字节」 | 一个 32768 字节数据单元（见 2.5 表）+ 罩着 k 的 extent 路径 |
| W3-终批重写 | 最后一批重写尾单元 k，意图与各批不碰它 | 只有与 W2-随批降配才能：中间态的 size = 留下单元数 × P，留下的单元都满载，声明长度 = P；终批之后 size = N、k = t | 同上，但尾单元重写落在最后一批的发布里（那一次已经有「放剩下的」「删条目」两样），`resolve` 的候选里 Data(k) 与尾部单元同次出现——这反而是今天判别的"自洽"形状 | 同一个单元，但落在终批；每批多写 inode 叶与根（W2） |
| W3-不重写 | 尾单元不动；读者的声明长度核对放宽成「声明长度 ≥ min(size − 起点, P)」，读时只拷 min | 能，但换了判据：核对本身变弱（`if u64::from(header.declared_length) != declared_length_expected {` `crates/singlefs-core/src/mounted_read.rs:1090`、`if u64::from(header.declared_length) != expected_payload_length {` `crates/singlefs-core/src/recovery.rs:5440` 要改成不等号），声明长度之后到单元末尾的 I-2.3 仍成立，但声明长度之内 size 之外还留着旧字节 | 读者两处；变长（W5）必须先把这个单元重写成零，否则旧字节重新露出（探针 D 量的是「只改 size 变长」读者拒；放宽之后它不拒，而是读出旧字节——读码推的，没跑） | 0 字节（截断时）；变长时再付一次尾单元重写 |

### 推荐与依据

推荐 **W3-意图重写**（与 W2-先落目标成对）。依据：
1. 探针 C 量的：尾单元不重写、只改 size，今天的 `read_at` 立刻拒（声明长度与 size 不符），池级 checker 一条不红——所以不重写就必须改读者，改读者就把「声明长度 = 文件该有的字节数」这条自检变成「≥」，还要在变长那头补一个重写（W3-不重写），代价只是挪了地方；
2. 重写在意图那次，意图发布之后的任何中间态（只读挂载、冷走读、模型读）都用同一把读者的尺子，不需要「有截断意图时放宽」这样的有条件判据（W3-终批重写配 W2-随批降同样不需要，但放弃了两元集合，见 W2）；
3. 旧载荷前 t 字节取自上一版的内存态（`fn carried_unit(previous: &TransactionOutput, identity: TransactionUnit) -> PublishedUnit {` `crates/singlefs-core/src/transaction.rs:9398`），可写挂载恢复时每个单元本来就读过一遍，不多一次读盘（读码推的）。

### 其余子问题

- **D16 已定项 5「一个事务最多写一个单元」**：意图那次写一个数据单元（或 0 个），一个事务；最后一批与各批 0 个。对齐 N（t = P）或 N 落在洞里时 0 个。（`.claude/kb/decisions/16-发布语义.md:155`）
- **I-2.3 补零**：重写后的单元声明长度 t，之后补零到单元末尾，校验和覆盖整个单元：「补齐字节非零、或校验和只覆盖声明长度，一律判红」（`crates/singlefs-checker/invariants/04-校验和.md:8`），成立。
- **`roles_replaced_via_mapping` 的判别怎么改、`:9068` 那条相等断言怎么守**：探针 F 量的缺口是：只重写 Data(0) 时换下清单把 Data(1)、Data(2) 一起算上（`roles.contains(&TransactionUnit::Data(DataUnitIndexInFile::FIRST));` `crates/singlefs-core/src/transaction.rs:3644`）。在先落目标里，意图那次的候选是 [Data(k)]、k = 0 时就是这个形状——不改会在意图那次把还挂在树里的单元 1 .. u−1 提前放掉（`(self.carried_data_units..previous.data_pointers.len())` 那条尾部清单本来是留给后面几批的，`crates/singlefs-core/src/transaction.rs:6091`）。改法：把「这次发布整份内容是不是从 0 重写」做成显式的，值来自 `resolve` 已经知道的 `self.file.is_some()`（`.previous_roles_without_a_successor(previous_version, self.file.is_some())` `crates/singlefs-core/src/transaction.rs:6374`，同一个量今天被用了一次），三个调用点各加一个参数。改完探针 F 第一行应当是 [Data(0)]；相等断言（`"resolve 判生死的那一组用户可见单元，就是释放判定路径按这次换下的全部角色走出来的用户可见那一组"` `crates/singlefs-core/src/transaction.rs:9068`）原样保留，它是改歪时的网（读码推的：断言两边用的是同一个展开函数，所以只有在三个调用点给的参数不一致时才红，三个调用点必须喂同一个值）。
- **截到 0**：留一个声明长度 0 的单元（重写 Data(0)，载荷空，补零 32634 字节）。依据：冷走读拒「一个数据单元都没有」（`what: "extent 树里第一个文件一个数据单元都没有",` `crates/singlefs-core/src/recovery.rs:4740`），写侧下界是 1（`pub fn data_unit_count_of_a_sequential_write(content_length_in_bytes: u64) -> u64 {` `crates/singlefs-core/src/write_request_split.rs:59` 的 `.max(1)`），覆盖写成空内容的现成用例（`fn a_file_overwritten_with_empty_content_still_opens_with_its_one_data_unit() {` `crates/singlefs-harness/tests/core_review_geometry_back_chain_and_empty_inode.rs:770`）钉着这个形状。没写过内容的 inode（标签 0，size 0，0 个单元）保持 0 个单元。
- **内联只罩单元 0，截到一个单元以内的 extent 树形态**：keep = 1 时下段整段换下、上段叶条目从标签 1（下段根）换成标签 2（内联）；`pub fn lower_segment_nodes_of_a_file_without_holes(` `crates/singlefs-core/src/extent_tree.rs:210` 对 ≤ 1 个单元交空。这就是 D8 已定项 14 射程点名要进崩溃点重放的「内联与下段互换」（`.claude/kb/decisions/08-核心索引结构.md:445`）。先落目标里它发生在最后一批；unlink 的批留下 1 个单元时已经走过一遍，但崩溃流是否罩到我没核（W9 的缩到 0 流罩它）。
- **诞生代号不再一致**：尾单元重写后，一个文件的数据单元出生 txg 不全相同（k 比 0 .. k−1 新）。模型今天按角色记一个出生 txg（`.get(&ModelUnitRole::Data)` `crates/singlefs-harness/src/model.rs:3020`，`.expect("换下数据单元的那一版有数据单元");` `:3021`），要改成按单元（W8）。

### 岔路

W3 在推荐形态下没有要用户定的岔路；唯一的选择（尾单元重写在意图那次还是终批）是 W2-批间 size 的推论。若 W2 的批间 size 交给用户（见 W2 的推荐），尾单元的时点跟着走。W3-不重写作为节省一次 32 KiB 数据单元写的路径，翻面观测是：用户负载里「截断点不对齐、且之后不变长」的占比（推的，没有数）。

## W4　快照下截断

**判定**：一致，条件成立的方式是复用，不是新判定。D5 已定项 13 的判定是「`birth > previous_snapshot_txg` 就直接释放，否则追加进活头自己的 deadlist」（`.claude/kb/decisions/05-快照-空间记账机制.md:302`），运行时形态在已定项 19：「elif birth > p: 立即释放」（`.claude/kb/decisions/05-快照-空间记账机制.md:422`）；今天删除批与覆盖写都经 `deadlist_changes_of_this_publish`（`crates/singlefs-core/src/transaction.rs:4841`）。

事实表第四节的六条逐条核（我自己读码，不抄）：
1. **被截的单元要出现在候选里**——要改：不写内容的发布靠 `carried_data_units` 交「留下几个」，`previous_roles_without_a_successor`（`(self.carried_data_units..previous.data_pointers.len())` `crates/singlefs-core/src/transaction.rs:6091`）才把 ≥ 它的 Data(i) 算作换下。截断的批要在 `DeletionStepOfThePublish`（`:4534`）或新枚举里交这个数；意图那次交 u（一个都不放）。
2. **只重写 Data(k) 的发布**——要改：W3 的缺口。
3. **两侧相等**——不改，是网（`:9068`）。
4. **「上一版有没有落点」按下标判**——成立不用改：洞只在尾部（本报告 2.3），单元恒是 0 .. u−1，`index < previous.data_pointers.len()` 照旧对（`fn has_a_placement_in_the_previous_version(` `crates/singlefs-core/src/transaction.rs:3667`）。
5. **映射 key 取上一版的**——成立：重写过的单元 k 被杀的是旧版那把 key（`mapping_key_carried_from`），新版 k 有自己的写序，不重号。
6. **下段被截那片叶与祖先在六类里**——成立：六类含 `ExtentLowerNode` 等（`pub const fn asks_the_snapshot_lifetime_judgement(identity: TransactionUnit) -> bool {` `crates/singlefs-core/src/transaction.rs:4991`），批的计划把它们交进 `replaced_previous_roles`。另有断言「一个单元只被杀一次」（`"一个单元只被杀一次：这次进 deadlist 的条目在上一版的 deadlist 里没有（它的映射 key 上一版还被这一版的树引用）"` `crates/singlefs-core/src/transaction.rs:4971`）：截断里每个单元至多被杀一次（k 在意图那次、k+1 .. 在各批），不会触发。

**快照读者 / 活头读者**：快照有自己的 inode 根与 extent 根（码 14 / 15 条目），截断改的是活头的两棵树；deadlist 点名的旧单元与旧叶的映射条目留存。现成用例的形状：`an_overwrite_under_a_snapshot_keeps_every_replaced_user_visible_unit_on_the_deadlist_and_releases_the_pool_metadata`（`crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:201`），截断照写一份（缩到单元中间、缩到 0 各一）。

**意图跨过建快照**：在会话里不可达。每次用户改动先把队列走空（`会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己` `.claude/kb/decisions/08-核心索引结构.md:486`），建快照和删快照也在内（`if !current_version_of_trees.deleted_inodes.is_empty()` `crates/singlefs-core/src/mounted_session.rs:1388` 的走空，与 `UserChange::CreateSnapshot`、`DestroySnapshot` 同一入口，读码推的）；W1 把截断加进走空之后，意图那次到最后一批之间活头的 `previous_snapshot_txg` 不变，所以「之后各批放的单元判给快照还是释放」由每个单元自己的出生 txg 对这个不变的 p 判，意图之前就定了。唯一能让「意图之后、最后一批之前有快照被建出来」的是回退把旧根带回（W1-3）。
**删快照与截断同在队列里时的次序**：同样只可能来自回退。走空的次序（`deleted_inodes` → 销毁中的快照 → 截断）两种次序都不出错：销毁意图把前驱 P 传给下一侧（`prev` 要跟着传递，`.claude/kb/decisions/05-快照-空间记账机制.md:305`），之后截断各批对新的 p 判，只被销毁中那个快照引用的单元判成直接释放，是对的；反过来先截断、后销毁，截断进 deadlist 的条目纪元落在该快照的区间里，销毁时按 [lo, hi) 放掉（`.claude/kb/decisions/05-快照-空间记账机制.md:278` 的「销毁 S」一行）。读码推的，没有跑。

**岔路**：无。推翻它的现象：W5/W6 选了需要「截断时写区间墓碑」的围栏——墓碑树不随快照滤，截断的墓碑是否要随 deadlist 的节奏回收，会重新打开 D18 已定项 10 的回收条款。

## W5　变长与洞

**判定**：冲突（三句话在「缩短之后再变长」这条历史上不相容，量的）。三句：D8 已定项 14 把洞定成格式上的缺席「洞就是缺席」（`.claude/kb/decisions/08-核心索引结构.md:419`）；D18 已定项 10 + D8 已定项 6 的读法 Z「文件变短不写新记录，扫描重建时数据单元按重建出的 inode 记录的对象出生代与 size 判死活」（`.claude/kb/decisions/18-块里携带什么信息.md:235`）和「truncate / punch 这类部分删除不写区间墓碑（区间墓碑不做）」（`.claude/kb/decisions/18-块里携带什么信息.md:235`）；用户 2026-10-16 定「缩短与变长（size 改大、中间留洞）都做」（正文第一节，`research/prompts/_m4-trunc-r1-body.md` 里的原话）。探针 A/B 量的：缩短放掉的单元还在盘上，size 拉大后读法 Z 的「⌈size ÷ P⌉ 之内都活」把它们认回来，I-1.13 在合法历史上判红。要围栏，选项在 W6 的岔路里；W5 其余部分不依赖围栏选哪个。里程碑页留着的问「截断变长（size 改大、留洞）进不进这一份」（`.claude/kb/milestone/04-fourth-txn.md:153`）已被用户答了「进」。

### 候选（变长这一次发布写什么）

| 候选 | 定义 | 要动 `crates/` 的哪几处 | 新写出的字节 | 读者要改什么 |
|---|---|---|---|---|
| W5-补零重写 | 变长时若最后一个存在的单元的声明长度小于 min(N − 起点, P)，把它重写成该长度（载荷补零）；之后的单元一律缺席。别的什么都不写。不立意图 | `crates/singlefs-core/src/transaction.rs`：与 W3 同一个「只重写第 k 个单元」输入；记录写入的一臂（`DeletionStepOfThePublish::Batch` 那一类带 `record_after` 的写法，`:4534`）；extent 计划走「重写一片下段叶的路」；不进走空队列。`crates/singlefs-core/src/truncate.rs` 的计划函数把「变长 / 缩短 / 同」统一成：`k = min(u, keep(N)) − 1`，当声明长度 ≠ min(N − k·P, P) 时重写 k | 最多一个 32768 字节数据单元 + 罩着 k 的 extent 路径 + inode 叶与根 + 固定点；最后一个存在的单元声明长度已是 P、或 u = 0（没写过内容的 inode）时 0 个数据单元；S = 0 但有一个声明长度 0 的单元时仍要重写它 | 只改一处：记录条数判由「等于 ⌈size ÷ P⌉.max(1)」改成「u ≤ 它」（`if extent_records != data_units_implied_by_the_file_size {` `crates/singlefs-core/src/mounted_read.rs:815`，`!= data_unit_count_of_a_sequential_write(inode_record.size)` `crates/singlefs-core/src/recovery.rs:5381`）。声明长度那两处判（`:1090`、`:5440`）一字不改——每个存在的单元仍然 = min(size − 起点, P) |
| W5-只改记录 | 变长只写 inode 记录（size、blocks、改动计数、时间），一个单元也不写；读者把「声明长度之后到 min(size − 起点, P)」读成 0（补齐字节恒 0 且被校验和罩住，I-2.3，所以这样读是真的） | 同上，少了单元与 extent 路径；读者改四处：条数判（同上）、声明长度判从「=」改成「≤」且只对最后一个存在的单元（`if u64::from(header.declared_length) != declared_length_expected {` `crates/singlefs-core/src/mounted_read.rs:1090`）、`read_at` 里按单元序号取记录的 `.expect` 改成缺席补零（`.get(usize::try_from(unit_number).expect("单元序号装得进 usize"))` `crates/singlefs-core/src/mounted_read.rs:940`）、拷贝的上界 `.min(unit_starts_at_file_byte + u64::from(header.declared_length))` 要多一段补零（`crates/singlefs-core/src/mounted_read.rs:970`） | 0 个数据单元；inode 叶与根 + 固定点 | 同左四处，加冷走读的声明长度判（`:5440`），共五处。池级 checker 要新判声明长度 ≤（W7） |
| W5-补零成单元（弃） | 变长把 [旧尾, N) 全写成真的零单元，不留洞 | 变长也要意图与分批（无界，C2） | 变长的跨度 / P 个单元 | 不用改读者；但违反用户「留洞」，也让 `洞就是缺席` 在写路径上永远不出现。只为完整列出，它同时是 W6 围栏的一个候选 |

### 推荐与依据

推荐 **W5-补零重写**。依据：
1. 每个存在的单元「声明长度 = min(size − 起点, P)」这条自检一字不改，今天的两份读者里它是唯一把「size 与单元对不上」拦在读之前的判（探针 C/D 量的：只改 size 就被它拦）；W5-只改记录要把它放成「≤」，放的方向正是漏检的方向；
2. 代价是尾单元声明长度不足 P 时变长多写一个单元，比 W5-只改记录贵 32 KiB 加一条 extent 路径（推的，没量），换来读者改动从五处降到一处；
3. 变长恒是有界操作（最多一个单元），不需要意图、不进队列，所以不引入新的续做路径。

推翻它的现象：用户的负载里变长非常频繁而且尾单元几乎总是不满（那时每次变长都写一个 32 KiB 单元），见岔路 W5-1。

### 子问题逐条

- **旧尾单元声明长度 < min(N − 起点, P) 时那条核对怎么改**：W5-补零重写下不改；W5-只改记录下，「声明长度之后、size 之内读出 0」就是洞里的 0，不是坏单元，判据要写成「声明长度 ≤ min(size − 起点, P)，且若小于则之后没有存在的单元」。
- **变长要不要意图**：不要。变长不放任何单元、最多写一个单元，一次发布写得完；D8 已定项 5 管的是「可能修改无界数量 extent」的操作（`.claude/kb/decisions/08-核心索引结构.md:108`），变长不在射程内。对应地，D8 已定项 16 第 2 条的「只有这一条路，不设一次做完的快路」（`.claude/kb/decisions/08-核心索引结构.md:482`）说的是 unlink，不外推到变长。入口上就有两条路（N > S 一次发布；N < S 意图 + 批），分支变量是 N 与 S 的大小关系，操作期间不变，测试用不同的 N 就能强制进入；运行时可观测（交回的版本数）。
- **读洞答 0 的路径**：今天两条读路径都拒洞（探针 E：`open_file` 报 `ExtentRecordCountDoesNotMatchTheFileSize`）。要改的是 `open_file` 的条数判（`crates/singlefs-core/src/mounted_read.rs:815`）、`read_at` 的取记录（`crates/singlefs-core/src/mounted_read.rs:940`）：缺席的单元号补零，不读盘；`OpenFileForRead::data_unit_count`（`crates/singlefs-core/src/mounted_read.rs:887`）报存在的单元数，文档要写明它不再等于 ⌈size ÷ P⌉。上段叶条目标签 0（没有单元）的 inode 配 size > 0 就是整个文件都是洞：`None | Some(ExtentUpperLeafTarget::NoDataUnit) => DataUnitsOfTheInode::NoneWritten,`（`crates/singlefs-core/src/mounted_read.rs:740`）之后 `data_unit_count_implied_by_the_file_size` 现在对「没写过 + size ≠ 0」照除法给数、与 0 条记录对不上（`pub fn data_unit_count_implied_by_the_file_size(` `crates/singlefs-core/src/write_request_split.rs:78`），要改成上界。
- **`TransactionOutput::data_pointers` 按下标当单元号的内存态怎么表达洞**：不用拆。洞恒在尾部（本报告 2.3），`data_pointers` 的第 i 项仍是单元 i，`data_pointers.len()` 是 u 而不是 ⌈size ÷ P⌉。需要核的是「等于」的隐含：我读到并认为要复核的调用点是 `previous.data_pointers.len()` 被当成文件单元数用的地方——`publish_unlink_step`（`// 第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批。` `crates/singlefs-core/src/transaction.rs:8166`）拿它当剩余单元数，对有洞文件仍对（放的就是存在的单元），但 unlink 每批把 size 写成「剩下单元数 × P」（`size: size_after_the_batch_in_bytes,` `crates/singlefs-core/src/transaction.rs:8181`）：有洞文件的第一批会把 size 从 S 降到 kept × P，这是合法的收缩，不是 bug。
- **洞之后又整份覆盖写**：整份内容从 0 写到末尾，洞被写成 0 的单元（写入口没有稀疏检测）。保持这样：跳过全零单元会在文件中间制造洞，破坏「洞只在尾部」这条让 `data_pointers` 不用拆的前提。
- **变长之后再缩回洞里（N 落在洞中）**：统一规则 `k = min(u, keep(N)) − 1`，尾单元重写当且仅当其声明长度 ≠ min(N − k·P, P)；N 落在洞里时 k = u − 1，声明长度已是 P（W5-补零重写下，变长时补满了），不重写，只改记录。N 落在最后一个存在的单元内部时按 W3 重写。
- **I-9.15 blocks 对洞成立**：blocks 是逻辑长度的 512 字节块数，「不表示分到的空间」（`.claude/kb/decisions/08-核心索引结构.md:197`），洞不改这条；`st_blocks` 挪到 POSIX 面的里程碑，不在这一轮。
- **C711 在有洞时换成什么**：C711 现在的叙述「每条 inode 记录在 extent 树里的数据单元数与它的 size 对得上」（`.claude/kb/checks-owed.md:475`）要从「等于」换成三条：存在的单元从 0 起连号、u ≤ keep(size)、每个存在的单元声明长度 = min(size − 起点, P)；"u < keep(size)" 本身合法（洞）。这条换法会放掉一类今天能抓到的错——尾部单元被错放——因为洞合法之后「少几个单元」和「洞」看起来一样。若不想放掉，岔路 W5-2。

### 岔路

| 编号 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W5-1 变长写不写尾单元 | 补零重写（变长至多写一个单元）/ 只改记录（0 个单元，读者放宽三处） | 用户负载里变长的频率和 S 不是 P 整数倍的占比；两条路各自的写入字节数 | 两条路在同一批历史上各一个绝对写入字节数（计数模型，钉绝对值、带变异表）；读者放宽的方案还要一个「放宽之后能漏掉的错」的必红世界清单（目前我只想到「尾部单元被错放」一类） |
| W5-2 洞要不要声明 | 隐含（u < ⌈size ÷ P⌉ 就是洞，丢单元与洞不可分）/ 上段叶条目加标签声明洞（`pub const EXTENT_UPPER_LEAF_ENTRY_TAG_INLINE_DATA_UNIT: u8 = 2;` `crates/singlefs-core/src/extent_tree.rs:47` 旁加两个标签；变长要多重写上段叶那条路，读者与 checker 认标签） | 池级 checker 现在靠「单元数 ≠ ⌈size ÷ P⌉」抓到的错（C711 补完之后）有多少类，换成洞合法后掉几类 | 一份故障注入清单：对「尾部单元被错放」「批多放了单元」「最后一批漏放」各注入一次，看 W5-2 两个候选各红几条；这件事没有数，是缺口 |

## W6　扫描重建与读法 Z

**判定**：冲突，两处。
(a) kb 内部：读法 Z 在 D8 里的原话是「按重建出的 inode 记录的对象出生代与 size 折成的单元数判，文件变短不写新记录（读法 Z）」（`.claude/kb/decisions/08-核心索引结构.md:203`），I-1.9 说「其外的可读垃圾每个 key 都被更大写序的可见版本或可见墓碑压住」（`crates/singlefs-checker/invariants/01-块自描述.md:14`）。缩短放掉的单元既没有更大写序的版本也没有墓碑，I-1.9（今天未实现）一旦实现就在每个缩过的文件上红，除非把「被 size 判死」写成第三种压住。
(b) 读法 Z 与洞（W5）：见探针 B。

### 今天的行为（读码 + 量的）

`crates/singlefs-checker/src/rebuild.rs` 对每个当前版本的数据单元：有同出生代的 inode 记录时按 `data_units_the_size_covers = ⌈size ÷ P⌉.max(1)` 判（`let data_units_the_size_covers = read_u64(record, INODE_RECORD_SIZE_OFFSET)` `crates/singlefs-checker/src/rebuild.rs:743`），偏移 ÷ P 不小于它就报 `ScanRebuildNotDecided`（`if offset_in_bytes / payload_capacity_in_bytes >= data_units_the_size_covers {` `crates/singlefs-checker/src/rebuild.rs:746`），I-1.13 因此报不适用（探针 A 量的）。整对象墓碑压着的只收整对象（`ScanRebuildNotDecided::TombstoneRangeOtherThanTheWholeObjectOfTheFirstVersion,` `crates/singlefs-checker/src/rebuild.rs:666`）。重建目前只有池级 checker 一个调用方（grep `rebuild_index_tables` / `scan_rebuild` 在 `crates/` 非测试代码里只命中 `crates/singlefs-checker/src/walk.rs` 5 行、`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 17 行，量的）：D18 已定项 5 的「级 1 只读挂载」还没有读重建出的索引的实现。

### 候选（Z 怎么落）

| 候选 | 定义 | 要动 `crates/` 的哪几处 | 新写出的字节 | 代价 |
|---|---|---|---|---|
| W6-保持不适用 | 不动 `rebuild.rs`，size 之外的单元继续报不适用（C827 那一格不还） | 无 | 0 | 缩过一次的池上 I-1.13 永远报不适用，覆盖随使用而掉光；与 C827 的叙述（`| C827 | 扫描重建对删除中与覆盖写变短的镜像报不适用 |` `.claude/kb/checks-owed.md:559`）同源，截断把它从"覆盖写变短"扩成每个缩过的文件 |
| W6-不认领 | Z 落成「size 之外的单元不认领」：重建时偏移 ÷ P 不小于 `data_units_the_size_covers` 就 `continue`，既不进数据单元表也不进中央映射 | `crates/singlefs-checker/src/rebuild.rs:746` 起那个 `return Err(...)` 换成 `continue`（及成员文档 `DataUnitBeyondTheSizeOfItsRecordCoveredByNoClause`）；`crates/singlefs-harness/tests/scan_rebuild_agrees_with_the_walked_index.rs:234` 的 `fn a_data_unit_left_beyond_the_size_by_a_shorter_overwrite_is_not_applicable_to_the_scan_rebuild() {` 由「报不适用」改成「成立」；I-1.13 行里列的"数据单元落在 size 之外"那一条不适用理由（`crates/singlefs-checker/invariants/01-块自描述.md:18`）删掉；C827 那一格改状态；遍历一侧见下 | 0 | 读法 Z 在 truncate 之后第一次真判；对"先落目标"形态要求遍历一侧同步滤（下一节）；单独存在时对洞不安全（探针 B） |

### 先落目标形态下，中间态两侧怎么算才逐项相同

I-1.13 比三张表，其中数据单元表与中央映射：「只从扫描方向扫出的码 1 / 码 3 已发布单元头……重建的三张派生表……与遍历方向从最新根读出的三张表（中央映射只取码 1 / 码 3 的条目）逐项相同」（`crates/singlefs-checker/invariants/01-块自描述.md:18`）。意图之后、最后一批之前，extent 树里还挂着 size 之外的单元，它们的中央映射条目也还在：遍历一侧会列出，重建一侧（W6-不认领）不认领，两表各差若干处，I-1.13 红。办法是遍历一侧也用同一把尺子滤：当前版本数据单元 ⟺ 同出生代记录 ∧ 偏移 ÷ P < ⌈size ÷ P⌉.max(1)；落点在 `IndexTablesWalkedFromTheNewestRoot` 的装填处（`struct IndexTablesWalkedFromTheNewestRoot` `crates/singlefs-checker/src/walk.rs:13158`）；中央映射一张表已有先例——它不收最新根 deadlist 点名的那几条（`fn mapping_keys_named_by_the_deadlist_of_the_newest_root(` `crates/singlefs-checker/src/walk.rs:13307`，I-1.13 那一行原话「遍历方向那一张中央映射不收最新根 deadlist 点名的那几条」），这里再加「不收 size 之外那几个单元的映射 key」。两侧各自在自己的代码里实现这一把尺子，不共享函数（I-1.13 那一行末句：「扫描方向与遍历方向共享的只有格式解析」）。随批降形态下每个中间态都没有 size 之外的挂着的单元，这一滤不需要；两种形态下"放掉之后还在盘上的物理单元"都由 W6-不认领处理。读码推的，没跑。

### 洞在重建一侧成立的理由，和不成立的那一类

重建的数据单元表按 (inode, 偏移) 一项一项存，不要求连号（`struct DataUnitPosition` `crates/singlefs-checker/src/rebuild.rs:102`），所以「尾部缺席」本身重建能装下。成立的条件是：没有"放掉但物理上还在"的单元落进 [u·P, ⌈size ÷ P⌉·P) 这一段。缩短再变长恰好制造这一类（探针 B：2 个单元被认回来，中央映射差 2 处，数据单元表差 2 处）。围栏是岔路 W6-1。

### 岔路

| 编号 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W6-1 洞围栏 | ① 区间墓碑：缩短（或变长）那次发布往墓碑树追加一条 [首个死单元的偏移, 2⁶⁴ − 1]、死亡 txg = 这次的区间记录，重建按「落在区间内且出生 txg < 死亡 txg 的单元不认领」；墓碑记录本来就有区间字段（`rebuild.rs` 读 `RANGE_START` / `RANGE_END`）。要动 `crates/singlefs-core/src/transaction.rs` 的墓碑追加（意图那臂）、`crates/singlefs-checker/src/rebuild.rs:666` 收区间、墓碑相关不变量、记账标签 6 的预付式子（它按"每个对象至多一条墓碑"算，`.claude/kb/layout/03-third-txn.md` 的"记账标签 6 的值"那一行），并**重开 D18 已定项 10 的「区间墓碑不做」**（`.claude/kb/decisions/18-块里携带什么信息.md:235`，用户 2026-10-08 X2 选读法 Z）。② 零单元：变长把洞写成零单元（W5-补零成单元），不留洞，围栏无需；违反用户「留洞」。③ 标不适用：I-1.13 对"遍历一侧有 u < ⌈size ÷ P⌉.max(1) 的 inode"整份报不适用并写理由；不改格式；重建今天没有真消费者，没有立刻的暴露面，但将来真有级 1 只读挂载时洞里会读出旧数据 | ①每次缩短多写的字节（40 字节记录 + 重写开放墓碑容器 32 KiB + 墓碑树根 16 KiB，推的，没量）与墓碑树增长（第一版不回收墓碑，`.claude/kb/decisions/18-块里携带什么信息.md:238`「第一版不回收墓碑（已定项 24）」）；③覆盖损失：有洞的镜像占随机历史镜像的比例 | ①对"缩短再变长"这一类历史各候选的违例数（①③各红几个镜像）与每次截断多写字节数的绝对值，用计数模型钉；③要用户答"重建的级 1 只读挂载什么时候真做"。目前没有这三个数 |

### 与 I-1.9 的关系

I-1.9 今天「未实现」。W6-不认领把"size 之外判死"算作压住，是对 I-1.9 第二句的一个补充：要么在 I-1.9 的叙述里加一类压住（size 之外的单元，条件：同出生代记录存在），要么 truncate 写出压住它们的东西（①区间墓碑）。这一格不阻塞截断落地（I-1.9 没有实现），但它是欠账的新项：落地时登记。
推翻 W6-不认领的现象：重建的消费者变成真正的只读挂载（不再只是 checker 判），那时"被 size 判死但物理上还在"的单元要有写下来的凭据，不能只靠 size。

## W7　checker 多判哪几条

**判定**：规则没说（新增判）。下面按推荐形态（先落目标·条目 + 补零重写）列；每条给必红世界。编号是我暂用的名字，正式编号归主 agent。

### 要加 / 要改的判

| 判 | 判什么 | 要动的处 | 必红世界（构造 → 应红谁） |
|---|---|---|---|
| T-1 I-4.4 白名单加一种 | 种类码 2 登记；value 只装 inode、出生代、目标 size、补零 10（全 0）；inode 记录在同一版存在、nlink ≥ 1、出生代相等；目标 size = 该记录的 size；签发 txg 不晚于这一版；全池至多一条截断条目 | `fn judge_compaction_intents_of_the_versions(` `crates/singlefs-checker/src/walk.rs:15270` 按种类分支；`crates/singlefs-checker/invariants/06-事务与崩溃一致性.md:9` 那一行里的「种类码是登记过的（第一版只有 1 = 整理）」改成两种 | ① 补零里塞「再放 5 个」；② 种类码 3；③ 两条截断条目；④ 目标 size ≠ 记录 size；⑤ 出生代与记录不符。造法：往 logged_ops 按字节造条目，照整理那一份（`crates/singlefs-harness/tests/checker_compaction_intent_and_relocation_must_red_worlds.rs`） |
| T-2 截断中间态有账（照 I-4.7 的写法） | 每个 nlink ≥ 1 的 inode，存在的单元数 u > keep(size) ⇒ 有一条对得上的截断条目；反向：每条截断条目对应的 inode 存在且目标 = size | 新函数，放在 `judge_inodes_being_deleted_against_the_queue_and_their_data_units`（`fn judge_inodes_being_deleted_against_the_queue_and_their_data_units(` `crates/singlefs-checker/src/walk.rs:10771`）旁；共用判法照 I-9.5 的"判走读到的任一版" | ⑥ 中间态镜像上把 logged_ops 根指针置零（单元还挂着、条目没了）→ 红。**这才是「最后一批与截断意图同事务」的必红世界**：D8 说的不可能状态是「意图没了但目标状态没达成」（`.claude/kb/decisions/08-核心索引结构.md:120`，原文里嵌着书名号，整行见源文件）。反方向——最后一批放了单元而条目还在——不是违例（下一步是空的最后一批，续做补上），所以 unlink 那种"最后一批不删 key"的变异在这里**不红**，要换成「最后一批只删条目、不放单元」 |
| T-3 单元数对 size（合并 I-4.6 与 C711） | 对每条 inode 记录（nlink = 0 与 ≥ 1 都判）：存在的单元从 0 起连号；u ≤ keep(size)（有截断条目时允许 u > keep）；u < keep(size) 即洞，合法 | 今天 I-4.6 只判 nlink = 0 且要求相等：`data_units == data_units_the_size_asks_for`（`crates/singlefs-checker/src/walk.rs:10825`），在 `fn judge_data_unit_counts_of_the_inodes_being_deleted_against_their_size(` 内（`crates/singlefs-checker/src/walk.rs:10811`）。洞合法之后，对有洞文件 unlink 的意图那一版（nlink = 0、u < ⌈size ÷ P⌉）这一条会误红——变长落地时必须一起改。C711 的欠账（`.claude/kb/checks-owed.md:475`）同时还清：这条就是它说的「立一条新的结构不变量」 | ⑦ 中间缺一个单元（u 不连号）；⑧ nlink ≥ 1、u > keep 且没有条目；⑨ 有洞文件 unlink 的意图版不得红（合法世界） |
| T-4 尾单元声明长度对 size | 每个存在的单元：声明长度 = min(size − 起点, P)（size 之外的挂着的单元只判 ≤ P） | `walk_extent_data_pointer` 已读整单元头（`    fn walk_extent_data_pointer(` `crates/singlefs-checker/src/walk.rs:4119`），登记 I-9.10 用的 `self.data_unit_objects.push((` `crates/singlefs-checker/src/walk.rs:4194` 旁加一份（偏移、声明长度），在 inode 记录那一遍里比；今天 checker 没有这一判 | ⑩ 探针 C / D 的两张镜像（只改 size、不重写尾单元）：今天池级 checker 一条不红（量的），判加上后必须红——它们现成就是必红世界，源码在 `probes/trunc_probe_readers_on_a_record_edited_size.rs` |
| T-5 I-1.13 | 遍历一侧滤掉 size 之外的单元与它们的映射条目（W6）；若 W6-1 选"标不适用"，对有洞的镜像整份报不适用并带理由 | `crates/singlefs-checker/src/walk.rs:13573`（`fn judge_scan_rebuild_agrees_with_the_walked_index(`）与 13158 的装填处 | ⑪ 探针 B 的镜像：选"区间墓碑"或"零单元"时必须成立；选"标不适用"时必须报不适用 |
| T-6 I-9.15 | 不改（blocks 由 size 现算）；洞与截断后仍成立 | 无 | 现成的坏镜像照用 |

### GPU 判器的覆盖表与内核（岔路 W7-1）

池级 checker 的每条不变量要进 GPU 判器的覆盖表，内核再决定置不置位。今天"覆盖了但内核不置位、对拍不比"的只有一条：`pub const POOL_CHECKER_INVARIANT_INDEXES_LEFT_UNSET_BY_THE_GPU_KERNELS: &[usize] = &[92];`（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:427`），注释写这是用户 2026-10-13 的先例：「先进覆盖表但内核不置位，记欠账」（`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs:417`）。I-4.4 的内核读法在 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl` 里从 `// ── I-4.4 / I-5.6（walk.rs judge_compaction_intents_of_the_versions）`（`:5621`）到下一节标题 `// ── I-3.15（walk.rs judge_unreleased_allocation_records_against_the_central_mapping）`（`:5744`），共 123 行（量的：5744 − 5621），里面写死了种类码 1（`const G_LOGGED_OPERATION_KIND_COMPACTION: u32 = 1u;` `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:5554`）。

| 编号 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W7-1 | 新判都进内核（T-1 在 wgsl 里加种类 2 的分支，T-2、T-3、T-4 各加一段）/ 新判进覆盖表但内核不置位，下标加进上面那张表，记欠账（先例） | 内核每条新判的行数与对拍运行的挂钟；"对拍不比"表长大之后 GPU 对 CPU 的覆盖还剩多少 | 一次实现员的行数估计（T-1 照 I-4.4 的 123 行量级，推的）+ 对拍在新判上的一次整轮结果；用户给"允许多少条内核不置位"的线。目前没有这两个数 |

### 其余
- `crates/singlefs-checker/invariants/` 的格式要同步（`.claude/rules/format-evolution.md` 的「硬约束」一节要求改格式时三者同步，转述）：06 的 I-4.4 行、新增的 T-2 / T-3 / T-4 各一行、01 的 I-1.13 行与 I-1.9 行；kb 同步：D26 已定项 2 的字段表旁加截断那张、`.claude/kb/layout/` 一份新页（本报告 2.5 的表）。
- T-4 和 T-3 合起来才能还 C711；只做 T-4 不够，洞合法之后 u < keep 不再是违例（W5-2 的取舍）。
- 推翻它的现象：W5-2 选"声明洞"（上段叶标签）——T-3 的 u < keep 要改成"必须有标签"，反而更强。

## W8　模型、RefFS、翻译层、随机历史

**判定**：规则没说（没有条款管模型怎么表示截断与洞）。下面按推荐形态（先落目标）写。事实表第六节给的"加一种操作要动的十处"我抽核了一部分行号，没逐处核；下面凡是我核过的标了行号。

### 理想模型（`crates/singlefs-harness/src/model.rs`）

今天一版的文件 = 写出它的发布加整份字节：`pub content: Rc<[u8]>,`（`crates/singlefs-harness/src/model.rs:569`），数据单元数由内容长度现算：`pub fn data_units_of_content(content_length_in_bytes: u64) -> u64 {`（`crates/singlefs-harness/src/model.rs:912`）。

| 问 | 候选 | 要动的处 | 说明 |
|---|---|---|---|
| 洞与写过的 0 | ① 模型不区分（内容里补 0），但 `ModelFileVersion` 加一个 `present_units: u64`（盘上存在的单元数）| `ModelFileVersion`、`data_units_of_content` 的调用点（它现在假定 = ⌈长度 ÷ P⌉.max(1)），`extent_lower_node_positions`（"单元号 0 .. data_units − 1、没有洞"）改按 present_units 算 | 读者答案相同（都是 0），盘上单元数不同，模型的角色集合、extent 节点数、释放的单元数都依赖单元数，所以模型要知道 u；不要知道洞在哪（洞恒在尾部，本报告 2.3）。推荐 ① |
| | ② 内容用稀疏段表示 | 模型整个内容读写 | 洞恒在尾部时没有额外信息，白加复杂度 |
| 每个单元的出生 txg | 现在模型对数据角色只记一个出生 txg：`.get(&ModelUnitRole::Data)`（`crates/singlefs-harness/src/model.rs:3020`）后面紧跟 `.expect("换下数据单元的那一版有数据单元");`（`crates/singlefs-harness/src/model.rs:3021`），换下 n 个单元就按同一个出生 txg 记 n 份。尾单元重写之后一个文件的单元出生 txg 不再相同（W3）；deadlist 判定按单元的出生 txg 对 p（W4），所以要改成按单元记 | `role_written_at` 里 Data 一项换成"每个单元一个 txg 的序列"；`units_replaced_by_the_publish`（`fn units_replaced_by_the_publish(` 在 `crates/singlefs-harness/src/model.rs:2944`）按被换下的单元序号取各自的 txg | 必改，没有候选 |
| `ModelPublishKind` 加哪几个 | 截断意图（带布尔：是否重写尾单元，两种角色集合）/ 截断批 / 截断最后一批 / 只改记录（变长与落在洞里的缩短，带布尔：是否重写尾单元） | `enum ModelPublishKind`（`crates/singlefs-harness/src/model.rs:157`）；`ModelPublishKind::UnlinkIntent` 在这个文件里出现 13 行（`grep -c`，量的），加四个成员约是同一量级的 `match` 要补 | 角色集合：意图 = inode 叶与根、条目根、（尾单元）、extent 上段根与下段路径、固定点；批 = extent 路径、固定点（没有 inode 叶与根——这是 W2 先落目标的模型侧账）；最后一批再加"移除条目根" |
| `units_replaced_by_the_publish` 怎么算 | 意图：换下旧的第 k 个单元（若重写）和罩着 k 的下段节点与上段那一串；批：换下被截的 K 个单元的出生 txg 序列、节点按 `last_data_unit_under(position) < data_units_kept` 的老写法；最后一批：剩下的 u − keep 个单元 + 条目根 | 同上函数里照 `DeletionBatchKeepingDataUnits` 那一臂（`let data_units_kept = data_units_remaining_before_this_publish.expect(` 起，`crates/singlefs-harness/src/model.rs` 约 2975–3000）写三臂 | 算法是纯函数，与实现的 `resolve` 各算各的，不共享代码 |
| 版本里有什么 | `ModelVersionContents` 加 `FileBeingTruncated { content（N 字节）, units_remaining, trees_written_by }`（对照 `FileBeingDeleted`，`crates/singlefs-harness/src/model.rs:583`）| `enum ModelVersionContents`（`:574`）及它的读法 | 读回的是 N 字节内容（不是"没有文件"） |
| 操作与答法 | `ModelOperationKind`（`pub enum ModelOperationKind {` 在 `crates/singlefs-harness/src/model.rs:1358`）加 `PublishTruncationIntent`、`PublishTruncationStep`、`PublishSizeOnlyChange`；答法对照 `answer_unlink_intent`（`:3292`）、`answer_deletion_step`（`:3334`）、`answer_create_snapshot`（`:3440`）；`judge_and_advance`（`:4390`）加臂 | | |

### RefFS（`crates/singlefs-reference-filesystem/src/lib.rs`）

RefFS 的内容就是整份字节：`pub struct FileContent(pub Vec<u8>);`（`crates/singlefs-reference-filesystem/src/lib.rs:22`），没有稀疏的概念；这对截断正好——POSIX 面上洞和 0 不可区分。推荐：`Operation::Truncate(u64)` 作内容改动型（和 `OverwriteTheFile(FileContent),` `crates/singlefs-reference-filesystem/src/lib.rs:44` 同一个 `EnteredKind::ContentChange`），施加 = 截短或补 0。崩溃落在意图与最后一批之间时读出的内容：先落目标下恰好是"截前"或"截后"，都是 RefFS 本来就枚举的前缀状态（操作序列的前缀），不用新构造。对照：随批降下批间的前缀内容不是任何操作序列前缀，RefFS 要新开一类"一个操作的中间内容"，并把批大小带进规格。需要主 agent 另定的小事：这个操作是不是也该像删快照那样作"独立发布"（`EnteredKind::DestroySnapshot => true,` `crates/singlefs-reference-filesystem/src/lib.rs:235`，一次调用写多条根）——我的读法是不必，因为每一条根读出的内容只有截前 / 截后两种，但 RefFS 的 `ends_a_publish` 那一支的语义（返回成功时是否抬下界 lo）这一格我没有读全，标"没核"。

### 翻译层（`crates/singlefs-harness/src/reference_filesystem_translation.rs`）

`events_of` 对 unlink 写了盘就停：`TranslationStopReason::TheReferenceFileSystemHasNoUnlinkInTheFirstVersion,`（`crates/singlefs-harness/src/reference_filesystem_translation.rs:421`）。截断要在同一个函数里加一臂：成功发成（不论发了几个版本）→ 一个 `Operation::Truncate(n)`；在任何写之前拒 → 空；写了之后报错 → `FailedAfterWriting`。多版本的先例是删快照那一臂（同文件 `HistoryOperation::CreateSnapshot | DestroySnapshot` 那一段，事实表记在 443–465 行，我没逐行核）。

### 随机历史（`crates/singlefs-harness/src/history.rs`）

要动的点：`pub enum HistoryOperation {`（`crates/singlefs-harness/src/history.rs:497`）加 `Truncate { target: TruncateTarget }`（目标取相对当前 size 的类：缩到 0 / 缩到单元边界 / 缩到单元中间 / 在单元内变长 / 变长跨单元；类比绝对 N 好，因为 N 要落在当时文件的结构上才有意义）；`HistoryOperationKind` 与 `pub const ALL: [HistoryOperationKind; 16] = [`（`crates/singlefs-harness/src/history.rs:609`，长度加一）；`kind`、比重表、`draw_operation`（`:1456`）、`expected_session_after`（`:1576`）、`expected_first_file_after`（`:1635`）、`note_outcome`（`:2932`）、执行器（对照 `apply_unlink`，`:4311`）、`apply_operation`（`:6502`）、`fn simpler_variants(operation: HistoryOperation) -> Vec<HistoryOperation> {`（`crates/singlefs-harness/src/history.rs:7505`；缩小：目标往 0 的方向、往更靠近当前 size 的方向、整个操作去掉）。比重：只进自己那一组（先例：「只进它自己那一组比重（[`GenerationWeights::SNAPSHOT_OPERATIONS`]），已有各组生成的历史」`crates/singlefs-harness/src/history.rs:545`），这样已有各组种子生成的历史不变，已入库的随机历史产物不用重跑。

### 会话入口（`crates/singlefs-core/src/mounted_session.rs`）

`UserChange` 加 `Truncate { inode: InodeNumber, new_size: u64, write_time_seconds: u64 }`（`pub enum UserChange<'content> {` `crates/singlefs-core/src/mounted_session.rs:98`；同文件 `kept_from`（`:651`）、`as_user_change`（`:682`）、`publish_the_change_once`（`:1091`）、`publish_the_change_raising_the_floor_for_space`（`:1361`）各补一臂）。拒什么：

| 情形 | 拒成 | 说明 |
|---|---|---|
| 现行那一版没有文件（树表 0 条）或文件集合为空 | 现成的 `NoFileVersionToChange {`（`crates/singlefs-core/src/mounted_session.rs:220`） | 先走空再拒，同别的改动 |
| inode 号不在或 nlink = 0 | 新增一个类型化错误（形状同 `UnlinkOfAnInodeThatIsNotALiveRecord`） | 会话里走空之后 nlink = 0 的记录已经没了，所以实际落在"不在" |
| N 超过上限 | 条款没有最大文件长度；算术上要求 ⌈N ÷ P⌉ · P 不溢出 u64 | 要主 agent 定一个上限常量，我不替定（规则没说） |
| N = S | 不拒：发一次只改记录（时间、改动计数）的发布，或拒成"没变化"——二选一，规则没说 | POSIX 的 ftruncate 到同长度仍更新时间，但这一轮没有条款；`InodeRecord` 只有一个 `write_time_seconds`（三个时间秒同值），所以时间怎么写没有分歧 |
| 对一个只有 size 0、没写过内容的 inode（建 inode 建出的）变长 | 允许（整个文件是洞，上段叶条目标签 0） | W5 |

### 岔路

| 编号 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W8-1 RefFS 里截断的类 | 内容改动型（同覆盖写）/ 独立发布型（同删快照，一次调用多条根） | RefFS 在两种设定下对同一批实现产物判出的"允许集合"是否相同（先落目标下两者应当相同） | 一次对拍：同一批随机历史、两种设定各一份 RefFS 判，违例集合逐项相同；我没跑，因为 RefFS 的 `ends_a_publish` 语义没读全 |

## W9　崩溃流、必红世界与只供测试的开关

**判定**：一致。D8 已定项 16 的射程写了「截断成 0 另立一条流」（`.claude/kb/decisions/08-核心索引结构.md:489`）；里程碑六·3 要求「改坏「最后一批与截断意图同事务」那一处，那条流的判器红」（`.claude/kb/milestone/04-fourth-txn.md:151`，原文里嵌着书名号，整行见源文件）。

### 流（接在哪、各要不要一条）

接法：和 unlink 流同一个前缀——多记录流「取号 → 暖机 × 2 → A → B → C」之后（C 之后是三个单元的文件），在这个前缀后面各接截断（`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs` 的 `prepare` 与 `mount_continuation_observer` 可以复用；公共模块 `common_crash_points` 里已有 `pub fn publish_the_unlink_intent(pool: &mut BuiltPool) -> TransactionOutput {`（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:146`）和 `pub const DELETION_BATCHES_BEFORE_THE_FINAL_BATCH: usize = 2;`（`crates/singlefs-checker-tier/tests/common_crash_points/mod.rs:143`），截断各加一份）。每条流登记进 `.claude/gate.d/crash-point-replay-items.tsv`（unlink 的行形如 `unlink-after-the-multi-record-publish`，`.claude/gate.d/crash-point-replay-items.tsv:18`），同目录还有 `.claude/gate.d/crash-point-replay-segments.tsv` 按流登记段（形态我没读）。版本表（`pub struct CrashPointReplayVersionTable<'table> {` `crates/singlefs-checker-tier/src/crash.rs:785`）里先落目标形态下只有意图那一版带 N 字节内容、批与最后一批不进表。崩在可写挂载续做里的那一格不用新相位：续做走的还是共用走空，`WritableMountPhase::Continuation`（`    Continuation,` `crates/singlefs-checker-tier/src/writable_mount_phases.rs:29`）已罩着。

| 候选流 | 内容 | 它独有的覆盖 | 要不要 |
|---|---|---|---|
| 缩到 0 | C → 截到 0（保一个声明长度 0 的单元，放另外 2 个；批大小压成 1：意图 → 一批 → 最后一批） | 条款点名要的这一条；尾单元重写落在 Data(0)（`roles_replaced_via_mapping` 那个缺口的形状，W3）；下段整段消失、上段叶条目由标签 1 变标签 2（内联与下段互换，`.claude/kb/decisions/08-核心索引结构.md:445`）；三次发布 | 要 |
| 缩到单元中间再变长 | C → 截到 1.5 个单元（重写第 1 个单元、放第 2 个）→ 变长到 4 个单元的长度（补零重写尾单元，其后缺席） | 尾单元落在非 0 位置；变长；缩短再变长的"放掉的单元还在盘上"那一类（探针 B 的历史）每个崩溃态上都被 I-1.13 看一遍 | 要 |
| 快照下缩 | A → 建快照 → 截到中间 →（对照 `unlink-under-a-snapshot`，`.claude/gate.d/crash-point-replay-items.tsv:25`） | 尾单元旧版与被截单元进 deadlist、映射条目留存；快照读者读截断前内容 | 要 |
| 变长留洞单独一条 | C → 只变长 | 变长是单次发布，没有中间态；覆盖已被"缩到单元中间再变长"那条的后半罩住 | 不要单独一条 |

枚举域大小我没有数：unlink 流的全量域是 1 463 812 278 个状态，快档只展开写数小于 10 的段（`快档只展开写数小于 10 的段` `crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:22`）；截断的流有多少段、每段几个写，要实现落地后用 `closed_form_state_count`（该测试已用，`crates/singlefs-checker-tier/tests/crash_enumeration_unlink_after_the_multi_record_publish_stream.rs:62` 起的 `use`）出闭式，是缺口。

### 必红世界（每条先证会红；变异表形态：变异名 / 文件 / 原文 / 替换文 / cargo test 参数 / 必须红的测试，`crates/mutations.tsv`）

先例一行：`第33批 三·4 删除流（接在多记录流后面 unlink）：最后一批不删 deleted_inodes 那条 key……`（`crates/mutations.tsv:3021`）。截断的对应行：

| 变异 | 改哪 | 应当红的 | 说明 |
|---|---|---|---|
| 最后一批删条目而不放单元 | 截断最后一批的 extent 计划那一臂不换下尾部（W2 的三处之一） | T-2（surplus 无账）在最后一批那一版；流的判器 | 这是"最后一批与截断意图同事务"的会红版本。不能照 unlink 写成"最后一批不删条目"：那一形是合法的中间态（下一步补一发空的最后一批），而且走空永远走不完——走空循环的收束靠注释写的"迭代上界"论证：「迭代上界：每一步要么让队首那个文件少至少一个单元、要么删掉队首那个号（最后一批）」（`crates/singlefs-core/src/transaction.rs:8244`），变异把它打破会让变异测试挂钟超时而不是干净地红 |
| 意图那次不重写尾单元 | 尾单元重写那一臂 | 读者拒（探针 C 的形状）、T-4 | 对应 W3 |
| 换下清单判别退回「含 Data(0)」 | `roles_replaced_via_mapping` 三个调用点的新参数改回推断 | 缩到 0 那条流在意图那一版：中央映射 / 已分配统计对不上（I-3.1 一类）、相等断言 panic | 对应探针 F |
| 条目的目标 size 写成旧 size | 签发条目那一处 | T-1 ④ | |
| 一批放整个文件 | 批大小开关的产品臂取最大 | 斜率测试 | 照 unlink 那份的判别力写法（`crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` 里 `判别力：批大小改成整个文件一批`） |
| 变长不补零重写 | 变长的尾单元重写那一臂 | 读者拒（探针 D 的形状）、T-4 | |
| 快照下强制全放 | 用已有的测试开关 `ForcedToReleaseIntoTheDeferQueueForTests` | 快照读者读出的内容与建快照那刻不同或读不出 | 已有开关，不新增 |

### 只供测试的开关（`.claude/rules/fs-design.md` 五条硬要求第 2 条：「每条分支必须能被测试强制进入。」`.claude/rules/fs-design.md:122`）

- 批大小：复用 `UnlinkBatchSize::CappedForTests`（W2）。
- 强制尾单元重写在意图那次还是最后一批：推荐形态里只有一个时点，不是分支，不需要开关；若 W2 的岔路选了两种形态并存，才要。
- 零放走快路（W1-2）：若用户要快路，才要 `TruncationFastPath::{FollowTheCriterion, NeverForTests}` 一类的开关，并按第 3 条为快路另跑崩溃点重放。
- 快照判定的强制臂：已有。
- 分支可观测（第 4 条）：交回的版本个数（意图一个、批若干、最后一批一个）已经能看出走了几步；变长与缩短的分叉由 N 与 S 的大小关系定，操作期间不变（第 5 条）。

### 推翻与岔路

推翻「接多记录流的前缀就够」的现象：缩到中间那一步在 A、B 版本上（单元数不同）也要独立覆盖，那时前缀不同，流要拆开。
岔路 W9-1：流的条数。候选：三条（缩到 0、缩到中间再变长、快照下缩）/ 四条（变长单独）/ 两条（缩到 0、快照下缩）。翻面观测：各流的全量枚举域（闭式）与挂钟；必红世界 × 流的红矩阵里，哪条流是某个变异唯一的红。够判条件：实现落地后每条流的闭式域大小一个绝对值，加上上表七个变异各在三条流上跑一遍的红 / 不红矩阵——这些要实现，现在没有。

## 补记（读码推的；前面各格没放下的几点）

1. 删除批的 extent 计划函数断言「删除的一批留下至少一个单元、至少截掉一个（留下 0 个是最后一批）」（`crates/singlefs-core/src/unlink.rs:189`；判的条件在 `(1..data_units_before_the_batch).contains(&data_units_kept),` `crates/singlefs-core/src/unlink.rs:188`）。截断的最后一批可能一个单元也不放（W1 单一路径，u = keep），所以截断的 extent 计划函数要自己写，不能直接调它；它的"整个落在留下那一段里才照抄"的判（`let is_entirely_kept = |position: &ExtentLowerNodePosition| {` `crates/singlefs-core/src/unlink.rs:191`）要多一个入参：被重写的尾单元序号，罩着它的节点即使整个落在留下那一段里也要重写。
2. 发意图的入口照 `publish_unlink_intent` 的写法在落盘前断言队列已走空：「"发意图之前队列已走空：任何一版的 deleted_inodes 至多一条 key（D8 已定项 16 第 6 条）"」（`crates/singlefs-core/src/transaction.rs:8100`）；截断的入口除了 `deleted_inodes` 与销毁中的快照为空，还要断言上一版没有截断条目。
3. 数据单元的对象号写死成 inode 1：`|| header.identity.object != FIRST_INODE_NUMBER`（`crates/singlefs-core/src/recovery.rs:5413`）与写侧同一个假设（`// 第一版只有第一个文件写数据：别的 inode 一个数据单元都没有，意图之后直接是最后一批。` `crates/singlefs-core/src/transaction.rs:8166`）。截断这一轮因此只有 inode 1 会真的放单元、重写尾单元；别的 inode（建出来 size 0 的）只会被变长成"整个文件是洞"、再缩回 0。这条限制不新增债，沿用 unlink 已有的那一条；推翻它的现象是第一版之外有第二个 inode 写数据。
4. 时间：`InodeRecord` 只有一个 `write_time_seconds`，截断意图那次取调用方给的值（同 unlink 意图），批不带时间。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| W1 意图载体与目标态 | 规则没说；D8 已定项 5 定案与 D8 已定项 16 第 1 条两句并存，要用户定 | 推荐 logged_ops 种类码 2（inode、对象出生代、目标 size、补零 10），全池至多一条，进共用走空，三处"队列空"守卫一起改；零字节的"记录推导"要放弃 D8 已定项 5 定案那句才选得了 |
| W2 分批与最后一批同事务 | 一致（D8 已定项 5 的四条与已定项 16 第 7 条五条可直接写在截断上）；批间 size 与批大小开关规则没说 | 推荐先落目标：意图那次 size ← N 并重写尾单元，批只放尾部单元不碰记录，最后一批在 `ordinary_allocation_part` 三处同时删条目；批大小共用 `DeletionSwitches.batch_size`；崩溃后读到的只有截前 / 截后 |
| W3 尾单元怎么 COW | 规则没说（条款预见了尾单元重写）+ 两处实现缺口（其中换下清单判别是量的） | 推荐意图那次重写尾单元，截到 0 留一个声明长度 0 的单元；`roles_replaced_via_mapping` 的"含 Data(0)"推断要换成显式参数，否则截到一个单元以内会提前放掉后面的单元（探针 F） |
| W4 快照下截断 | 一致 | 复用 `deadlist_changes_of_this_publish`；事实表六条里 1、2 要改、3–6 成立；意图跨过建快照在会话里不可达（每次改动先走空），只可能来自回退 |
| W5 变长与洞 | 冲突（"洞就是缺席"、读法 Z、不写区间墓碑、用户"留洞"在缩短再变长上不相容，探针 B 量的） | 洞恒在尾部，`data_pointers` 不用拆；推荐变长补零重写尾单元、不立意图、读者只改条数判一处；洞要不要在上段叶声明是岔路 |
| W6 扫描重建与读法 Z | 冲突（读法 Z 与 I-1.9；Z 与洞） | Z 落成"不认领"（`crates/singlefs-checker/src/rebuild.rs:746` 一处），先落目标下遍历一侧同尺子滤；洞的围栏三选一（区间墓碑 / 零单元 / 标不适用），重建今天只有 checker 一个消费者 |
| W7 checker 多判 | 规则没说 | 六条：I-4.4 加种类、截断中间态有账、单元数对 size（合并 I-4.6 与 C711，洞合法之后 I-4.6 不改会在有洞文件的 unlink 意图版上误红）、尾单元声明长度对 size、I-1.13 滤法、I-9.15 不改；"最后一批与意图同事务"的必红世界是"条目被删而单元还挂着" |
| W8 模型 / RefFS / 翻译层 / 随机历史 | 规则没说 | 模型加 present_units 与按单元记出生 txg，`ModelPublishKind` 加四个成员；RefFS 加 `Truncate(n)` 作内容改动型，先落目标下不需要新构造；随机历史加一种操作要动的十处，比重只进自己的组 |
| W9 崩溃流、必红世界、开关 | 一致 | 三条流（缩到 0、缩到中间再变长、快照下缩），七个变异，批大小开关复用，推荐形态下尾单元时点不是分支所以不要开关 |

## 要交用户的岔路汇总（候选 / 翻面观测 / 够判条件，不带倾向；上面各格里的原表这里并成一张）

| 编号 | 候选 | 翻面观测 | 够判条件 |
|---|---|---|---|
| W1-1 载体 | logged_ops 种类 2 / 记录推导（零字节，违 D8 已定项 5 定案）/ 新派生树 | 每次截断多写字节数；回退跨过未完成截断时"有账"判红次数 | 同一批放开用户动作的历史上三候选各一个多写字节绝对值与违例数（计数模型，目前没有） |
| W1-2 零放走快路 | 单一路径（放 0 也走最后一批）/ 加快路（要判据、开关、崩溃流） | 用户负载里"放 0 个单元的截断"占比；快路省的发布数 × 每次发布写入量 | 负载取样 + 产品路径单次发布写入量绝对值；比例线由用户定 |
| W1-3 回退 | 取 R_old 的 logged_ops / 随现行 / 回退时丢截断条目 | 回退落在意图与最后一批之间的历史上"有账"判红次数与泄漏单元数 | 该历史上三候选各一个违例数；与 C621 合在一轮 |
| W2-1 批间 size | 先落目标（崩溃读到截前 / 截后）/ 随批降（照 unlink，批间读到前缀，读者不用改） | 只读挂载在批间崩溃之后读到什么是否被要求；读者放宽三处的改动量 vs 每批多写的 inode 叶与根 | 两候选在同一批历史上各一个每批写入字节数（先落目标少 inode 叶 32 KiB + 根 16 KiB，推的），加用户对"批间中间态读法"的回答 |
| W2-2 准入类 | 当删除类（不判式子，环窗口罩，W 的 24 要重算）/ 当普通写（判式子，释放空间的操作可能被空间拒） | 截断意图那次每盘新占槽数是否 ≤ W 里那个 24 | 一个单测钉截断意图每盘新占槽数的绝对值；用户答"截断可不可以被空间拒" |
| W5-1 变长写不写尾单元 | 补零重写 / 只改记录（读者放宽四处加冷走读一处） | 变长的频率与 S 不对齐的占比；两条路的写入字节数 | 两条路同批历史上写入字节数绝对值；放宽方案的"放过的错"必红世界清单 |
| W5-2 洞要不要声明 | 隐含 / 上段叶条目加标签声明 | 洞合法之后池级 checker 掉了几类今天能抓的错 | 故障注入清单（尾部单元被错放、批多放、最后一批漏放）在两候选上各红几条 |
| W6-1 洞围栏 | 区间墓碑（重开 D18 已定项 10 "区间墓碑不做"）/ 零单元（违"留洞"）/ I-1.13 对有洞镜像标不适用（重建今天无真消费者） | 每次缩短多写字节与墓碑树增长；有洞镜像占比；级 1 只读挂载何时真做 | "缩短再变长"类历史上各候选的违例数与多写字节绝对值；用户对级 1 的时间表 |
| W7-1 GPU 判器 | 新判都进内核 / 进覆盖表但内核不置位（先例 I-1.13，下标 92） | 每条新判的内核行数（I-4.4 那段 123 行，量的）与对拍挂钟 | 实现员行数估计 + 一次对拍整轮；用户给"允许几条不置位"的线 |
| W8-1 RefFS 截断的类 | 内容改动型 / 独立发布型 | 同一批历史上两种设定判出的违例集合是否逐项相同 | 一次对拍（我没跑，因 `ends_a_publish` 的语义没读全） |
| W9-1 流的条数 | 三条 / 四条 / 两条 | 各流全量枚举域（闭式）与挂钟；必红世界 × 流的红矩阵 | 实现落地后每条流的闭式域绝对值 + 七个变异各在三条流上的红 / 不红 |

## 每格结论的推翻条件

| 格 | 什么现象会推翻 |
|---|---|
| W1 | 回退对 logged_ops 被用户定成"随现行"且那时条目与记录失配造成不可修复的泄漏；或 GPU 扫描内核的整理读法改不动 |
| W2 | 用户要求批间崩溃之后必须读到某个中间 size；或读者容忍多出单元被证明放过泄漏（W1-记录那一形） |
| W3 | 用户负载里"截断点不对齐且之后不变长"占绝大多数，使省一次 32 KiB 写比读者简单更重要 |
| W4 | W6-1 选了区间墓碑：墓碑要不要随 deadlist 回收，会重开 D18 已定项 10 的回收条款 |
| W5 | 出现"在偏移 X 写"的入口，洞不再只在尾部；或用户的负载里变长频繁且几乎从不对齐 |
| W6 | 重建的消费者变成真正的只读挂载 |
| W7 | W5-2 选了声明洞，T-3 变成必须有标签 |
| W8 | RefFS 的 `ends_a_publish` 语义要求截断作独立发布，那时先落目标下也要新构造 |
| W9 | 缩到中间那一步在 A、B 版本上（单元数不同）也要独立覆盖，前缀要拆开 |

## 反推与校验（照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md` 开头三步表：假设结论错，本该看到什么，看了没有）

| 结论 | 若错，本该看到 | 看了没有 |
|---|---|---|
| 批不用碰 inode 记录（W2） | 记录里有 extent 根指针字段 | 看了：`InodeRecord` 只有 inode、对象出生代、nlink、size、改动计数、写入时间六个字段（`pub struct InodeRecord {` 所在的 `crates/singlefs-core/src/records.rs:50` 起），extent 根在树表里 |
| 洞只在尾部（W5） | 有从非 0 偏移写的入口 | 看了：改文件内容的三个入口都接 `FirstFile { content, write_time_seconds }`，没有偏移（本报告 2.3 的三个入口行）；没看过 `crates/singlefs-harness` 之外是否有绕过这三个入口的调用方（grep `publish_` 的调用方我没逐个读） |
| 只重写 Data(0) 时换下清单带上后面的单元（W3） | 探针 F 第一行只有 Data(0) | 看了，是三个（量的）；没看过的是端到端的发布（只在函数层量，没有真去实现一个只重写 Data(0) 的发布） |
| 只改 size 不重写尾单元读者就拒（W3） | 探针 C 的 `read_at` 通过，或池级 checker 红 | 看了，`read_at` 拒、checker 一条不红（量的） |
| 缩短再变长会让重建认回放掉的单元（W5/W6） | 探针 B 报成立 | 看了，报红且差在两张表上、各 2 处（量的）；用的是"改记录的 size"来模拟变长，真正的变长发布没有实现，所以量的是"这张盘形"而不是"变长的实现" |
| 先落目标让崩溃后只有截前 / 截后两种读法（W2） | 意图那次之外还有别的发布改 size | 读码推的：批不碰记录是设计，不是现状；没有实现可看 |
| 队列守卫三处 + 准入一处（W1） | 有第五处 | `grep -rn 'deleted_inodes.is_empty()'` 在非测试代码里命中 `crates/singlefs-core/src/admission.rs:448`、`crates/singlefs-core/src/mounted_session.rs:1388`、`crates/singlefs-core/src/mounted_session.rs:2003`、`crates/singlefs-core/src/mount.rs:6574` 和 `crates/singlefs-core/src/transaction.rs` 里四处（`:3691`、`:6114`、`:6242`、`:8099`，这四处是发布路径内部，不是守卫），没有别的；`snapshots_being_destroyed` 另有几处，一一在上面的守卫里（量的） |

校验（第二条不共用代码的路子）：W3 的缺口我用两条路得到——读 `roles_replaced_via_mapping` 的代码，和探针 F 在垫片上调同一个函数；两条都读同一个函数，所以不独立于"函数本身"，只独立于"我的读法"。W6 的复活我用读 `rebuild.rs` 的代码，和探针 B 跑 `check_pool_image`；后者经过的是池级 checker 的整条路径（遍历一侧、重建一侧各自的代码），不共用我的读法。探针本身会不会红：探针 B 里 A 与 B 两张盘的判决不同（NotApplicable / Violated）就是它的阳性对照；探针 C/D/E 三张盘的拒绝原因各不相同（声明长度、声明长度、条数），说明不是同一个错误被三次复述。

## 没做什么

- 没判别：这一轮是正推腿，各格的判定是"条款与代码今天的样子相比"，采纳与判决归主 agent；岔路没有倾向（推荐写在各格"推荐"里，不进岔路表）。
- 没读别的腿这一轮的产出（禁读清单）。
- 没写 kb、没写 `crates/` 里任何东西；探针只在草稿副本里，入模型目录的是副本源码。
- 没实现任何候选，所以所有"新写出的字节""每批写入量"都是读码推的，没有量。要量需要实现落地后跑 `crates/singlefs-harness/tests/unlink_writes_per_transaction_do_not_grow_with_the_file.rs` 那一类测试的截断版。
- 没拆开读 D28 的 W 上界里"普通分配 16"的构成，所以截断意图那次是否在 24 以内只是"很可能"（W2）。
- 没读全的：RefFS 的 `ends_a_publish` 语义、`.claude/gate.d/crash-point-replay-segments.tsv` 的登记形态、GPU 扫描内核整理意图那 123 行的逐行内容、`history.rs` 里十处的逐处代码（只核了函数起点的行号）。
- 没跑门禁阶段、没跑 checker 档测试、没起 GPU；cargo 只起了三个探针各一条，同一时刻一条，用自己的编译目录，经 `scripts/hosts/run-with-memory-cap.sh 8G`，线程 31（`capped.sh default`）。
- 复核不了的：每次截断多写的字节数、枚举域大小、对拍挂钟（都要实现）；回退对 logged_ops 取哪一版的行为（没有实现）；真正的"变长"发布（没有实现，探针 B 用改记录模拟）。
- 没有要主 agent 代跑的命令。三个探针的重跑命令在第一节，输出已入模型目录。

## 产物

- 报告：`research/prompts/m4-trunc-r1-sonnet-output.md`（本文件）。
- 模型目录：`research/prompts/m4-trunc-r1-sonnet-model/`：`probes/`（三个探针源码 + 一个垫片）、`tools/rerun.sh`（从模板拷，只改 `rerun_steps`）、`out/`（三个探针的原样输出，来自 `rerun-work` 那一遍）。
- 草稿：`/home/fy5090/.local/share/singlefs/work/m4-trunc-r1-sonnet/`（`frozen/` 干净副本、`tree/` 第一遍跑的副本、`target/` 与 `rerun-work/` 编译目录、`ln.sh` 查行号的小脚本、`probe-run.log`、`probe2-run.log`、`probe3-run.log`、`rerun.log`）。

## 出口

交回前跑 `python3 scripts/agents/cite-check.py research/prompts/m4-trunc-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/m4-predev`，末行（原样）：

```
  ✓ 核了 47 处引文、153 处裸行号，对上 47 处，没判 153 处（其中裸行号没有引文可比 153 处；逐处列在上面）
```

没判的 153 处都是只写了行号、没有引文可比的裸行号：我用草稿目录的 `bare_scan.py` 把每处行号那一行的原文打出来、和报告里行号前的说法并排对过一遍（大部分行号写进报告之前另用 `grep -nF` 现取过），行号在文件范围内、内容对得上。
