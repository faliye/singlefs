# m4-validate-code-r3：里程碑四验证段代码轮第三轮正文（2026-10-24）

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 -->

## 一、题面

这一轮攻 m4-validate-code-r2 判决之后打进主工作区的产品代码改动，即判决第四节用户定的改法甲。来源只有一份补丁，在工作区里、没提交：

| 来源 | 关的 | 落点（产品代码） |
|---|---|---|
| 验证段修补一（`~/.local/share/singlefs/work/m4-validate-fix-one/patch/crates.patch`） | 判决 `research/prompts/m4-validate-code-r2-main-verification.md` 第二节 T1 行与第四节：用户 2026-10-24 定「甲：一律写标签 0」 | `crates/singlefs-core/src/extent_tree.rs`、`crates/singlefs-core/src/transaction.rs` |

改动全文：`research/prompts/_m4-validate-code-r3-diff.md`，2 份文件、5 个 hunk，对此刻主工作区 `git apply --check --reverse` 退 0。同一份补丁另改了 harness 与 checker 档的测试、`crates/singlefs-checker-tier/src/crash_judge_conclusions.rs` 的测试模块与 `crates/mutations.tsv`，是测试代码，不进这一轮，格里拿它们当对照时可以读。这两份文件里另有别的会话打进来、还没提交的改动，归它们自己的代码轮，这一轮不攻；格里引到它们时写明「不是这一批的」。

这是同一题的第三轮，照 `.claude/rules/three-way-inference.md`「三轮分工」第三轮：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。这一轮只攻前两轮站住的形态与改法甲；这一轮里新冒出的零轮形态写进判决的交用户表、标「零轮」，不再为它开第四轮。

## 二、实现今天的样子（主 agent 的观测，2026-10-24）

- 冻结副本：开工时由主 agent 拷此刻主工作区的 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`（不带 `target`），路径在派发提示里给。
- 读到的事实：
  - `crates/singlefs-core/src/extent_tree.rs:416` 起 `ExtentUpperLeafTarget::of_a_file_without_data_units(size_of_its_inode_record_in_bytes: Option<u64>)`：`Some(_)` 交 `Some(NoDataUnit)`，`None` 交 `None`；它只看第一个文件那条 inode 记录在不在，不看 size。
  - `crates/singlefs-core/src/transaction.rs:4000` 的 `inodes_with_an_extent_upper_leaf_entry` 与第 12340 行 `build_extent_tree` 里 `(None, None)` 那一臂，都经这个函数判写不写第一个文件那一条。
  - `crates/singlefs-core/src/recovery.rs:5962` 起：冷读那一路只给「第一个文件删完」留了上段里没有第一个文件那一条的口子。
  - `crates/singlefs-core/src/transaction.rs:13584` 一带的断言：`inode_record.is_some() || data_pointers.is_empty()`。
  - 修补一实现员自报（`~/.local/share/singlefs/work/m4-validate-fix-one/result.md`「第 1 条：改法甲」）：两个调用方改前改后只在「第一个文件有记录、0 个单元、size 0」这一形上不同，今天的写路径到不了这一形，只有读进来的标签 0 文件截到 0 走得到。这一句是实现员的结论，没被攻过。
  - 条款：`.claude/kb/decisions/08-核心索引结构.md` 第 425 行起「已定项 14」，第 451 行那一句在这一批写回成「第一个文件一个数据单元都没有（size 0，或 size > 0 整个文件都是洞）时，写者重写上段叶照写这一条」；`.claude/kb/checks-owed.md` 的 C900（池级 checker 判不出第一个文件有记录而上段叶没它的条目）。
- 这一段的验证情况：修补一的合入后验证前半段绿（293 个测试二进制、2696 条测试，fmt、clippy、build）；变异轻型核心部分加点名 6 行，272 条抓到 271 条、1 条撞内存上限（不是这一批的行，别的会话补了上限后抓到）。checker 档快档两格与崩溃轻型在派腿时还没跑出结论（本机内存与卡被别的会话占着，排队中），这一轮与它们并行。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　改法甲本身：今天的写路径真到不了「有记录、0 个单元、size 0」吗
对「两个调用方只在那一形上不同、今天的写路径到不了」这一句。格里要问：新池新建文件（里程碑一那条流）、建 inode、截断意图的最后一批、删除的一批与最后一批、回退、整理换下上段叶、扫描重建之后的第一次发布，哪一条会写出「第一个文件有记录、0 个单元、size 0」；走得到的，改法甲让写出的字节变了没有，变了的那一版三处读者（冷走读、可写重挂与冷恢复、挂载态读者）与池级 checker 各怎么判。

### T2　「已定」项撑不撑得住：D8 已定项 14 写回之后与代码对不对得上
对 `.claude/kb/decisions/08-核心索引结构.md` 第 451 行新写回的那一句。格里要问：标签 0 配 size 0 在读者一侧是不是每一处都收；`with_the_tail` 把标签 0 交回标签 0 那一臂在 size 0 上是否成立；这一句与同一已定项里标签 1、2、3、4 的口径（尤其「最后一个单元之后到 size 是洞的换成标签 3、4」）在 size 0、0 个单元上会不会给出两个答案。

### T3　未定项清单的完整性：C900 与第二轮打中那一形还剩哪些来路
对 C900 的「之后第一个文件有记录而没有条目只剩写者漏写一种来路」那一句。格里要问：改法甲之后，还有哪条路径让第一个文件有记录而上段叶没有它的条目（删到一半崩溃、回退到截断意图中间那一版、扫描重建、旧版写者写出的池）；有的话，C900 要的那一条池级 checker 判据会不会误判合法镜像。

### T4　第二轮报出的数能不能核
第二轮判决第二节 T1 行引了两条腿量到的数：12 种后续里 5 种拒收、改法甲之后 5 种都消失、自带的四个标签 0 测试目标照绿。格里要问：在这一轮的冻结副本（已打改法甲）上照同一段历史重做，拒收的是不是 0 种、读出的长度与内容对不对；第二轮两条腿的起点镜像带的那一条手造泄漏（I-3.1、I-3.11、I-1.13、I-3.15 四条红）换成不带泄漏的起点之后，结论变不变。

### T5　测试与变异钉得住改法甲吗
对修补一新加的 4 条 harness 用例（`crates/singlefs-harness/tests/whole_file_hole_of_upper_leaf_tag_zero_changed_by_the_session.rs`）与 `crates/mutations.tsv` 里点名它们的变异行（「m4-validate-code-r2 T1 甲：…」与锚点改过的「m4-validate-code-r1 T5 丑：…」两行）。格里要问：把改法甲改回「size 0 取 None」以外的另一种错法（例：size 0 写标签 1、或只在 size 0 时写），这几条用例红不红；红不了的，补哪一个取样点。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1–T5 | 复核第二轮判决与改法甲够不够得着：逐格对判决原文、D8 已定项 14 与冻结副本的行，核「代码做的是不是用户定的甲」，判决的结论是不是同样打中被否的乙、丙；每条结论写文件:行号 |
| 云端攻方（Opus） | T1–T5 | 在冻结副本的拷贝上造反例：今天的写路径写出「有记录、0 个单元、size 0」而改法甲让某个读者拒收；标签 0 配 size 0 在某一处读者被拒；C900 的判据会误判的合法镜像；第二轮报的数在改法甲之后对不上。每个打中给可复现的命令与原样输出；不许重复第一、二轮已攻过的角度（两轮判决第二节逐格写着） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。量法是拷冻结副本，不在冻结副本里改。几棵内容不同的树共用编译目录之前先把 `crates/` 下的源文件时间碰新，或每棵树各用一个编译目录。
- 每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 8G 的上限，同一时刻只起一条 cargo；编出来的测试二进制也经它跑。线程上限照派发提示，用自己的编译目录，跑完删。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起），不跑 checker 档测试，不起 GPU。
