# m3-close-b19-checker-r1 判决（第一轮，2026-10-29）

正文 `research/prompts/_m3-close-b19-checker-r1-body.md`；腿：正推 `research/prompts/m3-close-b19-checker-r1-sonnet-output.md`（模型 `research/prompts/m3-close-b19-checker-r1-sonnet-model/`）、攻方 `research/prompts/m3-close-b19-checker-r1-opus-output.md`（2cb1cebc…，模型 `research/prompts/m3-close-b19-checker-r1-opus-model/`）。

## 一、核查

- 开工快照倒推：`python3 scripts/agents/snapshot-tree.py m3-close-b19-checker-r1 --out ~/.local/share/singlefs/work/m3-close-b19-checker-r1-judge/tree`：清单 13 份，倒推出 12 份、sha256 对上 12 份；倒推不出 1 份是 `.claude/kb/checks-owed.md`（快照之后这一批改过去向），腿引它的只有 K1–K10 各笔登记行的编号与条款，主 agent 对着此刻的表逐处人工核，条款字句没变。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-checker-r1-sonnet-output.md research/prompts/m3-close-b19-checker-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-checker-r1-judge/tree --background research/prompts/_m3-close-b19-checker-r1-background.md`：「核了 26 处引文、90 处裸行号，对上 14 处，没判 102 处」，判了的 0 处对不上；没判的是裸行号与快照外的文件，判决用到的几处主 agent 现读。
- 复跑：两腿模型拷进 `~/.local/share/singlefs/work/m3-close-b19-checker-r1-judge/`，经 `run-with-memory-cap.sh` 各跑 `tools/rerun.sh --frozen <倒推的树>`：正推腿退 0、对上（`name=rerun_models …`）；攻方腿退 0、对上（`name=k8_outputs_identical scripted_pair=1 random_histories=1`）。
- 现读坐实一处：`crates/singlefs-checker/src/image.rs:650`–`672` `into_report` 里，`part_of_the_range_not_judged` 有这条不变量就整条报「不适用」，排在 `evaluated > 0`（成立）之前。攻方 K3 ① 属实：P1 第一波 C469 回绕那一支若走它，会把 ①② 已判成立的整条压成不适用。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| K1 C871 | **分歧，进第二轮**：两腿都认「只改偏移 0、只红 I-9.8」的镜像造不出；正推腿判区间内换号由 I-9.10、I-9.5 接，攻方判「inode 30 改名 35」那张镜像没有不变量红。分辨点是那张镜像上 I-9.10、I-9.5 红不红 | 两腿 K1 |
| K2 C740 | **分歧，进第二轮**：正推腿 I-3.19 三支；攻方判挂载时 T 槽一时读坏的合法历史与干净历史镜像可同而 W 不同 | 两腿 K2 |
| K3 C469 ② | **① 采攻方**：回绕那一支不走 `part_of_the_range_not_judged`，并进下一批改 P1 第一波的补丁。**② 分歧，进第二轮** | 主 agent 现读 `image.rs` `into_report`；两腿 K3 |
| K4 C706 | **分歧，进第二轮**：正推腿 I-5.7 前件取记录点名项；攻方判前件循环 | 两腿 K4 |
| K5 C685 | **采正推腿**：不重钉四个数，样一的原论证在 txg 有洞时不成立（模型 1 120 960 个世界、两式不同 19 707 对），LR5 等价测试随 RCW 落地退役；攻方没攻，进第二轮再攻 | 正推腿模型复跑对上 |
| K6 C697 | **方向采 RCW、条文进第二轮**：正推腿模型旧等式 105 812 个合法世界误红、RCW 在 12 681 102 对上 0 误红；攻方指新流会让 I-8.7 整条不适用、RCW 两处细节没答 | 两腿 K6 |
| K7 C699 / C698 | **采「实例表清扫打破前提」**（两腿一致，攻方 `instance_table_sweep.rs`、正推腿 `InstanceTablePlan::Rewrite(` 两处构造）；正推腿的第四条件零轮，进第二轮攻 | 两腿 K7 |
| K8 C717 | **采「只看根环做不出」**（两腿一致；攻方模型 S=4 合法与坏写者根环逐条相同、随机 20 000 段合法历史中 C717 条件命中 856 段，复跑对上）。C717 字面要求在根环上自相矛盾，交用户定走乙（盲带报部分没判、改 C717 字面）还是丁（借 journal 新根段判红、另开一批）；第二轮先给两条路的代价数 | 两腿 K8；攻方模型复跑 |
| K9 C764 | **采「只比实例代号更小的」**（攻方 ① 的担心由正推腿 I-8.14 的写法消掉）；读不出那一形（攻方候选丁、正推腿报部分没判）进第二轮 | 两腿 K9 |
| K10 C70 | **分歧，进第二轮**：正推腿 I-3.20 四项写射程外；攻方判可用(d) 不落盘、checkpoint 保留池与环窗口能从镜像现算 | 两腿 K10 |

## 三、交用户的

C717 走乙还是丁，等第二轮给出两条路的代价数之后弹窗。

## 四、被攻过零轮的

正推腿的 I-3.19、I-5.7、I-8.14、I-3.20 条文、RCW 三条、C699 第四条件；攻方的集合形收窄与候选丁——都进第二轮。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D28（挂载期承诺量） 已定项 1 | 备料 | 2026-10-29 不受影响：K10 只判 C70 的条文落点，式子本身没动 |
