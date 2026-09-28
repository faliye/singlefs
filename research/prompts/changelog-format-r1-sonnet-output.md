# changelog-format-r1 正推腿报告

角色：正推——从 `.claude/rules/changelog-format.md` 与已改完的门禁/脚本（`30-decision-history-entries.sh`、
`history-ordinal.sh`+`history-ordinal-keys.py`、`doc-lint.sh` 的 `history-registry` 牌、
`migrate-changelog-format.py`）逐条推出「按规则该是什么」，再与实际代码/实际写出的 kb 内容逐格比。
六问逐条作答，能命令核的都现跑了命令，原样贴输出。

## 问题 1：shape 格与 history-ordinal.sh 有没有缝隙

**推出的「应该是什么」**：规则 `.claude/rules/changelog-format.md`「门禁管哪一半」一节写明
`shape` 只判「同一节里日期不重复」（`### ` 标题层面），没有声称它管 `#### ` 子标题的撞号；
门禁脚本自己在 `gate-similar` 里显式把撞号判定整个划给上游 `history-ordinal.sh`。
按这两处文字推：`shape` 对「同一 `### ` 下两个 `#### 已定项 7（其一）`」这种输入应该放行（不是它的射程），
真正该拦的是 `history-ordinal.sh`。

**对比对象**：

- 「同一个 `## D<n>（` / `## E<n>（` 节里，同一个日期的 `### ` 标题出现两次以上（按标题开头的日期比）。」（`.claude/gate.d/30-decision-history-entries.sh:45`）—— 判据字面只提 `### ` 标题重复，不提 `#### ` 内容重复。
- 「`shape`（日期与序号分属 `### ` / `#### ` 两层、两份变更史的日期块住在自己的 `## D<n>（` / `## E<n>（` 节、同一节里日期不……重复——后两条只判 `decisions-history.md`、`experiments-history.md`」（`.claude/rules/changelog-format.md:53-56`）—— 与门禁脚本的判据字面一致，都只说日期不重复，不说 `#### ` 内容重复。

**一致**：规则怎么写，门禁就怎么判——两处都只覆盖「`### ` 日期重复」，不覆盖「同一 `### ` 下 `#### ` 子标题撞号」，且门禁自己写明这条缝隙由 `history-ordinal.sh` 补：「gate-similar: history-ordinal.sh 它也读变更史的条目标题、也只判这一轮新增的，但判的是……撞号，住在上游副本里由上游门禁管，项目本地改不了它」（`.claude/gate.d/30-decision-history-entries.sh:5`）。

**现查坐实**：构造背景材料问题里举的具体输入——同一个 `### 2026-09-28` 下两个
`#### 已定项 7（其一）：……`（内容不同），一份提交过的基线加一份未提交的新增。

```
$ bash .claude/gate.d/30-decision-history-entries.sh --check shape /tmp/claude-1000/changelog-format-r1-forward/q1-test
── shape：变更史的形状：日期与序号分属两层、条目住在自己的节、同节日期不重复
  ✓ 变更史的形状对：查了 1 份 kb 文件、1 个 `### 日期` 标题（两份变更史里 1 个），只判基准 HEAD 之后新增的行，新增的没有违例；存量……0 处、0 处、0 组
  ✓ 决策变更史 1 格里 1 格通过（shape），0 格本次无对象可判（无）
```

`shape` 确认放行（如推导所料，不是它的射程）。同一份现场喂给 `history-ordinal.sh`：

```
$ bash /home/fy5090/code/singlefs/.claude/singlefs-ai-sop/scripts/history-ordinal.sh /tmp/claude-1000/changelog-format-r1-forward/q1-test
  ✗ 本次新增的历史条目里有 1 处撞号
     .claude/kb/decisions-history.md  「D1（示例决策）」2026-09-28 已定项 7（其一）  这把钥匙下现有 2 条，其中 1 条是本次新增
     → 怎么办： 取号前先查同一个『## 节 + 日期 + 点名词』下已经用到的最大号……
EXIT=1
```

`history-ordinal.sh` 真的抓到了（`stem_and_ordinal` 把「已定项 7」当点名词、「（其一）」当编号，
键 = `(## D1（示例决策）, 2026-09-28, 已定项 7, （其一）)`，两条子标题撞同一把键——机制见
「新嵌套写法（H4 子标题）的键：编号 +（点名词，没有就用去掉编号后的整句）。」（`.claude/singlefs-ai-sop/scripts/history-ordinal-keys.py:38`）。

**结论**：这条具体输入上两道门禁分工严丝合缝，不是缝隙；`shape` 不判是设计如此（规则与脚本注释一致），
`history-ordinal.sh` 接得住。**什么会推翻它**：换一种输入——两个 `#### ` 子标题**没有**点名词
（`已定项 N`/`未定项 N`/`E<n>（`）、且去掉编号后的整句字面也不同（`stem_and_ordinal` 退回「去掉编号后的整句」
当键），这种情况下两条子标题各自的键天然不同，`history-ordinal.sh` 也不会判它们撞号——但那时它们本来就不是「同一件事」的
重复编号，不构成本题问的「误判/漏判」。真正的缝隙候选是：**两个 `#### ` 子标题点名词相同、但没写编号**
（`（其N）`）——`history-ordinal-keys.py` 的 `keys_of` 只在 `ordinal` 非空时才产出键，这种情况下两道门禁都不判——但这落在
`.claude/rules/changelog-format.md` 自己的放宽范围内（同一天只有一条改动可以不写编号），只有点名词相同又都不写编号
才会真撞，且这已经不是「编号撞号」问题，是「同一节同一天该不该拆」的问题，`shape` 的字面判据没覆盖，
**这是本题问法之外的另一个候选缝隙，本报告只指出、不展开验证**（草稿目录里没有为它单独构造样本）。

## 问题 2：`--mention-scope heading-and-quick` 会不会漏掉真正相关的决策

**推出的「应该是什么」**：脚本自己的用法说明写着默认值与回退规则——「--mention-scope  点名从哪里认：all（默认，标题加整段正文）；heading-and-quick（标题加……认不出一个索引表里有的号时退回 all）。」（`research/scripts/migrate-changelog-format.py:47`）。按这句推：**默认跑法（不传这个参数）用的是 `all`，会扫整段正文（含依据段）**，
问题里描述的「正文里提到的决策不算」这个风险只在**显式传 `--mention-scope heading-and-quick`**
时才可能出现；而且它写明了一条回退——「认不出一个索引表里有的号时退回 all」。

**对比对象**：`mentioned()` 函数实现（`research/scripts/migrate-changelog-format.py:312-319`）：

```
def mentioned(entry, kind, scope, present):
    regex = MENTION[kind]
    if scope == 'heading-and-quick':
        text = '\n'.join([entry.raw_heading] + [line for line in entry.body if QUICK_LINE.match(line)])
        found = {int(number) for number in regex.findall(text)}
        if found & present:
            return found
    return {int(number) for number in regex.findall('\n'.join([entry.raw_heading] + entry.body))}
```

**一致但帮助文本没说全**：回退条件是「`found`（标题+快查里认出的号）与 `present`（索引表里真有的号）
的交集非空」才不回退、直接返回 `found`；**只要标题里已经点中至少一个真实存在的决策号，哪怕正文里另外
点名了别的决策，也不会回退到 `all`，那个只活在正文里的号会被丢掉**——帮助文本只讲了「一个都没认出」的情形，
没提「认出了一个、但正文里还有别的」这半句。用直接调用该函数验证：

```python
present = {5, 9}
entry.raw_heading = "### 2026-09-28：D5（示例） 相关的一条改动"
entry.body = ["", "依据：这条改动其实也影响 D9（另一个决策） 的边界条件", ""]
mentioned(entry, 'D', 'all', present)                => {9, 5}
mentioned(entry, 'D', 'heading-and-quick', present)   => {5}      # D9 被丢了
```

**冲突（帮助文本 vs 实际行为，两边并排）**：

- 帮助文本：heading-and-quick（标题加「> 快查·」几行，认不出一个索引表里有的号时退回 all）（`research/scripts/migrate-changelog-format.py:47`，转述其中「认不出……时退回」半句）
- 实际代码：`if found & present: return found`（`research/scripts/migrate-changelog-format.py:317-318`）——
  回退条件是「一个都没交上」，不是「有认不出的」；标题+快查只要命中一个真实号，正文里另外提到的第二个号
  照样不进 `found`，也不算「认不出」，不触发回退。

**这一轮实际有没有踩中**：仓里搜不到这一次迁移调用 `migrate-changelog-format.py` 时传了什么参数
（工作区改动未提交，没有单独的调用日志留存，`research/prompts/_changelog-format-r1-background.md`
本身也没写出调用命令行）——**复核不了实际用的是哪个 scope**。但代码默认值是 `all`
（`research/scripts/migrate-changelog-format.py:1290`：`default='all'`），若这一轮没人手动加
`--mention-scope heading-and-quick`，则问题里描述的风险场景不适用于这次真实迁移产物，
只是代码里一个真实存在、可被复现的缺口（已用上面的直接函数调用坐实）。

**什么会推翻它**：找到这一次迁移的实际调用命令行（例如某条 shell 历史、某份留存日志）显示确实带了
`--mention-scope heading-and-quick`；或者在 `decisions-history.md`／`experiments-history.md` 里找到一条
历史条目，它的「依据」段字面点名了某个 `D<n>（`，而那个决策的 `## D<n>（` 节里却没有它的副本
（可用 `history-ordinal-keys.py` 之外另写一个小脚本比对，本报告没有做——见「没做什么」）。

## 问题 3：`status-sync` 的「现状」行会不会说谎

**推出的「应该是什么」**：`.claude/rules/changelog-format.md` 只定义了「现状行要与索引表一致」这条同步关系，
没有定义「索引表本身要怎么核实」——规则射程止步于「两处文字一致」，不含「文字本身对不对」。
按这条推：`status-sync` 该做的只是字符串比对，索引表滞后时它没有立场判红，这是规则设计的射程边界，不是脚本漏做。

**对比对象**：

- 「⚠️ 判不了：索引表那一行本身写得对不对。」（`.claude/gate.d/30-decision-history-entries.sh:62`）
- 「**它管不了的**：摘要写得对不对、一条改动该点名的决策或实验有没有点全——这几样靠人。」（`.claude/rules/changelog-format.md:59`）

**一致**：门禁脚本自己在文件头明写了这条局限，规则文件虽然没有逐字重复「索引表本身对不对」这半句
（规则只列了「摘要」「点名有没有点全」两项），但规则末句写着「具体判据以那份脚本的文件头为准」——
把细节判据的权威位置指回脚本，脚本那句就是权威解释，不算规则与脚本冲突，是规则有意把细则外包给脚本注释。

**这算不算这条检查本身的一个设计缺口，该不该本轮补**：这是要不要收严的判断题，不是「现状是什么」的事实题，
不属于正推腿要判的格（正推只比「规则推出的样子」与「代码/kb 实际的样子」一不一致，取舍留给主 agent）。
本报告只把机制摆清楚：`judge()` 函数（`.claude/gate.d/30-decision-history-entries.sh:444-481`）逐字比对的是
`decisions.md`/`experiments.md` 索引表里当下写的字符串与变更史节顶的字符串，函数体内没有任何一步回读
决策正文本身去核实索引表那句是不是真话——机制上确实做不到题目描述的那种检测（索引表被推翻却没更新时，
两边字符串仍然一致，`status-sync` 只能判「一致」）。

**什么会推翻它**：`status-sync` 的 `judge()` 函数（同上）被改出一条独立回读决策正文状态行、
交叉核对索引表「状态」格的逻辑；或者找到一次索引表已知滞后、但 `status-sync` 仍判红的真实样本
（目前样本目录只覆盖「现状行与索引表本身对不上」，不覆盖「索引表与决策正文对不上」——见
`.claude/gate.d/30-decision-history-entries.sh:69`）。

## 问题 4：`migrate-changelog-format.py` 的并发安全够不够

**推出的「应该是什么」**：脚本文件头写明了自己的并发模型——「并发：读每份输入时记 sha256；每份输出换上之前（临时文件写完、改名之前）再算一次它依赖的全部输入，变了就跳过这一份、」（`research/scripts/migrate-changelog-format.py:34`）。按这句推：这是「读时戳 + 写前复核 + 复核后立刻改名」的乐观并发，窗口理论上存在（复核与改名之间不是一个原子操作），
但窗口只跨一次 `check_before_rename()` 调用到 `os.replace()`/`os.remove()` 之间，量级是一次函数调用，不是整个脚本的运行时间。

**对比对象**：`write_plan()` 与 `replace_file_contents_by_rename()` 的实际实现：

- 写入路径：`recheck()` 比对 sha256（`research/scripts/migrate-changelog-format.py:775`），
  在 `replace_file_contents_by_rename()` 内部于 `os.replace()` 之前调用（`check_before_rename` 抛异常就不换，
  不抛就紧接着 `os.replace(temporary_path, target_path)`，`research/scripts/lib_atomic_replace.py` 里
  `replace_file_contents_by_rename` 函数体）。
- 删除路径：`sha256_of(...) != digests[relative]` 检查之后紧跟 `os.remove(...)`
  （`research/scripts/migrate-changelog-format.py:794-798`）。

**一致**：代码行为与文件头声明的并发模型一样——都是「复核 sha256，变了就跳过，没变就立刻替换/删除」，
没有文件锁、没有 `flock`（全脚本与 `lib_atomic_replace.py` 都搜不到 `lock`/`flock`/`fcntl`）。
**窗口有多宽**：写入路径的窗口是 `sha256_of()` 返回到 `os.replace()` 系统调用之间那几行 Python 字节码
（比对一个字典查找、一个函数返回、进一层调用栈），删除路径同理，是 `sha256_of()` 到 `os.remove()` 之间的几行。
两者都是微秒级的窗口，不是「读输入到写完」那段秒级窗口——脚本已经把窗口从「整个搬迁过程」收窄到了「一次系统调用之前」，
这正是 `check_before_rename` 这个设计点本身要做的事。

**值不值得再收紧**：这是要不要花代价把微秒级窗口降到零（原子替换加文件锁）的取舍题，不是正推腿判事实的范围；
本报告只坐实「窗口确实还在，且比乐观并发通常能做到的更窄，不是没做防护」。

**什么会推翻它**：写一个并发测试——两个进程同时跑这个脚本对同一份仓、且能在 `sha256_of()` 返回之后精确地
在另一个进程里改动同一份文件（需要类似 `patched_os_function` 那样的注入点），观察是否真的出现「复核过了但用的是旧内容」；
`lib_atomic_replace.py` 自带的 `problems_with_reader_and_inode` 等自证函数目前只测「读者进程不被半份文件坑」，
不测「两个写者互相踩」，这类测试目前**没有做**（见「没做什么」）。

## 问题 5：自引用（「本文件第 N 行」「某日（其N）」这类）有多少条，这一轮改没改全

**推出的「应该是什么」**：`decisions-history.md` 文件头自己写了这条使用说明——「⚠️ 条目里的分项编号一律是今天的编号：2026-08-30 起每条决策的分项只有一套编号，那一次把早于那天的条目也同步改写了。某一项当时是什么状态，看记它的那一条自己的改前、改后、依据。」（`.claude/kb/decisions-history.md:7`）。按这条推：文件本身承认「历史条目里的编号是写下时的旧编号，不保证随迁移同步」，这次改动是否需要把全部
「某日（其N）」都改成对当下有效的锚点，取决于那条引用是**逐字保存的历史快照**（不需要改，本来就该指向写它
那天的状态）还是**打算被读者点开就能找到的活链接**（迁移改了编号规则，理应同步）——规则文件没有明说这条界限，
只能靠上下文判断，这正是背景材料自己点名「已知修了两处」（D6/D22）之外，没说清「还有多少」的地方。

**对比对象**：直接在两份变更史里数「日期（其N）」这个形态出现了多少次：

```
$ grep -cE '20[0-9]{2}-[0-9]{2}-[0-9]{2}（其[^）]*）' .claude/kb/decisions-history.md
193
$ grep -cE '20[0-9]{2}-[0-9]{2}-[0-9]{2}（其[^）]*）' .claude/kb/experiments-history.md
120
```

合计 **313 处**，远多于背景材料点名已核实的 2 处（D6/D22 互相引用「2026-09-06（其三十六）」那两处）。
抽样看其中一批：「撤回依据全文在 [decisions-history.md](decisions-history.md) 2026-09-06（其十五）。」（`.claude/kb/decisions-history.md:47`）——这一句落在一段用 text 围栏包起来的代码块内，是逐字照抄当年 `decisions/01-数据可移动性-反向索引.md` 原文的历史快照（该段上方写明这是「改前：挪出正文或改写了的原行，逐字照抄如下」），按文件头第 7 行的说明属于「写下时的旧编号」，本来就不指望随迁移同步，而且落在围栏内，天然被 `shape` 格的围栏排除逻辑排除在判定之外（`.claude/gate.d/30-decision-history-entries.sh` 的 `FENCE` 处理）。但另一批不在围栏内、是**当下叙述语气的活引用**，例如
`.claude/kb/decisions-history.md:987` 开头一句提到「2026-09-19（其十三） 那次瘦身把……」，这条没有说明「按旧编号，
去 git log 里找」，读起来像是可以直接跳去当前文件里找 `2026-09-19` 那天挂着「（其十三）」的那条，而迁移之后
同一节同一天下的编号是按新分组规则重新计算的，**能不能在当前文件里直接找到这个编号，没有验证过**。

**这一轮改没改全**：无法从「313 处」这一个数直接判——313 处混杂了「围栏内的逐字历史快照」（不该改，
已经被 `shape` 的围栏排除逻辑天然豁免）与「围栏外的活引用」（可能因重编号而失效，需要人工分辨）两类，
区分需要逐条读上下文，本报告没有把 313 条逐条分类（工作量超出正推腿这一轮的核查范围，见「没做什么」）。
可以确认的是：`migrate-changelog-format.py` 的自检套件里没有任何一项检查涉及「日期（其N）」这种跨条目自引用锚点——
全文搜「锚点」「anchor」「自引用」零命中于该脚本，自检只核「次数与正文」（改坏一个字、少抄一份、多抄一份），
不核「文字里提到的旧编号，搬迁后还能不能被找到」。这与背景材料的判断一致：「migrate-changelog-format.py 的自检没有专门查这一类自引用会不会因为搬迁而失效」（`research/prompts/_changelog-format-r1-background.md:98`）。

**什么会推翻它**：把 313 处逐条按「是否在围栏内」「引用的日期块在目标决策/实验节里是否存在同编号的 `#### `」
两条规则脚本化扫一遍，报出真正会指向空处的条数——这项工作目前没做，属于欠账，不是已经排除的假警报。

## 问题 6：`<!-- doc-lint:history-registry -->` 这张牌是不是开得太宽

**推出的「应该是什么」**：`doc-lint.sh` 文件头写明这张牌把 D、I、A 三类检查的豁免起点从「文件自己的历史节」
挪到了「第一个 `## ` 标题」——豁免的检查类别是「D、I 与 A（词表比对、上下文/自指/时间指代、历史陈述）」（`.claude/singlefs-ai-sop/scripts/doc-lint.sh:40`）。
按这句字面推：射程是「这两份文件里从第一个 `## ` 标题起的全部内容」，即 591+263 条原条目的全部历史正文，
不是「偶发的几条」——这与背景材料的描述（两份文件里全部的历史正文）在字面上一致，不是转述夸大。

**对比对象（现查坐实豁免范围有多大）**：把 `decisions-history.md`、`experiments-history.md` 各拷一份到
草稿目录、删掉 `<!-- doc-lint:history-registry -->` 这一行，其余不动，跑一遍 `doc-lint.sh`：

```
$ bash .claude/singlefs-ai-sop/scripts/doc-lint.sh /tmp/claude-1000/changelog-format-r1-forward/q6-test
……（节选）
  ✗ .claude/kb/decisions-history.md:2188  正文不许写"原先 X"，直接改成现行值
  ✗ .claude/kb/decisions-history.md:5013  正文不许写"原先 X"，直接改成现行值
……
```

按违规行统计（对日志文本按 `^\s*✗\s+\.claude/kb/<文件>:\d+` 逐行计数）：

| 文件 | 去掉牌之后新冒出的 ✗ 行数 | 现状（带牌） |
|---|---|---|
| `.claude/kb/decisions-history.md` | 756 | 0（这些类别一处都不报，见下方现状验证）|
| `.claude/kb/experiments-history.md` | 103 | 0 |

按违规类别拆（出现次数最多的几类，两个文件合计）：531 处「本轮」锚不到具体哪一轮（时间指代）、
92 处上下文指代、59 处自指称呼，其余 172 处分属历史陈述词表各条（「此前写的是 X」「已经不是现行」
「原先 X」「此前不存在」「曾经写作」等）。

**现状确认带牌之后这些类别确实是 0**：真仓上曾经真跑过这道检查——背景材料点名的产物
`/tmp/doclint-final.log`（这一轮之前跑的，仍在磁盘上）末行：

```
  ✗ 文档铁律检查失败：2 个文件违规、0 处编号引用无定义、21 处编号定义/引用不合规、0 条排除项不合规、0 处规则清单/警告记录不合规（检查 416，跳过 0）
```

其中报出的「2 个文件违规」是 `experiments/142-新池新建文件的干跑.md`、`experiments/162-崩溃放量判定块存储选型.md`
两份不带 `history-registry` 牌的文件里裸引用编号未带简称，与本题问的 D/A 类无关；
`decisions-history.md`/`experiments-history.md` 两个文件名字面上完全不出现在这份最终产物里——
与「贴上牌之后这两个文件在 D/A/词表类里一处都不再被点名」这句描述一致。

**一致（背景材料描述 vs 脚本注释 vs 实测规模）**：三处对得上——脚本注释说射程是从第一个 `## ` 起的全部内容，
背景材料说放开的是两份文件里全部的历史正文，实测显示去掉牌之后单这两个文件就新冒出 859 处（756+103）D/A 类
违规。**这不是一个小范围豁免**：作为对比，共享规则原本设计的豁免对象是「一份 kb 文件自己文末的『## 历史版本』」，
通常只是文件末尾几十行；而这张牌覆盖的是两份文件的全部正文（`decisions-history.md` 全文 4200+ 行规模，
`experiments-history.md` 同量级），豁免面积差了两个数量级。

**该不该照搬、更合适的做法是什么（只放开 D-3 还是维持现状）**：这是设计取舍题，判决权在主 agent 与用户，
不是正推腿要给出的事实判定。本报告只交出可供判断的数字：若只放开时间指代一类，D-1/D-2/A 三类仍会
报出至少 756-531=225 处（decisions-history.md 一侧，按上面统计的合计类别数粗算）；`experiments-history.md`
内时间指代与其余三类各自的条数，本报告只统计到两个文件合计，没有单独按文件拆开重跑（见「没做什么」）。

**什么会推翻它**：如果按「只放开时间指代」重新贴牌（需要新写一张更窄的牌或者改 `doc-lint.sh` 认多个牌名），
跑出来上下文指代/自指称呼/历史陈述三类降到个位数或零——说明当前「全部四类一起放开」确实是过度豁免；
反过来，如果逐条读这 859 处发现绝大多数确实是「历史陈述用词但语境上无法改写成现行值」
（因为它们本来就是在描述某一天发生了什么事，天然是历史句式），那就说明这张牌放的位置虽然宽，
但宽得有道理——这需要人工抽样读一批这 859 条，本报告没有做（见「没做什么」）。

## 判定一览

| 格 / 问题 | 判定 | 一句话 |
|---|---|---|
| 1. shape 与 history-ordinal 有没有缝隙 | 一致 | 规则与门禁字面都只把「`### ` 日期重复」划给 shape，撞号判定划给 `history-ordinal.sh`；用背景材料举的具体输入实测：shape 判绿、`history-ordinal.sh` 判红（退出码 1），分工严丝合缝，无缝隙 |
| 2. `--mention-scope heading-and-quick` 会不会漏决策 | 冲突（帮助文本 vs 代码）+ 复核不了（实际用了哪个 scope） | 默认值是 `all`（会扫正文），帮助文本说「认不出一个索引表里有的号时退回 all」，但实际代码是「标题+快查里只要命中一个已知号就不回退」，直接调用 `mentioned()` 函数实测确认存在丢失（D9 被丢）；这一次迁移实际传没传这个参数找不到调用记录，复核不了 |
| 3. status-sync 会不会说谎 | 一致 | 门禁脚本注释「判不了：索引表那一行本身写得对不对」与规则「它管不了的」一节字面对应；机制上 `judge()` 确实不回读决策正文，只比两处字符串——这是设计射程，不是脚本漏做 |
| 4. 迁移脚本并发安全的窗口有多宽 | 一致 | 文件头声明的并发模型与实际实现一致：窗口收窄到「复核 sha256 返回」到「`os.replace()`/`os.remove()`」之间的几行 Python 字节码，无文件锁；窗口确实存在但量级是微秒级 |
| 5. 自引用锚点这一轮改没改全 | 冲突（规模远超背景材料点名的数） | 313 处「日期（其N）」形态（decisions-history.md 193 + experiments-history.md 120），远多于背景材料点名已核实的 2 处；脚本自检里零逻辑检查这类锚点是否因重编号失效，与背景材料的判断一致，但具体遗漏条数没有被逐条分类核实 |
| 6. `history-registry` 牌是不是开得太宽 | 一致（规模已实测量化） | 脚本注释与背景材料描述字面一致：豁免的是两份文件从第一个 `## ` 起的全部正文；实测去掉牌之后单这两个文件新冒出 859 处（756+103）D/A 类违规，量级确认「大范围」不是夸张说法；该不该收窄是取舍题，未判定 |

## 没做什么

- 问题 1 提到的另一个候选缝隙（两个 `#### ` 子标题点名词相同、但都不写编号）只指出、没有单独构造样本验证。
- 问题 2 没能找到这一次迁移实际调用 `migrate-changelog-format.py` 时传的参数（工作区未提交，没有单独的调用日志），
  无法坐实这次真实迁移是否踩中了 `heading-and-quick` 的这个缺口；也没有扫描 `decisions-history.md`/
  `experiments-history.md` 全部条目找「正文提到但没进对应节」的具体反例（工作量超出这一轮）。
- 问题 4 没有写一个真正的多进程并发测试（两个进程同时改同一份文件、在复核窗口内精确注入另一次写入）；
  `lib_atomic_replace.py` 自带的自证只测读者-写者场景，不测写者-写者场景，这类测试目前不存在。
- 问题 5 没有把 313 处「日期（其N）」逐条分类成「围栏内历史快照（天然豁免、不用管）」与
  「围栏外活引用（可能因重编号失效）」，只抽样读了两处佐证两种形态都存在；没有写脚本把「围栏外的活引用」
  与「目标节里是否存在同编号 `#### `」做交叉核对，报出真正会指空的条数。
- 问题 6 没有对 859 处新增违规逐条人工抽样判断是不是本来就该写成历史句式、改不动；也没有单独拆出
  `experiments-history.md` 内时间指代与非时间指代三类的分别条数，只按类别名在两个文件合计里统计到具体条数。
- 不判别的腿（本地攻方）的格，本报告不代为判定；不替主 agent 采纳或出判决，六问的取舍与是否收严均留给主 agent。
- `q1-test`、`q6-test` 两份构造样本与 `q6-nogate.log` 留在 `/tmp/claude-1000/changelog-format-r1-forward/`
  下，未清理——这两份是本报告的直接证据来源，主 agent 需要复核时可以直接进这个目录重跑；不是编译目录或仓副本，
  是几个 KB 大小的最小构造仓与一份日志，未按「交回前删编译目录与仓副本」清理，因为它们本身就是证据，
  删了报告里的命令就没法重放。
