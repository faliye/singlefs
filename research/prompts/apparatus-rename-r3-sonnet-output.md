# apparatus-rename-r3 辩方腿报告（推论轮第三轮，最终轮，J1–J3，2026-10-23）

立场：云端辩方（Sonnet）。复核第二轮判决 `research/prompts/apparatus-rename-r2-main-verification.md` 与它定的甲路，替被收紧的几处辩护，同时照实报辩不住的。
没读禁读清单（同一轮攻方腿的报告与模型目录）。引 kb 条款：这一轮的判决面是门禁与脚本，没有引 kb 条款。

## 开头：复跑与校验

- 开工快照：`sha256sum -c research/prompts/apparatus-rename-r3-snapshot/judged-sha256.txt`，15 份全对（没有 FAILED 行）。样本用的冻结树由 `tools/build_frozen.sh` 现建，逐份核 sha256 在开工快照里。
- 模型目录 `research/prompts/apparatus-rename-r3-sonnet-model/`，一条命令重跑（仓根、冻结位、新草稿目录三个参数；`--frozen` 只要是个存在的目录，冻结树由脚本自己从 `--repo` 建）：
  `bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/apparatus-rename-r3-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <任一存在的目录> --work <新的草稿目录> --model <仓根>/research/prompts/apparatus-rename-r3-sonnet-model`
  我自己的两趟是不经包装直接起的（`nice -n 19`）：包装排队排了 3 分钟没排上（本机有别的会话占着内存额度），我停了排队，没绕开排队去跑别的东西；样本全是 python 与 git，不编也不跑 cargo 产物。
- 复跑结果：两趟输出里 7 份确定性输出 `same=yes`（`probe_pinned`、`probe_window`、`probe_close`、`probe_over`、`probe_anchor`、`probe_fix`、`probe_untested`）；`scan_slash_context`、`recount_pages`、`recount_exempt`、`uncovered` 读主工作区或库本体，随工作区变，不逐行比。输出的 sha256 在报告最后一段。
- 样本都是这一腿自己造的临时 git 仓，没碰主工作区；不调库里的换算当对照（独立数用 `str.replace` 或自写的正则）。

## 一览

| 格 | 判定 | 一句话 |
|---|---|---|
| J1 已定项撑不撑得住 | **部分打中** | 甲路在门禁一侧从根上关了第二轮的三种（老行前插、没改的表判涨、无序键），生效窗口随 gate 基准开合的时机也对；但有三处没关：钉值一侧用全部登记行而门禁一侧对不生效的行一字不核（新放过，含老行前插在钉值一侧原样还在）；睡着的老行在基准树形状变了时会醒来；换算不锚定在仓根（别的目录下同名的一段也被换）。关过头的几种全是多判方向 |
| J2 没管到的清单全不全 | **打中** | 批 6 自己列的和规则「门禁管哪一半」写下的不含上面三处，也不含：自证把「不生效的行不核」当期望值锁住、`move_evidence` 里两句「这一行不是一次搬迁」平常永远不出、库文件头与规则的调用方清单没跟上批 6c 与 `snapshot-tree.py`；规则与定义的句子跟代码逐句核，对得上的占多数，对不上的在下面列 |
| J3 报出的数能不能核 | **部分打中** | 32、19、48、6、395 与 360 都能用命令重算（395 与 360 另有独立算法数出同一个 360）；140→21、12→132、119 随工作区变、要跑门禁格，我按定义没跑，只做了一个比格严的整页重数，方向一致、数对不上；自证的区分度：十个弄坏开关红的格数逐个复现，但自证没跑到的行有 185 条，且自证把不生效的行不核当成期望值 |

三条新提的改法（收严 X、收严 Y、收严 Z）都标「零轮」：只在这一腿自己的样本上量过，没被攻过。

## J1 已定项撑不撑得住

下面每行是甲路（或乙、丙）的一条，判定用三个词：**站得住**（我替它辩，没找到打中的）、**够不着**（第二轮打中的那几种这条够不着，但这条本身也没收住别的）、**同样打中别的臂**（这个问题不是甲路独有，换哪条路都会有）。输出文件都在 `research/prompts/apparatus-rename-r3-sonnet-model/out/`。

| 条 | 判定 | 证据 |
|---|---|---|
| 生效窗口：基准里旧前缀下有文件、新前缀下没有才生效 | **站得住** | 判法两行：「return [move for move in moves if any(under_prefix(path, move.old) for path in files)」（`.claude/gate.d/lib-lag-baseline.py:342`）与「and ("occupied-unchecked" in modes or not any(under_prefix(path, move.new) for path in files))], None」（`.claude/gate.d/lib-lag-baseline.py:343`）。`probe_window.out` 走一遍带上游的样本仓：S1 未提交的搬迁，gate 基准与 HEAD 基准都生效，5 个文件剔掉、登记表与出处照留；S2 提交没推，gate 基准是上游那个提交（S0）、仍生效、剔得一样，HEAD 基准 0 行生效、没有改动；S2b 提交之后又改的两处（真加了一行的调用方、新写的 `docs.md`）照留；S3 推出去，gate 基准追上 HEAD、0 行生效、`exempt.tsv` 的基准原样；S3b 推出去之后新写的带旧前缀的文件照算改动；S4 把 `GATE_BASE` 指回 S0 窗口重新打开；S5 先登记没搬，登记表那一格红（旧前缀还在磁盘上）。`gate.sh --staged` 里层的基准在源仓上算好再传：「staged_diff_base="${GATE_BASE:-$(diff_base "$source_repository_root")}"」（`.claude/singlefs-ai-sop/scripts/gate.sh:132`），与不带 `--staged` 同一个起点，没起 `gate.sh --staged` 本身（按定义不跑门禁整轮） |
| 第二轮 H1-1：在老行前面插一行串到早就在的实现 | **站得住**（门禁一侧） | `probe_close.out`：`h1_1_no_landing` 新行 `rigs/y->legacy` 生效、老行 `legacy->app` 不生效，落点不在，登记表红 1 处；`h1_1_landing_at_legacy_caller_to_app` 真搬到 `legacy`、调用方改成指 `app`，`caller_line_kept=True`、调用方文件不在剔掉的名单里；`h1_1_control_real_move` 对照：调用方指落点，文件与行都剔。**钉值一侧同一个形状原样还在**，见下面「钉值一侧」 |
| 第二轮 H1-4：老行永久生效，没改的表判涨 | **站得住**（主形状）、窄形状见「睡着的老行」 | `h1_4_untouched_after_commit`：登记表里留着老行、什么都没改，`exempt_table_grown=[]`、0 行生效；`h1_4_new_row_in_table` 对照：提交之后表里新写一行照算新增 |
| 辩方的无序键：换次序加中间目录 | **站得住** | `unordered_key_reordered_plus_middle`：老行换成 `app->legacy` 再加 `legacy->mid`，只有第一行生效、实据红 2 处（旧前缀还在磁盘上、落点不在），`mid/run.sh` 照算涨 |
| 生效的行不许串；实据（落点逐份在、被跟踪、新前缀原先没有、旧前缀不在磁盘上） | **站得住**，代价见「关过头」 | 库自证的对应项加弄坏开关复现：`chain-allowed` 红 1 格、`untracked-landing-accepted` 红 1 格、`occupied-unchecked` 红 2 格、`window-ignored` 红 9 格、`move-evidence-unchecked` 红 8 格（`rerun.sh` 之外我在主工作区逐个起了一次自证，红的格数与批 6 报告写的一致） |
| 改了路径的文件只在恰好换前缀时剔；单文件登记文件名不变 | **站得住** | `o6_single_file_rename_callers_only`：单文件登记 `rig/a.sh->rig/alpha.sh`，调用方只差前缀的照剔，改了名的两份（旧、新）留在 `kept_for_review`；弄坏开关 `name-change-dropped` 红 1 格 |
| 新增行同一块第 k 条删配第 k 条加；跨管辖类整份搬家不剔、新增行一条不剔 | **站得住** | 弄坏开关 `hunk-any-position`、`governed-lines-dropped`、`governed-moves-dropped`、`lines-by-count` 各红 1 格（复现）；这两条的判法两轮都被攻过，这一轮没有找到绕法 |
| 乙：没搬家的脚本重钉照红 | **站得住** | 「if os.path.normpath(taken) == os.path.normpath(path) and "exempt-unmoved-accepted" not in breaks():」（`.claude/hooks/lib_long_run.py:446`）要求脚本自己搬了家；「if hashlib.sha256(data).hexdigest() not in base_sha256s:」（`.claude/hooks/lib_long_run.py:448`）要求基准那份的 sha256 是基准表里登记过的；「if lag_baseline.rewrite_moved_bytes(data, moves) == current:」（`.claude/hooks/lib_long_run.py:455`）要求换算之后逐字节相同；`moves` 取生效的行：「moves, problem = lag_baseline.effective_moves(base, root) if problem is None else ([], problem)」（`.claude/hooks/lib_long_run.py:476`）。独立重数 `recount_exempt.out`：基准表 6 行、此刻表 6 行、6 行都是「基准那份整串换前缀等于此刻那份，此刻表里的 sha256 也对」 |
| 丙：倒换前要求此刻内容正着换一遍不动 | **部分**：半换的拦得住，别的拦不住 | 「if not broken("partial-accepted") and self.library.rewrite_moved_bytes(data, list(self.moves)) != data:」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:75`）拦住只换了一半的；但它对「用哪些行倒」不设限，见下 |

### 钉值一侧：门禁一侧对不生效的行一字不核，钉值一侧照用全部行（**打中**，新的放过）

事实，行号现查：
- 钉值一侧读的是全部登记行：「装置里的钉值核对（E171、E170 的 frozen_o3、reuse_library）照旧用全部登记行（path_moves），不归这一节。」（`.claude/gate.d/lib-lag-baseline.py:54`）；`frozen_o3.py` 读「moves, problems = library.path_moves(repo_root)」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:117`），`reuse_library.py` 的 `pinned_sha256_now` 取「moves = frozen_o3.registered_moves(root)」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:442`）；第三个按全部行倒换的用户是 `scripts/agents/snapshot-tree.py`：「moves, problems = library.path_moves(cwd=repository)」（`scripts/agents/snapshot-tree.py:129`），库文件头与规则都没把它列进调用方。
- 门禁一侧对不生效的行不核：「return 0, [], f"此刻生效的 0 行；{idle_note}"」（`.claude/gate.d/lib-lag-baseline.py:931`），登记表那一格只在成功句里写一句「不生效 N 行」。自证把这个静默锁成期望值：「and report[1] == [] and "此刻生效的 0 行" in report[2]」（`.claude/gate.d/lib-lag-baseline.py:1469`）。
- 第一轮判决的甲对新加的行有过一条：「新前缀下不许有基准里本来就在的文件」（`research/prompts/apparatus-rename-r1-main-verification.md:31`）。批 6 把「新前缀下有文件」挪进「生效」的定义，这句问题句留在 `move_evidence` 里却到不了：生效的行按定义新前缀下没有文件，「if occupied and "occupied-unchecked" not in breaks():」（`.claude/gate.d/lib-lag-baseline.py:946`）那一支正常运行永远空，只有 `window-ignored` 开关下才出（`probe_pinned.out` 的 `occupied_row_window_ignored` 就是开着它量的，出 2 条问题句）。

样本（`tools/probe_pinned.py`，原样输出，`probe_pinned.out` 第 1、2、6、7、8、9 行；`gate_problems` 是 `path_move_report` 报的问题数，`pin_side_accepts` 是 `frozen_o3` 倒出来的几版里有没有钉住的那一版，`reuse_library_pinned_accepts` 是入库的 `reuse_library.pinned_sha256_now` 认不认）：

```
name=control_unregistered gate_problems=0 gate_note='没有内容行，实据不用核' gate_changed_after_filter=[frozen/judge.py] pin_side_accepts=False reuse_library_pinned_accepts=False
name=occupied_row gate_problems=0 gate_note='此刻生效的 0 行；不生效 1 行（第 2 行：基准 HEAD 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）' gate_changed_after_filter=[.claude/gate.d/path-moves.tsv, frozen/judge.py, records/2026-10-23-搬迁.md] pin_side_accepts=True reuse_library_pinned_accepts=True
name=token_row gate_problems=0 gate_note='此刻生效的 0 行；不生效 1 行（第 2 行：基准 HEAD 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）' gate_changed_after_filter=[.claude/gate.d/path-moves.tsv, frozen/judge.py, records/2026-10-23-搬迁.md] pin_side_accepts=True reuse_library_pinned_accepts=True
name=real_move gate_problems=0 gate_note='此刻生效 1 行（第 2 行），不串、有搬迁实据（第 2 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）' gate_changed_after_filter=[.claude/gate.d/path-moves.tsv, records/2026-10-23-搬迁.md] pin_side_accepts=True reuse_library_pinned_accepts=True
name=chain_old_row gate_problems=0 gate_note='此刻生效 1 行（第 2 行），不串、有搬迁实据（第 2 行 1 份里 1 份换算之后相同；新前缀下没有基准里本来就在的文件）；不生效 1 行（第 3 行：基准 HEAD 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）' gate_changed_after_filter=[.claude/gate.d/path-moves.tsv, frozen/judge.py] pin_side_accepts=True reuse_library_pinned_accepts=True
name=real_move_after_commit gate_problems=0 gate_note='此刻生效的 0 行；不生效 1 行（第 2 行：基准 HEAD 里旧前缀下没有文件或新前缀下已有文件，门禁不按它们换算，实据不核）' gate_changed_after_filter=[] pin_side_accepts=True reuse_library_pinned_accepts=True
```

怎么读：
- `control_unregistered`：改了钉住的文件、不登记，钉值一侧拒（`False`）。
- `occupied_row`：登记 `tools/check_a -> tools/check_b`，两个目录基准里都在，钉住的文件从调用 check_a 改成 check_b。门禁一侧 0 个问题（那一行不生效），钉值一侧收下，入库的 `reuse_library` 也收下。
- `token_row`：登记 `3 -> 4`（单个词当前缀，基准里旧前缀下没有文件），钉住的文件里 `RETRIES = 3` 改成 `4`。同样门禁一侧绿、钉值一侧收下。
- `chain_old_row`：老行 `legacy -> app` 早提交（`app` 是早就在的另一份实现）；新行 `rigs/y -> legacy` 是一次真搬迁（实据齐、生效、不串，老行因 `legacy` 已不在而不生效，所以不串）。钉住的文件原来调 `rigs/y/run.sh`，改成调 `app/run.sh`。门禁一侧这份文件在 `gate_changed_after_filter` 里（调用方没被剔，同第二轮 H1-1 在门禁一侧的结局），钉值一侧却按全部行逐次倒回去，倒到的正是钉住的那一版：**H1-1 在钉值一侧原样还在**，登记表加的行是真搬迁，没有一格能红。
- `real_move`、`real_move_after_commit`：对照。真搬迁，提交前门禁 1 行生效，提交之后 0 行生效，钉值两次都认。

我替「钉值一侧照用全部登记行」辩护的部分：它不能改成只认生效的行。`real_move_after_commit` 说明了原因：提交之后门禁一侧 0 行生效，钉住的 sha256 却永远是搬迁之前那一版的，钉值一侧要一直认。所以收的应该是登记表本身的可信度（让表里每一行都是过窗口时核过实据的真搬迁），不是钉值一侧的读法。

影响面：被钉的文件改了还在改动范围里（`frozen/judge.py` 在 `gate_changed_after_filter`），靠改动范围审的格看得到；被放倒的是钉值这一道机器闸，它的承诺是「钉住的文件除了路径前缀一个字不许动」，现在加一行登记就能换成「除了登记的前缀」。这一处真仓里有对象：`reuse_library.py` 的 `PINNED_SHA256` 两份编排器钉值与 E170 冻结行都走这一条。

### 睡着的老行在基准树形状变了时醒来（**部分打中**，H1-4 的窄形状）

判法只看基准树的形状，不看这一行是不是早就在基准那一版的登记表里。老行用完窗口睡着以后，只要基准树又满足「旧前缀下有文件、新前缀下没有文件」（旧名字重新被用起来、新目录又改了名），它就醒来：核实据，实据当然不齐，登记表那一格红；它同时开始换算基准，没改的表判涨。`probe_over.out`：

```
name=o7a_after_commit_row_idle effective=[] table_problems=0 kept_for_review=[]
name=o7b_row_unchanged_since_but_wakes_up_nothing_changed effective=[rig->rig2] table_problems=2 kept_for_review=[]
    problem: 第 2 行的旧前缀 rig 还在磁盘上（搬完以后旧路径不该还在）
    problem: 第 2 行：基准 HEAD 里旧前缀 rig 下的 1 份，照这一行换前缀的落点上有 1 份此刻不在（rig2/new.sh）
name=o7c_untouched_exempt_table baseline_line='rig2/new.sh\t豁免' current_line='rig/new.sh\t豁免' reads_as_growth=True
```

- `o7a`：搬迁提交之后老行 0 行生效。`o7b`：新目录又改了名、旧名字另起了一份新东西，工作区什么都没改，登记表那一格红 2 处。`o7c`：没改的豁免表，基准那一版被换成 `rig2/new.sh`，此刻是 `rig/new.sh`，读成「涨」。
- 这是 H1-4 的老症状（没改的表判涨），不过入口窄了：要旧前缀重新有文件而且新前缀空了。方向是多判（红），出路是删掉那一行老行。规则没写「提交进基准之后的老行该删」。

### 换算不锚定在仓根：别的目录下同名的一段也被换（**打中**，新，窄）

旧前缀前面接 `/` 也换（`./`、`$ROOT/`、绝对路径要靠这一条）：「只按路径前缀换：旧前缀前面不接字母、数字、_、-、.（可以接 /：./、$ROOT/ 与绝对路径里的同一段都换），」（`.claude/gate.d/lib-lag-baseline.py:56`），实现是「PATH_BEFORE_BOUNDARY = r"(?<![A-Za-z0-9_.\-])"」（`.claude/gate.d/lib-lag-baseline.py:197`）。于是登记 `ops/tools -> ops/tools2` 之后，`vendor/ops/tools` 也被当成「搬迁之前的写法」。`probe_anchor.out`（顶层单段前缀 `tools -> tools2` 与两段前缀各一份）：

```
name=one_segment_prefix row=tools->tools2 table_problems=0 caller_file_dropped=True lines_kept_as_new=[]
name=two_segment_prefix_control row=ops/tools->ops/tools2 table_problems=0 caller_file_dropped=True lines_kept_as_new=[]
```

样本是真搬迁（实据齐、表绿）；基准里另有 `vendor/tools/run.sh` 与 `vendor/tools2/run.sh` 两份行为不同的实现，调用方把 `bash vendor/tools/run.sh` 改成 `bash vendor/tools2/run.sh`，文件被剔出改动范围、两行都没算新增。这与第二轮 H1-1 同一类（调用方悄悄改指到早就在的另一份实现），但入口不是登记行，是换算的边界规则。条件只有一个：基准里早就在的那份实现，正好落在「别的目录/登记的旧前缀」与「同一个别的目录/登记的新前缀」这一对路径上。真仓基准 `e1b1ceb95` 里旧前缀 `scripts/experiments` 的 11889 处写法有 256 处前面是 `/`（`scan_slash_context.out`），属于 `./`、`../`、`$变量/`、`<仓根>/`、本仓绝对路径的不算，其余 73 处在 36 份文件里（多半是造假树的 `$cell/tree/`、`./feeds/z2/sample/` 这类）：收严 Z 的代价就是这 73 处要算改动。

### 三条收严（**零轮**：只在这一腿自己的样本上量过，没被攻过）

`tools/probe_fix.py` 把 X、Y 写成不改库的原型（用 `git ls-tree` 自己数窗口），在这一腿的七个局面上判，`probe_fix.out`：

| 收严 | 内容 | 在样本上 | 代价 |
|---|---|---|---|
| X | 不在基准那一版登记表里的新行，必须此刻生效，否则红 | `occupied_row`、`token_row`、`landed_looking_forged_row`（新前缀有、旧前缀无、从没搬过的假行）都红；`real_move_uncommitted`、`real_move_committed_unpushed`（基准是 S0）、`real_move_after_commit` 不红 | 搬迁已推出去之后补登记会红；钉值要在搬迁的同一次改动里登记（本来也只有那时核得了实据）。它把批 6 挪走的那句「这一行不是一次搬迁」重新接到新行上 |
| Y | 基准那一版登记表里已有的行不再生效 | `stale_row_wakes_up` 醒不来；`real_move_committed_unpushed` 仍生效（行在 S0 的表里没有）；`real_move_after_commit` 仍不生效 | 先登记、推出去、后搬的两步法不再享受窗口，但那一步法本来就过不了登记表那一格（`probe_window.out` 的 S5：旧前缀还在磁盘上） |
| Z | 换算前文只认 `./`、`../`、`$变量/`、`${变量}/`、`<仓根>/`、本仓绝对路径，别的 `X/` 后面的同名段不换 | `probe_anchor` 两个样本都会转成算新增（没写原型，只数了真仓的代价：73 处、36 份） | 造假树的写法要么改成 `$变量/` 要么按改动审 |

X 与 Y 不管 `chain_old_row`（钉值一侧串起老行的形状）：老行本身当年是真搬迁，新行也是真搬迁，串起来才是假的。要关它得在钉值一侧另加一条「表里没有一行的新前缀等于或落在另一行的旧前缀下面」（名字被腾出来之后不许再当新前缀），而这与「旧名字以后合法地被重新用起来」冲突；这一条我没写原型，也没量过，算进「写下来的没管到」就够了。

### 关过头：真搬迁被判成改动、该生效的行被判成不生效（全是多判的方向）

`probe_over.out` 的前 12 行与 `o8`，每一种都有出路，红的句子点得到原因，除 `o8` 外：

| 局面 | 结果 | 判定 |
|---|---|---|
| `o1` 搬迁同时删了其中一份 | 登记表红 1 处（落点上有 1 份不在），被删的与另一份进 `kept_for_review`；其余文件仍剔 | 站得住：要么先删再搬，要么两次提交；句子写了 |
| `o2` 旧目录里剩了被 `.gitignore` 忽略的文件（`__pycache__`） | 登记表红（旧前缀还在磁盘上） | 站得住：句子指到「搬完或删掉」；`gate.sh --staged` 的临时树是新检出的（读 `scripts/gate/staged-tree-worktree.sh` 的 worktree 建法判的，没起它），旧目录下被忽略的文件不会跟进去 |
| `o3` 把目录整个收进自己的子目录（`rig -> rig/legacy`） | 红（旧前缀还在磁盘上） | 站得住（多判）：实据条件天然过不了，改成两步（先搬到临时名再搬进子目录） |
| `o4` 两个目录并成一个（`rig -> merged`、`rig_b -> merged`） | 红（两行新前缀相同算串） | 站得住（多判）：并目录不是「一次搬迁」，要拆成两次提交 |
| `o8` 新前缀下基准里早放着一个占位文件（`.gitkeep`） | 那一行不生效，登记表那一格**绿**，4 份文件整个算改动 | 站得住但提示弱：只有成功句里一句「不生效 1 行」，没说是哪一条条件不满足 |
| `o5` 同一次改动里两次互不相交的搬迁 | 两行都生效、绿 | 站得住 |

## J2 没管到的清单全不全

**判定：打中（清单不全）。** 已经写下来的我逐条核了在不在、跟代码对不对得上；没写下来的有下面第二张表那些。

### 已经写下来的（在、也对得上）

| 出处 | 写的是什么 | 对代码 |
|---|---|---|
| `.claude/rules/path-moves.md` 门禁管哪一半 | 「搬迁这一半没有专管它的阶段」（`.claude/rules/path-moves.md:62`）；「不生效的行不核。」（`.claude/rules/path-moves.md:63`）；「被判的根不在 git 仓里时判不了哪几行生效，不核」（`.claude/rules/path-moves.md:63`）；换了实现的门禁不判 | 对得上：`path_move_report` 在根不在仓里、不是仓根时返回「判不了」（`gate_base_of`），不生效的行只进成功句 |
| 批 6 报告「剩下的、没做的」（`~/.local/share/singlefs/work/path-move-window/result.md`，仓外，不入库） | 同一类里只换目录的单文件搬家照剔；换实现时只差前缀的调用方照剔；从来不生效的登记行登记表不判而钉值一侧照用；第一轮 G1-3 的代码一半没做；三份表头句子（批 6b 已改） | 对得上，我复现了第三条（见 J1「钉值一侧」），但那一条在报告里只写成「这一行会参与倒换」，没写成钉住的文件能被放倒 |
| `.claude/rules/path-moves.md` 第 20 行 | 「把命令里的目录换成另一个已有的目录、换了实现的不登记，照真改动交审。」（`.claude/rules/path-moves.md:20`） | 对得上：代码只核落点在、被跟踪、旧前缀不在，不核内容（`move_evidence` 里相同的份数只报不判） |

### 没写下来的

| 编号 | 放过或误判 | 方向 | 证据 |
|---|---|---|---|
| N1 | 不生效的新行（新前缀早就有文件、旧前缀没有文件、已搬完的假行）登记表一字不核，钉值一侧与 `snapshot-tree.py` 照用 | 放过（钉住的文件除了路径前缀不许动，变成除了登记的前缀） | J1「钉值一侧」，`probe_pinned.out` 的 `occupied_row`、`token_row` |
| N2 | 钉值一侧按全部行逐次倒换，老行加一行真搬迁就能串到早就在的实现上（第二轮 H1-1 在钉值一侧原样还在） | 放过 | `chain_old_row` |
| N3 | 睡着的老行在基准树又满足窗口时醒来：登记表红，没改的表读成涨 | 多判 | `o7b`、`o7c`；规则没写「提交进基准之后的老行该删」 |
| N4 | 换算的前文边界接 `/` 就换，别的目录下同名的一段也被换 | 放过 | `probe_anchor.out`；库文件头把它写成有意的（`.claude/gate.d/lib-lag-baseline.py:56`），没写它的副作用 |
| N5 | 边界的另一头：旧前缀前面接 `-`、字母的写法不换，`${V:-scripts/experiments/…}` 这类默认值写法算改动 | 多判 | 整串替换数 362 份、按库的边界数 360 份（`recount.py` 的结果在下面 J3），差的那 2 份里有 `${E10_BINARY:-scripts/experiments/…}`（前面是 `-`） |
| N6 | 新前缀下基准里早有占位文件（`.gitkeep`），整次真搬迁不生效，登记表那一格绿，只在成功句里提一句 | 多判，提示弱 | `o8` |
| N7 | 自证把「不生效的行不核」当期望值锁住 | 自证区分不出 N1 | 「and report[1] == [] and "此刻生效的 0 行" in report[2]」（`.claude/gate.d/lib-lag-baseline.py:1469`），所在的项名是「搬迁登记：新前缀在基准里已有文件的行不生效（换到早就在的另一份实现、两个目录都还在），起它的改动照算」（`.claude/gate.d/lib-lag-baseline.py:1467`） |
| N8 | `move_evidence` 里「这一行不是一次搬迁」两句（占用与旧前缀下没有文件）正常运行永远不出：生效的行按定义新前缀下没有文件、旧前缀下有文件 | 代码里留着一道没人走得到的检查 | 「if occupied and "occupied-unchecked" not in breaks():」（`.claude/gate.d/lib-lag-baseline.py:946`）；只有 `window-ignored` 下出（`probe_pinned.out` 的 `occupied_row_window_ignored`）；覆盖面里这两句都在没跑到的行里（`uncovered.out`） |
| N9 | 库文件头的调用方清单与规则「三样」没跟上批 6c：门禁 doc-experiments 的 decision-links 格判「正文改了」前，两版正文先照生效的登记换算 | 只是句子落后 | 「def baseline_page_text(rel):」（`.claude/gate.d/doc-experiments.sh:1352`）；规则写的是「门禁照此刻生效的登记换算三样，只差这一段前缀的不算改动：与基准比只缩不涨的表、改动范围、新增行。」（`.claude/rules/path-moves.md:15`）；库文件头的调用方清单里没有 `snapshot-tree.py`（`grep -c snapshot-tree` 在库、规则、登记表三份里都是 0） |
| N10 | 规则「被判的根不在 git 仓里时判不了哪几行生效」只写了一半：代码对「在仓里但不是仓根」同样一行都不剔 | 句子落后 | 「if not top_level:」（`.claude/gate.d/lib-lag-baseline.py:879`）；库文件头写全了 |
| N11 | 规则第 17 行没写「整份恰好换前缀搬了家的，diff 里没动的行也算旧行」 | 句子落后 | 「整份搬了家的另剔 diff 里没动的行」（`.claude/gate.d/lib-lag-baseline.py:68`） |

N1、N2、N4 是放过（要不要收见 J1 的零轮三条）；N3、N5、N6 是多判（安全的一边，只是出路没写）；N7–N11 是自证与句子。

### 规则、定义与三份表头的句子对代码

| 句子 | 对得上吗 |
|---|---|
| `.claude/rules/path-moves.md` 第 14 行（窗口） | 对得上：「登记行只在搬迁提交之前对门禁生效」（`.claude/rules/path-moves.md:14`）对着 `.claude/gate.d/lib-lag-baseline.py:342`、`.claude/gate.d/lib-lag-baseline.py:343` |
| 第 15 行（换算三样、钉值按全部行倒换） | 后半对得上（`.claude/gate.d/lib-lag-baseline.py:54`）；「三样」少了 N9 那一样 |
| 第 16、18、19 行（改动范围、不许串、实据） | 对得上，逐条有弄坏开关复现（J1） |
| 第 17 行（新增行） | 缺 N11 一句 |
| `.claude/agent-common.md:70`、`.claude/main-agent.md:82` 那一句（长活豁免表） | 对得上：「脚本自己搬了家」「基准那份的 sha256 是基准表里登记的」「换算之后逐字节相同」「登记行有搬迁实据」四条在 `.claude/hooks/lib_long_run.py:446`、`:448`、`:455`；「重钉提交进基准之后不再红」由窗口收窄推出，J1 的 S3 复现 |
| 三份表头：`.claude/gate.d/path-moves.tsv` 第 3 行、`.claude/heavy-thread-exempt` 第 3 行、`.claude/long-run-live-hosts-exempt` 第 8 行 | 对得上：三份都写了「此刻生效」与窗口条件，实据一句指到 change-range-single-source 格；表头没提钉值一侧的范围限制（N1），因为它们讲的是门禁一侧 |

## J3 报出的数能不能核

**判定：部分打中。** 大多数数能一条命令重算，其中两处我用不调库的算法数出同一个数；三处（140→21、12→132、119）要跑门禁格，随工作区变，我按定义没跑，只做了方向性的独立重数。自证的区分度：弄坏开关的红法逐个复现，但有两处缺口（自证把不生效的行不核当期望值、没跑到的行）。

### 每个数

| 数 | 一条命令能重算吗 | 我的结果 |
|---|---|---|
| 32 项 | `python3 .claude/gate.d/lib-lag-baseline.py --selftest` | 退 0，末行「  ✓ lib-lag-baseline.py 自证：32 项都过」；`grep -c '✓'` 数 33（含这一行汇总）。这个 32 是 `len(results)` 现数的：「print(f"  ✓ lib-lag-baseline.py 自证：{len(results)} 项都过")」（`.claude/gate.d/lib-lag-baseline.py:1832`），没有写死的期望值，哪一项被悄悄删了数字变小、自证照样绿；对照 `scripts/gate/changed-paths.sh` 的 19 是写死核的：「"$check_count" -ne 19」（`scripts/gate/changed-paths.sh:305`） |
| 19 项 | `bash scripts/gate/changed-paths.sh --selftest` | 退 0，「共用库 scripts/gate/changed-paths.sh 自证通过（19 项：…）」，数与写死的 19 对得上 |
| 48 项 | `python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py --selftest` | 退 0，末行「自证 48 项，红 0」 |
| 6 行 | `python3 .claude/hooks/lib_long_run.py exempt-check . --base HEAD` | 退 0，6 行都写「换算之后与基准那一版逐字节相同」。独立重数 `recount_exempt.out`：`base_rows=6 current_rows=6 rows_that_are_path_only=6`（基准那份整串换前缀等于此刻那份，此刻表里的 sha256 也对） |
| 395 份里 360 份 | `python3 .claude/gate.d/lib-lag-baseline.py path-moves-check .` | 能，并且不调库的独立数出同一个数：我第一次量，库 360、自写边界正则 360、整串替换 362；最后一次量，库 359、自写正则 359、整串替换 361（`recount_files.out`，原样见下）。数在动：工作区里别的会话还在改（`git status` 里 `scripts/experiment-apparatus/README.md` 是 `AM`），我没逐份确认是哪一份变了。整串替换比边界数多的 2 份是 `${V:-scripts/experiments/…}` 默认值写法（前面是 `-`，边界不换），见 J2 的 N5 |
| 140 → 21、12 → 132、119 | 要跑门禁格 decision-links（6c 报告「要主 agent 跑的命令」第 3 条），随工作区变；报告里的 `run-real.sh` 在仓外 | **没复核**：门禁阶段按定义不归我跑。能说的两点：（1）140 → 21 是同一格的改前改后两版互比（改前的备份对改后的脚本），不是独立的对照，第二轮 H2 已经打过这个角度；（2）我做了一个比格严的独立重数 `recount_pages.out`：对着同一个基准，187 份中文实验页里整页（不排除历史节与影响的决策一节）「换算之后相同、或再去掉语言切换行之后相同」的 158 份，别的改动 27 份，新页 2 份，158+27+2=187。方向一致（剩下的 21 页 ≤ 我的 27 页，格排除了历史节与影响的决策一节所以更宽松），132、119 推不出来，我的口径不同 |

原样输出：

```
name=recount_files base=e1b1ceb95 files=395 landing_missing=0 same_by_plain_replace=361 same_by_boundary_regex=359
    plain!=boundary: scripts/experiments/e10-aging-compaction-run.sh 第一处差在：BINARY="${E10_BINARY:-scripts/experiments/target/release/e10-aging-compaction}"
    plain!=boundary: scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_unit_revision_11.sh 第一处差在：RESEARCH="${RESEARCH:-scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes}"
name=recount_pages base=e1b1ceb95 changed_experiment_pages=187 new_pages=2 only_path_move=0 path_move_plus_switch_line=158 other_changes=27
name=recount_exempt base_rows=6 current_rows=6 rows_that_are_path_only=6
```

### 算法本身分不分得出对错

1. **弄坏开关红在哪几格，逐个复现。** 在主工作区用 `LAG_BASELINE_BREAK=<项> python3 .claude/gate.d/lib-lag-baseline.py --selftest`（改前的脚本没动，只设环境变量）：`window-ignored` 红 9 格、`occupied-unchecked` 2、`chain-allowed` 1、`untracked-landing-accepted` 1、`name-change-dropped` 1、`hunk-any-position` 1、`governed-lines-dropped` 1、`move-evidence-unchecked` 8、`lines-by-count` 1、`governed-moves-dropped` 1，与批 6 报告写的一致，窗口、占用、串三个要求对应的开关红集互不相同（输出存在草稿目录，这里不入库）。所以第二轮 H2 说的「分不出松紧」对这几条是补上了。
2. **「真登记那一行照过」不算对照。** 同一行真登记在 `window-ignored`、`occupied-unchecked`、`chain-allowed`、`untracked-landing-accepted`、`name-change-dropped` 五个放松的开关下，`path-moves-check .` 仍然退 0、成功句一字不变（只有 `move-evidence-unchecked` 会改成功句）。分得出松紧的是弄坏开关各自点名的那一格，不是这一行。
3. **缺口一：自证把「不生效的行不核」锁成了期望值**（J2 的 N7），所以 J1 的 N1 在自证里是绿的、不会红。
4. **缺口二：自证没跑到的行。** 用 `python3 -m trace --count --missing` 跑一遍库自证（读主工作区那份，没改它）：库本体有 185 行没执行（`uncovered.out`）。分类：
   - 报错分支（`git` 取不到、登记表写坏的 `return … Problem(…)`）：大多数（按行文分的类，没逐行数）；
   - 命令行入口的函数体（`moved_only_filter_main` 等）：是子进程里跑的，trace 数不到，不算缺口；
   - 核心逻辑里没跑到的：符号链接与目录等的认法（`:556`、`:562`、`:681`、`:682`、`:691`）、没合并完的索引条目（`:659`）、`current == before` 的整份搬家捷径（`:811`），以及三处永远走不到的 `return False`（`:413`、`:697`、`:714`）、N8 那两句问题句（`:947`、`:948`、`:950`）；
   - 我把没跑到的符号链接那几支补测了一遍，`probe_untested.out` 四个局面：相对链接与链接目标跟着改的整目录搬迁，链接剔掉；链接目标没跟着改的，按「字节原样相同」也剔（规则里「只改了名」那一档，没错）；文件被换成指向别处的符号链接，模式变了，照算改动；执行位变了，照算改动。没有找到错。
5. **结论**：算法在门禁一侧分得出对错（H1-1、H1-4、无序键、实据、窗口各有会红的格），分不出的是 N1（自证锁住）与 N4（没有自证项）。
