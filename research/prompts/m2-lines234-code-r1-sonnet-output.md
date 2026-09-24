# m2-lines234-code-r1：云端正推（Sonnet）交回——X3、X4、X5

分工：`crates/singlefs-core/src/inode_tree.rs`（X3）、`crates/singlefs-core/src/records.rs`（X4）、X5 一次发布重写范围。判据：每格只落「推得出 / 推不出 / 和条款说反话」三选一之一；抄不出整段原文的一律算「推不出」；不许拿「实现就是这么写的」当推导一步。

## 快照核对

`sha256sum -c research/prompts/m2-lines234-code-r1-snapshot/opening.sha256` 对这一格用到的三个文件（`inode_tree.rs`、`records.rs`）全部 `OK`：22 个文件与开工快照逐字节一致，本报告全部行号按主树现查，不用 diff 里的形态。

kb 原文整段抄进 `/tmp/claude-1000/lines234-sonnet/quotes.md`（`quote-kb.py` 回读逐字节一致，7 段），下面每处引用标出它在那份附录里的段落与在源文件里的行号（现查所得，不从背景材料里数）。

---

## X3　inode 树根的 key 区间取分隔 key 的首尾（`inode_tree.rs`）

**代码事实**：`inode_tree.rs` 第 76–83 行 `separator_key()`：

```
76	    /// 这片容器在父节点里的分隔 key：D8（核心索引结构） 已定项 6 逐字「**分隔 key** = 该孩子创建时的最小 key」。
...
81	    pub fn separator_key(&self) -> u64 {
82	        self.identity.container
83	    }
```

`separator_key()` 本身（分隔 key = 容器诞生时的最小 key）**兑现了条款**：`.claude/kb/decisions/08-核心索引结构.md` 第 160 行原句——

> 「**分隔 key** = 该孩子创建时的最小 key，条目按分隔 key 单调递增，查找取最后一个分隔 key ≤ 目标 key 的条目。」

（`grep -nF` 命中第 160 行，命中 1 次；整段见附录第 3 段，D8 已定项 6。）

但 X3 问的是**根节点头里那个 [min_key, max_key] 区间该取条目键区间还是子树覆盖区间**——这一步 `inode_tree.rs` 自己不做（它只产出 `separator_key()`，不组装码 2 头），实际组装在 `crates/singlefs-core/src/transaction.rs` 第 2025–2039 行（这个文件不在这一轮 22 个文件里，是消费 `inode_tree.rs` 输出的下游），那里把根的 `smallest_key` / `largest_key` 都取成 `leaf_containers.first()/.last()` 的 `separator_key()`——即**条目键区间**，不是子树覆盖区间（子树覆盖区间会是 `[第一片最小 inode 号, 最后一片最大 inode 号]`）。

**推不出**。理由：这一步没有任何已定分项能推出「取条目键区间」这个选择——`.claude/kb/checks-owed.md` 第 423 行 C478 原句（附录第 1 段）已经把这两个读法并列成没解决的分叉，逐字：

> 「D18（块里携带什么信息） 已定项 2 定「每一个码 2 索引节点都带 [min_key, max_key]」，理由逐字是「一个节点单独捡起来能自证它该覆盖哪一段 key」；而 checker 的 `check_index_node_keys`（`crates/singlefs-checker/src/lib.rs` 第 432 行）判的是「区间 = 首末两条**条目**的 key」。……并行线三建 233 个 inode 之后根写出的区间是 [2, 2]（一片容器、容器号 2），子树覆盖的却是 [2, 234] ⇒ 按条款的理由句读，这个节点「自证它该覆盖哪一段 key」写的是假话；按 checker 今天的读法它是对的。」

C478 自己给出的下一步是「先定条款（D18 已定项 2 补一句区间的口径……），再按定案改 checker 与写路径」——这句话本身就是「今天没有条款」的坐实：一个已还的欠账不会写「先定条款」。

（续下段：D18 已定项 2 原文核对、transaction.rs 里那句自造引用的核验、判据与推翻条件。）

**D18（块里携带什么信息） 已定项 2 原文核对**（附录第 4 段，`.claude/kb/decisions/18-块里携带什么信息.md` 第 48–52 行）：

> 「**定案**：**每一个码 2 索引节点都带 `[min_key, max_key]`，不按树分、不设开关。**……它买到的是：一个节点单独捡起来能自证它该覆盖哪一段 key，与 `.claude/rules/fs-design.md`「不为省空间牺牲自包含」同向。」

这条只定了「带不带」这个区间字段，没有一句说这个区间的两端该算成条目键还是子树覆盖——C478 的分叉正是从这句「自证它该覆盖哪一段 key」的措辞和 checker 今天的判法对不上来的。

**下游 `transaction.rs` 的自造引用**：第 2025 行的注释写「码 2 节点的 key 区间恒是这样，I-1.1（key 区间罩住条目）」——`grep -nF` 命中该行一次。但 `.claude/kb/invariants.md` 第 24 行登记的 I-1.1 简称是「块头自述逻辑地址」，条文是「任一单元，其头记录的**类内身份**与实际引用一致」，字面里没有「key 区间恒是条目键」这句判据，只在索引节点那一分句里写「+ 该树已定携带的 key 区间，D18（块里携带什么信息）已定项 2 口径」——即转手指回 D18 已定项 2，而 D18 已定项 2 恰恰没定条目键还是子树覆盖。`transaction.rs` 那句「I-1.1（key 区间罩住条目）」给 I-1.1 起了第二个简称，且这个简称本身就是把「条目键区间」当成已判定的事实写出来——这正是 `.claude/singlefs-ai-sop/rules/kb-discipline.md`「编号只能做索引，不能做称呼」条要防的失败：把编号换成它的定义句（块头自述逻辑地址），这句注释就读不通了。

**X3 判定：推不出。** `separator_key()` 的取值（分隔 key = 创建时最小 key）本身兑现 D8 已定项 6；但「根头的 [min_key, max_key] 取条目键区间、不取子树覆盖区间」这个选择没有任何已定分项支持，C478 已把它登记成待定账目，`transaction.rs` 那句注释引用的 I-1.1 与登记的定义不符。**两种取法在盘上分得开**：C478 给出的具体字节是并行线三建 233 个 inode 之后根写出 `[2, 2]`，子树覆盖读法会是 `[2, 234]`——同一个盘上位置两种取法给出不同的两个字节序列，`check_index_node_keys` 只判前一种（第 436–442 行，与今天的写法一致，不会红）。**今天没有会红的东西钉着后一种读法会不会被接受**：mutations.tsv 里没有一条把根区间从条目键改成子树覆盖的变异（`grep -n "smallest_key\|largest_key" crates/mutations.tsv` 零命中）。

**什么现象会推翻这个判定**：用户在 D18 已定项 2 补一句区间口径的定案（C478 给的下一步），且那句定案的字面等于「条目键区间」——到那时这一格从「推不出」变成「兑现了条款」；或者补的定案是「子树覆盖区间」，那今天的写法就从「替没写的条款做了选择」变成「和条款说反话」。

---

## X4　inode 记录 `blocks` 字段与建 inode 的固定值（`records.rs`）

**代码事实**，`crates/singlefs-core/src/records.rs`：

- 第 35–38 行，字段注释（`grep -nF` 命中第 37 行 1 次）：
  > 「字段表（D8（核心索引结构） 已定项 6）只写了这 8 字节叫 `blocks`，没写怎么算，所以由调用方按它给这个对象分了几个数据单元填。」
- 第 50–53 行 `to_bytes()`：`writer.put_u32(0o100_644); // mode`、`writer.put_u32(0); // uid`、`writer.put_u32(0); // gid`、`writer.put_u32(1); // nlink`（四行 `grep -nF` 各命中 1 次）。

**`InodeRecord` 结构体（第 30–41 行）没有 mode / uid / gid / nlink 字段**——这四个值不是「新建 inode 时给的默认值」，是 `to_bytes()` **无条件写死**在每一次记录序列化里，覆盖写（改 size、改 change_count）同样会把这四个值重新写一遍。这一点比 X4 题面「没有目录时新建 inode 的……写死」更宽：今天的实现里这四个字段**不可能**被写成除 `0o100644` / 0 / 0 / 1 以外的任何值——`records.rs` 里没有第二条产生这四个字段的代码路径（`grep -n "put_u32" crates/singlefs-core/src/records.rs` 只有这四处对应 mode/uid/gid/nlink）。

**`blocks` 字段判定：推不出，且已有登记（C480）。** `.claude/kb/checks-owed.md` 第 425 行原句（附录第 2 段）：

> 「inode 记录偏移 48 那 8 字节在 D8（核心索引结构） 已定项 6 的字段表里只写了名字叫 `blocks`、宽 8，**怎么算没有条款**；[layout/01-first-txn.md](layout/01-first-txn.md) 第 255 行登记的取值是「64（512 字节块计）」，那是第一个事务里唯一一个文件恰好占一个 32 KiB 数据单元时的数，不是口径。」

`records.rs` 第 37 行的自述与 C480 逐字吻合：这是实现自己承认「没写怎么算」的地方，不是我推出来的。**今天有会红的东西钉着「按分到的数据单元数填」这个选择**：`crates/mutations.tsv` 第 241 行——「并行线三：inode 记录的 blocks 写回常数 64（建出来没写过数据的 inode 也报占了一个单元）」，把 `occupied_blocks_of_512_bytes` 改回常数 64，判据 `widths_of_every_record_match_the_byte_table`（`grep -n "records.rs" crates/mutations.tsv` 第 241 行命中）。这条变异钉的是「不能退回常数 64」，钉不住「该怎么算」——C480 要补的口径（按分到的数据单元数 × 64，还是按 size 向上取整）今天两种都能通过这条变异。

**mode / uid / gid / nlink 判定：推不出，且全仓没有登记。** `.claude/kb/decisions/08-核心索引结构.md` 第 182 行字段表（附录第 3 段内）只给宽度，不给值：

> 「24 | mode / uid / gid / nlink，各 4 | 16」

`layout/01-first-txn.md` 第 254 行（附录第 7 段）把 `0o100644 / 0 / 0 / 1` 标成「已定」，但那一行说的是**第一个事务那唯一一次发布**里恰好写出的字节，指向仍是 D8 已定项 6——而已定项 6 原文（附录第 3 段）没有一句给出这四个默认值；「已定」在那张表里判的是「这个位置在那次发布里写出的字节能不能对上某条已定分项」，不是「这四个值是新建 inode 的通用默认策略」。`.claude/kb/milestone/02-second-txn.md` 第 520 行（附录第 6 段）自己也把这一处与 C478、C480 并列写出，但**没有给它配一个 C 编号**：

> 「……没有目录时新建 inode 的 mode / uid / gid / nlink 仍写死 `0o100644` / 0 / 0 / 1。」

`grep -rn "100644\|新建 inode\|默认.*mode" .claude/kb/checks-owed.md` 零命中——这四个值是三处（X3/X4 的 blocks/X4 的 mode 组）里唯一一个连欠账都没登记的选择。**没有会红的东西钉着**：`grep -n "put_u32(0o100\|// mode\|// uid\|// gid\|// nlink" crates/mutations.tsv` 零命中，四个值改成任何别的数字今天都不会被任何变异发现。

**X4 总判定：推不出。** `blocks` 那一半已被 C480 登记（有欠账、有变异钉住「不能是常数」但钉不住口径）；mode/uid/gid/nlink 那一半是四个连欠账都没有的写死值，且由于结构体本身缺字段，今天这四个值实际上是**唯一能写出的值**，不是「新建时的默认值」这么窄。

**什么现象会推翻这个判定**：D8 已定项 6 字段表补一句「mode/uid/gid/nlink 默认值 = ……」的定案，或者 `InodeRecord` 结构体加上这四个字段（说明它们已经不是写死的）；`blocks` 那半的推翻条件已经写在 C480 里（补口径 + checker 判 `blocks` 与 extent 叶记录数对得上）。

---

## X5　一次发布只重写被改到的那几片叶容器

**代码事实**：`inode_tree.rs` 第 176–184 行、第 189–234 行——`rewritten` 只在 `note_rewritten` 被调用时才收进一片容器的序号，调用点只有三处：原地换记录那一支（第 196 行）、追加进最右叶那一支（第 214 行）、末尾分裂新建右半那一支（第 227–230 行）。没被这三支碰到的容器，函数返回值 `containers` 里原样保留（`containers.to_vec()` 起手复制、循环里只改被碰到的下标），但**不进 `rewritten`**——调用方据此判断哪几片要重新分配落点、哪几片照抄上一版指针。

**没有条款、没有不变量，这一句是主 agent 自己核实过的现状，不是我这条腿推出来的**：`.claude/kb/milestone/02-second-txn.md` 第 520 行（附录第 6 段）逐字——

> 「**一次发布只重写被改到的那几片叶容器**今天只有用例钉着，没有条款也没有不变量。」

我核过 `.claude/kb/invariants.md` 里全部 I-9 系列（I-9.1 至 I-9.14，第 265–278 行）：没有一条判「未被这次发布碰到的叶容器，COW 前后身份 / 内容逐字节相同」或「重写清单之外的容器指针不变」。我也核过 D8（核心索引结构） 已定项 6 全文（附录第 3 段）：第 158 行只有「更新一条记录 = COW 叶 + 全部祖先」，管的是**被更新那条记录所在路径**要往上 COW 到根，没有一句说**没被更新的兄弟叶不许被牵连重写**。`.claude/kb/decisions/16-发布语义.md`、`21-权威态与派生态的分界.md` 里也没有「COW 只动被改路径」这句通用判据（`grep -n "只重写\|未改动.*不重写" .claude/kb/decisions/*.md` 只命中与本题无关的一处，见上一次检索）。

**这不该是一条不变量的理由**：I-9 那一类是靠**checker 读一份镜像**判的（"对着一个镜像判不了" 是 `inode_tree.rs` 文件头注释第 10–12 行原话，`grep -nF` 命中）——checker 只看得到某一次发布之后盘上的字节，看不到「这片容器这次发布前后的物理指针是不是同一个」，那需要跨两代的落点对照，是操作断言而不是单镜像不变量，与文件头注释所属的 C116（叶容器的分裂 / 合并纪律没有会失败的检查） 同族（这一点是主 agent 转述文件头注释，不是我替它下结论）。

**用例钉住的够不够**：`inode_tree.rs` 自己的单测里，第 382–428 行 `the_two_hundred_thirty_fourth_record_splits_at_the_end_and_the_left_half_keeps_everything` 断言分裂后 `after.rewritten == vec![位置 1]`（只有右半进清单）且 `after.containers[0] == before[0]`（左半逐字段相同）；第 592–625 行 `one_write_that_both_replaces_and_appends_lists_each_touched_container_once` 断言一次写里改一条、追加两条时 `rewritten` 只有两个位置、不含中间那片没碰到的容器。`crates/singlefs-harness/tests/second_transaction_parallel_line_three_many_inodes.rs` 第 136、308 行同样断言 `after.rewritten`（这个测试文件不在这一轮 22 个文件之列，是佐证材料，不是这一格要判的对象）。但 `crates/mutations.tsv` 里点名 `inode_tree.rs` 的 6 条变异（第 230–235 行，已在 X4 段核过）没有一条改 `note_rewritten` 的调用点或去掉某处「不调用它」——也就是说，「把一片没被碰到的容器也塞进 `rewritten`」或者「漏记一片真被碰到的容器」这两种变异今天不在变异表里，**不进复跑门禁**，只在有人手跑这几条单测时才会被发现。

**X5 判定：推不出，这一格与 kb 里主 agent 的观测一致，不是分歧。** 「它该不该是一条不变量」这问题本身不是我这条腿的判据范围（判据 3 要求的是「今天有没有会红的东西钉着」，不是「该不该立」这个设计取舍——那是留给主 agent 的岔路）；我能核实的是：今天没有条款、没有不变量、有单测断言但没有变异表条目。

**什么现象会推翻这个判定**：`crates/mutations.tsv` 里出现一条改 `note_rewritten` 调用点或调用条件的变异且判红——那时「用例钉住的够不够」这问题的答案会从「只有单测、没进复跑门禁」变成「进了复跑门禁」；或者用户在 D8 已定项 6 或 I-9 系列里补一句「未被本次发布碰到的容器物理指针不变」，那时这一格从「推不出」变成「兑现了条款」。

---

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| X3 | 推不出 | `separator_key()`（分隔 key = 创建时最小 key）兑现 D8 已定项 6，但根头 `[min_key, max_key]` 取条目键区间而非子树覆盖区间没有任何已定分项支持，C478 明登为待定账目，`transaction.rs` 里那句引用 I-1.1 的注释与该编号登记的定义不符（自造简称） |
| X4（`blocks`） | 推不出 | 字段表只给宽度、不给算法，`records.rs` 自述「没写怎么算」，与 C480 逐字吻合；`crates/mutations.tsv` 第 241 行只钉住「不能是常数 64」，钉不住该按什么口径算 |
| X4（mode/uid/gid/nlink） | 推不出 | D8 已定项 6 字段表只给宽度、不给默认值；`layout/01-first-txn.md` 第 254 行的「已定」说的是那一次发布写出的字节，不是通用策略；这四个值全仓没有 C 编号登记，也没有任何变异钉住 |
| X5 | 推不出 | 与主 agent 的观测一致（milestone/02-second-txn.md 第 520 行原句）：没有条款、没有不变量；`inode_tree.rs` 与 harness 的单测断言了这一行为，但 `crates/mutations.tsv` 里 6 条点名 `inode_tree.rs` 的变异都不改这一处，没进复跑门禁 |

## 没做什么

- 没判 X1、X2、X6（分给云端攻方）与「22 个文件的测试覆盖」那一维（分给本地攻方）。
- X5「它该不该是一条不变量」这个设计取舍本身不在我的判据范围内（判据 3 只问「今天有没有会红的东西钉着」），留给主 agent 定；我只核实了「没有条款、没有不变量、有单测、没进变异表」这四项事实。
- 没有跑 `cargo test` 复核这几条单测今天是不是真的通过——按 `.claude/agent-common.md`「不做」一节，不编译 Rust；`crates.diff` 与 `sha256sum -c` 已确认这三个源文件与开工快照逐字节一致，单测源码本身是现查的。
- 没有去核 `transaction.rs`、`second_transaction_parallel_line_three_many_inodes.rs` 之外别处是否还引用了同一个自造的「I-1.1（key 区间罩住条目）」简称——只核了 X3 用到的这一处。
- `.claude/kb/checks-owed.md`、`decisions/08-核心索引结构.md`、`decisions/18-块里携带什么信息.md`、`invariants.md`、`milestone/02-second-txn.md`、`layout/01-first-txn.md` 六份 kb 文件的引文已用 `research/scripts/quote-kb.py` 整段抽取并回读比对（`/tmp/claude-1000/lines234-sonnet/quotes.md`，7 段，7 段全部逐字节一致，退出码 0），行号已在正文各处标出；没有用这份脚本抽取的引文（`transaction.rs`、`mutations.tsv`、`inode_tree.rs` 自身源码）改用 `grep -nF` 现查命中行号，逐条列在正文里。
