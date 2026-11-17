# ledger-defs-r2 攻方腿（opus）报告

日期：2026-11-12。被判的四份与开工快照 `research/prompts/ledger-defs-r2-snapshot/judged-sha256.txt`：开工时与写报告前各 `sha256sum -c` 一次，四份都 OK（量的）。

## 复跑命令与模型文件

这一腿写了一个小探针（喂范围声明闸的 hook 输入，照主 agent 判，核退出码），不是计数模型，不起崩溃枚举。模型目录 `research/prompts/ledger-defs-r2-opus-model/`；复跑命令（交主 agent 跑，见文末「要主 agent 跑的命令」）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/ledger-defs-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/ledger-defs-r2-snapshot /home/fy5090/.local/share/singlefs/work/ledger-defs-r2-opus-probe-run /home/fy5090/code/singlefs/research/prompts/ledger-defs-r2-opus-model
```

各文件 sha256（量的，在模型目录里 `find . -type f | sort | xargs sha256sum`）：

```
180919bb57094ab1a7caaa91aeb9eed9664d1549913b6e6ad6aa4eea083b8479  ./expected.tsv
5f33b8493052f0dd07de716a506fbea45cb9b94f184ac54f2f76eca2b41949ca  ./inputs/cargo-test-env-amplification-variable.json
531a146a7b058395a89da56b8b45546b9d81feb008260f626019dc35c3de1888  ./inputs/control-mutation-gate-no-shrink.json
b7d2fa7b8ae247ef2cba4baee7bbc6dc46c380c63b0ee5f59ebd00bb6da32180  ./inputs/mutation-gate-env-amplification-variable.json
b6cf9ded9e1b79d11e9a1a3862a5e50c2c92009371c14c30715caa74f5bd2612  ./inputs/mutation-gate-env-judge-variable.json
f5a0b3a7808dc747992ef62395fcfff1ae368e1d731b0a8cb4eded198aa27dd7  ./inputs/new-case-row-on-a-tree-with-its-registration.json
86c21dfb5eb8036bd7bada7b2dc8590da823bc658713861685895337bf194cf2  ./inputs/new-case-row-on-a-tree-without-its-registration.json
e31f41506e9695e68a52aac449a5a3ed3bf242d3d9feafaefdb1e42e7127ba7c  ./inputs/prove-red-env-amplification-variable.json
557a2ff32f80ede1f59028c895025a4f49d8f8c89024ac10f528a55e29d859e5  ./inputs/prove-red-full-domain-declaration-as-written.json
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
cde6d3be45bcf7cd417782ddca9cfbc7e16bca1d5400fae13db4949de04e01f4  ./tools/rerun.sh
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：先在运行目录里造两棵小的「段树样子」（同一条新变异行、同一份不写缩域变量名的新测试源码；一棵的 `stage-inputs.tsv` 登记了这条新用例，一棵没登记），再把 `inputs/` 里八份 hook 输入（`@REPO@`、`@WORK@` 换成参数给的仓根与运行目录）在 `setsid --wait timeout -k 5 60` 里逐份喂给仓里的 `.claude/hooks/run-scope-guard.sh`，退出码与 `expected.tsv` 逐行比，另打一行 `name=way_out`（不缩域那一条的拒绝说明点名的是哪个缩域变量）。末行 `name=probe_summary samples=8 mismatches=<M>`，M 为 0 退 0。外层 `rerun-from-a-clean-work-directory.sh` 与第一轮那一份逐字节相同（sha256 同上，第一轮报告里也是 e3294c15…）。探针还没跑：下文引它的退出码都是「推的，待主 agent 跑」。内存与挂钟（推的）：八次 python 钩子，每次几秒、几十 MiB，整条两分钟以内。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一·1 从头走一遍 | 打中（R1、R2、R3） | 登记行只在段树里，第 1 步快照里的新崩溃枚举用例是「没登记的枚举用例」：验证包那一步一起门禁就被开跑前的 placement 判红，不起时它在测试那一步当普通测试全域跑；`crash-case-check.py` 的模板不带根，判的是主工作区；补丁改了包的 `Cargo.toml` 第 1 步退 3、不打合进主工作区的命令，定义没有路 |
| 一·2 核指纹只罩改到的包 | 打中（R4、R5、R6） | 口子量的：验证包的补丁不罩它的 8 个 path 依赖；更大的一处是指纹按「最后一趟」记而绿是几趟拼的：照定义「只重跑红的」，最后那一趟只带 `--item`，它的指纹只罩它自己开跑以来；核完到 N 份补丁排队打完之间不再核；热包与 `crates/mutations.tsv` 改动频繁，核对一再退非 0、整批重来，定义没有出口 |
| 一·3 段树不与主工作区核 | 打中（R7） | 段树里的门禁脚本与登记表是建段树那一刻的拷贝，第 2 步用旧判法判新补丁；该核，改法（零轮）：第 2 步改在第 1 步留下的快照上跑，三棵树并成两棵 |
| 一·4 主 agent 补的两句 | 打中（R8、R9、R10） | 第 2 步红了的修补没改到「第 1 步验过的文件」时第 1 步不重交，第 3 步照旧那份命令合，修补不进主工作区、合进去的正是第 2 步判红的那个组合；只重交红的几条罩不住修补弄坏原来绿的几条；登记行：快照里没有、`insert-row.py` 只插新行改不了合表已有的行、两份没人核、合完补丁再写登记行的那段时间正是第一百九十五节 ② 的样子 |
| 一·5 照留底换回 | 打中（R11） | 换回会冲掉别的会话在 `crates/mutations.tsv` 上的追加（那份表此刻对 HEAD 未提交的改动 +1824 −1106，量的）；`check` 退 0 证不了新建的文件删干净；`apply-writer-patch.py` 自己逐份记了改、增、删与改后 sha256，定义没用 |
| 一·6 段树的寿命 | 打中（R12） | 合完之后段树没有任何一步再用（第 5 步照样重建），留着只剩被闸拦、被看门狗列的代价；合完之后跨轮停下再续做，照定义把已合的补丁重打进新段树，带变异行的补丁一律退 2 |
| 二·1 与「同一批派、一起打」比 | 打中一半（R13） | 两种改法都只挡第一百九十五节 ①（删接口）；② 是入口组的脚本改动直接进主工作区，两种都挡不住；新流程的手工步约多 n+r+5 步（推的），自己还在第 1 步与第 3 步各造出一次 ② 的样子 |
| 二·2 做错了没有检查会红的手工步 | 打中（R14） | 九处，列在表里 |
| 三·1 两条路到不到测试进程 | 部分打中（R15） | `-- env` 闸认、服务不拒、环境一路带到测试进程（推的，探针第 2–6 条待跑）；但闸认两个变量里任一个，拒绝说明给的出路是判器那一个，而 56 个登记的崩溃枚举目标经引擎读的都是放量那一个，照出路改的命令过得了闸、域没缩 |
| 三·2 缩过域的证红 | 打中（R16） | 定义没写；变异判定的行键不带缩域变量，缩域下判成「抓到」的那一行，提交时全域那一趟照键复用、再不在全域上判；缩域下「全量域计数」那一类断言不判，只靠它的新测试在缩域下证不红 |
| 三·3 怎么知道会走崩溃枚举 | 打中（R17） | 判法给了，没说对着哪棵树的登记表：实现员写命令时新用例还没登记，过闸；第 2 步登记写进段树之后同一条命令被拒（探针第 7、8 条，推的待跑）；56 个登记目标里 46 个的测试源码两个变量都没写，「写测试源码读的那一个」答不出 |

## 一、验证段的新次序

走法：一段两批有依赖的实现员。第一批两个（补丁 P1、P2，互不撞文件），第二批一个（P3，以打了 P1、P2 的段树为基线），P3 新写一条崩溃枚举用例（往 `tests/common_crash_points.rs` 里加了搭建函数、新开一个测试目标）与一条要登记的拒算测试。段树、实现员副本照定义建得成、补丁打进段树照定义打得进，第一轮已量过，不重复。下表只列这一轮新改的几步。

| 步 | 命令（今天认不认） | 落在哪棵树 | 读的登记表是哪一份 |
|---|---|---|---|
| 写段收尾 | 实现员报告里 `crash-case-check.py --only placement,…` 不带根（R2） | 主工作区（脚本自己所在的仓） | 主工作区的，里面没有 P3 |
| 第 1 步 | `merge-and-verify.sh P1 P2 P3` 认 | 主工作区此刻的快照 + P1–P3 | 快照里那一份 = 主工作区的，没有新用例的 `crash-case:` 行（R1） |
| 第 2 步 | 带 `--item` 四条、证红、`refused-test-registry.py list <段树>` 认 | 段树 | 段树的，登记行此刻才写进去；门禁脚本也是段树里那份旧拷贝（R7） |
| 第 2 步红了 | 修补打进段树、只重交红的那几条 | 段树 | 同上（R8、R9） |
| 第 3 步 | 核指纹（主 agent 直接跑）、`copy-originals.py copy`、N 条 `apply-writer-patch.py` 认 | 主工作区 | —（R4–R6、R11） |
| 第 3 步之后 | `insert-row.py` 认新行，改不了已有行（R10） | 主工作区 | 主工作区的 |

### R1 第 1 步的快照里没有新用例的登记，新用例在第 1 步要么把验证包那一步判红、要么全域当普通测试跑（一·1、一·4）

- 定义：`.claude/main-agent.md:58`：「先写进段树那一份供这一步判，写了哪几行原样记进调度记录；第 3 步合完补丁，把同样的几行经 `scripts/kb/insert-row.py` 写进主工作区那一份，再交合完才交的那几样」。第 1 步的快照取自主工作区：`scripts/gate/merge-and-verify.sh:26`：「开跑那一刻把要验的树（主工作区；补丁已打在上面的带 --skip-apply）拷进 <日志目录>/repo」；补丁只带 `crates/` 与 `litmus/`：`scripts/gate/apply-writer-patch.py:15`：「`git diff -- crates litmus ':!crates/mutations.tsv'` 的输出」。所以第 1 步那棵树里新用例的测试函数在、`stage-inputs.tsv` 的 `crash-case:` 行与合表那一项不在。
- 今天的脚本两条路，看验证包那一步起不起门禁：
  1. 起：P3 改了 `tests/common_crash_points.rs`，挑目标照 mod 往回落到引它的每个目标（量的：56 个登记的崩溃枚举目标里 50 个写了 `mod common;`、32 个写了 `mod common_crash_points;`，在 `stage-inputs.tsv` 现取目标、逐份 grep），里面有登记过的用例，验证包那一步就交门禁 `crash-point-replay`。门禁开跑先判整棵树：`.claude/gate.d/crash-point-replay.sh:1283`：「开跑前的静态判据（placement、modules；判法在 scripts/crash/crash-case-check.py）：真跑与样本档都在复用判定之前判，红了不起 cargo。」，`.claude/gate.d/crash-point-replay.sh:1293`：「if python3 "$crash_point_replay_static_judge" --only placement,modules "$ROOT"; then」。快照里那条新用例没登记，`scripts/crash/crash-case-check.py:295`：「调了崩溃枚举一族的函数却没有登记成 crash-case」——第 1 步红，红的不是代码，是定义把登记推到了第 2 步。脚本给的出路是「红的测试函数并进写段的下一批」，指错了方向。
  2. 不起（P3 只新开测试目标、不碰共用模块）：新用例在测试那一步当普通测试函数跑。排掉的只有登记过的：`scripts/gate/merge-and-verify.sh:53`：「登记成崩溃枚举用例与放量分项的用例函数经 --skip 排掉」；崩溃域估算只对要交门禁的登记用例做，这一条没人估，全域跑（推的：读脚本文件头，没跑）。
- 四句：判别子——第 1 步那一刻主 agent 手里有段树里写好的登记行，看得到；字面分句——一·4「第 1 步的快照里有没有这些登记行（没有的话第 1 步验的东西缺什么）」；改法——在第 1 步开跑之前把登记行写进主工作区，又回到第一轮 G1-D 那一支（主工作区先有了登记、测试函数还没合进去）；能同时满足两边的是让登记行跟着补丁走（补丁格式加一份 `.claude/gate.d/` 表的改动，`apply-writer-patch.py` 打快照、段树、主工作区三处都带上），零轮，只在推演上。

### R2 实现员定义里核新用例登记的那一条，模板判的是主工作区（一·1）

- `.claude/agents/implementation-writer.md:49`：「交回前把 `task-server/task submit -- python3 scripts/crash/crash-case-check.py --only placement,modules,provenance,must-red` 写进报告」。不带 `--cwd`、不带根：`scripts/crash/crash-case-check.py:933`：「root = REPOSITORY_ROOT」，`scripts/crash/crash-case-check.py:143`：「REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))」——脚本自己所在的那棵，主 agent 在仓根交就是主工作区，P3 不在里面，这一条判不到新用例。
- `.claude/main-agent.md` 里 `crash-case-check` 出现 0 次（量的，`grep -c`）：第 2 步的清单也不起它。新用例的 placement、provenance、must-red 在合进主工作区之前没有一处判，第一次判是提交时的门禁，或别的会话的门禁（R10 第 4 点）。
- 改法（零轮）：第 7a 步那一条写成 `--cwd <段树>` 并带 `<段树>` 根，排进验证段第 2 步、在登记行写进段树之后交。

### R3 补丁改了包的 `Cargo.toml`，第 1 步退 3、不打合进主工作区的命令，定义没有路（一·1）

- 判不出测试目标的几种：`scripts/gate/merge-and-verify.sh:86`：「改到工作区根的 Cargo.toml、Cargo.lock 或 .cargo/；改到不落在任何包里的路径；落在包里而照上面几条落不到测试目标的」，下一行 `scripts/gate/merge-and-verify.sh:87`：「（包的 Cargo.toml、build.rs、tests/ 顶层的数据文件这类）」。退 3 时：`scripts/gate/merge-and-verify.sh:146`：「带 --skip-apply 的那一趟（补丁已在主工作区上）与退 1、2、3、75 的都不打这几条」。
- 定义只有「绿了（`exit=0`）不合、接着做第 2 步」一支（`.claude/main-agent.md:57`：「这一条绿了（末行 `TASK_SERVER_RESULT` 是 `exit=0`）不合，接着做第 2 步」），第 3 步又只认第 1 步打出的那一段。补丁加一条 dev 依赖、给验证包加一个 `[[test]]`，第 1 步退 3，第 3 步无命令可照；「整包归提交时」管的是测试，管不到合。主 agent 只能自己拼 `apply-writer-patch.py`：没有指纹、没有留底，落进 R14 那几处。
- 改法（零轮）：退 3 时脚本照样打合进主工作区那一段（指纹照记），定义写明退 3 的合法路径与它不算验过哪几样。

### R4 指纹罩的是「补丁改到的包」，第 1 步的绿依赖的远不止这些（一·2，量的）

- 定义：`.claude/main-agent.md:59`：「头一条在仓根核主工作区自第 1 步开跑那一刻以来，补丁改到的文件与它们所在的包有没有变」。脚本同口径：`scripts/gate/merge-and-verify.sh:152`：「退 0 是补丁改到的文件与它们所在的包自开跑以来一份没改、没加、没删」。字面写清了范围，没写的是这个范围之外第 1 步的绿还依赖什么。
- 包之间（量的：在仓根直接跑 `cargo metadata --format-version 1 --no-deps --offline`，按 path 依赖算各包的测试闭包——本包 normal 与 dev 依赖、再往下 normal 与 build 依赖）：
  - 只改 `crates/singlefs-verification` 的补丁：指纹罩它一个目录；它的测试编进去的 path 依赖 8 个不罩：`singlefs-checker`、`singlefs-core`、`singlefs-device-library`、`singlefs-format`、`singlefs-reference-filesystem`、`singlefs-replay-prefix`、`singlefs-unit-graph` 与仓根的 `prune-server`。
  - 只改 `crates/singlefs-background-gpu-tests`：不罩的 11 个（工作区其余全部加 `prune-server`）。
  - 只改 `crates/singlefs-core`：不罩 `singlefs-format`、`singlefs-replay-prefix`。
- 包之外第 1 步也在读、指纹不罩的（读脚本文件头，推的）：工作区根 `Cargo.toml` / `Cargo.lock` / `.cargo/`（补丁带不了，别的会话改得了）；`.claude/singlefs-ai-sop/scripts/check.sh`（clippy 的 lint 表与特性组合每次开跑现读，`scripts/gate/merge-and-verify.sh:94` 一段）；`.claude/gate.d/stage-inputs.tsv`（哪些用例经 `--skip` 排掉、交门禁）与快照里那一份门禁脚本；`litmus/` 下补丁没碰、测试编译期读的文件。
- 举例：P3 只改验证包，第 1 步与第 3 步之间别的会话改了 `singlefs-checker` 一条不变量的判定集合（第 2 步要几个小时，见 R6），核对退 0，合进去的是「新 checker + P1–P3」，第 1 步跑的是「旧 checker + P1–P3」。
- 四句：判别子——核对那一刻主 agent 只看得到 diff 的输出，看不到依赖包变了；字面分句——一·2「这个口子有多大，定义写没写」：有多大量的如上，定义没写；改法——把起点扩到「补丁改到的包的测试闭包」（照 `cargo metadata` 算，加仓根那几份），代价是 R6 的退非 0 更频繁；零轮。

### R5 指纹按「最后一趟」记，第 1 步的绿是几趟拼起来的（一·2，读码量的）

- 定义叫第 1 步红了只重跑红的：`.claude/main-agent.md:57`：「那一行里打出只重跑红的那几项的整条命令」。那一条：`scripts/gate/merge-and-verify.sh:153`：「测试那一步红：只重跑红的那几项的合入后验证（merge_rerun_command）不带 --skip-apply、带 --item <红的项>… 与同一批补丁目录」。
- 这一趟不带 `--skip-apply`，开跑照样记一份新指纹（`scripts/gate/merge-and-verify.sh:3948`：「：不带 --skip-apply 的那一趟，拷快照之前记」），绿了照样打合进主工作区那一段（`scripts/gate/merge-and-verify.sh:3990`：「if ((verify_rc == 0)) && { ((skip_apply == 0))」）。第 3 步照「第 1 步那一条」打出的做，拿到的是最后这一趟的指纹。
- 走法：T0 第 1 步整批跑，A、B 两项绿、C 红；修 C；T1 只带 `--item C` 重跑，绿。A、B 是在 T0 的主工作区上绿的，T0 到 T1 之间补丁改到的包变了，T1 记的指纹已经是变过之后的样子，第 3 步核对退 0。A、B 在合进去的那棵树上没判过。`--item` 那一趟不跑 A、B，也就不按键复用它们（复用只在跑到的项里发生）。
- 四句：判别子——主 agent 看得到「最后一趟带 `--item`」，定义没叫它看；改法——核对的起点取「整批第 1 步最早那一趟」（脚本在 `--item` 重跑时沿用前一趟日志目录的指纹，或拒绝在带 `--item` 的那一趟打合入命令），零轮。

### R6 核完一次就合 N 份，之间不再核；热包上核对一再退非 0，定义没有出口（一·2）

- 核与合之间：第 3 步头一条核、再留底、再「逐份 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录>`」，前一条退 0 才交下一条。N 份各要进服务的队、起、等结局。这段时间别的会话改了补丁碰的文件里不重叠的几行，`apply-writer-patch.py` 照三方合并静默合上：`scripts/gate/apply-writer-patch.py:100`：「全部合得上的，在临时仓里照…重出一份补丁」。合进去的是第 1 步没见过的内容，核对那一条早就退 0 过了。窗口长度是 N 次排队加执行，推的（没量服务的排队时长）。
- 热包：有变异行的补丁把 `crates/mutations.tsv` 整份算进起点（`scripts/gate/merge-and-verify.sh:2696`：「        paths.append("crates/mutations.tsv")」）；每条新测试都要一行变异（`.claude/agents/implementation-writer.md:36` 第 3 步），所以带新测试的补丁都带它。量的：`git diff HEAD --numstat -- crates/mutations.tsv` 是 `1824	1106`（HEAD 是 2026-11-07 那一次提交，五天里没提交的增删），文件修改时刻是今天；`crates/singlefs-verification` 657 份文件里 153 份修改时刻在 2026-11-11 之后、9 份在今天（`find -newermt`）。第 1、2 步加起来几个小时（推的，第一轮也这么估），别的会话往这两处写一笔的可能很大（推的；可以用 `who-touched.py` 量，见「要主 agent 跑的命令」第 2 条）。
- 退非 0 的出路是「重建段树、重打、第 1、2 步重验」，几个小时之后再核，再碰上别人写一笔又重来。`crates/mutations.tsv` 上别的会话追加的是不相干的行，对这批补丁的判定没有影响，照样判「变了」。定义没写重来几次之后怎么办，也没区分「只追加了别人的变异行」与「改了同包的代码」。
- 四句：判别子——核对的输出逐行点名哪份文件变了，主 agent 看得到是谁的哪一行；改法——`crates/mutations.tsv` 只核这批补丁自己那几行（按变异名）在不在、没被改，别的行不进指纹；合进去之后再核一遍「补丁改到的文件此刻 = 合后」（`apply-writer-patch.py --read-outcome` 已有这一判，见 R11）；零轮。

### R7 段树里的门禁脚本与登记表是建段树那一刻的拷贝，第 2 步用旧判法判新补丁（一·3）

- 段树照写段第 2 条带 `--with-gate-tables`、`--with scripts=…`、`--with .claude/scripts=…` 建：`scripts/hosts/minimal-copy.py:15`：「--with-gate-tables 再声明 .claude/gate.d/（要门禁表的副本）」。工具员直接在主工作区改门禁与脚本：`.claude/agents/tooling-writer.md:15`：「在主工作区改。」。段树建好之后主工作区里门禁 `crash-point-replay.sh`、`crates-mutation-replay.sh`、`crash-case-check.py`、合表、`stage-inputs.tsv` 怎么改，第 2 步都用的是旧的那份。第一百九十五节 ② 的事故正是入口组改了登记检查（`records/2026-11-05-删轻型并入重型.md:161`：「入口组按条款删了…的例外」）：那种改动发生在段树建好之后，第 2 步判绿的是旧判法，合进主工作区之后第一次碰上新判法。
- 第 1 步那一趟的快照是开跑那一刻的整个主工作区（含门禁与表），反倒是新的；第 2 步比第 1 步旧。三棵树：第 1 步 = 主工作区@T1 + 补丁；第 2 步 = 主工作区@建段树 + 补丁；合进去 = 主工作区@合 + 补丁。指纹只把第 1 步与合连起来（还只连补丁改到的包），第 2 步与谁都不连。
- 该核。怎么核（零轮，只在推演上）：第 2 步不在段树上跑，在第 1 步留下的快照上跑——`scripts/gate/merge-and-verify.sh:35`：「要留现场：MERGE_AND_VERIFY_KEEP_COPY=1」，快照本来就是「主工作区@T1 + 全部补丁」、带门禁与表、已 `git init`；登记行写进快照；第 3 步的指纹同时罩住第 1、2 步。代价：快照在工作根下多留几个小时；第 2 步红了要修补时，修补以快照为基线、第 1 步连同修补重交（新快照），段树只剩写段用。另一种是第 2 步开跑前比段树与主工作区（补丁没碰的文件逐份比），不一致就重建——比起来贵、窗口照旧在。

### R8 第 2 步红了，修补没改到「第 1 步验过的文件」时，修补合不进主工作区，合进去的正是第 2 步判红的组合（一·4）

- 主 agent 补的句子：`.claude/main-agent.md:58`：「修补的补丁照写段第 2 条打进段树，只重交红的那几条（同一条命令、`--item` 只点红的项），绿了再往下；修补改到第 1 步验过的文件的，第 1 步连同修补的补丁目录重交一遍」。
- 第 3 步只照第 1 步打出的那一段合：`.claude/main-agent.md:59`：「照第 1 步那一条在汇总行之后打出的…那一段逐条做」。那一段只列第 1 步给过的补丁目录：`scripts/gate/merge-and-verify.sh:2567`：「  for dir in "${CHECK_PATCH_DIRS[@]}"; do」，留底清单与指纹起点也只从这几份补丁算。
- 走法：第 2 步 formal-verification 的一项红（或新变异行没红），修补 R 改的是一份规约文件或一份测试文件，P1–P3 都没碰它。照「第 1 步验过的文件」最自然的读法（第 1 步的补丁改到的文件），第 1 步不重交。R 打进段树，第 2 步只重交红的那一项，绿。第 3 步照第 1 步那一段合 P1、P2、P3：R 不在列，主工作区得到的是「P1–P3、没有 R」——第 2 步刚判红的那个组合。定义没有一句叫人把 R 加进合入命令；主 agent 自己加上的话，R 的包不在指纹起点里、R 的文件不在留底清单里，R11 的回退也退不了它。
- 「第 1 步验过的文件」另有两种读法（第 1 步跑到的测试目标的文件；第 1 步编过的全部文件——`build --all-targets` 编整个工作区，等于每次都重交），三种读法给出的「重不重交」不一样，定义没定。
- 四句：判别子——主 agent 打 R 进段树那一刻看得到 R 改了哪些文件，也看得到第 1 步合入那一段没有它；字面分句——一·4「第 2 步红了…照着做」；改法——第 2 步有修补就一律连同修补重交第 1 步（合入命令、指纹、留底都从新的那一趟取），零轮；这样「第 1 步验过的文件」这半句可以删掉。

### R9 只重交红的那几条，罩不住修补弄坏原本绿的几条（一·4）

- 同一句「只重交红的那几条（同一条命令、`--item` 只点红的项）」。修补 R 改的是代码，第 2 步原本绿的几项（另一条崩溃枚举用例、另一项形式化、另一行变异）在 R 之后没再判。第 3 步照合。
- 第 2 步那几道都按单元键复用（崩溃状态块、变异行键）：照原来那一整条重交，键没变的照复用、只现判 R 牵连到的，不比「只点红的」贵多少（推的，没量）。「只重跑红的」在第 1 步里有键复用兜底，搬到第 2 步就没有：第 2 步点名以外的项根本不起。
- 改法（零轮）：第 2 步红了修补之后，第 2 步那几条照原来点的全部项重交，靠键复用省时间。

### R10 登记行抄两遍：插得进新行、改不了旧行，两份没人核，合补丁与写登记之间又是一次第一百九十五节 ②（一·4）

1. `insert-row.py` 只插整行：`scripts/kb/insert-row.py:11`：「  insert-row.py 文件 --after 正则 --line 文本 [--absent 正则] [--dry-run]」，一次一行、锚点恰好命中一次。合表一项一条流，新用例接在已有的流上时改的是已有那一行的第二列：`.claude/gate.d/crash-point-replay-items.tsv:7`：「#   ② 崩溃点重放那一套的部分：stage-inputs.tsv 里 crash-case: 的键，几条空格隔开」。这一笔 `insert-row.py` 写不了（要 `replace-once.py`，`.claude/main-agent.md` 里 0 处提到它，量的）。`stage-inputs.tsv` 的 `crash-case:` 新行、`refused-tests.tsv` 新行、`crash-amplification.sh` 的 `ITEMS` 新行写得了。
2. 两份没人核：段树那份与主工作区那份各由主 agent 手写一次，中间隔着调度记录的一次转抄。段树留到代码轮判决，可没有一步拿它与主工作区比；`crash-case-check.py` 在合之前不判新用例（R2）。抄错一字（目标名、用例名）要到提交时的门禁才红。
3. 段树里锚点那一行，主工作区里可能已被别的会话改过、挪过，`--after` 命中 0 次或几次就退 3，主 agent 现改锚点——第三份手写。
4. 次序：定义写「第 3 步合完补丁，把同样的几行经 `scripts/kb/insert-row.py` 写进主工作区那一份」。N 份补丁排队合完到 r 行登记插完之间，主工作区里有「调了枚举函数、没登记」的新用例。别的会话这时起的合入后验证，快照取的正是这个样子，验证包那一步一起门禁就被 placement 判红（R1 第 1 条同一条路）——`records/2026-10-29-subagent拆分提案.md:1058`：「入口组改过的登记检查同样让 38 的合入后验证（6754）在 checker 那一步开跑前就红」。窗口比当时短（几分钟到几十分钟，推的），形状一样。反过来先插登记、再合补丁，窗口里是「登记了、测试函数不在」，placement 同样判红（推的，没读到这一支的原句）。只要登记与补丁分两种命令、各自进主工作区，就有这一段。
- 改法（零轮）：登记行跟着补丁走（R1 末尾那一条），三棵树由同一个打补丁的脚本一次写，主 agent 不手抄。

### R11 照留底换回：冲掉别人的追加、证不了新文件删干净、放着脚本自己的记录不用（一·5）

- 定义：`.claude/main-agent.md:59`：「已合的补丁的 `crates.patch` 里新建的文件删掉，再 `check` 一遍退 0，主工作区回到合之前」。
- 冲掉别人的：留底清单里有 `crates/mutations.tsv`（R6 第 2 条）。合到一半那段时间别的会话照「只追加、不算撞文件」往里追加（五天未提交 +1824 −1106，量的），`copy-originals.py check` 列它「改过」，照定义「逐份用留底的拷贝换回」，别人的行一起没了。`check` 只比 sha256，不分「我合的」与「别人改的」：`scripts/agents/copy-originals.py:18`：「check 用清单开头记的仓根；改过的逐行写…不在了的写…」。工具员报告第七节第 5 条说「这个窗口很小」，对 `crates/mutations.tsv` 不小。
- 证不了删干净：留底只收当时已在的文件（`scripts/agents/copy-originals.py:14`：「<文件> 写相对仓根的路径或仓内的绝对路径，要是仓里已有的普通文件」），新建的文件不在清单里，`check` 退 0 与它们删没删无关；「回到合之前」这半句没有东西判。从 `crates.patch` 里认「新建」靠人读补丁；三方合并重出的补丁不认改名（`apply-writer-patch.py` 文件头「不认改名」），实现员交的原补丁带改名段时，改名的目标不是 `new file`，照读会漏（推的）。
- 脚本自己记了：`scripts/gate/apply-writer-patch.py:46`：「    文件<TAB>相对仓根的路径<TAB>动作（改、增、删）」，每份补丁目录里一份 `apply-outcome.tsv`，带补丁前与回读的 sha256；`scripts/gate/apply-writer-patch.py:54`：「4 完成或退回之后文件又被改了（逐个点名）」（`--read-outcome`）。拿它退回：「增」的照删、「改」「删」的只在此刻 sha256 等于回读那一份时才换回，不等的点名交人——别人的追加不会被冲掉。定义没用它。
- 只管退 2：被服务停掉、撞内存上限这类不是 2 的结局，`apply-writer-patch.py` 留着暂存目录（文件头「换上与被杀」），下一次往主工作区打补丁先拒；定义没写这一支（推的）。
- 改法（零轮）：回退照各份 `apply-outcome.tsv` 逆序做、先 `--read-outcome` 判每份是不是还等于合后；整批留底只当兜底。

### R12 段树的寿命：合完之后没有用处；合完之后跨轮续做，照定义重打已合的补丁会退 2（一·6）

- `.claude/main-agent.md:47`：「段树留到这一段的代码轮判决写完再删（整段不开代码轮的，照验证段第 3 步合完就删）」。合完之后谁还用段树？代码轮打中的修补：`.claude/main-agent.md:61`：「以主工作区此刻照写段第 2 条重建段树，修补以它为基线写、打进段树」——重建，不用留着的那一棵。验证段别处也不再读它。合完到代码轮判决写完这段时间（一轮三方，几个小时到一两天），段树只剩代价：完工清理闸把它当仓副本列、主 agent 声明不了（第一轮 G1-J）；看门狗 `scripts/agents/watch.conf:31`：「scratch-idle-hours=6」把没动的副本列给主 agent 删。代码轮打中之后同时有两棵「段树」，第 2 步的 `<段树>` 指哪一棵定义没说。
- 跨轮停下：`.claude/main-agent.md:47`：「一段跨轮停下就删，续做时以主工作区此刻照这一条重建段树，把这一段已有的补丁照调度记录里打的次序再打进去」。停在第 3 步合完之后、代码轮判决之前（段树还在的那段），续做时主工作区里已经有 P1–P3，再打一遍：变异行撞名，`scripts/gate/apply-writer-patch.py:16`：「    mutations-append.tsv         要追加的变异行，每行六段（与 crates/mutations.tsv 同），名字在表里不许已有」，整份退 2、一个字不改；带新测试的补丁都带变异行，等于每份都退 2。不带变异行的那几份三方合并两边改得一样、静默合上、什么都没改（推的，读 `apply-writer-patch.py` 文件头「三方合并」）。停在第 3 步合到一半（回退之前被打断）同理：已合的几份退 2，没合的照打。重建出来的东西取决于停在哪儿，定义只写了「已有的补丁照次序再打」。
- 改法（零轮）：段树第 3 步合完就删（代码轮修补本来就重建）；续做时只重打「调度记录里打进段树、还没合进主工作区」的那几份，判据用各补丁目录的 `apply-outcome.tsv`（有「完成」行的是已合）。这一条撤回第一轮 G1-J 我提的「许主 agent 把段树声明成新种类」：合完就删之后，只剩写段里跨轮那一种要留，那一种定义已写「停下就删」。

## 二、整套流程的操作面

### R13 与「删接口与改调用方的补丁同一批派、一起打」比（二·1）

第一百九十五节那一次是两件事：`records/2026-10-29-subagent拆分提案.md:1058`：「删了 `crash::in_place_subsets_with_copy_on_write_none_or_all_expansion` 与 `crash_state_enumeration::enumerate_crash_states_quick_tier_versions`」（① crates 补丁先删接口、调用方下一批才改），同一行「入口组改过的登记检查同样让 38 的合入后验证（6754）在 checker 那一步开跑前就红」（② 入口组的脚本改动先进了主工作区，用例与登记没跟上，`records/2026-11-05-删轻型并入重型.md:161` 那一行写的是这件）。另一种改法原文：`records/2026-10-29-subagent拆分提案.md:1060`：「或者删接口与改调用方的补丁同一批派、一起打」。

| | 段树 + 先验后合（今天的定义） | 同一批派、一起打（补丁照旧打主工作区） |
|---|---|---|
| 挡 ① | 挡：写段补丁不进主工作区 | 挡，前提是主 agent 把删接口与改调用方分进同一批；分错了没有检查（派发闸只判撞文件） |
| 挡 ② | 不挡：工具员直接改主工作区（`.claude/agents/tooling-writer.md:15`：「在主工作区改。」），段树只收 `crates/`、`litmus/` 的补丁 | 不挡，同 |
| 自己造出 ② 的样子 | 造两次：第 1 步快照里的新用例没登记（R1），第 3 步合完补丁到插完登记之间（R10 第 4 条） | 照旧：补丁与登记都在写段直接进主工作区，次序由主 agent 排 |
| 写段里主工作区有没验过的补丁 | 没有（这是它要买的那一件） | 有，每一份补丁打进去到验证段之前都是 |
| 主 agent 手工步（n 份补丁、r 行登记、第 2 步 q 条，推的） | 建段树 1、核撞文件约 n、打进段树 n、记次序 n、第 1 步 1、登记写段树 r 与转抄 r、第 2 步 q、核指纹 1、留底 1、合 n、插登记 r、mutation-tables 1、删段树 1：约 4n + 2r + q + 6 | 核撞文件约 n、打进主工作区 n、记次序 n、登记写主工作区 r、合入后验证（`--skip-apply`）1、第 2 步那几条 q、mutation-tables n：约 4n + r + q + 1 |
| 做错了留下没验过的主工作区、又没有检查会红的步 | R14 表里九处 | 分批分错；补丁打错次序（`--skip-apply` 会核次序，红）；其余与今天的旧流程同 |

- 结论（推的，没真走一段）：在第一百九十五节那一次上，两种改法挡住的是同一半；段树多挡住的是「写段里主工作区有没验过的补丁」这一条本身，代价是约 r + 5 步手工、三棵树的不一致（R4–R7）与它新造的两处 ② 形状（R1、R10）。另一种改法少的这几步里，「核指纹」「留底」「回退」三处在它那里根本不存在，因为它不合；它的毛病是写段里主工作区本就不干净，别的会话撞上半成品的概率随写段长度涨。
- 什么现象会推翻：第一百九十五节 ② 那一半其实是 crates 补丁带进去的（例如登记检查住在 `crates/` 下）——那样段树也挡 ②，表里第二行要改。我读到的是入口组改的是 `scripts/crash/` 与门禁表（`records/2026-11-05-删轻型并入重型.md:161` 那一行），没去查那一批的 diff。

### R14 主 agent 手工做的步里，做错了没有任何检查会红的（二·2）

| # | 步 | 做错的样子 | 会不会有东西红 |
|---|---|---|---|
| 1 | 第 1 步给的补丁目录 | 漏给一份、次序与段树不一致 | 不会：段树的 `<git common-dir>/singlefs-applied-writer-patches.tsv` 记着打进段树的那几份，没人拿它与第 1 步的参数比；合进主工作区的是第 1 步给的那几份 |
| 2 | 第 2 步点哪些 `--item` | 漏点补丁改到的项 | 不会：第 2 步的项是人挑的，没有「补丁 → 项」的核对；提交时整轮才判 |
| 3 | 第 2 步的 `--cwd` 与根 | 照旧习惯写成主工作区 | 部分：新变异行在主工作区表里没有，变异门禁退 2；形式化与已有的崩溃用例在主工作区上照绿（判的不是补丁） |
| 4 | 第 2 步红了要不要重交第 1 步 | 判成「没改到第 1 步验过的文件」 | 不会（R8） |
| 5 | 第 3 步头一条核指纹 | 跳过、或拿的是带 `--item` 那一趟的（R5） | 不会：`apply-writer-patch.py` 往主工作区打不看合入后验证有没有绿、核没核过 |
| 6 | 留底 | 漏掉 | 不会：打补丁不要求先留底；漏了到回退时才发现退不了 |
| 7 | 回退 | 新建的文件漏删、别人的改动被冲掉（R11） | 不会：`check` 退 0 |
| 8 | 登记行转抄 | 段树与主工作区两份不一样（R10） | 晚：提交时的门禁；之间别的会话的门禁先红 |
| 9 | 第 1 步退 3 时自己拼合入命令（R3） | 没指纹、没留底 | 不会 |

- 有检查兜着的：段树建法写错（缺 `--with` 时第 2 步起不来，退非 0）、补丁撞文件（`git apply --numstat` 人核，撞了打进段树时 `apply-writer-patch.py` 退 2）、`insert-row.py` 锚点不唯一（退 3）。
- 四句：字面分句——二·2「哪几步做错了没有任何检查会红」；改法——1、5、6 三处可以由 `apply-writer-patch.py` 往主工作区打时现核（要求同一批补丁目录里有一份绿的合入后验证日志、指纹现核退 0、留底清单在），零轮。

## 三、命令模板的两条路

### R15 `-- env` 那一条路闸认、服务不拒、到得了测试进程；但闸认的变量与测试读的变量不是一回事（三·1）

- 闸：`.claude/hooks/lib_run_scope.py:703`：「    return any(subject.environment.get(name) for name in CRASH_SHRINK_VARIABLES)」，两个变量任一个设了就放（`.claude/hooks/lib_run_scope.py:165`：「CRASH_SHRINK_VARIABLES = ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO", "SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO")」）。闸自己的自证只有「命令前带缩域变量」的证红与变异门禁两种、「`-- env` 之后带」的崩溃点重放与 `cargo test` 两种（文件头弄坏开关那一段），变异门禁与证红经 `task-server/task submit -- env …` 交的那一形没有样本：探针第 2、3、4、6 条补这几形，期望都退 0（推的，待主 agent 跑）。
- 服务：`env <名>=<值>` 不在拒的前缀里，退役变量表只有两个、不含这两个（`task-server/task_server_lib.py` 的 `RETIRED_VARIABLES`、`REFUSED_PREFIXES`，读码）；分片的门禁每一步起的是同一条命令（`task-server/task_server_lib.py:505`：「① 计划（plan）：服务挑台起这条命令」），`env` 跟着走。
- 到测试进程：变异门禁的工作进程拿 `dict(os.environ)` 起 cargo（`scripts/mutation/crates-mutation-rows.py` 的 `init_worker` 一路，读码），`cargo test` 把环境交给测试二进制；推的，没跑。
- 不是一回事：拒绝说明给的出路点名的是判器那一个：`.claude/hooks/lib_run_scope.py:892`：「出路：带缩域变量（经服务交的写成 `-- env {CRASH_SHRINK_VARIABLES[1]}=<n> <命令…>`」，下标 1 是 `SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO`。登记的崩溃枚举用例经引擎读的是放量那一个：`crates/singlefs-verification/src/crash_amplification.rs:201`：「崩溃枚举用例取它与用例自己的展开里小的那个」（「它」是 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`）；`crates/singlefs-verification/tests/` 下写了判器那一个的只有 GPU 对拍那一份（`grep -rn`，量的）。被闸拒了的主 agent 照共用约束「照出路把命令或脚本改成合规的写法」，写成 `-- env SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=<n>`：过闸（探针第 3 条，期望 0），测试不读它，跑的是全域。实现员定义写对了（`.claude/agents/implementation-writer.md:34`：「`<缩域变量>` 写测试源码读的那一个，放量引擎读的是 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`」），与闸的出路两说。
- 全域理由那一条路：实现员第 3a 步的整条（`.claude/agents/implementation-writer.md:37`：「确要全域的在 submit 前面带一份四段声明」）照抄成探针第 5 条，期望 0（推的，待跑；第一轮那条变异门禁带全域理由的已复跑退 0，证红这一形没跑过）。
- 四句：判别子——闸在交的那一刻看得到点的目标是不是登记的崩溃枚举用例，也就分得出该是哪个变量；改法——闸按目标判变量（登记的用例与放量分项只认放量那一个），出路照目标点名，零轮。

### R16 缩过域的证红：定义没写；判定库按同一把键记，提交时全域那一趟照键复用（三·2）

- 定义给了缩域写法（`.claude/agents/implementation-writer.md:37`：「缩域写成 `task-server/task submit -- env <缩域变量>=<n> bash scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名…>`」；变异门禁那一条在第 34 行，`cargo test` 那一条在 `.claude/agent-common.md:68`：「缩域写成 `task-server/task submit [--cwd <根>] -- env <缩域变量>=<n> <命令…>`」），没写这样证出的红与提交时全域那一趟是什么关系。合入后验证那一边写了（缩过域的「不算门禁通过」），这三条没有。
- 今天的库怎么记（读码，量的 grep）：证红与变异门禁共用一把行键、一份判定：`scripts/mutation/prove-red.sh:25`：「两边共用同一把行键、同一份判定，不另起种类」；点名跑与整张跑同一判定：`.claude/gate.d/crates-mutation-replay.sh:30`：「（照常写判定库记录、照复用判据复用：点名单跑与整张跑时那一行的判定相同）」；行键里的环境只收编译相关的几个：`scripts/mutation/lib_mutation_units.py:95`：「BUILD_VARIABLES = frozenset(("RUSTFLAGS"」，两个缩域变量在 `lib_mutation_units.py`、`crates-mutation-rows.py`、`crates-mutation-replay.sh` 里 0 处；复用的条件：`scripts/mutation/crates-mutation-rows.py:204`：「  复用            库里有这把行键的记录，判的是抓到」。对照：按条复用的测试函数键带上了缩域变量（`scripts/gate/admission.py:733`：「    ("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO", "value"),       # 放量的展开深度」），变异这一族没有。
- 于是：缩域下判成「抓到」的那一行，提交时不指定范围的 crates-mutation-replay 照键复用，这一行在全域上一次都不判。多数情况「缩域里抓到 ⇒ 全域里也抓到」（缩过的域是全域的子集），复用不致假绿；例外是改坏的正是读缩域变量的那一段的行（缩域里才走到的分支），推的，没找实例。
- 反过来：缩过域时「全量域计数」那一类断言不判：`crates/singlefs-verification/src/crash_state_enumeration.rs:1302`：「：闭式不判；每个崩溃块照判」。新测试的判别力要是只在计数断言上（例：改坏枚举的展开，只差在态数），缩域证红必定「没红」，照「记红、撤绿」记进库，实现员被叫去修一条没毛病的测试。定义没提醒。
- 四句：字面分句——三·2「缩过域的证红算不算证红…定义写没写它与提交时全域那一趟的关系」；改法——定义写明「缩域证出的红照算证红，提交时那一行不再在全域上判」或把缩域变量进变异行键（缩域那一趟另存一份、提交时现判），二选一要用户定；另写「只靠全量计数断言判别的新测试不许缩域证红」；零轮。

### R17 判法给了，没说对着哪棵树；「测试源码读的那一个」多数目标答不出（三·3）

- 判法：第 1d 步列了三条（`stage-inputs.tsv` 的 `crash-case:` 行 `test=`、`ITEMS`、源码里写了变量名），与闸一致。闸对着哪棵树读：`.claude/hooks/run-scope-guard.sh:58`：「登记表所在的树：从命令的当前目录往上找带 .claude/gate.d/stage-inputs.tsv 的那一层」。定义没写实现员该对着哪棵树判。
- 新用例的那一行：实现员写命令时，副本是从段树建的、新用例的登记要等第 2 步主 agent 写进段树（`.claude/agents/implementation-writer.md:49`：「`.claude/gate.d/` 下的两张表与 `scripts/crash/crash-amplification.sh` 的 `ITEMS` 由主 agent 或 tooling-writer 登记」）。照三条判，它的新变异行点的目标「不走崩溃枚举」，命令不带缩域也不写全域理由；第 2 步主 agent 照原样 `--cwd <段树>` 交，这时段树里登记了，闸拒。探针第 7、8 条造的正是这两棵（同一条命令：没登记的期望 0，登记了的期望 2；推的，待跑）。同一条模板照抄、在写段过闸、在验证段被拒，拒在主 agent 手上；主 agent 照出路改，又落进 R15 那一处。
- 「写测试源码读的那一个」：量的，`stage-inputs.tsv` 的 108 条 `crash-case:` 行点到 56 个测试目标，测试源码里写了放量那个变量名的 10 个（多在文档注释里），只写判器那个的 0 个，两个都没写的 46 个（逐份读 `crates/singlefs-verification/tests/<目标>.rs` 数的）。这 46 个靠后半句「放量引擎读的是…」才答得出，要实现员自己知道「登记的用例经引擎」。
- 改法（零轮）：判法写明「对着第 2 步要交的那棵树（段树，登记行写进去之后）判；自己新写的崩溃枚举用例一律按会走算」；变量那一句改成「登记的崩溃枚举用例与放量分项写放量那一个，只有源码里写了判器那个变量名的目标写判器那一个」。

## 没打中的形状

- 段树建得成、补丁打得进段树、第二批从段树建副本：第一轮量过，这一轮没重做，不另判。
- 指纹核对那一条自己的写法：python 段与 `find … | sort -z | xargs -0 -r sha256sum` 两边的剪枝（`.git` 任何种类、`target`、`target-*`、`__pycache__` 目录）、符号链接（两边都不跟、不列）、排序（两边按字节）、带反斜杠与换行的名字逐项对过（读码），没打中。起点在核对时被删（包整个删掉）时 `find` 报错而 `diff` 照样退非 0，照「变了」算，对。
- 新建的包（补丁新加 `crates/<新包>/`）：起点往上找不到带 `[package]` 的 `Cargo.toml`、又不是此刻在的文件，不进起点；但新包要改工作区根 `Cargo.toml` 的 members，第 1 步退 3，归 R3，不另成一格。
- 服务拒不拒 `-- env <缩域变量>=<n>`：不拒（读码），三·1 这一半没打中。
- 拒算登记行落在段树、`refused-test-registry.py list <段树>` 读段树那一份：今天对得上，工具员报告第七节第 1 条那一处由主 agent 补的句子补上了；合入后验证不读 `refused-tests.tsv`（`merge-and-verify.sh`、`admission.py` 里 0 处，量的 grep），第 1 步不受它影响。
- 段树里第 2 步要的门禁脚本连同 import 的库拷不拷得全：`minimal-copy.py` 照闭包拷 `--with` 目录里脚本 source、起、import 的（它文件头第 20 行），没打中；没在段树里真起门禁。
- 取样范围：一·1 只走了「两批、三份补丁、一条新崩溃枚举用例、一条拒算」一种布局；用户决定的动作（段分几批、哪一批停下等用户、别的会话什么时候写主工作区）都放开写成「哪一种都会到这一步」，R6 的「很可能」依赖别的会话的写频率，没量窗口里的真实次数。

## 撤回第一轮我提的

- 第一轮 G1-J 的改法「许主 agent 把段树在 `.deliverables` 里声明成一个新种类」：撤回。理由见 R12——合完之后段树没有用处，合完就删；写段里跨轮那一种定义已写「停下就删」。
- 第一轮 G1-B 的改法「照留底把已合的几份退回」：收窄。用 `copy-originals.py` 的整批留底照 `check` 换回，在 `crates/mutations.tsv` 这类共写的表上会冲掉别人的追加（R11）；该照各份 `apply-outcome.tsv` 逐份、先判此刻是不是还等于合后再退。
- 第一轮 G1-C 的改法里「第 2 步改在第 1 步的快照上跑」那半句，这一轮在 R7 写成了具体做法，不撤；「第 2 步开跑前核段树与主工作区、不一致重建」那半句，比起来贵而窗口照旧在，降成备选。

## 这条腿自己的限度

- 全部是对着定义、脚本文件头与代码推的走法，没有真派实现员、没有交合入后验证、没有在段树里起门禁。R1 第 1 条「验证包那一步一起门禁就被 placement 判红」靠的是门禁第 1283–1293 行无条件起 placement、整棵树判，没造一棵快照跑出来。
- R5、R8、R12 引的是脚本的分支条件与文件头，没跑出「带 `--item` 那一趟打出合入命令」「第 2 步修补没进合入命令」「重打已合补丁退 2」的原样输出。
- R6 的「很可能退非 0」只量了五天里的总改动量与修改时刻，没量第 1、2 步那几个小时的窗口里的次数；`who-touched.py` 那一条交主 agent。
- R13 的手工步数是照定义逐条数的，推的；两种改法的代价没有在一段真实的实现上量过。
- 范围声明闸只用探针看退出码，探针还没跑；服务与变异门禁一路的环境传递是读码，没跑。

## 要主 agent 跑的命令

1. 范围声明闸探针（喂钩子，照主 agent 判；不起编译，不用 `task clean`），在仓根起：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/ledger-defs-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/ledger-defs-r2-snapshot /home/fy5090/.local/share/singlefs/work/ledger-defs-r2-opus-probe-run /home/fy5090/code/singlefs/research/prompts/ledger-defs-r2-opus-model
```

   跑完看：末行 `TASK_SERVER_RESULT` 的 `exit=0`（退非 0 的件数为 0 算过）；`task-server/task log <号>` 里 8 行 `name=probe sample=… expected=… exit=… verdict=…`、一行 `name=way_out sample=control-mutation-gate-no-shrink names=judge-variable`（R15 那一句的实测）、末行 `name=probe_summary samples=8 mismatches=0`。`verdict=differs` 的那一行是对应那一格推断不成立的地方，照它的 `first_stderr_line` 与运行目录里 `<样本名>.err` 判：第 3 条不是 0 → R15「照出路改过得了闸」不成立；第 5 条不是 0 → 实现员第 3a 步全域理由那一形照抄过不了闸（新的打中）；第 7、8 条不是 0、2 → R17 那一段不成立。运行目录 `/home/fy5090/.local/share/singlefs/work/ledger-defs-r2-opus-probe-run` 里有 `results.tsv`、两棵小树与每条样本的 `.out`、`.err`，核完删。

2. 量 R6 的写频率（主 agent 的会话工具，在仓根直接跑，不经服务，`.claude/main-agent.md` 第 38 行点名的那一条）：

```
python3 scripts/agents/who-touched.py crates/mutations.tsv crates/singlefs-verification --since-minutes 1440
```

   跑完看：两条路径各有几个会话的 Edit / Write 与 Bash 点到次数不为 0。一天里不止一个会话写过 `crates/mutations.tsv` 或验证包，R6「第 1、2 步那几个小时里核对多半退非 0」就站得住；都是 0 或只有本会话，R6 降成「可能」。

## 没做什么

- 不判正推、辩方那几格（第四组与第二组第 1 条的辩护归辩方腿）；不替主 agent 采纳。
- 没往统一任务服务交任务；探针没跑，报告里引探针的退出码都是「推的，待主 agent 跑」。直接跑过的只有只读命令与 `cargo metadata --no-deps --offline`（不编）。
- 没在副本上改代码、没建仓副本与编译目录；草稿目录 `/home/fy5090/.local/share/singlefs/work/ledger-defs-r2-opus` 里只有文本草稿（`metadata.json`、生成探针输入的 `make_inputs.py`、`rerun_steps.sh` 与 `model-build/` 一份模型目录的草稿拷贝）。
- `crates/` 下的产品代码没读；读了验证包的两处（缩域变量谁读、缩域下计数断言不判），只为判三·1、三·2 的变量到不到、断言判不判。
- 只判正文第一、二、三组点名的句子；四份里别的会话的改动没攻。没读禁读清单里的文件，也没读名字相近的 `d137-ledger-defs-r2-*`。
