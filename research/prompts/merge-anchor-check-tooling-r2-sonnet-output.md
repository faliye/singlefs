# merge-anchor-check-tooling-r2 辩方腿（Sonnet）报告（2026-11-21）

复核对象：第一轮判决 `research/prompts/merge-anchor-check-tooling-r1-main-verification.md` 二、判决表 8 条；再核三、改法六条在主工作区此刻那六份里做的是不是判决说的。
没有写模型：这一题是治理脚本，没有模型目录、没有 `rerun.sh`；只有一份给主 agent 复跑的样本补丁（草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/`）。全部结论是读码推的，没起任何脚本、没跑任何门禁；主 agent 给的两条观测（自证 `exit=0`、查了 181 种）我只转述，没复跑。
被判六份的 sha256 与开工快照 `research/prompts/merge-anchor-check-tooling-r2-snapshot/judged-sha256.txt` 六行逐个相同（`sha256sum` 现取，开工时与交回前各核一次）。`scripts/mutation/` 下的变异判法在这一轮开工之后换了新版（读被判那棵树的登记表），我读的是换上之后的样子，见第三节。

## 一、总览（一格一行）

| 问 | 结论 |
|---|---|
| 判决 1–7 条采得对不对 | 第 1 条站得住但够不全（镜像方向漏、同文件追加误拒，见第二节）；第 2、4、5、6、7 条站得住；第 3 条归属判得对，且已经落地（第三节） |
| 判决第 8 条「没打中」 | 站得住；有一处要补一句限定（`--skip-apply` 不给补丁目录的那一种，第二节第 8 行） |
| 改法六条今天做的是不是判决说的 | 一至六条都对得上；两处小偏差：第 6 条耗时数偏低（第六节）、第 1 条核得比判决说的宽（见第四节） |
| 新发现（不在第一轮判决里） | 镜像方向：这一批只追加变异行、开跑之后别的会话把那些行锚着的代码改了，合进去之后断，没有任何一条核看得见（推的，没量过；样本补丁在第六节，交主 agent 跑） |

## 二、判决八条复核

引的行号都是主工作区此刻的，写进报告前逐句 `grep -nF` 过。

| 判决 | 辩方判定 | 依据 |
|---|---|---|
| 1 开跑之后别的会话只追加、锚在这一批改掉的原文上的变异行，合进去才断（采，量过） | **站得住（这一个方向）；够不全：同一形状还有镜像方向，且同文件上的追加会被误拒** | 现象本身成立：真打只在 `--dry-run` 那一支跑锚点核，`scripts/gate/apply-writer-patch.py:1014`：「refusal, notes = static_check_report(root, merged, patch, static_check)」；开跑那一刻的指纹不罩变异表，文件头写明登记表与 crates/mutations.tsv 不进起点。修补（记一份「锚在这一批改到的文件上的行」、合之前重算）确实拦这一个方向：`scripts/gate/merge-and-verify.sh:2428`：「def anchored_rows(touched):」。够不全的两处见下面两段 |
| 2 补丁前就红的 RELEASE 行因行号前移被判新红（采，量过） | **站得住；同类先例有、修法对；「两行 RELEASE 原因相同互相顶掉」打不中** | 先例：UNREACHABLE 早就走同一条路，自证格与弄坏开关都在，`scripts/gate/apply-writer-patch.py:294`：「unreachablerowshift（「✗ 自检：补丁删了前面一行变异」」。现在 RELEASE 并进去行号再比，`scripts/gate/apply_writer_patch_plan.py:48`：「STATIC_CHECK_ROW_NUMBER_KINDS = ("MALFORMED", "ANCHOR", "NAME", "ROW", "STRAY", "UNDEFINED", "UNREACHABLE", "RELEASE")」。键去掉第二段之后是（种类、变异名、依据），`scripts/gate/apply_writer_patch_plan.py:1061`：「key = tuple(fields[:1] + fields[2:]) if fields[0] in row_number_kinds and len(fields) > 2 else tuple(fields)」；不同的行变异名不同，进不了同一个键；同名的是同一行（替换），补丁前后都红就是旧红，不是新红；真有重名的，基线按 `Counter` 一对一扣，多出来的那一份照样判新红。static-check 里带行号的红种类现在全在这张表里（`scripts/mutation/crates_mutation_rows_static.py` 的五种行问题、UNDEFINED、RELEASE、UNREACHABLE；LITERAL 与四种可达性提示行本来就不当红），没有第二个漏网的种类 |
| 3 static-check 的 RELEASE 与拷贝范围读主工作区那一份登记表，补丁新登记的看不见（采，读码，不归这一题改） | **站得住，且已落地** | 判决写的「改的是 `scripts/mutation/` 那几处」归属对：读登记表的是 `scripts/mutation/crates_mutation_rows_commands.py:67`：「registry_root = judged_registry_root(root, os.environ)」；弄坏开关回到旧读法写在 `scripts/mutation/crates_mutation_rows_static.py:42`：「return STAGE_REPOSITORY if break_is(environment, "static-check-registry-from-script-repository") else root」。开工时这一处还是读主工作区那份，我读到的是另一个会话换上之后的样子。合入路这一边要配的两行（临时树里带登记表）在本轮换进的六份里，见第四节第 6 行与第五节 |
| 4 带 `--skip-apply` 那一条路一次都不核（采，读码） | **站得住，价值比判决写的低一点；不给补丁目录的那一种仍不核** | 现象成立：`--skip-apply` 那一趟 `dirs` 清空，`scripts/gate/merge-and-verify.sh:2678`：「check_patches_applied "$logs"; fi」；调核的位置原来在 `dirs` 非空的分支里。这条路按 `.claude/skills/merge-verify/SKILL.md:30`：「`--skip-apply` 只用于改动确已在主工作区上的（主 agent 自己写的探索性改动这类）」，补丁已经在主工作区上，核不拦合入（不打合入命令），只在编译之前把「这一批新弄断的」从 1000 多处旧红里分出来，价值是早报。补丁目录一个都不给的那一种照旧不核：`scripts/gate/merge-and-verify-steps-lib.sh:430`：「((${#CHECK_PATCH_DIRS[@]})) || return 0」，没有补丁就倒不出补丁之前的树，这一种修不了，判决没写，应当补一句 |
| 5 出路三处做不成（采） | **站得住** | 三处现在都有对应：日志全数写、标准输出列前 40 行并写明一共几处（`scripts/gate/merge-and-verify.sh:1084`：「ANCHOR_CHECK_SHOWN_LINES=40」）；删行的出路 `scripts/gate/merge-and-verify-steps-lib.sh:414`：「锚着的代码被这一批整段删掉、那一行已没有意义的：改写进 mutations-delete.txt」；没留快照的 `scripts/gate/merge-and-verify-steps-lib.sh:417`：「env MERGE_AND_VERIFY_KEEP_COPY=1 整条重交」。三处是读码核的，没跑 |
| 6 自证分不出三处改坏（采，量过） | **站得住** | 三处改坏现在都是读 `MERGE_AND_VERIFY_BREAK` 的开关：`scripts/gate/merge-and-verify.sh:2144`：「broken == "anchor-own-table-not-restored"」、`scripts/gate/merge-and-verify.sh:2150`：「broken != "anchor-created-files-kept"」，没有靠样本格总开关；点名的格与实际红的格，我逐条读码推过一遍（第四节第 5 行），推的，等主 agent 的开关复跑 |
| 7 代价与注释对不上（采，量过） | **站得住；换上去的数偏低（第六节）** | 旧注释「各约半分钟到一分钟」与量到的 423 秒对不上，成立。但判决只取 423 秒，没提攻方腿报告里另一个样本：同一份产物按修改时刻相减得到 625 秒（`research/prompts/merge-anchor-check-tooling-r1-opus-output.md` 第五节，量的、一个样本、哪一台不知道）。现在注释写成单个数：`scripts/gate/merge-and-verify.sh:286`：「约 423 秒、内存峰值 15.20 GiB（量的）」。两样本差 1.5 倍，每趟超时 900 秒（`scripts/gate/apply_writer_patch_plan.py:49`：「STATIC_CHECK_TIMEOUT_SECONDS = 900」）余量没有看上去大，注释该写成范围（第六节） |
| 8 判法复用 `static_check_report`、补丁之前的树摆法、多份合成之后再核、核没做成一律判红、红了不打合进主工作区的命令（没打中，两条腿都判） | **站得住（五样都没被第一轮的样本打中）；一处要补限定** | 复用：这一轮新加的程序只摆树、滤段、倒变异表，判法还是同一个函数，`scripts/gate/merge-and-verify.sh:2176` 调的就是它，`--dry-run` 那一支同一个函数（`scripts/gate/apply-writer-patch.py:1014`）；补丁之前的树：第一轮的样本里「新建一份文件」「只改变异表」两形都是绿格（判决引的 R0：own-only、own-with-code、created 三格「·」行），摆法没被打中；多份合成：合起来的是「原样 → 快照此刻」一份差，前弄断后修好落进去本来就好；核没做成：程序退非 0 一律记红、`skip-apply` 那一支倒不回也记红（`scripts/gate/merge-and-verify-steps-lib.sh:466`：「ANCHOR_SKIP_PROBLEM="补丁倒不回补丁之前：$REVERSE_CHECK_FAILED"」）；红了不打合入命令：`scripts/gate/merge-and-verify-items-lib.sh:706`：「if [[ $reds -ne 0 || $warned -ne 0 ]]; then return 1; fi」，合入命令只在 verify 退 0 或 3 时打。限定：第一轮的样本里没有「前一份追加、后一份删」「多份补丁、其中一份改变异表」这两形的格，这两形的结论靠读码（程序按名字比出追加、替换、删三份，同名的归替换），不是样本量过的，推的，没量过 |

## 三、第 1 条够不全的两处（新发现，推的，没量过）

### 3.1 镜像方向：这一批的追加行被别的会话的代码改动弄断

形状与判决第 1 条同一个：合进去才断，开跑时看不到，合入时看得到，没有一条核去看。判决第 1 条的修补只记「锚在这一批改到的文件上的变异行」：

- 记的范围是这一批补丁的 crates.patch 点到的路径，`scripts/gate/merge-and-verify.sh:2479`：「anchored = anchored_rows(set(touched_files))」；指纹的起点也只从 crates.patch 点到的包起算，`scripts/gate/merge-and-verify.sh:2477`：「packages, starts = touched_packages(patch_directories」、`scripts/gate/merge-and-verify.sh:2489`：「starts += sorted(packages)」。
- 只改变异表、没有 crates.patch 的一批（例如补一批变异行），点到的路径是空的：起点没有、记下的路径空，合进主工作区那一段的三条核（指纹、变异行、锚点行）一条都不打，`scripts/gate/merge-and-verify-items-lib.sh:866`：「[[ -s "$logs/$MAIN_TREE_ANCHORED_PATHS_NAME" && "${MERGE_AND_VERIFY_BREAK:-}" != "anchored-rows-check-dropped" ]]」那一条的前提就不成立。开跑之后别的会话把这些新行锚着的文件改了（别的会话先合），这一批照合，合进去之后那几行命中 0 次，只有合完补交的 mutation-tables 格报。
- 第一轮攻方腿报告在 1.3 末段写过这个反向（「Q 的指纹起点只算 crates.patch 改到的路径，只改变异表的 Q 没有起点，Q 照合」），判决第 1 条的依据与改法第 1 条都没有接它；攻方腿的 F1（合之前在主工作区此刻的树上再做一遍锚点核）能罩住两个方向，判决采的是更便宜的 F2，F2 只罩一个方向。判决没有写「镜像方向不采」及理由，是漏判，不是判过。
- 什么现象会推翻它：样本补丁（第六节）在今天的代码上不红（打出的合入核里有一条对改过的文件退非 0），或这一类批（只追加变异行）在合入窗口里被别的东西罩住（我没找到，读的是 `merge_in_commands` 全函数 `scripts/gate/merge-and-verify-items-lib.sh:803` 起，没有别的核）。
- 同类先例：第一轮攻方腿自己的 1.3 末段；仓里没有别的先例。

### 3.2 同文件上的追加：宽于「会断」，退到整批重交

合入核（转述）是重算锚在这些文件上的行，多出来或变了的都退非 0（文件头 `scripts/gate/merge-and-verify.sh:323` 起写明；行为在 `scripts/gate/merge-and-verify-items-lib.sh:866` 起）。别的会话追加的行只要第二段落在这一批改到的文件上就拒，不看那行锚的原文是不是这一批改到的那一段；别的会话删掉一行也退非 0（diff 两边都算）。拒的出路是整批重交（再做一遍编译与测试，按小时计）。这是用「不起 static-check」换来的保守，不是漏放，但代价没写进判决：判决只比了 F2 比 F1 便宜，没写 F2 的误拒要用整趟重验还。我没有频率数（表里每天有多少追加落在热门文件上），推的，没量过。推翻它的现象：一个月的合入记录里，被这一条拒掉而照 F1 去核其实不会红的次数占比低（可以在这一批开跑那一刻的表与合入那一刻的表之间离线数）。

## 四、改法六条逐条核（今天主工作区六份）

| 改法 | 对得上吗 | 依据（文件:行，现取） |
|---|---|---|
| 1 开跑那一刻另记锚在改到的文件上的行、合之前重算 | **对得上；比判决说的宽一点** | 记两份：路径单子与行（`scripts/gate/merge-and-verify.sh:2428`：「def anchored_rows(touched):」，在 record 里调用 `scripts/gate/merge-and-verify.sh:2479`：「anchored = anchored_rows(set(touched_files))」）；合入那一段多打一条 awk 重算再 diff，`scripts/gate/merge-and-verify-items-lib.sh:866`，退非 0 写出路是以合入那一刻为基线整批重交。「宽」指的是第三节 3.2，窄指的是 3.1；记下的路径是空的不打（只改变异表的批就是这样漏的） |
| 2 RELEASE 去行号再比，`--dry-run` 同一处 | **对得上** | `scripts/gate/apply_writer_patch_plan.py:48`（同一张表，`--dry-run` 与合入后验证共用）；自证格在 `scripts/gate/apply-writer-patch.py:1810`：「# releaserowshift：同一棵树、同一份补丁」，弄坏开关 `static-check-release-row-number-kept`，文件头 `scripts/gate/apply-writer-patch.py:294` 起登记 |
| 3 `--skip-apply` 那一趟倒出补丁之前的树照样核 | **对得上；倒不回一律红** | 调用点 `scripts/gate/merge-and-verify-items-lib.sh:515`：「skip_apply_anchor_check "$logs" "$tree" || reds=$((reds+1))」，在 `dirs` 分支之外。倒不回（补丁与别的改动叠在同一行、变异表追加行找不到原样、换掉的行找不到改后原样、删掉的行名还在）都记 `ANCHOR_SKIP_PROBLEM` 并返回 1，`scripts/gate/merge-and-verify-steps-lib.sh:466`；变异表倒回在 `scripts/gate/merge-and-verify.sh:2200` 起的程序里，任一处 problems 非空退 1、不写表。没倒出来不放行：我没找到「倒不回而核照常过」的路（读码）。不给补丁目录的那一种不核（`scripts/gate/merge-and-verify-steps-lib.sh:430`），判决第 4 条那一行已写 |
| 4 新红全数写日志、出路三处 | **对得上** | 全数：`scripts/gate/apply_writer_patch_plan.py:1146`：「listed = fresh if shown is None else fresh[:shown]」，合入后验证的程序传 `shown=None`（`scripts/gate/merge-and-verify.sh:2176`）、`--dry-run` 不传、默认 20 不变（`scripts/gate/apply_writer_patch_plan.py:1109`：「def static_check_report(root, merged, patch, static_check=None, shown=20):」）；出路：`scripts/gate/merge-and-verify-steps-lib.sh:414`、`scripts/gate/merge-and-verify-steps-lib.sh:417`，skip-apply 那一支出路另写（直接改表） |
| 5 自证收四格、三处改坏登成开关、各配弄坏开关 | **对得上；点名的格能分开（读码推的，开关复跑还没出）** | 平常就跑的格：并发追加 `scripts/gate/merge-and-verify-selftest-lib.sh:704`：「merge_case anchor-concurrent」、自己追加两格 `scripts/gate/merge-and-verify-selftest-lib.sh:720`：「for anchor_case in "own-only:」、新建 `scripts/gate/merge-and-verify-selftest-lib.sh:730`：「merge_case anchor-created」、RELEASE 前移 `scripts/gate/merge-and-verify-selftest-lib.sh:770`：「merge_case anchor-release-shift」，另有 skip-apply 两格（779 与下一格）、新红多（`scripts/gate/merge-and-verify-selftest-lib.sh:794`：「merge_case anchor-many」）、没留快照（`scripts/gate/merge-and-verify-selftest-lib.sh:802`：「merge_case anchor-no-keep」）。并发追加那一格已按判决改成必绿：它从这一趟打出的输出里取 awk 那一条，追加之后跑它，退 0 或没有那一条都判红，`scripts/gate/merge-and-verify-selftest-lib.sh:707`：「bash -c "$anchored_check_line"」。三处改坏的点名我逐条对了一遍：`anchor-own-table-not-restored` 红 own-only、own-with-code、skip-apply 自己追加；`anchor-created-files-kept` 红 created（补丁之前的树里文件还在，新建段套不上，核没做成）；`anchor-mutation-only-skipped` 红 own-only、skip-apply 自己追加、RELEASE 前移（那一份补丁也只有变异表文件）。三条红的格集合两两不同，分得开（文件头写的点名与此一致，`scripts/gate/merge-and-verify.sh:615` 起）。注意并发追加那一格造的样本里，追加之前表里没有锚在那份文件上的行，记下的行是空的，所以它同时钉住了「记下的行是空的照打」这一条 |
| 6 注释的耗时换成量到的数 | **对得上；数偏低（第六节）** | 三处：`scripts/gate/merge-and-verify.sh:286`、`scripts/gate/merge-and-verify-steps-lib.sh:389` 的开始行、`scripts/gate/apply_writer_patch_plan.py:49`：「STATIC_CHECK_TIMEOUT_SECONDS = 900」那一行的注释，都写 423 秒与 15.20 GiB；与判决里「一、腿与核查」第 3 条那一趟（break 趟，第 4 次尝试 423 秒、峰值 15.20 GiB）一致。旧的「半分钟到一分钟」与「约 35 秒」不再出现（`grep` 过六份，没有残留）。偏差：只写 423 秒，第一轮攻方腿那个 625 秒样本不见了（第二节第 7 行）。另：`scripts/gate/apply_writer_patch_plan.py:49` 的注释里引了判决文件路径，按共用约束「代码与脚本的注释、说明只写现状」写依据要写「现行登记位的编号与名字」，判决不是登记位；门禁 doc-text 的 comment-history 格判不判它，我没跑门禁，推的，没量过 |

## 五、管变异键那一件换上之后：补丁前后两棵临时树里各有登记表吗（第二节第 5 问，读码，推的，没量过）

结论：两棵树里都有，补丁改登记表时各是改前、改后的那一份；这一条我没打中，读法如下。

- 登记表在拷贝范围里：`scripts/gate/apply_writer_patch_plan.py:44`：「".claude/gate.d/stage-inputs.tsv", "static-check 读被判那棵树里的这张登记表」，另有弄坏开关下的小清单也加了：`scripts/gate/apply_writer_patch_plan.py:43`：「STATIC_CHECK_TREE_PATHS = ("crates", "litmus", "Cargo.toml", "Cargo.lock", ".cargo") + (".claude/gate.d/stage-inputs.tsv",)」。static-check 的新版读树里的那一份，没有就退 2、不回落：`scripts/mutation/crates_mutation_rows_commands.py:67`：「registry_root = judged_registry_root(root, os.environ)」。
- 补丁之后的树是在补丁之前的树的拷贝上套 crates.patch，登记表被补丁改到时，路径在 `merge-baseline.paths` 里（crates.patch 点到的），原样取自开跑那一刻（打补丁之前存的），所以补丁之前的树里是改前的一份，套上补丁是改后的一份；`inside(relative)` 认它在拷贝范围里（`scripts/gate/merge-and-verify.sh:2135` 起的 `inside`），所以那一段 crates.patch 不会被滤掉。补丁新建登记表（原样里没有）：补丁之前的树里被删，核退 2（响，不静默）。
- `--skip-apply` 那一支：补丁之前的树来自 `reverse_check_in_order` 倒回的原样，同一个清单；`--dry-run`：根是被判的树自己，同一个清单。
- 这一条没覆盖的：登记表里还有别的表被 static-check 读（`.claude/gate.d/` 下别的登记表）是不是也在临时树里，我没逐个核；这是攻方腿的活。

## 六、耗时补正（量的：文件修改时刻相减，每趟一个样本；判决引的 423 秒只是其中最短的一个）

判决第 7 条与改法第 6 条把「两趟 static-check 共约 423 秒、峰值 15.20 GiB」写进三处注释。主 agent 的探针运行目录里留着两趟的输出，探针每次开头 `rm -rf` 运行目录重建，所以文件时刻是最后一次尝试的。我用 `stat -c %Y` 取「准备完成」（`prepare.txt`）与「核输出写完」（`anchor-check.out`）两个时刻，相减就是核本身的挂钟（不含拷快照）。命令与原样输出：

```
cd ~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-main/; for r in run-break run-control; do a=$(stat -c %Y $r/logs/prepare.txt); b=$(stat -c %Y $r/logs/anchor-check.out); echo "$r prepare=$a out=$b diff=$((b-a))"; done
run-break prepare=1795180552 out=1795180958 diff=406
run-control prepare=1795181278 out=1795182018 diff=740
```

| 样本 | 核的挂钟 | 来源 |
|---|---|---|
| break 那一趟（判决引的那一趟） | 406 秒（任务第 4 次尝试整趟 423 秒，含拷快照） | 上面的命令；判决「一、腿与核查」第 3 条 |
| control 那一趟（判决只贴了「判绿」，没贴时长） | 740 秒 | 上面的命令 |
| 第一轮攻方腿量的另一份产物 | 625 秒（`anchor_minus_mergepatch`） | `research/prompts/merge-anchor-check-tooling-r1-opus-output.md` 第五节 |

三个样本跨 406 到 740 秒（相差 1.8 倍），哪一台、当时多忙都没记。注释现在写「约 423 秒」是三者里最短的一个附近，会让人低估：每次合入后验证多等 7 到 12 分钟，不是 7 分钟。每趟 static-check 的超时是 900 秒，两趟先后各算各的；740 秒除以二是每趟 370 秒左右，余量约 2.4 倍，不是不够，但机器忙一倍就贴边，超时判「核没做成」整批不合。改法：三处注释写成「量的：406 到 740 秒（两个真仓样本），峰值 15.20 GiB」，并把峰值标明是整趟探针任务的峰值（量的，同一趟）。这不影响任何判法，只是注释的数。
推翻它的现象：再量两三趟，核的挂钟都落在 400 到 450 秒（740 秒是那一趟撞了别的任务）。

## 七、要主 agent 跑的命令

草稿目录记作 `D` = `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet`，下面都写全路径。样本是第三节 3.1 的镜像格：补丁 `D/tools/mirror-cell.patch`（对 `scripts/gate/merge-and-verify-selftest-lib.sh` 的一处插入，33 行，`git apply -p2`），做的事：这一批只追加一行变异行（锚在 `crates/alpha/src/merge_table.txt` 的 `1`），开跑之后别的会话把那份文件改成 `2`，取这一趟打出的合入核（find、grep、awk 三种开头的行）逐条跑，一条不退非 0 就报「✗ 镜像样本命中」。预期（推的，没量过）：在今天的代码上这一格红（核一条都没打出来：只改变异表、记下的路径空，指纹没起点）；同一趟里别的格不红。

1. 建副本（直接跑，不经服务；仓根起）：`python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/copy --with-test-inputs --with scripts=副本里起合入后验证自证 --with .claude/scripts=副本里自证读的预检脚本 --with .claude/singlefs-ai-sop=副本里自证读的上游脚本`；看：末尾 `MINIMAL_COPY copied` 一行、退 0。
2. 先核副本里的自证库与我造补丁时的基线相同（直接跑，只读；仓根起）：`sha256sum /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/copy/scripts/gate/merge-and-verify-selftest-lib.sh`；看：`9ab68864412999fea715d262276abfc2659d9ae55fc85761724a40af40cc4579`；不同说明别的会话又改了那份文件，补丁要重做，不往下。
3. 打补丁（直接跑；仓根起）：`git -C /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/copy apply -p2 /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/tools/mirror-cell.patch`；看：退 0；之后 `sha256sum` 副本里那份应为 `d19a82ff2eab18928ccef38b5ed14e003bd57bdcba5eb02f093fa34704d6b6b0`。
4. 镜像格（交任务；`--cwd` 副本）：`task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/copy -- python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`；看：`TASK_SERVER_RESULT` 的 `exit=1`；`task-server/task log <号> --all` 里「✗ 自检：」0 行（别的格不红），有且只有一行「✗ 镜像样本命中」（3.1 坐实）；出现「✗ 镜像样本没走到」的这一格作废；`exit=0` 说明镜像格没红，3.1 撤回。
5. 补量核的挂钟（交任务；可选；用主 agent 已有的探针，运行目录放我的草稿目录，探针自己每次 `rm -rf` 重建）：`task-server/task submit -- bash /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-main/anchor-real-probe.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/probe-control-run control`；看：末行 `exit=0`、输出里「核 N 秒」那一行的 N（第六节的推翻条件：N 落在 400 到 450 秒）与「✓ control：判绿」；跑完 `du -sh` 记大小再删 `probe-control-run`（只删工作根下名字以 `-run` 收尾的目录）。
6. 用完删副本（直接跑；先 `du -sh` 记大小）：`du -sh /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/copy`，再 `rm -rf /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r2-sonnet/copy`。

开关复跑（`apply-writer-patch.py` 与 `merge-and-verify.sh` 的弄坏开关复跑）按主 agent 手里已排的那两趟，不在这份报告里另交：我读码推的点名与文件头一致，复跑结局出来前这一条是推的。

## 八、各条结论什么现象会推翻它

| 结论 | 推翻它的现象 |
|---|---|
| 第 1 条站得住（并发追加成立） | 自证里并发追加那一格在今天的代码上不绿（`scripts/gate/merge-and-verify-selftest-lib.sh:704` 起），或合入那一段打出的 awk 那一条对追加之后的表退 0 |
| 第 2 条站得住、两行 RELEASE 不会互相顶掉 | 真仓表里有重名的行（同名两行同依据），或 static-check 的 RELEASE 行第三段不是变异名；任一出现，Counter 一对一扣的结论要重判 |
| 第 3 条归属对、已落地 | `scripts/mutation/crates_mutation_rows_commands.py:67` 又被换回读主工作区那一份，或补丁新登记 crash-case 的样本在合入核里照样不红 |
| 第 4 条站得住、不给补丁目录的那一种不核 | `--skip-apply` 不给补丁目录时合入核有别的入口（我没找到，读的是 `scripts/gate/merge-and-verify-steps-lib.sh:430` 与 `scripts/gate/merge-and-verify-steps-lib.sh:40`） |
| 第 5、6 条站得住 | 开关复跑里点名的格与实际红的格对不上，或三条新改坏开关下现有自证照绿（只读码推的，复跑未出） |
| 第 7 条站得住、数偏低 | 第七节第 5 条再量的挂钟都在 400 到 450 秒（第六节） |
| 第 8 条没打中 | 镜像样本之外，主 agent 复跑别的两形（前追加后删、多份补丁其中一份改变异表）样本，发现放过新红 |
| 3.1 镜像方向 | 第七节第 4 条 `exit=0`（镜像格不红） |
| 3.2 同文件追加误拒 | 一个月合入记录里被这一条拒掉而照 F1 不会红的占比低 |

## 九、没做什么

- 没替主 agent撤销或维持判决，只交复核结果；没往任务服务交任务，没起任何脚本与门禁（`stat`、`grep`、`sha256sum`、只读读码除外）；判决要引的数（181 种、开关复跑、406 与 740 秒的复跑）没复跑，命令在第七节。
- 没跑两份脚本的弄坏开关复跑，点名与实际红的格的对应是读码推的。
- 没核 `.claude/gate.d/` 下别的登记表是不是都在临时树里（第五节末条）；没数每天有多少追加落在热门文件上（3.2）；每趟约 370 秒是 740 除以二的平均，两趟实际各多长没拆（推的）。
- 没碰主工作区任何文件；草稿目录里只有补丁、改前改后各一份自证库（`mirror/`）与样本片段（`tools/`），没有副本与编译目录。
- 没读禁读清单里的文件（第二轮攻方腿的报告与草稿目录）。
- `scripts/mutation/` 在这一轮里被另一个会话换了新版，行号取读时那一刻；换完之后我只读了登记表那一处，没重核那几份的别的改动。
