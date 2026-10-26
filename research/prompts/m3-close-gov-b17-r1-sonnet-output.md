# m3-close-gov-b17-r1 正推腿（Sonnet）报告

立场：从已定条款（D13 已定项 17、`.claude/main-agent.md` 第 11 条、驱动与钩子源码）推「应该是什么」，再与被判的两份 diff 逐格比。
所有行号是我在写报告时现查的；引文用 `grep -nF` 对过。引 kb 写 kb 文件自己的行号。

## 0. 复跑与输入

模型目录 `research/prompts/m3-close-gov-b17-r1-sonnet-model/`（只读判法函数与钩子，不起 cargo，不写仓）。复跑命令：

    SINGLEFS_RUN_SCOPE='种类=items; 跑什么=三方腿模型 tools/rerun.sh（只读判法函数与钩子喂命令）; 范围=1 个脚本; 为什么=核模型输出可复跑、与 out/ 逐字相同' bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/m3-close-gov-b17-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen research/prompts/m3-close-gov-b17-r1-snapshot --work <新的草稿目录>

（本轮没有冻结副本，`--frozen` 只为过模板的参数检查，指到快照目录；脚本一律读 `--repo` 的主工作区。）
末尾应打三行 `name=rerun_compare file=… result=identical`。我跑的最后一趟原样输出：

    name=rerun_compare file=feed.txt result=identical sha256=0c7434e4acb19d0a50684578b62a0998e994a09c3889a1ac20bcee01303bad9e
    name=rerun_compare file=hook_feed.txt result=identical sha256=2a7cfec3ab347b75c6a631e9c44a0b0cd25161926c7fa93c847b88f4a06fe4f6
    name=rerun_compare file=probe.txt result=identical sha256=aad526014861920ea5435be03bd55e2e12c59e2247421aee07931693b754debc

模型文件 sha256（写报告时现算）：见文末「附：sha256」。
一次不稳：第一次复跑（刚改完 hook_feed.py、重生成 out/ 之后）比出来 hook_feed 的「diff=」计数比 out/ 多 1，当时的差异行我没留全；之后同一份脚本复跑 4 趟（经 rerun.sh 3 趟、直接跑 1 趟）与 out/ 逐字相同。原因没查到，按「未稳」记，不当成已排除。

### 0.1 输入有没有被别的会话改

快照 `judged-sha256.txt` 的十行与 `kb-sha256.txt` 的三行，我开工时逐行现算，全部相同。写报告时（同一天稍后）：

- `.claude/hooks/lib_heavy_tests.py` 已被别的会话改（它加的是「here-string 与管道喂给从标准输入读命令的 shell」那几格，与 `replay.sh` 无关）：sha256 先后见到 `51b96bbb…`（快照）、`54101127…`、`b1c3ef8d…`。三版上 `feed.py` 的 46 行判结果逐行相同（我把 `51b96bbb…` 版的输出存了下来与现版 `diff`，只差 sha 那一行；`feed.py` 现在把 sha 打到 stderr，不进比对）。
- `.claude/hooks/completion-cleanup-guard.sh` 也被改过（开工时 `09143da1…`，现 `83ac5f26…`），行号随之漂了十行，下面按现版现查。
- `.claude/hooks/heavy-test-guard.sh` 也变了（sha256 `ed9e3102…`，快照是 `dcb0da8c…`）；`hook_feed.py` 19 格在改后版上与改前版上结果相同。
- 被判 diff 里写的路径是 `scripts/experiments/replay.sh`；今天登记的目录改名（`.claude/gate.d/path-moves.tsv:6`：「scripts/experiments」→「scripts/experiment-apparatus」）之后，快照里那两份已是新路径，`scripts/experiments/` 目录已不在。下面一律按新路径判；diff 文字里的旧路径当冻结证据，不改。
- `scripts/crash/lib_verdict_library.py`、`scripts/gate/lib_function_units.py`、`scripts/mutation/broad-mutation.py`、`scripts/hosts/run-with-memory-cap.sh`、三份被判规则、`commit-stages.md`：写报告时 sha 仍与快照相同。

## 1. 格一览（先给结论，依据在后面各节）

| 格 | 判定 | 量的还是推的 |
|---|---|---|
| G1 复述对不对 | 骨架照定案，有 5 处出入（2 处多、3 处少），详见第 2 节 | 逐句比（量的：我读了两边原文）；「多」的第 1 处是否真有单元漏进键是推的 |
| G2 复跑副本保留 | 与第 11 条字面矛盾；与收工闸的关系取决于副本里有没有仓副本/编译目录：有则收工闸拦一次再放，无则不拦 | 收工闸的行为：读源码 + 它的 `--selftest` 量过（40 格绿）；「标准复跑副本里会有仓副本」是推的 |
| G3 GPU 那一趟 | 24 项里 20 项相同，4 处出入（1 处多、3 处少） | 量的（读源码逐项落行）；没起驱动 |
| G4 重型闸放行 | 放行判法按它写的做；放过的只有同路径尾部的副本（已有同判先例）；one-scenario 目录对得上，规则写「tests/」比判法读的「tests/*.rs」宽一档 | 量的（`classify` 与真钩子各喂命令） |

## 2. G1：两处 D13 已定项 17 复述对不对

定案原文在 `.claude/kb/decisions/13-验证路线.md:435`（已定项 17「定案」第四条，一整行）。我只比这一条与它的射程句（`:439`）；两种排除不在射程句里另有限制，射程句管的是复用键，两处复述都落在这个射程里。
被判的复述两处：`.claude/rules/implementation-workflow.md:31`（「复用重型门禁的判定」那一节正文）与 `.claude/rules/verification.md:50`（「两档：轻型与重型」重型那一行）。

### 2.1 逐项对照表（定案原文 / 规则原文 / 相同·多·少）

`.claude/kb/decisions/13-验证路线.md:435`：「两种排除不进键，理由不同、分开记。」
`.claude/rules/implementation-workflow.md:31`：「算键时有两种排除，单元不进任何一份判定的键：」
判：多（对边级）。定案里「它们的单元不进任何一份判定的键」只挂在文件级；边级的原话是「不沿它收」。规则把「不进任何一份判定的键」提到前导句，同时盖住两种。`.claude/rules/verification.md:50` 同样把「单元不进键」写在两种排除之后。推翻它的现象：找到一个判定入口，它的闭包经别的路径到达 `host-resources.py` 的单元（那样这些单元照进那条判定的键，与「不进任何一份」相反）；我只验了形式化入口一个（`scripts/gate/lib_function_units.py:9882` 起的 `real_cut_cell`，切边前后少掉的单元只经这条边可达），别的入口没算——推的，没量过。

`.claude/kb/decisions/13-验证路线.md:435`：「文件级：`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与单元算法 `scripts/gate/lib_function_units.py`）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键。」
`.claude/rules/implementation-workflow.md:31`：「文件级——`scripts/crash/lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 那两份（算键的库自己与 `scripts/gate/lib_function_units.py`），按角色排；」
判：对象与「那两份」相同（代码里 `scripts/crash/lib_verdict_library.py:88` 的 `NOT_JUDGING_FILES` 恰是这两个文件）；少了「管怎么存、怎么算键，不管判对错」——这半句是「按角色排」的判据（以后要不要把第三个文件也按角色排，靠它判）。`.claude/rules/verification.md:50` 连「按角色排」都没有。

`.claude/kb/decisions/13-验证路线.md:435`：「边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条，第二台那一路只出提示，按这条边排出键。」
`.claude/rules/implementation-workflow.md:31`：「边级——只打提示、不改判定结局的子进程边，按实测排、不沿它收，现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条。」
判：相同，少「第二台那一路只出提示，按这条边排出键」（这是那一条边为什么排的事实，属论证，规则不写是对的，`rules-discipline.md` 第 1 条）。`.claude/rules/verification.md:50` 写成「边级排除（只打提示、不改判定结局的子进程边，现在只有 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条）的单元不进键」，少了「按实测排、不沿它收」。

`.claude/kb/decisions/13-验证路线.md:435`：「边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。」
`.claude/rules/implementation-workflow.md:31`：「新加边级排除配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达；」
判：三格的字相同；规则多了「新加」两个字，把定案的「边级排除要配」缩成「新加的才配」。已有的那一条在代码里确实配齐了（`scripts/gate/lib_function_units.py:9856` 的 `real_cut_cell`；`scripts/gate/verus-proof-check.py:645` 的 `HOST_LOOKUP_LINE`；`scripts/hosts/run-with-memory-cap.sh:125` 的「整体兜底」），所以「新加」是把定案落成可做的动作，不是错；但它是规则自己加的限定，定案没写。另一格「键不变而退出码变」必红：规则写了（「另加一格」，定案是「另有一格」），代码里对应 `scripts/gate/lib_function_units.py:9756` 的 `IMPORT_TIME_EXIT_CELL`。只有 implementation-workflow 写了这条自证，verification.md 没写：两处不一致吗——不矛盾（verification.md 是跑法表，没复述自证），但「只在一处写」成立。

`.claude/kb/decisions/13-验证路线.md:435`：「python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。」
`.claude/rules/implementation-workflow.md:31`：「python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。」
判：相同（「收进来」改「进闭包」，意思同）。`.claude/rules/verification.md:50`：「python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包；」少那四样的清单。

### 2.2 其余三问

1. 「现在只有那一条」这类现状句该不该进规则：不该。同一句话现在有三份（kb 定案、implementation-workflow、verification.md），第二条边出现时要改三处；而单一来源在代码里：`scripts/gate/lib_function_units.py:7150` 的 `EXCLUDED_PROGRAM_EDGES`。`rules-discipline.md` 第 1 条允许规则写「什么规则在哪里：指到别的规则、脚本或 kb」，所以规则写「登记在 `EXCLUDED_PROGRAM_EDGES`」比写「现在只有…」好。有没有门禁在核三处同步：没量过（我没找，不下结论）。推翻它的现象：仓里有门禁核这三处文字相同。
2. 理由与来历（`rules-discipline.md` 第 1、2 条）：被判的两处没有日期、没有「谁定的」、没有旧做法；「按角色排」「按实测排」是两个判据标签，不是论证，留着合规。
3. 有没有与别的规则读来矛盾：我 grep 了 `.claude/rules/` 里「不进键」「NOT_JUDGING_FILES」「EXCLUDED_PROGRAM_EDGES」，除这三处之外没有第四处复述，没有矛盾可比。

### 2.3 该不该改、改哪一句

- `.claude/rules/implementation-workflow.md:31`：把前导句「算键时有两种排除，单元不进任何一份判定的键：」改成「算键时有两种排除：」，文件级末尾补「，它们的单元不进任何一份判定的键」；边级末尾的「，现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条」换成「（登记在 `scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES`）」；「新加边级排除配三格自证」改「边级排除配三格自证」或保留「新加」并接受它是规则的限定（我倾向照定案去掉）。文件级补「管怎么存、怎么算键，不管判对错」。
- `.claude/rules/verification.md:50`：「文件级排除（…）与边级排除（…）的单元不进键」改成「文件级排除（…）的单元不进键、边级排除（…）的边不沿它收」，去掉「现在只有…」；模块级语句那句补不补四样清单是取舍：这一格是跑法表，指到 implementation-workflow 那一节也可。

## 3. G2：复跑副本保留那一句

被判的一句：`.claude/rules/three-way-inference.md:127` 末尾：「主 agent 判决复跑的副本，到这一题的下一轮判决写完、或这一题不再开下一轮时才算用完，那时再删」，后面括注说第 11 条「用完就删」的「用完」指这一步。

### 3.1 与 `.claude/main-agent.md` 第 11 条

`.claude/main-agent.md:30`（第 11 条）：「本会话建的仓副本、编译目录与运行目录里的库拷贝（崩溃放量、变异跑完留下的），哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」。
判：读来矛盾，字面上两处顶牛。
- 第 11 条把删的时刻定在「哪一步用完就在那一步」，并明说「不等一批收尾」；新句把删的时刻挪到「这一题的下一轮判决写完」，中间隔着下一轮的派腿、等腿、核查，比一批还长。
- 第 11 条的「不留给以后的趟或别的任务复用」：判决复跑副本留到下一轮，目的就是下一轮还要用它（否则不必留），这正是「留给以后的趟」。
- 括注替第 11 条解释「用完指这一步」，但第 11 条本身没有这个解释（它没提三方判决复跑）；所以新句不是澄清，是在改第 11 条的射程。要改就该两处同改，只改一处读者会照第 11 条删掉。
推翻它的现象：第 11 条里另有一句把三方判决复跑副本排除在「用完就删」之外（我读了第 11 条全文，没有；`.claude/main-agent.md` 其余条里也没 grep 到「判决复跑」）。

### 3.2 与收工闸 `.claude/hooks/completion-cleanup-guard.sh`

闸什么时候判：`.claude/hooks/completion-cleanup-guard.sh:11`：「Stop（主 agent）：手里没有在跑的后台任务与子 agent 时才判；有就放行、什么都不打。」——主 agent 在轮间等用户、手里没有腿在跑时，正是它判的时刻，也就是新句要求「留着」的那段时间。
判哪些目录：主 agent 判本会话认领（`work-directory-ownership.sh claim`）的目录与本会话的临时草稿目录（`.claude/hooks/completion-cleanup-guard.sh:19`）；没认领、也不在临时草稿目录里的判决复跑目录，闸看不见。三方腿报告里怎么认领主 agent 的复跑目录，我没找到规定（复核不了）。
判哪几样：`.claude/hooks/completion-cleanup-guard.sh:38`：「编译目录：带签名对得上的 CACHEDIR.TAG，或带 .rustc_info.json」；`.claude/hooks/completion-cleanup-guard.sh:39`：「仓副本：有 .git（目录；.git 是文件的报成工作树），或同时有 Cargo.toml 与 crates」。认法在 `scripts/agents/agent-watch.py:1443`：「if ".git" in names or ("Cargo.toml" in names and "crates" in names):」。只有脚本与输出文件的目录这三样都不是，闸不列。
能不能声明成交付物：`.claude/hooks/completion-cleanup-guard.sh:46`：「种类四选一：报告、补丁、备份、交接」。主 agent 免列的只有前三种：`.claude/hooks/completion-cleanup-guard.sh:349`：「exempt_kinds = set() if broken("deliverables-ignored") else {"报告", "补丁", "备份"}」；交接对主 agent 照列：`.claude/hooks/completion-cleanup-guard.sh:47`：「交接：子 agent 交给主 agent 接着用的副本（主 agent 照它的报告在里面跑命令）。子 agent 交回、收工时不列；主 agent 结束一轮时照列，用完就删。」；编译目录声明了也照列：`.claude/hooks/completion-cleanup-guard.sh:48`：「声明了的那一棵里仓副本与库拷贝不列，编译目录照列」。判决复跑副本既不是报告、补丁，也不是「打补丁前的备份」，没有一种如实的 `.deliverables` 写法能把它留下来。
拦了怎么办：`.claude/hooks/completion-cleanup-guard.sh:51`：「拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户定（免得卡死）；单子变了照拦。」——所以不是死拦：第一次收工拦下列单，主 agent 照旧不删再收工一次，闸放行并把单子丢给用户。代价是每次在轮间收工多一次拦与一条给用户的消息。
我跑了它的 `--selftest`（`bash .claude/hooks/completion-cleanup-guard.sh --selftest`，登记为 light，改的不是它，在主工作区跑）：「✓ completion-cleanup-guard.sh 自证：各格都对（判了 40 格）；26 个弄坏开关各红在点名的格」，退 0（现版）。与本节有关的格：`main-repository-copy-left`（`.claude/hooks/completion-cleanup-guard.sh:1220`）、`main-handover-copy-left`（`:1333`）、`build-directory-never-deliverable`（`:1325`）、`unchanged-after-block-releases`（`:1420`），这几格都在那 40 格里绿。开工时的那一版（sha256 `09143da1…`）上我也跑过一遍：36 格、25 个开关，同样绿；之后别的会话改了这份闸（加的是「没判全也放行但必记检出」，与交付物声明和续跑放行无关），现版 sha256 `83ac5f26…`。

### 3.3 标准复跑副本里有什么（推的）

模板 `scripts/agents/three-way-rerun-template.sh:76`：「rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"」——冻结副本（腿开工时拷出的整棵树，含 `crates/` 与 `Cargo.toml`）被拷进复跑的草稿目录，闸按上面的认法会把 `$WORK/tree` 认成仓副本；`rerun_steps` 里要是编过东西，`$WORK` 里还会有带 `.rustc_info.json` 的编译目录。所以：代码轮、带 Rust 原型的轮，留着的复跑副本会被闸列；像本轮这样的模型（只有 `feed.py` 与 `out/`，我的复跑目录里没有 `Cargo.toml` 加 `crates`）不会。
这一条我没在真的 Rust 复跑副本上量（本轮没有）。仓里留下的数据点：`records/2026-10-19-里程碑四GPU垃圾回收开工.md:291`：「根分区一度满（59M）：本会话删了约 32.5G（r1 判决复跑目录 6.7G」——判决复跑目录单个就是 6.7G 量级，一题留两轮就是翻倍。
`records/2026-10-18-里程碑三收尾与近期欠账.md:120` 记过按第 11 条的做法：「删：复跑副本编译树」，即现行做法是跑完判决就删。

### 3.4 结论

- 判定：新句「能不能照做」——对只含脚本与输出的复跑目录能照做、收工闸不拦；对含仓副本或编译目录的复跑目录，照做会被收工闸拦一次再放给用户，且与第 11 条字面矛盾。
- 推翻它的现象：（一）主 agent 的判决复跑目录既没认领、也不在临时草稿目录里（闸不看它）——那么与闸不冲突，只剩与第 11 条的字面矛盾；（二）闸的 `--selftest` 在这四格上红。
- 该不该改：该改。改哪一句：`.claude/rules/three-way-inference.md:127` 这一句，限定留的是什么，例如把「主 agent 判决复跑的副本，到…才算用完，那时再删」改成「判决复跑留下的输出与模型目录拷贝，到…写完才删；其中的仓副本、编译目录跑完这一步就删」，并去掉括注里替第 11 条下定义的那半句；或同时改 `.claude/main-agent.md` 第 11 条，写明三方判决复跑的输出例外——二选一，由主 agent 定。

## 4. G3：commit-stages 第 2b 步「GPU 那一趟」逐项落行

被判的是 `.claude/skills/crash-test/commit-stages.md` 第 2b 步新加的几句（第 23 行引言、第 26 行 ③、第 28 行 ③ 退非 0 之后）。对象是 `scripts/mutation/broad-mutation.py`（写报告时 sha256 仍是快照的 `772fe040…`；别的会话说这一两个小时会换新版，换了之后行号会漂，下面每处引文都带原文，重核时按原文找）。判法：我只读源码与驱动文件头，没起驱动（起驱动要 cargo 与卡，也不归这一轮）。

格式：每项「规则原句 ⇄ 源码落点 → 判」。判 = 相同 / 多（规则写了源码没做）/ 少（源码做了规则没写）。

1. 两趟分开记、判定在同一库目录：`.claude/skills/crash-test/commit-stages.md:23`：「判定在同一个库目录的 `gpu-verdicts.tsv`，两趟的结局分开记，一条两趟里任一趟红算抓到」 ⇄ `scripts/mutation/broad-mutation.py:142`：「GPU 那一趟的行在 gpu-verdicts.tsv」；`:132`：「一条两趟里任一趟红算抓到，所以只给 CPU 那一趟没红（missed）的条跑」→ 相同。
2. 只给 CPU 没红、又有带 gpu 特性测试够得着的条：同上 `:132`，代码在 `scripts/mutation/broad-mutation.py:3913`：「if not in_run_scope(item, arguments):」起的那段筛选（3911–3932）→ 相同。
3. 接在 CPU 那一趟全跑完之后：`.claude/skills/crash-test/commit-stages.md:26`：「CPU 那一趟全跑完之后驱动接着补 GPU 那一趟」 ⇄ `scripts/mutation/broad-mutation.py:3891`：「CPU 那一趟全跑完之后的 GPU 那一趟」→ 相同。
4. 先在没改过的树上跑绿当基线，不绿退 6：`.claude/skills/crash-test/commit-stages.md:26`：「先在没改过的树上把要跑的目标跑绿一遍当基线（不绿退 6）」 ⇄ `scripts/mutation/broad-mutation.py:139`：「这一趟要跑的目标先在没改过的树上跑一遍」；退出码值 `scripts/mutation/broad-mutation.py:603`：「EXIT_INCONSISTENT, EXIT_USAGE, EXIT_INFRASTRUCTURE, EXIT_LOCKED, EXIT_DIRTY_TREE, EXIT_BASELINE_RED, EXIT_STOPPED = 1, 2, 3, 4, 5, 6, 143」→ 相同。
5. 编译步命令：`.claude/skills/crash-test/commit-stages.md:26`：「`cargo test --release -p singlefs-checker-tier --features <tests/ 里 cfg 点到的特性> <目标> --no-run`」 ⇄ `scripts/mutation/broad-mutation.py:3382`：「head = ["cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE, "--features", ",".join(features), *targets]」→ 相同，少 `--locked` 一个词（命令由驱动拼，没人照规则手敲，无害；驱动文件头 `:126` 同样没写）。
6. 测试步加 `-- --nocapture --test-threads 1` 并整条经清场包装：`.claude/skills/crash-test/commit-stages.md:26`：「同一条命令不带 `--no-run`，后面加 `-- --nocapture --test-threads 1`，整条经 `scripts/crash/gpu-quiesce-run.sh` 照多机配置清场并回读」 ⇄ `scripts/mutation/broad-mutation.py:3387`：「return quiesce + head + ["--", "--nocapture", *threads]」，`:3386`：「threads = [] if break_is("gpu-test-threads-dropped") else ["--test-threads", "1"]」，`:3385`：「quiesce = [] if break_is("gpu-quiesce-dropped") else ["bash", QUIESCE_RUNNER, "--"]」→ 相同。
7. `BINDGEN_EXTRA_CLANG_ARGS` 取崩溃放量那个值：⇄ `scripts/mutation/broad-mutation.py:3791`：「extra = {"BINDGEN_EXTRA_CLANG_ARGS": self.bindgen} if self.bindgen else {}」，值的来源 `:3370` 读 `CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS` → 相同。
8. 每一步仍经内存包装、取 `--memory-cap` 与 `--threads`：⇄ `scripts/mutation/broad-mutation.py:3778`：「内存包装在最外层，清场包装在测试步命令里面」→ 相同。
9. 清场跑完不复原，复原手动：`.claude/skills/crash-test/commit-stages.md:26`：「清场跑完不复原（要复原手动跑多机配置 `QUIESCE_START_COMMAND` 那一条私有复原脚本，再跑 `QUIESCE_STARTED_CHECK_COMMAND` 回读到它退 0）」 ⇄ `scripts/crash/gpu-quiesce-run.sh:4`：「凡是起 GPU 的跑法都经这一份：照多机配置清场并回读、跑那条命令、交回那条命令的退出码。跑完、跑红或被打断都不复原：」→ 相同。
10. 几趟同时用卡共用状态目录：`.claude/skills/crash-test/commit-stages.md:26`：「共用清场包装的状态目录、不抢在跑的重型」 ⇄ `scripts/crash/gpu-quiesce-run.sh:45`：「同一台机器上同时用卡的各趟要用同一个状态目录」；驱动里 `GPU_QUIESCE_STATE_DIRECTORY` 零命中（`grep -c` 输出 0），即各趟都取包装的缺省值 → 相同（共用靠缺省值，推的：我没逐个核崩溃放量那几个入口是否另设了它）。
11. 没卡：`.claude/skills/crash-test/commit-stages.md:26`：「机器上没有卡（日志里有 `NOT_RUN_NO_GPU_ADAPTER`）」 ⇄ `scripts/mutation/broad-mutation.py:3306`：「GPU_NO_ADAPTER_MARK = "NOT_RUN_NO_GPU_ADAPTER"」→ 相同。
12. 清场回读没过：规则写「（退 78：卡上有不归清场白名单的进程，等它跑完再重起同一条 ③，不停它）」 ⇄ `scripts/mutation/broad-mutation.py:3450`：「return GpuOutcome(GPU_NOT_RUN, "", "quiesce-refused：清场回读没过（卡上有不归清场白名单的进程，或多机配置里没写全清场命令），测试没起")」；包装自己退 78 的原因表 `scripts/crash/gpu-quiesce-run.sh:60`：「78 命令没起：多机配置里 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 没写全、状态目录或本机日志建不了（含取不到工作根）、锁等不到、清场回读没过；」→ 相同，少：规则只写了其中一个原因及其出路；多机配置没写全、状态目录建不了、锁等不到这三种，「等它跑完」不是出路。
13. `--skip-gpu`：⇄ `scripts/mutation/broad-mutation.py:4514`：「parser.add_argument("--skip-gpu", action="store_true")」，`:3947`：「not_run_reason = "skip-gpu：这一趟带了 --skip-gpu" if arguments.skip_gpu else ""」→ 相同。
14. 未跑的条逐条打 `BROAD_MUTATION_GPU_NOT_RUN`，不算没红也不算通过：⇄ `scripts/mutation/broad-mutation.py:3956`：「say(f"BROAD_MUTATION_GPU_NOT_RUN sequence={item['sequence']} reason={not_run_reason.split('：')[0]} name={item['name']}")」→ 相同。
15. ④ 因此判红：⇄ `scripts/mutation/broad-mutation.py:4147`：「if state.state not in DECISIVE_OUTCOMES:」把「本次未跑」(状态 `NOT_RUN`) 算进没结论的一堆，`:4190`：「gate_red = bool(without or counts["missed"] or unlisted or total == 0)」→ 相同。
16. 报的三种行：`.claude/skills/crash-test/commit-stages.md:26`：「`BROAD_MUTATION_GPU_PLAN`（要这一趟几条、现判几条、沿用几条）」 ⇄ `scripts/mutation/broad-mutation.py:3934`：「say(f"BROAD_MUTATION_GPU_PLAN needed={needed} judge={len(pending)} carried={len(carried)} refused={refused} features={','.join(judging.features)}")」；ITEM 行 `:3976` 起（`cpu=` 与 `gpu=` 分开写），TOTAL 行 `:3898` 起（`needed judged carried not_run`）→ 相同（TOTAL 规则括号写「现判、复用、本次未跑」，源码还有 `needed`，不矛盾）。
17. 成功句末尾：⇄ `scripts/mutation/broad-mutation.py:4665`：「return f"GPU 趟（要 {totals.needed} 条）：现判 {totals.judged}、复用 {totals.carried}、本次未跑 {totals.not_run}"」→ 相同。
18. GPU 趟不做超时升档，超时、内存撞顶记没结论：⇄ `scripts/mutation/broad-mutation.py:147`：「没做：升档（超时、撞顶两趟取证）」，结局类 `:581`：「ENVIRONMENTAL_OUTCOMES = ("timeout", "memory-cap")」；出路「`--timeout-floor` / `--memory-cap` 加大再 `--item <序号> --rerun`」出自驱动文件头 `:138`（「加大 --timeout-floor / --memory-cap 再 --item <序号> --rerun」）→ 相同。
19. `gpu-driver-error` 也走「加大 `--timeout-floor` 或 `--memory-cap` 再 `--item <序号> --rerun`」：`.claude/skills/crash-test/commit-stages.md:28`：「理由写 `gpu-timeout` / `gpu-memory-cap` / `gpu-driver-error` 的是 GPU 趟没结论的，加大 `--timeout-floor` 或 `--memory-cap` 再 `--item <序号> --rerun`」 ⇄ 驱动只对 timeout / memory-cap 写这条出路（`scripts/mutation/broad-mutation.py:138`）；驱动错有四个来源：编译步退 101 而日志里没有带源码位置的编译错误（`scripts/mutation/broad-mutation.py:3421`：「编译那一步退 101 而日志里没有带源码位置的编译错误（磁盘满、锁文件不符这一类）不记无效，记驱动错。」）、编译步认不出的退出码、`:3446`：「if passed == 0:」（退 0 却一个测试都没跑到）与 `:3459`：「return GpuOutcome(DRIVER_ERROR, "", f"{GPU_TEST_STEP} 退 {exit_code}（认不出的退出码，不是这条变异的结局）")」——这四处都不是时限与内存的事（磁盘满、锁文件不符、一个测试都没跑到），加大两个参数不会改变结局。→ 多（这一格是 G3 里唯一规则写了而源码不支持的）。
20. 四个数抄在 `BROAD_MUTATION_GATE` 那一行：⇄ `scripts/mutation/broad-mutation.py:4136`：「gpu_fields = f" gpu_needed={len(gpu_needed)} gpu_reused={gpu_reused} gpu_not_run={gpu_not_run} gpu_without_conclusion={gpu_without}"」，GATE 行 `:4192` 起把 `{gpu_fields}` 接进去 → 相同。
21. `NO_CONCLUSION` 里理由 `gpu-not-run` / `gpu-timeout` / `gpu-memory-cap` / `gpu-driver-error`：⇄ `scripts/mutation/broad-mutation.py:3473`：「return ItemState(gpu.state, cpu.passed_stage_one), f"gpu-{gpu.state}"」（状态取 `timeout`、`memory-cap`、`driver-error`），`:3476`：「return ItemState(NOT_RUN, cpu.passed_stage_one), "gpu-not-run"」→ 相同。
22. 被停退 143、在途不落行，重起同一条 ③，GPU 趟沿用得了的也不重跑：⇄ `scripts/mutation/broad-mutation.py:261`：「143 被 TERM / INT 停（在途那一条不落行）」，沿用 `:3929`：「if gpu.state in DECISIVE_OUTCOMES and not arguments.full」（带 `--full`、`--rerun` 才不沿用）→ 相同。
23. 两台各跑一片：`.claude/skills/crash-test/commit-stages.md:26`：「没卡或不清场的那一台加 `--skip-gpu`，GPU 判定行不在导入的范围里」 ⇄ `scripts/mutation/broad-mutation.py:4226`：「peer_path = os.path.join(directory, "verdicts.tsv")」（导入只读 `verdicts.tsv`）与 `:147`：「--import-results 并 GPU 判定行（两台各跑一片时 GPU 趟在拿 GPU 的那一台上跑、那一台的库里有）」→ 相同。少：规则末尾「再在本机起一趟 ③…补另一片的 GPU 趟」默认本机有卡；若卡在第二台上（`--skip-gpu` 加在本机），本机补的那一趟就是「本次未跑」，GPU 判定行留在第二台库里导不过来。驱动自己的提示在 `scripts/mutation/broad-mutation.py:4211`：「要在有卡的机器上跑，没卡的机器上它们是本次未跑」，规则没写这个前提。
24. 里程碑末全量加 `--full`、GPU 趟也现判：⇄ `scripts/mutation/broad-mutation.py:3929`（`not arguments.full` 才沿用）→ 相同。

### 4.1 G3 判定

- 24 项里 20 项相同；4 处出入：第 19 项规则多写（driver-error 的出路）；第 12、23 项各少一块前提（78 的另三种原因、补 GPU 趟要在有卡的机器上起）；第 5 项少 `--locked`（无害）。
- 量的还是推的：落行是读源码逐项对的（量的）；第 10 项「各入口都取缺省状态目录」是推的；驱动没起，所以「GPU 趟真的按这些行为跑」我没验（要卡与 cargo，不归这一轮）。
- 什么现象会推翻：第 19 项——驱动里有一处把 driver-error 当超时或内存问题处理（我 grep 了 `DRIVER_ERROR` 在 GPU 路径上的全部用法：3433、3434、3447、3459 产生它，3472 把它合成理由 `gpu-driver-error`，没有哪一处因它加大时限或内存）；第 23 项——驱动在没卡的本机上能从第二台库里读到 GPU 行（`import_results` 只读 `verdicts.tsv`，已查）。
- 该不该改：改两句。`.claude/skills/crash-test/commit-stages.md:28`：把「`gpu-driver-error`」从那句拆出去，单独写成「理由写 `gpu-driver-error` 的是 GPU 趟一个测试都没跑到或退出码认不出，看那一步的日志找原因，修好再 `--item <序号> --rerun`」，「加大 `--timeout-floor` 或 `--memory-cap`」只留给 `gpu-timeout` / `gpu-memory-cap`。`.claude/skills/crash-test/commit-stages.md:26`：「再在本机起一趟 ③…补另一片的 GPU 趟」前加「（本机有卡时；卡在第二台上则在第二台上补）」，并把「清场回读没过」的括注改成「退 78：…」只写 gpu-quiesce-run.sh 退 78 的全部原因的指路（`scripts/crash/gpu-quiesce-run.sh` 文件头「退出码」），不再单写其中一种的出路。

## 5. G4：`replay.sh --list-items` 的放行判法，与 one-scenario 的目录

### 5.1 先从条款与脚本推（喂之前写下的预期）

- 钩子文件头自己的条款：`.claude/hooks/lib_heavy_tests.py:587`：「不算的只有摘掉 --force 之后恰好一个 --list-items 的」，`:589`：「别处同名的拷贝不按名字判，读正文。」；实现 `.claude/hooks/lib_heavy_tests.py:3247`：「if [argument for argument in arguments if argument != "--force"] == EXPERIMENT_REPLAY_QUERY_ARGUMENTS:」。
- 脚本自己的行为：`scripts/experiment-apparatus/replay.sh:1357`：「if [[ "${1:-}" == "--list-items" ]]; then」，带别的参数就退 2：`scripts/experiment-apparatus/replay.sh:1359`：「echo "  ✗ --list-items 不带别的参数（收到：$*）" >&2」；`--force` 不是 replay.sh 的参数，是准入脚本在入口先摘掉的：`scripts/experiment-apparatus/replay.sh:100`：「preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}」与 `.claude/singlefs-ai-sop/scripts/preflight.sh:31`：「if [[ "$preflight_argument" == --force ]]; then PREFLIGHT_FORCE_GIVEN=1; else PREFLIGHT_ARGUMENTS+=("$preflight_argument"); fi」——摘的是任意位置的 `--force`，所以「摘掉 --force 之后恰好一个」与脚本实际行为一致。
- 弄坏开关不能借环境变量在平常跑里打开：`scripts/experiment-apparatus/replay.sh:95`：「echo "  ✗ REPLAY_BREAK 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没跑" >&2」。
- 推出：放行 `--list-items`、`--force --list-items`、`--list-items --force`（任意包装里）；拒绝其余一切带 `replay.sh` 的重型写法（不带参数、实验号、`--admission-only`、`--full`、`--list-items` 后面带别的词）。
- 位置的判法（推）：`.claude/hooks/lib_heavy_tests.py:3326`：「return any(candidate == location or candidate.endswith("/" + location)」——按路径尾部 `scripts/experiment-apparatus/replay.sh` 认，不要求在本仓根下；所以任何一份路径尾部相同的拷贝都按名字判、不读正文。这与 `:589`「别处同名的拷贝不按名字判」的字面不同：只有路径尾部不同的才读正文。崩溃点重放那一类早就是同样的判法（`.claude/hooks/lib_heavy_tests.py:573`：「入口按名字与位置认：名字叫 crash-point-replay.sh、又落在 scripts 这一层（路径尾部」）。

### 5.2 喂进去（量的）

两份喂法，脚本与原样输出都在模型目录：`feed.py` 把 46 条命令文本交 `heavy_tests_in_command_text`（先剥包装再逐条 `classify`），`hook_feed.py` 把 19 条按 PreToolUse 的 JSON 交真钩子 `heavy-test-guard.sh`（子 agent 与主 agent 两种身份；检出记录经 `AGENT_HOOK_DETECTIONS` 改写进草稿目录）。`probe.py` 真跑 `replay.sh` 三种 `--list-items` 写法并比 `xargs`/`find -exec` 对三类的判法。

`out/feed.txt` 的计数（`awk` 数的）：放行类 13 条全部放行；拒绝类 26 条里 23 条判 `replay-all-stage`（`--list-items --selftest` 一条判成 `commit-check-selftest-unregistered`，也是拒），3 条判成放行（下面的 b22–b24）；记实际类 7 条（c01–c07）全部放行。放行的 13 条：裸写、`--force` 在前、`--force` 在后、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`bash -c '…'`、`env X=… bash …`、在 `scripts/experiment-apparatus` 里 `bash replay.sh`、接管道 `| head`、`/usr/bin/time -v`、`SINGLEFS_HEAVY_TESTS=commit` 前缀、`cd … && bash ./replay.sh`。
拒的 23 条：不带参数、`E142`、`--list-items E142`、`--admission-only`、`--full`、`--list-items --full`、`--list-items --list-items`、`--list-items; … E142`、`--list-items && …`、`capped.sh … E142`、`run-with-memory-cap.sh … 不带参数`、`bash -c '… E142'`、变量 `X=E142; bash … $X`、数组 `"${args[@]}"`、`--list-items=E142`、`-- --list-items`、`source`、`sh`、`./replay.sh E142`、`exec bash`、单独 `--force`、`--list-items --force E142`。
我在会话里另见一次实况：先前我拿循环变量 `$a` 拼 `bash scripts/experiment-apparatus/replay.sh $a` 去列项，真钩子拒了（变量不认得时按重型），这条没存进模型。

`out/hook_feed.txt`（真钩子，19 条，18 条与预期相同）：子 agent `bash replay.sh --list-items` 退 0；`E142`、不带参数、`--list-items; … E142`、`--list-items E142` 退 2；主 agent 不带前缀 `E142`、不带参数退 2，带 `SINGLEFS_HEAVY_TESTS=commit` 的 `E142` 退 0；`capped.sh`、`run-with-memory-cap.sh`、`bash -c` 里的 `--list-items` 退 0。

放过的（三类，逐类判）：
1. b22/b23：`echo E142 | xargs bash …/replay.sh`、`find . -exec bash …/replay.sh E142 ';'` 判放行；真钩子 h07 也退 0。这是已登记的老口子，不是这次加 replay.sh 才有：`.claude/hooks/lib_heavy_tests.py:610`：「xargs、find -exec、eval（对每一类重型都在的老口子）」；`out/probe.txt` 里同一写法对 `crash-point-replay.sh` 与 `crash-amplification.sh` 也判 `light`（六行 `gap`）。
2. b24 在 `classify` 这一层判放行（先 `cp` 再起拷贝），真钩子 h06 退 2：钩子读了拷出来的 `r.sh` 正文，在第 745 行撞上 replay 的重型行。所以整条钩子拦得住。
3. c01/h04：路径尾部相同的拷贝（`<草稿>/fake/scripts/experiment-apparatus/replay.sh`，正文我写成 `cargo test --all`）带 `--list-items`，`classify` 判放行、真钩子退 0、不读正文；路径尾部不同的拷贝（`<草稿>/fake2/replay.sh`，同样的正文）h05 退 2，读了正文第 2 行撞上 `cargo test --all`。这与 5.1 末条推出的一致。风险面：子 agent 在草稿目录里拷了整份仓（常见），又改了拷贝里 replay.sh 的 `--list-items` 分支去跑重型——才会漏；拷贝没改过时，`--list-items` 在 replay.sh 第一个分支里就列项退出，不编不跑（见下）。

真跑一遍证 `--list-items` 不编不跑（`out/probe.txt` 末四行，原样）：

    list	exit=0	stdout_lines=164	stderr_bytes=0	--list-items
    list	exit=0	stdout_lines=164	stderr_bytes=0	--force --list-items
    list	exit=0	stdout_lines=164	stderr_bytes=0	--list-items --force
    list	three_outputs_identical=True	new_output_directories=0

### 5.3 one-scenario 那一行读的目录

`.claude/rules/verification.md:133`：「harness 档与 checker 档一条用例一个场景」那一行写「读 `crates/singlefs-harness/tests/` 与 `crates/singlefs-checker-tier/tests/` 的用例源码，不跑 checker 档」。判法源码：`scripts/crash/crash-case-check.py:303`：「return sorted(glob.glob(os.path.join(root, "crates", HARNESS_PACKAGE, "tests", "*.rs")))」与 `scripts/crash/crash-case-check.py:307`：「return sorted(glob.glob(os.path.join(root, "crates", CHECKER_PACKAGE, "tests", "*.rs")))」；判法自己的文件头 `scripts/crash/crash-case-check.py:44`：「one-scenario：crates/singlefs-harness/tests/*.rs 与 crates/singlefs-checker-tier/tests/*.rs」；门禁阶段文件头 `.claude/gate.d/harness-model-differential-and-scenarios.sh:27`：「只读用例源码，不跑 checker 档」。
判：对得上（两个包、都是读源码文本、不起 cargo）。差一档：判法只读各 `tests/` 底下的顶层 `*.rs`，规则写成目录，读者会以为子目录也扫。我数了：harness 顶层 288 个 `.rs`、子目录 79 个；checker 档顶层 85 个、子目录 10 个；子目录里带 `#[test]` 的文件 0 个（`grep -rl '#\[test\]'`），所以今天没有被漏判的用例。
推翻它的现象：子目录里出现带 `#[test]` 的文件（那时规则那句话就会让人误信它被扫了）。

### 5.4 G4 判定与该不该改

- 放行判法：按它写的做，没有找到被放过的真重型的新类。放过的只有三样，都不是这次新增：路径尾部相同的拷贝（低风险，与崩溃点重放同判法）、`xargs`/`find -exec`（登记的老口子）、`classify` 这一层看不到的先拷后起（真钩子拦得住）。量的：46 条 `classify` + 19 条真钩子 + 真跑三遍 `--list-items`。
- 推翻它的现象：（一）找到一条经包装或改名的写法，摘掉 `--force` 后恰好剩 `--list-items` 却真的会跑实验（我试了 `capped.sh`、`run-with-memory-cap.sh`、`nice`、`bash -c`、`env`、`time`、`cd`、前缀、管道、`source`、`sh`、`exec`、变量、数组，没有）；（二）`replay.sh` 的第一个分支（`scripts/experiment-apparatus/replay.sh:1357`）被改成先做别的事再列项。
- 该不该改（都不是被判 diff 里的错，是写法与实现的差）：
  1. `.claude/hooks/lib_heavy_tests.py:589`：「别处同名的拷贝不按名字判，读正文。」改成「路径尾部不是 `scripts/experiment-apparatus/replay.sh` 的同名拷贝不按名字判，读正文」，与 `:3326` 的实现一致（崩溃点重放那一类 `:573` 已经写了「路径尾部」）。这是钩子文件头的字，归工具员，不是规则文件。
  2. `.claude/rules/verification.md:133`：「读 `crates/singlefs-harness/tests/` 与 `crates/singlefs-checker-tier/tests/` 的用例源码」改成 `crates/singlefs-harness/tests/*.rs` 与 `crates/singlefs-checker-tier/tests/*.rs`（照 `scripts/crash/crash-case-check.py:44` 的写法）。
  3. 规则没说（可补可不补，主 agent 定）：`.claude/rules/verification.md:136` 那一行、`.claude/agent-common.md:58` 的重型清单、`.claude/hooks/heavy-test-guard.sh:49` 的文件头，都没有点名 `scripts/experiment-apparatus/replay.sh` 直接起算重型、`--list-items` 与登记为 light 的单独 `--selftest` 不算（只有 `.claude/hooks/lib_heavy_tests.py:587` 一处写了）。广谱变异、崩溃放量在这几处都点了名并写了「`--list-items` … 不算」，对称起见该补一句。

## 6. 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 复述对不对 | 骨架相同，5 处出入 | 多：「单元不进任何一份判定的键」盖到了边级（kb 对边级只说「不沿它收」）、「新加」限定；少：文件级的「管怎么存、怎么算键，不管判对错」、verification.md 缺边级的「不沿它收」与模块级四样清单；「现在只有」现状句三处各存一份，该换成指到 `EXCLUDED_PROGRAM_EDGES` |
| G2 复跑副本保留 | 与第 11 条字面矛盾；收工闸对含仓副本/编译目录的副本拦一次再放 | 无如实的 `.deliverables` 种类能留；只含脚本与输出的副本闸不看 |
| G3 GPU 那一趟 | 24 项里 20 项相同，4 处出入 | 多：`gpu-driver-error` 套了「加大时限或内存」的出路；少：78 的另几种原因、补 GPU 趟要在有卡的机器上起、`--locked` |
| G4 重型闸放行 | 按写的做；放过的三样都不是新类 | `--list-items` 真跑 164 行、不建输出目录；one-scenario 的目录对得上，规则写「tests/」比判法读的「tests/*.rs」宽一档 |

## 7. 没做什么

- 没替主 agent 采纳或出判决；上面「该不该改」是推出来的建议，改不改、两处矛盾取哪一边由主 agent 定。
- 没起 `broad-mutation.py`（要 cargo 与卡）；G3 只对到了源码，没验驱动真跑时的行为。
- 没在真的 Rust 复跑副本上量收工闸会不会列（本轮没有 Rust 复跑副本）；G2 的「标准复跑副本里有仓副本」是从模板和认法推的。主 agent 的判决复跑目录有没有认领，我没找到规定，复核不了。
- 没核「三处复述有没有门禁在核同步」（G1 第 2.2 节第 1 条）。
- 没读攻方腿的报告与模型（按禁读清单）；没碰别的腿的目录。
- 没比 `.claude/kb/` 的英文、日文译文（只比了中文定案，译文 sha 与快照一致）。
- `.claude/hooks/lib_heavy_tests.py`、`heavy-test-guard.sh`、`completion-cleanup-guard.sh` 在我写的过程中被别的会话改过（见第 0.1 节与附）；它们的改动与本轮问到的行为无关，我对改后的版本重跑了喂命令，结果相同，收工闸的 `--selftest` 在改前、改后两个版本上各跑一遍，都绿。

## 附：sha256 与清理

模型文件（`research/prompts/m3-close-gov-b17-r1-sonnet-model/`，写完时现算）：

    ec240bb21cada6ff6fc0eac09d20930e4d113177e5365c06479261b4c0f1893e  feed.py
    4f5d0c7e37f362e953817ce60f41da26add284bbce670a41ced85c9b82510d6b  hook_feed.py
    c1bb78a2ed27c060669e1c5b8c13c4ac2c0ec40290fabe46e0ed58e5714c122c  probe.py
    9e482b11f615791a476d3af959ab55783e652d7ae907eac35485a800d4180818  tools/rerun.sh
    0c7434e4acb19d0a50684578b62a0998e994a09c3889a1ac20bcee01303bad9e  out/feed.txt
    2a7cfec3ab347b75c6a631e9c44a0b0cd25161926c7fa93c847b88f4a06fe4f6  out/hook_feed.txt
    aad526014861920ea5435be03bd55e2e12c59e2247421aee07931693b754debc  out/probe.txt

被喂的版本（写完时）：`lib_heavy_tests.py` `b1c3ef8d…`、`heavy-test-guard.sh` `ed9e3102…`、`completion-cleanup-guard.sh` `83ac5f26…`、`broad-mutation.py` `772fe040…`（后三者里 `broad-mutation.py` 与快照相同）。
`out/feed.txt` 在 `lib_heavy_tests.py` 的 `51b96bbb…` 与 `b1c3ef8d…` 两版上除 sha 那一行外逐行相同（`feed.py` 现在把 sha 打到 stderr）。

清理：草稿目录 `~/.local/share/singlefs/work/m3-close-gov-b17-r1-sonnet/` 里没有仓副本、编译目录或库拷贝；复跑用的 `rerun-work*` 与 `hookscratch*` 小目录（合计约 400K，`du -sh` 开工后量 488K，删后 92K）已删，留下的只有日志与输出文件。没有起 cargo，没有后台进程留着（第四趟复跑被内存包装排队卡住，我用 `proc.py stop` 停了，退 143）。

cite-check（交回前，`python3 scripts/agents/cite-check.py research/prompts/m3-close-gov-b17-r1-sonnet-output.md`）末行：

    ✓ 核了 79 处引文、29 处裸行号，对上 79 处，没判 29 处（其中裸行号没有引文可比 29 处；逐处列在上面）
