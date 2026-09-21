# archive-rename-r1 核查表

**核对表里的 ✗ 不免除主 agent 对推论的逐条现查。**

轮：archive-rename-r1（代码轮）。核的是引用/产物/复跑命令，不判推理本身、不判该不该采纳。

## 输入与快照说明

主 agent 起初漏给快照，之后补了倒推件：`/tmp/claude-1000/archive-rename-r1-snapshot/`
（`files/` 11 份、`snapshot.sha256`、`倒推说明.md`）。sha256 逐份核对：`diff` 两份 sha256 清单
（排序后按文件名对齐）退出码 0，11 份文件内容与登记的哈希一致。

独立复核倒推逻辑（不只信主 agent 的说法）：对 11 份文件各自取 `git show HEAD:`、
`git show :`（索引版）、当前工作区三个版本分别与快照内容 `diff -q`，结果：**11 份文件的快照内容
无一例外全部等于索引版（`git show :<路径>`）**，与主 agent 给的倒推规则（`[ M]`→HEAD、`[MM]`→索引、
`[M ]`/`[  ]`→工作区）在「索引==HEAD 时两者相等」「未改时工作区==索引」这两处退化情形下是一致的——
倒推件本身内部自洽，没发现构造错误。

## 关键发现：主 agent 说「其余九份自派腿以来一处没动」，现查不是这样

写这份报告的过程中，重新逐份 `diff` 快照与**此刻**主树，结果六份现在不一致：
`.claude/gate.d/40-results-cited.sh`、`.claude/gate.d/88-quoted-result-lines.sh`、
`research/perf-by-milestone.md`（这三份主 agent 自己说了）——
但另外三份，主 agent 说「没动，可以对主树核」，**现查却也不一致**：

| 文件 | 主 agent 怎么说 | 现查到的 |
|---|---|---|
| `.claude/gate.d/90-term-renames.sh` | 「自派腿以来一处没动」 | 第 17-19 行的 howto 已经从「源码与留存产物同改」改写成区分「跑得出来的」跟着源码换、「跑不出来的」一个字节不许动——**恰好是攻方 J8② 打中的那个矛盾点的修法方向** |
| `.claude/kb/experiments/152-按里程碑对比六家文件系统的文件性能.md` | 「没动，可以对主树核」 | `:243` 已经从 `system_configuration_slot×4,barrier` 改回 `superblock_slot×4,barrier`，与归档产物一致——**正是攻方 J8③ 打中的那一处** |
| `.claude/term-rename-exempt` | 「没动，可以对主树核」 | 新增 7 行豁免登记，其中一条逐字「`.claude/kb/experiments/152-...md  # 整行抄自留存产物的引文块：产物已归档进版本库，字节改不了，正文换名那句「整行抄自」就成假的（门禁 88 号会红）`」——同样是 J8③ 那一处的登记 |

⇒ **这份报告后面对全部 11 份文件的每一处核对，一律用快照 `/tmp/claude-1000/archive-rename-r1-snapshot/files/`，
不用主树**——即便主 agent 只点名了 2 份要求这样做。判别力自证与下面全部表格都遵守这一条。

这处漂移也解释了一个数字差：攻方 J8 结尾 `grep -rni "superblock" ... \| wc -l` 报的是 30，
我现在对主树跑同一条命令得到 34。差值 4 正好等于上表三处漂移新增的「superblock」大小写不敏感命中
（`term-rename-exempt` 新增 7 行里 3 行含 superblock + `152-...md` 那一行改回）——30 是快照状态下的真数，
34 是漂移后的数，**不构成对攻方腿的否定**。

## 判别力自证

挑的引用：攻方腿 J1 对 `crates/singlefs-harness/src/first_transaction_regions.rs:78` 的引文
「区域清单的一行。`name` 与 E142 装置里同一个结构的标签同名，量 5 按 `region=` 加 `device=` 两边配对。」

在草稿副本里把行号改成 79，按第 2 步核：取快照该文件第 79 行——

```
$ awk 'NR==79' /tmp/claude-1000/archive-rename-r1-snapshot/files/crates/singlefs-harness/src/first_transaction_regions.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
```

与引文不符（一个是文档注释、一个是 derive 属性）。**判 ✗**。核查方法能分辨，自证通过；
下面的核对同一套方法（到给定的行号取一行/一段，与引用处的文字比对）。

## 一、攻方腿（Opus）：`archive-rename-r1-opus-output.md`

全部行号对快照核（`crates/singlefs-harness/src/first_transaction_regions.rs`、
`research/e7-index-bench/src/bin/e142_first_transaction_dry_run.rs`、
`research/perf-by-milestone.md`、`.claude/gate.d/88-quoted-result-lines.sh`、
`.claude/gate.d/90-term-renames.sh`、`.claude/kb/experiments/152-...md` 六份用快照；
`.claude/singlefs-ai-sop/rules/evidence-discipline.md`、`research/scripts/replay.sh` 本轮未动，
对主树核结果与快照一致）。

### J1

| 引用 | 结果 | 命令/依据 |
|---|---|---|
| `first_transaction_regions.rs:78`「区域清单的一行……量 5 按 `region=` 加 `device=` 两边配对。」 | ✓ | 快照第 78 行逐字一致 |
| `e142_first_transaction_dry_run.rs:3932`（`find` 闭包比较那一行） | ✓ | 快照第 3932 行逐字一致 |
| 同文件 `:3905`（`region_geometry.push(("system_configuration", …))`） | ✓ | 快照第 3905 行逐字一致（转述，不是引号引文，核实为真） |
| 同文件 `:3935`（`reason=no_impl_snapshot_given` 那一行） | ✓ | 快照第 3935 行逐字一致 |
| `target/release/first_transaction_region_bytes` 编于 01:32:03 UTC；`first_transaction_regions.rs` 改于 02:21:05 UTC | ✓ | `stat -c '%y'` 现查主树两份文件的 mtime，与报告里的两个时刻逐秒一致（`01:32:03.735065856`／`02:21:05.513590294`） |
| **复跑核实**：报的 `regions=21 equal=19 unequal=0` 与两行 `equal=unknown…reason=no_impl_snapshot_given`，用的是过期二进制 | ✓（腿自己已声明限度，复跑证实「今天走真实路径不会复现同一降级」，与腿的说法一致，不是反驳） | 见下方「复跑」小节 |

**复跑**：`crates/` 与 `research/e7-index-bench/` 之外整份 rsync 到
`/tmp/claude-1000/archive-rename-r1-verifier/repo-copy/`（不含 `.git`、`target`），按
`research/scripts/replay.sh` 里 `driver_e142()` 的做法现编现跑：

```
cd repo-copy && nice -n 19 cargo run -q -p singlefs-harness --bin first_transaction_region_bytes \
  > impl-snapshot-fresh.txt   # exit=0，25 行
cd repo-copy && nice -n 19 cargo build --release --manifest-path research/e7-index-bench/Cargo.toml
cd repo-copy/research && nice -n 19 ./target/release/e142-first-txn-dry-run \
  ../impl-snapshot-fresh.txt > run-fresh.out   # exit=0
grep impl_bytes_equal_summary run-fresh.out
# E7RESULT name=impl_bytes_equal_summary regions=21 equal=21 unequal=0 snapshot_given=true
```

今天现编现跑得到的是 `equal=21`，不是攻方报的 `equal=19`——**这正是攻方腿自己在报告第 322 行「这条腿自己的
限度」第 2 点承认的**：「走 replay 这条路时过期二进制这个具体触发不会发生」。核实结论：攻方报告里的字面数字
（21/19 与两行 `unknown`）作为「用那份过期二进制跑出来的结果」为真（腿的原始输出文件仍在
`/tmp/claude-1000/archive-rename-r1-opus/run-current.out`，与报告里贴的四行逐字一致，现查过）；
作为「driver_e142 今天的真实路径会给出的结果」为假，腿没有这样声称，判 ✓（无夸大）。

**产物比对补充**：腿自己贴的 `run-current.out` 末四行（第 76–79 行原文）与该文件
`/tmp/claude-1000/archive-rename-r1-opus/run-current.out` 现查逐字一致（`grep -n` 定位第 228、230、
232、233 行），腿没有事后改过自己的原始产物。

### J2

| 引用 | 结果 | 命令/依据 |
|---|---|---|
| `research/perf-by-milestone.md:350`「第一个事务写出什么，E142（第一个事务的干跑） 的产物 `research/results/e142-first-txn-dry-run-2026-09-14-round2-slot4096.out` 里逐字是：」 | ✓ | **用快照**（索引版）第 350 行逐字一致 |
| 同文件 `:354`「…\|[system_configuration_slot×2]」 | ✓ | 快照第 354 行逐字一致（此行快照与当前主树相同，未受漂移影响） |
| 归档产物第 34 行写 `superblock_slot×2` | ✓ | `git log --all --diff-filter=D --name-only -- "*e142-first-txn-dry-run-2026-09-14-round2-slot4096.out"` → 命中提交 `3cff909`；`git show 3cff909^:研究/results/...out \| sed -n '34p'` → `E7RESULT name=segments path=transaction operations=23 segments=16+2+1+2 closed_form=65543 kinds=[unit_write×16,barrier]|[journal_record×2,barrier]|[root_record_fua]|[superblock_slot×2]`，与腿报告第 154 行抄的逐字一致 |
| `ls research/results/e142-first-txn-dry-run-2026-09-14-round2-slot4096.out` → 不存在 | ✓ | 现查 `No such file or directory` |

### J8

| 引用 | 结果 | 命令/依据 |
|---|---|---|
| ①`88-quoted-result-lines.sh:37`「`kb = sorted(f for f in glob.glob('.claude/kb/**/*.md', recursive=True)`」 | ✓ | **用快照**（HEAD 版）第 37 行逐字一致 |
| `grep -rn 'perf-by-milestone' .claude/gate.d/` 命中 0 行 | ✓ | 现查命中 0 行（这条与漂移无关，`.claude/gate.d/` 下没有任何脚本点名这份文件） |
| ②`90-term-renames.sh:17`「怎么办：python3 research/scripts/sweep-term.py --apply 一次换完，源码与留存产物同改；」 | ✓ | **用快照**（索引版）第 17 行逐字一致。⚠️ **主树现在这一行已经不是这句了**（见前面「关键发现」），但腿引的是它开工时看到的那句，判 ✓ |
| `evidence-discipline.md:49/52/55` 三处引文 | ✓ | 本轮未动，对主树核，三处逐字一致（`## 原样保存的证据不许事后改`／`**改一个字，产物就不再对应它的输入**，而外面看不出来：文件还在，日期还在，`／`所以改提示只能连同重跑一起改。要补说明就另开一份文件，别动原件。`） |
| `research/results/e115-...out` 改了 4 行（idx=0/2/3/7，超级块→系统配置） | ✓（但基准点需要更正，见下） | 见下方「e115 产物比对」 |
| `replay.sh:81`「`E115|e115-system-configuration-completeness||e115-system-configuration-completeness-2026-09-07.out|exact`」 | ✓ | 快照第 81 行逐字一致 |
| 手改产物与改名后现跑的新鲜输出逐字节相同（`diff` 无输出，`replay.sh E115` 判 exact） | ✓ | 独立复跑核实，见下 |
| ③`152-...md:240`「…第四次跑第 2 轮这几行的时间戳跨了 21.7 ms（整行抄自 `e152-...第2017.out` 第 21、26–29、32 行）：」 | ✓ | **用快照**（索引版）第 240 行逐字一致 |
| 同文件 `:243`「…\|[system_configuration_slot×4,barrier]」 | ✓ | 快照第 243 行逐字一致。⚠️ **主树现在这一行已经改回 `superblock_slot×4,barrier`**（见「关键发现」），腿引的是快照那一版，判 ✓ |
| 归档产物第 21 行写 `superblock_slot×4` | ✓ | `git log --all --diff-filter=D` 命中同一提交 `3cff909`；`git show 3cff909^:.../e152-...2017.out \| sed -n '21p'` 逐字一致 |

**e115 产物比对（基准需要更正）**：任务要求与 `3cff909^` 比，但 `git diff 3cff909^ --
research/results/e115-system-configuration-completeness-2026-09-07.out` 报的是「new file」——
`git show 3cff909^:该路径` 直接 `fatal: path … exists on disk, but not in '3cff909^'`：这份产物在
`3cff909^` 根本不存在，是 `3cff909`（归档提交本身）把它取回来的，`git log --oneline --follow` 确认
它只在 `99ad3f0`（创建）与 `3cff909`（取回）两次提交里出现过。**正确的基准是 `3cff909`，不是
`3cff909^`**——这与 J2、J8③ 引的另外两份归档产物不同：那两份在 `3cff909^` 就已存在（是被 `3cff909`
删掉的旧记录），`e115` 这份反而是被 `3cff909` 加回来的。改用 `3cff909` 作基准：

```
$ diff <(git show 3cff909:research/.../e115-...out) research/.../e115-...out
2c2
< …source=I-8.1 逐字「写进超级块」
---
> …source=I-8.1 逐字「写进系统配置」
4,5c4,5
< …逐字「超级块声明的常量」   （×2，idx=2 与 idx=3）
---
> …逐字「系统配置声明的常量」
9c9
< …source=I-6.6 逐字「与超级块声明的一致」
---
> …source=I-6.6 逐字「与系统配置声明的一致」
```

正好 4 行（idx=0、2、3、7），与腿的描述逐字一致。判 ✓（腿的「改了 4 行」与「全是超级块→系统配置」两句都成立；
只是任务里给的对比基准 `3cff909^` 本身不适用于这份产物，记在这里备查，不影响对腿的判定）。

**E115 复跑核实**：`repo-copy/` 里覆盖快照版 `replay.sh`（因为另一个会话此刻在用主树那份跑 E155，
按主 agent 指示不用主树），跑：

```
cd repo-copy && REPLAY_OUT=/tmp/claude-1000/archive-rename-r1-verifier/replay-out \
  nice -n 19 bash research/scripts/replay.sh E115
# E115  e115-system-configuration-completeness 字节一致 e115-...2026-09-07.out
# 字节一致 1／仅计时不同 0／对不上 0／跑不了 0／结论断言不中 0／产物已归档 0
$ sha256sum replay-out/E115.out
a22fbf30bb7688354bff885668c0f638090472312d2df4a9f771875da3ac3243
```

与腿报告里给的 `e115-fresh.out` sha256 **逐字节一致**，且与 `diff` 现查真的无输出。判 ✓。

### 「没打中」小节抽查（非重点核，抽 3 处核实腿没有夸大排除范围）

| 引用 | 结果 |
|---|---|
| `segments.rs:15`（derive PartialOrd, Ord）、`:16-21`（枚举体）、`:146`（`BTreeMap<StepKind, usize>`） | ✓ 三处均逐字/逐行核实一致（本轮虽标 `M `，但对主树核与快照逻辑一致，无漂移） |
| `write_accounting.rs:65/78/108`（`report_name`/`SystemConfigurationSlot`/`BTreeMap<WrittenStructureKind,…>`） | ✓ 三处均一致 |
| `grep -n 'system_configuration_slot' research/mutations/e142_first_transaction_dry_run.tsv` 命中第 21、34、65、66 行 | ✓ 现查恰好 4 行、行号一致（用更宽的模式会多算出第 20 行，但腿用的是这个更窄的词，判 ✓） |
| `git diff --cached --stat -- research/mutations/e142_first_transaction_dry_run.tsv` 为空 | ✓ 现查确认为空 |
| `grep -n 'system_configuration_slot\|superblock' crates/mutations.tsv` 命中 0 行 | ✓ 现查命中 0 行 |

### opus 小计

核了 22 处（含 2 条复跑），✓ 21，✗ 0；1 处需要更正对比基准（不算 ✗，记为补充说明）。

## 二、正推腿（Sonnet）：`archive-rename-r1-sonnet-output.md`

`.claude/agents/three-way-materials.md`、`research/scripts/archive-past-rounds.py`、
`.claude/gate.d/lib-link-targets.py` 三份本轮未动（`git status --porcelain` 均为空或与快照一致），
对主树核；`.claude/gate.d/88-quoted-result-lines.sh` 用快照。

### 材料现状核实（sonnet 自报的部分）

| 引用 | 结果 | 依据 |
|---|---|---|
| `git diff --cached --raw -- archive-past-rounds.py` 的 blob 是 `93435d9..93fb60d` | ✓ | 现查一致 |
| 索引项 blob `93fb60dc20b2c84d302c01ab68372eb1200238e6` | ✓ | `git rev-parse :research/scripts/archive-past-rounds.py` 逐字一致 |
| 「157 行插入、2 行删除」 | ✗ | `git diff --cached --numstat` 现查是 **155 行插入、2 行删除**，不是 157（此文件本轮 `M `、无进一步漂移，不是漂移造成的差） |

### J3①

| 引用 | 结果 | 依据 |
|---|---|---|
| `research/scripts/archive-past-rounds.py:29`「`SCAN = (".claude/kb", "records", ".claude/rules", "briefs")`」 | ✓ | 现查第 29 行逐字一致 |
| `still_an_input`（47-82 行）、`rewrite_links`（93-121 行） | ✓ | 现查两函数的 `def` 行与 `return` 行恰好落在 47/82 与 93/121 |
| `:32`「`INPUT_TREES = (".claude/gate.d", ".claude/scripts", "research", "crates")`」 | ✓ | 现查第 32 行逐字一致 |
| `.claude/agents/three-way-materials.md:20` 与 `:28` 都写着 `research/prompts/_m2-code-r1-diff.md`（「形态照」） | ✓ | 现查两行都含「形态照 `research/prompts/_m2-code-r1-diff.md`」，本轮未动，对主树核 |
| `ls research/prompts/_m2-code-r1-diff.md` → 不存在 | ✓ | 现查 `No such file or directory` |
| `git log --oneline --all -- 该路径` 命中 `9de47f9`、`3cff909` | ✓ | 现查命中这两个提交，`3cff909` 正是这一轮判的归档提交 |
| 门禁 23 号 `lib-link-targets.py` 只判 `[]()` 语法，正则在「`:80`」 | ✗ | 现查实际正则 `r'\[([^\]]*)\]\(([^)\s]+)\)'` 在**第 82 行**，不是 80；第 80 行是 `continue`（本轮未动，非漂移，纯粹行号数错） |
| `mask_code()`（54-61 行）先涂空反引号片段 | ✗（范围偏） | 现查 `def mask_code(s):` 实际在 52 行，函数体到 62 行 `return s` 结束；54-61 只是文档字符串尾部＋函数体前两行，masking 的两行 `re.sub` 恰好落在 60-61（在引用范围内），核心机制的描述方向没错，但函数边界引用不准 |

### J3②

| 引用 | 结果 | 依据 |
|---|---|---|
| `missing_code_inputs()` 注释（132-135 行）：「49 处命中里只有 0 处是真的」等 | ✓ | 现查第 132-135 行逐字一致（本轮未动） |
| `e131_livelist_carrier.rs:136`「`println!("判据写死在 research/prompts/e131-preregistration.md（写于本装置之前）");`」 | ✓ | 现查第 136 行逐字一致 |
| `research/prompts/e131-preregistration.md` 不存在 | ✓ | 现查确认不存在 |
| `.claude/gate.d/85-repro-command.sh:29`「`echo "    原始输出 \`research/results/eNN-xxx-YYYY-MM-DD.out\`。"`」 | ✓ | 现查第 29 行逐字一致 |
| `fixtures` 跳过逻辑在「54、61、140 行」 | ✗（54 不对） | 现查只有两处 `if "fixtures" in directory.split(os.sep):`：第 61 行（`still_an_input` 内）与第 140 行（`missing_code_inputs` 内）；第 54 行是 `still_an_input` 文档字符串收尾句，不含这个判断，三个数里有一个是错的 |
| `grep -rl "research/results\|research/prompts" research/mutations/ crates/mutations.tsv` 零命中 | ✓ | 现查零命中 |

### J4

| 引用 | 结果 | 依据 |
|---|---|---|
| `past_round_files()`（41-44 行）三行赋值 | ✓ | 现查第 41-44 行逐字对应（sonnet 给的是简写伪代码，不是逐字引用，语义与真代码一致） |
| `git mv`／部分暂存两种边界：`dirty` 会含新路径／带 MM 状态的文件仍在 dirty 里 | 核不动（逐字） | j4test 现场已推进到 gitlink 场景，前两种场景的临时提交没有单独留存复现，不重建；判据本身（`git diff --name-only HEAD` 不分暂存与未暂存）读代码可信，判 ✓（读代码，不算复跑） |
| 软链接：`os.remove()` 对符号链接生效、不删目标 | ✓（常识性行为，未单独复跑） | 未重跑，Python `os.remove` 对符号链接的行为是标准库文档行为，不单独验 |
| gitlink 场景：`git ls-files -s` 报 `160000`；`os.remove()` 抛 `IsADirectoryError` | ✓ | 复制 `j6test`→`j4test-copy` 独立重跑：`git ls-files -s research/results` 现查确实是 `160000 …research/results/nested-repo`；`git diff --name-only HEAD -- research/results` 现查为空；`python3 -c "import os; os.remove('research/results/nested-repo')"` 现查抛出 `IsADirectoryError: [Errno 21] Is a directory: 'research/results/nested-repo'`，与报告逐字一致 |
| `run()`（177-222 行）删除循环没有 try/except | ✓ | 现查 `def run(root, apply_changes):` 在 177 行、末尾 `return 0` 在 222 行，循环体现读确认无 try/except 包裹 `os.remove` |
| 仓里现在没有子模块（`.gitmodules` 不存在） | ✓ | 现查 `.gitmodules` 不存在 |

### J6

`.claude/agents/gate-triage.md:31`「共享 `gate.sh` 全绿时 `update-ref refs/singlefs/gate-ok`」 → ✓
现查第 31 行含此逐字片段，本轮未动。

`88-quoted-result-lines.sh`（快照 HEAD 版）四处行号引用：

| 引用 | 结果 |
|---|---|
| 第 4 条判据在「19-22 行」 | ✓ 逐字一致 |
| `body`（42-43 行）＝ `\n## 历史版本` 之前的部分 | ✓ 逐字一致 |
| `quoted`（44-47 行）只收 `E7RESULT ` 开头的行 | ✓ 逐字一致 |
| `BASE`（28-32 行）：`GATE_BASE` 优先，否则 `refs/singlefs/gate-ok`／`@{upstream}` | ✓ 逐字一致 |
| 有 `BASE` 时的过滤逻辑（50-69 行） | ✓ 逐字一致（50 行是 `base = os.environ.get(...)`，69 行是 `scope = '这次改动新增或改写的'`，恰好框住这段 if-block） |

**复跑「漏判一」**：`sonnet` 留的 `/tmp/claude-1000/archive-rename-r1-sonnet/j6test`
现在处于 commitC（历史节新写错行）状态，`refs/singlefs/gate-ok` 指向 commitB；直接对它跑（未修改该目录）：

```
$ nice -n 19 bash <快照>/.claude/gate.d/88-quoted-result-lines.sh "$(pwd)"
  ! 这次改动新增或改写的 kb 正文里没有整行抄的 E7RESULT 行，本阶段无对象可判
exit=77
```

与报告逐字一致，判 ✓。

**复跑「漏判二」**：把 `j6test` 拷到本核查员自己的草稿目录（不改动 sonnet 的原目录），
`git checkout` 到 commitB（`7d1cdef`）、`git update-ref refs/singlefs/gate-ok ce337ef`
（模拟 gate-ok 停在 commitA），跑：

```
$ nice -n 19 bash <快照>/.claude/gate.d/88-quoted-result-lines.sh "$(pwd)"
  ✗ 这次改动新增或改写的 kb 正文里有 1 行整行抄的产物行，在 research/results/ 的 1 份产物里一份都找不到：
     .claude/kb/page.md:5  E7RESULT another_wrong=1000
exit=1
```

只报出 commitB 的 `another_wrong=1000`，不报 commitA 的 `wrong_value=999`，与报告逐字一致，判 ✓。

### sonnet 小计

核了 27 处，✓ 24，✗ 3（`lib-link-targets.py:80` 应为 82；`mask_code()` 范围 54-61 应为 52-62；
`fixtures` 跳过行号「54、61、140」里 54 不对；「157 行插入」应为 155）。

**订正上面的小计**：写的时候数错了，重新用命令数了一遍本节的表格行（`grep '^|'` 排除表头与分隔线）：
`sonnet` 一节实际核了 **31 处**，✓ **26**，✗ **4**（`lib-link-targets.py:80` 应为 82；`mask_code()`
范围应为 52-62 不是 54-61；`fixtures` 跳过行号里的「54」不对；「157 行插入」应为 155），
**核不动 1**（`git mv`／部分暂存两种边界场景，j4test 现场没有单独留存复现，只核了判据逻辑本身）。

**订正 opus 小计**：同样用 `grep '^|'` 数了一遍——**25 处，✓ 25，✗ 0**；其中 1 处（e115 产物对比基准）
不算 ✗，是任务给的对比基准 `3cff909^` 本身不适用于这份产物（该产物在 `3cff909^` 不存在），
改用 `3cff909` 后腿的具体数字与措辞逐字核实成立。

## 三、本地攻方腿（J5/J7）与逐句核对表

报告文件：`archive-rename-r1-local-attack-output-s1.md`（Group A，5 个场景）、`-s2.md`
（同一组 Group A 的第二次抽样，两次答案一致：a/b/c/d/e 五格判定与理由逐句相同，只是表述略有出入）、
逐句核对表 `archive-rename-r1-local-attack-translation-audit.md`。

### 逐句核对表：`原文文件:行` 逐条核

| 引用 | 结果 | 依据 |
|---|---|---|
| `research/scripts/replay.sh:479`「逐字节这一档没有对照物」 | ✓ | 本轮未动，现查第 479 行含此逐字片段 |
| `.claude/term-rename-exempt:8`「扫了它就再也换不动东西」 | ✓ | **用快照**第 8 行逐字一致 |
| `.claude/kb/term-renames.md:9`「（2026-09-20 起，英文标识符与冻结目录 2026-09-21）」 | ✓ | 现查第 9 行逐字一致（本轮未动，`M ` 状态） |
| `.claude/kb/term-renames.md:11`「2026-09-21 定案：英文标识符、文件名与冻结目录一并改，「直到全工程都搜不出来」。」 | ✗ | 现查该句实际在**第 12 行**，第 11 行是空行；用 `git show HEAD:` 复核（HEAD=3cff909，早于这一轮全部改动）同样是第 12 行，**不是快照没给导致的漂移，是行号本身数错了 1 行** |
| `research/scripts/replay.sh:447`（指向 `show-me-test.md`「门禁不许假装通过」的括注） | ✓ | 第 447 行逐字一致 |
| `.claude/term-rename-exempt:7`（指向 `CLAUDE.md「怎么改」那一行`） | ✓ | **用快照**第 7 行逐字一致 |
| `.claude/term-rename-exempt:5`「btrfs、F2FS、ZFS 等自己的术语与原文引文」 | ✓ | **用快照**第 5 行逐字一致 |
| `research/scripts/replay.sh:446`「读的人会以为实验坏了」 | ✓ | 第 446 行逐字一致 |
| `.claude/term-rename-exempt:6`「它记的就是「这个词被改掉了」这件事本身」 | ✓ | **用快照**第 6 行逐字一致 |
| `.claude/term-rename-exempt:9`「才说得清豁免的是什么」 | ✓ | **用快照**第 9 行逐字一致 |
| `.claude/term-rename-exempt:10`「历史产物与旧提交的字段才对得上」 | ✓ | **用快照**第 10 行逐字一致 |

⚠️ 上表 11 处引用全部落在快照（或本轮未动的文件）里未被改动的行区间（1-10 行）；
`.claude/term-rename-exempt` 目前新增的 7 行（11-17 行，见「关键发现」）不在逐句核对表引用范围内，
不影响这张表的判定。

### 逐句核对表「表一」（已改稿的三处）：定稿是否真落进了最终提示

| 定稿断言 | 结果 | 依据 |
|---|---|---|
| Fact 3 补回「noting again that the byte for byte comparison tier has no counterpart to compare against for these rows」 | ✓ | 现查 `research/prompts/archive-rename-r1-local-attack.md:93-94` 含此逐字片段 |
| Row 4 删掉「this kind of」，改成「to ever rename anything again」 | ✓ | 现查 `:227` 逐字一致；全文 `grep -n "this kind of"` 零命中，确认没有残留旧稿 |
| Shared background 改写成「for English identifiers starting 2026-09-20, and the renaming was completed across the rest of the repository by 2026-09-21.」，不再提「frozen」 | ✓ | 现查 `:44-45` 逐字一致；全文 `grep -n "frozen as of"` 零命中 |

### 本地攻方两次抽样（s1/s2）一致性

Group A（Scenario a-e）与 Group B（Row 1-7）两次抽样的判定与理由逐条对比：五个场景的
true/false、标签（archived/mismatch）、三档结论（known to be fine / genuinely unknown /
known to be regressed）完全一致；Group B 七行 X/Y/none 判定完全一致。按
`.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，这是「打中」类结论
（Group A 里 d、e 两格判 mismatch/regressed，指向真实退步），一次就够判；这里额外抽了两次，
两次一致，不构成「不稳定」。

### 本地攻方小计

核了 14 处（11 处逐句核对表原文引用 + 3 处「表一」定稿落地核实），✓ 13，✗ 1
（`.claude/kb/term-renames.md:11` 应为 12）。

## 总计

按上面三节各自的表格行数出来（不手数）：

- 攻方腿（Opus）：25 处，✓ 25，✗ 0
- 正推腿（Sonnet）：31 处，✓ 26，✗ 4，核不动 1
- 本地攻方（J5/J7 逐句核对表）：14 处，✓ 13，✗ 1

**合计 70 处，✓ 64，✗ 5，核不动 1。**

✗ 的 5 处全部是**行号或计数偏差**，不涉及被引内容本身是否存在：

1. `lib-link-targets.py:80`（应为 82，正则文本本身核对无误）
2. `mask_code()` 范围「54-61 行」（应为 52-62 行，masking 的两行核心逻辑仍落在被引范围内）
3. `fixtures` 跳过行号「54、61、140」（54 不对，61 与 140 对）
4. `git diff --cached --numstat` 「157 行插入」（应为 155）
5. `.claude/kb/term-renames.md:11`（应为 12）

**没有发现任何一处「引用的内容根本不存在」或「产物数字被编造」**——包括对 opus 的复跑核实
（今天现编现跑走真实路径不复现 21/19 的降级，但腿自己已经写明这一点，不是腿的夸大）、
对 e115/152 两份归档产物的独立核对、对 sonnet 的 gitlink 场景与两条 88 号漏判场景的独立复跑，
全部与三份报告里的文字或数字逐字/逐位吻合。

## 关键发现小结（不是判决，供主 agent 判决时用）

1. 主 agent 说「(其余)九份自派腿以来一处没动」不准确：`90-term-renames.sh`、`152-...md`、
   `.claude/term-rename-exempt` 三份实际已在派腿之后被改动，且改动内容恰好分别对应
   J8②、J8③ 两处打中——**这暗示主 agent 已经看过腿的报告内容并据此动手改了主树，
   而不是在核对表落地之前先把这几处的现状钉住**。这几处改动本身对不对，不归本次核查判；
   但核查用的是快照（腿开工时的状态），结论只对「腿的引用当时是不是真的」负责。
2. 攻方腿 J1 的核心论点（区域名字符串在 `find` 里被当配对键、名字不对时静默降级成
   `equal=unknown` 而不触发 F1）本身经独立复跑确认为真；它报的具体数字（19/21）依赖一份
   过期构建产物，腿自己写清楚了这一点，复跑核实与它的自述一致。
3. J8②「门禁 90 号出路与证据纪律打架」这条打中的具体表现（`e115-...out` 改了 4 行且
   `replay.sh` 仍判 exact）经独立复跑确认为真；但主树现在的 90 号出路文字已经改写，
   这条打中在派腿时成立、在写这份核查报告时其成因（旧版出路文字）已经不在主树上了。

## 没做什么

- 不判任何一条推论打中成不成立、该不该采纳，只核引用、产物与复跑，按定义办。
- sonnet J4 的「git mv」「部分暂存」两种边界场景没有单独复跑（j4test 现场只留下 gitlink 场景的最终状态，
  前两种场景的临时提交没有留存复现），只核了判据代码本身的逻辑，记「核不动」。
- opus 报告「没打中」小节只抽了 5 处核实（未逐条核完全部 6 条），已抽到的与报告一致，
  未抽到的第 4 条（`name=width structure=` 解析）未核。
- 本地攻方两份场景表（s1/s2）里 Group B 每一行的分类理由是否与背景材料 Fact 3/Fact 4/Fact 5
  的措辞逐字对应，没有逐句核对——只核了逐句核对表本身点名的「原文文件:行」，没有反向去背景材料里
  找 Group A/B 判定依据的每一句话。
- 没有跑整轮门禁，没有编译除 `driver_e142`/E115 复跑所需之外的任何目标；`repo-copy/` 只编了
  crates 侧 debug 与 research/e7-index-bench 侧 release 两次，没有编译 `crates/` 的 release
  或跑任何测试套件。
- 没有判断主 agent 私下改动主树这件事本身合不合规——那不是这次核查的范围。

草稿与产出：`/tmp/claude-1000/archive-rename-r1-verifier/`（`repo-copy/`、`replay-out/`、
`impl-snapshot-fresh.txt`、`run-fresh.out`、`j4test-copy/`、`j6test-copy/`、`build-e142.log` 等），
没有改动主树任何文件，没有做任何 git 写操作。
