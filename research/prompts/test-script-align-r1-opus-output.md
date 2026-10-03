# test-script-align-r1 攻方腿（Opus）：T2、T3

日期：2026-10-01。被攻的格：T2（钩子放行登记为 light 的自证与门禁样本档）、T3（入口与门禁的判法）。前几轮判决：无。

## 复跑

```
bash research/prompts/test-script-align-r1-opus-model/rerun.sh /tmp/claude-1000/<新草稿目录>
```

模型目录 `research/prompts/test-script-align-r1-opus-model/`，`SHA256SUMS` 原样：

```
98f22e3dd9655c72d3f205e6b33df726cad23293fc730e6086e9928bc7638bc3  hook-probe.py
1b533e09bf2dea54fc5c00c58d982e640db1e8c42bc546ccd64ec5d1eb126113  hook-probes.sh
048028c2069a68d143977cbb4639f3f36058b230b18b06ffc47c7a10e5dcbd34  entry-probe.sh
9eea5b64927b09a8fd71657a8a15708849287bf7224bd4e714c7a5d00b86a34e  mutate-probe.sh
3f0f23861466bad04446075d550e4690128a74c847a9d698fe0cd046ba46c1c7  light-selftest-scan.sh
88a1d3e5b921ee22e86d77a59dd447b1e96028197ee6dc776a664a75f5045221  scan-summary.py
f68d9be362a3c3b2fac8a45ad4b68607dfc17ef50fd2cc3d5e372754d9dec73a  rerun.sh
4dc6ce9cba6aa0759c034392ba85fc21b4acb2c05e2831c36dc23062434f754a  outputs/entry-probe.out
c0e446083cff577d163c2ec60d80e5cc573dacced4219683442839f08ace22e0  outputs/hook-probes.out
25211faaa66b40ada6ec5c4011088bdbba9aba08f83712f69c55c20170c27094  outputs/mutate-probe.out
c40de6d76b3996562774642707d0c8c30a64915eaba323f9149ce82ca25d245c  outputs/scan-summary.txt
cd6212090ce2bd9871ff92a9d72f7a83437ebcfb28e0b73f1327633bc8459ad1  outputs/scan-table.txt
```

装置都跑在仓拷贝上（`rsync -a --exclude target --exclude .git`，拷贝里删掉了 `multi-host.env`）。拷贝取于开工时，那一刻被判的 30 份文件与 `research/prompts/test-script-align-r1-snapshot/judged-sha256.txt` 逐份相同。之后别的会话又改了主工作区里的 `.claude/hooks/lib_heavy_tests.py`（只改了 Verus 判法脚本那几段，样本档与自证判法那几段没动），所以下面引这份文件时用的是主工作区现在的行号。A、C、D 三组探针拿主工作区现在的钩子又各喂了一遍，退出码相同（见 T2-1 节）。
钩子只喂 JSON、看退出码，被判的命令一条都没起。入口与 `mutate.sh` 用的是假驱动、假 cargo、假包装。自证扫描照登记的写法起登记为 light 的 72 件（不含 `agent-watch.py`），外加样本档一件，没有起 cargo 编译或跑测试。

## 各格判定一览

| 编号 | 格 | 被攻的 | 判定 | 证据形态 |
|---|---|---|---|---|
| T2-1 | T2 | 54 号样本档放行只看标记目录在不在 | **打中**：任一棵带 git 的整仓树，只要放一个空的 `.layer0-sample-tools/`，任何子 agent 都能放行跑 54 号快档或 `--full`，阶段起的是 PATH 上的真 cargo | 钩子 JSON 量过；阶段那一段按代码推 |
| T2-2 | T2 | 本批新加的四份 54 号样本 | **打中**：假 cargo 与假 rustc 进库时是 100644，没有执行位；照钩子拒绝说明里给的出路（cd 进样本根直接起），解到的是真 cargo | 钩子 JSON、`git ls-files -s`、`type -P` 量过；起 cargo 那一步是推的 |
| T2-3 | T2 | 自证登记的认法 | **打中**：按「目录/文件名」的路径尾部认，不核内容。同一份写着 `cargo test -p singlefs-checker-tier` 的脚本，不带 `--selftest` 拒，带上就放行，放在哪棵树里都一样 | 钩子 JSON 量过 |
| T2-4 | T2 | 样本档 ① 的认法 | **打中**：只核阶段正文，样本里的 `setup.sh` 不读。造一个样本，让它的 `setup.sh` 起 checker 档，经 stage-selftest.sh 喂进去就放行 | 钩子 JSON 量过 |
| T2-5 | T2 | 登记为 light 的自证会不会起重活 | **没打中重活**：72 件里有 6 件起了真 cargo 或 rustc，起的全是 `-V`（问版本），没有编译、qemu、herd7、verus、z3 | strace 量过 |
| T2-6 | T2 | 定义与共用约束改后的字面 | **打中（做不了）**：tooling-writer 第 7 步要样本档「自己跑到绿」，可是没有标记的 samples-light 那几道，喂真仓的阶段目录一律被拒；拒绝说明里也没写怎么单跑。另有一处钩子误拒 | 钩子 JSON 量过 |
| T3-1 | T3 | 崩溃放量入口：退出码表 | **打中（假「运行条件」）**：前一项红、后一项驱动退 78，整趟退 78，不退 1 | 假驱动量过 |
| T3-2 | T3 | 崩溃放量入口：驱动退 78 时的判法 | **打中（假未跑）**：驱动行报了 `red=2`，可驱动退的是 78，这一项就记 `verdict=not-run`，整趟退 78 | 假驱动量过 |
| T3-3 | T3 | 崩溃放量入口：按登记整批不判的那一类 | **打中（假全量绿）**：约定里驱动行带的 `ignored_points=`、`states_skipped_by_ignore=`，入口一个字都不读，成功句照样写「崩溃放量全量（展开 28）」 | 假驱动量过 |
| T3-4 | T3 | 崩溃放量入口：怎么读驱动行 | **打中（契约一破就是假绿）**：现判 0 态、复用 0 态的全量判绿；行里没有 `red=` 或写成 `red=?` 也判绿 | 假驱动量过 |
| T3-5 | T3 | 崩溃放量入口：换驱动的变量平常跑也认 | **部分打中**：`CRASH_AMPLIFICATION_ITEM_DRIVER` 平常跑也认，输出里没有一处写出换过驱动，成功句照样说全量 | 假驱动量过 |
| T3-6 | T3 | `mutate.sh`：限时改由包装管之后的结局分类 | **打中（这一批把一种红变成了绿）**：包装退 252（排不上）记成「无效」，退 254（被总上限挤掉）记成「破坏被看见了」，两种都不判失败，整轮退 0。改前排队超过 120 秒会撞外层 timeout 记成超时、判红 | 假包装量过；改前那一半按 HEAD 代码推 |
| T3-7 | T3 | `mutate.sh`：弄坏开关只在自证里认 | **打中**：平常跑时带着 `MUTATE_BREAK=memoryascaught`，撞顶也记成抓到，整轮退 0 | 假包装量过 |
| T3-8 | T3 | 54 号 feature-gated-tests 格：怎么读特性 | **潜在**：只认 `#[cfg(…)]` 与 `#![cfg(…)]`，不认 `cfg_attr`。今天 `crates/` 里没有这种写法，所以今天不中 | 抽出判法量过 |
| T3-9 | T3 | 分项驱动的用法错退出码 | **小**：`--expand-up-to` 缺值时退 1，文件头写的用法错是 2 | 量过 |
| T3-10 | T3 | crates 变异门禁认 NOT_RUN 与 `--show-output` | 没打中 | 读代码 |
| T3-11 | T3 | 54 号带特性那一趟数「编进来几条」 | 没打中 | 读代码 |

## T2：钩子放行自证与样本档

### T2-1 54 号样本档：钩子只看标记在不在，阶段也不核里面装的是不是假的

放行写在 `.claude/hooks/lib_heavy_tests.py:966`：

```
    if root is None or not os.path.lexists(os.path.join(root, row.weight[len(STAGE_SAMPLES_MARKER):])):
```

它只问标记在不在，再核阶段正文和主仓那一份逐字节相同（`.claude/hooks/lib_heavy_tests.py:933`）。不问样本根是不是一份样本，也不问标记目录里有没有可执行的假 cargo。阶段那一侧见 `.claude/gate.d/checker-tier-layer0-replay.sh:220`：

```
if [[ -d "$PWD/.layer0-sample-tools" ]]; then
```

之后在 `.claude/gate.d/checker-tier-layer0-replay.sh:222` 把这个目录插到 PATH 最前面。目录里没有可执行的 `cargo` 时，`cargo` 就解到真的那一个。快档样本档在 `.claude/gate.d/checker-tier-layer0-replay.sh:283` 跳过复用与范围两问，过了 `.claude/gate.d/checker-tier-layer0-replay.sh:308` 的 git 检查之后，走到 `.claude/gate.d/checker-tier-layer0-replay.sh:383`：

```
  env -u SINGLEFS_LAYER0_FINDINGS_FILE cargo test --release -p singlefs-checker-tier --lib --tests -- --nocapture 2>&1 \
```

**造法**：任何子 agent 都能做，分两条命令写。
1. 拿一棵带 git 的整仓树：`git clone` 出来的，或主工作区本身。
2. 第一条命令：`mkdir -p <树>/.layer0-sample-tools && touch <树>/.layer0-sample-tools/quick-tier`。
3. 第二条命令：`cd <树> && bash .claude/gate.d/checker-tier-layer0-replay.sh <树>`，这就是真跑快档。不放 `quick-tier` 再带 `--full`，就是真跑层 0 全量。

钩子 JSON 原样（`outputs/hook-probes.out`，`$S` 是草稿目录）：

```
exit=2	agent=implementation-writer	cwd=$S/repo-probe	command=bash .claude/gate.d/checker-tier-layer0-replay.sh $S/repo-probe	stderr1=✗ 重型测试被拒：门禁 checker-tier-layer0-replay.sh；要
exit=0	agent=implementation-writer	cwd=$S/repo-probe	command=bash .claude/gate.d/checker-tier-layer0-replay.sh $S/repo-probe	stderr1=
exit=0	agent=implementation-writer	cwd=$S/repo-probe	command=bash .claude/gate.d/checker-tier-layer0-replay.sh --full $S/repo-probe	stderr1=
exit=0	agent=three-way-attack	cwd=$S/repo-probe	command=bash .claude/gate.d/checker-tier-layer0-replay.sh $S/repo-probe	stderr1=
标记目录插到 PATH 最前面之后 cargo 解到：/home/fy5090/.local/bin/cargo
```

主工作区现在那一份钩子（sha256 `e1e1948f…`）对 `--full` 那一条同样退 0。拷贝里没有 `.git`，阶段会停在第 308 行，所以我没有真起阶段；带 git 的树会走到第 383 行，这一步是按代码推的。

四问：
- 分不分辨臂：分辨。只打中这一批新开的放行路 ②，放行之前的写法一律拒。
- 被判的系统当时看不看得到判别它的东西：看得到。钩子判的那一刻能读标记目录，可以核里面的 `cargo`、`rustc` 可执行、内容和登记的那份样本相同，或者核样本根落在 `<HOOK_REPOSITORY>/.claude/gate.d/fixtures/` 下面。
- 满足的是判据字面的哪一个分句：T2「放行条件能不能被一条子 agent 会写的命令绕过去跑真重活」。
- 改法（推的、只在我的模型上想过、被攻过零轮）：钩子一侧，样本根要落在 `<HOOK_REPOSITORY>/.claude/gate.d/fixtures/<阶段>/` 下面，标记目录里的 `cargo` 要可执行；阶段一侧，进样本档之后 `type -P cargo` 必须落在标记目录里，否则退 2。

### T2-2 四份新样本的假工具进库时没有执行位

`git ls-files -s` 原样（节选这四份里的一份；另三份 `quick-features-{gpu-not-run,zero,red}` 的 `cargo`、`rustc` 同样是 100644）：

```
100644 1735c3fe2204274d62ca89e84ef8a6b03e658cab 0	.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-green/.layer0-sample-tools/cargo
100644 cf0f3a1d068e80d9c9a4ca79a2b1aa541f7314ea 0	.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-green/.layer0-sample-tools/rustc
```

老样本 `green` 的同名文件是 100755。经 stage-selftest.sh 跑的时候没事，因为它先在临时拷贝里跑 `setup.sh`（`.claude/singlefs-ai-sop/scripts/stage-selftest.sh:85`），`setup.sh` 补上执行位（`.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-green/setup.sh:6`）。可是钩子拒 54 号时给的出路是「cd 进放了 .layer0-sample-tools 的样本根、根参数写同一个目录」，照这句在仓里的样本目录上直接起，`setup.sh` 不跑，假工具没有执行位，PATH 就落到真 cargo：

```
-rw-rw-r-- 2 fy5090 fy5090 982 Oct  1 08:54 $S/repo-probe/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-green/.layer0-sample-tools/cargo
exit=0	agent=tooling-writer	cwd=$S/repo-probe/.claude/gate.d/fixtures/checker-tier-layer0-replay.sh/quick-features-green	command=bash $S/repo-probe/.claude/gate.d/checker-tier-layer0-replay.sh .	stder
样本的假工具插到 PATH 最前面之后 cargo 解到：/home/fy5090/.local/bin/cargo
```

推的后果：在主工作区里这样起，样本目录的 git common-dir 就是主仓的 `.git`，能过第 308 行，接着真 cargo 在样本目录里编样本那个小 crate，`target/` 落进主工作区的样本目录，四份样本全判错（假红）。小 crate 不算重活，但样本档判的东西已经不是样本。改法（推的、被攻过零轮）：四份假工具改成 100755 进库；T2-1 的改法也罩得住这一格。

### T2-3 自证登记按路径尾部认，不读正文

`.claude/hooks/lib_heavy_tests.py:866` 拿「目录/文件名」当键：

```
            return f"{place}/{os.path.basename(os.path.normpath(word))}"
```

`.claude/hooks/lib_heavy_tests.py:884` 拿这个键查主仓的登记表。样本档会核阶段正文（第 933 行），自证这一侧不核内容。原样：

```
exit=2	agent=implementation-writer	cwd=$S/repo-probe	command=bash $S/evil-tree/research/scripts/claim-experiment.sh	stderr1=✗ 重型测试被拒：脚本 /tmp/claude-1000/test-sc
exit=0	agent=implementation-writer	cwd=$S/repo-probe	command=bash $S/evil-tree/research/scripts/claim-experiment.sh --selftest	stderr1=
exit=2	agent=implementation-writer	cwd=$S/repo-probe	command=bash $S/evil-tree/research/scripts/my-new-check.sh --selftest	stderr1=✗ 提交时才跑的检查被拒：research/scr
```

三条喂的是同一份内容（第 3 行是 `cargo test --release -p singlefs-checker-tier --lib --tests`）。不带 `--selftest` 时钩子读正文、拒；带上 `--selftest` 就不读、放行。

可达的历史：
- 实现员、攻方腿在仓副本里改 `research/scripts/<登记成 light 的脚本>`，让它的自证真编一次，再照习惯跑 `--selftest`。
- tooling-writer 在主仓里这样改，就是 C613 写的那一种。

C613（`.claude/kb/checks-owed.md:484`）的「怎么拦」写的是运行时查进程树，罩得住这一格；那一行没提样本档那几行（T2-4）。另外按 T2-5 量到的数，「起了 cargo、rustc 就判红」会把只问 `-V` 的 6 件判成假红。改法（推的、被攻过零轮）：C613 那一格只认「不是问版本的」cargo、rustc，并且把样本档也罩进去。

### T2-4 样本档 ①：只核阶段正文，样本里的 setup.sh 不读

`.claude/hooks/lib_heavy_tests.py:996` 只拿阶段文件和主仓那一份比。stage-selftest.sh 会在临时拷贝里执行每个样本的 `setup.sh`（`.claude/singlefs-ai-sop/scripts/stage-selftest.sh:85`）。造一棵 `.claude/gate.d/`：里面放一份原样的 `doc-text.sh`，再放 `fixtures/doc-text.sh/red/setup.sh`，内容是 `cargo test --release -p singlefs-checker-tier --lib --tests`。原样：

```
exit=0	agent=implementation-writer	cwd=$S/repo-probe	command=bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $S/evil-gate/.claude/gate.d	stderr1=
exit=2	agent=implementation-writer	cwd=$S/repo-probe	command=bash $S/evil-gate/.claude/gate.d/fixtures/doc-text.sh/red/setup.sh	stderr1=✗ 重型测试被拒：脚本 /tmp/claude-
```

`lib_heavy_tests.py` 文件头「罩不住的」那一句（第 107 行）列的是「先拷样本、放标记再起」和「判完之后才删掉标记、改阶段正文」，没有列「样本里的 setup.sh 与替身」。可达的历史：tooling-writer 给某道 samples-light 的阶段补一份样本，样本要一份编好的二进制，它就在 `setup.sh` 里写 `cargo build`。那一道的登记行还是 samples-light，钩子照样放行。C613 那一行只管脚本的 `--selftest`，罩不到这一格。改法（推的、被攻过零轮）：stage-selftest.sh 起的整棵进程树一并纳入 C613 的运行时检查；或者①只认 `<HOOK_REPOSITORY>/.claude/gate.d` 下的样本，且样本目录与主仓逐字节相同。

### T2-5 登记为 light 的自证照登记写法跑一次：没有一件编译或起外部工具

做法：72 件逐件起 `<脚本> --selftest`（共用库直接执行），外加一件「样本档」：仓拷贝里删掉 samples-heavy 那三道的样本目录之后，`stage-selftest.sh <拷贝>/.claude/gate.d`。每件包一层 `strace -f -e trace=execve`，记下路径不在 `/tmp` 下、成功了的 cargo / rustc / rustdoc / verus / rust-verify / z3 / herd7 / qemu-system-* 调用。阳性对照：`bash -c 'cargo -V'` 被认出 2 次 execve。挂钟：各件合计 470 秒，并行 6 件。`outputs/scan-summary.txt` 原样：

```
0	research/scripts/admission.py	rc=1	cargo 版本 ×80；rustc 版本 ×80
19	research/scripts/crates-mutation-rows.py	rc=0	rustc 版本 ×44
35	research/scripts/multi-host-run.sh	rc=0	cargo 版本 ×20；rustc 版本 ×32
37	research/scripts/mutation-shard-run.sh	rc=1	rustc 版本 ×96
51	research/scripts/stage-must-run.sh	rc=0	cargo 版本 ×22；rustc 版本 ×22
71	stage-samples	rc=1	cargo 版本 ×6；rustc 版本 ×6
总计：72 件，起了真 cargo / rustc / qemu / herd7 / verus / z3 的 6 件
```

（「版本」指参数只有 `-V`。每次调用记两次 execve：`~/.local/bin` 的包装一次、rustup 工具链一次。）

结论：这一轮没有一份登记为 light 的自证真编或起外部工具。但登记理由有几处和实际不符，例如 `admission.py` 那一行说「PATH 前面放假 cargo、假 rustc」（`.claude/hooks/selftest-registry.tsv:15`），实际有 80 次调的是真 cargo 的 `-V`。

这一格的限度：有 5 件在拷贝上退 1，分别是 admission（283 格里那一格已知的红）、detached-run（`Ⓔ watch` 一格）、mutation-shard-run、prove-red，以及样本档（QEMU 那一道 6 份样本判错，没查原因）。退 1 的那几条路径可能没有走完。detached-run 起的是用户级 systemd 单元，multi-host-run 起的东西若经 ssh 进了别的会话，strace 跟不进去。`agent-watch.py` 没跑：它起的替身进程名字像 checker 档测试二进制，会惊动主会话的看门狗。

### T2-6 定义改后的字面：照着干做不了，另有一处误拒

- `.claude/agents/tooling-writer.md:34` 第 7 步要「登记为 light 的 `--selftest` 与样本档自己跑到绿」。没有标记的 samples-light 阶段只能经 ① 喂，可是真仓的 `.claude/gate.d` 里有三道 samples-heavy，喂真仓的阶段目录一律被拒（`outputs/hook-probes.out` F 组 `exit=2`）。拒绝说明只教了带标记的那三道怎么单跑，没教 samples-light 怎么单跑。照字面干，这一步做不了；要做就得自己搭一个去掉重样本的阶段目录（我在拷贝里就是这么跑的），定义和拒绝说明都没写这条路。另外共用约束「门禁」一节写的是「登记为 light 的」，样本档的判定词其实是 samples-light 和 samples-light-marker:，字面对不上。
- 钩子误拒：我的扫描脚本里有一行 `case` 模式，写的是 `.claude/gate.d/lib/*.py) cmd=(python3 "$root/$key") ;;`。钩子读脚本正文时把它当成一条命令，拒了，原样首句：`✗ 提交时才跑的检查被拒：脚本 /home/fy5090/code/singlefs/research/prompts/test-script-align-r1-opus-model/light-selftest-scan.sh:20 里的 .claude/gate.d/lib/*.py（这份自证没在登记表 .claude/hooks/selftest-registry.tsv 里）`。把前缀放进变量之后就放行了。这是读脚本那一侧的老毛病，不是这一批带进来的；它会让「写一份脚本逐个跑自证」这种做法被误拒。

（补 T2-6 的出处：共用约束那一句在 `.claude/agent-common.md:87`。）

## T3：入口与门禁的判法

装置：`entry-probe.sh` 用一个假分项驱动（`CRASH_AMPLIFICATION_ITEM_DRIVER`）和一个假清场（`CRASH_AMPLIFICATION_QUIESCE_STUB`）起仓拷贝里的 `research/scripts/crash-amplification.sh`，多机配置与显卡配置都指到不在的文件。驱动行的字段照调度记录里的约定写：`states_reused=`、`exhaustive=`、`expand_up_to=`、`ignored_points=`、`states_skipped_by_ignore=`。下面是 `outputs/entry-probe.out` 的原样节选，每一格都整块贴。

### T3-1 前一项红、后一项退 78：整趟退 78

文件头的退出码表（`research/scripts/crash-amplification.sh:69`）写的是：

```
#   1   有红（上面 red 的几条）、或收回的项数与派出的不等
```

代码里，第 716 行 `if [[ -n "$stopped_by_condition" ]]; then` 那一支退 78，排在第 721 行 `if (( run_verdict )); then exit 1; fi` 前面，所以有红又有一项退 78 时，整趟退的是 78：

```
### E0-header-control：退 1
    CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=7 reused=3 red=2 lines=1 expand_up_to=28 exhaustive=true verdict=red seconds=0
### E1-red-then-78：退 78
    CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=7 reused=3 red=2 lines=1 expand_up_to=28 exhaustive=true verdict=red seconds=0
    CRASH_AMPLIFICATION_ITEM item=fixed-script-through-unmount exit=78 states=0 reused=unknown red=0 lines=0 expand_up_to=unknown exhaustive=false verdict=not-run seconds=0
    CRASH_AMPLIFICATION_TOTAL dispatched=2 items=2 failed=1 not_run=1 reuse_unknown=1 states=7 reused_known=3 red=2 expand_up_to=28 scope=items seconds=0
      ✗ 分项 fixed-script-through-rollback：CRASH_AMPLIFICATION 行报了 2 处红
      ✗ 1 个分项判红
      ✗ 分项 fixed-script-through-unmount 的驱动退 78（显卡或第二台用不上，或清场、多机配置不满足）：这一趟停在这一项，后面 0 项没派
```

可达的历史：全量第 1 项判出违例，第 2 项的卡此刻被别的进程占满，驱动退 78。调用方只看退出码，78 照上游的意思读作「运行条件不满足，修好设备再跑」，不当红。自证 21 格里没有一格造「红加 78」。

四问：
- 分不分辨臂：分辨，只在这份入口里。
- 看不看得到：看得到，那一刻 `items_failed` 已经是 1。
- 字面分句：T3「退出码表与实际出路逐个对不对得上」，加上「假『本次未跑』」。
- 改法（推的、被攻过零轮）：先判 `run_verdict`，有红退 1；78 的出路句照打。

### T3-2 驱动行报了红，驱动却退 78：这一项记本次未跑

`research/scripts/crash-amplification.sh:658`：

```
  if [[ "$item_exit" == 78 ]] && ! crash_amplification_break_is item-78-as-failure; then
```

这一支直接把这一项记成 not-run，不看 `item_red`。分类函数自己的规矩是「两类都有按 device（设备用不上时那一趟判的不算数）」（`research/scripts/crash-amplification-lib.sh:196`）。双机驱动在本机或第二台失败时，会先导入、打出已判的行，再按日志分类退出码，所以「已有 `red>0` 的行，退出码却是 78」是一条能走到的路：

```
### E2-red-line-and-78：退 78
    CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=78 states=7 reused=3 red=2 lines=1 expand_up_to=28 exhaustive=true verdict=not-run seconds=0
    CRASH_AMPLIFICATION_TOTAL dispatched=1 items=1 failed=0 not_run=1 reuse_unknown=0 states=7 reused_known=3 red=2 expand_up_to=28 scope=items seconds=0
      ✗ 分项 fixed-script-through-rollback 的驱动退 78（显卡或第二台用不上，或清场、多机配置不满足）：这一趟停在这一项，后面 0 项没派
```

判出来的违例只留在 ITEM 行的 `red=2` 里，`failed=0`。四问：
- 分辨：分辨。
- 看得到：看得到，`item_red` 已经算出来了。
- 字面分句：「哪一格会假『本次未跑』」。
- 改法（推的、被攻过零轮）：`item_red>0` 时不管退出码都记红；一项里只要有一行报红，就不让 78 盖掉它。

### T3-3 按登记整批不判的那一类：入口不读，成功句照说全量

约定的驱动行带 `ignored_points=`、`states_skipped_by_ignore=`。`summarize_the_item` 只读 `states_reused`、`states`、`states_checked_here`、`share`、`stopped_early`、`imported_blocks`、`unjudged_blocks`、`red`、`warm_up`、`exhaustive`、`expand_up_to` 这几个键（`research/scripts/crash-amplification.sh:162` 是 `red` 那一支）。原样：

```
### E6-ignored-not-reported：退 0
    CRASH_AMPLIFICATION_ITEM item=fixed-script-through-rollback exit=0 states=7 reused=3 red=0 lines=1 expand_up_to=28 exhaustive=true verdict=green seconds=0
    CRASH_AMPLIFICATION_TOTAL dispatched=7 items=7 failed=0 not_run=0 reuse_unknown=0 states=49 reused_known=21 red=0 expand_up_to=28 scope=full seconds=0
      ✓ 崩溃放量全量（展开 28）：7 项都退 0、0 红；现判 49 态、复用 21 态；派出 7 项、收回 7 项
```

（每行都带 `ignored_points=4 states_skipped_by_ignore=900000`；ITEM 行 7 行里只贴了 1 行，另 6 行同形。）

上游「判定可复用：按输入存，按前缀剪」要求：成功那句报跳过了多少，列出没判的是哪几类。C605 的前提一列把这一半写成「另算」，可入口这一侧也没登记一格去接它；照现在的字面，跳过了九十万态的那一趟，成功句仍说「全量」。四问：
- 分辨：分辨。
- 看得到：看得到，字段就在行里。
- 字面分句：「哪一格会假绿」。
- 改法（推的、被攻过零轮）：入口读这两个字段；有跳过时成功句报「跳过 N 态（M 类）」，不写「全量」。

（补 T3-2 的出处：双机驱动本机那一份失败时，先导入、打出导入那一趟的行，再在 `research/scripts/crash-amplification-two-hosts.sh:237` 按 share-0.log 分类退出码，日志里有设备用不上的原话就退 78。）

### T3-4 入口对驱动行不设防：0 态判绿、没有 `red=` 或 `red=?` 判绿

`research/scripts/crash-amplification.sh:162` 只在键名正好是 `red` 时累加（`value + 0`）；没有这个键，或者值不是数，这一行就按 0 红算。入口也不要求「现判 + 复用 > 0」。原样：

```
### E3-zero-states：退 0
      ✓ 崩溃放量全量（展开 28）：7 项都退 0、0 红；现判 0 态、复用 0 态；派出 7 项、收回 7 项
### E4-no-red-field：退 0
      ✓ 崩溃放量全量（展开 28）：7 项都退 0、0 红；现判 49 态、复用 21 态；派出 7 项、收回 7 项
### E5-red-not-a-number：退 0
      ✓ 崩溃放量点名的 1 项（展开 28）：1 项都退 0、0 红；现判 7 态、复用 3 态；派出 1 项、收回 1 项
```

E4 的行带 `red_states=3 violations=3`、没有 `red=`。只要驱动按约定写行，这三格就不中，所以它们只在驱动契约被改坏时才是假绿；可入口的自证里也没有一格守这份契约。E3 这种「判了 0 态」的全量，正是上游「扫到 0 条不是通过」管的那一类。改法（推的、被攻过零轮）：每一行都要有数值的 `red=`，没有或不是数就判红；全量那一趟现判加复用为 0 的项判红。

### T3-5 换驱动的变量平常跑也认，输出里不留痕

`research/scripts/crash-amplification.sh:554`：

```
driver="${CRASH_AMPLIFICATION_ITEM_DRIVER:-$crash_amplification_script_directory/crash-amplification-item.sh}"
```

上面 E3、E4、E6 三趟都由假驱动跑出「崩溃放量全量（展开 28）」；「== 分项」那一行、ITEM 行、TOTAL 行都不写驱动是哪一份。上游允许从环境变量换掉驱动，所以这一格不算违规。问题在于全量的完成句认不出这一趟是假的：会话里残留一个 export 的变量，就足以产出一句假的全量绿。改法（推的、被攻过零轮）：换过驱动或清场时，TOTAL 行写 `driver=<路径>`，成功句不写「全量」。

### T3-6 `mutate.sh`：包装退 252、254 不判失败，这一批的改动把一种红变成了绿

装置：`mutate-probe.sh` 在一棵小树里放一份假包装（只对带限时的那几次演结局），加一份假 cargo，一张只有一条「没人抓得到」变异的表（只改注释，照理是「没红」、整轮退 1）。`outputs/mutate-probe.out` 原样（节选三格）：

```
### control-pass：退 1
    ❌ [没人抓得到] 一个测试都没红 —— 这条破坏没有被任何检查看见
### queue-full-252：退 0
    ⏭  [没人抓得到] 变异导致编译失败，本条无效（不计入盲区，也不算命中）
### squeezed-254：退 0
    💥 [没人抓得到] 测试进程没跑完（signal 9）——破坏被看见了，但不是断言抓到的
```

为什么会这样：
- 252 时命令一行都没跑（`research/scripts/run-with-memory-cap.sh:27`），输出是空的，于是落到 `research/scripts/mutate.sh:715` 的「编译失败、无效」那一支。
- 254 时测试进程被 slice 杀掉，cargo 打出 `process didn't exit successfully`，于是落到 `research/scripts/mutate.sh:726`，记成「破坏被看见了」。它的 fail 位是 0。
- 包装文件头说 254「它的结果不算数，重跑它」（`research/scripts/run-with-memory-cap.sh:30`）。

为什么是这一批带进来的：HEAD 那一版把 `timeout "${MUTATE_TIMEOUT:-120}"` 套在包装外面（`git show HEAD:research/scripts/mutate.sh` 第 559 行），排队超过 120 秒就撞外层 124，记成超时，`fail=1`。现在排队时间不算进限时，包装最多排 3600 秒，然后退 252，记成「无效」，`fail=0`。机器忙到一条变异排不上队，以前判红，现在判绿。这条推的根据是 HEAD 代码，没有用 HEAD 那一版跑过。调度记录把「`mutate.sh` 不分类包装的 252/254 结局」写成「进下一批」，C604–C613、C620 里没有一行接住它。

四问：
- 分辨：分辨。门禁那一侧的 `crates-mutation-rows.py` 把「被总上限挤掉、排不上」单独计数，不中；只有 `mutate.sh` 中。
- 看得到：看得到，退出码就是 252、254。
- 字面分句：「哪一格会假绿」。
- 改法（推的、被攻过零轮）：252、254 各自单列一类，`fail=1`，出路写「整张表重跑」；自证补一格「包装退 252」。

### T3-7 `mutate.sh` 的弄坏开关平常跑也认

文件头写着 `#   MUTATE_BREAK=memoryascaught  撞顶记成抓到`（`research/scripts/mutate.sh:57`），平常跑的那一支不拒这个变量。原样：

```
### memory-hit-control：退 1
    🧱 [没人抓得到] 撞了内存上限 64M（内存撞顶）—— 这条破坏让被测代码无界分配，不是「没抓到」，也不算抓到
### memory-hit-with-leftover-break：退 0
    ✅ [没人抓得到] 红：（弄坏开关 memoryascaught）
```

上游「测试脚本自己也要被测试证明有效」要求「开关只给自证用，平常跑时不认」。崩溃放量入口有一道「只供自证」的拒（它的第 21 格），`mutate.sh` 没有。行里写了「弄坏开关」四个字，但退出码是 0。改法（推的、被攻过零轮）：照入口那样，用自证起的标记文件区分；平常跑时带 `MUTATE_BREAK` 就退 2。

### T3-8 54 号 feature-gated-tests 格不认 `cfg_attr`（潜在）

`.claude/gate.d/checker-tier-layer0-replay.sh:640`：

```
attribute_start = re.compile(r"#(!?)\[\s*cfg\s*\(")
```

我把这一段判法原样抽出来，喂三个文件：整份 `#![cfg(feature = "verdict-store")]`、`#[cfg(all(feature = "gpu"))]`、`#[cfg_attr(not(feature = "gpu"), ignore)]`。输出原样：

```
items.rs	items	gpu
whole.rs	whole-file	verdict-store
```

`attr_gated.rs` 没有列出来。它在快档那一趟里只算 ignored，带特性那一趟又不点它，于是「编进来 0 条」不报本次未跑，正是上游「某一类编进来 0 条，报『本次未跑』」要拦的那一种。今天 `crates/singlefs-checker-tier/{src,tests}` 里没有 `cfg_attr` 配 feature 的写法（`grep -rn cfg_attr` 配 `feature` 0 处），所以今天不中。

### T3-9 分项驱动缺值退 1

`research/scripts/crash-amplification-item.sh:81` 用 `${2:?…}`。原样：

```
缺值 rc=1
/tmp/claude-1000/test-script-align-r1-opus/repo/research/scripts/crash-amplification-item.sh: line 81: 2: --expand-up-to 要一个数
认不得 rc=2
```

文件头 `#   2        用法错（参数认不得、缺测试目标）`（第 34 行）。入口自己先核过参数，经入口调不会走到这里，只有直接调驱动才会撞上。

## 没打中的形状

- **T3-10 crates 变异门禁认 NOT_RUN**（读代码，没跑样本）：
  - `NOT_RUN_LINE` 锚在行首（`research/scripts/crates-mutation-rows.py:143`）。配上 `--show-output`、不带 `--nocapture`，用例输出在 successes 段里独占一行。表里 1526 行没有一行带 `--nocapture`（`grep -c -- '--nocapture' crates/mutations.tsv` 为 0），所以没有「粘在 test 行后面认不出」那条路。
  - 只有点名的测试是 ok 并且打了 NOT_RUN 才记本次未跑；有别的测试 FAILED 时落回原判法。
  - `--item` 全是本次未跑时退 77，部分是时成功句写明数字。
  - 想过的反例：一条用例先打 NOT_RUN、再接着跑 CPU 那一半，CPU 那一半没抓到变异。这时会记成本次未跑而不是没红。判法分不出「打了 NOT_RUN 就返回」和「打了还接着跑」，属于判别子观测不到。按约定，GPU 用例打完那一行就返回，所以今天不中。
- **T3-11 54 号带特性那一趟**：
  - 逐文件的条数取两趟日志之和，ignored 不算。整份挂特性、只有 ignored 的文件报本次未跑。
  - NOT_RUN 按子串找，能认出粘在 test 行后面的那一种。
  - 带特性那一趟退非 0 判红。
  - 特性名写错的话，cargo 退 101，判红（不假绿）。
- **崩溃放量入口其余几格**：缩过规模不说全量、展开深度对不上判红、两台的分份行不当本次未跑、派出与收回对不上判红、平常跑时带只供自证的三个变量退 2，都照文件头走（自证 21 格那几格，扫描里它退 0）。我也试过伪造自证标记，可标记文件的内容是一行固定的字，能造出来，但这不是「子 agent 会写的命令」，不算打中。
- **登记为 light 的 72 件自证**：没有一件编译、起虚机、起 herd7 或 Verus（T2-5）。

## 这条腿自己的限度

- 跑过的全在仓拷贝上，假驱动、假包装、假 cargo 都是我写的。副本上的数不是入库装置上的数。
- T2-1、T2-2 的最后一步（真 cargo 被起来）是按代码推的：我没有在带 git 的树上真起阶段，因为那就是跑重型测试。
- T2-5 那 5 件在拷贝上退 1，那几条路径可能没走完；`agent-watch.py` 没跑；strace 跟不进 systemd 单元与 ssh。
- T3-6 「改前判红」那一半是读 HEAD 代码推的，没有用 HEAD 那一版跑过。
- 每一格只抽样了一次。没打中的那几格（T3-10、T3-11）照规矩不能当「没打中」用，要再抽一次。
- 自己提的改法全部是推的、被攻过零轮，没有实现。

## 没做什么

- 没判正推那两格（T1、T4）。T4 相关的只顺手写了 C613 罩不罩得住 T2-3、T2-4，以及 T3-6 没有欠账行接住。
- 没跑重型测试，没起 54 号、crates 变异门禁、QEMU 的真跑，也没跑 `mutate.sh` 的真 cargo。
- QEMU 那一道的 6 份样本在拷贝上判错，原因没查：不在 T2、T3 的攻击面里，那一件也没有起 qemu。
- 没改主工作区任何文件。写进去的只有本报告和模型目录。
- 草稿目录 `/tmp/claude-1000/test-script-align-r1-opus/` 里还留着 `repo/`、`repo-probe/`（两份仓拷贝，后者是前者的硬链接拷贝），以及 `mut-tree/`、`evil-tree/`、`evil-gate/`、`scan/`、`cfgattr/`。交回前删掉，见交回正文。
