# 规则压缩复核第三轮（verification）正推腿报告

日期：2026-11-16。轮名 `rules-compress-verification-r3`，第三轮（最后一轮），正推腿。草稿目录 `~/.local/share/singlefs/work/rules-compress-verification-r3-sonnet`。

## 先说清的几件事

- 「成立 / 不成立 / 推不出」的读法：三格问的都是「修补对 / 没引出新问题 / 能换上」，成立 = 这一问答「是」，不成立 = 找到了一处以上的问题。
- 核对的对象：`research/prompts/rules-compress-r3-merged/diff-from-r2.patch` 里落在 `dot-claude/rules/` 的 `implementation-workflow.md`、`three-way-inference.md`、`verification.md`、`verification-reference.md` 的全部 hunk（下面编号 1 到 27，6 号拆成 6a 到 6e）；技能 three-way 在这份补丁里没有 hunk（补丁里没有 `dot-claude/skills/three-way/`），所以没有逐处改动可核，只核了别的 hunk 指到它的节名在不在。
- 被判文件的 sha256 对上：仓里原件三份（`judged-sha256.txt`）与合稿树四份（`merged-tree-sha256.txt`）现读相同；`verification-reference.md` 在仓里没有原件（它是合稿新建的参考件，内容取自原件 `.claude/rules/verification.md` 与 `.claude/rules/implementation-workflow.md` 里挪出去的段），所以它的「原件」栏写原件里对应的那一段。
- 「改前」是第二版：我把合稿树拷进草稿目录、对着 `diff-from-r2.patch` 反向打补丁（`patch -R -p1`）还出第二版，再逐行 difflib 比（草稿目录 `r2/`、`r3/`，交回前删，见末尾）；还出来的第二版与补丁里的 `-` 行一致。
- 误读声明：我在全仓 `grep` 找「开不开下一轮」时，输出里带出了禁读文件 `research/prompts/rules-compress-verification-r3-opus-output.md` 的一行（第 142 行，列的是它核过的几个节名指针）。我没有读那份文件，这一行的内容没有用在下面任何一条里；下面的指针核对都是我自己对着合稿树查的。
- 没跑项目代码、没交任务；只读命令与 `python3 -I` 的文本比对。

## 复核用的材料位置（后面的引文都是 文件:行）

- 用户定案：`records/2026-11-16-三方豁免取消与存量补轮.md` 编号 2（第 119 行）、4（第 121 行）、17（第 169 行）、23（第 211 行）、24（第 212 行）、26（第 214 行）。
- 第二轮判决：`research/prompts/rules-compress-verification-r2-main-verification.md`（「三、改法」第 28 行）。
- 矛盾总表第三版 `research/prompts/rules-compress-r3-merged/contradictions.md`；去向表 `dispositions-r2.md`。

## 逐处核对表（落点简写：IW=implementation-workflow.md，TWI=three-way-inference.md，V=verification.md，VR=verification-reference.md，行号是第三版合稿树里的）

引文与依据放在后面「证据」一节，表里只写编号。

| 号 | 落点 | 改前（第二版） | 改后（第三版） | 依据 | 对不对 |
|---|---|---|---|---|---|
| 1 | IW 第 9 行 步骤 1 | 变异清单后面括号写 `crates/` 的进 `crates/mutations.tsv`、实验装置的进 `scripts/mutation/tables/` | 括号删，改指 `show-me-test-by-review.md`「新增的测试必须先证明它会红」 | 定案第 4 条（两样都要）；指到的那一节第 7 行有这两张表 | 对（证据 E1）|
| 2 | IW 第 10 行 步骤 2 | 一整段「代码轮的轮数不照先挂起、三轮多数……最多三轮」 | 换成「轮数与收法照「代码轮与治理文件轮怎么收」一节」 | 定案第 2、17、23 条；被挪的话都在新节里 | 对（挪法）；新节多出来的话见 6b 到 6e（E2）|
| 3 | IW 第 19 行 | 「改它们与改 crates 同规矩：写完要走一轮三方」 | 加括号「轮数与收法照……一节」 | 定案第 23 条「照代码轮收」 | 对（E3）|
| 4 | IW 第 21 行 治理文件同规矩 | 「按批走一轮三方（main-agent 第 4 条）」 | 括号里加「轮数与收法照……一节」 | 定案第 23 条 | 对（E3）|
| 5 | IW 第 23 行 | 括号写法 `show-me-test.md`（没有测试的 patch 一律不收） | 改回「没有测试的 patch 一律不收」带引号；末尾「只改文档和脚本除外」仍没有（第二版就没有） | 原件 IW 第 23 行；上游节名在 | 引号对；少的半句见清单 D（E4）|
| 6a | IW 第 25、27 行 新节头段 | 无（第二版散在步骤 2 与三方规则两处） | 代码轮与治理文件轮同一套收，不用先挂起、三轮多数；那一套只管设计推论轮，括号写了定义 | 定案第 17、23 条 | 对（E5）|
| 6b | IW 第 29 行 | 无 | 「第一轮的停法照同一份「开不开下一轮」那一段：两条腿都没打中就记没打中、这一题停」 | 定案第 2、26 条 | 对；第三处写同一件事（格二）（E6）|
| 6c | IW 第 30 行 | 步骤 2 里「现查坐实、做成会红的测试后写回代码，改完再攻一轮，那一轮没新打中就收，最多三轮」 | 同意思；另多了「治理文件照条款改」「那一轮只攻上一轮的修补」「开了第二轮的，在第二轮派一条辩方腿复核已有的判决」 | 定案第 17、23 条只有前一半 | 前一半对；后三半句定案没说（清单 A、B、C）（E7）|
| 6d | IW 第 31 行 | 无 | 「现查推翻了打中的那一条：不改，也不为它开一轮；判决里写明推翻的依据」 | 定案第 17 条的反面；三方规则「判决里逐条写去向（现查坐实而采、现查推翻而不采）」 | 推得出，可留（清单 E）（E8）|
| 6e | IW 第 32 行 | 无 | 「第三轮又打中的：现查坐实的改了就收，写进判决的交用户表，不开第四轮」 | 第二轮判决「三、改法」那一句，不是用户定案 | 做法与判决一致；依据标成用户定案是错的（清单 D2）（E9）|
| 7 | IW 第 47 行 | 括号写法（门禁不许假装通过） | 「门禁不许假装通过」带引号 | 原件 IW 第 50 行 | 对（E4）|
| 8 | TWI 第 43 行 | 第二版开头「多轮这一套只管设计推论轮；代码轮的轮数照 IW 步骤 2」，没有定义 | 新段：写了「设计推论轮」的段只管它，给出定义；代码轮与治理文件轮照 IW 新节；没写的段管三类轮 | 定案第 17、23 条 | 对；定义与 IW 第 27 行各写一份（格二）（E10）|
| 9 | TWI 第 45 行 开不开下一轮 | 「第一轮两条腿结论一致、没打中，就停……停是不开下一轮，不是已经能拿这个没打中去支撑别的结论」；辩方「其中至少一轮」 | 两条腿各是一次独立抽样，都没打中记没打中、这一题停；辩方「第二轮与第三轮里至少一轮（只开到第二轮的，在第二轮派）」 | 定案第 2、26 条；原件「其中至少一轮派一条辩方腿」 | 对（E11）|
| 10 | TWI 第 47 行 | 「第三轮之后停，不开第四轮。」 | 标题后加「（设计推论轮）」，正文不变 | 定案第 2 条「第三轮之后不再开」 | 对（E12）|
| 11 | TWI 第 72 行 | 「拿一条腿的没打中支撑任何结论之前，用同一份提示再派一条同立场腿……第一轮没打中就停不受两次都没打中才记拦」 | 一轮的正推（或辩方）腿与攻方腿各是一次独立抽样，两条腿都没打中记没打中、不再派同立场腿；只有一条腿覆盖到的范围才再派 | 定案第 26 条 | 对（E13）|
| 12 | V 第 23 行 崩溃枚举用例 | 「验证包里导入崩溃引擎模块的测试文件第一行声明它测哪几个模块」 | 删「里」 | 脚本 `crash-case-check.py` 文件头 modules 一行 | 对（E14）|
| 13 | V 第 25 行 判器调用点 | 「两样都不是的判红（……判法 cpu-judging-sites-check.py）」 | 「合规的位置不止这两样，有一样成立就行，列在判法文件头「一处调用点算合规，有一样成立就行」那一段，一样都不成立的判红」 | 判法脚本文件头 ①到⑤ 五样 | 对；原件只写两样，是照脚本改（E15）|
| 14 | V 第 38 行 提交之外 | 「提交之外要跑……一律先弹窗问用户，同意了才跑。提交时那一趟的确认就是带 --full-gate 的那一次弹窗」 | 带两个标识的由钩子弹的那一次就是确认，不带的主 agent 自己弹窗问一次 | 定案第 24 条；钩子 `run-scope-guard.sh` 文件头「放宽检查的提交标识」 | 对（E16）|
| 15 | V 第 67 行 | 「是那条测试的事，不在「环境相关的一切只由多机配置定」之内」 | 「是那条测试的事，与多机无关」 | 原件 V 第 78 行原话 | 对，是原件原话（E17）|
| 16 | VR 第 21 行 要上卡的三处 | 「领卡被拒的那一格判红、那一趟不起」 | 「领卡没成的各自怎么办在文件头同一段」，列三种：崩溃枚举用例录好的判红、一条都不判；对拍那一格判红、那一趟不起；挂特性那一趟记本次未跑、这一格判红、排在最后的一趟不再起 | `crash-point-replay.sh` 文件头「显卡与清场」三处 | 对（E18）|
| 17 | VR 第 42 行 | 「lkmm 每份 litmus」一个单位 | 「每份 litmus 两条（herd7 与绑定各一条，绑定那一条的键另含锚点指到的函数单元指纹）」 | `lkmm.sh` 文件头「复用」 | 对；原件是「每份 litmus」，照脚本改（E19）|
| 18 | VR 第 81 行 测试线程 | 「门禁 crash-point-replay 带 gpu 特性那一趟与产品 GPU 真显卡那一趟」 | 「门禁 crash-point-replay 各趟」 | `crash-point-replay.sh` 文件头第 256 行、`admission.py` 第 3439 行 | 对（E20）|
| 19 | VR 第 138 行 | 「证明整格、工具链、钉全各一项」 | 「`toolchain`、`proof`（整格）、`table` 各一项」 | `formal-verification.sh` 文件头第 51 行 | 对（E21）|
| 20 | VR 第 146 行 门禁管哪一半 | 「挂特性的验证包测试文件在不指定范围那一趟至少编进来一次、跑到或复用的用例不为 0」 | 「挂特性的验证包测试文件逐个跑到或复用的用例（跑到加复用 0 条与每处 NOT_RUN_NO_GPU_ADAPTER 都报本次未跑、不判红，整道照样退 0）」 | `crash-point-replay.sh` 第 249、223 行；第 17 行格名表写「不为 0」，与第 249 行不一致，这里照后一段 | 对（照第 249、223 行）；读起来有点怪（格二）（E22）|
| 21 | VR 第 148 行 形式化验证 | 「E165 的红例都钉进钉表、……」 | 钉表只判成形与重放，「红例都钉进钉表由 E165 第二段驱动写账时当场判，不归这道门禁」；加「不一致事件表……0 次即判红」 | `formal-verification.sh` 第 35、23 行 | 对（E23）|
| 22 | VR 第 149 行 广谱变异 | 「不指定范围那一趟这一格自己先起驱动剪枝跑」 | 经服务分片起的那一趟先起驱动；`gate.sh` 整轮、手跑与 `--check gate-verdict` 不起驱动、只读库 | `broad-mutation.sh` 第 16、30 行 | 对（E24）|
| 23 | VR 第 151 行 | 「验证包测试文件照 crash-case-check.py 的 modules 那一样声明模块」 | 「验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块」 | `crash-case-check.py` 第 7 行 | 对（同 12）|
| 24 | VR 第 154 行 | 「落在判定段里，或包住它的函数带标记；都不是的判红」 | 列五样（判定段、判器本体、标记、登记表 excluded、判器自测），一样都不成立的判红 | 判法脚本文件头 ①到⑤ | 对（E15）|
| 25 | VR 第 158 行 qemu | 「两块 virtio 盘上跑固定负载，设备侧录制与程序录制流逐项比」 | 「六个模式各档负载跑完，设备侧独立录制与程序录制流逐项相同，漏一道屏障与走页缓存两个对照红在该红的地方」 | `qemu-device-streams.sh` 第 5、12、19 行 | 对（E25）|
| 26 | VR 第 165 行 | 「格名表以 formal-verification.sh 文件头为准」 | 加「（七格）」 | 第 16 行格名表七格，第 17 到 23 行七条 `gate-cell` | 对（E21）|
| 27 | VR 第 170 行 | 无 | 新增 same-origin-counter 一条 | `formal-verification.sh` 第 23 行 | 对（E23）|

## 证据（文件:行 与原话；合稿树路径简写 `M/` = `research/prompts/rules-compress-r3-merged/dot-claude/rules/`，引文里写全路径）

- E1（号 1）：「会不会红写进变异清单与 commit message，两样都要」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:9`）；指到的那一节有被挪走的两张表：「下装置的进 scripts/mutation/tables/ 下它那一张表」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/show-me-test-by-review.md:7`）；定案：「两样都要」（`records/2026-11-16-三方豁免取消与存量补轮.md:121`）。
- E2（号 2）：改后「轮数与收法照…怎么收…一节」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:10`）。
- E3（号 3、4）：「轮数与收法照…一节」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:19`）与（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:21`）；定案：「照代码轮收」（`records/2026-11-16-三方豁免取消与存量补轮.md:211`）。
- E4（号 5、7）：原件「没有测试的 patch 一律不收」「只改文档和脚本除外」（`.claude/rules/implementation-workflow.md:23`）；第三版「没有测试的 patch 一律不收」「只管 crates/*/src/」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:23`）；上游节名在：「没有测试的 patch 一律不收」（`.claude/singlefs-ai-sop/rules/show-me-test.md:12`）、「门禁不许假装通过」（`.claude/singlefs-ai-sop/rules/show-me-test.md:27`）；原件「门禁不许假装通过」（`.claude/rules/implementation-workflow.md:50`），第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:47`）「门禁不许假装通过」。
- E5（号 6a）：「不用…先挂起、三轮多数…那一套只管设计推论轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:27`）；定案：「只管设计推论轮」（`records/2026-11-16-三方豁免取消与存量补轮.md:169`）。
- E6（号 6b）：「两条腿都没打中就记没打中、这一题停，不开第二轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:29`）；同一句话还在「两条腿结论一致、都没打中，就记没打中、这一题停，不开第二轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`）与「两条腿都没打中，就记」「这一题停」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:72`）；定案：「第一轮两条腿一致、没打中就停」（`records/2026-11-16-三方豁免取消与存量补轮.md:119`），「两条腿都没打中就记没打中、这一题停」（`records/2026-11-16-三方豁免取消与存量补轮.md:214`）。定案第 2 条还写了「写成一处、另一处指过去」（`records/2026-11-16-三方豁免取消与存量补轮.md:119`）。
- E7（号 6c）：第三版「坐实的就改」「治理文件照条款改」「那一轮只攻上一轮的修补」「在第二轮派一条辩方腿复核已有的判决」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:30`）。定案只有「改完再攻一轮，那一轮没新打中就收，最多三轮」（`records/2026-11-16-三方豁免取消与存量补轮.md:169`）与「现查坐实就改，改完再攻一轮，没新打中就收，最多三轮」（`records/2026-11-16-三方豁免取消与存量补轮.md:211`）。第二轮真做的是「判据四格：修补对不对、修补引出的新问题、第一轮没看到的、不采纳的那几条」（`records/2026-11-16-三方豁免取消与存量补轮.md:206`）；三方规则给设计推论轮的第二轮写的是「第二轮第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`）。总表把辩方那句记成定案第 23 条：「用户定案第 23 条：开了第二轮的在第二轮派辩方腿」（`research/prompts/rules-compress-r3-merged/contradictions.md:117`），去向表同（「用户定 23：开了第二轮的在第二轮派辩方腿」，`research/prompts/rules-compress-r3-merged/dispositions-r2.md:46`），而第 211 行没有这句。
- E8（号 6d）：「现查推翻了打中的那一条：不改，也不为它开一轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:31`）；三方规则本来就有「现查推翻而不采」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:73`）。
- E9（号 6e）：「现查坐实的改了就收，写进判决的交用户表，不开第四轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:32`）；出处是主 agent 第二轮判决：「又打中的改了收、写进交用户表，不开第四轮」（`research/prompts/rules-compress-verification-r2-main-verification.md:28`），不是调度记录里的用户定案。
- E10（号 8）：「设计判断、取舍，拿依据（包括实验结论）去推翻或确立某一条决策的那一类轮」「不是设计推论轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:43`）；另一份定义「设计判断、取舍、推翻或确立决策的那一类轮」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:27`）；它说的推论在「适用范围」一节：「推论指设计判断、取舍结论」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:5`）。
- E11（号 9）：「第二轮与第三轮里至少一轮派一条辩方腿复核已有的判决（只开到第二轮的，在第二轮派）」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:45`）；原件：「其中至少一轮派一条辩方腿去复核已有的判决」（`.claude/rules/three-way-inference.md:120`）。
- E12（号 10）：「第三轮之后停，不开第四轮（设计推论轮）」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:47`）；定案：「第三轮之后不再开」（`records/2026-11-16-三方豁免取消与存量补轮.md:119`）。
- E13（号 11）：「只有一条腿覆盖到的范围（分工只分给其中一条的格、只有一条读了的文件）」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/three-way-inference.md:72`）；定案：「只有一条腿覆盖到的范围」（`records/2026-11-16-三方豁免取消与存量补轮.md:214`）；原件：「至少再抽一次」（`.claude/rules/three-way-inference.md:158`）。
- E14（号 12、23）：脚本文件头「验证包导入崩溃引擎模块的测试文件第一行声明它测哪几个模块」（`scripts/crash/crash-case-check.py:7`）；第三版同一句（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:23`）与参考件（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:151`）。
- E15（号 13、24）：判法文件头「一处调用点算合规，有一样成立就行：」（`scripts/crash/cpu-judging-sites-check.py:15`）列五样，最后一样「落在判器自测里」（`scripts/crash/cpu-judging-sites-check.py:22`）；第三版「合规的位置不止这两样，有一样成立就行」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:25`）、参考件「落在判器自测里」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:154`）。
- E16（号 14）：定案：「命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问一次」（`records/2026-11-16-三方豁免取消与存量补轮.md:212`）；第三版：「命令不带这两个标记、钩子不弹的，主 agent 自己弹窗问用户一次」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:38`）；钩子头：「放宽检查的提交标识（lib_run_scope.py 的 submit_flag_asks）」（`.claude/hooks/run-scope-guard.sh:33`），「之前带这两个标识之一的，钩子不放行也不拒」（`.claude/hooks/run-scope-guard.sh:34`）。要注意：不指定范围的六道门禁与崩溃放量入口不带标识，钩子不是「不弹」而是「执行前拒」：「都没带的，执行前拒，说明写出路」（`.claude/hooks/run-scope-guard.sh:45`）。
- E17（号 15）：原件「是那条测试的事，与多机无关」（`.claude/rules/verification.md:78`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification.md:67`）同。
- E18（号 16）：文件头三处：「领卡被拒，录好的那几条判红、一条都不判」（`.claude/gate.d/crash-point-replay.sh:340`）；「领卡被拒那一格判红、那一趟不起」（`.claude/gate.d/crash-point-replay.sh:351`）；「领卡没成的测试目标记本次未跑、这一格判红」（`.claude/gate.d/crash-point-replay.sh:255`），「排在最后的那一趟不再起、它的目标记本次未跑，这一格判红」（`.claude/gate.d/crash-point-replay.sh:359`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:21`）三条各对得上。
- E19（号 17）：「每份 litmus 两条判定」（`.claude/gate.d/lkmm.sh:14`），「绑定那一条的键另含锚点指到的函数单元指纹」（`.claude/gate.d/lkmm.sh:17`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:42`）；原件「每份 litmus」（`.claude/rules/verification.md:42`）。
- E20（号 18）：「特性里有 gpu 时这一趟的测试参数带」（`.claude/gate.d/crash-point-replay.sh:256`）「不带 --test-threads」；「测试参数不带 --test-threads，GPU 开关都一样」（`scripts/gate/admission.py:3439`）；原件「带 gpu 特性那一趟与产品 GPU 真显卡那一趟」（`.claude/rules/verification.md:59`）。三个文件里没有一处不是弄坏开关而带 `--test-threads` 的，所以「各趟」不比脚本宽。
- E21（号 19、26）：「toolchain、proof（整格）、table 各一项」（`.claude/gate.d/formal-verification.sh:51`）；「不给就七格按下面的次序全跑」（`.claude/gate.d/formal-verification.sh:16`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:138`）与（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:165`）。
- E22（号 20）：「挂特性的文件跑到加复用 0 条、每处 NOT_RUN_NO_GPU_ADAPTER」（`.claude/gate.d/crash-point-replay.sh:249`）；「带特性那一格有本次未跑时也退 0」（`.claude/gate.d/crash-point-replay.sh:223`）；格名表那一行写「挂特性的测试文件逐个跑到或复用的用例不为 0」（`.claude/gate.d/crash-point-replay.sh:17`）；第三版「整道照样退 0」（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:146`）；原件「至少编进来一次」（`.claude/rules/verification.md:134`）。
- E23（号 21、27）：「钉全由 E165 第二段驱动」（`.claude/gate.d/formal-verification.sh:35`）；「same-origin-counter 同源性计数器」（`.claude/gate.d/formal-verification.sh:23`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:148`）、（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:170`）。
- E24（号 22）：「不是服务分片起的与 --check gate-verdict 不起驱动、只读库」（`.claude/gate.d/broad-mutation.sh:16`）；「不是服务分片起的这一道（gate.sh 整轮、手跑）与指定范围（--check gate-verdict）：不起驱动，只读 --summary」（`.claude/gate.d/broad-mutation.sh:30`）；「经统一任务服务交的一律分片」（`.claude/gate.d/broad-mutation.sh:6`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:149`）。
- E25（号 25）：「漏一道屏障与走页缓存两个对照必须判红」（`.claude/gate.d/qemu-device-streams.sh:5`）；「六个模式里键对不上的那几档并行各起一台虚机」（`.claude/gate.d/qemu-device-streams.sh:19`）；第三版（`research/prompts/rules-compress-r3-merged/dot-claude/rules/verification-reference.md:158`）。

## 改后比依据多出来或少掉的半句清单

多出来（依据里没有、改后有）：

- A（IW 第 30 行）「那一轮只攻上一轮的修补」：原件只有「打中的写回代码，再攻一轮」（`.claude/rules/implementation-workflow.md:10`），定案第 17、23 条也只说再攻一轮、没新打中就收（E7）；调度记录里第二轮真做的是四格（含「第一轮没看到的」），三方规则给第二轮留的也是三个攻击面。这半句把代码轮与治理文件轮的第二轮、第三轮收窄成只攻修补，是新做法，矛盾总表没登。
- B（IW 第 30 行）「开了第二轮的，在第二轮派一条辩方腿复核已有的判决」：原件代码轮没有这条（原件辩方只出现在三方规则的三轮多数里）；矛盾总表与去向表把它记成定案第 23 条，第 211 行没有这句（E7）。与三方规则第 45 行的「第二轮与第三轮里至少一轮」时点也不同。
- C（IW 第 30 行）「治理文件照条款改」：定案只说现查坐实就改；「照条款」读得出两种：照被打中的那条条款改，或照第 23 行「先在模型里做成必须报非 0 的世界再改定义」那一条改（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:23`）。两种读法在改定义时做法不同。
- D（IW 第 32 行）「写进判决的交用户表，不开第四轮」：做法来自主 agent 第二轮判决，不是用户定案（E9）；总表把它放在定案第 23 条名下（`research/prompts/rules-compress-r3-merged/contradictions.md:162`）。
- E（IW 第 31 行）「也不为它开一轮」：从定案第 17 条与三方规则「现查推翻而不采」推得出，定案没明说；做法与「没新打中就收」相同，没有害处。
- F（TWI 第 72 行）括号「（分工只分给其中一条的格、只有一条读了的文件）」：定案第 26 条只说「只有一条腿覆盖到的范围」；括号是修稿一方给的举例，读法不冲突。

少掉的（原件有、改后没有）：

- G（IW 第 23 行）「只改文档和脚本除外」：原件在「只管 crates/*/src/」后面有这半句（E4）。这半句是第一版就丢的，不在这次补丁里；上游那一节本来就写了这个例外（`.claude/singlefs-ai-sop/rules/show-me-test.md:13`），对「改定义不适用第 1 步」没有做法上的影响。总表也没登。
- H（TWI 第 45 行）原「停是不开下一轮，不是已经能拿这个没打中去支撑别的结论」：被定案第 26 条替掉，已登「用户定-26」。
- I（VR 第 146 行）「不指定范围那一趟至少编进来一次」：原件有；门禁头第 249、223 行写 0 条只报本次未跑，不判红，所以删得对；总表验证-14 已登。

## 三格的结论与证据

### 格一 第二轮修补对不对：不成立（两处）

- 放回原话的：号 15（`qemu.env` 那句，原件第 78 行原话）、号 5 与 7（带引号的节名，原件原写法）都是原件原话；号 5 的原件末尾「只改文档和脚本除外」仍没有（清单 G，不是这次补丁引入）。
- 照脚本文件头改的：号 12、13、16 到 27 对着文件头今天的原文逐处核过（E14 到 E25），十四处都与文件头一致、没有因此做法不同；其中号 20 照后一段，文件头自己前后不一致（第 17 行与第 249 行），修稿报告第 ④ 节已写。号 16 的三种被拒做法、号 18 的「各趟」、号 21 的「七格」、号 17 的「两条」都与文件头相同。
- 定案第 23 到 26 条：第 24 条（号 14）与第 26 条（号 11、9 的前半）落对了，意思与原话一致；第 23 条（号 2、3、4、6a、6c、8）落了「照代码轮收」「打中的现查坐实就改、改完再攻一轮、没新打中就收、最多三轮」，对；但同一处还塞进了清单 A、B（以及 C、D），这些不是第 23 条说的。其中 A 与 B 改了代码轮与治理文件轮的做法（第二轮只攻修补、第二轮多派一条腿），所以这一格有打中。
- 判：不成立。什么现象会推翻：用户确认清单 A、B 的意思（或在调度记录里找到那句原话）。

### 格二 修补引出的新问题：不成立（有几处，都不重）

- 同一判据又住了几处：不开第二轮的停法现在写在三处（TWI 第 45 行、TWI 第 72 行、IW 第 29 行），定案第 2 条要的是「写成一处、另一处指过去」，总表「用户定-2」认了两处不相抵；第三处 IW 第 29 行是复述加指针，目前一致，将来改一处容易漏另两处。辩方腿的时点住两处且不同（TWI 第 45 行「第二轮与第三轮里至少一轮」，IW 第 30 行「在第二轮」）。设计推论轮的定义住两处（TWI 第 43 行、IW 第 27 行），字面不同（IW 那份少了「拿依据（包括实验结论）」）。
- 指针：新节名「代码轮与治理文件轮怎么收」全树只有一个标题（`research/prompts/rules-compress-r3-merged/dot-claude/rules/implementation-workflow.md:25`）；「开不开下一轮」全树只有一处加粗开头（TWI 第 45 行）；「一条腿只抽一次样不算一次观测——否定结论尤其不算」是标题（TWI 第 68 行）；钩子「放宽检查的提交标识」、脚本头「一处调用点算合规，有一样成立就行」「显卡与清场」「feature-gated-tests 格」「判什么」各只有一处起头，都指得到。指到技能 three-way 的四个节名（开工快照、检查单、用户叫停、核查与判决、骨架起笔、分段落盘）在技能里各一个标题。
- 读得出两种意思的新句：(1) 清单 C「治理文件照条款改」；(2) IW 第 29 行「照同一份「开不开下一轮」那一段」：那一段的开头写着「（设计推论轮）」，TWI 第 43 行又说写了设计推论轮的段不管代码轮，后面接的冒号里写明借用的只是停法，读得通，但整段读下去会读到「先挂起、三轮多数」；(3) V 第 38 行「钩子不弹的」：不指定范围的六道门禁与崩溃放量不带标识时钩子是拒不是不弹（E16），句子对「不弹」的那一类（gate-staged.sh、gate.sh 整轮等）才成立；(4) VR 第 146 行那一项放在「判什么」一栏里却写「不判红」，读起来像没有判据。
- 判：不成立（有一处即打中）；(1) 最实，其余是低风险。什么现象会推翻：把 C 改成不二义的写法，并把第三处停法改成纯指针。

### 格三 能不能换上：不成立（有没登的不同，两处必须先处理）

- 范围：我核了补丁落在这一组的全部改动，以及这些改动所在的整句、整节；补丁之外的部分（第一、第二轮已判过的压缩）我只对原件三份做了一遍机械比对（按六字片段在整棵合稿树里找原件里带「不」「只」「一律」「除外」「先」「才」的句子），命中率低的句子大多是挪进参考件或技能的，逐句核没做，这部分读法推不出，留给攻方腿的格三。
- 没登的做法不同：清单 A（第二轮只攻修补）；清单 B（代码轮与治理文件轮多派辩方腿，总表登了但依据写错成定案第 23 条）；清单 D（总表同样把主 agent 的判决当成定案第 23 条）。清单 G（少半句）没登但没有做法上的影响。
- 登了且对的：号 11、14（定案 24、26 落对）、号 12、13、16 到 27 与号 1（总表验证-10 到验证-19、用户定-24、用户定-26）。
- 什么现象会推翻：用户说清单 A、B 就是他要的做法，或总表给出别的定案出处。

### 你问的一句

这一组能不能换上：除清单 A、B（含总表依据栏）之外都能换上；这两条在换上之前必须处理，其余可以换上之后再改。

- 换上之前必须改：
  - A：删去 IW 第 30 行「那一轮只攻上一轮的修补」，或先问用户（它改了代码轮与治理文件轮第二轮、第三轮的攻击范围，原件与定案都没有）。
  - B：IW 第 30 行「开了第二轮的，在第二轮派一条辩方腿复核已有的判决」：删去，或先问用户认；无论删不删，`contradictions.md` 第 117 行（验证-19）与第 162 行（用户定-23）、`dispositions-r2.md` 第 46 行里「用户定案第 23 条：开了第二轮的在第二轮派辩方腿」的依据要改成实情（出处是修稿一方的推，不是第 23 条）。
- 换上之后可以再改：
  - C：「治理文件照条款改」写成不二义（说明改定义时照 IW 第 23 行先做模型、改别的治理文件不要求）。
  - D：IW 第 32 行的依据在总表里改成「第二轮判决」，句子本身可留。
  - 停法三处并成「一处写全、两处指过去」（定案第 2 条）；辩方腿时点在 TWI 第 45 行与 IW 第 30 行对齐；设计推论轮的定义只留 TWI 第 43 行那一份、IW 第 27 行改指过去。
  - V 第 38 行补一句：不指定范围的六道门禁与崩溃放量不带标识时钩子是拒、不是不弹。
  - G：IW 第 23 行补回「只改文档和脚本除外」或写明不需要；VR 第 146 行那一项改成带动词的说法。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一 第二轮修补对不对 | 不成立 | 31 行（6 号拆五行）里 29 行与原件、文件头或定案一致（总表照脚本改的各条属实）；定案第 23 条落地时多塞了「只攻上一轮的修补」与「第二轮派辩方腿」 |
| 二 修补引出的新问题 | 不成立 | 停法三处、辩方时点两处不一；「治理文件照条款改」读得出两种；其余低风险 |
| 三 能不能换上 | 不成立 | 有两条没登的做法不同（A、B），改掉或问用户后可换上；补丁之外的部分推不出，未逐句核 |

## 没做什么

- 不判别的腿的格；不替主 agent 采纳或出判决；没往统一任务服务交任务，没起脚本、模型或装置，判决要引的数没有（本报告没有模型数）。
- 补丁之外未改动的句子没有逐句对原件核（只做了机械比对，见格三）；技能 three-way 没有 hunk，只核了别处指到它的节名。
- 没核总表依据栏里脚本行号（修稿报告已说明以文件头段落为准）；服务端对提交之外带 `--full-gate` 的命令收不收，没核。
- 草稿目录里的 `r2/`、`r3/` 两份树拷贝与 `cov.py` 交回前删（删前 `du -sh` 共 2.4M）；复现第二版：把合稿树拷一份后 `patch -R -p1 < diff-from-r2.patch`。

## 要主 agent 跑的命令

无。
