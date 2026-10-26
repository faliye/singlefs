# m3-close-gov-b17-r2 辩方腿（Sonnet）报告：H1、H5

日期：2026-10-23。立场：复核第一轮判决 `research/prompts/m3-close-gov-b17-r1-main-verification.md` 与修补 1（快照里 `governance-fix-1.patch`，主工作区现文为准）：判决每一格够不够得着、改法是不是只有一种、是不是同样打中别的读法；替被判「打中」的原文辩。引 kb 写 kb 文件自己的行号（`grep -nF` 现取）。没读禁读清单里的文件。

## 0. 复跑与输入

模型目录 `research/prompts/m3-close-gov-b17-r2-sonnet-model/`，全是 python / 只读，不起 cargo、不拷仓。复跑命令（工作目录随意，`--work` 给一个不存在的新目录）：

    SINGLEFS_RUN_SCOPE='种类=items; 跑什么=三方腿模型 tools/rerun.sh（只读判法函数、补丁片段与钩子喂命令）; 范围=1 个脚本; 为什么=核模型输出可复跑、与 out/ 逐字相同' bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-gov-b17-r2-sonnet-model/tools/rerun.sh --repo <仓根> --frozen <仓根>/research/prompts/m3-close-gov-b17-r2-snapshot --work <新目录>

末尾应打八行 `name=rerun_compare file=… result=identical`。我跑的最后一趟（退 0）原样：

    name=rerun_compare file=judged-sha256.txt result=identical sha256=67b9bb352ff8195443a73131c9e4152090aa6723dcb0bd7462e4e87942bc7e3f
    name=rerun_compare file=patch-landed.txt result=identical sha256=c5532b799146b42d67200ebee05e05715542f91ef8837b94c385cd3c8176207d
    name=rerun_compare file=g1-clauses.txt result=identical sha256=2913fbff1b3e8e04ca4a911966d7cfdc875fa6789171fc1833c9fe1a671e3230
    name=rerun_compare file=g3-tokens.txt result=identical sha256=56441c1a6d3c9dfde5e609b7f7270543339b824e3435196c933e4bf9b0562cd8
    name=rerun_compare file=edges-and-units.txt result=identical sha256=cdc9d88cfcbc2e399c3c1e5b8016647463e78e32b437c32a07fab87de14252f4
    name=rerun_compare file=h11.txt result=identical sha256=4587d6639bd9f49911cb9485fc837defca75f1e530e1ead33ea45af8e3c2a6d3
    name=rerun_compare file=unmodified-copy.txt result=identical sha256=4059d328964a4a20c932db12cd5603591fe57877dcbff36a35edd79f665d6307
    name=rerun_compare file=h5-compare.txt result=identical sha256=e8114ea1f1623a39f377231d8724c439fa8b7691f74dd1cc02fb64dddb8282e4

模型文件 sha256（写完时现算）：

    c675c939231222f8fb14c67a6ca8f15810baab4179b3ad7b4df1be1e898ea461  check_patch_landed.py
    3600f09ae1fa408ee23824ad517a9c369031e320bcfd459be009e04587b2a6da  check_g1_clauses.py
    f1050d9f6c62af90d840d721178433c26f1e1fdd7074b5d0777ac5267d9afe59  check_g3_tokens.py
    97c4605508a621a165a6ff1f58c8920bf2710bc9b29e4b5f903be61c106c874f  edges_and_units.py
    d0e432e2f1d9bc930a6cf8a451fcdd3b04c80ed72da6340ee7f2280d6eac015e  h5_compare.py
    d5367d357b6149298e347db1e778ee3b1954af11f0568ba4948e8fbc9b2ea20a  probe_h11.py
    059996c30c42510f6a1f0c20cda9f2245cbedc4d545877baa683a9b629172e38  probe_unmodified_copy.py
    6d7ecadf1d46cff8d5acd9e2b4c0d949e4ddefa58e60431afa9ffecc18b61e52  tools/rerun.sh
    其余是 out/ 下八个 .txt，sha256 即上面八行 rerun_compare 里的值

输入状态：开工时 `sha256sum -c research/prompts/m3-close-gov-b17-r2-snapshot/judged-sha256.txt` 与 `kb-sha256.txt` 全 OK。跑模型时 `scripts/mutation/crates-mutation-rows.py` 已对不上快照（别的会话在我开工之后又改了它，out/judged-sha256.txt 如实记了 FAILED）；本报告不靠它的正文，只靠它打的单元清单（465 个、出自那两份的 0 个，改前后不变）。模型输出对应的是「快照里另外 13 份 OK」的这棵树；复跑时先看 out 第一份是不是还是同一个 FAILED 集合，不是就别比其余几份。
被判规则、`.claude/agent-common.md`、`.claude/hooks/lib_heavy_tests.py`、`.claude/hooks/heavy-test-guard.sh`、`scripts/mutation/broad-mutation.py` 现文 sha256 都与快照相同。

## 1. 判定一览（判决条目 / 辩方判定 / 证据）

判定词：站得住（判决这一格的结论与修补照它写的做了，且我没找到同样打中别的读法的反例）；够不着（判决这一格没覆盖到、或修补没落到的地方）；同样打中别的臂（判决的理由换个读法一样成立于别的改法，或被采的改法有同形的代价）。证据列写文件:行与命令输出，细节在第 2、3 节。

| 编号 | 判决条目（第一轮判决位置） | 辩方判定 | 证据 |
|---|---|---|---|
| V1 | 第二节 G1「出入照正推那 5 处改」 | 站得住 | 5 处在 `.claude/rules/implementation-workflow.md:31` 与 `.claude/rules/verification.md:50` 都改了：边级不再挂「单元不进键」、去掉「新加」、文件级补「管怎么存、怎么算键，不管判对错」、verification 补边级「不沿它收」与模块级四样清单（out/g1-clauses.txt） |
| V2 | G1「「现在只有那一条」换成指 `EXCLUDED_PROGRAM_EDGES`」 | 站得住，且比判决写时更站得住 | 登记位今天三条边（out/edges-and-units.txt：`excluded_program_edges=3`），`scripts/gate/lib_function_units.py:77` 写「今天三条，都在内存包装里」；kb 定案还写「现在只有……那一条」（`.claude/kb/decisions/13-验证路线.md:435`），已与登记位不符 |
| V3 | G1「两处复述写成同一套（自证要求只写一处、另一处指过去）」 | 站得住，留一个尾 | 自证只在 implementation-workflow 一处；verification 指过去，但指的是标题前缀（`.claude/rules/verification.md:50` 写「复用重型门禁的判定：按每条判定的单元键判」，标题全文还有后半句），不是逐字标题 |
| V4 | G1「采两腿」 | 够不着（攻方 G1-c 没被回应） | 攻方判「理由不同、分开记」「管怎么存、怎么算键，不管判对错」「第二台那一路只出提示」是论证、该留在 kb（`research/prompts/m3-close-gov-b17-r1-opus-output.md:65`）；判决 G1 格只转述了攻方的另三点（`research/prompts/m3-close-gov-b17-r1-main-verification.md:21`），修补把前两句都抄进了两份规则 |
| V5 | G1 只比定案里「两种排除不进键」那一条 | 够不着 | 正推自述「我只比这一条与它的射程句」（`research/prompts/m3-close-gov-b17-r1-sonnet-output.md:43`）；两处复述同一句里还压着定案里键的组成与「一律不进键」两条，逐片比见 2.2：规则写「解析出的依赖版本」而定案写「闭包真引到的依赖版本」，等 |
| V6 | G1-g「判法单元指纹多收了算键的库，结果是多作废、不少作废（偏保守）」 | 站得住（超集论证）；「961」这个数撑不起「不少作废」 | `research/prompts/m3-close-gov-b17-r1-main-verification.md:21`；改后 465 个、出自那两份 0 个（961 减 496 恰为 465）；见第 3 节 H5-c |
| V7 | 第二节 G2「改成正推的限定写法」 | 站得住（与第 11 条不再字面冲突、与收工闸不冲突），但「留输出」在判决自己的复跑目录上已经留出了闸会列的东西 | 见 2.3：`m3-close-gov-b17-r1-verify` 里 `opus-work/cleanup/judgement-rerun/repo` 与它的 `target` 被闸的认法判成仓副本与编译目录，目录没有 `.owner` |
| V8 | G3「commit-stages 第 2b 步改四处」 | 站得住 | 四处现文逐项对驱动与清场包装（2.4，out/g3-tokens.txt）；`gpu-driver-error` 的「查备注列」必要：`scripts/mutation/broad-mutation.py:4196` 的明细行读的是 CPU 表 |
| V9 | G3 修补 4 之后的导入那一句 | 站得住 | `.claude/skills/crash-test/commit-stages.md:26` 的导入句逐点对 `scripts/mutation/broad-mutation.py` 的 `plan_gpu_import`（4278）与 `import_results` 的退出分支（4454），见 2.4 |
| V10 | G4`research/prompts/m3-close-gov-b17-r1-main-verification.md:24`：「改成只认本仓那一份（与 `--selftest` 那条一样比 sha256 或比真路径）」 | 站得住；「改法只有一种」不成立 | 判决写了两种，修补 2 取了比真路径：一字不差的拷贝带 `--list-items` 现在被拒（out/unmodified-copy.txt），比 sha256 那一种放得过 |
| V11 | 第四节另登「并进修红一批的文本那一份」（三处） | 够不着（落了两处半，少一处） | 见 2.5：`.claude/agent-common.md:58` 与 `.claude/rules/verification.md:136` 落了；`.claude/hooks/heavy-test-guard.sh` 文件头的类条目（49 行）没有，out/patch-landed.txt 记 `absent=2`；攻方点过的 `.claude/rules/implementation-workflow.md:49` 重型清单判决没列、修补没改 |
| V12 | 回看决策「D13 已定项 17：支撑，2026-10-23 不受影响」 | 够不着 | 定案里「两种排除不进键」那一条的现状句已经与登记位不符（V2），三份语言版都带着（`13-verification-route.en.md` 432、`13-検証ルート.ja.md` 433 行各一处 `HostLookup.query`） |
| V13 | 第一节 1「两腿各自在新旧两版上喂过命令，结果逐行相同」 | 站得住（有限定） | 是腿的自述，旧版（sha256 `51b96bbb…`）不在 git、快照只存了 sha；我用第一轮正推腿的喂命令脚本在今天的钩子上重喂，只有修补 2 要改的那一格变了（第 3 节 H5-a） |
| V14 | 第四节另登「正推一次复跑 hook_feed 的 diff 计数多 1，记未稳，不采」 | 站得住；并给出一个候选原因 | 第一轮正推模型的 h11 那条写的是改名前的旧路径，结局随旧路径在不在而变（退 2 对退 0，out/h11.txt）；时序没证 |

## 2. H1：文本修补与判决第二节每一格

### 2.1 修补 1 每一处新写的片段，现在还在不在主工作区那份文件里

做法：`check_patch_landed.py` 把补丁每个 hunk 的减行与加行各接成一段做字符级比对，取新写的片段（6 字以上，多行的按行拆），在现文里逐字找。原样输出（out/patch-landed.txt）：

    file=.claude/rules/implementation-workflow.md fragments=4 present=4 absent=0
    file=.claude/rules/verification.md fragments=7 present=7 absent=0
    file=.claude/rules/three-way-inference.md fragments=2 present=2 absent=0
    file=.claude/skills/crash-test/commit-stages.md fragments=5 present=5 absent=0
    file=.claude/agent-common.md fragments=1 present=1 absent=0
    file=.claude/hooks/heavy-test-guard.sh fragments=2 present=0 absent=2
      ABSENT: #                 实验复跑入口 scripts/experiment-apparatus/replay.sh 按名字判、不读正文（lib_heavy_tests.py 的 EXPERIMENT_REPL
      ABSENT: #                 只有恰好一个 --list-items（逐行打可点名的实验号，不编、不跑）与登记表里的单独 --selftest 不算

结论：六份文件里五份全落了；`.claude/hooks/heavy-test-guard.sh` 那一处（补丁里加在「全部实验复跑」类条目后面的两行）现文里没有。现文该类条目仍是 `.claude/hooks/heavy-test-guard.sh:49`：「只跑它的单测那一格（--check research-unit-tests）的除外：那一格落到下一类」，后面没有 `scripts/experiment-apparatus/replay.sh`；`git show :.claude/hooks/heavy-test-guard.sh | grep -c '实验复跑入口 scripts/experiment-apparatus/replay.sh 按名字判'` 与 `git show HEAD:…` 都输出 0。现文里有的是另一处：`.claude/hooks/heavy-test-guard.sh:146`：「放行靠参数的三份入口（实验复跑入口 replay.sh」，讲的是副本怎么判，不是这一类重型本身的条目。所以派发提示说的「这一处后来由另一会话经 hook-swap 换上」在现文里是没换上这一半。推翻它的现象：别的文件里（不是 49 行那一类条目）已有「replay.sh 直接起算重型、`--list-items` 与单独 `--selftest` 不算」的整句——`grep -noF 'experiment-apparatus/replay.sh' .claude/hooks/heavy-test-guard.sh` 的命中从 1132 行起（代码段），文件头（前 150 行）里一处也没有。

### 2.2 G1：两处复述与 D13 已定项 17 定案逐片比

定案在 `.claude/kb/decisions/13-验证路线.md:435`（「两种排除不进键」那一条），键的组成在 432 行，「一律不进键」在 433 行；已定项标题在 429 行。复述两处：`.claude/rules/implementation-workflow.md:31` 与 `.claude/rules/verification.md:50`。`check_g1_clauses.py` 逐片在三行里找（Y 在、- 不在）。原样输出（out/g1-clauses.txt）：

    kb_line=435 impl_line=31 verif_line=50
    kb_bullet1_line=432 kb_bullet2_line=433
    clause	kb	impl	verif
    理由不同、分开记	Y	Y	Y
    文件级对象 NOT_JUDGING_FILES 那两份	Y	Y	Y
    算键的库自己	Y	Y	-
    单元算法	Y	-	-
    按角色排	Y	Y	-
    管怎么存、怎么算键，不管判对错	Y	Y	Y
    它们的单元不进任何一份判定的键	Y	Y	Y
    边级定义：只打提示、不改判定结局的子进程边	Y	Y	Y
    按实测排	Y	Y	Y
    不沿它收	Y	Y	Y
    现在只有（现状句）	Y	-	-
    第二台那一路只出提示，按这条边排出键	Y	-	-
    登记位 EXCLUDED_PROGRAM_EDGES	-	Y	Y
    三格自证全句	Y	Y	-
    边级排除要配三格自证	Y	Y	-
    另有一格键不变而退出码变必红	Y	Y	-
    模块级语句：一起收进来	Y	-	-
    模块级语句：一起进闭包	-	Y	Y
    模块级四样清单	Y	Y	Y
    key-clause	kb_bullet1_or_2	impl	verif
    键组成：从 Cargo.lock 解析出的闭包真引到的依赖版本	Y	-	-
    键组成：含 cargo 解析后的生效构建环境与外部工具版本	Y	-	-
    不进键清单：种子串	Y	-	-
    不进键清单：路径（仓根也算）	Y	-	-

读法：
- 第一轮判决 G1 要改的 5 处全对上：边级不再挂「单元不进键」（impl、verif 的「它们的单元不进任何一份判定的键」都只跟在文件级后面），「新加」没了（impl 写「边级排除要配三格自证」），文件级有「管怎么存、怎么算键，不管判对错」，verif 有「不沿它收」与四样清单。登记位指针两处都有。三格自证只在 impl，verif 用指针指过去，符合判决「自证要求只写一处、另一处指过去」。
- 与定案相比规则里仍然少的：「单元算法」（impl 写「算键的库自己与 `scripts/gate/lib_function_units.py`」）、「现在只有……」「第二台那一路只出提示」（这两句是现状与论证，判决要去掉的就是它们）、「收进来」写成「进闭包」。都不改变能做什么。
- 修补多出来、判决五处里没有的：两处都加了「理由不同、分开记」（`.claude/rules/implementation-workflow.md:31`：「算键时有两种排除，理由不同、分开记。」）；verification 加了指针。「理由不同、分开记」在攻方看来是论证（V4）；按 `.claude/singlefs-ai-sop/rules/rules-discipline.md@2. 判据留下，论证搬走`：「删掉这句，读的人少知道什么？说不上来就删」，读的人删掉它还知道文件级与边级两种、各自的判据，我读不出它独有的内容，算多余但无害。「管怎么存、怎么算键，不管判对错」则是文件级「按角色排」的判据（以后要不要排第三个文件，靠它判），留着站得住——这一句上两腿相反，判决只写「采两腿」没回应攻方那一边（V4）。
- 判决没比到的：键的组成。定案 `.claude/kb/decisions/13-验证路线.md:432` 写「从 `Cargo.lock` 解析出的闭包真引到的依赖版本」，两处复述写「解析出的依赖版本」（`.claude/rules/implementation-workflow.md:31`、`.claude/rules/verification.md:50`，35 行同），没有「闭包真引到的」；而 `.claude/rules/verification.md:37` 写「`Cargo.lock` 里闭包真引到的外部包」。同一份规则里两种写法。读规则的人会以为 Cargo.lock 任何依赖变了都作废；偏保守，不致做错，但与定案不是同一句。定案 433 行「不进键」清单里的「种子串」，两处复述的清单（verification 50 行）也没有。这是「逐项比」没比到的地方，不是修补 1 引入的（这些字在补丁前就是这样）。
- 一个与修补无关、但会影响「回看决策」的事实见 2.6（V12）。

### 2.3 G2：限定写法与第 11 条、收工闸

现文：`.claude/rules/three-way-inference.md:127`：「复跑那一步做完，副本里的仓副本与编译目录当场删；复跑的输出与模型目录拷贝留到这一题的下一轮判决写完、或这一题不再开下一轮时再删」。第 11 条：`.claude/main-agent.md:30`：「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用」，它点名的三样是「本会话建的仓副本、编译目录与运行目录里的库拷贝」。
- 与第 11 条：留下的两样（输出、模型目录拷贝）不在第 11 条点名的三样里，仓副本与编译目录按第 11 条当场删，字面不再顶牛。站得住。
- 与收工闸：闸认的是仓副本、编译目录、库拷贝（`.claude/hooks/completion-cleanup-guard.sh:40`：「仓副本：有 .git（目录；.git 是文件的报成工作树），或同时有 Cargo.toml 与 crates」），声明了也只免仓副本与库拷贝（`.claude/hooks/completion-cleanup-guard.sh:49`：「声明了的那一棵里仓副本与库拷贝不列，编译目录照列」），扫到 `.claude/hooks/completion-cleanup-guard.sh:213`：「SCAN_MAX_DEPTH = 6」层。留输出与模型目录拷贝本身闸不列。数过：`research/prompts` 下 `*-model` 目录 529 个（含这一腿自己的），其中有 `Cargo.toml` 与 `crates` 并排的 0 个，带 `.git`、`CACHEDIR.TAG`、`.rustc_info.json`、`MANIFEST-*` 的 0 个（命令：`find . -path '*-model/*' -name Cargo.toml -printf '%h\n' | while read -r d; do [ -d "$d/crates" ] && echo "BOTH $d"; done | wc -l` 输出 0；`find . -path '*-model*' \( -name .git -o -name CACHEDIR.TAG -o -name .rustc_info.json -o -name 'MANIFEST-*' \) | wc -l` 输出 0）。所以按文字留「输出 + 模型目录拷贝」，闸不会拦。站得住。
- 但「复跑的输出」不是永远干净：第一轮判决自己的复跑目录 `~/.local/share/singlefs/work/m3-close-gov-b17-r1-verify` 里，攻方模型自己造的空壳还在。命令与原样输出：`cd ~/.local/share/singlefs/work/m3-close-gov-b17-r1-verify && find . -name Cargo.toml -o -name crates -o -name CACHEDIR.TAG` 输出 `./opus-work/cleanup/judgement-rerun/repo/crates`、`./opus-work/cleanup/judgement-rerun/repo/Cargo.toml`、`./opus-work/cleanup/judgement-rerun/repo/target/CACHEDIR.TAG`；用闸导入的认法函数（`scripts/agents/agent-watch.py` 的 `scratch_hog_kind`）逐层判，得 `[('opus-work/cleanup/judgement-rerun/repo', '仓副本'), ('opus-work/cleanup/judgement-rerun/repo/target', '编译目录')]`；`ls -A` 该目录只有 `opus-model opus-work sonnet-model sonnet-work`，没有 `.owner`。按修补 1 的话，这两样「当场删」，现在还在；闸没拦是因为目录没认领，不是因为不算。含义：「留输出」的那半句，遇到模型自己在 `--work` 里造仓形状的东西（第一轮攻方模型就是）时，要靠「当场删」那半句兜住，两半句得一起读；判决写时没提这一点。推翻它的现象：该目录被认领过又被闸放行（有 `.deliverables` 声明——但编译目录声明了也列）。
- 跨会话（攻方 G2-b，`research/prompts/m3-close-gov-b17-r1-opus-output.md`）：留下的输出与模型目录拷贝没有人规定谁在下一轮删；这一点修补 1 没覆盖，但它们不被闸列、也不占大空间（该目录 `du -sh` 412K），代价小。

### 2.4 G3：第 2b 步现文对驱动与清场包装（修补 4 之后）

`check_g3_tokens.py` 数第 2b 步（`.claude/skills/crash-test/commit-stages.md` 23–28 行）点名的标识在 `scripts/mutation/broad-mutation.py`、`scripts/crash/gpu-quiesce-run.sh`、`scripts/crash/crash-amplification-lib.sh` 里各出现几次（out/g3-tokens.txt，40 行，这里摘要，全表在文件里）：`BROAD_MUTATION_GPU_NOT_RUN/PLAN/ITEM/TOTAL`、`IMPORT_GPU_REJECTED`、`NOT_RUN_NO_GPU_ADAPTER`、`--skip-gpu`、`--shard`、`--import-results`、`gpu-verdicts.tsv`、`gpu_needed/gpu_reused/gpu_not_run/gpu_without_conclusion`、`gpu-not-run`、`gpu-timeout`、`广谱变异 GPU 趟`、`gpu:`、`FLIPPED`、`escalated-verdicts.tsv` 在驱动里都有；`GPU_QUIESCE_STATE_DIRECTORY`、`GPU_QUIESCE_LOG_DIRECTORY`、`QUIESCE_STOP_COMMAND`、`QUIESCE_STOPPED_CHECK_COMMAND`、`QUIESCE_START_COMMAND`、`QUIESCE_STARTED_CHECK_COMMAND` 在清场包装里都有；`CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS` 在驱动与 `crash-amplification-lib.sh` 里都有。`gpu-memory-cap`、`gpu-driver-error` 在驱动里字面 0 次，是因为理由串是拼出来的：`scripts/mutation/broad-mutation.py:3500` 附近 `f"gpu-{gpu.state}"`（状态取 `timeout`、`memory-cap`、`driver-error`），不是缺。

判决要的四处：
- `gpu-driver-error` 的出路：现文 `.claude/skills/crash-test/commit-stages.md:28`：「理由写 `gpu-driver-error` 的是 GPU 趟驱动错（不是这条变异的结局，加大限时与上限不对症）」，来历三种（编译步退 101 而日志里没有带源码位置的编译错误、测试步退 0 而一个测试都没跑到、编译步或测试步退了认不出的退出码），驱动里对应的三句话各出现 4、12、14 次（out/g3-tokens.txt 末段）。「查库目录 `gpu-verdicts.tsv` 里那条最后一行的备注列」这句必要且对：`scripts/mutation/broad-mutation.py:4196` 的 DRIVER_ERROR 明细行读的是 `judgement.by_key`（CPU 表），GPU 趟的驱动错在那里读出空备注；备注列在 GPU 表的第 13 段（`scripts/mutation/broad-mutation.py:88`：「十四段：键、序号、包、变异名、结局、抓到它的测试、测试键、覆盖者键、GPU 趟自己的键、测试秒、判定日期、跑法、备注、校验列」）。站得住。
- 清场退 78：现文 `.claude/skills/crash-test/commit-stages.md:26` 写了四种（卡上有不归白名单的进程；`QUIESCE_STOP_COMMAND`、`QUIESCE_STOPPED_CHECK_COMMAND` 没写全；状态目录或本机日志建不了，设 `GPU_QUIESCE_STATE_DIRECTORY`、`GPU_QUIESCE_LOG_DIRECTORY`；别的经清场包装的一趟卡在清场命令上，等它结束再起）。清场包装自己列的是六种（`scripts/crash/gpu-quiesce-run.sh:75`：「78        命令没起：多机配置里 QUIESCE_STOP_COMMAND、QUIESCE_STOPPED_CHECK_COMMAND 没写全」起，到「SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE 写了 local、peer 之外的值」），规则没写的两种是「第二台上起活那一方没清场」与角色变量写错，都是第二台那一路；规则的引子「出路照它打的「→ 怎么办」」把没列的交给包装自己的提示。第四种（锁等不到）我是从 `scripts/crash/gpu-quiesce-run.sh:43`：「进出都在同一把 flock 锁里」推的，没起清场验过：这一点是推的。站得住，但「四种」不是全集。
- 哨兵红退 1：`scripts/mutation/broad-mutation.py:4763` 与 `:4812` 的 `if gpu_totals.failed:` 后面都是 `return 1`；哨兵标签 `scripts/mutation/broad-mutation.py:3870` 是「广谱变异 GPU 趟」，消息头 `scripts/gate/lib_reuse_checks.py:271` 拼成「复用哨兵（广谱变异 GPU 趟）」，项名 `gpu:<序号>` 在 3872 行附近建（`out/g3-tokens.txt` 里 `gpu:` 在驱动 15 次）。现文 28 行「退 1 里有一种是 GPU 那一趟的复用哨兵判红（stderr 的「✗」那一行带「广谱变异 GPU 趟」，项名 `gpu:<序号>`）」对得上。站得住。
- 两台分片「在有卡的机器上补」：26 行与 28 行各一处（out 里 `--shard` 在 23–28 行 2 次），驱动的提示同样写「要在有卡的机器上跑」（`scripts/mutation/broad-mutation.py:4246`）。站得住。

修补 4 之后的导入句（26 行）：`verdicts.tsv` 与 `gpu-verdicts.tsv` 都并；GPU 行逐行核键（行的键在本机此刻清单的单元键里、`gpu_key` 等于本机此刻 GPU 趟自己的键）——对 `scripts/mutation/broad-mutation.py:4278`（`plan_gpu_import`）里的 `row["key"] not in current_keys`（理由 `key-not-in-list`）与 `row["gpu_key"] != pass_key`（`gpu-pass-changed`）；「核不过的不并、逐条列出（`IMPORT_GPU_REJECTED`，导入退 1，别的行已并进来）」对 `scripts/mutation/broad-mutation.py:4454`：`if conflicts or gpu_plan.rejected or gpu_plan.conflicts:` 之后 `return EXIT_INCONSISTENT`（`scripts/mutation/broad-mutation.py:631` 的赋值里 `EXIT_INCONSISTENT` 取值 1），且 `library.record_verdicts`、`record_gpu_verdicts` 在这之前已写（别的行已并）；「键相同而定论不同的不并、逐条列出」对 `IMPORT_GPU_CONFLICT`。站得住。小出入两处，都是判决当时就有、没被采的：
- 带 `--skip-stage-two` 时 GPU 趟不起（`scripts/mutation/broad-mutation.py:3938`：「GPU 趟接在第二段后面，这一趟带了 --skip-stage-two，不跑」），28 行列 `gpu-not-run` 的三种来历（没卡、清场没过、带了 `--skip-gpu`）里没有它（out/g3-tokens.txt 末行 `skip-stage-two-in-commit-stages-23_28 0`）。攻方第一轮提过（`research/prompts/m3-close-gov-b17-r1-opus-output.md` G3 表「没卡 / 清场没过 / --skip-gpu」那一行），判决没回应。够不着，小。
- 导入前提「本机已记过清单」只写了一半：缺清单时驱动 refuse 并点名 `--refresh-list`（驱动里 17 次），第 2b 步 23–28 行里 `--refresh-list` 0 次。小。

### 2.5 agent-common 那一行、并行的重型清单、定义级改动的手续

- `.claude/agent-common.md:58` 现文：「与入口 `scripts/experiment-apparatus/replay.sh` 的不带参数、点名实验号、`--admission-only`；`--list-items` 与登记为 light 的单独 `--selftest` 不算」，`.claude/rules/verification.md:136` 同一句。现判法 `.claude/hooks/lib_heavy_tests.py:621`：「不算的只有摘掉 --force 之后恰好一个 --list-items 的」。两边不矛盾：agent-common 的「不算」没写「恰好一个」「仅仓内那一份」，但 `--list-items` 带别的词时脚本自己退 2（第一轮正推读 `scripts/experiment-apparatus/replay.sh` 得的结论，`research/prompts/m3-close-gov-b17-r1-sonnet-output.md` 5.1 节；我没再跑），读错了也跑不起来。与 `.claude/rules/implementation-workflow.md` 那一节（37 行起）也不矛盾：该节表里「子 agent 只跑 harness 与形式化那一条轻型」，replay.sh 直接起算重型与它一致。
- 并行清单不齐：同一批「重型是……」的清单有三份（agent-common 58 行、verification 136 行、implementation-workflow 49 行）。命令：`for f in … ; do awk 'NR<=200' $f | grep -cF 'experiment-apparatus/replay.sh'; done` 得 agent-common 2、verification 1、implementation-workflow 0、heavy-test-guard.sh 前 200 行 0。判决第四节另登点了三处（verification、agent-common、heavy-test-guard 文件头），没点 `.claude/rules/implementation-workflow.md:49`——攻方第一轮报告点过（`research/prompts/m3-close-gov-b17-r1-opus-output.md:273`：「`.claude/rules/implementation-workflow.md:49`」）。现状：修补落了两份，implementation-workflow 那一份是「重型测试」一节的权威清单（`.claude/rules/implementation-workflow.md:49` 以「**重型测试**就是 checker 档那一批」起头），反而没有；heavy-test-guard 文件头那一份在 2.1 里已说是没落地。
- 手续：`.claude/agent-common.md` 是定义级文件。现文 sha256 `67f0fc7f…` 不在 `.claude/agent-def-review-exempt`（`grep -c` 输出 0）；第一轮判决没有「## 这一轮被判过的文件」一节（`grep -c '^## 这一轮被判过的文件'` 输出 0）。门禁格 agent-def-adversarial-review（`.claude/gate.d/doc-process-records.sh:20`）按它文件头的判据，应当要求判决在那一节点名这份文件；这一轮的判决补上就行。这一条是照门禁文件头推的，我没跑门禁（子 agent 不跑）。

### 2.6 回看决策与 D13 已定项 17

`.claude/kb/decisions/13-验证路线.md:435`：「边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；现在只有内存包装 `HostLookup.query` 起 `scripts/hosts/host-resources.py` 那一条」。今天登记位：`scripts/gate/lib_function_units.py:7154`：「EXCLUDED_PROGRAM_EDGES = {」，`scripts/gate/lib_function_units.py:77`：「今天三条，都在内存包装里」；`edges_and_units.py` 的输出（out/edges-and-units.txt）：

    excluded_program_edges=3
      scripts/hosts/run-with-memory-cap.sh :: HostLookup.query -> scripts/hosts/host-resources.py
      scripts/hosts/run-with-memory-cap.sh :: HostLookup.watch_events -> scripts/hosts/resource_release_broadcast.py
      scripts/hosts/run-with-memory-cap.sh :: broadcast_memory_released -> scripts/hosts/resource_release_broadcast.py
    crates_judging_units total=465 from_not_judging_files=0

第一轮攻方当时量到的是一条（`research/prompts/m3-close-gov-b17-r1-opus-output.md` 引 `(source, unit_name, target), = EXCLUDED_PROGRAM_EDGES` 的解包写法），第一轮开工快照记的 `lib_function_units.py` sha256 是 `9cdb5122…`，今天是 `5fc1e2f4…`（`research/prompts/m3-close-gov-b17-r1-snapshot/judged-sha256.txt` 与 `research/prompts/m3-close-gov-b17-r2-snapshot/judged-sha256.txt`）：两轮开工快照之间，登记位从一条变成三条。所以：判决把规则里的「现在只有那一条」换成指登记位是对的（站得住，且被事实验证）；但判决「回看决策」写 D13 已定项 17「2026-10-23 不受影响」（`research/prompts/m3-close-gov-b17-r1-main-verification.md:55`）够不着：定案里「两种排除不进键」那一条本身带着已经不真的现状句，中文、英文（`.claude/kb/decisions/13-verification-route.en.md` 432 行：「today the only one is」）、日文（`.claude/kb/decisions/13-検証ルート.ja.md` 433 行：「エッジ一本だけ」）三份都带。这属 kb-scribe 的活，不在这一轮修补里；我不改。推翻它的现象：定案另一处把这句限定成「写定案那一刻」。

### 2.7 G4：判决写了两种改法，修补取了一种

判决：`research/prompts/m3-close-gov-b17-r1-main-verification.md:24`：「改成只认本仓那一份（与 `--selftest` 那条一样比 sha256 或比真路径）」。修补 2 取的是比真路径：`.claude/hooks/lib_heavy_tests.py:780`：「OWN_COPY_ONLY_SCRIPTS = frozenset({"replay.sh", "crash-point-replay.sh", "crash-amplification.sh"})」，`.claude/hooks/lib_heavy_tests.py:3368` 起 `own_repository_script`（比 `os.path.realpath`）。同一道闸里已有的先例两种都有：真实路径（`stage_selftest_is_own_copy`，`own_repository_script` 的注释写「判法与 stage_selftest_is_own_copy 同一种」）与 sha256（`.claude/hooks/heavy-test-guard.sh:99`：「副本里被起的那份脚本与主仓（钩子所在仓）那一份 sha256 相同」，工作根底下仓副本里的单独 `--selftest` 放行的条件之一）。同一道闸对 `--selftest` 的做法是：工作根底下的一字不差的副本放行、改过正文的拒；修补 2 对 `--list-items` 没有对应的放行。

我造了一个一字不差的拷贝（`cmp` 无差异）放在路径尾部相同的目录里，喂真钩子（子 agent 身份），原样输出（out/unmodified-copy.txt）：

    copy_identical_to_repo_file=True
    本仓那一份 --list-items	exit=0	
    没改过的拷贝（路径尾部相同）--list-items	exit=2	✗ 重型测试被拒：脚本 <scratch>/copy/scripts/experiment-apparatus/replay.sh:760 里的 scripts/experiment-apparatus/replay.sh ${@:2} E1（实验复跑：每个实验起装置或驱动；只有仓内那一份摘掉 --force 之后恰好一个 --list-
    没改过的拷贝 cd 进去相对路径 --list-items	exit=2	✗ 重型测试被拒：脚本 <scratch>/copy/scripts/experiment-apparatus/replay.sh:760 里的 scripts/experiment-apparatus/replay.sh ${@:2} E1（实验复跑：每个实验起装置或驱动；只有仓内那一份摘掉 --force 之后恰好一个 --list-

判：口子补上了（第一轮的改坏副本现在被拒，见 3.1 的 h04），代价是一字不差的拷贝带 `--list-items` 也被拒：钩子读正文，正文第 760 行本身就写着起 replay.sh 的 `${@:2}`，静态读一定命中。判决写的「比 sha256」那一种能放得过这类拷贝并仍拦住改过的。所以「判决要的改法是不是只有这一种」——判决自己写了两种，修补取了更严的一种，不是只有一种。站得住，是取舍：更严一侧误拒的是「子 agent 在仓副本里自查 `--list-items`」，出路在拒绝说明里写的「只有仓内那一份」；放宽一侧（sha256）放过的是「副本与本仓逐字节相同」，后者与本仓那一份行为一样。推翻「更严的一种够用」的现象：子 agent 的常见流程里需要在仓副本里跑 `replay.sh --list-items`（这一点我没量过频率）。

同形的别的名字：`OWN_COPY_ONLY_SCRIPTS` 三个名字（`replay.sh`、`crash-point-replay.sh`、`crash-amplification.sh`）同一个判法，补丁一并改了，`.claude/hooks/lib_heavy_tests.py` 的自证在 4947–4950 行附近各有一格（本仓那一份、`replay.sh` 别处拷贝、`crash-point-replay.sh` 别处拷贝、`crash-amplification.sh` 别处拷贝）。`KNOWN_SCRIPT_LOCATIONS`（`.claude/hooks/lib_heavy_tests.py:764`）里别的靠参数放行的名字有没有同一个口子，是 H2 的问题，我没查。

## 3. H5：第一轮判决得出结论的方法

### 3.1 H5-a：「新旧两版钩子喂命令结果相同」

判决说法：`research/prompts/m3-close-gov-b17-r1-main-verification.md:7`：「两腿各自在新旧两版上喂过 `replay.sh` 相关的命令，结果逐行相同」。三点：

1. 这是腿的自述，判决没自己核。旧版 `lib_heavy_tests.py`（sha256 `51b96bbb…`，来自 `research/prompts/m3-close-gov-b17-r1-snapshot/judged-sha256.txt`）不在 git 里：`for c in $(git log --all --format=%h -8 -- .claude/hooks/lib_heavy_tests.py); do git show $c:.claude/hooks/lib_heavy_tests.py | sha256sum | cut -c1-12; done` 输出 `b85bb92c4830` 四次、`986158b115b4`、`2eb84651b55f`、`cd62cae81ca6`、`48a69aafd7c0`，暂存区 `df5acf9615bb`，没有 `51b96bbb`。开工快照目录只存了 sha256 与 diff，没存那一版的正文。所以「旧版上结果相同」今天谁都复现不了。
2. 但这句话只用来支撑「腿的观察不依赖钩子被换过」，判决的结论（G4 采）不靠它：判决五里主 agent 在当时的钩子上复跑了两个模型都逐字相同（`research/prompts/m3-close-gov-b17-r1-main-verification.md:49`、`:48`）。
3. 我用能做的代替：第一轮正推腿的喂命令脚本（`feed.py` 在 `classify` 层喂 46 条；`hook_feed.py` 按 PreToolUse JSON 喂真钩子 19 条）在今天又换过一版的钩子上重喂，与它当时的 `out/` 按格比（`h5_compare.py`，out/h5-compare.txt 原样）：

       hook_feed cells old=19 new=19
       hook_feed h04: exit 0 -> 2（want=0）
       hook_feed exit_changed_cells=1
       feed lines old=49 new=49 identical=True

   腿的 `out/` 是在 `b1c3ef8d…` 那一版上生成的（`research/prompts/m3-close-gov-b17-r1-sonnet-output.md` 附），今天是 `df5acf96…`，我直接比的是这两版：`classify` 层 49 行逐字相同，真钩子 19 格只有 `h04`（路径尾部相同的改坏副本）变，且正是修补 2 要改的那一格。作为方法：站得住，条件是两份要合起来看——`feed.py` 在 `classify` 层，对 G4-a 的口子是盲的（那一层对路径尾部相同的拷贝一直返回放行，所以 49 行不变），能分辨的是 `hook_feed.py` 的 h04。判决写「结果逐行相同」时若只看 `feed.txt`，会得出口子没被修的错觉。推翻它的现象：今天的钩子上 `hook_feed.py` 的另一格（不是 h04）退出码与第一轮 `out/` 不同。

   小提示：`hook_feed.py` 里 h06 的拒绝说明行号随 `replay.sh` 变（745 对 760），所以比对只能按格号加退出码，不能整行。

### 3.2 H5-b：正推一次复跑计数多 1，记「未稳」，不采

判决：`research/prompts/m3-close-gov-b17-r1-main-verification.md:41`：「主 agent 复跑一趟逐字相同」；腿自述：`research/prompts/m3-close-gov-b17-r1-sonnet-output.md:20`：「原因没查到」。做法站得住：一次不稳加五次一致，不采不稳那一次当证据；判决里 G4 的证据格（h04、h05、c01）都在稳的那几趟里。

我多做了一步，找候选原因：`hook_feed.py` 的 h11 写的是改名前的旧路径（`research/prompts/m3-close-gov-b17-r1-sonnet-model/hook_feed.py:27`：「bash scripts/experiments/replay.sh --list-items」，期望退 0），而这个目录在同一天、腿开跑之后被整个改名（`research/prompts/m3-close-gov-b17-r1-main-verification.md:8`：「正文与背景材料在腿开跑之后改过一次路径」；`.claude/gate.d/path-moves.tsv:6`）。h11 的结局取决于当前目录里旧路径在不在，造两个当前目录各喂一次（`probe_h11.py`，out/h11.txt 原样）：

    old-path-present	exit=2	✗ 重型测试被拒：脚本 <scratch>/with-old-path/scripts/experiments/replay.sh:760 里的 scripts/experiment-apparatus/replay.sh ${@:2} E1（实验复跑：每个实验起装置或驱动；只有仓内那一份摘掉 --
    old-path-absent	exit=0	! heavy-test-guard.sh：bash / sh 起的脚本 <scratch>/without-old-path/scripts/experiments/replay.sh 不存在，里面的命令没判；这条命令照常执行，记了一条检出

旧路径在时 h11 退 2（期望 0，计数 +1），不在时退 0。方向复现了「diff 计数多 1」；时序没证（我没看到腿第一趟复跑时旧路径在不在，腿也没留当时的差异行），所以只是候选。含义：「diff=」是聚合计数，定位不了哪一格动了；比对按格号加退出码（3.1）才能认出。推翻候选的现象：腿第一趟复跑时旧路径已经不在、而 diff 仍多 1。

### 3.3 H5-c：用 961 个单元数推「多作废不少作废」

判决：`research/prompts/m3-close-gov-b17-r1-main-verification.md:21`：「结果是多作废、不少作废（偏保守）」。数的出处：攻方 `research/prompts/m3-close-gov-b17-r1-opus-output.md` 的首行 `0ad75a2c… 961` 与按文件数的 `496 scripts/gate/lib_function_units.py`。
- 数能撑的：「`lib_function_units.py` 的单元进了判法单元指纹」这个事实（496 个出自它，是直接的清单归属，不是推的）。
- 数撑不起的：「不少作废」。这一半靠的是集合论证——指纹是闭包里单元（种类、指纹）的多重集（`scripts/mutation/lib_mutation_units.py` 里 `judging_units` 的文档字符串），多收的单元只会让更多改动改变指纹，所以对「该作废的改动」不会漏；这个论证成立，但它不依赖 961 这个数，而依赖「该收的单元都在 961 里」，这一半没人核。
- 数本身不稳：判决五里复跑 961 变 966（`crates-mutation-rows.py` 222 变 227），今天是 465；我今天的清单：`python3 scripts/mutation/crates-mutation-rows.py judging-units <仓根> --units` 首行 `c63379ff285eba00ca251e038844715f9057189fb3fca205c2985172c9d0dccc 465`，出自 `lib_function_units.py` 与 `lib_verdict_library.py` 的 0 个（out/edges-and-units.txt 末行 `crates_judging_units total=465 from_not_judging_files=0`）。961 减 496 恰为 465，与修补 3 去掉的正是 `lib_function_units.py` 那 496 个对得上（两个数是不同时刻量的，`lib_verdict_library.py` 在改前清单里是不是 0 个，我没单独量，推的）。
- 「偏保守」没算代价：改前每改一次 `lib_function_units.py`（一份一万多行的文件，开工快照里 sha256 在两轮之间已变过一次）就让 crates 变异整表的每一行判法单元指纹变、整表重判。这是修补 3 该做的理由，判决的措辞把它说轻了。
- 修补 3 之后方向翻转：从「多收」变成「恰好等于定案」，「少作废」那一侧（`lib_function_units.py` 在 crates 变异判法里除了算键还干什么）是 H3 的问题，我没查。

站得住的是「多收」；「不少作废」在当时是对的论证但不是 961 撑的。推翻它的现象：改前的指纹有一条路径让「该收的单元」没收进 961（那时「不少作废」就不成立）。

## 4. 替第一轮被判「打中」的原文辩（单列）

辩的是被改掉之前的原文：它站得住的理由是什么、辩不住的是什么。强度：强 = 判决不该采；中 = 采了没错但理由该写准；弱 = 辩不住。

| 编号 | 被判打中的原文 | 辩词 | 强度 | 推翻辩词的现象 |
|---|---|---|---|---|
| D1 | 边级排除也写「单元不进任何一份判定的键」（G1 多） | 对「只经这条边可达」的那些单元它是真的：自证第三格要求的恰是切边前后少掉的单元只经这条边可达，且自证把登记的几条边一起当根收、与不切的逐个相同（`scripts/gate/lib_function_units.py:80`：「把被起的程序都当另几个根一起收，与不切的逐个相同」）。错的只是对「别的路也能到」的单元，定案用「不沿它收」说的就是这个区别。所以这是照搬定案的问题，没有量到行为错 | 中 | 找到一份判定，它的闭包经别的路径到 `host-resources.py` 或 `resource_release_broadcast.py` 的单元，且旧写法让人漏算（攻方第一轮只查了 crates 变异清单，0 个） |
| D2 | 「新加边级排除配三格自证」（G1 多，「新加」） | 实现里三格自证是对登记的全部边循环做的（`scripts/gate/lib_function_units.py:9874`：「targets = sorted({target for _source, _unit_name, target in EXCLUDED_PROGRAM_EDGES})」），新边自动被覆盖，所以「新加的才配」与「边级排除要配」在代码里等价；去掉「新加」无害，但不是必须 | 弱（改了无害） | 自证里有一格只对登记的某一条边写死 |
| D3 | 「现在只有……那一条」（G1 现状句） | 写它时是真的（第一轮开工时登记位是一条）；规则里写具体边也让读者不用翻代码就认得是哪条。辩不住：规则纪律把「什么规则在哪里：指到别的规则、脚本或 kb」放在能写的一栏（`.claude/singlefs-ai-sop/rules/rules-discipline.md:16`），而事实已经三条（V2） | 弱 | — |
| D4 | verification 没抄模块级四样清单（攻方 G1-e：不致错） | 攻方的辩词在代码上成立：`scripts/gate/lib_function_units.py:72`：「其余模块级语句在 module-body 里，本来就连」，四样清单列的是 import-time 规则名收的那几类，不是闭包里能有的全部模块级语句。修补把清单抄进 verification 后，读规则的人更容易把它读成穷举 | 中（采正推没错，因为与定案同字；但「穷举」的读法没人挡） | 读规则的人因为清单做错了事（我没找到实例） |
| D5 | 复述里的「理由不同、分开记」「管怎么存、怎么算键，不管判对错」（攻方 G1-c 判为论证、该留在 kb） | 后一句是文件级「按角色排」的判据，删掉读的人就不知道第三个文件按什么判，应留（正推对）；前一句删掉不少信息，应删（攻方对）。判决只写「采两腿」，修补两句都抄了 | 中 | 规则纪律门禁（rules-lint）对这两句判红（我没跑） |
| D6 | 「用完」指这一步、副本留到下一轮判决写完（G2 原文） | 想省的是复跑副本的重拷与重编。辩不住：留的若是仓副本，量级是 6.7G 一份（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:291`：「r1 判决复跑目录 6.7G」），且与第 11 条「不等一批收尾」字面冲突；修补把留的缩成不被闸列的两样是对的，但判决复跑目录里模型自己造的仓形状东西要靠「当场删」兜（2.3） | 弱 | — |
| D7 | `gpu-driver-error` 套进「加大限时 / 上限」（G3） | 辩不住：来历三种都与限时、内存无关，超时、内存撞顶另有 `gpu-timeout`、`gpu-memory-cap`（`scripts/mutation/broad-mutation.py` 的 `f"gpu-{gpu.state}"` 拼法） | 弱 | — |
| D8 | 路径尾部相同的拷贝按名字判（G4 原判法） | 此前已与崩溃点重放入口同形（`.claude/hooks/lib_heavy_tests.py:3360`：「只看路径尾部，不核是不是本仓那一份」是 `known_script_at_location` 的文档字符串），漏的前提是有人改了拷贝里的 `--list-items` 一支又在拷贝里自查，门禁意图是挡无心之失；而且改正的代价是一字不差的拷贝也被拒（2.7）。辩不住的：文件头自己写了「别处同名拷贝读正文」，实现与字面不符；攻方给了起 cargo 的实例 | 中（采没错；但判决没写更严一侧的误拒代价） | 子 agent 流程里在仓副本跑 `replay.sh --list-items` 很常见（没量） |

## 5. 什么现象会推翻本报告的主要结论

- V2、V12：`scripts/gate/lib_function_units.py` 的 `EXCLUDED_PROGRAM_EDGES` 被缩回一条，或定案另处限定了「现在只有」的时点。
- V11：`.claude/hooks/heavy-test-guard.sh` 文件头别处（不是 49 行那一类条目）有「replay.sh 直接起算重型、`--list-items` 与单独 `--selftest` 不算」的整句；`grep -noF 'experiment-apparatus/replay.sh'` 在前 150 行命中就推翻。
- V7：`m3-close-gov-b17-r1-verify` 被认领过、或 `scratch_hog_kind` 在闸里对该目录不判；两样都能用 2.3 的命令现查。
- V10：子 agent 在仓副本里跑 `replay.sh --list-items` 在实际流程里几乎不发生，则「更严一种」的误拒不构成代价。
- V13、V14：腿第一趟复跑时旧路径 `scripts/experiments/` 已不在而 diff 仍多 1，则 h11 不是原因；今天的钩子上 `hook_feed.py` 除 h04 外还有格变，则「只有修补 2 要改的那一格变」不成立。
- 所有「站得住」：模型 out/ 在复跑时不再逐字相同，且 `judged-sha256.txt` 那一份集合没变——那就是我的脚本在抖。

## 6. 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；V1–V14 的「判定」是对「判决这一格站不站得住」的复核，不是采纳。
- H2（`own_repository_script` 对相对路径、符号链接、`source`、`bash -c` 包装、`--staged` 临时树的认法）、H3（变异判法排除后「少作废」的路径）、H4（GPU 判定导回的核键）没查，不归这一条腿；2.7 的误拒实测和 2.4 的导入句对照只是为 H1 的「改法是不是只有一种」「文本对不对得上驱动」做的，不替那几格下结论。
- 没跑 `heavy-test-guard.sh`、`lib_heavy_tests.py`、`broad-mutation.py` 的任何 `--selftest`，没跑门禁阶段与 `gate.sh`，没编译，没起重型；喂真钩子的几次都在草稿目录里造了假拷贝，只读主工作区，检出记录经 `AGENT_HOOK_DETECTIONS` 写进草稿目录。
- 没核 2.4 里「锁等不到」就是「别的经清场包装的一趟卡在清场命令上」（读源码推的，没起清场）。
- 没核三语 kb 译文里除 `HostLookup.query` 那一句之外的差别；没读 `.claude/kb/decisions-history.md` 里的来历。
- 没把 `m3-close-gov-b17-r1-verify` 里的东西删掉（不归我；它是主 agent 的复跑目录，只读了内容）。
- 没读禁读清单里的文件（`research/prompts/m3-close-gov-b17-r2-opus-output.md`、`research/prompts/m3-close-gov-b17-r2-opus-model/`）。
- 门禁 agent-def-adversarial-review 是否会因 `.claude/agent-common.md` 红，是照文件头推的，没跑。

## 7. 清理与 cite-check

- 这一腿没建仓副本、编译目录或库拷贝。草稿目录 `~/.local/share/singlefs/work/m3-close-gov-b17-r2-sonnet`：删前 `du -sh` 2.1M（复跑输出、假拷贝、日志），删后 16K（只剩 `.owner`、`.writer-lock` 与一份辅助脚本 `chardiff.py`）。没有后台进程。
- 交回前最后一趟复跑（退 0，八行 `result=identical`、0 行 `DIFFERENT`）后，复跑目录已删。
- `python3 scripts/agents/cite-check.py research/prompts/m3-close-gov-b17-r2-sonnet-output.md` 末行：

      ✓ 核了 40 处引文、39 处裸行号，对上 40 处，没判 39 处（其中裸行号没有引文可比 36 处；逐处列在上面）

  没判的 39 处里 36 处是裸行号（我逐个 `grep -nF` 过行号所在的文字，见各节），3 处是代码块里 `<scratch>` 路径换名后的输出行（路径认不出，原样输出，不是引文）。
