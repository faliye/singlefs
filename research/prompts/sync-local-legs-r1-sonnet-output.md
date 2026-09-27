# sync-local-legs-r1 云端正推腿报告（丙、丁两格）

立场：正推。逐句核改后的两份定义（`.claude/agents/three-way-local-attack.md`、
`.claude/agents/three-way-local-defense.md`）与今天的脚本（`research/scripts/ask-local.sh`、
`research/scripts/oov-check.py`）、规则（`.claude/rules/three-way-inference.md`）行为是否一致。
判给我的格：丙（退出码与计数）、丁（生词表与小节名）。逐句一行：说了什么 / 对应代码或规则在哪一行 / 一致与否。

## 丙：退出码与计数

| 改动句（`.claude/agents/three-way-local-attack.md` 里今天的原文） | 对应代码 / 规则 | 一致与否 |
|---|---|---|
| 「取号：`ls <前缀>-output-s*.md` 看已用到几号，取下一个没用过的 `<n>`」（`.claude/agents/three-way-local-attack.md:29`） | 这一句管的是取号本身，不直接对应脚本的某一行；它防的是「重定向到一个已有样本的号会把它整份盖掉」，与 bash `>` 对已存在文件会截断重写这一语言事实相符 | 一致（事实性描述，非脚本某一行） |
| 「退出码 5：作废副本由脚本留成 `-output-void<n>.md`」（`.claude/agents/three-way-local-attack.md:30`） | `research/scripts/ask-local.sh:107`（`[[ "${ASK_LOCAL_ALLOW_CORRUPT:-0}" == "1" ]] || exit 5`）配 `research/scripts/ask-local.sh:105`（判红分支先调 `save_void`）；头注释「5 与 6 都不打正文、把正文留成 -output-void<n>.md」（`research/scripts/ask-local.sh:15`） | 一致 |
| 「判红那次重定向建出的 `s<n>` 是空文件，先确认它大小为 0，再用 `>|` 重定向到同一个号重跑」（`.claude/agents/three-way-local-attack.md:30`） | 脚本唯一一处把正文打到 stdout 的语句是 `research/scripts/ask-local.sh:136`（`cat "$TXT"; echo`），exit 5 在 `research/scripts/ask-local.sh:107` 已经 return，走不到这一行；调用方重定向出来的 `s<n>.md` 因此确实是 0 字节 | 一致 |
| 「`>|` 在开没开 `noclobber` 时都写得进」（`.claude/agents/three-way-local-attack.md:30`） | 不对应仓内某一行代码，是 bash 语言本身的语义（`>|` 忽略 `set -o noclobber`）；无论开没开 noclobber 都成立 | 一致（语言事实，非仓内一行） |
| 「退出码 0 的每次调用占一个号，带损坏的也占号、文件留着，运行记录里标带损坏」（`.claude/agents/three-way-local-attack.md:30`） | exit 0 只在两种情形出现：字词损坏闸判绿（未进任何 exit 分支，落到 `research/scripts/ask-local.sh:136` 之后自然退出）、或判红但设了 `ASK_LOCAL_ALLOW_CORRUPT=1`（`research/scripts/ask-local.sh:107` 的 `||` 短路，不 exit 5，继续跑到同一行打正文） | 一致 |
| 「退出码非 0 非 5（包括 6：损坏检测器没跑成或找不到，这一份没验过）、或网关不通：停下」（`.claude/agents/three-way-local-attack.md:30`） | 这句是排除式（「非 0 非 5」），字面上天然覆盖脚本除 0、5 之外的**全部**退出码：2（`research/scripts/ask-local.sh:26`、`research/scripts/ask-local.sh:30`）、3（`research/scripts/ask-local.sh:41`、`research/scripts/ask-local.sh:52`、`research/scripts/ask-local.sh:54`、`research/scripts/ask-local.sh:57`）、4（`research/scripts/ask-local.sh:63`，经 `research/scripts/ask-local.sh:67` 的 `[[ $rc -eq 0 ]] || exit $rc` 传播成脚本自身的退出码）、6（`research/scripts/ask-local.sh:130`），以及头注释里单独提到、不属于脚本自定义分支的 1（`research/scripts/ask-local.sh:16`：「内嵌的 python 自己出错时退 1（不在上面几种里，当没跑成）」） | 一致，但比派发提示三节问题的转述（「把 2、3、6 都归」）更宽：定义原文不是枚举 {2,3,6}，而是「非 0 非 5」的补集，4 与 1 同样落在这一支里，不存在遗漏 |
| 「网关不通」单独列在「或」之后（`.claude/agents/three-way-local-attack.md:30`） | 网关不通这一现象在脚本里本身就走 `research/scripts/ask-local.sh:41` 的 exit 3（`curl` 失败），已经在「非 0 非 5」里 | 一致但冗余：单独提「网关不通」不会引出脚本里第三种落点，它已经是 exit 3 的一个成因，不构成遗漏也不构成矛盾 |
| exit 6 的 void 副本、`s<n>.md` 占位号没有清理或回收的规定 | `research/scripts/ask-local.sh:127`（`save_void`）证实 exit 6 同样落 void 副本，`research/scripts/ask-local.sh:15` 头注释也写明；但定义只在 exit 5 那一分句写「先确认它大小为 0，再用 `>|` 重定向到同一个号重跑」，没有对 exit 6 写同等的清理或回收步骤——exit 6 之后那个号码留着一个 0 字节的 `s<n>.md`，本会话「停下报缺席」，之后哪个会话再用同一前缀 `ls` 取号时会把它当「已用」而跳过 | 规则没说：不是冲突（因为「停下」这个动作本身对了），是缺一句「exit 6 也会留 0 字节占位号，不重跑」 |
| 「攒到至少两份干净样本为止；连续五次调用（判红作废的也算）拿不到两份干净的，停下照实报」（`.claude/agents/three-way-local-attack.md:33`，这一步在这次 diff 里没有改动） | 只有 exit 5 会走「作废、沿用同号重跑」的循环（`.claude/agents/three-way-local-attack.md:30`），才会进入这一条「连续五次调用」的计数；exit 2、3、4、6 在第 4 步已经是终态「停下」，不会进入这个计数循环 | 一致：`5` 和 `2/3/4/6` 分别只落进第 4 步里各自那一条分句，不存在「同一退出码走进两条规定」或「哪个退出码一条都没走进」的情形 |

丙小结：定义第 4 步的排除式写法（「非 0 非 5」）在数学上是补集，逐一核对脚本的每个退出点（0、1、2、3、4、5、6）之后，
没有发现哪个退出码同时落进「作废重跑」与「停下报缺席」两条分句、也没有发现哪个退出码两条分句都没提到。
唯一的缺口是「规则没说」而非「冲突」：exit 6 与 exit 5 一样会在磁盘上留一个空的 `s<n>.md` 占位号与一份 void 副本
（`research/scripts/ask-local.sh:15`、`research/scripts/ask-local.sh:127`），但定义只对 exit 5 写了「确认大小为 0、`>|` 同号重跑」，
没有对 exit 6 写等价的处置——这个号码此后被 `ls`（`.claude/agents/three-way-local-attack.md:29`）当「已用」永久跳过，不算错，
但也没人回收。什么现象会推翻「一致」这条结论：若在 `research/scripts/ask-local.sh` 里发现某个退出码既未落进 exit 0/5 的显式分支、
也未落进「非 0 非 5」这一支（例如脚本改出一个新的分支单独 `exit 7` 却不满足「非 0 非 5」以外的第三种字面条件——这在当前脚本文本下不可能出现，
因为「非 0 非 5」按定义覆盖除 0、5 外的一切整数），或若某次实测里 exit 5 之后同号重跑真的因 noclobber 被拒绝写入（`>|` 语义被验证不成立）。

## 丁：生词表与小节名

| 改动句 | 对应代码 / 规则 | 一致与否 |
|---|---|---|
| 「闸判绿也要跑 `python3 research/scripts/oov-check.py <样本> <提示文件>`（带上提示文件，提示里的专名才不算生词）」（`.claude/agents/three-way-local-attack.md:31`） | `research/scripts/oov-check.py:249`（`scan(sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else None, words)`）把第二个命令行参数当提示文件传进 `scan`；`research/scripts/oov-check.py:164`~166（`if prompt_path: ... known = set(re.findall(...))`）把提示文件里的词收进 `known` 集合，扫描时命中 `known` 就跳过、不算生词（`research/scripts/oov-check.py:170`、`research/scripts/oov-check.py:178`） | 一致 |
| 「生词表只打前 300 个字符」（`.claude/agents/three-way-local-attack.md:31`） | `research/scripts/oov-check.py:257`（`print("     生词: " + ' '.join(dict.fromkeys(oov))[:300])`）——截的是去重后的生词按空格拼接成的字符串，掐在第 300 个字符 | 一致 |
| 「打满了就不以它为全、逐段通读样本」（`.claude/agents/three-way-local-attack.md:31`）——判得出「打满了」吗 | `research/scripts/oov-check.py:257` 的截断没有任何显式标记（没有省略号、没有「还有 N 个」之类的提示）；同一函数在 `research/scripts/oov-check.py:251`（`print("%s %s  生词=%d 拼接=%d" % ...)`）打印的「生词=%d」是 `len(oov)`——这是两段正则各自命中的**原始计数**，`research/scripts/oov-check.py:168` 与 `research/scripts/oov-check.py:176` 两个循环都会往同一个 `oov` 列表 `append`，同一个词出现几次就计几次；而 `research/scripts/oov-check.py:257` 打印的是 `dict.fromkeys(oov)` 去重后的唯一词表。拿「生词=N」与截断串里数出来的唯一词个数相减，不能当「满没满」的判据，因为 N 本身可能远大于唯一词数（重复出现），跟有没有被截断无关 | 规则没说怎么判「满」：脚本没给专门信号，`.claude/agents/three-way-local-attack.md:31` 也没写怎么数。产物里确实留了一个可数的间接信号——截断串的字符长度严格小于 300 时一定没截断，恰好等于 300 时大概率被截断（原串正好 300 字这种巧合是唯一的假阳性），但这一步「数那一行的字符数」定义原文没有写出来，是我推出来的，不算「定义已经给出判据」 |
| local-defense 改的小节名「输入（主 agent 必须给）」（`.claude/agents/three-way-local-defense.md:13`） | `.claude/agents/three-way-local-attack.md:18`（`## 输入（主 agent 必须给）`） | 一致：逐字相同（含括注与顿号写法） |
| 「做什么」「写范围」「产出」三个小节名（`.claude/agents/three-way-local-defense.md:13`） | `.claude/agents/three-way-local-attack.md:25`（`## 做什么`）、`.claude/agents/three-way-local-attack.md:35`（`## 写范围`）、`.claude/agents/three-way-local-attack.md:39`（`## 产出`） | 一致：三个小节名都逐字相同 |

读完攻方定义那四节，回头核 local-defense 自己「与本地攻方不同的地方」一节的四条差异：

| local-defense 的差异句 | 对着攻方今天的原文 | 还站得住吗 |
|---|---|---|
| 「立场是辩方：…同样按攻方第 1 步给事实表、要模型逐格填。…攻方问题同样要逐句核转述。」（`.claude/agents/three-way-local-defense.md:18`） | 攻方第 1 步事实表要求见 `.claude/agents/three-way-local-attack.md:27`，第 2 步转述核对要求见 `.claude/agents/three-way-local-attack.md:28` | 站得住 |
| 「输入里的「攻击面」换成「要辩护的一方」」（`.claude/agents/three-way-local-defense.md:19`） | 攻方「输入（主 agent 必须给）」一节里「攻击面」一词出现在 `.claude/agents/three-way-local-attack.md:20`（「分给本地攻方的攻击面（与云端攻方腿不重叠）」） | 站得住 |
| 「行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」」（`.claude/agents/three-way-local-defense.md:20`） | 攻方第 1 步原句「核对表与运行记录里把样本自带的行号一律标「模型自给、未核」」（`.claude/agents/three-way-local-attack.md:27`） | 站得住 |
| 「前缀取派发提示给的（与攻方「输入」那一项同），不写死」（`.claude/agents/three-way-local-defense.md:21`） | 攻方那一节今天的标题是「输入（主 agent 必须给）」（`.claude/agents/three-way-local-attack.md:18`），不再是单字「输入」 | 站不住一半：这句自己引用的小节名还停在改名之前的旧写法，与同一份文件第 13 行刚刚改过的写法（「输入（主 agent 必须给）」）不一致，也与攻方今天的标题（`.claude/agents/three-way-local-attack.md:18`）逐字不同——「输入」是「输入（主 agent 必须给）」的前缀子串，人读仍能找到那一节，不构成指向错误的节，但这是一处没有跟着这次改名走的残留引用 |

丁小结：oov-check 带提示文件与 300 字截断两处改动都与今天的脚本逐字对得上；local-defense 第 13 行改的小节名与攻方今天的标题逐字一致，
第 18、19、20 行的三条差异经攻方今天的原文核对后都站得住。第 21 行「攻方「输入」那一项」是本轮判定范围内发现的唯一一处站不住：
它引用的小节名没有跟着 `.claude/agents/three-way-local-defense.md:13` 这次改名同步更新，还停在改名前的短称。
什么现象会推翻「站不住」这条结论：若攻方定义把标题改回单字「输入」（那样第 21 行反而对了）；
或若能证明「输入」在 kb 与规则语境里是一个允许被当成任意前缀引用的legal简称（本轮材料未见这类约定）。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 丙 | 一致 | 「非 0 非 5」是补集写法，逐一核对 exit 0/1/2/3/4/5/6 后没有重叠也没有遗漏；唯一的缺口是「规则没说」——exit 6 也留 void 副本与空占位号，定义没写要不要清理 |
| 丁-oov 参数与截断 | 一致 | 带提示文件排专名、截 300 字都与 `research/scripts/oov-check.py:249`、`research/scripts/oov-check.py:257` 逐字对得上 |
| 丁-「打满了」判据 | 规则没说 | 脚本没给截断标记，「生词=N」是原始重复计数、不能当判据；唯一可数的信号（截断串长度是否恰好 300）定义原文没写出来 |
| 丁-小节名 | 一致 | `.claude/agents/three-way-local-defense.md:13` 改的四个小节名与攻方今天四个标题逐字相同 |
| 丁-差异站得住吗 | 三条站得住、一条站不住 | `.claude/agents/three-way-local-defense.md:21` 的「攻方「输入」那一项」是改名前的残留引用，没跟着第 13 行同步 |

## 没做什么

- 不判甲、乙两格：甲（前台/后台判据与两条规则是否相反）归本地攻方腿，乙（钩子、写范围闸、看门狗）归云端攻方腿；这两格我读过背景材料，但按分工不出判定。
- 没有实跑 `ask-local.sh`（不必要：丙、丁两格靠读代码逐行对照就能判定，不需要真的调一次本地模型；且这一轮禁读清单之外没有现成提示文件可用，实跑会产生一次不在分工范围内的样本）。
- 没有检查 `.claude/hooks/`、`.claude/gate.d/` 是否会拦截这份定义描述的命令（那是乙格，云端攻方腿的范围）。
- 没有跑 `python3 research/scripts/oov-check.py --selftest` 之外的任何门禁或重型测试；这一轮「重型测试：不跑」。
- 没有对 `records/2026-09-27-同步远端冲突块.md`「去向」一节以外的冲突块逐条复核（那些不在背景材料的抄录范围内，按小节清单标「不抄」）。
