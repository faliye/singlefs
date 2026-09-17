# sweep 试跑的交回原文（主 agent 从转录里原样取出；该 agent 没写报告文件）

回扫完成（只读，未改任何文件）。范围：整个 `/home/fy5090/code/singlefs` 工作区现状（含未提交改动）。

## 输入复述
量：树表条目宽度（字节）。旧值 148 → 新值 200（2026-09-16，C338 树表条目预留没有余量 定案，预留 24→76）。常量名 `TREE_TABLE_ENTRY_BYTES`（`crates/singlefs-format/src/lib.rs:117`，当前值 200，已提交）。

## 结论先说
**这次改动已经被前面的会话/工作做得非常彻底**，我逐类搜索后只找到 2 处真正过期的表述（均在 `checks-owed.md`），且都是这次 148→200 之前就已存在的**更早一代**（145/112）残留，不含字面 "148"。`crates/`、`research/e7-index-bench/`、`research/mutations/`、harness 测试、门禁脚本、变异表全部已同步到 200/81/6561/76，且用了很多"防复发"写法（计算下标而非硬编码、从产物动态取值而非写死、`saturating_sub` 防溢出）。已登记的 `stale=` 串（9 个）经我逐一核验，现状确实是 0 命中（在门禁 27 号实际扫描范围内）。

---

## 一、按派生形态逐类搜（先数后看，不截断）

### 1. 字面量 "148"
```
grep -rn "148" . 2>/dev/null | grep -v "/\.git/" | wc -l
```
→ **1392** 处。按目录 `grep -rc`：
```
.claude/rules            7
.claude/kb              116
.claude/gate.d            3
.claude/singlefs-ai-sop   2
crates                    0   ← 已完全清空
research/e7-index-bench  22
research/scripts          1
research/mutations        0
research/results        213   ← 冻结产物（e142/e145/e146 各三代产物文件名与内容）
research/prompts       1021   ← 冻结证据（三方论证材料/输出）
records                   9   ← 建设过程记录，按日期
```
逐条读完 `.claude/kb`（116 处）、`.claude/rules`（7 处）、`.claude/gate.d`（3 处）、`research/e7-index-bench`（22 处）、`research/scripts`（1 处）、`records`（9 处）：**crates/ 与 research/mutations/ 干净；research/results 与 research/prompts 是冻结证据，逐字节/逐产物比对通过（E142 第 8/9 次跑、E145、E146 三代产物齐全，`replay.sh` 指向的都是最新一代）；.claude/kb 与 records 里的 148 全部落在「## 历史版本」/决策项内的 ⚠️ 日期前缀历史注 / `decisions-history*.md`（其本身就是变更史，按规则不判红）/ 已撤回条款（D18 已定项 15，正文自陈"不是现行设计"）范围内。**

E7 单独确认了一批**同名不同量**的假阳性（148 恰好是别的量）：`E102`（打包头宽区间 [9,148]）、`E131`（extent 内部扇出）、`E136`/`E137`/`E134`（映射内部扇出）、`E145` 第 388/389/390 行（extent fanout，非树表）、`checks-owed.md`/`decisions-history*.md` 里散落的 `E148`/`C148` 编号（实验号/欠账号，纯数字巧合）。

### 2. 消息串 `expect("148")`
```
grep -rn 'expect("148")' . 2>/dev/null | grep -v "/\.git/"
```
→ 6 处，全部是：规则文件里作为举例的反引号串（2 处：`format-evolution.md`、材料引用）+ `stale=` 标记自身（2 处）+ `research/prompts/` 冻结证据（2 处）。**crates 里真实的 `expect` 消息已全部是 `expect("200")`**（`records.rs:271/273/287`、`transaction.rs:1471`、`make_filesystem.rs:209`）。0 处需要改。

### 3. 按预留宽写的读写 `skip(24)` / `len() - 24`
```
grep -rn "skip(24)" . 2>/dev/null | grep -v "/\.git/"
grep -rn "len() *- *24\b" . 2>/dev/null | grep -v "/\.git/"
```
命中的 `skip(24)`（`crates/singlefs-core/src/superblock.rs:136/142`、`research/e7-index-bench/.../e142_first_transaction_dry_run.rs:1274/1281`）核实后**都是超级块的两个不同字段**（"映射来源"/"整理三条水位"），恰好也是 24 字节，与树表条目无关——**同一个数、不同的量，不改**。树表条目自己的预留读写在 `crates/singlefs-core/src/records.rs:277`（`writer.skip(76)`）与 `records.rs:288`（`bytes[bytes.len() - 76..]`），已经是 76 且用 `bytes.len()` 现算，不是硬编码。`len() - 24` 字面串 0 处出现在真实代码里（只在规则文件与冻结材料的举例里）。

### 4. 下标（值−1，即 `[147]`）
```
grep -rn "\[147\]" . 2>/dev/null | grep -v "/\.git/"
```
→ 8 处：规则文件举例（2 处）、`stale=` 标记（2 处）、`research/prompts/_m2-code-r2-diff.md:1276`（**diff 文件里的删除行** `-    tampered_entry[147] = 1;`，正是这次改动本身的 before/after 记录，冻结证据）。真实测试代码里已改成 `let last_reserved_byte = tampered_entry.len() - 1;`（`crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:1087-1088`），注释明写"下标从条目长度取，不写死（条目宽改过两次）"——**这条纪律本身已经被写代码的人吸收成了写法**。0 处需要改。

### 5. 倍数 `7 × 148 = 1036`
```
grep -rc "7 × 148\|7\*148\|7 \* 148" . 2>/dev/null | ... → 33 处
grep -rc "\b1036\b" . 2>/dev/null | ... → 40 处
```
文件清单去重后**全部落在** `decisions-history.md` / `decisions-history/2026-09.md`（变更史表格的"改前"列，规则明文允许）与 `research/prompts/`（冻结证据）与规则文件举例。`.claude/kb/first-txn-layout.md` 里唯一的一处是 `stale=` 标记自身。0 处需要改。

### 6. 商与平方 `⌊16253/148⌋=109`、`109²`、`11881`
```
grep -rc "每层 109\b" . → 23 处
grep -rc "109²\|109\^2" . → 53 处
grep -rc "\b11881\b" . → 5 处
```
文件清单：`.claude/kb/decisions/08-核心索引结构.md`（当前正文明确写 81/6561，109/11881 只出现在同一段落里带日期的"此前……"括注里）、`decisions-history*.md`（变更史，规则允许）、`.claude/kb/milestone/02-second-txn.md`（"已落地"节里带"此前 109/11881"的括注，日期齐全）、`research/prompts/`（冻结证据）。**crates/ 与 research/e7-index-bench 里 grep "\b109\b" 结果为 0**（`records.rs` 用的是符号 `NODE_POINTER_BYTES` 等表达式，不写死 109）。0 处需要改。

### 7. 测试与门禁脚本里钉的产物值
- `crates/mutations.tsv`：无 TREE_TABLE_ENTRY_BYTES 相关变异条目（这不是本次改动引入的缺口，是这份表本身还没覆盖这个常量——供参考，不算本次回扫的"要改"项，因为它不含任何旧值派生形态）。
- `research/mutations/e142_first_transaction_dry_run.tsv`：`M43_tree_table_entry_197`（200/197，已更新）、`M63_tree_table_reserved_bytes_not_checked`（锚点已改成 76 字节的读写代码），核对：
  ```
  grep -n "M43\|M63" research/mutations/e142_first_transaction_dry_run.tsv
  ```
  确认 0 处停在旧值。
- `research/e7-index-bench/src/bin/e146_livelist_entry_width.rs:20`：`TREE_TABLE_ENTRIES_PER_UNIT = 81`，注释显式写"它随条目宽变，改 TREE_TABLE_ENTRY_BYTES 要一起改这一行"，且减法已换成 `saturating_sub`（第 328 行）——这正是 `.claude/rules/mutation-sampling.md`"改一个格式常量之后，要看『无效』那一栏有没有变多"那条纪律的落地。
- `.claude/gate.d/55-qemu-first-transaction.sh:35-37`：反向链期望值已改成从产物文件动态 `sed` 提取（不写死数字），注释原样记录了"2026-09-16 树表条目 148→200"那次踩坑，属于允许保留的"记坑注释"（design-doc-discipline 例外条款）。
- `.claude/gate.d/88-quoted-result-lines.sh:10-11`：同样是记坑注释，不是活跃引用。
0 处需要改。

## 二、第二遍：按量名/常量名/kb 用词搜，找不含 148 却算同一个量的地方

```
grep -rln "树表条目\|树表单元每层\|每层.*棵" .claude/kb --include="*.md" | grep -v -- "-history"
```
→ 22 个非 history 文件。逐一读完（`decisions.md`、`05/06/08/18/19/22/26-*.md`、`experiments/128/131/132/133/134/136/142/145/146-*.md`、`experiments.md`、`first-txn-layout.md`、`invariants.md`、`milestone/01/02-*.md`）后，找到 **2 处真正的"更早一代残留"**（145/112，不含字面 148，是 2026-09-14 之前那一代）：

### 找到的问题（要改 / 要人看）

**① `.claude/kb/checks-owed.md:575`（欠账总清单汇总行）—— 要改**
原文：`C157（树表容量在两处按不同条目宽算）（112 棵/层，E79（根记录的容量） 重跑仍欠）`
问题：这一行**没有任何日期前缀**，是纯现状陈述，而"112 棵/层"是 2026-09-13 那一代（145 字节条目）的数，已经过时两代（148/109 → 200/81）。不受本仓"日期前缀=历史"惯例保护。
理由：C157 这条欠账本身还开着（E79 仍用 67 字节口径算出 243，需要补一档按现行 200 字节字段表算），但括注里点的具体数字应该跟着 D22 已定项 7 的现行值走，否则读者会以为"当前权威值是 112"。
建议：改成 `（81 棵/层，E79（根记录的容量） 重跑仍欠）`，或干脆去掉具体数字只写"E79 仍按旧口径算，重跑仍欠"。

**② `.claude/kb/checks-owed.md:159`（C157 详细行的"kb 这一半 2026-09-13 已收口"那句）—— 要人看**
原文：`**kb 这一半 2026-09-13 已收口**：D22（单元原子性怎么合成） 已定项 7 统一按 145 字节条目算成 112 棵/层（码 2 三档头同值）；仍欠：……`
问题：带日期前缀（"2026-09-13"），可以辩护为"记录那一天做了什么"，但字面读起来像是在陈述"D22 已定项 7 现在统一按 145 字节……"——而 D22 已定项 7 自己的正文已经两次改口径（2026-09-14→148/109，2026-09-16→200/81），且 D22 文件内部对每一次变动都补了一条 `⚠️ 2026-09-16 从 109 改成 81：……` 的历史注。checks-owed.md 这一行没有跟着补类似的追记。
判不下去的地方：checks-owed.md 里同一份文件的其它行（C338、C363 等）遇到类似情况时，是在原句后面追加新的 `⚠️ 日期：……` 段落而不是改写原句——如果这行也照此惯例处理，那"要不要动它"就取决于"要不要给这条 2026-09-13 的旧记录追加一条 2026-09-16 的更新"，这是内容判断，不是格式判断，所以留给人看。
核对命令（确认它确实不含字面 148、且确实是同一个量）：
```
grep -n "112 棵\|每层 112" .claude/kb/checks-owed.md
sed -n '159p;575p' .claude/kb/checks-owed.md
```

### 确认干净、无需处理的相关位置
- `.claude/kb/decisions/22-单元原子性怎么合成.md:511-517`：D22 已定项 7 正文本身已是 200/81/6561，且下面跟着的 `⚠️ 2026-09-16 从 109 改成 81`→`⚠️ 2026-09-14 从 112 改成 109`→`⚠️ 2026-09-13 从 243 改成 112` 三条历史注逐条可溯源，是本仓这类"item 内嵌历史"的标准写法，干净。
- `.claude/kb/decisions/08-核心索引结构.md:76/469/474/475`：全部已是 200/76/81/6561，历史括注都带日期，干净。
- `.claude/kb/experiments/79-根记录的容量.md:36`：`⚠️ 2026-09-13：…改按…145 字节条目算成 112 棵/层，这里的 243 已作废，加一档重跑欠在 C157` —— 带日期，且明确自陈"已作废"，是历史记录，不是当前权威陈述；它没有跟着后续两次改动继续更新，但它本身没有声称自己是"现在"的状态，判"不改"（历史/欠账追踪的一部分，跟 C157 是同一笔债，不重复登记）。
- `.claude/kb/milestone/01-first-txn.md:250`（"步 5 树表 7 条 × 148 字节……"）：明确在"### 2026-09-14（总审核第二轮定案）"历史小节里，且紧邻它的"### 2026-09-16（其二）"历史小节自陈"2026-09-14 那条历史条目里的『148 字节』是当时的定案……历史条目记的是当时，不跟着现状改"——**这是本次回扫最好的对照组：另一个会话已经明确做过一次这类判断并写下依据**，我核对后认同这个判断。
- `.claude/kb/decisions/18-块里携带什么信息.md:914`：树表条目 145 出现在**已撤回条款**（"已定项 15（已撤回·此编号不再使用）"）里，正文自陈"下面保留的是三轮留下的结论，不是现行设计"，属冻结记录，不改。
- `.claude/kb/decisions/19-块指针的结构与宽度预算.md:117`："树表条目的根指针 83 → 86"——这是指针宽度变化（另一个决策 D19 的产物），不是树表条目总宽度，同一个数不同的量，不改。
- `.claude/kb/decisions/06-快照实现模型.md:249`、`.claude/kb/decisions/05-快照-空间记账机制.md:471`：都是"⚠️ 2026-09-14 的两处改动……树表条目从 145 变 148，依据在 D8 已定项 8……改第一个事务的字节：是——……（200，2026-09-16 用户定案从 148 加宽）"，历史与现状分层清楚，干净。

## 三、`stale=` 候选核验

已登记（`.claude/kb/first-txn-layout.md:378`）：
```
<!-- format-const: TREE_TABLE_ENTRY_BYTES = 200 stale=expect("148")|预留 24 非零|树表条目预留 24|TREE_TABLE_ENTRY_BYTES, 148|7 × 148|1036|每层 109|109²|预留剩 24|[147] = 1 -->
```
我按门禁 27 号（`.claude/gate.d/27-format-constants.sh`）**实际的扫描范围**（`.claude/kb/**/*.md` 排除 `*-history.md` 与 `/decisions-history/`，加 `research/**/*.rs`+`crates/**/*.rs`，排除 `research/results/`；`research/prompts/` 因为既非 kb 也非 `.rs` 本来就不在扫描范围内）逐条核对 9 个串：

```
for s in 'expect("148")' '预留 24 非零' '树表条目预留 24' 'TREE_TABLE_ENTRY_BYTES, 148' '7 × 148' '1036' '每层 109' '109²' '预留剩 24' '\[147\] = 1'; do
  grep -rn -- "$s" --include="*.rs" --include="*.md" . 2>/dev/null \
    | grep -v "/\.git/" | grep -v "research/prompts/" | grep -v "research/results/"
done
```
**9 个串在门禁范围内均 0 命中**（唯一命中都是标记自身或规则文件里的举例反引号，两者都不在门禁扫描的 `kb_files`/`srcs` 集合里）。**结论：已登记的 9 个 stale= 串全部合格，不需要追加或修正。**

我**没有**新增候选：checks-owed.md 里"112 棵/层"目前还不满足"命中 0 次才给"（上面 ① 处仍然命中），要先改掉 ① 才能把"112 棵/层"或"145 字节条目算成 112"这类串登记进 stale=；改完之后可以追加，登记位建议仍用 `first-txn-layout.md:378` 那一条（同一常量不能开第二处登记位）。

## 四、试跑观察

- **搜索命令数**：本轮共执行约 28 条独立的 grep/搜索类 Bash 调用（含几条内部套 `for` 循环、一次遍历 9 个 stale 串、一次遍历约 10 个目录的计数循环），另有约 15 次 `sed`/`cat` 定点读取上下文用于分类判断（不计入搜索命令数）。
- **规则条文与实际操作的落差**：
  1. 「格式常量的旧值的派生形态要逐类搜」这条规则本身给的例子（消息串、预留宽读写、下标、倍数、商与平方、产物值）在本例里**全部已经被处理干净**，唯一漏网的是规则**没有覆盖到的第七种**：不含字面旧值、只是"更早一代"（145/112，先于 148 的一代）的残留数字。这正好印证了任务输入里第 2 条"按量名/常量名/kb 用词再搜一遍"的必要性——如果只按 148 的派生形态搜，① ② 两处永远找不到。
  2. `format-evolution.md` 规则原文对 `stale=` 的登记要求是"每个串现扫一遍 kb 正文与源码，命中 0 次才给"，但没有明说"kb 正文"具体等价于门禁 27 号脚本里的哪个文件集合（是否包含 `checks-owed.md`、`invariants.md` 等非 `-history` 结尾的文件）。我按门禁脚本源码的字面逻辑（`kb_files` 排除 `*-history.md` 与 `/decisions-history/`，不排除别的文件）执行，这一点建议在规则里显式点名一次门禁 27 号的排除清单，免得下次回扫员靠猜。
  3. `checks-owed.md` 这类"欠账表"文件的行文惯例是**在同一个单元格内不断追加带日期的 ⚠️ 段落**，而不是像 `design-doc-discipline.md` 要求的那样把历史挪到文末"## 历史版本"节。这导致"这一行是现状陈述还是历史陈述"经常要靠**有没有日期前缀**这一条弱信号来判断，而不是靠文档结构。① 处（无日期前缀）和 ② 处（有日期前缀）就是同一份文件里，仅因为有没有前缀就分别落进"要改"和"要人看"两类，这个判据本身不够硬，值得主 agent 或 `kb-scribe` 定一条更明确的口径（例如：欠账表的追记必须以 `⚠️ YYYY-MM-DD` 开头，没有前缀的句子一律按现状陈述判断是否过期）。
  4. `.claude/kb/experiments/79-根记录的容量.md:36` 与 `checks-owed.md` 的 C157 详细行讨论的其实是**同一笔债**（E79 用旧的 67 字节口径），但两处各自停在不同的"过期代"（79 号实验文件停在指出"145/112 已作废"，checks-owed.md 停在"145/112 已收口"）——这提示 C157 这笔债本身也该在树表宽度这次改动后补一条追记，但这已经超出"树表条目 148→200 sweep"这个具体任务的边界，只是顺带看到，列在这里供主 agent 参考，不建议我这一轮处理。
  5. `crates/mutations.tsv`（crates 层的变异表）目前没有任何一条变异 `TREE_TABLE_ENTRY_BYTES`，这不算本次 148→200 的"残留"（它从来没含过 148），但意味着 crates 层对这个格式常量目前没有变异测试覆盖，只有 `research/mutations/e142_first_transaction_dry_run.tsv` 覆盖了它。是否需要补，交主 agent 判断。
- **没有做的事**：没有改任何文件；没有跑门禁（`bash .claude/gate.d/27-format-constants.sh` 之类），只是手工核对了脚本源码的扫描逻辑，理论上门禁 27 号现在应该是绿的（因为 kb_files 范围内我没找到任何未在 `-history` 里的字面串命中），但我没有实际执行门禁去验证这一点，按 agent-common.md「不跑 `gate.sh` 全量，除非定义明写要做」的约束没有跑；如果要确认，需要主 agent 或下一轮跑一次 27 号阶段。checks-owed.md 之外，是否还有别的"更早一代残留"未被我的 22 文件二次搜索覆盖到（比如 `records/` 下按日期的建设记录里如果有类似无日期前缀的"现状"陈述），我没有对 `records/` 做第二遍按量名搜索（只做了第一遍按字面 148 搜索），这是本轮潜在的盲区。

## 报告与草稿路径
- 报告全文即本消息（未另写文件，因为字数在合理范围内；如需落盘，可写入 `/tmp/claude-1000/agents/trials/sweep/report.md`，目录 `/tmp/claude-1000/agents/trials/sweep/draft/` 本轮未使用，因为没有需要暂存的中间产物）。
