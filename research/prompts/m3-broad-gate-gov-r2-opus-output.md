# m3-broad-gate-gov-r2 攻方腿（Opus）报告（2026-10-03）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 H1 H2 H3 H4 H5 A1 A3 A4 A5 A6 A7 A8 A9 A10 C6 E4 -->

攻击面：正文 H1（十条改法本身）、H2（样本与自证够不够得着）、H5（几个会话共写库目录），兼攻 H3（丙的出路）。读与拷都用冻结副本 `/tmp/claude-1000/m3-gov-r2-freeze/tree/`（开工时对 `tree-sha256.txt` 跑 `sha256sum -c --quiet` 退 0）；下文引的被判文件在主仓与冻结副本里逐字节相同（`cmp` 过：门禁、驱动、admission.py、mutation-prune.py、detached-run.sh、commit-stages.md、main-agent.md、stage-inputs.tsv、lib_heavy_tests.py），行号两边通用。

## 复跑

模型目录 `research/prompts/m3-broad-gate-gov-r2-opus-model/`。在任意目录：

```
nice -n 19 bash research/prompts/m3-broad-gate-gov-r2-opus-model/rerun.sh /tmp/claude-1000/m3-gov-r2-freeze/tree <草稿目录>
```

它只起样本档（每份拷进临时目录、跑 setup.sh、cd 进样本根、根参数写同一个目录、带 `--force`，与 `stage-selftest.sh` 同一种起法）、载入驱动与 admission.py 的纯函数、只读真库，不起驱动真跑、不起整道门禁，本机约 3 分钟。产出覆盖写进 `outputs/`。驱动拷贝的 `--selftest` 子 agent 被闸拒，写成 `driver-selftest-mutants.sh` 交主 agent（见「要主 agent 跑的命令」）。

| 文件 | sha256 前 16 位 |
|---|---|
| `rerun.sh` | 1cea11835ca7bc66 |
| `run-samples.sh` | b952908784aa7bba |
| `exp1-marker-check.sh` | 9cdba0f695d8d14b |
| `h2-mutants.py` | f1c0da3eddfa1642 |
| `h1-7-combos.py` | 11674e034d491b08 |
| `h1-5-env-gap.py` | 595db402512e50b9 |
| `h1-5-real-key.py` | 793a41c885983f16 |
| `exp2-probe.py` | 1be14423323d66b3 |
| `driver-selftest-mutants.sh`（没跑，交主 agent） | d1c589ca81eeba0b |
| `outputs/samples-base.txt` | a880ae0fc1dd9e62 |
| `outputs/switches.txt` | 560d891d7d619cd4 |
| `outputs/h2-mutants.list` | a86867357063a1c4 |
| `outputs/h2-sweep.txt` | f828932da5e5694b |
| `outputs/exp1-marker-check.txt` | 454153abc05a0cce |
| `outputs/h1-7-combos-400.txt` | 2edd14b645619558 |
| `outputs/h1-7-combos-5000.txt` | 05f315a324bb2b19 |
| `outputs/h1-5-env-gap.txt` | 47046310bba61152 |
| `outputs/h1-5-real-key.txt` | 6ed50c12586391c0 |
| `outputs/exp2-probe.txt` | aad965e7ea7e5315 |

全部数是在冻结副本的拷贝或抽出来的纯函数上量的，副本上的数不算入库装置上的数。

## 各格判定一览

| 格 | 判 | 依据（详见各节） |
|---|---|---|
| H1-3a 写标记修了，查标记那一侧没修 | **打中**（样本上量过） | 合法标记 A 写下之后，暂存 B、工作区写回 A：`stage-marker-check` 退 0「ok」，同一现场门禁第一格红。commit-stages 第 19 行照这个退 0 记「复用」、第 2b 步不起；④ 自己的复用判定走同一条（推的）。整轮 `gate-staged.sh` 在临时 worktree 里按暂存树算，兜得住（推的） |
| H1-3b 与别的五道「同规矩」只做了一半 | **打中**（推的，带一个量过的旁证） | 别的五道输入路径脏就不跑；这一道照跑、照出判定、只不写标记。驱动、mutation-prune.py、排除表都读工作区那一份（排除表读工作区量过）；驱动或剪枝规则里没暂存的改动（不动 `JUDGE_RULES_VERSION`）判出来的结论，④ 与整轮都认 |
| H1-4 钉住 tree.source | 没打中 | 版本目录里的 manifest、toolchain、global-key.txt 由指纹定，结果表只追加；找到的只有保守的误红 |
| H1-5 门禁与驱动现算的全局键是否同一份 | 没打中（真库上量过）；旁支打中一处，射程外 | 照门禁算法在真库那一版上现算，16 行里 15 行逐字节相同，唯一不同是排除表那一行（排除表 10-01 之后确实改过）。旁支：全局键对 `CARGO_TARGET_*_RUSTFLAGS`、`RUSTC_WRAPPER`、`CARGO_BUILD_TARGET`、`~/.cargo/config.toml` 不变，仓里 admission.py 的构建环境定义对它们变（模型量过） |
| H1-6 `--changed-since` 重选超时、撞顶 | 机械上没打中 | 非确定的超时走得通；确定性的见 H3 |
| H1-7 两行互核在各种结局组合下 | 没打中（量过） | 5400 种随机组合（含沿用行、驱动错、只过第一段、没跑），门禁判定与「每条有结论且没红」全对上，0 次误报对不上 |
| H2-a 共用函数写错时样本翻不翻面 | **打中**（样本上量过；驱动自证那一半推的） | 16 种写错版里 11 种让 26 份样本一份都不翻；其中 6 种我在驱动 `--selftest` 里找不到会红的格（推的，交主 agent 跑） |
| H2-b 20 个开关是否只翻点名那几份 | 没打中（量过） | 逐个复现，翻的与门禁文件头写的逐项相同；互相兜住的两处文件头已写明 |
| H2-c 这一批新加的判红分支有没有样本 | 低价值，记一笔 | `global-key.txt` 与 tree.source 的 global_key 不是同一次写的那一支没有样本；今天的驱动写不出这种现场 |
| H3-a 确定性的超时、撞顶 | **打中，交用户**（推的，按代码） | 重跑任何次都同一结局；超时、撞顶不进结论表，之后每一次提交的 `--changed-since` 都重选它、照旧红；「交用户定」之后门禁没有认这个决定的入口 |
| H3-b ② 与 ③ 环境不同 | **打中，条件式**（推的，带模型旁证） | ② 在主会话的 shell 里记全局键，③ 经 detached-run 只带五个变量；主 shell 设了 `RUSTFLAGS` 或 `CARGO_PROFILE_*` 时 ③ 每次退 7，出路给的重做 ② 还在同一个 shell 里，绕不出去 |
| H3-c 规模 | 观测 | 真库此刻那一版没记全局键、排除表又改过，下一次提交 ③ 是 5091 条全量现判；09-01 起碰 crates 的 23 个提交里 5 个碰了 Cargo 文件（每个都换全局键、整版重判） |
| H5 别的会话在 ② 与 ④ 之间 | 部分打中（推的） | 钉住拦 ④ 期间的换树；退 7 拦 ② 之后改排除表、工具、限时、编译变量；第一格拦 ③ 之后换树。拦不住的：② 与 ③ 之间换树（③ 白跑几小时）；别的会话在共用工作区里没暂存的驱动、剪枝规则改动（同 H1-3b） |

## H1　修补本身

### H1-3a　第 3 条只修了写标记那一侧，查标记那一侧仍按工作区算指纹

许可它的几句：
- `.claude/skills/crash-test/commit-stages.md:19`：「checker-tier-broad-mutation 判的是暂存区（git 索引）上的源码，不是工作区：开跑前同样先 stage-marker-check，退 0 记」——后面紧跟「复用」与「第 2b 步的驱动也不起」，查标记之前不核工作区等于暂存区。
- `research/scripts/admission.py:2211`：「def stage_fingerprint(root, stage):」，它经 `research/scripts/admission.py:477`：「["git", "-C", root, "ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", *input_paths],」列文件、按磁盘内容算指纹（第一轮判决第一节已核）。
- ④ 在主工作区跑时自己的复用判定也先看标记：`.claude/gate.d/checker-tier-broad-mutation.sh:511`：「if ((reuse_exit == 77)); then」，它经 stage-must-run.sh 转到 `research/scripts/admission.py:1135`：「valid, lines = check_stage_marker(root, stage, fingerprint)」，作数就 `research/scripts/admission.py:1137`：「return EXIT_GATE_MAY_SKIP」。
- 第 3 条的改法只管写：`.claude/gate.d/checker-tier-broad-mutation.sh:484`：「write_marker_if_clean() {」。

历史（样本 marker-clean-written 上量过，`exp1-marker-check.sh`）：
1. 暂存区与工作区都是 A，门禁判绿、写标记 A（干净，第 3 条放行，合法）：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp1-marker-check.txt:9`：「stage(A) exit=0」。
2. 把 `crates/sample/src/lib.rs` 改成 B 并 `git add`，再把工作区写回 A（真仓里的对应动作：`git restore --source=HEAD <文件>` 只还原工作区，或编辑器撤销）：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp1-marker-check.txt:13`：「暂存区 lib.rs 第 2 行： left - right；工作区 lib.rs 第 2 行： left + right」。
3. commit-stages 第 19 行那一查：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp1-marker-check.txt:16`：「marker-check exit=0」，上一行是「ok …singlefs-stage-green.checker-tier-broad-mutation.sh.a8889c46…」。
4. 同一现场让门禁判：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp1-marker-check.txt:21`：「源码与库目录那一版不同（1 个文件）：内容不同 crates/sample/src/lib.rs」，`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp1-marker-check.txt:28`：「stage(B staged) exit=1」。

所以照第 19 行，第 1 步记「复用」、第 2b 步不起，B 一次没被判过。④ 若照样在主工作区起，阶段在格之前走复用判定，按同一个标记退 77（推的：样本模式不走这一段，`real_run` 在样本里是 0）。兜底：整轮 `gate-staged.sh` 在临时 worktree 里起这一道，那里的指纹按暂存树算，标记 A 对不上，往下按 staged-green 判，导出范围变了就真跑，第一格红在库目录还是 A 那一版（推的，没跑整轮；库目录要是已被谁换到 B 那一版则另说）。

四句：
- 分不分辨改法：第一轮判决第 3 行的改法（写之前核）在这格上照中；要修得改查的一侧：(i) 第 19 行给这一道的 `stage-marker-check` 前面加与别的四道同样的 `git diff --quiet` 与未跟踪文件两核；(ii) 这一道的复用指纹按索引算（`git ls-files -s` 取 blob，不读磁盘）；(iii) 去掉这一道的标记复用，只靠整轮。三样都只在推理上修这一格，没实现、没跑。
- 系统当时看不看得到：看得到，`git diff --name-only` 在第 3 步那一刻列出 lib.rs。
- 满足判据的哪一句：正文第五节第二句「五份文件里一句话与今天的做法不符」（第 19 行说退 0 就是复用，而那一版没判过）；门禁本身退 77 不是绿，不算第一句。
- 推翻条件：真仓上同样两步之后 `stage-marker-check` 退 1，或第 1 步之前另有一步核工作区等于暂存区而我没读到。

### H1-3b　「与别的五道同规矩」只做了标记一半

- 别的四道：`.claude/skills/crash-test/commit-stages.md:19`：「两样有一样不过就不跑那一道，停下」——输入路径上工作区与暂存区不同就不跑、不出判定。
- 这一道：判绿退 0 与写不写标记无关。`.claude/gate.d/checker-tier-broad-mutation.sh:555`：「elif ((stage_exit == 0 && sample_mode)) && [[ -f "$SAMPLE_DIRECTORY/write-marker" ]]; then」之后 `.claude/gate.d/checker-tier-broad-mutation.sh:558`：「exit "$stage_exit"」；没写标记只打一行「本次未写全绿标记」。
- 这一道的判定读的不只是导出范围。驱动本身取工作区：`.claude/gate.d/checker-tier-broad-mutation.sh:90`：「DRIVER="$STAGE_REPOSITORY/research/scripts/broad-mutation.py"」，排除表取驱动旁边那一份：`research/scripts/broad-mutation.py:300`：「EXCLUSIONS_FILE = os.path.join(SCRIPT_DIRECTORY, "broad-mutation-ignored-exclusions.tsv")」，剪枝规则同样：`research/scripts/broad-mutation.py:2206`：「specification = importlib.util.spec_from_file_location("mutation_prune_for_broad_mutation", os.path.join(SCRIPT_DIRECTORY, "mutation-prune.py"))」。②③④ 都在主工作区起：`.claude/skills/crash-test/commit-stages.md:23`：「在主工作区按次序做四步」。
- 旁证（量过）：排除表暂存一份、工作区多一行，按门禁载入驱动的办法读：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp2-probe.txt:2`：「门禁第一格算全局键用的 exclusions_digest_of(EXCLUSIONS_FILE) = 281b3133ea67eca2」，`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/exp2-probe.txt:3`：「暂存区那一份的 sha256 = 0ed6758525bbadd3」。

两种现场（推的，要真跑驱动才量得到，交主 agent）：
1. 工作区排除表与暂存的不同、②–④ 期间不变：②③④ 一致读工作区那一份，④ 判绿退 0、不写标记。整轮在临时 worktree 里用暂存的驱动与排除表现算全局键，与 ② 记的不同，第一格红：兜得住。
2. 工作区的驱动或 mutation-prune.py 有没暂存的改动、没动 `JUDGE_RULES_VERSION`（例：别的会话正改到一半；或主 agent 自己改着工具、这一次只暂存 crates）。全局键只记判法版本这一个常量（`research/scripts/broad-mutation.py:2246` 起的 `global_key_lines` 首行是 rules），结果行不记驱动内容。③ 用工作区那份判，④ 判绿；整轮里第一格全局键相同照绿，第二格只数结果行照绿；提交的是暂存的驱动，判的是工作区的驱动。改坏的样子照驱动已有的开关就有一类：`BROAD_MUTATION_BREAK=stage-two-missed-as-caught`（第二段没红记成抓到）那种改动没暂存时，这一版的没红全记成抓到，两处都绿。

四句：
- 分不分辨改法：第一轮第 3 行的改法（只管标记）不中这一格。可选改法：(i) 第 19 行给这一道也加别的四道那两核、核的路径取登记行里导出范围之外的那几条（驱动、mutation-prune.py、排除表、crates-mutation-rows.py、run-with-memory-cap.sh、capped.sh、stage-cells.sh、阶段自己），脏就不起 ②；(ii) 驱动与剪枝规则的内容 sha256 进全局键（与「判法变了整版重判」同字面，代价是每改一次驱动就整版重判，要先量）。都没实现、没跑，被攻过零轮。
- 系统当时看不看得到：看得到，`git diff` 列得出那几个文件；阶段在写标记前已经算了这一份（`marker_inputs_clean`），只是没让它影响退出码。
- 判据分句：现场 2 落「一个输入让门禁该红不红」（推的）；现场 1 落不上（整轮兜住）。
- 推翻条件：真跑驱动时 ③ 用的是暂存的那份驱动（例如 ③ 实际在临时 worktree 里起），或主 agent 流程里另有一步核驱动没脏。

### H1-4　钉住 tree.source：没打中

- 版本目录里第一格读的三份由指纹定：manifest.tsv 的内容决定源码指纹、全局键全文的 sha256 进指纹，toolchain.txt 只在不在时写。会在两格之间变的只有 results.tsv（只追加、按行里的指纹过滤），别的会话往同一版追加结论是合法的新信息。真跑重写 global-key.txt 在核过全局键之后：`research/scripts/broad-mutation.py:2800`：「check_global_key(library, arguments)」在前，`research/scripts/broad-mutation.py:2664`：「write_atomically(library.version_file("global-key.txt"), pruner.global_text.encode())」在后，写进去的全文与记的同 sha256。
- 试过的形状：A→B→A 换回（tree.source 里 synced_at_ns 变了，跑完再核照红）；② 半途被杀留 pending（第一格红）；第一格之后、第二格之前别的会话 `--sync-tree` 半途被杀（tree 换了、指针没换：第二格判的是 A 的结果表，判对）；同一提交重做 ②（保守误红，出路句写的「别的会话在跑 --sync-tree」照实）。

### H1-5　全局键：门禁现算与驱动记的同一份（没打中）；全局键的射程（旁支，射程外）

- 同一份：照门禁第一格的算法（`global_key_lines`、工具版本取 `tool_versions(llvm_bin=None)`、限时取那一版记的那一行）在真库此刻那一版的清单上现算，与那一版 2026-10-01 写下的 global-key.txt 比：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h1-5-real-key.txt:3`：「current (gate algorithm) sha256=1474ec27f69e1d61 lines=16」，不同的只有 `research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h1-5-real-key.txt:7`：「only-current exclusions 0ed6758525bbadd34240e1caa15e1803c6da97cca3dbd27b556ae02782167d2a」与对应的 only-recorded 一行。排除表在那之后确实改过（暂存区那一份的 sha256 正是 0ed67585…，见 `exp2-probe.txt` 第 3 行），工具链、Cargo 文件、三个工具版本、限时这几行逐字节相同。实现员报告第 5 条的推翻条件（「② 之后门禁第一格报全局键不同而没人改过」）在今天的真库上没出现。
- 沿用范围：全局键进指纹之后，键一变就是另一版、`carry_decision` 按键不同一条不沿用；键不变、源码变了照沿用。没找到「只该在全局键变了时不沿用」之外的情形。
- 旁支（量过，射程外，交主 agent 定要不要另立一题）：全局键只认五类环境变量，`research/scripts/broad-mutation.py:2254`：「if name in ("RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS") or name.startswith("CARGO_PROFILE_"):」；仓里 admission.py 自己的构建环境定义宽得多，`research/scripts/admission.py:44`：「环境变量 RUSTFLAGS、CARGO_ENCODED_RUSTFLAGS、CARGO_PROFILE_*、CARGO_BUILD_*（CARGO_BUILD_JOBS 除外：只定编译并行度，」，还有 CARGO_HOME 下的 config。模型上逐项换：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h1-5-env-gap.txt:4`：「CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS=-C overflow-checks=off global_key=d70784907e11d2a8」与不设时（第 1 行）同一个键；`RUSTC_WRAPPER`、`CARGO_BUILD_TARGET`、`~/.cargo/config.toml` 写 `[profile.dev] overflow-checks = false` 同样不变（第 5–7 行），admission 那一份每一项都变。③ 经 detached-run 带 HOME 与 CARGO_HOME 进单元，cargo 照读用户级配置：用户级配置改了 overflow-checks，下一次提交照沿用旧配置下判的「抓到」（推的）。

### H1-6　`--changed-since` 重选超时与撞顶：机械上没打中

`research/scripts/broad-mutation.py:2017`：「elif (plans is not None and state in ENVIRONMENTAL_OUTCOMES and not arguments.stage_two_only」只对 `plans` 里要现判的条成立。超时、撞顶不进结论表（`research/scripts/broad-mutation.py:1875`：「if runner.pruner and outcome in DECISIVE_OUTCOMES:」），所以它们总是「no-record」、总是要现判，总会被重选；同一版重起 ③、或另一版的 `--changed-since` 都选得中。改了 `--timeout-floor`、`--memory-cap` 不进全局键（`research/scripts/broad-mutation.py:2267` 的 limits 行只有 cpu 与 wall 四个数），`--item <序号> --rerun` 不会撞退 7。走不出去的在 H3。

### H1-7　两行各数互核：没打中（量过）

把门禁 gate-verdict 那段内嵌 python（门禁第 344–443 行原样）抽出来，喂驱动 `summarise` 在随机结果表上打的输出：每条 0–3 行，结局从七种里取，三成先带一行沿用的抓到，一成五不跑。`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h1-7-combos-5000.txt:4`：「mismatches=0」，5000 趟里判绿 748、判红 4252，判定与「每条都有抓到或无效的结论」逐趟一致，「对不上」一次都没误报；另一个种子 400 趟同样 0。驱动那一行的 `fresh` 是 `concluded - carried` 算出来的，`carried + fresh = concluded` 恒成立；汇总行八栏与判定行按同一份状态数，各等式在这些组合下都成立。没覆盖：结果表写坏、`--summary` 退 1 的那一支（门禁按「不是 0 也不是 8」红，不经互核）。

## H2　样本与自证够不够得着

### H2-a　共用函数写错：11 种写错版让 26 份样本一份不翻（量过）

`h2-mutants.py` 在冻结副本的硬链接拷贝里只换驱动这一份，逐个造 16 种写错版（门禁文件一字不动，样本照样本档起法跑）。`outputs/h2-sweep.txt` 原样（行尾明细略）：

| 写错版 | 26 份样本 |
|---|---|
| fingerprint_of_manifest 只算路径（fop-content-blind） | 判错 0 |
| fingerprint_of_manifest 只算内容（fop-path-blind） | 判错 0 |
| version_key 丢工具链 / 丢源码指纹 / 丢判法行（vk-no-toolchain、vk-no-source、vk-no-judge） | 各判错 0 |
| version_key 丢全局键（vk-no-global） | 判错 1：version-key-red |
| global_key_lines 丢 rules / toolchain / file / env / tool / limits 行 | 各判错 0 |
| global_key_lines 丢 exclusions 行；exclusions_digest_of 恒为 none | 各判错 1：global-key-red |
| limits_line_of 恒为「limits」 | 判错 5（格式变了，门禁按「那一版没记」红） |
| export_paths 丢 .cargo | 判错 11 |

逐行原样举三行：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h2-sweep.txt:1`：「fop-content-blind 判对 26 个、判错 0 个：无」，`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h2-sweep.txt:9`：「gkl-no-files 判对 26 个、判错 0 个：无」，`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h2-sweep.txt:11`：「gkl-no-tools 判对 26 个、判错 0 个：无」。原样副本上 `research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/samples-base.txt:1`：「判对 26 个、判错 0 个：无」。

为什么不翻：prepare.py 造「那一版」与门禁复算走同一个函数，`.claude/gate.d/checker-tier-broad-mutation.sh:264`：「staged_source_fingerprint = driver.fingerprint_of_manifest(staged)」、`.claude/gate.d/checker-tier-broad-mutation.sh:319` 起的 `driver.global_key_lines(...)`、`.claude/gate.d/checker-tier-broad-mutation.sh:326`：「and driver.version_key(home, staged_source_fingerprint, current_toolchain, global_digest) != fingerprint):」；样本里与函数无关的只有三处手写：version-key-red 的旧式子、global-key-red 的排除表在不在、source-fingerprint-red 的全零。所以门禁的三个判红分支——源码指纹不同（第 272 行）、全局键不同（第 321 行）、版本指纹不对（第 325 行）——在函数写错时只剩这三份手写样本盯，而它们各自只盯函数的一个角。

驱动自己的 `--selftest` 盯不盯得住这 11 种（推的，读代码；子 agent 起驱动拷贝的自证被闸拒）：
- 盯得住（推）：fop-content-blind（剪枝格第二版只改函数体：`research/scripts/broad-mutation.py:3722` 那一句；只算路径时两版同指纹、旧行算这一版的，`--changed-since` 选不出 2、4、5）；vk-no-toolchain、vk-no-source、vk-no-judge（`research/scripts/broad-mutation.py:3409` 的期望指纹是自证自己写的式子，不调 version_key）；gkl-no-rules（`research/scripts/broad-mutation.py:3915`：「and new_key_text.startswith(f"rules {JUDGE_RULES_VERSION}\n")」）。
- 我找不到会红的格（推）：gkl-no-toolchain（工具链格换工具链后只看 `not_run == 8`，不跑 `--changed-since`，丢了这一行也照过：`research/scripts/broad-mutation.py:3423`）、gkl-no-files、gkl-no-env、gkl-no-tools、gkl-no-limits（自证里换全局键输入的只有排除表一格：`research/scripts/broad-mutation.py:3435`；工具版本、限时、编译变量、Cargo 文件在 sync 与真跑之间都没换过）、fop-path-blind（没有只改文件名的格）。这 6 种要是主 agent 跑 `driver-selftest-mutants.sh` 也「都对」，就是谁都不盯。

后果（推的）：gkl-no-files 下改 Cargo.toml 的 `[profile.dev] overflow-checks` 不换全局键，下一次 ② 的新一版照沿用旧配置下判的抓到；gkl-no-tools、gkl-no-limits 同理换 nextest 版本、改 CPU 限时（卡住算抓到的门槛）照沿用。门禁第一格的全局键分支在这几种写错下与样本一起瞎。

四句：
- 分不分辨改法：可补的两种（被攻过零轮，没实现）：(i) 一份不共用函数的绿样本，prepare.py 写死一段手写的 global-key.txt 全文与手算的源码指纹、版本指纹字面值，函数丢哪一行都对不上、绿样本翻红（代价：驱动公式合法地改了要同时改这份字面值，这正是要它盯的）；(ii) 驱动自证补一格，在 sync 与真跑之间逐类换工具版本（`BROAD_MUTATION_TOOL_VERSIONS`）、限时（`--cpu-floor`）、编译变量（`RUSTFLAGS`）、Cargo.toml，各要退 7。(i) 修门禁这一侧，(ii) 修驱动沿用这一侧，两样不互相替代。
- 系统当时看不看得到：门禁看不到，它判对错用的就是被判的那个函数；这是方法上的缺口，不是现场的缺口。
- 判据分句：正文第五节第三句「一个判红分支只有与被判代码共用同一段函数的样本盯着、那段函数写错时样本不翻面」，分支是第 272、321、325 行，函数是 fingerprint_of_manifest、global_key_lines、version_key。
- 推翻条件：主 agent 跑 `driver-selftest-mutants.sh`，那 6 种写错版各有格红。那样只剩「门禁样本不翻」这一半，(i) 仍成立、(ii) 不必做。

### H2-b　20 个开关：没打中（量过）

逐个打开、26 份样本照跑（`outputs/switches.txt`，20 行），翻面的样本与门禁文件头「弄坏开关」那一段逐项相同，也与实现员报告那张表逐项相同。互相兜住的两处——global-key-unchecked 下 global-key-missing-red 仍退 1（version_key 那一支红）、version-unchecked 下 swapped-between-cells-red 仍退 1（跑完再核那一句红）——文件头都写了，stage-selftest 按 want 判，缺 want 照样判错，不算漏盯。

### H2-c　这一批新加、没有样本的判红分支（低价值）

- `.claude/gate.d/checker-tier-broad-mutation.sh:310`：「if hashlib.sha256(recorded_global.encode()).hexdigest() != global_digest:」没有样本也没有开关。删掉它，global-key.txt 被换成此刻全文而 tree.source 记的是别的键时会判绿；今天的驱动只在核过全局键之后重写 global-key.txt，造不出这种现场，所以只记一笔。
- 汇总行与判定行互核里的「抓到加没红加无效对 concluded」「八栏之和对 total」两条，与已有的 `concluded + without = total` 和另两条合起来是冗余的（任一条单独删掉，能触发它的输入都会被别的几条红），`carried + fresh` 那条在驱动是算出来的恒等式。都不构成漏判。

## H3　丙的出路（兼攻）

### H3-a　确定性的超时、内存撞顶：出路走不出去，交用户（推的，按代码）

- 超时只出在编译步与文档测试步：`research/scripts/broad-mutation.py:1639`：「return int(max(self.arguments.timeout_floor, self.arguments.timeout_multiplier * self.baseline_seconds.get(step_name, 0)))」（测试步经 runner 按单条 CPU 限，停了算抓到）。文档测试步每条变异都整步跑。一条变异让某个文档测试死循环，就是每次都超时，加大 `--timeout-floor` 只是等得更久。
- 内存没有按单条测试的限，测试步整步经内存包装，撞了记撞顶：`research/scripts/broad-mutation.py:1907`：「if exit_code == MEMORY_CAP_HIT_EXIT:」。一条变异让某个测试无界分配，CPU 限还没到就先撞 24G；`--memory-cap` 加不过本机可排的内存（再大内存包装退 252、驱动退 3 不落行）。
- 这两种不进结论表（`research/scripts/broad-mutation.py:1875`），下一版里它的键没有记录，`research/scripts/mutation-prune.py:642`：「if record is None:」交 no-record、现判。所以确定性的那一条在之后每一次提交的 ③ 里都被重选、每次付满一整步限时、每次照旧没结论。
- 出路的末一句是「仍没结论的照旧红、交用户定」（门禁 gate-verdict 格登记的出路、驱动文件头第 95 行同句）。门禁与驱动里没有任何地方读「用户定了这一条算什么」：没有忽略表、没有按条豁免、`.claude/gate.d/checker-tier-broad-mutation.sh` 也不读别的输入。用户能定的只有「这一次红着提交」，而那之后每一次提交的这一道都红（整轮因此也前移不了 staged-green）。
- 唯一真走得通的是改代码：给第一段加一条能杀它的测试（第一段先红、轮不到文档测试与撞顶的那一步）。出路文字里这一句只写给「没红的」，没写给确定性超时与撞顶。

四句：分不分辨候选——用户定案丙的三个候选里，甲（整步超时算抓到）在超时这一半上不中，内存撞顶在甲乙丙下都还中；系统当时看得到吗——看得到，同一条两次超时、同一步、同一限时；判据分句——正文第五节第四句「丙的出路在某种结局组合下走不出去」；推翻条件——门禁或驱动里另有我没找到的、按条记下用户决定的输入。

### H3-b　② 与 ③ 的环境不同：主 shell 设了编译变量时 ③ 永远退 7（条件式，推的）

- ② 在主会话的 shell 里起（`.claude/skills/crash-test/commit-stages.md:25`），记下的全局键带那一刻的 `RUSTFLAGS`、`CARGO_PROFILE_*` 等；③ 经 detached-run 起，带进单元的只有五个：`research/scripts/detached-run.sh:209`：「DETACHED_RUN_CARRIED_VARIABLES=(SINGLEFS_HEAVY_TESTS PATH HOME CARGO_HOME RUSTUP_HOME)」。主 shell 里设了其中任何一个（模型上 `CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false` 就换全局键：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h1-5-env-gap.txt:3`：「CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false global_key=a65e82f0ebce9180」），③ 的 `check_global_key` 每次退 7。退 7 的出路是在同一个 shell 里再做 ②（`.claude/skills/crash-test/commit-stages.md:28`：「7 是工具链或全局键与这一版记的不同，出路照驱动打的另起一版」），记下的还是带变量的键，③ 照退 7。④ 与 ② 同一个 shell、全局键对得上，第一格绿、第二格没结论红。
- 我这个会话的环境里这几个变量一个都没设（`env | grep -E '^(RUSTFLAGS|RUSTDOCFLAGS|CARGO_)'` 空），所以这是条件式的；设了的会话照上面走不出去。改法（零轮）：③ 也在前台的同一个环境里起，或 detached-run 把全局键认的那几类变量也带进单元。

### H3-c　规模（观测）

- 真库此刻那一版：`research/prompts/m3-broad-gate-gov-r2-opus-model/outputs/h1-5-real-key.txt:1`：「tree.source fingerprint=e1ba257413cbad99 global_key_in_pointer=None」，list.json 5091 条，结果表除表头 2 行（只读看过）；结论表只有 2 行、键是 fbd2f8c1…，而此刻现算的是 1474ec27…（排除表改过）。所以下一次提交的 ③ 是 5091 条全量现判，第一轮判决推的 10.6–82.0 小时落在这一次。
- `git log --since=2026-09-01`：碰 crates 或 Cargo 文件的提交 23 个，其中碰 Cargo.toml、Cargo.lock、.cargo 或某个包的 Cargo.toml 的 5 个，每一个都换全局键、整版重判（E4 的设计，不是这一批引入的）。排除表还没进过任何一个提交，进了之后每改一行同样整版重判。
- 新判法下真库里超时、撞顶占多少没有数（新一版只判了 3 条）；旧一版 304 行里 7 行超时，全是 one-test 整步超时，新判法下这类走 runner、算抓到。

## H5　几个会话共写库目录与工作区（推的，按代码）

| 别的会话在什么时候做什么 | 谁拦 | 结局 |
|---|---|---|
| ④ 两格之间 `--sync-tree` | 第 4 条钉住、跑完再核 | 红（拦住） |
| ② 之后改排除表、工具版本、限时、编译变量，再起 ③ | 第 5 条：`research/scripts/broad-mutation.py:2800` 真跑前核全局键，退 7 | ③ 不跑（拦住） |
| ③ 跑完之后、④ 之前 `--sync-tree` 到自己的暂存树 | 第一格逐文件比 | ④ 红（拦住） |
| ② 之后、③ 之前 `--sync-tree` 到自己的暂存树 | 没人拦 | ③ 照跑别人那一版：`research/scripts/broad-mutation.py:2794`：「library.load_source(for_run=True)」读此刻的指针，不核是不是 ① 那个提交；跑几小时之后 ④ 第一格红，只白费时间，不出假绿 |
| ②–④ 期间在共用工作区里改驱动或 mutation-prune.py、不暂存、不动判法版本 | 没人拦 | 同 H1-3b 现场 2：③ 用改到一半的那份判，④ 与整轮都绿（假绿的那一种） |
| 两个 ③ 同一版并跑 | 库目录锁 | 后一个等 120 秒退 4 |

第 4 条拦的是「④ 期间换指针」一种；第 5 条拦的是「全局键的输入在 ② 之后变了」一种。白跑那一格的改法（零轮）：③ 加一个「只跑这个提交」的参数（`--expect-commit <①>`，指针不是它就退），或 ② 打出指纹、③ 核指纹。

## 自己提的改法：各修哪一格

全部只在我的模型上量过或只推过，被攻过零轮。

| 改法 | H1-3a | H1-3b | H2-a | H3-a | H3-b | H5 白跑 |
|---|---|---|---|---|---|---|
| 第 19 行给这一道加别的四道那两核（导出范围之外的登记路径脏就不起 ②、不查标记） | 推的：修 | 推的：修 | — | — | — | — |
| 这一道的复用指纹按索引算 | 推的：修 | 推的：不修（判定照读工作区） | — | — | — | — |
| 驱动与剪枝规则内容进全局键 | 推的：不修 | 推的：修（整轮第一格会红） | — | — | — | — |
| 一份写死字面值的绿样本 | — | — | 推的：修门禁这一侧 | — | — | — |
| 驱动自证补「逐类换全局键输入要退 7」 | — | — | 推的：修驱动这一侧 | — | — | — |
| 按条记下用户决定、门禁认它（要用户先定记成什么） | — | — | — | 推的：修 | — | — |
| ③ 不经 detached-run、或 detached-run 带上全局键认的变量 | — | — | — | — | 推的：修 | — |
| ③ 核指针是不是 ① 那个提交 | — | — | — | — | — | 推的：修 |

## 没打中的形状

- 钉住：A→B→A 换回、② 半途被杀、两格之间半途被杀的 `--sync-tree`、同一提交重做 ②、别的会话往同一版追加结果、`--import-results`（导入核指纹，指纹里有全局键）。都没找到假绿。
- 全局键同一份：真库一次（16 行，见 H1-5）；样本里的全局键都由同一个函数造，不算第二次。
- `--changed-since` 重选：非确定的超时与撞顶、过过第一段只补第二段、换版本后重选。
- 两行互核：两个种子共 5400 趟随机组合（每趟 1–6 条，每条 0–3 行、七种结局、三成带沿用行）。
- 重型闸：`--check gate-verdict` 与别的格点名写成两次 `--check`、写成 `--check=` 的并集（`checked_cells` 取并集，照算重型），没找到放过去的。
- 样本档：26 份原样全对；20 个开关逐个，翻的与文件头一致。
- 取样范围的限：驱动写错版只造了 16 种，都是「丢一行、丢一个字段」这一类，没造「换算法」「换次序」；门禁分支删除只在抽出来的 python 与读代码上推，没改门禁文件跑样本（样本档要求阶段文件与登记的逐字节相同）。

## 这条腿自己的限度

- 「没打中」都只抽了一次样（H1-4、H1-5 同一份、H1-6、H1-7、H2-b），照规则不支撑结论。
- H1-3a 的第 4 步（④ 自己退 77）与整轮兜底、H1-3b 现场 2、H3 全部、H5 表里后两格都是推的：要驱动真跑或整轮门禁，子 agent 不起。
- H2-a 的驱动自证那一半是读代码推的；只要主 agent 跑出那 6 种里有格红，那一种就从「谁都不盯」降成「只有门禁样本不盯」。
- 样本档是在我的脚本里逐份起的（与 stage-selftest.sh 同一种拷贝、setup、cd 进样本根、根参数写同一个目录、`--force`）；单独直接起那一次，第一回把根参数写在变量里被重型闸拒，改写成字面路径后放行，没有绕开。

## 要主 agent 跑的命令

1. H2-a 驱动自证那一半（驱动拷贝的 `--selftest` 子 agent 被拒）：工作目录任意，
   `SINGLEFS_HEAVY_TESTS=user-request nice -n 19 bash research/prompts/m3-broad-gate-gov-r2-opus-model/driver-selftest-mutants.sh /tmp/claude-1000/m3-gov-r2-freeze/tree <草稿目录>`
   不经内存包装（驱动自证里自己经包装起假一步），线程不吃。看每行末尾：base 要是「55 格都对」；gkl-no-toolchain、gkl-no-files、gkl-no-env、gkl-no-tools、gkl-no-limits、fop-path-blind 六行要是也「都对」，H2-a 那一半坐实；fop-content-blind、vk-no-toolchain、vk-no-source、vk-no-judge、gkl-no-rules 五行我推的是有格红。
2. H1-3a 真仓复现（要先有一份合法标记，提交时第 2b 步 ④ 判绿写过之后最方便）：主工作区里挑导出范围内一个文件，改成 B、`git add`，再 `git restore --source=HEAD <文件>`（只还原工作区），跑 `python3 research/scripts/admission.py stage-marker-check $PWD checker-tier-broad-mutation.sh`。预期退 0、打「ok」。之后 `git restore --staged <文件>` 收回。再跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/checker-tier-broad-mutation.sh` 看是不是退 77。
3. H1-3b 现场 2 与 H5 末两格：要驱动真跑，按 commit-stages 第 2b 步做一次，期间在工作区给 `research/scripts/mutation-prune.py` 加一处不暂存的改动（例：`CARRIED_OUTCOMES` 加 `"missed"`），看 ④ 与整轮 `gate-staged.sh` 判什么。这是重型，要用户同意，跑完撤回。

## 没做什么

- 没跑重型：驱动的任何真跑、`--sync-tree`、门禁对真仓整道或 `--check library-version-is-staged-tree`、`stage-selftest.sh`、整轮门禁、驱动拷贝的 `--selftest`。
- 没读禁读清单里的辩方产物与调度记录；没碰 H4（commit-stages 第 28 行退出码表、main-agent 第 12 条指针、三处清单逐项对），只在 H3-b 用到 detached-run 带哪几个变量。
- 真库只读：读了 tree.source、两版的 list.json 条数、结果表、baseline、global-key.txt、conclusions.tsv，没起驱动的写操作。
- 草稿目录 `/tmp/claude-1000/gov-r2-attack/`：里面有冻结副本的拷贝 `trees/base`、16 棵硬链接拷贝 `trees/h2/`、`rr/trees/`、`exp2/`、`rr/exp2/`（各是 git init 的临时仓）与样本输出；交回前删掉这些仓副本，只留 `out/`、`h2/`、`h1-5/`、`h1-7/` 下的文本。
