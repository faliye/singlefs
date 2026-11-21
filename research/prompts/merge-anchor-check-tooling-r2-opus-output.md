# merge-anchor-check-tooling-r2：攻方腿（Opus）报告（2026-11-21）

被攻的是主工作区此刻的六份（`sha256sum -c research/prompts/merge-anchor-check-tooling-r2-snapshot/judged-sha256.txt` 八份都 OK，开工时核的）。没有写模型目录；样本是一份打在仓副本上的补丁，文件在草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/`（下称 D）：

| 文件 | sha256 |
|---|---|
| D/attack2.patch（三份脚本，211 行） | e49ba1b6d397a03a94defe2f060988fe6bc7f7a7ff39a3752758a8e18bc06ddd |
| D/new/scripts/gate/merge-and-verify.sh（打上之后） | d374fcfbb828f5799cb9d76747386728da6b7fa22a7ae39ebfacf0dd0398b12a |
| D/new/scripts/gate/merge-and-verify-steps-lib.sh | ba46231118c3ddfc233554d61bb7644502266825ffa5b7c3f1790e25c9321192 |
| D/new/scripts/gate/merge-and-verify-selftest-lib.sh | 21e9b938ff46eeb8a7d1389cba5e03aa67835694332dcb4dd66bc5c72c81b5c7 |

D/base/ 是打补丁之前那三份，sha256 与开工快照逐份相同；`git apply --check -v` 在 D/base 上三份都过。补丁只加两样：`ATTACK2_CELLS=1` 才跑的九格攻方样本（插在自证「没留快照的那一趟」那一格之后），`ATTACK2_MUTATION=<名>` 才生效的四处改坏。全部读数都是「推的，待主 agent 跑」，命令在第六节。

开工时另看到：`scripts/mutation/crates_mutation_rows_commands.py:67` 已是 `registry_root = judged_registry_root(root, os.environ)`（静态核读被判那棵树的登记表），即正文第一节说的「管变异键那一件」此刻已换进主工作区；第五节按换上之后的样子判。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 1a 改法第 1 条记的范围：这一批自己追加的变异行锚在补丁没改到的文件上，开跑之后那份文件被别的会话改了 | **打中**（推的，样本 own-row-concurrent-edit 两格待跑） | 记的路径只取 crates.patch；只改变异表的补丁指纹起点与锚定路径都是空的，合进主工作区那一段一条核都不打，真合不跑静态核 |
| 1b 记的范围：静态核的红不只看锚的那份文件（RELEASE 看登记表） | **打中**（推的，样本两格待跑） | 登记表不进指纹起点；开跑之后别的会话登记了这一批的行点名的目标，或这一批登记、别的会话追加锚在别处的行，合进去之后 RELEASE 红，核全退 0 |
| 1c 合之前那一条核：误拒 | **打中（轻，按改法的字面就是这样）**（推的） | 别的会话追加的行只要锚在改到的文件上就退非 0，锚点受不受这一批影响都整批重交 |
| 1d 合之前那一条核：核与真合之间 | 推不出新东西 | 核是直接跑、合是另交一条任务，中间追加的照样漏；指纹那一条是同一个窗口，改法没加新窗口 |
| 1e 只重跑红的那几项（带 --fingerprint-of）那一趟记的行 | 没打中 | 每一趟开跑都现记，锚点核每一趟都在那一趟的快照上做 |
| 2 RELEASE 去行号之后互相顶掉 | 没打中 | 键里留着变异名；比基线按 Counter 一处抵一处；追加同名的行打补丁时就拒 |
| 3a 带 --skip-apply 不给补丁目录 | **打中（轻）** | 变异锚点核一句不打就过去了，补丁打没打那一步至少写了「本次未核」 |
| 3b 倒不回是不是一律判红 | 没打中（读码；样本 skip-apply-irreversible 做成应当红的格） | 代码倒不回先在 check_patches_applied 退 1、不编；变异表倒不回记红 |
| 3c 补丁与别的改动叠在同一份文件、倒得回 | 没打中 | 别的改动两棵树都有，算基线；判的只是这一批 |
| 4a 日志全数、出路三处照着做 | 没打中 | 做得成 |
| 4b 自证：锚定行的记法改坏 | **分不出**（推的，样本 anchored-rows-unchanged 与两处改坏待跑） | 核那一类行的那一条只在追加之后跑一次、只要它退非 0；记多了、没排序都让它永远退非 0（每一批都误拒），现有格照绿 |
| 4c 自证：--skip-apply 变异表的替换不倒回 | **分不出**（推的，样本 skip-apply-replacement 与一处改坏待跑） | --skip-apply 的两格只有追加，没有替换 |
| 4d 自证：--skip-apply 倒不回照放行 | **分不出**（推的，样本 skip-apply-irreversible 与一处改坏待跑） | 没有倒不回的格 |
| 4e 十条新开关各红在点名的格 | 分得出（推的，逐条对过格的判据） | 第四节表 |
| 5 两棵临时树里的登记表；补丁改登记表时各是改前改后 | 没打中（读码） | 补丁之前的树换回原样、补丁之后的树套补丁；skip-apply 与 --dry-run 两路同样 |

## 一、改法第 1 条：记的范围与合之前那一条核

### 1a 这一批自己的行锚在没改到的文件上（打中，推的）

输入（样本 own-row-concurrent-edit，两种）：
- mutonly：补丁目录只有一份 `mutations-append.tsv`，一行 `attack2-own-mutonly-row`，锚在 `crates/beta/src/merge_extra.txt` 的原文 `1`。开跑、验绿（锚点核过）之后，别的会话把那份文件改成 `2`。
- with-code：补丁目录另有 crates.patch 改 alpha 的 `merge_table.txt`；追加的行锚在 demo 的 `merge_demo.txt`（demo 与 alpha 没有依赖关系）。开跑之后别的会话改了 `merge_demo.txt`。

一步一步：
1. 记的路径只取 crates.patch：`scripts/gate/merge-and-verify.sh:2478` 的 `touched_files` 由 `library.patch_directory_paths(directory)` 算，它只读补丁目录里的 crates.patch（`scripts/gate/lib_rebuilt_packages.py:226` `path = os.path.join(directory, PATCH_FILE_NAME)`）。变异表文件第二段的文件不进锚定路径，也不进指纹起点（同一个函数取起点，`scripts/gate/merge-and-verify.sh:2489`）。
2. mutonly 那一种起点是空的，合进主工作区那一段只打「（补丁改到的包与文件开跑那一刻要验的树里一份都没有，不用核主工作区没变）」（`scripts/gate/merge-and-verify-items-lib.sh:857`）；锚定路径文件是空的，核锚定行的那一条不打（`scripts/gate/merge-and-verify-items-lib.sh:866` 判 `-s`）；没有替换、删除，核变异行那一条也不打。with-code 那一种指纹罩 alpha 与依赖 alpha 的 beta，罩不到 demo；锚定路径只有 `merge_table.txt`，那一条照退 0。
3. 真合不跑静态核：`scripts/gate/apply-writer-patch.py:1013` 只在 `if dry_run:` 那一支调 `static_check_report`，合进主工作区的那一条不带 `--dry-run`。
4. 合进去之后那一行原文命中 0 次，到提交时门禁 mutation-tables 那一格才红。

四句：
- 分不分辨：这一格只打改法第 1 条（判决「三、改法」第 1 条）做出来的那一形，别的改法碰不到它；不是各改法共用前提的问题。
- 判别子看不看得到：看得到。开跑那一刻补丁目录里的 `mutations-append.tsv`、`mutations-replacements.tsv` 都在手边，第二段写着锚的文件。
- 满足的是哪一句：正文第二节第 1 条「记的范围对不对（这一批改到的文件怎么算…）」；判决第 1 条字面只罩「别的会话只追加、锚在这一批改掉的原文上的变异行」，这一格是另一形（这一批自己的行，锚着的代码开跑之后被改），不归它的字面，归「记的范围」那一问。
- 跑前条款的改法在这几格上还中不中：判决「三、改法」第 1 条照写的做（只记 crates.patch 改到的文件）在这两格上照漏；第 3 条（--skip-apply）不相干。

我提的改法（只在读码上推过、没实现没跑、被攻过零轮）：
- 改法 R2-1：开跑那一刻把这一批 `mutations-append.tsv` 与 `mutations-replacements.tsv` 新行（前六段）第二段的文件也算进指纹起点（不落在包里的单份文件那一种照算），合进主工作区那一段头一条就罩住它们。代价：多罩几份文件，别的会话改它们照样整批重交。
- 改法 R2-2：合进主工作区那一段在真合那一条之前多打一条 `apply-writer-patch.py --dry-run <日志目录>/merge-patch`（不带 --root，判主工作区此刻）：它拿主工作区此刻当基线、套 merge-patch 再跑一趟静态核（`static_check_report`），1a、1b 与改法第 1 条那一形都在里面，awk 那一条可以不要。代价：多两趟 static-check，量的一趟（补丁前后各一次）约 423 秒、峰值 15.20 GiB（判决「一、腿与核查」第 3 条）；窗口缩到 --dry-run 与真合之间。

| 改法 | own-row-concurrent-edit（mutonly / with-code） | own-row-concurrent-registration | registration-concurrent-row | 改法第 1 条那一形（concurrent） |
|---|---|---|---|---|
| 判决改法第 1 条（现在的 awk 那一条） | 不中（推的，待主 agent 跑） | 不中（推的，待主 agent 跑） | 不中（推的，待主 agent 跑） | 中（推的；自证格在） |
| R2-1 | 中（推的） | 不中（推的：登记表照不进起点） | 不中（推的） | 不相干 |
| R2-2 | 中（推的） | 中（推的） | 中（推的） | 中（推的） |

### 1b 静态核的红不只看锚的那份文件（打中，推的）

现状：静态核读被判那棵树的登记表定 RELEASE（`scripts/mutation/crates_mutation_rows_commands.py:67`；判法 `scripts/mutation/lib_release_only_tests.py:8` 写「crash-case 目标」是登记表里 `crash-case:` 行的 `test=<包>:<测试目标>:<用例函数>` 点到的那一对）。登记表不进指纹起点（`scripts/gate/merge-and-verify.sh:2450` 跳过 `registration_tables`），锚定行那一条只认第二段落在改到的文件上的行。两形：

- own-row-concurrent-registration：这一批只追加一行，第五段 `-p beta --test case_b`、没写 `--release`，锚得好好的。开跑之后别的会话往 `.claude/gate.d/stage-inputs.tsv` 加一行 `crash-case:attack2-case`、`test=beta:case_b:case_function`。合进主工作区那一段一条核都不打（同 1a 第 2 步），合进去之后那一行 RELEASE 红。
- registration-concurrent-row：这一批的 crates.patch 只往登记表加上面那一行（实现员把新崩溃枚举用例的登记行写进 crates.patch，`scripts/gate/merge-and-verify.sh` 文件头「登记表」那一条）。开跑之后别的会话追加一行点名 `-p beta --test case_b`、锚在 beta 的 `merge_extra.txt`。锚定路径是登记表本身，表里没有行的第二段是它，那一条退 0；指纹起点没有（登记表跳过）。合进去之后那一行 RELEASE 红。

同一类（读码，没做样本）：UNDEFINED 按被改文件所在包的标识符集合判（`scripts/mutation/crates_mutation_rows_static.py` 的 `replacement_undefined_identifiers_of_rows` 文件串：「被改文件与它所在的包的标识符集合」，转述），这一批删掉包里某份文件的一个定义，开跑之后别的会话追加的行锚在同包另一份文件、替换文用到它，同样漏；UNREACHABLE 走调用图，同理。

四句：同 1a，判别子看得到（登记表与补丁都在手边）；归「记的范围」那一问；判决改法第 1 条在这两格上不中，R2-2 中（表在 1a）。

### 1c 合之前那一条核的误拒（打中，轻，推的）

别的会话在这一趟开跑之后追加一行，第二段是这一批改到的文件、锚在这一批没碰的地方（锚点照样命中 1 次），awk 那一条照样退非 0，出路是以合入那一刻为基线整批重交（`scripts/gate/merge-and-verify-items-lib.sh:868` 那一句）。整批重交是一趟完整的合入后验证（锚点核一项就约 423 秒、峰值 15.20 GiB）。改法第 1 条的字面就是「多出来或变了的退非 0」，这一格是代价不是缺陷；R2-2 换掉 awk 那一条后它不再误拒（--dry-run 只拦真新红）。频率没量：要看多少会话同时往同一份文件追加变异行。

### 1d 核与真合之间（推不出新东西）

合进主工作区那一段写「核的那几条只读、直接跑」（`scripts/gate/merge-and-verify-items-lib.sh:844`），真合是另交一条 `task-server/task submit -- python3 …/apply-writer-patch.py <merge-patch>`（同文件第 870 行打出）。两步之间别的会话追加的行照漏；指纹那一条一直是这个窗口，改法第 1 条没有加宽它。R2-2 把窗口缩到 --dry-run 与真合之间，消不掉。

### 1e 只重跑红的那几项（没打中）

带 `--fingerprint-of` 的那一趟先核上一趟的指纹，再照常 `record`（`scripts/gate/merge-and-verify.sh` 开跑那一段），锚定行记的是这一趟开跑那一刻；锚点核每一趟都在这一趟的快照上做（`scripts/gate/merge-and-verify-items-lib.sh:507`）。上一趟与这一趟之间追加的行进了这一趟的快照、被锚点核判过。试过：两趟之间追加、两趟之间删了又加回同样的一行（删了又加回逐字相同，开跑记的与合前重算的一样，锚点核在这一趟快照上判过，不漏）。

## 二、改法第 2 条：RELEASE 去行号之后互相顶掉（没打中）

- 键：`scripts/gate/apply_writer_patch_plan.py:1061` 对 `STATIC_CHECK_ROW_NUMBER_KINDS` 里的种类只去掉第二段（行号），留下种类、变异名与依据。真 static-check 打 RELEASE 的那一行是 `RELEASE\t{line_number}\t{name}\t…`（`scripts/mutation/crates_mutation_rows_commands.py:142`），变异名在键里。
- 比基线：`scripts/gate/apply_writer_patch_plan.py:1137` 用 `collections.Counter`，补丁之后每一处红只抵掉基线上同键的一处；同键的行多出一处就是一处新红。不是集合比，所以「两行一样的键」不会少报。
- 两行键相同要求变异名相同：这一批追加同名的行，打补丁时 `mutation_edits` 就拒（`scripts/gate/apply_writer_patch_plan.py:999` 「追加的…表里已经有了」）；替换保留名字，同一行补丁前后都是 RELEASE 且依据相同，就是那一行补丁前就红，不算新红是对的。

试过的形状：补丁前一行 RELEASE 红、补丁把它换成另一行同名同依据（照「补丁前就红」放行，对）；补丁前一行 RELEASE 红、补丁删掉它再追加同名的一行（追加同名被拒，走不到）；同一行补丁前后依据变了（依据在键里，判新红，偏严不漏）；两行都没名字（MALFORMED 才有「（变异名空着）」，不是 RELEASE）。没打中的范围只是 RELEASE 这一种；MALFORMED 两行都空名、原因相同的，一处修好一处新坏会抵掉（推的，与改法第 2 条无关，第一轮之前就这样）。

## 三、改法第 3 条：--skip-apply 那一趟

### 3a 不给补丁目录（打中，轻）

`scripts/gate/merge-and-verify.sh:2582` 放行「带 --skip-apply、一个补丁目录都没给」；`skip_apply_anchor_check` 第一句 `((${#CHECK_PATCH_DIRS[@]})) || return 0`（`scripts/gate/merge-and-verify-steps-lib.sh:430`），一个字都不打就返回。同一趟里补丁打没打那一步写了「补丁打没打本次未核：带了 --skip-apply 而没给补丁目录」，锚点核没有对应的一句。按 `.claude/skills/merge-verify/SKILL.md` 第 30 行，--skip-apply 给主 agent 自己写的探索性改动用；不给补丁目录时这一条路照样一次都不核，判决第 4 条说的那一形对这一种跑法没变。样本 skip-apply-no-directories：断言输出里没有「变异锚点核」这几个字。

四句：分辨（只打改法第 3 条）；判别子看得到（`CHECK_PATCH_DIRS` 是空的）；满足正文第二节第 3 条「是不是一律判红、不放行」的反面：没核也没说；改法第 3 条的字面「带 --skip-apply 的那一趟……照样核」在这一格上不中。我提的改法（零轮）：没给补丁目录时打一句「变异锚点核本次未判：带 --skip-apply 没给补丁目录，补丁之前的树摆不出来」，汇总里记成没判，不当过。

### 3b 倒不回是不是一律判红（没打中）

- 代码倒不回：`check_patches_applied` 在 verify 之前就照同一份 `reverse_check_in_order` 倒一次，倒不回 `die … 1`、一个都不编（`scripts/gate/merge-and-verify-steps-lib.sh` 的 `check_patches_applied` 那一段）；锚点核里再倒一次用的是同一棵快照、同一组补丁，结局相同，走到锚点核时一定倒得回。
- 变异表倒不回：`MERGE_TABLE_REVERSE_PROGRAM` 找不到追加的行、找不到改后的行、删掉的行名还在，都记一处（`scripts/gate/merge-and-verify.sh:2231` 那一种），退 1；`skip_apply_anchor_check` 打「✗ 变异锚点核没做成」返回 1（`scripts/gate/merge-and-verify-steps-lib.sh:444`），记一项红。
- 样本 skip-apply-irreversible（补丁目录追加的那一行在表里已被改成另一行）做成「应当红」的格，第四节 4d 拿它证改坏分不分得出。

### 3c 补丁与别的改动叠在同一份文件（没打中）

倒得回时：倒的是这一批补丁的段，别的改动留在倒回的文件里，补丁之前、之后两棵树都有，算基线；合起来的补丁照「倒回的原样 → 快照此刻」现算，只剩这一批。别的改动弄断的行两棵树都红，不算这一批的，这是改法第 3 条要的。试过：别的改动在补丁段之外几行（git apply 认偏移，倒得回，判对）；别的改动压在补丁段的上下文里（倒不回，3b）；补丁目录由 `scripts/gate/patch-of-the-current-tree.py` 按此刻现算（文件头写它「主工作区上与补丁无关、但落在同一份文件里的别人的改动也在里面」，转述），这时别人的改动被当成这一批的，弄断的行照样点名，偏严不漏。

轻的一处（推的，没做样本）：`patch-of-the-current-tree.py` 只留「整行还在此刻树里」的追加行（它文件头第 2 步，转述），这一批追加之后又被主 agent 改过的行不进新补丁目录，两棵树都有它，它要是锚点不中，算补丁前就红、放过。只影响「这一批自己的行写错」那一种，不影响别批的行被弄断。

## 四、改法第 4、5 条：日志全数、出路、新格与新开关

### 4a 日志全数与出路三处（没打中）

- 日志全数：`MERGE_ANCHOR_CHECK_PROGRAM` 调 `static_check_report(…, shown=None)`，拒绝说明一行不省，整份落进 `anchor-check.log`；标准输出只列前 `ANCHOR_CHECK_SHOWN_LINES` 行，再从日志首行「✗ 补丁打上之后变异表有 N 处」取 N 写「新红一共 N 处」。核没做成或判不了时那一句不打，日志里是原因，读得到。
- 出路：别批的行照快照重出替换（前像是快照那一行）、整段删掉的写进 `mutations-delete.txt`（前像同样取快照）、没留快照的带 `MERGE_AND_VERIFY_KEEP_COPY=1` 重交，三条照做都打得上：apply-writer-patch.py 比前像比的是主工作区此刻那一行，主工作区那一行没变就对得上。这一批自己追加的行锚点不中时，「快照里那一行」是追加之后的那一行，新补丁目录接在后面替换它，merge-patch 合完是一行追加，也做得成。--skip-apply 那一趟的出路是直接改表、照原命令再验，对。
- 槽位那一种（这一轮射程外的槽位改动）没留快照时快照其实留在槽里、会被下一趟占槽盖掉，出路那一句写「上面的快照收尾已删」不准，但照它重交照样做得成，不算打中。

### 4b–4d 自证分不分得出（三处分不出，推的）

| 改坏（`ATTACK2_MUTATION=`） | 改哪一处 | 现有自证 | 该红的样本格 |
|---|---|---|---|
| `anchored-rows-all` | `MAIN_TREE_FINGERPRINT_PROGRAM` 的 `anchored_rows` 记下表里全部的行、不按改到的文件筛 | 照绿（推的）：核锚定行的那一条只在 concurrent 那一格追加之后跑一次（`scripts/gate/merge-and-verify-selftest-lib.sh:707`），只要它退非 0；那一格开跑那一刻锚在改到的文件上的行是 0 行（样本表只有锚在 alpha `lib.rs` 的 `merge-kept-row`，补丁改 `merge_table.txt`），记多记少它都退非 0 | anchored-rows-unchanged 红 |
| `anchored-rows-unsorted` | 同一处去重但不排序（与 awk 那一条 `sort -u` 的次序不同） | 照绿（推的），理由同上 | anchored-rows-unchanged 红 |
| `skip-replacements-not-reversed` | `MERGE_TABLE_REVERSE_PROGRAM` 不把替换倒回前像 | 照绿（推的）：两格 --skip-apply 一格只改代码、一格只追加 | skip-apply-replacement 红（补丁之前的表里也是改后那一行，红两边都有，退 0） |
| `skip-irreversible-passes` | `skip_apply_anchor_check` 倒不回时照放行 | 照绿（推的）：没有倒不回的格 | skip-apply-irreversible 红 |

前两处在真跑里的后果：核锚定行的那一条永远退非 0，每一批都被叫「整批重交」，重交一趟照样退非 0——合入后验证这一条路走不通。后两处：--skip-apply 那一趟这一批换坏的行、倒不回的情形照放行。

另有两处没做样本（推的）：`skip_apply_anchor_baseline` 里补丁目录倒着排的那一段改成正着排（只在同一批两份补丁目录一份追加、一份替换同一行时才见差别）；`anchor_check_new_breaks` 里 `if ((skip_apply))` 那一段出路删掉（--skip-apply 两格只判退出码与 ANCHOR 行，不看出路）。两处现有格都分不出。

我提的收法（零轮）：把 anchored-rows-unchanged、skip-apply-replacement、skip-apply-irreversible 三格收进自证常跑的格，四处改坏各配一条弄坏开关。

### 4e 十条新开关（分得出，推的）

逐条对过点名的那一格的判据（`scripts/gate/merge-and-verify-selftest-lib.sh` 第 694 行起那一段），每条开关下那一格的断言都会落空：

| 开关 | 点名的格 | 那一格为什么红 |
|---|---|---|
| anchored-rows-check-dropped | 开跑之后追加的行 | 没有 `LC_ALL=C awk` 起头那一条，`anchored_check_rc=missing` |
| anchor-check-release-row-number-kept | RELEASE 行号前移 | 假 static-check 按行在文件里的位置编号，删一行之后 RELEASE 那一行号少 1，连号比判新红、退 1 |
| anchor-own-table-not-restored | 自己追加两格、skip-apply 自己追加 | 补丁之前的表就是快照的表，自己那一行两边都红，退 0 |
| anchor-created-files-kept | 新建文件 | 补丁之前的树里已有那份文件，新建段套不上，「没做成」退 1 |
| anchor-mutation-only-skipped | 只改变异表、skip-apply 自己追加、RELEASE 前移 | 三份补丁（含倒出来的 anchor-skip-patch）都没有 crates.patch，核不做、退 0 |
| anchor-skip-apply-unchecked | skip-apply 两格 | 不核、退 0 |
| anchor-skip-table-not-reversed | skip-apply 自己追加 | 补丁之前的表是快照的表，退 0（只改代码那一格没有变异表文件，碰不到这一处） |
| anchor-check-log-truncated | 新红多过列的行数 | 日志里只有 20 处 `merge-many-row-` |
| anchor-way-out-delete-dropped | 补丁弄断别批的行 | 输出里 `mutations-delete.txt` 只出现在这一句出路里（红了不打合进主工作区那一段） |
| anchor-way-out-keep-copy-dropped | 没留快照 | 「env MERGE_AND_VERIFY_KEEP_COPY=1 整条重交」只出现在这一句出路里（`merge_in_commands` 那一句是「、整条重交」，红了也不打） |

结局以主 agent 交的 `code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh` 为准（正文第一节说还没出）。

## 五、管变异键那一件换上之后：两棵临时树里的登记表（没打中，读码）

- 合入后验证（不带 --skip-apply）：补丁之前的树照 `static_check_tree_items(tree)` 取拷贝范围（`scripts/gate/merge-and-verify.sh:2133`，`tree` 是快照），`STATIC_CHECK_DECLARED` 有 `.claude/gate.d/stage-inputs.tsv`，快照是整棵 rsync（`take_snapshot` 只排 .git 与编译目录），所以在里面。补丁的 crates.patch 改到登记表时，它在 `merge-baseline.paths` 里、`inside()` 认它，换回原样目录里补丁之前那一份。补丁之后的树由 `build_patched_tree(before, …)` 在补丁之前的树的拷贝范围上建（`scripts/gate/apply_writer_patch_plan.py:1094`），登记表同样在，再套 merge-patch 的 crates.patch（登记表那一段在范围里、保留），就是改后那一份。
- --skip-apply：登记表在 crates.patch 里时 `reverse_check_in_order` 把它拷进临时副本倒回，原样目录里是改前那一份；补丁之后是快照此刻。
- `apply-writer-patch.py --dry-run`：补丁之前直接判 `root`（主工作区此刻，改前那一份）；补丁之后 `build_patched_tree(root, …)` 的拷贝范围带登记表、`git apply` 整份补丁，改后那一份。
- 补丁不碰登记表时两棵树都是快照（或主工作区）那一份，静态核两边读同一张表。

没打中的范围：只核了「在不在、是改前还是改后」。登记表在开跑之后被别的会话改的那一形归 1b。

射程边上看到的一处（推的，没做样本；第一轮判过「补丁之前的树摆法」没打中，这里只记线索）：补丁之后的树的拷贝范围是照补丁之前的树现算的（`build_patched_tree` 对 `before` 调 `static_check_tree_items`），merge-patch 的段却是照快照（补丁之后）的拷贝范围留的。补丁新加一条测试、测试里读一份原来不在拷贝范围里的文件，又改了那份文件：那一段留下了，补丁之后的树里却没有那份文件，`git apply` 套不上，判「没做成」，出路照做重交照样红。补丁删掉一个包目录连同它在 `Cargo.toml` 里的成员时，补丁之前的树里那个包目录不在范围里拷不进来、`Cargo.toml` 却换回了点名它的原样，static-check 的 path 依赖那一步判不了（同一类，推的）。

## 没打中的形状

| 形状 | 取样 | 结论 |
|---|---|---|
| Python `anchored_rows` 与 awk 判据对不对得上 | 空行、只有一段的行、`#` 起头、`\r` 收尾、末行没换行、重复行 | 两边逐条同（Python 按字节、`sort -u` 在 C 区域按字节） |
| 锚定路径的写法 | 改名两侧、新建、删除、Git 引号路径 | `patch_paths` 新旧两侧、还原引号；落在表第二段的写法与 crates.patch 里一致才认，样本没造写法不同的 |
| 记了之后、拷快照之前追加 | 一行 | 快照里有、锚点核判过；记下的没有，合前那一条退非 0，偏严 |
| RELEASE 键 | 第二节四种 | 不顶 |
| --skip-apply 两份补丁目录先追加后替换同一行 | 一种 | 倒序摆回，读码对 |
| 改法第 6 条耗时注释 | `scripts/gate/merge-and-verify-steps-lib.sh` 开始那一句与文件头 | 与判决「一、腿与核查」第 3 条的数相同 |

## 这条腿自己的限度

- 全部打中都是读码推的；样本补丁没在任何树上跑过（只 `git apply --check` 过、`bash -n` 三份过、三段内联 Python 与假 static-check `py_compile` 过）。样本格里假 static-check 的 RELEASE 是照 `lib_release_only_tests.py` 文件头那一句仿的，不是真判法；1b 在真判法上成不成立，靠真 static-check 读被判那棵树的登记表这一点（`scripts/mutation/crates_mutation_rows_commands.py:67`），没在真仓上跑。
- registration-concurrent-row 与 own-row-concurrent-registration 两格的样本树多一份 `.claude/gate.d/stage-inputs.tsv`、补丁只改登记表，合入后验证别的步骤在这种树上退几没核过：格里接受退 0 或 3，别的退出码打「没走到」，那一格作废。
- 1c 的频率、R2-2 的真实耗时都没量。
- 这一轮只攻正文第二节五条；第一轮判过没打中的角度没重攻，射程边上那一处只记线索。

## 六、要主 agent 跑的命令

副本放 `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy`（D/.deliverables 已写一行「copy 交接 …」）；用完删。第 4 到 8 条串行交（同一个副本、同一个自证的临时目录名）。

1. 建副本（直接跑、不经服务），仓根起：
   `python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy --with-test-inputs --with scripts=副本里起合入后验证的自证与攻方样本格 --with .claude/scripts=副本里自证读的预检脚本 --with .claude/singlefs-ai-sop=副本里自证读的上游脚本`
   看：末尾 `MINIMAL_COPY` 那一行、退 0。
2. 核副本里三份与 D/base 逐份相同（直接跑），仓根起：
   `sha256sum /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy/scripts/gate/merge-and-verify.sh /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy/scripts/gate/merge-and-verify-steps-lib.sh /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy/scripts/gate/merge-and-verify-selftest-lib.sh`
   看：三个哈希依次是 d6d2081c…、019c9f50…、9ab68864…（开工快照那三份）；不同的说明主工作区又被改过，补丁可能打不上，打不上交回新派攻方腿照此刻重出。
3. 打样本补丁（直接跑），仓根起：
   `git -C /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy apply /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/attack2.patch`
   看：退 0；三份的 sha256 与本报告开头表里 D/new 那三行相同。
4. R0（样本格开、不改坏），`--cwd` 副本：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy -- env ATTACK2_CELLS=1 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看 `task-server/task log <号> --all`：「✗ 自检：」0 行（样本格不扰动现有格）；坐实打中的五行：「✗ 攻方样本 own-row-concurrent-edit（mutonly）：」「✗ 攻方样本 own-row-concurrent-edit（with-code）：」（1a）、「✗ 攻方样本 own-row-concurrent-registration：」「✗ 攻方样本 registration-concurrent-row：」（1b）、「✗ 攻方样本 skip-apply-no-directories：」（3a）；该绿的三行是「·」：anchored-rows-unchanged、skip-apply-replacement、skip-apply-irreversible；`TASK_SERVER_RESULT` 的 `exit=1`。带「没走到」的那一格作废、不算坐实；「·」的 own-row-concurrent-edit 这类行里写着合进主工作区那一段每条核的退出码与合进去之后的红行，坐实时引它。
5. R1 改坏「锚定行记下全部的行」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy -- env ATTACK2_CELLS=1 ATTACK2_MUTATION=anchored-rows-all python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：「✗ 自检：」0 行（4b 现有格分不出）；「✗ 攻方样本 anchored-rows-unchanged：」有。
6. R2 改坏「锚定行不排序」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy -- env ATTACK2_CELLS=1 ATTACK2_MUTATION=anchored-rows-unsorted python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：同第 5 条。
7. R3 改坏「--skip-apply 替换不倒回」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy -- env ATTACK2_CELLS=1 ATTACK2_MUTATION=skip-replacements-not-reversed python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：「✗ 自检：」0 行（4c）；「✗ 攻方样本 skip-apply-replacement：」有。
8. R4 改坏「--skip-apply 倒不回照放行」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy -- env ATTACK2_CELLS=1 ATTACK2_MUTATION=skip-irreversible-passes python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：「✗ 自检：」0 行（4d）；「✗ 攻方样本 skip-apply-irreversible：」有。
9. 用完删副本（直接跑）：`rm -rf -- /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-opus/copy`，再删 D/.deliverables 里 copy 那一行。

R1 到 R4 只用「✗ 自检：」0 行判「现有格分不出」：样本格的失败行都以「✗ 攻方样本」起头，不混进去。耗时没估：一趟合入后验证自证多九趟 `merge_case`（第一节到第四节那九格），比没开样本格的长；R0 到 R4 一共五趟。

## 没做什么

- 不判辩方那几格；不替主 agent 采纳；没往统一任务服务交任务、没起任何脚本与自证，判决要引的数都在第六节，等主 agent 跑。
- 没建仓副本（第六节第 1 条交主 agent）；草稿目录里只有三份脚本的 base/new 两份拷贝、样本补丁、样本格原文与工具员修补的差（`fix.diff`），没有编译目录与仓副本。
- 没写模型目录，没有 rerun.sh。
- 改法 R2-1、R2-2 与 3a、4b–4d 的收法只在读码上推过，被攻过零轮。
