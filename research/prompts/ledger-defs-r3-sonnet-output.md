# ledger-defs-r3 正推腿（Sonnet）报告

批次：`records/2026-11-12-subagent账本清欠.md#3`。草稿目录 `~/.local/share/singlefs/work/ledger-defs-r3-sonnet`（认领过）。没读禁读清单里的文件。

## 一、开工核的事（量的）

被判三份此刻的 sha256，与开工快照 `research/prompts/ledger-defs-r3-snapshot/judged-sha256.txt` 逐行相同（`sha256sum` 取，量的）：

```
614aaeabbb54b210a8ce09494be440aa3cbc6db157fe56050732be108af69ad5  .claude/main-agent.md
a4fcdce232aa33495774f1ac79b70d7b8614900bac1f80d717e57f0ec0d6566b  .claude/agent-common.md
d53dd7cb22425142d9541b4d71eb81188f29b0724262650d72cdd950bb62d0ba  .claude/agents/implementation-writer.md
```

「改前」的定义取 `git show HEAD:.claude/main-agent.md`、`git show HEAD:.claude/agents/implementation-writer.md`、`git show HEAD:.claude/agent-common.md`（草稿目录里存了三份拷贝，`head-*.md`）。HEAD 的「写与验分两段」是：有依赖的补丁逐份打进主工作区的工作树，验证段第 1 步分「带 `--skip-apply`（写段打过的那一批）」与「不带（没打过的那一批）」两条命令，没有段树、没有快照、没有合成补丁。

写法约定：引文是那一行里连续的原文（长行只截一段，段间用省略号）；脚本、钩子的行号是我这一次 `grep -nF` 现查的；「量的」指我这一次在盘上读到或数到的，「推的」指读码推的、没跑。行号在报告写完后由 `cite-check.py` 再核一遍（末行贴在第八节）。

## 二、第一组第 2 条：前两轮打中过的每一条，在现行版里的去向

去向只用四个词：消掉了、还在、换了个地方、不在这三份里。每条先给判，再给现行定义的文件:行与原文、脚本或钩子的文件:行与原文、让它消掉的那一句或那一处改动、什么现象会推翻。

### 二·甲 第一轮判决「二、判决」九条（`research/prompts/ledger-defs-r1-main-verification.md` 第 21–29 行）

**甲1 一·3（旧句与新句打架：验证段先合还是先验）——消掉了。**
- 定义：「绿了不合，之后在它留下的快照上验、合进主工作区的次序照」（`.claude/main-agent.md:7`）；「第 1、2 步都绿了才合进主工作区」（`.claude/main-agent.md:59`）；「绿了不合，之后在它留下的快照上交带 `--item` 的那几条、合进主工作区，次序照」（`.claude/main-agent.md:118`）；「验证段第 2 步在第 1 步留下的快照上交那一段打出的」（`.claude/main-agent.md:69`）；「验证段用的那一条点名这几行，由主 agent 在验证段第 2 步在第 1 步留下的快照上交」（`.claude/agents/implementation-writer.md:34`）；「由主 agent 在验证段第 2 步在第 1 步留下的快照上经服务交」（`.claude/agent-common.md:115`）。
- 脚本：「补丁这时只在快照上验过，主工作区一个字节没动」（`scripts/gate/merge-and-verify.sh:2681`）。
- 消掉它的是：第一轮点名的五处（职责、第 10 条、验证段、拒算一节、派发表）与实现员、共用约束各一处，现在都写成同一个次序（先在快照上验、第 2 步在同一棵快照上验、都绿了才合）；第一轮点名的两句旧话（拒算一节「在合入后的主工作区里跑」、变异证红「补丁打进主工作区之后用的那一条」）三份里 `grep -c` 都是 0（量的）。
- 推翻它：三份里再出现「合入后验证绿了就合」「合进主工作区之后再交某条验证」的句子。

**甲2 一·2（三棵树不一样：主工作区在建树之后被改而不冲突）——换了个地方，范围缩小，没消干净。**
- 定义：「先在仓根直接跑核的那几条：主工作区自那一趟开跑以来，补丁改到的包连同工作区里依赖它们与被它们依赖的包、树根的 `Cargo.toml`、`Cargo.lock`、`.cargo/` 有没有变，这一批要换掉、删掉的变异行有没有被改」（`.claude/main-agent.md:59`）。
- 脚本：「packages = closure(packages, path_dependencies) | closure(packages, lambda directory」（`scripts/gate/merge-and-verify.sh:3011`）；「starts += [os.path.join(tree, name) for name in WORKSPACE_ROOT_ENTRIES」（`scripts/gate/merge-and-verify.sh:3012`）；「if relative in registration_tables or relative == MUTATION_TABLE:」（`scripts/gate/merge-and-verify.sh:2997`，登记表与变异表被 `continue` 掉，不进起点）。
- 换了什么：第 2 步不再在段树上验，改在第 1 步的快照上验（`.claude/main-agent.md:58`），被验的树只剩一棵；主工作区漂移由开跑指纹核。
- 还在的：指纹的起点只有补丁改到的包、它们的依赖闭包与树根三样；`scripts/`、`.claude/gate.d/` 的门禁脚本、登记表里别的会话的追加不在起点里（`scripts/gate/merge-and-verify.sh:166` 写明登记表与 `crates/mutations.tsv` 交给三方合并）。第 2 步用的是开跑那一刻拷进快照的门禁脚本，开跑之后主工作区上这些脚本被改，合进去的组合没有被新脚本判过（推的，读码）。
- 推翻它：在快照开跑之后改 `scripts/gate/*.sh`、不碰补丁的包，核指纹那一条仍退非 0；或第 2 步的门禁脚本是合并时才现取的。

**甲3 一·1（走一遍）——消掉了，五处里有一处句子对不上脚本（第四部分 D1）。**
- 合到一半退 2：「它一次整份写完或一个字不改，没有合到一半」（`.claude/main-agent.md:59`）；脚本「几份补丁合成一份，合进主工作区就是一次 apply-writer-patch.py」（`scripts/gate/merge-and-verify.sh:149`）；「换上与被杀（要么整份写完、要么一个字不动）：」（`scripts/gate/apply-writer-patch.py:30`）。消掉它的是把「逐份打」改成「合成一份补丁再打一次」。
- 冲突出路里的建段树命令与写段第 2 条不一致：现在两处同一条，脚本「SEGMENT_TREE_COMMAND='python3 scripts/hosts/minimal-copy.py <仓根> <新段树> --with-test-inputs --with-gate-tables --with scripts=<为什么> --with .claude/scripts=<为什么> --with .claude/singlefs-ai-sop=<为什么>'」（`scripts/gate/merge-and-verify.sh:634`），定义「`python3 scripts/hosts/minimal-copy.py <仓根> <段树> --with-test-inputs --with-gate-tables --with scripts=<为什么> --with .claude/scripts=<为什么> --with .claude/singlefs-ai-sop=<为什么>`」（`.claude/main-agent.md:47`）。
- 第 2 步红了的重跑命令：「第 1 步连同它整批重交（不带 `--item` 与 `--fingerprint-of`，键相同的照复用），第 2 步那几条在新留下的快照上照原来点的全部项重交（键相同的照复用）」（`.claude/main-agent.md:58`）；脚本出路「退非 0 不合，照原命令去掉 --item 与 --fingerprint-of、整批重交，第 2 步在新留下的快照上重验」（`scripts/gate/merge-and-verify.sh:2682`）。
- 新崩溃枚举用例的登记落哪棵树：登记行进补丁，「新崩溃枚举用例与放量分项的登记行跟着实现员的补丁走」（`.claude/main-agent.md:47`）；脚本「登记表（REGISTRATION_TABLES：」那一条（`scripts/gate/merge-and-verify.sh:85`）与 `REGISTRATION_TABLES=(`（`scripts/gate/merge-and-verify.sh:642`）。
- mutation-tables 格：「再交它打出的」那一条写进了验证段第 3 步（`.claude/main-agent.md:59`）；脚本打出它：「补丁打进主工作区之后另交：」（`scripts/gate/merge-and-verify.sh:2443`）。
- 推翻它：脚本在快照上打补丁的次序里仍有按份合进主工作区的路径（`grep -n 'apply-writer-patch' scripts/gate/merge-and-verify.sh` 里除 `--root "$tree"` 那一条外再出现不带 `--root` 的逐份调用）。我现查只有 `scripts/gate/merge-and-verify.sh:2416`（带 `--root`）与 `:2702`（合成的那一份）。

**甲4 一·4（代码轮快照；段树合完就删之后代码轮打中的修补没有段树可做第 2 步）——主句仍没打中，附带那一处消掉了。**
- 定义：「整段的合入里有产品代码改动的，代码轮三方开一轮，快照取在全部合入之后，攻整段的合入」（`.claude/main-agent.md:60`）；「已合进主工作区之后才红的（代码轮判决打中这类），修补的实现员副本从主工作区建，第 1 步那一条只给这一批修补的补丁目录，再走第 2 步」（`.claude/main-agent.md:61`）。
- 消掉附带那一处的是：第 2 步不再要段树，只要第 1 步留下的快照，修补走「第 1 步只给修补目录 → 新快照 → 第 2 步」，不需要重建段树。
- 推翻它：主工作区合入之后代码轮打中，修补仍要求先建段树才走得通第 2 步。

**甲5 一·5（段树与完工清理闸）——段树那一半消掉了，留下的快照换了个地方、完工清理闸不列它。**
- 定义：「段树只给写段的下一批当基线：验证段第 1 步那一条交出去就删」、「一段在写段里跨轮停下也删」（`.claude/main-agent.md:47`）；快照：「删这一趟留下的快照」（`.claude/main-agent.md:59`）。
- 脚本：「要留现场：MERGE_AND_VERIFY_KEEP_COPY=1，收尾不删，打出快照与编译目录的路径和删法，看完自己删」（`scripts/gate/merge-and-verify.sh:36`）；删法那一行「→ 看完自己删：rm -rf」（`scripts/gate/merge-and-verify.sh:821`）；快照放在日志目录下：「日志落 $MERGE_AND_VERIFY_LOG_DIRECTORY」（`scripts/gate/merge-and-verify.sh:127`，默认 `工作根/merge-and-verify/<日期>-<进程号>`）。
- 闸：「主 agent：工作根（.claude/hooks/lib_work_root.py）下一层、归属文件 .owner 的 session 是本会话的每个目录」（`.claude/hooks/completion-cleanup-guard.sh:20`）；「种类四选一：报告、补丁、备份、交接」（`.claude/hooks/completion-cleanup-guard.sh:84`）。
- 量的（这一次在盘上读的，命令与输出原样）：

```
$ cat ~/.local/share/singlefs/work/merge-and-verify/.owner
cat: merge-and-verify/.owner: No such file or directory
$ find merge-and-verify -maxdepth 2 -name repo -type d | wc -l          （在 ~/.local/share/singlefs/work 下）
13
$ du -sh merge-and-verify
13G	merge-and-verify
```

  大小排在前 12 的 `repo` 在 455M 到 1.3G 之间（`find … -name repo -type d -print0 | xargs -0 du -sh | sort -h | tail -12` 取，量的；第 13 棵更小，没列）；它们是以前留快照的趟留下的，是哪一趟留的我没核（推的）。
- 判：段树只活到第 1 步交出去，一段停下也删，「段树按仓副本列、主 agent 声明不了」那一处没了；快照换成了新的同类对象，主 agent 声明不了（交付物种类里没有它），闸按归属文件认目录而 `merge-and-verify` 目录没有 `.owner`（量的），所以闸是否列它、列不列得全，我读码推是「不列」（推的，没喂闸）。
- 推翻它：喂完工清理闸一个有快照留在 `merge-and-verify/<日期>-<进程号>/repo` 的会话，闸列出了它；或 `merge-and-verify` 目录里出现本会话的 `.owner`。

**甲6 二·2（命令模板过不了范围闸）——消掉了。**
- 定义：「缩域只当写段的快速反馈，判出的结论不算数：副本上的那一条可以缩域，这一行在验证段那一条上要在全域上证过一次」（`.claude/agents/implementation-writer.md:34`）；「缩域只当写段的快速反馈，判出的结论不算数，同一行、同一个目标在验证段第 2 步要在全域上判过一次」（`.claude/agent-common.md:68`）；「超 200000 或估不出就不编不跑、退 79」（`.claude/agents/implementation-writer.md:34`）。
- 钩子：「def target_shrink_variables(root, target, registered):」（`.claude/hooks/lib_run_scope.py:844`）；「设了缩域变量而不是它读的那一个的拒，说明点名该设哪一个」（`.claude/hooks/run-scope-guard.sh:53`）；「拒绝说明的出路点对缩域变量两格」（`.claude/hooks/run-scope-guard.sh:134`，自证格）。
- 消掉它的是：三条模板都写了「缩域变量」与「全域理由」两条路，闸按目标判它读哪个变量；第一轮探针量到的「点到会走崩溃枚举的目标退 2」仍是闸的行为，模板现在把两条路写在命令里。
- 推翻它：照 `.claude/agents/implementation-writer.md:34` 的模板原样写一条点到会走崩溃枚举目标的变异门禁命令，范围声明闸退 2。

**甲7 二·1 与二·3——二·1 仍没打中，二·3 的两处消掉了、第三类换了个地方。**
- 没登记成崩溃枚举用例的测试函数：「目标里没登记成崩溃枚举用例、也不是放量分项的测试函数，按块那一路与放量入口都不起，走第 4 步动到的测试目标那一条 `cargo test`」（`.claude/agents/implementation-writer.md:34`）——消掉了。
- 整趟才跑的类与挂特性的文件（第二轮判决二·2 补采的两形）：「bash scripts/crash/crash-point-replay.sh --item whole-run:<类名> [--item whole-run:<类名>…]」与「bash .claude/gate.d/crash-point-replay.sh --check feature-gated-tests --item feature-gated:<文件名> [--item feature-gated:<文件名>…] <根>」（都在 `.claude/agents/implementation-writer.md:34`）；入口侧「整趟才跑的那几类也是项，写 whole-run:<类>」（`scripts/crash/crash-point-replay.sh:16`），「验证包 tests/ 顶层挂特性 cfg 的一个测试文件」（`.claude/gate.d/crash-point-replay.sh:52`）——消掉了。
- 第三类（产品 GPU 包的真卡对拍）：实现员定义里 `grep -c 'product-gpu-real-card'` 是 0（量的）；合入后验证脚本自己在验证包那一步起它：「补丁改到的产品 GPU 包的测试包 `singlefs-background-gpu-tests` 的真卡对拍逐项交门禁」（`.claude/main-agent.md:57`）——换了个地方（归脚本，不归实现员模板）。第二轮判决只要了前两形，这一条不算漏。
- 推翻它：实现员改了产品 GPU 包的真卡对拍测试，却没有一条命令让它在写段自己点名该格。

**甲8 三·1、三·2 的边角（交回那两句）——消掉了一处，另一处按第二轮判决不改。**
- 「这一行只在删不掉时用，删得掉的照删、不拿它代替」「闸只认自己认领的目录与派发提示点名的草稿目录里的那几份」（都在 `.claude/agent-common.md:51`）——消掉了「没有能删的不许用的禁止句」与「只认认领目录与草稿目录」两处。
- 钩子读法：「只认交付物声明与子 agent 交回正文里」（`.claude/hooks/completion-cleanup-guard.sh:6`）。上游交回闸与看门狗只认「没删」一类词那一处，第二轮判决「三、改法」第 5 条定「不改，记一句」，三份里没有对应改动——还在（按判决不改）。

**甲9 四·1、四·2——四·1 不在这三份里，四·2 消掉了。**
- 四·1：规则文件 `.claude/rules/long-run-split.md` 不在被判三份里，我只看了它此刻有没有那一句：「豁免也按整条任务判（`task-server/task_server.py` 的 `long_run_split_exempt`）」（`.claude/rules/long-run-split.md:34`），服务侧「这条任务在不在 .claude/long-run-split-exempt（用户点名豁免的项）里」（`task-server/task_server.py:5918`）。不判。
- 四·2：`.claude/main-agent.md` 第 33、57、72、118、122 行都写 `singlefs-background-gpu-tests`（`grep -n -o` 取，量的，五处），与「PRODUCT_GPU_PACKAGE_NAME="singlefs-background-gpu-tests"」（`.claude/gate.d/crash-point-replay.sh:803`）一致；`.claude/agent-common.md` 与 `.claude/agents/implementation-writer.md` 里没有旧包名。

### 二·乙 第二轮攻方报告的 R1–R17（`research/prompts/ledger-defs-r2-opus-output.md`）

**R1 登记行只在段树里，第 1 步快照里没有——消掉了。**
- 定义：「新崩溃枚举用例与放量分项的登记行跟着实现员的补丁走」（`.claude/main-agent.md:47`）；「崩溃枚举用例与放量分项的登记行写在副本里、跟着补丁走」（`.claude/agents/implementation-writer.md:15`）；「这一次的登记行跟 `crates/` 的改动一起走」（`.claude/agents/implementation-writer.md:59`）。
- 脚本：「登记表（REGISTRATION_TABLES：.claude/gate.d/stage-inputs.tsv、.claude/gate.d/crash-point-replay-items.tsv、.claude/gate.d/refused-tests.tsv 与 scripts/crash/crash-amplification.sh」（`scripts/gate/merge-and-verify.sh:85`）与「不落测试目标、不退 3，报一句」（`scripts/gate/merge-and-verify.sh:87`）；打补丁脚本对 `crates.patch` 的路径不设限：「touched = touched_paths(patch_directory, include_mutation_files=False)」（`scripts/gate/apply-writer-patch.py:1157`），合成补丁里「登记表的改动一起在里面」（`scripts/gate/merge-and-verify.sh:146`）。
- 消掉它的是：登记行由实现员写进补丁，快照打上补丁就有登记行，第 1 步看到的树与第 2 步是同一棵。
- 推翻它：验证包里新写一条崩溃枚举用例、补丁带登记行，第 1 步快照上 `crash-case-check.py --only placement` 仍报它没登记。

**R2 实现员核新用例登记的命令判的是主工作区——消掉了。**
- 定义：「交回前把 `task-server/task submit --cwd <根> -- python3 scripts/crash/crash-case-check.py --only placement,modules,provenance,must-red <根>` 写进报告」（`.claude/agents/implementation-writer.md:49`）。
- 脚本：打出第 2 步那一条时 `--cwd` 与根都是快照：「crash-case-check.py --only placement,modules,provenance,must-red」（`scripts/gate/merge-and-verify.sh:2663`）；脚本用法行「[仓根]     # 不给 --only 就八样都判」（`scripts/crash/crash-case-check.py:16`）。
- 推翻它：照定义写的命令在仓根起，判的是仓根而不是副本。

**R3 补丁改了包的 `Cargo.toml`，第 1 步退 3、没有路——消掉了；定义里一句「脚本点名的那几个包」对不上脚本（第四部分 D1）。**
- 定义：「退 3（判不出改到哪个测试目标：补丁改了包的 `Cargo.toml`、`build.rs`、不落在包里的路径这类，测试那一步与验证包那一步没跑，别的步绿）照…弹窗，用户同意的在这一趟留下的快照上整包跑脚本点名的那几个包」（`.claude/main-agent.md:57`）。
- 脚本：「if { ((verify_rc == 0)) || { ((verify_rc == 3))」（`scripts/gate/merge-and-verify.sh:4470`，退 3 也进 `merge_in_commands`）；「这一趟退 3：测试那一步与验证包那一步没跑」（`scripts/gate/merge-and-verify.sh:2660`）。
- 消掉它的是：脚本退 3 也打第 2 步与合进主工作区那两段，定义写了退 3 那一支。
- 推翻它：补丁改 `Cargo.toml` 的一趟，汇总行之后没有合进主工作区那一段。

**R4 指纹只罩补丁改到的包——消掉了。**
- 定义同甲2（`.claude/main-agent.md:59`）。脚本：「起点：补丁改到的路径（crates.patch 新旧两侧）往上找到的第一份带 [package] 的 Cargo.toml 所在的包目录，连同工作区里它们照 path 依赖传递依赖的包与传递依赖它们的包」（`scripts/gate/merge-and-verify.sh:163`）；代码「packages = closure(packages, path_dependencies) | closure(packages, lambda directory」（`scripts/gate/merge-and-verify.sh:3011`）。
- 消掉它的是：起点从「改到的包」扩到「依赖闭包两个方向 + 树根三样」。没罩的见甲2。
- 推翻它：只改验证包的补丁，起点里没有它的 path 依赖包（我读码不会，`path_dependencies` 读 dependencies、dev-dependencies、build-dependencies 与 target 下三张表）。

**R5 指纹按最后一趟记——消掉了。**
- 定义：「带 `env MERGE_AND_VERIFY_KEEP_COPY=1` 与 `--fingerprint-of <这一趟的日志目录>`」（`.claude/main-agent.md:57`）。
- 脚本：「--fingerprint-of <日志目录>：开跑先照那一趟记的起点与变异行现算一遍」（`scripts/gate/merge-and-verify.sh:173`）；「每一趟开跑时核过上一趟，最后合进主工作区那一趟的指纹在头一趟的起点上等于头一趟开跑那一刻」（`scripts/gate/merge-and-verify.sh:175`）；打出的重跑命令带它：「if ((carried && $#)); then words+=(--fingerprint-of "$logs"); fi」（`scripts/gate/merge-and-verify.sh:2610`）。
- 推翻它：带 `--item` 的重跑一趟，开跑之后没有核上一趟的指纹（`scripts/gate/merge-and-verify.sh:4412` 的 `if [[ -n "$FINGERPRINT_OF"` 那一段被绕开）。

**R6 核一次合 N 份、热包上核对一再退非 0——消掉了。**
- 定义：「核的那几条连着三趟退非 0（主工作区一直有人在改补丁碰的包）」，出口是 `who-touched.py` 认人、请它们暂停（`.claude/main-agent.md:59`）；「它一次整份写完或一个字不改，没有合到一半」（`.claude/main-agent.md:59`）。
- 脚本：登记表与变异表不进起点：「不进起点：各会话都在往里追加，三方合并在 apply-writer-patch.py 里核（冲突退 2）」（`scripts/gate/merge-and-verify.sh:166`）、「if relative in registration_tables or relative == MUTATION_TABLE:」（`scripts/gate/merge-and-verify.sh:2997`）；核与换之间被改：「换上：先核每份主工作区此刻的 sha256 仍等于记下的补丁前（不等就是核与换之间被别的会话改了，一个字不动、退 2）」（`scripts/gate/apply-writer-patch.py:33`）。
- 消掉它的是三处：变异表不再进起点；N 份并成一份，没有核完到 N 份打完之间的窗口；连着三趟退非 0 有出口。

**R7 段树里的门禁脚本与登记表是建段树那一刻的拷贝——换了个地方（范围缩小）。**
- 定义：「第 1 步绿了，合进主工作区之前在第 1 步最后那一趟留下的快照上验」（`.claude/main-agent.md:58`）：第 2 步用的是开跑那一刻的主工作区加补丁，不再用建段树那一刻的拷贝。
- 剩下的：第 2 步用的门禁脚本是开跑时拷的，指纹起点只有「WORKSPACE_ROOT_ENTRIES = (」那三样与依赖闭包里的包（`scripts/gate/merge-and-verify.sh:2873`、`scripts/gate/merge-and-verify.sh:3012`），开跑之后 `scripts/gate/` 与 `.claude/gate.d/` 里脚本被改，核指纹不报（推的，读码）。窗口从「建段树到合」缩成「开跑到合」。
- 推翻它：喂一趟开跑后改 `.claude/gate.d/crash-point-replay.sh`、补丁不碰的包的样本，核指纹那一条退非 0。

**R8、R9 第 2 步红了修补进不了合入命令、只重交红的罩不住修补弄坏的——消掉了。**
- 定义：「这一步有红的：修补交实现员（`refused-test-registry.py list` 报的照…一节第 2 条办），副本从这棵快照建、修补以它为基线写，修补的补丁目录接在这一批最后，第 1 步连同它整批重交（不带 `--item` 与 `--fingerprint-of`，键相同的照复用），第 2 步那几条在新留下的快照上照原来点的全部项重交（键相同的照复用）；合进主工作区的只有第 1、2 步在同一棵快照上都绿了的那一批」（`.claude/main-agent.md:58`）。
- 消掉它们的是：把「修补改到第 1 步验过的文件才重交」改成「一律整批重交第 1 步、第 2 步点全部项」；合的是第 1 步最后一趟算出的 `merge-patch`，所以修补一定在里面。
- 推翻它：第 2 步红了、修补不碰第 1 步补丁碰过的文件，定义里仍有一句让第 1 步不重交。

**R10 登记行抄两遍、插得进新行改不了旧行——消掉了；拒算那一行换成了一次手做、由第 2 步核。**
- 定义：崩溃用例与放量分项的登记行只在补丁里一处（`.claude/main-agent.md:47`，见 R1）；`insert-row.py` 在 `.claude/main-agent.md` 里 0 处、`copy-originals` 0 处（量的，`grep -c`）。拒算行：「拒算登记表 `.claude/gate.d/refused-tests.tsv` 不写（副本里那一份也不写），由主 agent 逐条判过、收的做成一份只含这张表的补丁目录并进这一批」（`.claude/agents/implementation-writer.md:39`）；「在 `<草稿>` 里 `git diff --no-index --no-prefix --full-index a/.claude/gate.d/refused-tests.tsv b/.claude/gate.d/refused-tests.tsv > <补丁目录>/crates.patch`」（`.claude/main-agent.md:68`）。
- 核它的是第 2 步：「的照第 1 条判，收的照第 1 条补进拒算补丁目录、不收的退给实现员改测试」（`.claude/main-agent.md:69`）。
- 消掉它的是：合表里已有行的改动（第二列加一条流）是 diff 的一段，不再需要 `insert-row.py`；写两遍、转抄的那一步没有了；合表、`stage-inputs.tsv`、放量入口的 `ITEMS` 与补丁同进同出。
- 留下的：拒算行仍是主 agent 手拷三份、追加、出 diff（第三部分第 6 行），做错了由 `refused-test-registry.py list` 核（未登记、已不拒算），不是没有检查。
- 推翻它：补丁带来的合表改动与 `stage-inputs.tsv` 改动在快照上打不上，却在主工作区上能 `insert-row.py` 插进去。

**R11 照留底换回：冲掉别人的追加、证不了新文件删干净——消掉了。**
- 定义：「不是 2 的结局（被停、撞内存上限这类）先交」`--pending-status`，退 3 的用 `--rollback-pending` 或 `--finish-pending`（`.claude/main-agent.md:59`）；留底、`copy-originals.py`、逐份退回都不在三份里了（`.claude/main-agent.md` 里 `copy-originals` 0 处，量的）。
- 脚本：打补丁脚本照内容判：「有被别的改动动过的文件两条都不做（会盖掉别人的改动），列出来交人」（`scripts/gate/apply-writer-patch.py:38`）；「判打全了没有只读这份文件、对着主工作区现算 sha256」（`scripts/gate/apply-writer-patch.py:52`）。
- 消掉它的是：合成一份、一次打，没有合到一半，也就没有留底；没换完的由它自己的暂存目录退回，被别人改过的不动。

**R12 段树寿命与跨轮续做重打已合的补丁——消掉了。**
- 定义：「段树只给写段的下一批当基线：验证段第 1 步那一条交出去就删」（`.claude/main-agent.md:47`）；「验证段里跨轮停下，续做时照 `--read-outcome` 那一条判：退 0 的已合，接合完之后那几样；别的从第 1 步整批重交」（`.claude/main-agent.md:59`）。
- 消掉它的是：合完之后没有段树可留；续做按结局文件判「合没合」，不再把已合的补丁重打进新段树（所以带变异行的补丁重打退 2 的路没有了）。
- 推翻它：合完之后、代码轮判决之前，盘上仍留着本会话建的段树（`ls` 本会话认领目录）。

**R13 与「同一批派、一起打」比（段树加先验后合 对 补丁照旧打主工作区）——还在，没有对应改动。**
- 第二轮判决只定了「两种不同序，哪一种都不是白赢」，并有一条守则「删接口与改调用方的补丁不拆成两批先后打」（`research/prompts/ledger-defs-r2-main-verification.md:35`）；用户定的是丙，不是甲（`research/prompts/ledger-defs-r2-main-verification.md:51`）。三份里「删接口」「改调用方」`grep` 命中 0 行（量的），那条守则没有进现行版。
- 判：还在（形状与第二轮判决一致：两种做法各有没挡住的一半）。推翻它：现行版里出现一句把删接口与改调用方拆批的限制。

**R14 主 agent 手工步里做错了没有检查会红的九处——五处消掉、四处还在（一处部分）。**
- 第 1 处（补丁目录漏给、次序不一致）：还在，没有加检查；后果变小，因为合的是第 1 步给了哪几份就合哪几份：`grep -c 'singlefs-applied-writer-patches' scripts/gate/merge-and-verify.sh` 输出 0（量的），没有任何一处拿段树的打补丁记录与第 1 步的参数比。
- 第 2 处（第 2 步点哪些 `--item`）：还在（部分由脚本打出）：「登记的崩溃枚举用例（门禁 `crash-point-replay` 按块那一路）、崩溃放量分项、formal-verification 带 `--item` 的那几条（点这一段全部补丁改到的项」（`.claude/main-agent.md:58`）仍是人挑；脚本只打出 `crash-case-check`、`refused-test-registry list` 与新写改写的变异行名（`scripts/gate/merge-and-verify.sh:2663`、`:2664`、`:2671`）。
- 第 3 处（第 2 步 `--cwd` 与根写成主工作区）：还在（部分）：脚本打出的几条已带快照路径（`scripts/gate/merge-and-verify.sh:2663`），其余照模板自己填，写成主工作区时形式化与已有用例照绿。
- 第 4 处（第 2 步红了要不要重交第 1 步）：消掉了，一律整批重交（`.claude/main-agent.md:58`）。
- 第 5 处（核指纹跳过）：还在：打补丁脚本不核合入后验证绿没绿、指纹核没核：`grep -c 'fingerprint\|main-tree-at-start\|merge-and-verify\|merge-patch' scripts/gate/apply-writer-patch.py` 输出 1（量的），那一处是注释「合入后验证不在主工作区上编与验」（`scripts/gate/apply-writer-patch.py:96`）。
- 第 6、7、8 处（留底漏掉、回退漏删、登记行转抄）：消掉了，见 R10、R11。
- 第 9 处（第 1 步退 3 自己拼合入命令）：消掉了，见 R3。

**R15 `-- env` 那一条路闸认、服务不拒，但闸认的变量与测试读的不是一回事——消掉了。**
- 钩子：「读哪一个（lib_run_scope.py 的 target_shrink_variables；放行照它判，拒绝说明的出路照它点名）」（`.claude/hooks/run-scope-guard.sh:59`）；「code = text if guard_switch_on("crash-domain-variable-comments-counted") else rust_code_without_comments(text)」（`.claude/hooks/lib_run_scope.py:859`，判读哪一个时注释里的变量名不算）。
- 定义：「自己新写的崩溃枚举用例一律按会走算」（`.claude/agent-common.md:68`）；`<缩域变量>` 的挑法在 `.claude/agents/implementation-writer.md:34`、`.claude/agent-common.md:68`。
- 消掉它的是：出路按点到的目标点名变量，登记的目标与放量分项点放量那个。
- 推翻它：照出路改写的一条 `-- env <变量>=<n>` 命令，变量是点到的目标不读的那一个而仍被放行。

**R16 缩过域的证红与提交时全域那一趟的关系——定义那一半消掉了，变异判定入库那一半换了地方（别的会话在改，不在这一轮）。**
- 定义：「缩域证出的红只当写段的快速反馈，判定不算数，这一行在验证段第 2 步要在全域上证过一次，第 1d 步验证段用的那一条；新测试只靠全量域的计数断言判别的不缩域证红」（`.claude/agents/implementation-writer.md:37`）。
- 脚本：「就是缩过域，run_stage、row-plan 与 scripts/mutation/prove-red.sh 同经这一个函数判」（`scripts/mutation/crates-mutation-rows.py:225`）；`grep -c EXPAND_UP_TO scripts/mutation/crates-mutation-rows.py` 输出 14，`scripts/mutation/lib_mutation_units.py` 输出 0（量的，工作区此刻）。
- 判：写定义的一半消掉；缩过域不入库的一半今天的工作区里有，归管变异脚本的会话（正文说不在这一轮），我只记现状。

**R17 判法给了、没说对着哪棵树；「测试源码读的那一个」多数目标答不出——消掉了。**
- 定义：「做得了的判法：`grep -n '<目标名>' <树>/.claude/gate.d/stage-inputs.tsv <树>/scripts/crash/crash-amplification.sh` 有命中，或 `grep -c EXPAND_UP_TO <树>/crates/singlefs-verification/tests/<目标名>.rs` 不是 0，就会走」（`.claude/agent-common.md:68`）；「写段的命令对着自己的副本判（这一次的登记行已写在副本里），自己新写的崩溃枚举用例一律按会走算」（`.claude/agents/implementation-writer.md:34`）。
- 钩子：「登记表所在的树：从命令的当前目录往上找带 .claude/gate.d/stage-inputs.tsv 的那一层，找不到取 CLAUDE_PROJECT_DIR」（`.claude/hooks/run-scope-guard.sh:62`）。
- 消掉它的是：定义写明对着命令当前目录那棵树判，登记行在副本里，所以写段、第 2 步（快照）、验证两处判得一样；第 3 个难点「读哪一个」由闸按目标判（R15）。
- 推翻它：同一条模板在副本上过闸、在快照上被拒（登记行在两棵树上不同）。

### 二·丙 第二轮判决「二、判决」第 2 条的三处更正（`research/prompts/ledger-defs-r2-main-verification.md:27`）

- 拒算登记那一句只够得着一半：现行写法是「拒算表 `.claude/gate.d/refused-tests.tsv` 只由主 agent 判过再登」那一节整节换过（`.claude/main-agent.md:66`、`:68`、`:69`），消掉了。
- 「三项整类跑法」缺 `whole-run:<类名>` 与 `--check feature-gated-tests` 两形：补上了，见甲7（`.claude/agents/implementation-writer.md:34`）。
- 边角（上游交回闸与看门狗只认「没删」一类词）没有去向：判决「三、改法」第 5 条定不改，三份里没有对应改动，见甲8，还在。

## 三、第二组：同一段实现照现行三份定义逐句数一遍

场景（正文第二组）：两批有依赖的实现员（前一批 P1、P2 互不撞文件，后一批 P3 以前一批改后的样子为基线），共三份补丁；P3 新写一条崩溃枚举用例（要 `stage-inputs.tsv` 一行 `crash-case:`、合表一项）、一条要登记的拒算测试、追加几行变异；没有放量分项与形式化项要点；一路绿（第 2 步不红、代码轮不打中）。

数法（数的，不是跑出来的）：主 agent 在这一段里交的每一条命令、直接跑的每一条只读命令、定义要求的每一次手写（改文件、拷文件、写记录）各算一步；「拷三份」按一步数；两边都有的不数：派发实现员、审 diff、写依赖表、实现员报告里在自己副本上跑的写段命令（测试目标、clippy、build、证红）、每条任务的 `task wait` 与 `task clean`。改前取 HEAD 的三份定义（草稿目录 `head-*.md`）；「第一份报告」指 `~/.local/share/singlefs/work/ledger-b3-round2-fixes/result.md` 第八节「改后」那张 19 行表（我对着它的行重数过一遍：19 行就是 19 步）。

| 行 | 主 agent 做什么 | 改前（HEAD，补丁打主工作区、`--skip-apply` 验） | 第一份报告「改后」 | 现行三份（我数） | 现行落在哪棵树 | 现行依据 |
|---|---|---|---|---|---|---|
| 1 | `git apply --numstat` 核 P1、P2 互不撞文件 | 1 | 1 | 1 | 只读 | `.claude/main-agent.md:47` |
| 2 | 建段树（`minimal-copy.py` 带三条 `--with`） | 0 | 1 | 1 | 段树 | `.claude/main-agent.md:47` |
| 3 | 打 P1、P2（两条 `apply-writer-patch.py`） | 2（进主工作区） | 2（进段树 `--root`） | 2 | 段树 | `.claude/main-agent.md:47` |
| 4 | 打的次序记进调度记录（手） | 0 | 1 | 1 | 调度记录 | `.claude/main-agent.md:47` |
| 5 | 判拒算（手判） | 并在行 12 | 1 | 1 | — | `.claude/main-agent.md:68` |
| 6 | 做拒算补丁目录：拷三份、给 b 追加一行、`git diff --no-index` 出补丁、写 `result.md` | 0 | 0 | 4 | 草稿目录（`a/`、`b/`、`<补丁目录>/baseline/`） | `.claude/main-agent.md:68` |
| 7 | 手写 `stage-inputs.tsv` 一行、合表一项 | 2（写进主工作区） | 0 | 0 | — | HEAD 实现员定义第 49 行（见下面 HEAD 原文）；现行 `.claude/main-agent.md:47` |
| 8 | 合入后验证 | 2（`--skip-apply` P1P2；再 P3） | 1（`KEEP_COPY=1`，P1P2P3） | 1（P1P2P3 加拒算补丁目录） | 快照（主工作区开跑那一刻加四份补丁目录） | `.claude/main-agent.md:57` |
| 9 | 删段树 | 0 | 1 | 1 | 段树 | `.claude/main-agent.md:47` |
| 10 | `crash-case-check.py` | 1（主工作区） | 1 | 1 | 快照 | `scripts/gate/merge-and-verify.sh:2663` |
| 11 | `refused-test-registry.py list` | 1 | 1 | 1 | 快照 | `scripts/gate/merge-and-verify.sh:2664` |
| 12 | 手写拒算行、再交一遍 `list` | 2 | 0 | 0 | — | HEAD 的 `main-agent.md` 拒算一节第 2、4 条 |
| 13 | 崩溃用例门禁 `--item crash-case:<名>` | 1 | 1 | 1 | 快照 | `.claude/main-agent.md:58` |
| 14 | 变异门禁 `--item` 点新变异行（全域） | 1 | 1 | 1 | 快照 | `.claude/main-agent.md:58` |
| 15 | 命名检查两条 | 2（主工作区） | 2（快照） | 2 | 快照 | `.claude/agent-common.md:115` |
| 16 | 核指纹（直接跑；没有要换删的变异行，变异行那一条不打） | 0 | 1 | 1 | 主工作区（只读） | `.claude/main-agent.md:59` |
| 17 | 一条 `apply-writer-patch.py <日志目录>/merge-patch` | 0（已在行 3、8 打过） | 1 | 1 | 主工作区（写） | `scripts/gate/merge-and-verify.sh:2702` |
| 18 | `--read-outcome` 核合全 | 0 | 1 | 1 | 主工作区（只读） | `scripts/gate/merge-and-verify.sh:2706` |
| 19 | mutation-tables 格 | （1，HEAD 定义没写，脚本每次真打都打出这一条命令） | 1 | 1 | 主工作区 | `scripts/gate/merge-and-verify.sh:2444` |
| 20 | 删快照 | 0 | 1 | 1 | 快照 | `.claude/main-agent.md:59` |
| | 合计 | 15（算行 19 为 16） | 19 | 23 | | |

合计是我按表各列加出来的（数的）：改前 1+2+2+2+1+1+2+1+1+2=15；第一份报告 1+1+2+1+1+1+1+1+1+1+1+2+1+1+1+1+1=19；现行 19+4=23。

HEAD 原文所在行（量的，`git show HEAD:<文件> | grep -nF '<句>' | cut -d: -f1`）：

```
.claude/agents/implementation-writer.md  第 49 行  含 由主 agent 或 tooling-writer 登记
.claude/main-agent.md                    第 47 行  含 逐份打进工作树
.claude/main-agent.md                    第 56 行  含 打过的与没打过的补丁分两条命令、依次跑
.claude/main-agent.md                    第 67 行  含 判过的由主 agent 在 `.claude/gate.d/refused-tests.tsv` 加一行
.claude/main-agent.md                    第 69 行  含 登记完再交一遍 `list`
.claude/agent-common.md                  第 113 行 含 由主 agent 在补丁打进主工作区之后经服务交
```

### 三·乙 差在哪几步

1. 现行 23 步对第一份报告的 19 步：差在行 6 一处，4 步。第一份报告把拒算行写成「实现员写进补丁、主 agent 审补丁时二次确认」（该报告第七节末段），所以只有行 5 的手判；第二份报告第一节第 2 条把拒算表改回只由主 agent 登，行 6 的四个动作（拷三份、追加一行、出 diff、写 `result.md`）是这一改带来的，那份报告没有重数总步数。其余 19 行我逐行对过，一一对得上。
2. 现行 23 步对改前（HEAD）的 15 步：净多 8 步（算行 19 则 16 对 23，多 7）。多的 13 步：建段树 1、记次序 1、拒算判单列 1（改前并在行 12 里）、拒算补丁目录 4、删段树 1、核指纹 1、合 1、`--read-outcome` 1、删快照 1、mutation-tables 1（改前定义没写、现行写明）；少的 5 步：手写崩溃用例登记 2、合入后验证两条并一条 1、手写拒算行与再交一遍 `list` 2。
3. 手写与转抄登记：改前 3 次手写（两条崩溃用例登记行、一条拒算行）、没有手记；第一份报告的「手写登记从 8 次降到 0 次」今天只对崩溃枚举用例的登记行成立；拒算行现在是 1 次手追加加手拷三份、手写 `result.md`，另有 1 次手记（行 4）。
4. 落的树：改前，P1、P2 在行 3 就进了主工作区，到行 8 第一条 `--skip-apply` 才验，中间主工作区带着没验过的补丁（HEAD 写段第 2 条写的就是「补丁只打不验」）；现行，主工作区在行 17 之前一个字节没动，行 17 合进去的是行 8 在快照上算出的那一份，第 2 步（行 10–15）在同一棵快照上验。
5. 拒算那一环多出的步：行 6 的 4 步；若第 2 步 `list` 报未登记或已不拒算，再加：重做补丁目录 4、第 1 步重交 1、删被替掉的那一棵快照 1、第 2 步那几条 6（行 10、11、13、14 与行 15 两条）共 12 步（推的）。依据：「拒算补丁目录重做了的，第 1 步连同它整批重交」（`.claude/main-agent.md:69`）、「被下一趟替掉的那一趟留下的快照，下一趟交出去就删」（`.claude/main-agent.md:57`）、「合进主工作区的只有第 1、2 步在同一棵快照上都绿了的那一批」（`.claude/main-agent.md:58`）。第 69 行只写第 1 步重交，没有写第 2 步要不要重做；按第 58 行末句推是要重做（推的）。
6. 行 13 与第 1 步重叠（推的）：第 1 步的验证包那一步已把「跑到的验证包目标里登记的崩溃枚举用例」逐项交门禁（`.claude/main-agent.md:57`，原文「跑到的验证包目标里登记的崩溃枚举用例、挂特性的测试文件与补丁改到的产品 GPU 包的测试包」），键相同第二次全复用；定义仍要第 2 步再交一遍。这一步不是数错，只是有一步多半不增加覆盖。
7. 数里没有的两样：每条会编的任务跑完要 `task clean <号>`（`.claude/main-agent.md:34`），两边一样多，没数；红一次修一次、代码轮打中一次的步数没数（正文第二组限定一路绿）。

## 四、第一组第 1 条顺带读到的：命令今天的脚本不认、或落的树与定义说的不一样

只列我读文件核到的；「推的」是读码推出来的、没喂钩子或脚本。

**D1 退 3 那一支让人「整包跑脚本点名的那几个包」，脚本对两种退 3 不点包名（对不上）。**
- 定义：「退 3（判不出改到哪个测试目标：补丁改了包的 `Cargo.toml`、`build.rs`、不落在包里的路径这类，测试那一步与验证包那一步没跑，别的步绿）照…弹窗，用户同意的在这一趟留下的快照上整包跑脚本点名的那几个包」（`.claude/main-agent.md:57`）。
- 脚本：改到工作区根的文件、不落在任何包里的路径，两种都不写包名：「补丁改到工作区根的 {path}（改的是整个工作区的编法，判不出改到哪个测试目标）」（`scripts/gate/merge-and-verify.sh:1840`）、「补丁改到不落在任何包里的 {path}（测试运行时可能读它，判不出归哪个测试目标）」（`scripts/gate/merge-and-verify.sh:1844`）；只有「补丁改到 {package} 包里的 {relative}，判不出改到哪个测试目标（不是测试目标的根文件、不在 src/ 下，tests/ 下也没有测试目标 mod 它）」点名了包（`scripts/gate/merge-and-verify.sh:1864`）。
- 本场景不触发（三份补丁都落得到测试目标）；退 3 那一支的第一种情形（根 `Cargo.toml`）没有「脚本点名的包」可整包跑，要主 agent 自己定包。

**D2 缩过域那一句让人给「那一条」加 `env`，脚本打出的重跑命令用的是 `--crash-expand-up-to`（对不上）。**
- 定义：「给了 `--crash-expand-up-to <n>`（或环境里设了 `SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO`）缩过域的，那一条在 `task-server/task submit --` 之后多一段 `env SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n>`」（`.claude/main-agent.md:57`），同一段前面又写「主 agent 照原样交，别的不改写」。
- 脚本：不带 `--skip-apply` 的那一趟，打出的重跑是合入后验证本身，缩域用参数交：「words+=(--crash-expand-up-to "$expand")」（`scripts/gate/merge-and-verify.sh:2607`）；`env <变量>=<n>` 那一种只在起门禁的那一条（带 `--skip-apply` 的路）里写（`scripts/gate/merge-and-verify.sh:1194` 一带），自证还核了不带 `--skip-apply` 的路不出现它（`scripts/gate/merge-and-verify.sh:3898`）。
- 后果：照定义加 `env` 并不会错（变量会重复设同一个值），但与「照原样交」自相矛盾；现行主路径是不带 `--skip-apply` 的，这一句描述的是带 `--skip-apply` 那一支（推的）。

**D3 拒算补丁目录打不上时，脚本的出路写的是「以新段树为基线重做」，对不上这个补丁目录（对不上）。**
- 脚本出路：「这一份退给实现员以新段树为基线重做」（`scripts/gate/merge-and-verify.sh:2428`）。定义：「打补丁核不过（脚本整批停、退 2）的，照它出路里段树那一句办」（`.claude/main-agent.md:57`）；而拒算补丁目录是主 agent 自己从主工作区此刻的拒算表做的，不是实现员出的、也没有段树基线：「拒算补丁目录要加行、删行的，照这一条从主工作区此刻重做一份，替掉这一批里原来那一份」（`.claude/main-agent.md:68`）。
- 触发：做拒算补丁目录之后、第 1 步开跑之前，别的会话往 `refused-tests.tsv` 末尾也追加了行（两边都追加在同一处，三方合并会冲突；推的）。这时脚本的出路文字指向实现员与段树，定义第 68 行指向自己重做，两句不是同一个办法。没有段树的一批（没有依赖）出路里仍写「照打进段树的次序给」，也对不上。

**D4 两份补丁各登记一条崩溃枚举用例时，`git apply --numstat` 那一步没有说法（定义没写，本场景不触发）。**
- 「`crates/mutations.tsv` 只追加、不算撞文件」（`.claude/main-agent.md:27`）只给了变异表；实现员定义把另三张登记表的改动并进 `crates.patch`（`.claude/agents/implementation-writer.md:59`），而派发闸按「要动的 crates 文件」判撞不撞（「派发闸按这一行判与在跑的实现员、工具员撞不撞文件」，`.claude/agents/implementation-writer.md:26`），表不在其中。第 47 行「`git apply --numstat` 核几份互不撞文件」遇到 P1、P2 都改 `stage-inputs.tsv` 时，字面上它们撞了，定义没写怎么办；两份在表尾追加时三方合并会冲突（推的）。

**D5 命名检查的 `<根>` 写 `<快照>`，交回闸认的根是仓根（定义对钩子，推的，没喂闸）。**
- 定义：「在副本里改、交补丁的，`<根>` 写 `<快照>`，由主 agent 在验证段第 2 步在第 1 步留下的快照上经服务交」（`.claude/agent-common.md:115`）。
- 闸：minimal-copy 建的副本按仓根算：「副本根 .git 是 git init 的目录、没登记成 worktree；这两种都按仓根算」（`.claude/hooks/handback-guard.sh:39`）；「共用约束写明 rsync 拷的副本 <根> 填仓根」（`.claude/hooks/handback-guard.sh:45`）；报告里那两条命令的根按这个正则取：「[^\s`'\"|;&()<>，。；、]+」（`.claude/hooks/handback-guard.sh:418`，字符类排除了 `<` 与 `>`）。
- 读码推：实现员照定义把 `<快照>` 写进「要主 agent 跑的命令」一节，交回闸取不到根，判命名检查没写、拒交回；写成仓根才过闸，但那样与定义「写 `<快照>`」不一致，主 agent 在第 2 步还得自己换成快照路径。
- 推翻它：喂交回闸一份「要主 agent 跑的命令」一节里命名检查两条的根写 `<快照>` 的报告（改了 `.rs` 的会话记录配套），闸放行。

**D6 结局只写了四支，脚本另有退 75（定义没写，本场景不触发）。**
- 定义：「结局分四支：」（`.claude/main-agent.md:57`）。脚本：「75 按条复用那一层连不上 prune-server（admission.py plan-failure 判不可用），别的没有红；有红退 1，判不出测试目标退 3。」（`scripts/gate/merge-and-verify.sh:203`）与「带 --skip-apply 的那一趟（补丁已在主工作区上）与退 1、2、75 的都不打这几条」（`scripts/gate/merge-and-verify.sh:161`）。退 75 时汇总行之后没有第 2 步与合进主工作区那两段，定义没写这时怎么办（照服务回来整批重交是我的推，定义里没有）。

**D7 打补丁脚本文件头仍写 `crates.patch` 只含 crates、litmus（脚本头对不上，行为不受限）。**
- 头：「`git diff -- crates litmus ':!crates/mutations.tsv'` 的输出；没有代码改动可以没有」（`scripts/gate/apply-writer-patch.py:15`）；定义把三张登记表与放量入口的改动并进 `crates.patch`（「这一次的登记行跟 `crates/` 的改动一起走」，`.claude/agents/implementation-writer.md:59`）、拒算补丁目录的 `crates.patch` 只含 `.claude/gate.d/refused-tests.tsv`（`.claude/main-agent.md:68`）。
- 代码不限路径：「touched = touched_paths(patch_directory, include_mutation_files=False)」（`scripts/gate/apply-writer-patch.py:1157`）之后只核 `crates/mutations.tsv` 与 checker 一节，没有按目录前缀拒的分支（我读 :1150–1180，量的）。头那一句只是旧写法。

**D8 副本取补丁基线的两条命令在 minimal-copy 副本里取不到（改前就有，待查，推的）。**
- 实现员定义：`crates.patch` 用 `git diff -- crates litmus …`，基线「在副本里 `git show :<路径>` 取」（`.claude/agents/implementation-writer.md:59`，HEAD 实现员定义第 59 行同句：`git show HEAD:.claude/agents/implementation-writer.md | sed -n 59p` 读到两条命令，量的）；minimal-copy 的头写明「副本里没有提交、索引是空的」（`scripts/hosts/minimal-copy.py:31`）。空索引下 `git show :<路径>` 取不到、`git diff` 不列没跟踪的文件。
- 以前实现员的报告里有副本 `git diff --stat` 的输出，我没查它们怎么建的基线，所以只记待查，不判。

**D9 留下的快照不在完工清理闸的单子里（见甲5）。** 这一条定义没写它在不在单子上；量的与读码见甲5。

**认得的命令（我对着脚本头核过用法，都认）：**
- 「refused-test-registry.py list [仓根] [--plan <文件>] [--admission <脚本>]」（`scripts/gate/refused-test-registry.py:12`）；「[--item …] [项目根]」形的变异门禁点名行（`.claude/gate.d/crates-mutation-replay.sh:28`）；「crash-amplification.sh --item <分项> [--item <分项>]」（`scripts/crash/crash-amplification.sh:11`，没有根参数，靠 `--cwd`）；「--crash-cases-only：只与点名崩溃枚举用例的 --item 同给」（`.claude/gate.d/crash-point-replay.sh:42`）；「--pending-status [--root 仓根]」（`scripts/gate/apply-writer-patch.py:8`）、「--read-outcome <结局文件> [--root 仓根]」（`scripts/gate/apply-writer-patch.py:11`）；`--fingerprint-of` 在合入后验证用法行里（`scripts/gate/merge-and-verify.sh:11`）。
- 判范围声明的闸对这几条都不要求缩域：合入后验证、崩溃点重放点名 `crash-case:<用例>`、崩溃放量点名分项「判不到的」（`.claude/hooks/run-scope-guard.sh:63`），变异门禁点名会走崩溃枚举的行才要缩域或全域理由。

## 五、要主 agent 跑的命令

没有。这一轮我只读文件、读码、数步数；没有起脚本、模型或装置，没有写模型目录。要核的几处写在这里，都是读码推的，真要坐实得由主 agent 喂：
- D5：造一份改了 `.rs` 的实现员会话记录，配一份「要主 agent 跑的命令」一节里命名检查两条的根写 `<快照>` 的报告，喂 `.claude/hooks/handback-guard.sh`，看放行还是拒。我没造，写「复核不了」。
- 甲5：在一个留着快照的会话里喂 `.claude/hooks/completion-cleanup-guard.sh`，看单子里有没有 `merge-and-verify/<日期>-<进程号>/repo`。我没喂，写「复核不了」。

### 补：第二部分里没单写推翻条件的几条

- R6：第 1、2 步之间别的会话在补丁碰的包里改一行不重叠的代码，核指纹仍退 0（它该退非 0）；或别人往 `crates/mutations.tsv` 追加一行就让核指纹退非 0（它不该）。
- R11：合到一半被停，`apply-writer-patch.py --pending-status` 退 0 而主工作区里已有文件被换上。
- R14：跳过核指纹，直接交 `apply-writer-patch.py <日志目录>/merge-patch`，脚本拒（今天不拒：上面 `grep -c` 输出 1，那一处只是注释）。
- 甲9：`.claude/main-agent.md` 里再出现不带 `-tests` 的 `singlefs-background-gpu`。
- R16 的入库一半：缩域证红判「抓到」的行，提交时 `crates-mutation-replay` 照键复用、没在全域上判（今天工作区里 `scripts/mutation/crates-mutation-rows.py` 写的是缩过域不入库，我没跑）。

## 六、判定一览

R1–R17 里，数的：消掉 13 条（R1、R2、R3、R4、R5、R6、R8、R9、R10、R11、R12、R15、R17）、换了个地方 1 条（R7）、半消掉半换地方 1 条（R16）、还在 1 条（R13）、混合 1 条（R14：五处消掉、四处还在）。

| 格 | 判定 | 一句话 |
|---|---|---|
| 甲1 一·3 验证段次序打架 | 消掉了 | 各处现在都写同一个次序：快照上验、第 2 步同一棵快照、都绿了才合 |
| 甲2 一·2 三棵树不一样 | 换了个地方 | 只剩一棵快照加开跑指纹；指纹不罩 `scripts/`、`.claude/gate.d/` 脚本与登记表里别人的追加 |
| 甲3 一·1 走一遍的五处 | 消掉了 | 合成一份一次打、建段树命令同一条、第 2 步重交写了、登记行进补丁、mutation-tables 写明；D1 一句对不上 |
| 甲4 一·4 代码轮快照 | 消掉了（主句仍没打中） | 第 2 步用第 1 步的快照，不再需要段树做修补 |
| 甲5 一·5 段树与完工清理闸 | 换了个地方 | 段树只活到第 1 步交出去；快照放在没有 `.owner` 的 `merge-and-verify/` 下，闸读码推是不列它（量的：13 棵 `repo` 在盘上） |
| 甲6 二·2 模板过范围闸 | 消掉了 | 三条模板写了缩域变量与全域理由两条路，闸按目标点名变量 |
| 甲7 二·1 与二·3 | 消掉了，第三类换了地方 | 没登记的测试函数、`whole-run`、`feature-gated` 都进了模板；产品 GPU 真卡对拍归脚本自己起 |
| 甲8 三·1、三·2 边角 | 消掉一处、还在一处 | 「交主 agent 删」只在删不掉时用已写；交回闸与看门狗只认「没删」按判决不改 |
| 甲9 四·1、四·2 | 不在这三份里 / 消掉了 | 规则文件里已有按整条任务豁免那一句；五处包名都带 `-tests` |
| R1 | 消掉了 | 登记行跟补丁走，快照里就有 |
| R2 | 消掉了 | `crash-case-check` 带根，脚本打出的那条用快照 |
| R3 | 消掉了 | 退 3 也打第 2 步与合入两段；一句「脚本点名的包」对不上（D1） |
| R4 | 消掉了 | 指纹起点扩到依赖闭包两个方向加树根三样 |
| R5 | 消掉了 | `--fingerprint-of` 链，最后一趟的指纹等于头一趟开跑那一刻 |
| R6 | 消掉了 | 一次合成一份、变异表不进起点、连着三趟退非 0 有出口 |
| R7 | 换了个地方 | 门禁脚本不再是建段树那一刻的拷贝，但仍是开跑那一刻的拷贝，不在指纹里 |
| R8、R9 | 消掉了 | 第 2 步有红一律整批重交第 1 步、第 2 步点全部项 |
| R10 | 消掉了 | 登记行只在补丁里一处；拒算行仍是一次手做，由 `list` 核 |
| R11 | 消掉了 | 没有留底与逐份退回，没换完的用脚本自己的 `--rollback-pending` |
| R12 | 消掉了 | 段树第 1 步交出去就删，跨轮按 `--read-outcome` 判 |
| R13 | 还在 | 没有对应改动，两种做法各有没挡住的一半（按用户定丙） |
| R14 | 五处消掉、四处还在 | 第 1、2、3、5 处没有检查兜着（第 2、3 处部分由脚本打出） |
| R15 | 消掉了 | 闸按目标判读哪个缩域变量，注释里的变量名不算 |
| R16 | 半消掉半换地方 | 定义写了「缩域只当快速反馈」；变异判定入库归别的会话 |
| R17 | 消掉了 | 定义写明对着命令当前目录那棵树判，给了 `grep` 判法 |
| 第二组 现行步数 | 23 步 | 数的：现行三份下同一段 23 步（行 1–20 表，算 mutation-tables 与拒算补丁目录） |
| 第二组 对第一份报告 | 差 4 步 | 差在拒算补丁目录的做法（拷三份、追加、出 diff、写 `result.md`）；其余 19 行对得上 |
| 第二组 对改前 | 15 步（算行 19 为 16） | 净多 8 步；手写登记 3 次对现行 1 次手追加；改前 P1、P2 在验之前就在主工作区里 |
| 第二组 拒算环 | 多 4 步，重做多 12 步（推的） | 第 69 行没写重做时第 2 步要不要重做，按第 58 行末句推是要重做 |
| D1 | 对不上 | 退 3 让人整包跑「脚本点名的包」，两种退 3 脚本不点包名 |
| D2 | 对不上 | 缩过域那一句让人加 `env`，不带 `--skip-apply` 的重跑命令用 `--crash-expand-up-to` |
| D3 | 对不上 | 拒算补丁目录打不上时脚本出路指向实现员与段树 |
| D4 | 定义没写 | 两份补丁各登记一条崩溃用例时表撞文件怎么办，本场景不触发 |
| D5 | 对不上（推的） | 命名检查 `<根>` 写 `<快照>`，交回闸认仓根，这一条没喂闸 |
| D6 | 定义没写 | 退 75 不在四支里，本场景不触发 |
| D7 | 脚本头对不上 | `crates.patch` 头写只含 crates、litmus，代码不限路径 |
| D8 | 待查（改前就有） | minimal-copy 副本索引为空，`git show :<路径>` 取基线要实现员自己想办法 |
| D9 | 见甲5 | 快照不在完工清理闸单子里 |

## 七、没做什么

- 不判攻方腿那几组（第一组第 1、3、4、5 条的逐步走一遍、拒算环会不会绕圈、快照多大怎么清、合成补丁带不带副产物、grep 判法对不对）；只在去向里顺带记了读到的事实，没替主 agent 采纳或出判决。
- 没往统一任务服务交任务，没跑任何脚本、钩子、自证与开关；判决要引的数（步数是数的、13 棵 `repo` 是量的、`grep -c` 是量的）由主 agent 复跑；D5、甲5 的闸行为是读码推的，没喂闸。
- 没写模型、没起装置；没读 `crates/` 下的产品代码；没读禁读清单里的文件，也没读 `d137-ledger-defs-r1-*` 一类名字相近的另一题。
- 只判正文点名的句子；三份里别的会话的改动没判。
- 草稿目录里只有 `head-*.md` 三份 HEAD 拷贝与几份 `cite*.txt` 输出，没有仓副本与编译目录，不需要删。

## 八、cite-check 末行

```
$ python3 scripts/agents/cite-check.py research/prompts/ledger-defs-r3-sonnet-output.md
  ✓ 核了 117 处引文、53 处裸行号，对上 117 处，没判 53 处（其中裸行号没有引文可比 53 处；逐处列在上面）
```
