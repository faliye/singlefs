# merge-anchor-check-tooling-r3 正推腿（Sonnet）报告

轮：合入锚点核治理文件轮第三轮。背景材料 `research/prompts/_merge-anchor-check-tooling-r3-body.md`，上一轮判决 `research/prompts/merge-anchor-check-tooling-r2-main-verification.md`。
被判对象：主工作区五份 `scripts/gate/merge-and-verify.sh`、`scripts/gate/merge-and-verify-steps-lib.sh`、`scripts/gate/merge-and-verify-items-lib.sh`、`scripts/gate/merge-and-verify-selftest-lib.sh`、`scripts/gate/apply-writer-patch.py`。
开工时五份的 sha256 与 `research/prompts/merge-anchor-check-tooling-r3-snapshot/judged-sha256.txt` 逐份相同（量的，`sha256sum` 现取）。
这条腿没写模型、没起脚本、没交任务：下面「推的」是读码推出来的，「量的」是只读命令（grep、读文件、`git apply` 在空目录里试一次）当场核到的。
腿的分工是正推：不读别的腿的报告与草稿（禁读清单两项都没读）。引规则：`.claude/rules/three-way-inference.md@各条腿必须互不重复`、`.claude/rules/implementation-first.md@规矩`（方案按今天的实现谈；本题不碰 `crates/`）。

## 一、第二轮判决「三、改法」六条在今天主工作区里做了没有

判决里六条的原文行：`research/prompts/merge-anchor-check-tooling-r2-main-verification.md:38` 到 `:43`。

| # | 改法 | 判 | 文件:行 | 依据 |
|---|---|---|---|---|
| 1 | 合入前多一条 `--dry-run <merge-patch>`，第一轮按文件比锚定行的核删掉，代价写进出路 | 成立（有保留，见第二节 1b、1c） | `scripts/gate/merge-and-verify-items-lib.sh:867`、`:869`、`:871` | 867 起打 `apply-writer-patch.py --dry-run "$merge_patch"`，排在 871 合的那一条之前；869 写「多一趟约 400–740 秒、内存峰值约 15 GiB」与退 0、退 2 的办法。按文件比的核：四份 .sh 里 `anchored|ANCHORED|锚定` 的 grep 命中 0 行（量的，命令在第五节 A），`MAIN_TREE_*_NAME` 只剩指纹、起点、变异行三个（`scripts/gate/merge-and-verify.sh:1106`、`:1108`） |
| 2 | 带 `--skip-apply` 不给补丁目录打「变异锚点核本次未核」并点名为什么 | 成立 | `scripts/gate/merge-and-verify-steps-lib.sh:433`、`:435` | 先过自证样本标记，再判 `CHECK_PATCH_DIRS` 个数为 0，打一行写明「不知道补丁改了哪些文件与变异行，摆不出补丁之前的树」，返回 0、不记红 |
| 3 | 自证收三格 `--skip-apply` 与镜像格，平常就跑；own-row 与 registration 改成合入前 `--dry-run`；改坏登成开关各红在点名那一格 | 成立（读码；开关复跑 `merge-and-verify.sh` 一份未出结局，红格只能算推的） | `scripts/gate/merge-and-verify-selftest-lib.sh:775`、`:893`、`:910`、`:917`、`:923`；开关 `scripts/gate/merge-and-verify.sh:642`、`:644`、`:646`、`:654` | 三格 skip-apply（替换、倒不回、不给目录）与 dry-mirror 都不挂开关条件；七格 dry-run 在 772 到 829；四条新开关：merge-in-dry-run-dropped、anchor-skip-replacement-not-reversed、anchor-skip-irreversible-passes、anchor-skip-no-directories-silent，各自代码在 items-lib:867、merge-and-verify.sh:2260、steps-lib:451、steps-lib:434。锚定行两处开关不再登（grep 0 命中） |
| 4 | 三处耗时注释写成范围「约 400–740 秒、约 15 GiB」 | **不成立（缺一处）** | `scripts/gate/apply_writer_patch_plan.py:49` | 做到的两处：`scripts/gate/merge-and-verify.sh:286`、`:298`，`scripts/gate/merge-and-verify-steps-lib.sh:389`（另 items-lib:869 同写法）。第三处 `apply_writer_patch_plan.py:49` 在主工作区仍写「共约 423 秒、内存峰值 15.20 GiB」（量的：该文件 sha256 与工具员草稿 `base/` 那份相同 `70caf1fe…`，不在被换的五份里）。工具员报告把它单列为「主 agent 决定换不换」 |
| 5 | `apply-writer-patch.py` 自证样本树补 `.claude/gate.d/stage-inputs.tsv` | 成立 | `scripts/gate/apply-writer-patch.py:1041`、`:1080`、`:1740`、`:1775` | 两棵真跑 static-check 的样本树（锚点样本、path 依赖样本）在 `git init` 之前都写了登记表一行；主 agent 已量到该脚本自证与 52 条开关复跑过（正文观测，没有我复跑） |
| 6 | 707 行 `bash -c "$anchored_check_line"` 改成闭包折得出的写法 | 成立（读码；开关复跑拒算数待主 agent） | `scripts/gate/merge-and-verify-selftest-lib.sh:760`、`scripts/gate/lib_function_units_registry.py:327`、`:329`、`scripts/gate/lib_function_units.py:720` | 那一格连同 `bash -c` 删了；剩下的 7 处 `bash -c` 是 `$merge_check_line`、`$merge_rows_line`，在 registry 里登记为不拒算词（327、329）。新格起驱动用 `python3 "$scratch/dry-run-driver.py" …`（760），落在 `scripts/gate/lib_function_units.py:721`「一份都没有的不是仓里的脚本，不连也不拒算」那一种；这一条是读 lib_function_units 文件头推的，没跑 |

第 4 条一句话：工具员的草稿 `apply_writer_patch_plan.py` 改了第 49 行，没进这一轮换上的五份，所以今天主工作区里仍有一处旧数。推翻它的现象：主工作区该文件 sha256 不再是 `70caf1fe…` 且第 49 行已写范围。

## 二、第二节五条：结论

### 1. 合入前那一条 `--dry-run`：打中（1b 是死路，1a、1c、1d 是次要的）

**1a 合入前后的窗口：只窄了、没关上（推的）。**
- 次序是：核主工作区没变、核变异行、`--dry-run`、合的那一条（`scripts/gate/merge-and-verify-items-lib.sh:849`、`:862`、`:867`、`:871`）。前两条毫秒级、直接跑；`--dry-run` 经服务交，一趟约 400–740 秒（`scripts/gate/merge-and-verify.sh:298`）。
- 真合不跑静态核：`scripts/gate/apply-writer-patch.py:1014` 只在 `if dry_run:` 那一支里调 `static_check_report`；头注释也这么写（`scripts/gate/apply-writer-patch.py:86`「不加 --dry-run 的真打不跑这一步」）。真合落地前只核补丁改到的那几份文件：「先核每份主工作区此刻的 sha256 仍等于记下的补丁前」（`scripts/gate/apply-writer-patch.py:39`）。
- 输入：指纹那一条退 0 之后，别的会话在 `--dry-run` 起跑到真合落地之间（含排队、下一趟回合）改了补丁没改到的文件，或追加了点名同一目标的变异行，或改了登记表。指纹已核过、`--dry-run` 已照旧树判过，真合三方合并照样落地，只有合完另交的 mutation-tables 那一格会看见（`scripts/gate/merge-and-verify-items-lib.sh:519`、`scripts/gate/apply-writer-patch.py:87`）。这正是这套工具想挪到合入之前的那类断。
- 这一条第一轮的核同样有这个缝（核与合之间），区别是现在缝里多了 `--dry-run` 的整个耗时：指纹检查被推早了 400–740 秒。把便宜的指纹两条放到 `--dry-run` 之后、紧贴合的那一条，缝里就只剩 `--dry-run` 自己的 `static-check` 窗口。这是读码推的，没有构造过并发样本。
- `--dry-run` 自己也有窗口：补丁之后的树先拷出来跑（`scripts/gate/apply_writer_patch_plan.py:1121`），基线直接在活的主工作区上后跑（`scripts/gate/apply_writer_patch_plan.py:1122`），两次之间隔一趟 static-check（每趟 200–370 秒量级，推的）。基线那一趟之前别的会话把一行原本就红的变异行修好了，补丁之后的树里还红着、基线已绿，就被当成这一批新弄断的（误拒，出路是整批重交）。窗口小，但方向是误拒，不是漏。

**1b 合并补丁里有拷贝范围之外的文件段时，合入前 `--dry-run` 一律退 2「没做成」，出路做不了（量的一半，推的一半）。**
- 输入：合入后验证带一份「只含拒算表的补丁目录」（`scripts/gate/merge-and-verify-items-lib.sh:490`、`scripts/gate/merge-and-verify.sh:1119` 登记的主 agent 做法）。`merge-patch/crates.patch` 带 `diff --git a/.claude/gate.d/refused-tests.tsv …` 那一段（`scripts/gate/merge-and-verify.sh:272`「登记表的改动一起在里面」）。
- `--dry-run` 把整份补丁 `git apply` 到静态核的临时树上（`scripts/gate/apply_writer_patch_plan.py:1103`）。临时树的内容是拷贝范围闭包加另声明的 `litmus/` 与 `.claude/gate.d/stage-inputs.tsv`（`scripts/gate/apply_writer_patch_plan.py:44`）。`refused-tests.tsv` 不在里面：`crates`、`litmus` 与 `Cargo.toml` 下没有任何文件提到它（量的：`grep -rln 'refused-tests' crates litmus Cargo.toml` 0 命中）；闭包是否真不收它要主 agent 跑第五节 B 才定，这一句是推的。
- 量的：在空目录里 `git apply` 同形补丁，输出 `error: .claude/gate.d/refused-tests.tsv: No such file or directory`、退 1（样本 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r3-sonnet/probe-refused/crates.patch`，sha256 `b48a3946a4e922bde1e94a7f1f81dceb9c02c4f41aa305c9250012eddc8ea406`；同一个补丁在主工作区 `git apply --check` 退 0）。
- 于是 `scripts/gate/apply_writer_patch_plan.py:1105` 抛 RuntimeError，`scripts/gate/apply_writer_patch_plan.py:1124` 返回「✗ 变异锚点静态核没做成」，`scripts/gate/apply-writer-patch.py:1016` 退 2。
- 出路对不上：`scripts/gate/merge-and-verify-items-lib.sh:869` 的退 2 只分「静态核红」与三种别的原因（开着的快照、在换、三方合并冲突），没有「没做成」；核自己的出路写「改好再跑一次 --dry-run」（`scripts/gate/apply_writer_patch_plan.py:1116`），补丁里那一段是这一批该有的，改不掉。照单做就走不到合的那一条，只剩删掉那一行（这正是弄坏开关 merge-in-dry-run-dropped 做的事）。
- 设计者知道这个坑：合入后验证自己的锚点核专门只收落在拷贝范围里的段（`scripts/gate/merge-and-verify.sh:2177`、`:2187`），原因注释写「范围外的文件临时树里没有，套不上」。换成不过滤的 `--dry-run` 之后这一道没跟过去。
- 同形的还有 `.claude/gate.d/crate-roles.tsv`（新增包的补丁要登一行角色，`scripts/gate/apply_writer_patch_plan.py:32`；`crates` 下没有 .rs 提到它）。
- 自证没造这一形：七格 dry-run 的补丁里 `diff --git` 的路径只有 `crates/alpha/src/merge_table.txt` 与 `.claude/gate.d/stage-inputs.tsv`（`scripts/gate/merge-and-verify-selftest-lib.sh:813`），都在临时树里。
- 推翻它的现象：第五节 B 的闭包输出里有 `.claude/gate.d/refused-tests.tsv` 与 `.claude/gate.d/crate-roles.tsv`；或第五节 C 的探针 `--dry-run` 没有退 2、起了 static-check。

**1c 两处出路在同一份输出里说法相反，且非 2 的结局出路偏重（推的）。**
- `--dry-run` 退 2 的拒绝说明自带一句「改好再跑一次 --dry-run」（`scripts/gate/apply_writer_patch_plan.py:1116`）。对着 `<日志目录>/merge-patch` 照它改，改的是派生产物：改完 `--dry-run` 绿，真合也进得去，快照上验过的却不是这份补丁。`scripts/gate/merge-and-verify-items-lib.sh:869` 写的是整批重交。合入后验证自己的锚点核打出同样内容时，专门把「→ 怎么办」那一行去掉，注释写明原因（`scripts/gate/merge-and-verify.sh:2203`）；合入前这一条是直接起 `apply-writer-patch.py`，没人去掉。
- 非 2 的结局（被停、服务重启、`task wait` 报 78 一类）：`scripts/gate/merge-and-verify-items-lib.sh:845` 的总则是「退非 0 不合……整批重交」，而 `--dry-run` 只读、同一条原样再交就行。整批重交要再编再测，按小时计。869 没单独写这一支。
- 推翻它的现象：主 agent 在第一次真合入时读到 `--dry-run` 的输出里没有那句「改好再跑一次 --dry-run」，或 869 已补上「被停的原样再交同一条」。

**1d 内存与耗时：合的那一步不会被它拖死；`--dry-run` 自己撞上限由服务乘 1.5 重排（推的，没量过）。**
- `--dry-run` 与真合是两个任务。真合不跑静态核，也不重排：它是改工作区的工具，下限 4 GiB（`task-server/task_server_constants.py:130`），撞上限一律打回（`task-server/task_server_constants.py:258`）。`--dry-run` 属只读参数（`task-server/task_server_constants.py:249`），不算改工作区，撞上限乘 1.5 重排：服务自证里干跑撞上限跑两次尝试后成功，真打只有一次、记 memory-cap-hit（`task-server/task_server_selftest_memory.py:981`、`:992`）。
- 乘数与上限：`task-server/task_server_lib_constants.py:498`（1.5）、`:499`（至多 16 次）。没有记录的命令缺省额度 4 GiB（`task-server/task_server_lib_constants.py:523`）：4、6、9、13.5、20.25，离 15 GiB 要撞 4 次才够；命令里的补丁目录每趟不同，命令键是否按路径分不清，所以每趟是不是从缺省额度起是推的。连撞 3 次服务推 repeated-cap-hits 提醒（`task-server/task_server_lib_constants.py:134`）。
- 单次 static-check 超时 900 秒（`scripts/gate/apply_writer_patch_plan.py:49`），一趟 `--dry-run` 两次，上限 1800 秒量级，超时按判不了退 2（`scripts/gate/apply_writer_patch_plan.py:1130` 一带）。
- 两趟先后跑、不并行，是为了不把峰值翻成约 31 GiB（`scripts/gate/apply_writer_patch_plan.py:1120`，量的数写在那一行注释里）。
- 被杀之后的出路：服务自己重排，主 agent 不用做什么；打回（memory-cap-at-maximum）的由 `task-server/task_server_lib_constants.py:140` 那一行判。869 对这一支一句没写（见 1c）。

### 2. 删掉按文件比锚定行的核之后：没有只有它拦得住的形（成立），有一个方向相反的小口子

- 旧核比的是「改到的文件上锚着的行」整行集合有没有变，对「改了又改回」本来就判绿；新核对着合入那一刻的树判，两者在这一形上一致，不存在「改回就漏」。
- 旧核拦、新核不拦的只有「锚点原文仍恰好命中 1 次、锚着的代码已搬位置」。第二轮判决已按误拒采（`research/prompts/merge-anchor-check-tooling-r2-main-verification.md:26`，第 3 条），本轮不当漏。
- 旧核与新核共有的缝（核与落地之间）见 1a；新核多出来的缝是耗时。没有别的形：指纹仍罩补丁改到的包与依赖关系上的包（`scripts/gate/merge-and-verify.sh:317`）。
- 补丁之前就红的替换成同样红的（键只含种类、变异名、原文所在文件与命中次数，`scripts/gate/apply_writer_patch_plan.py:1061`）按「补丁前就红」放行：旧核也不管，第一、二轮的「补丁之前的树摆法」已攻过，不重复。

### 3. `--skip-apply` 三格与「本次未核」：成立，三种情形分得开

- 不给补丁目录：打「变异锚点核本次未核」、返回 0（`scripts/gate/merge-and-verify-steps-lib.sh:433`、`:435`）；同一趟另有一行「补丁打没打本次未核」（`scripts/gate/merge-and-verify-steps-lib.sh:40`），是别的核。
- 给了目录而目录里什么都没有：另一句「变异锚点核不做：带 --skip-apply 给的补丁目录里没有 crates.patch，也没有变异表文件」（`scripts/gate/merge-and-verify-steps-lib.sh:446`），不含「本次未核」那几个字，自证那一格的 `grep -qF` 不会被它蒙过去（`scripts/gate/merge-and-verify-selftest-lib.sh:924`）。
- 倒不回：`✗ 变异锚点核没做成`、返回 1（`scripts/gate/merge-and-verify-steps-lib.sh:454`）；替换行倒回前像后改后行判红走锚点核自己的新红路径，点名 ANCHOR 与命中次数。
- 倒出的补丁之前的树：代码文件走临时副本里照次序倒着 `git apply --reverse`，表走 `MERGE_TABLE_REVERSE_PROGRAM`（`scripts/gate/merge-and-verify.sh:2259` 的替换换回前像、`:2271` 的删除接回表尾），目录次序倒过来给（`scripts/gate/merge-and-verify-steps-lib.sh:488`）。我没有找到树摆错的输入；追加的行、改后的行在表里找不到原样的都进「倒不回」红。
- 一格较弱：不给补丁目录那一格只核了那几个字，没核退出码（`scripts/gate/merge-and-verify-selftest-lib.sh:924`）。把这一支改成返回 1 的坏法，那一格不红；是否有别的格兜住，没核（推的）。

### 4. 新开关四条、改了红格的六条、七格 dry-run：成立（读码），有两处分辨力缺口

**样本是不是真走到了锚点静态核那一步（量的一半：读码；没有跑）。**
- 六格必拦：每格的判据是 `--dry-run` 退 2 且输出里有 `<ANCHOR 或 RELEASE> <行号> <变异名> `（`scripts/gate/merge-and-verify-selftest-lib.sh:766`）。这一行只有 `static_check_report` 把假 static-check 的 ANCHOR 或 RELEASE 行拼进拒绝说明才有；前置的核（补丁认不出、前像对不上、三方合并冲突、走不到的理由）也退 2，但不会有那一行，所以红在前置核上的样本过不了这一格。`scripts/gate/apply-writer-patch.py:1014` 到 `:1016` 就是那条路。
- 一格必放行：判据是退 0 且输出里有「核过了（--dry-run」（`scripts/gate/merge-and-verify-selftest-lib.sh:827`），这句只在 `scripts/gate/apply-writer-patch.py:1017` 静态核过了之后才打。
- 每格还先核合入前那一条真的打在合的那一条之前（`scripts/gate/merge-and-verify-selftest-lib.sh:757` 一带），与实际打出的整行用 `grep -nxF` 比。

**十条开关各红在哪一格（读源码手推，与文件头开关表 `scripts/gate/merge-and-verify.sh:583` 到 `:661` 逐条对过，一致；没有复跑）。**
- merge-in-dry-run-dropped：七格 dry-run 全红（`dry_printed` 为 0）；别的格不碰这一段。
- anchor-skip-replacement-not-reversed：只有 skip-replace 一格用 mutations-replacements.tsv，改后行不换回前像则补丁前后都红、判成「补丁前就红」退 0，那一格要退 1，所以只红它。
- anchor-skip-irreversible-passes：只有 skip-irreversible 一格走到倒不回那一支，只红它。
- anchor-skip-no-directories-silent：只有 skip-none 一格不给目录，只红它。
- 改了红格的六条：merge-in-commands-dropped 多红七格 dry-run；anchor-check-skipped、anchor-own-table-not-restored、anchor-mutation-only-skipped 各多红 skip-replace；anchor-skip-apply-unchecked、anchor-skip-table-not-reversed 各多红 skip-replace 与 skip-irreversible。逐条按代码走过，与表一致。
- 主 agent 正文观测：`merge-and-verify.sh` 的开关复跑「在跑、还没结局」，所以这一节只到「推的」，等复跑「点名的格与实际红的格逐格相同」。

**分辨力缺口两处（推的）。**
- 必放行那一格的退出码分支（`scripts/gate/merge-and-verify-selftest-lib.sh:827` 的 `dry_rc -ne 0`）没有任何弄坏开关能让它红：旧的按文件比的核已删，没有「过严」的代码可开；开关复跑只经 `MERGE_AND_VERIFY_BREAK`，到不了 `apply-writer-patch.py` 自己的开关（`APPLY_WRITER_PATCH_BREAK`，那几条在它自己的 52 条复跑里）。所以这一格在开关表里只因 merge-in-dry-run-dropped 红在「没打出」那一支，「放行」那一支没有被证过会红。
- 七格都用假 static-check（`scripts/gate/merge-and-verify-selftest-lib.sh:718` 起的 `fake-registered-static-check.py` 与 `fake-anchor-static-check.py`），RELEASE 的登记表读法（`test=<包>:<目标>:` 正则）是自证自己写的一份；真的 RELEASE 判法与它是否同形，这里不核。同样 `apply-writer-patch.py` 自证里 RELEASE 只有假脚本（`scripts/gate/apply-writer-patch.py:1831`），没有真 static-check 读登记表判 RELEASE 的格。
- 推翻第一处的现象：主 agent 给 `apply-writer-patch.py` 弄一个让 `--dry-run` 过严的开关并让七格里放行那格红；推翻第二处的现象：真 static-check 读一份带 `crash-case:` 行的登记表的格存在。

### 5. `apply-writer-patch.py` 自证样本树补的登记表：够（成立，推的）

- 补的是一行 `crates-mutation-replay.sh<TAB>-<TAB>copy-range=crates/ copy-range=litmus/ copy-range=Cargo.toml copy-range=Cargo.lock copy-range=.cargo/<TAB># 自证样本`（`scripts/gate/apply-writer-patch.py:1041`）。键与真登记表同名（`.claude/gate.d/stage-inputs.tsv:40`），列的读法是「第一列是键，第二列是入口（- 是没有），第三列起到 # 之前是条件」（`scripts/gate/admission_base.py:186`），static-check 缺这一行或缺这张表会打 ERROR 退 2（`scripts/mutation/crates-mutation-rows.py:77`）。
- 真跑 static-check 的格只有五个（弄断锚点、不弄断放行、补丁前就红、补丁前也红只拒新的、path 依赖目录；`scripts/gate/apply-writer-patch.py:1744` 到 `:1781`）。它们要的只是行存在与拷贝范围；RELEASE 要的 `crash-case:` 行它们都不判，用的是假脚本（1822 到 1836）。所以只放一行够。
- 没有被这张表覆盖的：真登记表里的 `copy-range=scripts/…` 各条没放，样本树里没有那些路径，没格需要。
- 推翻它的现象：加一个真 static-check 判 ROW（文件不在拷贝范围）或 RELEASE 的格，那一格红在登记表缺条目上。

## 三、要主 agent 跑的命令

我没有起任何脚本、没有交任务。下面三条都是只读性质（A 是读文件、B 是只算闭包、C 的 `--dry-run` 不写主工作区），在仓根起。

**A. 复核按文件比的核真删净（读文件，不必交任务）**
- 命令：`grep -nE 'anchored|ANCHORED|锚定' scripts/gate/merge-and-verify.sh scripts/gate/merge-and-verify-steps-lib.sh scripts/gate/merge-and-verify-items-lib.sh scripts/gate/merge-and-verify-selftest-lib.sh`
- 在哪起：仓根。看什么：没有输出（我现跑的结果是 0 行）。

**B. 第二节 1b 的前提：静态核临时树的闭包里有没有 `refused-tests.tsv`、`crate-roles.tsv`**
- 命令：`task-server/task submit -- python3 scripts/hosts/lib_repository_copy.py entries /home/fy5090/code/singlefs --test-inputs --with litmus=变异行可以锚在litmus下 --with .claude/gate.d/stage-inputs.tsv=static-check读登记表`
- 在哪起：仓根。看什么：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`，`task-server/task log <号>` 里列出的 `REPOSITORY_COPY entry` 行有没有 `.claude/gate.d/refused-tests.tsv`、`.claude/gate.d/crate-roles.tsv`。没有，1b 的前提成立；有，1b 撤回。

**C. 第二节 1b 的现象：合并补丁里有范围外文件段时 `--dry-run` 的结局**
- 命令：`task-server/task submit -- python3 scripts/gate/apply-writer-patch.py --dry-run /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-sonnet/probe-refused`
- 在哪起：仓根。目录里只有 `crates.patch`（sha256 `b48a3946a4e922bde1e94a7f1f81dceb9c02c4f41aa305c9250012eddc8ea406`，往 `refused-tests.tsv` 末尾加一行，主工作区 `git apply --check` 退 0，不会真写）。
- 看什么：`type=command-failed exit=2`，日志里有「✗ 变异锚点静态核没做成：补丁在临时树上套不上」与「No such file or directory」，且很快（秒级）退出。若它起了 static-check 跑到分钟级，说明闭包收了这张表，1b 撤回，到那一步可以让它跑完（读不写，约 15 GiB）。
- 退 2 但原因是别的（开着的三方轮开工快照里有这份表）：看输出第一句，不算打中也不算没打中，换一个不在快照里的登记表另做探针。

**D. 第二节 1d 的数（等第一次真合入时顺手读，不另交）**
- 命令：`task-server/task status <合入前 --dry-run 的任务号>`。看什么：尝试几次、每次的峰值与结局；缺省额度起、撞几次才跑过。这是推的数，没量过。

**E. 主工作区 `merge-and-verify.sh` 的开关复跑**（正文已说在跑）：第四节十条开关的格对应关系等它的「点名的格与实际红的格逐格相同」。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一、改法 1 | 成立（有保留） | `--dry-run` 已打在合之前，按文件比的核已删净；保留见 1b、1c |
| 一、改法 2 | 成立 | 不给补丁目录打「变异锚点核本次未核」 |
| 一、改法 3 | 成立（读码） | 三格 skip-apply 与镜像格平常就跑，四条新开关登了；红格手推与开关表一致，复跑待主 agent |
| 一、改法 4 | 不成立 | `scripts/gate/apply_writer_patch_plan.py:49` 仍写「约 423 秒、15.20 GiB」，没在被换的五份里 |
| 一、改法 5 | 成立 | 两棵真跑 static-check 的样本树补了登记表一行 |
| 一、改法 6 | 成立（读码） | 707 行那一格删了，新驱动写法落在 lib_function_units.py:720 不拒算那一种；拒算数待开关复跑 |
| 二、1 | 打中 | 1b：合并补丁带范围外文件段（拒算表、角色表）时 `--dry-run` 退 2「没做成」，出路做不了；1a 窗口只窄了；1c 出路相反且偏重；1d 撞上限由服务重排 |
| 二、2 | 成立 | 没有只有旧核拦得住的形；旧核拦的「锚点仍命中 1 次」已按误拒采 |
| 二、3 | 成立 | 三种情形输出分得开；倒出的前树我没找到错的输入；不给目录那一格只核文字 |
| 二、4 | 成立（读码） | 六格必拦要有 ANCHOR 或 RELEASE 行才过，不会红在前置核；两处分辨力缺口 |
| 二、5 | 成立（推的） | 真跑 static-check 的五格只需要行存在与拷贝范围，一行够 |

## 五、什么现象会推翻上面的结论（汇总）

- 1b：第三节 B 的闭包里有那两张表，或 C 的探针 `--dry-run` 没退 2、起了 static-check。
- 1a：主 agent 在 `--dry-run` 与真合之间改一份补丁没改到的锚着的文件，真合后 mutation-tables 那一格没有红。
- 1c：`--dry-run` 实际输出里没有「改好再跑一次 --dry-run」。
- 改法 4：`apply_writer_patch_plan.py` 第 49 行已是范围。
- 二、2：给出一个输入，旧核（`git log` 里第一轮那版，`scripts/gate/merge-and-verify-items-lib.sh` 的 anchored 段）拦、新核放，且不是「锚点仍命中 1 次」。
- 二、4：开关复跑里点名的格与实际红的格不符，或某个样本红在前置核。
- 二、5：真 static-check 的某一格红在登记表缺条目上。

## 六、没做什么

- 没有起脚本、没有交任务、没有跑门禁与自证、没有复跑开关、没有写模型（所以没有 `rerun.sh`）；除了报告与 `probe-refused/crates.patch`、一个临时空目录之外没写任何文件。
- 没有采纳或否决任何一条：判决由主 agent 做。
- 没读第一、二轮攻方腿与辩方腿的报告，只读了第二轮判决与工具员的修补报告 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r2-fix/result.md`（正文点名的修补材料）。
- 没有量内存与耗时：400–740 秒、15 GiB 是被判材料里的数，1d 里关于服务重排的次数是读服务常量推的。
- 第一、二轮已攻过没打中的角度（判法复用、补丁之前的树摆法、多份补丁合成再核、核没做成判红、红了不打合入命令、RELEASE 去行号、日志全数与出路、两棵临时树里的登记表）不重复；1c 说的是 `--dry-run` 这一条自己的出路，不是日志全数。
