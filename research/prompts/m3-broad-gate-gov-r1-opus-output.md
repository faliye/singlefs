# m3-broad-gate-gov-r1 攻方腿（Opus）报告（2026-10-02）

攻击面：G2（第 2b 步次序、几个会话共用库目录、复用判定）、G4（重型闸）、G5（样本与弄坏开关），兼攻 G1（造该红不红、不该红而红的输入）。
被判对象取冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/` 的拷贝（草稿目录 `/tmp/claude-1000/m3-broad-gate-gov-r1-opus/tree/`，`rsync -a --exclude target --exclude .git`）。
下文引的阶段、驱动、钩子、准入模块、样本、commit-stages.md 在冻结副本、快照清单与此刻主仓三处逐字节相同（sha256 前 12 位：阶段 b2b2f27bd9cb、驱动 b4940858f74d、lib_heavy_tests 55842520d7cf、heavy-test-guard c6000f7b5c41、admission 25c9cc76bd6a、commit-stages 1d17915e0a26，样本目录 `diff -rq` 无差），所以引文行号按主仓路径写。
重型的一样没跑：驱动的真跑、门禁对真仓整道、`gate.sh` 都没起；跑的是登记为 light 的样本档（重型闸「门禁样本」② 的起法，与 `stage-selftest.sh --item`）、驱动与准入模块的纯函数、草稿目录里的临时 git 仓。

## 复跑命令与模型文件

模型目录 `research/prompts/m3-broad-gate-gov-r1-opus-model/`。`<T>` 是冻结副本的一份拷贝（不带 .git），`<W>` 是一个空草稿目录。

```
bash  <模型>/prepare-samples.sh <T> <W>                       # 拷 12 份样本跑 setup.sh；另拷一份把库目录指纹改成今天的公式；写出逐份起阶段的命令文件
for f in base break swallow fixed-base fixed-break fixed-swallow; do bash <W>/gen-$f-cmds.txt; done   # 照「门禁样本」② 逐份起阶段（240 次）
python3 <模型>/judge.py <T>/.claude/gate.d/fixtures/checker-tier-broad-mutation.sh <W>/out <前缀>     # 照 stage-selftest.sh 的判法判；前缀见 outputs/judge-all.txt
python3 <模型>/matrix.py    <T> <W>/run-fixed <T>/.claude/gate.d/fixtures/checker-tier-broad-mutation.sh   # 14 条变异各删一个判红分支，看哪份样本翻面
python3 <模型>/witnesses.py <T> <W>/run-fixed <W>                                                       # 给没人盯的分支造输入
python3 <模型>/toctou-model.py <T> <W>/run-fixed <W>
python3 <模型>/driver-model.py <T> <W>
bash    <模型>/marker-model.sh <T> <W>
python3 <模型>/g4-probe.py <T>
bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item checker-tier-broad-mutation.sh   # 主仓根，登记 light
```

在草稿目录里从空目录整套重跑过一遍：matrix-fixed、toctou-model、driver-model、marker-model、g4-probe 与第一遍逐字节相同，witnesses 只差草稿路径。输出原样存在 `outputs/`。

```
c349abd2c969a88b142c9167641f3c9c2db9cfc9c6f4b509c7da45b60019a328  cells.py
c274e77c294c18b820bcaac109f6b96f17b851c32d37b649e9aba5f85dea8656  driver-model.py
01f021682b524dc0989f53d62d156cce239d1df9d64a57278cdaecaeaa606a54  fix-green.py
fe9a2792a858e8ee4fe95747f112c24d471a17058f3f63ead151b8dbf8b77f72  g4-probe.py
59838af9ae65b546839c387cf1172a983f86654928fc30c8484eabfbe9343f86  judge.py
fddd24ae7747477411d8225c299d63e00dbd49c24ddf20703510cf7d8bef05a9  matrix.py
965e81085a13b5165649fd6d14015e867d0ed572bb95c596c50a5f3e38c36904  toctou-model.py
7a18a9e543eaad8ec3b5127410d6e03e16fe1ecbf60613602beaee9ecd78b1d2  witnesses.py
e4225c69420c7aa8863fb0a04fe95ced45425a8a44d491023df560d6e2a6731b  marker-model.sh
815a3481961db710cc4cf80559b8134a9d6df73f8b094e571e4273f8c51edcdd  prepare-samples.sh
0edf279cccdbd773e6d52110109053057a65ab96d89eed5074d5f8dc9134225b  outputs/driver-model.txt
5c2551d3649b70a75b25c889361f5329a1f3d6586222990c6148d091082f16f1  outputs/g4-probe.txt
7ae57e686b9a1ad81039dc5a287e0bca4c85d60a52e23d12bf4c2e710c3a07f7  outputs/gate-on-green-sample-as-is.txt
1a0bd600f6e40a07c812851b099e7b4d4642f66e9cc2f4c211e1b9bb3e6af17f  outputs/gate-on-green-sample-fixed.txt
236ad181053b4b83b2a899d4d49c35a8bc1f2f85102a4d413ffae5b9738c6e7e  outputs/judge-all.txt
257d4ee808d1c07895826e3c53cbf358232319de7888d3c50c592b86f275b212  outputs/marker-model.txt
82cd84d6dfb3c1d2288c539dc1b665d8f55f18062e2a6431b26f123d3bdbbac1  outputs/matrix-asis.txt
eb70c267c4a40128425273c7dc54dc94db7bfd51e5f0efc8dd945f8ecafbc68b  outputs/matrix-fixed.txt
daf3fa6c740b9ef439e274ad8b32462ac8a06f128dd08aff7b5d0f929d5fca57  outputs/stage-selftest-live.txt
4d0ad81b4ac9588c0ddc48096fab2cbedd0b06b516ebf32216c13d49b719c495  outputs/toctou-model.txt
356476f38d1a2ff8559568ef90c87e394ee80ce05f03dbcf66af4813aa2024dc  outputs/witnesses.txt
```

`cells.py` 把阶段两格的内嵌 python 原样抽出（两段 sha256 25e9b9b3…、3718b17f…），照阶段里 bash 的调法起；保真核过：12 份样本上它与真阶段的退出码逐份相同（12/12）。

## 各格判定一览

| 编号 | 格 | 打中了什么 | 判 | 证据 |
|---|---|---|---|---|
| A1 | G5 | 冻结副本里 12 份样本 2 份判错：两份绿样本判红。样本 setup.sh 照「判法版本进指纹之前」的公式造库目录指纹，阶段按今天驱动的 version_key 复算，对不上 | 打中（量过，主仓上 `stage-selftest.sh --item` 也是 10 对 2 错） | outputs/stage-selftest-live.txt、judge-all.txt |
| A2 | G5 | 同一原因让两个弄坏开关在今天的样本上失去判别力：work-tree-read 点名的那份不开开关就已经错；export-range-unchecked 下点名的那份仍退 1 | 打中（量过） | judge-all.txt |
| A3 | G5 | 两格共 14 个判红分支，13 个删掉之后 12 份样本一份都不翻面；其中 8 个删掉就漏（各造了一份原样红、删后绿的输入），5 个被别的分支兜住 | 打中（量过；照第五节记一笔、补样本） | matrix-fixed.txt、witnesses.txt |
| A4 | G2 | 门禁判的是索引，判绿后写的全绿标记按工作区算指纹；之后把工作区那份暂存，第 1 步 stage-marker-check 与整轮 --staged 的 gate-reuse 都认它，没判过的那一版直接复用 | 打中（模型上量过：五种用户动作扫过，三种两处都复用、一种第 1 步复用、一种不复用） | marker-model.txt |
| A5 | G2 / G1 | 两格各读一次 tree.source，中间别的会话 --sync-tree 换了指针：第一格判 A 是暂存树，第二格判 B 的结论绿，整道绿 | 打中（模型上量过；窗口是两格之间，推的概率低） | toctou-model.txt |
| A6 | G1 | 全局键（排除表、限时倍数、nextest / cargo-mutants / llvm-tools 版本、编译配置环境变量）变了而版本指纹不变：这一版已有的现判结论不重判，--summary 判绿；判决 E4 写的是「全局键变了就整版重判」 | 打中（模型上量过：驱动纯函数） | driver-model.txt 情形一 |
| A7 | G1 | timeout 与 memory-cap 永远是 NO_CONCLUSION：--summary 判红退 8，给的出路 --changed-since 选不中它们，--item 不带 --rerun 跳过，下一版也不沿用；用户定「卡住算抓到」，编译步与文档测试步的整步超时仍记 timeout | 打中（模型上量过；真库这一版还没出现，频率推的） | driver-model.txt 情形二 |
| A8 | G1 | gate-verdict 不比汇总行与判定行的 total：汇总行 total=5091、not_run=5085 而判定行 total=6 green，原样的门禁判绿 | 打中（量过，原样的格） | witnesses.txt 末行 |
| A9 | G4 | 子 agent 跑第 2b 步 ②③④ 与驱动任何算重型的用法都拒；主 agent 不带前缀拒、带前缀整类放行（含 --milestone-full、--import-results、--rerun）；两处误拒：驱动 --check-ignored、门禁 --check gate-verdict | 没打中「该拒不拒」（量过分类）；误拒两处记下 | g4-probe.txt |
| A10 | G2 | 复用判定的登记路径少 run-with-memory-cap.sh、capped.sh，环境少 llvm-tools 版本；库目录在 24 小时内被里程碑全量改红，复用照旧认绿 | 推的，没建模；与 A6 同一个病根（结论不按这些输入分版） | — |

## G5　样本与弄坏开关

### A1　两份绿样本在今天的冻结副本上判红

样本造库目录指纹的那一行与它的说明：
`.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/library-version-is-staged-tree-green/setup.sh:35`：「fingerprint = hashlib.sha256(f"{source_fingerprint}\n{toolchain}".encode()).hexdigest()」
`.claude/gate.d/fixtures/checker-tier-broad-mutation.sh/library-version-is-staged-tree-green/setup.sh:4`：「版本指纹是 sha256(源码指纹、换行、工具链)」
今天驱动的公式多一行判法版本：
`research/scripts/broad-mutation.py:732`：「return hashlib.sha256(f"{source_fingerprint}\n{toolchain}judge {JUDGE_RULES_VERSION}\n".encode()).hexdigest()」
阶段在文件、工具链都相同之后按驱动复算：
`.claude/gate.d/checker-tier-broad-mutation.sh:246`：「elif "files-unchecked" not in broken and not problems and driver.version_key(home, staged_source_fingerprint, current_toolchain) != fingerprint:」

主仓根上跑登记为 light 的样本档（`outputs/stage-selftest-live.txt` 原样，退 1）：

```
  ✗ checker-tier-broad-mutation.sh library-version-is-staged-tree-green 期望退出 0，实测 1
  ✗ checker-tier-broad-mutation.sh library-version-is-staged-tree-unstaged-change-green 期望退出 0，实测 1
  ✗ 2 个样本判错（判对 10 个，0 个阶段没有样本）
```

红的原因（`outputs/gate-on-green-sample-as-is.txt` 原样）：

```
     按驱动的 version_key 算出的版本指纹不是 tree.source 记的 1fc77b805df0426f
```

只把六份 library-version 样本的库目录指纹换成驱动 version_key 的结果（`fix-green.py`，别的一字不动），同一个阶段 12 份全对（`outputs/judge-all.txt`：「fixed-base 判对 12 个、判错 0 个：无」）。
工具实现员报告第三节的「判对 12 个」与驱动修正（判法版本进指纹）是同一批里先后做的；后者改了 version_key，样本没跟着改。真库此刻那一版用的是新公式（草稿里只读核过：tree.source 的 fingerprint 等于带 judge 行的那个式子，不等于不带的），所以阶段的判法是对的，错的是样本。

四句：
- 分不分辨臂：分辨「样本与驱动同一个公式」与否；阶段本身不出局。
- 判别它的东西阶段当时看得到：看得到，阶段打出了是哪一支红。
- 判据字面：第五节「一个判红分支没有样本或开关盯着 ⇒ 记一笔，补样本」管不到「样本自己错了」；落在 G5 正文「每个开关打开之后是不是红在点名的那一份」与样本档本身判错（提交前 stage-selftest 会红）。
- 改法：setup.sh 别自己抄公式，载入驱动调 version_key（推的，只在我的模型上用 fix-green.py 量过同一件事）；今天这份旧公式的库目录正好是「判法版本进指纹之前的驱动换的那一版」，留作一份新红样本（见 A3 的 M3）。

### A2　两个弄坏开关在今天的样本上失去判别力

`outputs/judge-all.txt` 原样（前缀 brk- 是原样样本，fixed-brk- 是改了指纹的样本）：

```
brk-work-tree-read 判对 10 个、判错 2 个：library-version-is-staged-tree-green(期望0→实测1，缺 want 2 条) library-version-is-staged-tree-unstaged-change-green(期望0→实测1，缺 want 2 条)
fixed-brk-work-tree-read 判对 11 个、判错 1 个：library-version-is-staged-tree-unstaged-change-green(期望0→实测1，缺 want 2 条)
fixed-brk-export-range-unchecked 判对 11 个、判错 1 个：library-version-is-staged-tree-export-range-red(期望1→实测0，缺 want 3 条)
```

- work-tree-read：点名的 unstaged-change-green 不开开关已经判错，开关打开前后判错清单相同，开关证不出什么。
- export-range-unchecked：原样样本下 export-range-red 仍退 1（被 version_key 那一支兜成红），只是 want 少了；按退出码比（工具实现员 breaks.txt 的比法）它不算翻面。
- 指纹改对之后，8 个开关各自红在点名那一份，与工具实现员第三节的表相同；多一处：files-unchecked 还让 export-range-red 少一条 want「暂存树有、那一版没有 .cargo/config.toml」（退出码仍是 1），按 stage-selftest 的判法这一份也判错。

### A3　判红分支没有样本盯着

`matrix.py` 在改对指纹的 12 份样本上，每条变异删一个判红分支（`cells.py` 的 MUTANTS 表），看样本退出码变不变。`outputs/matrix-fixed.txt` 原样：

```
BASELINE 判对 12 判错 0：无
MUTANT M1-pending	tree.source.pending 在判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT M2-only-recorded	那一版有、暂存树没有（暂存区删了文件）判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT M3-version-key	按驱动 version_key 复算的版本指纹不等判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT M4-merge-conflict	索引里有合并冲突判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT M5-source-fingerprint	逐文件相同而源码指纹不同判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT M6-toolchain-missing	那一版没记工具链判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT M7-empty-staged	导出范围里一个文件都没有判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT N1-exit-not-0-8	--summary 退的不是 0 也不是 8 判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT N2-verdict-exit-mismatch	判定行与退出码对不上判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT N3-green-with-counts	判定行写 green 而计数不为零判红	翻面的样本 1 份：gate-verdict-silent-driver-red(1→0)
MUTANT N4-sum-not-total	concluded 加 without_conclusion 不等于 total 判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT N5-detail-vs-counts	明细行数与计数不同判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT N6-total-zero	清单 0 条判红	翻面的样本 0 份：无（没有样本盯着这一支）
MUTANT N7-unknown-verdict	verdict 不是 green / red 判红	翻面的样本 0 份：无（没有样本盯着这一支）
```

`witnesses.py` 给每一支造一份输入，原样的格与删掉那一支的格各跑一次（`outputs/witnesses.txt`，每行末尾一栏，原样摘出判定栏）：

| 分支 | 输入 | 原样 | 删掉那一支 | 归类 |
|---|---|---|---|---|
| M1 | tree.source.pending 在 | 退 1 | 退 0 | 这一支没了就漏 |
| M2 | 暂存区删了 lib.rs | 退 1 | 退 1 | 冗余（M5、M3 兜住） |
| M3 | 库目录是判法版本进指纹之前的驱动换的那一版 | 退 1 | 退 0 | 这一支没了就漏 |
| M4 | 索引里 lib.rs 在合并冲突（theirs 与那一版相同） | 退 1 | 退 0 | 这一支没了就漏 |
| M5 | tree.source 的源码指纹与 manifest.tsv 不是同一次写的 | 退 1 | 退 0 | 这一支没了就漏 |
| M6 | 那一版没有 toolchain.txt | 退 1 | 退 1（FileNotFoundError） | 冗余（崩出来的红） |
| M7 | 导出范围里一个文件都没有 | 退 1 | 退 1 | 冗余 |
| N1 | 判定行 green、--summary 退 1 | 退 1 | 退 1 | 冗余（N2 兜住） |
| N2 | 判定行 red、计数与明细一致、--summary 退 0 | 退 1 | 退 0，打「判绿」 | 这一支没了就漏 |
| N4 | concluded 加 without 是 7、total 6，red 而退 0 | 退 1 | 退 1 | 冗余（N2 兜住） |
| N5 | 判定行 green、计数全零，却列了 2 行 NO_CONCLUSION | 退 1 | 退 0 | 这一支没了就漏 |
| N6 | 清单 0 条、判定行 green | 退 1 | 退 0 | 这一支没了就漏 |
| N7 | verdict=pending、--summary 退 8 | 退 1 | 退 0 | 这一支没了就漏 |

M3 正是 G1 题面那个例子（「沿用的结论来自判法版本不同的一版」）在门禁这一侧唯一的一道：驱动的 --summary 不看判法版本，只有这一支复算版本指纹。今天没有任何样本或开关盯着它（files-unchecked、toolchain-unchecked 会顺带关掉它，但它们点名的样本是因为别的分支翻面）。
N2 删掉之后，门禁对一行写着 verdict=red 的判定行打「广谱变异提交门禁判绿」。

四句：分辨的是「这一支在不在」；阶段看得到；判据字面是第五节「一个判红分支没有样本或开关盯着 ⇒ 记一笔，补样本」；改法是补八份红样本（M1、M3、M4、M5、N2、N5、N6、N7，witnesses.py 里的输入可直接搬），冗余的五支可以不补或补一份（推的：补不补交主 agent）。

## G2　次序、共用库目录与复用

### A4　判的是索引，标记按工作区记：没判过的那一版被复用

许可这段历史的每一句：
- 第 2b 步之前的第 1 步对这一道不要求工作区与索引相同：`.claude/skills/crash-test/commit-stages.md:19`：「第 2b 步的驱动也不起；不跑 git diff --quiet 与未跟踪文件那两条」
- 格读的是索引：`.claude/gate.d/checker-tier-broad-mutation.sh:169`：「listed = subprocess.run(["git", "-C", root, "ls-files", "-s", "-z", "--", *export_paths], capture_output=True, env=environment)」
- 判绿之后写标记：`.claude/gate.d/checker-tier-broad-mutation.sh:377`：「if ! python3 "$ADMISSION_MODULE" stage-marker-write "$ROOT" "$(basename "$0")" >/dev/null; then」
- 标记的指纹读磁盘上的文件（已跟踪的加没忽略的未跟踪的）：`research/scripts/admission.py:477`：「["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],」，`research/scripts/admission.py:693`：「digest_hex = sha256_of_file(os.path.join(os.fsencode(root), name))」
- 复用判定先看标记，有就跳过，不看 SINGLEFS_STAGED_TREE：`research/scripts/admission.py:1137`：「return EXIT_GATE_MAY_SKIP, f"这一道这批输入（指纹 {fingerprint[:16]}…，{file_count} 个文件）有全绿标记：{lines[0]}"」

历史（`marker-model.sh`，草稿目录里的临时 git 仓，拷的是冻结副本；判绿那一步用阶段第 377 行同一条调用）：
1. 索引是 S1，工作区比 S1 多一处没暂存的 U（几个会话共写一个仓时就是别人的半成品）。
2. 主 agent 照第 2b 步判 S1，两格绿，阶段写标记。标记的指纹算的是 S1+U。
3. 之后有人把 U 暂存（用户动作，放开扫了五种）。
4. 下一次提交：第 1 步 stage-marker-check 与整轮 --staged 的 gate-reuse 都按「这批输入」判。

`outputs/marker-model.txt` 原样（时刻替成了 `<跑完的时刻>`）：

```
[dirty] 判绿那一刻：git diff --quiet 退 1；索引 lib.rs blob d7f40408cccc，工作区 lib.rs blob a731cee3e518
[dirty] 阶段判绿之后写标记（第 377 行那一条）：退 0
[dirty] 之后暂存 U：索引 lib.rs blob a731cee3e518（判绿时格判的不是这一份）
[dirty] 下一次提交第 1 步 stage-marker-check 退 0：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.f3a982e20feec333e5906766e3839ce245f969cac75f892d79bbc5f631f7df7d <跑完的时刻>
[dirty] 整轮 --staged 的临时树上 gate-reuse 退 10（10 = 可跳过、0 = 要跑）：这一道这批输入（指纹 f3a982e20feec333…，248 个文件）有全绿标记：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.f3a982e20feec333e5906766e3839ce245f969cac75f892d79bbc5f631f7df7d <跑完的时刻>
[commit-first] 下一次提交第 1 步 stage-marker-check 退 0：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.f3a982e20feec333e5906766e3839ce245f969cac75f892d79bbc5f631f7df7d <跑完的时刻>
[commit-first] 整轮 --staged 的临时树上 gate-reuse 退 10（10 = 可跳过、0 = 要跑）：这一道这批输入（指纹 f3a982e20feec333…，248 个文件）有全绿标记：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.f3a982e20feec333e5906766e3839ce245f969cac75f892d79bbc5f631f7df7d <跑完的时刻>
[untracked] 下一次提交第 1 步 stage-marker-check 退 0：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.3cd1adff98c453fbefbc46f993f0588f5f4f9756f938022eb709157df55a9f00 <跑完的时刻>
[untracked] 整轮 --staged 的临时树上 gate-reuse 退 10（10 = 可跳过、0 = 要跑）：这一道这批输入（指纹 3cd1adff98c453fb…，249 个文件）有全绿标记：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.3cd1adff98c453fbefbc46f993f0588f5f4f9756f938022eb709157df55a9f00 <跑完的时刻>
[partial] 下一次提交第 1 步 stage-marker-check 退 0：ok <repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.385526c4a912035a90c873d2ebe5298ba5a57f9a97be21bb3292b6e246513970 <跑完的时刻>
[partial] 整轮 --staged 的临时树上 gate-reuse 退 0（10 = 可跳过、0 = 要跑）：还没有过整轮全绿的暂存树（refs/sop/staged-green 不存在）
[clean] 下一次提交第 1 步 stage-marker-check 退 1：这批输入（指纹 02f4c3d7a6dae62b…）没有全绿标记：<repo>/.git/singlefs-stage-green.checker-tier-broad-mutation.sh.02f4c3d7a6dae62b99b529836bf318e05a72872895d1a0214c1bfd6b4eb86c26
```

（整份 26 行在 outputs/marker-model.txt；上面删了各情形的头三行，那三行与 dirty 同形。）

用户动作扫了五种（只固定「判绿那一刻工作区与索引的关系」与「之后暂存」）：dirty（暂存 U）、commit-first（先提交 S1 再暂存 U）、untracked（判绿时 crates 下有没跟踪的新文件，之后 git add）——第 1 步与整轮两处都复用；partial（工作区两处改动只暂存一处）——第 1 步复用、整轮要跑；clean（判绿时工作区与索引相同，之后才改）——两处都不复用。复用落点只取决于「判绿那一刻的工作区」等不等于「之后的暂存区」，与之后做什么无关，所以不是装置写死造出来的。

四句：
- 分不分辨臂：分辨「标记按被判的那份记」与「按工作区记」；同一个准入模块给另四道重阶段记标记没错，那四道开跑前都要求 git diff --quiet 与没有未跟踪文件，这一道明写不要求。
- 判别它的东西阶段当时看得到：看得到，格刚读过索引，`git diff --quiet -- <登记路径>` 一条就分得出。
- 判据字面：第五节「一个输入让门禁该红不红或不该红而红 ⇒ 打中，改门禁或驱动」，该红不红那一分句（U 里的变异一条没判过，提交前门禁记复用）。
- 改法各修哪一格（都是推的，没实现、没跑；被攻过零轮）：

| 改法 | dirty / commit-first / untracked 第 1 步 | 同三种整轮 gate-reuse | partial 第 1 步 |
|---|---|---|---|
| ① 阶段判绿后写标记之前核 `git diff --quiet` 与登记路径下没有未跟踪文件，不满足就不写 | 修（推的） | 修（推的：标记不写就没得复用） | 修（推的） |
| ② 这一道的标记指纹改按索引里的 blob 算（admission.py 加一种取法） | 修（推的） | 修（推的） | 修（推的） |
| ③ 只改 commit-stages.md 第 1 步，这一道也跑 git diff --quiet 与未跟踪两条 | 修（推的） | 不修：标记在前一趟已经按工作区写下，整轮的 gate-reuse 照认 | 修（推的） |

### A5　两格之间换了版本指针

- 两格先后跑，各自一个子 shell：`.claude/gate.d/lib/stage-cells.sh:218`：「elif ( "$function_name" ); then」
- 第二格只拿 --summary 的汇总行与此刻 tree.source 比：`.claude/gate.d/checker-tier-broad-mutation.sh:299`：「if "version-unchecked" not in broken and summary.get("fingerprint") != version[:16]:」
- --summary 不拿锁（锁在查询那一支之后才取）：`research/scripts/broad-mutation.py:2687`：「if arguments.list_items or arguments.summary or arguments.check_ignored:」，`research/scripts/broad-mutation.py:2717`：「library.lock_descriptor = acquire_lock(library.home, arguments.lock_wait)」
- --sync-tree 的提交点是一次改名：`research/scripts/broad-mutation.py:1114`：「os.replace(library.tree_source_pending, library.tree_source)」

`outputs/toctou-model.txt` 原样：

```
[swap-between-cells] 第一格 library-version-is-staged-tree 退 0：✓ 广谱变异库目录那一版（724b584c37f0758e，导出自 0123456789ab）就是被判的暂存树：导出范围 crates Cargo.toml Cargo.lock litmus .cargo 里 6 个文件逐个相同，工具链相同
[swap-between-cells] 两格之间别的会话 --sync-tree 换指针：tree.source 724b584c37f0758e → 7a2df53692c419d2（导出自 bbbbbbbbbbbb）
[swap-between-cells] 第二格 gate-verdict 退 0：✓ 广谱变异提交门禁判绿：这一版（7a2df53692c419d2）6 条每一条都有结论（沿用 4、现判 2），没红 0 条；里程碑全量翻面 0 条（只报不判）
[swap-between-cells] 整道：绿
[no-swap-control] 第二格 gate-verdict 退 1：✗ 汇总行的 fingerprint=7a2df53692c419d2 不是库目录 tree.source 那一版（724b584c37f0758e）：判的不是同一版
[no-swap-control] 整道：红
```

（对照组第一格一行与 swap 组相同，略。）B 那一版「每条都有结论」在几个会话轮流用库目录时是常态：B 会话判完自己那一版、被 A 换走，回来再 --sync-tree 到 B，那一版的结果表还在。窗口只有第一格读 tree.source 到第二格起 --summary 之间（推的：一两秒），单次撞上的概率低；撞上时整道判绿，两行 ✓ 里的指纹不同，只有逐字读输出的人看得出。

四句：分辨「两格判同一版」与否；阶段看得到（第一格知道 A，第二格看到 B）；判据是第五节该红不红那一分句；改法（推的、零轮）：第一格把核过的指纹写进阶段的临时目录、第二格要求汇总行等于它；或整道开跑时 flock 库目录的锁、跑完再放（驱动 --sync-tree 拿同一把锁，等 --lock-wait 秒拿不到就退 4，不会在两格之间换指针）。两种都修这一格；只改出路文字不修。

## G1　该红不红、不该红而红的输入

### A6　全局键变了，版本指纹不变：这一版的旧结论照算

- 用户定案那份判决的 E4：`research/prompts/tooling-broad-mutation-prune-r3-main-verification.md:43`：「全局键变了就整版重判。」
- 驱动自己的说法：`research/scripts/broad-mutation.py:2217`：「键变了，记着别的键的结论一条都不沿用（整版重判）。」
- 版本指纹只吃源码、工具链与判法版本：`research/scripts/broad-mutation.py:726`：「def version_key(home, source_fingerprint, toolchain):」（式子见上文第 732 行）
- --changed-since 的计划说「要现判」，选条只选这一版里没有定论的：`research/scripts/broad-mutation.py:1990`：「if state in (NOT_RUN, DRIVER_ERROR) and not arguments.stage_two_only:」
- 全局键全文每趟覆盖写一份，结果表的行不记它：`research/scripts/broad-mutation.py:2591`：「write_atomically(library.version_file("global-key.txt"), pruner.global_text.encode())」
- 门禁两格一处都不读全局键：`grep -c 'global' .claude/gate.d/checker-tier-broad-mutation.sh` 打 `0`。

`outputs/driver-model.txt` 情形一原样（驱动当模块载入，只调 version_key、global_key、carry_decision、select_items、summarise；库目录是草稿里手造的三条）：

```
[一] 排除表旧、CPU 倍数 3：版本指纹 563fcfa653ec66ba，全局键 042361b354cba42c
[一] 排除表加一行、CPU 倍数 3：版本指纹 563fcfa653ec66ba，全局键 140bc0966d63c172
[一] 排除表旧、CPU 倍数 4：版本指纹 563fcfa653ec66ba，全局键 ec6f46d818514ccf
[一] 结论表里那一行记的是旧全局键，carry_decision 在新全局键下交 (False, 'global-key-changed')
[一] --changed-since 的计划三条都要现判（全局键变了），select_items 选中的序号：[3]（这一版里已有现判结论的 1、2 不重判）
[一] 不跑 ③ 直接 --summary 退 0：BROAD_MUTATION_GATE total=3 concluded=3 carried=1 fresh=2 missed=0 without_conclusion=0 flipped=0 verdict=green
```

可达的历史：一次提交只改 `research/scripts/broad-mutation-ignored-exclusions.tsv`（或升了 cargo-nextest / cargo-mutants，或第 2b 步 ③ 换了 --cpu-multiplier），crates/ 一字没动。复用判定罩着排除表与工具版本，门禁照跑；第一格：暂存树与库目录那一版逐文件相同、工具链相同，绿；第二格：--summary 读的是同一版的结果表，上一次的现判行与沿用行都在，绿。照第 2b 步老实跑 ③ 也一样，只有上一次的沿用行那几条被重判。cargo-mutants 升级时连清单都不重新生成（同一版的 list.json 照用）。

四句：
- 分不分辨臂：分辨「这一版」按不按全局键分；与驱动、门禁两边都有关。
- 判别它的东西看不看得到：结果表的行不记全局键，看不到；结论表 conclusions.tsv 的每一行记了版本与全局键（只记 caught / missed / unviable），门禁要查得改读它。
- 判据字面：第五节该红不红那一分句；对着的是 E4 那一句「整版重判」。
- 改法（推的、零轮）：① 全局键进版本指纹（version_key 多吃一行全局键的 sha256），全局键一变就是新的一版，旧结论按 carry_decision 一条不沿用，与 E4 字面一致；代价是升级一次 nextest 就整版重判（E4 本来就这样要求，量没量过：没量）。② 结果行加一栏全局键，--summary 只认此刻全局键下的行。① 修这一格；② 也修，但门禁的第一格仍判不出全局键，要靠驱动。
- 同一个病根的推的变体：判法代码（驱动、mutation-prune.py）改了而没人改 JUDGE_RULES_VERSION，版本指纹不变，旧结论照算；文件头那一句「判法变了就改它」靠人记。

### A7　超时与内存撞顶：判红，而且出路走不出去

- 环境型结局算定论、不再自动重跑：`research/scripts/broad-mutation.py:307`：「ENVIRONMENTAL_OUTCOMES = ("timeout", "memory-cap")            # 也算定论、不再自动重跑，但不盖掉前面的 caught / missed / unviable」
- 门禁判法只认三种为有结论：`research/scripts/broad-mutation.py:2043`：「without = [item for item in library.items if (states.get(str(item["sequence"])) or ItemState(NOT_RUN, False)).state not in DECISIVE_OUTCOMES]」
- 结论表只收三种（下一版沿用不了）：`research/scripts/broad-mutation.py:1853`：「if runner.pruner and outcome in DECISIVE_OUTCOMES:」
- 用户定的是卡住算抓到，可只落在经 runner 的测试步：`research/scripts/broad-mutation.py:100`：「超了算那条测试没过 = 抓到」；编译步与文档测试步仍按整步时限：`research/scripts/broad-mutation.py:1614`：「整步的墙钟兜底：编译步与文档测试步照旧」

`outputs/driver-model.txt` 情形二原样：

```
[二] --summary 退 8：BROAD_MUTATION_GATE total=3 concluded=1 carried=0 fresh=1 missed=0 without_conclusion=2 flipped=0 verdict=red | NO_CONCLUSION	2	timeout	crates/singlefs-core/src/sample.rs:2:9: replace x2 | NO_CONCLUSION	3	memory-cap	crates/singlefs-core/src/sample.rs:3:9: replace x3 |   → 怎么办：没有结论的用 SINGLEFS_HEAVY_TESTS=commit python3 /tmp/claude-1000/m3-broad-gate-gov-r1-opus/tree/research/scripts/broad-mutation.py --changed-since <上一次判过的提交> 补齐（沿用得了的落沿用行，沿用不了的现判）；没红的加一条能杀它的测试再判，确是等价变异就写测试留档后从清单排除
[二] 照出路 --changed-since：计划三条都要现判，select_items 选中的序号：[]
[二] --item 2 --item 3 不带 --rerun：选中 []；驱动说：· 第 2 条已有定论（超时），跳过；要重跑加 --rerun | · 第 3 条已有定论（内存撞顶），跳过；要重跑加 --rerun
[二] 结论表只收 ('caught', 'missed', 'unviable')（process_item 里 outcome in DECISIVE_OUTCOMES 才 record_conclusion），carry_decision 只沿用 ('caught',)：下一版里这两条也是现判，再超时、再撞顶就再红
```

一条变异让文档测试卡住（或让某一步撞内存上限），这一版门禁就一直红，第 2b 步与门禁出路都走不出去；只有 `--item N --rerun`，再卡住还是 timeout。每一版都现判它，每一版都红，直到那个函数被改掉。真库此刻那一版（e1ba2574…）只有 2 行结果，没有 timeout；旧驱动那一版（19c45e02…，只读看过它的 results.tsv）有状态的 302 条里 7 条 timeout，备注全是「one-test 超过 1392 秒」（那一步今天归 runner 限时、记 caught），所以今天会不会出、多常出是推的。

四句：分辨「环境型结局算不算结论」；看得到（结局就在结果表里）；判据是第五节不该红而红那一分句（timeout 那一半，对着用户「卡住算抓到」）；memory-cap 算不算抓到我没找到用户定案，只能说出路是死路。改法（推的、零轮）：timeout 照用户定案记 caught（与 runner 那一支一致）；memory-cap 交用户定；门禁与驱动的出路文字补「--item <序号> --rerun」。只改出路文字修不了 timeout 那一半。

### A8　汇总行与判定行的 total 不比

- 这一格自称：`.claude/gate.d/checker-tier-broad-mutation.sh:12`：「判定行、汇总行与退出码互相对得上」
- 拿汇总行比的只有 missed：`.claude/gate.d/checker-tier-broad-mutation.sh:333`：「if len(no_conclusion) != without or len(missed_lines) != missed or number("missed", summary) != missed:」

`outputs/witnesses.txt` 末行原样：

```
WITNESS G1 汇总行 total=5091、not_run=5085，判定行 total=6 green（两行的 total 不一致）	原样 exit=0「✓ 广谱变异提交门禁判绿：这一版（970a802431217260）6 条每一条都有结论（沿用 4、现判 2），没红 0 条；里程碑全量翻面 0 条（只报不判）」
```

这是驱动判法失灵那一类（与样本 silent-driver-red 同一类），门禁给自己定的职责就是在驱动失灵时判红。四句：分辨「两行互核」与否；看得到（两行都在输出里）；判据第五节该红不红；改法（推的）：要求两行 total 相同，且汇总行里 not_run、stage_one_only、driver_error、timeout、memory_cap 之和等于判定行的 without_conclusion；补一份对应的红样本。

### A10　复用判定的登记（推的，没建模）

stage-inputs.tsv 这一道那一行登记了驱动、mutation-prune.py、排除表、crates-mutation-rows.py、stage-cells.sh，environment= 打 rustc -Vv、nextest、cargo-mutants 版本；没登记 run-with-memory-cap.sh、capped.sh（驱动每一步经它们起，退出码 250、253 的含义决定 memory-cap / timeout），--tool-versions 不打 llvm-tools 版本（全局键里有）。另外，24 小时内同一版被里程碑全量（--milestone-full）翻出没红的条，复用仍认上一次的绿（阶段文件头第 30 行自认靠 24 小时兜）。这些输入变了，门禁重跑也只是读同一张结果表（A6 的病根），所以补登记本身不改判定；补 A6 之后才有意义。

## G4　重型闸

- 子 agent 一律拒：`.claude/hooks/heavy-test-guard.sh:93`：「子 agent：一律拒，带不带前缀都拒（要跑的交回主 agent）。」
- 驱动只放过三个只读参数：`.claude/hooks/lib_heavy_tests.py:397`：「BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--summary", "--help", "-h"})」
- 整类放行：`.claude/hooks/lib_heavy_tests.py:276`：「放的是整类，不按参数再分。」

`g4-probe.py` 载入冻结副本拷贝里的 lib_heavy_tests.py，用 heavy-test-guard.sh 判命令文本的同一个入口（heavy_tests_in_command_text）逐条判，再照 refusal_reason 的写法写谁会被拒。`outputs/g4-probe.txt` 原样：

```
PROBE 驱动 --summary（门禁 gate-verdict 格起的那一条）	kind=None	类=—	谁都放行（不算重型）
PROBE 驱动 --list-items --changed-since HEAD（文件头说只读）	kind=None	类=—	谁都放行（不算重型）
PROBE 驱动 --check-ignored（文件头：不编不跑、不要前缀）	kind=broad-mutation	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE 驱动 --selftest（登记 light）	kind=None	类=—	谁都放行（不算重型）
PROBE 第 2b 步 ② --sync-tree --commit	kind=broad-mutation	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE 第 2b 步 ③ --changed-since HEAD	kind=broad-mutation	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE 里程碑末全量 --milestone-full（几天）	kind=broad-mutation	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE --import-results（往结果表里导入别处的行）	kind=broad-mutation	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE --item 1 --rerun（已有定论也重跑）	kind=broad-mutation	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE 门禁整道（第 2b 步 ④）	kind=broad-mutation-stage	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE 门禁 --check gate-verdict（只读库目录、起 --summary）	kind=broad-mutation-stage	类=广谱变异	子 agent 拒（带不带前缀）；主 agent 不带前缀拒、带 =commit / =user-request 放行
PROBE 门禁 --list	kind=None	类=—	谁都放行（不算重型）
PROBE 门禁 --tool-versions	kind=None	类=—	谁都放行（不算重型）
PROBE python3 读 stdin 喂的代码（文件头自认罩不住）	kind=None	类=—	谁都放行（不算重型）
PROBE python3 跑一份 import 驱动、调它函数的脚本（文件头自认罩不住）	kind=None	类=—	谁都放行（不算重型）
```

判：
- 「该拒不拒」没打中：第 2b 步 ②③④ 与驱动每一种起 cargo、改库目录的用法，子 agent 都拒，主 agent 不带前缀都拒。冻结副本里已经没有按 agent 类型的放行表（heavy-test-guard.sh 里搜不到 AGENT_KINDS），工具实现员报告里那张 crash-verifier 放行表随删定义一起没了；所以「放行整类而不按参数」今天的意思是：带前缀的主 agent 能起 --milestone-full、--import-results、--rerun。这与主 agent 带前缀能跑层 0 全量、崩溃放量是同一种信任，没有多出一个只有这一类才有的口子（推的）。
- 误拒两处（不该拒而拒，量过分类）：驱动 `--check-ignored`，文件头写不要前缀：`research/scripts/broad-mutation.py:13`：「不编不跑、不要前缀」，闸按重型拒，子 agent 与不带前缀的主 agent 都跑不了；门禁 `--check gate-verdict` 只读库目录、起 --summary，闸按这一类拒（lib 文件头写明「--check 一格算这一类」，是有意的）。
- 自认的口子照旧：python 读 stdin 的代码、import 驱动再调函数的脚本都不算重型（我自己的 driver-model.py 就是后一种，它只调纯函数、库目录是手造的，没起 cargo）。
- 推的、没量：`python3 research/scripts/admission.py stage-marker-write <根> checker-tier-broad-mutation.sh` 谁都能起（准入模块不算重型），能不跑门禁直接写一份全绿标记；这对每一道写标记的阶段都一样，不是这一批新开的口子。

## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 驱动参数组合骗过只读判定（--summary 加 --sync-tree、--list-items 加 --item、-h 夹在值里、--summary 写成别的参数的值） | 读 parse_arguments：几种模式互斥，--summary / --list-items / --check-ignored 一支在取锁与前缀之前就返回 | 没打中 |
| 导出范围里的符号链接、子模块、执行位、叫 target 的目录 | 读两边代码：驱动的 manifest_of_tree 与门禁第一格同样跳过符号链接与 IGNORED_DIRECTORY_NAMES，都不看执行位；主仓导出范围 241 个文件全是 100644 | 没打中 |
| .gitattributes 的 export-ignore / export-subst 让 git archive 与索引不同（不该红而红） | 主仓 `git ls-files` 里没有 .gitattributes，.git/info/attributes 不在 | 今天不可达（推的：加了就会永远红） |
| 合并冲突的索引 | witnesses.py M4 | 原样的格判红，对 |
| 暂存区删文件、导出范围空、toolchain.txt 不在、--summary 退 1 | witnesses.py M2、M7、M6、N1 | 原样判红，删掉那一支也被别的分支兜住 |
| 第 2b 步 ① 之后暂存区又变 | commit-stages.md 第 28 行要求再取一次 write-tree，不同就停 | 没打中 |
| 别的会话 ③ 占着锁时本会话 ②：退 4，④ 照跑判红 | 读 acquire_lock 与第 2b 步 | 判得对（红），代价是等别人那一趟跑完（以小时计，推的），不是错判 |
| 别的会话没提交的那棵树判出的 caught 被本会话沿用 | 读 carry_decision：键与 fn 项、函数外部分、宏依赖、测试的指纹都要相同才沿用 | 函数本身不同的沿用不了；只剩用户已定靠里程碑末全量兜的「别处改动」那一类（推的） |

顺带看到的（G4 文字，不判）：lib 文件头还指着一张已经没有的表：`.claude/hooks/lib_heavy_tests.py:264`：「子 agent 的放行表里没有这一类，只有主 agent 带前缀能跑（heavy-test-guard.sh 的 AGENT_KINDS）。」，heavy-test-guard.sh 里 `grep -c AGENT_KINDS` 打 0。

## 这条腿自己的限度

- 只抽了一次样（第五节：「没打中」的格不支撑「没有错」）。A9 的「该拒不拒没打中」与「没打中的形状」那张表都属于这一类。
- A3、A5、A8 用的是从阶段里抽出来的两段内嵌 python（cells.py），不是整道阶段；保真只核到 12 份样本上退出码逐份相同。阶段外壳（stage-cells.sh 的汇总、复用、写标记）在这三格里没跑。
- A4 的判绿那一步没真跑门禁两格，直接调了阶段第 377 行那一条 admission 调用；「判绿时格判的是索引 S1」来自代码（第 169 行）与模型里打的 blob 号，不是整道阶段的输出。
- A5 是顺序模拟，没起真的并发进程，窗口长短是推的。
- A6、A7 只调了驱动的纯函数，计划（plans）是手造的「全部要现判」，没经 plan_changed_since 与真的树切词；select_items 与 summarise 是驱动原样的代码。
- A7 的发生频率没有数：真库此刻那一版只有 2 行结果。
- 我提的每一个改法都只在我的模型上想过或量过同一件事，被攻过零轮；A1 的 fix-green.py 是唯一实现了的（它只改样本的拷贝），其余全是推的。
- 冻结副本的样本档在主仓上也跑了一遍（stage-selftest.sh --item，登记 light，不写仓）；别的都在草稿目录的拷贝上。

## 要主 agent 跑的命令（重型，我没跑）

1. 在真暂存树上核 A4 的前半句（格判索引、标记按工作区）：工作目录主仓根，前缀 `SINGLEFS_HEAVY_TESTS=commit`，先照第 2b 步 ①–③ 让库目录到这一次暂存树那一版，再在工作区留一处 crates/ 下没暂存的改动，跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh`；判绿后 `git add` 那处改动，跑 `python3 research/scripts/admission.py stage-marker-check . checker-tier-broad-mutation.sh`。看：它退 0（打 ok 行）就是 A4 在真仓上成立；退 1 就推翻 A4。不跑完这一趟的话，模型上的结论只到准入模块那一层。内存与线程照第 2b 步输入给的。
2. 不用跑重型的核法：`bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item checker-tier-broad-mutation.sh`（登记 light，谁都能跑），看末行是不是「✗ 2 个样本判错（判对 10 个，0 个阶段没有样本）」——这是 A1。

## 什么现象会推翻这些结论

- A1：样本 setup.sh 里那一行指纹式子与驱动 version_key 一致，或主仓上 stage-selftest 12 份全对。
- A3：有一份样本在删掉某一支之后翻面（照 matrix.py 跑出非 0 份）。
- A4：真仓上第 1 条命令里 stage-marker-check 退 1；或 admission.py 的这一道改成按索引算指纹。
- A5：第二格拿第一格核过的指纹比，或整道持锁。
- A6：版本指纹或结果行带上全局键；或用户改口「全局键变了不必整版重判」。
- A7：timeout / memory-cap 在 summarise 里算有结论；或 select_items 在 --changed-since 下重选它们。
- A8：门禁加上两行 total 的互核。

## 没做什么

- 没跑：驱动任何真跑用法（--sync-tree、--changed-since、跑条）、门禁对真仓整道、`gate.sh`、全量 cargo test、check.sh、54 号与另五道重阶段；也没编译 Rust。
- 没判 G1 / G3 的逐句对照（正推腿的格）；没判删定义那一批本身。
- 没改仓里任何文件（只写了这份报告与模型目录）；冻结副本只读，拷进草稿目录用。
- 只读看过真库目录 `~/.local/share/singlefs/broad-mutation/`（tree.source、两个版本目录的结果表与基线表、global-key.txt），没写。
- 草稿目录 `/tmp/claude-1000/m3-broad-gate-gov-r1-opus/`：冻结副本的拷贝、样本副本与模型造的临时 git 仓（含它们的 worktree）交回前都删了；只留第一遍的命令文件（`gen-*.txt`）、第一遍的输出 `out/` 与主仓 stage-selftest 的原始输出，全是文本、1.2M。复跑照上面的命令从冻结副本重新拷。没有编译目录。
