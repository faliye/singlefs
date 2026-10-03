# 第五道重型门禁写进规则与定义：定义三方第三轮 云端攻方（Opus）报告（2026-10-01）

<!-- doc-lint:not-numbers G9 G10 G11 J1 J2 J8 Y1 Y2 Y3 L1 L2 L3 L4 K1 K2 K3 K4 P1 P2 P3 P4 P5 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 M1 M2 F1 F2 F3 F4 F5 F6 F7 T1 T2 T3 T4 T5 Z1 Z2 Z3 S0 S1 -->

攻击面：正文第四节分工表「云端攻方（Opus）」那一行，L1、L2、L3。被判文件读主仓现行版本；开工时 `sha256sum -c research/prompts/defs-m3-formal-gate-r3-snapshot/sha256sums.txt` 九份全 OK，探针跑在与快照逐份相同的 rsync 拷贝上。

## 复跑

```
bash research/prompts/defs-m3-formal-gate-r3-opus-model/rerun.sh <仓根> <空草稿目录>
```

挂钟 73 秒（本机整条跑一次，`rerun-output.txt` 末行「rc=0 挂钟 73 秒」）。逐份比的结果：12 份里 11 份「相同」，`l2-K3-output` 报「不同」，只差一行：K3 把 VERUS_ROOT 的值（草稿目录里的路径）打进了那一行，所以摘要跟着草稿目录变（`rerun.sh` 文件头写了）。复跑不跑门禁、不跑判法脚本的干活用法、不跑 `--selftest`，钩子只喂 JSON。L2 那一步直接起 Verus 验一份三行的玩具文件，一趟 8 次，每次都经内存包装 400M 与 capped.sh 4。

模型目录 `research/prompts/defs-m3-formal-gate-r3-opus-model/`，逐份 sha256 在同目录 `SHA256SUMS`（25 份）。几份主要文件：

| 文件 | sha256 |
|---|---|
| rerun.sh | e5b2f771a1b2df2199ee7e75f095bc0e77f117103f87480f9317587e06c8d442 |
| l1-hook-cases.py | 6a5e40e6deb0aabfacb469127fdd9a82f0b920a9caad46ab35793eb2b5f29b92 |
| l1-e165-parse.py | 2a7c304b3ec1e2b918d94e82302f9a74f9acaa16abb150e0e0ff1f9cf37de6d0 |
| l1-argv-demo.py | 0bee4e9ea7806f3b25c460107f705717ae419ccb6ea2c6c94d20cab036e3b1aa |
| fix-draft-l1.py | 181eac9d65a58610a4b161fc5bd5a935c77d451a2c2dc6ecc2790431a76ccdd8 |
| l2-install.sh | 44ecdffaaca501975f96a3b5a00e5abc00c2fa340a1013a11d71c85d36d4f971 |
| fix-draft-l2.py | 0ce2864be8e5170dd3a81fa45f715ef7d6636ec8a15cf938afb23c703edfab37 |
| l3-cell.py | 211122d7c30addd6484eefe4a18021bc38fc118f002ef3ad61c0189ec195cc8c |
| l3-stage.sh | daf7a14dfd538a87fb1bff9f071776e523f3b6ca117d13a3749f0e06fee3890d |
| l3-judge.py | c8bc297333cc27127d000c63b46ab80358f0bd6a4641a8460ffa13f623821b43 |
| l3-judge-k4.py | 8935c1442dd87404918fa0643940d9d82914ff07fdc36e6fb239c3d6b9f48d4e |
| l1-output.txt | cdc15fb6cb9f51e5d4c9d3b01ee8f75e506834a6dd024fb86a0aadb1145b2ce5 |
| l2-output.txt | a74c017f84b6cdb0a30e2bedfd0b5e636e42b033308ecb1bc26544c0b467bd50 |
| l3-judge-output.txt | 97d4e5cd1cbeb33d7952fc629a05eae3c597099fd568208b0a3292f384f76b17 |

## 各格判定一览

| 格 | 形状 | 打中没有 | 量过 / 推的 | 分辨臂 |
|---|---|---|---|---|
| L1-a | e165 判法脚本：`--force` 写在带值选项的值位上，闸看到的模式是 `--check`，preflight 先把 `--force` 摘掉，真跑的是 `--replay`（编 checker 档装置、逐行重放） | 打中（P1、P2 两个身份都退 0） | 量过：闸那一侧喂 JSON，脚本那一侧换桩看 main 落到哪个模式 | 不分 G9 与改前那一版（两版都用 `e165_pin_mode`）；病根是闸不照 preflight 摘 `--force` |
| L1-b | `python3 -c` 的代码自己改 sys.argv，或直接调函数，代码之后写只读子命令（或 e165 什么都不写，判成「追加」） | 打中（C1、C2、C3、C8，各两个身份退 0） | 量过 | 分 G9 字面「参数取代码之后的词」；广谱变异同形（M1 同样退 0），病根同一个 |
| L1-c | `python3 -c` 里用 `__import__` / `importlib.import_module` 按模块名导入判法脚本 | 打中（C4、C5 退 0） | 量过 | 广谱变异同形（M2 退 0） |
| L1-d | python 代码里拼出路径、heredoc 喂 `python3 -` | 照放（C6、C7 退 0） | 量过 | 广谱变异那一段文件头列过的老口子；Verus 这一段的「罩不住的」没列，只记账 |
| L1-e | 误拒：同目录名字相近的脚本、`-m py_compile`、`-W` 选项、仓副本里的判法脚本 tool-versions | 没打中（F1、F3、F4、F6、F7 都退 0） | 量过 | — |
| L2-a | 安装目录子目录里有没有 Verus 读的文件 | 没打中：strace 看到子目录里被打开的文件 0 个 | 量过（玩具文件，一次） | — |
| L2-b | 整体摘要里有没有会变的东西 | 没打中：连算三次，加上 Verus 跑过之后再算一次，四次前 16 位相同；Verus 往安装里写 0 次 | 量过 | — |
| L2-c | Verus 读而 tool-versions 不记的环境变量：`VERUS_ROOT`（宏库、vstd 从那里读）、`VERUS_USE_RUSTUP=0`（rust_verify 起不来） | 打中：tool-versions 那一行与没设时相同，两条复用问法都复用（q1 退 0、q3 退 10），而 Verus 一个编不过、一个起不来 | 量过 | 不分 G10 与改前（都不记这两个变量）；与 J5、J6 同族 |
| L2-d | 安装之外 Verus 读的 rustup 工具链（44 个文件，含 librustc_driver、libLLVM） | 部分罩着：RUSTUP_HOME 换成空目录时复用判定退 2，因为指纹里的 `cargo -V` 起不来；同名工具链内容被换的情形没量 | 前半量过，后半推的 | 3-12 报告列过，没进仓内任何文件头 |
| L3-a | gate-triage 第 3 步改成「一道里每一条 ✗」：格共用库的汇总行「✗ <阶段>：N 格里 M 格红（…）」恒是一条非 25x 的 ✗，所以「全是没跑成」那一道照字面永远记红 | 打中（合成阶段：all-not-run 那一道照字面判「红」） | 量过（真格共用库、真判法打印函数，合成阶段） | 分 G11 的「一道里每一条」与改前逐行判的写法；research-unit-tests 与 harness-model-differential-and-scenarios 同形（三道都用格共用库），另外两道的 25x 那一格上面还贴着包装自己的 ✗ |
| L3-b | crash-verifier「一格里每一条」：格段之外，汇总段里另有一条点名这一格的 ✗ | 两读：只看格段记「没跑成」，加上汇总里那一条就记「红」 | 量过（机械读法） | 轻 |
| L3-c | e165 重放「装置被信号停了」：判法写「输出不算判定」，两份定义照字面把它算真红 | 字面不一致 | 推的（没起装置） | 3-12 报告自己列了「归哪一类没写」 |
| L3-d | gate.sh 汇总里这一道退 1 的说法 | 对得上：退 1 记 FAIL、打「✗ <gate-stage 名>」，不出「没跑成」这一类；名字用文件头的 gate-stage 标签，不用文件名 | 读代码 | 轻 |

各格的改法（K1–K4）都只在我自己的模型上量过，被攻过零轮；每格修了什么、哪些还照中，见各节末尾的表。

## L1：G9 的认法与只读名单

### 被判的字面

- 只读名单：「只放过字面写明的只读用法：verus-proof-check.py 的第一个参数是 toolchain」（`.claude/hooks/lib_heavy_tests.py:69`），e165 一侧是「e165-pin-red-examples.py 照它自己的参数切法落到追加或 --check 的」（`.claude/hooks/lib_heavy_tests.py:70`）。
- python -c：「或 run_module(…) 按模块名起它们的，照那一份判（参数取代码之后的词）」（`.claude/hooks/lib_heavy_tests.py:73`）；代码里认的是 `"verus-proof-check.py": re.compile(r"verus-proof-check\.py|run_module\s*\([^)]*verus-proof-check"),`，见 `.claude/hooks/lib_heavy_tests.py:230`；判的时候走「    return verus_judge_arguments(named[0], arguments[invocation.script_index:], f"python -c 的代码里点了 {named[0]}")」（`.claude/hooks/lib_heavy_tests.py:1241`）。
- e165 那一格放行：「    if script == E165_PIN_SCRIPT and literal and e165_pin_mode(arguments) in ("append", "check"):」（`.claude/hooks/lib_heavy_tests.py:1203`）。

### 探针原样输出（`l1-output.txt`；列：退出码、期望、身份、编号说明；喂的是拷贝里的 `.claude/hooks/heavy-test-guard.sh`）

```
0	2	implementation-writer	P1 --force 占 --target 的值位，后面写 --check  <- 与期望不同
0	2	主 agent	P1 --force 占 --target 的值位，后面写 --check  <- 与期望不同
0	2	implementation-writer	P2 同上带 --root  <- 与期望不同
0	2	主 agent	P2 同上带 --root  <- 与期望不同
2	2	implementation-writer	P3 对照：--force 写在末尾
2	2	主 agent	P3 对照：--force 写在末尾
0	0	implementation-writer	P4 对照：--check 带 --force
0	0	主 agent	P4 对照：--check 带 --force
2	-	implementation-writer	P5 verus-proof-check 首参前写 --force（闸更严，期望拒）
2	-	主 agent	P5 verus-proof-check 首参前写 --force（闸更严，期望拒）
0	2	implementation-writer	C1 -c 里改 sys.argv 成 proof，代码后写 tool-versions  <- 与期望不同
0	2	主 agent	C1 -c 里改 sys.argv 成 proof，代码后写 tool-versions  <- 与期望不同
0	2	implementation-writer	C2 -c 里改 sys.argv 成 --replay，代码后不写词  <- 与期望不同
0	2	主 agent	C2 -c 里改 sys.argv 成 --replay，代码后不写词  <- 与期望不同
0	2	implementation-writer	C3 -c 里改 sys.argv 成 --replay，代码后写 --check  <- 与期望不同
0	2	主 agent	C3 -c 里改 sys.argv 成 --replay，代码后写 --check  <- 与期望不同
0	2	implementation-writer	C4 -c 里 __import__ 按模块名导入再调 main  <- 与期望不同
0	2	主 agent	C4 -c 里 __import__ 按模块名导入再调 main  <- 与期望不同
0	2	implementation-writer	C5 -c 里 importlib.import_module 起红例重放  <- 与期望不同
0	2	主 agent	C5 -c 里 importlib.import_module 起红例重放  <- 与期望不同
0	2	implementation-writer	C6 -c 里路径拼出来  <- 与期望不同
0	2	主 agent	C6 -c 里路径拼出来  <- 与期望不同
0	2	implementation-writer	C7 heredoc 喂 python3 -  <- 与期望不同
0	2	主 agent	C7 heredoc 喂 python3 -  <- 与期望不同
0	2	implementation-writer	C8 -c 里调函数 check_proof（spec_from_file_location 点了文件名），代码后写 toolchain  <- 与期望不同
0	2	主 agent	C8 -c 里调函数 check_proof（spec_from_file_location 点了文件名），代码后写 toolchain  <- 与期望不同
2	2	implementation-writer	C9 对照：-c exec 文件、代码后写 proof
2	2	主 agent	C9 对照：-c exec 文件、代码后写 proof
2	2	implementation-writer	C10 对照：python3 -m runpy 加 PYTHONPATH
2	2	主 agent	C10 对照：python3 -m runpy 加 PYTHONPATH
0	2	implementation-writer	M1 广谱变异：-c 里改 sys.argv，代码后写 --summary  <- 与期望不同
0	2	主 agent	M1 广谱变异：-c 里改 sys.argv，代码后写 --summary  <- 与期望不同
0	2	implementation-writer	M2 广谱变异：-c 里 __import__ 按模块名  <- 与期望不同
0	2	主 agent	M2 广谱变异：-c 里 __import__ 按模块名  <- 与期望不同
0	0	implementation-writer	F1 python3 -m py_compile 两份判法脚本（共用约束点名的语法检查）
0	0	主 agent	F1 python3 -m py_compile 两份判法脚本（共用约束点名的语法检查）
2	-	implementation-writer	F2 python3 -c 只读文件做语法检查（文件头写的接受的误拒）
2	-	主 agent	F2 python3 -c 只读文件做语法检查（文件头写的接受的误拒）
0	0	implementation-writer	F3 -W 选项加 tool-versions
0	0	主 agent	F3 -W 选项加 tool-versions
0	0	implementation-writer	F4 同目录名字相近的研究脚本
0	0	主 agent	F4 同目录名字相近的研究脚本
0	-	implementation-writer	F5 -c 代码里只在字符串里写了 e165 的名字、代码后不写词（照 e165 判成追加）
0	-	主 agent	F5 -c 代码里只在字符串里写了 e165 的名字、代码后不写词（照 e165 判成追加）
0	0	implementation-writer	F6 grep 读判法脚本
0	0	主 agent	F6 grep 读判法脚本
0	0	implementation-writer	F7 仓副本里的判法脚本 tool-versions
0	0	主 agent	F7 仓副本里的判法脚本 tool-versions
对不上期望的：24 条（期望写 - 的是待观测的，不计）
```

24 条与期望不同，分成三组：P1、P2（4 条），C1–C8（16 条），M1、M2（4 条，广谱变异）。

### L1-a：preflight 先摘 `--force`，闸不摘

- 两份判法脚本末尾先调 preflight：「    preflight(__file__)」（`research/scripts/e165-pin-red-examples.py:632`，`research/scripts/verus-proof-check.py:1235`）。preflight 在 main 之前把每一个整词 `--force` 从 sys.argv 里摘掉：「    remaining = [value for value in given if value != '--force']」（`.claude/singlefs-ai-sop/scripts/preflight.py:583`），然后「        sys.argv[1:] = remaining」（`.claude/singlefs-ai-sop/scripts/preflight.py:585`）。
- 闸里的 `e165_pin_mode`（`.claude/hooks/lib_heavy_tests.py:1160`）照 main 的切法走：带值的选项跳过它的值，`--check` / `--replay` 后写的那个算数。但它切的是没摘过 `--force` 的参数。于是 `--replay --target --force --check` 在闸里被切成：`--target` 的值是 `--force`，最后写的 `--check` 定模式，判 check、放行。脚本那一侧先摘 `--force`，剩下 `--replay --target --check`：`--target` 的值变成 `--check`，模式是 replay。main 里同一句判法是「        if argument in ("--check", "--replay"):」（`research/scripts/e165-pin-red-examples.py:441`）。
- 另一半照原样起脚本自己 `__main__` 里的那两步（preflight 加 main），只把 replay / check / append 换成打模式的桩（`l1-e165-parse-output.txt`）：

```
P1 参数：--replay --target --force --check
    preflight 之后 sys.argv[1:]：--replay --target --check
    真跑的模式：replay（target='--check'）
    main 退 0
P2 参数：--root <草稿>/repo --replay --target --force --check
    preflight 之后 sys.argv[1:]：--root <草稿>/repo --replay --target --check
    真跑的模式：replay（target='--check'）
    main 退 0
P4 对照 参数：--check --force
    preflight 之后 sys.argv[1:]：--check
    真跑的模式：check
    main 退 0
```

- replay 不给 `--binary` 时，先 `cargo build --release --locked -p singlefs-checker-tier`，再经包装起装置。这是重型那一类，闸放过去了。
- 先例：切词库给 detached-run.sh 做的正是同一件事，「    return [argument for argument in arguments if argument != "--force"]」（`.claude/hooks/lib_shell_words.py:523`）。闸在判的那一刻看得到 `--force`，判别子是观测得到的。
- verus-proof-check 一侧没有同形的口子：它的第一个参数就是子命令，摘掉 `--force` 不会让第一个参数变成另一个只读子命令。P5（`--force tool-versions`）闸判拒，比实际更严。

### L1-b、L1-c：python -c 只看代码之后的词

- 代码自己能改 sys.argv（C1、C3），能直接调函数（C8：`spec_from_file_location` 载入之后调 `run_check("proof", …)`，代码后面写 `toolchain`），也能按模块名导入（C4 `__import__("verus-proof-check")`、C5 `importlib.import_module("e165-pin-red-examples")`，这两种正则认不出）。e165 一侧更松：代码之后什么都不写，`e165_pin_mode([])` 落到 append、放行（C2）。
- 换成同名的桩看被起的脚本收到什么参数（`l1-argv-demo-output.txt`，桩只打 sys.argv）：

```
C1 代码之后的词：tool-versions；桩看到的参数： proof .
C3 代码之后的词：--check；桩看到的参数： --replay
```

- 判别子：任意代码拿 sys.argv 做什么，闸在执行前看不出，这是「判别子观测不到」那一种。所以改法不是认得更细，而是一律不放：-c 的代码里点到这两份脚本（文件名或模块名）就整条算 verus-stage。
- 不分辨臂：广谱变异照的是同一套（M1 代码之后写 `--summary`、M2 `__import__("broad-mutation")`，都退 0）。G9 的「认法照广谱变异那一套」把那一套的口子一起带进来了。

### L1-d、L1-e：照放的老口子与误拒

- C6（代码里把路径拼出来）、C7（heredoc 喂 `python3 -`）照放。广谱变异那一段文件头写过这类口子：「  `python3 - < 脚本` 与 heredoc 喂给 python 的代码；uv run、pypy 这类别的解释器；multi-host-run.sh --、xargs、find -exec、eval（对每一类重型都在的老口子）。」（`.claude/hooks/lib_heavy_tests.py:129`）。Verus 这一段的罩不住清单是「罩不住的：改名拷贝（两份没有签名行）、同一条命令里先 ln -s / cp 再起的、`env -C` 与目录放变量、`script -qc`、xargs（lib_shell_words.py 那一侧的老口子）。」（`.claude/hooks/lib_heavy_tests.py:74`），里面没有这两种。记账，不另立格。
- 误拒没打中：F1（`python3 -m py_compile` 两份脚本，共用约束点名的语法检查）、F3（`-W ignore` 加 tool-versions）、F4（同目录、名字相近的 `verus-proof-check-notes.py`）、F6（grep）、F7（仓副本里的判法脚本 tool-versions），都退 0。
- 接受的误拒，量到一次真实的：我自己 L2 探针第一版里写了 `verus-proof-check.py tool-versions "$COPY"`，闸读进脚本，按「参数里有 `$`」拒掉整条 Bash（拒绝说明原样摘一句：「research/scripts/verus-proof-check.py tool-versions $COPY（门禁 checker-tier-formal-verification 的判法：起 Verus 或编 checker 档装置重放；…）（Verus）：three-way-attack 不跑「Verus」」）。这一条正是文件头写的接受的误拒，探针改成经阶段 `--tool-versions` 起。阶段自己文件里写的是 `exec python3 "$VERUS_CHECK" tool-versions "$ROOT"`，阶段按名字判，闸不读进去，不受影响。
- F2（-c 里只读文件做语法检查）、F5（-c 只在字符串里提到 e165 的名字）只观测：F2 改前就拒（接受的误拒）；F5 改前放，K2 之下改拒，算 K2 新加的误拒。

### 打中之后的四句（L1-a、L1-b、L1-c）

| 问 | L1-a（`--force` 值位） | L1-b / L1-c（python -c） |
|---|---|---|
| 分不分辨臂 | 不分 G9 与改前那一版（两版都用 `e165_pin_mode`）；病根在两版共用的「照它自己的参数切法」 | 分 G9 字面「参数取代码之后的词」；不分 G9 与广谱变异那一套（同形照放） |
| 系统当时看不看得到判别它的东西 | 看得到：命令文本里有 `--force`，切词库给 detached-run.sh 就是这么摘的 | 看不到：代码对 sys.argv 做什么，执行前判不出。所以改成一律算 |
| 满足判据字面的哪一句 | `.claude/hooks/lib_heavy_tests.py:70` 那一句「照它自己的参数切法」：它自己的切法前面还有 preflight 那一步，闸没照 | `.claude/hooks/lib_heavy_tests.py:73`「照那一份判（参数取代码之后的词）」 |
| 跑前给的改法在打中的格上还中不中 | 第二轮判决采的是 G9 整套，没有单独的改法 | 同左 |

### 改法与量过的表（只在我的模型上量过、被攻过零轮）

K1：判参数之前照 preflight 摘掉每一个整词 `--force`。K2：-c 的代码里点到两份脚本（文件名或模块名，正则放宽成只认名字）就一律算 verus-stage，不看代码之后的词。草稿 `fix-draft-l1.py`，改的是拷贝里的 `lib_heavy_tests.py` 三处。

| 格 | 改前 | K1 + K2 之后 | 标 |
|---|---|---|---|
| P1、P2 | 退 0 | 退 2 | 量过 |
| C1、C2、C3、C8 | 退 0 | 退 2 | 量过 |
| C4、C5 | 退 0 | 退 2 | 量过 |
| C6、C7 | 退 0 | 退 0（K 碰不到：路径拼出来、heredoc） | 量过 |
| M1、M2（广谱变异） | 退 0 | 退 0（K 没改广谱变异那一支） | 量过 |
| P5（`--force tool-versions`） | 退 2 | 退 0（与真实行为一致） | 量过 |
| F5（-c 只提名字） | 退 0 | 退 2（K2 新加的误拒） | 量过 |
| R1–R4 那几种只读用法 | 3-12 量过退 0 | 推的：K 没碰首参判法与阶段判法 | 推的 |

K 之后的原样（`l1-K-output.txt` 里对不上期望的那几行与末行）：

```
0	2	implementation-writer	C6 -c 里路径拼出来  <- 与期望不同
0	2	主 agent	C6 -c 里路径拼出来  <- 与期望不同
0	2	implementation-writer	C7 heredoc 喂 python3 -  <- 与期望不同
0	2	主 agent	C7 heredoc 喂 python3 -  <- 与期望不同
0	2	implementation-writer	M1 广谱变异：-c 里改 sys.argv，代码后写 --summary  <- 与期望不同
0	2	主 agent	M1 广谱变异：-c 里改 sys.argv，代码后写 --summary  <- 与期望不同
0	2	implementation-writer	M2 广谱变异：-c 里 __import__ 按模块名  <- 与期望不同
0	2	主 agent	M2 广谱变异：-c 里 __import__ 按模块名  <- 与期望不同
对不上期望的：8 条（期望写 - 的是待观测的，不计）
```

## L2：G10 的 tool-versions

### 被判的字面

- 「      设了写它的值、它指的文件与那个文件的 sha256：值里没有 / 的照 PATH 找到那一份，与 Verus 的找法相同，PATH 里找不到写「找不到」）」这一句（`research/scripts/verus-proof-check.py:48`）。顶层摘要的代码是「                for name in sorted(os.listdir(directory), key=lambda entry: entry.encode("utf-8", "surrogateescape")):」（`research/scripts/verus-proof-check.py:809`）与「                    if os.path.isfile(full):」（`research/scripts/verus-proof-check.py:811`）：只取顶层的普通文件。
- 安装（`verus-pins.tsv` release 行第五列）在 `~/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux`，共 1.6G。顶层有 54 份普通文件（`ls -A` 数的），子目录是 cargo 编译目录的布局（`build/`、`deps/`、`.fingerprint/`、`incremental/`、`examples/`），子目录里有 3819 份文件（`find -mindepth 2 -type f | wc -l`）。

### 探针原样输出（`l2-output.txt`）

```
== 1. Verus 读了安装里哪些文件（strace，原样环境）==
  S0-原样-strace	退 0	"verified": 1	stderr 末行：
  安装顶层被打开或执行的文件：8 个：libverus_builtin_macros.so libverus_builtin.rlib libverus_state_machines_macros.so libvstd.rlib rust_verify verus vstd.vir ./z3 
  安装子目录里被打开的文件：0 个
  安装之外 ~/.rustup 下被打开的文件：44 个（含 libLLVM.so.22.1-rust-1.98.1-stable librustc_driver-eaf3ed1d23ca5027.so settings.toml ）
  往安装目录里写（O_WRONLY / O_RDWR / O_CREAT）：0 次
  S1-VERUS_ROOT=A-strace	退 0	"verified": 1	stderr 末行：
  VERUS_ROOT=A 时从 A 打开的文件：5 个：libverus_builtin_macros.so libverus_builtin.rlib libverus_state_machines_macros.so libvstd.rlib vstd.vir ；从原安装打开的：3 个：rust_verify verus ./z3 
== 2. 环境变量换了，Verus 判不判得出来 ==
  T1-原样	退 0	"verified": 1	stderr 末行：
  T2-VERUS_ROOT=B	退 1	"verified": 0	stderr 末行：For more information about this error, try `rustc --explain E0463`.
  T3-VERUS_ROOT=A	退 0	"verified": 1	stderr 末行：
  T4-VERUS_USE_RUSTUP=0	退 127		stderr 末行：/home/fy5090/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/rust_verify: error while loading shared libraries: librustc_driver-eaf3ed1d23
  T5-RUSTUP_HOME=空目录	退 1		stderr 末行：help: run `rustup toolchain install 1.98.1-x86_64-unknown-linux-gnu` to install it
== 3. tool-versions 那一行（摘要前 16 位）稳不稳、几种环境下变不变，单次挂钟 ==
  第 1 次：8a10e7baebe2a413（0.98 秒）
  第 2 次：8a10e7baebe2a413（0.29 秒）
  第 3 次：8a10e7baebe2a413（0.27 秒）
  玩具 Verus 跑过之后：8a10e7baebe2a413
  VERUS_ROOT=<草稿>/B：8a10e7baebe2a413
  VERUS_USE_RUSTUP=0：8a10e7baebe2a413
  RUSTUP_HOME=<草稿>/empty-rustup-home：8a10e7baebe2a413
  VERUS_Z3_PATH=：e1a638b2fafad91b
== 4. 复用判定：判绿时原样，之后换环境（q1 stage-marker-check 退 0 = 复用；q3 gate-reuse 退 10 = 复用）==
  对照 不换	q1 退 0	q3 退 10
  对照 Y1 VERUS_Z3_PATH 未设 → 空串（G10 修的那一格）	q1 退 1	q3 退 0
  Z1 VERUS_ROOT 未设 → 拷贝 B（宏库截断，T2 编不过）	q1 退 0	q3 退 10
  Z2 VERUS_USE_RUSTUP 未设 → 0（T4 起不来）	q1 退 0	q3 退 10
  Z3 RUSTUP_HOME 换成空目录（T5）	q1 退 2	q3 退 0
```

### L2-a、L2-b：没打中

- 子目录：Verus 验玩具文件那一趟，strace 看到安装子目录里被打开的文件 0 个。被打开或执行的 8 份都在顶层，都进了顶层摘要。取样只有一个玩具文件、一组选项（`--no-cheating --rlimit 10 --output-json`），没覆盖 vstd 之外的宏（state machines 那一份在原样那一趟里同样被打开了）。被判的 crate 用到别的特性时，会不会读子目录没量。
- 会变的东西：Verus 往安装目录里写 0 次。摘要连算三次，加上 Verus 跑过之后再算一次，四次的前 16 位都是 `8a10e7baebe2a413`。摘要只取文件名与内容，不取 mtime。顶层那几份 `.cargo-lock` 一类是空文件，有人拿这个目录当 cargo 编译目录时才会变；没量。
- 代价：经阶段 `--tool-versions` 起一次要 0.27–0.98 秒（第一次冷读 0.98 秒，之后约 0.28 秒）。

### L2-c：VERUS_ROOT、VERUS_USE_RUSTUP（打中）

- rust_verify 二进制里出现的 `VERUS_` 开头的串：`VERUS_DRIVER_VIA_CARGO VERUS_ROOT VERUS_SPEC VERUS_SPH1 VERUS_SPH3 VERUS_UNERASED_P VERUS_Z3_PATH`，launcher `verus` 里出现的是 `VERUS_USE_RUSTUP`（`strings | grep -oE '\bVERUS_[A-Z0-9_]+'`，哪几个是环境变量没逐个核）。
- VERUS_ROOT：strace 显示设成拷贝 A 时，宏库、libverus_builtin.rlib、state machines 宏库、libvstd.rlib、vstd.vir 这 5 份从 A 读，rust_verify、verus、z3 照旧从原安装读（S1 那一行）。设成只把宏库截断的拷贝 B 时，Verus 退 1（T2「can't find crate for `vstd`」）；设成逐字节相同的 A 时退 0（T3）。tool-versions 那一行不变（`VERUS_ROOT=<草稿>/B` 那一行仍是 `8a10e7baebe2a413`），复用两问都复用（Z1：q1 退 0、q3 退 10）。这是 Y2（宏库截断）的另一条路，第二轮修的是安装目录里那一份，环境变量这条路没罩。
- VERUS_USE_RUSTUP=0：launcher 不经 rustup 起 rust_verify，加载 librustc_driver 失败、退 127（T4）。tool-versions 不变，复用两问都复用（Z2）。与 Y1（VERUS_Z3_PATH 空串，Verus 起不了 z3）同形。
- 对照：Y1 本身仍被 G10 罩着（q1 退 1、q3 退 0），这是 G10 修过的那一格。

### L2-d：rustup 工具链（部分罩着）

- Verus 经 `rustup run 1.98.1-x86_64-unknown-linux-gnu` 起 rust_verify，原样那一趟打开了 `~/.rustup` 下 44 份文件（librustc_driver、libLLVM、settings.toml 与 std 的 rlib）。这些都不在顶层摘要里。tool-versions 只打 version.json 里的 `toolchain=`。
- RUSTUP_HOME 换成空目录（T5，Verus 退 1）：复用判定 q1 退 2，原样报「判不了 checker-tier-formal-verification.sh 的全绿标记：cargo -V 退 1」，q3 退 0。拦住它的是指纹里的 `cargo -V`，不是 tool-versions。`cargo -V` 走的是默认工具链，不是 1.98.1；同名 1.98.1 的内容被换掉（例如 `rustup toolchain link`）这一种推的是照样复用，没量。3-12 报告把「rustup 工具链里的 librustc_driver」「VERUS_ROOT 等环境变量」列成罩不住的，但仓内文件里没写：`grep -n 'rustup\|librustc_driver\|VERUS_ROOT'` 三份文件（verus-proof-check.py、阶段文件、stage-inputs.tsv）命中 0 处。

### 打中之后的四句（L2-c）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 不分 G10 与改前（两版都只认安装目录与 VERUS_Z3_PATH）；与 J5–J7 同一个病根：tool-versions 只记列举过的维度 |
| 系统当时看不看得到 | 看得到：tool-versions 在判的那一刻读得到 os.environ |
| 满足判据字面的哪一句 | tool-versions 的用处「进复用判定与全绿标记的指纹」（`research/scripts/verus-proof-check.py:49` 那一行括注）：换了 Verus 实际读的东西就不该复用 |
| 跑前给的改法还中不中 | G10 的三项（顶层摘要、未设与空串、裸名照 PATH）在 Z1、Z2 上都照中（量过：改后仍复用） |

### 改法 K3（只在我的模型上量过、被攻过零轮）

`fix-draft-l2.py`：tool-versions 另打 `VERUS_ROOT`（分未设、空串、值；值是目录时加上那个目录的顶层摘要）与 `VERUS_USE_RUSTUP`（分未设、原值）。

| 格 | 改前 | K3 之后 | 标 |
|---|---|---|---|
| Z1 VERUS_ROOT → 拷贝 B | q1 0、q3 10（复用） | q1 1、q3 0（不复用） | 量过 |
| Z2 VERUS_USE_RUSTUP=0 | q1 0、q3 10 | q1 1、q3 0 | 量过 |
| 对照「不换」 | 复用 | 复用（q1 0、q3 10） | 量过 |
| 对照 Y1 | 不复用 | 不复用 | 量过 |
| 同名工具链内容被换 | 推的复用 | 推的复用（K3 碰不到） | 推的 |
| 摘要稳定 | 三次加 Verus 之后相同 | 三次加 Verus 之后相同（`14ce186ea86f9a3b`） | 量过 |

K3 之后的原样（`l2-K3-output.txt` 第 3、4 节）：

```
== 3. tool-versions 那一行（摘要前 16 位）稳不稳、几种环境下变不变，单次挂钟 ==
  第 1 次：14ce186ea86f9a3b（0.29 秒）
  第 2 次：14ce186ea86f9a3b（0.27 秒）
  第 3 次：14ce186ea86f9a3b（0.29 秒）
  玩具 Verus 跑过之后：14ce186ea86f9a3b
  VERUS_ROOT=<草稿>/B：20bcbfdd9ac686a5
  VERUS_USE_RUSTUP=0：4294cd6ccdbae7c2
  RUSTUP_HOME=<草稿>/empty-rustup-home：14ce186ea86f9a3b
  VERUS_Z3_PATH=：9a582bfd43f6f138
== 4. 复用判定：判绿时原样，之后换环境（q1 stage-marker-check 退 0 = 复用；q3 gate-reuse 退 10 = 复用）==
  对照 不换	q1 退 0	q3 退 10
  对照 Y1 VERUS_Z3_PATH 未设 → 空串（G10 修的那一格）	q1 退 1	q3 退 0
  Z1 VERUS_ROOT 未设 → 拷贝 B（宏库截断，T2 编不过）	q1 退 1	q3 退 0
  Z2 VERUS_USE_RUSTUP 未设 → 0（T4 起不来）	q1 退 1	q3 退 0
  Z3 RUSTUP_HOME 换成空目录（T5）	q1 退 2	q3 退 0
```

## L3：G11 的没跑成与真红

### 被判的字面

- gate-triage 第 3 步（`.claude/agents/gate-triage.md:29`，这一行有上千字，这里抄的是其中连续的两段）：「一道里每一条 ✗ 都写着内存包装退 25x 的」，以及「同一道里另有一条不是这样写的 ✗（真红）的，那一道记红、照真红那几条点名的文件与行归属，25x 那几条连同各自的 → 一并原样抄」。
- crash-verifier 第 3 步（`.claude/agents/crash-verifier.md:32`）：「checker-tier-formal-verification 一格里每一条」，后面接「✗」都写着「没跑成：…」的记没跑成，另有一条不是这样写的记红。
- 格共用库的汇总：有红格就打「    printf '  ✗ %s：%s 格里 %s 格红（%s）\n' "$STAGE_CELLS_STAGE_NAME" "${#STAGE_CELLS_SELECTED[@]}" "${#red[@]}" "${red[*]}"」（`.claude/gate.d/lib/stage-cells.sh:274`）。三道（checker-tier-formal-verification、checker-tier-research-build-and-replay、harness-model-differential-and-scenarios）都 source 这一份（`grep -l stage-cells.sh .claude/gate.d/*.sh` 列出了这三道）。

### 探针

合成阶段 `l3-stage.sh`：拷贝里的真格共用库，与真阶段同名同格名，标题与出路从真阶段文件里抠出来。必红世界那一格的输出由 `l3-cell.py` 调判法脚本自己的 `outcome_problem`、`problem_howto`、`report_cell` 打出（捏造 252、254 两个结局，不起 Verus、不起包装）。分三种：全是没跑成、混合、全是真红。`l3-judge.py` 照两份定义的字面机械地判。

全是没跑成那一道的原样输出（`l3-stage-all-not-run.txt`）：

```
── verus-toolchain-pinned：Verus 安装的 version 与 commit、Cargo.lock 里六个包的版本、本机 cargo 缓存里包源码的出处与内容指纹都与登记相同
  ✓ 合成：版本一致
── verus-must-red-worlds：必红世界表每行改坏核之后 Verus 都报错且出现期望片段，原文恰好命中一次
  ✗ 第 2 行「排不上 2」：没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 252（内存不够排不上，上限 4G），这一次的输出不算判定；包装自己打的：「✗ 内存不够排不上：等了 600 秒」「→ 怎么办：bash research/scripts/run-with-memory-cap.sh --status 看谁占着」
     → 怎么办：照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办：252 是排队等满还放不下：先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑，不改证明、核或必红世界表
  ✗ 第 3 行「排不上 3」：没跑成：内存包装 research/scripts/run-with-memory-cap.sh 退 254（被总上限挤掉，上限 4G），这一次的输出不算判定；包装自己打的：「✗ 被总上限挤掉」「→ 怎么办：看 --status」
     → 怎么办：照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办：254 是 slice 满了被挤掉：先跑 bash research/scripts/run-with-memory-cap.sh --status 看 slice 被谁占着，空出来再重跑这一格；不拿掉包装跑，不改证明、核或必红世界表
── 汇总：checker-tier-formal-verification.sh 跑了 2 格（全部登记的格）
  verus-toolchain-pinned：绿
  verus-must-red-worlds：红（退出 1）
  ✗ checker-tier-formal-verification.sh：2 格里 1 格红（verus-must-red-worlds）
     → verus-must-red-worlds：锚点腐化就照今天的核改那一行的原文；改坏了还验得过就补证明或收紧 spec；表在 .claude/gate.d/verus-pins.tsv 的 must-red-worlds 行点名的路径；✗ 行写「没跑成：内存包装」的照 research/scripts/run-with-memory-cap.sh 文件头「退出码」一段办，不改表或证明
     → 改完只重跑红的格：bash .claude/gate.d/checker-tier-formal-verification.sh --check verus-must-red-worlds
合成阶段退 1
```

三种格照字面判出来的（`l3-judge-output.txt`）：

```
l3-stage-all-not-run.txt：格里 ✗ 2 条（没跑成形 2 条）；整道 ✗ 3 条（没跑成形 2 条）
  crash-verifier 读法 a（只看格段）：没跑成；读法 b（加汇总里点名这一格的 ✗）：红
  gate-triage（整道）：红；整道里不是没跑成形的 ✗：['✗ checker-tier-formal-verification.sh：2 格里 1 格红（verus-must-red-worlds）']
l3-stage-mixed.txt：格里 ✗ 4 条（没跑成形 2 条）；整道 ✗ 5 条（没跑成形 2 条）
  crash-verifier 读法 a（只看格段）：红；读法 b（加汇总里点名这一格的 ✗）：红
  gate-triage（整道）：红；整道里不是没跑成形的 ✗：['✗ 第 4 行「改坏了也验得过」改坏之后 Verus 照样验过（verified 2、errors 0）：证明没盯住这一处', '✗ 这一格 1 条真红、2 条没跑成：按红记，不是没跑成（没跑成的那几处重跑不替真红作保）', '✗ checker-tier-formal-verification.sh：2 格里 1 格红（verus-must-red-worlds）']
l3-stage-all-red.txt：格里 ✗ 1 条（没跑成形 0 条）；整道 ✗ 2 条（没跑成形 0 条）
  crash-verifier 读法 a（只看格段）：红；读法 b（加汇总里点名这一格的 ✗）：红
  gate-triage（整道）：红；整道里不是没跑成形的 ✗：['✗ 第 4 行「改坏了也验得过」改坏之后 Verus 照样验过（verified 2、errors 0）：证明没盯住这一处', '✗ checker-tier-formal-verification.sh：2 格里 1 格红（verus-must-red-worlds）']
```

### L3-a：gate-triage 在「全是没跑成」那一道走不通（打中）

- 「一道」的输出里永远多一条格共用库打的汇总 ✗「checker-tier-formal-verification.sh：N 格里 M 格红（…）」。它不是「内存包装退 25x」的写法，所以「一道里每一条 ✗ 都写着内存包装退 25x 的」照字面永远不成立。250、252、254、251 那几种归类都到不了。分诊员按后半句记红，然后「照真红那几条点名的文件与行归属」：汇总那条 ✗ 没点任何文件与行，只能落到「分不清」。这一道其实是排队排不上（252）或被挤掉（254），本该记「并发」。
- 另外两道更早就破：research-unit-tests 那一格在 25x 那条 ✗ 上面先贴「    tail -8 <<<"$out" | sed 's/^/     /'」（`.claude/gate.d/checker-tier-research-build-and-replay.sh:189`），harness-model-differential-and-scenarios 先贴「      tail -40 "$log"」（`.claude/gate.d/harness-model-differential-and-scenarios.sh:197`）。包装自己会打一行 ✗。用法错那一种量过（`l3-wrapper-251.txt`）：

```
  ✗ 内存上限要写成 正整数加 K / M / G / T，单位必写（例 16G、512M；不带单位的数 systemd 当字节数），收到的是「1Q」
  → 怎么办：照 systemd 的 MemoryMax 写法给；默认值与依据写在文件头
rc=251
```

  这一行也不是「内存包装退 25x」的写法，落在同一道里（推的：没起那两道，只读了代码、量了包装的输出）。
- 改前（3-12 之前）的写法是逐行的：「✗ 行写着内存包装退 25x 的 … 不按它点名的文件判」，不全称量化。那时汇总 ✗ 与 25x 那条各判各的，252 那条照样归「并发」。所以打中分得出 G11 的「一道里每一条」与改前的写法。改前写法的毛病（J10：真红被盖住）是另一格。
- 混合与全是真红两种格：两份定义都记红，与判法打的「按红记」一致，走得通。

### L3-b：crash-verifier「一格里」有两种读法（轻）

照格段读（「── <格名>：」到下一个「──」），全是没跑成那一格记「没跑成」。若把汇总段里点名这一格的 ✗（「…1 格红（verus-must-red-worlds）」）也算进「一格里」，就记「红」。同一份输出里汇总段自己也写「verus-must-red-worlds：红（退出 1）」，汇总的 → 还写「改坏了还验得过就补证明」，与定义要记的「没跑成」打架。不过汇总的 → 末尾写了「✗ 行写「没跑成：内存包装」的照 … 办，不改表或证明」，按格段读走得通。

### L3-c：e165 的「装置被信号停了」（字面不一致，推的）

判法写「装置被信号停了的（直接起时 subprocess 交负的信号号，经包装或 bash 起时退 128 + 信号号、不在 250–254 里，例 137 = 外面 kill -9），判红，」（`research/scripts/e165-pin-red-examples.py:32`），✗ 行写「输出不算判定」（`research/scripts/e165-pin-red-examples.py:412`）。这一行不是「没跑成：内存包装」的写法，两份定义照字面都记成真红，gate-triage 还要「照真红那几条点名的文件与行归属」，而这一行没点文件。被外面 kill -9 的一趟于是记成「红 / 分不清」，不是「并发」：gate-triage 认「并发」的条件是输出里一行裸的 Terminated，这一行不是。3-12 报告自己写了「gate-triage 里「装置被信号停了」归哪一类没写（只改了出路）」。另外，e165 的包装结局表把 253 也算进去：「WRAPPER_OWN_EXITS = {250: "撞了这一条自己的内存上限", 251: "包装用法错或起不了 scope", 252: "内存不够排不上", 253: "超过限时", 254: "被总上限挤掉"}」（`research/scripts/e165-pin-red-examples.py:82`）。外面设了 RUN_WITH_MEMORY_CAP_TIME_LIMIT 时，e165 会打「没跑成 … 退 253」。gate-triage 这时按前半句判 253 ⇒「环境」，按后半句判「checker-tier-formal-verification 的 253 … 按红记」，两句冲突。推的，门禁里谁会设这个变量没查到（三道阶段文件里只在出路文字里提到它）。

### L3-d：gate.sh 汇总

阶段退 1 记 FAIL，汇总打「    *)      bad "${STAGES[$i]}"; failed=$((failed+1)) ;;」（`.claude/singlefs-ai-sop/scripts/gate.sh:778`），也就是「✗ <gate-stage 名>」，后面跟「红色阶段的细节与出路在上方对应段落里」。没有「没跑成」这一类；不前移 staged-green，下一趟照跑，方向是多跑，与定义不冲突。名字是文件头的 gate-stage 标签「形式化验证（里程碑三七·5：…）」，不是文件名，分诊员要自己对上（这一点所有阶段都一样，轻）。

### 打中之后的四句（L3-a）

| 问 | 答 |
|---|---|
| 分不分辨臂 | 分：G11 把逐行判改成「一道里每一条」全称，打中的是这一改；改前的逐行写法在这一格上走得通（推的：照改前原文读） |
| 系统当时看不看得到 | 看得到：汇总 ✗ 的写法固定（「<阶段文件名>：N 格里 M 格红」），包装自己的 ✗ 在 25x 那条的上面或行尾，分诊员都能认出来 |
| 满足判据字面的哪一句 | `.claude/agents/gate-triage.md:29` 的「一道里每一条 ✗ 都写着内存包装退 25x 的」，不是 crash-verifier 那一句（那一句的范围是「一格」） |
| 跑前给的改法还中不中 | 第二轮判决采的 G11 原话是「一格里每一条 ✗ 都是没跑成才记没跑成」，落到 gate-triage 时被写成了「一道」。按「一格」写，汇总 ✗ 不在格里；但 research-unit-tests 那一格自己的段里还贴着包装的 ✗，照中（推的） |

### 改法 K4（只在我的合成阶段上量过、被攻过零轮）

gate-triage 的全称只数格自己打的 ✗，写明三样不算：格共用库的汇总行「✗ <阶段文件名>：N 格里 M 格红（…）」；包装自己打的 ✗ 与 →（25x 那条上面贴的尾巴，或 Verus 一侧行尾「包装自己打的：」里带出的）；gate.sh「门禁结果」里那一行。另一条路是让格共用库自己把全是没跑成的格汇总成「没跑成」，定义只认汇总行（推的，没写没量）。

| 格 | 3-12 字面 | K4 读法 | 标 |
|---|---|---|---|
| 全是没跑成（252 + 254） | 红 | 没跑成，按码归「并发」 | 量过（`l3-K4-output.txt`） |
| 混合 | 红 | 红 | 量过 |
| 全是真红 | 红 | 红 | 量过 |
| research-unit-tests 带包装尾巴的 25x | 红 | 没跑成（K4 第二样把尾巴排除了） | 推的 |
| e165 被信号停了 | 红 / 分不清 | 红 / 分不清（K4 不碰） | 推的 |

```
l3-stage-all-not-run.txt：K4 读法下 gate-triage（整道，去掉汇总行）：没跑成（按码归环境 / 并发）
l3-stage-mixed.txt：K4 读法下 gate-triage（整道，去掉汇总行）：红
l3-stage-all-red.txt：K4 读法下 gate-triage（整道，去掉汇总行）：红
```

## 没打中的形状

- L1：同目录名字相近的脚本、`-m py_compile`、`-W` 选项、仓副本里的判法脚本、grep，共 5 种 × 2 个身份，没有误拒（F1、F3、F4、F6、F7）。只读名单本身：toolchain、fingerprint、tool-versions 读过代码，都不起子进程（`check_toolchain`、`package_source_fingerprint`、`tool_versions` 里没有 subprocess）；e165 的 append、check 两条路里也没有 subprocess（脚本里 subprocess 只出现在 replay 与 selftest）。preflight 在 `admission: always`、`run-condition: none` 之下不跑别的命令（读代码）。
- verus-proof-check 一侧的 `--force` 移位：没打中（P5 闸更严）。
- L2：子目录（strace 看到 0 个文件被打开）；会变的东西（四次摘要相同，Verus 不往安装里写）。取样是一个玩具文件、一组选项、一台机器、一次。
- L3：混合与全是真红两种格，两份定义都走得通；crash-verifier 照格段读，全是没跑成那一格也走得通。

## 这条腿自己的限度

- L1 探针只喂 JSON、看退出码；P1 的「真跑的是 replay」是把 replay 换成桩之后看模式量出来的，没真编装置。C1、C3 的「被起的脚本看到代码里的参数」是在同名的桩上量的。
- L2 的 strace 只跑了一个三行玩具文件，没跑被判的 crate（那属于判法的干活用法）。VERUS_ROOT 设成 B 时 Verus 退 1，报的是「can't find crate for `vstd`」，B 与 A 只差宏库这一份；为什么报成找不到 vstd 没追。
- L3 的合成阶段用的是真格共用库、真判法打印函数，但不是真阶段；gate-triage、crash-verifier 照字面怎么读，是 `l3-judge.py` 机械地套的，不是派一条真的分诊员去读。
- K1–K4 都只在我自己的模型上量过，没跑 `--selftest`，没量对已有自证格的影响。
- 一条腿只抽了一次样；「没打中」的几格照 `.claude/rules/three-way-inference.md` 不算数。

## 没做什么

- 没跑重型测试、门禁阶段、判法脚本的干活用法、任何 `--selftest`；起过的只有：钩子（喂 JSON）、admission.py 的 stage-marker-write、stage-marker-check、gate-reuse（拷贝自己新建的 .git 里）、阶段 `--tool-versions`、直接起的 Verus 验玩具文件（经包装 400M），以及包装用法错那一下（`1Q true`，退 251，命令一行都没跑）。
- 包装的结局记录：第一批玩具 Verus 用 2G 上限时，T2、T3、T5 被包装以 251 拒（「写 auto（取实测峰值的 1.5 倍，303M），或写不超过 404M 的上限」）；第二批改成 400M，T3、T5 又被以 251 拒（「写不超过 40M 的上限」：峰值表里记下的是 T2 失败那一趟的小峰值）；第三批给每种各自的键（RUN_WITH_MEMORY_CAP_KEY）与超过峰值的理由再跑。这几次被拒的输出都不算结果，报告里的数来自 `l2-install.sh` 那一趟与复跑。
- 不判 L4，不替主 agent 采纳；副本上量出的数不算入库装置上的数。
- 没读禁读清单里的同轮辩方报告与模型。
- 草稿目录 `/tmp/claude-1000/defs-m3-formal-gate-r3-opus/` 留着探针输出与复跑输出（约 900K），产物已经拷进模型目录。删掉的仓副本：`repo` 462M、`repo-k` 462M、`repo-l2` 507M、`repo-l2k` 507M、`rerun1/repo` 463M、`rerun1/repo-k` 463M、`rerun1/repo-l2` 507M、`rerun1/repo-l2k` 507M；安装拷贝 A、B（硬链接，删掉之后原安装宏库那份的链接数回到 1）也删了。
