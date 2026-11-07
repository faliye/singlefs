# m3-close-b19-switch-sweep-code-r1 判决：批 19 写段末两处产品代码改动的代码轮（2026-11-06）

正文 `research/prompts/_m3-close-b19-switch-sweep-code-r1-body.md`；腿报告：正推 `research/prompts/m3-close-b19-switch-sweep-code-r1-sonnet-output.md`（b307999c…），攻方 `research/prompts/m3-close-b19-switch-sweep-code-r1-opus-output.md`（dc65f1d6…）。被判的产品代码：`crates/singlefs-core/src/mount.rs`（E205（多处读坏倒挂三条改法的代价） 新实例第一个 txg 的只供测试开关，连同同一批并进的测试模块）、`crates/singlefs-core/src/instance_table_sweep.rs`（C973（实例表清扫读根环不核落点） 的改法）。

## 一、核查

- 开工快照：`task-server/task submit --cwd ~/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m3-close-b19-switch-sweep-code-r1 --out ~/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-main/m3-close-b19-switch-sweep-code-r1-tree --repository /home/fy5090/code/singlefs`（9261）退 0，末行「✓ 快照 m3-close-b19-switch-sweep-code-r1：清单点名 6 份，倒推出 6 份，sha256 对上 6 份」。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-switch-sweep-code-r1-sonnet-output.md research/prompts/m3-close-b19-switch-sweep-code-r1-opus-output.md --root ~/.local/share/singlefs/work/m3-close-b19-switch-sweep-code-r1-main/m3-close-b19-switch-sweep-code-r1-tree --background research/prompts/_m3-close-b19-switch-sweep-code-r1-background.md` 末行「✓ 核了 106 处引文、79 处裸行号，对上 85 处，没判 100 处（其中裸行号没有引文可比 67 处；逐处列在上面）」，没有判成对不上的。没判的引文都落在快照树之外，判决依靠的几处主 agent 现读：
  - `research/prompts/e205-preregistration.md` 第 41–43 行：甲两档定义与共用取法，第 43 行写「N = 这次挂载读根环读得出、自证过（含落点公式核）的根的最大 txg（与 `first_txg_of_new_instance` 里 `highest_root_txg` 是同一个量）」；
  - `.claude/kb/decisions/18-块里携带什么信息.md` 第 318 行：清扫「其余的槽须自证过，否则重读一次，仍不过就这次不删」；
  - `crates/mutations.tsv` 第 6874 行是「甲的 N 取读得出的根里最小的 txg」那一行变异，第 6908、6909 行是 C973 两行；
  - `crates/singlefs-core/src/mount.rs` 第 2991 行注释「别的读者（择根、算上限那一遍、清扫）照旧用 [`read_root_ring_slot`]」；
  - `crates/singlefs-core/src/recovery.rs` 第 5054 行在 `root_record_bytes_of_the_version` 里，读出的字节拿 `parse_slot` 的结果与已知根整条比（`== Some(root)`）；它有调用者 `crates/singlefs-verification/src/mount_transparency.rs:254`，攻方「本仓里没有调用者」那半句不成立。
- 攻方腿的六条命令，主 agent 在它交接的两份副本上原样交（副本上加的用例，读数不进 kb）：
  - 9255（副本，`first_txg_window_against_the_registration_definition_tests`）退 0：「test result: ok. 4 passed; 0 failed」；`WINDOW_AGAINST_REGISTRATION cases=388674 mismatches=0`；`WINDOW_EDGE` 11 行，code 与 registration 逐行相同（例 `WINDOW_EDGE genesis_only_region2_slot0_bad S=4 N=0 today=1 code=Ok(3) registration=Ok(3)`、`WINDOW_EDGE top_minus_one_every_slot S=5 N=18446744073709551614 today=18446744073709551615 code=Err(18446744073709551615) registration=Err(18446744073709551615)`）；`TRANSIENT_PASS registration_reading=Ok(22) code_reading=Ok(11)`。
  - 9256（副本，`instance_table_sweep::misplaced_ring_reads_tests`）退 101：「test result: FAILED. 3 passed; 1 failed」。打印行 `GENESIS_MISPLACED_READ rows_to_delete=[InstanceRow { instance: InstanceGeneration(1), selected_root_txg: CheckpointTxg(1), applied_transaction_high_water: 0 }] heads_to_erase=0`，主断言「实例 1 那一行判成要删」过了；红在用例末尾「那一槽读了两次，两次都错位」（`left: 1`）：错位读回的第 0 代根第一次就过落点核，不触发重读，只错位读了一次。红的是攻方用例自己的前提，打中照成立，而且一次错位读就够。
  - 9257（副本，验证包 `writable_mount_first_txg_switch_past_root_ring_slots_read_bad_or_all_zero`，原有 12 条加攻方 5 条）退 0：「test result: ok. 17 passed; 0 failed」；`NOT_POWER_OF_TWO` 6 行、`TODAY_WINS` 1 行、`GENESIS_ONLY` 1 行、`ALL_SLOTS_UNREADABLE` 3 行（三档都是 `refused=Recovery(NoValidRoot)`）。
  - 9258（变异体副本，去掉 `crates/singlefs-core/src/mount.rs:1652` 的 `first_txg_today.max(…)`，`first_txg_past_the_ring_slots_read_bad_at_mount_tests`）退 0：「test result: ok. 3 passed; 0 failed」。
  - 9283（变异体副本，同一个验证包目标；9259 因副本里 E205 装置没跟上 `PoolVersion::WithFile` 改收 `Arc` 编不过，补一行后重交）退 101：「test result: FAILED. 16 passed; 1 failed」，红的只有攻方加的 `writable_mount_of_the_starting_image_with_records_past_every_ring_slot_read_bad_keeps_todays_value_past_read_bad`，原有 12 条都在 passed 里。
  - 9260（变异体副本，攻方对拍）退 101：「test result: FAILED. 2 passed; 2 failed」，红的是 `todays_value_wins_when_the_records_run_past_every_ring_slot_passed` 与 `window_value_equals_the_registration_definition_at_every_slots_per_region_and_both_ends_of_u64`。
- 主 agent 另核：`grep -n 'FirstTxgOfTheNewInstance::PastTheHighestRootAndRecordInTheRing' crates/singlefs-core/src/mount.rs` 六处字面量在第 9398、9425、9450、9478、9556、9597 行；`crates/mutations.tsv` 里「产品入口走甲并全零」那一行的原文接 `before_the_one_reread,`，只命中第 9398 行那一处；第 9556 行（`mount_writable_with_background_compute_settings_and_the_gc_cpu_compensation_threshold`）与第 9597 行（`mount_writable_with_background_compute_settings_the_gc_cpu_compensation_threshold_and_compaction_switch`）两条后台计算产品入口都没有变异行，第 9597 行攻方没点到。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| Q1.1 甲两档与登记第 41、42 行逐句对 | **一致**（正推，读码；攻方 9255 对拍 388674 格 0 处不同） | 第一节 9255 |
| Q1.1 N「与 `highest_root_txg` 是同一个量」 | **字面打中（攻方，量的）**：代码在开关自己那一遍读数里取 N 与读坏槽（`crates/singlefs-core/src/mount.rs:1569`），今天的值另读一遍；挂载期间读故障变了时两者不同（9255 `TRANSIENT_PASS registration_reading=Ok(22) code_reading=Ok(11)`）。E205 的故障在挂载期间不变（正推 D1 推的，攻方读码），E205 读数不受影响。正推把这一句判「一致、两遍相同是推的」，攻方量出不同，取攻方 | 第一节 9255 |
| Q1.1 登记没钉死的三处（读哪一遍、N 空取 0、走出 u64） | **条款没说**（正推）；第一处即上一格 | 正推 D1–D3 |
| Q2.1 四种边角 | **没打中**：S = 5、6、7 与 N 贴 u64 顶端在对拍里与登记逐格相同；全部槽读坏时读阶段先拒，开关那一步不执行（`crates/singlefs-core/src/mount.rs:9214`；9257 三档都是 `refused=Recovery(NoValidRoot)`） | 第一节 9255 |
| Q2.1 只有第 0 代根时的「N 自己那一槽」 | **登记的缝（攻方，读码）**：N = 0 时代码与登记都把「N 自己那一槽」取成 `target_for_publish(0)` = 区域 0 槽 0，而第 0 代根三份同字节、读出 N 的可能是区域 1、2 的槽 0；后果只在多处读坏下出现（推的），在 C971（多处读坏时可写挂载写出倒挂的新实例根） 的承诺外那一形里 | 攻方第一节 |
| Q1.2 产品入口恒走今天那一档、产品行为不变 | **一致**（正推，读码逐层）；攻方 Q2.2 同判 | 正推 Q1.2；攻方第二节 |
| Q2.2 开关中途换档或漏出写行 | **没打中** | 攻方第二节 |
| Q1.3 清扫改法与 D16（发布语义） 已定项 6 逐句对 | **一致**（正推）；「恢复与挂载读根环时都调」字面没点清扫，kb 名单要补 | 正推 Q1.3 |
| Q1.3 错位读与 `recovery::read_root_ring_slot` 同形处置 | **一致**（正推）；重读一次的出处是 D18（块里携带什么信息） 已定项 11 第 318 行 | 正推 Q1.3 |
| Q2.3 无故障的合法历史少删或多拒 | **没打中**（攻方读码排除合法写者） | 攻方第三节 |
| Q2.3 错位读回第 0 代根 | **打中（攻方，副本上量的）**：实例 1 唯一的根在区域 1 槽 0，一次错位读回区域 0 槽 0 的第 0 代根，落点核按 `crates/singlefs-core/src/root_ring.rs:190` 认第 0 代根住任一个槽 0，照收，实例 1 那一行判成要删（9256）。这正是 C973 验收「某槽的真根被换成别槽自证过的根，被换掉的是该实例在环里唯一的根，清扫那一遍必须不删那一行」要拦的盘面；C973 的改法在这一格上不起作用，C973 不销账。射程：产品阈值 365 行下够不着是推的；直接判清扫那一步、只供测试的阈值与后台入口 `sweep_the_instance_table_while_mounted` 够得着 | 第一节 9256 |
| Q2.4 `crates/singlefs-core/src/mount.rs:1652` 的「max(今天的值, …)」 | **打中（量的）**：没有变异行罩它；去掉之后原有三条单元用例（9258）与原有 12 条端到端用例（9283）全绿，攻方新加的用例红（9283、9260） | 第一节 9258、9283、9260 |
| Q2.4 后台计算产品入口递今天那一档 | **打中（量的，grep）**：第 9556、9597 行两处都没有变异行 | 第一节主 agent 另核 |
| Q2.4 N 含不含被抛弃的根、端到端 S 只取 4、8、16 | **认（攻方，读码）**：前者没有用例罩；后者攻方加了 5、6、7（9257 绿） | 攻方第四节 |
| Q3 core 里读根环而不调落点核的地方 | **没有该核而没核的**：四处里三处核落点；`crates/singlefs-core/src/recovery.rs:5054` 拿读出的字节与已知根整条比，错位读只能读回与它相同的根，不用核（攻方判，主 agent 现读坐实；攻方「没有调用者」一句不成立） | 第一节 |
| 正文第 27 行「取 N = 读得出的根里最小的 txg」 | **正文写反**（两腿一致）：代码 `crates/singlefs-core/src/mount.rs:1576` 与登记都是最大；正文这一句作废，按代码 | 两腿 |
| `crates/singlefs-core/src/mount.rs:2991` 注释 | **过期**（正推）：清扫不调 `read_root_ring_slot` | 正推 Q1.3 |

## 三、交用户的

无岔路。Q2.3 的改法照 C973 已写死的验收句选（第五节），不动格式；格式改法（第 0 代根带区域号）不取，理由是验收句不需要它。

## 四、被攻过零轮的

第五节里 C973 补的那一条改法（攻方第三节「改法甲」）、「max(今天的值, …)」与两条后台计算入口补的用例与变异、登记第十四次修订。下一次代码轮攻这一批。

## 五、去向

| 事 | 去向 |
|---|---|
| C973 错位读回第 0 代根 | 并进下一批 `implementation-writer`：清扫读到某个区域 r 的槽 0 住着第 0 代根，而环里已有落点公式会写进这一槽、txg 更大的根时，算这一槽自证不过（走原有重读一次、仍不过这一轮不删）；攻方副本用例 `a_misplaced_read_returning_the_genesis_root_over_the_only_ring_root_of_an_instance_still_judges_its_row_to_delete` 改判成不删、去掉末尾读次数那一句后并进 `misplaced_ring_reads_tests`，阳性对照一并带上；配一行变异。C973 照旧开着，那一行的「怎么拦」与去向写回时改成这一条 |
| 「max(今天的值, …)」没罩 | 同一批：攻方副本里的单元用例与端到端用例（`todays_value_wins_when_the_records_run_past_every_ring_slot_passed`、`writable_mount_of_the_starting_image_with_records_past_every_ring_slot_read_bad_keeps_todays_value_past_read_bad`）并进主工作区，加一行变异「去掉 max(今天的值)」 |
| 两条后台计算产品入口 | 同一批：第 9556、9597 行各一条经该入口的用例与一行变异 |
| 攻方其余用例（窗口对拍、S 不是 2 的幂、只有第 0 代根、整环读坏） | 同一批并进（9257 绿） |
| N 取自哪一遍、N = 0 的槽 | E205 跑前登记第十四次修订（产物跑之前）：写明 N 与读坏槽取自开关自己那一遍读数、与 `highest_root_txg` 在故障不变时相同；N = 0 时「N 自己那一槽」取区域 0 槽 0。原句保留 |
| D16 已定项 6 名单没点清扫 | 写回批：「恢复与挂载读根环时都调」补上清扫（三语） |
| `crates/singlefs-core/src/mount.rs:2991` 注释 | 同一批实现员改成现状 |
| 正文第 27 行 | 不改（冻结证据），本判决记作废 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 6 | 备料 | 2026-11-06 不受影响：落点核口径不动，只补名单里「清扫」一词，写回批做 |
| D18（块里携带什么信息） 已定项 11 | 不影响 | 2026-11-06 不受影响：清扫「重读一次、仍不过这次不删」照旧，补的改法只多认一种「自证不过」 |
| D22（单元原子性怎么合成） 已定项 8 | 不影响 | 2026-11-06 不受影响：第 0 代根种进全部根环区域照旧，不取改格式的那条改法 |
