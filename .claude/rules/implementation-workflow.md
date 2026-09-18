# 实现改动的流程：写代码 → 三方对抗 → checker，提交前跑 herd7 与 QEMU

**这是 singlefs 的项目本地规则**，不在共享 SOP 里：它压在本机的三方论证（`.claude/rules/three-way-inference.md`）与
本工程接管的 herd7 / QEMU 装置上，别的项目没有这两样。共享规则在 `.claude/singlefs-ai-sop/rules/`。

## 三步，缺一步就不算做完

| 步 | 做什么 | 谁在判 |
|---|---|---|
| 1 写代码 | `crates/` 下的改动带测试，每条新测试先证明会红（`.claude/singlefs-ai-sop/rules/show-me-test.md`） | show-me-test 阶段判「有没有测试」；会不会红写在 commit message 里 |
| 2 多 agent 正反对抗推理 | 改完的代码走一轮三方：材料带上 diff 与它压着的条款，正推腿核「代码做的是不是条款说的」、反推腿攻「哪一格会错」、本地腿找反例；打中的写回代码，再攻一轮 | 门禁 56 号判形式：这次改动里每个 `crates/*/src/*.rs` 都要在同一次改动带来的 `research/prompts/*-main-verification.md` 里被按路径点名。点名了判得对不对，要人看 |
| 3 checker 测试 | 池级 checker、层 0 崩溃点重放、变异表——三方打中的每一条先做成会红的量再改；每一处改法在 `crates/mutations.tsv` 留一条「改回去它就红」的变异 | 54 号（层 0 全量）、59 号（crates 变异表复跑）、33 号（research 的变异表）、`check.sh` |

**次序不能倒**：三方攻的是写好的代码与它的测试，攻在计划上的那一轮（里程碑那份）替代不了这一轮。

## 改 agent 定义与共用约束，走同一条三步

**定义是工作流，不是说明**（用户 2026-09-18 定）：`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md` 只写怎么做——步骤、约束、判据、交付。为什么与是什么归 kb 自己管（`.claude/kb/`、`records/`），定义里不写、也不为它留解释的尾巴；要用到那些东西时，写成一条指令：「开工先读 `.claude/kb/<某份>` 的〈某节〉」。门禁 71 号判这一条。

**改它们与改 `crates/` 同规矩**：写完要走一轮三方，判决落 `research/prompts/<轮>-main-verification.md` 并按路径点名改过的每一份定义，门禁 72 号判形式（形态照 56 号）。

**为什么**：定义是所有 subagent 每次开工都要读的东西，混进说明会让它越写越长（起步上下文直接变贵），而且改一句的影响面与改一行代码一样宽——2026-09-18 一轮里 16 份定义里有 36 行是在定义里记经过，没有任何检查拦过。数与经过在 `records/2026-09-16-subagent拆分提案.md`。

## 代码轮派腿之前记一份开工快照

派三方的腿之前，把这一轮被判的文件与材料点名的 kb 文件记一份 `sha256sum` 快照（放这一轮的材料目录），交核查员当输入；腿跑着的时候主 agent 不改这些文件。

**为什么**：腿引的行号是它读那一刻的，主 agent 轮内改一行，核查员就分不清「腿引错了」还是「文件后来变了」。实测（2026-09-18，`m2-wave1-code-r1`）：主 agent 在两条腿交回之间改了 `.claude/kb/milestone/02-second-txn.md`（正是正推腿打中的那句），核查员靠 mtime 与 `git diff` 逐处比对才判出「改动是同行数原地替换、只碰一处」，它自己在报告里写「本轮靠 mtime + git diff 补救纯属运气」。要改的等判决时一起改。

## 提交前必跑 herd7 与 QEMU

上游 singlefs-ai-sop 2026-09-16 起不再管 herd7 / LKMM 与 QEMU（`.claude/handover/qemu-herd7/README.md`），两样都是本工程自己的阶段：

| 装置 | 阶段 | 判什么 |
|---|---|---|
| herd7 / LKMM | `.claude/gate.d/57-lkmm.sh`（逻辑在 `.claude/scripts/lkmm.sh`） | `litmus/` 下每条 Never 有对照组、绑到代码，herd7 判定与声明相符；缺 herd7 直接红，不静默跳过 |
| QEMU 真设备 | `.claude/gate.d/55-qemu-first-transaction.sh` | 两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比 |

两道都在 `gate.sh` 里，提交前跑全量门禁就把它们带上了；单跑一道不算跑过门禁。
`--staged` 那条路（几个会话共写一个仓时）同样跑它们。

## 为什么做成门禁而不是提醒句

2026-09-16 发布 B（覆盖写 + 释放）写完、单测绿、check 绿，若没有人拦，下一步就是提交——而三方一轮都没攻过那段代码。
写一句「记得走三方」拦不住这件事，56 号那道拒绝拦得住（`.claude/singlefs-ai-sop/rules/sop-first.md`）。
