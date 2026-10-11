# 删除批第 27 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers J1 J2 J3 J4 K1 K2 K3 -->

## 一、题面

删除批第 27 批一份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：j 关 m3-batch25-code-r1 判决第三节第 1、2 条与第 26 批实现员报告「交主 agent」第 1、2 条。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch27-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch27-code-r1/tree/`，被判的 5 份文件的 sha256 在 `research/prompts/m3-batch27-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`/tmp/claude-1000/m3-batch27/j/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| J1 | GPU 实例表链上限做成命名常量 `INSTANCE_TABLE_CHAIN_MOST_PAGES_WALKED`，走满还没完记判不了（`UNDECIDABLE_INSTANCE_TABLE_CHAIN_LONGER_THAN_THE_PAGES_WALKED`）；对拍补 256 / 257 片两条 | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 1580–1589 行起；对拍用例第 7464–7526 行一带 | `.claude/kb/invariants.md` I-9.29「最新根的实例表读不出、被崩溃恢复抛弃的根滤不掉时记判不了」；m3-batch25-code-r1 判决第二节 H-c、第三节第 1 条 |
| J2 | GPU 关门那一侧对拍补「链指针记录不合法」「第 1 片读不出」两条，`chain == 0u` 那一处配变异 | 对拍用例第 7442、7453 行一带 | 同上；m3-batch25-code-r1 判决第二节 K1 那一行、第三节第 2 条 |
| J3 | inode 树类型段不认得时不读孩子（CPU 新枚举 `RegisteredInodeInternalEntryType`，按层级登记引用：层级 1 两槽、≥ 2 一槽，只判 I-9.2 红），GPU 同改；坏镜像改钉「= 对照 ∪ 类标签自己带的 I-1.11」 | `crates/singlefs-checker/src/walk.rs` 第 141 行、第 2534 行一带；走读内核 `w_walk_inode_root`（第 2596 行起）；`crates/singlefs-harness/tests/checker_known_bad_images.rs` | `invariants.md` I-9.2「类型段 ∈ {0, 2} … 其余判该条目损坏」；第 26 批实现员报告「交主 agent」第 1 条（与墓碑树、谱系树类型段不认得那一支对齐） |
| J4 | GPU 读不出孩子时 I-9.2 补两条对拍流（不重封、只补校验和的改法）与变异 | 对拍用例第 7317–7375 行一带 | I-9.2；第 26 批实现员报告「交主 agent」第 2 条 |
| 测试 | 变异表追加 6 行、替换第 26 批 2 行（锚点随 J3 改没） | `crates/mutations.tsv` | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch26-code-r1 判决第三节（H-B 读不出孩子时不看孩子就判得了的小句不判、H-V 两份副本各自自证成不同类、测试尖度三条）：第 28 批。
- GPU 对拍与 GPU 那几行变异：提交时由主 agent 跑。
- 扫描内核展开大小（实现员草稿工具量到 700 575 个表达式，口径没核）：提交时库单测 `every_kernel_stays_within_the_inlined_size_budget` 判。
- 实例表走读「走过」的键（m3-batch25-code-r1 判决 H-b）：归根环线。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-06-2641815/`）；fmt（`cargo fmt --all -- --check`）与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0；`crash_enumeration_sharded_across_processes` 整份「10 passed」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（J1–J4）
- J1：走满上限的判法与 CPU（CPU 没有页数上限）在 256 片、257 片两点上是不是「判了的同判、判不了的回落 CPU 后同判」；新站点号与已有的不撞。
- J2：两条关门流是不是真走到关门那一支（门误开时对拍会红）；`chain == 0u` 那行变异是不是由点名的流抓。
- J3：类型段不认得那一支是不是与墓碑树、谱系树类型段不认得那一支（`walk_the_entries_of_a_container_tree_node`）逐项相同：读不读孩子、登记几槽、判什么、记不记走读失败；GPU 同判。
- J4、测试：每份坏镜像与对拍流的期望写没写全；变异行点名的测试与参数筛不筛得到；替换的两行是不是只换锚点。

### K2　哪一格会错（重点 J1、J3）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- J3：类型段不认得时按层级登记的槽数与孩子真占的槽数不符的形（孩子是码 3 叶容器却在层级 ≥ 2 下、孩子是码 2 节点却在层级 1 下），会不会让 I-3.1 一族误红或漏红；与原来先读孩子的写法比，有没有一形漏判了原来判得出的。
- J1：链长恰在上限附近（255、256、257 片）、链在中途断开、链里有重复片时，GPU 判不判、判什么，与 CPU 比。
- CPU 与 GPU 判得不一样的一形。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch26-code-r1 的 K2 H1「新写法翻而原写法不翻」、H3 误红、CPU 与 GPU。

### K3　被判的文件名单
`research/prompts/m3-batch27-code-r1-snapshot/crates-sha256.txt` 里那 5 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 J1、J3 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch27-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch27-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比之前排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
