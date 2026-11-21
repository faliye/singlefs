# 治理文件轮第三轮正推腿报告（ledger-open-fixes-governance-r3，第 6 到 8 问）

读的是主工作区此刻的文件（对着 HEAD 的 `git diff HEAD -- <文件>` 存在草稿目录，只读）。方案按 `crates/` 今天的实现来谈：这一轮被判的文件都在 `scripts/`、`.claude/` 下，与 `crates/` 的交点只有 doc-experiments 两格认 `crates/*/src/bin/` 下的装置，下面逐处核了。

范围：只读了「范围」一节里的八类文件和 19 份新样本里的 17 份（evidence-in-repo 六份、pinned-apparatus-revision 十一份的 setup.sh 与 expect），另两份（`-added-line-without-pin-red`、`-pin-in-other-section-red`）只读了头注释与 expect。没有跑任何东西：能跑的都写在末尾「要主 agent 跑的命令」。

## 总答

- 第 6 问：成立。六条都做了判决那一句说的事，逐条的文件:行与判决原句见下。
- 第 7 问：成立。三处偏离都偏得对，不用改回字面；各带要补的东西：取「每一条」是收严、代价大（17 条 timing 行 15 条产物不在盘）；钉值放宽在装置没变时站得住，但缺样本、成功句与 `gate-cell:` 行没跟上；换上工具名单按读码推，主 agent 那一次复跑撑着，`scripts/gate/hook-swap.py` 不在复跑登记里。
- 第 8 问：大部分成立；两格推不出：完工清理闸的 `nested-undeclared-copy-blocks` 与 `nested-lower-handover-main-listed` 只被通用开关证红，没有一条开关证明它们红在嵌套读法上。另有两处小缺口：timing 产物只存着 `.gz` 那一支没有样本，只改钉值而钉值已在第十二节的那一支没有样本。

## 第 6 问：第二轮「三、改法」六条逐条核代码

### 6.1 换上工具（判决第 1 条）：成立

判决原文：「`.taking` 里写持有者身份（与记录同一套），判它挡不挡照记录那一套（同一台看进程、另一台看修改时间）」（`research/prompts/ledger-open-fixes-governance-r2-main-verification.md:52`）。

- 写身份：「content = scratch_lock_content()」（`scripts/gate/hook-swap.py:2884`），与占用记录同一个生成函数，经 `link_scratch_record` 写同目录临时文件再 `os.link` 成 `.taking`。
- 判挡不挡：「scratch_holder_occupies(holder, age_seconds, mode)」（`scripts/gate/hook-swap.py:2900`），与判记录的是同一个函数，同一台看进程、另一台看 `age_seconds` 对 `SCRATCH_LOCK_STALE_SECONDS`。
- 不再占着的当场删掉重来：同一个循环里 `os.unlink(taking)`（`scripts/gate/hook-swap.py:2904` 一带），循环上限 `SCRATCH_LOCK_ATTEMPTS`。

判决原文：「TERM 与 HUP 处理里放掉这一趟占着的 `.taking`」（同上 `:52`）。

- 登记：「HELD_REMOVAL_CLAIMS.append(entry)」（`scripts/gate/hook-swap.py:2887`），放在 `os.link` 之前，TERM 落在 link 做成与返回之间也放得掉。
- 放掉：「for taking, content in list(HELD_REMOVAL_CLAIMS):」（`scripts/gate/hook-swap.py:2991`），在 `release_then_die` 里先放 `.taking` 再放草稿目录与仓的记录。
- 信号：「HANDLED_SIGNALS = (signal.SIGTERM, signal.SIGHUP)」（`scripts/gate/hook-swap.py:2679`），TERM 与 HUP 都在。

判决原文：「摘旧记录那一次 `link` 报已有而读回来是自己的，当放成」（同上 `:52`）。

- 「return read_scratch_record(path) == content」（`scripts/gate/hook-swap.py:2952`）：`link_scratch_record` 的 `except FileExistsError` 里读回来相等就交 True；`content` 里有随机串与开始秒数，读回来相等就是自己写的。`.taking` 走的是同一个函数（`scripts/gate/hook-swap.py:2891`）。

判决原文：「拒绝说明照新判法改」（同上 `:52`）。

- 「摘的那一趟若已被杀」（`scripts/gate/hook-swap.py:2762`）：`stale_holder_hint(removing=True)` 写「`.taking` 放了 300 秒就不再挡」，与 `scratch_holder_occupies` 对另一台起的看修改时间的判法一致；`SCRATCH_LOCK_STALE_SECONDS = 300`。

判决原文：「每一种起因先在自证里做成会红的格」（同上 `:52`）。

- 判决列了三种起因（KILL、TERM、link 回复丢了），代码做了四格：KILL（`scripts/gate/hook-swap.py:4865` 一带，按仓的真换、`--seal-outcome`、草稿目录三处各一遍）、TERM（`scripts/gate/hook-swap.py:4881` 一带，没占着与已占着草稿目录各一趟）、回复丢了（「os.link = link_reports_exists_for_taking」，`scripts/gate/hook-swap.py:4907`）、还占着的挡住（「("这一台起的、进程还在", {}, 0,」，`scripts/gate/hook-swap.py:4929`）。第四格是判决没列的反方向（摘的那一趟还在时不能放过），收严，不算偏。
- 数目缺口（小）：KILL 那一格前面没有 `checked += 1`（grep 出的计数点是 4786、4877、4894、4925，4786 与 4877 之间没有别的）。成功句「查了 N 种」比实际判的格少一格；不影响红绿。

本条要打破它的现象：同一台看进程这一支，`.taking` 里写的进程号已不在而 `.taking` 仍挡后来的一趟（自证里 KILL 格的三处任一处）；TERM 之后盘上还有 `.taking`。

### 6.2 evidence-in-repo（判决第 2 条）：成立（有两处小毛病，见 8.2）

判决原文：「按 timing 行点名的那一份产物比修改时刻，exact 行的产物与不相干的文件不算；`crates/*/src/bin/` 下的装置一并认」（同上 `:53`）。

- 取 timing 行点名的产物：「timing_path = product_path_by_file_name.get(timing_name) or product_path_by_file_name.get(timing_name + ".gz")」（`.claude/gate.d/doc-experiments.sh:1128`）；比的对象 `newest = min(...) if timing_present and not timing_missing else None`（`.claude/gate.d/doc-experiments.sh:1136`）：每一条都要在盘，取最旧的那一份比。
- exact 行不算：有 timing 行的实验从 exact 表里剔掉：「replay_exact_product_by_number.pop(timing_number, None)」（`.claude/gate.d/doc-experiments.sh:1085`）。
- 不相干的文件不算：有 timing 行时不再看 `newest_product_by_number`，分支在 `.claude/gate.d/doc-experiments.sh:1125` 的 `if timing_names:`。
- 认 `crates/` 下的装置：「re.compile(r"^crates/[^/]+/src/bin/e([0-9]+)_[^/]*\.rs$"),」（`.claude/gate.d/doc-experiments.sh:1004`）与它下一行的目录形；名字不是 `e<号>_` 起头的 `.rs` 不进对象。判决写的是「下的装置一并认」，代码认的是其中 `e<号>_` 起头的那部分，与「装置」的定义（实验装置）一致。

本条要打破它的现象：只重跑 exact 驱动、另存一份更新的 exact 产物或放一份 `e<号>` 起头的笔记，格仍然绿（样本 `evidence-in-repo-exact-only-rerun-red`、`-unrelated-file-red` 判绿就是打破）。

### 6.3 删旧闸（判决第 3 条）：成立

判决原文：「两条 git 的路径加 `:(literal)` 前缀，四份入口同改」（同上 `:54`）。

- 四份入口各一行，两条 git 共用这个变量：
  - 「output_pathspec=":(literal)$output_name"」（`scripts/experiment-apparatus/e07-vm-bench-run.sh:416`）
  - 「output_pathspec=":(literal)$output_name"」（`scripts/experiment-apparatus/e53-vm-bench-run.sh:439`）
  - 「output_pathspec=":(literal)$output_name"」（`scripts/experiment-apparatus/e159-vm-bench-run.sh:436`）
  - 「output_pathspec=":(literal)$output_name"」（`scripts/experiment-apparatus/e56-vm-bench-run.sh:410`）
- 两条 git 在 e07 是 `git -C "$output_directory" ls-files --error-unmatch -- "$output_pathspec"`（`scripts/experiment-apparatus/e07-vm-bench-run.sh:418`）与 `git -C "$output_directory" diff --quiet HEAD -- "$output_pathspec"`（`:419`）；另三份同形：四份里 `output_pathspec` 各命中 4 行（赋值、开关覆盖、两条 git 各一行，e53 在 `:441`、`:442`，e159 在 `:438`、`:439`，e56 在 `:412`、`:413`）。

本条要打破它的现象：输出名带 `[2]` 而没进 git、同目录有已提交的同名通配匹配项时，入口仍把它当已提交的删掉（样本格 `cell_existing_output_name_with_brackets`，`scripts/experiment-apparatus/e07-vm-bench-run.sh:268`，`committed == 0` 断言守住了「那份已提交的确实建成了」）。

### 6.4 钉值格（判决第 4 条）：成立（五小项都对；放宽一处见 7.2）

判决原文：「钉值变了或装置变了」「修订认法改成登记第十二节新加的一行里出现新钉值（至少前 12 位）」「钉值行允许行尾注释」「登记不在树里时从 git 历史里取它被删前的那一版判」（同上 `:55`；第二、三句是一整行里的片段）。

- 触发：「pin_changed = pin != base_pin」（`.claude/gate.d/doc-experiments.sh:4674`）与 `apparatus_changed = source in changed`（`:4673`）；`if not apparatus_changed and not (pin_changed ...)` 才算没变（`:4676`）。
- 修订认法：装置变了时 `added = nonblank(section) - nonblank(previous_section)`，且新加的行里要出现新钉值前 12 位（`:4736`、`:4741`），另加两条判决没点名的收严：钉值前 12 位基准那一版的第十二节里就有的不算（`:4745`），基准那一版写着旧钉值的行一行都不能少（「lost = nonblank(line for line in previous_section if base_pin[:12] in line.lower()) - nonblank(section)」，`:4755`）。这两条补的是判决同一条里点名的「原地改写、抄一行已有的、词中间多一个空格」三种写法，收严，不算偏。
- 行尾注释：「PIN = re.compile(r"^registered_apparatus_sha256=([0-9a-f]{64})[ \t]*$" if "pin-trailing-comment-unrecognized" in broken」（`:4519`），不开开关时的那一支是 `(?:#.*)?$`。
- 登记取脚本里写明的那一处：`LABELED_REGISTRY`（`.claude/gate.d/doc-experiments.sh:4524`）要「跑前登记」后接路径；注释里头一处点名别的登记的认错登记那一种，`registry_of` 只收写成「跑前登记 <路径>」的，写了两处不同的判红（`:4576`、`:4578`）。
- 不在树里取历史：「def deleted_registry_text(path):」（`.claude/gate.d/doc-experiments.sh:4627`），先取基准那一版，再 `git log -1 --diff-filter=D` 取删它前的那一版。

本条要打破它的现象：只改钉值、登记没动判绿（`-repin-only-red` 判红就是证）；在登记里把原值原地改写、复制一行或只改词间空白判绿（三份样本 `-inplace-rewrite-red`、`-duplicated-line-red`、`-internal-whitespace-red` 判红就是证）。

### 6.5 完工清理闸（判决第 5 条）：成立

判决原文：「下层已声明交接的副本，上层交回时算已声明」（同上 `:56`）。

- 取下层声明：「exempt.update(declarations([path], nested_role)[0])」（`.claude/hooks/completion-cleanup-guard.sh:880`），条件是 `nested_role is not None and depth > 0 and kind is None and DECLARATION_FILE in names`（`:879`）：下层不是副本、有 `.deliverables`。
- 只给子 agent 读：「nested_role = None if broken("nested-declaration-ignored") or role != "subagent" else role」（`.claude/hooks/completion-cleanup-guard.sh:1962`）；主 agent 不读下层的（文件头：「主 agent 不读下层的」，`.claude/hooks/completion-cleanup-guard.sh:104`）。
- 取的是「这个角色免列的」种类（「种类取这个角色免列的」，`.claude/hooks/completion-cleanup-guard.sh:104`）：比判决字面「交接」宽，包括子 agent 本来就免列的报告、补丁、备份。偏宽但不放过任何子 agent 自己写也会被放过的东西，不算偏错。
- 下层写坏的行不拦上层：自证 `nested-lower-handover-passes` 的下层文件里放了一行写坏的 `snapshot 留底`（`.claude/hooks/completion-cleanup-guard.sh:2704` 一带）。

本条要打破它的现象：下层声明了交接的副本，上层交回仍被拦（自证 `nested-lower-handover-passes` 红）。

### 6.6 `stage-owners.tsv:143`（判决第 6 条）：成立

判决原文：「改成封存的说法」（同上 `:57`）。第 143 行现在写「新内容不往任何账本里登」（`.claude/gate.d/stage-owners.tsv:143`），对得上 `CLAUDE.md` 的「是旧账本，不再往里登」（`CLAUDE.md:21`）。同批改的第 117、130 行见 7、8 问；第 142 行（`ledger-section-numbers`）也同口径改了，不在判决点名的行里，只是顺手同改。

## 第 7 问：偏离判决字面的三处，偏得对不对

### 7.1 evidence-in-repo 取「每一条都要」而判决写「那一份」：偏得对，不用改回字面

- 字面做不成：判决写「按 timing 行点名的那一份产物」（`research/prompts/ledger-open-fixes-governance-r2-main-verification.md:53`）。门禁自己的头注释写明「装置源码与复跑表的行之间没有机读的对应」（`.claude/gate.d/doc-experiments.sh:908`），一个实验有几条 timing 行时，装置这次改的是哪一段、该看哪一份，门禁认不出来。能实现的只有两种：每一条都要，或取其一。
- 取其一就是弄坏开关 `timing-newest-row-only` 的写法（「只比点名的产物里最新的那一份」，`.claude/gate.d/doc-experiments.sh:299`），在它下面样本 `evidence-in-repo-timing-rows-red` 的两个实验判绿：「只重跑其中一条 timing 行就放过」，是判决里「timing 那一段改了而没有复跑」的变体。
- 方向是收严：判决要的是「不能凭一份无关的新产物放过」，每一条都要比「那一份」多拦、不少拦。evidence-discipline 的收严规矩也满足（说得出多要求了什么：每一条 timing 行的产物都要在盘且不旧）。
- 实测代价（量的）：`scripts/experiment-apparatus/replay.sh` 里 timing 行 17 条，其中 15 条点名的产物此刻不在 `research/results/` 下（`.out` 与 `.out.gz` 两种名都找了，含子目录）。命令与输出：

```
$ while IFS='|' read -r num bin args prod kind; do if [ "$kind" = timing ]; then ...; fi; done < <(grep -E '^E[0-9]+[A-Z0-9]*\|[^|]*\|[^|]*\|[^|]*\|(exact|timing)$' scripts/experiment-apparatus/replay.sh)
missing: E44 e44-jsn-width-2026-08-30.out        （另 E58、E140、E107、E17、E20、E21、E128、E144 各一行）
missing: E181 e181-landing-check-fixed-point-and-mount-lower-bound-landing.out
missing: E162 e162-crash-verdict-block-store-s1-all-arms.out    （同一个实验另两行 s2、s4 也不在）
missing: E161 e161-crash-state-dedup-and-time-split-segment-one.out
missing: E184 e184-regular-layer-cost-counts-segment-one-2026-10-24.out
total_timing=17 missing=15
```

  也就是说，这 15 条所属的实验下一次有人改装置，这一格一定红「timing 行点名的产物不在盘」，出路只有重跑存回（或在按要求才跑的登记里）。`scripts/experiment-apparatus/on-request-experiments.tsv` 现登记 E152、E142、E158、E162、E172、E174、E178、E186、E187：E162 在、E161、E181、E184 不在。E184 的情形还有一处对不上：盘上是 `e184-regular-layer-cost-counts-segment-one-2026-10-19.out`，`replay.sh` 那一行写的是 `…-2026-10-24.out`（`ls research/results | grep -E '^e(162|161|184|181|174)'` 只列出 E174 三份与这一份 E184）。这是前后两处登记不一致、不是第三轮的偏差，写在这里给主 agent 判要不要另修。
- 这一代价不改变「偏得对」：这 15 条分属 13 个实验，用 `find research/results -name "e<号>[!0-9]*"` 数，12 个实验在盘一份产物都没有（旧写法比「这个实验号在盘最新的那一份」也红，`.claude/gate.d/doc-experiments.sh:1171` 一带）；只有 E184 在盘有一份（`-2026-10-19.out`），旧写法可能绿、新写法红。新写法多红的就是「有几份、但不是每一条 timing 行点名的都在」那一类。

什么现象会推翻：能给出一个「一个实验有几条 timing 行、装置这次只改了其中一段、不重跑别的段也该绿」的真实例子，且门禁有办法认出是哪一段（现在没有机读对应，头注释 `:908` 自己写了）。

### 7.2 钉值格只改钉值时「第十二节里写着新钉值前 12 位即可」：偏得对，有三处要补

判决字面：「修订认法改成登记第十二节新加的一行里出现新钉值（至少前 12 位）」（`research/prompts/ledger-open-fixes-governance-r2-main-verification.md:55`）。代码在只改钉值（装置没变）时放宽成：「elif not any(has_pin(line) for line in section):」（`.claude/gate.d/doc-experiments.sh:4748`）才判红，也就是新钉值前 12 位出现在第十二节任何一行都过，不要求是这一次新加的行。

为什么偏得对（推的，由代码与起跑脚本读出）：
- 起跑脚本开跑时核「装置源码的 sha256 要等于登记第十二节写的那一个」（`scripts/experiment-apparatus/e185-r4-segment-run.sh:18`），不等退 3，比较在 `scripts/experiment-apparatus/e185-r4-segment-run.sh:485`。装置没变的那一次，要让起跑脚本真能开跑，钉值只能等于装置此刻的 sha256；这个值在第十二节里已经写着，就说明装置此刻这一版是登记过的。判决当初怕的是「装置改了、登记没跟着写、只把钉值改成现算值」，装置改了的那一支（`apparatus_changed`）仍按判决字面要新加行并且加了三条收严，那一支没放宽。
- 真例子：三份 E185 起跑脚本 HEAD 里的钉值都是 `a5b48b73…`，今天改成各自登记里写着的值；r4 登记第十二节本来就写着 `9e813cbd…`（`research/prompts/e185-r4-prereg.md:579`）。照判决字面，要为改回一个登记里本来就有的值再补一行没有内容的修订。
- 钉回「已被后续修订作废的旧值」也放行：是，这是放宽的直接后果（第 3 问攻方腿的角度）。起跑脚本那一道停机在装置不是那一版时拦（上一条）；门禁这一格对它没有判法，头注释已写明管不到「钉值是不是装置此刻的 sha256」（`.claude/gate.d/doc-experiments.sh:4490` 一带）。

要补的三处（都不是偏错，是放宽没配齐）：
1. 没有样本罩住「钉值改成第十二节里本来就写着的值、不新加行、判绿」这一支：只改钉值的样本只有 `-repin-only-red` 与 `-repin-only-green`，后者的 setup 新加了一行（`registry 934 …` 末尾的 `R2-1 重钉` 行）。grep 命中证据：下面「8.4」里的命令。把放宽这一句删掉（要求新加行），没有一份样本变红；也没有开关点它（`pin-seen-in-base-allowed` 只管装置变了那一支，`:4745`）。
2. 成功句不对：「装置没变而钉值改了的 {len(repinned)} 份，登记第十二节都新加了写着新钉值的行」（`.claude/gate.d/doc-experiments.sh:4776`），放宽之后这些份不一定新加了行。已归档那一支同样：它把份数记进 `revised`（`:4726`），成功句说「都新加了修订」，而那一支只核历史里那一版第十二节出现过钉值。
3. 头注释不一致：`gate-cell:` 行写「钉值或装置源码在这次改动里变了，它的跑前登记第十二节要在这次改动里新加一行写着新钉值的修订」（`.claude/gate.d/doc-experiments.sh:32`），判法注释写的是放宽后的说法（`.claude/gate.d/doc-experiments.sh:4486`），两处互相矛盾。`stage-owners.tsv:130` 写的是放宽后的说法。

另一个一并报的不对称（推的，没有样本）：装置变了的那一支要求新钉值前 12 位基准那一版第十二节里没有（`:4745`），所以把装置回滚到登记里写过的老版本、钉值改回老值，没有任何写法能过（新加一行也带着基准里已有的前 12 位）。已归档登记那一支反过来：装置变了、钉值没改、登记历史里写着这个钉值，判绿（样本 `-archived-history-green` 就是这一形：`apparatus 943 v2` 之后起跑脚本钉值还是 v1 的）。两支对「装置变了而钉值没变」的判法不一致；因为起跑脚本开跑那一道停机在，不会真的放过一次错的开跑。

什么现象会推翻「偏得对」：构造一份起跑脚本，装置此刻的 sha256 前 12 位不在登记第十二节、钉值却过了这一格——放宽的那一支要求钉值前 12 位在第十二节，只有钉值等于装置此刻 sha256 才能开跑，所以这样的构造在「能开跑」的前提下应当不存在。

## 第 8 问：新加的自证格、样本与开关，红的地方是不是名字说的事

### 8.1 换上工具四格与十一条开关：成立（一处计数、一处「红在样本前提上」要知道）

四格各红在哪一行（一行一格，读 `scripts/gate/hook-swap.py`）：

| 格 | 红的判法行 | 与名字对不对 |
|---|---|---|
| KILL（三处各一遍） | 「problems.append(f"{what}被留下的 {REMOVAL_SUFFIX} 挡住：{busy[:1]}")」（`scripts/gate/hook-swap.py:4869`） | 对：名字说「被 KILL 之后后来的一趟不被挡、`.taking` 当场摘掉」，红的就是被挡与 `.taking` 还在盘上（`:4873`） |
| TERM（没占着、已占着草稿目录各一趟） | 「problems.append(f"{what}被 TERM 杀掉之后 {REMOVAL_SUFFIX} 还在盘上")」（`scripts/gate/hook-swap.py:4888`） | 对：名字说「放掉 `.taking` 与草稿目录的记录、退出状态仍是被 TERM 杀的」，三样各有一行问题 |
| 回复丢了 | 「problems.append(f"当成了别的趟正在摘：{lost_busy}")」（`scripts/gate/hook-swap.py:4914`） | 对：`os.link` 被换成「做成再报已有」只对 `.taking` 这个目标起作用（`:4901`、`:4907`），所以红的是摘旧记录那一次 link 的读回认自己 |
| 还占着的挡住 | 三臂（这一台进程还在、另一台刚放下、另一台过了期）：缺词行「problems += [f"{what}：输出里没有「{want}」" …]」（`scripts/gate/hook-swap.py:4944`）与记录或 `.taking` 被动 | 对：红的是拒绝说明没点名摘的那一趟，或过了期仍被挡 |

开关与格的对应我读代码逐条推了（没有跑）：
- `taking-claim-unjudged`（`:2900` 的 `mode == "taking-claim-unjudged" or …`）：KILL 格里被 KILL 的子进程留下的 `.taking` 被当成还占着，红在 `:4869`；过了期那一臂红在「退 10 而非换成」。头注释点名的就是这两格。
- `dead-holder-blocks`（「return mode == "dead-holder-blocks"」，`:2739`）：这条开关把「死持有者也挡」打开，四格里计划的记录都是死持有者留下的，于是四格都走不到 `.taking` 那一段：KILL、TERM、回复丢了三格红在「样本前提没造成」（子进程没走到摘旧记录、link 没被调到），还占着那一格红在拒绝说明点名的是记录而不是 `.taking`、过了期那一臂被挡。头注释点名这四格（`:301` 到 `:304`）对得上；红的位置不在 `.taking` 的判法行上，而在前提行上。
- `stale-record-renamed-away`（`:2833` 提前返回，从不建 `.taking`）：四格红的都是「样本前提没造成」（`.taking` 不在、link 没被调到）。这条开关在于撤掉整套 `.taking` 机制，红在前提上是它该有的样子，只是这四格对它不分「机制没有」与「机制坏了」。
- `taking-claim-kept-on-term`（`:2886` 不登记）只红 TERM 格；`taking-reply-lost-refused` 只红回复丢了格；`taking-holder-unnamed` 只红「拒绝说明点名」那一格（两处：这一台起的、另一台刚放下的）。头注释各点一格，读码对得上。
- `scratch-lock-skipped`：红的只有草稿目录那一处（`hold_scratch` 不放记录、`.taking` 留着，`scripts/gate/hook-swap.py:3032` 一带），与头注释 `:293` 的「（草稿目录那一处）」一致。
- 全部点名里出现的 37 个「」名字都在代码里有同字的标签（`sed -n 270,350p scripts/gate/hook-swap.py | grep -o '「[^」]*」'` 取名，逐个对 `sed -n '353,$p'` 的内容：命中 0 个缺）。

小毛病（量的）：KILL 格前面没有 `checked += 1`（命令输出 `grep -n 'checked += 1' scripts/gate/hook-swap.py` 在 4786 与 4877 之间没有别的行；`scripts/gate/hook-swap.py:4786` 是它前一格的计数点，`:4877` 是 TERM 格的），成功句里的「查了 N 种」少一格。不影响红绿。

一件有风险的事（量的）：`scripts/gate/hook-swap.py` 在 `.claude/gate.d/break-switch-replay-files.tsv` 里没有行（`grep -n '^scripts/gate/hook-swap.py' .claude/gate.d/break-switch-replay-files.tsv | wc -l` 输出 0），所以这 18 条开关的「点名的格 = 实际红的格」没有门禁格逐格复核；背景材料里「逐条红在点名的格」是主 agent 一次性的读数。以后谁再改这份脚本的格或开关，头注释可以悄悄对不上。这一点是第 7 问第三处「新格名单按读码推」的实质：读码推出的名单，只有主 agent 那一次复跑撑着（命令在末尾）。

### 8.2 evidence-in-repo 六份样本与五条开关：成立，缺一支、一处措辞

- `evidence-in-repo-exact-only-rerun-red`（E917）、`-unrelated-file-red`（E918）、`-timing-rows-red`（E919、E920）、`-crates-bin-red`（E921、E922）、`-crates-bin-green`（E923、E924 与非 `e<号>_` 的 `helper_tool.rs`）：红在 `newest[1] < source_modified_at`（`.claude/gate.d/doc-experiments.sh:1172`）与 timing 行在盘判（`:1168`、`:1169`），正是名字说的事；绿的那份断言「判了 2 份」与「没判 1 份只改了 // 注释的装置源码」，名字说的事也对。
- 开关对应（读码推）：`replay-exact-by-experiment` 让 E917、E915、E918 绿掉（exact 行点名的产物在盘、timing 的旧，exact 表留住该实验就被「放过」，`:1150` 起那一支）；`timing-rows-ignored` 让 E917、E918、E919、E920 绿掉（比这个实验号最新的一份）；`timing-newest-row-only` 只让 E919、E920 绿掉（取点名产物里最新的）；`crates-binaries-unrecognized` 让 `-crates-bin-*` 两份都不对。与头注释 `:289` 到 `:303` 一致，没有哪一份被漏点名、也没有多点名。
- 缺一支：「只存着 `.gz` 的也认」没有样本。命令与输出见 8.4：`grep -rl 'gz' evidence-in-repo-*/` 退 1、无输出。去掉 `or product_path_by_file_name.get(timing_name + ".gz")`（`.claude/gate.d/doc-experiments.sh:1128` 那半句）没有任何样本变红，也没有开关点它。这一支不是空想：近来的产物压成 `.out.gz` 入库（`.claude/gate.d/doc-experiments.sh:77` 起一段写的是别的格认 `.gz`）。
- 一处措辞：timing 行取的是最旧的那一份（「newest = min(timing_present, key=lambda item: item[1]) …」，`.claude/gate.d/doc-experiments.sh:1136`），而红句仍写「最新的 E{number} 产物 …」（`.claude/gate.d/doc-experiments.sh:1174`）。样本 `-timing-rows-red` 的 expect 把这个措辞钉死：E920 点名的另一份 `…-a-2026-09-06.out` 比装置新，红句点名的却是 `…-b-2026-09-01.out`（那份最旧）并叫它「最新的」。判对了，措辞误导。

### 8.3 钉值格十三份样本与十四条开关：成立，缺一支、两处措辞（同 7.2）

十一份红样本各红在它名字说的那一行：`-inplace-rewrite-red` 红在「lost = …」那一行（`.claude/gate.d/doc-experiments.sh:4755`，基准那一版写着旧钉值的行被改了）；`-internal-whitespace-red`、`-duplicated-line-red` 红在 `:4745` 起那一支（钉值前 12 位基准里就有）；`-added-line-without-pin-red` 红在「新加的行里没有出现钉值前 12 位」；`-pin-in-other-section-red` 红在第十二节没有新加行（新钉值只在第五节新加的行里）；`-pin-trailing-comment-red` 红在「第十二节没有新加的行」且脚本进了对象（`:4519`）；`-registry-mentioned-after-another-red`、`-registry-unlabeled-red` 红在 `registry_of`；`-repin-only-red` 红在 `:4748`；`-archived-history-red` 红在取到历史那一版而第十二节没有钉值前 12 位。绿样本三份：`-repin-only-green`、`-archived-history-green`、`-pin-prefix-green`（只写前 16 位）。开关与样本的对应与头注释 `:398` 到 `:450` 逐条对得上（读码推，没有跑）。

缺的与措辞：
- 缺一支：钉值改成第十二节里本来就写着的值、不新加行、判绿（7.2 的放宽）没有样本；证据见 8.4（只改钉值的样本只有 `-repin-only-*` 两份，绿的那份新加了 `R2-1` 一行）。
- 措辞：只改钉值与已归档这两支的成功句说「都新加了」（`.claude/gate.d/doc-experiments.sh:4776`），样本 `-repin-only-green` 与 `-archived-history-green` 的 expect 把这句钉死；`-archived-history-green` 的 setup 里装置改了、起跑脚本的钉值还是旧值，登记里什么也没新加，成功句却说「登记第十二节都新加了修订」。

### 8.4 样本缺口的命令与原样输出（在 `.claude/gate.d/fixtures/doc-experiments.sh/` 下跑，只读）

```
$ grep -rl 'gz' evidence-in-repo-*/ ; echo "rc=$?"
rc=1
$ grep -l 'repin' pinned-apparatus-revision-*/setup.sh ; echo "rc=$?"
pinned-apparatus-revision-repin-only-green/setup.sh
rc=0
$ grep -n '^registry 934' pinned-apparatus-revision-repin-only-green/setup.sh
35:registry 934 "R2-0 登记时的装置 sha256 $old"$'\n\n'"R2-1 重钉：装置在早先的提交里改过，新的 sha256 $new"
```

第一条：evidence-in-repo 六份样本里一份都没有 `.gz`；第二、三条：只改钉值（装置没变）的样本只有 `-repin-only-red`（不含字样 `repin` 的 setup，靠目录名）与 `-repin-only-green`，绿的那份第 35 行把第十二节写成两行（旧行加新行 `R2-1`）。

### 8.5 完工清理闸三格与开关：成立一格，两格要知道红在哪里

- `nested-lower-handover-passes`（`.claude/hooks/completion-cleanup-guard.sh:2704`）：成立。红在 `nested_outcome.get("action") == "pass"` 不成立：开关 `nested-declaration-ignored`（`("nested-declaration-ignored", "nested-lower-handover-passes"),`，`.claude/hooks/completion-cleanup-guard.sh:481`）与 `deliverables-ignored`（点名在 `:190`）下都判不出放行，与名字一致；下层文件里放了一行写坏的行，同一格也证「不拦上层」。
- `nested-undeclared-copy-blocks`：红的地方是「下层没声明的副本，上层照拦」，成立，但只在两条通用开关下红：`repository-copy-unrecognized`（点名 `:167`）与 `prompt-scratch-ignored`（点名 `:219`），这两条红是因为副本或草稿目录根本没认出来、什么也没列，不是因为嵌套读法过宽。没有一条开关把「读了下层声明就把整个下层目录当已声明」这种过宽读法造出来（`BREAK_TABLE` 里嵌套相关的只有 `:481` 那一条）；这一格本身能抓到那种过宽（下层文件只写了一行坏行，副本没声明），是读码推的，没有被开关证明过红。
- `nested-lower-handover-main-listed`（`.claude/hooks/completion-cleanup-guard.sh:2700`）：名字说「主 agent 结束一轮时不读下层的声明、交接的副本照列」；只在 `repository-copy-unrecognized` 下点名（`✗ 自证 nested-lower-handover-main-listed：` 在 `.claude/hooks/completion-cleanup-guard.sh:166`），同样红在「副本没认出」。真正该由它守的「主 agent 也去读下层声明」的写法（`nested_role` 对主 agent 也给，`.claude/hooks/completion-cleanup-guard.sh:1962` 的 `role != "subagent"` 一半）没有开关造出来，所以这一格有没有守住，要靠这条判法被人手改时它自己红，开关不替它作证。
- 77 条开关、106 格都与文件头对得上（读码量的）：`grep -c 'COMPLETION_CLEANUP_BREAK='` 命中 77，`BREAK_TABLE` 77 行，`check("` 命中 106。

建议（不是缺陷）：补两条开关，`nested-declaration-read-by-main`（主 agent 也读下层声明，点名 `nested-lower-handover-main-listed`）与 `nested-declaration-exempts-lower-directory`（读到声明就把整个下层目录当已声明，点名 `nested-undeclared-copy-blocks`）。

### 8.6 删旧闸与 `stage-owners.tsv`：成立

- 删旧闸：格 `cell_existing_output_name_with_brackets`（`scripts/experiment-apparatus/e07-vm-bench-run.sh:268` 一带）造的是「提交了 `e07-r2.out`、没进 git 的是 `e07-r[2].out`」，`committed == 0` 守住前提，红在「退 2、原文件原样留着、不起构建」那一条。开关 `output-name-as-pathspec`（头注释 `:74`、`:75`）把 `:417` 一行换成不加 `:(literal)` 的名字，这一格红（点名唯一），开关 `existing-output-deleted-unchecked`（`:70` 到 `:73`）下三格红（补进的点名对），两条都对得上名字。四份入口形同，每份各一个这格。
- `stage-owners.tsv:117`、`:130` 的「为什么」与门禁新判法一字不矛盾：第 117 行写「复跑表里有 timing 行的实验按每一条 timing 行点名的产物逐份比」（`.claude/gate.d/stage-owners.tsv:117`，对应 `.claude/gate.d/doc-experiments.sh:1136`），第 130 行写「只改钉值的第十二节里写着新钉值前 12 位即可」（`.claude/gate.d/stage-owners.tsv:130`，对应 `.claude/gate.d/doc-experiments.sh:4748`）；与 7.2 的第三处对照，门禁文件里 `gate-cell:` 行（`:32`）没跟着改，这两行倒是跟着改了。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 6.1 换上工具 `.taking` | 成立 | 写身份、同一判法、TERM 与 HUP 放掉、link 回复丢了当放成、拒绝说明都对上，另多一格反方向；KILL 格没记数 |
| 6.2 evidence-in-repo | 成立 | 每一条 timing 行点名的产物比修改时刻，exact 与不相干文件不算，`crates/*/src/bin/e<号>_` 一并认 |
| 6.3 删旧闸 | 成立 | 四份入口各一处 `:(literal)`，两条 git 共用 |
| 6.4 钉值格 | 成立 | 触发、认法、行尾注释、登记取法、历史取法都对；放宽一处见 7.2 |
| 6.5 完工清理闸 | 成立 | 下层声明读到、只给子 agent 读；取的种类比「交接」宽但不放过本不该放的 |
| 6.6 `stage-owners.tsv:143` | 成立 | 写成封存的说法，与 `CLAUDE.md:21` 对得上 |
| 7.1 取「每一条都要」 | 成立（偏得对） | 「那一份」认不出，取其一等于被打中的那一支；量到 17 条 timing 行里 15 条产物不在盘，是代价不是偏差 |
| 7.2 钉值格放宽 | 成立（偏得对，三处要补） | 装置没变时钉值只能等于装置此刻的 sha256，登记里写着即可；缺样本、成功句与 `gate-cell:` 行没跟上 |
| 7.3 新格名单按读码推 | 成立（有条件） | 37 个名字都能在代码里对上；点名与实际红的格要靠主 agent 复跑，hook-swap 不在复跑登记里，之后会漂 |
| 8.1 换上工具四格与开关 | 成立 | 四格红在名字说的判法行上；`dead-holder-blocks`、`stale-record-renamed-away` 红在样本前提上 |
| 8.2 evidence-in-repo 样本 | 成立（缺一支） | `.gz` 分支无样本无开关；红句「最新的」名的是最旧的一份 |
| 8.3 钉值样本 | 成立（缺一支） | 十三份各红在名字说的行；「不新加行也绿」无样本；两处成功句说「都新加了」 |
| 8.5 完工清理闸三格 | 部分成立 | `-passes` 成立；另两格只被通用开关证红，没有专门的嵌套开关 |
| 8.6 删旧闸与 stage-owners | 成立 | 样本、开关、点名对得上 |

## 要主 agent 跑的命令

以下都没有跑，命令在仓根起（`--cwd` 缺省即仓根）；判结果看 `TASK_SERVER_RESULT` 的 `type=` 与 `exit=`。

1. 换上工具整份自证，应当 `exit=0`、日志里有「✓ hook-swap 自检通过（查了 N 种）」，N 比实际判的少一格（8.1 小毛病）：
   `task-server/task submit -- python3 scripts/gate/hook-swap.py --selftest`
2. 十一条与 `.taking` 相关的开关逐条各一趟，每一趟应当 `exit=1`，日志里 `✗ hook-swap 自检：` 起头的行的格名（冒号前那一段，如「草稿目录的占用：…」）与头注释 `scripts/gate/hook-swap.py:288` 到 `:350` 里该开关名下面点名的格逐个相等（8.1 逐条读码推的，等这一步复核）：
   `task-server/task submit -- env HOOK_SWAP_BREAK=taking-claim-unjudged python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=taking-claim-kept-on-term python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=taking-reply-lost-refused python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=taking-holder-unnamed python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=scratch-lock-skipped python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=dead-holder-blocks python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=scratch-lock-kept-on-exit python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=scratch-lock-term-unhandled python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=elsewhere-holder-ignored python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=elsewhere-holder-never-expires python3 scripts/gate/hook-swap.py --selftest`
   `task-server/task submit -- env HOOK_SWAP_BREAK=stale-record-renamed-away python3 scripts/gate/hook-swap.py --selftest`
3. doc-experiments 样本档（19 份新样本与开关对应在内），应当判得都对；被项目代码闸拒了照它给的整条写法改：
   `task-server/task submit -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item doc-experiments`
4. 完工清理闸自证，应当「✓ 各格都对（判了 106 格…）」：
   `task-server/task submit -- bash .claude/hooks/completion-cleanup-guard.sh --selftest`

复跑脚本：没有写模型，没有 `rerun.sh`；上面这些是读码推出来的对应的复核命令。

## 没做什么

- 没有跑任何测试、自证、样本档、弄坏开关（子 agent 不交任务）：8.1、8.2、8.3 里「开关与格的对应」都是读码推的，要等上面 2、3 条结局。
- 没做攻方腿的第 1 到 5 问（两趟同摘一份死 `.taking`、`release_then_die` 的次序、时钟、`pinned` 回钉旧值的具体写法、`git` 其它路径解法、下层 `.deliverables` 写坏或换链接）。其中 7.2 里「钉回旧值」「装置回滚过不了」与 7.1 里的 15 条缺产物，是读这几处时顺带见到的线索，不替攻方腿判。
- 不替主 agent 采纳或出判决；没往任务服务交任务；没有写模型，没有 `<模型目录>`；没有建仓副本，草稿目录里只有 `git diff HEAD` 存下的几份差异与取名核对的中间文件（交回前删）。
