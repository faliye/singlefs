# C673（三 oracle 合谋测试的补 oracle 改法没做） 里要先定条款的三条：第一轮正文（2026-10-03）

<!-- doc-lint:not-numbers O1 O2 O3 K1 K2 K3 -->

## 一、题面

E171（三oracle合谋测试） 结论三轮（判决 `research/prompts/e171-conclusion-r3-main-verification.md`）之后，C673 列了十条补 oracle 的改法。其中三条在派实现员时停下，因为要先定条款或新立判据：

- **C673 ⑤**：池级 checker 判树表条目的头 ID 取值（E171 第 271 行：`deleted_inodes` 的树表条目头 ID 写 0，三条 oracle 都放行）。
- **C673 ⑥**：池级 checker 判新池新建文件的落点（E171 第 15 行：用户数据落点挪到开放段起点、十一个提交内生块整体后移 64 槽，三条都放行）。用户 2026-10-03 定：新池新建文件上用户数据的落点取值 t1@50180 算 layout 登记的字节，偏离它写出的新池新建文件镜像不合法（D3（空间分配） 已定项 8 射程）。
- **C673 ⑩**：D8（核心索引结构） 已定项 6 补「写入时间」取值一句（E171 第 92 行：覆盖写照抄上一版的写入时间，三条都看不见）。

实现员停下的报告：`/tmp/claude-1000/m3-formal-16-1/report.md`（主 agent 读过；腿不读 `/tmp`，要点写在第二节）。

## 二、实现今天的样子（主 agent 的观测，2026-10-03）

- 池级 checker 每条判定都要一个登记过的不变量 ID：`crates/singlefs-checker/src/image.rs` 第 190 行 `judge` 断言 ID 在 `IMPLEMENTED_INVARIANTS` 里，那张表的长度要等于 `.claude/kb/invariants.md` 状态列「已实现」的行数；GPU 判器的覆盖表要与它同长（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 第 74 行一带的编译期断言），加一条不变量要同时补 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 的内核。
- 树表条目：池级 checker 走树表在 `crates/singlefs-checker/src/walk.rs`（第 72、90、3040、3576 行一带；排序判在第 2953 行 `judge_tree_table_entries_ordering`），不读条目偏移 116 的头 ID（实现员读码，没跑）。
- 新池新建文件的落点：harness 档已有用例按字节钉住新池新建文件的写（`crates/singlefs-harness/tests/new_pool_file_creation_write_regions.rs`、`crates/singlefs-harness/tests/new_pool_file_creation_publish.rs` 里出现 50180）；E171 第 15 行那条变异（`crates/mutations.tsv` 第 1040 行「实审 A2c 第 15 条：用户数据落点不从这张图的单元区起点起扫」）点名的用例是 `core_review_unit_area_start_and_publish_limits` 里的一条。三条 oracle（O1 模型对拍、O2 池级 checker、O3 规约执行器）都不判落点。
- 写入时间：inode 记录的写入时间是发布参数、不取系统时钟（`.claude/kb/milestone/01-first-txn.md` 第 103 行、`02-second-txn.md` 第 103 行）；`layout/01-first-txn.md` 第 267、269 行登记「写入时刻」；里程碑二第 113 行验收写「写入时间留成 A 的 ⇒ 判红」。D8 已定项 6 正文没有这一句（第二轮判决 `research/prompts/e171-conclusion-r2-main-verification.md` 第 28 行）。

方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　树表条目的头 ID 怎么判（C673 ⑤）
D8（核心索引结构） 已定项 8「头 ID 8 字节的取值」逐字写 65537（inode 树、extent 树、`deleted_inodes`、墓碑树），而回退之后树会从水位重新发号。要定：
1. 判据读字面 65537，还是「同一张树表里 inode 树条目的树 ID」；字面那一读在哪一段合法历史上会红（举出来）。
2. 一张树表里 inode 树条目 0 条或多于 1 条时怎么判。
3. 按条目判还是按一版（一张树表）判；判据放进哪一条不变量（新立，还是并进已有的一条）。
4. 新立的这一条，GPU 判器同判的代价与写法（覆盖表同长的编译期断言逼着它）。

### K2　新池新建文件的落点怎么判（C673 ⑥）
候选（腿可以补，补的写明定义）：
- 甲：O2 新立一条不变量，先从单镜像认出「新池新建文件那一版」再判落点；认法要能写成条款（实现员推的认法：根的实例代号 1、`checkpoint_txg` 3、系统配置里单元区起点 50176、extent 树恰好引用一个数据单元且 inode 树恰好一条记录）。
- 乙：不在 O2 判，放 O1（模型给新池新建文件的落点答）。
- 丙：不新立判据，交给已有的按字节钉写的 harness 用例（第二节第 3 条），C673 ⑥ 改写成「不由三条 oracle 判，由钉写的用例判」。
要连同答：照 `layout/01-first-txn.md` 逐字段去认会不会变空真（实现员报树表条目 layout 第 78 行写 7 条、实现写 11 条）；非默认 journal 环（单元区起点不是 50176）、第一次发布写多个数据单元或零长文件算不算；t2–t12 要不要一起判（D3（空间分配） 已定项 10 射程里有没有「偏离即不合法」）。

### K3　D8 已定项 6 的写入时间（C673 ⑩）
补的那一句怎么写（取值是哪一次发布的参数、覆盖写与新建各是什么），放定案还是射程；三条 oracle 里哪一条判得到它（单镜像判不出发布参数）、判不到的写进「验不到什么」或记欠。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K2、K3 | 逐格把候选对到 kb 原文与 `crates/` 今天的代码，给出每格可直接写回的条款句与判据（标零轮） |
| 云端攻方（Opus） | K1、K2、K3 | 攻每个候选：造让它在合法历史上误红的镜像、或让坏实现照样过的形态；量得了的写小模型量（副本），量不了的写推的 |

## 五、跑前写死的判据

- 两腿同判且主 agent 现查坐实 ⇒ 按那一判；分歧 ⇒ 开第二轮。新立的不变量、新判据都是零轮，写回前交用户表标明。
- K2 选丙时，C673 ⑥ 改写、不新立不变量；选甲或乙时，那一判的实现并进下一批实现员（连同 GPU 覆盖表与内核）。

交付：腿的报告写 `research/prompts/c673-clauses-r1-<腿名>-output.md`，分段落盘（每次写进文件不超过 150 行，第一段排他新建，回复只写短句）；模型与脚本放 `research/prompts/c673-clauses-r1-<腿名>-model/`；草稿放 `/tmp/claude-1000/<腿名>/`。引 kb 条款写 kb 文件自己的行号（去 kb 文件里现查，不从背景材料里数）。方案按 `crates/` 今天的实现来谈。
