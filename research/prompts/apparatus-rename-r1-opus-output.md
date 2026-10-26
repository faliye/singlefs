# apparatus-rename-r1 攻方腿（Opus）报告（2026-10-23）

轮名 apparatus-rename-r1，第一轮；攻击面是正文「三、格」G1–G4。被判的那 14 份取自主工作区，sha256 与开工快照 `research/prompts/apparatus-rename-r1-snapshot/judged-sha256.txt` 逐个对上（14 个 OK，`build_frozen.sh` 末尾那一步打的）。崩溃枚举那一套（第 3b 步）这一题用不上：这一题没有盘、没有崩溃状态，样本全是临时 git 仓加 python。

## 复跑

模型目录 `research/prompts/apparatus-rename-r1-opus-model/`。复跑两步（工作目录随意；全是 python 与 git，没有编译出来的代码）：

```
bash research/prompts/apparatus-rename-r1-opus-model/build_frozen.sh <仓根> <新目录>/frozen
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/apparatus-rename-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <新目录>/frozen --work <新目录>/rerun
```

rerun 跑完逐份比 `out/` 下 8 份，打 `name=rerun_compare file=… same=yes|no` 与末行 `same=… differ=…`。我这边第二次复跑末行（草稿目录路径换成了〔草稿目录〕）：`name=rerun_done … same=7 differ=1`，不同的是 `probe.out`：它只读主工作区，别的会话在加文件，改动范围条数从 5423 变成 5432，「只差搬迁」的条数都是 726。`attack_samples.out` 两趟逐字节相同。我自己那两趟是直接 `nice -n 19 python3` 起的：经 `run-with-memory-cap.sh 4G` 起的那一趟在排队（「slice 已占 40.6 GiB，这一条要 4.0 GiB…总上限 40.1 GiB」），等了 120 秒还在排，我用 `proc.py stop` 停了自己那条 admit 进程。这一题跑的不是编译出来的代码，不受内存包装那条约束。

| 文件 | sha256 |
|---|---|
| apply_fixes.py | cc2b6456a8aa2ae5033470230d1bf67864fcbc1ce37c272ccb66c10766c38ea0 |
| attack_samples.py | bc603e699440ddcd0676a2f8efb993bab0cd2b79fa9142d33633c7cd4364fda7 |
| build_frozen.sh | c290f724451ff18ea3fb0651c4932a140dfd53587d96ab33c24235dd26a6fc8d |
| e171_real_check.py | 210379e0da22977e5c27ade9d8dd7f936e6a589b4c0e86fa7620b1813a74b5d0 |
| judged-sha256.txt | 07dfc5d7b5536411fac988e384b6cff84b4590829052528aabc00d4151891e02 |
| out/apply_fixes.out | 10d3a0219b29ec5c5234062f957666752e42b8c17b5a91f2a35be73c067b6b48 |
| out/attack_samples-fixes.out | 0aed3fc4e9b56a5e33fccebeefa2d3306c14847f7a55a4573c5310f6a0ef6ecf |
| out/attack_samples.out | a198f69a4f35f392fb07c532b0ae09cbce2fd2a5b0f5aed14c1dcb8533710a88 |
| out/e171-tree-fixes.out | aa3a7bd6df54068486f95746f2731416219454dd7a1d65e97e5cb84185c7f983 |
| out/e171-tree.out | 13e3704734f3282e6d5cd21213fc225a79f000c46a12e0673f86357f2ec8dd4a |
| out/exempt-tree-fixes.out | a96ce9f93c479475f2043aa029207845a5ebc222dc3ac779cd0305929e949282 |
| out/exempt-tree.out | a96ce9f93c479475f2043aa029207845a5ebc222dc3ac779cd0305929e949282 |
| out/probe.out | 365f2efa588e9054aa79f7f4f7a135ae1ffa652df90b1321e969194c8af1e638 |
| out/repro_command.out | dbd13ec5463d43e3ac323bfbaff6c682f703a006c7d264a1baf8742969e0fc60 |
| probe_real_repo.py | 6d4e4e0536ec38de4189268dfa845ecc2920141f81dab5b2733b86314572bade |
| repro_command_count.sh | b4cb8d4247d2ce1a7b7832ef9e07310a2a200a2104f9a843109aaf267055ec9f |
| tools/rerun.sh | 5850e98ed56f6c919a3324d13c1c2984868a1c6224fa3379bdee3361673bcfb4 |

## 各格判定一览

| 格 | 判定 | 打中的东西（样本名，在 `out/attack_samples.out` 里 `name=` 开头） |
|---|---|---|
| G1 误放 | **打中，4 处** | G1-1 加两行登记，让两个都还在的目录 A→B 通过登记检查，A 换成 B 的命令与重钉的豁免脚本都被放过（`launder_parent_excuse`）；G1-2 删掉 A、登记 A→B（B 是早就在的另一份实现），引用处被放过（`launder_deleted_directory`）；G1-3 指 git 历史的写法里只有「十六进制提交号:」不换，`v0:路径`、`git log -- 路径` 换了也算只差搬迁，换完命令的结果变了（`history_references`）；G1-4 新增行按次数抵、不按位置抵，挪到别的节的行不算新增（`lines_relocated`） |
| G2 读法一致 | **打中，3 处** | G2-1 钉值一侧倒着换（frozen_o3、reuse_library）认「只换了一半」的文件，共用库正着换判它不同（`reverse_partial`）；G2-2 两行串起来的搬迁、钉在两次之间：frozen_o3 判不同，reuse_library 与共用库判同（`chain_midway`）；G2-3 path_move_problems 的豁免比它自己的文件头、规则写的都宽（与 G1-1 同一处代码） |
| G3 剔除的副作用 | **打中，1 处** | 单文件登记把文件搬进 `.claude/agents/`、`crates/<包>/src/`，agent-def-adversarial-review、crates-adversarial-review 挑文件的正则在改动范围里挑不出它（`move_into_governed`）。invariant-anchors 那一处没打中：不登记也一样（见「没打中的形状」） |
| G4 定义与规则的句子 | **打中，5 处** | G4-1 规则写「旧前缀不在」，代码另有豁免；G4-2 规则第 2 步只列两类读者，漏了改动范围与新增行；G4-3 定义里「不算涨」那一句没写出代码的几个条件，而且登记表这一行不用问用户，绕开了「重钉交主 agent 问用户」；G4-4 第 1 步的四种搜法打不到拆开写的路径，主工作区里就有一处漏改：门禁 doc-experiments 的 repro-command 格，照它的循环算会有 83 页误判红（`out/repro_command.out`）；G4-5 第 4、6 步写「不留一处旧路径」「0 命中」，与库里有意留「提交号:旧路径」、与指 git 历史的句子换了就成假话两样对不上。「正文只写四样」那一问没打中 |

改法（第 5 步）：我提了 6 个，每个都**只在我的模型上量过、被攻过零轮**。逐格量没量过的表在文末「改法与各格」。

## G1 误放

### G1-1 加两行登记，让「两个都还在的目录 A→B」通过检查（`launder_parent_excuse`）

历史：基准提交里有 `tools/real-a/run.sh`（打「走的是 A」）、`tools/real-b/run.sh`（打「走的是 B」）、`ops/launch.sh`（`bash tools/real-a/run.sh`），还有豁免表登记的 `ops/live.sh`（文件头按现算写，正文 `worker=tools/real-a/run.sh`，表里钉着它的 sha256）。之后改 `ops/launch.sh` 与 `ops/live.sh` 里的 `tools/real-a` 为 `tools/real-b`，豁免表换成新 sha256，登记表写两行：`tools/real-a → tools/real-b`、`not-on-disk → tools`。两个目录都没动。

每一步许可它的那一句：
- 两行登记都过检查：`.claude/gate.d/lib-lag-baseline.py:639`：「moved_back = any(following.new == move.old or following.new.startswith(move.old + "/") or move.old.startswith(following.new + "/") for following in later)」。第三个分支在「后面某一行的新前缀是旧前缀的上一层」时也算「搬回来了」，后面那一行写 `→ tools` 就行；下一行 `.claude/gate.d/lib-lag-baseline.py:640`：「if os.path.lexists(os.path.join(root, move.old)) and not moved_back:」就不报旧前缀还在。第二行自己的检查：旧前缀 `not-on-disk` 不在、新前缀 `tools` 在、出处在。
- 只差搬迁就剔掉：`.claude/gate.d/lib-lag-baseline.py:550`：「return data == before or data == rewrite_moved_bytes(before, moves)」。
- 重钉放行：`.claude/hooks/lib_long_run.py:449`：「if lag_baseline.rewrite_moved_bytes(data, moves) == current:」。这一行原本的出路是 `.claude/hooks/lib_long_run.py:337`：「确要保留改动并重钉的，交主 agent 问用户」。

原样输出（`out/attack_samples.out`，对照臂只登记第一行）：

```
name=launder_parent_excuse.control_one_row rows=1 problems=['第 2 行的旧前缀 tools/real-a 还在磁盘上（搬完以后旧路径不该还在）']
name=launder_parent_excuse.path_move_problems rows=2 problems=[]
name=launder_parent_excuse.real_change diff=['-bash tools/real-a/run.sh', '+bash tools/real-b/run.sh', ''] launch_now='走的是 B'
name=launder_parent_excuse.moved_only_paths changed=['.claude/gate.d/path-moves.tsv', '.claude/long-run-live-hosts-exempt', 'ops/launch.sh', 'ops/live.sh'] moved_only=['ops/launch.sh', 'ops/live.sh'] problem=None
name=launder_parent_excuse.drop_moved_lines kept=[('.claude/long-run-live-hosts-exempt', 'ops/live.sh  29d1d8c273c570a6b8865f521ac959628c1ce4f4160e1f5f7e7a3bb28f06d6ee  # 样本：文件头按现算写、不能改，只缩不涨'), ('.claude/gate.d/path-moves.tsv', '# 样本登记表'), ('.claude/gate.d/path-moves.tsv', 'tools/real-a\ttools/real-b\t2026-10-23\trecords/move.md'), ('.claude/gate.d/path-moves.tsv', 'not-on-disk\ttools\t2026-10-23\trecords/move.md')] problem=None
name=launder_parent_excuse.exempt_check rc=0 out='✓ .claude/long-run-live-hosts-exempt 1 行：每行的脚本都在、登记的 sha256 都对得上，文件头都得靠现算读法才齐（按登记值读不齐）；与基准那一版比没涨（其中 1 行照 .claude/gate.d/path-moves.tsv 登记的搬迁换算之后与基准那一版逐字节相同：ops/live.sh）'
```

结果：门禁 code-tooling 的 change-range-single-source 格（它调的就是 path_move_problems）判绿；改动范围里看不到 `ops/launch.sh`、`ops/live.sh`；新增行一行不剩；豁免表重钉不报涨。`ops/launch.sh` 现在起的是另一份实现（`launch_now='走的是 B'`），`ops/live.sh` 从来没搬过家。

四句：
1. 分不分辨臂：分辨。它分得开「今天的 path_move_problems」与「去掉第三个分支的那一版」（改法一，下面量过）；今天的代码中，改法一不中。
2. 系统看不看得到判别它的东西：看得到。path_move_problems 手里有磁盘（`tools/real-a` 在）和整张登记表；第二行把 `tools` 当成搬进去的新前缀，是代码自己把它算成了豁免。
3. 满足的是判据字面的哪一句：G1 那一格的「登记表谁都能加一行，判法只核新前缀在、旧前缀不在、出处在——拿一行登记把别的改动…洗成「只差搬迁」做不做得到」。答：做得到，要两行。
4. 跑前条款的改法还中不中：正文没写跑前的改法。我的改法一在这一格上：`path_move_problems` 报出「第 2 行的旧前缀 tools/real-a 还在磁盘上」（量过，见改法表）；`moved_only_paths` 照样剔（它不判磁盘），整轮门禁靠 code-tooling 那一格红；改法三让 `ops/live.sh` 那一行的重钉照旧红（量过）。

### G1-2 删掉 A、登记 A→B，B 是早就在的另一份实现（`launder_deleted_directory`）

历史：基准里 `tools/real-a/run.sh`（打 A）与 `tools/real-b/run.sh`（打 B，另一份实现）都在，`ops/launch.sh` 起 A。之后 `git rm -r tools/real-a`，`ops/launch.sh` 改成起 B，登记一行 `tools/real-a → tools/real-b`。

```
name=launder_deleted_directory.path_move_problems rows=1 problems=[]
name=launder_deleted_directory.moved_only_paths changed=['.claude/gate.d/path-moves.tsv', 'ops/launch.sh', 'tools/real-a/run.sh'] moved_only=['ops/launch.sh'] problem=None
```

许可它的那几句：登记检查只看新前缀在、旧前缀不在、出处在，`.claude/gate.d/lib-lag-baseline.py:53`：「登记表指不指得到（新前缀照后面的登记换算之后要在磁盘上、旧前缀不该还在磁盘上、出处文件要在）」。它不核「旧前缀下的文件真的换算之后出现在新前缀下」。删掉的 `tools/real-a/run.sh` 自己还留在改动范围里（它与 `tools/real-b/run.sh` 换算之后不同），被放过的是改引用的那份。

四句：分辨（今天的代码中、打了改法二的不中，推的）；看得到（基准树里 `tools/real-a/run.sh` 与此刻 `tools/real-b/run.sh` 内容不同，`moved_only_paths` 本来就在取这两份）；满足的是 G1 格里那一句的例子「把一条命令里的目录 A 换成另一个已存在的目录 B」；改法一在这一格上照样中（量过：`out/attack_samples-fixes.out` 里这两行与 `out/attack_samples.out` 逐字相同；这里没有第二行，第三个分支用不上），改法二不中（推的）。

### G1-3 指 git 历史的写法，只有「十六进制提交号:」后面的不换（`history_references`）

历史：第一个提交放 `scripts/experiments/run.sh`、`README.md`，打标签 `v0`；基准提交加三份笔记，各写一条复现命令：`git show v0:scripts/experiments/run.sh`（标签引用）、`git log --format=%s -- scripts/experiments/README.md`（历史路径）、`git show <第一个提交的 8 位号>:scripts/experiments/run.sh`（十六进制引用）。之后 `git mv scripts/experiments scripts/experiment-apparatus`，照 `.claude/rules/path-moves.md:6`：「全仓指向它的地方一次改成新路径」，三份笔记里的旧路径全换，登记一行。

```
name=history_references.moved_only_paths registry_rows=1 registry_problems=[] changed=['.claude/gate.d/path-moves.tsv', 'notes-hex.md', 'notes-log.md', 'notes-tag.md', 'scripts/experiment-apparatus/README.md', 'scripts/experiment-apparatus/run.sh'] moved_only=['notes-log.md', 'notes-tag.md', 'scripts/experiment-apparatus/README.md', 'scripts/experiment-apparatus/run.sh'] problem=None
name=history_references.after_commit.tag_old command='git show v0:scripts/experiments/run.sh' rc=0 out='echo 第一版'
name=history_references.after_commit.tag_new command='git show v0:scripts/experiment-apparatus/run.sh' rc=128 out="fatal: path 'scripts/experiment-apparatus/run.sh' exists on disk, but not in 'v0'"
name=history_references.after_commit.log_old command='git log --format=%s -- scripts/experiments/README.md' rc=0 out='move | first'
name=history_references.after_commit.log_new command='git log --format=%s -- scripts/experiment-apparatus/README.md' rc=0 out='move'
```

`notes-tag.md`、`notes-log.md` 被剔出改动范围，可换完之后那两条命令一条报错，另一条少了一行；只有 `notes-hex.md` 照算改动。许可它的那一句：`.claude/gate.d/lib-lag-baseline.py:133`：「COMMIT_REFERENCE_BEFORE = re.compile(r"(?<![0-9A-Za-z_])[0-9a-fA-F]{7,40}:(?:\./)?$")」，只认十六进制；文件头 `.claude/gate.d/lib-lag-baseline.py:42`：「git show 31e47115:scripts/experiments/… 指的是那个提交里的文件，仓里有意留旧路径」。标签、`HEAD~1:`、`origin/master:`、`"$base":`、`git log/blame -- <路径>` 指的也是历史，这几种都不认。

主工作区现查（只读，`probe_real_repo.py`，基准 e1b1ceb95，即上游的 merge-base）：「只差搬迁」的 726 份里，找「git log/diff/blame/rev-list/show … -- …新前缀」或「非十六进制引用:新前缀」，只命中 1 处：`name=real_repo.history_bound_line path=research/prompts/e161-r2-prereg.md line=495 snippet='git diff --cached --stat -- $F crates/mutations.tsv scripts/experiment-apparatus'`。它比的是暂存区，不是历史，换了没错。所以这一处今天在仓里是**潜在的**，这一批没有踩中（正则会不会漏写法见限度一节）。仓里另有两处已经成了假话的，都不在「只差搬迁」的集合里，归 G4-5 讲。

四句：分辨（今天的代码中；改法五「凡是指历史的写法都不换」不中，推的）；看得到（这几种写法就在被换算的文本里，代码只认十六进制那一种）；满足的是 G1 那一句「有没有一种真改动经换算变成逐字节相同」；正文没给改法，改法一到四在这一格上都照样中（量过：`attack_samples-fixes.out` 里这五行逐字相同）。

### G1-4 新增行按次数抵、不按位置抵（`lines_relocated`）

许可它的那一句：`.claude/gate.d/lib-lag-baseline.py:580`：「新增行逐条去抵同文的可抵行，抵上一条剔一条（按次数抵，不按位置）」；整份搬了家的，每一行都可抵，`.claude/gate.d/lib-lag-baseline.py:605`：「budgets[path] = collections.Counter(new for old, new in zip(before_lines, after_lines) if relocated or new != old)」。

历史：原地那一页 `kb/page.md` 历史节有一行「- 旧说法见 scripts/experiments/x.sh」，这一次**留着那一行不改**，另在正文节加一行换了路径的同文行；整份搬家那一页 `docs/old/page.md → docs/new/page.md`，搬完把历史节那一行挪进正文、正文那一行挪进历史。

```
name=lines_relocated registry_rows=2 registry_problems=[] changed=['.claude/gate.d/path-moves.tsv', 'docs/new/page.md', 'docs/old/page.md', 'kb/page.md', 'scripts/experiment-apparatus/x.sh'] moved_only=['scripts/experiment-apparatus/x.sh'] problem=None
name=lines_relocated.added_lines added=[('docs/new/page.md', '## 正文'), ('docs/new/page.md', '- 旧说法见 scripts/experiment-apparatus/x.sh'), ('docs/new/page.md', '## 历史'), ('docs/new/page.md', '结论乙'), ('kb/page.md', '- 旧说法见 scripts/experiment-apparatus/x.sh')]
name=lines_relocated.after_drop kept=[] problem=None
```

两页都留在改动范围里（不只差搬迁），可它们的新增行一行不剩。按行判的格就看不到这几行：门禁 doc-decisions 的 shape、doc-experiments 的 quoted-result-lines（历史节）、经 `gate_added_lines` 取新增行的那些格。`kb/page.md` 正文节那一行是这一次真加进去的（旧行原样留着），`docs/new/page.md` 里有一行从正文挪进了历史节。

四句：分辨（今天的代码中；改法六「按 diff 的位置配对，只剔与被删的旧行同一处换掉的」不中，推的）；看得到（`git diff -U0` 里有这一行的位置，它旁边没有被删的那一行；`drop_moved_lines` 只拿到行文，看不到位置）；满足的是 G1 那一句「有没有一种真改动经换算变成逐字节相同」的按行版本：这一行与基准里某一行换算之后逐字节相同；改法一到四在这一格上照样中（量过：fixes 那一趟逐字相同）。

## G2 读法一致

几处的边界都是共用库那一份（frozen_o3 经 `path_moves_library` 载入共用库，倒换调的是同一个 `rewrite_moved_bytes`；lib_long_run 也一样），**边界规则与提交引用的跳法是同一套**，这一半没打中。不一致的是「往哪个方向换」与「串起来几次怎么倒」。

### G2-1 倒着换认「只换了一半」，正着换不认（`reverse_partial`）

钉住的那一版有两处 `scripts/experiments/…`；此刻有两种：两处都换了（full），只换了一处（partial，另一处还是旧前缀，指向一个已经不在的目录）。

```
name=reverse_partial.full frozen_o3_same=True reuse_library_pin_accepted=True lib_forward_equal=True lib_moved_only=True problem=None
name=reverse_partial.partial frozen_o3_same=True reuse_library_pin_accepted=True lib_forward_equal=False lib_moved_only=False problem=None
```

同一份 partial 内容：frozen_o3 与 reuse_library 判「与钉住的相同」，共用库（改动范围、豁免表、新增行那一侧）判「不同」。原因是倒着换不是一一对应的：此刻内容里残留的旧前缀倒换时不动，与「本来是新前缀、倒回旧前缀」的那一处结果一样，两份不同的文件倒出同一份。这正是 G2 格问的「倒换时此刻内容里本来就有的…串会被换走，会不会让两份不同的文件换出同一份」，只是被吞掉的是残留的旧前缀。许可它的那几句：`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:68`：「backwards = [move._replace(old=move.new, new=move.old) for move in reversed(self.moves)]」；`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:444`：「for count in range(1, len(moves.moves) + 1):」。两份文件头的「不会误放」写的是 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:16`：「不会误放：换回去只动登记过的新前缀，换回去之后逐字节等于钉住的那一版，此刻与那一版就只差登记过的那几次前缀替换」——按字面没说错（只换一部分也是「只差前缀替换」），可共用库对同一句话的读法是「每一处都换了」，两边的「只差搬迁」不是同一个集合。

四句：分辨（今天的代码中；改法四「倒换之前要求此刻的内容正着换一遍不动」不中，量过：`out/attack_samples-fixes.out` 里 `name=reverse_partial.partial frozen_o3_same=False reuse_library_pin_accepted=False lib_forward_equal=False lib_moved_only=False problem=None`）；看得到（此刻的内容就在手上，正着换一遍就知道有没有残留）；满足的是 G2「哪一处与别处不一致会让同一份内容一处判同、一处判不同」；改法四不动主工作区的 E171（量过：`out/e171-tree-fixes.out` 冻结执行器 `freeze_differences=[]`，两个钉值都 `accepted=True`，与没打改法的那一份相同）。

### G2-2 两行串起来、钉在两次之间（`chain_midway`）

登记 `tools/a → tools/b`、`tools/b → tools/c`；钉住的那一版写的是 `tools/b/run.sh`（钉在两次搬迁之间），此刻写 `tools/c/run.sh`。

```
name=chain_midway registry_rows=2 registry_problems=[] frozen_o3_reverse='x=tools/a/run.sh\n' frozen_o3_same=False reuse_library_pin_accepted=True lib_moved_only=True problem=None
```

frozen_o3 只整串倒到底（`tools/c → tools/b → tools/a`），判不同；reuse_library 逐次多倒一次、每一步都比，判同；共用库正着换（`tools/b → tools/c`）也判同。reuse_library 自己写了为什么要逐次：`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:440`：「钉值可能是在两次搬迁之间钉的（按现内容重钉过的那几个），所以从最近一次起逐次多倒换一次」；frozen_o3 的两份（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:68` 与 `scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py:70`）都只整串倒。这一处判得更严、不会误放；主工作区今天只有一行登记，碰不到它。它会踩中的地方：`freeze-path-only-changes.tsv` 里「现在」那一列是在某次搬迁之后记的（今天两份表各有 2 行内容行），而后又有第二次搬迁。

四句：分辨（frozen_o3 与 reuse_library 两种倒法在这一格上一判同、一判不同）；看得到（登记表的次序就在手上）；满足的是 G2「多次搬迁串起来的次序是不是同一套」；改法四在这一格上照样中（量过：fixes 那一趟这一行逐字相同），要的是改法「frozen_o3 照 reuse_library 逐次倒」（推的）。

### G2-3 path_move_problems 的豁免比它自己的文件头宽

函数文档 `.claude/gate.d/lib-lag-baseline.py:618`：「新前缀照它后面的登记换算之后要在磁盘上；旧前缀不该还在磁盘上（后面的登记又搬进这个前缀的除外）；出处文件要在。」「搬进这个前缀」对应 639 行前两个分支；第三个分支（后面某一行的新前缀是旧前缀的上一层）是这句话外面多出来的。G1-1 就是从这里过的。四句同 G1-1。

### 没打中的那一半（G2）

- 边界：共用库正着换、倒着换、lib_long_run、frozen_o3 两份都调同一个 `rewrite_moved_paths`，前后边界（`.claude/gate.d/lib-lag-baseline.py:130`：「PATH_BEFORE_BOUNDARY = r"(?<![A-Za-z0-9_.\-])"」与下一行）与提交引用的跳法逐字是同一份。改名那份草稿脚本另换了 `\n`、`:-`、拆开写的几种，共用库不认，结果是那些文件「多判改动」，不放过；这一点正文第二节已经写了，不重复。
- `moved_path`（按路径换）只认 `前缀` 与 `前缀/…`，文本换算另认「前缀后接别的字符」：路径不会带那些字符，没造出一处判不一样的。
- changed-paths.sh 判「登记表有没有内容行」用 grep，共用库用 `content_rows`；只有 `\r` 的行、只有空白的行、缩进的 `#` 行，两边都当没有：没造出分歧。

## G3 改动范围剔除的副作用

### G3-1 搬进按路径管的地方，挑文件的格看不见（`move_into_governed`）

历史：基准里 `drafts/agents/reviewer.md`（一份子 agent 定义草稿）、`research/proto/fast_path.rs`；之后 `git mv` 进 `.claude/agents/reviewer.md`、`crates/core/src/fast_path.rs`。登记那一臂写两行单文件登记（前缀可以是一份文件：`prefix_problem` 只拒空、空白、开头 `/` `./`、结尾 `/`、空段与 `.` `..` 段）；对照臂不登记。改动范围用冻结树里的 `scripts/gate/changed-paths.sh` 的 `gate_changed_paths HEAD untracked` 现取，再用两个格自己的正则挑：`.claude/gate.d/doc-process-records.sh:1744`：「definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed_since_gate" || true)"」（agent-def-adversarial-review），`.claude/gate.d/doc-process-records.sh:880`：「all_sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed_since_gate" || true)"」（crates-adversarial-review）。

```
name=move_into_governed.control registry_rows=0 registry_problems=[] gate_changed_paths_rc=0 changed=['.claude/agents/reviewer.md', 'crates/core/src/fast_path.rs'] agent_definitions=['.claude/agents/reviewer.md'] crates_sources=['crates/core/src/fast_path.rs'] stderr=''
name=move_into_governed.registered registry_rows=2 registry_problems=[] gate_changed_paths_rc=0 changed=['.claude/gate.d/path-moves.tsv'] agent_definitions=[] crates_sources=[] stderr=''
```

一份新生效的子 agent 定义、一份进了产品包 `src/` 的源文件，在改动范围里都没了，两格挑出 0 份（agent-def-adversarial-review 那一格这时退 77「无对象可判」，那一句在 `.claude/gate.d/doc-process-records.sh` 第 1746 行，推的：没起那一格）。对这两格要紧的不是内容，是「这条路径上的东西是新来的」：同一份字节放在 `drafts/` 下不生效，放进 `.claude/agents/` 就生效。`crates/` 那份要真编进去还得在 `lib.rs` 加 `mod`，那一处照算改动，可新文件本身不进 crates-adversarial-review 点名的文件。

四句：分辨（今天的代码中；改法七「整份搬了家、新旧路径落在不同管辖类的，不剔」不中，推的）；看得到（`moved_only_paths` 手里有旧路径与新路径）；满足的是 G3「依赖改动范围的格…有没有哪一格本该看见这类文件（新路径要核存在、搬家本身要同步记录）而现在看不见」；改法一到四在这一格上照样中（量过：fixes 那一趟这两行逐字相同）。

### 主工作区里这一层今天剔了什么（只读现查）

`probe` 那一步，基准 e1b1ceb95：改动范围 5423 份，其中「只差搬迁」726 份。按目录数，前几项是 `scripts/experiment-apparatus` 360、`research/prompts` 161、`.claude/gate.d` 105。落在按路径管的几处的是：`.claude/agents/mutation-triage.md`、`.claude/hooks/host-resources-reminder.sh`、`.claude/rules/format-evolution.md`、`.claude/rules/implementation-first.md`，以及 crates 下 7 份（checker-tier 的 4 份 bin、`singlefs-core/src/inode_tree.rs`、`singlefs-harness/src/new_pool_file_creation_regions.rs`、`singlefs-unit-graph/src/heavy_threads.rs`）。这些都是原地只改了路径，不是搬进来的，剔掉符合这一层的本意。正文第二节说只差路径的 crates 文件有 8 份；我按目录数，crates 下共 8 份（checker-tier 4、harness 2、core 1、unit-graph 1），其中 7 份在 `src/` 下，harness 另一份不在 `src/` 下（没再查是哪一份）。这一段只是观测，不算打中。

## G4 定义与规则的句子

### G4-1 规则写「旧前缀不在」，代码另有豁免

`.claude/rules/path-moves.md:56`：「写得对不对（写法、新前缀在、旧前缀不在、出处在）由门禁 code-tooling 的 change-range-single-source 格判」。代码在 `.claude/gate.d/lib-lag-baseline.py:639` 另有三种豁免，规则一个字没提；其中第三种连函数自己的文档（618 行「后面的登记又搬进这个前缀的除外」）都没写。证据就是 G1-1 那两行输出：只登记 A→B 时报「旧前缀 tools/real-a 还在磁盘上」，再加一行 `not-on-disk → tools` 就 `problems=[]`。改法：要么删掉第三个分支（改法一，量过），要么规则照代码写全。

### G4-2 第 2 步只列了两类读者

`.claude/rules/path-moves.md:13`：「门禁里与基准比只缩不涨的表、装置里按 sha256 钉的值，照它换算，只差这一段前缀的不算改动。」登记表还管另外三处，一处没写：(1) 改动范围剔掉只差搬迁的文件，`scripts/gate/changed-paths.sh` 第 19 行；它下游是全部按改动范围挑文件的格，包括 G3-1 那两格。(2) 新增行剔掉只因搬迁才变的行（`gate_added_lines`、doc-decisions 的 shape、doc-experiments 的 quoted-result-lines 历史节）。(3) 门禁 doc-decisions 的 entry-added 按文件剔。照第 2 步那一句去登记的人，不知道这一行也会让文件从三方点名、批次范围、新增行判形状这些格里消失。另外同一句开头是「整个目录搬家或改名的」，而登记表文件头 `.claude/gate.d/path-moves.tsv:1` 写「目录或文件搬了家、改了名」，代码也收单文件那一行（G3-1 就是单文件登记）：规则、表头、代码三处的射程不一样。

### G4-3 定义里「不算涨」那一句

`.claude/agent-common.md:70` 与 `.claude/main-agent.md:82` 同一句：「登记在 `.claude/gate.d/path-moves.tsv` 的目录搬迁照登记换算，基准那一版换算之后与此刻逐字节相同的不算涨；」。与 `.claude/hooks/lib_long_run.py` 的 `moved_script_unchanged` 逐条比：
- 代码多要求一条，句子没写：基准里取到的那份脚本，sha256 要在基准那一版表里登记过（`.claude/hooks/lib_long_run.py:442`：「if hashlib.sha256(data).hexdigest() not in base_sha256s:」）。这一条比句子严，不放过东西，只是句子没写全。
- 「按搬迁之前的路径取基准」句子没写；「基准那一版」指表还是指脚本，读不出来。
- 句子说「目录搬迁」，代码不看这一行的脚本搬没搬：G1-1 里的 `ops/live.sh` 从来没动过，照样「不算涨」。改法三让「自己没搬家的」照旧红，量过；主工作区今天那 6 行都是搬了家的，打了改法三照样绿（量过：`out/exempt-tree-fixes.out` 与 `out/exempt-tree.out` sha256 相同）。
- 句子不管登记那一行是哪来的。同一句前面写着豁免表「只缩不涨」，lib_long_run 的出路是「确要保留改动并重钉的，交主 agent 问用户」；而登记表里加一行既不用问用户，也不比基准只缩不涨，G1-1 加两行就把重钉放过去了。要不要写成「登记表新加的行同样交主 agent 问用户」，或者让门禁核「登记行要有搬迁实据」（改法二），归主 agent 判。

### G4-4 第 1 步的四种搜法打不到拆开写的路径，主工作区里已经漏了一处

`.claude/rules/path-moves.md:12`：「整条路径、`kb/` 起的写法、光文件名、不带 `.md` 的名字，四种各搜一次」。这四种是给搬 kb 文件写的；目录改名时，正则交替、拆成几段的写法都搜不到。主工作区现在就有一处：`.claude/gate.d/doc-experiments.sh:1650`：「REPRO_RE='cargo run --release --bin |scripts/(experiments/replay|devices/vm-bench)\.sh'」，旁边那行注释已经改成新路径（`.claude/gate.d/doc-experiments.sh:1641`：「或者点名复跑入口 scripts/experiment-apparatus/replay.sh」），正则没改。照 repro-command 格的循环原样算（`repro_command_count.sh`，只读，没起门禁阶段）：

```
name=repro_command regex_line_present=1 pages_pointing_products=144 bad_with_current_regex=83 bad_with_new_prefix_regex=0
```

kb 实验页已经全换成新路径；这一格照今天的正则会判 83 页「点了产物却没写怎么跑」，把正则里的 `experiments/replay` 换成 `experiment-apparatus/replay` 就是 0 页。这一处是误判红，不放过东西，可它说明第 1 步那份清单在目录改名上不够。G4 格问的「要不要写清整串替换打不到的写法要另搜」，我的答是要，而且清单至少得加：正则里的交替与分组（`scripts/(experiments/…|…)`）、拆成几段的写法（`"scripts", "experiments"`、`os.path.join(…, "experiments")`）、相对路径（`../experiments`）、rsync 与 glob 的排除写法。我搜过后三种，主工作区里没有残留，只有 kb/experiments 这些别的目录（命令见「没打中的形状」）。这一格的判定是按格的代码照抄循环算的，门禁本身没跑（子 agent 不跑门禁阶段）。

### G4-5「不留一处旧路径」与留旧路径的两种情形对不上

`.claude/rules/path-moves.md:6`：「全仓指向它的地方一次改成新路径」，`.claude/rules/path-moves.md:15`：「改完按同一份清单再搜一遍，必须 0 命中」；能留旧路径的只有 `.claude/rules/path-moves.md:16`：「说搬迁这件事本身的那一行保留旧路径」。可是：
- 库有意把 `<提交号>:<旧路径>` 这种写法留着不换（共用库 lib-lag-baseline.py 文件头第 42 行，正文说 kb 变更史三语里留了 6 处），规则不知道这一条。照规则把它们也换了，`git show` 就指空，库又会把这些文件算成改动。
- 规则没要求留指 git 历史、或抄命令输出的句子。主工作区现有一处已经成了假话：`records/2026-10-13-里程碑四预开发.md:838`：「只有 `scripts/experiment-apparatus/e173_admission_ceiling_bad_ring_slot_dispositions.rs` 一族」，说的是 `git log --all -- '**/e173_*'` 的结果。现跑 `git log --all --name-only --format= -- '**/e173_*' | sort -u` 列出的是 `scripts/experiments/e173_admission_ceiling_bad_ring_slot_dispositions.rs` 等 4 条，新路径一条都没有。这一行在暂存区里就是新路径，HEAD 里没有这一行，是谁、在哪一步换的我查不到。同一类还有 `research/prompts/subagent-landing-audit-2026-10-22/writers.md` 第 316 行，那一行配着输出「→ 1b6f98657 2026-10-14」，命令里的路径现在是新的：去掉 `-S<整行>` 那一段现跑 `git log --since=2026-09-20 --format='%h %cs' -- scripts/experiment-apparatus/README.md`，什么都不打；换回旧路径打出 767668191、04ceaa904、1b6f98657 三行。新路径在 git 历史里还没有任何提交，带不带 `-S` 都打不出 1b6f98657（推的）。
- 术语改名那一节已经有判据：`.claude/rules/path-moves.md:38`：「把旧名换成新名之后那句话还是不是真话」。路径搬迁的第 5 步没有同一条。

改法（推的）：第 5 步改成与术语那一节同一个判据：换了就成假话的留旧路径，例子写明「提交号:路径」、`git log/show/blame` 指历史的路径、抄下来的命令输出；第 4 步的「0 命中」改成「除第 5 步留的那几处外 0 命中」。

### 没打中：「正文只写四样」

第 7 条那几句（`.claude/agent-common.md:70` 加的分句、`.claude/rules/path-moves.md:13` 加的分句、`.claude/rules/path-moves.md:56` 加的句子）写的都是做什么、谁判，没有理由、日期与经过；按 `.claude/singlefs-ai-sop/rules/rules-discipline.md@1. 正文只写四样` 那张表逐格对过，没找到越界的字。

## 改法与各格

下面每个改法都**只在我的模型上量过、被攻过零轮**。「量过」是在冻结树的副本上打了 `apply_fixes.py`（改法一、三、四一起打）后再跑一趟样本，贴的是 `out/attack_samples-fixes.out` 原样；「推的」是按代码推，没实现、没跑。三个改法一起打，哪一格是哪一个改法修的，是按代码推着分的（各改法动的函数互不调用）。

| 改法 | 改什么 | G1-1 | G1-2 | G1-3 | G1-4 | G2-1 | G2-2 | G3-1 |
|---|---|---|---|---|---|---|---|---|
| 一 | `path_move_problems` 去掉第三个分支（后面某行新前缀是旧前缀的上一层） | 登记检查修了，量过（`problems=['第 2 行的旧前缀 tools/real-a 还在磁盘上（搬完以后旧路径不该还在）']`）；改动范围照剔，量过（`moved_only=['ops/launch.sh', 'ops/live.sh']` 不变），整轮靠 code-tooling 那一格红 | 不修，量过 | 不修，量过 | 不修，量过 | 无关 | 不修，量过 | 不修，量过 |
| 二 | 登记行要有搬迁实据：基准里旧前缀下的每份文件，此刻在新前缀下有换算之后相同的一份；新前缀下不许有基准里本来就在的文件 | 修（推的） | 修（推的） | 不修（推的） | 不修（推的） | 无关 | 无关 | 不修（推的） |
| 三 | `moved_script_unchanged` 只在这一行的脚本自己搬了家时认搬迁 | 豁免表那一半修了，量过（`rc=1`，红行末尾「ops/live.sh 自己没搬家（基准里同一路径就有），不按搬迁认」）；主工作区 6 行照绿，量过 | 无关 | 无关 | 无关 | 无关 | 无关 | 无关 |
| 四 | 倒换认钉值之前，要求此刻的内容正着换一遍不动（frozen_o3 与 reuse_library） | 无关 | 无关 | 无关 | 无关 | 修，量过（`frozen_o3_same=False reuse_library_pin_accepted=False`）；主工作区 E171 照过，量过 | 不修，量过 | 无关 |
| 五 | 提交引用的跳法扩到指历史的写法（`<任何 rev>:路径`、`git log/blame/show … -- 路径`），或者在 path-moves.md 第 5 步写明这类留旧路径 | 无关 | 无关 | 修（推的；正则认全 rev 写法不容易，规则那一半更稳） | 无关 | 无关 | 无关 | 无关 |
| 六 | `drop_moved_lines` 按 diff 位置配对：只剔与同一处被删的旧行换算之后相同的那一行 | 无关 | 无关 | 无关 | 修（推的） | 无关 | 无关 | 无关 |
| 七 | `moved_only_paths` 对新旧路径落在不同管辖类的整份搬家不剔（`.claude/agents/`、`.claude/rules/`、`.claude/hooks/`、`crates/<包>/src/`） | 无关 | 无关 | 无关 | 无关 | 无关 | 无关 | 修（推的） |

G2-2 要的是另一个改法：「frozen_o3 照 reuse_library 逐次倒」（推的，没实现）。G4 那几格要的是改句子，各节里写了，都是推的。

改法一、三、四打进去之后，共用库与 E171 的自证没跑：副本里改过的脚本不许在副本里起 `--selftest`（共用约束「执行前拒绝的写法」重型测试闸那一条），见文末「要主 agent 跑的命令」。

## 没打中的形状

| 形状 | 试了什么 | 取样 |
|---|---|---|
| 正着换「两份不同的基准换出同一份」 | 想过：基准是定的，正着换是一个确定的函数，误放只能出在「换出来恰好等于此刻」；有这个可能的就是 G1-1 到 G1-4 那几种，没另找到 | 推的 |
| 前缀边界放过别的目录 | `scripts/experiments2`、`scripts/experiments.md`、`scripts/experiments-old`、`_v2` 都不换（文件头与边界正则）；`/` 前缀的外树路径（`~/…/frozen/scripts/experiments/…`）会换，可规则第 6 步本来就要求它们一起改，按字面不算误放 | 读代码，没造样本 |
| 「十六进制:」误认 | 看起来像十六进制的词（`deadbeef:`）后面的路径会不换，那一份就多算改动，不放过 | 推的 |
| 改动范围的两侧（基准到工作区、HEAD 到暂存区） | 想造「暂存了真改动、工作区改回只差搬迁」：`moved_only` 两侧都要只差搬迁，暂存区那一份与 HEAD 不同就照样比，造不出 | 读代码（`.claude/gate.d/lib-lag-baseline.py:550` 上下） |
| 豁免表「涨」：把一个本来不豁免的脚本借登记换进表 | 基准里 `tools/b/live.sh` 在、表里是 `tools/a/live.sh`，删 a、登记 a→b、表行换成 b：`baseline_file_bytes` 先取到基准里 b 自己那一份，sha256 不在表里，判红 | 读代码（`.claude/hooks/lib_long_run.py:442`） |
| invariant-anchors（G3） | 整份搬家的不变量表不进「新增文件」，那一格就对它自己跑 `git diff -U0 <基准> -- <新路径>`，整份都是 + 行。可是不登记的对照臂一样（`out/attack_samples.out` 里 `invariant_file_moved.control` 的 `added_files=[]`：git 自己的改名识别就把它当 R、不当 A），不是这一层带来的 | 样本 `invariant_file_moved` 两臂 |
| 主工作区里「只差搬迁」的文件含指历史的写法 | 正则见 `probe_real_repo.py`，726 份里命中 1 处，是 `git diff --cached`，不是历史 | 726 份全扫 |
| 拆开写的旧路径残留（G4-4 之外） | `git grep` 搜了 `"experiments"`、`'experiments'`、`../experiments`、`experiments/replay`、`--exclude … experiments` 几种，除 `doc-experiments.sh:1650` 外命中的都是 `kb/experiments` 这些别的目录 | 排除 research、records、kb 之外全仓 |

由用户决定的步骤（第 3 步）：这一题里的用户动作是「登记几行、改哪几份、搬不搬」。判「不中」的只有上表几行与改法表里「量过」的「不修」，都是固定历史跑一遍，没放开扫；G1-1 到 G3-1 是打中的，不受这一条管。

## 这条腿自己的限度

- 样本都是临时小仓，调的是冻结树里被判那几份的函数与命令行（`moved_only_paths`、`drop_moved_lines`、`path_move_problems`、`lib_long_run.py exempt-check`、`changed-paths.sh` 的 `gate_changed_paths`、frozen_o3、reuse_library）。门禁的阶段与格一个都没起：G3-1 里两格「挑出 0 份」是拿格里那一行正则现套的，那一格最后退几是推的；G4-4 的 83 页是照 repro-command 格的循环抄出来算的。
- G1-1 说「整轮门禁看不出来」，依据是 change-range-single-source 格调的就是 `path_move_problems`（`.claude/gate.d/code-tooling.sh` 第 1603 行），我没起那一格。
- G1-3 在主工作区「今天没踩中」靠的是一条正则（`probe_real_repo.py` 里的 `history`），它认不全指历史的写法（比如命令拆在两行、写在表格里的）。
- 改法一、三、四是一起打进副本的，各格归哪一个改法修，是按「各改法动的函数互不调用」推的；三个改法打进去以后，共用库、lib_long_run、E171 自己的自证没跑。
- 每个样本只跑了固定的一段历史（两趟逐字节相同，但这是同一种取样）。「没打中」那几行按 `.claude/rules/three-way-inference.md@一条腿只抽一次样不算一次观测——否定结论尤其不算` 只算一次，不能拿来撑「没有」。
- 主工作区在变：`probe.out` 第二趟改动范围的条数变了；`repro_command.out`、`exempt-*.out`、`e171-*.out` 都读主工作区此刻的样子，主工作区一变，复跑就可能对不上。

## 要主 agent 跑的命令

不起重型测试。只在主 agent 想采纳改法一、三、四时才用得上：把 `apply_fixes.py` 打进一份副本之后，在那份副本里跑下面三条（副本里改过的脚本，子 agent 不许起它的 `--selftest`）。看末行 ✓、看查了几项：

```
python3 .claude/gate.d/lib-lag-baseline.py --selftest
python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py --selftest
SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 long-run-header-exempt、change-range-single-source 两格; 范围=2 格; 为什么=核改法一、三打进去之后样本照判' bash .claude/gate.d/code-tooling.sh --check long-run-header-exempt,change-range-single-source <副本根>
```

## 没做什么

- 正推、辩方那几格不判；改法没有一个替主 agent 采纳；副本上量的数不是入库装置上的数。
- 门禁阶段、各道 lint 都没跑（共用约束「门禁」一节）。
- 没读禁读清单里的 `research/prompts/apparatus-rename-r1-sonnet-output.md` 与 `research/prompts/apparatus-rename-r1-sonnet-model/`；`git grep` 时用 `':!research/prompts/apparatus-rename-r1-sonnet*'` 把它们排除了。
- 三个工具员的报告（正文第二节最后一条）没读，判的是代码。
- 草稿目录里建的冻结树与临时仓在交回前删掉，删前的大小记在下面一节。

## 草稿目录清理

建过又删掉的（`du -sh`，删在交回之前）：冻结树 `frozen` 4.7M、`frozen2` 4.7M（`build_frozen.sh` 建的那一棵）、打了改法的副本 `frozen-fix` 4.7M，样本临时仓 `run1` 3.0M、`runfix` 3.0M，两趟复跑的草稿 `rerun-a` 16M、`rerun-b` 16M。草稿目录里只留脚本草稿与日志（`draft/`、`*.out`、`*.log`），没有仓副本，也没有编译目录。
