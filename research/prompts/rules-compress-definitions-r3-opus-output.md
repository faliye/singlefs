# rules-compress-definitions-r3 攻方腿报告

日期：2026-11-16。轮名 rules-compress-definitions-r3，第三轮（最后一轮）；前几轮判决 `research/prompts/rules-compress-definitions-r1-main-verification.md`、`research/prompts/rules-compress-definitions-r2-main-verification.md`。这条腿没写模型、没有复跑脚本；草稿目录 `~/.local/share/singlefs/work/rules-compress-definitions-r3-opus/` 里只有只读的辅助脚本（`tools/` 下：指针核对、按句覆盖率差分、批次代号判据的另写一份、定义形状两格的另写一份）与它们的文本输出，另有两份喂钩子用的输入文件（见「要主 agent 跑的命令」）。

开工核对（量的）：仓根 `sha256sum -c research/prompts/rules-compress-definitions-r3-snapshot/judged-sha256.txt` 15 行全对；在合稿树目录 `research/prompts/rules-compress-r3-merged/` 里 `sha256sum -c ../rules-compress-definitions-r3-snapshot/merged-tree-sha256.txt` 56 行全对。

## 各格判定一览

| 编号 | 格 | 打在哪 | 判定 | 轻重 |
|---|---|---|---|---|
| 攻-1 | 一（用户定案第 25 条落没落对） | 共用约束第 11 行没留原件原话（「直接改」改成「直接做」、「不派发」改成「不派」、粗体去掉、句子并进一条长句）；矛盾总表入口-4 却写共用约束「各留原话」 | 打中 | 换上之前必须改（改一行） |
| 攻-2 | 二（修补引出的新问题） | 第三版共用约束第 39、96 行这一轮新加的批次代号例子写在「」里、不在反引号里，正是写闸第七道与检出钩子 ⑰ 拒的写法；原件同一句的例子都在反引号里 | 打中（按判据库另写一份量出 7 处命中，钩子本身没喂，待主 agent 跑） | 换上之前必须改（放进反引号） |
| 攻-3 | 二 | 调查员「查到就修」：第 6 步新句只管 `--cwd` 与 `<根>`，实现员第 1d 步要它写的验证段那一条 crates-mutation-replay 用的是 `<快照>` 占位，调查员这一支没有验证段与快照；同一处实现员第 4 步的 static-check 丢了原件「在主工作区改的填仓根」 | 打中（二义，做法有两种） | 换上之后可以再改 |
| 攻-4 | 二、三 | 实现员第 3 步把原件「（第 1d 步那几条要起的 scripts/ 与 .claude/gate.d）一并拷」改成「建副本时已…带上」的前提句，而第 15 行只在新写崩溃枚举用例、放量分项、拒算测试时才叫它带 `--with`；默认闭包里没有 `scripts/mutation/prove-red.sh` | 打中（量了闭包） | 换上之后可以再改 |
| 攻-5 | 三 | 共用约束第 58 行「名单以 TEXT_TOOLS_RUN_DIRECTLY 为准」从原件只管文本工具那一小段扩到整张「直接跑的只有」清单，而那张表里有 `task-server/task` 的 submit、wait、cancel、clean | 打中（读法有两种，钩子兜得住） | 换上之后可以再改 |

没打中的范围在末尾「没打中的形状」：重点四处（调查员验证命令落在哪棵树、规则怎么读与开工先读、进程停法对钩子 ⑥、共用约束对各道闸的转述）逐处写了查过什么。

## 格一：第二轮修补对不对（用户定案第 23 到 26 条落的那一句）

### 攻-1 用户定案第 25 条：共用约束那一处不是原话

定案：
- `records/2026-11-16-三方豁免取消与存量补轮.md:213`：「三处原话各留，不合并」

原件那一处：
- `.claude/agent-common.md:8`：「**简单任务不派发，自己直接改**，例如修改一下、查询一下、计划一下等，任务单一流程简单的任务。」

第三版：
- `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:11`：「简单任务（修改一下、查询一下、计划一下这类单一、流程简单的）自己直接做，不派」，同一行接着并进了规划、model 与 effort、三次失败三件事

矛盾总表把这一处记成原话照留：
- `research/prompts/rules-compress-r3-merged/contradictions.md:18`：「与共用约束「简单任务不派发，自己直接改」各留原话」

找过的地方：第三版共用约束全文 `grep -n '自己直接改'` 0 处（量的）；入口与 `CLAUDE-merged.md` 两处是原话（修稿报告 ② 的落点，主 agent 第三版现读也核过入口那一句）。

为什么算打中：定案要的是三处原话各留；另两处照留，共用约束这一处被改写了用字（「改」→「做」、「不派发」→「不派」）、去掉了粗体、并进了一条四件事的长句。总表入口-4 的说明与树里的字对不上，查总表的人会以为这一处已经照定案落了。做法差别小（「做」比「改」宽，查询、计划本来也不是「改」），所以只算落偏，不算改了判据。

子 agent 照第三版做会做成什么：这一条管的是主 agent，做法基本不变；差在定案的字面没落、总表写的不是真话。

四句：分辨（原件与第三版字面不同）；看得到（同一行）；满足格一「定案落偏」的分句；没有跑前条款给的改法。

改法（纸面、零轮）：第 11 行开头放回原件那一句原话（含粗体与例子），其余几件另起；或者在总表里如实写「改了措辞、交用户」。

什么现象会推翻：用户另有原话说「原话」只要求不合并、不要求逐字（`grep -n '原话' records/2026-11-16-三方豁免取消与存量补轮.md` 命中 7 行：第 76、108、129、141、213、222、227 行，只有第 213 行是第 25 条，其余说的是别的事，量的）。

## 格二：修补引出的新问题

### 攻-2 第 39、96 行新加的批次代号例子不在反引号里，写闸与检出钩子照拒

第三版这一轮新加的两处（补丁 `research/prompts/rules-compress-r3-merged/diff-from-r2.patch` 第 280、322 行的 `+` 行）：
- `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:39`：句中把天干编号的两个批号写在「」里（「第十五批」后接天干与数字、「批 10」后接天干那两个）
- `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:96`：写闸那一项列的四个例子同样写在「」里

原件同一条只用反引号里的例子：
- `.claude/agent-common.md:54`：「批次不写字母代号（`5a`、`9g`、`5g-rs`、`24c-1`、`SC02`）」

钩子与判据库（文件头今天的原文）：
- `.claude/hooks/write-guard.sh:42`：「库按后缀只判 .md、.tsv、.txt，不认放行标记，仓外不判」
- `.claude/gate.d/lib-forbidden-notations.py:54`：「上下文规则（batch_code_is_not_code）放过：反引号代码段里的」
- `.claude/gate.d/lib-forbidden-notations.py:177`：天干编号那一条判据行的正则，数字一段收汉字数字
- `.claude/hooks/bash-command-detector.sh:302`：⑰ 不判「内容不是字面的（cat 文件、cp、mv、别的命令的输出）」

量的（照判据库四条判据行另写一份 `tools/batch.py`，只读文本、不导入仓里的库；上下文只放过反引号代码段与紧挨在引号、逗号、分号、等号之后的）：第三版共用约束第 39 行 2 处、第 96 行 5 处命中（天干编号 4 处、短横序号 1 处、两位数字加字母 1 处、挨着批次词 1 处）；原件共用约束与 14 份定义、第三版 14 份定义都是 0 处。这是推的：钩子本身没喂，命令在「要主 agent 跑的命令」第一、二条。

为什么算打中：第二轮攻-H 报的是「共用约束没写写闸会拒天干批号」，第三版照它补了，补的方式是把被拒的写法原样写进 `.md`、不加反引号；写闸第七道判 Write 的整份内容与 Edit 的新串，检出钩子 ⑰ 判 Bash 里的字面写入。照第三版做会做成：
1. 换上时用 Write 写 `.claude/agent-common.md`，或用 `cat > … <<'EOF'` 这类字面写入，被拒（只有 `cp`、`mv` 换文件不判）；
2. 换上之后，工具员第二种活或主 agent 用 Edit 改第 39、96 行（新串里带着这两行），照拒；
3. 照这两行抄例子写进别的 `.md`（记录、判决、规则）的人照拒——第二轮攻方报告就是这样被 ⑰ 拒的（`research/prompts/rules-compress-definitions-r2-opus-output.md:166`：「本报告第一次落盘时我在攻-H 这一行引了钩子那两个例子，检出钩子 ⑰ 当场拒了」）。

不归门禁兜：判据库自己写批次代号一形态「还没有 doc-text.sh 的格」（`.claude/gate.d/lib-forbidden-notations.py:93`），提交时不会红，只在下一次写的时候被拒。

四句：分辨（原件那一句能写进仓，第三版那两行不能用 Write / Edit 写进）；看得到（写闸判的就是写进去的字）；满足格二「修补引出的新问题」，也是格三「照第三版做与照原件做做法不同」；没有跑前条款给的改法。

改法（纸面、零轮）：两行里的例子放进反引号（原件就是这样写的），或改成说明（「天干编号的批号」「挨着批字的数字加字母」），不写例子本身。

什么现象会推翻：主 agent 喂写闸那一条（下面第一条）退 0、标准错误里没有「批次代号」。

### 攻-3 调查员「查到就修」：`<快照>` 占位与 static-check 的根

第二轮攻-A 的修补落在两句：
- `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:28`：「验证改动的命令照第 6 步填它改的那棵树，不填修之前建的副本」
- `research/prompts/rules-compress-r3-merged/dot-claude/agents/investigator.md:33`：「这几条命令的 `--cwd` 与命令里的 `<根>` 写主工作区仓根」

调查员第 6 步叫它照实现员第 1d、2、3、3a、3b、4 步写。实现员第 1d 步要新写的变异行各写两条证红命令，第二条用的不是 `<根>`，是 `<快照>`：
- `research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:44`：「`<快照>` 原样写这个占位、主 agent 照第 1 步打出的快照路径填」
- 同一份第 47 行（第 3a 步）：「这一行在验证段第 2 步要在全域上证过一次，即第 1d 步验证段用的那一条」

`<快照>` 是合入后验证留下的那一棵：
- `research/prompts/rules-compress-r3-merged/dot-claude/main-agent.md:57`：「主 agent 经服务交合入后验证 `scripts/gate/merge-and-verify.sh`：在主工作区的快照上打这一批全部补丁」

调查员不交补丁，没有这一趟、也就没有快照。第 6 步新句点名的是 `<根>`，没说 `<快照>` 怎么办，读得出两种：
- 读法甲：照第 1d 步「原样写这个占位」写 `<快照>`，主 agent 手里没有可填的快照，这一条交不出去；调查员缩过域的证红「判定不算数」，要靠这一条在全域上补判，补判就落空，只剩第 5 步那一条 `--item` 碰巧点到同一行时才补上。
- 读法乙：把 `<快照>` 也当 `<根>` 换成仓根，在主工作区上全域判。
两种读法交给主 agent 的命令不同。

同一处的第二件：实现员第 4 步 static-check 的根。
- 原件 `.claude/agents/implementation-writer.md:40`：「（在主工作区改的填仓根；」
- 第三版 `research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:50`：「在副本上跑的 `python3 scripts/mutation/crates-mutation-rows.py static-check <副本根>`」

用户定案第 1 条之后实现员一律在副本里改，这半句对实现员没用了，删掉合理；但调查员第 6 步照第 4 步写，主工作区改的那一支正是它要的。调查员第 1 步那一句管得住（static-check 是验证改动的命令），所以做不错，只是第 4 步字面对调查员给的是「副本根」，它手里有修之前与修之后两份副本。

四句：分辨（甲乙两种读法交出的命令不同）；看得到（第 33 行与实现员第 44、50 行）；满足格二「新写的句子读不读得出两种意思」；没有跑前条款给的改法。

改法（纸面、零轮）：调查员第 6 步那一句补「第 1d 步验证段那一条的 `<快照>` 也写主工作区仓根（这一支没有验证段）；第 4 步的 static-check 根写仓根」。

什么现象会推翻：第三版别处写明调查员这一支有快照或不写验证段那一条（我在合稿树 `grep -rn '查到就修'` 了定义、共用约束、入口与技能，只有入口第 47、51 行、派发技能第 14 行、实现员第 15、31、46 行、调查员第 3、15、16、23、28、32、33、37、45 行与共用约束第 109 行，共 16 行，量的；没有一处提快照）。

### 攻-4 实现员第 3 步：「一并拷」改成了「已…带上」

- 原件 `.claude/agents/implementation-writer.md:36`：「在草稿目录建仓副本，拷哪几样照共用约束「写」一节建副本那一条（第 1d 步那几条要起的 `scripts/` 与 `.claude/gate.d` 一并拷）」
- 第三版 `research/prompts/rules-compress-r3-merged/dot-claude/agents/implementation-writer.md:46`：「第 1d 步那几条要起的 `scripts/` 与 `.claude/gate.d` 建副本时已照共用约束「建仓副本」的 `--with` 带上」
- 第三版同一份第 15 行（建那一份副本的地方）：「新写崩溃枚举用例、放量分项或要拒算测试的另带 `--with-gate-tables` 与 `--with scripts=<为什么>`」
- 第三版共用约束 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:35`：「要主 agent 在副本里交带 `--item` 的门禁与崩溃放量入口、起仓里脚本的，逐条加 `--with scripts=<为什么>`」

第三版把原件第 3 步的一条指令（拷）改成了一句前提（已带上）。实现员唯一建副本的地方是第 15 行，那一句把 `--with` 限在三种情形；不写新崩溃枚举用例、不拒算的实现员照第 15 行建出来的副本，第 3 步又告诉它「已带上」。共用约束那一条管得住（证红要在副本里起 `scripts/mutation/prove-red.sh`，算「起仓里脚本的」），所以不是丢了规则，是第三版两句给了相反的提示：第 15 行像是穷举，第 46 行说已经做了。

量的：`timeout 200 python3 scripts/hosts/lib_repository_copy.py entries /home/fy5090/code/singlefs --test-inputs`（直接跑名单上的 `entries`，不拷不编）末行 `REPOSITORY_COPY entries=35 started=…`；35 条里 `scripts/mutation` 0 条，`.claude/gate.d/formal-verification.sh`、`crates-mutation-replay.sh`、`crash-point-replay.sh` 0 条，`refused-test-registry`、`crash-case-check`、`crates-mutation-rows` 0 条（`grep -c` 数的，输出在草稿目录 `closure.txt`）。`scripts/hosts/minimal-copy.py:16` 写的 `--with-test-inputs` 是「连测试代码一起扫」，不拷证红脚本本身。

照第三版做会做成什么：实现员只带 `--with-test-inputs` 建副本，交回的 `task-server/task submit --cwd <副本> -- bash scripts/mutation/prove-red.sh …`、第 3c 步的 `refused-test-registry.py list <根>`、第 4 步在副本上的 static-check 在副本里都找不到脚本，主 agent 交上去当场失败、新派一个实现员重建副本（`minimal-copy.py` 对同一个副本目录带 `--with` 重跑即可）。错误看得见、不会判错，所以轻；原件这里是明写的。

四句：分辨（原件第 3 步明令拷，第三版第 15 行在常见情形下不带）；看得到（同一份定义第 15、46 行）；满足格二「新写的句子读不读得出两种意思」与格三；没有跑前条款给的改法。

改法（纸面、零轮）：第 15 行改成「证红、static-check 与第 1d 步要在副本里起的脚本照共用约束「建仓副本」带 `--with scripts=<为什么>`（起门禁阶段的另带 `--with-gate-tables`）；新写崩溃枚举用例、放量分项或要拒算测试的一样要」，第 46 行的「已」改成指令。

什么现象会推翻：`lib_repository_copy.py` 的闭包对某种实现员副本会自动带上 `scripts/mutation/`（例如测试源码里字面读了它），上面那条命令换成带改动的副本根再算一次能看到。

## 格三：能不能换上（原件里带「只」「一律」的句子对到第三版）

### 攻-5 「名单以 TEXT_TOOLS_RUN_DIRECTLY 为准」管的范围扩大了

- 原件 `.claude/agent-common.md:66`：这个指针只挂在「定义点名的读写 kb 与材料的文本工具」那一小段后面的括号里，「名单是 `.claude/hooks/lib_project_code_runs.py` 的 `TEXT_TOOLS_RUN_DIRECTLY`」
- 第三版 `research/prompts/rules-compress-r3-merged/dot-claude/agent-common.md:58`：整条「子 agent 直接跑、不经服务的只有：…」列完之后另起一句「名单以 `.claude/hooks/lib_project_code_runs.py` 的 `TEXT_TOOLS_RUN_DIRECTLY` 为准」

那张表今天的原文：
- `.claude/hooks/lib_project_code_runs.py:1095`：`TASK_SERVER_CLIENT: ("first", TASK_SERVER_CLIENT_SUBCOMMANDS),`
- `.claude/hooks/lib_project_code_runs.py:1060`：`TASK_SERVER_CLIENT_SUBCOMMANDS = frozenset({"submit", "confirm", "promote", "wait", "status", "cancel", "log", "clean"})`
- `.claude/hooks/lib_project_code_runs.py:1096`：`"task-server/control": ("first", frozenset({"check", "records", "bookings", "key-stats"})),`
- 子 agent 另有一道：`.claude/hooks/lib_project_code_runs.py:1062`：`SUBAGENT_TASK_SERVER_SUBCOMMANDS = frozenset({"status", "log"})`

第三版第 58 行的「为准」读得出两种：甲，只管文本工具那一段（原件的意思）；乙，整张「直接跑的只有」以那张表为准，而那张表里有 `task-server/task` 的 submit、wait、cancel、clean、confirm、promote 与 `task-server/control check`，和同一节第 55 行「子 agent 不起 `task-server/task` 的 `submit`、`wait`、`cancel`、`clean`」、第 69 行「不起 `task-server/control`」相反。钩子照拒（项目代码闸对子 agent 另用第 1062 行那一份），所以做不坏，只会多被拒一次。轻。

改法（纸面、零轮）：把这一句挪回文本工具那一段的括号里，或写「文本工具一段的名单以…为准；task-server/task 子 agent 只起 status 与 log」。

什么现象会推翻：第三版别处写明这句只管文本工具（我在第三版共用约束 `grep -n 'TEXT_TOOLS_RUN_DIRECTLY'` 只命中第 58 行，量的）。

## 没打中的形状

重点四处逐处查过的（没打中的说法只在这些范围里成立）：

1. 调查员「查到就修」的验证命令落在哪棵树（除攻-3 那两件）：第 28、33 行把 clippy、build、动到的测试目标、命名检查与 `--item` 那几条落到主工作区仓根，与原件第 7 步的意思（判改后那棵树）一致；「证红的 `prove-red.sh` 不认主工作区」与脚本文件头一致（`scripts/mutation/prove-red.sh:65`：「--copy 指主工作区应当以「主工作区」为由拒绝」）；改完新建的副本带得上改动（`minimal-copy.py` 文件头：rsync 按内容比、拷工作区文件，不取 HEAD）。对着交回闸文件头：⑤ 只判实现员、执行员、工具员三类（`.claude/hooks/handback-guard.sh:33`：「⑤ implementation-writer / experiment-runner / tooling-writer」），调查员这一支不判，共用约束第 109 行给调查员写「`<根>` 写仓根」不会被它拒；实现员在 `minimal-copy.py` 副本里改、两条都写 `<快照>`，与 `.claude/hooks/handback-guard.sh:47`：「两条都写 `<快照>` 才算，只在这一次的改动全落在这类副本里时认」一致。实现员第 31 行把调查员指到调查员第 6 步，次序与两份定义的步号都对得上（第 1d、2、3、3a、3b、4 步在第三版实现员里都在）。
2. 「规则怎么读」改回原话之后：第三版第 17 行与原件第 22 行那一句逐字相同（`grep -nF` 两边各命中 1 行，量的）。合稿共用约束与 14 份定义里「`文件`「节」」指针，按第三版合稿树解（被并掉的 8 份规则换到并进去的那一份、`.claude/` 下取 `dot-claude/`）全部指得到：标题逐字命中之外只有共用约束第 24、59 行两处「那一条」指的是加粗条目或正文（`tools/ptr.py`，量的）；目标文件里没有重名标题（`tools/dup.py`，量的）；14 份「开工先读：」行的指针集合与原件按并法换名之后逐个对过，差别三处、节都在：事务员那一处换成了并进 `machine-first.md` 的新节名；先行调研员原件整份点名 `verify-before-claiming.md`，第三版点名并进去的那一节连同三个子小节；工具员另加了新建参考件 `verification-reference.md` 的两节（`tools/kaigong.py`，量的；它另报的执行员一行是第 7b 步里「照…那一节写」被当成指针，不是差）。指到子 agent 读不到的文件：定义与共用约束里没有指 `CLAUDE.md` 的指针，指被并掉的 8 份规则的 0 处；指 `.claude/main-agent.md` 的两处（共用约束第 51、64 行）说的都是主 agent 的做法。事务员「规则只读三样」与「一律整份读」的关系修稿报告第 ④ 节第 5 条已列，这里不重报。
3. 进程停法：工具员第 5 步、共用约束第 67 行与第 80 行 ⑥ 对着检出钩子文件头 ⑥（第 165 到 199 行）与它代码里的注释（`.claude/hooks/bash-command-detector.sh:4114`：「os.killpg(<进程组号>, …) 与 os.kill(-<进程组号>, …) 放行，在循环里调也放行」）逐句对过：许的、不许的都在钩子的放行与拒绝里；第三版只写了 Python 循环里的 `os.killpg`，钩子另放行循环里的 `os.kill(-组号)`，第三版比钩子窄、照它写不会被拒，不算打中。工具员第 5 步点名 `proc.py stop` 与共用约束「`proc.py` 只给主 agent」那一处，第二轮攻-C 已报、修稿报告第 ④ 节第 4 条挂着，不重报。钩子给脚本留的 `# process-safety:own-scope` 标注（`.claude/hooks/bash-command-detector.sh:194` 那一段）原件与第三版都没写，不是这一轮的差。
4. 共用约束对各道闸的转述：对过定时闸文件头开头与「拒的 / 不拒的」两段、弹窗闸「判法」一段、交回正文闸 ①②③⑤⑧、完工清理闸「要留的交付物」与「删不掉的」两段、范围声明闸开头与「经 task-server 提交」「放宽检查的提交标识」「指定范围的崩溃域」三段、检出钩子 ⑪⑫⑮⑯⑰⑱⑲⑳、写闸五与七、项目代码闸两张直接跑名单（`lib_project_code_runs.py` 第 1076 到 1113 行）。除攻-2、攻-5 外没找到照第三版写会被拒、或照它不写会漏的。交回正文闸那一项写「主工作区或 git worktree 副本」比钩子窄（钩子另判 `minimal-copy.py` 副本、照仓根判），但原件同样、各定义另有命名检查的写法，不算这一轮的差。共用约束第 65 行「带它的提交只由主 agent 起，弹窗要用户确认」与定案第 24 条：第三版提交技能同样拿「弹窗要用户确认、对话不自己决定」说钩子那一次弹窗（`research/prompts/rules-compress-r3-merged/dot-claude/skills/crash-test/commit-stages.md:20`），读不出要主 agent 另问一次，不算。

另做的、没打中的：
- 句子覆盖：原件共用约束里带「不」「只」「一律」「除外」「先」「才」的句子，按 4 字片段对第三版整棵树算覆盖率，低于 0.75 的 132 句逐句看过（`sdiff-common.txt`）；14 份定义对它自己的第三版加第三版共用约束，低于 0.6 的 18 句逐句看过（`sdiff-defs.txt`）。没落的只有钩子的认法细节（定案第 7 条许下沉）与攻-3 那半句；共用约束原件第 81 行「结不结束、怎么处置由主 agent 定」第三版第 69 行没留，那一条前文已是「接着做定义里还能做的」，不改做法。
- 换上之后两格门禁的模拟（照 `.claude/gate.d/code-tooling.sh` 文件头的判据另写一份 `tools/shape.py`，只读文本）：agent-definition-shape ⑤ 定义、共用约束与入口里逐字抄规则的长行 0 处；⑥ 入口派发表点名的定义名与 14 份定义两向相等（另点名的 `cfg`、`gpu` 在 `NOT_DEFINITION_NAMES` 里登记过，`crash-point-replay` 是门禁阶段名）；agent-write-scope ⑧ 14 份都有一行以「开工先读：」起头。
- 用户定案第 23、24、26 条在这一组的落点：24 在共用约束第 59 行与 `dot-claude/rules/verification.md` 第 38 行一致；23、26 不落在这一组（三条三方腿定义只指过去）。

## 要主 agent 跑的命令

两条都是喂写闸（判一次 Write，不写任何文件），核攻-2。输入文件是这条腿在草稿目录里用 python 照第三版与原件的共用约束各生成的一份 hook JSON（`tool_name` 是 Write，`file_path` 是 `/home/fy5090/code/singlefs/.claude/agent-common.md`，没有 `agent_type`，按主 agent 判）；sha256：`621922ef9ec455bfb42f0e8ef5403cf22f134144981e4dec24b060d54eaec0b2  write-agent-common-merged.json`，`ec0c9cadb77fb1d21f44fc6e44f759f5262bcd97f79052b42f9f2c1bb5acfd7c  write-agent-common-original.json`（量的）。在仓根起。

1. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash -c 'bash .claude/hooks/write-guard.sh < /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r3-opus/write-agent-common-merged.json'`
   - 看：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=`；`task-server/task log <号> --all` 里写闸的拒绝说明。攻-2 成立：`exit=2`，说明里有「写了批次代号」、点名第 39、96 行。攻-2 不成立：`exit=0`、没有这一句。
2. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash -c 'bash .claude/hooks/write-guard.sh < /home/fy5090/.local/share/singlefs/work/rules-compress-definitions-r3-opus/write-agent-common-original.json'`
   - 对照：原件同一份内容。看同上；预期 `exit=0`。要是也退 2，看拒绝说明是不是别的道（那就只比两条说明里「批次代号」那一道有没有）。退 0 而标准错误写「没判」「判不了」的，两条都按没判算。

两条都不改仓、不编译，不用范围声明（不是起验证的命令）。

## 能不能换上

一句话：第三版这一组可以换上，但换上之前先改两处——共用约束第 11 行放回用户定案第 25 条要的原话（攻-1），第 39、96 行的批次代号例子放进反引号或改成说明（攻-2）；攻-3、攻-4、攻-5 是读法上的毛病，钩子与失败的任务兜得住，换上之后可以再改。

什么现象会推翻这个结论：主 agent 喂写闸那两条，第三版那一份退 0（攻-2 不成立，只剩攻-1 一处要先改）；或者用户说第 25 条的「原话」不要求逐字（攻-1 不成立）；或者第三版别处有我没找到的、给调查员定 `<快照>` 怎么填、给实现员定副本必带 `scripts/` 的句子（攻-3、攻-4 不成立）。

## 这条腿自己的限度

- 钩子会不会拒，攻-2 是按判据库的判据行另写一份算的（推的，待主 agent 跑上面两条），我的那一份只放过反引号代码段与紧挨在引号、逗号、分号、等号之后的，库里另有算式那一条上下文规则，对这 7 处不适用（前一个字都是「」）。
- 句子覆盖率是按 4 字片段数的，低于阈值的逐句看了，高于阈值的没逐句看；14 份定义里两版逐字相同的句子没再读。
- 指针核对与定义形状两格是照门禁文件头另写的只读脚本，不是门禁本身；门禁的真实结论要在换上之后由主 agent 跑。
- 只读了检出钩子文件头第 160 到 352 行与 ⑥ 那一段代码注释，项目代码闸只读了两张直接跑名单，交回正文闸读了文件头第 1 到 383 行，完工清理闸读了第 1 到 125 行，范围声明闸读了第 1 到 60 行，写闸读了第 28 到 46 行。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；改法都只在纸面上推过、被攻过零轮。
- 没往统一任务服务交任务，没跑钩子与门禁（命令在「要主 agent 跑的命令」一节）；直接跑的只有只读命令、`scripts/hosts/lib_repository_copy.py entries`（直接跑名单上的那一形）、`scripts/hosts/work-directory-ownership.sh claim`、`scripts/kb/replace-once.py`（改这份报告自己）与草稿目录里自己写的只读 python。
- 没建仓副本与编译目录，草稿目录里只有文本输出、辅助脚本与两份喂钩子的输入文件（主 agent 跑那两条要用，留着）。
- 没读禁读清单里的文件（正推腿的报告、模型目录与草稿目录，别的四题第三轮各腿的产出与草稿目录）。
