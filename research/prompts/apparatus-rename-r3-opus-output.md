# apparatus-rename-r3 攻方腿（Opus）报告（2026-10-23）

正文 `research/prompts/_apparatus-rename-r3-body.md`，被判的 15 份照开工快照 `research/prompts/apparatus-rename-r3-snapshot/judged-sha256.txt` 现核：开工时 `sha256sum -c` 15 份都 OK（冻结树 `build_frozen.sh` 末尾同一条命令，复跑时再核一遍）。没读禁读清单里的两样。

## 复跑

模型目录 `research/prompts/apparatus-rename-r3-opus-model/`（只起 python、git、bash，不起 cargo）：

```
bash research/prompts/apparatus-rename-r3-opus-model/build_frozen.sh <仓根> <新的冻结树目录>
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/apparatus-rename-r3-opus-model/tools/rerun.sh --repo <仓根> --frozen <冻结树目录> --work <新的空目录>
```

| 文件 | sha256 |
|---|---|
| `attack_samples.py` | `795c9ffa33ab84cc28fd80fbb0f1f8f81a7975afca34c3b43ab5fe0fb46a8e84` |
| `build_frozen.sh` | `bc70662195a66c3aba491a0b616b66b0cf4b32f44bc4a15ab02ee2e5f6867fc4` |
| `tools/rerun.sh` | `74920a15c506fbcce14b775f651991f385f929912e949035a87222a12e3e8708` |
| `out/attack_samples.out`（51 行） | `436481066834f1326420826f83100a9e6ebb804a8a53fcadc6d32bf60d7bb476` |

我在草稿目录里照上面两条跑过（经 `run-with-memory-cap.sh 4G`，最后一次排队三分多钟后起跑），最后一次 `rerun.sh` 原样输出：

```
name=rerun_compare file=attack_samples.out same=yes
name=rerun_sha256 436481066834f1326420826f83100a9e6ebb804a8a53fcadc6d32bf60d7bb476
name=probe.path_moves_check（会变）   ✓ 搬迁登记表 .claude/gate.d/path-moves.tsv 1 行：写法对、出处都在；此刻生效 1 行（第 6 行），不串、有搬迁实据（第 6 行 395 份里 359 份换算之后相同；新前缀下没有基准里本来就在的文件）
name=probe.gate_base（会变） e1b1ceb95a8b8c38bcd59e2e3ae9cc921575f865
name=rerun_done same=1 differ=0
```

样本里提交的作者与时刻写死，两次独立跑 `cmp` 相同。`tools/rerun.sh` 是 `scripts/agents/three-way-rerun-template.sh` 的拷贝，只换了 `rerun_steps`；它自带的 `--selftest` 核的是模板原来那段 `rerun_steps`（拷冻结副本），换掉之后退 1，这一点不当回事。

## 各格判定一览

| 格 | 判定 | 一句话 | 证据 |
|---|---|---|---|
| J1 已定项撑不撑得住 | **打中，三处** | J1-A：换算按「前缀接在 / 后面也算」做，别的目录下的同名段一起换，没登记的目录改名与换实现被当成只差搬迁，排除表不判涨、调用方与新增行被剔，登记表那一格照绿。J1-B：生效窗口跟着 gate 基准走，有上游时基准是 merge-base、`--staged` 推完之后是 HEAD~1，「提交之后」那一行仍生效：规则写的「搬一步提交一次」第二步照红、提交之后合规地删一份搬过去的文件照红、重用旧名照红；同一棵树不带 `--staged` 判绿。J1-C：从来不生效的登记行门禁不核，钉值那一侧照用它倒换，钉住的编排器改起另一份已有的实现，E171 两处核对都认 | `out/attack_samples.out` 的 `nested_same_name.*`、`window_after_commit.*`、`pin_never_effective_row.*`；主工作区两处真实例（J1-A 节） |
| J2 没管到的清单全不全 | **打中，四处** | 上面三样都没被规则、文件头、三份表头写下（J1-C 只在批 6 草稿目录里的报告提了一句，仓里没有，`reuse_library.py:417` 还写着「不会误放」）；登记表全部行的使用方里漏了 `scripts/agents/snapshot-tree.py`；规则第 18 行与库里出路句「搬一步提交一次」与代码对不上；表头「只按路径前缀换」与代码对不上 | J2 节逐句 |
| J3 报出的数能不能核 | **部分打中** | 32、19、48、6 行、395/360 一条命令都重算出来了（数对）；140 → 21 与 12 → 132 重算不出：改前那一侧只在批 6c 草稿里的备份上算得出、两侧都随工作区漂，今天是 146 → 0、12 → 159；算法那一半：自证 32 项里调到判法的全部把基准写死成 `"HEAD"`，弄坏开关与批 6 的「模拟提交之后」都在「基准就是刚提交的那一版」上量，分不出 J1-B；钉值一侧 26 格自证没有一格造过不生效的行。用不调库的写法把今天被剔的 146 页逐页核过，都只差前缀与切换行（这一半没打中） | J3 节 |

改法一律是我提的，只在我的模型上量过或只是推的，被攻过零轮（各节「改法」表逐格标「量过」「推的」）。

## J1 已定项撑不撑得住

### J1-A 换算把别的目录下的同名段当成同一个前缀（放过）

**依据**。登记行的「前缀」在规则里是相对仓根的路径，「改动范围：与基准只差这一段前缀的文件不算这一次碰了的。」（`.claude/rules/path-moves.md:16`）；可换算的边界只要求旧前缀前面不接字母、数字、`_`、`.`、`-`：「PATH_BEFORE_BOUNDARY = r"(?<![A-Za-z0-9_.\-])"」（`.claude/gate.d/lib-lag-baseline.py:197`），文件头写的是「只按路径前缀换：旧前缀前面不接字母、数字、_、-、.（可以接 /：./、$ROOT/ 与绝对路径里的同一段都换），」（`.claude/gate.d/lib-lag-baseline.py:56`）。所以登记 `docs → manual` 之后，`tools/docs/…`、`$FROZEN/…/docs/…` 里的 `docs` 也照换；单段前缀连散文里的词也换。实据只核登记的那个前缀（仓根起的 `docs/`），管不到被顺手换掉的那些。

**历史**（`attack_samples.py` 的 `nested_same_name`，两臂同一段历史，只差登记表那一行）：基准提交 `docs/`、`tools/docs/run.sh`（`echo "本机核数 $(nproc)"`）、排除表一行 `tools/docs/run.sh:$(nproc)  # 只打核数，不是重活`、`ci.sh` 起 `tools/docs/run.sh`、`kb/note.md` 一句「复跑先读 docs 再起」。改动：`git mv docs manual` 并登记；**另外**删掉 `tools/docs/run.sh`、新写 `tools/manual/run.sh`（另一份实现：`exec make -j"$(nproc)" all`，`tools/docs → tools/manual` 没登记）；排除表那一行、`ci.sh` 那一行改成 `tools/manual/run.sh`；散文里的 `docs` 改成 `manual`。

```
name=nested_same_name.registered.path_moves_check rc=0 base=HEAD out=「✓ 搬迁登记表 .claude/gate.d/path-moves.tsv 1 行：写法对、出处都在；此刻生效 1 行（第 1 行），不串、有搬迁实据（第 1 行 2 份里 2 份换算之后相同；新前缀下没有基准里本来就在的文件）」
name=nested_same_name.registered.base_table_shown rc=0 rows=['tools/manual/run.sh:$(nproc)  # 只打核数，不是重活']
name=nested_same_name.registered.exempt_table_grown grown=[]
name=nested_same_name.registered.change_range rc=0 paths=['.claude/gate.d/path-moves.tsv', 'tools/docs/run.sh', 'tools/manual/run.sh']
name=nested_same_name.registered.added_lines rc=0 lines=[]
name=nested_same_name.control_without_row.exempt_table_grown grown=['tools/manual/run.sh:$(nproc)']
name=nested_same_name.control_without_row.added_lines rc=0 lines=['.claude/heavy-thread-exempt\ttools/manual/run.sh:$(nproc)  # 只打核数，不是重活', 'ci.sh\tbash tools/manual/run.sh', 'kb/note.md\t复跑先读 manual 再起']
```

逐步指到许可它的那一句：
1. 排除表的「基准那一版」由门禁 code-tooling 的 heavy-thread-defaults 格经 `lib-lag-baseline.py show` 取（`base_table_shown` 就是这一条命令的输出），基准里的 `tools/docs/run.sh` 被换成 `tools/manual/run.sh`；格的比法是「grown = [(number, path, anchor) for number, path, anchor in exempt_rows if (path, anchor) not in base_keys]」（`.claude/gate.d/code-tooling.sh:3345`），样本照它原样的 `EXEMPT_ROW` 正则（运行时从冻结树的 `code-tooling.sh` 里取那一行）比出 `grown=[]`：一份没登记、没交审的新实现拿到了排除，排除表不判涨。对照臂判涨。
2. `ci.sh`、`.claude/heavy-thread-exempt`、`kb/note.md` 从改动范围里消失，三行新增行全被剔（`gate_changed_paths`、`gate_added_lines` 原样起）。
3. 登记表那一格照绿：实据只核仓根起的 `docs/`。

**主工作区里这种写法真有，而且这一批已经被这样剔了**（现跑，读主工作区）：基准 `e1b1ceb95` 里旧前缀前面接着别的目录名的写法，`git grep -h -o -E '[A-Za-z0-9_.~-]+/scripts/experiments(/|\b)' e1b1ceb95 -- . ':!research/results'` 数出 195 处、46 种前缀（`| sed -E 's#/scripts/experiments.*##' | sort -u | wc -l`），其中有 `FROZEN`、`verify-copy`、`binary-of`、`.origin-repository`；`.`、`ROOT`、`root`、`REPO` 这类是指仓根的，不算问题。其中两处这一批改了、又被改动范围整份剔掉（`gate_drop_moved_only` 对这两份交回空，`git diff --stat e1b1ceb95` 两份 5 行增 5 行删）：
- 「cp -r "$FROZEN/scripts/experiment-apparatus/e170_second_run_o3_spec_executor" o3copy」（`research/prompts/e171-conclusion-r1-opus-model/tools/rerun.sh:18`），`FROZEN` 是那一轮的冻结副本「FROZEN=/tmp/claude-1000/e171-conclusion-r1/tree」（`research/prompts/e171-conclusion-r1-opus-model/tools/rerun.sh:11`），那棵树是旧布局，路径换了复跑就找不到（推的：那棵树已经不在 `/tmp` 下，没法实测）。
- 一份判决里抄下来的 `diff -u` 输出的文件头「verify-copy/scripts/experiment-apparatus/e138_per_disk_floor.rs」（`research/prompts/d16-item1-r3-main-verification.md:11`）：那是当时一份临时副本里的路径，规则第 5 步要留旧路径的正是「抄下来的命令和它的输出」，并写「⚠️ 这条要人判，别做成正则。」（`.claude/rules/path-moves.md:23`）。门禁没做成正则，却把它从改动范围里剔掉了，交审时人看不到它。

**四句**：分辨臂——分辨。被判的系统（`rewrite_moved_paths`）在做决定的那一刻看得到旧前缀前面那一段（`tools/`、`$FROZEN/`），能区分；它选了不区分。满足的是正文 J1 格「有没有新的放过」与规则第 2 步「只差这一段前缀的不算改动」字面：差的不是登记的那一段前缀。跑前条款没给改法，下面是我的。

| 改法（零轮） | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 改法甲：换算只认仓根起的前缀——前面只许是行首、空白、引号、`(`、`=`、`:`、`./`、`$ROOT/`、`${…}/` 这类「仓根」写法，前面是 `<名字>/` 的不换 | J1-A 的排除表、调用方、新增行三样；主工作区那两处会回到改动范围 | 推的（没实现没跑）；代价：`$REPO/…`、`~/code/singlefs/…` 这类写法要逐个列进「仓根写法」，列漏的多判改动 |
| 改法乙：只对「此刻在磁盘上、换算之后指得到新前缀下一份被跟踪的文件」的那一处路径换算，指不到的那一处不换 | 主工作区两处（指向冻结副本与临时副本，换完指不到） | 推的；对 J1-A 的样本不起作用（`tools/manual/run.sh` 在、被跟踪） |

### J1-B 生效窗口跟着 gate 基准走，「提交之后」那一行常常还生效（误判，另放长了 J1-A 的窗口）

**依据**。生效的判法只看基准那棵树：「return [move for move in moves if any(under_prefix(path, move.old) for path in files)」（`.claude/gate.d/lib-lag-baseline.py:342`）。gate 基准不是「上一次提交」：不带 `--staged` 时有上游就取 merge-base（「elif git rev-parse --verify -q '@{upstream}' >/dev/null 2>&1 \」，`scripts/gate/changed-paths.sh:121`）；`--staged` 时里层的基准是「staged_diff_base="${GATE_BASE:-$(diff_base "$source_repository_root")}"」（`.claude/singlefs-ai-sop/scripts/gate.sh:132`），全推出去了、gate-ok 不是祖先时落到「if parent_of_head="$(git -C "$root" rev-parse -q --verify HEAD~1 2>/dev/null)"; then」（`.claude/singlefs-ai-sop/scripts/lib.sh:180`）。规则与库的出路却按「提交了就不生效」写：「一次改动里一条路径只搬一次，要分几步搬的，搬一步提交一次。」（`.claude/rules/path-moves.md:18`），「chain_howto = "此刻生效的几行不许串：一次改动里一条路径只搬一次；要分几步搬的，搬一步提交一次，再登记下一步"」（`.claude/gate.d/lib-lag-baseline.py:919`）。主工作区此刻正是第一种：HEAD `841b3beab` 比上游多一个提交，`gate_diff_base gate` 交 `e1b1ceb95…`（`rerun.sh` 的 `probe.gate_base` 一行）；这一批提交而没推之前，那一行登记一直生效。

**历史**（`window_after_commit`）：基准 B0（`old/a.sh`、`b.sh`、`c.sh`，`ops/launch.sh` 起 `old/a.sh`）推上去；M 提交 `git mv old new`、登记 `old → new`、`ops/launch.sh` 改路径。之后四种合规的改动，用户决定的那一步（推不推、走不走 `--staged`）放开成三臂：`upstream`（M 没推，有上游）、`no_upstream`（没有上游，基准是 HEAD；批 6 的「模拟提交之后」就是这一种）、`staged_after_push`（M 推了，基准取 `lib.sh` 的 `diff_base`，即 `--staged` 交给里层的 `GATE_BASE`），`staged_after_push` 另在同一棵树上不带 `GATE_BASE` 判一次。原样输出（每行截到问题句，全文在 `out/attack_samples.out`）：

```
name=window_after_commit.delete_a_moved_file.upstream.path_moves_check rc=1 base=基准_B0 out=「✗ 搬迁登记表 .claude/gate.d/path-moves.tsv 有 1 处不对（共 1 行）： | 第 1 行：基准 8e9cb98e2 里旧前缀 old 下的 3 份，照这一行换前缀的落点上有 1 份此刻不在（new/c.sh） | → 怎么办：…
name=window_after_commit.delete_a_moved_file.no_upstream.path_moves_check rc=0 base=HEAD out=「✓ 搬迁登记表 .claude/gate.d/path-moves.tsv 1 行：写法对、出处都在；此刻生效的 0 行；不生效 1 行（第 1 行：基准 HEAD 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）」
name=window_after_commit.delete_a_moved_file.staged_after_push.path_moves_check rc=1 base=基准_B0 out=「✗ 搬迁登记表 .claude/gate.d/path-moves.tsv 有 1 处不对（共 1 行）： | 第 1 行：基准 8e9cb98e2 里旧前缀 old 下的 3 份，照这一行换前缀的落点上有 1 份此刻不在（new/c.sh） | → 怎么办：…
name=window_after_commit.delete_a_moved_file.staged_after_push.same_tree_without_staged_base rc=0 base=搬迁_M out=「✓ 搬迁登记表 .claude/gate.d/path-moves.tsv 1 行：写法对、出处都在；此刻生效的 0 行；不生效 1 行（第 1 行：基准 fcaca822c 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）」
name=window_after_commit.second_step_as_the_rule_says.upstream.path_moves_check rc=1 base=基准_B0 out=「✗ 搬迁登记表 .claude/gate.d/path-moves.tsv 有 1 处不对（共 2 行）： | 第 1 行：基准 8e9cb98e2 里旧前缀 old 下的 3 份，照这一行换前缀的落点上有 3 份此刻不在（new/a.sh、new/b.sh、new/c.sh） | → 怎么办：…
name=window_after_commit.second_step_as_the_rule_says.upstream.change_range rc=0 paths=['.claude/gate.d/path-moves.tsv', 'newer/a.sh', 'newer/b.sh', 'newer/c.sh', 'ops/launch.sh']
name=window_after_commit.second_step_as_the_rule_says.no_upstream.change_range rc=0 paths=['.claude/gate.d/path-moves.tsv']
name=window_after_commit.reuse_the_old_name.upstream.path_moves_check rc=1 base=基准_B0 out=「✗ 搬迁登记表 .claude/gate.d/path-moves.tsv 有 1 处不对（共 1 行）： | 第 1 行的旧前缀 old 还在磁盘上（搬完以后旧路径不该还在） | → 怎么办：把旧路径下剩下的东西搬完（git mv 没搬全的补上）或删掉；没搬就删掉这一行登记」
name=window_after_commit.clean_after_move_commit.upstream.change_range rc=0 paths=['.claude/gate.d/path-moves.tsv']
```

| 改动（M 提交之后） | `upstream` | `no_upstream` | `staged_after_push`（同一棵树不带 `--staged`） |
|---|---|---|---|
| 不再改 | 绿，那一行仍生效、搬过去的 3 份仍被剔 | 绿，不生效 | 绿，仍生效（不带：不生效） |
| 删掉一份搬过去的文件 `new/c.sh` | **红**「落点…不在」 | 绿 | **红**（不带：绿） |
| 照规则第 18 行搬第二步 `new → newer` | **红**，`newer/` 三份照算改动 | 绿，三份剔掉 | **红**（不带：绿） |
| 重用旧名 `old/fresh.sh` | **红**「旧前缀还在磁盘上」 | 绿 | **红**（不带：绿） |

**四句**：分辨臂——分辨（同一段改动，三种基准判法相反）。被判的系统看得到 HEAD、上游与登记行，可「这一行的搬迁有没有提交」它是拿「基准里有没有」代替的，基准不前移它就看不到提交。满足的是正文 J1 格点名的「`scripts/gate/changed-paths.sh` 基准回退的几档、`gate.sh --staged` 的临时树、提交之后下一次改动」三处，与规则第 14 行「提交进基准之后门禁不再按它换算」字面一致、与第 18 行「搬一步提交一次」不一致。方向：误判（多红）；另外窗口没收的那段时间里 J1-A 的放过一直开着（`clean_after_move_commit.upstream` 那一行：提交之后搬过去的文件照剔）。

| 改法（零轮） | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 改法丙：生效与否再看「HEAD 里新前缀下有没有文件」：HEAD 里已经有了（搬迁已提交）就不生效，基准只管换算取哪一版 | 删文件、第二步、重用旧名三格的误红；`--staged` 与不带的分歧 | 推的；副作用：提交之后、推之前，与基准比的那几张表按旧路径比，搬过去的行判涨（第二轮 H1-4 的老问题回来一半，要另核） |
| 改法丁：规则第 18 行与 `chain_howto` 改成「搬一步、等基准前移（推上去，或 gate 基准那一版里已有新前缀）再登记下一步」 | 第二步那一格的出路句 | 推的；只改说法，代码不动，删文件与重用旧名两格照红 |

### J1-C 从来不生效的登记行：门禁不核，钉值那一侧照用它倒换（放过，两侧对同一份内容判得相反）

**依据**。门禁一侧「不生效的行不核。」（`.claude/rules/path-moves.md:63`），库里同一句「不生效的行不核实据。」（`.claude/gate.d/lib-lag-baseline.py:76`）；钉值一侧「装置里按 sha256 钉的值按全部登记行倒换，提交之后照旧。」（`.claude/rules/path-moves.md:15`），读的是全部行「moves, problems = library.path_moves(repo_root)」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:117`），倒换是「return [self.library.rewrite_moved_bytes(data, [move._replace(old=move.new, new=move.old) for move in reversed(self.moves[-count:])])」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:78`），`reuse_library.pinned_sha256_now` 经「moves = frozen_o3.registered_moves(root)」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:442`）走同一条。新前缀在基准里早就有文件的那一行永远不生效，于是一行「A → B（B 是另一份已有的实现）」登记表那一格不判、门禁不换算，钉值核对却拿它把此刻内容里的 B 倒回 A 再比钉值。

**历史**（`pin_never_effective_row`）：基准里两份实现 `impl/a/run.py`、`impl/b/run.py`，`rig/orchestrator.py` 一行 `RUNNER = "impl/a/run.py"`，钉值取它此刻的 sha256。改动：编排器改起 `impl/b`，登记表加一行 `impl/a → impl/b`。对照臂不加那一行。

```
name=pin_never_effective_row.never_effective_row.path_moves_check rc=0 out=「✓ 搬迁登记表 .claude/gate.d/path-moves.tsv 1 行：写法对、出处都在；此刻生效的 0 行；不生效 1 行（第 1 行：基准 HEAD 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）」
name=pin_never_effective_row.never_effective_row.gate_change_range rc=0 orchestrator_counted=True
name=pin_never_effective_row.never_effective_row.frozen_o3 rows=[('impl/a', 'impl/b')] matches_pin=True runner_now=RUNNER = "impl/b/run.py"
name=pin_never_effective_row.never_effective_row.reuse_library_pinned_sha256_now accepted=True
name=pin_never_effective_row.control_without_row.frozen_o3 rows=[] matches_pin=False runner_now=RUNNER = "impl/b/run.py"
name=pin_never_effective_row.control_without_row.reuse_library_pinned_sha256_now accepted=False
```

门禁一侧把编排器照算改动（`orchestrator_counted=True`），钉值一侧（`frozen_o3.RegisteredMoves.contents_before_moves` 与 `reuse_library.pinned_sha256_now`，样本从冻结树原样载入）认它「只差登记过的前缀」：钉住的编排器换了实现，E171 修订 11 的 `load_repinned` 与 E170 冻结行核对这两处会照过（推的：样本只调到这两个函数，没起整台装置）。登记表那一行要的只是写法对、出处 `records/…md` 在。

**四句**：分辨臂——分辨（只差那一行）。被判的系统（钉值核对）看得到登记表与此刻内容，看不到这一行生不生效、有没有实据：判别它的东西在门禁一侧，钉值一侧不读。满足的是正文 J1 格「钉值那一侧仍按全部登记行倒换而门禁一侧只认生效行，两边对同一份内容判得一不一致」字面。批 6 报告「剩下的」一节写过这一种（「不生效的登记行不核…这一行会参与倒换」），那份报告在草稿目录、不在仓里；`reuse_library.py:417` 的注释仍写「不会误放」（J2 节）。

| 改法（零轮） | 修哪一格 | 量过 / 推的 |
|---|---|---|
| 改法戊：登记表那一格对不生效的行也核：新前缀在基准里已有文件、旧前缀在基准里没有文件的，判红「这一行不是一次搬迁」（已提交、搬迁实据曾经成立的老行另认，例：旧前缀在基准里没有、新前缀下的文件都能在 git 历史里找到改名来源） | J1-C | 推的；「老行另认」那一半要能从历史里认出搬迁，不好写 |
| 改法己：钉值一侧只用「曾经生效过」的行：拿登记行的日期或出处对到那次搬迁的提交，核那次提交确是 `old → new` 的改名 | J1-C | 推的；要读 git 历史，钉值核对从此依赖仓的历史完整 |

## J2 没管到的清单全不全

自认管不到的，此刻写在三处：规则「门禁管哪一半」（「「换了实现的不登记」门禁不判」，`.claude/rules/path-moves.md:63`；第 5 步留旧路径「⚠️ 这条要人判，别做成正则。」，`.claude/rules/path-moves.md:23`）；库文件头「搬迁登记」一段（被判的根不是仓根时不剔、提交引用不换）；批 6 报告「剩下的、没做的」四条（同一类里只换目录照剔、换实现时调用方照剔、不生效的行不核而钉值照用、G1-3 代码一半），那份报告在 `~/.local/share/singlefs/work/path-move-window/result.md`，不在仓里。下面几样哪一处都没写，或写了而与代码对不上。

| 编号 | 没写下的 | 方向 | 证据 |
|---|---|---|---|
| J2-1 | J1-A：换算连别的目录下的同名段、单段前缀当词一起换。库文件头只写到 `./`、`$ROOT/` 与绝对路径；表头写「只按路径前缀换（scripts/experiments 不换 scripts/experiments2）」（`.claude/gate.d/path-moves.tsv:4`）；规则写「只差这一段前缀」。三处读来都是「只换登记的那个仓根前缀」，与代码对不上 | 放过 | J1-A 节，样本与主工作区两处 |
| J2-2 | J1-B：生效窗口取决于 gate 基准的回退档（有上游取 merge-base、`--staged` 推完取 HEAD~1）。`changed-paths.sh` 文件头记了两条路「判的窗口不同（C528）」，没连到搬迁登记；规则第 18 行与库第 919 行的出路「搬一步提交一次」在有上游、没推的仓里走不通 | 误判（另放长 J1-A 的窗口） | J1-B 节表格 |
| J2-3 | J1-C：从来不生效的行门禁不核、钉值照用。只在批 6 草稿里的报告写过；仓里的说法相反：「# 不会误放：换回去之后逐字节等于钉住的那一版，此刻与它就只差登记过的前缀」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:417`）——「登记过」不等于「核过」，这句在甲路之下不成立 | 放过 | J1-C 节 |
| J2-4 | 用全部登记行倒换的不只装置：`scripts/agents/snapshot-tree.py` 判决前倒推开工快照时读「moves, problems = library.path_moves(cwd=repository)」（`scripts/agents/snapshot-tree.py:129`，这份文件在我跑的过程中被别的会话改过，行号从 119 移到 129），倒换照全部行。库文件头「调它的各处」那一张单子自称「现查 `grep -rln lib-lag-baseline .claude/gate.d .claude/hooks scripts`」（`.claude/gate.d/lib-lag-baseline.py:15`），这条 grep 列得出 `scripts/agents/snapshot-tree.py`，单子里没有；规则第 15 行只说「装置里按 sha256 钉的值」。`grep -c snapshot-tree` 在库、规则、登记表三份里都是 0 | 清单不全（影响推的：它只在照原样倒推对不上时才换，J1-C 那种行也会参与） | 现查 |
| J2-5 | 按路径管的类只列五样：「GOVERNED_DIRECTORIES = (".claude/agents/", ".claude/rules/", ".claude/hooks/")」（`.claude/gate.d/lib-lag-baseline.py:189`）加两份定义文件与 `crates/<包>/src/`。别的按路径起作用的地方整份搬进去照剔：`.claude/gate.d/`（「共享门禁（gate.sh）把 gate.d 下每个 *.sh 都当阶段跑，」，`scripts/gate/changed-paths.sh:42`）、`.claude/singlefs-ai-sop/rules/`（共用约束要求整份读的规则目录）。样本 `governed_class_gaps`：单文件登记、文件名不变，搬进这两处的都从改动范围里消失，搬进 `.claude/rules/` 的照算 | 放过 | `name=governed_class_gaps.change_range rc=0 paths=['.claude/gate.d/path-moves.tsv', '.claude/rules/local.md']`；批 6 自己列的「同一类里只换目录」是另一种（同一类之内），这一种是类本身没列 |

逐句对照（规则、定义、三份表头与此刻代码）：

| 句子 | 对不对得上 |
|---|---|
| 规则第 14 行「一行此刻生效，当且仅当 gate 基准那一版里旧前缀下有文件、新前缀下没有文件；提交进基准之后门禁不再按它换算。」（`.claude/rules/path-moves.md:14`） | 对得上 `effective_among`；但「提交进基准」与小标题「登记行只在搬迁提交之前对门禁生效」读来像「提交」，J1-B 三臂里两臂提交之后照生效 |
| 规则第 16 行「改动范围：与基准只差这一段前缀的文件不算这一次碰了的。」（`.claude/rules/path-moves.md:16`） | 对不上：换算的不止「这一段」（J2-1） |
| 规则第 18 行「一次改动里一条路径只搬一次，要分几步搬的，搬一步提交一次。」（`.claude/rules/path-moves.md:18`） | 对不上：有上游没推、`--staged` 推完之后，第二步照红（J1-B） |
| 规则第 63 行「此刻生效的行不串、有搬迁实据…；不生效的行不核。」 | 对得上代码；没写不核的行钉值照用（J2-3） |
| 定义两处长活豁免表那一句（`.claude/agent-common.md:70` 与 `.claude/main-agent.md:82`，逐字相同的一段「此刻生效（gate 基准那一版里旧前缀下有文件、新前缀下没有文件；提交进基准之后不再生效）的搬迁照登记换算」） | 对得上 `lib_long_run.exempt_main`；「这一行的脚本自己搬了家」那一条用的是仓根前缀（`paths_before_moves` 按路径段倒推），J1-A 的同名段在这张表上换算之后指不到基准里的脚本，照判涨——这张表不受 J1-A，排除表受（两张「只缩不涨」的表对同一种改动判法不同；推的，按 `paths_before_moves` 与 `moved_script_unchanged` 读代码，长活豁免表这一半没造样本） |
| 表头 `.claude/heavy-thread-exempt:3`、`.claude/long-run-live-hosts-exempt:8` | 批 6 报告说没改的两句此刻已带「此刻生效」，与规则一致；`.claude/gate.d/path-moves.tsv:4`「只按路径前缀换」对不上（J2-1） |

改法：J2-1、J2-2、J2-3 的说法照 J1 各节的改法定了之后补进规则「门禁管哪一半」与库文件头；J2-4 在库文件头单子里加一行、规则第 15 行写成「装置与判决工具」（推的，零轮）；J2-5 把 `.claude/gate.d/` 顶层与 `.claude/singlefs-ai-sop/rules/` 加进按路径管的类（推的，零轮；代价：搬进来的以后都照算改动，与 `.claude/rules/` 同判）。

## J3 报出的数能不能核

在主工作区现跑（2026-10-23，只读；主工作区随别的会话在变）：

| 报的数 | 一条命令 | 现跑原样 | 判定 |
|---|---|---|---|
| 32 项 | `python3 .claude/gate.d/lib-lag-baseline.py --selftest` | 退 0，末行「  ✓ lib-lag-baseline.py 自证：32 项都过」；数出自 `len(results)`（库第 1832 行），不是写死的 | 数对 |
| 19 项 | `bash scripts/gate/changed-paths.sh --selftest` | 退 0，末行「  ✓ 共用库 scripts/gate/changed-paths.sh 自证通过（19 项：…」 | 数对 |
| 48 项 | `python3 -B scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py --selftest` | 退 0，末行「自证 48 项，红 0」；48 = 26 行「✓ 自证 …」+ 22 行「✓ 弄坏开关 … → 红在 …」（`grep -c` 分别数出），自证格与开关核对混在一个数里 | 数对，口径要写明 |
| 6 行 | `python3 .claude/hooks/lib_long_run.py exempt-check . --base HEAD` | 退 0，「✓ .claude/long-run-live-hosts-exempt 6 行：…与基准那一版比没涨（其中 6 行照 .claude/gate.d/path-moves.tsv 登记的搬迁换算之后…」；换成门禁真用的基准 `--base e1b1ceb95a8b8c38bcd59e2e3ae9cc921575f865`（`gate_diff_base gate` 现交）同一句 | 数对；报的是 `--base HEAD`，不是门禁那一格用的 gate 基准，此刻两者判得一样 |
| 395 份里 360 份 | `python3 .claude/gate.d/lib-lag-baseline.py path-moves-check .` | 退 0，「第 6 行 395 份里 360 份换算之后相同」（`j3-pmc.txt`，与 `--base HEAD` 同句） | 数对（那一刻）；几十分钟后 `rerun.sh` 再读同一句已是「395 份里 359 份」（上面复跑一节）：工作区在别的会话手里变，这个数不是钉得住的量。不调库的写法（`git ls-tree`、`git cat-file`、`bytes.replace` 整串换、不判边界）现算：`name=independent.base_files count=395 landing_present=395 same_bytes=290 same_after_plain_replace=361`。差的 1 份方向是库更严，与第二轮两条腿的结论同形，不新 |
| 140 → 21、只改切换行 12 → 132、119 页 | 没有一条命令：照批 6c 的 `run-real.sh` 抽 decision-links 格的内嵌 python 现跑，改前那一侧要批 6c 草稿目录里的备份 `doc-experiments.sh.orig`（sha256 `cf77a5deeccbf039df979405bcdb782b6831aaa63e51f5ea541b652c09267fe4`，不在仓里、不在 git 里） | 改动范围 5004 条；改前「没回看 146」「只改语言切换行的改动 12 份」；改后退 0、没回看 0、「只改语言切换行的改动 159 份」 | **重算不出**：改前一侧只在仓外备份上算得出，两侧都随工作区漂（140 → 146、21 → 0、132 → 159）；批 6c 自己报的「切换行 +120 比 119 多 1」也就没法再查 |

**算法分不分得出对错**：

1. 「140 → 21」比的是同一格新旧两版（同第二轮 H2 判过的那种互比）。我用不调那一格、不调库的写法把今天改前一侧点名的 146 页逐页核：基准那一版整串换前缀之后与此刻比（`difflib`），146 页都只差顶上四行（语言切换行与紧跟的空行），`name=independent.top_lines_only count=146`。这一半没打中：今天被剔的那些页确实只差前缀与切换行。这条独立写法与库共用「整串替换即相同」的定义，只算佐证。
2. 库的自证调到判法的地方，基准一律写死成 `"HEAD"`：`baseline_text("HEAD"` 16 处、`moved_only_paths("HEAD"` 11 处、`path_move_report(…, "HEAD")` 8 处、`effective_moves("HEAD", …)` 5 处，帮手「return [what for what, _howto in path_move_report(root, "HEAD")[1]]」（`.claude/gate.d/lib-lag-baseline.py:1374`）；自证那一段（第 1143 行起）里 `upstream`、`GATE_BASE`、`remote`、`merge-base`、`HEAD~` 零命中。「提交进基准之后不再生效」只在「基准 = 刚提交的那一版」上证过，弄坏开关 `window-ignored` 也只在这种基准上红。J1-B 的三臂里 `no_upstream` 一臂与自证同形（绿），另两臂红：自证与开关分不出 J1-B。
3. 批 6 的「模拟提交之后」（`effective=[]`、长活豁免表照绿、E171 两处钉值照过）是在一个没有上游的新临时仓里、基准取 HEAD 量的（报告原话「整份提交成基准」）。主工作区有上游（`git rev-parse --abbrev-ref '@{upstream}'` 交 `origin/master`，HEAD 比 merge-base `e1b1ceb95` 多一个提交），这一批提交之后、推之前，门禁的基准仍是 `e1b1ceb95`，那一行仍生效——J1-B `clean_after_move_commit.upstream` 一行就是这种局面。那份模拟的结论只对「推上去之后、不带 `--staged`」成立。
4. 钉值一侧的自证（`reuse_library.py --selftest` 的 `pin-moves`、`pin-moves-real-files`、`frozen-o3-moves`、`pin-moves-partial`、`frozen-o3-reverse-moves` 五格与五个开关）在第 1885 行到文件尾这一段里「生效」「effective」「早就」「已有」零命中：没有一格造过不生效的登记行，分不出 J1-C。
5. 「真登记那一行照过」作对照：那一行在 `--base HEAD` 与 gate 基准下都照过（上表），可 J1-A 的样本里实据那一格同样照绿而门禁剔了没搬的东西；登记表那一格绿只说明仓根前缀下的那些文件搬齐了，说明不了换算剔掉的都是搬迁。

**四句**：分辨臂——第 2、3、4 条分辨（同一个判法在自证的形状上绿、在 J1 的形状上红或放过）。被判的东西（自证与模拟）当时看得到基准的取法，却把它写死了。满足的是正文 J3 格「自证项与弄坏开关是不是只覆盖了自己造的形状，真登记那一行照过算不算对照」字面。改法：自证与 `code-tooling.sh` 的 change-range-single-source 样本加一种「仓有上游、搬迁已提交没推」与一种「`GATE_BASE=HEAD~1`」的形状，钉值自证加一格不生效的登记行（推的，零轮）。

## 没打中的形状

| 试过的形状 | 结果 | 取样范围 |
|---|---|---|
| 前两轮站住的乙：没搬家的脚本改了正文再重钉长活豁免表 | 没打中（读代码）：`moved_script_unchanged` 先比「取到的那条路径是不是它自己」，J1-A 的同名段在这张表上换不回基准里的脚本，照判涨 | 读代码，没造样本 |
| 前两轮站住的丙：钉值倒换前要求此刻内容正着换一遍不动 | 没打中：J1-C 绕开了它（编排器里已经没有旧前缀），没有破它 | `pin_never_effective_row` 一个样本 |
| 新增行同一块第 k 删配第 k 加：同一块里删两行加两行，第 1 行换前缀、第 2 行换成另一份实现的路径 | 没打中：第 2 行照留（位置配对按换算之后逐字等不等判） | 草稿里手跑一次，没进模型 |
| 搬到符号链接上：新前缀是指向别处目录的符号链接 | 没打中：落点那几份不在索引里、`ls-files --others` 只列链接本身，判「没被 git 跟踪」 | 读代码（`move_evidence` 第 966 行起那一段），没造样本 |
| 单文件登记把文件名一起改 | 没打中：`exact_rename` 要文件名不变，照算改动（批 6 已有自证格） | 读代码 |
| 批 6c 的 decision-links 换算剔错页 | 没打中：今天被剔的 146 页逐页只差前缀与切换行（J3 第 1 条） | 146 页全量 |
| 用户动作放开：J1-B 里推不推、走不走 `--staged` | 三臂都跑了；J1-A 里另登记 `tools/docs → tools/manual` 一行不改变结论（两行不串，实据照过，换实现按规格只报不判，是第二轮辩方已判过的那一种），没进模型 | 推的 |

## 这条腿自己的限度

- 样本是临时 git 仓，不是真仓；三处打中都只起了被判的函数与命令行（`path-moves-check`、`show`、`gate_changed_paths`、`gate_added_lines`、`lib.sh` 的 `diff_base`、`frozen_o3.RegisteredMoves`、`reuse_library.pinned_sha256_now`），没起整道门禁阶段（子 agent 不跑门禁阶段），排除表那一格的比法是照它原样的正则与那一行比法在样本里重做的。整道阶段上的复核要主 agent 跑。
- J1-B 的 `staged_after_push` 臂取的是 `lib.sh` 的 `diff_base` 交出的基准（`--staged` 外层交给里层的那一个），没起 `gate.sh --staged` 本身与它的临时 worktree；临时 worktree 里索引是暂存树，「被 git 跟踪」那一项与工作区同判（推的）。
- J1-A 主工作区两处的后果（冻结副本里找不到新路径）是推的：那棵冻结树已经不在。
- J1-C 只调到两个钉值函数，没起 E171 修订 11 的整台装置，也没核 E170 冻结执行器那一处（同一个 `contents_before_moves`，推的）。
- J3 的 140 → 21 这一格，改前一侧取自别的子 agent 草稿目录里的备份（只读拷进我的草稿目录）；主工作区在别的会话手里变，今天的数不是批 6c 那一刻的数。
- 我的改法甲到己都没实现，被攻过零轮。

## 没做什么

- 没跑重型测试、门禁阶段与各类 lint；没起 `gate.sh`、`gate-staged.sh`；没改主工作区里任何文件（只写了这份报告与模型目录）；没做 git 写操作。
- 没读禁读清单里的同一轮辩方报告与模型。
- 交回前删掉草稿目录里的临时仓与两棵冻结树（大小见下一节）。

## 草稿目录

`/home/fy5090/.local/share/singlefs/work/apparatus-rename-r3-opus`：草稿版模型、样本输出、J3 的现跑输出（`j3-*.txt`、`j3-decision-links/`）。冻结树与样本临时仓交回前删，删前大小写在交回里。
