# m4-gpugc-floor-raise-precheck-code-r1：判决（2026-11-10）

正文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-body.md`，改动全文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-diff.md`，开工快照 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端正推（Sonnet） | `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-sonnet-output.md`，探针在 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-sonnet/probe-copy` | T1 一致；条款「盘上逐字节不变」与文档「八种」冲突；提交内生块那一判漏拦；上界漂移待探针 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-output.md`，模型 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-opus-model/` | T1-1 没打中（推的）；第四处推串没过先算；短缺混着来白推一串；槽对那一判吃掉宽护出路（推的） |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 11939 退 1：快照点名 6 份，5 份对上，被判的三份产品代码文件与此刻工作区那一份 sha256 相同；`.claude/kb/checks-owed.md` 倒推不出（工作区与暂存区同时被几个会话改过）。`python3 scripts/agents/cite-check.py … --root ~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-main-check/tree --background research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-background.md` 末行「✓ 核了 83 处引文、61 处裸行号，对上 43 处，没判 101 处（其中裸行号没有引文可比 40 处；逐处列在上面）」，没有对不上的；没判的在树外，判决依赖的几行主 agent 现读过。腿引 `checks-owed.md` 的行判决不依赖。
2. 攻方探针任务 11937 退 0（主 agent 拷模型到 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1-main-check/opus-model` 跑），`out/probe.out` sha256 前缀 `36a57c96a4c02252`（量的）：`PROBE_RESULT histories=1240 hits=0 unsound_formula=0 unsound_pairs=0 first_round_fired=12087 later_round_fired=0 panics=0`；两条自证都红：`self-check-bound-without-deferred … hits=1574`、`self-check-soundness-without-deferred … unsound_formula=1986 unsound_pairs=1694`；`name=self_checks_red hits_when_the_bound_omits_deferred=1574 unsound_when_the_check_omits_deferred=1986`。
3. 正推腿探针任务 11935 写判决时还在排队，它的 `drift` 读数没进判决；上界在推一串前后站不站得住由第 2 条的 `unsound_formula=0 unsound_pairs=0` 回答。

「没打中」只各抽一份样本，只当线索。

## 二、判决

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 上界算小、误拒推了本来写得进的写（T1-1） | `hits=0 unsound_formula=0 unsound_pairs=0`，1240 条历史，两条自证红 | **这一份取样没撞上**。一份样本不撑「没打中」，登欠账要再抽一份 |
| 槽对那一判吃掉被宽护卡住的出路（T3-4） | 读码：再挂时分配器重建、这次挂载开过的聚簇段清空（`crates/singlefs-core/src/allocator.rs` 的 `cluster_segments: BTreeSet::new()`），聚簇段里没挡的槽对卸载重挂放得出、不在上界里；可达性没量 | **采，改**：违主 agent 在正文定的射程「保住被宽护卡住那条出路」。宁大勿小：槽对那一判的上界加上这次挂载开过的聚簇段里没挡的槽对，判不出放不下的照推，推到上限照旧报被宽护卡住、列卸载重挂 |
| 第四处推串不过先算（T1-3） | 读码：改动之后另发的那批搬迁走 `publish_the_relocation_batch_raising_the_floor_for_space` | **不改**：搬迁批是这一次改动发成之后的修复，不是用户写；写进 D16 已定项 10 射程「先算罩三处，不罩另发的搬迁批」 |
| 提交内生块那一判照推（T1-4、正推腿第 2 点） | 读码 | **不改**：用户定的射程是「可用加 defer」，主 agent 定的是式子与槽对两种；写进射程 |
| 短缺混着来白推一串（T3-3） | `later_round_fired=0`，这一份取样没出现 | **不改**：写进射程，带「推过之后才判出的那一次盘上有这次调用写的东西」 |
| 条款「盘上逐字节不变」少限定 | `.claude/kb/decisions/16-发布语义.md` 第 275 行与 `crates/singlefs-core/src/mounted_session.rs` 第 631 行 | **改**：条款补「头一次被拒就判出时」 |
| 文档「八种」 | `crates/singlefs-core/src/mounted_session.rs` 第 2899 行，`UserChangeRefused` 现在九个成员 | **改** |
| `let … else` 把别的拒绝一律当「判不出需求」 | 读码 | 不改，低危，记欠账 |

检查单：

| 格 | 判决 |
|---|---|
| 走不到的理由 | 成立（两腿逐处追过两处 `expect`） |
| 类型与名字 | 成立；文档「八种」并入第三节第 2 条 |
| 不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、带 --item 的命令与项、新崩溃流多罩了什么 | 成立（两腿一致） |
| 测试优先多线程 | **不成立**：运行库两条崩溃循环用例没写为什么不并行，补一句 |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/mounted_session.rs` | **要改**：第三节第 1、2 条 |
| `crates/singlefs-core/src/compaction_plan.rs` | 照旧（只改文档） |
| `crates/singlefs-mount-runtime/src/lib.rs` | 照旧（只改文档） |

## 三、改法（下一批的条款）

1. 槽对那一判的上界加上这次挂载开过的聚簇段里没挡的槽对；配一条判法小用例钉「聚簇段里空着的槽对算进上界」，配变异行。
2. 文档「八种」改「九种」。
3. 运行库两条崩溃循环用例补一行不并行的理由。
4. kb：D16 已定项 10 射程「盘上逐字节不变」前补「头一次被拒就判出时」；射程写明先算不罩另发的搬迁批与提交内生块那一判、短缺混着来时推过之后才判出。

改完开第二轮，只攻第 1 条。

## 交用户的

- 第 1 条是攻方腿提的形态，被攻过零轮，第二轮攻；可达性（宽护要读坏的被抛弃根）没量。

## 这一轮被判过的文件

- crates/singlefs-core/src/mounted_session.rs：判了抬 F 之前先算的判法与新拒绝成员，要改（第三节第 1、2 条）
- crates/singlefs-core/src/compaction_plan.rs：判了余量算术那一句文档，照旧
- crates/singlefs-mount-runtime/src/lib.rs：判了事件文档，照旧

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 10 | 改了 | 2026-11-10 改了：射程补「头一次被拒就判出时」与先算不罩的三形 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-10 不受影响：宽护那条出路照旧，上界加聚簇段那一截就是为了保它 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-10 不受影响：先算读它的式子，条款不动 |
