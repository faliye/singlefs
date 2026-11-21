# ledger-topdown-governance-r3：判决（2026-11-18）

正文 `research/prompts/_ledger-topdown-governance-r3-body.md`，开工快照 `research/prompts/ledger-topdown-governance-r3-snapshot/`。这一题的第三轮，也是最后一轮；前两轮的判决是 `research/prompts/ledger-topdown-governance-r1-main-verification.md` 与 `research/prompts/ledger-topdown-governance-r2-main-verification.md`。这一轮改的修补都标零轮，不再开第四轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方腿（第 1 到 7 问） | `research/prompts/ledger-topdown-governance-r3-opus-output.md` | 第 2 问成立；第 1、3、4、5、6、7 问不成立（打中）；撤回三个方向：逐种加写法不当主修法、「认出副本」整个方向、两处各写一份认法 |
| 辩方腿（第 8 到 11 问） | `research/prompts/ledger-topdown-governance-r3-sonnet-output.md` | 第二轮坐实 10 条、记账 5 条都「判决站得住」；第 4 条判决当时看得出副本带 `.git`；第 10 问以工具员做成的现状（库认一族、服务认一个）为准；第 11 问做偏 2 处、多做 3 处、少做 2 处 |

核查（命令原样）：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/governance-r3-verify -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py ledger-topdown-governance-r3 --out /home/fy5090/.local/share/singlefs/work/governance-r3-verify/ledger-topdown-governance-r3-tree --repository /home/fy5090/code/singlefs`：任务 20369「清单点名 19 份，倒推出 19 份，sha256 对上 19 份」。
- `python3 scripts/agents/cite-check.py research/prompts/ledger-topdown-governance-r3-opus-output.md research/prompts/ledger-topdown-governance-r3-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/governance-r3-verify/ledger-topdown-governance-r3-tree --background research/prompts/_ledger-topdown-governance-r3-background.md`：退 0，「核了 58 处引文、49 处裸行号，对上 39 处，没判 68 处」，没有对不上的；没判的那几处，判决用到的逐处在下面现查。
- 攻方腿九条探针，主 agent 照它报告「要主 agent 跑的命令」一节原样交（日志存 `~/.local/share/singlefs/work/governance-r3-verify/q1.log` 到 `q9.log`）：
  - 20368（第 1 问）退 1「喂了 23 条，18 条不对」：15 条「退 0（应当 2）」，unlink、find -delete、find -exec rm、rsync --delete、rsync --remove-source-files、gzip、`1<>` 重定向、shred、sort -o、gawk -i inplace、unzip -o、python 的 os.remove、os.rename、os.open 加 os.write、shutil.copytree；3 条「退 2（应当 0）」，tar -xO 与两种点名草稿文件的 patch；5 条对照都对。与腿推的一致。
  - 20370（第 1 问跨台硬链接）落在本机（`task status` 第 1 次「本机」），两行设备号都是 66306，照腿自己写的判法「落在本机的这一次不算数」，核不动。
  - 20371、20372（第 3 问）都退 3，说明「不在工作根下（认的根：/home/fy5090/.local/share/singlefs/work/governance-r3-attack/q3/fake-data/work）」。认的根取的是副本根上手写的那份多机配置，坐实「管 `--out` 能写到哪的是副本里手写的那份」；腿推 20371 会退 0、写出规格，没写出来，「写得出规格」与「经符号链接写进仓里」这两句没坐实，按推的记。
  - 20373（第 4 问）退 1：缺 2 处点名 `caller_dot.py`、`caller_plain.py`，本次未判 2 处点名 `caller_environment_path.py`、`caller_through_imported_path.py`，另 6 份两处都不出现。与腿推的一致。
  - 20374（第 4 问 qemu 那一道）退 1「.claude/gate.d/qemu-device-streams.sh：内嵌 python 第 1 段（第 513 行起）编译不过：unexpected indent（第 514 行）」，一份只改一行注释的规格在草稿里被判编译不过。与腿推的一致。
  - 20375（第 5 问库）「RECORDABLE binaries=1 green=0 withheld=3 markers=3」。20376（第 5 问服务）提醒「这一趟有 1 条用例本次未跑…：gpu_case_alpha」，同一份日志库认 3 个记号、服务认 1 条。与腿推的一致。
  - 20377（第 6 问）「11 行命中」，`scripts/probe.sh` 第 13 到 23 行全中、第 2 到 12 行一行不中：讲现状的 10 句判红，讲过去的 11 句不判。与腿推的一致。
- 辩方腿点的两处现查：第二轮判决第 57 行写的样本行是「`NOT_RUN_ANYTHING_HERE`」，第二轮攻方腿报告第 138 行的原文是「`CANNOT_RUN_ANYTHING_HERE`」，第二轮判决转述错了（冻结证据，不回改，更正在这里）；登记表 `.claude/gate.d/break-switch-replay-files.tsv` 第 61、75 行写着「红的格是否只在点名那一格交主 agent 跑过再算数」，而 20337、20351 已照这一格的判法复跑过、逐格相同。
- 攻方腿第 2 问（量的，grep 源码）现查：`.claude/hooks/bash-command-detector.sh` 的 `KNOWN_DETECTOR_BREAK_ITEMS` 与文件头开关表各 111 个；正文写的「短写法 19 个」是 20 个（登记表第 75 行自己写「另有二十行不带 BASH_COMMAND_DETECTOR_BREAK= 前缀的短写法开关」）。20351 的「117 条」不含这 20 条短写法，是 91 条加三个共用库变量的 26 条。

## 二、判决

两条腿结论不一致（攻方打中、辩方没打中），照治理文件轮的收法逐条现查、坐实的改。这是第三轮，改的修补标零轮。

### 坐实、这一轮改

1. **检出钩子第十四项还漏 15 种、误拒 3 种**（攻方第 1 问，20368）：第二轮补的 13 种之外，删除、挪走与在原 inode 上改的命令照样过得了；按目录判的解包与打补丁把只写标准输出、只改草稿文件的命令拒了。辩方腿第 11 问判第二轮修补照判决做了，与这一条不冲突：判决第 1 条只列了 13 种。
2. **规格工具认仓副本的判据认不出副本，工作根由副本根上手写的多机配置定**（攻方第 3 问，20309、20371、20372；辩方第 8 问）：判据是「仓根下没有 `.git`」（`scripts/gate/make-swap-spec.py:281`），而 `scripts/hosts/minimal-copy.py:28` 建的副本都在副本根 git init。辩方腿判第二轮判决当时看得出（它自己引的任务 20189 日志末行就是「在副本根 git init 了」），判决缺的是把判据写明的一句，这一条站得住。采攻方腿撤回「认出副本」这个方向的理由：副本是谁都改得了的树，按 `--out` 落在哪判就不用认副本。
3. **换上工具有两种读法错**（攻方第 4 问，20373、20374）：别的模块替调用方往 `sys.path` 插目录的（e201 经 `e171_research_side` 插 E171 的目录，`scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/e171_research_side.py:28`）认不出解到哪一份，照列本次未判，而它运行时真解到被换的那一份；heredoc 起头那一行以续行收尾时正文从下一行收（`scripts/gate/hook-swap.py:459`），qemu 那一道的内嵌 python 编不过，任何含这一道的规格都换不了。另有 4 种写法（`SourceFileLoader`、`runpy`、带点 import、拼库名）既不核也不列本次未判。
4. **任务服务一行里只认头一处记号**（攻方第 5 问，20375、20376）：同一份日志库认 3 个、服务认 1 条，服务报的本次未跑条数少了。
5. **变异那一处认记号只认行首**（攻方第 5 问 5b 表，读源码）：`scripts/mutation/crates-mutation-rows.py:1399` 的正则锚在行首；点名的测试 ok 又在行中间打了记号时，这一条记成「没红」写进判定库（`:3582`），与它文件头「宁可记本次未跑，不记没红」（`:3534`）相反。按线索记：没有任务读数，这一轮改之前先造样本证红。
6. **共用约束第 89、101 行与换上后的钩子对不上**（攻方第 7 问）：第 89 行「新建的文件…不判」写错（解包、打补丁、`ln` 从服务源码建出的新文件都拒，草稿目录里的硬链接也认）；第 89 行少写认不出类型的子 agent 与判不到的写法，第 101 行少写工具员在 Bash 里用定点替换工具分几次改脚本两道都不拦。
7. **登记表两行说明过时**（辩方第 11 问）：第 61、75 行改成已复跑的读数。
8. **`lag-growth-unjudged` 开关的点名少一份样本**（攻方第 6 问，源码与 `git show HEAD`）：这条开关下绿样本 `comment-history-lag-green` 照样判错，文件头只点名了红样本；换上前就是这样，不是第二轮修补带进来的，现行账本第二百七十一节记的七处之一，这一处归这一轮补点名。

### 认下、记账不在这一轮改

9. **第十四项的结构性改法**（攻方第 1 问撤回 ①）：按命令名逐种补没有闭集，这一轮照补 15 种，挡的仍只是一部分。攻方腿提的「服务只从登记过内容哈希的快照编」动的是两个服务怎么编，牵到别的会话与用户，不在这一轮定。登账本。
10. **记号认法一处定义、几处导入**（攻方第 5 问撤回 ③、辩方第 10 问）：认一族还是认一个，辩方腿判今天定不了（`crates/` 里只打 `NOT_RUN_NO_GPU_ADAPTER` 这一个），触发条件是 `crates/` 出现第二个往 libtest 输出里打的 `NOT_RUN_` 名字。这一轮只改第 4、5 条两处，入口脚本 `scripts/crash/crash-point-replay.sh:834`、门禁 `.claude/gate.d/crash-point-replay.sh:3706` 与 `:3753`、研究复跑 `.claude/gate.d/research-build-and-replay.sh:972`、广谱变异 `scripts/mutation/broad-mutation.py:2701` 五处登账本，那几份归别的会话管的先问。
11. **注释历史判法的误红与漏判**（攻方第 6 问，20377）：讲现状的 10 句判红、讲过去的 11 句不判，真仓里这几形今天 0 行（腿量的，`grep -P`）。判据看的是「以前」前后各一个字，攻方腿自己说分不开「以前一次读两遍」与「以前一份为准」；换判法要先在真仓上量新旧各判多少。登账本。
12. **换上工具没有锁**（辩方第 9 问）：辩方腿现数这一轮真换 10 趟、窗口相交 0 对，三回对时间拦的是换上与别人在跑的门禁互相读，不是两趟换上互盖。记账站得住；补一句数法：窗口相交照结局文件的起止时刻数。
13. **第二轮判决两处写错**（辩方第 8、10 问）：第 57 行的样本行原文是 `CANNOT_RUN_ANYTHING_HERE`，库误认的是它的后半截，缺的是左词界；第三节「两边整词相等」那一句以工具员做成的「库认一族、服务认一个」为准。冻结证据不回改，更正在这里。
14. **第二轮修补做偏的注释历史那一形**（辩方第 11 问）：判决要正面形态，工具员做成反面例外，改前也没在 872 行上量新旧。并进第 11 条一起登。

### 不采

15. 攻方腿第 3 问 3b 表 A、C 两种判据：它自己判两种都要回答「副本里的东西谁都改得了」，不采。

### 核不动的

1. 第 1 问跨台硬链接（20370 落在本机）。
2. 第 3 问「写得出规格」「经符号链接写进仓里」：20371 没写出规格，后一句腿没跑。
3. 辩方腿第 13 条（账本侧四处过时句、两台上限那一前提）它写明没重核。

### 这一轮的结局

第三轮有打中的，坐实 8 条。照治理文件轮的收法改完不再开第四轮，改的修补标零轮。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 标 |
|---|---|---|---|
| 检出钩子第十四项另认 unlink、find -delete 与 find -exec 启动的删除、rsync 的 --delete 与 --remove-source-files、gzip 一类原地换文件的、`<>` 读写重定向、shred、sort -o、gawk -i inplace、unzip 解进服务源码目录、python 的 os.remove / unlink / shutil.rmtree / os.rename 的源 / 带写标志的 os.open / shutil.copytree；解包认 -O、--to-stdout、--to-command 不写盘；patch 带位置参数的原文件时照那一份判 | 第 1 条 | 攻方腿 | 零轮：20368 那 23 条喂法当样本，换上前 18 条不对、换上后 0 条 |
| 规格工具不再认副本，改按 `--out` 落在哪判：`--out` 的 realpath 往上任何一层有 `.git` 就拒；检出钩子 ⑫ 把 `make-swap-spec.py` 的 `--out` 当定点写脚本的目标 | 第 2 条 | 攻方腿 | 零轮：20371、20372 那两条与副本当样本 |
| 换上工具跟模块级 import 进来的仓内模块往 `sys.path` 加的目录；heredoc 起头那一行以 `\` 收尾时正文从续行结束的下一行起；文本里提到库名而载入与绑定都没认出的调用方列本次未判；`X.__module__` 里的 X 是从目标库 import 的就折成那一份库 | 第 3 条 | 攻方腿 | 零轮：20373 那棵探针小树与 20374 那份规格当样本 |
| 任务服务逐处认一行里的每一个记号，认法从门禁那一份库导入（认一个名字照旧）；变异那一处同样导入库的认法 | 第 4、5 条 | 攻方腿 | 零轮：20375 那份日志当样本，服务换上后认 3 条；变异那一处先造一份行中间打记号的日志证红 |
| 共用约束第 89、101 行照攻方腿第 7 问那张表改 | 第 6 条 | 攻方腿 | 主 agent 照钩子文件头现读改 |
| 登记表第 61、75 行照 20337、20351 的读数改；第 75 行另写清 117 条不含 20 条短写法 | 第 7 条 | 辩方腿 | 主 agent 改 |
| `.claude/gate.d/doc-text.sh` 文件头 `lag-growth-unjudged` 那一条补点名 `comment-history-lag-green` | 第 8 条 | 攻方腿 | 零轮：门禁 doc-text 的样本开关复跑那一条当样本 |

这一批都是治理文件，没有要写进产品代码的零轮改法。每条零轮改法的样本先证红再换上。

## 交用户的

- 第 9 条：检出钩子第十四项按命令名逐种补没有闭集，三轮各补一批、每轮都还漏。另一条路是两个服务只从登记过内容哈希的快照编、主工作区里改到一半的源码编不进服务；这牵到服务怎么编，要不要做、什么时候做由用户定。代价没量。

## 这一轮被判过的文件

- research/prompts/_ledger-topdown-governance-r3-body.md：正文第二节写的「短写法 19 个」现数是 20 个，其余引的任务读数两条腿与主 agent 现查相符。
- research/prompts/ledger-topdown-governance-r2-main-verification.md：第 57 行样本行转述错、第三节「整词相等」一句以现状为准（第 13 条）；坐实 10 条、记账 5 条辩方腿判站得住，第 4 条缺写明判据的一句。
- .claude/hooks/bash-command-detector.sh：第十四项漏 15 种、误拒 3 种（第 1 条）；⑫ 不看 `make-swap-spec.py` 的 `--out`（第 2 条）；开关名单与开关表三处各 111 个、逐个对上（第 2 问）。
- .claude/hooks/write-guard.sh：第十一道门禁阶段那一支与共用约束第 101 行逐项对上，少写的是 Bash 里分几次定点替换脚本两道都不拦（第 6 条）。
- scripts/gate/hook-swap.py：别的模块替调用方插 `sys.path` 认不出、heredoc 续行读法错、4 种写法不核也不列（第 3 条）。
- scripts/gate/make-swap-spec.py：「没有 `.git`」认不出照规矩建的副本，工作根由副本根上手写的配置定（第 2 条）。
- task-server/task_server.py：一行里只认头一处记号（第 4 条）。
- task-server/task_server_lib.py：结局提醒的条数随 `task_server.py` 的认法少算（第 4 条）；退 9 与上一趟还在换的提醒是同一件事（第 12 条，辩方腿）。
- task-server/task_server_selftest.py：样本日志没有一行两处记号、没有一族别的原因，碰不到第 4 条那两格。
- task-server/README.md：三语说明与「整词、行内任一处」的认法一致，一行几处没写（第 4 条）。
- scripts/gate/lib_libtest_result_lines.py：认 `NOT_RUN_` 一族、逐处认，20375 认 3 个；一族对一个的差别记账（第 10 条）。
- scripts/gate/comment-history-check.py：讲现状的 10 句判红、讲过去的 11 句不判（第 11 条）；`lag-growth-unjudged` 下成功句嵌开关说明，绿样本照样判错（第 8 条）。
- .claude/gate.d/doc-text.sh：`lag-growth-unjudged` 只点名红样本，少点名 `comment-history-lag-green`（第 8 条）；两行改写过的文件头不再判红。
- .claude/gate.d/doc-process-records.sh：另三行叫法改了，辩方腿判多做的第 611 行与判决同向。
- .claude/agents/implementation-writer.md：第 47 行照证红脚本今天的取法改了，缩域证红那一形仍没量（第 12 条同类记账）。
- .claude/agents/tooling-writer.md：第 3 步门禁阶段那一支与写闸第十一道一致。
- .claude/agent-common.md：第 89 行「新建的文件…不判」写错、两行各少写（第 6 条）。
- .claude/gate.d/break-switch-replay-files.tsv：第 61、75 行写着还没复跑，实际已复跑逐格相同；第 75 行的 117 条不含 20 条短写法（第 7 条）。

## 回看决策

不涉及决策：这一轮判的都是钩子、换上工具、任务服务、注释历史判法与定义的写法，没有拿依据去推翻或确立任何决策分项。
