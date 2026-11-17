# ledger-defs-r1 攻方腿（opus）报告

日期：2026-11-12。被判的六份与开工快照 `research/prompts/ledger-defs-r1-snapshot/judged-sha256.txt` 开工时 `sha256sum -c` 六份全 OK（量的，本腿开工时跑）。

## 复跑命令与模型文件

这一腿写了一个小探针（喂范围声明闸的 hook 输入，照主 agent 判，核退出码），不是计数模型。模型目录 `research/prompts/ledger-defs-r1-opus-model/`；复跑命令（交主 agent 跑，见文末「要主 agent 跑的命令」）：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/ledger-defs-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/ledger-defs-r1-snapshot /home/fy5090/.local/share/singlefs/work/ledger-defs-r1-opus-probe-run /home/fy5090/code/singlefs/research/prompts/ledger-defs-r1-opus-model
```

各文件 sha256（量的，`find . -type f | sort | xargs sha256sum` 在模型目录里跑）：

```
74253dfa591c25d48606b696a096c62ca3e88e7f305562d2e4eb0ab7b8fb65fa  ./expected.tsv
9ea414f42f4cffe2e7dd1122224dd2deeac306915791b10e737b117fdde0dc69  ./inputs/amplification-item.json
29a398200288816a40d7bf8cb121aa83fc5adf5834379ee27d0cf09655174d25  ./inputs/cargo-test-on-a-crash-target.json
ccc6d084f497d93c84a4f2234d506ac8a7db624454dde8d58f7b624c0bb081ba  ./inputs/crash-case-through-the-gate.json
8a14976ad5ba6a66ef1b9031570a2171d25a35a2366db27c83dd9ef485f97ba1  ./inputs/merge-and-verify-three-patches.json
00b2124e8aa2861d34aba685bc363dd5ef3d9df9869ca6cef22c0c172842d91f  ./inputs/mutation-row-on-a-crash-target.json
a6755d58117bcb023c89c10eacdda6440f81c185d26063690db4485185d137c7  ./inputs/mutation-row-on-a-crash-target-with-full-domain-reason.json
7df9778c87bd7a22505e5b4bd036a92bd339650eabc1109324e367eb2c36994b  ./inputs/old-entry-item-on-the-same-flow.json
11bc28426a9f1be6150b2885ad11795ceb29b3c824ba474d9e94345e7f54d785  ./inputs/prove-red-on-a-crash-row.json
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  ./tools/rerun-from-a-clean-work-directory.sh
41cc3220ad790d839a6c12ac8e22a247e5742fe2c2dfd9c97d4d3319cf553a1f  ./tools/rerun.sh
```

`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：把 `inputs/` 里每份 hook 输入（`@REPO@` 换成 `--repo`）在 `setsid --wait timeout -k 5 60` 里喂给仓里的 `.claude/hooks/run-scope-guard.sh`（`AGENT_HOOK_DETECTIONS` 指到运行目录，不进默认检出记录），退出码与 `expected.tsv` 逐行比，末行 `name=probe_summary samples=8 mismatches=<M>`，M 为 0 才退 0。探针还没跑：下面引它的期望退出码都是「推的，待主 agent 跑」。

另做过一次直接跑的复现（不经服务，共用约束许 `minimal-copy.py` 直接跑）：照 `.claude/main-agent.md` 写段第 2 条的命令原样建段树，再从段树建实现员副本，两次都退 0（量的，输出在草稿目录 `segtree-build.txt`、`writer-copy-build.txt`；合计 123.7 MiB、11154 个文件，`MINIMAL_COPY git` 行写「在副本根 git init 了（没有提交、索引是空的）」）。两份副本交回前已删（见「没做什么」）。

## 各格判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| 一·1 走一遍、有没有没验过的补丁进主工作区 | 打中（G1-A、G1-B、G1-D、G1-F；G1-K 推的） | 三处句子叫人在合入后验证一绿就合、`--item` 那几条排在合之后；合到一半退 2 没有去向；崩溃枚举用例的登记不跟补丁走；第 2 步红了没有重跑写法 |
| 一·2 三棵树不一样时哪一步的绿不作数 | 打中（G1-C） | 只写了「核不过（冲突）」那一支；不冲突而内容分岔时第 2 步段树上的绿与第 1 步快照上的绿各自不罩合进去的那棵，定义没写 |
| 一·3 还照旧做法写的句子 | 打中（G1-A、G1-G、G1-H） | `.claude/main-agent.md:7`、`:33`、`:118`、`:65`，`.claude/agents/implementation-writer.md:34`（变异证红对着主工作区、合之后跑） |
| 一·4 代码轮快照取在全部合入之后 | 没打中主句；连带打中 G1-E | 合在第 2 步绿之后、代码轮在合之后，这一句成立；但代码轮打中之后的修补（以主工作区为基线）没有段树可做第 2 步 |
| 一·5 段树跨轮与完工清理闸 | 打中（G1-J） | 段树是仓副本，主 agent 只能声明报告、补丁、备份三种；一段没做完就停下等用户时收工被拦，定义没写怎么办；看门狗 6 小时没动的副本也报给主 agent 删 |
| 二·1 还叫人交崩溃点重放入口带 `--item` 的句子 | 没打中 | 六份里提到入口的两处是钩子说明，与钩子今天判的相符；项数写法与 `run-scope-guard.sh:65` 对得上 |
| 二·2 模板照抄过不过范围闸两判 | 打中（G2-2） | 改的两条（按块用例、放量分项）过得了，是因为钩子不判它们、门禁按块那一路也不估域，`verification.md:57` 的 20 万量级不再有东西管；放量入口没缩域自己退 79；同一份里没改的三条模板（变异门禁 `--item`、证红、`cargo test` 点名验证包目标）点到崩溃枚举目标照拒，六份里「全域理由」出现 0 次 |
| 二·3 旧写法点得到、新写法点不到 | 打中（G2-3） | `whole-run:<类>` 三项与每项「测试函数那一段」；GPU 那一类要走 `--check feature-gated-tests`，定义的模板里没有 |
| 三·1 两句与钩子逐字对不对得上 | 没打中（两处小出入列在 G3-1） | 目录种类、一字不差、8 个字、主 agent 一侧照列都对得上 |
| 三·2 会不会读成「不想删就写一行」 | 没打中文字；记一处管不到（G3-2） | `.claude/agent-common.md:51` 写明只在删不掉时用，钩子不核删没删过 |
| 四·1 门禁管哪一半三条与实现 | 打中（G4-1） | 服务按「命令里有没有豁免项名这个词」判整条任务豁免，规则只写了「按片或整条任务的用时判」那一半 |
| 四·2 换包名之后同说一件事的旧说法 | 弱打中（G4-2） | `.claude/main-agent.md:118`、`:72` 还写「产品 GPU 包」不带「测试包」；`.claude/rules/` 下没有旧包名 |

## 一、写段专用的一棵树，验证段先验后合

走法：一段两批有依赖的实现员，第一批两个（补丁 P1、P2，互不撞文件），第二批一个（P3，以打了 P1、P2 的段树为基线）。段树照写段第 2 条建（实测建得成，见上）；P1、P2 照 `task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录> --root <段树>` 打进段树（`--root` 不是主工作区时脚本只拿副本锁、不查开工快照：`scripts/gate/apply-writer-patch.py:114`：「副本上不查（副本不是那一轮取快照的工作区）。」）；第二批实现员从段树建副本（实测建得成）。写完进验证段：第 1 步 `merge-and-verify.sh P1 P2 P3`（快照取主工作区，`scripts/gate/merge-and-verify.sh:29`：「拷完在快照根 git init（不加文件、不提交）」），第 2 步在段树上交带 `--item` 的几条，再合。

### G1-A 三处句子叫人第 1 步一绿就合，`--item` 那几条排在合之后（一·1、一·3）

- 第 2 步的新句：`.claude/main-agent.md:58`：「在合进主工作区之前交，`--cwd` 与命令里的 `<根>` 都写段树（写段的补丁都已打在它上面）；这一步绿了才照第 1 步打出的那一段命令合进主工作区。」
- 同一份里叫人第 1 步绿了就合的句子：
  - `.claude/main-agent.md:57`：「这一条绿了（末行 `TASK_SERVER_RESULT` 是 `exit=0`），照它在汇总行之后打出的「合进主工作区」那几条命令」——第 1 步自己这一句就把合写在第 1 步里，读到第 2 步之前已经合了。
  - `.claude/main-agent.md:7`（职责）：「绿了照它打出的命令逐份合进主工作区，红了才派实现员改钉值」——没有第 2 步。
  - `.claude/main-agent.md:33`（第 10 条）：「脚本在主工作区的快照上打与验，绿了照它打出的命令逐份合进主工作区」，同一行后面才是「再由主 agent 经服务交登记的崩溃枚举用例（门禁 `crash-point-replay` 按块那一路）、崩溃放量分项、formal-verification、crates-mutation-replay 带 `--item` 的那几条」——次序是先合后交，也没写对着段树。
  - `.claude/main-agent.md:118`（派发表）：「绿了照它打出的命令逐份合进主工作区；各实现员报告里带 `--item` 的门禁与崩溃放量入口由主 agent 在各自的副本里经服务交过」，同一格后面才写交 `--item` 那几条——同样先合后交。
- 今天的脚本怎么做：`scripts/gate/merge-and-verify.sh:139`：「再照给的次序每份补丁一条 task-server/task submit -- python3 scripts/gate/apply-writer-patch.py <补丁目录>（不带 --root：打进这份脚本所在的主工作区）。」这几条在第 1 步验绿时就打出来，脚本不知道有第 2 步，没有任何东西挡着先合。
- 结果：照 `:7`、`:33`、`:57`、`:118` 任一句做，P1–P3 进了主工作区，formal-verification、crates-mutation-replay、崩溃放量分项（`merge-and-verify.sh` 的验证包那一步不起放量分项：`scripts/gate/merge-and-verify.sh:180`：「放量分项（scripts/crash/crash-amplification.sh 的 ITEMS）经 --skip 排掉之后这里不起，归提交时。」）在主工作区上一条都没判过——主工作区上出现了没验完的补丁。
- 四句：分辨臂——定义轮没有臂，不适用；判别子——主 agent 在第 1 步验绿那一刻手里只有脚本打的合入命令，看不到「第 2 步还没跑」的提示，判不出该停；字面分句——满足「有没有哪一步主工作区上出现没验过的补丁」与「同一份定义里有没有句子还照旧做法写」两句；改法——`:7`、`:33`、`:57`、`:118` 四处都改成「第 1 步绿 → 第 2 步在段树上绿 → 再照第 1 步打出的命令合」（只在我的推演上、被攻过零轮）；只改 `:58` 一处碰不到这几格。

### G1-B 合到一半有一份退 2，主工作区停在没验过的组合上（一·1）

- 定义：`.claude/main-agent.md:57`：「（前一条退 0 才交下一条），合完照写段第 2 条删段树」。只写了「退 0 才交下一条」，没写中间一份退 2 怎么办。
- 合进主工作区的 `apply-writer-patch.py`（不带 `--root`）会在这几种情形退 2、一个字不改：
  - 别的会话在这一段写的几个小时里开了一轮三方、快照里有 P2 改的文件：`scripts/gate/apply-writer-patch.py:111`：「落在还没判完的三方轮的开工快照里就拒、退 2、一个字没改。」（打进段树、打进快照时都不查，`:114`，所以这一种只在最后合的时候才冒出来。）
  - 第 1 步之后主工作区又被改了同一段：`scripts/gate/apply-writer-patch.py:104`：「冲突：git merge-file 报冲突的文件，逐个点名」。
  - 别的会话同一时刻也在往主工作区打补丁：`scripts/gate/apply-writer-patch.py:40`：「同一个仓同一时刻只许一个在换」。
- 结果：P1 已进主工作区、P2 退 2、P3 照「退 0 才交下一条」没交。主工作区此刻是「主工作区 + P1」，第 1 步验的是 P1+P2+P3、第 2 步验的也是 P1+P2+P3，这个组合哪一步都没验过；P3 依赖 P2 的话主工作区可能连编都编不过。脚本给了留底（`scripts/gate/merge-and-verify.sh:137`：「先一条 python3 scripts/agents/copy-originals.py copy <日志目录> <补丁动到的文件…> 留底」），定义没写什么时候用它退回、退回之后段树怎么办。
- 四句：判别子——退 2 的输出点名了哪一份，主 agent 看得到；缺的是定义里的去向。改法——在 `:57` 写「中途有一份退 2：照留底把已合的几份退回，按退 2 的原因（快照冲突等判决 / 三方合并冲突照 `merge-and-verify.sh` 文件头段树那一句重建段树）整批重来」（零轮）。

### G1-C 三棵树不一样、又没冲突时，哪一步的绿不作数（一·2）

- 三棵树：第 1 步的快照 = 第 1 步开跑那一刻的主工作区 + P1..P3；第 2 步的段树 = 段树建好那一刻的主工作区 + P1..P3；合进去的 = 合的那一刻的主工作区 + P1..P3（不冲突时三方合并静默合上：`scripts/gate/apply-writer-patch.py:100`：「全部合得上的，在临时仓里照「主工作区此刻 → 合后」重出一份补丁」）。
- 定义只写了冲突那一支：`.claude/main-agent.md:57`「打补丁核不过（脚本整批停、退 2）的，照它出路里段树那一句办」，脚本那一句只管主工作区改了补丁改的同一段这一种（`scripts/gate/merge-and-verify.sh:148`：「会停的是主工作区在段树建好之后又改了补丁改的同一段」）。
- 走法：段树建好之后别的会话改了 `crates/singlefs-checker/src/` 里一条不变量的判定（不碰 P1..P3 的文件）。第 1 步在新 checker 上绿；第 2 步在段树（旧 checker）上跑 crates-mutation-replay、formal-verification、放量分项也绿；合进去的那棵是新 checker + P1..P3，放量分项、formal-verification、变异行在这棵树上一次都没判过——第 2 步的绿不罩它。第 1 步与合之间（第 2 步要跑几个小时）主工作区再被改，第 1 步的绿也不罩合进去的那棵。
- 定义里没有：第 2 步开跑前核段树的基线与主工作区此刻是否一致、不一致就重建段树（或第 2 步改在第 1 步的快照上跑），合之前核主工作区与第 1 步的快照基线一致。
- 四句：判别子——主 agent 在第 2 步开跑那一刻能看得到差异（`diff` 段树与主工作区里补丁没碰的文件），定义没叫它看；改法——零轮，同上一句。

### G1-D 新崩溃枚举用例的登记不跟补丁走，落哪棵树定义没写（一·1）

- 走法：第二批实现员 P3 新写一条崩溃枚举用例。定义：`.claude/agents/implementation-writer.md:49`：「`.claude/gate.d/` 下的两张表与 `scripts/crash/crash-amplification.sh` 的 `ITEMS` 由主 agent 或 tooling-writer 登记」。补丁只装 `crates/` 与 `litmus/`：`scripts/gate/apply-writer-patch.py:15`：「crates.patch                 `git diff -- crates litmus ':!crates/mutations.tsv'` 的输出；没有代码改动可以没有」。
- 登记写进主工作区：段树里没有这一行，第 2 步在段树上交 `--item crash-case:<新用例>`，门禁照段树的表列项（`.claude/gate.d/crash-point-replay.sh:173`：「--list-items               逐行打可点名的项，不起 cargo：被判那棵树 stage-inputs.tsv 里键是 crash-case: 的行与它们的 test=」），项写错退 2（`.claude/gate.d/crash-point-replay.sh:58`：「项写错（没登记的用例、tests/ 里没有挂特性的这个文件）退 2」）。主工作区里则先有了登记、测试函数要等合进去才有——又一份没验过的改动先进了主工作区。
- 登记写进段树：合进主工作区的命令只有每份补丁一条 `apply-writer-patch.py`（G1-A 引的 `merge-and-verify.sh:139`），段树里的表改动没有任何一条命令带回主工作区；第 1 步的快照取自主工作区，没有这一行，新用例不走按块那一路，在测试那一步当普通测试函数跑。
- 定义里没写：登记改动在写段落到哪棵树、验证段第 1 步与第 2 步各用哪一份、合的时候怎么带进主工作区。

### G1-E 合进主工作区之后的修补与 `--skip-apply` 的改动没有段树可做第 2 步（一·1、一·4、一·5）

- `.claude/main-agent.md:47`：「段树照「一轮怎么开、怎么收」第 11 条用完就删：这一段的补丁照验证段第 1 步都合进主工作区之后删。」第 2 步只写段树（G1-A 引的 `:58`）。
- 走法一（代码轮打中）：合完、删段树、开代码轮，判决打中。`.claude/main-agent.md:60`：「已合进主工作区之后才红的（代码轮判决打中这类），修补以主工作区为基线写，第 1 步那一条只给这一批修补的补丁目录；绿了都照第 1 步合进主工作区」——这时段树已删，第 2 步「`--cwd` 与 `<根>` 都写段树」没有树可写；照字面要么跳过第 2 步直接合（formal-verification、变异行、放量分项没判就进主工作区），要么重建段树（没写从哪棵、打哪几份）。
- 走法二（主 agent 自己的探索性改动）：`.claude/main-agent.md:7`：「自己写的 `crates/` 改动并进同一批的代码轮三方」，改动本就在主工作区上，`.claude/main-agent.md:57`：「`--skip-apply` 只用于改动确已在主工作区上的（主 agent 自己写的探索性改动这类）：只编、只验」。这些改动不在段树里，第 2 步在段树上判不到；同一批里既有段树补丁又有主工作区改动时，段树上的第 2 步判的是缺了一半的树。

### G1-F 第 2 步红了，定义给的重跑写法不存在（一·1）

- `.claude/main-agent.md:60` 只分两支：「第 1 步红的（这一段的补丁还没合进主工作区），修补以段树为基线写」，与「已合进主工作区之后才红的」。第 2 步红（第 1 步已绿、还没合）两支都不是。
- 第 1 步那一支要的是「第 1 步那条脚本「→ 怎么办：」打出的只重跑红的那一条」；第 1 步绿时脚本打的是合入命令、不打重跑命令（`scripts/gate/merge-and-verify.sh:136`：「验绿（verify 退 0）：汇总行之后打「合进主工作区」的整条命令（merge_in_commands），这里自己不打：」），没有这一条可接。
- 没写的：修补打进段树之后第 1 步要不要整批重跑（修补改到的测试目标没在第 1 步里验过）、第 2 步只重跑红的那几项还是全部、合的时候用哪一次第 1 步打出的合入命令（旧的那一份不含修补的补丁目录）。

### G1-G 实现员定义里变异证红的那一条仍对着主工作区、排在合之后（一·3）

- `.claude/agents/implementation-writer.md:34`：「补丁打进主工作区之后用的那一条点名这几行」，命令里 `--cwd <仓根>`、根写 `<仓根>`，同一句「（`<行名>` 写变异名原样，`<N>` 是点名的行数，`<仓根>` 填主工作区的仓根）」；同一步又写「命令原样可抄，`<根>` 填你改动所在的树」。
- 与 `.claude/main-agent.md:58`（crates-mutation-replay 带 `--item` 在合之前、对着段树交）打架。主 agent 照 `.claude/main-agent.md:20`「主 agent 照原样经服务交」交这一条：合之前交，主工作区的 `crates/mutations.tsv` 里还没有新行，门禁退 2（`.claude/gate.d/crates-mutation-replay.sh:29`：「--item：写变异表里成形的一行的行号，或变异名原样（名字里的逗号不拆、同名的几行都判）；不在表里列出可点名的、退 2。」）；合之后交，又回到 G1-A 的先合后验。

### G1-H 还照旧做法写的两句（一·3）

- `.claude/main-agent.md:65`：「合入后验证（验证段第 1 步）绿了之后，主 agent 经服务交」`refused-test-registry.py list`，括号里写「（在合入后的主工作区里跑）」。先验后合下第 1 步绿了主工作区还没合（合在第 2 步绿之后）：照「第 1 步绿了之后」交，主工作区里没有这一批的新测试，「未登记」一行都列不出，第 4 小条「「未登记」「已不拒算」都没有才算判完」在没看到新测试的情况下就满足了。要么改「合进主工作区之后」，要么改在段树里跑（零轮）。
- `.claude/agent-common.md:115`：「在 rsync 拷的副本里改的，`<根>` 填仓根，由主 agent 在补丁打进主工作区之后经服务交。」命名检查排在合之后、对着主工作区；段树流程下实现员副本是 `minimal-copy.py` 从段树建的（不是 rsync），这一句既点不中新做法、也把命名检查放在合之后（命名红的补丁先进主工作区）。轻一档：命名检查不在验证段的清单里。

### G1-K（推的）承接的实现员以哪一棵为基线，没写（一·1）

- 次序：`.claude/main-agent.md:47`「前一批实现员交回后，补丁只打不验」与 `:48`「红了或证不红的新派实现员（带「承接：」）接着改；这是写的一部分，不算验证。」——P1 先打进段树，P1 的证红、测试红了再派承接的实现员。
- 承接的实现员若在原副本里接着改，副本的基线是打 P1 之前的段树，它交的补丁是「P1 + 修补」累计的；`mutations-append.tsv` 里再列一遍 P1 已追加的行，打进段树时撞名退 2（`scripts/gate/apply-writer-patch.py:16`：「mutations-append.tsv         要追加的变异行，每行六段（与 crates/mutations.tsv 同），名字在表里不许已有、在这个文件里也不许出现两次」）。`.claude/agents/implementation-writer.md:15` 只写「补丁出的是「段树此刻 → 改后」」，没写承接时要不要从段树重建副本。推的：没起实现员走一遍。

### 一·4 代码轮快照「取在全部合入之后」：主句没打中

- `.claude/main-agent.md:59`：「整段的合入里有产品代码改动的，代码轮三方开一轮，快照取在全部合入之后」；`.claude/main-agent.md:33`：「代码三方一轮攻这一批合入，快照取在合入之后，不按补丁开轮」。先验后合下「合」在第 2 步绿之后（`:58`），代码轮在合之后开，取的是主工作区，两句与新次序不矛盾。
- 查过的走法：第 1 步红 → 修补进段树 → 再验 → 合 → 开轮；第 2 步红（G1-F，次序没写，但仍在合之前）→ 合 → 开轮；合之后才开的轮打中 → 修补以主工作区为基线（`:60`）。三条里代码轮都开在合之后。
- 连带的两处：代码轮打中的修补没有段树可做第 2 步（G1-E）；别的会话的轮的快照只在合进主工作区时才查（G1-B 第一种）。照 G1-A 那四句先合后验的话，代码轮仍开在合之后，只是合进去的补丁没过第 2 步。

### G1-J 段树跨轮留着时，完工清理闸拦收工，定义没给去向（一·5）

- 段树是 `minimal-copy.py` 建的、带 `.git`：闸认它是仓副本（`.claude/hooks/completion-cleanup-guard.sh:71`：「仓副本：有 .git（目录；.git 是文件的报成工作树），或同时有 Cargo.toml 与 crates」）。
- 主 agent 能声明的交付物只有三种：`.claude/hooks/completion-cleanup-guard.sh:667`：「exempt_kinds = set() if broken("deliverables-ignored") else {"报告", "补丁", "备份"}」；「交接」主 agent 照列（`.claude/hooks/completion-cleanup-guard.sh:85`：「主 agent 结束一轮时照列，用完就删。」）。定义也只许这三种：`.claude/main-agent.md:34`：「要留的报告、补丁、打补丁前的备份照它文件头「要留的交付物」写」。段树哪一种都不是。
- 走法：一段写了两批、第三批要等用户拍一个设计问题，`.claude/main-agent.md:35`：「一批任务停下来就停，等用户指令」。主 agent 结束这一轮时手里没有在跑的后台任务与子 agent，闸列出段树、拦下；再收工一次单子没变就放行、把单子交给用户（`.claude/hooks/completion-cleanup-guard.sh:93`：「拦过一次、续跑回来（stop_hook_active 为真）还是同一张单（一个都没删）：放行，把单子作 systemMessage 交给用户处置（免得卡死）」）。
- 定义没写的三条路各有毛病：删掉——段树里是还没验、没合的几份补丁叠成的样子，重建要照调度记录里的次序（`.claude/main-agent.md:47`「打的次序记进调度记录」）把补丁目录逐份再打一遍，没写；声明成「备份」——过得了闸，但不是 `:34` 许的那种备份；不管——每次收工被拦一次、单子落到用户那里。
- 另一处会叫主 agent 删段树：看门狗在有子 agent 交回时列「草稿根里超阈值没动的编译目录与仓副本」，`scripts/agents/watch.conf:31`：「scratch-idle-hours=6」，`.claude/main-agent.md:86`：「主 agent 核过没人在用再删」。写段里一批实现员跑六小时以上、段树这段时间一个字没动，正好被列出；「没人在用」核得过（没有进程开着它），照这一句就删了还要接着打补丁的段树。
- 改法（零轮）：许主 agent 把段树在 `.deliverables` 里声明成一个新种类（例「段树」）并写进 `:34`，或在 `:47` 写明段树跨轮留着时怎么声明、看门狗报的段树不删。

## 二、崩溃点重放入口带 `--item` 换成两条

### 二·1 还有没有句子叫人交入口带 `--item`、或与两条新写法对不上：没打中

- 查过的范围：六份里「崩溃点重放入口」「崩溃放量入口」「带 `--item` 的门禁」的全部出现处（python 逐行搜，命中 `.claude/main-agent.md` 第 20、48、57、117、118 行，`.claude/agent-common.md` 第 50、66、106、107 行，`.claude/agents/implementation-writer.md` 第 31、40 行；`scripts/crash/crash-point-replay.sh` 只在 `.claude/agents/implementation-writer.md:49` 当登记合表的说明出现）。
- `.claude/agent-common.md` 第 106、107 行提到「崩溃点重放入口」是在描述派发闸 ⑲ 与范围声明闸判什么，钩子今天确实判这一形，不算叫人去交。
- 项数：模板「范围=<项数加一> 项（1 格加点名的 <N> 项）」与钩子的数法一致（`.claude/hooks/run-scope-guard.sh:65`：「带 --check / --item 是 items，N = --check 点名的格（逗号隔的逐个算）与 --item 点名的项，各自去重后相加；都不带是 whole。根参数给不给一样算。」）。

### G2-2 模板照抄过不过范围闸两判（二·2）

两判之一（不指定范围没带标识的拒）对这几条模板都不触发：模板都带 `--check` 或 `--item`。另一判（指定范围点到崩溃枚举、没缩域也没全域理由的拒）分三种：

1. 这一轮改的两条模板过得了，原因是钩子不判它们。`.claude/hooks/run-scope-guard.sh:59`：「判不到的：门禁 crash-point-replay 点名 crash-case 用例（--item crash-case:<用例>）、崩溃放量入口点名分项、合入后验证、不点名 --test 的整包 cargo test、验证包 src/ 下的内联测试；」。代码里只认 `scripts/crash/` 下那一份入口：`.claude/hooks/lib_run_scope.py:687`：「if name == "crash-point-replay.sh" and project_code_runs.located_in(word, directory, "scripts/crash"):」。门禁 `.claude/gate.d/crash-point-replay.sh` 自己也不估域：在它里面搜 `scoped_domain`、`SCOPED_CRASH_STATES_LIMIT` 命中 0 处（量的：`grep -c` 输出 `0`、退 1 即无命中）。旧写法（入口带 `--item`）既被钩子判、入口自己超 20 万态退 79；换成门禁按块那一路之后，规则 `.claude/rules/verification.md:57`：「指定范围的那几趟（点名测试目标、`--item`）崩溃状态数压在 20 万量级」在这条模板上没有东西管。照抄过得了，但过的理由是没人判。
2. 放量模板过得了钩子，入口自己会拒：`scripts/crash/crash-amplification.sh:18`：「或估不出就拒：不编、不跑，退 79，说明里给两条出路——带 --crash-expand-up-to <n>（算得出就写出能压到上限以内的 n），」。新登记的分项、这一段补丁改了键的分项，第一次跑没有同深度的记录，照「估不出」退 79。六份里写放量模板的几处（`.claude/agents/implementation-writer.md:34`、`.claude/main-agent.md:58`）都没提退 79、也没提缩域；缩了域又是「缩过域、不算门禁通过」。照抄会被入口拒，不是被钩子拒。
3. 同一份定义里没改的三条模板，点到崩溃枚举目标照拒，且必然会点到：
   - 变异门禁：`.claude/agents/implementation-writer.md:34` 的 `crates-mutation-replay.sh --item <项>` 模板；钩子判它（`.claude/hooks/run-scope-guard.sh:50`：「变异门禁 crates-mutation-replay.sh --item 点名的行。」）。
   - 证红：`.claude/agents/implementation-writer.md:37`：「3a. 证红命令一律写成 `task-server/task submit -- bash scripts/mutation/prove-red.sh --copy <副本> <crate> <变异名…>`」；钩子判它（`.claude/hooks/run-scope-guard.sh:49`：「证红脚本 scripts/mutation/prove-red.sh 点名的变异行、」）。
   - 点名测试目标：`.claude/agent-common.md:68`：「测试目标：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=<crate> 的测试目标 <自己的目标>; 范围=1 个测试目标; 为什么=…' task-server/task submit -- cargo test -p <crate> --test <自己的目标>`」；钩子判它（同 `:49`「cargo test 点名验证包测试目标」）。
   - 必然点到：每条新崩溃枚举用例都要一行点到它的变异行，`.claude/agents/implementation-writer.md:49`：「`crates/mutations.tsv` 里至少一行参数带 `-p singlefs-verification` 与 `--test <这个测试目标>` 的变异行」。这一行的证红（3a）、它在第 2 步的变异门禁（`.claude/main-agent.md:58` 点名的 crates-mutation-replay）、「动到的测试目标」那条 `cargo test`，照模板抄都没缩域、没全域理由。六份里「全域理由」出现 0 次，`EXPAND_UP_TO` / `--crash-expand-up-to` 只在 `.claude/main-agent.md` 出现 1 次（合入后验证那一条）（量的：`grep -c`）。
   - 探针（推的，待主 agent 跑）：`expected.tsv` 写的期望——变异门禁点名取号屏障那一行（`crates/mutations.tsv` 第 6659 行，第五段 `-p singlefs-verification --release --test crash_enumeration_acquisition_barrier …`，该目标在 `.claude/gate.d/stage-inputs.tsv` 第 131 行登记成 `crash-case:acquisition-barrier`）退 2；同一条在为什么里加「全域理由：…」退 0；证红同一行退 2；`cargo test -p singlefs-verification --test crash_enumeration_acquisition_barrier` 退 2；旧入口 `scripts/crash/crash-point-replay.sh --item acquisition-barrier` 退 2；门禁按块 `--item crash-case:acquisition-barrier` 退 0；放量 `--item fixed-script-through-rollback` 退 0；合入后验证三份补丁退 0。
- 四句：判别子——钩子在交那一刻读得到登记表，判得出；字面分句——满足「定义里的命令模板没带缩域也没写全域理由，照抄会不会被拒」；改法——在 `.claude/agents/implementation-writer.md` 第 1d、3a 步与 `.claude/agent-common.md:68` 的模板旁写「点到登记的崩溃枚举目标时，为什么一段写『全域理由：…』或带缩域变量」（零轮）；第 1 种（新模板没人估域）要另在门禁按块那一路或钩子上补，改定义碰不到。

### G2-3 旧写法点得到、新写法点不到的（二·3）

- 旧入口每一项另有「测试函数那一段」：`scripts/crash/crash-point-replay.sh:50`：「测试函数那一段 = 每项的测试目标里除登记成 crash-case: 的用例与放量分项之外的全部测试函数」。新写法的按块那一路不起它：`.claude/gate.d/crash-point-replay.sh:44`：「包里全部测试函数按条复用那一趟不起：不算计划、不起 cargo、不记库」。这些测试函数在写段由「动到的测试目标」那条 `cargo test` 罩，而那条点到登记过的崩溃枚举目标就被钩子拒（G2-2 第 3 种），照抄模板就罩不到。
- 整趟才跑的那几类本身就是项：`scripts/crash/crash-point-replay.sh:83`：「整趟才跑的那几类（WHOLE_RUN_GROUPS，一行「类名<TAB>测试目标，空格隔开；没有写 -」）：各是一项 whole-run:<类名>，--item 点得到、--list-items 列出，不带 --item 时一起跑」。三类：`gpu-and-cpu-agree-on-small-inputs`（七个测试目标）、`pruning-self-proof`（两个）、`pinned-red-examples`（没有目标）。它们既不是登记的崩溃枚举用例，也不是放量分项，新写法的两条都点不到。
- GPU 那一类的七个目标是挂特性的：例 `crates/singlefs-verification/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:158`：「#![cfg(feature = "gpu")]」，不带特性的 `cargo test --test` 编出来是空的；该走的是门禁 `--check feature-gated-tests --item feature-gated:<文件名>`，这一形在 `.claude/agents/implementation-writer.md` 第 1d 步的模板里没有（第 1d 步只列形式化、按块用例、放量、变异四条）。改了 GPU 判器的实现员照第 1d 步写，点不到这七个目标；第 2 步在段树上也就不交。
- `pruning-self-proof` 的两个目标不挂特性、源码里不读缩域变量（量的：`grep -c` 两份都是 0），照 `cargo test --test` 点得到、钩子也不拒；这一类只算「旧写法一条 `--item` 点得到、新写法要另写两条 `cargo test`」，不算点不到。

## 三、交回闸与检出钩子的两句

### 三·1 两句与钩子逐字对不对得上：没打中，两处小出入

- 对得上的：只认编译目录、工作树、仓副本三种（`.claude/agent-common.md:104` 与 `.claude/hooks/completion-cleanup-guard.sh:452` 的 `HAND_TO_MAIN_KINDS`）；原因至少 8 个字（钩子 `HAND_TO_MAIN_REASON_MINIMUM = 8`，按 `len()` 数字符，标点也算）；路径一字不差、上层不替下层（钩子 `os.path.normpath` 后与单子上的路径比）；主 agent 一侧照列（钩子 `handed_to_main` 对 `role == "main"` 交空）。
- 出入一：钩子只认认领目录与草稿目录里的（`.claude/hooks/completion-cleanup-guard.sh:87`：「删不掉的（只子 agent，只编译目录、工作树与仓副本三种，只认认领目录与草稿目录里单子上列出的那几处）」），`.claude/agent-common.md:104` 同一句前面列的判的地方还有「自己起的崩溃放量运行目录」，没写运行目录里的不认这一行。子 agent 不交任务、起不了运行目录，实际碰不到。
- 出入二：钩子的行格式要路径以 `/` 起头、路径里不能有空白与冒号（`HAND_TO_MAIN_LINE` 的 `(/[^\s`'"：:]+)`）；`~/…` 写法不认。`.claude/agent-common.md:51` 写「全路径照抄闸列的清单」，照做就是绝对路径，不算打中。
- 另：`.claude/agent-common.md:51` 的「交主 agent 删」与上游规则 `.claude/singlefs-ai-sop/rules/command-safety.md:181`：「没删的，在交回报告里逐个写一行「没删 <全路径>：<为什么>」。」是两种写法，工作根里的副本只认前一种（`.claude/hooks/completion-cleanup-guard.sh:6` 说它「不收「没删」说明」）。三方腿这类定义的开工先读点名了 command-safety 那一节，照它写「没删」的会被拦一次。不在这一轮要判的句子里，只记。

### 三·2 会不会读成「不想删就写一行交主 agent 删」：文字上没打中，记一处管不到

- `.claude/agent-common.md:51`：「删不掉的编译目录、工作树与仓副本（`rm` 被权限拒、被别的闸挡），在交回正文里逐个写一行「交主 agent 删 <全路径>：<删不掉的原因，至少 8 个字>」（全路径照抄闸列的清单），闸认这一行、不再拦这几份；主 agent 结束一轮时照旧列它们，由主 agent 删。」紧挨在「用完就删」那一句后面，限定词「删不掉的」与两个例子都在，读不成「不想删」。
- 管不到：钩子不核删没删过、`rm` 有没有报错，原因写够 8 个字就放（量的读法：`handed_to_main` 只比 `len(reason) >= minimum`）；定义也没要求在那一行旁边贴 `rm` 的原样报错。子 agent 写「副本太大删起来慢」能过。改法（零轮）：在 `:51` 要求原因里贴 `rm` 的原样报错一行。

## 四、长跑拆分规则的「门禁管哪一半」与三处包名

### G4-1 豁免按「命令里有没有这个词」整条判，规则没写这一半（四·1）

- 规则：`.claude/rules/long-run-split.md:34`：「它按片或整条任务的用时判，不拆到任务里的一项。」第二条与 `.claude/gate.d/code-tooling.sh:65` 那一格逐项相同（有 #、8 个字、存在的 `records/<文件>.md:<行>`、不重复、不是目录也不是门禁文件名），对得上。第一条的用时口径与 `task-server/task_server_lib.py:179` 一致，只少了服务表里的「估算的按估算子任务」（`task-server/task_server_lib.py:2426`：「任务（分片的按片、估算的按估算子任务）出结局」）。
- 差的那一半：服务判「在不在豁免表里」是按词：`task-server/task_server.py:5820`：「等于这条命令的某个词（--item 后面的项、--test 后面的测试名、」。于是：
  - 一条任务点了两项、其中一项在豁免表里：整条任务不报。另一项跑了三小时也收不到提醒；规则第三条「还靠主 agent：收到提醒之后按日志里各项的起止判是哪一项超了 7200 秒」（`.claude/rules/long-run-split.md:36`）以收到提醒为前提，这一种没有提醒可收。规则 `:30`：「一项一行，不按目录、不按整道门禁豁免。」——服务实际给出的是按整条任务豁免。
  - 提交时不指定范围那一趟（规则 `.claude/rules/long-run-split.md:8`：「提交时不指定范围那一趟同样按其中每一项、每一片判；整件的挂钟不判。」）命令里没有项名，豁免表一条都配不上：豁免了的项所在的那一片照样报。
- 这一节写了「按片或整条判，不拆到一项」，没写「豁免也是整条判、按命令里的词配」，也没写这两种情形主 agent 怎么补。
- 四句：判别子——服务出结局时只有命令与用时，看不到片内各项的起止，分不开；改法（零轮）：规则写明这两种情形，或服务只在任务点名的项全在豁免表里时才不报。

### G4-2 换了包名之后同一份里还有两处旧说法（四·2，弱）

- 三处已换：`.claude/main-agent.md` 第 33、57、122 行写「产品 GPU 包的测试包 `singlefs-background-gpu-tests`」。
- 说同一件事、没换的：`.claude/main-agent.md:118`：「跑到的验证包目标里登记的崩溃枚举用例、挂特性的测试文件与产品 GPU 包的真卡对拍逐项交门禁 `crash-point-replay`」——同是合入后验证起真卡对拍那一格，触发条件是补丁改到测试包（`scripts/gate/merge-and-verify.sh:177`：「包名是那一道里 PRODUCT_GPU_PACKAGE_NAME="<包名>" 那一行现读的那个包（此刻 singlefs-background-gpu-tests；」），只改产品 GPU 包 `crates/singlefs-background-gpu/` 本身不起这一格；`:118` 读起来是改产品包就起。`.claude/main-agent.md:72`：「挂特性的测试文件、产品 GPU 包与别的包里的新测试」，新测试在测试包里，这里仍写产品包。
- `.claude/rules/` 下：搜 `singlefs-background-gpu`，只有 `.claude/rules/verification.md:39` 一处，写的是新包名（量的）；`.claude/rules/implementation-workflow.md` 第 64、88 行写「产品 GPU 包的测试包」不带名字，不算旧说法。

## 没打中的形状

- 段树建不建得成：照 `.claude/main-agent.md:47` 的命令原样建（`--with scripts=…`、`.claude/scripts`、`.claude/singlefs-ai-sop` 三条加两个开关），量的 123.7 MiB、阈值 200 MiB，退 0；从段树再建实现员副本退 0。闭包没拒、没超阈值。
- 段树里起命令的多机配置：段树的 git common-dir 在段树里、没有 `multi-host.env`；查了服务给尝试的环境：本机那一次从服务自己的环境继承（`task-server/task_server.py:11203`，服务的单元环境里有 `SINGLEFS_MULTI_HOST_CONFIG`），第二台由 `multi-host-run.sh` 带过去（`task-server/task_server.py:2501`），不打中。没起任务核，算推的。
- 段树里第 2 步要的文件：formal-verification 的钉表格不读 `research/`（`.claude/gate.d/formal-verification.sh:35`），段树缺 `research/` 不影响；其余几道没逐一核。
- 补丁的基线：段树与实现员副本都是空索引的 git 仓，补丁头的基线 blob 在段树对象库里找不到时读补丁目录的 `baseline/`；顺序打 P1→P3 时内容逐份相同，`git apply --check` 直接过，不进三方合并。没打中。
- 范围声明闸对合入后验证模板：恒是 items、范围数是补丁份数，模板对得上（探针第 8 条，推的待跑）。
- 「门禁管哪一半」第二条与 `code-tooling.sh:65` 逐项相同。
- 三·1 的目录种类、字数、一字不差、主 agent 一侧，逐项对上。

取样范围：一·1 只走了「两批、三份补丁、互不撞文件」一种布局与 G1-B 列的四种退 2 原因；没走同一批两份撞文件、补丁只改 `crates/mutations.tsv`、补丁改工作区根 `Cargo.toml`（合入后验证退 3）这几种。二·2 的探针每种形状一条样本、只用取号屏障一条流。

## 要主 agent 跑的命令

1. 范围声明闸探针（喂钩子，照主 agent 判），在仓根起：

```
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/ledger-defs-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/ledger-defs-r1-snapshot /home/fy5090/.local/share/singlefs/work/ledger-defs-r1-opus-probe-run /home/fy5090/code/singlefs/research/prompts/ledger-defs-r1-opus-model
```

   跑完看：末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号>` 里 8 行 `name=probe sample=… expected=… exit=… verdict=…` 与末行 `name=probe_summary samples=8 mismatches=0`。`verdict=differs` 的那一行是 G2-2 那一条推断不成立的地方，照它的 `first_stderr_line` 判。运行目录 `/home/fy5090/.local/share/singlefs/work/ledger-defs-r1-opus-probe-run` 里有 `results.tsv` 与每条样本的 `.err`；用完删。这一条不起编译，不用 `task clean`。

## 这条腿自己的限度

- 一·1 到一·5 是对着定义与脚本文件头、代码推的走法，没有真派实现员、没有交合入后验证与带 `--item` 的门禁；G1-B、G1-C、G1-D、G1-F 的「脚本会怎样」引的是脚本文件头写的行为，没有在样本树上跑出来。
- G1-K 只是推的。
- G2-2 第 1 种「门禁按块那一路不估域」靠在门禁里搜不到估域的调用（0 命中）与钩子文件头那一句，没跑一条大域用例看它真的不拒。第 2 种的退 79 引的是入口文件头，没跑。
- G4-1 引的是服务代码的判法与文档串，没造任务核提醒推没推。
- 被判对象里由用户决定的动作（段的批数、哪一批停下等用户、别的会话什么时候改主工作区）都放开写成「哪一种都会到这一步」，没在一个固定次序上下结论；G1-J 依赖「一段中途停下等用户」这一种用户动作，用户一直说接着跑就碰不到收工那一支，看门狗那一支仍在。

## 没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没往统一任务服务交任务，探针没跑，报告里引探针的退出码都是「推的，待主 agent 跑」（命令在「要主 agent 跑的命令」一节）。
- 没在副本上改代码、没量副本上的数。
- 只判背景材料四组点名的句子；六份里别的会话的改动没攻。`crates/` 下的产品代码不在这一轮里，没读。
- 建过的两份仓副本交回前已删：`/home/fy5090/.local/share/singlefs/work/ledger-defs-r1-opus/segment-tree`（删前 `du -sh` 187M）、`/home/fy5090/.local/share/singlefs/work/ledger-defs-r1-opus/writer-copy`（182M）。草稿目录里留的是文本草稿（建副本的两份输出、引文清单、探针的生成脚本），没有副本与编译目录。
