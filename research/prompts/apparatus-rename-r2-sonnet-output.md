# apparatus-rename-r2 辩方腿报告（Sonnet，2026-10-23）

被复核的判决：`research/prompts/apparatus-rename-r1-main-verification.md`；依据的腿报告：`research/prompts/apparatus-rename-r1-sonnet-output.md`、`research/prompts/apparatus-rename-r1-opus-output.md`。背景材料 `research/prompts/_apparatus-rename-r2-background.md`、正文 `research/prompts/_apparatus-rename-r2-body.md`。没读禁读清单里的文件。

## 复跑

- 模型目录：`research/prompts/apparatus-rename-r2-sonnet-model/`（`tools/` 下五个样本脚本加 `rerun.sh`、`build_frozen.sh`，`out/` 下四份输出）。
- 建冻结副本：`bash research/prompts/apparatus-rename-r2-sonnet-model/tools/build_frozen.sh <仓根> <新目录>`（只拷五份，四份被判文件的 sha256 逐个核开工快照 `research/prompts/apparatus-rename-r2-snapshot/judged-sha256.txt`）。
- 复跑：`bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/apparatus-rename-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新的草稿目录> --model research/prompts/apparatus-rename-r2-sonnet-model`。本腿跑过一次：`probe_h1`、`probe_rows`、`probe_h3` 三份 `same=yes`；`probe_real` 读主工作区，数随工作区变，不逐行比。
- 样本都是临时 git 仓，调的是冻结副本里的 `lib-lag-baseline.py`；「过半」「字面」两列是按独立数出的相同份数现算的，没有改库、没有造改过的库。

冻结副本五份的 sha256（与开工快照一致）：

```
282acf47bee46bd40a36dd33dfac797514541fafc1cb3f831c1b4ace99dd50af  .claude/gate.d/lib-lag-baseline.py
66e8b4c30b03131d66236c65355a5c5d19c24213a9830f78a5a51713643fe0dc  .claude/hooks/lib_long_run.py
6f5a155211626541c349fd21857c1d3adf2cec644d9097a647044970b2623e14  scripts/gate/changed-paths.sh
c66d61e8ff8ec647ca4c3752d442c22779c1291bd0acad1d33f93531f6a13f85  .claude/gate.d/path-moves.tsv
6bd3d8a1184e3213c6b2259442230044a0a07f88e3481b15901c37c4a56dc8ba  .claude/scripts/project_preflight.py
```

模型目录文件的 sha256：

```
b8b61680bf7c04d9eadb010b86b71eff858b4130e08841f67fb0bd0be20cc8c1  tools/build_frozen.sh
77bfcaf5e1921a016a0e6d91aeb2a86994c694a88df6f8e8be93758f66d6a671  tools/common.py
f67638daa170d3a917dc928d19bfce0bba367eb06a01c5ca0660cd5cdd70b887  tools/probe_h1.py
7668098ade2b3c0ca57fc41d38a163ee0c28db93412ab3befccf225513dface7  tools/probe_h3.py
51360b620e7f38da9b6017f82bae0342ca3f7d6600b8a4d8002ee0050ac72350  tools/probe_real.py
2657186fafdfe7e17993bde3e154821cdb1e6744b0bfa682105f6f58073301aa  tools/probe_rows.py
eb1320f22a17f3d0b6ea13bc98924e9291fd3d4da45b9a0c40d7e04208cb6768  tools/rerun.sh
10073349b3f3d30cd57589dd9c87926b4cc6d171b0de5436a87679ee57e76cb0  out/probe_h1.out
588430e3960c698f7dbfb8c2d5e9b94aff407627e859119d5c9e68934c5ff0b0  out/probe_h3.out
c75cc2a715764b1a17188fad8c2539af2fd77b4d9aedc9de8925771b3d6a17ec  out/probe_real.out
b32c0d020967444b917eff27227723e11c4f5e9ad43ab31526e55a8ec42eec7d  out/probe_rows.out
```

另：攻方第一轮的 `attack_samples.py` 拿 r2 开工快照那 15 份建的冻结树重跑，23 行输出在草稿目录 `~/.local/share/singlefs/work/apparatus-rename-r2-defense/attack_r1_now.out`（不入库，原因见「没做什么」）。

## 总结一览（判决条目 / 辩方判定 / 证据）

判定三档：站得住 = 判决那一条现查仍成立；够不着 = 判决或改法没碰到同一形状的相邻写法；同样打中别的臂 = 同一形状也打中判决没选的替代方案。

| 判决条目 | 辩方判定 | 证据（细节在后面各节） |
|---|---|---|
| G1 误放 打中 | 站得住；改法甲对攻方当时造的两个形状够得着，对同一类的相邻形状（旧目录整个换成另一份实现、新目录里同名文件清空）够不着 | 攻方 `attack_samples.py` 现状：`launder_parent_excuse`、`launder_deleted_directory` 都红在登记表那一层；本腿 `probe_h1` 的 r0、r2、r3、r4 登记表判绿、`.claude/agents/caller.md` 被剔出改动范围 |
| G2 读法一致 部分打中 | 站得住；丙的「正着换不动」与规则第 5 步留旧路径的句子有一处潜在冲突 | 「2.4」；攻方 `reverse_partial.partial` 现状四项都 False |
| G3 改动范围剔除的副作用 打中 | 站得住；戊只修了路径层，新增行层同一形状仍在（判决没点） | `probe_h3` 的 c1、c3：改动范围剔掉 0 份，新增行 3 条留下 0 条 |
| G4 句子 打中 | 站得住；己改完之后有两处残留句子与代码对不上 | `.claude/rules/path-moves.md:6` 与 `:13`，`lib-lag-baseline.py:770` |
| 改法甲，判决原文（每份换算之后相同） | 站得住（最严）；对真登记行会红，红的原因是同一批里另改了 35 份 | `probe_real`：395 份里原样相同 290、整串替换之后相同 362；批 4 报告写 360 |
| 批 4 落地的「过半」门槛 | 站得住（弱）：挡 r0、r2、r3、r4、m3，真登记行零成本，挡不住垫相同文件的 r1 | 「二、H1」替代方案×形状表 |
| 主 agent 改成的无门槛（相同份数只报不判） | 比「过半」与判决字面都宽；在 r0、r2、r3、r4、m3 上放宽于「过半」，在 m2、r1、真行上放宽于判决字面 | 同上；`lib-lag-baseline.py:768`、`:1254` |
| 「只核这一次新加的登记行」这一取法 | 取法站得住（全量重核不可行）；键不稳：换登记行次序不算新加 | `probe_rows` t3、t1 |
| 乙（没搬家的脚本重钉照红） | 站得住，没找到新的放过；误判方向都是多判 | `lib_long_run.py:445`；`probe_real` 里库比独立写法严 7 份 |
| 丙（倒换前要求正着换不动） | 站得住；潜在冲突，今天没有实例 | `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:74`，路径搬迁规则第 5 步 |
| 丁（按 diff 位置配对） | 站得住（只读码加样本里的旁证，没单独攻） | `probe_h1` 里剔的行只有与被删旧行换算相同的 |
| 戊（跨管辖类整份搬家不剔） | 站得住，同样的形状在新增行层仍放过 | `lib-lag-baseline.py:541`、`:731` |
| G1-3 代码一半「不做」 | 出局的理由在保守方向上不成立 | `research/prompts/apparatus-rename-r1-main-verification.md:40` 的理由；真仓基准里命中 9 处 |
| 第一轮用来证「不再放过」的样本与对照 | 对照有一半不独立；样本只覆盖被打中的形状 | 「三、H2」 |

## 二、H1 修补本身

### 2.1 甲去掉「过半」之后，不是搬迁却满足三条的登记：有

现行三条的实现：新前缀下有基准里本来就在的文件判红（`.claude/gate.d/lib-lag-baseline.py:788` 的 `if occupied:`），旧前缀在基准里没有文件判红（`:791` 的 `if not subjects:`），落点上有一份此刻不在判红（`:807` 的 `if missing:`）；落点上有什么只数不判：「换算之后相同的份数只报不判」（`.claude/gate.d/lib-lag-baseline.py:768`），计数那一行是「same += mode == tree[path].mode and (data == before or data == rewrite_moved_bytes(before, moves))」（`.claude/gate.d/lib-lag-baseline.py:806`）。

所以「旧目录整个删掉，新目录里放同名文件、内容是另一份实现」满足三条。库自己的自证把它当成过的：「and rewritten_whats == []」（`.claude/gate.d/lib-lag-baseline.py:1254`），说明写着「换实现（实据照过，内容照算改动）」（`.claude/gate.d/lib-lag-baseline.py:1256`）。规则的句子说的是相反的：「换了实现的不登记，照真改动交审」（`.claude/rules/path-moves.md:13`）。库里给用户的出路提示也还带着「搬完又改了大半文件的，先把搬迁单独提交，再改」（`.claude/gate.d/lib-lag-baseline.py:770`），现在没有任何一条判法会产出这个红，提示是死的。

样本（`probe_h1.py`，基准里 `rig-a` 下 6 份，调用方 `.claude/hooks/caller.sh`、`.claude/agents/caller.md` 各一行指向 `rig-a/f1.sh`，登记 `rig-a → rig-b`）。「独立数相同」是不调库里换算、只用 `bytes.replace` 数的；三个方案列是按这个数现算的结论：

| 形状 | 独立数相同 | 判决字面（每份相同） | 批 4 的过半 | 现行（只报不判） | 现行下 agents 调用方出改动范围 |
|---|---|---|---|---|---|
| m1 真搬迁全相同 | 6/6 | 放 | 放 | 放 | 是（正确） |
| 真登记行（`probe_real`） | 362/395 整串替换；库算 360 | 拒 | 放 | 放 | — |
| m2 真搬迁另改一份 | 5/6 | 拒 | 放 | 放 | 是 |
| m3 搬完又改了大半 | 2/6 | 拒 | 拒 | 放 | 是 |
| r0 整个换实现 | 0/6 | 拒 | 拒 | 放 | 是 |
| r2 全清空成空文件 | 0/6 | 拒 | 拒 | 放 | 是 |
| r3 小目录两份全换实现 | 0/2 | 拒 | 拒 | 放 | 是 |
| r4 小目录换被调用的一份 | 1/2 | 拒 | 拒 | 放 | 是 |
| r1 垫 4 份相同、换被调用的 2 份 | 4/6 | 拒 | 放 | 放 | 是 |
| c1 拷贝、旧目录还在 | 6/6 | 拒 | 拒 | 拒（三列都红在「旧前缀还在磁盘上」，与实据无关；`:844` 一带） | 是（但登记表红） |

原样输出在 `research/prompts/apparatus-rename-r2-sonnet-model/out/probe_h1.out`（r0 一行：现行实据=放行 独立数相同=0/6 工具员的过半门槛会=拒 agents调用方被剔出改动范围=True）。

这些形状放过的东西：调用方（agent 定义里的「开工先读 rig-a/f1.sh」、钩子里的 `bash rig-a/f1.sh`）只差前缀，被剔出改动范围与新增行，被调用方换成了另一份实现，只在非管辖类的路径下留成改动。盯着 agent 定义改动的格用 `gate_changed_paths` 挑文件：`changed_since_gate="$(gate_changed_paths "$gate_base" untracked)"`（`.claude/gate.d/doc-process-records.sh:491`），`crates-adversarial-review|agent-def-adversarial-review) load_changed_since_gate; load_added_since_gate ;;`（`.claude/gate.d/doc-process-records.sh:2032`），剔行经 `_lib_changed_paths_added_lines_raw "$@" | python3 "$_lib_changed_paths_lag_baseline" moved-lines-filter "$base"`（`scripts/gate/changed-paths.sh:169`）。这条路径上的安全性靠的是「被调用方换算之后与旧的一样」，三条里没有一条在判它。

没造出来的：排除表、豁免表那一层。按 sha256 钉的豁免表要求基准里的脚本换算之后与此刻逐字节相同（`.claude/hooks/lib_long_run.py:437` 起的 `moved_script_unchanged`），换实现过不了；按路径锚串的排除表（`.claude/heavy-thread-exempt`）在原地改内容时本来就留着那一行，换实现与原地改一样，不比原来多放过什么。这两句是读码得的，没造样本。

### 2.2 「过半」站得住的理由，与它挡不住的

- 它挡的是上表 r0、r2、r3、r4、m3，真登记行（395 份里 360 份）零成本过。
- 它挡不住 r1（垫相同文件），这一形状同样打中「现行」；判决字面能挡，但真登记行会红（同一批里另改了 35 份，批 4 报告里的原话也是这么写的）。
- 先例：git 自己的改名检测对每个文件默认相似度 50%（`man git-diff` 的 -M 一节写 The default similarity index is 50%）。找到了同量级的先例，但它量的是一个文件的内容相似度，不是一个目录里有几份没动，不是同一个量。
- 它的门槛是为了让手里这一个正例过而取的（批 4 报告第一节：照字面真登记行就红，落地改成过半）。这一点算量法的缺点，不算门槛错；换成别的取法（见 2.6）同样要拿这一个正例定。

推翻这一条辩护的现象：造出一种合法搬迁，相同份数不过半，而且按规则（先把搬迁单独提交）无法拆开；或者证明 r0、r2、r3、r4 这类登记在下游被别的格拦住（本腿读到的下游没有）。

### 2.3 「只核这一次新加的登记行」

取法本身站得住：全量重核做不到。搬迁提交之后基准里旧前缀下就没有文件了，`probe_rows` 的 t3 把一行删了提交、再加回来，被判红：「旧前缀 rig-a 在基准 HEAD 里没有文件」，同一次还红在新前缀下有基准里的文件。重核已提交的行只会全红，所以只能认「基准那一版表里有的不再核」。

键不稳：新加与否看的是 `registered` 这个无序的（旧前缀、新前缀）对集合：「registered = {tuple(fields[:2]) for _number, fields in content_rows(base_table or "") if len(fields) >= 2}」（`.claude/gate.d/lib-lag-baseline.py:774`），「fresh = [(index, move) for index, move in enumerate(moves) if (move.old, move.new) not in registered]」（`.claude/gate.d/lib-lag-baseline.py:775`）。次序不在键里，而换算是按次序串起来的（`forward_path`）。`probe_rows` 的 t1：

```
基准：rig-x → rig-a、rig-a → rig-b 两次真搬迁都做完提交，文件在 rig-b；docs/note.md 留一行指历史的「当年在 rig-x/foo.sh 跑过」
工作区：把两行换次序（rig-a → rig-b 在前、rig-x → rig-a 在后），另起一个 rig-a/foo.sh（任意内容），docs/note.md 那行改成指 rig-a/foo.sh
结果：登记表判绿（实据句「没有新加的行（都在基准 HEAD 那一版的登记表里），实据不再核」）；docs/note.md 被剔出改动范围；那行新增行被剔
基准那行换算：原次序得 rig-b/foo.sh，换次序后得 rig-a/foo.sh
```

换次序之后旧前缀 rig-a 还在磁盘上能过，靠的是「后面的登记又搬进它」那一个豁免（`.claude/gate.d/lib-lag-baseline.py:844` 的 `moved_back`）。所以「换次序 + 另起一个中间目录」能把老登记行重新指到一个没有任何实据的新目录。这是第一轮没攻过的角度，也是「新加」这一取法的键的缺口，不是取法本身的错。

其余改写形状都判住：t2（把一行的新前缀改成别处）红在「旧前缀 rig-a 在基准 HEAD 里没有文件」；t4（同提交里改日期）与 t5（重复一行）绿，只改了不进键的列或复制了同一对，换算结果没变；t6（两行都改成直接搬到已有目录）红。

推翻 t1 这一条：用更长的真历史（三行以上的链）证明按换次序后基准里没有任何一行文本会被换成不同的落点，那就只是理论缺口。本腿没有量真仓里这类链的数量（现在表里只有一行）。

### 2.4 乙、丙、丁、戊各自有没有新的放过或误判

- 乙：没找到新的放过。没搬家的脚本重钉：「if os.path.normpath(taken) == os.path.normpath(path)」（`.claude/hooks/lib_long_run.py:445`）起的那一支照红，攻方 `launder_parent_excuse.exempt_check` 现状 rc=1。误判有一类，方向是多判：换算只认路径边界，前面不接字母、数字、_、.、-：「(?<![A-Za-z0-9_.\-])」（`.claude/gate.d/lib-lag-baseline.py:157`）。`${E10_BINARY:-scripts/experiments/target/…}` 前面是 `-`、`printf 'E910\tscripts/experiments/…'` 前面是 `t`，库不换。`probe_real` 用整串替换独立数，改动范围里独立写法比库多认 7 份只差搬迁的文件（第一条 `scripts/experiments/e10-aging-compaction-run.sh`、`.claude/gate.d/fixtures/doc-experiments.sh/evidence-in-repo-red/setup.sh`），没有库认而独立不认的。按 sha256 钉的豁免脚本若含这种写法，搬完原地改了路径，会因「仍不逐字节相同」判红、要重钉；今天豁免表 6 行都绿（`lib_long_run.py exempt-check . --base HEAD`，主 agent 正文里的观测，本腿没重跑）。
- 丙：没找到新的放过。潜在冲突：丙要求此刻内容照登记正着换一遍一个字节不动：「if not broken("partial-accepted") and self.library.rewrite_moved_bytes(data, list(self.moves)) != data:」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:74`），动了就交空。路径搬迁规则第 5 步要求记搬迁本身的行保留旧路径。被钉的文件里若有这样一行，丙会判它「不是钉住的那一份」。今天没有实例：两个 E171 目录里含 `scripts/experiments` 的只有 `reuse_library.py`，它不在 `PINNED_PATHS` 里。
- 丁：只读码加样本旁证。`probe_h1` 里被剔的新增行是被删旧行换算之后相同的那一行（r0 一行：剔掉的只有调用方那一行），被调用方的 f1.sh 新行没剔。整份搬家的分支「numbers |= set(range(1, len(current_lines) + 1)) - touched」（`.claude/gate.d/lib-lag-baseline.py:731`）是有意让没动的行都剔。没有单独攻 diff 解析（多块、重名行）。
- 戊：路径层站得住，见 4.2。

### 2.5 甲与登记行写法的已有判法

`attack_samples.py` 现状里 `launder_parent_excuse`、`launder_deleted_directory` 的登记表问题句都点名了新前缀下有基准里本来就在的文件，与判决 G1 描述一致，没有放过。`chain_midway` 现状红在登记表（第 2 行旧前缀在基准里没有文件），因为样本把两行同时登记、基准在两次搬迁之前；分两次提交登记时第二行基准里旧前缀下有文件，可以过，没有造样本。

### 2.6 可选的改法（零轮，没量过代价）

只列，不选：

1. 恢复判决字面（每份换算之后相同）：真登记行红；代价是这一批要拆成「先搬迁一个提交、再改另一个提交」，库自己的提示本来就这么说（`:770`）。
2. 过半：批 4 落地的写法，真行过；r1 过。
3. 实据不够（没到每份相同）时，登记行只用来换算只缩不涨的表与钉值，不用来剔改动范围与新增行。真行（35 份另改）就不剔那一批；这条没有造样本，也没量真仓里会多出多少改动范围。
4. 在「新加」的键里加次序：同一对在基准里出现时，还要比它在链里换算到的落点与基准里那一版相同；t1 会红。

### 2.7 G1-3 代码一半「不做」

判决的理由：「正则认全 rev 写法不稳」（`research/prompts/apparatus-rename-r1-main-verification.md:40`）。不稳的方向要分开看：多认一种不换的写法，只会让这一种写法的文件多算一次改动，是保守方向。真仓基准 `e1b1ceb95` 里命中数（`git grep` 现数）：「十六进制:旧路径」2 处，「别的词:旧路径」6 处（`root:` 4 处、`judge:` 2 处），`git log/show/blame/diff/ls-tree … -- 旧路径` 1 处，共 9 处，全仓含旧路径的文件 1082 份。保守地对这三类都不换，多算改动的至多是这九处所在的几份文件。所以「不稳」不是不做的好理由，不做的理由只剩人判（规则第 5 步那一半）。攻方 `history_references` 现状仍是：`git show v0:scripts/experiment-apparatus/run.sh` rc=128，那些文件仍被剔出改动范围。

推翻这一条：证明保守不换会在真仓里使改动范围多出大批文件（本腿数到 9 处，没数过按文件算的份数）。

## 三、H2 得出结论的方法

### 3.1 「真登记那一行照过」「726 份照剔」是怎么量的

- 「真登记那一行照过」：成功句出自 `path_move_report`，它既是判法也是对照，自己和自己比，量法分辨不出对错。本腿用不经库里换算的写法重量（`tools/probe_real.py`，只用 `git ls-tree`、`git cat-file` 与 `bytes.replace`）：基准 `e1b1ceb95` 里 `scripts/experiments` 下 395 份，落点上在 395 份，字节原样相同 290 份，整串替换之后相同 362 份；库算 360 份。差的 2 份是上面 2.4 说的边界字符（`-`、`t` 前面）：库比独立写法严。方向上与库一致，但独立写法与库共享同一个「相同」的定义（整串替换），所以不是定义上独立的对照。
- 「726 份照剔」：批 4 报告里的「改前剔 726、改后剔 726，两边差集都是空的」比的是同一个函数的新旧两版；旧版正是第一轮被打中的那一版，「新版没有比旧版少剔」说不了「剔的是对的」。本腿的独立写法（输出 `out/probe_real.out`）：工作区此刻改动范围 6039 份，库剔 1066，独立写法剔 1073，库有独立没有 0 份，独立有库没有 7 份（都是 2.4 说的边界字符那类）。数比批 4 当时的 726 大，是工作区在变（别的会话在改）；本腿没有拿到批 4 当时的现场，不能复核 726 本身。可以说的只有：此刻的库没有多剔独立写法认为不是只差搬迁的文件。

### 3.2 样本只覆盖被打中的形状吗：是

- 第一轮攻方与正推腿的样本、批 4 的新样本，都围绕被打中的那几个形状造（登记表加一行就能洗、指历史的写法、按次数抵、倒换认半搬、搬进管辖类）。本腿造的新形状里放过的有：r0、r2、r3、r4（整个换实现或清空）、t1（换次序再另起中间目录）、c1 与 c3（跨管辖类的新增行）。
- 其中 r0 一类批 4 工具员自己造过：自证里「删掉旧目录换一份新实现（0 份相同）」判红（批 4 报告「甲」一节），主 agent 收的时候取消了门槛，自证改成了断言放行：「and rewritten_whats == []」（`.claude/gate.d/lib-lag-baseline.py:1254`）。自证和判法出自同一批改动，只能证「判法是写的人要的样子」，证不了「要的样子是安全的」。
- 弄坏开关：批 4 报告写九个开关逐个打开都红在点名的那一项。这些变异都是「把检查整个关掉」，没有「检查变弱」一类（例如把落点逐份在改成任一份在、把无序键改成有序键）。本腿没有造这类变异跑自证，所以不能说自证能分辨这些。
- 本腿自己的限度：样本是同一个人造的，「过半」「字面」两列是按独立数现算，没有拿改过的库跑；r1 的「垫」只用了 6 份的小目录，没有在 395 份上造。

推翻这一节：造出一份弱化了的库（例如把 `if missing:` 换成只要一份在），它仍让 `--selftest` 26 项全过且各弄坏开关照红，说明自证确实分辨不了「变弱」；反过来，若现有自证里已有一项在这种弱化下红，就说明本节低估了。本腿没做。

## 四、H3 没看过的地方

### 4.1 `lib-lag-baseline.py` 的调用方

库文件头列了九处调用方（`.claude/gate.d/lib-lag-baseline.py` 文件头「调它的九处」一段），仓里现查调 `baseline_text`、`baseline_rows`、`baseline_file_bytes` 的行：`.claude/gate.d/checker-independence-and-sync.sh:682`、`.claude/gate.d/doc-decisions.sh:3471`、`.claude/gate.d/doc-experiments.sh:1109`、`:2173`、`:2897`、`.claude/gate.d/doc-process-records.sh:892`、`.claude/gate.d/code-source-discipline.sh:1595`、`:2096`、`.claude/hooks/lib_long_run.py:440` 与 `:474`。这些调用拿到的是同一种三结局（有表、没有表、git 取不到），库的自证里有对应的项：「搬迁登记：按搬迁之前的路径取表」（`:1034`）、「搬迁登记：写坏的登记表交问题句」（`:1058`）、「搬迁登记：照登记的次序串起来」（`:1051`），本腿在主工作区原样跑 `python3 .claude/gate.d/lib-lag-baseline.py --selftest`：退 0，末行「✓ lib-lag-baseline.py 自证：26 项都过」，`✗` 行 0 条。

这一节只做到读调用写法、跑库自证，没有对九处各造一个取不到基准、表自己搬了家的现场（调用方多数不传 `cwd`，靠当前目录是被判的根；`doc-experiments.sh:1109` 与 `lib_long_run.py` 传了 `cwd=root`）。没有发现判错的，也不能说各处都验过。

甲只在门禁 code-tooling 的 change-range-single-source 格里被调用（`path_move_report`）；九处调用方用登记表换算时都信任它，信任的依据就是那一格判过实据。实据只在「被判的根是仓根」时核，gate 基准取法是：`GATE_BASE` 有就用它，否则 `@{upstream}` 的 merge-base，再否则 `HEAD`（`scripts/gate/changed-paths.sh` 的 `gate_diff_base`，约 106–125 行）。基准推进到搬迁提交之后，新加的行就是 0，实据不再核（`probe_rows` t0 的实据句）；这是设计，不是缺口，但它意味着实据是提交那一刻的一次性判断。

### 4.2 `gate_added_lines` 剔行之后依赖它的格；跨管辖类时照旧剔行

`gate_added_lines` 的调用方有 doc-process-records（`records/*.md`、`research/prompts/*-sync.md` 的新增行）、doc-experiments（`.claude/kb`、`research/**/*.md`）、doc-decisions（kb）。它们要挑的是自己路径下的新增行。

戊（`moved_only_paths` 对跨类整份搬家不剔）：「return governed_kept and old_path != new_path and governed_class(old_path) != governed_class(new_path)」（`.claude/gate.d/lib-lag-baseline.py:541`）只在路径层。新增行层 `drop_moved_lines` 对整份搬了家的文件，没动的行都可剔：「return {number for number, (old, new) in enumerate(zip(before_lines, after_lines), 1) if relocated or old != new}」（`.claude/gate.d/lib-lag-baseline.py:692`）。`probe_h3`（单文件登记，`out/probe_h3.out`）：

```
c1 .claude/rules/draft.md → records/2026-10-23-draft.md：登记表问题=[]；改动范围剔掉=[]；新增行共3条留下0条
c3 .claude/rules/draft.md → .claude/kb/draft.md：          登记表问题=[]；改动范围剔掉=[]；新增行共3条留下0条
c2 同一类里改名：                                          改动范围剔掉=[两个路径]；新增行共3条留下0条（预期）
c0 没登记的对照：                                          新增行共3条留下3条
```

这与第一轮 G3-1 同一个形状（搬进别的类），只是落在新增行层：一份内容先放在不被 records、kb 的行级格查的位置，再用单文件登记搬进 records、kb，整份行都不进新增行。批 4 报告点名了这一点（「跨管辖类搬家时 drop_moved_lines 照旧剔行」），第一轮判决没有点。严重程度我没量：要搬的内容得先在别处提交过一次，那一次不会被 records、kb 的行级格看到。

### 4.3 `.claude/rules/path-moves.md`

- 开头一句仍无例外：「不留一处旧路径」（`.claude/rules/path-moves.md:6`）；第 4 步已改成「除第 5 步留的之外必须 0 命中」（`.claude/rules/path-moves.md:15`），第 5 步留旧路径（`:16`）。己改了第 4、5 步，没改开头这一句，规则里的原则句和做法句并排矛盾。
- 第 2 步「换了实现的不登记，照真改动交审」（`.claude/rules/path-moves.md:13`）是一句靠人守的话，库里没有判它的东西（2.1），也没写成「门禁不判」。第 56 行「门禁管哪一半」只写了门禁判的几样，没有声称判这一句，不是假话，但读的人容易以为实据判了它。
- 第 5 步与丙的冲突见 2.4。

## 五、收严还是放宽（对判决定的改法）

| 改法 | 对改前 | 对判决原文 | 说明 |
|---|---|---|---|
| 甲（主 agent 改后：在、旧有文件、新前缀原先没有） | 收严（原来没有实据） | 放宽：判决写每份换算之后相同：「基准里旧前缀下的每份文件此刻在新前缀下有换算之后相同的一份」（`research/prompts/apparatus-rename-r1-main-verification.md:31`），现在相同份数只报不判（`.claude/gate.d/lib-lag-baseline.py:768`） | 比批 4 的过半也宽；r0、r2、r3、r4 放过 |
| 去掉第三个豁免分支 | 收严 | 一致 | 攻方 G1-1 现状红 |
| 乙 | 收严 | 一致 | 误判方向多判 |
| 丙 | 收严 | 一致 | 潜在冲突，无实例 |
| 丁 | 收严 | 一致 | |
| 戊 | 收严（路径层） | 一致 | 新增行层未收 |
| 己、庚 | 句子层收严 | 一致 | 两处残留见 4.3 |

## 六、没做什么

- 不替主 agent 撤销或维持判决，只交复核结果与可选改法（2.6），可选改法都是零轮、没量代价。
- 没改库、没造改过的库；「判决字面」「过半」两列是按独立数出的相同份数现算的结论，没有拿对应版本的库跑。
- 没做弱化变异（检查变弱）的自证实验；没对九处调用方各造现场；没单独攻 diff 解析（多块、重名行）；没跑 code-tooling 的样本档（要暂存）；没量真仓里多行链式登记的数量（现在表里只有一行）。
- 没复核批 4 当时的 726 份（工作区已变）；没跑 E171 `reuse_library.py --selftest`、`changed-paths.sh --selftest`、`lib_long_run.py exempt-check`（主 agent 正文里的观测，本腿没重跑）。
- 重型测试不跑。样本全是临时 git 仓里的 python，没起 cargo、没编译。
- 草稿目录 `~/.local/share/singlefs/work/apparatus-rename-r2-defense/` 里的 `attack_r1_now.out`、`h1.out`、`rows.out`、`h3.out`、`real.out`、`selftest*.out` 不入库：前四份的内容已落进模型目录 `out/`，`attack_r1_now.out` 是攻方第一轮样本现状的 23 行，这里只引了结论。

## 草稿目录清理

交回前删掉自己建的冻结树与样本现场（删之前 `du -sh`）：`frozen` 208K、`frozen-full` 4.8M、`frozen2` 208K、`h1` 4.8M、`rows` 2.5M、`h3` 1.3M、`rerun1` 24K、`attack-out` 3.0M，共八份；留下只有几 KB 的 `*.out` 日志。
