# apparatus-rename-r2 攻方腿（Opus）报告（2026-10-23）

轮名 apparatus-rename-r2，第二轮；攻击面是正文「三、格」H1–H3 与「四、分工」攻方那一行，第一节列的已攻角度（G1-1 到 G4）不重复。被判的 15 份从主工作区拷出，`build_frozen.sh` 末尾逐个核开工快照 `research/prompts/apparatus-rename-r2-snapshot/judged-sha256.txt`，15 个 OK。这一题没有盘、没有崩溃状态，第 3b 步的崩溃枚举用不上；样本全是临时 git 仓加 python，没有编译出来的代码。

## 复跑

模型目录 `research/prompts/apparatus-rename-r2-opus-model/`。两步（工作目录随意）：

```
bash research/prompts/apparatus-rename-r2-opus-model/build_frozen.sh <仓根> <新目录>/frozen
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/apparatus-rename-r2-opus-model/tools/rerun.sh --repo <仓根> --frozen <新目录>/frozen --work <新目录>/rerun
```

`rerun.sh` 跑样本（`attack_samples.py`，临时仓）与主工作区只读现查（`probe_main_repo.py`，git 一律带 `GIT_OPTIONAL_LOCKS=0`，不刷新、不写主仓的索引），逐份比 `out/` 下两份。我这边经内存包装复跑一次的末三行（原样）：

```
name=rerun_compare file=attack_samples.out same=yes
name=rerun_compare file=probe.out same=no（读主工作区此刻的样子，会变）
name=rerun_done same=1 differ=1
```

`probe.out` 只差一行：改动范围的条数 5552 变成 5556（别的会话在加文件），「只差搬迁」两种算法的 725、730 都没变。样本那一份另直接起过两趟，`cmp` 相同。

| 文件 | sha256 |
|---|---|
| attack_samples.py | c8c8b7e620e80600a5b3124374ed05f43a4ec55fafa8ab5000a292d5653fa8e3 |
| build_frozen.sh | eea6c2977d52b2fe35c9cfca291ebecb27e7049727621fc7a8bddbf2f121dca8 |
| probe_main_repo.py | 14f33c289b1d27e3cd15f1bf3e8a3e46205ad60ee8cba5f3266ed58f282b464c |
| tools/rerun.sh | 3d5043a99592da2aab4e1b3aaea83b32aee66ed9506ecf080c39b2516415a671 |
| out/attack_samples.out | 722044f01f59f520f77ea2a190ee03b57d77916c9dba031010dd2d0a3b36d0dc |
| out/probe.out | c726c4abd196a24df6d9cdf16dcca82d15f07de3821f43d34dbf6f344c14cb91 |

## 各格判定一览

| 格 | 判定 | 打中的东西（样本名，在 `out/attack_samples.out` 里 `name=` 开头） |
|---|---|---|
| H1 修补本身 | **打中，6 处** | H1-1 新加一行插在基准里已有的那一行前面，自己的新前缀在基准里是空的，串上老行之后落在早就在的另一份实现上：甲判有实据，改动范围里调用方、搬了家的豁免脚本都没了，乙放行重钉，丙两处钉值都认（`chain_through_registered_row`）；H1-2 一对来回登记 A→B、B→A，磁盘上什么都没搬，满足甲的三条，B 换成 A 的改动被剔（`round_trip_pair`）；H1-3 落点是被 `.gitignore` 忽略、早有另一份实现的目录，甲只核「此刻在磁盘上」（`ignored_landing`）；H1-4 登记行提交以后永远生效：把旧名字重新用起来的真搬迁当时判绿，提交以后什么都不改、豁免表判「涨了」（`stale_row_moved_back`）；H1-5 戊按目录分类，同一类里从两格挑不中的名字改成挑得中的名字照剔（`governed_same_class`）；H1-6 丁按 hunk 配对，同一个 hunk 里挪过节标题的那一行照剔（`hunk_internal_relocation`）。「只核新加的行」在改写日期出处、删了再加、改写新前缀时守得住（`stale_row_edits`），换次序守不住（就是 H1-1） |
| H2 得出结论的方法 | **打中，3 处** | H2-1 三项自证分得出第一轮被打中的那几种（退回「过半」、去掉占用核对都红），分不出这一轮的：补了我的两个改法的实现与被判的实现在自证上一样全绿（`selftest_distinguish`）；H2-2「真登记那一行照过」是阳性对照，去掉占用核对、整个实据不核的实现照样过（`out/probe.out` 的 `real_row_control`）；H2-3 报数与判法出自同一个循环。另一条不共用代码的路子重算 725 份：被判的那份剔的全在独立算法的集合里，独立算法多 5 份，都是边界写法（`\n`、`:-`），方向是多判改动、不放过；395 份里相同的 360 对 362，同一原因。这一半是佐证，不是打中 |
| H3 没看过的地方 | **打中，3 处** | H3-1 九处调用方都经 `baseline_text` 拿换算之后的基准，老登记行永远作用在基准上：旧名字重用之后按路径做键的表判「涨了」（量过一处：长活豁免表，见 H1-4；别的几张表同一条代码路，推的）；H3-2 `scripts/gate/changed-paths.sh` 第 29 行还写「按次数抵」，丁改完没跟着改；H3-3 `.claude/rules/path-moves.md` 第 6 行仍写「不留一处旧路径」，与第 5 步对不上。跨管辖类搬家时 `drop_moved_lines` 照旧剔行：今天取新增行的六处调用（`gate_added_lines` 四处、直接调 `drop_moved_lines` 两处）没有一处读按路径管的类，没打中 |

改法：我提了 5 个，每个都**只在我的模型上量过、被攻过零轮**，逐格「量过 / 推的」在文末「改法与各格」。

## H1 修补本身

### H1-1 新加一行插在基准里已有的那一行前面（`chain_through_registered_row`）

历史：更早的提交里 B 那份实现在 `old-b/run.sh`；基准提交把它 `git mv` 到 `tools/real-b/run.sh`、登记 `old-b → tools/real-b` 一起提交了（第一轮之后的常态：本批提交以后，主工作区那一行 `scripts/experiments → scripts/experiment-apparatus` 就是这样一行老行）。基准里另有 A 那份 `tools/real-a/run.sh`、起 A 的 `ops/launch.sh`、豁免表钉着的 `bin/live.sh`（`worker=tools/real-a/run.sh`）。这一次：`git rm -r tools/real-a`；`ops/launch.sh` 改成起 B；`bin/live.sh` `git mv` 成 `bin/live-host.sh`、worker 换成 B，豁免表换路径与 sha256；登记表三行，新加的 `tools/real-a → old-b` 写在老行前面。

每一步许可它的那一句：
- 新加的行只核自己：`.claude/gate.d/lib-lag-baseline.py:775`：「fresh = [(index, move) for index, move in enumerate(moves) if (move.old, move.new) not in registered]」，老行不再核。
- 占用只按这一行之前的登记换算、只看这一行自己写的新前缀：`.claude/gate.d/lib-lag-baseline.py:785`：「positions = {path: forward_path(path, moves[:index]) for path in files}」，`.claude/gate.d/lib-lag-baseline.py:787`：「occupied = sorted(path for path in files if under_prefix(positions[path], move.new))」。`old-b` 下在基准里一份都没有（早搬走了），过。
- 落点按全部登记换、只要「在」：`.claude/gate.d/lib-lag-baseline.py:799`：「target = forward_path(path, moves)」，`.claude/gate.d/lib-lag-baseline.py:800`：「current = worktree_version(root, target)」。`tools/real-a/run.sh` 串到 `tools/real-b/run.sh`，那一份是 B 的实现，在，过。换算之后相同的份数只报：`.claude/gate.d/lib-lag-baseline.py:806`：「same += mode == tree[path].mode and (data == before or data == rewrite_moved_bytes(before, moves))」。
- 磁盘那一半：新前缀 `old-b` 照后面的老行换到 `tools/real-b`，在；旧前缀 `tools/real-a` 不在了。
- 乙：脚本自己确实搬了家，`.claude/hooks/lib_long_run.py:445`：「if os.path.normpath(taken) == os.path.normpath(path) and "exempt-unmoved-accepted" not in breaks():」不拦；内容照全部登记正着换，`.claude/hooks/lib_long_run.py:454`：「if lag_baseline.rewrite_moved_bytes(data, moves) == current:」，worker 那一处经两行串成 B，相同，放行。
- 丙：此刻内容里没有 `tools/real-a`、`old-b`，正着换一遍不动，`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:74`：「if not broken("partial-accepted") and self.library.rewrite_moved_bytes(data, list(self.moves)) != data:」不拦；逐次倒到全部三行，`tools/real-b` 倒回 `old-b` 再倒回 `tools/real-a`，等于钉住的那一版。

原样输出（`out/attack_samples.out` 第 1–10 行）：

```
name=chain_through_registered_row.base launch='走的是 A' base_rows=['old-b\ttools/real-b\t2026-10-23\trecords/move.md']
name=chain_through_registered_row.control_direct.path_move_report rows=3 problems=['第 2 行的新前缀 tools/real-b 下有基准 HEAD 里本来就在的文件（tools/real-b/run.sh，共 1 份）：这一行不是一次搬迁'] note='新加的 2 行（基准 HEAD 那一版的登记表里没有）实据不够，见上'
name=chain_through_registered_row.control_after_old_row.path_move_report rows=3 problems=['第 3 行的新前缀 old-b 不在磁盘上', '第 3 行：基准 HEAD 里旧前缀 tools/real-a 下的 1 份，照登记换到的落点上有 1 份此刻不在（old-b/run.sh）'] note='新加的 2 行（基准 HEAD 那一版的登记表里没有）实据不够，见上'
name=chain_through_registered_row.attack.path_move_report rows=3 problems=[] note='新加的 2 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 2 行 1 份里 0 份换算之后相同；第 3 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=chain_through_registered_row.attack.final_landing_arm problems=['第 2 行的新前缀 old-b 最终落在 tools/real-b，那里有基准里本来就在的文件（tools/real-b/run.sh）']
name=chain_through_registered_row.attack.gate_changed_paths rc=0 changed=['.claude/gate.d/path-moves.tsv', '.claude/long-run-live-hosts-exempt', 'tools/real-a/run.sh'] stderr=''
name=chain_through_registered_row.attack.gate_added_lines rc=0 added=['.claude/gate.d/path-moves.tsv\ttools/real-a\told-b\t2026-10-23\trecords/move.md', '.claude/gate.d/path-moves.tsv\tbin/live.sh\tbin/live-host.sh\t2026-10-23\trecords/move.md', '.claude/long-run-live-hosts-exempt\tbin/live-host.sh  29d1d8c273c570a6b8865f521ac959628c1ce4f4160e1f5f7e7a3bb28f06d6ee  # 样本：文件头按现算写、不能改，只缩不涨'] stderr=''
name=chain_through_registered_row.attack.real_change launch_now='走的是 B' diff=['-bash tools/real-a/run.sh', '+bash tools/real-b/run.sh', '']
name=chain_through_registered_row.attack.exempt_check result=(0, '✓ .claude/long-run-live-hosts-exempt 1 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；与基准那一版比没涨（其中 1 行照 .claude/gate.d/path-moves.tsv 登记的搬迁换算之后与基准那一版逐字节相同：bin/live-host.sh）')
name=chain_through_registered_row.attack.pins frozen_o3_same=True reuse_library_pin_accepted=True
```

结果：门禁 code-tooling 的 change-range-single-source 格调的那一份判绿；改动范围（经冻结树的 `scripts/gate/changed-paths.sh` 现取）里 `ops/launch.sh` 与搬了家又换了 worker 的 `bin/live-host.sh` 都没了（`git mv` 暂存成改名，旧名 `bin/live.sh` 本来就不单列），剩下删掉的 `tools/real-a/run.sh`；`ops/launch.sh` 现在起的是 B（`launch_now='走的是 B'`）；豁免表重钉不报涨；E171 两处钉值核对都认。对照臂：同样三行把新加的行直接写成 `tools/real-a → tools/real-b` 红在占用；把它挪到老行后面红在「新前缀不在磁盘上」。

在主工作区上（推的，同一条代码路）：本批提交以后，老行 `scripts/experiments → scripts/experiment-apparatus` 进了基准；任何一行 `<X> → scripts/experiments/<名>` 写在它前面，落点就是 `scripts/experiment-apparatus/<名>` 那一份早就在的文件（单文件前缀也收），G1-2 第一轮堵上的「把 A 换成早就在的 B」就从这条路回来了。

四句：
1. 分不分辨臂：分辨。被判的 `move_evidence` 中；按最终落点核占用的那一版不中（量过：`final_landing_arm` 那一行与 `selftest_distinguish.final_landing.chain_through_registered_row…` 那一行都报「最终落在 tools/real-b」）。
2. 系统当时看得到吗：看得到。`path_move_report` 在同一个函数里已经把新前缀照后面的登记换到落点（核「新前缀在磁盘上」那一步），手里有基准树与整张登记表。
3. 满足的是字面哪一句：H1「甲去掉『过半』之后，有没有一种不是搬迁、却满足三条的登记」与「『只核新加的行』在登记行……换了次序时还守不守得住」。三条逐条满足：旧前缀在基准里有文件、每份照登记换到的落点上都在、新前缀（这一行自己写的 `old-b`）在基准里没有文件。
4. 跑前的改法在这几格上：甲中（就是这一格）；乙中（脚本真搬了家，乙只拦没搬家的）；丙中（正着换不动、逐次倒得回钉值）；丁、戊与这一格无关。

### H1-2 一对来回的登记，什么都没搬（`round_trip_pair`）

历史：基准里 `tools/live/run.sh`（仓里那一份实现），`ops/launch.sh` 起 `build/out/run.sh`（生成物的落点，基准里没有文件、磁盘上也没有）。这一次 `ops/launch.sh` 改成起 `tools/live/run.sh`，登记两行 `tools/live → build/out`、`build/out → tools/live`，别的什么都不动。

许可它的那几句：第一行，旧前缀 `tools/live` 在基准里有文件；新前缀 `build/out` 在基准里没有文件（`.claude/gate.d/lib-lag-baseline.py:787`，同上）；落点照全部登记换完回到 `tools/live/run.sh`，在（`.claude/gate.d/lib-lag-baseline.py:799`，同上）。第二行，按第一行换算过后基准里的文件都「在」`build/out` 下，旧前缀有文件；新前缀 `tools/live` 按第一行换算过后没有文件；落点在。磁盘那一半：第一行旧前缀 `tools/live` 还在磁盘上，可后面一行搬回了它，`.claude/gate.d/lib-lag-baseline.py:844`：「moved_back = any(under_prefix(following.new, move.old) or (parent_counts and move.old.startswith(following.new + "/")) for following in later)」豁免；第二行旧前缀 `build/out` 不在磁盘上。换算的净效果是 `build/out` → `tools/live`、`tools/live` 不变：基准里任何写 `build/out` 的地方，此刻换成 `tools/live` 就算只差搬迁。

原样输出（`out/attack_samples.out` 第 11–16 行）：

```
name=round_trip_pair.control_one_row.path_move_report rows=1 problems=['第 2 行的新前缀 tools/live 下有基准 HEAD 里本来就在的文件（tools/live/run.sh，共 1 份）：这一行不是一次搬迁', '第 2 行的旧前缀 build/out 在基准 HEAD 里没有文件：这一行不是一次搬迁'] note='新加的 1 行（基准 HEAD 那一版的登记表里没有）实据不够，见上'
name=round_trip_pair.attack.path_move_report rows=2 problems=[] note='新加的 2 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 2 行 1 份里 1 份换算之后相同；第 3 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=round_trip_pair.attack.final_landing_arm problems=['第 2 行照全部登记换完，旧前缀下的文件落回原处：没有东西搬家', '第 3 行照全部登记换完，旧前缀下的文件落回原处：没有东西搬家']
name=round_trip_pair.attack.status porcelain=[' M ops/launch.sh', '?? .claude/']
name=round_trip_pair.attack.gate_changed_paths rc=0 changed=['.claude/gate.d/path-moves.tsv'] stderr=''
name=round_trip_pair.attack.pins frozen_o3_same=True reuse_library_pin_accepted=True
```

`git status` 只有 `ops/launch.sh` 改了与新建的登记表；改动范围只剩登记表；钉值那一侧也认（钉住的那一版写 `build/out/run.sh`，此刻写 `tools/live/run.sh`）。对照：只登记 `build/out → tools/live` 一行，两条实据都红。

四句：分辨臂（被判的中；我的改法一里「照全部登记换完，旧前缀下的文件落回原处」那一条不中，量过，第 13 行）；看得到（落点等于旧前缀，换一遍就知道）；满足的是 H1「不是搬迁、却满足三条的登记」，三条逐条满足；跑前改法：甲中，乙对这一格不起作用（这一类改动不碰豁免表时乙用不上；碰豁免表时脚本「自己没搬家」，乙拦，推的），丙中（钉值认，第 16 行）。

### H1-3 落点在被忽略的目录里（`ignored_landing`）

历史：`.gitignore` 写 `build/`；基准里 `tools/real-a/run.sh` 与起它的 `ops/launch.sh`。磁盘上 `build/run.sh` 早有另一份实现（没进 git）。这一次 `git rm -r tools/real-a`，`ops/launch.sh` 改成起 `build/run.sh`，登记 `tools/real-a → build`。

许可它的那一句：落点只核「此刻在磁盘上」，读的是工作区：`.claude/gate.d/lib-lag-baseline.py:441`：「if os.path.isfile(full):」（`worktree_version`），不核在不在索引里；占用只看基准树里的文件，被忽略的 `build/` 在基准树里一份都没有。

```
name=ignored_landing.path_move_report rows=1 problems=[] note='新加的 1 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 2 行 1 份里 0 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=ignored_landing.final_landing_arm problems=[] untracked_landings=['第 2 行：tools/real-a/run.sh 的落点 build/run.sh 不在索引里']
name=ignored_landing.gate_changed_paths rc=0 changed=['.claude/gate.d/path-moves.tsv', 'tools/real-a/run.sh'] stderr='' launch_now='走的是没进 git 的另一份'
```

`build/run.sh` 被忽略，不在改动范围里，`ops/launch.sh` 被剔；改动范围里只看得见「删了 `tools/real-a/run.sh`」，看不见它换成了什么。提交以后干净的检出里 `ops/launch.sh` 指空。

四句：分辨臂（被判的中；改法一不中这一格，量过：`final_landing_arm problems=[]`；改法二「落点要在索引里」中，量过：`untracked_landings` 报了 1 条）；看得到（`git ls-files` 就有）；满足的是 H1「有没有一种不是搬迁、却满足三条的登记」，第二条「此刻照登记换到的落点上逐份都在」按工作区读；跑前改法里只有甲碰得到这一格，中。

### H1-4 登记行提交以后永远生效：旧名字重用之后，没改的表判「涨了」（`stale_row_moved_back`，正文 H1 要的「被误判的真搬迁」）

历史：第一次提交把 `scripts/experiments` `git mv` 成 `scripts/experiment-apparatus`，登记连同 `tools/new-thing/live.sh`（豁免表钉着）一起提交。第二次：`git mv tools/new-thing scripts/experiments`（空出来的名字重新用起来），豁免表的路径跟着换，登记表加 `tools/new-thing → scripts/experiments`。当时判一次；提交以后工作区一个字节不改，再判一次。

许可它的那几句：`baseline_text` 交全文之前照全部登记换，不管哪一行早已在基准里：`.claude/gate.d/lib-lag-baseline.py:341`：「text = rewrite_moved_paths(text, moves)」。第二次提交以后，基准那一版豁免表写的是新东西的路径 `scripts/experiments/live.sh`，被老行换成 `scripts/experiment-apparatus/live.sh`；此刻的表没换，`.claude/hooks/lib_long_run.py:330`：「if path not in base_rows:」判涨。

```
name=stale_row_moved_back.before_commit.path_move_report rows=2 problems=[] note='新加的 1 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 3 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=stale_row_moved_back.before_commit.exempt_check result=(0, '✓ .claude/long-run-live-hosts-exempt 1 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；与基准那一版比没涨')
name=stale_row_moved_back.after_commit.status porcelain=[]
name=stale_row_moved_back.after_commit.path_move_report rows=2 problems=[] note='没有新加的行（都在基准 HEAD 那一版的登记表里），实据不再核'
name=stale_row_moved_back.after_commit.exempt_check result=(1, '✗ .claude/long-run-live-hosts-exempt 有 1 处不对（共 1 行）：\n     第 2 行 scripts/experiments/live.sh 是基准那一版里没有的（豁免表涨了）\n     → 怎么办：豁免只缩不涨：新脚本的文件头按登记值写；确实要加的，交主 agent 问用户再加')
name=stale_row_moved_back.after_commit.baseline_text rows=['scripts/experiment-apparatus/live.sh  0b872096c65a3316ee37e87fa3ac72138c541d1f07145b68294c47ff643e76c8  # 样本：文件头按现算写、不能改，只缩不涨'] problem=None
```

同一份表、同一次真搬迁：提交前绿，提交后什么都没改就红，出路写的是「交主 agent 问用户再加」。同一条代码路上的另几处（`.claude/heavy-thread-exempt` 的 heavy-thread-defaults 格、挂账表、两份变更史的 history-loss 格）都经 `baseline_text` 拿换算之后的基准，旧名字下的新文件被老行换走，一样判错（推的，没逐格起）。

四句：分辨臂（被判的中；改法三「基准那一版登记表里已有的行，给门禁那一侧换算基准时不再用」不中，推的：第二次提交以后两行都在基准表里，基准原样交出，与此刻逐字相同）；看得到（基准那一版的登记表就在手上，`move_evidence` 已经读它算「新加的行」）；满足的是 H1「或被误判的真搬迁」与「『只核新加的行』……守不守得住」的另一面：老行不核，也不退；跑前改法：甲不碰这一格（判绿），乙用不上（重钉那一步没走到，先红在路径涨了），丙、丁、戊无关。

H1-4 的另一半（与第一轮 G1-3 同一个后果，触发不同，列在这里不另算一处）：判决的己让第 5 步写明指 git 历史的句子留旧路径，主 agent 也把两行改回了旧路径；可老行永远生效，提交以后谁把这一行再改成新路径，改动范围与新增行里都看不见。样本里搬迁那次提交的记录写 `git log --format=%s -- scripts/experiments/replay.sh`，之后改成新路径：

```
name=stale_row_moved_back.kept_line_rewritten gate_changed_paths=[] rc=0 gate_added_lines=[] rc_lines=0 command_now=['搬迁那次（登记表连同第 5 步留旧路径的那一行一起提交）'] command_then=['搬迁那次（登记表连同第 5 步留旧路径的那一行一起提交）', '更早']
```

换完之后那条命令只打出一行，换之前两行。主工作区里本批提交以后同样成立的一处（推的）：`records/2026-10-13-里程碑四预开发.md` 第 838 行（第一轮判决第 42 行点名、主 agent 改回旧路径的那一行）。

### H1-5 同一类里从挑不中的名字改成挑得中的名字（`governed_same_class`）

历史：基准里 `.claude/agents/reviewer.md.draft`、`.claude/agents/drafts/helper.md`、`crates/core/src/fast_path.rs.txt`（加进来的那一次，两格的正则同样挑不中它们），另有一份 `drafts/agents/cross.md` 当跨类对照。这一次 `git mv` 成 `.claude/agents/reviewer.md`、`.claude/agents/helper.md`、`crates/core/src/fast_path.rs`、`.claude/agents/cross.md`，`crates/core/src/lib.rs` 加 `mod fast_path;`，四行单文件登记。

许可它的那几句：类按目录分，`.claude/gate.d/lib-lag-baseline.py:149`：「GOVERNED_DIRECTORIES = (".claude/agents/", ".claude/rules/", ".claude/hooks/")」、`.claude/gate.d/lib-lag-baseline.py:150`：「CRATE_SOURCE_DIRECTORY = re.compile(r"^crates/[^/]+/src/")」；只有类不同才照算，`.claude/gate.d/lib-lag-baseline.py:541`：「return governed_kept and old_path != new_path and governed_class(old_path) != governed_class(new_path)」。而两格挑文件按扩展名与层级：`.claude/gate.d/doc-process-records.sh:924`：「all_sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed_since_gate" || true)"」（crates-adversarial-review），`.claude/gate.d/doc-process-records.sh:1788`：「definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed_since_gate" || true)"」（agent-def-adversarial-review）。

```
name=governed_same_class.base_commit_selected agent_definitions=[] crates_sources=['crates/core/src/lib.rs']
name=governed_same_class.control_unregistered changed=['.claude/agents/cross.md', '.claude/agents/helper.md', '.claude/agents/reviewer.md', 'crates/core/src/fast_path.rs', 'crates/core/src/lib.rs'] agent_definitions=['.claude/agents/cross.md', '.claude/agents/helper.md', '.claude/agents/reviewer.md'] crates_sources=['crates/core/src/fast_path.rs', 'crates/core/src/lib.rs']
name=governed_same_class.registered.path_move_report rows=4 problems=[] note='新加的 4 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 2 行 1 份里 1 份换算之后相同；第 3 行 1 份里 1 份换算之后相同；第 4 行 1 份里 1 份换算之后相同；第 5 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=governed_same_class.registered rc=0 changed=['.claude/agents/cross.md', '.claude/gate.d/path-moves.tsv', 'crates/core/src/lib.rs'] agent_definitions=['.claude/agents/cross.md'] crates_sources=['crates/core/src/lib.rs'] stderr=''
name=governed_same_class.registered_in_process kept=['.claude/agents/cross.md', '.claude/gate.d/path-moves.tsv', 'crates/core/src/lib.rs'] agent_definitions=['.claude/agents/cross.md'] crates_sources=['crates/core/src/lib.rs'] problem=None
```

不登记时三份新生效的文件都挑得出；登记之后只剩跨类的 `cross.md` 与 `lib.rs`。`lib.rs` 那一行 `mod` 看得见，`fast_path.rs` 的正文看不见；`.claude/agents/reviewer.md` 是一份新生效的子 agent 定义，agent-def-adversarial-review 挑不出它。两步走就能放进去：第一步以挑不中的名字加进来（两格都不看），第二步同类改名（剔掉）。仓里今天就有挑不中的正文住在挑得中的目录里：`crates/singlefs-checker-tier/src/` 下 6 份 `.wgsl`（`git ls-files crates/singlefs-checker-tier/src/ | grep -c '\.wgsl$'` 现查，打 6）。

四句：分辨臂（被判的中；改法四「类按两格挑文件的正则分」不中，量过：`selftest_distinguish.class_by_reader.governed_same_class.registered_in_process` 那一行三份都留下）；看得到（新旧路径都在 `moved_only_paths` 手里）；满足的是 H1「戊（跨管辖类整份搬家不剔）有没有新的放过」；跑前改法：戊中（就是这一格），甲中（四行实据都过，第 33 行）。

### H1-6 同一个 hunk 里挪过节的那一行照剔（`hunk_internal_relocation`）

历史：`kb/page.md` 正文节里一行「结论甲的复跑见 scripts/old-rig/x.sh」；这一次搬 `scripts/old-rig`，同时改了上面那一行与「## 历史」节标题，把那一行（只换了路径）挪到历史节标题底下。三行旧、三行新落进同一个 hunk。

许可它的那一句：同一个 hunk 里任一被删的行换算之后等于新增行就配上，不看在 hunk 里的位置：`.claude/gate.d/lib-lag-baseline.py:713`：「if after_lines[old_number - 1] == text and before_lines[old_number - 1] != text:」。

```
name=hunk_internal_relocation.diff lines=['@@ -2,3 +2,3 @@', '-结论甲', '-- 结论甲的复跑见 scripts/old-rig/x.sh', '-## 历史', '+结论甲（改）', '+## 历史（2026-10-23 起）', '+- 结论甲的复跑见 scripts/new-rig/x.sh', '']
name=hunk_internal_relocation.drop_moved_lines kept=[('kb/page.md', '结论甲（改）', 2), ('kb/page.md', '## 历史（2026-10-23 起）', 3)] problem=None
name=hunk_internal_relocation.gate_added_lines rc=0 added=['kb/page.md\t结论甲（改）', 'kb/page.md\t## 历史（2026-10-23 起）'] stderr=''
```

那一行从正文节挪进了历史节，按行判的格（门禁 doc-experiments 的 quoted-result-lines 历史节、经 `gate_added_lines` 取新增行的那些）看不到它。窄：要节标题本身也改，才进得了同一个 hunk。

四句：分辨臂（被判的中；改法五「hunk 里按次序配，被删的第 k 行只配新增的第 k 行」不中，推的：这一行在旧的一侧是第 2 行、新的一侧是第 3 行）；看得到（`settle` 手里有块内次序）；满足的是 H1「丁（按 diff 位置配对剔行）有没有新的放过」；跑前改法：丁中（就是这一格），甲中（实据过，第 36 行）。

### 「只核新加的行」守住的那几种（`stale_row_edits`）

```
name=stale_row_edits.date_and_source_rewritten rows=1 problems=[] note='没有新加的行（都在基准 HEAD 那一版的登记表里），实据不再核'
name=stale_row_edits.deleted_then_added rows=2 problems=[] note='新加的 1 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 2 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=stale_row_edits.new_prefix_rewritten rows=1 problems=['第 2 行的新前缀 tools/real-c 下有基准 HEAD 里本来就在的文件（tools/real-c/run.sh，共 1 份）：这一行不是一次搬迁', '第 2 行的旧前缀 old-b 在基准 HEAD 里没有文件：这一行不是一次搬迁'] note='新加的 1 行（基准 HEAD 那一版的登记表里没有）实据不够，见上'
name=stale_row_edits.reordered_with_fresh_row rows=2 problems=[] note='新加的 1 行（基准 HEAD 那一版的登记表里没有）有搬迁实据（第 2 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）'
```

改写日期与出处：老行不再核，写法照核，无害；删了再加同一对：认成老行，无害；改写新前缀：成了新加的行，两条实据都红，守住；同一对老行与一行新加的真搬迁换次序：两行互不相干时无害。守不住的换次序只有 H1-1 那一种：新加的行写在老行前面、新前缀落在老行的旧前缀下。

## H2 得出结论的方法

### H2-1 自证分得出第一轮的形状，分不出这一轮的（`selftest_distinguish`）

做法：冻结树里那份库自己的三项自证（`path_move_evidence_selftest`、`moved_lines_position_selftest`、`governed_moves_selftest`，共 4 个判定）在同一个进程里换着几种实现跑：被判的原样；补了改法一（最终落点占用、落回原处）；补了改法四（类按挑文件的正则分）；退回「过半」；去掉占用核对。只换模块里的函数，不改冻结树。

```
name=selftest_distinguish.judged red=[] green=4
name=selftest_distinguish.final_landing red=[] green=4
name=selftest_distinguish.half_threshold red=['搬迁登记：登记行要有搬迁实据'] green=3
name=selftest_distinguish.no_occupied_check red=['搬迁登记：登记行要有搬迁实据'] green=3
name=selftest_distinguish.class_by_reader red=[] green=4
```

退回「过半」、去掉占用都红在「搬迁登记：登记行要有搬迁实据」：第一轮打中的那几种（G1-1、G1-2 一类），自证确实钉住了。补了改法一、改法四的两种实现与被判的一样 4 项全绿，而它们在 H1-1、H1-2、H1-5 上与被判的判法相反（同一个文件第 42–44、48–50 行）。自证里那一段甚至造了一行已提交的老登记（`scripts/ancient-rig → scripts/old-rig`），没有一例把新行写在它前面；也没有来回登记、被忽略的落点、同类改名、hunk 里挪节这几种。所以「批 4 自证全过、弄坏开关红」证的是「第一轮那几种不再放过」，证不了「改完之后不放过」。

四句：分辨臂（自证分不出被判的与补了改法一、四的，这本身就是 H2 问的那件事）；看得到（样本是自证自己造的，换一种历史就有）；满足的是 H2「样本是不是只覆盖了被打中的那几种形状」；跑前改法：不涉及。

### H2-2「真登记那一行照过」是阳性对照，分不出对错（`out/probe.out`）

做法：主工作区只读现查（基准 e1b1ceb95，git 带 `GIT_OPTIONAL_LOCKS=0`），在那一行真登记上换几种实据实现各判一次。

```
name=probe.real_row_control.judged problems=0 note='新加的 1 行（基准 e1b1ceb95 那一版的登记表里没有）有搬迁实据（第 6 行 395 份里 360 份换算之后'
name=probe.real_row_control.no_occupied_check problems=0 note='新加的 1 行（基准 e1b1ceb95 那一版的登记表里没有）有搬迁实据（第 6 行 395 份里 360 份换算之后'
name=probe.real_row_control.evidence_unchecked problems=0 note='实据不核'
```

去掉占用核对、整个实据都不核，真登记那一行照样零个问题：这个对照只能证「真搬迁不被误判」，证不了「假搬迁会被拦」。批 4 与判决用它说「改了之后真登记照过」没错，可它对「判法够不够严」没有分辨力。

### H2-3 报数与判法出自同一个循环；换一条路子重算

`move_evidence` 的「395 份里 360 份换算之后相同」与判红用的是同一个循环（`.claude/gate.d/lib-lag-baseline.py:806`，同上），「726 份照剔」是 `moved_only_paths` 自己数的。我另写一条不共用它代码的路子（`git cat-file --batch` 取基准那一份，`bytes.replace` 整串换，不判边界、不跳提交引用，与工作区逐字节比）：

```
name=probe.base base=e1b1ceb95
name=probe.lib_evidence rows=1 problems=[] note='新加的 1 行（基准 e1b1ceb95 那一版的登记表里没有）有搬迁实据（第 6 行 395 份里 360 份换算之后相同；新前缀下没有基准里本来就在的文件）'
name=probe.independent_evidence subjects=395 present=395 same_after_plain_replace=362
name=probe.change_range listed=5552 lib_moved_only=725 independent_moved_only=730 problem=None
name=probe.only_lib count=0 first=[]
name=probe.only_independent count=5 first=['.claude/gate.d/fixtures/code-source-discipline.sh/data-names-red/setup.sh', '.claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-red/setup.sh', 'scripts/experiment-apparatus/e10-aging-compaction-run.sh', 'scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_unit_revision_11.sh', 'scripts/gate/data-name-lint.py']
```

被判的那份剔掉的 725 份，全在独立算法的集合里（`only_lib count=0`）；独立算法多出的 5 份，逐份看过，都是旧前缀前面挨着 `\n` 的转义或 `:-`（`${VAR:-scripts/experiments/…}`），被判的边界不换它们，方向是多判改动、不放过。395 份里相同的 360 对 362，差的两份同一原因。这一半是佐证：两条路子对上了「真目录搬迁剔了哪些」。它证不了「剔得对」：两条路子共用同一个判据（只差前缀就不算改动），主工作区今天只有一行登记，H1-1 的串、H1-4 的老行这两样在这里根本不出现。另外 725 份里落在按路径管的类里的 11 份都是原地只改路径（`out/probe.out` 里 `name=probe.lib_moved_only_governed` 那一行），与第一轮观测一致。

四句（H2-2、H2-3 合）：分辨臂（阳性对照分不出被判的、去掉占用的、不核实据的三种）；看得到（换实现跑一次就知道）；满足的是 H2「量法本身能不能分辨对错（同一个函数既当判法又当对照，算不算独立）」：不独立，换一条路子之后数对上，分辨力没变；跑前改法：不涉及。

## H3 没看过的地方

### H3-1 九处调用方拿到的基准，被老登记行永远换着

九处（`.claude/gate.d/lib-lag-baseline.py` 文件头「调它的九处」）加 history-loss、layout-checker-sync 两处，都经 `baseline_text` 取基准那一版，`.claude/gate.d/lib-lag-baseline.py:341`：「text = rewrite_moved_paths(text, moves)」用的是此刻登记表的全部行，不分这一行是不是早已进了基准。三种结局逐个对：
- 取不到基准（③）：登记表不改变 git 的退出码，照旧交问题句，判红。没打中。
- 基准里还没有这张表（②）：`paths_before_moves` 倒着走全部登记去找。老行只在旧名字被重新用起来以后才可能把一张新表倒到另一份不相干的旧文件上（推的，没造样本）。
- 表自己搬了家：真搬迁的那一行在，倒得回去，换得过来。没打中。
- 基准里有这张表（①）：这里打中。老行把基准里本来写对的新东西换走，H1-4 那一份长活豁免表提交以后一个字节没改就判「涨了」（量过，`out/attack_samples.out` 第 24 行）。`.claude/heavy-thread-exempt` 这类行里写路径的表走的是同一条代码路，同样会判（推的）。两份变更史的 history-loss 格今天没事：三语变更史里留旧路径的 6 行全是「十六进制提交号:」的写法，换算跳过它们（`git grep -cE '[0-9a-f]{7,40}:scripts/experiments/'` 与不限写法的 `git grep -cE 'scripts/experiments([^-a-zA-Z0-9_.]|$)'` 在三份上都是各 2 行，现查）；以后第 5 步留下的若是 `git log … -- <旧路径>` 这种写法，进了变更史就会被老行换走，history-loss 判「丢了」（推的）。

四句：分辨臂（被判的中；改法三不中，推的）；看得到（基准那一版登记表就在手上）；满足的是 H3「九处调用方逐个对着换算之后的基准还判得对不对」；跑前改法：甲到戊都不碰 `baseline_text` 用哪几行。

### H3-2 `scripts/gate/changed-paths.sh` 的文件头还写按次数抵

`scripts/gate/changed-paths.sh:29`：「#     登记了搬迁的，剔掉只因搬迁才变了的行（lib-lag-baseline.py 的 drop_moved_lines：整份搬了家的每一行、原地改的换算前后不同的那几行，按次数抵）。」丁改了 `drop_moved_lines`（按 diff 位置配对，`.claude/gate.d/lib-lag-baseline.py:713`，同上），`gate_added_lines` 的文件头没跟着改。照这一句读的人会以为第一轮 G1-4 那种「挪到别的节的行」今天还剔。只是句子，不放过东西。

### H3-3 `.claude/rules/path-moves.md` 开头那一句与第 5 步对不上

`.claude/rules/path-moves.md:6`：「**文件与目录的路径不在冻结范围内**：一个文件或目录搬了家、改了名，全仓指向它的地方一次改成新路径——`research/prompts/`、`research/results/`、`records/`、变更史与历史版本节、别的会话的记录都算，不留一处旧路径。」第 4 步已经改成「除第 5 步留的之外」，第 5 步写明指 git 历史的写法与抄下来的命令输出留旧路径（`.claude/rules/path-moves.md@怎么做`）；开头这一句还是「不留一处」，`records/` 与变更史正是第 5 步要留旧路径的地方。另外第 2 步与讲门禁管哪一半的那一节（这份规则第 53 行起）写的实据三条与代码同义，都按「这一行自己的新前缀」读，H1-1 的串、H1-3 的被忽略落点照字面都满足；规则也没写登记行提交以后还一直作用（H1-4）。

### 没打中：跨管辖类搬家时 `drop_moved_lines` 照旧剔行（批 4 报告点名的那一条）

代码确实照剔：戊只改了 `moved_only_paths`，`drop_moved_lines` 不看类，整份搬了家的没动的行都剔。可今天取新增行的六处调用没有一处读按路径管的类：
- `.claude/gate.d/doc-experiments.sh:227`：「if gate_added_lines "$shared_base" .claude/kb ':(glob)research/**/*.md' > "$SHARED_ADDED_LINES"; then added_lines_ok=1; fi」
- `.claude/gate.d/doc-decisions.sh:1603`：「added_out="$(gate_added_lines "$BASE" "$f")" || {」，`$f` 来自 `.claude/gate.d/doc-decisions.sh:1596`：「changed_names="$(grep "^\.claude/kb/decisions/" <<<"$all_changed" || true)"」
- `.claude/gate.d/doc-process-records.sh:509`：「if added_record_lines="$(gate_added_lines "$gate_base" 'research/prompts/*-sync.md')"; then added_record_lines_status=0; else added_record_lines_status=1; fi」
- `.claude/gate.d/doc-process-records.sh:516`：「if added_records_directory_lines="$(gate_added_lines "$gate_base" 'records/*.md')"; then added_records_directory_lines_status=0; else added_records_directory_lines_status=1; fi」
- 直接调的两处：shape 格的 diff 只取 kb，`.claude/gate.d/doc-decisions.sh:2900`：「'--src-prefix=a/', '--dst-prefix=b/', base, '--', KB)」；quoted-result-lines 格逐页取，`.claude/gate.d/doc-experiments.sh:1842`：「shown = subprocess.run(['git', '-c', 'core.quotepath=false', 'diff', '--no-renames', '--no-color', '--no-ext-diff', '-U0', base, '--', path],」（页是 kb 实验页）。

所以今天没有格因此漏看。哪天有一格按新增行判 `.claude/agents/`、`.claude/rules/` 或 `crates/<包>/src/`，这一条就成了 G3-1 的按行版本（推的）。

## 改法与各格

每个改法都**只在我的模型上量过、被攻过零轮**。「量过」贴的是 `out/attack_samples.out` 的行号（同一进程里换函数跑，不改冻结树）；「推的」是按代码推，没实现、没跑。

| 改法 | 改什么 | H1-1 串 | H1-2 来回 | H1-3 忽略的落点 | H1-4 老行 | H1-5 同类改名 | H1-6 hunk 里挪节 |
|---|---|---|---|---|---|---|---|
| 一 | `move_evidence` 对新加的行，把新前缀照后面的登记换到最终落点，落点下基准里本来就在、又不是这一行搬过去的文件算占用；照全部登记换完旧前缀下的文件都落回原处的，算没搬 | 修，量过（第 5、42 行） | 修，量过（第 13、43 行） | 不修，量过（第 18 行 `problems=[]`） | 不修，推的（提交以后没有新加的行） | 不修，量过（第 44 行与被判的同一份留下） | 无关 |
| 二 | 新加的行的落点要在索引里，只在磁盘上的（未跟踪、被忽略）不算「在」 | 不修，推的（落点是跟踪着的 B） | 不修，推的 | 修，量过（第 18 行 `untracked_landings`） | 无关 | 无关 | 无关 |
| 三 | 门禁那一侧（`baseline_text`、`moved_only_paths`、`drop_moved_lines`、`path_move_report` 换算落点）只用基准那一版登记表里还没有的行；钉值那一侧（E171 两份 `frozen_o3`、`reuse_library`）照旧用全部行，钉值是搬迁之前钉的 | 门禁那一侧修，推的（老行不用，新前缀 `old-b` 不在磁盘上）；钉值那一侧不修，推的 | 不修，推的（两行都是新加的） | 不修，推的 | 修，推的（豁免表不再被换；改回新路径的那一行照算改动） | 无关 | 无关 |
| 四 | 按路径管的类按两格挑文件的正则分（挑得中、挑不中各算一类） | 无关（第 48 行与被判的相同） | 无关（第 49 行） | 无关 | 无关 | 修，量过（第 50 行三份都留下） | 无关 |
| 五 | `moved_line_numbers` 的 `settle` 按块里的次序配：被删的第 k 行只配新增的第 k 行 | 无关 | 无关 | 无关 | 无关 | 无关 | 修，推的 |

自证那一侧：改法一、四打进去，库自己的三项自证（4 个判定）仍全绿（量过，第 41、47 行）；这正是 H2-1 说的「自证分不出」，打进去要另补样本。真登记那一行在改法一下照过：只有一行登记，最终落点就是新前缀本身，与被判的同判（推的）。改法二在「`git mv` 之后还没 `git add`」的工作区里会把真搬迁判红（推的；`git mv` 本身会暂存，手工 `mv` 才会这样）。改法三是这几个里唯一同时碰 H1-1 与 H1-4 的，也是改动最大的：它让「哪一行算老行」取决于基准，钉值那一侧必须另走一套，两侧的「只差搬迁」从此不是同一个集合（第一轮 G2 那类不一致会换个形状回来，推的）。

## 没打中的形状

| 形状 | 试了什么 | 取样 |
|---|---|---|
| 「只核新加的行」：改写老行的日期与出处、删了再加同一对、改写老行的新前缀、老行与无关的新行换次序 | 改写新前缀两条实据都红；其余三种老行不再核，换算结果不变，无害 | 样本 `stale_row_edits` 四种，量过 |
| 乙：没搬家的脚本借来回登记重钉 | 来回登记下 `paths_before_moves` 先取到这条路径自己，`.claude/hooks/lib_long_run.py:445`（同上）拦，「自己没搬家」 | 读代码，推的 |
| 丙：正着换不动的内容倒换之后两份撞成一份 | 正着换不动就是此刻内容里没有旧前缀（提交引用位置除外），倒换在新前缀上是一一的；提交引用两个方向都跳，也是一一的。撞不出 | 读代码，推的 |
| 新前缀是子模块、或穿过一个跟踪着的符号链接落进早就在的目录 | 占用只数基准树里不是子模块的文件，符号链接只算一条；可主工作区今天跟踪着的符号链接与子模块都是 0（`git ls-files -s` 按模式数，现查），要走这条路得在同一次改动里加一个，它自己在改动范围里看得见 | 读代码与现查，没造样本 |
| H3：取不到基准、表自己搬了家这两种结局 | 登记表不改 git 的退出码；真搬迁那一行倒得回去 | 读代码，推的 |
| H3：跨管辖类搬家 `drop_moved_lines` 照剔 | 代码照剔，但六处取新增行的调用都不读按路径管的类 | 逐处读调用，现查 |
| 主工作区「只差搬迁」里藏着串或老行 | 主工作区只有一行登记，没有串，也没有老行 | `out/probe.out`，现查 |

由用户决定的步骤（第 3 步）：这一题里用户决定的是「登记几行、按什么次序、搬不搬、改哪几份」。H1-1 两个对照臂把次序放开了（写在老行前面、后面、直接写到 B），只有写在前面那一种中；H1-2 对照了只写一行；H1-5 对照了不登记与跨类。判「不修」「无关」的那几格都是固定历史跑一遍，没放开扫，按「没打中」只算一次。

## 什么现象会推翻这些结论

- H1-1、H1-2、H1-3：在冻结树上起门禁 code-tooling 的 change-range-single-source 格（样本仓当根），格红在这几行登记上，就说明格里另有我没读到的核对，结论推翻。我只调了它调的 `path_move_report`（`.claude/gate.d/code-tooling.sh` 里那一格怎么调，第一轮已核过），没起格本身。
- H1-4：老行在别处另有「已进基准就不换」的跳过（我只读了 `baseline_text` 与 `path_moves`），提交以后那一次 `exempt_check` 就该绿；量出来是红（第 24 行），没找到这种跳过。
- H1-5：Claude Code 与门禁要是另有一处按目录（而不是两格的正则）认子 agent 定义与 crates 源文件、又读改动范围，那一处看得见；我只找了读 `changed_since_gate` 的两条正则（`.claude/gate.d/` 下用 `grep -E` 从 `$changed_since_gate` 里挑文件的只有这两处，按 `<<<"$changed` 搜的；别的目录没搜）。
- H1-6：要是 `git diff -U0` 在这种改动上不把三行并成一块，就不中；样本里是一块（第 37 行 `@@ -2,3 +2,3 @@`），但别的 diff 算法设置下可能拆开。
- H2-1：库另有一份我没看到的自证（三项之外）在 H1-1、H1-2、H1-5 上红，H2-1 就不成立；我数的是 `selftest()` 里起的这三项。

## 这条腿自己的限度

- 样本全在临时小仓里，调的是冻结树里被判那几份的函数与命令行（`path_move_report`、`moved_only_paths`、`drop_moved_lines`、`baseline_text`、`changed-paths.sh` 的 `gate_changed_paths` 与 `gate_added_lines`、`lib_long_run.py exempt-check`、`frozen_o3`、`reuse_library.pinned_sha256_now`）。门禁的阶段与格一个没起；H1-5「两格挑出 0 份」是拿格里那两行正则现套的。
- H1-1 落到主工作区上是推的：本批还没提交，基准里还没有那一行老行；我说的是提交以后的样子。
- 改法一、四是在同一个进程里换模块函数量的，没打进任何一份文件，没跑 `--selftest` 命令行，没跑门禁样本；改法二只写了一个独立的核对函数，没接进 `move_evidence`；改法三、五没实现。
- `probe_main_repo.py` 读主工作区此刻的样子，别的会话在改，复跑会有一行数不同（已见：5552 变 5556）。它的独立算法与被判的那份共用「只差前缀就不算改动」这个判据，只能核数，不能核判据。
- 每个样本只跑了固定的一段历史；「没打中」那几行按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 只算一次。
- 跑的时候本机有主 agent 起的 checker 档 `cargo test` 在跑（`ps` 看到的）；我的样本只起 python 与 git，都加了 `nice -n 19`，复跑那一趟经 `scripts/hosts/run-with-memory-cap.sh 4G`，没排队。

## 要主 agent 跑的命令

不起重型测试。只在主 agent 想采纳改法一或四时用得上：把改法打进一份副本以后，在那份副本里跑下面两条（副本里改过的脚本，子 agent 不许起它的 `--selftest`），看末行 ✓ 与查了几项；另把 `attack_samples.py` 对那份副本的冻结树跑一遍，看 H1-1、H1-2、H1-5 三行变红：

```
python3 .claude/gate.d/lib-lag-baseline.py --selftest
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 change-range-single-source 格; 范围=1 格; 为什么=核改法一打进去以后真登记那一行照过' bash .claude/gate.d/code-tooling.sh --check change-range-single-source <副本根>
```

## 没做什么

- 正推、辩方那几格不判；改法一个都没替主 agent 采纳；副本与模型上量的数不是入库装置上的数。
- 门禁阶段、各道 lint 都没跑（共用约束「门禁」一节）。
- 没读禁读清单里的 `research/prompts/apparatus-rename-r2-sonnet-output.md` 与 `research/prompts/apparatus-rename-r2-sonnet-model/`。批 4 工具员的报告没读，判的是代码。
- 主工作区与它的 git 仓没改任何东西：只读现查一律带 `GIT_OPTIONAL_LOCKS=0`。

## 草稿目录清理

交回之前删掉的（`du -sh`）：冻结树 `frozen` 5.0M，样本临时仓 `run1` 4.2M、`run2` 352K、`run3` 4.2M，复跑草稿 `rerun-a` 4.3M。草稿目录 `/home/fy5090/.local/share/singlefs/work/apparatus-rename-r2-attack` 里只留日志与改模板时的临时文件（共 72K），没有仓副本，也没有编译目录。
