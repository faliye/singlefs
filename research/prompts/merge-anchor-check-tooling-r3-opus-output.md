# merge-anchor-check-tooling-r3：攻方腿（Opus）报告（2026-11-21）

正文 `research/prompts/_merge-anchor-check-tooling-r3-body.md`，开工快照 `research/prompts/merge-anchor-check-tooling-r3-snapshot/`（开工时在仓根 `sha256sum -c` 七份都 OK）。草稿目录 D = `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/`。

没写模型目录（这一题是治理文件轮，打中都做成了自证样本格），没有 `rerun.sh`。样本文件：

| 文件 | sha256 |
|---|---|
| D/attack3.patch（三份脚本，230 行；`git apply -p2` 在副本根打） | f3802ef9191665a2e5e7b155b306c5990a2ece66671bb464c07c2eac195ea7a6 |
| D/new/scripts/gate/merge-and-verify-selftest-lib.sh | 5bb12c7ffd552304132c0f7d127a40804ed468db77a9d9b80bb18eaa64742b32 |
| D/new/scripts/gate/merge-and-verify-steps-lib.sh | aee7c2dc3beead31c8545c0445a9098ae852f21aab614ab23db6fd3f8ed8754d |
| D/new/scripts/gate/merge-and-verify-items-lib.sh | 978ba39fe7c9dc3ee9515868e1812f3030f23f32129049ed73fdd86bdd3a05c2 |

D/base/ 是打补丁之前那三份，sha256 与开工快照逐份相同（d22f2ae5…、cf2af64a…、d76ba9df…）；`git apply -p2 --check -v` 在 D/base 的拷贝上三份都过；新版三份 `bash -n` 过，样本格里的假 static-check 抽出来 `py_compile` 过。补丁只加两样：`ATTACK3_CELLS=1` 才跑的攻方样本格（插在自证「同一份文件里不受影响的追加应当放行」那一格之后），`ATTACK3_MUTATION=<名>` 才生效的三处改坏。全部读数都是「推的，待主 agent 跑」，命令在「要主 agent 跑的命令」一节。

## 各格判定一览

| 攻击面 | 形 | 判定 | 样本格（失败行「✗ 攻方样本 …」） |
|---|---|---|---|
| 1 窗口 | 核主工作区没变那一条排在 --dry-run 之前，--dry-run 跑着时改了这一批的包，合之前再没有一条核 | **打中**（推的） | window-fingerprint |
| 1 窗口 | --dry-run 跑完到合之间别的会话追加锚在改到的文件上的行，合进去才断 | **打中**（推的） | window-table-append |
| 1 窗口 | --dry-run 两棵树不在同一刻取：补丁之后那一棵先拷、先跑，没打补丁那一棵是之后几百秒的主工作区，别的会话这时修好一行补丁前就红的行，算成这一批新弄断的 | **打中，误拒**（推的） | baseline-skew |
| 1 出路 | 合的那一条退 2、等完「重做核的那几条、再交同一条」不重做 --dry-run（等的可能是一整轮三方） | **打中**（推的） | redo-loop |
| 1 出路 | --dry-run 判不了、没做成、被停、撞上限的结局没有出路，落到「退非 0 不合…整批重交」 | **打中，代价**（推的） | dry-run-way-out-outcomes |
| 1 内存 | 15 GiB 撞上限 | **没打中**：服务撞 250 乘 1.5 自动重排；头一回没记录从 4G 起、要撞四次（推的） | 无 |
| 1 出路 | 照文档做：`.claude/skills/merge-verify/SKILL.md` 第 3 条只有核与合两步，没有 --dry-run | **打中**（读文件核过；不在这一题五份里） | 无（文档） |
| 2 | 改了又改回：别的会话开跑之后追加锚在改到的文件上的行；--dry-run 跑补丁之后那一棵时那份文件被临时改过、没打补丁那一棵跑完又改回，补丁前后都红、照放行；第一轮那条按文件比的核照记下的行一比就拦 | **打中**（推的） | transient-mask |
| 2 | 窗口本身：第一轮那一条只读、几秒，排在合的那一条紧前；--dry-run 几百秒、经服务排队，删掉之后没有快核贴着合 | **打中**（同 window-table-append） | window-table-append |
| 2 | 锚点还中、而这一批让那一行的变异失效（编不过、等价） | **没打中**：开跑之前就有的行同样不判，第一轮那条核只是顺带全拒 | 无 |
| 3 | --skip-apply 三格：倒出的补丁之前的树、三种情形分开 | **没打中** | 无 |
| 3 | 「本次未核」那一行改成记红、不写为什么，现有格照绿 | **自证分不出**（推的） | skip-none-not-red（配改坏 skip-none-records-red、skip-none-reason-dropped） |
| 4 | --dry-run 七格是不是走到了锚点静态核 | **没打中**：六格都在 --dry-run 的输出里找 ANCHOR 或 RELEASE 那一行，放行那一格找「核过了（--dry-run」 | 无 |
| 4 | 新开关四条与改了红格的六条 | **没打中**（读码逐条推，各红在点名那几格；主 agent 那一趟开关复跑还没结局） | 无 |
| 4 | --dry-run 那一条的出路（含代价）删掉，现有格照绿 | **自证分不出**（推的） | dry-run-way-out-cost（配改坏 dry-run-way-out-dropped） |
| 5 | 样本登记表只放 crates-mutation-replay.sh 一行 | **没打中**：真跑 static-check 的五格要的 ANCHOR、CHECKED 都判得到；RELEASE 那一支两份自证都只走假 static-check（射程边上的线索） | 无 |

## 一、合入前那一条 --dry-run：窗口、出路、内存

### 1a 时间线（读码，推的）

合进主工作区那一段打出的次序：「核主工作区没变与核变异行的那几条只读、直接跑，--dry-run 与合的那一条经服务交」（`scripts/gate/merge-and-verify-items-lib.sh:845`）；核主工作区没变那一条是 `find … | LC_ALL=C sort -z | xargs -0 -r sha256sum | diff -u <指纹> -`（第 855–856 行），之后才是「submit -- python3 "$root/scripts/gate/apply-writer-patch.py" --dry-run "$merge_patch"」（`scripts/gate/merge-and-verify-items-lib.sh:868`），再之后是合的那一条（第 871 行）。真打自己不核锚点：「不加 --dry-run 的真打不跑这一步」（`scripts/gate/apply-writer-patch.py:86`）。

--dry-run 一趟里读主工作区的有三个时刻（`scripts/gate/apply_writer_patch_plan.py`）：
- T0：`plan_with_three_way` 读主工作区此刻的变异表、合出合并后的表（「with open(table_path, encoding="utf-8") as handle:」（`scripts/gate/apply_writer_patch_plan.py:985`）），三方合并也照此刻重对补丁；
- T1：「tree = build_patched_tree(root, merged, patch)」（`scripts/gate/apply_writer_patch_plan.py:1119`）照主工作区此刻拷拷贝范围、写进 T0 合出的表、套补丁；
- T2：「after_code, after_lines = run_static_check(make_command(tree))」（`scripts/gate/apply_writer_patch_plan.py:1121`）跑完补丁之后那一棵（一趟约 200–370 秒，正文给的两趟合计 400–740 秒折半，推的），才「before_code, before_lines = run_static_check(make_command(root))」（`scripts/gate/apply_writer_patch_plan.py:1122`）直接在活的主工作区上跑基线。

所以「核主工作区没变」那一刻 Tf 到真合 Ta 之间隔着：--dry-run 排队（预测约 15 GiB，要等那一台 slice 放得下，推的）+ 400–740 秒 + 合的那一条排队；第二轮判决改法第 1 条要的是「在合入那一刻的主工作区上照补丁前后比锚点」（`research/prompts/merge-anchor-check-tooling-r2-main-verification.md:38`），而 --dry-run 判的是 T0、T1、T2 三个不同时刻，都不是合入那一刻。

### 1b 打中：window-fingerprint（核主工作区没变那一条被 --dry-run 隔开）

输入：任何一批补丁；照打出的次序做：Tf 跑核主工作区没变（退 0）→ 交 --dry-run → --dry-run 跑着的时候别的会话改了这一批改到的包里的别的文件（样本：alpha 的 `tests/case_a.rs`）→ --dry-run 退 0 → 合。打出的那一段里 --dry-run 之后、合的那一条之前没有一条核，合的那一条照三方合并合进去：补丁只在开跑那一刻的快照上验过，合进去的包已经不是验过的样子。第二轮的设计（没有 --dry-run）里这一段只隔几秒；这一轮把它拉长到几百秒加两次排队。样本格：照打出的次序跑，--dry-run 之后改那份文件，再跑 --dry-run 之后、合之前打出的核——一条都没有（「一条都没打」），而核主工作区没变那一条此刻重跑退非 0。

### 1c 打中：window-table-append（--dry-run 跑完到合之间追加的行）

输入：这一批 `crates.patch` 把 alpha 的 `merge_table.txt` 里的 `1` 改成 `2`（样本 `merge-anchor-break`）；--dry-run 退 0 之后、合之前，别的会话往主工作区变异表追加 `attack3-window-row`（锚在 `merge_table.txt` 的原文 `1`）。合的那一条：变异表三方合并留下这一行，补丁把 `1` 改没，合进去之后它命中 0 次。之后只有合完另交的 mutation-tables 那一格会红（`scripts/gate/merge-and-verify-items-lib.sh:519`），是合进去之后才看见，不是合之前拦住。

### 1d 打中：baseline-skew（两棵树不在同一刻取，误拒）

输入：主工作区表里有一行补丁之前就红的 `attack3-prebroken-row`（原文 `7` 在 beta 的 `merge_extra.txt` 里命中 0 次；开跑时的变异锚点核报「补丁前就红的 1 处」、照放行）。--dry-run 跑补丁之后那一棵（T1 拷的树、T0 合的表）时，别的会话把这一行的原文重取成 `1`（修好了）。T2 的基线是活的主工作区：这一行不红；补丁之后那一棵里它还红 → 算成「补丁之后有、补丁之前没有」→ 退 2，出路写「开跑之后别的会话改了锚着的文件…让这一批新弄断了那几行：不合…整批重交」。这一批一个字没碰那一行，真合也不会弄断它（真合读的是修好之后的表）。违反改法第 1 条「只拦这一批新弄断的」（`research/prompts/merge-anchor-check-tooling-r2-main-verification.md:38`）。

### 1e 打中：出路

- redo-loop：合的那一条退 2 的两种只等的（开着的三方轮开工快照、同一个仓有别的在换）写「这两样等完重做核的那几条、再交同一条，不重验」（`scripts/gate/merge-and-verify-items-lib.sh:872`）；三方合并冲突、核与换之间被改了的写「三方合并冲突、核与换之间主工作区被改了的，重做核的那几条：退非 0 照上面整批重交，退 0 再交同一条」（`scripts/gate/merge-and-verify-items-lib.sh:873`）；不是 2 的结局照它办（第 874 行）。第 845 行把「核的那几条」与 --dry-run 分开写，照字面重做的只有两条快核，不重做 --dry-run。等一轮三方判决可以是几小时，这段时间主工作区的表与登记表随便变；第二轮那一版的「重做核的那几条」里有按文件比锚定行的那一条，这一版删掉了它、又没把 --dry-run 补进来。另一读法（「核」含 --dry-run）同样站不住：--dry-run 自己退 2 的那几种（「退 2 的别的原因（开着的三方轮开工快照、同一个仓有别的在换、三方合并冲突）照下面合的那一条退 2 的办」（`scripts/gate/merge-and-verify-items-lib.sh:869`））回到第 872 行那一句，「再交同一条」指的是哪一条读不出来。
- dry-run-way-out-outcomes：--dry-run 的退 2 还有「变异锚点静态核判不了」（`scripts/gate/apply_writer_patch_plan.py:1130`，static-check 起不了、超时、ERROR、没打 CHECKED）、「变异锚点静态核没做成」（`scripts/gate/apply_writer_patch_plan.py:1124`，拷贝范围闭包算不成、补丁在临时树上套不上）、上一次没换完；退非 2 的有被停、撞上限、launcher-lost 这类。第 869 行那一句都没点名，照第 845 行「退非 0 不合…整批重交」办：整批重验（按小时计）去补一条只读、没写过任何东西的 --dry-run，该做的是查原因后再交同一条 --dry-run。
- 照文档做：`.claude/skills/merge-verify/SKILL.md` 第 3 条「合进主工作区」那一段写「先在仓根直接跑核的那几条」（`.claude/skills/merge-verify/SKILL.md:41`），下一条直接交 apply-writer-patch.py 真合（第 42 行）；全文 `grep -n 'dry-run'` 零命中（读文件核过）。主 agent 照技能文档做就整条跳过 --dry-run，第二轮判决改法第 1 条在这条路上一处都不起作用。工具员报告第七节已点名这一处「交主 agent 另派」，不在这一题五份里。

### 1f 没打中：15 GiB 撞上限

服务对撞了自己上限的尝试（内存包装退 250，`scripts/hosts/run-with-memory-cap.sh` 文件头：scope 的 Result 是 oom-kill 或 memory.events 里 oom 不为 0）自己乘 1.5 重排，「4096M → 6144M → 9216M」（`task-server/task_server_lib.py:137`），至多 16 次（`task-server/task_server_lib_constants.py:499`）；static-check 子进程被杀、父进程退 2 也照 oom_kill 判成 250（同一份文件头，推的）。这条命令头一回没有记录时从 4G 起，要撞 4G、6G、9G、13.5G 四次才到 20.25G（推的；每次撞之前跑了多久没量）；有了记录之后照记录起。被服务打回 memory-cap-at-maximum 的落到上面 dry-run-way-out-outcomes 那一格。

## 二、删掉按文件比锚定行的那一条核之后，只有它拦得住的形

第一轮那一条（第二轮快照里 `merge_in_commands` 的 `LC_ALL=C awk … $2 in touched … | diff -u <记下的行> -`，`D/fix.diff` 第 88–92 行是删掉的原文）比的是「此刻表里锚在这一批改到的文件上的行」与开跑那一刻记下的，多一行、变一行就拦，不看锚点断没断；只读、几秒，排在合的那一条紧前。

### 2a 打中：transient-mask（改了又改回）

输入（照正文第二节第 2 条的例子）：开跑之后别的会话往表里追加 `attack3-transient-row`（锚在这一批改到的 alpha `merge_table.txt` 的原文 `1`；这一批把 `1` 改成 `2`）；--dry-run 跑补丁之后那一棵的 static-check 时（T1 之后），别的会话临时把主工作区的 `merge_table.txt` 改成 `5`，没打补丁那一棵跑完（T2 之后）又改回 `1`。
- 补丁之后那一棵（T1 拷的，那时还是 `1`，套补丁成 `2`）：`attack3-transient-row` 命中 0 次，ANCHOR；
- 没打补丁那一棵（T2 的活主工作区，那时是 `5`）：同一行同样命中 0 次，键（去掉行号之后的种类、变异名、说明）完全相同 → 算「补丁前就红」，照放行、只提醒「补丁前就红的 1 处」；
- 合：`merge_table.txt` 已改回 `1`，补丁改成 `2`，那一行命中 0 次。
第一轮那条核：`attack3-transient-row` 是开跑之后多出来、锚在改到的文件上的一行，照记下的一比就退非 0。核主工作区没变那一条：Tf 在改之前、Ta 在改回之后，两头一样，退 0。所以这一形只有删掉的那一条拦得住。
样本格把「临时改、又改回」做在假 static-check 里（D/attack3-cells.sh.part 的 `attack3-transient-static-check.py`：判补丁之后那一棵时改主工作区那份文件，判主工作区那一棵之后改回），时序照 `scripts/gate/apply_writer_patch_plan.py:1121-1122` 的先后。

### 2b 打中：窗口（同 1c）

第一轮那一条能放在合的那一条紧前，只隔一次排队；--dry-run 放不到那里（几百秒、15 GiB）。开跑之后追加、锚在改到的文件上的行，在 --dry-run 的 T0 之后才追加的，--dry-run 看不见（T0 合出的表里没有它），合之前再没有一条核看它。样本格同 window-table-append。

### 2c 没打中：锚点还中、变异失效

别的会话开跑之后追加一行锚在改到的文件上、原文照样恰好命中一次，而这一批的改动让那一行的替换文编不过（类型变了、UNDEFINED 判不到的那种）或变成等价变异。--dry-run 的 static-check 判不出（静态），第一轮那条核会拦（多出来一行就拦）。不算打中：开跑之前就在表里、锚在同一份文件上的行同样不判（开跑时的变异锚点核也是 static-check），第一轮那条核拦它只是顺带全拒，不是判了这件事；这类在提交时的 crates-mutation-replay 判。

## 三、--skip-apply 三格与「本次未核」

### 3a 没打中：倒出的补丁之前的树、三种情形分不分开

读 `skip_apply_anchor_check` 与 `skip_apply_anchor_baseline`（`scripts/gate/merge-and-verify-steps-lib.sh:429-494`）与 `MERGE_TABLE_REVERSE_PROGRAM`：代码那一侧在临时副本里照给的次序倒着逐份 `git apply --reverse`；表那一侧照倒着的次序逐份把追加的去掉、替换的改后换回前像、删掉的接回。试过的形（读码推）：
- 两份补丁目录先追加 R、后替换 R：倒着先把 R 换回追加时的原样、再去掉，倒得回；给反了次序，先去掉时找不到原样，记「没做成」（偏严、不漏）。
- 先删 R、后追加同名 R：倒着先去掉新 R、再接回旧 R，倒得回。
- 补丁目录全是空的：打「变异锚点核不做」，与「本次未核」（没给补丁目录）、「没做成」（倒不回）三句不同。
- 三格各看各的：skip-replace 要退 1 且点名 merge-kept-row 命中 0 次；skip-irreversible 要退 1 且有「变异锚点核没做成」；skip-none 要有「变异锚点核本次未核」。改坏一处只红对应那一格（第四节逐条推过）。

### 3b 自证分不出：「本次未核」改成记红、或不写为什么

调用处是「if ((skip_apply)); then skip_apply_anchor_check "$logs" "$tree" || reds=$((reds+1)); fi」（`scripts/gate/merge-and-verify-items-lib.sh:515`），判决改法第 2 条与文件头都要它只打一行、不记红（文件头那一句在 `scripts/gate/merge-and-verify-steps-lib.sh:426`）。现有那一格只判「if ! grep -qF '变异锚点核本次未核' "$out"; then」（`scripts/gate/merge-and-verify-selftest-lib.sh:924`），不看退出码、不看为什么。两处改坏（样本补丁里的 `ATTACK3_MUTATION`）：
- skip-none-records-red：打完那一行 `return 1`（记一项红、这一趟判红）；
- skip-none-reason-dropped：只打「变异锚点核本次未核」，不写为什么。
两处现有自证都照绿（推的）。样本格 skip-none-not-red：同一棵样本树、同样不给补丁目录，一趟不放 `ANCHOR_CHECK_SAMPLE_MARK`（自证里这一步整个不做）、一趟放（打本次未核），两趟退出码要相同，后一趟要有「摆不出补丁之前的树」。

## 四、新开关、改了红格的开关、--dry-run 七格

### 4a 没打中：七格都走到了锚点静态核

自证里的 `dry-run-driver.py` 载入真的 apply-writer-patch.py、调 `apply(补丁目录, 样本树, True, …)`，static-check 换成假的；判法：
- 六格必拦：`dry_run_blocked` 要 --dry-run 退 2 且「grep -qE "$4 [0-9]+ $3 " "$dry_out"」（`scripts/gate/merge-and-verify-selftest-lib.sh:766`）——点名那一行的 ANCHOR 或 RELEASE 只由 `static_check_report` 打（「✗ 补丁打上之后变异表有 N 处静态核红」之后那几行，制表符换成空格）。前置的核（补丁目录认不出、追加重名、前像对不上、三方合并冲突、走不到的理由、开工快照、上一次没换完）红时输出里没有这种行，判不成那一格；
- 一格必放行：dry-same-file 要退 0 且有「核过了（--dry-run」，只有 `apply_planned` 走完锚点静态核才打。
- 打出的那一条还另核：整行逐字等于 `submit -- python3 <仓根>/scripts/gate/apply-writer-patch.py --dry-run <日志目录>/merge-patch`、排在合的那一条之前（`merge_dry_run`）。
没核到的（不算打中，记限度）：打出的那一条在真仓上起时对的是这份脚本所在的主工作区（不带 --root），自证里改成对样本树；两条路只差 `apply()` 的 root 参数。

### 4b 没打中：新开关四条与改了红格的六条（读码逐条推，没复跑）

| 开关 | 改坏后哪一步变了 | 该红的格（推的） | 与文件头点名的 |
|---|---|---|---|
| merge-in-dry-run-dropped | --dry-run 那一条不打，`dry_printed` 是 0 | 七格都红在第一个分支「…应当拦住：这一趟应当退 0、合进主工作区那一段里打出合入前的 --dry-run…」与放行那一格的同一句 | 相同（七条） |
| anchor-skip-replacement-not-reversed | 倒表时替换不换回，补丁之前的表里还是换成的那一行，补丁前后都红 | skip-replace（退 0）；别的 --skip-apply 格的补丁目录里没有替换 | 相同 |
| anchor-skip-irreversible-passes | 倒不回照放行 | skip-irreversible（退 0） | 相同 |
| anchor-skip-no-directories-silent | 不打本次未核 | skip-none | 相同 |
| merge-in-commands-dropped | 合进主工作区那一段整个不打 | 原有几格加 --dry-run 七格 | 相同 |
| anchor-check-skipped、anchor-own-table-not-restored、anchor-mutation-only-skipped | 各自原有的格；skip-replace 走到 `anchor_check_new_breaks` 之后补丁前后都红或不核 | 多红 skip-replace；skip-irreversible 在倒表那一步就记红、不受影响 | 相同 |
| anchor-skip-apply-unchecked、anchor-skip-table-not-reversed | 前者在「没给补丁目录」那一支之后一进来就放行；后者不倒表、也就没有倒不回 | 各多红 skip-replace 与 skip-irreversible；skip-none 那一支排在开关之前，照绿 | 相同 |

正文说 merge-and-verify.sh 那一趟开关复跑在跑、还没结局；上表等它的「点名的格与实际红的格逐格相同」那一句坐实，不另交。

### 4c 自证分不出：--dry-run 那一条的出路

改坏 dry-run-way-out-dropped（样本补丁里，`scripts/gate/merge-and-verify-items-lib.sh:869` 那一行不打）：判决改法第 1 条要「代价写进出路：多一趟约 400–740 秒、峰值约 15 GiB」（`research/prompts/merge-anchor-check-tooling-r2-main-verification.md:38`），现有格里没有一格读那一行（`grep -n '400–740\|15 GiB\|上一条是合入前\|以合入那一刻的主工作区为基线\|重做核的那几条' scripts/gate/merge-and-verify-selftest-lib.sh` 零命中，退 1），现有自证照绿（推的）。样本格 dry-run-way-out-cost：--dry-run 那一条与合的那一条之间要有「400–740」与「15 GiB」。

## 五、apply-writer-patch.py 自证样本树的登记表

没打中。真跑 static-check 的两棵样本树（`anchor_root`、`path_root`）上五格（anchorbreak、anchorkept、anchorpre、anchorprenew、pathdep）要的是 ANCHOR 与 CHECKED：static-check 要被判那棵树里有登记表、表里有 crates-mutation-replay 那一行（`scripts/mutation/crates-mutation-rows.py:77-78`），样本那一行的拷贝范围（crates/、litmus/、Cargo.toml、Cargo.lock、.cargo/）就是这两棵树里有的东西；正文写主 agent 已量到「复跑了 52 条开关（1 份），自证红且点名的格与实际红的格逐格相同 52 条」，五格判得到。别的格走假 static-check（reachabilityhints、unreachablerowshift、releaserowshift）或不 --dry-run，不读这张表。
射程边上的线索（推的，不算打中）：RELEASE 那一支（被判那棵树里 crash-case: 行点名的目标、第五段没写 --release）在 apply-writer-patch.py 自证与合入后验证的 --dry-run 两格里都只走假 static-check（`fake-release-static-check.py`、`fake-registered-static-check.py`），真 crates-mutation-rows.py 读被判那棵树的 crash-case: 行这一条两份自证都没有真跑过；样本登记表里没有 crash-case: 行，补上也够不着（真 static-check 的 RELEASE 要真的测试目标）。

## 六、打中的四句与改法

四句（`.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种`）：这一题的「臂」是第二轮判决改法第 1 条那一版（合入前 --dry-run，删掉按文件比的核）与第一轮那一版（按文件比的核，排在合的那一条紧前）。

| 形 | 分不分辨臂 | 当时看不看得到 | 满足的分句（`research/prompts/merge-anchor-check-tooling-r2-main-verification.md:38`） |
|---|---|---|---|
| window-fingerprint | 分辨：第一轮那一版核与合只隔几秒，这一版隔几百秒加两次排队 | 看得到：核主工作区没变那一条挪到 --dry-run 之后就看得到 | 「在合入那一刻的主工作区上…」那一刻不是合入那一刻（核的不是锚点，是改法带出的次序） |
| window-table-append | 分辨：第一轮那一条排在合的紧前就拦 | 看得到：--dry-run 之后一条快核就看得到 | 同上；「罩住…自己追加的行」之外的「别的会话追加锚在改到的文件上的行」那一形 |
| transient-mask | 分辨：只有第一轮那一条拦 | 看得到：两棵树在同一刻取就看得到 | 「只拦这一批新弄断的」：新弄断的被当成补丁前就红 |
| baseline-skew | 分辨：只有这一版误拒 | 看得到：同上 | 「只拦这一批新弄断的」：没弄断的被拦 |
| redo-loop、出路两格、技能文档 | 分辨：第一轮那一版的「核的那几条」里有那一条快核 | 看得到 | 「代价写进出路」与「真合之前多一条」：出路与技能文档里那一条不在 |
| 3b、4c 两处分不出 | 不涉及臂 | — | 改法第 2、3 条：「打一行…并点名为什么」「改坏…各红在点名那一格」 |

改法（都只在读码上推过、被攻过零轮；「量过」一格都没有，样本格的读数全是「推的，待主 agent 跑」）：

| 改法 | 改哪 | window-fingerprint | window-table-append | transient-mask | baseline-skew | redo-loop | 出路两格 |
|---|---|---|---|---|---|---|---|
| G1 次序：--dry-run 排到最前，核主工作区没变与核变异行两条挪到它之后、贴着合的那一条 | items-lib `merge_in_commands` | 修（推的） | 不修 | 不修 | 不修 | 不修 | 不修 |
| G2 --dry-run 之后补一条快核：--dry-run 之前记下「表里锚在这一批改到的文件上的行、这一批自己追加与替换的行锚着的文件的 sha256、登记表的 sha256」，之后照记下的比，对不上只重交 --dry-run、不整批重交 | items-lib，同第一轮那条 awk 的写法、基线换成 --dry-run 之前那一刻 | 不修 | 修（推的） | 不修 | 不修 | 不修 | 不修 |
| G3 两棵树同一刻取：--dry-run 开头先照拷贝范围拷一棵补丁之前的树、在它上面合表与三方合并，补丁之后那一棵从它拷出再套补丁；基线跑这一棵，不跑活的主工作区 | apply_writer_patch_plan.py 的 `static_check_report`、`build_patched_tree`（这一轮五份之外） | 不修 | 不修 | 修（推的） | 修（推的） | 不修 | 不修 |
| G4 出路：「重做核的那几条」写明连同 --dry-run；--dry-run 那一条另写判不了、没做成、不是 2 的结局怎么办（查原因、重交同一条 --dry-run，不整批重交）；技能文档第 3 条补上 --dry-run | items-lib 第 869、872–874 行，merge-and-verify.sh 文件头第 307–311 行，SKILL.md | 不修 | 不修 | 不修 | 不修 | 修（推的） | 修（推的） |
| G5 自证：skip-none 那一格另判退出码与「摆不出补丁之前的树」；--dry-run 七格之外另判出路那一行在 | selftest-lib | — | — | — | — | — | 3b、4c 分得出（推的） |

G1 + G2 合起来才把窗口关到「合的那一条排队那一段」；那一段仍开着（真合自己不核锚点），要全关得让真合在换上之前照 --dry-run 记下的那几样 sha256 再核一遍（apply-writer-patch.py，推的、零轮）。G3 多一棵拷贝（`crates/` 约 59 MB，`du -sh` 量的源码，不含 target），两趟 static-check 照旧先后跑，峰值不变（推的）。

## 要主 agent 跑的命令

副本放 `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy`（D/.deliverables 已写一行「copy 交接」）；用完删。第 4 到 7 条用同一个副本，串行交。

1. 建副本（直接跑、不经服务），在仓根起：
   `python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy --with-test-inputs --with scripts=副本里起合入后验证的自证与攻方样本格 --with .claude/scripts=副本里自证读的预检脚本 --with .claude/singlefs-ai-sop=副本里自证读的上游脚本`
   看：退 0、末尾 `MINIMAL_COPY` 那一行。
2. 核副本里三份与 D/base 逐份相同（直接跑），在仓根起：
   `sha256sum /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy/scripts/gate/merge-and-verify-selftest-lib.sh /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy/scripts/gate/merge-and-verify-steps-lib.sh /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy/scripts/gate/merge-and-verify-items-lib.sh`
   看：依次是 d22f2ae5…、cf2af64a…、d76ba9df…（开工快照那三份）；不同就是主工作区又改过，补丁可能打不上，新派攻方腿照此刻重出。
3. 打样本补丁（直接跑），在仓根起：
   `git -C /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy apply -p2 /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/attack3.patch`
   看：退 0；三份 sha256 与本报告开头表里 D/new 那三行相同。
4. R0（样本格开、不改坏），`--cwd` 副本：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy -- env ATTACK3_CELLS=1 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看 `task-server/task log <号> --all`：「✗ 自检：」0 行（样本格不扰动现有格）；坐实打中的六行：「✗ 攻方样本 window-fingerprint：」「✗ 攻方样本 redo-loop：」「✗ 攻方样本 dry-run-way-out-outcomes：」「✗ 攻方样本 window-table-append：」「✗ 攻方样本 transient-mask：」「✗ 攻方样本 baseline-skew：」；不该有的两行：「✗ 攻方样本 dry-run-way-out-cost」「✗ 攻方样本 skip-none-not-red」；带「没走到」的那一格作废、不算坐实；「· 攻方样本 …」各行写着每条核的退出码、--dry-run 的退出码与合进去之后的红行，坐实时引它。`TASK_SERVER_RESULT` 的 `exit=1`。
5. R1 改坏「本次未核记红」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy -- env ATTACK3_CELLS=1 ATTACK3_MUTATION=skip-none-records-red python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：「✗ 自检：」0 行（3b 现有格分不出）；比 R0 多一行「✗ 攻方样本 skip-none-not-red：」。「· 攻方样本 skip-none-not-red」那一行两趟退出码相同的，这一处改坏样本格也分不出（记下，样本作废）。
6. R2 改坏「本次未核不写为什么」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy -- env ATTACK3_CELLS=1 ATTACK3_MUTATION=skip-none-reason-dropped python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：同第 5 条。
7. R3 改坏「--dry-run 那一条的出路不打」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy -- env ATTACK3_CELLS=1 ATTACK3_MUTATION=dry-run-way-out-dropped python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：「✗ 自检：」0 行（4c 现有格分不出）；比 R0 多一行「✗ 攻方样本 dry-run-way-out-cost：」。
8. 用完删副本（直接跑）：`rm -rf -- /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r3-opus/copy`，再删 D/.deliverables 里 copy 那一行。

R1 到 R3 只用「✗ 自检：」0 行判「现有格分不出」：样本格的失败行都以「✗ 攻方样本」起头，不混进去。耗时没估：一趟自证比不开样本格多六趟 `merge_case` 与四次 `dry-run-driver.py`；R0 到 R3 一共四趟。merge-and-verify.sh 那一趟开关复跑（正文说在跑）不另交，等它的结局坐实第四节 4b 那张表。

## 没打中的形状

- --dry-run 退 2「变异表有 N 处静态核红」那一支的出路照做做不做得成：做得成。整批重交之后新快照里有别的会话那一行或那份改了的文件，开跑时的变异锚点核照新快照点名，出路是重出 mutations-replacements.tsv、接在后面再交；这一形最多多一趟整批，收敛（推的）。
- 同一份文件里不受影响的追加：--dry-run 比的是锚点断没断，不按文件比；样本 dry-same-file 那一格判过。
- --dry-run 不拿换上锁：同一个仓有别的会话正在换上时，T1 拷到换了一半的树。换上是逐份改名、几毫秒一份，样本做不出可信的时序，没算打中（推的）。
- --skip-apply 倒表：追加接替换、删除接追加同名、次序给反、补丁目录全空四种，见 3a。
- 新开关四条与改了红格的六条：见 4b 那张表，逐条与文件头点名的相同。
- 15 GiB：服务自己重排，见 1f。
- 自证样本登记表：见第五节。
- 取样范围：五份修补换上之后的主工作区原文（与开工快照逐份相同）；读了 `merge_in_commands`、`skip_apply_anchor_check`、`skip_apply_anchor_baseline`、`reverse_check_in_order`、`anchor_check_new_breaks`、`MERGE_TABLE_REVERSE_PROGRAM` 那一处开关、自证里新加的十格，`apply_writer_patch_plan.py` 的 `plan_with_three_way`、`build_patched_tree`、`static_check_report`，`apply-writer-patch.py` 的 `apply`、`check_then_apply`、`apply_planned` 与自证里真跑 static-check 的五格，服务的内存重排与命令键一节，`.claude/skills/merge-verify/SKILL.md` 第 3 条。

## 这条腿自己的限度

- 全部打中都是读码推的；样本补丁只在 D/base 的拷贝上 `git apply --check` 过、三份 `bash -n` 过、假 static-check `py_compile` 过，没在任何树上跑过。
- transient-mask 与 baseline-skew 两格把「别的会话在 --dry-run 跑着的时候改」做在假 static-check 里（判补丁之后那一棵时改主工作区、判主工作区之后改回），靠的是 `static_check_report` 先跑补丁之后那一棵、后跑主工作区那一棵的次序；真 static-check 一趟 200–370 秒是正文的数折半，没量。
- window 两格模拟的是「照打出的次序做」：--dry-run 之前、之后各跑打出的只读核（`find`、`LC_ALL=C`、`sha256sum`、`cmp`、`diff` 起头的行）；主 agent 照技能文档做时根本不跑 --dry-run（1e 末条），那一条没有样本。
- redo-loop 与 dry-run-way-out-outcomes 两格判的是出路的字，读法另有一种（「核」含 --dry-run），1e 写了两种读法都站不住的理由；判决按哪一种读归主 agent。
- 改法 G1–G5 与「真合在换上之前再核一遍」都只在读码上推过、被攻过零轮；G3 改的是 `apply_writer_patch_plan.py`，不在这一轮五份里。
- 第一、二轮已攻过的角度没重攻；这一轮只攻正文第二节五条。

## 没做什么

- 不判正推腿那几格；不替主 agent 采纳；没往统一任务服务交任务、没起任何脚本与自证，判决要引的数都在「要主 agent 跑的命令」一节，等主 agent 跑。
- 没建仓副本（交主 agent 照第 1 条建）；草稿目录里只有三份脚本的 base/new、样本补丁、样本格原文与工具员修补的差（`fix.diff`），没有编译目录与仓副本（`du -sh` 整个草稿目录 1.1M）。
- 没写模型目录，没有 `rerun.sh`。
- 副本上量出的数（等主 agent 跑出来）不算入库装置上的数。
