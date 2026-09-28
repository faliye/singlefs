# changelog-format-r1 云端攻方腿报告（2026-09-28）

攻击面：背景材料「## 要腿答的」第 2、3、4、6 条（本报告编号 ①②③④）。被判对象是开工时的工作区（HEAD e5253e8a 加暂存区里这一轮的改动），开工时拷进草稿目录的副本上跑；副本已删，复跑按下面的命令重建。
副本上量出的数不是入库装置上的数。凡标「推的」的只按代码推、没实现没跑；凡是我提的改法都「只在我的模型上量过或没量过、被攻过零轮」。

## 复跑命令与文件 sha256

模型与脚本都在 `/tmp/claude-1000/changelog-format-r1-cloud-attack/model/`（下称 `$M`），草稿目录下称 `$S`。

```
# 迁移前的树（①③ 用）：仓文件 + e5253e8a 的 .claude/kb + 开工时的两份索引与迁移脚本
bash $M/rebuild-premigration-tree.sh /home/fy5090/code/singlefs $S/mig
cd $S/mig
python3 $M/scope_diff.py $S/mig $S/scope-strong-examples.tsv      # ① 两种 scope 的差、按字段分
python3 $M/scope_worst.py $S/mig > $S/scope-worst.tsv              # ① 最尖的子集
python3 $M/scope_filetouch.py $S/mig > $S/scope-filetouch.tsv      # ① 有证据改了 Dx 自己正文的
python3 $M/scope_alternatives.py $S/mig                            # ① 几种 scope 的份数与字节
python3 $M/window_timing.py $S/mig $S/wt 5                         # ③ 各份输出的无保护窗口
python3 $M/rerun_after_skip.py $S/mig $S/wt                        # ③ 退 3 之后照出路重跑
python3 $M/race_sweep.py $S/mig $S/wt                              # ③ 4 个输入 × 22 份输出扫一遍
# ②④ 用迁移后的工作区副本
rsync -a --exclude target --exclude .git /home/fy5090/code/singlefs/ $S/copy/
bash $M/status_sync_probe.sh $S/copy $S/ssp                        # ② 正文推翻、索引不动
bash $M/doclint_probes.sh $S/copy $S/probes                        # ④ 三个 doc-lint 变体（约 2 分钟，三件并行）
python3 $M/d3_enclosing.py $S/probes/p-1.log $S/probes/v-nocard    # ④ 摘牌之后的违规分类
```

| 文件 | sha256 |
|---|---|
| rebuild-premigration-tree.sh | ae198eb0d1c32e327194644b73ff24a5c8e8f52c9a1fa315548dc2330473db05 |
| scope_diff.py | 6359be556e0842c116f5d30019d1d4db912abe9d270bb26f36bb95f417db52ab |
| scope_worst.py | 879721e14ca3d870848417966bcba5bb82595317eec4e7e569ef77b0ad1b2de3 |
| scope_filetouch.py | 68bb773b233c10cf4a60b054bf48c5094a8ff592622bcd8caeb603e99da46e68 |
| scope_alternatives.py | 0fad88e3bfdf30d722c5b682cf2131c6b0d2ac0daf288ebc2dc94beb499478ad |
| window_timing.py | 33b9e8bc01fa9ccb60b18d80b0a54d9c2baf622ae74e2584af75b15986cc677a |
| rerun_after_skip.py | 72a2b4e86fafdb1cee72f414bf1cebc48363387706691802022014ae1ee14571 |
| race_sweep.py | 9490b353796a25310eecaefdc3a7c29fef288e9bde3c605e310d4aa2300c192b |
| status_sync_probe.sh | 61cb56ccb615ab3886e2c6c825add13844d546cf013f174d14430ad24c771bc9 |
| doclint_probes.sh | 2f8b4388d1d4b4dd8eb2670876f224e20fe1990123661f5513e5cd6cc4be8898 |
| d3_enclosing.py | 26baa1950b1f1ebef0fd693374d1b211e5659c13889875f4b2437a9c5c0d4ad9 |
| inputs/decisions.md（开工时工作区的索引） | 0bde34321ff744c03e0e925538b1dff2f5d6174d183154f42106190e19b0f97e |
| inputs/experiments.md（同上） | eae55763461347425fe0d1be15d948c9eeb42f6954d542ce9a71a63fd18a12bb |
| inputs/migrate-changelog-format.py（与仓里那份逐字节相同） | 60c34afbdab159cbca712436117c1dfe1d05ad24b66b711a57283e440984b885 |

原样输出留在 `$S`：`mig-all.log`、`mig-heading-and-quick.log`、`scope-worst.tsv`、`scope-filetouch.tsv`、`race-sweep.txt`、`status-sync-probe.txt`、`b1-1.log`～`b1-3.log`、`d22-card.log`、`d22-nocard.log`、`probes-run.txt`。

⚠️ 迁移前的树只能从 e5253e8a 取，那里两份按月文件共 582 条决策条目、实验变更史 259 条；这一轮真跑的是 591 / 263 条（多出的 9 + 4 条是当时工作区里还没提交的条目，已随按月文件一起删掉，取不回）。①③ 的数都是 582 / 259 条上的数。

## 各格判定一览

| 格 | 判定 | 一句话 | 依据在 |
|---|---|---|---|
| ①-1 heading-and-quick 漏掉改了某条决策分项的条目 | **打中** | 582 条里 307 条至少丢一个正文点名的决策，共丢 1031 份（all 2203 份、heading-and-quick 1172 份）；其中 259 份那条决策的节里当天一条都没有；实例：2026-09-14「18 问同日用户定案」改了 13 条决策的分项，只进了 D3（空间分配） 一节 | ① 节 |
| ①-2 这一轮迁移用的就是 heading-and-quick | 坐实 | 两种 scope 干跑的输出 6866612 字节与 31033627 字节，仓里现在那份 6896014 字节 | ① 节 |
| ①-3 补全的代价 | 量过（我的模型） | all 约 31.0 MB、只补「改前/改后里带分项号」约 22.7 MB，heading-and-quick 6.9 MB | ① 节 |
| ②-1 索引滞后时现状行照样绿 | **打中，但归到 status-sync 名下是归错** | 正文推翻 D1（数据可移动性 / 反向索引） 已定项 7、索引不动：20 号八格与 30 号 status-sync 在改前改后判得一模一样，现状行仍写「要反向索引」；status-sync 的输入里没有正文，缺口在「索引结论列 ↔ 正文」这一段 | ② 节 |
| ②-2 这是不是这一轮新开的 | 决策一半不是，实验一半是 | 迁移前 decisions-history.md 就有 28 行同形的现状行；experiments-history.md 迁移前 0 行，现在 162 行 | ② 节 |
| ③-1 迁移脚本自己的无保护窗口 | 量过：很窄 | decisions-history.md 1.8–2.1 ms，按月文件删之前 24–218 µs，别的 21 份 ≤ 141 µs（5 轮） | ③ 节 |
| ③-2 读写之间被改过时的出路 | **打中** | 4 个输入 × 22 份输出扫 88 格：40 格退 3，40 格照出路重跑全被拒（退 1），树停在一半搬过的状态 | ③ 节 |
| ③-3 要不要上原子替换 + 文件锁 | 不值得（推的） | 已经是原子替换；锁是劝告式的，别的写者（Edit、replace-once.py、30 号 --write）都不拿锁 | ③ 节 |
| ③-4（相邻，不在分工里）30 号 --write 就地整份重写、不复核 | 推的 + 量了上界 | 整条命令 0.09 s；lib_atomic_replace.py 文件头点名的就是这种写法 | ③ 节 |
| ④-1 牌对任何 kb 文件都生效 | **打中** | 贴在一份决策正文第 2 行，整份决策正文的 A / I / D 类全部免检，20 号 kb-shape 照样绿 | ④ 节 |
| ④-2 对两份变更史是不是放宽 | 没打中（与迁移前同宽） | 迁移前按月文件第 6 行、experiments-history.md 第 17 行就是「## 历史版本」，这些条目本来就不扫 | ④ 节 |
| ④-3 「只放开 D-3」这个备选 | 前提不成立 | 摘牌后 D-3 的 531 处（125 行不同文字）全部挂在带日期的 `###` 下、最近标题（`####`）一处都不带日期：D-3 是两层标题造出来的，改 D-3 找日期的层级就不用豁免它 | ④ 节 |
| ④-4 牌豁免的类别 | 背景材料少写了一类 | doc-lint 文件头写的是「D、I 与 A」，文风 I 也一起豁免 | ④ 节 |

## ① `--mention-scope heading-and-quick` 漏掉真正相关的决策

**机制**。`research/scripts/migrate-changelog-format.py:315`：「text = '\n'.join([entry.raw_heading] + [line for line in entry.body if QUICK_LINE.match(line)])」——只拿标题与以「> 快查·」开头的行找 `D<n>（`；找到任何一个索引表里有的号就不再看正文，只有一个都找不到才退回整段（`research/scripts/migrate-changelog-format.py:47`：「认不出一个索引表里有的号时退回 all」）。
规则那一侧：`.claude/rules/changelog-format.md:27`：「节内是这条决策的全部历史」；`.claude/rules/changelog-format.md:31`：「一条改动点名了几条决策，就在几节里各写一份、内容相同」；「点名」没有定义，`.claude/rules/changelog-format.md:59` 把它明说交给人：「一条改动该点名的决策或实验有没有点全」。

**这一轮真跑用的是哪个 scope**（`$S/mig-*.log` 原样行）：

```
all:               · 决策变更史：2 份按月文件 582 条原条目 → 28 节有条目（索引表 28 行，都建节），抄了 2203 份；没点名的 1 条；点名了索引表里没有的号 0 个；输出 31033627 字节
heading-and-quick: · 决策变更史：2 份按月文件 582 条原条目 → 28 节有条目（索引表 28 行，都建节），抄了 1172 份；没点名的 1 条；点名了索引表里没有的号 0 个；输出 6866612 字节
```

仓里现在的 decisions-history.md 是 6896014 字节（多出的是 9 条取不回的条目与之后别的会话追加的），所以是 heading-and-quick。decisions-history.md 自己「## 历史版本」里那条迁移记录只写「原样搬进各自点名的 `## D<n>（` 节」，没写「点名」只认标题与快查。

**丢了多少**（`scope_diff.py` 原样首行）：

```
== D: entries 582; entries with >=1 dropped number 307; dropped pairs 1031; dropped pairs whose date is absent from that section under heading-and-quick 259; dropped pairs where 改前/改后 names "Dn（…） 已定项/未定项 k" 334
== E: entries 259; entries with >=1 dropped number 79; dropped pairs 147; dropped pairs whose date is absent from that section under heading-and-quick 91; dropped pairs where 改前/改后 names "En（…） 已定项/未定项 k" 0
```

被丢的点名落在哪个字段（一份可以落在几个字段，只列前五）：改前 399、改后 248、依据 116、快查两行之后第一个 `- **字段**` 之前的正文 112、判决 20。丢的大头在改前/改后，不在依据。

**最尖的子集**：改前/改后里写着「Dx（…） 已定项/未定项 k」、而 Dx 的节当天一条都没有，59 份（`scope_worst.py`）；其中 26 份出自「按瘦身形态重写」那类条目（改前段整段抄了旧正文，旧正文顺带引了别的决策，这 26 份算弱），另 33 份不是。

**实例一（在仓里现在的 decisions-history.md 上现查过）**。e5253e8a 的 `decisions-history/2026-09.md` 第 9327 行起那条「2026-09-14（其三）：总审核第二轮的 18 问同日用户定案——映射条目回一宽 55、码 2 头预留位 29 与 key 宽挪到偏移 51、指针 88 / 86 留压……」：标题与快查两行只点名 D3（空间分配），正文十行里带分项号点名了 D2、D3、D4、D5、D6、D8、D12、D13、D18、D19、D21、D22、D23 十三条。现查结果（原样）：

```
sections holding the 2026-09-14 18-question entry: [3]
D4: has ### 2026-09-14 block: False
D5: has ### 2026-09-14 block: False
D8: has ### 2026-09-14 block: False
D12: has ### 2026-09-14 block: False
D13: has ### 2026-09-14 block: False
D19: has ### 2026-09-14 block: False
D21: has ### 2026-09-14 block: False
entry lines: 10 ; D numbers named with 已定项/未定项 k: [2, 3, 4, 5, 6, 8, 12, 13, 18, 19, 21, 22, 23]
```

D19（块指针的结构与宽度预算） 那一节里按「88 / 86|88/86|85 / 83 → 88|指针 88」搜命中 5 行，全在 2026-09-20 那条瘦身条目抄的旧正文里；「指针宽从 85 / 83 改到 88 / 86 是哪天、为什么」这件事在 D19 的节里没有条目。

**实例二**。同一天的「2026-09-14（其二）：总审核第二轮的机械回扫」：快查只点名 D2、D3、D18，改后段写着「D8（核心索引结构） 已定项 10 的 `seq` 改成「一次发布里被写进树的第几次」」，另改了 D11 已定项 1 / 2 / 6、D13 已定项 4 的状态数、D15 冻结前清单、D17 条款 3。现查（原样）：

```
D2: section lines 528-2380; has ### 2026-09-14: True; lines containing the D8 item-10 seq edit text: 1
D3: section lines 2381-4107; has ### 2026-09-14: True; lines containing the D8 item-10 seq edit text: 1
D8: section lines 9737-12282; has ### 2026-09-14: False; lines containing the D8 item-10 seq edit text: 0
D11: section lines 16477-18020; has ### 2026-09-14: False; lines containing the D8 item-10 seq edit text: 0
D13: section lines 19229-20759; has ### 2026-09-14: False; lines containing the D8 item-10 seq edit text: 0
D15: section lines 22210-23165; has ### 2026-09-14: False; lines containing the D8 item-10 seq edit text: 0
D17: section lines 24606-25244; has ### 2026-09-14: False; lines containing the D8 item-10 seq edit text: 0
D18: section lines 25245-28906; has ### 2026-09-14: True; lines containing the D8 item-10 seq edit text: 1
```

（行号是现查那一刻的；decisions-history.md 之后还在被别的会话追加，交回前又复核了一次，见「这条腿自己的限度」。）

**读者会不会因此错过关键改动**：会，在按节读的时候。`seq` 的取值规则是盘上字段的定义；D8 的节里找不到它 2026-09-14 那次改写。按节读是这套形态的主用法：三方材料按 `.claude/rules/three-way-inference.md` 用 `quote-kb.py` 的 `@标题` 按节抽，抽 `## D8（核心索引结构）` 这一节就抽不到它。全文 grep「D8（」还找得到（它住在 D2、D3、D18 三节里），所以丢的是「按节读」这条路，不是内容本身。

**打中之后的四句**

1. 分不分辨臂：分辨。臂是 scope 的取法；all 按定义把正文点名全收，这两条实例在 all 下都进了对应的节，只有 heading-and-quick 丢。
2. 被判的系统当时看不看得到：看得到。丢掉的点名就在脚本手里那条 `Entry.body` 里，脚本自己的 all 分支就读它。
3. 满足判据字面的哪一句：`.claude/rules/changelog-format.md:27` 的「节内是这条决策的全部历史」。不是「点名」那一句——「点名」没定义，拿它判算不上违规；所以这一格的病根一半在规则：要么定义「点名」，要么收窄「全部历史」的承诺。
4. 各改法在打中的格上还中不中（「中」= 那一份还丢）：

| 改法 | 实例一（18 问，丢 D4 D5 D8 D12 D13 D19 D21 等） | 实例二（丢 D8 D11 D13 D15 D17） | 份数 / 约字节 | 标注 |
|---|---|---|---|---|
| 维持 heading-and-quick | 中 | 中 | 1172 / 6860477 | 量过 |
| all（脚本默认） | 不中 | 不中 | 2203 / 31049733 | 量过（我的模型；与真干跑的输出 31033627 字节差 0.05%） |
| heading-and-quick 加「改前/改后里带分项号点名的」 | 不中（集合 `[2, 3, 4, 5, 6, 8, 12, 13, 18, 19, 21, 22, 23]`） | **D15、D17 仍中**（它们写成「冻结前清单第 1 项」「条款 3」，不带分项号） | 1506 / 22676848 | 量过（我的模型） |
| heading-and-quick 加「正文任何处带分项号点名的」 | 不中 | D15、D17 仍中（同上） | 1790 / 23876521 | 量过（我的模型） |
| 丢掉的那几节各写一行指路（「见 D3（空间分配） 节 2026-09-14 那条」） | 不中 | 不中 | 约 1031 行 | 推的；指路要能指到一条，而新规则的「（其N）」按节内同一天重编，同一条在不同节里号可能不同，指路只能写日期加标题 |
| 维持现状，改规则：节只收标题与快查点名的 | 中（承诺变了，不算修） | 中 | — | 推的 |

大小是这一格真正的取舍：补全到 all 要 4.5 倍（31 MB，173629 行），补「分项级」要 3.3 倍，而且在实例二上仍漏两节。只加指路行最便宜，但没量过、指路的锚要另定。

**什么现象会推翻 ①-1**：仓里某处（规则、kb-scribe 定义、三方材料的取法）写明 decisions-history.md 的节只收标题与快查点名的、按节读的人被告知要另外全文 grep；或者在一份不来自 e5253e8a 的完整 591 条输入上重跑，丢的份数降到零。

## ② `status-sync` 的现状行会不会在索引表滞后时说谎

**机制**。30 号 status-sync 只比两样文本：`.claude/gate.d/30-decision-history-entries.sh:59`：「判四样：节顶上没有这一行；这一行与索引表对不上；节在索引表里没有那一行；索引表里的一行在变更史里没有节。」它的输入里没有决策正文。
索引表两格各由谁盯：「状态」格由 20 号格 decision-items-sync 与 kb-shape 第 7 段对着正文判（分项计数）；「结论（简报）」格对决策只有宽度一道，`.claude/gate.d/20-doc-decision-documents.sh:29`：「决策索引表每一行的「结论」列不超过 decisions.md 那一句写的字数上限」。实验一侧由 34 号 index-sync 部分盯住（作废字样、未跑 / 部分已跑 / 够判、结论列里的数），它自己写明拦不住以历史叙述形态留在正文里的旧值，`.claude/gate.d/34-doc-experiment-pages-and-products.sh:182`：「这一格拦不住它们** —— 拦得住的是第 1 项那一类：正文已经宣告作废而索引只字未提。」

**造的历史**（`status_sync_probe.sh`，副本上跑）：一个会话把 D1（数据可移动性 / 反向索引） 正文分项表第 7 行从「要，且 day-1 就设计进格式，不许后补」改成「不要；改成后台扫描按需重建，不进格式」，已定条数不变，索引表与变更史现状行都不碰。每一步许可它的那一句：改决策正文照 `.claude/kb/decisions.md` 首段「推翻某条决策时，直接改正文」；30 号 entry-added 只要求变更史有新增条目，改索引结论列那句 howto 只在 entry-added 判红时打出（`.claude/gate.d/30-decision-history-entries.sh:187`）。原样输出：

```
body row 7 now:
17:| 7 | **要不要反向索引，什么时候设计进格式** | 不要；改成后台�
index row:
32:| D1（数据可移动性 / 反向索引） | 已定 8 项 / 未定 0 项 | 要反向�
D1 现状 line:
11:**现状**：已定 8 项 / 未定 0 项。要反向索引，day-1 设计进格式，�
== /tmp/claude-1000/changelog-format-r1-cloud-attack/copy
  gate20 kb-shape rc=0
  gate20 item-ref-status rc=1
  gate20 status-redundancy rc=0
  gate20 cross-decision-status rc=0
  gate20 settled-item-self-open rc=0
  gate20 settled-ref-says-open rc=0
  gate20 decision-items-sync rc=0
  gate20 decision-summary-width rc=0
  gate30 status-sync rc=0
== /tmp/claude-1000/changelog-format-r1-cloud-attack/ssp
  gate20 kb-shape rc=0
  gate20 item-ref-status rc=1
  gate20 status-redundancy rc=0
  gate20 cross-decision-status rc=0
  gate20 settled-item-self-open rc=0
  gate20 settled-ref-says-open rc=0
  gate20 decision-items-sync rc=0
  gate20 decision-summary-width rc=0
  gate30 status-sync rc=0
```

item-ref-status 的红在没改过的副本上一样红（「✗ 分项引用与正文状态不一致 133 处」，改前改后两份输出去掉根路径之后逐字相同），是工作区里原有的，不是这条历史造出来的。结果：推翻之后，D1 节顶上的「**现状**」仍写「要反向索引」，没有一道格变。
没跑的格：20 号的 stale-open-items、settled-same-file、freeze-layer-membership、blocking-verdict、user-verdict-owed 与 30 号 entry-added、shape 要 git 或判别的是别的东西，副本不是 git 仓，没跑；它们判的对象里都没有「结论列 ↔ 正文」。

**真实先例**：`.claude/kb/experiments-history.md:4359` 记着 E79（根记录的容量） 的索引行停在「恰爆 1 字节」而正文已是「爆 43 字节」；34 号自己的文件头记着 E69（反向索引取权威态的增量维护代价） 的索引行带着作废结论停了五天。决策一侧没找到现成的先例（没系统搜，只 grep 了「索引…没跟 / 滞后 / 停在旧 / 漏改」几个说法）。

**打中之后的四句**

1. 分不分辨臂：臂是「status-sync 照现在的判据」与「status-sync 再多判点什么」。status-sync 不管怎么改，只要输入仍是索引与现状行两份文本，这条历史都判绿；能分辨的只有把正文拉进来的判据。
2. 被判的系统当时看不看得到：**status-sync 看不到**——判别它的是决策正文，不在它的输入里。照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」那张表，这是「判别子观测不到」，拿它判 status-sync 不成立。
3. 满足字面的哪一句：满足的是「索引结论列与正文一致」这条没有人立过的判据；status-sync 的字面（与索引表一致）它没违反。所以背景材料问的「算不算这条检查本身的设计缺口」，我的答案是**归错了**：缺口在「索引结论列 ↔ 正文」这一段，决策一侧只有宽度检查，实验一侧由 34 号部分盖住。
4. 这一轮新增的是什么：决策一侧不是新的——e5253e8a 的 decisions-history.md 就有 28 行同形的「**现状**：」（第 20 行就是 D1 那一行，由已撤的 brief-sync 生成）；实验一侧是新的——e5253e8a 的 experiments-history.md 里 `^\*\*现状\*\*：` 0 行，现在 162 行。新增的是「把索引结论列再抄一份，贴在历史条目正上方、标成现状」这一处，它不新增错误，只多一处照抄的错误。

**改法各修哪一格**

| 改法 | 上面那条历史还中不中 | 标注 |
|---|---|---|
| 维持现状，记一笔欠账（「索引结论列 ↔ 正文」没有会红的检查） | 中 | 推的 |
| 现状行只留「状态」格、不抄「结论（简报）」 | 现状行不再说谎（没有那半句了）；索引表本身仍错 | 推的 |
| entry-added 加一条：决策正文的定案行变了，要么索引那一行也变、要么变更史新条目里写明「结论（简报）不变」 | 不中（这条历史会判红） | 推的；「定案行」怎么认要另定，没实现 |
| 34 号 index-sync 的做法搬到决策：结论列里的数与关键词要在正文里找得到 | 这一条仍中（「要反向索引」在改后的正文别处仍出现） | 推的 |

我的判断：不在这一轮补，记欠账；理由是它在这一轮之前就存在（决策一侧），status-sync 的合同没有要求它、也做不到。
**什么现象会推翻 ②-1**：有一道已有的检查（我没看到的格或 hook）在上面那条历史上判红；或者规则里写明现状行只是索引的投影、不作「现状」读。

## ③ 迁移脚本读输入到写文件之间的并发窗口

**机制**。每份输出走 `research/scripts/migrate-changelog-format.py:779`：「replace_file_contents_by_rename(os.path.join(root, relative), text, check_before_rename=recheck)」——临时文件写完、fsync 之后，逐个重算它依赖的全部输入的 sha256，全对得上才 `os.replace`。决策变更史依赖四份（decisions-history.md 自己、decisions.md、两份按月文件），按这个次序算。删按月文件之前再算一次：`research/scripts/migrate-changelog-format.py:794`：「if BREAK != 'no-recheck' and sha256_of(os.path.join(root, relative)) != digests[relative]:」。
所以没保护的只有「某份输入被复核读完」到「那份输出换上 / 那份按月文件删掉」之间。

**量出来的宽度**（`window_timing.py`，5 轮，`$S/window-timing.txt` 原样；这 5 轮跑时机器上另有三件 doc-lint 与别的会话的 cargo 在跑）：

```
round 0: exit 0; guarded span first-read -> last-write 298.4 ms; decisions-history.md unguarded window 2202 us (over 4 dependency hashes, narrowest 76 us); month-file delete windows 207 us, 50 us; other outputs max window 793 us over 21 outputs
round 1: exit 0; guarded span first-read -> last-write 305.6 ms; decisions-history.md unguarded window 2508 us (over 4 dependency hashes, narrowest 76 us); month-file delete windows 204 us, 48 us; other outputs max window 151 us over 21 outputs
round 2: exit 0; guarded span first-read -> last-write 300.3 ms; decisions-history.md unguarded window 2467 us (over 4 dependency hashes, narrowest 78 us); month-file delete windows 169 us, 34 us; other outputs max window 154 us over 21 outputs
round 3: exit 0; guarded span first-read -> last-write 287.4 ms; decisions-history.md unguarded window 2237 us (over 4 dependency hashes, narrowest 69 us); month-file delete windows 209 us, 32 us; other outputs max window 164 us over 21 outputs
round 4: exit 0; guarded span first-read -> last-write 299.8 ms; decisions-history.md unguarded window 2271 us (over 4 dependency hashes, narrowest 106 us); month-file delete windows 220 us, 40 us; other outputs max window 149 us over 21 outputs
```

（更早一次负载轻时的 5 轮只打在终端、没落盘：decisions-history.md 1.8–2.1 ms，删文件 24–218 µs，整段 179–184 ms；不当证据，只当量级参考。）
「decisions-history.md 2.2–2.5 ms」是它自己被复核那一刻到换上的间隔，中间要再算三份输入的 sha256（两份按月文件合计 2925436 字节）；最常被别人写的 2026-09.md 排第三，它的无保护段只剩最后一份的哈希加改名，几百微秒。

**扫一遍「别的会话在哪一刻写」**（照「由用户决定的动作不写死」放开扫）：写的对象取 4 份输入（2026-09.md、decisions.md、experiments.md、experiments/142-新池新建文件的干跑.md），写的时刻取 22 份输出各自换上之前那一刻，88 格（`race_sweep.py`，`$S/race-sweep.txt` 原样）：

```
cells: 88 = 4 trigger files x 22 outputs
outcome (first-run exit, probe line survives in tree, rerun exit if first exit was 3): count
  (0, True, None): 48
  (3, True, 1): 40
cells where the probe line is lost: 0
```

88 格里探针那一行一次都没丢——复核在这些时刻都起作用。但退 3 的 40 格，**照退 3 的出路重跑全部被拒**。那条出路是 `research/scripts/migrate-changelog-format.py:868`：「等那个会话写完、提交之后，从最新的文件重跑这个脚本（它从头读、从头判）」。两个典型现场（`rerun_after_skip.py`，`$S/rerun-after-skip.txt` 原样）：

```
[A-before-DH-replace] first run exit 3; decisions-history.md migrated=False; experiments-history.md migrated=True; month files=['2026-08.md', '2026-09.md']; probe entry in decisions-history.md=False
[A-before-DH-replace] rerun as the exit-3 howto says: exit 1
    ✗ .claude/kb/experiments-history.md 里已经有 `## E<n>（` 节，不是搬迁前的形状
         → 怎么办：看它是不是已经被别的会话搬过；这个工具只从搬迁前的形状起步
[B-after-DH-replace] first run exit 3; decisions-history.md migrated=True; experiments-history.md migrated=True; month files=['2026-09.md']; probe entry in decisions-history.md=False
[B-after-DH-replace] rerun as the exit-3 howto says: exit 1
    ✗ .claude/kb/decisions-history.md 里找不到 <!-- gen:history-brief:start --> / <!-- gen:history-brief:end -->，不是搬迁前的形状
         → 怎么办：看它是不是已经被别的会话改过；这个工具只从搬迁前的形状起步
```

原因：跳过只跳被改过的那一份、别的照写，是设计好的（自证里写明了，`research/scripts/migrate-changelog-format.py:1262`：「'一份被跳过时别的输出也没写', '跳过只跳那一份，别的照写')」），而脚本开头只认「一份都没搬过」的形状。于是退 3 之后仓停在一半搬过的状态：A 现场实验变更史搬了、决策变更史没搬；B 现场决策变更史搬了而少了那条新条目、2026-08.md 删了、2026-09.md 留着且多一条。拒绝信息还把原因说成「别的会话搬过」，其实是它自己搬的。要恢复只能手工把搬过的几份退回去（在工作区里就是撤销未提交的改动，command-safety 那一类退不回去的操作）或者手工并条目。

**打中之后的四句**（③-2）

1. 分不分辨臂：臂是「窗口收不收紧」。这一格在收紧与不收紧两边都一样中（它发生在受保护的那一支上），**不分辨收紧与否**，分辨的是「退 3 之后怎么收场」。
2. 看不看得到：看得到——重跑时脚本读得到哪几份已经是新形态。
3. 满足字面的哪一句：满足的是它自己那句出路（`:868`）不成立，不是「并发没保护」。
4. 背景材料给的改法「原子替换 + 文件锁」在这 40 格上仍中：已经是原子替换；锁只让这个脚本自己串行，退 3 之后的半搬状态照旧。

**值不值得收紧窗口：不值得**（推的，理由如下，没实现锁去量）。
- 这个脚本是一次性的，已经在真仓上跑过；它对搬过的树拒绝再跑（上面两段拒绝信息），今后只在一棵没搬过的树上才会再跑。
- 文件锁是劝告式的，只拦同样拿锁的进程。仓里改这几份文件的别的写者都不拿锁：Edit 工具；`research/scripts/replace-once.py:47`：「notes = replace_file_contents_by_rename(path, replaced)」（读完到改名之间不复核）；30 号 status-sync 的 --write。
- 真值得改的是退 3 的收场（推的，都没实现）：要么在写第一份之前把全部输出的全部依赖复核一遍，有一份变了就一份都不写（写阶段本身仍有几百毫秒，只是缩小，不是消除）；要么让脚本能从半搬的形状续搬（按份判「已是新形态就跳过」）；最少把 `:868` 那句出路改成照实说要先退回已写的几份。

**相邻的一格（不在分工里，只记一句）**：30 号 status-sync 的 --write 是就地整份重写，`.claude/gate.d/30-decision-history-entries.sh:479`：「with open(history_path, 'w', encoding='utf-8') as handle:」，读到写之间不复核。这正是 `research/scripts/lib_atomic_replace.py:3` 点名不用的写法：「`open(路径, 'w')` 截断重写改的是同一个 inode」。它的窗口我只量了上界：在副本上改掉一行现状再跑 --write，整条命令 3 次都是 0.09 s（「按索引表重写了 1 行现状」）。这期间别的会话用改名换上的新条目，会被它按路径打开、截断、写回旧内容而丢掉——这一句是推的，没造竞争去量。它是常用工具（出路里叫人改完索引就跑），比一次性的迁移脚本更值得看。

## ④ `<!-- doc-lint:history-registry -->` 开得太宽没有

**机制**。牌的认法：`.claude/singlefs-ai-sop/scripts/doc-lint.sh:361`：「if head -5 "$f" | grep -qx '<!-- doc-lint:history-registry -->'; then」，放行的条件只有路径，`.claude/singlefs-ai-sop/scripts/doc-lint.sh:363`：「*/kb/*)」；牌的效果：`.claude/singlefs-ai-sop/scripts/doc-lint.sh:219`：「REGISTRY == 1 && /^## / { exit }」——正文扫描在第一个 `## ` 停下。豁免的类别写在 `.claude/singlefs-ai-sop/scripts/doc-lint.sh:40`：「D、I 与 A（词表比对、上下文/自指/时间指代、历史陈述）」，比背景材料列的四类多一类文风 I。

### ④-1 打中：牌对任何 kb 文件都生效

造的历史：一个会话在一份普通 kb 文件的前五行里贴上这张牌。每一步的许可：doc-lint 只判「在不在 kb 下」（上面 `:363` 那一行），它自己的拒绝只拦 kb 之外，`.claude/singlefs-ai-sop/scripts/doc-lint.sh:367`：「history-registry 标记只许用于 kb 里的文件」。
在两份文件上各注一行探针（「如上所述，本轮把本决策改成了新的；此前从未写过；这里进行分析。」，放在第一个 `## ` 之后），同一棵只含 kb 的树跑 doc-lint（`$S/b1-2.log` 不贴牌、`$S/b1-3.log` 贴牌，原样行）：

```
== b1-2（不贴牌）
  ✗ .claude/kb/decisions/22-单元原子性怎么合成.md:4  "此前从未写"是历史陈述，写进「历史版本」
  ✗ .claude/kb/decisions/22-单元原子性怎么合成.md:4  翻译腔：「进行/予以/加以 + 动词」→ 直接用那个动词
  ✗ .claude/kb/decisions/22-单元原子性怎么合成.md:4  kb 正文不许用上下文指代
  ✗ .claude/kb/decisions/22-单元原子性怎么合成.md:4  kb 正文里的「本轮」锚不到具体哪一轮
  ✗ .claude/kb/invariants.md:22  "此前从未写"是历史陈述，写进「历史版本」
  ✗ .claude/kb/invariants.md:22  翻译腔：「进行/予以/加以 + 动词」→ 直接用那个动词
  ✗ .claude/kb/invariants.md:22  kb 正文不许用上下文指代
  ✗ .claude/kb/invariants.md:22  kb 正文里的「本轮」锚不到具体哪一轮
== b1-3（贴牌）
  （这 8 行一行都没有）
```

两边都还有的「invariants.md 登记表写坏了」是我插探针的位置挤到了登记标记与表之间造出来的，贴不贴牌都在，与牌无关。
更狠的一种：牌贴在一份决策正文的第 2 行（标题 `## D22 …` 在第 1 行），`body_of` 在第 1 行就停，**整份决策正文**的 A、I、D 类全部不判（`$S/d22-card.log` 与 `$S/d22-nocard.log`：不贴牌时第 4 行那 4 条都在，贴牌后一条都没有，两边只剩同样的「不变量编号没有定义」——那是只拷了一份文件造出来的）。同一个改法下 20 号 `--check kb-shape` 退 0（「✓ kb 形状检查通过（kb 文件 210 份、规则 9 份）」），没有别的格拦它。

四句：分辨臂（「牌只认两份文件」这条臂不中，「牌认 kb 下任何文件」这条臂中）；被判的系统看得到判别它的东西（doc-lint 手里就有路径）；满足的是牌的用途说明以外的用法（用途写在 `.claude/singlefs-ai-sop/scripts/doc-lint.sh:39`：「decisions-history.md 这样一条决策一节、节内本身就是完整历史的登记表」），字面的「只许出现在 kb 里」它没违反；改法「按路径白名单只认 decisions-history.md 与 experiments-history.md」或「要求贴牌的文件里 `## ` 标题全是 `## D<n>（` / `## E<n>（` 加文末的历史节」都让这两处探针复红——推的，没实现。

### ④-2 没打中：对这两份变更史本身，牌没有比迁移前放宽

迁移前这些条目本来就不扫：e5253e8a 里两份按月文件的「## 历史版本」在第 6 行，experiments-history.md 的在第 17 行（`git show` 现查），条目全在它下面。所以背景材料说的「放开的是两份文件里全部的历史正文」对**这两份文件**是恢复原状，不是新放开。迁移前真被扫到、现在不扫的只有一块：旧 decisions-history.md 的生成块（每条决策最近 3 次的「改前 / 改后」一句话表，住在它自己的「## 历史版本」之前）；那块已经撤了。

### ④-3 摘牌之后会红什么；「只放开 D-3」这个备选的前提

摘掉两份文件的牌、同一棵树跑 doc-lint（`$S/b1-1.log`），按类别与去重之后的行文数（`d3_enclosing.py` 原样输出）：

```
file | category | hits | unique line texts | D-3: nearest heading has a date | D-3: enclosing ### has a date
decisions-history.md A 171 20 - -
decisions-history.md D-1 91 14 - -
decisions-history.md D-2 50 11 - -
decisions-history.md D-3 440 101 0 440
decisions-history.md I 4 3 - -
experiments-history.md A 2 1 - -
experiments-history.md D-1 1 1 - -
experiments-history.md D-2 9 5 - -
experiments-history.md D-3 91 24 0 91
```

（hits 比「不同行文」多，是因为同一条条目被抄进几节。）
D-3 的判据是 `.claude/singlefs-ai-sop/rules/kb-discipline.md:29`：「门禁的判据是**这一行或它所属的最近一个标题里有没有日期**，有就放行。」新形态把日期放在 `### YYYY-MM-DD`、条目标题 `#### ` 不带日期，于是 531 处 D-3 **全部**是「最近标题（`####`）不带日期、上一层 `###` 带日期」；在旧的融合写法 `### 日期（其N）：…` 下，同样的字最近标题就带日期、本来就放行。所以 D-3 的豁免需求是两层标题造出来的：把 D-3 找日期改成「最近一个 `###` 或它下面的 `####`」，这 531 处全放行（量过，我的模型按这条规则数的；doc-lint 本身没改、没跑）。这样「只放开 D-3」这个备选的前提就不成立了——D-3 根本不必豁免。
剩下的是 D-1 / D-2 与 A：A 是历史条目的本义（改前段就是「此前写的是 X」），判它等于不许写历史；D-1 / D-2 在两份文件里合计 31 行不同文字（14 + 11 + 1 + 5），其中有几行是在讨论规则本身时引了被禁的词（例如「「本文件」换成文件名，「本节 / 该节」换成那一节的标题」这一类），是误报。

**我对 ④ 的判断**：该收的是 ④-1（牌能贴到哪），不是 ④-2（两份变更史里豁免多宽）。「这两份里继续判 D-1 / D-2」是一条新的收严，代价约 31 行不同文字（每行要在它被抄进的每一节里各改一遍），不是回退到迁移前；要不要做与这一轮无关。
**什么现象会推翻 ④-1**：别的阶段或 hook 在牌出现在两份变更史之外时判红（我只跑了 doc-lint 与 20 号 kb-shape）。**推翻 ④-3**：D-3 的实现不是按「最近一个标题」判。我读了那段 awk：`.claude/singlefs-ai-sop/scripts/doc-lint.sh:580`：「line ~ /^#/ { head = line }」，任何级别的标题都会换掉 head，与规则原文一致。

## 没打中的形状

| 攻击面 | 试过的形状 | 取样范围 | 结果 |
|---|---|---|---|
| ① | 快查块跨行：以「>」开头、不以「> 快查·」开头的续行里点名决策，会不会被 heading-and-quick 漏掉 | 582 条决策条目 | 0 行（`scope_worst.py` 首行：「quick-block continuation lines that name a D number: 0 lines in 0 entries」） |
| ① | 实验一侧丢的是不是分项级改动 | 259 条实验条目 | 丢 147 份，其中改前/改后里「E<n>（…） 已定项 k」0 份：实验没有分项号，丢的多是改前、依据里顺带点名的别的实验；没再逐条判 |
| ② | 索引表同一个号出现两行、首现那行不是索引那一行 | decisions.md 28 行、experiments.md 162 行 | 两份都没有重号 |
| ② | 结论格里没转义的 `\|` 把格切错，现状行被截断而两边照样相等 | 同上 | 每行都恰好 4 格 |
| ② | 工作区现在就有索引与正文状态词对不上的实验 | 162 个实验 | 5 处字面不同，逐条看都是同义写法或我的取词器把 `<!-- -->` 当成了标题，没有真滞后 |
| ② | 工作区现在的现状行与索引表对不上 | 28 + 162 节 | 30 号 status-sync 在开工副本上「✓ 现状行与索引表一致：查了决策 28 节、实验 162 节」 |
| ③ | 探针那一行在某个时刻被悄悄丢掉 | 4 份输入 × 22 个时刻 = 88 格 | 0 格丢；「悄悄丢」只可能落在复核与改名之间那 ≤ 2.5 ms，钩子打不进那段，没造出来 |
| ④ | 牌贴在 kb 之外（agents/、rules/） | 没跑，读代码 | doc-lint 当场判红（`:367` 那一行），不算打中 |

## 这条腿自己的限度

- ①③ 用的迁移前的树来自 e5253e8a：少了真跑时工作区里的 9 条决策条目、4 条实验条目，数是 582 / 259 条上的数。两条实例在仓里现在的 decisions-history.md 上逐节现查过；decisions-history.md 在我干活期间还在被别的会话追加（迁移记录那一行从第 44236 行挪到了第 44246 行），交回前又现查了一次，结果见下一段。
- ① 的「强 / 弱」分类是正则分的（改前/改后里带分项号、正文里带「改成 / 补上 / 删掉…」），没有逐条人读；只有两条实例是逐条读过的。
- ② 只在决策一侧造了历史，实验一侧只引了 kb 里记下的先例（E79、E69），没造。
- ③ 的窗口是 Python 里打点量的，跑时机器上有别的负载；落盘的那一组比没落盘那一组宽约 20%。锁、全部先复核、可续搬三种改法都没实现。
- ④ 的 doc-lint 跑在只含 `.claude/kb` 的树上（为了快），编号类的判定与全仓跑不一样，我只看 A / I / D 类的行；这几类是逐文件判的，不受树的范围影响（推的）。
- 我没有读背景材料里「小节清单」那一大段（约 3800 行）与附录，只读了正文与「要腿答的」；引的行号都是在原文件里现查的。
- 在副本上跑过 20 号的八格与 30 号 status-sync / --write、doc-lint，都是为了造历史取判定，不是对仓的门禁判定。

交回前在仓里现在的 decisions-history.md（44250 行）上复核 ① 的两条实例（原样）：

```
18-question entry sections: [3]
D8 seq-edit sections: [2, 3, 18]
no ### 2026-09-14 in: [1, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17, 19, 21, 24, 25, 26, 27]
```

④ 的 doc-lint 三个变体用保存好的 `doclint_probes.sh` 又跑了一次（`$S/probes-run.txt`）：贴牌前探针相关违规 8 行、贴牌后 0 行；摘牌后按类别的计数与 `b1-1.log` 那一次完全相同（D-3 440 + 91，D-1 91 + 1，D-2 50 + 9，A 171 + 2，I 4）。第一次跑这个脚本时 `set -e` 把 doc-lint 的非零退出当成失败，退出码文件没写出来，那一批按作废处理（`$S/probes-run-void1.txt`），修了收退出码的写法再跑。

## 删掉的副本

交回前删了草稿目录里自己建的仓副本与派生树：`copy`（343M，工作区副本）、`mig`（335M，迁移前的树）、`g20`（6.0M）、`g20b`（5.9M）、`ssp`（5.9M）、`g30`（14M）、`mini-card`（14M）、`m-nocard`（14M）、`m-inject-nocard`（5.9M）、`m-inject-card`（5.9M）、`d22-card`、`d22-nocard`（各 100K）、`probes/v-*` 三棵（14M、5.9M、5.9M）、`wt`（9.2M）、`out-all`、`out-heading-and-quick`（两份干跑输出）。没有编译目录。留着的是报告、`model/` 与各份原样输出日志，复跑用上面的命令重建。
