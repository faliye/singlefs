# merge-anchor-check-tooling-r1 正推腿报告（Sonnet）

日期：2026-11-20。分工：逐条核换上的四份改动做的是不是正文第一节说的、能不能拦住第一节量到的那一形；第二节五条各给结论。
没跑任何脚本、没起任务；读码推出来没量过的写「推的，没量过」。所有行号取自主工作区此刻的文件。

## 0. 开工核的几件事

- 主工作区那四份与工具员草稿目录 `new/` 里的逐字节相同（`cmp` 四份都无输出、打了 same-as-new）；此刻 sha256：
  - `scripts/gate/merge-and-verify.sh` c88d382fbeb9104b3903c09bd14955f34d9b55ab3eff609703c53814ee4c04d5
  - `scripts/gate/merge-and-verify-steps-lib.sh` 5d46e3125dc89a2785c0b1b70033a984fc4978c2958bcc1517995c117487ec14
  - `scripts/gate/merge-and-verify-items-lib.sh` 73ce4dee734b752c4f4ed0e5cfb54c3faaa06ca2e60192e17b104a1365d73f19
  - `scripts/gate/merge-and-verify-selftest-lib.sh` 579cbcd8f64d8d4a414162a7dad4173ca3633b91de4cce92ea676bb00fa9c8a4
- 换上之前的原样在 `~/.local/share/singlefs/work/merge-anchor-static-check/base/scripts/gate/`；我用 `diff base 此刻` 读的改动，没读别的版本。
- `scripts/gate/apply-writer-patch.py` 与 `scripts/gate/apply_writer_patch_plan.py` 这一轮没改（工具员报告说没改；我只读它们此刻的样子）。
- 改动形态（`diff` 的增行数，量的）：steps-lib 增 31 行（一个函数）、items-lib 增 2 行（一处调用）、selftest-lib 增 45 行（两格加一句成功句）、merge-and-verify.sh 增文件头说明、开关表两条、两个全局、一段内联 python（`MERGE_ANCHOR_CHECK_PROGRAM`）。

## 1. 第一问：做的是不是第一节说的、拦不拦得住那一形

结论一行：做的就是第一节说的，能拦住「补丁让别批变异行原文命中 1 次变 0 次」那一形——
`scripts/gate/merge-and-verify-items-lib.sh:507`：「anchor_check_new_breaks "$logs" "$tree" || reds=$((reds+1))」，
红了记一项、`verify` 退 1（`scripts/gate/merge-and-verify-items-lib.sh:704`：「if [[ $reds -ne 0 || $warned -ne 0 ]]; then return 1; fi」），
合进主工作区的命令只在退 0 或 3 时才打（`scripts/gate/merge-and-verify.sh:2462`：「then merge_in_commands "$logs" "$verify_rc"; fi」）。
真 static-check 上「红」那一支没量过，只量过「绿」那一支（见 1.5）。

### 1.1 调用位置与前提（读码，推的、没量过）

- 调用在合成补丁目录成功之后：`scripts/gate/merge-and-verify-items-lib.sh:503`：「if ((${#dirs[@]})); then」那一支里，`merge-patch` 算成功才调；算不成另走红（同文件 509 行 `reds=$((reds+1))`）。
- 调用时快照 `$tree` 已经是补丁全部打上之后的样子（打补丁的循环在 503 行之前）。所以「补丁之后的树」=快照、「补丁之前的树」要现摆。
- 合成补丁目录的 `crates.patch` 不含变异表：`scripts/gate/merge-and-verify.sh:1866`：「files = [path for path in paths if path != MUTATION_TABLE]」；
  变异表的改动只走三份变异表文件。所以后面把「快照此刻的表」整份写进补丁之后的树、再套 `crates.patch`，不会因为补丁里也有表的段而套不上。

### 1.2 「补丁之前的树」怎么摆（`MERGE_ANCHOR_CHECK_PROGRAM`）

- 拷贝范围取快照此刻的：`scripts/gate/merge-and-verify.sh:1992`：「items = apply_module.static_check_tree_items(tree)」，再 `scripts/gate/merge-and-verify.sh:1999`：「apply_module.repository_copy_module().copy_entries(os.path.realpath(tree), before, items)」拷成 `before/`。
- 换回原样：路径单子是 `merge-baseline.paths`（补丁新旧两侧路径，有变异表文件的另加 `crates/mutations.tsv`，`scripts/gate/merge-and-verify.sh:1824`：「paths = list(dict.fromkeys(paths + [MUTATION_TABLE]))」）。
  只处理落在拷贝范围里的（`scripts/gate/merge-and-verify.sh:2001`：「if not inside(relative):」）；
  原样在 `merge-baseline/` 里的拷回（`scripts/gate/merge-and-verify.sh:2006`：「shutil.copy2(original, target)」），没有的（补丁新建的）删掉（`scripts/gate/merge-and-verify.sh:2007`：「elif os.path.isfile(target) or os.path.islink(target):」）。
  补丁删掉的文件，原样在 `merge-baseline/` 里 → 拷回，没问题（推的）。
- 补丁之后的树不是直接用快照，而是 `static_check_report` 内部再从 `before` 拷一份、写上快照此刻的变异表、套 `crates.patch`（`scripts/gate/apply_writer_patch_plan.py:1095`：「os.makedirs(os.path.join(tree, "crates"), exist_ok=True)」起一段，1103 行套不上抛 RuntimeError）。
  即补丁之后 = 原样 + 合成补丁，与快照内容应当相同；两边不同的唯一途径是 `crates.patch` 与「原样 → 快照」不自洽（做补丁时同一份输入，不会）或被过滤掉的段（1.3）。套不上就走「没做成」那一支（红），不会放行。
- 过滤：`crates.patch` 按 `diff --git` 切段，只留落在拷贝范围里的（`scripts/gate/merge-and-verify.sh:2019`：「if file_path is None or inside(file_path):」）。
  范围外的文件临时树里没有，`git apply` 会套不上，变异行也读不到它们（`ROW` 种类「文件不在每片拷的范围里」另判）。
  带引号的路径（git 对非 ASCII 路径加引号；`crates/singlefs-checker/invariants/` 下有 8 个中文名文件，`git ls-files crates litmus | grep -P` 命中 8 行，量的）`file_path` 取 None，一律保留，它们在 `crates/` 里，在范围内，不会套不上。

### 1.3 比基线（复用的 `static_check_report`，没另写）

- 调用：`scripts/gate/merge-and-verify.sh:2033`：「refusal, notes = apply_module.static_check_report(before, merged, patch_path, static_check)」，`merged` 读的是快照此刻的 `crates/mutations.tsv`（`scripts/gate/merge-and-verify.sh:2029`：「with open(os.path.join(tree, MUTATION_TABLE), encoding="utf-8") as handle:」）。
- `static_check_report` 里：补丁之后那趟 `scripts/gate/apply_writer_patch_plan.py:1116`：「tree = build_patched_tree(root, merged, patch)」；基线那趟 `scripts/gate/apply_writer_patch_plan.py:1119`：「before_code, before_lines = run_static_check(make_command(root))」，这里 `root` 就是摆好的 `before/`，所以基线是「补丁之前的树 + 补丁之前的变异表」（变异表被补丁动过的已在 `before/` 换回原样）。
- 比的方式：键去掉变异表行号（`scripts/gate/apply_writer_patch_plan.py:1059`：「key = tuple(fields[:1] + fields[2:]) if fields[0] in row_number_kinds and len(fields) > 2 else tuple(fields)」）；
  基线上的红按多重集扣（`scripts/gate/apply_writer_patch_plan.py:1134`：「inherited = collections.Counter(key for key, _ in before_red) if BROKEN != "static-check-baseline-ignored" else collections.Counter()」）；补丁之后多出来的才算新红（1142 行 `if fresh:`）。
- ANCHOR 行的键是 种类 + 变异名 + 原因文字（原因里含「命中 N 次」，不含树路径：`scripts/mutation/crates_mutation_rows_static.py:285`：「return None if count == 1 else f"原文在 {row.path} 里命中 {count} 次，要恰好 1 次"」）。
  所以：补丁之前恰好命中 1 次（不红）、之后 0 次 → 基线里没有这一行、补丁之后有 → 新红。这正是第一节量到的那一形。补丁之前就 0 次、之后还是 0 次 → 键相同、扣掉、算旧红。
  补丁之前 0 次、之后 2 次，键里的次数变了，也算新红（偏严，不漏）。
- 判不了一律红：任一趟 static-check 退码不是 0 或 1、或有 `ERROR` 行（`scripts/gate/apply_writer_patch_plan.py:1126`：「if code not in (0, 1) or any(line.startswith("ERROR") for line in lines):」），补丁之后那趟没有 `CHECKED` 行（1131 行），摆树套补丁失败（1121 行 `return [f"✗ 变异锚点静态核没做成：{error}", howto], []`）。内联程序再把摆树自己的失败收成退 2（`scripts/gate/merge-and-verify.sh:2045`：「except (OSError, ValueError, RuntimeError, ImportError) as error:」），别的异常未捕获，python 以退 1 带回溯退出——仍是非 0。
- 调用方只认退出码：`scripts/gate/merge-and-verify-steps-lib.sh:386`：「if ((rc == 0)); then」，其余一律走红的那一支（391 行 `✗ 变异锚点核红`），所以拿到的是 0 之外的任何码（含 137 被杀、127 找不到 python3）都判红，没有「读不出就当过」的分支（推的，没量过）。

### 1.4 多份补丁目录合成一份后才核，会不会盖掉某份弄断的行

- 核的是「快照此刻」对「摆回原样」，不是逐份补丁的并集：`merge-patch` 的 `crates.patch` 是「开跑那一刻的原样 → 全部补丁打完后的快照」的净差（`scripts/gate/merge-and-verify.sh:1866` 的 `files` 取各份补丁点到的路径的并集、同一份文件只出一段）。
- 某份补丁把行 R 的锚点弄断、后一份又修好：净差里没有这个断，核看终态，不红，这是对的（落进主工作区的就是终态）。弄断而没有任何一份修好：终态有断，红。没有「合成覆盖掉弄断」的路径，因为核读的是合成之后的终态、不是各份各自的样子（推的，没量过）。
- 变异表同理：核直接读快照此刻的表，不读 `mutations-append.tsv`、`mutations-replacements.tsv` 那三份按名字比出来的文件，所以三份文件的比出方式（同名行折叠）不影响这道核。

### 1.5 真 static-check 上量到的与没量到的

量的（主 agent 换上之后头一趟真用的日志目录 `~/.local/share/singlefs/work/merge-and-verify/2026-11-20-3774803/` 里读到）：

```
$ cat anchor-check.log
变异锚点静态核（static-check）：CHECKED 8680 1996 13
! 补丁前就红的 1004 处（不是这个补丁带来的，不拦；提交时门禁 code-source-discipline 的 mutation-tables 格照样判）
```

- 合成的补丁目录那一行：「合起来改 11 份文件，变异行追加 7、换 6、删 0」（同目录 `merge-patch.log`）；单次尝试（`task-attempt` 末字段 1）。
- 这趟只证明：核在真树上摆得出树、两趟 static-check 都判得了、补丁前红的 1004 处被扣成计数、这一份补丁没有新红。它没证明「红」那一支：真 static-check 上「补丁把别批行 1→0」被判红，没量过。
  造这个红的探针与命令写在第 6 节「要主 agent 跑的命令」第 1 条（我写了脚本，没跑）。

## 2. 第二问：五条各给结论

### 2.1 判法对不对（第二节第 1 条）：成立，没有打中；两处偏严、一处没样本覆盖

- 摆「补丁之前的树」的四个要素（拷贝范围、换回原样、删新建、只留落在范围里的补丁段）见 1.2，读码对得上；套不上、拷不成、判不了都走红（1.3），不会放行。
- 「补丁前就红的算成新红」：不会。基线是 `before/`（补丁之前的树 + 补丁之前的变异表），同一份 static-check 在同样的拷贝范围里判，键去掉行号再按多重集扣。变异表被补丁动过的，`before/` 里是换回的原样（`scripts/gate/merge-and-verify.sh:1824`：「paths = list(dict.fromkeys(paths + [MUTATION_TABLE]))」把表放进路径单子）。
- 「把新红漏成旧红」：键里带原因文字（含命中次数），补丁之前 0 次、之后 2 次也算新红；只有「变异名、原因文字完全相同」的才被扣成旧红，那就是没变的旧红。没找到漏的输入（推的，没量过）。
- 偏严的两处（红了不放，但可能红得不该红）：
  1. 带引号的路径一律保留进补丁（`scripts/gate/merge-and-verify.sh:2018`：「file_path = None if rest.startswith('"') else rest[2:2 + (len(rest) - 5) // 2]」），若补丁里有落在拷贝范围外、又是非 ASCII 名的文件，临时树上套不上，整道判「没做成」红。本仓范围内的中文名文件在 `crates/singlefs-checker/invariants/`（在范围内，不触发）；范围外的此刻没有这样的补丁，是预留的偏严。
  2. 补丁删掉一个被依赖的目录时，拷贝范围取的是补丁之后的快照（`scripts/gate/merge-and-verify.sh:1992`），已删的目录原样不会被换回 `before/`（`scripts/gate/merge-and-verify.sh:2001` 的 `inside` 判它不在范围里）；基线少一个目录，只可能让依赖它的行在基线里也红，不会把新红漏掉。推的，没量过。
- 没样本覆盖的（不是错，是自证看不见）：补丁新建文件（`before/` 里删掉那一支）、补丁追加一行锚点坏的新变异行（新行的新红）、只改变异表的补丁、范围外文件段被丢（打 `· 合起来的补丁里 N 份文件落在拷贝范围之外` 那一句）。两格样本的补丁都只改已有文件的一行（`scripts/gate/merge-and-verify-selftest-lib.sh:680`：「merge_patch_text crates/alpha/src/merge_table.txt 1 2 > "$scratch/merge-anchor-break/crates.patch"」）。
- 多份补丁目录合成一份：见 1.4，核终态，不会因合成盖掉某份弄断的行。

### 2.2 走不到与放过（第二节第 2 条）：四条路各有明确结论；核没做成一律红

| 路 | 核不核 | 依据 |
|---|---|---|
| 带 `--skip-apply` | 不核，成立 | `scripts/gate/merge-and-verify.sh:2457`：「if [[ $skip_apply -eq 1 ]]; then dirs=(); check_patches_applied "$logs"; fi」，`dirs` 空 → 不进 `scripts/gate/merge-and-verify-items-lib.sh:503` 的那一支，也没有 merge-patch；这一趟补丁已在主工作区上，没有「合进去」可拦（出路只剩合完另交的 mutation-tables 格）。 |
| 合起来没有改动 | 不核，成立 | `scripts/gate/merge-and-verify-steps-lib.sh:378`：「if [[ ! -s "$merge_patch/crates.patch" && ! -s "$merge_patch/mutations-append.tsv"」四份都空才不做；任一份有内容就做。 |
| 自证里的样本树 | 只核带标记的，成立 | `scripts/gate/merge-and-verify-steps-lib.sh:374`：「if inside_selftest && [[ ! -e "$snapshot/$ANCHOR_CHECK_SAMPLE_MARK" ]]; then return 0; fi」。自证里别的格（样本树没标记）不多跑一道、不多打一行；真跑时祖先里没有带 `--selftest` 的进程（任务服务起的进程链不带），推的，没量过。 |
| 只改变异表不改源码 | 核，成立 | 路径单子带变异表（`scripts/gate/merge-and-verify.sh:1824`），`crates.patch` 为空 → `patch_path` 取 None，补丁之后 = `before/` + 快照此刻的表；补丁之前 = `before/` + 原表。追加或替换成坏锚点的行是「新出现的红」，判红。真跑没量过；两格样本里「必绿」那格的补丁带变异表追加（`scripts/gate/merge-and-verify-selftest-lib.sh:566`：「printf '%s\t%s\t%s\t%s\t%s\t%s\n' 'merge-head-row' crates/alpha/src/lib.rs」），只证明了表有改动时核照跑、不红。 |

- 核没做成一律红：拷贝范围算不成（`static_check_tree_items` 抛 ValueError / OSError，内联程序收成退 2，`scripts/gate/merge-and-verify.sh:2045`：「except (OSError, ValueError, RuntimeError, ImportError) as error:」）；static-check 判不了、套不上补丁（`static_check_report` 返回拒绝，内联程序退 1，`scripts/gate/merge-and-verify.sh:2041`：「return 1 if refusal else 0」）；rsync 失败（`copy_entries` 抛 OSError：`scripts/hosts/lib_repository_copy.py:875`：「rsync 退非 0 抛 OSError」）；内存不够被杀——被杀的是 static-check 子进程时它退负码、不在 (0, 1) 里，判「判不了」红；被杀的是内联 python 或整条尝试时，shell 侧拿到非 0（`scripts/gate/merge-and-verify-steps-lib.sh:386`：「if ((rc == 0)); then」只认 0），整条被杀的由服务放大重排（推的，没量过）。
- 一处放过的路径检查：`inside_selftest` 对真跑只靠进程祖先里有没有 `--selftest`，真跑一般不会有；`MERGE_AND_VERIFY_STATIC_CHECK` 与 `MERGE_AND_VERIFY_BREAK` 平常设了被拒（`scripts/gate/merge-and-verify.sh:915`：「MERGE_AND_VERIFY_SLOT_TABLE MERGE_AND_VERIFY_SLOT_BUILD_ROOT MERGE_AND_VERIFY_STATIC_CHECK)」在 `BREAK_VARIABLES` 里）。
  `APPLY_WRITER_PATCH_BREAK` 不在这份 `BREAK_VARIABLES` 里，但内联程序把 `apply-writer-patch.py` 当库载入时它读到的开关值会让 `static_check_report` 返回空（`scripts/gate/apply_writer_patch_plan.py:1109`：「if BROKEN == "static-check-skipped":」）；真跑时前面打补丁那一步起的 `apply-writer-patch.py` 自己会因这个变量设着而退 2（`scripts/gate/apply-writer-patch.py:1986`：「setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]」起的 `settle_break_switches`），整批停在核之前，走不到这一道；所以这条放过的路堵住了，但靠的是前面那一步而不是核自己（推的，没量过）。

### 2.3 出路（第二节第 3 条）：红了不合成立；出路照做大体做得成，缺一句「没留快照怎么办」

- 红了之后「合进主工作区」那一段不打：成立。红记在 `reds`（`scripts/gate/merge-and-verify-items-lib.sh:507`），`verify` 在 `reds` 非 0 时退 1（`scripts/gate/merge-and-verify-items-lib.sh:704`），打合入命令的调用只在退 0 或 3 时才进（`scripts/gate/merge-and-verify.sh:2462`：「then merge_in_commands "$logs" "$verify_rc"; fi」）；`merge_in_commands` 只有这一处调用（`grep -n merge_in_commands` 在这四份脚本里的调用点只有 2462 行，量的）。退 3 要求 `reds` 为 0（`scripts/gate/merge-and-verify-items-lib.sh:705`：「((TESTS_REFUSED == 0)) || return 3」在红的判之后）。
- 红了还接着跑四步 cargo 与验证包那一步（`scripts/gate/merge-and-verify-items-lib.sh:507` 只记红、不退）：这一趟要等到底才给结论，红的补丁目录是要改的，后面的测试结果多半作废。与别的步骤一致（各步都「记一项、接着跑」），不算缺陷；真跑里 cargo 四步约占这一趟一半挂钟（见 2.5）。
- 点名：红时输出的行是「ANCHOR <行号> <变异名> 原文在 <文件> 里命中 N 次，要恰好 1 次」，带变异名、文件与次数（`scripts/gate/merge-and-verify-selftest-lib.sh:685` 那一格验这一形）；`static_check_report` 最多逐行列 20 处、多的只写一行 …还有 N 处 的计数（`scripts/gate/apply_writer_patch_plan.py:1144`：「for line in fresh[:20]」一段），所以一趟弄断超过 20 行的，要改完再核才看得见后面的行（读码，推的，没量过）。
- 出路文字（`scripts/gate/merge-and-verify-steps-lib.sh:393`：「→ 怎么办：上面点名的行（ANCHOR：原文在文件里原来恰好命中 1 次、现在 0 次或多次）是这一批补丁新弄断的」）要做的：以快照为基线、重取原文与替换文、重出 `mutations-replacements.tsv` 的十二段行、接在这一批补丁目录之后整批再交。
  - 十二段格式与 `scripts/gate/apply-writer-patch.py:17` 的说明一致：「mutations-replacements.tsv   要整行换掉的变异行，每行十二段：前六段是新行，后六段是写补丁时表里那一行的原样（前像）」；接在这一批之后时前像取快照（已打完这一批的样子）里那一行，与「后一份以前一份打完的样子为基线」一致。
  - 缺的一句：这句出路指的「这一趟留下的快照」只有设了 `MERGE_AND_VERIFY_KEEP_COPY=1`（或占到槽、槽没被别的趟占走）才在；别处的同类出路都带了「没留快照的照原命令带 env MERGE_AND_VERIFY_KEEP_COPY=1 重交一趟」（`scripts/gate/merge-and-verify-steps-lib.sh:569`），这一句没带。主 agent 平常交合入后验证带这个变量（合进主工作区的命令本来就要快照，`scripts/gate/merge-and-verify-items-lib.sh:804`：「if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" != 1 && "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-without-kept-snapshot" ]]; then」），所以实际多半在；没带的那一趟，出路走不通、要先重交一趟。推的，没量过。
  - 没做成的那几种（拷贝范围算不成、static-check 判不了），出路写「照那几行与日志修好 static-check 读的变异表、登记表与拷贝范围，再整批重交」，指向日志 `<日志目录>/anchor-check.log`（`scripts/gate/merge-and-verify-steps-lib.sh:385`：「) > "$logs/anchor-check.log" 2>&1; rc=$?」，写进去的就是内联程序的输出），做得成。

### 2.4 自证与开关（第二节第 4 条）：两格分得出「在不在」与「基线用没用」；两条开关各红在点名那一格；复用了 static_check_report

- 自证两格的分辨力（读码，推的；自证本身工具员那一趟报「查了 167 种」绿，主 agent 观测，我没复跑）：
  - 「必红」格（`scripts/gate/merge-and-verify-selftest-lib.sh:683`）：表里别批的行锚在 `merge_table.txt` 的原文 `1`，补丁把它改成 `2`。核不在 → 没有 ANCHOR 行、退 0，格红；基线没摆回原样（`before` 与补丁之后相同）→ 基线里也 0 次、被扣成旧红、退 0，格红；所以这格同时分得出「核在不在」与「基线是不是补丁之前的树」。
  - 「必绿」格（`scripts/gate/merge-and-verify-selftest-lib.sh:690`）：表里另有一行补丁之前就 0 次的行，补丁与它无关，要求退 0、有「变异锚点核过」与「补丁前就红的 1 处」、不点名那一行。核不在 → 没有「核过」那一句，格红；基线不扣 → 退 1、点名了旧红行，格红。
  - 分不出的（没样本）：见 2.1 末（新建文件、新追加坏行、只改表、范围外文件段）。
- 两条开关各红在点名那一格（读码推的；证红命令 `code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh` 主 agent 在跑，我没读到结果）：
  - `anchor-check-skipped`（`scripts/gate/merge-and-verify-steps-lib.sh:375`：「if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "anchor-check-skipped" ]]; then」）：一进来就放行 → 必红格拿不到 ANCHOR 行（实际退 0）红、必绿格拿不到「变异锚点核过」红；两格都在文件头开关表点名了。
  - `anchor-check-baseline-ignored`（`scripts/gate/merge-and-verify-steps-lib.sh:383`：「[[ "${MERGE_AND_VERIFY_BREAK:-}" != "anchor-check-baseline-ignored" ]] || export APPLY_WRITER_PATCH_BREAK=static-check-baseline-ignored」，在子 shell 里设、不外漏）：基线扣不掉 → 必绿格点名旧红行、退 1，格红；必红格仍红在原因上（照样点名 merge-data-row），不红，与表里只点名必绿一格一致。读开关走的是 `apply_writer_patch_plan.py` 的 `static-check-baseline-ignored`（`scripts/gate/apply_writer_patch_plan.py:1134`），它是 `apply-writer-patch.py` 自己已有的开关，没新写第二份判法。
- sop-first 的要求（`.claude/singlefs-ai-sop/rules/sop-first.md@加门禁或钩子之前，先找已有的`：「有管同一件事的，把新判据追加进它」）：新增的是一个调用点，不是新门禁文件；判法复用 `static_check_report`（`scripts/gate/merge-and-verify.sh:2033`）、拷贝闭包复用 `static_check_tree_items` 与 `repository_copy_module`（同一内联程序 1992、1999 行），没有第二份「比基线判红」的写法。
  新写的只有「摆补丁之前的树」与「把合成补丁按拷贝范围切段」两小段（约 40 行 python），在 `apply_writer_patch_plan.py` 里没有现成函数（它的 `build_patched_tree` 只摆补丁之后的那一棵，`scripts/gate/apply_writer_patch_plan.py:1090`：「def build_patched_tree(root, merged, patch):」）。
  工具员报告「已有的管这件事的只有两处」的依据是它读码；它存的 `gate-overlap-list.txt` 是 73 行的登记清单，里面没有 `apply-writer-patch` 或 `mutation-tables` 的行（对这两个词与「变异锚点」`grep -n` 都是 0 命中，量的），这一句不是那份清单给的。

### 2.5 代价（第二节第 5 条）：会明显拖长；没发现撞内存上限的路径，但这一趟里没有量到内存

量的（主 agent 换上之后头一趟真用的日志目录 `~/.local/share/singlefs/work/merge-and-verify/2026-11-20-3774803/`，下面的秒数是几个日志文件的修改时刻相减，命令 `stat -c '%.3Y %n' <日志目录>/main-tree-at-start.log <日志目录>/merge-patch.log <日志目录>/anchor-check.log <日志目录>/test-plan.singlefs-core.arguments <日志目录>/rebuilt.log`，输出是 epoch 秒；原输出不贴，只贴相减结果）：

| 区间 | 起止文件 | 秒 |
|---|---|---|
| 变异锚点核（两趟 static-check + 两次拷树） | `merge-patch.log` → `anchor-check.log` | 625 |
| 核结束到测试计划开写 | `anchor-check.log` → `test-plan.singlefs-core.arguments` | 4 |
| 四步 cargo 与重编核对 | `test-plan.singlefs-core.arguments` → `rebuilt.log` | 529 |
| 记开跑指纹到重编核对写完 | `main-tree-at-start.log` → `rebuilt.log` | 1199 |

- 这一趟里核占挂钟约一半（625 / 1199）。代码注释写每趟约 35 秒（`scripts/gate/apply_writer_patch_plan.py:49`：「STATIC_CHECK_TIMEOUT_SECONDS = 900   # 推的：真仓上一趟约 35 秒」，正文第二节转述成半分钟到一分钟），这一趟没有成立：625 秒里两趟 static-check 占绝大部分，一趟约 5 分钟量级（推的：两次 rsync 拷约 60 MB 的 `crates/` 与小目录，`du -sh crates` 为 59M，量的，拷贝用时可忽略）。是 static-check 本身变慢、还是那一刻机器上别的任务在抢，我分不开（量的是挂钟，不是 CPU）；第 6 节第 2 条的探针（control 模式会打「核 N 秒」）可以在别的任务较少的时候再量一次，主 agent 可对照。
- 每趟 static-check 的超时是 900 秒（同一行）：挂钟若再慢一倍，补丁之后那趟可能超时，走「起不了或超过 900 秒」→「判不了」→ 红（`scripts/gate/apply_writer_patch_plan.py:1043`：「return None, [f"起不了或超过 {STATIC_CHECK_TIMEOUT_SECONDS} 秒：{error}"]」）。安全一侧，但会让整趟红在核上。推的，没量过。
- 内存：两趟 static-check 先后跑，不并行（`scripts/gate/apply_writer_patch_plan.py:1117`：「# 两趟先后跑、不并行：每趟对整个工作区做可达性判约 16 GiB，并行时峰值 30.90 GiB，挡住两台排着的别的任务」；16 GiB 是那条注释的数，推的，我没量）。核在四步 cargo 之前跑完，所以这一趟的内存峰值是核与 cargo 各自的最大值，不是相加（推的）。这一趟真跑是第一次尝试就跑完的（`task-attempt` 文件的第二个字段是 1，量的），没有被服务放大重排。
  若服务给这条命令的内存上限比核的峰值小，被杀发生在这一趟的前几分钟（快照拷好、打补丁、算补丁目录之后），重排浪费的是快照与打补丁，不是 cargo（推的；放大重排的规则在 `scripts/gate/merge-and-verify.sh` 文件头「上限、线程、机器」一段）。
- 重复付费：每一次整批重交（出路改补丁、只重跑红的项带 `--item`）都再付一次约 10 分钟，没有按键复用（核不进按条复用的库，因为它的结论取决于整批补丁与整棵拷贝范围，不是某一项）。红的出路要整批重交，等于每改一轮多一趟。
- 拷树占盘：两份拷贝范围树（`before/` 与补丁之后那一棵）放在 `TMPDIR` 或 `/tmp` 下，收尾删（`scripts/gate/merge-and-verify.sh:2049`：「shutil.rmtree(scratch, ignore_errors=True)」；另一份由 `static_check_report` 自己的 `finally` 删）；`prune-server/target` 等编译目录被 `copy_entries` 排除（`scripts/hosts/lib_repository_copy.py:878`：「excludes = ["--exclude", ".git", "--exclude", "target/", "--exclude", "target-*/", "--exclude", "__pycache__"]」）。
  被杀、断电时内联程序的临时目录会留在 `/tmp`（`finally` 不跑），没有登记谁来清（读码，推的）。
- 要不要挪到 cargo 之后：工具员报告提过。挪后核的结果要等到 cargo 之后才看见，而核的红往往意味着补丁要重做、cargo 的结果作废；挪前已经把「核红」的这一趟放在最早，内存被杀也最便宜。我的看法：现位置合理，代价在挂钟，不在内存；要省挂钟，可在核红时直接停（省约 530 秒），但这与别的步骤「记一项、接着跑」的做法不同，由主 agent 定。

## 3. 没做成的几件、与「没量过」的清单

- 真 static-check 上的「红」那一支没量过（只量过绿）：见第 6 节第 1 条；带 `break` / `control` 两种模式。
- 两条新开关的证红结果我没读到（主 agent 在跑）；我只读了开关代码与两格样本，推出各红在哪一格。
- 函数单元键对新内联程序（`sys.argv[1]` 直接进 `spec_from_file_location`）报不报拒算：我没法判。同类的 `MERGE_PATCH_PROGRAM` 也是这个形，换上后自证已经 `exit=0`（主 agent 观测），这一条对口登记的事不归我，写进来只为不漏：复核不了，交主 agent 看门禁的函数单元键有没有新拒算。
- `scripts/gate/README.md` 里对合入后验证的说明有没有同步：没看。
- 临时目录泄漏（被杀时 `/tmp` 下的 `merge-anchor-check-*` 与 `apply-writer-patch-static-check-*`）：读码推的，没量过；清不清、谁清，由主 agent 定。

## 4. 什么现象会推翻上面的结论

| 结论 | 推翻它的现象 |
|---|---|
| 做的就是第一节说的、能拦住那一形 | 第 6 节第 1 条在真树上退 0 或没点名那一行（`break` 模式）；或同一批补丁已在主工作区上、而核仍没红 |
| 判法不会把旧红当新红 | 第 6 节第 2 条（`control` 模式）在真 static-check 上退 1，点名的行不是新弄断的（说明基线摆得不对，或 static-check 对没有 `.git` 的树行为不同） |
| 带 `--skip-apply` 不核是对的 | 有一条不经合入后验证、补丁没落主工作区却也不核的合入路径；`grep -n "skip_apply"` 能找到 `dirs` 不空而 `skip_apply` 为 1 的调用 |
| 核没做成一律红 | 任何一种退出码非 0 的核却被 `verify` 当过了：即 `scripts/gate/merge-and-verify-steps-lib.sh:386` 的「if ((rc == 0)); then」之外还有读日志当过的分支（我读到的没有） |
| 红了不合成立 | 红的那一趟输出里仍有「合进主工作区」的整条命令（自证必红格的第三条会抓） |
| 代价主要在挂钟、不在内存 | 同一条命令在同一台上被服务因内存放大重排过（日志目录里的 `task-attempt` 第二个字段大于 1，且重排发生在核里） |
| 两格样本分得出「在不在、基线用没用」 | 两条开关证红时，点名的格与实际红的格不是逐格相同 |

## 5. 草稿目录里的东西与复跑说明

- 没写模型（这一题读码即可，没有要另建的计数模型），所以没有 `rerun.sh`；唯一的新文件是真 static-check 探针脚本 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/tools/anchor-real-probe.sh`（绝对路径 `/home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/tools/anchor-real-probe.sh`），sha256 `fae77b3770c026d865013456cd16c868b73f4119f608e4ff7e5040228dbec657`（`sha256sum` 量的）；`bash -n` 过、抠出来的内联程序 `python3 -m py_compile` 过；没有跑。
- 脚本做什么：`minimal-copy.py` 拷一份快照、在变异表里挑一行原文在某个 `crates/` 下 `.rs` 里恰好命中 1 次的行，`break` 把那一处改掉（命中 0 次）、`control` 只在文件末尾加一行注释；造 `merge-baseline`、路径单子、`merge-patch/crates.patch`（用 `git add` 后 `git diff`，与 `MERGE_PATCH_PROGRAM` 同一做法）；把 `scripts/gate/merge-and-verify.sh` 里 `MERGE_ANCHOR_CHECK_PROGRAM` 的正文用 `awk` 原样抠出来，对这份快照跑一遍。不改主工作区，快照收尾删，日志留在运行目录的 `logs/`。
- 探针用的是真 `crates-mutation-rows.py static-check`，每趟约 16 GiB（推的），先后各交一条、不要同时交。
- 草稿目录 `~/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/`（`du -sh` 40K，量的）里只有这个脚本与认领文件，没有仓副本与编译目录，没有要删的；探针的运行目录（`probe-break-run`、`probe-control-run`）由主 agent 跑完删。

## 6. 要主 agent 跑的命令

在主工作区仓根起；先 1 后 2（各约 16 GiB，不并行）。

1. `break` 探针（真 static-check 上「补丁把别批变异行原文 1 次变 0 次」是否判红）：
   `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/tools/anchor-real-probe.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/probe-break-run break`
   起在：仓根。看：`TASK_SERVER_RESULT` 末行 `type=succeeded exit=0`；`task-server/task log <号>` 里有「选中的行：<变异名>（<路径>），模式 break」「核 N 秒，退 1」「✓ break：判红，点名 <变异名> 命中 0 次」；`exit=1` 带「✗ break：…」是没拦住（推翻第 1 节结论），`exit=2` 是探针没准备好（minimal-copy 或抠程序失败，看日志里点名的文件）。
2. `control` 探针（真 static-check 上不相干的补丁不判红；同时给出核本身的挂钟）：
   `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/tools/anchor-real-probe.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-anchor-check-tooling-r1-sonnet/probe-control-run control`
   起在：仓根。看：`exit=0`、日志里「核 N 秒，退 0」「✓ control：判绿」；`exit=1` 带「✗ control：…」是旧红被当新红或判不了（推翻 1.3 / 2.1 的「不会把旧红当新红」）；N 与上面表里的 625 秒对照（在别的任务少的时候看）。
3. 两条开关的证红（主 agent 已在跑，列在这里只为对口）：看汇总行「复跑了 N 条开关」N 比原来多 2、`自证红且点名的格与实际红的格逐格相同`；`anchor-check-baseline-ignored` 只红在「变异锚点核：补丁之前就失效的变异行不算这一批弄断的」那一格，`anchor-check-skipped` 红在文件头开关表点名的两格（2.4）。
4. 复核第 6 节之前的几个读数（只读、直接跑）：`ls -t -l <日志目录>` 与 `stat -c '%.3Y %n'` 的那条命令（2.5 开头）——换成这一次探针或下一趟真合入后验证的日志目录，看核的挂钟有没有变。

## 7. 判定一览

冲突的两边整行并排（引文都在文件里 `grep -nF` 过）：

| 格 | 应当是（同类做法 / 代码注释） | 被判对象 |
|---|---|---|
| 出路里的「快照」 | `scripts/gate/merge-and-verify-steps-lib.sh:569`：「没留快照的照原命令带 env MERGE_AND_VERIFY_KEEP_COPY=1 重交一趟」 | `scripts/gate/merge-and-verify-steps-lib.sh:393`：「以这一趟留下的快照 $snapshot 为基线」（没写没留快照怎么办） |
| 核的挂钟 | `scripts/gate/apply_writer_patch_plan.py:49`：「STATIC_CHECK_TIMEOUT_SECONDS = 900   # 推的：真仓上一趟约 35 秒」 | 真跑量到核整道 625 秒（两趟 static-check 加两次拷树，量的，2.5） |

| 格 | 判定 | 一句话 |
|---|---|---|
| 第一问：做的是不是第一节说的 | 一致 | 调用点、摆树、比基线、不打合入命令都对得上；只量过绿，红那支待主 agent 跑（第 6 节第 1 条） |
| 第一问：拦不拦得住 1→0 那一形 | 一致（推的，没量过红） | 补丁前命中 1 次、补丁后 0 次，ANCHOR 键在基线里没有 → 新红 |
| 2.1 判法：补丁前红算成新红 | 一致 | 基线是补丁之前的树 + 表，键去行号后按多重集扣；绿那一支真树上量过 |
| 2.1 判法：新红漏成旧红 | 一致 | 键含命中次数，没找到漏的输入；推的，没量过 |
| 2.1 判法：多份补丁合成覆盖 | 一致 | 核终态，不读合成出的三份变异表文件 |
| 2.2 `--skip-apply` | 一致 | `dirs` 空，不进核；补丁已在主工作区，无合可拦 |
| 2.2 合起来没有改动 | 一致 | 四份文件全空才不做 |
| 2.2 自证里的非标记样本树 | 一致 | 只核带标记的，别的格不受影响 |
| 2.2 只改变异表的补丁 | 一致（无样本） | 核照跑；没有只改表的样本格，推的 |
| 2.2 核没做成是否一律红 | 一致 | 范围算不成、判不了、套不上、被杀，shell 侧只认退 0；被杀的整条由服务重排，推的 |
| 2.3 红了不合 | 一致 | `merge_in_commands` 只在 verify 退 0 或 3 调；红退 1 |
| 2.3 出路能不能照做 | 冲突（小） | 缺「没留快照怎么办」一句，与同类出路并排见上表；平常带 KEEP_COPY 时走得通 |
| 2.3 点名行数 | 规则没说 | 最多逐行列 20 处，多的只报计数；一趟弄断超过 20 行要多改一轮 |
| 2.4 两格分辨力 | 一致（推的） | 必红格分得出核在不在与基线是不是补丁前的树，必绿格分得出基线扣没扣；新建文件、新追加坏行、只改表无样本 |
| 2.4 两条开关各红在点名格 | 一致（推的，证红结果没读到） | skipped 红两格、baseline-ignored 红必绿一格 |
| 2.4 复用而非另写（sop-first） | 一致 | 判法复用 `static_check_report`；新写的是摆树与切段，约 40 行 |
| 2.5 代价：挂钟 | 冲突 | 真跑核占 625 秒、约占整趟一半；注释写一趟约 35 秒，并排见上表 |
| 2.5 代价：内存 | 复核不了 | 这一趟里没量内存；注释的 16 GiB 是推的；两趟先后跑、核在 cargo 之前，峰值不相加（推的）；本次一次尝试跑完，未被重排 |
| 2.5 代价：重复付费 | 规则没说 | 每次整批重交都再付一次，没有复用；是否要缓存或红了就停由主 agent 定 |

## 8. 没做什么

- 不是判别的腿：这一轮我是正推，没有替主 agent 采纳或出判决；上表「一致 / 冲突」是读码的判定，不是判决。
- 没往统一任务服务交任务、没起任何脚本、没跑门禁；判决要引的数（探针的红绿、核的挂钟、两条开关的证红）都没复跑，命令在第 6 节。
- 读的是主工作区此刻的四份与 `apply-writer-patch.py`、`apply_writer_patch_plan.py`、`crates_mutation_rows_static.py`、`lib_repository_copy.py`；没读禁读清单里的同轮攻方腿的报告、模型与草稿。
- 没改主工作区任何文件；没碰 `apply-writer-patch.py`；没验 `scripts/gate/README.md`、没验函数单元键对新内联程序的登记（见第 3 节）。
- 内存数（16 GiB）、「各约半分钟」「35 秒」都是别处写的推的数，我没量；挂钟 625 秒、1199 秒是日志文件修改时刻相减，量的，但受同机别的任务影响，没分开。
