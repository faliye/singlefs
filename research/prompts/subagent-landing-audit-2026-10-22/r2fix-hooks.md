# landing-audit-r2 修补：钩子与协作脚本那一份（tooling-writer，2026-10-23）

批次：records/2026-10-22-子agent结果落地审计.md#7；判决 research/prompts/landing-audit-r2-main-verification.md「三、判决」第一张表「钩子与协作脚本」那一行（K2-3、K2-4、K3-2、K3-4 的 `\bfix`、正推 8 的交回闸拒绝说明）。

## 结论

五条出口都达成：

| 条 | 出口 | 读数 |
|---|---|---|
| 1 K2-3 报告行锚行首 | 两份真派发提示不再报报告行的拒；14 种骨架填好放行；报告行填带 report 的文件照拒；新开关红在新格 | 两份：报告行的拒 0 处（工具员那份另有一条 dispatch-prompt 自己的认领行，见下）；14 / 14 退 0；14 / 14 退 3、各只拒 1 处（报告文件名）；`report-line-unanchored` 红 3 格 |
| 2 K2-4 东京日期切天 | 四个子命令按东京日期切，`--until D` 上界是东京 D+1 日零点；成功句写明；自证造东京次日凌晨开工的不收 | 自证「东京日期切天」两格；`day-cut-by-record-zone` 下两格红；真数据 2026-10-01 到 2026-10-21 收 2125 个 |
| 3 K3-2 三方腿两桶 | 报告在仓（工作树或 git 历史）且同轮有判决记 verdict-archived，其余记新桶；桶表与说明列同步；自证两格 | 新桶 `three-way-report-or-verdict-missing`；`three-way-by-type` 红「判决不在」、`three-way-history-unread` 红「判决在」；真数据 463 条腿全进 verdict-archived、新桶 0 |
| 4 K3-4 `\bfix` 与门槛一句 | fixture、fixtures、fixed-script 不中，fix、fixes、fixed 照中；自证加名字；文件头写门槛没出处 | 词表改成 `\bfix(?:es|ed)?(?![\w-])`；`fix-no-word-end` 下「返工词表」红 |
| 5 正推 8 交回闸拒绝说明 | 写明草稿目录的 `result.md`、三方腿的 `research/prompts/<轮>-*-output.md`；自证同步 | 新格「超长交回的拒绝说明写明报告文件名…」；`limit-hint-unnamed` 下红 |

推翻条件：主工作区这三份的 `--selftest` 有一份不绿；`python3 scripts/agents/dispatch-prompt.py --check` 对两份真派发提示的拷贝（草稿目录 `k23/`）又报「报告行点名的文件名里带 report」或「三方腿报告」那两种拒；`AGENT_COST_BREAK=day-cut-by-record-zone` 下 agent-cost 自证退 0。

## 改过的文件

没新建文件；`scripts/agents/dispatch-prompt.py` 没改（它的自证与 `--check` 照原样跑绿，不用跟着改）。

`git diff --stat -- <这些文件>` 原样（对的是暂存区，里面含上一批没暂存的改动，dispatch-prompt.py 那 59 行全是上一批的）：

```
 .claude/hooks/handback-guard.sh        |  17 +-
 .claude/hooks/runner-dispatch-guard.sh | 223 ++++++++++++++++----
 scripts/agents/agent-cost.py           | 360 +++++++++++++++++++++++++++------
 scripts/agents/dispatch-prompt.py      |  59 +++++-
 4 files changed, 560 insertions(+), 99 deletions(-)
```

只算这一份的改动（开工时拷的原样 `orig/` 对此刻，`git diff --no-index --stat` 末行）：

```
runner-dispatch-guard.sh  1 file changed, 61 insertions(+), 22 deletions(-)
handback-guard.sh         1 file changed, 14 insertions(+), 3 deletions(-)
agent-cost.py             1 file changed, 184 insertions(+), 22 deletions(-)
```

sha256（交回时；开工时依次是 637fb1c9…、47be0d5a…、f1c67258…、774dd0b9…，开工时的那一版与上一批交回报告记的一致）：

```
9b5d3d00f12e37594015e3f8f1f0f9ce9de3d80b5631e7d1c0063a3362d44e1c  .claude/hooks/runner-dispatch-guard.sh
dd5bdf0a54b541aedffa8308e70ad07f4501121598173c1cca002ff3418a4790  .claude/hooks/handback-guard.sh
fd36133a34410bb4de60d651a54a23039e1860bdf5aef305b9d2f0faad4a3148  scripts/agents/agent-cost.py
774dd0b9991c733c9741e842334d94853c672dfdddce4f9d1085163ec6b30fd7  scripts/agents/dispatch-prompt.py
```

写法：两个钩子各经 `scripts/gate/hook-swap.py` 换上一次（规格由草稿全文与原文逐段对出、每处唯一；草稿里 `bash -n`、内嵌 python 编译、喂法全过才换，换上后主工作区再验），两次都退 0；派发闸第一次跑 hook-swap 退 1（喂法写成 Bash 命令、钩子收到的不是 Agent 调用），一个字节没写，改喂法后重跑。agent-cost.py 在草稿里用 Edit 改好，拷成同目录 `agent-cost.py.new` 再 `mv` 换上。每次换上之前核过主工作区那一份与开工时的 sha256 相同。

## 1. K2-3：报告行只在行首、句首认（`.claude/hooks/runner-dispatch-guard.sh`）

改了什么：
- `REPORT_LINE` 改成锚定：标签前面要么是行首（可带 `- * > _`、粗体与「1.」「2)」这类编号），要么是。！？；之后（句首，可带空格与粗体）；句中的（「，报告 <路径>」「交回报告：」）与「」『』“”、反引号里引着的都不认。四种标签与「报告」接冒号或接空白再接路径的认法照旧。旧的不锚写法留作 `REPORT_LINE_UNANCHORED`，只在新开关下用。
- 报告行的 `before`（④ 看否定词的那一段）改成标签前面同一行里的记号；锚在行首句首之后那里不会有否定词。
- 报告行正文里紧跟否定词（不、别、勿、禁止，隔不到五个字、中间没有标点）的文件名不交 ⑰ 判（`judged_tokens`），这是判决 K1 表里辩方 K2-4「报告行写（不要叫 report.md）被 ⑰ 拒」那一条「采：并进攻方 K2-3」的改法；路径交 ④ 与三方腿判形态的 `named` 不变。
- 文件头 ④、⑰ 两条、开关表、成功句、⑰ 拒绝说明同步。

和派发提示写的不一样的两处，都是为了达到出口：
- 认句首。攻方腿那份真提示的报告行写在一行中间：「轮名：landing-audit-r1。前几轮的判决：无（第一轮）。报告：research/prompts/landing-audit-r1-opus-output.md。」。只认行首的话这份提示没有报告行，三方腿形态那一判拒（「一行报告行都没有」），出口过不了。句首之后认，引号里的与逗号之后的仍不认。
- 自证「报告名:「报告写到」后面写 report.md 也拒」的样本原来写在句中（「交回之前，报告写到：…」），改成行首（「报告写到：…」），格名不变；不改它按新判法退 0。

自证格（派发闸，460 种，上一批交回报告记 454 种）：删「报告名:句中「，报告 <路径>」也认」；加——
- 报告名:正文引号里的报告标签不认（「」『』“”与反引号四种，应得 0）
- 报告名:句中逗号之后的报告标签不认（就是删掉那一格的样本，应得 0）
- 报告名:句号之后的报告标签照认（应得 2）
- 报告名:编号列表记号之后的报告标签照认（「1. **报告**：…/report.md」，应得 2）
- 报告名:报告行里否定词后面点名的 report.md 不判（「…/result.md（不要叫 report.md）」，应得 0）
- 三方报告:句号之后的报告行照认放行（照攻方腿真提示的写法，应得 0）
- 三方报告:正文引号里的「报告：」不当报告行（应得 0）

新开关：`RUNNER_DISPATCH_GUARD_BREAK=report-line-unanchored`（退回句中也认）、`report-name-negation-judged`（否定词后面的文件名也判）。打开后原样（`logs/rdg-new-switches.txt`）：

```
== report-line-unanchored
  ✗ 自检：报告名:正文引号里的报告标签不认 应当返回 0，实际 2
  ✗ 自检：报告名:句中逗号之后的报告标签不认 应当返回 0，实际 2
  ✗ 自检：三方报告:正文引号里的「报告：」不当报告行 应当返回 0，实际 2
== report-name-negation-judged
  ✗ 自检：报告名:报告行里否定词后面点名的 report.md 不判 应当返回 0，实际 2
```

文件头点名的项与失败行逐个对（`tools/rdg-switch-check.py`，派发闸不在 break-switch-replay-files.tsv 里，这是我照它的写法粗读的）：报告行相关的八条全对——

```
ok report-name-unchecked rc=1 失败行 15 点名 15 没红 []
ok report-line-labels-split rc=1 失败行 9 点名 9 没红 []
ok report-line-unanchored rc=1 失败行 3 点名 3 没红 []
ok report-name-negation-judged rc=1 失败行 1 点名 1 没红 []
ok three-way-report-unchecked rc=1 失败行 5 点名 5 没红 []
ok report-name-case-sensitive rc=1 失败行 1 点名 1 没红 []
ok report-name-whole-path rc=1 失败行 1 点名 1 没红 []
ok report-name-codeblock-judged rc=1 失败行 1 点名 1 没红 []
```

88 条开关全跑过、每条都退 1；另 19 条我的粗读法对不上（heavy-permission-unchecked、valve-reason-ignored、long-run 与 thread 那一族、snapshot-abandoned 三条、code-fence-loose、code-block-line-rule），点名写的是「<九种类型各一>」这类模板或别的开关条下的续行，19 条里点名报告行或写范围格的 0 条，这一份没碰它们。

红绿样本（`hook-swap.py --feed-only` 喂，喂法 `feeds-dispatch.json`；改前那一版拿开工时的原样搭一棵草稿树经 `--root` 喂，原样如下；换上之后主工作区同一份喂法「喂了 14 条，0 条不对」）：

```
  ✓ 退 0（应当 0） clerk 报告行写 result.md 放行
  ✓ 退 2（应当 2） clerk「报告：」写 report.md 拒
  ✓ 退 2（应当 2） clerk「报告路径：」写 report.md 拒
  ✓ 退 2（应当 2） clerk 句号之后的「报告：」写 report.md 拒
  ✓ 退 2（应当 2） clerk 编号列表之后的「报告：」写 report.md 拒
  ✗ 退 2（应当 0） clerk 正文引号里的报告标签不认（新判法）
  ✗ 退 2（应当 0） clerk 句中逗号之后的报告标签不认（新判法）
  ✗ 退 2（应当 0） clerk 否定词后面的 report.md 不判（新判法）
  ✓ 退 0（应当 0） 攻方腿报告写 research/prompts/x-r1-opus-output.md 放行
  ✓ 退 0（应当 0） 攻方腿句号之后的报告行放行
  ✓ 退 2（应当 2） 攻方腿报告写草稿目录下的 result.md 拒
  ✗ 退 2（应当 0） 攻方腿正文引号里的「报告：」不当报告行（新判法）
  ✗ 退 2（应当 0） 真派发提示 a3652270eb80e4fd0（攻方腿）（草稿目录一行去掉冒号，免建写者锁） 放行（新判法）
  ✗ 退 2（应当 0） 真派发提示 a303d33b0ccd5e68f（工具员）（草稿目录一行去掉冒号，免建写者锁） 放行（新判法）
✗ 只喂不换：.claude/hooks/runner-dispatch-guard.sh 喂了 14 条，6 条不对
```

（后两条喂的是两份真提示把「草稿目录：」改成「草稿目录 」的拷贝，免得 stdin 入口在原派发的草稿目录里建写者锁；报告行一字没动。两份原派发目录里的 .writer-lock 是原派发建的，会话号是本会话，不是喂法建的。）

出口的 `--check` 读数（两份真派发提示原样拷在草稿目录 `k23/`，`python3 scripts/agents/dispatch-prompt.py --check <文件>`）：

| 提示 | 改前 | 改后 |
|---|---|---|
| `k23/a3652270eb80e4fd0.md`（攻方腿） | 退 3：「会拒 2 处（报告文件名、三方腿报告路径）」 | 退 0：「✓ 派发闸照这份提示放行（three-way-attack，查了 37 行提示）」 |
| `k23/a303d33b0ccd5e68f.md`（工具员） | 退 3：「会拒 2 处（报告文件名、写范围）」，另「✗ 认领行」 | 退 3：「✓ 派发闸照这份提示放行（tooling-writer，查了 30 行提示）」，剩 dispatch-prompt 自己的「✗ 认领行：有「草稿目录：…/landing-audit-r1fix-hooks」却没有「认领：…」一行」 |

工具员那份剩的认领行一判（欠账 C792，dispatch-prompt.py 自己判，不经派发闸）不是报告行的拒，这份提示派在认领行要求之前，照它自己的结果列在这里。

14 种类型的骨架（`dispatch-prompt.py <类型> --set 草稿目录=<草稿目录>/skel/<类型> --set 认领用途=出口核对 --batch records/x.md#1`，占位照 `tools/fill-skeletons.py` 填：预计挂钟 2 小时内、删分片续跑多机与「上一段岔路表里还差」、三方腿报告填 `research/prompts/x-r1-opus-output.md`、变异分诊员填 `crates/mutations.tsv`，别的占位填一个词）：

```
填好：clerk、experiment-designer、experiment-runner、implementation-writer、investigator、kb-scribe、mutation-triage、
      prior-art、sweep、three-way-attack、three-way-defense、three-way-forward、three-way-materials、tooling-writer  各 rc=0「✓ 派发闸照这份提示放行」
报告行换成带 report 的文件（三方腿 research/prompts/x-r1-opus-report-output.md，别的 <草稿目录>/skel/<类型>/final-report.md；
设计员与材料员骨架没有报告行，加一行）：14 种各 rc=3，各「派发闸照这份提示会拒 1 处（报告文件名）」
```

（逐行原样在 `logs/skeletons-after.txt` 与 `logs/skeletons-report-named-summary.txt`。）

## 2. K2-4：`--since`、`--until` 按东京日期切天（`scripts/agents/agent-cost.py`）

- 新 `DATE_ZONE = zoneinfo.ZoneInfo("Asia/Tokyo")`（那一行带 `# clock-times:allow`）、`day_start`、`window_bounds`、`local_day`。`--since D` 取东京 D 日零点，`--until D` 取东京 D+1 日零点（不含）；usage、timing、dispatch、outcome 四个子命令的 `--since` 都走 `window_bounds`，`--until` 照旧只 outcome 认。`--since` 只认 `YYYY-MM-DD`（文件头用法写的就是这个；原来别的写法会抛 ValueError 退出，现在报「认不出参数」退 2）。
- 每个子 agent 一行的「开始日期」改按东京日期写（`local_day`）。
- 成功句：「✓ 查了 N 个子 agent（--since …、--until …（含那一天），日期按东京日期（Asia/Tokyo）切天，按首条记录的时刻筛），逐个归了结局桶」。文件头加一段切法说明。
- 自证新格「东京日期切天」（`selftest_date_zone`）：三份样本开工在东京 D 日零点过 30 分、D 日 23 点半、D+1 日零点过 30 分（时间戳写成会话记录那个时区），`outcome --since D --until D` 要收前两份、开始日期都写 D、不收第三份；`timing --since D` 要收三份。
- 新开关 `AGENT_COST_BREAK=day-cut-by-record-zone`（退回按会话记录时间戳的时区切天）。打开后原样：

```
  ✗ 自检：东京日期切天：outcome --since 2026-10-21 --until 2026-10-21 应当收东京 2026-10-21 零点刚过与深夜开工的两个、开始日期都写 2026-10-21，不收东京第二天零点刚过开工的，实际收了 ['a00000000000000d2', 'a00000000000000d3']、开始日期 ['2026-10-21', '2026-10-22']
  ✗ 自检：东京日期切天：timing --since 2026-10-21 应当收 3 个（东京 2026-10-21 零点刚过开工的也收），实际 2 个
```

真数据（`outcome --since 2026-10-01 --until 2026-10-21 --summary --jobs 4`，34 秒，输出 `logs/outcome-1001-1021.txt`）末句：

```
  ✓ 查了 2125 个子 agent（--since 2026-10-01、--until 2026-10-21（含那一天），日期按东京日期（Asia/Tokyo）切天，按首条记录的时刻筛），逐个归了结局桶
```

usage、timing、dispatch 各带 `--since 2026-10-22` 跑过一次，都退 0，标题「（2026-10-22 起）」。

## 3. K3-2：三方腿分两桶（`scripts/agents/agent-cost.py`）

- 三方类型（three-way 开头，照旧）交回了的：交回里点名的 `research/prompts/<名>-output.md`（不进子目录；绝对路径里的这一段也认）有一份在 `prompts_paths` 里，且同轮判决也在，记 `verdict-archived`；其余记新桶 `three-way-report-or-verdict-missing`。
- `prompts_paths`：`git worktree list` 列出的每棵工作树里 `research/prompts/` 下已跟踪与未跟踪未忽略的文件，加上 `git log --branches HEAD --no-renames --name-only` 在 `research/prompts/` 下出现过的路径（删过的也算）。只在窗口里有交回了的三方腿时读一次，真仓上不到 1 秒。
- 轮名：`<名>` 去掉开头的 `_` 之后按连字符从长到短取前缀，第一个有 `research/prompts/<前缀>-main-verification.md` 的就是。
- `BUCKET_GLOSS` 两行：`verdict-archived`「三方腿、交回点名的 -output.md 在仓、同轮有判决」；`three-way-report-or-verdict-missing`「三方腿、交回点名的 -output.md 不在仓或同轮没有判决」。桶名列宽改成按最长的桶名算。「并成调度记录…五行」那张表的「三方腿」一行照旧只含 verdict-archived，新桶落进「其余」那一行（逐个点名），行名没改，调度记录那一行的限定归主 agent。
- 自证：假仓里 fx 轮报告在提交里、判决提交过又删掉；fy 轮报告只在主工作树没提交、没有判决；fz 轮判决只在另一棵工作树、报告哪里都没有。原来那个三方腿（8 号）的交回改成点名 fx 的报告，新加 19 号（fy）、20 号（fz）。两格：「三方腿判决在」（8 号应得 verdict-archived）、「三方腿判决不在」（19、20 号应得新桶）。outcome 的子 agent 从 18 个变 20 个。
- 新开关 `AGENT_COST_BREAK=three-way-by-type`（退回按类型名一律记有效）与 `three-way-history-unread`（不读 git 历史）。打开后原样（「花费」一格跟着红，它数的是 verdict-archived 那一桶的个数）：

```
== three-way-by-type
  ✗ 自检：三方腿判决不在：报告在主工作树而同轮没有判决（fy-r1）、判决在另一棵工作树而报告哪里都没有（fz-r1），应得 three-way-report-or-verdict-missing，实际（预期, 实际）{'a0000000000000013': ('three-way-report-or-verdict-missing', 'verdict-archived'), 'a00000000000000
  ✗ 自检：花费：每个子 agent 折算 1000 + 5 × 100 = 1500，合计应当 30000、三方腿 1 个应当 1500，实际 30000.0、[3, 4500.0]
== three-way-history-unread
  ✗ 自检：三方腿判决在：交回点名的 research/prompts/fx-r1-opus-output.md 在提交里、同轮判决 fx-r1-main-verification.md 提交过又删掉（只在 git 历史里），应得 verdict-archived，实际（预期, 实际）{'a0000000000000008': ('verdict-archived', 'three-way-report-or-verdict-missing')}
  ✗ 自检：花费：每个子 agent 折算 1000 + 5 × 100 = 1500，合计应当 30000、三方腿 1 个应当 1500，实际 30000.0、[0, 0.0]
```

（行尾截在 330 字，原样在 `logs/ac-new-switches.txt`。）

真数据 `--summary` 的桶表（2026-10-01 到 2026-10-21，原样）：

```
## 按结局桶（2026-10-01 起、到 2026-10-21 为止，2125 个，折算合计 7004.91 百万）
桶                                        个数     花费占比  说明
written-landed                          842    50.0%  写了、不低于 70% 的行进了提交或工作树
written-partial                          86     4.2%  写了、30% 到 70%
verdict-archived                        463    28.9%  三方腿、交回点名的 -output.md 在仓、同轮有判决
three-way-report-or-verdict-missing       0     0.0%  三方腿、交回点名的 -output.md 不在仓或同轮没有判决
report-used-by-main                     591    14.1%  只交报告、交回后主 agent 点到
report-used-by-later-sub                  5     0.1%  只交报告、只有后来的子 agent 点到
report-unreferenced                       9     0.1%  只交报告、没人点到
written-overwritten                      22     0.6%  写了、低于 30%、主 agent 点到过
written-unused                            0     0.0%  写了、低于 30%、没人点到
cut-by-usage-limit                       36     1.0%  没交回、用量撞顶
cut-by-api-error                          5     0.1%  没交回、API 错
stopped-salvaged                         51     0.7%  没交回、被停掉、后来有人点到
stopped-abandoned                        15     0.1%  没交回、被停掉、没人点到
in-flight                                 0     0.0%  没交回、还在跑

```

三方腿 463 条全进 verdict-archived、新桶 0。抽查（`tools/leg-check.py`，原样摘）：「腿 463 类型 Counter({'three-way-attack': 238, 'three-way-forward': 163, 'three-way-defense': 62})」「交回点名 -output.md 的份数分布 {1: 459, 2: 4}」「匹配到的判决 225 种」「轮名之后的尾巴 [('-opus-output.md', 226), ('-sonnet-output.md', 225), ('-opus-s2-output.md', 12)]」——轮名取到的都是模型名之前那一段。新桶为 0 是推的原因：交回闸 ② 不让三方腿交回不点名 -output.md 的报告，而这一窗口里的轮都写了判决；调度记录说的「498 条腿里 4 条不在」是另一次核，窗口不同，没对。

## 4. K3-4：`\bfix` 加词尾与门槛一句（`scripts/agents/agent-cost.py`）

- `REWORK_WORDS` 的 `\bfix` 改成 `\bfix(?:es|ed)?(?![\w-])`（连字符也不算词尾）；文件头词表说明同改。
- 自证「返工词表」：算返工的加「Batch 9 merge repin and fixes」「E193 guard fixed」「fix the gate verdict」；不算的加「C531 fixture registrations excluded」「C487 segments-step red fixture」「doc-experiments fixtures 补齐」「fixed-script 流的崩溃点枚举」。
- 新开关 `AGENT_COST_BREAK=fix-no-word-end`（退回 `\bfix`）。打开后：「✗ 自检：返工词表：…判错的 ['C531 fixture registrations excluded', 'C487 segments-step red fixture', 'doc-e…」。
- 改前改后（`logs/rework-regex-before-after.txt`，原样）：

```
'C531 fixture registrations excluded'         改前 True  改后 False
'C487 segments-step red fixture'              改前 True  改后 False
'doc-experiments fixtures 补齐'                 改前 True  改后 False
'fixed-script 流的崩溃点枚举'                        改前 True  改后 False
'Fix E193 guard flush bug and rerun'          改前 True  改后 True
'Batch 9 merge repin and fixes'               改前 True  改后 True
'E193 guard fixed'                            改前 True  改后 True
'fix the gate verdict'                        改前 True  改后 True
'crash_enumeration_fixed_script_stream'       改前 False 改后 False
```

- 真数据返工字样：「描述带返工字样…的派发：313 个，占个数 14.7%，占花费 13.4%」（窗口改了切法，与攻方的 327 个、13.7% 不是同一个窗口，没拆开算两样改动各占多少）。
- 门槛一句：文件头「桶」一段末尾加「70%、30%、10 行这三个门槛（LANDED_AT、PARTIAL_AT、WRITTEN_MINIMUM）没有出处；换值之后各桶怎么变的读数见 research/prompts/landing-audit-r2-main-verification.md K3-1。」；常量那一行上面加一行注释指过去。门槛值没改。

## 5. 正推 8：交回闸超长的拒绝说明写文件名（`.claude/hooks/handback-guard.sh`）

- 改前那一句（`orig/handback-guard.sh` 第 202 行）：「→ 怎么办：全文写进报告文件（派发提示给的报告路径；没给就写进草稿目录），交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。」
- 改后（stdin 喂一份 4001 字的回扫员交回，退 2，stderr 原样，`logs/handback-refusal-after.txt`）：

```
✗ sweep 的交回正文 4001 字，超过 4000 字。
→ 怎么办：全文写进报告文件：三方腿写派发提示给的 research/prompts/<轮>-*-output.md；别的类型写派发提示给的报告路径，没给报告路径就写进草稿目录的 result.md（文件名不带 report）；交回只写结论、报告路径与 sha256sum（.claude/agent-common.md「报告」一节）。
```

- 自证原来没有一格点到这句；新加「超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md）」（应得 True），文件头 ① 条与成功句同步。
- 新开关 `HANDBACK_GUARD_BREAK=limit-hint-unnamed`（退回改前那一句）。打开后原样：「  ✗ 自检：超长交回的拒绝说明写明报告文件名（没给报告路径写草稿目录的 result.md，三方腿写 research/prompts/<轮>-*-output.md） 应当返回 True，实际 False」，退 1。
- hook-swap 喂法 4 条（超长拒、正好在上限放、工具员短交回放、通用 agent 超长不判），草稿里与换上后都对。

## 自证读数（主工作区，交回时这一版）

四份都登记为 light（`.claude/hooks/selftest-registry.tsv`），自己跑的：

| 命令 | 结果 |
|---|---|
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 退 0，「✓ 自检通过（查了 460 种）」（上一批交回报告记 454 种，删 1 加 7） |
| `bash .claude/hooks/handback-guard.sh --selftest` | 退 0，「✓ handback-guard 自检通过（查了 23 种）」（加 1 格；改前没跑） |
| `python3 scripts/agents/agent-cost.py --selftest` | 退 0，「✓ 自检通过（查了 18 项，其中 outcome 的 20 个子 agent）」（上一批交回报告记 14 项、18 个） |
| `python3 scripts/agents/dispatch-prompt.py --selftest` | 退 0，「✓ 自检通过（查了 90 项）」（没改，派发闸换上后复跑） |

开关逐条打开：
- 交回闸（在 break-switch-replay-files.tsv 里）：照 code-tooling break-switch-replay 格的读法（`tools/replay-emulate.py`）「共 10 条，错 0 条」。
- agent-cost（在表里）：同上「共 10 条，错 0 条」，新加的四条都在内。
- 派发闸（不在表里）：88 条逐条跑、都退 1；报告行相关八条点名与失败行逐个对上（见第 1 节）。

语法：两份钩子 `bash -n` 过、hook-swap 编译了内嵌 python；`python3 -m py_compile scripts/agents/agent-cost.py` 过。没改 `.rs`，不跑命名检查。

## 要主 agent 跑的命令

1. 照新切法重出结局账存档（工作目录仓根，不起重活、不经内存包装，读会话记录的进程数默认 12；我在草稿里用 `--jobs 4` 跑过一次，34 秒）：
   `python3 scripts/agents/agent-cost.py outcome --since 2026-10-01 --until 2026-10-21 --summary`
   看末句「✓ 查了 N 个子 agent（…日期按东京日期（Asia/Tokyo）切天…）」与桶表里 `three-way-report-or-verdict-missing` 一行；我这一次是 2125 个、新桶 0。
2. 提交前（归门禁，不是我跑的）：门禁 code-tooling 的 break-switch-replay 格会复跑交回闸与 agent-cost 的开关（新加五条）；派发闸新加两条开关不在复跑表里，要进表的在 `.claude/gate.d/break-switch-replay-files.tsv` 加一行（`.claude/gate.d/` 不归这一份）。

撤回：`hook-swap/` 下的 reverse-*.json 有三份，按文件名次序第一份是派发闸那次没换上的（不用），第二份撤派发闸、第三份撤交回闸：`python3 scripts/gate/hook-swap.py <反向规格> --hook <钩子> --scratch <草稿目录> --feeds <不含新判法的喂法>`。agent-cost.py 撤回拿 `orig/agent-cost.py`。

## 草稿目录里留着的

`/home/fy5090/.local/share/singlefs/work/landing-audit-r2fix-hooks/` 下：`orig/`（开工时四份的拷贝与 `orig.sha256`）、`draft/`（三份改后的全文）、`k23/`（两份真派发提示的原样拷贝）、`skel/`（14 种骨架、填好的与报告行换名的）、`spec-*.json`、`feeds-*.json`、`hook-swap/`（喂法输出与反向规格）、`tools/`（make-spec.py、replay-emulate.py 两份从上一批草稿拷来，rdg-switch-check.py、make-feeds-dispatch.py、fill-skeletons.py、leg-check.py 是这一份写的）、`logs/`。没有仓副本与编译目录；验证用的两棵符号链接草稿树已删。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没走定义三方；没提交。
- 没跑门禁阶段与各类 lint；没改 `.rs`，没跑命名检查。
- 没碰 `.claude/gate.d/`、`scripts/kb/`、`.claude/agents/`、`.claude/agent-common.md`、`.claude/main-agent.md`、`records/`、`research/`、`scripts/agents/subagent_transcript.py`；没改 `dispatch-prompt.py`（它文件头 ⑰ 那句只写四种标签、没写行首句首，判法在派发闸，不影响自证）。
- 报告标签的锚定只看一行之内：跨行的「」引文里某一行以「报告：」起头的，仍当报告行（没造出这种真提示，没处理）。
- usage、dispatch 两个子命令里主会话那部分仍按会话文件的修改时刻收整份文件（`recent`），没改成按每条记录的时刻筛；`--since` 的那一天现在按东京日期算，但收进来的主会话文件里早于它的记录照旧算进去（这一批改前就这样，判决没点）。`--until` 仍只 outcome 认。
- 三方类型照旧按 `three-way` 开头认，three-way-materials 交回不点名 -output.md，有的话会落进新桶；这一窗口里交回了的三方类型只有攻方、正推、辩方三种（463 条），没另分。
- 派发闸开关表里另 19 条我的粗读法对不上的（第 1 节列了名字）没核是不是改前就对不上，也没改；它们不点名报告行的格。
- 交回闸的 `three-way-verifier` 那一支、⑰ 的文件名判法（名字里带 report 一律拒）照旧，判决没要改。
