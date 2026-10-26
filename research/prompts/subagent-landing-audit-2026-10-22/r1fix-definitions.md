# landing-audit-r1 定义文字修补（K3-1 定义一侧、K3-2 到 K3-9）

批次：records/2026-10-22-子agent结果落地审计.md#5　　日期：2026-10-22

## 改了什么（九份，前后 diff 行数）

`git diff --stat`（工作树对暂存区，这一轮只有我改这九份）原样：

```
 .claude/agent-common.md                 |  4 ++--
 .claude/agents/experiment-designer.md   |  2 +-
 .claude/agents/experiment-runner.md     | 10 +++++-----
 .claude/agents/implementation-writer.md |  2 +-
 .claude/agents/sweep.md                 |  2 +-
 .claude/agents/three-way-attack.md      |  2 +-
 .claude/agents/three-way-defense.md     |  2 +-
 .claude/agents/tooling-writer.md        |  2 +-
 .claude/main-agent.md                   |  8 ++++----
 9 files changed, 17 insertions(+), 17 deletions(-)
```

开工前的原样存在 `before/`（九份），diff 用它比。

| 条 | 文件 | 改了什么 |
|---|---|---|
| K3-2 | `.claude/main-agent.md` 「派发提示怎么写」 | 「文件名一律写 result.md」改成：文件名里不带 report；没有别的形态要求的写 `<草稿目录>/result.md`，三方腿照 three-way-inference 写 `research/prompts/<轮>-*-output.md` |
| K3-2 | `.claude/agent-common.md` 「报告」节 | 「报告文件名一律 result.md」同改：名字里不带 report；没有别的形态要求的写 `result.md`；三方腿的报告名是 `research/prompts/<轮>-*-output.md` |
| K3-3 | `tooling-writer.md` 第 7 步 | 「交回前不跑门禁阶段与各类 lint」后加「（命名检查除外，共用约束「门禁」一节）」 |
| K3-3 | `experiment-runner.md` 第 6 步与「没做什么」 | 第 6 步同加「命名检查除外」；固定句「没跑门禁阶段」改成「没跑命名检查之外的门禁阶段」 |
| K3-3 | `implementation-writer.md` 第 4 步 | 「门禁阶段（…`--quick` 除外）」改成「…`--quick`、共用约束「门禁」一节的命名检查除外」（它没有「没做什么」里的同句，只这一处有同类句子） |
| K3-4 | `agent-common.md` 「命名检查交回前跑」 | 在主工作区（写仓根）或 git worktree 建的副本（写副本根）上跑；rsync 拷的副本里改的不在副本上跑，两条命令（`<根>` 填仓根）写进报告「要主 agent 跑的命令」一节，主 agent 在补丁打进主工作区之后跑；「贴进报告」改成「跑了的，…贴进报告」 |
| K3-5 | `main-agent.md` 第 2 条（两处）、第 7 条 | 登记位加第四处：子 agent 与协作工具的事记进 `records/2026-09-16-subagent拆分提案.md` 的编号小节（CLAUDE.md「经验与约定写在哪」）；第 2 条前半句「记进该记的地方」与第 7 条的列表同加「拆分提案的编号小节」 |
| K3-6 | `experiment-runner.md` 第 5 步与「写范围」 | 第 5 步加「重跑登记有比对对象表的，实现单元时往 `.claude/gate.d/replay-comparison-units.tsv` 每个比对对象登一行，列义见那份表的文件头」；写范围补「`.claude/gate.d/replay-comparison-units.tsv` 里这个实验的比对对象行」。写范围表（`agent-write-scope.tsv`）那一行已有，没动 |
| K3-7 | `experiment-designer.md` 第 3a 步 | ② 改成每个判据要分辨的差取岔路单那一行的翻面观测，写测量手段的分辨率分不分得出它，不写被测臂上预期产生多大的差；③ 只对触发是前提、不是被问的量的事件做，算出的结论列进第 2 步说的「跑之前已经存在的数」那一节；① 可跑性与「三样里算不出的」那一句没动 |
| K3-8 | `experiment-runner.md` 第 3c 步 | 加「入库装置那一支（`crates/singlefs-checker-tier/src/bin/`）的证红写进报告「要主 agent 跑的命令」一节，清单那一格写「待主 agent 证红」」 |
| K3-9 | `sweep.md` 「产出」第一条 | 「并进下一批交 implementation-writer」改成：记进下一轮的清单，钉值类的行照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 4 条先定条款再派 |
| K3-9 | `main-agent.md` 派发表「一批阶段任务结束」行 | 同改：「进下一轮的清单，钉值类的行照「一轮怎么开、怎么收」第 4 条先定条款再派 `implementation-writer`，不交写回员」 |
| K3-1 定义一侧 | `three-way-attack.md`、`three-way-defense.md` frontmatter | `required-inputs` 末尾补「报告」。「输入」一节两份本来就写了报告路径形态（攻方 `research/prompts/<轮>-opus-output.md`、辩方 `research/prompts/<轮>-sonnet-output.md`），没动 |

## 红绿输入（每条先造一个该红的输入）

第 3 步（`.claude/rules/implementation-workflow.md`「改 agent 定义与共用约束，走同一条三步」）：先在模型里做成必须报非 0 的世界，再改定义。模型是 `check-defs.py`（草稿目录里，43 项「必须在」「必须不在」的原文片段，按条标 K3-x），对一个根判，全绿 0、有红 1。

- 改之前（对 `before/` 九份）：`红：40 处不合（查了 43 项）`，K3-1 到 K3-9 每一条都有红。
- 改之后（对仓根）：`绿：0 处不合（查了 43 项）`。
- 第一次对仓根跑出过 2 处红（K3-6 写范围、K3-8 的片段），是检查串与实际写法不一致（我写检查时的串写成了别的字），改检查串对齐实际文字后，改前仍红、改后绿；这两项的检查串是事后对齐的，不是先于改动钉的，读数时按这个打折。

### 现有门禁或钩子会不会红，逐条

| 条 | 现有的会不会红 | 依据 | 要立的检查 |
|---|---|---|---|
| K3-1 定义一侧 | 会红，已量 | 探针提示 `probe/attack-no-report.txt`、`probe/defense-no-report.txt`（没有「报告」二字）；改前的定义放在仓副本里喂 `dispatch-prompt.py --check`，两份都「✓ 派发闸照这份提示放行」；改后仓根上攻方「派发闸照这份提示会拒 1 处（定义要的输入）」、辩方「✗ 派 three-way-defense 的提示缺它定义要的输入：报告（.claude/agents/three-way-defense.md 的 required-inputs）」；带「报告：…」的两份改前改后都放行 | 无 |
| K3-2 | 不会红 | 派发闸 ⑰ 只判「报告：」一行名字里带 report，不比定义与 three-way-inference 的报告名 | 要立的检查：三方腿提示的「报告：」一行不是 `research/prompts/*-output.md` 形态时，派发闸拒（第 18 号任务 K2-4/K3-1 的那处合并判法里做；定义文字之间的矛盾本身没有机读判法） |
| K3-3 | 不会红 | 没有门禁比共用约束点名的类型（`agent-common.md`「命名检查交回前跑」）与各类型定义「交回前不跑门禁阶段」那一句 | 要立的检查：code-tooling 的 agent-write-scope 一类格里加一条，共用约束「命名检查交回前跑」括号里点名的每个类型，其定义里「门禁阶段交回前不跑」那一句必须带「命名检查除外」 |
| K3-4 | 不会红 | 没有判「在哪棵树上跑」的东西；命名四格在最小副本上只会自己红 | 要立的检查：交回闸判报告里贴的两条命令输出的根是仓根或 worktree；已记在判决三（九十节正推 3，「命名检查说跑了没真跑」），不另立 |
| K3-5 | 不会红 | 延后项去向没有机读判法 | 要立的检查：九十节 ③ 的「延后项带编号的格」，按四处认（里程碑收口表、checks-owed、决策正文、拆分提案编号小节） |
| K3-6 | 不会红 | code-tooling 的 agent-write-scope 格只判到类型名一级（`code-tooling.sh` 该格的 ③④） | 要立的检查：写范围表某行出处写了「定义的写范围由…补」的，对应定义「写范围」一节必须点名那条路径（表的路径逐条在定义里出现） |
| K3-7 | 不会红 | 设计员定义里的步骤与 test-discipline 的冲突没有机读判法 | 无机读判法，不立 |
| K3-8 | 不会红 | 定义步骤之间的衔接没有机读判法 | 无机读判法，不立 |
| K3-9 | 不会红 | 同上 | 无机读判法，不立；钉值类的行先定条款是第 4 条的人工判 |

## 改了 frontmatter 的：派发闸读 `required-inputs` 的那一处

读它的是 `.claude/hooks/runner-dispatch-guard.sh` 的 `missing_required_inputs`（该文件 630 行起，`grep -n 'def missing_required_inputs'` 现查）：按逗号分组，组内用 `|` 分同义词，提示里出现一个同义词的原文（子串）就算有。「报告」是子串匹配，不要求「报告：」成行。

- 会拒的现有写法：攻方、辩方的提示里一个「报告」字都没有的（探针 `probe/*-no-report.txt` 就是这种，改后攻方 1 处、辩方 1 处）。
- 不会拒：写了「报告：…」「报告路径：…」或正文任何处带「报告」二字的；骨架 `python3 scripts/agents/dispatch-prompt.py three-way-attack` 现在会多出一行「报告：<草稿目录>/result.md」，它能过派发闸（`probe/attack-with-report.txt` 用 research/prompts 形态的路径：放行）。
- 骨架默认值 `<草稿目录>/result.md` 对三方腿是错的形态（交回闸只认 `research/prompts/<轮>-…-output.md`），改在 `scripts/agents/dispatch-prompt.py`，归第 18 号任务；这一条我没碰。`dispatch-prompt.py --selftest` 现在跑过：`✓ 自检通过（查了 78 项）`（light，只读定义，我没改它的文件）。

## 读数与限度

- 只改文字，没有脚本改动，所以没有新增 `# gate-similar:` / `# hook-events:` 行；语法检查对 md 不适用。
- K3-3 里 `implementation-writer.md` 的「没做什么」没有「没跑门禁阶段」同句（`grep` 只在第 4 步一处命中门禁阶段不跑），所以只改了那一处。
- K3-4 的写法把「git worktree 建的副本」当作四格判得到的树，这是攻方腿 K3-4 里「推的，没建 worktree 量」的那一半，我也没建 worktree 量。
- K3-6 的 stage-owners 归属行与「补上缺的 6 个单元」归第 18 号之外的门禁份，没碰。

## 改过的定义、共用约束与规则清单（交主 agent 开定义三方）

`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/sweep.md`、`.claude/agents/tooling-writer.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/three-way-defense.md`。规则文件没改。

## 没做什么

- 没走定义三方，没提交，没跑门禁阶段与 lint（rules-lint、doc-lint 等），没跑重型测试。
- 没碰 `.claude/gate.d/`、`scripts/kb/`、`.claude/hooks/`、`scripts/agents/`。
- 命名检查不适用（没改 `.rs`）。
- 草稿目录里留下 `before/`（九份原样）、`check-defs.py`、`probe/`（四份探针提示）；探针用过的 12M 仓副本已删。
- 推翻条件：改后用 `check-defs.py` 对仓根跑出红，或有人证明攻方/辩方的合法派发提示一个「报告」字都没有。
