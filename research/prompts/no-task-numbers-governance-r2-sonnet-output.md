# 进仓文字不写任务号治理文件轮第二轮：辩方腿报告（no-task-numbers-governance-r2，2026-11-19）

被复核的判决：`research/prompts/no-task-numbers-governance-r1-main-verification.md`（下称判决），依据的两份腿报告：攻方 `research/prompts/no-task-numbers-governance-r1-opus-output.md`（下称攻方报告）、正推 `research/prompts/no-task-numbers-governance-r1-sonnet-output.md`（下称正推报告）。用户原话在 `records/2026-11-19-进仓文字不写任务号与会话短号.md` 第 5 行与第 25 行。被判文件都在主工作区现读；判据库此刻的 sha256 与开工快照 `research/prompts/no-task-numbers-governance-r2-snapshot/judged-sha256.txt` 登记的一份相同（量的：`sha256sum` 得 `8eaed873…`，快照里同一串）。没读禁读清单里的文件。

这份报告自己过着判据库（任务号照判、反引号里也判），所以：现存行里的号一律不照抄，写成 `<号>`、`N`，只给 文件:行号；引文只引不带号的那几句。「成立」在这份报告里统一指**辩方这一条打中**：判决那一条够不着、结论比证据宽、或改法换上之后同一个缺陷换了形状还在；「不成立」是判决那一条站得住；「推不出」是手里的证据判不了。

## 要主 agent 跑的命令

模型目录：`research/prompts/no-task-numbers-governance-r2-sonnet-model/`。只有一份探针（造的 33 行喂判据库，对读正则推的预期）和一份纯文本量（records 下表格里批号落在哪一列），都不改仓里任何文件；探针对判据库的读数写「推的，待主 agent 跑」，纯文本量我自己跑过两遍（草稿版与模型目录版，读数相同，量的，见 Q2 第 3 行）。

1. `task-server/task submit --cwd /home/fy5090/code/singlefs -- bash research/prompts/no-task-numbers-governance-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-sonnet-run /home/fy5090/code/singlefs/research/prompts/no-task-numbers-governance-r2-sonnet-model`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`。
   - 看哪几行：任务末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0` 算过；`task-server/task log <号> --all` 里 `name=rerun_probe_cases exit=0` 与下一行 `name=probe_summary cases=33 disagreements=0`（有不一致退 1，逐条列在 `name=probe_case … agree=no` 行，那一条就是我读正则推错的）；`name=table_header_position`、`name=table_batch_cells` 两行是 Q2 第 3 行的数。退 3 是判据库不是快照那一份。
   - 范围声明：这是模型复跑，不是门禁与崩溃入口；被范围声明闸拒了照拒绝说明补 `SINGLEFS_RUN_SCOPE`。

要判决引的话：每条 `agree=yes` 的 `library=` 值就是该条「放 / 拒」的读数；Q1、Q3 里标「推的，待主 agent 跑」的，对应探针编号写在括号里。

| 文件 | sha256 | 做什么 |
|---|---|---|
| `tools/probe_cases.py` | `96593ce0b952b07d7635a00b62bd67d55b0913a3c755d41675144b182710691b` | 33 条造的行（编号 A1 到 F5）当成指定路径的 `.md` 喂 `all_hits`，打 `name=probe_case` 行与 `name=probe_summary` |
| `tools/scan_table_columns.py` | `3c77d4633a0346c68685b58c71380102debd6b53e2f990720de19e2cbbb98de3` | 只读 `records/*.md`：表头里有「批次」的表它在不在最后一列、批号落在「批次」列里与别的列里各多少个 |
| `tools/rerun.sh` | `7c906b45b7f7c4d331a2e11259d8be64cc955273de42a8a446d5b38ed0d2e026` | 从 `scripts/agents/three-way-rerun-template.sh` 拷、只换了 `rerun_steps`：核判据库 sha256、起上面两份 |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` | 先删 `-run` 收尾、在工作根 `~/.local/share/singlefs/work/` 下的那一个运行目录，再调同一个 `rerun.sh`；别的目录一律不删、退 2 |

探针的内存与时间：推的，没量（探针只在内存里判 33 行，估计一秒内、几十 MB 以内）；纯文本量读 `records/` 下 227 份、共约 29 MB（`du -sh` 与 `ls | wc -l` 量的），我跑那一份是几秒的量级，没量峰值内存（推的，按逐份读逐份放估在 100 MB 以内）。第二台上是同一份文件、同一条命令，不读本机独有状态。

## 判据与读法

- 判据（派发给的）：一条判决的结论比证据宽、或改法换上之后同一个缺陷换个形状还在（现读 `.claude/gate.d/lib-forbidden-notations.py` 核），就是打中；第 2 问找到原话推不出的判法、或被冤枉的写法，就是打中；第 3 问报告里提了、判决里找不到去向的，就是打中。
- 「量的」是我直接 `grep` 或纯文本脚本量到的；「推的」是读正则推的，探针编号对应待主 agent 跑的那一格。
- 判决条目编号照判决「二、判决」的 1 到 7。

## Q1 判决「二、判决」七条逐条

| 判决条目 | 辩方判定 | 一句话 |
|---|---|---|
| 1 漏拒 | 证据站得住；改法够不着判决自己列的两形：**成立** | 十一条现存真行都在、都带所说的形；但判决列的「单个的同 <号>」与号后接 ASCII 冒号的，改法换上之后仍放过（A1、A2） |
| 2 误拒 | 证据站得住；四个子项里三个打中：**成立** | 「判决旧写法」不是误拒、改法也没让它放行；括号步号的改法放过了判决里的两处真短号；泛指次序的改法把中文数字批号整个放掉 |
| 3 放行口子被借 | 证据站得住；改法换形还在：**成立** | 表格行从「任何文件」缩到「records 下任何列」，同一缺陷换了形（D1、D2）；反引号只对任务号关上，批号与短号仍能借（B5、C6） |
| 4 出路自己带号 | 判法站得住；闸没挡的同形一处：**成立（轻）** | 反引号里的任务号在 kb、规则、定义、技能里没有被冤枉的；结局行的 `log=` 一段文件名里带号，闸不判（F5） |
| 5 保留两处认法不一 | **不成立** | 判据库、派发闸、规则三处现在对得上 |
| 6 骨架要求写号 | **不成立** | 骨架两处提示现读已改 |
| 7 正推第 2 条与行号 | **成立（轻，只是引文）** | 判决给正推腿加的引号句不是腿的原话；事实（正文行号错）对 |

### 1 漏拒（`research/prompts/no-task-numbers-governance-r1-main-verification.md:24`）

- 证据（站得住，量的）：判决列的各形在现存记录里都有原样的行，我逐行读过：`records/2026-11-01-判定库服务化.md:473`（任务后接全角括号）、`records/2026-11-12-subagent账本清欠.md:63`（任务后接冒号）、`records/2026-11-02-文档代码逐项审核.md:549` 与 `records/2026-11-16-三方豁免取消与存量补轮.md:595`（号后接 `/`）、`records/2026-11-01-判定库服务化.md:327`（同 <号>/<号>）、`records/2026-11-11-欠账表按代码重梳调度.md:88`（单个同 <号>）、`records/2026-10-31-task-server巡检.md:13`（任务后接 `#`，表格行）、`records/2026-10-18-里程碑三收尾与近期欠账.md:166`（括号短号后接话）、`records/2026-10-19-里程碑四GPU垃圾回收开工.md:22` 与 `records/2026-11-10-全量提交门禁收尾调度.md:117`（`批 #N`）、`records/2026-10-13-Verus证明审计.md:26`（`第 N–M 批`）。判决说的「整行原样放行」对 `:88` 要缩一句：那一行放行的主因是一串光秃的号，库文件头写明管不到（`.claude/gate.d/lib-forbidden-notations.py:94`：「管不到的：没带「任务」「task」「同」的裸任务号」），单个的「同 <号>」只是其中一处。
- 够不着（推的，待主 agent 跑 A1、A2）：判决列的漏拒里有「单个的同 <号>」；改法（判决 `:36`）只认「与 / 和 / /」连的并列，库文件头此刻仍写「同」要「两个以上并列的号」（`.claude/gate.d/lib-forbidden-notations.py:69`：「「同」后面跟两个以上并列的号」）。攻方当时就把这一形标成交主 agent 定（`research/prompts/no-task-numbers-governance-r1-opus-output.md:147`：「单个 `同 N` 要不要判交主 agent 定」），判决没记决定。`records/` 下单个的同 <号>（不带并列）我用近似正则数得 28 处（`grep -rnoP` 的匹配数，推的，不认围栏与反引号）。同一格里另有：号后接 ASCII 冒号的 V06 形（`任务 <号>:`）攻方写明不修（`research/prompts/no-task-numbers-governance-r1-opus-output.md:146`：「V06（`任务 N:` 加 ASCII 冒号与行号分不开，推的）」），判决没提，库此刻仍把冒号排除在号后面（`.claude/gate.d/lib-forbidden-notations.py:233` 的 `TASK_NUMBER_AFTER`）。
- 改法宽过证据（推的，A5）：判决 `:36` 的「后面跟「个」「条」的放过」，证据只有一行用到「个」（`records/2026-11-10-全量提交门禁收尾调度.md:276`），「条」在 `records/`、kb、`research/prompts/` 里一行没有（量的：`grep -rnoP '任务\s*#?\d{4,}\s*[个条]'` 只命中那一行与这一轮自己的材料）；库里是单字跟随（`.claude/gate.d/lib-forbidden-notations.py:242`：`(?!\s*[个条])`），所以「任务 <号> 个别格退 1」这类真任务号也放过。
- 什么现象会推翻：A1、A2、A5 在主 agent 跑的读数里是 `library=refuse`，或库文件头写明这几形留作已知管不到并写了理由。

### 2 误拒（`research/prompts/no-task-numbers-governance-r1-main-verification.md:25`）

证据逐处读过（量的）：`.claude/kb/decisions/16-发布语义.md:177`（`seq 批 1`）、`.claude/kb/experiments/100-系统配置的三段几何.md:6`（「mkfs 写的第一批字节就是它」）、`.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:416`（括号 `a6`）、`.claude/kb/layout/01-first-txn.md:441`（括号 `a1`）、`records/2026-11-10-全量提交门禁收尾调度.md:276`（任务个数）都在、都是所说的形。四个子项：

1. **「判决旧写法」不是误拒，改法也没让它放行：成立。** 规则现在写的是「判决「改法」一节那一条下面单起一行，行首写」那种（`.claude/rules/three-way-inference.md:38`：「单起一行，行首写」），派发闸认的也是行首加冒号（`.claude/hooks/runner-dispatch-guard.sh:2569` 的 `ZERO_ROUND_READING_LINE`）。旧判决那两行写的是「零轮，A/B 读数任务」后面直接跟号、没有冒号、也不在行首（`research/prompts/device-library-write-many-code-r3-main-verification.md:42` 与 `:43`：「零轮，A/B 读数任务」），按规则它本来就不合规；改法 4 把放行锚到行首加冒号，库现在对它仍判拒（E1、E3，推的）。判决把它记成「误拒坐实」，又没让它放行，两头对不上：要么删出误拒清单，要么说明为什么仍拒。
2. **括号步号：成立。** 改法（判决 `:37`）：不挨会话词、数字字母混写的括号短号 kb 与 `research/prompts/` 里放过，理由是「（那里是步号）」（判决 `:37`：「kb 与 `research/prompts/` 里放过（那里是步号）」）。kb 一侧站得住：kb 里中文页（不含 `.en.md`、`.ja.md`）此刻括号数字字母短号 7 行，都是步号或标签（量的：`grep -rnP` 按库的正则改写，命中 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:416`、`:1379`、`:1459`，`.claude/kb/layout/01-first-txn.md:441`，`.claude/kb/decisions-history.md:9478`，`.claude/kb/experiments/178-叶容器与记录死亡的计数.md:618`、`:619`）。`research/prompts/` 一侧不是：攻方自己写明那 54 处里两处是真短号（`research/prompts/no-task-numbers-governance-r1-opus-output.md:85`：「`research/prompts` 那 54 处里两处是真短号」），我找到了这两行：`research/prompts/m4-close-c1040-c1041-code-r1-main-verification.md:50`（判决文件，「借卡那一批落主工作区之后的样子为基线」）与 `research/prompts/gpu-tests-placement-design-r2-sonnet-output.md:384`（会话列举，「以它们落完的样子为基线」）。改法换上之后这两行写进 `research/prompts/` 就放过（C1、C2，推的），而用户的范围含判决。同一个缺陷（短号写进仓）换形还在；`records/` 里步号仍被拒（C5），出路是放反引号，而反引号里写真短号同样放过（C6）。更窄的做法同样满足「不误拒步号」：只在 `.claude/kb/` 放过（那里 7 行全是步号），`research/prompts/` 照判，步号在那里放反引号（推的，没造探针）。
3. **任务个数：不成立，但改法宽过证据**（见 1 的第三点）。
4. **泛指次序：成立。** 证据只有一行真误拒（`.claude/kb/experiments/100-系统配置的三段几何.md:6`），攻方当时写明那一类里「混着调度批号」（`research/prompts/no-task-numbers-governance-r1-opus-output.md:84`：「里面混着调度批号」）；改法（判决 `:38`：「中文数字的「第几批」不判（泛指次序）」）把所有中文数字的批号整个放掉。`records/` 里这些多半是真调度批号（量的：`records/2026-11-15` 到 `records/2026-11-18` 的几份记录里「第<中文数字>批」共 34 处，我读了前 12 处全是调度批号，例如 `records/2026-11-18-等GPU拆到锁外与前台等锁让路.md:59`「写段第一批」、`records/2026-11-17-里程碑四收口续做.md:135`「第二批实现员已派」）。用户原话点的就是「批 N」写进记录，换成中文数字写法一样放过。更窄的做法：中文数字的批号只在 `.claude/kb/` 放过（泛指次序住在那里），`records/` 与 `research/prompts/` 照判（推的，kb 里泛指与真批号各占多少我没分，攻方也没分）。
- 什么现象会推翻：E1 读数是 `library=pass`（判决旧写法此刻被放行），或判决写明旧判决不回改、所以不算改法要解的误拒；C1、C2 读数是 `library=refuse`；`records/` 里中文数字批号的 34 处里多数是泛指次序。

### 3 放行口子被借（`research/prompts/no-task-numbers-governance-r1-main-verification.md:26`）

- 证据（站得住）：「A/B 读数：」行内任意位置、长跑拆分行、表格行、反引号、半截围栏五处攻方都有探针与读数（`research/prompts/no-task-numbers-governance-r1-opus-output.md:107` 到 `:111`）；围栏那一格的现场文件在（量的：`ls -la` 得 133 字节，`/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-opus/probes/heredoc-half-fence.after.md`）。改法 4 之后「A/B 读数」与围栏前文两处我查到的现状：库里的三条正则与派发闸逐字相同（量的：`.claude/gate.d/lib-forbidden-notations.py:260` 与 `.claude/hooks/runner-dispatch-guard.sh:2569` 的 `ZERO_ROUND_READING_LINE`），库的 `preceding_text` 接法在（`.claude/gate.d/lib-forbidden-notations.py:452`）。这两处判决的改法站得住。
- 小处：判决写「共用约束「号一律不写」没写这个例外」，但「号一律不写」不在共用约束里：`grep -rnF '号一律不写' .claude` 零命中，只在用户原话那一行（`records/2026-11-19-进仓文字不写任务号与会话短号.md:5`：「号一律不写」，弹窗选项）。结论（反引号例外没写）对，引文归属错（成立，轻）。
- 改法换形还在：**成立。**
  - 表格行：改法 4 把放过缩到 `records/` 下，但仍是任何表、任何列。判决自己说「宽过定案的「依赖表批次那一列」」，缩一圈之后在 `records/` 里还是同样宽（D1、D2，推的）。我量了 `records/` 下的表（量的，`tools/scan_table_columns.py`）：批号落在表头是「批次」那一格的有 52 个，落在别的列的 559 个（依赖 141、去向 119、件 65、现状 53 …）；这一轮自己的记录第 20、21 行就把批号写在「依赖」列（`records/2026-11-19-进仓文字不写任务号与会话短号.md:20`、`:21`），闸放行，共用约束写着别的列照样不写（`.claude/agent-common.md:43`：「别的列、别的表照样不写」）。规则与闸的差，这一轮自己就踩了。
  - 反引号与围栏：改法 5 只对任务号关上。批号与会话短号在反引号里、围栏里仍整行放过（B5、B6、C6，推的）；判决 `:26` 把「反引号三条判据行都放过」记成口子，改法只关了三条里的一条。共用约束第 44 行也写明留着（`.claude/agent-common.md:44`：「反引号与围栏代码块只放过批号与会话短号形状的代码字面」），这是有意的取舍，但「形状的代码字面」闸判不出是不是代码字面，写在反引号里的真批号、真短号照放；这是同一个口子换成只剩两种号。
  - 「#第 N 批」锚点：判决没提，见 Q3 第 3 条。
- 什么现象会推翻：D1、D2 读数是 `library=refuse`（表格行不再整行放）；或 B5、C6 是 `library=refuse`。

### 4 出路自己带号（`research/prompts/no-task-numbers-governance-r1-main-verification.md:27`）

- 判法站得住：攻方报告 R1（反引号里写 task 子命令加号）、R2（结局行原文）读数在 `research/prompts/no-task-numbers-governance-r1-opus-output.md:129`；正推报告对 R2 标「推不出」，因为用户原话本来就列了结局行原文（`research/prompts/no-task-numbers-governance-r1-sonnet-output.md:35`：「推不出：结局行原文本身带任务号。」），判决让主 agent 判，判成「只写 `type=` 与 `exit=` 一段」，见 Q2 第 1 行，站得住。
- 反引号一半没被冤枉（量的）：我用纯文本脚本数 `.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills` 里反引号代码段里带 task 号形状的，只有 14 行（含 `.en.md`、`.ja.md` 译文，去重 5 处），全是命令里的真任务号：`task-server/task log` 或 `status` 后接号，和一处 `cmp` 对着任务的 stdout；没有一处是计数或别的东西。反引号对任务号不是放行口子的取舍，在这几个目录上没有被冤枉的（草稿里的输出 `spans-out.txt`，在 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-sonnet/`）。
- 同形缺陷未挡（推的，F5，轻）：结局行带 `task=<号>`，行里 `log=` 一段是日志文件路径（`task-server/task_server_lib.py:4684`：`log={log or`），文件名本身带号与会话哈希，形如 `task-<八位十六进制>-<号>.attempt-N.log`（量的：`grep -rlP 'task-[0-9a-f]{8}-\d{3,}\.attempt' records .claude/kb research/prompts` 命中 40 份文件、`grep -o` 数得 119 处）。判据库的任务号正则要「task」后接空白、`=`、`#`、冒号或括号再接号，`task-` 后接哈希不命中，所以文件名里的号闸不判。共用约束写的是只抄 `type=` 与 `exit=` 一段（`.claude/agent-common.md:42`），没叫人抄 `log=`，规则文字没缺；缺的是闸，写「日志里的那一行」的人把路径抄进来就带出去。改法 5 没提这一形，它取代的「结局行原文」带出的 `log=` 路径与此同形。
- 什么现象会推翻：F5 读数是 `library=refuse`；或改法写明文件名里的号交写的人。

### 5 保留两处认法不一（`research/prompts/no-task-numbers-governance-r1-main-verification.md:28`）：不成立

三处对得上（量的）：`.claude/hooks/runner-dispatch-guard.sh:2569` 与 `.claude/gate.d/lib-forbidden-notations.py:260` 的 `ZERO_ROUND_READING_LINE` 逐字相同，规则 `.claude/rules/three-way-inference.md:38` 写「行首写」加「A/B 读数：任务 <号>」，库里的值部分认法也取同一条（`.claude/gate.d/lib-forbidden-notations.py:261` 的 `ZERO_ROUND_TASK_NUMBER`）。原来的不一致（规则写 `<任务号>`、闸要 `任务 <号>`）已经没了。推翻：派发闸或库的这两行此后被改而不同步（库头注写了逐字相同，没有自证盯这一点，我没查自证）。

### 6 骨架要求写号（`research/prompts/no-task-numbers-governance-r1-main-verification.md:29`）：不成立

现读骨架已改：`scripts/agents/verdict-skeleton.py:66` 的待填提示写「判决引的读数由主 agent 复跑，写那条命令与结局行的 type= 与 exit= 那一段，不写任务号」，`:68` 起叫人在零轮改法那一条下面单起一行，骨架自证（`:288` 起）核「骨架的待填提示不要求把任务号写进判决」。旧行不再要求写号，换上之后没有同形缺陷。推翻：骨架自证在主 agent 复跑里红。

### 7 正推第 2 条与行号（`research/prompts/no-task-numbers-governance-r1-main-verification.md:30`）：成立（轻）

判决写「正推腿「原话在第 22 行，实际在第 25 行」坐实」。正推报告里没有这句：`grep -nF '原话在第 22 行'` 对正推报告零命中；腿原话是「派发提示说的第 22 行此刻没有那句」（`research/prompts/no-task-numbers-governance-r1-sonnet-output.md:7`），指的是派发提示给的行号。事实对：正文自己也写第 22 行（`research/prompts/_no-task-numbers-governance-r1-body.md:3`：「第 5 行与第 22 行引的原话」），实际在第 25 行（`records/2026-11-19-进仓文字不写任务号与会话短号.md:25`）。只是把话记在腿头上；不影响判决。

## Q2 「交用户的」表三处判法（`research/prompts/no-task-numbers-governance-r1-main-verification.md:48` 到 `:50`）

用户原话两处：第 5 行（任务号、别的会话的批号、会话短号不写；改写成「哪条命令、结局行原文、日志里哪一行、谁在改哪个文件」；弹窗选「号一律不写」），第 25 行（自己会话的批号也不写，「只留依赖表批次那一列」）。共用约束现在的那一条在 `.claude/agent-common.md:40` 到 `:45`。

| 判法 | 判定 | 一句话 |
|---|---|---|
| 1 结局行只写 `type=` 与 `exit=` 那一段、号不进仓 | **不成立**（判法站得住，依据格引错句） | 从「号一律不写」推得出；`log=` 一段文件名里也带号，所以比「只删 `task=`」更稳 |
| 2 反引号里的任务号照判 | **不成立**（判法站得住）；被冤枉的面**推不出** | 数据支持，但表里「同上」的依据对批号与短号同样成立而判法只关任务号，这不对称没说理由；三方腿报告引证据行时受限，范围算不算在用户列的里面推不出 |
| 3 表格行批号只在 `records/` 放过、不细到哪一列 | **成立** | 比原话宽；「闸认不出列」这句同一轮自己做的前文接法就推翻得了；有更窄的同样满足原话 |

### 1 结局行只写 `type=` 与 `exit=`（判决 `:48`）

- 从原话推得出吗：推得出，但依据格引的不是那一句。判决依据格引「这些任务编号 对之后的任务追溯几乎 起不到什么作用」（`research/prompts/no-task-numbers-governance-r1-main-verification.md:48`：「用户原话「这些任务编号 对之后的任务追溯几乎 起不到什么作用」」），那句只说号没用，没说结局行怎么写。真正推出判法的是第 5 行里「弹窗三选一选「号一律不写」」加同一行列的改写办法里有「结局行原文」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:5`：「改写那件事本身：哪条命令、结局行原文、日志里哪一行、谁在改哪个文件」）：原文带 `task=<号>`，两句要同时成立，只能是结局行去掉号。依据格该换成这一句。
- 有没有更窄的判法同样满足原话：「只删 `task=` 一段、其余照抄」是更窄的。我读了结局行的格式（`task-server/task_server_lib.py:4683` 与 `:4684`：`task={task.get('id')}`、`log={log or`）：`log=` 一段是日志文件路径，文件名带号（见 Q1 第 4 条，119 处，量的），照抄 `log=` 就把号带进仓。所以「只写 `type=` 与 `exit=`」不是过严，而是比「只删 `task=`」更接近原话；判决没写这个理由，补上更稳。
- 被冤枉的写法（推的）：共用约束的报告一节同时要求「贴的命令输出是这一次真跑出来的原样」（`.claude/agent-common.md:125`），结局行整行贴进报告就被闸拒，要换成只抄两段。两句话在同一份文件里是张力，不是冲突：判决改法 5 只改了写号那一条，没动报告一节，写报告的人得自己权衡。轻，不推翻判法。
- 什么现象会推翻：弹窗选项里有一项明说「结局行整行照抄、号除外也行」；`records/2026-11-19-进仓文字不写任务号与会话短号.md:5` 只写了「三选一」没列另两项，我推不出选项原文，要主 agent 回看弹窗。

### 2 反引号里的任务号照判（判决 `:49`）

- 判法对不对（量的，站得住）：用纯文本脚本数了 `.claude/kb`、`.claude/rules`、`.claude/agents`、`.claude/skills`、`.claude/main-agent.md` 里反引号代码段中带任务号形状的：去重 5 处，全是命令里的真号（Q1 第 4 条）；没有一处是计数或别的东西。这几个目录上「反引号只放过不是任务号的代码字面」没有被冤枉的。`records/` 与 `research/prompts/` 里同形 125 处（草稿里的 `spans-rec.txt`），多数是旧记录里的 `task clean <号>`、结局行与日志名，也是真号；非真号的集中在上一轮三份腿报告里引用户例句与探针输入。
- 依据格的缺口：判决写「同上，用户在弹窗里选的是「号一律不写」」（`research/prompts/no-task-numbers-governance-r1-main-verification.md:49`：「同上，用户在弹窗里选的是「号一律不写」」）。「一律」对批号与会话短号同样成立，而判法只关任务号：批号与短号放在反引号、围栏里仍整行放过（B5、B6、C6，推的）。要成立得说出不对称的理由，我从数据能给：任务号形状的反引号字面在这几个目录里全是真号，批号、短号形状的反引号字面大量是步号与参数（Q1 第 2 条的 7 行与 `seq 批 1` 一类），关了会大面积误拒。这条理由在判决里没有，写进去。
- 被冤枉的写法（推不出）：三方腿的报告要引现存行当证据，行里带号（上一轮攻方报告 `research/prompts/no-task-numbers-governance-r1-opus-output.md:107` 与 `:108` 把带号的整行放反引号里引，正推报告 `research/prompts/no-task-numbers-governance-r1-sonnet-output.md:93`、`:94` 同样）。照判之后这些引文写不进仓，腿得换成 `<号>` 占位或转述（三方腿的定义已给出口：命中 0 次的引文改成转述，`.claude/agents/three-way-defense.md:28`：「命中 0 次不写成引文（换成转述并标「转述」）」）。这是不是冤枉取决于用户列的范围：原话列的是「`records/`、kb、判决、commit message」，腿报告在 `research/prompts/` 里但不在列的名目里；判据库却对所有 `.md` 判（`.claude/gate.d/lib-forbidden-notations.py:720` 起的排除前缀只有上游副本与两个样本目录）。原话没说腿报告算不算，我推不出；要问用户一句，或在共用约束里写明腿报告里引证据行的写法。
- 什么现象会推翻：反引号代码段里出现一处合法的、不是任务号的 `任务 <四位数>` 或 `task=<四位数>`（计数、端口、字节数），我没找到；或用户答腿报告不在范围。

### 3 表格行批号只在 `records/` 放过、不细到哪一列（判决 `:50`）：成立

- 原话推得出吗：推不出。用户第 25 行是「只留依赖表批次那一列」（`records/2026-11-19-进仓文字不写任务号与会话短号.md:25`：「只留依赖表批次那一列」）；判法放过的是 `records/` 下任何表任何列，共用约束自己都写了别的列照样不写（`.claude/agent-common.md:43`），闸放而规则禁，规则与闸的差就是缺口。这一轮自己的记录就踩了：依赖表的「依赖」列写着批号（`records/2026-11-19-进仓文字不写任务号与会话短号.md:20`、`:21`），闸放行。
- 量的（`tools/scan_table_columns.py`，我跑过两遍，草稿版与模型目录版读数相同）：`records/` 下表头含「批次」的表 232 张，其中「批次」在最后一列 118 张、不在最后一列 114 张，所以「取最后一格」这种不看表头的窄办法不成立；批号落在「批次」列里的单元格 52 个，落在别的列的 559 个。判法放过的范围是原话保留范围的十多倍。
- 判决给的理由「闸认不出是不是依赖表」（判决 `:50`：「不细到哪一列（闸认不出是不是依赖表）」）被同一轮的改法推翻得了：改法 4 给写闸、检出钩子与几份写入脚本都接了 `preceding_text`（`.claude/gate.d/lib-forbidden-notations.py:452`：「插入点前面的那段文字」，围栏用的），Write 是整份内容、Edit 是文件里插入点前面、追加是整份文件，表头都在手里。有更窄的同样满足原话的判法：批号只在「表头是「批次」那一格对应的列」里放过，其余列照判。正推报告当时就把它列成推翻条件（`research/prompts/no-task-numbers-governance-r1-sonnet-output.md:121`：「判据库认得出依赖表表头只放批次列」），判决没采纳也没写为什么不采纳，只写了一句认不出（推的：我没写这个实现，也没量过它在 Edit 单行改表格里行的接法上会不会取不到表头；单行改动的表头可能不在 `old_string` 里但在文件里，`preceding_text` 的接法能取到）。
- 被冤枉的写法：窄判法换上之后会拒的是依赖表以外列里的批号，这些在原话里本来就不该写，没有被冤枉的；已有行不回改。
- 什么现象会推翻：D1、D2 读数是 `library=refuse`（那就是闸已经按列判了，我读错了），或「批次」列的表头在 `records/` 里写法不一、认不出（我量的两种位置都是「批次」二字，读法没细到表头的别的写法）。

## Q3 第一轮两份报告里提了、判决里没去向的

判据：报告里提了、判决里找不到去向的一条就是打中。「去向」我看三处：判决的判与改法（`research/prompts/no-task-numbers-governance-r1-main-verification.md:24` 到 `:42`）、「交用户的」表（`:48` 到 `:50`）、换上之后的文件现状。「成立」= 无去向。

| 编号 | 报告里提的 | 判决里的去向 | 判定 |
|---|---|---|---|
| 1 | 单个的同 <号> 要不要判，交主 agent 定（攻方报告 `:147`） | 判决 `:24` 把它列成漏拒，改法 `:36` 只认并列；库文件头仍写要两个以上（`.claude/gate.d/lib-forbidden-notations.py:69`）；没有一句记决定 | **成立**（同 Q1 第 1 条，A1） |
| 2 | 号后接 ASCII 冒号的 V06 形写明不修（攻方报告 `:146`） | 判决没提；库头注把「号后面接冒号」写成设计（`.claude/gate.d/lib-forbidden-notations.py:70`） | **成立（轻）**：已知管不到，但判决没记「接受」（A2） |
| 3 | 「#第 N 批」锚点整个放过（攻方报告 `:70`：「`#第` 一律当锚点放过」；正推报告 `:103`：「「#第 N 批」锚点」，两条腿都写了） | 改法六条与「交用户的」表里都没有；库头仍写放过（`.claude/gate.d/lib-forbidden-notations.py:79`：「「#第 N 批」（调度记录的锚点）不算」）；共用约束又写「派发提示里的「批次：<调度记录>#<批号>」不进仓」（`.claude/agent-common.md:43`） | **成立**：闸放、规则禁，任何文件里写 `records/x.md#第 N 批` 都过（B4） |
| 4 | 量词表去掉「只」「处」「行」「位」「道」（攻方报告 `:150`：「量词表去掉「只」「处」「行」「位」「道」这几个常作副词与名词的字」），同一行写明去掉之后「批 3 处」这类真计数会新误拒 | 改法只去「只」（判决 `:38`：「量词表去掉「只」」），库里其余四个还在（`.claude/gate.d/lib-forbidden-notations.py:253`），没写为什么只去一个 | **成立（轻）**：我 grep「批 N 处理 / 位于 / 行动 …」类，`records/`、kb、`research/prompts/` 里只找到一行，那一行是计数（`records/2026-10-26-harness审计三项修复.md:63`），没找到被它放过的真批号（量的），所以没有实害，只是取舍没记 |
| 5 | 这一形态没有提交时的格，漏过就是终局；检出钩子不判 cp / mv 与别的命令的输出，草稿里写好再拷进 `research/prompts/` 不过任何一道；`records/` 下的 `.json` 不在判的后缀里（攻方报告 `:115`） | 判决没有一句；库头仍写这一形态没有 doc-text 的格（`.claude/gate.d/lib-forbidden-notations.py:143`）；检出钩子头注仍写不判（`.claude/hooks/bash-command-detector.sh:359`：「内容不是字面的（cat 文件、cp、mv、别的命令的输出）」） | **成立**：这是写闸之外唯一的兜底缺口，接不接受要主 agent 判，判了写进库头注的管不到清单 |
| 6 | ⑰ 的拒绝说明只取 remedy 第一条，拼接处多出「；。」（攻方报告 `:131`：「拼接处多出一个「；。」」） | 判决没提；但换上的检出钩子已改成取整条 remedy、各条去掉收尾的「；」「。」再接（`.claude/hooks/bash-command-detector.sh:5653` 起的 `literal_notation_remedies`，`:5657`），在改前改后 diff 里（`/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r1-materials/round-one-fixes.diff`） | **去向在文件、判决没记**：实质已修，按「判决里找不到」的字面算打中，轻 |
| 7 | 会话词表另加「范围」「判过」（攻方报告 `:149`：「转述 / 范围 / 判过」） | 判决 `:37` 写「这类词」；库里没有这两个（`.claude/gate.d/lib-forbidden-notations.py:246`） | **成立（极轻）**：我没找到现存行带这两个词的真短号，没有实害 |
| 8 | 记录里不再写会话名之后，「认会话看变异行与记录里的名字」这一办法在记录一侧变弱，标「推不出冲突」（正推报告 `:61`：「这一办法在记录一侧会变弱」） | 判决 `:30` 只记「腿没找到新的」；那一句不是要求写号的句子，所以没被收进去 | **成立（轻）**：`.claude/skills/dispatch/SKILL.md:20` 这一句现状仍写着那个办法，读法变弱是后果不是缺陷，要主 agent 判是不是把那句改成只指 `who-touched.py` 与 `ListAgents` |
| 9 | 没核 `.git/hooks` 有没有判 commit message 的钩子，写「推的」（正推报告 `:138`：「没核 `.git/hooks` 里有没有判 commit message 的钩子」） | 判决没提 | **不成立**：我核了（量的）：`git config --get core.hooksPath` 空，`ls .git/hooks` 除样本外只有 pre-commit；门禁 doc-text 自己写明 commit message 管不到（`.claude/gate.d/doc-text.sh:121`：「commit message（提交之后才有，要另挂 commit-msg 钩子）」），共用约束那一句成立（`.claude/agent-common.md:45`：「commit message 没有闸判，靠写的人」） |
| 10 | 依赖表只放批次列：判据库认得出表头，或规则加一句（正推报告 `:121`） | 规则加了一句（`.claude/agent-common.md:43`），库那条「认得出表头」只用一句「闸认不出」带过（判决 `:50`） | **有去向，判断不足**：见 Q2 第 3 行 |
| 11 | 半截围栏的 B5、B7（攻方报告 `:153`：改法修不了「整段新文字自己就是开着的围栏」） | 改法 4 修了前文接法；围栏里批号与短号放过是有意（`.claude/agent-common.md:44`），任务号在围栏里现在照判 | **不成立**：去向是这一条规则本身 |

## 什么现象会推翻

- Q3 第 1、3、5 条：主 agent 回看判决与「交用户的」表，找到这三条的去向（改法里的一行、或写明接受的一句）。A1、B4 读数是 `library=refuse`，或库头注把它们列进管不到。
- Q1 第 2、3 条、Q2 第 3 行：C1、C2、D1、D2、B5 读数是 `library=refuse`。
- 所有「推的」：探针 `name=probe_summary cases=33 disagreements=0` 说明我读正则推的预期都对；有 `agree=no` 的那一条就是我推错的。

## 没做什么

- 没撤销或维持判决，只交复核结果；Q1 到 Q3 里的「成立」是辩方打中，不是判决被推翻，由主 agent 判。
- 没往统一任务服务交任务；判决要引的数（探针 33 行、纯文本量）没由主 agent 复跑，命令在「要主 agent 跑的命令」一节；探针读数一律「推的，待主 agent 跑」。
- 没读禁读清单里的攻方第二轮报告与模型目录，也没读攻方腿的派发提示；所以判决第 1 条「判据是派发提示关 1」那句的出处我没核，只核了它引的现存行。
- 没改被判文件、没建仓副本、没编译；没实测写闸（Write / Edit）对表格与围栏的接法，只读了库的接口（`.claude/gate.d/lib-forbidden-notations.py:452`）。
- 没造 Edit 单行改表格行时「取不到表头」的探针（Q2 第 3 行，推的）；窄判法我没实现。
- 纯文本量与草稿里的 `spans.py`、`cols.py`、`spans-out.txt`、`spans-rec.txt` 是我直接跑的文本脚本（不起判据库），读数是量的；模型目录里的两份脚本与探针经主 agent 跑才算判决能引。

## 清理

- 没建仓副本、编译目录与库拷贝；模型目录里 `py_compile` 生成的 `__pycache__` 已删。
- 草稿目录 `/home/fy5090/.local/share/singlefs/work/no-task-numbers-governance-r2-sonnet`（36K，`du -sh` 量的）只有四份文本脚本与输出，是上面引的量的证据，留着；认领时记的用途是本轮辩方腿的草稿。
