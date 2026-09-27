# 门禁批第一轮判决（defs-gatebatch-m2-r1，2026-09-26）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 F1 F6 F7 F8 F9 F10 F11 F12 F13 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 -->

## 一、这一轮

- 正文 `research/prompts/_defs-gatebatch-m2-r1-body.md`，背景材料 `_defs-gatebatch-m2-r1-background.md`，附录二 `_defs-gatebatch-m2-r1-diff.md`（门禁批 diff 的「改前」一侧是它 `cp -p` 的备份，不是某个 git 提交，射程只到这一批），开工快照 `research/prompts/defs-gatebatch-m2-r1-snapshot/sha256sums.txt`（派核查员前主 agent 核过 11 个全 OK）。
- 腿：云端正推 `defs-gatebatch-m2-r1-sonnet-output.md`（只做文字核对，探针目录空）；云端攻方 `defs-gatebatch-m2-r1-opus-output.md`（模型 `defs-gatebatch-m2-r1-opus-model/`）；本地攻方 `-local-attack-output-s1.md`、`-s2.md` 两份干净（本地攻方的定义被别处把 `model` 改成 `local-model`，换账号后起不来，主 agent 派发时给 `model: sonnet` 覆盖）。
- 核查员 `defs-gatebatch-m2-r1-verifier-output.md`：104 处 ✓101 ✗1 分不清 1。✗ 在正推：它说 `.claude/gate.d/stage-inputs.tsv` 四条崩溃枚举行的 `count-line=` / `exhaustive=` / `threads=` 都齐，实际第 34 行（会话里推抬 F 那条）三项全缺、第 35 行（c561 那条）只有 `count-line=`。分不清的那一处：正推说层 0 规模第三轮判决里没有一处写「覆盖」，核查员现跑是 1 处（第 51 行）——那一行是主 agent 在正推交回之后照它的报告补的（正推报告 13:06:45 UTC，那一行 13:07:43 UTC），正推核的时候确实没有，它判得对。攻方 38 处引用与 12 处复跑全 ✓，J1-a、J1-b、J4-b 三格复跑逐字节一致。

## 二、逐格判

| 格 | 判 | 依据 |
|---|---|---|
| J1-a 只改登记表或 54 号本身 | **打中（量过），这一批新带进来的（改 54 号本身那一形之前就有）**：范围那一问（`54-layer0-replay.sh:114`）只按 `crates/ Cargo.toml Cargo.lock` 判，只改 `stage-inputs.tsv` 或 54 号的提交快档退 77、一格标记都不核；54 号的自证全带 `SINGLEFS_GATE_FULL=1` 跑所以没抓到；先提交带 ignore 的用例、后提交登记行，新用例一次都不会被核 | 攻方 J1、核查员复跑 |
| J1-b 判法不进指纹 | **打中（量过）**：判日志的代码搬进了 `admission.py`，它不进每条用例的指纹（`admission.py:168`），修前修后指纹都是 `639097891a56ec19…`，旧判法写下的标记照样被复用 | 同上 |
| J1-c 两趟并发 | 低危、推的：后一趟判红会删掉前一趟刚写的绿标记，只会假红 | 攻方 |
| J2-a 排除法漏认 | 打中（量过），今天的仓里没有这种写法：`include!(concat!(…))` 编译期拼文件名、别的包的 `build.rs` 按目录读本包 `tests/`，被减掉的文件其实编进了用例 | 攻方 |
| J2-b 排除法减得太少 | **打中（量过，真仓拷贝）**：只改 `crates/mutations.tsv`、`tests/common_tree_split/mod.rs` 或 harness 的 `src/bin/e142_*.rs`，四条用例的指纹全变，第二条流约 2.3 天（推的）要重跑；与用户「改了只跑改了的部分」相反，而每处改法都要在 `mutations.tsv` 留一行，这种提交最常见 | 攻方 |
| J3-a 指纹漏的构建输入 | 打中（量过），今天走到的可能性低：runner、`RUSTC_WRAPPER` 指的脚本换了内容，配置里 `build.rustc` 指的编译器换了版本，指纹都不变 | 攻方 |
| J3 其余 | `CARGO_BUILD_JOBS` 不进指纹对；主工作区与 worktree 的指纹不同只会偏假红，造不成假复用 | 攻方 |
| J4-a 闸的绕法 | 打中（量过，只喂 JSON）：`cargo nextest`、`cargo mutants`、`--config` 别名与 runner、`CARGO_TARGET_*_RUNNER`、拷走测试二进制再执行、`find -exec`、`/usr/bin/time`、`flock`、`rustup run`、`chrt`、`systemd-run --scope`、`prlimit`；包装这一类以前就有，层 0 同样放行 | 攻方 |
| J4-b 用例的 ignore 没人查 | **打中（量过）**：去掉 c561 那条的 `#[ignore]` 之后，不带 `--ignored` 的命令跑 2^18 个状态，闸放行、`crash-cases` 自查退 0 | 攻方、核查员复跑 |
| J4-c 误拒 | 打中、危害小：`-- --ignored --list`、`--include-ignored --exact <一条快用例>` 被拒 | 攻方 |
| J5 文字 | 几份文字之间没有互相矛盾，都改成了逐条复用；层 0 规模第三轮判决「乙不采纳」与用户 18:5x 定案之间没写覆盖——主 agent 已在那份判决第四节之后补了覆盖记录；正推说的「四行三项都齐」不对（核查员 ✗），见 Y8 | 正推、核查员 |
| 本地 | 两份样本一致：四条用例保留、减去的文件数与四个抽样文件的去留与 `admission.py crash-case-manifest` 现跑对得上 | 本地、核查员 |

## 三、改法（被攻过零轮，第二轮攻）

- **Y1**（J1-a、J1-b，攻方 F1，副本上量过）：54 号范围那一问再加 `.claude/gate.d/stage-inputs.tsv`、54 号自己、`research/scripts/admission.py`；`admission.py` 按 `--extra-file` 进每条用例的指纹；54 号自证的 `expected_fingerprint` 跟着改，另补一格「只改登记表的提交不带 `SINGLEFS_GATE_FULL=1` 也判红」。
- **Y2**（J4-b，F6 量过、F9 推的）：`crash-cases` 自查要求登记的用例函数标了 `#[ignore]`，没标就退 2；闸在登记的测试目标被点名、却不带 `--ignored` 时读源码核那条函数标没标 ignore，没标按重型算。
- **Y3**（J4-a，F7、F8 推的）：闸把 nextest（`--run-ignored`）、mutants、miri、llvm-cov、hack 也当跑测试的子命令；`--config` 里有 `alias.` 或 `.runner`、环境里有 `CARGO_TARGET_*_RUNNER` 的当可能带 `--ignored`；前缀表补 `/usr/bin/time`、`flock`、`rustup run`、`chrt`、`prlimit`、`systemd-run`、`strace`、`perf`；任意可执行文件带 `--ignored` / `--include-ignored` 且参数里有登记的用例函数名的按崩溃枚举用例算。闸是执行前的一道，看门狗在进程那一层兜底，两道都留。
- **Y4**（J4-c，F10）：带 `--list` 的不算重型；`--include-ignored --exact <快用例>` 那一种接受误拒，写进闸的文件头。
- **Y5**（J2-a，F11）：包里出现 `include!(concat!(`、或任何一个包的 `build.rs` 读 `tests`，这个包一份都不减；另把这两种写进 `admission.py` 文件头「认不出的」那一段。
- **Y6**（J2-b，F12）：`crates/mutations.tsv` 不进崩溃枚举用例的输入；`src/bin/*.rs` 在没有任何 `.rs` 读 `CARGO_BIN_EXE_` 时不进。共用测试模块（`tests/common_*/`）要顺着模块图才减得准，这一批不做，代价写进文件头。
- **Y7**（J3-a，F13）：环境变量与 cargo 配置里 runner、wrapper 指的可执行文件按内容哈希进指纹，`build.rustc` 与 `rustc` 一类取 `-V`。
- **Y8**（核查员 ✗）：`stage-inputs.tsv` 第 34、35 行两条崩溃枚举用例补齐 `count-line=` / `exhaustive=` / `threads=`；那两条用例今天不打这几行的，写进报告交主 agent（要改 `crates/` 的测试，另派）。
- J1-c 不改，写进 54 号文件头「两趟并发只会假红」。

## 四、第二轮

改完之后开第二轮：只攻 Y1–Y8 的改后字面与代码，攻击面不重复第一轮（第一轮攻：范围那一问、判法进不进指纹、排除法漏认与减得太少、构建输入、闸的绕法与误拒、文字矛盾）。

## 五、按路径点名被判的定义与文件（门禁 72 号）

被判的：`.claude/gate.d/54-layer0-replay.sh`、`research/scripts/admission.py`、`.claude/gate.d/stage-inputs.tsv`、`.claude/gate.d/stage-owners.tsv`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/agents/crash-verifier.md`、`.claude/rules/implementation-workflow.md`、`research/scripts/check-segment-registry.py`；一并核过的 `.claude/main-agent.md`、`.claude/agents/gate-triage.md`。

## 回看决策

不涉及决策：被判的是门禁、研究脚本、hook、agent 定义与规则，不改 `.claude/kb/decisions/` 里任何一条决策。
