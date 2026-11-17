# 池级 checker 在途改动的代码轮（补轮）

日期：2026-11-16。

## 范围

被判的是 `crates/singlefs-checker/src/` 下五份文件工作区相对 HEAD 的改动：`image.rs`、`lib.rs`、`position_addressed.rs`、`rebuild.rs`、`walk.rs`。材料都在 `research/prompts/checker-src-backfill-code-r1-snapshot/`：

- `changes.patch`：`git diff HEAD -- <这五份>` 的原样输出（1728 行）。
- `crates-sha256.txt`：五份文件开工时的 sha256。
- `facts.md`：事务员出的逐文件事实表（逐块一句、注释里引的编号、检查单八格的原始事实、测试与变异、线索）。它是线索：`walk.rs`、`rebuild.rs` 的逐块一句是脚本从 diff 里取的「函数上下文加首个加行」，没有逐块通读。
- `mutation-rows-new.tsv`：`crates/mutations.tsv` 里指到这五份、相对 HEAD 新出现的 21 行（按文件、原文、替换文三段为键比的）。`crates/mutations.tsv` 整份另开一题判，这一轮只判这 21 行罩不罩得住下面四件改动。

方案按 `crates/` 今天的实现来谈。

## 实现今天的样子（主 agent 的观测）

读过 `crates/singlefs-checker/src/image.rs:958-980`、`crates/singlefs-checker/src/walk.rs:814-828`、`crates/singlefs-checker/src/walk.rs:3743-3762` 与 `changes.patch`。改动是四件事加一批改名：

1. **环长判据加一格**（`image.rs:958-980`，`journal_ring_bytes_lie_in_the_supported_range`）：在飞记录数上限 `环长 ÷ JOURNAL_RECORD_BYTES ÷ JOURNAL_SAFETY_FACTOR` 除了不为 0，还要 `u32::try_from(...).is_ok()`。注释说依据是 D23（journal 的角色与格式） 已定项 18 与 `.claude/kb/layout/01-first-txn.md` 一「journal 在飞记录数上限 | 4」，装不进时归 I-7.13 先拒。新测试 `journal_ring_whose_in_flight_record_limit_overflows_its_four_byte_field_lies_outside_the_supported_range`。
2. **CRC 表法加按内容复用**（`lib.rs:57-200`，`crc32_castagnoli_table`）：每条线程一份记录（`thread_local!`），指纹是长度加 16 个固定位置的字，候选逐字节比对相同才取记下的值，每条线程记下的内容上限 8 MiB、超过整份清空，一个指纹之下至多 4 个候选；新类型 `ContentFingerprint`、`RememberedChecksum`、`RememberedChecksumsOfThisThread`，`#[cfg(test)]` 的计数，新测试 6 条。
3. **I-3.5 加「树表里有走到没有条款的形状的条目」一路**（`walk.rs:814-828` 的 `tree_table_entry_is_a_shape_without_a_clause`、`walk.rs:3743-3762` 的 `Walk::note_the_first_tree_table_entry_without_a_clause`）：这一版树表里有这样的条目，它的种类记进这一版走完的输入，`judge_the_appearance_rule_of_adjacent_candidate_versions` 里它参与的相邻一对报「不判」；注释说不进 `walk_failures`、由 I-9.16 ④ 判红。条款是 `crates/singlefs-checker/invariants/05-空间记账.md` I-3.5 那一行。
4. **扫描重建的错误成员改名**（`rebuild.rs`，`ScanRebuildNotDecided` 三个成员去掉名字里的 `I18`）。全仓 `grep -rn "TotalOrderKeyOfI18\|TotalOrderOfI18" crates scripts`（`.rs`、`.tsv`、`.py`）零命中。
5. 一批测试函数与局部变量去掉 `a_`、`an_` 前缀（`a_later_root_is_incomplete` 改成 `some_later_root_is_incomplete` 这一类），`walk.rs:7270` 一处文档注释改成普通注释。

## 要两条腿答的

- 正推：四件改动各自做的是不是它注释点名的条款说的（条款原文去 kb 与不变量清单里现读，引 kb 条款写 kb 文件名加那份文件自己的行号）；改名那一批有没有哪一处不是纯改名。
- 攻方：哪一格会错。至少攻这几处：按内容复用会不会让 `crc32_castagnoli_table` 对某个输入返回与现算不同的值（指纹取样、候选比对、清空、`#[cfg(test)]` 计数与上限覆盖会不会漏进非测试构建）；它是不是让池级 checker 的判定依赖了调用次序或线程；环长那一格与实现那一侧收的是不是同一张表、3<<44 附近的边界；I-3.5 那一路里「共享树表的版本不重走条目」时记的种类对不对、livelist 与稀疏旁表根指针全零与非全零两种的分法与条款对不对得上、「不判」会不会把本该红的镜像放成不红；新加的 21 行变异罩不罩得住这四件改动的每一处判红分支。
- 两条腿都逐格判下面的检查单，回「成立」「不成立」或「推不出」。

### 检查单：走不到的理由二次查实

这批改动没有实现员报告（补轮），照 `facts.md` 各文件第 3 样列：生产段新加的只有 `lib.rs` 三处 `expect`——`lib.rs:154`「候选数装得进 u64」（在 `#[cfg(test)]` 计数那一句里，腿核它是不是只在测试构建里）、`lib.rs:163`「小于 256」、`lib.rs:178`「低 8 位」。`image.rs`、`rebuild.rs`、`walk.rs`、`position_addressed.rs` 生产段没有新加 `unreachable!`、`panic!`、`expect`、`unwrap`、`assert!`。事实表按 `mod tests` 之前机械划分生产段，腿在 `changes.patch` 里自己再数一遍。

### 检查单：类型与名字

新加的名字：`lib.rs` 的 `REMEMBERED_CONTENT_BYTES_LIMIT_PER_THREAD`、`REMEMBERED_CANDIDATES_PER_FINGERPRINT_LIMIT`、`FINGERPRINT_SAMPLED_WORD_COUNT`、`ContentFingerprint`、`RememberedChecksum`、`RememberedChecksumsOfThisThread` 与几个线程局部量（全表在 `facts.md` 的 `lib.rs` 一节）；`walk.rs` 的 `tree_table_entry_is_a_shape_without_a_clause`、`note_the_first_tree_table_entry_without_a_clause`、字段 `kind_of_a_tree_table_entry_without_a_clause_in_this_version`、`root_pointer_is_all_zero`；`rebuild.rs` 三个改名后的成员。事实表报生产段没有新加 `_ =>` 与 `as`；`walk.rs:823` 的臂写成 `other_kind => key_schema_for_tree_kind(other_kind).is_none()`，腿判它算不算通配臂（`kind` 是盘上读来的 `u16`）。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：事实表按关键词（旧版、旧布局、旧格式、legacy、migrat、迁移、兼容旧、backward）查五份的加行，零命中。

### 检查单：不新建判定库

本轮改动没有新的判定存放处：事实表按关键词（rocksdb、sqlite、prune-server）查加行零命中。`lib.rs` 的按内容复用是进程内每条线程一份的记忆、不落盘；它算不算「判定复用」、该不该受 `.claude/rules/verification.md`「判定库只有一份」管，腿各自判。

### 检查单：所有任务多机可用

本轮改动没有这一类：事实表按关键词（hostname、`/proc/`、`/sys/`、`env::var`）查加行零命中。`lib.rs:60` 的注释点名 `.claude/heavy-thread-default`，只在注释里。

### 检查单：测试优先多线程

`lib.rs` 的新测试经 `std::thread::spawn` 在新线程上跑（`lib.rs:1359`），为的是线程局部记录从空的起，不是切片并行；别的新测试是单场景小用例。没有写死或覆盖线程数的地方（事实表查 `--test-threads`、`available_parallelism` 零命中）。

### 检查单：带 --item 的命令与项

本轮改动没有新加可点名的项：`.claude/gate.d/stage-inputs.tsv` 与崩溃用例登记表没有指到池级 checker 的行（`facts.md` 所在事实表的全局事实一节）；新测试是池级 checker 的库单测与验证包已有测试目标里的用例，随测试目标点名。新加的变异行在 `mutation-rows-new.tsv`，由门禁 crates-mutation-replay 按行点名。

### 检查单：新崩溃流多罩了什么

本轮改动没有新加崩溃点重放流或崩溃枚举用例（事实表查五份的加行，非注释行里提到崩溃的 0 处）。
