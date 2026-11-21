# merge-anchor-check-tooling-r1 攻方腿（Opus）报告（2026-11-20）

被判对象：主工作区四份 `scripts/gate/merge-and-verify*.sh` 与 `scripts/gate/apply-writer-patch.py`（开工时 sha256 与 `research/prompts/merge-anchor-check-tooling-r1-snapshot/judged-sha256.txt` 五行逐个对上，现查）；`static_check_report` 在 `scripts/gate/apply_writer_patch_plan.py`。前几轮判决：无。

没有写模型目录：这一题是治理文件，样本做成一份打进仓副本的补丁（攻方样本格加三处按环境变量生效的改坏），放草稿目录 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/`：

| 文件 | sha256 | 是什么 |
|---|---|---|
| `attack-anchor-check.patch` | `8fe9860f724ec061ad4cee595cf9c7f6ae723c7a88fccd96719f6b09f260a7d4` | 对三份脚本的补丁（`patch -p1 --dry-run` 对 `base/` 过，现跑） |
| `tools/attack-cells.part` | 见下面命令 | 插进 selftest-lib 的五格原文 |
| `tools/make-attack-patch.py` | 见下面命令 | 从 `base/` 生成 `new/` 的草稿脚本 |
| `base/scripts/gate/` | 与主工作区同（开工时 sha256） | 改前原样 |

自己建副本被权限拒（`minimal-copy.py` 那一条被自动模式拒），副本由主 agent 建，命令在「要主 agent 跑的命令」。全部读数都是「推的，待主 agent 跑」，只有第五节那一个时长是从现有日志文件的修改时刻量的。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 判法：补丁之前的树（换回原样、删新建、只留范围内的段） | 没打中 | 读码核过，见第一节 |
| 判法：多份补丁合成一份之后才核会不会盖掉 | 没打中 | 合起来的就是要落进去的那一份；盖得住的只有「锚点挪到别处仍命中 1 次」，那是计数判据本身的限度，单份也一样 |
| 判法：基线是开跑那一刻的快照，开跑之后别的会话只追加的变异行 | **打中** | 合进主工作区那一段的核故意放过只追加的行，锚点核不再做，合进去之后那一行命中 0 次（样本 concurrent-append） |
| 判法：补丁前就红的判成新红（RELEASE 行号前移） | **打中** | `RELEASE` 不在去行号的几种里，这一批删一行就让它下面所有补丁前就红的 RELEASE 行变成新红（样本 release-shift） |
| 判法：登记表改动看不见 | **打中（推的）** | static-check 的 RELEASE 与拷贝范围读主工作区那一份登记表，不读被判的树；补丁登记新的 crash-case 带来的 RELEASE 新红两棵树都看不见 |
| 走不到：`--skip-apply` | **打中** | 这一条路一次都不核；按 SKILL 它给主 agent 自己写的改动用，那类改动弄断别批锚点照旧只有 mutation-tables 格报 |
| 走不到：合起来没有改动、只改变异表、样本树之外的自证跑法 | 没打中 | 只改变异表的照核（四份文件任一不空就做）；`inside_selftest` 在真跑里走不到 |
| 核没做成一律判红 | 没打中 | 程序退非 0 一律记红，被杀（137）、static-check 退 -9 或超时也红 |
| 出路：照着做能不能做成 | **打中（轻）** | 新红超过 20 处只点名前 20 处；代码被删的行出路只写重取替换文不写删行；没带 `KEEP_COPY` 的那一趟出路里的快照已删 |
| 出路：红了不打合进主工作区那一段 | 没打中 | `reds` 记一项 → `verify` 退 1 → 不打 |
| 自证：两格分不分得出核在不在、基线用没用 | 分得出 | `anchor-check-skipped` 红两格、`anchor-check-baseline-ignored` 红 pre-broken 一格，与文件头写的一致（推的） |
| 自证：把核改坏照样绿 | **分不出（三处）** | 只改变异表的不核、变异表不换回原样、补丁新建的不删：现有 167 格照绿（推的，待主 agent 跑证红） |
| 复用 `static_check_report`，没另写第二份判法 | 没打中 | 程序只摆树、滤段、调它 |
| 代价 | **对不上** | 代码与报告写两趟「各约半分钟到一分钟」；现有唯一一趟真跑的日志修改时刻相差 625 秒（量的，一个样本）；每趟上限 900 秒 |

## 一、判法

### 1.1 补丁之前的树（没打中）

读码核过：程序 `scripts/gate/merge-and-verify.sh:2000-2008` 对 `merge-baseline.paths` 里落在拷贝范围的每条路径，原样目录里有就拷回、没有就删（补丁新建的）；变异表只在补丁目录带变异表文件时进路径单子（`scripts/gate/merge-and-verify.sh:1823`），这时同样换回原样。补丁之后那棵树由 `build_patched_tree(before, merged, patch)` 在「补丁之前的树」的拷贝上套 merge-patch 的 crates.patch、写快照此刻的变异表。补丁之前、之后两棵树只差这一批补丁，判的就是这一批。

试过的形状：补丁新建文件（删得掉，套新建段不冲突）；补丁删文件（原样目录有，拷回）；改名（merge-patch 用 `--no-renames` 出，新旧两侧都在路径单子里）；路径带非 ASCII 的段（`file_path=None` 一律保留：落在 `crates/` 里的照套得上；落在拷贝范围之外的会套不上、判「没做成」，是红不是放过；仓里要引号的路径在 `crates/` 下只有 `crates/singlefs-checker/invariants/` 八份 `.md`，在范围里）。

### 1.2 多份补丁合成一份之后才核（没打中）

merge-patch 是「原样 → 快照此刻」一份 diff，就是 `apply-writer-patch.py` 要落进主工作区的那一份。前一份弄断、后一份修好的，落进去本来就好；前一份追加行、后一份改掉它的原文的，合起来那一行在补丁之后的表里、不在之前的表里，判新红。能盖住的只有一种：前一份把原文改掉（命中 0 次）、后一份在同一份文件别处又写出同一段原文（命中 1 次），变异落在别的地方。这是判据「恰好命中 1 次」字面的限度，单份补丁、`--dry-run` 一样放过，不是合成那一步造的（四句：不分辨这道核与旧做法；系统当时看得到的只有计数；满足的是计数判据本身；第一节的改法碰不到它）。

### 1.3 开跑之后别的会话只追加的变异行（打中）

历史（每一步许可它的那一句）：

1. 这一批 P 开跑：基线是快照（「基线是开跑那一刻的快照」，`scripts/gate/merge-and-verify.sh:261`）。P 把 `crates/X/src/a.rs` 里的一段原文 T 改掉。快照的变异表里没有锚在 T 上的别批行，锚点核过。
2. P 跑着的这段时间（编、测、验证包，按小时计），别的会话 Q 的补丁只往主工作区变异表追加几行、锚在 `crates/X/src/a.rs` 的 T 上（补变异行那一类批）。Q 自己的合入后验证在 Q 的快照上核，那时 T 还在、命中 1 次，核过、合进去。
3. P 验绿，打合进主工作区那一段。开跑那一刻的指纹不算变异表：「登记表（REGISTRATION_TABLES）与 crates/mutations.tsv」（`scripts/gate/merge-and-verify.sh:286`）、「不进起点：各会话都在往里追加，三方合并在 apply-writer-patch.py 里核（冲突退 2）」（`scripts/gate/merge-and-verify.sh:287`）；变异行那一条只核这一批要换、要删的行。现有自证把这一点钉成要求：「开跑之后表里只追加别的会话的行，核变异行的那一条退 0」（`scripts/gate/merge-and-verify-selftest-lib.sh:631`）。`a.rs` 在 Q 里没改，指纹也对得上。
4. `apply-writer-patch.py <日志目录>/merge-patch` 真打不跑锚点核（`scripts/gate/apply-writer-patch.py:1007` 起的 `if dry_run:` 那一支才调 `static_check_report`），变异表按名字三方合并，Q 的行留着。合进去之后 Q 那几行命中 0 次，只有合完补交的 mutation-tables 格报，淹在 1004 处旧红里：就是第一节要关的那一形，换了一个先后。

反过来也一样：P 先合、Q 后合，Q 的指纹起点只算 crates.patch 改到的路径，只改变异表的 Q 没有起点，Q 照合。

样本：`attack-anchor-check.patch` 的 concurrent-append 格（照 anchor-break 那一格的样本树：样本树没有锚在 `1` 上的行，补丁 merge-anchor-break 把 alpha 的 `merge_table.txt` 1 改成 2；跑完往样本树变异表追加 `attack-other-session-row`（锚在 `merge_table.txt` 的 `1`），再起这一趟打出的核主工作区没变的那一条，再把 merge-patch 套到样本树的拷贝上、用自证的假 static-check 判）。预期（推的，待主 agent 跑）：这一趟退 0，核那一条退 0，合进去之后 `ANCHOR … attack-other-session-row 原文在 crates/alpha/src/merge_table.txt 里命中 0 次`，打出「✗ 攻方样本 concurrent-append：…」。

四句：① 分辨：这道核在这条历史上判绿，补一道合进去之前的复核才判红。② 系统看不看得到：开跑时看不到（Q 还没合）；合进主工作区那一刻看得到（主工作区此刻的变异表里有 Q 的行），只是那一段没有一条去看它。③ 满足的是题面「放过新弄断的变异行」字面那一句。④ 改法：见第六节 F1、F2。

### 1.4 补丁前就红的 RELEASE 行被判成新红（打中）

`static_check_red` 比基线时只对几种红去掉第二段的变异表行号：`STATIC_CHECK_ROW_NUMBER_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY", "UNDEFINED", "UNREACHABLE")`（`scripts/gate/apply_writer_patch_plan.py:48`），别的整行当键（`scripts/gate/apply_writer_patch_plan.py:1059`）。static-check 的 RELEASE 行第二段也是变异表行号：`print(f"RELEASE\t{line_number}\t{name}\t{`（`scripts/mutation/crates_mutation_rows_commands.py:134`），而 RELEASE 不在那几种里、也不在不算红的几种里（`scripts/gate/apply_writer_patch_plan.py:46`）。

历史：主工作区变异表第 k 行之后有一行 R 补丁之前就是 RELEASE 红（命中重测试目标而第五段没写 release）；这一批的补丁目录里 `mutations-delete.txt` 删第 k 行（或删 k 之前任何一行）。补丁之前的树上 R 打 `RELEASE <n> R …`，之后的树上打 `RELEASE <n-1> R …`，键不同，R 算这一批带来的新红：这一趟退 1、不打合进主工作区的命令，出路叫这一批重出 R 的替换行。删在表尾之前的每一行，都让它下面每一条补丁前就红的 RELEASE 行一起翻成新红。

样本：release-shift 格（这一格换一份假 static-check，第六段写 `release_red` 的行另打一行 RELEASE，格式照上面那一行；样本树表里 `merge-kept-row`、`attack-row-to-delete`、`attack-release-red-row` 三行，补丁只有 `mutations-delete.txt` 删中间那一行）。预期（推的，待主 agent 跑）：退 1，点名 `RELEASE 2 attack-release-red-row`，打出「✗ 攻方样本 release-shift：…」；照设计应当退 0、报「补丁前就红的 1 处」。

真仓上此刻有没有补丁前就红的 RELEASE 行：推不出（现有那一趟的 `anchor-check.log` 只有计数行，「补丁前就红的 1004 处」不分种类；这条腿不起 static-check）。

四句：① 分辨：与旧做法无关，是这道核自己的键。② 看得到：两棵树的 RELEASE 行都带变异名，按名字比就分得开。③ 满足的是题面「把补丁前就红的判成新红」。④ 改法 F3（RELEASE 进去行号的那几种）在这一格上修好；F1、F2 碰不到它。同一处在 `apply-writer-patch.py --dry-run` 那一支一样（同一个函数）。

### 1.5 补丁改登记表带来的新红两棵树都看不见（打中，推的）

static-check 判 RELEASE 读的登记表是 `RELEASE_REGISTRY_ROOT = [STAGE_REPOSITORY]`（`scripts/mutation/crates_mutation_rows_base.py:354`），拷贝范围（ROW 那一类）是 `def copied_paths(stage_repository=STAGE_REPOSITORY):`（`scripts/mutation/crates_mutation_rows_base.py:455`）：都是 static-check 脚本所在的那一份仓（主工作区），不是被判的树。补丁之前、之后两棵树都拿主工作区此刻那一份登记表判。

历史：这一批登记一条新的 `crash-case:`（`.claude/gate.d/stage-inputs.tsv`，写在补丁里，merge-patch 的 crates.patch 带着它），同时追加几行变异行点它的测试目标、第五段没写 `--release`。锚点核在两棵树上都看不到这条 crash-case，两边都不打 RELEASE，判过；合进主工作区之后那几行 RELEASE 红，又只有 mutation-tables 格报。没做样本（要换真 static-check 才演得出，自证换的是假的）。

四句：① 分辨：旧做法 `--dry-run` 同样看不见，这道核没补上。② 看得到：补丁之后的树里有新登记表，判的时候没读。③ 满足的是「放过新红」，红的种类是 RELEASE 不是 ANCHOR：题面第一节说「只把新红判红」，RELEASE 在 static-check 的红里，算在射程里；只认 ANCHOR 的读法下这一格不算。④ 改法 F4。

## 二、走不到与放过

| 路 | 核不核 | 判定 |
|---|---|---|
| 带 `--skip-apply` | 不核：`if [[ $skip_apply -eq 1 ]]; then dirs=(); check_patches_applied "$logs"; fi`（`scripts/gate/merge-and-verify.sh:2457`），调核的那一行在 `if ((${#dirs[@]})); then` 里面（`scripts/gate/merge-and-verify-items-lib.sh:503`，调用在 507） | 打中（见下） |
| 合起来没有改动 | 不核，四份文件都空才跳（`scripts/gate/merge-and-verify-steps-lib.sh:378`） | 应当不核，没打中 |
| 只改变异表不改源码 | 核：四份里任一不空就做；变异表进路径单子、换回原样 | 没打中（自证没钉住，见第四节） |
| 样本树之外的自证跑法 | `if inside_selftest && [[ ! -e "$snapshot/$ANCHOR_CHECK_SAMPLE_MARK" ]]; then return 0; fi`（`scripts/gate/merge-and-verify-steps-lib.sh:374`）：祖先命令行带 `--selftest` 才算自证，真合入后验证的祖先是任务服务，走不到；跳过时一行字都不打 | 没打中（跳过不留字是小毛病） |
| 核没做成 | 程序退 0 之外一律记红：拷贝范围算不成（ValueError）、补丁套不上（RuntimeError）、static-check 退 2 或 `ERROR` 行、超时（`run_static_check` 交 None）、被杀（static-check 退 -9 是判不了；程序本身被杀退 137，`rc` 非 0） | 没打中 |

`--skip-apply` 这一形：SKILL 写它只给「改动确已在主工作区上的（主 agent 自己写的探索性改动这类）」（`.claude/skills/merge-verify/SKILL.md:30`）。主 agent 自己改 `crates/` 弄断别批锚点，这一趟一次都不核，合完补交的 mutation-tables 格淹在旧红里，与第一节要关的那一形同。核是做得成的：`check_patches_applied` 已经在临时副本里把补丁倒着打掉，倒掉之后的那几份文件就是补丁之前的样子。四句：① 分辨：两种做法在这条路上一样都不核。② 看得到：补丁目录与倒打得到的原样都在。③ 满足的是「放过新弄断的变异行」。④ 改法 F5。

## 三、出路

红了不打合进主工作区那一段：核过（没打中）。`anchor_check_new_breaks "$logs" "$tree" || reds=$((reds+1))`（`scripts/gate/merge-and-verify-items-lib.sh:507`）→ `if [[ $reds -ne 0 || $warned -ne 0 ]]; then return 1; fi`（`scripts/gate/merge-and-verify-items-lib.sh:704`）→ 合进主工作区那一段只在退 0 或 3 时打（`scripts/gate/merge-and-verify.sh:2461-2462`）；`verify` 里锚点核之后没有提前退 0 或 3 的路（现查 424–706 行里的 return 只有 704–706 三处）。

照着做做不成的三处（打中，轻）：

1. 新红超过 20 处只点名前 20 处：`for line in fresh[:20]]`（`scripts/gate/apply_writer_patch_plan.py:1144`），日志里也只有这 20 处。出路「对每一行照补丁之后的代码重取原文与替换文」一趟做不全，剩下的要等下一整趟合入后验证（带编译与测试）才露出来。挪代码、拆文件的批最容易超 20。
2. 补丁把锚着的代码整段删了的行：出路只写「重出 mutations-replacements.tsv 里那几行」（`scripts/gate/merge-and-verify-steps-lib.sh:394`），没有可重取的原文，该做的是删行（`mutations-delete.txt`），出路没写。
3. 出路写「以这一趟留下的快照 $snapshot 为基线」（`scripts/gate/merge-and-verify-steps-lib.sh:393`），而快照只在带 `MERGE_AND_VERIFY_KEEP_COPY=1` 时留（`scripts/gate/merge-and-verify.sh:57`「要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，一趟一份那一种收尾不删」）。SKILL 的标准命令带它，没带的那一趟出路点的路径已经没了。另：「核没做成」的出路叫人修登记表与拷贝范围，超时（每趟 900 秒，见第五节）与被杀两种照它修不了。

## 四、自证与开关

两条开关（推的，读码；主 agent 的 break-switch-replay 在跑）：

- `anchor-check-skipped`：一进来 `return 0`。anchor-break 格退 0、没有 ANCHOR 行、打了合的命令 → 红；anchor-pre-broken 格没有「补丁前就红的 1 处」→ 红。与 `scripts/gate/merge-and-verify.sh:562-564` 写的两格一致。
- `anchor-check-baseline-ignored`：`APPLY_WRITER_PATCH_BREAK=static-check-baseline-ignored` 只在子 shell 里设，python 段载入的 `apply_writer_patch_plan` 在导入时读它（`BROKEN = os.environ.get("APPLY_WRITER_PATCH_BREAK", "")`，`scripts/gate/apply_writer_patch_plan.py:25`），`inherited` 成空表。pre-broken 格把 merge-old-broken-row 判成新红、退 1 → 红；anchor-break 格照绿（本来就该退 1）。只红点名那一格，分得出。

把核改坏、现有自证照样绿的三处（分不出，推的，证红命令在第七节）：

| 改坏（环境变量 `ATTACK_ANCHOR_MUTATION=`） | 改的是哪一处 | 真跑里的后果 | 为什么现有两格照绿 | 哪一格新格会红 |
|---|---|---|---|---|
| `mutation-only-skipped` | `anchor_check_new_breaks` 里 crates.patch 空就不做（`scripts/gate/merge-and-verify-steps-lib.sh:378` 之前插一句） | 只改变异表的批一律不核，自己追加的锚点不中的行放过 | 两格的补丁都带 crates.patch | own-only |
| `own-table-not-restored` | 程序换回原样时跳过 `crates/mutations.tsv`（`scripts/gate/merge-and-verify.sh:2000` 那个循环里） | 补丁之前的树也带着这一批追加、换掉的行：这一批自己写坏的行两边都红、算旧红，静默放过 | anchor-break 格的红行是补丁之前就在表里的别批行；pre-broken 格追加的 merge-head-row 锚点是好的 | own-only、own-with-code |
| `created-files-kept` | 程序不删补丁新建的文件（`scripts/gate/merge-and-verify.sh:2007`） | 补丁新建文件的批一律「补丁在临时树上套不上」判没做成（响亮的假红，不是放过） | 两格的补丁都不新建文件 | created |

另一处同类、没做进补丁（推的）：调核时把 `"$tree"`（快照）写成主工作区那棵树，补丁之后的表就是开跑之前的表，这一批自己追加的行一行都不判；两格照绿（anchor-break 的红行开跑之前就在表里），own-only 与 own-with-code 会红。

病根：自证里的假 static-check 只打 ANCHOR，样本只有「别批的行被弄断」与「补丁前就断」两种，这一批自己的变异表改动（追加、替换）与新建文件这两条路一格都没有，RELEASE 这类带行号的红也没有。补丁里那四格（own-only、own-with-code、created、release-shift）是建议加的格，只在我的样本上写过、被攻过零轮。

复用：程序 `refusal, notes = apply_module.static_check_report(before, merged, patch_path, static_check)`（`scripts/gate/merge-and-verify.sh:2033`），自己只摆树与滤段，判法没有第二份（`.claude/singlefs-ai-sop/rules/sop-first.md@加门禁或钩子之前，先找已有的` 没打中）。

## 五、代价

对不上（`.claude/rules/machine-first.md@对不上就当场说`）：

- 代码里写的：「各约半分钟到一分钟」（`scripts/gate/merge-and-verify-steps-lib.sh:381`）；超时 `STATIC_CHECK_TIMEOUT_SECONDS = 900`（`scripts/gate/apply_writer_patch_plan.py:49`），那一行注释「推的：真仓上一趟约 35 秒」。
- 量的（一个样本）：换上之后唯一一趟真跑的日志目录 `~/.local/share/singlefs/work/merge-and-verify/2026-11-20-3774803/`，`stat -c %Y` 取修改时刻相减：`anchor_minus_mergepatch=625 fmt_minus_anchor=380`。merge-patch 程序写完 `merge-patch.log` 之后紧接着起锚点核，程序的输出全在末尾一次写进 `anchor-check.log`，所以 625 秒就是这一趟锚点核的挂钟（含两次拷拷贝范围、两趟 static-check）。那一趟在哪一台、当时多忙：没查。那一份 `anchor-check.log` 原样两行：「变异锚点静态核（static-check）：CHECKED 8680 1996 13」「! 补丁前就红的 1004 处（不是这个补丁带来的，不拦；提交时门禁 code-source-discipline 的 mutation-tables 格照样判）」。
- 由此推的：每趟合入后验证多约 10 分钟，按条只重跑红的那几项的那一条也是整趟合入后验证、照样再多 10 分钟。两趟平均约 312 秒一趟，离 900 秒的超时不到 3 倍：机器更忙或变异表再长，一趟超时就判「核没做成」、整批不合，出路（修登记表与拷贝范围）对不上。内存：每趟约 16 GiB 是 `apply_writer_patch_plan.py` 注释里的数，这条腿没量；两趟先后跑、在编译之前，峰值不叠在 cargo 上（读码）。

## 六、改法（都只在我的样本上写过、被攻过零轮；表里每一格都是「推的，待主 agent 跑」，没有一格量过）

| 改法 | 修哪一格 | 在打中的格上 |
|---|---|---|
| F1 合进主工作区那一段多打一条：对主工作区此刻的树再做一次同样的锚点核（补丁之前 = 主工作区此刻，补丁之后 = 此刻加 merge-patch），退非 0 不合 | 1.3 | concurrent-append 修好（推的，待主 agent 跑）；1.4、1.5 不碰 |
| F2 开跑那一刻的指纹另记「变异表里锚在这一批改到的文件上的行」，合之前重算，多出来或变了退非 0 | 1.3 | 修好（推的），比 F1 便宜（不起 static-check），但别批行的锚点由谁修要另说 |
| F3 `RELEASE` 进 `STATIC_CHECK_ROW_NUMBER_KINDS` | 1.4 | release-shift 修好（推的，待主 agent 跑）；`--dry-run` 同一处一起修 |
| F4 static-check 认一个「登记表从哪棵树读」的参数，两趟各读各的树 | 1.5 | 推的；改的是 `scripts/mutation/`，射程比这一题大 |
| F5 `--skip-apply` 那一趟用 `check_patches_applied` 倒打出的原样摆补丁之前的树、照样核 | 第二节 `--skip-apply` | 推的 |
| F6 新红全数写进 `anchor-check.log`（标准输出照截 20 处）；出路补「代码被删的行写进 mutations-delete.txt」 | 第三节 1、2 | 推的 |
| F7 自证加 own-only、own-with-code、created、release-shift 四格（补丁里有） | 第四节三处改坏 | 推的，待主 agent 跑第七节 R1–R3 |

## 七、要主 agent 跑的命令

草稿目录 `D=/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus`（下面都写全路径）。副本建好、补丁打上之后，副本在 `D/.deliverables` 写一行「copy 交接 主 agent 在里面跑合入后验证自证的攻方样本格与三处改坏」；用完删。

1. 建副本（直接跑、不经服务；我这一侧被自动模式拒了）。在仓根起：
   `python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy --with-test-inputs --with scripts=副本里起合入后验证的自证与攻方样本格 --with .claude/scripts=副本里自证读的预检脚本 --with .claude/singlefs-ai-sop=副本里自证读的上游脚本`
   看：末尾 `MINIMAL_COPY copied` 一行、退 0。
2. 打样本补丁。在仓根起：
   `git -C /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy apply /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/attack-anchor-check.patch`
   看：退 0；之后 `sha256sum` 副本里三份 `scripts/gate/merge-and-verify{,-steps-lib,-selftest-lib}.sh` 与 `D/new/scripts/gate/` 下同名三份逐个相同。
3. R0 对照（样本格开、不改坏）。`--cwd` 副本：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_CELLS=1 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看 `task-server/task log <号> --all`：`✗ 自检：` 0 行（样本格不扰动现有格）；有「✗ 攻方样本 concurrent-append：」（1.3 坐实）、「✗ 攻方样本 release-shift：」（1.4 坐实）；有「· 攻方样本 own-only：退 1」「· 攻方样本 own-with-code：退 1」「· 攻方样本 created：退 0」（新格在没改坏时绿）；`TASK_SERVER_RESULT` 的 `exit=1`。出现「没走到」的，那一格作废、不算坐实。
4. R1 改坏「只改变异表的不核」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_CELLS=1 ATTACK_ANCHOR_MUTATION=mutation-only-skipped python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：`✗ 自检：` 0 行（现有格分不出）；「✗ 攻方样本 own-only：」有、own-with-code 是「·」行（新格分得出）。
5. R2 改坏「变异表不换回原样」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_CELLS=1 ATTACK_ANCHOR_MUTATION=own-table-not-restored python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：`✗ 自检：` 0 行；「✗ 攻方样本 own-only：」「✗ 攻方样本 own-with-code：」都有。
6. R3 改坏「补丁新建的不删」、样本格开：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_CELLS=1 ATTACK_ANCHOR_MUTATION=created-files-kept python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：`✗ 自检：` 0 行；「✗ 攻方样本 created：」有，原因里有「套不上」或「没做成」。
7. R4–R6 只改坏、样本格关（现有自证自己的结论）：三条各一行，`env` 后只写 `ATTACK_ANCHOR_MUTATION=<名>`：
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_MUTATION=mutation-only-skipped python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_MUTATION=own-table-not-restored python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-opus/copy -- env ATTACK_ANCHOR_MUTATION=created-files-kept python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`
   看：每条 `exit=0`、日志里「✓ 自检通过（查了 167 种）」——这就是「改坏了自证照样绿」；哪一条退 1，那一处改坏现有自证分得出，第四节那一行撤回。

预期全是推的，没量过。跑完副本用 `du -sh` 记大小后删掉，`D/.deliverables` 那一行一起删。

## 没打中的形状

- 补丁之前的树：新建、删除、同一文件几份补丁叠着改、补丁只动拷贝范围外的文件（段被滤掉、记一句「落在拷贝范围之外」）、路径要引号（保留，范围外的套不上判红）。
- 合成一份：前弄断后修好、前追加后改原文、前追加后删、前换后回退；能盖的只有「原文挪到别处仍命中 1 次」（计数判据本身，第一节 1.2）。
- 键的碰撞：`Counter` 按多重集扣，键带变异名，两行同名表里已被 `plan()` 拒；ANCHOR 的原因串里命中次数变了（2→3、0→2）算新红，这是「补丁前就红、补丁之后红得不一样」，照字面算新红，出路做得成，不记打中。
- 核没做成放行：试了闭包算不成、补丁套不上、static-check 退 2、`ERROR` 行、无 `CHECKED` 行、超时、程序与子进程被杀、程序里抛未接的异常（退 1），全红。
- 合进主工作区那一段红了还打：没有。
- 开关漏网：`MERGE_AND_VERIFY_STATIC_CHECK` 平常设了由 `settle_break_switches` 拒（`scripts/gate/merge-and-verify.sh:915` 在那张表里）；`inside_selftest` 真跑走不到。

取样范围：只读码与造样本，没跑任何东西；真仓上的红的种类分布、拷贝范围、各台的耗时都没量。

## 这条腿自己的限度

- 全部结论都是读码推的；唯一量的是一个日志目录的两处修改时刻差，一个样本、哪一台不知道。
- 样本格用自证的假 static-check（只打 ANCHOR，release-shift 那一格是我自己的假的、多打 RELEASE），真 static-check 的输出形状只对着它的文件头与 `print` 那一行核过，没跑。1.5 没做样本。
- `env ATTACK_…` 写在 `--` 之后、包装脚本之前；交的时候被闸按「包装写法」拒的，挪到包装脚本自己的 `--` 之后（`… --with-temporary-service -- env ATTACK_ANCHOR_CELLS=1 bash scripts/gate/merge-and-verify.sh --selftest`）同义。
- 样本格放在 anchor-pre-broken 那一格之后、登记表那几格之前，会改 `$merge_tree` 与临时换 `MERGE_AND_VERIFY_STATIC_CHECK`（换回）；会不会扰动后面的格，R0 的「`✗ 自检：` 0 行」那一条判。
- 没看 `apply-writer-patch.py` 三方合并之后变异表的行序（1.3 的样本用 `git apply` 模拟合进去，没走真合并）。
- F1–F7 都被攻过零轮。

## 没做什么

- 不判正推那几格；不替主 agent 采纳；没往任务服务交任务，没起任何脚本与门禁，判决要引的数没复跑（命令在第七节）。
- 没建仓副本（`minimal-copy.py` 那一条被自动模式拒，第七节第 1 条交主 agent 建）；草稿目录里只有补丁、改前原样（`base/`）、改后三份（`new/`）与 `tools/` 下的草稿，没有编译目录与仓副本。
- 没读禁读清单里的正推腿报告与草稿。

## 附：草稿文件的 sha256（`sha256sum` 现取）

| 文件（相对草稿目录） | sha256 |
|---|---|
| `tools/attack-cells.part` | `ca0b4bc7a365a1464268a19a4e9929b27e7b4c430b54b7539fed5ba84db41415` |
| `tools/make-attack-patch.py` | `4de5f8ac5968853eda392cd4bf428e8dc28d2392729dfeb3944cfb4adcfa8f8e` |
| `new/scripts/gate/merge-and-verify.sh` | `c29914bccac7c7cd2b8a52ae5650884ff1be6a64651a524ee6d2d6e5adbcc729` |
| `new/scripts/gate/merge-and-verify-steps-lib.sh` | `18dc108dfd90b43ed7956a09f18f6719c676a1b94655cac214ca5ce2e3378480` |
| `new/scripts/gate/merge-and-verify-selftest-lib.sh` | `9408ad150db7467f204a4c0d037260018f953cfe02c54d3b1add83a17f315e9b` |

草稿目录合计 1.3M（`du -sh`），里面没有编译目录与仓副本，交回不删（补丁与原样留给主 agent）。引文核对：`python3 scripts/agents/cite-check.py` 末行「✓ 核了 8 处引文、26 处裸行号，对上 8 处，没判 26 处（其中裸行号没有引文可比 26 处；逐处列在上面）」。
