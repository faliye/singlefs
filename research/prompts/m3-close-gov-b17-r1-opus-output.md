# m3-close-gov-b17-r1 攻方腿（Opus）：批 17 治理文件改动的反例

日期：2026-10-23。攻击面：正文第三节 G1–G4。被判对象是快照目录 `research/prompts/m3-close-gov-b17-r1-snapshot/` 的两份 diff；kb 与被判文件的 sha256 开工时 `sha256sum -c` 全 OK。开工之后别的会话改了 `.claude/hooks/lib_heavy_tests.py` 与 `.claude/hooks/heavy-test-guard.sh`（见 G4 开头），其余被判文件到交回时没变。

## 复跑

模型目录 `research/prompts/m3-close-gov-b17-r1-opus-model/`，复跑命令（工作目录随意）：

```
bash scripts/hosts/run-with-memory-cap.sh 2G bash research/prompts/m3-close-gov-b17-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen <仓根>/research/prompts/m3-close-gov-b17-r1-snapshot --work <新的草稿目录>
```

跑完看前三行：`name=probe_same_as_model lines=27`、`name=cleanup_same_as_model lines=5`、`name=crates_judging_units_same_as_model`。三样都只调判法函数与只读脚本，不起 cargo、不起被判的命令。`case=joined-semicolon-cargo` 那一行的 `stderr=` 取决于此刻主工作区合入锁有没有被占，复跑时可能不同，`lib=` 与 `guard_exit=` 两栏不变。我这一趟经内存包装起时排队超过 230 秒没起来（`ps` 看到别的会话两条包装各占 40G、24G），之后不经包装直接起（全是 python，不跑编译出来的代码），rerun-2 到 rerun-5 四趟同样结果。

```
90acd951ad8c7992dc86c17545644fb26679b70a283f7f785c94b418be544b48  research/prompts/m3-close-gov-b17-r1-opus-model/out/cleanup-output.txt
c26d96e22cbe544df9f6e92ad2ade822415af0bb5d0ffcd9a8fbea85f879c933  research/prompts/m3-close-gov-b17-r1-opus-model/out/crates-judging-units-by-file.txt
7c29074d00c3d6ac0bbee04e527c04091ccafce68709b97893b650890486d577  research/prompts/m3-close-gov-b17-r1-opus-model/out/probe-output.txt
6ec05983bee1bf7cba4636329e40ebd7f1d7cbfe56e778f4614bd71094e15b23  research/prompts/m3-close-gov-b17-r1-opus-model/probe_cleanup_kind.py
9e8eb7358646b6c795046b684587ffad965bffe369bb6ee38d30d7d83293f130  research/prompts/m3-close-gov-b17-r1-opus-model/probe_heavy_guard.py
7d2a83486a2b301f2609c4c1105dc69feaaf77a149ab274f8d3854eb74b29139  research/prompts/m3-close-gov-b17-r1-opus-model/tools/rerun.sh
```

## 各格判定一览

| 格 | 判定 | 量的 / 推的 | 依据（详见各节） | 什么现象会推翻 |
|---|---|---|---|---|
| G1-a 边级排除写成「单元不进任何键」 | 打中：两处规则比定案多写 | 文字比对量的；后果推的，没找到实例 | kb 435 对 implementation-workflow 31、verification 50 | 定案另有一句把边级也写成按单元排 |
| G1-b 边级三格自证 | 打中：implementation-workflow 只管「新加」，verification 一句都没有 | 量的 | 同上 | verification 别处另写了这条自证 |
| G1-c/d 理由词 | 「按角色排」是论证，该留在 kb；「按实测排」半句是判据 | 推的 | rules-discipline 第 1、2 条 | — |
| G1-e 模块级语句 | 不致错（verification 的宽读法与代码一致） | 量的 | lib_function_units.py 72 | 代码不收 module-body |
| G1-f 现状句「现在只有那一条」 | 不该进规则，写成指到 EXCLUDED_PROGRAM_EDGES | 推的 | rules-discipline、lib_function_units.py 7150、9861 | 规则纪律另许现状句 |
| G1-g NOT_JUDGING_FILES 在 crates 变异判法单元里 | 打中（代码不合定案与规则，不分辨两者） | 量的：961 个单元里 496 个出自 lib_function_units.py | crates-mutation-rows.py 66、lib_mutation_units.py 753 | 行键入库前另有一步滤掉它们 |
| G2-a 留着的复跑副本被收工闸拦 | 打中 | 认法量的，Stop 被拦是照闸文件头推的 | completion-cleanup-guard.sh 11、39、46–51；m3-formal-code-b17-r1 那份 rerun.sh | 闸对判决复跑副本另有放行 |
| G2-b 跨会话之后没人能删 | 打中 | 推的（三句字面合取） | main-agent 30、guard 19 | 归属能跨会话转、或第 11 条不再写「别的会话的不碰」 |
| G2-c 与第 11 条读来直接打架 | 打中 | 推的 | main-agent 30 | 第 11 条本身改了 |
| G3 逐项 | 打中三项（G3-a、G3-b、G3-c），其余对得上或只有小出入（少 `--locked`、少「真引到 gpu 原子」一个限定、ITEM 行只打现判的条），逐项见 G3 表 | 量的（源码） | broad-mutation.py 各行 | — |
| G3-a 两台分片 | 打中：第二台有卡时 GPU 趟两份库各判一遍 | GPU 行不导入是量的；第二台真跑起来是推的 | broad-mutation.py 4226、147、3913；kb 433 | 第二台清场一律退 78 |
| G3-b driver-error 出路 | 打中（小）：加大限时不对症 | 推的 | broad-mutation.py 137、3447 | driver-error 另有因限时而起的一处 |
| G3-c ③ 退 1 | 打中：GPU 复用哨兵红退 1，commit-stages 与驱动退出码一段都没写 | 量的 | broad-mutation.py 4656、253–254 | ④ 另读哨兵结果 |
| G4-a 副本里的 replay.sh `--list-items` | 打中：照放、不读正文，副本正文起 checker 档 cargo 也放 | 量的（钩子现喂） | lib_heavy_tests.py 3326、3247；heavy-test-guard.sh 814 | 副本另有一道闸拦改 replay.sh |
| G4-b BASH_ENV 外壳 | 打中，但不是这一次放行带来的 | 量的 | lib_heavy_tests.py 2977 | 钩子另读 BASH_ENV |
| G4-c xargs | 已知看不见 | 量的 | heavy-test-guard.sh 181 | — |
| G4-d one-scenario 读的目录 | 对得上 | 量的 | crash-case-check.py 303、307 | 子目录里出现 `#[test]` |

## G1 两处 D13（验证路线） 已定项 17 复述与定案逐项比

定案原文都在一行里（kb 文件自己的行号，`grep -nF` 现查）：

- `.claude/kb/decisions/13-验证路线.md:435`：「文件级：scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES 那两份（算键的库自己与单元算法 scripts/gate/lib_function_units.py）按角色排，管怎么存、怎么算键，不管判对错，它们的单元不进任何一份判定的键。」
- `.claude/kb/decisions/13-验证路线.md:435`：「边级：一条只打提示、不改判定结局的子进程边，按实测排，不沿它收；现在只有内存包装 HostLookup.query 起 scripts/hosts/host-resources.py 那一条，第二台那一路只出提示，按这条边排出键。」
- `.claude/kb/decisions/13-验证路线.md:435`：「边级排除要配三格自证：这条路上的任何异常不改包装退出码；判法读包装输出前去掉这条路打的行；切边前后少掉的单元只经这条边可达。」
- `.claude/kb/decisions/13-验证路线.md:435`：「另有一格…必红。python 模块有单元进闭包时，它在 import 时执行的模块级语句一起收进来：import、赋值、类头本身（不连方法）、带装饰器或缺省值的 def 头。」

规则原文：

- `.claude/rules/implementation-workflow.md:31`：「算键时有两种排除，单元不进任何一份判定的键：文件级——scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES 那两份（算键的库自己与 scripts/gate/lib_function_units.py），按角色排」
- `.claude/rules/implementation-workflow.md:31`：「边级——只打提示、不改判定结局的子进程边，按实测排、不沿它收，现在只有内存包装 HostLookup.query 起 scripts/hosts/host-resources.py 那一条。新加边级排除配三格自证」
- `.claude/rules/verification.md:50`：「文件级排除（scripts/crash/lib_verdict_library.py 的 NOT_JUDGING_FILES 那两份）与边级排除（只打提示、不改判定结局的子进程边，现在只有 HostLookup.query 起 scripts/hosts/host-resources.py 那一条）的单元不进键；python 模块有单元进闭包时，它在 import 时执行的模块级语句一起进闭包」

| 编号 | 差异 | implementation-workflow | verification | 判定 | 量的 / 推的 |
|---|---|---|---|---|---|
| G1-a | 「单元不进任何一份判定的键」定案只挂在文件级上；边级定案只说「不沿它收」（切一条边，被起程序的单元经别的路照收）。两处规则都把「单元不进键」挂到两种排除上 | 多了：冒号前一句管两种 | 多了，且连「不沿它收」都没写，读来就是按单元排 | 打中（与定案不一致） | 文字比对是量的；后果推的：没找到第二条路把 `host-resources.py` 的单元带进某份判定（crates 变异判法单元清单里 0 个 `host-resources` / `host-dispatch` 单元，量的） |
| G1-b | 边级「要配三格自证」与「另有一格…必红」：定案对每一条边级排除（含现有那一条）都要求 | 少了：写成「新加边级排除配三格自证」「另加一格」，现有那一条的自证不在规则管内 | 少了：整句没有 | 打中（两处不一致，verification 那一处缺判据） | 量的（`grep -nF '三格自证' .claude/rules/verification.md` 0 行） |
| G1-c | 「管怎么存、怎么算键，不管判对错」「理由不同、分开记」「第二台那一路只出提示，按这条边排出键」 | 没抄 | 没抄 | 没抄对：这几句是论证，照 rules-discipline 第 2 条该留在 kb | 推的 |
| G1-d | 「按角色排」「按实测排」 | 抄了 | 没抄 | 「按角色排」是理由（删掉读的人少知道什么：什么都不少，判据是 NOT_JUDGING_FILES 那张表）；「按实测排」半句是判据（新边要量过才能排），与三格自证重叠 | 推的 |
| G1-e | 模块级语句 | 逐字抄了四种 | 没列四种，只说「模块级语句一起进闭包」 | 不致错：代码比四种收得更宽，`scripts/gate/lib_function_units.py:72`：「其余模块级语句在 module-body 里，本来就连。」verification 那句的宽读法与代码一致 | 量的（代码原文） |
| G1-f | 现状句「现在只有…那一条」 | 有 | 有 | 不该进规则（理由在下） | 推的 |

G1-f 的理由：
- 规则要写的是「什么规则在哪里：指到别的规则、脚本或 kb」（`.claude/singlefs-ai-sop/rules/rules-discipline.md:16`：「什么规则在哪里：指到别的规则、脚本或 kb」）。这条边已有一张登记表，`scripts/gate/lib_function_units.py:7150`：「EXCLUDED_PROGRAM_EDGES = {」，规则写成「登记在 EXCLUDED_PROGRAM_EDGES」就不随条数变。
- 条数今天确是一条，而且自证把它钉成恰好一条：`scripts/gate/lib_function_units.py:9861`：「(source, unit_name, target), = EXCLUDED_PROGRAM_EDGES」（多一条就解包失败）。加第二条边时，同一句现状要改四处：两份规则、kb 那一行、`scripts/gate/lib_function_units.py:76`：「今天只一条：内存包装的 HostLookup.query 起 scripts/hosts/host-resources.py」。
- 回扫靠搜旧说法，`.claude/singlefs-ai-sop/rules/evidence-discipline.md:166`：「回扫之前先写出…拿这些说法全仓搜」；规则纪律门禁认不出这句：`.claude/singlefs-ai-sop/scripts/rules-lint.sh:171` 只认「今天」后跟数字那一种（`re.compile(r"今天\s*\d")`），「现在只有」不判。
- 推翻条件：rules-discipline 或规则纪律门禁另有一条明许规则写现状句、或 EXCLUDED_PROGRAM_EDGES 被删掉不再是登记表。

### G1-g（打中的是代码，不是规则字面）：文件级排除在 crates 变异的判法单元指纹里没生效

规则与定案都说 NOT_JUDGING_FILES 的单元「不进任何一份判定的键」。现跑 `python3 scripts/mutation/crates-mutation-rows.py judging-units <仓根> --units`（只算闭包，不起 cargo），首行与按文件数的单元在 `research/prompts/m3-close-gov-b17-r1-opus-model/out/crates-judging-units-by-file.txt`，首行原样：

```
0ad75a2c6f2873abb32dc13107d699545032c1ab169527cb27ef0c6151950954 961
```

按文件数单元（`awk -F'\t' 'NR>1{split($3,a,":"); print a[1]}' … | sort | uniq -c | sort -rn | head`）前三行原样：

```
    496 scripts/gate/lib_function_units.py
    222 scripts/mutation/crates-mutation-rows.py
     70 scripts/mutation/lib_mutation_units.py
```

这个指纹进每一行 crates 变异的行键：`scripts/mutation/crates-mutation-rows.py:66`：「行键是被改坏的函数单元指纹、原文、替换文、点名测试的单元闭包键、判…的判法单元指纹、工具链」。排除只做在 `scripts/crash/lib_verdict_library.py:221`：「NOT_JUDGING_FILES 两份的单元不进」；crates 变异走的是 `scripts/mutation/lib_mutation_units.py:753`：「def judging_units(repository, entries, whole_file=False):」，函数体里没有按 NOT_JUDGING_FILES 滤的一步。

照规则字面做会做错的情形：工具员只改 `scripts/gate/lib_function_units.py`（例如加一条边级排除），照规则判「这份文件的单元不进任何键、不让判定作废」，提交时 checker-tier-crates-mutation-replay 的每一行判法单元指纹都变、整表现判。

四句：分不分辨臂——不分辨规则字面与定案（两边写法相同，病根在代码）；被判的系统看不看得到——算键的代码看得到 NOT_JUDGING_FILES（同仓常量），只是这条路没用它；满足的分句——定案「它们的单元不进任何一份判定的键」那一分句；改法——两处规则怎么改都修不到这一格，要改 `lib_mutation_units.judging_units`（或在它的入口外滤掉 NOT_JUDGING_FILES 两份），这一条只在我的草稿上量过、被攻过零轮。

推翻条件：crates 变异的行键里判法单元指纹另有一步在入库前滤掉 lib_function_units.py 的单元（那一份清单里的 496 个不进最终的键）。

## G2 判决复跑副本留到下一轮判决写完

被判的句子，`.claude/rules/three-way-inference.md:127`：「主 agent 判决复跑的副本，到这一题的下一轮判决写完、或这一题不再开下一轮时才算用完，那时再删」。

它压着的条款，`.claude/main-agent.md:30`：「哪一步用完就在那一步删，不等一批收尾，不留给以后的趟或别的任务复用；两台一样，删之前 du -sh 记大小，别的会话的不碰」。

判定：打中，两样都给得出具体情形。

### G2-a 照字面留着，收工闸照拦（量了认法，拦不拦是按闸的文件头推的）

具体情形：这一批自己的代码轮 m3-formal-code-b17-r1。主 agent 判决时照第 3 步把 `research/prompts/m3-formal-code-b17-r1-opus-model/` 拷进自己认领的草稿目录跑它的 `tools/rerun.sh`。那份脚本做的事：

- `research/prompts/m3-formal-code-b17-r1-opus-model/tools/rerun.sh:17`：「cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/repo/"」
- `research/prompts/m3-formal-code-b17-r1-opus-model/tools/rerun.sh:18`：「rsync -a --exclude target --exclude .git "$FROZEN/crates" "$WORK/repo/"」
- `research/prompts/m3-formal-code-b17-r1-opus-model/tools/rerun.sh:25`：「cargo test --offline -p singlefs-harness」

跑完草稿目录里有 `repo/`（Cargo.toml 加 crates）与 `repo/target/`（cargo 的编译目录）。腿的模板默认步骤同样拷整棵冻结树：`scripts/agents/three-way-rerun-template.sh:76`：「rsync -a --exclude target --exclude .git "$FROZEN/" "$WORK/tree/"」。

闸怎么认（文件头原文）：
- `.claude/hooks/completion-cleanup-guard.sh:11`：「Stop（主 agent）：手里没有在跑的后台任务与子 agent 时才判」
- `.claude/hooks/completion-cleanup-guard.sh:39`：「仓副本：有 .git（目录；.git 是文件的报成工作树），或同时有 Cargo.toml 与 crates」
- `.claude/hooks/completion-cleanup-guard.sh:46`：「种类四选一：报告、补丁、备份、交接」
- `.claude/hooks/completion-cleanup-guard.sh:47`：「主 agent 结束一轮时照列，用完就删」
- `.claude/hooks/completion-cleanup-guard.sh:48`：「编译目录照列（编译目录不能声明成交付物）」
- `.claude/hooks/completion-cleanup-guard.sh:51`：「拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户定」

认法量过：照那份 rerun.sh 的形状造一个空壳（Cargo.toml、Cargo.lock、crates/、target/CACHEDIR.TAG，不拷仓、不编），调闸导入的同一个函数 `scripts/agents/agent-watch.py:1443`（「if ".git" in names or ("Cargo.toml" in names and "crates" in names):」），`research/prompts/m3-close-gov-b17-r1-opus-model/out/cleanup-output.txt` 原样：

```
kind repo-copy = 仓副本
kind its-target = 编译目录
kind model-copy-only = None
found judgement-rerun/repo 仓副本
visited=4 truncated=False
```

推出来的后果（推的，没在真会话里触发 Stop）：从第一轮判决写完到第二轮判决写完之间（第二轮要等腿、等实验，常跨好几轮对话），主 agent 每一次手里没有在跑的活而结束一轮，闸都列出这份仓副本与它的编译目录拦下；仓副本只能声明成四种之一，判决复跑的副本不是报告、补丁、备份，「交接」对主 agent 照列；编译目录怎么声明都照列。每一轮第一次 Stop 被拦、续跑回来同一张单才放行并把单子作 systemMessage 交给用户。只拷模型目录、模型自己不建仓副本也不编的（例如这一份攻方腿的模型目录），闸不列（`kind model-copy-only = None`），这一格只打中带编译的模型。

### G2-b 跨会话：照字面留着，之后谁都不能删

三条字面合起来：
- 新句子要留到「这一题的下一轮判决写完」；
- `.claude/main-agent.md:30` 管的是「本会话建的仓副本、编译目录」，又写「别的会话的不碰」；
- 闸只判本会话认领的目录：`.claude/hooks/completion-cleanup-guard.sh:19`：「归属文件 .owner 的 session 是本会话的每个目录」。

第一轮在会话甲判决、第二轮在会话乙判决（开新会话这件事规则不禁，只禁因上下文大换会话），会话乙写完第二轮判决时副本「才算用完」，可它是会话甲建的、归属文件写的是会话甲，乙照「别的会话的不碰」不删，甲已经不在，乙的收工闸又不判它：这份副本永远没人删。推的（三句字面的合取），没在两个真会话里走过。

### G2-c 两句读来直接打架

`.claude/main-agent.md:30`「不等一批收尾，不留给以后的趟」——下一轮判决就是「以后的趟」，常在别的批。新句子用括号把第 11 条的「用完」改成指这一步，第 11 条本身没改，只读第 11 条的人照它删；只读新句子的人照它留。判定：矛盾，推的（读法比对）。

四句（G2-a）：分不分辨——分辨「留」与「当步删」两种写法（当步删的那一臂闸不拦）；系统看不看得到——闸看得到目录内容，看不到「这是判决复跑、下一轮还要用」；满足的分句——闸文件头「仓副本……或同时有 Cargo.toml 与 crates」与「编译目录照列」；改法：(1) 新句子改成「判决里写的复跑数连同复跑命令进判决，副本当步删、下一轮要用时重拷重跑」——对 G2-a、G2-b、G2-c 三格都起作用（推的）；(2) 只给 `.deliverables` 加一个「判决复跑」种类——修 G2-a 的仓副本那一半，编译目录照列、G2-b 跨会话照旧没人删（推的）。两个改法都只在我的读法上推过、被攻过零轮。

推翻条件：闸对主 agent 另有认「判决复跑副本」的放行（文件头与代码里找得到），或 m3-formal-code-b17-r1 那份 rerun.sh 不在主 agent 的认领目录里建 `repo/`。

## G3 commit-stages 第 2b 步「GPU 那一趟」逐项对驱动源码

被判的是 `.claude/skills/crash-test/commit-stages.md` 第 23、26、28 行里新加的 GPU 那几句；源码是 `scripts/mutation/broad-mutation.py`（sha256 与快照 `judged-sha256.txt` 对得上，`sha256sum -c` 现跑 OK）。

| 项 | commit-stages 写的 | 源码 | 判定 |
|---|---|---|---|
| 两趟、两份库、任一趟红算抓到 | `.claude/skills/crash-test/commit-stages.md:23`：「每条变异两趟：CPU 那一趟（第一段、第二段，判定在库目录的 verdicts.tsv），加 GPU 那一趟——CPU 那一趟没红、又有带 gpu 特性的测试够得着的条」 | `scripts/mutation/broad-mutation.py:3297`：「GPU_VERDICTS = "gpu-verdicts.tsv"」；合起来的状态在 `scripts/mutation/broad-mutation.py:3473`：「return ItemState(gpu.state, cpu.passed_stage_one), f"gpu-{gpu.state}"」 | 对得上；「每条变异两趟」头半句说多了（只有 CPU 没红又够得着的条有第二趟），后半句自己收回 |
| 要这一趟的条件 | 同上「又有带 gpu 特性的测试够得着」 | `scripts/mutation/broad-mutation.py:133`：「含这条变异的容器单元、又真引到 gpu 原子」 | 少一个限定（闭包要真引到 gpu 原子）；只影响读的人估 gpu_needed，驱动自己判，不致错 |
| 基线 | `.claude/skills/crash-test/commit-stages.md:26`：「先在没改过的树上把要跑的目标跑绿一遍当基线（不绿退 6）」 | `scripts/mutation/broad-mutation.py:3809`：「refuse(f"GPU 趟基线 {GPU_BUILD_STEP} 在没改过的树上退」（退 EXIT_BASELINE_RED = 6） | 对得上 |
| 编译步命令 | 同一行「cargo test --release -p singlefs-checker-tier --features <tests/ 里 cfg 点到的特性> <目标> --no-run」 | `scripts/mutation/broad-mutation.py:3382`：「head = ["cargo", "test", "--locked", "--release", "-p", CHECKER_TIER_PACKAGE」 | 少 `--locked`；照 commit-stages 手抄命令重现一条的人会不带它，推的、不影响驱动 |
| 测试步、清场、单线程 | 同一行「跑测试步（同一条命令不带 --no-run，后面加 -- --nocapture --test-threads 1，整条经 scripts/crash/gpu-quiesce-run.sh 照多机配置清场并回读」 | `scripts/mutation/broad-mutation.py:3386`：「threads = [] if break_is("gpu-test-threads-dropped") else ["--test-threads", "1"]」；`scripts/mutation/broad-mutation.py:3387`：「return quiesce + head + ["--", "--nocapture", *threads]」 | 对得上（只有测试步经清场，编译步不经，与 commit-stages 写法一致） |
| BINDGEN | 同一行 | `scripts/mutation/broad-mutation.py:3791`：「extra = {"BINDGEN_EXTRA_CLANG_ARGS": self.bindgen} if self.bindgen else {}」 | 对得上 |
| 内存包装与线程 | 同一行「每一步仍经内存包装、取 --memory-cap 与 --threads」 | `scripts/mutation/broad-mutation.py:2628`：「argv = ["bash", MEMORY_CAP_RUNNER, memory_cap, "bash", THREAD_CAP_RUNNER, str(self.arguments.threads), *command]」 | 对得上 |
| 清场状态目录共用 | 同一行「共用清场包装的状态目录、不抢在跑的重型」 | `scripts/crash/gpu-quiesce-run.sh:45`：「同一台机器上同时用卡的各趟要用同一个状态目录」 | 对得上（是调用方的约定，驱动不另设，取默认那一份） |
| 没卡 / 清场没过 / --skip-gpu | 同一行「要这一趟的条逐条打 BROAD_MUTATION_GPU_NOT_RUN，本次未跑，不算没红也不算通过，④ 因此判红」 | `scripts/mutation/broad-mutation.py:3444`、`:3449`、`:3947` 三处分支；④ 判红在 `scripts/mutation/broad-mutation.py:3476`：「return ItemState(NOT_RUN, cpu.passed_stage_one), "gpu-not-run"」 | 对得上；两处小出入：「（退 78：…）」是清场包装的码，驱动自己照常退 0；带 `--skip-stage-two` 时整趟不跑、一行 GPU_NOT_RUN 都不打（`scripts/mutation/broad-mutation.py:3903`：「say("  · GPU 趟接在第二段后面，这一趟带了 --skip-stage-two，不跑")」），commit-stages 列的三种原因里没有它 |
| 报的行 | 同一行「每条一行 BROAD_MUTATION_GPU_ITEM（cpu= 与 gpu= 分开写）」 | `scripts/mutation/broad-mutation.py:3976`：「say(f"BROAD_MUTATION_GPU_ITEM sequence=」只在现判的循环里打（`scripts/mutation/broad-mutation.py:3960`：「for item, need, _reason in pending:」） | 小出入：沿用的条不打 ITEM，「每条」应是「每条现判的」 |
| 成功句 | 同一行「成功句末尾接…现判 a、复用 b、本次未跑 c」 | `scripts/mutation/broad-mutation.py:4665`（gpu_totals_text） | 对得上 |
| 超时、撞顶 | 同一行「GPU 趟不做超时升档，超时、内存撞顶记没结论（--timeout-floor / --memory-cap 加大再 --item <序号> --rerun）」 | `scripts/mutation/broad-mutation.py:137`：「超时（253）、内存撞顶（250）记 timeout / memory-cap（没结论，这一趟不做升档」 | 对得上 |
| 两台各跑一片 | 同一行「没卡或不清场的那一台加 --skip-gpu，GPU 判定行不在导入的范围里」「再在本机起一趟 ③（不带 --shard，沿用得了的不重跑）补另一片的 GPU 趟」 | `scripts/mutation/broad-mutation.py:4226`：「peer_path = os.path.join(directory, "verdicts.tsv")」；`scripts/mutation/broad-mutation.py:147`：「没做：升档（超时、撞顶两趟取证）、--import-results 并 GPU 判定行（两台各跑一片时 GPU 趟在拿 GPU 的那一台上跑、那一台的库里有）」 | **打中**，见 G3-a |
| ④ 四个数 | `.claude/skills/crash-test/commit-stages.md:28`：「BROAD_MUTATION_GATE 那一行另抄 gpu_needed、gpu_reused、gpu_not_run、gpu_without_conclusion 四个数」 | `scripts/mutation/broad-mutation.py:4136`：「gpu_fields = f" gpu_needed={len(gpu_needed)}」 | 对得上 |
| NO_CONCLUSION 理由串 | 同一行 gpu-not-run、gpu-timeout、gpu-memory-cap、gpu-driver-error | `scripts/mutation/broad-mutation.py:3473` 拼 `gpu-<状态>`，状态取 timeout、memory-cap（`ENVIRONMENTAL_OUTCOMES`）与 driver-error；`:3476` 是 gpu-not-run | 串对得上；driver-error 的出路不对症，见 G3-b |
| ③ 的退出码 | 同一行「退 3、5、6 照抄」「退 143 照 ③ 那一句重起。不论 ③ 退几，④ 照跑」 | `scripts/mutation/broad-mutation.py:4656`：「if gpu_totals.failed:」后 `return 1` | **漏一种**，见 G3-c |

### G3-a 两台各跑一片：照字面做，第二台的 GPU 趟白跑，同一批 GPU 判定两份库各判一遍

具体情形：本机多机配置里两台都点名了卡（只数了 `GPU_CARDS_BY_PRIORITY` 各项冒号前的那个词：`local` 1 项、`peer` 1 项，`QUIESCE_STOP_COMMAND` 有值；没抄任何值）。commit-stages 只叫「没卡或不清场的那一台」加 `--skip-gpu`，所以第二台有卡、清场命令也齐时，第二台那一趟 `--shard 1/2` 不带 `--skip-gpu`：CPU 那一趟跑完接着跑这一片的 GPU 趟，判定写进第二台的 `gpu-verdicts.tsv`。`--import-results` 只读对方的 `verdicts.tsv`，GPU 行不导。本机随后照 commit-stages「再在本机起一趟 ③（不带 --shard）」：这一片要 GPU 趟的条在本机库里没有 GPU 行，进 pending 全部现判（`scripts/mutation/broad-mutation.py:3913`：「if not in_run_scope(item, arguments):」只按选条参数筛，不看别处的库）。

结果：第二台判过的那一批 GPU 判定在本机再判一遍，两份库重复判同一批。定案把这算违反：`.claude/kb/decisions/13-验证路线.md:433`：「两份库重复判同一批，同样算违反」。驱动文件头那一句又说 GPU 判定留在拿 GPU 的那一台的库里，而 ④ 只读本机库，两份文字给的做法不一样。

量的 / 推的：GPU 行不导入、in_run_scope 不看别处是读源码量的；第二台上清场命令齐不齐、GPU 趟在第二台上真起不起来没量（不往第二台派活），这一格的前提「第二台清场能过」是推的。

四句：分不分辨——分辨「两台都 --skip-gpu、导入后本机补」与 commit-stages 现写法（前者不重判）；系统看不看得到——驱动在第二台上看不到本机之后会补跑；满足的分句——kb 433「两份库重复判同一批」；改法（只在我的读法上推过、被攻过零轮）：(1) commit-stages 改成两台各跑一片时两台都带 `--skip-gpu`，导入之后本机一趟不带 `--shard` 补全部 GPU 趟——修这一格；(2) 驱动的 `--import-results` 一并导 `gpu-verdicts.tsv`——同样修这一格，还要改驱动、补自证。

推翻条件：第二台的副本里没有多机配置的清场命令，`gpu-quiesce-run.sh` 在第二台上一律退 78（那样第二台 GPU 趟一律本次未跑、不落行，不重判；但这是配置碰巧，commit-stages 的字面仍许它跑）。

### G3-b gpu-driver-error 的出路不对症（推的）

commit-stages 把 driver-error 与超时、撞顶并在一处：`.claude/skills/crash-test/commit-stages.md:28`：「理由写 gpu-timeout / gpu-memory-cap / gpu-driver-error 的是 GPU 趟没结论的，加大 --timeout-floor 或 --memory-cap 再 --item <序号> --rerun」。driver-error 的来源是 `scripts/mutation/broad-mutation.py:137`：「退 0 而一个测试都没跑到记 driver-error（不算没红）」与认不出的退出码（`scripts/mutation/broad-mutation.py:3447`），与限时、上限无关；加大两样再跑，同一原因下还是 driver-error。出路该是「看那一条的日志、修好驱动或环境后 --item <序号> --rerun」。推翻条件：driver-error 在源码里另有一处因限时或内存而起。

### G3-c ③ 退 1 没有处置（量的：源码与两份文字都查了）

GPU 那一趟的复用哨兵判红时驱动退 1（`scripts/mutation/broad-mutation.py:4656`，CPU 那一趟没东西要判时同样一处在 `:4607`）。commit-stages 第 28 行只点了 4、2、3、5、6、143；驱动文件头「退出码」一段里 1 的几种原因也没写哨兵（`scripts/mutation/broad-mutation.py:253`：「退出码：0 选中的条都判完；1 --summary 栏数对不上或判定库有对不上的行」与 `:254` 两行列的都不是哨兵）。照字面：③ 退 1、④「照跑、原样抄它的判定」，复用失灵那一条现判过、新行已落库，④ 可以判绿，提交照常进，复用失灵没人处置。退 9（运行目录没删掉）同样不在 commit-stages 的单子上，那一处不是这一批新加的。推翻条件：④ 或 checker-tier-broad-mutation 另读哨兵的结果、哨兵红时判红。

## G4 重型闸对 `replay.sh --list-items` 的放行

喂法：`research/prompts/m3-close-gov-b17-r1-opus-model/probe_heavy_guard.py` 对每条命令调 `lib_heavy_tests.heavy_tests_in_command_text`，再把同一条命令包成钩子 JSON（`agent_type` 写 implementation-writer）从 stdin 喂给 `.claude/hooks/heavy-test-guard.sh`，检出记录指到草稿目录。被判的命令一条都没执行。造的「副本」只在草稿目录里写了一份 `scripts/experiment-apparatus/replay.sh`，它的 `--list-items` 那一支写着 `cargo build … -p singlefs-checker-tier --features verdict-store,gpu --lib` 与 `cargo test --release -p singlefs-checker-tier --lib --tests`。

钩子版本：前三趟（probe-run-1、rerun-2、rerun-3）喂的是快照那一版（sha256 与 `judged-sha256.txt` 相同，`git show :<路径>` 现算）；那之后别的会话改了这两份（加的是 here-string 与管道喂给 shell 的那一判），rerun-4 在新版（`lib_heavy_tests.py` b1c3ef8d…、`heavy-test-guard.sh` ed9e3102…）上重喂，27 行与模型目录里的逐行相同。下面引的钩子行号是新版的；`lib_heavy_tests.py` 第 627 行以后比快照那一版多 2 行。

`research/prompts/m3-close-gov-b17-r1-opus-model/out/probe-output.txt` 里与判定有关的行原样（`stderr=` 截 160 字，`<work>`、`<repo>` 是替换）：

```
name=judged_by_name copy=True elsewhere=False main=True
case=main-list-items	lib=none	guard_exit=0	stderr=
case=copy-absolute-path	lib=none	guard_exit=0	stderr=
case=copy-cd-relative	lib=none	guard_exit=0	stderr=
case=copy-cwd-relative	lib=none	guard_exit=0	stderr=
case=elsewhere-same-body-control	lib=none	guard_exit=2	stderr=✗ 重型测试被拒：脚本 <work>/elsewhere/replay.sh:5 里的 cargo test 跑到 singlefs-checker-tier 包的测试（checker 档，默认只在提交时跑；按包）（checker 档）：implementation-writer 不跑「checker 档」
case=copy-crash-point-replay-list-items	lib=none	guard_exit=0	stderr=
case=bash-env-replay-list-items	lib=none	guard_exit=0	stderr=
case=bash-env-env-replay-list-items	lib=none	guard_exit=0	stderr=
case=bash-env-export-replay-list-items	lib=none	guard_exit=0	stderr=
case=bash-env-bash-c-true-control	lib=none	guard_exit=0	stderr=
case=xargs-bare-entry	lib=none	guard_exit=0	stderr=
```

### G4-a 换路径：副本里改过的 replay.sh，`--list-items` 照放、不读正文（量的）

命令（在主工作区起，或 cd 进副本起，或在副本里按相对路径起，三种都放行）：

```
bash <工作根>/<某个草稿目录>/scripts/experiment-apparatus/replay.sh --list-items
```

那份文件的 `--list-items` 一支起 checker 档的 cargo build 与 cargo test，钩子退 0。同一份正文放到不叫这个路径的地方（`elsewhere-same-body-control`），钩子读进正文、在第 5 行拒。差别只在路径：

- `.claude/hooks/lib_heavy_tests.py:3326`：「return any(candidate == location or candidate.endswith("/" + location)」——任何以 `/scripts/experiment-apparatus/replay.sh` 收尾的路径都算按名字判（`copy=True`）。
- `.claude/hooks/heavy-test-guard.sh:814`：「if heavy_tests.judged_by_name(command.words[0], command.directory):」后一行 `continue`，按名字判的不读正文。
- `.claude/hooks/lib_heavy_tests.py:3247`：「if [argument for argument in arguments if argument != "--force"] == EXPERIMENT_REPLAY_QUERY_ARGUMENTS:」只看参数，不看是哪一份文件。

与别的文字对不上的地方：
- 文件头说只有仓内那一份按名字判：`.claude/hooks/lib_heavy_tests.py:589`：「别处同名的拷贝不按名字判，读正文」；登记表那一行的注释 `.claude/hooks/lib_heavy_tests.py:727`：「按名字判的仓内脚本：名字 → 它在仓里的位置」。实测副本也按名字判。
- 同一道闸对副本里的 `--selftest` 要求与主仓那一份 sha256 相同：`.claude/hooks/heavy-test-guard.sh:97`：「副本里被起的那份脚本与主仓（钩子所在仓）那一份 sha256 相同」；`--list-items` 没有这一判。
- 规则对副本里改过的自证写了「钩子不读副本里的正文」，要主 agent 先 diff：`.claude/rules/implementation-workflow.md:46`：「钩子不读副本里的正文，这一步只靠主 agent 看」；对副本里的 `--list-items` 没有对应的一句。

照字面做会做错的情形：工具员或实现员照共用约束把仓 rsync 进草稿目录、在副本里改 `replay.sh`（加驱动、挪参数解析），改坏了 `--list-items` 那一支的次序（例如把它挪到 GPU 试编之后），再在副本里跑 `--list-items` 自查：起的是 checker 档的 cargo，闸放行，没人看正文。主仓那一份的 `--list-items` 本身是稳的：`scripts/experiment-apparatus/replay.sh:1358`：「if (($# != 1)); then」，多带一个词就退 2；preflight 摘 `--force` 与钩子摘法一致（`.claude/singlefs-ai-sop/scripts/preflight.sh:31`）。

这不是这一次放行独有的：同一套「路径尾部相同就按名字判」早就管着崩溃点重放入口（`copy-crash-point-replay-list-items` 一样退 0）；ji8 把它延到了 `replay.sh`。

四句：分不分辨——分辨「副本按内容判」与「副本按路径判」两种做法；闸看不看得到——看得到，那份文件就在盘上，`elsewhere` 那一格证明它读得了；满足的分句——lib 文件头「别处同名的拷贝不按名字判，读正文」与 `--selftest` 那一条的 sha256 判据；改法（只在我的读法上推过、被攻过零轮）：(1) `judged_by_name` 对工作根下的副本另核与主仓那一份 sha256 相同（照 `--selftest` 那一条）——修这一格，也顺带修崩溃点重放与崩溃放量入口的同形问题；(2) 只把 lib 文件头那句改成「路径尾部相同的副本也按名字判」——只修文字对不上，不修放过。

推翻条件：工作根下的副本另有一道闸（写闸、交回闸）拦住改 `scripts/experiment-apparatus/replay.sh`；或 `judged_by_name` 被别处以「只认仓根」的调用方式调。

### G4-b 经包装：BASH_ENV 带进来的脚本，钩子不读（量的；不是这一次放行带来的）

```
BASH_ENV=<一份写着 cargo test --release -p singlefs-checker-tier --lib --tests 的文件> bash scripts/experiment-apparatus/replay.sh --list-items
```

`env BASH_ENV=…`、前面 `export BASH_ENV=…` 两种写法同样退 0。非交互的 bash 起脚本之前先 source `$BASH_ENV` 那份文件，里面的 checker 档 cargo test 真会跑。对照格 `BASH_ENV=… bash -c true` 同样退 0：放过的是任何 bash 起的东西，`--list-items` 只是借来的一条合法外壳。lib 里知道这个变量，只在自己展开通配时剥掉：`.claude/hooks/lib_heavy_tests.py:2977`：「environment = {key: value for key, value in os.environ.items() if key not in ("BASH_ENV", "ENV")}」。钩子文件头「看不见的」那一句里没有它。推翻条件：钩子另有一处读命令前缀与 env 里的 BASH_ENV / ENV。

### G4-c 已知看不见的：xargs（量的，钩子文件头已写明）

`echo E142 | xargs bash scripts/experiment-apparatus/replay.sh` 退 0，真跑起来是复跑 E142。钩子文件头已写：`.claude/hooks/heavy-test-guard.sh:181`：「make（Makefile 里起的命令）与 xargs 起的命令」。不算这一次的新洞，列在这里只为说明试过。

### 试过、都被拒或无害的

| 形状 | 结果 | 为什么无害 |
|---|---|---|
| `--list-items E142`、不带参数、`--list-item?`、`A=--list-items; … $A`、`A=E142; A=--list-items … $A` | 钩子退 2 | 拒了（后两种是误拒一侧，不是放过） |
| `--list-items --force`、经 `run-with-memory-cap.sh`、经 `env nice capped.sh`、`source`、`sh`、`bash -x` | 退 0 | 起的仍是主仓那一份，`--list-items` 只列项；`sh` 起时 bash 专有语法会出错（推的，没跑） |
| `--list-items && … E142`、`--list-items; cargo test -p singlefs-checker-tier` | 退 2 | 拼接的后一段照判 |
| `echo E142 \| xargs … --list-items` | 退 0 | 真参数是 `--list-items E142`，replay.sh 第 1358 行退 2 |

### G4-d verification.md one-scenario 那一行对判法读的目录（量的，对得上）

`.claude/rules/verification.md:133`：「读 crates/singlefs-harness/tests/ 与 crates/singlefs-checker-tier/tests/ 的用例源码，不跑 checker 档」。判法 `scripts/crash/crash-case-check.py:303` 与 `:307` 各取那个包 `tests/` 顶层的 `*.rs`（不进子目录）。两包 `tests/` 下子目录里的 `#[test]` 现数各 0 个（`grep -rc '#\[test\]'`，harness 顶层 2125、子目录 0；checker 档顶层 1003、子目录 0），今天「只读顶层」与「读 tests/」没有差别。推翻条件：子目录里出现 `#[test]`。

旁出的一处（推的）：verification.md 里 one-scenario 的做法句还写在「harness 档里再分快档用例与慢档用例」那一节，checker 档那几节没有；写 checker 档用例的人照「崩溃枚举用例住哪、怎么登记」一节找不到这条要求。另外 `.claude/rules/verification.md:136`「谁在什么时候跑得了 checker 档」那一行与 `.claude/rules/implementation-workflow.md:49` 的重型清单都没有点名直接起 `scripts/experiment-apparatus/replay.sh`（算重型）与它的 `--list-items` 例外，钩子有、规则没有。

## 没打中的形状

- G1：找第二条路把 `scripts/hosts/host-resources.py` 的单元带进某份判定的键（G1-a 的实例）。只查了 crates 变异的判法单元清单（0 个 host-resources / host-dispatch 单元）；`crash-point-replay.sh`、`detached-run.sh`、E171 那几份多机入口也起 host-resources.py，它们是不是某份判定闭包里的一条子进程边没查。
- G1：现有那一条边级排除缺不缺三格自证。找到的是有：`scripts/hosts/run-with-memory-cap.sh` 文件头「整体兜底」一段写了六格异常自证与三份判法去掉 HostLookup 行的正则，`lib_function_units.py` 有切边一格与「键不变而退出码变」一格。所以 G1-b 只打中文字（只管新加、verification 没写），没打中「现在缺自证」。
- G2：找有没有一种主 agent 的写法能让闸不列留着的复跑副本（不拷仓、不编的模型目录闸本来不列；拷仓或编过的只剩「备份」这个不对的种类能免仓副本，编译目录怎么都列）。
- G3：GPU 趟的命令、限时、BINDGEN、清场包装、四个数、理由串都对得上，没造出偏差。`--full` 下 GPU 趟翻了面不打 FLIPPED（只有 CPU 那一趟打），commit-stages 那一句没说包不包括 GPU，不算打中。
- G4：带别的参数（`E142`、`--list-item?`、变量）、经包装（内存包装、`env`、`nice`、`capped.sh`）、拼接（`&&`、`;`）、`--force` 前后、`source`、`sh`、`bash -x`，共 26 格，对主仓那一份全部要么拒、要么无害（replay.sh 第 1358 行只认单独一个 `--list-items`）。主仓那一份里能不能借 `--list-items` 起 cargo：没找到；`preflight.sh` 只摘 `--force`、`preflight.py check` 只判运行条件。工作根与 `/tmp/claude-1000` 下找了路径尾部相同的现成副本，没有一份是不带 `--list-items` 分支的旧版（有两份在正推腿的草稿目录里，只看了路径与 grep 计数，没读内容）。

## 这条腿自己的限度

- 只在本机跑，没往第二台派活：G3-a 的「第二台清场能过」是推的。
- G2-a 没在真会话里让 Stop 闸触发，拦不拦是照闸文件头与它导入的认法函数推的；认法函数是量的。
- G4 的副本样本是我造的（`--list-items` 一支直接写 cargo），没有现成的改坏副本；它证明的是「闸不读」，不是「已经有人这样改过」。
- 钩子在我跑的中途被别的会话改了两份（快照之后），rerun-4、rerun-5 在新版上重喂结果相同；新版的行号我按新版引，与快照那一版在 `lib_heavy_tests.py` 第 627 行以后差 2 行。
- 所有改法都只在我的读法或草稿上推过、被攻过零轮；表里标「推的」的没有实现、没有跑。

## 没做什么

- 没判正推、辩方那几格；没替主 agent 采纳。
- 没跑门禁阶段、没跑任何钩子的 `--selftest`、没编译、没跑重型；`crates-mutation-rows.py judging-units` 只算闭包，不起 cargo。
- 没读禁读清单里的文件，也没读正推腿草稿目录里的内容。
- 草稿目录清理：交回前 `du -sh` 824K，删掉自己建的 probe-run-1、cleanup-1、rerun-2 到 rerun-5 与几份草稿（其中 `judgement-rerun/repo` 是造的空壳，闸认作仓副本），删后 12K，只剩 `.owner` 与 `.writer-lock`。

## cite-check

`python3 scripts/agents/cite-check.py research/prompts/m3-close-gov-b17-r1-opus-output.md` 末行（写这一节之前跑的）：

```
  ✓ 核了 69 处引文、12 处裸行号，对上 69 处，没判 12 处（其中裸行号没有引文可比 11 处；逐处列在上面）
```
