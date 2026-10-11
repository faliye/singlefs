# 崩溃点重放拆成「跑数据 / 对比数据」两段：设计轮第二轮正文（2026-10-10）

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 P11 P12 P13 P14 P15 P16 P17 P18 A1 A2 A3 A4 B1 B2 B3 C1 C2 C3 D1 D2 D3 D4 E1 E2 E3 F1 F2 G1 G2 G3 G4 J6 J7 J8 M1 M2 M3 M4 -->

## 一、题面

用户 2026-10-10 定、不在这一轮攻的，原话与出处同第一轮正文 `research/prompts/_m3-run-compare-split-r1-body.md` 第一节（调度记录 `records/2026-09-30-里程碑三开工第一轮.md` 第 994–1003 行），这里不重抄，腿去那份正文读原话。

这一轮判的：第一轮判决 `research/prompts/m3-run-compare-split-r1-main-verification.md` 第三节七条修订形态（零轮），连同那份判决第二节的逐格判对不对。

条款同第一轮：`.claude/kb/decisions/13-验证路线.md` 已定项 4、9、15；`.claude/rules/verification.md`「两档：轻型与重型」「凡用 GPU 先清场，跑完不复原」；`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「判定可复用：按输入存，按前缀剪」「用上现有的硬件加速」；合并判决 `research/prompts/m3-crash-enumeration-merge-r3-main-verification.md` 第三节 J5、J6、J7、J8。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

- 第一轮的事实表、两腿报告与判决：`~/.local/share/singlefs/work/m3-merge-impl/gpu-split-survey/report.md`（sha256 f850c23a…b151）、`research/prompts/m3-run-compare-split-r1-sonnet-output.md`、`research/prompts/m3-run-compare-split-r1-opus-output.md`（连同 `m3-run-compare-split-r1-opus-model/`）、`research/prompts/m3-run-compare-split-r1-main-verification.md`。
- 代码底子：`~/.local/share/singlefs/work/m3-merge-impl/e5/repo` 是主工作区 + V…E5 的副本（与第一轮两腿的副本同一串补丁），可只读；要改、要编就自己拷一份到腿的目录。E6（GPU 两张表抬到 2048）补丁在 `~/.local/share/singlefs/work/m3-merge-impl/e6/patch/`，E7 在做；`crates/singlefs-checker-tier/src/` 下被补丁动过的文件引副本行号，没被动过的（harness、core、RefFS、`*.wgsl`）引主工作区行号。
- 主 agent 第一轮判决时现查过的代码事实写在那份判决第一节第三条，这一轮不重抄。
- 方案按 `crates/` 今天的实现（加补丁）来谈。

## 三、被判的形态

第一轮判决第三节第 1–7 条原样（分界三列、表、版本、库与双机、GPU、簇、红步之后），连同第五节交用户表四行的候选定义。

## 四、要回答的问题

辩方腿：
1. 第一轮判决第二节每一格「坐实」「打中」「说重了」够不够得着：引的代码行说的是不是那件事；判给一方的格，另一方的说法有没有被冤枉（例：读故障那一标判正推没有出处、RefFS 判攻方说重了、随机历史 60 步封顶留零轮）。
2. 第三节每一条修订，是不是同样打中被它替掉的原形态之外的别的候选；有没有更简单、判别力不丢的形态被漏了。

攻方腿（攻击面与第一轮不重复：第一轮攻的是漏、错配、落不下、分界越位、次序与第 9 问）：
3. **修补本身**：第三节七条每一条，造一个可达的输入、改动序列或跑法，让它判错、假绿、编不出或做不成；尤其版本分族与两条自证（写表调用方都在闭包里、读 `*.wgsl` 的文件都在闭包里）能不能被绕开，历史键与给定状态表块键之间的对应会不会错配。
4. **产生结论的方法**：第一轮两腿的数（体量、CPU、展开量、闭包探针）是怎么量的、量法本身有没有让结论偏的地方（取样点、参照树、外推口径）；哪一个数翻了会让第三节哪一条翻面。
5. **没被看过的地方**：两段拆开之后，续跑（同一条命令停了再起）、双机分片、`import_from`、判红钉回快档、变异表（`crates/mutations.tsv` 与广谱变异）、`layer0_progress` 断点续跑、e161 装置在新形态下各怎么走，哪一处会断。

## 五、交付

- 报告写进 `research/prompts/m3-run-compare-split-r2-<模型>-output.md`，分段写，每次写入不超过 150 行，第一段排他新建；回复只写短句。
- 草稿与临时文件放 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件名加那份文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写文件:行号，标明是主工作区还是哪份副本。
- 每一条结论写清是读代码得出、算出来的、还是推的。
