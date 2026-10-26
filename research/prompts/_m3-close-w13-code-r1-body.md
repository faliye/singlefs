# m3-close-w13-code-r1：mkfs 两趟写序与 fsid 生成、六处读者拒宽于字段表的条目（代码轮第一轮）正文（2026-10-21）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 -->

## 一、题面

里程碑三收尾批 13 往主工作区合进了两段产品代码改动，这一轮攻这两段：
- **mkfs 两趟写序与 fsid 生成**：C656（mkfs 中途崩溃的语义没有条款）与 C659（fsid 前 8 字节相同时重做 mkfs 实例代号不变式红），条款是 D22（单元原子性怎么合成） 已定项 8 第 3 条与它依据里「mkfs 两趟写序与中途崩溃的条文」「fsid 由对外入口生成、前 8 字节取自随机源」两行（判决 `research/prompts/m3-close-c656-r3-main-verification.md` 第二节 S1、S2、S3 与第三节；用户 2026-10-20 定「认，照改写稿写回」「对外生成，留内层入口」）。
- **六处读者拒宽于字段表的条目**：C867（另外六处读者只拒窄于字段表的条目宽），条款是不变量 I-1.10（码 2 条目宽等于字段表宽）。

被判的是 `crates/singlefs-core/src/` 下 7 份文件：`make_filesystem.rs`、`extent_tree.rs`、`allocator.rs`、`allocation_record_tree.rs`、`records.rs`、`recovery.rs`、`mounted_read.rs`。这 7 份相对 HEAD 841b3beab 的改动全是这两段，diff 原样存在 `research/prompts/m3-close-w13-code-r1-snapshot/product-code.diff`（419 行）。

这一轮要的是两件：代码做的是不是条款说的；哪一格会错。

## 二、实现今天的样子（主 agent 的观测，2026-10-21，行号按主工作区；7 份文件的 sha256 在 `research/prompts/m3-close-w13-code-r1-snapshot/code-sha256.txt`）

**mkfs**
- 对外入口 `make_filesystem_drawing_the_filesystem_identifier`（`make_filesystem.rs:254`）从 `RANDOM_SOURCE_PATH`（`make_filesystem.rs:224`，`/dev/urandom`）取 fsid，再调内层 `make_filesystem`（`make_filesystem.rs:533`）；内层入口的签名不变，fsid 由调用方给。
- 内层 `make_filesystem` 改成两趟写序：第一段是每块盘 [0, 根环基址) 清零（作废旧池的系统配置槽），一道屏障；然后其余清零与单元写、三条第 0 代根、四个系统配置槽各成段。checker 档用例 `crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs` 断言的段长是 `[2, 12, 1, 1, 1, 4]`。
- harness 档用例 `crates/singlefs-harness/tests/checker_a_pool_remade_over_an_old_pool_sharing_the_first_eight_identifier_bytes.rs` 造 fsid 前 8 字节相同的新旧池。

**读者**
- `extent_tree.rs`、`allocator.rs`、`allocation_record_tree.rs`、`records.rs`、`recovery.rs`、`mounted_read.rs` 里判条目宽的比较从「窄于字段表就拒」改成「不等于字段表就拒」。
- 拒绝的枚举成员 `PointerCarryingEntryRefusal::NarrowerThanItsFieldTable` 改名 `WidthOtherThanItsFieldTable`。
- 新用例 `crates/singlefs-harness/tests/tree_readers_refuse_extent_inode_allocation_and_deadlist_entries_wider_than_the_field_table.rs`。
- 实现员交回时报了还没改的几处，在这一轮之外：`RecoveryFailure::EntryNarrowerThanItsFieldTable` 这个名字、`recovery.rs` 文档注释里仍写「窄于 34」、extent 上段叶第七处、树表条目大于 200 的用例。

方案按 `crates/` 今天的实现来谈。

## 三、格

| 格 | 问什么 |
|---|---|
| K1 | 两趟写序与条文逐句对：第一段只含 [0, 根环基址) 清零、段后一道屏障；条文里的其余清零、根与系统配置槽的次序与分段，代码是不是逐条照做；有没有一条写落在条文没说的段里 |
| K2 | fsid：对外入口取随机源、前 8 字节的来源与条文一致；读不出随机源、读得不够时怎么办（拒、还是退回别的来源）；内层入口留着，谁还在调它、调它的地方会不会给出前 8 字节与旧池相同的 fsid |
| K3 | mkfs 中途崩溃：照条文，第一段持久前、持久后各崩，恢复与挂载落到什么结局；代码的写序能不能让一个崩溃态落到条文不许的结局 |
| K4 | 六处读者：每一处是不是真的「不等于就拒」，比较的两边是不是同一个量（条目宽与字段表宽，单位都是字节）；有没有第七处、第八处同形的读者没改（读码查全仓，写出 grep 命令与命中数） |
| K5 | 改名与拒因：`WidthOtherThanItsFieldTable` 在每个调用方的处置与改名前相同；拒宽条目会不会把一个合法的盘拒掉（例：将来某类条目合法地带尾随字段） |

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1–K5 | 核代码做的是不是条款说的：逐格从附录的条款原文推到 diff 与现行代码 |
| 云端攻方（Opus） | K1–K5 | 攻哪一格会错：找让代码与条款不一致的输入、崩溃点或调用方 |

引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
