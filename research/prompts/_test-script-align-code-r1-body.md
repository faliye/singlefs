# 对接上游测试脚本规范：代码轮第一轮正文（2026-10-01）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 -->

## 一、题面

`.claude/rules/implementation-workflow.md` 三步里的第 2 步，第一轮。被判的是这一批四份 `crates/` 补丁合入之后的代码（`records/2026-10-01-对接上游测试脚本规范.md`「第 3 批：补丁合入与下一批」）。补丁原样在 `research/prompts/test-script-align-code-r1-snapshot/` 下四份 `impl-*.patch`；四份实现员报告是 `/tmp/claude-1000/impl-e165-whole-row/patch/report.md`、`/tmp/claude-1000/impl-crash-amp-lines/patch/report.md`、`/tmp/claude-1000/impl-gpu-parity/patch/report.md`、`/tmp/claude-1000/impl-seed-repro/patch/report.md`（结论句当线索，以代码为准）。

被判的文件（判决逐份写去向）：

| 文件 | 改了什么 | 压着的条款 |
|---|---|---|
| `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs` | `--pinned-cases` 读钉表只认 21 段（8 段自己的列加 13 段整行抄的红例账），账行第 3、4、5、12、13 段要等于 ②③④⑦⑧，不等判红；读表抽成纯函数 `parse_the_pinned_table` | 上游 `test-script-discipline.md`「全量的红，钉回快档」：钉表登记「出红的那一行整行抄下」 |
| `crates/singlefs-checker-tier/src/crash.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` 与四份 `crates/singlefs-checker-tier/tests/crash_amplification_*.rs`、`tests/common_crash_points/mod.rs` | 驱动行末尾接 `states_reused= exhaustive= expand_up_to= ignored_points= states_skipped_by_ignore=`；展开上限认不出就 panic；复用与剪枝关掉再判的对照；内层按挂钟打进度 | C604（快档不开特性，崩溃放量与 GPU 对拍编进来零条）、C605（崩溃放量的复用与剪枝没有关掉再判的对照）、C606（崩溃放量入口不写展开深度，缩过规模的全量照样退 0）、C611（长跑的项不按挂钟打进度） |
| `crates/singlefs-checker-tier/tests/common_gpu_adapter/mod.rs`（新）、`tests/gpu_adapter_probe_reports_not_run_without_an_adapter_and_red_when_no_card_starts.rs`（新）、`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs`、`tests/gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte.rs` | GPU 对拍没卡时打 `NOT_RUN_NO_GPU_ADAPTER test=<名>` 后返回，有 NVIDIA 独显而一张卡都起不来判红；C601 新输入与用例（只在第 32 次写之后不同的两个根槽版本） | C608（GPU 变异不罩内核与 GPU 对 CPU 的对拍）、C601（判器查版本不比组合掩码高 32 位时没有会红的流）、C620（GPU 对拍认「没卡」只看 Vulkan 枚举，驱动坏了被当成本次未跑） |
| `crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/tests/random_histories.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs` | 随机历史与崩溃注入判红时打的复现行带全种子、步数、比重、checker、盘宽、空间准入（崩溃注入另带每段崩溃状态数与短段摆法），入口照同一张变量表读回；格式只在 `history.rs` 一处定义 | C607（随机取样点用例红了凭打出的种子复现不出）；上游「用例原子化」：「随机输入带种子。红了打印种子，凭种子能单独复现那一条」 |
| `crates/mutations.tsv` | 四份各自追加（E165 那一份另删 1 行） | 每处改法一条「改回去它就红」的变异 |

## 二、实现今天的样子（主 agent 的观测，2026-10-01，合入之后）

- 冻结副本 `/tmp/claude-1000/test-script-align-code-r1/tree/`（`crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo/`），sha256 清单 `research/prompts/test-script-align-code-r1-snapshot/crates-sha256.txt`（220 份）；读冻结副本，不读主仓 `crates/`（别的会话在改）。
- 主 agent 现跑：harness 档四个包 `run-with-memory-cap.sh 24G cargo test --offline --no-fail-fast -j 31 -p singlefs-format -p singlefs-core -p singlefs-harness -p singlefs-checker` 退 0，`test result: ok` 102 行、0 失败；clippy 在合入那一趟过（`merge-and-verify.sh`）。checker 档包的测试一条没跑（重型，归提交时）。种子复现 harness 档 8 条变异行在副本上 `prove-red.sh` 证红（结果写进判决）。
- 读过的路径：上面那张表里的全部文件；`crates/singlefs-checker-tier/src/crash_identity.rs`（复用键与判法版本）、`crates/singlefs-checker-tier/src/verdict_store.rs`（判定块的键）没改，是 `crash_amplification.rs` 复用那一段的上游。

## 三、格

### K1　代码做的是不是条款说的
逐条款核：C604–C608、C611、C620、C601 各自「怎么拦」那一格写的检查，代码里有没有一处会红的实现；上游 `test-script-discipline.md`「入口脚本」那一节「每项结束打一行：项名、退出码、判了多少、红了多少、复用了多少」与驱动行新字段对不对得上（「不复用」与「取不出」两种写法、不写 0）。

### K2　哪一格会错（攻）
造输入攻改后的判法：E165 交叉核五对列有没有对错列、8 段旧行与别的段数分支；驱动行字段在零态、全复用、ignore 全跳过时是不是照实；展开上限的 panic 有没有被别的路径吞掉；复用对照在小规模上是不是真的关掉了复用与剪枝（不是只换个开关名）；GPU 取卡判定在「有卡但驱动坏」「非 NVIDIA 独显」「集显」几种枚举上的结局；复现行喂回入口在每段取样上是不是逐步同一段历史。

### K3　变异表
四份追加的行：锚点是不是唯一命中、替换文改的是不是那一判、点名的测试在它的目标里跑得到；E165 删的那一行删得对不对。

### K4　编码纪律
`.claude/singlefs-ai-sop/rules/code-discipline.md`「名字」「分支」「错误」：新名字说没说清、有没有通配臂、`unwrap`、魔法数字；`history.rs` 那张变量表是不是一处定义、两个装置共用。

### K5　测试原子化与快慢档
上游「用例原子化」「分快慢两档」：新用例一条一个场景、不读别的用例留下的状态；GPU 用例在没卡的机器上报「本次未跑」而不是假绿；新用例归在哪一档、harness 档还是 checker 档放对了没有（`.claude/rules/verification.md`「定义与名字」的归类判据）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | K1、K4、K5 | 从条款逐条推代码该有什么，核代码是不是做到；读冻结副本 |
| 云端攻方（Opus） | K2、K3 | 造反例、造变异，在冻结副本的拷贝上量；只跑 harness 档（`cargo test -p singlefs-harness …`，经 `research/scripts/run-with-memory-cap.sh`）与纯 Rust 小程序，checker 档包的测试不跑 |

## 五、跑前写死的判据

- 一条条款「怎么拦」的检查在代码里找不到会红的实现，或实现判的不是条款说的那件事 ⇒ 打中 K1，写回代码。
- 造出一个输入让改后的判法给错结局（该红不红、该绿不绿、该报本次未跑而判绿）⇒ 打中 K2。
- 一条变异行锚点不唯一、替换文改的不是那一判、或点名的测试跑不到 ⇒ 打中 K3。

## 六、交付

- 腿的报告写 `research/prompts/test-script-align-code-r1-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建，回复只写短句）。
- 探针放 `research/prompts/test-script-align-code-r1-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`；草稿与临时文件放 `/tmp/claude-1000/<腿名>/`。
- 引 kb 条款写 kb 文件名加那份文件自己的行号（去 kb 文件里现查，不从背景材料里数）；引代码写冻结副本里的路径与行号。
- 方案按 `crates/` 今天的实现来谈。不跑重型测试；不改主工作区；只停自己起的进程。
