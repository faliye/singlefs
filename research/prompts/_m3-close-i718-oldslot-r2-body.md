# m3-close-i718-oldslot-r2：满环时根槽写失败、恢复之前的镜像上 I-7.18 该怎么判，第一轮之后的丁甲站不站得住（推论轮第二轮）正文（2026-10-23）

## 一、题面

第一轮判决 `research/prompts/m3-close-i718-oldslot-r1-main-verification.md`。被判的是：根环装满时一次发布的根槽写落在环里最旧那条根住的槽上、写成 0 或撕裂、进程在重发之前崩，恢复之前那张镜像上 I-7.18（根记录的回收门槛等于现算的门槛）判红；写者照 D16（发布语义） 已定项 1「写失败的槽按旧内容算」没错。这是欠账 C560 那一形。用户 2026-10-23 弹窗定「开一轮三方定（推荐）」，原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`。

第一轮：甲（按旧内容 N+1−3S 再现算一次）被「环洞落在下一槽」那段合法历史打穿（旧内容是 N+1−2·3S，合法镜像上红 36/36）；乙（这一态报不适用）被「挂载时钉住的槽正好轮到下一槽」那一形打穿（条文点名的两类坏写者全放过）；丙（认成已知红）两腿都不取（这一态上 I-7.18 与 I-3.1 没有共同绿的 t）。

这一轮攻第一轮判决第三节合成的**丁甲**（下面第三节照抄），丙留作对照。分工照 `.claude/rules/three-way-inference.md` 第二轮：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。

## 二、实现今天的样子（主 agent 的观测，2026-10-23，行号按主工作区）

- 写者：`crates/singlefs-core/src/allocator.rs` 第 1802 行 `reclaim_threshold_of_a_root_carrying` 取 `self.reclaim_floor.max(rollback_floor_of_the_root)`；`reclaim_floor` 跟着环里最旧有效根走（同文件第 1588 行 `oldest_valid_root_in_the_ring`、第 1756 行起）。
- 池级 checker：`crates/singlefs-checker/src/walk.rs` 第 8507 行起由 `deferred_slots_reclaim_threshold`（第 8366 行）按镜像上的环现算、判 `newest_root.reclaim_threshold == recomputed_threshold`（第一轮之后主工作区打进了 C478 补丁，改的是同一份文件里判码 2 头 key 区间的几处，I-7.18 那一判没动；行号以现查为准）。
- GPU 走读内核：`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 里 `judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold))`（C506、C478 两份补丁改过这一份文件别处，行号现查）。
- journal 记录：池级 checker 走读时有记录列表（`walk.rs` 里读 journal 环那一段，现查）；GPU 走读内核有没有现成的记录交接，第一轮正推没核。
- 不变量条文：`crates/singlefs-checker/invariants/02-根环.md` I-7.18、I-7.19 两行。
- 场景与造法：`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs`（plain、torn、zeroed）；环洞造法 `crates/singlefs-harness/tests/common_root_ring_holes/mod.rs` 第 207 行 `crash_a_mount_before_its_row_root`。
- 第一轮两腿的模型：`research/prompts/m3-close-i718-oldslot-r1-opus-model/`（攻方，含丁甲等开关、格 A、B、E、F 的探针）、`research/prompts/m3-close-i718-oldslot-r1-sonnet-model/`（正推，含甲含种子边与乙的 CPU、GPU 补丁、naga 校验）。只读，要改拷进自己这一轮的模型目录。

## 三、被判的形态（照抄第一轮判决第三节）

**丁甲**：最新根（txg N）之后那一槽读不出自证过的根，且 journal 里有点名 N+1 那次发布的记录（D16 已定项 7 的持久顺序：记录先于根槽落盘），且 N+1 ≥ 3S 时，I-7.18 判 t 等于下面几种现算之一就成立：

1. 今天的现算（照镜像上的环）；
2. 把那一槽按它在更早任一圈住过的根（txg ≡ N+1 (mod 3S)、小于 N+1）补进环之后的现算；
3. 被抛弃的根住在那一槽时的那一支（正推第 V1 格推的 N+2−3S）。

不满足前提（那一槽有自证过的根，或 journal 里没有点名 N+1 的记录，或 N+1 < 3S）照今天判。CPU 池级 checker（`crates/singlefs-checker/src/walk.rs`）与 GPU 走读内核（`crash_judge_checker.wgsl`）同改。

**丙**作退路留着，对照用。

## 四、格（照抄第一轮判决第三节「第二轮要答的格」）

| 格 | 问什么 |
|---|---|
| W1 | 丁甲第 2 种现算「任一圈住过的根」是不是只看镜像就定得出（判器看得到的只有 t、t ≡ N+1 (mod 3S) 与环上别的槽）；它比攻方量过的丁甲多放过哪些错 t、错 p；「p 写 255、N+1 在飞」与「环洞形 t 写 15」两格靠兄弟镜像兜住，在崩溃点重放的枚举里兄弟镜像是不是一定也被判 |
| W2 | 前提「journal 里有点名 N+1 的记录」：D16 已定项 7 的持久顺序是不是保证这一态上记录一定在；记录读坏、journal 环转过一圈之后这一前提是不是还判得出；判器读 journal 在 CPU、GPU 两侧各有没有现成的交接 |
| W3 | 从未被看过的形：正推列的「环没满时最旧那条根事后读坏」（三个候选都红）该不该绿、丁甲罩不罩；撕裂域里的邻槽；零单元发布与无文件版本上的复制根（I-7.18 条文里「不判」的那一类）与这一态叠在一起 |
| W4 | 产生结论的方法：两腿的装置各只取了 S = 4（环长 12）一种几何、手挑崩溃状态，没调 `enumerate_crash_states` 把写表整段展开；换几何、换历史（零单元发布、空发布）结论翻不翻 |

## 五、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | W1–W4 | 复核第一轮判决：替被判出局的甲（含种子边）、乙、丙辩护——第一轮那几击够不够得着、是不是同样打中丁甲；W4 换几何、换历史复核两腿的数 |
| 云端攻方（Opus） | W1–W4 | 找反例：丁甲放过的错 t / 错 p、前提判不出的镜像、丁甲在合法镜像上误红的历史、GPU 一侧做不到的地方。不重复第一轮攻过的角度（格 A、B、E、F 本身，甲的括注，乙的第一圈闸） |

方案按 `crates/` 今天的实现来谈。引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
