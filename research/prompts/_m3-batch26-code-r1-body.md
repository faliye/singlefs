# 删除批第 26 批合入的代码：代码轮第一轮正文（2026-10-06）

<!-- doc-lint:not-numbers H1 H2 H3 K1 K2 K3 F1 -->

## 一、题面

删除批第 26 批一份补丁 2026-10-06 合进主工作区（`records/2026-10-01-删除与并发事务.md` 2026-10-06 那几条）：h 关 m3-batch24-code-r1 判决第三节第 1、2 条——读孩子类标签时头里的类等于类型段那一类也先按那一类长度验整单元校验和、验不过看下一份（CPU 与 GPU 同改）；inode 树读不出孩子时 I-9.2 的判法与容器树对齐（CPU 与 GPU 同改）。这一轮判两件事：代码做的是不是条款与判决说的；哪一格会错。

改动全文在 `research/prompts/_m3-batch26-code-r1-diff.md`。冻结副本 `/tmp/claude-1000/m3-batch26-code-r1/tree/`，被判的 5 份文件的 sha256 在 `research/prompts/m3-batch26-code-r1-snapshot/crates-sha256.txt`。实现员报告（结论句只当线索）：`/tmp/claude-1000/m3-batch26/h/report.md`。

| # | 改动 | 主要落点（冻结副本） | 压着的条款或判决 |
|---|---|---|---|
| H1 | CPU：`child_unit_class_other_than_the_entry_type_names` 每条位置条目先按头里那一类的长度自证，自证不过看下一份；自证过的是类型段那一类才交回 None | `crates/singlefs-checker/src/walk.rs` 第 4108 行起 | `.claude/kb/invariants.md` I-9.2、I-9.17、I-9.23；m3-batch24-code-r1 判决第二节 K2 N1 两故障 H1、第三节第 1 条 |
| H2 | GPU：`w_child_unit_class_other_than_the_entry_type_names` 同改；对拍补一形（inode 根条目类型段 0、叶容器盘 0 那一份陈旧） | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 2960 行；`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` | 同上；用户 2026-10-03 定 GPU 全覆盖 |
| H3 | inode 树读不出孩子：CPU 新函数 `judge_an_inode_entry_whose_child_does_not_read_back`（第 4142 行，调用在第 2535 行）照容器树判 I-9.2；GPU `w_walk_inode_root` 读不出那一支同判 | `walk.rs`、走读内核 | I-9.2；m3-batch24-code-r1 判决第二节 K1 那一行、第三节第 2 条 |
| 测试 | 坏镜像 6 条（H1 四形两种盘序、读不出孩子两形）；变异表追加 7 行 | `crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/mutations.tsv` | 同上 |

**这一轮不判的**（各有去处，腿不攻）：
- m3-batch25-code-r1 判决第三节（H-c GPU 256 片上限、GPU 关门那一侧对拍与变异、F-N）：第 27 批。
- 实现员报告「交主 agent」第 1、2 条（inode 树类型段不认得、孩子读不出时照旧按 16 KiB 读孩子；读不出孩子那一支 GPU 没有对拍形）：已排第 27 批。
- GPU 对拍与 GPU 那一行变异：提交时由主 agent 跑。
- 实例表走读（m3-batch25-code-r1 判决 H-b）：归根环线。

## 二、实现今天的样子（主 agent 的观测，2026-10-06）

- 合入验证 `merge-and-verify.sh`「✓ 测试全绿（170 个测试二进制）」（`/tmp/claude-1000/merge-and-verify/2026-10-06-2215960/`）；fmt（`cargo fmt --all -- --check`）与 `cargo clippy --offline --all-targets --all-features -- -D warnings` 经内存包装单跑都退 0。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　条款对实现（H1–H3）
- H1、H2：CPU 与 GPU 自证的长度、位置条目的次序、两份都自证不过时交回什么是不是同一个；第 24 批 K 那两行变异（`第24批 K I-9.2 同形…`、`第24批 K 孩子的类不验整单元校验和…`）在新写法上还红不红（读码）。
- H3：类型段 0 / 2 / 不认得三支是不是与容器树读不出孩子时的判法逐支相同（`walk_the_entries_of_a_container_tree_node` 那一支）；GPU 同判。
- 测试：每份坏镜像是不是「= 只有陈旧副本那张对照红的 ∪ 目标、不连带 I-7.2、两种盘序相同」；7 行变异点名的测试与参数筛不筛得到。

### K2　哪一格会错（重点 H1、H3）
在冻结副本的拷贝上攻，只跑 harness 档（GPU 不跑，GPU 那一侧读 wgsl、用 CPU 算期望）：
- 找一形新写法让结论翻面而原写法不翻：两份副本都是类型段那一类、一份自证不过；两份各是不同类、各自自证过或不过；陈旧副本在盘 0 / 盘 1；墓碑树、谱系树、inode 树、容器树各层级。
- H3：读不出孩子的条目在不同层级、根长高前后、回退之后各判什么；有没有一形 I-9.2 误红或该红不红。
- CPU 与 GPU 判得不一样的一形。
- 上一轮「没打中（一次抽样）」的第二次抽样：m3-batch25-code-r1 的 K2「走到末片而没滤全」、N2 GPU 其余各支、合法历史。

### K3　被判的文件名单
`research/prompts/m3-batch26-code-r1-snapshot/crates-sha256.txt` 里那 5 份；冻结副本里每一份都要有一格判到。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K3 | 逐条从条款、判决格推到代码行、测试与变异行 |
| 云端攻方（Opus） | K2，重点 H1、H3 | 在冻结副本的拷贝上造反例，只跑 harness 档；用户决定的那几步放开扫 |

## 五、跑前写死的判据

- 两条腿都没打中：判站住。这只是一次抽样，「没打中」照 `.claude/rules/three-way-inference.md` 再抽一次才记。
- 打中：主 agent 现查坐实，改法进下一批；改法算零轮。
- GPU 那一侧：读代码找到一支与 CPU 不同、并用 CPU 算出期望写清 GPU 会给什么，算打中；GPU 不跑。
- 误红、漏判、漏红的口径同 m3-batch22-code-r1 正文第五节。

交付：
- 腿的报告写进 `research/prompts/m3-batch26-code-r1-<腿名>-output.md`，分段落盘：每次写进文件不超过 150 行，第一段排他新建，回复只写短句；早点开写，别等全部攻完才写。
- 模型与脚本放 `research/prompts/m3-batch26-code-r1-<腿名>-model/`，`rerun.sh` 交回前在空目录里从头跑通一次；仓路径从环境变量 `SINGLEFS_REPO` 取，包装脚本另给一个变量时在文件头写明变量名；编译与跑都经 `scripts/hosts/run-with-memory-cap.sh`，上限显式给（编译 6G、跑 4G），不用 `auto`；`compare` 比之前排序、自己不依赖 `SINGLEFS_REPO`；用自己的编译目录。草稿放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件自己的行号；引代码写冻结副本里的全路径:行号。
- 方案按 `crates/` 今天的实现来谈。
