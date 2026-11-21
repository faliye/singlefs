# ledger-topdown-governance-r2：判决（2026-11-18）

正文 `research/prompts/_ledger-topdown-governance-r2-body.md`，开工快照 `research/prompts/ledger-topdown-governance-r2-snapshot/`。这是这一题的第二轮；第一轮的判决是 `research/prompts/ledger-topdown-governance-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方腿（three-way-attack） | `research/prompts/ledger-topdown-governance-r2-opus-output.md`（sha256 以 532a2e76 起头） | 第 1 到 7 问都回推不出、算打中；要主 agent 跑 16 条命令 |
| 辩方腿（three-way-defense） | `research/prompts/ledger-topdown-governance-r2-sonnet-output.md`（sha256 以 7a65360b 起头） | 第一轮坐实的 12 条都站得住、三处要更正；认下记账的 8 条站得住；修补做偏 2 行、少做 2 行；主 agent 的小修有 2 处该验没验；要跑 2 条命令 |

倒推开工快照（任务 20195，退 0）：

`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r2-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py ledger-topdown-governance-r2 --out /home/fy5090/.local/share/singlefs/work/governance-r2-verify/tree --repository /home/fy5090/code/singlefs`

「✓ 快照 ledger-topdown-governance-r2：清单点名 23 份，倒推出 23 份，sha256 对上 23 份」。腿跑着的时候快照里的文件谁都没改。

引文核对（退 0）：

`python3 scripts/agents/cite-check.py research/prompts/ledger-topdown-governance-r2-opus-output.md research/prompts/ledger-topdown-governance-r2-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/governance-r2-verify/tree --background research/prompts/_ledger-topdown-governance-r2-background.md`

「✓ 核了 73 处引文、82 处裸行号，对上 46 处，没判 109 处（其中裸行号没有引文可比 51 处；逐处列在上面）」，判错 0 处。没判的另 58 处引的是快照之外的文件（调度记录、账本、任务日志、别的脚本）。

两条腿的命令逐条交了，日志摘在 `~/.local/share/singlefs/work/governance-r2-verify/main-agent-logs/`：

| 命令 | 任务 | 结局 | 读数 |
|---|---|---|---|
| 攻 1 喂检出钩子 20 条 | 20176 | 退 1 | 「喂了 20 条，13 条不对」：13 条「打中」的都「退 0（应当 2）」——`cp -b`、`install -b`、`mv --backup=numbered`、`cp -r 草稿/src/.`、`rsync -a`、`cp -r 草稿/src prune-server/`、`ln -sf`、在草稿里建硬链接、往那份硬链接追加、`tar -x`、`git apply`、`patch -p1`、`rm` 已存在的源码；7 条对照都对（`install` 不带 `-b`、`dd of=`、`truncate`、`cp` 进目录照拒） |
| 攻 2 喂写闸 2 条 | 20177 | 退 0 | `rm` 之后同名 Write 放行；整份盖已存在的照拒 |
| 攻 3 写闸拒门禁阶段 | 20178 | 退 2 | 说明里写的是「加 --verify '<被换脚本的 --selftest 命令>' 换上」（对象是 `.claude/gate.d/broad-mutation.sh`） |
| 攻 4 对照：拒登记成要临时服务的脚本 | 20179 | 退 2 | 说明里写的是「不加 --verify 换上（这份脚本在 .claude/hooks/selftest-temporary-service.tsv 里登记成 required…）」 |
| 攻 5 只核不换 e171 改后那一份 `run_directory.py` | 20180 | 退 1 | 「scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py:49 引了 run_directory.device_stream_summaries，可 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/run_directory.py 里没有这个名字」等 2 处 |
| 攻 6 同名模块的探针 | 20181 | 退 1 | 点名 `scripts/other/caller_from_sibling.py:1` |
| 攻 7 `sys.modules.get` 的探针 | 20182 | 退 0 | 「1 处引用，缺 0 处，本次未判 0 处」 |
| 攻 8 `from sys import modules` 的探针 | 20183 | 退 0 | 同上 |
| 攻 9 对照：下标取库 | 20185 | 退 1 | 点名 `caller_by_key.py:7` |
| 攻 10、11 拿造好的日志喂任务服务 | 20186 | 退 0 | 服务推了提醒：「这一趟有 3 条用例本次未跑（日志里 3 行带 NOT_RUN_NO_GPU_ADAPTER 记号）：form_line_start、form_indented、form_after_result_start」 |
| 攻 12 同一份日志喂门禁那一份库 | 20187 | 退 0 | `marker` 行 8 行，「RECORDABLE binaries=1 green=0 withheld=7 markers=8」；多出的 5 行是接在别的输出后面、带颜色码、隔着字、后接制表符四种，加一行 `NOT_RUN_ANYTHING_HERE 不是记号` |
| 攻 13 注释历史探针 | 20188 | 退 0 | 「4 行命中」：第 2 行「两份结果不同时以前面那一份为准」、第 3 行「路径可以前面带 ./，归一之后再比」、第 6 行「键可以前置一段命名空间」与对照第 5 行 |
| 攻 14、15 在仓副本里起规格工具 | 20189、20190 | 退 0、退 3 | 「✗ 取不到工作根（取不到数据根：环境里没设 SINGLEFS_DATA_ROOT，多机配置 …/q4/copy/multi-host.env 不在；不补缺省、不退回家目录），规格没写」 |
| 攻 16 对照：主工作区起 | 20191 | 退 0 | 「✓ 写出 …/q4/spec-from-main.json」 |
| 辩 1 旧串带不带尾空格的成对喂法 | 20193 | 退 0 | 三条都对：主 agent Edit `src/` 下的文件，旧串以空格收尾退 2、不以空白收尾退 0；工具员不以空白收尾退 2 |
| 辩 2 数换上趟数 | 直接跑 | 与它存的输出逐行相同 | 2026-11-18 真换 67 趟、三个会话、窗口两两相交 0 对 |

攻方腿建的探针副本（105M）跑完删了。

## 二、判决

两条腿都有打中的。逐条现查之后：

### 坐实、这一轮改

1. **检出钩子第十四项漏放十三种写法**（攻方腿第 1 问，任务 20176、20177）：带备份选项的 `cp -b`、`install -b`、`mv --backup`（钩子把带备份当成不盖目标）；整目录拷与 `rsync`（落点算成目录）；`ln -sf`、硬链接与往硬链接追加；`tar -x`；`git apply`、`patch`；`rm` 之后同名新建。第一轮修补要挡的「工具员在主工作区改到一半的服务源码」换这几种写法照样做得到。
2. **写闸对门禁阶段的出路写错**（攻方腿第 2 问，任务 20178 与对照 20179）：门禁阶段不在要临时服务的登记表里，说明照「不在表里」那一支写带验证命令换上；而门禁阶段的样本档有要临时服务的（这一轮我交 `crash-point-replay.sh` 的样本档就套了临时服务的包装）。照说明做会回滚。
3. **换上工具把同名不同目录的模块算进来**（攻方腿第 3 问，任务 20180、20181）：这一轮新核的 `from 库 import` 让只按文件名认库的毛病显出来，换 e171 改后那一份 `run_directory.py` 会被另一个目录里同名模块的调用方挡住。`sys.modules.get(…)` 与 `from sys import modules` 两种取法既不核也不列本次未判（任务 20182、20183），辩方腿点到真仓里 `scripts/crash/crash-flow-coverage.py:898` 取的正是要核的客户端库（按线索记）。
4. **规格工具在仓副本里起退 3，出路与「多机配置不拷进副本」相反**（攻方腿第 4 问，任务 20190 与对照 20191）：这一轮把工作根改成只从那份库所在仓的多机配置取，副本里没有多机配置，照约定交了 `SINGLEFS_MULTI_HOST_CONFIG` 也不认。工具员在草稿目录里起它、指主工作区的那一份不受影响，在副本里起的受影响。
5. **任务服务认的记号行形态比门禁那一份库少四种**（攻方腿第 5 问，任务 20186 与 20187）。第一轮判决第 9 条写的是「与门禁那一份库同一个认法」，修补只认三种，辩方腿判这是做偏。门禁那一份库把 `NOT_RUN_ANYTHING_HERE` 这种更长的词也认成记号（同一任务第 8 行），是它那一侧的宽。两边要对到同一个认法，以哪边为准写在第三节。
6. **注释历史判法把讲现状的句子判红**（攻方腿第 6 问，任务 20188）：「两份结果不同时以前面那一份为准」「路径可以前面带 ./」「键可以前置一段命名空间」三句都不是讲过去。第一轮把「面」的例外收窄到只认「所以前面」，收过头了；「可以前」「以前置」这两形原来也没有例外，是这一轮才量到的。
7. **主 agent 给 `.claude/gate.d/doc-text.sh` 两行加的特殊缘由标记该改写**（攻方腿第 6 问、辩方腿第 11 问）：共用约束列的特殊缘由是三类，这两行不在其中；同一段文件头里有不逐字引的写法可以照着改。撤掉标记、改写那两行。
8. **第一轮判决里拿退出码当「哪一道拒的」的同形错不止一句**（辩方腿第 10 问，任务 20193 成对读数）：除了已更正的 `tests/` 那一句，第一轮判决第一节表里「攻 3」对照行的「退 2」同样分不出是第十一道还是旧串以空格收尾那一道拒的；工具员 Edit `src/` 而旧串不以空白收尾的这一条（任务 20193 第三行）才证明第十一道拒 `src/`。结论不变，证据换成这一条。第一轮判决是冻结证据不回改，更正写在这里。
9. **实现员定义第 47 行与证红脚本的取法**（攻方腿第 7 问）：变异表与行键取副本、判法脚本与重测试目标登记表取主工作区，定义第 47 行还写着证红脚本在副本的 `scripts/` 里（按线索记，改之前现读）；副本里新登记的崩溃枚举用例证红时不补 `--release`、行键与验证段对不上这一形攻方腿只给了形状，没有读数。定义那一句照现状改；行键那一形记账。
10. **修补少做的两处**（辩方腿第 11 问）：`.claude/gate.d/doc-process-records.sh:610`、`:612`、`:617` 还有同一种「现行」的叫法（按线索记）；第 3 条里的两种取法。并进这一轮的修补。

### 认下、记账不在这一轮改

11. 换上工具没有锁：辩方腿现数今天真换 67 趟、三个会话轮流用、最短间隔 8 秒、窗口两两相交 0 对、退 9 的 0 趟（任务外直接跑的只读脚本，输出与它存的逐行相同）。记账站得住，补一条触发条件：两趟窗口相交、出现退 9、或任务服务推上一趟还在换的提醒，三样有一样就当场改。
12. 第一轮判决第 4 条的标题说宽了（辩方腿第 8 问）：带包装的验证命令在第二台过、在本机退 5，是两台结局不同，不是一律用不了。改法不变。
13. 第一轮记账的另几条辩方腿都判站得住；其中第 15 条（换上工具开关整份复跑）这一轮已兑现，第 19 条（定义与表头次序）已改，第 17 条的前提今天不成立（两台上限相同，按线索记）。账本侧四处过时句（辩方腿点了现行账本第 20、27、186、498 行）由主 agent 改去向行。
14. 各批工具员写明没做的几件，辩方腿判不算这一轮缺口的：检出钩子不判 `mv` 挪走（`rm` 那一半并进第 1 条）、认不出类型的子 agent 不判、`--nocapture` 提一句。照旧记账。换上工具的脚本规格没排除两个服务的构建清单：辩方腿的理由是换上工具改名换新 inode、没有改到一半；站得住，不改。
15. 证红命令缩域那一形今天没有任务跑过（辩方腿第 8、11 问）：定义改了而没量。记账：下一次有缩域证红时核。

### 不采

16. 攻方腿提的把记号认法整个搬成门禁那一份库的写法：门禁那一份库把更长的词也认成记号，直接搬会把这个宽带进任务服务。采的是「两边对到同一个认法」，不是照抄哪一边。

### 核不动的

1. 第 9 条里行键对不上那一形，只有形状。
2. 辩方腿抽查任务号 11 个与记录相符，主 agent 没有逐个复查它抽的那几个。

### 这一轮的结局

第二轮有打中的，坐实 10 条。照治理文件轮的收法：改完再攻第三轮（最后一轮），攻这一轮的修补与写明按线索记的地方；第三轮改的修补在判决里标零轮。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 标 |
|---|---|---|---|
| 检出钩子第十四项：带备份选项的不再当成不盖目标；落点是目录时按「目录里已存在的同名文件」判；`ln`、`tar -x`、`git apply`、`patch`、`rsync`、`rm` 进它认的写法；硬链接那一形判得出就判、判不出的在文件头写明 | 第 1 条 | 攻方腿 | 零轮：攻方腿的 20 条喂法当样本，13 条预期翻成拒、7 条对照不变 |
| 写闸拒绝说明：门禁阶段（`.claude/gate.d/` 顶层的脚本）单列一支，写「不带 --verify 换上，样本档另交一条」 | 第 2 条 | 攻方腿 | 零轮：任务 20178 那一条喂法当样本 |
| 换上工具：认库从只按文件名改成按调用方真解得到的那一份（同目录优先、`sys.path` 插入的目录其次；解不出的列本次未判）；`sys.modules.get(…)` 与 `from sys import modules` 两种取法照下标那一形认，认不出的列本次未判 | 第 3、10 条 | 攻方腿 | 零轮：四棵探针小树（`q3/stem`、`q3/get`、`q3/fromsys` 与任务 20180 那一条）当样本 |
| 规格工具：工作根照多机配置取时，认 `SINGLEFS_MULTI_HOST_CONFIG` 指到的那一份（只认它指的文件里的值，不认 `SINGLEFS_WORK_ROOT` 与 `SINGLEFS_DATA_ROOT`）；取不到时的出路改成与「多机配置不拷进副本」相符的说法 | 第 4 条 | 攻方腿与主 agent | 零轮：任务 20190 那一形当样本。改之前先判这样改会不会把第一轮堵上的口子（命令前设环境变量就能改 `--out` 能写到哪）重新打开：指到一份自己写的多机配置同样改得了工作根，判不过就改成「副本里起一律拒、出路写在主工作区或草稿目录里起」 |
| 任务服务与门禁那一份库对到同一个认法：记号前后以空白或行界为界、整词相等才算，行内任一处都认；两边各改到这一条，各配同一份样本日志 | 第 5、16 条 | 主 agent | 零轮：攻方腿那份 8 行的日志当样本，两边都该认 7 行、不认更长的那一行 |
| 注释历史判法：「以前」后接「面」的例外放回「前面是所、可、以」这几个字的写法之外再想一遍——改成认「以前」作时间词的正面形态（前面是句首、标点或空白，或后面接「是」「的」「只」「曾」「这」这类），别的不判；改前先在真仓 872 行命中上量新旧两种判法各判多少、差的行逐行看 | 第 6 条 | 主 agent | 零轮：攻方腿第二轮的探针与第一轮的两句红样本一起当样本 |
| `.claude/gate.d/doc-text.sh` 那两行撤掉特殊缘由标记、改写成不逐字引的说法 | 第 7 条 | 两条腿 | 量过的是现状 |
| 实现员定义第 47 行照证红脚本今天的取法改；`.claude/gate.d/doc-process-records.sh` 另三行的叫法 | 第 9、10 条 | 两条腿 | 改之前现读 |

这一批都是治理文件，没有要写进产品代码的零轮改法。每条零轮改法的样本先证红再换上。

## 交用户的

没有要用户定的岔路。告知两件：第一轮给检出钩子新加的那一道，换十三种写法就绕得过（第二节第 1 条），这一轮坐实了、下一批补；任务服务认「本次未跑」记号的认法与门禁那一份库还差四种形态（第 5 条）。

## 这一轮被判过的文件

- research/prompts/_ledger-topdown-governance-r2-body.md：这一轮的正文
- research/prompts/ledger-topdown-governance-r1-main-verification.md：辩方腿复核了它：坐实的 12 条与记账的 8 条站得住，拿退出码当证据的同形错不止一句（第二节第 8 条），第 4 条标题说宽了（第 12 条）
- task-server/task_server.py：判了认记号行的三种形态：比门禁那一份库少四种（第二节第 5 条）
- task-server/task_server_lib.py：判了异常表那一行与新开关：现状句成立
- task-server/task_server_selftest.py：判了新自证格喂的形态：只有三种认的与四种不认的，没有门禁那一份库认的另四种（第 5 条）
- task-server/README.md：判了不带 `--nocapture` 读不到的那一句：成立
- scripts/gate/hook-swap.py：判了新认的两种引法：同名模块被算进来（第 3 条）、两种取法漏核（第 3、10 条）；没有锁那一条记账并补了触发条件（第 11 条）
- scripts/gate/make-swap-spec.py：判了工作根与临时文件：仓副本里起退 3、出路写反（第 4 条）；临时文件那一半成立
- scripts/gate/README.md：判了第 21 行的说法：随第 4 条的改法再核
- .claude/hooks/write-guard.sh：判了拒绝说明分两种出路的那一段：门禁阶段走错了一支（第 2 条）；`rm` 之后同名新建放行（第 1 条）
- .claude/hooks/bash-command-detector.sh：判了第十四项：十三种写法漏放（第 1 条），七条对照都对
- .claude/agents/tooling-writer.md：判了第 3、7 步改后的说法：与写闸今天的行为一致，随第 2 条的改法补门禁阶段那一支
- .claude/agent-common.md：判了写闸第十一道那一句与新加的第十四项一行：成立
- .claude/agents/implementation-writer.md：判了证红命令的起点：第 3a 步成立，第 47 行还写着旧的取法（第 9 条）
- .claude/agents/investigator.md：判了证红那一句：与实现员定义同改，成立
- scripts/gate/comment-history-check.py：判了「面」的例外收窄：三句讲现状的句子被判红（第 6 条）
- .claude/gate.d/doc-text.sh：判了文件头新写的行：两行的特殊缘由标记该改写（第 7 条）
- .claude/gate.d/crash-point-replay.sh：判了第 294 行改后的指针：辩方腿现查对得上
- .claude/gate.d/doc-process-records.sh：判了三行样本说明：改了的成立，另三行同一种叫法少改（第 10 条）
- scripts/hosts/run-with-memory-cap.sh：判了第 73 行改后的指针：辩方腿现查对得上
- scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/list_check.py：判了第 416 行帮助串：成立
- .claude/gate.d/break-switch-replay-files.tsv：判了这一轮改的五行：与换上之后各份的开关数相符

## 回看决策

不涉及决策：这一轮判的是定义、钩子、门禁、脚本与任务服务的修补，没有拿依据去推翻或确立哪一条决策分项。
