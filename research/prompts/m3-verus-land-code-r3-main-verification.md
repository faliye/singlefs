# Verus 核接回 `crates/`：代码轮第三轮（最终轮）判决（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 J1 J2 J3 J4 J5 S03 S04 S05 S06 S07 S12 K20 H06 H07 H08 H09 H11 A02 U1 U2 U3 U4 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / diff | `research/prompts/_m3-verus-land-code-r3-body.md`、`_m3-verus-land-code-r3-background.md`、`_m3-verus-land-code-r3-diff.md` | — |
| 开工快照 | `research/prompts/m3-verus-land-code-r3-snapshot/`：`crates-sha256.txt`（217 份，冻结副本 `/tmp/claude-1000/m3-verus-land-code-r3/tree/`）、`kb-sha256.txt` | 腿交齐之后冻结副本 `sha256sum -c` 全 OK。`kb-sha256.txt` 只有 `.claude/kb/checks-owed.md` 对不上：别的会话插行，两条腿一处都没引它。主工作区 `crates/` 里快照内三份文件被别的会话改过，改前改后在 `research/prompts/m3-verus-land-code-r3-snapshot/external-changes-m3-code-r3-L.patch`；三份都不是被判文件，腿读冻结副本 |
| 云端辩方（Sonnet） | `research/prompts/m3-verus-land-code-r3-sonnet-output.md`（275 行，sha256 386190b1…8a8f）；模型 `…-sonnet-model/` | 主 agent 在 `/tmp/claude-1000/m3-verus-land-code-r3-main-check/` 照模型 README，从冻结副本拷两份（今天的核 `r3core`、套上 `h52-restore.diff` 的 `restored`），`gen_old.sh` 生成 fcc4d340 那一版与 m1–m4，跑同一组八条差分（内存包装 6G、10 线程），日志 `logs/r3core-a.txt`、`logs/restored-a.txt` |
| 云端攻方（Opus） | `research/prompts/m3-verus-land-code-r3-opus-output.md`（273 行，sha256 b49261e7…85f9）；模型 `…-opus-model/` | 主 agent 现跑三件：H5-2 退回的核在 rlimit 10 与 20 上各跑一次 Verus；J3 那四行照攻方 `verus-rows.sh` 在 rlimit 20 上重跑；S07 照辩方 `j2/run-j2.sh` 套到新用例上 |
| 引文 | 两份腿报告 | `python3 scripts/agents/cite-check.py <两份报告> --root /tmp/claude-1000/m3-verus-land-code-r3/tree --root . --background research/prompts/_m3-verus-land-code-r3-background.md`：「✓ 核了 58 处引文，对上 58 处，没判 0 处」 |

主 agent 现跑的原样输出：

- 今天的核对 fcc4d340（`logs/r3core-a.txt`）：`RESULT[chain+named k=3] Tally { total: 47775744, diverged: 331776, … value_differs: 0, log_only: 331776, log_only_new_has_extra_reads: 331776 … }`；`RESULT[random] Tally { total: 3000000, diverged: 4498, … value_differs: 0, log_only: 4498 … }`；`CONTROL[m1 去掉 take(limit)] diverged=1632 / 40000`、`m2 … 3872`、`m3 … 550`、`m4 … 571`。
- H5-2 退回的核对 fcc4d340（`logs/restored-a.txt`）：`RESULT[chain+named k=3] Tally { total: 47775744, diverged: 0, … }`、`RESULT[random] Tally { total: 3000000, diverged: 0, … }`，结构族九行 `diverged: 0`；`CONTROL[m1 去掉 take(limit)] diverged=1581 / 40000`、`m2 … 3821`、`m3 … 545`、`m4 … 520`（与第二轮逐个相同）；越出 `requires` 的两族 `key-mismatch` `value_differs: 161443`、`max-counter` `value_differs: 5410` 与今天的核相同（第二轮已判契约收紧）。
- H5-2 退回的核 Verus（`verus-restored-rlimit10.log`、`…20.log`）：rlimit 10「error: while loop: Resource limit (rlimit) exceeded」「verification results:: 31 verified, 1 errors」；rlimit 20「verification results:: 32 verified, 0 errors」。
- J3 四行 rlimit 20（`j3-r20/summary.tsv`）：`rc=1	36	rlimit=20	verified=31	errors=1	rlimit_error_lines=0	proof_errors=1	expected=verified, 2 errors	verdict=GATE_RED_fragment_missing`，第 45、48、87 行同为 `GATE_RED_fragment_missing`；四行与攻方 `results/verus-table-rlimit20.tsv` 逐字相同。
- S07 套到新用例（`j2-out/summary.tsv`）：`2	GREEN	rc=0	passed=1	failed=	S07 末条提交标记那一判挪到反向链回调之前`，套上了（`APPLIED row=2`）。

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| J1 纯重构 | 差分对 fcc4d340 只差读盘日志，来源是 H5-2；退回 2 行全族归零 | 回调调用打中：375 815 052 个输入结局 0 差异；锚点读不出、链首序号不是 1 时改后先调反向链回调再断；改过的镜像上才多读盘（推的）；核第 5、1310 行注释不再属实 | **打中（回调调用），两条腿独立同向，主 agent 复跑坐实**（第一节）。结局、报告五字段、报错一处不差；H5-1 与 H5-3、H5-4、H5-8 没打中 |
| J2 新用例 | S03、S05、S06、S08、S09 红，S04、S07、S10–S12 绿；可写挂载走 `ReadStageCache` | S07 Verus 验过、新用例绿；H06–H09 同样漏；核头第 11 行比用例宽；可写挂载上读几次盘没有用例钉 | **打中，两条腿独立同向，主 agent 复跑 S07 坐实**。用例只罩住「锚点读得出、两次单条发布都施加」这一种形状 |
| J3 变异表 | 必红表 93 / 93 红；K20 改锚后仍红 | 第 36、45、48、87 行期望数含 rlimit 用尽，rlimit 20 上门禁判「没有期望的片段」；K20 改锚后是更宽的错（新写法单独打中 843 552 个输入、旧写法单独 0 个），点名用例两种都红；「跳过下一条」一类在 Verus 表里没有一行 | **打中（rlimit 期望），主 agent 复跑坐实**；K20 判轻（表没漏错，只是判的不是原来那一判）；删「下一号改成加二」照判据没打中 |
| J4 编码纪律 | H5-3 两条改法没同样打中：改名不解决「一个概念一个名字」与手抄映射，「一处定义」原型 4 份文件、Verus 32 / 0、测试过 | 打中 6 处：同一量两个名字（`lib.rs:23`）、胶水手抄映射（`recovery.rs:3061`）、核两句注释与代码不符、新用例期望值字面数（第 274、306 行）、两份测试手抄三常量两函数、`writable_mount_takes_a_second_instance.rs:88` 的 `+ 60` | **打中**（引文都核对上）。H5-3 那一笔第二轮的账没还清 |
| J5 前两轮判决 | H1 结局层站得住、「零差异」够不着（只差读盘日志）；H2、H3、H4、H6 站得住；H5 八处里 H5-2「用不到」够不着、H5-7 判轻（测试代码放宽） | 不归 | **采**：第二轮 H5-2「用不到的代码删掉」判错了——那一判不改结局，但决定回调调不调，不是死代码；H5-7 按 `code-discipline.md` 对测试代码放宽，判轻，改了不退。其余前两轮判决维持 |

两条腿提的改法都只修错与补测试，没有一条是为性能改 core。

## 三、交用户的改法（全部零轮）

最终轮，下面的改法都被攻过零轮，照 `.claude/rules/three-way-inference.md`「多轮」那一节不再为它们开一轮。

| # | 打中的格 | 候选 | 代价与依据 |
|---|---|---|---|
| U1 | J1（H5-2） | **甲**：退回 H5-2，补证明提示让 Verus 在 rlimit 10 上过；补不出来再把登记的 rlimit 调到 20，照 `.claude/gate.d/verus-pins.tsv` 的流程整张重验。**乙**：保留删除，把核第 5、1310 行注释改成写明这一处次序与原文不同 | 甲：差分归零、对照回到第二轮的数（主 agent 现跑）；rlimit 10 上只差一处循环 rlimit 用尽（现跑）。乙：结局不变，多出的回调只在被改过的镜像上读盘（攻方推的，没造镜像量） |
| U2 | J2 | **甲**：核头第 11 行收窄到用例罩住的那一种形状，补三格用例（锚点读不出、多条记录的发布、候选在提交标记或在飞上限上被断），S07、H07–H09 追加进 `crates/mutations.tsv` 并证红；可写挂载走缓存那一半另记欠账。**乙**：spec 加幽灵读盘日志 | 甲：攻方推的，没实现；乙：两条腿都说罩得更宽、看不到块层读，没有原型 |
| U3 | J4（H5-3） | **甲**：一处定义，core 直接用核的四个类型，固有方法挪进核 crate 的 `verus!` 之外，胶水的手抄映射随之去掉。**乙**：维持 `PrefixKernel…` 改名，`lib.rs:23` 那句注释改成实话，另记一笔欠账 | 甲：辩方原型 `research/prompts/m3-verus-land-code-r3-sonnet-model/h53/h53-prototype.diff`（4 份文件），它在副本上量 Verus「32 verified, 0 errors」、core `--lib` 139 passed、五个 harness 目标过；主 agent 没复跑 |
| U4 | J3 | **甲**：期望片段只数证明错，门禁 verus-must-red-worlds 格比之前剔掉 rlimit 那一个错误（改 `scripts/gate/verus-proof-check.py` 与表头约定）。**乙**：只改这四行的变异写法，让它们在 rlimit 10 上不撞 rlimit | 甲：一次修掉这一类，四行在 rlimit 20 上 proof_errors 都 ≥ 1（现跑）；乙：只修已知的四行 |

不用交用户、并进下一批实现员照做的（零轮，编码纪律与测试，不改行为）：J4 其余五处（新用例期望值由式子算出、三常量两函数挪进 `common`、`+ 60` 与 `content_of(3300, 17)` 起名、核第 5、1310 行注释随 U1 改）；K20 那一行照攻方 H11 改写成原来那一判（`&& later.counter.0 != record.counter.0 + 1`）；`verus-mutations.tsv` 补攻方 A02 一行（「跳过紧挨着的下一条」）。

## 四、两条腿之外主 agent 记的

- 攻方报整个 harness 包对照行红在两条 litmus 用例（`the_rotation_barrier_litmus_writer_follows_the_last_three_steps_of_every_recorded_publish`、`litmus_writer_threads_follow_the_recorded_publish_order`），推的原因是副本里缺 `crates/` 之外的文件；主工作区合入验证 101 个测试二进制全绿，这两条在主工作区是绿的，不归这一轮。
- 辩方报必红表 13 行输出含 rlimit 用尽，正文写的是「9 行改期望里有 4 行」，口径不同；主 agent 合入后逐行核是 12 行加一行重跑（调度记录第 5 批那一条），三个数各数的是不同的集合，不矛盾。

## 五、下一步

1. U1–U4 交用户定，定了并进第 6 批实现员（连同第三节「不用交用户」那几条）；U4 甲是门禁与脚本，交 tooling-writer。
2. 第三轮之后不再开轮：第 6 批落地之后照常走合入验证，提交时由崩溃验证员跑 checker-tier-formal-verification。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-01 不受影响：这一轮只判实现改法，条款没动 |

## 用户定案（2026-10-01，弹窗）

| # | 用户选 |
|---|---|
| U1 | 甲：退回 H5-2（先补证明提示让 rlimit 10 过，补不出再调登记的 rlimit 并整张重验） |
| U2 | 甲：核头注释收窄、补三格用例、S07 与 H07–H09 进变异表证红，可写挂载那一半记欠账 |
| U3 | 乙：维持 `PrefixKernel…` 改名，`lib.rs:23` 那句注释改成实话，另记一笔欠账 |
| U4 | 甲：期望片段只数证明错，门禁比之前剔掉 rlimit 那一个错误 |
