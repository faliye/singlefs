# m3-broad-gate-gov-r1：云端正推腿报告（Sonnet，G1、G3）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 C6 E4 -->

立场：正推。从用户定案与判决推「应该是什么」，再与被判对象（工具实现员报告第二节、冻结副本里今天的样子）逐格比。
腿读的是冻结副本 `/tmp/claude-1000/m3-batch5-freeze/tree/`（下称冻结副本）与主仓里的判决文件；引文写「文件:行号」，行号是在被引文件里 `grep -nF` 现取的，不是背景材料里的行号。
引冻结副本里的文件，路径从冻结副本根起（`.claude/…`、`research/scripts/…`、`crates/…`）；引判决与正文的，路径从主仓根起（`research/prompts/…`）。
不判别的格（G2、G4、G5）不在这里。

## G1　门禁两格判的与用户定案、第三版 C6 逐句对不对得上

### G1.0　先说判据从哪来

- 判决文件 `research/prompts/tooling-broad-mutation-prune-r3-main-verification.md` 里没有 C6 全文，只有用户定案那一句与 E1–E6 的补丁；C6 的全文在 `research/prompts/_tooling-broad-mutation-prune-r3-body.md` 第三节。两处各抄一句并排：
  - C6 第一分句：「这次改到的变异每一条都有这一版的结论；沿用的结论都在；有任何一条没红就红。」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:44`）
  - C6 第二分句：「第二段基线先修绿：测试读的仓内文件进导出范围，或改测试不读导出树之外的文件。」（`research/prompts/_tooling-broad-mutation-prune-r3-body.md:45`）
  - 用户定案：「用户 2026-10-01 弹窗定」（`research/prompts/tooling-broad-mutation-prune-r3-main-verification.md:50`），内容是提交前跑完才能提交，同一行还有「每次提交都要过、红了不能提交」。
  - E4 补的两条：「全局键变了就整版重判」（`research/prompts/tooling-broad-mutation-prune-r3-main-verification.md:43`）与「无效的条每次重判，不沿用。」（`research/prompts/tooling-broad-mutation-prune-r3-main-verification.md:44`）。
- 所以「第三版 C6」的判据是五句：①改到的每条有这一版的结论；②沿用的结论都在；③有一条没红就红；④第二段基线先修绿；⑤E4：全局键变了整版重判、无效条每次重判。再加用户定案：⑥跑完才能提交（没跑完不能提交，红了不能提交）。

### G1.1　逐句对照（门禁是 `.claude/gate.d/checker-tier-broad-mutation.sh`，驱动是 `research/scripts/broad-mutation.py`）

| 判据 | 门禁 / 驱动今天做什么 | 判定 |
|---|---|---|
| ① 每条有这一版的结论 | 驱动把「有结论」定成三种结局：「DECISIVE_OUTCOMES = ("caught", "missed", "unviable")」（`research/scripts/broad-mutation.py:306`）；不在这三种里的（超时、内存撞顶、只过第一段、驱动错、没跑）都进 NO_CONCLUSION：「without = [item for item in library.items if (states.get(str(item["sequence"])) or ItemState(NOT_RUN, False)).state not in DECISIVE_OUTCOMES]」（`research/scripts/broad-mutation.py:2043`）。门禁一格再核计数：「if total <= 0:」（`.claude/gate.d/checker-tier-broad-mutation.sh:335`）与「if verdict == "green" and (missed or without or concluded != total):」（`.claude/gate.d/checker-tier-broad-mutation.sh:329`）。判的是这一版的全部 total 条，不只「改到的」那几条 | 一致，且比 C6 字面更严（改到的、没改到的都要有结论；没改到又没有可沿用结论的，驱动会现判，见 `plan_changed_since`） |
| ② 沿用的结论都在 | 沿用的条在这一版结果表里落一行 `carried`：「if judge or sequence in carried or (sequence in fresh and fresh[sequence].state in DECISIVE_OUTCOMES):」（`research/scripts/broad-mutation.py:2447`）是写这行的跳过条件；门禁只看这一行在不在（`concluded = total - len(without)`），不回头核结论表 `conclusions.tsv` 里那一行还在不在、全局键还对不对 | 一致于「在」；「还作不作数」C6 没说，E4 暗含（见 G1-c） |
| ③ 有一条没红就红 | 驱动：「gate_red = bool(without or counts["missed"])」（`research/scripts/broad-mutation.py:2047`），判红退 8（`EXIT_GATE_RED = 8`，`research/scripts/broad-mutation.py:304`）；门禁：「if verdict == "red" and summary_exit == 8 and "summary-red-passes" not in broken:」（`.claude/gate.d/checker-tier-broad-mutation.sh:314`）判红并抄 MISSED / NO_CONCLUSION 明细；判定行写 green 而计数不是 0、明细行数与计数不同、判定行与退出码对不上，也判红 | 一致 |
| ④ 第二段基线先修绿 | 代码侧已改成第二个出路：登记表不在（只导出 crates/ 的树）时测试打 `CRASH_CASE_REGISTRY_ABSENT` 并 return，不 panic：「广谱变异的导出树就是这样」（`crates/singlefs-checker-tier/src/crash_identity.rs:1518`），三处用法在 1552、1572、1594 行。这一格不是门禁的格：基线红，驱动在 ③ 退 6、一条结论都不落，门禁的 gate-verdict 因此 without_conclusion 大于 0 判红 | 句子一致；基线真绿没有证据：`/tmp/claude-1000/m3-r1/broad-smoke3/smoke.sh` 起的是 `--skip-stage-two`，没跑第二段基线；真跑是重型，复核不了（见 G1-f） |
| ⑤ E4：全局键与无效条 | 无效条：「if record["outcome"] == "unviable" and not broken("unviable-carried"):」（`research/scripts/mutation-prune.py:646`），沿用时一律拒、现判；全局键：「if record["global_key"] != global_key and not broken("global-key-ignored"):」（`research/scripts/mutation-prune.py:644`），键里含 `.cargo/`、Cargo.toml、Cargo.lock：「if os.path.basename(path) in ("Cargo.toml", "Cargo.lock") or path.startswith(".cargo/"):」（`research/scripts/broad-mutation.py:2220`）。门禁两格不比全局键，只比源码逐文件、导出范围、rustc -Vv 与版本指纹（含判法版本）：「driver.version_key(home, staged_source_fingerprint, current_toolchain) != fingerprint」（`.claude/gate.d/checker-tier-broad-mutation.sh:246`） | 沿用那一步一致；门禁一侧只罩住被盖了戳的判法版本与 rustc（见 G1-c） |
| ⑥ 跑完才能提交 | 没跑 ②③ 直接跑门禁：library-version 格红（库目录那一版不是这棵树），gate-verdict 格红（没结论）；两格都红退 1。但「不提交」只由句子要求：`.git/hooks/pre-commit` 只有东京时间窗口一条（第 2 行：「提交窗口闸：只在东京时间 20:00–02:00 放行。」，转述自 `/home/fy5090/code/singlefs/.git/hooks/pre-commit`，该文件没有扩展名，cite-check 认不出）；`refs/sop/staged-green` 在 `.claude/hooks`、`.claude/settings.json` 里零命中（命令与输出见 G1.3） | 门禁侧一致；没有机械拦截，靠 `.claude/main-agent.md:67` 与 `.claude/rules/implementation-workflow.md:53` 两句（见 G1-e） |

### G1.2　找输入：该红不红、不该红而红

下面每条写：输入、门禁今天的走法（行号）、判定（复现了 / 推的）、什么现象会推翻。门禁整道与驱动的真跑都是重型，这一腿没跑；能轻跑的（`admission.py` 的标记判定）在草稿目录的一次性仓里跑了。

**G1-a　复用标记按工作区文件算指纹，门禁判的是暂存区——暂存区与工作区在输入上不同时，标记可能对着没判过的那棵暂存树作数（该红不红，已在标记这一层复现）**

- 起因：commit-stages 第 1 步对这一道另写了一条捷径：「checker-tier-broad-mutation 判的是暂存区（git 索引）上的源码，不是工作区」「第 2b 步的驱动也不起」（`.claude/skills/crash-test/commit-stages.md:19`），即开跑前先 `stage-marker-check`、退 0 记复用、不跑 `git diff --quiet` 与未跟踪那两条；同一行对别的四道写的是「退 0 才说明工作区与暂存区在这一道的输入上相同」（`.claude/skills/crash-test/commit-stages.md:19`）。
- 推：标记的指纹由 `admission.py` 算，列文件用「["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],」（`research/scripts/admission.py:477`），再逐个读磁盘文件：「digest_hex = sha256_of_file(os.path.join(os.fsencode(root), name))」（`research/scripts/admission.py:693`）。读的是工作区，不是索引。广谱那一道在主工作区跑时自己写的标记也是按这个算的（`.claude/gate.d/checker-tier-broad-mutation.sh:351`、`:377` 一带）。
- 复现（草稿目录 `/tmp/claude-1000/m3-broad-gate-gov-r1-sonnet/marker-repro/` 里建的一次性仓，登记行 `x.sh<TAB>a.txt`，用冻结副本的 `research/scripts/admission.py`；报告交回前删掉），原样输出：

```
--- index a.txt:
v1
--- workspace a.txt:
v2
.../marker-repro/.git/singlefs-stage-green.x.sh.157ea281...      <- stage-marker-write（索引 v1、工作区 v2）
write rc=0
--- check, index=v1, workspace=v2:
ok .../singlefs-stage-green.x.sh.157ea281... 2026-10-02T12:17:56Z
check rc=0
--- after git add (index now v2, workspace unchanged); index a.txt:
v2
ok .../singlefs-stage-green.x.sh.157ea281... 2026-10-02T12:17:56Z
check rc=0
--- index=v3, workspace=v2:
v3
v2
ok .../singlefs-stage-green.x.sh.157ea281... 2026-10-02T12:17:56Z
check rc=0
```

- 读法：标记写下时索引是 v1，之后索引变成 v2、再变成 v3，工作区一直是 v2，`stage-marker-check` 三次都退 0。几个会话共写一个仓、`stage-mine.py` 只暂存自己的文件时，「工作区不变、暂存区多了别的文件」是日常形态。
- 打中的话改哪：commit-stages 第 1 步里广谱变异那半句改成与别的四道一样先 `git diff --quiet -- <这一道登记的路径>`（或这一道的标记判定改读索引）；整轮 `gate.sh --staged` 在临时 worktree 里按暂存树算，不受这条影响（`.claude/singlefs-ai-sop/scripts/gate.sh` 里 `git apply --index`）。
- 推翻它的现象：`stage-marker-check` 的指纹其实读索引（上面三次 check 有一次退 1）；或这一道的标记只在 `git diff --quiet` 为空时才被写、被查（没有这样的代码，`.claude/gate.d/checker-tier-broad-mutation.sh:372` 到 `:379` 只在 `real_run` 且 stage_exit 为 0 时写）。

**G1-b　两格各读一次 `tree.source`，`--summary` 不加锁：两格之间库目录被别的会话换树，两格可能判到两个版本（该红不红，推的，没复现）**

- 门禁两格各自开子 shell（「参数解析、每格一个子 shell、汇总与退出码经共用库 lib/stage-cells.sh。」，`.claude/gate.d/checker-tier-broad-mutation.sh:13`）。第一格读版本指针：「source_path = os.path.join(home, "tree.source")」（`.claude/gate.d/checker-tier-broad-mutation.sh:125`），第二格另读一次：「with open(os.path.join(home, "tree.source"), encoding="utf-8") as handle:」（`.claude/gate.d/checker-tier-broad-mutation.sh:293`），第二格只比汇总行的 fingerprint 与第二次读到的那一版：「if "version-unchecked" not in broken and summary.get("fingerprint") != version[:16]:」（`.claude/gate.d/checker-tier-broad-mutation.sh:299`）。
- 驱动 `--summary` 走只读那一支，在取锁之前就返回：「if arguments.list_items or arguments.summary or arguments.check_ignored:」（`research/scripts/broad-mutation.py:2687`）在「library.lock_descriptor = acquire_lock(library.home, arguments.lock_wait)」（`research/scripts/broad-mutation.py:2717`）之前。所以门禁不占库目录的锁，别的会话的 `--sync-tree` 随时能换。
- 走法：第一格对着版本 A 判绿，别的会话在两格之间 `--sync-tree` 到版本 B（B 的清单与结果表已齐、全部有结论，例如更早判完的一版），第二格读到 B 的汇总：fingerprint 与第二次读到的 tree.source 一致，判绿。两格都绿，各说的是不同的版本。新版本的情形是红的（换指针在生成清单之前，`--summary` 读不到清单，退码不是 0 或 8）。所以只有「B 早就判完」才打得中，窗口是两格之间的几秒到几十秒。
- 这个窗口 commit-stages 没写防它的办法：第 2b 步只写了暂存区变了就停下重做（`.claude/skills/crash-test/commit-stages.md:28`：「与 ① 不同（暂存区变了）就停下，①–④ 重做。」），没有写 ④ 前后库目录的版本指针有没有被别的会话换掉。
- 推翻它的现象：两格在同一个进程里共用一次读取（不是，两格各一个子 shell）；或 `--sync-tree` 换指针之前会等 `--summary`（`--summary` 不取锁，不会）。

**G1-c　门禁不比全局键里的工具版本、排除表、限时参数与未盖戳的判法代码（规则没说；E4 暗含；今天没拦）**

- 驱动写全局键全文到这一版的 `global-key.txt`：「write_atomically(library.version_file("global-key.txt"), pruner.global_text.encode())」（`research/scripts/broad-mutation.py:2591`）；键的组成见 `research/scripts/broad-mutation.py:2215` 起的 `global_key`：工具链、Cargo 三类文件、环境变量、nextest / cargo-mutants / llvm-tools 版本、排除表、限时参数、判法版本。
- 门禁一格只比 rustc -Vv 与指纹：版本指纹里只有「judge {JUDGE_RULES_VERSION}」这一个盖戳的判法版本（`research/scripts/broad-mutation.py:732`）。③ 之后换了 cargo-mutants、cargo-nextest、llvm-tools，或改了排除表、`mutation-prune.py` 里没改 `JUDGE_RULES_VERSION` 的判法逻辑，门禁两格照绿（这一道只读库目录）；复用那一层会因登记表里的 `environment=` 与路径变了而不复用，但「不复用」只是再读一遍库目录，读出来仍是绿（`.claude/gate.d/checker-tier-broad-mutation.sh:90` 打 `--tool-versions`，只给复用判定用）。
- C6 原文没有这一句，E4 的「全局键变了就整版重判」是对驱动沿用说的。所以这条算「规则没说」，不算冲突。影响面：③ 与 ④ 之间、或 ④ 在 ③ 之后过了很久才跑时工具换了。
- 推翻它的现象：门禁或驱动 `--summary` 里有一处把这一版的 `global-key.txt` 与此刻重算的键比（`grep -n 'global-key.txt' research/scripts/broad-mutation.py` 只命中写的那一处，见 `:2591`）。

**G1-d　门禁与驱动的出路里写「确是等价变异就写测试留档后从清单排除」，今天没有这个动作可做（句子无实现）**

- 同一句话在两处：门禁一格的出路「确是等价变异就写测试留档后从清单排除」（`.claude/gate.d/checker-tier-broad-mutation.sh:69`）与驱动打红时的「确是等价变异就写测试留档后从清单排除」（`research/scripts/broad-mutation.py:2064`）。
- 现查：驱动与 `.cargo/mutants.toml` 里没有按变异排除的开关，命令与输出：`grep -nE 'exclude_re|exclude_globs|--exclude' .cargo/mutants.toml research/scripts/broad-mutation.py` 在冻结副本里无输出，grep 退码 1。唯一的排除表 `research/scripts/broad-mutation-ignored-exclusions.tsv` 排的是第二段 `#[ignore]` 用例，不是变异。
- 推：要排一条等价变异只能改 `.cargo/mutants.toml`（`.cargo` 在导出范围里），而它进全局键：改一下就让全局键变、整版重判（`research/scripts/broad-mutation.py:2220`）。「排除」这一步的代价是整趟重判，句子没说。
- 推翻它的现象：驱动或配置里另有排除入口（grep 命中）；或判决另定了「等价变异登记」做法并已实现。

**G1-e　用户定案「提交前跑完才能提交」今天没有机械拦截，靠两句话**

- 两句并排：「有一条没结论或没红就判红、不许提交」（`.claude/main-agent.md:67`）；「有一条没结论或没红就判红，红了不提交」（`.claude/rules/implementation-workflow.md:53`）。两句一致。
- 现查（转述自下面的命令输出）：本机 `.git/hooks/pre-commit` 只判东京时间窗口；`.claude/hooks` 与 `.claude/settings.json` 里没有读 `refs/sop/staged-green` 的东西。命令与输出见 G1.3。所以「红了不提交」与「没跑完不提交」是主 agent 的纪律，不是闸。
- 这不算与定案冲突（定案没说要机械拦），记「规则没说」：要不要把 `staged-green` 接成提交闸，是主 agent 的判。
- 推翻它的现象：某个钩子或 settings 在 `git commit` 前读了 `staged-green` 或这一道的全绿标记（G1.3 里的 grep 有命中）。

**G1-f　C6 第二分句：代码改了，基线真绿没有证据；驱动文件头还写着旧的红**

- 驱动文件头写「第二段基线红（今天是 crash_identity.rs 读 .claude/gate.d/stage-inputs.tsv 那条）报成退 6 的拒绝，列红的测试与出路。」（`research/scripts/broad-mutation.py:108`），而测试已改成登记表不在就打 `CRASH_CASE_REGISTRY_ABSENT` 并 return（`crates/singlefs-checker-tier/src/crash_identity.rs:1594` 起）。文件头那半句「今天是…那条」与代码对不上，改文件头归工具实现员（不在五份里）。
- 改法的副作用（推的）：导出树里这条测试在登记表不在时直接 return，等于在被变异的树里不再判「登记的崩溃枚举用例的判法代码」；只有这条测试才抓得到的变异，在广谱变异里会记成没红或零覆盖。这不违反 C6 第二分句字面（「改测试不读导出树之外的文件」），但是一条真实的判别力让步，值得登记。
- 第二段基线真绿：`broad-smoke3` 的 `smoke.sh` 用的是 `--skip-stage-two`，第二段基线（checker 档 33 个目标）没起过；起它是重型，这一腿复核不了。
- 推翻它的现象：一趟不带 `--skip-stage-two` 的 `--changed-since` 在基线步退 0。

### G1.3　命令与原样输出

```
$ cd /tmp/claude-1000/m3-batch5-freeze/tree/.claude; grep -rln 'staged-green' hooks settings.json; echo "rc=$?"
rc=1
$ cat /home/fy5090/code/singlefs/.git/hooks/pre-commit
#!/usr/bin/env bash
# 提交窗口闸：只在东京时间 20:00–02:00 放行。
# 本机时区是 UTC，裸 date 会得出相反结论——这道闸的存在就是因为提醒句拦不住手敲命令。
# 这是个人偏好不是贡献者规则，所以留在 .git/hooks（本地、不入库），不进 gate.sh。
h=$(TZ=Asia/Tokyo date +%-H)
if (( h >= 20 || h < 2 )); then exit 0; fi
echo "✗ 提交窗口外：东京时间 $(TZ=Asia/Tokyo date '+%H:%M %A')，窗口是 20:00–02:00"
echo "  → 怎么办：把改动留在工作区，进窗口后再提交。"
echo "     确需绕过（例如用户明确要求）：git commit --no-verify"
exit 1
$ cd /tmp/claude-1000/m3-batch5-freeze/tree; grep -nE 'exclude_re|exclude_globs|--exclude' .cargo/mutants.toml research/scripts/broad-mutation.py; echo "rc=$?"
rc=1
```

（`grep -rln` 在 `hooks`、`settings.json` 里零命中；注意前面一次不带 `settings.json` 的 grep 命中的是 `skills/gate/staged-gate.md`，那是文档，不是闸。）

### G1.4　小结与反推

- 对 C6 四句 + E4 + 用户定案：门禁两格判的是「这一次暂存树那一版每一条有结论、没有没红」，而且比 C6 字面更严（整版 total 条，不止改到的）。逐句对：①②③ 一致，④ 句子一致、真绿复核不了，⑤ 门禁一侧只罩住被盖了戳的判法版本与 rustc，⑥ 门禁一致、提交没有闸。
- body 点名的两个例子：「库目录换到暂存树之后又被别的会话换走」→ 这一腿推得出一条能走通的路（G1-b，窗口小、要求 B 早判完）；「沿用的结论来自判法版本不同的一版」→ 盖了戳的判法版本进了版本指纹，门禁一格会红：「driver.version_key(home, staged_source_fingerprint, current_toolchain) != fingerprint」（`.claude/gate.d/checker-tier-broad-mutation.sh:246`）；没盖戳的改动罩不住（G1-c）。
- 不该红而红：这一腿没找到。读过的红分支：版本指针临时件在（`:125` 前后）、导出范围不同、逐文件不同、工具链不同、汇总行指纹不同、计数自相矛盾、total 为 0；每一支都对应 C6 或定案里一句。`flipped` 只报不判，与 C5「翻面的逐条列出」相符（`.claude/gate.d/checker-tier-broad-mutation.sh:25`：「flipped（里程碑全量翻面的条数）只报不判。」）。没找到只抽了一次样（读码一遍），不支撑「没有」。
- 反推（假设「门禁判的与 C6 一致」是错的，本该看到什么）：该红的输入不红。这一腿在码里找到两条路径（G1-a 在标记层复现，G1-b 推的）；没找到「判红分支缺失」的路径。所以结论写成「判红条件与 C6 逐句一致，外加两条没被罩住的路径与一条规则空白」，不写成「一致」。

## G3　五份文件里广谱那一批的句子，在删掉 crash-verifier 与 gate-triage 之后还对不对

被判的五份在冻结副本里：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/skills/crash-test/commit-stages.md`。
判据（从今天的实现推）：步号与文件名在不在、谁跑（今天只有主 agent）、带什么前缀（`SINGLEFS_HEAVY_TESTS=commit`）、驱动的参数与退出码在不在、数目（六道 / 五道）、红了怎么办、文件之间一致不一致。

### G3.1　数得出来的几件事（原样命令与输出）

```
$ cd /tmp/claude-1000/m3-batch5-freeze/tree/.claude; for f in main-agent.md agent-common.md rules/verification.md rules/implementation-workflow.md skills/crash-test/commit-stages.md; do printf '%s\t%s\n' "$f" "$(grep -cE 'crash-verifier|gate-triage|崩溃验证员|门禁分诊' $f)"; done
main-agent.md	0
agent-common.md	0
rules/verification.md	0
rules/implementation-workflow.md	0
skills/crash-test/commit-stages.md	0
$ grep -nE '五道|六道' main-agent.md agent-common.md rules/verification.md rules/implementation-workflow.md skills/crash-test/commit-stages.md | cut -d: -f1,2 | tr '\n' ' '
main-agent.md:53 main-agent.md:67 agent-common.md:88 skills/crash-test/commit-stages.md:1 skills/crash-test/commit-stages.md:3 skills/crash-test/commit-stages.md:22
$ grep -rn 'AGENT_KINDS' hooks | cut -c1-120
hooks/lib_heavy_tests.py:264:  子 agent 的放行表里没有这一类，只有主 agent 带前缀能跑（heavy-test-guard.sh 的 AGENT_KINDS）。
```

- 五份里已经没有 crash-verifier、gate-triage 的名字；谁跑一律是主 agent，五份一致。
- 「六道」出现在 `.claude/main-agent.md:53`、`:67`，`.claude/agent-common.md:88`，`.claude/skills/crash-test/commit-stages.md:1`、`:3`、`:22`，数目一致。「五道」只剩 commit-stages:22 一处点名，和 main-agent:67 的「其余五道」（六道去掉 `checker-tier-layer0-replay`，数得对）。
- 判决要求现查的 `heavy-test-guard.sh` 的 `AGENT_KINDS`：整个 `.claude/hooks` 里只剩 lib_heavy_tests.py:264 一句注释还点它，`heavy-test-guard.sh` 里已经没有这个名字。主 agent 那一支的判法归攻方的 G4；这里只记：注释里点的名字今天不存在（见 G3-9）。

### G3.2　逐句表（只列广谱那一批写下的句子）

判定三种：成立 / 不成立 / 规则没说（句子没写、但读的人会需要）。

| 文件:行 | 句子（整行抄，行内有「」括注的只抄括注之外的一段） | 判定与依据 |
|---|---|---|
| `.claude/main-agent.md:53` | 「checker-tier-broad-mutation 前面起的广谱变异驱动给每条命令的内存上限」（`.claude/main-agent.md:53`） | 成立：与「广谱变异驱动（第 2b 步）的两个上限」（`.claude/skills/crash-test/commit-stages.md:12`）同一件事，参数 `--memory-cap`、`--threads` 在驱动里都有（`research/scripts/broad-mutation.py:2512`、`:2511`）。小毛病见 G3-8 |
| `.claude/main-agent.md:62` | 「Verus 形式化验证、广谱变异提交门禁（命令带」（`.claude/main-agent.md:62`） | 成立：派发表两行（`:62`、`:64`）点名的六样与 commit-stages 一致 |
| `.claude/main-agent.md:67` | 「主 agent 照 .claude/skills/crash-test/commit-stages.md 跑这六道」（`.claude/main-agent.md:67`） | 成立：六个名字都在 commit-stages:22 的名单里 |
| `.claude/main-agent.md:67` | 「checker-tier-broad-mutation 排在 checker-tier-crates-mutation-replay 之后」（`.claude/main-agent.md:67`） | 成立：commit-stages 第 2b 步「等 checker-tier-crates-mutation-replay 跑完再起」（`.claude/skills/crash-test/commit-stages.md:23`） |
| `.claude/main-agent.md:67` | 「有一条没结论或没红就判红、不许提交」（`.claude/main-agent.md:67`） | 门禁判红成立（G1.1 ③）；「不许提交」是句子，没有闸（G1-e） |
| `.claude/main-agent.md:67` | 「其余五道同样带前缀、同样按各自输入的哈希只跑变了的、判绿写全绿标记」（`.claude/main-agent.md:67`） | 对广谱这一道成立在整轮里（`stage-must-run.sh` 比暂存树）；commit-stages 第 1 步给它的直接复用捷径不成立（G3-1） |
| `.claude/main-agent.md:67` | 「整轮里不另外直接调这六道」（`.claude/main-agent.md:67`） | 成立；注意广谱那一道在整轮里「真跑」只重读库目录，不起驱动（`.claude/gate.d/checker-tier-broad-mutation.sh:26` 一带的文件头说明），库目录不是这棵暂存树就整轮红 |
| `.claude/agent-common.md:52` | 「起 cargo 或改库目录的每一种用法；只有」（`.claude/agent-common.md:52`） | 比钩子窄一档，见 G3-4 |
| `.claude/agent-common.md:52` | 「重型测试只由主 agent 在提交时或用户要求时跑」（`.claude/agent-common.md:52`） | 成立：今天没有任何子 agent 类型带广谱变异的放行（`AGENT_KINDS` 已不在） |
| `.claude/agent-common.md:54` | 「与门禁 checker-tier-crates-mutation-replay 在里面逐条套了」（`.claude/agent-common.md:54`） | 规则没说：驱动自己经包装（commit-stages:26 写了），这一份的清单没列，见 G3-5 |
| `.claude/agent-common.md:88` | 「提交时的六道重阶段由它照」（`.claude/agent-common.md:88`） | 成立 |
| `.claude/rules/verification.md:13` | 「广谱变异（驱动 research/scripts/broad-mutation.py；提交门禁 checker-tier-broad-mutation 判这一次暂存树那一版每一条变异都有结论、没有没红）」（`.claude/rules/verification.md:13`） | 成立：与 gate-verdict 判整版 total 条一致 |
| `.claude/rules/verification.md:37` | 「checker-tier-broad-mutation 与 checker-tier-research-build-and-replay 的 experiment-replay 格照各自的复用判定跑」（`.claude/rules/verification.md:37`） | 成立：登记在 `.claude/gate.d/stage-inputs.tsv:33`；直接捷径的毛病见 G3-1 |
| `.claude/rules/verification.md:99` | 「广谱变异：这一次暂存树那一版里改到的变异每一条都有这一版的结论、沿用的结论都在、没有一条没红」（`.claude/rules/verification.md:99`） | 成立，字面窄于判法：这一格抄的是 C6 的话，门禁判的是整版每一条（G1.1 ①） |
| `.claude/rules/verification.md:99` | 「gate-verdict 格判驱动 --summary 的 BROAD_MUTATION_GATE 行；驱动由主 agent 提交时先起」（`.claude/rules/verification.md:99`） | 成立：格名、行名都在（`.claude/gate.d/checker-tier-broad-mutation.sh:68`、`research/scripts/broad-mutation.py:2048` 打 BROAD_MUTATION_GATE） |
| `.claude/rules/verification.md:107` | 「不算）与门禁 checker-tier-broad-mutation」（`.claude/rules/verification.md:107`） | 与 agent-common:52 同毛病，见 G3-4 |
| `.claude/rules/implementation-workflow.md:32` | 「checker-tier-formal-verification、checker-tier-broad-mutation 都登记着」（`.claude/rules/implementation-workflow.md:32`） | 成立：`.claude/gate.d/stage-inputs.tsv:33` |
| `.claude/rules/implementation-workflow.md:53` | 「先起广谱变异驱动把库目录换到这一次的暂存树、--changed-since HEAD 只现判改到的」（`.claude/rules/implementation-workflow.md:53`） | 成立：与第 2b 步 ②③ 一致 |
| `.claude/rules/implementation-workflow.md:53` | 「有一条没结论或没红就判红，红了不提交」（`.claude/rules/implementation-workflow.md:53`） | 同 main-agent:67：门禁成立，闸没有（G1-e） |
| `.claude/rules/implementation-workflow.md:57` | 「不算）与门禁 checker-tier-broad-mutation」（`.claude/rules/implementation-workflow.md:57`） | 同 G3-4 |
| `.claude/skills/crash-test/commit-stages.md:3` | 「子 agent 不跑这几道」（`.claude/skills/crash-test/commit-stages.md:3`） | 成立 |
| `.claude/skills/crash-test/commit-stages.md:19` | 「checker-tier-broad-mutation 判的是暂存区（git 索引）上的源码，不是工作区」（`.claude/skills/crash-test/commit-stages.md:19`）：开跑前先 stage-marker-check、退 0 记复用，不跑 git diff --quiet | **不成立**，G3-1 |
| `.claude/skills/crash-test/commit-stages.md:22` | 「五道的命令都带输入给的那个前缀」（`.claude/skills/crash-test/commit-stages.md:22`） | **不成立**（数目），G3-2 |
| `.claude/skills/crash-test/commit-stages.md:23` | 「2b. checker-tier-broad-mutation 等 checker-tier-crates-mutation-replay 跑完再起（两样都逐条改坏再编译，争 CPU），在主工作区按次序做四步：」（`.claude/skills/crash-test/commit-stages.md:23`） | 成立 |
| `.claude/skills/crash-test/commit-stages.md:24` | 第 2b 步 ①：`git write-tree`、`git commit-tree <树号> -p HEAD -m …`，转述 | 成立：`--sync-tree --commit` 对 `<提交>^{commit}` 解析，不要求提交在任何引用上（`research/scripts/broad-mutation.py` 的 `sync_tree` 起头一段）；只写一个提交对象，与 `:36` 的写范围句一致 |
| `.claude/skills/crash-test/commit-stages.md:26` | 「--changed-since HEAD --threads <输入给的线程数> --memory-cap <输入给的上限>」（`.claude/skills/crash-test/commit-stages.md:26`），一趟以小时计、「照第 2 步后台跑」 | 三个参数都在驱动里（`research/scripts/broad-mutation.py:2494`、`:2511`、`:2512`）；「照第 2 步后台跑」**不成立**（G3-3） |
| `.claude/skills/crash-test/commit-stages.md:27` | 「不起 cargo，外面不包内存包装」（`.claude/skills/crash-test/commit-stages.md:27`） | 基本成立，字面有一处例外，G3-6 |
| `.claude/skills/crash-test/commit-stages.md:28` | 「退 4（库目录的锁被别的一趟占着）抄」（`.claude/skills/crash-test/commit-stages.md:28`） | 成立：驱动文件头「4 锁被占着；5 树脏了恢复不了」（`research/scripts/broad-mutation.py:119`） |
| `.claude/skills/crash-test/commit-stages.md:28` | 「与 ① 不同（暂存区变了）就停下，①–④ 重做。」（`.claude/skills/crash-test/commit-stages.md:28`） | 成立；没写 ④ 期间别的会话换库目录怎么办（G1-b，规则没说） |

### G3.3　不成立与冲突的句子，两边并排

引文按句抄（commit-stages 第 1 步与第 2 步各是一整行、几千字，整行放不进来；句子之外的部分不省略任何限定词）。

**G3-1　commit-stages 第 1 步给广谱这一道的「标记复用、不跑 git diff --quiet」与标记的算法冲突（打中，G1-a 同一件事）**

| 这边（句子） | 那边（今天的实现） |
|---|---|
| 「checker-tier-broad-mutation 判的是暂存区（git 索引）上的源码，不是工作区」（`.claude/skills/crash-test/commit-stages.md:19`）；「第 2b 步的驱动也不起」（`.claude/skills/crash-test/commit-stages.md:19`）；对别的四道写的是「退 0 才说明工作区与暂存区在这一道的输入上相同」（`.claude/skills/crash-test/commit-stages.md:19`） | 标记指纹的文件清单：「["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],」（`research/scripts/admission.py:477`）；内容读磁盘：「digest_hex = sha256_of_file(os.path.join(os.fsencode(root), name))」（`research/scripts/admission.py:693`）；一次性仓里索引与工作区分叉后 `stage-marker-check` 三次都退 0（G1-a 的原样输出） |

- 为什么别的四道要先 `git diff --quiet`：标记按工作区算，只有工作区与暂存区在输入上相同，标记才等于「暂存的那一批判过」。广谱这一道判的是暂存区，更需要这一条，不是不需要。
- 复现程度：标记这一层复现了（G1-a）；广谱这一道真跑出「该红不红」没有跑，整道是重型。
- 推翻它的现象：`stage-marker-check` 对工作区与索引分叉的一次性仓退 1；或这一道的登记路径从来碰不到工作区与暂存区会分叉的文件（不是：`crates/` 在里面，`.claude/gate.d/stage-inputs.tsv:33`）。

**G3-2　commit-stages 第 2 步「五道的命令都带…前缀」的数目对不上同一步的「这六道」（打中，小）**

| 这边 | 那边 |
|---|---|
| 「五道的命令都带输入给的那个前缀」（`.claude/skills/crash-test/commit-stages.md:22`） | 同一步开头：「只在提交时（或用户要求时）跑这六道」（`.claude/skills/crash-test/commit-stages.md:22`）；广谱那一道的 ④ 也是同一种写法：「SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh」（`.claude/skills/crash-test/commit-stages.md:27`），③ 的驱动同样带前缀（`.claude/skills/crash-test/commit-stages.md:26`） |

- 读法：「五」是 crash-verifier 时代「五道」留下的数；加了广谱变异之后六道都带这个前缀，驱动的两条命令也带。
- 推翻它的现象：这句的「五道」指的是别的一组（如「除广谱外的五道」）并在句里写明。

**G3-3　commit-stages 第 2b 步 ③「照第 2 步后台跑」与共用约束对「跑得过后台时限的长活」的做法冲突（打中，推的）**

| 这边 | 那边 |
|---|---|
| 「--changed-since HEAD --threads <输入给的线程数> --memory-cap <输入给的上限>」「一趟以小时计，照第 2 步后台跑、退出码写进」（`.claude/skills/crash-test/commit-stages.md:26`），第 2 步的后台写法是 `{ …; echo "exit=$?"; } > <草稿目录>/<阶段>.log` 配 run_in_background | 「跑得过后台时限的长活经」detached-run.sh 起（`.claude/agent-common.md:64`，原句在 `bash research/scripts/detached-run.sh start --label <名> -- <命令>` 之前）；同一份 commit-stages 对崩溃放量全量（同样以小时计）写的就是 detached-run：「detached-run.sh start --label crash-amplification-full」（`.claude/skills/crash-test/commit-stages.md:22`）；判决给的量级：「推的耗时 10.6–82.0 小时（只算第一段）」（`research/prompts/tooling-broad-mutation-prune-r3-main-verification.md:29`） |

- 推：③ 的挂钟按判决推的是十几到几十小时，超过 Bash 工具 run_in_background 的时限（我这一腿看到的是自己的 Bash 工具说明：默认 30 分钟、最大 2 小时；主 agent 那一份没核）。到限被停，驱动退 143（`research/scripts/broad-mutation.py:120` 的退出码段）；commit-stages:28 列的退码是 3、4、6、7，没有 143，也没说停了之后怎么续（驱动本身能续：已有定论的条不重跑）。
- 反方向：重型闸对 detached-run 起广谱驱动的写法有样本（`.claude/hooks/heavy-test-guard.sh:1015` 起的几例，到 `:1021`），所以改成 detached-run 起是可行的。
- 推翻它的现象：主 agent 的后台命令没有时限；或一次真实的 ③ 在时限内跑完（判决给的是推的耗时，没有真跑量；`broad-smoke3` 里点名 2 条的一趟 29 秒，不是代表性的量）。

**G3-4　三份「重型清单」里的驱动用法描述比钩子窄，且与自己的定义不合（不成立，小，偏严）**

| 这边（三处同一句式） | 那边 |
|---|---|
| 「起 cargo 或改库目录的每一种用法；只有」`--list-items`、`--summary` 不算（`.claude/agent-common.md:52`；同句在 `.claude/rules/verification.md:107`、`.claude/rules/implementation-workflow.md:57`） | 钩子的只读集合是四个：「BROAD_MUTATION_QUERY_ARGUMENTS = frozenset({"--list-items", "--summary", "--help", "-h"})」（`.claude/hooks/lib_heavy_tests.py:397`）；仓内位置上的驱动 `--selftest` 登记为 light（`.claude/hooks/selftest-registry.tsv:27`）；门禁带 `--list`、`--tool-versions` 不算重型（`.claude/hooks/lib_heavy_tests.py:275`：「提交门禁那一道」一句）；`--check-ignored` 只读、不起 cargo（`research/scripts/broad-mutation.py:1426` 的 `check_ignored_command`），钩子照重型判 |

- 读法：句子说「只有这两个不算」，钩子还放了 `--help`、`-h`、`--selftest`；定义说「起 cargo 或改库目录」，而 `--check-ignored` 不起 cargo 也被算重型。按字面读，子 agent 不会被多放；主 agent 读句子会以为 `--help` 要带前缀。方向偏严，不会放过重活。
- 推翻它的现象：钩子里 `--help` 与 `-h` 不在只读集合里（上面的行号处是原样）。

**G3-5　共用约束「经内存包装」那一条的清单没有驱动（规则没说，小）**

- 「与门禁 checker-tier-crates-mutation-replay 在里面逐条套了」（`.claude/agent-common.md:54`）这一串清单列了 mutate.sh、replay.sh、几道门禁阶段，没有广谱驱动；commit-stages 写了「驱动每条命令自己经内存包装，外面不再包一层」（原句在 `.claude/skills/crash-test/commit-stages.md:26`）。今天只有主 agent 跑驱动，commit-stages 已经写了，影响小。
- 推翻它的现象：agent-common:54 里别处已列驱动（`grep -n 'broad-mutation' .claude/agent-common.md` 只命中 `:52`）。

**G3-6　commit-stages 第 2b 步 ④「不起 cargo」的字面例外（小）**

- 「不起 cargo，外面不包内存包装」（`.claude/skills/crash-test/commit-stages.md:27`）。门禁整道真跑时，复用判定要取环境，登记行 `environment=` 指到 `--tool-versions`，那一支会起 `cargo nextest --version` 与 `cargo mutants --version`：「cargo nextest --version 2>&1 | head -n 1」（`.claude/gate.d/checker-tier-broad-mutation.sh:90`）。只打版本，不编不跑，所以「不起 cargo」应读成不起 cargo 编译或跑测试；字面略大。
- 推翻它的现象：`--tool-versions` 不在复用判定的取环境路径上（`.claude/gate.d/stage-inputs.tsv:33` 的 `environment=` 列写着它）。

**G3-7　「输入给的…」指代落空（小）**

- 「输入给的那个前缀」（`.claude/skills/crash-test/commit-stages.md:22`）、第 2b 步 ③ 的 `<输入给的线程数>`、`<输入给的上限>`（`.claude/skills/crash-test/commit-stages.md:26`）这些话是 crash-verifier 定义的「输入（主 agent 必须给）」留下的口吻。今天没有派发，「输入」指的是第「跑之前定下来」一节（`.claude/skills/crash-test/commit-stages.md:12`）。读得通，指代已落空；同一行还留着「你原样抄进报告」（`.claude/skills/crash-test/commit-stages.md:22`）。这两处不属广谱那一批，只记。

**G3-8　main-agent 第 53 行的位置（小）**

- 「checker-tier-broad-mutation 前面起的广谱变异驱动给每条命令的内存上限」（`.claude/main-agent.md:53`）这一段在 `.claude/main-agent.md@派发提示怎么写` 这一节里，句子今天是主 agent 给自己定上限，不是写给被派的 agent。内容与 commit-stages「跑之前定下来」一致，不影响做法。

**G3-9　五份之外顺带看到的两处旧说法**

- `.claude/hooks/lib_heavy_tests.py:264`：「子 agent 的放行表里没有这一类，只有主 agent 带前缀能跑（heavy-test-guard.sh 的 AGENT_KINDS）。」`heavy-test-guard.sh` 里已没有 `AGENT_KINDS`（G3.1 的 grep）。
- `research/scripts/broad-mutation.py:108`：「第二段基线红（今天是 crash_identity.rs 读 .claude/gate.d/stage-inputs.tsv 那条）报成退 6 的拒绝，列红的测试与出路。」测试已改（G1-f）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| G1 ① 每条有结论 | 一致（比 C6 字面更严） | 判整版 total 条，超时、第一段、没跑都算没结论 |
| G1 ② 沿用的结论都在 | 一致（「在」）；「作不作数」规则没说 | 门禁只看沿用行在不在，不回核结论表与全局键 |
| G1 ③ 有一条没红就红 | 一致 | missed 大于 0 判红，退 8，计数与明细互核 |
| G1 ④ 第二段基线先修绿 | 句子一致；真绿复核不了 | 测试改成登记表不在就 return；基线真跑是重型；驱动文件头还写旧的红 |
| G1 ⑤ E4 全局键与无效条 | 沿用一侧一致；门禁一侧只罩判法版本与 rustc | 工具版本、排除表、限时与未盖戳的判法代码门禁不比（G1-c） |
| G1 ⑥ 跑完才能提交 | 门禁一致；提交没有闸 | 只有 main-agent:67、implementation-workflow:53 两句，pre-commit 只判时间窗口（G1-e） |
| G1-a 复用标记对工作区算 | 打中（标记层已复现；整道没跑） | 索引与工作区分叉时 stage-marker-check 照退 0，commit-stages 第 1 步还免了 git diff --quiet |
| G1-b 两格各读一次 tree.source | 打中（推的，没复现） | 两格之间别的会话换到一版已判完的树，两格各判各的版本 |
| G1-d 等价变异「从清单排除」 | 句子无实现 | 驱动与配置里没有按变异排除的开关，改 `.cargo` 等于整版重判 |
| G3 `.claude/main-agent.md` | 成立 | 六道、次序、前缀、驱动上限都对得上；位置小毛病（G3-8）；「不许提交」没有闸 |
| G3 `.claude/agent-common.md` | 成立，两处小毛病 | 重型描述比钩子窄（G3-4）；内存包装清单没列驱动（G3-5） |
| G3 `.claude/rules/verification.md` | 成立，一处小毛病 | 第 107 行同 G3-4；第 99 行字面窄于判法 |
| G3 `.claude/rules/implementation-workflow.md` | 成立，一处小毛病 | 第 57 行同 G3-4；第 53 行的「红了不提交」同 G1-e |
| G3 `.claude/skills/crash-test/commit-stages.md` | 三句不成立 | 第 19 行标记捷径（G3-1）；第 22 行「五道」（G3-2）；第 26 行「照第 2 步后台跑」（G3-3） |
| G3 文件之间一致 | 一致，三处例外 | 六道一致、没有 crash-verifier 与 gate-triage 的名字；例外是 G3-1、G3-2、G3-4 |
| G3 五份之外 | 两处旧说法 | `lib_heavy_tests.py:264` 的 AGENT_KINDS；驱动文件头 `:108` 的旧红 |

按判据（一句话与今天的做法不符 ⇒ 打中，改那一句）：commit-stages 三句打中（G3-1、G3-2、G3-3），G3-4 打中（小、偏严）；G1-a、G1-b 让门禁该红不红，打中「改门禁或驱动」那一条（G1-a 先改 commit-stages 一句，G1-b 要改门禁两格的读法或让驱动 `--summary` 回报它读的版本）。G1-c、G1-d、G1-e 是规则空白与句子无实现，记一笔。
本腿对 G1 的「不该红而红」只读码一遍，没找到；这只抽了一次样，不支撑「没有」。

## 没做什么

- 没跑任何重型：驱动的真跑、门禁整道（含样本模式的整道）、第二段基线、`stage-selftest.sh` 喂样本都没起；G1-b、G3-3 是读码与规则推的，G1-a 只在标记这一层用冻结副本的 `admission.py` 在一次性仓里复现，没有对真库目录与真暂存区复现。
- 没替主 agent 采纳或出判决；「打中」按跑前写死的判据字面标，是否改、怎么改归主 agent。
- 不判别的格：G2（次序与复用的 24 小时上限、复用罩没罩全输入）、G4（`classify` 的逐类判定、主 agent 不带前缀会不会被拒）、G5（12 份样本与 8 个弄坏开关）没判；只在上面顺手记了 `AGENT_KINDS` 已不在与重型描述比钩子窄两件读到的事。
- 没核主 agent 那一份 Bash 工具的后台时限（G3-3 引的是这一腿自己的 Bash 工具说明）。
- 没核 `broad-smoke3` 之外的真库冒烟产物，也没读背景材料附录里 `tooling.md` 的抄录（与这两格无关）。
- 没改任何仓内文件；草稿 `/tmp/claude-1000/m3-broad-gate-gov-r1-sonnet/` 下只留几份草稿串；一次性仓 `marker-repro/` 与对照用的 `orig/` 已在交回前删掉。
