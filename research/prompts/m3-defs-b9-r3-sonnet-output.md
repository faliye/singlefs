# 第二题第三轮：云端正推腿（Sonnet）报告

日期：2026-10-14。轮名 `m3-defs-b9-r3`。格：Q5、Q6。立场：逐条核「两轮判决与正文说的数和清单，就是产物与冻结副本里的样子」。

## 读法与披露

- 三份冻结副本在 `~/.local/share/singlefs/work/` 下（下称工作根）：`m3-defs-b9-r1/repo`（after-b9-fixup）叫 r1 副本，`m3-defs-b9-r2/repo`（after-b10-fixup）叫 r2 副本，`m3-defs-b9-r3/repo`（after-b11）叫 r3 副本；集成副本产物在 `m3-tooling-b2-integration`，主 agent 的门禁日志在 `m3-defs-b9-r1-main-out`、`m3-defs-b9-r2-main-out`。
- 模型目录 `research/prompts/m3-defs-b9-r3-sonnet-model/`：`tools/rerun.sh`（模板只换 `rerun_steps`）、`tools/q6_counts.sh`（Q6 每个数的现算）、`tools/q5_repro.sh` 加三个小驱动（Q5 漏列线的现喂）、`out/q6-counts.tsv`、`out/q5-repro.txt`、`out/other-models-rerun.txt`。复跑命令见末尾「复跑与核对」。
- 越界披露：定义「开工先读」点名主工作区的 `.claude/agent-common.md`，规则三份整份读；其中 `.claude/agent-common.md` 与 `.claude/rules/three-way-inference.md` 是被判文件，与禁读清单相抵（前两轮各腿同）。我读完才对上这条；本报告没有引用它们的字面，被判字面一律取 r3 副本。禁读清单里的攻方 r3 报告、模型、草稿目录我没读；冻结副本根下 `multi-host.env` 没打开，报告与模型目录里没有它的值。数 r1 攻方 `z5` 日志用到的私有值是 r1 攻方模型 `feeds/z5` 的编造样本，只用来数行，没有写出来。
- 判定词：对＝现算结果与引处一致；对不上＝数或结论与引处不同；部分对＝数对、口径或附带说法不对；复核不了＝缺产物或只有主 agent 起得了的格。
- Q6 每行「现算」列写的编号是 `out/q6-counts.tsv` 第一列，该文件第三列是原样结果。随时间变的只有两处：`A-r1-kb`、`A-r2-kb`、`A-r3-kb`（主工作区 kb 页被别的会话改过就变）和 `G-prompts-now`（工作根下现有提示份数）。

## Q6　两轮报出的数字能不能核

### 快照、副本与引文核对

| 号 | 引处 | 引的数 | 现算（编号：做了什么） | 现算结果 | 判 | 口径缺什么；什么现象会推翻 |
|---|---|---|---|---|---|---|
| Q6-01 | `research/prompts/m3-defs-b9-r1-main-verification.md:11` | 34 份、非 OK 0 行 | `A-r1-frozen`：r1 副本里 `sha256sum -c` r1 快照 | 34 行、34 OK、0 非 OK | 对 | 无；快照文件或副本被改会推翻 |
| Q6-02 | `research/prompts/m3-defs-b9-r1-main-verification.md:12` | kb 快照两份对不上；两份腿报告按行号或标题引这两份文件 0 处 | `A-r1-kb`：主工作区 `sha256sum -c` r1 kb 快照；`A-r1-quote`：grep 两份 r1 报告 | 2 行、2 FAILED（term-renames.md、171-三oracle合谋测试.md）；引用写法 0 行 | 对 | 判决没写今天仍 FAILED；今天再核仍是这两份 |
| Q6-03 | `research/prompts/m3-defs-b9-r2-main-verification.md:12` | 35 份非 OK 0 行；kb 快照非 OK 0 行 | `A-r2-frozen`、`A-r2-kb` | 35 行、35 OK、0 非 OK；kb 1 行、0 FAILED | 对 | 无 |
| Q6-04 | `research/prompts/_m3-defs-b9-r3-body.md:13` | 这一题动过的 35 份文件 | `A-r3-frozen`；`A-r2r3-paths`：r3 快照对 r2 快照；`D-b11-nonsample`：批 11 非样本改动对 r3 快照 | 35 行、35 OK、0 非 OK；路径集合与 r2 快照相同、18 份哈希变了；批 11 非样本改动 18 份、0 份不在这 35 份里 | 对 | 正文没写「35 份里 18 份是批 11 改的」 |
| Q6-05 | `research/prompts/_m3-defs-b9-r1-body.md:21`；`records/2026-10-12-里程碑三工具与治理批.md:154`；`records/2026-10-12-里程碑三工具与治理批.md:165` | 34 份（上一题 33 份并上 …）；35 份（第一轮 34 份并上 …） | `A-r1-frozen`、`A-b6-snapshot`、`A-r2-frozen` | 34；上一题第三轮快照 33 行；35 | 对 | 无 |
| Q6-06 | `research/prompts/m3-defs-b9-r1-main-verification.md:13`：「核 117 处、对上 117 处（正推 60/60）」 | 117 处引文对上 117；正推 60/60 | `B-r1-cite-frozen-only`：判决写的命令（两份报告、`--root` 只给 r1 副本、带背景材料）；`B-r1-cite-two-roots`：再加仓根；`B-r1-cite-sonnet`、`B-r1-cite-opus` 各单跑 | 单根：核 117 引文、42 裸行号，对上 91、没判 68；加仓根：对上 117、没判 42（裸行号，其中 26 处没有引文可比）；正推单跑 60/60，攻方 57/57 | 引文数对；命令口径对不上 | 判决写的命令今天只对上 91（`research/prompts/` 下被引的产物在 r1 副本里找不到，判「没判」），要加仓根才 117；今天的 cite-check 另报 42 处裸行号，r1 判决没提（r2 判决提了 17 处） |
| Q6-07 | `records/2026-10-12-里程碑三工具与治理批.md:156`；`records/2026-10-12-里程碑三工具与治理批.md:158` | 正推 60/60；攻方 57/57，两份合计 117/117 | `B-r1-cite-sonnet-main`：正推报告 `--root` 仓根；`B-r1-cite-opus` | 正推在仓根单根下今天 4 处判红（裸文件名解析到主工作区同名文件、行号超出）；攻方 57/57 | 部分对 | 腿报告末行的 60/60 是以 r1 副本为根的结果，换根今天就红；引文数要连根一起写 |
| Q6-08 | `research/prompts/m3-defs-b9-r2-main-verification.md:13` | 核 134 处引文、对上 134 处；17 处裸行号没有引文可比 | `B-r2-cite-both`、`B-r2-cite-sonnet`、`B-r2-cite-opus` | 134/134（辩方 89、攻方 45）；裸行号 17（辩方 17、攻方 0） | 对 | 无 |
| Q6-09 | `records/2026-10-12-里程碑三工具与治理批.md:167`；`records/2026-10-12-里程碑三工具与治理批.md:169` | 攻方 45/45；两份 134/134、裸行号 17 | 同 Q6-08 | 45/45；134/134；17 | 对 | 无 |
| Q6-10 | `research/prompts/m3-defs-b9-r1-main-verification.md:5`；`research/prompts/m3-defs-b9-r2-main-verification.md:5` | 四份腿报告的 sha256 头尾 | `B-report-sha` | 85cd6ce5…3385、1c1ade79…ae47、fa4d45ee…8a27、5dd4a08d…474f | 对 | r2 判决只写了头 8 位，头 8 位相同 |

### 腿报告、模型目录与复跑

| 号 | 引处 | 引的数 | 现算（编号：做了什么） | 现算结果 | 判 | 口径缺什么；什么现象会推翻 |
|---|---|---|---|---|---|---|
| Q6-11 | `records/2026-10-12-里程碑三工具与治理批.md:156` | 正推 62 句；Z1 不一致 2、Z4 不一致 4 | `B-r1-sonnet-rows`：数 r1 正推报告句表行 | 句表 62 行（Z1 16、Z3 5、Z4 17、Z6 13、Z7 11）；判「不一致」的 6 行：Z1-10、Z1-16、Z4-02、Z4-09、Z4-10、Z4-11 | 对 | 无 |
| Q6-12 | `records/2026-10-12-里程碑三工具与治理批.md:169` | 辩方「修到 22 条」 | `B-r2-sonnet-fixed`：r2 辩方总表里判定以「修到」开头的行 | 22 行 | 对 | 22 里含「修到（文字）」「修到（顺带）」等带括注的 |
| Q6-13 | `records/2026-10-12-里程碑三工具与治理批.md:158`；`records/2026-10-12-里程碑三工具与治理批.md:167`；`research/prompts/m3-defs-b9-r1-main-verification.md:14`；`research/prompts/m3-defs-b9-r2-main-verification.md:14` | 模型目录：r1 攻方 48 份；r2 攻方 24 份；r1 攻方 out 17 份；辩方 47 份；攻方 9 份 | `B-model-files`：数模型目录文件与 `out/` 文件 | r1 正推 17（out 9）；r1 攻方 48（out 17）；r2 辩方 105（out 48）；r2 攻方 24（out 9） | 对 | 辩方 out 48 份，判决写 47 是因为 `out/s1-real-prompts.txt` 不在复跑里（判决已写） |
| Q6-14 | `research/prompts/m3-defs-b9-r1-main-verification.md:14`：「正推模型拷进草稿目录经 `run-with-memory-cap.sh 2G` 复跑：全 same」 | 正推模型 9 份「全 same」 | `B-r1-main-rerun-log`：主 agent 自己的 `rerun.log` 里「逐行相同」「不同或」的份数；本腿再跑一遍该模型（`out/other-models-rerun.txt`） | 9 份里 8 份逐行相同、`z6-script-header.tsv` 不同；两份逐行比：10 行都在短者长度内逐字相同、退出码列 10 行相同，只差末尾几个字 | 对不上 | 判决说「全 same」，主 agent 的 `rerun.log` 与本腿复跑都是 8/9。差因与判决在攻方 `z7.txt` 上写的同一个（喂法先按字截行、之后才换草稿路径，路径长短不同截断点就不同），判定没变；判决只对攻方写了这一原因 |
| Q6-15 | `research/prompts/m3-defs-b9-r1-main-verification.md:14`；`records/2026-10-12-里程碑三工具与治理批.md:160` | 攻方复跑 17 份 16 same，`z7.txt` 两行 DIFF | `out/other-models-rerun.txt`：本腿复跑 r1 攻方模型（两趟） | 16 same，`z7.txt` DIFF；差在 `reuse-entry-被起脚本` 那两行的尾部 | 对 | 无 |
| Q6-16 | `research/prompts/m3-defs-b9-r2-main-verification.md:14`；`records/2026-10-12-里程碑三工具与治理批.md:170` | 攻方 9 份全 same；辩方 47 份逐字节相同 | 同上，r2 攻方（两趟）、r2 辩方（两趟） | 攻方 9 same；辩方 26＋21＝47 same、0 DIFF | 对 | 无 |

### 样本档

| 号 | 引处 | 引的数 | 现算（编号：做了什么） | 现算结果 | 判 | 口径缺什么；什么现象会推翻 |
|---|---|---|---|---|---|---|
| Q6-17 | `research/prompts/m3-defs-b9-r1-main-verification.md:15`；`records/2026-10-12-里程碑三工具与治理批.md:157` | code-tooling 样本档 44 例、doc-process-records 样本档 22 例 | `C-r1-fixture-dirs`：r1 副本 `fixtures/<门禁>` 目录数；`C-r1-main-out-stage`：主 agent 日志末行 | 目录 44、22（doc-text 25）；日志末行 44 个用例、22 个用例 | 对 | 无 |
| Q6-18 | `research/prompts/m3-defs-b9-r2-main-verification.md:16`；`records/2026-10-12-里程碑三工具与治理批.md:170` | 三道样本档 46、22、27 例；批 10 新加四份、改过两份 | `C-r2-fixture-dirs`、`C-r2-main-out-stage`、`C-fixture-diff` | r2 副本目录 46、22、27；日志末行 46、22、27；r1→r2 新加 4、改过 2 | 对 | 无 |
| Q6-19 | `research/prompts/_m3-defs-b9-r3-body.md:22`：「三道样本档 46 / 22 / 27 例」 | 46 / 22 / 27 | `C-r3-fixture-dirs`；`C-verify-stage`：verify13、verify14 里 stage-selftest 日志的用例数 | r3 副本目录 47、22、27；verify14 日志 47、27、22 个用例（verify13 是 46、27、22） | 对不上 | 46 是 after-b10-fixup 的数；after-b11 的 code-tooling 样本多 1 份（`heavy-thread-defaults-var-source-forms-red`，r2→r3 新加 1、另改过 3）。给 r3 腿的正文该写 47 |
| Q6-20 | `records/2026-10-12-里程碑三工具与治理批.md:153` | 派发闸自证 388 种过；doc-process-records 样本档 22 例对 | `D-verify12-dispatch`：verify12/13/14 的 `self-runner-dispatch-guard` 日志 | 388、398、417 种；样本档 22 例见 Q6-17 | 对（388 是当时的数） | 自证种数随批增长；我不跑登记的 `--selftest`，今天的数只读 verify14 日志（417） |

### 集成副本复核（verify12 到 verify14）

| 号 | 引处 | 引的数 | 现算（编号：做了什么） | 现算结果 | 判 | 口径缺什么；什么现象会推翻 |
|---|---|---|---|---|---|---|
| Q6-21 | `research/prompts/_m3-defs-b9-r1-body.md:26`；`records/2026-10-12-里程碑三工具与治理批.md:153` | verify12 派 210、收 210、判错 6；4 项复核脚本设库开关被覆盖加 2 项 | `D-verify12`、`D-verify12-errors`：数 summary 编号行、`.rc` 文件、含「错（」的行 | 210 编号行、210 个 `.rc`、6 行「错（」；错的是 33–36、198、199 | 对 | 无 |
| Q6-22 | `research/prompts/_m3-defs-b9-r2-body.md:29`；`records/2026-10-12-里程碑三工具与治理批.md:164` | verify13 派 223、收 223、判错 5 | `D-verify13`、`D-verify13-errors` | 223、223、5 行「错（」；错的是 34、205、210、211、218 | 对 | 无 |
| Q6-23 | `records/2026-10-12-里程碑三工具与治理批.md:164`：「`brk-long-run-exempt-needless-unchecked` 是复核脚本直接设库开关被门禁格覆盖（同 verify12）」 | 同 verify12 的那类错 | `D-verify12-errors`、`D-verify13-errors`、`D-verify14-errors` | verify12 错 4 项（33–36）；verify13 错 1 项（34）；verify14 错 4 项（33–36）。三份 `verify1*.sh` 第 23 行的命令行逐字相同 | 部分对 | 「同 verify12」只对 verify14。verify13 里 33、35、36 三项是红的（对），没人写它们为什么在 verify13 红、在 verify12 和 verify14 不红。要主 agent 查（见「要主 agent 跑」） |
| Q6-24 | `research/prompts/_m3-defs-b9-r2-body.md:29`；`records/2026-10-12-里程碑三工具与治理批.md:164` | 15 行在 6 份文件、豁免表 10 条指不到 | `D-verify13-doc-text` | 15、6、10；verify14 的同一格同红同数 | 对 | 「同一条命令在 after-b9-fixup 的树上一样红、数相同」复核不了：那棵树不在，门禁格我起不了 |
| Q6-25 | `research/prompts/_m3-defs-b9-r3-body.md:27`；`records/2026-10-12-里程碑三工具与治理批.md:174` | verify14 派 244、收 244、判错 7 | `D-verify14` | 244、244、7；244＝235 对＋2 记＋7 错 | 对 | 「记」2 项（`drift-b9-edited`、`drift-b9-twice`）不在判错数里，判决已写「只记不判」 |
| Q6-26 | `research/prompts/_m3-defs-b9-r3-body.md:27`；`records/2026-10-12-里程碑三工具与治理批.md:174` | 122 个文件名、964 处正文的日期早于第一个提交；崩溃点重放那道门禁缺红样本（它的脚本文件名在集成副本里还是旧名） | `D-verify14-fixture-claims` | 缺红格的门禁 1 道、122、964 | 对 | 无 |
| Q6-27 | `records/2026-10-12-里程碑三工具与治理批.md:174`：「批 11 加的样本一份都不在红项里」 | 批 11 改动的样本目录在 `cell-fixture-claims` 日志里 0 处 | `D-b11-samples`：`git diff --name-only after-b10-fixup after-b11` 列出的样本目录；`D-b11-samples-in-red`：每个目录在日志里出现的次数 | 4 个目录（新建 1、已有 3）；日志里 `abandoned-rounds-source-green` 1 次、`abandoned-rounds-source-red` 1 次，另两个 0 | 对不上 | 照判决写的命令算是 2 处。红的是这两份 tsv 里 b10 就有的旧日期行（green 第 6 行没动；red 第 12 行因批 11 加了一行挪到第 13 行），不是批 11 加的内容；只算新建的 1 个目录则是 0。命令口径要写成「新建的样本」 |
| Q6-28 | `research/prompts/_m3-defs-b9-r3-body.md:27` | 两棵漂移小树：原话改了字那棵退 1，原话出现两次那棵退 0 | 读 `verify14.summary` 的 231、232 行 | 231 `drift-b9-edited` 退 1、232 `drift-b9-twice` 退 0，都标「记」 | 对 | 无 |
| Q6-29 | `records/2026-10-12-里程碑三工具与治理批.md:151`；`records/2026-10-12-里程碑三工具与治理批.md:163`；`records/2026-10-12-里程碑三工具与治理批.md:173` | 补丁 34 份 +443/−153、54 份 +477/−135、46 份 +520/−116；sha 与报告 sha | `E-patch-b9`、`E-patch-b10`、`E-patch-b11` | 34/443/153、54/477/135、46/520/116；补丁 sha 头 8 位 35d6b63c、8d882105、82926c8d 对；报告 b4fe3c74…903d、c8cc3518…181c、f90e78d3… 对 | 对 | 无 |
| Q6-30 | `records/2026-10-12-里程碑三工具与治理批.md:154`；`records/2026-10-12-里程碑三工具与治理批.md:165` | 材料 kb 2 份、清单抄 40 行；kb 1 份、清单 6 行 | `E-checklist`：数三份清单里「抄」「不抄」行与 kb 清单标题 | r1 清单 抄 40、不抄 2、kb 清单 2 份；r2、r3 清单 抄 6、不抄 0、kb 清单 1 份 | 对 | 无 |

### 门禁格与样本树的点名数（主 agent 的日志）

| 号 | 引处 | 引的数 | 现算（编号：做了什么） | 现算结果 | 判 | 口径缺什么；什么现象会推翻 |
|---|---|---|---|---|---|---|
| Q6-31 | `research/prompts/m3-defs-b9-r2-main-verification.md:17`；`records/2026-10-12-里程碑三工具与治理批.md:170` | `htd-s3` 在 r2 点名 3 处、r1 点名 8 处 | `F-htd-named`：读 `m3-defs-b9-r2-main-out/log/a-htd-s3-r2.log`、`a-htd-s3-r1.log` 标题行 | 3 处、8 处 | 对 | 无 |
| Q6-32 | `research/prompts/m3-defs-b9-r2-main-verification.md:17` | 辩方 `s3-shell` 点名 15 行（13 份） | `F-htd-named`：`s-s3-shell.log` 标题行、点名行、去重文件 | 标题 15 处；点名行 15；去重 13 份 | 对 | 无 |
| Q6-33 | `research/prompts/m3-defs-b9-r2-main-verification.md:17` | 第一轮攻方 `htd-shell` 点名 4 处、不再点名 `var_from_registry.sh`；`htd-insert`、`htd-edit` 退 0 | `F-htd-named`、`F-htd-insert-edit` | 4 处、`var_from_registry` 出现 0 次；两棵绿（各「扫了 832 份」） | 对 | 无 |
| Q6-34 | `research/prompts/m3-defs-b9-r2-main-verification.md:18` | `abandoned-misread` r2 退 0、成功句列第 1594 行，r1 退 1；辩方四棵 r2 退 0、0、0、1，r1 四棵都退 1；`abandoned-after` 退 0 | `F-abandoned`、`F-misread-hint`：看各日志有没有绿行 | misread r2 0、r1 1；after 0；辩方四棵 r2 0 0 0 1、r1 1 1 1 1；成功句「第 1594 行」 | 对 | 无 |
| Q6-35 | `research/prompts/m3-defs-b9-r2-main-verification.md:19`；`research/prompts/m3-defs-b9-r2-main-verification.md:34` | doc-text 样本的（编的）私有值整串与半截 0 处 | `F-doc-text-leak`：三份 doc-text 日志里命中编的值的份数 | 0 份 | 对 | 判决写「处」，我数的是命中的日志份数，0 份命中即 0 处 |
| Q6-36 | `research/prompts/m3-defs-b9-r1-main-verification.md:16`；`research/prompts/m3-defs-b9-r1-main-verification.md:28` | 攻方 `z5` 退 1、输出里出现样本的（编的）私有值 3 处（「漏 3 处」） | `F-r1-z5-lines`：`m3-defs-b9-r1-main-out/attack/3-z5.log` 里含编的值（8 字窗口）的行 | 3 行 | 对 | 「3 处」是 3 行（clock-times 的半截 1 行、term-renames 1 行、date-once 1 行），不是 3 个值 |
| Q6-37 | `research/prompts/m3-defs-b9-r1-main-verification.md:16` | `abandoned-after` 在 b9 退 1 点名「命中 2 次」、r3 与 b9 副本本身退 0；`htd-insert`、`htd-edit` 退 0；`htd-shell` 点名第 5 行；long-run 豁免表有表退 1、没表退 0 | `H-r1-attack-logs`：`m3-defs-b9-r1-main-out/attack/` 各日志 | 命中 2 次 1 行；r3 与对照绿；两棵 htd 绿；`htd-shell` 5 处、`var_from_registry.sh` 1 次；有表红、没表绿 | 对 | 无 |
| Q6-38 | `records/2026-10-12-里程碑三工具与治理批.md:173`；`records/2026-10-12-里程碑三工具与治理批.md:174` | `htd-s3` 改后点名 6 处不是 7 处；`s3-shell` 22 处；`s5` 四棵 0 1 1 1；`htd-shell` 4 | `F-b11-htd`：verify14 对应日志与 `.rc` | 6、22、4；`s5` 四棵退出码 0 1 1 1 | 对 | 无 |
| Q6-39 | `research/prompts/m3-defs-b9-r2-main-verification.md:30` | 缺行出路 15 个形态放拒两版逐行相同 | `F-s2-rows`：辩方 `s2-exit-matrix.txt` 的形态行与 b9、b10 两列相同的行 | 15 行、15 行两列相同 | 对 | 无 |

### 语料与库里的数

| 号 | 引处 | 引的数 | 现算（编号：做了什么） | 现算结果 | 判 | 口径缺什么；什么现象会推翻 |
|---|---|---|---|---|---|---|
| Q6-40 | `research/prompts/_m3-defs-b9-r1-body.md:34`；`research/prompts/m3-defs-b9-r1-main-verification.md:44`；`records/2026-10-12-里程碑三工具与治理批.md:158` | 排除表 89 行；17 行的锚串命中 2–5 条被判行 | `G-r1-anchors`、`G-r3-anchors`：用 r1 攻方的 `z2_htd_cell.py`（抽出格里的 python 原样跑）打每行锚串命中的被判行数 | 89 行：命中 1 条 72 行、2 条 9、3 条 6、4 条 1、5 条 1；2–5 条共 17 行；r1、r3 副本相同 | 对 | 「17 行」是命中至少 2 条的行数 |
| Q6-41 | `research/prompts/_m3-defs-b9-r3-body.md:22`：「各样本点名的份数」；`records/2026-10-12-里程碑三工具与治理批.md:148`；`records/2026-10-12-里程碑三工具与治理批.md:149` | 扫 832 份、可疑写法 117 处、排除表 89 行、装置自己读的线程变量 74 个 | `G-htd-trees`：同一个抽出的格在六棵冻结副本上跑 | r1、r3 副本 832／117／89／74；r2 副本 832／117／89／75 | 对 | 变量数随树变，r2 副本是 75；引这个数要写是哪一棵 |
| Q6-42 | `records/2026-10-12-里程碑三工具与治理批.md:128`；`records/2026-10-12-里程碑三工具与治理批.md:149` | 832／94／66；「72 → 58」是批 6 收尾那一版树的数 | `G-htd-trees` | 上一题 r1 副本 832／94／66／58，r2 副本 832／94／66／72 | 对 | 无 |
| Q6-43 | `research/prompts/m3-defs-b9-r1-main-verification.md:44`：「仓里 306 个轮名都合」 | 306 个轮名 | `G-round-names`：用 r1、r3 副本里 `lib_abandoned_rounds.py` 的 `recognised_round_names` 与目录三种文件名 | 目录三种文件名认出 306（305 个以 -r<数字> 结尾，`test-granularity-rename` 不是）；库的全集（并上登记表第一列的 `smalldata-ttime-r1`）307；字符集外 0 | 部分对 | 306 不含登记表独有的名字，全集是 307；「都合」只对字符集成立，不是 306 个都以 -r<数字> 结尾 |
| Q6-44 | `research/prompts/m3-defs-b9-r1-main-verification.md:36` | 195 个标签 | `G-labels-195`：上一题辩方 `out/fm-*.txt` 行数 | 195 | 对 | 无 |
| Q6-45 | `research/prompts/m3-defs-b9-r2-main-verification.md:34` | doc-text 之外 16 道门禁不打码 | `G-gate-stages`：`gate.d/*.sh` 个数与调打码库的个数 | 17 道，只有 `doc-text.sh` 调打码库，其余 16 | 对 | 判的是「调不调打码库」，没有逐道造样本看红行里有没有路径（攻方也标了推的） |
| Q6-46 | `records/2026-10-12-里程碑三工具与治理批.md:169` | 真记录里 458 行、79 对（96 份记录） | `G-real-records`：我另写的判法（恰有一个 `<名>-r<数字>` 轮名且带 6 字以上「」的行），数 r2 副本 `records/*.md` | 96 份、458 行、18 份文件、79 对 | 对 | 判决没写 18 份文件的口径；我的判法与辩方逐项相同 |
| Q6-47 | `records/2026-10-12-里程碑三工具与治理批.md:167` | 现有派发提示攻方 50 份、辩方 49 份，围栏块 0 份 | `G-prompts-now`：工作根下 `*/prompt.md` 份数与含围栏行的份数 | 今天 54 份，含围栏行 0 份 | 复核不了 | 语料随时间变，两腿排除的目录也不同，原数只能读两腿的原话。今天 54 份里 0 份有围栏，「P1 在真提示上一次没起过作用」今天仍成立 |
| Q6-48 | `research/prompts/m3-defs-b9-r1-main-verification.md:17`；`research/prompts/m3-defs-b9-r2-main-verification.md:22` | 行号类：`dispatch-prompt.py` 第 43、90–91 行；钩子第 39 行与门禁第 103 行仍写「不按 -r<数字> 形」；登记表第 8 行的出处是第 1595 行 | `H-r1-lines`、`H-r2-registry` | 四处都在；登记表 `m3-batch63-code-r1` 在第 8 行、出处写第 1595 行，记录第 1594、1595 行都含轮名 | 对 | 无 |

### Q6 小结

- 48 行：对 40，部分对 3（Q6-07、Q6-23、Q6-43），对不上 4（Q6-06 的命令口径、Q6-14、Q6-19、Q6-27），复核不了 1（Q6-47 的原数）；Q6-24 里「after-b9-fixup 树上一样红」那半句也复核不了（树不在）。
- 对不上的四处都能一句话改：Q6-06 的命令加仓根并报裸行号；Q6-14「全 same」改「8/9，`z6-script-header.tsv` 差在截断位置」；Q6-19 的 46 改 47；Q6-27 改成「新建的样本 0 处；已有的两份样本里 b10 就有的旧日期行 2 处」。
- 数字本身几乎都对得上：快照、cite-check 引文数、verify 三份汇总、三道样本档、点名数、832／117／89／74、306、195、458／79 都现算复现。缺的是口径：哪一棵树、哪个根、数的是份还是行、随时间变不变。

## Q5　开着的线：出处、在不在两轮的另立账 / 改法表里

范围与读法：

- 清单来源：两轮判决的改法表（V1 第 36 到 42 行的 P1–P7，V2 第 41 到 44 行的 U1–U4）与「另立一笔账」（`research/prompts/m3-defs-b9-r1-main-verification.md:44`，`research/prompts/m3-defs-b9-r2-main-verification.md:46`），加前两轮四条腿报告里打中的每一条（含句表里判「不一致」「文字没写到」的行、攻方每个「打中」、辩方每个「没修到」「改过头」「带进来」、各腿专问里量出来的缺口）。上一题三轮判决的另立账照 `research/prompts/_m3-defs-b9-r3-body.md:28` 一并列在第二张表，只列不核。
- 「在」＝判决的改法表或另立账里点了这一条（或同一现象的写法）并给了去向；「部分在」＝点了一半；「不在」＝两轮判决都没写。判决里没写、调度记录里写了的，单列。
- 「不在」的线都在 r1 或 r3 副本上现喂过，命令与原样输出在 `out/q5-repro.txt`，编号 L1–L10 见表后。
- 表里的数据来自腿报告的行号，都是 grep 现查的；去向列的 V1、V2 行号同。r3 副本上的实况（U1–U4 改过之后）列在「漏列」那张表里。

### 第一张表：这一题前两轮四条腿打中的线

| 号 | 出处（腿报告:行） | 现象 | 判 | 去向（判决:行） |
|---|---|---|---|---|
| Q5-01 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:37`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:41`（Z1-02、Z1-06） | 九份定义「输入」只写已判行「也能顶替缺的行」，没写顶替后不起重活 | 判决里不在，调度记录里在 | `records/2026-10-12-里程碑三工具与治理批.md:151`：主 agent 定共用约束每个子 agent 都读，不重复写 |
| Q5-02 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:38`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:42`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:45`（Z1-03、Z1-07、Z1-10） | 定义没写围栏种类、⑭ 围栏内不判与同一标签第二行拒；「照 CommonMark」说过头 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:36`（P1） |
| Q5-03 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:39`（Z1-04） | 说明行不能自己以「线程上限：」起头 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（辩方「判轻」列） |
| Q5-04 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:51`（Z1-16） | `dispatch-prompt.py` 第 43 行写「书记员」 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:37`（P2） |
| Q5-05 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:62`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:63`（Z3-04、Z3-05） | 仓副本那一支的说明；指到的小节没写自证怎么跑 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:39`（P4） |
| Q5-06 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:81`（Z4-01） | 行号不能省 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（判轻列） |
| Q5-07 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:82`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:85`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:89`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:90`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:91`（Z4-02、Z4-05、Z4-09、Z4-10、Z4-11） | 原话要含轮名、整行重复无路可走、「判决文件」说宽、两份文件头旧句 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:40`（P5） |
| Q5-08 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:88`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:92`（Z4-08、Z4-12） | 轮名字符集；旧行名单三个名字已无行仍可免出处 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:44`（另立账） |
| Q5-09 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:117`（Z6-07） | 脚本文件头重复键与提示重复标签是两条路 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（判轻列） |
| Q5-10 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:122`（Z6-12） | 说明行内容一个字不判，现算说法可挪到下一行 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:42`（P7），再由 `research/prompts/m3-defs-b9-r2-main-verification.md:44`（U4）接 |
| Q5-11 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:138`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:148`（Z7-01 ③、Z7-11） | 基准是什么；重钉进基准之后不再红 | 部分在 | `research/prompts/m3-defs-b9-r1-main-verification.md:42`（P7）写了「重钉提交进基准之后不再红」，没写基准怎么取（`gate_diff_base`） |
| Q5-12 | `research/prompts/m3-defs-b9-r1-sonnet-output.md:138`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:140`；`research/prompts/m3-defs-b9-r1-sonnet-output.md:143`（Z7-01 ①②、Z7-03、Z7-06） | 基准旧写法只比路径；空基准逐行报涨与没有基准相反；无基准时新路径放行 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（判轻列） |
| Q5-13 | `research/prompts/m3-defs-b9-r1-opus-output.md:81`；`research/prompts/m3-defs-b9-r1-opus-output.md:82`；`research/prompts/m3-defs-b9-r1-opus-output.md:93`（Z1-甲、Z1-乙、Z6-甲） | 围栏认法误遮真声明、误判嵌套示例 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:36`（P1），再由 `research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1）接 |
| Q5-14 | `research/prompts/m3-defs-b9-r1-opus-output.md:83`（Z1-丙） | 缺行出路只补已判行，与共用约束「只有已判行不起重活」相抵 | 在 | 文字一半 `research/prompts/m3-defs-b9-r1-main-verification.md:37`（P2）；钩子放而子 agent 读成相反的一半 `research/prompts/m3-defs-b9-r2-main-verification.md:46`（S2 单独已判行而理由写数） |
| Q5-15 | `research/prompts/m3-defs-b9-r1-opus-output.md:84`；`research/prompts/m3-defs-b9-r1-opus-output.md:86`（Z2-甲、Z2-丙） | 排除表锚串命中多条被判行；`${X:=8}` 不点名 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:44`（另立账） |
| Q5-16 | `research/prompts/m3-defs-b9-r1-opus-output.md:85`（Z2-乙） | 从登记位取进变量再 `\|\| X=$var` 被点名 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:38`（P3），再由 `research/prompts/m3-defs-b9-r2-main-verification.md:42`（U2）接 |
| Q5-17 | `research/prompts/m3-defs-b9-r1-opus-output.md:87`；`research/prompts/m3-defs-b9-r1-opus-output.md:88`；`research/prompts/m3-defs-b9-r1-opus-output.md:89`（Z4-甲乙丙） | 追加补记、术语改名、date-once 让原话不再恰好一次 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:40`（P5），再由 `research/prompts/m3-defs-b9-r2-main-verification.md:43`（U3）接 |
| Q5-18 | `research/prompts/m3-defs-b9-r1-opus-output.md:90`（Z4-丁） | 两份文件头仍写「不按 -r<数字> 形」 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:40`（P5） |
| Q5-19 | `research/prompts/m3-defs-b9-r1-opus-output.md:91`；`research/prompts/m3-defs-b9-r1-opus-output.md:92`（Z5-甲、Z5-乙） | date-once、term-renames 路径与 clock-times 半截私有值 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:41`（P6） |
| Q5-20 | `research/prompts/m3-defs-b9-r1-opus-output.md:94`（Z6-乙） | 规范句下一行写现算，三版都放 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:42`（P7），再由 `research/prompts/m3-defs-b9-r2-main-verification.md:44`（U4）接 |
| Q5-21 | `research/prompts/m3-defs-b9-r1-opus-output.md:95`（Z7-甲） | 基准没有表时重钉不红，与 `stage-owners.tsv` 文字相反 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:42`（P7） |
| Q5-22 | `research/prompts/m3-defs-b9-r1-opus-output.md:96`（Z7-乙） | 被钉的入口 `exec` 没被钉的编排脚本 | 在 | `research/prompts/m3-defs-b9-r1-main-verification.md:44`（另立账） |
| Q5-23 | `research/prompts/m3-defs-b9-r1-opus-output.md:348` | 同一脚本 `git mv`、sha256 不变，按「基准里没有的路径」判涨（r2 同，没单列） | 不在 | 漏列 L6 |
| Q5-24 | `research/prompts/m3-defs-b9-r2-opus-output.md:58`（S1-甲） | 列表项里关在顶格、`<details>` 里有空行，遮掉块外真声明 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:29`；`research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1） |
| Q5-25 | `research/prompts/m3-defs-b9-r2-opus-output.md:59`（S1-乙） | 定义写「代码块（照 CommonMark）」，比钩子宽 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1 的文字一半） |
| Q5-26 | `research/prompts/m3-defs-b9-r2-opus-output.md:60`（S1-丙） | 引用块关块漏 `>`、关块缩进 4 格、开四关三 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（S1-丙） |
| Q5-27 | `research/prompts/m3-defs-b9-r2-opus-output.md:61`（S1-丁） | PATH 上的 `python3` 没有 markdown-it 时退路触发 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:29`；`research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1：退回逐行认法） |
| Q5-28 | `research/prompts/m3-defs-b9-r2-opus-output.md:141`（S1 专问 `\r\n`） | 钩子 stdin 入口把回车换行提示里写对的「线程上限：31」拒掉，`--check-prompt` 放 | 不在 | 漏列 L1 |
| Q5-29 | `research/prompts/m3-defs-b9-r2-opus-output.md:151` | P1 的收益没在真提示上量过 | 不在 | 漏列 L9（观察） |
| Q5-30 | `research/prompts/m3-defs-b9-r2-opus-output.md:62`（S3-甲） | `:=`、source 进来的字面数、`${CAP:-$变量}`、除以变量四形不点名 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:31`；`research/prompts/m3-defs-b9-r2-main-verification.md:42`（U2） |
| Q5-31 | `research/prompts/m3-defs-b9-r2-opus-output.md:63`（S3-乙） | `"$(heavy_thread_default 2>/dev/null)"` 仍点名 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:42`（U2：可带 `2>/dev/null`） |
| Q5-32 | `research/prompts/m3-defs-b9-r2-opus-output.md:190` | 经自己的小函数取登记值仍点名 | 不在 | 漏列 L2 |
| Q5-33 | `research/prompts/m3-defs-b9-r2-opus-output.md:64`（S5-甲） | 真记录另一行带轮名与「」原话，〔〕写转述也收 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:33`；`research/prompts/m3-defs-b9-r2-main-verification.md:43`（U3） |
| Q5-34 | `research/prompts/m3-defs-b9-r2-opus-output.md:65`；`research/prompts/m3-defs-b9-r2-opus-output.md:242`（S5-乙，漂移加重复） | 前面补一句、后面追加抄了原话的补记 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（S5 漂移加追加） |
| Q5-35 | `research/prompts/m3-defs-b9-r2-opus-output.md:65`；`research/prompts/m3-defs-b9-r2-opus-output.md:242`（S5-乙，漂移加改字） | 前面补一句、再按改名表换词 | 部分在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46` 只点名「漂移加追加」，漂移加改字 r1、r2、r3 都不收，名字缺 → 漏列 L5 |
| Q5-36 | `research/prompts/m3-defs-b9-r2-opus-output.md:66`；`research/prompts/m3-defs-b9-r2-opus-output.md:250`；`research/prompts/m3-defs-b9-r2-opus-output.md:251`；`research/prompts/m3-defs-b9-r2-opus-output.md:253`（S6） | 私有值首尾重叠；主机名大小写；doc-text 之外 16 道门禁不打码 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（S6） |
| Q5-37 | `research/prompts/m3-defs-b9-r2-opus-output.md:253` | 没有 `multi-host.env` 的仓副本上没有词可打 | 不在，文件头写明的设计 | — |
| Q5-38 | `research/prompts/m3-defs-b9-r2-opus-output.md:67`（S7-甲） | 用户给逐台不同线程数，两种写法都跑不成 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:35`；`research/prompts/m3-defs-b9-r2-main-verification.md:44`（U4） |
| Q5-39 | `research/prompts/m3-defs-b9-r2-opus-output.md:68`；`research/prompts/m3-defs-b9-r2-opus-output.md:279`（S7-乙） | 主 agent 定义没有「一概不照」那一句 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:44`（U4） |
| Q5-40 | `research/prompts/m3-defs-b9-r2-opus-output.md:279` | 钩子判不到的那一半（子 agent 实际起几台、几线程）没有机械判法，靠自守 | 不在，设计如此 | 同 `research/prompts/m3-defs-b9-r1-sonnet-output.md:36`（Z1-01：只有定义管，源码不判），两轮判决都没单列 |
| Q5-41 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:30`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:33` | `<div>` 后紧跟围栏由放变拒；定义字面比钩子宽 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:29`；`research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1） |
| Q5-42 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:31` | 顶层围栏闭合行缩进 4 格、制表符、带 `> ` | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（S1-丙）；`research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1）；我的喂法 r3 副本退 2、r1 与 r2 副本退 0 |
| Q5-43 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:32` | 库载不进时一行都不遮，比 b9 更严 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:29`；`research/prompts/m3-defs-b9-r2-main-verification.md:41`（U1） |
| Q5-44 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:130`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:131` | `python3-markdown-it` 是传递依赖、仓里没声明；放行时 stderr 不回给模型 | 不在 | 漏列 L7 |
| Q5-45 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:41` | S2 的另一半：单独已判行而理由里写了数 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`（S2 单独已判行而理由写数） |
| Q5-46 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:140` | S2 其余三形：N 等于登记值加写了别的数的已判行、N 在代码块里加外面的已判行、N 带括注加已判行 | 不在 | 漏列 L8 |
| Q5-47 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:50`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:158` | S3 盲区七份（`(( x = 8 ))`、`eval`、`read`、`printf -v`、`for x in 8`、`x+=0`、source 进来的文件） | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:31`；`research/prompts/m3-defs-b9-r2-main-verification.md:42`（U2）；r3 副本七份都点名 |
| Q5-48 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:149`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:152` | S3 语义洞三份（函数没调用、赋在使用之后、赋在子 shell）；先字面后登记位误报 | 不在 | 漏列 L3 |
| Q5-49 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:162`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:164`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:165` | S4 两类撞：light 登记的副本正文被改成起重活、heavy 登记的自证在副本里带 `=commit` | 不在 | 漏列 L4 |
| Q5-50 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:72`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:183` | S5 弱点：提示行恰好是另一句带轮名与「」的话 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:33`；`research/prompts/m3-defs-b9-r2-main-verification.md:43`（U3） |
| Q5-51 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:194` | doc-text 里三处没经打码的输出（参数、仓根、登记库路径与 OSError 文字） | 不在 | 漏列 L10 |
| Q5-52 | `research/prompts/m3-defs-b9-r2-sonnet-output.md:104` | 第一轮正推「文字没写到」里没进改法的：Z1-04、Z4-01、Z6-07、Z7-01 其余两处、Z7-06 | 在 | `research/prompts/m3-defs-b9-r2-main-verification.md:46`：「辩方「判轻」那一行列的第一轮正推 Z1-04、Z4-01、Z6-07、Z7-01 其余两处、Z7-06」 |

### 漏列：两轮判决都没写去向的线（L1–L10），在 r1、r3 副本上现喂

命令都在 `research/prompts/m3-defs-b9-r3-sonnet-model/out/q5-repro.txt` 的同名段里，原样输出在那里；「改前也中」指 r1 副本（P 改法之前）上同一喂法给出同样的结果。

| 号 | 线 | 出处 | 现喂（`out/q5-repro.txt` 的段） | r3 副本上的结果 | 改前也中 | 分量；什么现象会推翻 |
|---|---|---|---|---|---|---|
| L1 | 回车换行的派发提示：`--check-prompt` 放，钩子 stdin 入口拒 | Q5-28（`research/prompts/m3-defs-b9-r2-opus-output.md:141`） | 段 Q5-1：同一份对的提示写成 LF 与 CRLF，各喂两个入口，r1 r2 r3 三份副本 | r3：LF 两入口都退 0；CRLF `--check-prompt` 退 0、钩子 stdin 退 2，说写了「线程上限：31」后面接了别的话 | 是（r1、r2 同） | 轻：主 agent 写 LF 碰不上；但 `dispatch-prompt.py --check` 与钩子对同一份提示说不同的话。钩子读 stdin 时去掉 CR 后放行，或提示文件一律 LF，会推翻 |
| L2 | 经自己的小函数取登记值，`\|\| X=$变量` 仍被点名 | Q5-32（`research/prompts/m3-defs-b9-r2-opus-output.md:190`） | 段 Q5-3：r2 攻方的 `make_tree.sh` 样本对 r1、r3 副本的格 | r3 点名 6 处，含 `via_own_function.sh` 第 5 行；r1 点名 8 处，也含它 | 是 | 轻：误红，改成直接 `"$(heavy_thread_default)"` 就绿；违反不了用户定案。样本改后不点名会推翻 |
| L3 | S3 判法的三个语义洞与一个误报 | Q5-48（`research/prompts/m3-defs-b9-r2-sonnet-output.md:149`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:152`） | 段 Q5-2：辩方 29 份样本对 r1、r2、r3 副本，只列预期为盲区、语义洞、误报的样本 | r3：`s3_func_never_called`、`s3_assigned_after_use`、`s3_subshell_assign` 不点名（预期「变量到使用处是空的」）；`s3_twice_literal_then_registry` 点名（预期误报，最终值是登记值） | 语义洞三份：否（r1 副本点名，P3 之后不点名）；误报：是 | 轻：语义洞不是钉数，辩方判「不是钉数」；误报是误红。这三份在 r3 副本点名会推翻「语义洞」，误报样本不点名会推翻误报 |
| L4 | S4：登记为 light 的自证副本正文被改成起重活，主 agent 带 `=commit` 照放；登记为 heavy 的自证带 `=commit` 或 `=user-request` 在副本里放 | Q5-49（`research/prompts/m3-defs-b9-r2-sonnet-output.md:162`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:164`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:165`） | 段 Q5-4：辩方 `s4_heavy_guard_feed.py` 对 r1、r3 副本喂钩子 stdin 入口 | r3：B1、C2、C3 退 0；子 agent 自加前缀的 A1、A2 退 2；不带前缀的 B2、C1 退 2 | 是（r1 同） | 中：钩子不读副本正文，也分不出这条命令按哪一行办，判官只剩主 agent 读交回报告；子 agent 一侧仍拒，不涉「仓副本自证收回放宽」。钩子读副本正文或拒 C2、C3 会推翻 |
| L5 | S5：前面补一句再按改名表换词，叫停轮翻回「开着」 | Q5-35（`research/prompts/m3-defs-b9-r2-opus-output.md:65`；`research/prompts/m3-defs-b9-r2-opus-output.md:242`） | 段 Q5-5：用改名表里真有的一对（第一个事务 → 新池新建文件）造的小树，有表与无表各六种改动，r1 r2 r3 | r3 有表：漂移加改字不收（r1、r2 也不收）；改字单做、重复加改字收；漂移加重复不收 | 是（r1、r2 也不收） | 中：「补一句再跑术语改名」是日常组合，不收的后果是 ⑥ 与门禁 abandoned-rounds 翻回开着；`research/prompts/m3-defs-b9-r2-main-verification.md:46` 只写了「漂移加追加」。把名字补进另立账即可；要修需让行号提示在漂移后重定位 |
| L6 | 豁免表里被钉的脚本挪位置（内容不变），exempt-check 带基准判涨 | Q5-23（`research/prompts/m3-defs-b9-r1-opus-output.md:348`） | 段 Q5-6：把表里第一行脚本挪进 `moved/` 子目录、表里路径跟着改、sha256 不变，带原表当基准 | r1、r3 副本都退 1，点名 `…/moved/run_segment_one_…` 那一行为涨 | 是 | 中：门禁自己的说明 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/gate.d/code-tooling.sh:126`：「路径搬了就改」，只缩不涨却把改了的路径当新增判红；`.claude/rules/path-moves.md` 的搬迁是仓里常事。基准比较认出同 sha256 的搬迁会推翻 |
| L7 | `python3-markdown-it` 是传递依赖，仓里没声明 | Q5-44（`research/prompts/m3-defs-b9-r2-sonnet-output.md:130`；`research/prompts/m3-defs-b9-r2-sonnet-output.md:131`） | 段 Q5-8：grep r3 副本、`apt-mark showmanual`、`dpkg -l` | 只有 `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/hooks/runner-dispatch-guard.sh:1174`：「markdown_it = importlib.import_module」提到它；手装名单里带 markdown 的 0 行；已装 3.0.0-2 | P1 带进来（r1 副本没用这个库） | 轻：U1 把库载不进的退路改成逐行认法，不再一行都不遮，危害降到误拒或误放的边角；依赖声明缺口还在 |
| L8 | S2 钩子放而按定义读出来相反的三形 | Q5-46（`research/prompts/m3-defs-b9-r2-sonnet-output.md:140`） | 段 Q5-7：辩方 `s2_exit_matrix.py` 对 r1、r3 小树，15 个形态 | 15 个形态两副本逐行相同；「N 等于登记值加已判行里写了别的数」「N 不等于登记值写在代码块里加已判行在外」「N 带括注加已判行」「两行线程上限加已判行」都放 | 是 | 轻：子 agent 停下交回或按登记值跑，白派一次，不违反用户定案。钩子拒这些形态会推翻 |
| L9 | P1 在真提示上一次没起过作用 | Q5-29（`research/prompts/m3-defs-b9-r2-opus-output.md:151`） | `out/q6-counts.tsv` 的 `G-prompts-now` | 工作根下 54 份 `prompt.md`，含围栏行 0 份 | — | 观察：不是错；只说明 P1 的收益没在真提示上量过。出现带围栏行的真提示再量会改写这一条 |
| L10 | doc-text 里三处没经打码的输出 | Q5-51（`research/prompts/m3-defs-b9-r2-sonnet-output.md:194`） | 段 Q5-9：r3 副本 `doc-text.sh` 的三处 | `/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/gate.d/doc-text.sh:329`：「认不出的选项」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/gate.d/doc-text.sh:336`：「进不去项目根」，`/home/fy5090/.local/share/singlefs/work/m3-defs-b9-r3/repo/.claude/gate.d/doc-text.sh:350`：「读不到禁用写法的登记」，三处仍原样打出调用参数、仓根、登记库路径与 OSError 文字 | 是 | 轻：三处都不带仓内值，本机仓根路径进输出；辩方判「值本身不在这几条里」。这三条里放进私有值才会推翻 |

### 第二张表：上一题三轮判决的另立账（照旧，只列不核是否仍中）

`research/prompts/_m3-defs-b9-r3-body.md:28` 要求把它们并进清单。下面只抄出处与一句话，不判现在还中不中；其中 F-Y3 已在批 9 落地（`records/2026-10-12-里程碑三工具与治理批.md:151`），P7、U4 接了「现算说法挪到下一行」。

| 号 | 出处 | 线 | 现状 |
|---|---|---|---|
| C-01 | `research/prompts/m3-defs-b6-r1-main-verification.md:41` | V4 名单手写、没有检查与共用库绑定 | 开着（`research/prompts/m3-defs-b6-r3-main-verification.md:44` 并进「手写名单没有检查绑定」） |
| C-02 | `research/prompts/m3-defs-b6-r1-main-verification.md:41` | V5 甲：副本里 import 的库被改，闸看不到 | 开着 |
| C-03 | `research/prompts/m3-defs-b6-r1-main-verification.md:41` | 三种新类型多数派发不带重活却要写线程上限，出路是已判行 | 开着；L8、Q5-14 同一类 |
| C-04 | `research/prompts/m3-defs-b6-r2-main-verification.md:41` | X3 甲（`env -S` 里 `${…}` 展开）、X3 乙（`-p` 的值在变量里） | 开着 |
| C-05 | `research/prompts/m3-defs-b6-r2-main-verification.md:41` | V8 甲：文件头读法与正文行为不一致，静态看不出 | 开着；Q5-40 同一类 |
| C-06 | `research/prompts/m3-defs-b6-r2-main-verification.md:41` | X1 乙：代码块示例行 | 由 P1、U1 接 |
| C-07 | `research/prompts/m3-defs-b6-r3-main-verification.md:59` | Y1-丙 的 ⑥ 一侧 | 开着；P5、U3 改了库的取法，⑥ 一侧是否同时好了未核 |
| C-08 | `research/prompts/m3-defs-b6-r3-main-verification.md:59` | Y2-甲：几层 `env -C` 合一 | 开着 |
| C-09 | `research/prompts/m3-defs-b6-r3-main-verification.md:59`；`research/prompts/m3-defs-b6-r3-main-verification.md:39` | Y2-乙 与 Y5 第 5 条：heavy-thread-defaults 九种漏 | 开着；L2、L3 同一格 |
| C-10 | `research/prompts/m3-defs-b6-r3-main-verification.md:59` | Y3-甲、Y3-乙 | F-Y3 已在批 9 落地（`records/2026-10-12-里程碑三工具与治理批.md:151`） |
| C-11 | `research/prompts/m3-defs-b6-r3-main-verification.md:35`；`research/prompts/m3-defs-b6-r3-main-verification.md:41` | Y5 第 1 条「内存上限」定义要而闸不判；第 7 条未跟踪 1 GiB 单行文件整份读 | 开着 |
| C-12 | `research/prompts/m3-defs-b6-r3-main-verification.md:44` | 轮名全集三个小漏、旧行名单与登记表没有检查绑定 | 开着；Q5-08 同一类 |
| C-13 | `research/prompts/m3-defs-b6-r3-main-verification.md:37` | E171 六份入口与编排按现算（随 V8 甲） | 开着；Q5-22 同一类 |
| C-14 | `research/prompts/m3-defs-b6-r3-main-verification.md:59` | 攻方顺手记：「登记值的一半」挪到「多机：」下一行照放；M2 排除表 89 行对第二轮 66 行，基准没表时不比 | 前一条由 P7、U4 接；后一条随 Q6-41 |

### Q5 小结

- 52 条线（第一张表）：在 38；部分在 2（Q5-11 基准怎么取没写、Q5-35 漂移加改字名字缺）；判决里不在但调度记录里在 1（Q5-01）；不在且是设计 2（Q5-37、Q5-40）；不在 9（Q5-23、28、29、32、44、46、48、49、51）。
- 漏列线 L1–L10：九条「不在」加一条「部分在」（L5）。其中 L4、L5、L6 分量最重：L4 是钩子分不出副本里的 heavy 自证按哪一行办，L5 是「补一句再跑术语改名」让叫停轮翻回开着，L6 是搬迁被钉脚本被只缩不涨判涨且与门禁自己的说明相抵。其余是误红、白派一次或观察。
- 这些线都是改前（r1 副本）就中的，除 L3 的三份语义洞（P3 带进来，r1 副本点名）和 L7（P1 带进来）；按跑前条款「打中也打中 P 改法之前 ⇒ 另立一笔账」，是补进另立账、不是否改法。
- 第二张表 14 行是上一题三轮判决的另立账，原样列出，没核现在还中不中。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Q6 数字能不能核 | 48 个数：对 40、部分对 3、对不上 4、复核不了 1 | 对不上四处：Q6-06 判决写的 cite-check 命令今天只对上 91 处、要加仓根才 117；Q6-14「正推模型全 same」实为 8/9；Q6-19 三道样本档 46 对 r3 副本该是 47；Q6-27「批 11 加的样本 0 处在红项」照命令算是 2 处。其余数现算都复现 |
| Q5 清单完整性 | 52 条线：在 38、部分在 2、不在 12（含设计 2、调度记录里在 1） | 漏列 L1–L10：回车换行入口不一致、经函数取登记值仍点名、S3 语义洞与误报、S4 副本自证两类撞、漂移加改字、搬迁被钉脚本判涨、markdown-it 依赖没声明、S2 三形、P1 收益未量、doc-text 三处未打码；命令与原样输出在 `out/q5-repro.txt` |
| 校验路径 | 两条路 | Q6 的数各用两条独立路复现的有：46／22／27（日志末行与样本目录数）、458／79（辩方工具与我另写的判法）、832（抽出的格与主 agent 日志）、verify 汇总（summary 行数与 `.rc` 文件数）；只有一条路的是 cite-check 引文数与补丁规模（单次命令） |

## 要主 agent 跑

- Q6-23：verify13 里 `brk-long-run-exempt-path-unchecked`、`-growth-unchecked`、`-repin-unchecked` 三项判对（红了），verify12 与 verify14 同三项判错（没红）。三份 `verify1*.sh` 第 23 行命令逐字相同。按那一行的 `redany` 对这三个模式各在 after-b10-fixup 与 after-b11 两个标签上重跑一遍，看 `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh … --item code-tooling.sh` 在哪个状态下红、为什么；我起不了门禁格自证。
- Q6-24 后半：在 after-b9-fixup 的工作树上跑 `bash .claude/gate.d/doc-text.sh --check clock-times,term-renames`，看是不是 15 行、6 份、10 条；那棵树我这里没有。

## 没做什么

- 没碰 Q1–Q4，没替主 agent 采纳或出判决；L1–L10 要不要进另立账、Q6 对不上的四处怎么改，由主 agent 定。
- 没起门禁格（`--check`）、没跑任何登记的 `--selftest`、没编译、没跑重型测试。`tools/rerun.sh --selftest` 在本模型目录和前两轮各腿的模型目录里都红，因为 `rerun_steps` 换成了各腿自己的步骤，模板自证检查的是模板自带那一段。
- 第二张表（上一题另立账）只列出处，没核现在还中不中；U1–U4 是零轮形态，本腿没为它们开攻，L2、L5 是 U2、U3 之后的实况，它们改前（r1 副本）也中，不是 U 改法带进来的。
- 补丁行数与 sha、verify 汇总里的错项、集成副本的 git 差异读的是现成产物，没有重跑 `verify1*.sh` 本身。
- 随时间变的数（`G-prompts-now`、三份 `A-r*-kb`）只是今天的值。
- 前两轮各腿模型目录的复跑（`out/other-models-rerun.txt`）是本腿另跑的，不在本模型目录的 `rerun.sh` 里；跑法是各模型目录自己的 `tools/rerun.sh`，`RERUN_PARTS` 分几次。
- 草稿目录里的冻结副本拷贝和小树（几份 69M 的 `small-r*`、各趟 `run*`、`rr-*`）交回前删掉；留下 `prompt.md`、`seg/`（报告分段原稿）与 `tmp/` 里的日志。

## 复跑与核对

```
cd /home/fy5090/code/singlefs
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-defs-b9-r3-sonnet-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-defs-b9-r3/repo --work <新的草稿目录>
```

- 全跑约 1 分钟，输出落 `<草稿目录>/out/`，路径换占位后与 `research/prompts/m3-defs-b9-r3-sonnet-model/out/` 同名文件逐行比，末尾打 same / DIFF / 新出。最后一次从仓里的模型目录复跑：`q6-counts.tsv`、`q5-repro.txt` 都是 same。
- 模型目录每个文件的 sha256 在 `research/prompts/m3-defs-b9-r3-sonnet-model/SHA256SUMS`。
- 前两轮各腿模型的复跑：`out/other-models-rerun.txt`（r1 攻方 16 same＋`z7.txt` DIFF；r1 正推 8 逐行相同＋`z6-script-header.tsv` 不同；r2 攻方 9 same；r2 辩方 47 same）。

## cite-check 末行

`python3 scripts/agents/cite-check.py research/prompts/m3-defs-b9-r3-sonnet-output.md --root /home/fy5090/code/singlefs` 的末行：

```
  ✓ 核了 13 处引文、250 处裸行号，对上 13 处，没判 250 处（其中裸行号没有引文可比 250 处；逐处列在上面）
```
