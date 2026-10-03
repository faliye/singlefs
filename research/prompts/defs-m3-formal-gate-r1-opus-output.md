# defs-m3-formal-gate-r1 攻方腿（Opus）报告：D1、D2、D3

日期：2026-10-01。被判的 11 份文件读主仓现行版本，与快照 `research/prompts/defs-m3-formal-gate-r1-snapshot/sha256sums.txt` 逐份 `sha256sum -c` 全 OK（17 行）。
被判的命令一条都没执行：hook 只喂 JSON、只看退出码；D2 只调 `admission.py` 的判定子命令，在草稿拷贝里跑。

## 复跑

```
bash research/prompts/defs-m3-formal-gate-r1-opus-model/rerun.sh <仓根> <一个不存在的草稿目录>
```

它依次跑 `d1-hook-cases.py`（22 条 JSON 喂 `.claude/hooks/heavy-test-guard.sh`）、`d2-reuse.sh`（拷一份仓、自己 `git init`，48 格复用判定）、`d1b-untracked-results.sh`（未跟踪产物让标记对不上）、`fix-draft.py` 打两处草稿改法后再跑一遍 `d2-reuse.sh`，最后汇总成 `d2-summary.txt`。挂钟约 2.5 分钟，不编、不起 Verus、不跑门禁。git 写只落在草稿拷贝自己的 `.git` 里。

模型目录 `research/prompts/defs-m3-formal-gate-r1-opus-model/` 的 `SHA256SUMS`：

```
86ad45b6acb5ca1c51abfea38ca7ca807a5a115a3c89fc4513d1886538a702bb  d1-hook-cases.py
b1a96f117fce118778eaca23f41182e285ab0891a14842593b93f3fcda3a2c64  d1b-untracked-results.sh
3f02abe03f63098e37ac6c775b89b6db443f81ab1b0888f6a0b10dee5719bd9e  d2-reuse.sh
622d8ce96da337f7a9f3eb3fa2dfbc230b42f2ea61521df397b0af5cc70f977a  fix-draft.py
7b773326256cce150031f250a203dcb960877406b4fabba95cc619518c3e30ef  summarize.awk
e845118ca120f562b647bc9a30a442d7c0e085e59a15af1d1b51a90d0d683f59  rerun.sh
fed4a4508a9d9140a5f4233a86f9e512046276d3297f8cce17b3bba81e057ae8  d1-output.txt
675da3e9bee0c5bc25d5310508b7dc95a0667e764e951679d33587055acfda36  d1b-output.txt
2f8cccc3570d231d81dd6904fbf44cb6165f4fb8f04703738eafeff93fe49f5c  d2-output.txt
df22449473deaf9f75c84de3d36d2cb4290ec25bb49de26041c33385e76bf1f7  d2-F1-output.txt
fde9d4071b2e1fe366c0a0a5356943156f914a9919a1b98afcda2bfb154267ef  d2-F1F2-output.txt
6be5695d37e1cedd679efd5babd1ed8f41faa94ee979835b8555838b604080a5  d2-summary.txt
085ecbf1f255ea6979a0a972a2c16255d94414381e15c6bca7961c934bd93196  observations.txt
```

复跑时指纹值会变（拷的是那一刻的工作区，别的会话在改 `research/results/`），`d2-summary.txt` 的 48 格计数不变：两次复跑（草稿 `rerun1`、`rerun2`）`diff` 为空。

## 各格判定一览

| 编号 | 格 | 判定 | 量过 / 推的 | 一句话 |
|---|---|---|---|---|
| A1 | D1 | 打中 | 量过（hook 退出码） | 这一道六格的判法是两份 python 脚本，闸不认；任何子 agent 不带前缀直接跑 `verus-proof-check.py must-red-worlds`、`vacuity`、`e165-pin-red-examples.py --replay`，闸放行 |
| A2 | D1 | 打中 | 量过（草稿拷贝） | 「这五道在 `gate.sh` 里只核标记」不成立：`research/results/` 里有一份未跟踪文件，崩溃验证员第 1 步两条核对都过，它写的全绿标记在 `--staged` 的 worktree 里对不上，整轮里这一道真跑 |
| A3 | D1 | 打中（与 QEMU 同形） | 观测一次 | 崩溃验证员第 1 步对这一道跑 `git diff --quiet -- <整个 research/results/ 在内的登记路径>`；本会话里看到过一次 `AM research/results/e167-…-r4.log`，那一刻这一道（和 QEMU）只能停下 |
| A4 | D1 | 打中（轻） | 量过 | 只打版本的 `--tool-versions` 被闸当成「Verus」重型拒，主 agent 与崩溃验证员不带前缀都拒 |
| A5 | D1 | 没打中 | 量过 | `verus-stage` 与 `lib_heavy_tests.py` 第 132、152 行对得上：崩溃验证员带前缀放、不带拒，门禁分诊单跑拒，实现员带前缀也拒；崩溃验证员定义里的四种命令形状都放行 |
| B1 | D2 | 打中 | 量过（草稿拷贝） | 有作数的全绿标记时，换了 Verus（version、commit、`verus` 的 sha256 全变）三条问法都复用：崩溃验证员第 1 步、阶段在工作区自问、`gate.sh --staged` 里自问，24 格里 24 格复用 |
| B2 | D2 | 打中 | 量过（草稿拷贝） | 照正常流程（崩溃验证员先判绿、整轮复用它），`environment=` 一次都不会被记下；`environment=` 只在「标记不在、有暂存树」那一条路上起作用 |
| B3 | D2 | 打中 | 量过（草稿拷贝） | 就算走到 `environment=`，它只哈希 `verus` 这个外壳；只换 `z3`、或设 `VERUS_Z3_PATH` 指别的 z3，照样复用 |
| C1 | D3 | 打中（文字层量过，出现条件推的） | 代码读的 + 现场两次读到 slice 已占九成 | 内存包装在格里退 250–254（排队满 3600 秒、撞上限）时，两份判法把它报成这一格红，给的「→」指向改变异行、修实现回归；崩溃验证员与门禁分诊的定义都没有把这一道列进「25x 另判」 |
| C2 | D3 | 没打中 | 推的 | 600 秒限时从 scope 起跑算、不算排队；单次 Verus 8–21 秒，32 核上要慢约 28 倍以上才撞 |
| C3 | D3 | 没打中 | 推的 | 同时起的四道之间不抢编译目录或锁：装置编在自己的 `CARGO_TARGET_DIR`，Verus 不经 cargo；跨会话共用那个 `/tmp` 目录只有互相作废重编的代价 |
| C4 | D3 | 打中（轻） | 推的 | 装置那个上限写不成「量过的」：峰值表的键带根路径与线程数，`--staged` 的 worktree 每趟一个新路径；今天表里没有它 |

## D1：谁跑、什么时候跑

### A1 判法脚本绕过闸（打中）

被判对象许可它的那几句：

- `.claude/hooks/lib_heavy_tests.py:62`「认成重型就验不了自己写的证明；只有门禁那一道（逐格验、空真与必红世界逐处起 Verus、编装置）算。」
- `.claude/hooks/heavy-test-guard.sh:48`「直接起的 verus 与 cargo verus 不算（写证明的实现员要用，判法在 lib_heavy_tests.py）」
- 闸只按阶段文件名认这一类：`.claude/hooks/lib_heavy_tests.py:152`「"checker-tier-formal-verification.sh": "verus-stage"}」

而规矩说间接起法同样算跑：`.claude/agent-common.md:53`「写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑」，重型的名单写的是这一道：`.claude/agent-common.md:52`「形式化验证（门禁 checker-tier-formal-verification）」。
这一道的六格只是把两份脚本接进门禁（阶段第 115–120 行逐格 `python3 "$VERUS_CHECK" <格> "$ROOT"`、`python3 "$PIN_CHECK" --replay`），直接跑脚本就是跑那一格。

派发情形（喂 JSON 的原样退出码，`research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt`）：

- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:20`「0	-	implementation-writer	实现员直接跑必红世界格的判法（94 行逐行起 Verus）」
- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:22`「0	-	implementation-writer	实现员直接跑红例重放格的判法（编并跑 checker 档装置）」
- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:27`「0	-	主 agent	主 agent 不带前缀直接跑必红世界格的判法」
- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:28`「0	-	investigator	调查员直接跑红例重放格的判法（investigator 第 5 步说这一道不跑）」
- 对照，同一位实现员带前缀跑门禁本身被拒：`research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:25`「2	2	implementation-writer	对照：实现员带前缀跑这一道」

一条具体的历史：实现员在写证明，派发提示照规矩有「重型测试：不跑」；它改完核想确认必红世界还红，跑 `python3 research/scripts/verus-proof-check.py must-red-worlds .`。闸放行；这一条按 `research/scripts/verus-proof-check.py:531`「    jobs = options["jobs"] or min(len(edits), max(1, options["threads"] // 4))」同时起若干个 Verus，逐行验 94 行。豁免直接起 verus 的理由是「验一个 crate 是秒级」（`.claude/hooks/lib_heavy_tests.py:61`「直接起的 verus 与 cargo verus 不算重型：写证明的实现员改一处验一次，验一个 crate 是秒级（接回可行性那一趟量到 8–21 秒）」），这个理由罩不住 94 次。

四句：
1. 分不分辨臂：这一格不判臂，判「有没有绕过闸」；病根只在这一轮新加的那一类（别的重阶段的驱动按名字认：`mutate.sh` 带 `crates/mutations.tsv`、`vm-bench.sh`、`lkmm.sh`、`broad-mutation.py`），不是共用前提。
2. 系统当时看不看得到：看得到，命令词就是 `research/scripts/verus-proof-check.py` 加子命令名，与 `broad-mutation.py` 按名字与参数认同一种做法。
3. 满足的分句：D1 问句「有没有一条派发情形会……绕过闸跑重型」。
4. 改法：见末尾改法表 G4（推的）。红例重放那一半弱一些：闸本来就不拦直接执行 checker 档的装置二进制（`d1-output.txt:24` 那一条 `/tmp/singlefs-e165-pinned-target/release/e165_replay_bounded_enumeration` 也放行，实验跑装置都这样），所以 A1 的分量在 Verus 三格上。

### A2 「这五道在 `gate.sh` 里只核标记」不成立（打中）

被判的句子：`.claude/main-agent.md:66`「这五道在 `gate.sh` 里只核标记与复用判定，它不直接调」；`.claude/rules/implementation-workflow.md:74`「提交时由 `crash-verifier` 跑，按 `.claude/gate.d/stage-inputs.tsv` 它那一行的输入复用。」

崩溃验证员第 1 步对这一道只查两个范围的未跟踪文件：`.claude/agents/crash-verifier.md:28`「checker-tier-formal-verification 查 `crates` 与 `research/results/e165-*-red-examples.tsv`」。
这一道登记的输入却是整个 `research/results/`：`.claude/gate.d/stage-inputs.tsv:31`「红例账是 research/results/e165-*-red-examples.tsv，按前缀只能取整个 research/results/」，而指纹把没被忽略的未跟踪文件也算进去：`research/scripts/admission.py:475`「        ["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],」。

历史（`d1b-untracked-results.sh`，草稿拷贝；`research/results/` 里放一份别的会话还在写的未跟踪 log）：

- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1b-output.txt:2`「git diff --quiet -- <登记路径> <阶段脚本> 退 0」
- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1b-output.txt:3`「git ls-files --others --exclude-standard -- crates 'research/results/e165-*-red-examples.tsv' 打出 0 行」
- 两样都过，崩溃验证员照第 2 步跑、判绿、写标记（`d1b-output.txt:5`，指纹 f3e0…）。
- `--staged` 的临时 worktree 里同一批内容：`research/prompts/defs-m3-formal-gate-r1-opus-model/d1b-output.txt:7`「stage-marker-check 退 1 ：这批输入（指纹 0d7e96d4f72f260e…）没有全绿标记」
- `research/prompts/defs-m3-formal-gate-r1-opus-model/d1b-output.txt:8`「gate-reuse 退 0 （0 = 要跑）：输入与上次整轮全绿那棵树相同，但这一道上一次判绿时的环境没记下」
- 对照：删掉那份未跟踪文件，工作区的指纹回到 0d7e…（`d1b-output.txt:9`），只差这一份文件。

于是门禁分诊起的整轮里这一道真跑：编 E165 装置、起全部 Verus，上限取阶段的默认值（门禁分诊的定义与派发要求里没有这一道的两个上限），不是「只核标记」。
那一行 `environment=` 也救不回来：照正常流程它从没被记下（见 B2），于是 `gate-reuse` 判「环境没记下，按要跑处理」。

四句：
1. 分不分辨臂：不分辨这一道与 QEMU。QEMU 那一行同样登记整个 `research/results/`，崩溃验证员第 1 步对 QEMU 明写不查那里的未跟踪文件；改前的句子「这四道……只核标记」对 QEMU 已经不成立，这一轮把这一道并进同一句。病根是「工作区判绿的标记」与「暂存树上的指纹」看的文件集合不同，共用前提，另立一笔账。
2. 看不看得到：看得到，`git ls-files --others` 与指纹用的是同一份清单。
3. 分句：D1「整轮 `gate.sh` 里它怎么走（只核标记与复用判定，还是真跑）」——答：标记对得上才只核，对不上就真跑。
4. 改法：改法表 G6（推的）。

### A3 崩溃验证员第 1 步因 `research/results/` 停下（打中，与 QEMU 同形）

`.claude/agents/crash-verifier.md:28`「跑 `git diff --quiet -- <这些路径>`（退 0 才说明工作区与暂存区在这一道的输入上相同）」，「这些路径」对这一道是 `stage-inputs.tsv` 第 31 行的全部 11 条，含整个 `research/results/`。
本会话里一次现查的原样输出（之后那份文件被别的会话暂存了，`observations.txt` 第 1–2 行是后来再查、已经没有）：

```
$ git status --porcelain research/results | grep '^AM'
AM research/results/e167-multi-inflight-crash-cut-mutate-2026-10-01-r4.log
$ git diff --quiet -- research/results; echo "diff-quiet rc=$?"
diff-quiet rc=1
```

那一刻派崩溃验证员，这一道（与 QEMU）照第 1 步停下报「工作区与暂存区不同」，要等写 E167 产物的会话暂存或撤掉。这一道真正读的只有 `research/results/e165-*-red-examples.tsv`，E167 的 log 与它无关。
四句：不分辨这一道与 QEMU（同一个「整个目录」的登记）；系统看得到（停下前它就知道是哪个文件）；分句是共用问句「哪一步会……做不了」；改法 G6 同时管 A2、A3（推的）。

### A4 `--tool-versions` 被当成重型（打中，轻）

阶段自己把它写成只打版本的查询：`.claude/gate.d/checker-tier-formal-verification.sh:61`「#   bash .claude/gate.d/checker-tier-formal-verification.sh --tool-versions   打 Verus 的版本」。闸只豁免两样查询参数：`.claude/hooks/lib_heavy_tests.py:148`「STAGE_QUERY_ARGUMENTS = frozenset({"--list", "--list-items"})」。
`research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:10`「2	-	crash-verifier	CV 不带前缀打 --tool-versions（只打版本）」；主 agent 不带前缀同样退 2（`d1-output.txt:12`）。
派发情形：门禁分诊或主 agent 看到「这一道的环境与上次判绿时不同」想对一下此刻装的 Verus，跑这条被拒。`admission.py` 自己经 subprocess 起它不过闸，所以门禁本身不受影响。四句：只管这一道；看得到（参数就在命令里）；分句是「有没有一条派发情形会被闸拒」；改法 G5（推的）。

### A5 闸与定义对得上的部分（没打中）

`.claude/hooks/heavy-test-guard.sh:175`「"herd7-stage", "lkmm", "herd7", "crates-mutation-stage", "crates-mutation-mutate", "verus-stage"} | COMMIT_CHECK_KINDS,」与 `.claude/hooks/lib_heavy_tests.py:132`「"verus-stage": "Verus",」、第 152 行对得上。逐条（`d1-output.txt` 第 1–19 行）：崩溃验证员照第 1b、2 步的四种写法（上限写在 `nice` 前、上限写在前缀前、`{ …; echo "exit=$?"; } > log` 后台、与 54 号并行的 `.rc` 写法）都退 0；它单跑 `--check verus-proof` 带前缀退 0、不带退 2；它多包一层外层内存包装闸也放（定义写了不包，闸不拦，不算打中）；门禁分诊带前缀跑 `gate-staged.sh` 退 0，带前缀单跑这一道或一格都退 2（`research/prompts/defs-m3-formal-gate-r1-opus-model/d1-output.txt:18`「2	2	gate-triage	GT 带前缀单跑一格 --check verus-proof」），与 `.claude/agents/gate-triage.md:28`「这几道重阶段在 `gate.sh` 里照各自的复用判定走」后面那句「你不直接调它们」一致。
边角：同一行后面「单跑某一道或某一格（`--check <格名>`）也带同一个前缀」脱开上下文可以读成门禁分诊能带前缀单跑这一道；它后面括号写的是「提交时才跑的检查一类」，照括号读不冲突。

## D2：复用

### B1 有全绿标记时换了 Verus 照复用（打中，把 4-5「读代码推的，没验」坐实）

许可它的代码：`research/scripts/admission.py:1126`「    if not break_is_set("stage-marker-ignored"):」起先看标记，作数就直接返回，`research/scripts/admission.py:1134`「                return EXIT_GATE_MAY_SKIP, f"这一道这批输入（指纹 {fingerprint[:16]}…，{file_count} 个文件）有全绿标记：{lines[0]}"」；环境只在后面那条路上比：`research/scripts/admission.py:1178`「        return judge_gate_environment(root, stage, stage_rows, staged_tree, input_paths, reason)」。标记的指纹不含环境：`research/scripts/admission.py:2217`「    named_lines = [(f"<登记行：{stage}>", registration_text), toolchain_line(root)] + build_environment_lines(root)」；登记行进指纹的只有键与路径，见 `research/scripts/admission.py:298`「        return f"{self.key}\t{' '.join(self.input_paths)}\n"」。
被判的文字说的是另一回事：`.claude/gate.d/stage-inputs.tsv:31`「Verus 的 version、commit 与可执行文件的 sha256 经 environment=（本阶段 --tool-versions，判绿之后记进 git common-dir）进复用判定」；`.claude/gate.d/checker-tier-formal-verification.sh:36`「登记在 .claude/gate.d/stage-inputs.tsv 本阶段那一行的路径与 environment=（本阶段 --tool-versions 打的 Verus 版本）都没变就整道退 77；」。

装置（`d2-reuse.sh`）：拷一份仓（`admission.py`、`stage-inputs.tsv`、`verus-proof-check.py`、阶段脚本与仓里逐字节相同，`d2-output.txt` 第 2–5 行），三份假的 Verus 安装（只有 `version.json`、`verus`、`z3`）：A 是登记的版本，B 版本、commit、`verus` 全换，C 只换 `z3`；另一臂是 A 加 `VERUS_Z3_PATH` 指 B 的 z3。tool-versions 对 A、B、C 打的行在 `d2-output.txt` 第 7–9 行（A 与 C 逐字相同）。
只固定前缀（一次判绿、之后输入一字不改），放开扫用户与流程决定的几步：换 Verus 的四种方式 × 判绿那一趟由谁跑（H1 崩溃验证员在工作区、H2 `gate.sh --staged` 里真跑）× 之后标记在不在（H3、H4 删标记）× 三条问法（q1 崩溃验证员第 1 步 `stage-marker-check`，q2 阶段在工作区自问，q3 `gate.sh --staged` 里阶段自问）。4 × 4 × 3 = 48 格，全部取到。

汇总（`d2-summary.txt` 第 1–17 行，原样）：

```
### d2-output
H1 不换（A）：三条路里复用 3/3
H1 换版本（B：version、commit、verus 都变）：三条路里复用 3/3
H1 只换 z3（C：version.json、verus 不变）：三条路里复用 3/3
H1 A 加 VERUS_Z3_PATH 指别的 z3：三条路里复用 3/3
H2 不换（A）：三条路里复用 3/3
H2 换版本（B：version、commit、verus 都变）：三条路里复用 3/3
H2 只换 z3（C：version.json、verus 不变）：三条路里复用 3/3
H2 A 加 VERUS_Z3_PATH 指别的 z3：三条路里复用 3/3
H3 不换（A）：三条路里复用 1/3
H3 换版本（B：version、commit、verus 都变）：三条路里复用 0/3
H3 只换 z3（C：version.json、verus 不变）：三条路里复用 1/3
H3 A 加 VERUS_Z3_PATH 指别的 z3：三条路里复用 1/3
H4 不换（A）：三条路里复用 0/3
H4 换版本（B：version、commit、verus 都变）：三条路里复用 0/3
H4 只换 z3（C：version.json、verus 不变）：三条路里复用 0/3
H4 A 加 VERUS_Z3_PATH 指别的 z3：三条路里复用 0/3
```

H1、H2 的「换版本」两行就是 D2 那句话的反例：`research/prompts/defs-m3-formal-gate-r1-opus-model/d2-summary.txt:3`「H1 换版本（B：version、commit、verus 都变）：三条路里复用 3/3」。复用期上限是 24 小时（标记的 `finished_utc` 与 `refs/sop/staged-green` 的提交时刻各自判），这 24 小时里换 Verus 看不见；`research/scripts/stage-must-run.sh:36`「**环境变了而内容没变**——rustc 升级、系统库换了，树一模一样而结论已经不作数。」与第 38 行「这里用**复用上限**替代」说的正是这一格，`environment=` 登记的本意是补它，而标记那条路绕过了它。

四句：
1. 分不分辨臂：不分辨这一道与 herd7 那一道（`stage-inputs.tsv` 第 30 行同样登记了 `environment=`、同样写标记）。病根在 `gate_reuse` 的次序与标记指纹，是共用前提，改法落在 `admission.py`，不在被判的 11 份文件里。
2. 看不看得到：看得到。判复用那一刻 `current_gate_environment` 就能跑那条 `--tool-versions`（只读两个小文件、哈希一个可执行文件，开销没量，推的），草稿改法 F1 就是这样做的，判定结果量过。
3. 分句：D2 第二分句「`gate_reuse` 只要全绿标记作数就不再比 `environment=`，24 小时内换了 Verus、输入没变时会复用旧判定」，字面成立，三条问法都中，不只 `gate.sh` 那一条。
4. 改法：F1 修 B 那几格，F1 加 F2 修全部（量过，见改法表）。

### B2 照正常流程，`environment=` 一次都记不下（打中）

规定的流程是崩溃验证员先在工作区跑这一道（`.claude/main-agent.md:66` 那一行），阶段判绿后照第 144、147 行先记环境、再写标记：`.claude/gate.d/checker-tier-formal-verification.sh:144`「if ! python3 "$ADMISSION_MODULE" gate-record-environment "$ROOT" "$(basename "$0")"; then」。
崩溃验证员那一趟没有暂存树，记环境那一步什么都不做就返回：`research/scripts/admission.py:1081`「        staged_tree = os.environ.get("SINGLEFS_STAGED_TREE", "")」，下面两行 `if not staged_tree:` 返回空串。
`research/prompts/defs-m3-formal-gate-r1-opus-model/d2-output.txt:11`「[H1 判绿] gate-record-environment：（空 = 一个字都没打）」，接着整轮里阶段命中标记、整道退 77（`research/prompts/defs-m3-formal-gate-r1-opus-model/d2-output.txt:13`「[H1 整轮] gate-reuse 退 10」），也不记。
所以 H1 这条最常见的历史里，环境记录只在「整轮真跑了这一道」时才有；标记一过期，下一趟走到暂存树那条路，得到的是「环境没记下，按要跑处理」（汇总 H4 四行全是 0/3）。方向是多跑，不错判，但它说明 `stage-inputs.tsv` 第 31 行那句「经 environment= 进复用判定」在规定流程里几乎不生效。
四句：同 B1（共用前提）；看得到（崩溃验证员那一趟能现跑 `--tool-versions` 记下来，只是没有树可记）；分句同 B1；改法 F1 让环境进标记本身，不再依赖这一份记录（量过：F1 下 H1 换版本 0/3）。

### B3 走到 `environment=` 也只看 `verus` 外壳（打中）

`research/scripts/verus-proof-check.py:721`「    executable = os.path.join(directory, "verus")」，只哈希它。装的 Verus 目录里判定真正取决于的另有 `rust_verify`、`z3`、`vstd.vir`、`libvstd.rlib`（`observations.txt` 末尾 `ls` 原样列出这几样都在），两份二进制里都有 `VERUS_Z3_PATH` 这个串（`observations.txt` 同段，`verus` 1 处、`rust_verify` 3 处；这一点只是字符串在，没起 Verus 验它的效果，推的）。这几样都不在指纹里：`build_environment_lines` 只认 `RUSTFLAGS`、`CARGO_*` 这一类变量（`research/scripts/admission.py` 第 266–275 行）。
`research/prompts/defs-m3-formal-gate-r1-opus-model/d2-summary.txt:12`「H3 只换 z3（C：version.json、verus 不变）：三条路里复用 1/3」：标记不在、走到环境比对那一格（q3），只换 z3 照样复用；`VERUS_Z3_PATH` 那一臂同样 1/3（第 13 行）。对照：同一条路上换版本的 B 是 0/3（第 11 行），比对本身起作用。
rlimit 登记成 10（`.claude/gate.d/verus-pins.tsv` 的 `rlimit` 行），必红世界有「只红在 rlimit 上不作数」的判法，而 rlimit 计数跟 z3 的版本走（推的，没量）；换 z3 而复用旧判定，正是必红世界最怕的那一种漂移。
四句：分辨不了「换 z3」与「没换」——系统在判的那一刻看得到 z3 的内容，只是没去看；分句是 D2 第一分句「登记的输入与 `environment=` 够不够」；改法 F2（量过，与 F1 同上）。

## D3：并行与内存上限

照定义并行起的是：`.claude/agents/crash-verifier.md:31`「checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-formal-verification 与 `checker-tier-layer0-replay` 并行起」，这一道「也在里面套了」：`.claude/agents/crash-verifier.md:29`「checker-tier-formal-verification 也在里面套了（每次 Verus、E165 装置各经包装）」。两个变量名与代码对得上（`verus-proof-check.py` 第 737 行读 `GATE_VERUS_MEMORY_MAX`，`e165-pin-red-examples.py` 第 411 行读 `GATE_E165_PINNED_MEMORY_MAX`）。

### C1 包装在格里退 250–254 被报成「这一格红」，「→」指错方向（打中：文字层量于代码，出现条件推的）

两份判法把包装自己的结局并进这一格的红：
- Verus：`research/scripts/verus-proof-check.py:356`「        return f"内存包装自己的结局（退 {outcome['exit']}，{outcome['wrapper']}），这一次不算结果"」，在必红世界格里作为一行的问题判红，给的下一步是 `research/scripts/verus-proof-check.py:653`「"必红世界要红在证明上：改坏之后编不过的变异不算，换一处只改判定的改法"」。
- 装置：包装的退出码就是 `subprocess.run` 的退出码（`research/scripts/e165-pin-red-examples.py:378`「        completed = subprocess.run(command, cwd=root, capture_output=True, text=True, env=environment)」），非 0 就判红，下一步是 `research/scripts/e165-pin-red-examples.py:400`「"今天的实现在钉住的输入上与原实现不同：是改动引入的回归就修实现；装置报指纹漂了就先查装置的枚举次序与状态编码变没变"」。
定义里接不住：崩溃验证员的「250–254 照那一条办」挂在第 1b 步外层包装的退出码上，这一道照定义不包外层，外层看到的是阶段退 1；它照第 3 步原样抄「✗」与「→」（`.claude/agents/crash-verifier.md:32`「3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」」）。门禁分诊的另判只点名两道：`.claude/agents/gate-triage.md:29`「✗ 行写着内存包装退 25x 的（research-unit-tests 格与 harness-model-differential-and-scenarios 的」。
出现条件（推的）：带前缀的重阶段排队，`research/scripts/run-with-memory-cap.sh:54`「等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（默认 3600）秒还放不下，列出账上占着的每一条，退 252。」；一条要的量取峰值表里的实测峰值（`research/scripts/run-with-memory-cap.sh:46`「这一条要的量：峰值表里这条命令（键见下）上一次实测的峰值，不超过它的上限」）。slice 总上限 40.1 GiB（`observations.txt` 的 `--status` 段）；54 号整条实测 24G（同一份 `observations.txt`，峰值表第 301 行），QEMU 的上限不小于 6 档 × 2048M = 12G，两道就占 36G；本会话两次读 `--status`，别的会话账上已占 39.0 GiB（第一次，12.0 + 24.0 + 3.0，原样没存）、36.7 GiB（第二次，`observations.txt` 原样）。装置那条的键没测过、按上限 2G 算（见 C4），在这种 slice 上等一小时排不上是可达的；Verus 每条实测约 313 MiB，难饿死。
历史：主 agent 暂存后派崩溃验证员，别的会话正跑一个 14G、一个 12G 的长活；54 号与 QEMU 先排上，红例重放那条装置 2G 等满 3600 秒退 252；阶段报「钉住的 N 行在今天的实现上：装置退 252……」，「→」叫人修实现回归。崩溃验证员原样抄成红，主 agent 拿到的是一条指向实现的红。
四句：分辨这一道与另两道（54 号、QEMU、herd7 外层包了，结局落在外层，有定义接）；系统看得到（退出码 250–254 是包装自己的、判法里已经有 `WRAPPER_OWN_EXITS` 表）；分句是 D3「两个上限」「阶段里面已经包了内存不在外面再包」；改法 G7（推的）。

### C2 600 秒限时在并行争 CPU 时撞不撞（没打中，推的）

限时交给包装：`research/scripts/verus-proof-check.py:291`「        environment.update(RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(timeout), RUN_WITH_MEMORY_CAP_KEY=f"verus-proof-check {key}",」，包装从 scope 起跑算：`research/scripts/run-with-memory-cap.sh:86`「给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间」。所以 C1 那种排队不吃这 600 秒。
单次 Verus 8–21 秒（`lib_heavy_tests.py:61` 那句）；本机 32 核（`observations.txt`）。同时在跑的：54 号快档、QEMU 6 台各 4 vCPU（`vm-bench.sh` 第 15 行默认 `VM_CPUS` 4）、herd7、这一道的 J 个 Verus（J 按第 531 行，线程上限 32 时是 8，每个 4 线程）。全都 `nice -n 19`，可运行线程合计粗估 32 + 24 + 1 + 32 ≈ 89，按公平分时慢约 3 倍；要撞 600 秒得慢约 28 倍（600 ÷ 21）。没量，推的。rlimit 是 z3 的资源计数，不随墙钟变，CPU 争用不会把必红世界推成「只红在 rlimit 上」。
推翻条件：在提交时那种负载下量到一次 Verus 超过 200 秒。

### C3 抢编译目录或锁（同时起的几道之间没打中，推的）

- 装置编在自己的目录：`research/scripts/e165-pin-red-examples.py:376`「        environment = dict(os.environ, CARGO_TARGET_DIR=options["target"])」，默认 `research/scripts/e165-pin-red-examples.py:412`「"target": os.path.join(os.environ.get("GATE_CROSS_RUN_TMPDIR") or os.environ.get("TMPDIR") or "/tmp", "singlefs-e165-pinned-target")}」；54 号快档编在仓里的 `target/`，两者不争同一把编译目录锁。Verus 直接起、不经 cargo。
- 共用的只有 cargo 的包缓存（`CARGO_HOME`），`--locked` 且包都在时只在解析那一下拿锁（cargo 的行为，没在本机量，推的）。
- 编装置那一步不经内存包装（第 377 行就是 `nice -n 19 cargo build`），在 slice 外面吃内存，靠包装的 20G 余量兜（推的）。今天 `/tmp/singlefs-e165-pinned-target` 不在（`observations.txt`），第一趟是从零编 release 的 checker 档，与 54 号快档的 release 编译同时抢 CPU，只是慢。
- 跨会话：那个目录不分根、不分会话。两个会话从不同的根先后编，cargo 按路径作废对方的指纹、整批 crate 重编，后来的那一个等前一个放锁（`Blocking waiting for file lock on build directory`）。我试着造「A 编完、B 编完覆盖、A 才执行」的串：B 拿到锁后要重编工作区里的 crate，A 在放锁之后几毫秒就起了二进制，换上是改名，已经起来的进程不受影响；窗口造不出来。记成只有代价、不错判。

### C4 装置那个上限写不成「量过的」（打中，轻）

`.claude/agents/crash-verifier.md:22`「每个 Verus 进程一个（空真与必红世界几处同时验，每处各占一份）」，与装置那一个，各要写量过还是推的。Verus 那三个键表里有（`observations.txt`：must-red-worlds 313 MiB、proof 202 MiB、vacuity 207 MiB，线程列 4）。装置那条的键是命令原样拼起来（`research/scripts/run-with-memory-cap.sh:67`「键：RUN_WITH_MEMORY_CAP_KEY 给了就用它；不给就把命令的各个词用空格接起来」），含 `capped.sh` 的线程数、二进制的绝对路径与表的绝对路径（`e165-pin-red-examples.py` 第 384、386 行），`--staged` 的 worktree 每趟换一个临时目录，键每趟都是新的；表里带 `--pinned-cases` 的 8 行都是 3-6 草稿里的二进制路径（`./bin-after`、`./bin-final` 这一类，峰值 11 MiB），没有门禁那条。后果：主 agent 只能写「推的」，排队时这一条按上限（默认 2G）算，比实测的 11 MiB 大两百倍，加重 C1。

## 改法表（每一格标量过 / 推的；全部只在我的模型上量过、被攻过零轮）

| 改法 | 改哪 | 修哪几格 | 状态 |
|---|---|---|---|
| F1 | `admission.py` 的 `stage_fingerprint`：登记了 `environment=` 的阶段，环境的 sha256 进标记的指纹（`fix-draft.py`） | B1 的 B 臂、B2 | 量过：`d2-summary.txt` 第 20、24 行 H1、H2 换版本 0/3，A 臂仍 3/3（第 19、23 行）；C、Z 臂仍 3/3（第 21、22 行），对 B3 不起作用 |
| F2 | `verus-proof-check.py` 的 `tool_versions`：另打 `rust_verify`、`z3`、`vstd.vir`、`libvstd.rlib` 的 sha256 与 `VERUS_Z3_PATH` 指的文件 | B3 | 与 F1 一起量过：`d2-summary.txt` 第 37–39、41–43、45–47 行全 0/3，A 臂 H1、H2 仍 3/3（第 36、40 行）。F2 单独：只修 H3 那条路，标记那条路照旧复用（推的，没单独跑） |
| F3 | 定义：崩溃验证员第 1 步对这一道先跑 `--tool-versions` 与上一次记下的比，不同就带 `SINGLEFS_GATE_FULL=1` 跑 | B1 | 推的；它要一份「上一次」的记录，而 B2 说明规定流程里没有这份记录，单改定义落不了地；还撞 A4（不带前缀跑 `--tool-versions` 被拒） |
| G4 | `lib_heavy_tests.py`：`verus-proof-check.py` 的 `proof`、`vacuity`、`must-red-worlds` 与 `e165-pin-red-examples.py --replay` 按名字归「Verus」一类；`toolchain`、`tool-versions`、`fingerprint`、`--check`、`--selftest` 不算 | A1 | 推的。不动「直接起 verus 不算」这条豁免 |
| G5 | `STAGE_QUERY_ARGUMENTS` 加 `--tool-versions`（或只对这一道认） | A4 | 推的 |
| G6 | 崩溃验证员第 1 步对这一道的未跟踪核对改查整个 `research/results/`（与指纹看同一批文件）；或 `admission.py` 让登记行认 glob，这一行只登记 `research/results/e165-*-red-examples.tsv` | G6 前一半修 A2（不再写出对不上的标记，换成停下），不修 A3、反而多停；后一半同时修 A2、A3，并让这一道不再被别的实验的产物挡（QEMU 那一行另说） | 推的；后一半要改 `admission.py` 的登记语法，门禁 doc-registries 那一类可能也要跟 |
| G7 | 两份判法把包装自己的 250–254 判成「本次未跑」或单独一类并给「环境 / 并发」的出路；定义一侧把这一道加进门禁分诊第 3 步的 25x 名单、崩溃验证员第 3 步加一句 | C1 | 推的。只改定义一侧：「→」仍是错的，分诊员要靠 ✗ 行里的「退 252」认出来 |
| G8 | `e165-pin-red-examples.py` 起装置时给 `RUN_WITH_MEMORY_CAP_KEY` 一个不带路径与线程数的固定键 | C4（连带减轻 C1） | 推的 |

F1 与 F2 都改 `admission.py` / `verus-proof-check.py`，不在这一轮被判的 11 份文件里；它们修的是这 11 份文件里几句描述（`stage-inputs.tsv` 第 31 行、阶段第 36 行）背后的实现。只改文字（把那几句改成「标记作数期间不比环境」）也能让字面与实现一致，但留着 B1 那个洞。

## 没打中的形状与取样范围

- D1 hook：22 条 JSON（`d1-hook-cases.py`），角色取主 agent、崩溃验证员、门禁分诊、实现员、调查员；命令形状取前缀与两个上限的两种排法、后台写进日志、与 54 号并行的 `.rc` 写法、单跑一格带与不带前缀、多包一层外层包装、`--list`、`--tool-versions`、两份判法脚本直接起与经包装起、直接执行装置二进制。期望写死的 13 条全对上（`d1-output.txt` 末行「对不上期望的：0 条」），没打中的就是 A5 那几条。没试：`detached-run.sh start -- …` 这条路（子 agent 用它照拒，与这一道无关）、写进脚本再执行（闸读 shell 脚本正文，这一类已有自检格）、派发闸 `runner-dispatch-guard.sh`（这一轮只改了它的一句提示文字，判法没动）。
- D2：48 格（4 段历史 × 4 种换法 × 3 条问法），全部取到；另在两份打了草稿改法的拷贝上各 48 格。没试：24 小时上限本身（`SINGLEFS_REUSE_HOURS` 的边界），这一格与这一轮的改动无关；rustc 升级（`cargo -V`、`rustc -V` 进指纹，看过代码，没造）。
- D3：没有一条能在不跑重型的前提下实测，C2、C3 都是读代码加 `--status` 现场读数推的；C3 那条「先编后执行」的竞态按 cargo 的锁与改名换上推不出可达的窗口，没造出来。
- 共用问句里另看过、没打中的：崩溃验证员第 1 步先 `stage-marker-check`、复用就记「复用」不跑，这一道照样适用（`d2-output.txt` H1、H2 的 q1 行退 0）；带 `--check` 时阶段不复用、也不写标记（阶段第 123、142 行），单跑一格不会留下让整轮误复用的标记。

## 这条腿自己的限度

- 假的 Verus 安装只有 `version.json`、`verus`、`z3`，没有起过真的 Verus；「换 z3 会改变判定」这一句是推的（B3 只证明了「换了也看不见」）。
- H1–H4 的「`gate.sh --staged`」是在草稿拷贝本身上带 `SINGLEFS_STAGED_TREE` 调 `gate-reuse` 模拟的，不是真起 `gate.sh --staged`；只有 A2 用了真的 `git worktree`。指纹只看内容，拷贝的路径不同不影响结论（同一份拷贝里 q1 与 q3 命中同一个标记可证）。
- 草稿拷贝取自复跑那一刻的工作区，别的会话的未提交改动也在里面；判定用到的 4 份文件与仓里逐字节相同（`d2-output.txt` 第 2–5 行），指纹值每次复跑都会变，计数不变。
- C1 的出现条件靠两次 `--status` 读数与 54 号、QEMU 的上限推，没量 54 号快档与 QEMU 那一道实际跑多久，所以「一小时排不上」是可达而不是常见，频率说不出。
- 第一轮、单次抽样：「没打中」的几格（A5、C2、C3）照规矩不算数。
- 副本上量出的数（F1、F2 那几行）不算入库装置上的数。

## 没做什么

- 没跑任何重型测试、门禁阶段、`--selftest`、`gate.sh`；被判的命令一条都没执行；没起 Verus，没编 Rust。
- 没改仓里除这份报告与模型目录之外的任何文件；F1、F2 只打在草稿拷贝里（`fix-draft.py`）。
- 不判 D4、D5（归正推腿）；不替主 agent 采纳；没读禁读清单里的两样。
- 草稿目录 `/tmp/claude-1000/defs-m3-formal-gate-r1-opus/` 里的仓拷贝在交回前删掉（交回里写明删了哪些）。
