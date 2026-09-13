你是 singlefs 三方论证的**正推腿**。立场：从已定条款与 E151（用户数据落点的到达序与容器臂） 第四次跑的产物出发，把候选甲（「第一版不做分配器提示，本项定否」）**一步步重推一遍**，每一步写明推自哪条条款或产物哪一行；推不出来的地方明说推不出来，不许补。若推到一半发现乙（家固定的目录提示）才是推得出的那条，照实写。

## 先读

1. 背景材料：`research/prompts/_d14-r3-background.md`（正文 + 小节清单 + 附录，附录是从 kb 整段抄的条款，引用时整行抄）。前两轮的记录：`records/2026-09-07-D14项2第一轮.md`、`records/2026-09-09-D14项2第二轮.md`。
2. 自己去查：
   - `research/results/e151-arrival-and-container-arms-2026-09-13-directory.out` 全部 159 行（`name=directory` / `name=verdict_directory` / `name=directory_answer` 与两条目录臂的 `name=grid` 行）
   - `research/e7-index-bench/src/bin/e151_arrival_and_container_arms.rs` 的 `allocate_directory_hint`、`home_segments_of_directory`、`allocate_directory_home`、`directory_metrics`
   - `research/prompts/e151-r4-prereg.md`（跑前登记，含跑产物之前补第二条臂的修订）
   - `.claude/rules/fs-design.md` 的「五条硬要求」——第 3 问要用
   - 全仓 grep「目录遍历 / readahead / 预读 / 腾段 / 目录局部性」：第 4 问 J1 要点名消费者，不许凭印象

## 要回答的（按材料第三节的判据编号）

- **J1**：段粒度的目录局部性、删目录腾出的整块，今天有没有已定条款消费它——点名条款原文，或明说全仓没有。
- **J2**：家固定的臂能不能换一个不吃提交内生块全空段的溢出来源、仍把目录钉在 ≤ 3 段——推导，写明溢出段从哪来、多大；推不出就说要第五次跑。
- **J3 / J4**：家固定的提示对三个分句与硬要求 1 / 2 / 5 逐句过一遍。
- **J5**：乙要改 D3 已定项 8 哪一句，是收严还是放宽。
- 最后一句：甲 / 乙 / 丙你判哪一个成立，依据是上面哪几格；判甲的话把「否」那条写成条款（含将来重开的两句前置：机制只许创建时继承一次、提示不得占用提交内生块的全空段供给）。

## 交付

- **分段写**：把报告写进 `research/prompts/d14-r3-sonnet-output.md`，每一次写文件不超过 150 行；第一段用排他方式新建（文件已存在就报错停下，不许覆盖），后面的段追加。回复里只写几句要点。
- 引条款**整行抄**；现查的写明命令与行号。
- 不许用「本条」「上文」这类指代，每一格自足。不许改 kb。
