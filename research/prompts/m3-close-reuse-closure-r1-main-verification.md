# m3-close-reuse-closure-r1 判决（推论轮第一轮，2026-10-22）

被判对象：用户 2026-10-22 定的「丙加甲」（原话在调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md`）。丙：判法单元闭包不走内存包装 `scripts/hosts/run-with-memory-cap.sh` 里只打提示的查机器那一支（`HostLookup`），配一格自证钉住「那条路只打提示、不改判定结局」。甲：第二台的探测改成两台跑同一份仓里的脚本、由参数选机器，判法学会把函数里 `bash -c "$N"` 的串算成调用处给的、在调用处折开。正文 `research/prompts/_m3-close-reuse-closure-r1-body.md`。腿：正推（Sonnet）`research/prompts/m3-close-reuse-closure-r1-sonnet-output.md`（3bf95046…），攻方（Opus）`research/prompts/m3-close-reuse-closure-r1-opus-output.md`（6ac016d1…）。

第一轮的打中先挂起，改过的形态交第二轮攻。

## 一、核查

1. 开工快照：`judged-sha256.txt` 6 行（被判的六份脚本）、`kb-sha256.txt` 3 行，`sha256sum -c` 全部 OK。
2. `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-reuse-closure-r1-background.md`：核了 89 处引文、83 处裸行号，1 处对不上：正推引 `scripts/mutation/broad-mutation.py:1465` 的 `key, refusals = library.closure_key(closure, allow_refused=T…`，原文在第 1466 行。这份文件腿交回之前被 C489 的改动动过（不在快照里），只差一行。
3. 主工作区现查：
   - `scripts/hosts/run-with-memory-cap.sh` 第 939–940 行 `self.thread = threading.Thread(…)`、`self.thread.start()`，没有 try 包着；第 1007 行 `with tempfile.TemporaryFile(…) as output, tempfile.TemporaryFile(…) as errors:` 在 `self.guard` 与 try 之外。
   - `multi-host.env.example` 第 11 行：「第二台上放这一趟树副本的专用目录（绝对路径）：每一趟在它下面建一个子目录放树与编译目录、跑完删掉」。第二台今天没有常驻的同版本仓。
4. 两份模型在草稿副本 `/tmp/claude-1000/-home-fy5090-code-singlefs/59734d6f-e329-44d9-9b92-c8744e8de738/scratchpad/closure-verify/` 复跑（第五节）：攻方末行 `name=rerun_compare status=same lines=28`；正推的输出与腿的 `out/rerun-1.out` 逐行相同，只多了排内存队的几行包装提示。正推复跑里这几行原样：`RESULT tree=current entry=scripts/mutation/broad-mutation.py::Runner.escalate_step units=792 refused=[inline-script-not-literal scripts/hosts/peer-host-lib.sh:76;]`、`RESULT tree=cut-lookup entry=scripts/gate/verus-proof-check.py::check_proof units=1232 refused=[]`、`begin（起线程失败，主线程里）: 异常冒出来 RuntimeError: can't start new thread`、`finish（临时文件建不了，退 252 之前前台查，主线程里）: 异常冒出来 OSError: [Errno 28] No space left on device`。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| V1 内容 | 一致：查机器那一支只出标准错误提示、告警记录与节流文件；判法读 Verus 标准输出的 JSON | 打中（量过）：判法把包装的整份 stderr 当 diagnostics 读，HostLookup 那一行带着第二台 stderr 的末行，必红世界存进库的那一句判定随它多出一段；两次都判绿，记下的句子不同 | 攻方量过、正推没查判法读 stderr 这一层。打中，挂起：「只打提示」在写进库的内容这一层不成立 |
| V1 结局 | 冲突：主线程里两处没兜住（起线程、前台建临时文件），起不了线程或写不下临时文件时包装退 251、或 252 变 251；注入故障量的 | 打中（量过）：排队中途 TMPDIR 被删，前台 `TemporaryFile` 抛出、包装退 251；同一状态下关掉 HostLookup 的两条臂都退 252；形式化只在 252 时记 QueueRefusal，crates 变异遇 251 整张表停 | 两腿同指一处，主 agent 现查坐实第 940、1007 行。打中，挂起：那条路能改判定结局 |
| V2 按调用边排 | 一致：只切 `HostLookup.query` 起 `host-resources.py` 那条边，形式化三个入口拒算 1 → 0、单元 1579 → 1232；广谱变异 `Runner.escalate_step` 792 → 436 | 打中（量过）：切掉的约 350 个单元里有 `lib_long_run.py` 的 40 个；`need_of` 走 `predicted_bytes` 时 `exec_module` 起 `host-dispatch.py`，它的模块体 import 了 `lib_long_run`，副本里让这个 import 失败，包装从退 0 起跑变成退 252 | 两腿的数相容（1579 − 1232 = 347）。攻方补上正推没查的一半：被切掉的单元里有会改结局的代码，它今天只经那条边进闭包，预测路径的 `exec_module` 不连边。打中，挂起 |
| V2 按函数、按文件、按开关 | 按文件丢 `predict` 一类准入单元；按函数丢 `begin`、`finish`、`cancel`（正是会退 251 的代码）；按开关静态判不了 | 按函数写不成：HostLookup 不是闭包单元，原文算在 shell 函数 `admission` 的指纹里；现成的 `excluded_units` 挡不住这条拒算（拒算只按 `excluded_files` 滤，推的） | 两腿同向，采：只有按调用边一种写法可行 |
| V3 甲的前提 | 冲突：`multi-host.env.example` 第 11 行，第二台只放每一趟的树副本、跑完删 | 打中：同一条 | 坐实。甲要的「第二台有同版本的仓」今天不存在，甲要改写 |
| V3 键里算第二台 | 推：键里不放第二台，版本由消费者核（两台比判法单元指纹、导入时按本机现算的键逐条核） | 打中（夹具量过）：ssh 后面的命令串既不连边也不拒算；甲把 `peer_host_run` 折开之后键算得出，第二台真跑的那份脚本仍在键外；`"$目录/相对路径"` 按本机根里那一份认，对常驻目录不成立 | 攻方打中一个今天就有的盲区：ssh 串静默不进闭包、也不拒算。主 agent 采：这一条不分辨丙甲，另立欠账、要会红 |
| V3 `bash -c "$N"` 折法 | 21 处 `peer_host_run` 调用形状不一（整串字面 6、字面加数据词 10、不该折 5）；折对要五个条件，三个今天没有机制；丙一做，已量的入口不再需要它 | 同上第 ② 条 | 两腿同向：折法代价大、丙之后已量的入口用不上。甲里的折法这一半延后，不在这一题做 |
| V4 条文 | D13 已定项 17 补一条按边排除的条款（逐字候选），`implementation-workflow.md` 补一段 | 打中：候选句与已定项 17 第 432–435 行与 `implementation-workflow.md` 第 31 行打架；判据「不改判定结局」被 V1 结局那一条打破，收窄到「不改写进库的内容」又被 V1 内容那一条打破；只有先改判法、不读包装自己打的行，判据才自洽；给了 4 个自证格，两格今天红 | 采攻方：条文只在两条前提改完之后才写得成 |
| 两处现象 | 广谱变异的判法单元指纹带着拒算也出键（`broad-mutation.py` 第 1466 行 `allow_refused=True`）；新条文「不按文件排」与 `lib_verdict_library.py` 的 `NOT_JUDGING_FILES` 字面冲突 | 找到正文没列的入口 `Runner.escalate_step` | 坐实。前一条是不是已定项 17 说的「拒算照跑」要判，另立欠账；后一条进第二轮的条文候选 |

## 三、去向

### 第二轮攻的形态（这一轮合成，被攻过零轮）

丙改成三件，前两件是前提：

1. **查机器那一支出了异常只等于没有提示**：`HostLookup` 的起线程、前台建临时文件、读输出整段包在 try 里，任何异常都只记一行告警、不改包装的退出码（该退 252 照退 252）。自证加两格：注入「起不了线程」与「建不了临时文件」，包装退出码与关掉 HostLookup 的那一臂逐个相同；弄坏开关把 try 拿掉，这两格红。
2. **判法不读包装自己打的行**：凡读包装标准错误当诊断的判法（形式化三个入口、广谱变异、crates 变异），先去掉以 `run-with-memory-cap:` 起头的行再读。自证加一格：同一份 Verus 输出，HostLookup 那一行带不同的第二台末行，存进库的判定逐字相同；弄坏开关不去掉这些行，这一格红。
3. **按调用边排除，只排一条边**：判法单元闭包不走 `HostLookup.query` 起 `host-resources.py` 那一条子进程边；同时 `predicted_bytes` 的 `exec_module` 起 `host-dispatch.py` 要连成边（闭包算法认 `exec_module` 的字面路径，或改成静态 import），让 `lib_long_run.py` 等预测路径上的单元留在闭包里。自证：切边前后闭包里少掉的单元恰好是只经那条边可达的那几个；`lib_long_run.py` 的一个单元改了，形式化入口的键变。

甲改成：第二台跑的是同一份脚本，取自每一趟同步过去的树副本（`scripts/hosts/source-stamp.py sync` 拷的那一份），探测前先把 `scripts/hosts/` 下用到的几份同步过去，由参数选机器；host-resources 与 host-dispatch 不再送 base64 程序。键里不算第二台那一侧，第二台判出的东西导入本机时照已有做法按本机现算的键逐条核。`bash -c "$N"` 的折法这一题不做（丙之后已量的入口不再需要它）。

### 另登欠账（不挡被判对象）

| 欠账 | 出处 |
|---|---|
| 闭包算法对 `ssh <别名> "<命令串>"` 既不连边也不拒算，第二台跑的东西静默落在键外；要改成拒算（会红），再谈折开 | 攻方 V3 夹具 |
| 广谱变异的判法单元指纹带着拒算也出键（`scripts/mutation/broad-mutation.py` 第 1466 行 `allow_refused=True`），与 D13 已定项 17「拒算的照跑、不复用」是否相容没判 | 正推 V3 |
| `predicted_bytes` 经 `exec_module` 起 `host-dispatch.py` 不进闭包：今天经 HostLookup 那条边碰巧罩住，丙做了就露出来（随丙第 3 件一起关） | 攻方 V2 |

## 四、多轮小结

第一轮。丙在两层被打穿：包装退出码（252 变 251）与写进库的判定句子；按边排除会顺带丢掉预测路径上会改结局的单元。甲的前提（第二台常驻同版本仓）今天不存在。第二轮攻第三节的形态，分工照 `.claude/rules/three-way-inference.md`：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。

## 五、复跑

- 攻方模型：拷到草稿副本 `opus-model`，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <副本>/opus-work`，退出码 0，末行 `name=rerun_compare status=same lines=28`。
- 正推模型：拷到草稿副本 `sonnet-model`，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-reuse-closure-r1-sonnet/repo --work <副本>/sonnet-work --model <副本>/sonnet-model`，退出码 0；去掉排队提示后与腿的 `out/rerun-1.out` 逐行相同（`diff` 只列出复跑多出的 4 行 `run-with-memory-cap:` 排队提示）。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 17 | 支撑 | 2026-10-22 不受影响：第一轮打中挂起，按边排除的条文要等两条前提改完，交第二轮 |
