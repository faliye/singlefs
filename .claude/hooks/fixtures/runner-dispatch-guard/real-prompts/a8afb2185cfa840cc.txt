# 派发：experiment-designer

重型测试：不跑
批次：records/2026-10-22-里程碑五第二轮.md#批2
草稿目录：/home/fy5090/.local/share/singlefs/work/m5-r2-patrol-idle-prereg
认领：bash scripts/hosts/work-directory-ownership.sh claim /home/fy5090/.local/share/singlefs/work/m5-r2-patrol-idle-prereg --purpose '里程碑五一·4 巡回空转跑前登记'
被测条款：.claude/kb/decisions/27-小数据打包容器.md 的「已定项 6」节（巡回与限速）
岔路单：问题单 research/prompts/m5-knobs-questions.md
预计挂钟：2 小时内

回复与报告用中文。

仓根是 worktree `/home/fy5090/.local/share/singlefs/work/m5-dev`（分支 m5-dev），不是 `/home/fy5090/code/singlefs`。

## 关的是哪几条

问题单 `research/prompts/m5-knobs-questions.md` 的 K8 一行（巡回空转：无活可干时巡回一轮读中央映射的字节占设备读的比例，限速 X 至少三点，Y 跑前写死，低于 Y 达不到 ⇒ 改成事件触发）。写一个新实验的跑前登记。验收：

- 登记里每个量对应 K8；巡回一轮读字节按「中央映射全树条目数 × 条目宽 ÷ 每叶字节」报绝对值，条目宽与每叶字节的出处现查 kb 与 `crates/`（文件加行号）；Y 与 X 的取点在登记里写死，写明为什么取这几点。
- 必红形态写进登记：把巡回读字节记成 0 时判据必须判「不用改事件触发」，自证会红。
- 实验简称你起一个交主 agent 认；取号用 `scripts/kb/claim-experiment.sh`，但它只看本工作树：取号之前另查主工作区 `/home/fy5090/code/singlefs` 的 `.claude/kb/experiments.md`、`research/prompts/e*-preregistration.md`、`.claude/kb/experiments/` 下的文件名，再查 worktree 里 `research/prompts/` 下这一轮别的设计员刚写的 `e*-preregistration.md`，取合起来的下一个空号，报告里写明各处的最大号。

## 出口

跑前登记写成 worktree 的 `research/prompts/e<号>-preregistration.md`，报告写到 `/home/fy5090/.local/share/singlefs/work/m5-r2-patrol-idle-prereg/report.md`：取的号与各处最大号、简称、登记里每个量对 K8 的哪一格、问法有两种读法的地方。回复只写登记路径、报告路径与 sha256。