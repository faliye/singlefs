# apparatus-rename-r1 正推腿（Sonnet）报告

轮名 apparatus-rename-r1，格 G1–G4。立场：从脚本源码与规则正推，每一处换算做的是不是登记表与规则说的、几处是不是同一套。
方案按 `crates/` 与脚本今天的实现谈（`grep -rn 'path-moves\|path_moves\|rewrite_moved' crates/` 零命中，见「没做什么」）。
被判文件开工时与快照一致：`cd <仓根> && sha256sum -c research/prompts/apparatus-rename-r1-snapshot/judged-sha256.txt` 全 OK（14 行）。
没读禁读清单里的攻方腿报告与模型；没读三份工具员报告（只读代码）。

## 复跑

模型目录 `research/prompts/apparatus-rename-r1-sonnet-model/`，一条命令复跑（整条经内存包装，`--work` 给新目录）：

```
bash scripts/hosts/run-with-memory-cap.sh 4G bash research/prompts/apparatus-rename-r1-sonnet-model/tools/rerun.sh \
  --repo <仓根> --frozen <仓根>/research/prompts/apparatus-rename-r1-snapshot --work <新目录>
```

跑完应打 `name=compare_g1 same=yes`、`compare_g2`、`compare_g3` 各一行（样本产物与 `out/samples_g*.out` 逐行相同）；
`out/real_scan_first_run.out` 读的是活仓，不进逐行比。`--frozen` 这里只用来核被判文件的 sha256，样本读 `--repo` 里的读法库，一个字节不改。
样本都在 `--work` 下的临时 git 仓里造，不碰主树。每个文件的 sha256（`find . -type f | sort | xargs sha256sum`，在模型目录里跑）：

```
11729ec9f7e98062c7d4c604dcddccc86a2bf5072b43ad900a7230443b29d6e7  ./out/changed_paths_selftest.out
d9b878370af347d13d248f4c16e9017bebb98b34879c53c47b635adf2f5be9bd  ./out/compare_frozen_o3.out
8c80e9e83bf4dafcc1071d8966c1e675c06a9d8ccf2414d56b31cb63817811a2  ./out/lag_selftest.out
70e62836651f01a5b86238797bd25d75c138a42e8a32ee21ad32e4f4d6902c3e  ./out/real_scan_first_run.out
2932dabae9ab5bf9ded0b315a674af4b7882051608df8317ba546ab49d343fec  ./out/reuse_library_selftest.out
1b867f2cdf50aa15348fd81a02ad3c7de4367b9281f4551da3bc87d241863947  ./out/samples_g1.out
9a7c9c266454e08ac4c6762e606ff3abdd934f837f0e754ec2ba25eab64dccfd  ./out/samples_g2.out
d5e8d29e1c8244585d039771ece56e7464c5ea5ba402681366c236372d28dc0c  ./out/samples_g3.out
ee79b12e379f395aaa88980e0363134d44670efff409f9c60b288034fd2bdcda  ./tools/common.py
0fb5a678f945faa0ceed32f47b262a8646f7369e2080b3cc66eb7b906deba26b  ./tools/compare_frozen_o3.py
e8bef1f5a99b233bd5a1445a11084954b8693e7e1e72001aeebe098ee2ab06df  ./tools/real_scan.py
086d2a21a5305a9b387293b0d0745a4fea13ac09e1ee768a6524cf4299fbba97  ./tools/rerun.sh
3b01354f2c9c41981d3031b8a83cf102fe90bbc6e49cbff87a1039860e55bb5b  ./tools/samples_g1.py
08e3738986155b25e802b02099bb7d43575c098a78ed10af04ad9e3ceac4ff7c  ./tools/samples_g2.py
f8f6332dcb08390a579962a0fcff49bed4ec1012329954c8995643aad7825354  ./tools/samples_g3.py
```

现跑的自证（主 agent 观测的复核，原样输出在 `out/`）：

- `python3 .claude/gate.d/lib-lag-baseline.py --selftest` 末行「✓ lib-lag-baseline.py 自证：22 项都过」，退 0。
- `bash scripts/gate/changed-paths.sh --selftest` 退 0，打「19 项」。
- `python3 scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py --selftest` 末行「自证 43 项，红 0」，退 0。
- `python3 .claude/hooks/lib_long_run.py exempt-check . --base HEAD` 退 0，报 6 行照搬迁换算后与基准逐字节相同。
- `python3 .claude/gate.d/lib-lag-baseline.py path-moves-check .` 退 0，「1 行：写法对、新前缀在磁盘上、旧前缀不在了、出处都在」。

## 总判

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 误放 | 冲突 | 这一批那一行真登记本身没洗出别的东西；但登记表判法只核三件事，一行假登记就能把「改指到另一个已有目录 / 转移排除表一行 / 复制一行」洗成只差搬迁，样本复现 |
| G2 读法一致 | 部分一致 | 正着换、倒着换、冻结层两份用同一个换算函数，边界对称；路径一级与文本一级、正着比与倒着比、多次搬迁的取法、写坏登记表的反应各有不一致，全部落在「多算改动」或「多认前缀写法」，没找到同一内容一处判同一处判异而放过真改动的 |
| G3 改动范围剔除 | 部分冲突 | 上游 show-me-test、i18n-sync 不经这一层，仍看得见；剔除不分注释与代码；agent 定义里只差路径的一份（mutation-triage）门禁不再要求三方，而本轮被判集合里也没有它 |
| G4 句子 | 部分冲突 | 三处句子与脚本做的事对不上之处：基准里被钉脚本的 sha256 条件没写、剔除改动范围没写、拆开写的写法没写；两张表头仍写「新路径算涨」 |

以下各节：每格先写结论与推翻条件，再写依据。

## G1 误放

结论：冲突。「换算之后逐字节相同就不算改动 / 不算涨 / 认钉值」这一层本身做的是登记表说的那件事（边界规则、提交引用、次数抵、按搬迁之前的路径取，见 G2）；
但它信任登记表，而登记表的判法只核三件事，核不出「这一行登记对应一次真搬迁」。一行假登记就能洗。样本全部在 `out/samples_g1.out`。
这一批那一行真登记（`scripts/experiments` → `scripts/experiment-apparatus`）本身没洗出别的东西：基准里新前缀串 0 处（`git grep -l 'experiment-apparatus' HEAD | wc -l` 打 0，旧前缀 1083 份），正着换是单射。

推翻它的现象：在 `path_move_problems` 里（或登记表旁）加一道核「旧前缀在基准树里有文件、新前缀下对应文件在此刻存在且换算后相同」之类的判，并让下面样本 A1、A2、A3、A6、A7 变红；或者登记表有另一道「只增要问用户」的守卫。我没找到这样的守卫。

### 登记表判法只核三件事（正推）

登记表判法 `path_move_problems`（`.claude/gate.d/lib-lag-baseline.py:616`）逐行核：

- 「if not os.path.lexists(os.path.join(root, resolved)):」（`.claude/gate.d/lib-lag-baseline.py:635`）新前缀在磁盘上；
- 「if os.path.lexists(os.path.join(root, move.old)) and not moved_back:」（`.claude/gate.d/lib-lag-baseline.py:640`）旧前缀不在磁盘上；
- 「if not os.path.isfile(os.path.join(root, move.source)):」（`.claude/gate.d/lib-lag-baseline.py:643`）出处是一份存在的文件。

写法侧只禁空白、开头 / 与 ./、结尾 /、空段与 . .. 段（`.claude/gate.d/lib-lag-baseline.py:188`）。没有读 git：不核旧前缀在基准树里是否有文件、新前缀下是否有对应文件、出处里是否写了这次搬迁、日期是不是真的。
门禁里调它的只有一处：「move_count, move_problems = lag_baseline.path_move_problems('.')」（`.claude/gate.d/code-tooling.sh:1603`）；
同一格的红样本五行各错一样（旧前缀还在、新前缀不在、日期、三列、出处不在），没有一行是「写得对、指不对」。登记表本身不与基准比（不是只缩不涨的表）。

### 样本（实测，`out/samples_g1.out`）

| 样本 | 做法 | 结果 |
|---|---|---|
| A1 排除表 | 基准里 `.claude/heavy-thread-exempt` 有 `scripts/gone/runner.py:os.cpu_count()` 一行；删掉 `scripts/gone`，表里这一行改成已有的 `scripts/gate/runner.py:os.cpu_count()`；登记 `scripts/gone` → `scripts/gate`，出处是一份只写「# 样本」的 records 文件 | `path-moves-check` 退 0（「写法对、新前缀在磁盘上、旧前缀不在了、出处都在」）；排除表按 code-tooling 的比法（路径:锚串键）「涨了」的集合：不登记时 `['scripts/gate/runner.py:os.cpu_count()']`，登记后 `[]`。一条本不该放行的排除（`scripts/gate/runner.py` 真有 `os.cpu_count() * 2`）免费进表 |
| A2 改动范围 | `crates/x/src/lib.rs` 里 `"scripts/gone/run.sh"` 改成 `"scripts/gate/run.sh"`（两个都是已有的别的目录） | `moved-only-filter` 不登记时留下该文件，假登记后留下 `[]`；`moved-lines-filter` 同样把新增行剔成 `[]` |
| A3 单个词 | 登记 `deny` → `scripts`（`deny` 不在盘上、`scripts` 在） | 写法判得过（退 0）；`hook.sh` 里 `exit deny` 改成 `exit scripts` 被剔，`kept=[]` |
| A6 复制一行 | 基准 `docs/a.md` 有 `bash scripts/experiments/run.sh`；此刻原样留着这一行，再多写一行换成新前缀的拷贝 | `moved-lines-filter` 把多出的那一行剔掉（`added_line_kept=[]`）：`drop_moved_lines` 按次数抵「基准里因搬迁才变了的行」，不核基准那一行此刻是否真被换掉 |
| A7 旧前缀在盘上 | 两行登记：`scripts/gate` → `scripts/hosts`（`scripts/gate` 此刻在、还有真文件），再 `scripts/zzz` → `scripts/gate/zzz` | 退 0，且成功句仍写「旧前缀不在了」；`docs/a.md` 里 `scripts/gate/run.sh` 改成 `scripts/hosts/run.sh` 被剔。例外来自 `moved_back` 的写法（`.claude/gate.d/lib-lag-baseline.py:639`）：后面任何一行的新前缀是它的子路径或父路径，就不再要求旧前缀不在 |

A3 说明：前缀不必是目录名，任何不含空白与 . .. 段的串都过。新前缀必须是盘上已有的路径这一条，限制了能换成什么，没限制能换掉什么。

### 按「路径 + sha256」钉的表（lib_long_run exempt-check）守得住大半，守不住字节相同的拷贝

重钉那一支的条件在 `moved_script_unchanged`：基准那一版的脚本按搬迁之前的路径取，
「if hashlib.sha256(data).hexdigest() not in base_sha256s:」（`.claude/hooks/lib_long_run.py:442`）它的 sha256 要是基准那一版表里登记的，
再「if lag_baseline.rewrite_moved_bytes(data, moves) == current:」（`.claude/hooks/lib_long_run.py:449`）换算后与此刻逐字节相同才放行。

- A4（实测）：基准里 `scripts/old-rig/live.sh` 有一行，删掉它，表里改成已有的另一份 `scripts/other/live.sh`（正文不同），登记 `scripts/old-rig` → `scripts/other`：`exempt-check --base HEAD` 退 1，红句点名「基准里的 scripts/other/live.sh 的 sha256 … 不是基准那一版表里登记的，没法当成被钉住的那一版去比」。sha 钉挡住了。
- A5（实测）：同上，但 `scripts/other/live.sh` 与旧脚本字节相同（基准里就是它的一份拷贝、没有豁免行）：退 0，「与基准那一版比没涨」。豁免从被删的脚本转给了字节相同的另一份。

所以排除表（只比路径:锚串）、改动范围与新增行（比文本）都没有 sha 这一道，A1、A2、A6 能洗；有 sha 的豁免表只在「内容字节相同的拷贝」这一窄缝上被洗。

### 倒换（reuse_library / frozen_o3）不放过真改动，但不是单射

倒着换的实现：「backwards = [move._replace(old=move.new, new=move.old) for move in reversed(moves.moves[-count:])]」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:445`），
冻结层是「backwards = [move._replace(old=move.new, new=move.old) for move in reversed(self.moves)]」（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:68`）。
它只动新前缀（边界同 G2），所以「换回去之后逐字节等于钉值」的集合是：钉住那一版里每一处旧前缀写法，此刻随意是旧还是新。

- 实测 B2/B6（`out/samples_g2.out`）：钉住的内容含两处旧前缀，全搬了、只搬一处、一处没搬三种写法，倒着换出同一个 sha256，`pinned_sha256_now` 三种都认（`fully-moved=True half-moved=True not-moved=True`）。
- 此刻内容里本来就有的新前缀串被换走：对「搬迁之后才钉」的值（`ORCHESTRATOR_SHA256`、`SEGMENT_TWO_ORCHESTRATOR_SHA256`，`reuse_library.py:416`、`:417`）不会误放：先比 `actual == pinned`（`reuse_library.py:428` 起的函数头），不等才倒换，倒换后要等于钉值，钉值里有新前缀串时倒换后对不上，拒。
- 结论：两份不同的文件（全搬 / 半搬）换出同一份，是真的；它们只差登记的前缀，所以不放过「除了前缀还改了别处」的真改动（`out/samples_g2.out` 里 `b6-exempt-check variant=half-moved` 对应的另一路是红的，见 G2）。残余风险是半搬的文件（还指着已不存在的旧目录）被当成钉住的那一份载入执行，坏的是运行不是完整性。

两份重钉的值：`rows_orchestrator.py` 现 sha256 `b48bf322…`，`rows_orchestrator_segment_two.py` 现 sha256 `7dda75bc…`，与 `reuse_library.py` 的两个常量逐字相同（`sha256sum` 现算）；
重钉的理由（假仓函数里拆开写的路径也换了，不只差前缀）写在 `reuse_library.py:414` 的注释里，是人工重钉，不是换算认出来的；这一步靠 reuse_library 43 项自证与人审，本腿复核不了重钉前后的语义等价。

补 G1：门禁与钩子里提到 `path-moves.tsv` 的文件只有 `code-tooling.sh`、`doc-decisions.sh`、`doc-experiments.sh`、`lib-lag-baseline.py`、`lib_long_run.py` 五份
（`grep -ln 'path-moves.tsv' .claude/gate.d/*.sh .claude/gate.d/*.tsv .claude/gate.d/*.py .claude/hooks/*.sh .claude/hooks/*.py`），doc-decisions 与 doc-experiments 里只是注释；没有一处拿登记表去与基准比。

## G2 读法一致

结论：部分一致。换算的核心是同一个函数，边界规则对称；其余几处读法有出入，出入的方向都是「多算改动」或「多认前缀写法」，我没造出一个「一处判同、另一处判异，且放过了前缀之外的真改动」的内容。

推翻它的现象：造出一份内容，使 `moved_only_paths` / `drop_moved_lines` / `exempt-check` / `pinned_sha256_now` / `freeze_differences` 里有一处判「只差登记前缀」、另一处判「还改了别处」，且别处确是前缀之外的改动（半搬那一组是前缀之内的差，不算）。

### 同一套的部分（正推 + 实测）

- 正着换只有一个实现：「def rewrite_moved_paths(text, moves):」（`.claude/gate.d/lib-lag-baseline.py:249`）。
  `baseline_text`、`drop_moved_lines`（`rewrite_moved_paths(before, moves)`，`:600`）、`moved_only_paths`（`return data == before or data == rewrite_moved_bytes(before, moves)`，`:550`）、`lib_long_run` 的「if lag_baseline.rewrite_moved_bytes(data, moves) == current:」（`.claude/hooks/lib_long_run.py:449`）都调它或它的字节版。
- 倒着换不另写实现：冻结层与 reuse_library 把登记行的新旧前缀对调后，调同一个 `library.rewrite_moved_bytes`（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:68`、`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/reuse_library.py:445-446`）。
- 边界规则两个方向对称。`out/samples_g2.out` 的 `b1-forward` / `b1-backward-of-mirrored` 各 26 种写法，逐种「换了没有」与「换回去没有」同值（脚本现数：不对称 0 种）。
  边界本身：「PATH_BEFORE_BOUNDARY = r"(?<![A-Za-z0-9_.\-])"」（`.claude/gate.d/lib-lag-baseline.py:130`）、「PATH_AFTER_BOUNDARY = r"(?![A-Za-z0-9_\-]|\.[A-Za-z0-9_\-])"」（`.claude/gate.d/lib-lag-baseline.py:131`），
  提交引用：「COMMIT_REFERENCE_BEFORE = re.compile(r"(?<![0-9A-Za-z_])[0-9a-fA-F]{7,40}:(?:\./)?$")」（`.claude/gate.d/lib-lag-baseline.py:133`）。
  26 种里 15 种换（普通、`./`、`$ROOT/`、绝对路径、句号收尾、引号、`+` 行首、6 位十六进制冒号、41 位和 64 位十六进制冒号、紧挨字母的十六进制冒号、一行两处、新旧并存等），
  11 种不换（`experiments2`、`experiments-old`、`experiments.md`、`-` 行首、`:-` 之后、`\n` / `\t` 之后、拆开写的 `"scripts", "experiments"`、7 位及以上十六进制加冒号的提交引用两种、`defaced:` 这类恰好全是十六进制字母的英文词）。
- E171 两份 `frozen_o3.py`：读搬迁登记的 6 个函数（`RegisteredMoves`、`path_moves_library`、`registered_moves`、`sha256_before_moves`、`matches_the_freeze`、`seed_from_directory`）正文逐字相同，`freeze_differences` 不同（`out/compare_frozen_o3.out`）。
  不同处只在取冻结行的办法（一份读固定行号段，另一份缺产物时抛 `FreezeReferenceMissing`）与种子比较拆成两步的写法，搬迁比较的调用 `matches_the_freeze(…, seed_from_directory(directory, registered), changes.get(SEED_NAME))` 相同。
- `lib_long_run`、`doc-decisions`、`doc-experiments`、`changed-paths.sh` 的 shell 判断都落到 lib-lag-baseline.py 这一份，没有第二份换算。shell 那一层只多一个「有没有内容行」的判断：「[[ -f "$table" ]] && grep -qv -e '^[[:space:]]*#' -e '^[[:space:]]*$' "$table"」（`scripts/gate/changed-paths.sh:92`），与 `content_rows` 同判（空白行与行首 `#`）。

### 不一致处（逐条：现象、方向、实测）

1. 路径一级只认仓根起的前缀，文本一级认嵌在别的目录底下的同名段。
   路径一级：「if path == source or path.startswith(source + "/"):」（`.claude/gate.d/lib-lag-baseline.py:282`），`paths_before_moves`、`forward_path`、`moved_only_paths` 用它；文本一级：`b1-forward` 的 `nested-other-dir`（`tests/fixtures/scripts/experiments/a.sh` 被换）与 `absolute`。
   实测 B3：夹具目录 `fixtures/g/scripts/experiments` 改名后，里面的 `replay.sh` 路径一级认不出（`kept_as_changed=['fixtures/g/scripts/experiment-apparatus/replay.sh']`），`docs/a.md` 里指它的文本被剔。方向：改名的嵌套夹具多算改动；反过来，文本里指向别的目录的同名段也被当成只差搬迁（见 G1 补充，规则没限定「只换仓根起的前缀」）。
2. 正着比（逐字节等于换算后的基准）与倒着比（倒换后等于钉值）认的集合不同。`lib_long_run` 只认全搬；`pinned_sha256_now` 全搬、半搬、没搬都认。
   实测 B6：同一份「只搬了一处」的脚本，`pinned_sha256_now` 认（`half-moved=True`），`exempt-check` 红「仍不逐字节相同（除了登记的路径还改了别处，不算搬迁）」。红句的说法对半搬不准确：半搬的差仍只是登记的前缀。方向：钉值核对更宽，豁免表核对更严，差的都是前缀之内。
3. 多次搬迁取几档不同。reuse_library 逐次多倒换一次（`for count in range(1, len(moves.moves) + 1):`，`reuse_library.py:444`）；冻结层一口气倒换全部（`scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/frozen_o3.py:68`）；`baseline_text` 一口气正着串起来换，取文件逐次倒回（`paths_before_moves`，`.claude/gate.d/lib-lag-baseline.py:287`）。
   今天登记只有一行，三者同判；`b4-chain` 样本两行登记（先 `scripts/b` → `scripts/c`，再 `scripts/a` → `scripts/b`）正着串的结果 `scripts/a/x scripts/b/y` → `scripts/b/x scripts/c/y`，`paths_before_moves('scripts/c/y.sh')` 倒回 `scripts/b/y.sh`，次序按登记表的行序，不按日期列。
   两行以上时冻结层只认全倒换一档，钉在两次搬迁之间的值认不了；冻结层钉的是文件头写明取自更早产物的内容，今天够用，写进去的是「以后会不一致」而不是「今天不一致」。
4. 登记表写坏了的反应不同。`baseline_text`（`.claude/gate.d/lib-lag-baseline.py:306`）交 `PathMovesProblem`，调用方判红；冻结层抛 ValueError、reuse_library 抛 RuntimeError（退 3），也是红；
   `moved-only-filter` / `moved-lines-filter`、doc-decisions 的 `without_moved_lines`、doc-experiments 同款函数则打一行提示，按没登记交回：「print(f"  ! {problem.sentence()}；{what}没剔除只差搬迁的，照没有登记时交回", file=sys.stderr)」（`.claude/gate.d/lib-lag-baseline.py:736`）。
   实测 B5：同一份三列的表，`baseline_text` 交 PathMovesProblem，`moved-only-filter` 退 0 并把 `a.txt` 原样留下，shell 那一层的 grep 认它是内容行。方向：放行的一侧是「不剔除」，多算改动。
5. 被判的根不是仓根时，换算照做、剔除不做：`top_level, problem = at_top_level(cwd)`（`.claude/gate.d/lib-lag-baseline.py:442`）。文件头写明了这一点，两处一致，是有意的。
6. 基准里故意留旧路径的行，正着换会把它换成新前缀。提交引用（`git show 31e47115:scripts/experiments/…`）有守卫，「讲改名本身」的行没有。
   实测 B7：基准里一行「记改名：scripts/experiments 改名为 scripts/experiment-apparatus」，登记表还在，`baseline_text` 交回「scripts/experiment-apparatus 改名为 scripts/experiment-apparatus」，与此刻没动的原行不等。
   今天基准是改名之前，不触发；改名提交落地之后基准里就有这类行（`.claude/rules/path-moves.md@怎么做` 第 5 步要求保留），任何对基准文本逐行比的格（doc-decisions 的 history-loss、各挂账表）会拿到被换过的那一行。
   登记表的行没有寿命条款：规则与表头都没说登记行什么时候可以删。这条本腿只能用样本模拟，落地后的真仓复核不了。
7. 与主 agent 观测对不上的一项：观测写「diff 行首 `-` `+` 的写法也换了，而 `moved_prefix_pattern` 不认这几种」。实测 `b1-forward case=diff-plus replaced=True`：行首 `+` 库认（`+` 不在 `PATH_BEFORE_BOUNDARY` 的排除字符里），库不认的只有行首 `-`。
   其余四种（拆开写、`\n` `\t` 之后、`:-` 之后）库确实不认，与观测一致。改名用的替换脚本在会话草稿里，本腿没读到，复核不了它的写法。
8. 库不认的写法造成的「多算改动」，真仓实数（`out/real_scan_first_run.out`，活仓，只对那一次跑）：改动范围 5406 条，判成只差搬迁剔掉 726 条；`scripts/experiment-apparatus/` 下仍算改动的 42 条里，5 条的差只剩含新前缀的行（库不认的写法），30 条的差里还有前缀之外的内容（样例 `README.md`、`Cargo.toml` 的注释行，别的会话的改动；本腿没逐条核归属），7 条是基准里没有的新文件。

## G3 改动范围剔除的副作用

结论：部分冲突。剔除按文件内容判，不看文件是什么、路径改在哪：依赖改动范围的格看不见任何「与基准只差登记前缀」的文件与行，包括 agent 定义、钩子、门禁脚本、`crates/` 源码里改在代码里的路径。
上游共享的 show-me-test 与 i18n-sync 不经这一层，仍看得见；登记表与读法库这两份「搬家本身」的文件仍在范围里。

推翻它的现象：找到一个依赖改动范围的格，对只差登记前缀的文件原本会产出红或要求某个产物（回看、判决、同步记录），而剔除之后它放行并且该要求没有别的出处；下面「冲突」两条是我找到的。

### 谁读改动范围（代码）与剔除的效果

| 读的格 | 取的方式 | 只差搬迁的文件 / 行 | 本腿判 |
|---|---|---|---|
| doc-process-records 的 batch-scope | 「if changed_since_head="$(gate_changed_paths "$head_base" untracked)"」（`.claude/gate.d/doc-process-records.sh:466`） | 触发文件不用登记；登记了的会被判「登记项不起作用」：「stale = [path for path in registered if path not in triggers]」（`.claude/gate.d/doc-process-records.sh:538`）。今天 `.claude/batch-scope` 54 行里落进剔除集合的 0 行 | 设计目的，一致 |
| knowledge-sync | 同文件 `changed_since_gate` | 触发文件表有一行「研究与协作脚本」（`.claude/gate.d/knowledge-sync-triggers.tsv:15`）覆盖整个 `scripts/`，被搬走的几百份不进同步范围；登记表（`.claude/gate.d/*.tsv`）和读法库 `.py` 本身是触发文件，样本 C1 里 `path-moves.tsv` 仍列出 | 设计目的，一致；搬家本身的文件仍被看见 |
| crates-adversarial-review | 「all_sources="$(grep -E '^crates/[^/]+/src/.*\.rs$' <<<"$changed_since_gate" || true)"」（`.claude/gate.d/doc-process-records.sh:880`） | 路径改在字符串字面量里的 `.rs` 同样剔掉；该格对只改注释的文件另有判法，剔除比它宽 | 今天 `out/real_scan_first_run.out` 里被剔的 7 份 `crates/` 文件 `non_comment_lines=0`，全是注释，所以今天没有真放过；机制上不分 |
| agent-def-adversarial-review | 「definitions="$(grep -E '^\.claude/(agents/[^/]+\.md|agent-common\.md|main-agent\.md)$' <<<"$changed_since_gate" || true)"」（`.claude/gate.d/doc-process-records.sh:1744`） | 只差路径的 agent 定义不要求判决点名 | 冲突，见下 |
| doc-text 的 date-once、trilingual changed-together、private-values、out-of-project-names、utf8-valid 等 | 「changed="$(gate_changed_paths "$base" untracked)"」（`.claude/gate.d/doc-text.sh:847`）等六处 | 旧行已在基准上判过，只差路径的不再扫。新前缀串本身出现在登记表那一行里，登记表是改动范围里的文件，样本 C1 的 `gate_added_lines` 把 `.claude/gate.d/path-moves.tsv` 那一行交出，所以前缀里夹带的东西仍被扫 | 一致 |
| doc-experiments、doc-decisions 的新增行 / 历史节 / entry-added | 经 `gate_added_lines` / `drop_moved_lines` | 实验页「影响的决策」回看少 9 页（主 agent 观测，本腿没复跑该格） | 设计目的 |
| show-me-test.sh、i18n-sync.sh（上游共享） | 「changed_file_text="$(changed_files "$PROJECT_ROOT" "$DIFF_BASE")"」（`.claude/singlefs-ai-sop/scripts/show-me-test.sh:23`）；两份里 `grep -c 'path-moves\|lag-baseline'` 都是 0 | 不经 `changed-paths.sh`，不剔 | 仍看得见 |

### 实测（`out/samples_g3.out`，一个临时仓，登记 `scripts/experiments` → `scripts/experiment-apparatus`）

被剔（`seen_as_changed=False`）：整目录 `git mv` 后只改了路径的脚本、`.claude/agents/triage.md`、`crates/x/src/lib.rs` 里 `let p = "…/run.sh"` 的字符串字面量、`.claude/hooks/guard.sh` 里 `exec bash …/run.sh`、`docs/note.md`。
留下（`seen_as_changed=True`）：搬家又改了别处的、只改了模式的、真删掉的（老路径那一条）、基准里没有对应文件的新文件、暂存的是只差搬迁而工作区又多一行的、内容等于换算后的基准却换了名字的未跟踪拷贝、登记表本身。逐格与 `moved_only_paths` 的口径一致：路径逐条配对，不做内容相似度，暂存区和工作区都要只差搬迁。

### 冲突 1：agent 定义里只差路径的一份，门禁不再逼三方，本轮也没判它

调度记录写「用户在弹窗里定了…改到的 agent 定义与共用约束走一轮三方」（`records/2026-10-23-实验装置目录改名.md:9`），条 3 点名的是「`.claude/agents/` 下 3 份、`.claude/agent-common.md`、`.claude/main-agent.md`」（`records/2026-10-23-实验装置目录改名.md:15`）。
现查提到新前缀的 agent 定义正是 3 份：`experiment-designer.md`、`experiment-runner.md`、`mutation-triage.md`（`grep -l experiment-apparatus .claude/agents/*.md`）。
被判集合 `research/prompts/apparatus-rename-r1-snapshot/judged-sha256.txt` 14 行里一份 `.claude/agents/` 都没有，只有 `agent-common.md`、`main-agent.md`、`rules/path-moves.md`。
`mutation-triage.md` 与 HEAD 相比只差两行里的路径（`git diff HEAD -- .claude/agents/mutation-triage.md`），`moved_only_paths` 判它是只差搬迁（现跑：`DROPPED .claude/agents/mutation-triage.md`），所以 agent-def-adversarial-review 不会再要求判决点名它；另外两份在工作区里还带着别的会话的改动，仍在范围里。
这一条「应该是什么」与「实际是什么」对不上：弹窗定的是改到的定义走三方，剔除之后门禁只管内容改了的，路径改了的那份既不在本轮被判集合里，也不会被门禁拦。要不要补判由主 agent 定。

### 冲突 2：拷贝而旧文件还在，改动范围看不见新文件

样本 C2：基准里 `scripts/experiments/a.sh`，此刻新前缀下多一份内容等于换算后基准的拷贝，旧文件没删。`gate_changed_paths` 只交出登记表（`change_range=['.claude/gate.d/path-moves.tsv']`），新文件被剔；
拦它的是 code-tooling 的 change-range-single-source 格（旧前缀还在磁盘上，样本里 `registry_cell_exit=1`）。整轮门禁里这一格会跑，单跑别的格时没人拦；这是剔除层把「新文件要核旧文件是否真被搬走」交给了另一格。

### 找不到的

「新路径要核存在」：被剔的文件里的新路径有没有指到东西，我没找到按改动范围去核的格；全仓扫的是 Markdown 链接（共享 gate.sh 的链接指向阶段）与治理文档的反引号路径（doc-registries 的 governance-refs），代码、钩子、脚本里的路径靠人（`.claude/rules/path-moves.md@怎么做` 第 4 步）。这条是我没找到，不是确认没有，没有逐格枚举。

## G4 定义与规则的句子

结论：部分冲突。三处句子写的条件都是脚本做的事的子集，没有一句与脚本相反；对不上的是漏写的条件（见下 1–4），以及两张表头仍写着改前的话（5）。
三处都没有写理由与来历（`.claude/singlefs-ai-sop/rules/rules-discipline.md@1. 正文只写四样`：不写为什么、实测、日期、谁定的、旧做法、判别力的经过与数），这一条符合。

推翻它的现象：主 agent 或用户说明「基准那一版」本来就是指「按搬迁之前的路径取到、且 sha256 是基准表里登记的那一份脚本」，则 1 只是简写，不算漏写。

被比的三处原文（行号现查）：

- `.claude/agent-common.md:70` 与 `.claude/main-agent.md:82`，同一句：「登记在 `.claude/gate.d/path-moves.tsv` 的目录搬迁照登记换算，基准那一版换算之后与此刻逐字节相同的不算涨；」（两处各恰好一次，`grep -oF` 数过）。
- `.claude/rules/path-moves.md:13`（「怎么做」第 2 步）：「门禁里与基准比只缩不涨的表、装置里按 sha256 钉的值，照它换算，只差这一段前缀的不算改动。」
- `.claude/rules/path-moves.md:56`（「门禁管哪一半」）：「搬迁登记表 `.claude/gate.d/path-moves.tsv` 写得对不对（写法、新前缀在、旧前缀不在、出处在）由门禁 code-tooling 的 change-range-single-source 格判。」

逐条比：

1. 「不算涨」的条件没写全。脚本（`moved_script_unchanged`，`.claude/hooks/lib_long_run.py:434-453`）要同时满足：按搬迁之前的路径取基准那一版脚本（「data, taken, problem = lag_baseline.baseline_file_bytes(base, path, cwd=root)」，`.claude/hooks/lib_long_run.py:437`）；
   它的 sha256 是基准那一版表里这条路径登记过的（`.claude/hooks/lib_long_run.py:442`）；换算后与此刻逐字节相同（`.claude/hooks/lib_long_run.py:449`）。路径列则是 `baseline_text` 先整表换成新前缀再比，不涉及字节相同。
   句子里「基准那一版」可以读成表，也可以读成脚本；「按搬迁之前的路径取」「sha256 是基准表里登记的」两个条件没写。判据缺的这两条正是 A4（红）与 A5（绿）的分界：按句子字面，A5 的字节相同拷贝也「不算涨」，句子没有「该路径此刻的脚本原来就是被豁免的那一份」之类的限定。
   这些条件写在 `.claude/hooks/lib_long_run.py` 文件头 exempt-check 一段（「它的 sha256 要是基准那一版表里登记的」），不在规则句子里。
2. 「换算」是前缀边界规则，不是子串替换：`experiments2`、`experiments.md`、提交引用后面的路径、行首 `-`、拆开写的目录名都不换（G2 的 26 种写法里 11 种）。三处句子都只写「照登记换算」「只差这一段前缀」，没有一处指到边界规则在哪里（读法在 `.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」，登记表头 `.claude/gate.d/path-moves.tsv:4` 写了一半）。
   规则文字按「怎么做」写，指到别的脚本也是允许的（`rules-discipline.md@1. 正文只写四样` 的「什么规则在哪里」），所以这一条只是「没指」，不是写错。
3. `path-moves.md:13` 只说「表」与「钉值」照登记换算；脚本还把登记作用到改动范围（文件与新增行都剔，G3）。读规则的人按这句会以为，改名之后改到的 agent 定义、钩子、`crates/` 源码仍照常被三方、同步、回看这几格逼着过；实际只差路径的那些不再被逼（G3 冲突 1）。规则没说。
4. 目录改名要另搜的写法，规则没说。第 1 步的清单是整条路径、`kb/` 起的写法、光文件名、不带 `.md` 的名字四种（`.claude/rules/path-moves.md@怎么做`）；
   第 4 步「改完按同一份清单再搜一遍，必须 0 命中」对没进清单的写法天然通过。调度记录列出这一次整串替换打不到、靠自证红才发现的写法：「整串替换打不到、另行改的」（`records/2026-10-23-实验装置目录改名.md:43`）一行里有拆开写的目录名（`"..", "experiments"`）、相对写法（`../experiments`）、排除写法（`--exclude /experiments/target/`）、`root, "experiments"` 这几类。
   所以正推的回答是：要写；写在第 1 步的清单里（不是第 2 步），第 4 步随之按同一份清单再搜。只写判据，不写这一次的经过。
   规则里还缺：登记行什么时候可以删（G2 的 B7）、登记行谁来审、出处里要写什么（脚本只核文件在，`.claude/gate.d/code-tooling.sh:189` 的格说明写「出处写记这次搬迁的那份记录」，脚本不核）。
5. 两张表头仍是改前的话：
   「只缩不涨：按（路径，sha256）成对跟基准那一版比，基准里没有这一对的行就算涨——新路径算涨，同一路径的 sha256 变了（改了脚本再换表里的 sha256 重钉）也算涨；」（`.claude/long-run-live-hosts-exempt:7`）；
   「指向不存在的文件、锚串一处被判行都不命中，判红；只缩不涨：改动范围的基准那一版里没有的「路径:锚串」判红。改成经登记位取了就删掉那一行。」（`.claude/heavy-thread-exempt:3`）。
   登记了搬迁时这两句不再成立（`exempt-check` 现跑报 6 行照搬迁换算后放行）。agent-common 与 main-agent 那一句补了例外，这两张表头没跟上。
6. `path-moves.md:56` 的括号与脚本逐项一致（写法、新前缀、旧前缀、出处），但「旧前缀不在」有 `moved_back` 例外（A7），「出处在」只核文件在；「写得对不对」一词比脚本核的宽，脚本核不出这一行对应一次真搬迁（G1）。
7. 单个文件的搬迁：登记表头与读法库接受文件前缀（`.claude/gate.d/path-moves.tsv:1` 写「目录或文件搬了家、改了名」），`path-moves.md:13` 只写「整个目录搬家或改名的」。脚本比规则宽，规则没说单个文件要不要登记。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 误放 | 冲突 | 登记表判法只核写法、新前缀在盘、旧前缀不在盘（有例外）、出处文件在；一行假登记把排除表一行转给别的路径、把改指别的目录洗成只差搬迁（A1–A3、A6、A7）；有 sha 的豁免表只在字节相同的拷贝上被洗（A5）；这一批真登记本身没洗出别的 |
| G1 倒换 | 一致（带说明） | 倒着换不是单射：全搬、半搬、没搬换出同一份（B2、B6），差的只在登记的前缀，不放过前缀之外的改动 |
| G2 读法一致 | 部分一致 | 同一个换算函数、26 种写法两个方向对称、冻结层两份读法逐字相同；路径一级与文本一级、正着比与倒着比、多次搬迁取几档、写坏的表、基准里留旧路径的行有出入，方向都是多算改动或多认前缀写法；与主 agent 观测一项不符（行首 `+` 库认） |
| G3 改动范围剔除 | 部分冲突 | 上游 show-me-test 与 i18n-sync 不经剔除；不分注释与代码；agent 定义里只差路径的 mutation-triage 门禁不再逼三方、本轮也没判（冲突 1）；拷贝而旧文件还在时只有另一格拦（冲突 2）；「新路径要核存在」没找到格 |
| G4 句子 | 部分冲突 | 三句都是脚本做的事的子集，没有相反的；漏写：基准脚本按搬迁之前路径取、sha 要是登记过的、剔除改动范围、目录改名要另搜的写法、登记行的寿命与审；两张表头仍写「新路径算涨」 |

## 要主 agent 复核的

- 样本全部在临时仓里造，没有碰主树。B7（改名提交落地之后基准里留旧路径的行）是模拟，落地后的真仓上它会不会让 history-loss 一类格红，本腿复核不了，需要在提交之后现跑。
- 改名用的替换脚本在会话草稿里，本腿没读到，G2 第 7 条对它的描述只能照主 agent 的观测，其中行首 `+` 一项与库实测不符。
- 三条工具员报告没读。

## 没做什么

- 不替主 agent 采纳或出判决；上面「规则没说」「建议」只是正推的结果。
- 没读攻方腿的报告与模型（禁读清单）；没读三份工具员报告。
- 没改被判的任何文件；没跑 checker 档、门禁整轮、`cargo`；没动 `crates/`（`grep -rn 'path-moves\|path_moves\|rewrite_moved' crates/` 零命中，退出码 1）。
- 没逐格枚举所有读改动范围的格看它们对被剔文件的反应（只看了上表 7 类）；「新路径要核存在」是没找到，不是确认没有。
- 没验证重钉前后两份编排器的语义等价（只核了常量与现 sha256 相同、reuse_library 43 项自证红 0）。
- 没复跑 doc-experiments decision-links 格的「161 → 152」，没复跑 doc-decisions history-loss（需要整轮门禁环境，且别的会话的改动在里面）。
- 草稿目录 `~/.local/share/singlefs/work/apparatus-rename-r1-forward/` 里只有自证输出，没有仓副本与编译目录，不需要清理。
