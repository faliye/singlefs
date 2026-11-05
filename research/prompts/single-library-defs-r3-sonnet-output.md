# single-library-defs-r3 辩方腿报告（K1：前两轮判决落没落、改过头没有、三处处置对不对）

日期：2026-11-01。立场：辩方（Sonnet）。只读；不写模型，没有复跑脚本；判决要引的数没有一个是我量的（下面的计数都是 grep 数出来的，标「量的」，范围写在各处）。

## 开工核了什么

- 此刻四份被判文件的 sha256 与快照 `research/prompts/single-library-defs-r3-snapshot/defs-sha256.txt` 四行逐行相同（量的：`sha256sum` 对 `.claude/rules/verification.md`、`.claude/agent-common.md`、`.claude/main-agent.md`、`.claude/kb/checks-owed.md`）。所以下面引的「此刻的文字」就是快照那一版。
- 禁读清单四份没读。调度记录 `records/2026-11-01-判定库服务化.md` 在禁读里：判决说「交用户」的几条是否另有去向，我读不到，只能说「规则与 kb 与 research/prompts 里没有」。
- 用户原话只取正文第一节（`research/prompts/_single-library-defs-r3-body.md:16`）抄的那几句；没有别的原话来源，所以「用户没说」一律指「正文第一节那几句里没有」。

## 一、第二轮判决第二节每一格与第四节八条，对着此刻的文字逐格核（正文 K1 第一条）

判定列用「落了 / 没落 / 落偏」，末列给辩方判定（站得住 / 够不着 / 同样打中别的臂）。判决行号指 `research/prompts/single-library-defs-r2-main-verification.md`。

| 判决的格或第四节条 | 判决要的 | 此刻的原句（文件:行） | 落没落 | 辩方判定 |
|---|---|---|---|---|
| 第二节 K1.1 / 第四节 1：「判定」定义 | 改成「复用要读的一切」 | 「判定库的内容：按单元键或行键存下、给下一趟复用要读的一切」（`.claude/rules/verification.md:72`） | 落了 | 站得住 |
| K1.1 / 第四节 3：replay-row、flow-lines 进欠账 | 名单补全 | 「广谱变异的 TSV、E174 与 E178 装置自己的库、E182 的片结果库、replay.sh 的 replay-row、admission 的 flow-lines/、crates 变异的 target-occupancy.tsv 与 timings/、变异复跑给每片建的崩溃判定库」（`.claude/rules/verification.md:74`）；C948 前置同列一遍（`.claude/kb/checks-owed.md:605`） | 落了 | 站得住 |
| K1.1：qemu.env 进 C949 前置 | 第一轮判决写了去向，第二轮判决 K1.1 行写「采」（`research/prompts/single-library-defs-r2-main-verification.md:15`，「qemu.env 没进 C949 前置」） | `.claude/kb/checks-owed.md` 里 `qemu.env` 0 处命中（量的：`grep -cF`）；第二轮第四节八条里没有一条带它；`scripts/devices/vm-bench.sh:280` 仍读仓根 `qemu.env`，`.gitignore:29` 把它列为不进仓 | 没落（第一轮判决 `research/prompts/single-library-defs-r1-main-verification.md:22` 说记、第二轮判决说采，两次都没进 kb） | 够不着：判决写了去向，仓里没有；第二轮「采」那一格的去向列「第四节 1、3、5、6」里没有一条是它 |
| K1.1 / 第四节 5：自证空库只放行临时服务那一半 | 并不进去的报红；自证与压测对临时服务 | 「并不进去的，由开它的那一趟报红、不留着」「自证与压测一律对临时服务起的库，格完即删，不写真服务」（均 `.claude/rules/verification.md:76`） | 落了，但只落了判决写的那一半 | 够不着 Rust 一次性库那一半：`crates/` 下 16 份文件用 `DisposableLibraryDirectory`（量的：`grep -rlF` 对 `crates`），`crates/singlefs-checker-tier/src/crash_library.rs:12` 把它列为「除外」；它们不是临时服务的库，规则第 76 行字面算「另开一份 RocksDB」，欠账名单与 C948 前置都没点它。同样打中别的臂：不管怎么写，这 16 份要么进名单、要么改成对临时服务 |
| K1.1：「列全四张表」落偏 | 判决 K1.1 行说采；第四节 7 说不抄表名 | 「登记进 `task-server/task_server.py` 文件头点名的那几张只派一台的表并在文件头写明为什么」（`.claude/rules/verification.md:84`），句子里没有表名 | 落了（取第四节 7 的写法；第二节「列全」那半句作废） | 站得住：不抄表名正好躲开 `task-server/task_server.py:223`、`:226`、`:243`、`:245`、`:247` 这些活文件常量会变的问题；判决第二节和第四节两处说法不一致，以第四节为准 |
| K1.1：17 处的数 | 「改 9、顺带 2、留 4、漏 2」 | 第四节 3 把 `:37`、`:40`、`:42`、`:144`、`commit-stages.md:45`、登记表第 25 行列为留着（`research/prompts/single-library-defs-r2-main-verification.md:34`）：6 处 | 判决内自洽（9+2+6=17，推的，没重数） | 站得住 |
| K1.1 / 第四节 6：`verification.md:39` 的键前缀 | 本轮不动文字，记进 C948 前置 | 「各道的键与它在同一份判定库里的键前缀写在它的文件头」仍在（`.claude/rules/verification.md:39`）；C948 前置「每道文件头写它在那一份库里的键前缀」（`.claude/kb/checks-owed.md:605`） | 落了（按判决写的「不动」） | 站得住但前后不一：文件头里没有「键前缀」（量的：`grep -lF 键前缀` 对 `.claude/gate.d/*.sh`、`scripts/crash/*.sh`、`scripts/mutation/*.py`、`scripts/gate/verus-proof-check.py`，0 份），所以这句此刻是假话；判决对「布局只有一种」同一形状却收为「同类现状句没清」（第二节 K1.2 行，`research/prompts/single-library-defs-r2-main-verification.md:16`）。同一形状两种处置 |
| K1.2 / 第四节 2：删「迁完之前旧库只读不写」 | 删 | `verification.md` 里「只读不写」0 处（量的：`grep -cF`） | 落了 | 站得住 |
| K1.2 / 第四节 2、4：「布局只有一种」 | 判决 K1.2 行写「采 第四节 2、4」 | 「布局只有一种：全部表在默认列族、键原样、开库选项照 `crates/singlefs-checker-tier/src/verdict_store.rs`」仍是现在时（`.claude/rules/verification.md:72`）；正文第二节自己写「本机常驻的还是上一版（多列族布局）服务」（`research/prompts/_single-library-defs-r3-body.md:22`） | 没落（第四节 2、4 改的是接完三件与迁移句，没有一项碰这句） | 够不着：判决采了、文字没动 |
| K1.2：「全仓只有一份」而两处写文件 | 名单补全 | 同第二行 | 落了 | 站得住 |
| K1.2：「不并存」与过渡期 | 判决没提 | 「旧库不原地打开、不并存」（`.claude/rules/verification.md:75`）与「没带的走旧库（过渡）」（`.claude/rules/verification.md:77`）同节并存 | 无去向 | 够不着；小：「不并存」限定成「不并存写入」或挂「接完之后」即可 |
| K1.3 / 第四节 3：`:37`、`:42` 前半、`:40`、`:144`、`commit-stages.md:45`、登记表第 25 行「按这一条留着」 | 归「还没接完的族接完之前照旧」 | `:37`「崩溃放量、崩溃枚举用例、给定状态表与两种对拍共用一份崩溃判定库」、`:40`「这些的判定库放在库根那棵仓的 git common-dir 下（广谱变异的库目录除外）」、`:42`「崩溃判定库的键与库目录都不读仓根：库目录由家目录定」（均 `.claude/rules/verification.md`）都没改，都没挂过渡字样 | 落了（按「不动」） | 站得住，理由要改引：崩溃判定库和 Python 三家不在第 74 行名单里，罩住它们的是第 73 行「接完之前这一族的任务写旧库是过渡、不算违例」。判决写「第四节 3 把那张名单列全」（`research/prompts/single-library-defs-r2-main-verification.md:17`）指的名单，装不下这两族 |
| K1.4：「核心策略，没有例外」与例外子句并存 | 保留，例外子句改成「不算测试的活」的归类，不是例外 | 句子没动：「（核心策略，没有例外）」（`.claude/agent-common.md:39`、`.claude/main-agent.md:12`），「（核心策略，没有例外；」（`.claude/agent-common.md:40`） | 按判决「保留」就是不动，落了 | 见第三节 3.3：站得住的条件是「欠账」两个字也写上；两个子 agent 看得到的条没写 |
| K1.4：`.claude/agent-common.md:52` 与迁移命令冲突 | 「判决写明」（`research/prompts/single-library-defs-r2-main-verification.md:18`）：那一条管产品格式，判定库迁移不是兼容旧格式 | `.claude/agent-common.md:52` 仍写「迁移脚本与补旧行的子命令」「旧库、旧镜像、旧产物不迁移」；`verification.md:75` 要求「经迁移命令逐行拷进服务的库」 | 没落：只在判决里「写明」，规则文字没改；子 agent 不读判决 | 够不着：两句同在子 agent 的必读文件与规则里，字面冲突原样在。判决的道理对（迁移是搬数据再删旧库，不是认旧格式），但要写进 `.claude/agent-common.md:52` 才算数 |
| K1.4：「三家」未定义 | 「在新节第 1 条点名键空间文件」 | 「各族各自的键前缀（Python 一侧 `scripts/prune-server/prune_server_client.py` 文件头「键空间」，Rust 一侧 `verdict_store.rs` 的表标签）」（`.claude/rules/verification.md:72`）；第 75 行仍写「三家的文件库」 | 落了一半 | 站得住：键空间文件头第 4 行写「Python 写的三家判定库」（`scripts/prune-server/prune_server_client.py:4`），第 19 行起列族名；第 75 行的「三家」读者要跳两步，加「Python」二字即清 |
| K1.4：读也走服务「已在条款里（「读写」）」 | 判决称条款里有「读写」 | 「读写」在 `verification.md` 只出现在第 13 行（checker 档那一格的描述，与判定库无关）、在 `agent-common.md` 只出现在第 62 行（也无关）；判定库一节说的是「要存判定经 prune-server 写进那一份库」（`.claude/rules/verification.md:83`）与「判定一律经 prune-server 的客户端写进那一份」（`.claude/agent-common.md:39`） | 判决引的条款里没有这个词 | 够不着：判决的依据句在规则里查无；读也走服务靠「全仓只有一份」间接推，没有一句直说 |
| K2.1 | 定义改宽、名单补全 | 同前两行 | 落了 | 站得住 |
| K2.2：中转库后门 | 并不进去报红；自证压测对临时服务；变异每片与 E182 进名单；`CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 非 0 判红记进 C948 前置 | `.claude/rules/verification.md:76`、`:74`；「崩溃入口与门禁数 `CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED` 非 0 判红」（`.claude/kb/checks-owed.md:605`）；该行前缀常量真在代码里：`pub const LIBRARIES_LEFT_UNMERGED_LINE_PREFIX: &str = "CRASH_LIBRARY_LIBRARIES_LEFT_UNMERGED";`（`crates/singlefs-checker-tier/src/crash_library.rs:107`） | 落了（Rust 一次性库那半见上） | 站得住 |
| K2.3：多机空转 | 第二台全部现判、结局逐项相同；没派到的标「第二台未跑」收口前补跑；许读全路径；指到 task_server.py 文件头；「挑台」交用户 | 「这一趟不读判定库、全部现判，现判的结局与本机那一趟逐项相同才算写完」「这一条测试在收口表里标「第二台未跑」、收口前补跑」（均 `.claude/rules/verification.md:85`）；三条许读路径（`.claude/rules/verification.md:83`）；`.claude/main-agent.md:13` 同写；`.claude/agent-common.md:40` 只写「没派到第二台的在报告里写「第二台那一趟本次未跑」」（子 agent 一侧，不写收口表是对的） | 落了 | 站得住；「挑台交用户」见第三节 3.1 |
| K2.4：过渡期开关与接完三件 | 三件登记在登记表第三列；`--verify` 只判缺与异；接完之后再写旧库或另建本地库算违例 | 「一族「接完」= 三件都做到」「接完之后再写旧库、或不带开关时悄悄另建一份本地库，都算违例」（均 `.claude/rules/verification.md:73`）；「某一族接完之后这个开关对它不再有意义」（`.claude/rules/verification.md:77`） | 规则落了；登记表这头：第三列没有一行写 prune-server（量的：`grep -c prune-server`，0），表头注释只定义了「库在哪 / 无判定库 / 不算」三种写法（`.claude/gate.d/verdict-library-scripts.tsv:3`） | 站得住（此刻没有一族接完，0 行合理）；小：表头没写「接完」这一格怎么写，立 C948 那一格前要补 |
| K3 C948 | 判决「采」攻方四点 | 前置补了 flow-lines、target-occupancy、replay-row、E182、变异每片库、文件头键前缀、`LEFT_UNMERGED`；判法补了「一个目录存判定文件」与「旧库删后目录被重建」（`.claude/kb/checks-owed.md:605`；`.claude/kb/checks-owed.en.md`、`.claude/kb/checks-owed.ja.md` 的第 605 行同有 `LEFT_UNMERGED`、`flow-lines`、`target-occupancy`、`replay-row`、`E182`，量的：`grep -oE`） | 落了两点，两点没落 | 够不着：①「第三列 18 行全是本地路径，「过渡」与「中转库按文件头登记」没写法」——怎么拦那句还是「接进之前登记为过渡的行报本次未跑」「中转库（并回即删）与自证临时服务的库按文件头登记放行」，登记表里没有「过渡」二字（量的：`grep -n 过渡` 对登记表，0 行；库行共 18 行，量的：`awk` 数第三列非「不算」「无判定库」开头的行）；②「两份样本分不出两种实现」——样本那句一字没动（`.claude/kb/checks-owed.md:605`「判别力两份样本：…」）。判决第四节 8 只列了前置与判法，这两点连同「采」一起丢了 |
| K3 C949 | 动态格改成第二台全部现判；静态格补 `crates/*/src`、`var_os("HOME")`、存量重数 | 「第二台那一趟不读判定库、全部现判」「不靠服务记台（服务按键只存值，复用与逐字节跳过都不发写）」，静态词表含 `crates/*/src/`、`var_os("HOME")`（均 `.claude/kb/checks-owed.md:606`） | 落了；存量重数无结果（文字里没有数，滞后表这一格还没建） | 站得住；「存量重数」是动作不是文字，立格时做 |
| 第四节 7：`agent-common.md`、`main-agent.md` 同步 | 同步 | `.claude/agent-common.md:40`、`.claude/main-agent.md:13` 与规则第 83–85 行同口径 | 落了 | 站得住 |

小结（量的：上表 23 个数据行，每行一项）：没落 5 行（qemu.env、「布局只有一种」、`.claude/agent-common.md:52`、「读写」依据句、「不并存」无去向）；只落一半 3 行（Rust 一次性库、「三家」、C948 的「过渡」与样本两点）；其余 15 行落了。

### 一·结论与没核动的

结论：第二轮判决第四节八条里，规则与 C948、C949 的主体都落了；有五处判决说「采」或「写明」而文字里没有（qemu.env、「布局只有一种」、`.claude/agent-common.md:52`、「读写」依据句、「不并存」），三处只落一半（Rust 一次性库、「三家」、C948 的「过渡」与样本）。其中 qemu.env 是同一条去向被两轮判决各承诺一次、仓里一次都没出现。
依据：上表每行的文件:行。
什么现象会推翻：上表任一行的原句在别处（我没扫的文件）已有等价处置，则那一行改成「落了」。
没核动的：`.claude/kb/checks-owed.en.md`、`.claude/kb/checks-owed.ja.md` 只核了几个关键词在不在（第 605、606 行），没逐句对三语；第二轮辩方腿核过的 17 处里，我只重读了 `verification.md` 的 `:37`、`:40`、`:42` 与第 39 行，别的（`commit-stages.md:45` 等）没重读。

## 二、有没有改过头：六处条款与 C948、C949 里，用户没说的被写成核心策略的（正文 K1 第二条）

用户原话只取 `research/prompts/_single-library-defs-r3-body.md:16` 抄的：「以后不允许各自测试简历自己库。 这是核心的策略，不允许有其他的」「所有测试都必须多机可用，不得依赖一台机器」「不加任何参数 删掉布局选择 只能用 rocksdb」「不得原地打开。 迁移完成后 数据无误，删掉原来的数据库」「把库合并到一起 用rustdb」，task-server 监控会话转的「所有的脚本必须多机跑。抛出的结果需要多机都能读取」「所有的服务必须要不受限制的多机跑」；第一轮判决另抄了一句「测试这里所有的只要功能满足都要走 rocksdb」（`research/prompts/single-library-defs-r1-main-verification.md:29`）。

结论：没有哪一处是把用户的话反着写；但有六处是主 agent 自己加的东西，和用户原话挂在同一个「核心策略，没有例外」括注下面，读者分不出哪句是用户的、哪句是推的。

| 句子（文件:行） | 用户原话里的依据 | 判 |
|---|---|---|
| 「**不新建判定库**（核心策略，没有例外）」（`.claude/agent-common.md:39`）、「**判定库只有一份，不许各自建库**（核心策略，没有例外）」（`.claude/main-agent.md:12`） | 「这是核心的策略，不允许有其他的」 | 站得住：「核心策略」「没有例外」两个词都是用户的 |
| 「**所有测试都要多机可用，不许依赖某一台**（核心策略，没有例外；」（`.claude/agent-common.md:40`、`.claude/main-agent.md:13` 同） | 「所有测试都必须多机可用，不得依赖一台机器」——这句里没有「核心」二字；「核心」只跟判定库那句 | 加的：括注把第二句也升成核心策略。HEAD 上这两条都不存在（量的：`git show HEAD:.claude/agent-common.md`、`git show HEAD:.claude/main-agent.md` 各 `grep -n 核心策略`，0 行），是这一批新加的。用户若在正文没抄的地方说过「两条都是核心」，把原话补进背景材料；没有就去掉括注或只留「没有例外」 |
| 「判定库的内容：按单元键或行键存下、给下一趟复用要读的一切——结论（绿、红、拒算）与它的组成清单、判定向量、录入块、事实表、清单、覆盖、用时、流行、片结果都算」（`.claude/rules/verification.md:72`） | 「把库合并到一起」「各自测试简历自己库」：说的是「库」，没说用时、流行、覆盖也在内 | 推的：从「合并」推出「复用要读的一切」。把 timings/、target-occupancy.tsv 这些记成欠账、也受「没有例外」罩着，是这句定义带来的。该交用户确认一次，不该只在核心策略括注下默认 |
| 「自证与压测一律对临时服务起的库，格完即删，不写真服务」（`.claude/rules/verification.md:76`）；「自证与压测不写真服务」（`.claude/agent-common.md:39`） | 无。能撑它的是「只要功能满足都要走 rocksdb」里的「只要功能满足」（自证要空库，功能上走不了真库） | 加的；有依据撑一半：中转库「并回」就是用户要的合并过程，站得住；自证库那条按字面「各自测试建立自己库」就是例外，只是不开例外则服务自己的自证无处可跑。同样打中别的臂：不留这条，自证只能写真服务，更违「不写真库」 |
| 「交带判定库的任务带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`」（`.claude/agent-common.md` 无；`.claude/main-agent.md:12`，在核心策略那条里） | 用户说「不加任何参数」，指的是布局选择；这个开关是过渡期的参数 | 规则一侧挂了过渡：「没带的走旧库（过渡）」「某一族接完之后这个开关对它不再有意义」（`.claude/rules/verification.md:77`）；主 agent 定义这句没挂，读起来是永久要求。改：句首加「接完之前」 |
| 「这一趟不读判定库、全部现判，现判的结局与本机那一趟逐项相同才算写完」（`.claude/rules/verification.md:85`；同口径在 `.claude/agent-common.md:40`、`.claude/main-agent.md:13`） | 无。用户说的是「都必须多机可用」，没说每条新测试要在第二台完整现判一趟 | 加的：是怎么证「多机可用」的办法。成本（每条新测试多一趟不复用的跑）没人量，这是下一格攻方的事（正文 K2）；这里只记它不是原话。放在核心策略括注下面，读者会当用户定的 |
| 「只影响排程的本机状态许读、按台各一份，就这几处：`~/.local/share/singlefs/memory-peaks.tsv`、`~/.local/state/singlefs/run-durations.tsv`、`~/.local/share/singlefs/last-runs/`」（`.claude/rules/verification.md:83`） | 「不得依赖一台机器」 | 站得住：按台各一份的文件，不是依赖某一台；但它是「没有例外」里的一块窄化，且 `.claude/agent-common.md:40` 的概括句只写「不读会改变判定结局的本机独有状态」，没写这三处 |
| C949 静态格：`~/`、`$HOME`、`var_os("HOME")`、`expanduser` 判红（`.claude/kb/checks-owed.md:606`） | 同上 | 过头：比规则宽。规则第 83 行许读上面三处，静态词表没把它们放行；`scripts/`、`.claude/gate.d/*.sh`、`crates/*/src` 里含这些词的文件 74 份，这 74 份里提到这三处的 14 份、提到工作根路径 `~/.local/share/singlefs/work` 的 23 份（量的：`grep -rlE` 对这三个根，范围只是这三个根，文件名清单在草稿目录 `set74.txt`）。靠「存量登进滞后表」兜住，但滞后表会一开始就装进 74 份里的合规者 |
| C948 射程「开 RocksDB、写 TSV / sqlite 存判定的调用不经 prune-server 客户端判红」连 `crates/`（`.claude/kb/checks-owed.md:605`） | 「不允许有其他的」 | 站得住（用户的话就是这个宽度）；`DisposableLibraryDirectory` 的 16 份文件（量的）会一上来判红，要进欠账名单，见第一节 |

漏的（少写，不算过头）：「抛出的结果需要多机都能读取」「所有的服务必须要不受限制的多机跑」两句，在 `verification.md`、`agent-common.md`、`main-agent.md`、`checks-owed.md` 四份里 `都能读取`、`不受限制` 各 0 处（量的：`grep -cF`）；结果两台都读得到的做法只在 `.claude/agent-common.md:62` 的「子 agent 不交任务」那一条里有，不在多机那一节。是否要写进这六处，由主 agent 判；task-server 一侧已把这句话写进注释（`task-server/task_server.py:220`）。

什么现象会推翻第二行：用户在正文没抄的地方明说两条都是核心策略。推翻第三行：用户说过「用时、流行、覆盖也要进同一份库」。
没核动的：调度记录里用户的更长原话我读不到（禁读），上面「用户没说」都以正文第一节那几句为界。

## 三、三处处置对不对（正文 K1 第三条）

### 3.1 第二轮判决替用户不定的那一条（提交不挑台）

判决原文：「攻方 K2.3「要让一条任务必须在第二台跑一趟得改『提交不挑台』那条」要用户定，判决不替用户定：条款只写「服务派到第二台那一趟」，没派到的标「第二台未跑」等补跑」（`research/prompts/single-library-defs-r2-main-verification.md:28`）。

结论：站得住，带一个缺口。
- 不替用户定是对的：用户的话是「都必须多机可用」，没有「每条任务必须在第二台实跑一趟」；后者是怎么证前者的办法，要不要让调用方挑台是产品决策。
- 条款没要求调用方挑台，和共用约束一致：调用方不写 `--peer`、`--local-only`（`.claude/agent-common.md:62`）。
- 缺口：条款写「收口前补跑」（`.claude/rules/verification.md:85`、`.claude/main-agent.md:13`），而调用方没有办法让一条任务去第二台；服务里能钉台的只有登在只派第二台那张表里的入口（`task-server/task_server.py:2202`「钉了的只那一台」），而规则第 84 行又说新写的测试一律不登。正文第二节还写「崩溃三道不放」（`research/prompts/_single-library-defs-r3-body.md:25`）。结果是这几类测试的「补跑」没有人能做到，只能永远停在「第二台未跑」。条款没说这一类怎么收口。
- 三方规则要求交用户的岔路另写 `<轮>-forks.md`（`.claude/rules/three-way-inference.md@交岔路时写岔路单，派实验时带上它`）；`research/prompts/` 下没有这一题任一轮的岔路单（量的：`ls research/prompts | grep 'single-library-defs.*forks'`，0 行）。判决说「交用户」的那几条是否落在别处，我读不到调度记录，只能说这里没有。
- 同样打中别的臂：改成「调用方可挑台」要用户先定；改成「去掉第二台那一趟」弱于用户原话；现在这条是三条里被打得最少的。
什么现象会推翻：服务对没派到第二台的任务有自动换台重试（我没读 `task-server/task_server_lib.py` 的挑机器规则），则缺口不成立。

### 3.2 第一轮判决翻掉攻方两条

判决原文：「攻方 1.7「现在时写的是目标态」与「本机服务开的不是家目录那一份」两条因此不再成立」（`research/prompts/single-library-defs-r1-main-verification.md:29`），依据是用户后续的「不得原地打开……删掉原来的数据库」「把库合并到一起 用rustdb」。

- 「本机服务开的不是家目录那一份」：站得住。改后规则不再把家目录那份定成「那一份」：「全仓只有一份判定库：prune-server 自己的那份 RocksDB」（`.claude/rules/verification.md:72`）、「旧库不原地打开、不并存」（`.claude/rules/verification.md:75`）；服务一侧同向：`prune-server/README.md:25`「服务不原地打开别的库，只开自己那一份」。余留的同形问题换了样子：正文第二节写在跑的还是多列族布局的上一版服务（`research/prompts/_single-library-defs-r3-body.md:22`），而规则第 72 行说「布局只有一种」——规则和在跑的服务此刻仍对不上，要等换二进制。这条不是原攻点，是换了形的同一类。
- 「现在时写的是目标态」：翻得不干净。用户的话决定了目标是什么，没决定句子的时态对不对。第二轮辩方已指出同类现状句没清，第二轮判决「采」（`research/prompts/single-library-defs-r2-main-verification.md:16`）；清掉了「只读不写」一句（第一节第 9 行），「布局只有一种」没清（第一节第 10 行）。另有一处第二轮没点到的：「判定库只有一份，不许各自建库」那条把轻型、重型、……、广谱变异、实验装置的判定「全部写进 prune-server 自己的那份 RocksDB」（`.claude/main-agent.md:12`），而同一份规则的第 74 行点名广谱变异和 E174、E178 还没接。规则句是命令式，站得住；但这一句没挂「接完之前」。
- 什么现象会推翻：用户明说「规则写目标态就行，不要时态限定」，则第二条的余留项可以不改。

### 3.3 「没有例外」保留，例外子句归成「不算测试的活」

判决原文：「保留，例外子句改成「不算测试的活」的归类，不是例外」（`research/prompts/single-library-defs-r2-main-verification.md:18`）。

结论：保留「没有例外」站得住（判定库那条是用户的原话「不允许有其他的」，多机那条的「所有……不得」同样是全称）；归类这一步站得住，但只有带上「欠账」才站得住，两处子 agent 与主 agent 看得到的句子没带。
- 多机那条的归类有用户自己一侧的撑腰：task-server 一侧把只在本机的名单写成「用户：「所有的脚本必须多机跑」：每一条都是欠账，注释写着去掉它的路」（`task-server/task_server.py:220`）。规则第 84 行也写「表里每一条都是欠账、带去掉它的路」（`.claude/rules/verification.md:84`）。归类 + 欠账 + 去掉它的路三件齐了才不是例外。
- 缺：`.claude/agent-common.md:40` 只写「只能在一台上跑的活照那一节登表，不当测试」，`.claude/main-agent.md:13` 只写「只能在一台上跑的活登进 task-server 的那几张表（那一节点名的），不当测试」；「欠账」两个字不在。只读这两份的人读到的是「登表就不算」，正是例外。
- 判定库那条的几个「不算」不全是归类：中转库「不算另开」（并回是用户要的合并，归类成立）；自证库「对临时服务起的自证库不算」（见第二节第四行，是例外，只是必要的例外）；「一族接完之前照旧写旧库是过渡」（`.claude/agent-common.md:39`）是有期限的许可，不是归类。三样和「没有例外」同在一句括注下面。「过渡」不是错，错在括注没写期限：写「没有永久例外；过渡到各族接完为止」更准。
- 同样打中别的臂：去掉「没有例外」违反用户原话；把例外子句写成「例外」则三条里「没有例外」的字面最差。归类 + 欠账是被打得最少的。
- 什么现象会推翻：用户明说自证库也不许另开（则 `.claude/rules/verification.md:76` 与 `.claude/agent-common.md:39` 的「不算」要删，服务自证改成别的办法）。

## 四、交主 agent 的

该补的（判决说了、文字里没有；一项一行）：
1. `.claude/kb/checks-owed.md`（三语）C949 前置补 `qemu.env`（`scripts/devices/vm-bench.sh:280` 读仓根这份、`.gitignore:29` 不让它进仓）：两轮判决各承诺一次，都没落。
2. `.claude/agent-common.md:52` 加一句「判定库的迁移（`.claude/rules/verification.md`「判定库只有一份」）不在此列」，判决「写明」的内容要在规则文字里才算数。
3. Rust 一次性库（`crates/` 下 16 份文件用 `DisposableLibraryDirectory`，量的）：进 `.claude/rules/verification.md:74` 名单或 C948 前置，或改成对临时服务；现在规则第 76 行字面把它们判为违例而没有任何欠账行。
4. C948「怎么拦」：补「过渡」行在登记表里怎么写（表头 `.claude/gate.d/verdict-library-scripts.tsv:3` 只有三种格），补「接完」格怎么写；判别力样本一句补到能分出两种实现；判决第四节 8 补这两项。
5. 判决第二节 K1.4 那一行去向列写「第四节 7」，而第四节 7 讲的是多机一节，K1.4 的四件事没有对应的第四节条；补一条第四节 9。
6. `research/prompts/single-library-defs-r2-forks.md`（以及第一轮的）：三方规则要求交用户的岔路另写一份，目前没有。

该改的字句：
7. `.claude/agent-common.md:40` 与 `.claude/main-agent.md:13`：「不当测试」前加「（每条是欠账、带去掉它的路）」。
8. `.claude/main-agent.md:12`：「交带判定库的任务带 `SINGLEFS_VERDICT_LIBRARY_SERVICE=prune-server`」句首加「接完之前」。
9. `.claude/agent-common.md:39`、`.claude/main-agent.md:12`、`:13`、`.claude/agent-common.md:40` 的括注：多机两条的「核心策略」若用户没说过，去掉；判定库两条的「没有例外」改成「没有永久例外；过渡到各族接完为止」。
10. `.claude/rules/verification.md:72`「布局只有一种」改成目标式或挂「服务换上新二进制之后」；`:75`「不并存」改「不并存写入」或挂「接完之后」；`:75`「三家的文件库」改「Python 三家的文件库」。
11. `.claude/rules/verification.md:39`「各道的键与它在同一份判定库里的键前缀写在它的文件头」：文件头里没有；改成「各道的库目录名写在它的文件头」，或在句末挂「（接各家时补，C948 前置）」，与「布局只有一种」同一处置。
12. `.claude/rules/verification.md:83`「要存判定经 prune-server 写进那一份库」补「读也经它」；判决 K1.4 里那句「已在条款里（「读写」）」改成「靠『全仓只有一份』推」。
13. `.claude/rules/verification.md:85`「收口前补跑」改「收口前补跑；补不到的留在 C949」。
14. C949 静态格词表放行第 83 行三处许读路径，不然合规的 14 份（量的）一上来就判红。
15. 交用户表补三项（标「零轮」）：「判定库的内容」宽定义（含用时、流行、覆盖）、自证库对临时服务这一条例外、每条新测试在第二台完整现判一趟。

## 要主 agent 跑的命令

没有。我没写模型、没有复跑脚本、没有装置，也没有要经服务交的命令。

## 没做什么

- 不替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务、判决要引的数没复跑（没有要跑的命令）。
- 没读禁读清单四份；调度记录里用户的更长原话、「交用户」的几条去向、第二台实际派不派，都核不动。
- 没起任何服务，没查在跑的 prune-server 是什么布局（用正文第二节的观测，记「推的」）。
- 没碰 K2、K3 的攻点（接完三件的次序、第二台全部现判的代价、C948 前置够不够立格、C949 怎么拿到第二台那一趟）；第二节第七、八行只当「写进核心策略括注的是不是原话」记，没判它们立不立得成。
- 没扫第二轮辩方列过的 17 处里除 `verification.md` 之外的几处。
- 草稿目录里只有一份 `set74.txt`（文件名清单），没有仓副本、没有编译目录。
