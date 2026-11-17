# ledger-defs-r1：账本清欠那一轮改的六份定义与规则（2026-11-12）

<!-- doc-lint:not-numbers B8 A5 D3 -->

## 被判的是什么

2026-11-12 清 `records/2026-10-29-subagent拆分提案.md` 的欠账时，六份定义与规则被改了。这一轮判这些改动：改后的句子照着做，做不做得成、和仓里今天的脚本与钩子对不对得上、有没有互相矛盾。

被判的文件（开工快照 `research/prompts/ledger-defs-r1-snapshot/judged-sha256.txt`）：

- `.claude/main-agent.md`
- `.claude/agent-common.md`
- `.claude/agents/implementation-writer.md`
- `.claude/agents/investigator.md`
- `.claude/rules/implementation-workflow.md`
- `.claude/rules/long-run-split.md`

这六份里另有别的会话没提交的改动，不在这一轮里：只判下面四组点名的句子。改前改后的整句在各工具员的报告里，路径逐组给。

## 实现今天的样子（主 agent 的观测）

这一轮判的是流程文件，不改 `crates/`；与它相关的实现是三份脚本与四份钩子，主 agent 读过或经任务服务跑过：

- `scripts/gate/merge-and-verify.sh`：文件头写「只有这一种跑法，主工作区一个字节不写」，补丁打进快照去验。当天改成：不带 `--skip-apply` 那一趟验红时打的重跑命令是同一形的合入后验证，验绿时打「合进主工作区」那一段（一条 `copy-originals.py` 整批留底，再照次序每份一条 `apply-writer-patch.py`）。自证任务 14430 退 0（「自检通过（查了 104 种）」），六条新开关任务 14444–14449 各退 1、红的格数与文件头点名的条数相同。
- `scripts/gate/apply-writer-patch.py`：没改。工具员读到它有 `--root <树>`，指副本时拿副本锁、不查开工快照（报告 `~/.local/share/singlefs/work/ledger-b2-definitions/result.md` 第二节）。
- `scripts/hosts/minimal-copy.py`：没改。文件头写 `--with-gate-tables` 声明 `.claude/gate.d/`、`--with <条目>=<为什么>` 逐条声明要多拷的仓根条目。
- `.claude/gate.d/crash-point-replay.sh`：`--check crash-case-replay --crash-cases-only --item crash-case:<用例名>` 是按块那一路（工具员读的文件头第 49–52、167–183 行，报告 `~/.local/share/singlefs/work/ledger-b1-writer-definition/result.md`）。
- `.claude/hooks/completion-cleanup-guard.sh`：当天换上，认交回正文里「交主 agent 删 <全路径>：<原因>」一行；自证任务 14456 退 0（判了 84 格）。
- `.claude/hooks/bash-command-detector.sh`：当天换上新一道（子 agent 的 Bash 跑完比对未跟踪文件），之后照用户定的收窄成只记不报（件 19，报告 `~/.local/share/singlefs/work/ledger-b2-untracked-narrow/result.md`）。
- `task-server/task_server_lib.py` 第 179 行：`long-run-split` 提醒按任务（分片的按片）出结局时服务记的用时判。

方案按仓里今天的脚本与钩子来谈；`crates/` 下的产品代码不在这一轮里。

## 四组改动

### 一、写段补丁打进专用的一棵树，验证段先验后合

出处：账本第一百九十五节；用户 2026-11-12 两次定「打进专用的一棵树」「先验后合」。改的是 `.claude/main-agent.md`「写与验分两段」一节（写段第 2、6 条，验证段第 1、2、4 步）、「一轮怎么开、怎么收」第 4、10 条、职责一句、派发表一行；`.claude/agents/implementation-writer.md` 第 15 行一句；`.claude/agent-common.md` 第 50 行一句。改前改后整句：`~/.local/share/singlefs/work/ledger-b2-verify-then-merge/result.md` 第四节；之后主 agent 又改了 `.claude/main-agent.md` 两句（建段树的命令加 `--with-gate-tables` 与三条 `--with`；验证段第 2 步对着段树跑、绿了才合）。

新流程一句话：一段开工建段树；写段的补丁只打进段树，下一批以段树为基线；验证段先交不带 `--skip-apply` 的合入后验证（在快照里验），再对着段树交带 `--item` 的那几条，都绿了才照脚本打出的命令逐份合进主工作区。

要判的：

1. 照改后的句子从头走一遍一段实现（两批有依赖的实现员、三份补丁），每一步交的命令今天的脚本认不认、落在哪棵树上；有没有哪一步主工作区上出现没验过的补丁。
2. 合入后验证在「主工作区的快照加这一批补丁」上验，验证段第 2 步在段树上验，合进去的是主工作区。三棵树什么时候会不一样（段树建好之后主工作区被别的会话改了），不一样时哪一步的绿不作数，定义里写没写怎么办。
3. 同一份定义里有没有句子还照旧做法写（补丁打进主工作区再验、「打过的与没打过的分两条」），与新句子矛盾。
4. 代码轮三方的快照「取在全部合入之后」这一句在先验后合下还成不成立：代码轮开在合进主工作区之前还是之后。
5. 段树用完就删与完工清理闸：段树跨轮留着时，定义叫主 agent 怎么办。

### 二、崩溃点重放入口带 `--item` 换成两条

出处：账本第二百零七节。`.claude/agents/implementation-writer.md` 第 1d 步把崩溃点重放入口带 `--item` 那一条换成：登记的崩溃枚举用例走门禁 `crash-point-replay.sh --check crash-case-replay --crash-cases-only --item crash-case:<用例名>`，崩溃放量分项走 `scripts/crash/crash-amplification.sh --item <分项>`；`.claude/agent-common.md` 3 处、`.claude/agents/investigator.md` 2 处、`.claude/main-agent.md` 9 处、`.claude/rules/implementation-workflow.md` 1 处跟着改。改前改后整句：`~/.local/share/singlefs/work/ledger-b1-writer-definition/result.md` 与 `~/.local/share/singlefs/work/ledger-b2-definitions/result.md` 第一节。

要判的：

1. 六份里还有没有句子叫人交崩溃点重放入口带 `--item`，或与两条新写法对不上（命令形状、范围声明的项数写法）。
2. 写死的命令照抄去交，过不过得了范围闸 `.claude/hooks/run-scope-guard.sh` 当天换上的两判（不指定范围没带标识的拒；指定范围点到会走崩溃枚举的项、没缩域也没写「全域理由：」的拒）。定义里的命令模板没带缩域也没写全域理由，照抄会不会被拒。
3. 没被登记成崩溃枚举用例、也不是放量分项的东西，旧写法点得到而新写法点不到的有没有。

### 三、交回闸与检出钩子的两句

出处：账本第二百一十二、一百九十二节。`.claude/agent-common.md` 第 51、104 行写上「交主 agent 删 <全路径>：<原因>」这一种出路；第 99 行一带写检出钩子新一道。改前改后整句：`~/.local/share/singlefs/work/ledger-b1-detector-cleanup/result.md` 第 6 节与 `~/.local/share/singlefs/work/ledger-b2-untracked-narrow/result.md`。

要判的：

1. 这两句与钩子今天的行为逐字对不对得上（认哪几种目录、路径要不要一字不差、原因几个字、主 agent 一侧照不照列）。
2. 同一份里「用完就删」那一条与这条出路放在一起，子 agent 会不会读成「不想删就写一行交主 agent 删」；定义里有没有写清只在删不掉时用。

### 四、长跑拆分规则的「门禁管哪一半」与三处包名

出处：账本第二百二十节；管产品 GPU 包测试的会话给的三处包名。`.claude/rules/long-run-split.md`「门禁管哪一半」一节改成三条；`.claude/main-agent.md` 三处「产品 GPU 包 `singlefs-background-gpu`」换成「产品 GPU 包的测试包 `singlefs-background-gpu-tests`」。改前改后整句：`~/.local/share/singlefs/work/ledger-b2-definitions/result.md` 第三、四节。

要判的：

1. 「门禁管哪一半」三条与 `.claude/gate.d/code-tooling.sh` 第 65 行那一格、`task-server/task_server_lib.py` 第 179 行今天判的东西对不对得上；规则正文说「一项」而服务按片或整条判，这一节有没有把差的那一半写出来。
2. 三处包名换了之后，同一份与 `.claude/rules/` 下别处还写旧包名、说同一件事的句子有没有。

## 这一轮不判的

- 六份里别的会话的改动。
- 脚本与钩子本身写得对不对：它们各有自证与弄坏开关，当证据用；腿读到脚本与定义对不上时，报的是定义那一句。
- 「一百九十四节判器闭包排除」「第八十节内存记录」这两题另有各自的轮。

## 腿怎么交

- 正推腿：四组各自的「要判的」逐条回「成立 / 不成立 / 推不出」，每条引定义的文件:行与脚本、钩子的文件:行。
- 攻方腿：找照改后的定义做会做错、做不成、或两句打架的具体走法，每一条给出走到哪一步、读的是哪一句、今天的脚本会怎样。
- 要跑的命令写进报告「要主 agent 跑的命令」一节，腿不交任务。
