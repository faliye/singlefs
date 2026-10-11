# 崩溃点重放拆成「跑数据 / 对比数据」两段：设计轮第三轮（最终轮）正文（2026-10-10）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 P11 P12 P13 P14 P15 P16 P17 P18 R1 R2 R3 R4 R5 R6 R7 R8 R9 R10 N1 N2 N3 N4 Q1 Q2 Q3 Q4 Q5 S1 S2 S3 S4 S5 S6 S7 J6 J7 J8 M1 M2 M3 M4 M5 -->

## 一、题面

用户 2026-10-10 定、不在这一轮攻的，原话与出处同第一轮正文 `research/prompts/_m3-run-compare-split-r1-body.md` 第一节（调度记录 `records/2026-09-30-里程碑三开工第一轮.md` 第 994–1003 行），腿去那份正文读原话。

这一轮是最终轮。判的：第二轮判决 `research/prompts/m3-run-compare-split-r2-main-verification.md` 第三节七条修订形态（零轮），第一、二轮判决第二节里「坐实」的各格，连同第二轮判决第五节交用户表五行。这一轮之后不再开新轮；这一轮新冒出来的零轮形态写进交用户表、标「零轮」。

条款同第一轮：`.claude/kb/decisions/13-验证路线.md` 已定项 4、9、15；`.claude/rules/verification.md`「两档：轻型与重型」「凡用 GPU 先清场，跑完不复原」；`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」「用上现有的硬件加速」「全量的红，钉回快档」；合并判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J5、J6、J7、J8。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

- 前两轮的材料：事实表 `~/.local/share/singlefs/work/m3-merge-impl/gpu-split-survey/report.md`（sha256 f850c23a…b151）；第一轮两腿报告与判决 `research/prompts/m3-run-compare-split-r1-*`；第二轮两腿报告（连同 `m3-run-compare-split-r2-opus-model/`）与判决 `research/prompts/m3-run-compare-split-r2-*`。
- 代码底子：`~/.local/share/singlefs/work/m3-merge-impl/e5/repo` 是主工作区 + V…E5，只读；`~/.local/share/singlefs/work/m3-merge-impl/e7/repo` 是主工作区 + V…E7（不带 .git），只读，主 agent 正在里面跑验证，不在里面编。要改要编就拷到腿自己的目录。
- 这一轮之后才有的事实（主 agent 现查过）：
  - E6 之后 GPU 判器在 J4 与两条边界流上先撞 journal 记录表 `MAX_RECORDS` 512（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:142`）。调查报告在 `~/.local/share/singlefs/work/m3-merge-impl/j4-after-e6/report.md`，E9 在改。
  - 判不了的状态在流水线里退回 CPU 判（E5 副本 `crash_amplification.rs:9148`）。
- 方案按 `crates/` 今天的实现（加补丁）来谈。

## 三、被判的形态

第二轮判决第三节第 1–7 条原样，连同那份判决第五节交用户表五行的候选定义。

## 四、要回答的问题（第三轮的攻击面：与前两轮都不重复）

辩方腿：
1. 第二轮判决第二节每一格「坐实」「采」「不采」够不够得着；判给攻方的格里，有没有一格换一个取样点就不成立（例：R8 四成块换主用的是三组权重、R7「第 m 步多半判绿」是推的、R10 可达性没造出）。
2. 第二轮第三节每一条，是不是同样打中被它替掉的形态之外的别的候选；交用户表五行的候选是否列全、每个候选的定义是否写得出一句可判的话。

攻方腿：
3. **「已定」项撑不撑得住**：两轮判决第二节里「坐实」的各格与第二轮第三节七条，挑最脆弱的几条，在今天的代码上造可达的输入或改动序列打穿；尤其是第 2 条的「键按内容取」能不能罩全（还有什么输入不在那份清单里）、第 3 条叶子文件与「族闭包里不许有 WGSL 与判器入口」那条检查做不做得出来、第 6 条按项多重集验收的漏处。
4. **未定项清单的完整性**：两轮判决与四份腿报告里提过、却没进任何一处（第三节、交用户表、零轮与欠账）的问题，逐条找出来。
5. **报出的数能不能核**：两轮报出的数（体量 389.8 GiB 与 598.9 GB、CPU 116–525 线程时、展开量 62660/387189、块换主 40–43%、写表过 2048 在 71–136 步、层 0 片与块对不齐），逐个写清来路是读代码、算的、量的还是推的，哪一个只在腿的模型上有、入库装置上重做要什么。

## 五、交付

- 报告写进 `research/prompts/m3-run-compare-split-r3-<模型>-output.md`，分段写，每次写入不超过 150 行，第一段排他新建；回复只写短句。
- 草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件名加那份文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写文件:行号，标明是主工作区还是哪份副本。
- 每一条结论写清是读代码得出、算出来的、还是推的。
