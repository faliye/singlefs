# ledger-defs-r3 攻方腿（opus）报告

日期：2026-11-12。被判的三份与开工快照：`sha256sum -c research/prompts/ledger-defs-r3-snapshot/judged-sha256.txt` 开工时三行都是 OK（量的）。只攻正文点名的第一组第 1、3、4、5 条、「已定的做法里漏掉的情形」与第三组两条；第一组第 2 条、第二组归正推腿，没做。

## 复跑命令与模型文件

这一腿写了一个小探针（喂范围声明闸的 hook 输入，照主 agent 判，核退出码，并把定义给的 grep 判法对同几个目标并排打出），不起编译、不起崩溃枚举。模型目录 `research/prompts/ledger-defs-r3-opus-model/`。复跑命令（交主 agent 跑，见文末「要主 agent 跑的命令」第 1 条）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/ledger-defs-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/ledger-defs-r3-snapshot /home/fy5090/.local/share/singlefs/work/ledger-defs-r3-opus-probe-run /home/fy5090/code/singlefs/research/prompts/ledger-defs-r3-opus-model
```

各文件 sha256（量的，在模型目录里 `find . -type f | sort | xargs sha256sum`）：

```
6b5b708dafe7a074b49e521275b5e8e424d2386b2f2b93d1077edce886ced580  ./expected.tsv
a676ec9364f68fc56200e990c568d8b28e1b6b6f726a0a30b94cc2d7617678e8  ./inputs/cargo-test-registered-target-control.json
cb5e297643ffc3c04638ca4fdebcb969271d942e5aaa23c1e182b8842fa58657  ./inputs/cargo-test-target-the-definition-grep-counts-as-enumerating.json
6b1e8818581e7801d9fdc25ac3bebb33a5883447e460ea1ce48ed90d4243bb87  ./inputs/prove-red-new-case-with-cwd-at-the-writer-copy.json
511f47125a9e2b730164065a7056dfadb6ed98fb19e795bf95e35f9bbe18eaa2  ./inputs/prove-red-new-case-without-cwd.json
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
1d75d974163de5a9b90e350099d985625f29cab33edac99f683090259f9cbd24  ./tools/rerun.sh
```

- `tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`（换前第 1–79 行与模板逐字节相同，`diff` 过）：在运行目录里造一棵「实现员副本」样子的小树（新写的崩溃枚举用例只登记在这棵树的 `stage-inputs.tsv`、源码与变异行只在这棵树里），再把 `inputs/` 里四份 hook 输入（`@REPO@`、`@WORK@` 换成参数给的仓根与运行目录）在 `setsid --wait timeout -k 5 60` 里逐份喂给仓里的 `.claude/hooks/run-scope-guard.sh`，退出码与 `expected.tsv` 逐行比；另打 `name=definition_grep` 几行（照 `.claude/agent-common.md` 第 68 行给的 grep 判法，对同几个目标判「会不会走」）与一行 `name=detections_naming_the_new_case`（闸把判不出的那一处记进检出记录没有）。末行 `name=probe_summary samples=4 mismatches=<M>`，M 为 0 退 0。
- 外层 `rerun-from-a-clean-work-directory.sh` 与第二轮那一份逐字节相同（sha256 都是 e3294c15…）：先删掉以 `-run` 收尾、在工作根下的那一个运行目录，再调同目录的 `tools/rerun.sh`。
- 探针没跑（子 agent 不交任务）：下文引它的退出码都是「推的，待主 agent 跑」。内存与挂钟（推的）：四次 python 钩子，每次几秒、几十 MiB，整条一分钟以内。
- 另有一处量的不是探针：`git merge-file -p` 两边各在文件末尾追加一行，在草稿目录里直接跑过（git 自己的命令，不是项目代码），命令与原样输出在 T2；主 agent 可以照抄复跑（「要主 agent 跑的命令」第 3 条）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一·1 从头走一遍 | 打中（T1、T2、T3、T4） | 第 2 步红了修补之后「照原来点的全部项重交」漏掉修补新加、改名的项，脚本打的变异行名单又不减删掉的名字（点名删掉的行门禁退 2）；第 3 步合的那一条在登记表上三方合并冲突时，核的那几条不罩登记表、退 0，定义叫「再交同一条」，同一处再冲突，没有出口；脚本那一侧的出路是「主 agent 手工并好再打」，合进去的是没验过的手工合并；同一类冲突在第 1 步打补丁时让 P3 退回重做；指纹整份排除变异表之后，别的会话追加、锚在本批改动上的变异行合完才在 mutation-tables 红 |
| 一·3 拒算那一环 | 打中（T5） | `list` 判的是整个验证包、不分这一批；快照里带进来的别的会话的未登记拒算，照第 69 行只有「收进这一批」或「退给实现员改测试」两条，后者对别人的测试做不了，「都没有才算判完」可能不成立；每次重交都从主工作区此刻重取快照，主工作区漂来的又冒出——会绕圈。第 2 步要不要整步重做：第 69 行没写，第 58 行末句推得出要 |
| 一·4 留下的快照 | 打中（T6，量的） | 完工清理闸不列它（`merge-and-verify/` 没有归属文件，那一类只认库）；工作根 `merge-and-verify/` 下此刻留着 13 棵 `repo/`、共 11G（今天两棵各 1.3G）；定义没写退 3 而用户不同意、跨轮停在第 1、2 步之间那两支什么时候删；第 2 步还有任务在快照上跑时第 1 步重交，照「交出去就删」删的是正在用的树；看门狗 6 小时没动就把等用户的那一棵列给主 agent 删 |
| 一·5 合成补丁的差 | 没打中 | `merge-patch` 在打完补丁之后、编与测之前就算好（脚本第 2431–2433 行的次序），只罩补丁动到的路径，第 1 步测试与第 2 步写进快照的东西带不进去；文件头那句「快照此刻」字面会读成第 2 步之后，记一处措辞 |
| 二 已定做法漏掉的情形 | 打中（T7、T8；T2、T5 也属这一格） | 第 2 步的崩溃放量分项一放开全域就是入口自带上限的退 79（不认全域理由），定义「都绿才进第 3 步」，两条出路走哪条、走了能不能合没写；第 2 步某一项全域超四小时，服务进队就打回、不给填 why，定义没写 |
| 三·1 grep 判法与闸 | 部分打中（T9 量的、T10 推的待跑） | 同一棵树上 grep 是闸的超集：量到 11 个目标 grep 说会走、闸说不会，照 grep 多设缩域变量过得了闸，但按定义「判出的结论不算数」，变异门禁那一趟还不入库，要多跑一趟全域；注释算不算两边一致。树不一样：证红模板不带 `--cwd`，闸读的是主工作区，定义叫对着副本判；新写用例的证红在闸那里一律「判不出、放行、记检出」，同一条命令带 `--cwd <副本>` 就拒 |
| 三·2 全域那一趟 | 部分打中（T7、T8、T11） | 变异行的全域落在第 2 步、主 agent 交、带全域理由过得了闸；共用约束说 `cargo test` 点名的目标也在第 2 步全域，第 2 步清单里没有它（实际在第 1 步跑）；放量分项见 T7；两小时规则不挡合，四小时上限挡，定义没写怎么走（T8） |

## 一、现行版从头走一遍（第一组第 1 条）

走法：一段两批有依赖的实现员。第一批 P1、P2（互不撞文件），第二批 P3 以打了 P1、P2 的段树为基线；P3 新写一条崩溃枚举用例（登记行写在副本里、跟着补丁走）与一条要拒算的测试（只列进报告）。主 agent 判过拒算，做成拒算补丁目录 R0。验证段第 2 步红一次（P3 的新变异行在全域上没红），修补 F1；合进主工作区之后代码轮打中一条，修补 F2。段树建法、补丁打进段树、实现员副本从段树建，前两轮量过，不重复。下表只列这一轮现行版里每一步交什么、落在哪棵树、读哪一份登记表。

| 步 | 命令（今天认不认） | 落在哪棵树 | 读的登记表是哪一份 |
|---|---|---|---|
| 写段：P3 的证红 | `task-server/task submit -- bash scripts/mutation/prove-red.sh --copy <副本> …`（`.claude/agents/implementation-writer.md:37`）：闸认、不带 `--cwd` | 被证的是副本；闸判崩溃域读的是主 agent 起命令的目录（主工作区） | 闸：主工作区的（没有 P3 的登记）；定义说副本的（T10） |
| 写段：R0 | 主 agent 照 `.claude/main-agent.md:68` 拷三份、`git diff --no-index` 出 `crates.patch`：工具员报告里核过 `git apply --check` 打得上主工作区 | 主 agent 的草稿目录 | 主工作区此刻那一份拒算表 |
| 第 1 步 | `merge-and-verify.sh P1 P2 P3 R0`（带 `env MERGE_AND_VERIFY_KEEP_COPY=1`）认；段树这时删 | 快照＝主工作区@T1 + 全部补丁 | 快照里的（P3 的登记在里面）。P3 打进快照时登记表若与 T1 时的主工作区三方冲突，整批停（T3） |
| 第 2 步 | `crash-case-check.py`、`refused-test-registry.py list`、按块的崩溃用例、放量分项、形式化、变异门禁点新行（全域理由）、命名检查，`--cwd` 与根写快照：都认 | 快照 | 快照里的；`list` 判的是整个验证包（T5）；放量分项全域就退 79（T7） |
| 第 2 步红、修补 F1 | 实现员副本从快照建，F1 接在最后，第 1 步整批重交、第 2 步「照原来点的全部项」重交 | 新快照＝主工作区@T2 + P1…R0 + F1 | 新快照里的；F1 新加或改名的项不在「原来点的」里（T1） |
| 第 3 步 | 核指纹与变异行（仓根直接跑）、`apply-writer-patch.py <日志目录>/merge-patch`、`--read-outcome`、mutation-tables | 主工作区@T3 | 合的那一条三方合并；登记表与变异表不在指纹里（T2、T4） |
| 代码轮修补 F2 | 副本从主工作区建、第 1 步只给 F2、再走第 2 步、合：认 | 新快照＝主工作区@T4 + F2 | 新快照里的 |

「有没有哪一步让主工作区出现没在同一棵树上被第 1、2 步都验过的组合」：有两处，都出在第 3 步合的时候碰上指纹不罩的那几份文件——T2 照脚本出路手工并好的登记表，T4 别的会话追加的变异行。指纹罩到的那几个包，这一版没打中。

### T1 第 2 步红了的修补，「照原来点的全部项重交」漏掉修补新加、改名的项；脚本的名单又不减删掉的名字

- 定义：`.claude/main-agent.md:58`：「第 2 步那几条在新留下的快照上照原来点的全部项重交（键相同的照复用）」。同一行前面点项的规矩是「崩溃放量分项、formal-verification 带 `--item` 的那几条（点这一段全部补丁改到的项」，两句对修补 F1 给出不同的项：F1 也是「这一段的补丁」，却不在「原来点的」里。
- 脚本打的名单：`scripts/gate/merge-and-verify.sh:2667` 逐份补丁目录收 `mutations-append.tsv`、`mutations-replacements.tsv` 的第一段（`awk -F'\t' 'NF >= 6 && $1 !~ /^#/ { print $1 }'`），不看 `mutations-delete.txt`。变异门禁点到表里没有的名字：`.claude/gate.d/crates-mutation-replay.sh:29`「不在表里列出可点名的、退 2」。
- 走法：P3 的新变异行 M 在全域上没红。实现员照快照写 F1，改的是测试，同时把那一行重写成另一处改坏、起新名字 M2（F1 的 `mutations-delete.txt` 写 M、`mutations-append.tsv` 写 M2；用替换同名就没有这一支）。第 1 步整批重交 P1 P2 P3 R0 F1，新快照的表里 M 没了、M2 在。
  - 照定义字面「原来点的全部项」：`--item M` 退 2（不在表里），M2 一次都没在第 2 步判过。
  - 照新一趟脚本打出的名单：名单里 M（P3 的追加）与 M2 都在，同样点到 M、退 2。
  - 两种读法第 2 步都红在一条已经删掉的行上，定义「这一步有红的：修补交实现员」那一支没有可修的东西。
- F1 还碰到原来没点的形式化项、别的崩溃用例时（修测试顺手改了共用模块），照字面同样不点。
- 四句：判别子——F1 交回那一刻主 agent 看得到 F1 改了哪些文件、写了哪些变异名；字面分句——第一组第 1 条「第 2 步红一次修一次」与「每一步交的命令今天认不认」；改法（零轮）——红了之后第 2 步照新一趟脚本打出的那一段与「这一段全部补丁改到的项」重交，删掉「照原来点的全部项」；脚本的名单减去各份 `mutations-delete.txt` 与后面补丁的删除。

### T2 第 3 步合的那一条在登记表上三方合并冲突：定义叫「退 0 再交同一条」，同一处再冲突，没有出口；脚本的出路是手工并

- 定义：`.claude/main-agent.md:59`：「三方合并冲突、核与换之间主工作区被改了的，重做核的那几条，退非 0 照核的那几条退非 0 那一句整批重交、退 0 再交同一条」。升级只在核的那几条退非 0 时：同一行「核的那几条连着三趟退非 0」。
- 核的那几条不罩登记表：`scripts/gate/merge-and-verify.sh:166`：「不进起点：各会话都在往里追加，三方合并在 apply-writer-patch.py 里核（冲突退 2）」（上一行是「登记表（REGISTRATION_TABLES）与 crates/mutations.tsv」）。
- 合成补丁里登记表是按文本算的差（第 2431 行之后那段，`git diff`），R0 的行接在拒算表末尾（`.claude/main-agent.md:68`「收的行追加到 `b` 那一份末尾」），P3 的合表行也接在末尾。主工作区里别的会话同样往末尾追加。量的：

```
$ printf 'row-a\trazon-a\nrow-b\treason-b\n' > base && { cat base; printf 'row-other\tanother session row\n'; } > ours && { cat base; printf 'row-batch\tthis batch refused row\n'; } > theirs && git merge-file -p ours base theirs > merged.txt; echo rc=$?; cat merged.txt
rc=1
row-a	razon-a
row-b	reason-b
<<<<<<< ours
row-other	another session row
=======
row-batch	this batch refused row
>>>>>>> theirs
```

  （git 2.43.0，在草稿目录 `mergefile-probe/` 里跑的；`apply-writer-patch.py` 的三方合并就是逐个 `git merge-file`，它文件头第 100 行。）
- 这几份表多常被别的会话改（量的，`git diff HEAD --numstat`，HEAD 是 2026-11-07 那一次提交）：`refused-tests.tsv` +12 −0，唯一一段是 `@@ -8,0 +9,12 @@`（末尾追加）；`crash-point-replay-items.tsv` +5 −1，其中一段 `@@ -76,0 +77,4 @@`（末尾追加）；`stage-inputs.tsv` +64 −57；`scripts/crash/crash-amplification.sh` +188 −67。三份表的修改时刻是 2026-11-11 或今天。
- 走法：第 1 步在 T1 取快照，第 2 步几个小时；其间别的会话往拒算表或合表末尾加了一行。第 3 步合 `merge-patch`：拒算表那一段冲突，退 2。照定义重做核的那几条：指纹不罩这张表、变异行没动，都退 0 → 「再交同一条」→ 同一处冲突、退 2 → 再核、退 0 → 再交……「连着三趟退非 0」不触发，定义没有别的出口。
- 脚本那一侧的出路两样：`scripts/gate/apply-writer-patch.py:621`「或主 agent 照上面点名的段手工并好、重出补丁再打」（前一句是「退给实现员在主工作区此刻的样子上重做这几段」——合成补丁没有实现员）。照手工并的那一支，合进主工作区的登记表是快照里从没出现过的样子，第 1、2 步都没验过；登记表并错一个字（目标名、用例名）要到提交时才红。
- 四句：判别子——退 2 的拒绝说明逐个点名冲突的文件，主 agent 看得到冲突落在指纹不罩的登记表上；字面分句——第一组第 1 条「有没有哪一步让主工作区出现没在同一棵树上被第 1、2 步都验过的组合」（手工并那一支）与「已定的做法里有没有漏掉的情形」（「再交同一条」那一支）；改法（零轮）——冲突落在指纹不罩的文件上的一律整批重交，不「再交同一条」；更根本的是登记表照变异表那样按键合（见「替代做法」第 1 条）。

### T3 同一类冲突在第 1 步打补丁时：P3 的登记行打不进快照，整批停、P3 退回重做

- 「登记行跟着补丁走」之后，P3 的 `crates.patch` 里登记表那几段的基线是段树（建段树那一刻的主工作区）。第 1 步的快照取主工作区@T1；段树建好到 T1 之间别的会话往同一张表末尾加了一行，P3 那一段在快照上三方冲突，`merge-and-verify.sh` 整批停、退 2。脚本出路：`scripts/gate/merge-and-verify.sh:2428`「以主工作区此刻建一棵新段树（$(segment_tree_command)），这一份之前的补丁照次序打进去，这一份退给实现员以新段树为基线重做」。
- 冲突的只是登记表的末尾，代码一行没撞，照出路要新建段树、新派实现员重出 P3。R0 同理：它的基线是做它那一刻的主工作区。频率（推的）：写段跨几个小时到一两天，上一条量到的改动量下，碰上一次的可能不小；没量过真实一段里的次数。
- 这是第二轮我提的「登记行跟着补丁走」带出来的新代价：第二轮那一版登记行由主 agent 手写进主工作区，不经三方合并、不会冲突（它的毛病是 R1、R10）。见文末「撤回与收窄」。
- 四句：判别子——拒绝说明点名冲突的是登记表；字面分句——第一组第 1 条「每一步交的命令今天认不认」；改法见「替代做法」第 1 条（零轮）。

### T4 指纹整份排除变异表之后：别的会话追加、锚在本批改动上的变异行，合完才在 mutation-tables 红，红了怎么办没写

- 现行版只核「这一批要换掉、删掉的变异行」（`.claude/main-agent.md:59` 第 3 步头几条）；变异表整份不进指纹（`scripts/gate/merge-and-verify.sh:166`）。mutation-tables 那一格照旧只在合完之后对主工作区交：同一行「再交它打出的「补丁打进主工作区之后另交」那几条（code-source-discipline 的 mutation-tables 格这类读主工作区的），删这一趟留下的快照」，脚本打的那一条 `--cwd` 是主工作区（`scripts/gate/merge-and-verify.sh:2444`）。
- 走法（推的，没找实例）：P1 改了 `singlefs-core` 里一段函数体；T1 之后分诊那一路的会话照「补取样点」往变异表末尾追加一行，锚的正是这段函数体的原文、点的测试在已有的目标里（只改变异表，不碰任何包目录，指纹不变）。第 3 步按名字合变异表不冲突，合完主工作区里那一行的锚点已经不在源码里，mutation-tables 红。这个组合（P1 改后的源码 + 那一行）第 1、2 步都没见过。
- 定义合完之后那几条没有「红了怎么办」；第一轮判决第 3 条记过「合入后另交的 mutation-tables 格」，这里是现行版排除变异表之后多出来的一种来源。
- 四句：判别子——第 3 步核之前主 agent 看不到这一行（不在快照里、不在指纹里）；字面分句——第一组第 1 条「有没有哪一步让主工作区出现没在同一棵树上被第 1、2 步都验过的组合」；改法（零轮）——mutation-tables 那一格挪进第 2 步对快照判，合完之后再对主工作区判一次只当兜底、红了照第 15 条交那一行的会话。

## 二、拒算那一环（第一组第 3 条）

### T5 `list` 判的是整个验证包、不分这一批；别的会话的未登记拒算会让第 2 步判不完，每次重交都可能冒出新的

- 定义：`.claude/main-agent.md:69`：「有「未登记」的照第 1 条判，收的照第 1 条补进拒算补丁目录、不收的退给实现员改测试」；同一行「「未登记」「已不拒算」都没有才算判完」。只有「已不拒算」分了这一批与之前：同一行「这一批之前就有的「已不拒算」行不归这一批，主 agent 在主工作区直接删」。「未登记」没分。
- 脚本：`scripts/gate/refused-test-registry.py:13`：「打此刻的拒算清单，每条一行「已登记 / 未登记<TAB>测试函数全名<TAB>机器给的拒算理由」」，取的是整个包：同一份第 22 行「覆盖：只有验证包不带特性那一趟的测试函数」——不看哪个补丁改了哪个测试。
- 快照是主工作区@T1 整棵拷的，带着别的会话没提交的半成品。别的会话新写了一条读活树的测试、还没登记（它自己的验证段还没走到登记那一步），第 2 步的 `list <快照>` 就报它「未登记」。照第 69 行：收，就是这一批替别的会话登记（第 68 行「实现员不登记，也不替它登记」管实现员，没管这一种）；不收，「退给实现员改测试」——不是这一批实现员的测试，改不了；两条都走不通，「都没有才算判完」不成立，第 3 步进不去。
- 绕圈：收了的照第 69 行「拒算补丁目录重做了的，第 1 步连同它整批重交」，新一趟取主工作区@T2 的快照，T1→T2 之间漂进来的新的未登记又冒出来，再重做、再重交。定义没有轮数上限，也没有「只认这一批」的筛法。验证包的改动量（量的）：657 份文件里 156 份修改时刻在 2026-11-11 之后、18 份在今天；`tests/` 下对 HEAD 未提交 172 份文件 +24835 −5469；拒算表五天里别的会话追加了 12 行。今天主工作区上就有没登记的拒算吗：没量（要跑项目代码），交主 agent（「要主 agent 跑的命令」第 2 条）；有一条，这一格就是今天的事。
- 重交之后第 2 步要不要整步重做：第 69 行只写第 1 步整批重交；第 58 行末句「合进主工作区的只有第 1、2 步在同一棵快照上都绿了的那一批」推得出整步重做，第 57 行「被下一趟替掉的那一趟留下的快照，下一趟交出去就删」也让旧快照上的绿作废。照键复用，只改了拒算表的那一趟第 2 步多半全复用（推的：拒算表进不进按块用例与变异行的键，没读码核）。
- 四句：判别子——`list` 每行给测试函数全名，主 agent 对得出它在不在这一批补丁改到的文件里；字面分句——第一组第 3 条「这一环会不会绕圈」；改法（零轮）——「未登记」与「已不拒算」同样分这一批与之前：同一趟在打补丁之前的快照上（脚本已存了原样）也取一份清单，差集才归这一批，其余照「一轮怎么开、怎么收」第 15 条交给它的会话。

## 三、留下的快照（第一组第 4 条）

### T6 完工清理闸不列它；定义漏了两支删除时点；「交出去就删」会删掉还在用的那一棵

- 定义写了的：`.claude/main-agent.md:57`「被下一趟替掉的那一趟留下的快照，下一趟交出去就删（路径与删法在它收尾那一行），最后一趟的留到第 3 步合完」；第 3 步合完「删这一趟留下的快照」。删法脚本打的是 `rm -rf`（`scripts/gate/merge-and-verify.sh:821`「看完自己删：rm -rf」）。多大：定义没写；量的见下。
- 完工清理闸列不列：不列（读文件头，推的；钩子没喂）。快照在工作根 `merge-and-verify/<日期>-<进程号>/repo`，`merge-and-verify/` 下没有归属文件（量的：`ls .../merge-and-verify/.owner` 报 No such file）。主 agent 那一侧只判本会话认领的目录与崩溃放量的运行目录；没归属文件的工作根目录归「迁库遗留」一类：`.claude/hooks/completion-cleanup-guard.sh:54`「工作根下一层没写归属文件（.owner 不在或写得不对）的每个目录」，而那一类 `.claude/hooks/completion-cleanup-guard.sh:78`「迁库遗留那几处只认库拷贝、旧文件库与库备份三样，编译目录与仓副本不归这一类」。那份钩子里 `merge-and-verify` 出现 0 次（`grep -c`，量的）。
- 量的：工作根 `merge-and-verify/` 下此刻留着 13 棵 `repo/`（2026-11-07 到 2026-11-12），`du -sch */repo` 合计 11G，单棵 182M–1.3G，今天的两棵各 1.3G。这些是不是这一版流程留下的，没核（也可能是更早带 `MERGE_AND_VERIFY_KEEP_COPY=1` 跑的）；要点是没有一道闸会把它们列出来，漏删就一直留着。
- 定义没写的删除时点：
  1. 第 1 步退 3、弹窗用户不同意：`.claude/main-agent.md:57`「用户不同意的不合、这一批停在验证段等用户定」——快照留到哪天没写，等用户的时间不定。
  2. 第 2 步有一项红了，别的项还在这棵快照上跑（按块的崩溃用例、全域变异行一跑几个小时），修补写好、第 1 步整批重交，照「下一趟交出去就删」就是在任务还开着这棵树的时候 `rm -rf` 它。那几个任务照「原来点的全部项重交」反正要重跑，删了不改结论，但定义没说先 `cancel` 还在这棵树上跑的任务（服务那一侧会把它们当失败处理、推给会话，推的）。
  3. 被停、被 `SIGKILL` 的那一趟（撞内存上限、四小时上限）：带 `MERGE_AND_VERIFY_KEEP_COPY=1` 的照样留着快照；定义只说「被下一趟替掉的」删，没被替掉、直接放弃的那一棵没说。
- 看门狗那一侧反过来：它扫整个工作根往下四层（`scripts/agents/agent-watch.py` 的 `SCRATCH_SCAN_MAX_DEPTH = 4`，`repo/` 在第三层），带 `.git` 的算仓副本，`scripts/agents/watch.conf:31`「scratch-idle-hours=6」六小时没动就报给主 agent「核过没人在用再删」。上面第 1 支等用户、第 3 步「等那一轮判决写完」都可能超六小时；照报告删了，第 2 步没跑完的要整批重来（`merge-patch` 在日志目录、不在 `repo/` 里，第 2 步跑完之后才删的不碍合）。
- 两次重交各留一棵：照「交出去就删」同一时刻最多两棵（旧的在修补实现员建完副本之前不能删、新的刚交）；定义的次序（修补写完才重交）对得上，这一支没打中。
- 四句：判别子——主 agent 收工时看得到 `merge-and-verify/` 下有哪些 `repo/`，但闸不提醒；字面分句——第一组第 4 条「完工清理闸列不列它」「两次重交各留一棵时怎么办」；改法（零轮）——见「替代做法」第 5 条。

## 四、合成的那一份补丁（第一组第 5 条）：没打中

- 次序（读码）：`scripts/gate/merge-and-verify.sh:2431`「全部打上之后照「原样 → 快照此刻」算合进主工作区的那一份补丁目录」，紧接在逐份打补丁之后、挑测试目标与编之前（第 2414–2440 行的次序）。第 1 步的测试、验证包那一步、第 2 步的任何东西都在它之后才往快照里写。
- 范围（读码）：只算 `merge-baseline.paths` 里那几份——各补丁 `crates.patch` 新旧两侧的路径，有变异表文件的另加 `crates/mutations.tsv`（同文件第 2724–2726 行）。测试写进快照别处的东西不在单子上。
- 快照里第 2 步的东西会不会经修补带进来：修补的实现员副本从快照建，副本里的现状是补丁的基线，不进补丁的差；变异门禁每片各一份源码副本（`.claude/gate.d/crates-mutation-replay.sh` 文件头「并发」），不在快照上就地改坏。没打中。
- 一处措辞：脚本文件头第 144 行写「全部打上之后照「原样 → 快照此刻」」，「快照此刻」单读会理解成第 2 步之后的样子；写成「打完补丁那一刻」就没有歧义。不影响行为。

## 五、已定做法里漏掉的情形（第二个攻击面）

T2（合的那一条在登记表上冲突）、T5（别的会话的未登记拒算）也归这一格，上面写了。另两处：

### T7 第 2 步的崩溃放量分项一放开全域就是入口自己的退 79，定义「都绿才进第 3 步」，没写走哪条出路、走了能不能合

- 定义：第 2 步要交「崩溃放量分项…带 `--item` 的那几条」（`.claude/main-agent.md:58`），「都绿才进第 3 步」（同一行）。实现员模板里的放量那一条：`.claude/agents/implementation-writer.md:34`「超 200000 或估不出就不编不跑、退 79」，出路两条，一条缩域（同一句「缩过域的那一趟成功句写「缩过域、不算门禁通过」」），一条「或留给提交时不指定范围的那一趟」。
- 入口的上限不认全域理由：`scripts/crash/crash-amplification.sh:196`「79  指定范围的崩溃域超上限」；上限与估法在 `scripts/crash/lib_crash_point_replay_items.py`（`SCOPED_CRASH_STATES_LIMIT = 200000`），那份库里「全域理由」「SINGLEFS_RUN_SCOPE」都出现 0 次（`grep -n`，量的）。范围声明闸那一侧写了全域理由放行，入口这一侧照样退 79。
- 什么时候会碰上：补丁改了放量分项罩着的代码，键对不上的块要现判，放量分项按设计罩的是大流（入口只数键对不上的；会不会超 20 万要看那一项，推的，没量哪一项超）。碰上了：缩域——「不算门禁通过」，不是绿；留给提交时——那就是没在第 2 步判过就合，正是「合进主工作区的只有第 1、2 步在同一棵快照上都绿了的那一批」那一句不许的。定义没写第三条路，主 agent 只能自己选一条。
- 对照：同一个第 2 步里按块的崩溃用例交的是门禁 `.claude/gate.d/crash-point-replay.sh`，那一道没有这个上限（`grep -c 'scoped_domain\|SCOPED_CRASH_STATES_LIMIT\|200000'` 是 0，量的），全域照跑；第 1 步验证包那一步超上限会自己缩到 n 或 5、汇总写缩过域（`scripts/gate/merge-and-verify.sh:237`），由第 2 步那一格兜住全域。放量分项没有这一兜。
- 四句：判别子——退 79 的说明写明估了多少态；字面分句——第二个攻击面「已定的做法里有没有漏掉的情形」与第三组第 2 条「落在流程的哪一步」；改法（零轮）——见「替代做法」第 9 条，交用户定。

### T8 第 2 步某一项全域超四小时：服务进队就打回，定义没写怎么走（推的，没找到实例）

- 共用约束：`.claude/agent-common.md:69`「预计到四小时的进队那一刻与跑起来之后都直接打回、不给填 why，出路是收小范围或分片重交」；服务：`task-server/task_server.py:903`「不进 awaiting-long-run-why，当场记 over-four-hours（一次都没起），不给填 why」。分片的按片算。
- 第 2 步要的是全域（变异行、按块用例），「收小范围」里的缩域照第 68 行不算数；只剩拆测试（改代码，回写段，新一批补丁）或 `--full-gate`（要用户弹窗）。`.claude/main-agent.md` 验证段第 2 步对 over-four-hours 一句都没有。
- 两小时那一条不挡合：`.claude/rules/long-run-split.md:17`「撞了四小时上限被停的同样算跑完」之前一句是「让它跑完、出结局，结局照常用」，拆放进下一批；四小时被停的那一项没有结局，「都绿才进第 3 步」过不去。
- 实例：没找到。豁免表 `.claude/long-run-split-exempt` 此刻只有表头两行（量的），没有用户点名的长项；哪一项全域超四小时没量。所以这一格是「定义缺一支」，不是「今天一定发生」。

## 六、命令模板与缩域（第三组）

### T9 同一棵树上，定义的 grep 是闸的超集：量到 11 个目标 grep 说会走、闸说不会；照 grep 多设缩域变量过得了闸，代价是这一趟作废

- 定义：`.claude/agent-common.md:68`「做得了的判法：`grep -n '<目标名>' <树>/.claude/gate.d/stage-inputs.tsv <树>/scripts/crash/crash-amplification.sh` 有命中，或 `grep -c EXPAND_UP_TO <树>/crates/singlefs-verification/tests/<目标名>.rs` 不是 0，就会走」。
- 闸（`.claude/hooks/lib_run_scope.py`）：登记的只认 `crash-case:` 行里精确的 `test=singlefs-verification:<目标>:<用例>`（第 816 行 `targets.update(CRASH_CASE_TEST_WORD.findall(line))`）与 `ITEMS` 里成形的分项行；没登记的看源码全文里有没有两个变量的全名（第 841 行 `return any(name in text for name in CRASH_SHRINK_VARIABLES)`）。
- 两边的包含关系（读码）：变量全名里都含 `EXPAND_UP_TO`、精确的 `test=` 行里都含目标名，所以闸说会走的，grep 一定也说会走；反过来不一定。
- 量的（在主工作区 `crates/singlefs-verification/tests/` 顶层 466 份 `.rs` 上逐份判）：
  - 31 份写了 `EXPAND_UP_TO`，其中 19 份只是别的标识符（例：`gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker.rs` 第 47 行 `const EXPAND_UP_TO: usize = 8;`），两个变量全名都没写；这 19 份里 8 份没登记：`crash_judge_gpu_agrees_with_the_cpu_judge_on_root_ring_stride_refusals`、`gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker`、`gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker`、`crash_judge_gpu_tables_at_their_capacity`、`gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core`、`crash_judge_gpu_recovery_finds_central_mapping_entries_at_mapping_leaf_boundaries`、`crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal`、`crash_judge_gpu_reads_long_files_and_lineages_block_by_block`。
  - 另 3 个没登记的目标名字是别的登记行的子串：`instance_acquisition`（`stage-inputs.tsv` 第 55 行一个用例名里）、`random_histories`（第 42、95 行）、`writable_mount_of_a_formatted_pool`（第 151 行）。
  - 这 11 个，grep 判会走，闸判不会走（源码里没有变量全名、没登记）。探针第 1 条（`cargo test` 点名第一个、不缩域不写理由）期望闸退 0，第 2 条对照（点一个登记过的目标）期望退 2；同一趟打出 `name=definition_grep` 两行给 grep 的答案（推的，待主 agent 跑）。
- 后果：照 grep 走，这 11 个要么多设缩域变量、要么写全域理由。写理由的没有代价。设变量的：闸放行；按定义「缩域只当写段的快速反馈，判出的结论不算数」（第 68 行），还要再跑一趟；变异门禁那一侧设了变量就整趟当缩过域，「现判的不入库」（`scripts/mutation/crates-mutation-rows.py:226`）——而这些测试根本不读那个变量，跑的就是全域，结论白扔。不会被拒，只多一趟。
- 注释算不算：判会不会走，闸用全文（第 841 行，注释算），grep 也算注释，一致；判读哪一个，闸去掉注释（第 859 行 `rust_code_without_comments`），定义写「测试源码的代码里（注释不算）」，一致。工具员留的那一处两边对得上，没打中。
- 四句：判别子——闸手里有登记表与源码全文，分得出；字面分句——第三组第 1 条「对同一个目标会不会给出不同的答案」；改法（零轮）——定义的 grep 改成与闸同一个判法（`grep -E '^crash-case:.*test=singlefs-verification:<目标名>:'`、`ITEMS` 里 `\t<目标名>\t`、源码里两个变量的全名），或把闸的这一判做成只读的查询工具，定义指过去，不再抄一份。

### T12 照 grep 走到缩域那一条：变异门禁的缩域模板经服务交一律被它自己拒（退 2）

- 定义给的缩域写法：`.claude/agents/implementation-writer.md:34`「缩域写成 `task-server/task submit --cwd <根> -- env <缩域变量>=<n> bash .claude/gate.d/crates-mutation-replay.sh …`（这条门禁不认 `--crash-expand-up-to`，缩域只用变量）」；`.claude/agent-common.md:68` 的通用写法同形。
- 这一道经服务交一律分片：`.claude/gate.d/crates-mutation-replay.sh:2`「task-server-shards: *   经统一任务服务交的一律分片」。缩过域的那一趟不许分片：`scripts/mutation/crates-mutation-rows.py:233`「不分片：服务分片的计划与合并那一步调的 row-plan 退 2，分片的各片与合并那一步（TASK_SERVER_SHARD_PHASE 是 shard、merge）的 run_stage 退 2」，同一段的出路是「点名几行缩域证红走 prove-red.sh（不分片）」。
- 所以模板里变异门禁那一形照抄必退 2（读码，推的，没跑）；能缩域的只有证红那一形。T9 那 11 个目标照 grep 多设了变量的变异门禁命令，连「白跑一趟」都到不了，直接退 2。
- 四句：字面分句——第三组「命令模板与缩域」（模板本身，正文两条之外，同一句里的写法）；改法（零轮）——定义删掉变异门禁的缩域那一形，写段缩域只走证红；或者变异门禁缩域那一趟由服务不分片（脚本文件头声明改法，归工具员）。

### T10 树不一样：证红模板不带 `--cwd`，闸读主工作区；定义叫对着副本判。写段新用例的证红，闸一律「判不出、放行」

- 定义：`.claude/agent-common.md:68`「写段的命令对着自己改动所在的副本（这一次的登记行已写在副本里）」；`.claude/agents/implementation-writer.md:34`「写段的命令对着自己的副本判（这一次的登记行已写在副本里），自己新写的崩溃枚举用例一律按会走算」。证红模板：`.claude/agents/implementation-writer.md:37`「证红命令一律写成 `task-server/task submit -- bash scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名…>`」，没有 `--cwd`。
- 闸：`.claude/hooks/lib_run_scope.py:991`「domains = crash_subject_domains(subject, registry_root(subject.directory, project_root))」——登记表与源码取命令的当前目录那棵；证红只从 `--copy` 那棵读变异行（第 968 行 `tables = [os.path.join(base, arguments[1], MUTATION_TABLE), os.path.join(root, MUTATION_TABLE)]`）。主 agent 在仓根交，当前目录是主工作区。
- 新写的用例：变异行在副本里找得到，点的目标在主工作区既没登记、也没有源码 → 「判不出会不会走」→ 放行、记一条检出。实现员照定义设了变量的照放；忘了设、也没写理由的，一个全域证红照样过闸（定义说「一律按会走算」，闸那一侧没有东西兜）。探针第 3 条（不带 `--cwd`）期望退 0、第 4 条（同一条命令带 `--cwd <副本>`）期望退 2，另打一行检出记录里点到这个用例几次（推的，待主 agent 跑）。
- 反方向会被拒的（推的，没找实例）：主工作区里登记过、代码里读判器变量的目标，补丁在副本里把判器那一读删了，实现员照副本只设放量变量，闸照主工作区的源码要两个都设，拒。
- 四句：判别子——`--copy` 的值就在命令里，闸看得到副本在哪；字面分句——第三组第 1 条「对着哪棵树判」；改法（零轮）——证红模板加 `--cwd <副本>`（或闸对证红取 `--copy` 那棵当登记树），与变异门禁、`cargo test` 的模板对齐。

### T11 「验证段要在全域上证一次」落在哪：变异行在第 2 步、过得了闸；`cargo test` 那一形与整趟才跑的那几类没有落点

- 变异行：`.claude/main-agent.md:58`「在全域上判（点到会走崩溃枚举的目标的，范围声明的为什么一段末尾写全域理由」，主 agent 在第 2 步对快照交，命令照实现员第 1d 步验证段那一条。闸：`.claude/hooks/lib_run_scope.py:989` 写了全域理由就放（读码；第二轮探针第 5 条证红那一形已复跑退 0，这一形同一判法）。不缩域的变异门禁分片照跑，T12 那一条不碍它。
- 按块的崩溃用例：第 2 步交门禁那一格，没有 20 万上限（T7 对照那一段），全域照跑。放量分项：T7，落不下。
- `cargo test` 点名的验证包目标：共用约束 `.claude/agent-common.md:68` 说「缩域只当写段的快速反馈，判出的结论不算数，同一行、同一个目标在验证段第 2 步要在全域上判过一次」，`.claude/main-agent.md` 第 2 步的清单里没有 `cargo test` 那一形。全域那一趟实际只可能在第 1 步：合入后验证只跑补丁改到的测试目标与变异行点名的（`scripts/gate/merge-and-verify.sh:76`「src/ 下别的（不论是不是 .rs）→ 库目标」）。走法（推的）：实现员改了验证包 `src/` 里的枚举代码，写段拿一个没登记、读缩域变量的集成测试目标缩域跑过一次当快速反馈；补丁没碰那个目标的文件、也没有变异行点它，第 1 步只挑库目标，第 2 步清单里没有它——定义说要的全域那一趟哪一步都不跑，合进主工作区之后第一次全域是提交时。
- 整趟才跑的那几类：实现员第 1d 步给了 `crash-point-replay.sh --item whole-run:<类名>` 的模板，没写缩域或全域理由。闸判入口点名的项读合表，合表里 `whole-run` 出现 0 次（`grep -c`，量的），于是「判不出、放行、记检出」。那一类里的 `crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state` 代码里读判器变量（它第 338 行 `const EXPAND_UP_TO_ENVIRONMENT_VARIABLE: &str = "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO";`）：同一个目标用 `cargo test` 点名要设变量或写理由，走整趟那一类就不用。两条路判法不一，定义没说哪一条算第 2 步的全域。
- 由谁交：都是主 agent；实现员只写模板（`<快照>` 占位）。过不过闸：变异行那一形过；放量那一形过闸、被入口拒（T7）；四小时见 T8。
- 四句：字面分句——第三组第 2 条「落在流程的哪一步、由谁交、交的命令过不过得了闸」「定义之间打不打架」：共用约束说第 2 步、主 agent 定义第 2 步不列，打架；改法（零轮）——共用约束那一句改成「验证段第 2 步变异门禁点名那一行在全域上判；`cargo test` 点名的目标在第 1 步没跑到的，第 2 步照点名测试目标那一条不缩域交一次」，主 agent 第 2 步清单补上这一形。

## 七、这一轮新想到的替代做法（都是零轮，只在推演上；没在任何模型上量过）

| # | 做法 | 修哪一格 | 量过还是推的 |
|---|---|---|---|
| 1 | 登记表照变异表那样按键合：实现员与主 agent 交的不是 `stage-inputs.tsv`、合表、拒算表、`ITEMS` 的文本差，而是「按键追加 / 换 / 删」的行（键分别是 `crash-case:<名>`、合表项名、测试函数全名、分项名）；`apply-writer-patch.py` 与合成补丁那一段照名字合，不走 `git merge-file` | T2、T3（R0 同理） | 推的；`git merge-file` 在两边末尾追加时冲突是量的 |
| 2 | 第 3 步合的那一条三方冲突、冲突文件落在指纹不罩的那几份（登记表、变异表）：一律第 1 步整批重交，不「再交同一条」；也不照脚本「手工并好再打」 | T2 | 推的 |
| 3 | 拒算清单只认这一批：同一趟在打补丁之前的快照上也取一份 `list`（或拿快照那一刻的主工作区取），差集归这一批，其余照第 15 条交给它的会话；「未登记」「已不拒算」同一种分法 | T5 | 推的；要先量今天主工作区上有没有未登记（命令第 2 条） |
| 4 | mutation-tables 那一格挪进第 2 步对快照判；合完再对主工作区判一次只当兜底，红了照第 15 条交那一行的会话 | T4 | 推的 |
| 5 | 快照认归属：合入后验证在日志目录写一份归属记录（会话号取环境，写法同崩溃放量运行目录的 `.owner`），完工清理闸把 `merge-and-verify/*/repo` 当运行目录认；定义补「退 3 用户不同意、放弃这一批、被停」三支照删，第 2 步还有任务开着时先 `cancel` 再删 | T6 | 推的；13 棵、11G 的留存是量的 |
| 6 | 第 2 步红了：照新一趟脚本打出的那一段与「这一段全部补丁改到的项」重交，删「照原来点的全部项」；脚本收变异行名单时减去后面补丁的删除 | T1 | 推的 |
| 7 | 证红模板加 `--cwd <副本>`（或闸对证红取 `--copy` 那棵当登记树） | T10 | 推的；探针第 3、4 条待跑 |
| 8 | 定义的 grep 换成与闸同一个判法，或把闸这一判做成只读查询工具、定义指过去；删掉变异门禁的缩域那一形 | T9、T12 | 11 个分歧目标是量的；变异门禁缩域退 2 是读码推的 |
| 9 | 第 2 步碰上放量分项退 79 或某一项超四小时：定义写成弹窗交用户三选一——带 `--full-gate` 在第 2 步全域跑完再合；先拆再合（拆的改动进写段下一批，这一批停在验证段）；明写「这一项留给提交时」并照样合（写进调度记录，提交时那一趟红了照第 15 条） | T7、T8 | 推的 |
| 10 | 共用约束「同一个目标在验证段第 2 步要在全域上判过一次」与主 agent 第 2 步清单对齐：第 2 步补「写段缩域跑过、第 1 步没跑到的 `cargo test` 目标不缩域交一次」 | T11 | 推的 |

## 撤回与收窄我先前提的

- 第二轮 R1、R10 的改法「登记行跟着补丁走（补丁格式加一份 `.claude/gate.d/` 表的改动）」：收窄。跟着补丁走这一半照留（它消掉了第二轮的两处 ②），按文本差带过去那一半撤回：它带出 T3（第 1 步冲突、P3 退回重做）与 T2 的大半。换成「替代做法」第 1 条按键合。
- 第二轮 R6 的改法「`crates/mutations.tsv` 只核这批补丁自己那几行、别的行不进指纹」：收窄。现行版照这样做了，留下 T4；要配「替代做法」第 4 条（mutation-tables 挪进第 2 步）才完整。
- 第二轮 R7 的改法「第 2 步改在第 1 步留下的快照上跑」：不撤。它带出的是 T6 的留存与清理，不是验证上的洞。

## 没打中的形状

- 第一组第 5 条（合成补丁带进副产物）：见第四节，次序与范围都读码核过；措辞一处。
- 修补的实现员副本从快照建：快照根已 `git init`，`scripts/hosts/minimal-copy.py` 文件头「副本是 git 工作树」那一段认这种源树（读文件头，没跑）；没打中。
- 第 2 步在快照上起的那几条（`crash-case-check.py`、`refused-test-registry.py list`、形式化、变异门禁、命名检查）各自读的是快照里的登记表与脚本：读命令与脚本文件头核过，认、落在快照上；没打中。
- 第 1 步验证包那一步自己缩域（`scripts/gate/merge-and-verify.sh:237`）之后，登记用例在第 2 步门禁那一格全域重判，兜得住；没打中（放量分项兜不住，归 T7）。
- 第 2 步的变异门禁会不会把快照改脏、带进修补：每片一份源码副本，不在快照上就地改；没打中。
- 两次重交各留一棵的次序：修补写完才重交、重交时删旧的，次序对得上；没打中（T6 第 2 支是「还有任务开着」那一种）。
- 范围闸判「会不会走」时注释算不算：两边一致（T9 末段）；没打中。
- 取样范围：只走了「两批、三份补丁、一条新用例、一条拒算、第 2 步红一次、代码轮打中一次」一种布局。用户与别的会话决定的几步放开扫过：修补用「替换同名」还是「删了再起新名」（T1 只在后一种中）；第 1 步到第 3 步之间别的会话写不写登记表（T2、T3 只在写了时中）、主工作区上有没有别的会话没登记的拒算（T5 只在有时中）；退 3 时用户同不同意（T6 第 1 支只在不同意时中）。别的会话写这几份表的频率只量了五天里的总改动量，没量一段真实验证窗口里的次数。

## 这条腿自己的限度

- 全部是对着定义、脚本文件头与代码推的走法，没有真派实现员、没有交合入后验证、没有在快照上起门禁；T1、T2、T3、T5 的退出码都是读码推的。
- 量的只有：工作根留存的快照（`ls`、`du`）、`git merge-file` 两边末尾追加的冲突、登记表与验证包对 HEAD 的未提交改动量、11 个 grep 与闸分歧的目标（`grep` 逐份数）、几处 `grep -c`。
- 探针没跑；T9、T10 的闸那一侧退出码是「推的，待主 agent 跑」。T12 只读了变异脚本文件头，没交一次缩域的变异门禁看它退 2。T8 没找到真超四小时的项。
- 完工清理闸列不列快照是读它文件头推的，没喂钩子。

## 要主 agent 跑的命令

1. 范围声明闸探针（喂钩子，照主 agent 判；不起编译，不用 `task clean`），在仓根起：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/ledger-defs-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/ledger-defs-r3-snapshot /home/fy5090/.local/share/singlefs/work/ledger-defs-r3-opus-probe-run /home/fy5090/code/singlefs/research/prompts/ledger-defs-r3-opus-model
```

   跑完看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 4 行 `name=probe sample=… expected=… exit=… verdict=…`、末行 `name=probe_summary samples=4 mismatches=0`；`name=definition_grep tree=repo target=gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker … expand_up_to_count=2`（grep 说会走）与第 1 条 `exit=0`（闸说不会走）并排即 T9 坐实；`name=definition_grep tree=writer-copy … name_hits=1` 与第 3 条 `exit=0`、第 4 条 `exit=2` 并排即 T10 坐实；`name=detections_naming_the_new_case count=` 不是 0 即第 3 条是「判不出、放行、记检出」。`verdict=differs` 的那一条，照它的 `first_stderr_line` 与运行目录里 `<样本名>.err` 判那一格不成立。第 1、2 条读的是主工作区此刻的登记表与源码，主工作区改了这两个目标会跟着变。运行目录 `/home/fy5090/.local/share/singlefs/work/ledger-defs-r3-opus-probe-run`，核完删。

2. 今天主工作区上有没有别的会话没登记的拒算（T5 是不是今天的事；只算键、查判定库，不编），在仓根起：

```
task-server/task submit -- python3 scripts/gate/refused-test-registry.py list
```

   跑完看：末行 `TASK_SERVER_RESULT` 的 `exit=0`（77 是清单取不到、本次未跑，不算）；`task-server/task log <号> --all` 里行首「未登记」的行数。不是 0：T5 今天就会让任何一批的第 2 步判不完；是 0：T5 降成「主工作区上一有就会」。

3. `git merge-file` 两边末尾追加（git 自己的命令，不是项目代码；在主 agent 自己的草稿目录里直接跑，`<草稿>` 换成那个目录）：

```
cd <草稿> && printf 'row-a\trazon-a\nrow-b\treason-b\n' > base && { cat base; printf 'row-other\tanother session row\n'; } > ours && { cat base; printf 'row-batch\tthis batch refused row\n'; } > theirs && git merge-file -p ours base theirs > merged.txt; echo rc=$?; cat merged.txt
```

   跑完看：`rc=1` 与 `merged.txt` 里一对冲突标记，即 T2、T3 的前提坐实。

4. 量登记表的写频率（主 agent 的会话工具，在仓根直接跑，不经服务）：

```
python3 scripts/agents/who-touched.py .claude/gate.d/stage-inputs.tsv .claude/gate.d/refused-tests.tsv .claude/gate.d/crash-point-replay-items.tsv scripts/crash/crash-amplification.sh --since-minutes 1440
```

   跑完看：每份表一天里有几个会话的 Edit / Write 或 Bash 点到次数不为 0。不止本会话的，T2、T3「几个小时的窗口里碰上一次」站得住；只有本会话的，降成「可能」。

## 没做什么

- 不判正推那几格（第一组第 2 条、第二组数步数）；不替主 agent 采纳。
- 没往统一任务服务交任务；探针、拒算清单都没跑，报告里引它们的退出码与计数是「推的，待主 agent 跑」。直接跑过的只有只读命令、`bash -n`、`sha256sum`，与草稿目录里一次 `git merge-file -p`。
- 没建仓副本、没编译；草稿目录 `/home/fy5090/.local/share/singlefs/work/ledger-defs-r3-opus` 里只有文本：模型目录的草稿（`model-draft/`）、拼 `rerun_steps` 的 `rerun_steps.sh`、`mergefile-probe/` 下四份几十字节的文本。没有要删的副本与编译目录。
- `crates/` 下的产品代码没读；读了验证包 `tests/` 下几份测试源码，只为数 grep 与闸的分歧。
- 只判正文点名的句子；三份里别的会话的改动没攻。没读禁读清单里的另一条腿的报告，也没读名字相近的 `d137-ledger-defs-r*`。
