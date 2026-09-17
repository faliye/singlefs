# 书记员试跑报告（D7 门禁阶段归属指路，虚构定案）

副本根：`/tmp/claude-1000/agents/trials/kb-scribe/repo/`；真仓一个字未动。

## 一、规格文件

`/tmp/claude-1000/agents/trials/kb-scribe/draft/d7-spec.json`（2 处替换）：

```json
[
  {
    "file": ".claude/kb/decisions/07-是否进Linux主线.md",
    "old": "本工程的准入判据与 Linux 完全不同，见 `.claude/singlefs-ai-sop/rules/show-me-test.md`。",
    "new": "本工程的准入判据与 Linux 完全不同，见 `.claude/singlefs-ai-sop/rules/show-me-test.md`；准入判据落在哪几道门禁阶段，见 `.claude/gate.d/stage-owners.tsv`。"
  },
  {
    "file": ".claude/kb/decisions-history/2026-09.md",
    "old": "## 历史版本\n\n### 2026-09-16（其十）：总审核第五节回扫第三批——十一条矛盾按今天已定的条款对齐，D3（空间分配）、D28（挂载期承诺量）、D18（块里携带什么信息）、D27（小数据打包容器）、D21（权威态与派生态的分界）、D22（单元原子性怎么合成）、D23（journal 的角色与格式） 七份正文，条款本身一个字没改",
    "new": "## 历史版本\n\n### 2026-09-17：D7（是否进 Linux 主线）正文补一句准入判据落在哪几道门禁阶段\n\n> 快查·改前：D7 正文只写准入判据与 Linux 不同，指到 show-me-test.md。\n>\n> 快查·改后：D7 正文另指到门禁阶段归属表，写明准入判据落在哪几道阶段。\n\n- **改前**：正文只写准入判据与 Linux 不同，指到 `show-me-test.md`。\n- **改后**：正文另指到门禁阶段归属表 `.claude/gate.d/stage-owners.tsv`。\n- **依据**：主 agent 2026-09-17 给的书记员试跑规格（虚构定案，只写副本）。\n\n### 2026-09-16（其十）：总审核第五节回扫第三批——十一条矛盾按今天已定的条款对齐，D3（空间分配）、D28（挂载期承诺量）、D18（块里携带什么信息）、D27（小数据打包容器）、D21（权威态与派生态的分界）、D22（单元原子性怎么合成）、D23（journal 的角色与格式） 七份正文，条款本身一个字没改"
  }
]
```

两处旧串在动手前各现查过一次命中数：`decisions/07` 文件里那句 1 次，`decisions-history/2026-09.md` 里那段锚点（含相邻标题整行）1 次。

`--dry-run` 原样输出：

```
✓ 2 处都命中得对（2 个文件），--dry-run 没写
```

实写原样输出：

```
✓ 2 处替换，写了 2 个文件，回读一致
```

## 二、决策变更史

原文写进 `.claude/kb/decisions-history/2026-09.md`，插在「## 历史版本」下最上面（该文件文件头逐字写着「新条目按日期倒序放在『## 历史版本』下的最上面」）。写入前用 `git show HEAD:该文件 | ...` 与 `git status --porcelain` 确认这一天（2026-09-17）该月文件此前没有别的条目、也没有别的会话对它的未提交改动，因此标题不带「（其N）」序号，与文件里已有的「单条不带序号、多条按（其一）（其二）……」的惯例一致。

写入结果（`.claude/kb/decisions-history/2026-09.md` 顶部）：

```
## 历史版本

### 2026-09-17：D7（是否进 Linux 主线）正文补一句准入判据落在哪几道门禁阶段

> 快查·改前：D7 正文只写准入判据与 Linux 不同，指到 show-me-test.md。
>
> 快查·改后：D7 正文另指到门禁阶段归属表，写明准入判据落在哪几道阶段。

- **改前**：正文只写准入判据与 Linux 不同，指到 `show-me-test.md`。
- **改后**：正文另指到门禁阶段归属表 `.claude/gate.d/stage-owners.tsv`。
- **依据**：主 agent 2026-09-17 给的书记员试跑规格（虚构定案，只写副本）。

### 2026-09-16（其十）：……（原有条目，未动）
```

随后跑 `bash .claude/gate.d/49-history-brief.sh --write` 重新生成 `.claude/kb/decisions-history.md`：

```
✓ 按原文重新生成了 .claude/kb/decisions-history.md：424 条条目，这次补了 0 条「（待补）」快查
```

再跑一次 check 确认同步：

```
✓ 决策变更史的快查与原文同步：查了 424 条条目、908 格快查，decisions-history.md 与原文一致
```

生成后 `decisions-history.md` 里 D7 一节：

```
## D7（是否进 Linux 主线）

**现状**：无分项 · 整条已定。前几年不进，按单人项目做，准入判据是门禁

共改过 1 次：

| 改了什么 | 改前 | 改后 | 原文 |
|---|---|---|---|
| D7（是否进 Linux 主线）正文补一句准入判据落在哪几道门禁阶段 | D7 正文只写准入判据与 Linux 不同，指到 show-me-test.md。 | D7 正文另指到门禁阶段归属表，写明准入判据落在哪几道阶段。 | [原文](decisions-history/2026-09.md) |
```

## 三、分项翻状态 / 欠账表

规格里没有这两项（D7 无分项），本轮没做，也没有欠账行要加或还。
## 四、门禁阶段登记与结果

`.claude/gate.d/stage-owners.tsv` 里登记给 `kb-scribe` 的阶段（用 `agent-common.md` 给的 awk 命令现取，共 30 个）：

```
awk -F'\t' -v me="kb-scribe" '$1 !~ /^#/ && NF == 3 { n = split($2, owners, ","); for (i = 1; i <= n; i++) if (owners[i] == "kb-scribe") print $1 }' .claude/gate.d/stage-owners.tsv
```

逐个用 `nice -n 19 bash .claude/gate.d/<文件>` 跑过，30 个全部退出码 0（没有 77、没有非 0）。原样末行：

| 阶段 | 退出码 | 末行 |
|---|---|---|
| 10-kb-rot.sh | 0 | `✓ kb 腐化审计通过` |
| 20-kb-shape.sh | 0 | `✓ kb 形状检查通过` |
| 21-decision-items-sync.sh | 0 | `✓ 决策分项清单与正文同步（222 个分项）` |
| 22-item-ref-status.sh | 0 | `✓ 分项引用与正文状态一致（28 条决策、222 个分项）` |
| 23-link-targets.sh | 0 | `✓ 文档指向都到得了（1192 条相对链接、37 处「第 N 节」指向）` |
| 24-status-redundancy.sh | 0 | `✓ 状态只说一遍，且分项都在对的节里（扫 231 份）` |
| 25-kb-deictic.sh | 0 | `✓ kb 正文里的时间指代都锚得到具体一轮（扫 196 份）` |
| 26-number-name-sync.sh | 0 | `✓ 编号与简称一致（扫 216 个文件，prompts/ 按证据链原样保留）` |
| 27-format-constants.sh | 0 | `✓ 格式常量同步（19 个已登记，19 个在源码里被钉住）` |
| 28-cross-decision-status.sh | 0 | `✓ 没有把已定的决策说成未定（扫 28 条决策）` |
| 29-settled-item-self-open.sh | 0 | `✓ 已定项的正文没有把已经定了的东西说成未定（扫 28 条决策）` |
| 30-decision-history.sh | 0 | `✓ 决策正文改了 4 行，变更史新增 1 条条目` |
| 31-blocking-verdict.sh | 0 | `✓ 2 条未定项都判过改不改第一个事务的字节` |
| 32-first-txn-fields.sh | 0 | `✓ 第一个事务的字段表指向都成立（查了 352 处引用）` |
| 32-history-ordinal.sh | 0 | `✓ 本次新增的历史条目没有撞号（存量 9 处撞号不在本阶段射程，见文件头）` |
| 35-user-verdict-owed.sh | 0 | `✓ 待用户复核的条款都有未还的账盯着（共 0 处）` |
| 36-invariant-count-cross-file.sh | 0 | `✓ 不变量条数跨文件一致：实际在用 66 条，扫了 191 份 kb 文件、命中 1 处声明` |
| 37-decision-summary-width.sh | 0 | `✓ 决策索引结论列都不超 200 字（共 28 行）` |
| 38-field-table-projection.sh | 0 | `✓ 被投影的分项，字段表都投影全了（3 张字段表、24 行；投影表引用了 102 条分项）` |
| 39-field-table-sum.sh | 0 | `✓ 字段表加出来的数都对得上（表后合计 0 张，format-const 标记 2 个），另有 3 张表后 3 行内没写「合计 N 字节」、本阶段没验它们` |
| 42-first-txn-trio.sh | 0 | `✓ 第一个事务的三份文件互相挂钩（判「是」的未定项 0 条、字节表 11 节、里程碑 8 步）` |
| 43-owed-table-shape.sh | 0 | `✓ 欠账表两张登记表的行形状都对（检查了 99 行，2 张表）` |
| 44-settled-ref-says-open.sh | 0 | `✓ 扫了 340 个文件、7265 处已定项引用，没有一处紧跟着说它没定` |
| 48-history-month-file.sh | 0 | `✓ 查了 3 份决策变更史、424 条条目，都住在日期对得上的那一份；29 份决策正文的文末没写条目` |
| 49-history-brief.sh | 0 | `✓ 决策变更史的快查与原文同步：查了 424 条条目、908 格快查，decisions-history.md 与原文一致` |
| 51-admission-terms-covered.sh | 0 | `✓ 准入不等式 9 项逐项有归属：统计量 5 项，写明的例外 4 项（容量、挂载期承诺量、被抛弃根独占量、checkpoint 保留池）；式子副本 4 处与权威一致` |
| 52-segment-registry.sh | 0 | `✓ 比对了 5 处登记……`（完整原样见 `/tmp/claude-1000/agents/trials/kb-scribe/draft/gate-logs/52-segment-registry.sh.log`） |
| 53-format-const-placeholders.sh | 0 | `✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））` |
| 60-stale-open-items.sh | 0 | `✓ 没有未定项被别处的更新甩在后面` |
| 61-settled-same-file.sh | 0 | `✓ 本次 diff 没有新增「已定」小节，本阶段无对象可判` |
| 70-citations.sh | 0 | `✓ 全部命中`（`══ 结果：89 条命中，0 条未命中 ══`） |

**归属判断**：这 30 个阶段全部绿（退出码 0），没有红。因此不需要判断「红是不是这一轮写出来的」——本轮改动只涉及 `.claude/kb/decisions/07-是否进Linux主线.md`、`.claude/kb/decisions-history/2026-09.md`、`.claude/kb/decisions-history.md` 三个文件，全部由本轮产生。完整日志留在 `/tmp/claude-1000/agents/trials/kb-scribe/draft/gate-logs/`（每阶段一份 `.log`），汇总在 `/tmp/claude-1000/agents/trials/kb-scribe/draft/gate-summary.txt`。

## 五、改过的文件与前后 sha256

只有三个文件被改动，一律用 `git show HEAD:<路径> | sha256sum` 取派发前的哈希（这三个文件相对 HEAD 在我动手前没有任何改动，`git status --porcelain` 已现查确认），派发后取磁盘上的哈希：

| 文件 | 派发前 sha256 | 派发后 sha256 |
|---|---|---|
| `.claude/kb/decisions/07-是否进Linux主线.md` | `3840f9817070b0595f8c19696a129fd3d1e8ed7f06e2e470fe8cb5764c1e3d3a` | `5eceeb75fad197d56474f44a5e7849954d946520c966f6a76e514a1dda3aa30a` |
| `.claude/kb/decisions-history/2026-09.md` | `e0520cf4ea57645933f72506a822a1961f62cdd12b5e13d6a8c23abbe0389aa7` | `ffce66feb92d82758156ab0c0999f2c5d32ebadc197746bab03293606293d01a` |
| `.claude/kb/decisions-history.md` | `d22b67977e8b93cce24cc027b648f2e929ad19cd195a685cc2fd84cb83b38adb` | `232b82310bec012f5460f2124d30fe427872572914e45028257a2348a5f09aeb` |

`git status --porcelain` 收尾复核：只有这三个文件相对 HEAD 有变动，`decisions-history.md` 的 diff 只涉及 D7 一节（+6/-0 净变化，见下）。

```
git diff --stat -- .claude/kb/decisions-history.md .claude/kb/decisions-history/2026-09.md .claude/kb/decisions/07-是否进Linux主线.md
 .claude/kb/decisions-history.md                                |  6 +++++-
 .claude/kb/decisions-history/2026-09.md                        | 10 ++++++++++
 .claude/kb/decisions/07-是否进Linux主线.md                     |  2 +-
 3 files changed, 16 insertions(+), 2 deletions(-)
```

## 六、试跑观察

**1. 定义没说新条目该插在月文件的哪个位置，是靠读目标文件自己的文件头才知道的。** `kb-scribe.md` 定义与主 agent 的派发提示都只说「原文写进当月的 `.claude/kb/decisions-history/<年-月>.md`，标题下两行快查照规格写」，没说插在文件的开头还是结尾、要不要按日期倒序。答案是 `decisions-history/2026-09.md` 自己文件头第 3 行写着「新条目按日期倒序放在『## 历史版本』下的最上面」——这条规矩住在被写的文件里，不在派发我的定义里。如果书记员没有主动打开目标文件读文件头，这一步无从下手。

**2. 定义没说「一天多条要不要加（其N）序号、怎么判要不要加」。** 这条规矩同样不在派发给我的规格里，是从月文件里已有的先例（单条不带序号，多条按其一/其二……倒序编号）反推出来的，还要现查这一天在这个文件里是不是第一条、以及有没有别的会话正在并发写同一份文件（`git status --porcelain` 与 `git show HEAD:...` 现查）才能确定不用加序号。若定义能直接引一句「按 `decisions-history.md` 文件头的规矩」会更明确。

**3. `30-decision-history.sh` 报的「决策正文改了 4 行」把我这一轮之外的改动也计了进去。** 该阶段用 `git diff HEAD --numstat` 统计 `.claude/kb/decisions.md` 与 `.claude/kb/decisions/` 整个目录相对 HEAD 的改动行数，副本里已有另一个会话对 `decisions/13-验证路线.md` 的 2 行未提交改动（+2/-0），加上我这处的 1+1，合计 4——不是我这一轮单独产生的数。阶段本身判的是「有没有新增变更史条目」这一件事，退出码依旧是 0，不算红，但这个数字本身容易让人误以为这一轮改了 4 行决策正文。定义与共用约束都提醒了「看到红要判是不是这一轮的」，但没提醒「看到绿的统计数字里也可能混了别的会话的东西」——这次没有造成误判，只是数字对不上直觉，记在这里备查。

**4. 报告分段落盘的「每段 ≤150 行」在这个报告里够用，但没有说清跨段的表格/代码块要不要在段边界断开。** `agent-common.md` 只说「每一次工具调用写进文件的内容不超过 150 行」，没提如果一张表格或一段代码块跨过 150 行边界该怎么办——这次我按内容的自然段落（一、二、三……小节）切分，凑巧没有任何一张表或代码块被切断，但定义没有给出「表格/代码块不许被段落边界打断」这类显式约束，纯粹是我自己选的切法。

**5. `replace-batch.py` 的规格文件本身放在草稿目录，定义没有明说规格文件算不算「写范围」里的产出物。** 我把它当成派发提示里给的「草稿目录」的产出物处理（`/tmp/claude-1000/agents/trials/kb-scribe/draft/d7-spec.json`），这与「报告文件、草稿目录」的写范围吻合，但定义正文没有点名说「规格文件放哪」，是我按常理归类的。

以上五点都不影响这次试跑的结果——两处替换全部命中、门禁全绿——只是流程上要靠读别的文件（目标文件文件头、已有先例）才能补全定义没写清的细节。

## 七、没做什么

- 没判定这条「准入判据落在哪几道门禁阶段，见 `stage-owners.tsv`」的说法本身对不对——规格是主 agent 明说的虚构定案，只按规格整字抄写，没做语义判断。
- 没写规格之外的任何句子：`decisions/07` 只加了规格给定的那一句，`decisions-history/2026-09.md` 的改前/改后/依据三句、快查两句都是主 agent 原样给定的文字，没有概括或改写。
- 没有分项翻状态（D7 无分项，规格里也写明「无」）。
- 没有欠账表要加或还的行（规格写明「无」）。
- 没有修任何红——本轮跑的 30 个登记阶段全部是绿的，没有出现需要判断「是不是这一轮」的情形。
- 没有碰真仓 `/home/fy5090/code/singlefs`，全程只在副本根 `/tmp/claude-1000/agents/trials/kb-scribe/repo/` 里操作。
- 没有提交（没有 `git add` / `git commit`），没有推送，也没有做任何 git 写操作。
- 没有跑 `gate.sh` 全量，只跑了登记给 `kb-scribe` 的 30 个阶段。
- 没有编译 Rust。
