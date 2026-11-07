#!/usr/bin/env python3
# admission: always 每次照此刻的源码、清单、Cargo.lock 与工具链现算代码键，上一次算出的键不替这一次作保
# run-condition: none 要的 unit-graph 由它自己按 --unit-graph、SINGLEFS_UNIT_GRAPH、<仓根>/target/release/unit-graph 的次序找，找不到退 78（出路 cargo build --release -p singlefs-unit-graph）；run 里的 cargo 与装置由它自己起、自己判
"""实验装置的代码键：装置入口函数的单元闭包键，加工具链行与闭包真引到的外部依赖行。
E178 第四段与 E174 父进程的续跑键都由它取（登记 research/prompts/e178-preregistration.md 修订 4 之五、之六与修订 5，
research/prompts/e174-preregistration.md 修订 2 之五与修订 3）；装置拿它交的代码键再拼上一项（一轮）自己的参数，算项键（轮键）。

用法（<仓根> 是仓根：装着 crates/ 与 scripts/experiment-apparatus 的那一棵树）：
  device-code-key.py key <仓根> --binary <bin 名> [--package-directory <包目录>] --entry <fn 路径>… [--feature <特性>]… [--unit-graph <程序>] [--threads <N>] [--host 本机|第二台]
      标准输出打组成清单（行格式见「组成清单」），每个入口一行 code_key。闭包有拒算：照打拒算行、不打 code_key、退 3。
      --package-directory 是装置那个包相对仓根的目录，不给是 scripts/experiment-apparatus（下面「包目录」）。
      --threads 只给自证；交给统一任务服务的命令里不写，写了被拒（不写时取登记的每台默认重活线程数）。
  device-code-key.py run <仓根> <与 key 相同的选项> -- <装置参数…>
      先取键；再在构建根里 cargo build --release -p <包名> --bin <bin 名> [--features <特性,…>]；再起装置
      （<构建根>/<CARGO_TARGET_DIR 或 target>/release/<bin 名>，装置参数后面接 --code-key-file <组成清单>），装置读写判定库经这里转给 prune-server（下面「判定库的转交」）；
      装置退出后再取一次键：有一个入口的代码键与第一次不等，把这一趟经转交写进库的每一条改写成撤回标记、退 1
      （红句「源码在编译与跑之间变了」）；都相等就交装置的退出码。
      装置退出后那一次取不到键（unit-graph 崩了、找不到，拒算、工具链起不来，两种一样办）：核不了编译与跑之间源码变没变，键核不了的不入库——
      撤掉这一趟写进库的每一条（库里原有的、这一趟没写的不动），装置退非 0 交装置的退出码，退 0 时这里退 3（同开跑前键算不出那一条）；
      说一句「装置退出之后键取不到」，不改成 1 或 2。
      键算不出（拒算、工具链起不来）：照编、照起装置，装置参数原样，只是不给 --code-key-file、不开转交（不读不写库，不退回别的开关），装置退 0 时这里退 3；
      装置自己要不要代码键、没有它怎么办由装置定（E178、E174 不给 --code-key-file 退 2）。
      不认「键算不出时换一组装置参数」这类选项：写了按「认不得的参数」退 2。
      两种起法都给装置的环境另放 DEVICE_CODE_KEY_DEVICE_BINARY=<起的那一份装置的路径>：装置要起自己的子进程照它起（E174 的 run），不读自己的路径
      （单元键认不出 std::env::current_exe 这种读自己路径的起法，整个 main 闭包拒算）。
  device-code-key.py --selftest

包目录（--package-directory，只认两种；别的退 2）：
  scripts/experiment-apparatus  自成一个工作区（它自己的 Cargo.toml 里有 [workspace]、自己的 Cargo.lock）：构建根是这个目录，unit-graph 的根是 <仓根>/scripts
  crates/<名>                   仓根工作区的成员：构建根是仓根（仓根的 Cargo.toml、Cargo.lock、[profile.release]），unit-graph 的根是 <仓根>
  构建根定三样：Cargo.lock 与 [profile.release] 读哪一份、cargo build 在哪起、装置编到哪（<构建根>/<CARGO_TARGET_DIR 或 target>/release/<bin 名>）。
  入口的写法两种包一样，照 unit-graph：<包的 crate 名>::<模块…>::<函数名>，头一段是 crate 名（[lib] name，没写就是包名把 - 换成 _），
    不是 bin 名；bin 由 --binary 给、交 unit-graph 的 --target bin:<bin 名>。例 E205（多处读坏倒挂三条改法的代价）的装置在
    crates/singlefs-verification 的 bin e205_c971_inverted_first_txg_arm_costs 里：--package-directory crates/singlefs-verification
    --binary e205_c971_inverted_first_txg_arm_costs --entry singlefs_verification::fault_cell_verdict。头一段对不上包的 crate 名，取键之前退 2、点名该写什么。

键的组成（每个入口一把代码键；代码键 = unit-graph key 打的 key 那一个 hex，它把单元、工具链行、依赖行一起算进去）：
  单元     unit-graph closure 与 key：<unit-graph 的根> <入口> --target bin:<bin 名> [--cfg feature="<特性>"]… --threads <N>
  工具链行  scripts/gate/admission.py 的 KeyContext(<仓根>).rust_toolchain_lines(<构建根>) 的每一行（rustc -vV、cargo -V、
           构建环境的生效值）；另加「profile release」一行、构建根清单 [profile.release] 照 tomllib 解析后逐键摊开的 profile.release.<键>=<值>
           （admission.py 的 flattened_setting_lines，排好序）、「features none」或「features <特性,…>」一行、
           「host: <三元组>」一行（rustc -vV 的 host:，unit-graph 拿它定目标平台的 cfg；closure 与 key 两趟都给）
  依赖行   闭包里的 Rust 单元点到的外部包与它们的传递依赖（名、版本、来源、checksum）：每个单元的原文（unit-graph 交的「文件:起-止」那几行）
           经 scripts/gate/lib_function_units.py 的 rust_lex 与 token_references 取出点到的名字（裸名与路径的头一段），交给 admission.py 的
           KeyContext.dependency_lines 那一份收窄法（闭包里有单元的每个包，它清单写的外部依赖里代码名被点到的才收，再顺构建根的 Cargo.lock 收传递依赖）；
           单元落在哪个包：从单元那份文件的目录往上找最近的一份 Cargo.toml（到仓根为止，找不到的不算进哪个包），--package-directory 那个包总算一份
  不进键：入口名、bin 名、包目录与别的路径、行号、--threads、--host、日期（同一份源码放在两种包目录里，工具链、profile 与依赖都相同时代码键相同）。

判定库的转交（.claude/rules/verification.md「判定库只有一份」：装置不开本地库目录、不连服务，读写只经这一条路）：
  run 起装置前开两根管道，装置那一头的描述符号放进装置的环境 DEVICE_LIBRARY_REQUEST_FD（装置写请求）、DEVICE_LIBRARY_REPLY_FD（装置读回应），
  这里一个线程照请求经 scripts/prune-server/prune_server_client.py 读写服务；协议（一问一答）照装置一侧 crates/singlefs-device-library/src/lib.rs 文件头：
    read <相对键>                  → found <字节数> 加内容，或 absent（不在，或是撤回标记）
    write <相对键> <字节数> 加内容  → written（服务确认落盘之后才回）；先记进这一趟写过的键再写服务（没确认的写也可能落了盘，撤的时候一并撤）
    读写服务没成（PruneServerFailure）→ failed <退出码> <一句>，此后每个请求都照它回、不再连服务；装置照那个码退（75 是 prune-server 不可用）
    认不出的请求、相对键不只是字母数字与 . _ - / → failed 3
  键空间：族 apparatus-items（prune_server_client.py 文件头「键空间」），种类是 --binary 给的 bin 名（例 e174-lease-drop-wait-memory-return、
    e178-leaf-and-record-death-counts），相对键由装置给（E174（借据丢掉时等待与显存回还）是 <轮键>.out、codes/<代码键>.txt、index.tsv；E178（叶容器与记录死亡的计数）是 items/<项键>.out、codes/<代码键>.txt、index.tsv），
    命名空间按 <仓根> 定（落在临时目录底下的另占一份，自证一格一份；主仓不带：第二台在同一棵共享的树里、同一个绝对路径上跑，两台读写同一份）。
  撤回：服务不删键，撤一条就是把它改写成 WITHDRAWN_VALUE（撤回标记）；经转交读到标记回 absent，装置当不在、现判。撤的时候服务连不上，
    点名没撤成的键、照客户端的 75 退。
  复用哨兵（.claude/rules/verification.md「一个门禁：两种跑法」里「每一趟真跑开跑前……整道红」那一句；判法是 scripts/gate/lib_reuse_checks.py 的 ReuseSentinel，这里不另写一份）：
    项是相对键以 ITEM_SUFFIX（.out）收尾的那几条（E174 的 <轮键>.out、E178 的 items/<项键>.out）；codes/ 与 index.tsv 每趟照写，不是判定。
    命中 = 装置这一趟读一条项、库里有、不是撤回标记（键是装置照这一趟的代码键与参数算的，读到就是键相同、库里有判定）。
    种子取 SINGLEFS_REUSE_SENTINEL_SEED（不设现取），装置起来之前打进标准错误；强制现判是这一趟第 F 条命中（F 由种子在 1..FORCED_HIT_WINDOW 里抽，
      forced_hit_number）：那一条回 absent（装置现判、写回），记下库里那一份；命中不到 F 条的这一趟没有强制现判（报一句，不红）。
      回应要当场给，装置还没读完之前不知道一共命中几条，所以按到达的次序在头 FORCED_HIT_WINDOW 条命中里抽；装置几个线程并发读时到达次序不定，
      同一个种子抽中的可能是另一条（DEVICE_CODE_KEY_SENTINEL 那一行点名这一趟抽中的是哪一条）。
    哨兵：装置退出之后从别的命中里照种子抽 SINGLEFS_REUSE_SENTINEL_COUNT 条（默认 3；不足的报「哨兵不足」，不红）。
    装置退 0、转交没出错、编译与跑之间键没变时判：哨兵有一条这一趟又被写了（读到了还现判）⇒「复用失灵」；强制现判那一条写回的判定
      （去掉 meta 起头的行之后的 sha256，那几行是取键那天、跑法与机器）与库里那一份不同 ⇒「判定不稳」；强制现判那一条这一趟没写回 ⇒「强制现判没起作用」；
      有一样就整趟红、退 1（✗ 句带种子与出路）。末尾打一行 DEVICE_CODE_KEY_SENTINEL seed=<种子> forced_hit=<F> hits=<命中条数> forced=<相对键或 -> sentinels=<相对键,…或 ->。

组成清单（制表符分隔，一行一项；装置照原样存进库的 codes/<代码键>.txt，只给「键对不上时变了哪些单元」用，不进键）：
  form<TAB>device-code-key-v1
  judged<TAB><YYYY-MM-DD>        取键那天（东京时间），装置写进项文件的 meta 行
  host<TAB><本机|第二台>
  code_key<TAB><入口><TAB><hex>  每个入口一行，排在别的行前面（只在 form 之后）
  toolchain<TAB><行>
  dependency<TAB><入口><TAB><行>
  unit<TAB><入口><TAB><unit-graph 的单元行：种类、限定名、文件:起-止、指纹>
  refused<TAB><入口><TAB><unit-graph 的拒算行去掉开头的 refused 那一段>

复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：自证 reuse-three-questions 一格，两种包目录各一棵小树、各两个入口，四项一起问：
  假装置按代码键经转交读写临时服务里的库，读得回算复用、读不回现判并写回（转交抽中强制现判的那一条，假装置报现判，这里照 DEVICE_CODE_KEY_SENTINEL 那一行记成强制现判）；
  闭包内那一改两棵树各改一处只有第二个入口调到的函数，affected 是两棵树各自的第二个入口；打一行规范输出「REUSE_SELFTEST script=scripts/experiment-apparatus/device-code-key.py …」。复用哨兵在转交里（「判定库的转交」复用哨兵那一段），自证 reuse-sentinel 一格：
  同一棵两入口的小树跑两趟，第二趟种子取强制现判落在第 1 条命中的那一个：都照规矩绿；假装置读到了还写（哨兵被现判）、强制现判写回的不同、强制现判不写回三种各红。

自证只在临时 prune-server 上跑（prune_server_client.py 文件头「命令行」--with-temporary-service；开跑前调 selftest_service_problem()，不是临时服务退 2、一格都不跑）。

退出码：0 成；1 run 里编不过、两次键不等、起不了装置、复用哨兵红；2 用法错、包目录认不得、入口头一段不是包的 crate 名、入口找不到、自证没有临时服务；3 键算不出；78 unit-graph 找不到；
  75 撤回时 prune-server 不可用（照客户端的码）。run 里装置自己退非 0 时交装置的退出码（键算不出的那一趟也是；转交回了 failed 75 的装置退 75）。

弄坏开关 DEVICE_CODE_KEY_BREAK=<项>[,<项>…]：只在 --selftest 里认（命令行不是单独一个 --selftest、又不是自证起的子进程（自证给子进程设 DEVICE_CODE_KEY_SELFTEST_RUN=1）时设了它，
参数解析之前退 2、点名它只在自证里认）。逐项写开关名、弄坏哪一处，下一行起写打开后自证红的格（自证逐格打「✗ <格名>：…」，末尾一句「device-code-key 自证：N 格，红 M 格」）：
  DEVICE_CODE_KEY_BREAK=raw-source-digest  代码键改取闭包里那几份源文件整份原文的 sha256（不经 unit-graph 的去注释指纹）
      「只加一行注释键不变」「加一个没人调的函数键不变」「reuse-three-questions」
  DEVICE_CODE_KEY_BREAK=toolchain-lines-dropped  不给工具链行（只留 host: 那一行）
      「profile.release 改一个值键变」「crates 包：profile 读仓根清单，改一个值键变」
  DEVICE_CODE_KEY_BREAK=host-line-dropped  工具链行里不加 host: <三元组> 那一行（unit-graph 缺目标三元组退 2，取键的几格跟着红）
      「只加一行注释键不变」「改入口调到的函数键变」「加一个没人调的函数键不变」「profile.release 改一个值键变」「闭包点到的包在 Cargo.lock 里抬一版键变」「闭包没点到的包抬一版键不变」「拒算：退 3、装置照原参数起、不加别的开关、不给代码键」「前后两次键相等：项经转交写进临时服务、交装置的退出码」「装置退 4：这里也退 4」「编译与跑之间改了入口调到的函数：撤掉这一趟写进库的项、退 1，改回之后再跑经转交读成不在」「工具链行带一行 host: <三元组>，closure 与 key 两趟都给」「reuse-three-questions」「reuse-sentinel」「装置退出之后取键崩了：撤掉这一趟写进库的项、交装置的退出码」「装置退出之后取键拒算：撤掉这一趟写进库的项、装置退 0 时退 3」「prune-server 连不上：装置头一个读就收到 failed 75、这里退 75」「认不出的请求：回 failed 3、装置照它退 3」
      「crates 包：取键、在仓根 cargo build -p <包名>、起仓根 target 下的装置、项经转交写进临时服务」「同一份源码放两种包目录：代码键相同」
      「crates 包：闭包点到别的 crate 的单元，那个 crate 依赖的外部包抬一版键变」「crates 包：profile 读仓根清单，改一个值键变」
  DEVICE_CODE_KEY_BREAK=dependency-lines-dropped  不给依赖行
      「闭包点到的包在 Cargo.lock 里抬一版键变」「crates 包：闭包点到别的 crate 的单元，那个 crate 依赖的外部包抬一版键变」
  DEVICE_CODE_KEY_BREAK=dependency-lines-not-narrowed  依赖行收清单写的全部外部依赖（不按闭包点到的收窄）
      「闭包没点到的包抬一版键不变」
  DEVICE_CODE_KEY_BREAK=bracket-skipped  run 跑完装置不再取第二次键
      「编译与跑之间改了入口调到的函数：撤掉这一趟写进库的项、退 1，改回之后再跑经转交读成不在」「装置退出之后取键崩了：撤掉这一趟写进库的项、交装置的退出码」「装置退出之后取键拒算：撤掉这一趟写进库的项、装置退 0 时退 3」
  DEVICE_CODE_KEY_BREAK=after-key-crash-propagates  装置退出之后那一次取键 unit-graph 崩了，把异常照抛出去（main 退 2，这一趟写进库的项不撤、装置的退出码被盖）
      「装置退出之后取键崩了：撤掉这一趟写进库的项、交装置的退出码」
  DEVICE_CODE_KEY_BREAK=after-key-refusal-as-changed  装置退出之后那一次取键拒算，当成键变了（撤掉这一趟写进库的项、退 1，装置的退出码被盖）
      「装置退出之后取键拒算：撤掉这一趟写进库的项、装置退 0 时退 3」
  DEVICE_CODE_KEY_BREAK=device-binary-unset  起装置不放 DEVICE_CODE_KEY_DEVICE_BINARY
      「拒算：退 3、装置照原参数起、不加别的开关、不给代码键」「前后两次键相等：项经转交写进临时服务、交装置的退出码」
      「crates 包：取键、在仓根 cargo build -p <包名>、起仓根 target 下的装置、项经转交写进临时服务」
  DEVICE_CODE_KEY_BREAK=withdrawal-skipped  该撤的时候一条都不撤（只报那一句、照退）
      「编译与跑之间改了入口调到的函数：撤掉这一趟写进库的项、退 1，改回之后再跑经转交读成不在」「装置退出之后取键崩了：撤掉这一趟写进库的项、交装置的退出码」「装置退出之后取键拒算：撤掉这一趟写进库的项、装置退 0 时退 3」
  DEVICE_CODE_KEY_BREAK=withdrawn-read-as-present  经转交读到撤回标记照 found 交给装置
      「编译与跑之间改了入口调到的函数：撤掉这一趟写进库的项、退 1，改回之后再跑经转交读成不在」
  DEVICE_CODE_KEY_BREAK=relay-failure-reported-as-absent  读服务没成时回 absent（装置当不在、现判）
      「prune-server 连不上：装置头一个读就收到 failed 75、这里退 75」
  DEVICE_CODE_KEY_BREAK=unknown-request-answered-absent  认不出的请求回 absent
      「认不出的请求：回 failed 3、装置照它退 3」
  DEVICE_CODE_KEY_BREAK=sentinel-skipped  转交不抽强制现判、装置退出之后不判哨兵
      「reuse-sentinel」
  DEVICE_CODE_KEY_BREAK=forced-read-answered-found  抽中强制现判的那一条照 found 交给装置（强制现判没起作用的样子）
      「reuse-sentinel」
  DEVICE_CODE_KEY_BREAK=unit-graph-root-fixed  unit-graph 的根一律取 <仓根>/scripts（crates 包的入口在那里找不到 crate，unit-graph 退 2）
      「crates 包：取键、在仓根 cargo build -p <包名>、起仓根 target 下的装置、项经转交写进临时服务」「同一份源码放两种包目录：代码键相同」
      「crates 包：闭包点到别的 crate 的单元，那个 crate 依赖的外部包抬一版键变」「crates 包：profile 读仓根清单，改一个值键变」「reuse-three-questions」
  DEVICE_CODE_KEY_BREAK=build-in-apparatus-directory  cargo build 在 <仓根>/scripts/experiment-apparatus 里起、不带 -p <包名>
      「前后两次键相等：项经转交写进临时服务、交装置的退出码」「crates 包：取键、在仓根 cargo build -p <包名>、起仓根 target 下的装置、项经转交写进临时服务」
  DEVICE_CODE_KEY_BREAK=device-binary-under-package  装置在包目录的 target 下找（crates 包编在仓根的 target 下，起不了装置退 1）
      「crates 包：取键、在仓根 cargo build -p <包名>、起仓根 target 下的装置、项经转交写进临时服务」「reuse-three-questions」
  DEVICE_CODE_KEY_BREAK=dependency-lines-one-crate  闭包里的单元全算进 --package-directory 那一个包（别的 crate 依赖的外部包不收）
      「crates 包：闭包点到别的 crate 的单元，那个 crate 依赖的外部包抬一版键变」
  DEVICE_CODE_KEY_BREAK=entry-crate-unchecked  不核入口头一段是不是包的 crate 名（原样交 unit-graph，它报 crate 找不到）
      「入口头一段写成 bin 名：退 2、点名要写的 crate 名」
"""
import datetime
import hashlib
import os
import random
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import tomllib
import types

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "gate"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "prune-server"))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
import admission  # noqa: E402
import lib_function_units  # noqa: E402
import lib_reuse_checks  # noqa: E402
import prune_server_client  # noqa: E402
from heavy_threads import default_heavy_threads  # noqa: E402

APPARATUS_PACKAGE_DIRECTORY = os.path.join("scripts", "experiment-apparatus")   # 不给 --package-directory 时的包目录（文件头「包目录」）
CRATES_DIRECTORY = "crates"   # 仓根工作区成员所在的目录：crates/<名> 是另一种包目录
SCRIPT_KEY = "scripts/experiment-apparatus/device-code-key.py"   # 复用三问规范输出认的脚本名
FORM_LINE = "form\tdevice-code-key-v1"
RUSTC_HOST_PREFIX = "rustc -vV: host: "
DEVICE_BINARY_VARIABLE = "DEVICE_CODE_KEY_DEVICE_BINARY"
UNIT_GRAPH_VARIABLE = "SINGLEFS_UNIT_GRAPH"
DEFAULT_UNIT_GRAPH = os.path.join("target", "release", "unit-graph")
BREAK_VARIABLE = "DEVICE_CODE_KEY_BREAK"
SELFTEST_RUN_VARIABLE = "DEVICE_CODE_KEY_SELFTEST_RUN"
BREAK_SWITCHES = ("raw-source-digest", "toolchain-lines-dropped", "host-line-dropped", "dependency-lines-dropped", "dependency-lines-not-narrowed", "bracket-skipped",
                  "device-binary-unset", "after-key-crash-propagates", "after-key-refusal-as-changed", "withdrawal-skipped", "withdrawn-read-as-present",
                  "relay-failure-reported-as-absent", "unknown-request-answered-absent", "sentinel-skipped", "forced-read-answered-found",
                  "unit-graph-root-fixed", "build-in-apparatus-directory", "device-binary-under-package", "dependency-lines-one-crate", "entry-crate-unchecked")
HOST_LABELS = ("本机", "第二台")
REFUSAL_SHOWN = 3
TOKYO = datetime.timezone(datetime.timedelta(hours=9))
EXIT_FAILED, EXIT_USAGE, EXIT_REFUSED, EXIT_UNIT_GRAPH_MISSING = 1, 2, 3, 78
# 判定库的转交（文件头「判定库的转交」）
LIBRARY_FAMILY = prune_server_client.FAMILY_APPARATUS_ITEMS
REQUEST_DESCRIPTOR_VARIABLE = "DEVICE_LIBRARY_REQUEST_FD"
REPLY_DESCRIPTOR_VARIABLE = "DEVICE_LIBRARY_REPLY_FD"
WITHDRAWN_VALUE = b"withdrawn device-code-key: source changed between build and run, or the key could not be taken again\n"
RELATIVE_KEY_PATTERN = re.compile(r"[A-Za-z0-9._/-]+")
RELAY_DRAIN_SECONDS = 30      # 装置退出之后等转交线程读到请求管道尽头的上限（装置起的子进程还握着管道时不一直等）
WITHDRAWN_KEYS_SHOWN = 5
SELFTEST_RUN_SECONDS = 120    # 自证里起这份脚本自己一次的上限（转交卡住时那一格照超时记红，不一直等）
# 复用哨兵（文件头「判定库的转交」复用哨兵那一段）
ITEM_SUFFIX = ".out"
FORCED_HIT_WINDOW = 8
VERDICT_IGNORED_LINE_PREFIX = b"meta "
SENTINEL_LINE = "DEVICE_CODE_KEY_SENTINEL"


class UsageError(Exception):
    """命令行写错、入口找不到：退 2。"""


class UnitGraphMissing(Exception):
    """unit-graph 程序找不到：退 78。"""


def break_is_set(switch):
    if os.environ.get(SELFTEST_RUN_VARIABLE) != "1":
        return False
    return switch in os.environ.get(BREAK_VARIABLE, "").split(",")


def today_in_tokyo():
    return datetime.datetime.now(TOKYO).date().isoformat()


def find_unit_graph(root, given):
    """--unit-graph、SINGLEFS_UNIT_GRAPH、<仓根>/target/release/unit-graph，取第一个给了的；给了而不是可执行文件也算找不到。"""
    candidate = given or os.environ.get(UNIT_GRAPH_VARIABLE) or os.path.join(root, DEFAULT_UNIT_GRAPH)
    if os.path.isfile(candidate) and os.access(candidate, os.X_OK):
        return candidate
    raise UnitGraphMissing(f"unit-graph 找不到：{candidate}（--unit-graph、{UNIT_GRAPH_VARIABLE}、<仓根>/{DEFAULT_UNIT_GRAPH} 依次找）")


def read_manifest(directory):
    """directory 下的 Cargo.toml 解析出的表；读不了、解不开交 None。"""
    try:
        with open(os.path.join(directory, "Cargo.toml"), "rb") as handle:
            return tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError):
        return None


def package_layout(root, directory):
    """--package-directory 给的包目录 → 它的布局（文件头「包目录」）：directory、build_root、unit_graph_root 都相对仓根；
    package 是清单的包名，crate 是入口头一段要写的 crate 名。认不得、清单读不出抛 UsageError。"""
    normalized = os.path.normpath(directory)
    parts = normalized.split(os.sep)
    if normalized == APPARATUS_PACKAGE_DIRECTORY:
        build_root, unit_graph_root = normalized, os.path.dirname(normalized)
    elif len(parts) == 2 and parts[0] == CRATES_DIRECTORY and parts[1] not in ("", ".", ".."):
        build_root, unit_graph_root = ".", "."
    else:
        raise UsageError(f"--package-directory 只认 {APPARATUS_PACKAGE_DIRECTORY} 与 {CRATES_DIRECTORY}/<名>（相对仓根），给的是 {directory}")
    if break_is_set("unit-graph-root-fixed"):
        unit_graph_root = os.path.dirname(APPARATUS_PACKAGE_DIRECTORY)
    manifest = read_manifest(os.path.join(root, normalized))
    package = ((manifest or {}).get("package") or {}).get("name") or ""
    if not package:
        raise UsageError(f"包目录 {normalized} 的 Cargo.toml 读不出或没写 [package] name：{os.path.join(root, normalized, 'Cargo.toml')}")
    crate = ((manifest.get("lib") or {}).get("name")) or package.replace("-", "_")
    return types.SimpleNamespace(directory=normalized, build_root=build_root, unit_graph_root=unit_graph_root, package=package, crate=crate)


def toolchain_lines(root, layout, features):
    """工具链行（文件头「键的组成」）。工具链起不来、rustc -vV 交的 host: 不是恰好一行，抛 admission.UnitKeyError（调用方按键算不出办）。"""
    build_root = os.path.normpath(os.path.join(root, layout.build_root))
    lines = list(admission.KeyContext(root).rust_toolchain_lines(build_root))
    hosts = [line[len(RUSTC_HOST_PREFIX):] for line in lines if line.startswith(RUSTC_HOST_PREFIX)]
    if len(hosts) != 1:
        raise admission.UnitKeyError(f"rustc -vV 交了 {len(hosts)} 行 host:（要恰好一行：unit-graph 拿它定目标平台的 cfg）")
    triple = [] if break_is_set("host-line-dropped") else [f"host: {hosts[0]}"]
    if break_is_set("toolchain-lines-dropped"):
        return triple
    manifest = read_manifest(build_root)
    if manifest is None:
        raise admission.UnitKeyError(f"构建根的 Cargo.toml 读不出：{os.path.join(build_root, 'Cargo.toml')}")
    flattened = []
    admission.flattened_setting_lines("profile.release", (manifest.get("profile") or {}).get("release") or {}, flattened)
    lines += ["profile release"] + sorted(flattened)
    lines.append(f"features {','.join(sorted(features)) if features else 'none'}")
    return lines + triple


def unit_line_source_text(root, unit_line, file_cache):
    """单元行「种类 限定名 文件:起-止 指纹」那几行原文；数据单元、WGSL 单元、读不到的交 None。"""
    columns = unit_line.split("\t")
    if len(columns) != 4 or columns[0] == "data" or columns[0].startswith("wgsl"):
        return None
    path, _colon, span = columns[2].rpartition(":")
    start_text, _dash, end_text = span.partition("-")
    if not path.endswith(".rs") or not start_text.isdigit() or not end_text.isdigit():
        return None
    if path not in file_cache:
        try:
            with open(os.path.join(root, path), encoding="utf-8") as handle:
                file_cache[path] = handle.read().split("\n")
        except OSError:
            file_cache[path] = None
    lines = file_cache[path]
    if lines is None:
        return None
    return "\n".join(lines[int(start_text) - 1:int(end_text)])


def referenced_names(root, unit_lines):
    """闭包里的 Rust 单元点到的名字：裸名与「限定::名」的头一段（R-裸名与 R-路径读的那几样，lib_function_units.py 的 token_references）。"""
    names, file_cache = set(), {}
    for unit_line in unit_lines:
        text = unit_line_source_text(root, unit_line, file_cache)
        if text is None:
            continue
        kinds, texts, _lines, _documents = lib_function_units.rust_lex(text)
        plain, _methods, qualified, _dyn_traits = lib_function_units.token_references(kinds, texts, 0, len(texts))
        names |= set(plain) | {pair[0] for pair in qualified}
    return names


def nearest_package_directory(root, directory):
    """directory（相对仓根）往上最近一份带 Cargo.toml 的目录（相对仓根）；到仓根还没有交 None（仓根那一份是工作区清单，不算哪个包）。"""
    while directory not in ("", ".", os.sep):
        if os.path.isfile(os.path.join(root, directory, "Cargo.toml")):
            return directory
        directory = os.path.dirname(directory)
    return None


def closure_crates(root, layout, unit_lines):
    """闭包里有 Rust 单元的包（文件头「键的组成」依赖行那一条）：{相对构建根的包目录: 包（directory、package）}；--package-directory 那个包总算一份。
    弄坏开关 dependency-lines-one-crate 下只算 --package-directory 那一个包。"""
    directories = {layout.directory}
    scanned = () if break_is_set("dependency-lines-one-crate") else unit_lines
    for unit_line in scanned:
        columns = unit_line.split("\t")
        if len(columns) != 4 or columns[0] == "data" or columns[0].startswith("wgsl"):
            continue
        directory = nearest_package_directory(root, os.path.dirname(columns[2].rpartition(":")[0]))
        if directory is not None:
            directories.add(directory)
    build_root = os.path.normpath(os.path.join(root, layout.build_root))
    crates = {}
    for directory in sorted(directories):
        relative = os.path.relpath(os.path.join(root, directory), build_root)
        package = ((read_manifest(os.path.join(root, directory)) or {}).get("package") or {}).get("name") or ""
        crates[relative] = types.SimpleNamespace(directory=relative, package=package)
    return crates


def dependency_lines(root, layout, unit_lines):
    """闭包点到的外部包与它们的传递依赖：照 admission.py 的 KeyContext.dependency_lines 收（它要的闭包与工作区只用到下面这几个属性，这里给一份只装这几样的：
    闭包里有单元的每个包一个单元，点到的名字是整个闭包的，同 KeyContext.dependency_lines 自己的收法）。"""
    if break_is_set("dependency-lines-dropped"):
        return []
    build_root = os.path.normpath(os.path.join(root, layout.build_root))
    context = admission.KeyContext(root)
    names = referenced_names(root, unit_lines)
    crates = closure_crates(root, layout, unit_lines)
    if break_is_set("dependency-lines-not-narrowed"):
        for directory in crates:
            names |= set(admission.manifest_dependency_names(build_root, directory, False, context.host_triple(build_root)))
    workspace = types.SimpleNamespace(crate_of_path=crates.get, targets={})
    units = [types.SimpleNamespace(language="rust", data=False, file=directory, plain=names, qualified=()) for directory in crates]
    closure = types.SimpleNamespace(order=units, cfg_lines=[])
    return context.dependency_lines(build_root, workspace, closure, None)


def unit_graph_arguments(root, layout, command, entry, binary, features, threads, toolchain=(), dependencies=()):
    arguments = [command, os.path.normpath(os.path.join(root, layout.unit_graph_root)), entry, "--target", f"bin:{binary}"]
    for feature in sorted(features):
        arguments += ["--cfg", f'feature="{feature}"']
    arguments += ["--threads", str(threads)]
    for line in toolchain:
        arguments += ["--toolchain-line", line]
    for line in dependencies:
        arguments += ["--lock-line", line]
    return arguments


def call_unit_graph(unit_graph, arguments):
    """→ (退出码, 标准输出的行)。退 2（用法错、入口找不到）抛 UsageError。"""
    completed = subprocess.run([unit_graph] + arguments, capture_output=True, text=True)
    if completed.returncode == EXIT_USAGE:
        raise UsageError(f"unit-graph {' '.join(arguments[:3])} 退 2：{completed.stderr.strip()}")
    if completed.returncode not in (0, EXIT_REFUSED):
        raise UsageError(f"unit-graph {' '.join(arguments[:3])} 退 {completed.returncode}：{completed.stderr.strip()}")
    return completed.returncode, [line for line in completed.stdout.split("\n") if line]


def raw_source_key(root, unit_lines, toolchain, dependencies):
    """弄坏开关 raw-source-digest：闭包里那几份源文件整份原文（带注释）的 sha256，再接工具链行与依赖行。"""
    digest = hashlib.sha256()
    for path in sorted({line.split("\t")[2].rpartition(":")[0] for line in unit_lines if len(line.split("\t")) == 4}):
        with open(os.path.join(root, path), "rb") as handle:
            digest.update(handle.read())
    digest.update("\n".join(list(toolchain) + list(dependencies)).encode())
    return digest.hexdigest()


def compute_code_keys(root, options):
    """每个入口一把代码键。→ (组成清单的行, {入口: 代码键或 None}, 拒算行)；键算不出的入口交 None，拒算行写明是哪一处。"""
    unit_graph = find_unit_graph(root, options.unit_graph)
    head = [FORM_LINE, f"judged\t{today_in_tokyo()}", f"host\t{options.host}"]
    code_keys, refusals, body = {}, [], []
    layout = options.layout
    try:
        toolchain = toolchain_lines(root, layout, options.features)
    except admission.UnitKeyError as error:
        refusals = [f"refused\t{entry}\ttoolchain-unavailable\t-\t{error}" for entry in options.entries]
        return head + refusals, {entry: None for entry in options.entries}, refusals
    body += [f"toolchain\t{line}" for line in toolchain]
    for entry in options.entries:
        code, closure = call_unit_graph(unit_graph, unit_graph_arguments(root, layout, "closure", entry, options.binary, options.features, options.threads,
                                                                         toolchain))
        unit_lines = [line for line in closure if not line.startswith(("key\t", "refused\t"))]
        entry_refusals = [f"refused\t{entry}\t{line.partition(chr(9))[2]}" for line in closure if line.startswith("refused\t")]
        body += [f"unit\t{entry}\t{line}" for line in unit_lines]
        if code == EXIT_REFUSED or entry_refusals:
            refusals += entry_refusals or [f"refused\t{entry}\tunit-graph-exit-3\t-\t-"]
            code_keys[entry] = None
            continue
        dependencies = dependency_lines(root, layout, unit_lines)
        body += [f"dependency\t{entry}\t{line}" for line in dependencies]
        if break_is_set("raw-source-digest"):
            code_keys[entry] = raw_source_key(root, unit_lines, toolchain, dependencies)
            continue
        code, keyed = call_unit_graph(unit_graph, unit_graph_arguments(root, layout, "key", entry, options.binary, options.features, options.threads,
                                                                       toolchain, dependencies))
        key_lines = [line.partition("\t")[2] for line in keyed if line.startswith("key\t")]
        if code == EXIT_REFUSED or len(key_lines) != 1:
            refusals += [f"refused\t{entry}\t{line.partition(chr(9))[2]}" for line in keyed if line.startswith("refused\t")] or \
                [f"refused\t{entry}\tunit-graph-key-line\t-\t交了 {len(key_lines)} 行 key"]
            code_keys[entry] = None
            continue
        code_keys[entry] = key_lines[0]
    key_rows = [f"code_key\t{entry}\t{code_keys[entry]}" for entry in options.entries if code_keys[entry] is not None]
    return head + key_rows + body + refusals, code_keys, refusals


def refusal_summary(refusals):
    shown = "；".join(" ".join(line.split("\t")[2:4]) for line in refusals[:REFUSAL_SHOWN])
    more = f"；另 {len(refusals) - REFUSAL_SHOWN} 处" if len(refusals) > REFUSAL_SHOWN else ""
    return f"拒算 {len(refusals)} 处：{shown}{more}"


def run_key(root, options):
    lines, _code_keys, refusals = compute_code_keys(root, options)
    sys.stdout.write("".join(f"{line}\n" for line in lines))
    if refusals:
        print(f"device-code-key：键算不出（{refusal_summary(refusals)}）", file=sys.stderr)
        return EXIT_REFUSED
    return 0


def device_binary_path(root, options):
    """编好的装置：<构建根>/<CARGO_TARGET_DIR 或 target>/release/<bin 名>（文件头「包目录」）；弄坏开关 device-binary-under-package 下在包目录下找。"""
    base = options.layout.directory if break_is_set("device-binary-under-package") else options.layout.build_root
    target_directory = os.environ.get("CARGO_TARGET_DIR") or "target"
    return os.path.normpath(os.path.join(root, base, target_directory, "release", options.binary))


def device_environment(root, options):
    """起装置的环境：另放 DEVICE_CODE_KEY_DEVICE_BINARY=<编好、要起的那一份装置的路径>，装置起自己的子进程照它起（E174 的 run），不读自己的路径。
    弄坏开关 device-binary-unset 下不放。"""
    environment = dict(os.environ)
    if not break_is_set("device-binary-unset"):
        environment[DEVICE_BINARY_VARIABLE] = device_binary_path(root, options)
    return environment


def build_device(root, options):
    """在构建根里 cargo build --release -p <包名> --bin <bin 名>（文件头「包目录」）；弄坏开关 build-in-apparatus-directory 下在
    scripts/experiment-apparatus 里起、不带 -p。"""
    if break_is_set("build-in-apparatus-directory"):
        command, directory = ["cargo", "build", "--release", "--bin", options.binary], APPARATUS_PACKAGE_DIRECTORY
    else:
        command, directory = ["cargo", "build", "--release", "-p", options.layout.package, "--bin", options.binary], options.layout.build_root
    if options.features:
        command += ["--features", ",".join(sorted(options.features))]
    completed = subprocess.run(command, cwd=os.path.normpath(os.path.join(root, directory)))
    if completed.returncode != 0:
        print(f"device-code-key：{' '.join(command)} 退 {completed.returncode}，没起装置", file=sys.stderr)
    return completed.returncode == 0


def forced_hit_number(seed):
    """强制现判是这一趟第几条命中（1..FORCED_HIT_WINDOW，照种子抽；文件头「判定库的转交」复用哨兵那一段）。"""
    return 1 + random.Random(seed).randrange(FORCED_HIT_WINDOW)


def verdict_digest(data):
    """一条项的判定：去掉 meta 起头的行（取键那天、跑法与机器）之后的 sha256。"""
    kept = b"".join(line for line in data.splitlines(keepends=True) if not line.startswith(VERDICT_IGNORED_LINE_PREFIX))
    return hashlib.sha256(kept).hexdigest()


class LibraryRelay:
    """判定库的转交（文件头「判定库的转交」）：起装置时开两根管道，一个线程照装置的请求经 prune_server_client 读写 store。
    written 是这一趟写过的相对键（去重、保序，撤回时照它撤）；failure 是读写服务头一次没成时的（退出码, 一句）。
    复用哨兵（文件头复用哨兵那一段）：hits 是命中的项（保序去重），rewritten 是命中之后这一趟又写了的项，forced 是抽中强制现判的那一条
    （它库里那一份的判定 forced_stored、这一趟写回的 forced_written）。"""

    def __init__(self, store, label):
        self.store = store
        self.written = []
        self.failure = None
        self.lock = threading.Lock()
        self.label = label
        self.seed = lib_reuse_checks.sentinel_seed()
        self.forced_hit = forced_hit_number(self.seed)
        self.hits, self.rewritten = [], set()
        self.forced, self.forced_stored, self.forced_written = None, None, None

    def sentinel_announcement(self):
        if break_is_set("sentinel-skipped"):
            return f"device-code-key：复用哨兵（{self.label}）：弄坏开关 sentinel-skipped，这一趟不抽"
        return (f"device-code-key：复用哨兵（{self.label}）：种子 {self.seed}（{lib_reuse_checks.SENTINEL_SEED_VARIABLE}={self.seed} 复现）；"
                f"强制现判抽这一趟第 {self.forced_hit} 条命中的项（相对键以 {ITEM_SUFFIX} 收尾、库里有、不是撤回标记）")

    def note_hit(self, key, data):
        """命中一条项：抽中的那一条交 True（回 absent、记下库里那一份），别的记进 hits 交 False。"""
        if break_is_set("sentinel-skipped") or not key.endswith(ITEM_SUFFIX):
            return False
        if key not in self.hits:
            self.hits.append(key)
        if self.forced is None and len(self.hits) == self.forced_hit:
            self.forced, self.forced_stored = key, verdict_digest(data)
            return not break_is_set("forced-read-answered-found")
        return False

    def note_write(self, key, data):
        if key == self.forced:
            self.forced_written = verdict_digest(data)
        elif key in self.hits:
            self.rewritten.add(key)

    def sentinel_problems(self):
        """装置退出之后判哨兵（文件头复用哨兵那一段）：交 [(红句, 出路)]，判定照打到标准错误，末尾打 DEVICE_CODE_KEY_SENTINEL 那一行。"""
        if break_is_set("sentinel-skipped"):
            return []
        printer = lambda line: print(line, file=sys.stderr)   # noqa: E731
        sentinel = lib_reuse_checks.ReuseSentinel(SCRIPT_KEY, self.label, seed=self.seed)
        others = {key: "库里有" for key in self.hits if key != self.forced}
        sentinel.plan(others, printer=lambda _line: None)   # 只借它照种子抽哨兵；强制现判是转交当场抽的那一条，下面换上
        sentinel.forced_item = self.forced
        if self.forced is not None:
            sentinel.candidates[self.forced] = self.forced_stored
            written = self.forced_written is not None
            sentinel.observe(self.forced, lib_reuse_checks.FORCED if written else lib_reuse_checks.REUSED, self.forced_written or "-")
        for key in sentinel.sentinels:
            sentinel.observe(key, lib_reuse_checks.JUDGED if key in self.rewritten else lib_reuse_checks.REUSED, "-")
        if self.forced is None:
            printer(f"  · {sentinel.title()}：这一趟命中 {len(self.hits)} 条、不到第 {self.forced_hit} 条，没抽到强制现判（种子 {self.seed}）")
        if len(sentinel.sentinels) < sentinel.count:
            printer(f"  · {sentinel.title()}：哨兵不足 {sentinel.count} 条（强制现判之外的命中只有 {len(others)} 条），这一趟不因此判红")
        problems = sentinel.finish(printer=printer)
        printer(f"{SENTINEL_LINE} seed={self.seed} forced_hit={self.forced_hit} hits={len(self.hits)} forced={self.forced or '-'} "
                f"sentinels={','.join(sentinel.sentinels) or '-'}")
        return problems

    def run_device(self, command, environment):
        """起装置并转交到它退出；→ 装置的退出码。起不了抛 OSError（管道都关掉）。"""
        request_read, request_write = os.pipe()
        reply_read, reply_write = os.pipe()
        environment = dict(environment)
        environment[REQUEST_DESCRIPTOR_VARIABLE] = str(request_write)
        environment[REPLY_DESCRIPTOR_VARIABLE] = str(reply_read)
        try:
            process = subprocess.Popen(command, env=environment, pass_fds=(request_write, reply_read))
        except OSError:
            for descriptor in (request_read, request_write, reply_read, reply_write):
                os.close(descriptor)
            raise
        os.close(request_write)
        os.close(reply_read)
        relay = threading.Thread(target=self.serve, args=(request_read, reply_write), daemon=True)
        relay.start()
        code = process.wait()
        relay.join(RELAY_DRAIN_SECONDS)
        if relay.is_alive():
            print(f"device-code-key：装置退出之后 {RELAY_DRAIN_SECONDS} 秒请求管道还有人握着（装置起的子进程没退），转交线程不再等；"
                  f"这一趟写过的 {len(self.written)} 条照记下的算", file=sys.stderr)
        return code

    def serve(self, request_descriptor, reply_descriptor):
        """读请求、回应答，到请求管道的尽头（装置退了）或装置不再读回应为止。"""
        with os.fdopen(request_descriptor, "rb") as requests, os.fdopen(reply_descriptor, "wb") as replies:
            try:
                while True:
                    line = requests.readline()
                    if not line:
                        return
                    answer = self.answer(line, requests)
                    if answer is None:
                        return
                    replies.write(answer)
                    replies.flush()
            except BrokenPipeError:
                return

    def through_service(self, action):
        """→ (成没成, 值)。读写服务没成记下 failure，此后不再连服务。"""
        if self.failure is not None:
            return False, None
        try:
            return True, action()
        except prune_server_client.PruneServerFailure as failure:
            self.failure = (failure.code, " ".join(str(failure.message).split()))
            return False, None

    def failed(self):
        code, sentence = self.failure
        return f"failed {code} {sentence}\n".encode("utf-8", "surrogateescape")

    def answer(self, line, requests):
        """一个请求的应答（bytes）；请求读不全（装置中途退了）交 None。"""
        if not line.endswith(b"\n"):
            return None
        words = line[:-1].decode("utf-8", "surrogateescape").split(" ")
        if len(words) == 2 and words[0] == "read" and RELATIVE_KEY_PATTERN.fullmatch(words[1]):
            done, data = self.through_service(lambda: self.store.read(words[1]))
            if not done:
                return b"absent\n" if break_is_set("relay-failure-reported-as-absent") else self.failed()
            if data is None or (data == WITHDRAWN_VALUE and not break_is_set("withdrawn-read-as-present")):
                return b"absent\n"
            if self.note_hit(words[1], data):
                return b"absent\n"   # 复用哨兵抽中的强制现判：装置当不在、现判写回，写回的与库里那一份在装置退出之后比
            return f"found {len(data)}\n".encode() + data
        if len(words) == 3 and words[0] == "write" and RELATIVE_KEY_PATTERN.fullmatch(words[1]) and words[2].isdigit():
            data = requests.read(int(words[2]))
            if len(data) != int(words[2]):
                return None
            with self.lock:
                if words[1] not in self.written:
                    self.written.append(words[1])
            self.note_write(words[1], data)
            done, _ = self.through_service(lambda: self.store.write(words[1], data))
            return b"written\n" if done else self.failed()
        if break_is_set("unknown-request-answered-absent"):
            return b"absent\n"
        shown = line[:-1].decode("utf-8", "replace")[:120]
        return f"failed {EXIT_REFUSED} 认不出的请求「{shown}」（只认 read <相对键>、write <相对键> <字节数>；相对键只用字母、数字与 . _ - /）\n".encode()


def withdraw_written(relay):
    """把这一趟经转交写进库的每一条改写成撤回标记（文件头「判定库的转交」）；→ 撤了几条。服务连不上点名没撤成的键、照客户端的码退。"""
    keys = list(relay.written)
    if not keys or break_is_set("withdrawal-skipped"):
        return 0
    try:
        relay.store.write_many([(key, WITHDRAWN_VALUE) for key in keys])
    except prune_server_client.PruneServerFailure:
        more = f" 等 {len(keys)} 条" if len(keys) > WITHDRAWN_KEYS_SHOWN else ""
        print(f"device-code-key：✗ 这一趟写进库的 {len(keys)} 条没撤成（{relay.store.location()} 下 {'、'.join(keys[:WITHDRAWN_KEYS_SHOWN])}{more}）："
              "它们是编译与跑之间源码可能变了的那一趟写的，服务回来之后把这几条交主 agent 撤", file=sys.stderr)
        raise
    return len(keys)


def run_device(root, options):
    composition_lines, code_keys, refusals = compute_code_keys(root, options)
    if refusals:
        print(f"device-code-key：键算不出（{refusal_summary(refusals)}），这一趟装置每项现判、不读不写库", file=sys.stderr)
        if not build_device(root, options):
            return EXIT_FAILED
        completed = subprocess.run([device_binary_path(root, options)] + options.device_arguments,
                                   env=device_environment(root, options))
        return EXIT_REFUSED if completed.returncode == 0 else completed.returncode
    work_directory = tempfile.mkdtemp(prefix="device-code-key-")
    try:
        composition_path = os.path.join(work_directory, "composition.txt")
        with open(composition_path, "w", encoding="utf-8") as handle:
            handle.write("".join(f"{line}\n" for line in composition_lines))
        if not build_device(root, options):
            return EXIT_FAILED
        relay = LibraryRelay(prune_server_client.LibraryStore(LIBRARY_FAMILY, options.binary, root), options.binary)
        print(relay.sentinel_announcement(), file=sys.stderr, flush=True)
        try:
            device_code = relay.run_device([device_binary_path(root, options)] + options.device_arguments + ["--code-key-file", composition_path],
                                           device_environment(root, options))
        except OSError as error:
            print(f"device-code-key：起不了装置 {device_binary_path(root, options)}：{error}", file=sys.stderr)
            return EXIT_FAILED
        if not break_is_set("bracket-skipped"):
            # 装置退出之后取不到键（unit-graph 崩了或找不到、拒算、工具链起不来）两种一样办：键核不了的不入库，撤这一趟写的，装置自己的退出码不盖（文件头 run 那一段）
            unobtainable = None
            try:
                _after_lines, after_keys, after_refusals = compute_code_keys(root, options)
                if after_refusals and not break_is_set("after-key-refusal-as-changed"):
                    unobtainable = refusal_summary(after_refusals)
            except (UsageError, UnitGraphMissing) as error:
                if break_is_set("after-key-crash-propagates"):   # 弄坏开关：异常照抛
                    raise
                unobtainable = f"取键没跑成：{error}"
            if unobtainable is not None:
                withdrawn = withdraw_written(relay)
                exit_code = device_code if device_code != 0 else EXIT_REFUSED
                print(f"device-code-key：装置退出之后键取不到（{unobtainable}），核不了编译与跑之间源码变没变：撤掉这一趟写进库的 {withdrawn} 条"
                      f"（键核不了的不入库，库里原有的不动）；装置退 {device_code}，这里退 {exit_code}", file=sys.stderr)
                return exit_code
            changed = [entry for entry in options.entries if after_keys.get(entry) != code_keys[entry]]
            if changed:
                withdrawn = withdraw_written(relay)
                detail = "；".join(f"{entry}：{code_keys[entry]} → {after_keys.get(entry)}" for entry in changed)
                print(f"device-code-key：✗ 源码在编译与跑之间变了（{detail}），撤掉这一趟写进库的 {withdrawn} 条", file=sys.stderr)
                return EXIT_FAILED
        print("device-code-key：代码键 " + "，".join(f"{entry} {code_keys[entry]}" for entry in options.entries) + "（编译之前与装置退出之后相同）",
              file=sys.stderr)
        if device_code != 0 or relay.failure is not None:
            print(f"device-code-key：装置退 {device_code}{'、转交出过错' if relay.failure is not None else ''}，复用哨兵这一趟不判", file=sys.stderr)
            return device_code
        problems = relay.sentinel_problems()
        for message, howto in problems:
            print(f"  ✗ {message}", file=sys.stderr)
            print(f"     → 怎么办：{howto}", file=sys.stderr)
        return EXIT_FAILED if problems else device_code
    finally:
        shutil.rmtree(work_directory, ignore_errors=True)


def parse_options(command, arguments):
    """key / run 的参数 → (仓根, 选项)。写错抛 UsageError。"""
    options = types.SimpleNamespace(binary=None, entries=[], features=[], unit_graph=None, threads=None, host=HOST_LABELS[0],
                                    package_directory=APPARATUS_PACKAGE_DIRECTORY, device_arguments=[])
    if "--" in arguments:
        separator = arguments.index("--")
        arguments, options.device_arguments = arguments[:separator], arguments[separator + 1:]
        if command != "run":
            raise UsageError("只有 run 带 -- <装置参数…>")
    elif command == "run":
        raise UsageError("run 要写 -- <装置参数…>")
    if not arguments or arguments[0].startswith("--"):
        raise UsageError(f"{command} 要先给 <仓根>")
    root = os.path.abspath(arguments[0])
    valued = {"--binary": "binary", "--unit-graph": "unit_graph", "--threads": "threads", "--host": "host", "--package-directory": "package_directory"}
    index = 1
    while index < len(arguments):
        flag = arguments[index]
        if flag not in valued and flag not in ("--entry", "--feature"):
            raise UsageError(f"认不得的参数：{flag}")
        if index + 1 >= len(arguments):
            raise UsageError(f"{flag} 后面缺值")
        value = arguments[index + 1]
        if flag == "--entry":
            options.entries.append(value)
        elif flag == "--feature":
            options.features.append(value)
        else:
            setattr(options, valued[flag], value)
        index += 2
    if not options.binary or not options.entries:
        raise UsageError("--binary 与至少一个 --entry 都要给")
    if len(set(options.entries)) != len(options.entries):
        raise UsageError("同一个 --entry 给了两次")
    if options.host not in HOST_LABELS:
        raise UsageError(f"--host 只认 {' / '.join(HOST_LABELS)}")
    if options.threads is None:
        try:
            options.threads = default_heavy_threads(os.environ)
        except Exception as error:  # 登记文件读不出数：照用法错退 2，出路写修登记（经 task-server 交时命令里的 --threads 当场被拒，不教调用方传它）
            raise UsageError(f"登记的重活默认并行度读不出（{error}）：照这个原因修 .claude/heavy-thread-default；经 task-server/task submit 交时不写 --threads（服务当场拒）") from error
    elif not str(options.threads).isdigit() or int(options.threads) < 1:
        raise UsageError("--threads 要正整数")
    options.layout = package_layout(root, options.package_directory)
    for entry in options.entries:
        head = entry.partition("::")[0]
        # 同 unit-graph 认 crate 的那一条（Workspace::crate_named：包名，或 - 换成 _ 之后等于 crate 名）；bin 名不是 crate 名
        if not break_is_set("entry-crate-unchecked") and head != options.layout.package and head.replace("-", "_") != options.layout.crate:
            raise UsageError(f"入口 {entry} 的头一段要写包 {options.layout.package}（{options.layout.directory}）的 crate 名 {options.layout.crate}："
                             f"写成 {options.layout.crate}::<模块…>::<函数名>，bin 由 --binary 给（文件头「包目录」）")
    return root, options


USAGE = ("用法：device-code-key.py key <仓根> --binary <bin 名> [--package-directory <包目录>] --entry <fn 路径>… [--feature <特性>]… [--unit-graph <程序>] "
         "[--threads <N>] [--host 本机|第二台]\n"
         "      device-code-key.py run <仓根> <同上选项> -- <装置参数…>\n"
         "      device-code-key.py --selftest\n"
         f"  --package-directory 只认 {APPARATUS_PACKAGE_DIRECTORY}（不给就是它）与 {CRATES_DIRECTORY}/<名>；入口写 <包的 crate 名>::<模块…>::<函数名>\n"
         "  --threads 只给自证；交给统一任务服务的命令里不写，写了被拒")


def refuse_break_outside_selftest(arguments):
    """弄坏开关只在自证里认（文件头「弄坏开关」）：命令行不是单独一个 --selftest、又不是自证起的子进程（自证给子进程设了 DEVICE_CODE_KEY_SELFTEST_RUN=1）而 DEVICE_CODE_KEY_BREAK 非空
    （不管值是什么），退 2 并点名它只在自证里认；在参数解析之前判。"""
    if os.environ.get(BREAK_VARIABLE) and arguments != ["--selftest"] and os.environ.get(SELFTEST_RUN_VARIABLE) != "1":
        print(f"  ✗ {BREAK_VARIABLE} 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/experiment-apparatus/device-code-key.py --selftest", file=sys.stderr)
        return EXIT_USAGE
    return None


def main(arguments):
    refusal = refuse_break_outside_selftest(arguments)
    if refusal is not None:
        return refusal
    if arguments == ["--selftest"]:
        return selftest()
    if not arguments or arguments[0] not in ("key", "run"):
        print(USAGE, file=sys.stderr)
        return EXIT_USAGE
    try:
        root, options = parse_options(arguments[0], arguments[1:])
        return run_key(root, options) if arguments[0] == "key" else run_device(root, options)
    except UsageError as error:
        print(f"device-code-key：{error}\n{USAGE}", file=sys.stderr)
        return EXIT_USAGE
    except UnitGraphMissing as error:
        print(f"  ✗ device-code-key：{error}", file=sys.stderr)
        print("     → 怎么办：cargo build --release -p singlefs-unit-graph（在仓根），或用 --unit-graph / SINGLEFS_UNIT_GRAPH 指到编好的 unit-graph",
              file=sys.stderr)
        return EXIT_UNIT_GRAPH_MISSING


# ── 自证：假的 unit-graph、rustc、cargo 与装置，在临时目录里造一棵只有清单、Cargo.lock 与一份源码的小树，不编译 ──

SELFTEST_UNIT_GRAPH = r'''
import hashlib, os, re, sys
arguments = sys.argv[1:]
command, root, entry = arguments[0], arguments[1], arguments[2]
toolchain = [arguments[index + 1] for index, word in enumerate(arguments) if word == "--toolchain-line"]
locks = [arguments[index + 1] for index, word in enumerate(arguments) if word == "--lock-line"]
if sum(1 for line in toolchain if line.startswith("host: ")) != 1:
    print("unit-graph：缺目标三元组：--toolchain-line 里要有一行 host: <三元组>（rustc -vV）或 target: <三元组>（cargo --target）", file=sys.stderr)
    sys.exit(2)
spoiled = os.environ.get("FAKE_UNIT_GRAPH_MODE_FILE", "")
mode = open(spoiled, encoding="utf-8").read() if spoiled and os.path.isfile(spoiled) else ""
if mode == "crash":
    print("unit-graph：自证造的崩溃（装置跑完之后假装置写了 crash）", file=sys.stderr)
    sys.exit(101)
# 根照真的 unit-graph 认：底下有 crates/ 的是仓根（包在 crates/ 下），没有的这个目录本身装着各包、仓根是它的上一层；入口头一段按包名找包
crates_directory, repository = (os.path.join(root, "crates"), root) if os.path.isdir(os.path.join(root, "crates")) else (root, os.path.dirname(os.path.normpath(root)))
packages = {}
for directory in sorted(os.listdir(crates_directory)):
    manifest = os.path.join(crates_directory, directory, "Cargo.toml")
    found = re.search(r'^name = "([^"]+)"', open(manifest, encoding="utf-8").read(), re.M) if os.path.isfile(manifest) else None
    if found:
        packages[found.group(1).replace("-", "_")] = os.path.join(crates_directory, directory)
crate = entry.partition("::")[0].replace("-", "_")
if crate not in packages:
    print(f"unit-graph：crate「{crate}」找不到（入口的头一段是 crate 名）", file=sys.stderr)
    sys.exit(2)
binary = arguments[arguments.index("--target") + 1].partition(":")[2]
# 入口那个包的装置源码（device.rs 或 src/bin/<bin>.rs），加上别的包的 src/lib.rs（闭包可以点到别的 crate 的函数）
sources = [path for path in (os.path.join(packages[crate], "device.rs"), os.path.join(packages[crate], "src", "bin", binary + ".rs")) if os.path.isfile(path)]
sources += [os.path.join(package, "src", "lib.rs") for package in packages.values() if package != packages[crate] and os.path.isfile(os.path.join(package, "src", "lib.rs"))]
shown = lambda path: os.path.relpath(path, repository)   # noqa: E731
if os.environ.get("FAKE_UNIT_GRAPH_REFUSE") == "1" or mode == "refuse":
    print(f'refused\titem-level-include\t{shown(sources[0])}:1\tinclude ! ( "x.rs" ) ;')
    sys.exit(3)
functions = {}
for source in sources:
    lines, position = open(source, encoding="utf-8").read().split("\n"), 0
    while position < len(lines):
        match = re.match(r"fn (\w+)\(", lines[position])
        if not match:
            position += 1
            continue
        last = position
        while lines[last] != "}":
            last += 1
        body = [re.sub(r"//.*", "", line).strip() for line in lines[position:last + 1]]
        functions[match.group(1)] = (shown(source), position + 1, last + 1, "\n".join(line for line in body if line))
        position = last + 1
name = entry.rpartition("::")[2]
if name not in functions:
    print(f"入口 {entry} 找不到", file=sys.stderr)
    sys.exit(2)
closure, todo = set(), [name]
while todo:
    current = todo.pop()
    if current in closure:
        continue
    closure.add(current)
    todo += [other for other in functions if re.search(r"\b" + other + r"\b", functions[current][3])]
digest = lambda text: hashlib.sha256(text.encode()).hexdigest()   # noqa: E731
units = sorted(f"fn\t{crate}::{unit}\t{functions[unit][0]}:{functions[unit][1]}-{functions[unit][2]}\t{digest(functions[unit][3])}" for unit in closure)
key = digest("\n".join(sorted(unit.split("\t")[3] for unit in units) + sorted("toolchain " + line for line in toolchain) + sorted("lock " + line for line in locks)))
if command == "closure":
    print("\n".join(units))
print(f"key\t{key}")
'''

SELFTEST_RUSTC = r'''
import sys
print("rustc 0.0.0 (fake 2026-01-01)\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0" if sys.argv[1:] == ["-vV"] else "rustc 0.0.0 (fake 2026-01-01)")
'''

SELFTEST_CARGO = r'''
import os, sys
if sys.argv[1:] == ["-V"]:
    print("cargo 0.0.0 (fake 2026-01-01)")
    sys.exit(0)
with open(os.environ["FAKE_CARGO_LOG"], "a", encoding="utf-8") as handle:
    handle.write(os.getcwd() + "\t" + " ".join(sys.argv[1:]) + "\n")
sys.exit(int(os.environ.get("FAKE_CARGO_EXIT", "0")))
'''

SELFTEST_DEVICE = r'''
import os, sys
arguments = sys.argv[1:]
with open(os.environ["FAKE_DEVICE_LOG"], "a", encoding="utf-8") as handle:
    handle.write("\t".join(arguments) + "\n")
work = os.environ["FAKE_DEVICE_WORK"]   # 假装置的旁证目录（看到的环境、各入口现判还是复用、收到的回应），不是库
os.makedirs(work, exist_ok=True)
with open(os.path.join(work, "device-binary.txt"), "w", encoding="utf-8") as handle:
    handle.write(os.environ.get("DEVICE_CODE_KEY_DEVICE_BINARY", "没设"))
exit_code = int(os.environ.get("FAKE_DEVICE_EXIT", "0"))
if "--code-key-file" in arguments:
    composition = open(arguments[arguments.index("--code-key-file") + 1], encoding="utf-8").read()
    entries = [line.split("\t")[1:3] for line in composition.split("\n") if line.startswith("code_key\t")]
    requests = os.fdopen(int(os.environ["DEVICE_LIBRARY_REQUEST_FD"]), "wb")
    replies = os.fdopen(int(os.environ["DEVICE_LIBRARY_REPLY_FD"]), "rb")
    seen = open(os.path.join(work, "replies.txt"), "a", encoding="utf-8")
    written = open(os.path.join(work, "written-keys.txt"), "a", encoding="utf-8")
    def ask(request, body=b""):
        if request.startswith("write "):
            written.write(request.split(" ")[1] + "\n")
            written.flush()
        requests.write(request.encode() + b"\n" + body)
        requests.flush()
        line = replies.readline().decode().rstrip("\n")
        seen.write(line + "\n")
        seen.flush()
        if line.startswith("failed "):
            sys.exit(int(line.split(" ")[1]))
        return replies.read(int(line.split(" ")[1])) if line.startswith("found ") else None
    if os.environ.get("FAKE_DEVICE_UNKNOWN_REQUEST"):
        ask("erase items/x.out")
    outcomes = []
    # 复用哨兵那一格的三种坏装置：读到了还写（哨兵被现判）、现判写回的判定变了、读成不在却不写回（强制现判没起作用）
    rejudge_found, suffix, absent_unwritten = (os.environ.get(name, "") for name in ("FAKE_DEVICE_REJUDGE_FOUND", "FAKE_DEVICE_JUDGED_SUFFIX", "FAKE_DEVICE_ABSENT_UNWRITTEN"))
    for entry, code_key in entries:
        key = f"items/{code_key}.out"
        found = ask(f"read {key}") is not None
        if found and not rejudge_found:
            outcomes.append(f"{entry}\treused")
        elif not found and absent_unwritten:
            outcomes.append(f"{entry}\treused")
        else:
            data = f"judged {code_key}{suffix}\nmeta process={os.getpid()}\n".encode()   # meta 行每趟不同：转交比强制现判时要去掉它
            ask(f"write {key} {len(data)}", data)
            outcomes.append(f"{entry}\tjudged")
    data = composition.encode()
    ask(f"write codes/{entries[0][1]}.txt {len(data)}", data)
    with open(os.path.join(work, "outcomes.txt"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in outcomes))
    with open(os.path.join(work, "outcome-keys.txt"), "w", encoding="utf-8") as handle:
        handle.write("".join(f"{entry}\titems/{code_key}.out\n" for entry, code_key in entries))
spoil = os.environ.get("FAKE_DEVICE_SPOIL_UNIT_GRAPH")
if spoil:
    with open(os.environ["FAKE_UNIT_GRAPH_MODE_FILE"], "w", encoding="utf-8") as handle:
        handle.write(spoil)
edited = os.environ.get("FAKE_DEVICE_EDIT_SOURCE")
if edited:
    with open(edited, encoding="utf-8") as handle:
        text = handle.read()
    with open(edited, "w", encoding="utf-8") as handle:
        handle.write(text.replace("compute(1)", "compute(2)"))
sys.exit(exit_code)
'''


SELFTEST_SOURCE = """fn entry_function() {
    helper_function();
}

fn helper_function() {
    let value = fake_external::compute(1);
}
"""

SELFTEST_MANIFEST = """[package]
name = "fake-package"
version = "0.1.0"
edition = "2021"

[dependencies]
fake-external = "1.0"
unused-external = "2.0"

[profile.release]
opt-level = 3
overflow-checks = true
"""

SELFTEST_LOCK = """version = 4

[[package]]
name = "fake-external"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaa"

[[package]]
name = "fake-package"
version = "0.1.0"
dependencies = [
 "fake-external",
 "unused-external",
]

[[package]]
name = "unused-external"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbb"
"""

# crates 包那一种（文件头「包目录」）：仓根是工作区（profile 与 Cargo.lock 在仓根），crates/fake-crate 的 bin fake-device 是装置（源码与 scripts 那一份相同），
# crates/fake-helper 是它 path 依赖的库，自己依赖 helper-external
SELFTEST_ROOT_MANIFEST = """[workspace]
resolver = "2"
members = ["crates/fake-crate", "crates/fake-helper"]

[profile.release]
opt-level = 3
overflow-checks = true
"""

SELFTEST_CRATE_MANIFEST = """[package]
name = "fake-crate"
version = "0.1.0"
edition = "2021"

[dependencies]
fake-external = "1.0"
fake-helper = { path = "../fake-helper" }
unused-external = "2.0"
"""

SELFTEST_HELPER_MANIFEST = """[package]
name = "fake-helper"
version = "0.1.0"
edition = "2021"

[dependencies]
helper-external = "3.0"
"""

SELFTEST_HELPER_SOURCE = """fn helper_library_function() {
    let value = helper_external::compute(3);
}
"""

SELFTEST_ROOT_LOCK = """version = 4

[[package]]
name = "fake-crate"
version = "0.1.0"
dependencies = [
 "fake-external",
 "fake-helper",
 "unused-external",
]

[[package]]
name = "fake-external"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaa"

[[package]]
name = "fake-helper"
version = "0.1.0"
dependencies = [
 "helper-external",
]

[[package]]
name = "helper-external"
version = "3.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "cccc"

[[package]]
name = "unused-external"
version = "2.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "bbbb"
"""

SELFTEST_CRATE_DIRECTORY = os.path.join(CRATES_DIRECTORY, "fake-crate")
SELFTEST_SCRIPTS_SOURCE = os.path.join(APPARATUS_PACKAGE_DIRECTORY, "device.rs")             # 相对小树
SELFTEST_CRATE_SOURCE = os.path.join(SELFTEST_CRATE_DIRECTORY, "src", "bin", "fake-device.rs")


def selftest_tree(directory):
    """一棵小树，两种包目录各一个（文件头「包目录」）：scripts/experiment-apparatus 下的清单、Cargo.lock、装置源码与编好的假装置；
    仓根的工作区清单与 Cargo.lock、crates/fake-crate（装置源码同一份）与 crates/fake-helper，仓根 target 下编好的假装置；
    fake-bin 下假 rustc、cargo；tools 下假 unit-graph。→ 环境变量。
    环境照此刻的进程（连哪一处 prune-server 指在 --with-temporary-service 起的临时服务上，子进程沿用）。"""
    tree = os.path.join(directory, "tree")
    package = os.path.join(tree, APPARATUS_PACKAGE_DIRECTORY)
    files = {os.path.join(package, "Cargo.toml"): SELFTEST_MANIFEST, os.path.join(package, "Cargo.lock"): SELFTEST_LOCK,
             os.path.join(tree, SELFTEST_SCRIPTS_SOURCE): SELFTEST_SOURCE,
             os.path.join(tree, "Cargo.toml"): SELFTEST_ROOT_MANIFEST, os.path.join(tree, "Cargo.lock"): SELFTEST_ROOT_LOCK,
             os.path.join(tree, SELFTEST_CRATE_DIRECTORY, "Cargo.toml"): SELFTEST_CRATE_MANIFEST, os.path.join(tree, SELFTEST_CRATE_SOURCE): SELFTEST_SOURCE,
             os.path.join(tree, CRATES_DIRECTORY, "fake-helper", "Cargo.toml"): SELFTEST_HELPER_MANIFEST,
             os.path.join(tree, CRATES_DIRECTORY, "fake-helper", "src", "lib.rs"): SELFTEST_HELPER_SOURCE}
    for path, text in files.items():
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    programs = {os.path.join(directory, "fake-bin", "rustc"): SELFTEST_RUSTC, os.path.join(directory, "fake-bin", "cargo"): SELFTEST_CARGO,
                os.path.join(directory, "tools", "unit-graph"): SELFTEST_UNIT_GRAPH,
                os.path.join(package, "target", "release", "fake-device"): SELFTEST_DEVICE,
                os.path.join(tree, "target", "release", "fake-device"): SELFTEST_DEVICE}
    for path, text in programs.items():
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f"#!{sys.executable}\n{text}")
        os.chmod(path, 0o755)
    environment = {name: value for name, value in os.environ.items() if not name.startswith(("CARGO", "RUST", "FAKE_"))}
    environment.update({"PATH": os.path.join(directory, "fake-bin") + os.pathsep + os.environ.get("PATH", ""),
                        "CARGO_HOME": os.path.join(directory, "cargo-home"), UNIT_GRAPH_VARIABLE: os.path.join(directory, "tools", "unit-graph"),
                        "FAKE_CARGO_LOG": os.path.join(directory, "cargo.log"), "FAKE_DEVICE_LOG": os.path.join(directory, "device.log"),
                        "FAKE_DEVICE_WORK": os.path.join(directory, "device-work"), "FAKE_UNIT_GRAPH_MODE_FILE": os.path.join(directory, "unit-graph-mode.txt"),
                        SELFTEST_RUN_VARIABLE: "1"})
    return environment


def selftest_store(directory):
    """小树那一份库在临时服务里的样子（族 apparatus-items、种类 fake-device、命名空间按小树定，同 run 起的转交）：自证直接读它核转交写进去的东西。"""
    return prune_server_client.LibraryStore(LIBRARY_FAMILY, "fake-device", os.path.join(directory, "tree"))


def selftest_call(directory, environment, arguments):
    """在小树上起这份脚本自己。→ (退出码, 标准输出, 标准错误)；SELFTEST_RUN_SECONDS 秒没退照超时抛（那一格记红）。"""
    command = [sys.executable, os.path.realpath(__file__)] + arguments[:1] + [os.path.join(directory, "tree")] + arguments[1:]
    completed = subprocess.run(command, capture_output=True, text=True, env=environment, timeout=SELFTEST_RUN_SECONDS)
    return completed.returncode, completed.stdout, completed.stderr


SCRIPTS_ENTRY = "fake_package::entry_function"   # scripts/experiment-apparatus 那个包（fake-package）的入口
SCRIPTS_OTHER_ENTRY = "fake_package::other_entry"
CRATES_ENTRY = "fake_crate::entry_function"      # crates/fake-crate 的入口：头一段是 crate 名，bin 由 --binary 给
CRATES_OTHER_ENTRY = "fake_crate::other_entry"
KEY_ARGUMENTS = ["--binary", "fake-device", "--entry", SCRIPTS_ENTRY, "--threads", "1"]
CRATES_KEY_ARGUMENTS = ["--binary", "fake-device", "--package-directory", SELFTEST_CRATE_DIRECTORY, "--entry", CRATES_ENTRY, "--threads", "1"]


def edit_tree_file(directory, edit):
    """edit = (相对小树的文件, 原文, 替换文)，原文要恰好命中一次；→ 没改成时的说明，改成交 None。"""
    path = os.path.join(directory, "tree", edit[0])
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    if text.count(edit[1]) != 1:
        return f"{edit[0]} 里 {edit[1]!r} 命中 {text.count(edit[1])} 次"
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(edit[1], edit[2]))
    return None


def selftest_key(directory, environment, edit=None, arguments=KEY_ARGUMENTS):
    """先照 edit 改一处（edit_tree_file），再取一次代码键。→ (代码键或 None, 说明)。"""
    if edit is not None:
        problem = edit_tree_file(directory, edit)
        if problem is not None:
            return None, problem
    code, stdout, stderr = selftest_call(directory, environment, ["key"] + arguments)
    keys = [line.split("\t")[2] for line in stdout.split("\n") if line.startswith("code_key\t")]
    return (keys[0] if code == 0 and len(keys) == 1 else None), f"退 {code}，{stderr.strip()[:300]}"


def key_cell(edit, should_change, arguments=KEY_ARGUMENTS, prepare=None):
    """改一处之前、之后各取一次代码键：should_change 为真要不等，为假要相等。prepare 是两次取键之前先做的那一改（同 edit 的写法）。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        problem = edit_tree_file(directory, prepare) if prepare is not None else None
        if problem is not None:
            return False, problem
        before, before_detail = selftest_key(directory, environment, arguments=arguments)
        after, after_detail = selftest_key(directory, environment, edit, arguments)
        if before is None or after is None:
            return False, f"键没取出来：改前 {before_detail}；改后 {after_detail}"
        return (before != after) == should_change, f"改前 {before[:16]}，改后 {after[:16]}"


def same_source_two_packages_cell():
    """同一份装置源码放在两种包目录里（scripts/experiment-apparatus 与 crates/fake-crate，工具链、profile、依赖都一样）：两路取的代码键相同。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        scripts_key, scripts_detail = selftest_key(directory, environment)
        crates_key, crates_detail = selftest_key(directory, environment, arguments=CRATES_KEY_ARGUMENTS)
        if scripts_key is None or crates_key is None:
            return False, f"键没取出来：scripts 那一路 {scripts_detail}；crates 那一路 {crates_detail}"
        return scripts_key == crates_key, f"scripts 那一路 {scripts_key[:16]}，crates 那一路 {crates_key[:16]}"


def run_cell(arguments, extra_environment, check):
    """在小树上跑一次 run；check(退出码, 标准错误, 装置日志, 旁证目录, 小树那一份库, 临时目录, 环境) → (过不过, 说明)。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        environment.update({name: value.replace("<tree>", os.path.join(directory, "tree")) for name, value in extra_environment.items()})
        code, _stdout, stderr = selftest_call(directory, environment, ["run"] + arguments)
        log_path = environment["FAKE_DEVICE_LOG"]
        device_log = open(log_path, encoding="utf-8").read() if os.path.isfile(log_path) else ""
        return check(code, stderr, device_log, environment["FAKE_DEVICE_WORK"], selftest_store(directory), directory, environment)


def work_text(work, name):
    path = os.path.join(work, name)
    return open(path, encoding="utf-8").read() if os.path.isfile(path) else ""


def device_binary_seen(work, build_root=APPARATUS_PACKAGE_DIRECTORY):
    """假装置看到的 DEVICE_CODE_KEY_DEVICE_BINARY 与它自己的路径（小树里那个构建根下编好的那一份）：→ (相等不相等, 看到的)。"""
    seen = work_text(work, "device-binary.txt") or "假装置没起"
    expected = os.path.normpath(os.path.join(os.path.dirname(work), "tree", build_root, "target", "release", "fake-device"))
    return seen == expected, seen


def cargo_build_seen(directory, build_root, package):
    """假 cargo 记下的每一次 build（起的目录<TAB>参数）是不是恰好一次、在小树的这个构建根里起 build --release -p <包名> --bin fake-device：→ (是不是, 记下的)。"""
    seen = [line for line in work_text(directory, "cargo.log").split("\n") if line]
    expected = f"{os.path.realpath(os.path.join(directory, 'tree', build_root))}\tbuild --release -p {package} --bin fake-device"
    return seen == [expected], seen


def stored_item(store, work):
    """假装置这一趟头一个写的项（它记在旁证目录的 written-keys.txt）在临时服务里的原样（不经转交，撤回标记照原样读出）。→ (代码键, 原样或 None)。"""
    keys = [line for line in work_text(work, "written-keys.txt").split("\n") if line.startswith("items/")]
    if not keys:
        return "", None
    return keys[0][len("items/"):-len(".out")], store.read(keys[0])


def check_refused(code, stderr, device_log, work, _store, _directory, _environment):
    calls = [line.split("\t") for line in device_log.split("\n") if line]
    binary_passed, binary_seen = device_binary_seen(work)
    passed = code == EXIT_REFUSED and calls == [["segment-four"]] and "键算不出" in stderr and binary_passed and not work_text(work, "replies.txt")
    return passed, f"退 {code}，装置被起成 {calls}，装置看到的 {DEVICE_BINARY_VARIABLE}={binary_seen}，收到的回应「{work_text(work, 'replies.txt').strip()}」"


def check_kept_in(build_root, package, entry, source):
    """前后两次键相等的那一趟（build_root 那个构建根、package 那个包）：退 0、项与组成清单经转交写进临时服务、组成清单的单元行点到 source、
    假 cargo 在构建根里 build -p <包名> 恰好一次、装置看到的路径是构建根下编好的那一份。"""
    def check(code, stderr, device_log, work, store, directory, _environment):
        binary_passed, binary_seen = device_binary_seen(work, build_root)
        build_passed, builds = cargo_build_seen(directory, build_root, package)
        code_key, item = stored_item(store, work)
        composition = store.read(f"codes/{code_key}.txt") if code_key else None
        passed = code == 0 and item is not None and item.startswith(f"judged {code_key}\nmeta ".encode()) and "--code-key-file" in device_log \
            and composition is not None and f"code_key\t{entry}\t{code_key}".encode() in composition and FORM_LINE.encode() in composition \
            and f"\t{source}:".encode() in composition and binary_passed and build_passed
        return passed, (f"退 {code}，临时服务里的项 {item!r}，组成清单{'在' if composition else '不在'}，假 cargo 记下的 build {builds}，"
                        f"装置看到的 {DEVICE_BINARY_VARIABLE}={binary_seen}，{stderr.strip()[-200:]}")
    return check


def check_device_exit(code, _stderr, _device_log, _work, _store, _directory, _environment):
    return code == 4, f"退 {code}"


def check_withdrawn(expected_code, sentence):
    """装置退出之后那一次核不过（键变了、取不到）：退 expected_code、标准错误有 sentence、这一趟写的项在临时服务里是撤回标记。"""
    def check(code, stderr, _device_log, work, store, _directory, _environment):
        code_key, item = stored_item(store, work)
        passed = code == expected_code and item == WITHDRAWN_VALUE and sentence in stderr
        return passed, f"退 {code}（要 {expected_code}），临时服务里 items/{code_key[:16]}….out 是 {item!r}，{stderr.strip()[-200:]}"
    return check


def check_removed(code, stderr, device_log, work, store, directory, environment):
    """编译与跑之间源码变了：撤掉、退 1；把源码改回去（代码键回到这一趟那一把）再跑一趟，经转交读到撤回标记要当不在、现判。"""
    passed, detail = check_withdrawn(EXIT_FAILED, "源码在编译与跑之间变了")(code, stderr, device_log, work, store, directory, environment)
    source = os.path.join(directory, "tree", SELFTEST_SCRIPTS_SOURCE)
    with open(source, encoding="utf-8") as handle:
        text = handle.read()
    with open(source, "w", encoding="utf-8") as handle:
        handle.write(text.replace("compute(2)", "compute(1)"))
    again = {name: value for name, value in environment.items() if name != "FAKE_DEVICE_EDIT_SOURCE"}
    again_code, _stdout, again_stderr = selftest_call(directory, again, ["run"] + KEY_ARGUMENTS + ["--", "segment-four"])
    outcomes = work_text(work, "outcomes.txt")
    again_passed = again_code == 0 and outcomes == f"{SCRIPTS_ENTRY}\tjudged\n"
    return passed and again_passed, f"{detail}；改回之后再跑退 {again_code}、假装置报「{outcomes.strip()}」（要现判），{again_stderr.strip()[-160:]}"


def check_unreachable(code, _stderr, _device_log, work, _store, _directory, _environment):
    replies = work_text(work, "replies.txt").split("\n")
    passed = code == prune_server_client.EXIT_UNAVAILABLE and replies[0].startswith(f"failed {prune_server_client.EXIT_UNAVAILABLE} ")
    return passed, f"退 {code}（要 {prune_server_client.EXIT_UNAVAILABLE}），假装置收到的回应 {replies[:3]}"


def check_unknown_request(code, _stderr, _device_log, work, _store, _directory, _environment):
    replies = work_text(work, "replies.txt").split("\n")
    passed = code == EXIT_REFUSED and replies[0].startswith(f"failed {EXIT_REFUSED} 认不出的请求")
    return passed, f"退 {code}（要 {EXIT_REFUSED}），假装置收到的回应 {replies[:2]}"


def selftest():
    """每种判定至少一格；弄坏开关打开时（DEVICE_CODE_KEY_BREAK，文件头逐个列了红在哪一格）照样跑，红在点名的那一格。只在临时 prune-server 上跑。"""
    problem = prune_server_client.selftest_service_problem()
    if problem:
        print(f"  ✗ device-code-key 自证没跑：{problem}")
        print(f"     → 怎么办：{prune_server_client.SELFTEST_SERVICE_REMEDY}")
        return EXIT_USAGE
    run_arguments = KEY_ARGUMENTS + ["--", "segment-four"]
    scripts_manifest, scripts_lock = (os.path.join(APPARATUS_PACKAGE_DIRECTORY, name) for name in ("Cargo.toml", "Cargo.lock"))
    cells = [
        ("只加一行注释键不变", lambda: key_cell((SELFTEST_SCRIPTS_SOURCE, "    helper_function();\n", "    // 注释\n    helper_function();\n"), False)),
        ("改入口调到的函数键变", lambda: key_cell((SELFTEST_SCRIPTS_SOURCE, "compute(1)", "compute(2)"), True)),
        ("加一个没人调的函数键不变",
         lambda: key_cell((SELFTEST_SCRIPTS_SOURCE, "fn helper_function() {", "fn unrelated_function() {\n}\n\nfn helper_function() {"), False)),
        ("profile.release 改一个值键变", lambda: key_cell((scripts_manifest, "opt-level = 3", "opt-level = 2"), True)),
        ("闭包点到的包在 Cargo.lock 里抬一版键变", lambda: key_cell((scripts_lock, 'version = "1.0.0"', 'version = "1.0.1"'), True)),
        ("闭包没点到的包抬一版键不变", lambda: key_cell((scripts_lock, 'version = "2.0.0"', 'version = "2.0.1"'), False)),
        ("crates 包：取键、在仓根 cargo build -p <包名>、起仓根 target 下的装置、项经转交写进临时服务",
         lambda: run_cell(CRATES_KEY_ARGUMENTS + ["--", "segment-four"], {}, check_kept_in(".", "fake-crate", CRATES_ENTRY, SELFTEST_CRATE_SOURCE))),
        ("同一份源码放两种包目录：代码键相同", same_source_two_packages_cell),
        ("crates 包：闭包点到别的 crate 的单元，那个 crate 依赖的外部包抬一版键变",
         lambda: key_cell(("Cargo.lock", 'version = "3.0.0"', 'version = "3.0.1"'), True, CRATES_KEY_ARGUMENTS,
                          prepare=(SELFTEST_CRATE_SOURCE, "    helper_function();\n", "    helper_function();\n    helper_library_function();\n"))),
        ("crates 包：profile 读仓根清单，改一个值键变", lambda: key_cell(("Cargo.toml", "opt-level = 3", "opt-level = 2"), True, CRATES_KEY_ARGUMENTS)),
        ("入口头一段写成 bin 名：退 2、点名要写的 crate 名",
         lambda: run_usage_error(["key", "--binary", "fake-device", "--package-directory", SELFTEST_CRATE_DIRECTORY, "--entry", "fake_device::entry_function",
                                  "--threads", "1"], "的 crate 名 fake_crate")),
        ("包目录认不得：退 2", lambda: run_usage_error(["key", "--binary", "fake-device", "--package-directory", "scripts", "--entry", SCRIPTS_ENTRY,
                                                     "--threads", "1"], "（相对仓根），给的是 scripts")),
        ("拒算：退 3、装置照原参数起、不加别的开关、不给代码键", lambda: run_cell(run_arguments, {"FAKE_UNIT_GRAPH_REFUSE": "1"}, check_refused)),
        ("认不得的选项退 2（键算不出时换一组装置参数的选项已删，写了就是这一种）",
         lambda: run_usage_error(["run"] + KEY_ARGUMENTS + ["--never-used-option=segment-four", "--", "segment-four"])),
        ("unit-graph 找不到：退 78", lambda: (lambda result: (result[0] == EXIT_UNIT_GRAPH_MISSING, f"退 {result[0]}"))(
            run_missing_unit_graph())),
        ("前后两次键相等：项经转交写进临时服务、交装置的退出码",
         lambda: run_cell(run_arguments, {}, check_kept_in(APPARATUS_PACKAGE_DIRECTORY, "fake-package", SCRIPTS_ENTRY, SELFTEST_SCRIPTS_SOURCE))),
        ("装置退 4：这里也退 4", lambda: run_cell(run_arguments, {"FAKE_DEVICE_EXIT": "4"}, check_device_exit)),
        ("编译与跑之间改了入口调到的函数：撤掉这一趟写进库的项、退 1，改回之后再跑经转交读成不在",
         lambda: run_cell(run_arguments, {"FAKE_DEVICE_EDIT_SOURCE": f"<tree>/{SELFTEST_SCRIPTS_SOURCE}"}, check_removed)),
        ("装置退出之后取键崩了：撤掉这一趟写进库的项、交装置的退出码",
         lambda: run_cell(run_arguments, {"FAKE_DEVICE_SPOIL_UNIT_GRAPH": "crash", "FAKE_DEVICE_EXIT": "4"}, check_withdrawn(4, "装置退出之后键取不到"))),
        ("装置退出之后取键拒算：撤掉这一趟写进库的项、装置退 0 时退 3",
         lambda: run_cell(run_arguments, {"FAKE_DEVICE_SPOIL_UNIT_GRAPH": "refuse"}, check_withdrawn(EXIT_REFUSED, "装置退出之后键取不到"))),
        ("prune-server 连不上：装置头一个读就收到 failed 75、这里退 75",
         lambda: run_cell(run_arguments, {prune_server_client.SOCKET_VARIABLE: "<tree>/absent.sock", prune_server_client.TCP_ADDRESS_VARIABLE: "127.0.0.1:1",
                                          prune_server_client.RECONNECT_LIMIT_VARIABLE: "1"}, check_unreachable)),
        ("认不出的请求：回 failed 3、装置照它退 3", lambda: run_cell(run_arguments, {"FAKE_DEVICE_UNKNOWN_REQUEST": "1"}, check_unknown_request)),
        ("工具链行带一行 host: <三元组>，closure 与 key 两趟都给（unit-graph 缺它退 2）", host_line_cell),

        ("reuse-three-questions", three_questions_cell),
        ("reuse-sentinel", reuse_sentinel_cell),
        ("用法错：没给 --entry 退 2", lambda: run_usage_error(["key", "--binary", "fake-device"])),
        ("用法错：run 没写 -- 退 2", lambda: run_usage_error(["run"] + KEY_ARGUMENTS)),
    ]
    failures = 0
    for name, cell in cells:
        try:
            passed, detail = cell()
        except Exception as error:  # 自证里一格炸了记红、接着跑别的格
            passed, detail = False, f"{type(error).__name__}：{error}"
        failures += 0 if passed else 1
        print(f"  {'✓' if passed else '✗'} {name}" + ("" if passed else f"：{detail}"))  # gate-lint:detail
    print(f"device-code-key 自证：{len(cells)} 格，红 {failures} 格")
    if failures:
        print("  → 怎么办：照上面 ✗ 那几格后面写的不对处修；设着 DEVICE_CODE_KEY_BREAK 的，这就是要的红")
    return 1 if failures else 0


SELFTEST_TWO_ENTRY_SOURCE = """fn entry_function() {
    helper_function();
}

fn helper_function() {
    let value = fake_external::compute(1);
}

fn other_entry() {
    other_helper();
}

fn other_helper() {
    let value = 2;
}

fn unused_function() {
    let value = 3;
}
"""


THREE_QUESTION_PACKAGES = (
    # (包目录, 相对小树的装置源码, 两个入口（第二个是闭包内那一改只影响的那一个）, 另给的选项)
    (APPARATUS_PACKAGE_DIRECTORY, SELFTEST_SCRIPTS_SOURCE, (SCRIPTS_ENTRY, SCRIPTS_OTHER_ENTRY), ()),
    (SELFTEST_CRATE_DIRECTORY, SELFTEST_CRATE_SOURCE, (CRATES_ENTRY, CRATES_OTHER_ENTRY), ("--package-directory", SELFTEST_CRATE_DIRECTORY)),
)


def three_questions_package_runs(source_relative, entries, options):
    """一种包目录一棵小树（库各占一份）：头一趟、同一份输入、改没人调的 unused_function（闭包外）、改只有第二个入口调到的 other_helper（闭包内）各跑一趟 run，
    读假装置报的各入口现判还是复用。→ ([四趟各 {入口: 结局}] 或 None, 没跑成时的说明)。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        with open(os.path.join(directory, "tree", source_relative), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_TWO_ENTRY_SOURCE)
        runs = []
        for edit in (None, None, ("fn unused_function() {\n    let value = 3;\n}", "fn unused_function() {\n    let value = 4;\n}"),
                     ("    let value = 2;\n", "    let value = 5;\n")):
            problem = edit_tree_file(directory, (source_relative,) + edit) if edit is not None else None
            if problem is not None:
                return None, problem
            arguments = ["run", "--binary", "fake-device", "--threads", "1"] + list(options) + [word for entry in entries for word in ("--entry", entry)] \
                + ["--", "segment-four"]
            code, _stdout, stderr = selftest_call(directory, environment, arguments)
            outcomes = dict(line.split("\t") for line in work_text(environment["FAKE_DEVICE_WORK"], "outcomes.txt").split("\n") if line)
            if code != 0 or sorted(outcomes) != sorted(entries):
                return None, f"run 退 {code}、假装置报了 {outcomes}：{stderr.strip()[-300:]}"
            forced = forced_entry(environment["FAKE_DEVICE_WORK"], stderr)
            runs.append({entry: lib_reuse_checks.FORCED if entry == forced else lib_reuse_checks.REUSED if outcomes[entry] == "reused" else lib_reuse_checks.JUDGED
                         for entry in entries})
        return runs, ""


def three_questions_cell():
    """复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：两种包目录（THREE_QUESTION_PACKAGES）各一棵小树、各两个入口，四项合起来问一次，
    项名是「<包目录> <入口>」；假装置按代码键经转交读写临时服务里的库（读得回算复用，读不回现判并写回）；每趟两棵树各有一份哨兵，强制现判至多各一条；
    打一行规范输出。已有同义的格在这里点名：「只加一行注释键不变」「加一个没人调的函数键不变」是 ②，「改入口调到的函数键变」是 ③。"""
    merged, affected = [{}, {}, {}, {}], []
    for package_directory, source_relative, entries, options in THREE_QUESTION_PACKAGES:
        runs, problem = three_questions_package_runs(source_relative, entries, options)
        if runs is None:
            return False, f"{package_directory}：{problem}"
        for merged_run, run in zip(merged, runs):
            merged_run.update({f"{package_directory} {entry}": state for entry, state in run.items()})
        affected.append(f"{package_directory} {entries[1]}")
    passed, problems = lib_reuse_checks.report_three_questions(SCRIPT_KEY, *merged, affected, sentinels=len(THREE_QUESTION_PACKAGES))
    return passed, "；".join(problems)


SENTINEL_CASES = (
    # (哪一种, 第二趟另设的环境, 第二趟该退几, 标准错误里该有的红句；绿的那一种写 None)
    ("都照规矩", {}, 0, None),
    ("哨兵被现判（读到了还写）", {"FAKE_DEVICE_REJUDGE_FOUND": "1"}, EXIT_FAILED, "复用失灵"),
    ("强制现判写回的判定变了", {"FAKE_DEVICE_JUDGED_SUFFIX": " changed"}, EXIT_FAILED, "判定不稳"),
    ("强制现判读成不在却不写回", {"FAKE_DEVICE_ABSENT_UNWRITTEN": "1"}, EXIT_FAILED, "强制现判没起作用"),
)


def reuse_sentinel_cell():
    """复用哨兵在转交里（文件头「判定库的转交」复用哨兵那一段）：两入口的小树跑两趟，第二趟的种子取强制现判落在第 1 条命中的那一个
    （entry_function 那一条强制现判、other_entry 那一条当哨兵）；SENTINEL_CASES 四种各一棵树，绿的那一种另核 DEVICE_CODE_KEY_SENTINEL 那一行点名了两条。"""
    seed = next(seed for seed in range(1, 1 << 16) if forced_hit_number(seed) == 1)
    entries = [SCRIPTS_ENTRY, SCRIPTS_OTHER_ENTRY]
    arguments = ["run", "--binary", "fake-device", "--threads", "1"] + [word for entry in entries for word in ("--entry", entry)] + ["--", "segment-four"]
    wrong = []
    for label, variables, expected_code, sentence in SENTINEL_CASES:
        with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
            environment = selftest_tree(directory)
            with open(os.path.join(directory, "tree", SELFTEST_SCRIPTS_SOURCE), "w", encoding="utf-8") as handle:
                handle.write(SELFTEST_TWO_ENTRY_SOURCE)
            first_code, _stdout, first_stderr = selftest_call(directory, environment, arguments)
            second_environment = dict(environment, **variables, **{lib_reuse_checks.SENTINEL_SEED_VARIABLE: str(seed)})
            code, _stdout, stderr = selftest_call(directory, second_environment, arguments)
            fields = sentinel_fields(stderr)
            forced = forced_entry(environment["FAKE_DEVICE_WORK"], stderr)
            if sentence is None:
                passed = first_code == 0 and code == 0 and forced == entries[0] and fields.get("sentinels", "-") != "-" and f"种子 {seed}" in stderr
            else:
                passed = first_code == 0 and code == expected_code and sentence in stderr and f"种子 {seed}" in stderr
            if not passed:
                wrong.append(f"{label}：头一趟退 {first_code}，第二趟退 {code}（要 {expected_code}{'、红句「' + sentence + '」' if sentence else ''}），"
                             f"强制现判 {forced}，{SENTINEL_LINE} {fields}：{(stderr or first_stderr).strip()[-240:]}")
    return not wrong, "；".join(wrong)


def sentinel_fields(stderr):
    """标准错误里最后一行 DEVICE_CODE_KEY_SENTINEL 的各字段；没有交 {}。"""
    lines = [line for line in stderr.split("\n") if line.startswith(SENTINEL_LINE + " ")]
    return dict(field.split("=", 1) for field in lines[-1].split(" ")[1:] if "=" in field) if lines else {}


def forced_entry(work, stderr):
    """转交这一趟抽中强制现判的那一条是假装置的哪个入口（outcome-keys.txt 记着入口与相对键）；没抽到交 None。"""
    forced = sentinel_fields(stderr).get("forced", "-")
    entries_by_key = {key: entry for entry, key in (line.split("\t") for line in work_text(work, "outcome-keys.txt").split("\n") if line)}
    return entries_by_key.get(forced)


def host_line_cell():
    """取一次键：退 0、组成清单里有 toolchain<TAB>host: <假 rustc -vV 的三元组> 那一行（假 unit-graph 照真的缺它退 2）。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        code, stdout, stderr = selftest_call(directory, environment, ["key"] + KEY_ARGUMENTS)
        passed = code == 0 and "toolchain\thost: x86_64-unknown-linux-gnu" in stdout.split("\n")
        return passed, f"退 {code}，{stderr.strip()[:300]}"


def run_missing_unit_graph():
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        environment[UNIT_GRAPH_VARIABLE] = os.path.join(directory, "tools", "not-there")
        return selftest_call(directory, environment, ["key"] + KEY_ARGUMENTS)


def run_usage_error(arguments, sentence=""):
    """在小树上起一次：要退 2，sentence 给了的另要标准错误里有它。"""
    with tempfile.TemporaryDirectory(prefix="device-code-key-selftest-") as directory:
        environment = selftest_tree(directory)
        code, _stdout, stderr = selftest_call(directory, environment, arguments)
        return code == EXIT_USAGE and sentence in stderr, f"退 {code}，{stderr.strip()[:300]}"


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
