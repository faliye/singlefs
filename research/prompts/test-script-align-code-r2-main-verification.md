# test-script-align-code-r2 判决（2026-10-02，代码轮第二轮）

正文 `research/prompts/_test-script-align-code-r2-body.md`；腿：辩方 `research/prompts/test-script-align-code-r2-sonnet-output.md`（J5、J3、J4）、攻方 `research/prompts/test-script-align-code-r2-opus-output.md`（J1、J2、J3），模型目录各自 `-model/`。被复核的第一轮判决 `research/prompts/test-script-align-code-r1-main-verification.md`。

被判的 `crates/*/src/*.rs`：`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crates/singlefs-checker-tier/src/crash_identity.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/bad_disk_input.rs`、`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、`crates/singlefs-harness/src/history.rs`、`crates/singlefs-core/src/admission.rs`。

## 一、核查

- 开工快照：冻结副本 `/tmp/claude-1000/test-script-align-code-r2/tree/` 对 `research/prompts/test-script-align-code-r2-snapshot/crates-sha256.txt` `sha256sum -c` 全对。
- `cite-check.py`：辩方「核了 63 处引文，对上 63 处」；攻方「核了 19 处引文，对上 19 处」（主 agent 带 `--root .` 复跑，末行与腿报的一致）。
- 主 agent 在冻结副本上带 `SINGLEFS_HEAVY_TESTS=commit` 实跑 checker 档（这一批的 checker 档用例此前没人跑过：实现员定义不许跑，合入后验证只跑 harness 档）：
  - `cargo test --release -p singlefs-checker-tier --features verdict-store --test crash_amplification_two_flows_share_their_prefix`：退 101，7 条里 6 条红（第 322、811、908 行等），攻方 J1、J3 坐实。
  - `--features verdict-store,gpu --lib --test '*'`：35 个测试二进制 28 个全绿、16 条红。逐条对过来源：两流 6 条归这一批；`crash_identity::tests::crash_case_registration_…` 是冻结副本没拷 `.claude/` 造成的；`crash_enumeration_{findings_log_…,new_pool_file_creation_stream,sharded_across_processes}` 三条在 HEAD（`git archive HEAD`）上同样红（`scratchpad/r2-compare-head-tree.log`），不是这一批；GPU 判器五条（`the_gpu_judge_agrees_…three_flows`、`…sigma…`、`…tells_apart…past_the_thirty_second_write`、`a_planted_violated_bit_past_the_sixty_fourth…`、`the_card_stops_taking_blocks…`）单线程跑仍红、在 HEAD 上全绿，报错是「表越过了内核的容量」（恢复内核第 1 字 196608、checker 内核第 13 字 7171），这一批的补丁没动 wgsl 内核、`crash_judge_gpu.rs`、`crash_judge_tables.rs` 与核心层（`grep` 四份补丁现查），那几处是别的会话暂存的改动，不归这一批。
  - `cargo test --lib --tests` 会带上 `src/bin/` 装置的内联单测（e161 装置撞 24G 被 oom-kill），与 `.claude/rules/verification.md` 快档那一格「装置二进制 src/bin/ 的内联单测不在快档里」不符，记一笔。

## 二、判定与去向

| 格 | 判定 | 去向 |
|---|---|---|
| J1 第一个根槽写之前的点把整张版本表写进判器输入，前缀共享丢了 | 打中（实跑坐实 6 条红） | 改代码：够不着任何根的点按基镜像里的根截版本表（攻方改法 B0，零轮），或别的能让两流共享前缀而判器输入真不同时不共享的截法；验收：两流测试文件 7 条全绿、R1 的两个次序仍红在改前 |
| J3 M3 用例在基线上就红（那个点只有 1 态，145 种单字节改坏分不出） | 打中 | 改代码：重造 M3 那一对，使基线绿、第 1587 行变异红（攻方说只能红在直接比键那条断言上：就钉键与判定两条断言，写明哪条拦哪个变异）；第 1432–1434、1588、1680–1682 行点名的用例在修好 J1 后逐条实跑核基线绿 |
| J2 G1 没卡而 `/sys/bus/pci` 不在时判红 | 打中 | 改代码：`/sys/bus/pci` 整个不在（看不见 PCI 总线）报本次未跑；目录在而某个设备读不出照旧判红 |
| J5 `a_line_was_printed_at` 名不副实 | 采（辩方） | 改名：说它记的是收到消息的时刻 |
| J5 解析数值环境变量至少 9 份、C606 同形 `checking_threads()`（`crash_amplification.rs:1157`）与 `SINGLEFS_THREAD_CAP` 悄悄退回默认、`crash_amplification.rs:85` 与 `history.rs:4308` 同一件事两份 | 采 | 改代码：收成一份共用解析，写错一律 panic 带变量名与原值；线程数 0 的口径全仓一致 |
| J5 C611 层 0（`crash.rs:3282`）与 GPU 发射点没有站点级用例 | 采 | 补假时钟用例与变异 |
| J5 K5 库内两条同形用例（`crash_amplification.rs:2650`、`2745`）没守 | 采 | 走同一个环境守卫 |
| J4 `number_from_process_environment_panicking_when_unreadable`（`history.rs:4331`）没有调用方 | 采 | 删掉，或并进共用解析由调用方用上 |
| J3 `crash.rs:4178` 注释说门禁认不出 `should_panic`，而 `crates-mutation-rows.py:626` 认 | 采 | 改注释 |
| J3 C604 没留变异 | 部分采 | 留不出变异站得住；补一条与硬件无关的源码扫描用例（先例 `crash_amplification.rs` 第 2933 行起的 K5 用例），钉「GPU 用例都经同一个取卡入口」 |
| J4 宏挪进 `admission.rs` | 不采 | 不违「重复要生成」与 D17（实现分层与第三方管道），放哪可议不算打中 |
| J1 退成整段写表那一支没有用例或变异罩着 | 采（线索） | 补一条用例（造 txg 不严格递增的流）与变异 |

打中的改法被攻过零轮，第三轮攻改后的代码。**这一次合入之后由主 agent 在冻结副本上带前缀实跑这一批碰到的 checker 档测试文件，再取第三轮快照**。

## 三、交给实现员（按文件聚簇）

| 簇 | 关的格 | 要动的文件 |
|---|---|---|
| 崩溃放量 | J1、J3、J1 退成整段写表、`a_line_was_printed_at`、`crash_amplification.rs` 里的两处数值解析、C611 层 0 与 GPU 发射点、K5 库内两条、`crash.rs:4178` 注释 | `crates/singlefs-checker-tier/src/crash_amplification.rs`、`src/crash.rs`、`src/crash_identity.rs`（要动时）、`tests/crash_amplification_*.rs`、`tests/common_crash_points/mod.rs` |
| 解析与 GPU | J2 G1、共用解析收成一份（`history.rs` 那一份为准、删死代码）、`SINGLEFS_THREAD_CAP`、C604 源码扫描用例 | `crates/singlefs-harness/src/history.rs`、`SINGLEFS_THREAD_CAP` 所在文件、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs`、新加的源码扫描用例文件 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 9 | 不影响 | 2026-10-02 不受影响：这一轮判的是实现 |
| D17（实现分层与第三方管道） | 不影响 | 2026-10-02 不受影响：宏挪进核心层不违分层 |
