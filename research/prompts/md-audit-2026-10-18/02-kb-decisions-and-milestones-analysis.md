# 全仓 Markdown 文档分析报告（二）：知识库决策、里程碑、布局与总表

> 本报告针对 `singlefs` 知识库中的全部 **60 篇** 核心决策 (28 篇)、里程碑总表 (17 篇)、盘上布局 (3 篇) 及总账总览文件 (12 篇) 进行逐篇过时、失真与断链分析。

---

## [.claude/kb/INDEX.md](file:///home/fy5090/code/singlefs/.claude/kb/INDEX.md)
- **文件规模**：40 行，4607 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/checks-owed.md](file:///home/fy5090/code/singlefs/.claude/kb/checks-owed.md)
- **文件规模**：854 行，1195495 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 22 行：引用的文件路径已不存在：`scripts/gate/stage-must-run.sh`
  - 第 22 行：引用的文件路径已不存在：`scripts/gate/change-touches-crates.sh`
  - 第 310 行：引用的文件路径已不存在：`research/prompts/_m2-code-r1-diff.md`
  - 第 394 行：引用的文件路径已不存在：`src/lib.rs`
  - 第 651 行：引用的文件路径已不存在：`scripts/gate.sh`
  - 第 651 行：引用的文件路径已不存在：`scripts/gate.sh`
  - 第 658 行：引用的文件路径已不存在：`scripts/shell-lint.sh`
  - 第 660 行：引用的文件路径已不存在：`scripts/relay-timing-lint.py`
  - 第 716 行：引用的文件路径已不存在：`scripts/rewrite-moved-paths.py`
  - 第 841 行：引用的文件路径已不存在：`scripts/gate/lib_source_digest.py`
  - 第 843 行：引用的文件路径已不存在：`scripts/kb/oov-check.py`
  - 第 844 行：引用的文件路径已不存在：`scripts/kb/oov-check.py`
  - 第 844 行：引用的文件路径已不存在：`scripts/crash/corruption-check.py`
  - 第 845 行：引用的文件路径已不存在：`scripts/crash/corruption-check.py`
  - 第 846 行：引用的文件路径已不存在：`scripts/crash/corruption-check.py`
  - 第 401 行：提及的欠账项 C113 在 checks-owed.md 中已还清/销账，但正文未同步：`| C614 | 回退时墓碑树取错与删完对象无墓碑单镜像判不了 | 两种镜像单镜像 checker 判不出：① 管理员回退那次发布把墓碑树错取成 R_old 那一版（墓碑树随现行那一版，D23（journal 的角色与格式） 已定项 14）——攻方 13 种改坏里这一种对单镜像 checker 的 18 条判据一条都不红；② 已删完的对象：删完之后它既不在 inode 树、也不在 `deleted_inodes`，单镜像上没有东西说它存在过，「已删完的对象都有墓碑」写不成单镜像不变量 | 跨根比（崩溃点重放的崩溃流或录制流）：① 回退那次发布之后墓碑树的记录多重集等于回退之前那一版的，改坏成取 R_old 必红；② 相邻两条根之间，前一根 inode 树里有、后一根里没有的 (号, 对象出生代)，后一根的墓碑树里要有一条在谱系上的墓碑，后一根是回退那次发布（它的谱系树里有 T_rb 等于它的 txg 的记录）时不判；改坏成意图那次不写墓碑必红（推的） | 墓碑树、谱系树有代码（三·2）；跨根比的判器；去向：里程碑三三·5（意图与墓碑类：跨根比判回退墓碑树取错与删完无墓碑）（五·11，2026-10-12 判）。 | 2026-10-01 `research/prompts/m3-delete-r3-main-verification.md` 第五节记欠账那一段；`research/prompts/m3-tomb-r1-main-verification.md` 第三节判 checker 那一格；与 C113（扫描重建时多版单元的现行版本判定无输入） 同组 |`
  - 第 814 行：提及的欠账项 C639 在 checks-owed.md 中已还清/销账，但正文未同步：`| C646 | 弄坏开关文件头点名的判错项与实测不等 | `.claude/gate.d/checker-tier-crash-point-replay.sh` 文件头 `GATE_CRASH_POINT_REPLAY_BREAK` 15 个开关、`.claude/hooks/lib_heavy_tests.py` 文件头 71 个 `LIB_HEAVY_TESTS_BREAK` 开关逐个打开实测，点名的判错样本与红格改到与实测逐项相等（层 0 的 `no-feature-pass`、`features-not-from-source`、`ignore-feature-pass-exit` 三处，重型闸库 7 处含 `selftest-pattern-words-judged` 三格）；实测表在 `records/2026-10-13-欠账还五条.md` 课题 23 指到的报告。往后由 C625（弄坏开关文件头三样没有门禁） 与 C639（门禁阶段的弄坏开关没有复跑方） 的格按「红的项与点名的相等」判 | 2026-10-13 |`
  - 第 249 行：提及的欠账项 C143 在 checks-owed.md 中已还清/销账，但正文未同步：`| C342 | 树 ID 水位在根读不出时退回去重发 | 树 ID 水位 = 根环里全部根记录该字段的 max（D8（核心索引结构） 已定项 8 ②）；被抛弃时间线里建的克隆头的树 ID 在那些根读不出时被重发：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 第三轮 f11 十条臂一样 155 条、最少 1 个故障（`c143r3_model.out` 里「name=f11_clones arm=A tree_id_reissued_runs=155」那一行起）。与 C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 同形，但 C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 三轮的丙（身份带对象出生代）救不了它——树的身份就是树 ID。只藏根与记录在取号之前被 C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的乙-配置续拒可写；要连系统配置一起退回（整卷回滚，D18（块里携带什么信息） 已定项 11 宣布不防）才重发；新线最新而旧线根还在环里的那一窗是推的、没造 | 多次挂载录制流：回退或挂载之后建的每棵树的 ID 与此前任何一个写成了根的树都不同；判别力自证：立账时的取法在「藏住建树那次发布的根」下必须红；会红的检查是 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同）：全部可读根（含被抛弃的）的树表里，同一树 ID 的诞生 txg 相同 | 载体已定：也取记录里的（用户 2026-09-23，D8（核心索引结构） 已定项 8 ②；实现 `crates/singlefs-core/src/recovery.rs` 的 `highest_tree_identifier_watermark_in_the_ring`），「怎么拦」点名的 I-9.28（全部可读根里同一树 ID 的诞生 txg 相同） 已实现（`crates/singlefs-checker/src/walk.rs` 的 `judge_tree_identifier_birth_txg_across_every_readable_root`）；仍欠的只是根与它那条记录都读不出那一格：镜像上没有证据可判；用户 2026-10-18 弹窗定这一格「保留为已知射程、欠账开着」 | C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 第一轮（H6）、第三轮攻方腿 2.3；用户 2026-09-23 定「也取记录里的」（D8（核心索引结构） 已定项 8 ②），实现随 C511（回退到无文件那一版之后诞生代怎么接） 第 3 步落地（`crates/singlefs-core/src/recovery.rs` 的 `highest_tree_identifier_watermark_in_the_ring`，用例 E 与一条变异钉着），待代码三方与崩溃验证；剩下根与它那条记录都读不出那一格盖不住 |`
  - 第 662 行：提及的欠账项 C146 在 checks-owed.md 中已还清/销账，但正文未同步：`| C369 | 提交内生块段耗尽时没有回落 | 2026-09-17 实现、2026-09-18 过代码三方第一轮（`research/prompts/m2-wave1-code-r1-main-verification.md`；回落那一格另起里程碑「覆盖写、释放、回退与复用」增补 2 收口表第 59、60 行）：`DeviceFreeMap::lowest_commit_generated_fallback_slot`，用例 `commit_generated_units_fall_back_when_the_segment_runs_out.rs` 四条，`crates/mutations.tsv` 里名字以「增补 2：」开头的九行在门禁 checker-tier-crates-mutation-replay 里全红。C146（无空段时的回落政策全仓无定义） 第 ② 条随之还清一半，第 ① ③ 条（整理）仍开着。依据：`m2-closeout-recheck-b.md`「逐项表」 | 2026-09-19 |`
  - 第 471 行：提及的欠账项 C723 在 checks-owed.md 中已还清/销账，但正文未同步：`| C717 | 回退下界上限的单窗口盲带 | I-7.9（回退下界 F 不高于抬 F 的上限） 的池级 checker 与 GPU 判器重算上限只取锚在镜像最新根上的那一圈窗口里的根（后来的镜像上洞里留下的旧根会把上限算低、在合法镜像上误红，理由写在 `judge_rollback_floor_raises_against_their_ceilings` 的文档注释里）；分配器抬 F 的上限取整环（`mount.rs` 的 `rollback_floor_ceiling_not_passing_ring_slots_bad_since_the_mount`）。F 越过环上洞里留着的一条有效根 X 时整环口径下违例，窗口口径下 checker 看不见：非空有效根不足 4 个、上限落到最旧有效根时，窗口口径取窗口里最旧的那条 W，整环口径取 X，F 落在 X 与 W 之间两边不同（推的，没造盘面）。钉 checker 这一侧的三条用例都是正向，没有一条钉「F 越过洞里旧根时 checker 不判」这一边；这一带只有实现一侧的用例与变异在盯 | 造一张镜像：环上一个洞里留着有效根 X，窗口里非空有效根不足 4 个，F 抬到 X 与 W 之间；整环口径判它违例，要有一种判法把它判红，洞里留着旧根而 F 没越过它的合法镜像仍绿；判别力：同一张镜像把 F 放回 X 以下，由红转绿（推的，没造盘面）。推翻这一行的观测：造出这样一张镜像，现行池级 checker 就判 I-7.9（回退下界 F 不高于抬 F 的上限） 红（那是推错了窗口去掉的集合） | 要有一种既看得见 X、又不在合法镜像的洞上误红的判法，判法没定（取窗口就是为了不在洞上误红）；成立的已知代价，随 `crates/singlefs-checker/invariants/02-根环.md` 的 I-7.9（回退下界 F 不高于抬 F 的上限） 一行写明盲带；去向：里程碑三二·4（回退下界上限的单窗口盲带，同 C660（按盘回退下界的定案那一句与装置对不上）、C723（读坏槽上限实验三处条款与代码读法不同））（五·11，2026-10-12 判）。 | `m3-batch10-code-r1-sonnet-output.md` K3-1；判决 `research/prompts/m3-batch10-code-r1-main-verification.md` 第二节 K3 与第五节第一行（记欠账、条款写明） |`
  - 第 166 行：提及的欠账项 C314 在 checks-owed.md 中已还清/销账，但正文未同步：`| C199 | 实例代号递增与 jsn 断号即止互相矛盾 | D23（journal 的角色与格式） 已定项 9 定「实例代号 32 位 + 计数器 48 位」且代号每次恢复递增，E37（日志实例代号） 逐字 `jsn = (代号 << 48) \| 计数器` ⇒ 一次实例切换让 `jsn` 跳 2⁴⁸；而 I-8.3（重放前缀严格连续） 与 D23（journal 的角色与格式） 已定项 14「前缀判定的完整口径五条」第一条都是「jsn 严格连续（断号即止）」（`jsn == expected`）⇒ 照字面，实例切换之后新实例的记录一条都不施加（E37（日志实例代号） `epoch` 臂实测 `lost_new=1`）。**矛盾已收口（2026-09-14 用户复核）**：前缀规则不改；D23（journal 的角色与格式） 已定项 14 补了链首、比较序（实例代号为主）、计数器（全池接着走）、切换时的 W（窄读）四条注，D16（发布语义） 已定项 7 补了新实例第一个根是单点的代价（D23（journal 的角色与格式） 已定项 9 与 14、D16（发布语义） 已定项 7 都是用户定案）；要不要暖机落 D16（发布语义） 已定项 8，建议先还 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）。两轮三方的结局：维持严格前缀（甲）被打中——每次可写挂载都取新代号（D18（块里携带什么信息） 已定项 11「只读挂载不取号」），干净重挂之后第一个根所在的槽读不出、再在第二次发布之前崩一次，fsync 已返回的事务就丢，一个故障就够（根槽不镜像）；按实例表跨边界（乙）按字面同样丢；交接记录（丙）与链尾锚（丁）都把一次没持久的回退接上，丁一个故障就丢已确认的回退（被抛弃尾巴点名的单元被复用、验不过即停）出局；严格前缀 + 新实例先暖机（戊）单故障没打中；三条臂都被 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 那一格打中 ⇒ 不替用户选臂，回到严格前缀、写明代价（`research/prompts/c199-r1-main-verification.md`、`research/prompts/c199-r2-main-verification.md`） | 崩溃点重放造两格：① 所选根是新实例的根 ⇒ 新实例在水位之后的记录全部施加；② 新实例的根全读不出、退到旧实例的根 ⇒ 新实例的记录一条都不施加、写出的行 W = 0（D16（发布语义） 已定项 7 那条代价）。判别力自证：把链首改成「环里第一条」或把跨实例改成放行，② 必须由绿转红 | I-8.3（重放前缀严格连续） 未实现；崩溃点重放 harness 已接入（门禁 checker-tier-crash-point-replay；崩溃点重放第二条流有三次可写挂载含回退，但没有「新实例的根全读不出」那一格的故障注入） | 2026-09-08 三方论证（D22（单元原子性怎么合成） 那条世代号硬要求要不要销账）的反推腿指出，主 agent 逐字核 I-8.3（重放前缀严格连续）、D23（journal 的角色与格式） 已定项 14 的五条口径、E37（日志实例代号） 正文坐实 |`
  - 第 22 行引用的 `scripts/gate/stage-must-run.sh`、`change-touches-crates.sh` 路径已过时。
  - 第 747 行（C627）仍提及旧格名 `pinned-red-examples-complete`。
  - 存在多条 10-11 批留存的占位句需复核证据闭环。
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions-history.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions-history.md)
- **文件规模**：15295 行，2608324 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 955 行：引用的文件路径已不存在：`research/prompts/m3-admission-r1-forks.md`
  - 第 984 行：引用的文件路径已不存在：`research/prompts/m3-retire-r1-sonnet-output.md`
  - 第 1399 行：引用的文件路径已不存在：`.claude/gate.d/60-stale-open-items.sh`
  - 第 1966 行：引用的文件路径已不存在：`.claude/gate.d/20-kb-shape.sh`
  - 第 2042 行：引用的文件路径已不存在：`research/prompts/m3-w3-r2-opus-output.md`
  - 第 2113 行：引用的文件路径已不存在：`.claude/gate.d/94-checker-implementation-disjoint.sh`
  - 第 2121 行：引用的文件路径已不存在：`.claude/gate.d/92-layout-checker-sync.sh`
  - 第 3466 行：引用的文件路径已不存在：`research/prompts/m3-retire-r1-sonnet-output.md`
  - 第 3474 行：引用的文件路径已不存在：`research/prompts/m3-leaf-death-forks.md`
  - 第 3474 行：引用的文件路径已不存在：`research/prompts/m3-3-5-forks.md`
  - 第 3487 行：引用的文件路径已不存在：`research/prompts/m3-delete-checker-r2-forks.md`
  - 第 4271 行：引用的文件路径已不存在：`.claude/gate.d/22-item-ref-status.sh`
  - 第 4792 行：引用的文件路径已不存在：`.claude/rules/kb-discipline.md`
  - 第 5384 行：引用的文件路径已不存在：`.claude/gate.d/30-decision-history.sh`
  - 第 6244 行：引用的文件路径已不存在：`crates/singlefs-replay-prefix/verus-mutations.tsv`
  - 第 6244 行：引用的文件路径已不存在：`crates/singlefs-replay-prefix/verus-mutations.tsv`
  - 第 6325 行：引用的文件路径已不存在：`.claude/singlefs-ai-sop/rules/test-script-discipline.md`
  - 第 6353 行：引用的文件路径已不存在：`scripts/harness-test-timing.py`
  - 第 6355 行：引用的文件路径已不存在：`crates/singlefs-harness/test-timing.tsv`
  - 第 6400 行：引用的文件路径已不存在：`crates/singlefs-harness/test-timing.tsv`
  - 第 6403 行：引用的文件路径已不存在：`crates/singlefs-harness/test-timing.tsv`
  - 第 7065 行：引用的文件路径已不存在：`.claude/gate.d/60-stale-open-items.sh`
  - 第 7565 行：引用的文件路径已不存在：`research/prompts/m3-batch10y-code-r1-opus-output.md`
  - 第 7579 行：引用的文件路径已不存在：`research/prompts/m3-batch10-code-r3-forks.md`
  - 第 7579 行：引用的文件路径已不存在：`research/prompts/m3-batch10i-code-r1-forks.md`
  - 第 7587 行：引用的文件路径已不存在：`research/prompts/m3-batch10o-code-r1-forks.md`
  - 第 7598 行：引用的文件路径已不存在：`research/prompts/m3-batch17-code-r1-forks.md`
  - 第 7637 行：引用的文件路径已不存在：`research/prompts/m3-batch16-code-r1-forks.md`
  - 第 7642 行：引用的文件路径已不存在：`research/prompts/m3-batch14-code-r1-forks.md`
  - 第 8040 行：引用的文件路径已不存在：`.claude/kb/milestone-first-txn.md`
  - 第 8620 行：引用的文件路径已不存在：`research/prompts/m3-batch49-w3-implementer-report.md`
  - 第 8644 行：引用的文件路径已不存在：`research/prompts/m3-batch10o-code-r1-forks.md`
  - 第 9838 行：引用的文件路径已不存在：`.claude/gate.d/24-status-redundancy.sh`
  - 第 10248 行：引用的文件路径已不存在：`.claude/gate.d/43-owed-table-shape.sh`
  - 第 10762 行：引用的文件路径已不存在：`.claude/gate.d/61-settled-same-file.sh`
  - 第 11078 行：引用的文件路径已不存在：`scripts/agents/oov-check.py`
  - 第 11254 行：引用的文件路径已不存在：`.claude/gate.d/31-blocking-verdict.sh`
  - 第 11470 行：引用的文件路径已不存在：`.claude/gate.d/20-kb-shape.sh`
  - 第 11472 行：引用的文件路径已不存在：`.claude/gate.d/lib-index-vs-body.py`
  - 第 12464 行：引用的文件路径已不存在：`.claude/gate.d/28-cross-decision-status.sh`
  - 第 12479 行：引用的文件路径已不存在：`.claude/gate.d/29-settled-item-self-open.sh`
  - 第 12923 行：引用的文件路径已不存在：`research/prompts/m3-3-5-forks.md`
  - 第 12930 行：引用的文件路径已不存在：`research/prompts/m3-c694-r2-forks.md`
  - 第 13002 行：引用的文件路径已不存在：`scripts/kb/fixtures/stale-candidates-benchmark-facts.tsv`
  - 第 13003 行：引用的文件路径已不存在：`scripts/kb/fixtures/stale-candidates-benchmark-facts.tsv`
  - 第 13384 行：引用的文件路径已不存在：`crates/singlefs-checker/tests/crash_enumeration_new_pool_file_creation_stream.rs`
  - 第 13385 行：引用的文件路径已不存在：`.claude/gate.d/checker-tier-layer0-replay.sh`
  - 第 13631 行：引用的文件路径已不存在：`.claude/gate.d/27-format-constants.sh`
  - 第 13773 行：引用的文件路径已不存在：`.claude/singlefs-ai-sop/rules/doc-discipline.md`
  - 第 14268 行：引用的文件路径已不存在：`research/prompts/_d26-item5-r4-background.md`
  - 第 14268 行：引用的文件路径已不存在：`research/prompts/_d26-item4-r1-background.md`
  - 第 14273 行：引用的文件路径已不存在：`research/prompts/_d26-item5-r4-background.md`
  - 第 14298 行：引用的文件路径已不存在：`research/prompts/_d26-item4-r1-background.md`
  - 第 15037 行：引用的文件路径已不存在：`research/prompts/m3-admission-r1-forks.md`
  - 第 15046 行：引用的文件路径已不存在：`research/prompts/m3-batch50-m3b-implementer-report.md`
  - 第 13303 行：提及的欠账项 C1 在 checks-owed.md 中已还清/销账，但正文未同步：`997: 甲下恢复期仍有「续做未完成意图」这条常规路径（[checks-owed.md](checks-owed.md) C1（意图续做做成常规路径））。`
  - 第 10524 行：提及的欠账项 C34 在 checks-owed.md 中已还清/销账，但正文未同步：`C34（明文映射层的态别三处打架） 三处各自开着。`
  - 第 13103 行：提及的欠账项 C547 在 checks-owed.md 中已还清/销账，但正文未同步：`- **改后**：已定项 8 射程改成「现行不含反向链的头是 307」（E49（反向链宽度 32 还是 64） 扫过的 78、84、86、93、95、99 六档没有 307，半句「不在扫过的档位里」照留）；已定项 17 改成「改新池新建文件的字节（w1、w4、t9 三条记录的这一字节与头部校验和 [46, 50)；w4、t9 两条的反向链 [91, 95) 也跟着变，E142（新池新建文件的干跑） 空白 G26 逐字节核过）」；已定项 14 在「这一版的失败处置」之后加「回退见证的实现取法」五条（删除规则、写满、前缀第五条怎么读见证、见证读哪几槽、落点），欠账加 C547（回退见证表的删除规则与写满没有条款）。`
  - 第 9180 行：提及的欠账项 C113 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仍开着**：码 3 那 6 字节同样买不到东西，改它要动 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3`
  - 第 6346 行：提及的欠账项 C635 在 checks-owed.md 中已还清/销账，但正文未同步：`- **结论**：定案里「全量档提交时不默认跑、不限时刻，由用户要求时跑，也算准入判据，但只能是异步准入判据」「提交前 = 快速档全绿（必要条件）；里程碑收尾、动了崩溃点重放或判器的那一批提交之前 = 全量档全绿（准入条件），由主 agent 弹窗请用户跑一趟全量」「全量档红了，处置不是回退，是「之后所有提交都带着一个未结的红」直到修掉」三句改成两句：「全量档提交时不默认跑、不限时刻，由用户要求时跑；提交前 = 快速档全绿（必要条件）」「里程碑收尾、动了崩溃点重放或判器的那一批提交之前，由主 agent 弹窗请用户跑一趟全量；跑红或用户不跑，提交照样可以进，记一笔未结的红或「这一阶段收尾没跑全量」，直到修掉或补跑」；另补三句：「全量」指 checker-tier-crash-point-replay `--full`（已定项 15 的全量）与崩溃放量全量（`scripts/crash/crash-amplification.sh` 不带参数）两样；「动了崩溃点重放或判器」按暂存改动碰到的路径判（`crates/singlefs-checker/`、`crates/singlefs-checker-tier/src/` 下 `crash*`、`crash_point_replay_progress`、`verdict_store`、`crash_identity`、`crash_judge_*`、`crates/singlefs-replay-prefix/`）；「上一次全量」由主 agent 写进当前里程碑记录一行（日期、两样各自的结论）。索引行跟着改；依据加一行三方判决。欠账表 C20（罕见布局无人跑） 怎么拦一栏补全量的两个所指与「上一次全量」的记法，新记 C635（上一次全量没有机器可读的标记）。改之前照字面做会错三处（「全量」两个所指、「动了崩溃点重放或判器」没有判法、「上一次全量」没人记），「准入条件」与「异步准入判据、红了不回退」在同一段里互相打架。`
  - 第 15252 行：提及的欠账项 C126 在 checks-owed.md 中已还清/销账，但正文未同步：`- **改前**：C126（切换预留的最坏量没有口径） 第一轮三方论证判「这一轮不还」，三截各有前置：① 链重写的式子有了；② 固定点重做那一项随 C284（施加一条记录在指针层上做什么没有定义） 的空间侧语义二选一；③ 落点撞 D3（空间分配） 已定项 4 的两条 ⚠️（六项都是已发布统计量、后两项不带设备维），交用户。D23（journal 的角色与格式） 已定项 14 逐字「「一次固定点重做的最坏量」今天没有可算的口径，立成欠账」。`
  - 第 3752 行：提及的欠账项 C143 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：回扫员可靠性验收的逐行判报告（`sweep-acceptance-2026-09-18-v3-judge-3.md`），主 agent 现查：D18（块里携带什么信息） 第 398 行「2026-09-16 用户定案（三轮三方）：不加第六个字段 `locality_id`，提议不收」，欠账表 C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 那一行写着同日定案。同一份报告点名的 D14（双轨（大小文件 / 持久临时）） 第 283 行说 D15（格式冻结政策） 的冻结分层列着 `locality_id`，那要看 D15（格式冻结政策） 正文，这一轮没改。`
  - 第 13005 行：提及的欠账项 C143 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：四行欠账自己的正文：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 写「水位仍会退回去的只剩崩溃恢复落到较旧的根那一形」，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 写「被抛弃的时间线今天只由崩溃恢复造出」，C542（取号前预演不读盘核，坏盘加旧根全被盖掉会烧号） 写「挂载时回退那一支……没了」，D16（发布语义） 已定项 8 的正文不再提回退；`records/2026-09-24-里程碑二收尾调度.md` 第三节第 ④ 条把改名排到第 6 步。`
  - 第 6317 行：提及的欠账项 C671 在 checks-owed.md 中已还清/销账，但正文未同步：`- **结论**：定案「本工程功能正确性的全部故事就是这三个 oracle」改成功能正确性不只押在三个 oracle 上（RefFS、M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）、记录核对器）；表 O3（独立规约执行器） 行「验不到什么」加四样（被投影六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值、一次发布内各写的先后）；射程加 O3（独立规约执行器） 判一个崩溃态的做法，E171（三oracle合谋测试） 那一句改成冻结树上的读数与限定（「这处缺陷在修」去掉：装置已补取号、暖机出错记红），末尾加「三条 oracle 都不判的两样」一段；依据第二条改成引 E171（三oracle合谋测试） 的存在性读数（三者全放行的行、只靠一条的行、O3（独立规约执行器） 在坏实现表上 0 红、手造坏镜像里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的）；欠里那条合谋测试的欠账改简称成 C407（三 oracle 合谋测试的判官没进门禁），加 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）、C671（用户数据落点取值算不算 layout 登记字节没说清）、C672（交回值或内存状态错而写流不变的一类）、C673（三 oracle 合谋测试的补 oracle 改法没做）。实验页「影响的决策」D13（验证路线） 已定项 6 那一行由备料改支撑。`
  - 第 8075 行：提及的欠账项 C220 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：用户定案；欠账行 C123（用户定案被改后无人复核）、C298（头宽加 4 改动了用户定案里的数）、C155（跨实验同量无交叉断言）、C304（实例表链指针记录装不下 83 宽的指针）、C157（树表容量在两处按不同条目宽算）、C241（树表归谁，两条用户定案说反话）、C220（等价类仍是 2 与降级为待证互相欠着）、C276（权威态分界那段逐字节对照里两个数过期）。`
  - 第 15291 行：提及的欠账项 C220 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：用户定案；欠账行 C123（用户定案被改后无人复核）、C298（头宽加 4 改动了用户定案里的数）、C155（跨实验同量无交叉断言）、C304（实例表链指针记录装不下 83 宽的指针）、C157（树表容量在两处按不同条目宽算）、C241（树表归谁，两条用户定案说反话）、C220（等价类仍是 2 与降级为待证互相欠着）、C276（权威态分界那段逐字节对照里两个数过期）。`
  - 第 11925 行：提及的欠账项 C245 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：D23（journal 的角色与格式） 已定项 3、D18（块里携带什么信息） 已定项 14、D19（块指针的结构与宽度预算） 已定项 5 / 6 的原文；欠账行 C245（系统配置槽的写频率，两条条款说反话）、C68（不变量白名单与已定条款打架）、C266（livelist 条目的身份用了会过期的位置提示）。`
  - 第 13025 行：提及的欠账项 C245 在 checks-owed.md 中已还清/销账，但正文未同步：`- 改前：射程末句「⚠️ 取系统配置形态时用的代价表（E23（journal 几何） 的「两条臂差 21 : 500」）建在「tail 按 checkpoint 推进、比 fsync 稀疏得多」上，按 D16（发布语义） 已定项 6 与已定项 7 的注这个前提不成立；按新频率重算之后取系统配置形态的依据还站不站得住，以及 tail 还承不承重，是 C245（系统配置槽的写频率，两条条款说反话） 题面上的三方论证（`research/prompts/c245-r2-main-verification.md`）。」；欠「C245（系统配置槽的写频率，两条条款说反话）；C28（环几何不变量未实现）；C77（重放起点未定义）。」`
  - 第 3048 行：提及的欠账项 C260 在 checks-owed.md 中已还清/销账，但正文未同步：`- **新欠账**：C260（livelist 条目的预付没算进解开自己那个量）。`
  - 第 11925 行：提及的欠账项 C266 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：D23（journal 的角色与格式） 已定项 3、D18（块里携带什么信息） 已定项 14、D19（块指针的结构与宽度预算） 已定项 5 / 6 的原文；欠账行 C245（系统配置槽的写频率，两条条款说反话）、C68（不变量白名单与已定条款打架）、C266（livelist 条目的身份用了会过期的位置提示）。`
  - 第 13005 行：提及的欠账项 C314 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：四行欠账自己的正文：C143（inode 号水位在崩溃恢复落到旧根后退回去重发） 写「水位仍会退回去的只剩崩溃恢复落到较旧的根那一形」，C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 写「被抛弃的时间线今天只由崩溃恢复造出」，C542（取号前预演不读盘核，坏盘加旧根全被盖掉会烧号） 写「挂载时回退那一支……没了」，D16（发布语义） 已定项 8 的正文不再提回退；`records/2026-09-24-里程碑二收尾调度.md` 第三节第 ④ 条把改名排到第 6 步。`
  - 第 14485 行：提及的欠账项 C314 在 checks-owed.md 中已还清/销账，但正文未同步：`- **改前**：（快查·改前逐字：D26（后台整理与放置回收） 已定项 4 等用户定案；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 的修法未选；FUA 算不算段边界与系统配置槽写算不算发布序列的一步没有条款；D3（空间分配） 已定项 8 的卡点 1 写着候选乙被用户定案挡着；D14（双轨（大小文件 / 持久临时）） 已定项 2 的顺序与系统配置那一支没有重申。）`
  - 第 11556 行：提及的欠账项 C403 在 checks-owed.md 中已还清/销账，但正文未同步：`- **依据**：欠账表 C79（系统配置与 journal 环的放置没人定）、C403（引三方论证而没有腿况记录） 两行现文（C403（引三方论证而没有腿况记录） 写「2026-10-15 完成实现，待提交」）。`
  - 第 6317 行：提及的欠账项 C407 在 checks-owed.md 中已还清/销账，但正文未同步：`- **结论**：定案「本工程功能正确性的全部故事就是这三个 oracle」改成功能正确性不只押在三个 oracle 上（RefFS、M2–M4 落在 Verus 与 E165（恢复重放有界穷举与变异杀伤界）、记录核对器）；表 O3（独立规约执行器） 行「验不到什么」加四样（被投影六样的取值与六样之外的根记录字段、实现自己的恢复与读者代码、系统配置槽的见证值、一次发布内各写的先后）；射程加 O3（独立规约执行器） 判一个崩溃态的做法，E171（三oracle合谋测试） 那一句改成冻结树上的读数与限定（「这处缺陷在修」去掉：装置已补取号、暖机出错记红），末尾加「三条 oracle 都不判的两样」一段；依据第二条改成引 E171（三oracle合谋测试） 的存在性读数（三者全放行的行、只靠一条的行、O3（独立规约执行器） 在坏实现表上 0 红、手造坏镜像里 O2（独立解析器 + checker） 放行而 O3（独立规约执行器） 判不在的）；欠里那条合谋测试的欠账改简称成 C407（三 oracle 合谋测试的判官没进门禁），加 C670（无读故障的崩溃态上可写挂载见证判据为真没有判官）、C671（用户数据落点取值算不算 layout 登记字节没说清）、C672（交回值或内存状态错而写流不变的一类）、C673（三 oracle 合谋测试的补 oracle 改法没做）。实验页「影响的决策」D13（验证路线） 已定项 6 那一行由备料改支撑。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions.md)
- **文件规模**：479 行，43419 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/experiments-history.md](file:///home/fy5090/code/singlefs/.claude/kb/experiments-history.md)
- **文件规模**：5688 行，705528 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 22 行：引用的文件路径已不存在：`.claude/kb/decisions-history/2026-08.md`
  - 第 381 行：引用的文件路径已不存在：`.claude/gate.d/80-absolute-assertions.sh`
  - 第 585 行：引用的文件路径已不存在：`research/prompts/e16-r6-prereg.md`
  - 第 589 行：引用的文件路径已不存在：`research/prompts/e16-r6-prereg.md`
  - 第 661 行：引用的文件路径已不存在：`.claude/gate.d/28-cross-decision-status.sh`
  - 第 1010 行：引用的文件路径已不存在：`.claude/kb/decisions-history/2026-08.md`
  - 第 1029 行：引用的文件路径已不存在：`.claude/gate.d/80-absolute-assertions.sh`
  - 第 1179 行：引用的文件路径已不存在：`.claude/gate.d/85-repro-command.sh`
  - 第 1186 行：引用的文件路径已不存在：`.claude/gate.d/86-experiment-orphans.sh`
  - 第 1421 行：引用的文件路径已不存在：`.claude/gate.d/40-results-cited.sh`
  - 第 1501 行：引用的文件路径已不存在：`research/prompts/e43-r2-prereg.md`
  - 第 1504 行：引用的文件路径已不存在：`research/prompts/e43-r2-prereg.md`
  - 第 1505 行：引用的文件路径已不存在：`research/prompts/e43-r2-prereg.md`
  - 第 1505 行：引用的文件路径已不存在：`research/prompts/m3-r1-batch9-reruns-questions.md`
  - 第 1515 行：引用的文件路径已不存在：`.claude/gate.d/27-format-constants.sh`
  - 第 1549 行：引用的文件路径已不存在：`.claude/gate.d/27-format-constants.sh`
  - 第 1855 行：引用的文件路径已不存在：`research/prompts/e49-r2-prereg.md`
  - 第 1858 行：引用的文件路径已不存在：`research/prompts/e49-r2-prereg.md`
  - 第 1859 行：引用的文件路径已不存在：`research/prompts/e49-r2-prereg.md`
  - 第 1859 行：引用的文件路径已不存在：`research/prompts/m3-r1-batch9-reruns-questions.md`
  - 第 2340 行：引用的文件路径已不存在：`.claude/gate.d/33-mutation-tables.sh`
  - 第 2376 行：引用的文件路径已不存在：`.claude/gate.d/34-experiment-index-sync.sh`
  - 第 3451 行：引用的文件路径已不存在：`.claude/gate.d/27-format-constants.sh`
  - 第 3789 行：引用的文件路径已不存在：`research/prompts/e116-r2-prereg.md`
  - 第 3793 行：引用的文件路径已不存在：`research/prompts/e116-r2-prereg.md`
  - 第 4471 行：引用的文件路径已不存在：`research/prompts/e139-r2-prereg.md`
  - 第 4471 行：引用的文件路径已不存在：`research/prompts/m3-r1-e139-r2-questions.md`
  - 第 4479 行：引用的文件路径已不存在：`research/prompts/e139-r1-prereg.md`
  - 第 4565 行：引用的文件路径已不存在：`research/prompts/e142-r25-prereg.md`
  - 第 4571 行：引用的文件路径已不存在：`research/prompts/e142-r24-prereg.md`
  - 第 4572 行：引用的文件路径已不存在：`research/prompts/e142-r24-prereg.md`
  - 第 4575 行：引用的文件路径已不存在：`research/prompts/e142-r23-questions.md`
  - 第 4577 行：引用的文件路径已不存在：`research/prompts/e142-r23-prereg.md`
  - 第 4584 行：引用的文件路径已不存在：`research/prompts/e142-r22-questions.md`
  - 第 4590 行：引用的文件路径已不存在：`research/prompts/e142-r22-prereg.md`
  - 第 4597 行：引用的文件路径已不存在：`research/prompts/e142-r21-prereg.md`
  - 第 4601 行：引用的文件路径已不存在：`research/prompts/e142-r21-prereg.md`
  - 第 4631 行：引用的文件路径已不存在：`research/prompts/e142-r13-prereg.md`
  - 第 5002 行：引用的文件路径已不存在：`research/prompts/e156-r6-prereg.md`
  - 第 5009 行：引用的文件路径已不存在：`research/prompts/e156-r6-prereg.md`
  - 第 5009 行：引用的文件路径已不存在：`research/results/e156-alloc-basis-counts-2026-10-05-r6-vs-r5-compare.txt`
  - 第 5009 行：引用的文件路径已不存在：`research/results/e156-alloc-basis-counts-2026-10-05-r6-point-sets.txt`
  - 第 5014 行：引用的文件路径已不存在：`research/prompts/e156-r5-prereg.md`
  - 第 5014 行：引用的文件路径已不存在：`research/results/e156-alloc-basis-counts-2026-10-05-q2-normalise-compare.txt`
  - 第 5014 行：引用的文件路径已不存在：`research/results/e156-alloc-basis-counts-2026-10-05-mutations-replacements.tsv`
  - 第 5014 行：引用的文件路径已不存在：`research/results/e156-alloc-basis-counts-2026-10-05-mutations-append.tsv`
  - 第 5046 行：引用的文件路径已不存在：`research/prompts/e157-r2-prereg.md`
  - 第 5050 行：引用的文件路径已不存在：`research/prompts/e157-r2-prereg.md`
  - 第 5219 行：引用的文件路径已不存在：`research/results/e161-crash-state-dedup-and-time-split-feasibility-r2-judge-2026-10-05.py`
  - 第 5304 行：引用的文件路径已不存在：`research/prompts/e165-r6-prereg.md`
  - 第 5304 行：引用的文件路径已不存在：`research/prompts/m3-formal-e165-r6-questions.md`
  - 第 5454 行：引用的文件路径已不存在：`research/prompts/e171-r1-s3-segment-lengths-questions.md`
  - 第 5527 行：引用的文件路径已不存在：`research/prompts/e172-preregistration.md`
  - 第 5533 行：引用的文件路径已不存在：`research/prompts/e172-r2-questions.md`
  - 第 5536 行：引用的文件路径已不存在：`research/prompts/e172-r2-prereg.md`
  - 第 5574 行：引用的文件路径已不存在：`research/prompts/m3-batch10z-a-code-r1-forks.md`
  - 第 5586 行：引用的文件路径已不存在：`crates/singlefs-harness/tests/e175_product_path_inode_watermark_regression.rs`
  - 第 5586 行：引用的文件路径已不存在：`research/prompts/e175-preregistration.md`
  - 第 5607 行：引用的文件路径已不存在：`research/prompts/e176-preregistration.md`
  - 第 5615 行：引用的文件路径已不存在：`research/prompts/m3-3-5-forks.md`
  - 第 5627 行：引用的文件路径已不存在：`research/prompts/e177-preregistration.md`
  - 第 5650 行：引用的文件路径已不存在：`research/prompts/m3-leaf-death-forks.md`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/experiments.md](file:///home/fy5090/code/singlefs/.claude/kb/experiments.md)
- **文件规模**：205 行，115417 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/feature-bits.md](file:///home/fy5090/code/singlefs/.claude/kb/feature-bits.md)
- **文件规模**：22 行，3334 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/freeze-layer-membership.md](file:///home/fy5090/code/singlefs/.claude/kb/freeze-layer-membership.md)
- **文件规模**：118 行，24393 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/pending-rules.md](file:///home/fy5090/code/singlefs/.claude/kb/pending-rules.md)
- **文件规模**：89 行，8117 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/pitfalls.md](file:///home/fy5090/code/singlefs/.claude/kb/pitfalls.md)
- **文件规模**：41 行，4444 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/prior-art.md](file:///home/fy5090/code/singlefs/.claude/kb/prior-art.md)
- **文件规模**：929 行，118593 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/term-renames.md](file:///home/fy5090/code/singlefs/.claude/kb/term-renames.md)
- **文件规模**：340 行，52788 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 333 行：引用的文件路径已不存在：`crates/singlefs-core/src/superblock.rs`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/tooling.md](file:///home/fy5090/code/singlefs/.claude/kb/tooling.md)
- **文件规模**：218 行，20106 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/translation-glossary.md](file:///home/fy5090/code/singlefs/.claude/kb/translation-glossary.md)
- **文件规模**：134 行，12536 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/verification-build.md](file:///home/fy5090/code/singlefs/.claude/kb/verification-build.md)
- **文件规模**：307 行，58936 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 268 行：引用的文件路径已不存在：`tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`
  - 第 268 行：引用的文件路径已不存在：`tests/crash_facts_gpu_verifier_agrees_with_the_cpu_reference.rs`
  - 第 69 行：提及的欠账项 C146 在 checks-owed.md 中已还清/销账，但正文未同步：`**第一版就要带的运行时计数与准入自证**（2026-09-13 随三笔欠账登记；四行里三行已有实现——取号序计数 `transaction::count_key_order_mismatches`（在 `crates/singlefs-core`、新池新建文件上恒 0）、准入读数对分配器的槽数（C318（影子账隔离的单元没进准入不等式） 2026-09-28 还清，`crates/singlefs-core/src/admission.rs` 的 `available_on_each_device`，用例在 `crates/singlefs-harness/tests/admission_formula_on_the_mounted_state.rs`）、每次发布的写调用数与写字节（最后一行）——整理落点那一行还没有（整理还没进代码，C146（无空段时的回落政策全仓无定义） ① 仍开着）；模型里先做成会红的形态，实现照抄口径）：`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/vm-harness.md](file:///home/fy5090/code/singlefs/.claude/kb/vm-harness.md)
- **文件规模**：173 行，17752 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 105 行：引用的文件路径已不存在：`src/lib.rs`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/01-数据可移动性-反向索引.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/01-数据可移动性-反向索引.md)
- **文件规模**：106 行，16043 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/02-RAID条带策略.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/02-RAID条带策略.md)
- **文件规模**：366 行，57886 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 315 行：提及的欠账项 C568 在 checks-owed.md 中已还清/销账，但正文未同步：`**射程**：`io_min` = 4096、`physical_block_size` = 512 的普通 NVMe 过得了 (a)（4096 ≤ 4096），而每一次根槽写与系统配置槽写仍然是 512 字节 < `io_min` ⇒ 逐字违反已定项 16，这是**知情接受**，现状与三条候选记在 C212（固定结构的写小于 io_min），一条都还没选。取整数倍而不是取 `max` 的理由：`io_min` = 3072 这种非 2 的幂的几何上，两槽会共享一个映射单元（三方第一轮反推腿构造出来的），槽距取整数倍之后任何 ≤ 槽距的槽宽都不共享。序号到偏移的换算、根环基址与区域起点在 D22（单元原子性怎么合成） 已定项 16 第 2 句。`crates/` 写出了系统配置里那两个字段与槽距算式（`crates/singlefs-core/src/system_configuration.rs` 的 `SystemImmutableSizes::slot_spacing_for`，单测钉住「io_min = 3072 时槽距 6144」），挂载时拿实际探测的 `io_min` 与记录的槽距比对那一步还没实现（只有调用方 mkfs 参数与盘上记录的相等比对）。`io_min` 大到固定结构互相重叠的设备 mkfs 拒、不支持（用户定案不接受，原话在变更史）：S = 8 时槽距 384 KiB 收、512 KiB 起拒（根环区域 0 与区域 1 重叠），`io_min` = 1 MiB 时系统配置槽 1 与根环区域 0 同从 1 MiB 起、S 取多少都拒；依据是三方判决 `research/prompts/m3-admission-r1-main-verification.md` 第三节 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷） 那一行，欠账 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/03-空间分配.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/03-空间分配.md)
- **文件规模**：335 行，57116 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/04-校验和位置.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/04-校验和位置.md)
- **文件规模**：140 行，18980 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/05-快照-空间记账机制.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/05-快照-空间记账机制.md)
- **文件规模**：464 行，66683 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/06-快照实现模型.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/06-快照实现模型.md)
- **文件规模**：197 行，30369 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/07-是否进Linux主线.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/07-是否进Linux主线.md)
- **文件规模**：27 行，2031 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/08-核心索引结构.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/08-核心索引结构.md)
- **文件规模**：509 行，125744 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 108 行：提及的欠账项 C1 在 checks-owed.md 中已还清/销账，但正文未同步：`**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。**做法（bcachefs 已验证的形态）**：凡是**可能修改无界数量 extent** 的操作（truncate、collapse、删大目录），**先把意图写进 logged_ops 树，再分批推进**；崩溃后恢复时续做未完成的意图。删除中的 inode 单独进 `deleted_inodes` 树，由后台回收——用户的 `rm` 返回，不等于所有 extent 都已释放。第一版 unlink 不往 logged_ops 树写条目：意图就是 inode 记录 nlink = 0 加 `deleted_inodes` 那一条（已定项 16）。logged_ops 树在第一个文件版本那次与 `deleted_inodes` 一起零根注册，字段表、回退时取哪一版、态别留到第一个写者（整理，D26（后台整理与放置回收） 已定项 2）落地时定（C364（整理意图记录没有字段表））。意图分两类：**承诺类**（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空；**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载；续做入口按树分、共用发布路径。已定项 5 第 2 条与三态表、C1（意图续做做成常规路径） 的字面按承诺类写，整理落地时按类分开写（C621（整理落地时要动的删除条款没有清单外的检查））。`
  - 第 108 行：提及的欠账项 C364 在 checks-owed.md 中已还清/销账，但正文未同步：`**定案**：删大目录、截断大文件会产生海量墓碑，塞进一个事务必然爆掉。**做法（bcachefs 已验证的形态）**：凡是**可能修改无界数量 extent** 的操作（truncate、collapse、删大目录），**先把意图写进 logged_ops 树，再分批推进**；崩溃后恢复时续做未完成的意图。删除中的 inode 单独进 `deleted_inodes` 树，由后台回收——用户的 `rm` 返回，不等于所有 extent 都已释放。第一版 unlink 不往 logged_ops 树写条目：意图就是 inode 记录 nlink = 0 加 `deleted_inodes` 那一条（已定项 16）。logged_ops 树在第一个文件版本那次与 `deleted_inodes` 一起零根注册，字段表、回退时取哪一版、态别留到第一个写者（整理，D26（后台整理与放置回收） 已定项 2）落地时定（C364（整理意图记录没有字段表））。意图分两类：**承诺类**（unlink；truncate、删大目录）必须续做到底，可写挂载交回之前走到空；**提示类**（整理）可暂停、可在停机谓词下提前收、交给盘那条线上不续做，不挡挂载；续做入口按树分、共用发布路径。已定项 5 第 2 条与三态表、C1（意图续做做成常规路径） 的字面按承诺类写，整理落地时按类分开写（C621（整理落地时要动的删除条款没有清单外的检查））。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/09-加密.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/09-加密.md)
- **文件规模**：371 行，50923 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 218 行：提及的欠账项 C34 在 checks-owed.md 中已还清/销账，但正文未同步：`**射程**：撤回前六轮三方论证的记录（两条候选各塌在哪、明文层的键取物理、候选丁与候选戊、让渡区 A 的三处互斥、泄漏面推断链）只留在变更史与 `records/2026-09-06-D9项9三轮对抗.md`，不是现行设计。撤回前用户给过条件定案「如果物理键的性价比和效率高的话就定物理键，反之定逻辑键」，E108（明文层的代价交叉点） 把那个条件量成了数；那一轮的数留档，不再有消费者。E57（明文侧字段集的权威落点） 的「按字段拆」仍然成立，只是物理地址那一行的权威从明文映射层变成加密结构内的位置条目（D19（块指针的结构与宽度预算） 已定项 4）。C34（明文映射层的态别三处打架） 曾被三处当阻塞点名——D19（块指针的结构与宽度预算）、D21（权威态与派生态的分界） 的「引用者清单」那一半、D26（后台整理与放置回收） 已定项 5 的位置权威三选一——它们各自还剩什么挡着，已在欠账表「被解锁的三处没复核」一行逐处复核：三处复核齐了，仍欠的是门禁那一半。`
  - 第 218 行将已闭环的 C34 表述为仍欠门禁，需同步销账状态。
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/10-随机写-碎片化的真答案.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/10-随机写-碎片化的真答案.md)
- **文件规模**：58 行，6574 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/11-索引节点要不要留消息缓冲区.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/11-索引节点要不要留消息缓冲区.md)
- **文件规模**：122 行，14569 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/12-目标介质.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/12-目标介质.md)
- **文件规模**：138 行，27628 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/13-验证路线.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/13-验证路线.md)
- **文件规模**：448 行，97910 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 195 行：提及的欠账项 C407 在 checks-owed.md 中已还清/销账，但正文未同步：`**欠**：C445（模型对拍的取样点不随写路径增长）；C446（未实现的不变量没有前置登记，也没有到期）；C447（规约执行器没接进崩溃点重放与 RefFS 比对）；C407（三 oracle 合谋测试的判官没进门禁）：E171（三oracle合谋测试） 已量过一次，没有一格门禁读它的产物；C672（交回值或内存状态错而写流不变的一类）；C679（卸载记号没进模型对拍）`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/14-双轨大小文件-持久临时.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/14-双轨大小文件-持久临时.md)
- **文件规模**：152 行，23946 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/15-格式冻结政策.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/15-格式冻结政策.md)
- **文件规模**：235 行，33665 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 224 行：引用的文件路径已不存在：`.claude/kb/golden-images.toml`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/16-发布语义.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/16-发布语义.md)
- **文件规模**：380 行，95930 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/17-实现分层与第三方管道.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/17-实现分层与第三方管道.md)
- **文件规模**：161 行，23954 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/18-块里携带什么信息.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/18-块里携带什么信息.md)
- **文件规模**：608 行，147372 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 366 行：提及的欠账项 C113 在 checks-owed.md 中已还清/销账，但正文未同步：`**射程**：同一条依据对码 3 逐字成立，但改它要动 C113（扫描重建时多版单元的现行版本判定无输入） 定案 P3 与 I-1.8（归并后版本全序） 的码 3 全序键；改键（码 3 择新键退回二元）没采用，第一版的输入端已由总审核定案解掉（D19（块指针的结构与宽度预算） 已定项 12：一个实例在一个 checkpoint 里每个码 2 节点与码 3 容器只写出一次，第一版禁边插边 COW），以后放开边插边 COW 时改键这一步跟着重开；只欠会红的检查，记在 C163（固定点各轮写序相同使平局破除落空）。**预留只有 2 字节，C102（索引节点头没有预留区，加字段不是 compat） 只还得了一半**：C102（索引节点头没有预留区，加字段不是 compat） 自陈 flag 位「装不下字段」不算预留区，2 字节同样装不下大多数字段——它买到的是「有一个预留区」这件事本身，不是「以后加什么都 compat」。**空间不是依据**：砍 6 字节在码 2、码 3 两处都买不到空间（码 3 每容器记录数零变化，码 2 扇出至多多 1 格、树高全不变），定案取的是**把买不到东西的字节换成有登记欠账在等的字节**，不是省空间。两条被打回的判据不许再用：「今天零消费者 ⇒ 举证责任在留着那边」（回扫会一并砍掉已定项 7 的 nonce 与 MAC 预留位、D19（块指针的结构与宽度预算） 已定项 3 的 MAC 与 nonce 恒定占位、D8（核心索引结构） 已定项 8 的预留 32 字节；成立的是另一条：预留位的举证是「将来要用」，事务号的举证是「现在就买不到」，两者不同形）；「记账树里有不能由遍历重算的量 ⇒ 码 2 需要择版本」（多版记账行在**条目级**合并，不在容器级择版本——D5（快照 / 空间记账机制） 已定项 2 逐字「非单调统计量「后者胜」（后者由 seq 定），单调统计量取最大值」）。E73（节点 key 区间的扇出代价） / E97（记账与分配记录的条目编码） / E98（inode 记录与 inode 树的几何） / E103（inode 打包路 ① 的更新代价） 按旧三档头宽算的数归 C89（单元头收口的遗留重算） 第 ④ 笔与 C288（映射 key 定案的连带改动没落到别的决策） 第 ⑦ 条重算。`
  - 第 583 行：提及的欠账项 C113 在 checks-owed.md 中已还清/销账，但正文未同步：`**射程**：第一版单头、无快照、无清扫。扫盘重建怎么读墓碑与谱系记录不在这里定，记欠账（K-W5b，与 C113（扫描重建时多版单元的现行版本判定无输入） 同组）。谱系树的载体、打包记录类型 5 被攻过零轮；记录多带 H_cur 的 24 字节形态被攻过一轮（W3 第三轮两次抽样）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/19-块指针的结构与宽度预算.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/19-块指针的结构与宽度预算.md)
- **文件规模**：322 行，44640 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/20-承重面单元的原子性与自包含.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/20-承重面单元的原子性与自包含.md)
- **文件规模**：178 行，19719 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/21-权威态与派生态的分界.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/21-权威态与派生态的分界.md)
- **文件规模**：429 行，53604 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/22-单元原子性怎么合成.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/22-单元原子性怎么合成.md)
- **文件规模**：548 行，90909 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/23-journal的角色与格式.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/23-journal的角色与格式.md)
- **文件规模**：691 行，137580 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 57 行：提及的欠账项 C1 在 checks-owed.md 中已还清/销账，但正文未同步：`- **甲下 redo 为空**，不是「甲下恢复无动作」：恢复期仍有「续做未完成意图」这条常规路径（[checks-owed.md](../checks-owed.md) C1（意图续做做成常规路径））。`
  - 第 428 行：提及的欠账项 C113 在 checks-owed.md 中已还清/销账，但正文未同步：`- E104（扫描重建的现行版本判定）：新根取 T_old + 1 时压不过被抛弃的根——注 2、注 3 新实例的第一次发布从环里最大 + 1 起靠它；它的另一半「不写回退行时被回退抛弃的单元在全规则臂上复活」仍承重——设计就是不写回退行（墓碑第一轮判决 `research/prompts/m3-tomb-r1-main-verification.md` 第二节、`research/prompts/m3-delete-r3-main-verification.md` 第三节「墓碑第一轮「一轮」各行」）；扫盘重建认「挂载根是一次回退」靠谱系记录，读侧怎么用记欠账（K-W5b，与 C113（扫描重建时多版单元的现行版本判定无输入） 同组）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/24-后台重活能不能卸给GPU.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/24-后台重活能不能卸给GPU.md)
- **文件规模**：98 行，11932 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/25-目标负载优先级.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/25-目标负载优先级.md)
- **文件规模**：166 行，15765 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/26-后台整理与放置回收.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/26-后台整理与放置回收.md)
- **文件规模**：302 行，42784 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 67、69 行：仍包含旧常量 `LOGGED_OPERATIONS_KEY_BYTES` 与 `LOGGED_OPERATIONS_ENTRY_BYTES` 的描述。
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/decisions/27-小数据打包容器.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/27-小数据打包容器.md)
- **文件规模**：288 行，31967 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/decisions/28-挂载期承诺量.md](file:///home/fy5090/code/singlefs/.claude/kb/decisions/28-挂载期承诺量.md)
- **文件规模**：143 行，30482 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/01-first-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/01-first-txn.md)
- **文件规模**：237 行，54404 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 217 行：引用的文件路径已不存在：`src/crash.rs`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/02-second-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/02-second-txn.md)
- **文件规模**：613 行，309690 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 68 行：引用的文件路径已不存在：`tests/crash_enumeration_fixed_script_stream.rs`
  - 第 427 行：引用的文件路径已不存在：`tests/bad_disk_input_campaign.rs`
  - 第 364 行：提及的欠账项 C116 在 checks-owed.md 中已还清/销账，但正文未同步：`| 26 | checker：认「非空持久有效根」与 F 抬过上限要红的检查（定义已定，D16（发布语义） 已定项 1 表格后那段 ⚠️）；预想清单里没接的 I-1.2（块头写序已发布）、I-1.8（归并后版本全序）、I-3.4（可用空间扣待删占用）、I-4.2（无被引用未提交块）、I-4.3（提交原子）、I-5.3（报出的空闲都兑现得了）、I-7.3（环健康性）、I-8.1（环几何够大）、I-8.2（记录头不跨原子单元）、I-8.3（重放前缀严格连续）、I-8.4（重放幂等）、I-8.5（tail 槽有效集非空）、I-8.6（反向链算法）；记录核对器不认实例表行（跟 ①；回退行 2026-09-26 随管理员回退改成向前发布而删）；I-7.4（近 K 代块未被复用） 只靠校验和对不上判复用；被抛弃根引用的单元不在 checker 候选集里；I-7.2（最新根可完整遍历） 的不适用分支是死码；checker 的 `walk_root` 在树表单元已访问过时整条根返回、那条根的中央映射树不走（步 6 现状），先判可不可达 | 没实现 | 非空根那条与块头写序、版本全序、无被引用未提交块、提交原子、环健康性那五条不等；journal 那六条等（journal 形态可能变）；I-5.3（报出的空闲都兑现得了） 跟第 5 行 | **2026-09-22 现查 `crates/singlefs-checker/invariants/` 的状态列，项列那张清单已经腐烂**：I-1.8（归并后版本全序）、I-7.3（环健康性）、I-8.6（反向链算法） 三条 2026-09-21 都已实现（池级 checker `walk::check_pool_image`，坏镜像在 `crates/singlefs-harness/tests/checker_known_bad_images.rs`，崩溃点重放每个崩溃状态都判），I-7.2（最新根可完整遍历） 2026-09-14 已实现 ⇒ 别按项列那张清单重做。**真正还欠、而且不等的只剩三条**：I-1.2（块头写序已发布）、I-4.2（无被引用未提交块）、I-4.3（提交原子），**2026-09-22 交回**：I-1.2（块头写序已发布） 与 I-4.2（无被引用未提交块） 接进池级 checker（`walk::check_pool_image` 的 `judge_birth_identity_of_a_referenced_unit`，两份坏镜像各只红自己那一条、由 `the_birth_identity_bad_images_redden_only_their_own_invariant` 钉住，三条变异抓到 3 / 无效 0 / 没红 0，`check.sh` 绿）；**I-4.3（提交原子） 判不了，留在「未实现」、没写恒真的检查**——「部分生效」要拿「这次事务该有的全部效果」当 oracle 才分得出来，而一份镜像上任何状态都自洽，池级 checker 手上没有第二个输入，判它的是崩溃点重放加 oracle（形态同 C116（叶容器的分裂 / 合并纪律没有会失败的检查））。⚠️ **I-1.2（块头写序已发布） 的射程被收成「遍历方向」那一半**：扫描方向（「任一单元」都按谓词判、i > 挂载根实例就判损坏）对着一份崩溃镜像判不了——判不成已发布本身不是违例，而「i > 挂载根实例」只在恢复刚发布完新根那一刻成立，上一次挂载取了号、写出单元、一个根都没发布就崩是合法的，那一支由 I-7.7（系统配置实例代号不低于根环） ① 兜着。两条理由已写进 `crates/singlefs-checker/invariants/01-块自描述.md` 那一行。journal 那几条里 I-8.1（环几何够大） 与 I-8.5（tail 槽有效集非空） 2026-10-02 已实现，I-8.2（记录头不跨原子单元）、I-8.3（重放前缀严格连续）、I-8.4（重放幂等） 仍未实现；I-3.4（可用空间扣待删占用） 2026-10-05 已实现，I-5.3（报出的空闲都兑现得了） 仍未实现、跟第 5 行。**非空根那条**已立成 I-7.9（回退下界 F 不高于抬 F 的上限）（2026-09-24 已实现：checker `judge_rollback_floor_raises_against_their_ceilings`，坏镜像 `raising_the_floor_above_its_ceiling_reddens_only_the_rollback_floor_invariant`，`crates/mutations.tsv` 6 条变异，见 `crates/singlefs-checker/invariants/02-根环.md` 的 I-7.9（回退下界 F 不高于抬 F 的上限） 一行） | 步 5、步 6 |`
  - 第 574 行：提及的欠账项 C116 在 checks-owed.md 中已还清/销账，但正文未同步：`**现状（2026-09-22 落地）**：写路径与 checker 那两条都做出来了。`crates/singlefs-core/src/inode_tree.rs` 算「这次发布之后树里是哪几片容器、各装哪些记录、哪几片要重写」（不发写、不动分配器，10 条单测）；`crates/singlefs-harness/tests/many_inodes_write_and_read.rs` 4 条用例建 233 个 inode 一次发布、冷启动逐个读回、字段等于写入，记账里 inode 号水位 = 235；I-9.6（水位大于两处最大号） 与 I-9.12（分隔 key 落在孩子区间之外） 在 checker 里实现、各有坏镜像；左半容器身份不变由期望态带着分裂前的身份钉住；`crates/mutations.tsv` 末尾追加 12 条，12 条全红。C116（叶容器的分裂 / 合并纪律没有会失败的检查） 的分裂那三条已还；合并与空容器两条现在都有对象（`crates/singlefs-core/src/inode_tree.rs` 的 `remove_record_from_leaf_containers`：删空整片不落盘，左吸收右与左从右借都做）。**还没做的两样**：崩溃点重放小负载（一次发布让容器分裂一次，归提交时的主 agent）；十万个文件那一档（今天只跑到跨过 233 的一次分裂）。「inode 号复用已发过的号 ⇒ 判红」那条变异已做：C479（inode 号复用已发过的号今天造不出来） 的只供测试的开关 `InodeNumberWatermarkPressedBackForTests` 把水位压回，三条用例在 `crates/singlefs-harness/tests/inode_number_watermark_pressed_back_reissues_a_number.rs`，`crates/mutations.tsv` 里名字带这笔欠账编号的变异靶它们。**实做时定下的四处**：建 N 个 inode 算一个事务、一条 journal 记录（D16（发布语义） 已定项 5 的切分上界只管用户数据单元，建 inode 一个都不写）；inode 树根的 key 区间取分隔 key 的首尾、不取子树范围（跟 checker 今天的 `check_index_node_keys` 走，口径的账在 C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间））；inode 记录偏移 48 的 `blocks` 改成结构体字段、文件版本填 64、没写过数据的填 0（口径的账在 C480（inode 记录的 blocks 怎么算全仓没有条款））；没有目录时新建 inode 的 mode / uid / gid / nlink 仍写死 `0o100644` / 0 / 0 / 1。**一次发布只重写被改到的那几片叶容器**今天只有用例钉着，没有条款也没有不变量。`
  - 第 592 行：提及的欠账项 C116 在 checks-owed.md 中已还清/销账，但正文未同步：`**会碰到的决策点**：D8（核心索引结构） 已定项 6；D5（快照 / 空间记账机制） 已定项 4；C116（叶容器的分裂 / 合并纪律没有会失败的检查）；I-9.6（水位大于两处最大号） 与 I-9.12（分隔 key 落在孩子区间之外） 原写「未实现，前置：树的种类」，2026-09-22 都已实现；C100（inode 容器丢失是永久损失而没有任何冗余条款覆盖它）；目录进不进这个里程碑（交用户定）；出生序号的作用域：D19（块指针的结构与宽度预算） 已定项 9 逐字「同一棵树内每写出一个码 2 或码 3 单元加 1（两类共用一个计数）；作用域换到下一个 checkpoint 时清零」，而 2026-09-16 建档时 `transaction` 模块的 `BirthSequenceAllocator` 在每次 `publish_file_version` 调用里重建（今天已无此名，入口是 `publish_first_file` 与 `publish_overwrite`），一次调用 = 一次发布 = 一个文件时两者重合；多个对象在同一个 checkpoint 里各调一次就会在同一个 (树, txg, 实例) 键上从 0 重数、撞出重复的映射 key ⇒ 开工前把它挪到「一次 checkpoint」的作用域里跨调用共享（三方代码第一轮正推腿）。`
  - 第 194 行：提及的欠账项 C143 在 checks-owed.md 中已还清/销账，但正文未同步：`**会碰到的决策点**：D23（journal 的角色与格式） 已定项 14（管理员回退那一段与表格五格）；D16（发布语义） 已定项 1（回退候选集、F_生效）、已定项 7（持久顺序与确认时点）；D5（快照 / 空间记账机制） 已定项 4（记账统计量现算）；C493（回退候选集条文与实现说反话）（`RollbackCandidateExclusion` 搬进挂着时回退）；C511（回退到无文件那一版之后诞生代怎么接）（挂着时回退到树表 0 条那一版已不会发生：候选集要带文件；留下的是崩溃恢复落到暖机根再发第一个文件版本那一形）；C143（inode 号水位在崩溃恢复落到旧根后退回去重发）（管理员回退不再重发，崩溃恢复那一形仍开着）；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（影子账的用例换成由崩溃恢复造被抛弃的根）；崩溃点重放 oracle 与固定脚本里的回退换成向前发布（步 6）。`
  - 第 194 行：提及的欠账项 C314 在 checks-owed.md 中已还清/销账，但正文未同步：`**会碰到的决策点**：D23（journal 的角色与格式） 已定项 14（管理员回退那一段与表格五格）；D16（发布语义） 已定项 1（回退候选集、F_生效）、已定项 7（持久顺序与确认时点）；D5（快照 / 空间记账机制） 已定项 4（记账统计量现算）；C493（回退候选集条文与实现说反话）（`RollbackCandidateExclusion` 搬进挂着时回退）；C511（回退到无文件那一版之后诞生代怎么接）（挂着时回退到树表 0 条那一版已不会发生：候选集要带文件；留下的是崩溃恢复落到暖机根再发第一个文件版本那一形）；C143（inode 号水位在崩溃恢复落到旧根后退回去重发）（管理员回退不再重发，崩溃恢复那一形仍开着）；C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元）（影子账的用例换成由崩溃恢复造被抛弃的根）；崩溃点重放 oracle 与固定脚本里的回退换成向前发布（步 6）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/03-third-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/03-third-txn.md)
- **文件规模**：239 行，134059 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 204 行：引用的文件路径已不存在：`.claude/gate.d/e165-void-red-example-ledgers.tsv`
  - 第 230 行：引用的文件路径已不存在：`src/main.rs`
  - 门禁格名使用了过时的 `pinned-red-examples-complete`（现已重构为 `pinned-red-examples-table`）
  - 第 122 行：提及的欠账项 C568 在 checks-owed.md 中已还清/销账，但正文未同步：`| 关，准入条款簇（三方 `m3-admission-r1` 2026-10-12 已判决：`research/prompts/m3-admission-r1-main-verification.md`，岔路单 `m3-admission-r1-forks.md` 11 条；岔路 F1（需求的单位）与 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）那一条用户已定，其余下轮在入库装置上重量后并进实现批） | C356（第九项按统计量之差算，净释放时为负且没有钳位）、C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C546（抬 F 被拒时扣住的槽不退回）（剩两格：扣住位怎么放；判决「扣住的槽」那一行打中：写错重发后 F 已生效而扣住位不放，假性 ENOSPC，怎么放交用户）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）（同 C546（抬 F 被拒时扣住的槽不退回） 的判决）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）（前置改「无」；判决「段内关着的空槽」那一行：两腿一致，段外槽对读数今天已有，条款还缺，交用户）、C573（固定点上界没算有普通分配那次发布多出的删插）（判决「固定点上界」那一行打中：固定点上界会被超出，交用户）、C574（落得下那一判不核两槽对齐与各盘同槽）、C576（三条用例被准入先拒连带红，改法待选）（判决：行改成按变异名登记，第 149 行漂移、第 12 行不红）、C582（挂载盘容量下界没条款该取哪个） | C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判） 的前置已改「无」，C576（三条用例被准入先拒连带红，改法待选） 已改按变异名登记，其余各行的前置列仍是「条款没定 / 改法待选 / 要先定单位」，同落在 D28（挂载期承诺量） 与 D3（空间分配） 的准入式子上；C370（需求、可用与 df 没有共同单位） 的需求单位用户已定「盘字节、每盘相同」（弹窗定案，原话即选项标签，在 `records/2026-10-12-里程碑三工具与治理批.md`「用户弹窗定案」那一条），写回 D3（空间分配） 已定项 12、D28（挂载期承诺量） 已定项 1 各一句，乘盘数的变异已做成会红用例（`crates/mutations.tsv`「F1（m3-admission-r1）：一次发布的需求乘了盘数」，2026-10-14 随代码欠账批合入）；`df` 的写法仍交用户（判决「`df` 的写法」那一行：照 D16（发布语义） 已定项 1 字面 73/73 多报） |`
  - 第 130 行：提及的欠账项 C568 在 checks-owed.md 中已还清/销账，但正文未同步：`| 关，实现（条款已定：不接受，D2（RAID 条带策略） 已定项 19 射程写明 mkfs 拒到哪；补拒绝用例已落，`core_review_geometry_back_chain_and_empty_inode.rs`） | C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷） | 用户 2026-09-30 定「不接受 但是要推一下」，2026-10-12 推完（判决「io_min 1 MiB」那一行）、用户弹窗定「不接受，写进射程」，写进 D2（RAID 条带策略） 已定项 19 射程：`io_min` 大到固定结构重叠的设备 mkfs 拒、不支持（S = 8 时槽距 384 KiB 收、512 KiB 起拒，`io_min` = 1 MiB 任何 S 都拒）；补拒的用例已落（2026-10-14 随代码欠账批合入）；与 C582（挂载盘容量下界没条款该取哪个） 同属几何下界 |`
  - 第 133 行：提及的欠账项 C568 在 checks-owed.md 中已还清/销账，但正文未同步：`- **开工前要先定的**：五·1 的每一行去向 2026-09-30 已判完（「五·1 逐行去向」表），当时要用户拍板的只剩「交用户」那一行（C489（singlefs-core 的广谱变异对今天的代码没有结论））；之后准入条款簇的岔路单 11 条里除 F1 与 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷） 用户已定之外还开着，其中写「交用户」的有 C546（抬 F 被拒时扣住的槽不退回） 与 C570（抬 F 回收扣住的槽计数上空闲但分配器不发） 的扣住位怎么放、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判） 的条款、C573（固定点上界没算有普通分配那次发布多出的删插） 的固定点上界被超出、`df` 的写法，C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 ① 单盘择根与 ② 水位之上账读不出（岔路单 `m3-c554-c572-r1-forks.md`）也在等用户。`
  - 第 122 行：提及的欠账项 C570 在 checks-owed.md 中已还清/销账，但正文未同步：`| 关，准入条款簇（三方 `m3-admission-r1` 2026-10-12 已判决：`research/prompts/m3-admission-r1-main-verification.md`，岔路单 `m3-admission-r1-forks.md` 11 条；岔路 F1（需求的单位）与 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）那一条用户已定，其余下轮在入库装置上重量后并进实现批） | C356（第九项按统计量之差算，净释放时为负且没有钳位）、C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C546（抬 F 被拒时扣住的槽不退回）（剩两格：扣住位怎么放；判决「扣住的槽」那一行打中：写错重发后 F 已生效而扣住位不放，假性 ENOSPC，怎么放交用户）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）（同 C546（抬 F 被拒时扣住的槽不退回） 的判决）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）（前置改「无」；判决「段内关着的空槽」那一行：两腿一致，段外槽对读数今天已有，条款还缺，交用户）、C573（固定点上界没算有普通分配那次发布多出的删插）（判决「固定点上界」那一行打中：固定点上界会被超出，交用户）、C574（落得下那一判不核两槽对齐与各盘同槽）、C576（三条用例被准入先拒连带红，改法待选）（判决：行改成按变异名登记，第 149 行漂移、第 12 行不红）、C582（挂载盘容量下界没条款该取哪个） | C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判） 的前置已改「无」，C576（三条用例被准入先拒连带红，改法待选） 已改按变异名登记，其余各行的前置列仍是「条款没定 / 改法待选 / 要先定单位」，同落在 D28（挂载期承诺量） 与 D3（空间分配） 的准入式子上；C370（需求、可用与 df 没有共同单位） 的需求单位用户已定「盘字节、每盘相同」（弹窗定案，原话即选项标签，在 `records/2026-10-12-里程碑三工具与治理批.md`「用户弹窗定案」那一条），写回 D3（空间分配） 已定项 12、D28（挂载期承诺量） 已定项 1 各一句，乘盘数的变异已做成会红用例（`crates/mutations.tsv`「F1（m3-admission-r1）：一次发布的需求乘了盘数」，2026-10-14 随代码欠账批合入）；`df` 的写法仍交用户（判决「`df` 的写法」那一行：照 D16（发布语义） 已定项 1 字面 73/73 多报） |`
  - 第 133 行：提及的欠账项 C570 在 checks-owed.md 中已还清/销账，但正文未同步：`- **开工前要先定的**：五·1 的每一行去向 2026-09-30 已判完（「五·1 逐行去向」表），当时要用户拍板的只剩「交用户」那一行（C489（singlefs-core 的广谱变异对今天的代码没有结论））；之后准入条款簇的岔路单 11 条里除 F1 与 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷） 用户已定之外还开着，其中写「交用户」的有 C546（抬 F 被拒时扣住的槽不退回） 与 C570（抬 F 回收扣住的槽计数上空闲但分配器不发） 的扣住位怎么放、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判） 的条款、C573（固定点上界没算有普通分配那次发布多出的删插） 的固定点上界被超出、`df` 的写法，C554（崩溃恢复抛弃的根暂时读不出时影子账算不到） 的 ① 单盘择根与 ② 水位之上账读不出（岔路单 `m3-c554-c572-r1-forks.md`）也在等用户。`
  - 第 100 行：提及的欠账项 C122 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：清单 2026-09-28 列完（`records/2026-09-28-里程碑三第十项清单.md`）：欠账表开着的 479 行逐行终判，里程碑二留下的 144 行里 45 行「第十项自留」（就是这一项自留）、38 行疑似已还（核实后核销 30 行、留 22 行部分已还）、22 行挪到里程碑三之外、其余归项一至项四（各项作业点表里已点名；清单里归崩溃放量装置的随「增补 4」走，不在这份文件里）；不是里程碑二留下但与里程碑三相关的 50 行里归项的已写进各项，单列的 C30（链式保留池量化未验）、C70（跨统计量的和没人盯）、C122（分配记录不点删之后的条目数口径）、C191（同源性计数器只盖了一对）、C284（施加一条记录在指针层上做什么没有定义）、C397（反向判据的 N 没声明，事件数没人计） 在这一项判。用户 2026-09-28 定照终判写进各项。`
  - 第 148 行：提及的欠账项 C635 在 checks-owed.md 中已还清/销账，但正文未同步：`| 六·5 | 归这一项的其余欠账：C635（上一次全量没有机器可读的标记）（原 C20（罕见布局无人跑） 2026-10-11 并入）、C447（规约执行器没接进崩溃点重放与 RefFS 比对）；次要归这一项的 C476（盘上内容可控时走得到的 panic 有 21 处）、C541（原样重发把一次失败放大成整条发布流阻塞）、C558（回退目标是环里最旧的根时回退那次发布会写坏它） | 各自的登记行 | 各自登记行写的会红检查；开工日逐条判「这个里程碑关 / 留 / 不做」 | 无 |`
  - 第 112 行：提及的欠账项 C143 在 checks-owed.md 中已还清/销账，但正文未同步：`| 五·8 | 欠账改名留在 `crates/*/src/` 的注释：`crates/singlefs-checker/src/walk.rs` 第 8510 行一处还写着 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 改名前的旧简称「回退可以复用被抛弃的根引用的单元」（2026-10-17 现查；`crates/singlefs-core/src/mount.rs` 两处已改成新简称，随代码欠账批 W1 2026-10-15 批派）；顺带核 `crates/` 下别的 `.rs` 里 C143（inode 号水位在崩溃恢复落到旧根后退回去重发）、C498（后续挂载与崩了再挂的暖机次数没有条款）、C542（取号前预演不读盘核，坏盘加旧根全被盖掉会烧号） 三笔的引用（2026-10-01 改名经过在 `.claude/kb/decisions-history.md` D23（journal 的角色与格式） 节同日条目） | `records/2026-10-01-README更新与知识腐烂治理.md`「没做、留给下一批」 | `git grep -n -F` 四个旧简称在 `crates/` 下零命中；改的只是注释（去掉注释与空白之后代码不变），不开代码轮三方（`.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 2 步）；文档注释里编成 doctest 的代码块与 `// SAFETY:` 注释的增删照代码算、并进代码轮 | 随碰 `walk.rs` 的那一批实现员（排在别的会话提交之后） |`
  - 第 112 行：提及的欠账项 C314 在 checks-owed.md 中已还清/销账，但正文未同步：`| 五·8 | 欠账改名留在 `crates/*/src/` 的注释：`crates/singlefs-checker/src/walk.rs` 第 8510 行一处还写着 C314（崩溃恢复落到旧根后可以复用被抛弃的根引用的单元） 改名前的旧简称「回退可以复用被抛弃的根引用的单元」（2026-10-17 现查；`crates/singlefs-core/src/mount.rs` 两处已改成新简称，随代码欠账批 W1 2026-10-15 批派）；顺带核 `crates/` 下别的 `.rs` 里 C143（inode 号水位在崩溃恢复落到旧根后退回去重发）、C498（后续挂载与崩了再挂的暖机次数没有条款）、C542（取号前预演不读盘核，坏盘加旧根全被盖掉会烧号） 三笔的引用（2026-10-01 改名经过在 `.claude/kb/decisions-history.md` D23（journal 的角色与格式） 节同日条目） | `records/2026-10-01-README更新与知识腐烂治理.md`「没做、留给下一批」 | `git grep -n -F` 四个旧简称在 `crates/` 下零命中；改的只是注释（去掉注释与空白之后代码不变），不开代码轮三方（`.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」第 2 步）；文档注释里编成 doctest 的代码块与 `// SAFETY:` 注释的增删照代码算、并进代码轮 | 随碰 `walk.rs` 的那一批实现员（排在别的会话提交之后） |`
  - 第 122 行：提及的欠账项 C356 在 checks-owed.md 中已还清/销账，但正文未同步：`| 关，准入条款簇（三方 `m3-admission-r1` 2026-10-12 已判决：`research/prompts/m3-admission-r1-main-verification.md`，岔路单 `m3-admission-r1-forks.md` 11 条；岔路 F1（需求的单位）与 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）那一条用户已定，其余下轮在入库装置上重量后并进实现批） | C356（第九项按统计量之差算，净释放时为负且没有钳位）、C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C546（抬 F 被拒时扣住的槽不退回）（剩两格：扣住位怎么放；判决「扣住的槽」那一行打中：写错重发后 F 已生效而扣住位不放，假性 ENOSPC，怎么放交用户）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）（同 C546（抬 F 被拒时扣住的槽不退回） 的判决）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）（前置改「无」；判决「段内关着的空槽」那一行：两腿一致，段外槽对读数今天已有，条款还缺，交用户）、C573（固定点上界没算有普通分配那次发布多出的删插）（判决「固定点上界」那一行打中：固定点上界会被超出，交用户）、C574（落得下那一判不核两槽对齐与各盘同槽）、C576（三条用例被准入先拒连带红，改法待选）（判决：行改成按变异名登记，第 149 行漂移、第 12 行不红）、C582（挂载盘容量下界没条款该取哪个） | C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判） 的前置已改「无」，C576（三条用例被准入先拒连带红，改法待选） 已改按变异名登记，其余各行的前置列仍是「条款没定 / 改法待选 / 要先定单位」，同落在 D28（挂载期承诺量） 与 D3（空间分配） 的准入式子上；C370（需求、可用与 df 没有共同单位） 的需求单位用户已定「盘字节、每盘相同」（弹窗定案，原话即选项标签，在 `records/2026-10-12-里程碑三工具与治理批.md`「用户弹窗定案」那一条），写回 D3（空间分配） 已定项 12、D28（挂载期承诺量） 已定项 1 各一句，乘盘数的变异已做成会红用例（`crates/mutations.tsv`「F1（m3-admission-r1）：一次发布的需求乘了盘数」，2026-10-14 随代码欠账批合入）；`df` 的写法仍交用户（判决「`df` 的写法」那一行：照 D16（发布语义） 已定项 1 字面 73/73 多报） |`
  - 第 122 行：提及的欠账项 C370 在 checks-owed.md 中已还清/销账，但正文未同步：`| 关，准入条款簇（三方 `m3-admission-r1` 2026-10-12 已判决：`research/prompts/m3-admission-r1-main-verification.md`，岔路单 `m3-admission-r1-forks.md` 11 条；岔路 F1（需求的单位）与 C568（固定结构取严之后 io_min 1 MiB 的盘建不了卷）那一条用户已定，其余下轮在入库装置上重量后并进实现批） | C356（第九项按统计量之差算，净释放时为负且没有钳位）、C370（需求、可用与 df 没有共同单位）、C371（活元数据没有定义）、C546（抬 F 被拒时扣住的槽不退回）（剩两格：扣住位怎么放；判决「扣住的槽」那一行打中：写错重发后 F 已生效而扣住位不放，假性 ENOSPC，怎么放交用户）、C570（抬 F 回收扣住的槽计数上空闲但分配器不发）（同 C546（抬 F 被拒时扣住的槽不退回） 的判决）、C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判）（前置改「无」；判决「段内关着的空槽」那一行：两腿一致，段外槽对读数今天已有，条款还缺，交用户）、C573（固定点上界没算有普通分配那次发布多出的删插）（判决「固定点上界」那一行打中：固定点上界会被超出，交用户）、C574（落得下那一判不核两槽对齐与各盘同槽）、C576（三条用例被准入先拒连带红，改法待选）（判决：行改成按变异名登记，第 149 行漂移、第 12 行不红）、C582（挂载盘容量下界没条款该取哪个） | C571（df 算不算聚簇段内关着的空槽，假性 ENOSPC 待判） 的前置已改「无」，C576（三条用例被准入先拒连带红，改法待选） 已改按变异名登记，其余各行的前置列仍是「条款没定 / 改法待选 / 要先定单位」，同落在 D28（挂载期承诺量） 与 D3（空间分配） 的准入式子上；C370（需求、可用与 df 没有共同单位） 的需求单位用户已定「盘字节、每盘相同」（弹窗定案，原话即选项标签，在 `records/2026-10-12-里程碑三工具与治理批.md`「用户弹窗定案」那一条），写回 D3（空间分配） 已定项 12、D28（挂载期承诺量） 已定项 1 各一句，乘盘数的变异已做成会红用例（`crates/mutations.tsv`「F1（m3-admission-r1）：一次发布的需求乘了盘数」，2026-10-14 随代码欠账批合入）；`df` 的写法仍交用户（判决「`df` 的写法」那一行：照 D16（发布语义） 已定项 1 字面 73/73 多报） |`
  - 项六、项七的多处统计数值滞后：崩溃枚举用例记载为 14 条（现实际已达 24~29 条）；Verus 登记数记载为 56 条（现实际已达 78 条）；StepKind 记载为 6（现为 7）。
  - 引用的 `.claude/gate.d/e165-void-red-example-ledgers.tsv` 在今天批 3b 中已清理删除。
  - I-8.4 状态列与描述仍标为未实现（实际上在 crash_injection.rs 中已有对应判断）。
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/04-fourth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/04-fourth-txn.md)
- **文件规模**：157 行，35801 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 82 行：提及的欠账项 C109 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D21（权威态与派生态的分界） 已定项 13「救援、换树、迁移走同一条扫描重建路径」；D13（验证路线） 已定项 12 要 checker 做遍历与扫描两个方向；checker 的扫描方向按候选槽扫，读者不给候选时从单元区起点扫到盘尾（`crates/singlefs-checker/src/image.rs`、`walk.rs`），扫出来的只供 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-1.8（归并后版本全序） 与 journal 那几条判定用；checker 里没有用扫描结果重建索引的代码（2026-10-12 现查）。相关欠账：C113（扫描重建时多版单元的现行版本判定无输入）、C130（扫描重建的落点维没有任何度量）、C152（收窄后未发布号重发的扫描重建复核）、C301（deadlist 与 livelist 的态别没写）、C410（checker 两个方向共享什么没有检查）。按关键词搜到、归属是推的（2026-10-12）：欠账 C86（两种单元大小与扫描步进对不拢）、C99（打包共享单元里的记录扫描时发现不了）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C109（哪些树的 key 死亡要写墓碑）、C615（扫盘重建对挂着时的回退没有输入）、C768（退役记录的扫描重建读侧没有消费者）；不变量 I-1.9（清扫水位内无可读垃圾）。`
  - 第 82 行：提及的欠账项 C113 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D21（权威态与派生态的分界） 已定项 13「救援、换树、迁移走同一条扫描重建路径」；D13（验证路线） 已定项 12 要 checker 做遍历与扫描两个方向；checker 的扫描方向按候选槽扫，读者不给候选时从单元区起点扫到盘尾（`crates/singlefs-checker/src/image.rs`、`walk.rs`），扫出来的只供 I-7.8（根记录树 ID 水位不低于全池最大树 ID）、I-1.8（归并后版本全序） 与 journal 那几条判定用；checker 里没有用扫描结果重建索引的代码（2026-10-12 现查）。相关欠账：C113（扫描重建时多版单元的现行版本判定无输入）、C130（扫描重建的落点维没有任何度量）、C152（收窄后未发布号重发的扫描重建复核）、C301（deadlist 与 livelist 的态别没写）、C410（checker 两个方向共享什么没有检查）。按关键词搜到、归属是推的（2026-10-12）：欠账 C86（两种单元大小与扫描步进对不拢）、C99（打包共享单元里的记录扫描时发现不了）、C104（扫描路径上码 1 / 2 的载荷完整性无自包含证据）、C109（哪些树的 key 死亡要写墓碑）、C615（扫盘重建对挂着时的回退没有输入）、C768（退役记录的扫描重建读侧没有消费者）；不变量 I-1.9（清扫水位内无可读垃圾）。`
  - 第 55 行：提及的欠账项 C122 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D26（后台整理与放置回收） 已定项 1、2、3、4、5、6、7、11、12 定了触发、事务形态、准入底额、记账交互、观测点、回收分线与腿一的形态；未定项 8、9、10 只管交给盘那条线。E93（老化下的放置与碎片度） 判腿一是必需的；E10（老化：结构抗老化 vs 事后整理） 未跑。代码：`crates/singlefs-core/src/allocator.rs` 的 `reclaim_released_up_to` 只把已释放的落点放回空闲、不搬活数据；`crates/singlefs-core/src/system_configuration.rs` 的 `COMPACTION_WATERMARK_BUILT_IN_DEFAULT` 与 `compaction_watermarks` 让整理三条水位第一版恒 0（0 = 内置默认）；`grep -rn "discard" crates/singlefs-core` 零命中。D3（空间分配） 已定项 7 登记表第 7 行逐字「`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）」。相关欠账：C2（无界操作做成单事务）、C91（整理意图与增量触发量的重做漂移）、C139（整理停摆之后释放也发不出去）、C142（第一个 reflink 之后整理会造出静默陈旧读）、C364（整理意图记录没有字段表）、C412（整理的挂钟代价没量过）。GC 意图的载体是 logged_ops 树（`research/prompts/m3-intent-r1-main-verification.md`；用户定案 K-I1：零根注册、字段表等整理落地时再定，原话在变更史，被攻过零轮），今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`（`crates/singlefs-core/src/records.rs`），没有写者；删除落地的那套意图（`transaction.rs` 的 `DeletionStepOfThePublish`）只认 unlink，装不下 GC 意图：C364（整理意图记录没有字段表） 是二·1 的第一题（2026-10-12 现查）。E10（老化：结构抗老化 vs 事后整理） 没有跑前登记（`research/prompts` 下 `e10-*` 零个），实验页里没有数值门槛，长期老化（挪进里程碑九「五　长期老化」）要的门槛由二·1 先登记出来。按关键词搜到、归属是推的（2026-10-12）：欠账 C122（分配记录不点删之后的条目数口径）、C133（搬迁的转发信息没有载体）、C140（意图完成不等于空段被腾出）、C141（停机判据的输入不持久）、C146（无空段时的回落政策全仓无定义）、C197（迁移的三笔写是不是一个事务没写下来）、C243（停机谓词那个量从来没有过轨迹）、C326（runs 换口径之后 E93 的数还能不能引）、C372（全空聚簇段数两份定义相反）、C373（停机谓词判真时活意图删不删没有条款）、C413（碎片度与全空段统计没有不变量）、C621（整理落地时要动的删除条款没有清单外的检查）；分项 D3（空间分配） 已定项 9、14，D5（快照 / 空间记账机制） 已定项 15。`
  - 第 55 行：提及的欠账项 C146 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D26（后台整理与放置回收） 已定项 1、2、3、4、5、6、7、11、12 定了触发、事务形态、准入底额、记账交互、观测点、回收分线与腿一的形态；未定项 8、9、10 只管交给盘那条线。E93（老化下的放置与碎片度） 判腿一是必需的；E10（老化：结构抗老化 vs 事后整理） 未跑。代码：`crates/singlefs-core/src/allocator.rs` 的 `reclaim_released_up_to` 只把已释放的落点放回空闲、不搬活数据；`crates/singlefs-core/src/system_configuration.rs` 的 `COMPACTION_WATERMARK_BUILT_IN_DEFAULT` 与 `compaction_watermarks` 让整理三条水位第一版恒 0（0 = 内置默认）；`grep -rn "discard" crates/singlefs-core` 零命中。D3（空间分配） 已定项 7 登记表第 7 行逐字「`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）」。相关欠账：C2（无界操作做成单事务）、C91（整理意图与增量触发量的重做漂移）、C139（整理停摆之后释放也发不出去）、C142（第一个 reflink 之后整理会造出静默陈旧读）、C364（整理意图记录没有字段表）、C412（整理的挂钟代价没量过）。GC 意图的载体是 logged_ops 树（`research/prompts/m3-intent-r1-main-verification.md`；用户定案 K-I1：零根注册、字段表等整理落地时再定，原话在变更史，被攻过零轮），今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`（`crates/singlefs-core/src/records.rs`），没有写者；删除落地的那套意图（`transaction.rs` 的 `DeletionStepOfThePublish`）只认 unlink，装不下 GC 意图：C364（整理意图记录没有字段表） 是二·1 的第一题（2026-10-12 现查）。E10（老化：结构抗老化 vs 事后整理） 没有跑前登记（`research/prompts` 下 `e10-*` 零个），实验页里没有数值门槛，长期老化（挪进里程碑九「五　长期老化」）要的门槛由二·1 先登记出来。按关键词搜到、归属是推的（2026-10-12）：欠账 C122（分配记录不点删之后的条目数口径）、C133（搬迁的转发信息没有载体）、C140（意图完成不等于空段被腾出）、C141（停机判据的输入不持久）、C146（无空段时的回落政策全仓无定义）、C197（迁移的三笔写是不是一个事务没写下来）、C243（停机谓词那个量从来没有过轨迹）、C326（runs 换口径之后 E93 的数还能不能引）、C372（全空聚簇段数两份定义相反）、C373（停机谓词判真时活意图删不删没有条款）、C413（碎片度与全空段统计没有不变量）、C621（整理落地时要动的删除条款没有清单外的检查）；分项 D3（空间分配） 已定项 9、14，D5（快照 / 空间记账机制） 已定项 15。`
  - 第 55 行：提及的欠账项 C364 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D26（后台整理与放置回收） 已定项 1、2、3、4、5、6、7、11、12 定了触发、事务形态、准入底额、记账交互、观测点、回收分线与腿一的形态；未定项 8、9、10 只管交给盘那条线。E93（老化下的放置与碎片度） 判腿一是必需的；E10（老化：结构抗老化 vs 事后整理） 未跑。代码：`crates/singlefs-core/src/allocator.rs` 的 `reclaim_released_up_to` 只把已释放的落点放回空闲、不搬活数据；`crates/singlefs-core/src/system_configuration.rs` 的 `COMPACTION_WATERMARK_BUILT_IN_DEFAULT` 与 `compaction_watermarks` 让整理三条水位第一版恒 0（0 = 内置默认）；`grep -rn "discard" crates/singlefs-core` 零命中。D3（空间分配） 已定项 7 登记表第 7 行逐字「`N₀` 第一版取 0（整理未实现，整理进里程碑时重开）」。相关欠账：C2（无界操作做成单事务）、C91（整理意图与增量触发量的重做漂移）、C139（整理停摆之后释放也发不出去）、C142（第一个 reflink 之后整理会造出静默陈旧读）、C364（整理意图记录没有字段表）、C412（整理的挂钟代价没量过）。GC 意图的载体是 logged_ops 树（`research/prompts/m3-intent-r1-main-verification.md`；用户定案 K-I1：零根注册、字段表等整理落地时再定，原话在变更史，被攻过零轮），今天代码里只有零根与种类码 `TREE_KIND_LOGGED_OPERATIONS`（`crates/singlefs-core/src/records.rs`），没有写者；删除落地的那套意图（`transaction.rs` 的 `DeletionStepOfThePublish`）只认 unlink，装不下 GC 意图：C364（整理意图记录没有字段表） 是二·1 的第一题（2026-10-12 现查）。E10（老化：结构抗老化 vs 事后整理） 没有跑前登记（`research/prompts` 下 `e10-*` 零个），实验页里没有数值门槛，长期老化（挪进里程碑九「五　长期老化」）要的门槛由二·1 先登记出来。按关键词搜到、归属是推的（2026-10-12）：欠账 C122（分配记录不点删之后的条目数口径）、C133（搬迁的转发信息没有载体）、C140（意图完成不等于空段被腾出）、C141（停机判据的输入不持久）、C146（无空段时的回落政策全仓无定义）、C197（迁移的三笔写是不是一个事务没写下来）、C243（停机谓词那个量从来没有过轨迹）、C326（runs 换口径之后 E93 的数还能不能引）、C372（全空聚簇段数两份定义相反）、C373（停机谓词判真时活意图删不删没有条款）、C413（碎片度与全空段统计没有不变量）、C621（整理落地时要动的删除条款没有清单外的检查）；分项 D3（空间分配） 已定项 9、14，D5（快照 / 空间记账机制） 已定项 15。`
  - 截断与 compaction 合并后的最新阶段产物未回填更新。
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/05-fifth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/05-fifth-txn.md)
- **文件规模**：143 行，22695 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/06-sixth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/06-sixth-txn.md)
- **文件规模**：83 行，12385 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/07-seventh-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/07-seventh-txn.md)
- **文件规模**：49 行，4406 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/08-eighth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/08-eighth-txn.md)
- **文件规模**：67 行，8934 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/09-ninth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/09-ninth-txn.md)
- **文件规模**：110 行，15016 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 74 行：提及的欠账项 C6 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：崩溃放量的 0 红是在设备模型成立的前提下（flush / FUA 的顺序、原子宽度 = 运行时探测的 `physical_block_size`，D20（承重面：单元的原子性与自包含））；相关欠账 C6（块层语义假设写错）、C26（屏障数从没量过）。线索（未在本项目验证）：FAST'13《Understanding the Robustness of SSDs under Power Fault》测了 15 块消费级盘，只有一个型号严格守住掉电语义。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/10-tenth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/10-tenth-txn.md)
- **文件规模**：73 行，12830 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/11-eleventh-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/11-eleventh-txn.md)
- **文件规模**：62 行，6740 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 32 行：提及的欠账项 C109 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：全仓没有一条管目录结构的决策。相关的已定条款：D8（核心索引结构） 已定项 3「key = `(locality_id, inode, offset)`，`locality_id` 创建时从父目录继承、改名时故意不更新」；D8（核心索引结构） 已定项 5 接受墓碑、可能修改无界数量 extent 的操作先写意图再分批推进；D14（双轨（大小文件 / 持久临时）） 已定项 2 第一版不做分配器提示（重开条件三条）；D14（双轨（大小文件 / 持久临时）） 已定项 3 内联阈值 0。欠账：C109（哪些树的 key 死亡要写墓碑）、C527（改名覆盖借错删除规则的一半）、C2（无界操作做成单事务）（删大目录那一格还欠）、C115（inode 属性的溢出路线无落点）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/12-twelfth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/12-twelfth-txn.md)
- **文件规模**：79 行，8722 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 42 行：提及的欠账项 C6 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D16（发布语义） 已定项 11「所有结构、所有操作先进同一个 journal、共用同一个 checkpoint 节奏，fsync 只是提前触发一次发布」；D16（发布语义） 已定项 17「报错之后这次发布可能已成也可能未成，不许假设可复用旧内存态重试；成没成由调用方自己读现行根判定」；D16（发布语义） 已定项 2 checkpoint 的触发判据。全仓没有一条决策管页缓存脏页、FUSE 的 flush 与 release、`O_DIRECT`、mmap 写。`grep -n 'fuse' crates/*/Cargo.toml Cargo.toml` 零命中。欠账：C6（块层语义假设写错）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/13-thirteenth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/13-thirteenth-txn.md)
- **文件规模**：63 行，7965 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 44 行：提及的欠账项 C55 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D2（RAID 条带策略） 已定项 3「走显式「几何再求值」事务并原子发布，卷保持挂载，不许强制重挂」；已定项 4「几何不合格拒绝加盘、排空后再换几何、加盘后跑一次后台可续做的重平衡，搬迁单位是整个对象」；已定项 5「根环不重放置；重平衡搬用户数据、不动根环，掉盘掉到 devs < R 时接受退化」；D22（单元原子性怎么合成） 已定项 14 devs < R 时的根环判法（里程碑九二·3 先做单盘与两盘那一支）。欠账：C17（设备几何当全局参数）、C55（排空没排干净就换几何）、C82（io_min 不在挂载比对清单）、C120（分叉盘回归的判定与重同步）、C126（切换预留的最坏量没有口径）、C346（后台重新聚簇没有定义）。`
  - 第 44 行：提及的欠账项 C120 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D2（RAID 条带策略） 已定项 3「走显式「几何再求值」事务并原子发布，卷保持挂载，不许强制重挂」；已定项 4「几何不合格拒绝加盘、排空后再换几何、加盘后跑一次后台可续做的重平衡，搬迁单位是整个对象」；已定项 5「根环不重放置；重平衡搬用户数据、不动根环，掉盘掉到 devs < R 时接受退化」；D22（单元原子性怎么合成） 已定项 14 devs < R 时的根环判法（里程碑九二·3 先做单盘与两盘那一支）。欠账：C17（设备几何当全局参数）、C55（排空没排干净就换几何）、C82（io_min 不在挂载比对清单）、C120（分叉盘回归的判定与重同步）、C126（切换预留的最坏量没有口径）、C346（后台重新聚簇没有定义）。`
  - 第 44 行：提及的欠账项 C126 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D2（RAID 条带策略） 已定项 3「走显式「几何再求值」事务并原子发布，卷保持挂载，不许强制重挂」；已定项 4「几何不合格拒绝加盘、排空后再换几何、加盘后跑一次后台可续做的重平衡，搬迁单位是整个对象」；已定项 5「根环不重放置；重平衡搬用户数据、不动根环，掉盘掉到 devs < R 时接受退化」；D22（单元原子性怎么合成） 已定项 14 devs < R 时的根环判法（里程碑九二·3 先做单盘与两盘那一支）。欠账：C17（设备几何当全局参数）、C55（排空没排干净就换几何）、C82（io_min 不在挂载比对清单）、C120（分叉盘回归的判定与重同步）、C126（切换预留的最坏量没有口径）、C346（后台重新聚簇没有定义）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/14-fourteenth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/14-fourteenth-txn.md)
- **文件规模**：57 行，7309 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 42 行：提及的欠账项 C260 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D6（快照实现模型） 已定项 3「要；取 ZFS livelist 事件流形态，FREE 在块被杀掉那一刻就记；载体是一棵共享树，key 以头的树 ID 打头」；D6（快照实现模型） 已定项 5「做，且付查找：每删一个快照 O(N) 次反向索引查找」；D6（快照实现模型） 已定项 10「嵌套区间标号加删除前沿模型层已验能拉回 O(分支数)；两笔限度还清之前实现仍按付查找走」。欠账：C35（多可写头的 O(N) 没有优化过）、C260（livelist 条目的预付没算进解开自己那个量）、C265（销毁可写头的释放边界写成了固定的克隆时刻）、C266（livelist 条目的身份用了会过期的位置提示）、C270（分叉点闸的子计数数不数克隆）、C271（livelist 的 FREE 在什么时刻记没有定义）、C272（第二道销毁闸没有 O(1) 的检查法）、C337（被销毁的头何时不算活引用没有条款）。`
  - 第 42 行：提及的欠账项 C266 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D6（快照实现模型） 已定项 3「要；取 ZFS livelist 事件流形态，FREE 在块被杀掉那一刻就记；载体是一棵共享树，key 以头的树 ID 打头」；D6（快照实现模型） 已定项 5「做，且付查找：每删一个快照 O(N) 次反向索引查找」；D6（快照实现模型） 已定项 10「嵌套区间标号加删除前沿模型层已验能拉回 O(分支数)；两笔限度还清之前实现仍按付查找走」。欠账：C35（多可写头的 O(N) 没有优化过）、C260（livelist 条目的预付没算进解开自己那个量）、C265（销毁可写头的释放边界写成了固定的克隆时刻）、C266（livelist 条目的身份用了会过期的位置提示）、C270（分叉点闸的子计数数不数克隆）、C271（livelist 的 FREE 在什么时刻记没有定义）、C272（第二道销毁闸没有 O(1) 的检查法）、C337（被销毁的头何时不算活引用没有条款）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/15-fifteenth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/15-fifteenth-txn.md)
- **文件规模**：64 行，8474 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 55 行：提及的欠账项 C32 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D9（加密） 已定项 5「只做 scrub：加密开启时无密钥的一侧只读只检测，不搬、不疏散、不回收」；D3（空间分配） 已定项 16「只读只检测，既不搬也不回收；常驻自动整理那条承诺在加密开启时不成立」。欠账：C32（无密钥的一侧写不写得出字节没有闸）、C136（无密钥能验哪几条不变量没人标过）、C166（明文侧取消之后格式预留要不要跟着收）。`
  - 第 33 行：提及的欠账项 C185 在 checks-owed.md 中已还清/销账，但正文未同步：`- **仓里已有的**：D9（加密） 已定项 10「加密进不进第一个可运行版本：不进，只预留格式字段」，已定项 12 的四样预留已经实现。未实现：D9（加密） 已定项 11「整卷 AEAD，加密边界与校验和边界重合；除系统配置外全部加密，元数据留最小明文头」；已定项 1「指针里有 MAC 与密文校验和两个完整性字段，密文校验和不要密钥、所有读路径都用」；已定项 2「取满 128 位，不截断」；已定项 3「两种都实现，盘上记算法类型，运行时按 CPU 特性选默认值；加密路径必须能并行」；已定项 4「每 extent 存一份 96 位 nonce，不现算」；已定项 8「fsid 进 KDF、系统配置因 nonce 水位而必须认证、整卷回滚不防」；已定项 13「nonce 不许由物理位置派生；顺序固定为压缩 → 加密 → MAC」；D18（块里携带什么信息） 已定项 13「码 1 身份段与声明长度就地变密文，位置宽度不动」。欠账：C74（无加密期的跨卷错向写没防线）、C185（载荷 CRC 覆盖密文还是明文没定）、C193（加密开启时的单元头有多大，全仓没算过）、C399（nonce 不重用那条注的前提没人回看）、C240（搬运时重新加密的代价没算过）。`
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/milestone/16-sixteenth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/16-sixteenth-txn.md)
- **文件规模**：65 行，6970 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/milestone/17-seventeenth-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/milestone/17-seventeenth-txn.md)
- **文件规模**：69 行，7959 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/layout/01-first-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/layout/01-first-txn.md)
- **文件规模**：437 行，101101 字节
- **判定结论**：⚠️ **存在过时、断链或与现状脱节之处**
- **具体过时/不一致点**：
  - 第 402~404 行：段序列表仍记录与 E142 第二十五次跑的主产物比对，或引用已不存在的单测名 `registered_segment_sequences_match_every_recorded_path`。
- **建议处置**：定点同步最新常数值、更新格名指针、销账已还清条目并修正历史数字。

## [.claude/kb/layout/02-second-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/layout/02-second-txn.md)
- **文件规模**：26 行，10181 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。

## [.claude/kb/layout/03-third-txn.md](file:///home/fy5090/code/singlefs/.claude/kb/layout/03-third-txn.md)
- **文件规模**：78 行，20213 字节
- **判定结论**：✅ **一致/无显著过时**
- **核查说明**：各条款、状态列及决策依据与当前代码库保持一致。
