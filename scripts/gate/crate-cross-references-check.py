#!/usr/bin/env python3
# admission: always 判的是此刻被判那棵树各包的编译单元读了哪些文件与两张表；按包的单元键在判定库里复用，上一次的结论不由准入替这一次作保
# run-condition: command python3 git
"""门禁 checker-independence-and-sync 的 crate-cross-references 格的判法：crates/ 下各包的编译单元不读别的包目录下的源码。
判据在 .claude/rules/directory-layout.md「crates/：各包放什么」，一句话：编译单元读进来的文件落在别的包目录里、或不属任何包的，判红，登放行表才过；
  只有别的包目录下后缀是 .toml、.md、.tsv 的数据文件放行，条件是只经 include_str! / include_bytes! 读进来（当模块或 include! 编进来的不论后缀照源码判）；
  构建脚本报的 rerun-if-changed 落在别的包目录里的照同一句判，拷进 OUT_DIR 再编进来的别的包文件不论后缀判红。
读了哪些文件不在源码字面上认，读编译产物：cargo check 给每个编译单元写的 dep-info（rustc 的 .d）列着这个单元真读进来的每一份文件，写法不论
（cfg_attr、改名导入的 include 族宏、方括号与花括号定界、原始字符串、行内模块里的 #[path]、绝对路径、经别的后缀中转、.md 后缀当模块、名叫 target 或点起头的目录都一样落进 .d）。
rustc 的 .d 不分一份文件是当源码编进来还是当数据读进来；数据后缀的那几份是不是只当数据读，才在源码字面上认（下面「数据读法」），认不出的照源码判。

命令行：
  crate-cross-references-check.py check <根> [--lag-base-state <table|none|unreadable:<原因>>] [--lag-base-file <文件>] [--lag-base-name <基准名>]
      判一次（门禁那一格起它；cwd 不限，下面的路径都按 <根> 解）。滞后表只缩不涨的基准由门禁取好交进来：table 是取到了、在 --lag-base-file，
      none 是基准里还没有这张表或不在 git 仓里（不比），unreadable:<原因> 是取不到（判红）。
  crate-cross-references-check.py write-sample-tools <根>
      往 <根>/.crate-cross-references-sample-tools/ 写假 cargo 与假 rustc（样本档与自证用，下面「样本」）。
  crate-cross-references-check.py --selftest
      自证；只在临时 prune-server 底下跑（开跑先调客户端 selftest_service_problem，裸跑退 2）：
      python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 scripts/gate/crate-cross-references-check.py --selftest

编译：
  cargo check --offline --message-format=json --all-targets --all-features -p <这一趟要现判的包>…，在 <根> 起；编译目录取环境里的 CARGO_TARGET_DIR
  （门禁文件头的会编声明让统一任务服务给这一次尝试一份），没设、不是绝对路径、落在 <根> 里都判红、一个都不编（不往仓根 target 编）。
  起 cargo 之前先过 scripts/hosts/rocksdb-prebuilt.py entry --root <根> --cargo <cargo>（RocksDB 预编库的取值，判红照它的红句退 1）。
  编译输出里每条 compiler-artifact 是一个编译单元（包、目标种类、目标名）；它的 dep-info 是产物文件名同一目录下「去掉扩展名的名字 .d」
  （.rmeta、.rlib 去掉 lib 前缀）；构建脚本单元（target.kind 有 custom-build）的产物是 build/<包名>-<哈希>/build-script-build，
  rustc 在同一目录写的 .d 叫 build_script_build-<哈希>.d（产物名的连字符换成下划线，哈希取那个目录名最后一段）；落在判的包里的单元找不到 dep-info 判红。build-script-executed 给出各包构建脚本的 OUT_DIR，
  OUT_DIR 同一层的 output 里读构建脚本报的 rerun-if-changed 与 rerun-if-env-changed（复用键要它们，下面「复用」）。编不过判红，那几个包这一趟不入库。

判的包：cargo metadata --format-version 1 --offline --filter-platform <host>（不编；<host> 取这台机器的系统与架构查 HOST_TRIPLES，Linux 只认 glibc 的，
  表里没有的判红；不起 rustc 也不起 cargo -V，下面「复用」不进键的工具链才不会经判法闭包里的版本询问进键）里工作区成员中包目录在 <根>/crates/ 下的那几个；
  一个都没有判红。不带 --filter-platform 时 cargo metadata 连别的平台才用的依赖也要取，离线缓存里没有就退 101、一个包都判不了；cargo check 只编这一台 host 的，
  两边罩的依赖相同。
  dep-info 列的每份文件（相对路径按工作区根解、解开符号链接）照这一组分类（classify；先中先算）：
    同包            在这个包的目录里                                                                         放行
    构建脚本生成    在这一趟某个构建脚本的 OUT_DIR 里                                                         放行
      拷进来的别的包文件  同上，而内容与别的包目录里一份文件逐字节相同（下面「OUT_DIR 拷贝」）：不论后缀判红
    依赖源码        在 cargo metadata 里一个 registry 或 git 来的依赖包的目录里                                放行
    别的包          在 cargo metadata 里另一个本地包的目录里，或往上找到另一份带 [package] 的 Cargo.toml（工作区之外的本地工程、仓外的拷贝）：
                      后缀是 .toml、.md、.tsv 而这个单元只当数据读它（下面「数据读法」）的算数据文件，放行；别的一律算跨包源码，判红
    仓里不属任何包  在 <根> 里、往上到 <根> 都没有带 [package] 的 Cargo.toml：不论后缀判红
    仓外不属任何包  在 <根> 外、往上找不到带 [package] 的 Cargo.toml：不论后缀判红
  构建脚本报的 rerun-if-changed（output 里读，目录照它自己算一份）也分一次：落在别的包目录里的，后缀是 .toml、.md、.tsv 的算数据文件放行，
    别的判红（构建脚本运行时读别的包的文件）；落在同包、OUT_DIR、依赖源码与不属任何包的位置的不判（仓根的 Cargo.toml、Cargo.lock 这类工作区清单由构建脚本读来算键）。
  放行表另认一种前缀行（下面「两张表」）：一行放行一个构建脚本读一整个别的包，只对构建脚本报的 rerun-if-changed 这一类（分类名「构建脚本运行时读的别的包文件」）生效；
    编进编译单元的（dep-info 里的，同一份文件构建脚本也报了 rerun-if-changed 的算在内）与拷进 OUT_DIR 再编进来的照旧逐文件登，前缀行放不过它们。
  判红的那几类登在两张表里的放过：放行表 .claude/gate.d/crate-cross-references-allowed.tsv（确要留着的）、滞后表 .claude/gate.d/crate-cross-references-lag.tsv（只缩不涨）。
  数据读法（data_read_paths，编译时照这个单元 .d 列的文件现认，结果记进事实的 data_reads）：这个单元读的文件里，被当源码解析的那几份（.d 列的全部，
    减去只被当数据读的；后者照下一句反复算到不再变；读不到的不切，编译刚读过、平常都在）按 Rust 词法切开（去掉注释，字符串与原始字符串照字面量取值），认两样：
    数据读：include_str! / include_bytes! 的参数是一个字符串字面量，或 concat! 套的字符串字面量与 env!("CARGO_MANIFEST_DIR")，
      照所在文件的目录（带 env!("CARGO_MANIFEST_DIR") 的照这个包的目录）解出来就是那份文件；
    别的指法：不在 include_str! / include_bytes! 参数里的字符串字面量或 concat!，照所在文件的目录或这个包的目录解出来是那份文件，
      或去掉开头的 ./、../ 之后至少两段、与那份文件相对 <根> 的路径按段一头对齐（#[path]、include!、改名导入的宏、行内模块里的 #[path] 都落在这一样）。
    一份数据后缀的文件有数据读、没有别的指法，才算只当数据读；这个单元的源码里有 `as include_str`、`as include_bytes`、`macro_rules! include_str`、
    `macro_rules! include_bytes`（这两个名字被改指别的宏）的、有一份超过 SCANNED_SOURCE_BYTES_LIMIT 的，这个单元一份都不算。
  OUT_DIR 拷贝（编译时记 OUT_DIR 里每份被读的文件的大小与 sha256，进事实的 generated；分类时与 <根> 里本地包目录下的文件比，只比大小相同的）：
    内容去掉空白后为空的不比；与读它的包自己目录下的一份相同的算构建脚本生成、放行；只与别的包的相同的判红，红句点名那份原文件。
  判不到的：特性组合之外被 cfg 掉的文件（没编进来）、运行时读而构建脚本没报 rerun-if-changed 的文件、过程宏自己读的文件；
    拷进 OUT_DIR 时改过内容的别的包文件（加一行、拼几份、照模板生成，内容不再逐字节相同）；数据后缀的文件由宏拼出路径、没有一个字面量与它对得上地当模块编进来，
    而同一个单元另有一处 include_str! 字面量读它（这时照只当数据读放行）；
    只换了工具链、cargo 配置链、Cargo.lock 或构建环境变量（下面「复用」里不进键的那几样）而包目录与读过的文件都没变的包照复用，
    换了之后 cargo 让编译单元读的文件变了（RUSTFLAGS、cargo 配置翻了 #[path] 前的 cfg、换了依赖版本）也判不到：这是 D13（验证路线） 已定项 17
    「所有判定的键都不带工具链（含 cargo 解析后的生效构建环境）、外部工具版本」与不带依赖版本的代价，不另立判法。
  两张表一行「<编译单元所在的包目录><TAB><引到的文件><TAB># 理由>」：包目录与文件都相对 <根>（仓外的文件写绝对路径），理由至少 8 个字；
  第一列不是判到的包、写不对的行判红；点名的引用这一趟没有的行判红（那个包编不过、这一趟没有事实的不判）；滞后表比基准多出行判红；基准取不到判红。
  放行表的前缀行「<构建脚本所在的包目录><TAB><别的包目录>/<TAB># 理由>」：第二列以 / 收尾、写一个别的包的目录（相对 <根>，仓外的写绝对路径），放行这个包的构建脚本报的 rerun-if-changed 里落在那个目录下的每一份
    （构建脚本为算判定的单元键，逐份读别的包源码取指纹，只算哈希、不编进本包；理由写明是算单元键）。只放行构建脚本报的 rerun-if-changed 那一类，编进编译单元的与拷进 OUT_DIR 的不放行（上面「判的包」）。
    第二列指的目录不是一个包的目录（不在，或这一趟找不到以它为目录的包）、这一趟读它的包的构建脚本没有一处 rerun-if-changed 落在它里面的，同「点名的引用这一趟没有的行」判红；
    前缀行只在放行表里认，滞后表里第二列以 / 收尾的行照一份文件算（引不到，判红）。

复用（判定库：prune-server，unit-verdicts 族，种类 crate-cross-references-check.py:packages，库名照 scripts/gate/admission.py 的写法
  <被判那棵树的 git common-dir>/singlefs-unit-verdicts/<种类>，只当名字；取不到 git common-dir 判红、一个都不判）：
  一个包一条判定，存的是事实：这个包每个编译单元读的文件（OUT_DIR 里的写成 OUT_DIR:<包名>/<OUT_DIR 里的相对路径>，依赖源码写成 DEPENDENCY:<包名>-<版本>/<相对路径>，
  别的照上面的相对或绝对路径）、其中只当数据读的那几份（data_reads，只记这个包目录外的）、OUT_DIR 里被读的每份文件的大小与 sha256（generated）、
  它们的 env-dep 变量名（CARGO_ 起头的与 OUT_DIR 除外，那几个由清单与布局定）、构建脚本报的 rerun-if-changed
  （路径照读到的文件那样写：OUT_DIR 里的写 OUT_DIR:<包名>/<相对>，事实里不留编译目录的绝对路径）与 rerun-if-env-changed。
  分类与两张表每趟照事实现判，不入库（表改了不用重编）。判定另记单元键各成分（下面每行的头一个词）各自的 sha256，作废时比它报哪几样变了。
  单元键：下面这些行排好序、各以换行收尾取 sha256（package_key）：
    judge <判法闭包的键>          scripts/gate/admission.py unit-key <判法所在的仓> - judge:scripts/gate/crate-cross-references-check.py::recorded_package_facts 打的键
    command <参数>               上面那条 cargo check 去掉 -p 的参数
    manifest <路径> <sha256>      <根> 的 Cargo.toml（工作区成员与路径依赖定哪份文件属哪个包）
    package-file <路径> <sha256>  这个包目录下的每一份文件（跳过 .git 与带 CACHEDIR.TAG 的编译目录；点起头的目录与名叫 target 的源码目录照收）
    read <路径> <sha256>          上一条判定里这个包读过的、落在 <根> 里而在这个包目录外的文件（这一包上一次判定在库里 items/ 那一份；头一次没有）
    read-outside <sha256>         同上、落在 <根> 外而不是依赖源码与 OUT_DIR 的文件，只进内容
    build-input <路径> <sha256>   上一条判定里构建脚本报的 rerun-if-changed（目录展开成底下每一份文件；<根> 外的写 build-input-outside <sha256>）
    build-input-named <名>        同上、落在 OUT_DIR 里（OUT_DIR:<包名>/<相对>）或依赖源码里（DEPENDENCY:…）的，只进名字：OUT_DIR 里的文件是这一趟编的时候
                                  构建脚本才写的，起 cargo 之前没有可读的内容，它的内容由构建脚本的输入（这一组别的行）定；依赖源码不论内容都放行
    environment <名> <取值>       上一条判定里的 env-dep 变量与 rerun-if-env-changed 变量，取这一刻环境里的值（没设写 unset）；构建环境变量不进键：
                                  CARGO_ 起头的（cargo 的配置变量 CARGO_HOME、CARGO_PROFILE_*、CARGO_BUILD_*、CARGO_INCREMENTAL 与 cargo 给编译单元设的那几个）
                                  与 BUILD_ENVIRONMENT_VARIABLES 那几个（HOME、RUSTFLAGS、RUSTC 这类）
  不进键的（D13（验证路线） 已定项 17「所有判定的键都不带工具链（含 cargo 解析后的生效构建环境）、外部工具版本」，依赖版本同样不带）：
    工具链（cargo 与 rustc 的版本、<根> 的 rust-toolchain 与 rust-toolchain.toml）、cargo 配置链（<根>、它的上层目录与 cargo 家目录里的 .cargo/config 与
    .cargo/config.toml）、<根> 的 Cargo.lock、上面那几个构建环境变量的取值；换了它们而包目录与读过的文件都没变的照复用（文件头「判不到的」末一条）。
    host 照机器的系统与架构取（「判的包」），不起 rustc，也不进键。
  路径都相对 <根>：搬一份文件改变 rustc 找模块的结果，也改变它属于哪个包；<根> 本身、编译目录与 OUT_DIR 的绝对路径、家目录的路径、判定的名字、行号不进键。
  这一组是 cargo 自己判一个编译单元新不新鲜要看的输入里代码与数据那一部分（上一趟的 dep-info 与构建脚本的 rerun-if 声明）加包目录与工作区清单：
    都没变，同一份工具链与构建环境下 rustc 读的还是那一组文件。
  OUT_DIR 里生成的文件由构建脚本的输入罩（构建脚本没报 rerun-if-changed 的，cargo 照包目录里有没有改动判，package-file 罩着）。
  键与库里 keys/<键>.json 那一条判绿的相同就复用，不起 cargo；对不上的这一趟现判，判完照新事实重算键写 keys/<新键>.json 与 items/<包目录的 sha256>.json（同一帧）。
  现判那一包的那一行报为什么（与库里 items/ 那一条比）：「键变了：<成分名>、…」（这一刻照那一条的事实算的各成分 sha256 与它记的不同的那几样，
    成分名是上面每行的头一个词）；「键与库里那一条相同，这把键在库里没有绿判定」（例：判定不稳撤掉了）；「库里那一条认不得，照头一次现判」
    （那一条读不出事实或键的各成分，当没有）；库里没有这一包的、判法闭包拒算的不报。
  判法闭包算不出键（admission.py unit-key 退 4，拒算）的：全部现判、一条都不入库，成功句报拒算。
  复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」，ReuseSentinel）：可复用的包里抽 SINGLEFS_REUSE_SENTINEL_COUNT 个当哨兵、另抽一个强制现判；
    强制现判的那一包现判出的事实与库里那一条不同 ⇒「判定不稳」判红，并撤掉库里那一条（下一趟这一包照现判）；哨兵被现判 ⇒「复用失灵」判红。
    样本档（下面「样本」）不抽：样本钉死了现判与复用的个数。
  prune-server 不可用：客户端退 75，这一格照 75 退（不当库里没有现判）。

样本（门禁样本档）：被判的根不是这份脚本所在的仓、根里有 .gate-cells 与 .crate-cross-references-sample/、没设 GATE_STAGED_FROM 时按样本办：
  先查临时 prune-server（selftest_service_problem，不在退 2：样本不写真服务），往根里写假工具（write-sample-tools），PATH 前面接那个目录，
  编译目录用这一趟自己建的临时目录（判完删）；不抽复用哨兵。
  假工具读 .crate-cross-references-sample/scenario.json：packages（name、version、directory、source）、workspace_members、units（package、kind、name、src_path、
  deps_name、dep_info：.crate-cross-references-sample/ 下录好的 .d，写 scan 的由假 cargo 照根里的源码现录（从 src_path 起跟字面写的 mod、#[path]、include!、
  include_str!、include_bytes! 与 include!(concat!(env!("OUT_DIR"), "/<名>"))，路径写绝对路径，用到 OUT_DIR 的另写 env-dep:OUT_DIR）；构建脚本单元另给 build_directory：照真 cargo，.d 写在 build/<build_directory>/<deps_name>.d，
  compiler-artifact 给的产物是同一目录里的 <name>）、build_scripts（package、directory_name、output 各行、generated 生成的文件）、exit_code、
  stderr（exit_code 不是 0 时打到标准错误的几行，照真 cargo 编不过时的报错录；不给打一行缺省的）；
  录好的 .d、directory 与 stderr 里 @ROOT@、@TARGET@、@OUT_DIR:<包名>@ 换成这一趟的根、编译目录与那个包的 OUT_DIR。
  假 cargo：-V 打版本；metadata 照 packages 打，没带 --offline 或 --filter-platform <host> 的照真 cargo 在本仓工作区上的样子退 101（报取不到别的平台才用的依赖），
  --filter-platform 的值不是目标三元组的照真 cargo 问 rustc 时的样子退 101；
  check 照 -p 点名的包（没点名是全部）写 .d、OUT_DIR 与 output，逐条打 build-script-executed、compiler-artifact，
  末了 build-finished，退 exit_code；环境里设了 CRATE_CROSS_REFERENCES_SAMPLE_CHECK_LOG 的往那份文件追加一行这一趟点名的包（自证核只编了要现判的包）。
  编不过的那一种写法照真 cargo 录：编不过的编译单元没有 compiler-artifact、没有 .d，units 里不列它，exit_code 写 101、stderr 写真 cargo 的报错。
  假 cargo 带 rocksdb-prebuilt.py entry 认的那一行标记，entry 不判。假 rustc 只答 -V 与 -vV；判法不起它，key-unchanged-by-toolchain-and-cargo-configuration 换上另一份版本不同的，核键不随版本变。

退出码：0 绿；1 红；2 用法错、样本档没有临时 prune-server；75 prune-server 不可用。

弄坏开关 CRATE_CROSS_REFERENCES_BREAK=<项>（只给证红用；--selftest 本身与它起的子进程、门禁样本根里由门禁转来的认，平常跑设了退 2）。逐项写弄坏哪一处、打开后自证哪几格红（「✗ 自证 <格名>：」那一段）：
  CRATE_CROSS_REFERENCES_BREAK=dep-info-unread  不读 .d，每个编译单元只算它的根文件（compiler-artifact 的 src_path）
      「✗ 自证 build-script-dep-info-read：」「✗ 自证 cross-package-red：」「✗ 自证 allowed-row-green：」「✗ 自证 table-rows-keyed-by-package：」「✗ 自证 generated-and-dependency-allowed：」
      「✗ 自证 unpackaged-source-red：」「✗ 自证 missing-dep-info-red：」「✗ 自证 reuse-three-questions：」「✗ 自证 reuse-sentinel：」「✗ 自证 invalidated-key-names-changed-parts：」
      「✗ 自证 module-with-data-suffix-red：」「✗ 自证 unpackaged-any-suffix-red：」「✗ 自证 out-dir-copy-red：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」
  CRATE_CROSS_REFERENCES_BREAK=out-dir-not-allowed  OUT_DIR 里生成的文件不放行，照仓外不属任何包的文件分类（判红）
      「✗ 自证 generated-and-dependency-allowed：」「✗ 自证 out-dir-copy-red：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」「✗ 自证 reuse-across-hosts：」「✗ 自证 key-unchanged-by-toolchain-and-cargo-configuration：」
  CRATE_CROSS_REFERENCES_BREAK=table-keys-by-source-file  两张表的第一列照编译单元的根文件认（引用的键是「根文件、引到的文件」），不照包目录认
      「✗ 自证 build-script-dep-info-read：」「✗ 自证 cross-package-red：」「✗ 自证 allowed-row-green：」「✗ 自证 table-rows-keyed-by-package：」「✗ 自证 unpackaged-source-red：」
      「✗ 自证 reuse-three-questions：」「✗ 自证 reuse-sentinel：」「✗ 自证 module-with-data-suffix-red：」「✗ 自证 unpackaged-any-suffix-red：」「✗ 自证 out-dir-copy-red：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」
  CRATE_CROSS_REFERENCES_BREAK=stale-rows-accepted  表里点名的引用这一趟没有也不报
      「✗ 自证 stale-and-lag-rows-red：」
  CRATE_CROSS_REFERENCES_BREAK=lag-growth-accepted  滞后表比基准多出行不报
      「✗ 自证 stale-and-lag-rows-red：」
  CRATE_CROSS_REFERENCES_BREAK=unpackaged-sources-allowed  仓里与仓外不属任何包的文件一律照数据放行
      「✗ 自证 unpackaged-source-red：」「✗ 自证 unpackaged-any-suffix-red：」
  CRATE_CROSS_REFERENCES_BREAK=unpackaged-by-suffix  不属任何包的文件照后缀分：.rs、.wgsl 判红，别的照数据放行
      「✗ 自证 unpackaged-any-suffix-red：」
  CRATE_CROSS_REFERENCES_BREAK=data-suffix-by-name  别的包目录下后缀是 .toml、.md、.tsv 的文件不看读法，一律照数据放行（当模块编进来的也放）
      「✗ 自证 module-with-data-suffix-red：」
  CRATE_CROSS_REFERENCES_BREAK=out-dir-copies-unchecked  OUT_DIR 里被读的文件不与别的包目录里的文件比内容，一律照构建脚本生成放行
      「✗ 自证 out-dir-copy-red：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」
  CRATE_CROSS_REFERENCES_BREAK=build-inputs-unclassified  构建脚本报的 rerun-if-changed 不分类，落在别的包目录里的也不判
      「✗ 自证 build-script-runtime-read-red：」「✗ 自证 package-prefix-row-green：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」「✗ 自证 stale-package-prefix-rows-red：」
  CRATE_CROSS_REFERENCES_BREAK=package-prefix-rows-ignored  放行表的前缀行不认（照没有这一行）
      「✗ 自证 package-prefix-row-green：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」「✗ 自证 stale-package-prefix-rows-red：」
  CRATE_CROSS_REFERENCES_BREAK=package-prefix-allows-all-categories  前缀行连编进编译单元的也放行，不只构建脚本报的 rerun-if-changed
      「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」
  CRATE_CROSS_REFERENCES_BREAK=package-prefix-directory-unchecked  前缀行指的目录不核是不是一个包的目录
      「✗ 自证 stale-package-prefix-rows-red：」
  CRATE_CROSS_REFERENCES_BREAK=package-prefix-unused-accepted  前缀行这一趟没有一处被它放行也不报
      「✗ 自证 stale-package-prefix-rows-red：」
  CRATE_CROSS_REFERENCES_BREAK=compile-failure-accepted  cargo check 编不过照编过办（读得到的 dep-info 照记、照入库）
      「✗ 自证 compile-failure-red：」
  CRATE_CROSS_REFERENCES_BREAK=missing-dep-info-accepted  判的包里找不到 dep-info 的编译单元悄悄跳过
      「✗ 自证 missing-dep-info-red：」
  CRATE_CROSS_REFERENCES_BREAK=target-directory-unchecked  不核 CARGO_TARGET_DIR（没设照 cargo 的缺省编进根里的 target，落在根里也照编）
      「✗ 自证 target-directory-required：」
  CRATE_CROSS_REFERENCES_BREAK=package-files-unkeyed  单元键不收包目录下的文件
      「✗ 自证 reuse-three-questions：」「✗ 自证 invalidated-key-names-changed-parts：」
  CRATE_CROSS_REFERENCES_BREAK=previous-reads-unkeyed  单元键不收上一条判定里这个包读过的包外文件、构建脚本的输入与变量
      「✗ 自证 reuse-three-questions：」「✗ 自证 invalidated-key-names-changed-parts：」
  CRATE_CROSS_REFERENCES_BREAK=build-environment-variables-keyed  单元键照收构建环境变量（HOME、RUSTFLAGS、CARGO_ 起头的这类）的取值
      「✗ 自证 key-independent-of-compile-directory：」「✗ 自证 reuse-across-hosts：」「✗ 自证 key-unchanged-by-toolchain-and-cargo-configuration：」
  CRATE_CROSS_REFERENCES_BREAK=toolchain-and-cargo-configuration-keyed  单元键另收 RUSTC 的取值、<根> 的 Cargo.lock、rust-toolchain、rust-toolchain.toml、
      .cargo/config、.cargo/config.toml 与 cargo 家目录里 config、config.toml 的内容
      「✗ 自证 key-unchanged-by-toolchain-and-cargo-configuration：」
  CRATE_CROSS_REFERENCES_BREAK=judge-key-toolchain-keyed  判法闭包的键另收 RUSTC 的取值（判法闭包里有工具链询问时键随 RUSTC 的版本变，这里在键上叠 RUSTC 的值来造）
      「✗ 自证 key-unchanged-by-toolchain-and-cargo-configuration：」
  CRATE_CROSS_REFERENCES_BREAK=host-platform-unchecked  host 不核机器：表里没有的系统与架构照「<架构>-unknown-<系统>-gnu」猜一个，Linux 不是 glibc 的也认
      「✗ 自证 host-triple-from-machine：」
  CRATE_CROSS_REFERENCES_BREAK=target-directory-keyed  单元键另收 CARGO_TARGET_DIR 的取值（这一次尝试的编译目录）
      「✗ 自证 reuse-across-hosts：」
  CRATE_CROSS_REFERENCES_BREAK=out-dir-inputs-absolute  构建脚本报的 rerun-if-changed 落在 OUT_DIR 里的照绝对路径记进事实（带编译目录）
      「✗ 自证 key-independent-of-compile-directory：」「✗ 自证 reuse-across-hosts：」
  CRATE_CROSS_REFERENCES_BREAK=changed-components-unreported  现判那一包的那一行不报键里哪几样变了
      「✗ 自证 invalidated-key-names-changed-parts：」
  CRATE_CROSS_REFERENCES_BREAK=sentinel-skipped  不抽复用哨兵（强制现判那一包照复用）
      「✗ 自证 reuse-sentinel：」「✗ 自证 reuse-across-hosts：」「✗ 自证 key-unchanged-by-toolchain-and-cargo-configuration：」
  CRATE_CROSS_REFERENCES_BREAK=metadata-all-platforms  cargo metadata 不带 --filter-platform（假 cargo 照真 cargo 在本仓工作区上的样子退 101，一个包都判不了）
      「✗ 自证 build-script-dep-info-read：」「✗ 自证 cross-package-red：」「✗ 自证 allowed-row-green：」「✗ 自证 table-rows-keyed-by-package：」「✗ 自证 generated-and-dependency-allowed：」
      「✗ 自证 unpackaged-source-red：」「✗ 自证 stale-and-lag-rows-red：」「✗ 自证 compile-failure-red：」「✗ 自证 missing-dep-info-red：」
      「✗ 自证 reuse-three-questions：」「✗ 自证 reuse-sentinel：」「✗ 自证 invalidated-key-names-changed-parts：」
      「✗ 自证 module-with-data-suffix-red：」「✗ 自证 unpackaged-any-suffix-red：」「✗ 自证 out-dir-copy-red：」「✗ 自证 build-script-runtime-read-red：」「✗ 自证 package-prefix-row-green：」「✗ 自证 package-prefix-row-not-for-compiled-in-or-copies：」「✗ 自证 stale-package-prefix-rows-red：」「✗ 自证 reuse-across-hosts：」
      「✗ 自证 key-unchanged-by-toolchain-and-cargo-configuration：」
  CRATE_CROSS_REFERENCES_BREAK=build-script-dep-info-by-artifact-name  构建脚本单元的 .d 照产物名找（build-script-build.d），不照 rustc 真写的 build_script_build-<哈希>.d
      「✗ 自证 artifact-dep-info-path：」「✗ 自证 build-script-dep-info-read：」

自证的格（SELFTEST_CELLS，次序相同；各格在自己的临时目录里造小树、git init，判定写进临时服务里各自的命名空间，并行跑）：
  dep-info-parse                       rustc 的 .d：几条规则、每份读到的文件另有的空规则、转义的空格、两种 env-dep 行，交读到的文件与变量名
  artifact-dep-info-path               产物文件名到 .d 的候选：.rmeta 去 lib 前缀；构建脚本 build/<包名>-<哈希>/build-script-build 的是同一目录的 build_script_build-<哈希>.d
  build-script-dep-info-read           消费包的构建脚本单元（.d 照真 cargo 写在 build/<包名>-<哈希>/build_script_build-<哈希>.d）读提供包 tests/c/mod.rs：
                                       退 1、点名「crates/consumer  引 crates/provider/tests/c/mod.rs（custom-build:build-script-build；跨包源码）」，不报找不到 dep-info
  cross-package-red                    消费包的测试单元读提供包 tests/c/mod.rs，两张表都空：退 1、点名「crates/consumer  引 crates/provider/tests/c/mod.rs」
  allowed-row-green                    同上，放行表登了那一处：退 0、成功句「跨包源码与不属任何包的文件 1 处都登在放行表（1 行）」
  table-rows-keyed-by-package          同上，放行表第一列写测试根文件：退 1、点名那一行第一列不是判到的包、那一处没登记
  generated-and-dependency-allowed     消费包读 OUT_DIR 里生成的 .rs、依赖源码、经 include_str! 字面量读提供包的 Cargo.toml：退 0，三类各 1
  unpackaged-source-red                消费包读仓里不属任何包的 shared/common.rs：退 1、点名它
  unpackaged-any-suffix-red            消费包 include! 仓里不属任何包的 shared/snippet.inc、include_str! 读 notes/table.txt：退 1、两份都点名（仓里不属任何包的文件）
  module-with-data-suffix-red          消费包 #[path] 把提供包的 src/code.md 当模块编进来、include_str! 读提供包的 src/notes.md：退 1、只点名 code.md（跨包源码），
                                       notes.md 算别的包的数据文件 1
  out-dir-copy-red                     消费包的构建脚本往 OUT_DIR 写三份、库都 include! 进来：copied.rs 与提供包 src/x.rs 逐字节相同、own.rs 与自己包里一份相同、fresh.rs 谁都不同：
                                       退 1、只点名 copied.rs（构建脚本拷进 OUT_DIR 的别的包文件，内容与 crates/provider/src/x.rs 相同），构建脚本生成 2
  build-script-runtime-read-red        消费包的构建脚本报 rerun-if-changed=../provider/src/k.wgsl、../provider/Cargo.toml、../../notes/table.txt：退 1、只点名 k.wgsl
                                       （构建脚本运行时读的别的包文件）
  package-prefix-row-green             消费包的构建脚本报 rerun-if-changed=../provider/src/k.wgsl、../provider/src/shared.rs、../provider/Cargo.toml，放行表登一行前缀行 crates/consumer、crates/provider/：
                                       退 0、成功句「跨包源码与不属任何包的文件 2 处都登在放行表（1 行）」
  package-prefix-row-not-for-compiled-in-or-copies
                                       同一行前缀行，消费包的测试单元读提供包 tests/c/mod.rs（构建脚本也报了它的 rerun-if-changed）、读 OUT_DIR 里与提供包 src/x.rs 逐字节相同的 copied.rs，
                                       构建脚本另报 ../provider/src/k.wgsl：退 1、点名 mod.rs（「test:cross、构建脚本报的 rerun-if-changed；跨包源码」）与 copied.rs，k.wgsl 不点名、前缀行不报用不着
  stale-package-prefix-rows-red        放行表三行前缀行：crates/provider/（有 rerun-if-changed 落在里面）、crates/absent/（目录不在）、crates/bystander/（是一个包的目录、没有 rerun-if-changed 落在里面）：
                                       退 1、「两张表有 2 行写不对或用不着了」、点名后两行各自的原因，第一行不点名，k.wgsl 不在「两张表里都没登」里
  stale-and-lag-rows-red               放行表一行点名的引用不在、滞后表比基准多一行：退 1、两样都点名
  compile-failure-red                  假 cargo 退 101：退 1、点名编不过；再跑一趟两个包照现判（没入库）
  missing-dep-info-red                 一个编译单元没写 .d：退 1、点名找不到 dep-info
  target-directory-required            没设 CARGO_TARGET_DIR、设在根里：两趟都退 1、点名 CARGO_TARGET_DIR，一次都没起 cargo check
  sample-root-needs-temporary-service  样本根（.gate-cells 与 .crate-cross-references-sample/ 都在）而环境里没有临时服务的标记：退 2
  reuse-three-questions                三个包（alpha、beta 读 alpha 的 src/shared.rs、gamma），四趟：空库、同一份输入、改不属任何包也没人读的 docs/notes.md、
                                       改 crates/alpha/src/shared.rs；照三问判（affected 是 alpha、beta），打规范输出；另核假 cargo 每一趟只点名要现判的包
  reuse-sentinel                       记好三个包之后改录好的 .d（键外的输入）：强制现判那一包判出的事实与库里不同 ⇒ 退 1「判定不稳」，下一趟那一包现判
  key-independent-of-compile-directory 同一份包树放在两个根下、各配一份编译目录（OUT_DIR 前缀不同）与家目录（HOME 不同）录同一份编译输出（.d 读 OUT_DIR 里
                                       生成的 .rs，构建脚本报根里的绝对路径、OUT_DIR 里的文件与 rerun-if-env-changed=HOME、CARGO_HOME）：两份事实相同、构建脚本的输入是
                                       「crates/alpha/build.rs、OUT_DIR:alpha/gen.rs」，两把键相同
  reuse-across-hosts                   一台写库、另一台读库：两个根各带自己的编译目录与家目录，读库的根的 .git 指到写库的根的 git 目录（git common-dir 相同，
                                       两台挂同一份仓、读写同一份库）；假 cargo 不喂录好的 .d，照各自根里的源码现录（dep_info 写 scan 的单元）。写库的根空库现判 3 包、
                                       退 0；读库的根「现判 0、复用 2、强制现判 1」、退 0、没有判定不稳，强制现判的是 alpha（它的事实里有 OUT_DIR 里的文件与构建脚本报的 HOME），
                                       读库的根的假 cargo 只点名 alpha
  key-unchanged-by-toolchain-and-cargo-configuration
                                       同 reuse-across-hosts 那样的一棵树（alpha 的构建脚本另报 rerun-if-env-changed=CARGO_HOME、RUSTFLAGS、CARGO_PROFILE_DEV_DEBUG_ASSERTIONS），
                                       空库现判 3 包之后换工具链与 cargo 配置：RUSTC 指到另一份版本不同的假 rustc，<根> 加 rust-toolchain.toml、.cargo/config.toml 与 Cargo.lock，
                                       换一个家目录、里面放 .cargo/config.toml，CARGO_HOME 指到另一份带 config.toml 的目录，设 RUSTFLAGS 与 CARGO_PROFILE_DEV_DEBUG_ASSERTIONS：
                                       第二趟「现判 0、复用 2、强制现判 1」、退 0、没有一行报「键变了」、没有判定不稳，假 cargo 第二趟只点名 alpha
  host-triple-from-machine             host_triple 查表：Linux x86_64 与 aarch64（glibc）、Darwin arm64 与 x86_64 各得自己的三元组，Linux 不是 glibc 的与 Windows 得 None；
                                       host_for_run 对表里没有的机器记一条点名 HOST_TRIPLES 的红句，对跑自证的这台机器不记
  invalidated-key-names-changed-parts  三个包记好之后改 crates/alpha/src/shared.rs：alpha 那一行「现判（键变了：package-file；」，beta（读它）那一行「现判（键变了：read；」；
                                       空库那一趟没有一行报「键变了」
"""
import concurrent.futures
import datetime
import hashlib
import importlib.util
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

SCRIPT_RELATIVE = "scripts/gate/crate-cross-references-check.py"
REPOSITORY = os.path.realpath(os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", ".."))
BREAK_VARIABLE = "CRATE_CROSS_REFERENCES_BREAK"
BREAK_VARIABLES = (BREAK_VARIABLE,)
KIND = "crate-cross-references-check.py:packages"
VERDICT_DIRECTORY_NAME = "singlefs-unit-verdicts"
JUDGE_ENTRY = f"judge:{SCRIPT_RELATIVE}::recorded_package_facts"
ALLOWED_TABLE = ".claude/gate.d/crate-cross-references-allowed.tsv"
LAG_TABLE = ".claude/gate.d/crate-cross-references-lag.tsv"
SAMPLE_DIRECTORY = ".crate-cross-references-sample"
SAMPLE_TOOLS_DIRECTORY = ".crate-cross-references-sample-tools"
SAMPLE_CHECK_LOG_VARIABLE = "CRATE_CROSS_REFERENCES_SAMPLE_CHECK_LOG"
CHECK_ARGUMENTS = ("check", "--offline", "--message-format=json", "--all-targets", "--all-features")
# 这台机器的 host 三元组（文件头「判的包」）：（platform.system()、platform.machine()）→ rustc 报的 host；Linux 的几行是 glibc 的（要求 host_triple 的 glibc 为真）
HOST_TRIPLES = {("Linux", "x86_64"): "x86_64-unknown-linux-gnu", ("Linux", "aarch64"): "aarch64-unknown-linux-gnu",
                ("Darwin", "x86_64"): "x86_64-apple-darwin", ("Darwin", "arm64"): "aarch64-apple-darwin"}
ROOT_INPUTS = ("Cargo.toml",)
# 不进键的工具链与 cargo 配置（文件头「复用」不进键的那几样）：只给弄坏开关 toolchain-and-cargo-configuration-keyed 收回键里
UNKEYED_ROOT_FILES = ("Cargo.lock", "rust-toolchain", "rust-toolchain.toml", ".cargo/config", ".cargo/config.toml")
UNKEYED_CARGO_HOME_FILES = ("config", "config.toml")
DATA_SUFFIXES = (".toml", ".md", ".tsv")
SOURCE_SUFFIXES = (".rs", ".wgsl")
CARGO_SET_VARIABLE_PREFIX = "CARGO_"
BUILD_SCRIPT_KIND = "custom-build"
# 不进键的构建环境变量（文件头「复用」environment 那一行）：CARGO_ 起头的一律不进，另加这几个
BUILD_ENVIRONMENT_VARIABLES = ("HOME", "RUSTC", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "RUSTDOCFLAGS", "RUSTFLAGS", "RUSTUP_TOOLCHAIN")
GENERATED_PREFIX = "OUT_DIR:"
DEPENDENCY_PREFIX = "DEPENDENCY:"
REASON_MINIMUM = 8
ABSENT = "不在"
EXIT_GREEN, EXIT_RED, EXIT_USAGE, EXIT_UNAVAILABLE = 0, 1, 2, 75
EXIT_UNIT_KEY_REFUSED = 4
CATEGORY_NAMES = {
    "same": "同包", "generated": "构建脚本生成", "dependency": "依赖源码", "data": "别的包的数据文件",
    "cross": "跨包源码", "unpackaged": "仓里不属任何包的文件", "outside": "仓外不属任何包的文件",
    "copied": "构建脚本拷进 OUT_DIR 的别的包文件", "build-read": "构建脚本运行时读的别的包文件",
}
RED_CATEGORIES = ("cross", "unpackaged", "outside", "copied", "build-read")
ALLOWED_CATEGORIES = ("same", "generated", "dependency", "data")
# 数据读法（文件头「数据读法」）：只当数据读一份文件的两个宏；源码里把这两个名字改指别的宏的写法（认到就这个单元一份都不算只当数据读）
DATA_READ_MACROS = ("include_str", "include_bytes")
DATA_READ_MACRO_RENAMES = re.compile(r"\bas\s+include_(?:str|bytes)\b|\bmacro_rules!\s*include_(?:str|bytes)\b")
MANIFEST_DIRECTORY_VARIABLE = "CARGO_MANIFEST_DIR"
# 一个单元读的源码里有一份超过这么大的，不切词，这个单元一份都不算只当数据读（照源码判，宁可多红）
SCANNED_SOURCE_BYTES_LIMIT = 8 * 1024 * 1024
BUILD_INPUT_UNIT = "构建脚本报的 rerun-if-changed"
# 放行表前缀行（文件头「两张表」）：第二列以它收尾
PACKAGE_PREFIX_ENDING = "/"
ROW = re.compile(r"^(?P<package>[^\t#]+?)\t+(?P<file>[^\t#]+?)\t+#\s*(?P<reason>.*?)\s*$")
RULE_STEPS = ["怎么办：一个包的代码不读另一个包目录下的源码与不属任何包的文件（.claude/rules/directory-layout.md「crates/：各包放什么」）：要用另一个包的代码，",
              "        在 Cargo.toml 里依赖那个包，或在自己的包里写一份；读别的包的 .toml、.md、.tsv 数据用 include_str! / include_bytes! 的字面路径；",
              "        拆包时才改的存量登进 .claude/gate.d/crate-cross-references-lag.tsv（只缩不涨），",
              "        确要留着的登进 .claude/gate.d/crate-cross-references-allowed.tsv，一行「<包目录><TAB><引到的文件><TAB># 理由>」；",
              "        构建脚本为算判定的单元键读一整个别的包的，登一行前缀行「<包目录><TAB><别的包目录>/<TAB># 理由>」（只放行构建脚本报的 rerun-if-changed，编进编译单元的照旧逐文件登）。"]


def broken(item):
    return os.environ.get(BREAK_VARIABLE, "") == item


def today():
    return datetime.date.today().isoformat()


def sha256_of_file(path):
    try:
        with open(path, "rb") as handle:
            return hashlib.sha256(handle.read()).hexdigest()
    except OSError:
        return ABSENT


def sha256_of_text(text):
    return hashlib.sha256(text.encode("utf-8", "surrogateescape")).hexdigest()


def inside(path, directory):
    return path == directory or path.startswith(directory.rstrip(os.sep) + os.sep)


def load_module(name, relative):
    """仓里一份 python 库（这份脚本所在的仓），照名字只载入一次。"""
    if name not in sys.modules:
        specification = importlib.util.spec_from_file_location(name, os.path.join(REPOSITORY, relative))
        module = importlib.util.module_from_spec(specification)
        sys.modules[name] = module
        specification.loader.exec_module(module)
    return sys.modules[name]


def prune_client():
    return load_module("singlefs_prune_server_client", "scripts/prune-server/prune_server_client.py")


def reuse_checks():
    return load_module("singlefs_lib_reuse_checks", "scripts/gate/lib_reuse_checks.py")


# ── 读编译产物（判法闭包：recorded_package_facts 与它调到的几个函数，单元键里的 judge 就是它们） ──

def parse_dep_info(text):
    """rustc 写的 dep-info → ([读到的文件，照出现的次序、去重], [env-dep 变量名])。
    规则行「<产物>: <文件> <文件>…」里的文件照收（几条规则取并集、产物本身不算），空格转义成「\\ 」；「# env-dep:<名>[=<值>]」收变量名。
    rustc 另给每份读到的文件写一条没有依赖的空规则「<文件>:」（make 的 phony 写法，文件删了 make 不报错）：
    只有带依赖的规则的左边算产物，空规则的左边是读到的文件本身，不从读到的文件里剔掉。"""
    sources, seen, variables = [], set(), []
    targets = set()
    rules = []
    for line in text.split("\n"):
        if line.startswith("# env-dep:"):
            name = line[len("# env-dep:"):].split("=", 1)[0].strip()
            if name and name not in variables:
                variables.append(name)
            continue
        if not line.strip() or line.startswith("#"):
            continue
        words, word, index = [], "", 0
        while index < len(line):
            character = line[index]
            if character == "\\" and index + 1 < len(line) and line[index + 1] in " \\#:":
                word += line[index + 1]
                index += 2
                continue
            if character in " \t":
                if word:
                    words.append(word)
                word = ""
                index += 1
                continue
            word += character
            index += 1
        if word:
            words.append(word)
        if not words or not words[0].endswith(":"):
            continue
        if words[1:]:
            targets.add(words[0][:-1])
        rules.append(words[1:])
    for prerequisites in rules:
        for path in prerequisites:
            if path not in targets and path not in seen:
                seen.add(path)
                sources.append(path)
    return sources, variables


def dep_info_candidates(filename, kinds):
    """产物文件名与目标种类（compiler-artifact 的 target.kind）→ 它的 dep-info 可能叫什么。
    构建脚本（种类有 custom-build）：cargo 给的产物是 build/<包名>-<哈希>/build-script-build（rustc 真写的 build_script_build-<哈希> 的硬链接），
    rustc 的 .d 在同一目录、叫「产物名的连字符换成下划线-<那个目录名最后一段哈希>.d」；目录名没有「-<哈希>」的没有候选。
    别的单元：同一目录下去掉扩展名的名字 .d；lib 起头的另试去掉 lib 的那一个，排在前面。"""
    directory, name = os.path.split(filename)
    stem = name.split(".", 1)[0]
    if BUILD_SCRIPT_KIND in kinds and not broken("build-script-dep-info-by-artifact-name"):
        found = re.match(r"^.+-([0-9a-f]+)$", os.path.basename(directory))
        return [os.path.join(directory, f"{stem.replace('-', '_')}-{found.group(1)}.d")] if found else []
    names = ([stem[len("lib"):] + ".d"] if stem.startswith("lib") else []) + [stem + ".d"]
    return [os.path.join(directory, candidate) for candidate in names]


def build_script_declarations(text):
    """构建脚本的 output → ([rerun-if-changed 的路径], [rerun-if-env-changed 的变量名])；cargo: 与 cargo:: 两种前缀都认。"""
    paths, variables = [], []
    for line in text.split("\n"):
        found = re.match(r"^cargo::?(rerun-if-changed|rerun-if-env-changed)=(.*)$", line.strip())
        if not found:
            continue
        (paths if found.group(1) == "rerun-if-changed" else variables).append(found.group(2))
    return paths, variables


RUST_ESCAPES = {"n": "\n", "t": "\t", "r": "\r", "0": "\0", "\\": "\\", "'": "'", '"': '"'}
RUST_TOKEN = re.compile(r"""(?P<space>\s+)|(?P<line>//[^\n]*)|(?P<block>/\*)|(?P<raw>[bc]?r(?P<hashes>\#*)")|(?P<plain>[bc]?")"""
                        r"""|(?P<character>'(?:\\u\{[0-9A-Fa-f_]+\}|\\x[0-9A-Fa-f]{2}|\\.|[^\\'\n])')|(?P<ident>[^\W\d]\w*)|(?P<number>\d\w*)|(?P<punct>.)""", re.S)
RUST_PLAIN_STRING_BODY = re.compile(r'(?:[^"\\]|\\.)*', re.S)
RUST_ESCAPE = re.compile(r"\\(u\{[0-9A-Fa-f_]+\}|x[0-9A-Fa-f]{2}|\n\s*|.)", re.S)
RUST_BLOCK_COMMENT_EDGE = re.compile(r"/\*|\*/")


def rust_unescape(body):
    def replace(found):
        escaped = found.group(1)
        if escaped.startswith("u{"):
            return chr(int(escaped[2:-1].replace("_", ""), 16))
        if escaped.startswith("x") and len(escaped) == 3:
            return chr(int(escaped[1:], 16))
        if escaped.startswith("\n"):
            return ""
        return RUST_ESCAPES.get(escaped, escaped)
    return RUST_ESCAPE.sub(replace, body)


def rust_tokens(text):
    """Rust 源码 → [(种类, 值)]：种类是 ident、punct、string（值是字面量照转义取出的内容，原始字符串照原文）、number；
    注释与空白不出，字符字面量跳过，生命周期只出前面的 ' 与后面的名字。只为认 include_str! 一族与字符串字面量用，不当完整的词法分析。"""
    tokens, index, length = [], 0, len(text)
    while index < length:
        found = RUST_TOKEN.match(text, index)
        kind = found.lastgroup if found.lastgroup != "hashes" else "raw"
        index = found.end()
        if kind in ("space", "line", "character"):
            continue
        if kind == "block":
            depth = 1
            while depth:
                edge = RUST_BLOCK_COMMENT_EDGE.search(text, index)
                if edge is None:
                    index = length
                    break
                depth += 1 if edge.group(0) == "/*" else -1
                index = edge.end()
            continue
        if kind == "raw":
            terminator = '"' + found.group("hashes")
            end = text.find(terminator, index)
            end = length if end < 0 else end
            tokens.append(("string", text[index:end]))
            index = end + len(terminator)
            continue
        if kind == "plain":
            body = RUST_PLAIN_STRING_BODY.match(text, index)
            tokens.append(("string", rust_unescape(body.group(0))))
            index = body.end() + 1
            continue
        tokens.append((kind, found.group(0)))
    return tokens


def group_end(tokens, opener):
    """tokens[opener] 是 (、[、{ 之一 → 与它配对的那个闭括号的下标；配不上交 len(tokens)。"""
    depth = 0
    for index in range(opener, len(tokens)):
        kind, value = tokens[index]
        if kind != "punct":
            continue
        if value in "([{":
            depth += 1
        elif value in ")]}":
            depth -= 1
            if depth == 0:
                return index
    return len(tokens)


def macro_at(tokens, index):
    """tokens[index] 起是「名字 ! 开括号」→ (名字, 开括号的下标)；不是交 (None, None)。"""
    if (index + 2 < len(tokens) and tokens[index][0] == "ident" and tokens[index + 1] == ("punct", "!")
            and tokens[index + 2][0] == "punct" and tokens[index + 2][1] in "([{"):
        return tokens[index][1], index + 2
    return None, None


def string_expression(tokens, start, end):
    """tokens[start:end] 是一个字符串表达式（一个字符串字面量，或 concat! 套的字符串字面量与开头一个 env!("CARGO_MANIFEST_DIR")）
    → (从包目录起算吗, 拼出来的文本)；别的写法交 None。"""
    if end == start + 1 and tokens[start][0] == "string":
        return False, tokens[start][1]
    name, opener = macro_at(tokens, start)
    if name != "concat" or group_end(tokens, opener) != end - 1:
        return None
    parts, from_manifest, index = [], False, opener + 1
    while index < end - 1:
        piece_end = index
        while piece_end < end - 1 and tokens[piece_end] != ("punct", ","):
            inner, inner_opener = macro_at(tokens, piece_end)
            piece_end = group_end(tokens, inner_opener) + 1 if inner else piece_end + 1
        if piece_end > index:
            inner, inner_opener = macro_at(tokens, index)
            if inner == "env" and not parts and not from_manifest and tokens[inner_opener + 1:piece_end] == [("string", MANIFEST_DIRECTORY_VARIABLE), ("punct", ")")]:
                from_manifest = True
            else:
                nested = string_expression(tokens, index, piece_end)
                if nested is None or nested[0]:
                    return None
                parts.append(nested[1])
        index = piece_end + 1
    return from_manifest, "".join(parts)


def source_references(tokens):
    """一份源码的词 → [(是不是 include_str! / include_bytes! 的参数, 字符串表达式 (从包目录起算吗, 文本) 或 None)]。
    include_str! / include_bytes! 的参数整个算一项（认不出的写法交 None）；别处的 concat! 整个算一项，单个字符串字面量各算一项。"""
    found, index = [], 0
    while index < len(tokens):
        name, opener = macro_at(tokens, index)
        if name in DATA_READ_MACROS:
            closer = group_end(tokens, opener)
            found.append((True, string_expression(tokens, opener + 1, closer)))
            index = closer + 1
            continue
        if name == "concat":
            closer = group_end(tokens, opener)
            found.append((False, string_expression(tokens, index, closer + 1)))
            for kind, value in tokens[opener + 1:closer]:
                if kind == "string":
                    found.append((False, (False, value)))
            index = closer + 1
            continue
        if tokens[index][0] == "string":
            found.append((False, (False, tokens[index][1])))
        index += 1
    return found


def path_segments(text):
    """一段路径文本按 / 切开，去掉开头的空段、. 与 ..（只留能与目标对齐的尾巴）。"""
    segments = [segment for segment in text.replace("\\", "/").split("/")]
    while segments and segments[0] in ("", ".", ".."):
        segments.pop(0)
    return [segment for segment in segments if segment not in ("", ".")]


def data_read_paths(absolutes, own_directory, root, cache):
    """文件头「数据读法」：一个单元 .d 列的文件（绝对路径）→ 其中只当数据读的那几份（别的包目录下、后缀是数据后缀的才算候选）。
    cache：{绝对路径: 那份源码的 source_references，读不了的 None，过大的 too-large，改指了两个宏名的 renamed}，同一趟各单元共用。"""
    candidates = {path for path in absolutes if path.endswith(DATA_SUFFIXES) and not inside(path, own_directory)}
    if not candidates:
        return set()
    references = {}
    for path in absolutes:
        if path not in cache:
            try:
                if os.path.getsize(path) > SCANNED_SOURCE_BYTES_LIMIT:
                    cache[path] = "too-large"
                else:
                    with open(path, encoding="utf-8", errors="replace") as handle:
                        text = handle.read()
                    cache[path] = "renamed" if DATA_READ_MACRO_RENAMES.search(text) else source_references(rust_tokens(text))
            except OSError:
                cache[path] = None
        if cache[path] in ("too-large", "renamed"):
            return set()
        if cache[path] is not None:
            references[path] = cache[path]
    tails = {candidate: path_segments(os.path.relpath(candidate, root) if inside(candidate, root) else candidate) for candidate in candidates}
    names = {os.path.basename(candidate) for candidate in candidates}

    def resolved(path, expression, is_data_read):
        """数据读只照所在文件的目录解（include_str! 的规矩）；别的指法另照这个包的目录解一次（Path::new(env!("CARGO_MANIFEST_DIR")).join(…) 这类）。"""
        from_manifest, text = expression
        if from_manifest:
            return {os.path.realpath(own_directory + text if text.startswith("/") else os.path.join(own_directory, text))}
        if os.path.isabs(text):
            return {os.path.realpath(text)}
        beside = {os.path.realpath(os.path.join(os.path.dirname(path), text))}
        return beside if is_data_read else beside | {os.path.realpath(os.path.join(own_directory, text))}

    def pointed_at(expression, candidate):
        segments = path_segments(expression[1])
        tail = tails[candidate]
        return len(segments) >= 2 and (tail[-len(segments):] == segments or segments[-len(tail):] == tail)

    data_only = set()
    for _round in range(len(candidates) + 2):
        proven, pointed = set(), set()
        for path, items in references.items():
            if path in data_only:
                continue
            for is_data_read, expression in items:
                if expression is None or os.path.basename(expression[1].replace("\\", "/")) not in names:
                    continue
                targets = resolved(path, expression, is_data_read)
                for candidate in candidates:
                    if is_data_read:
                        if candidate in targets:
                            proven.add(candidate)
                    elif candidate in targets or pointed_at(expression, candidate):
                        pointed.add(candidate)
        settled = proven - pointed
        if settled == data_only:
            return settled
        data_only = settled
    return set()


def stored_path(absolute, root, out_directories, dependencies):
    """一份读到的文件在事实里怎么写：OUT_DIR 里的写 OUT_DIR:<包名>/<相对>，依赖源码写 DEPENDENCY:<包名>-<版本>/<相对>，<根> 里的写相对路径，别的写绝对路径。"""
    for directory, name in out_directories:
        if inside(absolute, directory):
            return f"{GENERATED_PREFIX}{name}/{os.path.relpath(absolute, directory)}"
    for directory, label in dependencies:
        if inside(absolute, directory):
            return f"{DEPENDENCY_PREFIX}{label}/{os.path.relpath(absolute, directory)}"
    if inside(absolute, root):
        return os.path.relpath(absolute, root)
    return absolute


def recorded_package_facts(messages_text, packages, root, wanted):
    """cargo check --message-format=json 的输出 → ({包目录: 事实}, {包目录: [找不到的 dep-info]})，只收 wanted 里的包（相对 <根> 的包目录）。
    packages：{package_id: {"name", "version", "directory"（解开链接的绝对路径）, "relative"（相对 <根>，不在 <根> 里是 None）, "source"}}。"""
    artifacts, executed = [], []
    for line in messages_text.split("\n"):
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("reason") == "compiler-artifact":
            artifacts.append(message)
        elif message.get("reason") == "build-script-executed":
            executed.append(message)
    out_directories = []
    for message in executed:
        information = packages.get(message.get("package_id"))
        if information and message.get("out_dir"):
            out_directories.append((os.path.realpath(message["out_dir"]), information["name"]))
    dependencies = [(information["directory"], f"{information['name']}-{information['version']}")
                    for information in packages.values() if information.get("source")]
    facts, missing, source_cache = {}, {}, {}
    for message in artifacts:
        information = packages.get(message.get("package_id"))
        if not information or information.get("relative") not in wanted:
            continue
        package = information["relative"]
        target = message.get("target") or {}
        unit = f"{'/'.join(target.get('kind') or ['?'])}:{target.get('name', '?')}"
        fact = facts.setdefault(package, {"units": [], "build": {"inputs": [], "environment": []}})
        if broken("dep-info-unread"):
            sources, variables = [target.get("src_path", "")], []
        else:
            dep_info = next((candidate for filename in message.get("filenames") or [] for candidate in dep_info_candidates(filename, target.get("kind") or [])
                             if os.path.isfile(candidate)), None)
            if dep_info is None:
                if not broken("missing-dep-info-accepted"):
                    missing.setdefault(package, []).append(f"{unit}（产物 {', '.join(message.get('filenames') or ['没有'])}）")
                continue
            with open(dep_info, encoding="utf-8", errors="surrogateescape") as handle:
                sources, variables = parse_dep_info(handle.read())
        written, absolutes, generated = [], [], {}
        for source in sources:
            absolute = os.path.realpath(source if os.path.isabs(source) else os.path.join(root, source))
            absolutes.append(absolute)
            stored = stored_path(absolute, root, out_directories, dependencies)
            written.append(stored)
            if stored.startswith(GENERATED_PREFIX):
                # OUT_DIR 拷贝（文件头）：编完这一刻 OUT_DIR 里的文件还在，记大小与 sha256，分类时与别的包目录里的文件比；内容去掉空白后为空的不比
                try:
                    with open(absolute, "rb") as handle:
                        content = handle.read()
                except OSError:
                    content = b""
                if content.strip():
                    generated[stored] = {"bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}
        data_reads = set() if broken("dep-info-unread") else data_read_paths(absolutes, information["directory"], root, source_cache)
        fact["units"].append({"unit": unit, "root": stored_path(os.path.realpath(target.get("src_path", "") or root), root, out_directories, dependencies),
                              "sources": sorted(set(written)),
                              "data_reads": sorted({stored_path(path, root, out_directories, dependencies) for path in data_reads}),
                              "generated": {name: generated[name] for name in sorted(generated)},
                              "environment": sorted({name for name in variables if not name.startswith(CARGO_SET_VARIABLE_PREFIX) and name != "OUT_DIR"})})
    for message in executed:
        information = packages.get(message.get("package_id"))
        if not information or information.get("relative") not in wanted or not message.get("out_dir"):
            continue
        fact = facts.setdefault(information["relative"], {"units": [], "build": {"inputs": [], "environment": []}})
        output = os.path.join(os.path.dirname(message["out_dir"]), "output")
        try:
            with open(output, encoding="utf-8", errors="surrogateescape") as handle:
                paths, variables = build_script_declarations(handle.read())
        except OSError:
            paths, variables = [], []
        for path in paths:
            absolute = os.path.realpath(path if os.path.isabs(path) else os.path.join(information["directory"], path))
            if broken("out-dir-inputs-absolute"):
                fact["build"]["inputs"].append(os.path.relpath(absolute, root) if inside(absolute, root) else absolute)
            else:
                fact["build"]["inputs"].append(stored_path(absolute, root, out_directories, dependencies))
        fact["build"]["environment"].extend(variables)
    for fact in facts.values():
        # 同名的单元（库的检查与库的测试检查）照整条内容排：cargo 打消息的次序随并行变，事实要与次序无关
        fact["units"].sort(key=lambda entry: json.dumps(entry, ensure_ascii=False, sort_keys=True))
        fact["build"]["inputs"] = sorted(set(fact["build"]["inputs"]))
        fact["build"]["environment"] = sorted(set(fact["build"]["environment"]))
    return facts, missing


def fact_digest(fact):
    return sha256_of_text(json.dumps(fact, ensure_ascii=False, sort_keys=True))


# ── 分类与两张表（每趟照事实现判，不入库） ──

class Layout:
    """这一趟的包布局：本地包目录（cargo metadata 里没有 source 的）、判的包、往上找 Cargo.toml 的缓存。"""

    def __init__(self, root, packages, judged):
        self.root = root
        self.local = sorted((information["directory"] for information in packages.values() if not information.get("source")), key=len, reverse=True)
        self.judged = set(judged)
        self.manifest_cache = {}
        self.sizes = None
        self.digests = {}

    def package_by_manifest(self, absolute):
        """往上找第一份带 [package] 的 Cargo.toml，交它所在的目录；没有交 None（<根> 里的找到 <根> 为止）。"""
        directory = os.path.dirname(absolute)
        visited = []
        found = None
        while True:
            if directory in self.manifest_cache:
                found = self.manifest_cache[directory]
                break
            visited.append(directory)
            manifest = os.path.join(directory, "Cargo.toml")
            if os.path.isfile(manifest):
                try:
                    with open(manifest, encoding="utf-8", errors="replace") as handle:
                        if re.search(r"(?m)^\s*\[package\]", handle.read()):
                            found = directory
                            break
                except OSError:
                    pass
            parent = os.path.dirname(directory)
            if parent == directory or (inside(absolute, self.root) and directory == self.root):
                break
            directory = parent
        for directory in visited:
            self.manifest_cache[directory] = found
        return found

    def owner(self, absolute):
        """一份文件（绝对路径）属于哪个包的目录（cargo metadata 里的本地包，或往上找到的带 [package] 的 Cargo.toml）；不属任何包交 None。"""
        return next((directory for directory in self.local if inside(absolute, directory)), None) or self.package_by_manifest(absolute)

    def copy_original(self, path, package, unit):
        """文件头「OUT_DIR 拷贝」：OUT_DIR 里这一份（事实里记了大小与 sha256 的）与别的包目录里哪一份逐字节相同 → 那一份相对 <根> 的路径；
        与读它的包自己目录里的一份相同、或谁都不同，交 None。<根> 里各本地包目录下的文件只在第一次问的时候列一遍大小，大小相同的才算 sha256。"""
        entry = ((unit or {}).get("generated") or {}).get(path)
        if not entry:
            return None
        if self.sizes is None:
            self.sizes = {}
            for directory in self.local:
                if not inside(directory, self.root):
                    continue
                for current, directories, names in os.walk(directory):
                    if "CACHEDIR.TAG" in names:
                        directories[:] = []
                        continue
                    directories[:] = sorted(name for name in directories if name != ".git")
                    for name in names:
                        member = os.path.join(current, name)
                        try:
                            self.sizes.setdefault(os.path.getsize(member), []).append(member)
                        except OSError:
                            continue
        matches = []
        for member in self.sizes.get(entry["bytes"], []):
            if member not in self.digests:
                self.digests[member] = sha256_of_file(member)
            if self.digests[member] == entry["sha256"]:
                matches.append(member)
        own = os.path.join(self.root, package)
        if any(inside(member, own) for member in matches):
            return None
        others = sorted(os.path.relpath(member, self.root) for member in matches if self.owner(member) not in (None, own))
        return others[0] if others else None

    def classify(self, path, package, unit=None):
        """事实里写下的一份文件 → 分类（文件头「判的包」那张表）。package 是读它的包（相对 <根>），unit 是读它的那个编译单元的事实
        （数据读法与 OUT_DIR 拷贝要它；不给的，数据后缀的别的包文件照源码判、OUT_DIR 里的照构建脚本生成）。"""
        if path.startswith(GENERATED_PREFIX):
            if broken("out-dir-not-allowed"):
                return "outside"
            if not broken("out-dir-copies-unchecked") and self.copy_original(path, package, unit):
                return "copied"
            return "generated"
        if path.startswith(DEPENDENCY_PREFIX):
            return "dependency"
        absolute = path if os.path.isabs(path) else os.path.join(self.root, path)
        own = os.path.join(self.root, package)
        if inside(absolute, own):
            return "same"
        owner = self.owner(absolute)
        if owner is not None and owner != own:
            if path.endswith(DATA_SUFFIXES) and (broken("data-suffix-by-name") or path in ((unit or {}).get("data_reads") or ())):
                return "data"
            return "cross"
        if broken("unpackaged-sources-allowed") or (broken("unpackaged-by-suffix") and not path.endswith(SOURCE_SUFFIXES)):
            return "data"
        return "unpackaged" if inside(absolute, self.root) else "outside"

    def classify_build_input(self, path, package):
        """构建脚本报的一处 rerun-if-changed（事实里的写法）→ 落在别的包目录里的分类（data 或 build-read）；别处的不判，交 None（文件头「判的包」）。"""
        if path.startswith((GENERATED_PREFIX, DEPENDENCY_PREFIX)):
            return None
        absolute = path if os.path.isabs(path) else os.path.join(self.root, path)
        own = os.path.join(self.root, package)
        if inside(absolute, own):
            return None
        owner = self.owner(absolute)
        if owner is None or owner == own:
            return None
        return "data" if path.endswith(DATA_SUFFIXES) else "build-read"

    def reference_note(self, path, package, unit, category):
        """判红那一处的补充说明（接在分类名后面）。"""
        if category == "copied":
            return f"，内容与 {self.copy_original(path, package, unit)} 相同"
        if category == "cross" and path.endswith(DATA_SUFFIXES):
            return "，后缀是数据后缀，但认不出这个单元只经 include_str! / include_bytes! 读它（判法文件头「数据读法」）"
        return ""


def table_rows(path, judged):
    """放行表、滞后表 → ([(行号, 包目录, 文件)], [写不对的那几句])；表不在当空表。"""
    rows, malformed = [], []
    if not os.path.isfile(path):
        return rows, malformed
    with open(path, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    for number, line in enumerate(lines, 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        found = ROW.match(line)
        if not found or len("".join(found.group("reason").split())) < REASON_MINIMUM:
            malformed.append(f"{os.path.basename(path)} 第 {number} 行要写成「<包目录><TAB><引到的文件><TAB># 理由」、理由至少 {REASON_MINIMUM} 个字：{line.strip()[:160]}")
            continue
        package, target = found.group("package").strip(), found.group("file").strip()
        if judged is not None and package not in judged and not broken("table-keys-by-source-file"):
            malformed.append(f"{os.path.basename(path)} 第 {number} 行第一列 {package} 不是这一趟判到的包目录（crates/ 下的工作区成员，相对仓根）")
            continue
        rows.append((number, package, target))
    return rows, malformed


def is_package_prefix(target):
    """放行表一行的第二列以 / 收尾，是前缀行（文件头「两张表」）：写一个别的包的目录，不写一份文件。"""
    return target.endswith(PACKAGE_PREFIX_ENDING)


def covered_by_package_prefix(key, category, prefix_rows):
    """前缀行放行这一处吗：key 是（读它的包，引到的文件），category 是它判红的分类。只放行构建脚本报的 rerun-if-changed（分类 build-read）
    而落在同一个包那几行里写的别的包目录下的；编进编译单元的（同一份文件构建脚本也报了的算在内）与拷进 OUT_DIR 的不放行。"""
    if broken("package-prefix-rows-ignored") or (category != "build-read" and not broken("package-prefix-allows-all-categories")):
        return False
    package, target = key
    return any(row_package == package and inside(target, prefix.rstrip(PACKAGE_PREFIX_ENDING)) for _number, row_package, prefix in prefix_rows)


def package_prefix_problem(layout, root, package, prefix, references):
    """一行前缀行为什么用不着（接在「<包目录> 的前缀行 <前缀> 」后面）；用得着交 None：指的目录要是一个包的目录（目录在，且以它为目录的是一个包，
    本地包或带 [package] 的 Cargo.toml 所在的目录），这一趟读它的包的构建脚本报的 rerun-if-changed 要有至少一处落在它里面。"""
    directory = prefix.rstrip(PACKAGE_PREFIX_ENDING)
    absolute = directory if os.path.isabs(directory) else os.path.join(root, directory)
    if not broken("package-prefix-directory-unchecked") and (not directory or layout.owner(os.path.join(absolute, "Cargo.toml")) != absolute):
        return "不是别的包的目录（目录不在，或这一趟找不到以它为目录的包）"
    if not broken("package-prefix-unused-accepted") and not any(
            category == "build-read" and row_package == package and inside(target, directory)
            for (row_package, target), (category, _units, _note) in references.items()):
        return f"这一趟没有一处{BUILD_INPUT_UNIT} 落在它里面"
    return None


def reference_keys(package, fact, layout):
    """这一包事实里判红那几类的引用 → {(键的第一列, 文件): (分类, [编译单元…], 补充说明)}；键的第一列是包目录（弄坏开关 table-keys-by-source-file 下
    编译单元读的那几份是单元的根文件）。构建脚本报的 rerun-if-changed 落在别的包目录里、判红的，编译单元那一格写「构建脚本报的 rerun-if-changed」。"""
    found = {}
    for unit in fact["units"]:
        for path in unit["sources"]:
            category = layout.classify(path, package, unit)
            if category in RED_CATEGORIES:
                first = unit["root"] if broken("table-keys-by-source-file") else package
                entry = found.setdefault((first, path), (category, [], layout.reference_note(path, package, unit, category)))
                entry[1].append(unit["unit"])
    if not broken("build-inputs-unclassified"):
        for path in fact["build"]["inputs"]:
            if layout.classify_build_input(path, package) == "build-read":
                found.setdefault((package, path), ("build-read", [], ""))[1].append(BUILD_INPUT_UNIT)
    return found


def category_counts(package, fact, layout):
    """这一包各编译单元读的文件按分类数（同一份文件几个单元都读算一份；有一个单元判红就算判红那一类）。"""
    chosen = {}
    for unit in fact["units"]:
        for path in unit["sources"]:
            category = layout.classify(path, package, unit)
            if path not in chosen or (category in RED_CATEGORIES and chosen[path] not in RED_CATEGORIES):
                chosen[path] = category
    counts = {name: 0 for name in CATEGORY_NAMES}
    for category in chosen.values():
        counts[category] += 1
    return counts


# ── 复用：单元键与判定库 ──

def package_files(root, package):
    """包目录下的每一份文件（相对 <根>，排好序）；跳过 .git 与带 CACHEDIR.TAG 的编译目录。"""
    found = []
    for directory, directories, names in os.walk(os.path.join(root, package)):
        if "CACHEDIR.TAG" in names:
            directories[:] = []
            continue
        directories[:] = sorted(name for name in directories if name != ".git")
        found.extend(os.path.relpath(os.path.join(directory, name), root) for name in names)
    return sorted(found)


def files_under(root, path):
    """rerun-if-changed 指的一处：文件交它自己，目录交底下每一份文件（相对 <根> 或绝对）。"""
    absolute = path if os.path.isabs(path) else os.path.join(root, path)
    if os.path.isdir(absolute):
        return sorted(os.path.join(path, os.path.relpath(os.path.join(directory, name), absolute))
                      for directory, _directories, names in os.walk(absolute) for name in names)
    return [path]


def build_environment_variable(name):
    """不进键的构建环境变量（文件头「复用」environment 那一行）：CARGO_ 起头的与 BUILD_ENVIRONMENT_VARIABLES 那几个。"""
    return name.startswith(CARGO_SET_VARIABLE_PREFIX) or name in BUILD_ENVIRONMENT_VARIABLES


def unkeyed_toolchain_and_configuration_lines(root, environment):
    """弄坏开关 toolchain-and-cargo-configuration-keyed 收回键里的那几样：RUSTC 的取值、<根> 的 UNKEYED_ROOT_FILES 与 cargo 家目录里那份配置的内容。"""
    home = environment.get("CARGO_HOME") or (os.path.join(environment["HOME"], ".cargo") if environment.get("HOME") else None)
    return ([f"toolchain-and-configuration RUSTC {environment.get('RUSTC', 'unset')}"]
            + [f"toolchain-and-configuration {name} {sha256_of_file(os.path.join(root, name))}" for name in UNKEYED_ROOT_FILES]
            + [f"toolchain-and-configuration cargo-home/{name} {sha256_of_file(os.path.join(home, name)) if home else ABSENT}" for name in UNKEYED_CARGO_HOME_FILES])


def package_key(root, package, fact, judge_key, environment):
    """这一包的单元键与各成分的 sha256（文件头「复用」）→ (键, {成分名: sha256})；成分名是每行的头一个词。
    fact 是上一条判定的事实（头一次没有交 None）。"""
    lines = [f"judge {judge_key}", f"command {' '.join(CHECK_ARGUMENTS)}"]
    lines += [f"manifest {name} {sha256_of_file(os.path.join(root, name))}" for name in ROOT_INPUTS]
    if not broken("package-files-unkeyed"):
        lines += [f"package-file {path} {sha256_of_file(os.path.join(root, path))}" for path in package_files(root, package)]
    variables = set()
    if fact is not None and not broken("previous-reads-unkeyed"):
        own = os.path.join(root, package)
        for path in sorted({path for unit in fact["units"] for path in unit["sources"]}):
            if path.startswith((GENERATED_PREFIX, DEPENDENCY_PREFIX)):
                continue
            if os.path.isabs(path):
                lines.append(f"read-outside {sha256_of_file(path)}")
            elif not inside(os.path.join(root, path), own):
                lines.append(f"read {path} {sha256_of_file(os.path.join(root, path))}")
        for path in fact["build"]["inputs"]:
            if path.startswith((GENERATED_PREFIX, DEPENDENCY_PREFIX)):
                lines.append(f"build-input-named {path}")
                continue
            for member in files_under(root, path):
                lines.append(f"build-input-outside {sha256_of_file(member)}" if os.path.isabs(member)
                             else f"build-input {member} {sha256_of_file(os.path.join(root, member))}")
        variables = {name for unit in fact["units"] for name in unit["environment"]} | set(fact["build"]["environment"])
    keyed = variables if broken("build-environment-variables-keyed") else {name for name in variables if not build_environment_variable(name)}
    lines += [f"environment {name} {environment.get(name, 'unset')}" for name in sorted(keyed)]
    if broken("toolchain-and-cargo-configuration-keyed"):
        lines += unkeyed_toolchain_and_configuration_lines(root, environment)
    if broken("target-directory-keyed"):
        lines.append(f"environment CARGO_TARGET_DIR {environment.get('CARGO_TARGET_DIR', 'unset')}")
    components = {}
    for line in lines:
        components.setdefault(line.split(" ", 1)[0], []).append(line)
    return (sha256_of_text("".join(line + "\n" for line in sorted(lines))),
            {name: sha256_of_text("".join(line + "\n" for line in sorted(group))) for name, group in components.items()})


def changed_components(recorded, current):
    """库里那一条记的各成分 sha256 与这一刻算的 → 不同的成分名（排好序；只在一边有的也算）。"""
    return sorted(name for name in set(recorded) | set(current) if recorded.get(name) != current.get(name))


def git_common_directory(root):
    completed = subprocess.run(["git", "-C", root, "rev-parse", "--git-common-dir"], capture_output=True, text=True)
    if completed.returncode != 0 or not completed.stdout.strip():
        return None
    return os.path.realpath(os.path.join(root, completed.stdout.strip()))


def item_path(package):
    return f"items/{sha256_of_text(package)}.json"


def key_path(key):
    return f"keys/{key}.json"


def decode_record(data):
    if data is None:
        return None
    try:
        record = json.loads(data.decode("utf-8"))
    except ValueError:
        return None
    return record if isinstance(record, dict) and isinstance(record.get("fact"), dict) and isinstance(record.get("components"), dict) else None


# ── check ──

class Run:
    """一趟 check 的现场：根、环境、打印与红句。"""

    def __init__(self, root, environment):
        self.root = root
        self.environment = environment
        self.problems = []

    def red(self, title, entries, steps):
        self.problems.append((title, entries, steps))

    def finish(self, success_lines):
        if self.problems:
            for title, entries, steps in self.problems:
                print(f"  ✗ {title}")  # gate-lint:summary
                for entry in entries:
                    print(f"      {entry}")  # gate-lint:detail
                for step in steps:
                    print(f"     → {step}")
            return EXIT_RED
        for line in success_lines:
            print(line)
        return EXIT_GREEN


def sample_root(root):
    return (os.path.realpath(root) != REPOSITORY and os.path.isfile(os.path.join(root, ".gate-cells"))
            and os.path.isdir(os.path.join(root, SAMPLE_DIRECTORY)) and not os.environ.get("GATE_STAGED_FROM"))


def target_directory_problem(target, root):
    if broken("target-directory-unchecked"):
        return None
    if not target:
        return "环境里没有 CARGO_TARGET_DIR（任务服务给的编译目录）"
    if not os.path.isabs(target):
        return f"CARGO_TARGET_DIR={target} 不是绝对路径"
    if inside(os.path.realpath(target), root):
        return f"CARGO_TARGET_DIR={target} 落在被判的根 {root} 里"
    return None


def run_text(command, cwd, environment):
    try:
        completed = subprocess.run(command, cwd=cwd, env=environment, capture_output=True, text=True, errors="surrogateescape")
    except OSError as error:
        return 127, "", str(error)
    return completed.returncode, completed.stdout, completed.stderr


def metadata_command(cargo, host):
    """cargo metadata 的整条命令（文件头「判的包」）：--filter-platform 只解析这一台 host 上编得到的依赖，
    不带它 cargo metadata 连别的平台才用的依赖也要取，离线缓存里没有就退 101（同 scripts/hosts/peer-host-lib.sh 第二台那一步的写法）。"""
    command = [cargo, "metadata", "--format-version", "1", "--offline"]
    if not broken("metadata-all-platforms"):
        command += ["--filter-platform", host]
    return command


def host_triple(system, machine, glibc):
    """这台机器的 host 三元组（文件头「判的包」）：（系统, 架构）查 HOST_TRIPLES，Linux 只认 glibc 的；表里没有的交 None。
    不起 rustc 与 cargo -V：判法闭包里有工具链询问，判法键就随 RUSTC 的版本变（文件头「复用」不进键的那几样）。"""
    triple = HOST_TRIPLES.get((system, machine))
    if broken("host-platform-unchecked"):
        return triple or f"{machine}-unknown-{system.lower()}-gnu"
    return triple if triple and (system != "Linux" or glibc) else None


def machine_facts():
    """host_triple 要的三样：系统、架构，与这台机器的 libc 是不是 glibc（os.confstr 的 CS_GNU_LIBC_VERSION 只有 glibc 答，别的 libc 与 macOS 不答）。"""
    try:
        glibc = (os.confstr("CS_GNU_LIBC_VERSION") or "").startswith("glibc")
    except (ValueError, OSError):
        glibc = False
    return platform.system(), platform.machine(), glibc


def host_for_run(run, system, machine, glibc):
    """host_triple 的结果；表里没有这台机器的记红交 None。"""
    host = host_triple(system, machine, glibc)
    if host is None:
        run.red(f"这台机器（系统 {system}、架构 {machine}、{'glibc' if glibc else '不是 glibc'}）不在 HOST_TRIPLES 里，cargo metadata 的 --filter-platform 没有值，这一格一个包都没判", [],
                ["怎么办：在 scripts/gate/crate-cross-references-check.py 的 HOST_TRIPLES 里照 rustc 在这台机器上报的 host 加一行（键是 platform.system() 与 platform.machine()），"
                 "并在自证的 host-triple-from-machine 格补一条。"])
    return host


def metadata_packages(run, cargo, host):
    """cargo metadata（不编）→ ({package_id: 信息}, [判的包目录]) 或 None（红句已记）。"""
    code, stdout, stderr = run_text(metadata_command(cargo, host), run.root, run.environment)
    try:
        metadata = json.loads(stdout) if code == 0 else None
    except ValueError:
        metadata = None
    if metadata is None:
        run.red(f"cargo metadata 退 {code}、读不出包的清单，这一格一个包都没判（命令：{' '.join(metadata_command('cargo', host)[1:])}）",
                [line for line in stderr.strip().split("\n")[-8:] if line],
                ["怎么办：照上面 cargo 的报错修好清单或 Cargo.lock（cargo metadata 不编）；它在 <根> 起，cargo 用的是 PATH 上那一份。"])
        return None
    packages = {}
    for package in metadata.get("packages", []):
        directory = os.path.realpath(os.path.dirname(package["manifest_path"]))
        packages[package["id"]] = {"name": package["name"], "version": package.get("version", ""), "directory": directory,
                                   "relative": os.path.relpath(directory, run.root) if inside(directory, run.root) else None,
                                   "source": package.get("source")}
    judged = sorted(packages[member]["relative"] for member in metadata.get("workspace_members", [])
                    if member in packages and (packages[member]["relative"] or "").startswith("crates/"))
    if not judged:
        run.red("cargo metadata 的工作区成员里一个包目录都不在 crates/ 下：这一格没有对象可判", [f"工作区成员 {len(metadata.get('workspace_members', []))} 个"],
                ["怎么办：看 <根> 的 Cargo.toml 的 [workspace] members；包搬了家的照新目录改这份判法与两张表。"])
        return None
    return packages, judged


def judge_key_of(run):
    """判法闭包的键（admission.py unit-key）：交 (键或 None, 拒算的说明)；admission.py 出了别的错记红交 (None, None)。
    这份文件里不写 rustc、cargo 的版本询问（命令里除程序外只有 -V、--version、-vV 一个词的）：admission.py 把这样的询问收成工具链单元进键，键就随 RUSTC 变。"""
    code, stdout, stderr = run_text([sys.executable, os.path.join(REPOSITORY, "scripts", "gate", "admission.py"), "unit-key", REPOSITORY, "-", JUDGE_ENTRY],
                                    REPOSITORY, run.environment)
    first = stdout.split("\n", 1)[0]
    if code == 0 and first.startswith("key\t"):
        key = first.split("\t", 1)[1].strip()
        if broken("judge-key-toolchain-keyed"):
            key = sha256_of_text(f"{key}\ntoolchain RUSTC {run.environment.get('RUSTC', 'unset')}")
        return key, ""
    if code == EXIT_UNIT_KEY_REFUSED:
        return None, (stdout.strip().split("\n") or ["拒算"])[0][:200]
    run.red(f"scripts/gate/admission.py unit-key 退 {code}，算不出判法闭包的键", [line for line in (stdout + stderr).strip().split("\n")[-6:] if line],
            [f"怎么办：单跑 python3 scripts/gate/admission.py unit-key {REPOSITORY} - {JUDGE_ENTRY} 看它的报错，修好再跑这一格。"])
    return None, None


def write_sample_tools(root):
    directory = os.path.join(root, SAMPLE_TOOLS_DIRECTORY)
    os.makedirs(directory, exist_ok=True)
    for name, text in (("cargo", FAKE_CARGO), ("rustc", FAKE_RUSTC)):
        path = os.path.join(directory, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.chmod(path, 0o755)
    return directory


def run_check(root, lag_state, lag_file, lag_name):
    root = os.path.realpath(root)
    environment = dict(os.environ)
    run = Run(root, environment)
    sample = sample_root(root)
    client = prune_client()
    own_target = None
    if sample:
        problem = client.selftest_service_problem()
        if problem:
            print(f"  ✗ {root} 是门禁样本根，样本不写真服务：{problem}")
            print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}；门禁样本档写成 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- "
                  "bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item checker-independence-and-sync.sh")
            return EXIT_USAGE
        tools = write_sample_tools(root)
        environment["PATH"] = tools + os.pathsep + environment.get("PATH", "")
        own_target = tempfile.mkdtemp(prefix="crate-cross-references-sample-target-")
        environment["CARGO_TARGET_DIR"] = own_target
    try:
        return judge(run, sample, lag_state, lag_file, lag_name)
    finally:
        if own_target:
            shutil.rmtree(own_target, ignore_errors=True)


def judge(run, sample, lag_state, lag_file, lag_name):
    root, environment = run.root, run.environment
    target = environment.get("CARGO_TARGET_DIR", "")
    problem = target_directory_problem(target, root)
    if problem:
        run.red(f"{problem}：这一格要起 cargo check，编译目录只由任务服务给（不往仓根 target 编），一个包都没编", [],
                ["怎么办：经任务服务交（门禁文件头的会编声明 # task-server-compiles: yes 让服务给这一次尝试一份编译目录）："
                 "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 checker-independence-and-sync.sh 的 crate-cross-references 格; 范围=1 格; "
                 "为什么=照 cargo check 写的 dep-info 判 crates/ 下各包有没有读别的包目录下的源码' "
                 "task-server/task submit -- bash .claude/gate.d/checker-independence-and-sync.sh --check crate-cross-references"])
        return run.finish([])
    cargo = shutil.which("cargo", path=environment.get("PATH", ""))
    if cargo is None:
        run.red("PATH 上没有 cargo，这一格一个包都没判", [], ["怎么办：在装了 Rust 工具链的机器上经任务服务交这一格。"])
        return run.finish([])
    common = git_common_directory(root)
    if common is None:
        run.red(f"{root} 不是 git 工作树（取不到 git common-dir），判定库没有名字，一个包都没判", [],
                ["怎么办：在 git 工作树里跑；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）。"])
        return run.finish([])
    host = host_for_run(run, *machine_facts())
    listed = metadata_packages(run, cargo, host) if host else None
    judge_key, refusal = judge_key_of(run) if listed else (None, None)
    if listed is None or (judge_key is None and refusal is None):
        return run.finish([])
    packages, judged = listed
    client = prune_client()
    store = client.library_store(client.FAMILY_UNIT_VERDICTS, KIND, os.path.join(common, VERDICT_DIRECTORY_NAME, KIND))
    previous_data = dict(zip(judged, store.read_many([item_path(package) for package in judged])))
    previous = {package: decode_record(data) for package, data in previous_data.items()}
    computed = {package: package_key(root, package, (previous[package] or {}).get("fact"), judge_key, environment) for package in judged}
    keys = {package: computed[package][0] for package in judged}
    stored = {}
    if judge_key is not None:
        for package, data in zip(judged, store.read_many([key_path(keys[package]) for package in judged])):
            record = decode_record(data)
            if record and record.get("verdict") == "green" and record.get("key") == keys[package]:
                stored[package] = record
    sentinel = reuse_checks().ReuseSentinel(SCRIPT_RELATIVE, "crate-cross-references", environment=environment)
    if sample:
        sentinel.plan({}, reuse=False, off_reason="样本档：样本钉死了现判与复用的个数")
    elif judge_key is None or broken("sentinel-skipped"):
        sentinel.plan({}, reuse=False, off_reason="判法闭包拒算、这一趟不读判定库" if judge_key is None else "弄坏开关 sentinel-skipped")
    else:
        sentinel.plan({package: fact_digest(record["fact"]) for package, record in stored.items()})
    states = {}
    for package in judged:
        if package in stored and not sentinel.forced(package):
            states[package] = "复用"
        else:
            states[package] = "强制现判" if package in stored else "现判"
    fresh = [package for package in judged if states[package] != "复用"]
    reasons = {}
    for package in fresh:
        if states[package] != "现判" or judge_key is None or previous_data[package] is None or broken("changed-components-unreported"):
            continue
        if previous[package] is None:
            reasons[package] = "库里那一条认不得，照头一次现判"
            continue
        changed = changed_components(previous[package]["components"], computed[package][1])
        reasons[package] = f"键变了：{'、'.join(changed)}" if changed else "键与库里那一条相同，这把键在库里没有绿判定"
    facts = {package: stored[package]["fact"] for package in judged if states[package] == "复用"}
    failed = {}
    if fresh:
        failed = compile_packages(run, cargo, packages, fresh, facts)
    written = []
    for package in fresh:
        if package not in facts:
            continue
        if states[package] == "强制现判":
            sentinel.observe(package, reuse_checks().FORCED, fact_digest(facts[package]))
            if fact_digest(facts[package]) != fact_digest(stored[package]["fact"]):
                store.delete_prefix(key_path(keys[package]))
                continue
        if judge_key is None:
            continue
        new_key, new_components = package_key(root, package, facts[package], judge_key, environment)
        record = {"verdict": "green", "item": package, "key": new_key, "components": new_components, "judged": today(), "how": " ".join(CHECK_ARGUMENTS),
                  "fact": facts[package]}
        data = json.dumps(record, ensure_ascii=False, sort_keys=True)
        written.append((key_path(new_key), data))
        written.append((item_path(package), data))
    if written:
        store.write_many(written)
    for package in judged:
        if states[package] == "复用":
            sentinel.observe(package, reuse_checks().REUSED, fact_digest(facts[package]))
    for title, step in sentinel.finish():
        run.red(title, [], [f"怎么办：{step}"])
    return report(run, packages, judged, facts, failed, states, reasons, refusal, lag_state, lag_file, lag_name)


def compile_packages(run, cargo, packages, fresh, facts):
    """cargo check 这一趟要现判的包，把事实填进 facts；交 {包目录: 为什么没有事实}。"""
    names = {information["relative"]: information["name"] for information in packages.values() if information.get("relative")}
    failed = {}
    code, stdout, stderr = run_text([sys.executable, os.path.join(REPOSITORY, "scripts", "hosts", "rocksdb-prebuilt.py"), "entry", "--root", run.root, "--cargo", cargo],
                                    run.root, run.environment)
    if code != 0:
        run.red(f"起 cargo 之前 RocksDB 预编库的取值没判过（rocksdb-prebuilt.py entry 退 {code}），一个包都没编", [line for line in (stdout + stderr).strip().split("\n")[-6:] if line],
                ["怎么办：照上面 rocksdb-prebuilt.py entry 的出路修（开新会话，或命令前 eval \"$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再重跑这一格。"])
        return {package: "RocksDB 预编库的取值没判过" for package in fresh}
    command = [cargo, *CHECK_ARGUMENTS] + [word for package in fresh for word in ("-p", names[package])]
    code, stdout, stderr = run_text(command, run.root, run.environment)
    finished = []
    for line in stdout.split("\n"):
        if "build-finished" not in line:
            continue
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if isinstance(message, dict) and message.get("reason") == "build-finished":
            finished.append(message)
    succeeded = code == 0 and bool(finished) and all(message.get("success") for message in finished)
    if not succeeded and not broken("compile-failure-accepted"):
        run.red(f"cargo check 退 {code}、编不过（{len(fresh)} 个包：{'、'.join(fresh)}），这几个包这一趟没有事实、不入库",
                [line for line in stderr.strip().split("\n")[-12:] if line],
                [f"怎么办：照上面的编译错误修；单跑 cargo {' '.join(CHECK_ARGUMENTS)} -p <包> 看全量报错（经任务服务交）。"])
        return {package: "编不过" for package in fresh}
    collected, missing = recorded_package_facts(stdout, packages, run.root, set(fresh))
    for package in fresh:
        if package in missing:
            failed[package] = "dep-info 找不到"
            run.red(f"{package} 有 {len(missing[package])} 个编译单元找不到 dep-info，读了哪些文件判不了", missing[package],
                    ["怎么办：看编译目录（CARGO_TARGET_DIR）里那几个产物旁边有没有 .d；cargo check 一定写 dep-info，没有的多半是编译目录被别的进程清过，换一份重跑。"])
        elif package not in collected:
            failed[package] = "编译输出里没有它的编译单元"
            run.red(f"cargo check 点名了 {package}，输出里却没有它的编译单元", [f"cargo 的 -p 是 {names[package]}"],
                    ["怎么办：看 cargo metadata 里这个包的名字与目录；照它改了再跑。"])
        else:
            facts[package] = collected[package]
    return failed


def report(run, packages, judged, facts, failed, states, reasons, refusal, lag_state, lag_file, lag_name):
    """reasons：{包目录: 现判那一包为什么现判（与库里那一条比）}，没有的不报。"""
    layout = Layout(run.root, packages, judged)
    references, totals = {}, {name: 0 for name in CATEGORY_NAMES}
    for package in judged:
        reason = f"{reasons[package]}；" if package in reasons else ""
        if package not in facts:
            print(f"  · {package}：红，{states[package]}（{reason}{failed.get(package, '没有事实')}）")  # gate-lint:detail
            continue
        found = reference_keys(package, facts[package], layout)
        references.update(found)
        counts = category_counts(package, facts[package], layout)
        for name, count in counts.items():
            totals[name] += count
        shown = "、".join(f"{CATEGORY_NAMES[name]} {counts[name]}" for name in ALLOWED_CATEGORIES)
        print(f"  · {package}：{'红' if found else '绿'}，{states[package]}（{reason}编译单元 {len(facts[package]['units'])} 个；读的文件：{shown}；"
              f"判红那几类 {len(found)} 处）")  # gate-lint:detail
    judged_keys = None if broken("table-keys-by-source-file") else set(judged)
    allowed_rows, allowed_malformed = table_rows(os.path.join(run.root, ALLOWED_TABLE), judged_keys)
    lag_rows, lag_malformed = table_rows(os.path.join(run.root, LAG_TABLE), judged_keys)
    prefix_rows = [row for row in allowed_rows if is_package_prefix(row[2])]
    file_rows = [row for row in allowed_rows if not is_package_prefix(row[2])]
    listed = {(package, target) for _number, package, target in file_rows + lag_rows}
    unlisted = sorted(key for key in references if key not in listed and not covered_by_package_prefix(key, references[key][0], prefix_rows))
    if unlisted:
        run.red(f"crates/ 下有 {len(unlisted)} 处编译单元读了别的包目录下的源码或不属任何包的文件，两张表里都没登：",
                [f"{package}  引 {target}（{'、'.join(references[(package, target)][1])}；{CATEGORY_NAMES[references[(package, target)][0]]}{references[(package, target)][2]}）"
                 for package, target in unlisted], RULE_STEPS)
    table_problems = allowed_malformed + lag_malformed
    without_facts = {package for package in judged if package not in facts}
    if not broken("stale-rows-accepted"):
        for table, rows in ((ALLOWED_TABLE, file_rows), (LAG_TABLE, lag_rows)):
            for number, package, target in rows:
                if package in without_facts:
                    continue
                if (package, target) not in references:
                    table_problems.append(f"{table} 第 {number} 行：{package} 的编译单元这一趟没有读 {target}，删掉这一行")
    for number, package, prefix in prefix_rows:
        problem = None if package in without_facts else package_prefix_problem(layout, run.root, package, prefix, references)
        if problem:
            table_problems.append(f"{ALLOWED_TABLE} 第 {number} 行：{package} 的前缀行 {prefix} {problem}，删掉这一行")
    if table_problems:
        run.red(f"两张表有 {len(table_problems)} 行写不对或用不着了：", table_problems,
                ["怎么办：照点名的行改：一行「<包目录><TAB><引到的文件><TAB># 理由」，路径相对仓根；引用改掉了的，同一次改动删掉那一行。"])
    growth_note = lag_growth(run, lag_rows, lag_state, lag_file, lag_name, judged_keys)
    judged_now = sum(1 for package in judged if states[package] == "现判" and package in facts)
    forced = sum(1 for package in judged if states[package] == "强制现判")
    reused = sum(1 for package in judged if states[package] == "复用")
    units = sum(len(fact["units"]) for fact in facts.values())
    refused_note = f"、拒算 {len(judged)}（判法闭包算不出键：{refusal}，一条都没入库）" if refusal else "、拒算 0"
    others = "、".join(f"{CATEGORY_NAMES[name]} {totals[name]}" for name in ALLOWED_CATEGORIES if name != "same")
    return run.finish([
        f"  ✓ crates/ 下 {len(judged)} 个包的 {units} 个编译单元（cargo {' '.join(CHECK_ARGUMENTS[:1] + CHECK_ARGUMENTS[3:])} 写的 dep-info）里，"
        f"跨包源码与不属任何包的文件 {len(references)} 处都登在放行表（{len(allowed_rows)} 行）或滞后表（{len(lag_rows)} 行）里，{growth_note}；"
        f"同包 {totals['same']} 份、{others} 份放行；单元键：现判 {judged_now}、复用 {reused}、强制现判 {forced}{refused_note}",
        "    这一格判不了的：特性组合之外被 cfg 掉的文件（没编进来）、运行时读而构建脚本没报 rerun-if-changed 的文件、过程宏自己读的文件、"
        "拷进 OUT_DIR 时改过内容的别的包文件（判法文件头「判不到的」）",
    ])


def lag_growth(run, lag_rows, lag_state, lag_file, lag_name, judged_keys):
    if lag_state == "table":
        base_rows, _malformed = table_rows(lag_file, None)
        base = {(package, target) for _number, package, target in base_rows}
        grown = [(number, package, target) for number, package, target in lag_rows if (package, target) not in base]
        if grown and not broken("lag-growth-accepted"):
            run.red(f"{LAG_TABLE} 比基准 {lag_name[:12]} 那一版多出 {len(grown)} 行（只缩不涨）：", [f"第 {number} 行 {package}  引 {target}" for number, package, target in grown],
                    ["怎么办：新加的跨包引用不进滞后表，照上面的出路改掉；确要编进别的包源码的登进放行表 .claude/gate.d/crate-cross-references-allowed.tsv。"])
        return f"滞后表与基准 {lag_name[:12]} 那一版比没涨"
    if lag_state.startswith("unreadable:"):
        if not broken("lag-growth-accepted"):
            run.red(f"取不到基准那一版的 {LAG_TABLE}，只缩不涨没法比：", [lag_state[len("unreadable:"):][:200]],
                    ["怎么办：按上面 git 的报错修好仓库状态（基准名在、对象库里有那一份）再跑这一格。"])
        return "基准取不到"
    return "基准里还没有这张表或不在 git 仓里，只缩不涨这一条没比"


# ── 样本与自证用的假工具 ──

FAKE_CARGO = r'''#!/usr/bin/env python3
# rocksdb-prebuilt-entry: selftest-fake-cargo
# 门禁 checker-independence-and-sync 的 crate-cross-references 格的样本与它的判法自证用的假 cargo（scripts/gate/crate-cross-references-check.py 文件头「样本」）：
# 照 .crate-cross-references-sample/scenario.json 答 -V、metadata 与 check，不编。
import json, os, re, sys
root = os.getcwd()
sample = os.path.join(root, ".crate-cross-references-sample")
arguments = sys.argv[1:]
if arguments[:1] in (["-V"], ["--version"]):
    print("cargo 1.0.0-crate-cross-references-sample")
    sys.exit(0)
with open(os.path.join(sample, "scenario.json"), encoding="utf-8") as handle:
    scenario = json.load(handle)
target = os.environ.get("CARGO_TARGET_DIR", os.path.join(root, "target"))
def out_directory(build):
    return os.path.join(target, "debug", "build", build["directory_name"], "out")
def substitute(text):
    text = text.replace("@ROOT@", root).replace("@TARGET@", target)
    for build in scenario.get("build_scripts", []):
        text = text.replace("@OUT_DIR:" + build["package"] + "@", out_directory(build))
    return text
def directory_of(package):
    directory = substitute(package["directory"])
    return directory if os.path.isabs(directory) else os.path.join(root, directory)
packages = {package["name"]: package for package in scenario["packages"]}
def package_id(package):
    return package["name"] + " " + package.get("version", "0.1.0") + " (" + (package.get("source") or "path+file://" + directory_of(package)) + ")"
if arguments[:1] == ["metadata"]:
    platform = arguments.index("--filter-platform") + 1 if "--filter-platform" in arguments else 0
    if "--offline" not in arguments or not platform or platform >= len(arguments):
        # 照真 cargo 在本仓工作区上的样子：离线缓存里只有这一台 host 编得到的依赖，不带 --filter-platform 就要取别的平台才用的那几个
        print("error: failed to download `sample-other-platform-dependency v1.0.0`\n\nCaused by:\n  attempting to make an HTTP request, but --offline was specified"
              if "--offline" in arguments else "error: 假 cargo 不连网，metadata 要带 --offline", file=sys.stderr)
        sys.exit(101)
    if not re.fullmatch(r"[a-z0-9_]+-(unknown|apple|pc)-[a-z0-9_]+(-[a-z0-9_]+)?", arguments[platform]):
        # 照真 cargo：--filter-platform 的值要是 rustc 认得的目标三元组，不是的 cargo 问 rustc 时退 101
        print("error: failed to run `rustc` to learn about target-specific information\n\nCaused by:\n  could not find specification for target `" + arguments[platform] + "`", file=sys.stderr)
        sys.exit(101)
    print(json.dumps({"packages": [{"name": package["name"], "version": package.get("version", "0.1.0"), "id": package_id(package), "source": package.get("source"),
                                    "manifest_path": os.path.join(directory_of(package), "Cargo.toml"), "targets": []} for package in scenario["packages"]],
                      "workspace_members": [package_id(packages[name]) for name in scenario["workspace_members"]], "workspace_root": root, "target_directory": target}))
    sys.exit(0)
if arguments[:1] != ["check"]:
    print("假 cargo 只答 -V、metadata 与 check：" + " ".join(arguments), file=sys.stderr)
    sys.exit(2)
selected = [arguments[index + 1] for index, word in enumerate(arguments) if word == "-p" and index + 1 < len(arguments)]
log = os.environ.get("CRATE_CROSS_REFERENCES_SAMPLE_CHECK_LOG")
if log:
    with open(log, "a", encoding="utf-8") as handle:
        handle.write(" ".join(sorted(selected)) + "\n")
for build in scenario.get("build_scripts", []):
    if selected and build["package"] not in selected:
        continue
    directory = out_directory(build)
    os.makedirs(directory, exist_ok=True)
    for name, text in build.get("generated", {}).items():
        with open(os.path.join(directory, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    with open(os.path.join(os.path.dirname(directory), "output"), "w", encoding="utf-8") as handle:
        handle.write("".join(substitute(line) + "\n" for line in build.get("output", [])))
    print(json.dumps({"reason": "build-script-executed", "package_id": package_id(packages[build["package"]]), "linked_libs": [], "linked_paths": [],
                      "cfgs": [], "env": [], "out_dir": directory}))
deps = os.path.join(target, "debug", "deps")
os.makedirs(deps, exist_ok=True)
# dep_info 写 scan 的单元：照根里的源码现录 .d（文件头「样本」），从 src_path 起跟字面写的 #[path] mod、mod <名>;、include! 一族与 OUT_DIR 里的 include!
MODULE_REFERENCE = re.compile(r'#\[path\s*=\s*"([^"]+)"\]\s*(?:pub\s+)?mod\s+\w+\s*;|(?:pub\s+)?mod\s+(\w+)\s*;'
                              r'|(include(?:_str|_bytes)?)!\(\s*"([^"]+)"\s*\)|include!\(\s*concat!\(\s*env!\(\s*"OUT_DIR"\s*\)\s*,\s*"/([^"]+)"\s*\)\s*\)')
def scanned_dep_info(unit, dep_info_path, artifact_path):
    out = next((out_directory(build) for build in scenario.get("build_scripts", []) if build["package"] == unit["package"]), None)
    start = os.path.realpath(os.path.join(root, unit["src_path"]))
    listed, pending, uses_out_dir = [], [start], False
    while pending:
        path = pending.pop(0)
        if path in listed:
            continue
        listed.append(path)
        try:
            with open(path, encoding="utf-8") as handle:
                text = handle.read()
        except OSError:
            continue
        directory = os.path.dirname(path)
        stem = os.path.splitext(os.path.basename(path))[0]
        module_base = directory if path == start or stem in ("lib", "main", "mod") else os.path.join(directory, stem)
        for found in MODULE_REFERENCE.finditer(text):
            if found.group(1):
                pending.append(os.path.realpath(os.path.join(directory, found.group(1))))
            elif found.group(2):
                for candidate in (os.path.join(module_base, found.group(2) + ".rs"), os.path.join(module_base, found.group(2), "mod.rs")):
                    if os.path.isfile(candidate):
                        pending.append(os.path.realpath(candidate))
                        break
            elif found.group(3):
                read = os.path.realpath(os.path.join(directory, found.group(4)))
                if found.group(3) == "include":
                    pending.append(read)
                elif read not in listed:
                    listed.append(read)
            elif out:
                uses_out_dir = True
                pending.append(os.path.join(out, found.group(5)))
    words = " ".join(path.replace(" ", "\\ ") for path in listed)
    return (f"{artifact_path}: {words}\n\n{dep_info_path}: {words}\n\n" + "".join(path.replace(" ", "\\ ") + ":\n" for path in listed)
            + (f"\n# env-dep:OUT_DIR={out}\n" if uses_out_dir else ""))
for unit in scenario.get("units", []):
    if selected and unit["package"] not in selected:
        continue
    # 构建脚本单元（给了 build_directory）照真 cargo：rustc 的产物与 .d 写在 build/<build_directory>/ 下、名字是 deps_name，
    # compiler-artifact 给的是同一目录里照目标名起的那一份（真 cargo 是硬链接）
    unit_directory = os.path.join(target, "debug", "build", unit["build_directory"]) if unit.get("build_directory") else deps
    os.makedirs(unit_directory, exist_ok=True)
    if unit.get("dep_info") == "scan":
        text = scanned_dep_info(unit, os.path.join(unit_directory, unit["deps_name"] + ".d"),
                                os.path.join(deps, "lib" + unit["deps_name"] + ".rmeta") if not unit.get("build_directory") else os.path.join(unit_directory, unit["deps_name"]))
        with open(os.path.join(unit_directory, unit["deps_name"] + ".d"), "w", encoding="utf-8") as handle:
            handle.write(text)
    elif unit.get("dep_info"):
        with open(os.path.join(sample, unit["dep_info"]), encoding="utf-8") as handle:
            text = substitute(handle.read())
        with open(os.path.join(unit_directory, unit["deps_name"] + ".d"), "w", encoding="utf-8") as handle:
            handle.write(text)
    if unit.get("build_directory"):
        artifact = os.path.join(unit_directory, unit["name"])
        for name in (unit["deps_name"], unit["name"]):
            with open(os.path.join(unit_directory, name), "w", encoding="utf-8") as handle:
                handle.write("")
    else:
        artifact = os.path.join(deps, "lib" + unit["deps_name"] + ".rmeta")
    print(json.dumps({"reason": "compiler-artifact", "package_id": package_id(packages[unit["package"]]),
                      "target": {"kind": unit["kind"], "crate_types": ["bin"] if unit.get("build_directory") else unit["kind"], "name": unit["name"],
                                 "src_path": os.path.join(root, unit["src_path"])},
                      "filenames": [artifact], "fresh": False}))
code = int(scenario.get("exit_code", 0))
if code:
    print("\n".join(substitute(line) for line in scenario.get("stderr") or ["error: 样本让 cargo check 编不过"]), file=sys.stderr)
print(json.dumps({"reason": "build-finished", "success": code == 0}))
sys.exit(code)
'''

FAKE_RUSTC = r'''#!/usr/bin/env bash
# crate-cross-references 格样本与自证用的假 rustc：只答版本
case "${1:-}" in
  -V|--version) echo "rustc 1.0.0-crate-cross-references-sample" ;;
  -vV) printf 'rustc 1.0.0-crate-cross-references-sample\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\n' ;;
  *) echo "假 rustc 只答 -V 与 -vV" >&2; exit 2 ;;
esac
'''


# ── 自证 ──

PROVIDER_CROSS = "crates/provider/tests/c/mod.rs"
PROVIDER_PREFIX = "crates/provider/"
SAMPLE_SCANNED_DEP_INFO = "scan"
REASON = "# 自证里登的一行，理由凑够八个字"


def write_file(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


class SampleTree:
    """自证的一棵小树：包、编译单元与录好的 .d 写进 .crate-cross-references-sample/，git init，假工具在 .crate-cross-references-sample-tools/。"""

    def __init__(self, work, name, packages, units, build_scripts=(), exit_code=0, files=None, allowed=(), lag=()):
        self.root = os.path.realpath(os.path.join(work, name))
        self.target = os.path.realpath(os.path.join(work, name + "-target"))
        self.log = os.path.join(work, name + "-check.log")
        self.scenario = {"packages": [{"name": package, "directory": f"crates/{package}"} for package in packages] + [
                             {"name": "sample-dependency", "version": "1.0.0", "directory": f"@ROOT@/{SAMPLE_DIRECTORY}/registry/sample-dependency-1.0.0",
                              "source": "registry+https://github.com/rust-lang/crates.io-index"}],
                         "workspace_members": list(packages), "units": [], "build_scripts": list(build_scripts), "exit_code": exit_code}
        for package in packages:
            write_file(os.path.join(self.root, "crates", package, "Cargo.toml"), f'[package]\nname = "{package}"\nversion = "0.1.0"\n')
            write_file(os.path.join(self.root, "crates", package, "src", "lib.rs"), f"// {package}\n")
        for relative, text in (files or {}).items():
            write_file(os.path.join(self.root, relative), text)
        for package, kind, name, sources in units:
            self.add_unit(package, kind, name, sources)
        self.tables(allowed, lag)
        subprocess.run(["git", "init", "-q", self.root], check=True)
        write_sample_tools(self.root)

    def add_unit(self, package, kind, name, sources):
        """kind 是 custom-build 的照真 cargo 录构建脚本单元：产物与 .d 在 build/<包名>-<哈希>/，.d 叫 build_script_build-<哈希>.d，根文件是包目录下的 build.rs。
        sources 是读到的文件（写进录好的 .d）；None 是不写 .d；SAMPLE_SCANNED_DEP_INFO 是由假 cargo 照根里的源码现录。"""
        unit = {"package": package, "kind": [kind], "name": name, "src_path": f"crates/{package}/src/lib.rs"}
        if kind == BUILD_SCRIPT_KIND:
            digest = sha256_of_text(f"{package}-{len(self.scenario['units'])}")[:16]
            unit.update(src_path=f"crates/{package}/build.rs", build_directory=f"{package}-{digest}", deps_name=f"{name.replace('-', '_')}-{digest}")
            products = [f"@TARGET@/debug/build/{unit['build_directory']}/{unit['deps_name']}.d", f"@TARGET@/debug/build/{unit['build_directory']}/{unit['deps_name']}"]
        else:
            unit["deps_name"] = f"{name}-{len(self.scenario['units']):04d}"
            products = [f"@TARGET@/debug/deps/lib{unit['deps_name']}.rmeta", f"@TARGET@/debug/deps/{unit['deps_name']}.d"]
        unit["dep_info"] = None
        if sources == SAMPLE_SCANNED_DEP_INFO:
            unit["dep_info"] = SAMPLE_SCANNED_DEP_INFO
        elif sources is not None:
            unit["dep_info"] = f"dep-info/{unit['deps_name']}.d"
            listed = " ".join(source.replace(" ", "\\ ") for source in sources)
            write_file(os.path.join(self.root, SAMPLE_DIRECTORY, unit["dep_info"]),
                       "".join(f"{product}: {listed}\n\n" for product in products)
                       + "".join(f"{source.replace(' ', chr(92) + ' ')}:\n" for source in sources))
        self.scenario["units"].append(unit)
        self.save()

    def save(self):
        write_file(os.path.join(self.root, SAMPLE_DIRECTORY, "scenario.json"), json.dumps(self.scenario, ensure_ascii=False, indent=1))

    def tables(self, allowed, lag):
        for table, rows in ((ALLOWED_TABLE, allowed), (LAG_TABLE, lag)):
            write_file(os.path.join(self.root, table), "# 自证的表\n" + "".join(f"{package}\t{target}\t{REASON}\n" for package, target in rows))

    def check(self, extra=None, without=(), arguments=()):
        environment = dict(os.environ)
        environment["PATH"] = os.path.join(self.root, SAMPLE_TOOLS_DIRECTORY) + os.pathsep + environment.get("PATH", "")
        environment["CARGO_TARGET_DIR"] = self.target
        environment[SAMPLE_CHECK_LOG_VARIABLE] = self.log
        environment.pop("RUSTC", None)
        environment.update(extra or {})
        for name in without:
            environment.pop(name, None)
        completed = subprocess.run([sys.executable, os.path.realpath(__file__), "check", self.root, *arguments], cwd=self.root, env=environment,
                                   capture_output=True, text=True, errors="surrogateescape")
        return completed.returncode, completed.stdout + completed.stderr

    def compiled(self):
        """假 cargo 每一趟 check 点名的包（一趟一行，照次序）；一趟都没起交空表。"""
        try:
            with open(self.log, encoding="utf-8") as handle:
                return [line.split() for line in handle.read().split("\n")[:-1]]
        except OSError:
            return []


def two_packages(work, name, consumer_sources, **extra):
    return SampleTree(work, name, ["consumer", "provider"],
                      [("consumer", "test", "cross", ["crates/consumer/src/lib.rs"] + consumer_sources),
                       ("provider", "lib", "provider", ["crates/provider/src/lib.rs"])], **extra)


def expect(failures, condition, detail):
    if not condition:
        failures.append(detail)


def cell_dep_info_parse(work):
    failures = []
    sources, variables = parse_dep_info("/t/debug/deps/libx-1.rmeta: crates/x/src/lib.rs crates/x/src/a\\ b.rs /abs/y.rs\n\n"
                                        "/t/debug/deps/x-1.d: crates/x/src/lib.rs crates/x/src/a\\ b.rs /abs/y.rs\n\n"
                                        "crates/x/src/lib.rs:\ncrates/x/src/a\\ b.rs:\n/abs/y.rs:\n\n# env-dep:CARGO_MANIFEST_DIR=/r/crates/x\n# env-dep:SINGLEFS_SAMPLE\n")
    expect(failures, sources == ["crates/x/src/lib.rs", "crates/x/src/a b.rs", "/abs/y.rs"], f"读到的文件是 {sources}")
    expect(failures, variables == ["CARGO_MANIFEST_DIR", "SINGLEFS_SAMPLE"], f"变量是 {variables}")
    return failures


def cell_artifact_dep_info_path(work):
    failures = []
    got = dep_info_candidates("/t/debug/deps/libalpha-ab12.rmeta", ["lib"])
    expect(failures, got == ["/t/debug/deps/alpha-ab12.d", "/t/debug/deps/libalpha-ab12.d"], f".rmeta 的候选是 {got}")
    got = dep_info_candidates("/t/debug/build/alpha-core-96e561e03ae0a7cf/build-script-build", ["custom-build"])
    expect(failures, got == ["/t/debug/build/alpha-core-96e561e03ae0a7cf/build_script_build-96e561e03ae0a7cf.d"], f"构建脚本的候选是 {got}")
    return failures


def cell_build_script_dep_info_read(work):
    tree = SampleTree(work, "build-script-dep-info-read", ["consumer", "provider"],
                      [("consumer", "lib", "consumer", ["crates/consumer/src/lib.rs"]),
                       ("consumer", BUILD_SCRIPT_KIND, "build-script-build", ["crates/consumer/build.rs", "crates/consumer/../provider/tests/c/mod.rs"]),
                       ("provider", "lib", "provider", ["crates/provider/src/lib.rs"])],
                      files={"crates/consumer/build.rs": "fn main() {}\n", PROVIDER_CROSS: "pub fn c() {}\n"})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    expect(failures, f"crates/consumer  引 {PROVIDER_CROSS}（{BUILD_SCRIPT_KIND}:build-script-build；跨包源码）" in output,
           f"没有点名构建脚本单元读的 {PROVIDER_CROSS}（.d 在 build/<包名>-<哈希>/build_script_build-<哈希>.d）")
    expect(failures, "找不到 dep-info" not in output, "构建脚本单元的 dep-info 没找到")
    return failures


def cell_cross_package_red(work):
    tree = two_packages(work, "cross-package-red", ["crates/consumer/tests/../../provider/tests/c/mod.rs"])
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1")
    expect(failures, f"crates/consumer  引 {PROVIDER_CROSS}（test:cross" in output, "没有点名 crates/consumer  引 crates/provider/tests/c/mod.rs")
    return failures


def cell_allowed_row_green(work):
    tree = two_packages(work, "allowed-row-green", ["crates/consumer/tests/../../provider/tests/c/mod.rs"], allowed=[("crates/consumer", PROVIDER_CROSS)])
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_GREEN, f"退 {code}，应退 0；输出：{output.strip()[-400:]}")
    expect(failures, "跨包源码与不属任何包的文件 1 处都登在放行表（1 行）" in output, "成功句里没有「1 处都登在放行表（1 行）」")
    return failures


def cell_table_rows_keyed_by_package(work):
    tree = two_packages(work, "table-rows-keyed-by-package", ["crates/consumer/tests/../../provider/tests/c/mod.rs"],
                        allowed=[("crates/consumer/src/lib.rs", PROVIDER_CROSS)])
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1")
    expect(failures, "第一列 crates/consumer/src/lib.rs 不是这一趟判到的包目录" in output, "没有点名那一行第一列不是判到的包")
    expect(failures, f"crates/consumer  引 {PROVIDER_CROSS}" in output, "没有点名那一处没登记")
    return failures


def cell_generated_and_dependency_allowed(work):
    build = {"package": "consumer", "directory_name": "consumer-0042", "output": ["cargo::rerun-if-changed=build.rs"], "generated": {"generated.rs": "pub const X: u32 = 1;\n"}}
    tree = two_packages(work, "generated-and-dependency-allowed",
                        ["@OUT_DIR:consumer@/generated.rs", f"@ROOT@/{SAMPLE_DIRECTORY}/registry/sample-dependency-1.0.0/src/lib.rs",
                         "crates/consumer/src/../../provider/Cargo.toml"],
                        build_scripts=[build], files={"crates/consumer/src/lib.rs": 'pub const PROVIDER_MANIFEST: &str = include_str!("../../provider/Cargo.toml");\n'})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_GREEN, f"退 {code}，应退 0；输出：{output.strip()[-400:]}")
    for phrase in ("构建脚本生成 1", "依赖源码 1", "别的包的数据文件 1"):
        expect(failures, phrase in output, f"成功句里没有「{phrase}」")
    return failures


def cell_unpackaged_source_red(work):
    tree = two_packages(work, "unpackaged-source-red", ["crates/consumer/src/../../../shared/common.rs"], files={"shared/common.rs": "pub fn f() {}\n"})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1")
    expect(failures, "crates/consumer  引 shared/common.rs" in output, "没有点名 shared/common.rs")
    return failures


def cell_unpackaged_any_suffix_red(work):
    tree = two_packages(work, "unpackaged-any-suffix-red", ["crates/consumer/src/../../../shared/snippet.inc", "crates/consumer/src/../../../notes/table.txt"],
                        files={"shared/snippet.inc": "pub fn from_unpackaged_inc() {}\n", "notes/table.txt": "31\n",
                               "crates/consumer/src/lib.rs": 'include!("../../../shared/snippet.inc");\npub const TABLE: &str = include_str!("../../../notes/table.txt");\n'})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    for target in ("shared/snippet.inc", "notes/table.txt"):
        expect(failures, f"crates/consumer  引 {target}（test:cross；仓里不属任何包的文件）" in output, f"没有点名 {target}（仓里不属任何包的文件，不论后缀判红）")
    return failures


def cell_module_with_data_suffix_red(work):
    tree = two_packages(work, "module-with-data-suffix-red", ["crates/consumer/src/../../provider/src/code.md", "crates/consumer/src/../../provider/src/notes.md"],
                        files={"crates/provider/src/code.md": "pub fn from_markdown() {}\n", "crates/provider/src/notes.md": "提供包的说明\n",
                               "crates/consumer/src/lib.rs": ('#[path = "../../provider/src/code.md"]\nmod from_markdown;\n'
                                                              'pub const NOTES: &str = include_str!("../../provider/src/notes.md");\n')})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    expect(failures, "crates/consumer  引 crates/provider/src/code.md（test:cross；跨包源码，后缀是数据后缀" in output,
           "没有点名当模块编进来的 crates/provider/src/code.md（跨包源码）")
    expect(failures, "引 crates/provider/src/notes.md" not in output, "include_str! 读的 crates/provider/src/notes.md 被点名了（应算别的包的数据文件）")
    expect(failures, "别的包的数据文件 1" in output, "那一包的那一行没有「别的包的数据文件 1」")
    return failures


def cell_out_dir_copy_red(work):
    copied, own = "pub fn copied() -> u32 { 7 }\n", "pub fn own() -> u32 { 5 }\n"
    build = {"package": "consumer", "directory_name": "consumer-5e7a2c41d09b8f63", "output": ["cargo::rerun-if-changed=build.rs"],
             "generated": {"copied.rs": copied, "own.rs": own, "fresh.rs": "pub const FRESH: u32 = 3;\n"}}
    tree = two_packages(work, "out-dir-copy-red", ["@OUT_DIR:consumer@/copied.rs", "@OUT_DIR:consumer@/own.rs", "@OUT_DIR:consumer@/fresh.rs"],
                        build_scripts=[build], files={"crates/provider/src/x.rs": copied, "crates/consumer/templates/own.rs": own})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    expect(failures, "crates/consumer  引 OUT_DIR:consumer/copied.rs（test:cross；构建脚本拷进 OUT_DIR 的别的包文件，内容与 crates/provider/src/x.rs 相同）" in output,
           "没有点名与提供包 src/x.rs 逐字节相同的 OUT_DIR:consumer/copied.rs")
    for name in ("own.rs", "fresh.rs"):
        expect(failures, f"引 OUT_DIR:consumer/{name}" not in output, f"OUT_DIR:consumer/{name} 被点名了（与自己包里的一份相同或谁都不同，应放行）")
    expect(failures, "构建脚本生成 2" in output, "那一包的那一行没有「构建脚本生成 2」")
    return failures


def cell_build_script_runtime_read_red(work):
    build = {"package": "consumer", "directory_name": "consumer-9b1d4e6f2a8c0753",
             "output": ["cargo::rerun-if-changed=build.rs", "cargo::rerun-if-changed=../provider/src/k.wgsl", "cargo::rerun-if-changed=../provider/Cargo.toml",
                        "cargo::rerun-if-changed=../../notes/table.txt"]}
    tree = two_packages(work, "build-script-runtime-read-red", [], build_scripts=[build],
                        files={"crates/consumer/build.rs": "fn main() {}\n", "crates/provider/src/k.wgsl": "fn kernel() {}\n", "notes/table.txt": "31\n"})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    expect(failures, f"crates/consumer  引 crates/provider/src/k.wgsl（{BUILD_INPUT_UNIT}；构建脚本运行时读的别的包文件）" in output,
           "没有点名构建脚本报的 rerun-if-changed=../provider/src/k.wgsl")
    for target in ("crates/provider/Cargo.toml", "notes/table.txt"):
        expect(failures, f"引 {target}" not in output, f"{target} 被点名了（别的包的数据文件与不属任何包的构建输入都不判）")
    return failures


def cell_package_prefix_row_green(work):
    build = {"package": "consumer", "directory_name": "consumer-4d7a0e2c6b913f58",
             "output": ["cargo::rerun-if-changed=build.rs", "cargo::rerun-if-changed=../provider/src/k.wgsl", "cargo::rerun-if-changed=../provider/src/shared.rs",
                        "cargo::rerun-if-changed=../provider/Cargo.toml"]}
    tree = two_packages(work, "package-prefix-row-green", [], build_scripts=[build], allowed=[("crates/consumer", PROVIDER_PREFIX)],
                        files={"crates/provider/src/k.wgsl": "fn kernel() {}\n", "crates/provider/src/shared.rs": "pub fn shared() {}\n"})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_GREEN, f"退 {code}，应退 0；输出：{output.strip()[-400:]}")
    expect(failures, "跨包源码与不属任何包的文件 2 处都登在放行表（1 行）" in output, "成功句里没有「2 处都登在放行表（1 行）」（k.wgsl 与 shared.rs 两处都该被前缀行放行）")
    return failures


def cell_package_prefix_row_not_for_compiled_in_or_copies(work):
    copied = "pub fn copied() -> u32 { 7 }\n"
    build = {"package": "consumer", "directory_name": "consumer-8e3b5a1f7c0d2469", "generated": {"copied.rs": copied},
             "output": ["cargo::rerun-if-changed=build.rs", "cargo::rerun-if-changed=../provider/src/k.wgsl", "cargo::rerun-if-changed=../provider/tests/c/mod.rs"]}
    tree = two_packages(work, "package-prefix-row-not-for-compiled-in-or-copies",
                        ["crates/consumer/tests/../../provider/tests/c/mod.rs", "@OUT_DIR:consumer@/copied.rs"],
                        build_scripts=[build], allowed=[("crates/consumer", PROVIDER_PREFIX)],
                        files={"crates/provider/src/k.wgsl": "fn kernel() {}\n", "crates/provider/src/x.rs": copied, PROVIDER_CROSS: "pub fn c() {}\n"})
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    expect(failures, "crates/ 下有 2 处编译单元读了别的包目录下的源码或不属任何包的文件，两张表里都没登：" in output,
           "没有点名「2 处」没登（编进编译单元的 mod.rs 与拷进 OUT_DIR 的 copied.rs；k.wgsl 由前缀行放行）")
    expect(failures, f"crates/consumer  引 {PROVIDER_CROSS}（test:cross、{BUILD_INPUT_UNIT}；跨包源码）" in output,
           "没有点名编进编译单元的 mod.rs（构建脚本也报了它的 rerun-if-changed，照旧是跨包源码）")
    expect(failures, "crates/consumer  引 OUT_DIR:consumer/copied.rs（test:cross；构建脚本拷进 OUT_DIR 的别的包文件，内容与 crates/provider/src/x.rs 相同）" in output,
           "没有点名拷进 OUT_DIR 的 copied.rs")
    expect(failures, "引 crates/provider/src/k.wgsl" not in output, "只由构建脚本报 rerun-if-changed 的 k.wgsl 被点名了（该由前缀行放行）")
    expect(failures, "两张表有" not in output, "前缀行被报用不着（k.wgsl 落在它里面，它用得着）")
    return failures


def cell_stale_package_prefix_rows_red(work):
    build = {"package": "consumer", "directory_name": "consumer-0a6f2d9c4e7b1835",
             "output": ["cargo::rerun-if-changed=build.rs", "cargo::rerun-if-changed=../provider/src/k.wgsl"]}
    tree = SampleTree(work, "stale-package-prefix-rows-red", ["consumer", "provider", "bystander"],
                      [("consumer", "test", "cross", ["crates/consumer/src/lib.rs"]), ("provider", "lib", "provider", ["crates/provider/src/lib.rs"]),
                       ("bystander", "lib", "bystander", ["crates/bystander/src/lib.rs"])],
                      build_scripts=[build], files={"crates/provider/src/k.wgsl": "fn kernel() {}\n"},
                      allowed=[("crates/consumer", PROVIDER_PREFIX), ("crates/consumer", "crates/absent/"), ("crates/consumer", "crates/bystander/")])
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1；输出：{output.strip()[-400:]}")
    expect(failures, "两张表有 2 行写不对或用不着了：" in output, "没有点名「2 行写不对或用不着了」")
    expect(failures, f"{ALLOWED_TABLE} 第 3 行：crates/consumer 的前缀行 crates/absent/ 不是别的包的目录" in output, "没有点名第 3 行（目录不在）")
    expect(failures, f"{ALLOWED_TABLE} 第 4 行：crates/consumer 的前缀行 crates/bystander/ 这一趟没有一处{BUILD_INPUT_UNIT} 落在它里面" in output,
           "没有点名第 4 行（是一个包的目录、没有 rerun-if-changed 落在里面）")
    expect(failures, f"{ALLOWED_TABLE} 第 2 行" not in output, "用得着的第 2 行（crates/provider/）被点名了")
    expect(failures, "两张表里都没登" not in output, "k.wgsl 没被第 2 行放行")
    return failures


def cell_stale_and_lag_rows_red(work):
    tree = two_packages(work, "stale-and-lag-rows-red", ["crates/consumer/tests/../../provider/tests/c/mod.rs"],
                        allowed=[("crates/consumer", "crates/provider/src/gone.rs")], lag=[("crates/consumer", PROVIDER_CROSS)])
    base = os.path.join(work, "stale-and-lag-rows-red-base.tsv")
    write_file(base, "# 基准那一版：没有数据行\n")
    code, output = tree.check(arguments=["--lag-base-state", "table", "--lag-base-file", base, "--lag-base-name", "selftest-base"])
    failures = []
    expect(failures, code == EXIT_RED, f"退 {code}，应退 1")
    expect(failures, "这一趟没有读 crates/provider/src/gone.rs" in output, "没有点名放行表那一行用不着了")
    expect(failures, "多出 1 行（只缩不涨）" in output, "没有点名滞后表涨了")
    return failures


def cell_compile_failure_red(work):
    tree = two_packages(work, "compile-failure-red", [], exit_code=101)
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED and "编不过" in output, f"退 {code}，应退 1 并点名编不过")
    code, output = tree.check()
    expect(failures, "· crates/consumer：红，现判" in output and "· crates/provider：红，现判" in output, "第二趟两个包没有照现判（编不过的那一趟入了库）")
    return failures


def cell_missing_dep_info_red(work):
    tree = two_packages(work, "missing-dep-info-red", [])
    tree.add_unit("consumer", "bin", "tool", None)
    code, output = tree.check()
    failures = []
    expect(failures, code == EXIT_RED and "找不到 dep-info" in output and "bin:tool" in output, f"退 {code}，应退 1 并点名 bin:tool 找不到 dep-info")
    return failures


def cell_target_directory_required(work):
    tree = two_packages(work, "target-directory-required", [])
    failures = []
    code, output = tree.check(without=["CARGO_TARGET_DIR"])
    expect(failures, code == EXIT_RED and "CARGO_TARGET_DIR" in output, f"没设 CARGO_TARGET_DIR：退 {code}，应退 1 并点名它")
    code, output = tree.check(extra={"CARGO_TARGET_DIR": os.path.join(tree.root, "target")})
    expect(failures, code == EXIT_RED and "落在被判的根" in output, f"编译目录设在根里：退 {code}，应退 1 并点名它落在根里")
    expect(failures, tree.compiled() == [], f"假 cargo 起过 check：{tree.compiled()}")
    return failures


def cell_sample_root_needs_temporary_service(work):
    tree = two_packages(work, "sample-root-needs-temporary-service", [])
    write_file(os.path.join(tree.root, ".gate-cells"), "crate-cross-references\n")
    code, output = tree.check(without=[prune_client().TEMPORARY_SERVICE_VARIABLE])
    failures = []
    expect(failures, code == EXIT_USAGE and "样本不写真服务" in output, f"退 {code}，应退 2 并说样本不写真服务")
    return failures


def three_packages(work, name):
    return SampleTree(work, name, ["alpha", "beta", "gamma"],
                      [("alpha", "lib", "alpha", ["crates/alpha/src/lib.rs", "crates/alpha/src/shared.rs"]),
                       ("beta", "test", "beta", ["crates/beta/src/lib.rs", "crates/beta/tests/../../alpha/src/shared.rs"]),
                       ("gamma", "lib", "gamma", ["crates/gamma/src/lib.rs"])],
                      files={"crates/alpha/src/shared.rs": "pub fn shared() -> u32 { 1 }\n", "docs/notes.md": "说明\n"},
                      allowed=[("crates/beta", "crates/alpha/src/shared.rs")])


def cell_reuse_three_questions(work):
    tree = three_packages(work, "reuse-three-questions")
    sentinel = {"SINGLEFS_REUSE_SENTINEL_COUNT": "1", "SINGLEFS_REUSE_SENTINEL_SEED": "7"}
    checks = reuse_checks()
    runs, failures = [], []
    for change in (None, None, ("docs/notes.md", "说明改了一句\n"), ("crates/alpha/src/shared.rs", "pub fn shared() -> u32 { 2 }\n")):
        if change:
            write_file(os.path.join(tree.root, change[0]), change[1])
        code, output = tree.check(extra=sentinel)
        expect(failures, code == EXIT_GREEN, f"第 {len(runs) + 1} 趟退 {code}，应退 0；输出：{output.strip()[-300:]}")
        runs.append(checks.states_from_output(output))
    compiled = tree.compiled()
    expect(failures, len(compiled) >= 1 and compiled[0] == ["alpha", "beta", "gamma"], f"空库那一趟点名的包是 {compiled[:1]}")
    expect(failures, all(set(words) <= {"alpha", "beta", "gamma"} for words in compiled) and len(compiled) >= 2 and set(compiled[-1]) >= {"alpha", "beta"},
           f"后几趟点名的包是 {compiled[1:]}（改 alpha 的那一趟要点名 alpha、beta）")
    passed, problems = checks.report_three_questions(SCRIPT_RELATIVE, *runs, {"crates/alpha", "crates/beta"}, sentinels=1)
    if not passed:
        failures.extend(problems)
    return failures


def cell_reuse_sentinel(work):
    tree = three_packages(work, "reuse-sentinel")
    sentinel = {"SINGLEFS_REUSE_SENTINEL_COUNT": "1", "SINGLEFS_REUSE_SENTINEL_SEED": "11"}
    failures = []
    code, output = tree.check(extra=sentinel)
    expect(failures, code == EXIT_GREEN, f"空库那一趟退 {code}，应退 0")
    for unit in tree.scenario["units"]:
        path = os.path.join(tree.root, SAMPLE_DIRECTORY, unit["dep_info"])
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        write_file(path, text.replace(".rs\n", ".rs crates/notes.txt\n", 2))
    code, output = tree.check(extra=sentinel)
    found = re.search(r"强制现判 1 条：(\S+)", output)
    expect(failures, code == EXIT_RED and "判定不稳" in output and found, f"改了录好的 .d 之后退 {code}，应退 1 并报判定不稳")
    if found:
        forced = found.group(1)
        code, output = tree.check(extra=sentinel)
        expect(failures, f"· {forced}：" in output and re.search(rf"· {re.escape(forced)}：(绿|红)，现判", output),
               f"判定不稳之后那一包（{forced}）下一趟没有现判")
    return failures


def facts_from_compile_directory(root, target):
    """key-independent-of-compile-directory 用：<root>/crates/alpha 照编译目录 <target> 录一份 cargo check 的输出交给 recorded_package_facts
    （OUT_DIR 里生成 gen.rs、库的 .d 读它；构建脚本报根里的绝对路径、OUT_DIR 里的 gen.rs 与 rerun-if-env-changed=HOME、CARGO_HOME），交 alpha 的事实。"""
    package_directory = os.path.join(root, "crates", "alpha")
    for relative, text in (("Cargo.toml", '[package]\nname = "alpha"\nversion = "0.1.0"\n'), ("build.rs", "fn main() {}\n"),
                           ("src/lib.rs", 'include!(concat!(env!("OUT_DIR"), "/gen.rs"));\n')):
        write_file(os.path.join(package_directory, relative), text)
    build = os.path.join(target, "debug", "build", "alpha-5d3c0b7e9a214f68")
    out_directory = os.path.join(build, "out")
    write_file(os.path.join(out_directory, "gen.rs"), "pub const GENERATED: u32 = 1;\n")
    write_file(os.path.join(build, "output"), "".join(line + "\n" for line in (
        f"cargo:rerun-if-changed={package_directory}/build.rs", f"cargo:rerun-if-changed={out_directory}/gen.rs",
        "cargo:rerun-if-env-changed=HOME", "cargo:rerun-if-env-changed=CARGO_HOME")))
    deps = os.path.join(target, "debug", "deps")
    sources = f"{package_directory}/src/lib.rs {out_directory}/gen.rs"
    write_file(os.path.join(deps, "alpha-0001.d"), f"{deps}/libalpha-0001.rmeta: {sources}\n\n{deps}/alpha-0001.d: {sources}\n\n# env-dep:OUT_DIR={out_directory}\n")
    identifier = f"alpha 0.1.0 (path+file://{package_directory})"
    messages = [{"reason": "build-script-executed", "package_id": identifier, "out_dir": out_directory},
                {"reason": "compiler-artifact", "package_id": identifier, "filenames": [f"{deps}/libalpha-0001.rmeta"],
                 "target": {"kind": ["lib"], "name": "alpha", "src_path": f"{package_directory}/src/lib.rs"}}]
    packages = {identifier: {"name": "alpha", "version": "0.1.0", "directory": package_directory, "relative": "crates/alpha", "source": None}}
    facts, _missing = recorded_package_facts("\n".join(json.dumps(message) for message in messages), packages, root, {"crates/alpha"})
    return facts.get("crates/alpha")


def cell_key_independent_of_compile_directory(work):
    failures = []
    work = os.path.realpath(work)
    sides = []
    for root, target, home in ((os.path.join(work, "root"), os.path.join(work, "target"), os.path.join(work, "home-local")),
                               (os.path.join(work, "copy", "nested-root"), os.path.join(work, "another-host", "builds", "task-target"), os.path.join(work, "home-other"))):
        sides.append((root, facts_from_compile_directory(root, target), home))
    (root_local, fact_local, home_local), (root_other, fact_other, home_other) = sides
    expect(failures, fact_local is not None and fact_local == fact_other, f"两个编译目录录出的事实不同：{fact_local} 与 {fact_other}")
    if fact_local is None or fact_other is None:
        return failures
    expect(failures, fact_local["build"]["inputs"] == ["OUT_DIR:alpha/gen.rs", "crates/alpha/build.rs"], f"构建脚本的输入是 {fact_local['build']['inputs']}")
    key_local, parts_local = package_key(root_local, "crates/alpha", fact_local, "selftest-judge", {"HOME": home_local})
    key_other, parts_other = package_key(root_other, "crates/alpha", fact_other, "selftest-judge", {"HOME": home_other})
    expect(failures, key_local == key_other, f"换了根、编译目录与家目录，键不同；变了的成分：{changed_components(parts_local, parts_other)}")
    return failures


def hosts_tree(work, name):
    """reuse-across-hosts 与 key-unchanged-by-toolchain-and-cargo-configuration 用的一棵树：三个包、各一个库单元、.d 由假 cargo 照源码现录；
    alpha 的构建脚本往 OUT_DIR 写 gen.rs、报根里的绝对路径与 HOME、CARGO_HOME、RUSTFLAGS、CARGO_PROFILE_DEV_DEBUG_ASSERTIONS 四个构建环境变量。"""
    build = {"package": "alpha", "directory_name": "alpha-77d0c1e5a9b3f246", "generated": {"gen.rs": "pub const GENERATED: u32 = 1;\n"},
             "output": ["cargo:rerun-if-changed=@ROOT@/crates/alpha/build.rs", "cargo:rerun-if-changed=@OUT_DIR:alpha@/gen.rs"]
             + [f"cargo:rerun-if-env-changed={name}" for name in ("HOME", "CARGO_HOME", "RUSTFLAGS", "CARGO_PROFILE_DEV_DEBUG_ASSERTIONS")]}
    return SampleTree(work, name, ["alpha", "beta", "gamma"],
                      [(package, "lib", package, SAMPLE_SCANNED_DEP_INFO) for package in ("alpha", "beta", "gamma")], build_scripts=[build],
                      files={"crates/alpha/build.rs": "fn main() {}\n",
                             "crates/alpha/src/lib.rs": 'mod shared;\ninclude!(concat!(env!("OUT_DIR"), "/gen.rs"));\npub fn alpha() -> u32 { shared::value() + GENERATED }\n',
                             "crates/alpha/src/shared.rs": "pub fn value() -> u32 { 1 }\n",
                             "crates/beta/src/lib.rs": '#[path = "inner/kept.rs"]\nmod kept;\npub use kept::kept;\n',
                             "crates/beta/src/inner/kept.rs": "pub fn kept() -> u32 { 2 }\n",
                             "crates/gamma/src/lib.rs": 'pub const DOCS: &str = include_str!("../README.md");\n',
                             "crates/gamma/README.md": "伽马包的说明\n"})


def cell_reuse_across_hosts(work):
    work = os.path.realpath(work)
    # 种子 6：三个包键都相同时抽 gamma 当哨兵、强制现判 alpha（它的事实里有 OUT_DIR 里的文件与 HOME，读库的根现录的要与写库的根写的相同）
    sentinel = {"SINGLEFS_REUSE_SENTINEL_COUNT": "1", "SINGLEFS_REUSE_SENTINEL_SEED": "6"}
    writer = hosts_tree(work, "host-writing")
    reader = hosts_tree(work, os.path.join("host-reading", "deeper", "repository"))
    shutil.rmtree(os.path.join(reader.root, ".git"))
    write_file(os.path.join(reader.root, ".git"), f"gitdir: {os.path.join(writer.root, '.git')}\n")
    homes = [os.path.join(work, "home-of-the-writing-host"), os.path.join(work, "home-of-the-reading-host")]
    for home in homes:
        os.makedirs(home, exist_ok=True)
    failures = []
    code, output = writer.check(extra=dict(sentinel, HOME=homes[0]))
    expect(failures, code == EXIT_GREEN and "单元键：现判 3、复用 0、强制现判 0" in output,
           f"写库那一台退 {code}，应退 0 并现判 3 包；输出：{output.strip()[-400:]}")
    code, output = reader.check(extra=dict(sentinel, HOME=homes[1]))
    expect(failures, code == EXIT_GREEN and "单元键：现判 0、复用 2、强制现判 1" in output,
           f"读库那一台（另一个根、编译目录与家目录）退 {code}，应退 0 并「现判 0、复用 2、强制现判 1」；输出：{output.strip()[-500:]}")
    expect(failures, "强制现判 1 条：crates/alpha" in output, "读库那一台强制现判的不是 crates/alpha（种子 6 的抽法变了，换一个抽中 alpha 的种子）")
    expect(failures, "判定不稳" not in output, "读库那一台现录的 alpha 的事实与写库那一台写的不同（判定不稳）")
    expect(failures, reader.compiled() == [["alpha"]], f"读库那一台的假 cargo 点名的包是 {reader.compiled()}，应只有强制现判的 alpha")
    return failures


OTHER_TOOLCHAIN_RUSTC = '''#!/usr/bin/env bash
# key-unchanged-by-toolchain-and-cargo-configuration 换上的另一份假 rustc：版本不同、host 相同
case "${1:-}" in
  -V|--version) echo "rustc 2.0.0-crate-cross-references-other-toolchain" ;;
  -vV) printf 'rustc 2.0.0-crate-cross-references-other-toolchain\\nbinary: rustc\\ncommit-hash: other\\nhost: x86_64-unknown-linux-gnu\\n' ;;
  *) echo "假 rustc 只答 -V 与 -vV" >&2; exit 2 ;;
esac
'''


def cell_key_unchanged_by_toolchain_and_cargo_configuration(work):
    work = os.path.realpath(work)
    # 种子 6 同 reuse-across-hosts：三个包都可复用时强制现判 alpha（它的事实里有构建脚本报的四个构建环境变量）
    sentinel = {"SINGLEFS_REUSE_SENTINEL_COUNT": "1", "SINGLEFS_REUSE_SENTINEL_SEED": "6"}
    tree = hosts_tree(work, "toolchain-and-configuration")
    home_before, home_after, cargo_home = (os.path.join(work, name) for name in ("home-before", "home-after", "cargo-home-after"))
    os.makedirs(home_before, exist_ok=True)
    failures = []
    code, output = tree.check(extra=dict(sentinel, HOME=home_before), without=("CARGO_HOME", "RUSTFLAGS", "CARGO_PROFILE_DEV_DEBUG_ASSERTIONS"))
    expect(failures, code == EXIT_GREEN and "单元键：现判 3、复用 0、强制现判 0" in output, f"空库那一趟退 {code}，应退 0 并现判 3 包；输出：{output.strip()[-400:]}")
    rustc = os.path.join(work, "other-toolchain", "rustc")
    write_file(rustc, OTHER_TOOLCHAIN_RUSTC)
    os.chmod(rustc, 0o755)
    configuration = '[build]\nrustflags = ["--cfg", "selftest"]\n\n[profile.dev]\ndebug-assertions = false\n'
    for path, text in ((os.path.join(tree.root, "rust-toolchain.toml"), '[toolchain]\nchannel = "nightly"\n'),
                       (os.path.join(tree.root, ".cargo", "config.toml"), configuration),
                       (os.path.join(tree.root, "Cargo.lock"), 'version = 4\n\n[[package]]\nname = "sample-dependency"\nversion = "2.0.0"\n'),
                       (os.path.join(home_after, ".cargo", "config.toml"), configuration),
                       (os.path.join(cargo_home, "config.toml"), configuration)):
        write_file(path, text)
    code, output = tree.check(extra=dict(sentinel, HOME=home_after, CARGO_HOME=cargo_home, RUSTC=rustc, RUSTFLAGS="--cfg selftest",
                                         CARGO_PROFILE_DEV_DEBUG_ASSERTIONS="false"))
    expect(failures, code == EXIT_GREEN and "单元键：现判 0、复用 2、强制现判 1" in output,
           f"换了工具链与 cargo 配置之后退 {code}，应退 0 并「现判 0、复用 2、强制现判 1」；输出：{output.strip()[-500:]}")
    changed = [line.strip() for line in output.split("\n") if "键变了" in line]
    expect(failures, not changed, f"换了工具链与 cargo 配置之后有包报键变了：{changed[:3]}")
    expect(failures, "判定不稳" not in output, "强制现判的 alpha 判出的事实与库里那一条不同（判定不稳）")
    expect(failures, tree.compiled()[1:] == [["alpha"]], f"第二趟假 cargo 点名的包是 {tree.compiled()[1:]}，应只有强制现判的 alpha")
    return failures


def cell_host_triple_from_machine(work):
    work = os.path.realpath(work)
    failures = []
    for (system, machine, glibc), expected in ((("Linux", "x86_64", True), "x86_64-unknown-linux-gnu"), (("Linux", "aarch64", True), "aarch64-unknown-linux-gnu"),
                                               (("Darwin", "arm64", False), "aarch64-apple-darwin"), (("Darwin", "x86_64", False), "x86_64-apple-darwin"),
                                               (("Linux", "x86_64", False), None), (("Windows", "AMD64", False), None)):
        triple = host_triple(system, machine, glibc)
        expect(failures, triple == expected, f"系统 {system}、架构 {machine}、{'glibc' if glibc else '不是 glibc'} 的 host 是 {triple}，应是 {expected}")
    unlisted = Run(work, {})
    expect(failures, host_for_run(unlisted, "Windows", "AMD64", False) is None and len(unlisted.problems) == 1 and "HOST_TRIPLES" in unlisted.problems[0][0],
           f"表里没有的机器没有记一条点名 HOST_TRIPLES 的红句：{unlisted.problems}")
    here = Run(work, {})
    expect(failures, host_for_run(here, *machine_facts()) is not None and not here.problems, f"跑自证的这台机器不在 HOST_TRIPLES 里：{here.problems}")
    return failures


def cell_invalidated_key_names_changed_parts(work):
    tree = three_packages(work, "invalidated-key-names-changed-parts")
    sentinel = {"SINGLEFS_REUSE_SENTINEL_COUNT": "1", "SINGLEFS_REUSE_SENTINEL_SEED": "7"}
    failures = []
    code, output = tree.check(extra=sentinel)
    expect(failures, "· crates/alpha：" in output and "键变了" not in output, f"空库那一趟退 {code}，应有 alpha 那一行、没有一行报键变了；输出：{output.strip()[-300:]}")
    write_file(os.path.join(tree.root, "crates", "alpha", "src", "shared.rs"), "pub fn shared() -> u32 { 2 }\n")
    code, output = tree.check(extra=sentinel)
    for package, part in (("crates/alpha", "package-file"), ("crates/beta", "read")):
        line = next((line.strip() for line in output.split("\n") if f"· {package}：" in line), "没有这一行")
        expect(failures, re.search(rf"· {re.escape(package)}：(绿|红)，现判（键变了：{re.escape(part)}；", line), f"改了 crates/alpha/src/shared.rs 之后 {package} 应报「现判（键变了：{part}；」：{line[:240]}")
    return failures


SELFTEST_CELLS = (
    ("dep-info-parse", cell_dep_info_parse),
    ("artifact-dep-info-path", cell_artifact_dep_info_path),
    ("build-script-dep-info-read", cell_build_script_dep_info_read),
    ("cross-package-red", cell_cross_package_red),
    ("allowed-row-green", cell_allowed_row_green),
    ("table-rows-keyed-by-package", cell_table_rows_keyed_by_package),
    ("generated-and-dependency-allowed", cell_generated_and_dependency_allowed),
    ("unpackaged-source-red", cell_unpackaged_source_red),
    ("unpackaged-any-suffix-red", cell_unpackaged_any_suffix_red),
    ("module-with-data-suffix-red", cell_module_with_data_suffix_red),
    ("out-dir-copy-red", cell_out_dir_copy_red),
    ("build-script-runtime-read-red", cell_build_script_runtime_read_red),
    ("package-prefix-row-green", cell_package_prefix_row_green),
    ("package-prefix-row-not-for-compiled-in-or-copies", cell_package_prefix_row_not_for_compiled_in_or_copies),
    ("stale-package-prefix-rows-red", cell_stale_package_prefix_rows_red),
    ("stale-and-lag-rows-red", cell_stale_and_lag_rows_red),
    ("compile-failure-red", cell_compile_failure_red),
    ("missing-dep-info-red", cell_missing_dep_info_red),
    ("target-directory-required", cell_target_directory_required),
    ("sample-root-needs-temporary-service", cell_sample_root_needs_temporary_service),
    ("reuse-three-questions", cell_reuse_three_questions),
    ("reuse-sentinel", cell_reuse_sentinel),
    ("key-independent-of-compile-directory", cell_key_independent_of_compile_directory),
    ("reuse-across-hosts", cell_reuse_across_hosts),
    ("key-unchanged-by-toolchain-and-cargo-configuration", cell_key_unchanged_by_toolchain_and_cargo_configuration),
    ("host-triple-from-machine", cell_host_triple_from_machine),
    ("invalidated-key-names-changed-parts", cell_invalidated_key_names_changed_parts),
)


def selftest():
    client = prune_client()
    problem = client.selftest_service_problem()
    if problem:
        print(f"  ✗ 自证只在临时 prune-server 上跑：{problem}")
        print(f"     → 怎么办：{client.SELFTEST_SERVICE_REMEDY}：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- python3 {SCRIPT_RELATIVE} --selftest")
        return EXIT_USAGE
    heavy_threads = load_module("singlefs_heavy_threads", "scripts/hosts/heavy_threads.py")
    try:
        jobs = int(heavy_threads.default_heavy_threads())
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}")
        return EXIT_RED
    work = tempfile.mkdtemp(prefix="crate-cross-references-selftest-")
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=max(1, jobs)) as pool:
            futures = [(name, pool.submit(function, os.path.join(work, name.replace("-", "_")))) for name, function in SELFTEST_CELLS]
            results = []
            for name, future in futures:
                try:
                    results.append((name, future.result()))
                except Exception as error:  # 一格自己出错算那一格红，别的格照收
                    results.append((name, [f"自证那一格出错：{type(error).__name__}: {error}"]))
    finally:
        shutil.rmtree(work, ignore_errors=True)
    if len(results) != len(SELFTEST_CELLS):
        print(f"  ✗ 自证收回 {len(results)} 格，派出去 {len(SELFTEST_CELLS)} 格，整份作废")
        print("     → 怎么办：看上面哪一格没收回（线程池异常），修好再跑")
        return EXIT_RED
    red = [(name, failures) for name, failures in results if failures]
    for name, failures in red:
        print(f"  ✗ 自证 {name}：{'；'.join(failures)}")  # gate-lint:detail
    if red:
        print(f"     → 怎么办：照上面逐格写的改 {SCRIPT_RELATIVE}；{BREAK_VARIABLE} 设着的话点名那几格本来就该红")
        return EXIT_RED
    print(f"  ✓ {SCRIPT_RELATIVE} 自证 {len(results)} 格都过（并行 {max(1, jobs)} 路）：dep-info 的读法与产物到 .d 的候选、构建脚本单元的 .d、跨包源码判红与放行表、表按包目录认、"
          f"构建脚本生成与依赖源码与只当数据读的数据放行、不属任何包的文件不论后缀判红、数据后缀当模块编进来判红、拷进 OUT_DIR 的别的包文件判红、"
          f"构建脚本运行时读别的包判红、放行表的前缀行放行构建脚本报的 rerun-if-changed 而放不过编进编译单元的与拷进 OUT_DIR 的、用不着的前缀行判红、表里用不着的行与滞后表涨了判红、编不过与找不到 dep-info 判红、编译目录只认任务服务给的、"
          f"样本根要临时服务、复用三问与复用哨兵、键不随根与编译目录与家目录变、一台写库另一台读库照复用、换工具链与 cargo 配置与构建环境变量键不变、host 三元组照机器取、作废时报键里变了哪几样")
    return EXIT_GREEN


# ── 入口 ──

def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；门禁样本根（当前目录有 .gate-cells 或 .gate-break-sample、没设 GATE_STAGED_FROM）里由门禁转来的；祖先里有自证。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    sampled = (os.path.exists(".gate-cells") or os.path.exists(".gate-break-sample")) and not os.environ.get("GATE_STAGED_FROM")
    if not setting or arguments[:1] == ["--selftest"] or sampled or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）和门禁样本根里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一格，跑 python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- "
          f"env {setting[0]}=<项> python3 {SCRIPT_RELATIVE} --selftest", file=sys.stderr)
    return EXIT_USAGE


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if len(arguments) == 2 and arguments[0] == "write-sample-tools":
        print(f"  · 假工具写进 {write_sample_tools(os.path.realpath(arguments[1]))}")
        return EXIT_GREEN
    if len(arguments) >= 2 and arguments[0] == "check":
        options = {"--lag-base-state": "none", "--lag-base-file": "", "--lag-base-name": ""}
        rest = arguments[2:]
        while rest:
            if len(rest) < 2 or rest[0] not in options:
                break
            options[rest[0]], rest = rest[1], rest[2:]
        if not rest and os.path.isdir(arguments[1]):
            return run_check(arguments[1], options["--lag-base-state"], options["--lag-base-file"], options["--lag-base-name"])
    print(f"  ✗ 用法不对：{' '.join(arguments) or '（没给参数）'}")
    print(f"     → 怎么办：{SCRIPT_RELATIVE} check <根> [--lag-base-state <table|none|unreadable:<原因>>] [--lag-base-file <文件>] [--lag-base-name <基准名>]；"
          "write-sample-tools <根>；--selftest（文件头「命令行」）")
    return EXIT_USAGE


if __name__ == '__main__':
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
