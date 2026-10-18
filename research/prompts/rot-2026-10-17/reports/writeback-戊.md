# 写回员戊报告（批6，kb 类）

没建副本、没起编译、没跑重型测试、没跑 doc-decisions.sh --write（并行写回，归主 agent）。

## 规格与命令输出
- 规格：`$W/spec.json`（46 条，sha256 a26695686bbfd0e4b4a3c8592142b01027836183063f5cc6bc364cb7e5ff28d5）、同内容 `spec.md`（sha256 fbbab1ddb790d3955ce77e4a9a0ff311d94305d88e9c7787905b73e1eb79e22c），W=/home/fy5090/.local/share/singlefs/work/rot-2026-10-17-wb-戊
- kb-spec-check.py 末行（退 0）：`✓ 规格 46 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 46 条）`
- 写范围闸：12 个目标文件逐个喂过，全退 0（检出记录 `$W/precheck.jsonl`）
- replace-batch.py --dry-run：`✓ 46 处都命中得对（12 个文件），--dry-run 没写`
- replace-batch.py 实写（退 0）：`✓ 46 处替换，写了 12 个文件（改名换上新 inode），回读一致`
- 前后 sha256：`$W/sha-before.txt`、`$W/sha-after.txt`；decisions.md 没动（哈希不变）

## 去向（三个条目逐处）
条目一（fixes2.tsv 26 行）：agent-common.md:62 不归我（主 agent）。其余 25 行都改了：
- checks-owed.md：C436（`mask_code_spans()`）、C457（`worker_threads_from_environment()` 现取法，超订阅前提标重核）、C610（行号改 `REMEDIES` 表里 `file-names` 那一项）、C638、C668 两处（P30020、P30021、P30022 依据的是暂存区：`git show :crates/singlefs-checker-tier/src/crash_identity.rs` 里已没有 `lexical_digest_matches_the_e164_anchors`、`RANDOM_HISTORY_THROUGH_LIBRARY_MODULES_ALLOWED`、`random_history_scaffolding_introducers_judged`，HEAD 里还有；改后句已写明依据暂存区）、C12。
- decisions/02 已定项 19 射程（`SystemImmutableSizes::slot_spacing_for`）、decisions/03 已定项 10 的欠、experiments/49 的 Q49.5 单测名、milestone/01 五处（第 19、43、76、159、217 行）、milestone/02 的 62、68（含内文 crash 模块位置）、94、198、359、485 行。
- decisions.md:392（P50005）不手改，源头是 decisions/22 已定项 14 的索引行，见条目三。
条目二：verification-build.md 的 5、39、76、154、301 行，tooling.md:198，experiments.md 的 19、24、26、28、33 行都改了。E1 改成「恢复时间那一半未跑（checker 已有）」；E3 事务层与 checker 已有；E5 仍欠加密实现；E10 文件系统已有、仍欠后台整理（现查：`crates/` 里没有加密代码，D26 索引行进度已实现 0 项）。489 字节、槽宽 4096 取自 `crates/singlefs-format/src/lib.rs` 的两个常量。
条目二里 `crates/singlefs-checker/invariants/README.md`：写闸拒（kb-scribe 范围不含），没写。规格留在 `$W/spec-invariants-readme.json`（3 处：第 36 行、第 40-46 行、第 48-52 行），sha256 82412915fc4f37a24963044333b7f3de92717eb40d599a356a412fcfc1275083，`kb-spec-check.py` 退 0（3 条全过），`replace-batch.py --dry-run` 命中 3 处；主 agent 用 `python3 scripts/kb/replace-batch.py <规格>` 套。第 40-46 行我没有逐类判哪几类现在写得出，改后句写明「没有逐类核过」，只去掉「定案后」「细化后」并补决策现状与 I-9.12、I-9.16。
条目三：
- checks-owed.md C590 行号改成 `grid_decision_items_sync()` 与 `open(idx…)` 两个代码锚串（两处）。
- milestone/02 第 592 行 `publish_file_version` 与 94 行同改。
- decisions/22 已定项 14 索引行「进度：未实现」改「进度：已实现」（现查：`IMPLEMENTED_INVARIANTS` 含 `I-7.6`，invariants/02-根环.md 状态列已实现）；变更史 D22 节 2026-10-17 块追加一项。decisions.md 索引没手改，D22 的进度计数与第 392 行投影要主 agent 跑 `bash .claude/gate.d/doc-decisions.sh --write`。
- milestone/02 第 550 行：现查不改。`grep -rl 'mount_read_only\|open_pool_for_read' crates/singlefs-checker-tier` 只中 `src/crash_injection.rs`（一个 unlink 之后的只读挂载专用用例）和 `src/bin/e158_root_choice_repair.rs`；崩溃点重放对每个恢复结果跑挂载态读的通用覆盖没找到，「还剩一样没做」未见被推翻。
- milestone/02 第 574 行：判不了，留。`crates/singlefs-checker-tier/tests/crash_enumeration_record_spill_over_stream.rs` 是一次发布建出 54 片左右叶容器、点名 65 项的崩溃枚举，`tree-split-streams` 流罩的是记账树与中央映射树的分裂，不是 inode 容器的「分裂一次」；十万个文件那一档全仓没找到。「小负载那一样已被覆盖」是不是该这么读，要主 agent 判。

## 变更史动了哪几节
- decisions-history.md：D2 节新开 2026-10-17 块（已定项 19 射程）；D3 节新开 2026-10-17 块（已定项 10 的欠）；D22 节 2026-10-17 块追加一项（已定项 14 进度）。写前写后各 `git diff` 一次，新增行只有这三处（`$W/hist-diff-before.txt`、`hist-diff-after.txt`）。
- experiments-history.md：E49 节 2026-10-17 块追加一项（Q49.5 单测名）。
- 实验页 E49 的「影响的决策」两行（D23 已定项 8 备料、D13 不影响）回看日期改成 2026-10-17「不受影响：理由」，关系没动。原因：页面正文改了，门禁 doc-experiments 的 decision-links 格要求这张表改过至少一行，且最近一次变动晚于回看日期判红。这是我加的，不在规格原文里，主 agent 觉得不该就还原这两行。

## 要主 agent 知道的
- experiments.md 的 E1、E3、E5、E10 四行改了，但这四个实验页的标题状态括注（E1「恢复时间那一半仍欠 checker」、E3「依赖事务层与 checker」、E5「依赖事务层、加密实现与 checker」、E10「依赖文件系统」）和 E5、E10 的「前置」行没动（不在派发范围，且要走 experiments-history 与回看）。索引与页面标题现在不一致，要不要同改你定。
- milestone/01 第 43 行「六个模块」后列了七个名字，不在 fixes2 里，没动。
- milestone/01 第 159 行「取号序不一致」那个计数是否还在，没核到对应名字（judge-S39 已提），没动。
- 没跑 `--write`、没跑 doc-lint 与任何门禁阶段。

## 没做什么
没判定案对不对、没写规格外的句子（上面标出的 E49 回看两行除外）、没修不是这一轮的红、没提交；条目三两处「要人看」（550、574）按现查结果留着。

## 改过的文件与前后哈希（前 → 后，前 12 位）
78bdf2863d73 .claude/kb/checks-owed.md	16a966f0a590
7ef9368c31f8 .claude/kb/decisions/02-RAID条带策略.md	6d08cf137e0c
4029fa744846 .claude/kb/decisions/03-空间分配.md	001163f1c650
f19b17696f76 .claude/kb/decisions/22-单元原子性怎么合成.md	693eefacd37e
936747078ce1 .claude/kb/decisions-history.md	2b5b52f992aa
c89fc6132807 .claude/kb/decisions.md	c89fc6132807
9537dae8b371 .claude/kb/experiments/49-反向链宽度32还是64.md	96d36add0f7d
76006f988b24 .claude/kb/experiments-history.md	5cf6d0e95d88
53152739734b .claude/kb/experiments.md	f6561bfeabc5
98b124c38b86 .claude/kb/milestone/01-first-txn.md	7d014694a4b3
c7587be79677 .claude/kb/milestone/02-second-txn.md	4a4851850887
ca2432f4481b .claude/kb/tooling.md	cd4feb218a86
09b3c4d68138 .claude/kb/verification-build.md	4b9019b83976
