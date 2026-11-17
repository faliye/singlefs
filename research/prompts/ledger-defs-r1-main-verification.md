# ledger-defs-r1：判决（2026-11-12）

正文 `research/prompts/_ledger-defs-r1-body.md`，开工快照 `research/prompts/ledger-defs-r1-snapshot/`。这一题的第一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（Opus） | `research/prompts/ledger-defs-r1-opus-output.md` | 第一组五条里四条打中、一条主句没打中；第二组第 2、3 条打中；第三组没打中；第四组两条打中 |
| 正推（Sonnet） | `research/prompts/ledger-defs-r1-sonnet-output.md` | 一·1 推不出（其中一处不成立）、一·2 不成立、一·3 不成立、一·4 成立、一·5 推不出；二·1 成立、二·2 两条新模板成立而变异模板不成立、二·3 有两类点不到；三·1、三·2 成立；四·1 成立而差的一半没写全、四·2 成立 |

核查：

- 快照树：`task-server/task submit --cwd ~/.local/share/singlefs/work/ledger-defs-r1-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py ledger-defs-r1 --out ~/.local/share/singlefs/work/ledger-defs-r1-verify/ledger-defs-r1-tree --repository /home/fy5090/code/singlefs`（任务 14554 退 0）：「快照 ledger-defs-r1：清单点名 6 份，倒推出 6 份，sha256 对上 6 份」，六份都取工作区此刻那一份。`sha256sum -c research/prompts/ledger-defs-r1-snapshot/judged-sha256.txt` 六行都对上：腿跑着的时候被判的文件没变。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/ledger-defs-r1-opus-output.md research/prompts/ledger-defs-r1-sonnet-output.md --root ~/.local/share/singlefs/work/ledger-defs-r1-verify/ledger-defs-r1-tree --background research/prompts/_ledger-defs-r1-body.md`：「核了 166 处引文、91 处裸行号，对上 75 处，没判 182 处」。没判的是指到脚本与钩子的引文（快照树里只有六份定义）与裸行号；判决引到的那几处主 agent 逐处现查，写在各条里。
- 探针：攻方腿的范围闸探针，主 agent 把模型目录拷到 `~/.local/share/singlefs/work/ledger-defs-r1-verify/model` 复跑（任务 14551 退 0），末行 `name=probe_summary samples=8 mismatches=0`，八条的退出码与腿预告的相同。正推腿的探针脚本里写死了它自己的草稿目录，没复跑；它要量的四种形状都在攻方那八条里。
- 核不动的：攻方 G1-K（承接的实现员以哪棵树为基线）是它推的；二·3 里「三项整类跑法」主 agent 在合表里按字样没搜到，按没核算；正推腿报的「段树里没有多机配置文件、在段树里起任务读不读得到」要真跑。

## 二、判决

1. 一·3（旧句与新句打架）：打中，采。主 agent 现查：`.claude/main-agent.md` 验证段第 1 步写合入后验证绿了就照脚本打出的那几条命令合进主工作区，第 2 步写带 `--item` 的那几条「在合进主工作区之前交…这一步绿了才照第 1 步打出的那一段命令合进主工作区」，两句先后相反；同一份第 65 行拒算登记还写「合入后验证（验证段第 1 步）绿了之后…在合入后的主工作区里跑」；`.claude/agents/implementation-writer.md` 第 34 行变异证红还写「补丁打进主工作区之后用的那一条」。两条腿一致。腿另点的 `.claude/main-agent.md` 第 7、33、118 行与 `.claude/agent-common.md` 第 115 行，主 agent 没逐句现查，改的时候按原文找。
2. 一·2（三棵树不一样）：打中，采。定义只写了补丁冲突退 2 那一种；主工作区在段树建好之后被别的会话改了而不冲突时，快照上的绿与段树上的绿都不是最后合进去的那棵树上的绿，定义没写这时怎么办。两条腿一致，主 agent 读定义坐实没有这一句。
3. 一·1（走一遍）：部分打中，采三处。合到一半有一份退 2 时主工作区停在没验过的组合上、定义没写去向（两条腿一致）；`scripts/gate/merge-and-verify.sh` 冲突出路里打的建段树命令只带 `--with-test-inputs`，与 `.claude/main-agent.md` 写段第 2 条不一致（主 agent 现查那一句原文）；验证段第 2 步红了之后的重跑命令定义没给。新崩溃枚举用例的登记落在哪棵树、合入后另交的 mutation-tables 格，两条腿各报一处，都读定义坐实没写，采。
4. 一·4（代码轮快照）：没打中。代码轮开在合进主工作区之后，「快照取在全部合入之后」照旧成立，两条腿一致。攻方连带的一处采：段树合完就删，代码轮打中后的修补没有段树可做第 2 步，定义没写这时重建。
5. 一·5（段树与完工清理闸）：打中，采。主 agent 现查完工清理闸里主 agent 一侧能声明的种类是「报告、补丁、备份」，段树按仓副本列；一段做到一半停下时定义没写怎么办。
6. 二·2（命令模板过范围闸）：打中，采。探针坐实：变异门禁点名行、证红、`cargo test` 点名验证包目标三种，点到会走崩溃枚举的目标时范围闸退 2，为什么一段写了全域理由的退 0；六份定义里「全域理由」出现 0 次（主 agent grep 三份都是 0）。两条新模板退 0 是因为范围闸不判这两种形状（`.claude/hooks/run-scope-guard.sh` 文件头「判不到的」那一句，主 agent 现读）；崩溃放量入口没缩域估不出时自己退 79，定义没提。
7. 二·1：没打中，两条腿一致。二·3：部分采——没登记成崩溃枚举用例的测试函数那一段，新两条点不到，走点名测试目标那一条，定义要写明；「三项整类跑法」核不动，不采。
8. 三·1、三·2：没打中。两条腿都记了几处边角（那一行只认认领目录与草稿目录、上游交回闸与看门狗认的是「没删」一类词、没有「能删的不许用」的禁止句），记进改法第 7 条。
9. 四·1：打中，采。主 agent 现查 `task-server/task_server.py` 的 `long_run_split_exempt`：豁免表的项名等于命令的某个词就整条任务豁免；规则没写这一半。四·2：采两处漏改（`.claude/main-agent.md` 里另两处「产品 GPU 包」），`.claude/skills/crash-test/commit-stages.md` 里那一处另一个会话已改。

## 三、改法

都是定义与脚本出路的文字，没有产品代码；逐条标「量过」或「零轮」。

1. 验证段的次序写成一处、别处指过去（量过：打架的句子现查过）：第 1 步合入后验证（快照上验）→ 第 2 步带 `--item` 的那几条与拒算登记、变异证红（段树上验）→ 都绿了才合进主工作区 → 合完再交要读主工作区的那几样（mutation-tables 格这类）→ 代码轮。`.claude/main-agent.md` 第 7、33、57、65、118 行、`.claude/agents/implementation-writer.md` 第 34 行、`.claude/agent-common.md` 第 115 行照这一处改。
2. 合进主工作区之前核三棵树（零轮）：合之前比主工作区此刻与开跑时快照的差，补丁改到的文件与它们同包的文件有变就不合，以主工作区此刻重建段树、补丁照次序重打、第 1、2 步重验；没变才合。怎么比（脚本记快照指纹、合之前现算）归脚本，定义写判据。
3. 合到一半失败（零轮）：整批留底在合之前做了，有一份退 2 就照留底把已合的几份退回、主工作区回到合之前，再照第 2 条重建段树。
4. `scripts/gate/merge-and-verify.sh` 冲突出路里的建段树命令改成与 `.claude/main-agent.md` 写段第 2 条同一条（量过）。
5. 段树的寿命（零轮）：段树留到这一段的代码轮判决写完再删；一段跨轮停下时定义先写「跨轮停下就删、续做时照第 2 条重建」，不依赖闸改；完工清理闸认主 agent 声明的段树归下一批工具改动。
6. 命令模板（量过，探针八条）：实现员定义第 1d 步、第 3a 步与 `.claude/agent-common.md` 里变异门禁点名行、证红、`cargo test` 点名验证包目标三条模板，写明点到走崩溃枚举的目标时带缩域（`--crash-expand-up-to` 或两个缩域变量之一）、确要全域的在为什么一段写「全域理由：」；崩溃放量入口点名分项的模板写明没缩域估不出时入口退 79 与出路；没登记成崩溃枚举用例的测试函数走点名测试目标那一条。
7. 交回那两句的边角（量过，读钩子）：`.claude/agent-common.md` 写明「交主 agent 删」只在删不掉时用、只认认领目录与草稿目录里的。
8. `.claude/rules/long-run-split.md`「门禁管哪一半」补一句：服务按命令里有没有一个词等于豁免表的项名判豁免，一条任务点了豁免的项就整条不提醒（量过）。
9. `.claude/main-agent.md` 另两处「产品 GPU 包」改成与那三处一致（量过）。

第 2、3、5 条是这一轮新提的做法，没被攻过；照规矩第二轮攻它们。

## 交用户的

- 第一组「写段专用的一棵树、先验后合」的定义这一轮站不住：次序自相矛盾、三棵树不一致与合到一半失败两处没写。改法第 1–5 条改完开第二轮。在第二轮判决之前，这套流程不拿去走真的一段实现。
- 范围闸换上之后，实现员照定义写的三条命令点到崩溃枚举目标会被拒；改法第 6 条改定义。

## 这一轮被判过的文件

- .claude/main-agent.md：判了「写与验分两段」与别处同一流程的句子（验证段次序自相矛盾、三棵树不一致与合到一半失败两处缺做法、段树寿命）、崩溃点重放入口换成两条的跟改、三处包名。
- .claude/agent-common.md：判了建副本那一句里的段树、写死的四条命令模板、「交主 agent 删」的出路与检出钩子那一句。
- .claude/agents/implementation-writer.md：判了第 1d 步换成两条之后的命令模板与变异证红那一句的次序、段树副本那一句。
- .claude/agents/investigator.md：判了跟改的两句与新写法一致。
- .claude/rules/implementation-workflow.md：判了跟改的一句与新写法一致。
- .claude/rules/long-run-split.md：判了「门禁管哪一半」三条与门禁那一格、服务提醒今天判的东西，差服务按词豁免那一句。

## 回看决策

不涉及决策：这一轮判的是 agent 定义与流程规则，不动 `.claude/kb/decisions/` 下任何一条。
