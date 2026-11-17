# m4-gpugc-floor-raise-precheck-code-r3：判决（2026-11-10）

正文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-body.md`，改动全文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-diff.md`，开工快照 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-snapshot/`。这一题的第三轮，也是最后一轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-sonnet-output.md` | 撤回干净、与第一轮被判那一版先算逐处相同；第二轮判决「撤回不丢出路」那一句的读数是装置没走到，不是没撞上；辩方探针的先算行全是式子那一判，聚簇段那一项从没被读；检查单八格成立 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-output.md`，模型 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/` | T1 撤回干净；T2-1 被遮的合取 7、T2-2 槽对上界每槽补一对的松打中；T2-3 条款两句互相顶；T2-4 开头走空那一句字面不成立；T3 出路读数都是没试；检查单八格成立 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 11975 退 0：「清单点名 2 份，倒推出 2 份，sha256 对上 2 份」（量的）。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-output.md research/prompts/m4-gpugc-floor-raise-precheck-code-r3-sonnet-output.md --root ~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-main-check/tree --background research/prompts/_m4-gpugc-floor-raise-precheck-code-r3-background.md` 退 0，末行「✓ 核了 102 处引文、24 处裸行号，对上 56 处，没判 70 处（其中裸行号没有引文可比 9 处；逐处列在上面）」，没有对不上的。T2-1 依据的几行主 agent 现读过：`crates/singlefs-core/src/transaction.rs` 第 13317 行落得下那一判排在第 13373 行合取 7 之前；`crates/singlefs-core/src/mounted_session.rs` 第 2093–2094 行注释「抬 F 把 defer 里的槽回收不改这个数，推了只多发几次空发布」；第 6015 行先算只认 `SpaceAdmissionRefused`。
2. 正推腿的两份推导（只读第二轮产物）主 agent 经服务交：任务 11972、11973 退 0（量的）。攻方第二轮探针：「含 'outlet-lost' 的 COUNT 键数=0（0 = 那一支一次都没跑到，不是跑了读成 0）」「stuck 格里试过卸载重挂的=610  其中写进=0  C 圈罩着的探测点数x每点上限=61x10=610」；辩方第二轮探针：`natural-new`、`warm-new`、`forms-new` 先算行全是 `FormulaAvailableAgainstDemand`（768、1030、8 行），槽对那一判 0 行；已有用例上「翻面（旧上界<需求<=新上界）的行=0」。
3. 攻方第三轮探针（主 agent 拷模型到 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r3-main-check/opus-model`，经它的包装交任务 11974，第 3 次退 0）：`name=exit_codes never=0 always=0 probe=0`；`opus-run/out/probe.out` sha256 前缀 `f6a8c64184eb8755`（量的），整行：

   `probe PROBE_RESULT configurations=372 cells=25886 c_published=0 c_white=19444 c_white_after_ceiling_no_outlet=6781 c_white_after_budget=0 c_white_after_raise_refused=0 c_white_stuck=4455 c_white_fired_after_a_series=7306 c_white_publish_refusal=902 c_white_conjunct7_short_before_the_series=11 c_white_first_slot_pairs=19370 c_white_first_formula=1 c_white_first_other=73 c_white_slot_pairs_bound_loose=17931 c_white_formula_unreachable_defer=1 o_white=20964 oracle_published_after_a_series=0 misrefused=0 precheck_saved_a_vain_series=1520 fired_first_with_versions_published=0 stuck_followed=285 stuck_remount_published=0 stuck_push_ring_admitted=0 lost_first_followed=177 lost_first_remount_published=0 lost_after_followed=201 lost_after_remount_published=0 arms_differ=9320 c_ring_holding_at_probe=59 queue_waiting_at_probe=0 panics=0`

   自证：关掉先算那一臂（`self-check-never.out` sha256 前缀 `8da7fd6a859c7e5a`）`c_white=7147 … arms_differ=0`，一律判放不下那一臂（`self-check-always.out` sha256 前缀 `954bd02570cf92c9`）`c_white=0 … precheck_saved_a_vain_series=204`；`name=self_checks arms_differ_when_off=0 fired_rows_when_off=0 oracle_when_always=0 misrefused_when_always=0`；`name=misrefusal_base_zero self_check_always_oracle_published_after_a_series=0`：误拒那一项的底是 0，`misrefused=0` 按装置走不到读（量的）。

## 二、判决

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 撤回干净（T1） | 两腿各自逐处比对：第一轮加的先算段全在，第 122 批那一截零残留；文档「九种」在 | **成立**，撤回之后的 `mounted_session.rs` 作数 |
| 第二轮判决「撤回不丢出路（这一份取样）」（T3） | 第一节第 2 条：出路那一支在第二轮两份探针里一次都没跑到 | **更正**：第二轮那一句是装置没走到，不是取样没撞上；撤回的依据只剩白推那一半（第二轮判决第一节第 3 条，`vain_series_by_the_inside_term=18105`），它站得住 |
| 被宽护卡住那条出路走不走得通 | 第三轮探针跟着出路卸载重挂：卡住的 0/285、头一次判出放不下的 0/177、推过一串之后判出的 0/201 写进；推环 0 | **这一份取样里出路一次都没让写写进**；两份探针都没造读故障的宽护（T3-6），出路要的那一形没量过。登欠账 |
| 槽对那一判「每个放得开的槽各补一对」太松（T2-2） | `c_white=19444`（25886 格），其中头一次被槽对那一判拒的 19370、判为上界松的 17931；先算挡下的白推 `precheck_saved_a_vain_series=1520` | **打中**（量的）：撤回之后的先算对槽对那一判挡得很少。最后一轮，不改代码，交用户：改法乙（按段增量维护 defer 里成对的计数）零轮 |
| 落得下那一判遮住合取 7（T2-1） | `c_white_publish_refusal=902`，其中推串之前合取 7 已不成立的 11 | **打中，份量小**（量的，11 格）；其余 891 格是推串自己把全空段推下去。交用户：改法甲零轮 |
| 式子那一判的 defer 含上限够不到的槽（T2-3） | `c_white_formula_unreachable_defer=1` | 条款许的（`.claude/kb/decisions/16-发布语义.md` 第 275 行「式子那一判的上界是可用加 defer 加隔离」）；同一行开头「推抬 F 放得出的全放出来也放不下就一串都不推」与它在这一形上互相顶。交用户 |
| 开头走空之后头一次就判出、盘上不是逐字节不变（T2-4） | `fired_first_with_versions_published=0`，可达要一次瞬时读写错，没造 | 字面不成立（读码）；条款与第 645 行文档补「这次调用开头没走空发成任何一步」。只改文字，登欠账 |
| 误拒 | `misrefused=0`，底是 0（第一节第 3 条） | 没量到，不算没打中 |

检查单：两腿八格都回「成立」；主 agent 核：撤回之后先算区间剩第一轮追过的两处 `expect`，没有新用例与新崩溃流，变异表里余下「抬 F 先算」那几行原文各命中一次（攻方腿 grep，正推腿另数点名的测试都在）。

| 格 | 判决 |
|---|---|
| 走不到的理由、类型与名字、不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程、带 --item 的命令与项、新崩溃流多罩了什么 | 成立 |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/mounted_session.rs` | **照旧**：撤回之后这一版作数；第三轮新冒出来的形态不在这一题里改 |

## 三、改法

这一题到此停，不开第四轮，代码不改。

1. 欠账（`.claude/kb/checks-owed.md`）登两行：
   - 先算对槽对那一判挡得少（T2-2）与被遮的合取 7（T2-1）：第三轮探针读数，改法乙、甲零轮；
   - 被宽护卡住那条出路没量过：读故障的宽护两份探针都没造，跟着出路卸载重挂的 663 次没有一次写进；连同 T2-4 那一句文字。
2. kb 不改条款：T2-3 两句互相顶、T2-4 补一句，都随用户定。

## 交用户的

| 条目 | 读数 | 标 |
|---|---|---|
| 先算对槽对那一判挡得少（T2-2） | 白推 19444 格里头一次被槽对那一判拒的 19370、上界松的 17931；先算挡下 1520 | 量的；改法乙零轮 |
| 被遮的合取 7（T2-1） | 11 格 | 量的；改法甲零轮 |
| 式子上界含够不到的 defer（T2-3） | 1 格 | 条款两句互相顶，要么认这一截白推，要么加按释放代的计数（改法丙零轮） |
| 开头走空那一句（T2-4） | 没造出 | 读码；补一句文字 |
| 被宽护卡住那条出路 | 跟着出路卸载重挂 663 次写进 0 次；读故障的宽护没造 | 第一轮采纳、第二轮撤回都以它为准，它自己没量过；要造宽护得另立一题 |

## 这一轮被判过的文件

- crates/singlefs-core/src/mounted_session.rs：判了撤回之后的先算判法，照旧作数

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 10 | 不影响 | 2026-11-10 不受影响：撤回之后代码与射程对齐；T2-3、T2-4 两处文字交用户 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-10 不受影响：被宽护卡住那一报照旧；出路走不走得通登欠账 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-10 不受影响：先算读它的式子，条款不动 |
