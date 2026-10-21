# m3-close-c693-r1 云端正推腿报告（2026-10-20）

立场：从 E186、E71、E97 三份产物与 `crates/` 今天的实现出发，推 W1–W4 每一块该怎么改。不读攻方腿的任何产出；不替主 agent 采纳或判决。引 kb 条款写 kb 文件名加那份文件自己的行号（开工后现查），长行只引被改的那一句。

## 各格判定一览（开头版，细节与推翻条件在各节，末尾另有完整版）

| 格 | 判定 | 一句话 |
|---|---|---|
| W1 D8 已定项 8 ㊀ | 冲突（部分） | E186 的反向接受条款字面在读法 B1 下触发；正推结论是 ㊀ 的取舍（搬运）不翻面，但 ㊀ 与射程的字面有两处与实现、与一代口径不符，要改；不按 `r_tree_dim` 分支 |
| W2 D8 已定项 8 ㊁ | 一致（条款）+ 冲突（实现） | ㊁ 不改：`w_gens` 在挂载时不可知，分不了支，只能取在 `w_gens > 0` 上不丢数据的一档；`crates/singlefs-core/src/recovery.rs:2804` 整表拒是实现落后于 ㊁，改实现（随 deadlist 合并那一步），条款加一句进度注 |
| W3 D5 已定项 4 的依据与树高 | 冲突（依据句）+ 一致（取舍） | 「哪几项按树分」的取舍站得住，但站在「谁要它」上，代价侧依据句要重写（两句数对不上 E71/E97 一代口径，另有两处出处错）；「树高恒为 2」三处改成「段宽不动树高」 |
| W4 点删与墓碑 | 冲突 | `crates/` 今天两个待答问题都答「没有墓碑」，且 key 带代段使「同一 key 写了又点删」在记账树上结构性不出现；D5 已定项 3 第 3 行的墓碑句与 C344 在记账树上没有对象，销账（不是缩题）；墓碑折叠留在 D8 已定项 13 / C383 |

## 0 先立住的实现事实（命令与原样输出）

读过的路径与看到的事实（`crates/` 今天，行号现查）：

| 事实 | 出处 |
|---|---|
| 一次发布只给一个可写头（inode 树）写水位行，其余池级行与每盘六行都记在无归属树 ID 上 | `crates/singlefs-core/src/transaction.rs:6882`：「这一行记在哪棵树上：inode 号水位记在 inode 树上，另两行不属于任何一棵树。」 |
| 一次发布的行集 = 池级三行 + 每盘六行（代段 = 这次 txg）+ 从上一版照抄的水位台阶行 | `crates/singlefs-core/src/transaction.rs:6931`：「fn accounting_entries_of_this_publish(」与 `crates/singlefs-core/src/transaction.rs:6968`：「entries.extend_from_slice(inode_number_watermark_steps_carried_from_the_previous_version);」 |
| 台阶行按统计量认、不按树认：代段小于这一版 txg 的水位行都算台阶行 | `crates/singlefs-core/src/transaction.rs:3685`：「pub fn inode_number_watermark_steps(&self) -> Vec<AccountingEntry> {」与 `crates/singlefs-core/src/transaction.rs:3691`：「entry.statistic == STATISTIC_INODE_WATERMARK && entry.generation < version_txg」 |
| 实现自己写明克隆路径还没有，台阶行不按树段分 | `crates/singlefs-core/src/transaction.rs:3682`：「今天没有克隆路径」 |
| 读水位现行值 = 取代段最大的那一行 | `crates/singlefs-core/src/transaction.rs:3668`：「.max_by_key(|entry| entry.generation)」 |
| 记账树形由「上一版 key 集合与这一版 key 集合之差」定：先删上一版独有的 key，再插这一版独有的 key，在内存里算出新的 COW 形状；没有墓碑条目 | `crates/singlefs-core/src/code_two_tree.rs:655`：「for deleted in previous_keys.difference(keys_after_this_publish) {」与 `crates/singlefs-core/src/code_two_tree.rs:658`：「for inserted in keys_after_this_publish.difference(&previous_keys) {」 |
| 记账条目六个字段，34 字节，没有删除记号；`seq` 直落叶恒 1 | `crates/singlefs-core/src/records.rs:404`：「pub struct AccountingEntry {」与 `crates/singlefs-core/src/records.rs:34`：「pub const ACCOUNTING_SEQUENCE_DIRECT_TO_LEAF: u32 = 1;」 |
| 恢复判「台阶行之外的记账行恰为 3 + 6 × 盘数」，不等就拒 | `crates/singlefs-core/src/recovery.rs:5724`：「if entries_other_than_the_inode_number_watermark_steps != 3 + 6 * device_count {」 |
| 树表条目 flags 有未知位 ⇒ `parse` 交回 `None` ⇒ 整张树表读不下去 | `crates/singlefs-core/src/records.rs:753`：「_flags_this_reader_refuses => return None,」与 `crates/singlefs-core/src/recovery.rs:2804`：「TreeTableEntry::parse(entry_bytes).ok_or(TreeTableEntriesRefusal::EntryMalformed)?;」 |
| 管理员回退遇到树 ID 集合不同就拒（没有多头路径） | `crates/singlefs-core/src/mount.rs:8982`：「if target_version.tree_identifiers != current.tree_identifiers {」 |

零命中（实现里没有的）：

```
$ grep -rn -i 'write_buffer\|write buffer\|WriteBuffer' crates --include=*.rs | wc -l
0
$ grep -rnE 'fn [a-z_]*(destroy|delete|drop)[a-z_]*head|fn [a-z_]*clone[a-z_]*head|fn [a-z_]*head[a-z_]*(destroy|delete)' crates/singlefs-core/src --include=*.rs | wc -l
0
```

背景材料第二节说「记账树整棵重写」，更准的说法是上面那条：按 key 集合之差定形、内容没变的节点标 `CarriedFrom`（`crates/singlefs-core/src/code_two_tree.rs:636` 起的 `plan_the_tree_after_this_publish`）。因为每发布一次，这一代的 key（代段 = txg）全换，实际上装着这一代行的叶都要重写。

## W1　D8 已定项 8 ㊀（旧写者遇到不认识的树，记账行怎么处置）

### 1.1 读数（产物整行抄）

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186059`：「E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=Carry+NoCarry+KeepWithRoot:1600|NoCarry:1280|NoCarry+KeepWithRoot:1920」

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186060`：「E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_2 arm=all distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600」

同一份产物里读法 B2（㊀ 自己也算一条要改的条款，页上只当附带量）的两行：

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186067`：「E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b2_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600」

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186068`：「E7RESULT name=verdict check=q1_overall_winner_accounting_arms reading=b2_w2_q2_counted group=legal_r_tree_dim_2 arm=all distinct_winner_sets=2 judgement=depends_on_point winner_sets=Carry:3200|Carry+NoCarry+KeepWithRoot:1600」

读出来的事：`r_tree_dim = 1`、`h_new > 0` 的 3200 点上，B1 下 `Carry` 不是胜者，B2 下 `Carry` 在全部 3200 点是唯一胜者；`r_tree_dim = 2` 两种读法下都是 `Carry`。合法点上 `Carry` 与 `NoCarry` 的差只有字典序最后一位。

### 1.2 我自己重算的一格：最后一位的差到底是多少（独立于装置的算术）

脚本 `~/.local/share/singlefs/work/m3-close-c693-r1-sonnet/diff_rows.py`（sha256 `79b8d5e200c53bcb3775d7b39a487d3397abb756ebf617422fb1f29d4fecb5cb`）只读产物的 `point_verdict` 与 `cell` 行，对每个合法点算 `RejectEntry_Carry` 减 `RejectEntry_NoCarry` 的 `rows_written_w2`，按 (`r_tree_dim`, `h_new`, 差) 计数：

```
$ python3 -I ~/.local/share/singlefs/work/m3-close-c693-r1-sonnet/diff_rows.py research/results/e186-unknown-tree-accounting-rederive.out.gz
('1', '0', 0) 1600
('1', '2', 2) 1600
('1', '4', 4) 1600
('2', '0', 0) 1600
('2', '2', 4) 1600
('2', '4', 8) 1600
```

`r_tree_dim = 1` 时差恰为 `h_new`：搬运每个不认识的可写头每次发布多写 1 行（34 字节，`crates/singlefs-core/src/records.rs:34` 同一份字段表）。产物里每格每代写行的最大值：

```
$ grep ' name=cell ' research/results/e186-unknown-tree-accounting-rederive.out.gz | awk '{for(i=1;i<=NF;i++){if($i~/^rows_written_w2=/){split($i,a,"=");if(a[2]>m)m=a[2]}}} END{print "max rows_written_w2 =",m}'
max rows_written_w2 = 47
```

`crates/` 的记账叶容量：`crates/singlefs-core/src/unit.rs:155`：「pub fn index_node_entry_capacity(key_width: usize, entry_width: usize) -> usize {」，E97 装置的实现几何一行 `research/results/e97-entry-encoding-single-generation-2026-10-20.out:218` 里 `leaf_fanout=477`（key 22、条目 34、头 115）。47 远小于 477：E186 的 11520 个取样点上，搬运多写的那几行都落在一片本来就要重写的叶里，没有变成多写的节点。这一句是推的（行数对叶容量的算术，没在 `crates/` 上量；E186 页「它答不了的」一节也写了第 7 位数的是行不是节点）。

### 1.3 正推：㊀ 该怎么改

先看三条臂在实现上各是什么：

- `NoCarry` 在 `crates/` 今天是「构造出来的默认」：`accounting_entries_of_this_publish`（6931）只给它认识的那一个头（inode 树）写这一代的行，台阶行按统计量照抄（3685、3691，不看树）。所以一个只认识一个头的写者遇到别的头，别的头的「这一代的行」自然丢掉、台阶行自然带走。
- `Carry` 在 `crates/` 里没有：要新写「读上一版里不认识的树的行、换代段、再写一遍」的一步，且恢复的行数判（`crates/singlefs-core/src/recovery.rs:5724` 的「恰为 3 + 6 × 盘数」）要同时改成按头数算。多头路径整个还没写（零命中、`crates/singlefs-core/src/transaction.rs:3682` 自己写明）。
- 随根留着要给 D5 已定项 2 与 I-3.13 各开例外，E186 在 `lpc = 0` 的点上记 6 条要改的条款（明细行 `research/results/e186-unknown-tree-accounting-rederive.out.gz:141194`：「clause_units_b1=6」所在的 cell 行），在 `lpc = 1` 的点上与 `NoCarry` 并列。

所以问题只剩 `Carry` 与 `NoCarry`。我的判断是**㊀ 取搬运不翻面**，理由四条，前三条是 `crates/` 与产物给的，第四条是规则：

1. `NoCarry` 的安全建立在两个今天还没落地的前提上：每个不认识的头至少有一级台阶（E186 把 (`s`, `lpc`) = (0, 0) 判成不合法点，靠的是 D5 已定项 2 射程「建克隆那次给新头留一行」，而实现里没有克隆路径，`crates/singlefs-core/src/transaction.rs:3682`），以及守这一前提的 I-9.11 `crates/singlefs-checker/invariants/09-inode树结构.md:23`：「未实现，前置：树的种类」。前提破了，`NoCarry` 在 (0, 0) 点丢整个元组，E186 把这种丢失记成有 I-9.11 盖（`research/prompts/e186-preregistration.md:401`：「水位元组丢失有 I-9.11 盖（记 0）」），而 I-9.11 今天是未实现。`Carry` 不依赖这两样。
2. 带树维的统计量一多，`NoCarry` 当场丢：`r_tree_dim = 2` 的 3200 点上胜者是 `Carry`（186060），第二个统计量没有台阶行，不搬就丢元组，也没有不变量盖。D8 自己的射程已写（`.claude/kb/decisions/08-核心索引结构.md:282`：「再加一个带树维的统计量，不搬的丢失量就按树数长」）。统计量清单不封闭（`.claude/kb/decisions/05-快照-空间记账机制.md:140`：「清单不封闭：D12（目标介质） 的单盘线与 zoned 线还没开」），`r_tree_dim` 取 1 是今天的值，不是格式承诺。
3. 恢复的行数判（`crates/singlefs-core/src/recovery.rs:5724`）早晚要推广到多头。搬运下推广后的式子仍是等式（非台阶行 = 3 + 6 × 盘数 + 额外头的这一代行）；不搬下，旧写者发布出来的版本比新写者发布的少那几行，式子只能放宽成「不少于」，丢行就查不出来了。这一条是推的。
4. `.claude/rules/fs-design.md@不为省空间牺牲自包含`：「自描述数据宁可写大一点。」默认答案是不省，除非实测省下来的是瓶颈。搬运多写的是每头每次发布 1 行，1.2 节的算术说它没变成节点；没有量过的收益不拿来改一条用户 2026-09-06 定的条款（`.claude/kb/decisions/08-核心索引结构.md:288`：「用户定案 2026-09-06：四问」）。

关于读法：反向接受条款字面（`.claude/kb/experiments/186-不认识的树-记账维重推.md:16`：「按登记第十节反向接受条款的字面：Q1 合法点上的记账胜者不是 {Carry}」）在 B1 下触发，在 B2 下不触发。B1 与 B2 只差 `NoCarry` 那一格的 U12（改 ㊀ 自己）。这个分歧不是实验量出来的，是主 agent 在装置写之前定的读法，我不替它改读法；我只指出：翻面与否完全取决于第 7 位的 `h_new` 行（B1）和 ㊀ 自己算不算一条（B2），这两样都不是 `NoCarry` 的失败模式有多严重的读数。

### 1.4 按 `r_tree_dim` 分支行不行

按 `.claude/rules/fs-design.md@分支的三条约束` 与 `.claude/rules/fs-design.md@五条硬要求` 逐条过（`r_tree_dim` = 写者自己登记表里带树维的统计量个数，这是写者那一版二进制的常量）：

| 约束 | 能不能过 | 依据 |
|---|---|---|
| 只在操作期间不会变的输入上分支 | 过：写者二进制的统计量登记表在一次挂载里不变 | `.claude/kb/decisions/05-快照-空间记账机制.md:138`：「第 12 项只为可写头维护」登记表是格式常量 |
| 判据显式、可计算 | 过：「每个带树维的统计量都有台阶行」可以从登记表算出 | 同上 |
| 每条分支能被测试强制进入 | **不过**：`r_tree_dim = 2` 那一支今天没有真实输入（只有一个带树维的统计量），要加仅供测试的开关；`r_tree_dim = 1` 那一支又连旧写者路径都没有 | `crates/` 零命中（第 0 节） |
| 每条分支单独验、含崩溃点重放 | 不划算：两支都要重放，换的是每头每次发布 1 行 | 1.2 节的算术 |
| 可观测 | 要加一个「走了哪支」的读出 | `.claude/rules/fs-design.md@五条硬要求` 第 4 条 |

判定：**形式上分得开，不值得分。** 同一条规则（搬运）在两档上都不丢，分支换来的只是最后一位的 `h_new` 行；`.claude/rules/fs-design.md@五条硬要求`：「判据写不出来时，先只做一条，而且先做慢而正确的那条。」这里判据写得出来，但收益没量过。分支的债（第 3、4、5 条）要在一条今天没有实现的路径上还。

### 1.5 改后条文（逐句；原句行号现查）

**D8 已定项 8 ㊀**，`.claude/kb/decisions/08-核心索引结构.md:261`：「**㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行原样带到新代**（值不变、只换代号；」这一句的「只换代号」对台阶行不成立（台阶行的代段是它自己的台阶号，不换），改成：

> **㊀ 旧写者对不认识的记账行**（限「树 ID 不认识、统计量标签认识」这一半）：**写者必须把这类行带到新代**——这一代的行（代段是上一版 txg 的行）值不变、代段换成这次的 txg；inode 号水位的台阶行（D5 已定项 2 射程「inode 号水位的台阶行」）代段不换，按台阶行的照抄规则原样留着，该规则按统计量认、不看树 ID，写者不必认识那棵树；有前端的写者把这类行当前端条目写、随发布排空到叶，第一版写者没有前端（已定项 10）、直落叶，盘上布局相同、读不错。

**D8 已定项 8 射程「不搬的代价」那一句**，`.claude/kb/decisions/08-核心索引结构.md:282`：「不搬的代价：D5（快照 / 空间记账机制） 已定项 2 定每条根的记账树只装这条根那一代的行、每次发布全写一遍 ⇒ 写者不认识的那棵树的行没人重写，下一次发布就消失（K 代说法已撤回，这一句牵连的取舍要不要重推记在 C693（K 代撤回之后要重推的论证） ③）。」改成：

> 不搬的代价：D5（快照 / 空间记账机制） 已定项 2 定每条根的记账树只装这条根那一代的行、每次发布全写一遍 ⇒ 写者不认识的那棵树的「这一代的行」没人重写，下一次发布就消失；inode 号水位的值不因此丢，因为台阶行的照抄按统计量认、不看树，只要那个头至少有一级台阶（建克隆那次给新头留一行，D5 已定项 2 射程），现行值（代段最大的那一行）仍读得出。仍取搬运的理由有三：一是再加一个带树维的统计量时它没有台阶行，不搬就丢（E186 `r_tree_dim = 2` 的合法点上不搬在丢行那一关输给搬运）；二是「每个头至少有一级台阶」靠克隆路径与 I-9.11 守，两样今天都没实现，搬运不依赖它；三是搬运每个不认识的头每次发布多写 1 行（34 字节），E186 全部取样点每代至多 47 行、远小于叶容量 477，多写的行没有变成多写的节点（推的，没在 `crates/` 上量）。

**D8 已定项 8 依据**，`.claude/kb/decisions/08-核心索引结构.md:287`：「记账那一维取 `Carry` 不取 `Exempt`（当时 `Exempt` 要在 D5（快照 / 空间记账机制） 已定项 2 的 K 代上开例外，那笔代价一个格子都没占；2026-10-04 K 代点删撤回，`Exempt` 臂失去对象，见 C693（K 代撤回之后要重推的论证） ③）。」括号里改成「`Exempt` 建在已撤回的 K 代点删上，失去对象；记账那一维在每条根只装一代的口径下由 E186 重推，见下一条」，并在这一条后加一条：

> - E186（不认识的树·记账维重推）：每条根只装一代之后，三条记账臂在 11520 点 × 15 格上重比，字典序新补「要改几条已定条款」一位。读法 B1（㊀ 自己不计）下，`r_tree_dim = 1`、`h_new > 0` 的 3200 点上 `Carry` 不是胜者（`NoCarry` 或 `NoCarry` 与随根留着并列），分开它们的是最后一位「每代写行」，差恰为 `h_new` 行；`r_tree_dim = 2` 的 3200 点上胜者是 `Carry`；读法 B2（㊀ 自己算一条）下合法点 `h_new > 0` 的 6400 点 `Carry` 全是胜者。㊀ 不改取舍的理由见射程。（`research/results/e186-unknown-tree-accounting-rederive.out.gz` 第 186059、186060、186067、186068 行；最后一位的差用 `diff_rows.py` 另算过。）

**欠**，`.claude/kb/decisions/08-核心索引结构.md:296`：「C693（K 代撤回之后要重推的论证）、C342」，四块都判完之后把 C693 从这一行拿掉（本节只管 ③ 这一块）。

**D8 已定项 12 定案**，`.claude/kb/decisions/08-核心索引结构.md:383`：「含已定项 8 ㊀ 那类旧写者搬运的记账行」：取舍不变，这一句不动。

**D5 已定项 2 与 I-3.13**：不开新例外。`.claude/kb/decisions/05-快照-空间记账机制.md:49`：「**每条根的记账树只装这条根那一代的行，不留多代**」与 `crates/singlefs-checker/invariants/05-空间记账.md:20`：「inode 号水位那一统计量的台阶行不判」两处都按现文成立（被搬运的这一代的行代段 = 这次 txg，I-3.13 照判）。

**检查表**，`.claude/kb/checks-owed.md:132`：「记账轴还缺「豁免那棵树的 K 代点删」」那一句与随后的「⚠️ D5（快照 / 空间记账机制） 已定项 2 不再有 K 代点删，Exempt 臂失去对象，记账轴那一条随 C693（K 代撤回之后要重推的论证） ③ 重推。」改成「记账轴由 E186 重推，结论见 D8 已定项 8 依据；Exempt 臂失去对象」。C159 的其余部分（三条 flags 臂、天平）不动。

### 1.6 备选：主 agent 若坚持读法 B1 的字面、取 `NoCarry`

要动的条款与先决条件（我不推荐，列出来是为了让判决有账可对）：

1. D8 已定项 8 ㊀ 整句改成「写者不为不认识的树写这一代的行；水位值靠台阶行的照抄规则（按统计量认、不看树）带走」，射程「不搬的代价」改成「不搬靠台阶行，前提是每个头至少有一级台阶」，依据里 E113 一条的 `Carry` 改成 `NoCarry`，D8 已定项 12 定案（383）里「含已定项 8 ㊀ 那类旧写者搬运的记账行」去掉。
2. 先决：实现 I-9.11（`crates/singlefs-checker/invariants/09-inode树结构.md:23`）并保证建克隆那次留一级台阶，否则 (0, 0) 点丢整个水位元组；`crates/singlefs-core/src/recovery.rs:5724` 的行数判推广到多头时要放宽成「不少于」；统计量登记表加「带树维的统计量必须留台阶行」一条约束，否则 `r_tree_dim = 2` 时丢行。
3. 这四样合起来，就是 `Carry` 用 1 行/头/发布换掉的东西。

### 1.7 什么现象会推翻 W1 的结论

- 在入库装置或 `crates/` 上量出搬运这 `h_new` 行造成了多写的节点（记账树分裂成多片叶、搬运的行落进本来不会被重写的叶）：那么第 7 位的差就不是 0 个节点，要重判。
- 主 agent 或用户定「两个带树维的统计量在可见的将来不会出现」并写进 D5 已定项 4：`r_tree_dim = 2` 那一支失去支撑，理由 2 消失，仍剩理由 1、3、4。
- 建克隆那一步被实现并带了 I-9.11 的机检：理由 1 消失。
- 读法 B1 被主 agent 认定是对的、且接受 `NoCarry` 的 1.6 节四样先决：翻面。

## W2　D8 已定项 8 ㊁（未知 flags 位）

### 2.1 读数（产物整行抄）

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186275`：「E7RESULT name=verdict check=q2_overall_winner_flags_arms reading=b1_w2_q2_counted group=legal_r_tree_dim_1 arm=all distinct_winner_sets=3 judgement=depends_on_point winner_sets=RejectEntry:3600|Skip:600|Skip+PerBit:600」

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186291`：「E7RESULT name=verdict check=q2_reject_entry_versus_skip group=legal_r_tree_dim_1 points=4800 separable_points_q2_counted=1200 separable_points_q2_not_counted=0 flags_winners_changed_points_b1_w2=1200 flags_winners_changed_points_b2_w2=1200 judgement_b1_w2=changes_tier」

`research/results/e186-unknown-tree-accounting-rederive.out.gz:186044`：「E7RESULT name=verdict check=self_proof_q2_reject_entry_versus_skip_position_six_where_w_gens_0 checked=8640 not_turning_red=0 turns_red=true」

读出来的事：读「计」时胜者在 `w_gens > 0` 的 3600 点是 `RejectEntry`，在 `w_gens = 0` 的 1200 点（每组）换成 `Skip`；读「不计」时 `w_gens = 0` 两条并列。换档只发生在 `w_gens = 0`。

### 2.2 `w_gens` 是什么、能不能当分支变量

`w_gens` 是「隐形窗口」的代数：`.claude/kb/experiments/113-不认识的树-补齐臂集.md:39`：「每代出生 `BLOCKS_BORN_PER_GENERATION` 块、隐形窗口 `W_GENS` 代 ⇒ 误释放数 = `BLOCKS_BORN_PER_GENERATION × W_GENS`，闭式。」`Skip` 的伤害是 64 × `w_gens` 块被误释放，`w_gens = 0` 时没有受害块，所以 `Skip` 与 `RejectEntry` 在前五位全平，只剩「`RejectEntry` 要定义一条新语义」这一位分开它们。

`w_gens` 是隐形窗口的发布代数；窗口从哪一代开始、到哪一代结束，取决于之后谁在什么时候建和销毁快照（我对上面那句闭式的读法，推的）：写者挂载时窗口还没发生，窗口里的发布数随之后每次发布增长。按 `.claude/rules/fs-design.md@分支的三条约束` 第 1 条「只在操作期间不会变的输入上分支」，它是危险的分支变量，挂载时不可知。所以 ㊁ 只能选一档，选在 `w_gens > 0` 上不丢数据的那一档：`w_gens > 0` 的 3600 点两种读法胜者都是 `RejectEntry`，`Skip` 在第一位输（误释放，丢数据）。`w_gens = 0` 的 1200 点上 `Skip` 赢的只是那一个「+1 语义」的位，而那条语义（「EIO 但位置还在」）已经在 D8 已定项 8 射程里定义过一次：`.claude/kb/decisions/08-核心索引结构.md:282`：「条目 EIO 但位置还在」，它不是一笔还没付的新定义。

判定：**㊁ 条款不改。** E186 的 Q2 读数说明「计」时 `RejectEntry` 在 `w_gens = 0` 的点上被 `Skip` 压一位，这个读数对「`RejectEntry` 对 `Skip` 的判别力自证」是有效的（8640 格由绿转红），但它不是改 ㊁ 的理由：对一个挂载时不知道自己会落在 `w_gens = 0` 还是 `> 0` 的政策，取最坏情形下仍不丢数据的一档。

### 2.3 ㊁ 与实现的出入：谁改

事实（命令与行号都是现查的）：

- ㊁ 的字面：`.claude/kb/decisions/08-核心索引结构.md:262`：「未知位一律拒收——该条目 EIO（对象级，不拒整个池）」。
- 实现：`TreeTableEntry::parse_with_destruction_mark`（`crates/singlefs-core/src/records.rs:734`：「pub fn parse_with_destruction_mark(bytes: &[u8]) -> Option<(Self, SnapshotDestructionMark)> {」）对未知 flags 位 `return None`（`crates/singlefs-core/src/records.rs:753`），读树表的唯一入口把 `None` 当成整张树表读不下去（`crates/singlefs-core/src/recovery.rs:2804`），再变成 `RecoveryFailure::UnitMalformed`（`crates/singlefs-core/src/recovery.rs:2687`：「TreeTableEntriesRefusal::EntryMalformed => RecoveryFailure::UnitMalformed {」）。`parse` 的注释写着「按 ㊁ 一律拒收」，字面与 ㊁ 的「拒收」一致，差在调用方的反应是整表。
- 实现里没有任何东西消费「位置还在」：deadlist 合并没实现（D5 已定项 13、16 的进度都是未实现：`.claude/kb/decisions/05-快照-空间记账机制.md:23`：「进度：未实现。」与 `.claude/kb/decisions/05-快照-空间记账机制.md:26`：「进度：未实现。」），所以整表拒绝在今天是 ㊁ 的更严近似，不会误释放；代价是可用性（一条带未知位的条目让整个池挂不上）和一处我没核的行为：整表读不下去会让恢复把这一根判成不可用，是否由此退到更旧的根，我没读恢复的选根逻辑，不下结论。
- `parse` 把「未知 flags 位」与「条目长度不对、预留非零、根指针头损坏」混成同一个 `None`（`crates/singlefs-core/src/recovery.rs:2670` 的注释把几种原因列在同一个枚举项里）。

判定：**改实现，不改 ㊁。** ㊁ 是用户 2026-09-06 的定案（`.claude/kb/decisions/08-核心索引结构.md:288`：「用户定案 2026-09-06：四问」），整表拒绝违反它「不拒整个池」的括注，也违反 ④ 的目的（旧读者跳过不认识的树）。但这笔实现没有消费者，现在改是写没人读的分支：改动放在做 deadlist 合并（D5 已定项 16）的那一步一起做。

### 2.4 改后条文与要动的条款

不动 ㊁ 的定案句。在 D8 已定项 8 射程「㊁ 拒收不丢数据依赖 ④ 的条目自带长度」那一句之后（`.claude/kb/decisions/08-核心索引结构.md:282` 同一行）加一句进度注：

> ⚠️ **实现今天比 ㊁ 更严**：`crates/singlefs-core/src/recovery.rs` 的 `tree_table_entries_each_kind_at_most_once` 遇到 flags 未知位（`TreeTableEntry::parse` 交回 `None`）按 `EntryMalformed` 让整张树表读不下去，不是「该条目 EIO、位置还在」。deadlist 合并（D5 已定项 16）还没实现，没有消费「位置还在」的读者，整表拒绝是更严的近似、不会误释放；做 deadlist 合并的那一步把 `TreeTableEntry::parse` 拆成「合法」「长度、预留、根指针合法而 flags 有未知位」「损坏」三态，树表读取只对第三态拒整表，对第二态把条目标成 EIO 并保留位置。

D8 总表里已定项 8 一行（`.claude/kb/decisions/08-核心索引结构.md:20`）的「**进度：已实现。**」要不要改成「部分实现」交主 agent：该行写的是「未知 flags 位拒收该条目」，而实现是整表拒。㊀ 那一半实现里一行也没有（命令 `grep -rn -E '旧写者|不认识的树|unknown_tree|carry_unknown' crates --include=*.rs | grep -v '/tests/' | wc -l` 输出 `0`），这一行的进度本来就没有分开写 ㊀ ㊁。

要动的条款：D8 已定项 8 射程一句进度注；D8 总表第 8 行进度（交主 agent）；C158（checker 跳过不认识的树会判假阳性，`.claude/kb/checks-owed.md:131`）不动。`crates/` 的改动写进实现排期：`crates/singlefs-core/src/records.rs:734` 的返回类型拆三态，`crates/singlefs-core/src/recovery.rs:2797` 起的 `tree_table_entries_each_kind_at_most_once` 与五个读树表的调用点（函数注释里列的 `rebuild_version`、`allocation_records_under_root`、`walk_to_file`、`user_visible_tree_root_pointers`、`mounted_read::open_pool_for_read`）按三态处理。

### 2.5 什么现象会推翻 W2 的结论

- 有人量出（或定出）`w_gens` 在挂载时可知（例如旧写者挂载时就能判断池里没有隐藏快照可销毁、且之后不会有）：那么 `w_gens = 0` 可以成为一个稳定的分支输入，`Skip` 或 `ReadOnly` 在那一档的取舍要重判。
- 整表拒绝被证明在选根时会退到更旧的根并静默丢掉新写者的发布：那么实现的出入不是「可用性」问题，而是数据问题，改实现要提前到现在（我没读选根逻辑，核不动）。
- 用户对 ㊁ 的括注「（对象级，不拒整个池）」改主意：那是用户的决定，改 ㊁ 而不改实现。

## W3　D5 已定项 4 的依据与「树高恒为 2」

### 3.1 读数（产物整行抄；E71 是 2026-09-01 那张九项表的新口径，E97 是 2026-09-03 那张十一项表的新口径，两份页上都写明了）

`research/results/e71-accounting-keys-single-generation-2026-10-20.out:288`：「E7RESULT name=judgement_cell arm=upper_bound width=middle trees=1024 devices=64 old_bytes=35389440 old_over=1 new_bytes=17694720 new_over=0 budget=21474836」

`research/results/e71-accounting-keys-single-generation-2026-10-20.out:91`：「E7RESULT name=publish_budget width=middle width_role=registered_width key_bytes=22 value_bytes=8 sequence_bytes=0 entry_bytes=30 arm=per_statistic_wide arm_role=incidental_wide_reading trees=1024 devices=64 entries_per_generation=267264 old_bytes=16035840 old_over=0 new_bytes=8017920 new_over=0 new_permille_of_budget=373 budget=21474836」

`research/results/e71-accounting-keys-single-generation-2026-10-20.out:289`：「E7RESULT name=over_budget_count arm=upper_bound widths=registered_widths cells=45 old_over=9 new_over=5 new_over_cells=narrow:8192:64,middle:8192:64,wide:1024:64,wide:8192:8,wide:8192:64」

`research/results/e71-accounting-keys-single-generation-2026-10-20.out:291`：「E7RESULT name=over_budget_count arm=per_statistic_narrow widths=registered_widths cells=45 old_over=0 new_over=0 new_over_cells=none」

`research/results/e71-accounting-keys-single-generation-2026-10-20.out:293`：「E7RESULT name=over_budget_count arm=per_statistic_wide widths=registered_widths cells=45 old_over=5 new_over=3 new_over_cells=narrow:8192:64,middle:8192:64,wide:8192:64」

`research/results/e97-entry-encoding-single-generation-2026-10-20.out:226`：「E7RESULT name=judged_cell question=Q-E97-1 arm=acct_wide_19B trees=1024 devs=64 reading=wide old_node_bytes=227901440 old_over=1 new_node_bytes=9076736 new_over=0 new_percent=42 new_permille=422」

`research/results/e97-entry-encoding-single-generation-2026-10-20.out:228`：「E7RESULT name=wide_over_cells question=Q-E97-3 arm=acct_wide_19B old_over_cells=3 old_over_grids=1024x64,8192x8,8192x64 new_over_cells=1 new_over_grids=8192x64」

`research/results/e97-entry-encoding-single-generation-2026-10-20.out:229`：「E7RESULT name=narrow_tightest question=Q-E97-4 arm=acct_wide_19B old_node_bytes=11419648 old_grids=8192x64 old_percent=53 old_permille=531 old_over=0 new_node_bytes=458752 new_grids=8192x8,8192x64 new_percent=2 new_permille=21 new_over=0」

`research/results/e97-entry-encoding-single-generation-2026-10-20.out:113`：「E7RESULT name=acct_key_ceiling_new_caliber trees=8192 devs=64 reading=wide touched_entries=2670592 budget=21474836 max_entry_bytes=8 max_key_bytes=0 feasible=0」

算术核（上界、宽读对窄读的倍数，E71 页 t=64 / d=8 / K=25 一格的 115 200、59 200、4 075）：

```
$ python3 -c "print(59200/4075, 115200/4075)"
14.52760736196319 28.26993865030675
```

### 3.2 D5 已定项 4 依据两句逐句对数

`.claude/kb/decisions/05-快照-空间记账机制.md:145`：「E71（记账 key 的条目数与宽度）：按分项计维与维度用满两条臂在同一格差 14.5 倍」——对不上：「按分项计维」是窄读，「维度用满」是上界，两者差 28 倍（`.claude/kb/experiments/71-记账key的条目数与宽度.md:63`：「上界比窄读大 28 倍」）；14.5 倍是宽读对窄读（同一页 `.claude/kb/experiments/71-记账key的条目数与宽度.md:224`：「所以窄读与宽读之间差 14.5 倍这件事」）。出处错一。

同一行：「`树数 × 设备数` 到 `1024 × 64` 时宽读那条臂过 `T_dirty` 的 1%」——在 E71 名下本来就不成立：旧口径 30 字节宽读 `1024 × 64` 是 16 035 840（上面 `out:91` 的 `old_over=0`），只有 50 字节那一档旧口径过界（`out:136` 的 `old_over=1`）；过界的是 E97 判据 10 的节点口径（旧口径 227 901 440，上面 `e97 out:226`）。出处错二（把 E97 的数记在 E71 名下）。一代口径下两边都不过界：E71 8 017 920（373‰），E97 9 076 736（42%）。FC1 在两份页上都触发（`.claude/kb/experiments/71-记账key的条目数与宽度.md:139` 与 `.claude/kb/experiments/97-记账与分配记录的条目编码.md:323`）。

同一行：「窄读四格全过」——成立：E71 窄读 45 格 0 过界（`out:291`），E97 窄读四格 0 过界。

`.claude/kb/decisions/05-快照-空间记账机制.md:146`：「E97（记账与分配记录的条目编码）：宽读法下最紧的一格占预算 53%」——53% 是窄读 `8192 × 64` 的旧口径（`.claude/kb/experiments/97-记账与分配记录的条目编码.md:153`：「| t=8192 d=64 | 窄 | 16 708 | 902 232 | 11 419 648（占预算 **53%**）| 否 |」），宽读与窄读写反。出处错三。一代口径下窄读最紧一格是 2%（`e97 out:229`），宽读只剩 `8192 × 64` 一格过界（`e97 out:228`）且该格没有可行的条目宽（`e97 out:113` 的 `feasible=0`）。

### 3.3 「哪几项按树分」的取舍还站不站

站得住，但它站的位置要换。三条推理：

1. **采用的读法比 E71 / E97 的「窄读」还窄。** E71 的窄读是 `.claude/kb/experiments/71-记账key的条目数与宽度.md:50` 的「只有第 8（每树独占）、9（每树共享）两项」，这两项已撤回（`.claude/kb/decisions/05-快照-空间记账机制.md:127`：「已撤回·此编号不再使用」）；今天只有第 12 项带树维、只为可写头维护（`.claude/kb/decisions/05-快照-空间记账机制.md:138`：「第 12 项只为可写头维护」，实现里 `crates/singlefs-core/src/transaction.rs:6882` 同样只有水位一行记在树上）。所以窄读的结果（E71 0/45 过界、E97 四格全过且最紧 2%）是采用读法的上界，付得起。
2. **宽读没有被一代口径救成「该选」。** 过界的格从 E97 的 3 个降到 1 个、E71 宽读从 5 降到 3，剩下的是 `8192 × 64` 一格（所有宽度、E97 反解无可行条目宽）。但这一格是扫出来的取样点，不是负载（`.claude/kb/experiments/71-记账key的条目数与宽度.md:222`：「t=8192 那几档是扫出来的，不是负载」），代价侧依据从「宽读在三格上撑不住」弱化成「宽读在最大那一格撑不住」。
3. **取舍真正的依据是「谁要它」。** D5 已定项 4 自己的完备性判据：`.claude/kb/decisions/05-快照-空间记账机制.md:91`：「没出现在任何一条判定里的就不要」。没有读者要每树的空闲字节、每树的碎片度，宽读白付一份条目；这一条和 E71 / E97 的数无关。

判定：**取舍不改（仍只有第 12 项带树维），依据句重写。**

### 3.4 改后条文（逐句）

**D5 已定项 4 依据，`.claude/kb/decisions/05-快照-空间记账机制.md:145`**，整条改成：

> - E71（记账 key 的条目数与宽度）：按 2026-09-01 那张九项表、一代口径（判据 2 第 1 次重跑，每条根只装一代、每次发布写一次）：上界（维度用满）与窄读（两项按树）在 t=64 / d=8 一格差 28.3 倍，宽读与窄读差 14.5 倍；`树数 × 设备数` 先撑爆预算，30 字节、`1024 × 64` 上界臂 17 694 720 字节（门槛 21 474 836 的 823‰）不过界，宽读臂 8 017 920 字节（373‰）不过界，过界的格集中在 `树数 × 设备数` 大的一端（原三档宽度 45 格里上界 5 格、宽读 3 格全在 `8192 × 64`、窄读 0 格）；`统计量数 × 树数 × 设备数` 是一代的上界不是条目数（inode 号水位的台阶行除外：它随根留着，条目数随水位变过的发布数增长，不受这个上界管）。

**同一节，`.claude/kb/decisions/05-快照-空间记账机制.md:146`**，整条改成：

> - E97（记账与分配记录的条目编码）：按 2026-09-03 那张十一项表、一代口径（判据 10 第五次重跑，脏节点计）：窄读四格全不过界，最紧一格占预算 2%（`8192 × 8` 与 `8192 × 64` 并列）；宽读 `1024 × 64` 占 42%、`8192 × 8` 占 47% 不过界，`8192 × 64` 占 337% 过界且该格反解无可行条目宽。这是「哪几项按树分」那一列的代价侧依据之一：窄读和实际采用的读法（只有第 12 项带树维、只为可写头维护）都付得起，宽读只在最大那一格付不起；那一列的取舍本身由「谁要它」定（上面「谁要它」那一列与本项的完备性判据）。

**D5 已定项 1 依据，`.claude/kb/decisions/05-快照-空间记账机制.md:42`**：「E97（记账与分配记录的条目编码）：四档 key 宽的树高恒为 2，多带一维的段宽不构成代价。」改成：

> - E97（记账与分配记录的条目编码）：四档 key 宽下记账树的树高彼此相同（一代口径、t=64 / d=8 一格：窄读 1、宽读 2），多带一维的段宽不动树高，不构成代价。

`.claude/kb/decisions/05-快照-空间记账机制.md:41` 的 E71 一条（「按分项计维算出的条目数比」…「先撑爆预算的是 `树数 × 设备数` 这个乘积」）一代口径下仍成立（上界臂 5 格过界都在乘积大的一端），不动。

**D5 已定项 5**，`.claude/kb/decisions/05-快照-空间记账机制.md:159` 表里「而四档 key 宽的树高恒为 2」改成「而四档 key 宽下树高彼此相同（段宽不动树高）」；`.claude/kb/decisions/05-快照-空间记账机制.md:169`：「E97（记账与分配记录的条目编码）：四档 key（7 / 14 / 19 / 22 字节）的树高恒为 2，代价曲线是平的」改成「……的树高彼此相同（一代口径下窄读 1、宽读 2），代价曲线是平的」，分号后面（代段回绕）不动。

「恒为 2」不是实现的前提：树高从根节点头里的层级现读（`crates/singlefs-core/src/code_two_tree.rs:212`：「树高 = 根的层级 + 1（D8（核心索引结构） 已定项 11 ⑤）；还没建过的树是 0。」）；两块盘的池只有常态 15 行加台阶行（`.claude/kb/decisions/08-核心索引结构.md:32`：「两块盘的池上除台阶行外常态 15 行」），少于叶容量 477，树高是 1，D8 已定项 1 射程说 80 块盘起才分裂（`.claude/kb/decisions/08-核心索引结构.md:34`）。三处 kb 句子（`05-...:42`、`:159`、`:169`）以外，kb、不变量、`crates/` 里没有第四处写「树高恒为 2」（`grep -rn '树高恒' .claude/kb/decisions/*.md .claude/kb/checks-owed.md crates/singlefs-checker/invariants/*.md` 命中这三行加一行无关的 D23 表格行）。

### 3.5 什么现象会推翻 W3 的结论

- 找到一个需要每树空闲字节或每树碎片度的读者（准入、整理、df）：那一项就该带树维，「谁要它」的依据反过来，取舍要重开。
- 入库装置按今天的十二项表（带树维只有第 12 项）重算，结果与「窄读是上界」的推断不符（例如第 12 项带树维引起的行数比两项窄读还多）：1 的论证要重做。E71 / E97 都没按十二项表算（E71 页「它答不了的」一节与 E97 页末），我是靠「第 12 项的行数 ≤ 窄读的两项」这个包含关系推的，没量。
- 实现几何下（`leaf_fanout=477`，`e97 out:218`）宽读的两格落到门槛附近：E97 页 `.claude/kb/experiments/97-记账与分配记录的条目编码.md:373` 记了 S2 上宽读判决两格占 53% 和 60%，离门槛比 19B 臂近；宽读本来就不采用，但若有人要用宽读，这两格要按实现几何重算。

## W4　① 块：点删与墓碑那一族

### 4.1 摸底报告两个待答问题，在 `crates/` 今天的实现上答

**问题一：开 write buffer 前端之后，整批换代是重建整棵记账树，还是写新一代、点删上一代？**

- 今天的实现没有前端，也没有墓碑：`crates/` 里 `write_buffer` 零命中（第 0 节）；记账条目只有六个字段、34 字节、没有删除记号（`crates/singlefs-core/src/records.rs:404`：「pub struct AccountingEntry {」）；第一版记账直落叶是已定条款（`.claude/kb/decisions/08-核心索引结构.md:337`：「**第一版记账直落记账树叶，不走 write buffer 前端**」）。
- 上一代的行怎么离开：行集每次发布整组重算，树形由「上一版 key 集合与这一版 key 集合之差」定（`crates/singlefs-core/src/code_two_tree.rs:655`：「for deleted in previous_keys.difference(keys_after_this_publish) {」与 `crates/singlefs-core/src/code_two_tree.rs:658`：「for inserted in keys_after_this_publish.difference(&previous_keys) {」），删的是内存规划里的 key，新树按 COW 写出，记账树的盘上形状里没有任何东西叫墓碑（`crates/singlefs-core/src/tombstone_tree.rs` 的墓碑树是另一棵树，记的是对象墓碑，不是记账行的删除）。所以答案是**前者（重建）**：用摸底报告的话，「两处失去对象」。
- 前端开了以后会怎样，`crates/` 答不了，条款只给了方向：`.claude/kb/decisions/08-核心索引结构.md:383`：「**第一版两级都不开**」，记账是走前端的三棵树之一。前端条目是变更，「上一代的行离开」在前端里最直接的写法是点删，墓碑会回来；这一步没有任何实现可看，下面 4.2 用结构说明它回来也碰不到取最大。

**问题二：删一个可写头或克隆头时，它的台阶行怎么离开记账树？**

- 今天没有这条路径：`crates/singlefs-core/src` 里名字带 destroy / delete / clone 与 head 的函数零命中（第 0 节第二条命令）；实现自己写明克隆路径还没有、台阶行不按树段分（`crates/singlefs-core/src/transaction.rs:3682`：「今天没有克隆路径」）；管理员回退遇到树 ID 集合不同就拒（`crates/singlefs-core/src/mount.rs:8982`）。
- 台阶行今天一直留着：下一次发布把它们逐行照抄（`crates/singlefs-core/src/transaction.rs:6968`），没有任何一处按树过滤。所以今天它们根本不离开。
- 做销毁头那一步时，实现能表达的唯一写法是「不带过去」：行集是整组重算的，台阶行从上一版取（3685）；给取台阶行这一步按树分开、只取还活着的头的台阶，其余的头的台阶就不在新行集里，树形之差把它们删掉。这与 `crates/singlefs-core/src/transaction.rs:3682` 那条 TODO 是同一件事（按树段分开取）。不需要墓碑，也没有「增量点删」这一步可写。

### 4.2 为什么墓碑回来了也碰不到「取最大」：结构上的理由

- 记账 key 的代段 = 写下它的那次发布的 txg，I-3.13 逐行判（`crates/singlefs-checker/invariants/05-空间记账.md:20`：「每一行的代段（记账 key 偏移 14，8 字节）等于这一版的 `checkpoint_txg`」），台阶行是唯一例外，且台阶行是写下去就不再变的（值与代段都原样照抄）。所以一个记账 key 在它自己那次发布里得到值，之后只有「被带过去」（台阶行）或「被删」两种命运；删它发生在后面的发布里，那时没有新值与它同 key。第一版直落叶，`seq` 恒 1；开前端以后同一 key 在窗口里可以有几个值，由 `seq` 定后者（D5 已定项 3 第 3 行前半句，取最大值的统计量取最大），但墓碑不与它们同一次发布同一个 key。
- 同一次发布里写了又删（一个头建出来又在同一个 checkpoint 里销毁）的情形属于 D8 已定项 13 的折叠：`.claude/kb/decisions/08-核心索引结构.md:397`：「E 是墓碑时，折叠的结果就是墓碑，不许把墓碑当数值与 A–D 一起取最大」，该条射程是「全部索引树的全部落叶路径」，记账树在内；这一句已经覆盖了记账行，D5 已定项 3 第 3 行的那半句是它在单调统计量上的重复。
- 没有读者要上一代的账：`.claude/kb/decisions/05-快照-空间记账机制.md:60`：「没有一个读者同时要多代的账」。被点删的行没有读者，「读一个 key 先看最新一条是不是墓碑」在记账树上没有读它的人。
- C344 自己写明它只在前端让同一个 key 多条并存时才发生：`.claude/kb/checks-owed.md:252`：「撞不上——它在 write buffer 前端或节点缓冲让同一个 key 多条并存时才发生」。K 代点删时每次发布点删第「当前代 − K」代的 key，那时同 key 的叶值与前端墓碑并存；一代口径下被删的是上一代的 key，当前代的 key 没有第二个值，这一类并存不再出现，剩下的只有「写了又删」的折叠，归 D8 已定项 13。

### 4.3 判定与改后条文

判定：**D5 已定项 3 第 3 行的墓碑半句与 C344 在记账树上没有对象：销账（并入 C383），不缩题。** 缩题要有一个可以写出来的检查，而 `crates/` 里没有前端、没有墓碑条目、没有销毁头的路径，检查没有可强制进入的分支（`.claude/rules/fs-design.md@五条硬要求` 第 2 条）。留下来的东西是「墓碑折叠」本身，它已经在 D8 已定项 13 与 C383。

**D5 已定项 3 第 3 行**，`.claude/kb/decisions/05-快照-空间记账机制.md:75`：「点删的墓碑不参加取最大——读一个 key 先看最新一条是不是墓碑，是就判死，否则只在存活的行之间取最大；把墓碑当数值折进 max 会让被点删的那一代永远留下」，把这一句换成：

> 记账行没有点删：每条根的记账树只装这条根那一代的行（已定项 2），上一代的行靠整组行重算后的树形之差离开（`crates/singlefs-core/src/code_two_tree.rs` 的 `plan_the_tree_after_this_publish`），条目里没有墓碑记号（`AccountingEntry` 六个字段、34 字节）；一个 key 只在写下它的那次发布里有值（代段是那次的 txg），同一次发布里写了又删的情形归 D8 已定项 13 的折叠规则，不在这里另立。

**D5 已定项 3 欠**，`.claude/kb/decisions/05-快照-空间记账机制.md:87`：「**欠**：C344（单调统计量的点删遇墓碑没有会红的检查）。」改成「**欠**：无。」

**D8 已定项 12**：`.claude/kb/decisions/08-核心索引结构.md:387`：「单调统计量的点删遇墓碑（D5（快照 / 空间记账机制） 已定项 3 第 2 条）；」整句从「**连带**」那串里删掉；`.claude/kb/decisions/08-核心索引结构.md:393`：「C344（单调统计量的点删遇墓碑没有会红的检查）、C345」删掉 C344 一项；`.claude/kb/decisions/08-核心索引结构.md:391`：「K 代点删」后补「（K 代 2026-09-23 随 D5 已定项 2 撤回）」。

**D8 已定项 13**，`.claude/kb/decisions/08-核心索引结构.md:397`：「（C344（单调统计量的点删遇墓碑没有会红的检查） 实测这种朴素折叠四条候选 16 / 16 丢点删，`b.out`）」改成「（`b.out` 实测这种朴素折叠四条候选 16 / 16 丢点删；那批候选的点删对象是 K 代时留下的墓碑，记账树里现在没有墓碑，这条规矩对别的树不变，检查归 C383）」。

**D8 已定项 7**，`.claude/kb/decisions/08-核心索引结构.md:248`：「点删当场不成立；」前补括注「（D5 已定项 2 已改成每条根只装这一代、不再点删，这一条随之没有对象；`seq` 住 value 由其余三条撑）」（摸底报告 A9 的新句，我对着现文核过位置，内容不动）。

**检查表**：`.claude/kb/checks-owed.md:252` C344 一行搬进已还清表，写法照已还清表里 C341 那几行；还清说明写「K 代撤回后记账树没有点删，记账条目没有删除记号，`crates/` 没有 write buffer 前端；墓碑折叠的检查并入 C383」。`.claude/kb/checks-owed.md:277`（C383 行）「墓碑那一半与 C344（单调统计量的点删遇墓碑没有会红的检查） 同一个判据。」改成「墓碑那一半的判据是 D8 已定项 13「E 是墓碑时，折叠的结果就是墓碑」；曾由 C344 单列，并入这里。」C383 的取样点要含「一个头在同一次发布里建出来又销毁」一格。`.claude/kb/checks-owed.md:314` 的三条例子拿掉 C344。`.claude/kb/checks-owed.md:452` 的 C693 ① 块在判决后改成「已判」。

### 4.4 摸底报告 ① 一节 A1–A14 的去向

| 编号 | 去向 |
|---|---|
| A1、A2 | D5 已定项 3 第 3 行换句（上） |
| A3 | D5 已定项 3 欠改「无」 |
| A4、A5、A6 | C344 搬进已还清表 |
| A7、A8 | 随 C344 改措辞（上） |
| A9 | D8 已定项 7 补括注（上） |
| A10 | D8 已定项 12 连带串删一项（上） |
| A11 | D8 已定项 12 依据补「K 代已撤回」（上） |
| A12 | D8 已定项 12 欠删 C344（上） |
| A13 | D8 已定项 13 括注换句（上） |
| A14 | `.claude/kb/milestone/08-eighth-txn.md:34` 的欠账串删 C344（该文件我没读，列出来交 kb-scribe 现查） |

### 4.5 什么现象会推翻 W4 的结论

- 记账前端的设计把「上一代的行离开」定成前端点删，且有读者在窗口中途读被点删的那些行（例如销毁头进行到一半时有人读它的水位）：这时 C344 的对象回来，不能销账。我没有看到这样的读者（`.claude/kb/decisions/05-快照-空间记账机制.md:60`），也没有这样的设计；但前端是未实现项，这是设计空白而不是实现事实。
- 在记账树里找到一类 key，代段不等于写下它那次发布的 txg 且值会被改（台阶行以外）：4.2 第一条的结构理由破了。
- 摸底报告外还有一个以「记账行点删」为前提的句子（`grep -rn '点删' .claude/kb crates/singlefs-checker/invariants`）没进 A1–A14：销账的范围要扩。

## 判定一览（完整版）

| 格 | 判定 | 一句话 | 推翻它的现象 |
|---|---|---|---|
| W1 ㊀ | 冲突（部分）：字面反向接受条款在 B1 下触发；正推结论不翻面 | 保留搬运（`Carry`）；㊀ 的「只换代号」对台阶行不成立、射程「不搬的代价」在一代口径下说得过头，两处改字；不按 `r_tree_dim` 分支（形式上分得开，不值得分）；D5 已定项 2 与 I-3.13 不开新例外 | 量出搬运那 `h_new` 行多写了节点；定下不会再有带树维的统计量；克隆路径与 I-9.11 落地；认定 B1 并接受 1.6 节四样先决 |
| W2 ㊁ | 一致（条款）；冲突（实现） | ㊁ 不改：`w_gens` 挂载时不可知、不能当分支变量，取 `w_gens > 0` 不丢数据的一档；`crates/singlefs-core/src/recovery.rs:2804` 整表拒是实现比 ㊁ 更严，改实现（随 deadlist 合并），条款加进度注 | `w_gens` 能在挂载时判定；整表拒绝导致恢复静默退到旧根；用户改 ㊁ 的括注 |
| W3 D5 已定项 4 依据与树高 | 冲突（依据句）；一致（取舍） | 取舍仍只有第 12 项带树维，站在「谁要它」和「窄读是采用读法的上界」上；D5 的 145、146 两句改数并更正三处出处错；「树高恒为 2」三处改成「段宽不动树高」 | 找到需要每树空闲字节或碎片度的读者；按十二项表重算推翻包含关系；实现几何下宽读要被采用 |
| W4 点删与墓碑 | 冲突：记账树上没有对象，销账 | 问题一答重建（按 key 集合之差定形、条目无删除记号、前端零命中）；问题二答不带过去（实现今天没有销毁头路径，是同一条 TODO）；D5 已定项 3 第 3 行墓碑半句换句、C344 并入 C383 | 前端设计把上一代离开定成点删且有读者在窗口里读它；找到代段不等于写下它那次 txg 且值会改的记账 key；摸底范围外还有以记账点删为前提的句子 |

## 要动的条款汇总（供主 agent 对着判决用，改写由 kb-scribe 做，我没改任何 kb）

| 文件 | 行（开工后现查） | 动什么 | 来源格 |
|---|---|---|---|
| `.claude/kb/decisions/08-核心索引结构.md` | 261 | ㊀ 改字（台阶行不换代段） | W1 |
| 同上 | 282 | 「不搬的代价」一句重写；加 ㊁ 进度注 | W1、W2 |
| 同上 | 287 | `Exempt` 括号改写，加 E186 一条 | W1 |
| 同上 | 20 | 总表第 8 行进度要不要拆（交主 agent） | W2 |
| 同上 | 296 | 欠里去掉 C693（四块都判完之后） | W1 |
| 同上 | 387、391、393 | 连带串删一项、补注、欠删 C344 | W4 |
| 同上 | 397、248 | 括注换句、补括注 | W4 |
| `.claude/kb/decisions/05-快照-空间记账机制.md` | 145、146 | 两条依据整条重写 | W3 |
| 同上 | 42、159、169 | 「树高恒为 2」改成「段宽不动树高」 | W3 |
| 同上 | 75、87 | 第 3 行墓碑半句换句；欠改「无」 | W4 |
| `.claude/kb/checks-owed.md` | 132 | C159 记账轴一句改写 | W1 |
| 同上 | 252、277、314、452 | C344 搬进已还清；C383 措辞与取样点；例子拿掉 C344；C693 各块标已判 | W4、W1 |
| `.claude/kb/milestone/08-eighth-txn.md` | 34 | 欠账串去掉 C344 | W4 |
| `crates/singlefs-core/src/records.rs`、`recovery.rs` | 734、2797 起 | 树表条目三态（随 deadlist 合并，不是现在） | W2 |

不动的条款：D5 已定项 2（49 行起）、I-3.13（`crates/singlefs-checker/invariants/05-空间记账.md` 第 20 行）、D8 已定项 12 定案（383）、D8 已定项 8 ㊁ 定案句（262）。

## 要主 agent 现查或交别人的

- 读法 B1 与 B2 是主 agent 在装置写之前定的，我只指出翻面与否由它决定，没有判哪一个对。
- 1.2 节「47 行远小于叶容量 477 ⇒ 没有多写的节点」是行数对叶容量的算术，没在 `crates/` 或入库装置上量；477 来自 E97 装置的实现几何一行，我没独立重算那一行。
- 2.3 节「整表读不下去时恢复是否退到更旧的根」我没读选根逻辑，没下结论。
- 3.3 节「第 12 项一项的行数不超过窄读的两项」是包含关系，E71 与 E97 都没按今天的十二项表算。
- 我报的唯一自算数是 1.2 节的 `diff_rows.py` 与 `max rows_written_w2 = 47`，脚本与输出已贴，没在入库装置里；按 `.claude/rules/three-way-inference.md@判决由主 agent 做，不由投票做` 的口径，副本上的数不进 kb。

## 复跑

```
cd /home/fy5090/code/singlefs
python3 -I ~/.local/share/singlefs/work/m3-close-c693-r1-sonnet/diff_rows.py research/results/e186-unknown-tree-accounting-rederive.out.gz
grep ' name=cell ' research/results/e186-unknown-tree-accounting-rederive.out.gz | awk '{for(i=1;i<=NF;i++){if($i~/^rows_written_w2=/){split($i,a,"=");if(a[2]>m)m=a[2]}}} END{print "max rows_written_w2 =",m}'
grep -rn -i 'write_buffer\|write buffer\|WriteBuffer' crates --include=*.rs | wc -l
grep -rn -E '旧写者|不认识的树|unknown_tree|carry_unknown' crates --include=*.rs | grep -v '/tests/' | wc -l
python3 -c "print(59200/4075, 115200/4075)"
sha256sum research/results/e186-unknown-tree-accounting-rederive.out.gz research/results/e71-accounting-keys-single-generation-2026-10-20.out research/results/e97-entry-encoding-single-generation-2026-10-20.out
```

三份产物的 sha256（现算）：`f1dfd78bc92b7ed919af4484e334b379b615f3914b71a5cf36f513843439a86a`（E186，与页上登记一致）、`885c238a93c4519e58a4dd1f1251ebd0669f25bdbca7629ee21acfe94caa9c07`（E71，页上 V4 写的是 `885c238a…`）、`29cafdd37626b06fed52b4c3b8c66ce302be74a7e516cb0227e92e5e51610e58`（E97）。`diff_rows.py` 的 sha256 `79b8d5e200c53bcb3775d7b39a487d3397abb756ebf617422fb1f29d4fecb5cb`。没有写模型目录（本腿的算术只有这一个脚本，放草稿目录，不入库）。

## 没做什么

- 没有跑重型测试，没有编译 Rust，没有起门禁阶段；没有改 kb、`crates/`、装置、产物。
- 没有判读法 B1 与 B2 谁对，没有替主 agent 采纳；没有判 W1 的备选（1.6 节）要不要走。
- 没有读禁读清单里的文件；E186、E71、E97 的装置源码没读，只读了页与产物。
- 没有读恢复的选根逻辑；没有按今天的十二项表重算 E71 与 E97；没有在入库装置里复算 1.2 节的数。
- 摸底报告 ① 一节里 A14 指的 `.claude/kb/milestone/08-eighth-txn.md` 只核了第 34 行含 C344，没读全文。
- 没有给 W1 至 W4 每一格第二次独立抽样；这是一条腿的一次观测，攻方腿与主 agent 的现查才是校验。

## cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/m3-close-c693-r1-sonnet-output.md --background research/prompts/_m3-close-c693-r1-background.md
  ✓ 核了 85 处引文、41 处裸行号，对上 85 处，没判 41 处（其中裸行号没有引文可比 41 处；逐处列在上面）
```
