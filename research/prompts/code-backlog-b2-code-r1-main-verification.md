# code-backlog-b2-code-r1 判决：代码欠账批甲簇（C572、B3）代码轮第一轮（2026-10-14）

被判的改动（按路径点名）：`crates/singlefs-core/src/mount.rs`（B3 五处与新私有辅助、C572 那一步与新拒因、`#[cfg(test)] mod shadow_ledger_exempts_and_isolates_slot_by_slot_tests`）；连带看了 `crates/singlefs-harness/src/history.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/tests/rollback_by_a_forward_publish_verifies_every_copy_of_an_isolated_unit_the_target_references.rs` 与 `crates/mutations.tsv` 追加 6 行、整行替换 8 行。`crates/singlefs-core/src/transaction.rs` 经 `comment-only-change.py . HEAD` 判只改注释，不在这一轮。

<!-- doc-lint:not-numbers B3 K1 K2 K3 K4 A1 A2 A3 -->

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本里对 `research/prompts/code-backlog-b2-code-r1-snapshot/crates-sha256.txt` 的 `crates/` 那几行 `sha256sum -c`；主工作区 `mount.rs` 同样核 | 冻结副本六份全对上，主工作区 `mount.rs` 与快照相同。kb 三份里 `.claude/kb/checks-owed.md` 对不上：别的会话在第 554、555 行插了 C833、C834 两行；去掉这两行之后 sha256 cfea72c9… 与快照逐字相同。两条腿引这份文件只引第 365、377 行，都在插入点之前 |
| 正推腿 `research/prompts/code-backlog-b2-code-r1-sonnet-output.md`（sha256 4d96cabe9876b076…）、攻方 `research/prompts/code-backlog-b2-code-r1-opus-output.md`（sha256 c0f442b923697437…，260 行） | `python3 scripts/agents/cite-check.py <两份> --root /home/fy5090/code/singlefs --background research/prompts/_code-backlog-b2-code-r1-background.md` | 「✓ 核了 108 处引文、5 处裸行号，对上 108 处，没判 5 处」；5 处裸行号都在攻方报告（`mount.rs:1690`、`:2555`、`:2481`、`:1908`、`23-journal的角色与格式.md:390`），主 agent 在冻结副本里逐处现读，指的是报告说的那一段 |
| 两条腿的模型 | 模型目录拷进 `~/.local/share/singlefs/work/code-backlog-b2-code-r1-main-rerun/`，各经 `scripts/hosts/run-with-memory-cap.sh 16G` 跑 `tools/rerun.sh --repo <仓根> --frozen <冻结副本> --work <新目录>` | 正推 rc=0，末行「name=rerun_compare differing=0」；攻方 rc=0，末行「name=rerun_compare tally_same=yes lines_same=yes」 |
| 攻方报的计数 | 复跑日志 `opus.log` 的计数行 | 影子账 838 + 2968 + 22199 + 35968 = 61973 次（「main-harness-46」「main-random-fast-and-ignored」「main-large-rollback-unit-area-384」「main-large-reuse-unit-area-240」四行的 `b3_calls`），`b3_new_only_nonempty=0`、`b3_old_only_nonempty=0`；回退 45 + 776 + 19911 + 2938 = 23670 次，`k2_isolated_nonempty` 合计 2；次序互换那一趟「name=swap-core-lib tests_passed=255 tests_failed=2」、「name=swap-harness-46-with-ignored tests_passed=389 tests_failed=0」 |
| 次序为什么承重 | 冻结副本 `mount.rs` 第 1744–1756 行现读 | `slots_of_a_placement_to_isolate_now` 是 `.filter(\|covered\| !exempt.contains(covered) && isolated.insert(*covered))`：豁免集合各圈各取（被抛弃根那一圈豁免候选根与现行账的槽，C 那一圈只豁免现行账的槽），判重集合 `isolated` 各圈共用；先看豁免时被豁免的槽不进 `isolated`，C 那一圈还隔离得到它，次序互换就进了 `isolated`、C 那一圈跳过 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| K1 B3 五处按槽展开 | 对得上（量的）：五处都按槽认豁免与判重，追加 B3 五行各改回去红在点名用例；末槽 `expect` 各来路走不到；另交 K1.5（次序钉不住）、K1.6（交叠时少隔离一槽） | **打中用例缺口（量过）**：豁免先于判重这一次序没有用例钉，互换后六条 B3 用例、core 全库、46 个 harness 目标全绿，只有攻方两条直调用例红；经公开入口 61973 次影子账按槽与按起点槽交给分配器的槽一次都没分开（随机历史里没有被抛弃根，只说明这些历史走不到）；判重跨两种隔离调法没打中（推的） | 代码对得上条款 D23（journal 的角色与格式） 已定项 14「只隔离其中只被被抛弃根引用的槽」。两腿都量到次序承重而无用例钉住（主 agent 现读第 1752 行坐实机理）：采纳 A1。K1.6 两腿结论不一（正推量到探针上少隔离一槽、攻方推回收时兜得住），都没经公开入口造出、条款没写读盘得来的被抛弃根回收时靠谁护：不在这一轮判，另立 C835 |
| K2 C572 隔离单元怎么认 | 对得上（量的）：现行那一版的槽集合恰好认出被隔离的单元，四个角色造共用单元带坏份都做成；**K2.3**：「cur 那一版引用的不进逐盘验」那一道（第 7821 行）去掉后入库两条与 `rollback_by_a_forward_publish.rs` 25 条全绿，只有正推两个探针红；K2.4 两条位置项不同槽判不了（条款没写） | 没打中（量过）：23670 次回退里隔离单元清单只 2 次非空、都出自 C572 用例；回退自己释放造出的隔离再回退回去照样认出、两块坏盘各拒 | 代码对得上条款 D23（journal 的角色与格式） 已定项 14「其余 cur 的账里仍分配的单元……不在逐盘验之内」。K2.3 是用例缺口（正推量的，主 agent 复跑 differing=0）：采纳 A2。K2.4 记进 A2 的射程说明，不另立 |
| K3 拒在什么时候 | 对得上（量的）：新一步排在复活集逐盘验之后、`allocator.clone()`（第 8290 行）之前，之间全是只读；`first_copy_that_does_not_read_back` 抽出后复活集那一判不变；去掉 C572 那一步拒绝用例红 | 没打中（量过）：拒之后分配器整份 Debug 串、现行版本、盘与录制流都不变；两种拒因同时成立报复活集那一条 | 一致，判对。两种拒因的先后条款没定，今天报复活集那一条，不改 |
| K4 harness 两臂、用例与变异 | 对得上；**K4.4**：整行替换第 1 行「步 4：影子账不豁免候选根引用的槽（保守读法）」点名的 harness 用例不红，HEAD f111c3f3 那一版同样不红（不是这一批带进来的），换到 core 两条用例上红 | 没打中（量过）：新拒因在模型里是 `Unexplained`，只出现在 C572 用例里；14 行变异锚点各命中 1 次 | 一致，判对。K4.4 照 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」是取样点不敏感：采纳 A3 |

单次抽样：攻方在 K2、K3、K4 的「没打中」与 K1 的公开入口一条各只抽了一次样，不单独当「没打中」用；这几格判「对得上」靠的是正推腿的正向核对与主 agent 复跑，下一批代码轮另换攻击面再攻 A1–A3 的改动。

## 三、采纳的改法（并进代码欠账批下一批）

| 号 | 改什么 | 验收 | 被攻过 |
|---|---|---|---|
| A1 | 把攻方 `research/prompts/code-backlog-b2-code-r1-opus-model/src/k1-tests.rs` 的 `opus_k1_exempt_before_dedup_keeps_the_c_ring_isolating_a_slot_the_abandoned_ring_exempted`、`opus_k1_exempt_before_dedup_keeps_the_c_ring_isolating_a_slot_the_ring_slot_read_bad_ring_exempted`（或正推探针 P1 `probe_p1_a_slot_exempt_for_the_abandoned_roots_by_a_candidate_is_still_isolated_by_the_c_circle`）收进 `mount.rs` 的用例模块，改名照命名规矩（不带腿名）；`crates/mutations.tsv` 加一行把第 1752 行的过滤改成先判重、后看豁免 | 改前（次序互换）红、改后绿；`prove-red.sh` 抓到新行 | 零轮（攻方与正推各自在模型上量过） |
| A2 | 加一条用例钉「cur 那一版引用的单元不进逐盘验」：共用单元带一份坏、回退目标引用它，回退照做（取正推 `out/s6-k2-probes-guard-dropped.txt` 那两个探针的形）；`crates/mutations.tsv` 加一行把第 7821 行改成 `true \|\| !is_referenced_by_the_current_version` | 那一行变异红在新用例上；两条位置项不同槽的映射条目怎么认条款没写（K2.4），用例只取同槽 | 零轮 |
| A3 | `crates/mutations.tsv` 整行替换第 1 行「步 4：影子账不豁免候选根引用的槽（保守读法）」的点名测试换成 core 的 `a_candidate_roots_two_slot_placement_exempts_its_second_slot_from_an_abandoned_roots_placement` | 门禁 checker-tier-crates-mutation-replay 复跑那一行红 | 零轮 |

A1、A2 改的是 `mount.rs` 的 `#[cfg(test)]` 模块与 harness 用例，`mount.rs` 的用例模块在产品代码射程里（`.claude/rules/implementation-workflow.md`「三步，缺一步就不算做完」），下一批照样开代码轮。

## 四、另立的账

- C835（读盘得来的被抛弃根回收时谁护没条款）：正推 K1.6 与攻方旁见，`.claude/kb/checks-owed.md` 第 556 行。先定条款再走代码轮，不在这一轮判。
- 正推 K2.4（两条位置项不同槽的映射条目怎么认）：条款 D19（块指针的结构与宽度预算） 已定项 5 没写，今天的代码按「任一条位置项的槽在现行那一版里」认；不另立，随 A2 的射程说明交下一批。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-14 不受影响：代码对得上「只隔离其中只被被抛弃根引用的槽」与「在任何写之前拒」那一格；读盘得来的被抛弃根回收时谁护条款没写，另立 C835 |
| D19（块指针的结构与宽度预算） 已定项 5 | 不影响 | 2026-10-14 不受影响：隔离单元按硬规则 1「两份都留在已分配」认，代码与条款同；两条位置项不同槽的认法条款没写 |

## 五、复跑核对

- 正推：`~/.local/share/singlefs/work/code-backlog-b2-code-r1-main-rerun/sonnet.log` 末行「name=rerun_compare differing=0」（日志里「test result: FAILED. 4 passed; 2 failed」是探针在弄坏的副本上按设计红）。
- 攻方：同目录 `opus.log` 末行「name=rerun_compare tally_same=yes lines_same=yes」，计数行见第一节。

## 更正（2026-10-14，判决写完之后）

K4 那一格「随机历史走不到新拒因」不成立。主 agent 照 C600（吞写之后可写挂载被拒而模型说该成） 登记的那组参数在主工作区重跑故障注入大档（`SINGLEFS_FAULT_INJECTION_FIRST_SEED=7463871032432355113 SINGLEFS_FAULT_INJECTION_SEEDS=512 SINGLEFS_FAULT_INJECTION_OPERATIONS=30 SINGLEFS_FAULT_INJECTION_FAULTS=6`，`cargo test --release -p singlefs-harness --test fault_injection_fast_tier -- --ignored --exact fault_injection_large_tier_from_the_environment`，经 16G 内存包装），整条红在「以「已知红」收尾 {}、新发现 5」，5 次都是同一个签名：「种子 7463871032432355312 write_is_swallowed：整池第 146 次write，落在第 10 步（PublishOverwrite）：ModelDisagreement { aspect: "模型说该成、实现拒了" }」「模型 RollbackWhileMounted 该成（写出 [(19, 2)]；允许拒的只有容量墙区间与 []）；实现 拒了：RollbackError::IsolatedUnitCopyUnreadableOrMismatched」。正推腿那一句是推的（没跑随机历史与故障注入），攻方的取样里没有故障注入大档。

判：实现照 D23（journal 的角色与格式） 已定项 14「有一份读不出或对不上就在任何写之前拒」拒，是对的（设备吞了写，那一份对不上）；错在模型那一侧不认设备撒谎之后回退的这一拒（`crates/singlefs-harness/src/model_comparison.rs` 把新成员判成 `Unexplained`，与 `ResurrectedUnitCopyUnreadableOrMismatched` 同处置）。采纳 A4：故障注入里设备撒谎（吞写、读回坏字节）落在回退目标引用的隔离单元上时，模型认 `IsolatedUnitCopyUnreadableOrMismatched` 这一拒，`ResurrectedUnitCopyUnreadableOrMismatched` 同一形照同一个判据核一遍；验收是同一组参数重跑大档新发现 0、种子 7463871032432355312 钉成一条 harness 用例（先证它在改前的模型上红）。只改测试代码，零轮，并进下一批。
