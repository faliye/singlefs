# m2-checker-supp-code-r1 核查员报告

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

时区：本机时钟 UTC，人在东京 UTC+9。核验时间 2026-09-21 UTC 当天。

## 0. 判别力自证

取样：Opus 报告 `crates/singlefs-core/src/mount.rs:691` 的 grep 输出行「`PublishPlan {`」。
把引用行号 691 加 1 变成 692，在草稿副本上核：

```
$ sed -n '691p' /tmp/claude-1000/m2-checker-supp-r1-verifier/selftest/mount.rs
            PublishPlan {
$ sed -n '692p' /tmp/claude-1000/m2-checker-supp-r1-verifier/selftest/mount.rs
                txg: CheckpointTxg(current.root.checkpoint_txg.0 + 1),
```

691 行原文与引用一致，692 行与引用（`PublishPlan {`）不一致 ⇒ **判 ✗**。核查方法能分辨行号错位，自证通过。

## 1. 输入核对

- 三条腿报告路径与 sha256 均按派发提示核过：
  Opus `m2-checker-supp-code-r1-opus-output.md`（348 行）、
  Sonnet `m2-checker-supp-code-r1-sonnet-output.md`（220 行）、
  本地攻方样本/核对表/运行记录三份。
- 开工快照 `research/prompts/m2-checker-supp-code-r1-start-snapshot.sha256`（9 个文件）核过：
  除 `crates/singlefs-checker/src/walk.rs` 外全部与当前主树 `sha256sum` 一致；`walk.rs` 与快照不同，
  与主 agent 披露的「事后改了 `TotalOrderKey::UndecidedForDataUnit` 上方那段注释」一致（处理方法见第 2 节）。
- 云端腿产物 sha256：`m2-checker-supp-code-r1-opus-model/` 三个文件当前 `sha256sum` 与 Opus 报告第 14-16 行、
  与目录本身完全一致（`e14bd300…`／`b93285f6…`／`8d7ad9e9…`）。

- 复核：两份云端腿报告文件当前 `wc -l` 与 `sha256sum` 与派发提示给的行数/哈希逐一相符（348 行 `d77e1c47…`；
  220 行 `151b9f50…`），说明交回之后未被改动。

## 2. walk.rs 已知差异的处理方法（对下面所有 walk.rs 引用统一适用）

`sha256sum -c` 核实：9 个快照文件中只有 `walk.rs` 与快照不同，其余 8 个逐字节相同（含 `image.rs`、
`transaction.rs`、`mount.rs`、`recovery.rs`、`invariants.md`、`checks-owed.md`、`decisions/23`、`_body.md`）——
对这 8 个文件的引用，核对当前主树等价于核对腿开工时的快照。

`walk.rs` 按主 agent 披露只改了 `TotalOrderKey::UndecidedForDataUnit` 上方那段注释一处。三条腿的报告都是在
这处编辑**之前**写的（都用同一份开工快照核过 sha256），所以三条腿引用 `walk.rs` 中该编辑点**之后**位置的行号，
在当前主树里应统一后移。逐一核对以下独立锚点，全部得到 **+3** 的一致偏移，判定这不是腿的引用错误，
是已知差异的必然结果，按派发提示口径不计 ✗：

| 腿引用的行号（编辑前） | 当前主树实际行号 | 锚点内容 |
|---|---|---|
| walk.rs:1441（Sonnet） | 1444 | 「『类身份段全部字段含写序』去掉载荷校验和那一段：同一份内容的两个副本在这上面逐字节相同。」 |
| walk.rs:1550（Sonnet） | 1553 | I-1.8 函数级文档注释「码 1 / 3 的可读已发布单元按…」 |
| walk.rs:1608-1610（Sonnet） | 1611-1613 | `if members[0].total_order_key == TotalOrderKey::UndecidedForDataUnit { continue; }` |
| walk.rs:1658-1681（Sonnet，`judge_root_ring_health`） | 实际 fn 声明在 1661 | `fn judge_root_ring_health(...)` |
| walk.rs:1687（Sonnet，`FIRST_JOURNAL_COUNTER`） | 1690 | `const FIRST_JOURNAL_COUNTER: u64 = 1;` |
| walk.rs:1735-1738（Sonnet，三分支穷举） | 1738-1741 | 「① 计数器 == 1 ⇒ …②…③…」逐字一致 |
| walk.rs:1739-1740（Sonnet） | 1742-1743 | 「计数器 − 1 那一条读不出…这一条判不了，跳过。」逐字一致 |
| walk.rs:1741-1781（Sonnet，`judge_journal_back_chain`） | 实际 fn 声明在 1744 | `fn judge_journal_back_chain(` |
| walk.rs:1752（Sonnet） | 1755 | `let expected_back_chain = if *counter == FIRST_JOURNAL_COUNTER {` |
| walk.rs:1759（Sonnet） | 1762 | `Some(_record_of_another_instance) => Some(0),` |
| walk.rs:1796-1800（本地攻方 Fact 2） | 1799-1803 | 「一块盘的两个系统配置槽都无效时不早退……仍欠在 C461。」逐字一致 |
| walk.rs:1801（本地攻方 Fact 1） | 1804 | `if chosen.is_empty() {` |

## 3. 云端攻方腿（Opus）核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| L34/51-58：`PublishPlan` 核心 5 处（mount.rs 691/746/774、transaction.rs 1200/1244） | ✓ 逐行一致 | `grep -rnE "(^\|[^a-zA-Z_])PublishPlan \{" crates/singlefs-core/src/` |
| L61-62：测试里另 3 处（`…accounting_node_full.rs:72`、`…row_publish_admission.rs:79`、`…second_instance.rs:450`） | ✓ 一致，全仓共 8 处 | `grep -rnE "(^\|[^a-zA-Z_])PublishPlan \{" crates/ --include=*.rs \| wc -l` → 8 |
| L63-64：「两处写死 0」（前者 txg3/jsn3/事务号1，后者「新实例（2）自己的第一条记录…」） | ✓ 指的是 `highest_transaction_number_before_this_publish` 字段：`accounting_node_full.rs:72` 用 `FIRST_TRANSACTION_NUMBER=1`/`FIRST_TRANSACTION_TXG=3`（都现查过常量定义），`second_instance.rs:450` 注释逐字相符；`row_publish_admission.rs:79` 该字段是动态值，不在「写死 0」之列 | `grep -n "pub const FIRST_TRANSACTION_NUMBER" crates/singlefs-core/src/transaction.rs`；`sed -n '445,460p' second_transaction_step_three_second_instance.rs` |
| L66-72：transaction.rs:2190-2192 代码块 | ✓ 逐字一致 | `sed -n '2190,2192p' crates/singlefs-core/src/transaction.rs` |
| L74：`ZeroUnitPublishPlan` 定义与 3 处构造（transaction.rs:512、mount.rs:800/1063、transaction.rs:488） | ✓ 全部一致 | `grep -rnE "ZeroUnitPublishPlan \{" crates/ --include=*.rs` |
| L83：transaction.rs:2143 持久顺序注释 | ✓ 一致（转述非逐字引号，语义相符） | `sed -n '2143p' crates/singlefs-core/src/transaction.rs` |
| L96-98：D23 决策 `:196` 已定项 7 整行 | ✓ 逐字一致 | `sed -n '196p' ".claude/kb/decisions/23-journal的角色与格式.md"` |
| L100-102：`checks-owed.md:336`（C381）「代码那一半整个仍欠…」 | ✓ 子串命中 | `sed -n '336p' .claude/kb/checks-owed.md` |
| L133-137：K1-d 核对表 5 行（mount.rs:229/1144/1272/1171/1305/1017、mount.rs:749-751 引用） | ✓ 全部一致，含 `mount.rs:749-751` 逐字引用 | 逐行 `grep -n`/`sed -n` |
| L136：`grep -rn 实例切换 crates/` 零命中 | ✓ | 现查 0 |
| L150-154：mount.rs:1135 注释引用 | ✓ 逐字一致 | `sed -n '1135p' crates/singlefs-core/src/mount.rs` |
| L156：recovery.rs:709 取 `record.transaction`；:710 `TransactionOutput` | ✓ 均一致 | `sed -n '705,712p' crates/singlefs-core/src/recovery.rs` |
| L163-164：`checks-owed.md:254`（C286） | ✓ 子串命中 | `sed -n '254p' .claude/kb/checks-owed.md` |
| L167-168：`pub fn mount` 只有 mount.rs:1118/1196；`remount` 26 处 | ✓ 均一致 | `grep -n "pub fn mount" …`；`grep -rn remount crates/ --include=*.rs \| wc -l` → 26 |
| L180-188：recovery.rs:957-958 代码块；:895-899（第 898 行过滤条件） | ✓ 逐字一致 | `sed -n '957,958p'`/`sed -n '895,899p'` |
| L188：`grep -c maximum_applied_transaction …diff.md` 交回 0、退出码 1 | ✓ 一致 | 现查同一命令 |
| L212-220：判别力自证输出（改回旧写法 8→7、`(3,1)` 重复、assertion 数值） | ✓ **独立复跑重现，见第 6 节** | 见第 6 节 |
| L222-235：walk.rs:1430-1431 引用「那条脚本上真有一对」 | ✓ **逐字确认**：Sonnet 遗留在 `/tmp/claude-1000/m2-checker-supp-r1-sonnet/repo/` 的副本仍保留编辑前的 `walk.rs`，1430-1431 行与 Opus 引文逐字节相同，行号也一致（不需要 +3 折算，说明两条腿看到的是同一份编辑前文本） | `sed -n "1430,1431p" /tmp/claude-1000/m2-checker-supp-r1-sonnet/repo/crates/singlefs-checker/src/walk.rs` |
| L243-245：decisions/23:458 已定项 19① 整行 | ✓ 逐字一致 | `sed -n '458p'` |
| L249-252：transaction.rs:547 `is_commit: true` | ✓ 一致 | `sed -n '547p'` |
| L256-266：K2-d 表（recovery.rs:501、walk.rs:1414、mount.rs:973/980/1168/1302、invariants.md:25、walk.rs:1393-1394） | 前 5 项 ✓；`walk.rs:1393-1394` **✗ 位置偏差**：真正含「不替它报」的注释在 1394-1395，1393 行是 `impl PublishedPredicate<'_> {`，与该转述内容无关（此处在已知编辑点之前，不属于第 2 节的位移，是独立的一行范围写窄） | `grep -n "判损坏归 I-1.2，这一条不替它报" crates/singlefs-checker/src/walk.rs` → 1395 |
| L268：invariants.md:25 I-1.2 状态列 | ✓ 子串命中 | `sed -n '25p' .claude/kb/invariants.md` |
| L274-280：model.rs 四处 W=0（1008/1055/1086/2006）+ 1007 注释；两处非 0 断言 | ✓ **全部独立复核，见第 6 节** | 见第 6 节 |
| L279：新用例名 | ✓ 存在，且在 `mutations.tsv:221` 对应同一条变异 | `grep -rn "the_transaction_number_keeps_counting…" crates/` |
| L14-16：产物 3 个文件 sha256 | ✓ 与目录当前哈希一致 | `sha256sum research/prompts/m2-checker-supp-code-r1-opus-model/*` |

**Opus 计数**：核 30 处，✓ 29，✗ 1（walk.rs:1393-1394 范围偏 1 行）。

## 4. 云端正推腿（Sonnet）核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| invariants.md:52（I-7.3 判据原文整行） | ✓ 逐字一致 | `sed -n '52p' .claude/kb/invariants.md` |
| walk.rs:1658-1681（`judge_root_ring_health`，代码块） | ✓ 内容一致（含已知 +3 偏移，见第 2 节）；代码块本身是重排版（把多行 rustfmt 链式调用合并、用 `{ ... }` 显式省略闭包体），不是逐字节抄写，但省略已标注、语义与当前实现相符 | `sed -n '1661,1681p' crates/singlefs-checker/src/walk.rs` |
| checker_known_bad_images.rs:948-1003（阳性对照测试，干净/坏镜像细节） | ✓ 独立复跑通过，见下 | 见复跑命令 |
| 复跑：`the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs` | ✓ **独立复跑，`... ok`，与报告 L59 一致** | `cd /tmp/…/repo-sonnet-checks && cargo test -p singlefs-harness --test checker_known_bad_images the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs -- --nocapture` |
| invariants.md:81（I-8.6 判据原文整行） | ✓ 逐字一致 | `sed -n '81p' .claude/kb/invariants.md` |
| walk.rs:1741-1781（`judge_journal_back_chain`，三分支 match） | ✓ 内容一致（+3 偏移） | `sed -n '1744,1784p'` |
| walk.rs:1687/1735-1738/1739-1740/1752/1759 | ✓ 全部一致（+3 偏移，见第 2 节表） | 逐条 `sed -n`/`grep -n` |
| 复跑：`first_transaction_step_seven_layer0` 全量 262165 状态（release, --ignored） | ✓ **独立复跑，`I-1.8=262165/0`、`I-7.3=262165/0`、`I-8.6=262161/0`，262165-262161=4，与报告 L103-105 逐字一致** | `cargo test --release -p singlefs-harness --test first_transaction_step_seven_layer0 layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations -- --ignored --nocapture` |
| invariants.md:31（I-1.8 判据原文整行） | ✓ 逐字一致（与本地攻方 Fact 13 引同一行，互相印证） | `sed -n '31p' .claude/kb/invariants.md` |
| decisions/18:168,171-177（D18 已定项 7 码 1 字段表，含载荷 CRC 一行） | ✓ 逐字一致 | `sed -n '168p;171,177p' .claude/kb/decisions/18-块里携带什么信息.md` |
| decisions/18:271,274-286（D18 已定项 11 码 3 字段表，含载荷校验和一行） | ✓ 逐字一致 | `sed -n '271p;274,286p' 同上` |
| walk.rs:1327-1370（`ContentUnitHeaderOffsets` 结构体，编辑点之前，无偏移） | ✓ 逐字一致，含 42..101/105..105（码1）与 42..89/93..107（码3）四个偏移量 | `sed -n '1327,1370p' crates/singlefs-checker/src/walk.rs` |
| walk.rs:1335-1336（归并键排除注释） | ✓ 逐字一致 | `sed -n '1335,1336p'` |
| walk.rs:1441→1444、1550→1553、1608-1610→1611-1613（第 3 节代码/注释引用） | ✓ 内容一致，均为已知 +3 偏移 | 见第 2 节表 |
| walk.rs:1423-1431（`TotalOrderKey::UndecidedForDataUnit` 定义与引用位置说明） | 该段落落在已知编辑区间内/紧邻，当前主树该处已改写，不再具备逐字比对意义；不计 ✗（第 2 节口径） | — |
| 第 4 节反例（改归并键字面纳入载荷校验和后 Holds/Violated 对照，含具体 hex 校验和值） | ✓ **主 agent 点名独立重做，完全重现，见第 6 节** | 见第 6 节 |
| checker_known_bad_images.rs:706-713 / :684-701（坏镜像构造细节） | ✓ 一致 | `sed -n '684,701p'` |
| L196-198：「清理」小节 `diff … \| wc -l` = 9 | **分不清（非 ✗）**：`/tmp/claude-1000/m2-checker-supp-r1-sonnet/` 副本其实还在（派发提示说「可能没了」，现查未删）。今天重跑同一条命令得到 18，不是 9；拆开看两处差异——① 归并键改动已按 Sonnet 所述清理干净，`grep -c "42\\.\\.105\\|42\\.\\.107" 该副本/crates/singlefs-checker/src/walk.rs` 为 0，与主树一致；② 差异全部落在两处：a) walk.rs:1430-1434 一处（正是本表上一行核到的、编辑前后的注释差异，与 Sonnet 无关）；b) walk.rs:1615-1622 一段 `SINGLEFS_DEBUG_I18` 调试 `eprintln!`（Sonnet 自己加的，与其判定一览最后一行「object_key_groups=2, candidates_with_2plus_versions=1」的量法吻合）。9 这个数是 Sonnet 收尾时（主 agent 编辑 walk.rs 之前）量的，那一刻主树与它的副本在编辑点上还相同，只差调试代码；今天多出的 9 行差额=主 agent 事后编辑贡献的净行数，与 Sonnet 报告无关，不计 ✗ | `diff /tmp/claude-1000/m2-checker-supp-r1-sonnet/repo/crates/singlefs-checker/src/walk.rs /home/fy5090/code/singlefs/crates/singlefs-checker/src/walk.rs \| wc -l` → 18 |

**Sonnet 计数**：核 19 处，✓ 19，分不清 1（清理小节的 `diff \| wc -l` 数值随主 agent 事后编辑而变化，非 Sonnet 报告本身的错误，见上表说明）。

## 5. 本地攻方腿核对表

### 5a. 转述核对表（16 条 Fact 的「原文文件:行」）

| Fact | 原文文件:行 | 核的结果 |
|---|---|---|
| 1 | diff.md:580、walk.rs:1801（本表按 +3 折算=1804）、body.md:34 | ✓ 三处均逐字/逐条一致 |
| 2 | walk.rs:1796-1800（+3=1799-1803） | ✓ 逐字一致 |
| 3 | recovery.rs:230-231、118-127 | ✓ 逐字一致（枚举字段：`NoValidSystemConfiguration` 带 `DeviceIdentity`，`SystemConfigurationsDisagree` 不带字段） |
| 4 | recovery.rs:238-263、285-293 | ✓ 一致，`chosen = Some(...)` 确认只在 274 行赋值一次 |
| 5 | recovery.rs:273-283 | ✓ 逐字一致 |
| 6 | walk.rs:1810-1822（+3=1813-1825）、grep device_count | ✓ 一致，checker 侧 `device_count` 零命中（仅 `lib.rs:122/182`） |
| 7 | walk.rs:665-686（编辑点之前，无偏移）、:685 | ✓ 逐字一致，685 行原文与核对表引文完全相同 |
| 8 | system_configuration_per_device_redundancy.rs:273-322，含 269-271 说明注释 | ✓ 全部一致 |
| 9 | invariants.md:52 | ✓ 逐字一致（与 Sonnet 引同一行，互证） |
| 10 | walk.rs:1653-1657、1658-1677（+3 后=1656-1660、1661-1680） | ✓ 一致 |
| 11 | invariants.md:81 | ✓ 逐字一致 |
| 12 | walk.rs:1732-1739、1741-1772（+3 后=1735-1742、1744-1775） | ✓ 一致 |
| 13 | invariants.md:31 | ✓ 逐字一致 |
| 14 | walk.rs:1335、1441(+3=1444)、1550(+3=1553)、1337-1338/1359-1360/1369-1370（编辑点前，无偏移） | ✓ 全部一致 |
| 15 | decisions/18:168、171-177 | ✓ 逐字一致（载荷 CRC 一行原文照抄） |
| 16 | decisions/18:271、274-286 | ✓ 逐字一致（载荷校验和一行原文照抄，与首稿改写后的因果关系表述吻合） |

### 5b. ⚠️ 自报现查（D18 载荷校验和列为类身份段字段、实现却排除）——两处出处核过

- Fact 15 出处 `decisions/18:177`：载荷 CRC 一行——✓ 已在第 5a 表核过，原文逐字含「只加在码 1 自己的类身份段」。
- Fact 16 出处 `decisions/18:284`：载荷校验和一行——✓ 已核过。
- 与实现的对照：`walk.rs:1359-1360`（码 1 `merge_key_before_payload_checksum: 42..101`，跳过 `101..105`）、
  `walk.rs:1369-1370`（码 3 `42..89`+`93..107`，跳过 `89..93`）——✓ 两处均现查一致，载荷字段确实被排除在归并键之外。
  这与 Sonnet 第 3、4 节的独立结论（同一处代码、同一份 D18 定义）完全吻合，两条腿互不知道对方产出（K3 归 Sonnet、
  K4/K3 转述归本地攻方），结论一致但推导路径独立，不算重复。

### 5c. 命令与输出复跑

| 命令 | 核对表给的输出 | 我重跑的输出 | 结果 |
|---|---|---|---|
| `grep -n "device_count" crates/singlefs-checker/src/*.rs crates/singlefs-core/src/recovery.rs` | 3 行（recovery.rs:278/1196/1211） | 同样 3 行，内容逐字相同 | ✓ |
| `sed -n '118,127p' crates/singlefs-core/src/recovery.rs` | 枚举两个变体 | 逐字相同 | ✓ |
| `grep -n "chosen.is_empty() \|\| chosen.len()\|if chosen.is_empty() {" research/prompts/_m2-checker-supp-code-r1-diff.md` | `580:-...`、`586:+...` | 逐字相同 | ✓ |

### 5d. 运行记录（runlog）核对

| 声明 | 复核结果 | 命令 |
|---|---|---|
| s1 词数 407、s2 词数 188 | ✓ 精确一致 | `wc -w m2-checker-supp-code-r1-local-attack-output-s1.md m2-checker-supp-code-r1-local-attack-output-s2.md` |
| s1 oov-check 绿，生词=2（`checker's`），拼接=0 | ✓ 精确一致 | `python3 research/scripts/oov-check.py …-s1.md` |
| s2 oov-check 绿，生词=1，拼接=0 | ✓ 精确一致 | 同上 -s2.md |
| s1 corruption-check 绿，`cjk=0 words=387 fffd=0` 各项 0 | ✓ 精确一致 | `python3 research/scripts/corruption-check.py …-s1.md` |
| s2 corruption-check 绿，`cjk=0 words=183 fffd=0` 各项 0 | ✓ 精确一致 | 同上 -s2.md |
| 没有 void 副本、目录下只有 5 个文件 | ✓ 一致 | `ls research/prompts/ \| grep m2-checker-supp-code-r1-local-attack` |
| 两份样本 5 道 Judgment 各给 verdict/justification/refuted-by 三项、无缺词断句 | ✓ 通读一致，`local-attack.md` 里的 16 条 Fact 编号与措辞与两份样本引用的 Fact 编号一一对应 | 通读 `s1.md`、`s2.md`、`local-attack.md` |

**本地攻方计数**：核 16（Fact）+ 2（现查出处）+ 3（命令）+ 6（runlog 声明）= 27 处，✓ 27，✗ 0。

## 6. 主 agent 点名要独立重核的四处

### 6.1 正推腿第 4 节反例（D18 字面归并键 vs 今天的排除写法）

在**新副本**（`rsync -a --exclude target --exclude .git … /tmp/claude-1000/m2-checker-supp-r1-verifier/repo-sonnet-redo/`，
开工 `sha256sum -c` 核过，仅 `walk.rs` 与快照不同、与第 2 节已知差异一致）上独立重做，自己写测试
（未借用 Sonnet 任何残留文件），追加到 `checker_known_bad_images.rs` 末尾，用同一个
`mutate_one_device_copy_of_unit(&mut image, &UNITS, 1, DATA_UNIT, |bytes| { bytes[200] ^= 0xFF; })`：

- **今天实际排除写法**（未改代码）：`cargo test -p singlefs-harness --test checker_known_bad_images
  verifier_redo_debug_i18_literal_merge_key_misses_data_unit_payload_mismatch -- --nocapture` →
  `verdict=Violated("归并成一组的成员载荷不同（盘 0 槽 50180 的码 1 数据单元 载荷校验和 0x73e0742b、
  盘 1 槽 50180 的码 1 数据单元 载荷校验和 0x8408a295）：…")` —— **与 Sonnet 报告 L186 逐字节相同，
  连十六进制校验和都相同**（同一 `build_pool` 标签给出确定性内容）。
- 按 D18 字面把载荷校验和纳入归并键（`merge_key_before_payload_checksum: 42..105`/`105..105`；
  码 3：`42..107`/`107..107`，与 Sonnet 报告 L146-149 描述的补丁一致，逐行核过）后重跑同一条测试：
  `verdict=Holds` —— **与 Sonnet 报告 L178 逐字一致**。
- 顺带重跑 `the_merge_and_the_back_chain_bad_images_redden_only_their_own_invariant`（码 3 的同构攻击），
  在字面归并键下仍 `ok`——与 Sonnet「② 侥幸兜住码 3」的结论一致。

**判定：两个方向均独立复现，逐字/逐值吻合，含具体的十六进制校验和。**

### 6.2 Opus 的 K1-c（根槽失败重试拿同一个事务号，及「改回旧写法同样中」）

用 Opus 模型目录的 `opus_probe_txn_chain.rs` 原样放入新副本（`sha256sum` 与产物目录一致）独立复跑
`cargo test -p singlefs-harness --test opus_probe_txn_chain -- --nocapture`：3 个测试全部 `ok`，
`sweeping_the_failure_point_and_the_number_of_publishes_before_it` 打印的 8 行输出与 Opus 报告 L321-328
**逐字节相同**（8 组「失败前发过 N 次 / 失败点 … / 环里那条记录的事务号 … / 重试拿到 …」）。

「改回旧写法同样中」这句本身**在探针文件里没有对应的 A/B 测试**（`opus_probe_txn_chain.rs` 里不含
`previous.record.transaction` 字样，未直接跑过旧写法）。我用代码现查坐实了它背后的机制性前提：
`transaction.rs` 差异行（`_m2-checker-supp-code-r1-diff.md:839-840`）显示改动只把
`transaction: previous.record.transaction + 1` 换成 `previous.highest_transaction_number_in_this_instance + 1`；
两者都是 `previous`（调用方持有的旧版本）的字段，而失败返回路径不更新调用方手里的 `previous`
（`publish_overwrite` 签名里 `previous: &TransactionOutput` 只读，`Err` 分支不产出新版本）——
所以无论用哪个字段，两次调用算出的都是同一个值。**这句结论本身可信，但它是推理坐实、不是探针里的实测对照**，
与 Opus 自己在同一张表里给其它格标注「推的 / 量的」的做法不一致（这一格没标）。

### 6.3 Opus 的 K2-b（判别力自证）与 K2-e（`model.rs` 里 W 是否写死 0）

K2-b：把 `step_five_write_order_scan.rs.append` 追加到新副本的 `second_transaction_step_five_reuse.rs` 末尾，
`cargo test -p singlefs-harness --test second_transaction_step_five_reuse opus_probe -- --nocapture`：

- 今天写法：`盘 0 上码 1 单元 8 个，写序去重之后 8 个：[(1, 1), (1, 2), (2, 1), (3, 1), (3, 2), (3, 3), (3, 4), (3, 5)]`
  —— **与 Opus 报告 L208-209 逐字节相同**。
- 判别力自证：把 `transaction.rs:1247` 改回 `transaction: previous.record.transaction + 1,`，
  同一条测试红：`写序去重之后 7 个：[(1, 1), (1, 2), (2, 1), (3, 1), (3, 1), (3, 2), (3, 3), (3, 4)]`，
  `assertion left == right failed … left: 7 right: 8` —— **与 Opus 报告 L215-219 逐字节相同**。

K2-e：`grep -n "applied_transaction_high_water:" crates/singlefs-harness/src/model.rs` 现查确认恰好 4 处
（1008/1055/1086/2006）全部 `= 0`；1007 行注释「W = 这次恢复施加的记录里最大的事务号（D23（journal 的角色与格式）
已定项 14 第 4 条）：不建崩溃，一条都不施加。」与 Opus 报告逐字一致；`grep -rn applied_transaction_high_water
crates/singlefs-harness/tests/*.rs | grep -v ": 0"` 只命中两处非 0 赋值（`second_transaction_step_zero_layer0.rs:782`、
`second_transaction_step_three_second_instance.rs:401`，都是 `: 3`），与 Opus 报告一致。

**判定：K2-b 与 K2-e 均独立复现，数值逐字吻合。**

### 6.4 本地攻方的转述核对表（16 条 Fact 全核，见第 5a/5b 节）

见上文第 5a 节（16 条 Fact 全部 ✓）与 5b 节（⚠️ 自报现查的两处出处，`decisions/18:177`、`:284`，均逐字一致，
与实现排除载荷字段的事实吻合）。

## 7. 总计数

| 腿 | 核了几处 | ✓ | ✗ | 分不清/核不动 |
|---|---|---|---|---|
| Opus | 30 | 29 | 1（walk.rs:1393-1394 范围偏 1 行，实际含义段落在 1394-1395） | 0 |
| Sonnet | 19 | 19 | 0 | 1（清理小节 `diff \| wc -l`=9，今天重跑得 18，原因见第 4 节说明，非报告错误） |
| 本地攻方 | 27 | 27 | 0 | 0 |
| 主 agent 点名的 4 处独立重做 | 4 | 4（全部独立复现，含 6.2 一条推理型子结论已单独说明其证据形态） | 0 | 0 |
| **合计** | **80** | **79** | **1** | **1** |

复跑命令：全部 6 条实际执行的 `cargo test`（K1-c 探针 3 个子测试、K2-b 判别力自证前后两次、
Sonnet 反例两个方向、Sonnet I-7.3 阳性对照、Sonnet layer0 全量 262165 状态、码 3 侥幸兜底测试）
均在 `/tmp/claude-1000/m2-checker-supp-r1-verifier/` 下的三份独立副本（`repo`、`repo-sonnet-redo`、
`repo-sonnet-checks`）里跑，一律 `nice -n 19`，主工作区全程未写一个字节
（每份副本收尾前后各 `sha256sum -c` 开工快照核过，除 `walk.rs` 已知差异外全部 `OK`）。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑（K1-c「不分辨」那句的机制性前提
  我现查坐实了，但它是不是构成「打中/没打中」的判断本身留给主 agent）。
- 不判 C464（I-1.8 对码 1 的全序键写窄了）本身成不成立、Sonnet 提议的「归并键排除条款该不该写回
  `invariants.md`」该不该采纳——三条腿自己都写明这归主 agent。
- 不判三方是否一致、分歧点怎么处置——那是判决，不是核查。
- 没有跑 `cargo test --all`、没有跑门禁任何阶段（不在本轮核查员职责内）。
- Opus 报告「没打中的形状」表里三行标「推的/没实测」的没有单独复核（腿自己已如实标注未实测，
  按规则「没打中」一次不算，不需要核查员再验证一次没打中）。
- Sonnet 报告「没做什么」里列的几条（I-1.8 少字段方向未实测、I-8.6 隐含前提未核、跨 checkpoint
  覆盖率缺口未穷举）未去补做——腿自己已如实列出未做，不属于「核引用」范围。
- 没有单独复核 Opus 报告「打中之后先答的四句」表格里 K1-d、K2-b 两列的具体措辞是否精确对应
  `.claude/rules/three-way-inference.md`「判据自己也会写错」小节的四问格式本身（只核了它引用的行号
  与事实，没有核这套四问方法论应用得对不对——那属于推理层面）。
- 没有重新调用本地模型生成第三份样本——运行记录显示两份干净样本已达标、未撞停机线，不需要第三次抽样，
  按规则不应额外抽样制造判否定结论的假象。
