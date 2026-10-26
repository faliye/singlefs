# m4-validate-code-r2：里程碑四验证段代码轮第二轮正文（2026-10-24）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 -->

## 一、题面

这一轮攻 m4-validate-code-r1 判决之后，这一段打进主工作区、还没被任何代码轮快照罩住的产品代码。来源三份补丁，都只在工作区里（没提交）：

| 来源 | 关的 | 落点（产品代码） |
|---|---|---|
| 标签 0 写者照写（`~/.local/share/singlefs/work/m4-tag-zero-writer/patch/crates.patch`） | 判决 `research/prompts/m4-validate-code-r1-main-verification.md`「四、交用户」T5 甲，用户定「丑」 | `crates/singlefs-core/src/extent_tree.rs`、`transaction.rs`、`truncate.rs` |
| 隔离的盘上记号，落点甲（`~/.local/share/singlefs/work/m4-quarantine-mark-writer/patch/crates.patch`） | E187（隔离记号与回退）岔路用户定「落点甲，checker 只认记号」；D19（块指针的结构与宽度预算） 已定项 5「隔离的盘上记号」一段 | `allocation_record_tree.rs`、`allocator.rs`、`compaction_plan.rs`、`mount.rs`、`recovery.rs`、`region_scan.rs`、`transaction.rs` |
| 测试删自己的临时目录（`~/.local/share/singlefs/work/m4-tests-delete-temp-dirs/patch/crates.patch`） | 无条款：只核测试临时目录守卫不改被测行为 | `block_device.rs` 的 `#[cfg(test)]` 模块 |

改动全文：`research/prompts/_m4-validate-code-r2-diff.md`，三份补丁里落在产品代码的部分（标签 0：3 份 13 个 hunk；隔离记号：7 份 34 个 hunk；临时目录：1 份 18 个 hunk）。三份对此刻主工作区 `git apply --check --reverse` 都退 0。这几份文件里另有别的会话打进来、还没提交的改动（`git diff HEAD --stat` 列 24 份产品代码文件），归它们自己的代码轮，这一轮不攻；格里引到它们时写明「不是这一批的」。

隔离记号那一批里另有池级 checker（`crates/singlefs-checker/`）、GPU 判器内核、harness 与 checker 档的改动，是测试代码，不进这一轮；格里要拿它们当对照时可以读。

## 二、实现今天的样子（主 agent 的观测，2026-10-24）

- 冻结副本：`~/.local/share/singlefs/work/m4-validate-code-r2/repo/`，此刻主工作区的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`（`cp -r`，不带 `target`）。被判的 10 份产品代码文件与主工作区逐字节相同（`cmp`）。
- 读到的事实：
  - D19（块指针的结构与宽度预算） 已定项 5 第 117 行「隔离的盘上记号」：写者隔离那次发布把两块盘上那条分配记录的跨度段置 0x4000（最高位 0x8000 仍是已释放标志），分配代不变；回退复活那一次同一次发布清掉两块盘上的 0x4000；释放隔离记录时改写成已释放、0x4000 随之清掉；池级 checker 只认记号。
  - D8（核心索引结构） 已定项 14 第 451 行起：上段叶条目标签 0 全零是第一个文件一个数据单元都没有而 size > 0；第 477 行：用户定案「写者接得住这一形」。
  - 落点甲实现员报告（`~/.local/share/singlefs/work/m4-quarantine-mark-writer/report.md`）自报：核心层今天没有释放隔离记录的路径，只单测走得到；`AllocationRecord::parse` 认 0x4000、0xC000 判坏。它停下的五条主 agent 已判（`records/2026-10-13-里程碑四预开发.md` 那一段）：区域扫描不认记号、回退遇一块盘带记号照清照做、内存里「已释放又带记号」写得出只靠断言拦，三条记欠账；旧盘不带记号的隔离一律判违例是用户选的后果。
- 这一段的验证情况：合入后验证 2026-10-23 那一趟 harness 档 315 个测试二进制 3037 条全绿、fmt、clippy、build 过；变异轻型点名这一段 94 行加核心部分，356 条里抓到 332、没红 0、无效 0、4 条撞内存上限。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　标签 0 配 size > 0：写者接得住这一形
`transaction.rs` 的 `publish_size_change`、`truncate.rs` 那两处断言的前提、`extent_tree.rs` 新加的那一条，对判决 m4-validate-code-r1「四、交用户」T5 甲的「丑」：0 个单元的文件变长、N = S、截短都只改记录，标签留 0。格里要问：三种改 size 是不是都不碰 extent 树、不取 txg 之前就 panic；截到 0 与从 0 变长的边界；标签 0 的文件之后第一次写数据单元，上段叶条目怎么从标签 0 换成别的标签。

### T2　隔离那次发布置记号
`allocator.rs`、`allocation_record_tree.rs`、`transaction.rs`，对 D19 已定项 5 第 117 行：两块盘都置 0x4000、分配代不变、与那次发布的别的改动一起落盘。格里要问：只有一块盘读坏时另一块盘那条记录置不置；同一次发布里隔离两个槽；记号与已释放标志的组合是不是只写得出允许的那几种。

### T3　清记号：回退复活与释放
`mount.rs` 的回退那一路、`allocator.rs` 释放那一路，对同一段「回退复活那一次同一次发布清掉两块盘上的 0x4000；释放隔离记录时改写成已释放」。格里要问：回退目标里本来没有那条隔离的，清不清、清错的后果；核心层没有释放隔离记录的路径时，这一句有没有被别的路径（整理、扫描重建）意外走到。

### T4　读者认记号
`allocation_record_tree.rs` 的 `AllocationRecord::parse`、`recovery.rs`、`region_scan.rs`、`compaction_plan.rs`、`mount.rs` 里读跨度段的每一处，对 D3（空间分配） 第 131 行跨度段那一行与 `.claude/kb/layout/01-first-txn.md` 第 299 行：跨度取值要去掉记号位再用；0xC000 判坏。格里要问：有没有哪一处读跨度没去掉 0x4000（把跨度读大 16384 倍）；区域扫描、整理计划、恢复三处对带记号的记录各怎么处置，与「安全一侧」的判法对不对得上。

### T5　第一轮的改法本身与从没被看过的地方
第二轮的分工照 `.claude/rules/three-way-inference.md`「三轮分工」第二轮：第一轮的修补本身、产生结论的方法、从未被看过的地方。格里要问：T1 的「丑」是不是把判决里「要改 K5 的『变长』一句与 `truncate.rs` 两处断言的前提」都做到了；r1 判决第四节第二条岔路（大 size 冷读与 RefFS 整份分配）还开着，这一批有没有让它更容易打中；不带 incompat 位的旧读者遇到带记号的池在哪一层拒、拒得对不对（E187 第一段 old_reader 那一行）。

### T6　测试临时目录守卫
`block_device.rs` 的 `#[cfg(test)]` 模块，只核守卫（`TemporaryPathRemovedOnDrop`）不改被测路径、测试照旧判同一件事。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1–T6 | 复核 m4-validate-code-r1 的判决够不够得着这一批：逐格对判决原文、D19 已定项 5、D8 已定项 14 与冻结副本的行，核「代码做的是不是用户定的那一种」；判决里定的改法是不是同样打中替代方案，每条结论写文件:行号 |
| 云端攻方（Opus） | T1–T6 | 在冻结副本的拷贝上造反例：带记号的合法镜像被拒、坏镜像被放行；某处读跨度没去掉记号；回退或释放把记号清错、漏清；标签 0 的文件改 size 之后读出的内容或 extent 不对。每个打中给可复现的命令与原样输出；不许重复第一轮已攻过的角度（第一轮判决第二节逐格写着） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。量法是拷冻结副本，不在冻结副本里改。拷编译目录当种子之前先 `find crates -type f -exec touch {} +`，让改过的文件都重编。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑。线程上限照派发提示，用自己的编译目录，跑完删。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
