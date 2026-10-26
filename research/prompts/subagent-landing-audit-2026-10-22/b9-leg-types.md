# 第九批 landing-audit-b9-leg-types：三方腿类型表收成一处、派发闸 ⑰ 加判三个报告名前缀、登记表一行理由（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#9；依据：拆分提案 `records/2026-09-16-subagent拆分提案.md` 第九十节 ⑯、㉑ 两行，㉑ 的实测 `research/prompts/subagent-landing-audit-2026-10-22/b9-prefix-probe.md`。

## 结论

三条的验收标准都达成：

| 条 | 验收标准 | 读数 |
|---|---|---|
| ⑯ | 类型表只在一处登记 | 登记处是三份定义 frontmatter 里的一行 `leg: three-way`（三份都在第 9 行）；读法只在新库 `.claude/hooks/lib_three_way_legs.py` 的 `three_way_leg_types`（第 40 行） |
| ⑯ | 四处都改读它 | 派发闸 `three_way_report_types`（第 1451 行）、交回闸 `citation_checked_agents`（第 171 行）、骨架 `three_way_report_types`（第 130 行）、账表 `three_way_leg_types_of`（第 1053 行）都调那份库；四处原来的类型名常量只剩弄坏开关下用的那一份 |
| ⑯ | 四处自证各加一格，读到的集合与登记处逐个相同 | 四份自证都加了「三方腿类型表」一格（派发闸 4 项、交回闸 4 项、骨架 2 项、账表 1 项），样本仓与本仓两份登记都核；改前探针：样本仓上四处 0 处相同，本仓上账表多读 three-way-materials、three-way-verifier；改后四处都相同 |
| ⑯ | 弄坏开关红在点名的那一格 | 新开关 4 个（派发闸、交回闸、骨架 `leg-types-hardcoded`，账表 `leg-types-by-prefix`），打开后失败行都只落在「三方腿类型表」那一格（原样见第 3 节） |
| ㉑ | ⑰ 对落在本提示草稿目录下、以 SUMMARY、FINDINGS、ANALYSIS 开头的 `.md` 报告名同样拒，拒绝说明给出 `result.md` 的写法 | 13 个红绿样本：改前判对 6 个，改后 13 个；拒绝说明第一行点名文件名与三个前缀，「→ 怎么办」照旧给「报告行改成草稿目录下的 result.md」 |
| ㉑ | `x-summary.md`、`summary.txt` 照旧放行 | 样本与自证格都放行 |
| ㉑ | 自证加红绿格与弄坏开关 | 加 8 格（拒 4、放 3、拒绝说明 1），开关 `report-name-prefixes-unchecked` 打开后红的正是拒的 4 格与说明那一格 |
| ㉑ | 在真提示上比改前改后 | 本项目全部 3324 份子 agent 首条派发提示（兄弟工具员那时 3317 份，之后又派了几份）：新拒 1 份 a2aeb330cc47cbfcf（investigator，2026-10-18，报告行写 findings.md）；三方形态与写范围一份都没变 |
| ③ | 照 `agent-watch.py --selftest` 今天起的东西改理由，档位不动 | 理由补上「判定行里登记成轻的自证临时目录那一格另起三个只睡的 bash」，档位还是 light（第 3 列） |

推翻条件：
- 主工作区下面四条有一条不退 0：`bash .claude/hooks/runner-dispatch-guard.sh --selftest`、`bash .claude/hooks/handback-guard.sh --selftest`、`python3 scripts/agents/dispatch-prompt.py --selftest`、`python3 scripts/agents/agent-cost.py --selftest`；
- 四个新开关里有一个打开后自证退 0，或红的不在「三方腿类型表」那一格（`report-name-prefixes-unchecked` 是「报告名」那几格）；
- 往某份定义 frontmatter 加或删 `leg: three-way` 之后，四处读到的集合有一处不跟着变；
- 加了 `leg:` 这一行之后，three-way-forward、three-way-attack、three-way-defense 派不出去（Claude Code 报 frontmatter 不认）。

## 1. 依据：定义 frontmatter 加键能不能照常派

查了 `.claude/agents/` 下 14 份定义的 frontmatter：每一份都有 `required-inputs:` 一行，这不是 Claude Code 的键，只有派发闸（`agent_definition_fields`）与骨架（`frontmatter_fields`）读它。它从提交 260fa60a9（2026-09-27，「里程碑二收尾：agent 定义、共用约束与主 agent 入口」）起就在每份定义里，此后这些定义一直照常派（这一次派我的 tooling-writer 定义也带着它）。所以按先例加一个键 `leg: three-way`，放在三份定义 frontmatter 的 `required-inputs:` 之后、收尾的 `---` 之前，没有改成登记表。

这一条是按先例推的：Claude Code 不认的键会被忽略，我没查它的文档，也没在加了键之后真派一次三方腿。推翻条件写在结论里。

## 2. 改过的文件

| 文件 | 改了什么 | 相对改前原样（`git diff --no-index --stat` 末行） |
|---|---|---|
| `.claude/agents/three-way-forward.md` | 只动 frontmatter：加 `leg: three-way` | 1 file changed, 1 insertion(+) |
| `.claude/agents/three-way-attack.md` | 同上 | 1 file changed, 1 insertion(+) |
| `.claude/agents/three-way-defense.md` | 同上 | 1 file changed, 1 insertion(+) |
| `.claude/hooks/lib_three_way_legs.py` | 新建（66 行）：`three_way_leg_types`（拆键值的读法）、`three_way_leg_types_scanned`（自证用的另一种读法：不拆键值，在 frontmatter 那一段里按整行找） | 新文件 |
| `.claude/hooks/runner-dispatch-guard.sh` | ⑯ 改读库；㉑ ⑰ 加判三个前缀；自证 12 格；开关 2 个；文件头 ⑰ 与开关表 | 1 file changed, 83 insertions(+), 12 deletions(-) |
| `.claude/hooks/handback-guard.sh` | ⑯ 改读库；自证 4 格；开关 1 个；文件头 ② 与开关表 | 1 file changed, 54 insertions(+), 7 deletions(-) |
| `.claude/hooks/selftest-registry.tsv` | 只改 agent-watch.py 那一行的理由 | 1 file changed, 1 insertion(+), 1 deletion(-) |
| `scripts/agents/dispatch-prompt.py` | ⑯ 改读库（骨架打三方腿报告占位挪进 skeleton()）；自证 2 项；开关 1 个；文件头 ⑰ 那句补三个前缀 | 1 file changed, 59 insertions(+), 12 deletions(-) |
| `scripts/agents/agent-cost.py` | ⑯ 改读库；假仓加定义与一个材料员；自证 1 项；开关 1 个；文件头桶那一段 | 1 file changed, 50 insertions(+), 8 deletions(-) |

`git diff --stat -- <这九份>` 原样（工作区对暂存区；这一批干活的时候有别的会话把这几份的一部分改动 `git add` 进了暂存区——我没做任何 git 写操作——所以这里只剩暂存之后的那一截，不是这一批的全部改动，全部改动看上表）：

```
 .claude/hooks/handback-guard.sh        |  11 +-
 .claude/hooks/lib_three_way_legs.py    |  21 ++-
 .claude/hooks/runner-dispatch-guard.sh |   8 +-
 .claude/hooks/selftest-registry.tsv    |   2 +-
 scripts/agents/agent-cost.py           | 272 ++++++++++++++++++++++++++++-----
 scripts/agents/dispatch-prompt.py      |  71 +++++++--
 6 files changed, 319 insertions(+), 66 deletions(-)
```

`git status --short` 里 `.claude/hooks/lib_three_way_legs.py` 是 `AM`、三份定义是 `M `：被别的会话暂存了，提交时以工作区这一版为准。

sha256（主工作区，交回时）：

```
7cd15e90da562040c399974b23b0c9bb358271a3b494b883a4a3d3e1f2474d7b  .claude/agents/three-way-forward.md
e6e1cb3e3dd0acfc760ffe8539952fda9c82f57e6d154a8cc63cc4f8009c8896  .claude/agents/three-way-attack.md
f1e03d96b9e4f0f5c8f88ab01ed8c3b4cab4a3147cf5dcad77ecf2ef69b5dea9  .claude/agents/three-way-defense.md
be47321288e64d3509f51312898febd05ca8a72a658d876bf45751ad9053a3bf  .claude/hooks/lib_three_way_legs.py
ce5cc2616c2667979836c6ddf2dfd460f358350cae533b17992af3a1e953c6a1  .claude/hooks/runner-dispatch-guard.sh
bab11748f599621eaf13ef5fd47364b537513a32de04fecae9ad6a169fa07548  .claude/hooks/handback-guard.sh
d10f678fc1ae507edadbf7b247cf26a52c23789fe41e92790e83c90c0a7f313f  .claude/hooks/selftest-registry.tsv
e6f5e4f51df55da831b56e7bfe20036a5b66eab19ed1c9b49966ba9a24d6d49e  scripts/agents/dispatch-prompt.py
2553b7dcd263957fcf870091650f2871b4d6b137d46fcd4c0c255d5344942487  scripts/agents/agent-cost.py
```

开工时的基线（`originals/manifest.tsv`）：派发闸 b3db189f…（兄弟工具员交回时那一版）、交回闸 d755973d…（b9-handback-guard.md 记的那一版）、agent-cost.py 2fbd9c46…（b9-watch-cost.md 记的那一版）、dispatch-prompt.py 774dd0b9…。每次换之前都核过主工作区与基线逐字相同，兄弟工具员改的那几处没动。

## 3. ⑯ 三方腿类型表收成一处

### 读法与四处

- 登记处：定义 frontmatter 里 `leg:` 那一行的值（去掉行尾 # 注释）等于 `three-way`。正文里写的不算。
- `lib_three_way_legs.three_way_leg_types(<仓根>)` 读 `<仓根>/.claude/agents/*.md`，交 frozenset；目录不在或一份都没登记交空集。
- 派发闸：`three_way_report_types(project_root)`，按派发那一刻提示的仓根读（与 required-inputs 同一个根）；⑰ 三方形态只对它判。读不到库时交空集、stderr 写「派发闸读不到共用模块 …」（hook-swap 认这句话）。
- 交回闸：`citation_checked_agents(project_root)`；② 只对它判。读不到库同样交空集、stderr 写一句。
- 骨架：`three_way_report_types(agents_dir)`；skeleton() 里按它打 `报告：research/prompts/<填 轮>-<填 模型>-output.md`（原来在导入时按写死的表塞进 EXTRA_LINES，改成在 skeleton() 里现判，次序不变）。
- 账表：`collect_outcome` 里 `three_way_leg_types_of(repository)` 读 `--repo` 那棵树，逐个子 agent 记 `three_way_leg`，outcome_bucket 看它；那棵树有 `.claude/agents` 而一份都没登记时 stderr 写一句。

行为上变了的（只在这两处，派发闸与骨架在本仓上读到的与改前相同）：
- 交回闸原来的表多一个 three-way-verifier。这个定义在提交 dbcc0d2f5 里删了（`git log --oneline --all -- .claude/agents/three-way-verifier.md`），没有定义的类型交回闸本来就不判，所以在本仓上没有可见的变化。
- 账表原来按类型名以 three-way 开头认，three-way-materials 交回了也进三方腿两个桶（第三轮辩方 N11）；现在不进，按引用分桶。历史里的 three-way-verifier、three-way-local-attack 这类已经没有定义的类型同样不再进那两个桶。我没在真会话记录上跑 outcome 比改前改后（见「没做什么」）。

### 改前造的该红的输入：探针 `tools/leg_probe.py`

照四处各自的判法走一遍（派发闸调 three_way_report_verdict、交回闸调 decide、骨架调 skeleton、账表在造出来的会话目录上调 collect_outcome），不读它们的常量，拿读到的集合比登记处。样本仓 `sample-root/`（git 仓，交回前已删）里的登记故意与改前写死的表不同：
three-way-forward 写 `leg: three-way`；three-way-attack 不写、正文里写一行 `leg: three-way`；three-way-defense 写 `leg: three-way  # 注释`；three-way-materials 不写；three-way-verifier 写 `leg: other`；sample-leg 写 `leg: three-way`；clerk 不写。

改前（主工作区的四份原样），样本仓，原样：

```
runner-dispatch-guard.sh 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处不同：多 ['three-way-attack'] 少 ['sample-leg']
handback-guard.sh 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier']；与登记处不同：多 ['three-way-attack', 'three-way-verifier'] 少 ['sample-leg']
dispatch-prompt.py 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处不同：多 ['three-way-attack'] 少 ['sample-leg']
agent-cost.py 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-materials', 'three-way-verifier']；与登记处不同：多 ['three-way-attack', 'three-way-materials', 'three-way-verifier'] 少 ['sample-leg']
登记处（/home/fy5090/.local/share/singlefs/work/landing-audit-b9-leg-types/sample-root/.claude/agents/ 里 frontmatter 写了 leg: three-way 的）['sample-leg', 'three-way-defense', 'three-way-forward']；四处与它逐个相同的 0 处
```

改前，本仓（三份定义已经加了键、四份脚本还是原样），原样：

```
runner-dispatch-guard.sh 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
handback-guard.sh 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
dispatch-prompt.py 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
agent-cost.py 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-materials', 'three-way-verifier']；与登记处不同：多 ['three-way-materials', 'three-way-verifier'] 少 []
登记处（/home/fy5090/code/singlefs/.claude/agents/ 里 frontmatter 写了 leg: three-way 的）['three-way-attack', 'three-way-defense', 'three-way-forward']；四处与它逐个相同的 3 处
```

改后（主工作区，交回时这一版），样本仓与本仓，原样：

```
runner-dispatch-guard.sh 读到 ['sample-leg', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
handback-guard.sh 读到 ['sample-leg', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
dispatch-prompt.py 读到 ['sample-leg', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
agent-cost.py 读到 ['sample-leg', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
登记处（/home/fy5090/.local/share/singlefs/work/landing-audit-b9-leg-types/sample-root/.claude/agents/ 里 frontmatter 写了 leg: three-way 的）['sample-leg', 'three-way-defense', 'three-way-forward']；四处与它逐个相同的 4 处
runner-dispatch-guard.sh 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
handback-guard.sh 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
dispatch-prompt.py 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
agent-cost.py 读到 ['three-way-attack', 'three-way-defense', 'three-way-forward']；与登记处逐个相同
登记处（/home/fy5090/code/singlefs/.claude/agents/ 里 frontmatter 写了 leg: three-way 的）['three-way-attack', 'three-way-defense', 'three-way-forward']；四处与它逐个相同的 4 处
```

### 自证新加的格

四份的格都是同一个意思：「读到的类型集合」取那一处判法真用的函数，「frontmatter 里写着什么」取库里 `three_way_leg_types_scanned`（不拆键值的另一种读法，四处共用，不各写一份）；样本仓的登记同上面那样故意与写死的表不同，本仓按 frontmatter 现扫、至少要有一种。

| 文件 | 新格（失败行「✗ 自检：」之后的开头） | 弄坏开关 |
|---|---|---|
| 派发闸 | `三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同`、`三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种`、`三方腿类型表:样本仓登记的 sample-leg 报告写草稿目录下的 result.md 拒`、`三方腿类型表:样本仓没登记的攻方报告写草稿目录下的 result.md 不按三方腿判` | `RUNNER_DISPATCH_GUARD_BREAK=leg-types-hardcoded`（文件头第 216 行） |
| 交回闸 | `三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同`、`…本仓读到的…至少登记了一种`、`三方腿类型表:样本仓登记的 sample-leg 没点名报告拒`、`三方腿类型表:样本仓没登记的攻方不核引文` | `HANDBACK_GUARD_BREAK=leg-types-hardcoded`（文件头第 102 行） |
| 骨架 | `三方腿类型表：样本仓骨架按三方腿打报告占位的类型应当与 frontmatter 逐个相同`、`三方腿类型表：本仓骨架按三方腿打报告占位的类型应当与 frontmatter 逐个相同、至少登记了一种` | `DISPATCH_PROMPT_BREAK=leg-types-hardcoded`（文件头第 84 行） |
| 账表 | `三方腿类型表：`（假仓与本仓的集合，加上假仓里没登记的材料员应得 report-unreferenced） | `AGENT_COST_BREAK=leg-types-by-prefix`（文件头第 65 行） |

为了让这几格判得动，原有样本跟着补了登记：派发闸自证的样本仓 `work` 里三种腿各补一份带 `leg: three-way` 的定义；交回闸样本仓里攻方与核查员的定义补 frontmatter；骨架自证里三种腿的样本定义补一行；账表假仓补四份定义（三种腿登记、材料员不登记）与第 21 个子 agent（材料员交回）。

四个开关打开后的失败行（主工作区、交回时这一版，`tools/final_selftests.py` 跑的，`logs/final-summary.txt` 原样）：

```
== runner-dispatch-guard-leg-types-hardcoded RUNNER_DISPATCH_GUARD_BREAK=leg-types-hardcoded：退出码 1；没有成功句；失败行 3 条
   ✗ 自检：三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同 应当返回 ['sample-leg', 'three-way-defense', 'three-way-forward']，实际 ['three-way-attack', 'three-way-defense', 'three-way-forward']
   ✗ 自检：三方腿类型表:样本仓登记的 sample-leg 报告写草稿目录下的 result.md 拒 应当返回 2，实际 0
   ✗ 自检：三方腿类型表:样本仓没登记的攻方报告写草稿目录下的 result.md 不按三方腿判 应当返回 0，实际 2
== runner-dispatch-guard-report-name-prefixes-unchecked RUNNER_DISPATCH_GUARD_BREAK=report-name-prefixes-unchecked：退出码 1；没有成功句；失败行 5 条
   ✗ 自检：报告名:草稿目录下以 SUMMARY 开头的 .md 拒 应当返回 2，实际 0
   ✗ 自检：报告名:草稿目录下以 findings 开头的 .md 拒（小写） 应当返回 2，实际 0
   ✗ 自检：报告名:草稿目录下以 Analysis 开头的 .md 拒（大小写混写） 应当返回 2，实际 0
   ✗ 自检：报告名:不带目录的裸文件名以 summary 开头的 .md 照判 应当返回 2，实际 0
   ✗ 自检：报告名:以三个前缀开头的 .md 拒绝说明点名文件名、三个前缀与 result.md 的写法 应当返回 True，实际 False
== handback-guard-leg-types-hardcoded HANDBACK_GUARD_BREAK=leg-types-hardcoded：退出码 1；没有成功句；失败行 4 条
   ✗ 自检：三方腿类型表:样本仓读到的类型与 frontmatter 逐个相同 应当返回 ['sample-leg', 'three-way-defense', 'three-way-forward']，实际 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier']
   ✗ 自检：三方腿类型表:本仓读到的类型与 frontmatter 逐个相同、至少登记了一种 应当返回 (['three-way-attack', 'three-way-defense', 'three-way-forward'], True)，实际 (['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-verifier'], True)
   ✗ 自检：三方腿类型表:样本仓登记的 sample-leg 没点名报告拒 应当返回 2，实际 0
   ✗ 自检：三方腿类型表:样本仓没登记的攻方不核引文 应当返回 0，实际 2
== dispatch-prompt-leg-types-hardcoded DISPATCH_PROMPT_BREAK=leg-types-hardcoded：退出码 1；没有成功句；失败行 1 条
   ✗ 自检：三方腿类型表：样本仓骨架按三方腿打报告占位的类型应当与 frontmatter 逐个相同 ['sample-leg', 'three-way-defense', 'three-way-forward']，实际 ['three-way-attack', 'three-way-defense', 'three-way-forward']
== agent-cost-leg-types-by-prefix AGENT_COST_BREAK=leg-types-by-prefix：退出码 1；没有成功句；失败行 1 条
   ✗ 自检：三方腿类型表：假仓读到的 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-materials'] 应当与 frontmatter 的 ['three-way-attack', 'three-way-defense', 'three-way-forward'] 逐个相同，本仓读到的 ['three-way-attack', 'three-way-defense', 'three-way-forward', 'three-way-materials'] 应当与 frontmatter 的 
```

上面后五段里，派发闸 `report-name-prefixes-unchecked` 那一段属于 ㉑（第 4 节）。交回闸那一个开关下「本仓…至少登记了一种」也红（写死的表多 three-way-verifier），文件头没点名它：点名里要写本仓此刻的类型名，加一种腿它就对不上，所以只点名样本仓那三格（复跑方按「点名的都在失败行里」判，多红的不算错）。

（上面每条失败行由 `tools/final_selftests.py` 截到 300 字；全文在 `logs/final-<名>.txt`。）

## 4. ㉑ 派发闸 ⑰ 加判三个前缀

改法（派发闸，行号是交回时这一版）：
- 第 1322 行 `REPORT_REFUSED_PREFIX_NAME = re.compile(r"(?i:summary|findings|analysis)[^/]*\.md")`，第 1432 行对报告行里交文件名判的每个词的最后一段做 fullmatch；原来「名字里带 report」那一判照旧，两判任一成立就拒。只判 own_report_token 认的词（本提示草稿目录下的、裸文件名），别的目录下的当转述不判，与 report 那一判同一套。
- 拒绝说明第一行改成「报告行点名的文件名里带 report，或是以 SUMMARY、FINDINGS、ANALYSIS 开头的 .md：…」，「→ 规矩」补一句三个前缀与实测出处，「→ 怎么办」不变（改成草稿目录下的 result.md）。
- 文件头 ⑰ 的「文件名」那一条、report_file_name_verdict 的说明、自证成功句跟着补；骨架 `dispatch-prompt.py` 文件头 ⑰ 那一句也补了三个前缀。
- `.md` 只认小写：探针只测了小写 `.md`，`SUMMARY.MD` 这类工具层拒不拒没测过，这里不拦。

改前造的该红的输入：`tools/prefix_probe.py`，13 个样本直接喂 report_file_name_verdict。改前（主工作区原样），原样：

```
错  应得 拒  实得 放  草稿目录下 SUMMARY.md
错  应得 拒  实得 放  草稿目录下 summary-x.md（小写）
错  应得 拒  实得 放  草稿目录下 FINDINGS.md
错  应得 拒  实得 放  草稿目录下 findings-x.md
错  应得 拒  实得 放  草稿目录下 ANALYSIS.md
错  应得 拒  实得 放  草稿目录下 Analysis-notes.md（大小写混写）
错  应得 拒  实得 放  裸文件名 summary.md
对  应得 放  实得 放  草稿目录下 x-summary.md（前缀不在开头）
对  应得 放  实得 放  草稿目录下 x-findings.md
对  应得 放  实得 放  草稿目录下 summary.txt（不是 .md）
对  应得 放  实得 放  别的草稿目录下的 SUMMARY.md 当转述不判
对  应得 放  实得 放  草稿目录下 result.md
对  应得 拒  实得 拒  草稿目录下 report.md（原来就拒）
13 个样本，判对 6 个
```

改后（主工作区，交回时这一版），原样：

```
对  应得 拒  实得 拒  草稿目录下 SUMMARY.md
对  应得 拒  实得 拒  草稿目录下 summary-x.md（小写）
对  应得 拒  实得 拒  草稿目录下 FINDINGS.md
对  应得 拒  实得 拒  草稿目录下 findings-x.md
对  应得 拒  实得 拒  草稿目录下 ANALYSIS.md
对  应得 拒  实得 拒  草稿目录下 Analysis-notes.md（大小写混写）
对  应得 拒  实得 拒  裸文件名 summary.md
对  应得 放  实得 放  草稿目录下 x-summary.md（前缀不在开头）
对  应得 放  实得 放  草稿目录下 x-findings.md
对  应得 放  实得 放  草稿目录下 summary.txt（不是 .md）
对  应得 放  实得 放  别的草稿目录下的 SUMMARY.md 当转述不判
对  应得 放  实得 放  草稿目录下 result.md
对  应得 拒  实得 拒  草稿目录下 report.md（原来就拒）
13 个样本，判对 13 个
```

自证新加的 8 格（文件头开关表第 213 行 `report-name-prefixes-unchecked` 点名其中 5 格，失败行见第 3 节末尾那段）：`报告名:草稿目录下以 SUMMARY 开头的 .md 拒`、`报告名:草稿目录下以 findings 开头的 .md 拒（小写）`、`报告名:草稿目录下以 Analysis 开头的 .md 拒（大小写混写）`、`报告名:不带目录的裸文件名以 summary 开头的 .md 照判`（应得 2）；`报告名:前缀在名字中间或末尾的放行`、`报告名:以 summary 开头但不是 .md 的放行`、`报告名:别的草稿目录下以 SUMMARY 开头的 .md 当转述不判`（应得 0）；`报告名:以三个前缀开头的 .md 拒绝说明点名文件名、三个前缀与 result.md 的写法`。

换上时 hook-swap 的喂法（`feeds-dispatch.json` 9 条，stdin 入口）：改前那一版喂下去，`SUMMARY.md`、`findings-x.md` 两条退 0（应当 2），别的 7 条对；换上之后 9 条都对。

骨架：14 种定义照 dispatch-prompt.py 打骨架、填好，主工作区派发闸 `--check-prompt` 核（`tools/skeletons_check.py`，`logs/skeletons.txt` 末行原样）：

```
种类 14；填好放行 14；报告行换成草稿目录下的 SUMMARY.md 被「报告文件名」拒 12
```

设计员与材料员的骨架没有「报告：」一行，不在那 12 种里。

### 真提示上比改前改后

照兄弟工具员的比法：拿他的 `landing-audit-b9-dispatch-guard/tools/corpus.py`（把派发闸内嵌的 python 当模块载入，不跑 main 与自证），改前取开工时的原样（`originals/`），改后取主工作区这一版，在本项目全部子 agent 会话记录的首条派发提示上比 ⑰ 文件名、⑰ 三方形态、④ 写范围三样判定。`logs/corpus-final.txt` 原样：

```
prompts 3324
name:old=放:new=拒 1
```

新拒的那 1 份：a2aeb330cc47cbfcf（investigator，首条记录 2026-10-18），提示里一行「报告：~/.local/share/singlefs/work/review-report-central-mapping-investigation/findings.md」，落在它自己的草稿目录下。三方形态、写范围一份都没变。

这一份的实情：investigator 的 tools 里没有 Write，它在 Bash 里建了 findings.md，之后两次用 Edit 改都成了（它的会话记录里两次 Edit 的工具结果都是「has been updated successfully」）。也就是说工具层的拦截只挡 Write 新建，这一份当时并没有被挡；派发闸现在照验收标准拒它，对没有 Write 的类型是多拦了。要不要按类型的 tools 放宽，交主 agent 定，我没改。

## 5. ③ 登记表一行理由

`.claude/hooks/selftest-registry.tsv` 第 28 行，改前改后（`git diff --no-index` 原样）：

```
-scripts/agents/agent-watch.py	light	# 起的「重型进程」是名字像 checker 档测试二进制、里面只睡的 python 替身，占内存那一格至多 256M，阈值的两处来源换成造出来的；判定行那一格是两个开着样本日志的 sleep，不起 cargo
+scripts/agents/agent-watch.py	light	# 起的「重型进程」是名字像 checker 档测试二进制、里面只睡的 python 替身，占内存那一格至多 256M，阈值的两处来源换成造出来的；判定行那一格是两个开着样本日志的 sleep，判定行里登记成轻的自证临时目录那一格另起三个只睡的 bash（一个经 bash -c、一个经 timeout 起样本自证脚本，它在临时目录里 mktemp、printf 几份带 ✗ 的样本输出再 sleep；一个开着样本日志 exec sleep），不起 cargo
```

依据：`agent-watch.py` 的 `selftest_verdict_lines` 起两个 `bash -c 'exec 3>>"$0"; exec sleep 60'`（就是原理由里那两个开着样本日志的 sleep）；`selftest_verdict_selftest_scratch` 另起三个：`bash -c 'bash "$0" --selftest; :'` 与 `timeout 60 bash <样本脚本> --selftest` 起样本自证脚本 sample-wheel.sh（mktemp 三个目录、printf 几份带 ✗ 的输出、开着它们 sleep 60），第三个 `bash -c 'exec 3>>"$0"; exec sleep 60'` 开着一份样本日志。都不起 cargo。档位 light 没动；经 hook-swap 换上（喂的是 heavy-test-guard.sh 内置的 28 条，草稿与主工作区都对）。

## 6. 自证总数与读数（主工作区、交回时这一版）

四份都登记为 light，我自己跑的（`logs/final-summary.txt` 前四行原样）：

```
== runner-dispatch-guard：退出码 0；✓ 自检通过（查了 483 种）；失败行 0 条
== handback-guard：退出码 0；✓ handback-guard 自检通过（查了 49 种）；失败行 0 条
== dispatch-prompt：退出码 0；✓ 自检通过（查了 92 项）；失败行 0 条
== agent-cost：退出码 0；✓ 自检通过（查了 23 项，其中 outcome 的 21 个子 agent）；失败行 0 条
```

| 自证 | 开工时 | 交回时 | 新加 |
|---|---|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 471 种 | 483 种 | ⑯ 4 格、㉑ 8 格 |
| `bash .claude/hooks/handback-guard.sh --selftest` | 45 种 | 49 种 | ⑯ 4 格 |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 90 项 | 92 项 | ⑯ 2 项 |
| `python3 scripts/agents/agent-cost.py --selftest` | 22 项、outcome 20 个子 agent | 23 项、21 个子 agent | ⑯ 1 项、材料员 1 个 |

开工时的数取自兄弟工具员的报告（b9-dispatch-guard.md、b9-handback-guard.md、b9-watch-cost.md），我没在开工时重跑。

开关逐条复跑（照门禁 code-tooling 的 break-switch-replay 格的读法，读法从 code-tooling.sh 里现取；派发闸、交回闸用兄弟工具员的 `replay-emulate.py`，.py 两份用我照抄后只改起法的 `tools/replay-emulate-any.py`）：

| 文件 | 条数 | 结局 |
|---|---|---|
| 派发闸 | 98（开工时 96，新加 2） | 对 98 |
| 交回闸 | 19（新加 1） | 对 19 |
| agent-cost.py | 13（新加 1） | 对 13 |
| dispatch-prompt.py | 18（新加 1） | 对 15、不符 2（long-run-lines-dropped、live-words-narrowed）、没点名 1（long-run-types-narrowed） |

dispatch-prompt.py 那三条是开工之前就有的写法（点名里带 `<类型>`、`…` 这类占位，或点名不在「」里），这一批的规格没碰那三条（`diff` 原样对改后，三条开关名命中 0 处），新加的 leg-types-hardcoded 是「对」。dispatch-prompt.py 不在复跑登记表里，门禁不复跑它。

语法：hook-swap 每次在草稿与主工作区都跑了 `bash -n` 与内嵌 python 编译；两份 .py `python3 -m py_compile` 过。没改 `.rs`，没跑命名检查。

## 7. 新写的 `# gate-similar:` / `# hook-events:` 行

无：没有新钩子、新门禁阶段。新文件只有共用库 `.claude/hooks/lib_three_way_legs.py`（同目录别的 lib_*.py 一样，文件头只写用途与谁用它）。它不读弄坏开关，不进 break-switch-headers 的判。

## 8. 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`，不起重活、不用内存包装。

1. 新开关进了复跑登记表里的两份（交回闸、agent-cost.py），文件头开关表另两份也加了条目。跑复跑方与文件头两格：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay、break-switch-headers 两格; 范围=2 格; 为什么=交回闸、账表、派发闸、骨架新加 leg-types-hardcoded、leg-types-by-prefix、report-name-prefixes-unchecked 开关，核文件头三样写全与点名的都在失败行里' bash .claude/gate.d/code-tooling.sh --check break-switch-replay,break-switch-headers /home/fy5090/code/singlefs`
   看两格的成功句：交回闸 19 条、agent-cost.py 13 条都记「对」；break-switch-headers 不点名这四份。
2. 三份定义改了 frontmatter，进第九批收尾的定义三方（门禁 doc-process-records 的 agent-def-adversarial-review 格），不归我开。
3. 提交前整轮门禁里的门禁查重（gate-overlap）、hooks-registered、gate-lint、shell-lint 归主 agent，我没跑。
4. 可选：要看账表在真会话记录上桶变了多少，在改前改后各跑一次 `python3 scripts/agents/agent-cost.py outcome --since 2026-09-27 --summary` 比两份汇总（改前那一版在 `originals/scripts/agents/agent-cost.py`，要放回仓里的位置才起得来，所以这一条只能先跑改后、对着上一份落账比）。
5. ㉑ 新拒的那一类（没有 Write 工具的类型报告行写以三个前缀开头的 .md）要不要放宽，见第 4 节末，交主 agent 定。

## 9. 换法与回滚

- 两轮换上。第一轮：库 Write 新建；派发闸、交回闸、登记表经 `scripts/gate/hook-swap.py`（规格 `spec-dispatch-guard.json` 14 处、`spec-handback-guard.json` 11 处、`spec-registry.json` 1 处），`scripts/agents/` 两份经 `scripts/kb/replace-batch.py`（`spec-scripts-agents.json` 22 处，先 `--dry-run`）。第二轮：把四份自证里各写一份的「frontmatter 现扫」收进库（`three_way_leg_types_scanned`），免得四份留同一段：库与派发闸一起经 hook-swap（`spec2-lib-and-dispatch-guard.json` 3 处），交回闸经 hook-swap（`spec2-handback-guard-sh.json` 2 处），两份 .py 经 replace-batch（`spec2-scripts-agents.json` 2 处）。
- 规格都由兄弟工具员的 `make-spec.py` 从主工作区此刻原文与草稿全文逐段对出，施加得到草稿、倒着施加回到原文都核过；每次换之前核主工作区与基线逐字相同，换上之后核主工作区与草稿逐字相同。
- 草稿树里起不了改过的脚本的自证（重型闸拒副本里内容不同的脚本），所以自证都在换上之后在主工作区跑；草稿里用的是探针（把内嵌 python 当模块载入）与 hook-swap 的喂法。
- 回滚：反向规格在 `hook-swap/reverse-*.json`（hook-swap 每次换上写一份）与 `reverse-spec-scripts-agents.json`、`reverse-spec2-scripts-agents.json`，按时间倒着跑；三份定义各删 `leg: three-way` 那一行；库最后删。改前原样在 `originals/`。

## 10. 草稿目录里留着的

`/home/fy5090/.local/share/singlefs/work/landing-audit-b9-leg-types/`：

- `originals/`：开工时八份文件的原样与 `manifest.tsv`（`python3 scripts/agents/copy-originals.py check <草稿目录>` 列出改过的），`.deliverables` 里登记成备份；
- `spec-*.json`、`spec2-*.json`、`reverse-spec*.json`、`hook-swap/`（每次换上的喂法输出与反向规格）、`feeds-dispatch.json`、`feeds-handback.json`；
- `tools/`：`leg_probe.py`（四处读到的类型集合对登记处）、`prefix_probe.py`（㉑ 的 13 个样本）、`skeletons_check.py`、`final_selftests.py`、`replay-emulate-any.py`；
- `logs/`：上面各节引的原样输出。

大小（`du -sh`）：

```
2.9M .
```

交回前删掉的：草稿树 `tree/`（912K，`.claude`、`scripts` 下没改的项是指回主工作区的符号链接，四份脚本与三份定义是拷贝）、样本仓 `sample-root/`（312K，git 仓）、`skel/`（108K，14 种骨架）、第二轮草稿 `draft2/`（五份文件的拷贝，与主工作区逐字相同之后删的）、比真提示用的 `corpus-old/`（一份派发闸原样加指回主工作区库的符号链接）。没有仓副本，也没有编译目录。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交，没做任何 git 写操作（暂存区里这几份的改动是别的会话 `git add` 的）。
- 没跑门禁阶段与各类 lint（break-switch-replay、break-switch-headers、gate-overlap 等写进第 8 节交主 agent）；没改 `.rs`，没跑命名检查。
- 加了 `leg:` 键之后没真派一次三方腿核 Claude Code 认不认（第 1 节，按 `required-inputs:` 的先例推的）。
- 账表改读登记处之后，没在真会话记录上跑 outcome 比改前改后桶变了多少；历史里已经没有定义的 three-way-verifier、three-way-local-attack 与 three-way-materials 交回的那些不再进三方腿两个桶，数没量。
- ㉑ 只认小写 `.md`；`SUMMARY.MD`、`summaryx.md`（前缀后面直接跟字母）、`.markdown` 这几种工具层拒不拒，探针没测，这里按「以前缀开头、扩展名是小写 .md」判（`summaryx.md` 照拒、`.MD` 不拦）。
- ㉑ 新拒的那 1 份真提示是没有 Write 工具的 investigator，工具层当时没挡它（第 4 节末）；按类型的 tools 放宽没做。
- dispatch-prompt.py 开关表里开工前就对不上的三条（long-run-lines-dropped、live-words-narrowed、long-run-types-narrowed）没改；`dispatch-prompt.py` 文件头「派发闸 ⑰ 对这三种另判形态：报告行点名的文件不是 … 或没有报告行就拒」那一句兄弟工具员记过已经不准，我只把「三种」改成按 frontmatter 认，那半句判法描述没改。
- 交回闸文件头 ①、③–⑥ 与别的格没碰；`.claude/gate.d/`、`scripts/agents/agent-watch.py`、`scripts/agents/watch.sh`、`.claude/hooks/bash-command-detector.sh`、三份定义的正文、别的定义、`.claude/agent-common.md`、`.claude/main-agent.md`、`records/`、`research/` 都没碰。
