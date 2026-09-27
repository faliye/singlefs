# kb-scribe 写回报告（第二批）

## 规格

- `/tmp/claude-1000/kb-writeback-batch2/spec.json`（sha256 0f757eb1cfc54f1b962d74779ffbaf9d94c89c0ce8786b12365173a193fb3405，现核一致）
- `/tmp/claude-1000/kb-writeback-batch2/spec.md`（sha256 7c82a661fa6bb634d2e31d4c86985c316e5eaea00d415e0cda2ad24cbb024d1c，现核一致）
- 起草员报告：`research/prompts/m2-kb-writeback-batch2-drafter-report.md`（已读，「要主 agent 判的点」7 条与「没做什么」已知悉，未替它做判断）

## 写范围闸预检（10 个目标文件，写之前）

10 个目标路径逐个喂 `write-guard.sh`，检出记录指到 `/tmp/claude-1000/kb-writeback-batch2/scribe/precheck.jsonl`，10 条全部退出码 0，无一条被拒。

## replace-batch.py 原样输出

`--dry-run`：
```
✓ 24 处都命中得对（10 个文件），--dry-run 没写
```

实写：
```
✓ 24 处替换，写了 10 个文件（改名换上新 inode），回读一致
```

## 决策变更史

`.claude/kb/decisions-history/2026-09.md` 按规格追加「其三」「其四」两条原文（各带两行快查），
`其三`=D13（验证路线） 已定项 7，`其四`=D28（挂载期承诺量） 已定项 4；起草员报告核过当月文件已有其一、其二，这两条续编，写入时现查未撞号。

`bash .claude/gate.d/49-history-brief.sh`（不带 `--write`）先跑，只报「decisions-history.md 与按原文重新生成的不一致」一处，未列任何「（待补）」条目；`git diff` 只核出这一轮自己写的两条新条目，`decisions-history/` 下别处无改动、无别的会话未提交内容混入，随即跑 `--write`：

```
✓ 按原文重新生成了 .claude/kb/decisions-history.md：563 条条目，这次补了 0 条「（待补）」快查
```

`--write` 前后 `git diff -- .claude/kb/decisions-history.md` 只新增 D13、D28 两行（各自表格从「共改过 27/22 次」变「28/23 次」），被挤出「最近 3 次」窗口的旧行各 1 条（D13 已定项 9、D28 已定项 1「第九项」那条）——按设计只减不增，不是这一轮的内容丢失。

## 门禁阶段（kb-scribe 登记的 41 个阶段，逐个跑过）

```
$ awk -F'\t' -v me="kb-scribe" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == me) print $1 }' .claude/gate.d/stage-owners.tsv | wc -l
41
```

逐个 `nice -n 19 bash .claude/gate.d/<文件>`，原样末行汇总见 `research/prompts/kb-writeback-batch2-tmp-evidence/gate-summary.txt`（每个阶段的完整输出在同目录 `gate-out/<文件>.out`）。34 个绿或 77（本次未跑）；7 个红：27、33、43、52、69、75、92 号。归属逐个判过：

| 阶段 | 归属 | 判据 |
|---|---|---|
| 27-format-constants.sh | **这一批造成，已知后果** | 红在 `research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs:106`（`JOURNAL_NEW_ROOT_SEGMENT_BYTES` 写成算式）与 `e157_parallel_line_one_clauses.rs:74`（`JOURNAL_NAMED_ENTRIES_PER_RECORD` 写成算式），与派发提示预告的两处逐字一致；派发提示明写「由主 agent 另派实现员改，不归你」，未修 |
| 33-mutation-tables.sh | **不是这一批** | 红在 `crates/mutations.tsv:953` 对 `research/e158_root_choice_repair.rs` 的锚点，这两个文件都不在这份规格的改动范围里，`git status` 里它们是别的会话未提交的改动；未修 |
| 43-owed-table-shape.sh | **这一批造成** | 见下节「43 号：C570/C571 插入点落错表」；停下，未自行挪动 |
| 52-segment-registry.sh | **不是这一批** | 写批之前先跑 precheck 就已经报同样「共 2 处」（见下方 PostToolUse 提示原文），文本前后不变；点名的 `second_transaction_step_zero_layer0.rs`、`e142_first_transaction_dry_run.rs` 不在这份规格改动范围；未修 |
| 69-evidence-in-repo.sh | **不是这一批** | 点名 `research/mutations/e158_arms.tsv`、`e158_r4_arm_mutations.tsv`，两者在 `git status` 里都是别的会话的未提交/未跟踪改动，这份规格没有触碰 `research/`；未修 |
| 75-decision-experiment-links.sh | **这一批造成** | 见下节「75 号：E142/E157 正文改动触发『没回看』」；停下，未自行补回看行 |
| 92-layout-checker-sync.sh | **这一批造成** | 见下节「92 号：新登记的 5 个 format-const 触发 checker 同步要求」；停下，未自行改 checker 或登记 lag 表 |

## 43 号：C570/C571 插入点落错表

规格条 9（`.claude/kb/checks-owed.md`）把 C570、C571 两行接在 C419 行之后，而 C419 行位于「### 已还清」一节的 4 列表（表头在第 502 行：`| # | 简称 | 怎么还的 | 还清日期 |`），C570/C571 是新立的未还债务、写成 6 列（与全文首个「欠着」表的表头 `| # | 简称 | 要拦什么 | 怎么拦（会红的形态） | 前置 | 出处 |` 一致），落进了错的表。43 号红：

```
✗ 欠账表有 2 行的格数与所在表的表头对不上、0 个编号登记了两处、0 处还在欠着的表里却写着已还、0 处被空行断开：
    第 585 行 C570：6 格，而它所在那张表（「已还清」下、表头在第 502 行）是 4 格
    第 586 行 C571：6 格，而它所在那张表（「已还清」下、表头在第 502 行）是 4 格
```

这两行该挪到欠着的那张表（表头第 15 行）末尾，接在它最后一行之后，但具体接在哪一行原文之后规格没有给出锚点——挪动属于「补内容」，不是照规格逐字写，停在这里交回，不自行挑锚点搬动。

## 75 号：E142/E157 正文改动触发「没回看」

规格条 23、24 改写了 `.claude/kb/experiments/142-第一个事务的干跑.md`、`157-并行线一两条条款的计数模型.md` 的正文（把已删的共享函数 `index_node_header_bytes` 改成「三方各算一份」的现状，并各补一句门禁 27 号会红在哪一行）。这两处编辑不是任何决策分项的「**依据**」段改动（本批 D28 已定项 4 的依据新增只指向 A4c 报告，起草员报告已说明它不是 E 编号实验页，不适用「依据段改了要同步实验页表」那条），但 `format-evolution.md` 另有一条更宽的规则：实验页正文只要改了（不限于历史节），它的「### 影响的决策」表每一行都要重新回看。75 号红：

```
[没回看] .claude/kb/experiments/142-第一个事务的干跑.md：这次改动里 E142 正文改了，影响的决策表一行都没回看
[没回看] .claude/kb/experiments/157-并行线一两条条款的计数模型.md：这次改动里 E157 正文改了，影响的决策表一行都没回看
```

同一次输出还点了 `.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md`，该文件不在这份规格的改动范围内（`git status` 显示它是别的会话未提交的改动），按「不是这一轮的不修」照写，未处理。

E142、E157 的「### 影响的决策」表要不要回看、写「改了」还是「不受影响：理由」，是判一个实验撑不撑一条分项的关系判断，规格没有给出这两行的旧串/新串，定义明写这类判断不归我；停在这里，交回给主 agent 补规格或裁定关系。

## 92 号：新登记的 5 个 format-const 触发 checker 同步要求

规格条 13、17、19、20、21（`DATA_UNIT_PAYLOAD_OFFSET`、`NONCE_MAC_ALGORITHM_RESERVED_BYTES`、`PACKED_UNIT_RECORDS_OFFSET`、`JOURNAL_RING_DEFAULT_BYTES`、`ROOT_RING_CHUNK_BYTES`，均在 `.claude/kb/layout/01-first-txn.md`）第一次给已有的字节数值挂上 `format-const` 标记；这些数值本身没有变，只是第一次被登记。92 号仍把它判成「常量在这次改动里变了」，要求同一次改动里跟着改这套布局的 checker 判定路径，或登记进 `.claude/gate.d/layouts-checker-lag.tsv`：

```
✗ 5 个格式常量在这次改动里变了，而它所属布局的 checker 判定路径一个都没被碰（基准 faf255e235300d129ede6d6f85af31d686a88519）：
    第一条纯 SSD 布局线（...）（checker 判定路径：crates/singlefs-checker/src/lib.rs, .../walk.rs, .../image.rs）
      新增 DATA_UNIT_PAYLOAD_OFFSET = 134（.claude/kb/layout/01-first-txn.md）
      新增 JOURNAL_RING_DEFAULT_BYTES = 805306368（.claude/kb/layout/01-first-txn.md）
      新增 NONCE_MAC_ALGORITHM_RESERVED_BYTES = 29（.claude/kb/layout/01-first-txn.md）
      新增 PACKED_UNIT_RECORDS_OFFSET = 136（.claude/kb/layout/01-first-txn.md）
      新增 ROOT_RING_CHUNK_BYTES = 1048576（.claude/kb/layout/01-first-txn.md）
```

两条出路都不在我的写范围：改 checker 判定路径要动 `crates/`；登记进 `layouts-checker-lag.tsv` 要写 `.claude/gate.d/`，两者都不在 kb-scribe 的「写范围」里，也不是规格给的内容。停在这里，交回给主 agent。

## doc-lint（全仓，`bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .`）

末行：

```
✗ 文档铁律检查失败：0 个文件违规、0 处编号引用无定义、1 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 518，跳过 0）
```

红的一处：

```
✗ .claude/kb/checks-owed.md:586  编号 C571 的简称显示宽度 49，超过 48（汉字算 2、ASCII 算 1）
   → 怎么办： 起一个更短的简称——引用处每次都要带着它，带不动的名字等于没名字。
              完整陈述留在登记位的其它列里，别塞进简称列。
```

C571 的简称是规格条 9 里逐字给的「df 算不算聚簇段内关着的空槽没量，假性 ENOSPC 待判」，这一批新写的句子。按定义「红在这一轮写的句子上，停下交主 agent 改规格，不自己改句子」，停在这里，未改简称。

## 全部改过的文件、开工与收尾两次 sha256

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/checks-owed.md` | `805287d6343c01d8aebfd412ab479db45ff4b55a175ea1e369e102fdf7e933b8` | `0245aa11080ea43cfe2dfc507d4f5c021910058766ad86e72b90d1907a350ee6` |
| `.claude/kb/decisions-history/2026-09.md` | `d55580f5b1acb82bd6d2c8c58fd53a76922627cd22cbe2f26d31f571eee62a43` | `dbf65f597bbfe21cad1a0dd5c1b5e4fd6eaafa8641c193151a20181076376e14` |
| `.claude/kb/decisions-history.md` | `dc81b258c6e995e2cd52a95a2913192994dcd069a6f1e80be4db653192b1d3ad` | `ddceceebf413372727986c4d2e06c99c018d66881279caeb636a4c5fade4ae2e` |
| `.claude/kb/decisions/13-验证路线.md` | `b779b105b19c079f09fb636dfb491ed4c392fa8040e1377a0e1c831e041ce5e6` | `927a40579caabb519a080a7eb2b0c9a4831696d1477c050a23136522b5d1ff93` |
| `.claude/kb/decisions/18-块里携带什么信息.md` | `4b2e287b65c53ef0d1d40061ab17ee4139e05c545cd4263410ff6cbfb0c8ef05` | `cf40fa6514fca9714039001318a81a16659f8bdff02d87f5d2993b1af2416bc2` |
| `.claude/kb/decisions/23-journal的角色与格式.md` | `0217aad0c83bcf93a06946ec7db3d3e28c4f153e69173e5bd39b62f72b0929b0` | `8c51cf2f3fb158f7b928611709ab5596c7056b5a1ad3d743aea172e9fabad999` |
| `.claude/kb/decisions/28-挂载期承诺量.md` | `33e92e204f305de5724469035eb5185c6e55b8f2ceb473ddd8f4c0ac4f7e1a9b` | `0b843b2b39e9bec49271213e63f7624544ba6a0773bd6fc68e074dceeae9e80c` |
| `.claude/kb/experiments/142-第一个事务的干跑.md` | `0fb863978e8e6fa58d8f6e5bb2ca7412d9909c5dfab0a52bb159096c449edd1c` | `80a1bfa8439e72e2332b22c1f23519aa5aaf39e1dd4b5a95cf4ecad6d640ff24` |
| `.claude/kb/experiments/157-并行线一两条条款的计数模型.md` | `b45f29993ebf05f3b60d5c6a876e0ebbb8359ddf1822f8af336b7c4a30844659` | `d4385a1dbaa3abbae2ba1a4253a622ab92da40bf7a01509bcd61737406ea7795` |
| `.claude/kb/invariants.md` | `fe720b089ff80d3177feb168e7b5cbf09505912c3f8e99ad04f1e05cc2ae1039` | `122c73f819935cb45685119d834795f57f6792ca2682c8ee883c0395553cd698` |
| `.claude/kb/layout/01-first-txn.md` | `04c1ddc74824cbae2bf55fd9db18412bdcb310ac76bce9d068c3e564273903f2` | `c6cce7fbe0ada1ecdb01888fef6cacea93e1f184a4b34bd5666d11b29d4c3c31` |
| `.claude/kb/decisions.md` | `0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895` | `0d9469f1d3d908ff1b938ebdc306024f082c62e317bd6584f5a9328e5c90f895`（未改，规格没点它） |

## 没做什么

- 没判 43、75、92 号红的对不对该怎么修，也没自行挪 C570/C571、补回看行、动 checker 或登记 lag 表——三处都需要规格没给的补内容或判断，停在这里交回。
- 没改 C571 的简称——doc-lint 红在这一批新写的句子上，按定义交主 agent 改规格。
- 没修 27、33、52、69 号：27 是派发提示预告的已知后果、不归我；33、52、69 点名的文件都不在这份规格里，是别的会话未提交的改动，照写不修。
- 没跑分项翻状态相关步骤（`relabel-item.py`、`21-decision-items-sync.sh --write`）——这份规格没有分项翻状态。`21-decision-items-sync.sh` 在第 4 步整轮跑过，绿（350 个分项，未受影响）。
- 没提交、没跑重型测试、没编译 Rust。
