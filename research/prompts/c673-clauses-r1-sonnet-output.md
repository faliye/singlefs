# c673-clauses-r1　K1、K2、K3　云端正推（Sonnet，第一轮）

立场：把每个候选对到 kb 原文与 `crates/` 今天的代码，给每格可直接写回的条款句与判据。本报告里凡新立的不变量、新判据、新条款句一律是零轮（没被攻过）。
代码引文读冻结副本 `/tmp/claude-1000/c673-clauses-r1/tree/`（报告里 `crates/…` 路径都在那棵树里取行号）；kb 引文读仓里今天的文件，行号现查。
冻结副本核对（`sha256sum -c` 对快照清单 `research/prompts/c673-clauses-r1-snapshot/crates-sha256.txt`，在树根里跑）：261 行全 OK；kb 一份 `kb-sha256.txt` 6 行全 OK。
不读禁读清单里的文件。没编译、没跑 cargo、没跑重型测试；下面凡写「推的」都是读码加算术，不是跑出来的。
模型与脚本：`research/prompts/c673-clauses-r1-sonnet-model/head_id_and_first_publish_placement_model.py`（纯算术，只用标准库），输出落 `/tmp/claude-1000/c673-clauses-r1-sonnet/model.out`（13 行）。⚠️ 它是 `issued_from_watermark` 与首次适配规则的算术镜像，不是第二条独立路径：它的发号偏移与首次适配规则就是我从代码读出来的那几条，判不了「代码对不对」，只判「按这几条读，两种读法在哪些表上分开」。

## 一、K1　树表条目的头 ID 怎么判（C673 ⑤）

### 1.1　今天的条款与实现

条款（kb 原文）：

- 「inode 树条目写自己的树 ID（65537），extent 树条目写它服务的那个头（65537），`deleted_inodes` 与墓碑树写 inode 树的树 ID（65537），谱系树写 0（全池一份）」（`.claude/kb/decisions/08-核心索引结构.md:250`）。
- 「可写头的 ID = 该头 inode 树的树 ID。」（`.claude/kb/decisions/05-快照-空间记账机制.md:227`）；同一行接着写「值的规则是每头一份的结构写它服务的那个头的树 ID，全池一份的写 0」（`.claude/kb/decisions/05-快照-空间记账机制.md:227`）。
- 树 ID 从水位发：「十二棵树的号从水位起连号发，次序照已定项 11 那一组 65536..65547」（`.claude/kb/decisions/08-核心索引结构.md:243`）。
- 登记位「新池新建文件写出的 **7** 条树表条目照此表」（`.claude/kb/decisions/08-核心索引结构.md:300`）。这一句与实现冲突，见 §2.4。

代码（冻结副本）：

- 写：core 写头 ID 取 `trees.inode.0`，不取常量：「头 ID（D5（快照 / 空间记账机制） 已定项 9）：inode 树写自己，extent 树」（`crates/singlefs-core/src/transaction.rs:8164`），extent、inode、`deleted_inodes`、墓碑四条各一处 `trees.inode.0`（`crates/singlefs-core/src/transaction.rs:8178`、`:8184`、`:8227`、`:8237`）。
- 发号：「pub fn issued_from_watermark(」（`crates/singlefs-core/src/transaction.rs:2597`）按「水位 + 偏移」发十二个号，注释写「回退到树表 0 条的一版之后再发，」那一版水位带着环里的 max（`crates/singlefs-core/src/transaction.rs:2549`）。所以一旦发号的水位不是 mkfs 种下的 65536，inode 树的树 ID 就不是 65537，头 ID 跟着变。
- 钉：新池新建文件的用例钉死字面值「vec![65537, 65537, 0, 0, 0, 0, 0, 65537, 65537, 0, 0],」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1256`）。这条只管 mkfs 水位发号那一版。
- 读：池级 checker 与 GPU 判器都不读偏移 116 的头 ID。命令与原样输出（在 `crates/` 冻结副本里跑）：

```
$ cd /tmp/claude-1000/c673-clauses-r1/tree/crates; grep -cE '\b116\b' singlefs-checker/src/walk.rs; grep -cE '116u' singlefs-checker-tier/src/crash_judge_checker.wgsl; grep -rnE 'head_identifier|head_id' singlefs-checker singlefs-checker-tier/src | wc -l
0
0
0
```

  树表条目偏移（`walk.rs` 自己的常量）：「const TREE_TABLE_ENTRY_PREVIOUS_SNAPSHOT_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86;」（`crates/singlefs-checker/src/walk.rs:162`）= 100，诞生 txg「const TREE_TABLE_ENTRY_BIRTH_TXG_OFFSET: usize = 8 + 2 + 2 + 2 + 86 + 8;」（`crates/singlefs-checker/src/walk.rs:2943`）= 108，头 ID 在 108 + 8 = 116。
- 树表条目判的现状：每版判一次的表级判定「fn judge_tree_table_entries_ordering(」（`crates/singlefs-checker/src/walk.rs:2953`）在读出树表节点之后调（`crates/singlefs-checker/src/walk.rs:1102`）；逐条判的「fn walk_tree_table_entry(&mut self, entry: &[u8]) {」（`crates/singlefs-checker/src/walk.rs:1452`）只读树 ID、种类与 `previous_snapshot_txg`。GPU 对应：「fn w_judge_tree_table_entries_ordering(n: Node) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1417`）、调用点「w_judge_tree_table_entries_ordering(tt);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2564`）。
- E171 第 271 行记的就是这个洞：「头 ID 不判」（`.claude/kb/experiments/171-三oracle合谋测试.md:171`）。

### 1.2　问题 1：字面 65537，还是「同一张树表里 inode 树条目的树 ID」

**判：读后者。** 理由三条，都能核：

1. D5 已定项 9 的定义是关系（「可写头的 ID = 该头 inode 树的树 ID」，`.claude/kb/decisions/05-快照-空间记账机制.md:227`），65537 是那条关系在 mkfs 水位发号那一版上的取值，D8 第 250 行括注里的 65537 是同一个意思。字面读法让 D8 的括注压过 D5 的定义。
2. 实现自己就按关系写（`trees.inode.0`，`crates/singlefs-core/src/transaction.rs:8178`），checker 对同类问题已经按关系读：映射树的树 ID 取根记录里指针的出生树，不写死 65540（「号不写死映射树的号（mkfs 那条流上是 65540）」，`crates/singlefs-checker/src/walk.rs:1112`）。
3. 字面读法在什么历史上红（举出来）：
   - 历史 A（今天的代码能发出，用例没造）：第一个文件那条根在环里读得到（水位 65548 在环里），而恢复按实例表把它判成被抛弃、落到树表 0 条的暖机根上，新实例再发第一个文件版本。这时 `issued_from_watermark` 从 65548 起发，inode 树 = 65549，extent = 65548，`deleted_inodes` = 65556，墓碑树 = 65557（算术在 `model.out` 第 2 行，`label=clean_issued_from_65548`：字面读法红 `['extent', 'inode', 'deleted_inodes', 'tombstone']` 四条、相对读法 0 条）。
   - 历史 B（条款允许、今天没实现）：多头。D5 已定项 9 逐字让每个可写头有自己的 inode 树，克隆头的 inode 树的树 ID 不是 65537，它的 extent 树头 ID 自然也不是。
   - ⚠️ 管理员回退之后再建不在这一族里：「向前回退的候选要带文件，退不到树表 0 条的暖机根」（`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs:6`）。我起初想用「回退到暖机根再建」当例子，这一条把它排除了，不用它。
   - 今天的用例镜像上字面读法一格不红：全仓没有一处用例或装置造出 inode 树 ID ≠ 65537 的镜像（现跑：`cd /tmp/claude-1000/c673-clauses-r1/tree; grep -rn '65549\|65561' crates | wc -l` 输出 `0`），所以历史 A 是推的（读码加算术），要主 agent 在副本上造一次才能当观测。造法：用例 `the_row_publish_after_the_first_file_root_is_abandoned_carries_the_highest_watermark_in_the_ring`（`crates/singlefs-harness/tests/tree_identifier_watermark_after_the_first_file_root_is_abandoned.rs:134`）的前半（藏起第一个文件的根槽与数据单元、落到暖机根、新实例可写挂载）之后，把那条根还原成读得出（该用例自己的做法），再对暖机那一版发 `publish_first_file`。

### 1.3　问题 2：一张树表里 inode 树条目 0 条或多于 1 条

今天合法的树表只有三种形态：0 条（mkfs 与暖机）、11 条（第一个文件版本起，每种种类各一条）。11 条里 inode 树条目恰一条；core 另在读树表时拒同一种树两条（「树表里同一种树有两条」，`crates/singlefs-core/src/recovery.rs:2277`，判在「pub(crate) fn tree_table_entries_each_kind_at_most_once(」，`crates/singlefs-core/src/recovery.rs:2308`），池级 checker 对同一种两条故意不另立判定（`crates/singlefs-checker/src/walk.rs:2985`），`invariants.md` 里也没有「同一种树至多一条」这一行（现跑：`grep -nE '同一种树|每种树至多' .claude/kb/invariants.md | wc -l` 输出 `0`）。所以：

| 树表 | 判 | 理由 |
|---|---|---|
| 0 条 | 一条也不判（这条不变量评估 0 次，记「不适用」） | 没有条目就没有头 ID |
| inode 树条目恰一条（今天唯一的非空形态） | 逐条判，H = 那条的树 ID | 严格读法，等价于「头 ID == H」 |
| 非空而 inode 树条目 0 条 | extent / `deleted_inodes` / 墓碑树三种条目判红（它们服务的头不在这张表里）；inode 树自己与写 0 的几种不受影响 | D5 已定项 9「写它服务的那个头的树 ID」：表里没有头，这三种的取值无处可对 |
| inode 树条目多于 1 条 | 不判重复本身；inode 树条目仍要等于自己的树 ID，extent / `deleted_inodes` / 墓碑树的头 ID 放宽成「是表里某条 inode 树条目的树 ID」 | 今天 core 已拒，写不进合法历史；多头之后只看单镜像分不出哪条 extent 树服务哪个头（得靠 inode 记录或快照谱系，单镜像表里没有），放宽读法不会在合法的多头表上误红，收严留给多头那一批（记欠） |

### 1.4　问题 3：按条目判还是按一版判；放进哪一条不变量

**判：判据是表级（一张树表一个谓词），评估按条目计数（每条种类登记过的条目一次 `judge`，第一处违例写明第几条、什么种类、读到什么、该是什么）；新立一条，不并进已有的。**

- 为什么表级：extent、`deleted_inodes`、墓碑树三种的取值要对同表里 inode 树条目，单条条目手里没有 H。`walk_tree_table_entry` 只拿到一条条目（`crates/singlefs-checker/src/walk.rs:1452`），所以判放在读出树表节点之后的表级位置，紧挨 I-9.16 的调用（`crates/singlefs-checker/src/walk.rs:1102`）；GPU 放在 `w_judge_tree_table_entries_ordering(tt);` 同一处（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2564`）。
- 树表单元被几版共享时的口径：读出树表节点的那一步对已走过的单元返回空（「if !self.visited_units.insert(placement) {」，`crates/singlefs-checker/src/walk.rs:969`），所以表级判定只在第一个走到这片树表的版本上判一次。头 ID 的取值只是树表字节的函数，不依赖哪一版的 txg（I-9.25 才依赖，见 `crates/singlefs-checker/src/walk.rs:1467`），共享它的别的版本字节相同，判一次不丢东西。
- 为什么不并进 I-9.16：core 把 I-9.16 当成读树表时拒绝挂载的常量名（「pub const TREE_TABLE_ENTRIES_ORDERING_CONTRACT: &str = "I-9.16";」，`crates/singlefs-core/src/recovery.rs:2328`，用在 `crates/singlefs-core/src/recovery.rs:2374` 与 `:2407` 的拒绝里）。并进去等于让头 ID 不对也变成「读任何一棵树之前拒」，条款没要这个行为变更。I-9.16 的判别力又落在发号次序那一道（`.claude/kb/invariants.md:302` 写明「判别力落在 ②」）。
- 为什么不并进 I-9.25：它的定义域是 `previous_snapshot_txg`，判法随这一版的 txg 变（`.claude/kb/invariants.md:311`），表级字节关系塞进去会让这一行的「已实现」说明与坏镜像对不上号。
- 新立的编号：下一个没用过的是 I-9.27（现跑：`grep -rn 'I-9\.27\|I_9_27' .claude/kb /tmp/claude-1000/c673-clauses-r1/tree/crates | wc -l` 输出 `0`；今天最后一条是 `.claude/kb/invariants.md:312` 的 I-9.26）。

### 1.5　问题 4：GPU 判器同判的代价与写法

覆盖表同长的编译期断言逼着内核同补：`IMPLEMENTED_INVARIANTS` 是定长数组「pub const IMPLEMENTED_INVARIANTS: [&str; 65] = [」（`crates/singlefs-checker/src/image.rs:72`），覆盖表「pub const POOL_CHECKER_COVERED_INVARIANT_INDEXES: &[usize] = &[」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:63`）长度不等或次序不对编译不过（断言在 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:72` 起）。位宽有余：违例位容量 128（「pub const POOL_CHECKER_VIOLATED_BIT_CAPACITY: usize = POOL_CHECKER_VIOLATED_WORDS * 32;」，`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:41`），66 条装得下。

要改的地方（逐条，都是对 `crates/` 的改动）：

1. `crates/singlefs-checker/src/image.rs:72`：数组长度 65 → 66，末尾追加 `"I-9.27"`。
2. `crates/singlefs-checker/src/walk.rs`：紧接 `judge_tree_table_entries_ordering`（`:2953`）新写 `judge_tree_table_head_identifiers(judgements, entries, version_name)`，在 `:1102` 之后调。读偏移 0（树 ID）、10（种类）、116（头 ID，常量 `8+2+2+2+86+8+8`）。条目宽不到 124 字节时只判读得到的，其余走 `part_of_the_range_not_judged`（照 `crates/singlefs-checker/src/walk.rs:2978` 的做法）。
3. `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl`：在最后一条常量「const INVARIANT_I_8_5: u32 = 64u;」（`:138`）之后加 `const INVARIANT_I_9_27: u32 = 65u;`。
4. `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`：仿「fn w_judge_tree_table_entries_ordering(n: Node) {」（`:1417`）写 `w_judge_tree_table_head_identifiers(n: Node)`，在 `:2564` 那一行之后调。写法：外层按条目走；种类 2 的条目比 `rd64(o+116) == rd64(o)`；种类 1 / 9 / 10 的条目在内层再走一遍全部条目，找一条种类 2 且树 ID 等于这条头 ID 的（恰一条时就是严格读法）；种类 3 / 4 / 6 / 7 / 8 / 11 / 12 的条目比 `is_zero64`；别的种类码不判。纯函数、不碰草稿区（`SCRATCH_WORDS` 不动），条目 ≤ 11 条、内层循环常数小。
5. `crates/singlefs-checker-tier/src/crash_judge_gpu.rs:63`：下标列表补 65。
6. 对拍：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 逐位比、不屏蔽，补坏镜像之后它自然覆盖；坏镜像要在那个文件里为 I-9.27 各补一份，跟 I-9.25 同形（那里 `:1117` 起有 I-9.25 的先例）。
7. kb：`.claude/kb/invariants.md:12` 的「判 65 条」、`:16` 的「今天用了 65 位」、`:19` 的「现共 102 条在用」各 +1，并在 `:312`（I-9.26）之后加一行。

⚠️ 重型的部分我不跑：改内核之后整套崩溃放量要重判（26 亿态量级，提交 fcc4d340 的标题记的是「七个分项 26.16 亿态零红」，`git log -1 --format=%s fcc4d340` 现取），GPU ↔ CPU 逐态对拍与 `cargo test -p singlefs-checker-tier` 都是 checker 档，只归主 agent。

### 1.6　可直接写回的条款句（零轮）

**写回一：D8 已定项 8，第 250 行「头 ID 8 字节的取值」那一条，原句不动，句末追加：**

> ⚠️ 括号里的 65537 是树 ID 从 mkfs 水位 65536 起发的那一版（新池新建文件）上的取值，判据是关系、不是字面值：inode 树条目的头 ID 等于它自己的树 ID；extent 树、`deleted_inodes`、墓碑树条目的头 ID 等于同一张树表里 inode 树条目的树 ID；谱系树、logged_ops、分配记录树、记账树、livelist 共享树、稀疏旁表树、deadlist 树写 0。树 ID 不是从 mkfs 水位起发的那一版（水位已被抬过，或一个克隆头的 inode 树）上，头 ID 随这张表里 inode 树的树 ID 走，不再是 65537（D5 已定项 9「可写头的 ID = 该头 inode 树的树 ID」）。判它的是 I-9.27。

**写回二：`.claude/kb/invariants.md` 在 I-9.26 之后新加一行：**

> | I-9.27 | 树表条目的头 ID 取值 | 走读到的任一版（候选集里每条根，含由记录施加出来的那几版）的树表里，设 H 是种类码 2（inode 树）条目的树 ID（条目偏移 0）。逐条判偏移 116 的头 ID：种类 2 的条目 == 它自己的树 ID；种类 1（extent 树）、9（`deleted_inodes`）、10（墓碑树）的条目 == 这张树表里某条种类 2 条目的树 ID（恰一条时就是 H；表里没有种类 2 条目时判红）；种类 3、4、6、7、8、11、12 的条目 == 0；别的种类码不判。树表 0 条的版本一条也不判。不拿字面 65537 比：它只是 mkfs 水位发号那一版上的取值。依据 D8 已定项 8「头 ID 8 字节的取值」与 D5 已定项 9。表里有多于一条种类 2 条目时 extent 树等三种只判「是其中某一条的树 ID」，收严留给多头那一批。 | 未实现（零轮，C673 ⑤） |

### 1.7　会红的镜像、不该红的镜像（算术在 `model.out`，两种读法并排）

| 镜像（`label=`） | 字面 65537 读法 | 相对读法（本条） |
|---|---|---|
| 水位 65536 发的干净表（`clean_issued_from_65536`） | 不红 | 不红 |
| 水位 65548 发的干净表（`clean_issued_from_65548`，inode 树 65549） | 红 4 条（误红） | 不红 |
| E171 第 271 行：`deleted_inodes` 头 ID → 0（`e171_row271_deleted_inodes_head_zero`） | 红 `deleted_inodes` | 红 `deleted_inodes` |
| 水位 65548 的表上 inode 树头 ID 抄旧值 65537（`reissued_table_inode_head_copies_the_old_65537`） | 红 3 条，且判错了条目（红 extent 等三条，漏 inode） | 红 `inode` |
| 谱系树头 ID 写成 inode 树 ID（`lineage_head_not_zero`） | 红 | 红 |
| 拿掉 inode 树条目（`no_inode_tree_entry`） | 不红（漏） | 红 extent、`deleted_inodes`、墓碑树 |
| 两条 inode 树条目（`two_inode_tree_entries`） | 红第二条（今天这张表本身就不合法，core 会拒） | 不红（放宽，收严留多头） |
| 0 条（`empty_table`） | 不红 | 不评估 |

写进坏镜像用例（`crates/singlefs-harness/tests/checker_known_bad_images.rs`，那里 `fn violated_invariants(` 在 `:1811`）的最小一组：E171 第 271 行形态、inode 头 ID ≠ 自己、extent 头 ID ≠ H、谱系树头 ID ≠ 0，各只红 I-9.27；外加一张**绿镜像**：水位 65548 发号的干净镜像（`crates/` 里没有现成的，造法见 §1.2 末条），它是判别「字面读」与「相对读」的那一张——只有它在，字面读才会被抓出来。

### 1.8　K1 什么现象会推翻它

- 造出水位 65548 的干净镜像（§1.2 造法）、在副本上用 `check_pool_image` 判：本条判红 ⇒ 相对读法不成立，或实现写头 ID 的路径与我读的 `trees.inode.0` 不同。
- 发现某个合法历史里 extent 树条目的头 ID 不等于同表 inode 树的树 ID（例如多头之后 extent 服务别的头）：放宽读法照样站得住，严格读法（恰一条）要收回。
- 条款方改判「头 ID 是个常量登记值」（把 D5 已定项 9 的定义句改掉）⇒ 回到字面读法，本节作废。
- 字面读法在一张今天用例里已有的合法镜像上红（我 `grep` 零命中说明目前没有；有就推翻 §1.2 第 3 点）。

## 二、K2　新池新建文件的落点怎么判（C673 ⑥）

### 2.1　条款与实现今天的样子

条款（kb 原文，行号现查）：

- 用户 2026-10-03 定的例外：「新池新建文件是例外」…「算格式级，偏离它写出的新池新建文件镜像不合法」（`.claude/kb/decisions/03-空间分配.md:171`）；同一句的后半「其余的写（覆盖写、之后的发布、重挂之后）上违反它的实现写出的镜像合法」（`.claude/kb/decisions/03-空间分配.md:171`）。「偏离即不合法」只写在已定项 8 射程，且只点了 t1。
- 已定项 10 ③：「③ 数据单元取最低的、起点 32768 对齐、两槽都空」（`.claude/kb/decisions/03-空间分配.md:205`）；④ 单元区起点：「④ 系统配置几何段加「单元区起始槽号」8 字节」（`.claude/kb/decisions/03-空间分配.md:205`），同一行写「默认 768 MiB 环是 50176」（`.claude/kb/decisions/03-空间分配.md:205`）。
- 已定项 10 射程：「**改新池新建文件的字节：是**——七个提交内生块的落点整体重排」（`.claude/kb/decisions/03-空间分配.md:221`）。这一句只说字节算格式级，没有「偏离即不合法」。
- 登记：t1 行「| t1 | 事务 | 数据单元（文件内容）」（`.claude/kb/layout/01-first-txn.md:67`，槽号 50180–50181）、政策行「| 用户数据落点政策 |」（`.claude/kb/layout/01-first-txn.md:309`，写「已定（2026-09-13）」）。

代码（冻结副本）：

- 首次适配：「pub fn lowest_user_data_slot(」（`crates/singlefs-core/src/allocator.rs:736`）从单元区起点起按 2 槽步进，取第一个两槽都空的偶数槽；空不空看「pub fn is_free(&self, slot: SlotNumber) -> bool {」（`crates/singlefs-core/src/allocator.rs:518`），它同时排除已分配、影子账隔离、抬 F 扣住的槽。
- 影子账隔离的是「只被被抛弃根引用的槽」（`crates/singlefs-core/src/mount.rs:1336` 起的注释与 `isolate_slots_referenced_only_by_abandoned_roots`）。所以第一个文件那条根若在环里读得到、又被恢复判成被抛弃，t1 的两槽会被隔离，新实例的 t1 就不是 50180（推的，读码；没跑）。
- 池级 checker 已经把单元区起点读进几何：「let unit_area_start_slot = read_u64(slot, 417);」（`crates/singlefs-checker/src/image.rs:328`）；GPU 内核里有 `handoff_unit_area_start_slot()`（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:692`）。
- 钉（harness 档，按字节）：「("data_unit",                       0, 50180 * 16384, 32768, HeadAndTail), // t1」（`crates/singlefs-harness/tests/new_pool_file_creation_write_regions.rs:110`）与「assert_eq!(placement.slot.0, 50180, "t1@50180");」（`crates/singlefs-harness/tests/data_unit_allocation_and_write.rs:187`）。E171 第 15 行那条变异（`crates/mutations.tsv:1040`）点名的用例是自由图单测「fn a_free_map_whose_unit_area_follows_a_128_mebibyte_journal_ring_hands_out_slots_right_after_the_ring(」（`crates/singlefs-harness/tests/core_review_unit_area_start_and_publish_limits.rs:430`），它在 128 MiB 环的空闲图上钉，不钉默认环的 50180。
- E171 第 15 行的读数：用户数据落点 822149120 → 823132160（`.claude/kb/experiments/171-三oracle合谋测试.md:86` 的「822149120 → 823132160」）。÷16384 = 50180 → 50240，我算过（`python3 -c "print(822149120/16384, 823132160/16384)"` 输出 `50180.0 50240.0`），模型里「扫描起点 + 64 槽」那条变异也给 50240（`model.out` 的 `name=k2_first_fit label=default_768MiB` 行，`mutated_scan_from_U_plus_64_t1=50240`）。

### 2.2　要连同答的四问

**问 a：照 `layout/01-first-txn.md` 逐字段去认，会不会变空真？——会，认法里不能带树表条数。**

| 字段 | `layout/01` 写 | 实现今天 | 认法里能不能用 |
|---|---|---|---|
| 树表条数 | 「| t12 | 事务 | 树表单元第 1 版（7 条条目」（`.claude/kb/layout/01-first-txn.md:78`），七节表「| 树表条目 | **树 ID 8**（打头」行写「第 1 版 **7 条**」（`.claude/kb/layout/01-first-txn.md:387`） | 11 条：「pub const NEW_POOL_FILE_CREATION_TREE_TABLE_ENTRIES: u64 = 11;」（`crates/singlefs-format/src/lib.rs:188`），用例「assert_eq!(entries.len(), 11,」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1241`）；`layout/03` 另写「树表 7 → 11 条」（`.claude/kb/layout/03-third-txn.md:15`） | **不能**：认「恰 7 条」永不命中，不变量对任何镜像恒真（空真），坏实现也过 |
| extent 树的树 ID | 65536（`.claude/kb/layout/01-first-txn.md:49` 写「树 ID 水位在 mkfs 时写 65536、第一次发布后写 **65548**」） | 从水位发号（`crates/singlefs-core/src/transaction.rs:2597`） | 能：水位 = mkfs 种下的 65536 时 extent 树就是 65536 |
| 树表条目诞生 txg == 发布 txg | D8 已定项 8 ① 的口径 | 用例钉全部 11 条都等于 3：「.all(|entry| entry.birth_txg == CheckpointTxg(3)));」（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1267`） | 能：它把「建树那一版」与之后的覆盖写版本分开（后者的诞生 txg 小于它们的 txg，I-9.14） |
| 实例代号 1、txg 3 | 暖机后的第一次发布 | `NEW_POOL_FILE_CREATION_TXG` 只管 mkfs 同一个进程里那条流，只做过 mkfs 的池重开再写文件时那一版已经是 txg 5（`crates/singlefs-core/src/transaction.rs:5554` 起的注释，2026-09-23 用户定案） | **不能当判据**：写成 (1, 3) 会把第一个文件在第二个实例里写的版本全放过去（漏，不是误红） |
| 单元区起点 50176 | 默认环 | 非默认环真有合法镜像：用例按 1 GiB 环（起点 66560）与 128 MiB 环（起点 9216）造第一个文件、挂载、冷读、判池级 checker，断言「池级 checker 按偏移 417 那 8 字节划单元区，一条不红」（`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:406`，两条用例在 `:432`、`:438`） | **不能写死 50180**：这两张合法镜像的 t1 不在 50180（66560 起的盘上 50180 连单元区都不在） |

另一处 kb 与 kb 不同步：已定项 10 的落点表还写 t2 的树 ID 是 11（「| t2 | extent 树根兼叶（码 2，16 KiB） | 11 | 50240 | 0 |」，`.claude/kb/decisions/03-空间分配.md:211`），`layout/01` 写 65536 起（`.claude/kb/layout/01-first-txn.md:68`）。不影响 K2，随这一轮的写回一起记给 kb-scribe。

**问 b：非默认 journal 环（单元区起点不是 50176）算不算？——算，t1 = U + 4，U 读系统配置偏移 417。**
mkfs 在 U、U+1 放实例表（2 槽）、在 U+2 放树表（1 槽），U 是 64 槽段边界上的偶数槽（D3 已定项 10 ④「起点不在 64 槽段边界上在任何写之前拒」），第一个 32768 对齐的空槽对因此恒是 U+4。`model.out` 三行：

```
name=k2_first_fit label=default_768MiB unit_area_start=50176 aligned_64=True open_segment_start=50240 t1=50180 t1_minus_U=4 mutated_scan_from_U_plus_64_t1=50240
name=k2_first_fit label=128MiB unit_area_start=9216 aligned_64=True open_segment_start=9280 t1=9220 t1_minus_U=4 mutated_scan_from_U_plus_64_t1=9280
name=k2_first_fit label=1GiB unit_area_start=66560 aligned_64=True open_segment_start=66624 t1=66564 t1_minus_U=4 mutated_scan_from_U_plus_64_t1=66624
```

⚠️ 默认环那一行（50180）有 harness 钉；9220 与 66564 是我从首次适配规则算的，用例没钉（上面那两条用例只钉「分配记录都落在 U 之后」，没钉 t1），属于「推的」。条款若只想判默认环，就把句子写成字面 50180 并说明「非默认环不判」；我推荐写 U + 4（理由：字面 50180 在 U ≠ 50176 的合法镜像上恒红，见上表末行）。

**问 c：第一次发布写多个数据单元或零长文件算不算？**
`layout/01` 登记的只有 3000 字节、一个数据单元的那条（`.claude/kb/decisions/08-核心索引结构.md:195` 写「新池新建文件那条记录 3000 字节 ⇒ 6」）。已定项 8 射程点的是「那一次用户数据的落点取值（t1@50180）」，单数。判据写成「每块盘上落点槽号最小的那个数据单元 == U + 4」：多单元时第一个写的单元仍是最低的偶数空槽对（首次适配），后面的单元不判；零长文件没有数据单元，评估 0 次。这条是我从首次适配推的（零轮），保守的替代写法是「恰一个数据单元才判」。

**问 d：t2–t12 要不要一起判？——这一轮不要。**
「偏离即不合法」只写在已定项 8 射程、只点 t1（`.claude/kb/decisions/03-空间分配.md:171`）；已定项 10 射程只说 ⑤ 的落点「改新池新建文件的字节：是」（`.claude/kb/decisions/03-空间分配.md:221`），没有那一句。t2–t12 的落点还依赖盘数与盘大小（分配记录树的节点数随树高变，`.claude/kb/layout/01-first-txn.md:75` 写「两块 4 GiB 盘时树高 3」），不是 U 的闭式。E171 第 15 行的变异把 t1 挪了，只判 t1 就够它红（`.claude/kb/experiments/171-三oracle合谋测试.md:86` 的变异落点：t1 50180 → 50240）。要判 t2–t12 得先由用户在已定项 10 射程补一句「偏离即不合法」，再单独立一条，另一轮。

### 2.3　单镜像怎么认「新池新建文件那一版」（甲的认法，写得成条款的一版）

实现员推的认法（根的实例代号 1、`checkpoint_txg` 3、系统配置里单元区起点 50176、extent 树恰好引用一个数据单元且 inode 树恰好一条记录）按 §2.2 前两问逐条核过：实例代号 / txg 与单元区起点 50176 不能写死，条数认法不能带树表条数。我改成只用镜像里读得到、且今天的实现与条款都站得住的两个量：

> 设 V 是被走读的一版（候选集里每条根，含由记录施加出来的那几版）。V 是「池里第一次建树的那一版」当且仅当：① V 的树表非空，且种类 1（extent 树）条目的树 ID == 65536（D8 已定项 11 登记的 extent 树号，也是 mkfs 种下的水位）；② V 的树表里每一条条目的诞生 txg（条目偏移 108）都等于 V 的 `checkpoint_txg`。

两条各管什么：

- ① 管「环里没有可见的、带文件的根」：树从 mkfs 水位发号 ⇔ 发号那一刻环里读得出的根里没有带水位 65548 的（`crates/singlefs-core/src/transaction.rs:2549` 起的注释：水位取环里的 max）；而影子账隔离 t1 的前提是环里有一条被抛弃的、读得到的带文件根（`crates/singlefs-core/src/mount.rs:1336` 起）。两者读的是同一批可读根，所以 ① 为真时 U+4 没被隔离（推的，读码；两遍读不一致的情形见 §2.5 风险 a）。
- ② 管「V 就是建树的那次发布」：覆盖写与之后的版本树还是那几棵，诞生 txg 小于它们自己的 txg（I-9.14 钉住的跨根不变），所以不会被误认；用例钉着第一个文件版本全部 11 条诞生 txg 都等于发布 txg（`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1267`）。

判据（对认出来的 V）：

> V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号都等于 U + 4。U 是所选系统配置偏移 417 的单元区起始槽号（默认 768 MiB 环 U = 50176，所以是 50180）。V 没有数据单元时不判。不满足 ①② 的版本不判。

它在空真上站得住：①②在 E142 干跑那条流的第一个文件版本上都为真（用例钉 11 条诞生 txg 全等于 3、树 ID 从 65536 起，`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs:1241` 起），所以不是恒不评估；写进用例时还要断言干净镜像上这条评估次数 ≥ 1（`Judgements` 数每条评估几次：「evaluated: BTreeMap<&'static str, u64>,」，`crates/singlefs-checker/src/image.rs:181`），让「认法悄悄再也认不出任何版本」变成看得见的红。

### 2.4　三个候选各自：要的条款、会判红的镜像、误红的风险

**甲：O2 新立一条不变量（上面的认法 + 判据）。**

- 要的条款：D3 已定项 8 射程那一句补「t1 = 单元区起始槽号 + 4，默认环即 50180」（见 §2.6 写回一）；`invariants.md` I-5（分配器）类新加一行，编号 I-5.5（`grep -c 'I-5\.5' .claude/kb/invariants.md` 输出 `0`，最后一行是 I-5.4，`.claude/kb/invariants.md:198`）。
- 实现：CPU 在 `crates/singlefs-checker/src/walk.rs` 每版开头清零两个累加器（每盘最小的数据单元槽号）、在「fn walk_extent_data_pointer(」（`crates/singlefs-checker/src/walk.rs:2272`）里记、每版走完判；认法用到的树表条目诞生 txg 已有读取（`tree_table_birth_txg`，`crates/singlefs-checker/src/walk.rs:3072`）。GPU 在「fn w_walk_extent_data_pointer(」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:1453`）里同样记、用 `handoff_unit_area_start_slot()`（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:692`）取 U；还要加一个每版的小状态与判点，比 K1 的纯函数重。覆盖表、`IMPLEMENTED_INVARIANTS` 同长断言照 §1.5（65 → 66 若与 K1 同批则 → 67）。
- 会判红：E171 第 15 行那条变异（t1 50180 → 50240，指针与校验和随之变）：该镜像里 V 的 ①② 照旧为真、最小槽号 50240 ≠ 50180，红；t1 挪 2 槽到 50182 同理红。E171 现在记的是「`o2_red_states=0`」（`.claude/kb/experiments/171-三oracle合谋测试.md:52`），补完要在 E171 复跑里变成大于 0。
- 不该红：默认环、非默认环（U = 9216、66560，t1 = U + 4）的干净第一个文件镜像；覆盖写那一版（②不成立，不判）；零长文件（无数据单元）；第一个文件在第二个实例里写（txg 5，认法没用实例代号）；mkfs 与暖机的树表 0 条的版本（①不成立）。
- 误红的风险（推的，要攻方造镜像）：见 §2.5。

**乙：不在 O2 判，放 O1（模型给新池新建文件的落点答）。**

- 要的条款：D13 的 O1 行要改。O1 一行今天写「POSIX 可见语义：内容、大小、目录结构」，边界列写「不碰盘、不解析磁盘格式」（`.claude/kb/decisions/13-验证路线.md:161`）；落点是盘上的取值，放进 O1 就要先改这一行的边界。
- 实现：`ObservedAllocationRecord` 今天只带盘、分配代、是否已释放，没有槽号（「pub struct ObservedAllocationRecord {」，`crates/singlefs-harness/src/model.rs:865`），要加槽号、模型要知道单元区起点与「这一版是不是池里第一个文件」。
- 会判红：对拍历史里第一个文件那次调用交回的 t1 ≠ 模型的答，E171 第 15 行那条变异会红。不吃崩溃镜像（O1 的崩溃判官只比读回）。
- 误红的风险：低，模型知道历史（第一个文件是不是池里第一个）；但模型要装一份「首次适配 = U + 4」的闭式，等于把分配政策抄进模型，与实现的分配器同源（D13 表里 O1 的「不能证明的」一列也写着两份实现对同一条语义的共同误解）。
- 不选的主要理由：改变 D13 里 O1 的定义；判不了别的写者写出的镜像；E171 记的「三条都不判落点」只是把洞从 O1 / O2 / O3 换成 O1 一条里补，崩溃放量的镜像仍没人判。

**丙：不新立判据，交给按字节钉写的 harness 用例，C673 ⑥ 改写。**

- 要的条款：`.claude/kb/checks-owed.md` 的 C673 ⑥ 改写成「新池新建文件落点不由三条 oracle 判，由钉写的用例判（`crates/singlefs-harness/tests/new_pool_file_creation_write_regions.rs:110`、`crates/singlefs-harness/tests/data_unit_allocation_and_write.rs:187`）」；D3 已定项 8 射程那句「新池新建文件那一格的检查欠在 C673 ⑥」随之改成「由钉写的用例判」。
- 会判红：只有实现自己的用例；E171 第 15 行的变异在变异表里本来就被自由图单测杀掉（`crates/mutations.tsv:1040`），三条 oracle 仍放行。
- 误红的风险：0。
- 代价：「镜像不合法」只在用例钉的那一个实现上成立，另一个写者写出的 t1 ≠ 50180 的镜像没人判；非默认环的 t1 也没人钉（§2.2 问 b）。这与 kb 里别的 layout 字节（t2–t12、树 ID 起点）今天的处理一致（它们也只靠钉写）。

### 2.5　推荐：甲（收窄成上面的认法与判据）；若复跑出现合法格上的红，退到丙。乙不选。

理由：

1. 用户 2026-10-03 定的是「偏离它写出的新池新建文件镜像不合法」，说的是镜像的合法性；D13 已定项 7 逐字把 O2 定义成对单个镜像的一元谓词（「O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员」，`.claude/kb/decisions/13-验证路线.md:15`）。只有甲让这句话对任意镜像（含崩溃放量的几十亿个态、别的写者写出的）成立；丙只对钉死的那一个实现成立，乙连崩溃镜像都不碰。
2. 认法只用 kb 与代码里都站得住的两个量，避开了会空真的树表条数和不能写死的 (1, 3) / 50176；单元区起点读系统配置，非默认环的合法镜像不误红。
3. 判据只判 t1，与用户这次定的射程一致；t2–t12 没有「偏离即不合法」那一句，不替用户扩。

风险与退路：

- a. **两遍读不一致**（推的，没造出来）：mount 里算水位与算影子账分别读环，代码自己承认两次读环之间会有瞬时读错（「两次读环之间的瞬时读错」，`crates/singlefs-core/src/mount.rs:1250`）。若第一个文件那条根只被影子账那一遍读到、水位那一遍读不到（瞬时读故障），新实例的 extent 树仍是 65536（① 成立），而 t1 的两槽被隔离、落到别处，甲在这个合法历史上误红。D8 已定项 8 ② 自己承认这一类「盖不住」并记在 C342。收紧办法（可选）：认法加 ③「环里读得出的每条自证根里，没有被实例表判成被抛弃、树表又非空的」，CPU 有被抛弃判定（`is_abandoned`，`crates/singlefs-core/src/mount.rs:1350` 一带是 core 的写法，checker 的在 I-9.14 的收窄里），GPU 要再补一块，代价明显大。我的建议是先不加 ③，让攻方造这一格，真红了再加。
- b. **同一实例内发布中途失败后重试**：我没读到分配器在中途失败后回滚到哪（`crates/singlefs-core/src/transaction.rs:5571` 的 `publish_first_file` 只说「都在任何写之前返回」那几类拒绝），推不出 t1 会不会变，留给攻方。
- c. **检查档里已有的第二个第一个文件**：`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs:312` 的流，崩在第一个文件版本的每个前缀上，可写挂载之后「接着现行那一版发第一个文件版本」（动作分支在 `:176`），池级 checker 要求一条违例都没有。这些镜像里新实例在 txg ≥ 4 上再建一次树，树 ID 从暖机根的水位 65536 起（水位 65548 的根没落盘），甲会认出它们。按 §2.1 的读码，那时分配器是挂载交出来的（「let mut allocator = mounted.allocator.clone();」，`crates/singlefs-checker-tier/tests/crash_points_tree_identifier_watermark_orphans.rs:177`），影子账没有可读的被抛弃根可隔离，t1 应是 U + 4（推的）；这是 §2.3 的判据第一次碰到的真实复杂格，**要主 agent 在副本上跑这个用例（checker 档）看是否仍一条不红**，我不能跑。

### 2.6　可直接写回的条款句（零轮）

**写回一：D3 已定项 8 射程（第 171 行），把括号里的「（t1@50180）」之后补：**

> （t1@50180，即单元区起始槽号 U 加 4：mkfs 的实例表占 U、U + 1，树表占 U + 2，第一个 32768 对齐的空槽对从 U + 4 起；默认 768 MiB 环 U = 50176，环长不是默认值时 U 按已定项 10 ④ 现算、t1 仍是 U + 4）

**写回二：`.claude/kb/invariants.md` I-5 类，I-5.4（第 198 行）之后新加：**

> | I-5.5 | 新池新建文件的用户数据落点 | 走读到的任一版 V（候选集里每条根，含由记录施加出来的那几版）若满足 ① V 的树表非空且种类 1（extent 树）条目的树 ID == 65536；② V 的树表里每条条目的诞生 txg（偏移 108）都等于 V 的 `checkpoint_txg`，则 V 引用的数据单元里，每块盘上落点槽号最小的那一个，槽号 == U + 4，U 是所选系统配置偏移 417 的单元区起始槽号（默认环 U = 50176 ⇒ 50180）。V 没有数据单元时不判；不满足 ①② 的版本不判。依据 D3 已定项 8 射程「偏离它写出的新池新建文件镜像不合法」与已定项 10 ③ ④。只判 t1，t2–t12 不判（已定项 10 射程没有「偏离即不合法」）。 | 未实现（零轮，C673 ⑥） |

**写回三（若 K2 改选丙）：** `.claude/kb/checks-owed.md` 的 C673 ⑥ 与 D3 已定项 8 射程末句各改成「由钉写的用例判」，不新立不变量。

### 2.7　K2 什么现象会推翻它

- 在副本上造一张合法镜像（§2.5 风险 a 或 c），甲认出来、t1 ≠ U + 4 ⇒ 甲误红，退到丙。
- E171 复跑里第 15 行在甲落地后 O2 仍 0 态红（`o2_red_states=0` 不变）⇒ 认法认不出那条流上的第一个文件版本（§2.3 的 ① 或 ② 在那张镜像上为假），甲没补上洞。
- 「恒评估 0 次」：写进用例的 `evaluated ≥ 1` 断言红 ⇒ 空真，认法要改。
- 用户补一句「t2–t12 偏离即不合法」⇒ §2.2 问 d 的结论要重做。

## 三、K3　D8 已定项 6 的写入时间（C673 ⑩）

### 3.1　条款与实现今天的样子

条款：

- D8 已定项 6 的字段表只定了位置与宽度：「| 64 | atime / mtime / ctime 秒，各 8 | 24 |」（`.claude/kb/decisions/08-核心索引结构.md:188`）、「| 96 | atime / mtime / ctime 纳秒，各 4 | 12 |」（`.claude/kb/decisions/08-核心索引结构.md:190`）。决策正文里没有「写入时间」一个字：`grep -cF '写入时间' .claude/kb/decisions/08-核心索引结构.md` 输出 `0`。
- 登记在字节表与里程碑：「| inode 记录（偏移 64） | atime / mtime / ctime 秒 | 24 | 写入时刻（mkfs 参数，可复现）」（`.claude/kb/layout/01-first-txn.md:267`）、「| inode 记录（偏移 96） | atime / mtime / ctime 纳秒 | 12 | 写入时刻 |」（`.claude/kb/layout/01-first-txn.md:269`）；里程碑一「写入时间是发布参数，不取系统时钟」（`.claude/kb/milestone/01-first-txn.md:103`）、里程碑二「写入时间是发布参数」（`.claude/kb/milestone/02-second-txn.md:103`）与验收「写入时间留成 A 的 ⇒ 判红」（`.claude/kb/milestone/02-second-txn.md:113`）。
- 删除步的相关条款：「①b 同一份崩溃镜像各自重挂载续做同一批，两份盘逐字节相同」（`.claude/kb/decisions/08-核心索引结构.md:464`）。

代码（冻结副本）：

- 记录写法：三个秒字段同取一个参数（「for _clock in 0..3 {」，`crates/singlefs-core/src/records.rs:77`），纳秒段整段留 0（「writer.skip(12);」，`crates/singlefs-core/src/records.rs:83`；`skip` 跳过的字节保持 0，「留位、补齐：跳过的字节保持 0。」在 `crates/singlefs-core/src/bytes.rs:38`）。
- 参数怎么进：新池新建文件与覆盖写各带一个「pub write_time_seconds: u64,」（`crates/singlefs-core/src/transaction.rs:3848`，`FirstFile`；`:3859`，`FileVersionPlan`）；建空 inode 经 `UserChange::NewInodes` 带参数（`crates/singlefs-core/src/mounted_session.rs:60`）。
- 不改记录的发布照抄：unlink 的意图与批各写「write_time_seconds: record.write_time_seconds,」（`crates/singlefs-core/src/transaction.rs:5874`、`:5936`），注释「写入时间不改：记录里没有 ctime 字段」（`crates/singlefs-core/src/transaction.rs:5836`）。
- 钉：覆盖写读回是 B 那一次的参数——「"盘上 inode 记录的写入时间是 B 那一次写入的时间，不是 A 的 {FIXED_WRITE_TIME_SECONDS}"」（`crates/singlefs-harness/tests/overwrite_in_one_instance.rs:403`）；E171 第 92 行那条变异（`crates/mutations.tsv:292`）由 `cold_start_reads_the_second_content_and_the_pool_checker_stays_green` 杀（我读了变异表那一行第 5、6 段，没跑）。

**两处不一致（kb 与代码，各把两句并排）：**

1. 来源：「| inode 记录（偏移 64） | atime / mtime / ctime 秒 | 24 | 写入时刻（mkfs 参数，可复现） | D8（核心索引结构） 已定项 6 | 已定 |」（`.claude/kb/layout/01-first-txn.md:267`）说是「mkfs 参数」；实现是每次发布各带一个参数（`crates/singlefs-core/src/transaction.rs:3859` 的 `FileVersionPlan` 一次覆盖写一个），里程碑二也写「写入时间是发布参数」（`.claude/kb/milestone/02-second-txn.md:103`）。以实现与里程碑为准，layout 那一格要改。
2. 纳秒：「| inode 记录（偏移 96） | atime / mtime / ctime 纳秒 | 12 | 写入时刻 | D8（核心索引结构） 已定项 6 | 已定 |」（`.claude/kb/layout/01-first-txn.md:269`）写「写入时刻」，实现整段写 0（`crates/singlefs-core/src/records.rs:83`），参数类型只有秒（`u64`）。要么条款写明纳秒取参数的纳秒部分、今天参数没有纳秒所以 0，要么给参数加纳秒；我按前者写（§3.2）。

### 3.2　补的那一句怎么写：放定案，射程另加一句

**判：取值规则放已定项 6 的定案（紧接字段表、在「`blocks` 是逻辑长度的 512 字节块数」那一段之前），「单镜像判不出」放射程。** 理由：取值规则决定盘上字节，与 `blocks` 那一段（同在字段表之后、同属偏移取值）同类，属于定案；射程管的是「条款管到哪」与「谁判得到」。它不改新池新建文件的字节（写进去的还是第一个文件那次发布的参数），所以不触发「改新池新建文件的字节」那一栏。

**写回一：D8 已定项 6 定案，字段表之后新加一段（零轮）：**

> **写入时间（偏移 64 起三个秒字段、偏移 96 起三个纳秒字段）是写这条记录的那一次发布的参数，不取系统时钟**：每次发布调用带一个写入时间，同一个参数两次发布逐字节相同（可复现）。三个秒字段都写参数的秒数；三个纳秒字段写参数的纳秒部分，今天的参数只带秒数，所以写 0。**新建**（新池新建文件、新建 inode）写创建那次发布的参数；**覆盖写与顺序写**重写这条记录时写这一次发布的参数，不照抄上一版的值（两次参数恰好相同时字节相同，但取值来源是这一次的参数）；**不改这条记录内容的发布**——同一片叶里别的记录改动而叶被重写、unlink 的意图与批（本项第 16 条要求同一份崩溃镜像续做出的盘逐字节相同，删除步没有确定的参数来源）——照抄这条记录上一版的值。

**写回二：D8 已定项 6 射程，句末追加（零轮）：**

> 写入时间取值是发布参数，单个镜像里看不出参数是什么：O2（单镜像一元谓词）判不到，O3 的投影不带它；今天判它的是钉写的用例（覆盖写读回 B 的参数）与变异表里「写入时间留成 A 的」那一行。

**写回三：`layout/01-first-txn.md` 第 267 行把「mkfs 参数」改成「发布参数」，第 269 行把「写入时刻」改成「写入时刻的纳秒部分（今天的参数只带秒，写 0）」（与定案句对齐）。**

### 3.3　三条 oracle 里哪一条判得到

| oracle | 判得到吗 | 依据 |
|---|---|---|
| O2（池级 checker） | 判不到 | D13 已定项 7「O2（独立解析器 + checker） 的定义域是单个镜像，只判集合成员」（`.claude/kb/decisions/13-验证路线.md:15`）；参数不在镜像里。比「覆盖写那一版与上一版的写入时间不同」也不行：两次参数相同是合法的 |
| O3（规约执行器） | 判不到 | E171 自己读码读出：「实现一致地写错的字段（容器号、写入时间、F 的取值、记账行）它照样判「在」」（`.claude/kb/experiments/171-三oracle合谋测试.md:543`） |
| O1（对拍模型） | 判得到，但今天的模型没有这个量 | `ObservedFile` 只带内容（「pub enum ObservedFile {」，`crates/singlefs-harness/src/model.rs:819`）；`grep -c 'write_time' crates/singlefs-harness/src/model.rs` 输出 `0`；调用参数在 harness 的 `history.rs` 里（`apply_publish_first_file` 带 `write_time_seconds`，`crates/singlefs-harness/src/history.rs:3019`）。要判就要：`ObservedFile` 加写入时间、模型记每次调用的参数、`unlink` 照抄。D13 O1 一行写「POSIX 可见语义：内容、大小、目录结构」（`.claude/kb/decisions/13-验证路线.md:161`），mtime 是 POSIX 可见属性但没列在内，加进去要先改这一行 |

判不到的怎么记：D13 表 O2 一行末列今天写「只判结构自洽：一个块被覆盖成另一份**合法**内容，Merkle 链照样自洽、记账照样对」（`.claude/kb/decisions/13-验证路线.md:162`），可在这一句后追加「发布参数的取值（写入时间）」；O3 一行同样追加。C673 ⑩ 本身的记法：「条款已补；三条 oracle 判不到；由钉写的用例与变异表第 292 行杀；O1 扩展写入时间是可选的后续，不是这一批的出口」。**我不推荐为它在 O2 里立「三个秒字段相等」这类弱关系**：E171 第 92 行那条变异（留 A 的写入时间）让三个字段仍然相等，这类关系抓不到它，白占一个不变量号和 GPU 内核。

### 3.4　K3 什么现象会推翻它

- 在副本上让覆盖写读回 A 的参数（变异表第 292 行那条）：`overwrite_in_one_instance` 里 `cold_start…` 仍绿 ⇒ 钉不住，§3.3「由钉写的用例杀」这句不成立，改记欠。
- 出现不属于「新建 / 覆盖写 / 不改记录」三类的会改记录的发布（truncate、chmod 一类）：定案句要补第四类；今天 `crates/` 里没有这些发布（`UserChange` 只有 `Unlink`、`Overwrite`、`SequentialWrite`、`NewInodes` 四个，`crates/singlefs-core/src/mounted_session.rs:60` 起），属于条款没说。
- 用户把纳秒段也定成要写非零值 ⇒ 定案句里「写 0」要改、参数类型要改。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 问 1 | 一致（读关系，不读字面） | D5 已定项 9 的定义是关系，实现写 `trees.inode.0`；字面 65537 在水位 ≠ 65536 发号的合法历史上误红（历史 A，推的，今天没有用例镜像造出它） |
| K1 问 2 | 规则没说 | 0 条评估 0 次；恰一条严格；0 条而表非空时 extent / `deleted_inodes` / 墓碑树判红；多于一条放宽成「是其中之一」并记欠 |
| K1 问 3 | 一致 | 判据表级、评估按条目；新立 I-9.27，不并 I-9.16（core 拿它拒挂载）、不并 I-9.25（随版本 txg 变） |
| K1 问 4 | 一致 | 纯函数、不碰草稿区；加 `IMPLEMENTED_INVARIANTS`、覆盖表下标、WGSL 常量与一个函数；重判崩溃放量归主 agent |
| K2 候选甲 | 推荐（收窄形态） | 认法：extent 树 ID == 65536 且全部条目诞生 txg == 本版 txg；判据：每盘最小数据单元槽号 == U + 4；风险 a 要攻方造 |
| K2 候选乙 | 冲突（与 D13 的 O1 行边界） | O1 一行写「不碰盘、不解析磁盘格式」；不选 |
| K2 候选丙 | 一致（零误红，镜像合法性只由钉写的用例保证） | 与 kb 里别的 layout 字节同法；退路 |
| K2 问 a（照 layout/01 认会不会空真） | 冲突 | `layout/01` 写 7 条（`:78`、`:387`），实现与 `layout/03` 写 11 条；认「恰 7 条」恒假 ⇒ 空真 |
| K2 问 b（非默认环） | 一致，推的 | t1 = U + 4；9220 与 66564 是算的，没有用例钉 |
| K2 问 c（多单元、零长） | 规则没说 | 取「每盘最小槽号」，零长不判；保守替代是「恰一个单元才判」 |
| K2 问 d（t2–t12） | 规则没说 | 「偏离即不合法」只在已定项 8 射程、只点 t1；这一轮不判 |
| K3 取值句 | 规则没说（D8 无此句）；layout 267 与实现冲突 | 补定案句（新建 / 覆盖写 / 不改记录三类），layout 267 的「mkfs 参数」与 269 的「写入时刻」要改 |
| K3 放哪 | 一致 | 定案放字段表之后，「单镜像判不出」放射程 |
| K3 谁判得到 | 规则没说 | O2、O3 判不到；O1 判得到但今天没有这个量；今天只有钉写的用例与变异表第 292 行 |

## 五、没做什么

- 没编译、没跑任何测试、没跑 cargo 与重型测试；`model.out` 的 13 行是 `python3` 纯算术，不是第二条独立路径（开头说过）。
- 要主 agent 复核、我复核不了的：① 造水位 65548 发号的干净镜像（§1.2 造法）并用 `check_pool_image` 判，核历史 A；② 在副本上跑 `crash_points_tree_identifier_watermark_orphans` 这条检查档用例，看甲落地后「第二个第一个文件」那一格是否仍一条不红（§2.5 风险 c）；③ 非默认环（U = 9216、66560）第一个文件的 t1 是不是 U + 4（§2.2 问 b，今天用例只钉到「分配记录都在 U 之后」）；④ GPU ↔ CPU 逐态对拍与崩溃放量重判（K1、K2 若落地都要）。
- 不替主 agent 采纳或出判决；写回条款句都是零轮，交用户表要标明。
- 没读禁读清单里的文件；没碰别的腿的产出。
- E171 页引 `03-空间分配.md` 的两个行号与今天差 1（它写第 206、220 行，今天是 `:207`（⑤ 落点表引导句）与 `:221`（已定项 10 射程）），不影响本报告，记给 kb-scribe。

## 附：`model.out` 原样（脚本复跑，与第一次逐字节相同；`sha256sum` 见交回）

```
name=k1_table label=clean_issued_from_65536 entries=11 inode_tree=65537 literal_red=[] relative_red=[]
name=k1_table label=clean_issued_from_65548 entries=11 inode_tree=65549 literal_red=['extent', 'inode', 'deleted_inodes', 'tombstone'] relative_red=[]
name=k1_table label=clean_issued_from_65560 entries=11 inode_tree=65561 literal_red=['extent', 'inode', 'deleted_inodes', 'tombstone'] relative_red=[]
name=k1_table label=e171_row271_deleted_inodes_head_zero entries=11 inode_tree=65537 literal_red=['deleted_inodes'] relative_red=['deleted_inodes']
name=k1_table label=e171_row271_on_reissued_table entries=11 inode_tree=65549 literal_red=['extent', 'inode', 'deleted_inodes', 'tombstone'] relative_red=['deleted_inodes']
name=k1_table label=reissued_table_inode_head_copies_the_old_65537 entries=11 inode_tree=65549 literal_red=['extent', 'deleted_inodes', 'tombstone'] relative_red=['inode']
name=k1_table label=lineage_head_not_zero entries=11 inode_tree=65537 literal_red=['lineage'] relative_red=['lineage']
name=k1_table label=no_inode_tree_entry entries=10 inode_tree=None literal_red=[] relative_red=['extent', 'deleted_inodes', 'tombstone']
name=k1_table label=two_inode_tree_entries entries=12 inode_tree=65537 literal_red=['inode_second'] relative_red=[]
name=k1_table label=empty_table entries=0 inode_tree=None literal_red=[] relative_red=[]
name=k2_first_fit label=default_768MiB unit_area_start=50176 aligned_64=True open_segment_start=50240 t1=50180 t1_minus_U=4 mutated_scan_from_U_plus_64_t1=50240
name=k2_first_fit label=128MiB unit_area_start=9216 aligned_64=True open_segment_start=9280 t1=9220 t1_minus_U=4 mutated_scan_from_U_plus_64_t1=9280
name=k2_first_fit label=1GiB unit_area_start=66560 aligned_64=True open_segment_start=66624 t1=66564 t1_minus_U=4 mutated_scan_from_U_plus_64_t1=66624
```

## 六、cite-check 末行

```
$ python3 research/scripts/cite-check.py research/prompts/c673-clauses-r1-sonnet-output.md --root /tmp/claude-1000/c673-clauses-r1/tree --root . --background research/prompts/_c673-clauses-r1-background.md
  ✓ 核了 62 处引文，对上 62 处，没判 0 处（逐处列在上面）
```
