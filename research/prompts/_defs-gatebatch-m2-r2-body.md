# 里程碑二收尾：门禁批（层 0 续跑接入、崩溃枚举按用例复用）三方，第二轮正文（2026-09-27）

<!-- doc-lint:not-numbers J1 J2 J3 J4 J5 K1 K2 K3 K4 K5 Y1 Y2 Y3 Y4 Y5 Y6 Y7 Y8 -->

## 一、这一轮要判什么

第一轮判决 `research/prompts/defs-gatebatch-m2-r1-main-verification.md` 第三节给了改法 Y1–Y8（被攻过零轮）。改法由通用 agent 做完交回（规格 `/tmp/claude-1000/gate-batch-m2-r1-fixes/spec.md`），被判的改动是它交回时的 diff `/tmp/claude-1000/gate-batch-m2-r1-fixes/my-changes.diff`（1707 行、6 个文件：`.claude/gate.d/54-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv`、`research/scripts/admission.py`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`.claude/rules/implementation-workflow.md`；sha256 `87c52ef8912779cdb388d740327e54e42079d247bfff6d9f1f9b235a944c0f24`，材料员原样放进附录二）。这一轮只攻 Y1–Y8 的改后字面与代码，攻击面不重复第一轮（第一轮攻过：54 号范围那一问、判法进不进指纹、排除法漏认与减得太少、构建输入、闸的绕法与误拒、文字矛盾）。

| 格 | 被攻的 | 问题 |
|---|---|---|
| K1 | **修补本身**：Y1 的 `--extra-file`（`admission.py` 进每条用例指纹）、Y2 的 `#[ignore]` 属性解析（`attributes_before_function`、`test_function_is_marked_ignored`）、Y3 的剥包装与别名 / runner 判定（`command_under_launcher`、`test_invocation`）、Y5 / Y6 的排除规则、Y7 的 runner / wrapper 按内容进指纹 | 每一处修补有没有引入新的假绿（该红不红）或新的假红（合法命令被拒、合法复用被作废）：`crash-case-manifest` 与 `crash-case-record` 两处算指纹时 `--extra-file` 是不是同一份；属性解析在 `#[cfg_attr(…, ignore)]`、`#[rustfmt::skip]` 夹在中间、宏生成的用例、`#[ignore = "…"]` 里带方括号的字符串上判成什么；剥包装递归有没有把一条重型命令剥成非重型（例 `flock … bash -c '…'`、`env -i`、`systemd-run --scope`）；`include!(concat!(` 只认字面时 `include_str!(env!(…))` 这类怎么算；runner 指向的脚本再读别的文件时指纹跟不跟 |
| K2 | **产生结论的方法**：改法的证红办法是「新自证格在旧代码上跑必须红」加「攻方探针在改后仓复跑」（交回报告逐条列的） | 这个办法证得了什么、证不了什么：旧代码上红只说明新检查认得旧行为，认不认得第一轮攻方造的那几种情形要看探针，探针里 O2、O4、B6、B7、R2 照旧 BROKEN 各自是「规格明写不做」还是没修好；54 号自己的场景自证 18 格加的 4 格用的是假 cargo，真 `--full` 一次没跑（Y8 的登记只在合成日志上核） |
| K3 | **从未被看过的地方**：`stage-inputs.tsv` 第 36、37 行两条用例的判法字段（floor-raise 那条 `count-line=LAYER0_PARALLEL_FINISHED threads=LAYER0_PARALLEL_FINISHED`、没有 `exhaustive=`；c561 那条只有 `count-line=C561_SIGMA_FULL`）与 54 号快档对这两条的判法 | 这两条今天在 54 号快档与 `--full` 里各判到哪一步：没有 `exhaustive=` 的一条，判绿要的只是一行计数行加线程判定，那么一次没跑完、只跑了部分切片就退出的日志会不会判绿；c561 那条没有线程判定，单线程跑完也判绿，用户「优先多线程」那一条在它身上没人看 |
| K4 | **指纹变了之后的处置**：Y1（准入模块进指纹）与 Y6（减去的文件变了）让四条用例的指纹都变，已有全绿标记全部不再作数 | 下一趟 `--full` 四条都要从头跑（第二条流约 2.8 天，推的）——这一次是改法引起的，以后每改一次 `admission.py` 四条又全变：这个代价对不对、有没有更窄的办法（只让判这条用例那几段进指纹）而不放过第一轮 J1-b 打中的「判法不进指纹」 |
| K5 | **第一轮判决自身**（辩方）：J1-a、J1-b、J4-b、J2-a、J2-b、J3-a 各自打中的是不是真的、改法对不对症、有没有被判决略过的攻方发现 | 逐条核第一轮判决第二节的判定与第三节的改法，替被判出局的写法辩护（例 J1-c「两趟并发只会假红」不改对不对） |

**共用问句**：照改后的字面与代码，哪一步会做错、放过、或误拒；给具体的命令、改动或派发情形；能在临时拷贝上量的量出来。

## 二、实现今天的样子（主 agent 的观测，2026-09-27 JST 00:2x）

- 改法交回之后主 agent 复跑：`python3 research/scripts/admission.py --selftest` → `✓ admission.py 自证通过：167 格都对`（改前 140 格）；`python3 .claude/hooks/lib_heavy_tests.py --selftest` → `✓ lib_heavy_tests 自检通过（查了 109 种：…）`（改前 55 种）；`bash .claude/hooks/heavy-test-guard.sh --selftest` → `✓ 自检通过（查了 628 种，其中该拒 112 种）`（改前 580 / 101）；62 号 `✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent）`；63 号绿。
- 改法自报的证红日志在 `/tmp/claude-1000/gate-batch-m2-r1-fixes/logs/`（`redproof-*.log`、`probe-*.log`、`final/`），改前六份文件的备份在同目录 `backup/`。
- 真仓四条用例（`python3 research/scripts/admission.py crash-cases .`）每条输入从 69 个文件减到 61 个（减去 `crates/mutations.tsv` 与 harness 的 7 个 `src/bin/*.rs`，改法自报，腿现查）；四条指纹都变了，全绿标记一个都不再作数。
- `crates/` 这一轮不碰（实八、E156、E158、实分一四条线在改 `crates/singlefs-harness/`）；开工快照 `research/prompts/defs-gatebatch-m2-r2-snapshot/sha256sums.txt`（11 个文件，与第一轮同一份清单）。
- c561 那条用例今天打不出 `exhaustive=true` 与线程行（`crates/singlefs-harness/tests/record_checker_judges_absence_by_the_persisted_set.rs:585` 只打 `C561_SIGMA_FULL states=…`，自己起线程、不经 `enumerate_layer0_in_state_slices`），要改 `crates/` 的测试，另派。

## 三、条款（材料员整段抄进附录）

- `.claude/rules/implementation-workflow.md` 全文（改后）；
- `.claude/agents/crash-verifier.md` 全文；
- 第一轮判决 `research/prompts/defs-gatebatch-m2-r1-main-verification.md` 全文；
- 第一轮正文 `research/prompts/_defs-gatebatch-m2-r1-body.md` 第一节那张表（J1–J5 的问题）；
- `records/2026-09-24-里程碑二收尾调度.md` 第三节「崩溃枚举的跑法」那一行（用户原话）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | K5，兼 K2 | 复核第一轮判决：每一处打中站不站得住、改法对不对症、判决略过了什么；再判改法的证红办法证得了什么 |
| 云端攻方（Opus） | K1、K3、K4 | 在临时拷贝上造改动、命令与日志攻改后的代码：`admission.py` 的子命令与 `--selftest`、54 号的场景自证（假 cargo）、hook 喂 JSON 看退出码；被判的重型命令一条都不执行 |
| 本地攻方 | K3 的事实表 | 按事实表逐格核：两条用例的登记字段、用例源码打得出哪些行（grep 原样）、54 号快档与 `--full` 对每个字段各判什么（引 `admission.py` 的判法行号）；每行写来源（命令与输出行），每张表不超过 6 行 |

两条攻方腿不重叠：Opus 造情形攻修补的语义与代价，本地只逐格核两条登记行的字面与用例打出的行。

## 五、交付

- 腿的报告写 `research/prompts/defs-gatebatch-m2-r2-<腿名>-output.md`，分段落盘（每次工具调用写进文件不超过 150 行，第一段排他新建）。
- 探针放 `research/prompts/defs-gatebatch-m2-r2-<腿名>-model/`，带 `SHA256SUMS` 与 `rerun.sh`。
- 引文件写那份文件自己的行号，引的行是那句原文自己所在的行。
- 不跑重型测试：54、55、57、59、87 号本身、`gate.sh`、名字带 layer0 的测试目标、全量 `cargo test` 一律不跑；只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`；等后台任务就结束本轮等通知；交回之前后台不许留着跑的东西。
