# ledger-open-fixes-governance-r3：攻方腿报告（第 1 到 5 问，2026-11-19）

被判的文件读的是主工作区此刻那一版；八份被判文件的 sha256 与开工快照 `research/prompts/ledger-open-fixes-governance-r3-snapshot/judged-sha256.txt` 逐份相同（`sha256sum` 现核）。没读禁读清单里的五份报告。

## 各格判定一览

| 问 | 格 | 判定 | 依据 |
|---|---|---|---|
| 1 | 两趟同时摘同一份已死的 `.taking` | 成立（交错存在），后果被文件头说轻了：按仓的那一份上两趟都当自己占到，文件头列的兜底里头一样就是它自己 | 读码；探针 `probe-taking.py` 臂 `double-dead-taking-*`，推的、待主 agent 跑 |
| 1 | 摘的那一趟停过 300 秒（另一台） | 推不出（打中）：`.taking` 没有心跳，摘的当中停住超过 300 秒就被当成死了，恢复后删掉别人刚放的记录；文件头「留下的缝」没列这一种 | 读码；探针臂 `remover-stall-elsewhere`，推的 |
| 1 | `release_then_die` 的放法次序 | 成立：被第二个信号打断时留下的不超过被 KILL 留下的；另有一处先放草稿目录、把 TERM 处理还原成默认，之后才放按仓的那一份 | 读码，推的 |
| 1 | 360 秒与记录的过期判法 | 不成立（正文写错）：代码里是同一个常数 300 秒、同一个钟、同一个判法；360 是自证样本拨的年龄；差别是记录有心跳、`.taking` 没有 | 读码；探针臂 `stale-threshold` |
| 2 | 「每一条 timing 行都要」的出路 | 成立，但只有「全部 timing 行都重跑」一条路；只重跑改到的那一段没有绿的路：14 个有 timing 行的实验里 13 个的 timing 产物今天都不在盘（已归档） | 现查 `scripts/experiment-apparatus/replay.sh` 与 `research/results/` |
| 2 | 复跑表行键带后缀的 timing 行 | 推不出（打中）：`E926R2` 这种行键的 timing 行不归到 E926，回到第二轮打中的「放一份不相干的 e<号> 文件就绿」 | 读码；探针 `probe-evidence.sh` 臂 `suffix-timing-row`，推的 |
| 2 | 只改变异表 | 成立（照判法），代价：改一张变异表也要把全部 timing 行重跑 | 读码；探针臂 `mutation-table-only` |
| 2 | 目录形装置、`e<号>_` 起头而不是装置的二进制 | 成立：都按那个号的装置判；`#[path]` 引进来的 `tests/common*` 共用模块不判，门禁的「管不到」没写 | 读码，现查 |
| 3 | 钉回已被作废的旧值 | 推不出（打中）：登记已归档的，装置回退成登记里写过的作废那一版、钉值钉回它，格判绿、开跑的 sha 核对也过 | 读码；探针 `probe-pin.sh` 臂 `archived-revert-to-superseded`，推的 |
| 3 | 登记路径写错与已归档分不开 | 推不出（打中）：登记被改名（`git mv`）而起跑脚本没跟着改，格拿改名之前冻住的那一版判，装置改了、钉值不动判绿 | 读码；探针臂 `renamed-registry-*`，推的 |
| 3 | 两条判据之间的写法 | 推不出（打中）：把第十二节里不带基准钉值的旧行原地改成新钉值，三条都过，没新加一行 | 读码；探针臂 `older-line-rewritten`，推的 |
| 3 | 合规的回退 | 打中（误红、没有出路）：登记在树里时，装置回退成登记里早先的一版，新加一行写明回退也判红 | 读码；探针臂 `intree-revert-with-new-line`，推的 |
| 4 | `:(literal)` 之外的路径解法 | 推不出（打中）：输出文件带 assume-unchanged 或 skip-worktree 标记、改过没提交的，两条 git 都说「在 git 里、没改过」，删掉 | 读码；探针 `probe-delete.sh` 臂 `*-modified`，推的 |
| 4 | 大小写 | 环境带 `GIT_ICASE_PATHSPECS=1` 时 `:(literal)` 照样不分大小写（推的，平常没人设）；`core.ignorecase` 不进 pathspec 的比对（推的） | 探针臂 `icase-environment`、`ignorecase-config` |
| 4 | 第三处按 pathspec 解输出路径 | 不成立：四份入口各只有那两条 git 碰输出路径 | `grep` 现查 |
| 5 | 下层是别的会话、别的 agent 的目录 | 成立：上层照认，不看是谁写的；后果有界：主 agent 收工时不读下层声明、照列 | 读码，推的 |
| 5 | 下层 `.deliverables` 写坏、被删、换成符号链接 | 成立：那几行或整份不算，副本照列、上层被拦；拦的说明不点名下层写坏的行与符号链接 | 读码，推的 |

撤回（第二轮攻方腿给、判决「三、改法」采了的，按判决里的转述）：删旧闸「两条 git 加 `:(literal)`」那条改法修的只是通配，「在 git 里、和 HEAD 没差」这个判据本身在索引标记下不成立，撤回它当完整修法，换成按内容比（第四节）；钉值格「登记不在树里时从 git 历史里取它被删前的那一版判」那条，判不出新加的修订、还把改名当归档，撤回（第三节）；evidence-in-repo「同一个实验有 timing 行时按 timing 行点名的产物比」没写复跑表行键怎么归到实验号，有后缀的行键漏掉，那一句写宽了（第二节）。

## 探针（都在草稿目录，读被判文件、不改仓）

草稿目录 `/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/`：

| 文件 | sha256 | 行数 |
|---|---|---|
| `probe-taking.py` | `42945ecef03db75ffa43ffad49f0aca301b6b83a816b081c6e1b9b39632d8196` | 232 |
| `probe-evidence.sh` | `0cc8a30c9dd003bfc9036cc29d6dae99cda0241ec0916de610a897d3c8000bee` | 72 |
| `probe-pin.sh` | `e9d48a58d9b550f4582ae757123e45475b3ddbde8972cee4ffacd3080d4bdfa8` | 107 |
| `probe-delete.sh` | `9f95e2c92269fbae17f10931e4fd0a6064bf858ecba71980fbcbae91685303db` | 61 |

四份都收 `<仓根> <工作目录>` 两个参数，样本仓与记录都建在工作目录下；`probe-taking.py` 载入 `scripts/gate/hook-swap.py` 当模块（同它自证里 `SELFTEST_CLAIM_CHILD` 的载法），两趟用同一进程的两条线程演，靠包住 `os.unlink`、`os.link` 的闸门固定交错次序，等待最多 30 秒；另外三份照 doc-experiments 样本的写法建样本仓，只跑 `--check <一格>`，或把入口拷进样本树、假 cargo 记一笔就退 101。估时（推的）：每份一分钟以内，内存几十 MiB。命令在末尾「要主 agent 跑的命令」。没写模型目录：派发提示写的是探针放草稿目录；判决要引的数由主 agent 复跑。

## 一、换上工具 `.taking`（第 1 问）

### 两趟同时摘同一份已死的 `.taking`：交错存在，后果在按仓的那一份上被说轻了

摘已死的 `.taking` 是先读后删，中间不原子：`scripts/gate/hook-swap.py:2903`：「if read_scratch_record(taking) == text:」，下一行才 `os.unlink(taking)`。交错（A、B 两趟，旧记录 R0 与已死的 `.taking` T0 都在）：

1. A、B 在 `claim_record_removal` 里都读到 T0、都判写它的那一趟不再占着、都过了 2903 那一句的比较。
2. A 删掉 T0，下一圈 link 放上自己的 CA，当放成。
3. B 删掉此刻在那里的 CA（它比的是 T0，删的是 CA），下一圈 link 放上 CB，也当放成。
4. A、B 在 `remove_stale_record` 里都读到 R0（`scripts/gate/hook-swap.py:2862`：「current = read_scratch_record(path)」），都是判过的那一份。
5. A 删 R0，放 `.taking` 时读到 CB 不是自己的、不删，回到 `acquire_record` link 放上自己的记录 RA，交 None（占到）。
6. B 删掉 RA（`scripts/gate/hook-swap.py:2868`：「os.unlink(path)」，它比的是 R0），放掉 CB，link 放上 RB，交 None（也占到）。

之后 A 的心跳读到记录不是自己的就不刷，也不报（`scripts/gate/hook-swap.py:2963`：「持有期间每 interval_seconds 秒刷新一次占用记录的修改时间（另一台靠它看出持有者还在）；记录已被换成别人的就不刷。」），A 照常往下跑。

文件头自己列了这一种：`scripts/gate/hook-swap.py:96`：「同一份不再挡的 .taking 被两趟同时判成不再挡、先后各删一次、各放自己的：后删的那一趟可能删掉先删那一趟刚放的 .taking，两趟同时当自己在摘旧记录；」，后果写成 `scripts/gate/hook-swap.py:97`：「摘的那一步先读记录、只在还是判过的那一份时才删，撞上的后果同上，只是回到没有这一道占用的样子。」。「同上」指 `scripts/gate/hook-swap.py:95`：「真换另有按仓的占用、基线核对与结局文件拦着」。可按仓的那一份用的是同一段代码（`scripts/gate/hook-swap.py:102`：「记录的放法、持有者还占不占着、摘旧记录、放锁、心跳都同」）：撞在按仓的记录上时，兜底的头一样就是撞穿的这一样，剩下的是两趟真换重叠，正是 `scripts/gate/hook-swap.py:100`：「后写开始行的整份盖掉先写的」那件事。两份规格碰的文件不重叠时，基线核对各自都过（推的）。所以「只是回到没有这一道占用的样子」对草稿目录那一份说得对，对按仓那一份说轻了。

前提（推的）：先要有一份已死的 `.taking`（摘的那一趟在几毫秒的摘的当中被 KILL，或被 TERM 之后放的当中又被 KILL），再要两趟真换在同几毫秒里同时碰上它。概率很低；第二轮修补把「已死的 `.taking` 永远挡」换成了这一种「两趟各占一次」，方向是对的，代价是这条缝。

探针臂 `double-dead-taking-scratch`、`double-dead-taking-repository` 推的读数：`believers=2 A_record_on_disk=False B_record_on_disk=True`；对照臂 `double-dead-taking-sequential`（同一份已死的 `.taking`，先后跑）推的 `believers=1`。

四句：分得出「一趟占到」与「两趟都占到」；被判的代码当时看得到 `.taking` 的内容与修改时间，看不到别的趟已经比过了；满足的是正文第 1 问「有没有一种交错让两趟都以为自己摘成、都去删记录」那一分句；第二轮判决第 1 条的改法（`.taking` 照记录那一套判死活）正是让这一步发生的那一处，它对这一格不起作用。

我的改法（只在读码上推过、被攻过零轮）：真换在写结局文件开始行之前、每换上一份文件之前，各重读一次按仓的记录，不是自己写的那一份就停、退 10、不写；心跳读到不是自己的那一刻记下来，同一个检查也查这个标记。它不消掉交错，把「两趟都写」变成「后发现的那一趟停」。

### 摘的那一趟停住超过 300 秒（另一台）：文件头没列的一条缝

`.taking` 没有心跳（只有占用记录有，`scripts/gate/hook-swap.py:2670`：「SCRATCH_LOCK_HEARTBEAT_SECONDS = 15」只给 `keep_scratch_lock_fresh` 用），它的年龄从放上那一刻起算（`scripts/gate/hook-swap.py:2896`：「age_seconds = max(0.0, time.time() - os.stat(taking).st_mtime)」），另一台起的照 `scripts/gate/hook-swap.py:2744`：「return age_seconds < SCRATCH_LOCK_STALE_SECONDS」判。摘的那一趟 R 在另一台，放好 `.taking`、读过记录是 R0 之后、删 R0 之前停住超过 300 秒（共享盘那一端暂时连不上、进程被 SIGSTOP，推的），这一台的 S 判 `.taking` 过期、删掉它、摘掉 R0、放上 RS；R 恢复之后删掉的是 RS，再放上自己的 RR，两趟都占到。文件头「留下的缝」只列了持有者那一边停住又恢复（`scripts/gate/hook-swap.py:94`），没列摘的那一边。

探针臂 `remover-stall-elsewhere`（把 R 写的身份换成另一台的机器标识，`.taking` 的修改时间拨回 400 秒）推的 `believers=2`；对照臂 `remover-stall-same-machine` 推的 `believers=1`、`s_refused=True`。后果同上一格，前提比上一格还少见（要一次 300 秒以上的停顿正好落在两条相邻的系统调用之间）。

### `release_then_die` 的次序：留下的不超过被 KILL 留下的

处理的次序是先放 `.taking`（`scripts/gate/hook-swap.py:2991`：「for taking, content in list(HELD_REMOVAL_CLAIMS):」），再放占着的记录（`scripts/gate/hook-swap.py:2993`：「for held in list(HELD_SCRATCH_LOCKS):」），最后还原默认、重发。两者之间再来一个 TERM：处理还没还原，Python 再进一次 `release_then_die`，把剩下的放完再死，什么都不留（推的）。真正有窗口的一处在后半：`HELD_SCRATCH_LOCKS` 里先是草稿目录那一份（`scripts/gate/hook-swap.py:3083` 先占）、后是按仓那一份（`scripts/gate/hook-swap.py:3098`），放草稿目录那一份时 `scripts/gate/hook-swap.py:2977`：「restore_signal_handlers(held.previous_handlers)」把 TERM 还原成默认，按仓那一份还没放；这几微秒里再来一个 TERM 就照默认死，留下按仓的记录。留下的东西与被 KILL 一样：这一台起的立刻不挡，另一台起的挡 300 秒，结局文件照「没换完」那一套拦。正常返回的 `finally` 是先放按仓那一份再放草稿目录那一份，没有这个窗口。后果轻，不建议为它改。

### 360 秒：正文写错了，代码里是同一个数、同一个钟

正文 `research/prompts/_ledger-open-fixes-governance-r3-body.md:15` 写「过期 360 秒」。代码只有一个常数：`scripts/gate/hook-swap.py:2671`：「SCRATCH_LOCK_STALE_SECONDS = 300」；记录与 `.taking` 走同一个判法 `scratch_holder_occupies`（2744），年龄都是这一台的 `time.time()` 减文件的修改时间（记录 `scripts/gate/hook-swap.py:2810`：「age_seconds = max(0.0, time.time() - os.stat(path).st_mtime)」，`.taking` 2896）。360 是自证样本拨的年龄：`scripts/gate/hook-swap.py:4931`：「("另一台起的、过了期", {"机器": "f" * 16}, SCRATCH_LOCK_STALE_SECONDS + 60, None))):」。两者真正的差别是心跳：记录持有期间每 15 秒刷一次，`.taking` 从放上起算、不刷——正是上一格那条缝的来处。探针臂 `stale-threshold` 打出常数与 330 秒、290 秒两个年龄各判成什么，推的 `SCRATCH_LOCK_STALE_SECONDS=300 elsewhere_330s_occupies=False elsewhere_290s_occupies=True`。

## 二、evidence-in-repo「每一条 timing 行都要」（第 2 问）

### 出路：只有「全部 timing 行都重跑」一条路通

判法：有 timing 行的实验，比的是 timing 行点名的产物里最旧的那一份，有一份不在盘就当没有（`.claude/gate.d/doc-experiments.sh:1136`：「newest = min(timing_present, key=lambda item: item[1]) if timing_present and not timing_missing else None」）；出路写的是 `.claude/gate.d/doc-experiments.sh:1248`：「复跑表里有 timing 行的实验，每一条 timing 行点名的产物都要重跑存回」。照出路把每一条 timing 行都重跑、产物存回原名，判法里每一份都在盘、都不比装置旧，判绿——这条路走得通（读码，推的）。

只重跑改到的那一段走不通。现查今天的复跑表与 `research/results/`（命令：`grep -E '^E[0-9]+[A-Z0-9]*\|[^|]*\|[^|]*\|[^|]*\|timing$' scripts/experiment-apparatus/replay.sh` 得 17 行，逐行 `find research/results -name <产物名>` 与 `<产物名>.gz`）：17 条 timing 行分属 14 个实验，只有 E174 的两份在盘；E17、E20、E21、E44、E58、E107、E128、E140、E144、E161、E162（三条）、E181、E184 的 timing 产物都不在盘（归档进了历史，例如 `e162-crash-verdict-block-store-s1-all-arms.out` 最后出现在 `git log --all` 的 `a24379563` 那一次归档）。这 13 个实验的装置或变异表这一轮有任何不只是 `//` 注释的改动，格就红，直到这个实验的每一条 timing 行都重跑过；重跑存回的产物下一次提交又被归档删掉，下一次改装置再全部重跑。E162 的装置有四份（`e162_crash_verdict_block_store.rs`、`e162_verdict_store_network.rs`、`e162_verdict_store_power_cut.rs`、`e162_verdict_store_sender.rs`），变异表四张，改其中任何一份都要重跑 s1、s2、s4 三趟 all-arms（`scripts/experiment-apparatus/replay.sh:2819`：「E162|@driver_e162_s4_all_arms||e162-verdict-store-power-cut-s4-all-arms.out|timing」是其中一条）；E44、E58、E140 的 timing 行要真盘（参数 `$REPLAY_DEV45`、`$REPLAY_DEV58`、`$REPLAY_DEV140`）。

所以第 2 问前半答「成立」：出路给得出、照做就绿；只是那条路的代价是整个实验的全部 timing 行，与「只重跑改到的那一段」之间没有中间的路。判法的理由是 `.claude/gate.d/doc-experiments.sh:908`：「装置源码与复跑表的行之间没有机读的对应」——直接点名二进制的行（`E44|e44_jsn_width|…`）第二列就是对应；`@driver_` 起头的行要读驱动函数才知道起了哪几个二进制。这是判决写「那一份」、工具员取「每一条」的根子，归正推腿第 7 问；我这里只报代价。

变异表也走这一路（`.claude/gate.d/doc-experiments.sh:1012`：「(re.compile(r"^scripts/mutation/tables/e([0-9]+)_[^/]*\.tsv$"), "变异表"),」与装置源码共用后面的比法）：只改一张变异表（碰不到产物），也要把全部 timing 行重跑。探针臂 `mutation-table-only`（E927 两条 timing 行、exact 那一条刚重跑、只改变异表）推的 `exit=1`、红句点名 timing 产物还是旧的；臂 `archived-timing-product`（E928 timing 产物不在盘、exact 刚重跑、另放新笔记）推的 `exit=1`、红句写 timing 行点名的产物不在盘。

### 打中：行键带后缀的 timing 行不归到实验号

timing 行按行键去掉头一个字母归到实验号：`.claude/gate.d/doc-experiments.sh:1079`：「replay_timing_names_by_number.setdefault(replay_row.group(1)[1:], []).append(replay_row.group(4))」，行键的写法允许数字后面带字母数字后缀（`scripts/gate/admission.py:795` 的 `REPLAY_ROW_FORM`，`^(E[0-9]+[A-Z0-9]*)`），装置的号只取数字（`e([0-9]+)_`）。仓里重跑的行就是带后缀写的：`scripts/experiment-apparatus/replay.sh:2765` 的 `E16R5|…`、`scripts/experiment-apparatus/replay.sh:2798` 的 `E155R2|…`（今天这些都是 exact 行）。一条 `E926R2|…|timing` 的行归到「926R2」，查 E926 的装置时 `.claude/gate.d/doc-experiments.sh:1124` 取到空表，退回「这个实验号在盘最新的那一份」——第二轮打中、这一轮要修的那一格原样回来：放一份 `e926-notes-…` 就绿。

探针 `probe-evidence.sh` 臂 `suffix-timing-row`（与样本 `evidence-in-repo-unrelated-file-red` 同一个样子，只把行键写成 `E926R2`）推的 `exit=0`、成功句「判了 1 份」；对照臂 `suffix-timing-control`（行键 `E926`）推的 `exit=1`。

四句：分得出（只差行键后缀，结论翻面）；判法看得到行键全文，只是切掉了后缀以外的头一个字母；满足第 2 问「修没修到位」与判决第 2 条「同一个实验有 timing 行时，按 timing 行点名的那一份产物比」；判决第 2 条的改法没说行键怎么归到实验号，对这一格不起作用。今天的复跑表里没有带后缀的 timing 行，所以今天的真仓不受影响；E<号>R<次> 是重跑登记的现行写法，下一次 timing 重跑登记进来就撞上。

我的改法（被攻过零轮）：行键归实验号取 `E` 后面的那一串数字（`re.match(r"E([0-9]+)", 行键)`），timing 与 exact 两处同改；exact 那一处随之让 `E16R5` 这类行也替 E16 的装置作保，是放宽，要在样本里各放一份。

### 目录形装置与不是装置的 `e<号>_` 二进制

- 目录形：`.claude/gate.d/doc-experiments.sh:1005`：「re.compile(r"^crates/[^/]+/src/bin/e([0-9]+)_[^/]+/.+\.rs$"),」认目录下任意深的 `.rs`，按目录名的号判。真仓里唯一的一处是 `crates/singlefs-implementation-experiments/src/bin/e184_regular_layer_cost_counts_write_dump/mod.rs`，它不是 cargo 的目录形二进制（没有 `main.rs`），是两个 E184 二进制经 `#[path]` 引进去的共用模块（`crates/singlefs-verification/src/bin/e184_regular_layer_cost_counts.rs:52`：「#[path = "e184_regular_layer_cost_counts_write_dump/mod.rs"]」）；按 E184 判，E184 的 timing 产物不在盘，改它就要重跑 segment one。判得对。
- 不是实验装置本身的 `e<号>_` 二进制（`e142_new_pool_file_creation_write_dump.rs` 这类只读导出）：按那个号的装置判，要一份不比它旧的 e142 产物（E142 没有 timing 行，走「最新的一份」）。判得严，不漏。
- 漏的一类：同样经 `#[path]` 编进实验二进制、但住在 `crates/*/tests/` 下的共用模块。`crates/singlefs-verification/src/bin/e161_crash_state_dedup_and_time_split.rs:28`：「#[path = "../../tests/common/mod.rs"]」，E170、E171、E184 也引它；`crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs:240`：「#[path = "../../tests/common_pairing_input_through_the_library/mod.rs"]」。改这几份等于改了实验二进制，格不判；格的「管不到」一节（`.claude/gate.d/doc-experiments.sh:920` 起）没写这一类。同一种 `#[path]` 共用模块，住在 `src/bin/e<号>_…/` 下的判、住在 `tests/` 下的不判。归不归这一轮：判决第 2 条只说「`crates/*/src/bin/` 下的装置一并认」，这一类在它外面，记一笔交主 agent 定要不要写进「管不到」。

## 三、pinned-apparatus-revision 的放宽与登记认法（第 3 问）

### 钉回已被作废的旧值：登记已归档时整条放行

已归档的登记只核「那一版第十二节里写没写着钉值前 12 位」：`.claude/gate.d/doc-experiments.sh:4489`：「在那一版的第十二节里核钉值前 12 位有没有出现（那一版没法再新加行，不核新旧）」，代码 `.claude/gate.d/doc-experiments.sh:4725`：「if any(has_pin(line) for line in section):」之后直接记成已修订。第十二节常写着几个装置 sha256，其中有的是被后一行作废的。真例：E185 第 2 次重跑登记（今天已归档，`git show a24379563928^:research/prompts/e185-r2-prereg.md` 第 427 行起）第十二节同时写着交回时那一份 `d6e1cb04…`、「以这一份为准」的 `0bd22743…`、挪文档注释之前那一份 `19d44942…`，起跑脚本 `scripts/experiment-apparatus/e185-r2-segment-run.sh` 钉的是 `0bd22743…`。把装置换回 `d6e1cb04…` 那一版、r2 的钉值改成它：格对 r2 判「已修订」；开跑时只比装置与钉值（`scripts/experiment-apparatus/e185-r4-segment-run.sh:485`：「if [[ "$apparatus_sha256" != "$registered_apparatus_sha256" ]]」，r2 那份同一个判法），也过——用登记自己说作废了的装置跑，格与停机都不拦。

在真仓里这一步会被 r4 那份拦：三份 E185 起跑脚本点的是同一份装置源码，r4 的登记在树里，装置一变 r4 就要新加一行写着新钉值。所以真仓今天要连 r4 一起改才放得过（推的）；格自己的判法对「登记已归档」那一支没有任何新旧约束。

探针 `probe-pin.sh` 臂 `archived-revert-to-superseded`（登记写着作废的 sha1 与为准的 sha2，归档之后装置回退成 v1、钉值 sha1）推的 `exit=0`，成功句把它算进「装置源码这次改动里变了的 1 份，登记第十二节都新加了修订」。成功句本身也不对：`.claude/gate.d/doc-experiments.sh:4775` 把从历史里取来核过的几份与真新加了行的几份算在一起，都说「新加了修订」。

在树里的登记只改钉值时（第二轮偏离判决字面的那一处）同样放行作废值：`.claude/gate.d/doc-experiments.sh:4748`：「elif not any(has_pin(line) for line in section):」只问第十二节里写没写着。这一支开跑时会被 sha 核对挡（装置没变、钉值成了旧值，对不上），所以只是格绿、跑不起来；但成功句 `.claude/gate.d/doc-experiments.sh:4776`：「f"装置没变而钉值改了的 {len(repinned)} 份，登记第十二节都新加了写着新钉值的行{named(repinned)}；"」在这一支上说的是没发生的事（什么都没新加）。探针臂 `intree-repin-superseded` 推的 `exit=0`、成功句照这样写。正文说真仓上三份 E185 起跑脚本都落在这一句里，r2、r3 那两份的登记已归档、没法新加行，句子说「都新加了」，与事实不符。

### 登记路径与已归档分不开：今天就会撞上的是改名

格先取基准那一版，再取历史里「被删之前」的那一版（`.claude/gate.d/doc-experiments.sh:4635` 的 `git log -1 --diff-filter=D … HEAD -- path`）。登记被 `git mv` 改名而起跑脚本里「跑前登记 <路径>」没跟着改：只给旧路径当 pathspec 时 git 把改名记成旧路径被删（推的，探针臂打出 `deletion_commit_found`），格拿改名之前那一版冻住的第十二节判。

- 装置改了、钉值不动（`renamed-registry-pin-kept`）：冻住那一版写着钉值 → 判「已修订」、`exit=0`（推的）；新名字那份登记一行没加。开跑时 sha 对不上会停，害处是格的结论错、成功句说「新加了修订」。
- 装置改了、钉值改新、新名字的登记照规矩新加了一行（`renamed-registry-live-revised`）：冻住那一版没有新钉值 → 红，出路写 `.claude/gate.d/doc-experiments.sh:4767`：「已有的行不改写、不重抄；登记已归档（不在树里）的，新钉值要写进新的重跑登记」——照它去写新登记是错路，该改的是起跑脚本里那条路径（推的 `exit=1`）。

格自己写的「管不到」是 `.claude/gate.d/doc-experiments.sh:4491`：「注释里点名的登记路径写错、历史里也从没有过这份文件的，与已归档的分不开，都归「不判」」。改名不在这一句里：历史里有过、格也判了，判的是一份死了的版本。「今天就会被当成不判的真实写错」：今天三份起跑脚本点的路径都找得到（r4 在树里，r2、r3 在历史里），我给不出今天就在仓里的一处；最可能来的是改名（`.claude/rules/path-moves.md` 管的那类挪动）与「先写起跑脚本、登记还没建」。

### 两条判据之间：原地改写不带基准钉值的旧行，三条都过

「一行不能少」只护着写着基准钉值的那几行：`.claude/gate.d/doc-experiments.sh:4755`：「lost = nonblank(line for line in previous_section if base_pin[:12] in line.lower()) - nonblank(section)」。「新加的行」按多重集合相减（`.claude/gate.d/doc-experiments.sh:4736`：「added = nonblank(section) - nonblank(previous_section)」），改写过的旧行算新加的。所以把第十二节里一行更早的修订（写的是基准之前的钉值）或一行说明原地改成写着新钉值：一（改写的那一行算新加、带新钉值）、二（新钉值基准里没有）、三（写着基准钉值的行都在）都过，行数没变、早先那一行的记录没了。红句自己写的规矩是 `.claude/gate.d/doc-experiments.sh:4757`：「修订只许新加行，不改写已有的」，判法只罩住了其中一类行。真例的形状：E185 r4 登记第十二节 `R4-1` 那一段，写基准钉值 `9e813cbd…` 的那一行受护，下一行（重判脚本、自证、变异表三个 sha256）不受护。

探针臂 `older-line-rewritten` 推的 `exit=0`；对照臂 `control-base-line-rewritten`（改写的是写着基准钉值那一行，同样本 `-inplace-rewrite-red`）推的 `exit=1`。

四句：分得出（改写哪一行，结论翻面）；格看得到基准那一版整节，只是只护一部分；满足第 3 问第三分句「两条都过而其实没修订」；判决第 4 条的改法（新加的一行里出现新钉值）对这一格不起作用。

### 合规的回退没有出路（误红）

登记在树里，装置回退成登记里早先写过的一版、钉值钉回它、第十二节新加一行「R2-1 的改动撤回，回退到登记时那一版」：二要求新钉值不在基准那一版第十二节里（`.claude/gate.d/doc-experiments.sh:4745`：「if "pin-seen-in-base-allowed" not in broken and any(has_pin(line) for line in previous_section):」），回退的钉值必在，判红；出路只有「换一个钉值」，即把装置改出一个字节的差别。探针臂 `intree-revert-with-new-line` 推的 `exit=1`。

我的改法（被攻过零轮）：一、三合成一条：基准那一版第十二节的每一个非空行（去首尾空白、词中间空白并成一个）都要还在，不只写着基准钉值的；二改成「新加的那一行（同样规整空白之后）与基准第十二节的每一行都不同，且写着新钉值」，回退因此放得过、抄一行与改空白照红；登记不在树里的一律红，出路写「起跑脚本改指一份在树里的登记（改名的改路径，归档的新写重跑登记）」，r2、r3 这类老起跑脚本随之要退役或改指——这是收严，交主 agent 定。

## 四、删旧闸 `:(literal)`（第 4 问）

判据是两条 git 都说「在 git 里、和 HEAD 没差」才删：`scripts/experiment-apparatus/e07-vm-bench-run.sh:416`：「output_pathspec=":(literal)$output_name"」，接着 418–419 两行 `ls-files --error-unmatch` 与 `diff --quiet HEAD`。`:(literal)` 只关掉了通配，「和 HEAD 没差」这一问在下面两种情形下答错。

### 打中：带 assume-unchanged 或 skip-worktree 索引标记的输出

提交过的 `e07.out` 带 `git update-index --assume-unchanged`（或 `--skip-worktree`，稀疏检出给的就是它），之后内容被改过、没提交：`ls-files` 认它在索引里；`diff HEAD` 对带这两种标记的项不去读工作区的文件，照索引报「没差」，删旧闸放行、`rm -f` 删掉改过的内容，HEAD 里只有提交时那一版（推的，git 对这两种标记的处理）。仓里自己的归档脚本把这一类当成已知的坑：`scripts/kb/archive-past-rounds.py:60`：「带 assume-unchanged 或 skip-worktree 索引标记的文件（Git 可能跳过差异检查，保守留下）」；删旧闸没有。今天主工作区 `git ls-files -v | grep -c '^[a-zS]'` 得 0，没有带标记的文件，概率低；后果是删掉找不回的输出，正是这道闸要防的那一件。

探针 `probe-delete.sh` 四份入口各一臂 `*-assume-unchanged-modified`、`*-skip-worktree-modified`，推的 `exit=2 output_kept=false refused_by_delete_gate=false reached_build=true`；对照臂 `*-control-untracked` 推的 `exit=2 output_kept=true refused_by_delete_gate=true reached_build=false`。

四句：分得出（只差索引标记，结论翻面）；判法当时看得到索引标记，没问；满足文件头 `scripts/experiment-apparatus/e07-vm-bench-run.sh:27` 的「输出文件已存在而在 git 里没有（或在 git 里而有没提交的改动）的不删」；判决第 3 条的改法（加 `:(literal)`）对这一格不起作用。

### 大小写

- 调用环境带 `GIT_ICASE_PATHSPECS=1` 时，git 给每个 pathspec 加 icase，与逐项的 `:(literal)` 并存（推的，git 只拒 literal 与 glob 同时给）：没进 git 的 `E07.out` 会对上提交了的 `e07.out`，`diff HEAD` 比的是 `e07.out`、没差，删掉 `E07.out`。没有人会设这个变量，只记一笔；探针臂 `*-icase-environment` 推的被删。
- `core.ignorecase=true`（区分大小写的盘上手设）不进 pathspec 的比对（推的，pathspec 的不分大小写只认 icase），探针臂 `*-ignorecase-config` 推的被拒。两台的共享根今天都在区分大小写的盘上（推的，没核两台的挂载类型）。
- 带 `:` 起头的文件名：`:(literal)` 之后整串按字面，不再解魔法（读码）。

### 第三处：没有

四份入口里碰输出路径的 git 只有那两条（`grep -n 'git ' scripts/experiment-apparatus/e{07,53,159,56}-vm-bench-run.sh` 去掉注释与自证，各剩 `ls-files` 与 `diff` 两行：e07 第 418–419 行、e53 第 441–442 行、e159 第 438–439 行、e56 第 412–413 行），`[[ -e ]]` 与 `rm -f --` 都按字面。第 4 问后半「不成立」。

我的改法（被攻过零轮）：不问「在不在 git 里、有没有差」，直接比内容：`git -C <目录> rev-parse --verify --quiet "HEAD:./<文件名>"` 取 HEAD 里这一条路径的 blob（树路径，不是 pathspec，不受索引标记、icase、通配影响），与 `git -C <目录> hash-object -- <文件名>` 相同才删；任一条失败照拒。这一改把第二轮的 `:(literal)` 也一起替掉，四份入口同改，自证加带两种标记的两格。

## 五、完工清理闸「下层已声明交接的副本，上层交回时算已声明」（第 5 问）

判法：子 agent 交回时扫自己的认领目录与草稿目录，往下每一层不是副本的目录里有 `.deliverables` 就读，按子 agent 的角色收免列的声明（`.claude/hooks/completion-cleanup-guard.sh:879`：「if nested_role is not None and depth > 0 and kind is None and DECLARATION_FILE in names:」，下一行 `exempt.update(declarations([path], nested_role)[0])`）。

### 下层是别的会话、别的 agent 的目录：上层照认

扫描往下走时不看下层目录归谁：认领记录的归属只在挑起头的那几处时看（`.claude/hooks/completion-cleanup-guard.sh:742`：「if owner is not None and owner != session:」，在 `subagent_roots` 里），`scan` 里没有。所以别的会话认领在上层目录底下的子目录、另一个子 agent 的草稿目录，写的 `.deliverables` 上层都当已声明（读码，推的）。后果有界：

- 下层能声明的只有它自己那一层底下的路径（`declarations` 里落到那个目录外的行不算），伸不到上层别处。
- 免列的种类照子 agent 的那一套（报告、补丁、备份、交接），「基线」不认；编译目录声明了照列。
- 主 agent 收工时不读下层声明（`.claude/hooks/completion-cleanup-guard.sh:1962` 只在子 agent 的角色下给 `nested_role`），交接的副本照列，由主 agent 删。

所以借下层声明放过上层，最多把「交回时拦」推迟到「主 agent 收工时列」，副本不会没人管。第 5 问前半「成立」，没有打中有后果的形状。

### 下层 `.deliverables` 写坏、被删、换成符号链接

- 写坏的行：不算声明；下层这一层的坏行不拦上层（`.claude/hooks/completion-cleanup-guard.sh:1956` 只判自己那几层的坏行），副本没被声明，上层交回时被拦、点名副本。
- 被删：同上，副本照列、上层被拦。
- 符号链接：`.claude/hooks/completion-cleanup-guard.sh:775`：「if os.path.islink(path) and not broken("declaration-symlink-skipped"):」判写法不合、整份不读；嵌套那一路只取声明（`[0]`），这条写法不合被丢掉，副本照列、上层被拦。
- 下层目录本身是符号链接：扫描不跟目录链接（`.claude/hooks/completion-cleanup-guard.sh:906`：「is_directory = entry.is_dir(follow_symlinks=False)」），里面有什么都不看。这是旧行为，不归这一轮。

三种都往严的一边错。只有一处不顺：上层被拦的说明只点名副本，不说是下层那份 `.deliverables` 的哪一行写坏或是符号链接，上层要自己去找；出路照旧走得通（在自己那一层的 `.deliverables` 写 `lower/copy 交接 <理由>`，或交回正文写「交主 agent 删」那一行）。不建议为它改。

## 第二轮正文与判决里只抽查过、没看过的地方（在我的攻击面内）

- 第二轮判决第 2 条说「同一个实验有 timing 行时」，没说复跑表行键怎么归到实验号：第二节打中的那一格就落在这里。
- 第二轮判决第 4 条「登记不在树里时从 git 历史里取它被删前的那一版判」：没看过改名那一支（第三节）。
- 换上工具文件头「留下的缝」只列了两种，第二轮没人看过摘的那一边停住与按仓的那一份上的后果（第一节）。
- 正文说开关「18 条开关减去 16 格副本缺文件的环境格之后逐条红在文件头点名的格」：那 16 格在副本上没判过，归正推腿第 8 问，我没攻。

## 没打中的形状

- 第 1 问：`.taking` 与记录的修改时间用不同的钟（两者都是这一台的 `time.time()` 减文件修改时间，同一个判法）；同一台上活着的摘的那一趟被误删 `.taking`（这一台看进程，活着就挡）；摘的当中被 TERM、放到一半再被 TERM（第二次进同一个处理，放完再死）；`link` 回复丢了被当成别人的（读回来比内容）。
- 第 2 问：`.out` 与 `.out.gz` 两份同时在盘时取了旧的那一份（`.out` 优先，重跑存回的通常是 `.out`）；同名产物在 `research/results/` 不同子目录里各一份时按遍历次序取了第一份（今天的 timing 产物名没有重名，推的）；补零的实验号（`e07`）与复跑表 `E7` 对不上（装置目录里 `e0` 起头的只有 `e07-vm-bench-run.sh`，不是 `.rs`，不进这一格）。
- 第 3 问：第十二节以外的节里出现新钉值（样本 `-pin-in-other-section-red` 罩着）；围栏里放一个假的「## 十二、」（围栏里的标题不算）；在原第十二节之前再插一个「## 十二、」标题（取到的是新插的那一节，写着基准钉值的行都不在，判红）。
- 第 4 问：`GIT_LITERAL_PATHSPECS=1`（整串当文件名，对不上、照拒）、`GIT_GLOB_PATHSPECS=1`（与逐项 literal 冲突，git 报错、照拒）、`GIT_DIR` 指到别的仓（对上的话内容在那个仓的 HEAD 里，删了找得回）、输出路径带 `..` 或经目录链接（`git -C` 与 `rm` 走的是同一条内核路径）。
- 第 5 问：下层声明伸到下层目录外（`declarations` 按字面判，`..` 规整之后落在外面的不算）；下层声明「基线」（子 agent 角色不认）；声明编译目录（照列）。

## 这条腿自己的限度

- 探针都没跑：子 agent 不往任务服务交任务。报告里每一个 `exit=`、`believers=` 都是读码推的，要主 agent 跑出来才算。
- 第 1 问的两臂用一个进程里的两条线程演两趟：进程号相同，判「这一台、进程在」的那一支两边都看成活着；交错是用包住 `os.unlink` 与 `os.link` 的闸门硬排的，证明的是「这个次序走得到、走到了就两趟都占」，不是「多常见」。另一台用改写身份里的机器标识演，没在第二台上跑。
- git 对 assume-unchanged、skip-worktree、`GIT_ICASE_PATHSPECS` 与 `core.ignorecase` 的行为是按我对 git 的了解推的，要靠第 4 问的探针坐实；两台共享根的盘区不区分大小写没核。
- 第 2 问只数了今天复跑表与 `research/results/` 的状态，没估每条 timing 行重跑要多久。
- 没读：派发提示禁读的五份报告；第二轮攻方腿的模型目录（没点名禁读，没打开）；`stage-owners.tsv` 三行只读了原文，没攻（不在第 1 到 5 问里）。
- 没写模型目录、没用 `scripts/agents/three-way-rerun-template.sh`：派发提示要探针放草稿目录，四份探针都只读仓、样本都建在给的工作目录里。

## 没做什么

- 不判正推腿那三问（6 到 8）；不替主 agent 采纳；四条改法都只在读码上推过、被攻过零轮，没在副本上实现、没跑 A/B。
- 没往统一任务服务交任务，判决要引的数没复跑：命令在下一节。
- 没建仓副本与编译目录；草稿目录里只有四份探针、`timing-rows.txt`（第二节那条 `grep` 的输出）与认领、写者锁文件。

## 要主 agent 跑的命令

四条互不依赖，都在草稿目录起；每条把样本仓或记录建在给的工作目录下（那几个样本是 `git init` 的小仓，看完连目录删）。

1. 第 1 问：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=攻方腿探针 probe-taking.py 的六臂; 范围=6 臂; 为什么=第一问两趟交错摘已死的 taking 与摘的那一趟停住各判几趟占到，判决要引的读数要主 agent 跑出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus -- python3 /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/probe-taking.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/run-taking`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus`
   - 看哪几行：`TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里六行 `arm=`，末行 `probe-taking done`；打中看 `arm=double-dead-taking-repository … believers=2` 与 `arm=remover-stall-elsewhere … believers=2`，对照看 `double-dead-taking-sequential` 与 `remover-stall-same-machine` 的 `believers=1`；`stuck=` 不是 `none` 的那一臂作废。
2. 第 2 问：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=攻方腿探针 probe-evidence.sh 的四臂，每臂只跑 doc-experiments 的 evidence-in-repo 一格; 范围=4 臂; 为什么=第二问复跑表行键带后缀的 timing 行与只改变异表各判什么，判决要引的读数要主 agent 跑出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/probe-evidence.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/run-evidence`
   - 在哪起：同上
   - 看哪几行：日志里四行 `arm=… exit=…` 与各自下面缩进的 ✓ / ✗ 句；打中看 `arm=suffix-timing-row exit=0`，对照 `arm=suffix-timing-control exit=1`。
3. 第 3 问：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=攻方腿探针 probe-pin.sh 的七臂，每臂只跑 doc-experiments 的 pinned-apparatus-revision 一格; 范围=7 臂; 为什么=第三问作废钉值、登记改名、原地改写旧行与合规回退各判什么，判决要引的读数要主 agent 跑出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/probe-pin.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/run-pin`
   - 在哪起：同上
   - 看哪几行：七行 `arm=… exit=…`；打中看 `older-line-rewritten`、`archived-revert-to-superseded`、`renamed-registry-pin-kept` 的 `exit=0`，`intree-revert-with-new-line` 的 `exit=1`；对照 `control-base-line-rewritten exit=1`；两条改名臂上面一行的 `deletion_commit_found=`。
4. 第 4 问：
   - 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=攻方腿探针 probe-delete.sh，四份虚机入口各五臂只走到删旧闸与假构建; 范围=20 臂; 为什么=第四问索引标记与大小写下删旧闸删不删没进 git 或改过的输出，判决要引的读数要主 agent 跑出' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus -- bash /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/probe-delete.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/open-fixes-governance-r3-opus/run-delete`
   - 在哪起：同上
   - 看哪几行：二十行 `arm=e<号>-<臂> …`；打中看 `*-assume-unchanged-modified` 与 `*-skip-worktree-modified` 的 `output_kept=false reached_build=true`；对照 `*-control-untracked` 的 `output_kept=true refused_by_delete_gate=true`；`exit` 不是 2 的臂下面贴了日志末三行（入口的运行条件不满足会在这里看到，那一臂作废）。
